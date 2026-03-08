#!/usr/bin/env python3
"""
A12 SecureROM — Fast Manifest Timing Oracle (v3)
==================================================
Fix: properly trigger manifest via zero-length DNLOAD.
Lean: fewer tests, fewer repeats, focused on key questions.

KEY QUESTIONS:
  1. Does DER content affect manifest timing at 2048B?
  2. Does a DER length claiming >2048 cause OOB read (longer timing)?
  3. What state transitions happen during manifest?
"""
import time, json, struct, statistics
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
be = lambda: usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)

class FastOracle:
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
            if s[0] == 10: self.ctrl(0x21, 4, 0, 0, 0)  # CLR_STATUS
            elif s[0] == 4: time.sleep(s[2]/1000.0+0.01); self.status(); self.ctrl(0x21, 6, 0, 0, 0)
            else:
                try: self.ctrl(0x21, 6, 0, 0, 0)  # ABORT
                except: pass
            time.sleep(0.02)
        return False

    def manifest_cycle(self, payload):
        """
        DNLOAD(data) → GET_STATUS → DNLOAD(empty) → GET_STATUS → poll manifest
        Returns dict with precise timings for each phase.
        """
        if not self.to_idle(): return {"error": "no_idle"}

        # Step 1: DNLOAD with data
        t0 = time.perf_counter_ns()
        try:
            self.ctrl(0x21, 1, 0, 0, payload, timeout=5000)
        except usb.core.USBError as e:
            return {"error": f"dnload: {e}", "dnload_us": (time.perf_counter_ns()-t0)/1000}

        t1 = time.perf_counter_ns()

        # Step 2: GET_STATUS (should transition to state 5 = DNLOAD-IDLE)
        s = self.status()
        t2 = time.perf_counter_ns()
        if not s: return {"error": "no_status_after_dnload"}
        state_after_dnload = s[0]

        if state_after_dnload != 5:
            return {"error": f"unexpected_state={state_after_dnload}", 
                    "dnload_us": (t1-t0)/1000}

        # Step 3: Zero-length DNLOAD (triggers manifest)
        try:
            self.ctrl(0x21, 1, 0, 0, b"", timeout=5000)
        except usb.core.USBError as e:
            return {"error": f"zl_dnload: {e}"}
        t3 = time.perf_counter_ns()

        # Step 4: GET_STATUS → should enter manifest (state 6 or 7)
        s = self.status()
        t4 = time.perf_counter_ns()
        if not s: return {"error": "no_status_after_zl"}
        
        manifest_entry_state = s[0]
        manifest_poll = s[2]
        manifest_bstatus = s[1]

        # Step 5: Poll until terminal state (max 100 polls, max 5s)
        states_seen = [(s[0], (t4-t0)/1000)]
        deadline = time.perf_counter_ns() + 5_000_000_000  # 5s max
        
        while time.perf_counter_ns() < deadline:
            if s[0] == 4:  # BUSY — respect poll_ms
                time.sleep(max(0.001, s[2]/1000.0))
            s = self.status()
            tn = time.perf_counter_ns()
            if not s: break
            states_seen.append((s[0], (tn-t0)/1000))
            if s[0] in (2, 10, 8): break  # terminal

        t_end = time.perf_counter_ns()

        state_flow = []
        prev = None
        transitions = []
        for st, us in states_seen:
            if st != prev:
                state_flow.append(st)
                transitions.append({"s": st, "us": round(us,1)})
                prev = st

        return {
            "size": len(payload),
            "dnload_us": round((t1-t0)/1000, 1),
            "to_idle_us": round((t2-t0)/1000, 1),
            "zl_dnload_us": round((t3-t2)/1000, 1),
            "manifest_entry_us": round((t4-t0)/1000, 1),
            "manifest_entry_state": manifest_entry_state,
            "manifest_poll_ms": manifest_poll,
            "manifest_bstatus": manifest_bstatus,
            "total_us": round((t_end-t0)/1000, 1),
            "final_state": states_seen[-1][0] if states_seen else None,
            "state_flow": state_flow,
            "transitions": transitions,
            "n_polls": len(states_seen),
        }

    def run(self):
        log("="*60)
        log("A12 — Fast Manifest Timing Oracle v3")
        log("="*60)

        if not self.connect():
            log("NO DFU DEVICE"); return

        results = {"timestamp": datetime.now().isoformat()}
        REPS = 3

        # ---- Test A: Simple baseline (single size) ----
        log("\n--- Test A: Baseline manifest cycle (2048B zeros) ---")
        baselines = []
        for i in range(REPS):
            r = self.manifest_cycle(bytes(2048))
            baselines.append(r)
            if "error" in r:
                log(f"  rep{i}: ERROR: {r['error']}")
            else:
                log(f"  rep{i}: total={r['total_us']:.0f}us "
                    f"flow={r['state_flow']} manifest_poll={r['manifest_poll_ms']}ms "
                    f"bstatus={r['manifest_bstatus']}")
        results["baseline"] = baselines

        # ---- Test B: DER length claiming >2048B (OOB read probe) ----
        log("\n--- Test B: DER length overflow (all payloads 2048B) ---")
        der_tests = {}

        cases = [
            # (name, prefix)
            ("zeros",           bytes(6)),
            ("seq_len=2042",    b"\x30\x82\x07\xFA"),       # fits exactly
            ("seq_len=4090",    b"\x30\x82\x0F\xFA"),       # 2x buffer, OOB read?
            ("seq_len=8186",    b"\x30\x82\x1F\xFA"),       # 4x buffer
            ("seq_len=65530",   b"\x30\x82\xFF\xFA"),       # 32x buffer
            ("seq_len=4G",      b"\x30\x84\xFF\xFF\xFF\xFA"),
            ("img4_fit",        b"\x30\x82\x07\xF6\x16\x04IMG4"),
            ("img4_oob_4k",    b"\x30\x82\x0F\xF6\x16\x04IMG4"),
            ("img4_oob_64k",   b"\x30\x82\xFF\xF6\x16\x04IMG4"),
            ("im4p_fit",       b"\x30\x82\x07\xF6\x16\x04IM4P"),
            ("im4p_oob_4k",   b"\x30\x82\x0F\xF6\x16\x04IM4P"),
            ("octet_oob_4k",   b"\x04\x82\x0F\xFA"),
            ("ctx0_oob_4k",    b"\xA0\x82\x0F\xFA"),
        ]

        for name, prefix in cases:
            payload = prefix + bytes(2048 - len(prefix))
            timings = []
            for i in range(REPS):
                r = self.manifest_cycle(payload)
                timings.append(r)
            
            der_tests[name] = timings
            totals = [x["total_us"] for x in timings if "total_us" in x]
            flows = [str(x.get("state_flow","?")) for x in timings if "state_flow" in x]

            if totals:
                med = statistics.median(totals)
                log(f"  {name:20s}: total={med:>9.0f}us flow={flows[0]}")
            else:
                log(f"  {name:20s}: FAILED {[x.get('error','?') for x in timings]}")

        results["der_overflow"] = der_tests

        # ---- Test C: Different sizes to see if manifest scales ----
        log("\n--- Test C: Manifest timing vs payload size ---")
        size_tests = {}
        for sz in [16, 128, 512, 1024, 2048]:
            payload = bytes(sz)
            timings = []
            for i in range(REPS):
                r = self.manifest_cycle(payload)
                timings.append(r)
            size_tests[str(sz)] = timings
            totals = [x["total_us"] for x in timings if "total_us" in x]
            if totals:
                log(f"  {sz:5d}B: total={statistics.median(totals):>9.0f}us")

        results["size_vs_manifest"] = size_tests

        # ---- Test D: Valid-looking IMG4 vs garbage ----
        log("\n--- Test D: Valid-ish IMG4 structure vs garbage ---")
        # A somewhat valid-looking IMG4 wrapper
        img4_hdr = (
            b"\x30\x82\x07\xF6"      # SEQUENCE, len=2038
            b"\x16\x04IMG4"            # IA5String "IMG4"
            b"\x30\x82\x07\xE8"       # SEQUENCE (payload container), len=2024
            b"\x16\x04IM4P"            # IA5String "IM4P"
            b"\x16\x04ibss"            # IA5String "ibss"
            b"\x16\x00"               # IA5String "" (description)
            b"\x04\x82\x07\xD0"       # OCTET STRING, len=2000 (actual payload)
        )
        valid_img4 = img4_hdr + bytes(2048 - len(img4_hdr))
        
        d_tests = {}
        for name, payload in [
            ("garbage_2048",  b"\xDE\xAD" * 1024),
            ("zeros_2048",    bytes(2048)),
            ("valid_img4",    valid_img4),
        ]:
            timings = []
            for i in range(REPS):
                r = self.manifest_cycle(payload)
                timings.append(r)
            d_tests[name] = timings
            totals = [x["total_us"] for x in timings if "total_us" in x]
            if totals:
                med = statistics.median(totals)
                log(f"  {name:20s}: {med:>9.0f}us")

        results["img4_vs_garbage"] = d_tests

        # Save
        outf = Path(__file__).parent / "results" / "manifest_oracle_v3.json"
        outf.parent.mkdir(exist_ok=True)
        with open(outf, "w") as f:
            json.dump(results, f, indent=2)
        log(f"\nSaved: {outf}")
        log("DONE")

if __name__ == "__main__":
    FastOracle().run()
