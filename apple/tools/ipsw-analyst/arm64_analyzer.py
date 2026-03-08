#!/usr/bin/env python3
"""
Advanced ARM64 Kernelcache Analyzer - Chain B Blocker Resolution
================================================================
Addresses all 5 remaining blockers through deep ARM64 disassembly:

  BLOCKER 1: VTable extraction for IOSurfaceRootUserClient
  BLOCKER 2: ml_io_map / ml_phys_read function address resolution
  BLOCKER 3: ExternalMethod dispatch table recovery
  BLOCKER 4: Proc struct offsets from allproc patterns
  BLOCKER 5: Key kernel function signatures

Uses Capstone ARM64 + Mach-O segment parsing for precise analysis.

Target: iPhone 11 Pro (A13/T8030), iOS 26.3 kernelcache
"""

import struct
import json
import sys
import re
from pathlib import Path
from collections import defaultdict

try:
    from capstone import (Cs, CS_ARCH_ARM64, CS_MODE_ARM,
                          CS_GRP_CALL, CS_GRP_JUMP, CS_GRP_BRANCH_RELATIVE)
    HAS_CAPSTONE = True
except ImportError:
    HAS_CAPSTONE = False
    print("[!] Capstone not available - disassembly phases will be limited")

try:
    import lief
    HAS_LIEF = True
except ImportError:
    HAS_LIEF = False

EXTRACTED = Path("extracted")

# Kernel virtual address base
KERNEL_TEXT_VA = 0xfffffff007004000
KERNEL_FILE_BASE = 0  # file offset 0 = VA KERNEL_TEXT_VA


# ============================================================
# Mach-O Segment Parser
# ============================================================
class MachOParser:
    """Parse Mach-O segments from raw kernelcache."""
    
    def __init__(self, data):
        self.data = data
        self.segments = {}
        self.sections = {}
        self._parse()
    
    def _parse(self):
        """Parse Mach-O header and load commands."""
        # Check magic
        magic = struct.unpack_from("<I", self.data, 0)[0]
        if magic == 0xFEEDFACF:    # MH_MAGIC_64
            self._parse_macho64(0)
        elif magic == 0xBEBAFECA:  # FAT_MAGIC
            print("  [!] FAT binary - extracting arm64 slice")
        else:
            # Try kernel collection (fileset)
            # The kernelcache might start with a different header
            # Search for MH_MAGIC_64 within first 4KB
            for off in range(0, min(len(self.data), 4096), 4):
                if struct.unpack_from("<I", self.data, off)[0] == 0xFEEDFACF:
                    self._parse_macho64(off)
                    return
            print(f"  [!] Unknown format, magic = 0x{magic:08x}")
    
    def _parse_macho64(self, base):
        """Parse 64-bit Mach-O."""
        # struct mach_header_64: magic, cputype, cpusubtype, filetype,
        #                        ncmds, sizeofcmds, flags, reserved
        (magic, cputype, cpusubtype, filetype,
         ncmds, sizeofcmds, flags, _) = struct.unpack_from("<IIIIIIII", self.data, base)
        
        print(f"  Mach-O 64: filetype={filetype}, ncmds={ncmds}, flags=0x{flags:x}")
        
        # filetype 0xC = MH_FILESET (kernel collection)
        is_fileset = (filetype == 0xC)
        if is_fileset:
            print("  ** Kernel Collection (MH_FILESET) detected **")
        
        offset = base + 32  # sizeof(mach_header_64)
        
        for _ in range(ncmds):
            if offset + 8 > len(self.data):
                break
            
            cmd, cmdsize = struct.unpack_from("<II", self.data, offset)
            
            if cmd == 0x19:  # LC_SEGMENT_64
                self._parse_segment64(offset)
            elif cmd == 0x35:  # LC_FILESET_ENTRY
                self._parse_fileset_entry(offset, cmdsize)
            
            offset += cmdsize
    
    def _parse_segment64(self, offset):
        """Parse LC_SEGMENT_64."""
        # cmd, cmdsize, segname[16], vmaddr, vmsize, fileoff, filesize,
        # maxprot, initprot, nsects, flags
        fmt = "<II16sQQQQIIII"
        fields = struct.unpack_from(fmt, self.data, offset)
        
        segname = fields[2].rstrip(b'\x00').decode('ascii', errors='replace')
        vmaddr = fields[3]
        vmsize = fields[4]
        fileoff = fields[5]
        filesize = fields[6]
        nsects = fields[9]
        
        self.segments[segname] = {
            "vmaddr": vmaddr,
            "vmsize": vmsize,
            "fileoff": fileoff,
            "filesize": filesize,
            "nsects": nsects,
        }
        
        # Parse sections
        sect_offset = offset + struct.calcsize(fmt)
        for _ in range(nsects):
            if sect_offset + 80 > len(self.data):
                break
            sect_fmt = "<16s16sQQIIIIIII"
            sect_fields = struct.unpack_from(sect_fmt, self.data, sect_offset)
            
            sectname = sect_fields[0].rstrip(b'\x00').decode('ascii', errors='replace')
            seg = sect_fields[1].rstrip(b'\x00').decode('ascii', errors='replace')
            
            self.sections[f"{seg}.{sectname}"] = {
                "vmaddr": sect_fields[2],
                "vmsize": sect_fields[3],
                "fileoff": sect_fields[4],
                "filesize": sect_fields[5],
                "align": sect_fields[6],
            }
            
            sect_offset += 80  # sizeof(section_64)
    
    def _parse_fileset_entry(self, offset, cmdsize):
        """Parse LC_FILESET_ENTRY (kernel collection component)."""
        # cmd, cmdsize, vmaddr, fileoff, entry_id_offset, reserved
        if offset + 32 > len(self.data):
            return
        cmd, cmdsize_, vmaddr, fileoff = struct.unpack_from("<IIQQ", self.data, offset)
        entry_id_off = struct.unpack_from("<I", self.data, offset + 24)[0]
        
        # Read entry ID string
        str_off = offset + entry_id_off
        end = self.data.find(b'\x00', str_off)
        if end > str_off:
            name = self.data[str_off:end].decode('ascii', errors='replace')
        else:
            name = f"unknown_{fileoff:x}"
        
        # Don't overwrite if already exists
        key = f"FILESET:{name}"
        self.segments[key] = {
            "vmaddr": vmaddr,
            "fileoff": fileoff,
            "name": name,
        }
    
    def va_to_file(self, va):
        """Convert virtual address to file offset."""
        for seg_name, seg in self.segments.items():
            if "vmaddr" in seg and "vmsize" in seg and "fileoff" in seg:
                if seg["vmaddr"] <= va < seg["vmaddr"] + seg.get("vmsize", 0):
                    return seg["fileoff"] + (va - seg["vmaddr"])
        # Fallback: simple offset from KERNEL_TEXT_VA
        return va - KERNEL_TEXT_VA
    
    def file_to_va(self, foff):
        """Convert file offset to virtual address."""
        for seg_name, seg in self.segments.items():
            if "fileoff" in seg and "filesize" in seg and "vmaddr" in seg:
                if seg["fileoff"] <= foff < seg["fileoff"] + seg.get("filesize", 0):
                    return seg["vmaddr"] + (foff - seg["fileoff"])
        return KERNEL_TEXT_VA + foff
    
    def print_summary(self):
        """Print segment summary."""
        print("\n  Segments:")
        fileset_count = 0
        for name, seg in sorted(self.segments.items()):
            if name.startswith("FILESET:"):
                fileset_count += 1
                continue
            vmaddr = seg.get("vmaddr", 0)
            vmsize = seg.get("vmsize", 0)
            fileoff = seg.get("fileoff", 0)
            filesize = seg.get("filesize", 0)
            print(f"    {name:25s} VA=0x{vmaddr:016x}  size=0x{vmsize:x}  "
                  f"file=0x{fileoff:x}  fsize=0x{filesize:x}")
        
        if fileset_count:
            print(f"    ... and {fileset_count} FILESET entries")
        
        if self.sections:
            print(f"\n  Sections ({len(self.sections)}):")
            for name, sect in sorted(self.sections.items()):
                if any(k in name for k in ["__TEXT_EXEC", "__DATA_CONST",
                                            "__DATA.__", "__cstring"]):
                    print(f"    {name:35s} VA=0x{sect['vmaddr']:016x}  "
                          f"size=0x{sect['vmsize']:x}  file=0x{sect['fileoff']:x}")


