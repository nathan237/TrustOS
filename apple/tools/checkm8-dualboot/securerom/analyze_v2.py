#!/usr/bin/env python3
"""
SecureROM T8020 B1 (A12) — Cross-reference analysis with known T8010/T8015 gadgets
Uses pattern matching from fully-reversed T8010/T8015 SecureROMs to find
equivalent functions in T8020.

Known addresses from ipwndfu checkm8.py:
  T8010 (A10, iBoot-2696.0.0.1.33):
    nop_gadget      = 0x10000CC6C  (LDP x29,x30,[sp,#0x10]; LDP x20,x19,[sp],#0x20; RET)
    func_gadget     = 0x10000CC4C  (LDP x8,x10,[x0,#0x70]; ...; BLR x10; ...)
    USB_CORE_DO_IO  = 0x10000DC98
    gUSBDescriptors = 0x180088A30
    gUSBSerialNumber= 0x180083CF8
    LOAD_ADDRESS    = 0x1800B0000
    dc_civac        = 0x10000046C
    dmb_ret         = 0x100000478
    write_ttbr0     = 0x1000003E4
    tlbi            = 0x100000434
    enter_critical  = (not published)
    
  T8015 (A11, iBoot-3332.0.0.1.23):
    nop_gadget      = 0x10000A9C4
    USB_CORE_DO_IO  = 0x10000B9A8
    gUSBDescriptors = 0x180008528
    gUSBSerialNumber= 0x180003A78
    LOAD_ADDRESS    = 0x18001C000

  T8020 (A12, iBoot-3865.0.0.4.7): OUR TARGET
    ROM base        = 0x100000000
    SRAM base       = 0x19C000000
    DFU load addr   = 0x19C018800
    Heap base       = 0x19C0D8000
"""

import struct, sys, os, json
from collections import defaultdict
from capstone import *

ROM_PATH = os.path.join(os.path.dirname(__file__), "t8020_B1_securerom.bin")
BASE_ADDR = 0x100000000

with open(ROM_PATH, "rb") as f:
    rom = f.read()

print(f"SecureROM T8020 B1: {len(rom)} bytes ({len(rom)//1024} KB)")

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

# ============================================================
# PHASE 0: Check if ROM is encrypted/compressed
# ============================================================
print("\n" + "="*70)
print("PHASE 0: ROM Content Analysis")
print("="*70)

# Check entropy of different sections
import math
def entropy(data):
    if not data: return 0
    freq = defaultdict(int)
    for b in data:
        freq[b] += 1
    e = 0.0
    for count in freq.values():
        p = count / len(data)
        if p > 0:
            e -= p * math.log2(p)
    return e

sections = [
    ("Header (0x000-0x400)", rom[0:0x400]),
    ("Code 1 (0x400-0x4000)", rom[0x400:0x4000]),
    ("Code 2 (0x4000-0x10000)", rom[0x4000:0x10000]),
    ("Mid (0x10000-0x1C000)", rom[0x10000:0x1C000]),
    ("Strings (0x1C000-0x20000)", rom[0x1C000:0x20000]),
    ("Code 3 (0x20000-0x40000)", rom[0x20000:0x40000]),
    ("Data (0x40000-0x60000)", rom[0x40000:0x60000]),
    ("End (0x60000-0x80000)", rom[0x60000:0x80000]),
]

for name, data in sections:
    e = entropy(data)
    zeros = data.count(0)
    # Count valid AArch64 instructions
    valid_instrs = 0
    for off in range(0, len(data) - 3, 4):
        word = struct.unpack_from("<I", data, off)[0]
        if word == 0: continue  # skip zero padding
        # Quick check: is it a valid ARM64 instruction?
        # Most invalid: 0x00000000 (UDF), 0xFFFFFFFF
        # Valid ARM64 instructions have recognizable top-byte patterns
        top = (word >> 24) & 0xFF
        if top in range(0x00, 0xFF):
            valid_instrs += 1
    print(f"  {name:30s}: entropy={e:.2f}, zeros={zeros}/{len(data)} ({zeros*100//len(data)}%)")

