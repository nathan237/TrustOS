#!/usr/bin/env python3
"""
IOSurface Deep Analyzer - Chain B Stage 1 Preparation
======================================================
Deep analysis of IOSurfaceRootUserClient in the kernelcache for:
1. ExternalMethod dispatch table recovery (selectors + input validation)
2. Shared memory management code paths (race condition targets)
3. Property get/set handlers (R/W primitive candidates)
4. Size/offset calculation patterns (integer overflow targets)
5. Concrete attack surface map for Chain B exploitation

Target: iPhone 11 Pro (A13 / T8030), iOS 26.3
"""

import struct
import re
import json
import sys
from pathlib import Path
from collections import defaultdict

try:
    from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM
    HAS_CAPSTONE = True
except ImportError:
    HAS_CAPSTONE = False

EXTRACTED = Path("extracted")

# ============================================================
# IOSurface known selectors (from XNU open-source + prior research)
# These are the externalMethod selectors for IOSurfaceRootUserClient
# Selector numbers may shift between iOS versions
# ============================================================
KNOWN_IOSURFACE_SELECTORS = {
    0:  ("s_create_surface",        "Create new IOSurface - MAIN ENTRY POINT"),
    1:  ("s_release_surface",       "Release surface reference"),
    2:  ("s_lock_surface",          "Lock surface for CPU access"),
    3:  ("s_unlock_surface",        "Unlock surface"),
    4:  ("s_get_property",          "Get surface property - potential R primitive"),
    5:  ("s_set_property",          "Set surface property - potential W primitive"),
    6:  ("s_copy_property",         "Copy property between surfaces"),
    7:  ("s_remove_property",       "Remove property from surface"),
    8:  ("s_increment_use_count",   "Increment reference count"),
    9:  ("s_decrement_use_count",   "Decrement reference count - UAF target"),
    10: ("s_get_values",            "Get multiple values - bulk read"),
    11: ("s_set_values",            "Set multiple values - bulk write"),
    12: ("s_lookup_surface",        "Lookup by global ID - cross-process"),
    13: ("s_get_surface_count",     "Get open surface count"),
    14: ("s_get_surface_info",      "Get surface metadata"),
    15: ("s_create_shared",         "Create shared memory region"),
    16: ("s_set_notify",            "Set notification port"),
    17: ("s_remove_notify",         "Remove notification"),
    18: ("s_set_compressed_tile_data", "Compressed tile data - complex parsing"),
    19: ("s_get_compressed_tile_data", "Get compressed tile data"),
}

# Known vulnerability patterns in IOSurface (from public CVEs)
VULN_PATTERNS = {
    "integer_overflow": {
        "description": "Integer overflow in size calculations",
        "strings": [
            b"CSBufferPitch",           # Buffer pitch calculation
            b"bytesPerRow",             # Row size calculation
            b"allocationSize",          # Total allocation size
            b"totalSize",              
            b"pixelFormat",             # Format-dependent size
            b"width",                   # Dimension validation
            b"height",
            b"bytesPerElement",
            b"elementWidth",
            b"elementHeight",
        ],
        "relevance": "Size calculations without overflow checks -> heap overflow",
    },
    "race_condition": {
        "description": "TOCTOU / double-fetch in shared memory",
        "strings": [
            b"IOSurfaceSharedListEntry",    # Shared list management
            b"shared memory region",         # Shared mem setup
            b"no longer mapped",             # Stale mapping check
            b"ownership",                    # Ownership transfer
            b"prepare",                      # Memory prepare/complete
            b"complete",
            b"IOBufferMemoryDescriptor",     # Backing allocation
        ],
        "relevance": "Race between validation and use of shared memory -> UAF",
    },
    "use_after_free": {
        "description": "Reference counting issues",
        "strings": [
            b"use_count",
            b"retain",
            b"release", 
            b"dealloc",
            b"free",
            b"destroy",
            b"os_refcnt",
            b"resurrection",            # Ref count resurrection bug
        ],
        "relevance": "Unbalanced retain/release -> UAF -> controlled reuse",
    },
    "type_confusion": {
        "description": "Property type confusion",
        "strings": [
            b"OSSerialize",
            b"OSUnserialize",
            b"OSDictionary",
            b"OSData",
            b"OSNumber",
            b"OSString",
            b"xml_data",
            b"property_type",
        ],
        "relevance": "Deserialize untrusted data -> type confusion in properties",
    },
    "global_namespace": {
        "description": "Cross-process surface access via global ID",
        "strings": [
            b"global",
            b"insecure",
            b"lookup",
            b"surface_id",
            b"any other process",
        ],
        "relevance": "Global lookups allow cross-process attacks without sandbox escape",
    },
}


