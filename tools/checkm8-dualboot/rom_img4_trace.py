#!/usr/bin/env python3
"""
ROM img4_verify RETURN VALUE TRACE
===================================
THE LEAD: cbz w8 only checks lower 32 bits of img4_verify return.
Sub-function 0x10000F1EC returns 64-bit values.
If lower 32 bits = 0 but upper bits != 0 -> SIGNATURE BYPASS

This script:
1. Fully disassembles img4_verify (0x10000A704)
2. Traces ALL return paths to find which returns reach cbz w8
3. Fully RE's 0x10000F1EC (the sub-function with 64-bit returns)
4. Traces what x19/x20/x22 are in 0x10000F1EC
5. Checks if ANY path can produce 0xXXXXXXXX00000000
6. Examines the signature gate code in detail
"""

import struct
from capstone import *
from capstone.arm64 import *

ROM_PATH = "securerom/t8020_B1_securerom.bin"
ROM_BASE = 0x100000000

with open(ROM_PATH, "rb") as f:
    rom = f.read()

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

def disasm_at(addr, count=200):
    off = addr - ROM_BASE
    code = rom[off:off + count * 4]
    return list(md.disasm(code, addr, count))

def disasm_func(addr, max_instrs=500):
    """Disassemble a function until RET"""
    instrs = []
    for i in disasm_at(addr, max_instrs):
        instrs.append(i)
        if i.mnemonic == 'ret':
            break
    return instrs

def read32(addr):
    off = addr - ROM_BASE
    return struct.unpack('<I', rom[off:off+4])[0]

def fmt(i):
    return f"  0x{i.address:X}: {i.mnemonic:16s} {i.op_str}"

WIDTH = 120
SEP = "=" * WIDTH

print(SEP)
print("  IMG4_VERIFY RETURN VALUE TRACE - THE CBZ W8 WIDTH MISMATCH")
print(SEP)

# ============================================================
# PART 1: The Signature Gate - exact code around cbz w8
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 1: THE SIGNATURE GATE (boot flow around cbz w8)")
print(f"# Location: 0x100001BC8")
print(f"{'#' * WIDTH}\n")

gate_instrs = disasm_at(0x100001B90, 40)
for i in gate_instrs:
    marker = ""
    if i.address == 0x100001BC4:
        marker = "  <<< mov x8, x0 (COPIES FULL 64-BIT x0 to x8)"
    elif i.address == 0x100001BC8:
        marker = "  <<< CBZ W8 !!! ONLY CHECKS LOWER 32 BITS !!!"
    elif "10000a704" in i.op_str.lower() or "a704" in i.op_str.lower():
        marker = "  <<< bl img4_verify"
    print(f"{fmt(i)}{marker}")

# ============================================================
# PART 2: Full img4_verify disassembly with return path analysis
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 2: img4_verify @ 0x10000A704 - FULL DISASSEMBLY")
print(f"# Focus: every return path and what x0 contains")
print(f"{'#' * WIDTH}\n")

img4_instrs = disasm_func(0x10000A704, 300)
# Find all returns and what sets x0 before them
return_paths = []
call_targets = set()

for idx, i in enumerate(img4_instrs):
    marker = ""
    # Track mov to x0/w0
    if i.mnemonic in ('mov', 'movz', 'movk', 'movn') and i.op_str.startswith(('x0,', 'w0,')):
        width = "X64" if i.op_str.startswith('x0') else "W32"
        marker = f"  <<< SETS RETURN VALUE ({width})"
    # Track BL calls
    if i.mnemonic == 'bl':
        target = int(i.op_str.lstrip('#'), 16)
        call_targets.add(target)
        marker = f"  <<< CALL to 0x{target:X}"
    # Track RET
    if i.mnemonic == 'ret':
        marker = "  <<< RETURN"
    # Track cbz/cbnz on x0/w0
    if i.mnemonic in ('cbz', 'cbnz') and ('x0' in i.op_str or 'w0' in i.op_str):
        marker = "  <<< BRANCH on return value"
        
    print(f"{fmt(i)}{marker}")

print(f"\n  Total instructions: {len(img4_instrs)}")
print(f"  Call targets from img4_verify:")
for t in sorted(call_targets):
    print(f"    0x{t:X}")

# ============================================================
# PART 3: Deep dive into 0x10000F1EC (the sub-function with x64 returns)
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 3: SUB-FUNCTION 0x10000F1EC - THE CRITICAL LEAD")
print(f"# This function returns mov x0, x19/x20/x22 (64-bit)")
print(f"# If these values have lower 32 bits = 0 -> BYPASS")
print(f"{'#' * WIDTH}\n")

