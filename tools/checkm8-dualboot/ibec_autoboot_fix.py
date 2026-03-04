#!/usr/bin/env python3
"""
Full pipeline: DFU -> iBSS -> set auto-boot=false -> iBEC -> interactive console

PREREQUISITES:
  1. Device must be in DFU mode (PID=0x1227)
  2. Cached files: ibss_signed.img4, ibec_signed.img4 (or components to stitch)
  
The key fix: set auto-boot=false in iBSS before sending iBEC,
so iBEC enters interactive mode instead of trying to auto-boot.
"""
import usb.core, usb.util, usb.backend.libusb1
import time, os, sys, plistlib

try:
    import libusb_package
    be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
except:
    be = None

CACHE = "tools/checkm8-dualboot/cache"

def find_device(pid, timeout=10):
    """Wait for a device with given PID."""
    for i in range(timeout * 2):
        dev = usb.core.find(idVendor=0x05AC, idProduct=pid, backend=be)
        if dev:
            return dev
        time.sleep(0.5)
    return None

def send_cmd(dev, cmd):
    """Send command to iBSS/iBEC via ctrl transfer. Returns (response, ok)."""
    try:
        dev.ctrl_transfer(0x40, 0, 0, 0, cmd.encode() + b"\x00", timeout=5000)
    except usb.core.USBError as e:
        return f"STALL: {e}", False
    try:
        resp = dev.ctrl_transfer(0xC0, 0, 0, 0, 4096, timeout=3000)
        text = bytes(resp).rstrip(b"\x00").decode("utf-8", errors="replace").strip()
        return text, True
    except usb.core.USBError:
        return "", True

def getenv(dev, var):
    return send_cmd(dev, f"getenv {var}")

def setenv(dev, var, val):
    return send_cmd(dev, f"setenv {var} {val}")

# ============================================================
# STEP 1: Check for DFU device
# ============================================================
print("=" * 60)
print("STEP 1: Looking for DFU device (PID=0x1227)...")
print("=" * 60)

dfu_dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
if not dfu_dev:
    # Check if device is in Recovery
    rec_dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
    if rec_dev:
        print("Device is in RECOVERY mode, not DFU.")
        print("Please force-restart the device:")
        print("  1. Tap Volume Up")
        print("  2. Tap Volume Down")
        print("  3. Hold Side button for 10+ seconds (until screen goes black)")
        print("  4. Release, then hold both Vol Down + Side for 5 seconds")
        print("  5. Release Side, keep holding Vol Down for 10 seconds")
        print("\nWaiting for DFU device...")
        dfu_dev = find_device(0x1227, timeout=120)
    if not dfu_dev:
        print("ERROR: No DFU device found. Put device in DFU mode and re-run.")
        sys.exit(1)

print(f"DFU device found!")
try:
    sn = dfu_dev.serial_number
    print(f"  Serial: {sn[:80]}")
except:
    pass

# ============================================================
# STEP 2: Send signed iBSS
# ============================================================
print("\n" + "=" * 60)
print("STEP 2: Sending signed iBSS...")
print("=" * 60)

ibss_path = os.path.join(CACHE, "ibss_signed.img4")
if not os.path.exists(ibss_path):
    print(f"ERROR: {ibss_path} not found!")
    sys.exit(1)

with open(ibss_path, "rb") as f:
    ibss_data = f.read()
print(f"  IMG4: {len(ibss_data)} bytes")

# Use pymobiledevice3 IRecv for proper DFU send protocol
from pymobiledevice3.irecv import IRecv
print("  Connecting via IRecv...")
try:
    irecv = IRecv()
    print(f"  Mode: {irecv.mode}")
    print(f"  Sending via IRecv.send_buffer()...")
    irecv.send_buffer(ibss_data)
    print("  iBSS sent successfully!")
except Exception as e:
    print(f"  IRecv error: {e}")
    print("  Trying manual DFU send...")
    # Fallback: manual DFU protocol
    try:
        dfu_dev.set_configuration()
    except:
        pass
    try:
        usb.util.claim_interface(dfu_dev, 0)
    except:
        pass
    
    # Send via DFU DNLOAD
    BLOCK = 2048
    offset = 0
    pkt_idx = 0
    while offset < len(ibss_data):
        chunk = ibss_data[offset:offset + BLOCK]
        try:
            dfu_dev.ctrl_transfer(0x21, 1, pkt_idx, 0, chunk, timeout=5000)
            offset += len(chunk)
            pkt_idx += 1
        except Exception as ex:
            print(f"  DFU send error at {offset}: {ex}")
            break
    print(f"  Sent {offset}/{len(ibss_data)} bytes via DFU")

