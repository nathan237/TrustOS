#!/usr/bin/env python3
"""
T8020 SecureROM — Critical Function Deep Comparison.
Now that we've matched functions, let's compare:
1. The boot function at 0x1000017A4 (259 insns in A0 vs 333 in B1)
2. img4_verify: A0@0x10000A3A0 vs B1@0x10000A704
3. The 6 NEW functions unique to B1
4. The USB/DFU handler functions (matched by call graph)
"""

import struct
from collections import defaultdict
from capstone import *

ROM_BASE = 0x100000000

a0 = open("securerom/t8020_A0_securerom.bin", "rb").read()
b1 = open("securerom/t8020_B1_securerom.bin", "rb").read()

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

def find_functions(data, code_end=0x24B40):
    functions = []
    for i in range(0, min(len(data), code_end), 4):
        if i + 3 < len(data) and data[i] == 0xFD and data[i+1] == 0x7B and data[i+3] == 0xA9:
            functions.append(ROM_BASE + i)
    return sorted(set(functions))

a0_funcs = find_functions(a0)
b1_funcs = find_functions(b1)

def get_func_insns(data, addr, all_funcs, max_insns=600):
    """Get all instructions until next function or RET."""
    off = addr - ROM_BASE
    idx = all_funcs.index(addr) if addr in all_funcs else -1
    if idx >= 0 and idx + 1 < len(all_funcs):
        max_end = all_funcs[idx + 1] - ROM_BASE
    else:
        max_end = min(len(data), off + 4096)
    
    insns = []
    ret_count = 0
    for insn in md.disasm(data[off:max_end], addr):
        insns.append(insn)
        if insn.mnemonic == 'ret':
            ret_count += 1
            # Allow multiple rets (error paths), but stop after 3 consecutive
            if ret_count >= 3:
                break
        else:
            ret_count = 0
        if len(insns) >= max_insns:
            break
    return insns

def print_func(insns, label):
    """Print function disassembly with annotations."""
    print(f"\n  {label}:")
    for i, insn in enumerate(insns):
        ann = ""
        if insn.mnemonic == 'bl':
            ann = "  ; CALL"
        elif insn.mnemonic in ('cbz', 'cbnz'):
            ann = "  ; NULL/ZERO CHECK"
        elif insn.mnemonic in ('tbz', 'tbnz'):
            ann = "  ; BIT TEST"
        elif insn.mnemonic in ('cmp', 'tst'):
            ann = "  ; COMPARE"
        elif insn.mnemonic in ('b.eq', 'b.ne', 'b.lo', 'b.hs', 'b.hi', 'b.ls', 'b.gt', 'b.lt', 'b.ge', 'b.le'):
            ann = "  ; COND BRANCH"
        elif insn.mnemonic == 'ret':
            ann = "  ; RETURN"
        elif insn.mnemonic in ('str', 'stp') and 'sp' in insn.op_str:
            ann = "  ; STACK SAVE"
        elif insn.mnemonic in ('ldr', 'ldp') and 'sp' in insn.op_str:
            ann = "  ; STACK LOAD"
        elif 'adrp' in insn.mnemonic or 'adr' == insn.mnemonic:
            ann = "  ; ADDR REF"
        
        print(f"    {i:3d} 0x{insn.address:X}: {insn.mnemonic:<8} {insn.op_str:<45}{ann}")

print("=" * 120)
print("  T8020 SecureROM — CRITICAL FUNCTION DEEP COMPARISON")
print("=" * 120)

# ============================================================================
# 1. THE BOOT FUNCTION @ 0x1000017A4
# ============================================================================
print("\n\n" + "#" * 120)
print("# 1. THE BOOT FUNCTION — 0x1000017A4")  
print("#    A0: 259 instructions | B1: 333 instructions | B1 has +74 more!")
print("#    This is the function containing THE cbz w8 signature gate")
print("#" * 120)

boot_a0 = get_func_insns(a0, 0x1000017A4, a0_funcs, max_insns=400)
boot_b1 = get_func_insns(b1, 0x1000017A4, b1_funcs, max_insns=500)

print(f"\n  A0 boot function: {len(boot_a0)} instructions")
print(f"  B1 boot function: {len(boot_b1)} instructions")

