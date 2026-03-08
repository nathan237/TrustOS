#!/usr/bin/env python3
"""
T8020 B1 SecureROM — DEEP BLR/BR BACKWARD SLICE
=================================================
For EVERY indirect call/jump, trace backward through the instruction
stream to find exactly where the function pointer is loaded from.
Goal: find ALL SRAM-sourced pointers (not just the ones the simple
register tracker caught), and determine which ones an attacker might
be able to control.
"""

import struct, os, sys
from collections import defaultdict
from capstone import *
from capstone.arm64 import *

ROM_PATH = os.path.join(os.path.dirname(__file__), "securerom", "t8020_B1_securerom.bin")
ROM_BASE = 0x100000000
ACTIVE_END = 0x25000

# Key addresses
POOL_ALLOC  = 0x10000F3B0
POOL_FREE   = 0x10000F468
HEAP_ALLOC  = 0x10000F1EC
MEMCPY      = 0x100010BD0
PANIC       = 0x100008978

# SRAM range
SRAM_BASE = 0x19C000000
SRAM_END  = 0x1A0000000

# ROM data range
ROM_DATA_BASE = ROM_BASE + ACTIVE_END
ROM_DATA_END  = ROM_BASE + 0x80000

def load_rom():
    with open(ROM_PATH, "rb") as f:
        return f.read()

def get_md():
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    return md

def find_functions(rom, md):
    """Find functions via STP prologue + BL targets."""
    funcs = set()
    for off in range(0, ACTIVE_END, 4):
        code = rom[off:off+4]
        for ins in md.disasm(code, ROM_BASE + off):
            if ins.mnemonic == 'stp' and 'x29, x30' in ins.op_str and '[sp' in ins.op_str:
                funcs.add(ins.address)
            if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
                target = ins.operands[0].imm
                if ROM_BASE <= target < ROM_BASE + ACTIVE_END:
                    funcs.add(target)
    return sorted(funcs)

def disasm_range(rom, md, start, end):
    """Disassemble a range of ROM addresses."""
    off_start = start - ROM_BASE
    off_end = end - ROM_BASE
    if off_start < 0 or off_end > len(rom):
        return []
    code = rom[off_start:off_end]
    return list(md.disasm(code, start))

def disasm_function(rom, md, start, max_bytes=8192):
    """Disassemble from start to first RET."""
    off = start - ROM_BASE
    if off < 0 or off >= len(rom):
        return []
    code = rom[off:off+max_bytes]
    result = []
    for ins in md.disasm(code, start):
        result.append(ins)
        if ins.mnemonic == 'ret':
            break
    return result


