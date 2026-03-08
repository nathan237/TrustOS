#!/usr/bin/env python3
"""Stitch signed iBSS with APTicket and send to DFU device."""
import plistlib, time, struct
import usb.core, usb.util
import libusb_package

from pymobiledevice3.restore.img4 import stitch_component, IMG4, IM4P, IM4R
from pymobiledevice3.restore.tss import TSSResponse

# Load iBSS IM4P
print("Loading iBSS IM4P...")
with open('tools/checkm8-dualboot/cache/5f574195af3c7b8bd9a14dcd1eed019c_iBSS.n841.RELEASE.im4p', 'rb') as f:
    ibss_im4p_data = f.read()
print(f"  iBSS IM4P: {len(ibss_im4p_data)} bytes")

# Load TSS response (APTicket)
print("Loading TSS response...")
with open('tools/checkm8-dualboot/cache/tss_success_response.plist', 'rb') as f:
    tss_dict = plistlib.load(f)
tss_resp = TSSResponse(tss_dict)
ticket = tss_resp.ap_img4_ticket
print(f"  ApImg4Ticket: {len(ticket)} bytes")

# Stitch IMG4 = IM4P + IM4M (ticket)
print("\nStitching IMG4 container...")
im4p = IM4P(data=ibss_im4p_data)
print(f"  IM4P fourcc: {im4p.fourcc}")
print(f"  IM4P description: {im4p.description}")

img4 = IMG4(im4p=im4p, im4m=ticket)
img4_data = img4.output()
print(f"  IMG4 output: {len(img4_data)} bytes")

# Save stitched IMG4
with open('tools/checkm8-dualboot/cache/ibss_signed.img4', 'wb') as f:
    f.write(img4_data)
print("  Saved to cache/ibss_signed.img4")

# Find DFU device
print("\nLooking for DFU device...")
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227, 
                     backend=libusb_package.get_libusb1_backend())
if dev is None:
    print("ERROR: DFU device not found!")
    exit(1)
print(f"  Found: {dev.idVendor:04x}:{dev.idProduct:04x}")

# Read current string for confirmation
s1 = usb.util.get_string(dev, 1)
print(f"  Serial: {s1[:60]}...")

# Send IMG4 via DFU DNLOAD
# DFU protocol: bmRequestType=0x21, bRequest=1 (DNLOAD), wValue=0, wIndex=0
print(f"\nSending {len(img4_data)} bytes via DFU DNLOAD...")
BLOCK_SIZE = 2048  # Match SecureROM buffer size
offset = 0
block_num = 0
total_blocks = (len(img4_data) + BLOCK_SIZE - 1) // BLOCK_SIZE

t_start = time.time()
while offset < len(img4_data):
    chunk = img4_data[offset:offset + BLOCK_SIZE]
    try:
        dev.ctrl_transfer(0x21, 1, 0, 0, chunk, timeout=5000)
    except usb.core.USBError as e:
        print(f"  Block {block_num}/{total_blocks}: USB error: {e}")
        break
    offset += len(chunk)
    block_num += 1
    if block_num % 100 == 0:
        print(f"  Block {block_num}/{total_blocks} ({offset}/{len(img4_data)} bytes)")

t_send = time.time() - t_start
print(f"  Sent {block_num} blocks in {t_send:.1f}s")

# Send empty DNLOAD to signal end (ZLP)
print("\nSending empty DNLOAD (ZLP) to trigger manifest...")
try:
    dev.ctrl_transfer(0x21, 1, 0, 0, b'', timeout=5000)
    print("  ZLP sent successfully")
except usb.core.USBError as e:
    print(f"  ZLP error: {e}")

# Wait for device to process - check DFU status
print("\nWaiting for device to process firmware...")
time.sleep(1)

for i in range(10):
    try:
        # DFU_GETSTATUS: bmRequestType=0xA1, bRequest=3
        status = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=5000)
        bStatus = status[0]
        bwPollTimeout = status[1] | (status[2] << 8) | (status[3] << 16)
        bState = status[4]
        print(f"  Status check {i+1}: status={bStatus}, state={bState}, poll={bwPollTimeout}ms")
        
        # State 2 = dfuDNLOAD-IDLE (more data expected)
        # State 5 = dfuMANIFEST (processing)
        # State 7 = dfuMANIFEST-WAIT-RESET 
        # State 8 = dfuDNLOAD-IDLE after manifest
        if bState == 7:
            print("  Device in MANIFEST-WAIT-RESET state!")
            break
        if bwPollTimeout > 0:
            time.sleep(bwPollTimeout / 1000.0)
        else:
            time.sleep(0.5)
    except usb.core.USBError as e:
        print(f"  Status check {i+1}: USB error: {e}")
        break

# Check if device rebooted to Recovery mode
print("\nChecking for Recovery mode device (PID=0x1281)...")
time.sleep(3)
recovery_dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281,
                              backend=libusb_package.get_libusb1_backend())
if recovery_dev:
    print("*** RECOVERY MODE DEVICE FOUND! ***")
    try:
        s = usb.util.get_string(recovery_dev, 1)
        print(f"  Serial: {s[:100]}")
    except:
        pass
else:
    # Also check if still in DFU
    dfu_dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227,
                             backend=libusb_package.get_libusb1_backend())
    if dfu_dev:
        print("Device still in DFU mode")
        try:
            status = dfu_dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=5000)
            print(f"  Final status: {list(status)}")
        except:
            pass
    else:
        print("Device disconnected (may be booting...)")
        # Wait more and check again
        time.sleep(5)
        recovery_dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281,
                                      backend=libusb_package.get_libusb1_backend())
        if recovery_dev:
            print("*** RECOVERY MODE DEVICE FOUND (after wait)! ***")
        else:
            print("No recovery device found after 8s wait")
