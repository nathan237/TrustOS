#!/usr/bin/env python3
"""
Crash Boundary Fuzzer — Find exact size threshold for SecureROM crash
======================================================================

KNOWN: Payloads ≤10B → 73ms crash (device dies)
KNOWN: Payloads at 512B → 3562ms normal manifest (survives)
KNOWN: 2048→2049B sharp DNLOAD timing boundary

This script binary-searches between 10B and 512B to find the EXACT byte
threshold where the SecureROM's manifest parser stops crashing.

CAREFUL: Each crash requires device recovery (force restart + re-enter DFU).
The script pauses between crash tests for manual device recovery.

Strategy: Start from the safe side (512B, known to survive), decrease
until we find the crash point. This minimizes device deaths.
"""

import sys, os, time, json
from datetime import datetime
from pathlib import Path

import usb.core, usb.util

try:
    import libusb_package, usb.backend.libusb1
    USB_BACKEND = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
except:
    USB_BACKEND = None

APPLE_VID = 0x05AC
DFU_PID = 0x1227

DFU_DNLOAD    = 1
DFU_GETSTATUS = 3
DFU_CLRSTATUS = 4
DFU_GETSTATE  = 5
DFU_ABORT     = 6

RESULTS_DIR = Path(__file__).parent.resolve() / "results"
RESULTS_DIR.mkdir(exist_ok=True)

log_lines = []
results = []

def log(msg, level="INFO"):
    ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    line = f"[{ts}] [{level:4s}] {msg}"
    print(line)
    log_lines.append(line)

def find_dfu():
    dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=USB_BACKEND)
    if dev:
        try: dev.set_configuration()
        except: pass
    return dev

def get_status(dev):
    try:
        data = dev.ctrl_transfer(0xA1, DFU_GETSTATUS, 0, 0, 6, timeout=2000)
        return {"bStatus": data[0], "bState": data[4],
                "poll_ms": data[1]|(data[2]<<8)|(data[3]<<16)}
    except:
        return None

def alive(dev):
    try:
        dev.ctrl_transfer(0xA1, DFU_GETSTATE, 0, 0, 1, timeout=500)
        return True
    except:
        return False

def reset_to_idle(dev):
    for _ in range(20):
        st = get_status(dev)
        if st is None: return False
        if st["bState"] == 2: return True
        if st["bState"] == 10:
            dev.ctrl_transfer(0x21, DFU_CLRSTATUS, 0, 0, b'', timeout=1000)
            time.sleep(0.05)
        elif st["bState"] in (5, 3):
            dev.ctrl_transfer(0x21, DFU_ABORT, 0, 0, b'', timeout=1000)
            time.sleep(0.05)
        elif st["bState"] == 8:
            try: dev.ctrl_transfer(0x21, DFU_ABORT, 0, 0, b'', timeout=1000)
            except: pass
            try: dev.reset()
            except: pass
            time.sleep(2)
            return False
        else:
            try: dev.ctrl_transfer(0x21, DFU_ABORT, 0, 0, b'', timeout=1000)
            except: pass
            time.sleep(0.1)
    return False

def reconnect_dfu(dev):
    """Full reconnect after manifest-wait-reset state."""
    try:
        dev.ctrl_transfer(0x21, DFU_ABORT, 0, 0, b'', timeout=2000)
    except: pass
    time.sleep(0.1)
    try:
        dev.reset()
    except: pass
    time.sleep(2)
    try:
        usb.util.dispose_resources(dev)
    except: pass
    
    for attempt in range(10):
        time.sleep(0.5)
        try:
            new_dev = find_dfu()
            if new_dev:
                st = get_status(new_dev)
                if st and st["bState"] == 2:
                    return new_dev
                if st and st["bState"] == 10:
                    new_dev.ctrl_transfer(0x21, DFU_CLRSTATUS, 0, 0, b'', timeout=1000)
                    time.sleep(0.1)
                    st2 = get_status(new_dev)
                    if st2 and st2["bState"] == 2:
                        return new_dev
                if st and st["bState"] == 8:
                    try: new_dev.ctrl_transfer(0x21, DFU_ABORT, 0, 0, b'', timeout=1000)
                    except: pass
                    try: new_dev.reset()
                    except: pass
                    time.sleep(2)
                    continue
        except:
            pass
    
    return None

