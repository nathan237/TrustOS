#!/usr/bin/env python3
"""
A12 TOCTOU — All-in-one: detect DFU + run test in same process.
Also DISABLES Apple services (not just stops them).
"""
import time, os, json, struct, statistics
from datetime import datetime
from pathlib import Path
import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
BE = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)

def kill_apple():
    for p in ["iTunesHelper","iTunes","AppleMobileDeviceService",
              "usbmuxd","AMPDeviceDiscoveryAgent","AppleMobileDeviceHelper"]:
        os.system(f'taskkill /F /IM "{p}.exe" >nul 2>&1')
    # DISABLE the service so it can't restart
    os.system('sc config "Apple Mobile Device Service" start=disabled >nul 2>&1')
    os.system('net stop "Apple Mobile Device Service" >nul 2>&1')

def find_dfu():
    return usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=BE)

def get_status(dev):
    try:
        r = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=5000)
        return (r[4], r[0], r[1]|(r[2]<<8)|(r[3]<<16)) if len(r)>=6 else None
    except: return None

def dnload(dev, data, timeout=5000):
    try:
        dev.ctrl_transfer(0x21, 1, 0, 0, data, timeout=timeout)
        return True
    except: return False

def abort(dev):
    try: dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=1000); return True
    except: return False

def clr_status(dev):
    try: dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=1000); return True
    except: return False

def to_idle(dev, max_tries=20):
    for _ in range(max_tries):
        s = get_status(dev)
        if not s: return False
        st = s[0]
        if st == 2: return True
        if st == 10: clr_status(dev)
        elif st == 5: abort(dev)
        elif st in (3, 6): get_status(dev)
        elif st == 7: time.sleep(0.01)
        elif st == 8:
            try: dev.reset()
            except: pass
            time.sleep(1)
            return False  # need reconnect
        elif st == 4: time.sleep(s[2]/1000 + 0.05)
        else: abort(dev)
        time.sleep(0.02)
    return False

def reconnect_idle():
    """Find device and get to idle. Returns dev or None."""
    for _ in range(15):
        dev = find_dfu()
        if dev:
            try: dev.set_configuration()
            except: pass
            if to_idle(dev):
                return dev
        time.sleep(0.3)
    return None

