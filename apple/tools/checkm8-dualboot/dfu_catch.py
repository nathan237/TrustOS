#!/usr/bin/env python3
"""
Auto-DFU catcher: waits for DFU device, runs TOCTOU v3 immediately.
Run this BEFORE entering DFU mode.
"""
import subprocess, sys, time, os
from datetime import datetime

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S')}] {msg}", flush=True)

def kill_apple():
    """Kill Apple services that steal DFU device"""
    import ctypes
    procs = ["iTunesHelper.exe","iTunes.exe","AppleMobileDeviceService.exe",
             "usbmuxd.exe","AMPDeviceDiscoveryAgent.exe","AppleMobileDeviceHelper.exe"]
    for p in procs:
        os.system(f'taskkill /F /IM "{p}" >nul 2>&1')
    os.system('net stop "Apple Mobile Device Service" >nul 2>&1')
    os.system('net stop "AppleMobileDeviceService" >nul 2>&1')

def wait_dfu():
    """Wait for DFU device to appear"""
    import usb.core, libusb_package, usb.backend.libusb1
    be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
    
    log("Waiting for DFU device (VID=05AC PID=1227)...")
    log("Enter DFU mode now!")
    
    while True:
        dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
        if dev:
            return True
        time.sleep(0.2)

def main():
    log("="*50)
    log("DFU Auto-Catcher + TOCTOU v3")
    log("="*50)
    
    # Step 1: Kill Apple services preemptively
    log("Killing Apple services...")
    kill_apple()
    time.sleep(0.5)
    
    # Step 2: Wait for DFU
    wait_dfu()
    log("DFU device detected!")
    
    # Step 3: Kill Apple services AGAIN (they may have restarted)
    kill_apple()
    time.sleep(0.3)
    
    # Step 4: Run TOCTOU v3
    log("Launching TOCTOU v3...")
    script = os.path.join(os.path.dirname(__file__), "toctou_v3.py")
    os.execv(sys.executable, [sys.executable, script])

if __name__ == "__main__":
    main()
