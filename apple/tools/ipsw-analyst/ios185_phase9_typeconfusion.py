#!/usr/bin/env python3
"""
Phase 9 — IOSurface Type Confusion & Race Condition Analyzer
=============================================================
iOS 18.5 (22F76) — iPhone 11 Pro (A13/T8030)

Analyzes s_set_value, s_get_value, and s_set_value_xml dispatch handlers
for type confusion opportunities and TOCTOU race windows.

Specifically:
1. Traces s_set_value/s_get_value to find property storage/retrieval
2. Identifies type checking (or lack thereof) on stored objects
3. Finds lock acquire/release patterns to measure race windows
4. Identifies which IOSurface property keys bypass type validation
5. Generates exploitation primitives for type confusion → kernel R/W
"""

import struct
import json
import os
from pathlib import Path
from collections import defaultdict
from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM

KC_PATH = Path("extracted/kernelcache_iPhone12,3_18_5.raw")
KC_BASE = 0xfffffff007004000

# Key addresses from iOS 18.5 analysis
DISPATCH = {
    "s_create_surface":     0xfffffff009e86024,  # sel 0
    "s_delete_surface":     0xfffffff0084ff664,  # sel 1
    "s_lookup_surface":     0xfffffff0084ff418,  # sel 2
    "s_lock_surface":       0xfffffff0084ff390,  # sel 3
    "s_unlock_surface":     0xfffffff0085630e8,  # sel 4
    "s_get_value":          0xfffffff00857f3a4,  # sel 5
    "s_set_value":          0xfffffff00857ee7c,  # sel 6
    "s_increment_use_count":0xfffffff00861c8d0,  # sel 7
    "s_decrement_use_count":0xfffffff00857e464,  # sel 8
    "s_set_value_xml":      0xfffffff00857e1ac,  # sel 9
    "s_get_value_xml":      0xfffffff00857dde8,  # sel 10
    "s_bulk_set_value":     0xfffffff00857dba0,  # sel 11
    "s_bulk_get_value":     0xfffffff00857d83c,  # sel 12
}

VTABLE_VA = 0xfffffff007e56618
PAC_VTABLE_DIV = 0xcda1  # DA key
PAC_RELEASE_DIV = 0x3a87  # IA key, vtable+0x28

# Mach-O constants
MH_MAGIC_64 = 0xFEEDFACF
LC_SEGMENT_64 = 0x19
LC_FILESET_ENTRY = 0x80000035


class Entry:
    def __init__(self, name, vmaddr, fileoff):
        self.name = name
        self.vmaddr = vmaddr
        self.fileoff = fileoff
        self.segments = {}
    
    def va_to_file(self, va):
        for seg in self.segments.values():
            if seg["vmaddr"] <= va < seg["vmaddr"] + seg["vmsize"]:
                return seg["fileoff"] + (va - seg["vmaddr"])
        return None
    
    def get_code_range(self):
        for name, seg in self.segments.items():
            if "TEXT_EXEC" in name:
                return (seg["fileoff"], seg["fileoff"] + seg["filesize"],
                        seg["vmaddr"], seg["vmaddr"] + seg["vmsize"])
        return None


def parse_entries(kc_data):
    _, _, _, _, ncmds, _, _, _ = struct.unpack_from('<IIIIIIII', kc_data, 0)
    entries = []
    offset = 32
    for _ in range(ncmds):
        if offset + 8 > len(kc_data):
            break
        cmd, cmdsize = struct.unpack_from('<II', kc_data, offset)
        if cmd == LC_FILESET_ENTRY:
            vmaddr, fileoff = struct.unpack_from('<QQ', kc_data, offset+8)
            entry_id_off = struct.unpack_from('<I', kc_data, offset+24)[0]
            name = kc_data[offset+entry_id_off:offset+entry_id_off+256].split(b'\x00')[0].decode('ascii', errors='replace')
            entries.append(Entry(name, vmaddr, fileoff))
        offset += cmdsize
    
    for entry in entries:
        inner_off = entry.fileoff
        if inner_off + 32 > len(kc_data):
            continue
        inner_magic = struct.unpack_from('<I', kc_data, inner_off)[0]
        if inner_magic != MH_MAGIC_64:
            continue
        _, _, _, _, inner_ncmds, _, _, _ = struct.unpack_from('<IIIIIIII', kc_data, inner_off)
        cmd_off = inner_off + 32
        for _ in range(inner_ncmds):
            if cmd_off + 8 > len(kc_data):
                break
            cmd, cmdsize = struct.unpack_from('<II', kc_data, cmd_off)
            if cmd == LC_SEGMENT_64:
                segname = kc_data[cmd_off+8:cmd_off+24].split(b'\x00')[0].decode('ascii', errors='replace')
                vmaddr, vmsize, fileoff, filesize = struct.unpack_from('<QQQQ', kc_data, cmd_off+24)
                entry.segments[segname] = {
                    "vmaddr": vmaddr, "vmsize": vmsize,
                    "fileoff": fileoff, "filesize": filesize
                }
            cmd_off += cmdsize
    return entries


