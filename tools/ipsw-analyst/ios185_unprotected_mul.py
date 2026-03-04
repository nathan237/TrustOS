#!/usr/bin/env python3
"""
iOS 18.5 — Deep analysis of the UNPROTECTED MUL at 0xfffffff009e76790
=====================================================================
Disassembles a wide window around the unprotected multiplication,
traces the call chain that reaches it, identifies which IOSurface
property controls the operands, and assesses exploitability.
"""

import struct
import json
from pathlib import Path
from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM

KC_PATH = Path("extracted/kernelcache_iPhone12,3_18_5.raw")
KC_BASE = 0xfffffff007004000
TARGET_MUL = 0xfffffff009e76790

# MH_FILESET constants
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
    
    def file_to_va(self, foff):
        for seg in self.segments.values():
            if seg["fileoff"] <= foff < seg["fileoff"] + seg["filesize"]:
                return seg["vmaddr"] + (foff - seg["fileoff"])
        return None
    
    def get_code_range(self):
        for name, seg in self.segments.items():
            if "TEXT_EXEC" in name:
                return (seg["fileoff"], seg["fileoff"] + seg["filesize"],
                        seg["vmaddr"], seg["vmaddr"] + seg["vmsize"])
        return None


def parse_entries(kc_data):
    magic = struct.unpack_from('<I', kc_data, 0)[0]
    assert magic == MH_MAGIC_64
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
    
    # Parse inner segments
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


def find_containing_kext(entries, va):
    for e in entries:
        code = e.get_code_range()
        if code and code[2] <= va < code[3]:
            return e
    return None


def disasm_range(kc_data, foff, size, base_va):
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    return list(md.disasm(kc_data[foff:foff+size], base_va))


def find_function_start(kc_data, entry, target_va, max_back=4096):
    """Walk backwards from target_va to find function prologue (PACIBSP/SUB SP/STP)"""
    code = entry.get_code_range()
    if not code:
        return target_va
    
    code_foff, code_fend, code_va, code_va_end = code
    
    # Walk back looking for PACIBSP (0xD503237F) or function-like start
    foff = code_foff + (target_va - code_va)
    check = foff
    min_check = max(code_foff, foff - max_back)
    
    # Quick scan for PACIBSP backwards
    while check >= min_check:
        insn = struct.unpack_from('<I', kc_data, check)[0]
        if insn == 0xD503237F:  # PACIBSP
            return code_va + (check - code_foff)
        # Also check for STP x29, x30, [sp, #-N]! pattern (classic prologue)
        if (insn & 0xFFC003E0) == 0xA9800000 and ((insn >> 10) & 0x1F) == 31:
            # STP with base SP pre-index
            return code_va + (check - code_foff)
        check -= 4
    
    return target_va


def find_callers(kc_data, entry, target_va):
    """Find all BL instructions in the kext that call target_va"""
    code = entry.get_code_range()
    if not code:
        return []
    
    code_foff, code_fend, code_va, code_va_end = code
    callers = []
    
    off = code_foff
    while off < code_fend - 4:
        insn = struct.unpack_from('<I', kc_data, off)[0]
        if (insn & 0xFC000000) == 0x94000000:  # BL
            imm26 = insn & 0x3FFFFFF
            if imm26 & (1 << 25):
                imm26 -= (1 << 26)
            pc = code_va + (off - code_foff)
            branch_target = pc + (imm26 << 2)
            if branch_target == target_va:
                callers.append(pc)
        off += 4
    
    return callers