# Count all zero bytes vs non-zero
total_zero = rom.count(0)
total_ff = rom.count(0xFF)
print(f"\nOverall: {total_zero} zero bytes ({total_zero*100//len(rom)}%), {total_ff} 0xFF bytes ({total_ff*100//len(rom)}%)")

# ============================================================
# PHASE 1: Full disassembly scan (skip invalid, continue)
# ============================================================
print("\n" + "="*70)
print("PHASE 1: Full Disassembly (skip-invalid mode)")
print("="*70)

all_instructions = {}  # addr -> (mnemonic, op_str, bytes)
skipped_regions = 0
current_offset = 0

# Disassemble in chunks, skipping over invalid regions
chunk_size = 0x1000  # 4KB chunks
for chunk_start in range(0, len(rom), chunk_size):
    chunk_end = min(chunk_start + chunk_size, len(rom))
    chunk_data = rom[chunk_start:chunk_end]
    
    for i in md.disasm(chunk_data, BASE_ADDR + chunk_start):
        all_instructions[i.address] = (i.mnemonic, i.op_str, i.bytes)

print(f"Total valid instructions: {len(all_instructions)}")

# Also try disassembling from every 4-byte aligned offset for regions
# that the chunk-based approach might miss
extra = 0
for offset in range(0, len(rom), 4):
    addr = BASE_ADDR + offset
    if addr in all_instructions:
        continue
    word = struct.unpack_from("<I", rom, offset)[0]
    if word == 0 or word == 0xFFFFFFFF:
        continue
    # Try to disassemble this single instruction
    for i in md.disasm(rom[offset:offset+4], addr):
        all_instructions[addr] = (i.mnemonic, i.op_str, i.bytes)
        extra += 1

print(f"Extra instructions from per-word scan: {extra}")
print(f"Total after full scan: {len(all_instructions)}")

# Instruction statistics
mnemonic_counts = defaultdict(int)
for addr, (mnem, ops, _) in all_instructions.items():
    mnemonic_counts[mnem] += 1

print(f"\nTop 30 instruction types:")
for mnem, count in sorted(mnemonic_counts.items(), key=lambda x: -x[1])[:30]:
    print(f"  {mnem:10s}: {count:6d}")

# ============================================================
# PHASE 2: Find ALL strings (comprehensive)
# ============================================================
print("\n" + "="*70)
print("PHASE 2: Complete String Table")
print("="*70)

strings = {}
i_str = 0
while i_str < len(rom):
    if 0x20 <= rom[i_str] < 0x7F:
        start = i_str
        while i_str < len(rom) and 0x20 <= rom[i_str] < 0x7F:
            i_str += 1
        if i_str < len(rom) and rom[i_str] == 0 and (i_str - start) >= 4:
            s = rom[start:i_str].decode('ascii', errors='replace')
            strings[BASE_ADDR + start] = s
    i_str += 1

print(f"Total strings: {len(strings)}")
print("\n--- ALL strings ---")
for addr, s in sorted(strings.items()):
    print(f"  0x{addr:011X}: \"{s[:100]}\"")

# ============================================================
# PHASE 3: Pattern search for known gadgets
# ============================================================
print("\n" + "="*70)
print("PHASE 3: Known Gadget Pattern Search")
print("="*70)

# Specific byte patterns from T8010 exploit
# nop_gadget on T8010: LDP X29,X30,[SP,#0x10]; LDP X20,X19,[SP],#0x20; RET
# Encoding: A9 41 7B FD  |  A8 C1 53 F4  |  D6 5F 03 C0
nop_pattern = bytes([0xFD, 0x7B, 0x41, 0xA9, 0xF4, 0x53, 0xC1, 0xA8, 0xC0, 0x03, 0x5F, 0xD6])

