#!/usr/bin/env python3
"""Clear DFU state and re-send iBSS properly."""
import usb.core, usb.util, usb.backend.libusb1
import time, sys

try:
    import libusb_package
    be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
except:
    be = None

def find_apple(pid):
    return usb.core.find(idVendor=0x05AC, idProduct=pid, backend=be)

dev = find_apple(0x1227)
if not dev:
    print("No DFU device")
    sys.exit(1)

print(f"DFU device found")

# Step 1: Clear the MANIFEST-WAIT-RESET state
print("\n1. Clearing DFU state...")
try:
    dev.set_configuration()
except:
    pass
try:
    usb.util.claim_interface(dev, 0)
except:
    pass

# Check current state
try:
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
    print(f"   Current state: {st[4]}")
except:
    pass

# DFU_CLRSTATUS to move out of error/manifest state
try:
    dev.ctrl_transfer(0x21, 4, 0, 0, timeout=1000)
    print("   CLRSTATUS sent")
except Exception as e:
    print(f"   CLRSTATUS: {e}")

time.sleep(0.5)

# Check state again
try:
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
    print(f"   State after CLRSTATUS: {st[4]}")
except Exception as e:
    print(f"   Status: {e}")
    # Device might have reset - try to reconnect
    time.sleep(2)
    dev = find_apple(0x1227)
    if dev:
        try:
            dev.set_configuration()
        except:
            pass
        try:
            usb.util.claim_interface(dev, 0)
        except:
            pass
        try:
            st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
            print(f"   State after reconnect: {st[4]}")
        except Exception as e:
            print(f"   Still can't read status: {e}")

# Check if we're in Recovery already
rec = find_apple(0x1281)
if rec:
    print("\n*** Already in Recovery mode! ***")
    sys.exit(0)

# If still in DFU, check state
dev = find_apple(0x1227)
if not dev:
    print("Device disconnected, waiting...")
    time.sleep(5)
    rec = find_apple(0x1281)
    if rec:
        print("\n*** In Recovery mode! ***")
        sys.exit(0)
    dev = find_apple(0x1227)
    if not dev:
        print("Device gone")
        sys.exit(1)

print(f"\n2. Device still in DFU, re-sending iBSS with manual protocol...")

try:
    dev.set_configuration()
except:
    pass
try:
    usb.util.claim_interface(dev, 0)
except:
    pass

# Make sure we're in dfuIDLE (state 2)
try:
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
    state = st[4]
    print(f"   State: {state}")
    if state != 2:
        # Try ABORT first, then CLRSTATUS
        try:
            dev.ctrl_transfer(0x21, 6, 0, 0, timeout=1000)
        except:
            pass
        try:
            dev.ctrl_transfer(0x21, 4, 0, 0, timeout=1000)
        except:
            pass
        time.sleep(0.5)
        try:
            st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
            print(f"   State after ABORT+CLRSTATUS: {st[4]}")
        except:
            pass
except:
    pass

# Now send iBSS via standard DFU DNLOAD protocol
with open("tools/checkm8-dualboot/cache/ibss_signed.img4", "rb") as f:
    data = f.read()
print(f"   iBSS: {len(data)} bytes")

# DFU DNLOAD: bmRequestType=0x21, bRequest=1 (DNLOAD), wValue=blocknum
BLOCK = 2048  # Standard DFU block size
offset = 0
blocknum = 0

while offset < len(data):
    chunk = data[offset:offset + BLOCK]
    try:
        dev.ctrl_transfer(0x21, 1, blocknum, 0, chunk, timeout=5000)
        offset += len(chunk)
        blocknum += 1
        if blocknum % 100 == 0:
            pct = 100 * offset // len(data)
            print(f"   {offset}/{len(data)} ({pct}%)")
    except usb.core.USBError as e:
        print(f"   Error at block {blocknum}, offset {offset}: {e}")
        break

print(f"   Sent {offset}/{len(data)} bytes in {blocknum} blocks")

# Send zero-length DNLOAD to signal end
try:
    dev.ctrl_transfer(0x21, 1, blocknum, 0, b"", timeout=5000)
    print("   Zero-length end marker sent")
except Exception as e:
    print(f"   End marker: {e}")

# Check status — should be MANIFEST or MANIFEST-WAIT-RESET
time.sleep(0.5)
try:
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
    print(f"   State after send: {st[4]}")
except:
    print("   Device disconnected (executing!)")

# Now trigger the USB reset
print("\n3. Triggering USB reset for iBSS execution...")
try:
    dev.reset()
    print("   USB reset sent")
except usb.core.USBError:
    print("   USB reset triggered (device disconnected)")
except Exception as e:
    print(f"   Reset: {e}")

# Wait for Recovery mode
print("\n4. Waiting for Recovery mode...")
for i in range(30):
    time.sleep(1)
    rec = find_apple(0x1281)
    if rec:
        print(f"\n*** RECOVERY MODE FOUND after {i+1}s! ***")
        try:
            print(f"   Product: {rec.product}")
            s1 = usb.util.get_string(rec, 1)
            if s1:
                print(f"   NONC: {s1[:80]}")
        except:
            pass
        sys.exit(0)
    
    dfu = find_apple(0x1227)
    if dfu and i % 5 == 4:
        print(f"   Still in DFU after {i+1}s")
        # Try reset again
        try:
            dfu.reset()
        except:
            pass

print("\nTimeout - device did not enter Recovery")
