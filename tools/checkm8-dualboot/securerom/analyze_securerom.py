#!/usr/bin/env python3
"""
SecureROM T8020 B1 (A12 Bionic) — Full static analysis
Base address: 0x100000000 (mapped ROM)
SRAM: 0x19C000000
DFU load addr: 0x19C018800
Heap: 0x19C0D8000

This script:
1. Disassembles the reset vector and early boot
2. Finds function prologues (all functions)
3. Locates USB/DFU handlers via string references
4. Extracts ROP gadgets
5. Finds AES engine references
6. Maps the heap allocator
"""

import struct, sys, os, json
from collections import defaultdict
from capstone import *

ROM_PATH = os.path.join(os.path.dirname(__file__), "t8020_B1_securerom.bin")
BASE_ADDR = 0x100000000
SRAM_BASE = 0x19C000000

# Load ROM
with open(ROM_PATH, "rb") as f:
    rom = f.read()

print(f"SecureROM T8020 B1 loaded: {len(rom)} bytes ({len(rom)//1024} KB)")
print(f"Base: 0x{BASE_ADDR:X}, End: 0x{BASE_ADDR + len(rom):X}")

# Initialize Capstone AArch64
md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

# ============================================================
# PHASE 1: Reset vector and early boot
# ============================================================
print("\n" + "="*70)
print("PHASE 1: Reset Vector & Exception Table")
print("="*70)

# AArch64 reset vector is at offset 0
# Exception vector table (VBAR_EL3) typically at a 0x800-aligned address
# Each entry is 0x80 bytes (32 instructions)

# First 16 instructions (reset handler)
print("\n--- Reset Vector (first 32 instructions) ---")
for i in md.disasm(rom[:128], BASE_ADDR):
    print(f"  0x{i.address:011X}: {i.mnemonic:8s} {i.op_str}")

# ============================================================
# PHASE 2: Find ALL function prologues
# ============================================================
print("\n" + "="*70)
print("PHASE 2: Function Discovery")
print("="*70)

functions = []
# AArch64 function prologues: STP x29, x30, [sp, #-N]! (push FP+LR)
# Encoding: 0xA9B... or 0xA98... patterns
# Also: SUB sp, sp, #N as first instruction

# Method 1: Look for STP X29, X30, [SP, #imm]! (standard prologue)
for offset in range(0, len(rom) - 4, 4):
    instr = struct.unpack_from("<I", rom, offset)[0]
    # STP x29, x30, [sp, #imm]! — pre-index
    # Encoding: 1x101001 1xxxxxxx xxxxxxxx 11111101
    # Mask for STP x29, x30, [sp, ...]!
    if (instr & 0xFFE003E0) == 0xA9800FE0 or (instr & 0xFFE003E0) == 0xA9A00FE0 or \
       (instr & 0xFFC003FF) == 0xA9BF7BFD:
        addr = BASE_ADDR + offset
        functions.append(addr)
    # Also catch: STP x29, x30, [sp, #-0x10]! = 0xA9BF7BFD
    elif instr == 0xA9BF7BFD:
        addr = BASE_ADDR + offset
        if addr not in functions:
            functions.append(addr)

# Method 2: Look for PACIBSP (PAC instruction, common on A12)
# PACIBSP = 0xD503237F
for offset in range(0, len(rom) - 4, 4):
    instr = struct.unpack_from("<I", rom, offset)[0]
    if instr == 0xD503237F:  # PACIBSP
        addr = BASE_ADDR + offset
        if addr not in functions:
            functions.append(addr)

functions.sort()
print(f"Found {len(functions)} potential function prologues")
if functions[:10]:
    print("First 10 functions:")
    for addr in functions[:10]:
        print(f"  0x{addr:011X}")

# ============================================================
# PHASE 3: String table analysis
# ============================================================
print("\n" + "="*70)
print("PHASE 3: String References")
print("="*70)

