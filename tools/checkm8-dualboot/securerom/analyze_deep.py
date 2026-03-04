#!/usr/bin/env python3
"""
SecureROM T8020 B1 — Deep-dive analysis of callback dispatch & USB functions
This script disassembles around the key addresses found in analyze_v2.py
to map T8020 equivalents of known T8010 exploit primitives.

Key findings from v2:
  Callback dispatch: LDP x8,x9,[x0,#0x70] at 0x10000A444 and 0x10000B9F0
  nop_gadget equivalent: LDP x29,x30,[sp,#0x10]; LDP x20,x19,[sp],#0x20; RET → 96 instances, first at 0x100002BA0
  VID 0x05AC: MOV w0,#0x5AC at 0x100011F14
  dmb; ret: 0x10000053C
  MSR TTBR0_EL1: 0x1000004A8
  MSR SCTLR_EL1: 0x10000044C, 0x10000046C
  Most referenced SRAM: 0x19C010B20 (23x), 0x19C010670 (16x), 0x19C010A90 (14x)
  0x800 refs: 0x10000E314, 0x10000E324
"""

import struct, os
from collections import defaultdict
from capstone import *

ROM_PATH = os.path.join(os.path.dirname(__file__), "t8020_B1_securerom.bin")
BASE_ADDR = 0x100000000

with open(ROM_PATH, "rb") as f:
    rom = f.read()

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

def disasm_range(start_addr, length=0x100):
    """Disassemble a range and return list of (addr, mnemonic, op_str)"""
    offset = start_addr - BASE_ADDR
    if offset < 0 or offset >= len(rom):
        return []
    data = rom[offset:offset+length]
    result = []
    for i in md.disasm(data, start_addr):
        result.append((i.address, i.mnemonic, i.op_str))
    return result

def print_disasm(addr, length=0x100, label=""):
    """Pretty-print disassembly"""
    if label:
        print(f"\n{'='*70}")
        print(f"  {label}")
        print(f"  Address: 0x{addr:011X}")
        print(f"{'='*70}")
    for a, m, o in disasm_range(addr, length):
        print(f"  0x{a:011X}: {m:10s} {o}")

def find_function_start(addr):
    """Walk backwards from addr to find function prologue (STP x29,x30,...)"""
    for off in range(0, 0x200, 4):
        check = addr - off
        instrs = disasm_range(check, 4)
        if instrs:
            m, o = instrs[0][1], instrs[0][2]
            # STP x29, x30, [sp, #-N]! is the standard prologue
            if m == 'stp' and 'x29, x30' in o and 'sp' in o and '!' in o:
                return check
    return addr

def resolve_adrp_add(addr):
    """Resolve ADRP+ADD pair at addr to full address"""
    offset = addr - BASE_ADDR
    if offset < 0 or offset + 8 > len(rom):
        return None
    
    instr1 = struct.unpack_from("<I", rom, offset)[0]
    instr2 = struct.unpack_from("<I", rom, offset + 4)[0]
    
    # ADRP
    if (instr1 & 0x9F000000) != 0x90000000:
        return None
    immhi = (instr1 >> 5) & 0x7FFFF
    immlo = (instr1 >> 29) & 0x3
    imm = (immhi << 2) | immlo
    if imm & 0x100000:
        imm = imm - 0x200000
    page = (addr & ~0xFFF) + (imm << 12)
    
    # ADD
    if (instr2 & 0xFFC00000) != 0x91000000:
        return page
    add_imm = (instr2 >> 10) & 0xFFF
    shift = (instr2 >> 22) & 0x3
    if shift == 1: add_imm <<= 12
    return page + add_imm

# Build full instruction map for cross-referencing
print("Building full instruction map...")
all_instrs = {}
for offset in range(0, min(0x20000, len(rom)), 4):
    addr = BASE_ADDR + offset
    for i in md.disasm(rom[offset:offset+4], addr):
        all_instrs[addr] = (i.mnemonic, i.op_str)

print(f"Mapped {len(all_instrs)} instructions in code region (0x00000 - 0x20000)\n")

# ============================================================
# ANALYSIS 1: Callback dispatch functions
# ============================================================
print("="*70)
print("ANALYSIS 1: Callback Dispatch Functions")
print("="*70)
print("""
On T8010, the func_gadget at 0x10000CC4C does:
  LDP X8, X10, [X0, #0x70]   ; load callback + next from io_request
  MOV X0, X8                  ; argument = callback data
  BLR X10                     ; call callback

T8020 uses (x8, x9) instead of (x8, x10). Let's examine both sites.
""")