# ============================================================
# BLOCKER 1: VTable Extraction
# ============================================================
def extract_vtable(data, macho):
    """Find IOSurfaceRootUserClient vtable structure."""
    print("\n" + "=" * 70)
    print("BLOCKER 1: IOSURFACE VTABLE EXTRACTION")
    print("=" * 70)
    
    results = {}
    
    # Step 1: Find the __DATA_CONST segment (vtables live here)
    data_const = None
    for name, sect in macho.sections.items():
        if "__DATA_CONST.__const" in name:
            data_const = sect
            break
    
    if not data_const:
        # Try segment level
        for name, seg in macho.segments.items():
            if name == "__DATA_CONST":
                data_const = seg
                break
    
    if data_const:
        print(f"  __DATA_CONST: VA=0x{data_const['vmaddr']:x}, "
              f"size=0x{data_const.get('vmsize', data_const.get('filesize', 0)):x}")
    
    # Step 2: Find IOSurfaceRootUserClient in mangled names
    # VTable region from previous analysis: file offset 0x8422fb - 0x844a43
    vtable_region_start = 0x8422fb
    vtable_region_end = 0x844a43
    
    print(f"\n  Scanning vtable region: 0x{vtable_region_start:x} - 0x{vtable_region_end:x}")
    
    # Find all mangled names in this region
    region = data[vtable_region_start:vtable_region_end]
    
    # C++ mangled names start with _Z or have specific patterns
    # For kernel collections, look for "VIOSurfaceRootUserClient" fragments
    mangled_hits = []
    pos = 0
    while pos < len(region):
        # Look for common vtable markers
        for marker in [b"VIOSurfaceRootUserClient", b"IOSurfaceRootUserClient",
                        b"OSharedUserClient", b"FSurface", b"TSurfaceRoot"]:
            idx = region.find(marker, pos)
            if idx != -1 and idx < pos + 1024:
                file_off = vtable_region_start + idx
                va = macho.file_to_va(file_off)
                mangled_hits.append({
                    "marker": marker.decode(),
                    "file_offset": file_off,
                    "va": va,
                })
                pos = idx + len(marker)
                break
        else:
            pos += 256
    
    print(f"  Found {len(mangled_hits)} vtable markers")
    for hit in mangled_hits[:15]:
        print(f"    {hit['marker']:35s} file=0x{hit['file_offset']:x}  "
              f"VA=0x{hit['va']:x}")
    
    # Step 3: Look for pointer arrays near these markers
    # A vtable is a contiguous array of function pointers
    # Each pointer points to __TEXT_EXEC (code segment)
    
    text_exec = None
    for name, sect in macho.sections.items():
        if "__TEXT_EXEC.__text" in name:
            text_exec = sect
            break
    
    if not text_exec:
        for name, seg in macho.segments.items():
            if name == "__TEXT_EXEC":
                text_exec = seg
                break
    
    code_range = None
    if text_exec:
        code_start = text_exec["vmaddr"]
        code_end = code_start + text_exec.get("vmsize", text_exec.get("filesize", 0))
        code_range = (code_start, code_end)
        print(f"\n  __TEXT_EXEC: 0x{code_start:x} - 0x{code_end:x}")
    
    # Step 4: Scan for pointer arrays in the vtable region
    # In a kernel collection with chained fixups, pointers may be encoded
    # But let's try raw pointer scan first
    
    vtable_candidates = []
    
    if code_range:
        # Scan near each marker for runs of valid code pointers
        for hit in mangled_hits:
            # Look backward and forward from each marker for pointer arrays
            for scan_start in [hit["file_offset"] - 2048,
                               hit["file_offset"] + 64]:
                scan_start = max(0, scan_start)
                scan_start &= ~7  # align to 8 bytes
                
                consecutive = 0
                start_off = scan_start
                
                for off in range(scan_start, min(scan_start + 4096, len(data)), 8):
                    val = struct.unpack_from("<Q", data, off)[0]
                    # Check if this looks like a kernel code pointer
                    # Kernel code pointers: 0xfffffff0070xxxxx - 0xfffffff00a7xxxxx
                    stripped = val & ~0x007F000000000000  # strip PAC bits
                    
                    if (code_range[0] <= stripped <= code_range[1]) or \
                       (0xfffffff007000000 <= stripped <= 0xfffffff00b000000):
                        consecutive += 1
                    else:
                        if consecutive >= 8:  # At least 8 valid pointers = vtable
                            vtable_va = macho.file_to_va(start_off)
                            vtable_candidates.append({
                                "file_offset": start_off,
                                "va": vtable_va,
                                "entries": consecutive,
                                "near_marker": hit["marker"],
                            })
                        consecutive = 0
                        start_off = off + 8
                
                if consecutive >= 8:
                    vtable_va = macho.file_to_va(start_off)
                    vtable_candidates.append({
                        "file_offset": start_off,
                        "va": vtable_va,
                        "entries": consecutive,
                        "near_marker": hit["marker"],
                    })
    
    # Also try scanning __DATA_CONST broadly for IOSurface vtables
    # by looking for large runs of code pointers
    if data_const and code_range:
        dc_start = data_const.get("fileoff", 0)
        dc_size = data_const.get("filesize", data_const.get("vmsize", 0))
        
        print(f"\n  Scanning __DATA_CONST for vtable pointer arrays...")
        scan_end = min(dc_start + dc_size, len(data))
        
        consecutive = 0
        run_start = dc_start
        
        for off in range(dc_start, scan_end, 8):
            val = struct.unpack_from("<Q", data, off)[0]
            stripped = val & ~0x007F000000000000
            
            if (0xfffffff007000000 <= stripped <= 0xfffffff00b000000):
                if consecutive == 0:
                    run_start = off
                consecutive += 1
            else:
                if consecutive >= 20:  # Large vtable = likely a UserClient
                    vtable_candidates.append({
                        "file_offset": run_start,
                        "va": macho.file_to_va(run_start),
                        "entries": consecutive,
                        "near_marker": "DATA_CONST_scan",
                    })
                consecutive = 0
    
    # Deduplicate and sort
    seen = set()
    unique = []
    for v in sorted(vtable_candidates, key=lambda x: -x["entries"]):
        key = v["file_offset"]
        if key not in seen:
            seen.add(key)
            unique.append(v)
    vtable_candidates = unique
    
    print(f"\n  VTable candidates found: {len(vtable_candidates)}")
    for i, vt in enumerate(vtable_candidates[:20]):
        print(f"    #{i+1:3d}: file=0x{vt['file_offset']:x}  VA=0x{vt['va']:x}  "
              f"entries={vt['entries']:3d}  near={vt['near_marker']}")
        
        # Show first few entries
        if vt["entries"] >= 8:
            for j in range(min(5, vt["entries"])):
                off = vt["file_offset"] + j * 8
                ptr = struct.unpack_from("<Q", data, off)[0]
                stripped = ptr & ~0x007F000000000000
                print(f"          [{j:2d}] 0x{ptr:016x} (stripped: 0x{stripped:016x})")
    
    results["vtable_candidates"] = vtable_candidates[:20]
    return results


