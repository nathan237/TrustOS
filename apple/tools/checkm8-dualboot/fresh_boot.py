#!/usr/bin/env python3
"""
Fresh iBSS boot: get new TSS ticket with current nonce, stitch, send, boot.
All-in-one pipeline that handles nonce changes.
"""
import plistlib, asyncio, time, os, sys, warnings
import urllib3
urllib3.disable_warnings()

from pymobiledevice3.irecv import IRecv, Mode
from pymobiledevice3.restore.tss import TSSRequest, TSSResponse
from pymobiledevice3.restore.img4 import IMG4, IM4P

CACHE = "tools/checkm8-dualboot/cache"
MANIFEST = os.path.join(CACHE, "3a4b009f41c729764f254068727c5545_BuildManifest.plist")
IBSS_IM4P = os.path.join(CACHE, "5f574195af3c7b8bd9a14dcd1eed019c_iBSS.n841.RELEASE.im4p")
IBEC_IM4P = os.path.join(CACHE, "iBEC.n841.RELEASE.im4p")

# ===========================================================
# STEP 1: Connect to DFU device and read current nonce
# ===========================================================
print("=" * 60)
print("STEP 1: Connect to DFU device")
print("=" * 60)

irecv = IRecv()
print(f"  Mode: {irecv.mode}")
if irecv.mode != Mode.DFU_MODE:
    print(f"  ERROR: Expected DFU mode, got {irecv.mode}")
    sys.exit(1)

ap_nonce = irecv.ap_nonce
sep_nonce = irecv.sep_nonce
ecid = irecv.ecid

print(f"  ECID: {ecid:#x}")
print(f"  AP Nonce: {ap_nonce.hex()}")
print(f"  SEP Nonce: {sep_nonce.hex()}")

# ===========================================================
# STEP 2: Request fresh TSS ticket with current nonce
# ===========================================================
print("\n" + "=" * 60)
print("STEP 2: Request TSS ticket (Apple signing server)")
print("=" * 60)

with open(MANIFEST, 'rb') as f:
    manifest = plistlib.load(f)

# Find n841ap identity
identity = None
for ident in manifest['BuildIdentities']:
    if ident.get('Info', {}).get('DeviceClass', '') == 'n841ap':
        identity = ident
        break

if not identity:
    print("ERROR: n841ap identity not found in BuildManifest!")
    sys.exit(1)

# Build parameters
parameters = {}
parameters['ApECID'] = ecid
parameters['ApNonce'] = ap_nonce
parameters['ApSepNonce'] = sep_nonce
parameters['ApProductionMode'] = True
parameters['ApSecurityMode'] = True
parameters['ApSupportsImg4'] = True

# Copy manifest keys
key_list = [
    'ApBoardID', 'ApChipID', 'UniqueBuildID', 'Ap,OSLongVersion',
    'Ap,OSReleaseType', 'Ap,ProductType', 'Ap,SDKPlatform', 'Ap,SikaFuse',
    'Ap,Target', 'Ap,TargetType', 'ApSecurityDomain',
    'BMU,BoardID', 'BMU,ChipID', 'BbChipID', 'BbProvisioningManifestKeyHash',
    'BbActivationManifestKeyHash', 'BbCalibrationManifestKeyHash',
    'Ap,ProductMarketingVersion', 'BbFactoryActivationManifestKeyHash',
    'BbFDRSecurityKeyHash', 'BbSkeyId', 'SE,ChipID', 'Savage,ChipID',
    'Savage,PatchEpoch', 'Yonkers,BoardID', 'Yonkers,ChipID',
    'Yonkers,PatchEpoch', 'Rap,BoardID', 'Rap,ChipID', 'Rap,SecurityDomain',
    'Baobab,BoardID', 'Baobab,ChipID', 'Baobab,ManifestEpoch',
    'Baobab,SecurityDomain', 'eUICC,ChipID', 'PearlCertificationRootPub',
    'Timer,BoardID,1', 'Timer,BoardID,2', 'Timer,ChipID,1', 'Timer,ChipID,2',
    'Timer,SecurityDomain,1', 'Timer,SecurityDomain,2', 'Manifest', 'NeRDEpoch',
]

for k in key_list:
    v = identity.get(k)
    if v is not None:
        if isinstance(v, str) and v.startswith('0x'):
            v = int(v, 16)
        parameters[k] = v

tss = TSSRequest()
tss.add_common_tags(parameters)
tss.add_ap_tags(parameters)
tss.add_ap_img4_tags(parameters)

print(f"  Sending TSS request (nonce: {ap_nonce.hex()[:16]}...)...")
try:
    resp = asyncio.run(tss.send_receive())
except Exception as e:
    print(f"  TSS ERROR: {e}")
    sys.exit(1)

print("  TSS SUCCESS!")

# Save response
rdata = plistlib.dumps(dict(resp))
with open(os.path.join(CACHE, 'tss_success_response.plist'), 'wb') as f:
    f.write(rdata)

