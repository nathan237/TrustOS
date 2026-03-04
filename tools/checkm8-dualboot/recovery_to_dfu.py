#!/usr/bin/env python3
"""
recovery_to_dfu.py — Push iPhone from Recovery mode to DFU mode via USB
 
Method 1: Send iBoot commands via USB control transfers
Method 2: USB reset / stall technique  
Method 3: Guided interactive mode with real-time USB monitoring
"""

import sys
import time
import struct
import libusb_package
import usb.core
import usb.util
import usb.backend.libusb1

# Initialize libusb backend
_backend = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

APPLE_VID  = 0x05AC
RECOV_PID  = 0x1281
DFU_PID    = 0x1227
NORMAL_PID = 0x12A8

# iBoot USB command interface
IBOOT_REQ_TYPE   = 0x40   # USB_DIR_OUT | USB_TYPE_VENDOR | USB_RECIP_DEVICE
IBOOT_REQ_CMD    = 0x00   # Send command
IBOOT_REQ_STATUS = 0x03   # Get status
IBOOT_REQ_TYPE_IN = 0xC0  # USB_DIR_IN | USB_TYPE_VENDOR | USB_RECIP_DEVICE


def find_device(pid):
    """Find Apple device with given PID."""
    return usb.core.find(idVendor=APPLE_VID, idProduct=pid, backend=_backend)


def send_iboot_command(dev, cmd):
    """Send a text command to iBoot Recovery mode via USB control transfer."""
    cmd_bytes = cmd.encode('ascii') + b'\x00'
    try:
        ret = dev.ctrl_transfer(
            IBOOT_REQ_TYPE,  # bmRequestType
            IBOOT_REQ_CMD,   # bRequest
            0,               # wValue
            0,               # wIndex
            cmd_bytes,        # data
            5000             # timeout ms
        )
        print(f"  [OK] Sent '{cmd}' → {ret} bytes accepted")
        return True
    except usb.core.USBError as e:
        print(f"  [ERR] Sending '{cmd}': {e}")
        return False


def get_iboot_info(dev):
    """Read iBoot serial/info string."""
    try:
        # Try to read the serial number string descriptor
        serial = usb.util.get_string(dev, dev.iSerialNumber)
        return serial
    except Exception:
        return None


def get_iboot_response(dev, size=512):
    """Read response from iBoot."""
    try:
        data = dev.ctrl_transfer(
            IBOOT_REQ_TYPE_IN,
            IBOOT_REQ_CMD,
            0, 0,
            size,
            5000
        )
        return bytes(data)
    except usb.core.USBError:
        return None


def wait_for_device(pid, timeout=15):
    """Wait for device with given PID to appear on USB."""
    print(f"  Waiting for 0x{pid:04X}...", end='', flush=True)
    start = time.time()
    while time.time() - start < timeout:
        dev = find_device(pid)
        if dev:
            print(f" FOUND! ({time.time()-start:.1f}s)")
            return dev
        time.sleep(0.5)
        print(".", end='', flush=True)
    print(f" timeout ({timeout}s)")
    return None


def method1_iboot_reboot(dev):
    """
    Method 1: Use iBoot commands to reboot the device.
    After reboot, if we interrupt early, it may land in DFU.
    """
    print("\n[METHOD 1] iBoot command reboot technique")
    print("="*50)
    
    # Get device info
    info = get_iboot_info(dev)
    if info:
        print(f"  iBoot info: {info[:200]}")
    
    # Try various commands
    print("\n  Sending 'setenv auto-boot false'...")
    send_iboot_command(dev, "setenv auto-boot false")
    time.sleep(0.3)
    
    print("  Sending 'saveenv'...")
    send_iboot_command(dev, "saveenv")
    time.sleep(0.3)
    
    # Now try to reboot - device should come back in Recovery/DFU
    print("  Sending 'reboot'...")
    send_iboot_command(dev, "reboot")
    
    print("\n  Device should reboot... scanning for DFU...")
    time.sleep(2)
    
    dfu = wait_for_device(DFU_PID, timeout=15)
    if dfu:
        return dfu
    
    # Check if it came back in Recovery
    recov = find_device(RECOV_PID)
    if recov:
        print("  Device came back in Recovery mode (not DFU)")
        return None
    
    return None


