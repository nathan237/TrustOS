#!/usr/bin/env python3
"""
T8020 B1 SecureROM — USB DEEP DIVE ANALYSIS (Part 2)
=====================================================
Targeted deep analysis of:
  1. ADRP-based MMIO address resolution (the real pattern Apple uses)
  2. USB controller base address identification
  3. Main DFU state machine at 0x100004CB8 (full annotated disasm)
  4. USB entry points from boot sequence
  5. Global state variables in SRAM (0x19C00xxxx)
  6. Buffer size tracking & data flow
"""

import struct
import os
from collections import defaultdict

from capstone import *
from capstone.arm64 import *

ROM_PATH = os.path.join(os.path.dirname(__file__), "securerom", "t8020_B1_securerom.bin")
ROM_BASE = 0x100000000

def load_rom():
    with open(ROM_PATH, "rb") as f:
        return f.read()

def get_md():
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    return md

def disasm_range(md, rom, start_off, end_off):
    code = rom[start_off:end_off]
    return list(md.disasm(code, ROM_BASE + start_off))

# ============================================================
# SECTION A: ADRP-based Address Resolution
# ============================================================
def analyze_adrp_addresses(md, rom):
    """Find ALL ADRP+ADD/LDR patterns to resolve actual MMIO and data addresses."""
    print("=" * 100)
    print("SECTION A: ADRP-BASED MMIO & DATA ADDRESS MAP")
    print("=" * 100)
    
    code_end = 0x1BC00
    instrs = disasm_range(md, rom, 0, code_end)
    
    # Build instruction map for forward-looking
    instr_list = list(instrs)
    instr_by_addr = {}
    for ins in instr_list:
        instr_by_addr[ins.address] = ins
    
    # Track ADRP pages and their subsequent ADD/LDR to get full addresses
    adrp_targets = {}  # addr -> (page, reg)
    full_addresses = defaultdict(list)  # full_addr -> [(code_addr, access_type)]
    
    for i, ins in enumerate(instr_list):
        if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
            if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_IMM:
                reg = ins.operands[0].reg
                page = ins.operands[1].imm
                adrp_targets[ins.address] = (page, reg)
                
                # Look at next few instructions for ADD/LDR/STR
                for j in range(i + 1, min(i + 5, len(instr_list))):
                    next_ins = instr_list[j]
                    
                    # ADD Xd, Xn, #offset (where Xn is the ADRP result)
                    if next_ins.mnemonic == 'add' and len(next_ins.operands) >= 3:
                        if (next_ins.operands[1].type == ARM64_OP_REG and 
                            next_ins.operands[1].reg == reg and 
                            next_ins.operands[2].type == ARM64_OP_IMM):
                            full_addr = page + next_ins.operands[2].imm
                            full_addresses[full_addr].append((next_ins.address, "ADD"))
                            break
                    
                    # LDR Xt, [Xn, #offset] (load from ADRP page + offset)
                    if next_ins.mnemonic in ('ldr', 'ldrb', 'ldrh', 'ldrsw', 'ldrsh', 'ldrsb'):
                        if len(next_ins.operands) >= 2 and next_ins.operands[1].type == ARM64_OP_MEM:
                            if next_ins.operands[1].mem.base == reg:
                                full_addr = page + next_ins.operands[1].mem.disp
                                full_addresses[full_addr].append((next_ins.address, f"LDR"))
                                break
                    
                    # STR Xt, [Xn, #offset]
                    if next_ins.mnemonic in ('str', 'strb', 'strh'):
                        if len(next_ins.operands) >= 2:
                            mem_op = next_ins.operands[-1] if next_ins.operands[-1].type == ARM64_OP_MEM else (next_ins.operands[1] if next_ins.operands[1].type == ARM64_OP_MEM else None)
                            if mem_op and mem_op.type == ARM64_OP_MEM and mem_op.mem.base == reg:
                                full_addr = page + mem_op.mem.disp
                                full_addresses[full_addr].append((next_ins.address, f"STR"))
                                break
                    
                    # If reg is overwritten, stop
                    if next_ins.operands and next_ins.operands[0].type == ARM64_OP_REG:
                        if next_ins.operands[0].reg == reg and next_ins.mnemonic not in ('ldr', 'ldrb', 'str', 'strb'):
                            break
    
    # Classify addresses by region
    regions = {
        "ROM_CODE":       (0x100000000, 0x10001BC00),
        "ROM_DATA":       (0x10001BC00, 0x100022000),
        "SRAM_0x19C008":  (0x19C008000, 0x19C009000),
        "SRAM_0x19C009":  (0x19C009000, 0x19C00A000),
        "SRAM_0x19C00A":  (0x19C00A000, 0x19C00B000),
        "SRAM_0x19C00B":  (0x19C00B000, 0x19C00C000),
        "SRAM_0x19C00C":  (0x19C00C000, 0x19C00D000),
        "SRAM_0x19C010":  (0x19C010000, 0x19C011000),
        "SRAM_0x19C014":  (0x19C014000, 0x19C015000),
        "MMIO_0x200xxx":  (0x200000000, 0x210000000),
        "MMIO_0x210xxx":  (0x210000000, 0x240000000),
    }
    
    by_region = defaultdict(list)
    for addr in sorted(full_addresses.keys()):
        classified = False
        for name, (start, end) in regions.items():
            if start <= addr < end:
                by_region[name].append((addr, full_addresses[addr]))
                classified = True
                break
        if not classified:
            by_region["OTHER"].append((addr, full_addresses[addr]))
    
    # Print SRAM global variables
    print("\n### SRAM Global Variables (state/context)")
    for region_name in sorted(by_region.keys()):
        if not region_name.startswith("SRAM"):
            continue
        entries = by_region[region_name]
        print(f"\n  --- {region_name} ({len(entries)} unique addresses) ---")
        for addr, refs in entries:
            access_types = [t for _, t in refs]
            reads = sum(1 for t in access_types if t.startswith("LDR"))
            writes = sum(1 for t in access_types if t.startswith("STR"))
            adds = sum(1 for t in access_types if t == "ADD")
            
            # Determine usage from code locations
            code_regions = set()
            for code_addr, _ in refs:
                off = code_addr - ROM_BASE
                if 0x2000 <= off < 0x3000: code_regions.add("USB_CTRL")
                elif 0x3000 <= off < 0x5000: code_regions.add("DFU")
                elif 0x5000 <= off < 0x6000: code_regions.add("IMG4")
                elif 0x6000 <= off < 0x7000: code_regions.add("PLATFORM")
                elif 0x8000 <= off < 0x9000: code_regions.add("PANIC")
                elif 0x9000 <= off < 0xA000: code_regions.add("SECURITY")
                elif 0xA000 <= off < 0xB000: code_regions.add("IMG4_FW")
                elif 0xB000 <= off < 0xD000: code_regions.add("IO_TRANS")
                elif 0xD000 <= off < 0xF000: code_regions.add("DER")
                elif 0xF000 <= off < 0x11000: code_regions.add("HEAP")
                elif 0x11000 <= off < 0x12000: code_regions.add("SYNC")
                elif 0 <= off < 0x2000: code_regions.add("BOOT")
                else: code_regions.add(f"@0x{off:X}")
            
            print(f"  0x{addr:X}: {len(refs):3d} refs (R={reads} W={writes} A={adds}) "
                  f"from [{', '.join(sorted(code_regions))}]")
    
    # Print MMIO addresses
    print("\n\n### MMIO Register Addresses")
    for region_name in sorted(by_region.keys()):
        if not region_name.startswith("MMIO"):
            continue
        entries = by_region[region_name]
        print(f"\n  --- {region_name} ({len(entries)} unique addresses) ---")
        for addr, refs in entries:
            # Try to identify the peripheral
            periph = "UNKNOWN"
            offset_in_periph = 0
            
            if 0x200350000 <= addr < 0x200360000:
                periph = "USB_OTG_DWC"
                offset_in_periph = addr - 0x200350000
            elif 0x200340000 <= addr < 0x200350000:
                periph = "USB_DEVICE"
                offset_in_periph = addr - 0x200340000
            elif 0x2000A0000 <= addr < 0x2000C0000:
                periph = "USB_PHY"
                offset_in_periph = addr - 0x2000A0000
            elif 0x200000000 <= addr < 0x200010000:
                periph = "AIC"
                offset_in_periph = addr - 0x200000000
            elif 0x200060000 <= addr < 0x200070000:
                periph = "TIMER"
                offset_in_periph = addr - 0x200060000
            elif 0x200100000 <= addr < 0x200110000:
                periph = "GPIO"
                offset_in_periph = addr - 0x200100000
            elif 0x2102B0000 <= addr < 0x2102C0000:
                periph = "PMGR"
                offset_in_periph = addr - 0x2102B0000
            elif 0x235000000 <= addr < 0x235100000:
                periph = "DART"
                offset_in_periph = addr - 0x235000000
            elif 0x23B100000 <= addr < 0x23B200000:
                periph = "SEP"
                offset_in_periph = addr - 0x23B100000
            elif 0x2003B0000 <= addr < 0x2003C0000:
                periph = "USB_EHCI/xHCI"
                offset_in_periph = addr - 0x2003B0000
            
            code_addrs = [f"0x{a:X}" for a, _ in refs[:5]]
            print(f"  0x{addr:012X} [{periph}+0x{offset_in_periph:04X}]: {len(refs)} refs from {', '.join(code_addrs)}")
    
    return full_addresses

