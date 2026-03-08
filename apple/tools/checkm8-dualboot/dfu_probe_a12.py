#!/usr/bin/env python3
"""
DFU SecureROM Probe — A12 (T8020) Attack Surface Scanner
=========================================================
On sait que:
  - checkm8 (heap UAF via USB) est corrigé dans A12
  - MAIS le SecureROM doit toujours parser USB + IMG4/DER avant de rejeter
  - Il y a potentiellement d'autres bugs dans ce code C burned-in

Ce script explore systématiquement le SecureROM A12 via DFU USB:
  1. USB Control Transfer fuzzing (bmRequestType, bRequest combos)
  2. DFU state machine edge cases 
  3. IMG4 header parsing edge cases
  4. Timing analysis (détection de chemins de code différents)
  5. DFU_DNLOAD boundary conditions (tailles limites)

SAFE: Aucune modification NAND. Pire cas = device reboot → re-enter DFU.
"""

import sys
import os
import json
import time
import struct
import hashlib
import traceback
from datetime import datetime
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent.resolve()
RESULTS_DIR = SCRIPT_DIR / "results"
RESULTS_DIR.mkdir(exist_ok=True)

try:
    import usb.core
    import usb.util
    import libusb_package
    import usb.backend.libusb1
    HAS_USB = True
except ImportError:
    HAS_USB = False
    print("[!] pyusb not found")

# Constants
APPLE_VID = 0x05AC
DFU_PID   = 0x1227

DFU_DNLOAD     = 1
DFU_UPLOAD     = 2
DFU_GET_STATUS = 3
DFU_CLR_STATUS = 4
DFU_GET_STATE  = 5
DFU_ABORT      = 6

# DFU states
DFU_STATES = {
    0: "appIDLE",
    1: "appDETACH", 
    2: "dfuIDLE",
    3: "dfuDNLOAD-SYNC",
    4: "dfuDNBUSY",
    5: "dfuDNLOAD-IDLE",
    6: "dfuMANIFEST-SYNC",
    7: "dfuMANIFEST",
    8: "dfuMANIFEST-WAIT-RESET",
    9: "dfuUPLOAD-IDLE",
    10: "dfuERROR",
    255: "unknown"
}

