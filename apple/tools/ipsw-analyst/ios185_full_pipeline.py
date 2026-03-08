#!/usr/bin/env python3
"""
iOS 18.5 (22F76) Full Analysis Pipeline — iPhone 11 Pro (iPhone12,3)
=====================================================================
Downloads the correct kernelcache, parses MH_FILESET, resolves IOSurface
dispatch table + vtable, traces overflow protections, and outputs a
complete address database for Phase 9 exploitation.

Replaces all prior iOS 26.3 analysis with correct iOS 18.5 data.
"""

import struct
import json
import sys
import os
import hashlib
from pathlib import Path
from collections import defaultdict

# --- Config ---
DEVICE = "iPhone12,3"
IOS_VERSION = "18.5"
BUILD_ID = "22F76"
EXTRACTED = Path("extracted")
KC_FILENAME = f"kernelcache_{DEVICE}_{IOS_VERSION.replace('.','_')}.raw"
KC_IM4P     = f"kernelcache_{DEVICE}_{IOS_VERSION.replace('.','_')}.im4p"
OUTPUT_JSON = f"ios185_full_analysis.json"

# Mach-O constants
MH_MAGIC_64       = 0xFEEDFACF
LC_SEGMENT_64     = 0x19
LC_FILESET_ENTRY  = 0x80000035
LC_SYMTAB         = 0x02
LC_DYSYMTAB       = 0x0B
LC_DYLD_CHAINED_FIXUPS = 0x80000034

# ---- Step 1: Download kernelcache ----
def download_kernelcache():
    """Download iOS 18.5 kernelcache for iPhone 11 Pro"""
    import requests
    import remotezip
    
    kc_raw = EXTRACTED / KC_FILENAME
    if kc_raw.exists():
        size_mb = kc_raw.stat().st_size / (1024*1024)
        print(f"[+] Kernelcache already exists: {kc_raw} ({size_mb:.1f} MB)")
        return kc_raw
    
    print(f"[*] Fetching firmware info for {DEVICE} iOS {IOS_VERSION}...")
    url = f"https://api.ipsw.me/v4/device/{DEVICE}?type=ipsw"
    resp = requests.get(url, timeout=30)
    resp.raise_for_status()
    data = resp.json()
    
    fw = None
    for f in data.get("firmwares", []):
        if f.get("buildid") == BUILD_ID:
            fw = f
            break
        if f.get("version", "").startswith(IOS_VERSION):
            fw = f
            break
    
    if not fw:
        # Try listing available
        print(f"[!] Build {BUILD_ID} not found. Listing recent firmwares:")
        for f in data.get("firmwares", [])[:15]:
            print(f"    {f['version']} build={f['buildid']} signed={f.get('signed',False)}")
        sys.exit(1)
    
    ipsw_url = fw["url"]
    print(f"[+] Found: iOS {fw['version']} build {fw['buildid']}")
    print(f"    URL: {ipsw_url}")
    
    # List contents and find kernelcache
    print("[*] Reading IPSW ZIP directory (remote)...")
    with remotezip.RemoteZip(ipsw_url) as rz:
        files = rz.namelist()
    
    kc_candidates = [f for f in files if "kernelcache" in f.lower()]
    print(f"[*] Kernelcache candidates: {kc_candidates}")
    
    if not kc_candidates:
        print("[!] No kernelcache found in IPSW!")
        sys.exit(1)
    
    kc_name = kc_candidates[0]
    im4p_path = EXTRACTED / KC_IM4P
    
    # Download
    print(f"[*] Downloading {kc_name}...")
    with remotezip.RemoteZip(ipsw_url) as rz:
        kc_data = rz.read(kc_name)
    
    EXTRACTED.mkdir(exist_ok=True)
    im4p_path.write_bytes(kc_data)
    size_mb = len(kc_data) / (1024*1024)
    print(f"[+] Downloaded IM4P: {im4p_path} ({size_mb:.1f} MB)")
    
    # Decompress
    print("[*] Decompressing with pyimg4...")
    from pyimg4 import IM4P
    import lzfse
    
    im4p = IM4P(kc_data)
    print(f"    FourCC: {im4p.fourcc}")
    payload_data = im4p.payload.data
    print(f"    Payload: {len(payload_data):,} bytes, first 4: {payload_data[:4]}")
    
    # Check if payload is raw Mach-O or compressed
    if payload_data[:4] in (b'\xfe\xed\xfa\xcf', b'\xcf\xfa\xed\xfe'):
        raw = payload_data
        print(f"    Already decompressed Mach-O")
    elif payload_data[:4] == b'bvx2':
        print(f"    LZFSE compressed, decompressing...")
        raw = lzfse.decompress(payload_data)
        print(f"    Decompressed: {len(raw):,} bytes")
    else:
        # Try lzfse on whole thing
        try:
            raw = lzfse.decompress(payload_data)
        except Exception:
            raw = payload_data
    
    kc_raw.write_bytes(raw)
    size_mb = len(raw) / (1024*1024)
    print(f"[+] Raw kernelcache: {kc_raw} ({size_mb:.1f} MB)")
    return kc_raw


