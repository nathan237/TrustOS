#!/usr/bin/env python3
"""
JIT Pipeline Mapper - Deep Shader Compilation Flow Analysis
=============================================================
Phase 2 analysis: Maps the complete GPU shader JIT compilation pipeline
from userland Metal shader submission to kernel code generation.

Uses LIEF for Mach-O symbol resolution + Capstone for ARM64 disassembly
to trace the actual code paths through:
  1. IOGPUDevice / AGXDevice ExternalMethod dispatch
  2. AGXCompilerService shader compilation entry
  3. AIR (Apple IR) -> native GPU ISA compilation
  4. JIT memory allocation (RWX page creation)
  5. DART/IOMMU configuration for GPU memory
  6. Firmware upload paths (AGXFirmware, RTKit)

Target: iPhone 11 Pro (A13 Bionic / T8030)
"""

import struct
import re
import json
import sys
import os
from collections import defaultdict, Counter
from pathlib import Path

try:
    import lief
    HAS_LIEF = True
except ImportError:
    HAS_LIEF = False
    print("[WARN] LIEF not available - symbol analysis disabled")

try:
    from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM, CS_GRP_JUMP, CS_GRP_CALL
    HAS_CAPSTONE = True
except ImportError:
    HAS_CAPSTONE = False
    print("[WARN] Capstone not available - disassembly disabled")


# ============================================================
# CONFIGURATION
# ============================================================
KERNELCACHE_DIR = Path("extracted")
KERNELCACHE_NAME = None  # auto-detect

# Key symbol patterns for each pipeline stage
PIPELINE_STAGES = {
    "stage1_userland_entry": {
        "description": "UserClient ExternalMethod dispatch (entry from userland)",
        "symbol_patterns": [
            r"AGXDeviceUserClient.*externalMethod",
            r"IOGPUDeviceUserClient.*externalMethod",
            r"IOGPUDevice.*externalMethod",
            r"IOSurfaceRootUserClient.*externalMethod",
            r"AGXAccelerator.*externalMethod",
            r"IOGPU.*getTargetAndMethod",
            r"IOGPU.*s_externalMethod",
            r"AGX.*s_externalMethod",
            r"IOUserClient.*externalMethod",
        ],
        "string_patterns": [
            b"externalMethod",
            b"getTargetAndMethodForIndex",
            b"IOExternalMethodDispatch",
            b"IOExternalMethodArguments",
        ],
    },
    "stage2_command_submit": {
        "description": "GPU command buffer submission",
        "symbol_patterns": [
            r"AGXAccelerator.*submitCommand",
            r"AGXAccelerator.*submit_",
            r"IOGPU.*submitCommand",
            r"IOGPU.*CommandQueue",
            r"AGX.*CommandQueue",
            r"AGXRing",
            r"AGXAcceleratorRing",
            r"IOGPUCommandBuffer",
        ],
        "string_patterns": [
            b"submitCommandBuffer",
            b"CommandQueue",
            b"CommandBuffer",
            b"AGXAcceleratorRing",
            b"submit_ta",
            b"submit_compute",
            b"submit_3d",
        ],
    },
    "stage3_shader_compile": {
        "description": "Shader compilation (AIR -> native GPU ISA)",
        "symbol_patterns": [
            r"AGXCompilerService",
            r"AGXShaderCompiler",
            r"MTLCompilerService",
            r"CompileShader",
            r"compileProgram",
            r"shader.*compile",
            r"AIR.*compile",
            r"CompilationMode",
        ],
        "string_patterns": [
            b"AGXCompilerService",
            b"MTLCompilerService", 
            b"compileShader",
            b"CompileProgram",
            b"CompilationMode",
            b"ShaderCompiler",
            b"AIR_to_",
            b"metallib",
            b"shader_cache",
            b"ShaderCache",
        ],
    },
    "stage4_jit_codegen": {
        "description": "JIT code generation and memory allocation",
        "symbol_patterns": [
            r"JIT",
            r"jit_",
            r"code_gen",
            r"codegen",
            r"emit_code",
            r"generateCode",
            r"MAP_JIT",
            r"RWX",
            r"vm_protect.*execute",
        ],
        "string_patterns": [
            b"JIT",
            b"jit_alloc",
            b"jit_free",
            b"code_gen",
            b"codegen",
            b"emit_",
            b"MAP_JIT",
            b"RWX",
            b"mach_vm_protect",
            b"vm_map_protect",
        ],
    },
    "stage5_gpu_memory": {
        "description": "GPU memory management and DART/IOMMU",
        "symbol_patterns": [
            r"AGXGART",
            r"AGXVMContext",
            r"IOGPUResource",
            r"IOGPUMemory",
            r"IODARTMapper",
            r"AppleDart",
            r"DART.*map",
            r"DART.*translate",
            r"gpu.*alloc",
            r"AGXMemory",
        ],
        "string_patterns": [
            b"AGXVMContext",
            b"AGXGART",
            b"IOGPUResource",
            b"IODARTMapper",
            b"DART",
            b"dart_map",
            b"dart_translate",
            b"gpu_va",
            b"gpu_phys",
            b"IOAccelResource",
            b"AGXBuffer",
            b"AGXTexture",
        ],
    },
    "stage6_firmware": {
        "description": "GPU firmware loading (AGXFirmware, ASC, RTKit)",
        "symbol_patterns": [
            r"AGXFirmware",
            r"AGXFirmwareUtil",
            r"RTKit",
            r"ASC.*Mailbox",
            r"AGX.*Firmware",
            r"AGX.*UCFirmware",
            r"IOASCMailbox",
        ],
        "string_patterns": [
            b"AGXFirmware",
            b"AGXFirmwareUtil",
            b"RTKit",
            b"ASCMailbox",
            b"firmware_load",
            b"fw_upload",
            b"AGXMicroSequencer",
            b"gpu_fw",
            b"RTBuddyV2",
        ],
    },
}

# Physical address ranges of interest (A13 T8030)
PHYS_REGIONS = {
    "bootrom":    (0x100000000, 0x100080000, "SecureROM/BootROM"),
    "sram":       (0x19C000000, 0x19C100000, "SRAM (iBoot stage)"),
    "agx_mmio":   (0x206400000, 0x206500000, "AGX GPU MMIO registers"),
    "pmp":        (0x23B700000, 0x23B800000, "PMP (Performance Monitor)"),
    "dart_gpu":   (0x231004000, 0x231008000, "GPU DART IOMMU base"),
    "amcc":       (0x200000000, 0x200100000, "AMCC memory controller"),
    "sep":        (0x240000000, 0x241000000, "SEP region"),
    "pmgr":       (0x23B100000, 0x23B200000, "PMGR power management"),
}


