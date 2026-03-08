#!/usr/bin/env python3
"""
Kernel Collection Fileset Parser + IOSurface Kext Deep Analysis
================================================================
Properly handles MH_FILESET kernel collections where each kext is a
separate embedded Mach-O with its own segments. This resolves why
ADRP/ADD xrefs weren't found - we need per-kext segment mapping.

Approach:
1. Parse top-level Mach-O to get all FILESET entries
2. Parse each fileset entry as inner Mach-O
3. Find the IOSurface kext's own __TEXT, __TEXT_EXEC, __DATA_CONST
4. Scan ADRP/ADD within each kext's own code/data segments
5. Resolve vtable pointers via chained fixups understanding

Target: iOS 26.3 kernelcache (A13/T8030)
"""

import struct
import json
import sys
import re
from pathlib import Path
from collections import defaultdict

try:
    from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM
    HAS_CAPSTONE = True
except ImportError:
    HAS_CAPSTONE = False
    print("[!] Capstone not available")

EXTRACTED = Path("extracted")

# LC constants
LC_SEGMENT_64     = 0x19
LC_FILESET_ENTRY  = 0x80000035   # LC_REQ_DYLD | 0x35
LC_SYMTAB         = 0x02
LC_DYSYMTAB       = 0x0B
LC_DYLD_CHAINED_FIXUPS = 0x80000034
MH_MAGIC_64       = 0xFEEDFACF


class FilesetEntry:
    """Represents one kext/component in the kernel collection."""
    def __init__(self, name, vmaddr, fileoff):
        self.name = name
        self.vmaddr = vmaddr
        self.fileoff = fileoff
        self.segments = {}
        self.sections = {}
    
    def va_to_file(self, va):
        """Convert VA to file offset using this entry's segments."""
        for seg in self.segments.values():
            seg_va = seg["vmaddr"]
            seg_size = seg["vmsize"]
            if seg_va <= va < seg_va + seg_size:
                return seg["fileoff"] + (va - seg_va)
        return None
    
    def file_to_va(self, foff):
        """Convert file offset to VA using this entry's segments."""
        for seg in self.segments.values():
            seg_foff = seg["fileoff"]
            seg_fsize = seg["filesize"]
            if seg_foff <= foff < seg_foff + seg_fsize:
                return seg["vmaddr"] + (foff - seg_foff)
        return None
    
    def get_code_range(self):
        """Get the code segment (file offset range)."""
        for name, seg in self.segments.items():
            if "TEXT_EXEC" in name:
                return (seg["fileoff"], seg["fileoff"] + seg["filesize"],
                        seg["vmaddr"], seg["vmaddr"] + seg["vmsize"])
        # Fallback to __TEXT
        for name, seg in self.segments.items():
            if name == "__TEXT":
                return (seg["fileoff"], seg["fileoff"] + seg["filesize"],
                        seg["vmaddr"], seg["vmaddr"] + seg["vmsize"])
        return None
    
    def get_data_const_range(self):
        """Get __DATA_CONST segment."""
        for name, seg in self.segments.items():
            if "DATA_CONST" in name:
                return (seg["fileoff"], seg["fileoff"] + seg["filesize"],
                        seg["vmaddr"], seg["vmaddr"] + seg["vmsize"])
        return None
    
    def get_cstring_range(self):
        """Get __cstring section or __TEXT segment for strings."""
        for key, sect in self.sections.items():
            if "__cstring" in key:
                return (sect["fileoff"], sect["fileoff"] + sect["size"],
                        sect["vmaddr"], sect["vmaddr"] + sect["size"])
        # Fallback to TEXT
        for name, seg in self.segments.items():
            if name == "__TEXT":
                return (seg["fileoff"], seg["fileoff"] + seg["filesize"],
                        seg["vmaddr"], seg["vmaddr"] + seg["vmsize"])
        return None