# ---- Step 2: Parse MH_FILESET ----
class FilesetEntry:
    def __init__(self, name, vmaddr, fileoff):
        self.name = name
        self.vmaddr = vmaddr
        self.fileoff = fileoff
        self.segments = {}
        self.sections = {}
    
    def va_to_file(self, va):
        for seg in self.segments.values():
            if seg["vmaddr"] <= va < seg["vmaddr"] + seg["vmsize"]:
                return seg["fileoff"] + (va - seg["vmaddr"])
        return None
    
    def file_to_va(self, foff):
        for seg in self.segments.values():
            if seg["fileoff"] <= foff < seg["fileoff"] + seg["filesize"]:
                return seg["vmaddr"] + (foff - seg["fileoff"])
        return None
    
    def get_code_range(self):
        for name, seg in self.segments.items():
            if "TEXT_EXEC" in name:
                return (seg["fileoff"], seg["fileoff"] + seg["filesize"],
                        seg["vmaddr"], seg["vmaddr"] + seg["vmsize"])
        return None
    
    def get_data_const_range(self):
        for name, seg in self.segments.items():
            if "DATA_CONST" in name:
                return (seg["fileoff"], seg["fileoff"] + seg["filesize"],
                        seg["vmaddr"], seg["vmaddr"] + seg["vmsize"])
        return None


def parse_fileset(kc_data):
    """Parse MH_FILESET kernel collection"""
    magic = struct.unpack_from('<I', kc_data, 0)[0]
    if magic != MH_MAGIC_64:
        print(f"[!] Bad magic: {hex(magic)}")
        sys.exit(1)
    
    # Header: magic, cputype, cpusubtype, filetype, ncmds, sizeofcmds, flags, reserved
    _, cputype, cpusubtype, filetype, ncmds, sizeofcmds, flags, _ = struct.unpack_from('<IIIIIIII', kc_data, 0)
    
    print(f"[+] Mach-O: cputype={hex(cputype)}, filetype={filetype}, ncmds={ncmds}")
    is_fileset = (filetype == 12)  # MH_FILESET = 12
    print(f"    MH_FILESET: {is_fileset}")
    
    entries = []
    top_segments = {}
    kc_base = None
    
    offset = 32  # after mach_header_64
    for _ in range(ncmds):
        if offset + 8 > len(kc_data):
            break
        cmd, cmdsize = struct.unpack_from('<II', kc_data, offset)
        
        if cmd == LC_SEGMENT_64:
            segname = kc_data[offset+8:offset+24].split(b'\x00')[0].decode('ascii', errors='replace')
            vmaddr, vmsize, fileoff, filesize = struct.unpack_from('<QQQQ', kc_data, offset+24)
            top_segments[segname] = {
                "vmaddr": vmaddr, "vmsize": vmsize,
                "fileoff": fileoff, "filesize": filesize
            }
            if segname == "__TEXT" and kc_base is None:
                kc_base = vmaddr
        
        elif cmd == LC_FILESET_ENTRY:
            # struct fileset_entry_command { uint32_t cmd, cmdsize; uint64_t vmaddr, fileoff; uint32_t entry_id_offset; uint32_t reserved; }
            vmaddr, fileoff = struct.unpack_from('<QQ', kc_data, offset+8)
            entry_id_off = struct.unpack_from('<I', kc_data, offset+24)[0]
            name_off = offset + entry_id_off
            name = kc_data[name_off:name_off+256].split(b'\x00')[0].decode('ascii', errors='replace')
            entries.append(FilesetEntry(name, vmaddr, fileoff))
        
        offset += cmdsize
    
    print(f"[+] KC_BASE: {hex(kc_base) if kc_base else 'UNKNOWN'}")
    print(f"[+] Fileset entries: {len(entries)}")
    
    # Parse each entry's inner Mach-O to get segments
    for entry in entries:
        inner_off = entry.fileoff
        if inner_off + 32 > len(kc_data):
            continue
        inner_magic = struct.unpack_from('<I', kc_data, inner_off)[0]
        if inner_magic != MH_MAGIC_64:
            continue
        _, _, _, _, inner_ncmds, inner_sizeofcmds, _, _ = struct.unpack_from('<IIIIIIII', kc_data, inner_off)
        
        cmd_off = inner_off + 32
        for _ in range(inner_ncmds):
            if cmd_off + 8 > len(kc_data):
                break
            cmd, cmdsize = struct.unpack_from('<II', kc_data, cmd_off)
            if cmd == LC_SEGMENT_64:
                segname = kc_data[cmd_off+8:cmd_off+24].split(b'\x00')[0].decode('ascii', errors='replace')
                vmaddr, vmsize, fileoff, filesize = struct.unpack_from('<QQQQ', kc_data, cmd_off+24)
                # nsects
                nsects = struct.unpack_from('<I', kc_data, cmd_off+56 + 8)[0] if cmdsize > 64 else 0
                # Actually: maxprot(4), initprot(4), nsects(4), flags(4) at offset 64
                if cmd_off + 64 + 8 <= len(kc_data):
                    nsects = struct.unpack_from('<I', kc_data, cmd_off + 64)[0]
                entry.segments[segname] = {
                    "vmaddr": vmaddr, "vmsize": vmsize,
                    "fileoff": fileoff, "filesize": filesize
                }
                # Parse sections
                sect_off = cmd_off + 72
                for si in range(nsects):
                    if sect_off + 80 > len(kc_data):
                        break
                    sectname = kc_data[sect_off:sect_off+16].split(b'\x00')[0].decode('ascii', errors='replace')
                    stype = kc_data[sect_off+16:sect_off+32].split(b'\x00')[0].decode('ascii', errors='replace')
                    saddr, ssize = struct.unpack_from('<QQ', kc_data, sect_off+32)
                    soffset = struct.unpack_from('<I', kc_data, sect_off+48)[0]
                    entry.sections[f"{stype}.{sectname}"] = {
                        "addr": saddr, "size": ssize, "offset": soffset
                    }
                    sect_off += 80
            cmd_off += cmdsize
    
    return entries, kc_base, top_segments


