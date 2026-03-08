#!/usr/bin/env python3
"""Try to access Recovery mode device despite Apple driver."""
import usb.core, usb.util, libusb_package

be = libusb_package.get_libusb1_backend()
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    print("No recovery device found")
    exit(1)

print("Found Recovery device (PID=0x1281)")

# Try to detach kernel driver
for intf in range(2):
    try:
        if dev.is_kernel_driver_active(intf):
            print(f"  Kernel driver active on intf {intf}, detaching...")
            dev.detach_kernel_driver(intf)
            print(f"  Detached intf {intf}!")
        else:
            print(f"  No kernel driver on intf {intf}")
    except Exception as e:
        print(f"  Kernel driver intf {intf}: {e}")

# Try set_configuration
try:
    dev.set_configuration()
    print("set_configuration OK!")
except Exception as e:
    print(f"set_configuration: {e}")

# Try control transfer - read string descriptor 1
try:
    ret = dev.ctrl_transfer(0x80, 6, 0x0301, 0x0409, 255, timeout=5000)
    serial = bytes(ret[2:]).decode("utf-16-le", errors="replace")
    print(f"Serial: {serial}")
except Exception as e:
    print(f"ctrl_transfer: {e}")

# Try to enum config
try:
    cfg = dev.get_active_configuration()
    print(f"Config: {cfg.bConfigurationValue}")
    for intf in cfg:
        print(f"  Interface {intf.bInterfaceNumber}: {intf.bNumEndpoints} endpoints")
        for ep in intf:
            d = 'IN' if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN else 'OUT'
            print(f"    EP 0x{ep.bEndpointAddress:02X} ({d}) maxPkt={ep.wMaxPacketSize}")
except Exception as e:
    print(f"get_active_configuration: {e}")

# Try sending a recovery command via control transfer
try:
    cmd = b"getenv build-version\x00"
    dev.ctrl_transfer(0x40, 0, 0, 0, cmd, timeout=5000)
    print("Command sent!")
    # Read response
    data = dev.read(0x81, 512, timeout=2000)
    print(f"Response: {bytes(data)}")
except Exception as e:
    print(f"Send command: {e}")
