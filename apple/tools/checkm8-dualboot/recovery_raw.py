#!/usr/bin/env python3
"""Explore iBoot Recovery mode using raw pyusb (bypass IRecv issue)."""
import usb.core, usb.util, libusb_package, time

be = libusb_package.get_libusb1_backend()
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    print("No recovery device")
    exit(1)

print("=== RECOVERY MODE ===")
print(f"VID:PID = {dev.idVendor:04X}:{dev.idProduct:04X}")

# Set configuration
dev.set_configuration(1)

# Claim interfaces
try:
    usb.util.claim_interface(dev, 0)
    print("Interface 0 claimed")
except:
    pass
try:
    usb.util.claim_interface(dev, 1)
    dev.set_interface_altsetting(1, 1)  # Alt setting 1 has the bulk endpoints
    print("Interface 1 alt=1 claimed")
except Exception as e:
    print(f"Interface 1 claim error: {e}")

# Recovery mode command protocol:
# Send: ctrl_transfer OUT (0x40, bRequest=0, wValue=0, wIndex=0, data=command+\0)
# Response: ctrl_transfer IN (0xC0, bRequest=0, wValue=0, wIndex=0, len=N)

def send_cmd(dev, cmd, read_response=True):
    """Send command to iBoot and read response."""
    # Send command via control transfer (vendor, host-to-device)
    dev.ctrl_transfer(0x40, 0, 0, 0, cmd.encode() + b"\x00", timeout=5000)
    
    if not read_response:
        return ""
    
    # Read response  
    try:
        resp = dev.ctrl_transfer(0xC0, 0, 0, 0, 4096, timeout=3000)
        return bytes(resp).rstrip(b"\x00").decode("utf-8", errors="replace").strip()
    except usb.core.USBError:
        return "(no response)"

# Environment variables
envvars = [
    "build-version", "build-style", "firmware-version", "platform-name",
    "chip-id", "board-id", "security-domain", "auto-boot", "dark-boot",
    "boot-args", "display-color-space", "display-timing", "device-name",
    "usb-enabled", "debug-enabled", "production-cert",
]

print("\n=== Environment Variables ===")
for var in envvars:
    try:
        val = send_cmd(dev, f"getenv {var}")
        if val and val != "(no response)":
            print(f"  {var}: {val}")
    except Exception as e:
        print(f"  {var}: error - {e}")

# Try some direct commands
print("\n=== Commands ===")

# bgcolor - changes screen color (useful test)
try:
    resp = send_cmd(dev, "bgcolor 0 0 255")
    print(f"bgcolor 0 0 255: {resp}")
except Exception as e:
    print(f"bgcolor: {e}")

time.sleep(1)

try:
    resp = send_cmd(dev, "bgcolor 0 0 0")
    print(f"bgcolor 0 0 0: {resp}")
except Exception as e:
    print(f"bgcolor: {e}")