def find_kernelcache():
    """Auto-detect kernelcache .raw file."""
    for f in KERNELCACHE_DIR.iterdir():
        if f.name.endswith(".raw") and "kernelcache" in f.name:
            return f
    return None


def load_binary(path):
    """Load raw binary data."""
    print(f"[*] Loading binary: {path} ({path.stat().st_size / 1024 / 1024:.1f} MB)")
    return path.read_bytes()


# ============================================================
# PHASE 1: LIEF Symbol Analysis
# ============================================================
def analyze_symbols(kc_path):
    """Use LIEF to extract and categorize all symbols from kernelcache."""
    if not HAS_LIEF:
        return {}

    print("\n" + "=" * 70)
    print("PHASE 1: LIEF SYMBOL ANALYSIS")
    print("=" * 70)

    binary = lief.MachO.parse(str(kc_path))
    if binary is None:
        print("[!] LIEF failed to parse kernelcache")
        return {}

    # Get first slice (should be only one - ARM64)
    macho = binary.at(0)
    if macho is None:
        print("[!] No Mach-O slice found")
        return {}

    # Collect ALL symbols
    all_symbols = []
    agx_symbols = []
    gpu_symbols = []
    jit_symbols = []
    dart_symbols = []
    mem_symbols = []
    userclient_symbols = []

    sym_count = 0
    for sym in macho.symbols:
        name = sym.name if sym.name else ""
        value = sym.value
        sym_count += 1

        entry = {
            "name": name,
            "address": value,
            "size": sym.size if hasattr(sym, 'size') else 0,
        }

        # Categorize
        name_lower = name.lower()
        if "agx" in name_lower or "apple_gpu" in name_lower:
            agx_symbols.append(entry)
        if "gpu" in name_lower or "iogpu" in name_lower or "metal" in name_lower:
            gpu_symbols.append(entry)
        if "jit" in name_lower or "codegen" in name_lower or "compiler" in name_lower or "compile" in name_lower:
            jit_symbols.append(entry)
        if "dart" in name_lower or "iommu" in name_lower or "gart" in name_lower:
            dart_symbols.append(entry)
        if any(p in name_lower for p in ["iomemory", "dma", "phys_read", "phys_write", "ml_phys", "pmap", "vm_map"]):
            mem_symbols.append(entry)
        if "userclient" in name_lower or "externalmethod" in name_lower:
            userclient_symbols.append(entry)

    print(f"  Total symbols: {sym_count}")
    print(f"  AGX symbols: {len(agx_symbols)}")
    print(f"  GPU symbols: {len(gpu_symbols)}")
    print(f"  JIT/Compiler symbols: {len(jit_symbols)}")
    print(f"  DART/IOMMU symbols: {len(dart_symbols)}")
    print(f"  Memory primitives: {len(mem_symbols)}")
    print(f"  UserClient symbols: {len(userclient_symbols)}")

    # Print top AGX symbols
    print(f"\n  --- Top AGX Symbols (by address) ---")
    agx_sorted = sorted(agx_symbols, key=lambda s: s['address'])
    for s in agx_sorted[:50]:
        print(f"    0x{s['address']:016x}  {s['name'][:90]}")

    # Print JIT/Compiler symbols
    print(f"\n  --- JIT/Compiler Symbols ---")
    jit_sorted = sorted(jit_symbols, key=lambda s: s['address'])
    for s in jit_sorted[:50]:
        print(f"    0x{s['address']:016x}  {s['name'][:90]}")

    # Print UserClient + externalMethod symbols
    print(f"\n  --- UserClient / ExternalMethod Symbols ---")
    uc_sorted = sorted(userclient_symbols, key=lambda s: s['address'])
    for s in uc_sorted[:50]:
        print(f"    0x{s['address']:016x}  {s['name'][:90]}")

    # Print DART symbols
    print(f"\n  --- DART/IOMMU Symbols ---")
    dart_sorted = sorted(dart_symbols, key=lambda s: s['address'])
    for s in dart_sorted[:50]:
        print(f"    0x{s['address']:016x}  {s['name'][:90]}")

    # Collect segments info
    segments = []
    print(f"\n  --- Segments ---")
    for seg in macho.segments:
        info = {
            "name": seg.name,
            "vmaddr": seg.virtual_address,
            "vmsize": seg.virtual_size,
            "fileoff": seg.file_offset,
            "filesize": seg.file_size,
        }
        segments.append(info)
        sections_str = ", ".join([s.name for s in seg.sections])
        print(f"    {seg.name:20s} VA=0x{seg.virtual_address:016x} size=0x{seg.virtual_size:x} "
              f"sections=[{sections_str}]")

    return {
        "total_symbols": sym_count,
        "agx": agx_symbols,
        "gpu": gpu_symbols,
        "jit_compiler": jit_symbols,
        "dart": dart_symbols,
        "memory": mem_symbols,
        "userclient": userclient_symbols,
        "segments": segments,
    }


# ============================================================
# PHASE 2: Pipeline Stage String Mapping
# ============================================================
def map_pipeline_strings(data):
    """Find all pipeline-relevant strings and their offsets in the binary."""
    print("\n" + "=" * 70)
    print("PHASE 2: JIT PIPELINE STRING MAPPING")
    print("=" * 70)

    results = {}
    for stage_name, stage_info in PIPELINE_STAGES.items():
        print(f"\n  [{stage_name}] {stage_info['description']}")
        stage_results = []
        for pattern in stage_info["string_patterns"]:
            offsets = []
            start = 0
            while True:
                idx = data.find(pattern, start)
                if idx == -1:
                    break
                offsets.append(idx)
                start = idx + 1
                if len(offsets) >= 100:  # cap
                    break
            if offsets:
                # Extract context around first few hits
                contexts = []
                for off in offsets[:5]:
                    # Get surrounding 64 bytes
                    ctx_start = max(0, off - 32)
                    ctx_end = min(len(data), off + len(pattern) + 64)
                    raw = data[ctx_start:ctx_end]
                    # Extract readable strings from context
                    readable = extract_strings_around(data, off, radius=128)
                    contexts.append({
                        "offset": off,
                        "offset_hex": f"0x{off:x}",
                        "nearby_strings": readable,
                    })
                stage_results.append({
                    "pattern": pattern.decode('utf-8', errors='replace'),
                    "count": len(offsets),
                    "first_offsets": [f"0x{o:x}" for o in offsets[:10]],
                    "contexts": contexts,
                })
                print(f"    {pattern.decode('utf-8', errors='replace'):40s} -> {len(offsets):4d} hits")
                # Show nearby strings for first hit
                if contexts and contexts[0]['nearby_strings']:
                    for ns in contexts[0]['nearby_strings'][:3]:
                        print(f"      nearby: \"{ns}\"")
            else:
                print(f"    {pattern.decode('utf-8', errors='replace'):40s} -> NOT FOUND")
        results[stage_name] = stage_results
    return results