# Site 1: 0x10000A444
func_start_1 = find_function_start(0x10000A444)
print_disasm(func_start_1, 0x120, "CALLBACK SITE 1 — Function containing LDP x8,x9,[x0,#0x70] at 0x10000A444")

# Site 2: 0x10000B9F0
func_start_2 = find_function_start(0x10000B9F0)
print_disasm(func_start_2, 0x120, "CALLBACK SITE 2 — Function containing LDP x8,x9,[x0,#0x70] at 0x10000B9F0")

# Site 3: LDP x16,x17,[x0,#0x78] at 0x10000AAAC
func_start_3 = find_function_start(0x10000AAAC)
print_disasm(func_start_3, 0x120, "CALLBACK SITE 3 — Function containing LDP x16,x17,[x0,#0x78] at 0x10000AAAC")

# ============================================================
# ANALYSIS 2: USB initialization (from VID 0x05AC)
# ============================================================
print("\n" + "="*70)
print("ANALYSIS 2: USB Initialization Chain (VID 0x05AC at 0x100011F14)")
print("="*70)

# The MOV w0, #0x5AC is at 0x100011F14. Find the function containing it.
usb_vid_func = find_function_start(0x100011F14)
print_disasm(usb_vid_func, 0x200, "USB VID function (sets Apple vendor ID)")

# ============================================================
# ANALYSIS 3: USB Serial Number construction
# ============================================================
print("\n" + "="*70)
print("ANALYSIS 3: DFU Serial Number Construction")
print("="*70)
print("String 'CPID:%04X...' at 0x10001C279")

# Find all ADRP that reference the page containing 0x10001C279
target_page = 0x10001C000
adrp_refs = []
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    instr = struct.unpack_from("<I", rom, offset)[0]
    if (instr & 0x9F000000) == 0x90000000:
        immhi = (instr >> 5) & 0x7FFFF
        immlo = (instr >> 29) & 0x3
        imm = (immhi << 2) | immlo
        if imm & 0x100000:
            imm = imm - 0x200000
        page = (addr & ~0xFFF) + (imm << 12)
        if page == target_page:
            # Check next instruction for ADD with offset 0x279
            resolved = resolve_adrp_add(addr)
            if resolved and 0x10001C200 <= resolved <= 0x10001C300:
                adrp_refs.append((addr, resolved))

print(f"\nCode referencing strings at 0x10001C2xx:")
for code_addr, target in adrp_refs:
    func_start = find_function_start(code_addr)
    print(f"  0x{code_addr:011X} → 0x{target:011X} (function at 0x{func_start:011X})")

# Show the serial number construction function
if adrp_refs:
    # The one referencing 0x10001C279 (CPID format) is the serial number builder
    for code_addr, target in adrp_refs:
        if 0x10001C270 <= target <= 0x10001C280:
            func = find_function_start(code_addr)
            print_disasm(func, 0x300, f"Serial number builder (ref CPID format @ 0x{code_addr:011X})")
            break

# ============================================================
# ANALYSIS 4: DFU mode handler (0x800 buffer)
# ============================================================
print("\n" + "="*70)
print("ANALYSIS 4: DFU Transfer Handlers (0x800 buffer size)")
print("="*70)

# Sites referencing 0x800: 0x10000E314, 0x10000E324
for site in [0x10000E314, 0x10000E324, 0x100006318, 0x10000B120]:
    func = find_function_start(site)
    print_disasm(func, 0x200, f"Function at 0x{func:011X} (ref 0x800 at 0x{site:011X})")

# ============================================================
# ANALYSIS 5: Key SRAM global variables  
# ============================================================
print("\n" + "="*70)
print("ANALYSIS 5: Top SRAM Global Variables")
print("="*70)

# For each top SRAM variable, find what code references it
top_sram = [
    (0x19C010B20, 23, "gUSB_state? (most referenced)"),
    (0x19C010670, 16, "gUSB_descriptors?"),
    (0x19C010A90, 14, "gDFU_state?"),
    (0x19C00C1B0, 12, "gUSB_config?"),
    (0x19C010BE0, 12, "gUSB_ep_state?"),
    (0x19C008000,  8, "heap_base?"),
    (0x19C00BBC0,  6, "global_1"),
    (0x19C00BBF0,  6, "global_2"),
    (0x19C00BC20,  6, "global_3"),
    (0x19C010630,  6, "global_4"),
    (0x19C008B48,  5, "global_5"),
    (0x19C010B18,  5, "global_6"),
]

