#!/usr/bin/env python3
"""
Chain B Phase 7: Deep Size Computation Tracer
==============================================
Traces the actual buffer size multiplication in IOSurface:
  1. IOSurface_allocate sub-call 0xfffffff00a1dd038
  2. create_internal (0xfffffff00a1edafc) full call graph
  3. s_create_surface dimension processing
  4. Find ALL MUL/UMULL/MADD across the call chain
  5. Identify the exact overflow point
"""

import struct, json, os, sys
from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM

KC_PATH = os.path.join(os.path.dirname(__file__), "extracted", "kernelcache_iPhone12,3_26_3.raw")
KC_BASE = 0xfffffff007004000

# IOSurface kext TEXT_EXEC bounds
KEXT_TEXT_START = 0xfffffff00a1c5c80
KEXT_TEXT_END   = 0xfffffff00a1f75dc

# Key functions
ALLOC_SUB      = 0xfffffff00a1dd038  # Called from IOSurface_allocate
CREATE_INTERNAL = 0xfffffff00a1edafc  # create_internal
S_CREATE       = 0xfffffff00a1eba5c  # s_create_surface
IOSURFACE_ALLOC = 0xfffffff00a1cece8  # IOSurface_allocate
MAX_CHECK      = 0xfffffff00a1d02d0  # IOSurface_max_check

def va_to_file(va):
    return va - KC_BASE

def disasm_function(data, va, max_insn=200):
    """Disassemble a function until RET or max_insn"""
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    foff = va_to_file(va)
    if foff < 0 or foff + max_insn*4 > len(data):
        return []
    chunk = data[foff:foff + max_insn*4]
    result = []
    for insn in md.disasm(chunk, va):
        result.append((insn.address, insn.mnemonic, insn.op_str, insn.bytes))
        if len(result) >= max_insn:
            break
        # Stop at RET (but not if it's the first instruction)
        if len(result) > 2 and insn.mnemonic in ('ret', 'retab', 'retaa'):
            break
    return result

def find_mul_ops(insns):
    """Find all multiply-related instructions"""
    mul_mnemonics = {'mul', 'madd', 'msub', 'umull', 'smull', 'umulh', 'smulh', 
                     'umaddl', 'smaddl', 'umsubl', 'smsubl', 'mneg', 'umnegl', 'smnegl'}
    return [(a, m, o) for a, m, o, _ in insns if m in mul_mnemonics]

def find_bl_targets(insns, kext_only=False):
    """Find all BL targets in the instruction list"""
    targets = []
    for addr, mn, ops, _ in insns:
        if mn == 'bl':
            try:
                tgt = int(ops.replace('#', ''), 16)
                if kext_only and not (KEXT_TEXT_START <= tgt <= KEXT_TEXT_END):
                    continue
                targets.append((addr, tgt))
            except:
                pass
    return targets

def find_ldr_struct_offsets(insns):
    """Find LDR from struct offsets (looking for dimension reads)"""
    patterns = []
    for addr, mn, ops, _ in insns:
        if mn in ('ldr', 'ldur', 'ldrb', 'ldrh', 'ldrsw') and '#0x' in ops:
            patterns.append((addr, mn, ops))
    return patterns

def trace_deep(data, va, label, depth=0, visited=None, max_depth=4):
    """Recursively trace a function and all its IOSurface-internal callees"""
    if visited is None:
        visited = set()
    if va in visited or depth > max_depth:
        return []
    visited.add(va)
    
    insns = disasm_function(data, va, 250)
    if not insns:
        return []
    
    results = []
    mul_ops = find_mul_ops(insns)
    if mul_ops:
        results.append({
            'va': va,
            'label': label,
            'depth': depth,
            'mul_ops': mul_ops,
            'insn_count': len(insns)
        })
    
    # Recurse into IOSurface-internal BL targets
    bl_targets = find_bl_targets(insns, kext_only=True)
    for call_addr, tgt_va in bl_targets:
        if tgt_va != va and tgt_va not in visited:  # avoid recursion
            sub_label = f"{label} -> {tgt_va:#x}"
            sub_results = trace_deep(data, tgt_va, sub_label, depth+1, visited, max_depth)
            results.extend(sub_results)
    
    return results

