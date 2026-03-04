#!/usr/bin/env python3
"""
T8020 SecureROM Function-Level Matcher & Comparator.
Matches equivalent functions between A0 and B1 steppings using:
  - String cross-references
  - Call graph structure  
  - Function size and instruction patterns
Then compares matched DFU/USB handler and security-critical functions side-by-side.
"""

import struct
from collections import defaultdict
from capstone import *

ROM_BASE = 0x100000000

a0 = open("securerom/t8020_A0_securerom.bin", "rb").read()
b1 = open("securerom/t8020_B1_securerom.bin", "rb").read()

md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

CODE_END = 0x24B40  # Both ROMs have code up to here

def find_functions(data, code_end=CODE_END):
    """Find function boundaries using STP x29,x30 prologues and SUB sp patterns."""
    functions = []
    for i in range(0, min(len(data), code_end), 4):
        # STP x29, x30, [sp, #imm]! or [sp, #imm]
        if i + 3 < len(data) and data[i] == 0xFD and data[i+1] == 0x7B and data[i+3] == 0xA9:
            functions.append(ROM_BASE + i)
        # SUB sp, sp, #imm (FF xx xx D1)
        elif i + 3 < len(data) and data[i] == 0xFF and data[i+3] == 0xD1:
            # Verify it's SUB sp, sp by checking more bits
            word = struct.unpack_from('<I', data, i)[0]
            if (word & 0xFF0003FF) == 0xD10003FF:
                # Check if preceded by STP or if this looks like a function entry
                # Only count if not in the middle of a function  
                pass  # Skip SUB-only entries for now
    return sorted(set(functions))

def get_function_body(data, func_addr, all_funcs):
    """Get instructions of a function from its start to the next function or RET."""
    off = func_addr - ROM_BASE
    # Find end: either next function start or first RET
    idx = all_funcs.index(func_addr) if func_addr in all_funcs else -1
    if idx >= 0 and idx + 1 < len(all_funcs):
        max_end = all_funcs[idx + 1] - ROM_BASE
    else:
        max_end = min(len(data), off + 4096)
    
    insns = []
    for insn in md.disasm(data[off:max_end], func_addr):
        insns.append(insn)
        if insn.mnemonic == 'ret' and len(insns) > 2:
            break
        if len(insns) > 500:
            break
    return insns

def get_function_signature(data, func_addr, all_funcs, all_strings):
    """Create a signature for function matching."""
    insns = get_function_body(data, func_addr, all_funcs)
    
    sig = {
        'addr': func_addr,
        'size': len(insns),
        'calls': [],       # BL targets (relative to this func)
        'strings': [],     # String references
        'mnemonics': [],   # Instruction pattern
        'mmio': [],        # MMIO references
        'constants': [],   # Immediate constants
        'sysregs': [],     # System register accesses
    }
    
    for insn in insns:
        sig['mnemonics'].append(insn.mnemonic)
        
        if insn.mnemonic == 'bl':
            try:
                target = int(insn.op_str.lstrip('#'), 16)
                # Store as relative offset from function start
                sig['calls'].append(target)
            except:
                pass
        
        # Look for ADRP+ADD/LDR patterns for string references
        if insn.mnemonic in ('adr', 'adrp'):
            try:
                target = int(insn.op_str.split('#')[1].rstrip(')'), 16)
                target_off = target - ROM_BASE
                if 0 <= target_off < len(data):
                    # Check if this points to a string
                    for s_off, s_text in all_strings:
                        if abs(target_off - s_off) < 0x1000:
                            sig['strings'].append(s_text)
                            break
                    # Check for MMIO
                    if target >= 0x200000000:
                        sig['mmio'].append(target)
            except:
                pass
        
        # System register accesses
        if insn.mnemonic in ('mrs', 'msr'):
            sig['sysregs'].append(insn.op_str)
        
        # Notable constants
        if insn.mnemonic in ('mov', 'movz', 'movk') and '#' in insn.op_str:
            try:
                val = int(insn.op_str.split('#')[1].rstrip(')'), 16) if '0x' in insn.op_str else int(insn.op_str.split('#')[1])
                if val > 0xFF:
                    sig['constants'].append(val)
            except:
                pass
    
    return sig

def find_strings(data, min_len=6):
    """Find ASCII strings in ROM."""
    strings = []
    i = 0
    while i < len(data):
        if 0x20 <= data[i] < 0x7F:
            end = i
            while end < len(data) and 0x20 <= data[end] < 0x7F:
                end += 1
            if end - i >= min_len:
                s = data[i:end].decode('ascii', errors='replace')
                strings.append((i, s))
            i = end
        else:
            i += 1
    return strings

