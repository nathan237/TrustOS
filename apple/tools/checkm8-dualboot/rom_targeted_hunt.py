#!/usr/bin/env python3
"""
ROM TARGETED VULNERABILITY HUNT v3
===================================
Based on A0→B1 diff findings, this script:
1. Fully RE's the 6 NEW B1 functions (especially addr validation 0x100006D80)
2. Hunts ALL 32-bit size comparisons that should be 64-bit (the pattern Apple fixed)
3. Analyzes the 49 unchecked NULL returns in security context
4. Traces img4_verify return value for truncation bugs
5. Checks for integer overflow in size calculations near memory ops
6. Looks for remaining use-after-free patterns in USB/DFU code
"""

import struct
from capstone import *
from capstone.arm64 import *
from collections import defaultdict

ROM_FILE = "securerom/t8020_B1_securerom.bin"
ROM_BASE = 0x100000000

# Load ROM
with open(ROM_FILE, "rb") as f:
    rom = f.read()

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

def disasm_range(start_offset, size):
    """Disassemble a range from offset"""
    data = rom[start_offset:start_offset+size]
    return list(md.disasm(data, ROM_BASE + start_offset))

def disasm_func(func_addr, max_instr=200):
    """Disassemble a function starting at func_addr until RET"""
    offset = func_addr - ROM_BASE
    instrs = []
    for i in md.disasm(rom[offset:offset+max_instr*4], func_addr):
        instrs.append(i)
        if i.mnemonic == 'ret' and len(instrs) > 1:
            break
    return instrs

def disasm_func_full(func_addr, max_instr=500):
    """Disassemble a function fully, handling multiple return paths"""
    offset = func_addr - ROM_BASE
    instrs = []
    ret_count = 0
    for i in md.disasm(rom[offset:offset+max_instr*4], func_addr):
        instrs.append(i)
        if i.mnemonic == 'ret':
            ret_count += 1
            # Check if next instruction is a new function prologue
            next_off = (i.address - ROM_BASE) + 4
            if next_off + 8 <= len(rom):
                next_bytes = rom[next_off:next_off+8]
                # Check for STP x29,x30 or SUB sp pattern (function prologues)
                if next_bytes[3] in (0xa9,) and (next_bytes[2] & 0x3f) in range(0x00, 0x40):
                    break  # New function starts
                if ret_count >= 3:
                    break
    return instrs

print("=" * 120)
print("  ROM TARGETED VULNERABILITY HUNT v3 — DEEP ANALYSIS")
print("=" * 120)

# ============================================================================
# SECTION 1: FULL RE OF 6 NEW B1 FUNCTIONS
# ============================================================================
print("\n" + "█" * 120)
print("█ SECTION 1: FULL REVERSE ENGINEERING OF 6 NEW B1 FUNCTIONS")
print("█" * 120)

new_funcs = {
    0x100006CDC: "NEW_FUNC_1 (0 xrefs — dead code or indirect?)",
    0x100006D64: "NEW_FUNC_2 (1 xref — called from boot 0x1000017B0)",
    0x100006D80: "ADDR_VALIDATOR (3 xrefs — security-critical!)",
    0x10000955C: "NEW_FUNC_4 (0 xrefs — dead code or indirect?)",
    0x100009684: "NEW_FUNC_5 (0 xrefs — dead code or indirect?)",
    0x10001AC78: "NEW_FUNC_6 (0 xrefs — dead code or indirect?)",
}

