"""
Verify: Is D3D0 (standard_device_request_cb) the callback for EP0 IN io_requests?
Find all ADRP+ADD references to 0x10000D3D0 in the ROM.
Also: scan for descriptor response lengths to find ZLP-triggering combinations.
Also: measure heap zone size from ROM data.
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

def disasm_at(offset, count=40):
    """Disassemble count instructions starting at ROM offset"""
    results = []
    for i in md.disasm(rom[offset:offset+count*4], ROM_BASE + offset):
        results.append(i)
    return results

def disasm_range(start_off, end_off):
    """Disassemble a range of offsets"""
    results = []
    for i in md.disasm(rom[start_off:end_off], ROM_BASE + start_off):
        results.append(i)
    return results

# ============================================================
# PART 1: Find all ADRP+ADD sequences referencing D3D0
# ============================================================
print("=" * 70)
print("PART 1: Scanning for references to 0x10000D3D0 in ROM")
print("=" * 70)

# Scan all instructions for ADRP that could resolve to page containing D3D0
# D3D0 is on page 0x10000D000
TARGET_ADDR = 0x10000D3D0
TARGET_PAGE = TARGET_ADDR & ~0xFFF  # 0x10000D000

hits = []
instrs = list(md.disasm(rom[:ACTIVE_END], ROM_BASE))
print(f"Total instructions disassembled: {len(instrs)}")

for idx, i in enumerate(instrs):
    if i.mnemonic == 'adrp':
        # Parse the target page from operands
        parts = i.op_str.split(',')
        if len(parts) == 2:
            try:
                page = int(parts[1].strip().lstrip('#'), 0)
                if page == TARGET_PAGE:
                    # Check next instruction for ADD with offset 0xD3D0 & 0xFFF = 0x3D0
                    if idx + 1 < len(instrs):
                        next_i = instrs[idx + 1]
                        if next_i.mnemonic == 'add':
                            # Check for immediate 0x3D0
                            if '#0x3d0' in next_i.op_str.lower():
                                hits.append((i, next_i))
                                print(f"\n  FOUND ADRP+ADD → 0x10000D3D0:")
                                print(f"    0x{i.address:X}: {i.mnemonic} {i.op_str}")
                                print(f"    0x{next_i.address:X}: {next_i.mnemonic} {next_i.op_str}")
                                
                                # Show context: next 10 instructions
                                print(f"    Context after:")
                                for k in range(2, min(12, len(instrs) - idx)):
                                    ctx = instrs[idx + k]
                                    print(f"      0x{ctx.address:X}: {ctx.mnemonic} {ctx.op_str}")
            except (ValueError, IndexError):
                pass

if not hits:
    print("\n  No direct ADRP+ADD references to D3D0 found.")
    print("  Trying broader search: any ADD with #0x3d0...")
    for idx, i in enumerate(instrs):
        if i.mnemonic == 'add' and '#0x3d0' in i.op_str.lower():
            prev = instrs[idx-1] if idx > 0 else None
            print(f"\n  0x{i.address:X}: {i.mnemonic} {i.op_str}")
            if prev:
                print(f"  prev: 0x{prev.address:X}: {prev.mnemonic} {prev.op_str}")

# ============================================================
# PART 2: Also search for ADR to D3D0
# ============================================================
print("\n" + "=" * 70)
print("PART 2: Scanning for ADR to 0x10000D3D0")
print("=" * 70)

for i in instrs:
    if i.mnemonic == 'adr':
        parts = i.op_str.split(',')
        if len(parts) == 2:
            try:
                addr = int(parts[1].strip().lstrip('#'), 0)
                if addr == TARGET_ADDR:
                    print(f"  FOUND: 0x{i.address:X}: {i.mnemonic} {i.op_str}")
            except (ValueError, IndexError):
                pass

# ============================================================
# PART 3: Scan for callback registration at +0x20 offset
# Look for STR <reg>, [<base>, #0x20] near ADRP/ADR of D3D0
# ============================================================
print("\n" + "=" * 70)
print("PART 3: Finding where callbacks are stored at io_req+0x20")
print("=" * 70)

# Let's look at usb_core_do_transfer (E0D8) and ep0_transfer_setup (D368)
# to trace how the callback is stored

print("\n--- ep0_transfer_setup (D368) ---")
off = 0xD368
for i in disasm_at(off, 30):
    ann = ""
    if i.address == ROM_BASE + 0xD368:
        ann = " ; ← ep0_transfer_setup entry"
    if "str" in i.mnemonic and "#0x20" in i.op_str:
        ann = " ; ★ STORES callback at +0x20"
    if i.mnemonic == 'bl':
        target = int(i.op_str.lstrip('#'), 0)
        if target == ROM_BASE + 0xF3B0:
            ann = " ; → calloc"
        elif target == ROM_BASE + 0xF1EC:
            ann = " ; → malloc"
    print(f"  0x{i.address:X}: {i.bytes.hex()} {i.mnemonic:6s} {i.op_str}{ann}")
    if i.mnemonic == 'ret':
        break

print("\n--- usb_core_do_transfer (E0D8) ---")
off = 0xE0D8
for i in disasm_at(off, 25):
    ann = ""
    if i.address == ROM_BASE + 0xE0D8:
        ann = " ; ← usb_core_do_transfer entry"
    if i.mnemonic == 'bl':
        target = int(i.op_str.lstrip('#'), 0)
        if target == ROM_BASE + 0xD368:
            ann = " ; → ep0_transfer_setup"
    print(f"  0x{i.address:X}: {i.bytes.hex()} {i.mnemonic:6s} {i.op_str}{ann}")
    if i.mnemonic == 'ret':
        break

# ============================================================
# PART 4: Find callers of usb_core_do_transfer (E0D8) 
# to see what callbacks are passed
# ============================================================
print("\n" + "=" * 70)
print("PART 4: Finding callers of usb_core_do_transfer (E0D8)")
print("=" * 70)

callers_e0d8 = []
for i in instrs:
    if i.mnemonic == 'bl':
        try:
            target = int(i.op_str.lstrip('#'), 0)
            if target == ROM_BASE + 0xE0D8:
                callers_e0d8.append(i.address)
        except:
            pass

print(f"\n  Found {len(callers_e0d8)} callers of usb_core_do_transfer (E0D8):")
for addr in callers_e0d8:
    off = addr - ROM_BASE
    print(f"\n  Caller at 0x{addr:X}:")
    # Show 15 instructions before the call to find the callback argument setup
    start = max(0, off - 60)
    context = list(md.disasm(rom[start:off+4], ROM_BASE + start))
    for ci in context[-16:]:
        ann = ""
        if ci.address == addr:
            ann = " ; ← CALL to usb_core_do_transfer"
        # x3 is the callback parameter
        if 'x3' in ci.op_str and ci.mnemonic in ('mov', 'add', 'adrp', 'ldr'):
            ann += " ; ★ sets x3 (callback)"
        print(f"    0x{ci.address:X}: {ci.mnemonic:6s} {ci.op_str}{ann}")

# ============================================================
# PART 5: USB descriptor response lengths
# Look for descriptors that might be multiples of 64
# ============================================================
print("\n" + "=" * 70)
print("PART 5: USB descriptor scan (looking for lengths % 64 == 0)")
print("=" * 70)

# Known DFU mode descriptors:
print("""
Standard USB DFU Descriptors (typical):
  Device Descriptor: 18 bytes → 18 % 64 = 18 → NO ZLP
  Config Descriptor: 27 bytes (9+9+9) → NO  
  String Descriptor 0: 4 bytes → NO
  String Descriptors: variable
  DFU Functional Descriptor: 9 bytes → NO
  BOS Descriptor: varies
  