# ============================================================
# BLOCKER 2: Find ml_io_map / ml_phys_read Functions
# ============================================================
def find_kernel_functions(data, macho):
    """Locate key kernel function addresses via string xref tracing."""
    print("\n" + "=" * 70)
    print("BLOCKER 2: KERNEL FUNCTION ADDRESS RESOLUTION")
    print("=" * 70)
    
    if not HAS_CAPSTONE:
        print("  [!] Capstone required for this phase")
        return {}
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    
    results = {}
    
    # Functions to find
    targets = [
        {
            "name": "ml_io_map",
            "string": b"ml_io_map",
            "string_offset": 0x3d270a6,
            "description": "Maps physical address to kernel VA",
        },
        {
            "name": "ml_phys_read",
            "string": b"ml_phys_read",
            "string_offset": 0x55813,
            "description": "Read from physical address",
        },
        {
            "name": "ml_phys_write",
            "string": b"ml_phys_write",
            "description": "Write to physical address",
        },
        {
            "name": "allproc",
            "string": b"allproc",
            "description": "Head of process list",
        },
        {
            "name": "current_proc",
            "string": b"current_proc",
            "description": "Get current process struct",
        },
        {
            "name": "proc_ucred",
            "string": b"proc_ucred", 
            "description": "Get process credentials",
        },
        {
            "name": "IOSurfaceRootUserClient::externalMethod",
            "string": b"erClientDefaultLockingSingleThreadExternalMethod",
            "description": "IOSurface dispatch entry",
        },
    ]
    
    for target in targets:
        name = target["name"]
        pattern = target["string"]
        
        # Find all string occurrences
        str_offsets = []
        idx = 0
        while True:
            idx = data.find(pattern, idx)
            if idx == -1:
                break
            str_offsets.append(idx)
            idx += 1
            if len(str_offsets) >= 10:
                break
        
        if not str_offsets:
            print(f"\n  [{name}] String not found")
            results[name] = {"status": "string_not_found"}
            continue
        
        str_va_list = [macho.file_to_va(o) for o in str_offsets]
        print(f"\n  [{name}] String at {len(str_offsets)} locations, "
              f"first: file=0x{str_offsets[0]:x} VA=0x{str_va_list[0]:x}")
        
        # Strategy: Find ADRP/ADD sequences that reference this string VA
        # ADRP Xn, #page -> loads page-aligned address
        # ADD  Xn, Xn, #offset -> adds page offset
        # These form the string reference in ARM64 code
        
        str_va = str_va_list[0]
        str_page = str_va & ~0xFFF
        str_pageoff = str_va & 0xFFF
        
        # Search for ADRP instructions targeting this page
        # ADRP encoding: [31] op=1, [30:29] immlo, [28:24] 10000, 
        #                [23:5] immhi, [4:0] Rd
        # The immediate forms a 21-bit value << 12
        
        func_candidates = find_adrp_add_refs(data, macho, md, str_va, name)
        
        if func_candidates:
            results[name] = {
                "status": "found",
                "string_va": f"0x{str_va:x}",
                "function_candidates": func_candidates[:5],
            }
            for i, candidate in enumerate(func_candidates[:5]):
                print(f"    Candidate {i+1}: function at VA=0x{candidate['func_va']:x} "
                      f"(file=0x{candidate['func_file']:x})")
                if "prologue" in candidate:
                    print(f"      Prologue: {candidate['prologue']}")
        else:
            results[name] = {
                "status": "xrefs_not_found",
                "string_va": f"0x{str_va:x}",
            }
            print(f"    No ADRP/ADD xrefs found (may use different addressing)")
    
    return results


