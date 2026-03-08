#!/usr/bin/env python3
"""
TrustOS IPSW Analyst — Phase 1: Kernelcache Fetcher
Downloads only the kernelcache from an IPSW without downloading the full 6GB file.
Uses Apple's public IPSW API + partial ZIP extraction via HTTP Range requests.

Usage:
    python ipsw_fetch.py                          # Interactive device selection
    python ipsw_fetch.py --device iPhone12,3      # iPhone 11 Pro specific
    python ipsw_fetch.py --url <ipsw_url>         # Direct IPSW URL
"""

import os
import sys
import json
import struct
import hashlib
import argparse
import plistlib
from pathlib import Path
from io import BytesIO

try:
    import requests
except ImportError:
    print("[!] pip install requests")
    sys.exit(1)

try:
    import remotezip
except ImportError:
    print("[!] pip install remotezip")
    sys.exit(1)

# --- Constants ---
IPSW_API = "https://api.ipsw.me/v4"
OUTPUT_DIR = Path(__file__).parent / "extracted"
DEVICES_OF_INTEREST = {
    "iPhone12,3": "iPhone 11 Pro",
    "iPhone12,1": "iPhone 11",
    "iPhone12,5": "iPhone 11 Pro Max",
    "iPhone11,8": "iPhone XR",
    "iPhone10,6": "iPhone X (GSM)",
    "iPhone10,3": "iPhone X (Global)",
    "iPhone10,4": "iPhone 8 (GSM)",
    "iPhone10,1": "iPhone 8 (Global)",
    "iPhone9,3":  "iPhone 7 (GSM)",
    "iPhone9,1":  "iPhone 7 (Global)",
}

# SoC info for security analysis
SOC_INFO = {
    "iPhone12,3": {"soc": "A13", "codename": "T8030", "checkm8": False, "pac": "v1", "mte": False},
    "iPhone12,1": {"soc": "A13", "codename": "T8030", "checkm8": False, "pac": "v1", "mte": False},
    "iPhone11,8": {"soc": "A12", "codename": "T8020", "checkm8": False, "pac": "v1", "mte": False},
    "iPhone10,6": {"soc": "A11", "codename": "T8015", "checkm8": True,  "pac": None, "mte": False},
    "iPhone10,3": {"soc": "A11", "codename": "T8015", "checkm8": True,  "pac": None, "mte": False},
    "iPhone9,3":  {"soc": "A10", "codename": "T8010", "checkm8": True,  "pac": None, "mte": False},
    "iPhone9,1":  {"soc": "A10", "codename": "T8010", "checkm8": True,  "pac": None, "mte": False},
}


def get_firmware_info(device_id: str, version: str = None) -> dict:
    """Fetch firmware info from ipsw.me API"""
    print(f"[*] Querying IPSW API for {device_id}...")
    url = f"{IPSW_API}/device/{device_id}?type=ipsw"
    
    try:
        resp = requests.get(url, timeout=30)
        resp.raise_for_status()
        data = resp.json()
    except Exception as e:
        print(f"[!] API error: {e}")
        return None
    
    firmwares = data.get("firmwares", [])
    if not firmwares:
        print(f"[!] No firmwares found for {device_id}")
        return None
    
    if version:
        # Find specific version
        for fw in firmwares:
            if fw.get("version", "").startswith(version):
                return fw
        print(f"[!] Version {version} not found. Available versions:")
        for fw in firmwares[:10]:
            signed = "SIGNED" if fw.get("signed", False) else "unsigned"
            print(f"    {fw['version']} (build {fw['buildid']}) [{signed}]")
        return None
    else:
        # Return latest
        return firmwares[0]


def list_ipsw_contents(ipsw_url: str) -> list:
    """List files inside the IPSW ZIP without downloading it"""
    print(f"[*] Reading IPSW ZIP directory (remote)...")
    try:
        with remotezip.RemoteZip(ipsw_url) as rz:
            files = rz.namelist()
            return files
    except Exception as e:
        print(f"[!] Failed to read remote ZIP: {e}")
        return []