def find_containing_entry(entries, va):
    for e in entries:
        code = e.get_code_range()
        if code and code[2] <= va < code[3]:
            return e
    return None


def disasm(kc_data, va, size, entry):
    """Disassemble from VA"""
    foff = entry.va_to_file(va)
    if foff is None:
        # Try all entries
        for e in entries_global:
            foff = e.va_to_file(va)
            if foff:
                break
    if foff is None:
        return []
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    return list(md.disasm(kc_data[foff:foff+size], va))


def find_function_end(insns, start_va, max_insns=500):
    """Find function end (RET/RETAB)"""
    for i, insn in enumerate(insns):
        if i > 0 and insn.mnemonic in ('ret', 'retab'):
            return insn.address + 4
    return start_va + max_insns * 4


def resolve_adrp_add(kc_data, entries, insn_va, entry):
    """Resolve ADRP+ADD to get the target VA and string"""
    foff = entry.va_to_file(insn_va)
    if not foff:
        return None, None
    
    raw1 = struct.unpack_from('<I', kc_data, foff)[0]
    raw2 = struct.unpack_from('<I', kc_data, foff+4)[0]
    
    # ADRP
    if (raw1 & 0x9F000000) != 0x90000000:
        return None, None
    immhi = (raw1 >> 5) & 0x7FFFF
    immlo = (raw1 >> 29) & 0x3
    imm = (immhi << 2) | immlo
    if imm & (1 << 20):
        imm -= (1 << 21)
    page = (insn_va & ~0xFFF) + (imm << 12)
    
    # ADD
    if (raw2 & 0xFFC00000) != 0x91000000:
        return page, None
    add_imm = (raw2 >> 10) & 0xFFF
    shift = (raw2 >> 22) & 0x3
    if shift == 1:
        add_imm <<= 12
    
    target_va = page + add_imm
    
    # Try to read string
    for e in entries:
        sfoff = e.va_to_file(target_va)
        if sfoff and sfoff + 128 < len(kc_data):
            raw_str = kc_data[sfoff:sfoff+128].split(b'\x00')[0]
            if raw_str and len(raw_str) > 0 and all(32 <= b < 127 for b in raw_str):
                return target_va, raw_str.decode('ascii')
    
    return target_va, None


