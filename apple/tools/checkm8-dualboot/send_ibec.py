#!/usr/bin/env python3
"""Download iBEC, get TSS ticket, stitch IMG4, and send to device in iBSS Recovery mode."""
import plistlib, asyncio, time, struct, binascii, math, hashlib
import usb.core, usb.util, libusb_package
import urllib3
urllib3.disable_warnings()

from pymobiledevice3.restore.tss import TSSRequest, TSSResponse
from pymobiledevice3.restore.img4 import IMG4, IM4P

IPSW_URL = "https://updates.cdn-apple.com/2026WinterFCS/fullrestores/047-53269/25427FD8-2F38-4E14-88D6-AA5BA5FE340B/iPhone11,8_18.7.5_22H311_Restore.ipsw"
CACHE_DIR = "tools/checkm8-dualboot/cache"

import requests
session = requests.Session()
session.verify = False

# ===== STEP 1: Download iBEC from IPSW =====
import os, zipfile, io

def download_component_from_ipsw(ipsw_url, component_path, cache_name):
    """Download a specific file from IPSW using HTTP range requests (partial ZIP)."""
    cache_path = os.path.join(CACHE_DIR, cache_name)
    if os.path.exists(cache_path):
        print(f"  Using cached: {cache_path}")
        with open(cache_path, 'rb') as f:
            return f.read()
    
    print(f"  Downloading {component_path} from IPSW...")
    
    # Get IPSW size
    resp = session.head(ipsw_url, allow_redirects=True)
    ipsw_size = int(resp.headers['Content-Length'])
    
    # Read the End of Central Directory (last 65KB should be enough)
    tail_size = min(65536, ipsw_size)
    resp = session.get(ipsw_url, headers={'Range': f'bytes={ipsw_size - tail_size}-{ipsw_size - 1}'})
    tail = resp.content
    
    # Find EOCD signature
    eocd_sig = b'\x50\x4b\x05\x06'
    eocd_pos = tail.rfind(eocd_sig)
    if eocd_pos < 0:
        raise ValueError("EOCD not found")
    
    # Parse EOCD
    eocd = tail[eocd_pos:]
    cd_size = struct.unpack('<I', eocd[12:16])[0]
    cd_offset = struct.unpack('<I', eocd[16:20])[0]
    
    # Check for ZIP64
    if cd_offset == 0xFFFFFFFF:
        zip64_locator = tail[eocd_pos - 20:eocd_pos]
        zip64_offset = struct.unpack('<Q', zip64_locator[8:16])[0]
        resp = session.get(ipsw_url, headers={'Range': f'bytes={zip64_offset}-{zip64_offset + 100}'})
        zip64_eocd = resp.content
        cd_size = struct.unpack('<Q', zip64_eocd[40:48])[0]
        cd_offset = struct.unpack('<Q', zip64_eocd[48:56])[0]
    
    # Read central directory
    resp = session.get(ipsw_url, headers={'Range': f'bytes={cd_offset}-{cd_offset + cd_size - 1}'})
    cd_data = resp.content
    
    # Parse entries to find our file
    pos = 0
    target_offset = None
    target_comp_size = None
    while pos < len(cd_data):
        if cd_data[pos:pos+4] != b'\x50\x4b\x01\x02':
            break
        fname_len = struct.unpack('<H', cd_data[pos+28:pos+30])[0]
        extra_len = struct.unpack('<H', cd_data[pos+30:pos+32])[0]
        comment_len = struct.unpack('<H', cd_data[pos+32:pos+34])[0]
        comp_size = struct.unpack('<I', cd_data[pos+20:pos+24])[0]
        uncomp_size = struct.unpack('<I', cd_data[pos+24:pos+28])[0]
        local_offset = struct.unpack('<I', cd_data[pos+42:pos+46])[0]
        
        fname = cd_data[pos+46:pos+46+fname_len].decode('utf-8', errors='replace')
        
        # Handle ZIP64 extra field
        if comp_size == 0xFFFFFFFF or local_offset == 0xFFFFFFFF:
            extra = cd_data[pos+46+fname_len:pos+46+fname_len+extra_len]
            epos = 0
            while epos < len(extra) - 4:
                eid = struct.unpack('<H', extra[epos:epos+2])[0]
                esz = struct.unpack('<H', extra[epos+2:epos+4])[0]
                if eid == 1:  # ZIP64
                    edata = extra[epos+4:epos+4+esz]
                    idx = 0
                    if uncomp_size == 0xFFFFFFFF and idx + 8 <= len(edata):
                        uncomp_size = struct.unpack('<Q', edata[idx:idx+8])[0]
                        idx += 8
                    if comp_size == 0xFFFFFFFF and idx + 8 <= len(edata):
                        comp_size = struct.unpack('<Q', edata[idx:idx+8])[0]
                        idx += 8
                    if local_offset == 0xFFFFFFFF and idx + 8 <= len(edata):
                        local_offset = struct.unpack('<Q', edata[idx:idx+8])[0]
                    break
                epos += 4 + esz
        
        if fname == component_path:
            target_offset = local_offset
            target_comp_size = comp_size
            print(f"  Found {fname}: offset={local_offset}, size={comp_size}")
            break
        
        pos += 46 + fname_len + extra_len + comment_len
    
    if target_offset is None:
        raise FileNotFoundError(f"{component_path} not found in IPSW")
    
    # Read local file header to get actual data offset
    resp = session.get(ipsw_url, headers={'Range': f'bytes={target_offset}-{target_offset + 100}'})
    local_header = resp.content
    lfname_len = struct.unpack('<H', local_header[26:28])[0]
    lextra_len = struct.unpack('<H', local_header[28:30])[0]
    data_offset = target_offset + 30 + lfname_len + lextra_len
    
    # Download the file data
    resp = session.get(ipsw_url, headers={'Range': f'bytes={data_offset}-{data_offset + target_comp_size - 1}'})
    file_data = resp.content
    
    # Save to cache
    with open(cache_path, 'wb') as f:
        f.write(file_data)
    print(f"  Saved to {cache_path} ({len(file_data)} bytes)")
    
    return file_data