# Find all printable strings >= 4 chars
strings = {}
i = 0
while i < len(rom):
    if 0x20 <= rom[i] < 0x7F:
        start = i
        while i < len(rom) and 0x20 <= rom[i] < 0x7F:
            i += 1
        if rom[i:i+1] == b'\x00' and (i - start) >= 4:
            s = rom[start:i].decode('ascii', errors='replace')
            addr = BASE_ADDR + start
            strings[addr] = s
    i += 1

print(f"Found {len(strings)} strings")

# Categorize interesting strings
dfu_strings = {a: s for a, s in strings.items() if any(k in s.lower() for k in ['dfu', 'usb', 'interface', 'endpoint', 'transfer', 'descriptor', 'device', 'config'])}
crypto_strings = {a: s for a, s in strings.items() if any(k in s.lower() for k in ['aes', 'sha', 'rsa', 'key', 'encrypt', 'decrypt', 'hash', 'sign', 'verify', 'img4', 'cert'])}
heap_strings = {a: s for a, s in strings.items() if any(k in s.lower() for k in ['heap', 'malloc', 'free', 'alloc', 'memalign', 'task', 'mem'])}
boot_strings = {a: s for a, s in strings.items() if any(k in s.lower() for k in ['boot', 'ibss', 'ibec', 'iboot', 'load', 'exec', 'jump', 'image'])}

print(f"\n--- USB/DFU related strings ({len(dfu_strings)}) ---")
for addr, s in sorted(dfu_strings.items()):
    print(f"  0x{addr:011X}: \"{s[:80]}\"")

print(f"\n--- Crypto/Signing strings ({len(crypto_strings)}) ---")
for addr, s in sorted(crypto_strings.items()):
    print(f"  0x{addr:011X}: \"{s[:80]}\"")

print(f"\n--- Heap/Memory strings ({len(heap_strings)}) ---")
for addr, s in sorted(heap_strings.items()):
    print(f"  0x{addr:011X}: \"{s[:80]}\"")

print(f"\n--- Boot/Image strings ({len(boot_strings)}) ---")
for addr, s in sorted(boot_strings.items()):
    print(f"  0x{addr:011X}: \"{s[:80]}\"")

# ============================================================
# PHASE 4: Find USB descriptor structures
# ============================================================
print("\n" + "="*70)
print("PHASE 4: USB Descriptor Structures")
print("="*70)

# USB Device Descriptor: 18 bytes, bLength=18, bDescriptorType=1
# Apple VID = 0x05AC, DFU PID = 0x1227
for offset in range(0, len(rom) - 18):
    if rom[offset] == 18 and rom[offset+1] == 1:  # device descriptor
        vid = struct.unpack_from("<H", rom, offset + 8)[0]
        pid = struct.unpack_from("<H", rom, offset + 10)[0]
        if vid == 0x05AC:
            addr = BASE_ADDR + offset
            bcd_usb = struct.unpack_from("<H", rom, offset + 2)[0]
            bcd_device = struct.unpack_from("<H", rom, offset + 12)[0]
            print(f"  USB Device Descriptor at 0x{addr:011X}")
            print(f"    VID=0x{vid:04X} PID=0x{pid:04X}")
            print(f"    bcdUSB=0x{bcd_usb:04X} bcdDevice=0x{bcd_device:04X}")
            print(f"    bDeviceClass={rom[offset+4]} bDeviceSubClass={rom[offset+5]}")
            print(f"    iManufacturer={rom[offset+14]} iProduct={rom[offset+15]} iSerialNumber={rom[offset+16]}")

