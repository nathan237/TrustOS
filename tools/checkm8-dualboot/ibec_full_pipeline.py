#!/usr/bin/env python3
"""
Full pipeline using pymobiledevice3 IRecv throughout.
DFU -> iBSS -> set auto-boot=false -> send iBEC -> interactive console.

IRecv handles backend selection internally, avoiding libusbK issues.
"""
import time, sys, os, plistlib
from pymobiledevice3.irecv import IRecv, Mode
from pymobiledevice3.restore.img4 import IMG4, IM4P
from pymobiledevice3.restore.tss import TSSResponse

CACHE = "tools/checkm8-dualboot/cache"

# ============================================================
# STEP 1: Connect to DFU device
# ============================================================
print("=" * 60)
print("STEP 1: Connecting to DFU device...")
print("=" * 60)

try:
    irecv = IRecv()
    print(f"  Mode: {irecv.mode}")
    print(f"  ECID: {irecv.ecid:#x}")
    print(f"  Device info: {irecv}")
except Exception as e:
    print(f"  Failed: {e}")
    sys.exit(1)

if not irecv.mode.name.startswith("DFU"):
    print(f"  ERROR: Device is in {irecv.mode}, not DFU!")
    print(f"  If in Recovery, force restart to DFU first.")
    sys.exit(1)

# ============================================================
# STEP 2: Send signed iBSS
# ============================================================
print("\n" + "=" * 60)
print("STEP 2: Sending signed iBSS...")
print("=" * 60)

with open(os.path.join(CACHE, "ibss_signed.img4"), "rb") as f:
    ibss_data = f.read()
print(f"  IMG4: {len(ibss_data)} bytes")

print("  Sending via IRecv.send_buffer()...")
irecv.send_buffer(ibss_data)
print("  iBSS sent! IRecv re-initialized.")
print(f"  New mode: {irecv.mode}")

# The IRecv should now be connected to the Recovery mode device (iBSS)
if not irecv.mode.is_recovery:
    print(f"  WARNING: Expected Recovery mode, got {irecv.mode}")
    print("  Waiting for Recovery...")
    time.sleep(5)
    try:
        irecv = IRecv()
        print(f"  Mode: {irecv.mode}")
    except:
        print("  Still can't find Recovery device")
        sys.exit(1)

# ============================================================
# STEP 3: Test iBSS commands
# ============================================================
print("\n" + "=" * 60)
print("STEP 3: Testing iBSS commands...")
print("=" * 60)

# IRecv.send_command uses ctrl_transfer(0x40, b_request, 0, 0, cmd)
# IRecv.getenv uses send_command + ctrl_transfer(0xC0, 0, 0, 0, 256)
try:
    ver = irecv.getenv("build-version")
    print(f"  build-version: {ver}")
except Exception as e:
    print(f"  getenv error: {e}")

# ============================================================
# STEP 4: Set auto-boot=false (THE KEY)
# ============================================================
print("\n" + "=" * 60)
print("STEP 4: Setting auto-boot=false...")
print("=" * 60)

try:
    cur = irecv.getenv("auto-boot")
    print(f"  Current auto-boot: {cur}")
except:
    print("  Could not read auto-boot")

try:
    # IRecv has a built-in method!
    irecv.set_autoboot(False)
    print("  set_autoboot(False) called (setenv + saveenv)")
except Exception as e:
    print(f"  set_autoboot error: {e}")
    # Try manually
    try:
        irecv.send_command("setenv auto-boot false")
        print("  setenv auto-boot false: OK")
    except Exception as e2:
        print(f"  manual setenv: {e2}")

try:
    new_val = irecv.getenv("auto-boot")
    print(f"  Verify auto-boot: {new_val}")
except:
    print("  Could not verify auto-boot")

# ============================================================
# STEP 5: Send signed iBEC
# ============================================================
print("\n" + "=" * 60)
print("STEP 5: Sending signed iBEC...")
print("=" * 60)

ibec_path = os.path.join(CACHE, "ibec_signed.img4")
if not os.path.exists(ibec_path):
    # Stitch on the fly
    ibec_raw = os.path.join(CACHE, "iBEC.n841.RELEASE.im4p")
    tss_path = os.path.join(CACHE, "tss_success_response.plist")
    with open(ibec_raw, "rb") as f:
        im4p = IM4P(data=f.read())
    with open(tss_path, "rb") as f:
        tss = TSSResponse(plistlib.load(f))
    img4 = IMG4(im4p=im4p, im4m=tss.ap_img4_ticket)
    ibec_data = img4.output()
    with open(ibec_path, "wb") as f:
        f.write(ibec_data)
    print(f"  Stitched: {len(ibec_data)} bytes")
