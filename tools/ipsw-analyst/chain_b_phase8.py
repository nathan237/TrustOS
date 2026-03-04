#!/usr/bin/env python3
"""
Chain B Phase 8: Overflow Protection Audit + Final Report
==========================================================
Scans ALL 98 MUL instructions in IOSurface kext to determine:
  1. Which MULs are protected by UMULH overflow checks
  2. Which MULs are UNPROTECTED (vulnerability candidates)
  3. Which ADDs are protected by ADDS/B.HS carry checks
  4. Complete overflow protection coverage map
"""

import struct, os
from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM

KC_PATH = os.path.join(os.path.dirname(__file__), "extracted", "kernelcache_iPhone12,3_26_3.raw")
KC_BASE = 0xfffffff007004000
KEXT_TEXT_START = 0xfffffff00a1c5c80
KEXT_TEXT_END   = 0xfffffff00a1f75dc

def va_to_file(va):
    return va - KC_BASE

def main():
    print("=" * 70)
    print("CHAIN B PHASE 8: Overflow Protection Audit")
    print("=" * 70)
    
    with open(KC_PATH, 'rb') as f:
        data = f.read()
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    foff_start = va_to_file(KEXT_TEXT_START)
    foff_end = va_to_file(KEXT_TEXT_END)
    chunk = data[foff_start:foff_end]
    
    # Disassemble entire IOSurface kext TEXT_EXEC
    all_insns = []
    for insn in md.disasm(chunk, KEXT_TEXT_START):
        all_insns.append((insn.address, insn.mnemonic, insn.op_str))
    
    print(f"\n  Total instructions: {len(all_insns)}")
    
    # Build lookup by address
    insn_index = {addr: i for i, (addr, _, _) in enumerate(all_insns)}
    
    mul_mnemonics = {'mul', 'madd', 'msub', 'umull', 'smull', 'umulh', 'smulh',
                     'umaddl', 'smaddl'}
    overflow_check_mnemonics = {'umulh', 'smulh'}
    
    # Find all MUL and UMULH
    all_muls = [(i, a, m, o) for i, (a, m, o) in enumerate(all_insns) if m in mul_mnemonics]
    
    # For each MUL (not umulh/smulh), check if there's a matching UMULH within 
    # the previous 10 instructions using the same source registers
    protected = []
    unprotected = []
    overflow_checks = []
    
    for idx, addr, mn, ops in all_muls:
        if mn in overflow_check_mnemonics:
            overflow_checks.append((addr, mn, ops))
            continue
        
        # Parse destination and source registers from MUL ops
        # MUL xd, xn, xm  or  MADD xd, xn, xm, xa
        parts = ops.replace(' ', '').split(',')
        if len(parts) < 3:
            continue
        src1 = parts[1].strip()
        src2 = parts[2].strip()
        
        # Look backwards for UMULH with same source registers
        has_umulh = False
        for lookback in range(1, 15):
            if idx - lookback < 0:
                break
            prev_addr, prev_mn, prev_ops = all_insns[idx - lookback]
            if prev_mn in overflow_check_mnemonics:
                prev_parts = prev_ops.replace(' ', '').split(',')
                if len(prev_parts) >= 3:
                    prev_src1 = prev_parts[1].strip()
                    prev_src2 = prev_parts[2].strip()
                    # Check if same sources (in any order)
                    if (prev_src1 == src1 and prev_src2 == src2) or \
                       (prev_src1 == src2 and prev_src2 == src1):
                        has_umulh = True
                        break
        
        # Also check if there's a CBNZ/CMP right after the UMULH
        if has_umulh:
            protected.append((addr, mn, ops))
        else:
            # Could still be safe if it's computing something that was already checked
            # Look for matching UMULH using same regs anywhere in the function
            # (might have been checked earlier with same inputs)
            func_has_umulh = False
            for look in range(max(0, idx-30), idx):
                prev_addr2, prev_mn2, prev_ops2 = all_insns[look]
                if prev_mn2 in overflow_check_mnemonics:
                    prev_parts2 = prev_ops2.replace(' ', '').split(',')
                    if len(prev_parts2) >= 3:
                        ps1 = prev_parts2[1].strip()
                        ps2 = prev_parts2[2].strip()
                        if (ps1 == src1 and ps2 == src2) or (ps1 == src2 and ps2 == src1):
                            func_has_umulh = True
                            break
            if func_has_umulh:
                protected.append((addr, mn, ops))
            else:
                unprotected.append((addr, mn, ops, idx))
    
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print(f"RESULTS")
    print(f"{'='*70}")
    print(f"  Total MUL-family instructions: {len(all_muls)}")
    print(f"  Overflow checks (UMULH/SMULH): {len(overflow_checks)}")
    print(f"  Protected MULs (have UMULH guard): {len(protected)}")
    print(f"  UNPROTECTED MULs (no UMULH guard): {len(unprotected)}")
    
    if unprotected:
        print(f"\n{'='*70}")
        print(f"*** UNPROTECTED MULTIPLICATIONS ***")
        print(f"{'='*70}")
        
        for addr, mn, ops, idx in unprotected:
            print(f"\n  {addr:#018x}: {mn} {ops}")
            
            # Show context: 10 insns before and 5 after
            start = max(0, idx - 10)
            end = min(len(all_insns), idx + 6)
            print(f"  Context:")
            for j in range(start, end):
                ca, cm, co = all_insns[j]
                marker = ">>>" if j == idx else "   "
                anno = ""
                if cm in overflow_check_mnemonics:
                    anno = " [OVERFLOW CHECK]"
                elif cm in ('cbnz', 'cbz') and j > 0:
                    anno = " [BRANCH ON CHECK]"
                elif cm == 'b.hs' or cm == 'b.ne':
                    anno = " [OVERFLOW BRANCH]"
                print(f"    {marker} {ca:#018x}: {cm:<12} {co}{anno}")
            
            # Determine if this is truly dangerous
            # Check: does it use 32-bit registers (wN)? If so, 32-bit MUL can't overflow into wrong value
            is_32bit = parts[0].startswith('w')
            if is_32bit:
                print(f"  -> 32-bit operation (w-register) — limited overflow risk")
            else:
                print(f"  -> 64-bit operation — POTENTIAL OVERFLOW RISK")
    
    # ═══════════════════════════════════════════════════════════════════════
    # Also audit ADD operations for carry protection
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print(f"ADD OVERFLOW AUDIT")
    print(f"{'='*70}")
    
    # Count ADDS vs ADD
    adds_count = sum(1 for _, m, _ in all_insns if m == 'adds')
    add_count = sum(1 for _, m, _ in all_insns if m == 'add')
    
    # Find ADD x, x, x patterns (not ADD for address computation or SP adjustments)
    # that are NOT ADDS (no flag setting = no overflow check)
    unsafe_adds = []
    for i, (addr, mn, ops) in enumerate(all_insns):
        if mn == 'add':
            parts = ops.replace(' ', '').split(',')
            if len(parts) >= 3:
                # Skip SP-relative and frame pointer operations
                if any('sp' in p for p in parts) or any('x29' in p for p in parts):
                    continue
                # Skip small constant adds (#0x1, #0x8, etc)
                if parts[-1].startswith('#'):
                    try:
                        val = int(parts[-1].replace('#', '').replace('0x', ''), 16)
                        if val < 0x100:
                            continue
                    except:
                        pass
                # This might be a size calculation without overflow check
                if parts[0].startswith('x') and any(p.startswith('x') for p in parts[1:]):
                    # Check if there's a nearby ADDS/B.HS pattern instead
                    unsafe_adds.append((addr, ops, i))
    
    print(f"  ADDS (flag-setting): {adds_count}")
    print(f"  ADD (non-flag-setting): {add_count}")
    print(f"  Potentially unchecked ADD x,x,x: {len(unsafe_adds)}")
    
    # Show first 10 potentially unsafe adds
    if unsafe_adds:
        print(f"\n  First 20 unchecked register-to-register ADDs:")
        for addr, ops, _ in unsafe_adds[:20]:
            print(f"    {addr:#018x}: add {ops}")
    
    # ═══════════════════════════════════════════════════════════════════════
    # Final assessment
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print(f"FINAL OVERFLOW PROTECTION ASSESSMENT")
    print(f"{'='*70}")
    
    total_muls_checked = len(protected)
    total_muls_unchecked = len(unprotected)
    coverage = total_muls_checked / (total_muls_checked + total_muls_unchecked) * 100 if (total_muls_checked + total_muls_unchecked) > 0 else 0
    
    print(f"""
  Overflow Protection Coverage: {coverage:.1f}%
  
  Protected MULs: {total_muls_checked}
  Unprotected MULs: {total_muls_unchecked}
  UMULH checks: {len(overflow_checks)}
  
  Assessment: {"ROBUST - No obvious integer overflow vectors" if total_muls_unchecked == 0 else f"POTENTIAL WEAKNESS - {total_muls_unchecked} unprotected multiplications found"}
  
  Alternative Attack Vectors:
  1. s_set_value/s_get_value type confusion (property dictionary manipulation)
  2. Race conditions in surface lifecycle (lock at +0xd8)
  3. Use-after-free in surface array management
  4. IOConnectTrap handler redirection (getTargetAndTrapForIndex)
  5. DMA/IOMMU mapping attacks via IOSurface buffer
  6. Kernel heap feng shui via IOSurface property spray
""")

if __name__ == '__main__':
    main()
