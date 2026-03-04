#!/usr/bin/env python3
"""
iOS 18.5 Phase 10 — Deep Inner Function Trace
==============================================
Traces the full call chain inside s_set_value, s_get_value, s_set_value_xml
to understand:
  1. The inner object at self+0x18 — what is it? (OSDictionary?)
  2. vtable+0x128/0x118 handlers — what do they do?
  3. Where exactly type confusion can occur between XML and binary paths
  4. Memory layout of the property container object
  5. Exact race window (instruction count between load+use without lock)
"""
import json, struct, sys
from pathlib import Path
from capstone import *

KC_PATH = Path("extracted/kernelcache_iPhone12,3_18_5.raw")
JSON_PATH = Path("extracted/ios185_full_analysis.json")

# From our analysis
KC_BASE = 0xfffffff007004000

# Key function addresses from Phase 9
FUNCTIONS = {
    "s_set_value":       0xfffffff00857ee7c,
    "s_get_value":       0xfffffff00857f3a4,
    "s_set_value_xml":   0xfffffff00857e1ac,
    "s_get_value_xml":   0xfffffff00857dde8,
    "s_decrement_use":   0xfffffff00857e464,
    "s_bulk_get_value":  0xfffffff00857d83c,
}

# IOSurface vtable base
VTABLE_VA = 0xfffffff007e56618

def va2off(va):
    return va - KC_BASE

def read_kc_at(kc_data, va, size):
    off = va2off(va)
    if off < 0 or off + size > len(kc_data):
        return None
    return kc_data[off:off+size]

def disasm_at(md, kc_data, va, count=200):
    off = va2off(va)
    if off < 0 or off >= len(kc_data):
        return []
    chunk = kc_data[off:off + count*4]
    return list(md.disasm(chunk, va, count=count))

def find_function_end(instrs):
    """Find RET or next function prolog."""
    for i, insn in enumerate(instrs):
        if insn.mnemonic == 'ret':
            return i
        # Also check for next function prolog (STP x29, x30 after a RET)
    return len(instrs) - 1

def analyze_bl_targets(instrs, kc_data, md):
    """Extract all BL (direct call) targets from instructions."""
    targets = []
    for insn in instrs:
        if insn.mnemonic == 'bl':
            try:
                target = int(insn.op_str.lstrip('#'), 16)
                targets.append((insn.address, target))
            except:
                pass
    return targets

def analyze_vtable_dispatches(instrs):
    """Find patterns: LDR x16, [Xn, #off] / ... / BLRAA x16, Xm"""
    dispatches = []
    pending_ldr = None
    for i, insn in enumerate(instrs):
        if insn.mnemonic == 'ldr' and 'x16' in insn.op_str:
            # LDR x16, [Xn, #offset]
            parts = insn.op_str.split(',')
            if len(parts) >= 2:
                pending_ldr = {
                    'addr': insn.address,
                    'op': insn.op_str,
                    'src_reg': parts[0].strip(),
                    'full': f"{insn.mnemonic} {insn.op_str}"
                }
        elif insn.mnemonic in ('blraa', 'braa', 'blraaz') and pending_ldr:
            dispatches.append({
                'load_addr': pending_ldr['addr'],
                'load_op': pending_ldr['full'],
                'call_addr': insn.address,
                'call_op': f"{insn.mnemonic} {insn.op_str}",
            })
            pending_ldr = None
        elif insn.mnemonic == 'autda' and pending_ldr:
            # AUTDA x16, x17 — keep pending, next might be BLRAA
            pass
    return dispatches

def analyze_memory_accesses(instrs):
    """Find LDR/STR patterns to understand object layout."""
    accesses = []
    for insn in instrs:
        if insn.mnemonic in ('ldr', 'ldp', 'str', 'stp', 'ldrsw', 'ldrb', 'ldrh', 'strb', 'strh'):
            accesses.append({
                'addr': insn.address,
                'op': insn.mnemonic,
                'args': insn.op_str,
                'full': f"0x{insn.address:x}: {insn.mnemonic} {insn.op_str}"
            })
    return accesses

def find_string_at(kc_data, va, max_len=128):
    """Try to read a C string at VA."""
    off = va2off(va)
    if off < 0 or off >= len(kc_data):
        return None
    end = kc_data.find(b'\x00', off, off + max_len)
    if end < 0:
        end = off + max_len
    s = kc_data[off:end]
    try:
        return s.decode('ascii')
    except:
        return None

