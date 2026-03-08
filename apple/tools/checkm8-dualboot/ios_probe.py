#!/usr/bin/env python3
"""
iOS Device Probe — Detects iPhone in normal mode via USB and reads device info.
Works without screen — uses pymobiledevice3 to query lockdownd.
"""
import sys, time, json
from datetime import datetime

def probe_with_pymobiledevice3():
    """Try pymobiledevice3 (modern, well-maintained)."""
    try:
        from pymobiledevice3.usbmux import list_devices
        devices = list_devices()
        if not devices:
            print("pymobiledevice3: No devices found via usbmux")
            return None
        
        print(f"pymobiledevice3: Found {len(devices)} device(s)")
        for i, d in enumerate(devices):
            print(f"  Device {i}: {d}")
            # Try to get more info
            try:
                print(f"    Serial: {d.serial}")
                print(f"    UDID: {d.serial}")
                print(f"    Connection: {d.connection_type}")
            except:
                pass
        
        # Try lockdown connection for device info
        try:
            from pymobiledevice3.lockdown import create_using_usbmux
            lockdown = create_using_usbmux()
            info = {
                "DeviceName": lockdown.get_value(key="DeviceName"),
                "ProductType": lockdown.get_value(key="ProductType"),
                "ProductVersion": lockdown.get_value(key="ProductVersion"),
                "BuildVersion": lockdown.get_value(key="BuildVersion"),
                "HardwareModel": lockdown.get_value(key="HardwareModel"),
                "CPUArchitecture": lockdown.get_value(key="CPUArchitecture"),
                "UniqueChipID": lockdown.get_value(key="UniqueChipID"),
                "SerialNumber": lockdown.get_value(key="SerialNumber"),
                "WiFiAddress": lockdown.get_value(key="WiFiAddress"),
                "ActivationState": lockdown.get_value(key="ActivationState"),
                "PasswordProtected": lockdown.get_value(key="PasswordProtected"),
            }
            print("\n=== DEVICE INFO (lockdownd) ===")
            for k, v in info.items():
                print(f"  {k}: {v}")
            return info
        except Exception as e:
            print(f"  Lockdown connection failed: {e}")
            # Try without pairing
            try:
                from pymobiledevice3.lockdown import create_using_usbmux
                lockdown = create_using_usbmux(autopair=False)
                # Even without pairing, some values are readable
                info = {}
                for key in ["DeviceName", "ProductType", "ProductVersion", 
                            "BuildVersion", "UniqueChipID", "ActivationState"]:
                    try:
                        val = lockdown.get_value(key=key)
                        info[key] = val
                        print(f"  {key}: {val}")
                    except:
                        pass
                return info if info else None
            except Exception as e2:
                print(f"  Lockdown (no-pair) also failed: {e2}")
                return None
    except ImportError:
        print("pymobiledevice3 not available")
        return None

def probe_usb_raw():
    """Fallback: check for Apple USB device in normal mode."""
    try:
        import usb.core, usb.util, libusb_package, usb.backend.libusb1
        be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
        
        # Find all Apple devices
        devs = list(usb.core.find(find_all=True, idVendor=0x05AC, backend=be))
        if not devs:
            print("No Apple USB devices found")
            return
        
        print(f"\n=== RAW USB: {len(devs)} Apple device(s) ===")
        for dev in devs:
            print(f"  VID:0x{dev.idVendor:04X} PID:0x{dev.idProduct:04X}")
            # Classify PID
            pid = dev.idProduct
            if pid == 0x1227:
                print("    Mode: DFU")
            elif pid == 0x1281:
                print("    Mode: Recovery")
            elif 0x1290 <= pid <= 0x12AF:
                print("    Mode: Normal (iPhone)")
            elif 0x12A0 <= pid <= 0x12BF:
                print("    Mode: Normal (iPad)")
            else:
                print(f"    Mode: Unknown (PID=0x{pid:04X})")
            
            try:
                sn = usb.util.get_string(dev, dev.iSerialNumber)
                print(f"    Serial: {sn}")
            except:
                pass
            try:
                prod = usb.util.get_string(dev, dev.iProduct)
                print(f"    Product: {prod}")
            except:
                pass
    except Exception as e:
        print(f"USB probe error: {e}")

def wait_and_probe(max_wait=60):
    """Wait for device to appear and probe it."""
    print(f"Waiting up to {max_wait}s for iOS device to appear...")
    start = time.time()
    
    while time.time() - start < max_wait:
        elapsed = int(time.time() - start)
        
        # Check usbmux first
        try:
            from pymobiledevice3.usbmux import list_devices
            devices = list_devices()
            if devices:
                print(f"\n[{elapsed}s] Device appeared via usbmux!")
                info = probe_with_pymobiledevice3()
                probe_usb_raw()
                return info
        except:
            pass
        
        # Check raw USB
        try:
            import usb.core, libusb_package, usb.backend.libusb1
            be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
            devs = list(usb.core.find(find_all=True, idVendor=0x05AC, backend=be))
            apple_normal = [d for d in devs if d.idProduct not in (0x1227, 0x1281)]
            if apple_normal:
                print(f"\n[{elapsed}s] Apple device appeared via USB (not DFU/Recovery)")
                probe_usb_raw()
                # Give usbmux a moment to catch up
                time.sleep(3)
                info = probe_with_pymobiledevice3()
                return info
        except:
            pass
        
        if elapsed % 10 == 0:
            print(f"  [{elapsed}s] Still waiting...")
        time.sleep(2)
    
    print(f"\nTimeout after {max_wait}s — no device found")
    probe_usb_raw()
    return None

if __name__ == "__main__":
    if "--wait" in sys.argv:
        info = wait_and_probe(max_wait=90)
    else:
        print("=== Quick probe ===")
        info = probe_with_pymobiledevice3()
        probe_usb_raw()
    
    if info:
        print("\n=== SUMMARY ===")
        ios_ver = info.get("ProductVersion", "unknown")
        build = info.get("BuildVersion", "unknown")
        model = info.get("ProductType", "unknown")
        activation = info.get("ActivationState", "unknown")
        locked = info.get("PasswordProtected", "unknown")
        print(f"  Model: {model}")
        print(f"  iOS: {ios_ver} ({build})")
        print(f"  Activation: {activation}")
        print(f"  Password: {locked}")
        
        # Key question for Aether Chain applicability
        if ios_ver != "unknown":
            major, minor = ios_ver.split(".")[:2]
            if int(major) == 18 and int(minor) <= 3:
                print(f"\n  >>> iOS {ios_ver} — Aether Chain CVEs MAY apply! <<<")
            elif int(major) < 18:
                print(f"\n  >>> iOS {ios_ver} — Older kernel exploits available (kfd, etc.) <<<")
            else:
                print(f"\n  >>> iOS {ios_ver} — Need new 0days for kernel exploit <<<")