for func_addr, name in new_funcs.items():
    print(f"\n{'#' * 120}")
    print(f"# {name}")
    print(f"# Address: 0x{func_addr:X}")
    print(f"{'#' * 120}")
    
    instrs = disasm_func_full(func_addr, max_instr=300)
    
    # Analyze function
    calls = []
    mmio_refs = []
    branches = []
    comparisons = []
    mem_ops = []
    sysreg_ops = []
    
    for i in instrs:
        print(f"  0x{i.address:X}: {i.mnemonic:12s} {i.op_str}")
        
        if i.mnemonic == 'bl':
            calls.append(i)
        elif i.mnemonic in ('b', 'b.eq', 'b.ne', 'b.hi', 'b.ls', 'b.lo', 'b.hs',
                           'b.gt', 'b.lt', 'b.ge', 'b.le', 'cbz', 'cbnz', 'tbz', 'tbnz'):
            branches.append(i)
        elif i.mnemonic in ('cmp', 'cmn', 'ccmp'):
            comparisons.append(i)
        elif i.mnemonic in ('ldr', 'str', 'ldp', 'stp', 'ldrb', 'strb', 'ldrh', 'strh'):
            mem_ops.append(i)
        elif i.mnemonic in ('msr', 'mrs'):
            sysreg_ops.append(i)
        
        # Check for MMIO references
        if i.mnemonic == 'movk' and 'lsl #16' in i.op_str:
            mmio_refs.append(i)
    
    print(f"\n  --- Analysis ---")
    print(f"  Instructions: {len(instrs)}")
    print(f"  Calls (BL): {len(calls)}")
    print(f"  Branches: {len(branches)}")
    print(f"  Comparisons: {len(comparisons)}")
    print(f"  Memory ops: {len(mem_ops)}")
    print(f"  MMIO refs: {len(mmio_refs)}")
    print(f"  Sysreg ops: {len(sysreg_ops)}")
    
    if calls:
        targets = []
        for c in calls:
            if '#' in c.op_str:
                targets.append('0x' + c.op_str.split('#')[1])
            else:
                targets.append(c.op_str)
        print(f"  Call targets: {', '.join(targets)}")
    if comparisons:
        for c in comparisons:
            width = "32-bit" if c.op_str.startswith("w") else "64-bit"
            print(f"  CMP: {c.op_str} ({width}) at 0x{c.address:X}")
    if mmio_refs:
        for m in mmio_refs:
            print(f"  MMIO: {m.op_str} at 0x{m.address:X}")

# ============================================================================
# SECTION 2: ADDR_VALIDATOR 0x100006D80 DEEP DIVE
# ============================================================================
print("\n" + "█" * 120)
print("█ SECTION 2: ADDRESS VALIDATOR 0x100006D80 — DEEP REVERSE ENGINEERING")
print("█ This function validates addresses in the 0x19C00xxxx-0x19C03xxxx range")
print("█ Called from 3 locations: 0x100001A04, 0x100006E80, 0x100008CBC")
print("█ QUESTION: Can we bypass the validation? Off-by-one? Range error?")
print("█" * 120)

# Disassemble more generously for sub-functions
addr_val = disasm_func_full(0x100006D80, max_instr=150)
print("\n  Full disassembly with annotations:")
for i in addr_val:
    ann = ""
    if i.mnemonic == 'cmp':
        width = "W32" if i.op_str.startswith("w") else "X64"
        ann = f"  ◄◄◄ {width} COMPARISON"
    elif i.mnemonic in ('cbz', 'cbnz', 'tbz', 'tbnz'):
        ann = f"  ◄◄◄ BRANCH"
    elif i.mnemonic in ('b.hi', 'b.lo', 'b.hs', 'b.ls', 'b.eq', 'b.ne', 'b.gt', 'b.lt', 'b.ge', 'b.le'):
        ann = f"  ◄◄◄ COND BRANCH"
    elif i.mnemonic == 'bl':
        ann = f"  ◄◄◄ CALL"
    elif 'movk' in i.mnemonic and '#0x9c0' in i.op_str:
        ann = f"  ◄◄◄ SRAM BASE"
    elif i.mnemonic == 'ret':
        ann = f"  ◄◄◄ RETURN"
    print(f"  0x{i.address:X}: {i.mnemonic:12s} {i.op_str:45s}{ann}")

# Now analyze each caller context
print("\n  Caller contexts:")
for caller_addr in [0x100001A04, 0x100006E80, 0x100008CBC]:
    print(f"\n  --- Caller at 0x{caller_addr:X} ---")
    # Show 10 instructions before and 5 after the call
    pre = disasm_range(caller_addr - ROM_BASE - 40, 80)
    for i in pre:
        marker = " >>>" if i.address == caller_addr else "    "
        print(f"  {marker} 0x{i.address:X}: {i.mnemonic:12s} {i.op_str}")

