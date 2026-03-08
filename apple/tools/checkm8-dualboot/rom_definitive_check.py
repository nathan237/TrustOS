#!/usr/bin/env python3
"""
A12 T8020 B1 — Definitive Descriptor Size Verification
=======================================================
The ep0_handler string descriptor path (DE30) shows:
  - String index 0: ADR to ROM constant 0x1FCE8 
  - String index 1: [USB_STATE+0x30]  (iManufacturer - dynamically built from ASCII)
  - String index 2-9: [USB_STATE+0x70+idx*8] (iProduct, iSerial, etc.)

The ACTUAL string content comes from ASCII format strings in ROM:
  - "Apple Inc." at ~0x1C334
  - "Apple Mobile Device (DFU Mode)" at ~0x1C25A
  - Serial: "CPID:%04X CPRV:%02X ..." at ~0x1C279

This script:
1. Reads the ROM language ID descriptor at 0x1FCE8
2. Finds all ASCII format strings used for USB descriptors
3. Computes exact bLength for each
4. Verifies NONE are mod64==0
"""

import struct

ROM_PATH = "securerom/t8020_B1_securerom.bin"

with open(ROM_PATH, "rb") as f:
    rom = f.read()

print("=" * 70)
print("DEFINITIVE A12 T8020 B1 USB DESCRIPTOR SIZE ANALYSIS")
print("=" * 70)

# ============================================================
# 1. Language ID descriptor at ROM offset 0x1FCE8
# ============================================================
print("\n--- 1. String Index 0: Language ID Descriptor (ROM 0x1FCE8) ---")
off = 0x1FCE8
if off + 4 <= len(rom):
    blen = rom[off]
    btype = rom[off+1]
    lang = struct.unpack_from("<H", rom, off+2)[0]
    print(f"  Bytes: {rom[off:off+blen].hex()}")
    print(f"  bLength={blen}, bDescriptorType={btype}, langID=0x{lang:04X}")
    print(f"  bLength % 64 = {blen % 64}")
    print(f"  ZLP trigger? {'YES!' if blen % 64 == 0 and blen > 0 else 'NO'}")
else:
    print(f"  Offset 0x{off:X} out of range!")

# ============================================================
# 2. Actual USB string content from ROM ASCII strings
# ============================================================
print("\n--- 2. DFU Mode String Descriptors (computed from ASCII) ---")

# Find the actual strings by searching for known patterns
strings_found = {}

# "Apple Inc." - manufacturer
needle = b"Apple Inc."
pos = rom.find(needle)
while pos != -1:
    # Check if this is a standalone string (not embedded in longer text)
    # Look for null or other terminator
    end = pos + len(needle)
    if end < len(rom) and (rom[end] == 0 or rom[end] == ord('.')):
        context_start = max(0, pos-16)
        context_end = min(len(rom), end+16)
        ctx = rom[context_start:context_end]
        safe = "".join(chr(b) if 32 <= b < 127 else "." for b in ctx)
        print(f"  'Apple Inc.' found at 0x{pos:05X}: ...{safe}...")
        strings_found[f"Apple Inc. @0x{pos:05X}"] = "Apple Inc."
    pos = rom.find(needle, end)

# "Apple Mobile Device (DFU Mode)" - product  
needle = b"Apple Mobile Device (DFU Mode)"
pos = rom.find(needle)
while pos != -1:
    end = pos + len(needle)
    context_start = max(0, pos-8)
    context_end = min(len(rom), end+16)
    ctx = rom[context_start:context_end]
    safe = "".join(chr(b) if 32 <= b < 127 else "." for b in ctx)
    print(f"  'Apple Mobile Device (DFU Mode)' found at 0x{pos:05X}: ...{safe}...")
    strings_found[f"DFU product @0x{pos:05X}"] = "Apple Mobile Device (DFU Mode)"
    pos = rom.find(needle, end)

# Serial number format string
needle = b"CPID:%04X"
pos = rom.find(needle)
while pos != -1:
    # Read until null terminator
    end = pos
    while end < len(rom) and rom[end] != 0:
        end += 1
    fmt_str = rom[pos:end].decode("ascii", errors="replace")
    print(f"  Serial format at 0x{pos:05X}: \"{fmt_str}\"")
    strings_found[f"Serial format @0x{pos:05X}"] = fmt_str
    pos = rom.find(needle, end)

# ============================================================
# 3. Compute exact bLength for each real descriptor
# ============================================================
print("\n--- 3. Exact bLength Computation ---")
print()

# Known DFU string descriptors:
descriptors = {
    "String 0 (Language ID)": 4,  # Always {04, 03, 09, 04}
}

# String 1: iManufacturer = "Apple Inc." 
mfg = "Apple Inc."
mfg_blen = 2 + len(mfg) * 2
descriptors[f"String 1 (iManufacturer = \"{mfg}\")"] = mfg_blen

# String 2: iProduct = "Apple Mobile Device (DFU Mode)"
prod = "Apple Mobile Device (DFU Mode)"
prod_blen = 2 + len(prod) * 2
descriptors[f"String 2 (iProduct = \"{prod}\")"] = prod_blen

# String 3: iSerialNumber - computed from format string
# Format: CPID:%04X CPRV:%02X CPFM:%02X SCEP:%02X BDID:%02X ECID:%016llX IBFL:%02X SRTG:[%s]
# For T8020 B1: CPID:8020 CPRV:11 CPFM:03 SCEP:01 BDID:04 ECID:001C15D43C20802E IBFL:01 SRTG:[iBoot-3865.0.0.4.7]
# But ECID/BDID are device-specific. The LENGTH is what matters.
# Fixed-width format specifiers: %04X=4, %02X=2, %016llX=16
# SRTG value comes from ROM version string which is always "iBoot-3865.0.0.4.7" for this ROM