# ============================================================
# SECTION B: USB Controller Register Access Patterns
# ============================================================
def analyze_usb_register_access(md, rom):
    """Find the exact USB controller base and all register accesses."""
    print("\n" + "=" * 100)
    print("SECTION B: USB CONTROLLER REGISTER ACCESS DETAIL")
    print("=" * 100)
    
    # Disassemble USB controller init region with full ADRP tracking
    usb_regions = [(0x2000, 0x3000, "USB_CTRL"), (0x3000, 0x5000, "DFU_HANDLER")]
    
    for start, end, name in usb_regions:
        print(f"\n### {name} (0x{ROM_BASE+start:X} - 0x{ROM_BASE+end:X})")
        
        instrs = disasm_range(md, rom, start, end)
        instr_list = list(instrs)
        
        # Track all ADRP results
        adrp_regs = {}  # reg -> page_addr
        mmio_accesses = []
        
        for i, ins in enumerate(instr_list):
            if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
                if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_IMM:
                    adrp_regs[ins.operands[0].reg] = ins.operands[1].imm
            
            # Track ADD that builds full address from ADRP
            if ins.mnemonic == 'add' and len(ins.operands) >= 3:
                if (ins.operands[1].type == ARM64_OP_REG and 
                    ins.operands[2].type == ARM64_OP_IMM and
                    ins.operands[1].reg in adrp_regs):
                    full = adrp_regs[ins.operands[1].reg] + ins.operands[2].imm
                    adrp_regs[ins.operands[0].reg] = full  # propagate to dst reg
            
            # Track LDR/STR with known base
            if ins.mnemonic in ('ldr', 'str', 'ldrb', 'strb', 'ldrh', 'strh', 'ldrsw'):
                for op in ins.operands:
                    if op.type == ARM64_OP_MEM and op.mem.base != 0:
                        if op.mem.base in adrp_regs:
                            base_val = adrp_regs[op.mem.base]
                            if isinstance(base_val, int):
                                full_addr = base_val + op.mem.disp
                                # Only report MMIO and SRAM
                                if full_addr >= 0x19C000000:
                                    rw = "R" if ins.mnemonic.startswith('ldr') else "W"
                                    mmio_accesses.append((ins.address, full_addr, rw, ins.mnemonic, ins.op_str))
            
            # Clear on function boundaries
            if ins.mnemonic == 'ret':
                adrp_regs = {}
        
        # Group by target address
        by_target = defaultdict(list)
        for code_addr, target, rw, mnem, ops in mmio_accesses:
            by_target[target].append((code_addr, rw, mnem, ops))
        
        print(f"  Found {len(mmio_accesses)} MMIO/SRAM accesses to {len(by_target)} unique addresses:")
        for target in sorted(by_target.keys()):
            refs = by_target[target]
            reads = sum(1 for _, rw, _, _ in refs if rw == "R")
            writes = sum(1 for _, rw, _, _ in refs if rw == "W")
            
            # Try to identify
            tag = ""
            if 0x19C008000 <= target < 0x19C009000:
                tag = f"SRAM[0x{target-0x19C008000:03X}]"
            elif 0x19C00B000 <= target < 0x19C00C000:
                tag = f"SRAM_B[0x{target-0x19C00B000:03X}]"
            elif 0x19C010000 <= target < 0x19C011000:
                tag = f"SRAM_10[0x{target-0x19C010000:03X}]"
            elif 0x19C014000 <= target < 0x19C015000:
                tag = f"SRAM_14[0x{target-0x19C014000:03X}]"
            elif 0x200000000 <= target < 0x300000000:
                tag = f"MMIO[0x{target:012X}]"
            
            first_refs = "; ".join(f"{'R' if rw == 'R' else 'W'}@0x{a:X}" for a, rw, _, _ in refs[:4])
            print(f"    0x{target:012X} ({tag:20s}): {len(refs):3d}× (R={reads} W={writes})  {first_refs}")

