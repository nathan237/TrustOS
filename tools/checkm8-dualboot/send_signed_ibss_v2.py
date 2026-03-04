#!/usr/bin/env python3
"""Send signed iBSS to DFU device using correct protocol (CRC, wValue, USB reset)."""
import plistlib, time, struct, binascii, math
import usb.core, usb.util
import libusb_package
from pymobiledevice3.restore.img4 import IMG4, IM4P
from pymobiledevice3.restore.tss import TSSResponse

PACKET_SIZE = 2048

def dfu_get_state(dev):
    """Get DFU state (bRequest=5 GETSTATE)."""
    resp = dev.ctrl_transfer(0xA1, 5, 0, 0, 1, timeout=5000)
    return resp[0]

def dfu_get_status(dev):
    """Get DFU status (bRequest=3 GETSTATUS). Returns (bStatus, bwPollTimeout, bState)."""
    resp = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=5000)
    status = resp[0]
    poll = resp[1] | (resp[2] << 8) | (resp[3] << 16)
    state = resp[4]
    return status, poll, state

def dfu_clrstatus(dev):
    """Clear DFU status (bRequest=4)."""
    dev.ctrl_transfer(0x21, 4, 0, 0, timeout=5000)

def dfu_abort(dev):
    """Abort DFU (bRequest=6)."""
    dev.ctrl_transfer(0x21, 6, 0, 0, timeout=5000)

# Load iBSS IM4P
print("Loading iBSS IM4P...")
with open('tools/checkm8-dualboot/cache/5f574195af3c7b8bd9a14dcd1eed019c_iBSS.n841.RELEASE.im4p', 'rb') as f:
    ibss_im4p_data = f.read()

# Load TSS response
print("Loading TSS response...")
with open('tools/checkm8-dualboot/cache/tss_success_response.plist', 'rb') as f:
    tss_dict = plistlib.load(f)
tss_resp = TSSResponse(tss_dict)
ticket = tss_resp.ap_img4_ticket

# Stitch IMG4
print("Stitching IMG4...")
im4p = IM4P(data=ibss_im4p_data)
img4_data = IMG4(im4p=im4p, im4m=ticket).output()
print(f"  IMG4: {len(img4_data)} bytes")

# Find DFU device
print("\nLooking for DFU device...")
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227,
                     backend=libusb_package.get_libusb1_backend())
if dev is None:
    print("ERROR: DFU device not found!")
    exit(1)

# Verify nonce
s1 = usb.util.get_string(dev, 1)
nonc = s1.split('NONC:')[1].split(' ')[0]
expected_nonc = '38B4510C5DBB94AB3CE33A55C3FF45133F9AD1B2E438F1DBC90CE90DBA2F6539'
print(f"  NONC match: {nonc == expected_nonc}")
if nonc != expected_nonc:
    print("WARNING: Nonce mismatch! Need new TSS ticket.")
    exit(1)

# Check initial state
state = dfu_get_state(dev)
print(f"  Initial state: {state}")
if state == 10:
    print("  Device in error state, clearing...")
    dfu_clrstatus(dev)
    state = dfu_get_state(dev)
    print(f"  State after clear: {state}")
if state not in (2, 8):
    print(f"  Unexpected state {state}, aborting...")
    try:
        dfu_abort(dev)
        state = dfu_get_state(dev)
        print(f"  State after abort: {state}")
    except:
        pass
if state == 8:
    # MANIFEST-WAIT-RESET from previous transfer, need USB reset
    print("  State 8 (MANIFEST-WAIT-RESET), resetting USB...")
    try:
        dev.reset()
    except:
        pass
    time.sleep(3)
    dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227,
                         backend=libusb_package.get_libusb1_backend())
    if dev is None:
        print("  Device lost after reset, waiting...")
        time.sleep(5)
        dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227,
                             backend=libusb_package.get_libusb1_backend())
    if dev is None:
        print("ERROR: Device not recovered after reset")
        exit(1)
    state = dfu_get_state(dev)
    print(f"  State after reset: {state}")

# Send IMG4 using proper DFU protocol
buf = img4_data
num_packets = math.ceil(len(buf) / PACKET_SIZE)
print(f"\nSending {len(buf)} bytes in {num_packets} packets...")

crc = 0xFFFFFFFF  # Start with -1 as unsigned

