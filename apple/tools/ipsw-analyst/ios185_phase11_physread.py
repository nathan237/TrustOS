#!/usr/bin/env python3
"""
iOS 18.5 Phase 11 — Locate ml_phys_read/write and gPhysBase/gVirtBase
======================================================================
Scans the kernel portion (not IOSurface kext) for:
  1. ml_phys_read_data / ml_phys_read_double_data  
  2. ml_phys_write_data / ml_phys_write_double_data
  3. gPhysBase / gVirtBase global variables
  4. copyin / copyout (alternative kernel R/W)
  5. IOMemoryDescriptor::createMappingInTask for MMIO approach

Strategy:
  - ml_phys_read pattern: 
      MRS x, DAIF / MSR DAIFSet, ... / LDRB/LDR from phys→virt translated addr
  - gPhysBase: loaded from [Xn] with pattern: phys_va = virt_va - gVirtBase + gPhysBase
  - String scan for "ml_phys_read", "phys_read", "gPhysBase" in __cstring
"""
import struct, sys, re
from pathlib import Path
from capstone import *

KC_PATH = Path("extracted/kernelcache_iPhone12,3_18_5.raw")
KC_BASE = 0xfffffff007004000

def va2off(va):
    return va - KC_BASE

def off2va(off):
    return KC_BASE + off