print("\n--- Searching for nop_gadget pattern (LDP x29,x30; LDP x20,x19; RET) ---")
for offset in range(0, len(rom) - len(nop_pattern)):
    if rom[offset:offset+len(nop_pattern)] == nop_pattern:
        addr = BASE_ADDR + offset
        print(f"  FOUND at 0x{addr:011X}")
        # Disassemble context
        for ins in md.disasm(rom[offset:offset+16], addr):
            print(f"    {ins.mnemonic} {ins.op_str}")

# func_gadget on T8010: LDP X8,X10,[X0,#0x70]
# Encoding (partial): A9 43 C1 A8 is not right... let me search by instruction
print("\n--- Searching for func_gadget (LDP x8, x10, [x0, #0x70]) ---")
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if mnem == 'ldp' and 'x8, x10' in ops and 'x0' in ops:
        print(f"  FOUND at 0x{addr:011X}: {mnem} {ops}")
        # Show surrounding instructions
        for delta in range(-8, 24, 4):
            a = addr + delta
            if a in all_instructions:
                m, o, _ = all_instructions[a]
                marker = " >>>" if delta == 0 else "    "
                print(f"  {marker} 0x{a:011X}: {m} {o}")

# dc_civac: SYS #3, c7, c14, #1, X0 followed by RET
print("\n--- Searching for dc_civac gadget (sys #3, c7, c14, #1) ---")  
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if mnem == 'sys' and 'c7' in ops and 'c14' in ops:
        print(f"  FOUND at 0x{addr:011X}: {mnem} {ops}")
        # Check if RET follows
        next_addr = addr + 4
        if next_addr in all_instructions and all_instructions[next_addr][0] == 'ret':
            print(f"    → followed by RET = dc_civac gadget!")

# dmb; ret
print("\n--- Searching for dmb; ret gadget ---")
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if mnem == 'dmb':
        next_addr = addr + 4
        if next_addr in all_instructions and all_instructions[next_addr][0] == 'ret':
            print(f"  FOUND at 0x{addr:011X}: dmb {ops}; ret")

# write_ttbr0: MSR TTBR0_EL1, Xn; ISB; RET
print("\n--- Searching for write_ttbr0 (MSR TTBR0_EL1) ---")
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if mnem == 'msr' and 'ttbr0_el1' in ops.lower():
        print(f"  FOUND at 0x{addr:011X}: {mnem} {ops}")

# MSR SCTLR_EL1 (for WXN disable)
print("\n--- Searching for MSR SCTLR_EL1 ---")
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if mnem == 'msr' and 'sctlr_el1' in ops.lower():
        print(f"  FOUND at 0x{addr:011X}: {mnem} {ops}")

# MSR VBAR
print("\n--- Searching for MSR VBAR_EL1 ---")
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if mnem == 'msr' and 'vbar' in ops.lower():
        print(f"  FOUND at 0x{addr:011X}: {mnem} {ops}")

# TLBI
print("\n--- Searching for TLBI sequence (DSB; TLBI; DSB; ISB; RET) ---")
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if mnem == 'dsb':
        # Check DSB; SYS(TLBI); DSB; ISB; RET in next instructions
        a1, a2, a3, a4 = addr+4, addr+8, addr+12, addr+16
        if all(a in all_instructions for a in [a1, a2, a3, a4]):
            if all_instructions[a1][0] == 'sys' and \
               all_instructions[a2][0] == 'dsb' and \
               all_instructions[a3][0] == 'isb' and \
               all_instructions[a4][0] == 'ret':
                print(f"  FOUND at 0x{addr:011X}: DSB; TLBI; DSB; ISB; RET")

# Enter/exit critical section: MSR DAIFSet/DAIFClr
print("\n--- Searching for enter_critical (MSR DAIFSet) and exit_critical (MSR DAIFClr) ---")
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if mnem == 'msr' and 'daifset' in ops.lower():
        print(f"  enter_critical at 0x{addr:011X}: {mnem} {ops}")
    if mnem == 'msr' and 'daifclr' in ops.lower():
        print(f"  exit_critical at 0x{addr:011X}: {mnem} {ops}")

# ============================================================
# PHASE 4: BL targets (most called functions)
# ============================================================
print("\n" + "="*70)
print("PHASE 4: Function Call Graph (BL targets)")
print("="*70)

