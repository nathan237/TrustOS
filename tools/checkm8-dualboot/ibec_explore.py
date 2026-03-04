#!/usr/bin/env python3
"""Full iBEC console exploration."""
import usb.core, usb.util, libusb_package, time

be = libusb_package.get_libusb1_backend()
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    print("No recovery device")
    exit(1)

print(f"VID:PID = {dev.idVendor:04X}:{dev.idProduct:04X}")

# Full setup like idevicerestore
try:
    dev.set_configuration(1)
    print("Configuration set")
except Exception as e:
    print(f"set_config: {e}")

# Claim interface 0
for intf_num in [0, 1]:
    try:
        if dev.is_kernel_driver_active(intf_num):
            dev.detach_kernel_driver(intf_num)
    except:
        pass
    try:
        usb.util.claim_interface(dev, intf_num)
        print(f"Interface {intf_num} claimed")
    except Exception as e:
        print(f"Interface {intf_num} claim: {e}")

# Set alt setting for interface 1
try:
    dev.set_interface_altsetting(1, 1)
    print("Interface 1 alt=1 set")
except Exception as e:
    print(f"Alt setting: {e}")

# Read string descriptors
for i in [1, 2, 3, 4, 5]:
    try:
        s = usb.util.get_string(dev, i)
        if s:
            print(f"String #{i}: {s}")
    except:
        pass

# Enumerate endpoints
try:
    cfg = dev.get_active_configuration()
    for intf in cfg:
        for ep in intf:
            d = "IN" if ep.bEndpointAddress & 0x80 else "OUT"
            t = ["CTRL", "ISO", "BULK", "INT"][usb.util.endpoint_type(ep.bmAttributes)]
            print(f"  EP 0x{ep.bEndpointAddress:02X} ({d}) {t} maxPkt={ep.wMaxPacketSize}")
except Exception as e:
    print(f"Enumerate: {e}")

def send_cmd(cmd_str):
    """Send command and get response."""
    try:
        dev.ctrl_transfer(0x40, 0, 0, 0, cmd_str.encode() + b"\x00", timeout=5000)
    except usb.core.USBError as e:
        return f"SEND_ERROR({e})"
    
    try:
        resp = dev.ctrl_transfer(0xC0, 0, 0, 0, 4096, timeout=3000)
        return bytes(resp).rstrip(b"\x00").decode("utf-8", errors="replace").strip()
    except usb.core.USBError:
        return "STALL"

print("\n=== Environment Variables ===")
envvars = [
    "build-version", "build-style", "auto-boot", "config_board",
    "loadaddr", "boot-device", "boot-path", "display-timing",
    "display-color-space", "idle-off", "current-running-image",
    "debug-uarts", "debug-shmcon", "bootdelay", "boot-partition",
    "com.apple.System.boot-nonce", "effective-production-status-ap",
    "effective-security-mode-ap", "permit-random-gen-nonce",
]

for var in envvars:
    val = send_cmd(f"getenv {var}")
    if val and val != "STALL":
        print(f"  {var} = {val}")

print("\n=== Command Probing ===")
commands = [
    "version", "help", "printenv", "devicetree", "meminfo",
    "bgcolor 0 128 0", "bgcolor 0 0 0",
]
for c in commands:
    val = send_cmd(c)
    display = val[:500] if val else "(empty)"
    print(f"  [{c}] => {display}")