def parse_kernel_collection(data):
    """Parse the full kernel collection with all fileset entries."""
    print("[*] Parsing Kernel Collection...")
    
    magic = struct.unpack_from("<I", data, 0)[0]
    if magic != MH_MAGIC_64:
        print(f"[!] Not a Mach-O 64: magic=0x{magic:08x}")
        return None, []
    
    (_, cputype, cpusubtype, filetype,
     ncmds, sizeofcmds, flags, _) = struct.unpack_from("<IIIIIIII", data, 0)
    
    print(f"  filetype={filetype} ({'MH_FILESET' if filetype == 12 else 'other'})")
    print(f"  ncmds={ncmds}")
    
    # Parse all load commands
    fileset_entries = []   # LC_FILESET_ENTRY
    top_segments = {}      # LC_SEGMENT_64 at top level
    
    offset = 32  # sizeof(mach_header_64)
    
    for _ in range(ncmds):
        if offset + 8 > len(data):
            break
        cmd, cmdsize = struct.unpack_from("<II", data, offset)
        
        if cmd == LC_SEGMENT_64:
            fmt = "<II16sQQQQIIII"
            fields = struct.unpack_from(fmt, data, offset)
            segname = fields[2].rstrip(b'\x00').decode('ascii', errors='replace')
            top_segments[segname] = {
                "vmaddr": fields[3],
                "vmsize": fields[4],
                "fileoff": fields[5],
                "filesize": fields[6],
                "nsects": fields[9],
            }
        
        elif cmd == LC_FILESET_ENTRY:
            # struct fileset_entry_command {
            #     uint32_t cmd;              // 0
            #     uint32_t cmdsize;          // 4
            #     uint64_t vmaddr;           // 8
            #     uint64_t fileoff;          // 16
            #     uint32_t entry_id_offset;  // 24 (offset from start of cmd to string)
            #     uint32_t reserved;         // 28
            # };
            vmaddr, fileoff = struct.unpack_from("<QQ", data, offset + 8)
            entry_id_off = struct.unpack_from("<I", data, offset + 24)[0]
            
            str_off = offset + entry_id_off
            end = data.find(b'\x00', str_off)
            name = data[str_off:end].decode('ascii', errors='replace') if end > str_off else "?"
            
            entry = FilesetEntry(name, vmaddr, fileoff)
            fileset_entries.append(entry)
        
        offset += cmdsize
    
    print(f"  Top-level segments: {len(top_segments)}")
    print(f"  Fileset entries: {len(fileset_entries)}")
    
    # Now parse each fileset entry's inner Mach-O
    for entry in fileset_entries:
        parse_inner_macho(data, entry)
    
    return top_segments, fileset_entries


def parse_inner_macho(data, entry):
    """Parse the inner Mach-O at a fileset entry's fileoff."""
    base = entry.fileoff
    if base + 32 > len(data):
        return
    
    magic = struct.unpack_from("<I", data, base)[0]
    if magic != MH_MAGIC_64:
        return
    
    (_, _, _, filetype,
     ncmds, sizeofcmds, flags, _) = struct.unpack_from("<IIIIIIII", data, base)
    
    offset = base + 32
    
    for _ in range(ncmds):
        if offset + 8 > len(data):
            break
        cmd, cmdsize = struct.unpack_from("<II", data, offset)
        
        if cmd == LC_SEGMENT_64:
            fmt = "<II16sQQQQIIII"
            if offset + struct.calcsize(fmt) > len(data):
                offset += cmdsize
                continue
            fields = struct.unpack_from(fmt, data, offset)
            segname = fields[2].rstrip(b'\x00').decode('ascii', errors='replace')
            nsects = fields[9]
            
            entry.segments[segname] = {
                "vmaddr": fields[3],
                "vmsize": fields[4],
                "fileoff": fields[5],
                "filesize": fields[6],
            }
            
            # Parse sections
            sect_off = offset + struct.calcsize(fmt)
            for _ in range(nsects):
                if sect_off + 80 > len(data):
                    break
                sect_fmt = "<16s16sQQIIIIIII"
                sf = struct.unpack_from(sect_fmt, data, sect_off)
                sectname = sf[0].rstrip(b'\x00').decode('ascii', errors='replace')
                sect_segname = sf[1].rstrip(b'\x00').decode('ascii', errors='replace')
                
                entry.sections[f"{sect_segname}.{sectname}"] = {
                    "vmaddr": sf[2],
                    "size": sf[3],
                    "fileoff": sf[4],
                    "filesize": sf[5],
                }
                sect_off += 80
        
        offset += cmdsize


def find_iosurface_kext(entries):
    """Find the IOSurface kext entry."""
    for entry in entries:
        if "IOSurface" in entry.name and "Accelerator" not in entry.name:
            return entry
    # Broader search
    for entry in entries:
        if "iosurface" in entry.name.lower():
            return entry
    return None


