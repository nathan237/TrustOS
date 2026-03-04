#!/usr/bin/env python3
"""
iOS 18.5 Phase 11b — Find ml_phys_read/write by string xref
============================================================
Uses the ml_phys_read_data and phystokv strings to locate the actual functions.
"""
import struct, sys
from pathlib import Path
from capstone import *

KC_PATH = Path("extracted/kernelcache_iPhone12,3_18_5.raw")
KC_BASE = 0xfffffff007004000

# Found strings
STRINGS = {
    "ml_phys_read_data": 0xfffffff0070586f2,
    "ml_phys_write_data": 0xfffffff007058731,
    "kvtophys_nofail": 0xfffffff0070529a6,
    "phystokv": 0xfffffff007058251,
    "phystokv_range": 0xfffffff007058268,
}

def va2off(va):
    return va - KC_BASE

def off2va(off):
    return KC_BASE + off

def find_adrp_xrefs(kc, target_va):
    """Find all ADRP+ADD pairs that reference a given VA."""
    target_page = target_va & ~0xFFF
    target_off = target_va & 0xFFF
    
    results = []
    for i in range(0, len(kc) - 8, 4):
        word = struct.unpack('<I', kc[i:i+4])[0]
        # ADRP: 1 immlo 10000 immhi Rd
        # = (word >> 24) & 0x9F == 0x90
        if (word >> 24) & 0x9F != 0x90:
            continue
        
        rd = word & 0x1F
        immhi = (word >> 5) & 0x7FFFF
        immlo = (word >> 29) & 0x3
        imm = (immhi << 2) | immlo
        # Sign extend 21-bit
        if imm & (1 << 20):
            imm -= (1 << 21)
        
        insn_va = off2va(i)
        page_base = (insn_va & ~0xFFF) + (imm << 12)
        
        if page_base != target_page:
            continue
        
        # Check next instruction for ADD Xn, Xrd, #target_off
        next_word = struct.unpack('<I', kc[i+4:i+8])[0]
        
        # ADD (immediate): 1 00 100010 sh imm12 Rn Rd
        if (next_word >> 23) & 0x1FF == 0x122:  # 64-bit ADD immediate
            rn = (next_word >> 5) & 0x1F
            add_imm = (next_word >> 10) & 0xFFF
            shift = (next_word >> 22) & 1
            if shift:
                add_imm <<= 12
            
            if rn == rd and add_imm == target_off:
                results.append(off2va(i))
    
    return results

