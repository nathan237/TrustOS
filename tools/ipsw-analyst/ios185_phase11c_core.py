#!/usr/bin/env python3
"""
iOS 18.5 Phase 11c — Trace actual ml_phys_read core at 0x807b4f8
================================================================
The ml_phys_read_data wrapper at 0x807b738 calls 0x807b4f8 with w1=4.
The ml_phys_write wrappers branch to 0x807b7a8.
Let's disassemble these core functions to find gPhysBase/gVirtBase.
"""
import struct
from pathlib import Path
from capstone import *

KC_PATH = Path("extracted/kernelcache_iPhone12,3_18_5.raw")
KC_BASE = 0xfffffff007004000

# The core implementations
TARGETS = {
    "ml_phys_read_core":      0xfffffff00807b4f8,
    "ml_phys_read_data":      0xfffffff00807b738,   # wrapper, calls 0x807b4f8
    "ml_phys_write_core":     0xfffffff00807b7a8,   # write entry from jump table
}

def va2off(va):
    return va - KC_BASE

def off2va(off):
    return KC_BASE + off

def main():
    kc = Path(KC_PATH).read_bytes()
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = False
    
    print("=" * 70)
    print(" iOS 18.5 Phase 11c — ml_phys_read/write Core Implementation")
    print("=" * 70)
    
    for name, va in TARGETS.items():
        off = va2off(va)
        # Read generous amount for disassembly
        chunk = kc[off:off + 2048]
        instrs = list(md.disasm(chunk, va, count=512))
        
        print(f"\n{'━'*70}")
        print(f" {name} @ 0x{va:x}")
        print(f"{'━'*70}")
        
        # Find function boundary
        func_end_idx = len(instrs) - 1
        ret_count = 0
        for i, insn in enumerate(instrs):
            if insn.mnemonic in ('ret', 'retab'):
                ret_count += 1
                if ret_count == 1:
                    func_end_idx = i
                    # Don't break — might have epilog patterns
                if ret_count >= 3:  # After 3 rets, probably past our function
                    break
        
        # Find ALL ADRP targets (potential global variable references)
        print(f"\n  Global variable references (ADRP+ADD/LDR):")
        adrp_targets = []
        for i, insn in enumerate(instrs[:func_end_idx + 50]):
            if insn.mnemonic == 'adrp' and i + 1 < len(instrs):
                next_insn = instrs[i + 1]
                try:
                    parts = insn.op_str.split(',')
                    dest_reg = parts[0].strip()
                    imm = int(parts[-1].strip().lstrip('#'), 16)
                    page = (insn.address & ~0xFFF) + imm
                    
                    if next_insn.mnemonic == 'add':
                        add_parts = next_insn.op_str.split(',')
                        add_imm = int(add_parts[-1].strip().lstrip('#'), 16)
                        target = page + add_imm
                        
                        # Read value at target
                        toff = va2off(target)
                        if 0 <= toff < len(kc) - 8:
                            val = struct.unpack('<Q', kc[toff:toff+8])[0]
                            is_chained = (val >> 63) & 1
                            print(f"    0x{insn.address:x}: {dest_reg} → 0x{target:x} = 0x{val:016x} {'(chained fixup)' if is_chained else ''}")
                            adrp_targets.append((insn.address, target, val, dest_reg))
                        else:
                            print(f"    0x{insn.address:x}: {dest_reg} → 0x{target:x}")
                            
                    elif next_insn.mnemonic == 'ldr':
                        # ADRP+LDR: loading from page + offset
                        ldr_parts = next_insn.op_str.split('#')
                        if len(ldr_parts) > 1:
                            ldr_off = int(ldr_parts[-1].rstrip(']').rstrip('!'), 16)
                            target = page + ldr_off
                            toff = va2off(target)
                            if 0 <= toff < len(kc) - 8:
                                val = struct.unpack('<Q', kc[toff:toff+8])[0]
                                print(f"    0x{insn.address:x}: {dest_reg} ← [0x{target:x}] = 0x{val:016x}")
                                adrp_targets.append((insn.address, target, val, dest_reg))
                except:
                    pass
        
        # Print full disassembly with annotations
        print(f"\n  Disassembly (up to 2nd retab):")
        ret_seen = 0
        for i, insn in enumerate(instrs):
            annotation = ""
            
            if insn.mnemonic == 'mrs':
                annotation = "  ◄ MRS (save state?)"
            elif insn.mnemonic == 'msr':
                annotation = "  ◄ MSR (set state?)"
            elif insn.mnemonic in ('ldp',):
                annotation = "  ◄ LOAD PAIR"
            elif insn.mnemonic == 'sub':
                annotation = "  ◄ SUB (phys→virt addr translation?)"
            elif insn.mnemonic == 'add' and i > 5:
                annotation = "  ◄ ADD (addr translation?)"
            elif insn.mnemonic in ('ldr', 'ldrb', 'ldrh', 'ldrsw'):
                if 'x0' in insn.op_str.split(',')[0]:
                    annotation = "  ◄ LOAD (return value / phys read)"
                else:
                    annotation = "  ◄ LOAD"
            elif insn.mnemonic in ('str', 'strb', 'strh'):
                annotation = "  ◄ STORE (phys write?)"
            elif insn.mnemonic == 'bl':
                try:
                    target = int(insn.op_str.lstrip('#'), 16)
                    annotation = f"  ◄ CALL 0x{target:x}"
                except:
                    annotation = "  ◄ CALL"
            elif insn.mnemonic in ('b', 'b.eq', 'b.ne', 'b.hi', 'b.lo', 'b.hs', 'b.ls', 'b.ge', 'b.lt', 'b.gt', 'b.le'):
                try:
                    target = int(insn.op_str.lstrip('#'), 16)
                    annotation = f"  ◄ BRANCH to 0x{target:x}"
                except:
                    annotation = "  ◄ BRANCH"
            elif insn.mnemonic in ('cbz', 'cbnz', 'tbz', 'tbnz'):
                annotation = "  ◄ COND BRANCH"
            elif insn.mnemonic == 'pacibsp':
                annotation = "  ◄ FUNCTION START"
            elif insn.mnemonic in ('ret', 'retab'):
                annotation = "  ◄ RETURN"
                ret_seen += 1
            elif insn.mnemonic == 'autibsp':
                annotation = "  ◄ AUTH RETURN"
            elif insn.mnemonic == 'brk':
                annotation = "  ◄ PAC FAILURE TRAP"
            
            print(f"    0x{insn.address:x}: {insn.mnemonic:8s} {insn.op_str}{annotation}")
            
            if ret_seen >= 2:
                break
        
        # Look for the phys-virt translation pattern
        print(f"\n  Pattern analysis:")
        for i, insn in enumerate(instrs[:func_end_idx+20]):
            # SUB + ADD pattern for phys→virt:
            #   SUB Xn, Xphys, gPhysBase  
            #   ADD Xn, Xn, gVirtBase
            if insn.mnemonic == 'sub' and 'x' in insn.op_str:
                if i + 1 < len(instrs) and instrs[i+1].mnemonic == 'add':
                    print(f"    SUB+ADD pair at 0x{insn.address:x}:")
                    print(f"      {insn.mnemonic} {insn.op_str}")
                    print(f"      {instrs[i+1].mnemonic} {instrs[i+1].op_str}")
            
            # LDP pattern for loading gPhysBase+gVirtBase together:
            if insn.mnemonic == 'ldp' and 'x' in insn.op_str.split(',')[0]:
                # Check if this loads from a global
                print(f"    LDP at 0x{insn.address:x}: {insn.op_str}")
    
    # Now let's look at the full region 0x807b400-0x807bc00 to understand the complete file
    print(f"\n{'━'*70}")
    print(f" Full region scan 0x807b400 - 0x807bc00")
    print(f"{'━'*70}")
    
    start_va = 0xfffffff00807b400
    end_va = 0xfffffff00807bc00
    start_off = va2off(start_va)
    chunk = kc[start_off:start_off + (end_va - start_va)]
    instrs = list(md.disasm(chunk, start_va, count=2000))
    
    # Find all function boundaries
    funcs = []
    for i, insn in enumerate(instrs):
        if insn.mnemonic == 'pacibsp':
            funcs.append(insn.address)
    
    print(f"  Functions (pacibsp) in this region:")
    for f in funcs:
        # Find next ret/retab
        f_off = va2off(f)
        f_chunk = kc[f_off:f_off + 512]
        f_instrs = list(md.disasm(f_chunk, f, count=128))
        f_end = f
        f_size = 0
        for insn in f_instrs:
            if insn.mnemonic in ('ret', 'retab'):
                f_end = insn.address
                f_size = f_end - f + 4
                break
        
        # Check for LDP (which could load gPhysBase/gVirtBase)
        has_ldp = any(i.mnemonic == 'ldp' for i in f_instrs[:f_size//4 + 1])
        has_sub = any(i.mnemonic == 'sub' for i in f_instrs[:f_size//4 + 1])
        has_mrs = any(i.mnemonic == 'mrs' for i in f_instrs[:f_size//4 + 1])
        
        print(f"    0x{f:x} → 0x{f_end:x} ({f_size} bytes) LDP={has_ldp} SUB={has_sub} MRS={has_mrs}")

if __name__ == "__main__":
    main()