def find_kernel_entry(entries):
    """Find the main kernel entry."""
    for entry in entries:
        name_lower = entry.name.lower()
        if name_lower in ("kernel", "com.apple.kernel", "mach_kernel"):
            return entry
    # Try first entry or largest
    for entry in entries:
        if "kernel" in entry.name.lower():
            return entry
    return entries[0] if entries else None


def print_entry_details(entry):
    """Print details of a fileset entry."""
    print(f"\n  Entry: {entry.name}")
    print(f"    VM addr: 0x{entry.vmaddr:x}")
    print(f"    File offset: 0x{entry.fileoff:x}")
    print(f"    Segments ({len(entry.segments)}):")
    for name, seg in sorted(entry.segments.items()):
        print(f"      {name:20s} VA=0x{seg['vmaddr']:016x}  "
              f"size=0x{seg['vmsize']:x}  file=0x{seg['fileoff']:x}")
    if entry.sections:
        print(f"    Sections ({len(entry.sections)}):")
        for name, sect in sorted(entry.sections.items()):
            print(f"      {name:35s} VA=0x{sect['vmaddr']:016x}  "
                  f"size=0x{sect['size']:x}  file=0x{sect['fileoff']:x}")


# ============================================================
# ADRP/ADD Reference Scanner (per-kext)
# ============================================================
def find_adrp_refs_in_entry(data, entry, target_va, max_results=10):
    """Find ADRP+ADD pairs referencing target_va within a specific kext's code."""
    target_page = target_va & ~0xFFF
    target_pageoff = target_va & 0xFFF
    
    code_range = entry.get_code_range()
    if not code_range:
        return []
    
    code_file_start, code_file_end, code_va_start, code_va_end = code_range
    code_file_end = min(code_file_end, len(data) - 4)
    
    results = []
    
    for off in range(code_file_start & ~3, code_file_end, 4):
        insn_val = struct.unpack_from("<I", data, off)[0]
        
        # Check ADRP: bits[28:24] = 10000
        if (insn_val & 0x9F000000) != 0x90000000:
            continue
        
        # Decode ADRP
        rd = insn_val & 0x1F
        immhi = (insn_val >> 5) & 0x7FFFF
        immlo = (insn_val >> 29) & 0x3
        imm21 = (immhi << 2) | immlo
        
        # Sign-extend 21-bit
        if imm21 & (1 << 20):
            imm21 -= (1 << 21)
        
        pc_va = entry.file_to_va(off)
        if pc_va is None:
            # Fallback: compute from code segment
            pc_va = code_va_start + (off - code_file_start)
        
        adrp_result = (pc_va & ~0xFFF) + (imm21 << 12)
        
        if adrp_result != target_page:
            continue
        
        # Check next instruction for ADD Xd, Xn, #pageoff
        if off + 8 > len(data):
            continue
        next_insn = struct.unpack_from("<I", data, off + 4)[0]
        
        # ADD (immediate) 64-bit: 1_0_0_10001_00_imm12_Rn_Rd
        if (next_insn & 0xFFC00000) == 0x91000000:
            add_imm = (next_insn >> 10) & 0xFFF
            add_rn = (next_insn >> 5) & 0x1F
            add_rd = next_insn & 0x1F
            
            if add_rn == rd and add_imm == target_pageoff:
                # Found xref! Find function start
                func_off = find_function_start_backward(data, off, 
                                                        code_file_start, max_back=4096)
                func_va = None
                if func_off is not None:
                    func_va = entry.file_to_va(func_off)
                    if func_va is None:
                        func_va = code_va_start + (func_off - code_file_start)
                
                results.append({
                    "ref_file": off,
                    "ref_va": pc_va,
                    "func_file": func_off,
                    "func_va": func_va,
                    "dest_reg": rd,
                })
                
                if len(results) >= max_results:
                    return results
    
    return results