# DFU Functional Descriptor: bLength=9, bDescriptorType=0x21
for offset in range(0, len(rom) - 9):
    if rom[offset] == 9 and rom[offset+1] == 0x21:
        bmAttributes = rom[offset+2]
        wDetachTimeout = struct.unpack_from("<H", rom, offset + 3)[0]
        wTransferSize = struct.unpack_from("<H", rom, offset + 5)[0]
        bcdDFU = struct.unpack_from("<H", rom, offset + 7)[0]
        if wTransferSize in [2048, 4096, 8192] and bcdDFU in [0x0100, 0x0101, 0x0110]:
            addr = BASE_ADDR + offset
            print(f"\n  DFU Functional Descriptor at 0x{addr:011X}")
            print(f"    bmAttributes=0x{bmAttributes:02X} (bitCanDnload={bmAttributes&1} bitCanUpload={(bmAttributes>>1)&1})")
            print(f"    wDetachTimeout={wDetachTimeout}ms")
            print(f"    wTransferSize={wTransferSize} bytes")
            print(f"    bcdDFUVersion=0x{bcdDFU:04X}")

# ============================================================
# PHASE 5: ROP Gadget Extraction
# ============================================================
print("\n" + "="*70)
print("PHASE 5: ROP Gadgets")
print("="*70)

gadgets = defaultdict(list)

# Scan all instructions
for i in md.disasm(rom, BASE_ADDR):
    # RET gadgets
    if i.mnemonic == 'ret':
        # Look at previous instructions (up to 4)
        prev_offset = (i.address - BASE_ADDR) - 4
        if prev_offset >= 0:
            prev_bytes = rom[prev_offset:prev_offset+4]
            for pi in md.disasm(prev_bytes, i.address - 4):
                gadget_str = f"{pi.mnemonic} {pi.op_str}; ret"
                gadgets[gadget_str].append(pi.address)
    
    # LDP x29, x30, [sp], #N ; RET — function epilogue
    if i.mnemonic == 'ldp' and 'x29, x30' in i.op_str and 'sp' in i.op_str:
        # Check if next is RET
        next_offset = (i.address - BASE_ADDR) + 4
        if next_offset < len(rom) - 4:
            next_instr = struct.unpack_from("<I", rom, next_offset)[0]
            if next_instr == 0xD65F03C0:  # RET
                gadget_str = f"ldp x29, x30, {i.op_str.split(',', 2)[2].strip()}; ret"
                gadgets[gadget_str].append(i.address)

    # MOV X0, Xn; RET — return value control
    if i.mnemonic == 'mov' and i.op_str.startswith('x0,'):
        next_offset = (i.address - BASE_ADDR) + 4
        if next_offset < len(rom) - 4:
            next_instr = struct.unpack_from("<I", rom, next_offset)[0]
            if next_instr == 0xD65F03C0:  # RET
                gadgets[f"mov {i.op_str}; ret"].append(i.address)

    # STR Xn, [Xm]; RET — write gadget
    if i.mnemonic == 'str' and i.op_str.startswith('x'):
        next_offset = (i.address - BASE_ADDR) + 4
        if next_offset < len(rom) - 4:
            next_instr = struct.unpack_from("<I", rom, next_offset)[0]
            if next_instr == 0xD65F03C0:
                gadgets[f"str {i.op_str}; ret"].append(i.address)

    # LDR Xn, [Xm]; RET — read gadget
    if i.mnemonic == 'ldr' and i.op_str.startswith('x'):
        next_offset = (i.address - BASE_ADDR) + 4
        if next_offset < len(rom) - 4:
            next_instr = struct.unpack_from("<I", rom, next_offset)[0]
            if next_instr == 0xD65F03C0:
                gadgets[f"ldr {i.op_str}; ret"].append(i.address)

    # BLR Xn — indirect call gadgets
    if i.mnemonic == 'blr':
        gadgets[f"blr {i.op_str}"].append(i.address)

    # BR Xn — indirect jump  
    if i.mnemonic == 'br':
        gadgets[f"br {i.op_str}"].append(i.address)

    # NOP; RET
    if i.mnemonic == 'nop':
        next_offset = (i.address - BASE_ADDR) + 4
        if next_offset < len(rom) - 4:
            next_instr = struct.unpack_from("<I", rom, next_offset)[0]
            if next_instr == 0xD65F03C0:
                gadgets["nop; ret"].append(i.address)

    # MSR instructions (system register writes)
    if i.mnemonic == 'msr':
        gadgets[f"msr {i.op_str}"].append(i.address)