# Disassemble the function - it might have multiple RET paths
sub_instrs = disasm_at(0x10000F1EC, 200)
ret_count = 0
for i in sub_instrs:
    marker = ""
    if i.mnemonic in ('mov', 'movz', 'movk') and i.op_str.startswith(('x0,', 'w0,')):
        width = "X64" if i.op_str.startswith('x0') else "W32"
        marker = f"  <<< SETS x0 ({width})"
    if i.mnemonic == 'ret':
        marker = "  <<< RETURN PATH"
        ret_count += 1
    if i.mnemonic == 'bl':
        target = int(i.op_str.lstrip('#'), 16)
        marker = f"  <<< CALLS 0x{target:X}"
    if i.mnemonic in ('cbz', 'cbnz', 'tbz', 'tbnz'):
        marker = f"  <<< CONDITIONAL BRANCH"
    if i.mnemonic.startswith('b.'):
        marker = f"  <<< COND BRANCH ({i.mnemonic})"
    if i.mnemonic == 'stp' and 'x29' in i.op_str and ret_count > 0:
        print(f"  --- (next function prologue reached after {ret_count} RETs) ---")
        break
    print(f"{fmt(i)}{marker}")

# ============================================================
# PART 4: What calls 0x10000F1EC? Trace every caller
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 4: ALL CALLERS of 0x10000F1EC")
print(f"# How is its return value used? Does it propagate to img4_verify?")
print(f"{'#' * WIDTH}\n")

# Search entire ROM for bl #0x10000F1EC
target_bytes_patterns = []
# BL encoding: imm26 * 4 = target - PC
callers = []
for off in range(0, len(rom) - 4, 4):
    insn_bytes = struct.unpack('<I', rom[off:off+4])[0]
    # BL opcode: 0x94000000 | imm26
    if (insn_bytes & 0xFC000000) == 0x94000000:
        imm26 = insn_bytes & 0x03FFFFFF
        # Sign extend
        if imm26 & 0x02000000:
            imm26 |= ~0x03FFFFFF
            imm26 = imm26 & 0xFFFFFFFF
            imm26 = struct.unpack('<i', struct.pack('<I', imm26))[0]
        target = (ROM_BASE + off) + (imm26 * 4)
        if target == 0x10000F1EC:
            callers.append(ROM_BASE + off)

print(f"  Total callers of 0x10000F1EC: {len(callers)}")
for caller in callers:
    print(f"\n  --- Caller at 0x{caller:X} ---")
    # Show context: 5 before, the call, 10 after
    ctx = disasm_at(caller - 20, 20)
    for i in ctx:
        prefix = "   >>> " if i.address == caller else "       "
        use_marker = ""
        # Check if x0/w0 is used after the call
        if i.address > caller:
            if 'x0' in i.op_str or 'w0' in i.op_str:
                if i.mnemonic in ('cbz', 'cbnz'):
                    width = "64-bit" if 'x0' in i.op_str else "32-bit"
                    use_marker = f"  <<< CHECKS RETURN ({width})"
                elif i.mnemonic == 'mov' and not i.op_str.startswith(('x0', 'w0')):
                    use_marker = f"  <<< SAVES RETURN"
                elif i.mnemonic in ('str', 'stp'):
                    use_marker = f"  <<< STORES RETURN"
        print(f"{prefix}0x{i.address:X}: {i.mnemonic:12s} {i.op_str}{use_marker}")

# ============================================================
# PART 5: Trace the EXACT return path from img4_verify back to cbz w8
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 5: img4_verify INTERNAL FLOW - does 0x10000F1EC return propagate?")
print(f"# We need to trace: bl 0x10000F1EC -> ... -> ret (img4_verify) -> cbz w8")
print(f"{'#' * WIDTH}\n")

# Find the BL to 0x10000F1EC within img4_verify
for i in img4_instrs:
    if i.mnemonic == 'bl' and '0x10000f1ec' in i.op_str.lower():
        bl_addr = i.address
        print(f"  img4_verify calls 0x10000F1EC at: 0x{bl_addr:X}")
        # Show what happens after this call
        print(f"  Code AFTER bl 0x10000F1EC:")
        after = disasm_at(bl_addr, 30)
        past_bl = False
        for ai in after:
            if ai.address == bl_addr:
                past_bl = True
                print(f"  >>> {fmt(ai)}")
                continue
            if past_bl:
                marker = ""
                if ai.mnemonic == 'ret':
                    marker = "  <<< img4_verify RETURNS HERE"
                if 'x0' in ai.op_str or 'w0' in ai.op_str:
                    if ai.mnemonic == 'mov' and ai.op_str.startswith(('x0', 'w0')):
                        marker = "  <<< OVERWRITES x0"
                    elif ai.mnemonic == 'mov' and not ai.op_str.startswith(('x0', 'w0')):
                        marker = "  <<< COPIES x0"
                    elif ai.mnemonic in ('cbz', 'cbnz'):
                        width = "X64" if 'x0' in ai.op_str.split(',')[0] else "W32"
                        marker = f"  <<< CHECKS x0 ({width})"
                print(f"      {fmt(ai)}{marker}")
                if ai.mnemonic == 'ret':
                    break

# ============================================================
# PART 6: The CRITICAL question - x19/x20/x22 in 0x10000F1EC
# What ARE these values? Can they have low32=0, high32!=0?
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 6: WHAT ARE x19/x20/x22 in 0x10000F1EC?")
print(f"# These registers are set by the caller before bl 0x10000F1EC")
print(f"# If any of them can be 0xXXXXXXXX00000000 -> VULNERABILITY")
print(f"{'#' * WIDTH}\n")

