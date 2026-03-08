#!/usr/bin/env python3
"""Scan SecureROM for hidden functionality in unlikely places"""
from capstone import *
from collections import defaultdict, Counter

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

with open('securerom/t8020_B1_securerom.bin', 'rb') as f:
    rom = f.read()
base = 0x100000000

# === 1. ALL MMIO ADDRESSES IN LITERAL POOLS ===
print("=== ALL MMIO PERIPHERAL REGIONS ===")
mmio = set()
for off in range(0, len(rom), 8):
    val = int.from_bytes(rom[off:off+8], 'little')
    if 0x200000000 <= val <= 0x400000000:
        mmio.add(val)

periph = defaultdict(list)
for addr in sorted(mmio):
    periph_base = addr & 0xFFFFFF0000
    periph[periph_base].append(addr)

print(f"Total unique MMIO addresses: {len(mmio)}")
print(f"Peripheral regions: {len(periph)}")
for pb in sorted(periph.keys()):
    addrs = sorted(periph[pb])
    name = 'UNKNOWN'
    if 0x235100000 <= pb <= 0x2351FFFFF: name = 'AES ENGINE'
    elif 0x23D240000 <= pb <= 0x23D24FFFF: name = 'USB/DWC3'
    elif 0x23B0B0000 <= pb <= 0x23B0BFFFF: name = 'PMGR/BOOT'
    elif 0x23D280000 <= pb <= 0x23D28FFFF: name = 'USB-PHY'
    elif 0x23B100000 <= pb <= 0x23B1FFFFF: name = 'GPIO'
    elif 0x23A000000 <= pb <= 0x23AFFFFFF: name = 'DART/IOMMU'
    elif 0x20E000000 <= pb <= 0x20EFFFFFF: name = 'CHIPID/FUSES'
    
    print(f"  {pb:#014x} [{name:15s}] {len(addrs)} regs: ", end="")
    if len(addrs) <= 5:
        print(', '.join(f"+{(a-pb):#06x}" for a in addrs))
    else:
        print(f"+{(addrs[0]-pb):#06x} .. +{(addrs[-1]-pb):#06x} ({len(addrs)} regs)")

# === 2. HINT instructions ===
print()
print("=== HINT INSTRUCTIONS (unusual opcodes) ===")
hints = []
for i in md.disasm(rom[0x400:0x20000], base+0x400):
    if i.mnemonic == 'hint':
        hints.append((i.address, i.op_str))

hint_types = Counter()
for addr, op in hints:
    hint_types[op] += 1
for op, cnt in hint_types.most_common():
    print(f"  hint {op}: x{cnt}")

# === 3. MAGIC TAGS (4-byte ASCII constants used as identifiers) ===
print()
print("=== MAGIC TAGS / CONSTANTS IN CODE ===")
# Find MOV immediate values that look like ASCII tags
tags_found = set()
for i in md.disasm(rom[0x400:0x20000], base+0x400):
    if i.mnemonic in ('movz', 'mov') and '#0x' in i.op_str:
        parts = i.op_str.split('#0x')
        if len(parts) >= 2:
            try:
                val_str = parts[1].split(',')[0].strip()
                val = int(val_str, 16)
                if 0x1000 <= val <= 0xFFFFFFFF:
                    # Try as 4-char ASCII tag
                    b = val.to_bytes(4, 'big')
                    if all(0x20 <= c <= 0x7e for c in b if c != 0):
                        ascii_str = b.decode('ascii', errors='replace')
                        if ascii_str.strip('\x00') and len(ascii_str.strip('\x00')) >= 2:
                            key = (val, ascii_str)
                            if key not in tags_found:
                                tags_found.add(key)
                                print(f"  {i.address:#010x}: {i.mnemonic} {i.op_str:40s} = '{ascii_str}'")
            except:
                pass

# === 4. Functions that are NEVER called (dead code?) ===
print()
print("=== UNREACHABLE / RARELY-CALLED FUNCTIONS ===")
# Find all BL targets
bl_targets = Counter()
br_targets = set()
for i in md.disasm(rom[0x400:0x20000], base+0x400):
    if i.mnemonic == 'bl' and '#0x' in i.op_str:
        target = int(i.op_str.split('#')[1], 16)
        bl_targets[target] += 1
    elif i.mnemonic in ('br', 'blr'):
        br_targets.add(i.address)

# Find function starts (after ret or udf or at known bl targets)
func_starts = set(bl_targets.keys())

# Find functions with very few callers
for func, count in sorted(bl_targets.items()):
    if count == 1 and 0x100000400 <= func <= 0x100020000:
        # Show what this function does (first 4 instructions)
        off = func - base
        if off < len(rom) - 16:
            insns = list(md.disasm(rom[off:off+32], func, count=6))
            if insns and insns[0].mnemonic not in ('udf', '.byte'):
                # Check if this looks interesting
                instrs = ' → '.join(f"{x.mnemonic}" for x in insns[:4])
                # Only show non-trivial single-caller functions
                if any(x.mnemonic in ('msr', 'str') for x in insns[:6]):
                    print(f"  {func:#010x} (1 caller): {instrs}")