else:
    with open(ibec_path, "rb") as f:
        ibec_data = f.read()
    print(f"  Loaded: {len(ibec_data)} bytes")

print("  Sending iBEC via IRecv.send_buffer()...")
try:
    irecv.send_buffer(ibec_data)
    print("  iBEC sent! IRecv re-initialized.")
    print(f"  New mode: {irecv.mode}")
except Exception as e:
    print(f"  send_buffer error: {e}")
    print("  Trying manual send via ctrl 0x41 + bulk EP 0x04...")
    
    # Manual recovery-mode send
    try:
        irecv.ctrl_transfer(0x41, 0)
    except:
        pass
    
    import usb.core
    BLOCK = 8192
    offset = 0
    while offset < len(ibec_data):
        chunk = ibec_data[offset:offset + BLOCK]
        try:
            n = irecv._device.write(0x04, chunk, timeout=5000)
            offset += n
        except Exception as ex:
            print(f"    Write error at {offset}: {ex}")
            break
    print(f"    Sent {offset}/{len(ibec_data)} bytes")
    
    # Send "go" to execute
    try:
        irecv.send_command("go")
        print("    'go' sent!")
    except:
        pass
    
    # Wait and reconnect
    time.sleep(4)
    try:
        irecv = IRecv()
        print(f"    Reconnected: {irecv.mode}")
    except Exception as e2:
        print(f"    Reconnect failed: {e2}")
        sys.exit(1)

# ============================================================
# STEP 6: Test iBEC interactive console
# ============================================================
print("\n" + "=" * 60)
print("STEP 6: Testing iBEC interactive console...")
print("=" * 60)

# Test if commands work now!
test_passed = False

try:
    ver = irecv.getenv("build-version")
    print(f"  build-version: {ver}")
    if ver:
        test_passed = True
except Exception as e:
    print(f"  getenv: {e}")

try:
    irecv.send_command("bgcolor 128 0 255")
    print("  bgcolor: OK (screen should be purple)")
    test_passed = True
except Exception as e:
    print(f"  bgcolor: {e}")

if test_passed:
    print("\n*** SUCCESS! iBEC interactive console is active! ***")
    
    # Full exploration
    print("\n=== iBEC ENVIRONMENT ===")
    envvars = [
        "build-version", "build-style", "auto-boot", "boot-device",
        "boot-path", "loadaddr", "config_board", "display-timing",
        "debug-uarts", "debug-shmcon", "filesize", "bootdelay",
        "permit-random-gen-nonce", "allow-mix-and-match",
        "effective-production-status-ap", "effective-security-mode-ap",
        "device-material-color", "display-rotation",
    ]
    for var in envvars:
        try:
            val = irecv.getenv(var)
            if val:
                print(f"  {var} = {val}")
        except:
            pass
    
    # Try iBEC-specific commands
    print("\n=== iBEC COMMANDS ===")
    for cmd in ["version", "help", "printenv", "devicetree", "meminfo"]:
        try:
            irecv.send_command(cmd)
            # Read response
            try:
                resp = irecv.ctrl_transfer(0xC0, 0, data_or_wLength=4096)
                text = bytes(resp).rstrip(b"\x00").decode("utf-8", errors="replace").strip()
                if text:
                    print(f"\n  [{cmd}]:")
                    for line in text[:1000].split('\n'):
                        print(f"    {line}")
                else:
                    print(f"  [{cmd}]: (ok, empty)")
            except:
                print(f"  [{cmd}]: sent OK, no readable response")
        except Exception as e:
            print(f"  [{cmd}]: {e}")
    
    # Memory dump
    print("\n=== MEMORY ACCESS ===")
    for addr, name in [("0x100000000", "SecureROM"), ("0x802000000", "LoadAddr")]:
        try:
            irecv.send_command(f"md {addr} 0x40")
            resp = irecv.ctrl_transfer(0xC0, 0, data_or_wLength=4096)
            text = bytes(resp).rstrip(b"\x00").decode("utf-8", errors="replace").strip()
            if text:
                print(f"\n  [{name} @ {addr}]:")
                for line in text[:500].split('\n'):
                    print(f"    {line}")
        except Exception as e:
            print(f"  [{name} @ {addr}]: {e}")

else:
    print("\n*** iBEC commands still failing ***")
    print("  Device state may need different approach")

print("\nDone.")
