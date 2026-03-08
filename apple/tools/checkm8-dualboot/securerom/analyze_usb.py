#!/usr/bin/env python3
"""
SecureROM T8020 B1 — USB_CORE_DO_IO and key function identification
Traces the USB transfer path: VID setup → serial number → io_request → callback
"""

import struct, os
from capstone import *

ROM_PATH = os.path.join(os.path.dirname(__file__), "t8020_B1_securerom.bin")
BASE_ADDR = 0x100000000

with open(ROM_PATH, "rb") as f:
    rom = f.read()

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

def disasm_range(start_addr, length=0x100):
    offset = start_addr - BASE_ADDR
    if offset < 0 or offset >= len(rom): return []
    data = rom[offset:offset+length]
    return [(i.address, i.mnemonic, i.op_str) for i in md.disasm(data, start_addr)]

def find_function_start(addr):
    for off in range(0, 0x400, 4):
        check = addr - off
        instrs = disasm_range(check, 4)
        if instrs and instrs[0][1] == 'stp' and 'x29, x30' in instrs[0][2] and '!' in instrs[0][2]:
            return check
        # Also check SUB SP pattern (different prologue style)
        if instrs and instrs[0][1] == 'sub' and 'sp, sp' in instrs[0][2]:
            return check
    return addr

def print_func(addr, length=0x200, label=""):
    """Print full function from prologue"""
    func_start = find_function_start(addr)
    if label:
        print(f"\n{'='*70}")
        print(f"  {label}")
        print(f"  Function: 0x{func_start:011X} (target: 0x{addr:011X})")
        print(f"{'='*70}")
    for a, m, o in disasm_range(func_start, length):
        marker = " >>>" if a == addr else "    "
        print(f"  {marker} 0x{a:011X}: {m:10s} {o}")
        if m == 'ret' and a > addr + 4:
            break  # Stop after first ret past our target

def resolve_adrp_add(addr):
    offset = addr - BASE_ADDR
    if offset < 0 or offset + 8 > len(rom): return None
    instr1 = struct.unpack_from("<I", rom, offset)[0]
    instr2 = struct.unpack_from("<I", rom, offset + 4)[0]
    if (instr1 & 0x9F000000) != 0x90000000: return None
    immhi = (instr1 >> 5) & 0x7FFFF
    immlo = (instr1 >> 29) & 0x3
    imm = (immhi << 2) | immlo
    if imm & 0x100000: imm = imm - 0x200000
    page = (addr & ~0xFFF) + (imm << 12)
    if (instr2 & 0xFFC00000) != 0x91000000: return page
    add_imm = (instr2 >> 10) & 0xFFF
    shift = (instr2 >> 22) & 0x3
    if shift == 1: add_imm <<= 12
    return page + add_imm

# Build BL call graph
bl_graph = {}  # caller -> set of callees
bl_callers = {}  # callee -> set of callers
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    instrs = disasm_range(addr, 4)
    if instrs and instrs[0][1] == 'bl':
        try:
            target = int(instrs[0][2].replace('#', ''), 16)
            bl_graph.setdefault(addr, set()).add(target)
            bl_callers.setdefault(target, set()).add(addr)
        except: pass

# ============================================================
# 1. VID setup function at 0x100011F14 — trace the USB init chain
# ============================================================
print("="*70)
print("1. USB INITIALIZATION CHAIN")
print("="*70)

# 0x100011F14: mov w0, #0x5ac — this is called from where?
vid_func = 0x100011F14
callers = bl_callers.get(vid_func, set())
print(f"\n0x100011F14 (VID setup) called from: {[hex(c) for c in sorted(callers)]}")

# The surrounding function at 0x100011F14:
print_func(0x100011F14, 0x100, "VID setup function (0x100011F14)")

# From deep analysis, at 0x10000D440, call to 0x100011F14 happens right after
# storing to gUSBDeviceState (0x19C010B20+0x30). Let's trace that initialization.
print_func(0x10000D42C, 0x200, "USB Device Init (calls VID setup + serial builder)")

