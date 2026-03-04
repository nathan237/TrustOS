#!/usr/bin/env python3
"""
Complete pipeline: TSS ticket + iBSS + iBEC boot.
Handles nonce changes, uses IRecv for DFU send, pyusb for Recovery commands.
"""
import plistlib, asyncio, time, os, sys
import urllib3
urllib3.disable_warnings()

import usb.core, usb.util, usb.backend.libusb1, libusb_package
from pymobiledevice3.irecv import IRecv, Mode
from pymobiledevice3.restore.tss import TSSRequest
from pymobiledevice3.restore.img4 import IMG4, IM4P

be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
CACHE = "tools/checkm8-dualboot/cache"
MANIFEST = os.path.join(CACHE, "3a4b009f41c729764f254068727c5545_BuildManifest.plist")
IBSS_IM4P = os.path.join(CACHE, "5f574195af3c7b8bd9a14dcd1eed019c_iBSS.n841.RELEASE.im4p")
IBEC_IM4P = os.path.join(CACHE, "iBEC.n841.RELEASE.im4p")

def find_recovery():
    return usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)

def recovery_cmd(dev, cmd):
    """Send command to Recovery device via ctrl 0x40."""
    data = (cmd + "\x00").encode("utf-8")
    dev.ctrl_transfer(0x40, 0, 0, 0, data, timeout=5000)

def recovery_getenv(dev, var):
    """Get env var from Recovery device."""
    recovery_cmd(dev, f"getenv {var}")
    time.sleep(0.1)
    resp = dev.ctrl_transfer(0xC0, 0, 0, 0, 512, timeout=2000)
    return bytes(resp).split(b"\x00")[0].decode("utf-8", errors="replace")

# ===== STEP 1: Read DFU nonce =====
print("=" * 60)
print("STEP 1: Connect to DFU")
print("=" * 60)

irecv = IRecv()
assert irecv.mode == Mode.DFU_MODE, f"Not in DFU: {irecv.mode}"
ap_nonce = irecv.ap_nonce
sep_nonce = irecv.sep_nonce
ecid = irecv.ecid
print(f"  ECID: {ecid:#x}")
print(f"  Nonce: {ap_nonce.hex()[:32]}...")

# ===== STEP 2: TSS ticket =====
print("\n" + "=" * 60)
print("STEP 2: TSS ticket")
print("=" * 60)

with open(MANIFEST, 'rb') as f:
    manifest = plistlib.load(f)

identity = next(i for i in manifest['BuildIdentities'] 
                if i.get('Info', {}).get('DeviceClass', '') == 'n841ap')

params = {
    'ApECID': ecid, 'ApNonce': ap_nonce, 'ApSepNonce': sep_nonce,
    'ApProductionMode': True, 'ApSecurityMode': True, 'ApSupportsImg4': True,
}
for k in ['ApBoardID','ApChipID','UniqueBuildID','Ap,OSLongVersion',
           'Ap,OSReleaseType','Ap,ProductType','Ap,SDKPlatform','Ap,SikaFuse',
           'Ap,Target','Ap,TargetType','ApSecurityDomain',
           'BMU,BoardID','BMU,ChipID','BbChipID','BbProvisioningManifestKeyHash',
           'BbActivationManifestKeyHash','BbCalibrationManifestKeyHash',
           'Ap,ProductMarketingVersion','BbFactoryActivationManifestKeyHash',
           'BbFDRSecurityKeyHash','BbSkeyId','SE,ChipID','Savage,ChipID',
           'Savage,PatchEpoch','Yonkers,BoardID','Yonkers,ChipID',
           'Yonkers,PatchEpoch','Rap,BoardID','Rap,ChipID','Rap,SecurityDomain',
           'Baobab,BoardID','Baobab,ChipID','Baobab,ManifestEpoch',
           'Baobab,SecurityDomain','eUICC,ChipID','PearlCertificationRootPub',
           'Timer,BoardID,1','Timer,BoardID,2','Timer,ChipID,1','Timer,ChipID,2',
           'Timer,SecurityDomain,1','Timer,SecurityDomain,2','Manifest','NeRDEpoch']:
    v = identity.get(k)
    if v is not None:
        params[k] = int(v, 16) if isinstance(v, str) and v.startswith('0x') else v

tss = TSSRequest()
tss.add_common_tags(params)
tss.add_ap_tags(params)
tss.add_ap_img4_tags(params)

print("  Sending TSS request...")
resp = asyncio.run(tss.send_receive())
ticket = resp['ApImg4Ticket']
print(f"  SUCCESS! Ticket: {len(ticket)} bytes")

# Save
plistlib.dump(dict(resp), open(os.path.join(CACHE, 'tss_success_response.plist'), 'wb'))
open(os.path.join(CACHE, 'apticket.der'), 'wb').write(ticket)

# ===== STEP 3: Stitch =====
print("\n" + "=" * 60)
print("STEP 3: Stitch IMG4")
print("=" * 60)

ibss_img4 = IMG4(im4p=IM4P(data=open(IBSS_IM4P, 'rb').read()), im4m=ticket).output()
open(os.path.join(CACHE, 'ibss_signed.img4'), 'wb').write(ibss_img4)
print(f"  iBSS: {len(ibss_img4)} bytes")

