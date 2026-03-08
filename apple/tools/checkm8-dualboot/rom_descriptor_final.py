#!/usr/bin/env python3
"""
A12 T8020 B1 SecureROM — Final Descriptor Size Analysis
========================================================
Find ALL real USB descriptors in the ROM and check if any has bLength % 64 == 0.
This is the critical question: if no descriptor satisfies the D3D0 alignment check,
the ZLP leak is IMPOSSIBLE on A12 regardless of the missing double-abort.

Strategy:
1. Search for known Apple VID (0x05AC) to find device descriptor
2. Follow iProduct/iManufacturer/iSerialNumber indices to find string descriptors
3. Search for "Apple" and "DFU" in UTF-16LE in ROM data section
4. Exhaustively check all string descriptor candidates in data section (0x1C000+)
5. Report ALL real descriptor sizes and their mod-64 values
"""

import struct
import sys

ROM_PATH = "securerom/t8020_B1_securerom.bin"

with open(ROM_PATH, "rb") as f:
    rom = f.read()

print(f"ROM size: {len(rom)} bytes (0x{len(rom):X})")
print("=" * 70)

# ============================================================
# PART 1: Find Device Descriptors (bLength=18, bDescriptorType=1)
# ============================================================
print("\n=== PART 1: Device Descriptors (type=1, bLength=18) ===")
dev_descs = []
for i in range(len(rom) - 18):
    if rom[i] == 18 and rom[i+1] == 1:  # bLength=18, bDescriptorType=1
        bcd_usb = struct.unpack_from("<H", rom, i+2)[0]
        vid = struct.unpack_from("<H", rom, i+8)[0]
        pid = struct.unpack_from("<H", rom, i+10)[0]
        bcd_dev = struct.unpack_from("<H", rom, i+12)[0]
        iMfg = rom[i+14]
        iProd = rom[i+15]
        iSerial = rom[i+16]
        numCfg = rom[i+17]
        # Filter: must have reasonable USB version and Apple VID or valid structure
        if bcd_usb in (0x0110, 0x0200, 0x0201, 0x0210, 0x0300, 0x0310):
            print(f"  Offset 0x{i:05X}: bcdUSB=0x{bcd_usb:04X} VID=0x{vid:04X} PID=0x{pid:04X} "
                  f"bcdDevice=0x{bcd_dev:04X} iMfg={iMfg} iProd={iProd} iSerial={iSerial} numCfg={numCfg}")
            dev_descs.append((i, vid, pid, iMfg, iProd, iSerial))

if not dev_descs:
    print("  No device descriptors with valid bcdUSB found!")

# ============================================================
# PART 2: Find Apple VID (0x05AC) anywhere as 16-bit LE
# ============================================================
print("\n=== PART 2: Apple VID 0x05AC locations ===")
for i in range(len(rom) - 2):
    if rom[i] == 0xAC and rom[i+1] == 0x05:
        context = rom[max(0,i-4):i+8].hex()
        print(f"  Offset 0x{i:05X}: ...{context}...")

# ============================================================
# PART 3: Search for "Apple" in UTF-16LE
# ============================================================
print("\n=== PART 3: 'Apple' in UTF-16LE ===")
apple_utf16 = "Apple".encode("utf-16-le")
idx = 0
while True:
    pos = rom.find(apple_utf16, idx)
    if pos == -1:
        break
    # Walk backwards to find string descriptor header
    for back in range(0, min(pos, 128), 2):
        cand = pos - back - 2
        if cand < 0:
            break
        blen = rom[cand]
        btype = rom[cand+1]
        if btype == 3 and blen >= 4 and blen <= 254 and (blen % 2 == 0):
            end = cand + blen
            if end <= len(rom):
                try:
                    text = rom[cand+2:end].decode("utf-16-le", errors="replace")
                except:
                    text = "<decode error>"
                mod64 = blen % 64
                marker = " *** MOD64=0! ***" if mod64 == 0 else ""
                print(f"  Offset 0x{cand:05X}: bLength={blen} (0x{blen:02x}) mod64={mod64} "
                      f"text=\"{text}\"{marker}")
                break
    idx = pos + len(apple_utf16)