def match_score(sig_a, sig_b):
    """Score how well two function signatures match. Higher = better match."""
    score = 0
    
    # String match (strongest signal)
    common_strings = set(sig_a['strings']) & set(sig_b['strings'])
    score += len(common_strings) * 100
    
    # System register match
    common_sysregs = set(sig_a['sysregs']) & set(sig_b['sysregs'])
    score += len(common_sysregs) * 50
    
    # Size similarity
    size_diff = abs(sig_a['size'] - sig_b['size'])
    if size_diff == 0:
        score += 30
    elif size_diff < 5:
        score += 20
    elif size_diff < 20:
        score += 10
    
    # Call count similarity
    if len(sig_a['calls']) == len(sig_b['calls']):
        score += 15
    elif abs(len(sig_a['calls']) - len(sig_b['calls'])) < 3:
        score += 5
    
    # Mnemonic pattern similarity (first 10 instructions)
    a_mn = sig_a['mnemonics'][:10]
    b_mn = sig_b['mnemonics'][:10]
    common_mn = sum(1 for a, b in zip(a_mn, b_mn) if a == b)
    score += common_mn * 3
    
    # Constants match
    common_consts = set(sig_a['constants']) & set(sig_b['constants'])
    score += len(common_consts) * 20
    
    # MMIO match (after normalizing base)
    a_mmio_low = set(m & 0xFFFFF for m in sig_a['mmio'])
    b_mmio_low = set(m & 0xFFFFF for m in sig_b['mmio'])
    common_mmio = a_mmio_low & b_mmio_low
    score += len(common_mmio) * 40
    
    return score

print("=" * 120)
print("  T8020 SecureROM FUNCTION-LEVEL COMPARISON — A0 vs B1")
print("=" * 120)

# Step 1: Find all strings
print("\n[1] Finding strings...")
a0_strings = find_strings(a0)
b1_strings = find_strings(b1)
print(f"  A0: {len(a0_strings)} strings, B1: {len(b1_strings)} strings")

# Step 2: Find all functions
print("\n[2] Finding function boundaries...")
a0_funcs = find_functions(a0)
b1_funcs = find_functions(b1)
print(f"  A0: {len(a0_funcs)} functions, B1: {len(b1_funcs)} functions")

# Step 3: Generate signatures for all functions
print("\n[3] Generating function signatures...")
a0_sigs = {}
for f in a0_funcs:
    a0_sigs[f] = get_function_signature(a0, f, a0_funcs, a0_strings)

b1_sigs = {}
for f in b1_funcs:
    b1_sigs[f] = get_function_signature(b1, f, b1_funcs, b1_strings)

# Step 4: Match functions
print("\n[4] Matching functions between A0 and B1...")
matches = []  # (a0_addr, b1_addr, score)
used_b1 = set()

for a0_addr, a0_sig in sorted(a0_sigs.items()):
    best_score = 0
    best_b1 = None
    for b1_addr, b1_sig in b1_sigs.items():
        if b1_addr in used_b1:
            continue
        score = match_score(a0_sig, b1_sig)
        if score > best_score:
            best_score = score
            best_b1 = b1_addr
    
    if best_score >= 50 and best_b1:
        matches.append((a0_addr, best_b1, best_score))
        used_b1.add(best_b1)

matches.sort(key=lambda x: -x[2])
print(f"  Matched {len(matches)} function pairs")

# Show top matches
print(f"\n  Top 30 matched function pairs:")
for a0_addr, b1_addr, score in matches[:30]:
    a0_sig = a0_sigs[a0_addr]
    b1_sig = b1_sigs[b1_addr]
    offset_shift = (b1_addr - a0_addr)
    strs = ", ".join(a0_sig['strings'][:2]) if a0_sig['strings'] else "-"
    print(f"    A0:0x{a0_addr:X} → B1:0x{b1_addr:X} (shift={offset_shift:+d}, score={score}, "
          f"size={a0_sig['size']}/{b1_sig['size']}, strings=[{strs}])")

# Step 5: Find the DFU handler function in both ROMs
print("\n\n" + "=" * 120)
print("  SECTION A: DFU/USB HANDLER FUNCTION COMPARISON")
print("=" * 120)

# Find functions that reference "Apple Mobile Device (DFU Mode)" or "DFU"
dfu_matches_a0 = []
dfu_matches_b1 = []

for addr, sig in a0_sigs.items():
    for s in sig['strings']:
        if 'DFU' in s or 'USB' in s or 'Apple Mobile' in s or 'CPID' in s:
            dfu_matches_a0.append((addr, sig, s))
            break

