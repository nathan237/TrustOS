#!/usr/bin/env python3
"""Trace CPFM fuse reads and security decisions in SecureROM"""
from capstone import *
from collections import defaultdict

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

with open('securerom/t8020_B1_securerom.bin', 'rb') as f:
    rom = f.read()
base = 0x100000000

# === 1. Find the CPFM format string and trace where CPFM is used ===
print("=== WHERE IS CPFM READ & USED? ===")
# The DFU serial string: "CPID:%04X CPRV:%02X CPFM:%02X SCEP:%02X BDID:%02X ECID:%016llX IBFL:%02X"
# We know from A12 research that CPFM bits:
#   bit 0 = production mode (1=production, 0=development)
#   bit 1 = security mode (1=secure, 0=insecure)
# In production iPhones, CPFM = 0x03 (both bits set)
# In development boards, CPFM = 0x00 or 0x01

# Find where CPFM value is stored in SRAM and where it's used for decisions
# Typically read from a fuse register and stored in a global

# First, search for where the ROM reads fuses
# Apple fuse controller is typically at a specific MMIO address
# On A12, the fuse controller base varies

# Let's find all ADRP/ADD combos that reference the known SRAM data area
print("\n--- SRAM DATA REFERENCES (global variables) ---")
sram_refs = defaultdict(list)
insns = list(md.disasm(rom[0x400:0x25000], base+0x400))
for idx, inst in enumerate(insns):
    if inst.mnemonic == 'adrp' and '0x19c0' in inst.op_str:
        page = int(inst.op_str.split('#')[1], 16)
        # Find the ADD that follows
        if idx + 1 < len(insns):
            next_i = insns[idx+1]
            if next_i.mnemonic in ('add', 'ldr', 'str') and 'nop' not in next_i.op_str:
                sram_refs[page].append((inst.address, next_i.mnemonic, next_i.op_str))

for page, refs in sorted(sram_refs.items()):
    if len(refs) > 0:
        print(f"  PAGE {page:#010x}: {len(refs)} refs")
        for addr, mn, op in refs[:3]:
            print(f"    {addr:#010x}: {mn} {op}")
        if len(refs) > 3:
            print(f"    ... and {len(refs)-3} more")

# === 2. TRACE THE BOOT DECISION PATH MORE DEEPLY ===
# We know the critical path is:
#   img4_verify (0xA704) → returns status → cbz w8, 0x1C5C at 0x1BC8
# But WHAT makes img4_verify succeed or fail?
# Specifically, does it check CPFM to decide how strict to be?

print("\n\n=== TRACING img4_verify AND ITS CALLEES ===")
# img4_verify calls sub-functions. Let's find all BL targets from 0xA704 region
def disasm_func(start_off, max_insns=200):
    """Disassemble a function until RET"""
    insns = []
    for inst in md.disasm(rom[start_off:start_off+max_insns*4], base+start_off):
        insns.append(inst)
        if inst.mnemonic == 'ret':
            break
    return insns

# Trace all BL targets from the img4 verification flow
# Start from 0x1A88 (which is the function that calls img4_verify)
print("\n--- Call tree from load_and_verify (0x1A88) ---")
visited_funcs = set()
def trace_bl_targets(start_off, depth=0, max_depth=3):
    if depth > max_depth or start_off in visited_funcs:
        return
    visited_funcs.add(start_off)
    func = disasm_func(start_off)
    bl_targets = []
    for inst in func:
        if inst.mnemonic == 'bl':
            target = int(inst.op_str.split('#')[1], 16) if '#' in inst.op_str else 0
            if target and target != start_off + base:
                bl_targets.append(target)
    
    indent = "  " * depth
    print(f"{indent}FUNC {base+start_off:#010x}: {len(func)} insns, {len(bl_targets)} calls")
    for target in bl_targets:
        target_off = target - base
        if 0 < target_off < len(rom):
            # Show first instructions
            first_insns = list(md.disasm(rom[target_off:target_off+16], target, count=4))
            summary = ' → '.join(f"{x.mnemonic}" for x in first_insns[:3])
            print(f"{indent}  → BL {target:#010x}: {summary}")
            trace_bl_targets(target_off, depth+1, max_depth)

trace_bl_targets(0x1A88, 0, 2)

# === 3. WHAT DOES THE ROM DO DIFFERENTLY IN DEV vs PROD? ===
# Search for all places that check specific bits that could be CPFM
print("\n\n=== ALL TBZ/TBNZ (single bit checks) - potential fuse/flag reads ===")
bit_checks = []
for inst in insns:
    if inst.mnemonic in ('tbz', 'tbnz'):
        bit_checks.append((inst.address, inst.mnemonic, inst.op_str))

print(f"Total TBZ/TBNZ: {len(bit_checks)}")
# Group by which bit is tested
from collections import Counter
bit_test_stats = Counter()
for addr, mn, op in bit_checks:
    parts = op.split(',')
    if len(parts) >= 2:
        bit_num = parts[1].strip().replace('#', '')
        bit_test_stats[f"bit{bit_num}"] += 1