def find_function_start_backward(data, ref_offset, region_start, max_back=4096):
    """Walk backward from ref to find function prologue."""
    start = max(region_start, ref_offset - max_back)
    start &= ~3
    
    for off in range(ref_offset - 4, start, -4):
        if off + 4 > len(data):
            continue
        insn = struct.unpack_from("<I", data, off)[0]
        
        # PACIBSP
        if insn == 0xD503237F:
            return off
        
        # STP X29, X30, [SP, #imm]! (pre-index)
        if (insn & 0xFFC003E0) == 0xA98003E0:
            rt1 = insn & 0x1F
            rt2 = (insn >> 10) & 0x1F
            if rt1 == 29 and rt2 == 30:
                return off
        
        # RET = end of previous function
        if insn == 0xD65F03C0:
            candidate = off + 4
            if candidate <= ref_offset:
                next_insn = struct.unpack_from("<I", data, candidate)[0]
                if next_insn == 0xD503237F:  # PACIBSP
                    return candidate
                if (next_insn & 0xFFC003E0) == 0xA98003E0:  # STP pre-index
                    rt1 = next_insn & 0x1F
                    rt2 = (next_insn >> 10) & 0x1F
                    if rt1 == 29 and rt2 == 30:
                        return candidate
    
    return None


# ============================================================
# IOSurface Kext Analysis
# ============================================================
def analyze_iosurface_kext(data, iosurface_entry):
    """Deep analysis of the IOSurface kext component."""
    print("\n" + "=" * 70)
    print("IOSURFACE KEXT DEEP ANALYSIS")
    print("=" * 70)
    
    print_entry_details(iosurface_entry)
    
    results = {}
    
    # Step 1: Find all strings in this kext's __TEXT segment
    text_seg = iosurface_entry.segments.get("__TEXT")
    if text_seg:
        text_start = text_seg["fileoff"]
        text_end = text_start + text_seg["filesize"]
        text_data = data[text_start:text_end]
        
        # Find IOSurface strings within this kext's text
        key_strings = [
            b"s_create_surface",
            b"IOSurfaceRootUserClient",
            b"CSBufferPitch",
            b"IOSurfaceSharedListEntry",
            b"externalMethod",
            b"getTargetAndMethodForIndex",
            b"s_set_value",
            b"s_get_value", 
            b"s_lookup",
            b"s_decrement_use_count",
            b"IOSurface: %s exceeds maximum value",
            b"IOSurface::allocate()",
            b"bytesPerRow",
            b"allocationSize",
            b"global (insecure) IOSurface lookups",
            b"IOBufferMemoryDescriptor::inTaskWithOptions",
            b"IOSurfaceDefaultLockingSingleThread",
        ]
        
        print(f"\n  Strings in IOSurface kext (__TEXT at 0x{text_start:x}):")
        string_results = {}
        
        for pat in key_strings:
            idx = text_data.find(pat)
            if idx != -1:
                file_off = text_start + idx
                va = iosurface_entry.file_to_va(file_off)
                if va is None:
                    va = text_seg["vmaddr"] + idx
                
                string_results[pat.decode('utf-8', errors='replace')] = {
                    "file_offset": file_off,
                    "va": va,
                }
                print(f"    {pat.decode('utf-8',errors='replace'):55s} "
                      f"file=0x{file_off:x}  VA=0x{va:x}")
            else:
                print(f"    {pat.decode('utf-8',errors='replace'):55s} NOT IN KEXT")
        
        results["strings"] = string_results
    
    # Step 2: Find functions via ADRP/ADD xrefs within this kext
    print(f"\n  Resolving function addresses via ADRP xrefs...")
    
    code_range = iosurface_entry.get_code_range()
    if code_range:
        code_start, code_end, code_va_start, code_va_end = code_range
        print(f"  Code segment: file=0x{code_start:x}-0x{code_end:x}  "
              f"VA=0x{code_va_start:x}-0x{code_va_end:x}")
    
    func_results = {}
    for name, info in results.get("strings", {}).items():
        target_va = info["va"]
        refs = find_adrp_refs_in_entry(data, iosurface_entry, target_va, max_results=5)
        
        if refs:
            func_results[name] = refs
            for i, ref in enumerate(refs[:3]):
                func_str = f"func=0x{ref['func_va']:x}" if ref['func_va'] else "func=?"
                print(f"    {name[:50]:50s} -> ref at 0x{ref['ref_va']:x}, {func_str}")
        # Don't print NOT FOUND for every string, too noisy
    
    results["function_xrefs"] = {
        k: [{"ref_va": f"0x{r['ref_va']:x}", 
             "func_va": f"0x{r['func_va']:x}" if r['func_va'] else None,
             "func_file": f"0x{r['func_file']:x}" if r['func_file'] else None}
            for r in v]
        for k, v in func_results.items()
    }
    
    found_count = sum(1 for v in func_results.values() if v)
    total = len(results.get("strings", {}))
    print(f"\n  Functions resolved: {found_count}/{total}")
    
    # Step 3: VTable extraction from __DATA_CONST
    print(f"\n  Extracting vtable from __DATA_CONST...")
    vtable_results = extract_kext_vtable(data, iosurface_entry)
    results["vtable"] = vtable_results
    
    # Step 4: Dispatch table search in __DATA_CONST
    print(f"\n  Searching for dispatch tables...")
    dispatch_results = find_kext_dispatch(data, iosurface_entry)
    results["dispatch"] = dispatch_results
    
    # Step 5: Disassemble key functions
    if HAS_CAPSTONE and func_results:
        print(f"\n  Disassembling key functions...")
        disasm_results = disassemble_key_functions(data, iosurface_entry, func_results)
        results["disassembly"] = disasm_results
    
    return results


