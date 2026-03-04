#!/usr/bin/env python3
"""Analyze A12 SecureROM boot chain - signature verification path"""
from capstone import *
import struct

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True
with open('securerom/t8020_B1_securerom.bin', 'rb') as f:
    rom = f.read()
base = 0x100000000

# At 1B08: mov w1, #0x69620000; movk w1, #0x7373 -> 'ibss'
# At 1B64: mov w1, #0x696c0000; movk w1, #0x6c62 -> 'illb' 
w1_ibss = 0x69620000 | 0x7373
w1_illb = 0x696c0000 | 0x6c62
print(f"Tag 1: {struct.pack('>I', w1_ibss)}")
print(f"Tag 2: {struct.pack('>I', w1_illb)}")

# A86C is the critical image verification function
print()
print("=== A86C: Image verification function ===")
off = 0xA86C
code = rom[off:off+0x200]
cnt = 0
for i in md.disasm(code, base+off):
    print(f"  {i.address:#010x}: {i.mnemonic:8s} {i.op_str}")
    cnt += 1
    if i.mnemonic == 'ret' or cnt >= 60:
        break

# Check what's at 1B6C-1C20 (after tag setup, image verify + trampoline)
print()
print("=== 1B6C-1C20: Image verify + trampoline to iBoot ===")
off2 = 0x1B6C
code2 = rom[off2:off2+0xC0]
for i in md.disasm(code2, base+off2):
    print(f"  {i.address:#010x}: {i.mnemonic:8s} {i.op_str}")

# Check what happens at 1BF0 (error path from tbnz w26)
print()
print("=== 1BF0-1C30: Error/alternate path after DFU ===")
off3 = 0x1BF0
code3 = rom[off3:off3+0x50]
for i in md.disasm(code3, base+off3):
    print(f"  {i.address:#010x}: {i.mnemonic:8s} {i.op_str}")

# Check what's at 1C10 (cbz x26 target - verification SUCCESS path?)
print()
print("=== 1C10: If img4_verify returned NULL (=SUCCESS?) ===")
off4 = 0x1C10
code4 = rom[off4:off4+0x40]
for i in md.disasm(code4, base+off4):
    print(f"  {i.address:#010x}: {i.mnemonic:8s} {i.op_str}")

# Check for NOR/recovery boot path strings
print()
print("=== Searching for boot mode strings ===")
import re
for m in re.finditer(b'DFU|recovery|NOR|iBSS|iBEC|LLB|iBoot|boot-mode', rom):
    off = m.start()
    ctx = rom[max(0,off):off+40]
    printable = ''.join(chr(b) if 32 <= b < 127 else '.' for b in ctx)
    print(f"  {base+off:#012x}: {printable}")

# Check for the trampoline/jump to loaded image
# This is typically: blr or br to the entry point of the loaded image
print()
print("=== Searching for trampoline jump (br/blr to loaded image) ===")
for off in range(0x1B00, 0x1D00, 4):
    code = rom[off:off+4]
    for i in md.disasm(code, base+off):
        if i.mnemonic in ('br', 'blr'):
            print(f"  {i.address:#010x}: {i.mnemonic:8s} {i.op_str}")
