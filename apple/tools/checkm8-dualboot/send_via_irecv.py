#!/usr/bin/env python3
"""Use pymobiledevice3's IRecv to send signed iBSS properly."""
import plistlib, time
from pymobiledevice3.restore.img4 import IMG4, IM4P
from pymobiledevice3.restore.tss import TSSResponse

# Verify IMG4
print("Verifying IMG4...")
with open('tools/checkm8-dualboot/cache/ibss_signed.img4', 'rb') as f:
    data = f.read()

img4 = IMG4(data=data)
print(f"  IM4P fourcc: {img4.im4p.fourcc}")
print(f"  IM4P description: {img4.im4p.description}")
print(f"  IM4P payload: {len(img4.im4p.payload)} bytes")
print(f"  IM4M present: {img4.im4m is not None}")
print(f"  IM4R present: {img4.im4r is not None}")
out = img4.output()
print(f"  Roundtrip: {len(out)} bytes, match={out == data}")

# Now try using IRecv  
from pymobiledevice3.irecv import IRecv, Mode
import usb.core
import libusb_package

print("\nConnecting via IRecv...")
try:
    irecv = IRecv()
    print(f"  Mode: {irecv.mode}")
    print(f"  ECID: {irecv.ecid:#x}")
    print(f"  ApNonce: {irecv.ap_nonce.hex() if irecv.ap_nonce else 'None'}")
    
    # Reset device if not in idle state
    state = irecv.ctrl_transfer(0xA1, 5, data_or_wLength=1)[0]
    print(f"  State: {state}")
    if state != 2:
        print(f"  Not in dfuIDLE ({state}), resetting...")
        try:
            irecv._device.reset()
        except:
            pass
        time.sleep(3)
        # Reconnect
        irecv = IRecv()
        state = irecv.ctrl_transfer(0xA1, 5, data_or_wLength=1)[0]
        print(f"  State after reset: {state}")
    
    # Send the IMG4 data
    print(f"\nSending {len(data)} bytes via IRecv.send_buffer()...")
    irecv.send_buffer(data)
    print("  send_buffer completed!")
    
except Exception as e:
    print(f"  Error: {e}")
    import traceback
    traceback.print_exc()

# Check for recovery mode
print("\nChecking for Recovery mode...")
time.sleep(5)
try:
    dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281,
                         backend=libusb_package.get_libusb1_backend())
    if dev:
        print("*** RECOVERY MODE FOUND! ***")
    else:
        print("No recovery device")
        dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227,
                             backend=libusb_package.get_libusb1_backend())
        if dev:
            print("Still in DFU")
        else:
            print("Device disconnected")
except Exception as e:
    print(f"Error: {e}")