def method2_usb_stall(dev):
    """
    Method 2: Trigger USB stall/reset to potentially crash iBoot into DFU.
    Sends malformed transfers to try to crash the USB stack.
    """
    print("\n[METHOD 2] USB stall/reset technique")
    print("="*50)
    
    try:
        # Send oversized control transfer
        print("  Sending oversized control request...")
        try:
            dev.ctrl_transfer(0x21, 1, 0, 0, b'\x00' * 2048, 1000)
        except usb.core.USBError:
            pass
        
        # Send invalid request types
        print("  Sending invalid request types...")
        for req in [0x01, 0x02, 0x04, 0xFF]:
            try:
                dev.ctrl_transfer(0x40, req, 0, 0, b'', 500)
            except usb.core.USBError:
                pass
        
        # Try DFU class requests on Recovery device
        print("  Sending DFU class requests...")
        # DFU_DNLOAD
        try:
            dev.ctrl_transfer(0x21, 1, 0, 0, b'\x00' * 256, 500)
        except usb.core.USBError:
            pass
        # DFU_ABORT
        try:
            dev.ctrl_transfer(0x21, 6, 0, 0, None, 500)
        except usb.core.USBError:
            pass
        
        # USB reset
        print("  Sending USB reset...")
        try:
            dev.reset()
        except usb.core.USBError:
            pass
        
        time.sleep(3)
        
        dfu = wait_for_device(DFU_PID, timeout=10)
        if dfu:
            return dfu
            
    except Exception as e:
        print(f"  Error: {e}")
    
    return None


def method3_heap_overflow_crash(dev):
    """
    Method 3: Try to crash iBoot by exploiting known USB heap issues.
    This sends carefully crafted transfers that may trigger a crash,
    landing the device in DFU.
    """
    print("\n[METHOD 3] iBoot USB heap crash technique")
    print("="*50)
    
    try:
        # Technique: send multiple large serial number requests
        # Known to sometimes crash iBoot on some versions
        print("  Sending rapid large descriptor requests...")
        for i in range(50):
            try:
                dev.ctrl_transfer(0x80, 6, 0x0300 | 0xFF, 0, 0xFF, 100)
            except usb.core.USBError:
                pass
        
        # Send large vendor-specific requests
        print("  Sending large vendor requests...")
        for i in range(20):
            try:
                dev.ctrl_transfer(0xC0, 0, 0, 0, 0x10000, 100)
            except usb.core.USBError:
                pass
        
        # Try to trigger USB reset via control transfer abuse
        print("  Triggering USB stack stress...")
        big_data = b'\x41' * 4096
        for i in range(10):
            try:
                dev.ctrl_transfer(0x40, 0, 0, 0, big_data, 200)
            except usb.core.USBError:
                pass
            try:
                dev.ctrl_transfer(0x21, 1, i, 0, big_data, 200)
            except usb.core.USBError:
                pass
        
        print("  Checking result...")
        time.sleep(3)
        
        dfu = wait_for_device(DFU_PID, timeout=10)
        if dfu:
            return dfu
            
    except usb.core.USBError as e:
        print(f"  Device disconnected (possibly crashed): {e}")
        time.sleep(3)
        dfu = wait_for_device(DFU_PID, timeout=15)
        if dfu:
            return dfu
    
    return None


def method4_guided_interactive():
    """
    Method 4: Real-time guided DFU entry with USB monitoring.
    Watches USB bus and tells user exactly what to do.
    """
    print("\n[METHOD 4] Guided interactive DFU entry")
    print("="*50)
    print()
    print("  L'iPhone est en Recovery. Pour entrer en DFU :")
    print()
    print("  ┌─────────────────────────────────────────────┐")
    print("  │ ETAPE 1: Maintiens LATERAL + VOL BAS        │")
    print("  │          ensemble pendant 10 secondes        │")
    print("  │                                              │")
    print("  │ ETAPE 2: Relache LATERAL uniquement          │")
    print("  │          mais garde VOL BAS enfonce          │")
    print("  │          pendant encore 5 secondes           │")
    print("  │                                              │")
    print("  │ RESULTAT: Ecran NOIR complet = DFU OK        │")
    print("  │           Logo Apple = rate, recommence      │")
    print("  └─────────────────────────────────────────────┘")
    print()
    
    input("  Appuie ENTREE quand tu es pret, puis fais les boutons...")
    print()
    print("  Monitoring USB en temps reel...")
    print("  ─────────────────────────────")
    
    start = time.time()
    last_state = "recovery"
    found_dfu = False
    
    while time.time() - start < 30:
        elapsed = time.time() - start
        
        dfu = find_device(DFU_PID)
        recov = find_device(RECOV_PID)
        normal = find_device(NORMAL_PID)
        
        if dfu:
            print(f"\r  [{elapsed:5.1f}s] ★★★ DFU DETECTE (0x1227) ★★★                    ")
            found_dfu = True
            return dfu
        elif recov:
            if last_state != "recovery":
                print(f"\r  [{elapsed:5.1f}s] Recovery mode (0x1281) — toujours Recovery...   ", end='', flush=True)
                last_state = "recovery"
            else:
                print(f"\r  [{elapsed:5.1f}s] Recovery mode — appuie LATERAL + VOL BAS...     ", end='', flush=True)
        elif normal:
            print(f"\r  [{elapsed:5.1f}s] Mode normal (0x12A8) — l'iPhone a reboote         ", end='', flush=True)
            last_state = "normal"
        else:
            if last_state != "disconnected":
                print(f"\r  [{elapsed:5.1f}s] Deconnecte — potentiellement en transition...   ", end='', flush=True)
                last_state = "disconnected"
            else:
                print(f"\r  [{elapsed:5.1f}s] En attente...                                  ", end='', flush=True)
        
        time.sleep(0.3)
    
    print(f"\n\n  Timeout apres 30s. DFU non detecte.")
    return None