# ---- Step 3: Find IOSurface kext ----
def find_iosurface(entries):
    """Find the IOSurface kext in fileset entries"""
    for e in entries:
        if "IOSurface" in e.name and "IOSurfaceRoot" not in e.name:
            # Prefer exact match
            if e.name.endswith("IOSurface") or "com.apple.iokit.IOSurface" in e.name:
                return e
    # Broader search
    for e in entries:
        if "IOSurface" in e.name:
            return e
    return None


def find_iosurface_root(entries):
    for e in entries:
        if "IOSurfaceRoot" in e.name:
            return e
    return None


# ---- Step 4: Disassembly + analysis ----
from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM

def disasm(kc_data, file_off, size, base_va):
    """Disassemble ARM64 code"""
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    code = kc_data[file_off:file_off+size]
    return list(md.disasm(code, base_va))


def decode_chained_ptr(raw_val, kc_base):
    """Decode DYLD_CHAINED_PTR_64_KERNEL_CACHE pointer"""
    is_auth = (raw_val >> 63) & 1
    if is_auth:
        target_off = raw_val & 0x3FFFFFFF  # bits[29:0]
        diversity = (raw_val >> 32) & 0xFFFF
        addr_div = (raw_val >> 48) & 1
        key = (raw_val >> 49) & 3
        key_names = {0: "IA", 1: "IB", 2: "DA", 3: "DB"}
        target_va = kc_base + target_off
        return {
            "auth": True, "target": target_va, "target_off": target_off,
            "diversity": diversity, "addr_div": addr_div, 
            "key": key_names.get(key, f"?{key}"),
            "raw": raw_val
        }
    else:
        target_off = raw_val & 0x3FFFFFFF
        target_va = kc_base + target_off
        return {
            "auth": False, "target": target_va, "target_off": target_off,
            "raw": raw_val
        }


def resolve_vtable(kc_data, entry, kc_base):
    """Find and resolve IOSurface vtable in __DATA_CONST.__const"""
    dc = entry.get_data_const_range()
    if not dc:
        print("[!] No __DATA_CONST segment for IOSurface")
        return None, None
    
    dc_foff, dc_fend, dc_va, dc_va_end = dc
    
    # Scan for vtable: look for a sequence of authenticated pointers
    # vtables are aligned to 8 bytes and have many consecutive auth pointers
    best_run = 0
    best_off = 0
    
    off = dc_foff
    while off < dc_fend - 8:
        val = struct.unpack_from('<Q', kc_data, off)[0]
        if val == 0:
            off += 8
            continue
        
        decoded = decode_chained_ptr(val, kc_base)
        if decoded["auth"]:
            # Count consecutive auth pointers
            run = 0
            check = off
            while check < dc_fend - 8:
                v = struct.unpack_from('<Q', kc_data, check)[0]
                if v == 0:
                    check += 8
                    run += 1
                    continue
                d = decode_chained_ptr(v, kc_base)
                if d["auth"]:
                    run += 1
                    check += 8
                else:
                    break
            if run > best_run:
                best_run = run
                best_off = off
        off += 8
    
    if best_run < 10:
        print(f"[!] No convincing vtable found (best run: {best_run})")
        return None, None
    
    vtable_foff = best_off
    vtable_va = entry.file_to_va(vtable_foff)
    
    # Read vtable entries
    vtable_entries = []
    off = vtable_foff
    consecutive_zero = 0
    while off < dc_fend - 8 and consecutive_zero < 4:
        val = struct.unpack_from('<Q', kc_data, off)[0]
        if val == 0:
            consecutive_zero += 1
            vtable_entries.append({"offset": hex(off - vtable_foff), "raw": 0, "null": True})
            off += 8
            continue
        consecutive_zero = 0
        decoded = decode_chained_ptr(val, kc_base)
        decoded["vt_offset"] = hex(off - vtable_foff)
        vtable_entries.append(decoded)
        off += 8
    
    print(f"[+] IOSurface vtable at VA {hex(vtable_va)} ({len(vtable_entries)} entries, longest auth run: {best_run})")
    return vtable_va, vtable_entries


