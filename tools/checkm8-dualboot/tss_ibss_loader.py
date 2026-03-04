#!/usr/bin/env python3
"""
TSS iBSS Loader — Get legitimately signed iBSS and boot to iBoot Recovery
==========================================================================

Flow:
  1. Read device info from DFU (ECID, CPID, BDID, NONC nonce)
  2. Find currently signed IPSW for iPhone XR (iPhone11,8)
  3. Download BuildManifest.plist + iBSS from IPSW (partial ZIP extraction)
  4. Build TSS request from BuildManifest identity + device info
  5. Send TSS request to gs.apple.com/TSS/controller
  6. Receive signed APTicket (SHSH2 blob)
  7. Stitch iBSS + APTicket into IMG4 container
  8. Send via DFU DNLOAD → device boots to iBoot Recovery mode (PID=0x1281)

Target: Apple A12 (T8020) iPhone XR — CPID:0x8020, BDID:0x0C
"""

import sys, os, io, re, time, struct, hashlib, uuid, json, warnings
import plistlib
import zipfile
from pathlib import Path
from datetime import datetime
from urllib.parse import urlparse

import requests
import urllib3
import usb.core, usb.util

# Suppress SSL warnings (corporate proxy / self-signed cert)
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)
warnings.filterwarnings('ignore', message='Unverified HTTPS request')

try:
    import libusb_package, usb.backend.libusb1
    USB_BACKEND = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
except:
    USB_BACKEND = None

# ============================================================
# Constants
# ============================================================
APPLE_VID     = 0x05AC
DFU_PID       = 0x1227
RECOVERY_PID  = 0x1281

DFU_DNLOAD    = 1
DFU_UPLOAD    = 2
DFU_GETSTATUS = 3
DFU_CLRSTATUS = 4
DFU_GETSTATE  = 5
DFU_ABORT     = 6

TSS_URL = "https://gs.apple.com/TSS/controller?action=2"
IPSW_API = "https://api.ipsw.me/v4"

DEVICE_ID = "iPhone11,8"  # iPhone XR

CACHE_DIR = Path(__file__).parent.resolve() / "cache"
CACHE_DIR.mkdir(exist_ok=True)

RESULTS_DIR = Path(__file__).parent.resolve() / "results"
RESULTS_DIR.mkdir(exist_ok=True)

# Create a global session with SSL verification disabled (corporate proxy workaround)
session = requests.Session()
session.verify = False

# ============================================================
# Logging
# ============================================================
log_lines = []
def log(msg, level="INFO"):
    ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    line = f"[{ts}] [{level:4s}] {msg}"
    print(line)
    log_lines.append(line)

# ============================================================
# USB/DFU helpers
# ============================================================
def find_dfu():
    """Find DFU device."""
    dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=USB_BACKEND)
    if dev:
        try: dev.set_configuration()
        except: pass
    return dev

def find_recovery():
    """Find Recovery mode device."""
    dev = usb.core.find(idVendor=APPLE_VID, idProduct=RECOVERY_PID, backend=USB_BACKEND)
    if dev:
        try: dev.set_configuration()
        except: pass
    return dev

def get_dfu_status(dev):
    """Get DFU status (6-byte response)."""
    try:
        data = dev.ctrl_transfer(0xA1, DFU_GETSTATUS, 0, 0, 6, timeout=2000)
        return {"bStatus": data[0], "bState": data[4],
                "poll_ms": data[1]|(data[2]<<8)|(data[3]<<16)}
    except:
        return None

def dfu_dnload(dev, payload, block=0, timeout=5000):
    """Send DFU_DNLOAD."""
    try:
        dev.ctrl_transfer(0x21, DFU_DNLOAD, block, 0, payload, timeout=timeout)
        return True
    except Exception as e:
        log(f"DNLOAD error: {e}", "ERR")
        return False

def dfu_abort(dev):
    try: dev.ctrl_transfer(0x21, DFU_ABORT, 0, 0, b'', timeout=1000)
    except: pass

def dfu_clrstatus(dev):
    try: dev.ctrl_transfer(0x21, DFU_CLRSTATUS, 0, 0, b'', timeout=1000)
    except: pass

def reset_to_idle(dev):
    """Get device back to dfuIDLE."""
    for _ in range(10):
        st = get_dfu_status(dev)
        if st is None: return False
        if st["bState"] == 2: return True  # dfuIDLE
        if st["bState"] == 10:  # dfuERROR
            dfu_clrstatus(dev)
            time.sleep(0.05)
        elif st["bState"] in (5, 3):  # dnIDLE/dnSYNC
            dfu_abort(dev)
            time.sleep(0.05)
        elif st["bState"] == 8:  # MANIFEST-WAIT-RESET
            dfu_abort(dev)
            try: dev.reset()
            except: pass
            time.sleep(2)
            return False
        else:
            dfu_abort(dev)
            time.sleep(0.1)
    return False

def parse_dfu_serial(dev):
    """Parse DFU serial number string for device info.
    Format: CPID:8020 CPRV:11 CPFM:03 SCEP:01 BDID:0C ECID:001C15D43C20802E IBFL:3C SRTG:[iBoot-3865.0.0.4.7]
    Also parse NONC (nonce) and SNON fields.
    """
    info = {}
    try:
        serial = usb.util.get_string(dev, dev.iSerialNumber)
        log(f"DFU Serial: {serial}")
        
        # Parse key:value pairs
        for m in re.finditer(r'(\w+):(\[?[^\s\]]*\]?)', serial):
            key, val = m.group(1), m.group(2).strip('[]')
            info[key] = val
        
        # Also try String descriptor #4 (hidden, has more info)
        try:
            s4 = usb.util.get_string(dev, 4)
            if s4:
                log(f"String #4: {s4}")
                for m in re.finditer(r'(\w+):(\[?[^\s\]]*\]?)', s4):
                    key, val = m.group(1), m.group(2).strip('[]')
                    if key not in info:
                        info[key] = val
        except:
            pass
    except Exception as e:
        log(f"Failed to read serial: {e}", "ERR")
    
    return info