print("Bit test frequency:")
for bit, cnt in bit_test_stats.most_common(10):
    print(f"  {bit}: {cnt} times")

# === 4. FIND THE FUNCTION THAT READS CPFM FROM FUSES ===
# CPFM is typically read during early boot and stored in SRAM
# Look for functions that build the DFU serial string (which includes CPFM)
print("\n\n=== TRACING DFU SERIAL STRING BUILDER ===")
# Find the string address
cpfm_str_off = None
for off in range(len(rom)):
    if rom[off:off+4] == b'CPFM':
        cpfm_str_off = off
        break

if cpfm_str_off:
    str_addr = base + cpfm_str_off
    print(f"CPFM string found at {str_addr:#010x}")
    
    # Find where this string is referenced (ADRP+ADD to nearby page)
    str_page = str_addr & ~0xFFF
    str_off_in_page = str_addr & 0xFFF
    
    # Search for ADRP pointing to this page
    for inst in insns:
        if inst.mnemonic == 'adrp' and str_page != 0:
            target = int(inst.op_str.split('#')[1], 16) if '#' in inst.op_str else 0
            if target == str_page:
                # Check if next instruction adds the right offset
                idx = insns.index(inst) if inst in insns else -1
                if idx >= 0 and idx + 1 < len(insns):
                    next_i = insns[idx+1]
                    if next_i.mnemonic == 'add':
                        # Check offset
                        pass
                # Print context
                ctx_start = max(0, insns.index(inst) - 5) if inst in insns else 0
                ctx_end = min(len(insns), insns.index(inst) + 15) if inst in insns else 0

# === 5. LOOK AT WHAT 0x23B080000 BLOCK DOES ===
# This is the MASSIVE 115-register block - almost certainly PMGR (Power Manager)
# PMGR controls clock gates - could enable/disable debug hardware
print("\n\n=== 0x23B080000 BLOCK ANALYSIS (likely PMGR) ===")
# Find code that accesses this range
# It's in the literal pool data tables at 0x1F838+
# Let's trace the actual CODE that uses these tables
# The literal pool entries are in a table structure starting at 0x1F838

# Check the format - they're at regular 0x40 byte intervals
print("Peripheral register table format (at end of ROM):")
for off in range(0x1F838, min(0x1FC00, len(rom)), 0x40):
    entries = []
    for i in range(0, 0x40, 8):
        val = int.from_bytes(rom[off+i:off+i+8], 'little')
        if val > 0:
            entries.append(f"{val:#014x}")
    if entries:
        print(f"  {base+off:#010x}: {entries[0]} + {len(entries)-1} more")

# === 6. LOOK FOR PRODUCTION/DEVELOPMENT CONDITIONALS ===
# Check: are there functions that return different values based on a global variable?
# Especially functions like: is_production_mode(), is_secure_boot_enabled()
print("\n\n=== SMALL UTILITY FUNCTIONS (potential flag checkers) ===")
# Find tiny functions (< 10 instructions) that load a global and return
for idx in range(len(insns) - 10):
    if insns[idx].mnemonic == 'adrp' and '0x19c0' in insns[idx].op_str:
        # Check if this is start of a small function (preceded by ret or func boundary)
        if idx > 0 and insns[idx-1].mnemonic in ('ret', 'b'):
            # This might be a function start
            func_insns = []
            for j in range(idx, min(idx+10, len(insns))):
                func_insns.append(insns[j])
                if insns[j].mnemonic == 'ret':
                    break
            
            if len(func_insns) <= 8 and func_insns[-1].mnemonic == 'ret':
                # Small function that reads from SRAM - could be a flag checker
                has_ldr = any(x.mnemonic in ('ldr', 'ldrb', 'ldrh') for x in func_insns)
                if has_ldr:
                    print(f"\n  FLAG CHECKER at {insns[idx].address:#010x}:")
                    for fi in func_insns:
                        print(f"    {fi.address:#010x}: {fi.mnemonic:8s} {fi.op_str}")

# === 7. THE ULTIMATE QUESTION: Where does img4 check production mode? ===
print("\n\n=== IMG4 VERIFICATION: WHAT DECIDES PASS/FAIL ===")
# The img4 verification at 0xA704 - show its full disassembly with comments
func_start = 0xA704
func = disasm_func(func_start, 500)
print(f"img4_verify at {base+func_start:#010x}: {len(func)} instructions")
# Show all branches and calls
for inst in func:
    if inst.mnemonic in ('bl', 'b', 'cbz', 'cbnz', 'tbz', 'tbnz', 'b.eq', 'b.ne', 
                          'b.lo', 'b.hi', 'b.le', 'b.ge', 'b.lt', 'b.gt',
                          'msr', 'mrs', 'ldr', 'ret', 'cmp', 'tst'):
        if inst.mnemonic in ('bl', 'cbz', 'cbnz', 'tbz', 'tbnz', 'ret', 'cmp', 'tst') or \
           ('b.' in inst.mnemonic) or \
           (inst.mnemonic == 'b' and '#0x' in inst.op_str):
            print(f"  {inst.address:#010x}: {inst.mnemonic:8s} {inst.op_str}")