ibec_img4 = IMG4(im4p=IM4P(data=open(IBEC_IM4P, 'rb').read()), im4m=ticket).output()
open(os.path.join(CACHE, 'ibec_signed.img4'), 'wb').write(ibec_img4)
print(f"  iBEC: {len(ibec_img4)} bytes")

# ===== STEP 4: Send iBSS =====
print("\n" + "=" * 60)
print("STEP 4: Send iBSS")
print("=" * 60)

irecv = IRecv()
print(f"  Sending {len(ibss_img4)} bytes...")
try:
    irecv.send_buffer(ibss_img4)
    print("  send_buffer OK")
except Exception as e:
    # "More than one device" means it rebooted to Recovery successfully
    if "More then one" in str(e) or "more than one" in str(e).lower():
        print(f"  (multi-device detection = device rebooted, OK)")
    else:
        print(f"  Error: {e}")
        sys.exit(1)

# ===== STEP 5: Wait for iBSS Recovery =====
print("\n" + "=" * 60)
print("STEP 5: Wait for iBSS Recovery")
print("=" * 60)

time.sleep(3)

dev = None
for i in range(30):
    dev = find_recovery()
    if dev:
        try:
            sn = dev.serial_number
            print(f"  Found! Serial: {sn[:60]}")
        except:
            print(f"  Found PID=0x1281")
        break
    if i % 5 == 0:
        print(f"  Waiting... {i}s")
    time.sleep(1)

if not dev:
    print("  No Recovery device!")
    sys.exit(1)

# Configure
dev.set_configuration()
usb.util.claim_interface(dev, 0)

# Verify it's our iBSS (version 11881.140.96) not factory Recovery
bv = recovery_getenv(dev, "build-version")
print(f"  build-version: {bv}")

if "11881.140.96" in bv:
    print("  *** This is our iBSS! ***")
elif "11881.0.193" in bv:
    print("  WARNING: This is factory Recovery, not our iBSS!")
    print("  The iBSS may not have been validated. Aborting.")
    sys.exit(1)

# ===== STEP 6: Configure iBSS (NO auto-boot change, NO reset) =====
print("\n" + "=" * 60)
print("STEP 6: iBSS commands")
print("=" * 60)

# bgcolor green to confirm
try:
    recovery_cmd(dev, "bgcolor 0 255 0")
    print("  bgcolor GREEN sent - check screen!")
except Exception as e:
    print(f"  bgcolor: {e}")

# Read some env vars
for var in ['loadaddr', 'config_board', 'auto-boot', 'boot-device']:
    try:
        val = recovery_getenv(dev, var)
        print(f"  {var} = {val}")
    except:
        print(f"  {var} = (error)")

# ===== STEP 7: Send iBEC via bulk =====
print("\n" + "=" * 60)
print("STEP 7: Send iBEC")
print("=" * 60)

# Init file transfer
try:
    dev.ctrl_transfer(0x41, 0, 0, 0, b"", timeout=5000)
except:
    pass

# Send via bulk EP 0x04
CHUNK = 0x8000
sent = 0
t0 = time.time()
while sent < len(ibec_img4):
    chunk = ibec_img4[sent:sent + CHUNK]
    written = dev.write(0x04, chunk, timeout=10000)
    sent += written
    if (sent // CHUNK) % 5 == 0:
        print(f"  {100*sent//len(ibec_img4)}%")

dt = time.time() - t0
print(f"  Sent {sent} bytes in {dt:.1f}s")

# Execute
try:
    dev.ctrl_transfer(0x41, 2, 0, 0, b"", timeout=5000)
    print("  Execute sent")
except:
    print("  (device disconnected - normal)")

# ===== STEP 8: Wait for iBEC =====
print("\n" + "=" * 60)
print("STEP 8: Wait for iBEC")
print("=" * 60)

time.sleep(3)

dev2 = None
for i in range(30):
    dev2 = find_recovery()
    if dev2:
        try:
            sn = dev2.serial_number
            print(f"  Found! Serial: {sn[:60]}")
        except:
            print(f"  Found PID=0x1281")
        break
    if i % 5 == 0:
        print(f"  Waiting... {i}s")
    time.sleep(1)

if not dev2:
    print("  No iBEC device found!")
    sys.exit(1)

dev2.set_configuration()
usb.util.claim_interface(dev2, 0)

# ===== STEP 9: Explore iBEC =====
print("\n" + "=" * 60)
print("STEP 9: iBEC Exploration")
print("=" * 60)

bv = recovery_getenv(dev2, "build-version")
print(f"  build-version: {bv}")

# bgcolor blue
try:
    recovery_cmd(dev2, "bgcolor 0 0 255")
    print("  bgcolor BLUE - check screen!")
except Exception as e:
    print(f"  bgcolor: {e}")

# All env vars
print("\n  Environment variables:")
for var in ['build-version', 'build-style', 'loadaddr', 'config_board',
            'auto-boot', 'boot-device', 'display-color-space', 'display-timing',
            'boot-command', 'boot-args', 'platform-name', 'security-domain']:
    try:
        val = recovery_getenv(dev2, var)
        print(f"    {var} = {val}")
    except:
        print(f"    {var} = (error)")

print("\n*** PIPELINE COMPLETE ***")
print("Device is in iBEC Recovery mode.")
print("You can send commands via recovery_cmd(dev2, 'command')")
