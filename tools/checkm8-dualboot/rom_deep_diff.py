#!/usr/bin/env python3
"""
Deep analysis of T8020 A0 vs B1 SecureROM differences.
Focus: DFU/USB handler, top hot spots, and security-relevant code changes.
Side-by-side disassembly of changed regions.
"""

import struct
from capstone import *

ROM_BASE = 0x100000000

# Load ROMs
a0 = open("securerom/t8020_A0_securerom.bin", "rb").read()
b1 = open("securerom/t8020_B1_securerom.bin", "rb").read()

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

def disasm(data, offset, addr, count=30):
    """Disassemble and return list of (addr, mnemonic, op_str) tuples."""
    result = []
    for insn in md.disasm(data[offset:offset+count*4], addr):
        result.append((insn.address, insn.mnemonic, insn.op_str))
        if len(result) >= count:
            break
    return result

def disasm_range(data, start_off, end_off, base=ROM_BASE):
    """Disassemble a byte range."""
    addr = base + start_off
    result = []
    for insn in md.disasm(data[start_off:end_off], addr):
        result.append((insn.address, insn.mnemonic, insn.op_str))
    return result

def print_side_by_side(a0_insns, b1_insns, label=""):
    """Print two instruction lists side by side."""
    if label:
        print(f"\n{'='*100}")
        print(f"  {label}")
        print(f"{'='*100}")
    
    max_len = max(len(a0_insns), len(b1_insns))
    print(f"  {'A0 (iBoot-3865.0.0.1.23)':<48} | {'B1 (iBoot-3865.0.0.4.7)':<48}")
    print(f"  {'-'*48}-+-{'-'*48}")
    
    for i in range(max_len):
        if i < len(a0_insns):
            a = a0_insns[i]
            a_str = f"0x{a[0]:X}: {a[1]} {a[2]}"
        else:
            a_str = ""
        if i < len(b1_insns):
            b = b1_insns[i]
            b_str = f"0x{b[0]:X}: {b[1]} {b[2]}"
        else:
            b_str = ""
        
        # Mark differences
        marker = " "
        if i < len(a0_insns) and i < len(b1_insns):
            if a0_insns[i][1] != b1_insns[i][1] or a0_insns[i][2] != b1_insns[i][2]:
                marker = "*"
        elif i >= len(a0_insns) or i >= len(b1_insns):
            marker = "+"
        
        print(f"{marker} {a_str:<48} | {b_str:<48}")

def analyze_region(name, offset, size=256, context=64):
    """Analyze a region in both ROMs with context."""
    print(f"\n{'#'*100}")
    print(f"# REGION: {name}")
    print(f"# Offset: 0x{offset:X} | ROM addr: 0x{ROM_BASE+offset:X}")
    print(f"# Size: {size} bytes")
    print(f"{'#'*100}")
    
    # Show with context
    start = max(0, offset - context)
    end = min(len(b1), offset + size + context)
    
    a0_insns = disasm_range(a0, start, end)
    b1_insns = disasm_range(b1, start, end)
    
    print_side_by_side(a0_insns, b1_insns, f"{name} — Full Disassembly")
    
    # Highlight the changed bytes
    print(f"\n  Changed bytes at 0x{offset:X}:")
    print(f"  A0: {a0[offset:offset+min(size,64)].hex()}")
    print(f"  B1: {b1[offset:offset+min(size,64)].hex()}")

print("=" * 100)
print("  T8020 SecureROM DEEP DIFF — A0 vs B1")
print("  Focus: Security-critical regions, DFU handler, and hot spots")
print("=" * 100)

# ============================================================================
# SECTION 1: Early boot / initialization (the first change at 0x3C4)
# ============================================================================
print("\n\n" + "█" * 100)
print("█ SECTION 1: EARLY BOOT INITIALIZATION (x0 register setup)")
print("█" * 100)
analyze_region("Early boot init — x0 register cleared in B1", 0x3C4, size=64)

