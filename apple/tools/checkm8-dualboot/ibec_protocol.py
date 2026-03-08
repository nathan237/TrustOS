#!/usr/bin/env python3
"""Try different protocols to talk to iBEC."""
import usb.core, usb.util, libusb_package, time

be = libusb_package.get_libusb1_backend()
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    print("No device")
    exit(1)

dev.set_configuration(1)
for i in [0, 1]:
    try:
        usb.util.claim_interface(dev, i)
    except:
        pass
try:
    dev.set_interface_altsetting(1, 1)
except:
    pass

print("=== Try 1: Read bulk IN banner ===")
try:
    data = dev.read(0x81, 4096, timeout=3000)
    print(f"Got {len(data)} bytes: {bytes(data)!r}")
except usb.core.USBError as e:
    print(f"Bulk read: {e}")

print("\n=== Try 2: Send via bulk OUT EP 0x02 ===")
cmd = b"getenv build-version\n"
try:
    n = dev.write(0x02, cmd, timeout=5000)
    print(f"Wrote {n} bytes")
    time.sleep(0.5)
    try:
        data = dev.read(0x81, 4096, timeout=3000)
        print(f"Response: {bytes(data)!r}")
    except usb.core.USBError as e:
        print(f"Read: {e}")
except usb.core.USBError as e:
    print(f"Write: {e}")

print("\n=== Try 3: Different ctrl bRequests ===")
for breq in range(5):
    try:
        dev.ctrl_transfer(0x40, breq, 0, 0, b"getenv build-version\x00", timeout=2000)
        try:
            r = dev.ctrl_transfer(0xC0, breq, 0, 0, 512, timeout=2000)
            text = bytes(r).rstrip(b"\x00").decode("utf-8", errors="replace")
            print(f"  bRequest={breq}: {text}")
        except usb.core.USBError:
            print(f"  bRequest={breq}: send OK, read STALL")
    except usb.core.USBError as e:
        print(f"  bRequest={breq}: PIPE ({e})")

print("\n=== Try 4: Send via bulk EP 0x04 (fw upload EP) ===")
cmd = b"getenv build-version\x00"
try:
    n = dev.write(0x04, cmd, timeout=5000)
    print(f"EP 0x04 wrote {n} bytes")
except usb.core.USBError as e:
    print(f"EP 0x04 write: {e}")

print("\n=== Try 5: DFU-style ctrl (0x21/0xA1) ===")
try:
    dev.ctrl_transfer(0x21, 1, 0, 0, b"getenv build-version\x00", timeout=2000)
    print("  DFU DNLOAD sent")
    try:
        r = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        print(f"  DFU status: {list(r)}")
    except usb.core.USBError as e:
        print(f"  DFU status: {e}")
except usb.core.USBError as e:
    print(f"  DFU DNLOAD: {e}")

print("\n=== Try 6: Check if it's still the SAME device (iBSS not iBEC?) ===")
# The build-version was readable before with send_cmd in iBSS mode
# Maybe this IS iBSS and not iBEC
# Check: does bgcolor work? (it worked in iBSS)
try:
    dev.ctrl_transfer(0x40, 0, 0, 0, b"bgcolor 255 0 0\x00", timeout=2000)
    print("  bgcolor sent (no error)")
    try:
        r = dev.ctrl_transfer(0xC0, 0, 0, 0, 512, timeout=2000)
        print(f"  bgcolor response: {bytes(r).rstrip(b'\x00').decode()}")
    except:
        print("  bgcolor response: STALL")
except usb.core.USBError as e:
    print(f"  bgcolor: PIPE ERROR - this is likely a new iBEC stage!")
    print("  (iBSS could do bgcolor, iBEC cannot via ctrl => DIFFERENT FIRMWARE)")