def deep_backward_slice(instrs, target_idx):
    """
    Starting from instrs[target_idx] (a BLR/BR), trace backward to find
    where the register value comes from. Returns a dict describing the source.
    
    Tracks: ADRP+ADD, LDR from resolved address, MOV+MOVK chains, MOV reg,
    and LDR from [base + offset] where base was resolved.
    """
    target_ins = instrs[target_idx]
    if not target_ins.operands:
        return {'type': 'no_operands'}
    
    target_reg = target_ins.operands[0].reg
    
    # State tracking
    reg_values = {}       # reg -> immediate value
    reg_adrp = {}         # reg -> ADRP page
    reg_resolved = {}     # reg -> fully resolved address
    reg_mem_source = {}   # reg -> (base_addr, offset) loaded from memory
    reg_origin = {}       # reg -> description of origin
    
    # Walk forward from function start to target, building state
    for i in range(target_idx):
        ins = instrs[i]
        
        # ADRP: set page
        if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
            if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_IMM:
                r = ins.operands[0].reg
                reg_adrp[r] = ins.operands[1].imm
                reg_resolved[r] = ins.operands[1].imm
                reg_origin[r] = f"adrp 0x{ins.operands[1].imm:X}"
        
        # ADD: resolve ADRP+ADD
        elif ins.mnemonic == 'add' and len(ins.operands) >= 3:
            dst = ins.operands[0]
            src = ins.operands[1]
            imm = ins.operands[2]
            if dst.type == ARM64_OP_REG and src.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                if src.reg in reg_resolved:
                    full = reg_resolved[src.reg] + imm.imm
                    reg_resolved[dst.reg] = full
                    reg_origin[dst.reg] = f"resolved 0x{full:X}"
        
        # MOV/MOVZ: set immediate
        elif ins.mnemonic in ('mov', 'movz') and len(ins.operands) >= 2:
            dst = ins.operands[0]
            src = ins.operands[1]
            if dst.type == ARM64_OP_REG:
                if src.type == ARM64_OP_IMM:
                    reg_values[dst.reg] = src.imm
                    reg_resolved[dst.reg] = src.imm
                    reg_origin[dst.reg] = f"imm 0x{src.imm:X}"
                elif src.type == ARM64_OP_REG:
                    # MOV between registers
                    if src.reg in reg_resolved:
                        reg_resolved[dst.reg] = reg_resolved[src.reg]
                        reg_origin[dst.reg] = reg_origin.get(src.reg, 'reg_copy')
                    if src.reg in reg_mem_source:
                        reg_mem_source[dst.reg] = reg_mem_source[src.reg]
        
        # MOVK: modify existing value with shifted immediate
        elif ins.mnemonic == 'movk' and len(ins.operands) >= 2:
            dst = ins.operands[0]
            src = ins.operands[1]
            if dst.type == ARM64_OP_REG and src.type == ARM64_OP_IMM:
                r = dst.reg
                val = src.imm
                shift = 0
                if src.shift.type != 0:
                    shift = src.shift.value
                if r in reg_values:
                    reg_values[r] = (reg_values[r] & ~(0xFFFF << shift)) | (val << shift)
                    reg_resolved[r] = reg_values[r]
                    reg_origin[r] = f"movk_chain 0x{reg_values[r]:X}"
                elif r in reg_resolved:
                    reg_resolved[r] = (reg_resolved[r] & ~(0xFFFF << shift)) | (val << shift)
                    reg_origin[r] = f"movk_chain 0x{reg_resolved[r]:X}"
                else:
                    reg_values[r] = val << shift
                    reg_resolved[r] = val << shift
                    reg_origin[r] = f"movk_partial 0x{val << shift:X}"
        
        # LDR: load from memory
        elif ins.mnemonic in ('ldr', 'ldrsw') and len(ins.operands) >= 2:
            dst = ins.operands[0]
            src = ins.operands[1]
            if dst.type == ARM64_OP_REG and src.type == ARM64_OP_MEM:
                base_reg = src.mem.base
                disp = src.mem.disp
                
                if base_reg in reg_resolved:
                    mem_addr = reg_resolved[base_reg] + disp
                    reg_mem_source[dst.reg] = mem_addr
                    reg_resolved[dst.reg] = mem_addr  # pointer value at this addr
                    reg_origin[dst.reg] = f"ldr [0x{mem_addr:X}]"
                else:
                    # Unknown base — mark as ldr from unknown
                    reg_origin[dst.reg] = f"ldr [{ins.reg_name(base_reg)}+0x{disp:X}]"
                    if dst.reg in reg_resolved:
                        del reg_resolved[dst.reg]
        
        # LDP: load pair
        elif ins.mnemonic == 'ldp' and len(ins.operands) >= 3:
            # Just mark both dst regs as modified
            for op_idx in range(2):
                if ins.operands[op_idx].type == ARM64_OP_REG:
                    r = ins.operands[op_idx].reg
                    if r in reg_resolved:
                        del reg_resolved[r]
                    reg_origin[r] = f"ldp"
        
        # BL: x0 is return value, caller-saved regs may be clobbered
        elif ins.mnemonic == 'bl':
            # x0 holds return value
            # Don't clear x19-x28 (callee-saved)
            for clobber_name in ['x0', 'x1', 'x2', 'x3', 'x4', 'x5', 'x6', 'x7',
                                  'x8', 'x9', 'x10', 'x11', 'x12', 'x13', 'x14', 'x15',
                                  'x16', 'x17', 'x18']:
                # We need the capstone register ID — approximate by clearing known
                pass
            # For simplicity, mark x0-x18 as potentially clobbered after BL
            # But we track by register ID, not name, so let's just note the BL
            if ins.operands and ins.operands[0].type == ARM64_OP_IMM:
                bl_target = ins.operands[0].imm
                # If x0 was our target register, note it's a return value
                # We handle this below
    
    # Now check what we know about the target register
    result = {'reg': instrs[target_idx].reg_name(target_reg)}
    
    if target_reg in reg_mem_source:
        mem_addr = reg_mem_source[target_reg]
        result['type'] = 'mem_load'
        result['address'] = mem_addr
        if SRAM_BASE <= mem_addr < SRAM_END:
            result['region'] = 'SRAM'
        elif ROM_BASE <= mem_addr < ROM_DATA_END:
            result['region'] = 'ROM'
        else:
            result['region'] = f'MMIO/OTHER'
        result['origin_chain'] = reg_origin.get(target_reg, 'unknown')
    elif target_reg in reg_resolved:
        addr = reg_resolved[target_reg]
        if SRAM_BASE <= addr < SRAM_END:
            result['type'] = 'resolved_sram'
            result['address'] = addr
            result['region'] = 'SRAM'
        elif ROM_BASE <= addr < ROM_DATA_END:
            result['type'] = 'resolved_rom'
            result['address'] = addr
            result['region'] = 'ROM'
        else:
            result['type'] = 'resolved_other'
            result['address'] = addr
        result['origin_chain'] = reg_origin.get(target_reg, 'unknown')
    else:
        result['type'] = 'unresolved'
        result['origin_chain'] = reg_origin.get(target_reg, 'completely_unknown')
    
    return result


