#!/usr/bin/env python3
"""
ROM Vulnerability Hunter v2 — Focused second pass
 
Key issues from v1:
1. DFU handler search used only ADRP, missed ADR references
2. Call graph was broken (only found 1 callee)
3. Cross-references to new functions found 0 (search was wrong)
4. State machine detection too narrow

Focus areas:
A. Fix call graph to properly map ALL function calls
B. Trace the USB/DFU handler from known strings
C. Map what the new B1 functions do in context
D. Deep analysis of img4_verify expansion for new attack surface
E. Find the USB IO buffer allocation/free lifecycle
"""

import struct
from capstone import *
from collections import defaultdict

ROM_BASE = 0x100000000

def load_rom(path):
    with open(path, 'rb') as f:
        return f.read()

def get_disasm(data, code_end=0x24B40):
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    md.skipdata = True  # Skip over data/invalid bytes instead of stopping
    return list(md.disasm(data[:code_end], ROM_BASE))

def find_function_start(data, addr):
    """Walk backwards from addr to find function prologue"""
    off = addr - ROM_BASE
    for back in range(off, max(off - 0x1000, 0), -4):
        w = struct.unpack('<I', data[back:back+4])[0]
        # STP x29, x30, [sp, #imm] patterns
        if (w & 0xFFC07FFF) == 0xA9007BFD:  # stp x29, x30, [sp, #imm]
            return ROM_BASE + back
        if (w & 0xFFE0FFFF) == 0xA9BF7BFD:  # stp x29, x30, [sp, #-imm]!
            return ROM_BASE + back
        # Some functions start with sub sp, sp (pacibsp)
    return None

# =====================================================================================================================
# A. COMPLETE CALL GRAPH
# =====================================================================================================================
def build_call_graph(insns):
    """Build complete call graph from all BL instructions"""
    callers_of = defaultdict(list)  # callee -> [caller_addr]
    calls_from = defaultdict(list)  # caller_func -> [callee_addr]
    
    for insn in insns:
        if insn.mnemonic == 'bl':
            try:
                target = int(insn.op_str.replace('#', ''), 16)
                callers_of[target].append(insn.address)
            except:
                pass
    
    return callers_of

def print_call_graph(callers_of, data, insns):
    print(f"\n{'='*120}")
    print(f"  COMPLETE CALL GRAPH — TOP 30 MOST-CALLED FUNCTIONS")
    print(f"{'='*120}\n")
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    
    sorted_callees = sorted(callers_of.items(), key=lambda x: len(x[1]), reverse=True)[:30]
    for callee, callers in sorted_callees:
        off = callee - ROM_BASE
        if 0 <= off < len(data) - 40:
            code = data[off:off+40]
            func_insns = list(md.disasm(code, callee))[:5]
            sig = ' | '.join([f"{i.mnemonic} {i.op_str}" for i in func_insns])
            
            # Check if this function has a string reference (helps identification)
            has_adr = False
            for fi in func_insns:
                if fi.mnemonic in ('adr', 'adrp'):
                    has_adr = True
            
            print(f"  0x{callee:X} called {len(callers):3d}× : {sig}")
        else:
            print(f"  0x{callee:X} called {len(callers):3d}× : (outside code range)")
    
    return sorted_callees