for sram_addr, refcount, hypothesis in top_sram:
    print(f"\n--- 0x{sram_addr:011X} ({hypothesis}, {refcount} refs) ---")
    
    # Find all ADRP+ADD pairs that resolve to this address
    target_page = sram_addr & ~0xFFF
    refs = []
    for offset in range(0, 0x20000, 4):
        addr = BASE_ADDR + offset
        resolved = resolve_adrp_add(addr)
        if resolved == sram_addr:
            refs.append(addr)
    
    for ref_addr in refs[:3]:
        # Show context around the reference
        instrs = disasm_range(ref_addr - 8, 0x20)
        for a, m, o in instrs:
            marker = " >>>" if a == ref_addr else "    "
            print(f"  {marker} 0x{a:011X}: {m:10s} {o}")
        print()

# ============================================================
# ANALYSIS 6: compare io_request struct offsets
# ============================================================
print("\n" + "="*70)
print("ANALYSIS 6: io_request struct layout analysis")
print("="*70)
print("""
T8010 io_request struct (from leaked iBoot source + RE):
  +0x00: next         (linked list)
  +0x08: prev
  +0x10: endpoint
  +0x18: io_buffer
  +0x20: io_length
  +0x28: status
  +0x30: ...
  +0x70: callback     (BLR target)
  +0x78: callback_arg (or next in chain)

We need to find T8020's equivalent offsets by looking at how the 
callback dispatch loads from the io_request struct.
""")

# Print all LDP/LDR instructions that use [x0, #offset] near the callback sites
print("All struct field accesses via X0 near callback dispatch sites:")
for site_name, site_addr in [("Site1", 0x10000A444), ("Site2", 0x10000B9F0), ("Site3", 0x10000AAAC)]:
    func = find_function_start(site_addr)
    instrs = disasm_range(func, 0x200)
    print(f"\n  --- {site_name} (func at 0x{func:011X}) ---")
    for a, m, o in instrs:
        if m in ('ldr', 'ldp', 'str', 'stp') and 'x0' in o and '#' in o:
            print(f"    0x{a:011X}: {m:10s} {o}")
        if m in ('ldr', 'ldp', 'str', 'stp') and 'x19' in o and '#' in o:
            print(f"    0x{a:011X}: {m:10s} {o}")
        if m in ('ldr', 'ldp', 'str', 'stp') and 'x20' in o and '#' in o:
            print(f"    0x{a:011X}: {m:10s} {o}")

# ============================================================
# ANALYSIS 7: System bootstrap (exception vectors, VBAR)
# ============================================================
print("\n" + "="*70)
print("ANALYSIS 7: Exception Vectors & Bootstrap")
print("="*70)

# At 0x100000048: MSR VBAR_EL1, X10 — this is the very beginning of the ROM
print_disasm(BASE_ADDR, 0x100, "ROM Entry Point (0x100000000)")

# SCTLR_EL1 access at 0x10000044C
func_sctlr = find_function_start(0x10000044C)
print_disasm(func_sctlr, 0x80, "WXN/SCTLR Config (contains MSR SCTLR_EL1 at 0x10000044C)")

# Second SCTLR_EL1 at 0x10000046C
print_disasm(0x100000460, 0x40, "Second SCTLR_EL1 write at 0x10000046C")

# ============================================================
# ANALYSIS 8: Search for heap management functions
# ============================================================
print("\n" + "="*70)
print("ANALYSIS 8: Heap Management (malloc/free candidates)")
print("="*70)

# The exploit relies on heap feng-shui. We need to find:
# - malloc (heap_alloc): takes size, returns pointer  
# - free (heap_free): takes pointer
# - The T8010 heap is a simple free-list allocator

# malloc candidates: functions called with a size argument that return a pointer
# free candidates: functions called with a pointer argument, no return value
# Most-called functions are likely utilities; malloc/free would be called moderately

# Look at 0x100008978 (most called, 136x)
func_top1 = find_function_start(0x100008978)
print_disasm(0x100008978, 0x80, "Top function #1 (0x100008978, called 136x)")

# Look at 0x100010BD0 (76x)
print_disasm(0x100010BD0, 0x80, "Top function #2 (0x100010BD0, called 76x)")

# Look at 0x100008B58 (69x)
print_disasm(0x100008B58, 0x80, "Top function #3 (0x100008B58, called 69x)")

# 0x100011C70 (39x) — might be USB related
func_usb = find_function_start(0x100011C70)
print_disasm(0x100011C70, 0xC0, "Function #5 (0x100011C70, called 39x)")