# ============================================================================
# SECTION 2: DFU/USB Handler region — THE checkm8 area
# ============================================================================
print("\n\n" + "█" * 100)
print("█ SECTION 2: DFU/USB HANDLER — The checkm8 Target Area")
print("█ This is where checkm8 exploited A11. What changed for A12?")
print("█" * 100)

# The DFU handler string "Apple Mobile Device (DFU Mode)" is at:
# A0: 0x1BE9A, B1: 0x1C25A
# The DFU code is near these strings. Let's look at the code region before them.

# DFU handler entry — look at the function that references the DFU string
# In checkm8 on A11, the bug was in USB control transfer handling
# Specifically: a use-after-free in the DFU request handling

# Let's find the DFU handler code by looking at the changed area near 0x1BCA8
for region_off, region_size, label in [
    (0x1BCA8, 512, "DFU Handler start — near 'Apple Mobile Device (DFU Mode)'"),
    (0x1BE72, 512, "DFU Handler main body — large 319B change"),
    (0x1C05B, 256, "DFU region — 74B change near USB code"),
    (0x1C0A6, 256, "USB DART region — DART configuration changes"),
    (0x1C12F, 256, "Malloc/memory region near DFU handler"),
]:
    analyze_region(label, region_off, size=region_size)

# ============================================================================
# SECTION 3: THE CRITICAL CBZ — img4 signature verification gate
# ============================================================================
print("\n\n" + "█" * 100)
print("█ SECTION 3: THE CRITICAL cbz w8, #0x1C5C @ 0x1BC8")
print("█ This is the one instruction that gates boot: pass/fail signature check")
print("█" * 100)

# In B1, the critical cbz is at offset 0x1BC8
# In A0, what's at the same location? Did it move?
# Let's check both ROMs around 0x1BC8
analyze_region("Signature verification gate — cbz at 0x1BC8", 0x1B80, size=256)

# Also look for cbz w8 patterns near img4_verify return
# img4_verify is at 0xA704 in B1
print("\n--- img4_verify vicinity (0xA700) ---")
analyze_region("img4_verify function area", 0xA700, size=512)

# ============================================================================
# SECTION 4: TOP HOT SPOTS from scoring
# ============================================================================
print("\n\n" + "█" * 100)
print("█ SECTION 4: TOP 5 HOT SPOTS (highest change scores)")
print("█" * 100)

hot_spots = [
    (0x13AB0, 129, "#1 score=2850 — largest change"),
    (0x1271C, 62, "#2 score=2512 — function restructuring with NULL checks"),
    (0x14F24, 118, "#3 score=2250 — compression/decompression?"),
    (0x0C628, 88, "#4 score=1938 — completely different logic"),
    (0x10DC8, 86, "#5 score=1936 — unknown function"),
]

for offset, size, label in hot_spots:
    analyze_region(f"HOT SPOT {label}", offset, size=max(size, 256), context=96)

# ============================================================================
# SECTION 5: Conditional branch changes near security functions
# ============================================================================
print("\n\n" + "█" * 100)
print("█ SECTION 5: SECURITY-CRITICAL BRANCH CHANGES")
print("█ Branches that changed condition or target — potential patch locations")
print("█" * 100)

# Key addresses where branches changed (from the diff output)
# Focus on areas near known security functions
security_branches = [
    (0x1BC8, "B1 cbz w8 @ 0x1BC8 — THE signature gate"),
    (0x1964, "B1 cbz w0 @ 0x1964 — early branch change"),
    (0x19B4, "A0 tbnz w25 bit31 @ 0x19B4 — sign bit check"),
    (0x1AE0, "B1 tbnz w26 bit31 @ 0x1AE0 — sign bit check added in B1"),
    (0x1C78, "A0 cbnz w8 @ 0x1C78 — branch on non-zero removed?"),
    (0x2D9C, "Both: cbnz x8 — target changed"),
    (0x2DC4, "A0 only: tbnz w2 bit9 — register flag check"),
    (0x2F48, "B1 only: tbz w2 bit9 — INVERTED bit check!"),
]