For ZLP: need io_length > 0, io_length % 64 == 0, wLength > io_length
  Smallest qualifying io_length = 64 (0x40)
""")

# Scan ROM for descriptor-like structures
# USB Device Descriptor starts with bLength=18, bDescriptorType=1
print("Scanning ROM for USB Device Descriptor (18, 1, ...):")
for off in range(0, ACTIVE_END - 18):
    if rom[off] == 18 and rom[off+1] == 1:  # Device descriptor
        bcd = struct.unpack_from('<H', rom, off+2)[0]
        if bcd in (0x0200, 0x0210, 0x0300, 0x0310):  # Valid USB spec versions
            vid = struct.unpack_from('<H', rom, off+8)[0]
            pid = struct.unpack_from('<H', rom, off+10)[0]
            print(f"  Offset 0x{off:X}: bcdUSB={bcd:#06x} VID={vid:#06x} PID={pid:#06x} len=18")

# USB Config Descriptor starts with bLength=9, bDescriptorType=2
print("\nScanning ROM for USB Config Descriptor (9, 2, ...):")
for off in range(0, ACTIVE_END - 9):
    if rom[off] == 9 and rom[off+1] == 2:
        total_len = struct.unpack_from('<H', rom, off+2)[0]
        num_intf = rom[off+4]
        if total_len < 256 and num_intf <= 4:
            print(f"  Offset 0x{off:X}: wTotalLength={total_len} bNumInterfaces={num_intf}")
            print(f"    total_len={total_len} → {total_len} % 64 = {total_len % 64}", 
                  "→ ★ ZLP candidate!" if total_len % 64 == 0 and total_len > 0 else "→ no ZLP")

# BOS Descriptor starts with bLength=5, bDescriptorType=15
print("\nScanning ROM for BOS Descriptor (5, 15, ...):")
for off in range(0, ACTIVE_END - 5):
    if rom[off] == 5 and rom[off+1] == 15:
        total_len = struct.unpack_from('<H', rom, off+2)[0]
        num_caps = rom[off+4]
        if total_len < 256 and num_caps <= 8:
            print(f"  Offset 0x{off:X}: wTotalLength={total_len} bNumDeviceCaps={num_caps}")
            print(f"    total_len={total_len} → {total_len} % 64 = {total_len % 64}",
                  "→ ★ ZLP candidate!" if total_len % 64 == 0 and total_len > 0 else "→ no ZLP")

# ============================================================
# PART 6: Heap zone size from ROM
# ============================================================
print("\n" + "=" * 70)
print("PART 6: Heap zone analysis")
print("=" * 70)

# Look for heap init / zone setup code
# memalign at F680, free at F468, malloc at F1EC
# Zone structure typically has: base, size, free_count, bucket lists
# Scan for zone addresses referenced in heap code

print("\n--- Heap init scan (looking for zone setup near F1EC/F680/F468) ---")

# Let's look at the heap init function — typically called early in boot
# Search for references to known heap zone addresses
# 0x19C011E88 and 0x19C011468 were mentioned as zone addresses
for zone_addr in [0x19C011E88, 0x19C011468, 0x19C011000, 0x19C010000]:
    page = zone_addr & ~0xFFF
    page_off = zone_addr & 0xFFF
    for idx, i in enumerate(instrs):
        if i.mnemonic == 'adrp':
            parts = i.op_str.split(',')
            if len(parts) == 2:
                try:
                    p = int(parts[1].strip().lstrip('#'), 0)
                    if p == page:
                        if idx + 1 < len(instrs):
                            ni = instrs[idx+1]
                            if ni.mnemonic == 'add':
                                off_str = f"#0x{page_off:x}"
                                if off_str in ni.op_str.lower():
                                    print(f"  Reference to 0x{zone_addr:X} at 0x{i.address:X}")
                except:
                    pass

# Also look at heap_init type functions
# The function that sets up the heap zone typically calls memalign or writes zone fields
# Let's find what function references the SRAM heap region
print("\n--- Scanning for heap base address patterns ---")
SRAM_BASES = [0x19C000000, 0x19C008000, 0x19C010000, 0x19C018000, 0x19C020000]
for sram in SRAM_BASES:
    page = sram & ~0xFFF
    for i in instrs:
        if i.mnemonic == 'adrp':
            parts = i.op_str.split(',')
            if len(parts) == 2:
                try:
                    p = int(parts[1].strip().lstrip('#'), 0)
                    if p == page and i.address not in [ROM_BASE + x for x in range(0xC000, 0xF000)]:
                        # Skip USB functions we already know about
                        func_off = i.address - ROM_BASE
                        if func_off < 0xB000 or func_off > 0xF800:
                            print(f"  ADRP to page 0x{page:X} at 0x{i.address:X} (offset 0x{func_off:X})")
                except:
                    pass

# ============================================================
# PART 7: Disassemble heap zone init near memalign
# ============================================================
print("\n" + "=" * 70)
print("PART 7: Heap functions - zone structure")
print("=" * 70)

# Look at free (F468) to understand zone structure
print("\n--- free (F468) first 50 instructions ---")
off = 0xF468
for i in disasm_at(off, 50):
    ann = ""
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            if t == ROM_BASE + 0x10D80: ann = " ; → bzero"
            elif t == ROM_BASE + 0x10E00: ann = " ; → memset"
        except: pass
    if 'adrp' in i.mnemonic: ann = " ; ← address load"
    print(f"  0x{i.address:X}: {i.bytes.hex()} {i.mnemonic:6s} {i.op_str}{ann}")
    if i.mnemonic == 'ret':
        break

# FDEC bucket computation
print("\n--- Bucket computation (FDEC) ---")
off = 0xFDEC
for i in disasm_at(off, 20):
    print(f"  0x{i.address:X}: {i.bytes.hex()} {i.mnemonic:6s} {i.op_str}")
    if i.mnemonic == 'ret':
        break

print("\n\nDone.")