# Load manifest to find iBEC path
print("Loading BuildManifest...")
with open(os.path.join(CACHE_DIR, '3a4b009f41c729764f254068727c5545_BuildManifest.plist'), 'rb') as f:
    manifest = plistlib.load(f)

identity = None
for ident in manifest['BuildIdentities']:
    if ident.get('Info', {}).get('DeviceClass', '') == 'n841ap':
        identity = ident
        break

ibec_info = identity['Manifest']['iBEC']['Info']
ibec_path = ibec_info['Path']
print(f"  iBEC path in IPSW: {ibec_path}")

# Generate cache name from path
ibec_cache_name = hashlib.md5(ibec_path.encode()).hexdigest() + '_' + os.path.basename(ibec_path)
ibec_data = download_component_from_ipsw(IPSW_URL, ibec_path, ibec_cache_name)
print(f"  iBEC size: {len(ibec_data)} bytes")

# Parse as IM4P
im4p = IM4P(data=ibec_data)
print(f"  iBEC fourcc: {im4p.fourcc}")
print(f"  iBEC description: {im4p.description}")

# ===== STEP 2: Get TSS ticket (reuse existing or get new) =====
print("\nLoading existing TSS response...")
tss_path = os.path.join(CACHE_DIR, 'tss_success_response.plist')
with open(tss_path, 'rb') as f:
    tss_dict = plistlib.load(f)
tss_resp = TSSResponse(tss_dict)
ticket = tss_resp.ap_img4_ticket
print(f"  APTicket: {len(ticket)} bytes")

# ===== STEP 3: Stitch iBEC IMG4 =====
print("\nStitching iBEC IMG4...")
img4 = IMG4(im4p=im4p, im4m=ticket)
img4_data = img4.output()
print(f"  IMG4 output: {len(img4_data)} bytes")

with open(os.path.join(CACHE_DIR, 'ibec_signed.img4'), 'wb') as f:
    f.write(img4_data)
print("  Saved to cache/ibec_signed.img4")

# ===== STEP 4: Send to Recovery mode device =====
print("\nLooking for Recovery mode device...")
be = libusb_package.get_libusb1_backend()
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    print("ERROR: No recovery device found!")
    exit(1)

dev.set_configuration(1)

# In Recovery mode, firmware is sent via BULK endpoint (EP 0x04)
# First, initiate transfer with control request
print(f"\nSending {len(img4_data)} bytes via Recovery mode bulk transfer...")

# Initiate transfer (recovery mode uses 0x41, 0)
try:
    dev.ctrl_transfer(0x41, 0, 0, 0, timeout=5000)
    print("  Transfer initiated (0x41)")
except usb.core.USBError as e:
    print(f"  Initiate error: {e}")

# Send via BULK EP 0x04
BLOCK_SIZE = 8192  # Recovery mode supports larger transfers
offset = 0
block = 0
total = len(img4_data)

t_start = time.time()
while offset < total:
    chunk = img4_data[offset:offset + BLOCK_SIZE]
    try:
        written = dev.write(0x04, chunk, timeout=5000)
        offset += written
        block += 1
        if block % 50 == 0:
            print(f"  Block {block}: {offset}/{total} bytes ({100*offset//total}%)")
    except usb.core.USBError as e:
        print(f"  Write error at block {block}: {e}")
        break

t_send = time.time() - t_start
print(f"  Sent {offset} bytes in {t_send:.1f}s")

# Finalize - send control to signal completion
try:
    dev.ctrl_transfer(0x41, 0, 0, 0, timeout=5000)
    print("  Finalized")
except usb.core.USBError as e:
    print(f"  Finalize: {e}")

# Wait and check if device reboots into iBEC
print("\nWaiting for iBEC mode...")
time.sleep(5)

for wait in range(10):
    # Check all possible PIDs
    for pid, name in [(0x1281, 'Recovery'), (0x1227, 'DFU')]:
        d = usb.core.find(idVendor=0x05AC, idProduct=pid, backend=be)
        if d:
            try:
                d.set_configuration(1)
                s = usb.util.get_string(d, 1)
                print(f"Found {name} (0x{pid:04X})")
                print(f"  String #1: {s[:100]}")
                if name == 'Recovery':
                    # Check if build-version changed (iBEC vs iBSS)
                    try:
                        usb.util.claim_interface(d, 0)
                    except: pass
                    d.ctrl_transfer(0x40, 0, 0, 0, b"getenv build-version\x00", timeout=5000)
                    try:
                        resp = d.ctrl_transfer(0xC0, 0, 0, 0, 512, timeout=3000)
                        ver = bytes(resp).rstrip(b"\x00").decode()
                        print(f"  build-version: {ver}")
                    except: pass
                exit(0)
            except:
                pass
    time.sleep(2)
    print(f"  Waiting... ({(wait+1)*2 + 5}s)")

print("No device found after wait")