# ============================================================
# STEP 3: Wait for iBSS Recovery mode
# ============================================================
print("\n" + "=" * 60)
print("STEP 3: Waiting for iBSS (Recovery mode)...")
print("=" * 60)

time.sleep(3)
ibss_dev = find_device(0x1281, timeout=30)
if not ibss_dev:
    print("ERROR: iBSS did not boot to Recovery mode!")
    print("Device might still be in DFU. Check the device.")
    sys.exit(1)

print("iBSS Recovery mode found!")
try:
    ibss_dev.set_configuration(1)
except:
    pass
try:
    usb.util.claim_interface(ibss_dev, 0)
except:
    pass

# Verify we're in iBSS
ver, ok = getenv(ibss_dev, "build-version")
if ok:
    print(f"  build-version: {ver}")
else:
    print(f"  getenv returned: {ver}")

# ============================================================
# STEP 4: Set auto-boot=false (THE KEY FIX)
# ============================================================
print("\n" + "=" * 60)
print("STEP 4: Setting auto-boot=false in iBSS...")
print("=" * 60)

# First check current value
cur, ok = getenv(ibss_dev, "auto-boot")
print(f"  Current auto-boot: {cur} (ok={ok})")

# Set it to false
result, ok = setenv(ibss_dev, "auto-boot", "false")
print(f"  setenv result: ok={ok} resp='{result}'")

# Verify
new_val, ok = getenv(ibss_dev, "auto-boot")
print(f"  Verify auto-boot: {new_val} (ok={ok})")

if "false" in new_val.lower():
    print("  *** auto-boot=false SET SUCCESSFULLY! ***")
else:
    print("  WARNING: auto-boot might not have changed!")
    print("  Continuing anyway...")

# ============================================================
# STEP 5: Send signed iBEC
# ============================================================
print("\n" + "=" * 60)
print("STEP 5: Sending signed iBEC to iBSS...")
print("=" * 60)

ibec_path = os.path.join(CACHE, "ibec_signed.img4")
if not os.path.exists(ibec_path):
    # Try to stitch iBEC on the fly
    print("  ibec_signed.img4 not found, stitching...")
    from pymobiledevice3.restore.img4 import IMG4, IM4P
    from pymobiledevice3.restore.tss import TSSResponse
    
    ibec_raw_path = os.path.join(CACHE, "iBEC.n841.RELEASE.im4p")
    tss_path = os.path.join(CACHE, "tss_success_response.plist")
    
    with open(ibec_raw_path, "rb") as f:
        ibec_im4p_data = f.read()
    im4p = IM4P(data=ibec_im4p_data)
    
    with open(tss_path, "rb") as f:
        tss = TSSResponse(plistlib.load(f))
    ticket = tss.ap_img4_ticket
    
    img4 = IMG4(im4p=im4p, im4m=ticket)
    ibec_data = img4.output()
    print(f"  Stitched: {len(ibec_data)} bytes")
else:
    with open(ibec_path, "rb") as f:
        ibec_data = f.read()
    print(f"  Loaded: {len(ibec_data)} bytes")

# Initiate upload
print("  Initiating firmware upload...")
try:
    ibss_dev.ctrl_transfer(0x41, 0, 0, 0, b"", timeout=5000)
    print("  Upload initiated (0x41)")
except Exception as e:
    print(f"  Init: {e}")

# Send via bulk EP 0x04
BLOCK = 8192
offset = 0
t0 = time.time()
while offset < len(ibec_data):
    chunk = ibec_data[offset:offset + BLOCK]
    try:
        n = ibss_dev.write(0x04, chunk, timeout=5000)
        offset += n
    except usb.core.USBError as e:
        print(f"  Write error at {offset}: {e}")
        break

dt = time.time() - t0
print(f"  Sent {offset}/{len(ibec_data)} bytes in {dt:.1f}s")

# Execute with "go"
print("  Sending 'go' command...")
try:
    ibss_dev.ctrl_transfer(0x40, 0, 0, 0, b"go\x00", timeout=5000)
    print("  'go' sent!")
except usb.core.USBError as e:
    # Might get error if device immediately disconnects
    print(f"  go result: {e} (normal if device reset)")