# ============================================================================
# SECTION 3: ALL 32-BIT SIZE COMPARISONS IN B1
# ============================================================================
print("\n" + "█" * 120)
print("█ SECTION 3: HUNTING 32-BIT TRUNCATION BUGS (the pattern Apple fixed A0→B1)")
print("█ A0 had `cmp w2, w20` (32-bit) which Apple fixed to `cmp x2, x24` (64-bit)")
print("█ Are there similar unfixed cases in B1?")
print("█" * 120)

# Disassemble all code
code_end = 0x24B40  # Code section end
all_instrs = list(md.disasm(rom[:code_end], ROM_BASE))

# Find all CMP instructions involving size-like operands
size_related_cmps = []
suspicious_cmps = []

for idx, i in enumerate(all_instrs):
    if i.mnemonic in ('cmp', 'cmn'):
        # Check if it's a 32-bit comparison
        is_32bit = i.op_str.startswith('w')
        is_64bit = i.op_str.startswith('x')
        
        # Look ahead for memory operations using the compared value
        # This pattern: CMP wA, wB -> B.xx -> LDR/STR with xA/xB indicates
        # that a 32-bit compare gates a 64-bit memory access (POTENTIAL BUG)
        if is_32bit:
            # Get the registers involved
            parts = i.op_str.split(',')
            if len(parts) >= 2:
                reg1 = parts[0].strip()
                reg2 = parts[1].strip()
                reg1_x = reg1.replace('w', 'x')
                reg2_x = reg2.replace('w', 'x')
                
                # Look at surrounding instructions for 64-bit use of same regs
                context_start = max(0, idx - 10)
                context_end = min(len(all_instrs), idx + 15)
                
                for j in range(context_start, context_end):
                    ci = all_instrs[j]
                    if j == idx:
                        continue
                    # Check if any surrounding instruction uses the x-variant of the compared register
                    if (reg1_x in ci.op_str or reg2_x in ci.op_str):
                        if ci.mnemonic in ('ldr', 'str', 'ldp', 'stp', 'add', 'sub',
                                          'bl', 'sxtw', 'ubfiz', 'sbfiz'):
                            suspicious_cmps.append({
                                'addr': i.address,
                                'cmp': f"{i.mnemonic} {i.op_str}",
                                'related': f"{ci.mnemonic} {ci.op_str} @ 0x{ci.address:X}",
                                'reg32': reg1 if reg1.startswith('w') else reg2,
                                'reg64_use': ci
                            })

# Filter for truly suspicious ones (near memory operations or size checks)
print(f"\n  Total 32-bit CMP/CMN in code: {sum(1 for i in all_instrs if i.mnemonic in ('cmp','cmn') and i.op_str.startswith('w'))}")
print(f"  Total 64-bit CMP/CMN in code: {sum(1 for i in all_instrs if i.mnemonic in ('cmp','cmn') and i.op_str.startswith('x'))}")
print(f"  Suspicious 32-bit CMP with nearby 64-bit use: {len(suspicious_cmps)}")

# Group by function (find nearest function prologue before each)
func_prologues = []
for i in all_instrs:
    if i.mnemonic == 'stp' and 'x29, x30' in i.op_str:
        func_prologues.append(i.address)

def find_func(addr):
    """Find the function containing this address"""
    for j in range(len(func_prologues) - 1, -1, -1):
        if func_prologues[j] <= addr:
            return func_prologues[j]
    return 0

# Security-critical regions for filtering
SECURITY_REGIONS = [
    (0x100001640, 0x100001E00, "Boot flow / DFU handler"),
    (0x100005000, 0x100006A00, "USB endpoint setup"),
    (0x100006700, 0x100007200, "Security dispatch / validation"),
    (0x10000A400, 0x10000B000, "img4_verify / signature check"),
    (0x10000E000, 0x10000F000, "Image loading / memory map"),
    (0x10000F000, 0x100010000, "Heap allocator"),
]

