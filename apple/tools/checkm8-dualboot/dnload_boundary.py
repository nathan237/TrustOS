#!/usr/bin/env python3
"""
A12 SecureROM — DNLOAD Size Boundary Precision Probe
======================================================
Found: DNLOAD 16-2048B = ~484-1131us (linear)
       DNLOAD 3072B = ~1,007,000us (1 second!)
       
There's a critical boundary between 2048 and 3072 bytes.
This tool binary-searches for the exact byte where timing jumps.

Also measures manifesting timing at sizes around the boundary
because the 1s delay might indicate a different code path
that could be exploitable.
"""
import time, json, statistics
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227

def be():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    print(f"[{ts}] {msg}", flush=True)

class BoundaryProbe:
    def __init__(self):
        self.dev = None

    def connect(self):
        self.dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=be())
        if self.dev:
            try: self.dev.set_configuration()
            except: pass
            return True
        return False

    def status(self):
        try:
            r = self.dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=5000)
            return (r[4], r[0], r[1]|(r[2]<<8)|(r[3]<<16)) if len(r)>=6 else None
        except: return None

    def to_idle(self):
        for _ in range(30):
            s = self.status()
            if not s:
                time.sleep(0.1); self.connect(); continue
            state = s[0]
            if state == 2: return True
            elif state == 10: self.dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
            elif state in (3, 5): 
                try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                except: pass
            elif state == 4:
                time.sleep(s[2]/1000.0 + 0.01); self.status()
                try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                except: pass
            elif state == 8:
                try: self.dev.reset()
                except: pass
                time.sleep(2); self.connect()
            else:
                try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                except: pass
            time.sleep(0.02)
        return False

    def measure_dnload(self, size, repeats=5):
        """Measure DNLOAD time for exact size, return median in us"""
        payload = bytes([0x42]) * size
        times = []
        
        for rep in range(repeats):
            if not self.to_idle():
                continue
            t0 = time.perf_counter_ns()
            try:
                self.dev.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=10000)
                dt_us = (time.perf_counter_ns() - t0) / 1000
                times.append(dt_us)
            except usb.core.USBError as e:
                dt_us = (time.perf_counter_ns() - t0) / 1000
                times.append(dt_us)  # Error timing is data too
                log(f"  {size}B rep{rep}: err {e} ({dt_us:.0f}us)")
            
            # Cleanup
            s = self.status()
            if s and s[0] in (3, 5):
                try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                except: pass
            elif s and s[0] == 4:
                time.sleep(s[2]/1000.0 + 0.01)
                self.status()
                try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                except: pass
        
        if not times:
            return None, []
        return statistics.median(times), times

    def run(self):
        log("="*60)
        log("A12 SecureROM — DNLOAD Size Boundary Probe")
        log("="*60)

        if not self.connect():
            log("NO DFU DEVICE"); return

        results = {"timestamp": datetime.now().isoformat()}

        # Phase 1: Coarse scan (confirm the boundary region)
        log("\n--- Phase 1: Coarse scan 2048-3072 ---")
        coarse = []
        for size in range(2048, 3200, 64):
            median, raw = self.measure_dnload(size, repeats=3)
            if median is not None:
                is_slow = median > 100000  # >100ms = slow path
                coarse.append({"size": size, "median_us": round(median, 1), "slow": is_slow})
                marker = " <<< SLOW!" if is_slow else ""
                log(f"  {size:5d}B: {median:12.1f} us{marker}")
                
                if is_slow and len(coarse) >= 2 and not coarse[-2]["slow"]:
                    # Found the transition!
                    boundary_low = coarse[-2]["size"]
                    boundary_high = size
                    log(f"\n  >>> Boundary between {boundary_low} and {boundary_high}")
                    break
        else:
            # Didn't find boundary in coarse scan
            boundary_low = None
            boundary_high = None
            # Check if all were fast or all slow
            if coarse:
                if all(not c["slow"] for c in coarse):
                    log("  All fast — boundary might be higher")
                    # Try larger sizes
                    for size in [3072, 4096, 5120, 6144, 8192]:
                        median, _ = self.measure_dnload(size, repeats=3)
                        if median and median > 100000:
                            log(f"  {size}B: {median:.1f}us — SLOW found")
                            boundary_high = size
                            boundary_low = coarse[-1]["size"] if coarse else size - 1024
                            break

        results["coarse_scan"] = coarse

        # Phase 2: Binary search for exact boundary
        if boundary_low is not None and boundary_high is not None:
            log(f"\n--- Phase 2: Binary search [{boundary_low}, {boundary_high}] ---")
            lo, hi = boundary_low, boundary_high
            fine = []
            
            while hi - lo > 1:
                mid = (lo + hi) // 2
                median, _ = self.measure_dnload(mid, repeats=3)
                if median is None:
                    log(f"  {mid}B: measurement failed")
                    break
                is_slow = median > 100000
                fine.append({"size": mid, "median_us": round(median, 1), "slow": is_slow})
                marker = " SLOW" if is_slow else " FAST"
                log(f"  {mid:5d}B: {median:12.1f} us{marker}")
                
                if is_slow:
                    hi = mid
                else:
                    lo = mid

            log(f"\n  >>> EXACT BOUNDARY: {lo}B=FAST, {hi}B=SLOW")
            results["boundary"] = {"fast_max": lo, "slow_min": hi}
            results["fine_scan"] = fine

            # Phase 3: Precise timing around boundary
            log(f"\n--- Phase 3: Precise timing around boundary ---")
            precise = []
            for offset in [-8, -4, -2, -1, 0, 1, 2, 4, 8]:
                sz = lo + offset
                if sz < 16: continue
                median, raw = self.measure_dnload(sz, repeats=5)
                if median:
                    precise.append({"size": sz, "median_us": round(median, 1),
                                    "all_us": [round(t, 1) for t in raw]})
                    log(f"  {sz:5d}B: {median:12.1f} us  raw={[round(t) for t in raw]}")
            results["precise_boundary"] = precise

        # Save
        outf = Path(__file__).parent / "results" / "size_boundary_timing.json"
        outf.parent.mkdir(exist_ok=True)
        with open(outf, "w") as f:
            json.dump(results, f, indent=2)
        log(f"\nSaved: {outf}")

if __name__ == "__main__":
    b = BoundaryProbe()
    b.run()