t_start = time.time()
for packet_index in range(num_packets):
    offset = packet_index * PACKET_SIZE
    chunk = buf[offset:offset + PACKET_SIZE]
    
    if offset + PACKET_SIZE >= len(buf):
        # LAST PACKET - append CRC
        # Calculate CRC of all data
        crc = binascii.crc32(buf, crc) & 0xFFFFFFFF
        
        # Add CRC of dfu_xbuf (salted value)
        dfu_xbuf = bytearray([0xFF, 0xFF, 0xFF, 0xFF, 0xAC, 0x05, 0x00, 0x01, 
                               0x55, 0x46, 0x44, 0x10])
        crc = binascii.crc32(dfu_xbuf, crc) & 0xFFFFFFFF
        crc_chunk = bytes(dfu_xbuf) + struct.pack("<I", crc)
        
        if len(chunk) + 16 > PACKET_SIZE:
            # CRC exceeds packet size, send separately
            dev.ctrl_transfer(0x21, 1, packet_index, 0, chunk, timeout=5000)
            dev.ctrl_transfer(0x21, 1, packet_index, 0, crc_chunk, timeout=5000)
        else:
            dev.ctrl_transfer(0x21, 1, packet_index, 0, chunk + crc_chunk, timeout=5000)
    else:
        dev.ctrl_transfer(0x21, 1, packet_index, 0, chunk, timeout=5000)
    
    if (packet_index + 1) % 100 == 0:
        print(f"  Packet {packet_index + 1}/{num_packets}")

t_send = time.time() - t_start
print(f"  Sent {num_packets} packets in {t_send:.1f}s")

# Post-send: wait for status == 5 (dfuDNBUSY)
print("\nWaiting for dfuDNBUSY (status 5)...")
for i in range(30):
    try:
        bStatus, bPoll, bState = dfu_get_status(dev)
        print(f"  Status: bStatus={bStatus} bState={bState} poll={bPoll}ms")
        if bState == 5:
            print("  Got dfuDNBUSY!")
            break
        if bPoll > 0:
            time.sleep(bPoll / 1000.0)
        else:
            time.sleep(0.5)
    except usb.core.USBError as e:
        print(f"  USB error: {e}")
        time.sleep(1)
        break

# Send final empty DNLOAD with wValue=num_packets
print(f"\nSending final empty DNLOAD (wValue={num_packets})...")
try:
    dev.ctrl_transfer(0x21, 1, num_packets, 0, b'', timeout=5000)
    print("  Sent")
except usb.core.USBError as e:
    print(f"  Error: {e}")

# Read status twice (as idevicerestore does)
for i in range(2):
    try:
        bStatus, bPoll, bState = dfu_get_status(dev)
        print(f"  Post-send status {i+1}: bStatus={bStatus} bState={bState} poll={bPoll}ms")
        if bPoll > 0:
            time.sleep(bPoll / 1000.0)
    except usb.core.USBError as e:
        print(f"  Post-send status {i+1}: USB error: {e}")

# USB RESET - critical for DFU to actually boot the firmware!
print("\nSending USB reset...")
try:
    dev.reset()
    print("  USB reset sent!")
except usb.core.USBError as e:
    print(f"  Reset error: {e}")

# Wait and check for Recovery mode
print("\nWaiting for Recovery mode device (PID=0x1281)...")
for wait in range(8):
    time.sleep(2)
    recovery_dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281,
                                  backend=libusb_package.get_libusb1_backend())
    if recovery_dev:
        print(f"*** RECOVERY MODE FOUND at {wait*2+2}s! ***")
        try:
            s = usb.util.get_string(recovery_dev, 1)
            print(f"  Serial: {s[:120]}")
        except:
            print("  (couldn't read serial)")
        break
    # Also check DFU
    dfu_check = usb.core.find(idVendor=0x05AC, idProduct=0x1227,
                               backend=libusb_package.get_libusb1_backend())
    if dfu_check:
        print(f"  {wait*2+2}s: Still DFU")
    else:
        print(f"  {wait*2+2}s: Device disconnected (booting?)")
else:
    print("No recovery device found after 16s")
    # Final check for any Apple device
    for pid in [0x1227, 0x1281, 0x12A8]:
        d = usb.core.find(idVendor=0x05AC, idProduct=pid,
                           backend=libusb_package.get_libusb1_backend())
        if d:
            print(f"  Found device with PID=0x{pid:04X}")