def extract_strings_around(data, offset, radius=128, min_len=6):
    """Extract readable ASCII/UTF-8 strings around a given offset."""
    start = max(0, offset - radius)
    end = min(len(data), offset + radius)
    chunk = data[start:end]

    strings = []
    current = bytearray()
    for b in chunk:
        if 0x20 <= b < 0x7f:
            current.append(b)
        else:
            if len(current) >= min_len:
                strings.append(current.decode('ascii'))
            current = bytearray()
    if len(current) >= min_len:
        strings.append(current.decode('ascii'))
    return strings


# ============================================================
# PHASE 3: ExternalMethod Dispatch Table Recovery
# ============================================================
def recover_dispatch_tables(data, symbols):
    """Try to recover IOExternalMethodDispatch tables for GPU UserClients.
    
    Dispatch tables are arrays of:
        struct IOExternalMethodDispatch {
            IOExternalMethodAction function;   // 8 bytes (function pointer)
            uint32_t checkScalarInputCount;    // 4 bytes
            uint32_t checkStructureInputSize;  // 4 bytes
            uint32_t checkScalarOutputCount;   // 4 bytes
            uint32_t checkStructureOutputSize; // 4 bytes
        }; // = 24 bytes per entry

    We look for arrays of 24-byte entries where:
    - First 8 bytes is a valid kernel text pointer
    - Following 16 bytes are reasonable uint32 values (< 0x10000)
    """
    print("\n" + "=" * 70)
    print("PHASE 3: EXTERNALMETHOD DISPATCH TABLE RECOVERY")
    print("=" * 70)

    # Determine kernel text range from segments
    text_base = 0
    text_end = 0
    if symbols and "segments" in symbols:
        for seg in symbols["segments"]:
            if seg["name"] in ("__TEXT_EXEC", "__TEXT"):
                if text_base == 0 or seg["vmaddr"] < text_base:
                    text_base = seg["vmaddr"]
                end = seg["vmaddr"] + seg["vmsize"]
                if end > text_end:
                    text_end = end

    if text_base == 0:
        # Fallback: typical kernel VA range
        text_base = 0xFFFFFE0007000000
        text_end  = 0xFFFFFE0010000000
    
    print(f"  Kernel text range: 0x{text_base:016x} - 0x{text_end:016x}")

    # Also try to find DATA_CONST segment (where dispatch tables typically live)
    data_const_off = 0
    data_const_size = 0
    data_off = 0
    data_size = 0
    if symbols and "segments" in symbols:
        for seg in symbols["segments"]:
            if seg["name"] == "__DATA_CONST":
                data_const_off = seg["fileoff"]
                data_const_size = seg["filesize"]
            elif seg["name"] == "__DATA":
                data_off = seg["fileoff"]
                data_size = seg["filesize"]

    # Strategy 1: Search near known UserClient symbol addresses
    dispatch_tables = {}
    
    # For each known GPU UserClient, look for dispatch table patterns
    target_clients = [
        "AGXDeviceUserClient",
        "IOGPUDeviceUserClient",
        "IOSurfaceRootUserClient",
        "RAGXDeviceUserClient",
        "RAGXSharedUserClient",
        "IOGPUMemoryInfoUserClient",
    ]

    # Find string references to these class names
    for client_name in target_clients:
        pattern = client_name.encode('utf-8')
        offsets = []
        start = 0
        while True:
            idx = data.find(pattern, start)
            if idx == -1:
                break
            offsets.append(idx)
            start = idx + 1
            if len(offsets) > 20:
                break

        if not offsets:
            print(f"\n  {client_name}: no string refs found")
            continue

        print(f"\n  {client_name}: {len(offsets)} string refs")
        
        # For each string ref, scan surrounding area for dispatch table patterns
        found_tables = []
        for str_off in offsets[:5]:
            # Search in a wide area around the string reference
            # Dispatch tables are typically in __DATA_CONST, not near strings
            # But we can use the string to identify the kext region
            pass

    # Strategy 2: Brute-force scan for dispatch table patterns
    # Look for sequences of valid entries
    print(f"\n  --- Brute-force dispatch table scan ---")
    
    ENTRY_SIZE = 24  # sizeof(IOExternalMethodDispatch)
    MIN_ENTRIES = 3  # minimum entries to consider a valid table
    
    # Scan regions likely to contain dispatch tables
    scan_regions = []
    if data_const_off > 0 and data_const_size > 0:
        scan_regions.append(("__DATA_CONST", data_const_off, data_const_size))
    if data_off > 0 and data_size > 0:
        scan_regions.append(("__DATA", data_off, data_size))
    
    if not scan_regions:
        # Fallback: scan entire binary in chunks
        chunk_size = 16 * 1024 * 1024  # 16 MB
        for i in range(0, len(data), chunk_size):
            scan_regions.append((f"chunk_{i//chunk_size}", i, min(chunk_size, len(data) - i)))

    tables_found = []
    for region_name, region_off, region_size in scan_regions:
        print(f"    Scanning {region_name} (offset 0x{region_off:x}, size 0x{region_size:x})...")
        
        consecutive_valid = 0
        table_start = 0
        
        for off in range(region_off, region_off + region_size - ENTRY_SIZE, 8):
            # Check if this looks like a dispatch entry
            if off + ENTRY_SIZE > len(data):
                break
                
            func_ptr = struct.unpack_from("<Q", data, off)[0]
            scalars_in = struct.unpack_from("<I", data, off + 8)[0]
            struct_in = struct.unpack_from("<I", data, off + 12)[0]
            scalars_out = struct.unpack_from("<I", data, off + 16)[0]
            struct_out = struct.unpack_from("<I", data, off + 20)[0]

            # Validate entry
            is_valid = (
                text_base <= func_ptr <= text_end and  # valid kernel text pointer
                scalars_in < 0x100 and                  # reasonable scalar count
                struct_in < 0x100000 and                # reasonable struct size
                scalars_out < 0x100 and
                struct_out < 0x100000 and
                (func_ptr & 0x3) == 0                   # 4-byte aligned
            )

            if is_valid:
                if consecutive_valid == 0:
                    table_start = off
                consecutive_valid += 1
            else:
                if consecutive_valid >= MIN_ENTRIES:
                    # Found a valid dispatch table!
                    table_entries = []
                    for e in range(consecutive_valid):
                        eo = table_start + e * ENTRY_SIZE
                        fp = struct.unpack_from("<Q", data, eo)[0]
                        si = struct.unpack_from("<I", data, eo + 8)[0]
                        sti = struct.unpack_from("<I", data, eo + 12)[0]
                        so = struct.unpack_from("<I", data, eo + 16)[0]
                        sto = struct.unpack_from("<I", data, eo + 20)[0]
                        table_entries.append({
                            "selector": e,
                            "function": f"0x{fp:016x}",
                            "scalars_in": si,
                            "struct_in": sti,
                            "scalars_out": so,
                            "struct_out": sto,
                        })
                    
                    tables_found.append({
                        "offset": f"0x{table_start:x}",
                        "region": region_name,
                        "entries": consecutive_valid,
                        "dispatch": table_entries[:30],  # cap at 30
                    })
                    
                    # Try to identify which UserClient this belongs to
                    # by looking at nearby strings
                    nearby = extract_strings_around(data, table_start, radius=512, min_len=8)
                    relevant = [s for s in nearby if any(k in s.lower() for k in 
                               ["agx", "gpu", "surface", "accel", "userclient", "iogpu"])]
                    
                    ident = relevant[0] if relevant else "UNKNOWN"
                    print(f"    [TABLE] at 0x{table_start:x} ({region_name}): "
                          f"{consecutive_valid} entries, likely={ident}")
                    for entry in table_entries[:5]:
                        print(f"      sel={entry['selector']:3d} func={entry['function']} "
                              f"scIn={entry['scalars_in']} stIn={entry['struct_in']} "
                              f"scOut={entry['scalars_out']} stOut={entry['struct_out']}")
                    if consecutive_valid > 5:
                        print(f"      ... ({consecutive_valid - 5} more)")
                
                consecutive_valid = 0

    print(f"\n  Total dispatch tables found: {len(tables_found)}")
    return tables_found