def extract_kext_vtable(data, entry):
    """Extract vtable from the kext's __DATA_CONST."""
    dc_range = entry.get_data_const_range()
    if not dc_range:
        print("    No __DATA_CONST found in this kext")
        return {}
    
    dc_file_start, dc_file_end, dc_va_start, dc_va_end = dc_range
    dc_file_end = min(dc_file_end, len(data))
    
    print(f"    __DATA_CONST: file=0x{dc_file_start:x}-0x{dc_file_end:x}  "
          f"VA=0x{dc_va_start:x}-0x{dc_va_end:x}")
    
    code_range = entry.get_code_range()
    code_va_start = code_range[2] if code_range else 0xfffffff007000000
    code_va_end = code_range[3] if code_range else 0xfffffff00b000000
    
    # Scan for pointer arrays - but handle chained fixups
    # In kernel collections, pointers may be:
    # 1. Raw kernel VAs (0xfffffff0xxxxxxxx)
    # 2. Chained fixup encoded (high bits have bind/rebase flags)
    # 3. Zero (NULL entries)
    
    # Strategy: Look for runs of 8-byte aligned values where
    # either the value is a valid kernel VA, or all high bits 
    # match a pattern consistent with chained fixups
    
    vtables = []
    off = dc_file_start & ~7
    
    while off < dc_file_end - 8:
        val = struct.unpack_from("<Q", data, off)[0]
        
        # Check if this starts a pointer array
        # Chained fixup rebase: bit[62] = 0, bits[61:51] = 0, target in low bits
        # OR raw pointer: 0xfffffff0xxxxxxxx
        
        consecutive = 0
        entries = []
        scan = off
        
        while scan < dc_file_end - 8:
            v = struct.unpack_from("<Q", data, scan)[0]
            
            is_kernel_va = (0xfffffff007000000 <= (v & ~0x007F000000000000) <= 0xfffffff00b000000)
            is_chained_rebase = (v != 0 and (v >> 62) == 0 and (v & 0xFFFFFFFF) != 0)
            is_null = (v == 0)
            
            if is_kernel_va or (is_chained_rebase and not is_null):
                entries.append(v)
                consecutive += 1
                scan += 8
            elif is_null and consecutive > 0:
                # Allow sparse NULLs in vtable
                entries.append(0)
                consecutive += 1
                scan += 8
                if consecutive > 3 and entries[-3:] == [0, 0, 0]:
                    break  # Too many NULLs
            else:
                break
        
        if consecutive >= 10:
            va = entry.file_to_va(off)
            if va is None:
                va = dc_va_start + (off - dc_file_start)
            
            # Count actual non-null entries
            non_null = sum(1 for e in entries if e != 0)
            
            vtables.append({
                "file_offset": off,
                "va": va,
                "total_entries": consecutive,
                "non_null": non_null,
                "sample": entries[:10],
            })
            off = scan
        else:
            off += 8
    
    # Sort by size 
    vtables.sort(key=lambda x: -x["non_null"])
    
    print(f"    Found {len(vtables)} vtable candidates")
    for i, vt in enumerate(vtables[:10]):
        print(f"      #{i+1}: file=0x{vt['file_offset']:x}  VA=0x{vt['va']:x}  "
              f"entries={vt['total_entries']} (non-null={vt['non_null']})")
        for j, val in enumerate(vt["sample"][:5]):
            stripped = val & ~0x007F000000000000
            tag = ""
            if code_range and code_va_start <= stripped <= code_va_end:
                tag = " [CODE]"
            elif 0xfffffff007000000 <= stripped <= 0xfffffff00b000000:
                tag = " [KERNEL]"
            elif val == 0:
                tag = " [NULL]"
            else:
                tag = f" [CHAIN? hi=0x{val>>48:04x}]"
            print(f"        [{j:2d}] 0x{val:016x}{tag}")
    
    return {
        "count": len(vtables),
        "candidates": [{
            "file": f"0x{v['file_offset']:x}",
            "va": f"0x{v['va']:x}",
            "entries": v["total_entries"],
            "non_null": v["non_null"],
        } for v in vtables[:10]],
    }