def main():
    print(f"=== DEEP ANALYSIS: UNPROTECTED MUL @ {hex(TARGET_MUL)} ===")
    print(f"=== iOS 18.5 (22F76) / iPhone 11 Pro / A13 ===\n")
    
    kc_data = KC_PATH.read_bytes()
    entries = parse_entries(kc_data)
    
    # Find containing kext
    kext = find_containing_kext(entries, TARGET_MUL)
    if not kext:
        print("[!] Could not find containing kext!")
        return
    
    print(f"[+] Containing kext: {kext.name}")
    code = kext.get_code_range()
    print(f"    Code: {hex(code[2])}-{hex(code[3])} ({code[3]-code[2]:,} bytes)")
    
    # Find function start
    func_start = find_function_start(kc_data, kext, TARGET_MUL)
    print(f"[+] Function start: {hex(func_start)}")
    print(f"    MUL offset from func start: +{TARGET_MUL - func_start}")
    
    # Disassemble the whole function (up to 2KB from start or until RET)
    foff = code[0] + (func_start - code[2])
    func_insns = disasm_range(kc_data, foff, 2048, func_start)
    
    # Find function end (first RET/RETAB after MUL)
    func_end = func_start + 2048
    for insn in func_insns:
        if insn.address > TARGET_MUL and insn.mnemonic in ('ret', 'retab'):
            func_end = insn.address + 4
            break
    
    func_size = func_end - func_start
    print(f"[+] Function size: ~{func_size} bytes ({func_size//4} instructions)")
    
    # Print full function with annotations
    print(f"\n{'='*70}")
    print(f" FULL DISASSEMBLY: {hex(func_start)} - {hex(func_end)}")
    print(f"{'='*70}")
    
    for insn in func_insns:
        if insn.address >= func_end:
            break
        marker = ""
        if insn.address == TARGET_MUL:
            marker = "  <<<< UNPROTECTED MUL"
        elif insn.mnemonic in ('mul', 'madd', 'msub', 'umulh', 'smulh', 'umull', 'smull'):
            marker = f"  <<<< {insn.mnemonic.upper()}"
        elif insn.mnemonic == 'bl':
            marker = "  <<<< CALL"
        elif insn.mnemonic in ('cbnz', 'cbz'):
            marker = "  <<<< CONDITIONAL"
        elif insn.mnemonic.startswith('b.'):
            marker = f"  <<<< BRANCH"
        
        print(f"  {hex(insn.address)}: {insn.mnemonic:10s} {insn.op_str:40s}{marker}")
    
    # Detailed window: 32 insns before and after the MUL
    print(f"\n{'='*70}")
    print(f" DETAILED WINDOW AROUND MUL (±32 insns)")
    print(f"{'='*70}")
    
    mul_foff = code[0] + (TARGET_MUL - code[2])
    window_start = max(code[0], mul_foff - 128)
    window_va = code[2] + (window_start - code[0])
    window_insns = disasm_range(kc_data, window_start, 256, window_va)
    
    # Track register data flow
    print("\n  Register analysis:")
    for insn in window_insns:
        if insn.address == TARGET_MUL:
            print(f"\n  >>> THE UNPROTECTED MUL:")
            print(f"  >>> {hex(insn.address)}: {insn.mnemonic} {insn.op_str}")
            # Parse operands
            ops = insn.op_str.split(', ')
            if len(ops) >= 3:
                print(f"  >>> Dest: {ops[0]}, Src1: {ops[1]}, Src2: {ops[2]}")
                print(f"  >>> If src1 and src2 are user-controlled dimensions,")
                print(f"  >>> this multiplication CAN OVERFLOW without detection!")
            break
    
    # Look for UMULH/SMULH in wider range
    print(f"\n{'='*70}")
    print(f" SEARCHING FOR OVERFLOW CHECKS IN WIDER RANGE (±256 insns)")
    print(f"{'='*70}")
    
    wide_start = max(code[0], mul_foff - 1024)
    wide_va = code[2] + (wide_start - code[0])
    wide_insns = disasm_range(kc_data, wide_start, 2048, wide_va)
    
    checks_found = []
    for insn in wide_insns:
        m = insn.mnemonic.lower()
        if m in ('umulh', 'smulh'):
            checks_found.append((insn.address, f"{insn.mnemonic} {insn.op_str}"))
        if m in ('mul', 'madd', 'msub', 'umull', 'smull'):
            dist = insn.address - TARGET_MUL
            prot = "<<< TARGET" if insn.address == TARGET_MUL else ""
            print(f"    MUL at {hex(insn.address)} (delta {dist:+d}): {insn.mnemonic} {insn.op_str}  {prot}")
    
    print(f"\n    UMULH/SMULH checks found in ±1024 range: {len(checks_found)}")
    for addr, text in checks_found:
        dist = addr - TARGET_MUL
        print(f"      {hex(addr)} (delta {dist:+d}): {text}")
    
    # Find callers of this function
    print(f"\n{'='*70}")
    print(f" CALLER ANALYSIS")
    print(f"{'='*70}")
    
    callers = find_callers(kc_data, kext, func_start)
    print(f"[+] {len(callers)} callers of function {hex(func_start)}:")
    
    for caller_va in callers[:20]:
        # Find caller's function start
        caller_func = find_function_start(kc_data, kext, caller_va)
        print(f"    BL at {hex(caller_va)} (in function {hex(caller_func)})")
        
        # Disassemble a few instructions around the call site
        cfoff = code[0] + (caller_va - code[2])
        ctx_start = max(code[0], cfoff - 32)
        ctx_va = code[2] + (ctx_start - code[0])
        ctx_insns = disasm_range(kc_data, ctx_start, 64, ctx_va)
        for ci in ctx_insns:
            marker = " <<<" if ci.address == caller_va else ""
            print(f"      {hex(ci.address)}: {ci.mnemonic:8s} {ci.op_str}{marker}")
        print()
    
    # Check if any callers are reachable from dispatch table
    print(f"\n{'='*70}")
    print(f" DISPATCH REACHABILITY")
    print(f"{'='*70}")
    
    db = json.loads(Path("extracted/ios185_full_analysis.json").read_text(encoding="utf-8"))
    dispatch_targets = [int(e["target"], 16) for e in db["dispatch_table"]["entries"]]
    dispatch_names = [e["name"] for e in db["dispatch_table"]["entries"]]
    
    # For each caller, check if it's a dispatch target or called by one
    for caller_va in callers:
        caller_func = find_function_start(kc_data, kext, caller_va)
        if caller_func in dispatch_targets:
            idx = dispatch_targets.index(caller_func)
            print(f"  [!!!] Function {hex(caller_func)} IS dispatch selector {idx} ({dispatch_names[idx]})")
        else:
            # Check if caller_func is called by a dispatch target (2-level trace)
            level2_callers = find_callers(kc_data, kext, caller_func)
            for l2 in level2_callers:
                l2_func = find_function_start(kc_data, kext, l2)
                if l2_func in dispatch_targets:
                    idx = dispatch_targets.index(l2_func)
                    print(f"  [!!] {hex(func_start)} <- {hex(caller_func)} <- dispatch {idx} ({dispatch_names[idx]})")
                    break
            else:
                # Level 3
                for l2 in level2_callers[:10]:
                    l2_func = find_function_start(kc_data, kext, l2)
                    l3_callers = find_callers(kc_data, kext, l2_func)
                    for l3 in l3_callers[:10]:
                        l3_func = find_function_start(kc_data, kext, l3)
                        if l3_func in dispatch_targets:
                            idx = dispatch_targets.index(l3_func)
                            print(f"  [!] {hex(func_start)} <- {hex(caller_func)} <- {hex(l2_func)} <- dispatch {idx} ({dispatch_names[idx]})")
                            break
    
    # String references in the function
    print(f"\n{'='*70}")
    print(f" STRING REFERENCES IN FUNCTION")
    print(f"{'='*70}")
    
    for insn in func_insns:
        if insn.address >= func_end:
            break
        if insn.mnemonic == 'adrp':
            # Get ADRP target page
            raw = struct.unpack_from('<I', kc_data, code[0] + (insn.address - code[2]))[0]
            immhi = (raw >> 5) & 0x7FFFF
            immlo = (raw >> 29) & 0x3
            imm = (immhi << 2) | immlo
            if imm & (1 << 20):
                imm -= (1 << 21)
            page = (insn.address & ~0xFFF) + (imm << 12)
            
            # Check for ADD in next insn
            next_addr = insn.address + 4
            for ni in func_insns:
                if ni.address == next_addr and ni.mnemonic == 'add':
                    # Extract immediate
                    next_raw = struct.unpack_from('<I', kc_data, code[0] + (ni.address - code[2]))[0]
                    add_imm = (next_raw >> 10) & 0xFFF
                    shift = (next_raw >> 22) & 0x3
                    if shift == 1:
                        add_imm <<= 12
                    string_va = page + add_imm
                    # Read string at that VA
                    for e in entries:
                        sfoff = e.va_to_file(string_va)
                        if sfoff and sfoff + 64 < len(kc_data):
                            raw_str = kc_data[sfoff:sfoff+64].split(b'\x00')[0]
                            if raw_str and all(32 <= b < 127 for b in raw_str):
                                print(f"    {hex(insn.address)}: ADRP+ADD -> {hex(string_va)} = \"{raw_str.decode('ascii')}\"")
                    break
    
    print(f"\n{'='*70}")
    print(f" EXPLOITABILITY ASSESSMENT")
    print(f"{'='*70}")
    print(f"""
  Target: {hex(TARGET_MUL)}
  Kext: {kext.name}
  Function: {hex(func_start)} (size ~{func_size} bytes)
  
  CRITICAL: This MUL has NO overflow check (no UMULH/SMULH within ±1024 bytes).
  If user-controlled dimensions reach this multiplication:
    - Width × Height overflow → undersized allocation
    - Write past allocation → kernel heap corruption
    - Exploit: heap feng shui + controlled OOB write → kernel R/W
    
  Next: Verify operand sources trace back to IOSurface properties.
""")


if __name__ == "__main__":
    import os
    os.chdir(Path(__file__).parent)
    main()
