#!/usr/bin/env python3
"""
A12 SecureROM DER Truncated Sequence Probe
=============================================
CONFIRMED ANOMALY: Case 17 (der_truncated_seq) triggers:
  - 73ms manifest time instead of 3562ms (different parser path!)
  - Device instability → eventual USB disconnect
  - Payload: \x30\x82\x10\x00\x16\x04IMG4 (10 bytes only)
  - SEQUENCE claiming 4096B but containing only 6B of "IMG4" magic

This probe systematically explores:
  1. REPRODUCE: Send the exact anomalous payload
  2. MAGIC: Does "IMG4" magic matter? Compare with random/zero content
  3. CLAIMED SIZE: Vary the claimed length (512, 1024, 4096, 65535)
  4. ACTUAL SIZE: Vary actual content size (0, 6, 16, 64, 256, 511)
  5. PAD: Test with/without zero padding to 512B
  6. MULTI-BLOCK: Test with small payload across multiple DNLOAD blocks
  7. RAPID: Repeated fast manifest cycles without reconnect

Each test that causes instability will be logged. If device disappears,
re-enter DFU and resume with --start.
"""

import sys, os, time, struct, json, traceback
from datetime import datetime
from pathlib import Path

RESULTS_DIR = Path(__file__).parent.resolve() / "results"
RESULTS_DIR.mkdir(exist_ok=True)

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
DFU_DNLOAD = 1; DFU_GET_STATUS = 3; DFU_CLR_STATUS = 4
DFU_GET_STATE = 5; DFU_ABORT = 6

LOG_FILE = RESULTS_DIR / "truncation_probe_log.txt"
RESULT_FILE = RESULTS_DIR / "truncation_probe.json"


