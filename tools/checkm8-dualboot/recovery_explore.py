#!/usr/bin/env python3
"""Connect to iPhone in Recovery mode (iBSS loaded) and explore."""
import usb.core, usb.util
import libusb_package
import time

be = libusb_package.get_libusb1_backend()

# Find recovery device
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    # Check DFU
    dfu = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
    if dfu:
        print("Device still in DFU mode (0x1227), not recovery")
    else:
        print("No Apple device found")
    exit(1)

print("=== RECOVERY MODE DEVICE ===")
print(f"VID:PID = {dev.idVendor:04X}:{dev.idProduct:04X}")

# Set configuration
try:
    dev.set_configuration()
except usb.core.USBError:
    pass  # Already configured

# String descriptors
for i in range(6):
    try:
        s = usb.util.get_string(dev, i)
        if s:
            print(f"String #{i}: {s}")
    except:
        pass

# Enumerate
cfg = dev.get_active_configuration()
print(f"\nConfiguration {cfg.bConfigurationValue}, {cfg.bNumInterfaces} interfaces")

for intf in cfg:
    print(f"\n  Interface {intf.bInterfaceNumber} alt={intf.bAlternateSetting}")
    print(f"    Class={intf.bInterfaceClass} SubClass={intf.bInterfaceSubClass} Proto={intf.bInterfaceProtocol}")
    print(f"    Endpoints: {intf.bNumEndpoints}")
    for ep in intf:
        d = 'IN' if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN else 'OUT'
        t = {0: 'CTRL', 1: 'ISOC', 2: 'BULK', 3: 'INTR'}[usb.util.endpoint_type(ep.bmAttributes)]
        print(f"      EP 0x{ep.bEndpointAddress:02X} ({d}) type={t} maxPkt={ep.wMaxPacketSize}")

# Try sending recovery commands via control transfer
# Recovery mode uses: bmRequestType=0x40 (vendor OUT), bRequest=0, cmd in data
print("\n=== RECOVERY COMMANDS ===")

def recv_response(dev, size=512, timeout=2000):
    """Read response from BULK IN endpoint."""
    try:
        data = dev.read(0x81, size, timeout=timeout)
        return bytes(data)
    except usb.core.USBTimeoutError:
        return None
    except usb.core.USBError as e:
        return f"error: {e}"

def send_command(dev, cmd, timeout=5000):
    """Send command via control transfer (Recovery protocol)."""
    try:
        dev.ctrl_transfer(0x40, 0, 0, 0, cmd.encode() + b'\x00', timeout=timeout)
        return True
    except usb.core.USBError as e:
        print(f"  cmd error: {e}")
        return False

# Claim interface 0 and alt setting
try:
    usb.util.claim_interface(dev, 0)
except:
    pass

try:
    usb.util.claim_interface(dev, 1)
    dev.set_interface_altsetting(1, 1)
except:
    pass

# Try getenv commands
commands = [
    "getenv build-version",
    "getenv build-style", 
    "getenv firmware-version",
    "getenv product-name",
    "getenv platform-name",
    "getenv device-name",
    "getenv chip-id",
    "getenv board-id",
    "getenv device-color",
    "getenv auto-boot",
]

for cmd in commands:
    print(f"\n> {cmd}")
    if send_command(dev, cmd):
        # Try to read response  
        resp = recv_response(dev)
        if resp and not isinstance(resp, str):
            try:
                text = resp.decode('utf-8', errors='replace').strip('\x00').strip()
                print(f"  => {text}")
            except:
                print(f"  => {resp.hex()[:80]}")
        elif isinstance(resp, str):
            print(f"  => {resp}")
        else:
            print("  => (no response)")

# Try to get environment dump
print("\n> printenv")
if send_command(dev, "printenv"):
    # Read potentially large response
    full_resp = b""
    for _ in range(10):
        chunk = recv_response(dev, size=16384, timeout=1000)
        if chunk and not isinstance(chunk, str):
            full_resp += chunk
            if len(chunk) < 16384:
                break
        else:
            break
    if full_resp:
        text = full_resp.decode('utf-8', errors='replace').strip('\x00').strip()
        print(f"  Environment ({len(full_resp)} bytes):")
        for line in text.split('\n')[:40]:
            print(f"    {line}")