# Side-by-side with alignment by BL calls
print(f"\n  {'A0':<60} | {'B1':<60}")
print(f"  {'-'*60}-+-{'-'*60}")

# Build BL sequences for both
a0_bls = [(i, insn) for i, insn in enumerate(boot_a0) if insn.mnemonic == 'bl']
b1_bls = [(i, insn) for i, insn in enumerate(boot_b1) if insn.mnemonic == 'bl']

print(f"\n  Function calls comparison:")
print(f"  A0: {len(a0_bls)} calls | B1: {len(b1_bls)} calls")
print(f"\n  {'#':>3} {'A0 idx':>6} {'A0 call target':<30} | {'B1 idx':>6} {'B1 call target':<30}")
print(f"  {'-'*3} {'-'*6} {'-'*30}-+-{'-'*6} {'-'*30}")

max_calls = max(len(a0_bls), len(b1_bls))
for i in range(max_calls):
    a_str = ""
    b_str = ""
    if i < len(a0_bls):
        a_idx, a_insn = a0_bls[i]
        a_str = f"{a_idx:>6} {a_insn.mnemonic} {a_insn.op_str}"
    if i < len(b1_bls):
        b_idx, b_insn = b1_bls[i]
        b_str = f"{b_idx:>6} {b_insn.mnemonic} {b_insn.op_str}"
    marker = " " if i < len(a0_bls) and i < len(b1_bls) else "+"
    print(f"  {i:3d} {a_str:<37} | {b_str:<37}")

# Print both functions fully
print_func(boot_a0, "A0 Boot Function @ 0x1000017A4 (FULL)")
print_func(boot_b1, "B1 Boot Function @ 0x1000017A4 (FULL)")

# Find the cbz w8 in both
print("\n\n  === Searching for the img4_verify call + cbz gate ===")
for name, insns in [("A0", boot_a0), ("B1", boot_b1)]:
    for i, insn in enumerate(insns):
        if insn.mnemonic == 'cbz' and 'w8' in insn.op_str:
            # Show context: 5 before and 5 after
            start = max(0, i - 8)
            end = min(len(insns), i + 8)
            print(f"\n  {name} — cbz w8 found at instruction #{i}:")
            for j in range(start, end):
                marker = ">>>" if j == i else "   "
                print(f"    {marker} 0x{insns[j].address:X}: {insns[j].mnemonic} {insns[j].op_str}")

# ============================================================================
# 2. IMG4_VERIFY FUNCTION COMPARISON
# ============================================================================
print("\n\n" + "#" * 120)
print("# 2. img4_verify — A0 @ 0x10000A3A0 vs B1 @ 0x10000A704")
print("#    Image verification function that checks iBoot signature")
print("#" * 120)

img4_a0 = get_func_insns(a0, 0x10000A3A0, a0_funcs, max_insns=150)
img4_b1 = get_func_insns(b1, 0x10000A704, b1_funcs, max_insns=150)

print(f"\n  A0 img4_verify: {len(img4_a0)} instructions")
print(f"  B1 img4_verify: {len(img4_b1)} instructions")

# Side by side
max_len = max(len(img4_a0), len(img4_b1))
print(f"\n  {'#':>3} {'A0 @ 0x10000A3A0':<55} | {'B1 @ 0x10000A704':<55}")
print(f"  {'-'*3} {'-'*55}-+-{'-'*55}")

for i in range(max_len):
    a_str = ""
    b_str = ""
    if i < len(img4_a0):
        a = img4_a0[i]
        a_str = f"0x{a.address:X}: {a.mnemonic} {a.op_str}"
    if i < len(img4_b1):
        b = img4_b1[i]
        b_str = f"0x{b.address:X}: {b.mnemonic} {b.op_str}"
    
    marker = " "
    if i < len(img4_a0) and i < len(img4_b1):
        if img4_a0[i].mnemonic != img4_b1[i].mnemonic:
            marker = "!"
        elif img4_a0[i].op_str != img4_b1[i].op_str:
            marker = "~"
    else:
        marker = "+"
    
    print(f"  {i:3d}{marker}{a_str:<55} | {b_str:<55}")