# ============================================================
# SECTION C: Main DFU Handler Deep Disassembly 
# ============================================================
def analyze_main_dfu_handler(md, rom):
    """Full annotated disassembly of the giant DFU function at 0x100004CB8."""
    print("\n" + "=" * 100)
    print("SECTION C: MAIN DFU STATE MACHINE — 0x100004CB8 (840 bytes)")
    print("=" * 100)
    
    FUNC_START = 0x4CB8
    FUNC_SIZE = 840
    
    KNOWN_FUNCS = {
        0x100010BD0: "memcpy",
        0x100010E00: "memset",
        0x100010D80: "bzero",
        0x10000F1EC: "heap_alloc",
        0x100008978: "panic",
        0x10000F3B0: "pool_alloc",
        0x100005E50: "img4_fn_5E50",
        0x100005F04: "img4_fn_5F04",
        0x100005F7C: "img4_fn_5F7C",
        0x1000062E8: "platform_62E8",
        0x100006754: "platform_6754",
        0x100008B58: "panic_with_info",
        0x1000126FC: "cert_26FC",
        0x100012924: "cert_2924",
        0x100012A6C: "cert_2A6C",
        0x100012AD8: "cert_2AD8",
        0x100012B44: "cert_2B44",
        0x10001C094: "data_C094",
        0x100009BA8: "security_BA8",
        0x100009B64: "security_B64",
        0x100009B78: "security_B78",
        0x100007DE0: "validate_DE0",
        0x100007DE8: "validate_DE8",
        0x100007168: "validate_168",
    }
    
    DFU_STATES = {0: "appIDLE", 1: "appDETACH", 2: "dfuIDLE", 3: "dfuDNLOAD-SYNC",
                  4: "dfuDNBUSY", 5: "dfuDNLOAD-IDLE", 6: "dfuMANIFEST-SYNC",
                  7: "dfuMANIFEST", 8: "dfuMANIFEST-WAIT-RESET", 9: "dfuUPLOAD-IDLE", 10: "dfuERROR"}
    
    instrs = disasm_range(md, rom, FUNC_START, FUNC_START + FUNC_SIZE)
    
    # Track state for annotations
    adrp_regs = {}
    sram_labels = {}
    branch_targets = set()
    
    # First pass: collect branch targets for labeling
    for ins in instrs:
        if ins.mnemonic in ('b', 'b.eq', 'b.ne', 'b.gt', 'b.lt', 'b.ge', 'b.le',
                            'b.hi', 'b.lo', 'b.hs', 'b.ls', 'cbz', 'cbnz', 'tbz', 'tbnz'):
            for op in ins.operands:
                if op.type == ARM64_OP_IMM:
                    branch_targets.add(op.imm)
    
    # Second pass: annotated disassembly
    print("\n  Legend: ◄ = annotation, ► = branch target\n")
    
    prev_cmp_reg = None
    prev_cmp_val = None
    
    for ins in instrs:
        label = ""
        if ins.address in branch_targets:
            label = f"  ► loc_{ins.address:X}:"
            print(label)
        
        ann = ""
        
        # Track ADRP
        if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
            if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_IMM:
                page = ins.operands[1].imm
                adrp_regs[ins.operands[0].reg] = page
                if page == 0x19C008000:
                    ann = "  ◄ SRAM page 0x19C008000"
                elif page == 0x19C00B000:
                    ann = "  ◄ SRAM page 0x19C00B000 (DFU state?)"
                elif page == 0x19C010000:
                    ann = "  ◄ SRAM page 0x19C010000 (USB buffers?)"
                elif page == 0x19C014000:
                    ann = "  ◄ SRAM page 0x19C014000"
                elif page >= 0x100000000 and page < 0x100080000:
                    ann = f"  ◄ ROM data page 0x{page:X}"
        
        # Track ADD from ADRP
        if ins.mnemonic == 'add' and len(ins.operands) >= 3:
            if (ins.operands[1].type == ARM64_OP_REG and 
                ins.operands[2].type == ARM64_OP_IMM and
                ins.operands[1].reg in adrp_regs):
                full = adrp_regs[ins.operands[1].reg] + ins.operands[2].imm
                adrp_regs[ins.operands[0].reg] = full
                if full >= 0x19C000000 and full < 0x1A0000000:
                    ann = f"  ◄ = 0x{full:X} (SRAM)"
                elif full >= 0x100000000 and full < 0x100022000:
                    # Check for string at this address
                    rom_off = full - ROM_BASE
                    if 0x1BC00 <= rom_off < 0x22000:
                        try:
                            end_pos = rom.index(0, rom_off, rom_off + 100)
                            s = rom[rom_off:end_pos].decode('ascii', errors='replace')
                            if len(s) > 2:
                                ann = f'  ◄ = 0x{full:X} → "{s}"'
                            else:
                                ann = f"  ◄ = 0x{full:X} (ROM data)"
                        except:
                            ann = f"  ◄ = 0x{full:X} (ROM data)"
        
        # Track LDR/STR with annotated base
        if ins.mnemonic in ('ldr', 'ldrb', 'ldrh', 'ldrsw', 'str', 'strb', 'strh'):
            for op in ins.operands:
                if op.type == ARM64_OP_MEM and op.mem.base != 0:
                    if op.mem.base in adrp_regs:
                        base = adrp_regs[op.mem.base]
                        if isinstance(base, int):
                            addr = base + op.mem.disp
                            rw = "READ" if ins.mnemonic.startswith('ldr') else "WRITE"
                            if 0x19C008000 <= addr < 0x19C009000:
                                ann = f"  ◄ {rw} SRAM[0x{addr - 0x19C008000:03X}] @ 0x{addr:X}"
                            elif 0x19C00B000 <= addr < 0x19C00C000:
                                off = addr - 0x19C00B000
                                tag = ""
                                if off == 0xC10:
                                    tag = " (DFU_STATE)"
                                elif off == 0xC08:
                                    tag = " (DFU_STATUS?)"
                                ann = f"  ◄ {rw} SRAM_B[0x{off:03X}]{tag} @ 0x{addr:X}"
                            elif 0x19C010000 <= addr < 0x19C011000:
                                ann = f"  ◄ {rw} SRAM_10[0x{addr-0x19C010000:03X}] @ 0x{addr:X}"
                            elif 0x19C014000 <= addr < 0x19C015000:
                                ann = f"  ◄ {rw} SRAM_14[0x{addr-0x19C014000:03X}] @ 0x{addr:X}"
                            elif addr >= 0x200000000:
                                ann = f"  ◄ {rw} MMIO @ 0x{addr:X}"
        
        # Annotate calls
        if ins.mnemonic == 'bl':
            target = ins.operands[0].imm if ins.operands and ins.operands[0].type == ARM64_OP_IMM else 0
            if target in KNOWN_FUNCS:
                ann = f"  ◄ CALL {KNOWN_FUNCS[target]}()"
            elif 0x100003000 <= target < 0x100005000:
                ann = f"  ◄ CALL dfu_{target & 0xFFFF:04X}()"
            elif 0x100002000 <= target < 0x100003000:
                ann = f"  ◄ CALL usb_ctrl_{target & 0xFFFF:04X}()"
            else:
                ann = f"  ◄ CALL sub_{target:X}()"
        
        # Annotate comparisons
        if ins.mnemonic == 'cmp' and len(ins.operands) >= 2:
            if ins.operands[1].type == ARM64_OP_IMM:
                v = ins.operands[1].imm
                prev_cmp_val = v
                if v in DFU_STATES:
                    ann = f"  ◄ CMP with {DFU_STATES[v]} (state={v})"
                elif v in (0x05AC, 0x1227, 0x1281):
                    names = {0x05AC: "Apple VID", 0x1227: "DFU PID", 0x1281: "Recovery PID"}
                    ann = f"  ◄ CMP with {names[v]}"
                elif v in (0, 1, 2, 3, 4, 5, 6):
                    # Could be DFU request type
                    dfu_req = {0: "DFU_DETACH", 1: "DFU_DNLOAD", 2: "DFU_UPLOAD",
                              3: "DFU_GETSTATUS", 4: "DFU_CLRSTATUS", 5: "DFU_GETSTATE", 6: "DFU_ABORT"}
                    ann = f"  ◄ CMP #{v} (maybe {dfu_req.get(v, '?')} or state={DFU_STATES.get(v, '?')})"
                else:
                    ann = f"  ◄ CMP #{v} (0x{v:X})"
        
        # Annotate conditional branches with context
        if ins.mnemonic.startswith('b.') and len(ins.operands) > 0:
            target = ins.operands[0].imm if ins.operands[0].type == ARM64_OP_IMM else 0
            cond_map = {'b.eq': 'EQUAL', 'b.ne': 'NOT_EQUAL', 'b.gt': 'GREATER',
                       'b.lt': 'LESS', 'b.ge': 'GREATER_EQ', 'b.le': 'LESS_EQ',
                       'b.hi': 'HIGHER', 'b.lo': 'LOWER', 'b.hs': 'HIGHER_SAME', 'b.ls': 'LOWER_SAME'}
            cond = cond_map.get(ins.mnemonic, ins.mnemonic)
            if target > ins.address:
                ann = f"  ◄ if {cond} → forward 0x{target:X}"
            else:
                ann = f"  ◄ if {cond} → back 0x{target:X} (LOOP?)"
        
        if ins.mnemonic == 'cbz' and len(ins.operands) >= 2:
            target = ins.operands[1].imm if ins.operands[1].type == ARM64_OP_IMM else 0
            ann = f"  ◄ if {ins.op_str.split(',')[0]} == 0 → 0x{target:X}"
        if ins.mnemonic == 'cbnz' and len(ins.operands) >= 2:
            target = ins.operands[1].imm if ins.operands[1].type == ARM64_OP_IMM else 0
            ann = f"  ◄ if {ins.op_str.split(',')[0]} != 0 → 0x{target:X}"
        
        # MOV constants
        if ins.mnemonic in ('mov', 'movz') and len(ins.operands) >= 2:
            if ins.operands[1].type == ARM64_OP_IMM:
                v = ins.operands[1].imm
                if v == 0x05AC: ann = "  ◄ Apple VID"
                elif v == 0x1227: ann = "  ◄ DFU PID"
                elif v == 0x1281: ann = "  ◄ Recovery PID"
        
        # RET
        if ins.mnemonic == 'ret':
            ann = "  ◄ RETURN"
        
        print(f"  0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str:45s}{ann}")

