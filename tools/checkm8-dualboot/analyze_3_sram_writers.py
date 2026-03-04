#!/usr/bin/env python3
"""
Analyze 3 specific SRAM writer functions in T8020 B1 SecureROM
to determine if values written to SRAM are attacker-controllable.
"""

import struct
import re
from capstone import *

ROM_PATH = r"C:\Users\nathan\Documents\Scripts\OSrust\tools\checkm8-dualboot\securerom\t8020_B1_securerom.bin"
ROM_BASE = 0x100000000
ROM_SIZE = 524288

with open(ROM_PATH, "rb") as f:
    rom_data = f.read()

print(f"[*] Loaded ROM: {len(rom_data)} bytes")

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = False

def va_to_offset(va):
    off = va - ROM_BASE
    if 0 <= off < len(rom_data):
        return off
    return None

def read_u64(va):
    off = va_to_offset(va)
    if off is not None and off + 8 <= len(rom_data):
        return struct.unpack('<Q', rom_data[off:off+8])[0]
    return None

def disasm_at(va, size=4):
    off = va_to_offset(va)
    if off is None:
        return []
    return list(md.disasm(rom_data[off:off+size], va))

def disasm_range(start_va, end_va, label=None):
    off_s = va_to_offset(start_va)
    off_e = va_to_offset(end_va)
    if off_s is None or off_e is None:
        return []
    insns = list(md.disasm(rom_data[off_s:off_e], start_va))
    if label:
        print(f"\n{'='*80}")
        print(f"  {label}")
        print(f"{'='*80}")
    for i in insns:
        print(f"  {i.address:#012x}:  {i.mnemonic:8s} {i.op_str}")
    return insns

def find_func_start(va, max_search=0x400):
    for off in range(0, max_search, 4):
        check_va = va - off
        insns = disasm_at(check_va)
        if insns:
            ins = insns[0]
            if ins.mnemonic == 'stp' and 'x29, x30' in ins.op_str and 'sp' in ins.op_str:
                prev = disasm_at(check_va - 4)
                if prev and prev[0].mnemonic == 'sub' and 'sp' in prev[0].op_str:
                    return check_va - 4
                prev2 = disasm_at(check_va - 4)
                if prev2 and prev2[0].mnemonic == 'stp':
                    prev3 = disasm_at(check_va - 8)
                    if prev3 and prev3[0].mnemonic == 'sub' and 'sp' in prev3[0].op_str:
                        return check_va - 8
                    return check_va - 4
                return check_va
    return None

def find_func_end(va, max_search=0x600):
    for off in range(0, max_search, 4):
        insns = disasm_at(va + off)
        if insns and insns[0].mnemonic == 'ret':
            return va + off + 4
    return va + max_search

def disasm_function(start_va, label=None, max_size=0x600):
    end_va = find_func_end(start_va, max_size)
    return disasm_range(start_va, end_va, label)

def find_adrp_add_pairs(insns):
    results = []
    for i, ins in enumerate(insns):
        if ins.mnemonic == 'adrp':
            m = re.match(r'x(\d+),\s*#(0x[0-9a-fA-F]+)', ins.op_str)
            if not m:
                continue
            reg_num = int(m.group(1))
            page = int(m.group(2), 16)
            for j in range(i+1, min(i+5, len(insns))):
                if insns[j].mnemonic == 'add':
                    ma = re.match(r'x(\d+),\s*x(\d+),\s*#(0x[0-9a-fA-F]+)', insns[j].op_str)
                    if ma and int(ma.group(2)) == reg_num:
                        full = page + int(ma.group(3), 16)
                        results.append((ins.address, insns[j].address, f"x{ma.group(1)}", full))
                        break
    return results

def find_adrp_mem_pairs(insns):
    """Find ADRP followed by LDR/STR [reg, #off] to compute effective address."""
    results = []
    for i, ins in enumerate(insns):
        if ins.mnemonic == 'adrp':
            m = re.match(r'x(\d+),\s*#(0x[0-9a-fA-F]+)', ins.op_str)
            if not m:
                continue
            reg_num = int(m.group(1))
            page = int(m.group(2), 16)
            for j in range(i+1, min(i+5, len(insns))):
                nxt = insns[j]
                if nxt.mnemonic.startswith(('ldr', 'str')):
                    mm = re.search(r'\[x(\d+)(?:,\s*#(0x[0-9a-fA-F]+))?\]', nxt.op_str)
                    if mm and int(mm.group(1)) == reg_num:
                        disp = int(mm.group(2), 16) if mm.group(2) else 0
                        ea = page + disp
                        results.append((ins.address, nxt.address, nxt.mnemonic, nxt.op_str, ea))
    return results