# ============================================================
# STEP 6: Wait for iBEC and test commands
# ============================================================
print("\n" + "=" * 60)
print("STEP 6: Waiting for iBEC with interactive console...")
print("=" * 60)

time.sleep(4)
ibec_dev = find_device(0x1281, timeout=30)
if not ibec_dev:
    print("ERROR: iBEC did not boot!")
    sys.exit(1)

print("iBEC Recovery mode found!")
try:
    ibec_dev.set_configuration(1)
except:
    pass
try:
    usb.util.claim_interface(ibec_dev, 0)
except:
    pass
try:
    usb.util.claim_interface(ibec_dev, 1)
    ibec_dev.set_interface_altsetting(1, 1)
except:
    pass

# === THE CRITICAL TEST: does ctrl 0x40 work now? ===
print("\n--- Testing ctrl 0x40 command interface ---")
test_passed = False

# Try getenv
ver, ok = getenv(ibec_dev, "build-version")
if ok:
    print(f"  *** build-version: {ver} ***")
    test_passed = True
else:
    print(f"  build-version: {ver}")

# Try bgcolor to see visual change
if not test_passed:
    # Try purple
    result, ok = send_cmd(ibec_dev, "bgcolor 128 0 255")
    if ok:
        print(f"  bgcolor: OK")
        test_passed = True
    else:
        print(f"  bgcolor: {result}")

if test_passed:
    print("\n*** SUCCESS! iBEC interactive console is active! ***")
    
    # Explore the full console
    print("\n=== iBEC ENVIRONMENT ===")
    envvars = [
        "build-version", "build-style", "auto-boot", "boot-device",
        "boot-path", "loadaddr", "config_board", "display-timing",
        "debug-uarts", "debug-shmcon", "filesize", "bootdelay",
        "permit-random-gen-nonce", "allow-mix-and-match",
        "effective-production-status-ap", "effective-security-mode-ap",
    ]
    for var in envvars:
        val, ok = getenv(ibec_dev, var)
        if ok and val:
            print(f"  {var} = {val}")
    
    print("\n=== iBEC COMMANDS ===")
    commands = [
        "help",
        "version",
        "printenv",
        "devicetree",
        "meminfo",
    ]
    for cmd in commands:
        val, ok = send_cmd(ibec_dev, cmd)
        if ok:
            # Show first 500 chars
            if val:
                print(f"\n  [{cmd}]:")
                for line in val[:1000].split('\n'):
                    print(f"    {line}")
            else:
                print(f"  [{cmd}]: (ok, empty)")
        else:
            print(f"  [{cmd}]: STALL (not available)")
    
    # Try memory dump
    print("\n=== MEMORY ACCESS ===")
    mem_addrs = [
        ("0x100000000", "SecureROM"),
        ("0x800000000", "SRAM"),
        ("0x802000000", "Load address"),
    ]
    for addr, name in mem_addrs:
        val, ok = send_cmd(ibec_dev, f"md {addr} 0x40")
        if ok and val:
            print(f"\n  [{name} @ {addr}]:")
            for line in val[:500].split('\n'):
                print(f"    {line}")
        else:
            print(f"  [{name} @ {addr}]: {val[:100] if val else 'STALL'}")

else:
    print("\n*** FAILED: iBEC still rejects commands ***")
    print("Trying alternative approaches...")
    
    # Try bulk serial
    print("\n--- Bulk serial (EP 0x02/0x81) ---")
    total = b""
    for _ in range(3):
        try:
            data = ibec_dev.read(0x81, 16384, timeout=1000)
            total += bytes(data)
        except:
            break
    if total:
        print(f"  Serial output ({len(total)} bytes):")
        print(f"  {total.decode('utf-8', errors='replace')[:500]}")
    
    # Maybe we need to drain serial first
    print("\n--- Retry ctrl after serial drain ---")
    ver, ok = getenv(ibec_dev, "build-version")
    if ok:
        print(f"  *** IT WORKS AFTER DRAIN! build-version: {ver} ***")
    else:
        print(f"  Still fails: {ver}")
    
    # Check what 0xC0 returns
    try:
        ret = ibec_dev.ctrl_transfer(0xC0, 0, 0, 0, 4096, timeout=2000)
        print(f"\n  0xC0 default read: {bytes(ret)!r}")
    except Exception as e:
        print(f"\n  0xC0 read: {e}")

print("\nDone.")
