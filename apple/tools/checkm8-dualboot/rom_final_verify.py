"""
FINAL VERIFICATION:
1. Scan ROM for ALL string descriptor data (look for USB string format: bLength, bDescriptorType=3, UTF-16 data)
2. Scan for Apple VID (0x05AC) to find the actual DFU device descriptor
3. Scan for any response buffer with length % 64 == 0
4. Check the interface handlers for non-NULL callback registrations via BLR
5. Look at the vendor request path (A5E0)
"""
import struct, sys
from capstone import *

ROM_PATH = "securerom/t8020_B1_securerom.bin"
ROM_BASE = 0x100000000
ACTIVE_END = 0x25000

with open(ROM_PATH, "rb") as f:
    rom = f.read()

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

def disasm_range(start_off, end_off):
    results = []
    for i in md.disasm(rom[start_off:end_off], ROM_BASE + start_off):
        results.append(i)
    return results

# ============================================================
# PART 1: Find ALL USB string descriptors in ROM
# Format: bLength(1), bDescriptorType=3(1), UTF-16 chars...
# ============================================================
print("=" * 70)
print("PART 1: USB String Descriptors in ROM")
print("=" * 70)

string_descs = []
for off in range(0, len(rom) - 4):
    blen = rom[off]
    btype = rom[off + 1]
    if btype == 3 and blen >= 4 and blen <= 254 and blen % 2 == 0:
        # Check if it looks like valid UTF-16
        valid = True
        text = ""
        for i in range(2, blen, 2):
            if off + i + 1 < len(rom):
                ch = struct.unpack_from('<H', rom, off + i)[0]
                if ch == 0:
                    valid = False
                    break
                if ch < 0x20 or ch > 0x7F:
                    # Non-ASCII but still valid UTF-16
                    if ch > 0xFFFF:
                        valid = False
                        break
                    text += chr(ch) if ch < 0x10000 else '?'
                else:
                    text += chr(ch)
            else:
                valid = False
                break
        
        if valid and len(text) >= 2:  # At least 2 chars
            mod64 = blen % 64
            marker = " ★★★ MULTIPLE OF 64! ★★★" if mod64 == 0 else ""
            string_descs.append((off, blen, text, mod64))
            if blen >= 4:  # Only show non-trivial ones
                print(f"  Offset 0x{off:05X}: bLength={blen:3d} (0x{blen:02x}) mod64={mod64:2d} \"{text}\"{marker}")

print(f"\n  Total string descriptors found: {len(string_descs)}")
print(f"  With bLength % 64 == 0: {sum(1 for _, bl, _, m in string_descs if m == 0)}")

# Also check for Language ID descriptor (bLength=4, bType=3, langid=0x0409)
print("\n--- Language ID descriptors ---")
for off in range(0, len(rom) - 4):
    if rom[off] == 4 and rom[off+1] == 3:
        langid = struct.unpack_from('<H', rom, off+2)[0]
        if langid == 0x0409:
            print(f"  Offset 0x{off:05X}: LangID = 0x{langid:04X} (US English)")

# ============================================================
# PART 2: Find Apple VID device descriptors
# ============================================================
print("\n" + "=" * 70)
print("PART 2: Apple VID (0x05AC) Device Descriptors")
print("=" * 70)

for off in range(0, len(rom) - 18):
    if rom[off] == 18 and rom[off+1] == 1:  # Device descriptor
        bcd = struct.unpack_from('<H', rom, off+2)[0]
        vid = struct.unpack_from('<H', rom, off+8)[0]
        pid = struct.unpack_from('<H', rom, off+10)[0]
        if vid == 0x05AC:
            print(f"  Offset 0x{off:05X}: bcdUSB={bcd:#06x} VID={vid:#06x} PID={pid:#06x}")
            # Print full descriptor
            for i in range(18):
                print(f"    [{i:2d}] = 0x{rom[off+i]:02x}", end="")
                field_names = ["bLength","bDescriptorType","bcdUSB_lo","bcdUSB_hi",
                              "bDeviceClass","bDeviceSubClass","bDeviceProtocol","bMaxPacketSize0",
                              "idVendor_lo","idVendor_hi","idProduct_lo","idProduct_hi",
                              "bcdDevice_lo","bcdDevice_hi","iManufacturer","iProduct",
                              "iSerialNumber","bNumConfigurations"]
                print(f"  ({field_names[i]})")

# ============================================================
# PART 3: Find config descriptors with wTotalLength % 64 == 0
# ============================================================
print("\n" + "=" * 70)
print("PART 3: Config Descriptors (checking wTotalLength % 64)")
print("=" * 70)

for off in range(0, len(rom) - 9):
    if rom[off] == 9 and rom[off+1] == 2:
        total_len = struct.unpack_from('<H', rom, off+2)[0]
        num_intf = rom[off+4]
        if total_len < 512 and num_intf <= 8:
            mod64 = total_len % 64
            marker = " ★★★ MULT OF 64!" if mod64 == 0 else ""
            print(f"  Offset 0x{off:05X}: wTotalLength={total_len} bNumInterfaces={num_intf} mod64={mod64}{marker}")