def main():
    kc = Path(KC_PATH).read_bytes()
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = False
    
    print("=" * 70)
    print(" iOS 18.5 Phase 11b — ml_phys_read/write Function Resolution")
    print("=" * 70)
    
    for name, str_va in STRINGS.items():
        print(f"\n{'─'*70}")
        print(f" String: \"{name}\" @ 0x{str_va:x}")
        print(f"{'─'*70}")
        
        xrefs = find_adrp_xrefs(kc, str_va)
        print(f"  ADRP+ADD xrefs: {len(xrefs)}")
        
        for xref in xrefs[:5]:
            print(f"\n  Xref @ 0x{xref:x}:")
            
            # Walk backwards to find function prolog
            func_start = xref
            xref_off = va2off(xref)
            for back in range(4, 1024, 4):
                check_off = xref_off - back
                if check_off < 0:
                    break
                word = struct.unpack('<I', kc[check_off:check_off+4])[0]
                # PACIBSP = 0xD503237F
                if word == 0xD503237F:
                    func_start = off2va(check_off)
                    break
                # STP x29, x30: common prolog without PAC (older patterns)
                # Check for any previous function's RET/RETAB
                if word & 0xFFFFFFFF == 0xD65F0FFF:  # RETAB
                    func_start = off2va(check_off + 4)
                    break
                if word & 0xFFFFFFE0 == 0xD65F03C0:  # RET
                    func_start = off2va(check_off + 4)
                    break
            
            print(f"  Function start: 0x{func_start:x}")
            
            # Disassemble the full function
            func_off = va2off(func_start)
            chunk = kc[func_off:func_off + 1024]
            instrs = list(md.disasm(chunk, func_start, count=256))
            
            # Find function end
            func_end = func_start
            for insn in instrs:
                if insn.mnemonic in ('ret', 'retab'):
                    func_end = insn.address
                    break
            
            func_size = func_end - func_start + 4
            print(f"  Function end: 0x{func_end:x}, Size: {func_size} bytes")
            
            # Print full function
            print(f"  Disassembly:")
            for insn in instrs:
                annotation = ""
                if insn.mnemonic == 'mrs':
                    annotation = " ◄ MRS"
                elif insn.mnemonic == 'msr':
                    annotation = " ◄ MSR"
                elif insn.mnemonic in ('ldr', 'ldrb', 'ldrh', 'ldrsw'):
                    if insn.address > xref:  # After string ref, likely the phys read
                        annotation = " ◄ LOAD"
                elif insn.mnemonic in ('str', 'strb', 'strh'):
                    annotation = " ◄ STORE"
                elif insn.mnemonic == 'bl':
                    annotation = " ◄ CALL"
                elif insn.mnemonic == 'sub':
                    annotation = " ◄ SUB"
                elif insn.mnemonic == 'add' and 'x0' in insn.op_str.split(',')[0]:
                    annotation = " ◄ ADD"
                elif insn.mnemonic == 'ldp':
                    annotation = " ◄ LOAD PAIR"
                
                print(f"    0x{insn.address:x}: {insn.mnemonic:8s} {insn.op_str}{annotation}")
                
                if insn.mnemonic in ('ret', 'retab'):
                    break
            
            # Check for ADRP references to globals (gPhysBase/gVirtBase)
            globals_accessed = []
            for i, insn in enumerate(instrs):
                if insn.address > func_end:
                    break
                if insn.mnemonic == 'adrp' and i + 1 < len(instrs):
                    next_insn = instrs[i+1]
                    try:
                        parts = insn.op_str.split(',')
                        base = int(parts[-1].strip().lstrip('#'), 16)
                        page = (insn.address & ~0xFFF) + base
                        
                        if next_insn.mnemonic == 'add':
                            add_parts = next_insn.op_str.split(',')
                            add_imm = int(add_parts[-1].strip().lstrip('#'), 16)
                            target = page + add_imm
                        elif next_insn.mnemonic == 'ldr':
                            ldr_parts = next_insn.op_str.split('#')
                            if len(ldr_parts) > 1:
                                ldr_off = int(ldr_parts[-1].rstrip(']').rstrip('!'), 16)
                                target = page + ldr_off
                            else:
                                continue
                        else:
                            continue
                        
                        # Check if it's NOT a string pointer
                        if target != str_va:
                            globals_accessed.append((insn.address, target))
                    except:
                        pass
            
            if globals_accessed:
                print(f"\n  Global variables referenced:")
                for addr, gva in globals_accessed:
                    # Read value at global
                    goff = va2off(gva)
                    if 0 <= goff < len(kc) - 8:
                        val = struct.unpack('<Q', kc[goff:goff+8])[0]
                        # Check if it looks like a kernel pointer
                        is_kptr = (val >> 48) == 0xFFFF or val == 0
                        print(f"      0x{addr:x} → global 0x{gva:x} = 0x{val:016x}{' (kernel ptr)' if is_kptr else ''}")
                    else:
                        print(f"      0x{addr:x} → global 0x{gva:x}")
    
    # Try to find the phystokv function — it should do:
    # virt = phys - gPhysBase + gVirtBase
    print(f"\n{'='*70}")
    print(f" Looking for phystokv function via SUB+ADD with global pairs")
    print(f"{'='*70}")
    
    # Scan for functions that load two adjacent globals and do SUB+ADD
    # This is the phys↔virt translation pattern
    
    # Find LDP Xn, Xm, [Xk] or two LDR from adjacent addresses
    # near MRS DAIF or near the strings we found
    
    # Let's look at functions near the ml_phys_read_data string
    # The string is at 0x70586f2, so code referencing it should be nearby in __TEXT_EXEC
    # Actually code can be anywhere. Let me search in the kernel main __TEXT_EXEC segment
    
    # Search for functions containing both:
    # 1. Reference to gPhysBase/gVirtBase area
    # 2. LDR/LDP pattern
    # 3. SUB+ADD translation
    
    # Alternative: search for the specific pattern:
    #   LDP x_phys, x_virt, [Xn] ; or LDR x8, [Xn]; LDR x9, [Xn, #8]
    #   SUB Xn, X_input, x_virt
    #   ADD Xn, Xn, x_phys
    
    # Let's look at what phystokv xrefs find
    phystokv_va = STRINGS["phystokv"]
    xrefs = find_adrp_xrefs(kc, phystokv_va)
    if xrefs:
        xref = xrefs[0]
        xref_off = va2off(xref)
        # Find function containing this xref
        func_start = xref
        for back in range(4, 2048, 4):
            check_off = xref_off - back
            if check_off < 0:
                break
            word = struct.unpack('<I', kc[check_off:check_off+4])[0]
            if word == 0xD503237F:  # PACIBSP
                func_start = off2va(check_off)
                break
            if word == 0xD65F0FFF or (word & 0xFFFFFFE0) == 0xD65F03C0:
                func_start = off2va(check_off + 4)
                break
        
        print(f"\n  phystokv function via string xref: 0x{func_start:x}")
        func_off = va2off(func_start)
        chunk = kc[func_off:func_off + 512]
        instrs = list(md.disasm(chunk, func_start, count=128))
        
        print(f"  Full disassembly:")
        ret_count = 0
        for insn in instrs:
            print(f"    0x{insn.address:x}: {insn.mnemonic:8s} {insn.op_str}")
            if insn.mnemonic in ('ret', 'retab'):
                ret_count += 1
                if ret_count >= 3:
                    break

if __name__ == "__main__":
    main()