# Print most useful gadgets
print(f"\nTotal unique gadget types: {len(gadgets)}")

# Priority gadgets for exploitation
priority_patterns = [
    'nop; ret',
    'mov x0,', 'mov x1,', 'mov x2,', 'mov x3,',
    'str x0,', 'str x1,',
    'ldr x0,', 'ldr x1,',
    'blr x8', 'blr x9', 'blr x16', 'blr x17',
    'br x8', 'br x16',
    'msr ttbr', 'msr sctlr', 'msr vbar',
]

print("\n--- Priority Gadgets for Exploitation ---")
for pattern in priority_patterns:
    matching = {k: v for k, v in gadgets.items() if pattern in k}
    for gname, addrs in sorted(matching.items()):
        print(f"  [{len(addrs):3d}x] {gname}")
        for a in addrs[:3]:  # Show first 3 addresses
            print(f"         0x{a:011X}")
        if len(addrs) > 3:
            print(f"         ... and {len(addrs)-3} more")

# ============================================================
# PHASE 6: Find MMIO regions (AES, USB, etc)
# ============================================================
print("\n" + "="*70)
print("PHASE 6: MMIO / Hardware Register References")
print("="*70)

# Known T8020 MMIO bases (from public docs / devicetree)
# AES engine: 0x23D2_0xxxx region
# USB DWC3: 0x39000xxxx region  
# Look for ADRP/ADD pairs that reference these regions

mmio_refs = defaultdict(int)
adrp_values = {}

for offset in range(0, len(rom) - 4, 4):
    instr = struct.unpack_from("<I", rom, offset)[0]
    
    # ADRP Xd, #imm — load page address
    if (instr & 0x9F000000) == 0x90000000:
        rd = instr & 0x1F
        immhi = (instr >> 5) & 0x7FFFF
        immlo = (instr >> 29) & 0x3
        imm = (immhi << 2) | immlo
        if imm & 0x100000:  # sign extend 21-bit
            imm |= ~0x1FFFFF
        page = ((BASE_ADDR + offset) & ~0xFFF) + (imm << 12)
        # Filter to MMIO ranges (not ROM, not SRAM)
        if page > 0x200000000 or (0x100000000 < page < BASE_ADDR):
            if 0 < page < 0xFFFFFFFFFFFF:  # sanity
                mmio_refs[page & ~0xFFF] += 1
                adrp_values[BASE_ADDR + offset] = (rd, page)

# Sort by frequency
print(f"\nTop MMIO page references (most accessed regions):")
for page, count in sorted(mmio_refs.items(), key=lambda x: -x[1])[:30]:
    region = "???"
    if 0x200000000 <= page < 0x240000000:
        region = "Peripheral/AES?"
    elif 0x380000000 <= page < 0x3C0000000:
        region = "USB/DWC3?"
    elif 0x19C000000 <= page < 0x1A0000000:
        region = "SRAM"
    elif 0x23D000000 <= page < 0x240000000:
        region = "AES/Crypto?"
    print(f"  0x{page:011X} ({region:15s}) — referenced {count}x")

# ============================================================ 
# PHASE 7: Find DFU handler dispatch
# ============================================================
print("\n" + "="*70)
print("PHASE 7: DFU Request Handler Dispatch")
print("="*70)

# Look for known DFU request numbers in CMP instructions near USB strings
# DFU requests: DETACH=0, DNLOAD=1, UPLOAD=2, GETSTATUS=3, CLRSTATUS=4, GETSTATE=5, ABORT=6
# The dispatch will have CMP Xn, #0..#6 in sequence