# ============================================================
# ANALYSIS 9: Look for the USB task loop
# ============================================================
print("\n" + "="*70)
print("ANALYSIS 9: Task/Event Loop")
print("="*70)

# "idle task" at 0x10001C312, "bootstrap" at 0x100024818
# Find code referencing "idle task"
idle_page = 0x10001C000
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    resolved = resolve_adrp_add(addr)
    if resolved and 0x10001C310 <= resolved <= 0x10001C31F:
        func = find_function_start(addr)
        print(f"\n'idle task' referenced at 0x{addr:011X} (function: 0x{func:011X})")
        print_disasm(func, 0x100, f"Idle task setup function")
        break

# "bootstrap" at 0x100024818 — but this is in the 0x24000 range which was mostly zeros
# It might be data, not code. Let's check what's around it.
print_disasm(0x100024800, 0x60, "Area around 'bootstrap' string (0x100024818)")

# ============================================================
# ANALYSIS 10: Find USB_CORE_DO_IO equivalent
# ============================================================
print("\n" + "="*70)
print("ANALYSIS 10: USB_CORE_DO_IO Identification")
print("="*70)
print("""
USB_CORE_DO_IO in T8010 is at 0x10000DC98.
It sets up an io_request for a USB transfer.
Key signature: 
  - Takes endpoint + buffer + length + callback
  - References gUSBDescriptors/gUSBSerialNumber globals
  - Writes to io_request at offsets 0x18, 0x20, 0x70, 0x78
""")

# Look for functions that write to struct offsets 0x70 and 0x78
# (callback + callback_arg in io_request)
print("Functions that store to [Xn, #0x70] and [Xn, #0x78]:")
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    if addr in all_instrs:
        m, o = all_instrs[addr]
        if m == 'str' and '#0x70' in o:
            func = find_function_start(addr)
            print(f"  STR to #0x70 at 0x{addr:011X} (func: 0x{func:011X})")
        if m == 'stp' and '#0x70' in o:
            func = find_function_start(addr)
            print(f"  STP to #0x70 at 0x{addr:011X} (func: 0x{func:011X})")

# ============================================================
# FINAL: Compile T8020 gadget addresses
# ============================================================
print("\n" + "="*70)
print("FINAL: T8020 GADGET SUMMARY")
print("="*70)

gadgets = {
    "nop_gadget":       "0x100002BA0  (ldp x29,x30,[sp,#0x10]; ldp x20,x19,[sp],#0x20; ret) — 96 instances",
    "dmb_ret":          "0x10000053C  (dmb sy; ret)",
    "write_ttbr0":      "0x1000004A8  (msr ttbr0_el1, x0)",
    "write_sctlr":      "0x10000044C  (msr sctlr_el1, x0) — for WXN disable",
    "write_vbar":       "0x100000048  (msr vbar_el1, x10)",
    "enter_critical":   "0x100000548  (msr daifset, #3)",
    "exit_critical":    "0x1000003A8  (msr daifclr, #4)",
    "stack_pivot":      "0x100011130  (mov sp, x9; ret)",
    "mov_x0_0_ret":     "0x100005470  (mov x0, #0; ret) — 10 instances",
    "mov_x0_x8_ret":    "0x10000B4B4  (mov x0, x8; ret) - 5 instances",  
    "str_x1_x0_ret":    "0x100009860  (str x1, [x0]; ret) — arbitrary write",
    "stp_xzr_x0_ret":   "0x100009538  (stp xzr, xzr, [x0]; ret) — zero memory",
    "callback_dispatch_1": "0x10000A444  (ldp x8, x9, [x0, #0x70]) — usb_core_complete?",
    "callback_dispatch_2": "0x10000B9F0  (ldp x8, x9, [x0, #0x70]) — usb_ep_complete?",
    "VID_setup":        "0x100011F14  (mov w0, #0x5ac)",
}

for name, info in gadgets.items():
    print(f"  {name:25s}: {info}")

print("\nSRAM Globals (candidates for gUSBDescriptors/gUSBSerialNumber):")
sram_candidates = [
    (0x19C010B20, 23, "gUSB_state?"),
    (0x19C010670, 16, "gUSB_descriptors?"),
    (0x19C010A90, 14, "gDFU_state?"),
    (0x19C00C1B0, 12, "gUSB_config?"),
    (0x19C010BE0, 12, "gUSB_endpoint?"),
    (0x19C008000,  8, "heap_base?"),
]
for addr, refs, hyp in sram_candidates:
    print(f"  0x{addr:011X}: {refs:3d} refs — {hyp}")

print("\n\nDONE — Deep analysis complete.")