def find_adrp_add_refs(data, macho, md, target_va, func_name):
    """Find ADRP+ADD pairs that reference a target VA, then trace to function start."""
    target_page = target_va & ~0xFFF
    target_pageoff = target_va & 0xFFF
    
    candidates = []
    
    # Scan all code-like regions
    # In a kernel collection, code can be anywhere but mainly in __TEXT_EXEC
    # Let's scan regions that look like code (have STP/LDP/BL patterns)
    
    # Quick approach: Search for ADRP instructions
    # ADRP: bit[31]=1, bits[28:24]=10000
    # Encoding: 1_immlo_10000_immhi_Rd
    # Mask:     1_00_10000_xxxxxxxxxxxxxxxxxxx_xxxxx
    # Match:    X_XX_10000 = 0x90000000 (with bit 31 = 1)
    
    # We need to find ADRP that loads the page of target_va
    # Then an ADD with the page offset
    # ADRP uses PC-relative: target_page = (PC & ~0xFFF) + (imm << 12)
    
    search_ranges = []
    
    # Add all known code sections
    for name, sect in macho.sections.items():
        if "__text" in name.lower() or "TEXT_EXEC" in name:
            search_ranges.append((sect.get("fileoff", 0),
                                  sect.get("fileoff", 0) + sect.get("filesize", sect.get("vmsize", 0))))
    
    # If no sections found, scan common code areas
    if not search_ranges:
        # Heuristic: code is between 0x0 and the end of text
        search_ranges.append((0, min(len(data), 0x4000000)))
    
    for range_start, range_end in search_ranges:
        range_end = min(range_end, len(data) - 4)
        
        # Scan for ADRP instructions (every 4 bytes, aligned)
        for off in range(range_start & ~3, range_end, 4):
            insn_val = struct.unpack_from("<I", data, off)[0]
            
            # Check if ADRP: bits[28:24] = 10000, bit[31] = 1
            if (insn_val & 0x9F000000) != 0x90000000:
                continue
            
            # Decode ADRP immediate
            rd = insn_val & 0x1F
            immhi = (insn_val >> 5) & 0x7FFFF  # bits [23:5]
            immlo = (insn_val >> 29) & 0x3       # bits [30:29]
            imm = (immhi << 2) | immlo
            
            # Sign extend 21-bit
            if imm & (1 << 20):
                imm |= ~((1 << 21) - 1)
                imm = imm & 0xFFFFFFFFFFFFFFFF  # keep as uint64
                imm = struct.unpack("<q", struct.pack("<Q", imm))[0]
            
            # Compute ADRP result
            pc_va = macho.file_to_va(off)
            adrp_result = (pc_va & ~0xFFF) + (imm << 12)
            
            if adrp_result != target_page:
                continue
            
            # Found ADRP loading the right page!
            # Now check next instruction for ADD with page offset
            if off + 8 > len(data):
                continue
            
            next_insn = struct.unpack_from("<I", data, off + 4)[0]
            
            # ADD Xd, Xn, #imm12
            # Encoding: 1_0_0_10001_00_imm12_Rn_Rd
            # Or with shift: 1_0_0_10001_01_imm12_Rn_Rd (shift=12)
            if (next_insn & 0xFFC00000) == 0x91000000:
                add_imm = (next_insn >> 10) & 0xFFF
                add_rn = (next_insn >> 5) & 0x1F
                add_rd = next_insn & 0x1F
                
                # Check: ADD uses same register as ADRP dest, and imm matches
                if add_rn == rd and add_imm == target_pageoff:
                    # Found the reference! Now walk backward to find function prologue
                    ref_file = off
                    ref_va = pc_va
                    
                    prologue_off = find_function_start(data, off, max_search=4096)
                    
                    if prologue_off is not None:
                        func_va = macho.file_to_va(prologue_off)
                        
                        # Get prologue instruction
                        prologue_insn = struct.unpack_from("<I", data, prologue_off)[0]
                        if prologue_insn == 0xD503237F:
                            ptype = "PACIBSP"
                        else:
                            ptype = f"STP at 0x{prologue_insn:08x}"
                        
                        candidates.append({
                            "func_va": func_va,
                            "func_file": prologue_off,
                            "ref_va": ref_va,
                            "ref_file": ref_file,
                            "prologue": ptype,
                            "adrp_rd": rd,
                        })
    
    # Deduplicate by function address
    seen = set()
    unique = []
    for c in candidates:
        if c["func_va"] not in seen:
            seen.add(c["func_va"])
            unique.append(c)
    
    return unique