# =====================================================================================================================
# B. DFU HANDLER — Trace from boot function
# =====================================================================================================================
def trace_dfu_handler(data, insns, callers_of):
    print(f"\n{'='*120}")
    print(f"  DFU HANDLER TRACE — Following the boot flow")
    print(f"{'='*120}\n")
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    
    # We know the boot function is at 0x1000017A4
    # It calls bl #0x100001CD8 to load the image
    # That function likely implements the DFU image reception
    
    # Map ALL functions called by the boot function
    boot_start = 0x17A4
    boot_code = data[boot_start:boot_start + 0x600]
    boot_insns = list(md.disasm(boot_code, ROM_BASE + boot_start))
    
    print(f"  Boot function (0x1000017A4) calls:")
    boot_calls = []
    for insn in boot_insns:
        if insn.mnemonic == 'bl':
            try:
                target = int(insn.op_str.replace('#', ''), 16)
                boot_calls.append((insn.address, target))
                num_callers = len(callers_of.get(target, []))
                print(f"    0x{insn.address:X}: bl 0x{target:X} (called from {num_callers} places total)")
            except:
                pass
    
    # The image loader function at 0x100001CD8 (B1)
    loader_func = 0x1CD8
    print(f"\n  === Image Loader Function @ 0x{ROM_BASE + loader_func:X} ===")
    loader_code = data[loader_func:loader_func + 0x400]
    loader_insns = list(md.disasm(loader_code, ROM_BASE + loader_func))
    
    # Print until RET
    loader_calls = []
    for insn in loader_insns:
        print(f"    0x{insn.address:X}: {insn.mnemonic:8s} {insn.op_str}")
        if insn.mnemonic == 'bl':
            try:
                target = int(insn.op_str.replace('#', ''), 16)
                loader_calls.append((insn.address, target))
            except:
                pass
        if insn.mnemonic == 'ret':
            break
    
    # Follow into sub-functions of the image loader
    print(f"\n  Image loader calls:")
    for addr, target in loader_calls:
        off = target - ROM_BASE
        if 0 <= off < len(data) - 40:
            tc = data[off:off+60]
            ti = list(md.disasm(tc, target))[:8]
            sig = ' ; '.join([f"{i.mnemonic} {i.op_str}" for i in ti])
            print(f"    0x{addr:X} → 0x{target:X}: {sig}")
    
    # Look for USB/DFU-specific patterns in the deeper call chain
    # The DFU handler typically calls:
    # 1. usb_init() - initialize USB controller
    # 2. usb_start() - start USB stack
    # 3. dfu_init() - initialize DFU state
    # 4. A polling/event loop waiting for USB requests
    
    # Find functions that reference USB-related MMIO addresses
    # T8020 USB OTG controller is typically at 0x39000000 range
    print(f"\n  Searching for USB controller MMIO patterns...")
    
    usb_mmio_funcs = set()
    for insn in insns:
        if insn.mnemonic == 'movk':
            ops = insn.op_str
            # USB OTG registers typically at high addresses
            # Common Apple USB base: varies by SoC
            # Look for 0x39xxxxx or 0x38xxxxxx patterns in movk with lsl #16 or lsl #32
            if 'lsl #16' in ops:
                try:
                    val_str = ops.split('#')[1].split(',')[0]
                    val = int(val_str, 16)
                    if val in range(0x3800, 0x3A00):
                        func = find_function_start(data, insn.address)
                        usb_mmio_funcs.add((insn.address, func, f"{insn.mnemonic} {insn.op_str}"))
                except:
                    pass
    
    if usb_mmio_funcs:
        print(f"  Found {len(usb_mmio_funcs)} USB MMIO references:")
        for addr, func, insn_str in sorted(usb_mmio_funcs):
            print(f"    0x{addr:X} in func 0x{func:X}: {insn_str}")
    else:
        # Try broader search: any large MMIO addresses
        print(f"  No USB MMIO at 0x38xx-0x39xx. Searching broader MMIO patterns...")
        mmio_ranges = defaultdict(int)
        for insn in insns:
            if insn.mnemonic == 'movk' and 'lsl #32' in insn.op_str:
                try:
                    val_str = insn.op_str.split('#')[1].split(',')[0]
                    val = int(val_str, 16)
                    mmio_ranges[val] += 1
                except:
                    pass
        
        print(f"  MMIO base ranges (movk with lsl #32):")
        for val, count in sorted(mmio_ranges.items(), key=lambda x: -x[1])[:20]:
            print(f"    0x{val:X}xxxxxxxx — {count} references")
    
    return boot_calls, loader_calls


# =====================================================================================================================
# C. NEW B1 FUNCTION CALLERS (fixed search)
# =====================================================================================================================
def find_new_func_callers(callers_of):
    print(f"\n{'='*120}")
    print(f"  NEW B1 FUNCTION CROSS-REFERENCES (fixed)")
    print(f"{'='*120}\n")
    
    new_funcs = [0x100006CDC, 0x100006D64, 0x100006D80, 0x10000955C, 0x100009684, 0x10001AC78]
    
    for func in new_funcs:
        callers = callers_of.get(func, [])
        print(f"  0x{func:X} called from {len(callers)} locations:")
        for caller in callers:
            func_start = None
            print(f"    - 0x{caller:X}")
    
    # Also check: are any of these functions targets of indirect calls (BLR)?
    # Or are they tail-called via B (branch, no link)?
    print(f"\n  Note: Also checking for B (tail call) references...")
    

