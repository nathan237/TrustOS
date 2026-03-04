#!/usr/bin/env python3
"""
Second-pass deep analysis: find callers and trace value sources for all 3 targets.
"""

import struct
import re
from capstone import *

ROM_PATH = r"C:\Users\nathan\Documents\Scripts\OSrust\tools\checkm8-dualboot\securerom\t8020_B1_securerom.bin"
ROM_BASE = 0x100000000
ROM_SIZE = 524288

with open(ROM_PATH, "rb") as f:
    rom_data = f.read()

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = False

def va_to_offset(va):
    off = va - ROM_BASE
    if 0 <= off < len(rom_data):
        return off
    return None

def read_u32(va):
    off = va_to_offset(va)
    if off is not None and off + 4 <= len(rom_data):
        return struct.unpack('<I', rom_data[off:off+4])[0]
    return None

def read_u64(va):
    off = va_to_offset(va)
    if off is not None and off + 8 <= len(rom_data):
        return struct.unpack('<Q', rom_data[off:off+8])[0]
    return None

def disasm_range(start_va, end_va, label=None):
    off_s = va_to_offset(start_va)
    off_e = va_to_offset(end_va)
    if off_s is None or off_e is None:
        return []
    insns = list(md.disasm(rom_data[off_s:off_e], start_va))
    if label:
        print(f"\n{'='*80}")
        print(f"  {label}")
        print(f"{'='*80}")
    for i in insns:
        print(f"  {i.address:#012x}:  {i.mnemonic:8s} {i.op_str}")
    return insns

# ============================================================================
# 1. Find ALL xrefs (BL and B) to the 3 writer functions
# ============================================================================
print("=" * 80)
print("PHASE 1: Finding all xrefs (BL and B) to target functions")
print("=" * 80)

targets = {
    0x100008AF0: "Target1 writer (SRAM 0x19C008448)",
    0x100002844: "Target2 writer (SRAM 0x19C008B48)",
    0x10000ADDC: "Target3 writer (SRAM 0x19C010620)",
    0x1000028C0: "Target2 BLR consumer",
    0x10000AD80: "Target3 BLR consumer",
}

print("\n[*] Scanning entire ROM for BL/B to target functions...")
all_insns = list(md.disasm(rom_data[:ROM_SIZE], ROM_BASE))
print(f"  Total instructions decoded: {len(all_insns)}")

for target_va, desc in targets.items():
    refs = []
    for ins in all_insns:
        if ins.mnemonic in ('bl', 'b'):
            m = re.match(r'#(0x[0-9a-fA-F]+)', ins.op_str)
            if m and int(m.group(1), 16) == target_va:
                refs.append((ins.address, ins.mnemonic))
    print(f"\n  {target_va:#x} ({desc}):")
    if refs:
        for addr, mnem in refs:
            print(f"    {mnem.upper()} at {addr:#x}")
    else:
        print(f"    No direct BL/B xrefs found — must be called indirectly")

# ============================================================================
# 2. Search ROM data section for function pointer references
# ============================================================================
print("\n\n" + "=" * 80)
print("PHASE 2: Searching ROM for function pointers in data")
print("=" * 80)

for target_va, desc in targets.items():
    print(f"\n  Searching for {target_va:#x} ({desc}) in ROM data...")
    # Search for 8-byte little-endian encoding
    target_bytes = struct.pack('<Q', target_va)
    offset = 0
    found = False
    while True:
        idx = rom_data.find(target_bytes, offset)
        if idx == -1:
            break
        found = True
        print(f"    Found at ROM offset {idx:#x} (VA {ROM_BASE + idx:#x})")
        offset = idx + 1
    if not found:
        print(f"    Not found as literal pointer in ROM data")

# ============================================================================
# 3. Search for ADRP sequences that reference writer function addresses
# ============================================================================
print("\n\n" + "=" * 80)
print("PHASE 3: ADRP sequences that reference target addresses")
print("=" * 80)