# ============================================================
# SECTION D: USB Entry Points from Boot Sequence
# ============================================================
def analyze_boot_to_usb_path(md, rom):
    """Trace how boot code reaches USB/DFU initialization."""
    print("\n" + "=" * 100)
    print("SECTION D: BOOT → USB ENTRY PATH")
    print("=" * 100)
    
    # The boot sequence:
    # 1. Reset vector @ 0x100000000
    # 2. Early init (PAC keys, MMU, etc.)
    # 3. Platform init
    # 4. USB controller init
    # 5. DFU mode entry
    
    # Trace from PLATFORM region into USB_CTRL
    print("\n### PLATFORM → USB_CTRL entry points:")
    
    # Already found from call graph:
    # USB_CTRL:0x1000023FC called from PLATFORM
    # USB_CTRL:0x100002D38 called from DFU
    
    # Let's trace 0x1000068A4 (platform) which calls DFU
    platform_callers = [
        (0x1000068A4, "platform_68A4 → DFU init"),
        (0x1000069BC, "platform_69BC → DFU state"),
        (0x100006A24, "platform_6A24 → DFU init"),
        (0x100006EB8, "platform_6EB8 → DFU entry"),
        (0x100006EE4, "platform_6EE4 → USB setup"),
    ]
    
    for func_addr, desc in platform_callers:
        off = func_addr - ROM_BASE
        # Find function end
        end_off = off + 512
        for check_off in range(off + 4, off + 512, 4):
            insn_bytes = struct.unpack_from("<I", rom, check_off)[0]
            if insn_bytes == 0xD65F03C0:  # RET
                end_off = check_off + 4
                break
        
        instrs = disasm_range(md, rom, off, end_off)
        
        print(f"\n  --- {desc} (0x{func_addr:X}, {end_off-off} bytes) ---")
        
        adrp_regs = {}
        for ins in instrs:
            ann = ""
            
            if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
                if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_IMM:
                    adrp_regs[ins.operands[0].reg] = ins.operands[1].imm
            
            if ins.mnemonic == 'add' and len(ins.operands) >= 3:
                if (ins.operands[1].type == ARM64_OP_REG and 
                    ins.operands[2].type == ARM64_OP_IMM and
                    ins.operands[1].reg in adrp_regs):
                    full = adrp_regs[ins.operands[1].reg] + ins.operands[2].imm
                    adrp_regs[ins.operands[0].reg] = full
                    if full >= 0x19C000000:
                        ann = f"  ◄ = 0x{full:X}"
            
            if ins.mnemonic == 'bl':
                target = ins.operands[0].imm if ins.operands and ins.operands[0].type == ARM64_OP_IMM else 0
                t_off = target - ROM_BASE
                if 0x2000 <= t_off < 0x3000:
                    ann = f"  ◄ CALL USB_CTRL"
                elif 0x3000 <= t_off < 0x5000:
                    ann = f"  ◄ CALL DFU_HANDLER"
                elif target == 0x100008978:
                    ann = "  ◄ panic()"
                else:
                    ann = f"  ◄ sub_{target:X}"
            
            if ins.mnemonic in ('ldr', 'ldrb', 'str', 'strb'):
                for op in ins.operands:
                    if op.type == ARM64_OP_MEM and op.mem.base != 0:
                        if op.mem.base in adrp_regs:
                            base = adrp_regs[op.mem.base]
                            if isinstance(base, int):
                                addr = base + op.mem.disp
                                rw = "R" if ins.mnemonic.startswith('ldr') else "W"
                                if addr >= 0x19C000000:
                                    ann = f"  ◄ {rw} 0x{addr:X}"
            
            print(f"    0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str:40s}{ann}")