for offset, label in security_branches:
    print(f"\n--- {label} ---")
    start = max(0, offset - 48)
    end = min(len(b1), offset + 48)
    
    a0_insns = disasm_range(a0, start, end)
    b1_insns = disasm_range(b1, start, end)
    
    print(f"  A0 @ 0x{ROM_BASE+offset:X}:")
    for addr, mn, op in a0_insns:
        marker = " >>>" if (addr - ROM_BASE) == offset else "    "
        print(f"  {marker} 0x{addr:X}: {mn} {op}")
    
    print(f"  B1 @ 0x{ROM_BASE+offset:X}:")
    for addr, mn, op in b1_insns:
        marker = " >>>" if (addr - ROM_BASE) == offset else "    "
        print(f"  {marker} 0x{addr:X}: {mn} {op}")

# ============================================================================
# SECTION 6: INVERTED BIT CHECK — potentially exploitable logic error
# ============================================================================
print("\n\n" + "█" * 100)
print("█ SECTION 6: INVERTED BIT CHECKS — tbnz→tbz / cbz→cbnz changes")
print("█ When Apple inverts a condition, it may indicate a logic fix")
print("█ But inversions can also introduce NEW logic bugs")
print("█" * 100)

# Scan for all places where a condition was inverted between A0 and B1
# This means cbz→cbnz, tbnz→tbz, b.eq→b.ne, b.lo→b.hs, etc.
# These are the most likely candidates for incomplete patches

inversions = []
common_len = min(len(a0), len(b1))

# Disassemble both fully in the code region
print("\n  Scanning for inverted conditions in code region (0x000-0x24B40)...")
a0_code = list(md.disasm(a0[0:0x24B40], ROM_BASE))
b1_code = list(md.disasm(b1[0:0x24B40], ROM_BASE))

# Build address->instruction maps  
a0_map = {insn.address: insn for insn in a0_code}
b1_map = {insn.address: insn for insn in b1_code}

# Check for inversions at same address
inverse_pairs = {
    ('cbz', 'cbnz'), ('cbnz', 'cbz'),
    ('tbz', 'tbnz'), ('tbnz', 'tbz'),
    ('b.eq', 'b.ne'), ('b.ne', 'b.eq'),
    ('b.lo', 'b.hs'), ('b.hs', 'b.lo'),
    ('b.lt', 'b.ge'), ('b.ge', 'b.lt'),
    ('b.le', 'b.gt'), ('b.gt', 'b.le'),
    ('b.hi', 'b.ls'), ('b.ls', 'b.hi'),
    ('b.mi', 'b.pl'), ('b.pl', 'b.mi'),
}

print(f"\n  Inverted conditions at matching addresses:")
inversion_count = 0
for addr in sorted(set(a0_map.keys()) & set(b1_map.keys())):
    a_mn = a0_map[addr].mnemonic
    b_mn = b1_map[addr].mnemonic
    if (a_mn, b_mn) in inverse_pairs:
        inversion_count += 1
        print(f"    0x{addr:X}: A0={a_mn} {a0_map[addr].op_str}  →  B1={b_mn} {b1_map[addr].op_str}")
        if inversion_count <= 10:
            # Show context
            for ctx_off in range(-16, 20, 4):
                ctx_addr = addr + ctx_off
                a_str = f"{a0_map[ctx_addr].mnemonic} {a0_map[ctx_addr].op_str}" if ctx_addr in a0_map else "???"
                b_str = f"{b1_map[ctx_addr].mnemonic} {b1_map[ctx_addr].op_str}" if ctx_addr in b1_map else "???"
                marker = ">>>" if ctx_off == 0 else "   "
                print(f"      {marker} 0x{ctx_addr:X}: [{a_str:<35}] | [{b_str:<35}]")
            print()