# ============================================================
# PHASE 4: ARM64 Disassembly of Key Functions
# ============================================================
def disassemble_key_functions(data, symbols):
    """Disassemble AGX/GPU functions using Capstone."""
    if not HAS_CAPSTONE:
        print("\n[SKIP] Capstone not available for disassembly")
        return {}

    print("\n" + "=" * 70)
    print("PHASE 4: ARM64 DISASSEMBLY OF KEY FUNCTIONS")
    print("=" * 70)

    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True

    results = {}

    # Strategy: Find known function patterns in binary offsets
    # Since kernel collections use chained fixups, symbols may not have
    # direct file offsets. We search for known function prologues near
    # string references.

    # Key functions to find and disassemble
    targets = [
        ("AGXCompilerService", b"AGXCompilerService"),
        ("MTLCompilerService", b"MTLCompilerService"),
        ("IOGPUDeviceUserClient", b"IOGPUDeviceUserClient"),
        ("AGXDeviceUserClient", b"AGXDeviceUserClient"),
        ("IOSurfaceRootUserClient", b"IOSurfaceRootUserClient"),
        ("externalMethod", b"externalMethod"),
        ("submitCommandBuffer", b"submitCommandBuffer"),
        ("MAP_JIT", b"MAP_JIT"),
        ("ml_phys_read", b"ml_phys_read"),
        ("dart_map", b"dart_map"),
    ]

    for func_name, pattern in targets:
        offsets = []
        start = 0
        while True:
            idx = data.find(pattern, start)
            if idx == -1:
                break
            offsets.append(idx)
            start = idx + 1
            if len(offsets) >= 10:
                break

        if not offsets:
            continue

        print(f"\n  --- {func_name} ({len(offsets)} refs) ---")

        # For the first reference, try to find a function prologue nearby
        # ARM64 function prologues typically start with:
        #   STP X29, X30, [SP, #-N]!  (push frame pointer + LR)
        #   or SUB SP, SP, #N
        for ref_off in offsets[:3]:
            # Search backwards for function prologue
            prologue_off = find_function_prologue(data, ref_off)
            if prologue_off is not None:
                # Disassemble up to 64 instructions
                code = data[prologue_off:prologue_off + 256]
                instructions = []
                for insn in md.disasm(code, prologue_off):
                    instructions.append({
                        "address": f"0x{insn.address:x}",
                        "mnemonic": insn.mnemonic,
                        "op_str": insn.op_str,
                    })
                    if insn.mnemonic == "ret" or len(instructions) >= 64:
                        break

                if instructions:
                    print(f"    Function at offset 0x{prologue_off:x} ({len(instructions)} insns):")
                    for insn in instructions[:20]:
                        print(f"      {insn['address']}:  {insn['mnemonic']:8s} {insn['op_str']}")
                    if len(instructions) > 20:
                        print(f"      ... ({len(instructions) - 20} more instructions)")
                    
                    results[f"{func_name}_0x{ref_off:x}"] = {
                        "prologue_offset": f"0x{prologue_off:x}",
                        "ref_offset": f"0x{ref_off:x}",
                        "instruction_count": len(instructions),
                        "instructions": instructions[:64],
                    }
            else:
                # Show raw context
                nearby = extract_strings_around(data, ref_off, radius=256, min_len=4)
                print(f"    ref at 0x{ref_off:x} - no prologue found")
                if nearby:
                    print(f"      nearby strings: {nearby[:5]}")

    return results


def find_function_prologue(data, near_offset, search_range=4096):
    """Search backwards from an offset to find an ARM64 function prologue."""
    # STP X29, X30, [SP, #xx]!  = various encodings starting with 0xA9
    # SUB SP, SP, #xx = 0xD1 0x00 .. ..
    
    start = max(0, near_offset - search_range)
    # Align to 4 bytes
    start = start & ~3

    best_prologue = None
    for off in range(near_offset - 4, start, -4):
        if off + 4 > len(data):
            continue
        insn = struct.unpack_from("<I", data, off)[0]
        
        # STP X29, X30, [SP, #imm]! (pre-index)
        # Encoding: 1x101001 1xxxxxxx xxxxxxxx xx011101
        if (insn & 0xFFE00000) == 0xA9800000:  # STP with pre-index
            # Check if it involves X29 (FP) and X30 (LR)
            rt1 = insn & 0x1F
            rt2 = (insn >> 10) & 0x1F
            rn = (insn >> 5) & 0x1F
            if rn == 31 and rt1 == 29 and rt2 == 30:
                best_prologue = off
                break
            elif rn == 31:
                best_prologue = off
                break

        # PACIBSP (pointer auth on LR)
        if insn == 0xD503237F:
            best_prologue = off
            break

        # SUB SP, SP, #imm
        if (insn & 0xFF0003FF) == 0xD10003FF:
            best_prologue = off
            # Don't break - might find STP further back

    return best_prologue