for addr, sig in b1_sigs.items():
    for s in sig['strings']:
        if 'DFU' in s or 'USB' in s or 'Apple Mobile' in s or 'CPID' in s:
            dfu_matches_b1.append((addr, sig, s))
            break

print(f"\n  A0 DFU-related functions: {len(dfu_matches_a0)}")
for addr, sig, s in dfu_matches_a0:
    print(f"    0x{addr:X}: size={sig['size']}, calls={len(sig['calls'])}, str='{s[:50]}'")

print(f"\n  B1 DFU-related functions: {len(dfu_matches_b1)}")
for addr, sig, s in dfu_matches_b1:
    print(f"    0x{addr:X}: size={sig['size']}, calls={len(sig['calls'])}, str='{s[:50]}'")

# Step 6: Find the img4_verify caller — THE critical function
print("\n\n" + "=" * 120)
print("  SECTION B: IMG4_VERIFY CALLER — THE BOOT GATE FUNCTION")
print("=" * 120)

# In B1, img4_verify is at 0x10000A704, called from 0x10001BC0 (bl #0x10000a704)
# Find what function in A0 calls the equivalent of img4_verify
# img4_verify in A0 might be at a different address

# Find functions that reference "Apple Secure Boot" or the root CA
boot_funcs_a0 = []
boot_funcs_b1 = []

for addr, sig in a0_sigs.items():
    for s in sig['strings']:
        if 'Secure Boot' in s or 'Root CA' in s:
            boot_funcs_a0.append((addr, sig, s))
            break

for addr, sig in b1_sigs.items():
    for s in sig['strings']:
        if 'Secure Boot' in s or 'Root CA' in s:
            boot_funcs_b1.append((addr, sig, s))
            break

print(f"\n  A0 functions referencing Secure Boot/Root CA: {len(boot_funcs_a0)}")
for addr, sig, s in boot_funcs_a0:
    calls = [f"0x{c:X}" for c in sig['calls'][:5]]
    print(f"    0x{addr:X}: size={sig['size']}, calls=[{', '.join(calls)}], str='{s[:50]}'")

print(f"\n  B1 functions referencing Secure Boot/Root CA: {len(boot_funcs_b1)}")
for addr, sig, s in boot_funcs_b1:
    calls = [f"0x{c:X}" for c in sig['calls'][:5]]
    print(f"    0x{addr:X}: size={sig['size']}, calls=[{', '.join(calls)}], str='{s[:50]}'")

# Step 7: Find the boot flow / img4_verify function in both ROMs
print("\n\n" + "=" * 120)
print("  SECTION C: FINDING img4_verify IN BOTH ROMS")
print("=" * 120)

# In B1: img4_verify is known to be at 0x10000A704
# Let's get its signature
img4_b1_addr = 0x10000A704
img4_b1_sig = get_function_signature(b1, img4_b1_addr, b1_funcs, b1_strings)
print(f"\n  B1 img4_verify @ 0x{img4_b1_addr:X}:")
print(f"    Size: {img4_b1_sig['size']} instructions")
print(f"    Calls: {[f'0x{c:X}' for c in img4_b1_sig['calls']]}")
print(f"    Strings: {img4_b1_sig['strings']}")
print(f"    Constants: {[hex(c) for c in img4_b1_sig['constants']]}")
print(f"    SysRegs: {img4_b1_sig['sysregs']}")

# Find matching function in A0
print(f"\n  Searching for img4_verify equivalent in A0...")
best_a0_match = None
best_a0_score = 0

for a0_addr, a0_sig in a0_sigs.items():
    score = match_score(a0_sig, img4_b1_sig)
    if score > best_a0_score:
        best_a0_score = score
        best_a0_match = a0_addr

if best_a0_match:
    a0_match_sig = a0_sigs[best_a0_match]
    print(f"  Best match: A0 @ 0x{best_a0_match:X} (score={best_a0_score})")
    print(f"    Size: {a0_match_sig['size']} instructions")
    print(f"    Calls: {[f'0x{c:X}' for c in a0_match_sig['calls']]}")
    print(f"    Strings: {a0_match_sig['strings']}")
    print(f"    Constants: {[hex(c) for c in a0_match_sig['constants']]}")

# Step 8: Find the ACTUAL boot flow function (the one that calls img4_verify)
print("\n\n" + "=" * 120)
print("  SECTION D: THE BOOT FLOW FUNCTION (calls img4_verify)")
print("=" * 120)