def find_kernelcache(files: list) -> list:
    """Find kernelcache files in the IPSW file list"""
    candidates = []
    for f in files:
        fname = f.lower()
        if "kernelcache" in fname:
            candidates.append(f)
        elif "kernel" in fname and fname.endswith(('.im4p', '.img4')):
            candidates.append(f)
    return candidates


def find_build_manifest(files: list) -> str:
    """Find BuildManifest.plist"""
    for f in files:
        if f.lower() == "buildmanifest.plist":
            return f
    return None


def find_restore_plist(files: list) -> str:
    """Find Restore.plist"""
    for f in files:
        if f.lower() == "restore.plist":
            return f
    return None


def download_file_from_ipsw(ipsw_url: str, filename: str, output_path: Path) -> bool:
    """Download a single file from the IPSW using HTTP Range"""
    print(f"[*] Extracting: {filename}")
    try:
        with remotezip.RemoteZip(ipsw_url) as rz:
            data = rz.read(filename)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_bytes(data)
            size_mb = len(data) / (1024 * 1024)
            print(f"    -> {output_path} ({size_mb:.1f} MB)")
            return True
    except Exception as e:
        print(f"[!] Download failed: {e}")
        return False


def parse_img4(data: bytes) -> dict:
    """Parse IMG4/IM4P container to extract payload info"""
    result = {"format": "unknown", "type": None, "description": None, "payload_offset": 0, "payload_size": 0}
    
    # Check for IMG4 magic
    if data[:4] == b'\x30\x82':
        # ASN.1 DER sequence - likely IMG4 or IM4P
        # Look for IM4P tag
        im4p_offset = data.find(b'IM4P')
        if im4p_offset >= 0:
            result["format"] = "IM4P"
            # Extract fourcc type (4 bytes after IM4P + length)
            type_offset = im4p_offset + 4
            # Skip ASN.1 overhead to find the type string
            search = data[im4p_offset:im4p_offset+64]
            # Look for known types
            for tag in [b'krnl', b'rkrn', b'rtsc', b'ibss', b'ibec', b'ibot']:
                pos = search.find(tag)
                if pos >= 0:
                    result["type"] = tag.decode('ascii')
                    break
        
        img4_offset = data.find(b'IMG4')
        if img4_offset >= 0:
            result["format"] = "IMG4"
    
    elif data[:4] == b'\xfe\xed\xfa\xce' or data[:4] == b'\xce\xfa\xed\xfe':
        result["format"] = "MachO"
    elif data[:4] == b'\xca\xfe\xba\xbe':
        result["format"] = "FatMachO"
    elif data[:2] == b'MZ':
        result["format"] = "PE"
    elif data[:4] == b'comp':
        result["format"] = "complzss"
    elif data[:4] == b'bvx2':
        result["format"] = "lzfse"
    
    return result