for target_va, desc in targets.items():
    target_page = target_va & ~0xFFF
    target_off = target_va & 0xFFF
    print(f"\n  Looking for ADRP #{target_page:#x} (page for {target_va:#x}, {desc})...")
    for i, ins in enumerate(all_insns):
        if ins.mnemonic == 'adrp':
            m = re.match(r'x(\d+),\s*#(0x[0-9a-fA-F]+)', ins.op_str)
            if m and int(m.group(2), 16) == target_page:
                # Check following instructions for ADD with matching offset
                for j in range(i+1, min(i+5, len(all_insns))):
                    nxt = all_insns[j]
                    if nxt.mnemonic == 'add':
                        ma = re.match(r'x(\d+),\s*x(\d+),\s*#(0x[0-9a-fA-F]+)', nxt.op_str)
                        if ma and int(ma.group(3), 16) == target_off:
                            print(f"    ADRP+ADD at {ins.address:#x}..{nxt.address:#x} => x{ma.group(1)} = {target_va:#x}")

# ============================================================================
# 4. Deep analysis of Target 2 (USB code)
# ============================================================================
print("\n\n" + "=" * 80)
print("PHASE 4: DEEP ANALYSIS — Target 2 (SRAM 0x19C008B48)")
print("=" * 80)

print("""
[*] func_0x100002844 analysis:
    
    Arguments: x0, x1, x2
    Saves: x19=x0 (index), x21=x1 (callback ptr), x20=x2 (callback arg)
    
    Computes: x8 = 0x19C008B48 + w19 * 0x18
    Stores: STP x21, x20, [x8]  →  stores (x1, x2) at array[x0]
    
    This is an INTERRUPT/EVENT HANDLER REGISTRATION function.
    Array base = 0x19C008B48, entry size = 0x18 (24 bytes)
    Each entry: [callback_ptr (8 bytes), callback_arg (8 bytes), ???]
    
    The value stored at SRAM 0x19C008B48 comes directly from the
    function's x1 argument (the callback pointer).

[*] func_0x1000028C0 (BLR consumer) analysis:
    
    Reads a hardware register at 0x23B102004
    Loops while events pending:
      - Extracts index = w8 & 0x1FF
      - Loads callback = array[index].ptr from [0x19C008B48 + index*24]
      - Loads arg = array[index].arg from [0x19C008B48 + index*24 + 8]
      - If callback != NULL: BLR callback(arg)
    
    This is an INTERRUPT DISPATCH LOOP.
""")

# Now the critical question: who calls 0x100002844 with what arguments?
# Since no direct BL was found, check for it being loaded as a function pointer
# or being in a vtable

# Let's also check if it might be called from setup/init code
# Check the broader USB init flow
print("[*] Checking USB initialization flow...")
print("    Looking for functions that reference 0x100002844...")

# Search for the function address in ADRP+ADD patterns
page_2844 = 0x100002844 & ~0xFFF  # = 0x100002000
off_2844 = 0x100002844 & 0xFFF    # = 0x844

print(f"    Page: {page_2844:#x}, Offset: {off_2844:#x}")
for i, ins in enumerate(all_insns):
    if ins.mnemonic == 'adrp':
        m = re.match(r'x(\d+),\s*#(0x[0-9a-fA-F]+)', ins.op_str)
        if m and int(m.group(2), 16) == page_2844:
            reg = int(m.group(1))
            # Check for ADD to construct full address
            for j in range(i+1, min(i+5, len(all_insns))):
                nxt = all_insns[j]
                if nxt.mnemonic == 'add':
                    ma = re.match(r'x(\d+),\s*x(\d+),\s*#(0x[0-9a-fA-F]+)', nxt.op_str)
                    if ma and int(ma.group(2)) == reg and int(ma.group(3), 16) == off_2844:
                        print(f"    ADRP+ADD constructs 0x100002844 at {ins.address:#x}..{nxt.address:#x}")
                        # Show context
                        disasm_range(ins.address - 16, nxt.address + 32, 
                                    f"Context where 0x100002844 is referenced")

# Also check if function 0x100002844 is the same as some func in a table
# Let's check broader USB init
print("\n[*] Disassembling USB init functions that might register handlers:")
# 0x1000020C8, 0x1000020F4, 0x100002120 are USB funcs called from elsewhere
for func_va in [0x1000020C8, 0x1000020F4, 0x100002120, 0x1000021E4, 0x10000226C]:
    print(f"\n[*] func_{func_va:#x}:")
    disasm_range(func_va, func_va + 0x80, f"func_{func_va:#x}")


