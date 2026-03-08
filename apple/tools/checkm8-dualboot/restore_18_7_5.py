#!/usr/bin/env python3
"""
restore_18_7_5.py - Restore iPhone XR to iOS 18.7.5 from Recovery Mode
Target: iPhone11,8 (A12 T8020)

iOS 18.7.5 (22H311) is the latest signed version for iPhone XR.
It is vulnerable to CVE-2026-20700 (memory corruption, CISA KEV, exploited ITW).

This script:
1. Downloads the IPSW if not present
2. Initiates restore via pymobiledevice3
"""

import os
import sys
import time
import hashlib
import subprocess
import urllib.request

# iPhone XR iOS 18.7.5 IPSW info
IPSW_URL = "https://updates.cdn-apple.com/2026WinterFCS/fullrestores/047-53269/25427FD8-2F38-4E14-88D6-AA5BA5FE340B/iPhone11,8_18.7.5_22H311_Restore.ipsw"
IPSW_FILENAME = "iPhone11,8_18.7.5_22H311_Restore.ipsw"
IPSW_BUILD = "22H311"
IPSW_VERSION = "18.7.5"

# Download directory
DOWNLOAD_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ipsw")


def check_device():
    """Check if device is in recovery mode."""
    print("[*] Checking for device in Recovery mode...")
    try:
        import usb.core
        import libusb_package
        backend = libusb_package.get_libusb1_backend()
        dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=backend)
        if dev:
            print(f"[+] Device in Recovery mode (PID:1281)")
            try:
                print(f"    Serial: {dev.serial_number}")
            except:
                pass
            return True
        
        # Check DFU too
        dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=backend)
        if dev:
            print("[!] Device is in DFU mode, not Recovery.")
            print("    Restore is possible from DFU too.")
            return True
        
        print("[-] No Apple device detected")
        return False
    except Exception as e:
        print(f"[-] USB check error: {e}")
        return False


def download_ipsw():
    """Download the IPSW file if not present."""
    os.makedirs(DOWNLOAD_DIR, exist_ok=True)
    ipsw_path = os.path.join(DOWNLOAD_DIR, IPSW_FILENAME)
    
    if os.path.exists(ipsw_path):
        size_gb = os.path.getsize(ipsw_path) / (1024**3)
        if size_gb > 5:  # IPSW should be > 5GB
            print(f"[+] IPSW already downloaded: {ipsw_path} ({size_gb:.1f} GB)")
            return ipsw_path
        else:
            print(f"[!] IPSW file incomplete ({size_gb:.1f} GB), re-downloading...")
            os.remove(ipsw_path)
    
    print(f"[*] Downloading iOS {IPSW_VERSION} IPSW ({IPSW_BUILD})...")
    print(f"    URL: {IPSW_URL}")
    print(f"    Destination: {ipsw_path}")
    print(f"    This is ~8 GB, it will take a while...\n")
    
    try:
        # Use urllib with progress reporting
        def progress_hook(block_num, block_size, total_size):
            downloaded = block_num * block_size
            if total_size > 0:
                pct = min(downloaded / total_size * 100, 100)
                downloaded_gb = downloaded / (1024**3)
                total_gb = total_size / (1024**3)
                bar_len = 40
                filled = int(bar_len * pct / 100)
                bar = '=' * filled + '-' * (bar_len - filled)
                print(f"\r    [{bar}] {pct:.1f}% ({downloaded_gb:.2f}/{total_gb:.2f} GB)", end='', flush=True)
        
        urllib.request.urlretrieve(IPSW_URL, ipsw_path, reporthook=progress_hook)
        print(f"\n[+] Download complete: {ipsw_path}")
        return ipsw_path
        
    except KeyboardInterrupt:
        print("\n[!] Download interrupted")
        if os.path.exists(ipsw_path):
            os.remove(ipsw_path)
        return None
    except Exception as e:
        print(f"\n[-] Download error: {e}")
        # Try with curl as fallback
        print("[*] Trying with curl...")
        try:
            subprocess.run([
                'curl', '-L', '-o', ipsw_path, '--progress-bar', IPSW_URL
            ], check=True)
            return ipsw_path
        except Exception as e2:
            print(f"[-] Curl also failed: {e2}")
            return None