bl_targets = defaultdict(int)
blr_count = 0

for addr, (mnem, ops, raw_bytes) in sorted(all_instructions.items()):
    if mnem == 'bl':
        try:
            target = int(ops.replace('#', ''), 16)
            bl_targets[target] += 1
        except:
            pass
    elif mnem == 'blr':
        blr_count += 1

print(f"Total BL (direct call) instructions: {sum(bl_targets.values())}")
print(f"Total BLR (indirect call) instructions: {blr_count}")
print(f"Unique BL targets (functions): {len(bl_targets)}")

print(f"\nTop 40 most called functions:")
for target, count in sorted(bl_targets.items(), key=lambda x: -x[1])[:40]:
    # Try to find nearby strings for naming
    name = ""
    for str_addr, s in strings.items():
        if abs(str_addr - target) < 0x100:
            name = f" (near: \"{s[:30]}\")"
            break
    print(f"  0x{target:011X}: called {count:4d}x{name}")

# ============================================================
# PHASE 5: ROP Gadgets (comprehensive)
# ============================================================
print("\n" + "="*70)
print("PHASE 5: Comprehensive ROP Gadget Search")
print("="*70)

gadgets = {}

# Find all RET instructions first
ret_addrs = [a for a, (m, _, _) in all_instructions.items() if m == 'ret']
print(f"Total RET instructions: {len(ret_addrs)}")

# For each RET, look back 1-3 instructions for useful gadgets
for ret_addr in ret_addrs:
    for lookback in range(1, 4):
        gadget_start = ret_addr - (lookback * 4)
        if gadget_start < BASE_ADDR: continue
        
        # Build gadget string
        parts = []
        valid = True
        for j in range(lookback + 1):
            a = gadget_start + (j * 4)
            if a in all_instructions:
                m, o, _ = all_instructions[a]
                parts.append(f"{m} {o}")
            else:
                valid = False
                break
        
        if not valid: continue
        gadget_str = "; ".join(parts)
        
        # Classify useful gadgets
        useful = False
        if lookback == 1:  # 2-instruction gadgets
            m0 = all_instructions.get(gadget_start, ("","",""))[0]
            if m0 in ('mov', 'str', 'ldr', 'ldp', 'stp', 'msr', 'add', 'sub',
                       'orr', 'and', 'eor', 'nop', 'sys', 'dsb', 'dmb', 'isb'):
                useful = True
        elif lookback == 2:  # 3-instruction gadgets
            m0 = all_instructions.get(gadget_start, ("","",""))[0]
            m1 = all_instructions.get(gadget_start + 4, ("","",""))[0]
            # ldp x29,x30; ldp x20,x19; ret  (nop gadget)
            if m0 == 'ldp' and m1 == 'ldp':
                useful = True
            # mov x0,xn; mov x1,xn; ret
            if m0 == 'mov' and m1 == 'mov':
                useful = True
        
        if useful:
            if gadget_str not in gadgets:
                gadgets[gadget_str] = []
            gadgets[gadget_str].append(gadget_start)

# Print categorized gadgets
categories = {
    "STACK PIVOT": ["mov sp", "add sp", "ldp x29, x30"],
    "REGISTER CONTROL": ["mov x0,", "mov x1,", "mov x2,", "mov x3,"],
    "MEMORY WRITE": ["str x", "str w", "stp x"],
    "MEMORY READ": ["ldr x0", "ldr x1", "ldp x0"],
    "SYSTEM REG": ["msr ", "sys "],
    "NOP SLED": ["nop; ret", "ldp x29, x30"],
    "INDIRECT CALL": ["blr "],
}

for cat_name, patterns in categories.items():
    matching = {}
    for gname, addrs in gadgets.items():
        for p in patterns:
            if p in gname:
                matching[gname] = addrs
                break
    
    if matching:
        print(f"\n--- {cat_name} ({len(matching)} types) ---")
        for gname, addrs in sorted(matching.items(), key=lambda x: -len(x[1]))[:15]:
            print(f"  [{len(addrs):3d}x] {gname[:80]}")
            for a in addrs[:2]:
                print(f"         0x{a:011X}")