def try_decompress_kernelcache(data: bytes, output_path: Path) -> Path:
    """Attempt to decompress kernelcache using pyimg4"""
    print("[*] Attempting kernelcache extraction with pyimg4...")
    
    try:
        from pyimg4 import IM4P
        
        im4p = IM4P(data)
        print(f"    Format: IM4P")
        print(f"    FourCC: {im4p.fourcc}")
        print(f"    Description: {im4p.description}")
        
        # Extract payload
        payload = im4p.payload
        
        if payload.compression != payload.Compression.NONE:
            print(f"    Compression: {payload.compression.name}")
            print(f"    Decompressing...")
            raw = payload.decompress()
        else:
            raw = payload.output().data
        
        raw_path = output_path.with_suffix('.raw')
        raw_path.write_bytes(raw)
        size_mb = len(raw) / (1024 * 1024)
        print(f"    -> Raw kernelcache: {raw_path} ({size_mb:.1f} MB)")
        return raw_path
        
    except Exception as e:
        print(f"[!] pyimg4 extraction failed: {e}")
        print("[*] Attempting manual decompression...")
        
        # Try to find compressed payload manually
        # Look for LZFSE magic (bvx2) or LZSS (complzss)
        for magic, name in [(b'bvx2', 'LZFSE'), (b'comp', 'LZSS')]:
            offset = data.find(magic)
            if offset >= 0:
                print(f"    Found {name} at offset {offset}")
                try:
                    if name == 'LZFSE':
                        import lzfse
                        compressed = data[offset:]
                        raw = lzfse.decompress(compressed)
                    elif name == 'LZSS':
                        # Skip complzss header (24 bytes)
                        import lzma
                        compressed = data[offset+24:]
                        raw = lzma.decompress(compressed)
                    
                    raw_path = output_path.with_suffix('.raw')
                    raw_path.write_bytes(raw)
                    size_mb = len(raw) / (1024 * 1024)
                    print(f"    -> Raw kernelcache: {raw_path} ({size_mb:.1f} MB)")
                    return raw_path
                except Exception as e2:
                    print(f"    Decompression failed: {e2}")
        
        # If still Mach-O, just save as-is
        if data[:4] in [b'\xfe\xed\xfa\xce', b'\xce\xfa\xed\xfe', b'\xca\xfe\xba\xbe']:
            raw_path = output_path.with_suffix('.raw')
            raw_path.write_bytes(data)
            print(f"    -> Already a Mach-O, saved as {raw_path}")
            return raw_path
        
        return None


def analyze_kernelcache_macho(path: Path) -> dict:
    """Analyze a raw (decompressed) kernelcache Mach-O using LIEF"""
    print(f"\n[*] Analyzing Mach-O: {path}")
    
    try:
        import lief
    except ImportError:
        print("[!] pip install lief")
        return {}
    
    data = path.read_bytes()
    result = {
        "path": str(path),
        "size": len(data),
        "format": None,
        "arch": None,
        "segments": [],
        "kexts": [],
        "iokit_classes": [],
        "symbols_count": 0,
        "has_pac": False,
        "has_bti": False,
    }
    
    # Check for kernel collection (fileset) vs traditional kernelcache
    is_fileset = False
    
    try:
        binary = lief.MachO.parse(str(path))
        if binary is None:
            print("[!] LIEF failed to parse Mach-O")
            return result
        
        # Get the first (or only) binary
        if hasattr(binary, '__iter__'):
            macho = binary[0] if len(binary) > 0 else binary
        else:
            macho = binary
            
    except Exception as e:
        print(f"[!] LIEF parse error: {e}")
        # Try manual parsing
        return analyze_kernelcache_manual(data)
    
    # Architecture
    header = macho.header
    cpu_type = header.cpu_type
    if cpu_type == lief.MachO.Header.CPU_TYPE.ARM64:
        result["arch"] = "arm64"
        result["format"] = "MachO-ARM64"
    elif cpu_type == lief.MachO.Header.CPU_TYPE.X86_64:
        result["arch"] = "x86_64"
        result["format"] = "MachO-x86_64"
    else:
        result["arch"] = str(cpu_type)
        result["format"] = f"MachO-{cpu_type}"
    
    print(f"    Architecture: {result['arch']}")
    print(f"    File type: {header.file_type}")
    
    # Check for fileset (kernel collection)
    fileset_entries = []
    for cmd in macho.commands:
        if cmd.command == lief.MachO.LoadCommand.TYPE.FILESET_ENTRY:
            is_fileset = True
            if hasattr(cmd, 'name'):
                fileset_entries.append(cmd.name)
    
    if is_fileset:
        print(f"    Type: Kernel Collection (fileset)")
        print(f"    Embedded kexts: {len(fileset_entries)}")
        result["kexts"] = fileset_entries
    
    # Segments
    for seg in macho.segments:
        seg_info = {
            "name": seg.name,
            "vmaddr": hex(seg.virtual_address),
            "vmsize": hex(seg.virtual_size),
            "fileoff": hex(seg.file_offset),
            "filesize": hex(seg.file_size),
        }
        result["segments"].append(seg_info)
    
    # Symbols
    try:
        symbols = list(macho.symbols)
        result["symbols_count"] = len(symbols)
        print(f"    Symbols: {len(symbols)}")
        
        # Look for IOKit-related symbols
        iokit_syms = [s.name for s in symbols if 'IOUserClient' in str(s.name) or 'IOService' in str(s.name)]
        result["iokit_symbols_sample"] = iokit_syms[:50]
        
        # Look for PAC-related symbols
        pac_syms = [s.name for s in symbols if 'pac' in str(s.name).lower() or 'auth' in str(s.name).lower()]
        if pac_syms:
            result["has_pac"] = True
            
    except Exception as e:
        print(f"    Symbol enumeration failed: {e}")
    
    return result