def find_kext_dispatch(data, entry):
    """Find IOExternalMethodDispatch arrays in this kext."""
    dc_range = entry.get_data_const_range()
    if not dc_range:
        return {}
    
    dc_file_start, dc_file_end, dc_va_start, dc_va_end = dc_range
    dc_file_end = min(dc_file_end, len(data))
    
    code_range = entry.get_code_range()
    code_va_start = code_range[2] if code_range else 0xfffffff007000000
    code_va_end = code_range[3] if code_range else 0xfffffff00b000000
    
    ENTRY_SIZE = 24  # sizeof(IOExternalMethodDispatch)
    
    tables = []
    off = dc_file_start & ~7
    
    while off < dc_file_end - ENTRY_SIZE:
        entries = []
        scan = off
        
        while scan + ENTRY_SIZE <= dc_file_end:
            func_ptr = struct.unpack_from("<Q", data, scan)[0]
            scalar_in, struct_in, scalar_out, struct_out = \
                struct.unpack_from("<IIII", data, scan + 8)
            
            stripped = func_ptr & ~0x007F000000000000
            
            # Check function pointer validity
            is_valid_ptr = (
                func_ptr == 0 or
                (code_va_start <= stripped <= code_va_end) or
                (0xfffffff007000000 <= stripped <= 0xfffffff00b000000) or
                # Chained fixup: target in low 36 bits with tag in high bits
                (func_ptr != 0 and scalar_in <= 20 and struct_in <= 0x100000)
            )
            
            is_valid_counts = (
                scalar_in <= 20 and
                struct_in <= 0x100000 and
                scalar_out <= 20 and
                struct_out <= 0x100000
            )
            
            if is_valid_ptr and is_valid_counts and func_ptr != 0:
                entries.append({
                    "func": func_ptr,
                    "func_stripped": stripped,
                    "scalar_in": scalar_in,
                    "struct_in": struct_in,
                    "scalar_out": scalar_out,
                    "struct_out": struct_out,
                })
                scan += ENTRY_SIZE
            else:
                break
        
        if len(entries) >= 5:
            va = entry.file_to_va(off)
            if va is None:
                va = dc_va_start + (off - dc_file_start)
            
            tables.append({
                "file_offset": off,
                "va": va,
                "count": len(entries),
                "entries": entries,
            })
            off = scan
        else:
            off += 8
    
    tables.sort(key=lambda x: -x["count"])
    
    print(f"    Found {len(tables)} dispatch table candidates")
    for i, tbl in enumerate(tables[:5]):
        print(f"      #{i+1}: file=0x{tbl['file_offset']:x}  VA=0x{tbl['va']:x}  "
              f"entries={tbl['count']}")
        for j, e in enumerate(tbl["entries"][:10]):
            print(f"        [{j:2d}] func=0x{e['func']:016x}  "
                  f"scIn={e['scalar_in']} stIn=0x{e['struct_in']:x}  "
                  f"scOut={e['scalar_out']} stOut=0x{e['struct_out']:x}")
    
    return {
        "count": len(tables),
        "tables": [{
            "file": f"0x{t['file_offset']:x}",
            "va": f"0x{t['va']:x}",
            "entry_count": t["count"],
            "entries": [{
                "func": f"0x{e['func']:016x}",
                "scalar_in": e["scalar_in"],
                "struct_in": e["struct_in"],
                "scalar_out": e["scalar_out"],
                "struct_out": e["struct_out"],
            } for e in t["entries"][:25]]
        } for t in tables[:5]],
    }


