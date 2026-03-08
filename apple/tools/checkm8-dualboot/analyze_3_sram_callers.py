#!/usr/bin/env python3
"""
Trace callers of Target 2 writer (0x100002844) and Target 3 true writer (0x10000AD14).
"""

import struct
import re
from capstone import *

ROM_PATH = r"C:\Users\nathan\Documents\Scripts\OSrust\tools\checkm8-dualboot\securerom\t8020_B1_securerom.bin"
ROM_BASE = 0x100000000
ROM_SIZE = 524288

with open(ROM_PATH, "rb") as f:
    rom_data = f.read()

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = False
md.skipdata = True

def va_to_offset(va):
    off = va - ROM_BASE
    return off if 0 <= off < len(rom_data) else None

def disasm_range(start_va, end_va, label=None):
    off_s, off_e = va_to_offset(start_va), va_to_offset(end_va)
    if off_s is None or off_e is None:
        return []
    insns = [i for i in md.disasm(rom_data[off_s:off_e], start_va) if i.mnemonic != '.byte']
    if label:
        print(f"\n{'='*80}")
        print(f"  {label}")
        print(f"{'='*80}")
    for i in insns:
        print(f"  {i.address:#012x}:  {i.mnemonic:8s} {i.op_str}")
    return insns

def find_func_start(va, max_search=0x300):
    for off in range(0, max_search, 4):
        check = va - off
        ci = va_to_offset(check)
        if ci is None: continue
        insns = list(md.disasm(rom_data[ci:ci+4], check))
        if insns and insns[0].mnemonic == 'stp' and 'x29, x30' in insns[0].op_str:
            prev_ci = va_to_offset(check - 4)
            if prev_ci:
                prev = list(md.disasm(rom_data[prev_ci:prev_ci+4], check-4))
                if prev and prev[0].mnemonic == 'sub' and 'sp' in prev[0].op_str:
                    return check - 4
                if prev and prev[0].mnemonic == 'stp':
                    pp = va_to_offset(check - 8)
                    if pp:
                        pp_i = list(md.disasm(rom_data[pp:pp+4], check-8))
                        if pp_i and pp_i[0].mnemonic == 'sub' and 'sp' in pp_i[0].op_str:
                            return check - 8
                    return check - 4
            return check
    return None

def find_func_end(va, max_search=0x400):
    for off in range(0, max_search, 4):
        ci = va_to_offset(va + off)
        if ci is None: continue
        insns = list(md.disasm(rom_data[ci:ci+4], va+off))
        if insns and insns[0].mnemonic == 'ret':
            return va + off + 4
    return va + max_search

# Full ROM scan
all_insns = [i for i in md.disasm(rom_data[:ROM_SIZE], ROM_BASE) if i.mnemonic != '.byte']

# ============================================================================
print("="*80)
print("TARGET 2: Callers of 0x100002844 (interrupt handler registration)")
print("="*80)

callers_2844 = [0x10000A15C, 0x10000A178, 0x10000AFF8, 0x10000BB44]

for caller_addr in callers_2844:
    func_start = find_func_start(caller_addr)
    func_end = find_func_end(func_start if func_start else caller_addr)
    start = func_start if func_start else caller_addr - 0x40
    
    print(f"\n[*] Caller at {caller_addr:#x}, function starts at {start:#x}:")
    insns = disasm_range(start, min(func_end, start + 0x120),
                        f"func_{start:#x} — calls register_handler at {caller_addr:#x}")
    
    # Trace what x0, x1, x2 are just before the BL
    print(f"  --- What are the arguments to register_handler? ---")
    # Look backward from the BL for mov/ldr into x0, x1, x2
    for ins in insns:
        if ins.address >= caller_addr - 0x30 and ins.address < caller_addr:
            if any(f'x{r}' in ins.op_str.split(',')[0] or f'w{r}' in ins.op_str.split(',')[0] 
                   for r in [0,1,2]):
                print(f"    ARG SETUP: {ins.address:#x}: {ins.mnemonic} {ins.op_str}")

