#!/usr/bin/env python3
"""
Deep DFU Analysis — T8020 B1 SecureROM
Answers specific questions about:
  A) Oversized DNLOAD path (wLength > 0x800)
  B) Dangling pointer after USB reset
  C) DFU state machine transitions  
  D) Heap layout around io_buffer
  E) Alternative data injection vectors
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
    return addr

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

def print_func_full(addr, max_len=0x400, label=""):
    """Print a complete function from its start to RET"""
    func_start = find_function_start(addr)
    if label:
        print(f"\n{'='*80}")
        print(f"  {label}")
        print(f"  Function start: 0x{func_start:011X}")
        print(f"{'='*80}")
    ret_count = 0
    for a, m, o in disasm_range(func_start, max_len):
        marker = " >>>" if a == addr else "    "
        # Resolve ADRP+ADD inline
        resolved = resolve_adrp_add(a)
        extra = ""
        if resolved and resolved != a:
            extra = f"  ; → 0x{resolved:011X}"
        # Resolve BL targets
        if m == 'bl':
            try:
                target = int(o.replace('#', ''), 16)
                extra = f"  ; call 0x{target:011X}"
            except: pass
        print(f"  {marker} 0x{a:011X}: {m:10s} {o:40s}{extra}")
        if m == 'ret':
            ret_count += 1
            if ret_count >= 1 and a > func_start + 0x10:
                break

# Build instruction map
print("Building instruction map...")
all_instrs = {}
for offset in range(0, min(0x20000, len(rom)), 4):
    addr = BASE_ADDR + offset
    for i in md.disasm(rom[offset:offset+4], addr):
        all_instrs[addr] = (i.mnemonic, i.op_str)

# Build BL call graph
bl_callers = defaultdict(set)
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    if addr in all_instrs:
        m, o = all_instrs[addr]
        if m == 'bl':
            try:
                target = int(o.replace('#', ''), 16)
                bl_callers[target].add(addr)
            except: pass

print(f"Mapped {len(all_instrs)} instructions\n")

# ============================================================
# SECTION A: DFU DNLOAD REQUEST HANDLER — OVERSIZED PATH
# ============================================================
print("="*80)
print("SECTION A: DFU REQUEST HANDLER (0x10000E3EC) — OVERSIZED DNLOAD PATH")
print("="*80)
print("""
The DFU request handler at 0x10000E3EC dispatches based on bRequest:
  bRequest=1 → DNLOAD
  bRequest=3 → GETSTATUS
  bRequest=4 → CLRSTATUS
  bRequest=5 → GETSTATE
  bRequest=6 → ABORT

KEY QUESTION: When bRequest=1 (DNLOAD), does the handler:
  (a) Check wLength against io_buffer size (0x800)?
  (b) Set expectedLength = wLength WITHOUT checking?
  
If (b), then handle_ep0_data_phase writes wLength bytes into 0x800 buffer = HEAP OVERFLOW
""")

# Print the FULL DFU request handler
print_func_full(0x10000E3EC, 0x500, "dfu_request_handler (0x10000E3EC)")

# Now print the DFU init - shows buffer allocation
print_func_full(0x10000E2D0, 0x200, "dfu_init (0x10000E2D0) — allocates io_buffer")

# Print DFU completion callback
print_func_full(0x10000E708, 0x300, "dfu_completion_callback (0x10000E708)")

# ============================================================
# SECTION B: handle_ep0_data_phase — WHERE OVERFLOW HAPPENS
# ============================================================
print("\n" + "="*80)
print("SECTION B: USB EP0 DATA PHASE HANDLER")
print("="*80)
print("""
After SETUP phase accepts DNLOAD, the DATA phase writes to 
ep0DataPhaseBuffer (= io_buffer). If wLength > 0x800, this is where
the overflow ACTUALLY happens. Need to find this function.
""")

# Find references to 0x19C010BE0 (gDFU_interface) to trace buffer usage
# gDFU_interface+0x28 = DFU download buffer ptr
print("--- References to gDFU_interface (0x19C010BE0) ---")
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    resolved = resolve_adrp_add(addr)
    if resolved == 0x19C010BE0:
        func = find_function_start(addr)
        print(f"  0x{addr:011X} (in func 0x{func:011X})")

# The USB core do_io function sets up EP0 data transfer
print_func_full(0x10000B558, 0x400, "usb_core_do_io (0x10000B558) — EP0 transfer setup")

# EP0 completion handler
print_func_full(0x10000B858, 0x400, "usb_core_complete_ep_io (0x10000B858)")

# ============================================================
# SECTION C: handle_interface_request — SETUP PACKET PARSING
# ============================================================
print("\n" + "="*80)
print("SECTION C: USB SETUP PACKET / INTERFACE REQUEST HANDLER")
print("="*80)
print("""
The USB setup packet has:
  bmRequestType, bRequest, wValue, wIndex, wLength
  