# In B1, the boot flow function contains:
#   bl #0x10000a704  (img4_verify)
#   mov x8, x0
#   cbz w8, #0x100001c5c
# Find which function in B1 calls 0x10000A704

b1_boot_func = None
for addr, sig in b1_sigs.items():
    if 0x10000A704 in sig['calls']:
        b1_boot_func = addr
        print(f"\n  B1 boot function: 0x{addr:X} (calls img4_verify)")
        print(f"    Size: {sig['size']} instructions")
        print(f"    All calls: {[f'0x{c:X}' for c in sig['calls']]}")
        print(f"    Strings: {sig['strings']}")
        break

# Find the equivalent in A0
# The A0 img4_verify is likely at ~0x10000A3XX or so
# Find functions that call many of the same sub-functions
if b1_boot_func:
    b1_boot_sig = b1_sigs[b1_boot_func]
    print(f"\n  Looking for A0 equivalent of boot function 0x{b1_boot_func:X}...")
    
    # Try matching by the unique combination of calls and strings
    candidates = []
    for a0_addr, a0_sig in a0_sigs.items():
        score = match_score(a0_sig, b1_boot_sig)
        if score > 30:
            candidates.append((a0_addr, score, a0_sig))
    
    candidates.sort(key=lambda x: -x[1])
    print(f"  Top 5 candidates:")
    for addr, score, sig in candidates[:5]:
        calls = [f"0x{c:X}" for c in sig['calls'][:5]]
        print(f"    0x{addr:X}: score={score}, size={sig['size']}, calls=[{', '.join(calls)}], "
              f"strings={sig['strings'][:2]}")

# Step 9: Side-by-side the matched functions
print("\n\n" + "=" * 120)
print("  SECTION E: SIDE-BY-SIDE COMPARISON OF MATCHED CRITICAL FUNCTIONS")
print("=" * 120)

def compare_matched_functions(a0_addr, b1_addr, label, a0_data=a0, b1_data=b1):
    """Compare two matched functions side by side."""
    print(f"\n{'#' * 120}")
    print(f"# {label}")
    print(f"# A0: 0x{a0_addr:X}  |  B1: 0x{b1_addr:X}  |  Shift: {b1_addr-a0_addr:+d}")
    print(f"{'#' * 120}")
    
    a0_insns = get_function_body(a0_data, a0_addr, a0_funcs)
    b1_insns = get_function_body(b1_data, b1_addr, b1_funcs)
    
    max_len = max(len(a0_insns), len(b1_insns))
    print(f"  {'A0 (iBoot-3865.0.0.1.23)':<55} | {'B1 (iBoot-3865.0.0.4.7)':<55}")
    print(f"  {'-'*55}-+-{'-'*55}")
    
    for i in range(max_len):
        if i < len(a0_insns):
            a = a0_insns[i]
            a_str = f"0x{a.address:X}: {a.mnemonic} {a.op_str}"
        else:
            a_str = ""
        if i < len(b1_insns):
            b = b1_insns[i]
            b_str = f"0x{b.address:X}: {b.mnemonic} {b.op_str}"
        else:
            b_str = ""
        
        marker = " "
        if i < len(a0_insns) and i < len(b1_insns):
            if a0_insns[i].mnemonic != b1_insns[i].mnemonic:
                marker = "!"
            elif a0_insns[i].op_str != b1_insns[i].op_str:
                marker = "~"
        elif i >= len(a0_insns) or i >= len(b1_insns):
            marker = "+"
        
        print(f"{marker} {a_str:<55} | {b_str:<55}")
    
    print(f"\n  A0 size: {len(a0_insns)} insns, B1 size: {len(b1_insns)} insns, diff: {len(b1_insns)-len(a0_insns):+d}")

# Compare top matched DFU functions
for a0_addr, sig, s in dfu_matches_a0[:3]:
    # Find its B1 match
    for m_a0, m_b1, score in matches:
        if m_a0 == a0_addr:
            compare_matched_functions(a0_addr, m_b1, f"DFU Function: {s[:60]} (match_score={score})")
            break

# Step 10: Find ALL callers of img4_verify and compare
print("\n\n" + "=" * 120)
print("  SECTION F: ALL CALLERS OF img4_verify")
print("=" * 120)

# Find EVERY function in B1 that calls img4_verify
print("\n  B1 — All callers of img4_verify (0x10000A704):")
b1_img4_callers = []
for addr, sig in b1_sigs.items():
    if 0x10000A704 in sig['calls']:
        b1_img4_callers.append(addr)
        print(f"    0x{addr:X}: size={sig['size']}, strings={sig['strings'][:2]}")