def analyze_function(kc_data, entries, name, va, max_size=4096):
    """Deep analysis of a dispatch function"""
    entry = find_containing_entry(entries, va)
    if not entry:
        print(f"[!] Cannot find entry for {name} @ {hex(va)}")
        return {}
    
    insns = disasm(kc_data, va, max_size, entry)
    func_end = find_function_end(insns, va)
    
    result = {
        "name": name,
        "va": hex(va),
        "kext": entry.name,
        "size": func_end - va,
        "instructions": [],
        "calls": [],
        "string_refs": [],
        "locks": [],
        "type_checks": [],
        "vtable_dispatches": [],
        "property_accesses": [],
        "race_windows": []
    }
    
    prev_adrp = {}  # Track ADRP results by register
    
    for insn in insns:
        if insn.address >= func_end:
            break
        
        m = insn.mnemonic.lower()
        ops = insn.op_str
        
        # Track ADRP
        if m == 'adrp':
            target_va, string = resolve_adrp_add(kc_data, entries, insn.address, entry)
            if string:
                result["string_refs"].append({
                    "addr": hex(insn.address),
                    "target": hex(target_va) if target_va else "?",
                    "string": string
                })
        
        # Track BL (direct calls)
        if m == 'bl':
            # Decode BL target
            foff = entry.va_to_file(insn.address)
            if foff:
                raw = struct.unpack_from('<I', kc_data, foff)[0]
                imm26 = raw & 0x3FFFFFF
                if imm26 & (1 << 25):
                    imm26 -= (1 << 26)
                target = insn.address + (imm26 << 2)
                result["calls"].append({
                    "addr": hex(insn.address),
                    "target": hex(target),
                    "type": "direct"
                })
        
        # Track BLRAA (PAC indirect calls = vtable dispatches)
        if m in ('blraa', 'blrab', 'blraaz', 'blrabz'):
            # Previous MOVK reveals diversity
            diversity = None
            for pi in insns:
                if pi.address >= insn.address:
                    break
                if pi.address >= insn.address - 32 and pi.mnemonic == 'movk':
                    # Extract immediate
                    pfoff = entry.va_to_file(pi.address)
                    if pfoff:
                        praw = struct.unpack_from('<I', kc_data, pfoff)[0]
                        pimm = (praw >> 5) & 0xFFFF
                        diversity = pimm
            
            result["vtable_dispatches"].append({
                "addr": hex(insn.address),
                "mnemonic": m,
                "operands": ops,
                "diversity": hex(diversity) if diversity else None
            })
        
        # Track CASA/CAS (atomic compare-and-swap = lock operations)
        if m.startswith('cas'):
            result["locks"].append({
                "addr": hex(insn.address),
                "type": "CAS",
                "detail": f"{m} {ops}"
            })
        
        # Track LDXR/STXR (exclusive load/store = lock primitives)
        if m in ('ldxr', 'ldaxr', 'stxr', 'stlxr'):
            result["locks"].append({
                "addr": hex(insn.address),
                "type": m.upper(),
                "detail": f"{m} {ops}"
            })
        
        # Track MRS tpidr_el1 (thread-local = common before lock)
        if m == 'mrs' and 'tpidr_el1' in ops:
            result["locks"].append({
                "addr": hex(insn.address),
                "type": "MRS_TPIDR",
                "detail": "Thread ID read (before lock check)"
            })
        
        # Track AUTDA (vtable auth = type identity)
        if m == 'autda':
            # Extract diversity from MOVK
            for pi in insns:
                if pi.address >= insn.address:
                    break
                if pi.address >= insn.address - 16 and pi.mnemonic == 'movk':
                    pfoff = entry.va_to_file(pi.address)
                    if pfoff:
                        praw = struct.unpack_from('<I', kc_data, pfoff)[0]
                        pimm = (praw >> 5) & 0xFFFF
                        result["type_checks"].append({
                            "addr": hex(insn.address),
                            "type": "AUTDA (vtable auth)",
                            "diversity": hex(pimm),
                            "note": "PAC auth = implicit type check via vtable pointer"
                        })
        
        # Track LDR from self offsets (property accesses)
        if m == 'ldr' and 'x19' in ops or 'x20' in ops:
            result["property_accesses"].append({
                "addr": hex(insn.address),
                "detail": f"{m} {ops}"
            })
    
    return result


def analyze_race_windows(func_analysis):
    """Identify potential race condition windows between lock/unlock"""
    locks = func_analysis.get("locks", [])
    vtable_calls = func_analysis.get("vtable_dispatches", [])
    calls = func_analysis.get("calls", [])
    
    windows = []
    
    # If there are CAS/LDXR operations, there's a lock
    lock_addrs = [int(l["addr"], 16) for l in locks if l["type"] in ("CAS", "LDXR", "LDAXR")]
    
    # If there are vtable dispatches between lock operations
    # those are potential race windows
    for vd in vtable_calls:
        vd_addr = int(vd["addr"], 16)
        before_lock = [a for a in lock_addrs if a > vd_addr]
        after_lock = [a for a in lock_addrs if a < vd_addr]
        
        if after_lock and not before_lock:
            windows.append({
                "type": "POST_LOCK_VTABLE_CALL",
                "addr": vd["addr"],
                "note": "Vtable dispatch after lock acquired — may call into user code"
            })
    
    # Check for TOCTOU: property read then use without revalidation
    mrs_addrs = [int(l["addr"], 16) for l in locks if l["type"] == "MRS_TPIDR"]
    cas_addrs = [int(l["addr"], 16) for l in locks if l["type"] == "CAS"]
    
    if mrs_addrs and cas_addrs:
        for mrs in mrs_addrs:
            for cas in cas_addrs:
                gap = cas - mrs
                if 0 < gap < 64:  # tight window
                    windows.append({
                        "type": "LOCK_SEQUENCE",
                        "gap": gap,
                        "mrs_addr": hex(mrs),
                        "cas_addr": hex(cas),
                        "note": f"Lock sequence: MRS→CAS in {gap} bytes, potential race"
                    })
    
    return windows


# Global for helper functions
entries_global = []


