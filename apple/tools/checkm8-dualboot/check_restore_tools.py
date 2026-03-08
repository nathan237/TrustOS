#!/usr/bin/env python3
"""Quick check of restore tools and start download."""
import os, sys, shutil, subprocess

# Check pymobiledevice3 CLI
venv = os.path.join(os.path.dirname(__file__), '..', '..', '.venv', 'Scripts')
pmd3 = os.path.join(venv, 'pymobiledevice3.exe')
print(f"pymobiledevice3 CLI exists: {os.path.exists(pmd3)}")

if os.path.exists(pmd3):
    try:
        r = subprocess.run([pmd3, 'restore', '--help'], capture_output=True, text=True, timeout=10)
        # Just show first 5 lines
        lines = r.stdout.strip().split('\n')[:5] if r.stdout else []
        for l in lines:
            print(f"  {l}")
        if r.stderr:
            err_lines = r.stderr.strip().split('\n')[:3]
            for l in err_lines:
                print(f"  ERR: {l}")
    except Exception as e:
        print(f"  Error running: {e}")

# Check device  
try:
    import usb.core, libusb_package
    backend = libusb_package.get_libusb1_backend()
    dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=backend)
    print(f"\nDevice in Recovery: {'YES' if dev else 'NO'}")
except Exception as e:
    print(f"\nDevice check error: {e}")

# Check disk space for IPSW download (~8.7 GB)
import ctypes
free_bytes = ctypes.c_ulonglong(0)
ctypes.windll.kernel32.GetDiskFreeSpaceExW(
    os.path.dirname(__file__), None, None, ctypes.pointer(free_bytes))
free_gb = free_bytes.value / (1024**3)
print(f"Free disk space: {free_gb:.1f} GB (need ~9 GB for IPSW)")

if free_gb < 10:
    print("WARNING: Low disk space!")