# ============================================================================
# 3. THE 6 NEW B1 FUNCTIONS
# ============================================================================
print("\n\n" + "#" * 120)
print("# 3. NEW FUNCTIONS IN B1 (not present in A0)")
print("#    These are Apple's security additions between steppings")
print("#" * 120)

new_b1_funcs = [
    0x100006CDC,
    0x100006D64,
    0x100006D80,
    0x10000955C,
    0x100009684,
    0x10001AC78,
]

for func_addr in new_b1_funcs:
    insns = get_func_insns(b1, func_addr, b1_funcs, max_insns=50)
    print(f"\n  === NEW B1 Function @ 0x{func_addr:X} ({len(insns)} insns) ===")
    
    # Analyze what it does
    calls = [i for i in insns if i.mnemonic == 'bl']
    sysregs = [i for i in insns if i.mnemonic in ('mrs', 'msr')]
    mem_ops = [i for i in insns if i.mnemonic in ('ldr', 'str', 'ldp', 'stp')]
    branches = [i for i in insns if i.mnemonic in ('cbz', 'cbnz', 'tbz', 'tbnz', 'b.eq', 'b.ne')]
    
    print(f"    Calls: {len(calls)}, SysRegs: {len(sysregs)}, MemOps: {len(mem_ops)}, Branches: {len(branches)}")
    
    for insn in insns:
        ann = ""
        if insn.mnemonic == 'bl':
            ann = "  ; CALL"
        elif insn.mnemonic in ('mrs', 'msr'):
            ann = "  ; SYS REG"
        elif insn.mnemonic in ('cbz', 'cbnz'):
            ann = "  ; NULL CHECK"
        elif 'adrp' in insn.mnemonic:
            ann = "  ; ADDR"
        print(f"    0x{insn.address:X}: {insn.mnemonic:<8} {insn.op_str:<45}{ann}")

# ============================================================================
# 4. WHO CALLS THE NEW B1 FUNCTIONS?
# ============================================================================
print("\n\n" + "#" * 120)
print("# 4. CROSS-REFERENCES TO NEW B1 FUNCTIONS")
print("#    Who calls these new functions? This shows where Apple added hardening.")
print("#" * 120)

# Scan ALL code for BL to new function addresses
b1_code = list(md.disasm(b1[0:0x24B40], ROM_BASE))

for func_addr in new_b1_funcs:
    callers = []
    for insn in b1_code:
        if insn.mnemonic == 'bl':
            try:
                target = int(insn.op_str.lstrip('#'), 16)
                if target == func_addr:
                    callers.append(insn.address)
            except:
                pass
    
    print(f"\n  0x{func_addr:X} called from {len(callers)} locations:")
    for caller in callers[:20]:
        # Find which function contains this caller
        containing_func = None
        for f in b1_funcs:
            if f <= caller:
                containing_func = f
            else:
                break
        print(f"    0x{caller:X} (in function 0x{containing_func:X})" if containing_func else f"    0x{caller:X}")

# ============================================================================
# 5. FIND WHAT'S DIFFERENT IN THE BOOT FUNCTION
# ============================================================================
print("\n\n" + "#" * 120)
print("# 5. BOOT FUNCTION DIFF — What Apple ADDED in B1")
print("#    The 74 extra instructions in B1's boot function are the key")
print("#" * 120)

# Use a simple LCS-like approach to find added/removed instructions
# by matching on mnemonic+operand patterns

def simplify_insn(insn):
    """Normalize instruction for matching (remove address-dependent parts)."""
    op = insn.op_str
    # Normalize addresses in bl/b/adr
    if insn.mnemonic in ('bl', 'b', 'adr', 'adrp') or insn.mnemonic.startswith('b.'):
        # Keep the mnemonic but mark the target type
        if insn.mnemonic == 'bl':
            return f"bl FUNC"
        return f"{insn.mnemonic} TARGET"
    if insn.mnemonic in ('cbz', 'cbnz', 'tbz', 'tbnz'):
        # Keep the register but normalize the target
        reg = op.split(',')[0]
        return f"{insn.mnemonic} {reg}, TARGET"
    return f"{insn.mnemonic} {op}"