def find_function_start(data, ref_offset, max_search=8192):
    """Walk backward from a reference to find function prologue."""
    start = max(0, ref_offset - max_search)
    start &= ~3
    
    best = None
    
    for off in range(ref_offset, start, -4):
        if off + 4 > len(data):
            continue
        insn = struct.unpack_from("<I", data, off)[0]
        
        # PACIBSP: 0xD503237F
        if insn == 0xD503237F:
            return off
        
        # STP X29, X30, [SP, #imm]! (pre-index)
        # Encoding: 10_101_0_011_0_imm7_11110_11101 
        if (insn & 0xFFE00000) == 0xA9800000:
            rn = (insn >> 5) & 0x1F    # base register
            rt1 = insn & 0x1F          # first reg
            rt2 = (insn >> 10) & 0x1F  # second reg
            if rn == 31 and rt1 == 29 and rt2 == 30:  # SP, X29, X30
                return off
        
        # SUB SP, SP, #imm (stack frame setup, sometimes before STP)
        if (insn & 0xFF0003FF) == 0xD10003FF:  # SUB SP, SP, #imm
            # Check next instruction for STP
            if off + 4 < len(data):
                next_insn = struct.unpack_from("<I", data, off + 4)[0]
                if (next_insn & 0xFFE00000) == 0xA9000000:  # STP pair
                    return off
        
        # RET instruction = end of previous function = start of ours
        if insn == 0xD65F03C0:  # RET
            # Next aligned instruction after RET is function start
            candidate = off + 4
            if candidate <= ref_offset:
                # Verify it looks like a function start
                next_insn = struct.unpack_from("<I", data, candidate)[0]
                if next_insn == 0xD503237F:  # PACIBSP
                    return candidate
                if (next_insn & 0xFFE00000) == 0xA9800000:  # STP pre-index
                    return candidate
    
    return None