class TruncationProbe:
    def __init__(self):
        self.dev = None
        self.lines = []
        self.findings = []
        self.results = []

    def log(self, msg, level="INFO"):
        ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        line = f"[{ts}] [{level:4s}] {msg}"
        print(line, flush=True)
        self.lines.append(line)

    def finding(self, title, detail, severity="HIGH"):
        f = {"title": title, "detail": detail, "severity": severity,
             "timestamp": datetime.now().isoformat()}
        self.findings.append(f)
        self.log(f"*** [{severity}] {title}: {detail}", "FIND")

    def connect(self):
        for attempt in range(5):
            backend = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
            self.dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend)
            if self.dev:
                try: self.dev.set_configuration()
                except: pass
                sn = self.dev.serial_number[:60]
                self.log(f"DFU connected: {sn}")
                return True
            time.sleep(1)
        self.log("DFU not found!", "ERR")
        return False

    def ctrl(self, bmRT, bReq, wVal, wIdx, d_or_l, timeout=2000):
        try:
            r = self.dev.ctrl_transfer(bmRT, bReq, wVal, wIdx, d_or_l, timeout=timeout)
            if isinstance(r, int): return (None, None)
            return (bytes(r), None)
        except usb.core.USBTimeoutError: return (None, "TIMEOUT")
        except usb.core.USBError as e:
            if e.errno == 32 or "pipe" in str(e).lower(): return (None, "STALL")
            return (None, str(e))

    def get_status(self):
        data, err = self.ctrl(0xA1, DFU_GET_STATUS, 0, 0, 6)
        if data and len(data) >= 6:
            return {"bStatus": data[0], "bState": data[4],
                    "poll_ms": data[1]|(data[2]<<8)|(data[3]<<16),
                    "raw": data.hex()}
        return None

    def get_state(self):
        data, err = self.ctrl(0xA1, DFU_GET_STATE, 0, 0, 1)
        if data and len(data) >= 1:
            return data[0]
        return None

    def dnload(self, payload, block=0):
        _, err = self.ctrl(0x21, DFU_DNLOAD, block, 0, payload)
        return err is None

    def reset_to_idle(self):
        for attempt in range(20):
            time.sleep(0.030)
            st = self.get_status()
            if st and st["bState"] == 2: return True
            if st and st["bState"] == 10:
                self.ctrl(0x21, DFU_CLR_STATUS, 0, 0, 0)
                time.sleep(0.050); continue
            if st and st["bState"] == 8:
                return self._reconnect_dfu()
            if st and st["bState"] in (5, 3):
                self.ctrl(0x21, DFU_ABORT, 0, 0, 0)
                time.sleep(0.050); continue
            if st is None:
                return self._reconnect_dfu()
            self.ctrl(0x21, DFU_ABORT, 0, 0, 0)
            time.sleep(0.050)
        st = self.get_status()
        return st is not None and st["bState"] == 2

    def _reconnect_dfu(self):
        self.log("    _reconnect: ABORT + USB reset cycle...")
        try:
            if self.dev:
                self.dev.ctrl_transfer(0x21, DFU_ABORT, 0, 0, 0, timeout=2000)
                time.sleep(0.1)
        except: pass
        try:
            if self.dev: self.dev.reset()
        except: pass
        time.sleep(2.0)
        try:
            if self.dev: usb.util.dispose_resources(self.dev)
        except: pass
        self.dev = None
        
        for attempt in range(10):
            time.sleep(0.5)
            try:
                backend = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
                dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend)
                if dev:
                    self.dev = dev
                    try: self.dev.set_configuration()
                    except: pass
                    time.sleep(0.3)
                    r = self.dev.ctrl_transfer(0xA1, DFU_GET_STATUS, 0, 0, 6, timeout=2000)
                    state = r[4] if len(r) >= 5 else -1
                    self.log(f"    _reconnect: attempt {attempt}: state={state}")
                    if state == 2: return True
                    if state == 8:
                        try: self.dev.ctrl_transfer(0x21, DFU_ABORT, 0, 0, 0, timeout=2000)
                        except: pass
                        time.sleep(0.1)
                        try: self.dev.reset()
                        except: pass
                        time.sleep(2.0)
                        try: usb.util.dispose_resources(self.dev)
                        except: pass
                        self.dev = None; continue
                    if state == 10:
                        self.dev.ctrl_transfer(0x21, DFU_CLR_STATUS, 0, 0, 0, timeout=2000)
                        time.sleep(0.1)
                        r2 = self.dev.ctrl_transfer(0xA1, DFU_GET_STATUS, 0, 0, 6, timeout=2000)
                        if len(r2) >= 5 and r2[4] == 2: return True
            except Exception as e:
                self.log(f"    _reconnect: attempt {attempt}: {e}", "WARN")
        self.log("    _reconnect: FAILED", "ERR")
        return False

    def device_alive(self):
        try:
            backend = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
            d = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend)
            return d is not None
        except:
            return False

    def send_and_observe(self, payload, name="test"):
        """
        Full DFU manifest flow with detailed state tracking.
        
        Records every state transition with timestamps for analysis.
        """
        result = {"name": name, "size": len(payload),
                  "payload_hex": payload[:32].hex() + ("..." if len(payload) > 32 else "")}
        states = []
        
        # Ensure idle
        if not self.reset_to_idle():
            result["error"] = "cannot_reach_idle"
            self.log(f"  {name}: SKIP (not idle)", "WARN")
            return result

        t_start = time.perf_counter()
        
        # === DNLOAD ===
        block_size = 4096
        block = 0
        remaining = payload
        while remaining:
            chunk = remaining[:block_size]
            remaining = remaining[block_size:]
            ok = self.dnload(chunk, block=block)
            if not ok:
                result["error"] = f"dnload_fail_block_{block}"
                return result
            st = self.get_status()
            if st is None:
                result["crashed"] = True
                result["crash_phase"] = "DNLOAD"
                result["crash_time_ms"] = round((time.perf_counter()-t_start)*1000, 1)
                return result
            states.append({"t_ms": round((time.perf_counter()-t_start)*1000, 1),
                          "phase": "DNLOAD", "state": st["bState"], "poll": st["poll_ms"]})
            if st["bState"] == 10:
                result["error"] = "dnload_error"
                result["states"] = states
                return result
            time.sleep(max(0.010, st["poll_ms"]/1000.0) + 0.010)
            for _ in range(5):
                st = self.get_status()
                if st and st["bState"] == 5: break
                if st is None:
                    result["crashed"] = True
                    result["crash_phase"] = "DNLOAD_POLL"
                    result["crash_time_ms"] = round((time.perf_counter()-t_start)*1000, 1)
                    return result
                time.sleep(0.060)
            block += 1

        t_dnload = (time.perf_counter() - t_start) * 1000
        result["dnload_time_ms"] = round(t_dnload, 1)
        
        # === TRIGGER MANIFEST (zero-length DNLOAD) ===
        ok = self.dnload(b"", block=block)
        if not ok:
            result["error"] = "manifest_trigger_fail"
            result["states"] = states
            return result

        t_manifest_start = time.perf_counter()
        
        # === TRACK STATE MACHINE IN DETAIL ===
        # Poll GET_STATUS rapidly to capture every transition
        for poll_idx in range(40):
            st = self.get_status()
            t_now = (time.perf_counter() - t_manifest_start) * 1000
            
            if st is None:
                states.append({"t_ms": round(t_now, 1), "phase": "MANIFEST",
                              "state": None, "poll": None, "note": "GET_STATUS returned None"})
                # Device might be gone — check
                time.sleep(0.5)
                if not self.device_alive():
                    result["crashed"] = True
                    result["crash_phase"] = "MANIFEST"
                    result["crash_time_ms"] = round(t_now, 1)
                    self.finding("CRASH in manifest", f"{name}: device gone @ {t_now:.0f}ms")
                    break
                # Device there but unresponsive — try again
                continue
            
            states.append({"t_ms": round(t_now, 1), "phase": "MANIFEST",
                          "state": st["bState"], "poll": st["poll_ms"],
                          "bStatus": st["bStatus"]})
            
            if st["bState"] == 6:  # MANIFEST-SYNC
                time.sleep(max(0.010, st["poll_ms"]/1000.0) + 0.010)
                continue
            elif st["bState"] == 7:  # MANIFEST (processing)
                # Wait full poll time
                wait = max(0.5, st["poll_ms"] / 1000.0) + 0.500
                time.sleep(wait)
                continue
            elif st["bState"] == 8:  # MANIFEST-WAIT-RESET (done)
                result["survived"] = True
                break
            elif st["bState"] == 10:  # ERROR
                result["error_state"] = True
                # Try to clear
                self.ctrl(0x21, DFU_CLR_STATUS, 0, 0, 0)
                break
            elif st["bState"] == 2:  # IDLE (shouldn't happen?)
                result["unexpected_idle"] = True
                break
            else:
                time.sleep(0.030)
                continue
        
        t_manifest_end = time.perf_counter()
        manifest_time = (t_manifest_end - t_manifest_start) * 1000
        result["manifest_time_ms"] = round(manifest_time, 1)
        result["total_time_ms"] = round((t_manifest_end - t_start) * 1000, 1)
        result["states"] = states
        result["n_state_transitions"] = len(states)
        
        # Final state
        try:
            st_final = self.get_status()
            if st_final:
                result["final_state"] = st_final["bState"]
                result["final_bStatus"] = st_final["bStatus"]
                if not result.get("crashed"):
                    result["crashed"] = False
            else:
                if not result.get("crashed"):
                    result["crashed"] = True
                    result["crash_phase"] = "POST_MANIFEST"
        except:
            if not result.get("crashed"):
                result["crashed"] = True
                result["crash_phase"] = "EXCEPTION"
        
        # Log
        ms = result.get("manifest_time_ms", 0)
        fs = result.get("final_state", "?")
        crashed = result.get("crashed", False)
        tag = "CRASH" if crashed else "OK"
        self.log(f"  [{tag:5s}] {name:45s} | {len(payload):5d}B | {ms:7.1f}ms | state={fs}")
        
        if ms < 1000 and not crashed:
            self.finding("FAST MANIFEST",
                        f"{name}: manifest={ms:.1f}ms (expected ~3562ms) — different parser path!",
                        severity="HIGH")
        
        self.results.append(result)
        return result

    def generate_tests(self):
        """Generate truncation-focused test suite."""
        tests = []

        # ===== GROUP 1: REPRODUCE original anomaly =====
        # Exact payload from case 17 of img4_fuzzer
        img4_magic = b"\x16\x04IMG4"  # IA5String "IMG4"
        tests.append(("REPRO_exact_10B",
            b"\x30\x82\x10\x00" + img4_magic))  # 10 bytes, claims 4096B

        # ===== GROUP 2: MAGIC matters? =====
        # Same structure but without IMG4 magic
        tests.append(("magic_zeroes_10B",
            b"\x30\x82\x10\x00" + b"\x00" * 6))  # 10B, zeroes instead of IMG4
        tests.append(("magic_random_10B",
            b"\x30\x82\x10\x00" + b"\x41\x42\x43\x44\x45\x46"))  # 10B, "ABCDEF"
        tests.append(("magic_IM4P_10B",
            b"\x30\x82\x10\x00" + b"\x16\x04IM4P"))  # IM4P magic
        tests.append(("magic_iBoot_10B",
            b"\x30\x82\x10\x00" + b"\x16\x04iBot"))  # iBot magic

        # ===== GROUP 3: CLAIMED SIZE variations (all 10B actual) =====
        tests.append(("claim_512B",
            b"\x30\x82\x02\x00" + img4_magic))   # claims 512B
        tests.append(("claim_1024B",
            b"\x30\x82\x04\x00" + img4_magic))   # claims 1024B
        tests.append(("claim_2048B",  
            b"\x30\x82\x08\x00" + img4_magic))   # claims 2048B
        tests.append(("claim_4096B",
            b"\x30\x82\x10\x00" + img4_magic))   # claims 4096B (=original)
        tests.append(("claim_8192B",
            b"\x30\x82\x20\x00" + img4_magic))   # claims 8192B
        tests.append(("claim_32768B",
            b"\x30\x82\x80\x00" + img4_magic))   # claims 32768B
        tests.append(("claim_65535B",
            b"\x30\x82\xFF\xFF" + img4_magic))    # claims 65535B

        # ===== GROUP 4: ACTUAL SIZE variations (all claim 4096B) =====
        tests.append(("actual_4B",
            b"\x30\x82\x10\x00"))                 # 4B: just the header, no content
        tests.append(("actual_5B",
            b"\x30\x82\x10\x00\x00"))             # 5B: header + 1 zero
        tests.append(("actual_6B",
            b"\x30\x82\x10\x00\x00\x00"))         # 6B: header + 2 zeroes
        tests.append(("actual_10B",
            b"\x30\x82\x10\x00" + img4_magic))    # 10B (=original)
        tests.append(("actual_16B",
            b"\x30\x82\x10\x00" + img4_magic + b"\x00" * 6))  # 16B
        tests.append(("actual_64B",
            b"\x30\x82\x10\x00" + img4_magic + b"\x00" * 54)) # 64B
        tests.append(("actual_256B",
            b"\x30\x82\x10\x00" + img4_magic + b"\x00" * 246)) # 256B
        tests.append(("actual_511B",
            b"\x30\x82\x10\x00" + img4_magic + b"\x00" * 501)) # 511B

        # ===== GROUP 5: PADDED vs UNPADDED (does 512B alignment matter?) =====
        tests.append(("padded_512B",
            (b"\x30\x82\x10\x00" + img4_magic).ljust(512, b"\x00")))  # 512B aligned
        tests.append(("padded_4096B",
            (b"\x30\x82\x10\x00" + img4_magic).ljust(4096, b"\x00")))  # 4096B aligned (full claim!)
        
        # ===== GROUP 6: EXTREME TRUNCATION (header only, various tags) =====
        tests.append(("trunc_SEQUENCE_alone",
            b"\x30\x82\x10\x00"))                  # Just SEQUENCE header
        tests.append(("trunc_OCTET_alone",
            b"\x04\x82\x10\x00"))                  # Just OCTET STRING header
        tests.append(("trunc_INTEGER_alone",
            b"\x02\x82\x10\x00"))                  # Just INTEGER header
        tests.append(("trunc_IA5_alone",
            b"\x16\x82\x10\x00"))                  # Just IA5STRING header
        
        # ===== GROUP 7: COMPARE with minimal valid-length payloads =====
        # These claim the CORRECT length — should work normally
        tests.append(("correct_len_6B",
            b"\x30\x06" + img4_magic))              # Correct: claims 6B, has 6B
        tests.append(("correct_len_0B",
            b"\x30\x00"))                            # Empty SEQUENCE
        
        # ===== GROUP 8: EDGE CASES =====
        tests.append(("empty_1B",
            b"\x30"))                                # Just tag, no length
        tests.append(("empty_2B",
            b"\x30\x82"))                            # Tag + incomplete length
        tests.append(("empty_3B",
            b"\x30\x82\x10"))                        # Tag + truncated length field

        return tests

    def run(self, start_at=0, max_tests=None):
        tests = self.generate_tests()
        total = len(tests)
        
        if max_tests:
            tests = tests[:max_tests]
        tests = tests[start_at:]
        
        self.log(f"\n{'#'*60}")
        self.log(f"DER TRUNCATED SEQUENCE PROBE — A12 SecureROM")
        self.log(f"Tests: {len(tests)}/{total} (starting at {start_at})")
        self.log(f"{'#'*60}\n")
        
        if not self.connect():
            self.log("Cannot connect to DFU device!", "ERR")
            return
        
        for i, (name, payload) in enumerate(tests):
            idx = start_at + i
            self.log(f"\n--- Test {idx+1}/{total}: {name} ({len(payload)}B) ---")
            result = self.send_and_observe(payload, name=name)
            result["index"] = idx
            
            if result.get("crashed"):
                self.log(f"\n{'!'*60}")
                self.log(f"DEVICE CRASHED on '{name}'!")
                self.log(f"Re-enter DFU and run with --start {idx+1}")
                self.log(f"{'!'*60}")
                break
            
            # Health check between tests
            time.sleep(0.5)
            if not self.device_alive():
                self.log(f"Device disappeared after '{name}'!", "ERR")
                result["crashed"] = True
                result["crash_phase"] = "DELAYED"
                self.finding("DELAYED CRASH", f"{name}: device gone between tests")
                break
        
        self.save_results()

    def save_results(self):
        data = {
            "probe": "truncation_probe",
            "device": "A12 T8020 (iPhone XR)",
            "timestamp": datetime.now().isoformat(),
            "results": self.results,
            "findings": self.findings,
            "summary": {
                "total_tests": len(self.results),
                "crashes": sum(1 for r in self.results if r.get("crashed")),
                "survived": sum(1 for r in self.results if not r.get("crashed") and not r.get("error")),
                "errors": sum(1 for r in self.results if r.get("error")),
                "fast_manifests": sum(1 for r in self.results 
                                     if r.get("manifest_time_ms", 9999) < 1000 
                                     and not r.get("crashed")),
            }
        }
        
        existing = []
        if RESULT_FILE.exists():
            try:
                with open(RESULT_FILE, "r") as f:
                    old = json.load(f)
                    existing = old.get("all_runs", [])
            except: pass
        existing.append(data)
        
        with open(RESULT_FILE, "w") as f:
            json.dump({"all_runs": existing, "latest": data}, f, indent=2)
        
        with open(LOG_FILE, "a") as f:
            f.write("\n".join(self.lines) + "\n")
        
        self.log(f"\nResults: {RESULT_FILE}")
        
        # Summary
        self.log(f"\n{'='*60}")
        self.log(f"SUMMARY")
        self.log(f"  Tests:    {data['summary']['total_tests']}")
        self.log(f"  Survived: {data['summary']['survived']}")
        self.log(f"  Crashed:  {data['summary']['crashes']}")
        self.log(f"  Errors:   {data['summary']['errors']}")
        self.log(f"  Fast:     {data['summary']['fast_manifests']}")
        
        if self.findings:
            self.log(f"\nFINDINGS:")
            for f in self.findings:
                self.log(f"  [{f['severity']}] {f['title']}: {f['detail']}")
        
        # Timing table
        ok_results = [r for r in self.results if not r.get("crashed") and not r.get("error")]
        if ok_results:
            self.log(f"\nTIMING TABLE:")
            self.log(f"  {'Name':45s} | {'Size':>5s} | {'Manifest':>8s} | {'State':>5s}")
            self.log(f"  {'-'*45}-+-{'-'*5}-+-{'-'*8}-+-{'-'*5}")
            for r in ok_results:
                ms = r.get("manifest_time_ms", 0)
                fs = r.get("final_state", "?")
                marker = " <<<" if ms < 1000 else ""
                self.log(f"  {r['name']:45s} | {r['size']:5d} | {ms:7.1f}ms | {str(fs):>5s}{marker}")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="A12 DER Truncation Probe")
    parser.add_argument("--start", type=int, default=0,
                       help="Start at test index N (for resuming)")
    parser.add_argument("-n", "--max", type=int, default=None,
                       help="Max tests to run")
    args = parser.parse_args()
    
    probe = TruncationProbe()
    probe.run(start_at=args.start, max_tests=args.max)


if __name__ == "__main__":
    main()