# === 5. EXCEPTION HANDLERS — what's at hint #0x45 vectors? ===
print()
print("=== EXCEPTION VECTOR ANALYSIS (VBAR @ 0x800) ===")
vec_names = ['SP0_SYNC','SP0_IRQ','SP0_FIQ','SP0_SERR',
             'SPx_SYNC','SPx_IRQ','SPx_FIQ','SPx_SERR',
             'L64_SYNC','L64_IRQ','L64_FIQ','L64_SERR',
             'L32_SYNC','L32_IRQ','L32_FIQ','L32_SERR']

for idx in range(16):
    vec_off = 0x800 + (idx * 0x80)
    insns = list(md.disasm(rom[vec_off:vec_off+0x80], base+vec_off, count=20))
    if insns:
        first = insns[0]
        # Count real instructions (not padding)
        real_insns = [x for x in insns if x.mnemonic != 'udf' and x.mnemonic != 'hint']
        handler_type = "ACTIVE" if len(real_insns) > 2 else "DEAD/TRAP"
        if first.mnemonic == 'hint':
            handler_type = "BTYPE_TRAP"
        print(f"  [{vec_names[idx]:10s}] {handler_type:12s} | {first.mnemonic} {first.op_str}")
        if handler_type == "ACTIVE":
            for inst in insns[:8]:
                print(f"    {inst.address:#010x}: {inst.mnemonic:8s} {inst.op_str}")

# === 6. SCAN FOR WRITES TO STANDARD REGS WITH UNUSUAL VALUES ===
print()
print("=== UNUSUAL SCTLR/TCR/TTBR WRITES ===")
# Already found at 0x430/0x44C, but what VALUE is written?
for i in md.disasm(rom[0x400:0x600], base+0x400):
    if i.mnemonic == 'msr':
        print(f"  {i.address:#010x}: {i.mnemonic} {i.op_str}")
    elif i.mnemonic == 'orr' and 'x0' in i.op_str:
        print(f"  {i.address:#010x}: {i.mnemonic} {i.op_str}")
    elif i.mnemonic == 'and' and 'x0' in i.op_str:
        print(f"  {i.address:#010x}: {i.mnemonic} {i.op_str}")
    elif i.mnemonic == 'bic':
        print(f"  {i.address:#010x}: {i.mnemonic} {i.op_str}")

# === 7. LOOK FOR DEBUG REGISTERS (MDSCR, OSLAR, DBGBCR, DBGWCR) ===
print()
print("=== DEBUG REGISTER ACCESSES ===")
debug_regs = []
for off in range(0x400, min(0x20000, len(rom)), 4):
    word = int.from_bytes(rom[off:off+4], 'little')
    if (word & 0xFFF00000) in (0xD5100000, 0xD5300000):
        crn = (word >> 12) & 0xF
        op0 = 2 + ((word >> 19) & 1)
        op1 = (word >> 16) & 0x7
        crm = (word >> 8) & 0xF
        op2 = (word >> 5) & 0x7
        # Debug registers are in c0 (BVR/BCR/WVR/WCR), c1 (MDSCR etc)
        # OS Lock: s2_0_c1_c0_4 (OSLAR_EL1)
        # MDSCR: s2_0_c0_c2_2 (op0=2, op1=0, crn=0, crm=2, op2=2)
        # External Debug: s2_0_c0_c0-7_x
        if op0 == 2:  # Debug registers are op0=2
            is_mrs = (word >> 21) & 1
            mn = 'MRS' if is_mrs else 'MSR'
            reg_name = f"s{op0}_{op1}_c{crn}_c{crm}_{op2}"
            rt = word & 0x1F
            debug_regs.append((base+off, mn, reg_name, rt))

if debug_regs:
    print(f"Found {len(debug_regs)} debug register accesses:")
    for addr, mn, reg, rt in debug_regs:
        print(f"  {addr:#010x}: {mn} {reg}, x{rt}")
else:
    print("NO debug register accesses found in ROM code!")

# === 8. LOOK FOR FUSE/CHIPID reads ===
print()
print("=== FUSE/CHIPID MMIO READS (what does ROM check?) ===")
# Find all LDR instructions from MMIO space
# The literal pools tell us which MMIO addresses are used
# 0x235100000 = likely AES/crypto engine
# 0x23D240000 = likely USB DWC3 registers
# 0x23B0B8000 is known (boot device / PMGR)
# What else?
for off in range(0, len(rom), 8):
    val = int.from_bytes(rom[off:off+8], 'little')
    if 0x200000000 <= val <= 0x400000000:
        region = (val >> 16) & 0xFFFF
        if region not in (0x2351, 0x23D2, 0x23B0):  # Skip known USB/AES/PMGR
            print(f"  {base+off:#010x} → {val:#014x} (unknown peripheral)")