def load_kernelcache():
    """Find and load the raw kernelcache."""
    for f in EXTRACTED.iterdir():
        if f.name.endswith(".raw") and "kernelcache" in f.name:
            print(f"[*] Loading: {f} ({f.stat().st_size / 1024 / 1024:.1f} MB)")
            return f.read_bytes(), f
    print("[!] No kernelcache found")
    sys.exit(1)


def extract_strings_at(data, offset, radius=256, min_len=6):
    """Extract readable strings around an offset."""
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
# PHASE 1: IOSurface String Mapping
# ============================================================
def map_iosurface_strings(data):
    """Find all IOSurface-related strings and their locations."""
    print("\n" + "=" * 70)
    print("PHASE 1: IOSURFACE STRING MAPPING")
    print("=" * 70)

    # Core IOSurface strings to find
    core_strings = [
        b"IOSurfaceRootUserClient",
        b"IOSurfaceRoot",
        b"IOSurface",
        b"s_create_surface",
        b"s_release_surface",
        b"s_lock_surface",
        b"s_unlock_surface",
        b"s_get_value",
        b"s_set_value",
        b"s_get_values",
        b"s_set_values",
        b"s_copy_value",
        b"s_remove_value",
        b"s_lookup",
        b"s_increment_use_count",
        b"s_decrement_use_count",
        b"IOSurfaceClient",
        b"IOSurfaceSharedListEntry",
        b"IOSurfaceSendRight",
        b"CSBufferPitch",
        b"bytesPerRow",
        b"bytesPerElement",
        b"allocationSize",
        b"IOSurfaceDefaultLockingSingleThread",
        b"IOSurfaceUserClientDefaultLocking",
        b"serClientDefaultLockingSingleThreadExternalMethod",
        b"global (insecure) IOSurface lookups",
        b"creating global IOSurfaces accessible to any other process",
    ]

    results = {}
    print(f"\n  Core IOSurface strings:")
    for pat in core_strings:
        offsets = []
        start = 0
        while True:
            idx = data.find(pat, start)
            if idx == -1:
                break
            offsets.append(idx)
            start = idx + 1
            if len(offsets) >= 50:
                break
        
        name = pat.decode('utf-8', errors='replace')
        if offsets:
            results[name] = offsets
            print(f"    {name:55s} -> {len(offsets):3d} hits  (first: 0x{offsets[0]:x})")
        else:
            print(f"    {name:55s} -> NOT FOUND")

    return results