def analyze_kernelcache_manual(data: bytes) -> dict:
    """Manual kernelcache analysis when LIEF fails"""
    result = {
        "size": len(data),
        "format": "manual_parse",
        "kexts": [],
        "iokit_classes": [],
        "strings_of_interest": [],
    }
    
    print("[*] Manual binary analysis...")
    
    # Find kext bundle IDs
    kext_pattern = b'com.apple.'
    offset = 0
    kexts_found = set()
    while True:
        pos = data.find(kext_pattern, offset)
        if pos < 0:
            break
        # Extract the full string
        end = data.find(b'\x00', pos)
        if end > pos and end - pos < 256:
            kext_name = data[pos:end].decode('ascii', errors='ignore')
            if '.' in kext_name and len(kext_name) < 128:
                # Validate it looks like a bundle ID
                if all(c.isalnum() or c in '.-_' for c in kext_name):
                    kexts_found.add(kext_name)
        offset = pos + 1
    
    result["kexts"] = sorted(kexts_found)
    print(f"    Found {len(kexts_found)} kext bundle IDs")
    
    # Find IOKit class names  
    iokit_pattern = b'IOUserClient'
    offset = 0
    iokit_classes = set()
    while True:
        pos = data.find(iokit_pattern, offset)
        if pos < 0:
            break
        # Try to find the full class name by scanning backward
        start = pos
        while start > 0 and data[start-1:start].isalpha():
            start -= 1
        end = pos + len(iokit_pattern)
        while end < len(data) and (data[end:end+1].isalnum() or data[end:end+1] == b'_'):
            end += 1
        class_name = data[start:end].decode('ascii', errors='ignore')
        if len(class_name) > 5:
            iokit_classes.add(class_name)
        offset = pos + 1
    
    result["iokit_classes"] = sorted(iokit_classes)
    print(f"    Found {len(iokit_classes)} IOUserClient-related classes")
    
    # Security-relevant strings
    security_patterns = [
        b'AMFI', b'KTRR', b'PPL', b'CoreTrust', b'APRR',
        b'pointer_auth', b'pac_', b'__auth_', 
        b'sandbox', b'entitlement', b'task_for_pid',
        b'kalloc', b'kfree', b'zone_', b'zalloc',
    ]
    for pattern in security_patterns:
        count = data.count(pattern)
        if count > 0:
            result["strings_of_interest"].append({"pattern": pattern.decode(), "count": count})
    
    return result


def extract_kext_list(data: bytes) -> list:
    """Extract the list of kernel extensions from the kernelcache"""
    kexts = []
    
    # Method 1: __PRELINK_INFO plist
    prelink_start = data.find(b'<dict>')
    if prelink_start >= 0:
        # Try to find the prelink info plist
        prelink_markers = [b'_PrelinkBundlePath', b'CFBundleIdentifier']
        for marker in prelink_markers:
            if marker in data:
                print(f"    Found PRELINK_INFO markers")
                break
    
    # Method 2: String scan for com.apple.* bundle IDs
    kext_pattern = b'com.apple.driver.'
    offset = 0
    while True:
        pos = data.find(kext_pattern, offset)
        if pos < 0:
            break
        end = data.find(b'\x00', pos)
        if end > pos and end - pos < 256:
            name = data[pos:end].decode('ascii', errors='ignore')
            if all(c.isalnum() or c in '.-_' for c in name):
                kexts.append(name)
        offset = pos + 1
    
    # Also look for IOKit family kexts
    for prefix in [b'com.apple.iokit.', b'com.apple.kec.', b'com.apple.security.']:
        offset = 0
        while True:
            pos = data.find(prefix, offset)
            if pos < 0:
                break
            end = data.find(b'\x00', pos)
            if end > pos and end - pos < 256:
                name = data[pos:end].decode('ascii', errors='ignore')
                if all(c.isalnum() or c in '.-_' for c in name):
                    kexts.append(name)
            offset = pos + 1
    
    return sorted(set(kexts))