# ============================================================
# BLOCKER 3: ExternalMethod Dispatch Table Recovery
# ============================================================
def recover_dispatch_table(data, macho, kernel_funcs):
    """Attempt to recover the IOSurfaceRootUserClient dispatch table."""
    print("\n" + "=" * 70)
    print("BLOCKER 3: EXTERNALMETHOD DISPATCH TABLE RECOVERY")
    print("=" * 70)
    
    if not HAS_CAPSTONE:
        print("  [!] Capstone required")
        return {}
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    
    results = {}
    
    # The dispatch table is typically accessed in externalMethod like:
    #   CMP selector, #max_sel    ; bounds check
    #   B.HI error                ; reject if > max
    #   ADRP X8, dispatch_table@PAGE
    #   ADD  X8, X8, dispatch_table@PAGEOFF
    #   LDR  X9, [X8, selector, LSL#3]  ; load handler pointer
    #   BR   X9                    ; jump to handler
    #
    # OR (IOExternalMethodDispatch style, 24 bytes per entry):
    #   ADRP X8, table@PAGE
    #   ADD  X8, X8, table@PAGEOFF
    #   MOV  X9, #24              ; sizeof(IOExternalMethodDispatch)
    #   MADD X10, selector, X9, X8  ; table + sel * 24
    #   LDR  X11, [X10]           ; load function ptr
    
    # Find the externalMethod function from our previous xref analysis
    ext_method_info = kernel_funcs.get("IOSurfaceRootUserClient::externalMethod", {})
    
    if ext_method_info.get("status") == "found" and ext_method_info.get("function_candidates"):
        for candidate in ext_method_info["function_candidates"][:3]:
            func_file = candidate["func_file"]
            func_va = candidate["func_va"]
            
            print(f"\n  Analyzing function at VA=0x{func_va:x} (file=0x{func_file:x})")
            
            # Disassemble up to 512 instructions
            code = data[func_file:func_file + 2048]
            
            cmp_max = None
            adrp_targets = []
            branches = []
            
            for insn in md.disasm(code, func_va):
                # Look for CMP with immediate (selector bounds check)
                if insn.mnemonic == "cmp" and "#" in insn.op_str:
                    try:
                        imm_str = insn.op_str.split("#")[-1].strip()
                        imm = int(imm_str, 0)
                        if 10 <= imm <= 50:  # Reasonable selector count
                            cmp_max = imm
                            print(f"    CMP max selector: {imm} at 0x{insn.address:x}")
                    except:
                        pass
                
                # Track ADRP targets (potential table bases)
                if insn.mnemonic == "adrp":
                    adrp_targets.append({
                        "addr": insn.address,
                        "op": insn.op_str,
                    })
                
                # Track branch targets
                if insn.mnemonic in ("bl", "b", "br", "blr"):
                    branches.append({
                        "addr": insn.address,
                        "mnemonic": insn.mnemonic,
                        "op": insn.op_str,
                    })
                
                if insn.mnemonic == "ret":
                    break
            
            results["externalMethod"] = {
                "va": f"0x{func_va:x}",
                "file": f"0x{func_file:x}",
                "max_selector": cmp_max,
                "adrp_count": len(adrp_targets),
                "branch_count": len(branches),
            }
            
            if cmp_max:
                print(f"    >> Max selector: {cmp_max} -> {cmp_max + 1} handlers")
                results["selector_count"] = cmp_max + 1
    else:
        print("  externalMethod function not located - searching by pattern...")
        
        # Alternative: Find by searching for the DefaultLocking string xref
        # and then the CBNZ/CMP pattern
        alt_str = b"DefaultLockingSingleThreadExternalMethod"
        idx = data.find(alt_str)
        if idx != -1:
            print(f"  DefaultLocking string at file 0x{idx:x}")
            # The function referencing this string should be nearby
            # in the IOKit base code, not IOSurface-specific
            # We need to find what calls INTO IOSurface's handler
    
    # Step 2: Search for IOExternalMethodDispatch arrays
    # These are 24-byte structures: {func_ptr(8), 4xu32(16)}
    print("\n  Searching for IOExternalMethodDispatch arrays...")
    
    dispatch_tables = find_dispatch_arrays(data, macho)
    
    if dispatch_tables:
        results["dispatch_tables"] = dispatch_tables[:10]
        for i, dt in enumerate(dispatch_tables[:10]):
            print(f"    Table {i+1}: file=0x{dt['file_offset']:x} "
                  f"VA=0x{dt['va']:x} entries={dt['entries']}")
            for j, entry in enumerate(dt["sample_entries"][:5]):
                print(f"      [{j}] func=0x{entry['func']:016x} "
                      f"scalarIn={entry['scalar_in']} structIn={entry['struct_in']} "
                      f"scalarOut={entry['scalar_out']} structOut={entry['struct_out']}")
    
    return results


def find_dispatch_arrays(data, macho):
    """Search for IOExternalMethodDispatch arrays in __DATA_CONST."""
    tables = []
    
    # IOExternalMethodDispatch = {
    #   IOExternalMethodAction function;  // 8 bytes (kernel function pointer)
    #   uint32_t checkScalarInputCount;   // 4 bytes (0-16 typical)
    #   uint32_t checkStructureInputSize; // 4 bytes (0-4096 typical)
    #   uint32_t checkScalarOutputCount;  // 4 bytes (0-16 typical)  
    #   uint32_t checkStructureOutputSize;// 4 bytes (0-65536 typical)
    # }  // = 24 bytes total
    
    ENTRY_SIZE = 24
    
    # Scan regions that could contain const data
    scan_regions = []
    for name, sect in macho.sections.items():
        if "__const" in name.lower() or "DATA_CONST" in name:
            scan_regions.append((sect.get("fileoff", 0),
                                  sect.get("fileoff", 0) + sect.get("filesize", sect.get("vmsize", 0))))
    
    if not scan_regions:
        # Scan a broad region
        scan_regions.append((0, min(len(data), 0x4000000)))
    
    for region_start, region_end in scan_regions:
        region_end = min(region_end, len(data) - ENTRY_SIZE)
        
        off = region_start & ~7  # align
        while off < region_end:
            # Try to parse a dispatch entry
            entries = []
            valid = True
            scan_off = off
            
            while scan_off + ENTRY_SIZE <= region_end and len(entries) < 50:
                func_ptr = struct.unpack_from("<Q", data, scan_off)[0]
                scalar_in, struct_in, scalar_out, struct_out = \
                    struct.unpack_from("<IIII", data, scan_off + 8)
                
                # Validate function pointer (should be kernel code addr or 0)
                stripped = func_ptr & ~0x007F000000000000
                is_valid_ptr = (
                    func_ptr == 0 or
                    (0xfffffff007000000 <= stripped <= 0xfffffff00b000000)
                )
                
                # Validate counts (should be small)
                is_valid_counts = (
                    scalar_in <= 20 and
                    struct_in <= 0x100000 and
                    scalar_out <= 20 and
                    struct_out <= 0x100000
                )
                
                if is_valid_ptr and is_valid_counts and func_ptr != 0:
                    entries.append({
                        "func": func_ptr,
                        "scalar_in": scalar_in,
                        "struct_in": struct_in,
                        "scalar_out": scalar_out,
                        "struct_out": struct_out,
                    })
                    scan_off += ENTRY_SIZE
                else:
                    break
            
            if len(entries) >= 5:  # At least 5 entries = potential dispatch table
                va = macho.file_to_va(off)
                tables.append({
                    "file_offset": off,
                    "va": va,
                    "entries": len(entries),
                    "sample_entries": entries[:10],
                })
                off = scan_off  # skip past this table
            else:
                off += 8
    
    # Sort by entry count (most entries first)
    tables.sort(key=lambda x: -x["entries"])
    
    return tables