# =====================================================================================================================
# D. img4_verify EXPANDED — New verification code analysis
# =====================================================================================================================
def analyze_img4_expansion(data_b1, insns, callers_of):
    print(f"\n{'='*120}")
    print(f"  IMG4_VERIFY EXPANSION — What Apple ADDED (new code after main verify)")
    print(f"{'='*120}\n")
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    
    # B1 img4_verify starts at 0x10000A704 and has ~150 instructions
    # The main verify ends around 0x10000A864 (ret)
    # After that, there are new helper functions that are part of the same "module"
    
    # New B1 calls NOT in A0:
    new_calls = {
        0x10000f1ec: "ALLOC? (called with w0=0x18, x1=0)",
        0x100011ed8: "CRYPTO_CHECK_1?",
        0x100011ed0: "CRYPTO_CHECK_2?", 
        0x100011f04: "CRYPTO_INIT?",
    }
    
    for target, desc in new_calls.items():
        off = target - ROM_BASE
        if 0 <= off < len(data_b1) - 200:
            code = data_b1[off:off+400]
            func_insns = list(md.disasm(code, target))
            
            print(f"\n  === Function 0x{target:X} ({desc}) ===")
            callers = callers_of.get(target, [])
            print(f"  Called from {len(callers)} locations: {[f'0x{c:X}' for c in callers]}")
            
            count = 0
            for insn in func_insns:
                print(f"    0x{insn.address:X}: {insn.mnemonic:8s} {insn.op_str}")
                count += 1
                if insn.mnemonic == 'ret' and count > 3:
                    break
                if count > 80:
                    print(f"    ... (truncated)")
                    break
            
            # Count internal BL calls
            calls = [i for i in func_insns[:80] if i.mnemonic == 'bl']
            print(f"  Internal calls: {len(calls)}")
            for c in calls:
                print(f"    → bl {c.op_str}")


# =====================================================================================================================
# E. USB IO BUFFER LIFECYCLE
# The checkm8 vulnerability was in how USB IO buffers were allocated/freed
# Find the buffer management around DFU transfers
# =====================================================================================================================
def analyze_usb_io_buffers(data, insns, callers_of):
    print(f"\n{'='*120}")
    print(f"  USB IO BUFFER LIFECYCLE ANALYSIS")
    print(f"  Looking for allocation → use → free patterns in DFU context")
    print(f"{'='*120}\n")
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    
    # Find all allocation patterns: mov w0, #<size>; bl <malloc>
    # or: mov x0, #<size>; mov x1, #0; bl <alloc>
    alloc_patterns = []
    
    for i in range(len(insns) - 3):
        if insns[i].mnemonic == 'mov' and insns[i].op_str.startswith(('w0,', 'x0,')) and '#' in insns[i].op_str:
            # Next 1-3 instructions should be bl
            for j in range(i+1, min(i+4, len(insns))):
                if insns[j].mnemonic == 'bl':
                    try:
                        target = int(insns[j].op_str.replace('#', ''), 16)
                        # Extract size
                        size_str = insns[i].op_str.split('#')[1].split(',')[0]
                        size = int(size_str, 16) if '0x' in size_str else int(size_str)
                        
                        # Check if return value is stored (allocation)
                        if j+1 < len(insns) and insns[j+1].mnemonic == 'mov' and insns[j+1].op_str.startswith('x') and ', x0' in insns[j+1].op_str:
                            alloc_patterns.append({
                                'addr': insns[i].address,
                                'size': size,
                                'target': target,
                                'store_reg': insns[j+1].op_str.split(',')[0].strip(),
                                'mov_insn': f"{insns[i].mnemonic} {insns[i].op_str}",
                                'bl_insn': f"bl {insns[j].op_str}",
                            })
                    except:
                        pass
                    break
    
    # Group by allocation target (identifies malloc-like functions)
    alloc_targets = defaultdict(list)
    for a in alloc_patterns:
        alloc_targets[a['target']].append(a)
    
    print(f"  Allocation patterns (mov w0/x0, #size; bl <func>; mov xN, x0):")
    for target, allocs in sorted(alloc_targets.items(), key=lambda x: -len(x[1])):
        if len(allocs) >= 2:
            sizes = [a['size'] for a in allocs]
            print(f"\n    Target 0x{target:X} ({len(allocs)} allocations, sizes: {[hex(s) for s in sizes]}):")
            for a in allocs[:10]:
                print(f"      0x{a['addr']:X}: {a['mov_insn']} → {a['bl_insn']} → {a['store_reg']}")
    
    # Find the likely free function: called frequently, takes one pointer arg, no return
    # Pattern: mov x0, xN; bl <free>; (no use of x0 after)
    print(f"\n  Looking for free() candidates...")
    free_candidates = defaultdict(int)
    
    for i in range(len(insns) - 2):
        if insns[i].mnemonic == 'bl':
            try:
                target = int(insns[i].op_str.replace('#', ''), 16)
            except:
                continue
            
            # Check if prev instruction is mov x0, xN
            if i > 0 and insns[i-1].mnemonic == 'mov' and insns[i-1].op_str.startswith('x0, x'):
                # Check if x0 is NOT used after (free doesn't return useful value)
                if i+1 < len(insns):
                    next_insn = insns[i+1]
                    if next_insn.mnemonic != 'mov' or 'x0' not in next_insn.op_str.split(',')[1:]:
                        free_candidates[target] += 1
    
    print(f"  Free() candidates (mov x0, xN → bl <func> → x0 not saved):")
    for target, count in sorted(free_candidates.items(), key=lambda x: -x[1])[:10]:
        total_calls = len(callers_of.get(target, []))
        print(f"    0x{target:X}: {count} free-like patterns (total calls: {total_calls})")