dfu_dispatch_candidates = []
for offset in range(0, len(rom) - 40, 4):
    # Look for a sequence of CMP with values 0-6 within 64 bytes
    cmp_vals = set()
    for j in range(0, 64, 4):
        if offset + j + 4 > len(rom):
            break
        instr = struct.unpack_from("<I", rom, offset + j)[0]
        # CMP Xn, #imm  (alias of SUBS XZR, Xn, #imm)
        # 1111000100 imm12 Rn 11111
        if (instr & 0xFF80001F) == 0xF100001F:
            imm12 = (instr >> 10) & 0xFFF
            if imm12 <= 6:
                cmp_vals.add(imm12)
        # CMP Wn, #imm
        if (instr & 0x7F80001F) == 0x7100001F:
            imm12 = (instr >> 10) & 0xFFF
            if imm12 <= 6:
                cmp_vals.add(imm12)
    
    if len(cmp_vals) >= 4 and {0, 1, 2, 3}.issubset(cmp_vals):
        addr = BASE_ADDR + offset
        dfu_dispatch_candidates.append((addr, cmp_vals))

print(f"DFU dispatch candidates (CMP #0..#6 clusters): {len(dfu_dispatch_candidates)}")
for addr, vals in dfu_dispatch_candidates[:10]:
    print(f"  0x{addr:011X} — CMP values: {sorted(vals)}")
    # Disassemble the region
    region = rom[addr - BASE_ADDR:addr - BASE_ADDR + 80]
    for ins in md.disasm(region, addr):
        print(f"    0x{ins.address:011X}: {ins.mnemonic:8s} {ins.op_str}")

# ============================================================
# PHASE 8: Find specific patterns
# ============================================================
print("\n" + "="*70)
print("PHASE 8: Specific Pattern Search")
print("="*70)

# 8a: Find DFU_DNLOAD buffer size check (2048 = 0x800)
print("\n--- Buffer size references (0x800 = 2048) ---")
for offset in range(0, len(rom) - 4, 4):
    instr = struct.unpack_from("<I", rom, offset)[0]
    # CMP Xn, #0x800
    if (instr & 0xFF80001F) == 0xF100001F:
        imm12 = (instr >> 10) & 0xFFF
        if imm12 == 0x800:
            addr = BASE_ADDR + offset
            rn = (instr >> 5) & 0x1F
            print(f"  0x{addr:011X}: CMP X{rn}, #0x800 (2048)")
    if (instr & 0x7F80001F) == 0x7100001F:
        imm12 = (instr >> 10) & 0xFFF
        if imm12 == 0x800:
            addr = BASE_ADDR + offset
            rn = (instr >> 5) & 0x1F
            print(f"  0x{addr:011X}: CMP W{rn}, #0x800 (2048)")

# 8b: Find "dfu_xbuf" or CRC32 salt
print("\n--- DFU xbuf salt (0xAC050001) ---")
# The DFU CRC uses a salt: FF FF FF FF AC 05 00 01 55 46 44 10
salt_bytes = bytes([0xAC, 0x05, 0x00, 0x01])
for offset in range(0, len(rom) - 4):
    if rom[offset:offset+4] == salt_bytes:
        addr = BASE_ADDR + offset
        context = rom[max(0,offset-4):offset+16]
        print(f"  0x{addr:011X}: Found salt bytes, context: {context.hex()}")

# Also look for the MOV/MOVK instruction loading 0xAC050001
for offset in range(0, len(rom) - 8, 4):
    instr = struct.unpack_from("<I", rom, offset)[0]
    # MOVZ Xn, #0x0001, LSL #16 followed by MOVK Xn, #0xAC05
    # or MOVZ Wn, #0xAC05, etc
    # Check for 0x05AC in MOV immediate
    if (instr & 0x7F800000) == 0x52800000:  # MOVZ/MOVK Wn
        imm16 = (instr >> 5) & 0xFFFF
        if imm16 == 0x05AC or imm16 == 0xAC05:
            addr = BASE_ADDR + offset
            rd = instr & 0x1F
            print(f"  0x{addr:011X}: MOV* W{rd}, #0x{imm16:04X}")