def main():
    kc = Path(KC_PATH).read_bytes()
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = False
    
    print("=" * 70)
    print(" iOS 18.5 Phase 11 — Kernel Physical Memory Access Primitives")
    print("=" * 70)
    print(f"KC size: {len(kc):,} bytes")
    
    # ═════════════════════════════════════════════
    # PART 1: String scan for known symbols
    # ═════════════════════════════════════════════
    print("\n" + "─" * 70)
    print(" PART 1: String scan for kernel symbols")
    print("─" * 70)
    
    search_strings = [
        b"ml_phys_read",
        b"ml_phys_write",
        b"phys_read",
        b"phys_write",
        b"gPhysBase",
        b"gVirtBase",
        b"physmap",
        b"pmap_find_phys",
        b"copyin",
        b"copyout",
        b"ml_io_map",
        b"kvtophys",
        b"phystokv",
        b"ml_static_mfree",
        b"pmap_map_bd",
    ]
    
    for s in search_strings:
        pos = 0
        found = []
        while True:
            idx = kc.find(s, pos)
            if idx < 0:
                break
            # Check it's a reasonable string (preceded by \0 or another printable)
            if idx > 0 and (kc[idx-1] == 0 or kc[idx-1] == 0x0a):
                found.append(idx)
            elif idx == 0:
                found.append(idx)
            pos = idx + 1
        
        if found:
            for idx in found[:5]:  # Limit output
                va = off2va(idx)
                # Read surrounding context
                ctx_start = max(0, idx - 8)
                ctx_end = min(len(kc), idx + len(s) + 32)
                ctx = kc[ctx_start:ctx_end]
                # Find null terminator
                null_pos = kc.find(b'\x00', idx + len(s), idx + 128)
                full_str = kc[idx:null_pos].decode('ascii', errors='replace') if null_pos > 0 else s.decode()
                print(f"  \"{full_str}\" @ 0x{va:x} (offset 0x{idx:x})")
        else:
            print(f"  \"{s.decode()}\" — NOT FOUND")
    
    # ═════════════════════════════════════════════
    # PART 2: Find MRS DAIF pattern (ml_phys_read signature)
    # ═════════════════════════════════════════════
    print("\n" + "─" * 70)
    print(" PART 2: MRS DAIF pattern scan (ml_phys_read signature)")
    print("─" * 70)
    
    # ml_phys_read typically starts with:
    #   MRS x8, DAIF         (save interrupt state)
    #   MSR DAIFSet, #0xf    (disable interrupts)  
    #   ...load gPhysBase/gVirtBase...
    #   ...translate address...
    #   LDR from translated addr (the actual physical read)
    #   MSR DAIF, x8          (restore interrupt state)
    #   RET
    
    # Encoding: MRS Xn, DAIF = D53B4200 | Xn  (DAIF = S3_3_C4_2_0)
    # MSR DAIFSet, #imm = D503 41{imm}F (varies)
    
    mrs_daif_pattern = bytes([0x00, 0x42, 0x3B, 0xD5])  # MRS X0, DAIF  
    # But Xn varies, so we search for the upper 3 bytes: 42 3B D5
    
    mrs_hits = []
    for i in range(0, len(kc) - 4, 4):
        word = struct.unpack('<I', kc[i:i+4])[0]
        # MRS Xn, DAIF: 1101 0101 0011 1011 0100 0010 000x xxxx
        # = 0xD53B4200 | Xn
        if (word & 0xFFFFFFE0) == 0xD53B4200:
            mrs_hits.append(i)
    
    print(f"  Found {len(mrs_hits)} MRS Xn, DAIF instructions")
    
    # For each hit, check if followed by MSR DAIFSet within a few instructions
    ml_phys_candidates = []
    for hit in mrs_hits:
        va = off2va(hit)
        # Disassemble surrounding code (from function start)
        # Look backwards for function prolog (STP x29,x30 or PACIBSP)
        func_start = hit
        for back in range(0, 128, 4):
            check_off = hit - back
            if check_off < 0:
                break
            word = struct.unpack('<I', kc[check_off:check_off+4])[0]
            # PACIBSP = 0xD503237F
            if word == 0xD503237F:
                func_start = check_off
                break
            # STP x29, x30, [sp, #-XX]! — common prolog
            if (word & 0xFFC003E0) == 0xA9800000 and ((word >> 10) & 0x1F) == 31:
                func_start = check_off
                break
        
        # Disassemble from func_start, look for pattern
        chunk = kc[func_start:func_start + 256]
        instrs = list(md.disasm(chunk, off2va(func_start), count=64))
        
        has_msr_daifset = False
        has_msr_daif_restore = False
        has_ldr_after_msr = False
        has_sub_pattern = False  # SUB for phystokv
        func_size = 0
        
        for j, insn in enumerate(instrs):
            if 'daifset' in insn.op_str.lower():
                has_msr_daifset = True
            if insn.mnemonic == 'msr' and 'daif' in insn.op_str.lower() and 'set' not in insn.op_str.lower():
                has_msr_daif_restore = True
            if has_msr_daifset and insn.mnemonic in ('ldr', 'ldrb', 'ldrh', 'ldrsw', 'ldar'):
                has_ldr_after_msr = True
            if insn.mnemonic == 'sub' and has_msr_daifset:
                has_sub_pattern = True
            if insn.mnemonic in ('ret', 'retab'):
                func_size = (insn.address - off2va(func_start)) + 4
                break
        
        if has_msr_daifset and has_msr_daif_restore:
            confidence = "HIGH" if has_ldr_after_msr and has_sub_pattern else "MEDIUM" if has_ldr_after_msr else "LOW"
            ml_phys_candidates.append({
                'func_va': off2va(func_start),
                'mrs_va': va,
                'size': func_size,
                'has_ldr': has_ldr_after_msr,
                'has_sub': has_sub_pattern,
                'confidence': confidence
            })
    
    print(f"  Candidates with MRS DAIF + MSR DAIFSet + MSR DAIF restore: {len(ml_phys_candidates)}")
    
    # Sort by confidence
    for conf in ['HIGH', 'MEDIUM', 'LOW']:
        cands = [c for c in ml_phys_candidates if c['confidence'] == conf]
        if cands:
            print(f"\n  [{conf}] confidence ({len(cands)}):")
            for c in cands[:10]:
                print(f"    Func: 0x{c['func_va']:x}, MRS: 0x{c['mrs_va']:x}, Size: {c['size']}, LDR: {c['has_ldr']}, SUB: {c['has_sub']}")
    
    # ═════════════════════════════════════════════
    # PART 3: Detailed analysis of top candidates
    # ═════════════════════════════════════════════
    print("\n" + "─" * 70)
    print(" PART 3: Detailed analysis of HIGH confidence candidates")
    print("─" * 70)
    
    high_cands = [c for c in ml_phys_candidates if c['confidence'] == 'HIGH'][:5]
    
    for c in high_cands:
        func_va = c['func_va']
        func_off = va2off(func_va)
        chunk = kc[func_off:func_off + 512]
        instrs = list(md.disasm(chunk, func_va, count=128))
        
        print(f"\n  Function @ 0x{func_va:x}:")
        
        # Find ADRP targets (potential gPhysBase/gVirtBase references)
        adrp_targets = []
        for i, insn in enumerate(instrs):
            if insn.mnemonic == 'adrp' and i + 1 < len(instrs):
                next_insn = instrs[i+1]
                if next_insn.mnemonic == 'add':
                    try:
                        parts = insn.op_str.split(',')
                        base = int(parts[-1].strip().lstrip('#'), 16)
                        page = (insn.address & ~0xFFF) + base
                        
                        add_parts = next_insn.op_str.split(',')
                        add_imm = int(add_parts[-1].strip().lstrip('#'), 16)
                        
                        target = page + add_imm
                        dest_reg = parts[0].strip()
                        adrp_targets.append((insn.address, target, dest_reg))
                    except:
                        pass
                elif next_insn.mnemonic == 'ldr':
                    try:
                        parts = insn.op_str.split(',')
                        base = int(parts[-1].strip().lstrip('#'), 16)
                        page = (insn.address & ~0xFFF) + base
                        
                        # LDR Xn, [Xm, #off] — offset is in next insn
                        ldr_parts = next_insn.op_str.split('#')
                        if len(ldr_parts) > 1:
                            ldr_off = int(ldr_parts[-1].rstrip(']').rstrip('!'), 16)
                            target = page + ldr_off
                            dest_reg = parts[0].strip()
                            adrp_targets.append((insn.address, target, dest_reg))
                    except:
                        pass
        
        if adrp_targets:
            print(f"    Global references:")
            for addr, target, reg in adrp_targets:
                # Try to read the value at target to see if it's a pointer
                target_off = va2off(target)
                if 0 <= target_off < len(kc) - 8:
                    val = struct.unpack('<Q', kc[target_off:target_off+8])[0]
                    print(f"      0x{addr:x}: {reg} → 0x{target:x} (value: 0x{val:016x})")
                else:
                    print(f"      0x{addr:x}: {reg} → 0x{target:x}")
        
        # Print first 30 instructions
        for insn in instrs[:30]:
            if insn.mnemonic in ('ret', 'retab'):
                print(f"    0x{insn.address:x}: {insn.mnemonic:8s} {insn.op_str} ◄─ END")
                break
            annotation = ""
            if 'daif' in insn.op_str.lower():
                annotation = " ◄ DAIF"
            elif insn.mnemonic == 'sub':
                annotation = " ◄ phys→virt?"
            elif insn.mnemonic in ('ldr', 'ldrb', 'ldrh') and 'x0' in insn.op_str.split(',')[0]:
                annotation = " ◄ PHYS READ?"
            elif insn.mnemonic in ('str', 'strb', 'strh') and len(insn.op_str.split(',')) > 1:
                annotation = " ◄ PHYS WRITE?"
            print(f"    0x{insn.address:x}: {insn.mnemonic:8s} {insn.op_str}{annotation}")
    
    # ═════════════════════════════════════════════
    # PART 4: Find gPhysBase/gVirtBase pattern
    # ═════════════════════════════════════════════
    print("\n" + "─" * 70)
    print(" PART 4: gPhysBase/gVirtBase global variable scan")
    print("─" * 70)
    
    # Pattern: Two adjacent globals in kernel __DATA
    # gPhysBase and gVirtBase are typically:
    #   gVirtBase = kernel virtual base (0xfffffff0XXXXXXXX)
    #   gPhysBase = physical base (0x8XXXXXXXX typically on A13)
    # They're usually adjacent in memory and referenced together
    
    # Search for the pattern in HIGH candidates: 
    # LDP x_phys, x_virt, [Xn] or two LDR from adjacent globals
    # The address translation is: phys = virt - gVirtBase + gPhysBase
    # So we look for: SUB + ADD pattern
    
    for c in high_cands[:3]:
        func_va = c['func_va']
        func_off = va2off(func_va)
        chunk = kc[func_off:func_off + 256]
        instrs = list(md.disasm(chunk, func_va, count=64))
        
        # Look for LDP pattern (loading two globals at once)
        for insn in instrs:
            if insn.mnemonic == 'ldp' and 'x' in insn.op_str:
                print(f"    LDP @ 0x{insn.address:x}: {insn.op_str}")
    
    # ═════════════════════════════════════════════
    # PART 5: Find xrefs to "phys" strings for function identification
    # ═════════════════════════════════════════════
    print("\n" + "─" * 70)
    print(" PART 5: Cross-reference phys-related strings")
    print("─" * 70)
    
    # Find interesting string addresses, then search for ADRP pages pointing to them
    phys_strings = {}
    for s in [b"phys", b"kvtophys", b"phystokv", b"ml_phys", b"pmap"]:
        pos = 0
        while True:
            idx = kc.find(s, pos)
            if idx < 0:
                break
            if idx > 0 and kc[idx-1] == 0:
                # Valid string start
                null_end = kc.find(b'\x00', idx, idx + 128)
                if null_end > 0:
                    full = kc[idx:null_end].decode('ascii', errors='replace')
                    if full.isprintable() and len(full) < 80:
                        phys_strings[off2va(idx)] = full
            pos = idx + 1
    
    print(f"  Found {len(phys_strings)} phys-related strings:")
    for va, s in sorted(phys_strings.items())[:30]:
        print(f"    0x{va:x}: \"{s}\"")
    
    # ═════════════════════════════════════════════
    # PART 6: Alternative - copyin/copyout for kernel R/W
    # ═════════════════════════════════════════════
    print("\n" + "─" * 70)
    print(" PART 6: copyin/copyout alternatives")
    print("─" * 70)
    
    # Search for copyin/copyout strings
    for name in [b"copyin\x00", b"copyout\x00", b"copyinstr\x00"]:
        idx = kc.find(name)
        if idx >= 0:
            va = off2va(idx)
            print(f"  \"{name.rstrip(b'\\x00').decode()}\" string @ 0x{va:x}")
    
    # ═════════════════════════════════════════════
    # PART 7: Scan for physical address constant patterns  
    # ═════════════════════════════════════════════
    print("\n" + "─" * 70)
    print(" PART 7: BootROM physical address references")
    print("─" * 70)
    
    # A13 BootROM is at physical 0x100000000
    # Search for MOVZ/MOVK patterns loading this address
    # MOVZ Xn, #1, LSL #32 followed by related instructions
    
    # Encoding: MOVZ Xn, #1, LSL #32 = 0xD2A00020 | Xn (64-bit)
    movz_hits = []
    for i in range(0, len(kc) - 4, 4):
        word = struct.unpack('<I', kc[i:i+4])[0]
        # MOVZ Xn, #imm, LSL #32: 1101 0010 101 imm16 Xn
        # For imm=1: 0xD2A00020
        if (word & 0xFFFFFFE0) == 0xD2A00020:
            movz_hits.append(i)
    
    print(f"  MOVZ Xn, #1, LSL #32 (0x100000000): {len(movz_hits)} hits")
    for hit in movz_hits[:10]:
        va = off2va(hit)
        # Disassemble context
        start = max(0, hit - 16)
        chunk = kc[start:start + 64]
        instrs = list(md.disasm(chunk, off2va(start), count=16))
        ctx = " | ".join(f"{i.mnemonic} {i.op_str}" for i in instrs[:6])
        print(f"    0x{va:x}: {ctx}")
    
    # Also search for 0x100000000 as a 64-bit value in data
    bootrom_addr = struct.pack('<Q', 0x100000000)
    data_hits = []
    pos = 0
    while True:
        idx = kc.find(bootrom_addr, pos)
        if idx < 0:
            break
        data_hits.append(idx)
        pos = idx + 8
    
    print(f"\n  0x100000000 as 64-bit literal in data: {len(data_hits)} hits")
    for hit in data_hits[:5]:
        va = off2va(hit)
        print(f"    0x{va:x}")

if __name__ == "__main__":
    main()