print(f"\n  Total inverted conditions at same address: {inversion_count}")

# ============================================================================
# SECTION 7: Functions that exist in A0 but NOT in B1 (removed code)
# ============================================================================
print("\n\n" + "█" * 100)
print("█ SECTION 7: FUNCTION-LEVEL ANALYSIS")
print("█ Looking for function boundaries via STP x29, x30 prologues")
print("█" * 100)

# Better function detection: look for stp x29, x30 patterns
# STP x29, x30, [sp, #imm] = FD 7B xx A9
def find_functions(data, name):
    functions = []
    for i in range(0, min(len(data), 0x24B40), 4):
        word = struct.unpack_from('<I', data, i)[0]
        # STP x29, x30, [sp, #offset]! or STP x29, x30, [sp, #offset]
        # Encoding: x01x1001 xx011101 xxxxxxxx 10101001 
        # Simplified: look for FD 7B at bytes [0:2] and A9 at byte [3]
        if (data[i] == 0xFD and data[i+1] == 0x7B and data[i+3] == 0xA9):
            functions.append(ROM_BASE + i)
        # Also: SUB sp, sp, #imm (FF xx xx D1) as function entry
    return functions

a0_funcs = find_functions(a0, "A0")
b1_funcs = find_functions(b1, "B1")

print(f"  A0 function prologues (stp x29, x30): {len(a0_funcs)}")
print(f"  B1 function prologues (stp x29, x30): {len(b1_funcs)}")

# Find functions only in one version
a0_only = set(a0_funcs) - set(b1_funcs)
b1_only = set(b1_funcs) - set(a0_funcs)
common = set(a0_funcs) & set(b1_funcs)

print(f"  At same address in both: {len(common)}")
print(f"  Only in A0: {len(a0_only)}")
print(f"  Only in B1: {len(b1_only)}")

if a0_only:
    print(f"\n  Functions ONLY in A0 (removed or moved in B1):")
    for addr in sorted(a0_only)[:30]:
        off = addr - ROM_BASE
        insns = list(md.disasm(a0[off:off+20*4], addr))[:5]
        code = "; ".join(f"{i.mnemonic} {i.op_str}" for i in insns)
        print(f"    0x{addr:X}: {code}")

if b1_only:
    print(f"\n  Functions ONLY in B1 (added or moved from A0):")
    for addr in sorted(b1_only)[:30]:
        off = addr - ROM_BASE
        insns = list(md.disasm(b1[off:off+20*4], addr))[:5]
        code = "; ".join(f"{i.mnemonic} {i.op_str}" for i in insns)
        print(f"    0x{addr:X}: {code}")

# ============================================================================
# SECTION 8: CHECKM8-SPECIFIC — USB request handling comparison
# ============================================================================
print("\n\n" + "█" * 100)
print("█ SECTION 8: CHECKM8-SPECIFIC — USB Request Handling")
print("█ checkm8 used a use-after-free in USB control transfer handling.")
print("█ The fix likely added: proper request lifecycle, heap hardening,")
print("█ or state machine changes. Let's find the USB request code.")
print("█" * 100)

# Search for USB-related patterns in both ROMs:
# - References to USB descriptor strings
# - USB control transfer constants (bmRequestType patterns)
# - DFU class-specific codes (0x21 = DFU class)

# Find xrefs to the DFU string
dfu_str_a0 = a0.find(b"Apple Mobile Device (DFU Mode)")
dfu_str_b1 = b1.find(b"Apple Mobile Device (DFU Mode)")
print(f"  DFU string: A0 @ 0x{dfu_str_a0:X}, B1 @ 0x{dfu_str_b1:X}")
print(f"  String offset shift: {dfu_str_b1 - dfu_str_a0} bytes")

# Look for USB setup packet handling (8-byte setup packet)
# bmRequestType, bRequest, wValue, wIndex, wLength
# DFU_DNLOAD = 1, DFU_UPLOAD = 2, DFU_GETSTATUS = 3
# DFU_CLRSTATUS = 4, DFU_GETSTATE = 5, DFU_ABORT = 6
# DFU_DETACH = 0