# ============================================================
# BLOCKER 4: Proc Struct Offsets
# ============================================================
def find_proc_offsets(data, macho):
    """Find proc struct field offsets from XNU patterns."""
    print("\n" + "=" * 70)
    print("BLOCKER 4: PROC STRUCT OFFSET EXTRACTION")
    print("=" * 70)
    
    results = {}
    
    # Strategy: Find strings that reference proc fields with format strings
    # The kernel has numerous debug strings that reference proc fields:
    # "proc_pid", "proc_ucred", "p_pid", "p_comm", etc.
    
    # Key strings that reveal proc struct layout
    proc_strings = [
        (b"p_pid", "PID field"),
        (b"p_comm", "Process name"),
        (b"p_ucred", "Credentials pointer"),
        (b"p_fd", "File descriptor table"),
        (b"p_list", "Process list links"),
        (b"p_pgrp", "Process group"),
        (b"p_ppid", "Parent PID"),
        (b"p_flag", "Process flags"),
        (b"p_stat", "Process status"),
        (b"p_textvp", "Text vnode"),
        (b"task_t", "Mach task"),
        (b"allproc", "Process list head"),
        (b"kernproc", "Kernel process"),
        (b"initproc", "Init process"),
        (b"proc_ref", "Process reference"),
        (b"proc_rele", "Process release"),
        (b"proc_lock", "Process lock"),
        (b"kauth_cred", "Credential functions"),
        (b"cr_uid", "Credential UID"),
        (b"cr_ruid", "Real UID"),
        (b"cr_svuid", "Saved UID"),
        (b"cr_groups", "Credential groups"),
        (b"posix_cred", "POSIX credentials"),
    ]
    
    print("\n  Searching for proc-related strings:")
    for pattern, desc in proc_strings:
        offsets = []
        idx = 0
        while True:
            idx = data.find(pattern, idx)
            if idx == -1:
                break
            offsets.append(idx)
            idx += 1
            if len(offsets) >= 20:
                break
        
        if offsets:
            va = macho.file_to_va(offsets[0])
            print(f"    {pattern.decode():20s} ({desc:25s}): "
                  f"{len(offsets):3d} hits, first=0x{offsets[0]:x} VA=0x{va:x}")
            results[pattern.decode()] = {
                "hits": len(offsets),
                "first_file": f"0x{offsets[0]:x}",
                "first_va": f"0x{va:x}",
            }
        else:
            print(f"    {pattern.decode():20s} ({desc:25s}): NOT FOUND")
    
    # Try to find specific offset values from LDR instructions
    # near proc-related strings
    # e.g., LDR X0, [X8, #0x100] might be loading p_ucred at offset 0x100
    
    if HAS_CAPSTONE:
        print("\n  Analyzing LDR offsets near 'proc_ucred':")
        md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
        
        ucred_idx = data.find(b"proc_ucred")
        if ucred_idx != -1:
            ucred_va = macho.file_to_va(ucred_idx)
            # Find functions referencing this string
            refs = find_adrp_add_refs(data, macho, md, ucred_va, "proc_ucred")
            
            for ref in refs[:3]:
                print(f"    Function at 0x{ref['func_va']:x} references proc_ucred")
                
                # Disassemble and look for LDR with offsets
                code_off = ref["func_file"]
                code = data[code_off:code_off + 1024]
                
                ldr_offsets = set()
                for insn in md.disasm(code, ref["func_va"]):
                    if insn.mnemonic.startswith("ldr") and "#" in insn.op_str:
                        # Extract offset from LDR [Xn, #offset]
                        match = re.search(r'#(0x[0-9a-f]+|\d+)', insn.op_str)
                        if match:
                            offset_val = int(match.group(1), 0)
                            if 0 < offset_val < 0x1000:  # Reasonable struct offset
                                ldr_offsets.add(offset_val)
                    if insn.mnemonic == "ret":
                        break
                
                if ldr_offsets:
                    print(f"      LDR offsets: {sorted(ldr_offsets)}")
                    results["proc_ucred_ldr_offsets"] = sorted(ldr_offsets)
    
    # Known iOS offsets (from XNU source and prior research)
    # These are APPROXIMATE and need verification
    print("\n  Known proc struct offsets (XNU reference, may vary):")
    known = {
        "p_list.le_next": "+0x00",
        "p_list.le_prev": "+0x08",
        "task": "+0x10",
        "p_pid": "+0x68   (varies: 0x60-0x70)",
        "p_fd": "+0x100  (varies)",
        "p_ucred": "+0x0D8 (varies: 0xD0-0x100)",
        "p_comm": "+0x2B8 (varies: 0x280-0x300)",
        "p_flag": "+0x0E0 (varies)",
    }
    for field, offset in known.items():
        print(f"    {field:25s} {offset}")
    
    results["known_offsets_reference"] = known
    return results