def resolve_dispatch_table(kc_data, entry, kc_base):
    """Find IOSurfaceRootUserClient dispatch table (external method table)."""
    dc = entry.get_data_const_range()
    if not dc:
        return None, []
    
    dc_foff, dc_fend, dc_va, dc_va_end = dc
    
    # Dispatch table: 24-byte stride entries
    # Each: [function_ptr(8), selector_ptr_or_flags(8), struct_sizes(8)]
    # Look for consecutive authenticated function pointers at 24-byte intervals
    
    best_count = 0
    best_off = 0
    
    off = dc_foff
    while off < dc_fend - 24:
        val = struct.unpack_from('<Q', kc_data, off)[0]
        if val == 0:
            off += 8
            continue
        
        decoded = decode_chained_ptr(val, kc_base)
        if not decoded["auth"]:
            off += 8
            continue
        
        # Check if this starts a 24-byte stride pattern
        count = 0
        check = off
        while check + 24 <= dc_fend:
            v = struct.unpack_from('<Q', kc_data, check)[0]
            if v == 0:
                break
            d = decode_chained_ptr(v, kc_base)
            if d["auth"]:
                count += 1
                check += 24
            else:
                break
        
        if count > best_count:
            best_count = count
            best_off = off
        off += 8
    
    if best_count < 5:
        print(f"[!] No convincing dispatch table found (best: {best_count} entries)")
        return None, []
    
    dt_foff = best_off
    dt_va = entry.file_to_va(dt_foff)
    
    # Read entries
    dt_entries = []
    off = dt_foff
    for i in range(best_count):
        if off + 24 > dc_fend:
            break
        func_raw = struct.unpack_from('<Q', kc_data, off)[0]
        arg2 = struct.unpack_from('<Q', kc_data, off+8)[0]
        arg3 = struct.unpack_from('<Q', kc_data, off+16)[0]
        
        func = decode_chained_ptr(func_raw, kc_base)
        func["selector_index"] = i
        func["input_struct_size"] = arg3 & 0xFFFFFFFF
        func["output_struct_size"] = (arg3 >> 32) & 0xFFFFFFFF
        dt_entries.append(func)
        off += 24
    
    print(f"[+] Dispatch table at VA {hex(dt_va)} ({len(dt_entries)} entries, 24-byte stride)")
    return dt_va, dt_entries


def find_function_by_string_xref(kc_data, entries, kc_base, target_string, search_kext=None):
    """Find a function by locating a string and then its ADRP+ADD reference"""
    # Find string in __TEXT or __DATA
    string_bytes = target_string.encode('utf-8')
    pos = 0
    found = []
    while True:
        pos = kc_data.find(string_bytes + b'\x00', pos)
        if pos == -1:
            break
        found.append(pos)
        pos += 1
    
    if not found:
        return None
    
    results = []
    for str_foff in found:
        # Get VA of string
        str_va = None
        for e in entries:
            str_va = e.file_to_va(str_foff)
            if str_va:
                break
        if not str_va:
            # Use top-level mapping
            str_va = kc_base + str_foff if kc_base else None
        if not str_va:
            continue
        
        # Search for ADRP+ADD that reference this VA
        page = str_va & ~0xFFF
        pageoff = str_va & 0xFFF
        
        kexts_to_search = [search_kext] if search_kext else entries
        for e in kexts_to_search:
            if not e:
                continue
            code = e.get_code_range()
            if not code:
                continue
            code_foff, code_fend, code_va, code_va_end = code
            
            # Quick scan for ADRP instructions targeting this page
            off = code_foff
            while off < code_fend - 8:
                # ADRP: bits[31]=1, bits[28:24]=10000
                insn_raw = struct.unpack_from('<I', kc_data, off)[0]
                if (insn_raw & 0x9F000000) == 0x90000000:  # ADRP
                    rd = insn_raw & 0x1F
                    # Decode ADRP immediate
                    immhi = (insn_raw >> 5) & 0x7FFFF
                    immlo = (insn_raw >> 29) & 0x3
                    imm = (immhi << 2) | immlo
                    if imm & (1 << 20):
                        imm -= (1 << 21)
                    pc = code_va + (off - code_foff)
                    adrp_target = (pc & ~0xFFF) + (imm << 12)
                    
                    if adrp_target == page:
                        # Check next instruction for ADD with matching pageoff
                        next_raw = struct.unpack_from('<I', kc_data, off+4)[0]
                        if (next_raw & 0xFFC00000) == 0x91000000:  # ADD imm
                            add_rd = next_raw & 0x1F
                            add_rn = (next_raw >> 5) & 0x1F
                            add_imm = (next_raw >> 10) & 0xFFF
                            shift = (next_raw >> 22) & 0x3
                            if shift == 1:
                                add_imm <<= 12
                            if add_rn == rd and add_imm == pageoff:
                                results.append(pc)
                off += 4
    
    return results if results else None