# ============================================================================
print("\n\n" + "="*80)
print("TARGET 3: True writer of 0x19C010620 (func at 0x10000AD14)")
print("="*80)

# The writer function
print("\n[*] Writer function at 0x10000AD14:")
insns_ad14 = disasm_range(0x10000AD14, 0x10000AD80, "func_0x10000AD14 — STR x1 to [0x19C010620]")

# Who calls 0x10000AD14?
print("\n[*] Callers of 0x10000AD14:")
callers_ad14 = []
for ins in all_insns:
    if ins.mnemonic in ('bl', 'b'):
        m = re.match(r'#(0x[0-9a-fA-F]+)', ins.op_str)
        if m and int(m.group(1), 16) == 0x10000AD14:
            callers_ad14.append(ins.address)
            print(f"  {ins.mnemonic.upper()} at {ins.address:#x}")

for caller_addr in callers_ad14:
    func_start = find_func_start(caller_addr)
    func_end = find_func_end(func_start if func_start else caller_addr)
    start = func_start if func_start else caller_addr - 0x40
    
    print(f"\n[*] Caller function (calls 0x10000AD14 at {caller_addr:#x}):")
    disasm_range(start, min(func_end, start + 0x120),
                f"func_{start:#x} — calls set_callback at {caller_addr:#x}")

# ============================================================================
# Also check who calls the BLR consumer 0x10000AD80
# ============================================================================
print("\n\n" + "="*80)
print("TARGET 3: Caller of BLR consumer 0x10000AD80")
print("="*80)

# BL at 0x10000AC5C
func_ac = find_func_start(0x10000AC5C)
if func_ac:
    print(f"\n[*] Function calling 0x10000AD80 (at BL 0x10000AC5C), starts at {func_ac:#x}:")
    disasm_range(func_ac, find_func_end(func_ac), f"func_{func_ac:#x}")

# ============================================================================
# Check callers of 0x10000ADDC (frequency function)
# ============================================================================
print("\n\n" + "="*80)
print("TARGET 3: Callers of freq function 0x10000ADDC")  
print("="*80)

for caller in [0x10000ADC4, 0x10000AE84, 0x10000AEAC]:
    func_start = find_func_start(caller)
    if func_start:
        print(f"\n[*] Caller at {caller:#x}, func starts at {func_start:#x}:")
        disasm_range(func_start, find_func_end(func_start, 0x60),
                    f"func_{func_start:#x}")

# ============================================================================
# Check early init caller of Target 1 writer
# ============================================================================
print("\n\n" + "="*80)
print("TARGET 1: Caller at 0x1000017FC (early init)")
print("="*80)

func_17fc = find_func_start(0x1000017FC)
if func_17fc:
    print(f"\n[*] Early init function calling 0x100008AF0, starts at {func_17fc:#x}:")
    disasm_range(func_17fc, find_func_end(func_17fc, 0x200),
                f"func_{func_17fc:#x} — early init, calls canary writer")

# ============================================================================
# DFU reachability: Can we reach 0x10000A15C, 0x10000A178, etc from USB?
# ============================================================================
print("\n\n" + "="*80)
print("DFU REACHABILITY: Are Target 2 callers reachable from USB?")
print("="*80)

# Find what calls the functions containing the callers of 0x100002844
for caller_addr in callers_2844:
    func_start = find_func_start(caller_addr)
    if func_start:
        print(f"\n[*] Who calls {func_start:#x} (contains BL to register_handler at {caller_addr:#x})?")
        refs = []
        for ins in all_insns:
            if ins.mnemonic in ('bl', 'b'):
                m = re.match(r'#(0x[0-9a-fA-F]+)', ins.op_str)
                if m and int(m.group(1), 16) == func_start:
                    refs.append((ins.address, ins.mnemonic))
        for addr, mnem in refs:
            in_usb = " *** USB RANGE ***" if 0x100002000 <= addr < 0x100003000 else ""
            print(f"    {mnem.upper()} at {addr:#x}{in_usb}")

print("\n[*] Caller trace complete.")