def test_size(dev, size, content_byte=0x00):
    """
    Test: DNLOAD `size` bytes → zero-length DNLOAD → measure manifest behavior.
    
    Returns: (survived: bool, manifest_ms: float, final_state: int, new_dev or None)
    """
    payload = bytes([content_byte]) * size
    
    if not reset_to_idle(dev):
        # Try reconnect
        dev2 = reconnect_dfu(dev)
        if dev2 is None:
            return (None, 0, -1, None)
        dev = dev2
        if not reset_to_idle(dev):
            return (None, 0, -1, None)
    
    # DNLOAD payload
    try:
        dev.ctrl_transfer(0x21, DFU_DNLOAD, 0, 0, payload, timeout=5000)
    except Exception as e:
        log(f"  DNLOAD error: {e}", "ERR")
        return (None, 0, -1, dev)
    
    st = get_status(dev)
    if st is None:
        return (False, 0, -1, dev)
    
    # Wait poll
    if st["poll_ms"] > 0:
        time.sleep(st["poll_ms"] / 1000.0 + 0.01)
    
    # Wait for dnIDLE
    for _ in range(10):
        st = get_status(dev)
        if st and st["bState"] == 5: break
        if st and st["bState"] == 10:
            return (True, 0, 10, dev)  # Error but survived
        time.sleep(0.05)
    
    if st is None or st["bState"] == 10:
        return (True, 0, st["bState"] if st else -1, dev)
    
    # Zero-length DNLOAD to trigger manifest
    try:
        dev.ctrl_transfer(0x21, DFU_DNLOAD, 1, 0, b'', timeout=5000)
    except:
        return (False, 0, -1, dev)
    
    t0 = time.perf_counter()
    
    # GETSTATUS → should go to state 6 (MANIFEST-SYNC)
    st = get_status(dev)
    if st is None:
        elapsed = (time.perf_counter() - t0) * 1000
        return (False, elapsed, -1, dev)
    
    if st["bState"] == 6:  # MANIFEST-SYNC
        time.sleep(max(0.01, st["poll_ms"] / 1000.0) + 0.01)
        st = get_status(dev)
        if st is None:
            elapsed = (time.perf_counter() - t0) * 1000
            return (False, elapsed, -1, dev)
    
    if st and st["bState"] == 7:  # dfuMANIFEST
        manifest_poll = st["poll_ms"]
        time.sleep(max(0.5, manifest_poll / 1000.0) + 0.5)
        st = get_status(dev)
    
    elapsed = (time.perf_counter() - t0) * 1000
    
    if st is None:
        # Device gone after manifest — CRASH
        return (False, elapsed, -1, None)
    
    final_state = st["bState"]
    
    if final_state == 8:  # MANIFEST-WAIT-RESET (normal completion)
        # Need to reconnect to dfuIDLE
        new_dev = reconnect_dfu(dev)
        return (True, elapsed, 8, new_dev)
    
    if final_state == 10:  # ERROR
        return (True, elapsed, 10, dev)
    
    return (True, elapsed, final_state, dev)