# ============================================================
# IPSW download helpers
# ============================================================
def get_signed_ipsw_info():
    """Query ipsw.me API for currently signed IPSW for iPhone XR."""
    log(f"Querying ipsw.me API for {DEVICE_ID}...")
    
    url = f"{IPSW_API}/device/{DEVICE_ID}?type=ipsw"
    try:
        resp = session.get(url, timeout=30)
        resp.raise_for_status()
        data = resp.json()
    except Exception as e:
        log(f"ipsw.me API error: {e}", "ERR")
        return None
    
    # Find currently signed firmware
    firmwares = data.get("firmwares", [])
    signed = [fw for fw in firmwares if fw.get("signed", False)]
    
    if not signed:
        log("No currently signed IPSW found!", "ERR")
        return None
    
    # Pick the latest signed version
    fw = signed[0]
    log(f"Found signed IPSW: iOS {fw['version']} (build {fw['buildid']})")
    log(f"  URL: {fw['url']}")
    log(f"  Size: {fw['filesize'] / 1e9:.2f} GB")
    log(f"  SHA1: {fw.get('sha1sum', 'N/A')}")
    
    return fw

def download_from_ipsw(ipsw_url, file_path):
    """Download a specific file from an IPSW (ZIP) using HTTP range requests."""
    cache_key = hashlib.md5(f"{ipsw_url}:{file_path}".encode()).hexdigest()
    cache_file = CACHE_DIR / f"{cache_key}_{Path(file_path).name}"
    
    if cache_file.exists():
        log(f"Using cached: {cache_file.name}")
        return cache_file.read_bytes()
    
    log(f"Downloading {file_path} from IPSW...")
    log(f"  (Full IPSW URL: {ipsw_url})")
    
    # For ZIP extraction we need to download the Central Directory first
    # which is at the END of the file. We need the file size.
    
    # HEAD request to get file size
    head = session.head(ipsw_url, allow_redirects=True, timeout=30)
    total_size = int(head.headers.get('Content-Length', 0))
    
    if total_size == 0:
        log("Cannot determine IPSW size", "ERR")
        return None
    
    log(f"  IPSW size: {total_size / 1e9:.2f} GB")
    
    # Download the End of Central Directory (last 65KB should be enough)
    eocd_size = min(65536 + 22, total_size)
    eocd_start = total_size - eocd_size
    
    log(f"  Fetching ZIP central directory ({eocd_size} bytes from end)...")
    resp = session.get(ipsw_url, 
                       headers={"Range": f"bytes={eocd_start}-{total_size-1}"},
                       timeout=60)
    eocd_data = resp.content
    
    # Find End of Central Directory record
    eocd_sig = b'\x50\x4b\x05\x06'
    eocd_pos = eocd_data.rfind(eocd_sig)
    if eocd_pos < 0:
        log("Cannot find ZIP EOCD", "ERR")
        return None
    
    # Parse EOCD
    eocd = eocd_data[eocd_pos:]
    cd_size = struct.unpack_from('<I', eocd, 12)[0]
    cd_offset = struct.unpack_from('<I', eocd, 16)[0]
    
    # Check for ZIP64
    if cd_offset == 0xFFFFFFFF or cd_size == 0xFFFFFFFF:
        log("  ZIP64 detected, looking for ZIP64 EOCD locator...")
        # ZIP64 end of central directory locator is 20 bytes before EOCD
        z64_loc_sig = b'\x50\x4b\x06\x07'
        z64_loc_pos = eocd_data.rfind(z64_loc_sig, 0, eocd_pos)
        if z64_loc_pos >= 0:
            z64_eocd_offset = struct.unpack_from('<Q', eocd_data, z64_loc_pos + 8)[0]
            # Need to download the ZIP64 EOCD record
            z64_resp = session.get(ipsw_url,
                                   headers={"Range": f"bytes={z64_eocd_offset}-{z64_eocd_offset+255}"},
                                   timeout=30)
            z64_eocd = z64_resp.content
            if z64_eocd[:4] == b'\x50\x4b\x06\x06':
                cd_size = struct.unpack_from('<Q', z64_eocd, 40)[0]
                cd_offset = struct.unpack_from('<Q', z64_eocd, 48)[0]
                log(f"  ZIP64 CD: offset={cd_offset}, size={cd_size}")
            else:
                log("  ZIP64 EOCD not found at expected offset", "ERR")
                return None
        else:
            log("  ZIP64 locator not found", "ERR")
            return None
    
    log(f"  Central Directory: offset={cd_offset}, size={cd_size}")
    
    # Download Central Directory
    log(f"  Fetching central directory ({cd_size} bytes)...")
    cd_resp = session.get(ipsw_url,
                          headers={"Range": f"bytes={cd_offset}-{cd_offset+cd_size-1}"},
                          timeout=120)
    cd_data = cd_resp.content
    
    # Parse Central Directory entries to find our file
    target_offset = None
    target_comp_size = None
    target_uncomp_size = None
    target_method = None
    
    pos = 0
    cd_sig = b'\x50\x4b\x01\x02'
    while pos < len(cd_data) - 46:
        if cd_data[pos:pos+4] != cd_sig:
            break
        
        method = struct.unpack_from('<H', cd_data, pos + 10)[0]
        comp_size = struct.unpack_from('<I', cd_data, pos + 20)[0]
        uncomp_size = struct.unpack_from('<I', cd_data, pos + 24)[0]
        name_len = struct.unpack_from('<H', cd_data, pos + 28)[0]
        extra_len = struct.unpack_from('<H', cd_data, pos + 30)[0]
        comment_len = struct.unpack_from('<H', cd_data, pos + 32)[0]
        local_offset = struct.unpack_from('<I', cd_data, pos + 42)[0]
        
        name = cd_data[pos+46:pos+46+name_len].decode('utf-8', errors='replace')
        
        # Handle ZIP64 extra fields
        if comp_size == 0xFFFFFFFF or uncomp_size == 0xFFFFFFFF or local_offset == 0xFFFFFFFF:
            extra = cd_data[pos+46+name_len:pos+46+name_len+extra_len]
            epos = 0
            while epos < len(extra) - 4:
                eid = struct.unpack_from('<H', extra, epos)[0]
                esz = struct.unpack_from('<H', extra, epos + 2)[0]
                if eid == 0x0001:  # ZIP64
                    idx = epos + 4
                    if uncomp_size == 0xFFFFFFFF:
                        uncomp_size = struct.unpack_from('<Q', extra, idx)[0]
                        idx += 8
                    if comp_size == 0xFFFFFFFF:
                        comp_size = struct.unpack_from('<Q', extra, idx)[0]
                        idx += 8
                    if local_offset == 0xFFFFFFFF:
                        local_offset = struct.unpack_from('<Q', extra, idx)[0]
                    break
                epos += 4 + esz
        
        # Check if this is our file (case-insensitive, partial match)
        if file_path.lower() in name.lower() or name.lower().endswith(file_path.lower()):
            log(f"  Found: {name} (comp={comp_size}, uncomp={uncomp_size}, method={method})")
            target_offset = local_offset
            target_comp_size = comp_size
            target_uncomp_size = uncomp_size
            target_method = method
            break
        
        pos += 46 + name_len + extra_len + comment_len
    
    if target_offset is None:
        # Try broader search
        log(f"  File '{file_path}' not found. Listing matching entries...")
        pos = 0
        matches = []
        while pos < len(cd_data) - 46:
            if cd_data[pos:pos+4] != cd_sig:
                break
            name_len = struct.unpack_from('<H', cd_data, pos + 28)[0]
            extra_len = struct.unpack_from('<H', cd_data, pos + 30)[0]
            comment_len = struct.unpack_from('<H', cd_data, pos + 32)[0]
            name = cd_data[pos+46:pos+46+name_len].decode('utf-8', errors='replace')
            
            keywords = file_path.lower().split('/')
            if any(kw in name.lower() for kw in keywords if len(kw) > 2):
                matches.append(name)
            pos += 46 + name_len + extra_len + comment_len
        
        for m in matches[:20]:
            log(f"    {m}")
        return None
    
    # Download the local file header + data
    # Local file header is at least 30 bytes + name_len + extra_len
    header_size = 30 + 256  # generous header read
    
    log(f"  Fetching file data ({target_comp_size} bytes)...")
    data_resp = session.get(ipsw_url,
                            headers={"Range": f"bytes={target_offset}-{target_offset + header_size + target_comp_size - 1}"},
                            timeout=300,
                            stream=True)
    
    file_data = data_resp.content
    
    # Parse local file header to find actual data start
    if file_data[:4] != b'\x50\x4b\x03\x04':
        log("Invalid local file header", "ERR")
        return None
    
    local_name_len = struct.unpack_from('<H', file_data, 26)[0]
    local_extra_len = struct.unpack_from('<H', file_data, 28)[0]
    data_start = 30 + local_name_len + local_extra_len
    
    compressed_data = file_data[data_start:data_start + target_comp_size]
    
    if target_method == 0:  # Stored
        result = compressed_data
    elif target_method == 8:  # Deflated
        import zlib
        result = zlib.decompress(compressed_data, -15)
    else:
        log(f"Unknown compression method: {target_method}", "ERR")
        return None
    
    if len(result) != target_uncomp_size:
        log(f"Size mismatch: got {len(result)}, expected {target_uncomp_size}", "WARN")
    
    # Cache
    cache_file.write_bytes(result)
    log(f"  Cached: {cache_file.name} ({len(result)} bytes)")
    
    return result