# Find A0 equivalent img4_verify address from our match
if best_a0_match:
    print(f"\n  A0 — All callers of img4_verify equivalent (0x{best_a0_match:X}):")
    a0_img4_callers = []
    for addr, sig in a0_sigs.items():
        if best_a0_match in sig['calls']:
            a0_img4_callers.append(addr)
            print(f"    0x{addr:X}: size={sig['size']}, strings={sig['strings'][:2]}")

# Step 11: Unmatched functions (NEW in B1)
print("\n\n" + "=" * 120)
print("  SECTION G: FUNCTIONS UNIQUE TO B1 (SECURITY ADDITIONS)")
print("=" * 120)

matched_b1_addrs = set(m[1] for m in matches)
unmatched_b1 = [addr for addr in b1_funcs if addr not in matched_b1_addrs]
print(f"\n  Unmatched B1 functions: {len(unmatched_b1)}")
for addr in unmatched_b1[:30]:
    sig = b1_sigs.get(addr)
    if sig:
        calls = len(sig['calls'])
        strs = sig['strings'][:2] if sig['strings'] else []
        size = sig['size']
        consts = [hex(c) for c in sig['constants'][:3]]
        print(f"    0x{addr:X}: size={size}, calls={calls}, strings={strs}, consts={consts}")

# Step 12: CRITICAL — Check for new NULL/bounds checks added in B1
print("\n\n" + "=" * 120)
print("  SECTION H: NEW SAFETY CHECKS IN B1 (NULL checks, bounds checks)")
print("=" * 120)

for a0_addr, b1_addr, score in matches[:50]:
    if score < 50:
        continue
    
    a0_insns = get_function_body(a0, a0_addr, a0_funcs)
    b1_insns = get_function_body(b1, b1_addr, b1_funcs)
    
    # Count safety-related instructions
    a0_checks = sum(1 for i in a0_insns if i.mnemonic in ('cbz', 'cbnz', 'cmp', 'tst', 'tbz', 'tbnz'))
    b1_checks = sum(1 for i in b1_insns if i.mnemonic in ('cbz', 'cbnz', 'cmp', 'tst', 'tbz', 'tbnz'))
    
    if b1_checks > a0_checks + 2:  # B1 has significantly more checks
        print(f"\n  0x{a0_addr:X} → 0x{b1_addr:X}: A0 has {a0_checks} checks, B1 has {b1_checks} checks (+{b1_checks-a0_checks})")
        # Show the NEW checks in B1
        a0_check_ops = set()
        for i in a0_insns:
            if i.mnemonic in ('cbz', 'cbnz', 'cmp', 'tst', 'tbz', 'tbnz'):
                a0_check_ops.add(f"{i.mnemonic} {i.op_str}")
        
        for i in b1_insns:
            if i.mnemonic in ('cbz', 'cbnz', 'cmp', 'tst', 'tbz', 'tbnz'):
                op = f"{i.mnemonic} {i.op_str}"
                # This is approximate — flag NEW checks not in A0
                # (register names might differ, so this is fuzzy)
                print(f"    B1 0x{i.address:X}: {i.mnemonic} {i.op_str}")

# Step 13: Find the DFU USB request handler specifically
print("\n\n" + "=" * 120)
print("  SECTION I: DFU USB REQUEST PATHS — Following the Call Graph")
print("=" * 120)

# In B1, the DFU handler setup function likely references the "Apple Mobile Device" string
# and sets up USB descriptors. Let's trace from there.

# Build call graph for B1
print("\n  B1 Call Graph from DFU-related functions:")
b1_callgraph = defaultdict(list)  # caller -> [callees]
for addr, sig in b1_sigs.items():
    for target in sig['calls']:
        b1_callgraph[addr].append(target)

# DFS from DFU functions
dfu_related = set()
for addr, sig, s in dfu_matches_b1:
    stack = [addr]
    visited = set()
    while stack:
        current = stack.pop()
        if current in visited:
            continue
        visited.add(current)
        dfu_related.add(current)
        for callee in b1_callgraph.get(current, []):
            if callee not in visited and callee in b1_sigs:
                stack.append(callee)
    
    print(f"\n  From 0x{addr:X} ('{s[:40]}'): reaches {len(visited)} functions")
    for v in sorted(visited)[:20]:
        v_sig = b1_sigs.get(v)
        if v_sig:
            print(f"    → 0x{v:X}: size={v_sig['size']}, strings={v_sig['strings'][:1]}")

print("\n\n" + "=" * 120)
print("  FUNCTION-LEVEL ANALYSIS COMPLETE")
print("=" * 120)
