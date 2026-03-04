#!/usr/bin/env python3
"""
DEEP TRACE: 0x100005480 — THE REAL SIGNATURE VERIFIER
=====================================================
img4_verify calls this function at 0x10000A810: bl #0x100005480
Then checks: cbnz w0, #0x10000a838 (32-bit only!)

If 0x100005480 returns x0 where lower 32 bits = 0 but upper != 0:
  -> cbnz w0 PASSES (sees zero)
  -> img4_verify returns w0 = 0 (SUCCESS)
  -> Boot code executes unsigned image

This script:
1. Fully disassembles 0x100005480
2. Maps ALL return paths and what x0/w0 contains
3. Traces sub-calls and their return types
4. Determines if ANY path can produce high32!=0, low32=0
5. Also checks the SXTW at 0x10000D6D4 (missed by scanner)
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

def fmt(i):
    return f"  0x{i.address:X}: {i.mnemonic:16s} {i.op_str}"

WIDTH = 120
SEP = "=" * WIDTH

# ============================================================
# PART 1: Disassemble 0x100005480 - find ALL return paths
# ============================================================
print(SEP)
print("  DEEP TRACE: 0x100005480 - THE REAL SIGNATURE VERIFICATION FUNCTION")
print(SEP)

print(f"\n{'#' * WIDTH}")
print(f"# PART 1: FULL DISASSEMBLY of 0x100005480")
print(f"# This is the function img4_verify delegates to for actual verification")
print(f"{'#' * WIDTH}\n")

# This could be a large function. Disassemble until we find a clear end.
# We'll look for function prologues after RETs to determine end.
max_instrs = 600
all_instrs = disasm_at(0x100005480, max_instrs)

func_instrs = []
ret_addrs = []
call_targets = set()
x0_setters = []  # (addr, mnemonic, op_str, width)
branches = []

# Find function end: look for STP x29,x30 prologue after a RET
in_func = True
last_ret = None
for i in all_instrs:
    if i.mnemonic == 'ret':
        last_ret = i.address
        func_instrs.append(i)
        ret_addrs.append(i.address)
        continue
    
    # Check if this is a new function prologue after a ret
    if last_ret is not None:
        # If we see STP with x29 or SUB sp, it's likely a new function
        if i.mnemonic == 'stp' and 'x29' in i.op_str and 'sp' in i.op_str:
            break
        if i.mnemonic == 'sub' and i.op_str.startswith('sp,'):
            break
    
    func_instrs.append(i)
    
    # Track x0/w0 setters
    if i.mnemonic in ('mov', 'movz', 'movk', 'movn', 'orr', 'and', 'add', 'sub'):
        ops = i.op_str.split(',')
        if ops[0].strip() in ('x0', 'w0'):
            width = 64 if ops[0].strip() == 'x0' else 32
            x0_setters.append((i.address, i.mnemonic, i.op_str, width))
    
    # Track BL          
    if i.mnemonic == 'bl':
        target_str = i.op_str.lstrip('#')
        try:
            target = int(target_str, 16)
            call_targets.add(target)
        except:
            pass
    
    # Track branches
    if i.mnemonic.startswith('b') and i.mnemonic not in ('bl', 'blr', 'br'):
        branches.append((i.address, i.mnemonic, i.op_str))

# Print full disassembly with annotations
for i in func_instrs:
    marker = ""
    ops_parts = i.op_str.split(',')
    dst = ops_parts[0].strip() if ops_parts else ""
    
    if i.mnemonic in ('mov', 'movz', 'movk', 'movn') and dst in ('x0', 'w0'):
        width = "X64" if dst == 'x0' else "W32"
        marker = f"  <<< SETS x0 ({width})"
    elif i.mnemonic in ('orr', 'and', 'add', 'sub') and dst in ('x0', 'w0'):
        width = "X64" if dst == 'x0' else "W32"
        marker = f"  <<< MODIFIES x0 ({width})"
    elif i.mnemonic == 'bl':
        target_str = i.op_str.lstrip('#')
        try:
            target = int(target_str, 16)
            marker = f"  <<< CALL (x0 = callee return)"
        except:
            marker = f"  <<< CALL"
    elif i.mnemonic == 'blr':
        marker = f"  <<< INDIRECT CALL (x0 = callee return)"
    elif i.mnemonic == 'ret':
        marker = "  <<< RETURN"
    elif i.mnemonic in ('cbz', 'cbnz') and ('x0' in i.op_str or 'w0' in i.op_str):
        width = "64" if 'x0' in i.op_str.split(',')[0] else "32"
        marker = f"  <<< BRANCHES on x0 ({width}-bit)"
    elif i.mnemonic in ('ldr', 'ldp') and dst in ('x0', 'w0'):
        width = "X64" if dst == 'x0' else "W32"
        marker = f"  <<< LOADS into x0 ({width})"
    
    print(f"{fmt(i)}{marker}")

print(f"\n  Function size: {len(func_instrs)} instructions (0x{func_instrs[0].address:X} - 0x{func_instrs[-1].address:X})")
print(f"  Return points: {len(ret_addrs)}")
for r in ret_addrs:
    print(f"    RET @ 0x{r:X}")
print(f"  Call targets: {len(call_targets)}")
for t in sorted(call_targets):
    print(f"    0x{t:X}")

# ============================================================
# PART 2: Map every return path - what is x0 on each one?
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 2: RETURN VALUE ANALYSIS for 0x100005480")
print(f"# For each RET instruction, trace backwards to find last x0 setter")
print(f"{'#' * WIDTH}\n")

# Simple approach: collect ALL x0 writes with their widths
print("  ALL x0/w0 write points in 0x100005480:")
for addr, mne, ops, width in x0_setters:
    print(f"    0x{addr:X}: {mne:8s} {ops:40s} [{width}-bit]")

# For each BL call, x0 could be set by the callee
print(f"\n  ALL BL calls (x0 = callee return, unknown width):")
for i in func_instrs:
    if i.mnemonic in ('bl', 'blr'):
        print(f"    0x{i.address:X}: {i.mnemonic:8s} {i.op_str}")

# ============================================================
# PART 3: Trace EACH sub-call's return type
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 3: SUB-CALL RETURN VALUE TYPES")
print(f"# For each function called by 0x100005480, what does it return?")
print(f"{'#' * WIDTH}\n")

for target in sorted(call_targets):
    # Disassemble the target function to find its return value patterns
    sub_instrs = disasm_at(target, 300)
    sub_x0_writes = []
    found_ret = False
    
    for si in sub_instrs:
        parts = si.op_str.split(',')
        dst = parts[0].strip() if parts else ""
        
        if si.mnemonic in ('mov', 'movz', 'movk', 'movn') and dst in ('x0', 'w0'):
            width = 64 if dst == 'x0' else 32
            sub_x0_writes.append((si.address, si.mnemonic, si.op_str, width))
        elif si.mnemonic in ('orr', 'and', 'add', 'sub') and dst in ('x0', 'w0'):
            width = 64 if dst == 'x0' else 32
            sub_x0_writes.append((si.address, si.mnemonic, si.op_str, width))
        elif si.mnemonic in ('ldr', 'ldp') and dst in ('x0', 'w0'):
            width = 64 if dst == 'x0' else 32
            sub_x0_writes.append((si.address, si.mnemonic, si.op_str, width))
        
        if si.mnemonic == 'ret':
            found_ret = True
            break
    
    has_64bit = any(w == 64 for _, _, _, w in sub_x0_writes)
    flag = " *** 64-BIT RETURN ***" if has_64bit else ""
    print(f"  Sub-function 0x{target:X}:{flag}")
    for addr, mne, ops, width in sub_x0_writes:
        print(f"    0x{addr:X}: {mne:8s} {ops:40s} [{width}-bit]")
    if not sub_x0_writes:
        print(f"    (no x0/w0 writes found - may propagate caller's x0)")
    print()

# ============================================================
# PART 4: Specifically look at the CRITICAL return path
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 4: CRITICAL ANALYSIS - CAN 0x100005480 RETURN HIGH32!=0, LOW32=0?")
print(f"{'#' * WIDTH}\n")

# Collect all x0 writes that are 64-bit
writes_64 = [(a, m, o) for a, m, o, w in x0_setters if w == 64]
# Collect all x0 writes that are 32-bit
writes_32 = [(a, m, o) for a, m, o, w in x0_setters if w == 32]

print(f"  64-bit x0 writes: {len(writes_64)}")
for a, m, o in writes_64:
    print(f"    0x{a:X}: {m} {o}")

print(f"\n  32-bit w0 writes: {len(writes_32)}")
for a, m, o in writes_32:
    print(f"    0x{a:X}: {m} {o}")

if writes_64:
    print(f"\n  !!! WARNING: 0x100005480 has 64-bit x0 writes !!!")
    print(f"  These could produce values where low32=0 but high32!=0")
    print(f"  This would bypass the `cbnz w0` check in img4_verify!")
    print(f"  NEED TO DETERMINE: what values can these x0 writes hold?")
    
    # For each 64-bit write, trace what register/value is being assigned
    for a, m, o in writes_64:
        src = o.split(',')[1].strip() if ',' in o else '?'
        print(f"\n  --- 0x{a:X}: {m} {o} ---")
        print(f"  Source register: {src}")
        # Look for what sets this source register nearby
        for fi in func_instrs:
            if fi.address >= a:
                break
            parts = fi.op_str.split(',')
            if parts and parts[0].strip() == src:
                print(f"    Defined at 0x{fi.address:X}: {fi.mnemonic} {fi.op_str}")
else:
    print(f"\n  SAFE: All x0 writes in 0x100005480 use 32-bit w0")
    print(f"  mov/movk to w0 automatically zero-extends upper 32 bits")
    print(f"  Therefore cbnz w0 in img4_verify correctly detects all errors")

# ============================================================
# PART 5: Check ALL callee functions too
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 5: DO ANY SUB-CALLS RETURN 64-BIT VALUES?")
print(f"# If 0x100005480 passes through a callee's x0, the width matters")
print(f"{'#' * WIDTH}\n")

for target in sorted(call_targets):
    sub_instrs = disasm_at(target, 300)
    has_64_ret = False
    for si in sub_instrs:
        parts = si.op_str.split(',')
        dst = parts[0].strip() if parts else ""
        if si.mnemonic in ('mov',) and dst == 'x0':
            has_64_ret = True
        if si.mnemonic == 'ret':
            break
    
    if has_64_ret:
        print(f"  !!! 0x{target:X}: returns via x0 (64-bit)")
        # Show the relevant instructions
        for si in sub_instrs:
            parts = si.op_str.split(',')
            dst = parts[0].strip() if parts else ""
            if (si.mnemonic in ('mov',) and dst == 'x0') or si.mnemonic == 'ret':
                print(f"      0x{si.address:X}: {si.mnemonic} {si.op_str}")
            if si.mnemonic == 'ret':
                break
    else:
        print(f"  OK  0x{target:X}: returns via w0 (32-bit) or no x0 write")

# ============================================================
# PART 6: After the BL to sub-calls, does 0x100005480 check w0 or x0?
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 6: HOW DOES 0x100005480 USE RETURN VALUES FROM SUB-CALLS?")
print(f"# After each BL, is x0 checked as 32-bit or 64-bit?")
print(f"{'#' * WIDTH}\n")

for idx, i in enumerate(func_instrs):
    if i.mnemonic in ('bl', 'blr'):
        # Look at next few instructions to see how x0 is used
        print(f"  After {i.mnemonic} {i.op_str} @ 0x{i.address:X}:")
        for j in range(idx + 1, min(idx + 6, len(func_instrs))):
            ni = func_instrs[j]
            relevance = ""
            if 'x0' in ni.op_str or 'w0' in ni.op_str:
                parts = ni.op_str.split(',')
                if ni.mnemonic in ('cbz', 'cbnz'):
                    width = "64-bit" if 'x0' in parts[0] else "32-bit"
                    relevance = f"  *** CHECKS x0 as {width} ***"
                elif ni.mnemonic == 'mov' and parts[0].strip() in ('x0', 'w0'):
                    relevance = "  (overwrites x0)"
                elif ni.mnemonic == 'mov':
                    relevance = "  (saves x0)"
                elif ni.mnemonic in ('str', 'stp'):
                    relevance = "  (stores x0)"
            print(f"    0x{ni.address:X}: {ni.mnemonic:12s} {ni.op_str}{relevance}")
        print()

# ============================================================
# PART 7: BONUS - Find ALL SXTW in full ROM
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 7: ALL SXTW INSTRUCTIONS IN ROM (scanner said 0, but we found 1)")
print(f"{'#' * WIDTH}\n")

sxtw_count = 0
full_rom_instrs = list(md.disasm(rom, ROM_BASE, len(rom) // 4))
for i in full_rom_instrs:
    if i.mnemonic == 'sxtw':
        sxtw_count += 1
        # Show context
        print(f"  0x{i.address:X}: {i.mnemonic} {i.op_str}")

# Also check for SBFM with extent=31 (equivalent to SXTW)
# SXTW Xd, Wn is encoded as SBFM Xd, Xn, #0, #31
print(f"\n  Total SXTW found: {sxtw_count}")

# Check for SBFM equivalents
sbfm_count = 0
for i in full_rom_instrs:
    if i.mnemonic == 'sbfm' and '#0x1f' in i.op_str:
        sbfm_count += 1
        print(f"  SBFM(sxtw): 0x{i.address:X}: {i.mnemonic} {i.op_str}")
print(f"  Total SBFM #31 (sxtw equivalent): {sbfm_count}")

# Also SXTB, SXTH which could cause issues
for ext_type in ['sxtb', 'sxth']:
    for i in full_rom_instrs:
        if i.mnemonic == ext_type:
            print(f"  {ext_type}: 0x{i.address:X}: {i.mnemonic} {i.op_str}")

print(f"\n{SEP}")
print(f"  TRACE COMPLETE")
print(f"{SEP}")