# ============================================================
# PHASE 6: USB Descriptor & Handler Structures
# ============================================================
print("\n" + "="*70)
print("PHASE 6: USB Structures & VID/PID References")
print("="*70)

# Search for Apple VID 0x05AC in immediate loads
print("--- References to 0x05AC (Apple VID) ---")
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if '0x5ac' in ops.lower() or '0x05ac' in ops.lower():
        print(f"  0x{addr:011X}: {mnem} {ops}")

# Search for 0x1227 (DFU PID)
print("\n--- References to 0x1227 (DFU PID) ---")
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if '0x1227' in ops.lower():
        print(f"  0x{addr:011X}: {mnem} {ops}")

# Search for 0x1281 (Recovery PID)
print("\n--- References to 0x1281 (Recovery PID) ---")
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if '0x1281' in ops.lower():
        print(f"  0x{addr:011X}: {mnem} {ops}")

# Search for DFU buffer size 0x800
print("\n--- References to 0x800 (DFU buffer size) ---")
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if mnem in ('mov', 'movz', 'cmp', 'sub', 'add') and '#0x800' in ops:
        print(f"  0x{addr:011X}: {mnem} {ops}")

# ============================================================
# PHASE 7: ADRP analysis (pointer references)
# ============================================================
print("\n" + "="*70)
print("PHASE 7: ADRP+ADD Pointer References (SRAM/MMIO)")
print("="*70)

adrp_refs = {}
for offset in range(0, len(rom) - 4, 4):
    instr = struct.unpack_from("<I", rom, offset)[0]
    
    # ADRP Xd, #imm
    if (instr & 0x9F000000) == 0x90000000:
        rd = instr & 0x1F
        immhi = (instr >> 5) & 0x7FFFF
        immlo = (instr >> 29) & 0x3
        imm = (immhi << 2) | immlo
        # Sign extend 21-bit
        if imm & 0x100000:
            imm = imm - 0x200000
        page = ((BASE_ADDR + offset) & ~0xFFF) + (imm << 12)
        
        addr = BASE_ADDR + offset
        
        # Check if next instruction is ADD (ADRP+ADD pair = full address)
        if offset + 4 < len(rom):
            next_instr = struct.unpack_from("<I", rom, offset + 4)[0]
            # ADD Xd, Xn, #imm12
            if (next_instr & 0xFFC00000) == 0x91000000:
                add_imm = (next_instr >> 10) & 0xFFF
                shift = (next_instr >> 22) & 0x3
                if shift == 1: add_imm <<= 12
                full_addr = page + add_imm
                if full_addr != BASE_ADDR and full_addr != 0:
                    if full_addr not in adrp_refs:
                        adrp_refs[full_addr] = []
                    adrp_refs[full_addr].append(addr)

# Show SRAM references (global variables)
sram_refs = {a: refs for a, refs in adrp_refs.items() if 0x19C000000 <= a < 0x1A0000000}
print(f"\nSRAM references ({len(sram_refs)} unique addresses):")
# Most referenced SRAM addresses = important global variables
for sram_addr, refs in sorted(sram_refs.items(), key=lambda x: -len(x[1]))[:30]:
    print(f"  0x{sram_addr:011X}: referenced {len(refs):3d}x")

# Non-SRAM, non-ROM references = MMIO
mmio_refs = {a: refs for a, refs in adrp_refs.items() 
             if a >= 0x200000000 or (0x100080000 < a < 0x19C000000)}
print(f"\nMMIO references ({len(mmio_refs)} unique addresses):")
for mmio_addr, refs in sorted(mmio_refs.items(), key=lambda x: -len(x[1]))[:20]:
    region = "Unknown"
    if 0x200000000 <= mmio_addr < 0x240000000: region = "Peripheral"
    elif 0x380000000 <= mmio_addr < 0x400000000: region = "USB/IO"
    elif 0x240000000 <= mmio_addr < 0x280000000: region = "AES/Crypto?"
    print(f"  0x{mmio_addr:011X} ({region:15s}): referenced {len(refs):3d}x")

