#!/usr/bin/env python3
"""
recovery_probe.py - Probe iPhone in Recovery Mode (iBoot)
Reads device info, iBoot version, and maps to iOS version.
Recovery mode does NOT require Trust - only physical button combo.
"""

import sys
import time
import struct

# --- Method 1: pymobiledevice3 ---
def probe_pymobiledevice3():
    """Use pymobiledevice3 to talk to recovery mode device."""
    print("\n=== pymobiledevice3 Recovery Probe ===")
    try:
        from pymobiledevice3.irecv import IRecv
        print("[*] Connecting to device in Recovery mode...")
        client = IRecv()
        print("[+] Connected!")
        
        # Read all environment variables
        info_keys = [
            'CPID', 'CPRV', 'CPFM', 'SCEP', 'BDID', 'ECID',
            'IBFL', 'SRNM', 'IMEI', 'NONC', 'SNON', 'PWND',
            'build-version', 'build-style', 'firmware-version',
            'hardware-model', 'model-number', 'region-info',
            'serial-number'
        ]
        
        print("\n--- Device Info ---")
        for key in info_keys:
            try:
                val = client.getenv(key)
                if val:
                    print(f"  {key}: {val}")
            except Exception:
                pass

        # Get iBoot version from USB serial string
        try:
            srtg = client.getenv('SRTG')
            if srtg:
                print(f"\n  SRTG (iBoot): {srtg}")
        except:
            pass
        
        # Try to get the full auto-boot and other info
        for var in ['auto-boot', 'boot-command', 'com.apple.System.boot-nonce',
                     'darkboot', 'device-color', 'effective-production-status-ap',
                     'obliteration', 'recoveryos-boot-mode']:
            try:
                val = client.getenv(var)
                if val:
                    print(f"  {var}: {val}")
            except:
                pass
        
        return True
    except ImportError:
        print("[-] pymobiledevice3 not available")
        return False
    except Exception as e:
        print(f"[-] pymobiledevice3 error: {e}")
        return False


# --- Method 2: Raw USB via pyusb ---
def probe_raw_usb():
    """Use raw USB to detect and probe recovery mode device."""
    print("\n=== Raw USB Recovery Probe ===")
    try:
        import usb.core
        import usb.util
        import libusb_package
        
        backend = libusb_package.get_libusb1_backend()
        
        # Recovery mode PIDs (Apple VID = 0x05AC)
        # DFU: 0x1227, Recovery: 0x1280-0x1283 depending on device
        APPLE_VID = 0x05AC
        RECOVERY_PIDS = [0x1280, 0x1281, 0x1282, 0x1283, 0x1284, 0x1290, 0x1291]
        DFU_PID = 0x1227
        
        # First scan all Apple devices
        all_apple = list(usb.core.find(find_all=True, idVendor=APPLE_VID, backend=backend))
        print(f"[*] Apple devices found: {len(all_apple)}")
        
        for dev in all_apple:
            pid = dev.idProduct
            mode = "UNKNOWN"
            if pid == DFU_PID:
                mode = "DFU"
            elif pid in RECOVERY_PIDS:
                mode = "RECOVERY"
            elif 0x1290 <= pid <= 0x12AF:
                mode = "RECOVERY (extended)"
            elif pid >= 0x12A0:
                mode = "NORMAL/OTHER"
            
            print(f"\n  VID:PID = {APPLE_VID:04X}:{pid:04X} [{mode}]")
            
            # Try to read string descriptors
            try:
                print(f"  Manufacturer: {dev.manufacturer}")
            except:
                pass
            try:
                print(f"  Product: {dev.product}")
            except:
                pass
            try:
                serial = dev.serial_number
                print(f"  Serial: {serial}")
                # Parse serial string - in recovery mode it contains device info
                parse_recovery_serial(serial)
            except:
                pass
            
            # Read device descriptor
            print(f"  bcdDevice: {dev.bcdDevice:04X}")
            print(f"  bMaxPacketSize0: {dev.bMaxPacketSize0}")
            
            # Read config descriptors
            try:
                for cfg in dev:
                    print(f"  Config {cfg.bConfigurationValue}: {cfg.bNumInterfaces} interface(s)")
                    for intf in cfg:
                        print(f"    Interface {intf.bInterfaceNumber}: class={intf.bInterfaceClass:#04x} subclass={intf.bInterfaceSubClass:#04x} protocol={intf.bInterfaceProtocol:#04x}")
            except Exception as e:
                print(f"  Config read error: {e}")
        
        if not all_apple:
            print("[-] No Apple device detected on USB")
            # Scan ALL devices to confirm USB is working
            all_devs = list(usb.core.find(find_all=True, backend=backend))
            print(f"[*] Total USB devices: {len(all_devs)}")
            return False
        
        return True
    except Exception as e:
        print(f"[-] Raw USB error: {e}")
        return False


def parse_recovery_serial(serial):
    """Parse the USB serial string from recovery mode.
    Format: CPID:XXXX CPRV:XX CPFM:XX SCEP:XX BDID:XX ECID:XXXXXXXXXXXXXXXX IBFL:XX SRTG:[iBoot-XXXXX]
    """
    if not serial:
        return
    
    print("\n  --- Parsed Serial ---")
    parts = serial.split(' ')
    for part in parts:
        if ':' in part:
            key, val = part.split(':', 1)
            print(f"    {key}: {val}")
            
            if key == 'SRTG':
                iboot_ver = val.strip('[]')
                print(f"\n  >> iBoot Version: {iboot_ver}")
                map_iboot_to_ios(iboot_ver)