# 8c: Find io_buffer / io_request allocation (memalign or malloc with 0x800)
print("\n--- Allocation calls with size 0x800 ---")
for offset in range(0, len(rom) - 8, 4):
    instr = struct.unpack_from("<I", rom, offset)[0]
    # MOV X0, #0x800 or MOV X1, #0x800 (arg to malloc/memalign)
    if (instr & 0xFFE0001F) in [0xD2810000, 0xD2810001]:  # MOVZ X0/X1, #0x800
        # Wrong encoding, let me compute: MOVZ Xd, #imm16 = 0xD2800000 | (imm16 << 5) | Rd
        pass
    # MOV W0, #0x800 = MOVZ W0, #0x800 = 0x52810000
    if (instr & 0xFFFFFFE0) == 0x52810000:
        addr = BASE_ADDR + offset
        rd = instr & 0x1F
        print(f"  0x{addr:011X}: MOVZ W{rd}, #0x800")
        # Check if followed by BL (function call)
        next4 = struct.unpack_from("<I", rom, offset + 4)[0]
        if (next4 & 0xFC000000) == 0x94000000:  # BL
            imm26 = next4 & 0x3FFFFFF
            if imm26 & 0x2000000: imm26 |= ~0x3FFFFFF  # sign extend
            target = BASE_ADDR + offset + 4 + (imm26 << 2)
            print(f"         → BL 0x{target:011X} (likely allocator)")

# ============================================================
# PHASE 9: Overall statistics & summary
# ============================================================
print("\n" + "="*70)
print("PHASE 9: Summary")
print("="*70)

# Count instruction types
instr_counts = defaultdict(int)
total_instrs = 0
bl_targets = defaultdict(int)

for i in md.disasm(rom, BASE_ADDR):
    instr_counts[i.mnemonic] += 1
    total_instrs += 1
    if i.mnemonic == 'bl':
        # Extract target
        try:
            target = int(i.op_str.replace('#', ''), 16)
            bl_targets[target] += 1
        except:
            pass

print(f"Total instructions: {total_instrs}")
print(f"Total functions found: {len(functions)}")
print(f"Total strings: {len(strings)}")
print(f"Total unique gadget types: {len(gadgets)}")

print(f"\nTop 20 most called functions (BL targets):")
for target, count in sorted(bl_targets.items(), key=lambda x: -x[1])[:20]:
    print(f"  0x{target:011X} — called {count}x")

print(f"\nInstruction distribution (top 20):")
for mnemonic, count in sorted(instr_counts.items(), key=lambda x: -x[1])[:20]:
    pct = count / total_instrs * 100
    print(f"  {mnemonic:8s}: {count:6d} ({pct:.1f}%)")

# ============================================================
# Save results to JSON for later use
# ============================================================
output = {
    "rom_info": {
        "path": ROM_PATH,
        "size": len(rom),
        "base_addr": hex(BASE_ADDR),
        "chip": "T8020",
        "revision": "B1",
        "iboot_version": "iBoot-3865.0.0.4.7"
    },
    "functions": [hex(a) for a in functions],
    "strings": {hex(a): s for a, s in strings.items()},
    "dfu_strings": {hex(a): s for a, s in dfu_strings.items()},
    "crypto_strings": {hex(a): s for a, s in crypto_strings.items()},
    "gadgets": {k: [hex(a) for a in v[:5]] for k, v in gadgets.items()},
    "top_called_functions": {hex(t): c for t, c in sorted(bl_targets.items(), key=lambda x: -x[1])[:50]},
    "dfu_dispatch_candidates": [hex(a) for a, _ in dfu_dispatch_candidates],
    "mmio_pages": {hex(p): c for p, c in sorted(mmio_refs.items(), key=lambda x: -x[1])[:50]},
}

out_path = os.path.join(os.path.dirname(__file__), "t8020_B1_analysis.json")
with open(out_path, "w") as f:
    json.dump(output, f, indent=2)
print(f"\nFull analysis saved to: {out_path}")
print("Done!")