def generate_attack_surface_report(analysis: dict, device_id: str, ios_version: str, output_dir: Path):
    """Generate an attack surface report from kernelcache analysis"""
    
    soc = SOC_INFO.get(device_id, {})
    
    # Classify kexts by attack surface
    high_priority_drivers = []
    medium_priority_drivers = []
    low_priority_drivers = []
    
    high_risk_keywords = [
        'AGX', 'GPU', 'AVE', 'JPEG', 'Surface', 'Framebuffer',
        'USB', 'HID', 'Bluetooth', 'WiFi', 'Neural', 'ANE',
        'DMA', 'Thunderbolt', 'Audio', 'Codec', 'Sensor',
    ]
    
    medium_risk_keywords = [
        'Storage', 'NVMe', 'NAND', 'Crypto', 'KeyStore',
        'IOMMU', 'DART', 'Timer', 'Watchdog', 'Thermal',
        'Power', 'PMGR', 'SIO', 'Serial',
    ]
    
    for kext in analysis.get("kexts", []):
        classified = False
        for keyword in high_risk_keywords:
            if keyword.lower() in kext.lower():
                high_priority_drivers.append(kext)
                classified = True
                break
        if not classified:
            for keyword in medium_risk_keywords:
                if keyword.lower() in kext.lower():
                    medium_priority_drivers.append(kext)
                    classified = True
                    break
        if not classified:
            low_priority_drivers.append(kext)
    
    report = {
        "meta": {
            "device": device_id,
            "device_name": DEVICES_OF_INTEREST.get(device_id, "Unknown"),
            "ios_version": ios_version,
            "soc": soc,
            "analysis_date": __import__('datetime').datetime.now().isoformat(),
        },
        "kernelcache": {
            "size": analysis.get("size", 0),
            "arch": analysis.get("arch", "unknown"),
            "format": analysis.get("format", "unknown"),
            "segments": analysis.get("segments", []),
            "symbols_count": analysis.get("symbols_count", 0),
        },
        "attack_surface": {
            "total_kexts": len(analysis.get("kexts", [])),
            "high_priority": high_priority_drivers,
            "medium_priority": medium_priority_drivers, 
            "low_priority_count": len(low_priority_drivers),
            "iokit_classes": analysis.get("iokit_classes", []),
        },
        "security_features": {
            "has_pac": analysis.get("has_pac", False),
            "pac_version": soc.get("pac", "unknown"),
            "has_mte": soc.get("mte", False),
            "checkm8_vulnerable": soc.get("checkm8", False),
        },
        "strings_of_interest": analysis.get("strings_of_interest", []),
    }
    
    # Write JSON report
    report_path = output_dir / f"attack_surface_{device_id}_{ios_version.replace('.','_')}.json"
    report_path.write_text(json.dumps(report, indent=2, default=str), encoding='utf-8')
    print(f"\n[+] Attack surface report: {report_path}")
    
    # Write human-readable summary
    summary_path = output_dir / f"attack_surface_{device_id}_{ios_version.replace('.','_')}.txt"
    with open(summary_path, 'w', encoding='utf-8') as f:
        f.write(f"=" * 70 + "\n")
        f.write(f"TrustOS IPSW Analyst — Attack Surface Report\n")
        f.write(f"=" * 70 + "\n\n")
        f.write(f"Device:      {DEVICES_OF_INTEREST.get(device_id, device_id)}\n")
        f.write(f"iOS Version: {ios_version}\n")
        f.write(f"SoC:         {soc.get('soc', '?')} ({soc.get('codename', '?')})\n")
        f.write(f"PAC:         {soc.get('pac', 'None')}\n")
        f.write(f"MTE:         {'Yes' if soc.get('mte') else 'No'}\n")
        f.write(f"checkm8:     {'VULNERABLE' if soc.get('checkm8') else 'NOT vulnerable'}\n")
        f.write(f"\n{'=' * 70}\n")
        f.write(f"KERNEL EXTENSIONS: {len(analysis.get('kexts', []))} total\n")
        f.write(f"{'=' * 70}\n\n")
        
        f.write(f"[HIGH] PRIORITY TARGETS ({len(high_priority_drivers)}):\n")
        f.write(f"{'-' * 50}\n")
        for k in sorted(high_priority_drivers):
            f.write(f"  {k}\n")
        
        f.write(f"\n[MEDIUM] PRIORITY ({len(medium_priority_drivers)}):\n")
        f.write(f"{'-' * 50}\n")
        for k in sorted(medium_priority_drivers):
            f.write(f"  {k}\n")
        
        f.write(f"\n[LOW] PRIORITY: {len(low_priority_drivers)} kexts\n")
        
        if analysis.get("iokit_classes"):
            f.write(f"\n{'=' * 70}\n")
            f.write(f"IOKIT USER CLIENT CLASSES ({len(analysis['iokit_classes'])}):\n")
            f.write(f"{'=' * 70}\n\n")
            for cls in sorted(analysis["iokit_classes"]):
                f.write(f"  {cls}\n")
        
        if analysis.get("strings_of_interest"):
            f.write(f"\n{'=' * 70}\n")
            f.write(f"SECURITY-RELEVANT STRING COUNTS:\n")
            f.write(f"{'=' * 70}\n\n")
            for item in sorted(analysis["strings_of_interest"], key=lambda x: x["count"], reverse=True):
                f.write(f"  {item['pattern']:25s} × {item['count']}\n")
    
    print(f"[+] Summary report: {summary_path}")
    return report