For DFU DNLOAD: bmRequestType=0x21, bRequest=1, wValue=blockNum, wIndex=0, wLength=size

The interface request handler parses the setup packet and dispatches to
the registered interface handler (DFU request handler at 0x10000E3EC).

CRITICAL: Does this handler pass wLength to the DFU handler? Does the
DFU handler check wLength before calling usb_core_do_io?
""")

# The function at the USB interface handler vtable (0x19C00BC20)
# +0x10, +0x18, +0x20 are handler function pointers
print("--- USB interface handler vtable (0x19C00BC20) references ---")
for offset in range(0, 0x20000, 4):
    addr = BASE_ADDR + offset
    resolved = resolve_adrp_add(addr)
    if resolved == 0x19C00BC20:
        func = find_function_start(addr)
        print(f"  0x{addr:011X} (in func 0x{func:011X})")

# Find the USB control transfer dispatcher
# This would reference bRequestType and dispatch to interface handlers
# Look for CMP with 0x21 (class-specific interface request)
print("\n--- Searching for bRequestType dispatch (CMP #0x21) ---")
for addr, (m, o) in sorted(all_instrs.items()):
    if m == 'cmp' and '#0x21' in o:
        if 0x10000C000 <= addr <= 0x10000F000:
            func = find_function_start(addr)
            print(f"  0x{addr:011X}: {m} {o} (func 0x{func:011X})")

# Look for CMP with 0xA1 (class-specific interface IN request)
print("\n--- Searching for bRequestType dispatch (CMP #0xa1) ---")
for addr, (m, o) in sorted(all_instrs.items()):
    if m == 'cmp' and ('#0xa1' in o.lower() or '#0xa1' in o):
        if 0x10000C000 <= addr <= 0x10000F000:
            func = find_function_start(addr)
            print(f"  0x{addr:011X}: {m} {o} (func 0x{func:011X})")

# Main USB request handler — likely called from USB IRQ
# Look for the function that does switch(bRequest) with values 0-6
print("\n--- USB request dispatch functions ---")
# CMP #0x01 (GET_DESCRIPTOR or DNLOAD)
for addr, (m, o) in sorted(all_instrs.items()):
    if m == 'cmp' and '#1' in o and 'w' in o:
        next_addr = addr + 4
        if next_addr in all_instrs:
            nm, no = all_instrs[next_addr]
            if nm.startswith('b.') or nm == 'b':
                if 0x10000D000 <= addr <= 0x10000F000:
                    func = find_function_start(addr)
                    # Only if also checking other values
                    for delta in range(-0x30, 0x30, 4):
                        check = addr + delta
                        if check in all_instrs:
                            cm, co = all_instrs[check]
                            if cm == 'cmp' and any(f'#{v}' in co for v in ['3', '4', '5', '6']):
                                print(f"  Multi-case switch at 0x{func:011X} (cmp#1 at 0x{addr:011X})")
                                break

# ============================================================
# SECTION D: DFU STATE MACHINE — ALL STATES
# ============================================================
print("\n" + "="*80)
print("SECTION D: DFU STATE MACHINE")
print("="*80)
print("""
DFU states per USB DFU spec:
  0 = appIDLE
  1 = appDETACH
  2 = dfuIDLE
  3 = dfuDNLOAD-SYNC
  4 = dfuDNBUSY
  5 = dfuDNLOAD-IDLE
  6 = dfuMANIFEST-SYNC
  7 = dfuMANIFEST
  8 = dfuMANIFEST-WAIT-RESET
  9 = dfuUPLOAD-IDLE
 10 = dfuERROR