def find_xrefs(target_va):
    results = []
    for ins in md.disasm(rom_data[:ROM_SIZE], ROM_BASE):
        if ins.mnemonic == 'bl':
            m = re.match(r'#(0x[0-9a-fA-F]+)', ins.op_str)
            if m and int(m.group(1), 16) == target_va:
                results.append(ins.address)
    return results


# ============================================================================
# TARGET 1
# ============================================================================
print("\n" + "#"*80)
print("# TARGET 1: SRAM 0x19C008448")
print("# Write instruction at 0x100008B48, claimed writer func 0x100008978 (panic?)")
print("#"*80)

print("\n[*] Func at 0x100008978 (claimed panic) — first 0x80 bytes:")
disasm_range(0x100008978, 0x1000089F8, "func_0x100008978 (partial)")

func1_start = find_func_start(0x100008B48)
print(f"\n[*] Function containing STR at 0x100008B48 starts at: {func1_start:#x}" if func1_start else "\n[!] No prologue found")

if func1_start:
    insns_w1 = disasm_function(func1_start, f"func_{func1_start:#x} — writer to SRAM 0x19C008448")
    
    print(f"\n[*] ADRP+ADD pairs:")
    for a1, a2, reg, addr in find_adrp_add_pairs(insns_w1):
        print(f"  {a1:#x}..{a2:#x}: {reg} = {addr:#x}")
    
    print(f"\n[*] ADRP+MEM (LDR/STR) pairs:")
    for a1, a2, mnem, ops, ea in find_adrp_mem_pairs(insns_w1):
        extra = ""
        if ROM_BASE <= ea < ROM_BASE + ROM_SIZE:
            val = read_u64(ea)
            if val:
                extra = f" [ROM value = {val:#x}]"
        print(f"  {a1:#x}..{a2:#x}: {mnem} {ops} => EA={ea:#x}{extra}")

# BLR users — show context for USB handler
print(f"\n[*] USB handler 0x1000023CC (uses BLR from SRAM 0x19C008448):")
disasm_range(0x1000023CC, 0x100002480, "func_0x1000023CC — partial, through BLR at 0x100002458")


# ============================================================================
# TARGET 2
# ============================================================================
print("\n\n" + "#"*80)
print("# TARGET 2: SRAM 0x19C008B48  *** HIGHEST PRIORITY — USB CODE ***")
print("# Writer: func 0x100002844, store at 0x100002874")
print("# BLR at 0x100002914 loads from [0x19C008B48]")
print("#"*80)

print("\n[*] Writer function 0x100002844:")
insns_w2 = disasm_function(0x100002844, "func_0x100002844 — writer to SRAM 0x19C008B48")

print(f"\n[*] ADRP+ADD pairs:")
for a1, a2, reg, addr in find_adrp_add_pairs(insns_w2):
    print(f"  {a1:#x}..{a2:#x}: {reg} = {addr:#x}")

print(f"\n[*] ADRP+MEM pairs:")
for a1, a2, mnem, ops, ea in find_adrp_mem_pairs(insns_w2):
    extra = ""
    if ROM_BASE <= ea < ROM_BASE + ROM_SIZE:
        val = read_u64(ea)
        if val:
            extra = f" [ROM value = {val:#x}]"
    elif 0x19C000000 <= ea <= 0x19C100000:
        extra = " [SRAM address]"
    print(f"  {a1:#x}..{a2:#x}: {mnem} {ops} => EA={ea:#x}{extra}")

# Trace: what value gets STR'd at 0x100002874?
print(f"\n[*] STR instruction at 0x100002874:")
disasm_range(0x100002874, 0x100002878, None)

# Show entire flow from func entry to the store
print(f"\n[*] Data flow trace: 0x100002844 -> 0x100002878")
print("    (Tracing what gets written — look at what register the STR uses")
print("     and where that register was loaded from)")

