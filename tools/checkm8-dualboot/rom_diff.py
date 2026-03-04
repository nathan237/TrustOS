#!/usr/bin/env python3
"""
ROM Binary Differ — T8020 A0 vs B1 SecureROM
=============================================
Compare two revisions of the same SecureROM to find:
1. Structural differences (size, sections, padding)
2. Patched functions (what Apple changed between steppings)
3. New code / removed code
4. Changed constants, strings, MMIO addresses
5. Security-relevant modifications (potential 1-day leads)

Strategy: Apple fixes bugs between steppings. Every change = something they
considered important enough to modify in MASK ROM. Those are the hot spots.
"""

import sys
import struct
import hashlib
from pathlib import Path
from capstone import *

ROM_BASE = 0x100000000

B1_PATH = Path("securerom/t8020_B1_securerom.bin")
A0_PATH = Path("securerom/t8020_A0_securerom.bin")

def load_rom(path):
    data = path.read_bytes()
    return data

def find_strings(data, min_len=4):
    """Extract ASCII strings from binary data."""
    strings = []
    current = b""
    start = 0
    for i, b in enumerate(data):
        if 0x20 <= b < 0x7f:
            if not current:
                start = i
            current += bytes([b])
        else:
            if len(current) >= min_len:
                strings.append((start, current.decode('ascii', errors='replace')))
            current = b""
    if len(current) >= min_len:
        strings.append((start, current.decode('ascii', errors='replace')))
    return strings

def find_code_region(data):
    """Find the actual code region (non-zero, non-padding)."""
    # Find first non-zero byte
    first_nonzero = 0
    for i in range(len(data)):
        if data[i] != 0:
            first_nonzero = i
            break
    
    # Find last non-zero byte
    last_nonzero = len(data) - 1
    while last_nonzero > 0 and data[last_nonzero] == 0:
        last_nonzero -= 1
    
    return first_nonzero, last_nonzero + 1

def compute_block_hashes(data, block_size=256):
    """Hash data in blocks to find matching/different regions."""
    blocks = []
    for i in range(0, len(data), block_size):
        block = data[i:i+block_size]
        h = hashlib.md5(block).hexdigest()[:8]
        blocks.append((i, h, block))
    return blocks

def find_functions(data, base_addr):
    """Find function prologues (STP X29, X30, [SP, #...]) in AArch64."""
    functions = []
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    
    # Look for common function prologues
    i = 0
    while i < len(data) - 4:
        word = struct.unpack_from('<I', data, i)[0]
        
        # STP x29, x30, [sp, #imm]! (pre-index)  or  STP x29, x30, [sp, #imm]
        # Also SUB sp, sp, #imm as prologue
        is_stp_fp_lr = (word & 0xFFE00FFF) == 0xA9800FE0 or \
                       (word & 0xFFE00FFF) == 0xA9000FE0 or \
                       (word & 0xFFC003FF) == 0xA98003E0
        
        # More general: STP with x29 and x30
        is_stp_29_30 = (word & 0x7FC07FFF) == 0x29807BFD or \
                       (word & 0x7FC07FFF) == 0x2D007BFD
        
        # Match STP X29, X30, [SP, ...] patterns more broadly
        if (word & 0xFFE003E0) == 0xA98003E0:  # STP x29,x30,[sp,#]!
            try:
                # Verify it's followed by reasonable code
                next_word = struct.unpack_from('<I', data, i+4)[0]
                if next_word != 0 and next_word != 0xFFFFFFFF:
                    functions.append(i)
            except:
                pass
        
        i += 4
    
    return functions

def disassemble_at(data, offset, base_addr, count=20):
    """Disassemble instructions at a given offset."""
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    lines = []
    code = data[offset:offset + count * 4]
    for insn in md.disasm(code, base_addr + offset):
        lines.append(f"  0x{insn.address:09X}: {insn.mnemonic:8s} {insn.op_str}")
        if len(lines) >= count:
            break
    return lines