# Look at what sets x19, x20, x22 BEFORE the BL to 0x10000F1EC
# Since this is called from img4_verify, look at img4_verify context
for i in img4_instrs:
    if i.mnemonic == 'bl' and '0x10000f1ec' in i.op_str.lower():
        bl_off = i.address
        # Scan backwards in img4_instrs to find what sets x19, x20, x22
        print(f"  Scanning img4_verify for x19/x20/x22 assignments before BL at 0x{bl_off:X}:")
        for prev in img4_instrs:
            if prev.address >= bl_off:
                break
            ops = prev.op_str
            for reg in ['x19', 'x20', 'x22', 'w19', 'w20', 'w22']:
                if ops.startswith(f"{reg},") or ops.startswith(f"{reg} "):
                    # This instruction writes to the register
                    width = "64-bit" if reg.startswith('x') else "32-bit"
                    print(f"    0x{prev.address:X}: {prev.mnemonic:12s} {ops}  ({width} write)")

# ============================================================
# PART 7: Alternative - check 0x10000A890 and what it does
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 7: CODE AROUND 0x10000A890 (where img4_verify calls sub-func)")
print(f"{'#' * WIDTH}\n")

# Disassemble around 0x10000A890
ctx = disasm_at(0x10000A860, 40)
for i in ctx:
    marker = ""
    if i.mnemonic == 'bl':
        target = int(i.op_str.lstrip('#'), 16)
        if target == 0x10000F1EC:
            marker = "  <<< CALLS THE CRITICAL SUB-FUNC"
        else:
            marker = f"  <<< bl 0x{target:X}"
    if i.mnemonic in ('mov','movz') and i.op_str.startswith(('x0,','w0,')):
        width = "X64" if i.op_str.startswith('x0') else "W32"
        marker = f"  <<< SETS RETURN ({width})"
    if i.mnemonic == 'ret':
        marker = "  <<< RETURN"
    if i.mnemonic in ('cbz','cbnz') and ('x0' in i.op_str or 'w0' in i.op_str):
        marker = "  <<< CHECKS x0"
    print(f"{fmt(i)}{marker}")

# ============================================================
# PART 8: Exhaustive return value analysis
# What happens on EVERY exit path of img4_verify?
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 8: EVERY EXIT PATH OF img4_verify")
print(f"# For each RET, what was the last instruction setting x0/w0?")
print(f"{'#' * WIDTH}\n")

# Build a simple trace - for each instruction, track last x0 write
last_x0_write = None
exit_paths = []
for i in img4_instrs:
    if i.mnemonic in ('mov', 'movz', 'movk', 'movn') and i.op_str.startswith(('x0,', 'w0,')):
        width = 64 if i.op_str.startswith('x0') else 32
        last_x0_write = (i.address, i.mnemonic, i.op_str, width)
    elif i.mnemonic == 'bl':
        # After a BL, x0 could be anything the callee returns
        target = int(i.op_str.lstrip('#'), 16)
        last_x0_write = (i.address, 'bl', f'return from 0x{target:X}', 0)  # unknown width
    elif i.mnemonic == 'ret':
        exit_paths.append((i.address, last_x0_write))

for ret_addr, x0_info in exit_paths:
    if x0_info:
        addr, mne, ops, width = x0_info
        width_str = f"{width}-bit" if width else "UNKNOWN (callee return)"
        print(f"  RET @ 0x{ret_addr:X}: x0 set by 0x{addr:X}: {mne} {ops} [{width_str}]")
    else:
        print(f"  RET @ 0x{ret_addr:X}: x0 setter UNKNOWN")

# ============================================================
# PART 9: THE VERDICT
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 9: VERDICT - IS CBZ W8 EXPLOITABLE?")
print(f"{'#' * WIDTH}\n")

has_64bit_return = False
for ret_addr, x0_info in exit_paths:
    if x0_info:
        _, mne, ops, width = x0_info
        if width == 64 or width == 0:
            has_64bit_return = True

if has_64bit_return:
    print("  !!! POTENTIAL VULNERABILITY !!!")
    print("  img4_verify has return paths where x0 is 64-bit or from callee")
    print("  The signature gate uses 'cbz w8' which only checks lower 32 bits")
    print("  If ANY return path produces x0 with low32=0, high32!=0:")
    print("    -> cbz w8 sees SUCCESS")
    print("    -> Unsigned code EXECUTES")
    print()
    print("  EXPLOITABILITY DEPENDS ON:")
    print("  1. Can we reach a 64-bit return path in img4_verify?")
    print("  2. Can the returned value have low32=0?")
    print("  3. Is the value controllable by attacker input?")
else:
    print("  CLEAN: All return paths use 32-bit writes to w0")
    print("  cbz w8 correctly catches all error values")

print(f"\n{SEP}")
print(f"  TRACE COMPLETE")
print(f"{SEP}")