def scan_mul_instructions(kc_data, entry):
    """Scan for MUL-family instructions in a kext's code"""
    code = entry.get_code_range()
    if not code:
        return []
    
    code_foff, code_fend, code_va, code_va_end = code
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    
    muls = []
    chunk_size = 0x10000
    off = code_foff
    while off < code_fend:
        end = min(off + chunk_size, code_fend)
        va = code_va + (off - code_foff)
        for insn in md.disasm(kc_data[off:end], va):
            mnemonic = insn.mnemonic.lower()
            if mnemonic in ('mul', 'madd', 'msub', 'smull', 'umull', 'smulh', 'umulh',
                            'smaddl', 'umaddl', 'smsubl', 'umsubl'):
                muls.append({
                    "addr": insn.address,
                    "mnemonic": mnemonic,
                    "op_str": insn.op_str,
                    "file_off": off + (insn.address - va)
                })
        off = end
    
    return muls


def check_overflow_protection(kc_data, entry, mul_addr, window=64):
    """Check if a MUL instruction has overflow protection nearby"""
    code = entry.get_code_range()
    if not code:
        return "unknown"
    
    code_foff, code_fend, code_va, code_va_end = code
    
    # Disassemble a window around the MUL
    foff = code_foff + (mul_addr - code_va)
    start = max(code_foff, foff - window)
    end = min(code_fend, foff + window)
    start_va = code_va + (start - code_foff)
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    
    has_umulh = False
    has_cbnz = False
    has_cmp = False
    has_bcs = False
    
    for insn in md.disasm(kc_data[start:end], start_va):
        m = insn.mnemonic.lower()
        if m in ('umulh', 'smulh'):
            has_umulh = True
        if m in ('cbnz', 'cbz'):
            has_cbnz = True
        if m == 'cmp':
            has_cmp = True
        if m.startswith('b.') and ('cs' in m or 'hi' in m):
            has_bcs = True
    
    if has_umulh and has_cbnz:
        return "UMULH+CBNZ (strong)"
    elif has_umulh:
        return "UMULH (partial)"
    elif has_cmp and has_bcs:
        return "CMP+branch (bounds)"
    elif has_cbnz:
        return "CBNZ (conditional)"
    else:
        return "UNPROTECTED"


def find_key_functions(kc_data, entries, kc_base, iosurface_entry):
    """Find critical kernel functions by string cross-references"""
    functions = {}
    
    # ml_phys_read — look for "ml_phys_read" or nearby strings
    # Actually we search for known strings that are near these functions
    
    targets = {
        "IOSurfaceBufferTileFormat": iosurface_entry,
        "IOSurfaceWidth": iosurface_entry,
        "IOSurfaceHeight": iosurface_entry,
        "IOSurfaceBytesPerRow": iosurface_entry,
        "IOSurfaceBytesPerElement": iosurface_entry,
        "IOSurfaceAllocSize": iosurface_entry,
        "IOSurfaceElementWidth": iosurface_entry,
        "IOSurfaceElementHeight": iosurface_entry,
        "IOSurfacePlaneWidth": iosurface_entry,
        "IOSurfacePlaneHeight": iosurface_entry,
        "IOSurfacePlaneBytesPerRow": iosurface_entry,
    }
    
    for name, kext in targets.items():
        refs = find_function_by_string_xref(kc_data, entries, kc_base, name, kext)
        if refs:
            functions[name] = [hex(r) for r in refs]
    
    return functions


def find_kernel_text_entry(entries):
    """Find the kernel itself (first entry, usually 'com.apple.kernel')"""
    for e in entries:
        if "kernel" in e.name.lower() and "kext" not in e.name.lower():
            return e
    return entries[0] if entries else None


def scan_for_ml_phys(kc_data, kernel_entry, kc_base):
    """Scan kernel TEXT_EXEC for ml_phys_read/write by known patterns"""
    code = kernel_entry.get_code_range()
    if not code:
        return {}
    
    code_foff, code_fend, code_va, code_va_end = code
    
    results = {}
    
    # Search for "gPhysBase" string
    for s in [b"gPhysBase", b"ml_phys_read", b"phys_read", b"ml_io_map"]:
        pos = kc_data.find(s)
        if pos != -1:
            va = None
            for e in [kernel_entry] + list(entries for entries in []):
                va = kernel_entry.file_to_va(pos)
                if va:
                    break
            results[s.decode()] = hex(pos) if not va else hex(va)
    
    # Scan for MRS DAIF (common in ml_phys_read prologue) 
    # ml_phys_read pattern: MRS x?, DAIF; MSR DAIFSet, #0xF; ...LDR from phys... MSR DAIF, x?
    # MRS DAIF = 0xD53B4200 | Rt
    found_mrs_daif = []
    chunk_size = 0x100000
    off = code_foff
    while off < code_fend:
        end = min(off + chunk_size, code_fend)
        pos = off
        while pos < end - 4:
            insn_raw = struct.unpack_from('<I', kc_data, pos)[0]
            if (insn_raw & 0xFFFFFFE0) == 0xD53B4200:  # MRS x?, DAIF
                va = code_va + (pos - code_foff)
                found_mrs_daif.append(va)
            pos += 4
        off = end
    
    results["MRS_DAIF_count"] = len(found_mrs_daif)
    if found_mrs_daif:
        results["MRS_DAIF_first_10"] = [hex(x) for x in found_mrs_daif[:10]]
    
    return results


