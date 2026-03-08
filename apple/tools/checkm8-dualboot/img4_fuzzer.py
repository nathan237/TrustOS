#!/usr/bin/env python3
"""
A12 SecureROM IMG4 Parser Fuzzer
=================================
CONFIRMED: Manifest phase processes attacker data for ~3 seconds.
Flow: DNLOAD blocks → zero-length DNLOAD → dfuMANIFEST (3s) → back to DFU.

This fuzzer sends carefully crafted malformed IMG4/DER payloads to the
SecureROM's parser and measures:
  - Manifest processing time (baseline: ~3000ms)
  - Whether device reaches MANIFEST-WAIT-RESET vs dfuERROR
  - Whether device crashes (disappears from USB)
  - Timing anomalies that indicate different parser paths

Attack targets in IMG4 parser:
  1. DER length overflow (32-bit integer)
  2. Deep ASN.1 nesting (stack overflow)
  3. Type confusion (wrong tags)
  4. Truncated structures 
  5. IM4P component fuzzing
  6. IM4M signature chain
  7. OCTET STRING size variants
  8. Nonce/hash field manipulation
"""

import sys, os, time, struct, json, traceback, hashlib
from datetime import datetime
from pathlib import Path

RESULTS_DIR = Path(__file__).parent.resolve() / "results"
RESULTS_DIR.mkdir(exist_ok=True)

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
DFU_DNLOAD = 1; DFU_UPLOAD = 2; DFU_GET_STATUS = 3
DFU_CLR_STATUS = 4; DFU_GET_STATE = 5; DFU_ABORT = 6