# ============================================================
# SECTION E: DFU State Variable Map  
# ============================================================
def analyze_dfu_state_variables(md, rom):
    """Map all SRAM variables used by DFU code and their meanings."""
    print("\n" + "=" * 100)
    print("SECTION E: DFU STATE VARIABLES IN SRAM")
    print("=" * 100)
    
    # From the main DFU handler, we see accesses to 0x19C00B000 + 0xC10
    # This appears to be the DFU state byte
    # Let's map ALL SRAM offsets used by DFU code
    
    instrs = disasm_range(md, rom, 0x3000, 0x5000)
    instr_list = list(instrs)
    
    adrp_regs = {}
    sram_ops = defaultdict(list)  # sram_addr -> [(code_addr, R/W, width)]
    
    for ins in instr_list:
        if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
            if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_IMM:
                adrp_regs[ins.operands[0].reg] = ins.operands[1].imm
        
        if ins.mnemonic == 'add' and len(ins.operands) >= 3:
            if (ins.operands[1].type == ARM64_OP_REG and 
                ins.operands[2].type == ARM64_OP_IMM and
                ins.operands[1].reg in adrp_regs):
                full = adrp_regs[ins.operands[1].reg] + ins.operands[2].imm
                adrp_regs[ins.operands[0].reg] = full
        
        if ins.mnemonic in ('ldr', 'ldrb', 'ldrh', 'ldrsw', 'ldrsb', 'ldrsh',
                            'str', 'strb', 'strh', 'stp', 'ldp'):
            for op in ins.operands:
                if op.type == ARM64_OP_MEM and op.mem.base != 0:
                    if op.mem.base in adrp_regs:
                        base = adrp_regs[op.mem.base]
                        if isinstance(base, int) and 0x19C000000 <= base < 0x1A0000000:
                            addr = base + op.mem.disp
                            rw = "R" if ins.mnemonic.startswith('ldr') or ins.mnemonic == 'ldp' else "W"
                            width = 1 if 'b' in ins.mnemonic else (2 if 'h' in ins.mnemonic else (4 if 'w' in ins.mnemonic.replace('ldrsw','w') else 8))
                            sram_ops[addr].append((ins.address, rw, width, ins.mnemonic))
        
        # Reset on RET
        if ins.mnemonic == 'ret':
            adrp_regs = {}
    
    # Print organized by SRAM page
    print(f"\n  Found {sum(len(v) for v in sram_ops.values())} SRAM accesses to {len(sram_ops)} unique addresses from DFU code:")
    
    for addr in sorted(sram_ops.keys()):
        ops = sram_ops[addr]
        reads = sum(1 for _, rw, _, _ in ops if rw == "R")
        writes = sum(1 for _, rw, _, _ in ops if rw == "W")
        widths = set(w for _, _, w, _ in ops)
        width_str = "/".join(f"{w}B" for w in sorted(widths))
        
        page_off = addr & 0xFFF
        page = addr & ~0xFFF
        
        # Guess meaning based on usage patterns
        meaning = ""
        if reads > 5 and writes == 0:
            meaning = "CONFIG/CONSTANT"
        elif writes > reads:
            meaning = "OUTPUT/STATUS"
        elif reads > 0 and writes > 0:
            meaning = "STATE VARIABLE"
        
        # Specific known addresses
        if addr == 0x19C00BC10:
            meaning = "★ DFU_STATE (byte)"
        
        code_refs = ", ".join(f"{'R' if rw == 'R' else 'W'}@0x{a:X}" for a, rw, _, _ in ops[:6])
        print(f"  0x{addr:X} [page+0x{page_off:03X}] ({width_str:5s}) R={reads:2d} W={writes:2d} {meaning:20s} {code_refs}")