def list_ipsw_contents(ipsw_url, pattern=""):
    """List files in an IPSW matching a pattern."""
    log(f"Listing IPSW contents matching '{pattern}'...")
    
    # Get file size
    head = session.head(ipsw_url, allow_redirects=True, timeout=30)
    total_size = int(head.headers.get('Content-Length', 0))
    
    # Download EOCD
    eocd_size = min(65536 + 22, total_size)
    eocd_start = total_size - eocd_size
    resp = session.get(ipsw_url,
                       headers={"Range": f"bytes={eocd_start}-{total_size-1}"},
                       timeout=60)
    eocd_data = resp.content
    
    eocd_sig = b'\x50\x4b\x05\x06'
    eocd_pos = eocd_data.rfind(eocd_sig)
    if eocd_pos < 0:
        return []
    
    cd_size = struct.unpack_from('<I', eocd_data, eocd_pos + 12)[0]
    cd_offset = struct.unpack_from('<I', eocd_data, eocd_pos + 16)[0]
    
    if cd_offset == 0xFFFFFFFF or cd_size == 0xFFFFFFFF:
        z64_loc_sig = b'\x50\x4b\x06\x07'
        z64_loc_pos = eocd_data.rfind(z64_loc_sig, 0, eocd_pos)
        if z64_loc_pos >= 0:
            z64_eocd_offset = struct.unpack_from('<Q', eocd_data, z64_loc_pos + 8)[0]
            z64_resp = session.get(ipsw_url,
                                   headers={"Range": f"bytes={z64_eocd_offset}-{z64_eocd_offset+255}"},
                                   timeout=30)
            z64_eocd = z64_resp.content
            if z64_eocd[:4] == b'\x50\x4b\x06\x06':
                cd_size = struct.unpack_from('<Q', z64_eocd, 40)[0]
                cd_offset = struct.unpack_from('<Q', z64_eocd, 48)[0]
    
    cd_resp = session.get(ipsw_url,
                          headers={"Range": f"bytes={cd_offset}-{cd_offset+cd_size-1}"},
                          timeout=120)
    cd_data = cd_resp.content
    
    files = []
    pos = 0
    cd_sig = b'\x50\x4b\x01\x02'
    while pos < len(cd_data) - 46:
        if cd_data[pos:pos+4] != cd_sig:
            break
        comp_size = struct.unpack_from('<I', cd_data, pos + 20)[0]
        uncomp_size = struct.unpack_from('<I', cd_data, pos + 24)[0]
        name_len = struct.unpack_from('<H', cd_data, pos + 28)[0]
        extra_len = struct.unpack_from('<H', cd_data, pos + 30)[0]
        comment_len = struct.unpack_from('<H', cd_data, pos + 32)[0]
        name = cd_data[pos+46:pos+46+name_len].decode('utf-8', errors='replace')
        
        if not pattern or pattern.lower() in name.lower():
            files.append((name, uncomp_size))
        
        pos += 46 + name_len + extra_len + comment_len
    
    return files