# ============================================================
# PART 4: BOS descriptors
# ============================================================
print("\n" + "=" * 70)
print("PART 4: BOS Descriptors")
print("=" * 70)

for off in range(0, len(rom) - 5):
    if rom[off] == 5 and rom[off+1] == 15:
        total_len = struct.unpack_from('<H', rom, off+2)[0]
        num_caps = rom[off+4]
        if total_len < 256 and num_caps <= 8:
            mod64 = total_len % 64
            marker = " ★★★ MULT OF 64!" if mod64 == 0 else ""
            print(f"  Offset 0x{off:05X}: wTotalLength={total_len} bNumDeviceCaps={num_caps} mod64={mod64}{marker}")

# ============================================================
# PART 5: Analyze standard_device_request_cb (D3D0) conditions precisely
# ============================================================
print("\n" + "=" * 70)
print("PART 5: standard_device_request_cb (D3D0) precise analysis")
print("=" * 70)

for i in disasm_range(0xD3D0, 0xD400):
    ann = ""
    if i.mnemonic == 'ret':
        ann = " ; ← RET"
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            if t == ROM_BASE + 0xD334: ann = " ; → zlp_send"
        except: pass
    if 'cbz' in i.mnemonic or 'cbnz' in i.mnemonic:
        ann = " ; condition check"
    if 'and' in i.mnemonic and '#0x3f' in i.op_str:
        ann = " ; mod 64 check"
    if 'ldr' in i.mnemonic and '#0x14' in i.op_str:
        ann = " ; load io_length"
    if i.mnemonic == 'cmp':
        ann = " ; compare"
    print(f"  0x{i.address:X}: {i.mnemonic:6s} {i.op_str}{ann}")
    if i.mnemonic in ('ret', 'b') and i.address > ROM_BASE + 0xD3D0:
        if i.mnemonic == 'ret':
            break

# ============================================================
# PART 6: Check all possible response sizes
# What sizes does ep0_transfer_setup get called with?
# Trace all paths to DD80 (the common "mov x1,x20; bl D368")
# ============================================================
print("\n" + "=" * 70)
print("PART 6: All descriptor response sizes")
print("=" * 70)

print("""
From the ep0_handler disassembly:

GET_STATUS (standard): DBCC → DD80
  DBCC: adr x2, D3D0
  DBD4: mov w20, #2          → size = 2, 2 % 64 = 2 ✗

GET_DESCRIPTOR dispatch (DCC0-DF14) via jump table:
  Type 1 (DEVICE): DCE8 → min(wLength, 18) → DEF4 → DC7C → DD80
    Size = min(wLength, 18). Natural = 18, 18 % 64 = 18 ✗
  Type 2 (CONFIGURATION): DE1C → min(wLength, wTotalLength) → DEE0 → DC7C → DD80
    Size = min(wLength, wTotalLength). Natural = 27 (typ), 27 % 64 = 27 ✗
  Type 3 (STRING): DE30 → varies by index → DEDC → DC7C → DD80
    Size = min(wLength, bLength). Depends on string! NEEDS CHECK.
  Type 4 (INTERFACE): not implemented
  Type 5 (ENDPOINT): not implemented  
  Type 6 (DEVICE_QUALIFIER): DE50 → min(wLength, 10) → DEF4 → DC7C → DD80
    Size = min(wLength, 10). Natural = 10, 10 % 64 = 10 ✗
  Type 7 (OTHER_SPEED_CONFIG): DE7C → min(wLength, wTotalLength_alt) → DC7C → DD80
    Same as config for other speed.

GET_CONFIGURATION: DD18 → DD74 → DD80
  DD74: adr x2, D3D0
  DD7C: mov w20, #1          → size = 1, 1 % 64 = 1 ✗  

SET_ADDRESS callback: DC10 → DD80
  DC10: adr x2, A5BC (different callback!)  
  DC18: b DD80
  DC00: mov w20, #0          → size = 0 ✗

SET_CONFIGURATION: DC1C → DC78 → DC7C → DD80
  DC78: mov w20, #0          → size = 0 ✗

CONCLUSION: The ONLY way to get io_length % 64 == 0 is through a STRING 
DESCRIPTOR with bLength that is a multiple of 64.
""")

# ============================================================
# PART 7: Specifically search what string descriptors the DFU mode uses
# The descriptor data addresses are at:
# - USB_STATE+0x38 = response buffer
# - USB_STATE+0x50 = config descriptor pointer
# - USB_STATE+0x58 = other_speed_config pointer  
# - DFU device desc reference at 0x19C0088F0 + 0xA (offset from PART7 DD10)
# Check ROM data near the device descriptor area
# ============================================================
print("\n" + "=" * 70)
print("PART 7: DFU Mode Device Descriptor Area")
print("=" * 70)