# =====================================================================================================================
# F. COMPREHENSIVE VULNERABILITY SCAN — Looking for what might still be exploitable
# =====================================================================================================================
def comprehensive_vuln_scan(data, insns):
    print(f"\n{'='*120}")
    print(f"  COMPREHENSIVE VULNERABILITY SCAN")
    print(f"{'='*120}\n")
    
    # 1. Find all CBZ/CBNZ on return values of BL — missing error checks
    print(f"  === Missing error checks after function calls ===")
    unchecked_calls = []
    total_calls = 0
    checked_calls = 0
    
    for i in range(len(insns) - 3):
        if insns[i].mnemonic == 'bl':
            total_calls += 1
            # Check if x0/w0 is checked within next 5 instructions
            x0_checked = False
            x0_used_unchecked = False
            
            for j in range(i+1, min(i+6, len(insns))):
                op = insns[j].op_str
                mn = insns[j].mnemonic
                
                # x0 checked: cbz/cbnz x0/w0, or cmp x0/w0 
                if mn in ('cbz', 'cbnz') and op.startswith(('w0,', 'x0,')):
                    x0_checked = True
                    checked_calls += 1
                    break
                if mn == 'cmp' and op.startswith(('w0,', 'x0,')):
                    x0_checked = True
                    checked_calls += 1
                    break
                # x0 stored (might be checked later)
                if mn == 'mov' and 'x0' in op.split(',')[1:]:
                    x0_checked = True  # stored for later check
                    break
                if mn == 'str' and op.startswith(('w0,', 'x0,')):
                    x0_checked = True
                    break
                # x0 used as pointer without check
                if mn in ('ldr', 'str', 'stp', 'ldp') and '[x0' in op:
                    x0_used_unchecked = True
                    break
                if mn == 'bl':  # another call, x0 passed as arg
                    x0_checked = True
                    break
            
            if x0_used_unchecked:
                unchecked_calls.append(insns[i].address)
    
    print(f"  Total BL calls: {total_calls}, Checked: {checked_calls}")
    print(f"  x0 used as pointer without null check: {len(unchecked_calls)}")
    for addr in unchecked_calls[:15]:
        func = find_function_start(data, addr)
        print(f"    0x{addr:X} (in func 0x{func:X})")
    
    # 2. Find all TBNZ/TBZ patterns — bit testing (permission/flag checks)
    print(f"\n  === Security-critical bit tests (TBNZ/TBZ) ===")
    bit_tests = []
    for insn in insns:
        if insn.mnemonic in ('tbnz', 'tbz'):
            ops = insn.op_str.split(',')
            if len(ops) >= 2:
                try:
                    bit = int(ops[1].strip().replace('#', ''), 0)
                    bit_tests.append((insn.address, insn.mnemonic, insn.op_str, bit))
                except:
                    pass
    
    print(f"  Found {len(bit_tests)} bit tests:")
    # Group by bit number
    by_bit = defaultdict(list)
    for addr, mn, op, bit in bit_tests:
        by_bit[bit].append((addr, mn, op))
    
    for bit, tests in sorted(by_bit.items()):
        print(f"    Bit #{bit}: {len(tests)} tests")
        for addr, mn, op in tests[:3]:
            print(f"      0x{addr:X}: {mn} {op}")
    
    # 3. Find all DSB/DMB/ISB barriers
    print(f"\n  === Memory barriers ===")
    barriers = []
    for insn in insns:
        if insn.mnemonic in ('dsb', 'dmb', 'isb'):
            barriers.append((insn.address, f"{insn.mnemonic} {insn.op_str}"))
    
    print(f"  Total barriers: {len(barriers)}")
    for addr, b in barriers[:20]:
        print(f"    0x{addr:X}: {b}")
    
    # 4. Find MSR/MRS (system register access) — privilege operations
    print(f"\n  === System register access (MSR/MRS) ===")
    sysregs = defaultdict(list)
    for insn in insns:
        if insn.mnemonic in ('msr', 'mrs'):
            sysregs[insn.op_str.split(',')[0].strip() if insn.mnemonic == 'msr' else insn.op_str.split(',')[1].strip()].append(
                (insn.address, f"{insn.mnemonic} {insn.op_str}")
            )
    
    print(f"  System registers accessed ({len(sysregs)} unique):")
    for reg, accesses in sorted(sysregs.items()):
        rw = 'R' if any('mrs' in a[1] for a in accesses) else ''
        rw += 'W' if any('msr' in a[1] for a in accesses) else ''
        print(f"    {reg:30s} [{rw}] ({len(accesses)} accesses)")
    
    # 5. Look for interesting constant comparisons that might indicate state machine
    print(f"\n  === Interesting constant comparisons (state/type/cmd values) ===")
    const_cmps = defaultdict(list)
    for insn in insns:
        if insn.mnemonic == 'cmp':
            ops = insn.op_str.split(',')
            if len(ops) >= 2 and '#' in ops[1]:
                try:
                    val = int(ops[1].strip().replace('#', ''), 0)
                    if 1 <= val <= 20:  # Small constants = likely state/command values
                        const_cmps[val].append(insn.address)
                except:
                    pass
    
    print(f"  Small constant comparisons (potential state/command dispatch):")
    for val, addrs in sorted(const_cmps.items()):
        print(f"    cmp ?, #{val}: {len(addrs)} occurrences — {[f'0x{a:X}' for a in addrs[:6]]}")