Looking for DFU state reads/writes in ROM code.
gDFU_interface+0x14 = DFU state byte
""")

# Find all code that accesses gDFU_interface+0x14 (state byte)
# This is at 0x19C010BE0+0x14 = 0x19C010BF4
# But it's accessed via register + offset, e.g., LDRB w0, [x8, #0x14] or STRB w0, [x8, #0x14]
print("--- Code accessing offset #0x14 (DFU state) ---")
for addr, (m, o) in sorted(all_instrs.items()):
    if (m == 'ldrb' or m == 'strb') and '#0x14' in o:
        if 0x10000E000 <= addr <= 0x10000F000:
            print(f"  0x{addr:011X}: {m} {o}")

# State transition constants — look for stores of specific DFU state values
print("\n--- DFU state transitions (STRB with state constants) ---")
for addr, (m, o) in sorted(all_instrs.items()):
    if m == 'strb' and '#0x14' in o:
        # Look backwards for the MOV that sets the value
        for delta in range(-16, 0, 4):
            prev = addr + delta
            if prev in all_instrs:
                pm, po = all_instrs[prev]
                if pm in ('mov', 'movz', 'orr') and 'w' in po:
                    print(f"  0x{prev:011X}: {pm} {po}  → 0x{addr:011X}: {m} {o}")

# ============================================================
# SECTION E: HEAP ALLOCATOR — UNDERSTAND METADATA
# ============================================================
print("\n" + "="*80)
print("SECTION E: HEAP ALLOCATOR INTERNALS")
print("="*80)

# malloc at 0x10000F1EC
print_func_full(0x10000F1EC, 0x200, "malloc (0x10000F1EC)")

# free at 0x10000F468
print_func_full(0x10000F468, 0x200, "free (0x10000F468)")

# memalign at 0x10000F680
print_func_full(0x10000F680, 0x200, "memalign (0x10000F680)")

# calloc at 0x10000F3B0
print_func_full(0x10000F3B0, 0x100, "calloc (0x10000F3B0)")

# ============================================================
# SECTION F: USB QUIESCE/ABORT PATH
# ============================================================
print("\n" + "="*80)
print("SECTION F: USB ABORT / QUIESCE PATH")
print("="*80)
print("""
The usb_quiesce function is called on DFU_ABORT.
It frees the io_buffer and other USB state.
Understanding this path reveals what's dangling after abort.
""")

# DFU ABORT resets state — trace what happens
# In the DFU request handler, bRequest=6 (ABORT) branches to...
# Let's look at known references to free()
print("--- All calls to free (0x10000F468) ---")
for caller in sorted(bl_callers.get(0x10000F468, set())):
    func = find_function_start(caller)
    print(f"  0x{caller:011X} in func 0x{func:011X}")

# USB reset handler
print("\n--- USB bus reset handler ---")
# Look for the function that's called on USB bus reset
# It would reference the USB controller registers and reset state
# Typically writes 0 to various USB state fields

# Find DFU state setup (0x10000D0A8) — installs/clears DFU vtable  
print_func_full(0x10000D0A8, 0x100, "dfu_state_setup (0x10000D0A8) — install/clear DFU vtable")

# DFU callback dispatch trampoline
print_func_full(0x10000D100, 0x100, "dfu_callback_dispatch (0x10000D100)")

# ============================================================
# SECTION G: ALTERNATIVE BUFFERS — VENDOR REQUESTS
# ============================================================
print("\n" + "="*80)
print("SECTION G: VENDOR/CLASS REQUEST HANDLERS")  
print("="*80)
print("""
Questions:
- Do vendor requests (type=0x40/0xC0) use the same io_buffer as DFU?
- Or do they allocate separate buffers?
- Can we use vendor requests to write to heap WITHOUT touching freed io_buffer?
""")

# Look for bmRequestType=0x40 (vendor OUT) dispatch
print("--- Searching for vendor request dispatch (CMP #0x40, #0xC0) ---")
for addr, (m, o) in sorted(all_instrs.items()):
    if m == 'cmp' and ('#0x40' in o or '#0xc0' in o.lower()):
        if 0x10000C000 <= addr <= 0x10000F000:
            func = find_function_start(addr)
            print(f"  0x{addr:011X}: {m} {o} (func 0x{func:011X})")

# Look for the USB register_interface function that sets up handler vtables
print_func_full(0x10000D924, 0x200, "usb_register_interface (0x10000D924)")

# Get descriptor handler (GET_DESCRIPTOR dispatch)
print_func_full(0x10000DCC8, 0x400, "get_descriptor_handler (0x10000DCC8)")

# ============================================================
# SECTION H: EP0 BUFFER ALLOCATION TRACE
# ============================================================
print("\n" + "="*80)
print("SECTION H: EP0 BUFFER ALLOCATION")
print("="*80)
print("""
Trace: Where is io_buffer (0x800 bytes) allocated?
- dfu_init allocates it
- It's stored at gDFU_interface+0x28 (0x19C010BE0+0x28 = 0x19C010C08)
- It's also set as the EP0 data phase buffer
""")

# Find where 0x800 is used as malloc size
print("--- All MOV w?,#0x800 instructions ---")
for addr, (m, o) in sorted(all_instrs.items()):
    if m in ('mov', 'movz') and '#0x800' in o:
        print(f"  0x{addr:011X}: {m} {o}")
        # Check if followed by BL to malloc
        for delta in range(4, 24, 4):
            next_a = addr + delta
            if next_a in all_instrs:
                nm, no = all_instrs[next_a]
                if nm == 'bl':
                    try:
                        target = int(no.replace('#', ''), 16)
                        if target in (0x10000F1EC, 0x10000F3B0, 0x10000F680):
                            print(f"    → BL to alloc at 0x{next_a:011X} (target 0x{target:011X})")
                    except: pass

# Also check for mov w?,#0x801 (the boundary check)
print("\n--- All CMP/MOV with #0x801 ---")
for addr, (m, o) in sorted(all_instrs.items()):
    if ('#0x801' in o or '#2049' in o):
        print(f"  0x{addr:011X}: {m} {o}")

# ============================================================
# SECTION I: USB TRANSFER SUBMIT — HOW DATA GETS TO BUFFER
# ============================================================
print("\n" + "="*80)
print("SECTION I: USB TRANSFER SUBMIT (0x10000E92C)")
print("="*80)

print_func_full(0x10000E92C, 0x300, "usb_transfer_submit (0x10000E92C)")

# ============================================================  
# SECTION J: USB CONTROLLER INIT — DWC3 SPECIFICS
# ============================================================
print("\n" + "="*80)
print("SECTION J: USB CONTROLLER INIT (0x10000C3A4)")
print("="*80)

print_func_full(0x10000C3A4, 0x400, "usb_controller_init (0x10000C3A4)")

# ============================================================
# SECTION K: WHAT'S AFTER io_buffer IN HEAP?
# ============================================================
print("\n" + "="*80)
print("SECTION K: ALLOCATION ORDER ANALYSIS")
print("="*80)
print("""
To understand what's adjacent to io_buffer on the heap,
we need to know the ORDER of allocations during USB/DFU init.