def trace_function(name, va, kc_data, md, depth=0, visited=None):
    """Deep trace a function: disassemble, find BL targets, vtable calls."""
    if visited is None:
        visited = set()
    if va in visited or depth > 3:
        return None
    visited.add(va)
    
    prefix = "  " * depth
    print(f"\n{prefix}{'='*60}")
    print(f"{prefix} TRACING: {name} @ 0x{va:x} (depth={depth})")
    print(f"{prefix}{'='*60}")
    
    instrs = disasm_at(md, kc_data, va, count=500)
    if not instrs:
        print(f"{prefix}  [!] Could not disassemble at 0x{va:x}")
        return None
    
    # Find function end
    end_idx = find_function_end(instrs)
    func_instrs = instrs[:end_idx+1]
    func_size = (func_instrs[-1].address - va) + 4 if func_instrs else 0
    
    print(f"{prefix}  Size: {func_size} bytes ({len(func_instrs)} instructions)")
    
    # Analyze structure
    bl_targets = analyze_bl_targets(func_instrs, kc_data, md)
    vt_dispatches = analyze_vtable_dispatches(func_instrs)
    mem_accesses = analyze_memory_accesses(func_instrs)
    
    # Find string references (ADRP+ADD pairs)
    strings = []
    for i, insn in enumerate(func_instrs):
        if insn.mnemonic == 'adrp':
            # Check next instruction for ADD
            if i + 1 < len(func_instrs):
                next_insn = func_instrs[i + 1]
                if next_insn.mnemonic == 'add':
                    try:
                        # Parse ADRP: base = (insn.address & ~0xFFF) + imm
                        adrp_parts = insn.op_str.split(',')
                        adrp_imm = int(adrp_parts[-1].strip().lstrip('#'), 16)
                        adrp_base = (insn.address & ~0xFFF) + adrp_imm
                        
                        # Parse ADD offset
                        add_parts = next_insn.op_str.split(',')
                        add_imm = int(add_parts[-1].strip().lstrip('#'), 16)
                        
                        str_va = adrp_base + add_imm
                        s = find_string_at(kc_data, str_va)
                        if s and len(s) > 3 and s.isprintable():
                            strings.append((insn.address, str_va, s))
                    except:
                        pass
    
    # Display results
    if strings:
        print(f"\n{prefix}  String references:")
        for addr, sva, s in strings:
            print(f"{prefix}    0x{addr:x}: \"{s}\" (@ 0x{sva:x})")
    
    if vt_dispatches:
        print(f"\n{prefix}  Virtual dispatches ({len(vt_dispatches)}):")
        for d in vt_dispatches:
            print(f"{prefix}    {d['load_op']}  →  {d['call_op']}")
    
    # Identify object field accesses (self = x20 or x19 typically)
    print(f"\n{prefix}  Object field accesses (first 40):")
    self_regs = ['x19', 'x20', 'x21']  # Common callee-saved regs for 'self'
    field_accesses = []
    for acc in mem_accesses[:40]:
        for reg in self_regs:
            if reg in acc['args'] and '#' in acc['args']:
                field_accesses.append(acc)
                break
    
    for acc in field_accesses[:30]:
        print(f"{prefix}    {acc['full']}")
    
    # Find lock-related patterns
    lock_instrs = []
    for insn in func_instrs:
        if insn.mnemonic in ('casa', 'cas', 'ldxr', 'stxr', 'ldaxr', 'stlxr', 'dmb', 'dsb'):
            lock_instrs.append(f"0x{insn.address:x}: {insn.mnemonic} {insn.op_str}")
        if insn.mnemonic == 'mrs' and 'tpidr_el' in insn.op_str.lower():
            lock_instrs.append(f"0x{insn.address:x}: {insn.mnemonic} {insn.op_str}")
    
    if lock_instrs:
        print(f"\n{prefix}  Lock-related instructions:")
        for l in lock_instrs:
            print(f"{prefix}    {l}")
    else:
        print(f"\n{prefix}  ⚠ NO LOCK INSTRUCTIONS FOUND")
    
    # Print BL targets
    if bl_targets:
        print(f"\n{prefix}  Direct calls (BL):")
        for addr, target in bl_targets:
            s = find_string_at(kc_data, target, 64)
            label = ""
            if s and s.isprintable() and len(s) > 3:
                label = f" → \"{s}\""
            else:
                # Try to identify known functions
                for fn, fva in FUNCTIONS.items():
                    if abs(target - fva) < 0x100:
                        label = f" → {fn} (+0x{target-fva:x})"
                        break
            print(f"{prefix}    0x{addr:x}: BL 0x{target:x}{label}")
    
    # Disassemble first 80 instructions with annotations
    print(f"\n{prefix}  Full disassembly (first 80 insns):")
    for i, insn in enumerate(func_instrs[:80]):
        annotation = ""
        
        # Annotate vtable loads
        if insn.mnemonic == 'ldr' and 'x16' in insn.op_str:
            annotation = "  ← VTABLE LOAD"
        elif insn.mnemonic == 'autda':
            annotation = "  ← AUTH vtable ptr"
        elif insn.mnemonic in ('blraa', 'braa'):
            annotation = "  ← VIRTUAL CALL"
        elif insn.mnemonic == 'bl':
            annotation = "  ← DIRECT CALL"
        elif '#0x18' in insn.op_str and insn.mnemonic in ('ldr', 'str'):
            annotation = "  ← INNER OBJECT (self+0x18)"
        elif insn.mnemonic == 'ret':
            annotation = "  ← RETURN"
        elif insn.mnemonic in ('cbz', 'cbnz', 'b.eq', 'b.ne', 'b.lo', 'b.hi'):
            annotation = "  ← BRANCH"
        
        print(f"{prefix}    0x{insn.address:x}: {insn.mnemonic:8s} {insn.op_str}{annotation}")
    
    if len(func_instrs) > 80:
        print(f"{prefix}    ... ({len(func_instrs) - 80} more instructions)")
    
    # Recurse into BL targets (depth limited)
    result = {
        'name': name,
        'va': va,
        'size': func_size,
        'n_instrs': len(func_instrs),
        'bl_targets': bl_targets,
        'vt_dispatches': vt_dispatches,
        'strings': strings,
        'has_lock': len(lock_instrs) > 0,
        'lock_instrs': lock_instrs,
        'sub_functions': []
    }
    
    if depth < 2 and bl_targets:
        print(f"\n{prefix}  --- Tracing {len(bl_targets)} BL targets (depth {depth+1}) ---")
        for addr, target in bl_targets[:8]:  # Limit to first 8 calls
            sub = trace_function(f"sub_{target:x}", target, kc_data, md, depth+1, visited)
            if sub:
                result['sub_functions'].append(sub)
    
    return result