def classify_sram_region(addr):
    """Classify SRAM address into functional region."""
    if 0x19C008000 <= addr < 0x19C008100:
        return "task/scheduler_ctx"
    elif 0x19C008800 <= addr < 0x19C008C00:
        return "platform_config"
    elif 0x19C00B000 <= addr < 0x19C00C000:
        return "img4/dfu_ctx"
    elif 0x19C010000 <= addr < 0x19C011000:
        return "io/block_device"
    elif 0x19C011000 <= addr < 0x19C012000:
        return "pool/heap_meta"
    elif 0x19C012000 <= addr < 0x19C014000:
        return "transfer_ctx"
    else:
        return "unknown_sram"


def check_dfu_reachability(rom, md, funcs, target_func_addr):
    """
    Check if a function is reachable from DFU handlers (0x3000-0x5000 range).
    Simple: check if any DFU function calls target_func_addr directly or
    calls a function that calls it (2-deep).
    """
    dfu_funcs = [f for f in funcs if 0x100003000 <= f < 0x100005000]
    usb_funcs = [f for f in funcs if 0x100002000 <= f < 0x100003000]
    
    # Build callgraph (direct BL targets)
    callgraph = defaultdict(set)
    for func_start in funcs:
        instrs = disasm_function(rom, md, func_start, 4096)
        for ins in instrs:
            if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
                callgraph[func_start].add(ins.operands[0].imm)
    
    # Check 1-deep and 2-deep reachability
    for dfu in dfu_funcs + usb_funcs:
        if target_func_addr in callgraph[dfu]:
            return (True, dfu, 1)
        for callee in callgraph[dfu]:
            if target_func_addr in callgraph.get(callee, set()):
                return (True, dfu, 2)
    
    return (False, 0, 0)