ticket = resp['ApImg4Ticket']
print(f"  APTicket: {len(ticket)} bytes")

with open(os.path.join(CACHE, 'apticket.der'), 'wb') as f:
    f.write(ticket)

# ===========================================================
# STEP 3: Stitch iBSS + ticket into IMG4
# ===========================================================
print("\n" + "=" * 60)
print("STEP 3: Stitch iBSS IMG4")
print("=" * 60)

with open(IBSS_IM4P, 'rb') as f:
    ibss_im4p_data = f.read()

im4p = IM4P(data=ibss_im4p_data)
print(f"  IM4P: {im4p.fourcc} - {im4p.description}")

img4 = IMG4(im4p=im4p, im4m=ticket)
ibss_img4 = img4.output()
print(f"  IMG4: {len(ibss_img4)} bytes")

with open(os.path.join(CACHE, 'ibss_signed.img4'), 'wb') as f:
    f.write(ibss_img4)
print("  Saved ibss_signed.img4")

# Also stitch iBEC for later
print("  Stitching iBEC...")
with open(IBEC_IM4P, 'rb') as f:
    ibec_im4p_data = f.read()

im4p_ibec = IM4P(data=ibec_im4p_data)
img4_ibec = IMG4(im4p=im4p_ibec, im4m=ticket)
ibec_img4 = img4_ibec.output()

with open(os.path.join(CACHE, 'ibec_signed.img4'), 'wb') as f:
    f.write(ibec_img4)
print(f"  Saved ibec_signed.img4 ({len(ibec_img4)} bytes)")

# ===========================================================
# STEP 4: Send iBSS via IRecv
# ===========================================================
print("\n" + "=" * 60)
print("STEP 4: Send iBSS to DFU device")
print("=" * 60)

# Re-connect fresh IRecv
irecv = IRecv()
print(f"  Mode: {irecv.mode}")

print(f"  Sending {len(ibss_img4)} bytes...")
irecv.send_buffer(ibss_img4)
print("  send_buffer() completed!")

# ===========================================================
# STEP 5: Wait for iBSS (Recovery mode)
# ===========================================================
print("\n" + "=" * 60)
print("STEP 5: Waiting for iBSS Recovery mode...")
print("=" * 60)

time.sleep(3)

for attempt in range(20):
    try:
        irecv2 = IRecv()
        print(f"  Mode: {irecv2.mode}")
        if irecv2.mode in (Mode.RECOVERY_MODE_1, Mode.RECOVERY_MODE_2, 
                           Mode.RECOVERY_MODE_3, Mode.RECOVERY_MODE_4):
            print("  *** iBSS BOOTED! We're in Recovery mode! ***")
            
            # Set auto-boot=false immediately
            print("\n  Setting auto-boot=false...")
            try:
                irecv2.set_autoboot(False)
                print("  auto-boot set to false!")
            except Exception as e:
                print(f"  auto-boot error: {e}")
            
            # Test with bgcolor
            print("  Testing bgcolor (green)...")
            try:
                irecv2.send_command("bgcolor 0 255 0")
                print("  bgcolor sent! Check if screen is GREEN.")
            except Exception as e:
                print(f"  bgcolor error: {e}")
            
            # Now send iBEC
            print("\n" + "=" * 60)
            print("STEP 6: Send iBEC")
            print("=" * 60)
            
            print(f"  Sending iBEC ({len(ibec_img4)} bytes)...")
            try:
                irecv2.send_buffer(ibec_img4)
                print("  iBEC sent!")
            except Exception as e:
                print(f"  iBEC send error: {e}")
            
            time.sleep(3)
            
            # Connect to iBEC
            try:
                irecv3 = IRecv()
                print(f"  iBEC mode: {irecv3.mode}")
                
                # Test iBEC console
                print("\n  Testing iBEC console...")
                try:
                    irecv3.send_command("bgcolor 0 0 255")
                    print("  bgcolor blue sent!")
                except Exception as e:
                    print(f"  bgcolor: {e}")
                
                try:
                    env = irecv3.getenv("build-version")
                    print(f"  build-version: {env}")
                except Exception as e:
                    print(f"  getenv: {e}")
                    
            except Exception as e:
                print(f"  iBEC connect error: {e}")
            
            print("\n*** PIPELINE COMPLETE! ***")
            sys.exit(0)
        
        elif irecv2.mode == Mode.DFU_MODE:
            if attempt < 19:
                print(f"  Still DFU (attempt {attempt+1}/20)...")
            else:
                print("  Still DFU after 20 attempts.")
    except Exception as e:
        if attempt < 19:
            print(f"  Connection error (attempt {attempt+1}/20): {e}")
        else:
            print(f"  Final attempt failed: {e}")
    
    time.sleep(2)

print("\nFailed to reach Recovery mode.")
print("The device may need a physical cable unplug/replug.")
print("Or the Zadig USB driver may be interfering with the bus reset.")
sys.exit(1)