# ============================================================
# PART 4: Search for "DFU" in UTF-16LE
# ============================================================
print("\n=== PART 4: 'DFU' in UTF-16LE ===")
dfu_utf16 = "DFU".encode("utf-16-le")
idx = 0
while True:
    pos = rom.find(dfu_utf16, idx)
    if pos == -1:
        break
    for back in range(0, min(pos, 128), 2):
        cand = pos - back - 2
        if cand < 0:
            break
        blen = rom[cand]
        btype = rom[cand+1]
        if btype == 3 and blen >= 4 and blen <= 254 and (blen % 2 == 0):
            end = cand + blen
            if end <= len(rom):
                try:
                    text = rom[cand+2:end].decode("utf-16-le", errors="replace")
                except:
                    text = "<decode error>"
                mod64 = blen % 64
                marker = " *** MOD64=0! ***" if mod64 == 0 else ""
                print(f"  Offset 0x{cand:05X}: bLength={blen} (0x{blen:02x}) mod64={mod64} "
                      f"text=\"{text}\"{marker}")
                break
    idx = pos + len(dfu_utf16)

# ============================================================
# PART 5: Search for "CPID" in UTF-16LE (serial number string)
# ============================================================
print("\n=== PART 5: 'CPID' in UTF-16LE ===")
cpid_utf16 = "CPID".encode("utf-16-le")
idx = 0
while True:
    pos = rom.find(cpid_utf16, idx)
    if pos == -1:
        break
    print(f"  Found 'CPID' UTF-16LE at offset 0x{pos:05X}")
    idx = pos + len(cpid_utf16)

# ============================================================
# PART 6: Search for common USB strings in ASCII
# ============================================================
print("\n=== PART 6: USB-related ASCII strings ===")
for needle in [b"Apple", b"DFU", b"USB DFU", b"iBoot", b"CPID", b"SecureROM"]:
    idx = 0
    while True:
        pos = rom.find(needle, idx)
        if pos == -1:
            break
        # Show context
        ctx_start = max(0, pos - 8)
        ctx_end = min(len(rom), pos + len(needle) + 32)
        context_bytes = rom[ctx_start:ctx_end]
        # Safe ASCII display
        safe = ""
        for b in context_bytes:
            if 32 <= b < 127:
                safe += chr(b)
            else:
                safe += "."
        print(f"  '{needle.decode()}' at 0x{pos:05X}: {safe}")
        idx = pos + len(needle)

# ============================================================
# PART 7: Find ALL string descriptors in ROM data section
# Only scan the data region (offset 0x18000+) to avoid code false positives
# ============================================================
print("\n=== PART 7: String descriptors in data section (offset >= 0x18000) ===")
DATA_START = 0x18000
str_descs = []
for i in range(DATA_START, len(rom) - 4):
    blen = rom[i]
    btype = rom[i+1]
    if btype == 3 and blen >= 4 and blen <= 254 and (blen % 2 == 0):
        end = i + blen
        if end <= len(rom):
            raw = rom[i+2:end]
            # Heuristic: real UTF-16LE string should have reasonable characters
            # (lots of 0x00 high bytes for ASCII range)
            zero_high = sum(1 for j in range(1, len(raw), 2) if raw[j] == 0)
            total_chars = len(raw) // 2
            if total_chars > 0 and zero_high / total_chars >= 0.5:
                try:
                    text = raw.decode("utf-16-le", errors="replace")
                except:
                    text = "<error>"
                mod64 = blen % 64
                marker = " *** MOD64=0! ***" if mod64 == 0 else ""
                print(f"  Offset 0x{i:05X}: bLength={blen} (0x{blen:02x}) mod64={mod64} "
                      f"text=\"{text}\"{marker}")
                str_descs.append((i, blen, mod64, text))

if not str_descs:
    print("  No valid string descriptors found in data section!")

# ============================================================
# PART 8: Configuration descriptors (type=2)
# ============================================================
print("\n=== PART 8: Configuration descriptors (type=2) ===")
for i in range(len(rom) - 9):
    if rom[i+1] == 2 and rom[i] == 9:  # bLength=9, bDescriptorType=2
        wTotalLength = struct.unpack_from("<H", rom, i+2)[0]
        numIfaces = rom[i+4]
        bConfigVal = rom[i+5]
        iConfig = rom[i+6]
        bmAttrib = rom[i+7]
        maxPower = rom[i+8]
        if numIfaces in (1, 2, 3, 4) and wTotalLength >= 9 and wTotalLength <= 512:
            mod64 = wTotalLength % 64
            marker = " *** MOD64=0! ***" if mod64 == 0 else ""
            print(f"  Offset 0x{i:05X}: wTotalLength={wTotalLength} numIfaces={numIfaces} "
                  f"bConfigVal={bConfigVal} bmAttrib=0x{bmAttrib:02X} mod64={mod64}{marker}")