print(f"\n  Suspicious comparisons in SECURITY-CRITICAL regions:")
shown = set()
for s in suspicious_cmps:
    addr = s['addr']
    for start, end, name in SECURITY_REGIONS:
        if start <= addr <= end:
            key = f"{addr:#x}"
            if key not in shown:
                shown.add(key)
                func = find_func(addr)
                print(f"\n  ★ 0x{addr:X} (func 0x{func:X}) [{name}]")
                print(f"    CMP: {s['cmp']}")
                print(f"    64-bit use: {s['related']}")
                # Show context
                offset = addr - ROM_BASE - 20
                ctx = disasm_range(offset, 60)
                for ci in ctx:
                    marker = " >>>" if ci.address == addr else "    "
                    print(f"    {marker} 0x{ci.address:X}: {ci.mnemonic:12s} {ci.op_str}")

# ============================================================================
# SECTION 4: IMG4_VERIFY RETURN VALUE TRACE
# ============================================================================
print("\n" + "█" * 120)
print("█ SECTION 4: IMG4_VERIFY RETURN VALUE ANALYSIS")
print("█ B1 flow: bl #0x10000a704 → mov x8, x0 → cbz w8, #0x1C5C")
print("█ QUESTION: Does img4_verify ever return a value where w0=0 but x0≠0?")
print("█" * 120)

# Disassemble img4_verify fully
img4_func = disasm_func_full(0x10000A704, max_instr=500)
print(f"\n  img4_verify @ 0x10000A704: {len(img4_func)} instructions")

# Find all return value assignments (mov w0/x0)
ret_vals = []
for i in img4_func:
    if i.mnemonic == 'mov' and (i.op_str.startswith('w0,') or i.op_str.startswith('x0,')):
        ret_vals.append(i)
    elif i.mnemonic == 'csel' and (i.op_str.startswith('w0,') or i.op_str.startswith('x0,')):
        ret_vals.append(i)

print(f"\n  Return value assignments in img4_verify:")
for r in ret_vals:
    width = "W32" if r.op_str.startswith('w') else "X64"
    print(f"    0x{r.address:X}: {r.mnemonic} {r.op_str} ({width})")

# Check sub-call return values
print(f"\n  Sub-calls from img4_verify that could propagate return value:")
for idx, i in enumerate(img4_func):
    if i.mnemonic == 'bl':
        # Check if the return value is used as the function's return
        if idx + 1 < len(img4_func):
            next_i = img4_func[idx + 1]
            if next_i.mnemonic in ('cbnz', 'cbz') and ('w0' in next_i.op_str or 'x0' in next_i.op_str):
                print(f"    0x{i.address:X}: bl → checked with {next_i.mnemonic} {next_i.op_str}")
            elif next_i.mnemonic == 'mov' and 'x0' in next_i.op_str:
                print(f"    0x{i.address:X}: bl → return saved: {next_i.mnemonic} {next_i.op_str}")

# Trace the critical return path
print(f"\n  CRITICAL RETURN PATH (success → cbz):")
# Find the sub-function called right before the return
for idx, i in enumerate(img4_func):
    if i.mnemonic == 'bl':
        target = i.op_str.replace('#', '')
        target_addr = int(target, 16)
        # Disassemble target to see how it sets w0/x0
        sub_instrs = disasm_func(target_addr, max_instr=50)
        sub_rets = [s for s in sub_instrs if s.mnemonic == 'mov' and
                   (s.op_str.startswith('w0,') or s.op_str.startswith('x0,'))]
        if sub_rets:
            for sr in sub_rets:
                width = "W32" if sr.op_str.startswith('w') else "X64"
                if width == "X64" and '#0' not in sr.op_str:
                    print(f"    ★★★ 0x{i.address:X} calls 0x{target_addr:X}")
                    print(f"         Sub-func returns {width}: {sr.mnemonic} {sr.op_str} @ 0x{sr.address:X}")
                    print(f"         IF upper 32 bits non-zero, cbz w8 COULD MISS IT!")