def main():
    log("=" * 70)
    log("CRASH BOUNDARY FUZZER — SecureROM Small Payload Crash")
    log("=" * 70)
    log("")
    log("Known: ≤10B → crash (73ms), 512B → survives (3562ms)")
    log("Goal: Find exact byte threshold between crash and survive")
    log("")
    
    dev = find_dfu()
    if not dev:
        log("No DFU device!", "ERR")
        sys.exit(1)
    
    serial = usb.util.get_string(dev, dev.iSerialNumber)
    log(f"Device: {serial}")
    
    # Strategy: test from safe sizes DOWN toward crash territory
    # This way we minimize crashes
    
    # Phase 1: Quick scan with big steps (512, 384, 256, 192, 128, 96, 64, 48, 32, 16, 8)
    test_sizes = [512, 384, 256, 192, 128, 96, 64, 48, 32, 24, 16, 12, 10, 8, 4, 2]
    
    log(f"\n--- Phase 1: Quick scan ({len(test_sizes)} sizes) ---")
    log(f"{'Size':>6s} | {'Survived':>8s} | {'Time (ms)':>10s} | {'State':>5s}")
    log("-" * 50)
    
    last_survive_size = None
    first_crash_size = None
    
    for size in test_sizes:
        if dev is None:
            log("\nDevice lost! Waiting for recovery...", "WARN")
            log(">>> Please force-restart iPhone (Vol Up tap, Vol Down tap, Side hold)")
            log(">>> Then re-enter DFU mode")
            input(">>> Press ENTER when device is back in DFU: ")
            dev = find_dfu()
            if dev is None:
                log("Still no device!", "ERR")
                break
        
        survived, elapsed, final_state, dev = test_size(dev, size)
        
        result = {
            "size": size,
            "survived": survived,
            "elapsed_ms": round(elapsed, 1),
            "final_state": final_state,
            "timestamp": datetime.now().isoformat()
        }
        results.append(result)
        
        tag = " OK " if survived else "DEAD"
        if survived is None:
            tag = " ?? "
        
        log(f"  {size:5d} | [{tag}]  | {elapsed:10.1f} | {final_state:5d}")
        
        if survived:
            last_survive_size = size
        elif survived is False:
            first_crash_size = size
            log(f"\n  *** CRASH at {size}B! ***")
            
            if last_survive_size and last_survive_size - size > 2:
                # There's a gap — we need phase 2 binary search
                log(f"  Boundary is between {size}B (crash) and {last_survive_size}B (survive)")
                break
            elif last_survive_size:
                log(f"  Exact boundary: {size}B crashes, {last_survive_size}B survives")
                break
    
    # Phase 2: Binary search if needed
    if first_crash_size and last_survive_size and last_survive_size - first_crash_size > 2:
        lo = first_crash_size  # crashes
        hi = last_survive_size  # survives
        
        log(f"\n--- Phase 2: Binary search [{lo}B..{hi}B] ---")
        
        while hi - lo > 1:
            mid = (lo + hi) // 2
            
            if dev is None:
                log(f"\nDevice lost! Need recovery for test at {mid}B...", "WARN")
                log(">>> Force-restart iPhone, re-enter DFU")
                input(">>> Press ENTER when ready: ")
                dev = find_dfu()
                if dev is None:
                    log("Still no device!", "ERR")
                    break
            
            survived, elapsed, final_state, dev = test_size(dev, mid)
            
            result = {
                "size": mid,
                "survived": survived,
                "elapsed_ms": round(elapsed, 1),
                "final_state": final_state,
                "phase": "binary_search",
                "timestamp": datetime.now().isoformat()
            }
            results.append(result)
            
            tag = " OK " if survived else "DEAD"
            log(f"  {mid:5d}B | [{tag}] | {elapsed:10.1f}ms | state={final_state}")
            
            if survived:
                hi = mid
            elif survived is False:
                lo = mid
                log(f"  *** CRASH at {mid}B! ***")
            else:
                log(f"  ??? Unknown result at {mid}B", "WARN")
                break
        
        log(f"\n  === BOUNDARY: {lo}B crashes, {hi}B survives ===")
    
    # Summary
    log(f"\n{'=' * 70}")
    log("BOUNDARY SEARCH SUMMARY")
    log(f"{'=' * 70}")
    
    survived_sizes = [r["size"] for r in results if r["survived"] == True]
    crashed_sizes = [r["size"] for r in results if r["survived"] == False]
    
    if survived_sizes:
        log(f"  Survived (smallest): {min(survived_sizes)}B")
    if crashed_sizes:
        log(f"  Crashed (largest):   {max(crashed_sizes)}B")
    
    if survived_sizes and crashed_sizes:
        boundary_lo = max(crashed_sizes)
        boundary_hi = min(survived_sizes)
        log(f"  BOUNDARY: between {boundary_lo}B and {boundary_hi}B")
        log(f"  (crash ≤{boundary_lo}B, survive ≥{boundary_hi}B)")
    
    log(f"\n  All results:")
    for r in sorted(results, key=lambda x: x["size"]):
        tag = "OK" if r["survived"] else "CRASH" if r["survived"] is False else "??"
        log(f"    {r['size']:5d}B: {tag:5s} | {r['elapsed_ms']:8.1f}ms | state={r['final_state']}")
    
    # Save
    out = {
        "results": results,
        "survived_sizes": survived_sizes,
        "crashed_sizes": crashed_sizes,
        "timestamp": datetime.now().isoformat()
    }
    json_path = RESULTS_DIR / "crash_boundary.json"
    with open(json_path, 'w') as f:
        json.dump(out, f, indent=2)
    log_path = RESULTS_DIR / "crash_boundary_log.txt"
    with open(log_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(log_lines))
    
    log(f"\nSaved: {json_path}")
    log(f"Saved: {log_path}")

if __name__ == "__main__":
    main()