class IMG4Fuzzer:
    def __init__(self):
        self.dev = None
        self.lines = []
        self.findings = []
        self.results = []
        self.case_num = 0

    def log(self, msg, level="INFO"):
        ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        line = f"[{ts}] [{level:4s}] {msg}"
        print(line)
        self.lines.append(line)

    def finding(self, title, detail, severity="high"):
        f = {"title": title, "detail": detail, "severity": severity,
             "case": self.case_num, "timestamp": datetime.now().isoformat()}
        self.findings.append(f)
        self.log(f"*** [{severity.upper()}] {title}: {detail}", "FIND")

    def connect(self):
        """Connect to DFU device, with retry."""
        for attempt in range(5):
            backend = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
            self.dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend)
            if self.dev:
                try: self.dev.set_configuration()
                except: pass
                return True
            time.sleep(1)
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
            if st and st["bState"] == 2:  # dfuIDLE
                return True
            if st and st["bState"] == 10:  # dfuERROR
                self.ctrl(0x21, DFU_CLR_STATUS, 0, 0, 0)
                time.sleep(0.050)
                continue
            if st and st["bState"] == 8:  # MANIFEST-WAIT-RESET
                # Device needs full USB re-enumeration
                return self._reconnect_dfu()
            if st and st["bState"] in (5, 3):  # DNLOAD-IDLE / DNLOAD-SYNC
                self.ctrl(0x21, DFU_ABORT, 0, 0, 0)
                time.sleep(0.050)
                continue
            if st is None:
                # Lost device — try reconnect
                return self._reconnect_dfu()
            # Unknown state — try abort
            self.ctrl(0x21, DFU_ABORT, 0, 0, 0)
            time.sleep(0.050)
        st = self.get_status()
        return st is not None and st["bState"] == 2

    def _reconnect_dfu(self):
        """Reset device from MANIFEST-WAIT-RESET back to dfuIDLE.
        Required sequence: ABORT → dev.reset() → reconnect → dfuIDLE."""
        self.log("    _reconnect: ABORT + USB reset cycle...", "INFO")
        
        # 1. Send ABORT first (critical: puts device in transitional state)
        try:
            if self.dev:
                self.dev.ctrl_transfer(0x21, DFU_ABORT, 0, 0, 0, timeout=2000)
                time.sleep(0.1)
        except: pass
        
        # 2. USB bus reset
        try:
            if self.dev:
                self.dev.reset()
        except: pass
        time.sleep(2.0)
        
        # 3. Dispose stale handle
        try:
            if self.dev:
                usb.util.dispose_resources(self.dev)
        except: pass
        self.dev = None
        
        # 4. Re-find with fresh backend
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
                    self.log(f"    _reconnect: attempt {attempt}: state={state}", "INFO")
                    if state == 2:
                        return True
                    if state == 8:
                        # ABORT then reset again
                        try: self.dev.ctrl_transfer(0x21, DFU_ABORT, 0, 0, 0, timeout=2000)
                        except: pass
                        time.sleep(0.1)
                        try: self.dev.reset()
                        except: pass
                        time.sleep(2.0)
                        try: usb.util.dispose_resources(self.dev)
                        except: pass
                        self.dev = None
                        continue
                    if state == 10:
                        self.dev.ctrl_transfer(0x21, DFU_CLR_STATUS, 0, 0, 0, timeout=2000)
                        time.sleep(0.1)
                        r2 = self.dev.ctrl_transfer(0xA1, DFU_GET_STATUS, 0, 0, 6, timeout=2000)
                        if len(r2) >= 5 and r2[4] == 2:
                            return True
            except Exception as e:
                self.log(f"    _reconnect: attempt {attempt}: {e}", "WARN")
        
        self.log("    _reconnect: FAILED", "ERR")
        return False

    def send_and_manifest(self, payload, name="test"):
        """
        Full DFU flow: DNLOAD payload → GET_STATUS → zero-length DNLOAD → 
        measure manifest time → record outcome.
        
        Returns dict with timing and state info.
        """
        self.case_num += 1
        result = {"case": self.case_num, "name": name, "size": len(payload)}
        
        # Ensure idle
        if not self.reset_to_idle():
            result["error"] = "cannot_reach_idle"
            self.log(f"  [{self.case_num:3d}] {name}: SKIP (not idle)", "WARN")
            return result

        # DNLOAD the payload (may need multiple blocks for large payloads)
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
            if st is None or st["bState"] == 10:
                result["error"] = "dnload_error"
                result["state_after_dnload"] = st
                return result
            # Wait poll timeout
            time.sleep(max(0.010, (st["poll_ms"] / 1000.0)) + 0.010)
            # Poll until dfuDNLOAD-IDLE
            for _ in range(5):
                st = self.get_status()
                if st and st["bState"] == 5: break
                time.sleep(0.060)
            block += 1

        result["blocks"] = block
        
        # Trigger manifest: zero-length DNLOAD
        ok = self.dnload(b"", block=block)
        if not ok:
            result["error"] = "manifest_trigger_fail"
            return result

        # Track manifest timing
        t_manifest_start = time.perf_counter()
        
        st = self.get_status()
        result["manifest_sync"] = st
        
        if st and st["bState"] == 6:  # dfuMANIFEST-SYNC
            # Wait poll timeout and check again
            time.sleep(max(0.010, st["poll_ms"]/1000.0) + 0.010)
            st = self.get_status()
            result["manifest_state"] = st
            
            if st and st["bState"] == 7:  # dfuMANIFEST
                manifest_poll = st["poll_ms"]
                result["manifest_poll_ms"] = manifest_poll
                # Wait for manifest to complete
                time.sleep(max(0.100, manifest_poll / 1000.0) + 0.500)
                st = self.get_status()
                result["after_manifest"] = st
        elif st and st["bState"] == 10:  # dfuERROR
            result["error"] = "manifest_error"
        
        t_manifest_end = time.perf_counter()
        manifest_time = (t_manifest_end - t_manifest_start) * 1000
        result["manifest_time_ms"] = round(manifest_time, 1)
        
        # Final state check
        try:
            st_final = self.get_status()
            result["final_state"] = st_final
            if st_final:
                result["final_bState"] = st_final["bState"]
            else:
                result["final_bState"] = -1
        except:
            result["final_bState"] = -2  # crashed

        # Log result
        ms = result.get("manifest_time_ms", 0)
        fst = result.get("final_bState", -1)
        mp = result.get("manifest_poll_ms", -1)
        self.log(f"  [{self.case_num:3d}] {name:40s} | {len(payload):6d}B | "
                f"manifest={ms:7.1f}ms poll={mp:5d}ms | final_state={fst}")
        
        self.results.append(result)
        return result

    # ============================================================
    # IMG4/DER Payload Generators
    # ============================================================

    @staticmethod
    def der_len(length):
        """Encode a DER length field."""
        if length < 0x80:
            return bytes([length])
        elif length < 0x100:
            return bytes([0x81, length])
        elif length < 0x10000:
            return bytes([0x82, (length >> 8) & 0xFF, length & 0xFF])
        elif length < 0x1000000:
            return bytes([0x83, (length >> 16) & 0xFF, (length >> 8) & 0xFF, length & 0xFF])
        else:
            return bytes([0x84, (length >> 24) & 0xFF, (length >> 16) & 0xFF,
                         (length >> 8) & 0xFF, length & 0xFF])

    @staticmethod
    def der_seq(contents):
        """Wrap contents in a DER SEQUENCE."""
        return b"\x30" + IMG4Fuzzer.der_len(len(contents)) + contents

    @staticmethod
    def der_ia5(text):
        """DER IA5String."""
        data = text.encode("ascii")
        return b"\x16" + IMG4Fuzzer.der_len(len(data)) + data

    @staticmethod
    def der_octet(data):
        """DER OCTET STRING."""
        return b"\x04" + IMG4Fuzzer.der_len(len(data)) + data

    @staticmethod
    def der_int(value):
        """DER INTEGER."""
        if value < 0x80:
            return b"\x02\x01" + bytes([value])
        elif value < 0x8000:
            return b"\x02\x02" + struct.pack(">H", value)
        elif value < 0x800000:
            return b"\x02\x03" + struct.pack(">I", value)[1:]
        else:
            return b"\x02\x04" + struct.pack(">I", value)

    @staticmethod
    def der_bool(value):
        """DER BOOLEAN."""
        return b"\x01\x01" + (b"\xFF" if value else b"\x00")

    def generate_payloads(self):
        """Generate test cases targeting different parser paths."""
        payloads = []
        D = IMG4Fuzzer  # shorthand

        # ===== BASELINE =====
        payloads.append(("baseline_zeroes", b"\x00" * 512))
        payloads.append(("baseline_random", os.urandom(512)))

        # ===== VALID-LOOKING IMG4 CONTAINERS =====

        # Minimal valid IMG4 structure:
        # SEQUENCE { IA5String "IMG4", SEQUENCE { IM4P }, [0] IM4M }
        
        # 1: Valid IM4P header, tiny payload
        im4p_payload = b"\xCC" * 16
        im4p = D.der_seq(
            D.der_ia5("IM4P") +
            D.der_ia5("ibss") +    # component: iBSS
            D.der_ia5("testing") + # version string
            D.der_octet(im4p_payload)
        )
        img4 = D.der_seq(D.der_ia5("IMG4") + im4p)
        payloads.append(("valid_img4_ibss", img4))

        # 2: IM4P with different component types
        for comp in ["ibot", "ibss", "ibec", "illb", "dtre", "krnl", "rkrn", "sepi", "rsep"]:
            im4p = D.der_seq(
                D.der_ia5("IM4P") + D.der_ia5(comp) +
                D.der_ia5("3865.0.0.4.7") + D.der_octet(b"\x41" * 64)
            )
            img4 = D.der_seq(D.der_ia5("IMG4") + im4p)
            payloads.append((f"img4_comp_{comp}", img4))

        # 3: IMG4 with IM4M (manifest) — this reaches the signature verification path
        im4m = b"\xA0" + D.der_len(20) + D.der_seq(
            D.der_ia5("IM4M") + D.der_int(0) + D.der_octet(b"\x00" * 8)
        )
        im4p = D.der_seq(
            D.der_ia5("IM4P") + D.der_ia5("ibss") +
            D.der_ia5("test") + D.der_octet(b"\xBB" * 32)
        )
        img4_with_manifest = D.der_seq(D.der_ia5("IMG4") + im4p + im4m)
        payloads.append(("img4_with_im4m", img4_with_manifest))

        # 4: IM4M with fake certificate chain
        fake_cert = D.der_seq(
            D.der_seq(D.der_int(2) + D.der_int(0x12345678)) +  # tbsCertificate
            D.der_seq(b"\x06\x09\x2A\x86\x48\x86\xF7\x0D\x01\x01\x0B") +  # sha256WithRSA OID
            b"\x03\x20" + b"\x00" * 32  # signature
        )
        im4m_with_cert = b"\xA0" + D.der_len(len(fake_cert) + 30) + D.der_seq(
            D.der_ia5("IM4M") + D.der_int(0) + 
            D.der_octet(fake_cert)
        )
        img4_cert = D.der_seq(D.der_ia5("IMG4") + im4p + im4m_with_cert)
        payloads.append(("img4_fake_cert", img4_cert))

        # ===== DER PARSER ATTACKS =====

        # 5: Huge DER length (integer overflow)
        payloads.append(("der_huge_len_32bit", 
            b"\x30\x84\xFF\xFF\xFF\xFF" + b"\x00" * 506))
        payloads.append(("der_huge_len_24bit",
            b"\x30\x83\xFF\xFF\xFF" + b"\x00" * 507))
        
        # 6: Length says more than data (truncated)
        payload = D.der_ia5("IMG4")
        payloads.append(("der_truncated_seq",
            b"\x30\x82\x10\x00" + payload))  # Claims 4096B, only has ~6B

        # 7: Negative / weird length encoding
        payloads.append(("der_len_0x80", b"\x30\x80" + b"\x00" * 510))  # indefinite length
        payloads.append(("der_len_0x85", b"\x30\x85\xFF\xFF\xFF\xFF\xFF" + b"\x00" * 505))  # 5-byte length
        payloads.append(("der_len_0xFF", b"\x30\xFF" + b"\x00" * 510))  # invalid length byte

        # 8: Deep nesting (stack overflow)
        deep = b"\x05\x00"  # NULL
        for i in range(200):
            inner_len = len(deep)
            if inner_len < 128:
                deep = b"\x30" + bytes([inner_len]) + deep
            else:
                deep = b"\x30" + D.der_len(inner_len) + deep
            if len(deep) > 3500: break
        payloads.append(("der_deep_nest_200", deep + b"\x00" * max(0, 512 - len(deep))))

        # Even deeper nesting with minimal payload (2 bytes each level)
        deep2 = b"\x05\x00"
        for i in range(500):
            if len(deep2) < 128:
                deep2 = b"\x30" + bytes([len(deep2)]) + deep2
            else:
                # Truncate to keep nesting going
                deep2 = b"\x30\x7F" + deep2[:127]
            if len(deep2) > 3500: break
        payloads.append(("der_deep_nest_500", deep2))

        # 9: All possible ASN.1 tag types at top level
        for tag in [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x0A, 0x0C, 0x13, 
                    0x16, 0x17, 0x1E, 0x30, 0x31, 0xA0, 0xA1, 0xA2, 0xA3]:
            payloads.append((f"der_tag_0x{tag:02X}",
                bytes([tag]) + b"\x82\x01\x00" + b"\x00" * 256))

        # 10: Context-specific tags [0]-[31]
        for ctx in [0, 1, 2, 3, 4, 5, 10, 15, 31]:
            tag_byte = 0xA0 | ctx  # constructed, context-specific
            payloads.append((f"der_ctx_{ctx}",
                bytes([tag_byte]) + D.der_len(256) + b"\x00" * 256))

        # ===== IMG4 STRUCTURE ATTACKS =====

        # 11: Wrong magic string
        for magic in ["IMG3", "img4", "IM4P", "AAAA", "\x00\x00\x00\x00"]:
            wrong = D.der_seq(D.der_ia5(magic) + D.der_octet(b"\x00" * 64))
            payloads.append((f"img4_magic_{magic.replace(chr(0),'NUL')}", wrong))

        # 12: IMG4 with empty IM4P 
        empty_im4p = D.der_seq(D.der_ia5("IM4P"))
        payloads.append(("img4_empty_im4p", D.der_seq(D.der_ia5("IMG4") + empty_im4p)))

        # 13: IMG4 with oversized IM4P payload
        big_im4p = D.der_seq(
            D.der_ia5("IM4P") + D.der_ia5("ibss") +
            D.der_ia5("ver") + D.der_octet(b"\xDD" * 3000)
        )
        payloads.append(("img4_big_payload_3k", D.der_seq(D.der_ia5("IMG4") + big_im4p)))

        # 14: IMG4 with multiple IM4P sections (parser confusion)
        dual_im4p = D.der_seq(
            D.der_ia5("IMG4") + 
            D.der_seq(D.der_ia5("IM4P") + D.der_ia5("ibss") + D.der_ia5("v1") + D.der_octet(b"\x11" * 32)) +
            D.der_seq(D.der_ia5("IM4P") + D.der_ia5("ibec") + D.der_ia5("v2") + D.der_octet(b"\x22" * 32))
        )
        payloads.append(("img4_dual_im4p", dual_im4p))

        # 15: IM4P with IV/Key fields (encrypted payload)
        im4p_enc = D.der_seq(
            D.der_ia5("IM4P") + D.der_ia5("ibss") + D.der_ia5("ver") +
            D.der_octet(b"\xEE" * 64) +
            D.der_octet(b"\x00" * 16) +  # IV
            D.der_octet(b"\x00" * 32)    # Key
        )
        payloads.append(("img4_encrypted_im4p", D.der_seq(D.der_ia5("IMG4") + im4p_enc)))

        # ===== SIZE VARIANTS =====
        
        # 16: Various sizes of valid-looking IMG4
        for sz in [16, 64, 128, 256, 1024, 2048, 4096]:
            pad_size = max(0, sz - 50)
            im4p = D.der_seq(D.der_ia5("IM4P") + D.der_ia5("ibss") +
                            D.der_ia5("v") + D.der_octet(b"\x00" * pad_size))
            payloads.append((f"img4_size_{sz}B", D.der_seq(D.der_ia5("IMG4") + im4p)))

        # ===== MACH-O HEADER (different parser path?) =====
        
        # 17: Mach-O ARM64 header
        macho = struct.pack("<IIIIIIII",
            0xFEEDFACF, 0x0100000C, 0, 0x02, 0, 0, 0, 0) + b"\x00" * 480
        payloads.append(("macho_arm64", macho))

        # 18: Mach-O FAT binary
        fat = struct.pack(">II", 0xCAFEBABE, 2)  # 2 architectures
        fat += struct.pack(">IIIII", 0x0100000C, 0, 0x1000, 0x100, 14)
        fat += struct.pack(">IIIII", 0x0000000C, 0, 0x2000, 0x100, 14)
        payloads.append(("macho_fat", fat + b"\x00" * (512 - len(fat))))

        # ===== APPLE SPECIFIC PAYLOADS =====

        # 19: iBoot image header
        iboot_hdr = b"iB1x" + struct.pack("<IIII", 0, 0x100, 0x800, 0xDEADBEEF) + b"\x00" * 476
        payloads.append(("iboot_header", iboot_hdr))

        # 20: All 0xFF (flash erased pattern)
        payloads.append(("all_0xFF", b"\xFF" * 512))

        # 21: Mix of sharp edge cases
        payloads.append(("edge_0x00010000", b"\x00\x01\x00\x00" * 128))
        payloads.append(("edge_alternating", bytes([i & 0xFF for i in range(512)])))

        return payloads

    # ============================================================
    # Run Fuzzer
    # ============================================================
    def run(self, max_cases=None):
        self.log("=" * 60)
        self.log("A12 SecureROM IMG4 Parser Fuzzer")
        self.log(f"Date: {datetime.now().isoformat()}")
        self.log("=" * 60)

        if not self.connect():
            self.log("No DFU device!", "ERR")
            return

        payloads = self.generate_payloads()
        if max_cases:
            payloads = payloads[:max_cases]
        
        self.log(f"Generated {len(payloads)} test cases")
        self.log(f"{'Case':>4s} | {'Name':40s} | {'Size':>6s} | {'Manifest':>10s} | {'Poll':>5s} | State")
        self.log("-" * 80)

        baseline_time = None
        
        for i, (name, payload) in enumerate(payloads):
            result = self.send_and_manifest(payload, name)
            
            if result.get("error"):
                self.log(f"  Error: {result['error']}", "WARN")
                # Try to recover
                time.sleep(1)
                if not self.connect():
                    self.log("Lost device! Waiting 5 sec...", "ERR")
                    time.sleep(5)
                    if not self.connect():
                        self.log("Device gone — stopping", "ERR")
                        break
                continue

            # Detect anomalies
            mt = result.get("manifest_time_ms", 0)
            mp = result.get("manifest_poll_ms", -1)
            fst = result.get("final_bState", -1)

            if baseline_time is None and mt > 100:
                baseline_time = mt

            # Check for timing anomalies
            if baseline_time and mt > 0:
                delta = mt - baseline_time
                if abs(delta) > 500:  # 500ms difference
                    self.finding(f"Timing anomaly: {name} ({delta:+.0f}ms)",
                               f"Expected ~{baseline_time:.0f}ms, got {mt:.0f}ms — different code path?",
                               "high")
                if mp > 0 and mp != 3000 and mp != 50:
                    self.finding(f"Unusual poll timeout: {name} (poll={mp}ms)",
                               f"Expected 50 or 3000ms, got {mp}ms", "medium")

            # Check for unexpected final states
            if fst not in (-1, -2, 2, 8, 10):
                self.finding(f"Unusual final state: {name} (state={fst})",
                           f"Expected dfuIDLE/ERROR/MANIFEST-WAIT, got state {fst}", "high")

            # Device crash
            if fst == -2:
                self.finding(f"Device crash: {name}",
                           f"Device stopped responding after manifest", "critical")

        # Summary
        self.log(f"\n{'='*60}")
        self.log("FUZZER SUMMARY")
        self.log(f"{'='*60}")
        self.log(f"Cases run: {len(self.results)}")
        crit = [f for f in self.findings if f["severity"] == "critical"]
        high = [f for f in self.findings if f["severity"] == "high"]
        med  = [f for f in self.findings if f["severity"] == "medium"]
        self.log(f"Findings: {len(self.findings)} ({len(crit)} CRITICAL, {len(high)} HIGH, {len(med)} MEDIUM)")
        for f in crit + high:
            self.log(f"  [{f['severity'].upper()}] Case {f['case']}: {f['title']}")

        # Timing analysis
        if len(self.results) > 2:
            times = [(r["name"], r.get("manifest_time_ms", 0)) for r in self.results if r.get("manifest_time_ms", 0) > 0]
            if times:
                times.sort(key=lambda x: x[1])
                self.log(f"\nTiming Analysis (sorted):")
                self.log(f"  Fastest: {times[0][0]} = {times[0][1]:.1f}ms")
                self.log(f"  Slowest: {times[-1][0]} = {times[-1][1]:.1f}ms")
                self.log(f"  Delta: {times[-1][1] - times[0][1]:.1f}ms")
                if len(times) > 5:
                    avg = sum(t for _, t in times) / len(times)
                    self.log(f"  Average: {avg:.1f}ms")
                    # Flag outliers (>2 stddev)
                    stddev = (sum((t - avg)**2 for _, t in times) / len(times)) ** 0.5
                    for name, t in times:
                        if abs(t - avg) > 2 * stddev:
                            self.log(f"  OUTLIER: {name} = {t:.1f}ms (avg={avg:.1f} ± {stddev:.1f})")

        # Save results
        out = {
            "findings": self.findings,
            "results": self.results,
            "timestamp": datetime.now().isoformat()
        }
        json_path = RESULTS_DIR / "a12_img4_fuzzer.json"
        with open(json_path, "w") as fh:
            def sanitize(o):
                if isinstance(o, bytes): return o.hex()
                if isinstance(o, dict): return {k: sanitize(v) for k, v in o.items()}
                if isinstance(o, list): return [sanitize(i) for i in o]
                return o
            json.dump(sanitize(out), fh, indent=2)
        log_path = RESULTS_DIR / "a12_img4_fuzzer_log.txt"
        with open(log_path, "w", encoding="utf-8") as fh:
            fh.write("\n".join(self.lines))
        self.log(f"\nSaved: {json_path}")
        self.log(f"Saved: {log_path}")


def main():
    import argparse
    p = argparse.ArgumentParser(description="A12 IMG4 Parser Fuzzer")
    p.add_argument("-n", "--max-cases", type=int, default=None, help="Max test cases to run")
    args = p.parse_args()
    
    f = IMG4Fuzzer()
    f.run(max_cases=args.max_cases)

if __name__ == "__main__":
    main()