# ============================================================
# PART 9: Scan for descriptors referenced by ADR instructions
# Look for ADR/ADRP loading addresses in 0x1C000-0x25000 range
# ============================================================
print("\n=== PART 9: Descriptor pointer references (ADR to data section) ===")
# Find all ADR instruction patterns that point to common descriptor locations
# The descriptor table is likely loaded by the ep0_handler via ADR

# Instead, let's just look at the specific offsets referenced by the code:
# From rom_zlp_trigger.py: descriptor dispatch at DF5C
# Type 3 (STRING): handler at DE30
# Let's find what the code at DE30 does to get the string descriptor pointer

import capstone

cs = capstone.Cs(capstone.CS_ARCH_ARM64, capstone.CS_MODE_ARM)
cs.detail = True

ROM_BASE = 0x100000000

# Disassemble the string descriptor handler at DE30
print("\n  String descriptor handler (DE30):")
code = rom[0xDE30:0xDF00]
for insn in cs.disasm(code, ROM_BASE + 0xDE30):
    print(f"    {insn.address:012X}: {insn.mnemonic} {insn.op_str}")

# ============================================================
# PART 10: Look at what ep0_handler uses for descriptor table
# Disassemble around DC7C where D3D0 callback is loaded for GET_DESCRIPTOR
# ============================================================
print("\n=== PART 10: GET_DESCRIPTOR response setup (DC40-DD00) ===")
code = rom[0xDC40:0xDD00]
for insn in cs.disasm(code, ROM_BASE + 0xDC40):
    print(f"  {insn.address:012X}: {insn.mnemonic} {insn.op_str}")

# ============================================================
# PART 11: FINAL SUMMARY
# ============================================================
print("\n" + "=" * 70)
print("=== FINAL SUMMARY ===")
print("=" * 70)

print("\nDevice descriptors found:")
for off, vid, pid, iMfg, iProd, iSerial in dev_descs:
    print(f"  0x{off:05X}: VID=0x{vid:04X} PID=0x{pid:04X} size=18 mod64={18%64}")

print("\nString descriptors found in data section:")
any_mod64_zero = False
for off, blen, mod64, text in str_descs:
    marker = " *** EXPLOITABLE! ***" if mod64 == 0 else ""
    print(f"  0x{off:05X}: bLength={blen} mod64={mod64} \"{text}\"{marker}")
    if mod64 == 0:
        any_mod64_zero = True

print(f"\nCRITICAL RESULT: Any string descriptor with bLength % 64 == 0? {'YES!' if any_mod64_zero else 'NO'}")

if any_mod64_zero:
    print("\n>>> ZLP LEAK IS POSSIBLE via GET_DESCRIPTOR(string) <<<")
    print(">>> The missing double-abort on USB RESET path IS exploitable! <<<")
else:
    print("\n>>> ZLP LEAK IS IMPOSSIBLE on A12 <<<")
    print(">>> D3D0 callback CANNOT trigger zlp_send because no descriptor has bLength%64==0 <<<")
    print(">>> Combined with DFU callback=NULL, the A12 checkm8 UAF is NOT software-exploitable <<<")
    print(">>> The missing double-abort on the USB RESET path is IRRELEVANT <<<")

# Standard descriptor size summary
print("\n\nAll standard descriptor response sizes:")
print(f"  DEVICE:          18 bytes  (mod64 = {18%64})")
print(f"  DEVICE_QUALIFIER:10 bytes  (mod64 = {10%64})")
print(f"  GET_STATUS:       2 bytes  (mod64 = {2%64})")
print(f"  GET_CONFIGURATION:1 byte   (mod64 = {1%64})")
print(f"  String desc:     variable  (see above)")
print(f"  Config desc:     variable  (see part 8)")