# ============================================================
# TSS (Tatsu Signing Server) helpers
# ============================================================
def build_tss_request(device_info, manifest_identity, component_name="iBSS"):
    """Build a TSS request plist from device info and BuildManifest identity.
    
    Based on idevicerestore's tss_request_new / tss_request_add_ap_img4_tags.
    """
    
    ecid_hex = device_info.get("ECID", "")
    ecid_int = int(ecid_hex, 16) if ecid_hex else 0
    
    cpid_hex = device_info.get("CPID", "8020")
    cpid_int = int(cpid_hex, 16) if cpid_hex else 0x8020
    
    bdid_hex = device_info.get("BDID", "0C")
    bdid_int = int(bdid_hex, 16) if bdid_hex else 0x0C
    
    # A12 uses 32-byte nonces (SHA256-based)
    nonce_hex = device_info.get("NONC", "")
    if nonce_hex:
        nonce_bytes = bytes.fromhex(nonce_hex)
    else:
        nonce_bytes = b'\x00' * 32  # 32 bytes for A12+
    
    sep_nonce_hex = device_info.get("SNON", "")
    if sep_nonce_hex:
        sep_nonce = bytes.fromhex(sep_nonce_hex)
    else:
        sep_nonce = b'\x00' * 32
    
    # Core AP IMG4 tags (idevicerestore: tss_request_add_ap_img4_tags)
    req = {
        "@HostPlatformInfo": "mac",
        "@VersionInfo": "libaple-565.40.29",
        "@UUID": str(uuid.uuid4()).upper(),
        "ApBoardID": bdid_int,
        "ApChipID": cpid_int,
        "ApECID": ecid_int,
        "ApNonce": nonce_bytes,
        "ApProductionMode": True,
        "ApSecurityDomain": 1,
        "ApSecurityMode": True,
        "ApSupportsImg4": True,
    }
    
    # Add the component we want signed from the manifest identity
    if manifest_identity:
        manifest_props = manifest_identity.get("Manifest", {})
        
        # Add identity-level fields that TSS needs
        if "UniqueBuildID" in manifest_identity:
            req["UniqueBuildID"] = manifest_identity["UniqueBuildID"]
        
        # Add the component entry with ONLY TSS-required fields
        # TSS expects: Digest, Trusted, and applied rule tags (EPRO, ESEC, DPRO)
        # TSS does NOT want: Path, IsFTAB, IsLoadedByiBoot, Personalize, etc.
        if component_name in manifest_props:
            comp = manifest_props[component_name]
            comp_req = {}
            
            # Copy Digest - REQUIRED
            if "Digest" in comp:
                comp_req["Digest"] = comp["Digest"]
            
            # Copy Trusted flag - REQUIRED
            if "Trusted" in comp:
                comp_req["Trusted"] = comp["Trusted"]
            
            # Apply RestoreRequestRules to generate personalization tags
            # For a production device in DFU:
            # - ApRawProductionMode = True → DPRO = True
            # - ApCurrentProductionMode = True, ApRequiresImage4 = True → EPRO = True
            # - ApRawSecurityMode = True, ApRequiresImage4 = True → ESEC = True
            rules = comp.get("Info", {}).get("RestoreRequestRules", [])
            for rule in rules:
                conditions = rule.get("Conditions", {})
                actions = rule.get("Actions", {})
                
                # Check if conditions match our device state
                match = True
                for cond_key, cond_val in conditions.items():
                    if cond_key == "ApRawProductionMode" and cond_val != True:
                        match = False
                    elif cond_key == "ApCurrentProductionMode" and cond_val != True:
                        match = False
                    elif cond_key == "ApRequiresImage4" and cond_val != True:
                        match = False
                    elif cond_key == "ApRawSecurityMode" and cond_val != True:
                        match = False
                    elif cond_key == "ApDemotionPolicyOverride":
                        match = False  # Not demoted
                    elif cond_key == "ApInRomDFU":
                        # We ARE in ROM DFU mode
                        if cond_val != True:
                            match = False
                
                if match:
                    for act_key, act_val in actions.items():
                        comp_req[act_key] = act_val
                        log(f"  Applied rule: {act_key}={act_val}")
            
            req[component_name] = comp_req
            log(f"TSS request includes component: {component_name}")
            log(f"  Component tags: {list(comp_req.keys())}")
        else:
            log(f"Component '{component_name}' not in manifest!", "ERR")
            available = list(manifest_props.keys())
            log(f"Available: {available[:20]}")
    
    return req