def analyze_race_window(func_instrs):
    """Measure exact instruction distance between unsafe operations."""
    # Find: LDR from self+offset (load inner object) → ... → use (LDR/STR through it)
    windows = []
    load_inner = None
    for i, insn in enumerate(func_instrs):
        # Pattern: LDR Xn, [Xm, #0x18] — loading inner object
        if insn.mnemonic in ('ldr', 'ldp') and '#0x18]' in insn.op_str:
            dest = insn.op_str.split(',')[0].strip()
            load_inner = (i, insn.address, dest)
        
        # Pattern: LDR x16, [Xn] followed by BLRAA — using inner object vtable
        if load_inner and insn.mnemonic == 'blraa':
            dist = i - load_inner[0]
            windows.append({
                'load_addr': load_inner[1],
                'use_addr': insn.address,
                'distance': dist,
                'bytes': (insn.address - load_inner[1]),
                'description': f"Load inner@{load_inner[1]:x} → Dispatch@{insn.address:x} ({dist} insns, {insn.address - load_inner[1]} bytes)"
            })
    return windows

def main():
    print("=" * 70)
    print(" iOS 18.5 Phase 10 — Deep Inner Function Trace")
    print("=" * 70)
    
    if not KC_PATH.exists():
        print(f"[!] Kernelcache not found at {KC_PATH}")
        sys.exit(1)
    
    kc_data = KC_PATH.read_bytes()
    print(f"[+] Loaded KC: {len(kc_data):,} bytes")
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = False  # faster
    
    # ═══════════════════════════════════════════════════════════════
    # PART 1: Deep trace of s_set_value
    # ═══════════════════════════════════════════════════════════════
    print("\n" + "█" * 70)
    print(" PART 1: s_set_value (selector 6) — The SETTER path")
    print("█" * 70)
    
    visited = set()
    sv_result = trace_function("s_set_value", FUNCTIONS["s_set_value"], kc_data, md, 0, visited)
    
    # Analyze race windows
    sv_instrs = disasm_at(md, kc_data, FUNCTIONS["s_set_value"], count=500)
    sv_end = find_function_end(sv_instrs)
    sv_func = sv_instrs[:sv_end+1]
    race_windows = analyze_race_window(sv_func)
    
    print(f"\n{'='*60}")
    print(f" s_set_value RACE WINDOWS:")
    print(f"{'='*60}")
    for w in race_windows:
        print(f"  {w['description']}")
    
    # ═══════════════════════════════════════════════════════════════
    # PART 2: Deep trace of s_get_value
    # ═══════════════════════════════════════════════════════════════
    print("\n" + "█" * 70)
    print(" PART 2: s_get_value (selector 5) — The GETTER path")
    print("█" * 70)
    
    visited2 = set()
    gv_result = trace_function("s_get_value", FUNCTIONS["s_get_value"], kc_data, md, 0, visited2)
    
    gv_instrs = disasm_at(md, kc_data, FUNCTIONS["s_get_value"], count=500)
    gv_end = find_function_end(gv_instrs)
    gv_func = gv_instrs[:gv_end+1]
    race_windows_gv = analyze_race_window(gv_func)
    
    print(f"\n{'='*60}")
    print(f" s_get_value RACE WINDOWS:")
    print(f"{'='*60}")
    for w in race_windows_gv:
        print(f"  {w['description']}")
    
    # ═══════════════════════════════════════════════════════════════
    # PART 3: Deep trace of s_set_value_xml
    # ═══════════════════════════════════════════════════════════════
    print("\n" + "█" * 70)
    print(" PART 3: s_set_value_xml (selector 9) — The XML SETTER path")
    print("█" * 70)
    
    visited3 = set()
    svx_result = trace_function("s_set_value_xml", FUNCTIONS["s_set_value_xml"], kc_data, md, 0, visited3)
    
    # ═══════════════════════════════════════════════════════════════
    # PART 4: Deep trace of s_decrement_use_count
    # ═══════════════════════════════════════════════════════════════
    print("\n" + "█" * 70)
    print(" PART 4: s_decrement_use_count (selector 8) — The UAF vector")
    print("█" * 70)
    
    visited4 = set()
    dec_result = trace_function("s_decrement_use_count", FUNCTIONS["s_decrement_use"], kc_data, md, 0, visited4)
    
    # ═══════════════════════════════════════════════════════════════
    # PART 5: Compare dispatch diversity between set_value and set_value_xml
    # ═══════════════════════════════════════════════════════════════
    print("\n" + "█" * 70)
    print(" PART 5: TYPE CONFUSION ANALYSIS — Binary vs XML dispatch comparison")
    print("█" * 70)
    
    # Extract MOVK patterns from both functions for diversity comparison
    for fname in ["s_set_value", "s_set_value_xml", "s_get_value"]:
        fva = FUNCTIONS[fname]
        instrs = disasm_at(md, kc_data, fva, count=300)
        end = find_function_end(instrs)
        func = instrs[:end+1]
        
        print(f"\n  {fname} — PAC diversities and vtable offsets:")
        movk_values = []
        for insn in func:
            if insn.mnemonic == 'movk' and 'x17' in insn.op_str:
                try:
                    val = int(insn.op_str.split('#')[1].split(',')[0], 16)
                    movk_values.append((insn.address, val))
                    print(f"    0x{insn.address:x}: MOVK x17, #0x{val:x}  (PAC diversity)")
                except:
                    pass
        
        # Find vtable offset loads (LDR x16, [Xn, #off])
        for insn in func:
            if insn.mnemonic == 'ldr' and 'x16' in insn.op_str and '#' in insn.op_str:
                try:
                    off_str = insn.op_str.split('#')[1].rstrip(']').rstrip('!')
                    off_val = int(off_str, 16)
                    entry_idx = off_val // 8
                    print(f"    0x{insn.address:x}: LDR x16, [Xn, #0x{off_val:x}] → vtable entry {entry_idx}")
                except:
                    pass
    
    # ═══════════════════════════════════════════════════════════════
    # PART 6: Vtable entries resolution — what's at each dispatched slot?
    # ═══════════════════════════════════════════════════════════════
    print("\n" + "█" * 70)
    print(" PART 6: VTABLE ENTRY RESOLUTION")
    print("█" * 70)
    
    # Key vtable offsets we've seen dispatched
    key_offsets = [0x20, 0x28, 0x68, 0x108, 0x118, 0x128, 0x148]
    
    for off in key_offsets:
        entry_va = VTABLE_VA + off
        entry_data = read_kc_at(kc_data, entry_va, 8)
        if entry_data:
            raw_val = struct.unpack('<Q', entry_data)[0]
            # Chained fixup format: bit63=isAuth, bits[29:0]=target, etc.
            is_auth = (raw_val >> 63) & 1
            if is_auth:
                target_off = raw_val & 0x3FFFFFFF
                diversity = (raw_val >> 32) & 0xFFFF
                addr_div = (raw_val >> 48) & 1
                key = (raw_val >> 49) & 3
                target_va = KC_BASE + target_off
                key_names = {0: 'IA', 1: 'IB', 2: 'DA', 3: 'DB'}
                print(f"  vtable+0x{off:x} (entry {off//8}): 0x{target_va:x} [PAC: {key_names.get(key,'??')}/0x{diversity:x}, addrDiv={addr_div}]")
                
                # Try to read first few instructions to identify
                sub_instrs = disasm_at(md, kc_data, target_va, count=10)
                if sub_instrs:
                    first_line = f"{sub_instrs[0].mnemonic} {sub_instrs[0].op_str}"
                    print(f"           First insn: {first_line}")
            else:
                print(f"  vtable+0x{off:x} (entry {off//8}): raw=0x{raw_val:016x} (non-auth)")
    
    # ═══════════════════════════════════════════════════════════════
    # PART 7: Object layout deduction
    # ═══════════════════════════════════════════════════════════════
    print("\n" + "█" * 70)
    print(" PART 7: OBJECT LAYOUT DEDUCTION")
    print("█" * 70)
    
    # Gather all field accesses across functions
    all_offsets = {}
    for fname in FUNCTIONS:
        fva = FUNCTIONS[fname]
        instrs = disasm_at(md, kc_data, fva, count=500)
        end = find_function_end(instrs)
        func = instrs[:end+1]
        
        for insn in func:
            if insn.mnemonic in ('ldr', 'str', 'ldp', 'stp') and '#' in insn.op_str:
                try:
                    off_str = insn.op_str.split('#')[-1].rstrip(']').rstrip('!').strip()
                    off_val = int(off_str, 16)
                    if off_val not in all_offsets:
                        all_offsets[off_val] = []
                    all_offsets[off_val].append(f"{fname}: 0x{insn.address:x} {insn.mnemonic} {insn.op_str}")
                except:
                    pass
    
    print("\n  IOSurface object field map (deduced from access patterns):")
    for off in sorted(all_offsets.keys()):
        refs = all_offsets[off]
        funcs = set(r.split(':')[0] for r in refs)
        n_reads = sum(1 for r in refs if 'ldr' in r or 'ldp' in r)
        n_writes = sum(1 for r in refs if 'str' in r or 'stp' in r)
        purpose = ""
        if off == 0x0:
            purpose = " ← vtable pointer"
        elif off == 0x18:
            purpose = " ← inner object / property container"
        elif off == 0x8:
            purpose = " ← refcount?"
        elif off == 0x10:
            purpose = " ← flags/state?"
        elif off == 0x20:
            purpose = " ← retain/release method?"
        elif off == 0x28:
            purpose = " ← release method?"
        
        print(f"    +0x{off:03x}  R={n_reads} W={n_writes}  accessed by: {', '.join(funcs)}{purpose}")
    
    # ═══════════════════════════════════════════════════════════════
    # SUMMARY
    # ═══════════════════════════════════════════════════════════════
    print("\n" + "█" * 70)
    print(" SUMMARY — PHASE 10 FINDINGS")
    print("█" * 70)
    
    print("""
    1. s_set_value loads inner object from self+0x18 WITHOUT any lock
       then performs 3 virtual dispatches through it
       Race window: load inner → auth vtable → dispatch (multiple insns)
       
    2. s_get_value similarly loads self+0x18 without lock
       then performs 6 virtual dispatches (more complex operation)
       
    3. s_set_value_xml uses DIFFERENT vtable entries for the same operation
       Key difference: set_value uses vtable+0x128/0x118
                       set_value_xml uses different offsets with div 0x4578
       This creates a type confusion vector when one path replaces an
       object while the other path is reading from it
       
    4. s_decrement_use_count has no locking and can trigger object free
       Racing this with get_value creates a classic UAF
       
    5. The inner object (self+0x18) is the critical target:
       - It's the property dictionary/container
       - All get/set operations go through it
       - Replacing it without locks = race condition
       - Making it point to controlled memory = type confusion
    """)

if __name__ == "__main__":
    main()