a0_simplified = [simplify_insn(i) for i in boot_a0]
b1_simplified = [simplify_insn(i) for i in boot_b1]

# Find instructions in B1 not in A0 (approximate via sequence matching)
# Use simple diff
from difflib import SequenceMatcher
matcher = SequenceMatcher(None, a0_simplified, b1_simplified)

print("\n  Diff operations (A0 → B1):")
for op, a_start, a_end, b_start, b_end in matcher.get_opcodes():
    if op == 'equal':
        continue
    
    if op == 'replace':
        print(f"\n  CHANGED (A0[{a_start}:{a_end}] → B1[{b_start}:{b_end}]):")
        print(f"    A0:")
        for i in range(a_start, min(a_end, a_start + 10)):
            insn = boot_a0[i]
            print(f"      0x{insn.address:X}: {insn.mnemonic} {insn.op_str}")
        if a_end - a_start > 10:
            print(f"      ... ({a_end - a_start - 10} more)")
        print(f"    B1:")
        for i in range(b_start, min(b_end, b_start + 10)):
            insn = boot_b1[i]
            print(f"      0x{insn.address:X}: {insn.mnemonic} {insn.op_str}")
        if b_end - b_start > 10:
            print(f"      ... ({b_end - b_start - 10} more)")
    
    elif op == 'insert':
        print(f"\n  ADDED in B1 ({b_end - b_start} insns at B1[{b_start}:{b_end}]):")
        for i in range(b_start, min(b_end, b_start + 15)):
            insn = boot_b1[i]
            print(f"    + 0x{insn.address:X}: {insn.mnemonic} {insn.op_str}")
        if b_end - b_start > 15:
            print(f"    ... ({b_end - b_start - 15} more)")
    
    elif op == 'delete':
        print(f"\n  REMOVED from A0 ({a_end - a_start} insns at A0[{a_start}:{a_end}]):")
        for i in range(a_start, min(a_end, a_start + 10)):
            insn = boot_a0[i]
            print(f"    - 0x{insn.address:X}: {insn.mnemonic} {insn.op_str}")
        if a_end - a_start > 10:
            print(f"    ... ({a_end - a_start - 10} more)")

# ============================================================================
# 6. DFU HANDLER — Find by tracing from USB strings
# ============================================================================
print("\n\n" + "#" * 120)
print("# 6. DFU HANDLER IDENTIFICATION")
print("#    The DFU handler is the function processing USB control requests")  
print("#    In checkm8, the bug was in USB request handling")
print("#" * 120)

# Find references to the DFU string in code (ADRP/ADR patterns)
dfu_str_b1 = b1.find(b"Apple Mobile Device (DFU Mode)")
dfu_str_a0 = a0.find(b"Apple Mobile Device (DFU Mode)")

print(f"  DFU string: A0 @ 0x{dfu_str_a0:X}, B1 @ 0x{dfu_str_b1:X}")

# Search for ADRP instructions that point near these strings
print("\n  Searching for code references to DFU string...")

for name, data, string_off, code in [("A0", a0, dfu_str_a0, None), ("B1", b1, dfu_str_b1, None)]:
    refs = []
    page = (ROM_BASE + string_off) & ~0xFFF  # 4KB page
    
    for i in range(0, 0x24B40, 4):
        # Check for ADRP that targets the right page
        word = struct.unpack_from('<I', data, i)[0]
        if (word & 0x9F000000) == 0x90000000:  # ADRP
            rd = word & 0x1F
            immhi = (word >> 5) & 0x7FFFF
            immlo = (word >> 29) & 0x3
            imm = (immhi << 2) | immlo
            if imm & 0x100000:
                imm -= 0x200000
            target_page = ((ROM_BASE + i) & ~0xFFF) + (imm << 12)
            if abs(target_page - page) <= 0x1000:
                refs.append((ROM_BASE + i, rd, target_page))
    
    print(f"\n  {name}: {len(refs)} ADRP references near DFU string page (0x{page:X}):")
    for addr, rd, target in refs[:20]:
        # Find containing function
        func_list = a0_funcs if name == "A0" else b1_funcs
        containing = None
        for f in func_list:
            if f <= addr:
                containing = f
            elif f > addr:
                break
        print(f"    0x{addr:X}: adrp x{rd}, #0x{target:X} (in func 0x{containing:X})" if containing else f"    0x{addr:X}: adrp x{rd}, #0x{target:X}")