# From DD08-DD10: adrp x8, 0x19C008000; add x8, x8, #0x8F0; add x1, x8, #0xA
# The device descriptor is at 0x19C008000 + 0x8F0 = 0x19C0088F0 (runtime SRAM)
# But the ROM might have the template. The adr+offset pattern suggests
# the device descriptor template might be near the end of ROM

# Let's look for the DFU string "Apple" or "DFU" in ROM
print("  Searching for 'A\x00p\x00p\x00l\x00e' (UTF-16LE) in ROM...")
search = b'A\x00p\x00p\x00l\x00e'
off = 0
while True:
    off = rom.find(search, off)
    if off == -1:
        break
    # Check if this is inside a string descriptor
    # Look backwards for bDescriptorType=3
    if off >= 2:
        possible_start = off - 2
        blen = rom[possible_start]
        btype = rom[possible_start + 1]
        if btype == 3:
            print(f"    String desc at 0x{possible_start:05X}: bLength={blen}")
            text = rom[possible_start+2:possible_start+blen].decode('utf-16-le', errors='replace')
            print(f"    Content: \"{text}\"")
            print(f"    bLength % 64 = {blen % 64}")
    off += 1

print("\n  Searching for 'D\x00F\x00U' (UTF-16LE) in ROM...")
search = b'D\x00F\x00U'
off = 0
while True:
    off = rom.find(search, off)
    if off == -1:
        break
    if off >= 2:
        possible_start = off - 2
        blen = rom[possible_start]
        btype = rom[possible_start + 1]
        if btype == 3 and blen < 200:
            print(f"    String desc at 0x{possible_start:05X}: bLength={blen}")
            text = rom[possible_start+2:possible_start+blen].decode('utf-16-le', errors='replace')
            print(f"    Content: \"{text}\"")
            print(f"    bLength % 64 = {blen % 64}")
    off += 1

# ============================================================
# PART 8: Exhaustive search for ANY sequence of bytes that could be 
# a 64-byte string descriptor (bLen=64, bType=3)
# ============================================================
print("\n" + "=" * 70)
print("PART 8: Searching for 64-byte, 128-byte, 192-byte string descriptors")
print("=" * 70)

for target_len in [64, 128, 192]:
    print(f"\n  Looking for bLength={target_len} (0x{target_len:02x}), bType=3:")
    count = 0
    for off in range(0, len(rom) - target_len):
        if rom[off] == target_len and rom[off+1] == 3:
            # Check if it's plausibly a string descriptor
            valid_chars = 0
            for i in range(2, target_len, 2):
                ch = struct.unpack_from('<H', rom, off + i)[0]
                if 0x20 <= ch <= 0x7E:  # Printable ASCII
                    valid_chars += 1
            ratio = valid_chars / ((target_len - 2) // 2)
            if ratio > 0.5:  # More than half chars are printable
                text = rom[off+2:off+target_len].decode('utf-16-le', errors='replace')
                print(f"    Offset 0x{off:05X}: \"{text[:40]}...\" (printable ratio: {ratio:.0%})")
                count += 1
    if count == 0:
        print(f"    None found.")

# ============================================================
# PART 9: Summary of ALL response sizes for ep0_transfer_setup 
# ============================================================
print("\n" + "=" * 70)
print("PART 9: Summary — Can D3D0 callback EVER trigger zlp_send on A12?")
print("=" * 70)

print("""
For D3D0 to call zlp_send, ALL of these must be true:
  1. io_length > 0
  2. io_length % 64 == 0  
  3. wLength > io_length (from SETUP packet, controlled by attacker)

Condition 3 is trivially satisfiable (set wLength = desc_size + 1).

Condition 2 requires a descriptor/response with natural size that's a multiple of 64.

DFU mode response sizes:
  Device Descriptor:     18 bytes  → 18 % 64 = 18  ✗
  Config Descriptor:     ~27 bytes → 27 % 64 = 27  ✗
  String 0 (Lang ID):   4 bytes   → 4 % 64 = 4    ✗
  String 1 (Manuf.):    varies    → CHECK ROM
  String 2 (Product):   varies    → CHECK ROM
  String 3+ (Serial):   varies    → CHECK ROM
  Device Qualifier:      10 bytes  → 10 % 64 = 10  ✗
  GET_STATUS:            2 bytes   → 2 % 64 = 2    ✗
  GET_CONFIGURATION:     1 byte    → 1 % 64 = 1    ✗
  BOS Descriptor:        varies    → CHECK ROM

DFU responses (callback=NULL, so D3D0 never called):
  DFU_GETSTATUS:  6 bytes, callback=NULL  → N/A
  DFU_GETSTATE:   1 byte, callback=NULL   → N/A  
  DFU_UPLOAD:     not implemented          → N/A

If NO descriptor has bLength % 64 == 0, then D3D0 NEVER triggers zlp_send,
and the ZLP leak is COMPLETELY BLOCKED regardless of the double-abort.
""")

print("\nDone.")
