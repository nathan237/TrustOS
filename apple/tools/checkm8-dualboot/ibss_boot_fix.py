#!/usr/bin/env python3
"""
Send iBSS and use DFU_ABORT + multiple reset strategies to trigger boot.
The key insight: libusb reset on Windows doesn't do a real port power cycle.
We need to try: abort, USB reset, port cycle.
"""
import plistlib, asyncio, time, os, sys, math, binascii, struct
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

def find_device(pid):
    return usb.core.find(idVendor=0x05AC, idProduct=pid, backend=be)

# ===== STEP 1: Read nonce =====
print("STEP 1: DFU device")
irecv = IRecv()
assert irecv.mode == Mode.DFU_MODE
nonce = irecv.ap_nonce
sep_nonce = irecv.sep_nonce
ecid = irecv.ecid
print(f"  Nonce: {nonce.hex()[:32]}...")

# ===== STEP 2: Fresh TSS ticket =====
print("\nSTEP 2: TSS ticket")
with open(MANIFEST, 'rb') as f:
    manifest = plistlib.load(f)
identity = next(i for i in manifest['BuildIdentities']
                if i.get('Info', {}).get('DeviceClass', '') == 'n841ap')
params = {
    'ApECID': ecid, 'ApNonce': nonce, 'ApSepNonce': sep_nonce,
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
resp = asyncio.run(tss.send_receive())
ticket = resp['ApImg4Ticket']
print(f"  Ticket: {len(ticket)} bytes")

# Stitch
ibss_img4 = IMG4(im4p=IM4P(data=open(IBSS_IM4P, 'rb').read()), im4m=ticket).output()
print(f"  iBSS IMG4: {len(ibss_img4)} bytes")

# ===== STEP 3: Send iBSS manually via pyusb (not IRecv) =====
print("\nSTEP 3: Send iBSS via DFU protocol")
dev = find_device(0x1227)
dev.set_configuration()
usb.util.claim_interface(dev, 0)

# Verify dfuIDLE
st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
state = st[4]
print(f"  DFU state: {state}")
if state != 2:
    dev.ctrl_transfer(0x21, 4, 0, 0, timeout=500)  # CLRSTATUS
    dev.ctrl_transfer(0x21, 6, 0, 0, timeout=500)  # ABORT
    time.sleep(0.5)
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
    print(f"  After clear: {st[4]}")

PACKET_SIZE = 2048
num_packets = math.ceil(len(ibss_img4) / PACKET_SIZE)
crc = -1
t0 = time.time()

for offset in range(0, len(ibss_img4), PACKET_SIZE):
    chunk = ibss_img4[offset:offset + PACKET_SIZE]
    packet_index = offset // PACKET_SIZE

    if offset + PACKET_SIZE >= len(ibss_img4):
        crc = binascii.crc32(ibss_img4, crc)
        dfu_xbuf = bytearray([0xFF,0xFF,0xFF,0xFF,0xAC,0x05,0x00,0x01,0x55,0x46,0x44,0x10])
        crc = binascii.crc32(dfu_xbuf, crc)
        crc_chunk = dfu_xbuf + struct.pack("<I", crc)
        if len(chunk) + 16 > PACKET_SIZE:
            dev.ctrl_transfer(0x21, 1, packet_index, 0, chunk, timeout=5000)
            dev.ctrl_transfer(0x21, 1, packet_index, 0, crc_chunk, timeout=5000)
        else:
            dev.ctrl_transfer(0x21, 1, packet_index, 0, chunk + crc_chunk, timeout=5000)
    else:
        dev.ctrl_transfer(0x21, 1, packet_index, 0, chunk, timeout=5000)

    if packet_index % 200 == 0 and packet_index > 0:
        print(f"  {100*offset//len(ibss_img4)}%")

dt = time.time() - t0
print(f"  Sent {num_packets} packets in {dt:.1f}s")

# Wait for status 5
print("  Waiting for DNLOAD-IDLE...")
for _ in range(20):
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
    if st[4] == 5:
        print(f"  State: 5 (DNLOAD-IDLE)")
        break
    time.sleep(0.5)

# Send empty DNLOAD
dev.ctrl_transfer(0x21, 1, num_packets, 0, b"", timeout=5000)

# Read status twice 
for _ in range(2):
    try:
        st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        print(f"  State: {st[4]}")
    except:
        print("  (disconnected)")
        break

# ===== STEP 4: Multiple reset strategies =====
print("\nSTEP 4: Trigger execution")

strategies = [
    "usb_reset",
    "abort",
    "detach", 
    "pnp_cycle",
]

for strat in strategies:
    print(f"\n  [{strat}]")
    
    if strat == "usb_reset":
        try:
            dev.reset()
            print("    libusb reset done")
        except Exception as e:
            print(f"    {e}")
    
    elif strat == "abort":
        try:
            dev.ctrl_transfer(0x21, 6, 0, 0, timeout=1000)  # DFU_ABORT
            print("    DFU ABORT sent")
        except Exception as e:
            print(f"    {e}")
    
    elif strat == "detach":
        try:
            # DFU_DETACH with timeout
            dev.ctrl_transfer(0x21, 0, 1000, 0, timeout=2000)
            print("    DFU DETACH sent")
        except Exception as e:
            print(f"    {e}")
    
    elif strat == "pnp_cycle":
        import subprocess
        iid = "USB\\VID_05AC&PID_1227\\*"
        try:
            subprocess.run(["powershell", "-c", 
                f"Disable-PnpDevice -InstanceId (Get-PnpDevice | Where-Object {{$_.InstanceId -like '{iid}'}}).InstanceId -Confirm:$false"],
                capture_output=True, timeout=10)
            print("    Disabled")
            time.sleep(2)
            subprocess.run(["powershell", "-c",
                f"Enable-PnpDevice -InstanceId (Get-PnpDevice | Where-Object {{$_.InstanceId -like '{iid}'}}).InstanceId -Confirm:$false"],
                capture_output=True, timeout=10)
            print("    Enabled")
        except Exception as e:
            print(f"    {e}")
    
    time.sleep(3)
    
    # Check if Recovery appeared
    r = find_device(0x1281)
    if r:
        print(f"  *** RECOVERY FOUND after {strat}! ***")
        try:
            print(f"  Serial: {r.serial_number[:60]}")
        except:
            pass
        sys.exit(0)
    
    d = find_device(0x1227)
    if d:
        try:
            d.set_configuration()
            usb.util.claim_interface(d, 0)
            st = d.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
            print(f"    Still DFU, state: {st[4]}")
            dev = d  # Update reference
        except:
            print("    DFU found but can't communicate")

print("\n=== ALL STRATEGIES FAILED ===")
print("The USB reset doesn't trigger iBSS boot from state 8.")
print("Options:")
print("  1. Try a DIFFERENT USB PORT or USB HUB")
print("  2. Try without any Zadig driver (uninstall libusbK)")
print("  3. Try on Linux (libusb reset works natively)")
sys.exit(1)
