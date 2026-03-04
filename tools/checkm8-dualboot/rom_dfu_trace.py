"""
DEEP TRACE: DFU code path → find which callback is used for DFU_UPLOAD 
and whether it triggers ZLP via D3D0.

Also: Fix the scanning to use known code regions (not offset 0).
Also: Find ALL callers of usb_core_do_transfer (E0D8) and ep0_transfer_setup (D368).
Also: Trace the DFU request handler to understand UPLOAD/DOWNLOAD flow.
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

def disasm_at(offset, count=40):
    results = []
    for i in md.disasm(rom[offset:offset+count*4], ROM_BASE + offset):
        results.append(i)
    return results

# ============================================================
# PART 1: Scan KNOWN code regions for BL targets
# Code is roughly at 0x0000-0x25000, but with data gaps.
# Strategy: scan in 4KB chunks, skip chunks with too few valid instrs.
# ============================================================
print("=" * 70)
print("PART 1: Full code scan for BL to key targets")
print("=" * 70)

targets = {
    ROM_BASE + 0xE0D8: "usb_core_do_transfer",
    ROM_BASE + 0xD368: "ep0_transfer_setup",
    ROM_BASE + 0xD334: "zlp_send",
    ROM_BASE + 0xD2D0: "hal_abort_call",
    ROM_BASE + 0xE158: "ep_abort_wrapper",
    ROM_BASE + 0xC084: "dwc3_ep_abort",
    ROM_BASE + 0xE0A8: "dwc3_callback_and_free",
    ROM_BASE + 0xBE50: "dwc3_start_transfer",
}

# Scan entire active code region in 0x1000 chunks
all_calls = {t: [] for t in targets}

for chunk_start in range(0, ACTIVE_END, 0x1000):
    chunk_end = min(chunk_start + 0x1000, ACTIVE_END)
    instrs = list(md.disasm(rom[chunk_start:chunk_end], ROM_BASE + chunk_start))
    
    for i in instrs:
        if i.mnemonic == 'bl':
            try:
                t = int(i.op_str.lstrip('#'), 0)
                if t in targets:
                    all_calls[t].append(i.address)
            except:
                pass

for target_addr, name in targets.items():
    callers = all_calls[target_addr]
    print(f"\n  {name} (0x{target_addr:X}): {len(callers)} callers")
    for addr in callers:
        print(f"    → 0x{addr:X} (offset 0x{addr-ROM_BASE:X})")

# ============================================================
# PART 2: For each caller of usb_core_do_transfer, show context
# to identify callback argument (x3)
# ============================================================
print("\n" + "=" * 70)
print("PART 2: Callback arguments for usb_core_do_transfer callers")
print("=" * 70)

for addr in all_calls[ROM_BASE + 0xE0D8]:
    off = addr - ROM_BASE
    print(f"\n--- Caller at 0x{addr:X} ---")
    start = max(0, off - 80)
    context = disasm_range(start, off + 4)
    for ci in context[-20:]:
        ann = ""
        if ci.address == addr:
            ann = " ; ← BL usb_core_do_transfer"
        if ci.mnemonic in ('adrp', 'add', 'mov', 'ldr') and 'x3' in ci.op_str:
            ann += " ; ★ x3 (callback)"
        if ci.mnemonic == 'adrp':
            # Try to resolve page
            parts = ci.op_str.split(',')
            if len(parts) == 2:
                try:
                    page = int(parts[1].strip().lstrip('#'), 0)
                    ann += f" ; page=0x{page:X}"
                except: pass
        print(f"    0x{ci.address:X}: {ci.mnemonic:6s} {ci.op_str}{ann}")

# Also for ep0_transfer_setup callers (D368) — callback is x2
print("\n" + "=" * 70)
print("PART 2b: Callback arguments for ep0_transfer_setup callers")
print("=" * 70)

for addr in all_calls[ROM_BASE + 0xD368]:
    off = addr - ROM_BASE
    print(f"\n--- Caller at 0x{addr:X} ---")
    start = max(0, off - 80)
    context = disasm_range(start, off + 4)
    for ci in context[-20:]:
        ann = ""
        if ci.address == addr:
            ann = " ; ← BL ep0_transfer_setup"
        if ci.mnemonic in ('adrp', 'add', 'mov', 'ldr') and 'x2' in ci.op_str:
            ann += " ; ★ x2 (callback)"
        print(f"    0x{ci.address:X}: {ci.mnemonic:6s} {ci.op_str}{ann}")

# ============================================================
# PART 3: DFU handler functions
# Look for DFU class request dispatch (bRequest = 1=DNLOAD, 2=UPLOAD, etc.)
# DFU requests come via interface request handler
# ============================================================
print("\n" + "=" * 70)
print("PART 3: DFU request handlers — searching for DFU_UPLOAD path")
print("=" * 70)

# Known DFU-related functions from knowledge base:
# E72C = dfu_run/dfu_exit area
# D94C = ep0_handler
# gDFU state at USB_STATE+0x20 area or separate global

# Let's look at the standard_device_request_cb (D3D0) callers
# But also: DFU interface handler is likely registered via USB IF registration

# Scan for DFU class codes: bRequest values
# DFU_DETACH=0, DFU_DNLOAD=1, DFU_UPLOAD=2, DFU_GETSTATUS=3, 
# DFU_CLRSTATUS=4, DFU_GETSTATE=5, DFU_ABORT=6

# Let's look at functions near the DFU area
# E700-E800 is likely the DFU module
print("\n--- Disassembly around E700-E800 (DFU area) ---")
for i in disasm_range(0xE700, 0xE900):
    ann = ""
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            name = targets.get(t, "")
            if name: ann = f" ; → {name}"
            if t == ROM_BASE + 0xF3B0: ann = " ; → calloc"
            if t == ROM_BASE + 0xF1EC: ann = " ; → malloc"
            if t == ROM_BASE + 0xF680: ann = " ; → memalign"
            if t == ROM_BASE + 0xF468: ann = " ; → free"
            if t == ROM_BASE + 0x10D80: ann = " ; → bzero"
        except: pass
    if i.mnemonic == 'cmp' and '#' in i.op_str:
        # Check for DFU bRequest comparisons
        try:
            val = i.op_str.split('#')[1].strip()
            ival = int(val, 0)
            dfu_names = {0: "DFU_DETACH", 1: "DFU_DNLOAD", 2: "DFU_UPLOAD", 
                         3: "DFU_GETSTATUS", 4: "DFU_CLRSTATUS", 5: "DFU_GETSTATE", 6: "DFU_ABORT"}
            if ival in dfu_names:
                ann = f" ; ★ {dfu_names[ival]}"
        except: pass
    print(f"  0x{i.address:X}: {i.mnemonic:6s} {i.op_str}{ann}")

# ============================================================
# PART 4: Wider DFU scan — E500-EA00
# ============================================================
print("\n" + "=" * 70)
print("PART 4: Wide DFU scan — E500-EA00")
print("=" * 70)

for i in disasm_range(0xE500, 0xEA00):
    ann = ""
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            name = targets.get(t, "")
            if name: ann = f" ; → {name}"
            if t == ROM_BASE + 0xF3B0: ann = " ; → calloc"
            if t == ROM_BASE + 0xF680: ann = " ; → memalign"
            if t == ROM_BASE + 0xF468: ann = " ; → free"
            if t == ROM_BASE + 0xE0D8: ann = " ; → usb_core_do_transfer"
        except: pass
    if i.mnemonic == 'cmp' and '#' in i.op_str:
        try:
            val = i.op_str.split('#')[1].strip()
            ival = int(val, 0)
            dfu_names = {0: "DFU_DETACH", 1: "DFU_DNLOAD", 2: "DFU_UPLOAD", 
                         3: "DFU_GETSTATUS", 4: "DFU_CLRSTATUS", 5: "DFU_GETSTATE", 6: "DFU_ABORT"}
            if ival in dfu_names:
                ann = f" ; ★ {dfu_names[ival]}"
        except: pass
    # Mark function prologues
    if i.mnemonic == 'stp' and 'sp' in i.op_str and '!' in i.op_str:
        ann = " ; ← FUNCTION ENTRY"
    if i.mnemonic == 'ret':
        ann = " ; ← RET"
    # Mark ADRP
    if i.mnemonic == 'adrp':
        parts = i.op_str.split(',')
        if len(parts) == 2:
            try:
                page = int(parts[1].strip().lstrip('#'), 0)
                ann = f" ; page=0x{page:X}"
            except: pass
    # Mark STR with #0x28 (gDFU io_buffer pointer)
    if 'str' in i.mnemonic and '#0x28' in i.op_str:
        ann = " ; ★ store at +0x28 (io_buffer?)"
    if 'ldr' in i.mnemonic and '#0x28' in i.op_str:
        ann = " ; ★ load from +0x28 (io_buffer?)"
    print(f"  0x{i.address:X}: {i.mnemonic:6s} {i.op_str}{ann}")

# ============================================================
# PART 5: Who references D3D0? Scan by raw bytes
# ADRP+ADD encoding for D3D0:
# ADRP: immhi:immlo → page(D3D0) = 0x10000D000
# From various PC values, the ADRP encoding varies.
# Instead, let's scan for the 4-byte pattern of ADD x?, x?, #0x3D0
# ADD immediate: 1 0 0 1 0 0 01 shift imm12 Rn Rd
# #0x3D0 = 0b 1111_0100_00 = 0x3D0
# imm12 = 0x3D0, shift=0: bits[21:10] = 0x3D0 = 0b001111010000
# Encoding: 1001_0001_00 | 001111010000 | Rn(5) | Rd(5)
# = 0x91 0F40 | (Rn << 5) | Rd
# Byte pattern: 0x91 at top, then 0x0F4 in middle
# ============================================================
print("\n" + "=" * 70)
print("PART 5: Raw byte scan for ADD x?, x?, #0x3D0")
print("=" * 70)

for off in range(0, ACTIVE_END, 4):
    word = struct.unpack_from('<I', rom, off)[0]
    # ADD (immediate) 64-bit: 1001_0001_00xx_xxxx_xxxx_xxRn_nnRd_dddd
    # sf=1, op=0, S=0: 1001_0001
    # shift=00, imm12=0x3D0=0b001111010000
    # So: 0x910F4xxx where xxx = Rn:Rd
    if (word & 0xFFFFC000) == 0x910F4000:
        rd = word & 0x1F
        rn = (word >> 5) & 0x1F
        addr = ROM_BASE + off
        print(f"  0x{addr:X}: ADD x{rd}, x{rn}, #0x3D0")
        # Show next few instructions for context
        for ci in disasm_at(off, 8):
            print(f"    0x{ci.address:X}: {ci.mnemonic:6s} {ci.op_str}")

# Now also scan for ADRP that loads page 0x10000D000
print("\n--- Scanning for ADRP to page 0x10000D000 ---")
for off in range(0, ACTIVE_END, 4):
    word = struct.unpack_from('<I', rom, off)[0]
    # ADRP: 1 immlo(2) 10000 immhi(19) Rd(5)
    if (word & 0x9F000000) == 0x90000000:
        rd = word & 0x1F
        immlo = (word >> 29) & 0x3
        immhi = (word >> 5) & 0x7FFFF
        imm = (immhi << 2) | immlo
        # Sign extend 21-bit
        if imm & (1 << 20):
            imm -= (1 << 21)
        pc_page = (ROM_BASE + off) & ~0xFFF
        target_page = pc_page + (imm << 12)
        if target_page == 0x10000D000:
            addr = ROM_BASE + off
            # Check next instruction for ADD #0x3D0
            if off + 4 < len(rom):
                next_word = struct.unpack_from('<I', rom, off+4)[0]
                if (next_word & 0xFFFFC000) == 0x910F4000:
                    next_rn = (next_word >> 5) & 0x1F
                    next_rd = next_word & 0x1F
                    if next_rn == rd:
                        print(f"\n  ★ ADRP+ADD → 0x10000D3D0 at 0x{addr:X}:")
                        for ci in disasm_at(off, 12):
                            print(f"    0x{ci.address:X}: {ci.mnemonic:6s} {ci.op_str}")

# ============================================================
# PART 6: Heap zone sizes — read from ROM data at zone address offsets
# Zone at 0x19C011E88 has: +0xA10=lower, +0xA18=upper
# But these are runtime values (SRAM), not in ROM.
# Instead, look for heap_init function that sets up zones.
# Search for store to 0x19C011E88+0xA10 = 0x19C012898
# ============================================================
print("\n" + "=" * 70)
print("PART 6: Heap zone init — searching for zone setup code")
print("=" * 70)

# Find all ADRP to 0x19C012000 (page containing zone+0xA10=0x19C012898)
for off in range(0, ACTIVE_END, 4):
    word = struct.unpack_from('<I', rom, off)[0]
    if (word & 0x9F000000) == 0x90000000:
        rd = word & 0x1F
        immlo = (word >> 29) & 0x3
        immhi = (word >> 5) & 0x7FFFF
        imm = (immhi << 2) | immlo
        if imm & (1 << 20):
            imm -= (1 << 21)
        pc_page = (ROM_BASE + off) & ~0xFFF
        target_page = pc_page + (imm << 12)
        if target_page == 0x19C012000:
            addr = ROM_BASE + off
            print(f"  ADRP to 0x19C012000 at 0x{addr:X}")
            for ci in disasm_at(off, 6):
                print(f"    0x{ci.address:X}: {ci.mnemonic:6s} {ci.op_str}")

# Search for heap_create / heap_init — often near F000-F200 area
print("\n--- Heap init area (F000-F200) ---")
for i in disasm_range(0xF000, 0xF1EC):
    ann = ""
    if i.mnemonic == 'stp' and 'sp' in i.op_str and '!' in i.op_str:
        ann = " ; ← FUNCTION ENTRY"
    if i.mnemonic == 'ret':
        ann = " ; ← RET"
    if i.mnemonic == 'adrp':
        parts = i.op_str.split(',')
        if len(parts) == 2:
            try:
                page = int(parts[1].strip().lstrip('#'), 0)
                if page >= 0x19C000000:
                    ann = f" ; page=0x{page:X}"
            except: pass
    if i.mnemonic == 'bl':
        try:
            t = int(i.op_str.lstrip('#'), 0)
            if t == ROM_BASE + 0x10D80: ann = " ; → bzero"
            if t == ROM_BASE + 0x10E00: ann = " ; → memset"
        except: pass
    # mov with large immediate — could be heap size
    if i.mnemonic == 'mov' and '#0x' in i.op_str:
        try:
            val = int(i.op_str.split('#')[1].strip(), 0)
            if val > 0x1000:
                ann = f" ; ★ large immediate = {val} ({val:#x})"
        except: pass
    print(f"  0x{i.address:X}: {i.mnemonic:6s} {i.op_str}{ann}")

print("\n\nDone.")