def run():
    log("="*55)
    log("A12 TOCTOU — All-In-One (wait+kill+test)")
    log("="*55)
    
    # Kill Apple before waiting
    log("Disabling Apple services...")
    kill_apple()
    
    # Wait for DFU
    log("Waiting for DFU device... Enter DFU now!")
    dev = None
    for i in range(300):  # 60 seconds
        dev = find_dfu()
        if dev:
            break
        time.sleep(0.2)
    
    if not dev:
        log("Timeout — no DFU device found"); return
    
    log("DFU detected! Claiming device...")
    kill_apple()  # kill again in case they restarted
    time.sleep(0.2)
    
    # Re-find after kill (in case handle got stale)
    dev = find_dfu()
    if not dev:
        log("Lost device after service kill"); return
    
    try: dev.set_configuration()
    except: pass
    
    s = get_status(dev)
    log(f"Initial state: {s}")
    
    if not to_idle(dev):
        dev = reconnect_idle()
        if not dev:
            log("Can't reach IDLE"); return
    log("In IDLE — starting tests")

    # ============= TEST 1: Quick state map =============
    log("\n--- Test 1: State after DNLOAD ---")
    for size in [0, 16, 256, 2048]:
        if not to_idle(dev):
            dev = reconnect_idle()
            if not dev: log(f"  {size}B: LOST DEVICE"); return
        ok = dnload(dev, bytes(size))
        s = get_status(dev)
        log(f"  {size:5d}B: dn={'OK' if ok else 'FAIL'} → state={s[0] if s else 'X'} "
            f"bst={s[1] if s else '?'} poll={s[2] if s else '?'}")

    # ============= TEST 2: Manifest window =============
    log("\n--- Test 2: Manifest window (rapid poll) ---")
    for trial in range(3):
        if not to_idle(dev):
            dev = reconnect_idle()
            if not dev: break
        dnload(dev, bytes(2048))
        s = get_status(dev)
        if not s or s[0] != 5:
            log(f"  trial {trial}: not state 5 ({s})"); continue
        dnload(dev, b"")  # trigger manifest
        t0 = time.perf_counter_ns()
        states = []
        for i in range(50):
            s = get_status(dev)
            us = (time.perf_counter_ns()-t0)/1000
            if not s: break
            states.append((round(us), s[0], s[1]))
            if s[0] in (2, 8, 10): break
        log(f"  trial {trial}: {len(states)} polls")
        prev = None
        for us, st, bs in states:
            if st != prev:
                log(f"    state={st} bstatus={bs} @ {us}us")
                prev = st

    # ============= TEST 3: TOCTOU =============
    log("\n--- Test 3: TOCTOU — overwrite during manifest ---")
    overwrite = b"\xDE\xAD\xBE\xEF" * 512

    for delay_us in [0, 50, 200, 500, 1000, 2000, 5000]:
        if not to_idle(dev):
            dev = reconnect_idle()
            if not dev: log(f"  delay={delay_us}: LOST"); break

        dnload(dev, bytes(2048))
        s = get_status(dev)
        if not s or s[0] != 5:
            log(f"  delay={delay_us}: not state 5"); continue

        dnload(dev, b"")  # manifest
        t0 = time.perf_counter_ns()
        s = get_status(dev)
        entry = s[0] if s else -1

        if delay_us > 0:
            target = time.perf_counter_ns() + delay_us * 1000
            while time.perf_counter_ns() < target: pass

        t_ow = time.perf_counter_ns()
        ow_ok = dnload(dev, overwrite)
        ow_us = (time.perf_counter_ns() - t_ow) / 1000

        s = get_status(dev)
        after = s[0] if s else -1
        alive = find_dfu() is not None
        crashed = not alive

        log(f"  delay={delay_us:5d}us: entry={entry} ow={'OK' if ow_ok else 'FAIL'} "
            f"ow_time={ow_us:.0f}us after={after} "
            f"{'**CRASH**' if crashed else 'alive'}")

        if crashed:
            log("  >>> CRASH! Waiting...")
            dev = reconnect_idle()
            if not dev: break

    # ============= TEST 4: Double manifest =============
    log("\n--- Test 4: Double manifest ---")
    for desc, p1, p2 in [
        ("zeros->img4",  bytes(2048), b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038)),
        ("zeros->crash", bytes(2048), b"\x30\x80" + b"\xFF" * 2046),
        ("zeros->ptr",   bytes(2048), struct.pack("<Q", 0x100000000) * 256),
    ]:
        if not to_idle(dev):
            dev = reconnect_idle()
            if not dev: break

        dnload(dev, p1)
        s = get_status(dev)
        if not s or s[0] != 5: continue

        dnload(dev, b"")
        s = get_status(dev)

        ow_ok = dnload(dev, p2)
        s = get_status(dev)
        ow_state = s[0] if s else -1

        m2_state = None
        if ow_state == 5:
            dnload(dev, b"")
            for _ in range(50):
                s = get_status(dev)
                if not s or s[0] in (2, 8, 10): break
            m2_state = s[0] if s else -1

        alive = find_dfu() is not None
        log(f"  {desc:15s}: ow={'OK' if ow_ok else 'FAIL'} ow_st={ow_state} "
            f"m2={m2_state} alive={alive}")

        if not alive:
            log(f"  >>> CRASH on {desc}!")
            dev = reconnect_idle()
            if not dev: break

    # ============= TEST 5: Timing =============
    log("\n--- Test 5: Timing comparison ---")

    def meas(payload, n=5):
        times = []
        nonlocal dev
        for _ in range(n):
            if not to_idle(dev):
                dev = reconnect_idle()
                if not dev: return times
            dnload(dev, payload)
            get_status(dev)
            dnload(dev, b"")
            t0 = time.perf_counter_ns()
            for _ in range(50):
                s = get_status(dev)
                if not s or s[0] in (2, 8, 10): break
            times.append((time.perf_counter_ns()-t0)/1000)
        return times

    t_base = meas(bytes(2048))
    if t_base: log(f"  Zeros:    med={statistics.median(t_base):.0f}us raw={[round(t) for t in t_base]}")

    t_img4 = meas(b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038))
    if t_img4: log(f"  IMG4:     med={statistics.median(t_img4):.0f}us raw={[round(t) for t in t_img4]}")

    # With race overwrite
    ow_times = []
    for _ in range(5):
        if not to_idle(dev):
            dev = reconnect_idle()
            if not dev: break
        dnload(dev, bytes(2048))
        get_status(dev)
        dnload(dev, b"")
        get_status(dev)
        dnload(dev, b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038))
        s = get_status(dev)
        if s and s[0] == 5:
            dnload(dev, b"")
            t0 = time.perf_counter_ns()
            for _ in range(50):
                s = get_status(dev)
                if not s or s[0] in (2, 8, 10): break
            ow_times.append((time.perf_counter_ns()-t0)/1000)
    if ow_times:
        log(f"  OW race:  med={statistics.median(ow_times):.0f}us raw={[round(t) for t in ow_times]}")

    if t_base and t_img4:
        log(f"\n  IMG4 vs Zeros: {statistics.median(t_img4)-statistics.median(t_base):+.0f}us")
    if ow_times and t_base:
        log(f"  Race vs Zeros: {statistics.median(ow_times)-statistics.median(t_base):+.0f}us")

    # Re-enable Apple service
    os.system('sc config "Apple Mobile Device Service" start=auto >nul 2>&1')
    
    log("\nDONE")

if __name__ == "__main__":
    run()
