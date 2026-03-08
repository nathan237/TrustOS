#!/usr/bin/env python3
"""Final deep-dive: signature verifier + CPFM usage + complete decision tree"""
from capstone import *

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

with open('securerom/t8020_B1_securerom.bin', 'rb') as f:
    rom = f.read()
base = 0x100000000

# === 1. FULL DISASSEMBLY OF img4_verify (0xA704) ===
print("=== FULL img4_verify (0x10000A704) ===")
off = 0xA704
for inst in md.disasm(rom[off:off+400], base+off):
    print(f"  {inst.address:#010x}: {inst.mnemonic:8s} {inst.op_str}")
    if inst.mnemonic == 'ret':
        break

# === 2. FULL DISASSEMBLY OF crypto_verify (0x5480) — THE CORE ===
print("\n\n=== FULL crypto_verify (0x100005480) ===")
off = 0x5480
count = 0
for inst in md.disasm(rom[off:off+1200], base+off):
    print(f"  {inst.address:#010x}: {inst.mnemonic:8s} {inst.op_str}")
    if inst.mnemonic == 'ret':
        count += 1
        if count >= 1:  # Stop after first ret
            break

# === 3. What is 0x10001C094? (adr; nop; ret — called from crypto verify) ===
print("\n\n=== MYSTERY FUNCTION at 0x10001C094 ===")
off = 0x1C094
for inst in md.disasm(rom[off:off+20], base+off, count=5):
    print(f"  {inst.address:#010x}: {inst.mnemonic:8s} {inst.op_str}")

# === 4. What is 0x100007464? (cbz as first instr — conditional path) ===
print("\n\n=== CONDITIONAL FUNCTION at 0x100007464 ===")
off = 0x7464
for inst in md.disasm(rom[off:off+100], base+off):
    print(f"  {inst.address:#010x}: {inst.mnemonic:8s} {inst.op_str}")
    if inst.mnemonic == 'ret':
        break

# === 5. What is 0x1000073AC? (cbz as first instr — another conditional) ===
print("\n\n=== CONDITIONAL FUNCTION at 0x1000073AC ===")
off = 0x73AC
for inst in md.disasm(rom[off:off+100], base+off):
    print(f"  {inst.address:#010x}: {inst.mnemonic:8s} {inst.op_str}")
    if inst.mnemonic == 'ret':
        break

# === 6. THE DECISION AT 0x1BC8 — complete context ===
print("\n\n=== THE CRITICAL DECISION: 0x1BC8 with FULL CONTEXT ===")
off = 0x1B90
for inst in md.disasm(rom[off:off+200], base+off):
    marker = "===>>>" if inst.address == base + 0x1BC8 else "      "
    print(f"  {marker} {inst.address:#010x}: {inst.mnemonic:8s} {inst.op_str}")
    if inst.address > base + 0x1C60:
        break

# === 7. Find ALL places that branch into the "success" path (0x1C5C) ===
print("\n\n=== WHO ELSE CAN REACH 0x1C5C (success path)? ===")
target = base + 0x1C5C
insns = list(md.disasm(rom[0x400:0x25000], base+0x400))
for inst in insns:
    if inst.mnemonic in ('b', 'cbz', 'cbnz', 'tbz', 'tbnz', 'b.eq', 'b.ne',
                          'b.lo', 'b.hi', 'b.le', 'b.ge'):
        if f'#0x{target:x}' in inst.op_str or f'#{target:#x}' in inst.op_str:
            print(f"  {inst.address:#010x}: {inst.mnemonic} {inst.op_str}")

# === 8. Find ALL places that read SRAM global variables ===
# Specifically looking for globals that influence the boot decision
print("\n\n=== SRAM GLOBALS USED IN BOOT DECISION PATH ===")
# The load_and_verify at 0x1A88 uses SRAM globals
# Let's see what ADRP references are in that function
off = 0x1A88
for inst in md.disasm(rom[off:off+700], base+off):
    if inst.mnemonic == 'adrp' and '0x19c' in inst.op_str:
        print(f"  {inst.address:#010x}: {inst.mnemonic:8s} {inst.op_str}")
    elif inst.mnemonic in ('ldrb', 'ldr', 'ldrh') and ('x19' in inst.op_str or 'x20' in inst.op_str):
        # Loads from saved base regs often access globals
        pass
    if inst.mnemonic == 'ret':
        break

# === 9. THE 0x200000001..4 and 0x300000002..807 VALUES — what are they? ===
print("\n\n=== CHECKING 0x200000xxx / 0x300000xxx VALUES ===")
# These are likely IMG4_TAG values (4CC codes)
# Let's check where they appear and if they're used as tags
for target_val in [0x200000001, 0x200000002, 0x200000004, 0x300000002, 0x300000003, 0x300000807, 0x2FE000002]:
    for off in range(0, len(rom), 8):
        val = int.from_bytes(rom[off:off+8], 'little')
        if val == target_val:
            # Show context around this value
            ctx_before = rom[off-8:off]
            ctx_after = rom[off+8:off+16]
            before_val = int.from_bytes(ctx_before, 'little') if len(ctx_before) == 8 else 0
            after_val = int.from_bytes(ctx_after, 'little') if len(ctx_after) == 8 else 0
            print(f"  {target_val:#014x} at ROM offset {off:#010x}:")
            print(f"    before: {before_val:#018x}")
            print(f"    value:  {target_val:#018x}")
            print(f"    after:  {after_val:#018x}")
            # Are surrounding values also in this range? (suggests a data table)
            if 0x100000000 <= before_val <= 0x400000000 or 0x100000000 <= after_val <= 0x400000000:
                print(f"    → LOOKS LIKE A TABLE OF ADDRESSES")
            else:
                print(f"    → ISOLATED VALUE (likely a constant)")