def send_tss_request(tss_plist):
    """Send TSS request to Apple and get the response."""
    log("Sending TSS request to Apple...")
    
    # Serialize to XML plist
    plist_data = plistlib.dumps(tss_plist, fmt=plistlib.FMT_XML)
    
    # Debug: save request
    req_path = CACHE_DIR / "tss_request.plist"
    req_path.write_bytes(plist_data)
    log(f"  Saved request to {req_path}")
    
    headers = {
        "Content-Type": "text/xml; charset=utf-8",
        "User-Agent": "InetURL/1.0",
        "Expect": "",
    }
    
    try:
        resp = session.post(TSS_URL, data=plist_data, headers=headers, timeout=30)
        log(f"  TSS response: HTTP {resp.status_code}")
        log(f"  Response headers: {dict(resp.headers)}")
        
        # Save raw response
        resp_path = CACHE_DIR / "tss_response.raw"
        resp_path.write_bytes(resp.content)
        
        if resp.status_code == 200:
            # Parse response
            body = resp.content
            
            # Apple TSS returns: STATUS=0&REQUEST_STRING=<plist>...</plist>
            # or: STATUS=X&MESSAGE=error message
            body_str = body.decode('utf-8', errors='replace')
            
            if "STATUS=0" in body_str:
                log("  TSS: SUCCESS (STATUS=0)")
                # Extract the plist portion
                plist_start = body_str.find("<?xml")
                if plist_start >= 0:
                    plist_str = body_str[plist_start:]
                    ticket = plistlib.loads(plist_str.encode('utf-8'))
                    # Save
                    ticket_path = CACHE_DIR / "tss_ticket.plist"
                    with open(ticket_path, 'wb') as f:
                        plistlib.dump(ticket, f)
                    log(f"  Saved ticket to {ticket_path}")
                    return ticket
                else:
                    log(f"  No plist in response: {body_str[:500]}", "ERR")
            else:
                log(f"  TSS error: {body_str[:500]}", "ERR")
                # Parse status
                status_match = re.search(r'STATUS=(\d+)', body_str)
                msg_match = re.search(r'MESSAGE=(.*?)(&|$)', body_str)
                if status_match:
                    log(f"  Status code: {status_match.group(1)}")
                if msg_match:
                    log(f"  Message: {msg_match.group(1)}")
        else:
            log(f"  HTTP error: {resp.status_code} {resp.text[:500]}", "ERR")
            
    except Exception as e:
        log(f"  TSS request failed: {e}", "ERR")
    
    return None

# ============================================================
# IMG4 container builder (minimal, for stitching iBSS + APTicket)
# ============================================================
def der_length(length):
    """Encode DER length."""
    if length < 0x80:
        return bytes([length])
    elif length < 0x100:
        return bytes([0x81, length])
    elif length < 0x10000:
        return bytes([0x82, (length >> 8) & 0xFF, length & 0xFF])
    elif length < 0x1000000:
        return bytes([0x83, (length >> 16) & 0xFF, (length >> 8) & 0xFF, length & 0xFF])
    else:
        return bytes([0x84, (length >> 24) & 0xFF, (length >> 16) & 0xFF,
                     (length >> 8) & 0xFF, length & 0xFF])

def der_sequence(content):
    return b'\x30' + der_length(len(content)) + content

def der_ia5string(s):
    data = s.encode('ascii')
    return b'\x16' + der_length(len(data)) + data

def der_octetstring(data):
    return b'\x04' + der_length(len(data)) + data