def main():
    rom = load_rom()
    md = get_md()
    
    print("T8020 B1 SecureROM — DEEP BLR/BR BACKWARD SLICE")
    print(f"ROM size: {len(rom)} bytes")
    print()
    
    # Find functions
    print("Finding functions...")
    funcs = find_functions(rom, md)
    print(f"Found {len(funcs)} functions\n")
    
    # =========================================================================
    # PHASE 1: Deep backward slice on ALL BLR/BR sites
    # =========================================================================
    print("=" * 100)
    print("PHASE 1: DEEP BACKWARD SLICE — ALL INDIRECT CALLS/JUMPS")
    print("=" * 100)
    
    all_blr = []     # (func, addr, mnemonic, reg_name, slice_result)
    sram_blr = []    # subset: SRAM-sourced
    rom_blr = []     # subset: ROM-sourced
    unknown_blr = [] # subset: unresolved
    
    seen_sites = set()  # (blr_addr) to deduplicate
    
    for func_start in funcs:
        instrs = disasm_function(rom, md, func_start, 8192)
        if not instrs:
            continue
        
        for i, ins in enumerate(instrs):
            if ins.mnemonic in ('blr', 'br'):
                if ins.address in seen_sites:
                    continue
                seen_sites.add(ins.address)
                
                result = deep_backward_slice(instrs, i)
                entry = (func_start, ins.address, ins.mnemonic, result.get('reg', '?'), result)
                all_blr.append(entry)
                
                if result.get('region') == 'SRAM' or result.get('type') == 'resolved_sram':
                    sram_blr.append(entry)
                elif result.get('region') == 'ROM' or result.get('type') == 'resolved_rom':
                    rom_blr.append(entry)
                elif result['type'] == 'unresolved':
                    unknown_blr.append(entry)
    
    print(f"\nTotal unique BLR/BR sites: {len(all_blr)}")
    print(f"  SRAM-sourced:   {len(sram_blr)}")
    print(f"  ROM-sourced:    {len(rom_blr)}")
    print(f"  Unresolved:     {len(unknown_blr)}")
    print()
    
    # =========================================================================
    # PHASE 2: SRAM-sourced indirect calls — full detail  
    # =========================================================================
    print("=" * 100)
    print("PHASE 2: ALL SRAM-SOURCED INDIRECT CALLS (ATTACK SURFACE)")
    print("=" * 100)
    
    sram_targets = defaultdict(list)  # sram_addr -> [(func, blr_addr, mnemonic)]
    
    for func, addr, mnem, reg, result in sram_blr:
        sram_addr = result.get('address', 0)
        sram_targets[sram_addr].append((func, addr, mnem, reg, result))
    
    print(f"\nUnique SRAM pointer addresses: {len(sram_targets)}")
    
    for sram_addr in sorted(sram_targets.keys()):
        callers = sram_targets[sram_addr]
        region = classify_sram_region(sram_addr)
        print(f"\n  {'='*80}")
        print(f"  SRAM 0x{sram_addr:X}  [{region}]")
        print(f"  Used by {len(callers)} BLR/BR sites:")
        for func, addr, mnem, reg, result in callers:
            chain = result.get('origin_chain', '?')
            print(f"    func=0x{func:X}  @0x{addr:X}: {mnem} {reg}  chain={chain}")
    
    # =========================================================================
    # PHASE 3: ROM-sourced indirect calls — are these ROM function tables?
    # =========================================================================
    print(f"\n\n{'='*100}")
    print("PHASE 3: ROM-SOURCED INDIRECT CALLS (FUNCTION TABLES)")
    print("=" * 100)
    
    rom_targets = defaultdict(list)
    for func, addr, mnem, reg, result in rom_blr:
        rom_addr = result.get('address', 0)
        rom_targets[rom_addr].append((func, addr, mnem, reg))
    
    print(f"\nUnique ROM pointer addresses: {len(rom_targets)}")
    for rom_addr in sorted(rom_targets.keys()):
        callers = rom_targets[rom_addr]
        # Read the pointer value from ROM if in range
        off = rom_addr - ROM_BASE
        ptr_val = 0
        if 0 <= off < len(rom) - 8:
            ptr_val = struct.unpack_from('<Q', rom, off)[0]
        print(f"  ROM[0x{rom_addr:X}] → 0x{ptr_val:X}  ({len(callers)} callers)")
        for func, addr, mnem, reg in callers[:3]:  # show max 3
            print(f"    func=0x{func:X}  @0x{addr:X}: {mnem} {reg}")
    
    # =========================================================================
    # PHASE 4: UNRESOLVED — deeper analysis with context
    # =========================================================================
    print(f"\n\n{'='*100}")
    print("PHASE 4: UNRESOLVED INDIRECT CALLS — CONTEXT DUMP")
    print("=" * 100)
    print(f"\n{len(unknown_blr)} sites with unresolved source\n")
    
    # Group by the origin chain hint
    by_origin = defaultdict(list)
    for func, addr, mnem, reg, result in unknown_blr:
        origin = result.get('origin_chain', 'completely_unknown')
        by_origin[origin].append((func, addr, mnem, reg))
    
    for origin, entries in sorted(by_origin.items()):
        print(f"\n  Origin: {origin}  ({len(entries)} sites)")
        for func, addr, mnem, reg in entries[:5]:  # max 5 per type
            print(f"    func=0x{func:X}  @0x{addr:X}: {mnem} {reg}")
        if len(entries) > 5:
            print(f"    ... and {len(entries)-5} more")
    
    # For the most interesting unknown sites, dump surrounding instructions
    print(f"\n\n{'='*100}")
    print("PHASE 5: CONTEXT DUMP FOR KEY UNRESOLVED BLR SITES")
    print("=" * 100)
    
    # Focus on BLR sites in USB/DFU range (0x2000-0x5000) that are unresolved
    usb_dfu_unknown = [(f, a, m, r, res) for f, a, m, r, res in unknown_blr
                       if 0x100002000 <= a < 0x100005000]
    
    print(f"\nUnresolved BLR/BR in USB/DFU code range: {len(usb_dfu_unknown)}")
    
    seen_blr_addrs = set()
    for func, blr_addr, mnem, reg, result in usb_dfu_unknown:
        if blr_addr in seen_blr_addrs:
            continue
        seen_blr_addrs.add(blr_addr)
        
        print(f"\n  --- @0x{blr_addr:X}: {mnem} {reg} (in func 0x{func:X}) ---")
        # Dump 10 instructions before the BLR
        instrs = disasm_function(rom, md, func, 8192)
        blr_idx = None
        for i, ins in enumerate(instrs):
            if ins.address == blr_addr:
                blr_idx = i
                break
        
        if blr_idx is not None:
            start = max(0, blr_idx - 12)
            end = min(len(instrs), blr_idx + 3)
            for i in range(start, end):
                ins = instrs[i]
                marker = " >>>" if ins.address == blr_addr else "    "
                ann = ""
                if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
                    t = ins.operands[0].imm
                    known = {POOL_ALLOC: 'pool_alloc', POOL_FREE: 'pool_free',
                             HEAP_ALLOC: 'heap_alloc', MEMCPY: 'memcpy', PANIC: 'panic',
                             0x100004454: 'get_dfu_ctx', 0x100001D14: 'acquire_lock',
                             0x100001F0C: 'enter_critical', 0x100001F84: 'release_lock'}
                    ann = f"  ; {known.get(t, f'sub_{t:X}')}"
                if ins.mnemonic == 'adrp' and len(ins.operands) >= 2 and ins.operands[1].type == ARM64_OP_IMM:
                    ann = f"  ; page=0x{ins.operands[1].imm:X}"
                print(f"  {marker} 0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str}{ann}")
    
    # =========================================================================
    # PHASE 6: SRAM WRITE PATHS FROM DFU/USB
    # =========================================================================
    print(f"\n\n{'='*100}")
    print("PHASE 6: CAN DFU/USB CODE WRITE TO SRAM POINTER LOCATIONS?")
    print("=" * 100)
    
    # For each SRAM pointer target, find ALL functions that write near that address
    # and check if they're reachable from DFU handlers
    
    print("\nBuilding callgraph...")
    callgraph = defaultdict(set)
    for func_start in funcs:
        instrs = disasm_function(rom, md, func_start, 4096)
        for ins in instrs:
            if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
                callgraph[func_start].add(ins.operands[0].imm)
    
    dfu_funcs = set(f for f in funcs if 0x100003000 <= f < 0x100005000)
    usb_funcs = set(f for f in funcs if 0x100002000 <= f < 0x100003000)
    
    # Find transitive closure of DFU-reachable functions (3 levels deep)
    dfu_reachable = set()
    level0 = dfu_funcs | usb_funcs
    dfu_reachable.update(level0)
    level1 = set()
    for f in level0:
        level1.update(callgraph.get(f, set()))
    dfu_reachable.update(level1)
    level2 = set()
    for f in level1:
        level2.update(callgraph.get(f, set()))
    dfu_reachable.update(level2)
    level3 = set()
    for f in level2:
        level3.update(callgraph.get(f, set()))
    dfu_reachable.update(level3)
    
    print(f"DFU/USB functions: {len(dfu_funcs)} + {len(usb_funcs)}")
    print(f"Transitively reachable (3-deep): {len(dfu_reachable)}")
    
    # Find all STR instructions that write to SRAM, per function
    print(f"\nSearching for SRAM writes in all functions...")
    sram_writers = defaultdict(list)  # sram_region -> [(func, str_addr, exact_addr)]
    
    for func_start in funcs:
        instrs = disasm_function(rom, md, func_start, 8192)
        if not instrs:
            continue
        
        # Track register values
        reg_resolved = {}
        
        for ins in instrs:
            if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
                if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_IMM:
                    reg_resolved[ins.operands[0].reg] = ins.operands[1].imm
            
            elif ins.mnemonic == 'add' and len(ins.operands) >= 3:
                if (ins.operands[0].type == ARM64_OP_REG and 
                    ins.operands[1].type == ARM64_OP_REG and
                    ins.operands[2].type == ARM64_OP_IMM and
                    ins.operands[1].reg in reg_resolved):
                    reg_resolved[ins.operands[0].reg] = reg_resolved[ins.operands[1].reg] + ins.operands[2].imm
            
            elif ins.mnemonic in ('mov', 'movz') and len(ins.operands) >= 2:
                if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_REG:
                    if ins.operands[1].reg in reg_resolved:
                        reg_resolved[ins.operands[0].reg] = reg_resolved[ins.operands[1].reg]
            
            elif ins.mnemonic in ('str', 'stp') and len(ins.operands) >= 2:
                for op in ins.operands:
                    if op.type == ARM64_OP_MEM:
                        base = op.mem.base
                        disp = op.mem.disp
                        if base in reg_resolved:
                            write_addr = reg_resolved[base] + disp
                            if SRAM_BASE <= write_addr < SRAM_END:
                                sram_writers[write_addr].append((func_start, ins.address))
    
    # Now for each SRAM pointer target from BLR analysis, check writers
    print(f"\n--- SRAM POINTER TARGETS vs WRITERS vs DFU REACHABILITY ---\n")
    
    critical_targets = sorted(sram_targets.keys())
    
    for sram_addr in critical_targets:
        region = classify_sram_region(sram_addr)
        blr_count = len(sram_targets[sram_addr])
        
        # Find writers within ±16 bytes (pointer might be in a struct)
        nearby_writers = []
        for check_addr in range(sram_addr - 16, sram_addr + 17, 8):
            for func, str_addr in sram_writers.get(check_addr, []):
                nearby_writers.append((func, str_addr, check_addr))
        
        # Check DFU reachability of writers
        dfu_reachable_writers = []
        for func, str_addr, exact_addr in nearby_writers:
            if func in dfu_reachable:
                dfu_reachable_writers.append((func, str_addr, exact_addr))
        
        status = "SAFE" if not dfu_reachable_writers else "★ DFU-REACHABLE WRITER"
        
        print(f"  SRAM 0x{sram_addr:X} [{region}]  BLR refs={blr_count}  "
              f"writers={len(nearby_writers)}  dfu_writers={len(dfu_reachable_writers)}  [{status}]")
        
        if dfu_reachable_writers:
            for func, str_addr, exact_addr in dfu_reachable_writers:
                in_dfu = "DFU" if func in dfu_funcs else "USB" if func in usb_funcs else "CALLEE"
                print(f"    ★ {in_dfu} func=0x{func:X}  str@0x{str_addr:X} → [0x{exact_addr:X}]")
        elif nearby_writers:
            for func, str_addr, exact_addr in nearby_writers[:3]:
                print(f"    writer: func=0x{func:X}  str@0x{str_addr:X} → [0x{exact_addr:X}]")
    
    # =========================================================================
    # PHASE 7: SRAM FUNCTION POINTER MAP (complete)
    # =========================================================================
    print(f"\n\n{'='*100}")
    print("PHASE 7: COMPLETE SRAM FUNCTION POINTER MAP")
    print("=" * 100)
    
    # Gather ALL SRAM addresses used as indirect call/jump sources
    all_sram_ptrs = set()
    for entry in all_blr:
        func, addr, mnem, reg, result = entry
        if result.get('region') == 'SRAM':
            all_sram_ptrs.add(result.get('address', 0))
    
    # Also check BR sites from the big dispatch table
    for func, addr, mnem, reg, result in all_blr:
        if result.get('type') == 'mem_load' and result.get('region') == 'SRAM':
            all_sram_ptrs.add(result['address'])
    
    print(f"\nTotal unique SRAM addresses used as function pointers: {len(all_sram_ptrs)}")
    print(f"\nComplete map:")
    for sram_addr in sorted(all_sram_ptrs):
        region = classify_sram_region(sram_addr)
        writer_count = 0
        for check in range(sram_addr - 8, sram_addr + 9, 8):
            writer_count += len(sram_writers.get(check, []))
        dfu_writable = any(
            func in dfu_reachable
            for check in range(sram_addr - 8, sram_addr + 9, 8)
            for func, _ in sram_writers.get(check, [])
        )
        flag = "★ DFU-WRITABLE" if dfu_writable else ""
        print(f"  0x{sram_addr:X}  [{region:20s}]  writers={writer_count:3d}  {flag}")
    
    # =========================================================================
    # FINAL VERDICT
    # =========================================================================
    print(f"\n\n{'='*100}")
    print("FINAL VERDICT: BLR POISONING EXPLOITABILITY")
    print("=" * 100)
    
    exploitable = []
    for sram_addr in sorted(sram_targets.keys()):
        nearby_writers = []
        for check_addr in range(sram_addr - 16, sram_addr + 17, 8):
            for func, str_addr in sram_writers.get(check_addr, []):
                if func in dfu_reachable:
                    nearby_writers.append((func, str_addr, check_addr))
        if nearby_writers:
            exploitable.append((sram_addr, nearby_writers))
    
    if exploitable:
        print(f"\n★★★ {len(exploitable)} SRAM POINTER TARGETS HAVE DFU-REACHABLE WRITERS ★★★\n")
        for sram_addr, writers in exploitable:
            region = classify_sram_region(sram_addr)
            print(f"  0x{sram_addr:X} [{region}]:")
            for func, str_addr, exact_addr in writers:
                print(f"    writer func=0x{func:X} str@0x{str_addr:X} → [0x{exact_addr:X}]")
    else:
        print("\n  No SRAM pointer targets have DFU-reachable writers.")
        print("  BLR poisoning requires an independent memory corruption primitive.")


if __name__ == "__main__":
    main()