def disassemble_key_functions(data, entry, func_xrefs):
    """Disassemble resolved IOSurface functions."""
    if not HAS_CAPSTONE:
        return {}
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = False  # Faster
    
    results = {}
    
    # Priority functions to disassemble
    priority = [
        "s_create_surface",
        "CSBufferPitch",
        "IOSurface: %s exceeds maximum value",
        "getTargetAndMethodForIndex",
        "externalMethod",
        "IOSurfaceSharedListEntry",
        "IOSurfaceRootUserClient",
        "global (insecure) IOSurface lookups",
    ]
    
    disassembled = set()
    
    for pname in priority:
        if pname not in func_xrefs or not func_xrefs[pname]:
            continue
        
        ref = func_xrefs[pname][0]
        func_file = ref["func_file"]
        func_va = ref["func_va"]
        
        if func_file is None or func_va is None:
            continue
        
        if func_va in disassembled:
            continue
        disassembled.add(func_va)
        
        print(f"\n    --- Disassembling function near '{pname[:45]}' ---")
        print(f"    Function at VA=0x{func_va:x} (file=0x{func_file:x})")
        
        code = data[func_file:func_file + 2048]
        
        insns = []
        calls = []
        compares = []
        
        for insn in md.disasm(code, func_va):
            entry_info = {
                "addr": f"0x{insn.address:x}",
                "mnemonic": insn.mnemonic,
                "op_str": insn.op_str,
            }
            insns.append(entry_info)
            
            # Track calls
            if insn.mnemonic in ("bl", "blr"):
                calls.append(entry_info)
            
            # Track CMP instructions (selector bounds, size checks)
            if insn.mnemonic == "cmp":
                compares.append(entry_info)
            
            if insn.mnemonic == "ret" or len(insns) >= 200:
                break
        
        # Print summary
        print(f"    Instructions: {len(insns)}, Calls: {len(calls)}, Compares: {len(compares)}")
        
        # Show first 40 instructions
        for insn in insns[:40]:
            print(f"      {insn['addr']}: {insn['mnemonic']:8s} {insn['op_str']}")
        if len(insns) > 40:
            print(f"      ... ({len(insns) - 40} more)")
        
        # Highlight compares (may reveal selector count or size checks)
        if compares:
            print(f"    CMP instructions (potential bounds checks):")
            for c in compares:
                print(f"      {c['addr']}: {c['mnemonic']} {c['op_str']}")
        
        results[pname] = {
            "func_va": f"0x{func_va:x}",
            "instruction_count": len(insns),
            "calls": len(calls),
            "compares": [{
                "addr": c["addr"],
                "operands": c["op_str"],
            } for c in compares],
        }
    
    return results


# ============================================================
# Kernel Component Analysis (ml_io_map, proc, etc.)
# ============================================================
def analyze_kernel_component(data, kernel_entry):
    """Analyze the main kernel component for ml_io_map, allproc, etc."""
    print("\n" + "=" * 70)
    print("KERNEL COMPONENT ANALYSIS")
    print("=" * 70)
    
    print_entry_details(kernel_entry)
    
    results = {}
    
    # Find key strings in kernel
    text_seg = kernel_entry.segments.get("__TEXT")
    if not text_seg:
        print("  No __TEXT segment in kernel entry")
        return results
    
    text_start = text_seg["fileoff"]
    text_end = text_start + text_seg["filesize"]
    
    key_strings = [
        b"ml_io_map",
        b"ml_phys_read",
        b"ml_phys_write",
        b"current_proc",
        b"allproc",
        b"kernproc",
        b"p_ucred",
        b"task_for_pid",
        b"proc_find",
        b"proc_ref_locked",
    ]
    
    print(f"\n  Key strings in kernel __TEXT:")
    for pat in key_strings:
        idx = data.find(pat, text_start, text_end)
        if idx != -1:
            va = kernel_entry.file_to_va(idx)
            if va is None:
                va = text_seg["vmaddr"] + (idx - text_start)
            
            results[pat.decode()] = {"file": idx, "va": va}
            print(f"    {pat.decode():30s} file=0x{idx:x}  VA=0x{va:x}")
            
            # Find ADRP refs to this string
            refs = find_adrp_refs_in_entry(data, kernel_entry, va, max_results=5)
            for ref in refs[:3]:
                func_str = f"func=0x{ref['func_va']:x}" if ref['func_va'] else "?"
                print(f"      -> xref at 0x{ref['ref_va']:x}, {func_str}")
            
            if refs:
                results[pat.decode()]["functions"] = [
                    {"func_va": f"0x{r['func_va']:x}" if r['func_va'] else None,
                     "func_file": f"0x{r['func_file']:x}" if r['func_file'] else None}
                    for r in refs[:5]
                ]
        else:
            print(f"    {pat.decode():30s} NOT IN KERNEL __TEXT")
    
    return results