def find_byte_diffs(data_a, data_b, min_common_size):
    """Find all byte-level differences between two buffers."""
    diffs = []
    run_start = None
    
    for i in range(min_common_size):
        if data_a[i] != data_b[i]:
            if run_start is None:
                run_start = i
        else:
            if run_start is not None:
                diffs.append((run_start, i, data_a[run_start:i], data_b[run_start:i]))
                run_start = None
    
    if run_start is not None:
        diffs.append((run_start, min_common_size, 
                      data_a[run_start:min_common_size], 
                      data_b[run_start:min_common_size]))
    
    return diffs

def classify_diff(offset, size, data_a, data_b, a0_data, b1_data):
    """Try to classify what kind of change a diff represents."""
    # Check if it's in a string region
    a_printable = all(0x20 <= b < 0x7f for b in data_a if b != 0)
    b_printable = all(0x20 <= b < 0x7f for b in data_b if b != 0)
    
    if a_printable and b_printable:
        return "STRING_CHANGE"
    
    # Check if it looks like code (4-byte aligned, instruction patterns)
    if offset % 4 == 0 and size % 4 == 0:
        return "CODE_CHANGE"
    
    # Check if it's an address/pointer change
    if size == 4 or size == 8:
        return "POINTER_OR_CONST"
    
    return "DATA_CHANGE"

def analyze_instruction_diffs(a0_data, b1_data, offset, size):
    """Disassemble and compare instructions at a diff location."""
    if offset % 4 != 0 or size < 4:
        return None, None
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    
    a0_insns = []
    for insn in md.disasm(a0_data[offset:offset+max(size, 20*4)], ROM_BASE + offset):
        a0_insns.append(f"0x{insn.address:09X}: {insn.mnemonic:8s} {insn.op_str}")
        if len(a0_insns) >= 20:
            break
    
    b1_insns = []
    for insn in md.disasm(b1_data[offset:offset+max(size, 20*4)], ROM_BASE + offset):
        b1_insns.append(f"0x{insn.address:09X}: {insn.mnemonic:8s} {insn.op_str}")
        if len(b1_insns) >= 20:
            break
    
    return a0_insns, b1_insns

def scan_mmio_refs(data, base_addr):
    """Find all MMIO address references (ADRP+ADD patterns)."""
    mmio_refs = {}
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    
    adrp_regs = {}  # reg -> page address
    
    for i in range(0, min(len(data), 0x20000), 4):
        try:
            code = data[i:i+4]
            if len(code) < 4:
                break
            for insn in md.disasm(code, base_addr + i):
                if insn.mnemonic == 'adrp':
                    # Extract the page address
                    op_str = insn.op_str
                    parts = op_str.split(',')
                    if len(parts) == 2:
                        try:
                            addr = int(parts[1].strip().replace('#', ''), 0)
                            reg = parts[0].strip()
                            adrp_regs[reg] = addr
                        except:
                            pass
                elif insn.mnemonic in ('add', 'ldr', 'str') and insn.op_str:
                    # Check if using an ADRP result
                    parts = insn.op_str.split(',')
                    if len(parts) >= 2:
                        base_reg = parts[1].strip().split(']')[0].strip('[').strip()
                        if base_reg in adrp_regs:
                            page = adrp_regs[base_reg]
                            # Try to extract offset
                            if '#' in insn.op_str:
                                try:
                                    off_str = insn.op_str.split('#')[-1].rstrip(']').strip()
                                    off = int(off_str, 0)
                                    full_addr = page + off
                                    if full_addr >= 0x200000000:  # MMIO range
                                        if full_addr not in mmio_refs:
                                            mmio_refs[full_addr] = []
                                        mmio_refs[full_addr].append(i)
                                except:
                                    pass
        except:
            continue
    
    return mmio_refs

