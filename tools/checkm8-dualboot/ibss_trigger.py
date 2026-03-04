#!/usr/bin/env python3
"""
Send iBSS (ticket still valid) and try all boot triggers.
Nonce unchanged: 926daf47...
"""
import usb.core, usb.util, usb.backend.libusb1, libusb_package
import time, os, math, binascii, struct, sys, subprocess

be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
CACHE = "tools/checkm8-dualboot/cache"

def find_dev(pid):
    return usb.core.find(idVendor=0x05AC, idProduct=pid, backend=be)

def check_recovery():
    r = find_dev(0x1281)
    if r:
        try:
            sn = r.serial_number[:60]
            print(f"\n*** RECOVERY FOUND! {sn} ***")
        except:
            print("\n*** RECOVERY FOUND! ***")
        return True
    return False

# ===== Send iBSS =====
print("Loading cached ibss_signed.img4...")
with open(os.path.join(CACHE, "ibss_signed.img4"), "rb") as f:
    data = f.read()
print(f"  {len(data)} bytes")

dev = find_dev(0x1227)
if not dev:
    print("No DFU device!"); sys.exit(1)

dev.set_configuration()
usb.util.claim_interface(dev, 0)

# Check state
st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
state = st[4]
print(f"DFU state: {state}")

if state != 2:
    # Clear to idle
    for _ in range(3):
        try: dev.ctrl_transfer(0x21, 4, 0, 0, timeout=500)
        except: pass
    time.sleep(0.5)
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
    state = st[4]
    print(f"  After CLRSTATUS: {state}")

if state != 2:
    print("  Can't get to dfuIDLE, aborting")
    sys.exit(1)

# Send data
PACKET_SIZE = 2048
num_packets = math.ceil(len(data) / PACKET_SIZE)
crc = -1
t0 = time.time()

for offset in range(0, len(data), PACKET_SIZE):
    chunk = data[offset:offset + PACKET_SIZE]
    idx = offset // PACKET_SIZE
    
    if offset + PACKET_SIZE >= len(data):
        crc = binascii.crc32(data, crc)
        dfu_xbuf = bytearray([0xFF,0xFF,0xFF,0xFF,0xAC,0x05,0x00,0x01,0x55,0x46,0x44,0x10])
        crc = binascii.crc32(dfu_xbuf, crc)
        crc_chunk = dfu_xbuf + struct.pack("<I", crc)
        if len(chunk) + 16 > PACKET_SIZE:
            dev.ctrl_transfer(0x21, 1, idx, 0, chunk, timeout=5000)
            dev.ctrl_transfer(0x21, 1, idx, 0, crc_chunk, timeout=5000)
        else:
            dev.ctrl_transfer(0x21, 1, idx, 0, chunk + crc_chunk, timeout=5000)
    else:
        dev.ctrl_transfer(0x21, 1, idx, 0, chunk, timeout=5000)

dt = time.time() - t0
print(f"Sent {num_packets} packets in {dt:.1f}s")

# Wait for status 5
for _ in range(20):
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
    if st[4] == 5:
        print(f"State: 5 (DNLOAD-IDLE) OK")
        break
    time.sleep(0.3)

# Finalize
dev.ctrl_transfer(0x21, 1, num_packets, 0, b"", timeout=5000)
for _ in range(2):
    try:
        st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        print(f"State: {st[4]}")
    except:
        print("(disconnected during status read)")
        time.sleep(3)
        if check_recovery(): sys.exit(0)
        break

# ===== Now try triggers =====
print("\n=== TRIGGER STRATEGIES ===")

# Strategy 1: DFU_DETACH (bRequest=0, wValue=timeout_ms)
# Apple SecureROM may honor DFU_DETACH to leave DFU mode
print("\n[1] DFU_DETACH (0x21, 0, wValue=1000)...")
try:
    dev.ctrl_transfer(0x21, 0, 1000, 0, timeout=2000)
    print("    Sent OK")
except usb.core.USBError as e:
    print(f"    {e}")
time.sleep(3)
if check_recovery(): sys.exit(0)

# Re-find device
dev = find_dev(0x1227)
if not dev:
    time.sleep(2)
    if check_recovery(): sys.exit(0)
    print("Device lost and no Recovery!")
    sys.exit(1)
try:
    dev.set_configuration()
    usb.util.claim_interface(dev, 0)
except: pass

# Strategy 2: USB reset
print("\n[2] libusb reset...")
try:
    dev.reset()
    print("    Reset done")
except Exception as e:
    print(f"    {e}")
time.sleep(5)
if check_recovery(): sys.exit(0)

# Re-find
dev = find_dev(0x1227)
if dev:
    try:
        dev.set_configuration()
        usb.util.claim_interface(dev, 0)
        st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
        print(f"    DFU state after reset: {st[4]}")
    except: pass
else:
    time.sleep(3)
    if check_recovery(): sys.exit(0)

# Strategy 3: Release interface + dispose
print("\n[3] Release + dispose...")
try:
    usb.util.release_interface(dev, 0)
    usb.util.dispose_resources(dev)
    print("    Released + disposed")
except: pass
time.sleep(3)
if check_recovery(): sys.exit(0)

# Strategy 4: Re-send just the CRC/finalization
print("\n[4] Re-finalize (empty DNLOAD again)...")
dev = find_dev(0x1227)
if dev:
    try:
        dev.set_configuration()
        usb.util.claim_interface(dev, 0)
        dev.ctrl_transfer(0x21, 1, 0, 0, b"", timeout=5000)
        print("    Sent empty DNLOAD")
        time.sleep(1)
        st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        print(f"    State: {st[4]}")
        dev.reset()
        print("    Reset after re-finalize")
    except Exception as e:
        print(f"    {e}")
    time.sleep(3)
    if check_recovery(): sys.exit(0)

# Strategy 5: ABORT from state 8
print("\n[5] DFU ABORT from current state...")
dev = find_dev(0x1227)
if dev:
    try:
        dev.set_configuration()
        usb.util.claim_interface(dev, 0)
        st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
        print(f"    State before abort: {st[4]}")
        dev.ctrl_transfer(0x21, 6, 0, 0, timeout=1000)
        print("    ABORT sent")
    except usb.core.USBError as e:
        if "No such device" in str(e):
            print("    Device disconnected after ABORT!")
            time.sleep(5)
            if check_recovery(): sys.exit(0)
        else:
            print(f"    {e}")
    time.sleep(3)
    if check_recovery(): sys.exit(0)

# Strategy 6: Double reset
print("\n[6] Double reset...")
dev = find_dev(0x1227)
if dev:
    for i in range(3):
        try:
            dev.reset()
            print(f"    Reset {i+1} done")
        except:
            print(f"    Reset {i+1} error")
        time.sleep(1)
    time.sleep(3)
    if check_recovery(): sys.exit(0)

print("\n=== ALL STRATEGIES FAILED ===")
print("The device stays in DFU after all reset attempts.")
print("\nThis is a Windows USB stack limitation.")
print("The libusb_reset_device() doesn't generate the port reset")
print("that SecureROM needs to trigger iBSS execution.")
print("\nNEXT STEPS:")
print("  1. Try a USB 2.0 hub (not USB 3.0)")
print("  2. Try WSL2 with USB passthrough (usbipd)")
print("  3. Use idevicerestore on a Mac/Linux machine")
print("  4. Or proceed to TrustOS bare-metal xHCI approach")