def main():
    parser = argparse.ArgumentParser(description="TrustOS IPSW Analyst — Kernelcache Fetcher & Analyzer")
    parser.add_argument("--device", default="iPhone12,3", help="Device identifier (default: iPhone12,3 = iPhone 11 Pro)")
    parser.add_argument("--version", default=None, help="iOS version to target (default: latest)")
    parser.add_argument("--url", default=None, help="Direct IPSW URL (skip API lookup)")
    parser.add_argument("--local", default=None, help="Path to local kernelcache file (skip download)")
    parser.add_argument("--output", default=None, help="Output directory")
    args = parser.parse_args()
    
    print("=" * 60)
    print("  TrustOS IPSW Analyst — Kernelcache Security Scanner")
    print("=" * 60)
    
    device_id = args.device
    output_dir = Path(args.output) if args.output else OUTPUT_DIR
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # --- Step 1: Get IPSW URL ---
    if args.local:
        print(f"\n[*] Using local file: {args.local}")
        kernelcache_data = Path(args.local).read_bytes()
        ios_version = args.version or "unknown"
    else:
        ipsw_url = args.url
        if not ipsw_url:
            fw = get_firmware_info(device_id, args.version)
            if not fw:
                print("[!] Could not resolve firmware. Use --url or --version")
                sys.exit(1)
            
            ipsw_url = fw["url"]
            ios_version = fw["version"]
            build_id = fw["buildid"]
            signed = fw.get("signed", False)
            
            print(f"\n[+] Found firmware:")
            print(f"    Version: iOS {ios_version} (build {build_id})")
            print(f"    Signed:  {'YES' if signed else 'NO'}")
            print(f"    URL:     {ipsw_url}")
            print(f"    Size:    {fw.get('filesize', 0) / (1024**3):.1f} GB")
        else:
            ios_version = args.version or "unknown"
        
        # --- Step 2: List IPSW contents ---
        files = list_ipsw_contents(ipsw_url)
        if not files:
            print("[!] Could not list IPSW contents")
            sys.exit(1)
        
        print(f"\n[+] IPSW contains {len(files)} files")
        
        # Show interesting files
        interesting = [f for f in files if any(k in f.lower() for k in 
                      ['kernel', 'manifest', 'restore', 'sep', 'baseband'])]
        if interesting:
            print("[+] Files of interest:")
            for f in interesting:
                print(f"    {f}")
        
        # --- Step 3: Download kernelcache ---
        kernels = find_kernelcache(files)
        if not kernels:
            print("[!] No kernelcache found in IPSW!")
            print("    All files:", files[:20])
            sys.exit(1)
        
        kernel_file = kernels[0]
        print(f"\n[+] Kernelcache: {kernel_file}")
        
        kc_path = output_dir / f"kernelcache_{device_id}_{ios_version.replace('.','_')}.im4p"
        if not download_file_from_ipsw(ipsw_url, kernel_file, kc_path):
            sys.exit(1)
        
        # Also download BuildManifest  
        manifest = find_build_manifest(files)
        if manifest:
            manifest_path = output_dir / "BuildManifest.plist"
            download_file_from_ipsw(ipsw_url, manifest, manifest_path)
            try:
                with open(manifest_path, 'rb') as f:
                    plist_data = plistlib.load(f)
                print(f"    BuildManifest loaded: {len(plist_data.get('BuildIdentities', []))} identities")
            except Exception as e:
                print(f"    BuildManifest parse: {e}")
        
        kernelcache_data = kc_path.read_bytes()
    
    # --- Step 4: Parse IMG4 container ---
    print(f"\n[*] Kernelcache size: {len(kernelcache_data) / (1024*1024):.1f} MB")
    img4_info = parse_img4(kernelcache_data)
    print(f"    Container format: {img4_info['format']}")
    
    # --- Step 5: Decompress ---
    raw_path = output_dir / f"kernelcache_{device_id}_{ios_version.replace('.','_')}.raw"
    
    if img4_info['format'] in ('IM4P', 'IMG4'):
        raw_path = try_decompress_kernelcache(kernelcache_data, raw_path)
    elif img4_info['format'] in ('MachO', 'FatMachO'):
        raw_path.write_bytes(kernelcache_data)
        print(f"    Already decompressed Mach-O: {raw_path}")
    else:
        # Try decompress anyway
        raw_path = try_decompress_kernelcache(kernelcache_data, raw_path)
    
    if not raw_path or not raw_path.exists():
        print("[!] Could not extract raw kernelcache. Analyzing container directly...")
        raw_path = output_dir / f"kernelcache_{device_id}_{ios_version.replace('.','_')}.im4p"
    
    # --- Step 6: Analyze Mach-O ---
    analysis = analyze_kernelcache_macho(raw_path)
    
    # If LIEF didn't find kexts, try manual extraction
    if not analysis.get("kexts"):
        print("[*] Trying manual kext extraction...")
        raw_data = raw_path.read_bytes()
        analysis["kexts"] = extract_kext_list(raw_data)
        
        # Also do string-based IOKit analysis
        manual = analyze_kernelcache_manual(raw_data)
        analysis["iokit_classes"] = manual.get("iokit_classes", [])
        analysis["strings_of_interest"] = manual.get("strings_of_interest", [])
    
    # --- Step 7: Generate report ---
    report = generate_attack_surface_report(analysis, device_id, ios_version, output_dir)
    
    # Summary
    print(f"\n{'=' * 60}")
    print(f"  ANALYSIS COMPLETE")
    print(f"{'=' * 60}")
    print(f"  Device:     {DEVICES_OF_INTEREST.get(device_id, device_id)}")
    print(f"  iOS:        {ios_version}")
    print(f"  Kexts:      {len(analysis.get('kexts', []))}")
    print(f"  IOKit:      {len(analysis.get('iokit_classes', []))} classes")
    print(f"  Symbols:    {analysis.get('symbols_count', 'N/A')}")
    print(f"  High-risk:  {len(report.get('attack_surface', {}).get('high_priority', []))}")
    print(f"  Output:     {output_dir}")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