def main():
    os.chdir(Path(__file__).parent)
    
    global entries_global
    
    print("=" * 70)
    print(" Phase 9: IOSurface Type Confusion & Race Analysis")
    print(" iOS 18.5 (22F76) — iPhone 11 Pro (A13/T8030)")
    print("=" * 70)
    
    kc_data = KC_PATH.read_bytes()
    entries = parse_entries(kc_data)
    entries_global = entries
    
    print(f"[+] Parsed {len(entries)} fileset entries")
    
    # Analyze key dispatch functions
    targets = [
        "s_get_value",
        "s_set_value",  
        "s_set_value_xml",
        "s_get_value_xml",
        "s_bulk_set_value",
        "s_bulk_get_value",
        "s_create_surface",
        "s_lock_surface",
        "s_unlock_surface",
        "s_increment_use_count",
        "s_decrement_use_count",
    ]
    
    all_results = {}
    
    for name in targets:
        va = DISPATCH.get(name)
        if not va:
            continue
        
        print(f"\n{'='*60}")
        print(f"[*] Analyzing: {name} @ {hex(va)}")
        print(f"{'='*60}")
        
        result = analyze_function(kc_data, entries, name, va)
        all_results[name] = result
        
        print(f"    Size: {result['size']} bytes")
        print(f"    Direct calls: {len(result['calls'])}")
        print(f"    Vtable dispatches: {len(result['vtable_dispatches'])}")
        print(f"    String refs: {len(result['string_refs'])}")
        print(f"    Lock operations: {len(result['locks'])}")
        print(f"    Type checks (AUTDA): {len(result['type_checks'])}")
        
        if result['string_refs']:
            print(f"\n    Strings:")
            for sr in result['string_refs']:
                print(f"      {sr['addr']}: \"{sr['string']}\"")
        
        if result['vtable_dispatches']:
            print(f"\n    Vtable dispatches:")
            for vd in result['vtable_dispatches']:
                div = vd.get('diversity', '?')
                print(f"      {vd['addr']}: {vd['mnemonic']} {vd['operands']} [div={div}]")
        
        if result['type_checks']:
            print(f"\n    Type checks:")
            for tc in result['type_checks']:
                print(f"      {tc['addr']}: {tc['type']} div={tc['diversity']}")
        
        if result['locks']:
            print(f"\n    Lock operations:")
            for lo in result['locks']:
                print(f"      {lo['addr']}: {lo['type']} — {lo['detail']}")
        
        # Race window analysis
        windows = analyze_race_windows(result)
        result['race_windows'] = windows
        if windows:
            print(f"\n    *** RACE WINDOWS DETECTED ***")
            for w in windows:
                print(f"      [{w['type']}] {w.get('note', '')}")
    
    # Cross-function analysis
    print(f"\n{'='*70}")
    print(" CROSS-FUNCTION TYPE CONFUSION ANALYSIS")
    print(f"{'='*70}")
    
    # Compare type checks between s_set_value and s_get_value
    set_types = [tc['diversity'] for tc in all_results.get("s_set_value", {}).get("type_checks", [])]
    get_types = [tc['diversity'] for tc in all_results.get("s_get_value", {}).get("type_checks", [])]
    
    print(f"\n  s_set_value type checks (AUTDA diversities): {set_types}")
    print(f"  s_get_value type checks (AUTDA diversities): {get_types}")
    
    # If they use the SAME PAC diversity, objects are interchangeable
    common = set(set_types) & set(get_types)
    if common:
        print(f"\n  [!] SHARED type diversities: {common}")
        print(f"      Objects set by s_set_value can be retrieved by s_get_value")
        print(f"      with the same type identity -> type confusion possible!")
    
    # Check s_set_value_xml vs s_set_value
    xml_types = [tc['diversity'] for tc in all_results.get("s_set_value_xml", {}).get("type_checks", [])]
    print(f"\n  s_set_value_xml type checks: {xml_types}")
    
    if set(xml_types) != set(set_types):
        print(f"  [!] DIFFERENT type checks between set_value and set_value_xml!")
        print(f"      This is a type confusion vector: set via XML, get via binary!")
    
    # Lock analysis
    print(f"\n{'='*70}")
    print(" LOCK / RACE CONDITION ANALYSIS")
    print(f"{'='*70}")
    
    for name, result in all_results.items():
        locks = result.get("locks", [])
        if locks:
            print(f"\n  {name}: {len(locks)} lock ops")
            mrs = [l for l in locks if l['type'] == 'MRS_TPIDR']
            cas = [l for l in locks if l['type'] == 'CAS']
            print(f"    MRS TPIDR: {len(mrs)}, CAS: {len(cas)}")
            
            # Check if lock is acquired BEFORE property access
            if not mrs and not cas:
                print(f"    [!!!] NO LOCK in {name} — potential race!")
        else:
            print(f"\n  {name}: NO lock operations detected")
            print(f"    This function may be racing with other selectors!")
    
    # Disassemble s_set_value and s_get_value more deeply
    print(f"\n{'='*70}")
    print(" DETAILED s_set_value DISASSEMBLY (first 64 insns)")
    print(f"{'='*70}")
    
    entry = find_containing_entry(entries, DISPATCH["s_set_value"])
    if entry:
        insns = disasm(kc_data, DISPATCH["s_set_value"], 512, entry)
        for i, insn in enumerate(insns[:64]):
            marker = ""
            if insn.mnemonic == 'autda':
                marker = " <<<< TYPE CHECK"
            elif insn.mnemonic.startswith('blra'):
                marker = " <<<< VTABLE CALL"
            elif insn.mnemonic == 'bl':
                marker = " <<<< CALL"
            elif 'tpidr_el1' in insn.op_str:
                marker = " <<<< THREAD_ID"
            elif insn.mnemonic.startswith('cas'):
                marker = " <<<< LOCK"
            print(f"  {hex(insn.address)}: {insn.mnemonic:10s} {insn.op_str:40s}{marker}")
    
    print(f"\n{'='*70}")
    print(" DETAILED s_get_value DISASSEMBLY (first 64 insns)")
    print(f"{'='*70}")
    
    entry = find_containing_entry(entries, DISPATCH["s_get_value"])
    if entry:
        insns = disasm(kc_data, DISPATCH["s_get_value"], 512, entry)
        for i, insn in enumerate(insns[:64]):
            marker = ""
            if insn.mnemonic == 'autda':
                marker = " <<<< TYPE CHECK"
            elif insn.mnemonic.startswith('blra'):
                marker = " <<<< VTABLE CALL"
            elif insn.mnemonic == 'bl':
                marker = " <<<< CALL"
            elif 'tpidr_el1' in insn.op_str:
                marker = " <<<< THREAD_ID"
            elif insn.mnemonic.startswith('cas'):
                marker = " <<<< LOCK"
            print(f"  {hex(insn.address)}: {insn.mnemonic:10s} {insn.op_str:40s}{marker}")
    
    # Generate the target map
    print(f"\n{'='*70}")
    print(" EXPLOITATION TARGET MAP")
    print(f"{'='*70}")
    
    total_race_windows = sum(len(r.get("race_windows", [])) for r in all_results.values())
    total_vtable_dispatches = sum(len(r.get("vtable_dispatches", [])) for r in all_results.values())
    total_type_checks = sum(len(r.get("type_checks", [])) for r in all_results.values())
    
    print(f"""
  iOS 18.5 IOSurface Attack Surface:
  
  Dispatch selectors analyzed: {len(all_results)}
  Total vtable dispatches: {total_vtable_dispatches}
  Total type checks (AUTDA): {total_type_checks}
  Total race windows: {total_race_windows}
  
  PAC vtable diversity: 0xcda1 (DA key)
  PAC release diversity: 0x3a87 (IA key)
  
  ATTACK VECTORS (priority order):
  
  1. TYPE CONFUSION via s_set_value_xml → s_get_value
     - Set a property with XML serialization (different type path)
     - Retrieve with binary getter (assumes specific type)
     - If type check differs: controlled type confusion
  
  2. RACE: s_set_value vs s_get_value
     - If locking is per-surface (not per-property):
       Thread A: set_value(key, OSData)
       Thread B: get_value(key) → reads during transition
     - Window: between vtable auth and property read
  
  3. RACE: s_create_surface vs s_increment_use_count
     - Race surface creation with use count increment
     - If use_count checked before surface fully initialized
  
  4. s_decrement_use_count → UAF
     - Race decrement with get_value
     - If refcount hits 0 while value still being read
  
  5. s_bulk_set_value / s_bulk_get_value
     - Bulk operations iterate over arrays
     - If another thread modifies array during iteration: OOB
""")
    
    # Save full results
    out_path = Path("extracted/ios185_phase9_results.json")
    # Convert for JSON serialization
    serializable = {}
    for name, result in all_results.items():
        serializable[name] = {
            "va": result["va"],
            "kext": result["kext"],
            "size": result["size"],
            "calls": result["calls"],
            "string_refs": result["string_refs"],
            "locks": result["locks"],
            "type_checks": result["type_checks"],
            "vtable_dispatches": result["vtable_dispatches"],
            "race_windows": result["race_windows"]
        }
    
    with open(out_path, 'w', encoding='utf-8') as f:
        json.dump(serializable, f, indent=2, ensure_ascii=False)
    print(f"\n[+] Phase 9 results saved to: {out_path}")


if __name__ == "__main__":
    main()