def main():
    print("=" * 80)
    print("  T8020 SecureROM Binary Differ — A0 vs B1")
    print("  Finding what Apple patched between silicon steppings")
    print("=" * 80)
    
    a0_data = load_rom(A0_PATH)
    b1_data = load_rom(B1_PATH)
    
    print(f"\n[1] BASIC COMPARISON")
    print(f"    A0 size: {len(a0_data):,} bytes ({len(a0_data)//1024}KB)")
    print(f"    B1 size: {len(b1_data):,} bytes ({len(b1_data)//1024}KB)")
    print(f"    Size diff: {len(a0_data) - len(b1_data):,} bytes")
    print(f"    A0 SHA256: {hashlib.sha256(a0_data).hexdigest()}")
    print(f"    B1 SHA256: {hashlib.sha256(b1_data).hexdigest()}")
    
    # =========================================================================
    # SECTION 2: Code region analysis
    # =========================================================================
    print(f"\n[2] CODE REGION ANALYSIS")
    
    a0_start, a0_end = find_code_region(a0_data)
    b1_start, b1_end = find_code_region(b1_data)
    
    print(f"    A0: code from 0x{a0_start:06X} to 0x{a0_end:06X} ({a0_end-a0_start:,} bytes)")
    print(f"    B1: code from 0x{b1_start:06X} to 0x{b1_end:06X} ({b1_end-b1_start:,} bytes)")
    
    # Check if A0 has extra data beyond 512KB
    a0_extra = a0_data[len(b1_data):]
    a0_extra_nonzero = sum(1 for b in a0_extra if b != 0)
    print(f"    A0 extra region (0x{len(b1_data):06X}-0x{len(a0_data):06X}): {a0_extra_nonzero:,} non-zero bytes")
    
    if a0_extra_nonzero > 0:
        extra_start, extra_end = find_code_region(a0_extra)
        print(f"    A0 extra code: 0x{len(b1_data)+extra_start:06X}-0x{len(b1_data)+extra_end:06X}")
        extra_strings = find_strings(a0_extra)
        if extra_strings:
            print(f"    A0 extra strings ({len(extra_strings)}):")
            for off, s in extra_strings[:20]:
                print(f"      0x{len(b1_data)+off:06X}: \"{s}\"")
    
    # =========================================================================
    # SECTION 3: String comparison
    # =========================================================================
    print(f"\n[3] STRING COMPARISON")
    
    a0_strings = find_strings(a0_data, 6)
    b1_strings = find_strings(b1_data, 6)
    
    a0_str_set = set(s for _, s in a0_strings)
    b1_str_set = set(s for _, s in b1_strings)
    
    only_a0 = a0_str_set - b1_str_set
    only_b1 = b1_str_set - a0_str_set
    
    print(f"    A0 strings: {len(a0_strings)}")
    print(f"    B1 strings: {len(b1_strings)}")
    print(f"    Only in A0: {len(only_a0)}")
    print(f"    Only in B1: {len(only_b1)}")
    
    if only_a0:
        print(f"\n    --- Strings REMOVED in B1 (only in A0) ---")
        for s in sorted(only_a0):
            # Find offset in A0
            off = next((o for o, st in a0_strings if st == s), 0)
            print(f"      A0@0x{off:06X}: \"{s}\"")
    
    if only_b1:
        print(f"\n    --- Strings ADDED in B1 (only in B1) ---")
        for s in sorted(only_b1):
            off = next((o for o, st in b1_strings if st == s), 0)
            print(f"      B1@0x{off:06X}: \"{s}\"")
    
    # =========================================================================
    # SECTION 4: Version strings
    # =========================================================================
    print(f"\n[4] VERSION IDENTIFICATION")
    
    for label, data, strings in [("A0", a0_data, a0_strings), ("B1", b1_data, b1_strings)]:
        for off, s in strings:
            if 'iBoot' in s or 'ROMV' in s or 'BUILD' in s or 'securerom' in s or \
               'CPID' in s or 'Copyright' in s or 'Apple' in s:
                print(f"    {label}@0x{off:06X}: \"{s}\"")
    
    # =========================================================================
    # SECTION 5: Byte-level diff in common region
    # =========================================================================
    print(f"\n[5] BYTE-LEVEL DIFF (common region: first {len(b1_data)//1024}KB)")
    
    min_size = min(len(a0_data), len(b1_data))
    diffs = find_byte_diffs(a0_data, b1_data, min_size)
    
    total_diff_bytes = sum(end - start for start, end, _, _ in diffs)
    print(f"    Total diff regions: {len(diffs)}")
    print(f"    Total diff bytes: {total_diff_bytes:,}")
    print(f"    Identical bytes: {min_size - total_diff_bytes:,} ({(min_size-total_diff_bytes)/min_size*100:.1f}%)")
    
    # Categorize diffs by size
    small_diffs = [(s, e, a, b) for s, e, a, b in diffs if e-s <= 4]
    medium_diffs = [(s, e, a, b) for s, e, a, b in diffs if 4 < e-s <= 64]
    large_diffs = [(s, e, a, b) for s, e, a, b in diffs if e-s > 64]
    
    print(f"    Small (<=4B): {len(small_diffs)}")
    print(f"    Medium (5-64B): {len(medium_diffs)}")
    print(f"    Large (>64B): {len(large_diffs)}")
    
    # =========================================================================
    # SECTION 6: Detailed diff analysis
    # =========================================================================
    print(f"\n[6] DETAILED DIFF ANALYSIS")
    
    for idx, (start, end, a_bytes, b_bytes) in enumerate(diffs):
        size = end - start
        dtype = classify_diff(start, size, a_bytes, b_bytes, a0_data, b1_data)
        
        print(f"\n  === DIFF #{idx+1}: offset 0x{start:06X}-0x{end:06X} ({size} bytes) [{dtype}] ===")
        print(f"      ROM address: 0x{ROM_BASE+start:09X}-0x{ROM_BASE+end:09X}")
        
        # Show hex bytes (limited)
        show_bytes = min(size, 32)
        a_hex = ' '.join(f'{b:02X}' for b in a_bytes[:show_bytes])
        b_hex = ' '.join(f'{b:02X}' for b in b_bytes[:show_bytes])
        if size > show_bytes:
            a_hex += " ..."
            b_hex += " ..."
        print(f"      A0: {a_hex}")
        print(f"      B1: {b_hex}")
        
        # If it looks like code change, disassemble both
        if dtype == "CODE_CHANGE" and start % 4 == 0:
            print(f"\n      A0 disassembly:")
            a0_insns, b1_insns = analyze_instruction_diffs(a0_data, b1_data, start, size)
            if a0_insns:
                for line in a0_insns[:15]:
                    print(f"        {line}")
            print(f"\n      B1 disassembly:")
            if b1_insns:
                for line in b1_insns[:15]:
                    print(f"        {line}")
        
        # Check if A0 bytes near this diff contain interesting strings
        context_start = max(0, start - 64)
        context_end = min(min_size, end + 64)
        context_a = a0_data[context_start:context_end]
        context_b = b1_data[context_start:context_end]
        
        ctx_str_a = find_strings(context_a, 4)
        ctx_str_b = find_strings(context_b, 4)
        
        if ctx_str_a or ctx_str_b:
            nearby = set()
            for off, s in ctx_str_a:
                nearby.add(f"A0: \"{s}\"")
            for off, s in ctx_str_b:
                nearby.add(f"B1: \"{s}\"")
            if nearby:
                print(f"      Nearby strings: {', '.join(list(nearby)[:5])}")
    
    # =========================================================================
    # SECTION 7: Function-level comparison
    # =========================================================================
    print(f"\n[7] FUNCTION PROLOGUE COMPARISON")
    
    a0_funcs = find_functions(a0_data[:min_size], ROM_BASE)
    b1_funcs = find_functions(b1_data, ROM_BASE)
    
    a0_func_set = set(a0_funcs)
    b1_func_set = set(b1_funcs)
    
    only_a0_funcs = a0_func_set - b1_func_set
    only_b1_funcs = b1_func_set - a0_func_set
    common_funcs = a0_func_set & b1_func_set
    
    print(f"    A0 function prologues: {len(a0_funcs)}")
    print(f"    B1 function prologues: {len(b1_funcs)}")
    print(f"    Common: {len(common_funcs)}")
    print(f"    Only A0: {len(only_a0_funcs)}")
    print(f"    Only B1: {len(only_b1_funcs)}")
    
    if only_a0_funcs:
        print(f"\n    Functions REMOVED/MOVED in B1:")
        for off in sorted(only_a0_funcs)[:20]:
            print(f"      0x{ROM_BASE+off:09X} (file offset 0x{off:06X})")
    
    if only_b1_funcs:
        print(f"\n    Functions ADDED/MOVED in B1:")
        for off in sorted(only_b1_funcs)[:20]:
            print(f"      0x{ROM_BASE+off:09X} (file offset 0x{off:06X})")
    
    # =========================================================================
    # SECTION 8: Changed functions analysis
    # =========================================================================
    print(f"\n[8] FUNCTIONS CONTAINING CHANGES")
    
    # For each diff, find the function it belongs to
    all_b1_funcs_sorted = sorted(b1_funcs)
    
    changed_functions = set()
    for start, end, _, _ in diffs:
        # Find the function this diff is in
        func_off = None
        for f in reversed(all_b1_funcs_sorted):
            if f <= start:
                func_off = f
                break
        if func_off is not None:
            changed_functions.add(func_off)
    
    print(f"    Functions with diffs: {len(changed_functions)}")
    for off in sorted(changed_functions):
        # Count bytes changed in this function
        next_func = None
        for f in all_b1_funcs_sorted:
            if f > off:
                next_func = f
                break
        if next_func is None:
            next_func = off + 0x1000
        
        func_diffs = [(s, e) for s, e, _, _ in diffs if off <= s < next_func]
        total_changed = sum(e - s for s, e in func_diffs)
        
        print(f"    0x{ROM_BASE+off:09X}: {total_changed} bytes changed, {len(func_diffs)} diff regions")
    
    # =========================================================================
    # SECTION 9: MMIO comparison
    # =========================================================================
    print(f"\n[9] MMIO REFERENCE COMPARISON")
    print(f"    Scanning A0 MMIO refs...")
    a0_mmio = scan_mmio_refs(a0_data[:min_size], ROM_BASE)
    print(f"    Scanning B1 MMIO refs...")
    b1_mmio = scan_mmio_refs(b1_data, ROM_BASE)
    
    a0_mmio_set = set(a0_mmio.keys())
    b1_mmio_set = set(b1_mmio.keys())
    
    only_a0_mmio = a0_mmio_set - b1_mmio_set
    only_b1_mmio = b1_mmio_set - a0_mmio_set
    
    print(f"    A0 MMIO addresses: {len(a0_mmio_set)}")
    print(f"    B1 MMIO addresses: {len(b1_mmio_set)}")
    print(f"    Only in A0: {len(only_a0_mmio)}")
    print(f"    Only in B1: {len(only_b1_mmio)}")
    
    if only_a0_mmio:
        print(f"\n    MMIO addresses REMOVED in B1:")
        for addr in sorted(only_a0_mmio)[:20]:
            refs = a0_mmio[addr]
            print(f"      0x{addr:09X} (referenced from {len(refs)} locations)")
    
    if only_b1_mmio:
        print(f"\n    MMIO addresses ADDED in B1:")
        for addr in sorted(only_b1_mmio)[:20]:
            refs = b1_mmio[addr]
            print(f"      0x{addr:09X} (referenced from {len(refs)} locations)")
    
    # =========================================================================
    # SECTION 10: Security-relevant patterns
    # =========================================================================
    print(f"\n[10] SECURITY-RELEVANT PATTERN SEARCH IN DIFFS")
    
    # Look for CBZ/CBNZ/TBZ/TBNZ changes (conditional branches = security checks)
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    
    security_diffs = []
    for start, end, a_bytes, b_bytes in diffs:
        if start % 4 != 0:
            continue
        size = end - start
        
        for off in range(0, min(size, 256), 4):
            abs_off = start + off
            
            # Check if A0 or B1 has a conditional branch at this offset
            if off + 4 <= len(a_bytes):
                a_word = struct.unpack_from('<I', a_bytes, off)[0]
            else:
                a_word = struct.unpack_from('<I', a0_data, abs_off)[0] if abs_off + 4 <= len(a0_data) else 0
            
            if off + 4 <= len(b_bytes):
                b_word = struct.unpack_from('<I', b_bytes, off)[0]
            else:
                b_word = struct.unpack_from('<I', b1_data, abs_off)[0] if abs_off + 4 <= len(b1_data) else 0
            
            # CBZ/CBNZ: 0x34/0x35 (top byte)
            # TBZ/TBNZ: 0x36/0x37
            # B.cond: 0x54
            for word, label in [(a_word, "A0"), (b_word, "B1")]:
                top = (word >> 24) & 0xFF
                if top in (0x34, 0x35, 0x36, 0x37, 0x54, 0xB4, 0xB5):
                    # Disassemble
                    for insn in md.disasm(struct.pack('<I', word), ROM_BASE + abs_off):
                        security_diffs.append((abs_off, label, f"{insn.mnemonic} {insn.op_str}"))
    
    if security_diffs:
        print(f"    Found {len(security_diffs)} conditional branch changes:")
        for off, label, insn_str in security_diffs:
            print(f"      [{label}] 0x{ROM_BASE+off:09X}: {insn_str}")
    else:
        print(f"    No conditional branch changes found in diff regions")
    
    # =========================================================================
    # SECTION 11: DFU/USB related changes
    # =========================================================================
    print(f"\n[11] DFU/USB HANDLER CHANGES")
    
    # Look for diffs near known DFU-related offsets
    # Known from our RE: USB handling around 0x8000-0xC000, DFU state machine
    dfu_related = []
    for start, end, a_bytes, b_bytes in diffs:
        # Check if near USB/DFU strings
        for label, slist in [("A0", a0_strings), ("B1", b1_strings)]:
            for soff, s in slist:
                if any(kw in s.lower() for kw in ['usb', 'dfu', 'transfer', 'abort', 
                                                     'endpoint', 'setup', 'request',
                                                     'image', 'img4', 'verify', 'memz']):
                    if abs(soff - start) < 0x200:
                        dfu_related.append((start, end, s, end-start))
                        break
    
    if dfu_related:
        print(f"    Found {len(dfu_related)} DFU/USB-related changes:")
        for start, end, nearby_str, size in dfu_related:
            print(f"      0x{ROM_BASE+start:09X} ({size}B) near \"{nearby_str}\"")
    else:
        print(f"    No DFU/USB-related changes detected")
    
    # =========================================================================
    # SECTION 12: Summary and hot spots
    # =========================================================================
    print(f"\n{'='*80}")
    print(f"  SUMMARY — HOT SPOTS FOR FURTHER INVESTIGATION")
    print(f"{'='*80}")
    
    print(f"\n  Total changes: {len(diffs)} regions, {total_diff_bytes:,} bytes")
    print(f"  A0 extra data: {len(a0_data) - len(b1_data):,} bytes beyond B1 boundary")
    print(f"  Changed functions: {len(changed_functions)}")
    print(f"  MMIO changes: +{len(only_b1_mmio)}/-{len(only_a0_mmio)} addresses")
    print(f"  String changes: +{len(only_b1)}/-{len(only_a0)}")
    print(f"  Conditional branch changes: {len(security_diffs)}")
    
    # Rank the most interesting diffs
    print(f"\n  TOP INVESTIGATION TARGETS:")
    print(f"  (Security patches = potential latent bugs in A0 that MIGHT")
    print(f"   reveal overlooked issues in nearby B1 code)")
    
    ranked = []
    for start, end, a_bytes, b_bytes in diffs:
        score = 0
        size = end - start
        
        # Larger changes are more interesting
        score += min(size, 100)
        
        # Code changes are more interesting
        if start % 4 == 0:
            score += 50
        
        # Near USB/DFU code is most interesting
        for _, s in b1_strings:
            if any(kw in s.lower() for kw in ['usb', 'dfu', 'abort', 'verify', 'img4']):
                if abs(_ - start) < 0x500:
                    score += 200
                    break
        
        # Conditional branch changes are very interesting
        for off, label, insn in security_diffs:
            if start <= off < end:
                score += 300
        
        ranked.append((score, start, end, size))
    
    ranked.sort(reverse=True)
    for i, (score, start, end, size) in enumerate(ranked[:15]):
        print(f"  #{i+1}. [score={score:3d}] 0x{ROM_BASE+start:09X} ({size}B)")

if __name__ == "__main__":
    main()