# ============================================================================
# SECTION 5: SXTW (Sign Extension) AUDIT
# ============================================================================
print("\n" + "█" * 120)
print("█ SECTION 5: SIGN EXTENSION AUDIT (sxtw)")
print("█ Apple fixed A0's signed/unsigned confusion — are there more in B1?")
print("█ Pattern: sxtw x1, wN → used as size/offset → could be negative → huge!")
print("█" * 120)

sxtw_instrs = [i for i in all_instrs if i.mnemonic == 'sxtw']
print(f"\n  Total SXTW instructions in B1: {len(sxtw_instrs)}")
for s in sxtw_instrs:
    # Check context for sign-bit check BEFORE sxtw
    offset_idx = None
    for idx, i in enumerate(all_instrs):
        if i.address == s.address:
            offset_idx = idx
            break
    
    if offset_idx is None:
        continue
    
    # Look back for tbnz #0x1f (sign bit check)
    has_sign_check = False
    for j in range(max(0, offset_idx - 20), offset_idx):
        ci = all_instrs[j]
        if ci.mnemonic == 'tbnz' and '#0x1f' in ci.op_str:
            has_sign_check = True
            break
        if ci.mnemonic == 'tbz' and '#0x1f' in ci.op_str:
            has_sign_check = True
            break
    
    # Look forward for memory operations using the sign-extended value
    dest_reg = s.op_str.split(',')[0].strip()
    mem_use = None
    for j in range(offset_idx + 1, min(len(all_instrs), offset_idx + 15)):
        ci = all_instrs[j]
        if dest_reg in ci.op_str and ci.mnemonic in ('ldr', 'str', 'bl', 'add', 'cmp'):
            mem_use = ci
            break
    
    status = "✓ PROTECTED" if has_sign_check else "⚠ NO SIGN CHECK"
    func = find_func(s.address)
    
    if not has_sign_check:
        print(f"\n  ★ 0x{s.address:X} (func 0x{func:X}): {s.mnemonic} {s.op_str}  [{status}]")
        if mem_use:
            print(f"    Used in: {mem_use.mnemonic} {mem_use.op_str} @ 0x{mem_use.address:X}")
        # Show context
        start = s.address - ROM_BASE - 20
        ctx = disasm_range(start, 60)
        for ci in ctx:
            marker = " >>>" if ci.address == s.address else "    "
            print(f"    {marker} 0x{ci.address:X}: {ci.mnemonic:12s} {ci.op_str}")

# ============================================================================
# SECTION 6: INTEGER OVERFLOW BEFORE MEMORY ACCESS
# ============================================================================
print("\n" + "█" * 120)
print("█ SECTION 6: INTEGER OVERFLOW AUDIT")
print("█ Pattern: ADD/MADD/UMADDL computing size → no overflow check → memory access")
print("█" * 120)

overflow_suspects = []
for idx, i in enumerate(all_instrs):
    if i.mnemonic in ('add', 'madd', 'umaddl', 'mul') and idx + 10 < len(all_instrs):
        # Check if result flows to a memory operation without overflow check
        dest = i.op_str.split(',')[0].strip()
        has_overflow_check = False
        has_mem_use = False
        mem_instr = None
        
        for j in range(idx + 1, min(len(all_instrs), idx + 10)):
            ci = all_instrs[j]
            # Overflow checks: adds/subs with carry, b.vs, cmp
            if ci.mnemonic in ('adds', 'subs', 'b.vs', 'b.cs', 'b.cc'):
                has_overflow_check = True
                break
            if ci.mnemonic in ('cmp', 'cmn') and dest in ci.op_str:
                has_overflow_check = True
                break
            if dest in ci.op_str and ci.mnemonic in ('ldr', 'str', 'ldp', 'stp', 'ldrb', 'strb'):
                has_mem_use = True
                mem_instr = ci
        
        if has_mem_use and not has_overflow_check:
            func = find_func(i.address)
            # Only report in security regions
            for start, end, name in SECURITY_REGIONS:
                if start <= i.address <= end:
                    overflow_suspects.append({
                        'addr': i.address,
                        'instr': f"{i.mnemonic} {i.op_str}",
                        'mem': f"{mem_instr.mnemonic} {mem_instr.op_str}" if mem_instr else "?",
                        'func': func,
                        'region': name
                    })