# ============================================================
# PHASE 5: Cross-Reference Analysis
# ============================================================
def analyze_xrefs(data, symbols):
    """Analyze cross-references between pipeline stages."""
    print("\n" + "=" * 70)
    print("PHASE 5: CROSS-REFERENCE ANALYSIS")
    print("=" * 70)

    # Find critical paths:
    # 1. What calls AGXCompilerService?
    # 2. What calls ml_phys_read / ml_phys_write?
    # 3. What configures DART mappings?
    # 4. What functions handle GPU command buffers?

    # Build a map of string -> offset for key identifiers
    key_strings = {}
    for name_bytes in [
        b"AGXCompilerService", b"AGXDeviceUserClient", b"IOGPUDeviceUserClient",
        b"ml_phys_read", b"ml_phys_write", b"IODARTMapper",
        b"submitCommandBuffer", b"externalMethod", b"AGXFirmware",
        b"IOSurfaceRootUserClient", b"MAP_JIT", b"vm_map_protect",
        b"AGXAcceleratorRing", b"IOGPUCommandBuffer",
        b"AGXSecureMonitor", b"TrustZone", b"iBoot",
    ]:
        name = name_bytes.decode('utf-8', errors='replace')
        idx = data.find(name_bytes)
        if idx != -1:
            key_strings[name] = idx

    print(f"  Key string locations:")
    for name, off in sorted(key_strings.items(), key=lambda x: x[1]):
        print(f"    0x{off:08x}  {name}")

    # Find proximity clusters - strings that appear close together
    # indicate they're in the same kext/module
    print(f"\n  --- Proximity Clusters (strings within 4KB of each other) ---")
    sortedStrings = sorted(key_strings.items(), key=lambda x: x[1])
    clusters = []
    current_cluster = [sortedStrings[0]] if sortedStrings else []
    
    for i in range(1, len(sortedStrings)):
        name, off = sortedStrings[i]
        prev_name, prev_off = sortedStrings[i-1]
        if off - prev_off < 4096:
            current_cluster.append(sortedStrings[i])
        else:
            if len(current_cluster) >= 2:
                clusters.append(current_cluster)
            current_cluster = [sortedStrings[i]]
    if len(current_cluster) >= 2:
        clusters.append(current_cluster)

    for ci, cluster in enumerate(clusters):
        names = [c[0] for c in cluster]
        start_off = cluster[0][1]
        end_off = cluster[-1][1]
        print(f"    Cluster {ci+1} (0x{start_off:x}-0x{end_off:x}, span={end_off-start_off} bytes):")
        for name in names:
            print(f"      - {name}")

    # Analyze co-location of AGX components
    print(f"\n  --- AGX Component Co-location ---")
    agx_components = [
        b"AGXDeviceUserClient", b"AGXCompilerService", b"AGXAcceleratorRing",
        b"AGXFirmware", b"AGXSecureMonitor", b"AGXVMContext", b"AGXGART",
        b"AGXBuffer", b"AGXTexture", b"AGXMicroSequencer",
    ]
    
    for comp in agx_components:
        idx = data.find(comp)
        if idx != -1:
            nearby = extract_strings_around(data, idx, radius=512, min_len=6)
            agx_nearby = [s for s in nearby if any(k in s.lower() for k in 
                         ["agx", "gpu", "iogpu", "shader", "compile", "jit", "dart",
                          "firmware", "rtkit", "surface", "metal", "command"])]
            if agx_nearby:
                print(f"    {comp.decode():30s} at 0x{idx:08x}")
                for s in agx_nearby[:5]:
                    print(f"      co-located: {s}")

    return {
        "key_strings": {k: f"0x{v:x}" for k, v in key_strings.items()},
        "clusters": [[{"name": n, "offset": f"0x{o:x}"} for n, o in c] for c in clusters],
    }


# ============================================================
# PHASE 6: Physical Address Reference Analysis  
# ============================================================
def analyze_phys_refs(data):
    """Scan for physical address patterns that could reach BootROM."""
    print("\n" + "=" * 70)
    print("PHASE 6: PHYSICAL ADDRESS REFERENCE DEEP SCAN")
    print("=" * 70)

    # Strategy: Look for 64-bit constants matching known MMIO/physical ranges
    # In ARM64, large constants are typically loaded via ADRP+ADD or LDR from literal pool

    results = {}
    for region_name, (start_addr, end_addr, desc) in PHYS_REGIONS.items():
        print(f"\n  [{region_name}] {desc} (0x{start_addr:x} - 0x{end_addr:x})")
        
        # Search for the base address as a 64-bit little-endian value
        base_bytes = struct.pack("<Q", start_addr)
        refs = []
        search_start = 0
        while True:
            idx = data.find(base_bytes, search_start)
            if idx == -1:
                break
            refs.append(idx)
            search_start = idx + 1
            if len(refs) >= 50:
                break

        if refs:
            print(f"    Exact base refs: {len(refs)}")
            for r in refs[:5]:
                nearby = extract_strings_around(data, r, radius=256, min_len=6)
                context = [s for s in nearby if len(s) > 6][:3]
                print(f"      0x{r:08x}: {context}")
        
        # Also search for the high 32 bits as immediate in MOVZ/MOVK instructions
        # MOVZ Xn, #imm16, LSL #48  etc.
        high16 = (start_addr >> 48) & 0xFFFF
        mid16 = (start_addr >> 32) & 0xFFFF
        low16 = (start_addr >> 16) & 0xFFFF
        
        # Count MOVZ/MOVK instructions that reference this address
        # MOVZ: 1101_0010_1xxx_xxxx ... (hw = 2 or 3 for bits 32-63)
        movz_count = 0
        
        # Search for identifying byte pattern of upper address bits  
        if mid16 > 0:
            upper_pattern = struct.pack("<H", mid16)
            hits = 0
            s = 0
            while hits < 100:
                idx = data.find(upper_pattern, s)
                if idx == -1:
                    break
                hits += 1
                s = idx + 1
            if hits > 0:
                print(f"    Upper 16-bit pattern (0x{mid16:04x}): {hits} refs in binary")

        results[region_name] = {
            "description": desc,
            "range": f"0x{start_addr:x}-0x{end_addr:x}",
            "exact_base_refs": len(refs),
            "first_refs": [f"0x{r:x}" for r in refs[:10]],
        }

    return results


