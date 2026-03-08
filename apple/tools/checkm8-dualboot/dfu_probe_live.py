#!/usr/bin/env python3
"""Quick DFU probe — reads status, descriptors, tests ZLP conditions."""
import usb.core, usb.util, libusb_package, usb.backend.libusb1

be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
if not dev:
    print("No DFU device found!")
    exit(1)

try:
    dev.set_configuration()
except:
    pass

STATES = {0:'appIDLE',1:'appDETACH',2:'dfuIDLE',3:'dfuDNLOAD-SYNC',
          4:'dfuDNBUSY',5:'dfuDNLOAD-IDLE',6:'dfuMANIFEST-SYNC',
          7:'dfuMANIFEST',8:'dfuMANIFEST-WAIT-RESET',9:'dfuUPLOAD-IDLE',10:'dfuERROR'}

# 1. DFU GET_STATUS
print("=== DFU GET_STATUS ===")
try:
    r = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
    state_name = STATES.get(r[4], "unknown")
    poll_ms = r[1] | (r[2] << 8) | (r[3] << 16)
    print(f"  bStatus={r[0]}, bState={r[4]} ({state_name}), pollTimeout={poll_ms}ms")
except Exception as e:
    print(f"  Error: {e}")

# 2. Device Descriptor
print("\n=== DEVICE DESCRIPTOR ===")
try:
    r = dev.ctrl_transfer(0x80, 6, 0x0100, 0, 255, timeout=2000)
    print(f"  Length: {len(r)} bytes")
    print(f"  bcdUSB: 0x{r[3]:02X}{r[2]:02X}")
    print(f"  bMaxPacketSize0: {r[7]}")
    print(f"  idVendor: 0x{r[9]:02X}{r[8]:02X}")
    print(f"  idProduct: 0x{r[11]:02X}{r[10]:02X}")
except Exception as e:
    print(f"  Error: {e}")

# 3. Config Descriptor
print("\n=== CONFIG DESCRIPTOR ===")
try:
    r = dev.ctrl_transfer(0x80, 6, 0x0200, 0, 255, timeout=2000)
    print(f"  Length: {len(r)} bytes")
    print(f"  Raw: {' '.join(f'{b:02X}' for b in r[:32])}")
except Exception as e:
    print(f"  Error: {e}")

# 4. Test wLength > actual (ZLP trigger condition)
# Device descriptor is 18 bytes. Request 64 bytes → if response is 18 (multiple of... no, not 64)
# Request exactly 64 bytes for a descriptor that returns <64 bytes
# standard_device_request_cb triggers ZLP when: io_length > 0 && io_length % 64 == 0 && wLength > io_length
print("\n=== ZLP CONDITION TEST ===")
try:
    # Request 192 bytes for device descriptor (actual=18). io_length will be 18, not mod 64 → no ZLP
    r = dev.ctrl_transfer(0x80, 6, 0x0100, 0, 192, timeout=2000)
    print(f"  DevDesc with wLength=192: got {len(r)} bytes (io_length={len(r)}, mod64={len(r)%64})")
    
    # Config descriptor — if it returns exactly 64 or 128 bytes with wLength > actual → ZLP
    r = dev.ctrl_transfer(0x80, 6, 0x0200, 0, 512, timeout=2000)
    print(f"  CfgDesc with wLength=512: got {len(r)} bytes (io_length={len(r)}, mod64={len(r)%64})")
except Exception as e:
    print(f"  Error: {e}")

# 5. Try small DFU DNLOAD to test DFU handler
print("\n=== DFU DNLOAD TEST (16 bytes) ===")
try:
    # Send 16 bytes of 0xAA — should transition to dfuDNLOAD-SYNC
    dev.ctrl_transfer(0x21, 1, 0, 0, bytes([0xAA]*16), timeout=2000)
    r = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
    state_name = STATES.get(r[4], "unknown")
    print(f"  After DNLOAD: bState={r[4]} ({state_name})")
except Exception as e:
    print(f"  Error: {e}")

# 6. DFU ABORT to reset
print("\n=== DFU ABORT (reset to idle) ===")
try:
    dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
    r = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
    state_name = STATES.get(r[4], "unknown")
    print(f"  After ABORT: bState={r[4]} ({state_name})")
except Exception as e:
    print(f"  Error: {e}")

print("\n=== DONE ===")