# ============================================================
# SECTION F: Data Flow — USB Receive → Buffer → Verify
# ============================================================
def analyze_data_flow(md, rom):
    """Trace how USB received data flows to img4_verify."""
    print("\n" + "=" * 100) 
    print("SECTION F: DATA FLOW — USB RECEIVE → BUFFER → VERIFY")
    print("=" * 100)
    
    # Key path:
    # USB Controller (DWC) receives data via EP0
    # DFU handler copies data to a buffer (memcpy)
    # When download complete, buffer passed to img4_verify
    
    # Find ALL memcpy calls in USB+DFU region with context
    print("\n### memcpy calls in USB_CTRL + DFU region:")
    
    instrs = disasm_range(md, rom, 0x2000, 0x5000)
    instr_list = list(instrs)
    
    for idx, ins in enumerate(instr_list):
        if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
            target = ins.operands[0].imm
            name = ""
            if target == 0x100010BD0: name = "memcpy"
            elif target == 0x100010E00: name = "memset"
            elif target == 0x100010D80: name = "bzero"
            elif target == 0x10000F1EC: name = "heap_alloc"
            elif target == 0x10000F3B0: name = "pool_alloc"
            elif target == 0x10000F468: name = "pool_free?"
            elif target == 0x100010EA4: name = "heap_fn_EA4"
            elif target == 0x100010014: name = "heap_fn_014"
            
            if name:
                print(f"\n  ★ {name}() @ 0x{ins.address:X}")
                # Print context
                start = max(0, idx - 10)
                end = min(len(instr_list), idx + 3)
                
                adrp_regs = {}
                for j in range(start, end):
                    ci = instr_list[j]
                    marker = " >>>" if j == idx else "    "
                    
                    ann = ""
                    if ci.mnemonic == 'adrp' and len(ci.operands) >= 2:
                        if ci.operands[0].type == ARM64_OP_REG and ci.operands[1].type == ARM64_OP_IMM:
                            adrp_regs[ci.operands[0].reg] = ci.operands[1].imm
                            ann = f"  (page=0x{ci.operands[1].imm:X})"
                    
                    if ci.mnemonic == 'add' and len(ci.operands) >= 3:
                        if (ci.operands[1].type == ARM64_OP_REG and 
                            ci.operands[2].type == ARM64_OP_IMM and
                            ci.operands[1].reg in adrp_regs):
                            full = adrp_regs[ci.operands[1].reg] + ci.operands[2].imm
                            ann = f"  (=0x{full:X})"
                    
                    if ci.mnemonic in ('mov', 'movz') and len(ci.operands) >= 2:
                        if ci.operands[1].type == ARM64_OP_IMM:
                            v = ci.operands[1].imm
                            if v > 0x10:
                                ann = f"  (=0x{v:X}, {v})"
                    
                    if ci.mnemonic == 'ldr' and len(ci.operands) >= 2:
                        for op in ci.operands:
                            if op.type == ARM64_OP_MEM and op.mem.base != 0:
                                if op.mem.base in adrp_regs:
                                    addr = adrp_regs[op.mem.base] + op.mem.disp
                                    ann = f"  (mem@0x{addr:X})"
                    
                    print(f"  {marker} 0x{ci.address:X}: {ci.mnemonic:8s} {ci.op_str:40s}{ann}")
    
    # Track calls to img4_verify from DFU
    print(f"\n\n### DFU → IMG4_VERIFY calls:")
    # From the call graph: DFU_HANDLER → IMG4_VERIFY: 3 calls
    # The main DFU handler 0x100004CB8 calls 0x100005E50, 0x100005F04, 0x100005F7C
    
    img4_targets = [0x100005E50, 0x100005F04, 0x100005F7C]
    for target in img4_targets:
        print(f"\n  IMG4 function 0x{target:X}:")
        off = target - ROM_BASE
        # Get first 30 instructions
        instrs = disasm_range(md, rom, off, off + 200)
        
        adrp_regs = {}
        for ins in instrs[:30]:
            ann = ""
            if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
                if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_IMM:
                    adrp_regs[ins.operands[0].reg] = ins.operands[1].imm
            
            if ins.mnemonic == 'bl':
                t = ins.operands[0].imm if ins.operands else 0
                known = {0x100010BD0: "memcpy", 0x100010E00: "memset", 0x10000F1EC: "heap_alloc",
                        0x100008978: "panic"}
                ann = f"  ◄ {known.get(t, f'sub_{t:X}')}"
            
            print(f"    0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str:40s}{ann}")


# ============================================================
# MAIN
# ============================================================
def main():
    print("╔══════════════════════════════════════════════════════════════════════════════════╗")
    print("║  T8020 B1 SecureROM — USB DEEP DIVE ANALYSIS (Part 2)                          ║")
    print("╚══════════════════════════════════════════════════════════════════════════════════╝")
    
    rom = load_rom()
    md = get_md()
    
    addresses = analyze_adrp_addresses(md, rom)
    analyze_usb_register_access(md, rom)
    analyze_main_dfu_handler(md, rom)
    analyze_boot_to_usb_path(md, rom)
    analyze_dfu_state_variables(md, rom)
    analyze_data_flow(md, rom)
    
    print("\n" + "=" * 100)
    print("DEEP DIVE COMPLETE")
    print("=" * 100)

if __name__ == "__main__":
    main()