# ============================================================================
# 7. USB CONTROL TRANSFER HANDLER — The checkm8 vulnerability area
# ============================================================================
print("\n\n" + "#" * 120)
print("# 7. USB CONTROL TRANSFER HANDLING")
print("#    Looking for USB setup packet processing (8-byte USB control request)")
print("#    checkm8 exploited: USB control transfer → DFU DNLOAD → use-after-free")
print("#" * 120)

# In USB, the device processes SETUP packets. DFU uses class requests.
# bRequest values: DNLOAD=1, UPLOAD=2, GETSTATUS=3, CLRSTATUS=4, GETSTATE=5, ABORT=6
# Look for comparison chains like cmp w8, #1; b.eq ...; cmp w8, #2; etc.

print("\n  Looking for USB request dispatch (cmp + branch chains)...")
for name, data, func_list in [("A0", a0, a0_funcs), ("B1", b1, b1_funcs)]:
    code_insns = list(md.disasm(data[0:0x24B40], ROM_BASE))
    
    # Find sequences of CMP + B.xx with consecutive small values
    dispatch_candidates = []
    for i in range(len(code_insns) - 10):
        insn = code_insns[i]
        if insn.mnemonic == 'cmp' and '#1' in insn.op_str and ',' in insn.op_str:
            # Check if followed by more comparisons with 2, 3, 4, 5
            vals_found = {1}
            for j in range(i+1, min(i+20, len(code_insns))):
                if code_insns[j].mnemonic == 'cmp':
                    for v in [2, 3, 4, 5, 6]:
                        if f'#{v}' in code_insns[j].op_str:
                            vals_found.add(v)
            if len(vals_found) >= 3:  # At least comparing with 1, 2, 3
                dispatch_candidates.append((insn.address, vals_found))
    
    print(f"\n  {name}: {len(dispatch_candidates)} potential USB request dispatch tables:")
    for addr, vals in dispatch_candidates[:10]:
        # Find containing function
        containing = None
        for f in func_list:
            if f <= addr:
                containing = f
            elif f > addr:
                break
        print(f"    0x{addr:X} (in func 0x{containing:X}): compares with {sorted(vals)}")

# ============================================================================
# 8. HEAP FUNCTIONS — malloc, free, memalign  
# ============================================================================
print("\n\n" + "#" * 120)
print("# 8. MEMORY MANAGEMENT FUNCTIONS")
print("#    Finding malloc/free/memcpy by their behavior patterns")
print("#" * 120)

# "malloc() returns NULL" string indicates the malloc function
malloc_null_a0 = a0.find(b"malloc() returns NULL")
malloc_null_b1 = b1.find(b"malloc() returns NULL")
print(f"  'malloc() returns NULL': A0 @ 0x{malloc_null_a0:X}, B1 @ 0x{malloc_null_b1:X}")

# Find the function that references this string
for name, data, string_off, func_list in [
    ("A0", a0, malloc_null_a0, a0_funcs),
    ("B1", b1, malloc_null_b1, b1_funcs)
]:
    page = (ROM_BASE + string_off) & ~0xFFF
    # Find ADRP near this string
    for i in range(0, 0x24B40, 4):
        word = struct.unpack_from('<I', data, i)[0]
        if (word & 0x9F000000) == 0x90000000:
            immhi = (word >> 5) & 0x7FFFF
            immlo = (word >> 29) & 0x3
            imm = (immhi << 2) | immlo
            if imm & 0x100000:
                imm -= 0x200000
            target_page = ((ROM_BASE + i) & ~0xFFF) + (imm << 12)
            if target_page == page:
                containing = None
                for f in func_list:
                    if f <= ROM_BASE + i:
                        containing = f
                    elif f > ROM_BASE + i:
                        break
                if containing:
                    print(f"  {name}: malloc referenced at 0x{ROM_BASE+i:X} (func 0x{containing:X})")
                    break

print("\n\n" + "=" * 120)
print("  CRITICAL FUNCTION DEEP COMPARISON COMPLETE")
print("=" * 120)
