#!/usr/bin/env python3
"""
Final verification: confirm the exact callback dispatch sequence at both sites
to determine if T8020 uses BLR x9 (not BLR x10 like T8010)
"""
import struct, os
from capstone import *

ROM_PATH = os.path.join(os.path.dirname(__file__), "t8020_B1_securerom.bin")
BASE_ADDR = 0x100000000

with open(ROM_PATH, "rb") as f:
    rom = f.read()

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

def disasm(addr, length=0x80):
    off = addr - BASE_ADDR
    for i in md.disasm(rom[off:off+length], addr):
        print(f"  0x{i.address:011X}: {i.mnemonic:10s} {i.op_str}")

print("=" * 70)
print("CALLBACK DISPATCH SITE 1 — 0x10000A444 ± context")
print("=" * 70)
# Walk back to function start
disasm(0x10000A430, 0x50)

print("\n" + "=" * 70)
print("CALLBACK DISPATCH SITE 2 — 0x10000B9F0 ± context")
print("=" * 70)
disasm(0x10000B9D0, 0x60)

# Also check: does the ROM have ANY BLR x9 instructions?
print("\n" + "=" * 70)
print("ALL BLR instructions in ROM")
print("=" * 70)
blr_stats = {}
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    for i in md.disasm(rom[offset:offset+4], addr):
        if i.mnemonic == 'blr':
            blr_stats.setdefault(i.op_str, []).append(addr)

for reg, addrs in sorted(blr_stats.items(), key=lambda x: -len(x[1])):
    print(f"  BLR {reg:5s}: {len(addrs):3d} instances")
    for a in addrs[:3]:
        print(f"    0x{a:011X}")

# Check for BR (indirect jump, no link) 
print("\n" + "=" * 70)
print("ALL BR instructions in ROM")
print("=" * 70)
br_stats = {}
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    for i in md.disasm(rom[offset:offset+4], addr):
        if i.mnemonic == 'br':
            br_stats.setdefault(i.op_str, []).append(addr)

for reg, addrs in sorted(br_stats.items(), key=lambda x: -len(x[1])):
    print(f"  BR {reg:5s}: {len(addrs):3d} instances")
    for a in addrs[:3]:
        print(f"    0x{a:011X}")

# Check: any PACIA/AUTIA/PACIB/AUTIB near callback sites?
print("\n" + "=" * 70)
print("PAC INSTRUCTIONS IN ROM (PACIA/AUTIA/PACIB/AUTIB/BRAA/BLRAA)")
print("=" * 70)
pac_instrs = []
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    for i in md.disasm(rom[offset:offset+4], addr):
        if any(p in i.mnemonic for p in ['paci', 'auti', 'pacib', 'autib', 'braa', 'blraa', 'retaa', 'retab', 'ereta']):
            pac_instrs.append((addr, i.mnemonic, i.op_str))

if pac_instrs:
    print(f"Found {len(pac_instrs)} PAC instructions:")
    for addr, m, o in pac_instrs[:20]:
        print(f"  0x{addr:011X}: {m} {o}")
else:
    print("NO PAC INSTRUCTIONS FOUND IN ROM — Callbacks are NOT PAC-protected!")

# Verify: check for PACIASP/AUTIASP (function-level PAC)
print("\n" + "=" * 70)
print("PACIASP/AUTIASP (Function-level PAC)")
print("=" * 70)
pac_sp = []
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    word = struct.unpack_from("<I", rom, offset)[0]
    if word == 0xD503233F:  # PACIASP
        pac_sp.append((addr, "paciasp"))
    elif word == 0xD50323BF:  # AUTIASP  
        pac_sp.append((addr, "autiasp"))

if pac_sp:
    print(f"Found {len(pac_sp)} PACIASP/AUTIASP:")
    for addr, name in pac_sp[:20]:
        print(f"  0x{addr:011X}: {name}")
else:
    print("NO PACIASP/AUTIASP FOUND — SecureROM does NOT use function-level PAC!")
    print("This means return addresses on stack are NOT authenticated.")
    print("ROP chains will work without PAC bypass!")