# ---- Step 5: Full analysis ----
def run_full_analysis(kc_path):
    """Run complete analysis pipeline"""
    print(f"\n{'='*70}")
    print(f"  iOS {IOS_VERSION} ({BUILD_ID}) Full Kernelcache Analysis")
    print(f"  Device: {DEVICE} (iPhone 11 Pro / A13 / T8030)")
    print(f"{'='*70}\n")
    
    kc_data = kc_path.read_bytes()
    kc_size = len(kc_data)
    kc_hash = hashlib.sha256(kc_data).hexdigest()[:16]
    print(f"[+] Kernelcache: {kc_size:,} bytes, SHA256 prefix: {kc_hash}")
    
    # Parse fileset
    entries, kc_base, top_segments = parse_fileset(kc_data)
    
    # Find IOSurface
    iosurface = find_iosurface(entries)
    iosurface_root = find_iosurface_root(entries)
    kernel = find_kernel_text_entry(entries)
    
    if not iosurface:
        # Try broader search
        for e in entries:
            if "surface" in e.name.lower():
                print(f"    Candidate: {e.name}")
        print("[!] IOSurface kext not found!")
        # List all entries
        print("\n[*] All fileset entries:")
        for e in entries:
            print(f"    {e.name} @ VA {hex(e.vmaddr)}")
        sys.exit(1)
    
    print(f"\n[+] IOSurface kext: {iosurface.name}")
    print(f"    VA: {hex(iosurface.vmaddr)}, FileOff: {hex(iosurface.fileoff)}")
    for sname, seg in iosurface.segments.items():
        print(f"    Seg {sname}: VA {hex(seg['vmaddr'])} size {hex(seg['vmsize'])}")
    
    code_range = iosurface.get_code_range()
    if code_range:
        code_foff, code_fend, code_va, code_va_end = code_range
        code_size = code_fend - code_foff
        print(f"    __TEXT_EXEC: VA {hex(code_va)}-{hex(code_va_end)} ({code_size:,} bytes)")
    
    if iosurface_root:
        print(f"\n[+] IOSurfaceRoot kext: {iosurface_root.name}")
        for sname, seg in iosurface_root.segments.items():
            print(f"    Seg {sname}: VA {hex(seg['vmaddr'])} size {hex(seg['vmsize'])}")
    
    # Resolve vtable
    print(f"\n{'='*50}")
    print("[*] RESOLVING IOSURFACE VTABLE")
    print(f"{'='*50}")
    vtable_va, vtable_entries = resolve_vtable(kc_data, iosurface, kc_base)
    
    if vtable_entries:
        print(f"\n  Vtable entries (first 20):")
        for i, ve in enumerate(vtable_entries[:20]):
            if ve.get("null"):
                print(f"    [{i:3d}] +{ve['offset']}: NULL")
            elif ve.get("auth"):
                print(f"    [{i:3d}] +{ve['vt_offset']}: -> {hex(ve['target'])} [{ve['key']}/0x{ve['diversity']:04x}]")
            else:
                print(f"    [{i:3d}] +{ve['vt_offset']}: -> {hex(ve['target'])} (no auth)")
    
    # Also resolve IOSurfaceRootUserClient vtable
    root_vtable_va = None
    root_vtable_entries = None
    if iosurface_root:
        print(f"\n{'='*50}")
        print("[*] RESOLVING IOSurfaceRootUserClient VTABLE")
        print(f"{'='*50}")
        root_vtable_va, root_vtable_entries = resolve_vtable(kc_data, iosurface_root, kc_base)
    
    # Resolve dispatch table
    print(f"\n{'='*50}")
    print("[*] RESOLVING DISPATCH TABLE")
    print(f"{'='*50}")
    
    # Search in IOSurface kext first, then IOSurfaceRoot
    dt_va, dt_entries = resolve_dispatch_table(kc_data, iosurface, kc_base)
    if not dt_entries and iosurface_root:
        dt_va, dt_entries = resolve_dispatch_table(kc_data, iosurface_root, kc_base)
    
    if dt_entries:
        print(f"\n  Dispatch table entries:")
        for i, de in enumerate(dt_entries):
            prot = ""
            if de.get("auth"):
                prot = f"[{de['key']}/0x{de['diversity']:04x}]"
            print(f"    [{i:2d}] -> {hex(de['target'])} {prot} in={de.get('input_struct_size',0)} out={de.get('output_struct_size',0)}")
    
    # Find functions by string xrefs
    print(f"\n{'='*50}")
    print("[*] STRING XREF FUNCTION RESOLUTION")
    print(f"{'='*50}")
    string_funcs = find_key_functions(kc_data, entries, kc_base, iosurface)
    for name, refs in string_funcs.items():
        print(f"    {name}: {refs}")
    
    # MUL scan
    print(f"\n{'='*50}")
    print("[*] MUL INSTRUCTION SCAN")
    print(f"{'='*50}")
    muls = scan_mul_instructions(kc_data, iosurface)
    print(f"    Total MUL-family instructions in IOSurface: {len(muls)}")
    
    # Overflow protection check
    protected = 0
    unprotected = 0
    mul_details = []
    for m in muls:
        prot = check_overflow_protection(kc_data, iosurface, m["addr"])
        m["protection"] = prot
        mul_details.append(m)
        if "UNPROTECTED" in prot:
            unprotected += 1
        else:
            protected += 1
    
    print(f"    Protected: {protected}")
    print(f"    Unprotected: {unprotected}")
    
    # Show unprotected MULs
    if unprotected > 0:
        print(f"\n    === UNPROTECTED MUL INSTRUCTIONS ===")
        for m in mul_details:
            if "UNPROTECTED" in m["protection"]:
                print(f"    {hex(m['addr'])}: {m['mnemonic']} {m['op_str']}")
    
    # Scan for ml_phys_read
    print(f"\n{'='*50}")
    print("[*] KERNEL FUNCTION SCAN (ml_phys_read, gPhysBase)")
    print(f"{'='*50}")
    if kernel:
        phys_info = scan_for_ml_phys(kc_data, kernel, kc_base)
        for k, v in phys_info.items():
            print(f"    {k}: {v}")
    
    # Scan for gPhysBase/gVirtBase pointers in kernel __DATA
    print(f"\n{'='*50}")
    print("[*] SCANNING FOR gPhysBase/gVirtBase IN KERNEL DATA")
    print(f"{'='*50}")
    if kernel:
        for sname, seg in kernel.segments.items():
            if "DATA" in sname and "CONST" not in sname:
                # Look for plausible physical base values
                off = seg["fileoff"]
                end = off + min(seg["filesize"], 0x100000)
                found_phys = []
                while off < end - 8:
                    val = struct.unpack_from('<Q', kc_data, off)[0]
                    # gPhysBase is typically 0x800000000 range for A13
                    if 0x800000000 <= val <= 0x900000000:
                        va = kernel.file_to_va(off)
                        if va:
                            found_phys.append((va, val))
                    off += 8
                if found_phys:
                    print(f"    In {sname}: {len(found_phys)} candidates")
                    for va, val in found_phys[:5]:
                        print(f"      {hex(va)}: {hex(val)}")
    
    # Disassemble key dispatch entries
    print(f"\n{'='*50}")
    print("[*] DISASSEMBLING KEY DISPATCH FUNCTIONS")
    print(f"{'='*50}")
    
    # The important selectors: s_create_surface (typically 0), s_set_value, s_get_value, etc.
    # Disassemble first 32 instructions of each dispatch target
    selector_names = [
        "s_create_surface", "s_delete_surface", "s_lookup_surface", "s_lock_surface",
        "s_unlock_surface", "s_get_value", "s_set_value", "s_increment_use_count",
        "s_decrement_use_count", "s_set_value_xml", "s_get_value_xml",
        "s_bulk_set_value", "s_bulk_get_value",
    ]
    
    dispatch_details = {}
    for i, de in enumerate(dt_entries[:len(selector_names)]):
        target = de["target"]
        name = selector_names[i] if i < len(selector_names) else f"selector_{i}"
        
        # Find which kext this target belongs to
        target_entry = None
        for e in entries:
            code = e.get_code_range()
            if code and code[2] <= target < code[3]:
                target_entry = e
                break
        
        if target_entry:
            code = target_entry.get_code_range()
            foff = code[0] + (target - code[2])
            insns = disasm(kc_data, foff, 128, target)
            dispatch_details[name] = {
                "va": hex(target),
                "selector": i,
                "kext": target_entry.name,
                "first_insns": [(hex(ins.address), ins.mnemonic, ins.op_str) for ins in insns[:16]]
            }
            print(f"\n    === {name} (selector {i}) @ {hex(target)} ===")
            for ins in insns[:16]:
                print(f"      {hex(ins.address)}: {ins.mnemonic:8s} {ins.op_str}")
    
    # ---- Build output database ----
    print(f"\n{'='*70}")
    print("[*] BUILDING ADDRESS DATABASE")
    print(f"{'='*70}")
    
    db = {
        "meta": {
            "device": DEVICE,
            "device_name": "iPhone 11 Pro",
            "soc": "A13 Bionic",
            "codename": "T8030",
            "ios_version": IOS_VERSION,
            "build_id": BUILD_ID,
            "pac": "v1 (ARM8.3-A, 7-bit context)",
            "kc_size": kc_size,
            "kc_sha256_prefix": kc_hash,
            "kc_base": hex(kc_base) if kc_base else None,
            "fileset_entries": len(entries),
            "analysis_tool": "ios185_full_pipeline.py"
        },
        "iosurface": {
            "kext_name": iosurface.name,
            "vmaddr": hex(iosurface.vmaddr),
            "fileoff": hex(iosurface.fileoff),
            "segments": {k: {sk: hex(sv) if isinstance(sv, int) else sv for sk, sv in v.items()} for k, v in iosurface.segments.items()},
            "code_range": {
                "va_start": hex(code_range[2]) if code_range else None,
                "va_end": hex(code_range[3]) if code_range else None,
                "size": code_range[1] - code_range[0] if code_range else 0
            }
        },
        "vtable": {
            "va": hex(vtable_va) if vtable_va else None,
            "entry_count": len(vtable_entries) if vtable_entries else 0,
            "entries": []
        },
        "dispatch_table": {
            "va": hex(dt_va) if dt_va else None,
            "entry_count": len(dt_entries),
            "stride": 24,
            "entries": []
        },
        "overflow_analysis": {
            "total_mul": len(muls),
            "protected": protected,
            "unprotected": unprotected,
            "unprotected_addrs": [hex(m["addr"]) for m in mul_details if "UNPROTECTED" in m["protection"]],
            "protection_types": {}
        },
        "string_xrefs": string_funcs,
        "dispatch_functions": {},
        "kernel_functions": {},
        "all_fileset_entries": [(e.name, hex(e.vmaddr)) for e in entries]
    }
    
    # Vtable entries
    if vtable_entries:
        for ve in vtable_entries:
            if ve.get("null"):
                db["vtable"]["entries"].append({"offset": ve["offset"], "target": None})
            elif ve.get("auth"):
                db["vtable"]["entries"].append({
                    "offset": ve["vt_offset"],
                    "target": hex(ve["target"]),
                    "key": ve["key"],
                    "diversity": hex(ve["diversity"]),
                    "addr_div": ve["addr_div"]
                })
            else:
                db["vtable"]["entries"].append({
                    "offset": ve.get("vt_offset", "?"),
                    "target": hex(ve["target"]),
                    "auth": False
                })
    
    # Dispatch entries
    for i, de in enumerate(dt_entries):
        name = selector_names[i] if i < len(selector_names) else f"selector_{i}"
        entry = {
            "selector": i,
            "name": name,
            "target": hex(de["target"]),
            "input_struct_size": de.get("input_struct_size", 0),
            "output_struct_size": de.get("output_struct_size", 0),
        }
        if de.get("auth"):
            entry["key"] = de["key"]
            entry["diversity"] = hex(de["diversity"])
        db["dispatch_table"]["entries"].append(entry)
    
    # Dispatch function details
    db["dispatch_functions"] = dispatch_details
    
    # Protection types count
    prot_types = defaultdict(int)
    for m in mul_details:
        prot_types[m["protection"]] += 1
    db["overflow_analysis"]["protection_types"] = dict(prot_types)
    
    # Save
    out_path = EXTRACTED / OUTPUT_JSON
    with open(out_path, 'w', encoding='utf-8') as f:
        json.dump(db, f, indent=2, ensure_ascii=False)
    print(f"\n[+] Full analysis saved to: {out_path}")
    
    # Print summary
    print(f"\n{'='*70}")
    print(f"  ANALYSIS COMPLETE — iOS {IOS_VERSION} ({BUILD_ID}) iPhone 11 Pro")
    print(f"{'='*70}")
    print(f"  KC_BASE:           {hex(kc_base) if kc_base else 'N/A'}")
    print(f"  Fileset entries:   {len(entries)}")
    print(f"  IOSurface kext:    {iosurface.name}")
    if code_range:
        print(f"  IOSurface code:    {hex(code_range[2])}-{hex(code_range[3])}")
    print(f"  Vtable VA:         {hex(vtable_va) if vtable_va else 'N/A'} ({len(vtable_entries) if vtable_entries else 0} entries)")
    print(f"  Dispatch table:    {hex(dt_va) if dt_va else 'N/A'} ({len(dt_entries)} entries)")
    print(f"  MUL instructions:  {len(muls)} total, {protected} protected, {unprotected} unprotected")
    
    if unprotected > 0:
        print(f"\n  *** {unprotected} POTENTIALLY UNPROTECTED MULTIPLICATIONS FOUND ***")
        print(f"  This may be exploitable on iOS 18.5 (older than 26.3)")
    else:
        print(f"\n  All multiplications appear protected. Focus on type confusion / races.")
    
    # Print key dispatch addresses
    print(f"\n  Key dispatch function addresses:")
    for i, de in enumerate(dt_entries[:10]):
        name = selector_names[i] if i < len(selector_names) else f"selector_{i}"
        print(f"    {name:30s} -> {hex(de['target'])}")
    
    return db


# ---- Main ----
if __name__ == "__main__":
    os.chdir(Path(__file__).parent)
    
    print(f"[*] Working directory: {os.getcwd()}")
    print(f"[*] Target: {DEVICE} / iOS {IOS_VERSION} / Build {BUILD_ID}")
    
    kc_path = download_kernelcache()
    db = run_full_analysis(kc_path)
