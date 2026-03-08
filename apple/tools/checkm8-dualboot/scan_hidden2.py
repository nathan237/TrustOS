#!/usr/bin/env python3
"""Deep dive into mysterious peripherals and code patterns"""
from capstone import *
from collections import defaultdict

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

with open('securerom/t8020_B1_securerom.bin', 'rb') as f:
    rom = f.read()
base = 0x100000000

# === 1. FIND CODE THAT ACCESSES MYSTERIOUS PERIPHERALS ===
# Focus on: 0x2FE000002, 0x239100000, 0x239900000, 0x23E000000/400000, 0x242000000/400000, 0x200000xxx, 0x300000xxx

# First, find which ROM offsets reference these addresses (literal pool entries)
mysterious = {
    0x2FE000002: "MYSTERY_2FE",
    0x239100000: "MYSTERY_239_1",
    0x239900000: "MYSTERY_239_9",
    0x23E000000: "MYSTERY_23E_0",
    0x23E400000: "MYSTERY_23E_4",
    0x242000000: "MYSTERY_242_0",
    0x242400000: "MYSTERY_242_4",
    0x200000001: "MYSTERY_200_1",
    0x200000002: "MYSTERY_200_2",
    0x200000004: "MYSTERY_200_4",
    0x300000002: "MYSTERY_300_2",
    0x300000003: "MYSTERY_300_3",
    0x300000807: "MYSTERY_300_807",
    0x235004000: "MYSTERY_235_04",
}

print("=== TRACING CODE THAT ACCESSES MYSTERIOUS PERIPHERALS ===\n")

for target_val, name in sorted(mysterious.items()):
    # Find literal pool entries for this value
    lit_offsets = []
    for off in range(0, len(rom), 8):
        val = int.from_bytes(rom[off:off+8], 'little')
        if val == target_val:
            lit_offsets.append(off)
    
    if not lit_offsets:
        continue
    
    print(f"--- {name} ({target_val:#014x}) --- found at {len(lit_offsets)} literal pool location(s)")
    
    for lit_off in lit_offsets:
        lit_addr = base + lit_off
        
        # Find any LDR instruction that references this literal pool entry
        # LDR Xn, [PC, #imm] — search nearby code
        # The literal pool is usually near the code that uses it
        # Search backwards for the function that loads this address
        # LDR literal: encoding = 0x58000000 | (imm19 << 5) | Rt
        
        # Search code regions for LDR instructions pointing to this literal
        for code_off in range(0x400, min(0x25000, len(rom)), 4):
            word = int.from_bytes(rom[code_off:code_off+4], 'little')
            # LDR (literal) 64-bit: 0101 1000 xxxx xxxx xxxx xxxx xxxR RRRR
            if (word & 0xFF000000) == 0x58000000:
                rt = word & 0x1F
                imm19 = (word >> 5) & 0x7FFFF
                # Sign extend
                if imm19 & 0x40000:
                    imm19 |= ~0x7FFFF
                    imm19 = imm19 & 0xFFFFFFFFFFFFFFFF  # mask to 64-bit
                    imm19 = -(0x10000000000000000 - imm19) if imm19 > 0x7FFFFFFFFFFFFFFF else imm19
                target_pc = (base + code_off) + (imm19 * 4)
                if target_pc == lit_addr:
                    code_addr = base + code_off
                    # Found it! Now show context (16 instructions around it)
                    ctx_start = max(0x400, code_off - 48)
                    ctx_end = min(len(rom), code_off + 80)
                    print(f"\n  LDR x{rt}, [{lit_addr:#x}] at code addr {code_addr:#010x}:")
                    for inst in md.disasm(rom[ctx_start:ctx_end], base + ctx_start):
                        marker = " >>>" if inst.address == code_addr else "    "
                        print(f"  {marker} {inst.address:#010x}: {inst.mnemonic:8s} {inst.op_str}")
                    print()
    print()

# === 2. LOOK FOR "FUSE READ" PATTERNS ===
# Apple often reads fuses to decide behavior
# Fuse reads typically: load MMIO addr, read 32-bit, mask bits, branch
print("\n=== CONDITIONAL BRANCHES AFTER MMIO READS ===")
print("(patterns: LDR from MMIO → AND/TST → CBZ/CBNZ/TBZ/TBNZ)")

