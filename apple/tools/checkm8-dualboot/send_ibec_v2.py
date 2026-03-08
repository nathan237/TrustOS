#!/usr/bin/env python3
"""Send iBEC to iBSS Recovery mode device."""
import plistlib, time, os
import usb.core, usb.util, libusb_package
from pymobiledevice3.restore.img4 import IMG4, IM4P
from pymobiledevice3.restore.tss import TSSResponse

CACHE = "tools/checkm8-dualboot/cache"

# Load iBEC
print("Loading iBEC...")
with open(os.path.join(CACHE, "iBEC.n841.RELEASE.im4p"), "rb") as f:
    ibec_data = f.read()
im4p = IM4P(data=ibec_data)
print(f"  fourcc={im4p.fourcc} desc={im4p.description} size={len(ibec_data)}")

# Load ticket
with open(os.path.join(CACHE, "tss_success_response.plist"), "rb") as f:
    tss = TSSResponse(plistlib.load(f))
ticket = tss.ap_img4_ticket
print(f"  APTicket: {len(ticket)} bytes")

# Stitch
img4 = IMG4(im4p=im4p, im4m=ticket)
img4_data = img4.output()
print(f"  IMG4: {len(img4_data)} bytes")

# Find Recovery device
be = libusb_package.get_libusb1_backend()
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    print("No recovery device!")
    exit(1)

dev.set_configuration(1)
try:
    usb.util.claim_interface(dev, 0)
except:
    pass

# Initiate upload
print(f"\nSending {len(img4_data)} bytes to iBSS...")
try:
    dev.ctrl_transfer(0x41, 0, 0, 0, timeout=5000)
except usb.core.USBError as e:
    print(f"  Initiate: {e}")

# Send via bulk EP 0x04
BLOCK = 8192
offset = 0
block = 0
t0 = time.time()
while offset < len(img4_data):
    chunk = img4_data[offset:offset + BLOCK]
    try:
        n = dev.write(0x04, chunk, timeout=5000)
        offset += n
        block += 1
        if block % 50 == 0:
            pct = 100 * offset // len(img4_data)
            print(f"  {offset}/{len(img4_data)} ({pct}%)")
    except usb.core.USBError as e:
        print(f"  Write error at {offset}: {e}")
        break

dt = time.time() - t0
print(f"  Sent {offset} bytes in {dt:.1f}s")

# Signal done with go command
print("Signaling iBSS to execute iBEC...")
try:
    dev.ctrl_transfer(0x40, 0, 0, 0, b"go\x00", timeout=5000)
    print("  'go' command sent")
except usb.core.USBError as e:
    print(f"  go error: {e}")

# Wait for iBEC to boot
print("\nWaiting for iBEC...")
time.sleep(3)

for i in range(20):
    for pid, name in [(0x1281, "Recovery"), (0x1227, "DFU")]:
        d = usb.core.find(idVendor=0x05AC, idProduct=pid, backend=be)
        if d:
            try:
                d.set_configuration(1)
                s1 = usb.util.get_string(d, 1)
                s4 = None
                try:
                    s4 = usb.util.get_string(d, 4)
                except:
                    pass
                print(f"Found {name} (0x{pid:04X})")
                if s1:
                    print(f"  String #1: {s1[:120]}")
                if s4:
                    print(f"  String #4: {s4[:120]}")

                if name == "Recovery":
                    try:
                        usb.util.claim_interface(d, 0)
                    except:
                        pass
                    try:
                        d.ctrl_transfer(0x40, 0, 0, 0, b"getenv build-version\x00", timeout=5000)
                        r = d.ctrl_transfer(0xC0, 0, 0, 0, 512, timeout=3000)
                        ver = bytes(r).rstrip(b"\x00").decode("utf-8", errors="replace")
                        print(f"  build-version: {ver}")
                    except:
                        pass
                exit(0)
            except Exception as e:
                print(f"  {name} open error: {e}")
    time.sleep(2)
    elapsed = (i + 1) * 2 + 3
    print(f"  {elapsed}s...")

print("No device found after 43s")
