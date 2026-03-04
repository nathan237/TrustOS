#!/usr/bin/env python3
"""
A12 DFU Overflow Probe — Focused Multi-Block Investigation
============================================================
Phase 1+2 findings:
  - Buffer accepts 4096 bytes max per transfer
  - wValue (block #) accepts 0-0xFFFF
  - 10/10 rapid DNLOAD bursts accepted without GET_STATUS
  - GET_STATUS returns bwPollTimeout=50ms, we need to respect it
  - CLR_STATUS works from dfuIDLE
  - DETACH accepted from dfuIDLE

This script carefully tests:
  1) Proper DFU state machine with 50ms poll wait
  2) Multi-block DNLOAD: Does block N write to buffer + N*4096?
  3) Accumulation test: How much data can we feed before error/crash?
  4) Rapid burst: What happens when we skip status checks?
  5) State verification after each step
"""

import sys, os, time, struct, json, traceback
from datetime import datetime
from pathlib import Path

RESULTS_DIR = Path(__file__).parent.resolve() / "results"
RESULTS_DIR.mkdir(exist_ok=True)

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC
DFU_PID   = 0x1227
POLL_DELAY = 0.060  # 60ms — device needs 50ms

DFU_DNLOAD     = 1
DFU_UPLOAD     = 2
DFU_GET_STATUS = 3
DFU_CLR_STATUS = 4
DFU_GET_STATE  = 5
DFU_ABORT      = 6

DFU_STATES = {
    0: "appIDLE", 1: "appDETACH", 2: "dfuIDLE",
    3: "dfuDNLOAD-SYNC", 4: "dfuDNBUSY", 5: "dfuDNLOAD-IDLE",
    6: "dfuMANIFEST-SYNC", 7: "dfuMANIFEST", 8: "dfuMANIFEST-WAIT-RESET",
    9: "dfuUPLOAD-IDLE", 10: "dfuERROR",
}
DFU_STATUS = {
    0x00: "OK", 0x01: "errTARGET", 0x02: "errFILE", 0x03: "errWRITE",
    0x04: "errERASE", 0x05: "errCHECK_ERASED", 0x06: "errPROG",
    0x07: "errVERIFY", 0x08: "errADDRESS", 0x09: "errNOTDONE",
    0x0A: "errFIRMWARE", 0x0B: "errVENDOR", 0x0C: "errUSBR",
    0x0D: "errPOR", 0x0E: "errUNKNOWN", 0x0F: "errSTALLEDPKT",
}