def restore_with_pymobiledevice3(ipsw_path):
    """Restore using pymobiledevice3's restore module."""
    print(f"\n[*] Starting restore with pymobiledevice3...")
    print(f"    IPSW: {ipsw_path}")
    print(f"    Target: iOS {IPSW_VERSION} ({IPSW_BUILD})")
    print()
    
    # Try CLI first (more reliable)
    venv_bin = os.path.join(os.path.dirname(os.path.abspath(__file__)), 
                            '..', '..', '.venv', 'Scripts')
    pmd3_cli = os.path.join(venv_bin, 'pymobiledevice3.exe')
    
    if os.path.exists(pmd3_cli):
        print(f"[*] Using pymobiledevice3 CLI: {pmd3_cli}")
        cmd = [pmd3_cli, 'restore', 'update', ipsw_path]
        print(f"    Command: {' '.join(cmd)}")
        print()
        print("=" * 60)
        print("  RESTORE IN PROGRESS - DO NOT DISCONNECT!")
        print("=" * 60)
        print()
        
        try:
            proc = subprocess.run(cmd, timeout=3600)  # 1 hour timeout
            if proc.returncode == 0:
                print("\n[+] Restore completed successfully!")
                return True
            else:
                print(f"\n[-] Restore failed with code {proc.returncode}")
                print("[*] Trying Python API fallback...")
        except subprocess.TimeoutExpired:
            print("\n[-] Restore timed out after 1 hour")
            return False
        except Exception as e:
            print(f"\n[-] CLI error: {e}")
            print("[*] Trying Python API fallback...")
    
    # Fallback: Python API
    try:
        from pymobiledevice3.restore.restore import Restore
        print("[*] Using pymobiledevice3 Python API...")
        print()
        print("=" * 60)
        print("  RESTORE IN PROGRESS - DO NOT DISCONNECT!")
        print("=" * 60)
        print()
        
        restore = Restore(ipsw_path)
        restore.update()
        
        print("\n[+] Restore completed successfully!")
        return True
        
    except Exception as e:
        print(f"\n[-] Restore API error: {e}")
        return False


def main():
    print("=" * 60)
    print(f"  iPhone XR Restore to iOS {IPSW_VERSION}")
    print(f"  Build: {IPSW_BUILD}")
    print(f"  Purpose: Exploit CVE-2026-20700 (kernel memory corruption)")
    print("=" * 60)
    print()
    
    # Step 1: Check device
    if not check_device():
        print("\n[!] Put your iPhone XR in Recovery mode first:")
        print("    1. Plug in USB cable")
        print("    2. Press & release Volume UP")
        print("    3. Press & release Volume DOWN")
        print("    4. Hold SIDE button ~15s until cable+computer icon")
        return 1
    
    # Step 2: Download IPSW
    print()
    ipsw_path = download_ipsw()
    if not ipsw_path:
        print("\n[-] Cannot proceed without IPSW")
        return 1
    
    # Step 3: Confirm
    print()
    print("=" * 60)
    print("  WARNING: This will ERASE ALL DATA on the iPhone!")
    print("  The device will be restored to a fresh iOS 18.7.5")
    print("=" * 60)
    
    if '--yes' not in sys.argv:
        resp = input("\n  Continue? [y/N] > ").strip().lower()
        if resp != 'y':
            print("  Aborted.")
            return 0
    
    # Step 4: Restore
    success = restore_with_pymobiledevice3(ipsw_path)
    
    if success:
        print()
        print("=" * 60)
        print("  RESTORE SUCCESSFUL!")
        print(f"  iOS Version: {IPSW_VERSION} ({IPSW_BUILD})")
        print("  Next steps:")
        print("    1. Device will reboot and show setup screen")
        print("    2. We can probe it via USB for exploit research")
        print("    3. CVE-2026-20700 affects this version")
        print("=" * 60)
        return 0
    else:
        print()
        print("=" * 60)
        print("  RESTORE FAILED")
        print("  Try using iTunes/Apple Devices app instead:")
        print("  1. Install 'Apple Devices' from Microsoft Store")
        print("  2. Open it with iPhone in Recovery mode")
        print("  3. Click 'Restore iPhone'")
        print("=" * 60)
        return 1


if __name__ == '__main__':
    sys.exit(main())
