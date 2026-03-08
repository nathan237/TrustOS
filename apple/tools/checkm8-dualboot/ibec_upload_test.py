#!/usr/bin/env python3
"""Test what iBEC accepts — firmware upload path + various ctrl experiments"""
import usb.core
import usb.util
import usb.backend.libusb1
import time
import sys
import struct

try:
    import libusb_package
    be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
except:
    be = None

dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    print("No device")
    sys.exit(1)

print(f"Mode: Recovery (PID=0x1281)")
try:
    dev.set_configuration()
except:
    pass
for i in range(2):
    try:
        usb.util.claim_interface(dev, i)
    except:
        pass

# === Test 1: zero-length ctrl OUT ===
print("\n=== ZERO-LENGTH CTRL 0x40 ===")
for bReq in range(4):
    try:
        ret = dev.ctrl_transfer(0x40, bReq, 0, 0, b"", timeout=2000)
        print(f"  bReq={bReq}: OK ({ret})")
        # Try read
        try:
            r = dev.ctrl_transfer(0xC0, 0, 0, 0, 256, timeout=500)
            print(f"    Read: {bytes(r)!r}")
        except:
            pass
    except Exception as e:
        print(f"  bReq={bReq}: {e}")

# === Test 2: single null byte ===
print("\n=== SINGLE NULL BYTE ===")
try:
    ret = dev.ctrl_transfer(0x40, 0, 0, 0, b"\x00", timeout=2000)
    print(f"  OK: {ret}")
except Exception as e:
    print(f"  {e}")

# === Test 3: Try DFU-specific ctrl (bmRequestType = 0x21 class OUT) ===
print("\n=== DFU CLASS CTRL ===")
# DFU_DNLOAD = 0x21, bRequest=1
for bReq in range(6):
    try:
        ret = dev.ctrl_transfer(0x21, bReq, 0, 0, b"test\x00", timeout=1000)
        print(f"  0x21 bReq={bReq}: OK ({ret})")
    except Exception as e:
        err = str(e)
        if "Pipe" in err:
            pass  # Expected
        else:
            print(f"  0x21 bReq={bReq}: {e}")

# === Test 4: Try uploading a small buffer to EP 0x04 ===
print("\n=== FIRMWARE UPLOAD TEST (EP 0x04) ===")
# Try to initiate a transfer
try:
    # This is how send_ibec_v2.py started the transfer
    ret = dev.ctrl_transfer(0x41, 0, 0, 0, b"", timeout=2000)
    print(f"  Init (0x41): OK ({ret})")
except Exception as e:
    print(f"  Init (0x41): {e}")

# Try writing small data to EP 0x04
small_test = b"TEST" * 128  # 512 bytes
try:
    written = dev.write(0x04, small_test, timeout=5000)
    print(f"  EP 0x04 write: {written} bytes")
except usb.core.USBTimeoutError:
    print(f"  EP 0x04 write: TIMEOUT")
except Exception as e:
    print(f"  EP 0x04 write: {e}")

# === Test 5: Check if setenv works differently ===
print("\n=== SETENV ATTEMPTS ===")
# Maybe setenv needs a specific bRequest value
for bReq in range(5):
    cmd = b"setenv auto-boot false\x00"
    try:
        ret = dev.ctrl_transfer(0x40, bReq, 0, 0, cmd, timeout=2000)
        print(f"  bReq={bReq}: OK ({ret}) !!!")
    except Exception as e:
        err = str(e)
        if "Pipe" in err:
            continue
        print(f"  bReq={bReq}: {e}")
print("  (all Pipe errors = ctrl 0x40 fully disabled)")

# === Test 6: GET_STATUS / GET_DESCRIPTOR for more info ===
print("\n=== USB STANDARD REQUESTS ===")
# GET_STATUS (standard)
try:
    ret = dev.ctrl_transfer(0x80, 0x00, 0, 0, 2, timeout=1000)
    print(f"  GET_STATUS: {bytes(ret).hex()}")
except Exception as e:
    print(f"  GET_STATUS: {e}")

# GET_DESCRIPTOR - device
try:
    ret = dev.ctrl_transfer(0x80, 0x06, 0x0100, 0, 18, timeout=1000)
    print(f"  Device desc: {bytes(ret).hex()}")
except Exception as e:
    print(f"  Device desc: {e}")

# GET_DESCRIPTOR - BOS
try:
    ret = dev.ctrl_transfer(0x80, 0x06, 0x0F00, 0, 64, timeout=1000)
    print(f"  BOS desc ({len(ret)} bytes): {bytes(ret).hex()}")
except Exception as e:
    print(f"  BOS: {e}")

# === Test 7: Read 0xC0 with large buffer ===
print("\n=== LARGE CTRL READ ===")
try:
    ret = dev.ctrl_transfer(0xC0, 0, 0, 0, 4096, timeout=2000)
    data = bytes(ret)
    print(f"  Got {len(data)} bytes")
    print(f"  Data: {data!r}")
except Exception as e:
    print(f"  {e}")

# === Summary ===
print("\n=== SUMMARY ===")
print("- ctrl 0x40 (send command): ALL STALL")
print("- ctrl 0xC0 (read resp): returns 'n841' (default/buffered)")
print("- EP 0x04 (firmware): tested above")
print("- EP 0x02/0x81 (serial): writes OK, reads timeout")
print()
print("RECOMMENDATION: Restart from DFU, set auto-boot=false in iBSS,")
print("then send iBEC. This should enable interactive iBEC console.")