class DFUProbe:
    """Low-level DFU USB probe for SecureROM research."""
    
    def __init__(self):
        self.dev = None
        self.serial = None
        self.cpid = None
        self.findings = []
        self.log_lines = []
        
    def connect(self):
        """Connect to DFU device."""
        backend = usb.backend.libusb1.get_backend(
            find_library=libusb_package.find_library
        )
        self.dev = usb.core.find(
            idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend
        )
        if self.dev is None:
            print("[!] No DFU device found")
            return False
            
        try:
            self.serial = self.dev.serial_number
            for part in self.serial.split(" "):
                if part.startswith("CPID:"):
                    self.cpid = int(part.split(":")[1], 16)
        except:
            pass
            
        try:
            self.dev.set_configuration()
        except:
            pass
            
        self.log(f"Connected: CPID=0x{self.cpid:04X}, Serial={self.serial}")
        return True
    
    def log(self, msg, level="INFO"):
        ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        line = f"[{ts}] [{level:4s}] {msg}"
        print(line)
        self.log_lines.append(line)
    
    def finding(self, category, title, detail, severity="info"):
        """Record an interesting finding."""
        f = {
            "category": category,
            "title": title, 
            "detail": detail,
            "severity": severity,
            "timestamp": datetime.now().isoformat()
        }
        self.findings.append(f)
        marker = "!!" if severity in ("high", "critical") else "**" if severity == "medium" else "--"
        self.log(f"[{marker}] FINDING: {title} — {detail}", "FIND")
    
    def ctrl(self, bmRequestType, bRequest, wValue, wIndex, data_or_wLength, timeout=1000):
        """Raw USB control transfer with timing."""
        t0 = time.perf_counter()
        result = None
        error = None
        stall = False
        timeout_err = False
        
        try:
            result = self.dev.ctrl_transfer(
                bmRequestType, bRequest, wValue, wIndex,
                data_or_wLength, timeout=timeout
            )
        except usb.core.USBTimeoutError:
            timeout_err = True
        except usb.core.USBError as e:
            if e.errno == 32 or "pipe" in str(e).lower():
                stall = True
            else:
                error = str(e)
        
        elapsed_ms = (time.perf_counter() - t0) * 1000
        
        # ctrl_transfer returns array for IN, int (bytes written) for OUT
        if result is not None and not isinstance(result, (bytes, bytearray)):
            try:
                result_bytes = bytes(result)
            except TypeError:
                result_bytes = None  # int return from OUT transfer
        else:
            result_bytes = bytes(result) if result is not None else None
        
        data_len = len(result_bytes) if result_bytes is not None else (result if isinstance(result, int) else 0)
        
        return {
            "data": result_bytes,
            "length": data_len,
            "stall": stall,
            "timeout": timeout_err,
            "error": error,
            "time_ms": round(elapsed_ms, 3)
        }
    
    def get_dfu_state(self):
        """Get current DFU state byte."""
        r = self.ctrl(0xA1, DFU_GET_STATE, 0, 0, 1)
        if r["data"] and len(r["data"]) > 0:
            return r["data"][0]
        return -1
    
    def get_dfu_status(self):
        """Get DFU status (6 bytes)."""
        r = self.ctrl(0xA1, DFU_GET_STATUS, 0, 0, 6)
        if r["data"] and len(r["data"]) == 6:
            return {
                "bStatus": r["data"][0],
                "bwPollTimeout": r["data"][1] | (r["data"][2] << 8) | (r["data"][3] << 16),
                "bState": r["data"][4],
                "iString": r["data"][5],
                "state_name": DFU_STATES.get(r["data"][4], f"?{r['data'][4]}"),
                "time_ms": r["time_ms"]
            }
        return None
    
    def ensure_idle(self):
        """Return device to dfuIDLE."""
        st = self.get_dfu_status()
        if st is None:
            return False
        if st["bState"] == 2:
            return True
        if st["bState"] == 10:  # dfuERROR
            self.ctrl(0x21, DFU_CLR_STATUS, 0, 0, 0)
            time.sleep(0.01)
        elif st["bState"] == 5:  # dfuDNLOAD-IDLE
            self.ctrl(0x21, DFU_ABORT, 0, 0, 0)
            time.sleep(0.01)
        else:
            self.ctrl(0x21, DFU_ABORT, 0, 0, 0)
            time.sleep(0.01)
        st = self.get_dfu_status()
        return st is not None and st["bState"] == 2

    # ==================================================================
    # PROBE 1: USB Control Transfer Scan
    # ==================================================================
    def probe_usb_control_requests(self):
        """
        Scan all USB control transfer types.
        
        The SecureROM's USB stack handles various bRequest values.
        Standard DFU only uses bRequest 1-6, but the USB handler may
        have code paths for other values (vendor, class, standard).
        
        Interesting if: response != STALL for unexpected requests
        """
        self.log("=" * 60)
        self.log("PROBE 1: USB Control Request Scan")
        self.log("=" * 60)
        
        # Standard USB requests (bmRequestType bit patterns)
        request_types = [
            (0x00, "Std Dev OUT"),   (0x80, "Std Dev IN"),
            (0x01, "Std Iface OUT"), (0x81, "Std Iface IN"),
            (0x02, "Std EP OUT"),    (0x82, "Std EP IN"),
            (0x20, "Class Iface OUT"), (0xA0, "Class Iface IN"),  # DFU class
            (0x21, "Class Iface OUT"), (0xA1, "Class Iface IN"),
            (0x40, "Vendor Dev OUT"), (0xC0, "Vendor Dev IN"),
            (0x41, "Vendor Iface OUT"), (0xC1, "Vendor Iface IN"),
            (0x42, "Vendor EP OUT"), (0xC2, "Vendor EP IN"),
        ]
        
        interesting_requests = []
        
        for bmReqType, desc in request_types:
            is_in = bmReqType & 0x80
            
            for bRequest in range(0, 256):
                # Skip known DFU requests to avoid state changes
                if bmReqType in (0x21, 0xA1) and bRequest in range(1, 7):
                    continue
                
                if is_in:
                    r = self.ctrl(bmReqType, bRequest, 0, 0, 64, timeout=200)
                else:
                    r = self.ctrl(bmReqType, bRequest, 0, 0, b"\x00" * 8, timeout=200)
                
                if not r["stall"] and not r["timeout"] and r["error"] is None:
                    self.log(f"  [!] RESPONSE from bmReqType=0x{bmReqType:02X} bReq={bRequest}: "
                             f"len={r['length']} time={r['time_ms']:.1f}ms")
                    interesting_requests.append({
                        "bmRequestType": bmReqType,
                        "bRequest": bRequest,
                        "desc": desc,
                        "length": r["length"],
                        "data": r["data"].hex() if r["data"] else None,
                        "time_ms": r["time_ms"]
                    })
                    
                    if bRequest not in (0, 6, 8) and bmReqType not in (0xA1,):
                        self.finding("usb_ctrl", 
                            f"Unexpected USB response: 0x{bmReqType:02X}/0x{bRequest:02X}",
                            f"Got {r['length']}B response from non-standard request. "
                            f"Data: {r['data'].hex()[:32] if r['data'] else 'none'}",
                            "medium")
            
            # Quick reconnect check
            if self.get_dfu_state() < 0:
                self.log("[!] Device disconnected during scan, reconnecting...")
                time.sleep(1)
                if not self.connect():
                    self.log("[!] Failed to reconnect")
                    break
        
        self.log(f"  Found {len(interesting_requests)} responsive request types")
        return interesting_requests
    
    # ==================================================================
    # PROBE 2: DFU State Machine Edge Cases
    # ==================================================================
    def probe_state_machine(self):
        """
        Test DFU state machine transitions for anomalies.
        
        The DFU spec defines strict state transitions. SecureROM implementations
        may have bugs when receiving unexpected sequences of commands.
        """
        self.log("=" * 60)
        self.log("PROBE 2: DFU State Machine Edge Cases")
        self.log("=" * 60)
        
        results = []
        
        # Test 1: Double DNLOAD without GET_STATUS
        self.log("  [2a] Double DNLOAD without GET_STATUS...")
        self.ensure_idle()
        r1 = self.ctrl(0x21, DFU_DNLOAD, 0, 0, b"\xAA" * 0x100)
        r2 = self.ctrl(0x21, DFU_DNLOAD, 0, 0, b"\xBB" * 0x100)
        st = self.get_dfu_status()
        results.append({
            "test": "double_dnload",
            "r1_error": r1["error"], "r2_error": r2["error"],
            "final_state": st["state_name"] if st else "none"
        })
        if st and st["bState"] not in (2, 10):
            self.finding("state_machine", "Unusual state after double DNLOAD",
                        f"State: {st['state_name']}", "medium")
        self.ensure_idle()
        
        # Test 2: GET_STATUS without prior DNLOAD
        self.log("  [2b] Multiple GET_STATUS from dfuIDLE...")
        statuses = []
        for i in range(10):
            st = self.get_dfu_status()
            if st:
                statuses.append(st["bState"])
        # Check if state changes unexpectedly
        if len(set(statuses)) > 1:
            self.finding("state_machine", "State drift on repeated GET_STATUS",
                        f"States seen: {statuses}", "high")
        results.append({"test": "repeated_getstatus", "states": statuses})
        
        # Test 3: ABORT from every state
        self.log("  [2c] ABORT from various states...")
        # From dfuIDLE
        self.ensure_idle()
        r = self.ctrl(0x21, DFU_ABORT, 0, 0, 0)
        st = self.get_dfu_status()
        results.append({"test": "abort_from_idle", 
                        "error": r["error"], "stall": r["stall"],
                        "state_after": st["state_name"] if st else "none"})
        
        # From dfuDNLOAD-IDLE
        self.ensure_idle()
        self.ctrl(0x21, DFU_DNLOAD, 0, 0, b"\x00" * 0x100)
        self.get_dfu_status()  # → dfuDNLOAD-IDLE
        r = self.ctrl(0x21, DFU_ABORT, 0, 0, 0)
        st = self.get_dfu_status()
        results.append({"test": "abort_from_dnload_idle",
                        "error": r["error"], "stall": r["stall"],
                        "state_after": st["state_name"] if st else "none"})
        
        # Test 4: CLR_STATUS from non-error states (was the A11 checkm8 vector)
        self.log("  [2d] CLR_STATUS from non-error states...")
        self.ensure_idle()
        r = self.ctrl(0x21, DFU_CLR_STATUS, 0, 0, 0)
        st = self.get_dfu_status()
        results.append({"test": "clrstatus_from_idle",
                        "error": r["error"], "stall": r["stall"],
                        "state_after": st["state_name"] if st else "none"})
        if not r["stall"] and r["error"] is None:
            self.finding("state_machine", "CLR_STATUS accepted from dfuIDLE",
                        "SecureROM accepted CLR_STATUS outside dfuERROR — potential state confusion",
                        "high")
        
        # Test 5: UPLOAD from dfuIDLE (should stall on non-pwned)
        self.log("  [2e] DFU_UPLOAD from dfuIDLE...")
        self.ensure_idle()
        r = self.ctrl(0xA1, DFU_UPLOAD, 0, 0, 0x800)
        results.append({"test": "upload_from_idle",
                        "data_len": r["length"],
                        "stall": r["stall"],
                        "has_data": r["data"] is not None and len(r["data"]) > 0})
        if r["data"] and len(r["data"]) > 0 and any(b != 0 for b in r["data"]):
            self.finding("state_machine", "DFU_UPLOAD returned non-zero data!",
                        f"Got {len(r['data'])}B, first: {r['data'][:16].hex()}",
                        "critical")
        
        # Test 6: Zero-length DNLOAD (triggers manifest sequence per DFU spec)
        self.log("  [2f] Zero-length DNLOAD (manifest trigger)...")
        self.ensure_idle()
        r = self.ctrl(0x21, DFU_DNLOAD, 0, 0, b"")
        st = self.get_dfu_status()
        results.append({"test": "zero_dnload",
                        "error": r["error"], "stall": r["stall"],
                        "state_after": st["state_name"] if st else "none"})
        if st and st["bState"] in (6, 7, 8):
            self.finding("state_machine", "Manifest state reached via zero DNLOAD",
                        f"State: {st['state_name']} — device processing manifest",
                        "high")
        self.ensure_idle()
        
        self.log(f"  State machine tests complete: {len(results)} tests")
        return results

    # ==================================================================
    # PROBE 3: DNLOAD Size Boundary Conditions
    # ==================================================================
    def probe_dnload_boundaries(self):
        """
        Test various DNLOAD sizes to find edge cases.
        
        The SecureROM allocates buffers based on the transfer size.
        Interesting sizes: 0, 1, EP0_MAX_PACKET_SZ (0x40), 
        DFU_MAX_TRANSFER_SZ (0x800), just over, powers of 2, 
        0xFFFF, etc.
        """
        self.log("=" * 60)
        self.log("PROBE 3: DNLOAD Size Boundaries")
        self.log("=" * 60)
        
        test_sizes = [
            0, 1, 2, 3, 7, 8, 15, 16, 
            0x3F, 0x40, 0x41,      # EP0_MAX_PACKET_SZ boundary
            0x7F, 0x80, 0x81,
            0xFF, 0x100, 0x101,
            0x1FF, 0x200, 0x201,
            0x3FF, 0x400, 0x401,
            0x7FF, 0x800, 0x801,   # DFU_MAX_TRANSFER_SZ boundary  
            0xFFF, 0x1000, 0x1001,
            0x1FFF, 0x2000,
            0x4000, 0x8000, 0xFFFF
        ]
        
        results = []
        
        for size in test_sizes:
            self.ensure_idle()
            
            data = bytes([0xCC] * size) if size > 0 else b""
            
            r = self.ctrl(0x21, DFU_DNLOAD, 0, 0, data, timeout=2000)
            st = self.get_dfu_status()
            
            entry = {
                "size": size,
                "size_hex": f"0x{size:X}",
                "error": r["error"],
                "stall": r["stall"],
                "timeout": r["timeout"],
                "time_ms": r["time_ms"],
                "state_after": st["state_name"] if st else "none",
                "status": st["bStatus"] if st else -1
            }
            results.append(entry)
            
            status_str = "STALL" if r["stall"] else "TIMEOUT" if r["timeout"] else r["error"] or "OK"
            state_str = st["state_name"] if st else "?"
            
            self.log(f"  Size 0x{size:05X} ({size:6d}B): {status_str:8s} → {state_str:20s} ({r['time_ms']:.1f}ms)")
            
            # Any size > 0x800 that doesn't stall is interesting
            if size > 0x800 and not r["stall"] and r["error"] is None:
                self.finding("dnload_boundary", 
                    f"Large DNLOAD (0x{size:X}) accepted",
                    f"SecureROM accepted {size}B DNLOAD without error. "
                    f"Max expected: 0x800",
                    "high")
            
            # Timing anomaly detection
            if r["time_ms"] > 100 and not r["timeout"]:
                self.finding("timing", 
                    f"Slow response for size 0x{size:X}",
                    f"Took {r['time_ms']:.1f}ms — may indicate complex processing path",
                    "medium")
            
            self.ensure_idle()
        
        return results

    # ==================================================================
    # PROBE 4: IMG4 Parser Probing
    # ==================================================================
    def probe_img4_parser(self):
        """
        Send malformed IMG4/IM4P headers and observe timing/behavior.
        
        The SecureROM must parse the DER/ASN.1 IMG4 container to find
        the signature before verifying it. Bugs in the DER parser
        (integer overflows, malformed lengths, deep nesting) could
        lead to memory corruption.
        
        We use timing differences to detect which payloads reach 
        deeper parsing stages.
        """
        self.log("=" * 60)
        self.log("PROBE 4: IMG4/DER Parser Probing")
        self.log("=" * 60)
        
        test_payloads = {}
        
        # Baseline: random garbage
        test_payloads["random_garbage"] = bytes(range(256)) * 8
        
        # Valid DER SEQUENCE header, garbage inside
        test_payloads["der_seq_garbage"] = b"\x30\x82\x07\xF0" + b"\xCC" * 0x7F0
        
        # Valid IMG4 magic start (IM4P)
        # DER: SEQUENCE { IA5STRING "IM4P", IA5STRING "ibot", IA5STRING "desc", OCTETSTRING data }
        im4p_hdr = (
            b"\x30\x82\x07\xF0"  # SEQUENCE, length ~2K
            b"\x16\x04" b"IM4P"  # IA5STRING "IM4P"
            b"\x16\x04" b"ibss"  # IA5STRING type = "ibss" 
            b"\x16\x0E" b"iBSS for test"  # description
            b"\x04\x82\x07\xD0"  # OCTET STRING, big payload
        )
        test_payloads["valid_im4p_header"] = im4p_hdr + b"\x00" * 0x7D0
        
        # IMG4 wrapper around IM4P
        img4_hdr = (
            b"\x30\x82\x07\xF0"  # outer SEQUENCE
            b"\x16\x04" b"IMG4"  # IA5STRING "IMG4"
            b"\x30\x82\x07\xE0"  # inner SEQUENCE (IM4P)
            b"\x16\x04" b"IM4P"  
            b"\x16\x04" b"ibss"  
            b"\x16\x04" b"test"  
            b"\x04\x82\x07\xC0"  # OCTET STRING payload
        )
        test_payloads["valid_img4_header"] = img4_hdr + b"\x00" * 0x7C0
        
        # DER with huge length (integer overflow test)
        test_payloads["huge_der_length"] = b"\x30\x84\xFF\xFF\xFF\xFF" + b"\x00" * 0x100
        
        # DER with deeply nested SEQUENCEs (stack overflow test)
        nested = b""
        for i in range(200):
            nested = b"\x30\x82" + struct.pack(">H", len(nested) + 4) + nested
        test_payloads["deep_nesting"] = nested[:0x800]
        
        # IMG4 with incorrect type tag
        test_payloads["wrong_type_tag"] = img4_hdr.replace(b"ibss", b"XXXX") + b"\x00" * 0x7C0
        
        # DER with zero length fields
        test_payloads["zero_length_der"] = b"\x30\x00" * 0x400
        
        # Just the DFU suffix (16 bytes as per spec)
        dfu_suffix = struct.pack("<HHHH", 0x0100, 0x05AC, 0x1227, 0x0110)
        dfu_suffix += b"\x55\x46\x44"  # "UFD" (reverse "DFU")
        dfu_suffix += bytes([16])       # bLength
        dfu_suffix += b"\x00" * 4       # CRC pad
        test_payloads["dfu_suffix_only"] = dfu_suffix
        
        # Valid-looking iBSS header (Mach-O feedfacf)
        macho_hdr = struct.pack("<IIIIIIII",
            0xFEEDFACF,  # magic (Mach-O 64)
            0x0100000C,  # cputype ARM64
            0x00000002,  # cpusubtype
            0x00000005,  # filetype (preload)
            0x00000001,  # ncmds
            0x00000050,  # sizeofcmds
            0x00000001,  # flags
            0x00000000   # reserved
        )
        test_payloads["macho_header"] = macho_hdr + b"\x00" * (0x800 - len(macho_hdr))
        
        results = []
        # First get baseline timing for a simple DNLOAD
        self.ensure_idle()
        baseline = self.ctrl(0x21, DFU_DNLOAD, 0, 0, b"\x00" * 0x800)
        baseline_time = baseline["time_ms"]
        self.ensure_idle()
        
        self.log(f"  Baseline DNLOAD time: {baseline_time:.1f}ms")
        
        for name, payload in test_payloads.items():
            self.ensure_idle()
            
            # Send the payload
            send_time = time.perf_counter()
            r_send = self.ctrl(0x21, DFU_DNLOAD, 0, 0, payload[:0x800], timeout=5000)
            
            # Immediately get status (this is where parsing happens)
            r_status = self.ctrl(0xA1, DFU_GET_STATUS, 0, 0, 6, timeout=5000)
            total_ms = (time.perf_counter() - send_time) * 1000
            
            st = None
            if r_status["data"] and len(r_status["data"]) == 6:
                st = {
                    "bStatus": r_status["data"][0],
                    "bState": r_status["data"][4],
                    "state_name": DFU_STATES.get(r_status["data"][4], "?")
                }
            
            entry = {
                "name": name,
                "size": len(payload),
                "send_ok": r_send["error"] is None and not r_send["stall"],
                "send_time_ms": r_send["time_ms"],
                "status_state": st["state_name"] if st else "none",
                "status_code": st["bStatus"] if st else -1,
                "total_time_ms": round(total_ms, 3),
                "time_delta_vs_baseline": round(total_ms - baseline_time, 3) if baseline_time else 0
            }
            results.append(entry)
            
            status_str = entry["status_state"]
            delta = entry["time_delta_vs_baseline"]
            delta_str = f"+{delta:.1f}ms" if delta > 0 else f"{delta:.1f}ms"
            
            self.log(f"  {name:25s}: {status_str:20s} total={total_ms:.1f}ms ({delta_str})")
            
            # Timing anomaly: > 2x baseline suggests deeper parsing
            if total_ms > baseline_time * 2 and total_ms > 10:
                self.finding("img4_parser",
                    f"Timing anomaly for '{name}'",
                    f"Took {total_ms:.1f}ms vs baseline {baseline_time:.1f}ms "
                    f"({total_ms/baseline_time:.1f}x slower). "
                    f"Payload may reach deeper SecureROM parsing code.",
                    "medium")
            
            # State anomaly: anything other than dfuDNLOAD-IDLE or dfuERROR after DNLOAD
            if st and st["bState"] not in (3, 4, 5, 10, 2):
                self.finding("img4_parser",
                    f"Unexpected state for '{name}'",
                    f"State: {st['state_name']} — unexpected after DNLOAD+GETSTATUS",
                    "high")
            
            self.ensure_idle()
        
        return results

    # ==================================================================
    # PROBE 5: USB Reset Timing Window
    # ==================================================================
    def probe_usb_reset_timing(self):
        """
        Test USB reset at different points during DFU transfers.
        
        checkm8 was about USB reset during data phase creating a UAF.
        A12 fixed the specific bug, but the timing window between
        allocation and use may still have edge cases with different
        transfer patterns.
        """
        self.log("=" * 60)
        self.log("PROBE 5: USB Reset Timing Windows")
        self.log("=" * 60)
        
        results = []
        delays = [0, 0.0001, 0.0005, 0.001, 0.002, 0.005, 0.01, 0.05, 0.1]
        
        for delay in delays:
            self.ensure_idle()
            
            # Start a DNLOAD transfer
            self.ctrl(0x21, DFU_DNLOAD, 0, 0, b"\xAB" * 0x800)
            
            # Wait specific delay
            time.sleep(delay)
            
            # USB reset
            try:
                self.dev.reset()
            except:
                pass
            
            time.sleep(1.0)
            
            # Reconnect and check state
            try:
                self.connect()
                st = self.get_dfu_status()
                serial = self.serial or ""
                
                entry = {
                    "delay_ms": delay * 1000,
                    "reconnected": True,
                    "state": st["state_name"] if st else "none",
                    "serial_changed": "PWND" in serial or serial != self.serial,
                    "serial": serial[:50]
                }
                
                if "PWND" in serial:
                    self.finding("usb_reset", 
                        f"PWNED via reset timing at {delay*1000:.1f}ms!",
                        f"Serial: {serial}",
                        "critical")
                
                self.log(f"  Delay {delay*1000:6.1f}ms: reconnected, state={entry['state']}")
                
            except Exception as e:
                entry = {
                    "delay_ms": delay * 1000,
                    "reconnected": False,
                    "error": str(e)
                }
                self.log(f"  Delay {delay*1000:6.1f}ms: reconnect failed — {e}")
            
            results.append(entry)
        
        return results

    # ==================================================================
    # PROBE 6: wValue / wIndex Exploration  
    # ==================================================================
    def probe_wvalue_windex(self):
        """
        Test non-standard wValue and wIndex in DFU requests.
        
        Standard DFU uses wValue=0 wIndex=0. The SecureROM handler
        may have alternate code paths for different wValue/wIndex.
        """
        self.log("=" * 60)
        self.log("PROBE 6: wValue/wIndex Exploration")
        self.log("=" * 60)
        
        results = []
        
        # Test various wValue for DFU_DNLOAD
        self.log("  Testing wValue variations on DFU_DNLOAD...")
        test_values = [0, 1, 2, 0xFF, 0x100, 0xFFFF]
        for wval in test_values:
            self.ensure_idle()
            r = self.ctrl(0x21, DFU_DNLOAD, wval, 0, b"\x00" * 0x100)
            st = self.get_dfu_status()
            state = st["state_name"] if st else "none"
            ok = not r["stall"] and r["error"] is None
            self.log(f"    DNLOAD wValue=0x{wval:04X}: {'OK' if ok else 'FAIL'} → {state}")
            if ok and wval != 0:
                self.finding("wvalue", 
                    f"DNLOAD accepted with wValue=0x{wval:04X}",
                    "Non-zero wValue accepted — may index different buffer/block",
                    "medium")
            results.append({"request": "DNLOAD", "wValue": wval, "ok": ok, "state": state})
            self.ensure_idle()
        
        # Test various wIndex for DFU_DNLOAD
        self.log("  Testing wIndex variations on DFU_DNLOAD...")
        for widx in test_values:
            self.ensure_idle()
            r = self.ctrl(0x21, DFU_DNLOAD, 0, widx, b"\x00" * 0x100)
            st = self.get_dfu_status()
            state = st["state_name"] if st else "none"
            ok = not r["stall"] and r["error"] is None
            self.log(f"    DNLOAD wIndex=0x{widx:04X}: {'OK' if ok else 'FAIL'} → {state}")
            if ok and widx != 0:
                self.finding("windex",
                    f"DNLOAD accepted with wIndex=0x{widx:04X}", 
                    "Non-zero wIndex accepted — may target different interface/endpoint",
                    "medium")
            results.append({"request": "DNLOAD", "wIndex": widx, "ok": ok, "state": state})
            self.ensure_idle()
        
        # Test DFU_UPLOAD with different wValue (gaster uses wValue=0xFFFF)
        self.log("  Testing wValue variations on DFU_UPLOAD...")
        for wval in [0, 1, 0xFF, 0xFFFF]:
            self.ensure_idle()
            r = self.ctrl(0xA1, DFU_UPLOAD, wval, 0, 0x40)
            has_data = r["data"] is not None and len(r["data"]) > 0
            nonzero = has_data and any(b != 0 for b in r["data"])
            self.log(f"    UPLOAD wValue=0x{wval:04X}: len={r['length']} nonzero={nonzero}")
            if nonzero:
                self.finding("wvalue",
                    f"DFU_UPLOAD returned data with wValue=0x{wval:04X}!",
                    f"Data: {r['data'][:32].hex()}",
                    "critical")
            results.append({"request": "UPLOAD", "wValue": wval, "has_data": has_data, "nonzero": nonzero})
        
        return results

    # ==================================================================
    # PROBE 7: String Descriptor Leak
    # ==================================================================
    def probe_string_descriptors(self):
        """
        Enumerate USB string descriptors.
        
        The SecureROM serves USB string descriptors (like the serial number
        with CPID/ECID). Requesting string indices beyond the used range
        may leak memory or hit uninitialized buffers.
        """
        self.log("=" * 60)
        self.log("PROBE 7: USB String Descriptor Enumeration")
        self.log("=" * 60)
        
        results = []
        
        for idx in range(0, 256):
            # GET_DESCRIPTOR, type=3 (string), index=idx, langid=0x0409
            r = self.ctrl(0x80, 6, (3 << 8) | idx, 0x0409, 255, timeout=200)
            
            if r["data"] and len(r["data"]) > 2:
                # Parse USB string descriptor
                desc_len = r["data"][0]
                desc_type = r["data"][1]
                if desc_type == 3:  # String descriptor
                    try:
                        text = r["data"][2:desc_len].decode("utf-16-le", errors="replace")
                    except:
                        text = r["data"][2:desc_len].hex()
                    
                    self.log(f"  String[{idx:3d}]: \"{text}\" ({desc_len}B)")
                    results.append({"index": idx, "text": text, "length": desc_len,
                                   "raw": r["data"][:desc_len].hex()})
                    
                    if idx > 10:
                        self.finding("string_desc",
                            f"Unexpected string descriptor at index {idx}",
                            f"Content: \"{text[:50]}\"",
                            "medium")
            elif r["data"] and len(r["data"]) > 0:
                self.log(f"  String[{idx:3d}]: raw={r['data'].hex()}")
                results.append({"index": idx, "raw": r["data"].hex()})
        
        self.log(f"  Found {len(results)} string descriptors")
        return results

    # ==================================================================
    # Master run
    # ==================================================================
    def run_all(self):
        """Run all probes and save results."""
        self.log("=" * 60)
        self.log("T8020 (A12) SecureROM DFU Probe")
        self.log(f"Date: {datetime.now().isoformat()}")
        self.log("=" * 60)
        
        if not self.connect():
            return None
        
        self.log(f"\nDevice: CPID=0x{self.cpid:04X}")
        self.log(f"Serial: {self.serial}\n")
        
        all_results = {
            "device": {
                "cpid": f"0x{self.cpid:04X}",
                "serial": self.serial,
                "timestamp": datetime.now().isoformat()
            }
        }
        
        try:
            # Probe 7 first (low risk)
            all_results["string_descriptors"] = self.probe_string_descriptors()
            
            # Probe 6
            all_results["wvalue_windex"] = self.probe_wvalue_windex()
            
            # Probe 2
            all_results["state_machine"] = self.probe_state_machine()
            
            # Probe 3
            all_results["dnload_boundaries"] = self.probe_dnload_boundaries()
            
            # Probe 4
            all_results["img4_parser"] = self.probe_img4_parser()
            
            # Probe 1 (long, do last — can be skipped with --fast)
            if "--fast" not in sys.argv:
                all_results["usb_control_scan"] = self.probe_usb_control_requests()
            
            # Probe 5 (reset timing — causes reconnects)
            if "--with-reset" in sys.argv:
                all_results["usb_reset_timing"] = self.probe_usb_reset_timing()
            
        except Exception as e:
            self.log(f"[!] Probe error: {e}", "ERR")
            traceback.print_exc()
        
        # Summary
        all_results["findings"] = self.findings
        all_results["total_findings"] = len(self.findings)
        
        critical = [f for f in self.findings if f["severity"] == "critical"]
        high = [f for f in self.findings if f["severity"] == "high"]
        medium = [f for f in self.findings if f["severity"] == "medium"]
        
        self.log("\n" + "=" * 60)
        self.log("SUMMARY")
        self.log("=" * 60)
        self.log(f"Total findings: {len(self.findings)}")
        self.log(f"  CRITICAL: {len(critical)}")
        self.log(f"  HIGH:     {len(high)}")
        self.log(f"  MEDIUM:   {len(medium)}")
        
        for f in critical + high:
            self.log(f"  → {f['title']}: {f['detail'][:80]}")
        
        # Save results
        result_file = RESULTS_DIR / "a12_probe_results.json"
        with open(result_file, "w") as fp:
            json.dump(all_results, fp, indent=2, default=str)
        self.log(f"\nResults saved: {result_file}")
        
        log_file = RESULTS_DIR / "a12_probe_log.txt"
        with open(log_file, "w", encoding="utf-8") as fp:
            fp.write("\n".join(self.log_lines))
        self.log(f"Log saved: {log_file}")
        
        return all_results


# ==================================================================
# CLI
# ==================================================================
if __name__ == "__main__":
    if not HAS_USB:
        sys.exit(1)
    
    probe = DFUProbe()
    
    if "--help" in sys.argv:
        print("Usage: python dfu_probe_a12.py [options]")
        print("  --fast          Skip full USB control scan (Probe 1)")
        print("  --with-reset    Include USB reset timing (Probe 5)")
        print("  --probe N       Run only probe N (1-7)")
        print("  --help          This help")
        sys.exit(0)
    
    results = probe.run_all()
    
    if results:
        n = results.get("total_findings", 0)
        if n > 0:
            print(f"\n[+] {n} findings — check results/a12_probe_results.json")
        else:
            print("\n[-] No findings. SecureROM looks tight.")