The init sequence is roughly:
1. usb_init() → allocates USB state structures
2. dfu_init() → allocates io_buffer (0x800 bytes)
3. Various descriptor allocations

Tracking every malloc call in the init path:
""")

# Find all BL to malloc in the USB/DFU init range
print("--- Malloc calls in USB/DFU code (0x10000B000-0x10000F000) ---")
for caller in sorted(bl_callers.get(0x10000F1EC, set())):
    if 0x10000B000 <= caller <= 0x10000F000:
        func = find_function_start(caller)
        # Find what size is being passed
        for delta in range(-20, 0, 4):
            prev = caller + delta
            if prev in all_instrs:
                pm, po = all_instrs[prev]
                if pm in ('mov', 'movz') and ('w0' in po or 'x0' in po):
                    print(f"  0x{caller:011X}: malloc() [func 0x{func:011X}] size hint: {pm} {po}")
                    break
        else:
            print(f"  0x{caller:011X}: malloc() [func 0x{func:011X}]")

# Also memalign calls
print("\n--- Memalign calls in USB/DFU code ---")
for caller in sorted(bl_callers.get(0x10000F680, set())):
    if 0x10000B000 <= caller <= 0x10000F000:
        func = find_function_start(caller)
        print(f"  0x{caller:011X}: memalign() [func 0x{func:011X}]")

# Calloc calls
print("\n--- Calloc calls in USB/DFU code ---")
for caller in sorted(bl_callers.get(0x10000F3B0, set())):
    if 0x10000B000 <= caller <= 0x10000F000:
        func = find_function_start(caller)
        print(f"  0x{caller:011X}: calloc() [func 0x{func:011X}]")

print("\n\n" + "="*80)
print("ANALYSIS COMPLETE")
print("="*80)
