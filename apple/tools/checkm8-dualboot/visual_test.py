#!/usr/bin/env python3
"""Visual test: send bgcolor commands and check if screen changes."""
import usb.core, usb.util, usb.backend.libusb1
import time

try:
    import libusb_package
    be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
except:
    be = None

dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
if not dev:
    dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
    if dev:
        print(f"Found Recovery 0x1281")
    else:
        print("No device")
        exit(1)
else:
    print("Found DFU 0x1227")

try:
    dev.set_configuration()
except:
    pass
try:
    usb.util.claim_interface(dev, 0)
except:
    pass

# Check DFU state
try:
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
    print(f"DFU state: {st[4]}, status: {st[0]}")
except Exception as e:
    print(f"DFU status: {e} (not in DFU?)")

# Send RED bgcolor
print("\n>>> Sending bgcolor RED (255,0,0)...")
print(">>> LOOK AT DEVICE SCREEN NOW!")
try:
    dev.ctrl_transfer(0x40, 0, 0, 0, b"bgcolor 255 0 0\x00", timeout=5000)
    print("    Sent OK")
except Exception as e:
    print(f"    Error: {e}")

time.sleep(4)

# Send GREEN bgcolor
print("\n>>> Sending bgcolor GREEN (0,255,0)...")
print(">>> LOOK AT DEVICE SCREEN NOW!")
try:
    dev.ctrl_transfer(0x40, 0, 0, 0, b"bgcolor 0 255 0\x00", timeout=5000)
    print("    Sent OK")
except Exception as e:
    print(f"    Error: {e}")

time.sleep(4)

# Send BLUE bgcolor
print("\n>>> Sending bgcolor BLUE (0,0,255)...")
print(">>> LOOK AT DEVICE SCREEN NOW!")
try:
    dev.ctrl_transfer(0x40, 0, 0, 0, b"bgcolor 0 0 255\x00", timeout=5000)
    print("    Sent OK")
except Exception as e:
    print(f"    Error: {e}")

print("\nDid the screen change to RED, then GREEN, then BLUE?")
print("If YES: we are in iBSS/iBEC (commands work!)")
print("If NO (stayed black): still in DFU SecureROM (need to fix boot)")