# ============================================================================
# 5. Check if 0x19C010620 ever gets written with a function pointer
# ============================================================================
print("\n\n" + "=" * 80)
print("PHASE 5: Who writes to SRAM 0x19C010620?")
print("=" * 80)

# Search for STR to [xN, #0x620] with ADRP 0x19C010000
page_10620 = 0x19C010000
off_10620 = 0x620

print(f"[*] Looking for ADRP #{page_10620:#x} followed by STR [reg, #{off_10620:#x}]...")
for i, ins in enumerate(all_insns):
    if ins.mnemonic == 'adrp':
        m = re.match(r'x(\d+),\s*#(0x[0-9a-fA-F]+)', ins.op_str)
        if m and int(m.group(2), 16) == page_10620:
            reg = int(m.group(1))
            # Look for STR within next 10 instructions
            for j in range(i+1, min(i+10, len(all_insns))):
                nxt = all_insns[j]
                if nxt.mnemonic.startswith('str'):
                    mm = re.search(r'\[x(\d+)(?:,\s*#(0x[0-9a-fA-F]+))?\]', nxt.op_str)
                    if mm and int(mm.group(1)) == reg:
                        disp = int(mm.group(2), 16) if mm.group(2) else 0
                        ea = page_10620 + disp
                        if ea == 0x19C010620:
                            print(f"  STR to 0x19C010620 at {nxt.address:#x} (ADRP at {ins.address:#x})")
                            disasm_range(ins.address - 16, nxt.address + 16,
                                        f"Context: STR to SRAM 0x19C010620")

# ============================================================================
# 6. Check who writes to 0x19C008448 (Target 1 callers)
# ============================================================================
print("\n\n" + "=" * 80)
print("PHASE 6: Who calls the Target 1 writer (0x100008AF0)?")
print("=" * 80)

# We know it has no direct BL. Check for ADRP+ADD constructing the address
page_8af0 = 0x100008AF0 & ~0xFFF  # = 0x100008000
off_8af0 = 0x100008AF0 & 0xFFF    # = 0xAF0

print(f"[*] Looking for ADRP #{page_8af0:#x} + ADD #{off_8af0:#x}...")
for i, ins in enumerate(all_insns):
    if ins.mnemonic == 'adrp':
        m = re.match(r'x(\d+),\s*#(0x[0-9a-fA-F]+)', ins.op_str)
        if m and int(m.group(2), 16) == page_8af0:
            reg = int(m.group(1))
            for j in range(i+1, min(i+5, len(all_insns))):
                nxt = all_insns[j]
                if nxt.mnemonic == 'add':
                    ma = re.match(r'x(\d+),\s*x(\d+),\s*#(0x[0-9a-fA-F]+)', nxt.op_str)
                    if ma and int(ma.group(2)) == reg and int(ma.group(3), 16) == off_8af0:
                        print(f"  ADRP+ADD => 0x100008AF0 at {ins.address:#x}..{nxt.address:#x}")
                        disasm_range(ins.address - 8, nxt.address + 24, "Context")

# Also check nearby — the function at 0x100008B58 is called from USB
print(f"\n[*] What's at 0x100008B58 (called from USB):")
disasm_range(0x100008B58, 0x100008BC0, "func_0x100008B58")


# ============================================================================
# 7. Check the interrupt setup flow for Target 2
# ============================================================================
print("\n\n" + "=" * 80)
print("PHASE 7: Interrupt registration flow for Target 2")
print("=" * 80)

# The writer function 0x100002844 registers {callback, arg} at index x0
# Let's look for who passes what to this function
# Check if there's a function table or hardcoded calls

# Look for all BLR in USB range where target might be loaded from a table
print("[*] Checking for BLR calls in range where 0x100002844 might be the target...")

# The key insight: 0x100002844 itself has no BL callers.
# It's likely stored in a vtable/struct and called via BLR.
# Let's search for the address in ROM data
addr_bytes = struct.pack('<Q', 0x100002844)
idx = rom_data.find(addr_bytes)
if idx != -1:
    print(f"  Found 0x100002844 as pointer in ROM data at offset {idx:#x} (VA {ROM_BASE + idx:#x})")
    # Show surrounding data
    ctx_start = max(0, idx - 32)
    ctx_end = min(len(rom_data), idx + 40)
    print(f"  Surrounding data:")
    for off in range(ctx_start, ctx_end, 8):
        val = struct.unpack('<Q', rom_data[off:off+8])[0]
        marker = " <-- 0x100002844" if off == idx else ""
        print(f"    [{ROM_BASE + off:#x}] = {val:#x}{marker}")
