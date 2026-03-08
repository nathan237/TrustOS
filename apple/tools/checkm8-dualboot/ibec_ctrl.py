#!/usr/bin/env python3
"""iBEC communication via ctrl transfers — force libusb1 backend"""
import usb.core
import usb.util
import usb.backend.libusb1
import time
import sys
import ctypes

# Try libusb1 backend first
try:
    import libusb_package
    be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
    print(f"Using libusb1 backend via libusb_package")
except:
    be = usb.backend.libusb1.get_backend()
    print(f"Using system libusb1 backend")

dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    print("No Recovery device found with libusb1, trying default...")
    dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281)
    if not dev:
        print("No device found at all")
        sys.exit(1)

print(f"Device: {dev.manufacturer} {dev.product}")
print(f"Backend: {dev.backend.__class__.__name__}")

# Setup
try:
    dev.set_configuration()
except:
    pass

for i in range(2):
    try:
        if dev.is_kernel_driver_active(i):
            dev.detach_kernel_driver(i)
    except:
        pass
    try:
        usb.util.claim_interface(dev, i)
    except:
        pass

# === Test 1: ctrl READ (should work — got n841 before) ===
print("\n=== CTRL READ TESTS (0xC0) ===")
for bReq in range(5):
    try:
        ret = dev.ctrl_transfer(0xC0, bReq, 0, 0, 256, timeout=1000)
        print(f"  bReq={bReq}: {len(ret)} bytes: {bytes(ret)[:100]!r}")
    except Exception as e:
        print(f"  bReq={bReq}: {e}")

# === Test 2: ctrl SEND command (the critical test) ===
print("\n=== CTRL SEND TESTS (0x40) ===")
# In irecovery: bmRequestType=0x40, bRequest=0, wValue=0, wIndex=0
commands = [
    b"getenv build-version\x00",
    b"bgcolor 0 0 255\x00",
    b"help\x00",
]

for cmd in commands:
    label = cmd.rstrip(b'\x00').decode()
    print(f"\n  Sending: '{label}'")
    try:
        ret = dev.ctrl_transfer(0x40, 0, 0, 0, cmd, timeout=2000)
        print(f"    Write OK: {ret} bytes")
        # Read response
        time.sleep(0.1)
        try:
            resp = dev.ctrl_transfer(0xC0, 0, 0, 0, 256, timeout=1000)
            print(f"    Response: {bytes(resp)!r}")
        except Exception as e:
            print(f"    Read: {e}")
    except Exception as e:
        print(f"    Write FAILED: {e}")

# === Test 3: Try wIndex=1 (interface 1) ===
print("\n=== CTRL WITH wIndex=1 ===")
try:
    ret = dev.ctrl_transfer(0x40, 0, 0, 1, b"getenv build-version\x00", timeout=2000)
    print(f"  Write OK: {ret}")
except Exception as e:
    print(f"  Write (wIndex=1): {e}")

# === Test 4: Try getenv via wValue as variable selector ===
print("\n=== CTRL READ with wValue variants ===")
for wVal in range(10):
    try:
        ret = dev.ctrl_transfer(0xC0, 0, wVal, 0, 256, timeout=500)
        print(f"  wValue={wVal}: {bytes(ret)!r}")
    except Exception as e:
        err = str(e)
        if "Pipe" in err or "timeout" in err.lower():
            pass
        else:
            print(f"  wValue={wVal}: {e}")

# === Test 5: Bulk IN — drain any serial output ===
print("\n=== BULK IN EP 0x81 ===")
try:
    dev.set_interface_altsetting(1, 1)
except:
    pass

total = b""
for _ in range(5):
    try:
        data = dev.read(0x81, 16384, timeout=500)
        chunk = bytes(data)
        total += chunk
        print(f"  Read {len(chunk)} bytes")
    except usb.core.USBTimeoutError:
        break
    except Exception as e:
        print(f"  Read: {e}")
        break

if total:
    print(f"  Total: {len(total)} bytes")
    print(f"  Data: {total.decode('utf-8', errors='replace')[:500]}")
else:
    print("  (no data)")

# === Test 6: Bulk OUT command + Bulk IN response ===
print("\n=== BULK OUT 0x02 -> BULK IN 0x81 ===")
test_cmds = [b"help\n", b"help\x00", b"getenv build-version\n", b"\n"]
for cmd in test_cmds:
    print(f"\n  Sending: {cmd!r}")
    try:
        dev.write(0x02, cmd, timeout=2000)
        time.sleep(0.3)
        try:
            resp = dev.read(0x81, 16384, timeout=1000)
            print(f"  Response ({len(resp)} bytes): {bytes(resp).decode('utf-8', errors='replace')[:300]}")
        except usb.core.USBTimeoutError:
            print(f"  (read timeout)")
        except Exception as e:
            print(f"  Read: {e}")
    except Exception as e:
        print(f"  Write: {e}")

print("\nDone.")