print(f"\n  Unchecked arithmetic → memory in security regions: {len(overflow_suspects)}")
for s in overflow_suspects:
    print(f"\n  ★ 0x{s['addr']:X} (func 0x{s['func']:X}) [{s['region']}]")
    print(f"    Arithmetic: {s['instr']}")
    print(f"    Memory use: {s['mem']}")

# ============================================================================
# SECTION 7: UNCHECKED NULL IN SECURITY FUNCTIONS
# ============================================================================
print("\n" + "█" * 120)
print("█ SECTION 7: UNCHECKED NULL RETURNS — SECURITY CONTEXT")
print("█ 49 locations where x0 is used as pointer after BL without NULL check")
print("█ Focus: which ones are in security-critical code?")
print("█" * 120)

null_deref_addrs = [
    0x1000029D0, 0x100002B68, 0x100002DF4, 0x1000035C4, 0x100003838,
    0x100004334, 0x1000044D0, 0x1000045D0, 0x100004604, 0x100004628,
    0x1000046A4, 0x100004810, 0x1000048EC, 0x100004908, 0x100004928,
]  # First 15 from the 49

for addr in null_deref_addrs:
    func = find_func(addr)
    is_security = False
    region_name = ""
    for start, end, name in SECURITY_REGIONS:
        if start <= addr <= end:
            is_security = True
            region_name = name
            break
    
    if is_security:
        print(f"\n  ★ 0x{addr:X} (func 0x{func:X}) [{region_name}]")
        offset = addr - ROM_BASE - 24
        ctx = disasm_range(offset, 64)
        for ci in ctx:
            marker = " >>>" if ci.address == addr else "    "
            print(f"    {marker} 0x{ci.address:X}: {ci.mnemonic:12s} {ci.op_str}")

# ============================================================================
# SECTION 8: USB MMIO RACE CONDITIONS
# ============================================================================ 
print("\n" + "█" * 120)
print("█ SECTION 8: USB CONTROLLER MMIO — RACE CONDITION ANALYSIS")
print("█ 5 USB MMIO refs found at 0x200000000 + 0x39000000 region")
print("█ Are there TOCTOU races in how USB registers are read/written?")
print("█" * 120)

usb_funcs = [0x100002994, 0x100002A80, 0x100002B04, 0x100002BB0]
for func_addr in usb_funcs:
    instrs = disasm_func_full(func_addr, max_instr=200)
    print(f"\n  --- USB Function 0x{func_addr:X} ({len(instrs)} instrs) ---")
    
    # Find LDR/STR to MMIO and check for read-check-reread patterns
    mmio_reads = []
    mmio_writes = []
    for i in instrs:
        # MMIO accesses often use registers loaded with 0x200000000+0x39000000
        if i.mnemonic == 'ldr' and any(f'x{r}' in i.op_str for r in ['8','9','10','11','19','20','21']):
            mmio_reads.append(i)
        elif i.mnemonic == 'str' and any(f'x{r}' in i.op_str for r in ['8','9','10','11','19','20','21']):
            mmio_writes.append(i)
    
    # Check for barriers between reads and writes
    has_barriers = False
    for i in instrs:
        if i.mnemonic in ('dsb', 'dmb', 'isb'):
            has_barriers = True
            break
    
    print(f"    Memory reads: {len(mmio_reads)}, writes: {len(mmio_writes)}")
    print(f"    Memory barriers: {'YES' if has_barriers else 'NO ⚠'}")
    
    # Print full disassembly for interesting functions
    if not has_barriers and (mmio_reads or mmio_writes):
        print(f"    ⚠ NO MEMORY BARRIERS in USB function with MMIO access!")
        for i in instrs[:60]:
            print(f"    0x{i.address:X}: {i.mnemonic:12s} {i.op_str}")