# Search for immediate values 0x21 (DFU class request type) in code
print("\n  Searching for DFU class request handling (0x21 = bmRequestType)...")
for name, code_insns in [("A0", a0_code), ("B1", b1_code)]:
    dfu_refs = []
    for insn in code_insns:
        op = insn.op_str
        # Look for comparisons with 0x21 or immediate loads of 0x21
        if "#0x21" in op and insn.mnemonic in ('cmp', 'mov', 'movz', 'and', 'tst'):
            dfu_refs.append(insn)
    print(f"  {name}: {len(dfu_refs)} references to #0x21")
    for ref in dfu_refs[:15]:
        print(f"    0x{ref.address:X}: {ref.mnemonic} {ref.op_str}")

# Search for free/dealloc patterns — the use-after-free bug
print("\n  Looking for memory deallocation patterns near USB code...")
# In the DFU handler region (0x1B000-0x1C500)
for name, data, code_insns in [("A0", a0, a0_code), ("B1", b1, b1_code)]:
    # Find BL (function calls) in the DFU region
    calls_in_dfu = []
    for insn in code_insns:
        off = insn.address - ROM_BASE
        if 0x1B000 <= off <= 0x1C500 and insn.mnemonic == 'bl':
            target = int(insn.op_str.lstrip('#'), 16)
            calls_in_dfu.append((insn.address, target))
    
    # Count call targets to find frequently called helpers (likely alloc/free)
    from collections import Counter
    target_counts = Counter(t for _, t in calls_in_dfu)
    print(f"\n  {name} — Function calls in DFU region (0x1B000-0x1C500):")
    print(f"  Total calls: {len(calls_in_dfu)}")
    print(f"  Unique targets: {len(target_counts)}")
    for target, count in target_counts.most_common(15):
        # Check if target is in malloc/free area
        print(f"    0x{target:X} called {count}x")

# ============================================================================
# SECTION 9: HEAP HARDENING — malloc/free changes
# ============================================================================
print("\n\n" + "█" * 100)
print("█ SECTION 9: MEMORY MANAGEMENT — malloc/free Changes")
print("█ checkm8 was a heap use-after-free. Apple likely hardened heap ops.")
print("█" * 100)

# Find "malloc() returns NULL" string and its xrefs
malloc_str_a0 = a0.find(b"malloc() returns NULL")
malloc_str_b1 = b1.find(b"malloc() returns NULL")
print(f"  'malloc() returns NULL' string: A0 @ 0x{malloc_str_a0:X}, B1 @ 0x{malloc_str_b1:X}")

# Disassemble around malloc area
if malloc_str_b1 > 0:
    # The malloc function is likely referenced from nearby code
    # Look backwards from the string for code that references it
    print("\n  Code near malloc string in B1:")
    # The code that calls malloc is before the string
    # Let's look at functions that reference this area
    analyze_region("Malloc implementation area", malloc_str_b1 - 512, size=768)

# ============================================================================  
# SECTION 10: LOOK FOR SUBTLE BUGS IN B1
# ============================================================================
print("\n\n" + "█" * 100)
print("█ SECTION 10: POTENTIAL VULNERABILITY PATTERNS IN B1")
print("█ Searching for common ARM64 vulnerability patterns:")
print("█ - Missing bounds checks before memory access")  
print("█ - Unchecked function return values")
print("█ - Integer overflow/truncation")
print("█ - Type confusion (comparing wrong-size registers)")
print("█" * 100)

