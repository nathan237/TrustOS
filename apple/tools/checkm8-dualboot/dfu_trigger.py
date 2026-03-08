#!/usr/bin/env python3
"""Check DFU status and try to trigger iBSS execution after send."""
import usb.core, usb.util, usb.backend.libusb1
import time, sys

try:
    import libusb_package
    be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
except:
    be = None

dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
if not dev:
    print("No DFU device found")
    dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
    if dev:
        print(f"Already in Recovery! PID=0x{dev.idProduct:04X}")
    sys.exit(1)

print(f"DFU device found (PID=0x{dev.idProduct:04X})")
try:
    print(f"  Serial: {dev.serial_number[:80]}")
except:
    pass

try:
    dev.set_configuration()
except:
    pass
try:
    usb.util.claim_interface(dev, 0)
except:
    pass

# Check DFU status
print("\n=== DFU STATUS ===")
try:
    status = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
    states = {0:"appIDLE", 1:"appDETACH", 2:"dfuIDLE", 3:"dfuDNLOAD-SYNC",
              4:"dfuDNBUSY", 5:"dfuDNLOAD-IDLE", 6:"dfuMANIFEST-SYNC",
              7:"dfuMANIFEST", 8:"dfuMANIFEST-WAIT-RESET", 9:"dfuUPLOAD-IDLE",
              10:"dfuERROR"}
    bStatus = status[0]
    bwPollTimeout = status[1] | (status[2] << 8) | (status[3] << 16)
    bState = status[4]
    iString = status[5]
    state_name = states.get(bState, f"UNKNOWN({bState})")
    print(f"  bStatus: {bStatus} ({'OK' if bStatus==0 else 'ERROR'})")
    print(f"  bwPollTimeout: {bwPollTimeout}ms")
    print(f"  bState: {bState} ({state_name})")
    print(f"  iString: {iString}")
except Exception as e:
    print(f"  Status read: {e}")

# Try approach 1: re-send iBSS fully via IRecv
print("\n=== RE-SENDING iBSS (fresh IRecv) ===")
try:
    from pymobiledevice3.irecv import IRecv
    irecv = IRecv()
    print(f"  Mode: {irecv.mode}")
    
    with open("tools/checkm8-dualboot/cache/ibss_signed.img4", "rb") as f:
        data = f.read()
    
    print(f"  Sending {len(data)} bytes...")
    irecv.send_buffer(data)
    print("  send_buffer done!")
    
    # Check status
    time.sleep(1)
    try:
        st = irecv.ctrl_transfer(0xA1, 3, data_or_wLength=6)
        print(f"  Post-send status: state={st[4]}")
    except:
        print("  Post-send status read failed (device may have disconnected)")
    
except Exception as e:
    print(f"  IRecv: {e}")
    import traceback
    traceback.print_exc()

# Wait and check
print("\nWaiting for Recovery mode...")
time.sleep(5)

for attempt in range(20):
    for pid, name in [(0x1281, "RECOVERY"), (0x1227, "DFU")]:
        d = usb.core.find(idVendor=0x05AC, idProduct=pid, backend=be)
        if d:
            if name == "RECOVERY":
                print(f"\n*** {name} found! (PID=0x{pid:04X}) ***")
                try:
                    print(f"  Product: {d.product}")
                    print(f"  Serial: {d.serial_number[:80]}")
                except:
                    pass
                sys.exit(0)
            else:
                if attempt == 0:
                    print(f"  Still in {name}")
                    # Try USB reset to trigger execution
                    print("  Attempting USB reset to trigger execution...")
                    try:
                        d.reset()
                    except:
                        pass
                    time.sleep(3)
                    continue
    time.sleep(1)
    if attempt % 5 == 4:
        print(f"  {attempt+1}s...")

print("\nDevice did not enter Recovery. Trying manual DFU clear-status + abort...")
dev2 = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
if dev2:
    # DFU_CLRSTATUS
    try:
        dev2.ctrl_transfer(0x21, 4, 0, 0, timeout=1000)
        print("  CLRSTATUS sent")
    except Exception as e:
        print(f"  CLRSTATUS: {e}")
    # DFU_ABORT
    try:
        dev2.ctrl_transfer(0x21, 6, 0, 0, timeout=1000)
        print("  ABORT sent")
    except Exception as e:
        print(f"  ABORT: {e}")
    
    time.sleep(2)
    # Check again
    try:
        st = dev2.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
        print(f"  State now: {st[4]}")
    except:
        pass

print("\nDone.")