class A12OverflowProbe:
    def __init__(self):
        self.dev = None
        self.serial = ""
        self.lines = []
        self.findings = []

    def log(self, msg, level="INFO"):
        ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        line = f"[{ts}] [{level:4s}] {msg}"
        print(line)
        self.lines.append(line)

    def finding(self, title, detail, severity="high"):
        f = {"title": title, "detail": detail, "severity": severity,
             "timestamp": datetime.now().isoformat()}
        self.findings.append(f)
        self.log(f"*** FINDING [{severity.upper()}]: {title} — {detail}", "FIND")

    def connect(self):
        backend = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
        self.dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend)
        if not self.dev:
            self.log("No DFU device found", "ERR")
            return False
        try:
            self.serial = self.dev.serial_number or ""
        except: pass
        try:
            self.dev.set_configuration()
        except: pass
        self.log(f"Connected: {self.serial[:60]}")
        return True

    def ctrl(self, bmRT, bReq, wVal, wIdx, data_or_len, timeout=2000):
        """USB control transfer. Returns (data_bytes_or_None, error_string_or_None)."""
        try:
            r = self.dev.ctrl_transfer(bmRT, bReq, wVal, wIdx, data_or_len, timeout=timeout)
            if isinstance(r, int):
                return (None, None)  # OUT transfer OK
            return (bytes(r), None)
        except usb.core.USBTimeoutError:
            return (None, "TIMEOUT")
        except usb.core.USBError as e:
            if e.errno == 32 or "pipe" in str(e).lower():
                return (None, "STALL")
            return (None, str(e))

    def dfu_get_status(self):
        """GET_STATUS → (bStatus, bState, bwPollTimeout, raw_hex) or None."""
        data, err = self.ctrl(0xA1, DFU_GET_STATUS, 0, 0, 6)
        if data and len(data) >= 6:
            return {
                "bStatus": data[0], "status": DFU_STATUS.get(data[0], f"?0x{data[0]:02x}"),
                "bState": data[4], "state": DFU_STATES.get(data[4], f"?{data[4]}"),
                "poll_ms": data[1] | (data[2]<<8) | (data[3]<<16),
                "raw": data.hex()
            }
        return None

    def dfu_get_state(self):
        """GET_STATE → bState byte or -1."""
        data, err = self.ctrl(0xA1, DFU_GET_STATE, 0, 0, 1)
        if data and len(data) >= 1:
            return data[0]
        return -1

    def dfu_dnload(self, payload, block=0, wIndex=0):
        """DNLOAD → (ok: bool, error_string)."""
        _, err = self.ctrl(0x21, DFU_DNLOAD, block, wIndex, payload)
        return (err is None, err)

    def dfu_upload(self, length=4096, block=0):
        """UPLOAD → (data_bytes, error_string)."""
        return self.ctrl(0xA1, DFU_UPLOAD, block, 0, length)

    def dfu_abort(self):
        self.ctrl(0x21, DFU_ABORT, 0, 0, 0)

    def dfu_clr_status(self):
        self.ctrl(0x21, DFU_CLR_STATUS, 0, 0, 0)

    def wait_poll(self, poll_ms=50):
        """Wait the poll timeout plus a small margin."""
        time.sleep(max(poll_ms / 1000.0, 0.01) + 0.010)

    def reset_to_idle(self):
        """Robustly return to dfuIDLE, with retries."""
        for attempt in range(10):
            time.sleep(0.020)
            st = self.dfu_get_status()
            if st is None:
                # Try abort, then clr_status
                self.dfu_abort()
                time.sleep(0.050)
                self.dfu_clr_status()
                time.sleep(0.050)
                continue
            if st["bState"] == 2:  # dfuIDLE
                return True
            if st["bState"] == 10:  # dfuERROR
                self.dfu_clr_status()
                time.sleep(0.050)
            else:
                self.dfu_abort()
                time.sleep(0.050)
        # Last check
        st = self.dfu_get_status()
        return st is not None and st["bState"] == 2

    def fmt_st(self, st):
        if st is None: return "None"
        return f"{st['state']} ({st['status']}) poll={st['poll_ms']}ms raw={st['raw']}"

    # ============================================================
    # TEST 1: Proper DFU State Machine Trace
    # ============================================================
    def test_state_trace(self):
        self.log("=" * 60)
        self.log("TEST 1: DFU State Machine Full Trace")
        self.log("=" * 60)

        if not self.reset_to_idle():
            self.log("Cannot reach dfuIDLE!", "ERR")
            return

        # Step 1: Initial state
        st = self.dfu_get_status()
        self.log(f"  [1] Initial: {self.fmt_st(st)}")

        # Step 2: DNLOAD 256B (block 0)
        ok, err = self.dfu_dnload(b"\x41" * 256, block=0)
        self.log(f"  [2] DNLOAD 256B block=0: ok={ok} err={err}")

        # Step 3: Poll status (should go: dfuDNLOAD-SYNC → dfuDNBUSY → dfuDNLOAD-IDLE)
        for i in range(10):
            st = self.dfu_get_status()
            self.log(f"  [3.{i}] GET_STATUS: {self.fmt_st(st)}")
            if st and st["bState"] in (2, 5, 10):  # idle, dnload-idle, error
                break
            self.wait_poll(st["poll_ms"] if st else 50)

        # Step 4: Second DNLOAD (block 1) — from dfuDNLOAD-IDLE
        if st and st["bState"] == 5:
            ok2, err2 = self.dfu_dnload(b"\x42" * 256, block=1)
            self.log(f"  [4] DNLOAD 256B block=1: ok={ok2} err={err2}")
            for i in range(5):
                st = self.dfu_get_status()
                self.log(f"  [4.{i}] GET_STATUS: {self.fmt_st(st)}")
                if st and st["bState"] in (2, 5, 10):
                    break
                self.wait_poll(st["poll_ms"] if st else 50)

        # Step 5: Zero-length DNLOAD (manifest trigger)
        if st and st["bState"] == 5:
            self.log(f"  [5] Zero-length DNLOAD (manifest trigger)...")
            ok3, err3 = self.dfu_dnload(b"", block=2)
            self.log(f"  [5] Zero-len DNLOAD: ok={ok3} err={err3}")
            for i in range(10):
                st = self.dfu_get_status()
                self.log(f"  [5.{i}] GET_STATUS: {self.fmt_st(st)}")
                if st is None:
                    self.log(f"  [5.{i}] Status returned None — device processing?")
                    time.sleep(0.100)
                    continue
                if st["bState"] in (2, 10, 8):  # idle, error, manifest-wait-reset
                    break
                self.wait_poll(st["poll_ms"] if st else 100)
            if st and st["bState"] == 10:
                self.log(f"  [5] Manifest resulted in dfuERROR: {st['status']} — expected (invalid firmware)")
            elif st and st["bState"] == 8:
                self.finding("Manifest accepted!", 
                           f"SecureROM accepted manifest of garbage data! State={st['state']}", "critical")

        # Step 6: Return to idle
        self.reset_to_idle()
        st = self.dfu_get_status()
        self.log(f"  [6] Final: {self.fmt_st(st)}")

    # ============================================================
    # TEST 2: Multi-Block Accumulation (Heap Overflow Hypothesis)
    # ============================================================
    def test_multiblock_overflow(self):
        self.log("=" * 60)
        self.log("TEST 2: Multi-Block DNLOAD Accumulation")
        self.log("=" * 60)
        self.log("  Hypothesis: wValue is block# → data writes to buffer + block*size")
        self.log("  If buffer is 4KB and we send block 0 (4KB) + block 1 (4KB), total 8KB")
        self.log("  This could overflow the DFU buffer into adjacent memory")
        self.log("")

        if not self.reset_to_idle():
            self.log("Cannot reach dfuIDLE!", "ERR")
            return

        # Test: Send increasing number of blocks
        for num_blocks in [1, 2, 3, 4, 8, 16]:
            self.reset_to_idle()
            self.log(f"  --- {num_blocks} blocks × 256B (total {num_blocks * 256}B) ---")
            
            all_ok = True
            for blk in range(num_blocks):
                marker = bytes([0xA0 + (blk & 0x0F)]) * 256
                ok, err = self.dfu_dnload(marker, block=blk)
                if not ok:
                    self.log(f"    Block {blk}: FAILED ({err})")
                    all_ok = False
                    break
                # GET_STATUS to advance state machine
                st = self.dfu_get_status()
                if st is None:
                    self.log(f"    Block {blk}: sent OK but GET_STATUS returned None")
                    time.sleep(0.060)
                    st = self.dfu_get_status()
                if st:
                    self.log(f"    Block {blk}: OK → {st['state']} ({st['status']})")
                    if st["bState"] == 10:  # error
                        self.log(f"    ERROR after block {blk}: {st['status']}")
                        all_ok = False
                        break
                    if st["bState"] != 5:  # not dfuDNLOAD-IDLE
                        # Wait poll timeout
                        self.wait_poll(st["poll_ms"])
                        st = self.dfu_get_status()
                        self.log(f"    Block {blk} (poll): → {st['state'] if st else '?'}")
                else:
                    self.log(f"    Block {blk}: sent OK, no status")
            
            if all_ok:
                self.log(f"  ✓ {num_blocks} blocks accepted!")
                if num_blocks > 1:
                    self.finding(f"{num_blocks} blocks × 256B ({num_blocks*256}B total) accepted",
                               f"Device accumulated {num_blocks*256}B across {num_blocks} blocks. "
                               f"Buffer overflow possible if buffer < {num_blocks*256}B",
                               "high" if num_blocks <= 4 else "critical")

        # Test: Full-size 4KB blocks
        self.log("")
        self.log("  --- Full-size 4KB blocks ---")
        for num_blocks in [1, 2, 3]:
            self.reset_to_idle()
            self.log(f"  >> {num_blocks}×4096B = {num_blocks*4096}B total")
            succeeded = 0
            for blk in range(num_blocks):
                marker = bytes([0xF0 + blk]) * 4096
                ok, err = self.dfu_dnload(marker, block=blk)
                st = self.dfu_get_status()
                self.wait_poll(50)
                st2 = self.dfu_get_status() if st and st["bState"] not in (5, 10) else st
                final_st = st2 or st
                state_name = final_st["state"] if final_st else "?"
                self.log(f"    4KB Block {blk}: ok={ok} → {state_name}")
                if not ok or (final_st and final_st["bState"] == 10):
                    break
                succeeded += 1
            
            if succeeded == num_blocks and num_blocks > 1:
                self.finding(f"{num_blocks}×4KB blocks accepted ({num_blocks*4096}B)",
                           f"Device accepted {num_blocks*4096}B total — buffer overflow into heap likely!",
                           "critical")

    # ============================================================
    # TEST 3: Rapid Fire Burst (Race Condition)
    # ============================================================
    def test_rapid_burst(self):
        self.log("=" * 60)
        self.log("TEST 3: Rapid-Fire DNLOAD Burst (Race Condition)")
        self.log("=" * 60)

        if not self.reset_to_idle():
            return

        # Send N blocks as fast as possible, no GET_STATUS between them
        for burst_size in [5, 10, 20, 50]:
            self.reset_to_idle()
            ok_count = 0
            stall_count = 0
            err_count = 0
            t_start = time.perf_counter()
            for i in range(burst_size):
                ok, err = self.dfu_dnload(bytes([i & 0xFF]) * 64, block=i)
                if ok:
                    ok_count += 1
                elif err == "STALL":
                    stall_count += 1
                else:
                    err_count += 1
            t_elapsed = (time.perf_counter() - t_start) * 1000
            
            # Check final state
            st = self.dfu_get_status()
            self.log(f"  Burst {burst_size}: OK={ok_count} STALL={stall_count} ERR={err_count} "
                    f"time={t_elapsed:.1f}ms → {self.fmt_st(st)}")
            
            if ok_count == burst_size:
                self.finding(f"Burst of {burst_size} accepted (no state check)",
                           f"All {burst_size} DNLOAD requests accepted in {t_elapsed:.1f}ms — "
                           f"no state machine enforcement, data piling up?",
                           "high" if burst_size < 20 else "critical")

        # Check if device is still alive
        self.reset_to_idle()
        st = self.dfu_get_status()
        self.log(f"  After all bursts: {self.fmt_st(st)}")

    # ============================================================
    # TEST 4: Block Number as Memory Offset  
    # ============================================================
    def test_block_offset(self):
        self.log("=" * 60)
        self.log("TEST 4: Block Number as Memory Offset")
        self.log("=" * 60)
        self.log("  If DNLOAD with block=N writes to buf + N*blocksize,")
        self.log("  then block=0x100 with 256B blocks writes 64KB away (buf+0x10000)")
        self.log("")

        if not self.reset_to_idle():
            return

        # Send to distant block numbers and check for crashes/errors
        distant_blocks = [0, 1, 0x10, 0x40, 0x80, 0xFF, 0x100, 0x200, 
                         0x400, 0x800, 0x1000, 0x4000, 0x8000, 0xFFFE, 0xFFFF]
        
        for blk in distant_blocks:
            self.reset_to_idle()
            # Write a recognizable pattern
            pattern = struct.pack("<HH", blk, blk ^ 0xFFFF) * 64  # 256B
            ok, err = self.dfu_dnload(pattern, block=blk)
            
            # Try to get status
            st = self.dfu_get_status()
            self.wait_poll(50)
            st2 = self.dfu_get_status()
            final = st2 or st
            
            state_name = final["state"] if final else "None"
            status_name = final["status"] if final else "None"
            
            alive = (final is not None)
            self.log(f"  Block 0x{blk:04X} ({blk*256:>10,}B offset): ok={ok} → {state_name} ({status_name}) alive={alive}")
            
            if not alive:
                self.finding(f"Device unreachable after block 0x{blk:04X}!",
                           f"Block {blk} (offset {blk*256} bytes) may have caused crash",
                           "critical")
                # Try to reconnect
                time.sleep(1)
                try:
                    self.dev.reset()
                    time.sleep(1)
                    self.connect()
                except:
                    self.log("  Cannot reconnect — device likely rebooted!", "ERR")
                    return

    # ============================================================
    # TEST 5: UPLOAD After DNLOAD (Data Leak Check)
    # ============================================================
    def test_upload_leak(self):
        self.log("=" * 60)
        self.log("TEST 5: UPLOAD After Proper DNLOAD Flow (Data Leak)")
        self.log("=" * 60)

        if not self.reset_to_idle():
            return

        # DNLOAD a known marker, then try to UPLOAD it back
        marker = b"\xDE\xAD\xBE\xEF" * 64  # 256B
        ok, err = self.dfu_dnload(marker, block=0)
        st = self.dfu_get_status()
        self.wait_poll(50)
        st2 = self.dfu_get_status()
        self.log(f"  After DNLOAD marker: {self.fmt_st(st2 or st)}")

        # Try UPLOAD from different states
        for blk in [0, 1, 2]:
            data, err = self.dfu_upload(256, block=blk)
            if data:
                nonzero = any(b != 0 for b in data)
                has_marker = data[:4] == b"\xDE\xAD\xBE\xEF"
                self.log(f"  UPLOAD block={blk}: {len(data)}B nonzero={nonzero} marker={has_marker}")
                if has_marker:
                    self.finding("UPLOAD reflects DNLOAD buffer!",
                               f"Block {blk}: our marker DEADBEEF visible → buffer can be read back", "critical")
                elif nonzero:
                    self.finding(f"UPLOAD returned non-zero data (block {blk})",
                               f"Data: {data[:32].hex()}", "high")
            else:
                self.log(f"  UPLOAD block={blk}: empty (err={err})")

        # Try from dfuIDLE (after abort)
        self.reset_to_idle()
        for blk in [0, 1]:
            data, err = self.dfu_upload(4096, block=blk)
            if data and any(b != 0 for b in data):
                self.finding(f"UPLOAD from dfuIDLE returned data (block {blk})",
                           f"Possible memory leak: {data[:32].hex()}", "critical")
            else:
                self.log(f"  UPLOAD from idle block={blk}: empty")

    # ============================================================
    # TEST 6: Vendor Request Scan
    # ============================================================
    def test_vendor_requests(self):
        self.log("=" * 60)
        self.log("TEST 6: Apple Vendor USB Request Scan")
        self.log("=" * 60)

        # Scan vendor IN requests (most likely to return data)
        vendor_in_types = [0xC0, 0xC1, 0xC2]  # Vendor: Device, Interface, Endpoint
        
        for bmRT in vendor_in_types:
            self.log(f"  Scanning bmRT=0x{bmRT:02X}...")
            for bReq in range(256):
                data, err = self.ctrl(bmRT, bReq, 0, 0, 256, timeout=100)
                if data and len(data) > 0:
                    self.log(f"    [HIT] bReq=0x{bReq:02X}: {len(data)}B → {data[:16].hex()}", "FIND")
                    self.finding(f"Vendor IN 0x{bmRT:02X}/0x{bReq:02X} returned data",
                               f"{len(data)}B: {data[:32].hex()}", "critical")
                elif err == "TIMEOUT":
                    self.log(f"    [TIMEOUT] bReq=0x{bReq:02X}", "WARN")
                    self.finding(f"Vendor IN 0x{bmRT:02X}/0x{bReq:02X} timeout",
                               "Device processing without STALL", "medium")
                # STALL = normal, skip silently

        # Scan vendor OUT with small payload
        self.log(f"  Scanning vendor OUT (0x40)...")
        for bReq in range(256):
            _, err = self.ctrl(0x40, bReq, 0, 0, b"\x00" * 4, timeout=100)
            if err is None:
                self.log(f"    [HIT] OUT bReq=0x{bReq:02X}: accepted!", "FIND")
                self.finding(f"Vendor OUT 0x40/0x{bReq:02X} accepted",
                           "Device accepted vendor OUT request", "high")
            elif err == "TIMEOUT":
                self.log(f"    [TIMEOUT] OUT bReq=0x{bReq:02X}", "WARN")

    # ============================================================
    # RUN ALL
    # ============================================================
    def run(self, tests=None):
        self.log("=" * 60)
        self.log("A12 (T8020) OVERFLOW PROBE")  
        self.log(f"Date: {datetime.now().isoformat()}")
        self.log("=" * 60)

        if not self.connect():
            return

        all_tests = {
            "1": ("State Trace", self.test_state_trace),
            "2": ("Multi-Block Overflow", self.test_multiblock_overflow),
            "3": ("Rapid Burst", self.test_rapid_burst),
            "4": ("Block Offset", self.test_block_offset),
            "5": ("Upload Leak", self.test_upload_leak),
            "6": ("Vendor Scan", self.test_vendor_requests),
        }
        to_run = tests or list(all_tests.keys())

        for t in to_run:
            if t in all_tests:
                name, func = all_tests[t]
                self.log(f"\n{'='*60}")
                self.log(f">>> TEST {t}: {name}")
                try:
                    func()
                except Exception as e:
                    self.log(f"Test {t} exception: {e}", "ERR")
                    traceback.print_exc()
                # Always try to recover
                try:
                    self.reset_to_idle()
                except:
                    self.log("Device recovery failed", "ERR")

        # SUMMARY
        self.log(f"\n{'='*60}")
        self.log("SUMMARY")
        self.log(f"{'='*60}")
        crit = [f for f in self.findings if f["severity"] == "critical"]
        high = [f for f in self.findings if f["severity"] == "high"]
        med  = [f for f in self.findings if f["severity"] == "medium"]
        self.log(f"Total: {len(self.findings)} findings — {len(crit)} CRITICAL, {len(high)} HIGH, {len(med)} MEDIUM")
        for f in crit:
            self.log(f"  [CRITICAL] {f['title']}: {f['detail']}")
        for f in high:
            self.log(f"  [HIGH] {f['title']}: {f['detail']}")

        # Save
        out = {"findings": self.findings, "device": self.serial,
               "timestamp": datetime.now().isoformat()}
        json_path = RESULTS_DIR / "a12_overflow_probe.json"
        with open(json_path, "w") as fh:
            json.dump(out, fh, indent=2)
        log_path = RESULTS_DIR / "a12_overflow_log.txt"
        with open(log_path, "w", encoding="utf-8") as fh:
            fh.write("\n".join(self.lines))
        self.log(f"Saved: {json_path}")
        self.log(f"Saved: {log_path}")

def main():
    import argparse
    p = argparse.ArgumentParser(description="A12 DFU Overflow Probe")
    p.add_argument("--tests", type=str, default=None,
                  help="Comma-separated test numbers (1-6). Default: all")
    p.add_argument("--quick", action="store_true",
                  help="Run tests 1,2,3,5 only (skip vendor scan + distant block)")
    args = p.parse_args()

    probe = A12OverflowProbe()
    if args.quick:
        tests = ["1", "2", "3", "5"]
    elif args.tests:
        tests = [t.strip() for t in args.tests.split(",")]
    else:
        tests = None
    probe.run(tests)

if __name__ == "__main__":
    main()