def der_integer(val):
    if val == 0:
        return b'\x02\x01\x00'
    raw = val.to_bytes((val.bit_length() + 8) // 8, 'big', signed=(val < 0))
    return b'\x02' + der_length(len(raw)) + raw

def build_img4(im4p_data, im4m_data=None):
    """Build an IMG4 DER container from IM4P payload and optional IM4M manifest."""
    content = der_ia5string("IMG4") + im4p_data
    if im4m_data:
        # IM4M goes in a context-specific [0] CONSTRUCTED tag
        content += b'\xa0' + der_length(len(im4m_data)) + im4m_data
    return der_sequence(content)

def extract_img4_components(data):
    """Try to parse raw firmware data as IMG4/IM4P.
    
    Apple firmware files can be:
    1. Raw IMG4 container (SEQUENCE { IA5 "IMG4", IM4P, [0] IM4M })
    2. Just IM4P (SEQUENCE { IA5 "IM4P", type, version, OCTET STRING })
    3. Raw binary (Mach-O or other)
    """
    info = {"format": "unknown", "size": len(data)}
    
    if len(data) < 4:
        return info
    
    # Check for DER SEQUENCE
    if data[0] == 0x30:
        # Try to find magic strings
        if b'IMG4' in data[:50]:
            info["format"] = "IMG4"
        elif b'IM4P' in data[:50]:
            info["format"] = "IM4P"
            # The IM4P is what we need — it's already the signed payload envelope
        elif b'IM4M' in data[:50]:
            info["format"] = "IM4M"
    elif data[:4] == b'\xfe\xed\xfa\xcf':
        info["format"] = "MachO-64"
    elif data[:4] == b'\xca\xfe\xba\xbe':
        info["format"] = "MachO-FAT"
    
    # First 32 bytes hex
    info["header_hex"] = data[:32].hex()
    
    return info

# ============================================================
# DFU send firmware
# ============================================================
def send_firmware_via_dfu(dev, firmware_data):
    """Send firmware data via DFU DNLOAD in correct block sequence."""
    BLOCK_SIZE = 2048  # Stay under the 2048B boundary (SecureROM's buffer size)
    
    log(f"Sending firmware ({len(firmware_data)} bytes) via DFU DNLOAD...")
    
    if not reset_to_idle(dev):
        log("Cannot reach dfuIDLE", "ERR")
        return False
    
    block = 0
    offset = 0
    
    while offset < len(firmware_data):
        chunk = firmware_data[offset:offset + BLOCK_SIZE]
        
        if not dfu_dnload(dev, chunk, block=block, timeout=10000):
            log(f"DNLOAD block {block} failed at offset {offset}", "ERR")
            return False
        
        # Wait for GETSTATUS
        st = get_dfu_status(dev)
        if st is None:
            log(f"Lost device after block {block}", "ERR")
            return False
        
        if st["bState"] == 10:  # dfuERROR
            log(f"DFU error at block {block}: status={st['bStatus']}", "ERR")
            return False
        
        # Wait for poll timeout
        poll = st["poll_ms"]
        if poll > 0:
            time.sleep(poll / 1000.0 + 0.01)
        
        # Wait until we're in DNLOAD-IDLE
        for _ in range(20):
            st = get_dfu_status(dev)
            if st and st["bState"] == 5:  # dnIDLE
                break
            if st and st["bState"] == 10:  # error
                log(f"DFU error during block {block}: status={st['bStatus']}", "ERR")
                return False
            time.sleep(0.05)
        
        offset += BLOCK_SIZE
        block += 1
        
        if block % 50 == 0:
            log(f"  Sent {offset}/{len(firmware_data)} bytes ({100*offset/len(firmware_data):.0f}%)")
    
    log(f"  Sent {block} blocks ({offset} bytes total)")
    
    # Trigger manifest: zero-length DNLOAD
    log("Triggering manifest (zero-length DNLOAD)...")
    if not dfu_dnload(dev, b'', block=block, timeout=10000):
        log("Manifest trigger failed", "ERR")
        return False
    
    # Wait for manifest processing
    t0 = time.perf_counter()
    
    st = get_dfu_status(dev)
    log(f"  After manifest trigger: state={st['bState'] if st else '?'}, poll={st['poll_ms'] if st else '?'}ms")
    
    if st and st["bState"] == 6:  # MANIFEST-SYNC
        time.sleep(max(0.01, st["poll_ms"] / 1000.0) + 0.01)
        st = get_dfu_status(dev)
        log(f"  Manifest state: state={st['bState'] if st else '?'}, poll={st['poll_ms'] if st else '?'}ms")
    
    if st and st["bState"] == 7:  # dfuMANIFEST
        manifest_poll = st["poll_ms"]
        log(f"  Manifest processing... (poll={manifest_poll}ms)")
        # Wait for manifest to complete
        time.sleep(max(0.5, manifest_poll / 1000.0) + 1.0)
    
    elapsed_ms = (time.perf_counter() - t0) * 1000
    
    # Check final state
    st = get_dfu_status(dev)
    if st:
        log(f"  Final state: {st['bState']} (elapsed={elapsed_ms:.0f}ms)")
        if st["bState"] == 8:  # MANIFEST-WAIT-RESET
            log("  Device in MANIFEST-WAIT-RESET — firmware was processed")
            log("  Checking if device re-enumerates as Recovery mode...")
            return True
        elif st["bState"] == 10:  # ERROR
            log(f"  Manifest ERROR — firmware rejected (status={st['bStatus']})", "WARN")
            return False
        elif st["bState"] == 2:  # dfuIDLE
            log("  Back to dfuIDLE — something went wrong", "WARN")
            return False
    else:
        log(f"  Device disappeared after manifest (elapsed={elapsed_ms:.0f}ms)")
        # This could mean the firmware was accepted and device is rebooting!
        log("  Waiting for Recovery mode device...")
    
    return True

def wait_for_recovery(timeout=15):
    """Wait for device to re-enumerate as Recovery mode."""
    log(f"Waiting up to {timeout}s for Recovery mode (PID=0x{RECOVERY_PID:04X})...")
    
    t0 = time.time()
    while time.time() - t0 < timeout:
        dev = find_recovery()
        if dev:
            log("*** RECOVERY MODE DEVICE FOUND! ***", "FIND")
            try:
                serial = usb.util.get_string(dev, dev.iSerialNumber)
                log(f"  Serial: {serial}")
            except:
                pass
            
            # List interfaces
            for cfg in dev:
                for intf in cfg:
                    log(f"  Interface {intf.bInterfaceNumber}: class={intf.bInterfaceClass:#x}, "
                        f"subclass={intf.bInterfaceSubClass:#x}, protocol={intf.bInterfaceProtocol:#x}")
                    for ep in intf:
                        direction = "IN" if ep.bEndpointAddress & 0x80 else "OUT"
                        log(f"    EP {ep.bEndpointAddress:#x} {direction} maxPacket={ep.wMaxPacketSize}")
            
            return dev
        
        # Also check if device is still in DFU
        dfu = find_dfu()
        if dfu:
            st = get_dfu_status(dfu)
            if st and st["bState"] == 8:
                # Still in MANIFEST-WAIT-RESET, give it more time
                pass
            elif st and st["bState"] == 2:
                log("  Device back in dfuIDLE — firmware not accepted", "WARN")
                return None
        
        time.sleep(0.5)
    
    log("  Timeout — no Recovery mode device appeared", "WARN")
    return None

# ============================================================
# MAIN
# ============================================================
def main():
    log("=" * 70)
    log("TSS iBSS LOADER — Boot iPhone XR to iBoot Recovery")
    log("=" * 70)

    # ---- Step 1: Connect to DFU device ----
    log("\n[Step 1] Connecting to DFU device...")
    dev = find_dfu()
    if not dev:
        log("No DFU device found! Is the iPhone in DFU mode?", "ERR")
        sys.exit(1)
    
    device_info = parse_dfu_serial(dev)
    log(f"  CPID: {device_info.get('CPID', '?')}")
    log(f"  BDID: {device_info.get('BDID', '?')}")
    log(f"  ECID: {device_info.get('ECID', '?')}")
    log(f"  NONC: {device_info.get('NONC', '?')}")
    log(f"  SNON: {device_info.get('SNON', '?')}")
    log(f"  SRTG: {device_info.get('SRTG', '?')}")
    
    if device_info.get('CPID', '') != '8020':
        log(f"WARNING: Expected CPID 8020 (A12), got {device_info.get('CPID', '?')}", "WARN")
    
    # ---- Step 2: Find signed IPSW ----
    log("\n[Step 2] Finding currently signed IPSW...")
    fw = get_signed_ipsw_info()
    if not fw:
        log("No signed IPSW found!", "ERR")
        sys.exit(1)
    
    ipsw_url = fw["url"]
    ios_version = fw["version"]
    build_id = fw["buildid"]

    # ---- Step 3: Download BuildManifest.plist ----
    log("\n[Step 3] Downloading BuildManifest.plist...")
    manifest_data = download_from_ipsw(ipsw_url, "BuildManifest.plist")
    if not manifest_data:
        log("Failed to download BuildManifest.plist!", "ERR")
        sys.exit(1)
    
    manifest = plistlib.loads(manifest_data)
    log(f"  Build: {manifest.get('ProductBuildVersion', '?')}")
    log(f"  Version: {manifest.get('ProductVersion', '?')}")
    
    # Find the correct identity for our device
    identities = manifest.get("BuildIdentities", [])
    log(f"  {len(identities)} build identities found")
    
    target_identity = None
    for ident in identities:
        info = ident.get("Info", {})
        # Match: RestoreBehavior = Erase or Update, DeviceClass matches
        device_class = info.get("DeviceClass", "")
        restore_behavior = info.get("RestoreBehavior", "")
        
        # Check if iBSS is present
        manifest_entries = ident.get("Manifest", {})
        has_ibss = "iBSS" in manifest_entries
        
        variant = info.get("Variant", "")
        
        log(f"  Identity: class={device_class}, behavior={restore_behavior}, "
            f"variant={variant[:50]}, has_iBSS={has_ibss}")
        
        if has_ibss and restore_behavior in ("Erase", ""):
            if target_identity is None:
                target_identity = ident
                log(f"  >>> Selected this identity")
    
    if not target_identity:
        log("No suitable build identity found!", "ERR")
        # Try any identity with iBSS
        for ident in identities:
            if "iBSS" in ident.get("Manifest", {}):
                target_identity = ident
                log("  Using first identity with iBSS")
                break
    
    if not target_identity:
        log("No identity with iBSS component!", "ERR")
        sys.exit(1)
    
    # ---- Step 4: Download iBSS ----
    log("\n[Step 4] Downloading iBSS from IPSW...")
    ibss_manifest = target_identity["Manifest"]["iBSS"]
    ibss_info = ibss_manifest.get("Info", {})
    ibss_path = ibss_info.get("Path", "")
    
    log(f"  iBSS path in IPSW: {ibss_path}")
    
    if not ibss_path:
        log("No iBSS path in manifest!", "ERR")
        # Try to find it by listing
        files = list_ipsw_contents(ipsw_url, "iBSS")
        sys.exit(1)
    
    ibss_data = download_from_ipsw(ipsw_url, ibss_path)
    if not ibss_data:
        log("Failed to download iBSS!", "ERR")
        sys.exit(1)
    
    log(f"  iBSS size: {len(ibss_data)} bytes")
    
    # Analyze format
    ibss_info_parsed = extract_img4_components(ibss_data)
    log(f"  Format: {ibss_info_parsed['format']}")
    log(f"  Header: {ibss_info_parsed['header_hex']}")
    
    # ---- Step 5: TSS request ----
    log("\n[Step 5] Building and sending TSS request...")
    tss_req = build_tss_request(device_info, target_identity, "iBSS")
    
    # Debug: print key fields
    log(f"  ApChipID: {tss_req.get('ApChipID', '?'):#x}")
    log(f"  ApBoardID: {tss_req.get('ApBoardID', '?'):#x}")
    log(f"  ApECID: {tss_req.get('ApECID', '?'):#x}")
    log(f"  ApNonce: {tss_req.get('ApNonce', b'').hex()}")
    
    ticket = send_tss_request(tss_req)
    
    if not ticket:
        log("\n*** TSS request failed ***", "ERR")
        log("Possible reasons:")
        log("  - Nonce mismatch (device nonce not in TSS request)")
        log("  - ECID format issue")
        log("  - Firmware no longer signed")
        log("  - TSS request format incorrect")
        log("\nTrying alternative: send raw iBSS without ticket...")
        log("(This will be rejected by SecureROM but we can observe the error)")
        
        # Send raw iBSS anyway (will fail signature check but useful to see behavior)
        log("\nSending raw iBSS (IM4P, no APTicket) to observe ROM behavior...")
        dev = find_dfu()
        if dev:
            result = send_firmware_via_dfu(dev, ibss_data)
            if result:
                rec = wait_for_recovery(timeout=10)
                if rec:
                    log("!!! RECOVERY MODE REACHED !!!")
                else:
                    log("No recovery mode — firmware rejected as expected")
        
        # Save what we have for analysis
        out = {
            "device_info": device_info,
            "ipsw_version": ios_version,
            "ipsw_build": build_id,
            "ibss_size": len(ibss_data),
            "ibss_format": ibss_info_parsed,
            "tss_failed": True,
            "timestamp": datetime.now().isoformat()
        }
        json_path = RESULTS_DIR / "tss_ibss_result.json"
        with open(json_path, 'w') as f:
            json.dump(out, f, indent=2, default=str)
        log(f"\nSaved: {json_path}")
        
        sys.exit(1)
    
    log("TSS ticket received!")
    
    # ---- Step 6: Stitch iBSS + APTicket ----
    log("\n[Step 6] Stitching iBSS + APTicket...")
    
    # The iBSS from IPSW is already in IM4P format usually.
    # The APTicket from TSS is the IM4M manifest.
    # We need to wrap as: IMG4 { IM4P, [0] IM4M }
    
    # Check if iBSS is already IMG4 or IM4P
    if ibss_info_parsed["format"] == "IMG4":
        log("  iBSS is already IMG4 — need to extract IM4P and re-wrap with new IM4M")
        # For now, send as-is with the ticket somehow...
        # Actually, the iBSS from IPSW should be IM4P, not full IMG4
        firmware = ibss_data
    elif ibss_info_parsed["format"] == "IM4P":
        log("  iBSS is IM4P — wrapping with APTicket as IMG4")
        # Extract the APTicket blob
        ap_ticket = ticket.get("ApImg4Ticket", b'')
        if not ap_ticket:
            # Try other key names
            for key in ["APTicket", "ApImg4Ticket", "ap-ticket"]:
                if key in ticket:
                    ap_ticket = ticket[key]
                    break
        
        if ap_ticket:
            log(f"  APTicket: {len(ap_ticket)} bytes")
            firmware = build_img4(ibss_data, ap_ticket)
            log(f"  IMG4 container: {len(firmware)} bytes")
        else:
            log("  No APTicket in TSS response! Sending IM4P only...")
            log(f"  TSS response keys: {list(ticket.keys())}")
            firmware = ibss_data
    else:
        log(f"  Unknown format: {ibss_info_parsed['format']} — sending raw")
        firmware = ibss_data
    
    # Save firmware for debugging
    fw_path = CACHE_DIR / "ibss_signed.img4"
    fw_path.write_bytes(firmware)
    log(f"  Saved signed firmware: {fw_path} ({len(firmware)} bytes)")
    
    # ---- Step 7: Send to device ----
    log("\n[Step 7] Sending signed iBSS to device via DFU...")
    dev = find_dfu()
    if not dev:
        log("Lost DFU device!", "ERR")
        sys.exit(1)
    
    result = send_firmware_via_dfu(dev, firmware)
    
    # ---- Step 8: Check for Recovery mode ----
    log("\n[Step 8] Checking for iBoot Recovery mode...")
    
    if result:
        rec = wait_for_recovery(timeout=15)
        if rec:
            log("\n" + "=" * 70)
            log("*** SUCCESS: iPHONE BOOTED TO iBOOT RECOVERY MODE ***")
            log("=" * 70)
            log("The device is now running iBoot with a rich USB surface:")
            log("  - BULK IN/OUT endpoints for data transfer")
            log("  - Text command interface (getenv, setenv, bgcolor, reboot)")
            log("  - Recovery protocol for further firmware loading")
            
            # Save success result
            out = {
                "success": True,
                "device_info": device_info,
                "ipsw_version": ios_version,
                "ipsw_build": build_id,
                "firmware_size": len(firmware),
                "timestamp": datetime.now().isoformat()
            }
        else:
            log("\nDevice did not enter Recovery mode")
            log("Checking if still in DFU...")
            dfu = find_dfu()
            if dfu:
                st = get_dfu_status(dfu)
                log(f"  Still in DFU, state={st['bState'] if st else '?'}")
            else:
                log("  Device not found in DFU either — may need force restart")
            
            out = {
                "success": False,
                "device_info": device_info,
                "ipsw_version": ios_version,
                "firmware_size": len(firmware),
                "timestamp": datetime.now().isoformat()
            }
    else:
        log("Firmware send failed")
        out = {
            "success": False,
            "error": "send_failed",
            "device_info": device_info,
            "timestamp": datetime.now().isoformat()
        }
    
    # Save results
    json_path = RESULTS_DIR / "tss_ibss_result.json"
    with open(json_path, 'w') as f:
        json.dump(out, f, indent=2, default=str)
    log(f"\nSaved: {json_path}")
    
    # Save log
    log_path = RESULTS_DIR / "tss_ibss_log.txt"
    with open(log_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(log_lines))
    log(f"Saved: {log_path}")

if __name__ == "__main__":
    main()