# Pattern 1: BL followed by no check on w0 (return value ignored)
print("\n  Pattern 1: Function calls with unchecked return values")
unchecked_returns = []
b1_insn_list = list(b1_code)
for i in range(len(b1_insn_list) - 1):
    insn = b1_insn_list[i]
    next_insn = b1_insn_list[i + 1]
    if insn.mnemonic == 'bl':
        # Check if next instruction uses w0/x0 in a comparison
        if next_insn.mnemonic not in ('cbz', 'cbnz', 'cmp', 'tst', 'tbnz', 'tbz'):
            # Also not a mov from x0 (saving return value)
            if 'x0' not in next_insn.op_str and 'w0' not in next_insn.op_str:
                off = insn.address - ROM_BASE
                # Only in security-relevant areas
                if (0x1800 <= off <= 0x2500) or (0xA600 <= off <= 0xAC00) or (0x1B000 <= off <= 0x1C500):
                    unchecked_returns.append((insn, next_insn))

print(f"  Found {len(unchecked_returns)} unchecked returns in security regions")
for bl_insn, next_insn in unchecked_returns[:20]:
    print(f"    0x{bl_insn.address:X}: {bl_insn.mnemonic} {bl_insn.op_str}")
    print(f"      next: {next_insn.mnemonic} {next_insn.op_str}")

# Pattern 2: W-register comparison after X-register load (truncation)
print("\n  Pattern 2: Potential register width mismatches (truncation bugs)")
width_mismatches = []
for i in range(len(b1_insn_list) - 2):
    insn = b1_insn_list[i]
    # LDR Xn (64-bit load) followed by CMP Wn (32-bit compare)
    if insn.mnemonic == 'ldr' and insn.op_str.startswith('x'):
        reg_num = insn.op_str.split(',')[0][1:]  # get register number
        for j in range(1, 4):
            if i + j < len(b1_insn_list):
                next_i = b1_insn_list[i + j]
                if next_i.mnemonic in ('cmp', 'cbz', 'cbnz') and f'w{reg_num}' in next_i.op_str:
                    off = insn.address - ROM_BASE
                    if off < 0x24B40:
                        width_mismatches.append((insn, next_i))
                    break

print(f"  Found {len(width_mismatches)} potential width mismatches")
for ld_insn, cmp_insn in width_mismatches[:15]:
    print(f"    0x{ld_insn.address:X}: {ld_insn.mnemonic} {ld_insn.op_str}")
    print(f"      then: 0x{cmp_insn.address:X}: {cmp_insn.mnemonic} {cmp_insn.op_str}")

# Pattern 3: Integer overflow — ADD without overflow check
print("\n  Pattern 3: ADD/SUB near memory operations without overflow check")
arith_before_mem = []
for i in range(len(b1_insn_list) - 3):
    insn = b1_insn_list[i]
    if insn.mnemonic in ('add', 'sub') and '#' in insn.op_str:
        # Check if followed by a memory access using the result register
        dst_reg = insn.op_str.split(',')[0].strip()
        for j in range(1, 4):
            if i + j < len(b1_insn_list):
                next_i = b1_insn_list[i + j]
                if next_i.mnemonic in ('str', 'ldr', 'stp', 'ldp') and dst_reg in next_i.op_str:
                    # Check if there's NO bounds check between them
                    has_check = False
                    for k in range(1, j):
                        mid = b1_insn_list[i + k]
                        if mid.mnemonic in ('cmp', 'cbz', 'cbnz', 'tbnz', 'tbz', 'b.hs', 'b.lo'):
                            has_check = True
                            break
                    if not has_check:
                        off = insn.address - ROM_BASE
                        # Only in DFU/USB area
                        if 0x1B000 <= off <= 0x1C500:
                            arith_before_mem.append((insn, next_i))
                    break

print(f"  Found {len(arith_before_mem)} unchecked arithmetic-then-memory in DFU area")
for arith, mem in arith_before_mem[:15]:
    print(f"    0x{arith.address:X}: {arith.mnemonic} {arith.op_str}")
    print(f"      → 0x{mem.address:X}: {mem.mnemonic} {mem.op_str}")

print("\n\n" + "=" * 100)
print("  DEEP ANALYSIS COMPLETE")
print("=" * 100)