# ============================================================
# 2. Functions that STP to #0x70 — candidates for USB_CORE_DO_IO
# ============================================================
print("\n" + "="*70)
print("2. USB_CORE_DO_IO CANDIDATES (STP to io_request+0x70)")
print("="*70)

# From the v2 analysis, several functions STP to #0x70.
# USB_CORE_DO_IO signature:
# - Takes (endpoint, buffer, length, direction, callback, callback_arg) 
# - Allocates or fills an io_request struct
# - Stores callback at +0x70, callback_arg at +0x78
# - Queues the io_request
# - Is called from USB descriptor handlers and DFU code

# Best candidates are in the USB code area (0x10000D000-0x10000E000)
# and the descriptor handler area (0x10000B000-0x10000C000)

interesting_stp_sites = [
    0x10000D5EC,  # In USB area
    0x10000E250,  # Near DFU endpoint code (0x19C010BE0)
    0x10000E3AC,  # Near 0x800 reference
    0x10000B640,  # In USB descriptor area
    0x10000C3BC,  # 
]

for site in interesting_stp_sites:
    print_func(site, 0x180, f"STP #0x70 at 0x{site:011X}")

# ============================================================
# 3. The DFU DNLOAD handler (receives firmware data)
# ============================================================
print("\n" + "="*70)
print("3. DFU DOWNLOAD HANDLER (0x800 buffer)")
print("="*70)

# 0x10000E314: mov w0, #0x800 and 0x10000E324: mov w1, #0x800
# This is in the function using gUSBEndpointState (0x19C010BE0)
print_func(0x10000E2F0, 0x200, "DFU endpoint handler (refs 0x800 buffer size)")

# ============================================================
# 4. Callback dispatch chain
# ============================================================
print("\n" + "="*70)
print("4. CALLBACK DISPATCH (usb_core_complete_endpoint_io)")
print("="*70)

# Site 1 at 0x10000A444: ldp x8, x9, [x0, #0x70]
# This loads callback (x8) and callback_arg (x9) from io_request
# Then what? Let's see the full sequence:
print_func(0x10000A444, 0x60, "Callback dispatch Site 1")

# Site 2 at 0x10000B9F0: ldp x8, x9, [x0, #0x70]  
print_func(0x10000B9F0, 0x60, "Callback dispatch Site 2")

# ============================================================
# 5. Memory allocator (malloc/free)
# ============================================================
print("\n" + "="*70)
print("5. MEMORY ALLOCATOR")
print("="*70)

# 0x10000F1EC is called 14x — called with size arg, returns pointer
# Let's check 0x10000F1EC and related functions
print_func(0x10000F1EC, 0x180, "Heap alloc candidate (0x10000F1EC, 14 calls)")

# 0x10000F3B0 is called from USB code at 0x10000B2DC with args (1, size, 0)
print_func(0x10000F3B0, 0x180, "Heap alloc candidate #2 (0x10000F3B0)")

# 0x10000F468 is called 18x
print_func(0x10000F468, 0x180, "Heap free candidate (0x10000F468, 18 calls)")

# ============================================================
# 6. usb_create_string_descriptor
# ============================================================
print("\n" + "="*70)
print("6. USB STRING DESCRIPTOR CREATION")
print("="*70)

# On T8010: usb_create_string_descriptor at 0x10000D150
# This function is called to build the serial number string descriptor
# It references gUSBSerialNumber and writes to gUSBDescriptors

# The serial number format string "CPID:%04X..." is at 0x10001C279
# Find all code that references it
print("Finding references to CPID format string (0x10001C279)...")
refs = []
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    resolved = resolve_adrp_add(addr)
    if resolved and 0x10001C270 <= resolved <= 0x10001C280:
        refs.append(addr)

for ref in refs:
    print(f"  Referenced at 0x{ref:011X}")
    print_func(ref, 0x200, f"Serial number builder (ref at 0x{ref:011X})")

