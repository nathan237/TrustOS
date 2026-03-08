#!/usr/bin/env python3
"""
Rapid-fire DNLOAD/ABORT without any pause or status check between cycles.
This is to reproduce the crash seen earlier.
"""
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
    print("Not stable after 60s"); exit()

time.sleep(1)
d = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
d.set_configuration()
r = d.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
print(f"Starting state: {r[4]}")

# ============================================================
# Test A: Pure rapid-fire DNLOAD/ABORT - NO pause, NO status
# ============================================================
print("\n=== TEST A: Rapid-fire DNLOAD/ABORT (no pause, no status) ===")
payload = b"\x41" * 2048
t_start = time.perf_counter()

for i in range(50):
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
        d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=200)
    except usb.core.USBTimeoutError:
        print(f"  Cycle {i}: TIMEOUT ({(time.perf_counter()-t_start)*1000:.0f}ms)")
    except usb.core.USBError as e:
        dt = (time.perf_counter()-t_start)*1000
        print(f"  Cycle {i}: ERROR: {e} ({dt:.0f}ms)")
        time.sleep(0.5)
        d2 = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
        if not d2:
            print(f">>> CRASH at cycle {i}! Total time: {dt:.0f}ms")
            exit()
        else:
            print(f"  Device still alive, re-connecting")
            d = d2
            try: d.set_configuration()
            except: pass

dt = (time.perf_counter()-t_start)*1000
print(f"Test A: 50 cycles done, no crash ({dt:.0f}ms)")

# Check health
time.sleep(0.5)
d2 = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
if not d2:
    print("Device gone after test A!")
    exit()
d = d2
try: d.set_configuration()
except: pass

# ============================================================
# Test B: DNLOAD/ABORT with very short timeout on ABORT
# ============================================================
print("\n=== TEST B: DNLOAD/ABORT with 50ms ABORT timeout ===")
# Reset to idle first
for _ in range(5):
    try:
        r = d.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        if r[4] == 2: break
        if r[4] == 10: d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
        else: d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
    except: break
    time.sleep(0.05)

t_start = time.perf_counter()
for i in range(50):
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=500)
        d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=50)
    except usb.core.USBTimeoutError:
        print(f"  Cycle {i}: TIMEOUT")
    except usb.core.USBError as e:
        dt = (time.perf_counter()-t_start)*1000
        print(f"  Cycle {i}: {e} ({dt:.0f}ms)")
        time.sleep(0.5)
        d2 = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
        if not d2:
            print(f">>> CRASH at cycle {i}!")
            exit()
        d = d2
        try: d.set_configuration()
        except: pass

dt = (time.perf_counter()-t_start)*1000
print(f"Test B: 50 cycles done ({dt:.0f}ms)")

# ============================================================
# Test C: Vendor request 0x00 between DNLOAD/ABORT cycles
# (This mimics the original crash scenario)
# ============================================================
time.sleep(0.5)
d2 = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
if not d2:
    print("Device gone before test C!"); exit()
d = d2
try: d.set_configuration()
except: pass
for _ in range(5):
    try:
        r = d.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        if r[4] == 2: break
        if r[4] == 10: d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
        else: d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
    except: break
    time.sleep(0.05)

print("\n=== TEST C: Vendor req 0x00 OUT then DNLOAD/ABORT ===")
# Send some vendor requests first (like the original probe did)
for breq in [0x00, 0x07, 0x08, 0x09, 0x0A]:
    try:
        d.ctrl_transfer(0x21, breq, 0, 0, b"\x00" * 8, timeout=500)
    except: pass

t_start = time.perf_counter()
for i in range(10):
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
        d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=200)
    except usb.core.USBError as e:
        dt = (time.perf_counter()-t_start)*1000
        print(f"  Cycle {i}: {e} ({dt:.0f}ms)")
        time.sleep(0.5)
        d2 = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
        if not d2:
            print(f">>> CRASH at cycle {i} after vendor priming!")
            exit()
        d = d2
        try: d.set_configuration()
        except: pass

dt = (time.perf_counter()-t_start)*1000
print(f"Test C: 10 cycles done ({dt:.0f}ms)")

# ============================================================
# Test D: Many vendor requests then DNLOAD/ABORT
# ============================================================
time.sleep(0.5)
d2 = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
if not d2:
    print("Device gone before test D!"); exit()
d = d2
try: d.set_configuration()
except: pass

print("\n=== TEST D: Heavy vendor spray then DNLOAD/ABORT ===")
# Send many vendor requests with data (like the full probe)
for breq in [0x00, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x40, 0x41, 0x42, 0xA0, 0xA1, 0xFF]:
    for psize in [0, 8, 64, 256]:
        try:
            payload_v = bytes([0x41]) * psize if psize > 0 else b""
            d.ctrl_transfer(0x21, breq, 0, 0, payload_v, timeout=500)
        except: pass

print("  Vendor spray done, now DNLOAD/ABORT...")
t_start = time.perf_counter()
for i in range(10):
    try:
        d.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=2000)
        d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=200)
    except usb.core.USBError as e:
        dt = (time.perf_counter()-t_start)*1000
        print(f"  Cycle {i}: {e} ({dt:.0f}ms)")
        time.sleep(0.5)
        d2 = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
        if not d2:
            print(f">>> CRASH at cycle {i} after heavy vendor spray!")
            exit()
        d = d2
        try: d.set_configuration()
        except: pass

dt = (time.perf_counter()-t_start)*1000
print(f"Test D: 10 cycles done ({dt:.0f}ms)")

# Final
time.sleep(0.5)
d2 = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
print(f"\nFinal: device alive = {d2 is not None}")