# Callers
print(f"\n[*] Who calls 0x100002844?")
callers2 = find_xrefs(0x100002844)
for c in callers2:
    print(f"  BL at {c:#x}")
    # Brief context
    disasm_range(max(c - 16, ROM_BASE), c + 8, f"  caller context {c:#x}")

# BLR consumer
print(f"\n[*] Context around BLR at 0x100002914:")
disasm_range(0x1000028D0, 0x100002940, "BLR at 0x100002914 — context")

func2_blr = find_func_start(0x100002914)
if func2_blr:
    print(f"\n[*] Func containing BLR 0x100002914 starts at {func2_blr:#x}:")
    insns_blr2f = disasm_function(func2_blr, f"func_{func2_blr:#x} — calls [0x19C008B48]")
    
    print(f"\n[*] Callers of {func2_blr:#x}:")
    callers_blr2f = find_xrefs(func2_blr)
    for c in callers_blr2f:
        print(f"  BL at {c:#x}")
        if 0x100002000 <= c < 0x100003000:
            print(f"    *** IN USB RANGE ***")


# ============================================================================
# TARGET 3
# ============================================================================
print("\n\n" + "#"*80)
print("# TARGET 3: SRAM 0x19C010620")
print("# Writer: func 0x10000ADDC, BLR at 0x10000ADA8")
print("#"*80)

print("\n[*] Writer function 0x10000ADDC:")
insns_w3 = disasm_function(0x10000ADDC, "func_0x10000ADDC — writer near SRAM 0x19C010620")

print(f"\n[*] ADRP+ADD pairs:")
for a1, a2, reg, addr in find_adrp_add_pairs(insns_w3):
    print(f"  {a1:#x}..{a2:#x}: {reg} = {addr:#x}")

print(f"\n[*] ADRP+MEM pairs:")
for a1, a2, mnem, ops, ea in find_adrp_mem_pairs(insns_w3):
    extra = ""
    if ROM_BASE <= ea < ROM_BASE + ROM_SIZE:
        val = read_u64(ea)
        if val:
            extra = f" [ROM value = {val:#x}]"
    elif 0x19C000000 <= ea <= 0x19C100000:
        extra = " [SRAM address]"
    print(f"  {a1:#x}..{a2:#x}: {mnem} {ops} => EA={ea:#x}{extra}")

# Callers
print(f"\n[*] Callers of 0x10000ADDC:")
callers3 = find_xrefs(0x10000ADDC)
for c in callers3:
    print(f"  BL at {c:#x}")

# BLR context
print(f"\n[*] Context around BLR at 0x10000ADA8:")
disasm_range(0x10000AD70, 0x10000ADE0, "BLR 0x10000ADA8 context")

func3_blr = find_func_start(0x10000ADA8)
if func3_blr:
    print(f"\n[*] Func containing BLR 0x10000ADA8 starts at {func3_blr:#x}:")
    insns_blr3f = disasm_function(func3_blr, f"func_{func3_blr:#x} — calls [0x19C010620]")
    
    print(f"\n[*] Callers of {func3_blr:#x}:")
    callers_blr3f = find_xrefs(func3_blr)
    for c in callers_blr3f:
        print(f"  BL at {c:#x}")
        if 0x100002000 <= c < 0x100003000:
            print(f"    *** IN USB RANGE ***")

# ============================================================================
# Check reachability from DFU
# ============================================================================
print("\n\n" + "#"*80)
print("# DFU/USB REACHABILITY ANALYSIS")
print("#"*80)

# Functions called from USB range
print("\n[*] Functions called FROM USB range (0x100002000-0x100003000):")
usb_code = rom_data[va_to_offset(0x100002000):va_to_offset(0x100003000)]
usb_call_targets = set()
for ins in md.disasm(usb_code, 0x100002000):
    if ins.mnemonic == 'bl':
        m = re.match(r'#(0x[0-9a-fA-F]+)', ins.op_str)
        if m:
            usb_call_targets.add(int(m.group(1), 16))

for t in sorted(usb_call_targets):
    marker = ""
    if t == 0x10000ADDC:
        marker = " *** TARGET 3 WRITER ***"
    elif func1_start and t == func1_start:
        marker = " *** TARGET 1 WRITER ***"
    elif t == 0x100002844:
        marker = " *** TARGET 2 WRITER (self-USB) ***"
    print(f"  {t:#x}{marker}")

print("\n[*] Analysis complete.")