# ============================================================
# PHASE 8: Cross-reference with T8010 patterns  
# ============================================================
print("\n" + "="*70)
print("PHASE 8: T8010/T8015 Cross-Reference")
print("="*70)

# The T8010 exploit uses these key operations:
# 1. stall: GET_DESCRIPTOR(STRING, idx=4, wLength=0xC0) → allocates 0x30 byte io_request
# 2. DNLOAD with 0x800 bytes → fills io_buffer
# 3. DFU_CLRSTATUS resets DFU → frees io_buffer
# 4. Overflow via stale pointer

# Key function signatures to find in T8020:
# usb_core_do_io: called with io_request struct, handles USB endpoint I/O
# This function has the callback dispatch: LDP X8,X10,[X0,#0x70]; ...; BLR X10

print("\nSearching for callback dispatch pattern (usb_core_complete_endpoint_io)...")
# The pattern is: load callback+next from io_request, call callback, then free
# Key: LDP Xa, Xb, [X0, #0x70] or similar (load from io_request at offset 0x70-0x80)
for addr, (mnem, ops, _) in sorted(all_instructions.items()):
    if mnem == 'ldp' and '#0x70' in ops and 'x0' in ops:
        print(f"  LDP at 0x{addr:011X}: {mnem} {ops}")
    if mnem == 'ldp' and '#0x68' in ops and 'x0' in ops:
        print(f"  LDP at 0x{addr:011X}: {mnem} {ops}")
    if mnem == 'ldp' and '#0x78' in ops and 'x0' in ops:
        print(f"  LDP at 0x{addr:011X}: {mnem} {ops}")

# Search for the serial number format string reference
# "CPID:%04X CPRV:%02X CPFM:%02X SCEP:%02X BDID:%02X ECID:%016llX IBFL:%02X SRTG:[%s]"
print("\n--- Searching for DFU serial number format patterns ---")
for addr, s in sorted(strings.items()):
    if 'CPID' in s or 'ECID' in s or 'SRTG' in s or 'IBFL' in s:
        print(f"  0x{addr:011X}: \"{s[:100]}\"")

# Search for "DFU" string
for addr, s in sorted(strings.items()):
    if 'DFU' in s or 'dfu' in s.lower():
        print(f"  0x{addr:011X}: \"{s[:80]}\"")

# ============================================================
# PHASE 9: Save full analysis
# ============================================================
print("\n" + "="*70)
print("PHASE 9: Save Results")
print("="*70)

output = {
    "rom_info": {
        "chip": "T8020",
        "revision": "B1",
        "iboot_version": "iBoot-3865.0.0.4.7",
        "size": len(rom),
        "base_addr": hex(BASE_ADDR),
        "sram_base": "0x19C000000",
        "dfu_load_addr": "0x19C018800",
        "heap_base": "0x19C0D8000",
    },
    "stats": {
        "total_instructions": len(all_instructions),
        "total_strings": len(strings),
        "total_bl_targets": len(bl_targets),
        "total_ret": len(ret_addrs),
        "total_gadget_types": len(gadgets),
        "total_sram_refs": len(sram_refs),
    },
    "strings": {hex(a): s for a, s in sorted(strings.items())},
    "top_functions": {hex(t): c for t, c in sorted(bl_targets.items(), key=lambda x: -x[1])[:50]},
    "sram_globals": {hex(a): len(refs) for a, refs in sorted(sram_refs.items(), key=lambda x: -len(x[1]))[:100]},
    "gadgets_summary": {k: [hex(a) for a in v[:3]] for k, v in sorted(gadgets.items(), key=lambda x: -len(x[1]))[:100]},
}

out_path = os.path.join(os.path.dirname(__file__), "t8020_B1_analysis.json")
with open(out_path, "w") as f:
    json.dump(output, f, indent=2)
print(f"Analysis saved to: {out_path}")
print("DONE!")