else:
    print("  0x100002844 not found as literal in ROM data - called via computed address")

# Check 0x1000028C0 too
addr_bytes2 = struct.pack('<Q', 0x1000028C0)
idx2 = rom_data.find(addr_bytes2)
if idx2 != -1:
    print(f"\n  Found 0x1000028C0 as pointer in ROM data at offset {idx2:#x} (VA {ROM_BASE + idx2:#x})")
    ctx_start = max(0, idx2 - 32)
    ctx_end = min(len(rom_data), idx2 + 40)
    for off in range(ctx_start, ctx_end, 8):
        val = struct.unpack('<Q', rom_data[off:off+8])[0]
        marker = " <-- 0x1000028C0" if off == idx2 else ""
        print(f"    [{ROM_BASE + off:#x}] = {val:#x}{marker}")

# ============================================================================
# 8. Check 0x10000AD80 pointer references
# ============================================================================
print("\n\n" + "=" * 80)
print("PHASE 8: Pointer references for Target 3 (0x10000AD80)")
print("=" * 80)

addr_bytes3 = struct.pack('<Q', 0x10000AD80)
idx3 = rom_data.find(addr_bytes3)
if idx3 != -1:
    print(f"  Found 0x10000AD80 in ROM data at offset {idx3:#x} (VA {ROM_BASE + idx3:#x})")
else:
    print("  0x10000AD80 not found as literal in ROM data")

# Check 0x10000ADDC
addr_bytes4 = struct.pack('<Q', 0x10000ADDC)
idx4 = rom_data.find(addr_bytes4)
if idx4 != -1:
    print(f"  Found 0x10000ADDC in ROM data at offset {idx4:#x} (VA {ROM_BASE + idx4:#x})")
else:
    print("  0x10000ADDC not found as literal in ROM data")

# Let's check what function registers a callback into 0x19C010620
# Search for STR to 0x19C010620
print(f"\n[*] All stores to SRAM 0x19C010620:")
for i, ins in enumerate(all_insns):
    if ins.mnemonic.startswith('str'):
        mm = re.search(r'\[x(\d+)(?:,\s*#(0x[0-9a-fA-F]+))?\]', ins.op_str)
        if mm:
            disp_str = mm.group(2)
            disp = int(disp_str, 16) if disp_str else 0
            base_reg = int(mm.group(1))
            # To know the full EA we need the ADRP, so look backward
            if disp == 0x620:
                # Could be 0x19C010620 if base is 0x19C010000
                print(f"  Potential: {ins.address:#x}: {ins.mnemonic} {ins.op_str} (offset #0x620)")
                # Show context
                disasm_range(ins.address - 32, ins.address + 8, f"Context for STR at {ins.address:#x}")

# ============================================================================
# 9. Final summary with value controllability assessment
# ============================================================================
print("\n\n" + "=" * 80)
print("FINAL DETAILED ASSESSMENT")
print("=" * 80)

