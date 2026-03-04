#!/usr/bin/env python3
"""Quick DNLOAD/ABORT cycle counter on fresh DFU"""
import usb.core, usb.util, libusb_package, usb.backend.libusb1, time

be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

# Wait for stable DFU
print("Waiting for stable DFU...")
stable = 0
for i in range(60):
    d = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
    if d:
        try:
            d.set_configuration()
            r = d.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
            if r[4] == 2:
                stable += 1
                if stable >= 3:
                    print(f"Stable DFU confirmed ({i+1}s)")
                    break
            else:
                stable = 0
        except:
            stable = 0
    else:
        stable = 0
    time.sleep(1)
else:
    print("Not stable after 60s")
    exit()

time.sleep(1)

# Fresh connect
d = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
d.set_configuration()

print("\n=== FRESH DFU: DNLOAD/ABORT cycles ===")
payload = b"\x41" * 2048

for i in range(10):
    t0 = time.perf_counter()
    
    # DNLOAD
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
    except Exception as e:
        dt = (time.perf_counter()-t0)*1000
        print(f"Cycle {i}: DNLOAD err: {e} ({dt:.1f}ms)")
        time.sleep(0.5)
        d2 = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
        if not d2:
            print(f">>> CRASH at cycle {i} (DNLOAD)")
            break
        d = d2
        try: d.set_configuration()
        except: pass
        continue
    
    # ABORT immediately (no GET_STATUS)
    try:
        d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
    except Exception as e:
        dt = (time.perf_counter()-t0)*1000
        print(f"Cycle {i}: ABORT err: {e} ({dt:.1f}ms)")
        time.sleep(0.5)
        d2 = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
        if not d2:
            print(f">>> CRASH at cycle {i} (ABORT)")
            break
        d = d2
        try: d.set_configuration()
        except: pass
        continue

    dt = (time.perf_counter()-t0)*1000
    
    # Status check
    time.sleep(0.1)
    try:
        r = d.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        state = r[4]
        print(f"Cycle {i}: OK state={state} ({dt:.1f}ms)")
        if state == 10:
            d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
    except:
        time.sleep(0.5)
        d2 = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
        if not d2:
            print(f">>> CRASH at cycle {i} (STATUS)")
            break
        d = d2
        try: d.set_configuration()
        except: pass
else:
    print("10 cycles - no crash")