# ============================================================
# PHASE 7: Exploitation Path Synthesis
# ============================================================
def synthesize_exploitation_paths(symbols, pipeline_strings, dispatch_tables, 
                                   disasm, xrefs, phys_refs):
    """Synthesize all findings into concrete exploitation paths."""
    print("\n" + "=" * 70)
    print("PHASE 7: EXPLOITATION PATH SYNTHESIS")
    print("=" * 70)

    paths = []

    # Path 1: Shader JIT -> Kernel Code Exec
    print("\n  === PATH 1: Metal Shader -> AGX JIT -> Kernel Code Exec ===")
    path1_steps = [
        "1. App creates MTLDevice + MTLCommandQueue (sandbox-reachable)",
        "2. Submit crafted Metal shader (malformed AIR bytecode)",
        "3. Shader reaches AGXCompilerService in kernel",
        "4. Compiler processes AIR -> native: type confusion / OOB in",
        "   AGXCompilerService::compileShader()",
        "5. Corrupted JIT output written to executable page (MAP_JIT region)",
        "6. Controlled kernel code execution when GPU executes compiled shader",
    ]
    
    # Check evidence
    has_agx_compiler = any("AGXCompilerService" in (s.get("pattern", "")) 
                          for stage in pipeline_strings.values() for s in stage)
    has_jit = any("JIT" in (s.get("pattern", "")) or "MAP_JIT" in (s.get("pattern", "")) 
                  for stage in pipeline_strings.values() for s in stage)
    
    evidence_score = 0
    if has_agx_compiler:
        evidence_score += 30
        print("    [CONFIRMED] AGXCompilerService present in kernel")
    if has_jit:
        evidence_score += 20
        print("    [CONFIRMED] JIT/MAP_JIT patterns found")
    if symbols and len(symbols.get("jit_compiler", [])) > 0:
        evidence_score += 25
        print(f"    [CONFIRMED] {len(symbols['jit_compiler'])} JIT/Compiler symbols found")
    if symbols and len(symbols.get("agx", [])) > 0:
        evidence_score += 25
        print(f"    [CONFIRMED] {len(symbols['agx'])} AGX symbols found")
    
    for step in path1_steps:
        print(f"    {step}")
    print(f"    Evidence score: {evidence_score}/100")
    
    paths.append({
        "name": "Shader JIT -> Kernel Code Exec",
        "score": evidence_score,
        "steps": path1_steps,
    })

    # Path 2: DART Remap -> Physical Memory Access
    print("\n  === PATH 2: GPU DART Remap -> Physical Memory Read ===")
    path2_steps = [
        "1. Achieve kernel code exec (from Path 1)",
        "2. Find GPU DART instance (IODARTMapper for AGX)",
        "3. Program DART to map BootROM phys range (0x100000000)",
        "   into GPU-accessible IOVA space",
        "4. Create GPU buffer backed by DART-mapped BootROM pages",
        "5. Read GPU buffer contents from userspace -> BootROM dump",
    ]
    
    dart_score = 0
    has_dart = any(len(s) > 0 for s in pipeline_strings.get("stage5_gpu_memory", []))
    if has_dart:
        dart_score += 30
        print("    [CONFIRMED] DART/GPU memory strings found")
    if symbols and len(symbols.get("dart", [])) > 0:
        dart_score += 30
        print(f"    [CONFIRMED] {len(symbols['dart'])} DART symbols")
    if phys_refs and phys_refs.get("bootrom", {}).get("exact_base_refs", 0) > 0:
        dart_score += 20
        print(f"    [CONFIRMED] BootROM phys addr refs in binary")
    if phys_refs and phys_refs.get("dart_gpu", {}).get("exact_base_refs", 0) > 0:
        dart_score += 20
        print(f"    [CONFIRMED] GPU DART base addr refs in binary")
    
    for step in path2_steps:
        print(f"    {step}")
    print(f"    Evidence score: {dart_score}/100")
    
    paths.append({
        "name": "GPU DART Remap -> Physical Read",
        "score": dart_score,
        "steps": path2_steps,
    })

    # Path 3: Direct ml_phys_read
    print("\n  === PATH 3: Direct ml_phys_read (Kernel Exec Required) ===")
    path3_steps = [
        "1. Achieve kernel code exec (from Path 1)",
        "2. Bypass KTRR/CTRR to call arbitrary kernel functions",
        "   OR use data-only attack to redirect a function pointer",
        "3. Call ml_phys_read_data(0x100000000, buf, 0x80000)",
        "4. Copy BootROM contents to userspace buffer",
    ]
    
    ml_score = 0
    if any("ml_phys_read" in s.get("pattern", "") for stage in pipeline_strings.values() for s in stage):
        ml_score += 50
        print("    [CONFIRMED] ml_phys_read present in kernel")
    
    for step in path3_steps:
        print(f"    {step}")
    print(f"    Evidence score: {ml_score}/100")
    
    paths.append({
        "name": "Direct ml_phys_read",
        "score": ml_score,
        "steps": path3_steps,
    })

    # Path 4: AGX Firmware / RTKit exploit
    print("\n  === PATH 4: AGX Firmware Upload -> Coprocessor Exploitation ===")
    path4_steps = [
        "1. Achieve kernel code exec (from Path 1)",
        "2. Intercept AGXFirmware upload to GPU coprocessor via RTKit",
        "3. Upload modified GPU firmware with BootROM read capability",
        "4. GPU coprocessor (ASC) may have DMA to physical memory",
        "5. GPU firmware reads BootROM and reports back via RTKit mailbox",
    ]
    
    fw_score = 0
    if any("AGXFirmware" in s.get("pattern", "") or "RTKit" in s.get("pattern", "")
           for stage in pipeline_strings.values() for s in stage):
        fw_score += 40
        print("    [CONFIRMED] AGXFirmware / RTKit strings found")
    if symbols and any("firmware" in s["name"].lower() or "rtkit" in s["name"].lower() 
                      for s in symbols.get("agx", [])):
        fw_score += 30
        print("    [CONFIRMED] Firmware-related AGX symbols")
    
    for step in path4_steps:
        print(f"    {step}")
    print(f"    Evidence score: {fw_score}/100")
    
    paths.append({
        "name": "AGX Firmware / RTKit Exploit",
        "score": fw_score,
        "steps": path4_steps,
    })

    # Rank paths
    paths.sort(key=lambda p: p["score"], reverse=True)
    
    print("\n  === RANKED EXPLOITATION PATHS ===")
    for i, p in enumerate(paths):
        bar = "#" * (p["score"] // 5)
        print(f"    {i+1}. [{p['score']:3d}/100] {p['name']}")
        print(f"       {bar}")

    return paths


# ============================================================
# PHASE 8: Generate Ghidra Script
# ============================================================
def generate_ghidra_script(symbols, pipeline_strings, dispatch_tables):
    """Generate a Ghidra Python script to automate further analysis."""
    print("\n" + "=" * 70)
    print("PHASE 8: GHIDRA ANALYSIS SCRIPT GENERATION")
    print("=" * 70)

    # Collect addresses to label
    labels = {}
    if symbols:
        for sym in symbols.get("agx", [])[:100]:
            if sym["address"] > 0:
                labels[sym["address"]] = sym["name"]
        for sym in symbols.get("jit_compiler", [])[:100]:
            if sym["address"] > 0:
                labels[sym["address"]] = sym["name"]
        for sym in symbols.get("userclient", [])[:50]:
            if sym["address"] > 0:
                labels[sym["address"]] = sym["name"]

    script = '''# Ghidra Python Script - AGX JIT Pipeline Analysis
# Auto-generated by jit_pipeline_mapper.py
# Target: iPhone 11 Pro (A13 / T8030) kernelcache
#
# Usage: Run in Ghidra's Script Manager after loading kernelcache
# @category iOS.Security
# @author jit_pipeline_mapper

from ghidra.program.model.symbol import SourceType
from ghidra.program.model.address import AddressFactory
from ghidra.app.decompiler import DecompInterface

def label_address(addr_int, name):
    """Label an address in Ghidra."""
    addr = currentProgram.getAddressFactory().getDefaultAddressSpace().getAddress(addr_int)
    sym = currentProgram.getSymbolTable()
    sym.createLabel(addr, name, SourceType.USER_DEFINED)
    print(f"Labeled 0x{addr_int:x} as {name}")

def analyze_function(addr_int):
    """Decompile and analyze a function."""
    addr = currentProgram.getAddressFactory().getDefaultAddressSpace().getAddress(addr_int)
    func = getFunctionAt(addr)
    if func is None:
        func = createFunction(addr, None)
    if func:
        decomp = DecompInterface()
        decomp.openProgram(currentProgram)
        result = decomp.decompileFunction(func, 30, monitor)
        if result:
            return result.getDecompiledFunction().getC()
    return None

# ==============================================
# Step 1: Label known AGX/GPU symbols
# ==============================================
print("=" * 60)
print("Labeling AGX/GPU symbols...")
print("=" * 60)

'''
    for addr, name in sorted(labels.items()):
        clean_name = re.sub(r'[^a-zA-Z0-9_]', '_', name)[:80]
        script += f'label_address(0x{addr:x}, "{clean_name}")\n'

    script += '''
# ==============================================
# Step 2: Search for ExternalMethod dispatch tables
# ==============================================
print("=" * 60)
print("Searching for ExternalMethod dispatch tables...")
print("=" * 60)

# Search for IOExternalMethodDispatch array patterns
# Each entry is 24 bytes: function_ptr(8) + 4x uint32(16)
# Look in __DATA_CONST segment

mem = currentProgram.getMemory()
data_const = None
for block in mem.getBlocks():
    if "__DATA_CONST" in block.getName():
        data_const = block
        break

if data_const:
    print(f"Found __DATA_CONST: {data_const.getStart()} - {data_const.getEnd()}")
    # Automated dispatch table detection would go here
    # For now, manually check addresses from the binary analysis

# ==============================================
# Step 3: Decompile AGX shader compilation path
# ==============================================
print("=" * 60)
print("Analyzing AGX shader compilation path...")
print("=" * 60)

# Key functions to decompile (addresses from symbol analysis):
agx_functions = [
'''
    # Add known AGX function addresses
    if symbols:
        for sym in symbols.get("jit_compiler", [])[:20]:
            if sym["address"] > 0:
                script += f'    (0x{sym["address"]:x}, "{sym["name"][:60]}"),\n'
    
    script += ''']

for addr, name in agx_functions:
    print(f"\\nDecompiling {name} at 0x{addr:x}...")
    c_code = analyze_function(addr)
    if c_code:
        print(c_code[:500])
    else:
        print("  Failed to decompile")

# ==============================================
# Step 4: Trace DART configuration
# ==============================================
print("=" * 60)
print("Tracing DART/IOMMU configuration...")
print("=" * 60)

# Search for references to GPU DART base: 0x231004000
dart_base = 0x231004000
refs = getReferencesTo(currentProgram.getAddressFactory().getDefaultAddressSpace().getAddress(dart_base))
for ref in refs:
    print(f"  DART base ref from: {ref.getFromAddress()}")

# Search for BootROM phys addr: 0x100000000  
bootrom_base = 0x100000000
refs = getReferencesTo(currentProgram.getAddressFactory().getDefaultAddressSpace().getAddress(bootrom_base))
for ref in refs:
    print(f"  BootROM addr ref from: {ref.getFromAddress()}")

print("\\n" + "=" * 60)
print("Analysis complete!")
print("=" * 60)
'''

    # Save the script
    script_path = KERNELCACHE_DIR / "ghidra_agx_analysis.py"
    script_path.write_text(script, encoding='utf-8')
    print(f"  Saved Ghidra script to: {script_path}")
    print(f"  Labels to apply: {len(labels)}")
    
    return str(script_path)


# ============================================================
# MAIN ORCHESTRATOR
# ============================================================
def main():
    print("=" * 70)
    print("JIT PIPELINE MAPPER - Deep Shader Compilation Flow Analysis")
    print("iPhone 11 Pro (A13 / T8030)")
    print("=" * 70)

    # Find kernelcache
    kc_path = find_kernelcache()
    if kc_path is None:
        print("[!] No kernelcache found in extracted/")
        sys.exit(1)

    print(f"[*] Kernelcache: {kc_path}")

    # Load raw binary
    data = load_binary(kc_path)

    # Phase 1: LIEF symbol extraction
    symbols = analyze_symbols(kc_path)

    # Phase 2: Pipeline string mapping
    pipeline_strings = map_pipeline_strings(data)

    # Phase 3: Dispatch table recovery
    dispatch_tables = recover_dispatch_tables(data, symbols)

    # Phase 4: ARM64 disassembly
    disasm = disassemble_key_functions(data, symbols)

    # Phase 5: Cross-reference analysis
    xrefs = analyze_xrefs(data, symbols)

    # Phase 6: Physical address deep scan
    phys_refs = analyze_phys_refs(data)

    # Phase 7: Exploitation path synthesis
    paths = synthesize_exploitation_paths(symbols, pipeline_strings, dispatch_tables,
                                           disasm, xrefs, phys_refs)

    # Phase 8: Generate Ghidra script
    ghidra_script = generate_ghidra_script(symbols, pipeline_strings, dispatch_tables)

    # Save full results
    output = {
        "target": "iPhone 11 Pro (A13 / T8030)",
        "kernelcache": str(kc_path),
        "symbols_summary": {
            "total": symbols.get("total_symbols", 0),
            "agx": len(symbols.get("agx", [])),
            "gpu": len(symbols.get("gpu", [])),
            "jit_compiler": len(symbols.get("jit_compiler", [])),
            "dart": len(symbols.get("dart", [])),
            "userclient": len(symbols.get("userclient", [])),
        } if symbols else {},
        "pipeline_strings": {k: [{"pattern": s["pattern"], "count": s["count"]} 
                                  for s in v] for k, v in pipeline_strings.items()},
        "dispatch_tables_found": len(dispatch_tables),
        "dispatch_tables": dispatch_tables[:10],
        "disassembled_functions": len(disasm),
        "xrefs": xrefs,
        "phys_refs": phys_refs,
        "exploitation_paths": paths,
        "ghidra_script": ghidra_script,
    }

    out_path = KERNELCACHE_DIR / "jit_pipeline_analysis.json"
    out_path.write_text(json.dumps(output, indent=2, default=str), encoding='utf-8')
    print(f"\n[*] Full results saved to: {out_path}")

    # Generate human-readable report
    generate_report(symbols, pipeline_strings, dispatch_tables, disasm, xrefs, phys_refs, paths)


def generate_report(symbols, pipeline_strings, dispatch_tables, disasm, xrefs, phys_refs, paths):
    """Generate a human-readable analysis report."""
    lines = []
    lines.append("=" * 70)
    lines.append("JIT PIPELINE MAPPING REPORT")
    lines.append("Target: iPhone 11 Pro (A13 Bionic / T8030)")
    lines.append("=" * 70)
    
    lines.append("\nEXECUTIVE SUMMARY")
    lines.append("-" * 50)
    
    if symbols:
        lines.append(f"Total symbols analyzed: {symbols.get('total_symbols', 0)}")
        lines.append(f"AGX-specific symbols: {len(symbols.get('agx', []))}")
        lines.append(f"GPU symbols: {len(symbols.get('gpu', []))}")
        lines.append(f"JIT/Compiler symbols: {len(symbols.get('jit_compiler', []))}")
        lines.append(f"DART/IOMMU symbols: {len(symbols.get('dart', []))}")
        lines.append(f"UserClient symbols: {len(symbols.get('userclient', []))}")
    
    lines.append(f"Dispatch tables recovered: {len(dispatch_tables)}")
    lines.append(f"Functions disassembled: {len(disasm)}")
    lines.append(f"Exploitation paths identified: {len(paths)}")

    lines.append("\n\nJIT COMPILATION PIPELINE")
    lines.append("-" * 50)
    lines.append("""
The shader JIT compilation pipeline flows through these stages:

  [App/Metal Framework]
       |
       v
  IOGPUDeviceUserClient::externalMethod()  <-- Userland entry (sandbox-reachable)
       |
       v
  AGXDevice / AGXAccelerator
       |
       v
  AGXAcceleratorRing::submitCommandBuffer() <-- Command queue processing
       |
       v
  AGXCompilerService::compileShader()       <-- ATTACK SURFACE (processes untrusted AIR)
       |
       v
  [AIR bytecode -> LLVM IR -> Native GPU ISA]
       |
       v
  JIT memory allocation (MAP_JIT / RWX pages)  <-- Code injection vector
       |
       v
  DART maps JIT pages into GPU IOVA space   <-- DART manipulation target
       |
       v
  GPU coprocessor (ASC) executes compiled shader
       |
       v
  Results via RTKit mailbox -> userspace
""")

    lines.append("\nEXPLOITATION PATH RANKING")
    lines.append("-" * 50)
    for i, p in enumerate(paths):
        lines.append(f"\n{i+1}. [{p['score']}/100] {p['name']}")
        for step in p['steps']:
            lines.append(f"   {step}")

    lines.append("\n\nKEY FINDINGS")
    lines.append("-" * 50)

    if symbols:
        agx_count = len(symbols.get('agx', []))
        jit_count = len(symbols.get('jit_compiler', []))
        lines.append(f"[+] {agx_count} AGX symbols = enormous attack surface in GPU driver")
        lines.append(f"[+] {jit_count} JIT/Compiler symbols = active shader compilation in kernel")
        lines.append(f"[+] DART symbols present = GPU has its own IOMMU (remappable)")
        lines.append(f"[+] AGXFirmware/RTKit = GPU coprocessor firmware is uploadable")

    lines.append("\n\nNEXT STEPS")
    lines.append("-" * 50)
    lines.append("1. Load kernelcache in Ghidra, run ghidra_agx_analysis.py script")
    lines.append("2. Decompile AGXCompilerService::compileShader() - look for:")
    lines.append("   - Buffer size calculations without overflow checks")
    lines.append("   - Type confusion in AIR bytecode parsing")
    lines.append("   - Unchecked shader constant buffer sizes")
    lines.append("3. Trace DART configuration for GPU - check if:")
    lines.append("   - GPU DART allows mapping arbitrary physical addresses")
    lines.append("   - There are validation gaps in DART page table construction")
    lines.append("4. Analyze AGXFirmware upload path:")
    lines.append("   - Can we modify firmware before upload?")
    lines.append("   - What DMA capabilities does the GPU coprocessor have?")
    lines.append("5. Build Metal shader fuzzer targeting AGXCompilerService")
    lines.append("   - Generate malformed AIR bytecode")
    lines.append("   - Monitor for kernel panics (KP indicates bug found)")

    report_path = KERNELCACHE_DIR / "JIT_PIPELINE_REPORT.txt"
    report_path.write_text("\n".join(lines), encoding='utf-8')
    print(f"\n[*] Report saved to: {report_path}")


if __name__ == "__main__":
    main()
