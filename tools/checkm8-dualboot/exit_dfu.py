#!/usr/bin/env python3
"""
Force iPhone out of DFU mode via USB commands.
Sends empty DFU_DNLOAD (0 bytes) which triggers manifest → reboot.
"""
import sys, time
try:
    import usb.core, usb.util
    import libusb_package
except ImportError:
    print("pip install pyusb libusb-package")
    sys.exit(1)

dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227)
if not dev:
    print("No DFU device found")
    sys.exit(1)

print(f"[*] Found DFU device: {usb.util.get_string(dev, dev.iSerialNumber)[:60]}")

# Method 1: Empty DNLOAD → triggers manifest → reboot
print("[*] Sending empty DFU_DNLOAD (0 bytes) to trigger reboot...")
try:
    dev.ctrl_transfer(0x21, 1, 0, 0, b'', timeout=5000)  # DFU_DNLOAD, wLength=0
    print("[+] DNLOAD sent")
except Exception as e:
    print(f"[-] DNLOAD: {e}")

time.sleep(0.3)

# Get status to advance state machine
print("[*] Sending DFU_GETSTATUS to advance state machine...")
for i in range(5):
    try:
        data = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        state = data[4]
        states = {2:"dfuIDLE", 3:"dfuDNLOAD-SYNC", 5:"dfuDNLOAD-IDLE", 
                  6:"dfuMANIFEST-SYNC", 7:"dfuMANIFEST", 8:"dfuMANIFEST-WAIT-RESET", 10:"dfuERROR"}
        print(f"[+] State: {state} ({states.get(state, '?')})")
        if state in (7, 8):  # MANIFEST or MANIFEST-WAIT-RESET
            print("[+] Device is rebooting!")
            break
        if state == 10:  # ERROR
            print("[*] Sending DFU_CLRSTATUS...")
            dev.ctrl_transfer(0x21, 4, 0, 0, b'', timeout=1000)
    except Exception as e:
        print(f"[+] Device disconnected — rebooting! ({e})")
        break
    time.sleep(0.5)

# Method 2: USB reset
print("[*] Sending USB bus reset...")
try:
    dev.reset()
    print("[+] USB reset sent")
except:
    print("[+] Device gone — reboot in progress")

print("\n[*] Wait 10-15 seconds. Phone should show Apple logo then boot normally.")
print("[*] To enter Recovery: as soon as you see Apple logo, hold VOLUME DOWN")