# ============================================================
# MAIN
# ============================================================
def main():
    print("=" * 70)
    print("KERNEL COLLECTION FILESET ANALYZER")
    print("IOSurface Kext + Kernel Deep Analysis")
    print("iPhone 11 Pro (A13/T8030), iOS 26.3")
    print("=" * 70)
    
    kc_path = None
    for f in EXTRACTED.iterdir():
        if f.name.endswith(".raw") and "kernelcache" in f.name:
            kc_path = f
            break
    
    if not kc_path:
        print("[!] Kernelcache not found")
        sys.exit(1)
    
    print(f"[*] Loading: {kc_path} ({kc_path.stat().st_size / 1024 / 1024:.1f} MB)")
    data = kc_path.read_bytes()
    
    # Parse kernel collection
    top_segments, entries = parse_kernel_collection(data)
    
    # List interesting fileset entries
    print(f"\n  Fileset entries of interest:")
    for entry in entries:
        if any(k in entry.name.lower() for k in
               ["iosurface", "kernel", "iokit", "gpu", "agx", "sandbox"]):
            segs = ", ".join(entry.segments.keys())
            print(f"    {entry.name:50s} segs=[{segs}]")
    
    # Find IOSurface kext
    ios_entry = find_iosurface_kext(entries)
    if ios_entry:
        iosurface_results = analyze_iosurface_kext(data, ios_entry)
    else:
        print("\n[!] IOSurface kext not found in fileset")
        iosurface_results = {}
    
    # Find and analyze kernel 
    kern_entry = find_kernel_entry(entries)
    if kern_entry:
        kernel_results = analyze_kernel_component(data, kern_entry)
    else:
        print("\n[!] Kernel entry not found")
        kernel_results = {}
    
    # Save all results
    all_results = {
        "fileset_entries": [
            {"name": e.name, "vmaddr": f"0x{e.vmaddr:x}",
             "segments": {k: {sk: (f"0x{sv:x}" if isinstance(sv, int) else sv)
                              for sk, sv in v.items()}
                          for k, v in e.segments.items()}}
            for e in entries
        ],
        "iosurface_kext": iosurface_results if isinstance(iosurface_results, dict) else {},
        "kernel": kernel_results,
    }
    
    out_path = EXTRACTED / "fileset_analysis.json"
    
    def sanitize(obj):
        if isinstance(obj, dict):
            return {str(k): sanitize(v) for k, v in obj.items()}
        elif isinstance(obj, list):
            return [sanitize(v) for v in obj]
        elif isinstance(obj, (int, float, str, bool, type(None))):
            return obj
        return str(obj)
    
    out_path.write_text(json.dumps(sanitize(all_results), indent=2), encoding='utf-8')
    print(f"\n[*] Results saved: {out_path}")
    
    # Summary
    print("\n" + "=" * 70)
    print("ANALYSIS SUMMARY")
    print("=" * 70)
    
    if ios_entry:
        ios_str_count = len(iosurface_results.get("strings", {}))
        ios_func_count = len(iosurface_results.get("function_xrefs", {}))
        ios_vtables = iosurface_results.get("vtable", {}).get("count", 0)
        ios_dispatch = iosurface_results.get("dispatch", {}).get("count", 0)
        print(f"  IOSurface kext: {entry.name}")
        print(f"    Strings found:       {ios_str_count}")
        print(f"    Functions resolved:   {ios_func_count}")
        print(f"    VTable candidates:    {ios_vtables}")
        print(f"    Dispatch candidates:  {ios_dispatch}")
    
    kern_funcs = sum(1 for v in kernel_results.values()
                     if isinstance(v, dict) and "functions" in v)
    print(f"  Kernel functions resolved: {kern_funcs}")
    
    print("\n[*] COMPLETE")


if __name__ == "__main__":
    main()