# ============================================================================
# SECTION 9: STACK CANARY / COOKIE CHECK
# ============================================================================
print("\n" + "█" * 120)
print("█ SECTION 9: STACK CANARY ANALYSIS")
print("█ Does B1 use stack canaries? If not, stack buffer overflows are exploitable")
print("█" * 120)

# Stack canary pattern: LDR from fixed address at start, compare at end
canary_loads = []
for idx, i in enumerate(all_instrs):
    if i.mnemonic == 'ldr' and '#0x448' in i.op_str:
        canary_loads.append(i)

print(f"\n  Stack canary loads (ldr ..., [xx, #0x448]): {len(canary_loads)}")
for c in canary_loads:
    func = find_func(c.address)
    print(f"    0x{c.address:X} (func 0x{func:X})")

# Check if canary is checked before return
# Pattern: ldr reg, [stack]; adrp/ldr canary; sub; cbnz → __stack_chk_fail
canary_checks = 0
for idx, i in enumerate(all_instrs):
    if i.mnemonic == 'sub' and idx + 2 < len(all_instrs):
        next_i = all_instrs[idx + 1]
        if next_i.mnemonic == 'cbnz':
            # Could be stack check
            # Look ahead for bl to __stack_chk_fail equivalent
            for j in range(idx + 2, min(len(all_instrs), idx + 5)):
                if all_instrs[j].mnemonic == 'bl':
                    target = all_instrs[j].op_str.replace('#', '')
                    try:
                        target_addr = int(target, 16)
                        if target_addr == 0x100008B58:  # Known panic/assert function
                            canary_checks += 1
                    except:
                        pass

print(f"  Stack canary verifications found: {canary_checks}")

# The sub x8, x9, x8; cbnz x8, <fail> pattern near function epilogues
stack_check_pattern = []
for idx, i in enumerate(all_instrs):
    if i.mnemonic == 'sub' and idx + 1 < len(all_instrs):
        next_i = all_instrs[idx + 1]
        if next_i.mnemonic == 'cbnz' and idx - 3 >= 0:
            # Check if there was an adrp + ldr before (loading canary)
            prev = [all_instrs[idx - j] for j in range(1, 4)]
            for p in prev:
                if p.mnemonic == 'adrp' and '#0x19c00' in p.op_str:
                    stack_check_pattern.append(i)
                    break

print(f"  Stack canary check patterns (sub+cbnz after adrp 0x19c00): {len(stack_check_pattern)}")
for s in stack_check_pattern:
    func = find_func(s.address)
    print(f"    0x{s.address:X} (func 0x{func:X})")

# ============================================================================
# SECTION 10: COMPREHENSIVE SUMMARY
# ============================================================================
print("\n" + "█" * 120)
print("█ SECTION 10: COMPREHENSIVE VULNERABILITY SUMMARY")
print("█" * 120)

print("""
  FINDINGS FROM A0→B1 DIFF (bugs Apple FIXED):
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1. [FIXED] 32-bit comparison truncation (cmp w2,w20 → cmp x2,x24)
  2. [FIXED] Uninitialized SCTLR register (missing mov x0, xzr)
  3. [FIXED] Hardcoded size 0x1B0000 → dynamic x24 validation
  4. [FIXED] Missing sign-bit check (tbnz w26, #0x1f added in B1)
  5. [FIXED] Stack buffer not zeroed (5× bl 0x1000119CC added)
  6. [NEW]   Address validation function 0x100006D80 added

  POTENTIAL B1 VULNERABILITIES (new findings):
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
""")

print(f"  A. 32-bit truncation suspects in security regions: {len(shown)}")
print(f"  B. SXTW without sign-bit guard: {len([s for s in sxtw_instrs])}")
print(f"  C. Unchecked arithmetic → memory: {len(overflow_suspects)}")
print(f"  D. Stack canary checks: {len(stack_check_pattern)} (check if comprehensive)")
print(f"  E. USB functions without barriers: listed above")

print("\n" + "=" * 120)
print("  TARGETED HUNT v3 COMPLETE")
print("=" * 120)