# ============================================================
# 7. gUSBSerialNumber identification
# ============================================================
print("\n" + "="*70)
print("7. gUSBSerialNumber IDENTIFICATION")
print("="*70)

# The serial number string is built into a buffer pointed to by gUSBSerialNumber
# On T8010: gUSBSerialNumber = 0x180083CF8
# On T8020: SRAM starts at 0x19C000000
# The serial builder will likely: ADRP+ADD to load gUSBSerialNumber address,
# then use it as the output buffer for snprintf of the CPID format string

# We already found gUSBDeviceState at 0x19C010B20
# Let's look at what's stored at 0x19C010B20+0x38 (referenced at 0x10000D3B0):
# "ldr x8, [x8, #0x38]; str x8, [x0, #8]"
# This loads gUSBDeviceState.serial_string_buffer and stores it

# Also: 0x19C0088F0 (referenced at 0x10000D444+area)
# From earlier: adrp x22, #0x19c008000; add x22, x22, #0x8f0
print_func(0x10000D444, 0x80, "Post-VID: loads 0x19C0088F0 (gUSBSerialNumber?)")

# ============================================================
# 8. Full USB init function
# ============================================================
print("\n" + "="*70)
print("8. FULL USB INIT (0x10000D42C area)")
print("="*70)

# The function at around 0x10000D42C:
# BL 0x1000063D4 (some init)
# BL 0x10000D514 (USB controller init?)
# ADRP+ADD x21, 0x19C010B20 (gUSBDeviceState)
# STR x0, [x21, #0x30] (store something at gUSBDeviceState+0x30)
# BL 0x100011F14 (VID setup)
# ADRP+ADD x22, 0x19C0088F0 (serial number buffer)

func_usb_init = find_function_start(0x10000D42C)
print_func(func_usb_init, 0x400, f"USB init function (0x{func_usb_init:011X})")

# ============================================================
# 9. DFU state machine
# ============================================================
print("\n" + "="*70)
print("9. DFU STATE MACHINE (gDFU_state at 0x19C010A90)")
print("="*70)

# 0x19C010A90 is 0x78 bytes, has a callback at +0x08
# The callback dispatch at 0x10000D128: ldr x0, [x8, #8]; cbz x0; br x0
# This loads a function pointer from gDFU+8 and jumps to it

# Find the DFU state machine setup
print_func(0x10000D0A0, 0x100, "DFU handler setup (copies 0x78 bytes to gDFU_state)")

# ============================================================
# 10. handle_interface_request (USB control transfer dispatcher)  
# ============================================================
print("\n" + "="*70)
print("10. USB CONTROL TRANSFER HANDLER")
print("="*70)

# On T8010, handle_interface_request dispatches GET_DESCRIPTOR, SET_INTERFACE etc.
# It's typically the function that processes bRequest from the setup packet.
# In the DFU exploit, the stall is triggered by GET_DESCRIPTOR(STRING, idx>4)
# which goes through this path.

# The global at 0x19C00BC20 (global_3, 6 refs) has function pointers at +0x10, +0x18, +0x20
# that are loaded and called via BLR x8 — this is the interface handler table!
print_func(0x10000D100, 0x100, "DFU callback dispatch (0x19C010A90+0x08 → BR X0)")

# Find the setup packet parser — it will reference bRequest values like 0x06 (GET_DESCRIPTOR)
# and wValue fields
print("\nSearching for GET_DESCRIPTOR (bRequest=0x06) handler...")
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    instrs = disasm_range(addr, 4)
    if instrs:
        m, o = instrs[0][1], instrs[0][2]
        if m == 'cmp' and '#6' in o and 'w' in o:
            # Check if this is in a USB context (near USB code)
            if 0x10000C000 <= addr <= 0x10000F000:
                print(f"  CMP w?,#6 at 0x{addr:011X}")
                print_func(addr, 0x100, f"Possible GET_DESCRIPTOR check at 0x{addr:011X}")

print("\n\nDONE - USB function tracing complete.")