# Find all LDR from MMIO followed by conditional branches
insns = list(md.disasm(rom[0x400:0x25000], base+0x400))
for idx in range(len(insns)-5):
    i = insns[idx]
    # Look for: STR/LDR to [Xn] where Xn was loaded with MMIO address
    # Simpler: look for TST/TBNZ/TBZ patterns that decide paths
    if i.mnemonic in ('tbnz', 'tbz'):
        # These test a specific bit — often used for fuse/flag checks
        # Get context
        if True:
            ctx_start = max(0, idx-4)
            ctx_end = min(len(insns), idx+2)
            has_ldr = False
            for j in range(ctx_start, ctx_end):
                if insns[j].mnemonic in ('ldr', 'ldrsw'):
                    has_ldr = True
            if has_ldr:
                print(f"\n  BIT TEST at {i.address:#010x}: {i.mnemonic} {i.op_str}")
                for j in range(ctx_start, ctx_end):
                    marker = ">>>" if j == idx else "   "
                    print(f"    {marker} {insns[j].address:#010x}: {insns[j].mnemonic:8s} {insns[j].op_str}")

# === 3. LOOK FOR "WRITE MAGIC VALUE THEN READ BACK" PATTERNS ===
print("\n\n=== WRITE-THEN-READ / MAGIC UNLOCK PATTERNS ===")
for idx in range(len(insns) - 3):
    # STR Xn, [Xm] followed by LDR Xp, [Xm] (write then read same addr)
    if insns[idx].mnemonic == 'str' and insns[idx+1].mnemonic in ('ldr', 'dmb', 'dsb', 'isb'):
        i0 = insns[idx]
        # Check if next non-barrier instruction is LDR from same base
        str_ops = i0.op_str.split(',')
        if len(str_ops) >= 2:
            str_base = str_ops[1].strip()
            for j in range(idx+1, min(idx+4, len(insns))):
                if insns[j].mnemonic == 'ldr':
                    ldr_ops = insns[j].op_str.split(',')
                    if len(ldr_ops) >= 2:
                        ldr_base = ldr_ops[1].strip()
                        # Same base register?  
                        if str_base == ldr_base:
                            # Show context
                            print(f"\n  WRITE→READ at {i0.address:#010x}:")
                            ctx_s = max(0, idx-3)
                            ctx_e = min(len(insns), j+3)
                            for k in range(ctx_s, ctx_e):
                                marker = ">>>" if k in (idx, j) else "   "
                                print(f"    {marker} {insns[k].address:#010x}: {insns[k].mnemonic:8s} {insns[k].op_str}")
                            break

# === 4. LOOK FOR INTERESTING FUNCTION AT 0x23D2B8030 (mystery peripheral) ===
print("\n\n=== 0x23D2B8030 PERIPHERAL (accessed from 2 places in ROM) ===")
# This was at literal pool 0x100006710 and 0x100007a48
# Find code near these literals
for target_lit in [0x6710, 0x7A48]:
    print(f"\n  Code near literal at {base+target_lit:#010x}:")
    # Search back for the function
    ctx_start = max(0x400, target_lit - 200)
    for inst in md.disasm(rom[ctx_start:target_lit+8], base+ctx_start):
        if (inst.address - base) >= target_lit - 100:
            print(f"    {inst.address:#010x}: {inst.mnemonic:8s} {inst.op_str}")

# === 5. SCAN FOR DFT/JTAG/SWD STRINGS ===
print("\n\n=== STRINGS SCAN (debug/test/fuse related) ===")
import re
for match in re.finditer(rb'[\x20-\x7E]{4,}', rom):
    s = match.group().decode('ascii')
    sl = s.lower()
    if any(k in sl for k in ['debug', 'test', 'jtag', 'swd', 'fuse', 'dft', 'produc', 'demot', 
                               'secur', 'unlock', 'enable', 'disable', 'bypass', 'factory',
                               'serial', 'uart', 'probe', 'scan', 'boundary', 'bist',
                               'lifetime', 'board_id', 'ecid', 'chipid', 'chip_id',
                               'demote', 'promote', 'manufacturing']):
        addr = base + match.start()
        print(f"  {addr:#010x}: \"{s}\"")
