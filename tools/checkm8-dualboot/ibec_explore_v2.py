#!/usr/bin/env python3
"""iBEC exploration using libusb1 backend for proper read/write support."""
import usb.core, usb.util, usb.backend.libusb1
import time, sys

try:
    import libusb_package
    be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
except:
    be = None

# Find device
print("=== DEVICE SEARCH ===")
for pid, name in [(0x1227, "DFU"), (0x1281, "Recovery")]:
    dev = usb.core.find(idVendor=0x05AC, idProduct=pid, backend=be)
    if dev:
        print(f"Found {name} device (PID=0x{pid:04X})")
        break
else:
    # Try without explicit backend
    for pid, name in [(0x1227, "DFU"), (0x1281, "Recovery")]:
        dev = usb.core.find(idVendor=0x05AC, idProduct=pid)
        if dev:
            backend_name = dev.backend.__class__.__name__
            print(f"Found {name} device via {backend_name}")
            break
    else:
        print("No device found!")
        sys.exit(1)

# Setup
try:
    dev.set_configuration()
except:
    pass
try:
    usb.util.claim_interface(dev, 0)
except:
    pass

print(f"Backend: {dev.backend.__class__.__name__}")

# Read strings
try:
    print(f"Product: {dev.product}")
except:
    print("Product: (unreadable)")
try:
    sn = dev.serial_number
    print(f"Serial: {sn[:100]}")
except:
    print("Serial: (unreadable)")

# === Test ctrl 0x40 SEND ===
print("\n=== CTRL 0x40 SEND TESTS ===")
def send_cmd(cmd, label=None):
    if label is None:
        label = cmd
    try:
        dev.ctrl_transfer(0x40, 0, 0, 0, cmd.encode() + b"\x00", timeout=5000)
        # Try reading response
        try:
            resp = dev.ctrl_transfer(0xC0, 0, 0, 0, 4096, timeout=3000)
            text = bytes(resp).rstrip(b"\x00").decode("utf-8", errors="replace").strip()
            print(f"  {label}: {text}" if text else f"  {label}: (ok, empty)")
            return text
        except usb.core.USBError as e:
            print(f"  {label}: send OK, read error: {e}")
            return None
    except usb.core.USBError as e:
        print(f"  {label}: STALL ({e})")
        return None

# Visual confirmation test
print("\nVisual test: bgcolor 0 255 0 (green)...")
send_cmd("bgcolor 0 255 0", "bgcolor green")
time.sleep(1)

print("Visual test: bgcolor 0 0 255 (blue)...")
send_cmd("bgcolor 0 0 255", "bgcolor blue")
time.sleep(1)

# Environment variables
print("\n=== ENVIRONMENT VARIABLES ===")
envvars = [
    "build-version", "build-style", "auto-boot", "boot-device",
    "boot-path", "loadaddr", "config_board", "display-timing",
    "debug-uarts", "debug-shmcon", "filesize", "bootdelay",
    "permit-random-gen-nonce",
]
for var in envvars:
    send_cmd(f"getenv {var}", var)

# Commands
print("\n=== COMMANDS ===")
for cmd in ["version", "help", "printenv", "meminfo"]:
    send_cmd(cmd)

# Memory dump attempts
print("\n=== MEMORY DUMP ===")
for addr, name in [
    ("0x100000000", "SecureROM"),
    ("0x802000000", "LoadAddr"),
    ("0x800000000", "SRAM"),
]:
    send_cmd(f"md {addr} 0x40", f"md {name}")

# Try bulk serial for responses
print("\n=== BULK SERIAL TEST ===")
try:
    usb.util.claim_interface(dev, 1)
    dev.set_interface_altsetting(1, 1)
except:
    pass

# Send a command that should produce output
send_cmd("bgcolor 255 0 0", "bgcolor red (for output test)")
time.sleep(0.5)

# Try to read from bulk IN
total = b""
for _ in range(5):
    try:
        data = dev.read(0x81, 16384, timeout=500)
        chunk = bytes(data)
        total += chunk
        print(f"  Bulk IN: {len(chunk)} bytes")
    except usb.core.USBTimeoutError:
        break
    except Exception as e:
        print(f"  Bulk IN: {e}")
        break

if total:
    print(f"  Total serial: {len(total)} bytes")
    print(f"  Data: {total.decode('utf-8', errors='replace')[:500]}")
else:
    print("  (no serial data)")

# Check DFU status (to determine actual mode)
print("\n=== DFU STATUS ===")
try:
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
    states = {0:"appIDLE", 1:"appDETACH", 2:"dfuIDLE", 3:"dfuDNLOAD-SYNC",
              4:"dfuDNBUSY", 5:"dfuDNLOAD-IDLE", 6:"dfuMANIFEST-SYNC",
              7:"dfuMANIFEST", 8:"dfuMANIFEST-WAIT-RESET", 9:"dfuUPLOAD-IDLE",
              10:"dfuERROR"}
    print(f"  Status={st[0]}, State={st[4]} ({states.get(st[4], '?')})")
except:
    print("  DFU status: not available (probably in Recovery, not DFU)")

# Get DFU state via GETSTATE
try:
    st = dev.ctrl_transfer(0xA1, 5, 0, 0, 1, timeout=1000)
    print(f"  GETSTATE: {st[0]}")
except:
    print("  GETSTATE: not available")

print("\nDone. Check device screen for color changes!")