# ============================================================
# PHASE 2: IOSurface Code Region Identification
# ============================================================
def identify_iosurface_region(data, string_map):
    """Identify the contiguous code region containing IOSurface."""
    print("\n" + "=" * 70)
    print("PHASE 2: IOSURFACE CODE REGION IDENTIFICATION")
    print("=" * 70)

    # Collect all IOSurface string offsets
    all_offsets = []
    for name, offsets in string_map.items():
        if "IOSurface" in name or "s_create" in name or "s_set_value" in name:
            all_offsets.extend(offsets)

    if not all_offsets:
        print("  [!] No IOSurface strings found")
        return None

    all_offsets.sort()
    min_off = all_offsets[0]
    max_off = all_offsets[-1]

    print(f"  IOSurface string range: 0x{min_off:x} - 0x{max_off:x}")
    print(f"  Span: {max_off - min_off} bytes ({(max_off - min_off) / 1024:.1f} KB)")

    # Find clusters within the IOSurface region
    clusters = []
    current = [all_offsets[0]]
    for off in all_offsets[1:]:
        if off - current[-1] < 4096:
            current.append(off)
        else:
            if len(current) >= 2:
                clusters.append(current)
            current = [off]
    if len(current) >= 2:
        clusters.append(current)

    print(f"\n  IOSurface string clusters: {len(clusters)}")
    for i, cluster in enumerate(clusters):
        start = cluster[0]
        end = cluster[-1]
        # Get all strings in this cluster
        nearby = extract_strings_at(data, (start + end) // 2, radius=(end - start) // 2 + 256)
        iosurface_strs = [s for s in nearby if any(k in s.lower() for k in
                         ["iosurface", "surface", "s_create", "s_set", "s_get", "s_lock",
                          "csbuffer", "bytesper", "pixel", "allocation", "shared"])]
        
        print(f"\n    Cluster {i+1}: 0x{start:x} - 0x{end:x} ({end - start} bytes, {len(cluster)} refs)")
        for s in iosurface_strs[:20]:
            print(f"      \"{s}\"")

    return {
        "range": (min_off, max_off),
        "clusters": clusters,
    }


# ============================================================
# PHASE 3: ExternalMethod Dispatch Analysis
# ============================================================
def analyze_external_methods(data, string_map):
    """Deep analysis of IOSurfaceRootUserClient::externalMethod dispatch."""
    print("\n" + "=" * 70)
    print("PHASE 3: EXTERNALMETHOD DISPATCH ANALYSIS")
    print("=" * 70)

    # Find the key entry: "serClientDefaultLockingSingleThreadExternalMethod"
    # This string is nearby the actual dispatch table reference
    key_strings = [
        b"serClientDefaultLockingSingleThreadExternalMethod",
        b"IOSurfaceUserClientDefaultLocking",
        b"IOSurfaceDefaultLockingSingleThread",
        b"s_create_surface",
    ]

    findings = {}
    for pat in key_strings:
        idx = data.find(pat)
        if idx != -1:
            # Get rich context around this string
            nearby = extract_strings_at(data, idx, radius=2048, min_len=4)
            
            # Look for selector-like strings (s_xxx pattern)
            selectors = [s for s in nearby if s.startswith(("s_", "Security:")) 
                        or "ExternalMethod" in s or "IOReturn" in s]
            
            print(f"\n  [{pat.decode('utf-8',errors='replace')[:50]}] at 0x{idx:x}")
            print(f"    Nearby selector-like strings:")
            for s in selectors[:15]:
                print(f"      \"{s}\"")
            
            findings[pat.decode('utf-8', errors='replace')] = {
                "offset": idx,
                "nearby_selectors": selectors,
                "all_nearby": nearby,
            }

    # Now scan for the dispatch table structure
    # IOSurfaceRootUserClient uses a static array of IOExternalMethodDispatch
    # or uses getTargetAndMethodForIndex pattern
    
    print(f"\n  --- Scanning for IOSurface dispatch patterns ---")
    
    # Pattern: series of function name strings that match known selectors
    selector_strings = [
        b"s_create_surface",
        b"s_set_value",
        b"s_get_value",
        b"s_remove_value", 
        b"s_increment_use_count",
        b"s_decrement_use_count",
        b"s_lookup",
    ]
    
    found_selectors = {}
    for sel in selector_strings:
        offsets = []
        start = 0
        while True:
            idx = data.find(sel, start)
            if idx == -1:
                break
            offsets.append(idx)
            start = idx + 1
            if len(offsets) >= 20:
                break
        if offsets:
            found_selectors[sel.decode()] = offsets
            print(f"    {sel.decode():40s} at {[hex(o) for o in offsets[:5]]}")

    # Look for the getTargetAndMethodForIndex pattern
    # This function returns the dispatch entry for a given selector index
    target_method = data.find(b"getTargetAndMethodForIndex")
    if target_method != -1:
        print(f"\n    getTargetAndMethodForIndex at 0x{target_method:x}")
        nearby = extract_strings_at(data, target_method, radius=1024)
        for s in nearby[:10]:
            print(f"      \"{s}\"")

    return findings, found_selectors


# ============================================================
# PHASE 4: Vulnerability Pattern Scan
# ============================================================
def scan_vuln_patterns(data, string_map, region):
    """Scan for known vulnerability patterns in IOSurface code."""
    print("\n" + "=" * 70)
    print("PHASE 4: VULNERABILITY PATTERN SCANNING")
    print("=" * 70)

    vuln_findings = {}
    
    for vuln_type, info in VULN_PATTERNS.items():
        print(f"\n  === {vuln_type.upper()}: {info['description']} ===")
        type_hits = []
        
        for pat in info["strings"]:
            # Search within IOSurface region if available
            search_data = data
            offsets = []
            start = 0
            while True:
                idx = search_data.find(pat, start)
                if idx == -1:
                    break
                # Check if this is near IOSurface code
                nearby = extract_strings_at(data, idx, radius=1024)
                is_iosurface = any("IOSurface" in s or "surface" in s.lower() 
                                   for s in nearby)
                
                offsets.append({
                    "offset": idx,
                    "near_iosurface": is_iosurface,
                    "context": [s for s in nearby if len(s) > 8][:5],
                })
                start = idx + 1
                if len(offsets) >= 20:
                    break
            
            iosurface_hits = [o for o in offsets if o["near_iosurface"]]
            total = len(offsets)
            ios_count = len(iosurface_hits)
            
            if total > 0:
                marker = " <-- IOSURFACE" if ios_count > 0 else ""
                print(f"    {pat.decode('utf-8',errors='replace'):35s} "
                      f"total={total:3d}  near_IOSurface={ios_count}{marker}")
                
                # Show IOSurface-related contexts
                for hit in iosurface_hits[:3]:
                    ctx = [c for c in hit["context"] if "IOSurface" in c or "surface" in c.lower()]
                    if ctx:
                        print(f"      0x{hit['offset']:x}: {ctx[0][:80]}")
                
                type_hits.append({
                    "pattern": pat.decode('utf-8', errors='replace'),
                    "total_hits": total,
                    "iosurface_hits": ios_count,
                    "details": iosurface_hits[:5],
                })
        
        vuln_findings[vuln_type] = {
            "description": info["description"],
            "relevance": info["relevance"],
            "patterns": type_hits,
            "total_iosurface_hits": sum(h["iosurface_hits"] for h in type_hits),
        }
        
        total_ios = vuln_findings[vuln_type]["total_iosurface_hits"]
        if total_ios > 0:
            print(f"    >> TOTAL IOSurface-related hits: {total_ios}")

    return vuln_findings


# ============================================================
# PHASE 5: Disassemble IOSurface Functions
# ============================================================
def disassemble_iosurface_functions(data, string_map):
    """Disassemble key IOSurface functions with Capstone."""
    if not HAS_CAPSTONE:
        print("\n[SKIP] Capstone not available")
        return {}

    print("\n" + "=" * 70)
    print("PHASE 5: ARM64 DISASSEMBLY OF IOSURFACE FUNCTIONS")
    print("=" * 70)

    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True

    results = {}
    
    # Key functions to find and disassemble
    targets = [
        ("s_create_surface", b"s_create_surface"),
        ("s_set_value", b"s_set_value"),
        ("s_get_value", b"s_get_value"),
        ("s_lookup", b"s_lookup"),
        ("s_decrement_use_count", b"s_decrement_use_count"),
        ("IOSurfaceSharedListEntry", b"IOSurfaceSharedListEntry"),
        ("CSBufferPitch", b"CSBufferPitch"),
        ("DefaultLockingExternalMethod", b"DefaultLockingSingleThreadExternalMethod"),
    ]

    for func_name, pattern in targets:
        idx = data.find(pattern)
        if idx == -1:
            continue

        print(f"\n  --- {func_name} (string at 0x{idx:x}) ---")

        # Search backward for function prologue (STP X29, X30 or PACIBSP)
        prologue = find_prologue(data, idx, search_range=8192)
        if prologue is not None:
            # Disassemble
            code = data[prologue:prologue + 512]
            insns = []
            branches = []
            calls = []
            
            for insn in md.disasm(code, prologue):
                entry = {
                    "addr": insn.address,
                    "mnemonic": insn.mnemonic,
                    "op_str": insn.op_str,
                    "size": insn.size,
                }
                insns.append(entry)
                
                # Track branches and calls
                if insn.mnemonic in ("bl", "b"):
                    calls.append(entry)
                elif insn.mnemonic.startswith("b."):
                    branches.append(entry)
                
                if insn.mnemonic == "ret" or len(insns) >= 100:
                    break

            print(f"    Prologue at 0x{prologue:x}, {len(insns)} instructions")
            print(f"    Calls (BL): {len(calls)}, Branches: {len(branches)}")
            
            # Show first 30 instructions
            for insn in insns[:30]:
                print(f"      0x{insn['addr']:08x}: {insn['mnemonic']:8s} {insn['op_str']}")
            if len(insns) > 30:
                print(f"      ... ({len(insns) - 30} more)")

            results[func_name] = {
                "string_offset": f"0x{idx:x}",
                "prologue_offset": f"0x{prologue:x}",
                "instruction_count": len(insns),
                "calls": len(calls),
                "branches": len(branches),
                "instructions": insns[:100],
            }
        else:
            # Try looking for function refs AFTER the string
            # (strings in __cstring, code in __TEXT_EXEC)
            print(f"    No prologue found near string ref")
            
            # Show context for manual analysis
            nearby = extract_strings_at(data, idx, radius=512, min_len=4)
            print(f"    Context strings:")
            for s in nearby[:10]:
                print(f"      \"{s}\"")

    return results


def find_prologue(data, near_offset, search_range=8192):
    """Find ARM64 function prologue near an offset."""
    start = max(0, near_offset - search_range)
    start &= ~3  # align

    for off in range(near_offset - 4, start, -4):
        if off + 4 > len(data):
            continue
        insn = struct.unpack_from("<I", data, off)[0]

        # PACIBSP: 0xD503237F
        if insn == 0xD503237F:
            return off

        # STP X29, X30, [SP, #imm]! 
        if (insn & 0xFFE00000) == 0xA9800000:
            rn = (insn >> 5) & 0x1F
            rt1 = insn & 0x1F
            rt2 = (insn >> 10) & 0x1F
            if rn == 31 and rt1 == 29 and rt2 == 30:
                return off

    return None


# ============================================================
# PHASE 6: Attack Surface Synthesis
# ============================================================
def synthesize_attack_surface(string_map, region, dispatch, vulns, disasm):
    """Synthesize all IOSurface findings into a concrete attack guide."""
    print("\n" + "=" * 70)
    print("PHASE 6: CHAIN B ATTACK SURFACE SYNTHESIS")
    print("=" * 70)

    guide = []
    guide.append("=" * 70)
    guide.append("CHAIN B: IOSurface ATTACK GUIDE")
    guide.append("Target: IOSurfaceRootUserClient on A13/T8030, iOS 26.3")
    guide.append("=" * 70)

    # Entry points
    guide.append("\n1. ENTRY POINTS (sandbox-reachable from any app)")
    guide.append("-" * 50)
    
    if "IOSurfaceRootUserClient" in string_map:
        off = string_map["IOSurfaceRootUserClient"][0]
        guide.append(f"  IOSurfaceRootUserClient string at: 0x{off:x}")
    
    entry_points = [
        ("IOSurface.framework", "Link with -framework IOSurface"),
        ("IOSurfaceCreate(dict)", "Create surface with property dictionary"),
        ("IOSurfaceLookup(id)", "Lookup existing surface by global ID"),
        ("IOSurfaceGetPropertyMaximum", "Query property limits"),
        ("IOSurfaceSetValue/GetValue", "Property R/W - key attack primitives"),
        ("IOKit IOServiceOpen", "Direct IOKit to IOSurfaceRootUserClient"),
        ("IOConnectCallMethod(sel)", "Call externalMethod with selector"),
    ]
    
    for name, desc in entry_points:
        guide.append(f"  {name:40s} - {desc}")

    # Known selectors
    guide.append("\n2. KNOWN EXTERNALMETHOD SELECTORS")
    guide.append("-" * 50)
    for sel_id, (name, desc) in sorted(KNOWN_IOSURFACE_SELECTORS.items()):
        priority = ""
        if "create" in name:
            priority = " [!!!] PRIMARY TARGET"
        elif "set" in name or "get" in name:
            priority = " [!!] R/W PRIMITIVE"
        elif "decrement" in name:
            priority = " [!!] UAF TARGET"
        elif "lookup" in name:
            priority = " [!] CROSS-PROCESS"
        guide.append(f"  Selector {sel_id:3d}: {name:40s}{priority}")
        guide.append(f"             {desc}")

    # Vulnerability patterns found
    guide.append("\n3. VULNERABILITY PATTERNS FOUND IN KERNELCACHE")
    guide.append("-" * 50)
    
    for vuln_type, info in vulns.items():
        ios_hits = info["total_iosurface_hits"]
        if ios_hits > 0:
            guide.append(f"\n  [{vuln_type.upper()}] {info['description']}")
            guide.append(f"  IOSurface-related hits: {ios_hits}")
            guide.append(f"  Relevance: {info['relevance']}")
            for pat in info["patterns"]:
                if pat["iosurface_hits"] > 0:
                    guide.append(f"    - {pat['pattern']}: {pat['iosurface_hits']} IOSurface refs")

    # Concrete attack strategies
    guide.append("\n4. CONCRETE ATTACK STRATEGIES")
    guide.append("-" * 50)
    
    strategies = [
        {
            "name": "Strategy 1: Property Size Integer Overflow",
            "difficulty": "MEDIUM",
            "steps": [
                "1. Create IOSurface with IOSurfaceCreate()",
                "2. Set properties with carefully crafted sizes:",
                "   - Large bytesPerRow * height can overflow uint32",
                "   - allocationSize can wrap around to small value",
                "   - Kernel allocates small buffer, copies large data -> heap overflow",
                "3. Heap overflow into adjacent kernel objects",
                "4. Control adjacent object vtable/function pointers",
                "Key offsets: CSBufferPitch, bytesPerRow, bytesPerElement",
            ],
        },
        {
            "name": "Strategy 2: Shared Memory Race Condition",
            "difficulty": "MEDIUM-HIGH",
            "steps": [
                "1. Thread A: create IOSurface with shared memory",
                "2. Thread B: rapidly modify/destroy the shared memory",
                "3. Race window between IOSurfaceSharedListEntry allocation",
                "   and its use in IOBufferMemoryDescriptor::inTaskWithOptions",
                "4. Win race -> stale pointer to freed shared memory",
                "5. Reallocate freed region with controlled data",
                "6. Use-after-free with controlled content",
                "Key strings: IOSurfaceSharedListEntry, 'Unable to prepare'",
            ],
        },
        {
            "name": "Strategy 3: Reference Count Manipulation",
            "difficulty": "MEDIUM",
            "steps": [
                "1. Create IOSurface, get client handle",
                "2. Call s_increment_use_count repeatedly to overflow refcount",
                "   OR race s_decrement_use_count with surface destruction",
                "3. Refcount reaches 0 while references still exist -> UAF",
                "4. Spray IOSurface properties to reuse freed memory",
                "5. Controlled data in freed object -> arbitrary R/W via properties",
                "Key: os_refcnt 'attempted resurrection' check may need bypass",
            ],
        },
        {
            "name": "Strategy 4: Global Lookup Type Confusion",
            "difficulty": "LOW-MEDIUM",
            "steps": [
                "1. Process A creates IOSurface S1 with specific layout",
                "2. Process B looks up S1 via global ID (IOSurfaceLookup)",
                "   -> 'global (insecure) IOSurface lookups' warning confirms this works",
                "3. Process A destroys S1, reallocates with different type",
                "4. Process B still has handle to S1 -> type confusion",
                "5. Process B reads/writes assuming original type -> OOB access",
                "ADVANTAGE: Works from sandboxed app, no special entitlements",
            ],
        },
    ]

    for strat in strategies:
        guide.append(f"\n  {strat['name']} [{strat['difficulty']}]")
        for step in strat["steps"]:
            guide.append(f"    {step}")

    # Post-exploitation: kernel R/W via IOSurface
    guide.append("\n5. POST-EXPLOITATION: KERNEL R/W VIA IOSURFACE")
    guide.append("-" * 50)
    guide.append("""
  Once you have an initial corruption (UAF/OOB/overflow):
  
  A. IOSurface Property Spray R/W (PREFERRED):
     1. Spray IOSurface objects with s_set_value properties
     2. Each property is an OSData/OSNumber in kernel heap
     3. Overlap corrupted object with IOSurface property buffer
     4. Use s_get_value to READ arbitrary kernel addresses
     5. Use s_set_value to WRITE arbitrary kernel addresses
     
  B. Pipe Buffer R/W (BACKUP):
     1. Create many pipes: pipe(fds)
     2. Write controlled data to pipe buffers
     3. Pipe buffers allocated from same zone as IOSurface objects
     4. Overlap -> R/W primitive via read()/write() syscalls
     
  Verification:
     - Read kern.version to confirm R/W works
     - Read current_proc()->p_ucred to verify address accuracy
  
  Privilege Escalation (data-only, no PAC bypass needed):
     - Overwrite proc->p_ucred->cr_uid = 0 (root)  
     - Overwrite proc->p_ucred->cr_svuid = 0
     - Modify sandbox label to remove restrictions
     - OR: steal kernel_task's credentials pointer""")

    # BootROM final step
    guide.append("\n6. BOOTROM DUMP (FINAL STAGE)")
    guide.append("-" * 50)
    guide.append("""
  With kernel R/W primitive active:
  
  Method A - ml_io_map (RECOMMENDED):
     uint64_t kva = call_kernel_func(ml_io_map, 0x100000000, 0x80000);
     for (off = 0; off < 0x80000; off += 8) {
         bootrom[off] = kernel_read64(kva + off);
     }
     // Save bootrom_t8030.bin (512 KB)
  
  Method B - Direct page reads:
     for (page = 0; page < 32; page++) {
         phys = 0x100000000 + page * 0x4000;
         kernel_call(ml_phys_read_data, phys, kbuf, 0x4000);
         kernel_read_buf(kbuf, user_buf, 0x4000);
     }
  
  Method C - DART remap (if ml_phys_read blocks phys addr):
     1. Find GPU DART at 0x231004000
     2. Write PTEs mapping 0x100000000 into GPU IOVA
     3. Create IOGPUResource at that IOVA
     4. Map to userspace -> direct read
  
  Physical addresses (A13 T8030):
     BootROM base: 0x100000000
     BootROM size: 0x80000 (512 KB = 32 pages of 16KB)
     GPU DART:     0x231004000""")

    # Ghidra analysis tasks
    guide.append("\n7. GHIDRA REVERSE ENGINEERING TASKS")
    guide.append("-" * 50)
    
    re_tasks = []
    if "IOSurfaceRootUserClient" in string_map:
        off = string_map["IOSurfaceRootUserClient"][0]
        re_tasks.append(f"[ ] Go to 0x{off:x} -> find IOSurfaceRootUserClient vtable")
    if "s_create_surface" in string_map:
        off = string_map["s_create_surface"][0]
        re_tasks.append(f"[ ] Go to 0x{off:x} -> find s_create_surface xrefs")
    
    re_tasks.extend([
        "[ ] Find IOSurfaceRootUserClient::externalMethod dispatch table",
        "[ ] Reverse each selector handler (0-19+)",
        "[ ] Map input validation for s_create_surface (size params)",
        "[ ] Find IOSurfaceSharedListEntry alloc/free paths",
        "[ ] Trace s_set_value -> OSData allocation path",
        "[ ] Trace s_get_value -> OSData read path",
        "[ ] Find os_refcnt checks in use_count handlers",
        "[ ] Map the global IOSurface lookup namespace code",
        "[ ] Identify heap zones used for IOSurface objects",
        "[ ] Find IOSurface destructor for UAF exploitation",
    ])
    
    for task in re_tasks:
        guide.append(f"  {task}")

    # Print and save
    report_text = "\n".join(guide)
    print(f"\n{report_text}")

    report_path = EXTRACTED / "CHAIN_B_IOSURFACE_ATTACK_GUIDE.txt"
    report_path.write_text(report_text, encoding='utf-8')
    print(f"\n[*] Attack guide saved to: {report_path}")

    return guide


# ============================================================
# MAIN
# ============================================================
def main():
    print("=" * 70)
    print("IOSURFACE DEEP ANALYZER - Chain B Preparation")
    print("iPhone 11 Pro (A13 / T8030), iOS 26.3")
    print("=" * 70)

    data, kc_path = load_kernelcache()

    # Phase 1: String mapping
    string_map = map_iosurface_strings(data)

    # Phase 2: Code region identification
    region = identify_iosurface_region(data, string_map)

    # Phase 3: ExternalMethod dispatch analysis
    dispatch, found_selectors = analyze_external_methods(data, string_map)

    # Phase 4: Vulnerability pattern scan
    vulns = scan_vuln_patterns(data, string_map, region)

    # Phase 5: Disassembly
    disasm = disassemble_iosurface_functions(data, string_map)

    # Phase 6: Attack surface synthesis
    guide = synthesize_attack_surface(string_map, region, dispatch, vulns, disasm)

    # Save JSON results
    output = {
        "target": "iPhone 11 Pro (A13/T8030) iOS 26.3",
        "string_map": {k: [hex(o) for o in v[:10]] for k, v in string_map.items()},
        "region": {
            "start": hex(region["range"][0]) if region else "N/A",
            "end": hex(region["range"][1]) if region else "N/A",
            "clusters": len(region["clusters"]) if region else 0,
        },
        "found_selectors": {k: [hex(o) for o in v[:5]] for k, v in found_selectors.items()},
        "vulnerability_patterns": {
            k: {
                "description": v["description"],
                "iosurface_hits": v["total_iosurface_hits"],
            } for k, v in vulns.items()
        },
        "disassembled_functions": len(disasm),
        "known_selectors": {str(k): v[0] for k, v in KNOWN_IOSURFACE_SELECTORS.items()},
    }

    out_path = EXTRACTED / "iosurface_analysis.json"
    out_path.write_text(json.dumps(output, indent=2), encoding='utf-8')
    print(f"\n[*] JSON results: {out_path}")
    print("[*] ANALYSIS COMPLETE")


if __name__ == "__main__":
    main()
