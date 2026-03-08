#!/usr/bin/env python3
"""
A12 SecureROM — 2048B Boundary Overflow Analysis
===================================================
CONFIRMED: DFU buffer = exactly 2048 bytes.
  - 2048B = 1.1ms (normal, single USB transaction)
  - 2049B = 1005ms (1 second! different code path)

Questions to answer:
  1. What happens in manifest phase with 2049B overflow?
  2. Does the extra byte(s) overwrite heap metadata?
  3. Can we control what gets overwritten?
  4. Is the 1s delay a timeout or actual processing?
  5. What's the timing for manifest at exactly 2048B vs 2049B?
"""
import time, json, statistics, struct
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227

def be():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    print(f"[{ts}] {msg}", flush=True)

class OverflowAnalysis:
    def __init__(self):
        self.dev = None

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

    def full_dnload_manifest(self, payload, timeout_s=15):
        """
        Full DNLOAD + manifest cycle. Returns timing details.
        """
        if not self.to_idle():
            return {"error": "no_idle"}

        # DNLOAD
        t0 = time.perf_counter_ns()
        try:
            self.dev.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=15000)
        except usb.core.USBError as e:
            dt_ms = (time.perf_counter_ns() - t0) / 1e6
            return {"dnload_error": str(e), "dnload_ms": round(dt_ms, 3)}
        t_dnload = time.perf_counter_ns()
        dnload_ms = (t_dnload - t0) / 1e6

        # GET_STATUS (triggers processing)
        s = self.status()
        t_status1 = time.perf_counter_ns()
        status1_ms = (t_status1 - t0) / 1e6

        if not s:
            return {"error": "no_status_after_dnload", "dnload_ms": round(dnload_ms, 3)}

        state1 = s[0]
        poll1 = s[2]

        # If in DNLOAD-IDLE (state 5), send zero-length to trigger manifest
        if state1 == 5:
            try:
                self.dev.ctrl_transfer(0x21, 1, 0, 0, b"", timeout=5000)
            except usb.core.USBError as e:
                return {"manifest_trigger_error": str(e), "state1": state1}
            s = self.status()
            t_manifest_start = time.perf_counter_ns()
        else:
            t_manifest_start = t_status1

        manifest_state = s[0] if s else None
        manifest_poll = s[2] if s else 0

        # Wait for manifest completion
        states_seen = [manifest_state]
        polls_seen = [manifest_poll]

        for _ in range(200):
            if not s or s[0] not in (3, 4, 7):  # not in sync/busy/manifest states
                break
            if s[0] == 4:  # BUSY
                time.sleep(s[2] / 1000.0)
            else:
                time.sleep(0.01)
            s = self.status()
            if s:
                states_seen.append(s[0])
                polls_seen.append(s[2])

        t_done = time.perf_counter_ns()
        total_ms = (t_done - t0) / 1e6
        manifest_ms = (t_done - t_manifest_start) / 1e6
        final_state = s[0] if s else None

        return {
            "size": len(payload),
            "dnload_ms": round(dnload_ms, 3),
            "status1_ms": round(status1_ms, 3),
            "state_after_dnload": state1,
            "poll_after_dnload": poll1,
            "manifest_ms": round(manifest_ms, 3),
            "total_ms": round(total_ms, 3),
            "final_state": final_state,
            "states_seen": states_seen,
            "polls_seen": list(set(polls_seen)),
        }

    def run(self):
        log("="*60)
        log("A12 SecureROM — 2048B Boundary Overflow Analysis")
        log("="*60)

        if not self.connect():
            log("NO DFU DEVICE"); return

        results = {"timestamp": datetime.now().isoformat()}

        # ============================================================
        # Test 1: Full manifest timing at boundary sizes
        # ============================================================
        log("\n--- Test 1: Full manifest timing at boundary ---")
        boundary_tests = []

        for size in [2040, 2044, 2046, 2047, 2048, 2049, 2050, 2052, 2056, 2064]:
            payload = bytes([0x41]) * size
            log(f"  Testing {size}B...")
            r = self.full_dnload_manifest(payload)
            boundary_tests.append(r)
            
            if "error" in r or "dnload_error" in r:
                log(f"  {size}B: {r}")
            else:
                log(f"  {size}B: dnload={r['dnload_ms']:.1f}ms manifest={r['manifest_ms']:.1f}ms "
                    f"total={r['total_ms']:.1f}ms final={r['final_state']} "
                    f"states={r['states_seen']} polls={r['polls_seen']}")

        results["boundary_manifest"] = boundary_tests

        # ============================================================
        # Test 2: Overflow byte content matters?
        # ============================================================
        log("\n--- Test 2: Does overflow byte content affect timing? ---")
        overflow_tests = []

        for name, extra in [
            ("zero_1B",     b"\x00"),
            ("0x41_1B",     b"\x41"),
            ("0xFF_1B",     b"\xFF"),
            ("zero_4B",     b"\x00\x00\x00\x00"),
            ("ptr_lo_4B",   struct.pack("<I", 0x19C018000 & 0xFFFFFFFF)),
            ("ptr_hi_4B",   struct.pack("<I", 0x19C018000 >> 32)),
            ("zero_8B",     b"\x00" * 8),
            ("sram_ptr_8B", struct.pack("<Q", 0x19C018000)),
            ("null_ptr_8B", struct.pack("<Q", 0)),
            ("zero_16B",    b"\x00" * 16),
            ("zero_64B",    b"\x00" * 64),
            ("zero_128B",   b"\x00" * 128),
            ("zero_256B",   b"\x00" * 256),
        ]:
            payload = bytes([0x42]) * 2048 + extra
            r = self.full_dnload_manifest(payload)
            r["overflow_name"] = name
            r["overflow_size"] = len(extra)
            overflow_tests.append(r)

            if "error" in r or "dnload_error" in r:
                log(f"  {name:15s} (+{len(extra):3d}B): {r.get('error', r.get('dnload_error', '?'))}")
            else:
                log(f"  {name:15s} (+{len(extra):3d}B): dnload={r['dnload_ms']:.1f}ms "
                    f"manifest={r['manifest_ms']:.1f}ms total={r['total_ms']:.1f}ms "
                    f"final={r['final_state']} polls={r['polls_seen']}")

            # Check if device died
            if not self.alive():
                log(f"  >>> DEVICE DIED with overflow {name}!")
                overflow_tests[-1]["device_died"] = True
                break

        results["overflow_content"] = overflow_tests

        # ============================================================
        # Test 3: Compare manifest timing at exactly 2048B 
        # with different DER headers
        # ============================================================
        log("\n--- Test 3: 2048B manifest with various DER headers ---")
        header_tests = []

        headers_2048 = {
            "zeros":       bytes(2048),
            "0x41":        b"\x41" * 2048,
            "img4":        b"\x30\x82\x10\x00\x16\x04IMG4" + b"\x00" * 2038,
            "im4p":        b"\x30\x82\x10\x00\x16\x04IM4P" + b"\x00" * 2038,
            "ibss":        b"\x30\x82\x10\x00\x16\x04IMG4\x16\x04ibss" + b"\x00" * 2028,
            "illb":        b"\x30\x82\x10\x00\x16\x04IMG4\x16\x04illb" + b"\x00" * 2028,
            "der_big_len": b"\x30\x84\xFF\xFF\xFF\xFF" + b"\x00" * 2042,
            "nested_der":  b"\x30\x82\x07\xFC" + b"\x30\x82\x03\xF8" + b"\x00" * 2040,
        }

        for name, payload in headers_2048.items():
            r = self.full_dnload_manifest(payload)
            r["header_name"] = name
            header_tests.append(r)

            if "error" in r or "dnload_error" in r:
                log(f"  {name:15s}: {r.get('error', r.get('dnload_error', '?'))}")
            else:
                log(f"  {name:15s}: dnload={r['dnload_ms']:.1f}ms manifest={r['manifest_ms']:.1f}ms "
                    f"total={r['total_ms']:.1f}ms final={r['final_state']} polls={r['polls_seen']}")

        results["header_at_2048"] = header_tests

        # Save
        outf = Path(__file__).parent / "results" / "overflow_analysis.json"
        outf.parent.mkdir(exist_ok=True)
        with open(outf, "w") as f:
            json.dump(results, f, indent=2)
        log(f"\nSaved: {outf}")

if __name__ == "__main__":
    a = OverflowAnalysis()
    a.run()