# ============================================================
# BLOCKER 5: Key Kernel Function Signatures  
# ============================================================
def find_function_signatures(data, macho):
    """Find additional kernel function addresses by signature matching."""
    print("\n" + "=" * 70)
    print("BLOCKER 5: KERNEL FUNCTION SIGNATURES")
    print("=" * 70)
    
    results = {}
    
    # Strategy: Search for known instruction patterns at function entries
    # 
    # ml_io_map signature:
    #   PACIBSP
    #   STP X29, X30, [SP, #-0x30]!
    #   ... uses "ml_io_map" string for panic messages
    #
    # We already have string-based resolution from BLOCKER 2
    # Here we do pattern matching on known byte sequences
    
    # Search for panic strings that reveal function identity
    panic_patterns = [
        (b"ml_io_map: Failed to map phys", "ml_io_map_panic"),
        (b"ml_phys_read: illegal phys", "ml_phys_read_check"),
        (b"ml_phys_write: illegal phys", "ml_phys_write_check"),
        (b"process_psets: pset", "process_psets"),
        (b"task_suspend_internal", "task_suspend"),
        (b"IOSurface: %s exceeds maximum", "iosurface_max_check"),
        (b"IOSurface::allocate()", "iosurface_allocate"),
        (b"IOSurfaceRoot::createSurface", "iosurface_create"),
    ]
    
    for pattern, name in panic_patterns:
        idx = data.find(pattern)
        if idx != -1:
            va = macho.file_to_va(idx)
            print(f"  {name:35s} string at file=0x{idx:x} VA=0x{va:x}")
            
            # Find functions referencing this string
            if HAS_CAPSTONE:
                md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
                refs = find_adrp_add_refs(data, macho, md, va, name)
                if refs:
                    for ref in refs[:2]:
                        print(f"    -> function at VA=0x{ref['func_va']:x} "
                              f"({ref.get('prologue', '?')})")
                    results[name] = {
                        "string_va": f"0x{va:x}",
                        "functions": [{"va": f"0x{r['func_va']:x}",
                                       "file": f"0x{r['func_file']:x}"}
                                      for r in refs[:3]],
                    }
                else:
                    results[name] = {"string_va": f"0x{va:x}", "functions": []}
        else:
            print(f"  {name:35s} NOT FOUND")
    
    return results


# ============================================================
# MAIN
# ============================================================
def main():
    print("=" * 70)
    print("ADVANCED ARM64 KERNELCACHE ANALYZER")
    print("Chain B Blocker Resolution")
    print("iPhone 11 Pro (A13/T8030), iOS 26.3")
    print("=" * 70)
    
    # Load kernelcache
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
    
    # Parse Mach-O
    print("\n[*] Parsing Mach-O structure...")
    macho = MachOParser(data)
    macho.print_summary()
    
    # Resolve blockers
    all_results = {}
    
    # BLOCKER 1: VTable
    vtable_results = extract_vtable(data, macho)
    all_results["vtable"] = vtable_results
    
    # BLOCKER 2: Kernel functions
    func_results = find_kernel_functions(data, macho)
    all_results["kernel_functions"] = func_results
    
    # BLOCKER 3: Dispatch table
    dispatch_results = recover_dispatch_table(data, macho, func_results)
    all_results["dispatch_table"] = dispatch_results
    
    # BLOCKER 4: Proc offsets
    proc_results = find_proc_offsets(data, macho)
    all_results["proc_offsets"] = proc_results
    
    # BLOCKER 5: Function signatures
    sig_results = find_function_signatures(data, macho)
    all_results["function_signatures"] = sig_results
    
    # Save results
    # Convert non-serializable items
    def sanitize(obj):
        if isinstance(obj, dict):
            return {k: sanitize(v) for k, v in obj.items()}
        elif isinstance(obj, list):
            return [sanitize(v) for v in obj]
        elif isinstance(obj, (int, float, str, bool, type(None))):
            return obj
        return str(obj)
    
    out_path = EXTRACTED / "arm64_advanced_analysis.json"
    out_path.write_text(json.dumps(sanitize(all_results), indent=2), encoding='utf-8')
    print(f"\n[*] Results saved: {out_path}")
    
    # Summary
    print("\n" + "=" * 70)
    print("BLOCKER RESOLUTION SUMMARY")
    print("=" * 70)
    
    vtable_count = len(vtable_results.get("vtable_candidates", []))
    func_found = sum(1 for v in func_results.values() 
                     if isinstance(v, dict) and v.get("status") == "found")
    func_total = len(func_results)
    dispatch_max = dispatch_results.get("max_selector", "?")
    dispatch_tables = len(dispatch_results.get("dispatch_tables", []))
    proc_found = sum(1 for v in proc_results.values() 
                     if isinstance(v, dict) and v.get("hits", 0) > 0)
    sig_found = sum(1 for v in sig_results.values()
                    if isinstance(v, dict) and len(v.get("functions", [])) > 0)
    
    print(f"  B1 VTable candidates:     {vtable_count}")
    print(f"  B2 Functions resolved:     {func_found}/{func_total}")
    print(f"  B3 Max selector:           {dispatch_max}")
    print(f"  B3 Dispatch table candidates: {dispatch_tables}")
    print(f"  B4 Proc strings found:     {proc_found}")
    print(f"  B5 Function signatures:    {sig_found}")
    
    print("\n[*] ANALYSIS COMPLETE")


if __name__ == "__main__":
    main()
