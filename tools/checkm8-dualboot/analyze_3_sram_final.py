#!/usr/bin/env python3
"""
Final pass: Fix ROM scanning (chunk-based) and find the true writer of 0x19C010620.
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
md.skipdata = True  # Skip invalid data instead of stopping

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
        if i.mnemonic != '.byte':  # skip data
            print(f"  {i.address:#012x}:  {i.mnemonic:8s} {i.op_str}")
    return insns

# ============================================================================
# Scan entire ROM with skipdata enabled
# ============================================================================
print("[*] Full ROM scan with skipdata enabled...")
all_insns = list(md.disasm(rom_data[:ROM_SIZE], ROM_BASE))
real_insns = [i for i in all_insns if i.mnemonic != '.byte']
print(f"  Total items: {len(all_insns)}, Real instructions: {len(real_insns)}")

# ============================================================================
# 1. Find ALL BL/B xrefs to target functions (proper scan)
# ============================================================================
print("\n" + "="*80)
print("PROPER XREF SCAN")
print("="*80)

target_funcs = [0x100008AF0, 0x100002844, 0x10000ADDC, 0x1000028C0, 0x10000AD80]
target_names = {
    0x100008AF0: "T1 writer",
    0x100002844: "T2 writer (USB)",
    0x10000ADDC: "T3 writer",
    0x1000028C0: "T2 BLR consumer",
    0x10000AD80: "T3 BLR consumer",
}

for tgt in target_funcs:
    refs = []
    for ins in real_insns:
        if ins.mnemonic in ('bl', 'b'):
            m = re.match(r'#(0x[0-9a-fA-F]+)', ins.op_str)
            if m and int(m.group(1), 16) == tgt:
                refs.append((ins.address, ins.mnemonic))
    name = target_names.get(tgt, "")
    if refs:
        print(f"\n  {tgt:#x} ({name}): {len(refs)} refs")
        for addr, mnem in refs:
            print(f"    {mnem.upper()} at {addr:#x}")
    else:
        print(f"\n  {tgt:#x} ({name}): No BL/B refs")

# ============================================================================
# 2. Find ALL STR instructions that write to offset 0x620 from a register
#    that could be 0x19C010000 (via ADRP)
# ============================================================================
print("\n\n" + "="*80)
print("FINDING TRUE WRITER OF SRAM 0x19C010620")
print("="*80)

# Strategy: Find all ADRP 0x19C010000 and check for STR with offset 0x620
print("\n[*] All ADRP references to page 0x19C010000:")
for i, ins in enumerate(real_insns):
    if ins.mnemonic == 'adrp':
        m = re.match(r'x(\d+),\s*#(0x[0-9a-fA-F]+)', ins.op_str)
        if m and int(m.group(2), 16) == 0x19C010000:
            reg = int(m.group(1))
            # Check next 15 instructions for STR/LDR with offset 0x620
            for j in range(i+1, min(i+15, len(real_insns))):
                nxt = real_insns[j]
                if nxt.mnemonic.startswith('str'):
                    mm = re.search(r'\[x(\d+)(?:,\s*#(0x[0-9a-fA-F]+))?\]', nxt.op_str)
                    if mm:
                        base = int(mm.group(1))
                        disp = int(mm.group(2), 16) if mm.group(2) else 0
                        if base == reg and disp == 0x620:
                            print(f"  *** STR to [x{base}, #0x620] at {nxt.address:#x} (ADRP at {ins.address:#x})")
                            disasm_range(ins.address - 32, nxt.address + 32,
                                        f"WRITER of SRAM 0x19C010620 at {nxt.address:#x}")
                if nxt.mnemonic.startswith('ldr'):
                    mm = re.search(r'\[x(\d+)(?:,\s*#(0x[0-9a-fA-F]+))?\]', nxt.op_str)
                    if mm:
                        base = int(mm.group(1))
                        disp = int(mm.group(2), 16) if mm.group(2) else 0
                        if base == reg and disp == 0x620:
                            print(f"  LDR from [x{base}, #0x620] at {nxt.address:#x} (ADRP at {ins.address:#x})")

# Also check ADD to construct full address
print("\n[*] All ADRP+ADD constructing 0x19C010620:")
for i, ins in enumerate(real_insns):
    if ins.mnemonic == 'adrp':
        m = re.match(r'x(\d+),\s*#(0x[0-9a-fA-F]+)', ins.op_str)
        if m and int(m.group(2), 16) == 0x19C010000:
            reg = int(m.group(1))
            for j in range(i+1, min(i+5, len(real_insns))):
                nxt = real_insns[j]
                if nxt.mnemonic == 'add':
                    ma = re.match(r'x(\d+),\s*x(\d+),\s*#(0x[0-9a-fA-F]+)', nxt.op_str)
                    if ma and int(ma.group(2)) == reg and int(ma.group(3), 16) == 0x620:
                        print(f"  ADRP+ADD => 0x19C010620 at {ins.address:#x}..{nxt.address:#x}")
                        disasm_range(ins.address - 16, nxt.address + 48,
                                    f"Reference to 0x19C010620")

# ============================================================================
# 3. Find callers of 0x100002844 (Target 2 writer) — proper scan
# ============================================================================
print("\n\n" + "="*80)
print("FINDING CALLERS OF 0x100002844 (Target 2)")
print("="*80)

# Check ADRP+ADD constructing 0x100002844
for i, ins in enumerate(real_insns):
    if ins.mnemonic == 'adrp':
        m = re.match(r'x(\d+),\s*#(0x[0-9a-fA-F]+)', ins.op_str)
        if m and int(m.group(2), 16) == 0x100002000:
            reg = int(m.group(1))
            for j in range(i+1, min(i+5, len(real_insns))):
                nxt = real_insns[j]
                if nxt.mnemonic == 'add':
                    ma = re.match(r'x(\d+),\s*x(\d+),\s*#(0x[0-9a-fA-F]+)', nxt.op_str)
                    if ma and int(ma.group(2)) == reg and int(ma.group(3), 16) == 0x844:
                        print(f"  ADRP+ADD => 0x100002844 at {ins.address:#x}..{nxt.address:#x}")
                        disasm_range(ins.address - 16, nxt.address + 64,
                                    f"Caller constructing 0x100002844")

# Check ADRP+ADD constructing 0x1000028C0
print("\n[*] ADRP+ADD constructing 0x1000028C0 (interrupt dispatch):")
for i, ins in enumerate(real_insns):
    if ins.mnemonic == 'adrp':
        m = re.match(r'x(\d+),\s*#(0x[0-9a-fA-F]+)', ins.op_str)
        if m and int(m.group(2), 16) == 0x100002000:
            reg = int(m.group(1))
            for j in range(i+1, min(i+5, len(real_insns))):
                nxt = real_insns[j]
                if nxt.mnemonic == 'add':
                    ma = re.match(r'x(\d+),\s*x(\d+),\s*#(0x[0-9a-fA-F]+)', nxt.op_str)
                    if ma and int(ma.group(2)) == reg and int(ma.group(3), 16) == 0x8C0:
                        print(f"  ADRP+ADD => 0x1000028C0 at {ins.address:#x}..{nxt.address:#x}")
                        disasm_range(ins.address - 16, nxt.address + 64,
                                    f"Caller constructing 0x1000028C0")

# ============================================================================
# 4. Check what 0x100008B58 does (called from USB, might call 0x100008AF0)
# ============================================================================
print("\n\n" + "="*80)
print("func_0x100008B58 — called from USB, might be related to Target 1")
print("="*80)

# Check if func 0x100008B58 calls func 0x100008AF0
print("[*] Does 0x100008B58 call 0x100008AF0?")
insns_8b58 = list(md.disasm(rom_data[va_to_offset(0x100008B58):va_to_offset(0x100008C60)], 0x100008B58))
for ins in insns_8b58:
    if ins.mnemonic in ('bl', 'b'):
        m = re.match(r'#(0x[0-9a-fA-F]+)', ins.op_str)
        if m:
            target = int(m.group(1), 16)
            if target == 0x100008AF0:
                print(f"  YES: {ins.mnemonic.upper()} 0x100008AF0 at {ins.address:#x}")

# ============================================================================
# 5. Find ALL xrefs to 0x100008AF0 (Target 1 writer)
# ============================================================================
print("\n\n" + "="*80)
print("ALL XREFS TO 0x100008AF0 (Target 1 writer)")
print("="*80)

for ins in real_insns:
    if ins.mnemonic in ('bl', 'b'):
        m = re.match(r'#(0x[0-9a-fA-F]+)', ins.op_str)
        if m and int(m.group(1), 16) == 0x100008AF0:
            print(f"  {ins.mnemonic.upper()} at {ins.address:#x}")
            disasm_range(ins.address - 8, ins.address + 8, f"ref to 0x100008AF0")

# ============================================================================
# 6. Check the BLR at 0x100002458 more carefully — what's in x27+8?
# ============================================================================
print("\n\n" + "="*80)
print("DETAILED ANALYSIS: BLR x8 at 0x100002458")
print("="*80)

print("""
From the USB handler at 0x1000023CC:
  0x100002420: adrp x8, #0x19c008000    ; (NOP follows, meaning ADRP was relaxed)
  0x100002424: nop
  0x100002428: ldr  x8, [x8, #0x448]    ; x8 = [0x19C008448]  -- this is the CANARY
  0x10000242c: str  x8, [sp, #8]        ; save canary to stack
  0x100002430: mov  w0, #0x57
  0x100002434: mov  w1, #1
  0x100002438: bl   #0x100008370        ; some_func(0x57, 1)
  0x10000243c: mov  w0, #0x300
  0x100002440: mov  w1, #0x70000
  0x100002444: mov  x2, sp
  0x100002448: bl   #0x1000113b4        ; heap_alloc/get_buffer(0x300, 0x70000, &ptr)
  0x10000244c: ldr  x27, [sp]           ; x27 = allocated buffer/object
  0x100002450: ldr  x8, [x27, #8]       ; x8 = object->vtable[1] or object->method1
  0x100002454: mov  w0, #7
  0x100002458: blr  x8                  ; call object->func(7)

The BLR here loads from [x27+8], where x27 is a pointer returned by
func_0x1000113b4. This is NOT loading from SRAM 0x19C008448 directly.

0x19C008448 is loaded at 0x100002428 and saved as a STACK CANARY (sp+8).
It's checked later to detect stack corruption.

So the BLR at 0x100002458 is calling through an OBJECT POINTER
returned by an allocator, not through the SRAM cell 0x19C008448.
""")

# Now let's understand func 0x1000113B4 to know what object is returned
print("[*] func_0x1000113B4 (called before the BLR):")
disasm_range(0x1000113B4, 0x100011440, "func_0x1000113B4 — returns object/buffer")

# ============================================================================
# 7. Check if SRAM 0x19C008B48 region overlaps with DFU heap
# ============================================================================
print("\n\n" + "="*80)
print("SRAM LAYOUT CHECK: Where is the DFU heap vs interrupt table?")
print("="*80)

print("""
Key SRAM regions identified:
  0x19C008448     — Stack canary / cookie
  0x19C008B48     — Interrupt callback table (24-byte entries, up to 512)
                    Table spans: 0x19C008B48 to 0x19C008B48 + 512*24 = 0x19C00BB48
  0x19C00BD90     — Debug/tracking buffer (8 bytes)
  0x19C00BDA0     — Panic counter
  0x19C010620     — Unknown callback pointer
  0x19C010628     — Cached block device frequency
  0x19C011408     — Panic handler pointer
  0x19C011410     — Panic string pointer

The interrupt callback table at 0x19C008B48 spans ~12KB (0x3000 bytes).
If the DFU heap is nearby and can overflow into this region, the callbacks
could be corrupted to achieve code execution.
""")

# ============================================================================
# 8. Check what the NOP after ADRP means — is it an ADRP relaxation?
# ============================================================================
print("\n[*] Checking ADRP+NOP patterns (linker relaxation):")
# In some ARM64 toolchains, ADRP+ADD is relaxed to ADRP+NOP when the target
# is within the same page. In Apple's SecureROM, NOP after ADRP typically
# means the ADRP was enough (the offset is 0, so ADD #0 became NOP).

# Let's check all ADRP+NOP in the USB function
usb_insns = list(md.disasm(rom_data[va_to_offset(0x1000023CC):va_to_offset(0x100002600)], 0x1000023CC))
for i, ins in enumerate(usb_insns):
    if ins.mnemonic == 'adrp' and i+1 < len(usb_insns) and usb_insns[i+1].mnemonic == 'nop':
        m = re.match(r'x(\d+),\s*#(0x[0-9a-fA-F]+)', ins.op_str)
        if m:
            page = int(m.group(2), 16)
            reg = m.group(1)
            print(f"  {ins.address:#x}: ADRP x{reg}, #{page:#x} + NOP => x{reg} = {page:#x}")

# Also check ADR (which loads a full address in one instruction)
print("\n[*] ADR instructions in USB handler:")
for ins in usb_insns:
    if ins.mnemonic == 'adr':
        m = re.match(r'x(\d+),\s*#(0x[0-9a-fA-F]+)', ins.op_str)
        if m:
            addr = int(m.group(2), 16)
            print(f"  {ins.address:#x}: ADR x{m.group(1)}, #{addr:#x}")

print("\n[*] Final pass analysis complete.")