def main():
    print("=" * 70)
    print("CHAIN B PHASE 7: Deep Size Computation Tracer")
    print("=" * 70)
    
    with open(KC_PATH, 'rb') as f:
        data = f.read()

    print(f"\n  Kernelcache: {len(data):,} bytes")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 1: Disassemble allocation sub-call 0xfffffff00a1dd038
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print(f"1. ALLOCATION SUB-CALL: {ALLOC_SUB:#018x}")
    print(f"   Called from IOSurface_allocate — computes buffer size?")
    print(f"{'='*70}")
    
    insns = disasm_function(data, ALLOC_SUB, 300)
    mul_ops = find_mul_ops(insns)
    bl_targets = find_bl_targets(insns)
    ldr_offsets = find_ldr_struct_offsets(insns)
    
    print(f"\n  Function length: {len(insns)} instructions")
    print(f"  MUL operations: {len(mul_ops)}")
    print(f"  BL calls: {len(bl_targets)}")
    print(f"  LDR struct access: {len(ldr_offsets)}")
    
    # Print full disassembly
    for addr, mn, ops, _ in insns:
        annotation = ""
        if mn in ('mul', 'madd', 'umull', 'smull', 'umulh'):
            annotation = " <<=== MULTIPLY ==="
        elif mn == 'bl':
            try:
                tgt = int(ops.replace('#', ''), 16)
                in_kext = "IOSurface" if KEXT_TEXT_START <= tgt <= KEXT_TEXT_END else "kernel"
                annotation = f" [{in_kext}]"
            except:
                pass
        elif '#0x58' in ops or '#0x60' in ops or '#0x78' in ops or '#0x80' in ops or '#0x90' in ops or '#0x98' in ops:
            if 'ldr' in mn or 'str' in mn:
                off_map = {
                    '0x58': 'width', '0x60': 'height', '0x78': 'bpe',
                    '0x80': 'elem_w', '0x90': 'bpr', '0x98': 'alloc_size'
                }
                for off, name in off_map.items():
                    if f'#0x{off}' in ops or f'#{off}' in ops:
                        annotation = f" <-- {name}"
                        break
        elif mn == 'lsl':
            annotation = " <-- SHIFT"
        elif mn in ('adds', 'add') and 'lsl' in ops.lower():
            annotation = " <-- ADD+SHIFT"
        print(f"    {addr:#018x}: {mn:<12} {ops}{annotation}")
    
    if mul_ops:
        print(f"\n  === MUL INSTRUCTIONS FOUND ===")
        for a, m, o in mul_ops:
            print(f"    {a:#018x}: {m} {o}")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 2: Deep trace from s_create_surface
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print(f"2. DEEP MUL TRACE: s_create_surface call tree")
    print(f"   Recursive search for MUL/UMULL/MADD in all IOSurface callees")
    print(f"{'='*70}")
    
    results = trace_deep(data, S_CREATE, "s_create_surface", max_depth=5)
    
    if results:
        print(f"\n  Found MUL operations in {len(results)} functions:")
        for r in results:
            print(f"\n  --- {r['va']:#018x} (depth={r['depth']}, {r['insn_count']} insns) ---")
            print(f"      Path: {r['label']}")
            for a, m, o in r['mul_ops']:
                print(f"      {a:#018x}: {m} {o}")
    else:
        print(f"\n  No MUL found in s_create_surface call tree!")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 3: Deep trace from IOSurface_allocate
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print(f"3. DEEP MUL TRACE: IOSurface_allocate call tree")
    print(f"{'='*70}")
    
    results2 = trace_deep(data, IOSURFACE_ALLOC, "IOSurface_allocate", max_depth=5)
    
    if results2:
        print(f"\n  Found MUL operations in {len(results2)} functions:")
        for r in results2:
            print(f"\n  --- {r['va']:#018x} (depth={r['depth']}, {r['insn_count']} insns) ---")
            print(f"      Path: {r['label']}")
            for a, m, o in r['mul_ops']:
                print(f"      {a:#018x}: {m} {o}")
    else:
        print(f"\n  No MUL found in IOSurface_allocate call tree!")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 4: Brute force scan ALL MUL in IOSurface kext
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print(f"4. BRUTE FORCE: All MUL/UMULL/MADD in IOSurface __TEXT_EXEC")
    print(f"   Scanning {KEXT_TEXT_END - KEXT_TEXT_START:#x} bytes")
    print(f"{'='*70}")
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    foff_start = va_to_file(KEXT_TEXT_START)
    foff_end = va_to_file(KEXT_TEXT_END)
    chunk = data[foff_start:foff_end]
    
    mul_mnemonics = {'mul', 'madd', 'msub', 'umull', 'smull', 'umulh', 'smulh',
                     'umaddl', 'smaddl', 'umsubl', 'smsubl'}
    all_muls = []
    
    for insn in md.disasm(chunk, KEXT_TEXT_START):
        if insn.mnemonic in mul_mnemonics:
            all_muls.append((insn.address, insn.mnemonic, insn.op_str))
    
    print(f"\n  Total MUL-family instructions: {len(all_muls)}")
    
    for addr, mn, ops in all_muls:
        # For each MUL, show context (5 insns before and after)
        ctx_start = addr - 20
        ctx_foff = va_to_file(ctx_start)
        ctx_insns = []
        for ci in md.disasm(data[ctx_foff:ctx_foff+44], ctx_start):
            ctx_insns.append((ci.address, ci.mnemonic, ci.op_str))
        
        # Check if this MUL uses 32-bit or 64-bit registers
        is_64bit = ops.startswith('x') or mn in ('umull', 'smull', 'umaddl', 'smaddl')
        overflow_risk = "HIGH" if is_64bit else "LOW (32-bit)"
        
        print(f"\n  {addr:#018x}: {mn} {ops}  [risk: {overflow_risk}]")
        
        # Show surrounding context
        for ca, cm, co in ctx_insns:
            marker = ">>>" if ca == addr else "   "
            dim_anno = ""
            if '#0x58' in co or '#0x60' in co or '#0x78' in co or '#0x80' in co or '#0x90' in co:
                if 'ldr' in cm or 'str' in cm:
                    dim_anno = " <-- dimension?"
            print(f"    {marker} {ca:#018x}: {cm:<12} {co}{dim_anno}")

    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 5: Trace alloc_sub's internal BL targets for MUL
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print(f"5. DEEP TRACE: alloc_sub (0xfffffff00a1dd038) internal callees")
    print(f"{'='*70}")
    
    insns = disasm_function(data, ALLOC_SUB, 300)
    internal_bls = find_bl_targets(insns, kext_only=True)
    
    print(f"\n  IOSurface-internal BL targets from alloc_sub:")
    for call_addr, tgt in internal_bls:
        print(f"    {call_addr:#018x}: BL {tgt:#018x}")
        
        # Disassemble each target and look for MUL
        sub_insns = disasm_function(data, tgt, 200)
        sub_muls = find_mul_ops(sub_insns)
        sub_bls = find_bl_targets(sub_insns, kext_only=True)
        
        if sub_muls:
            print(f"      *** HAS {len(sub_muls)} MUL OPS ***")
            for a, m, o in sub_muls:
                print(f"        {a:#018x}: {m} {o}")
        else:
            print(f"      No MUL ({len(sub_insns)} insns, {len(sub_bls)} sub-calls)")
        
        # One more level deep
        for _, deeper_tgt in sub_bls:
            if KEXT_TEXT_START <= deeper_tgt <= KEXT_TEXT_END:
                deeper_insns = disasm_function(data, deeper_tgt, 200)
                deeper_muls = find_mul_ops(deeper_insns)
                if deeper_muls:
                    print(f"      -> Sub-call {deeper_tgt:#018x} HAS {len(deeper_muls)} MUL OPS:")
                    for a, m, o in deeper_muls:
                        print(f"            {a:#018x}: {m} {o}")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 6: For each MUL found, trace the full context
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print(f"6. MUL CONTEXT ANALYSIS")
    print(f"   For each MUL in IOSurface, show full function context")
    print(f"{'='*70}")
    
    # For relevant MULs, find what function they're in and trace the data flow
    for mul_addr, mul_mn, mul_ops in all_muls:
        # Find function start (look backwards for PACIBSP or SUB SP)
        func_start = None
        for probe in range(mul_addr, mul_addr - 0x1000, -4):
            foff = va_to_file(probe)
            if foff < 0 or foff + 4 > len(data):
                break
            insn_word = struct.unpack_from('<I', data, foff)[0]
            # PACIBSP = 0xd503237f
            if insn_word == 0xd503237f:
                func_start = probe
                break
            # STP x29, x30 pattern often at function entry
            # Check for SUB SP, SP, #imm at function entry
        
        if func_start is None:
            func_start = mul_addr - 40  # fallback: show 10 insns before
        
        # Disassemble the full function
        func_insns = disasm_function(data, func_start, 200)
        ldr_offsets = find_ldr_struct_offsets(func_insns)
        
        # Check if this function accesses IOSurface dimension fields
        dimension_offsets = {'0x58', '0x60', '0x78', '0x80', '0x90', '0x98'}
        dim_refs = [l for l in ldr_offsets if any(f'#0x{d}' in l[2] or f'#{d}' in l[2] for d in dimension_offsets)]
        
        if dim_refs:
            print(f"\n  *** CRITICAL: MUL at {mul_addr:#018x} in function accessing dimensions ***")
            print(f"      Function start: {func_start:#018x}")
            print(f"      MUL: {mul_mn} {mul_ops}")
            print(f"      Dimension accesses:")
            for a, m, o in dim_refs:
                print(f"        {a:#018x}: {m} {o}")
            print(f"      Full function:")
            for a, m, o, _ in func_insns:
                marker = ">>>" if a == mul_addr else "   "
                print(f"        {marker} {a:#018x}: {m:<12} {o}")
        else:
            print(f"\n  MUL at {mul_addr:#018x}: {mul_mn} {mul_ops}")
            print(f"    Function at {func_start:#018x} — no dimension field access")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 7: Summary
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print(f"7. PHASE 7 SUMMARY")
    print(f"{'='*70}")
    print(f"  Total MUL instructions in IOSurface kext: {len(all_muls)}")
    print(f"  Functions with MUL in s_create call tree: {len(results)}")
    print(f"  Functions with MUL in allocate call tree: {len(results2)}")
    
    # Save results
    out = {
        'total_mul_ops': len(all_muls),
        'mul_locations': [{'va': hex(a), 'mn': m, 'ops': o} for a, m, o in all_muls],
        's_create_mul_funcs': len(results),
        'allocate_mul_funcs': len(results2),
    }
    out_path = os.path.join(os.path.dirname(__file__), "extracted", "CHAIN_B_PHASE7_MUL_TRACE.json")
    with open(out_path, 'w', encoding='utf-8') as f:
        json.dump(out, f, indent=2, ensure_ascii=False)
    print(f"  Saved: {out_path}")

if __name__ == '__main__':
    main()