print("""
============================================================
TARGET 1: SRAM 0x19C008448
============================================================
WRITER: func_0x100008AF0
  This is NOT the panic function (panic is 0x100008978).
  The writer is a separate small function at 0x100008AF0.

WHAT IT WRITES:
  Path A (lock acquired, cbz taken → w0==0):
    - Reads current [0x19C008448] into x8
    - ANDs with 7 → gets 3-bit index
    - Clears a byte in SRAM buffer at [0x19C00BD90 + index]
    - Reloads x8 from [0x19C00BD90] (first 8 bytes of buffer)
    - Stores x8 → [0x19C008448]
  
  Path B (lock not acquired):
    - Constructs constant x8 = 0x4752_4004_4430_3631
    - Stores to [0x19C00BD90]
    - Then stores same x8 → [0x19C008448]

VALUE SOURCE: Entirely ROM-derived constant or self-referential SRAM manipulation.
  The constant 0x47524004_44303631 = ASCII "16D@RG" (part of a debug/tracking string).
  This is a cyclic buffer pointer/counter — NOT a function pointer.

IS THIS A FUNCTION POINTER? UNLIKELY.
  The value is manipulated with AND #7 and byte operations.
  At the BLR site (0x100002458), the code does:
    ldr x8, [x27, #8] → loads from a struct, NOT directly from 0x19C008448
  The 0x19C008448 is used as a CANARY/COOKIE, not a code pointer.

VERDICT: NOT EXPLOITABLE
  The value is ROM-constant or self-derived. Not attacker-controllable.
  The BLR at 0x100002458 uses a different value (struct field).

============================================================
TARGET 2: SRAM 0x19C008B48  *** USB CODE ***
============================================================
WRITER: func_0x100002844 — Interrupt handler registration
  
  void register_handler(uint32_t index, void* callback, void* arg) {
    lock();
    uint64_t* entry = (uint64_t*)(0x19C008B48 + index * 24);
    entry[0] = callback;   // x1
    entry[1] = arg;        // x2
    if (index <= 0x1FF) {
      // Configure hardware interrupt controller at 0x23B102000
      uint32_t* hw = (uint32_t*)0x23B102000;
      uint32_t bit = 1 << hw[0];
      hw[index + 0x400] = bit;
    }
    unlock();
    return 0;
  }

WHAT IT WRITES:
  The callback pointer (x1) and argument (x2) from the caller.
  These are stored at SRAM 0x19C008B48 + index*24.

CONSUMER (BLR at 0x100002914):
  func_0x1000028C0 — Interrupt dispatch loop
  Reads from hardware register 0x23B102004
  Extracts interrupt index, looks up callback from the array,
  and BLR's to it with the stored argument.

VALUE SOURCE:
  The stored values come from whoever calls register_handler().
  No direct BL callers found — meaning it's either:
  a) Called via a function pointer table (vtable), or
  b) Called via a trampoline/wrapper

ATTACKER CONTROLLABILITY:
  The callback registration happens during USB/interrupt initialization.
  The caller provides ROM function addresses as callbacks.
  The interrupt controller hardware register 0x23B10xxxx is not
  USB-writable — an attacker cannot trigger arbitrary interrupt indices.
  
  HOWEVER: If the SRAM at 0x19C008B48 can be overflowed/corrupted
  via a DFU heap overflow (like checkm8), an attacker could overwrite
  the callback pointer stored there, and the next interrupt dispatch
  would BLR to the attacker's value.

VERDICT: CONDITIONALLY EXPLOITABLE
  NOT directly controllable via USB protocol data.
  BUT exploitable via SRAM corruption (heap overflow) since:
  - The callback table lives in SRAM
  - The dispatch loop blindly trusts the loaded value
  - No pointer authentication on the BLR

============================================================
TARGET 3: SRAM 0x19C010620
============================================================
WRITER: func_0x10000ADDC — Block device frequency detection
  
  uint32_t get_block_freq() {
    uint64_t cookie = *(uint64_t*)0x19C008448;  // canary
    uint32_t cached = *(uint32_t*)0x19C010628;
    if (cached != 0) return cached;
    
    // Query hardware for clock frequency
    uint32_t raw;
    hw_query(0x700, 0x10000, &raw);
    if (raw == 0) goto check_canary;
    
    uint32_t freq = raw / 1000000;  // Divide by 1M
    if (raw % 1000000 != 0) goto check_canary;
    
    *(uint32_t*)0x19C010628 = freq;  // Cache it
    // ...check canary and return
  }

WHAT IT WRITES:
  A hardware-derived clock frequency value to 0x19C010628 (NOT 0x19C010620).
  The value at 0x19C010620 is not written by this function!

CONSUMER (BLR at 0x10000ADA8):
  func_0x10000AD80 loads from [0x19C010620] (not [0x19C010628]).
  These are DIFFERENT offsets — the BLR reads from +0x620, the writer
  writes to +0x628.

  The value at 0x19C010620 must be written by a DIFFERENT function.

ATTACKER CONTROLLABILITY:
  The writer at 0x10000ADDC writes a hardware clock constant, not user data.
  But the BLR source (0x19C010620) is written elsewhere — need to find that writer.

VERDICT: NEEDS FURTHER ANALYSIS
  The 0x10000ADDC function only writes to 0x19C010628 (cached frequency).
  The BLR at 0x10000ADA8 reads from 0x19C010620 — a different cell.
  The actual writer of 0x19C010620 needs to be identified.
""")

print("[*] Second-pass analysis complete.")
