#!/usr/bin/env python3
"""Check device state - iBSS vs iBEC, and try pymobiledevice3 irecv"""
import usb.core
import usb.util
import usb.backend.libusb1
import sys
import time

# Use libusb1
try:
    import libusb_package
    be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
except:
    be = None

dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281)
if not dev:
    print("No device")
    sys.exit(1)

print(f"Device: VID={dev.idVendor:#06x} PID={dev.idProduct:#06x}")
print(f"Manufacturer: {dev.manufacturer}")
print(f"Product: {dev.product}")

# Read ALL string descriptors
print("\n=== STRING DESCRIPTORS ===")
for i in range(10):
    try:
        s = usb.util.get_string(dev, i)
        if s:
            print(f"  String #{i}: {s}")
    except:
        pass

# Read serial number (often contains mode info)
print(f"\nSerial Number: {dev.serial_number}")

# Check device descriptor details
print(f"\nbDeviceClass: {dev.bDeviceClass}")
print(f"bDeviceSubClass: {dev.bDeviceSubClass}")
print(f"bDeviceProtocol: {dev.bDeviceProtocol}")
print(f"bNumConfigurations: {dev.bNumConfigurations}")

# Configuration details
cfg = dev.get_active_configuration()
print(f"\nConfiguration: {cfg.bConfigurationValue}")
print(f"Num Interfaces: {cfg.bNumInterfaces}")

for intf in cfg:
    print(f"\n  Interface {intf.bInterfaceNumber} alt={intf.bAlternateSetting}")
    print(f"    Class: {intf.bInterfaceClass} SubClass: {intf.bInterfaceSubClass} Protocol: {intf.bInterfaceProtocol}")
    for ep in intf:
        direction = "IN" if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN else "OUT"
        etype = {0: "CTRL", 1: "ISO", 2: "BULK", 3: "INT"}[usb.util.endpoint_type(ep.bmAttributes)]
        print(f"    EP 0x{ep.bEndpointAddress:02X} {direction} {etype} maxpkt={ep.wMaxPacketSize}")

# === Try pymobiledevice3 IRecv ===
print("\n=== PYMOBILEDEVICE3 IRECV ===")
try:
    from pymobiledevice3.irecv import IRecv
    client = IRecv()
    print(f"IRecv mode: {client.mode}")
    print(f"IRecv product_type: {client.product_type}")
    
    # Try sending a command
    try:
        client.send_command("getenv build-version")
        print("Command sent OK!")
    except Exception as e:
        print(f"send_command: {e}")
    
    # Try getenv
    try:
        val = client.getenv("build-version")
        print(f"build-version: {val}")
    except Exception as e:
        print(f"getenv: {e}")
        
except Exception as e:
    print(f"IRecv failed: {e}")

# === Alternative: try to detect mode from SRTG ===
print("\n=== MODE DETECTION ===")
# Try reading SRTG-style info from ctrl
# In Apple DFU/Recovery, the serial number string descriptor contains CPID, BDID, etc.
# But the SRTG tag tells us the exact firmware stage
try:
    # Re-read serial number raw
    sn = dev.serial_number
    print(f"Serial string: {sn}")
    if "SRTG:" in sn:
        srtg = sn.split("SRTG:")[1].split("]")[0] if "]" in sn else sn.split("SRTG:")[1]
        print(f"SRTG: {srtg}")
    if "CPID:" in sn:
        print("  -> Device is in DFU/Recovery mode")
        if "iBSS" in sn:
            print("  -> Running iBSS")
        elif "iBEC" in sn:
            print("  -> Running iBEC")
except Exception as e:
    print(f"Mode detection: {e}")

# === Try with specific interface claim order ===
print("\n=== RETRY CTRL WITH FRESH CLAIM ===")
dev2 = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if dev2:
    try:
        dev2.reset()
        time.sleep(0.5)
    except:
        pass
    
    dev2 = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
    if dev2:
        try:
            dev2.set_configuration()
        except:
            pass
        try:
            usb.util.claim_interface(dev2, 0)
        except:
            pass
        
        # Try 0x40 after fresh reset
        try:
            ret = dev2.ctrl_transfer(0x40, 0, 0, 0, b"getenv build-version\x00", timeout=5000)
            print(f"After reset - 0x40 write: {ret} bytes")
            time.sleep(0.1)
            ret = dev2.ctrl_transfer(0xC0, 0, 0, 0, 256, timeout=1000)
            print(f"After reset - 0xC0 read: {bytes(ret)!r}")
        except Exception as e:
            print(f"After reset: {e}")

print("\nDone.")