# =====================================================================================================================
# MAIN
# =====================================================================================================================
if __name__ == '__main__':
    print("ROM VULNERABILITY HUNTER v2 — FOCUSED ANALYSIS")
    print("=" * 120)
    
    data_b1 = load_rom('securerom/t8020_B1_securerom.bin')
    data_a0 = load_rom('securerom/t8020_A0_securerom.bin')
    
    insns_b1 = get_disasm(data_b1)
    print(f"  B1: {len(insns_b1)} instructions disassembled")
    
    # A. Call graph
    callers_of = build_call_graph(insns_b1)
    sorted_callees = print_call_graph(callers_of, data_b1, insns_b1)
    
    # B. DFU handler trace
    boot_calls, loader_calls = trace_dfu_handler(data_b1, insns_b1, callers_of)
    
    # C. New function cross-refs
    find_new_func_callers(callers_of)
    
    # D. img4_verify expansion
    analyze_img4_expansion(data_b1, insns_b1, callers_of)
    
    # E. USB IO buffers
    analyze_usb_io_buffers(data_b1, insns_b1, callers_of)
    
    # F. Comprehensive scan
    comprehensive_vuln_scan(data_b1, insns_b1)
    
    print(f"\n{'='*120}")
    print(f"  HUNT v2 COMPLETE")
    print(f"{'='*120}")