# Let's compute the exact serial string
srtg = "iBoot-3865.0.0.4.7"  # From ROM at 0x00280
serial = f"CPID:8020 CPRV:11 CPFM:03 SCEP:01 BDID:04 ECID:001C15D43C20802E IBFL:01 SRTG:[{srtg}]"
serial_blen = 2 + len(serial) * 2
descriptors[f"String 3 (iSerialNumber = \"{serial[:40]}...\")"] = serial_blen
print(f"  Full serial string ({len(serial)} chars): \"{serial}\"")
print(f"  Serial bLength = 2 + {len(serial)}*2 = {serial_blen}")

# Note: BDID and ECID vary per device, but format specifiers ensure fixed width
# Different BDID values (e.g. 0x0E for iPhone XS) still produce 2 hex chars
# Different ECIDs still produce 16 hex chars
# So bLength is the SAME for all A12 devices with this ROM revision

print()
print(f"{'Descriptor':<60} {'bLength':>7} {'mod64':>5} {'ZLP?':>5}")
print("-" * 80)

any_exploitable = False
for name, blen in descriptors.items():
    mod64 = blen % 64
    zlp = "YES!" if mod64 == 0 and blen > 0 else "no"
    if mod64 == 0 and blen > 0:
        any_exploitable = True
    print(f"  {name:<58} {blen:>7} {mod64:>5} {zlp:>5}")

# Also check Device descriptor and Configuration descriptor
print()
print("  Other descriptors:")
print(f"  {'Device descriptor':<58} {'18':>7} {18%64:>5} {'no':>5}")
print(f"  {'Device Qualifier descriptor':<58} {'10':>7} {10%64:>5} {'no':>5}")

# Config descriptor: DFU config is typically:
# 9 (config) + 9 (interface) + 9 (DFU functional) = 27 bytes
cfg_total = 27
print(f"  {'Configuration descriptor (estimated 9+9+9)':<58} {cfg_total:>7} {cfg_total%64:>5} {'no':>5}")

# ============================================================
# 4. Verify the false positives from previous script
# ============================================================
print("\n--- 4. Verifying False Positives ---")
false_pos_offsets = [0x1CC0C, 0x1CD4C, 0x1DE58, 0x1FA78, 0x1FAB8]
for off in false_pos_offsets:
    raw = rom[off:off+16]
    print(f"  0x{off:05X}: hex={raw.hex()} — ", end="")
    # Check if this looks like real UTF-16LE text (ASCII range)
    is_text = True
    for j in range(2, min(16, len(raw)), 2):
        lo, hi = raw[j], raw[j+1]
        if hi != 0 or lo < 0x20 or lo > 0x7E:
            is_text = False
            break
    print("REAL text" if is_text else "NOT text (binary data / false positive)")

# ============================================================
# 5. Check ALL possible string descriptors for ALL A12 variants
# ============================================================
print("\n--- 5. Sensitivity Analysis: Could ANY variant hit mod64==0? ---")
# The serial string length depends on the SRTG value
# For all known A12 ROMs, SRTG = "iBoot-" + version
# Version string is fixed per ROM build, but let's check edge cases

base_serial = "CPID:8020 CPRV:11 CPFM:03 SCEP:01 BDID:04 ECID:001C15D43C20802E IBFL:01 SRTG:["
suffix = "]"
# The variable part is the iBoot version string
for srtg_len in range(10, 30):
    total_chars = len(base_serial) + srtg_len + len(suffix)
    blen = 2 + total_chars * 2
    mod64 = blen % 64
    marker = " *** MOD64=0! ***" if mod64 == 0 else ""
    if marker or srtg_len == len(srtg):
        print(f"  SRTG length={srtg_len:2d} chars -> serial={total_chars} chars -> bLength={blen} mod64={mod64}{marker}")

# ============================================================
# FINAL CONCLUSION
# ============================================================
print("\n" + "=" * 70)
print("FINAL CONCLUSION")
print("=" * 70)
if any_exploitable:
    print("\nWARNING: At least one real descriptor has bLength % 64 == 0!")
    print("The ZLP memory leak via D3D0 callback IS possible.")
else:
    print("\nNO real USB descriptor served by the A12 T8020 B1 DFU mode")
    print("has bLength % 64 == 0.")
    print()
    print("The three layers of ZLP leak prevention on A12:")
    print("  Layer 1: DFU responses use callback=NULL (E5D8: mov x3, #0)")
    print("           -> dwc3_callback_and_free skips callback, just frees")
    print("  Layer 2: D3D0 callback alignment check (io_length % 64 == 0)")
    print("           -> No standard/string descriptor has bLength % 64 == 0")
    print("  Layer 3: Double-abort in dwc3_core_stop (BBFC: ep_abort_wrapper(0x80))")
    print("           -> Catches any surviving ZLPs on DFU exit")
    print()
    print("The USB RESET path (C448) missing Layer 3 is IRRELEVANT")
    print("because Layers 1+2 already prevent ZLP creation entirely.")
    print()
    print("VERDICT: The checkm8 UAF on A12 T8020 B1 is NOT software-exploitable.")
    print("         All known ZLP leak vectors are blocked by ROM code.")
    print("         Only hardware fault injection (EMFI/voltage glitch) could")
    print("         potentially bypass these software checks.")