def main():
    print("╔══════════════════════════════════════════════╗")
    print("║  Recovery → DFU Push Tool                   ║")
    print("║  iPhone 11 Pro (A13/T8030)                  ║")
    print("╚══════════════════════════════════════════════╝")
    print()
    
    # Check current state
    dfu = find_device(DFU_PID)
    if dfu:
        print("[!] Device ALREADY in DFU mode! Ready for checkm8.")
        sys.exit(0)
    
    recov = find_device(RECOV_PID)
    if not recov:
        normal = find_device(NORMAL_PID)
        if normal:
            print("[!] Device in Normal mode (0x12A8), not Recovery.")
            print("    Need Recovery mode first. Put into Recovery:")
            print("    1. Press Vol Up then release")
            print("    2. Press Vol Down then release")
            print("    3. Hold Side button until Recovery screen")
            sys.exit(1)
        else:
            print("[!] No Apple device found on USB.")
            sys.exit(1)
    
    print(f"[OK] iPhone detected in Recovery mode (0x1281)")
    
    # Get iBoot info
    info = get_iboot_info(recov)
    if info:
        print(f"[INFO] iBoot: {info[:100]}")
    
    # Parse command line
    method = None
    if len(sys.argv) > 1:
        method = sys.argv[1]
    
    if method == "--method1" or method == "-1":
        result = method1_iboot_reboot(recov)
    elif method == "--method2" or method == "-2":
        result = method2_usb_stall(recov)
    elif method == "--method3" or method == "-3":
        result = method3_heap_overflow_crash(recov)
    elif method == "--method4" or method == "-4":
        result = method4_guided_interactive()
    elif method == "--all":
        # Try all automated methods, then guided
        result = None
        
        # Re-find device for each attempt
        recov = find_device(RECOV_PID)
        if recov:
            result = method1_iboot_reboot(recov)
        
        if not result:
            recov = find_device(RECOV_PID)
            if recov:
                result = method2_usb_stall(recov)
        
        if not result:
            recov = find_device(RECOV_PID)
            if recov:
                result = method3_heap_overflow_crash(recov)
        
        if not result:
            print("\n[!] Automated methods failed. Trying guided mode...")
            result = method4_guided_interactive()
    else:
        # Default: try automated methods first, silently
        print("\nTrying automated DFU push methods...")
        result = None
        
        # Method 1: iBoot reboot
        try:
            result = method1_iboot_reboot(recov)
        except Exception as e:
            print(f"  Method 1 error: {e}")
        
        if not result:
            recov = find_device(RECOV_PID)
            if recov:
                try:
                    result = method2_usb_stall(recov)
                except Exception as e:
                    print(f"  Method 2 error: {e}")
        
        if not result:
            recov = find_device(RECOV_PID)
            if recov:
                try:
                    result = method3_heap_overflow_crash(recov)
                except Exception as e:
                    print(f"  Method 3 error: {e}")
        
        if not result:
            print("\n[!] Automated push failed. Switching to guided mode...")
            result = method4_guided_interactive()
    
    if result:
        print("\n" + "="*50)
        print(" SUCCESS — iPhone is now in DFU mode (0x1227)")
        print(" Ready for checkm8 exploit!")
        print("="*50)
        
        # Write result for live_test.py to pick up
        try:
            import json
            with open("results/dfu_entry_result.json", "w") as f:
                json.dump({
                    "status": "DFU_READY",
                    "pid": "0x1227",
                    "timestamp": time.time(),
                    "message": "iPhone in DFU mode, ready for checkm8"
                }, f, indent=2)
        except:
            pass
        
        sys.exit(0)
    else:
        print("\n" + "="*50)
        print(" FAILED — Could not enter DFU mode")
        print(" Tips:")
        print("   - Ensure iPhone is in Recovery (cable + iTunes icon)")
        print("   - Try the button sequence manually")
        print("   - Use: python recovery_to_dfu.py --method4")
        print("="*50)
        sys.exit(1)


if __name__ == "__main__":
    main()