# iBoot version to iOS version mapping (approximate)
IBOOT_IOS_MAP = {
    # iOS 18.x
    'iBoot-11881': 'iOS 18.5.x',
    'iBoot-11880': 'iOS 18.5',
    'iBoot-11379': 'iOS 18.4.x',
    'iBoot-11107': 'iOS 18.3.x',
    'iBoot-11106': 'iOS 18.3.2',
    'iBoot-11105': 'iOS 18.3.1',
    'iBoot-10871': 'iOS 18.2.x',
    'iBoot-10626': 'iOS 18.1.x',
    'iBoot-10380': 'iOS 18.0.x',
    # iOS 17.x
    'iBoot-10151': 'iOS 17.7.x',
    'iBoot-9892': 'iOS 17.6.x',
    'iBoot-9651': 'iOS 17.5.x',
    'iBoot-9393': 'iOS 17.4.x',
    'iBoot-9150': 'iOS 17.3.x',
    'iBoot-8422': 'iOS 17.0-17.2.x',
    # iOS 16.x
    'iBoot-8419': 'iOS 16.7.x',
    'iBoot-7459': 'iOS 16.0-16.6.x',
    # iOS 15.x
    'iBoot-6723': 'iOS 15.x',
    'iBoot-6694': 'iOS 15.0-15.7.x',
    # iOS 14.x
    'iBoot-5540': 'iOS 14.x',
    # iOS 13.x
    'iBoot-4513': 'iOS 13.x',
    # iOS 12.x (original XR shipped with this)
    'iBoot-3865': 'iOS 12.x',
}


def map_iboot_to_ios(iboot_str):
    """Map iBoot version string to approximate iOS version."""
    # Extract the numeric part
    if 'iBoot-' in iboot_str:
        parts = iboot_str.replace('iBoot-', '').split('.')
        major = parts[0] if parts else '0'
        
        print(f"  >> Looking up iBoot major version: {major}")
        
        # Find closest match
        for iboot_key, ios_ver in sorted(IBOOT_IOS_MAP.items(), 
                                          key=lambda x: x[0], reverse=True):
            iboot_major = iboot_key.replace('iBoot-', '')
            if major >= iboot_major:
                print(f"  >> Estimated iOS Version: {ios_ver}")
                print(f"  >> (Based on iBoot {iboot_key} mapping)")
                return ios_ver
        
        print(f"  >> Unknown iBoot version, could not map to iOS")
    return None


# --- Method 3: Wait/poll mode ---
def wait_for_device(timeout=120):
    """Poll for device appearance in recovery or DFU mode."""
    print(f"\n=== Waiting for device (timeout: {timeout}s) ===")
    print("[*] Put iPhone in RECOVERY mode:")
    print("    1. Press & release Volume UP")
    print("    2. Press & release Volume DOWN")  
    print("    3. Hold SIDE button ~15s until cable+computer logo")
    print()
    
    import usb.core
    import libusb_package
    
    backend = libusb_package.get_libusb1_backend()
    start = time.time()
    
    while time.time() - start < timeout:
        devs = list(usb.core.find(find_all=True, idVendor=0x05AC, backend=backend))
        if devs:
            for dev in devs:
                pid = dev.idProduct
                if pid == 0x1227:
                    print(f"\n[!] Device detected in DFU mode (PID:{pid:04X})")
                    print("    This is DFU, not Recovery. For Recovery:")
                    print("    - Force restart (Vol+, Vol-, hold Side)")
                    print("    - Then at the Apple logo, hold Vol Down")
                elif 0x1280 <= pid <= 0x12AF:
                    print(f"\n[+] Device detected in RECOVERY mode! (PID:{pid:04X})")
                    return True
                else:
                    print(f"\n[?] Apple device detected PID:{pid:04X}")
                    return True
        
        elapsed = int(time.time() - start)
        print(f"\r[*] Scanning... {elapsed}s", end='', flush=True)
        time.sleep(2)
    
    print(f"\n[-] Timeout after {timeout}s - no device detected")
    return False


def main():
    print("=" * 60)
    print("  iPhone Recovery Mode Probe")
    print("  Target: iPhone XR (A12 T8020)")
    print("=" * 60)
    
    mode = '--wait' in sys.argv
    
    if mode:
        if wait_for_device(timeout=120):
            print()
        else:
            return
    
    # Try raw USB first (always works)
    usb_ok = probe_raw_usb()
    
    # Then try pymobiledevice3 for richer info
    if usb_ok:
        probe_pymobiledevice3()
    
    print("\n" + "=" * 60)
    if usb_ok:
        print("  Device detected! Check iBoot version above.")
        print("  DFU SecureROM: iBoot-3865.0.0.4.7 (iOS 12 era)")
        print("  Recovery iBoot will show the INSTALLED iOS version.")
    else:
        print("  No device detected.")
        print("  Make sure iPhone is in Recovery mode (cable+computer icon)")
        print("  Try: python recovery_probe.py --wait")
    print("=" * 60)


if __name__ == '__main__':
    main()
