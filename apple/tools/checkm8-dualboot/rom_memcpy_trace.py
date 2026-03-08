#!/usr/bin/env python3
"""
rom_memcpy_trace.py - Deep backward-slice verification of CRITICAL memcpy findings
==================================================================================
For each CRITICAL finding (memcpy/memset without x2 size check), we trace 
backward from the call site to determine WHERE x2 comes from:
  - Hardcoded constant? → NOT exploitable (safe)
  - Register loaded from parsed DER data (ldrb/ldrh/ldr from input buffer)? → POTENTIALLY EXPLOITABLE
  - Bounded by a CMP before the call? → Safe if check is correct
  - From heap_alloc return? → Need to check if alloc size was validated

Target: A12 T8020 B1 SecureROM (iPhone XR)
"""

import struct, sys

ROM_PATH = r"C:\Users\nathan\Documents\Scripts\OSrust\tools\checkm8-dualboot\securerom\t8020_B1_securerom.bin"
ROM_BASE = 0x100000000

# Key addresses
MEMCPY    = 0x100010BD0
MEMSET    = 0x100010E00
BZERO     = 0x100010D80
HEAP_ALLOC = 0x10000F1EC
PANIC     = 0x100008978

# CRITICAL finding sites: (function_start, call_addr, target_func)
CRITICAL_SITES = [
    # Function          Call site       Target         Description
    (0x10000D0A8,  0x10000D0C8,  MEMCPY,   "DER entry from IO - memcpy #1"),
    (0x10000D0A8,  0x10000D0E0,  MEMSET,   "DER entry from IO - memset #2"),
    (0x10000D408,  0x10000D4CC,  MEMSET,   "DER alloc+init - memset"),
    (0x10000D5EC,  0x10000D7DC,  MEMCPY,   "DER core parser - memcpy #1"),
    (0x10000D5EC,  0x10000D7F4,  MEMCPY,   "DER core parser - memcpy #2"),
    (0x10000E2D8,  0x10000E328,  BZERO,    "DER setup - bzero"),
    (0x10000E730,  0x10000E758,  MEMSET,   "DER small func - memset"),
    (0x10000E9F0,  0x10000EAA0,  MEMCPY,   "DER chain builder - memcpy #1"),
    (0x10000E9F0,  0x10000EC60,  MEMCPY,   "DER chain builder - memcpy #2"),
    (0x10000ECD0,  0x10000ED84,  MEMCPY,   "DER chain ops - memcpy #1"),
    (0x10000ECD0,  0x10000EE18,  MEMCPY,   "DER chain ops - memcpy #2"),
    (0x10000ECD0,  0x10000EEAC,  MEMCPY,   "DER chain ops - memcpy #3"),
    (0x100012708,  0x10001276C,  MEMCPY,   "X.509 field copy"),
    (0x100012804,  0x100012860,  MEMCPY,   "X.509 dual copy - memcpy #1"),
    (0x100012804,  0x1000128A0,  MEMCPY,   "X.509 dual copy - memcpy #2"),
    (0x100012EA8,  0x100012EE0,  BZERO,    "X.509 init - bzero"),
    (0x100013EB0,  0x100013ED4,  BZERO,    "Cert deep - bzero"),
    (0x100014108,  0x10001413C,  BZERO,    "Cert deep ops - bzero #1"),
    (0x100014108,  0x100014150,  MEMSET,   "Cert deep ops - memset #2"),
    (0x100014108,  0x10001419C,  MEMSET,   "Cert deep ops - memset #3"),
    (0x100014108,  0x100014234,  BZERO,    "Cert deep ops - bzero #4"),
    (0x100014440,  0x1000146B4,  MEMSET,   "Cert parser - memset"),
    (0x100014750,  0x100014768,  BZERO,    "Cert tail - bzero"),
]

try:
    from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM
    from capstone.arm64_const import *
except ImportError:
    print("ERROR: capstone not available")
    sys.exit(1)

cs = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
cs.detail = True

with open(ROM_PATH, "rb") as f:
    rom = f.read()

def rom_read(addr, size):
    off = addr - ROM_BASE
    if 0 <= off < len(rom) - size:
        return rom[off:off+size]
    return None

def disasm_range(start, end):
    data = rom_read(start, end - start)
    if not data:
        return []
    return list(cs.disasm(data, start))

def find_func_end(start, max_size=4096):
    """Find the end of a function (ret instruction)."""
    data = rom_read(start, max_size)
    if not data:
        return start + 256
    last_ret = start
    for insn in cs.disasm(data, start):
        if insn.mnemonic == 'ret':
            last_ret = insn.address + 4
        # Stop at next function prologue (after we've seen some instructions)
        if insn.address > start + 16:
            if insn.mnemonic == 'stp' and 'x29' in insn.op_str and 'x30' in insn.op_str:
                # Check if this looks like a new function
                if insn.address - start > 32:
                    return insn.address
    return last_ret

def trace_x2_backward(instrs, call_idx):
    """
    Trace backward from call_idx to find where x2/w2 gets its value.
    Returns a list of (addr, mnemonic, op_str, analysis) tuples showing the chain.
    """
    chain = []
    target_regs = {'x2', 'w2'}  # We're looking for what sets x2/w2
    
    # Also track secondary registers that feed into x2
    secondary_targets = set()
    
    for i in range(call_idx - 1, max(call_idx - 60, -1), -1):
        insn = instrs[i]
        mn = insn.mnemonic
        ops = insn.op_str
        
        # Check if this instruction writes to any of our target registers
        writes_target = False
        for reg in target_regs | secondary_targets:
            # Check if the first operand (destination) matches
            parts = ops.split(',')
            if parts and parts[0].strip() in (reg, reg.replace('x', 'w'), reg.replace('w', 'x')):
                writes_target = True
                break
        
        if not writes_target:
            continue
        
        dest = ops.split(',')[0].strip()
        rest = ','.join(ops.split(',')[1:]).strip()
        
        # Classify the source
        analysis = ""
        
        if mn == 'mov':
            if rest.startswith('#'):
                val = int(rest.replace('#', '').replace('0x', ''), 16) if '0x' in rest else int(rest.replace('#', ''))
                analysis = f"CONSTANT: x2 = {val} (0x{val:x})"
                chain.append((insn.address, mn, ops, analysis))
                if dest in target_regs or dest.replace('w','x') in target_regs or dest.replace('x','w') in target_regs:
                    return chain, "SAFE_CONSTANT", val
                continue
            else:
                # mov from another register - need to trace that reg too
                src_reg = rest.strip()
                analysis = f"REG_MOVE: x2 <- {src_reg} (tracing {src_reg})"
                chain.append((insn.address, mn, ops, analysis))
                secondary_targets.add(src_reg)
                # Also add the other-width version
                if src_reg.startswith('w'):
                    secondary_targets.add('x' + src_reg[1:])
                elif src_reg.startswith('x'):
                    secondary_targets.add('w' + src_reg[1:])
                continue
                
        elif mn in ('ldr', 'ldrb', 'ldrh', 'ldrsh', 'ldrsw', 'ldrsb'):
            # Loading from memory
            if '[sp' in rest or '[x29' in rest:
                analysis = f"STACK_LOAD: x2 from stack {rest}"
                chain.append((insn.address, mn, ops, analysis))
                # Need to find what stored to this stack slot
                secondary_targets.add(dest)
                continue
            elif any(f'[x{r}' in rest for r in range(0, 29)):
                analysis = f"MEM_LOAD: x2 from {rest} ← potential attacker-controlled data!"
                chain.append((insn.address, mn, ops, analysis))
                return chain, "ATTACKER_CONTROLLED_LOAD", rest
            else:
                analysis = f"LOAD: {rest}"
                chain.append((insn.address, mn, ops, analysis))
                continue
                
        elif mn in ('add', 'sub'):
            # Arithmetic on the size
            analysis = f"ARITHMETIC: {ops} — size computed from arithmetic"
            chain.append((insn.address, mn, ops, analysis))
            # Trace the source registers
            for part in rest.split(','):
                p = part.strip()
                if p.startswith(('x', 'w')) and not p.startswith('#'):
                    secondary_targets.add(p)
                    if p.startswith('w'):
                        secondary_targets.add('x' + p[1:])
                    elif p.startswith('x'):
                        secondary_targets.add('w' + p[1:])
            continue
            
        elif mn in ('and', 'orr', 'eor', 'bfi', 'ubfx', 'sbfx', 'ubfiz', 'lsl', 'lsr', 'asr'):
            analysis = f"BITOP: {ops}"
            chain.append((insn.address, mn, ops, analysis))
            for part in rest.split(','):
                p = part.strip()
                if p.startswith(('x', 'w')) and not p.startswith('#'):
                    secondary_targets.add(p)
            continue
            
        elif mn == 'str':
            # Store doesn't set x2
            continue
            
        elif mn in ('adr', 'adrp'):
            analysis = f"CODE/DATA_ADDR: {ops} — pointer, not attacker data"
            chain.append((insn.address, mn, ops, analysis))
            if dest in target_regs:
                return chain, "SAFE_POINTER", ops
            continue
            
        elif mn == 'bl':
            # Return value from a function call
            if 'x0' in target_regs or 'w0' in target_regs:
                analysis = f"FUNC_RETURN: x2 derived from return value of {rest}"
                chain.append((insn.address, mn, ops, analysis))
                return chain, "FUNC_RETURN", rest
            continue
        
        else:
            if dest in target_regs or dest in secondary_targets:
                analysis = f"OTHER: {mn} {ops}"
                chain.append((insn.address, mn, ops, analysis))
    
    return chain, "UNKNOWN", None

def check_bounds_between(instrs, start_idx, call_idx, size_reg='w2'):
    """Check if there's a CMP involving the size register between start_idx and call_idx."""
    checks = []
    reg_variants = {size_reg, size_reg.replace('w', 'x'), size_reg.replace('x', 'w')}
    
    for i in range(start_idx, call_idx):
        insn = instrs[i]
        if insn.mnemonic in ('cmp', 'cmn', 'tst', 'subs', 'cbz', 'cbnz', 'tbnz', 'tbz'):
            ops = insn.op_str
            for reg in reg_variants:
                if reg in ops:
                    checks.append((insn.address, insn.mnemonic, insn.op_str))
    return checks

def analyze_function_context(func_start, call_site, target):
    """Full analysis of a CRITICAL memcpy site."""
    func_end = find_func_end(func_start, 4096)
    func_size = func_end - func_start
    
    instrs = disasm_range(func_start, func_end)
    if not instrs:
        return None
    
    # Find the call instruction index
    call_idx = None
    for i, insn in enumerate(instrs):
        if insn.address == call_site:
            call_idx = i
            break
    
    if call_idx is None:
        # Try wider search
        for i, insn in enumerate(instrs):
            if abs(insn.address - call_site) <= 4:
                call_idx = i
                break
    
    if call_idx is None:
        return {"error": f"Call site 0x{call_site:X} not found in function"}
    
    result = {
        "func_start": func_start,
        "func_end": func_end,
        "func_size": func_size,
        "call_site": call_site,
        "call_idx": call_idx,
        "total_instrs": len(instrs),
    }
    
    # Show the call instruction
    call_insn = instrs[call_idx]
    result["call_insn"] = f"{call_insn.mnemonic} {call_insn.op_str}"
    
    # Show 8 instructions before the call (the setup)
    setup_start = max(0, call_idx - 12)
    setup_lines = []
    for i in range(setup_start, call_idx + 1):
        insn = instrs[i]
        marker = " <<<" if i == call_idx else ""
        setup_lines.append(f"  0x{insn.address:X}: {insn.mnemonic:8s} {insn.op_str}{marker}")
    result["setup_context"] = setup_lines
    
    # Trace x2 backward
    chain, verdict, detail = trace_x2_backward(instrs, call_idx)
    result["x2_trace"] = chain
    result["verdict"] = verdict
    result["verdict_detail"] = detail
    
    # Also trace x0 (destination) and x1 (source) for context
    # For x0:
    x0_source = None
    for i in range(call_idx - 1, max(call_idx - 20, -1), -1):
        insn = instrs[i]
        parts = insn.op_str.split(',')
        if parts and parts[0].strip() in ('x0', 'w0'):
            x0_source = (insn.address, insn.mnemonic, insn.op_str)
            break
    result["x0_source"] = x0_source
    
    # For x1:
    x1_source = None
    for i in range(call_idx - 1, max(call_idx - 20, -1), -1):
        insn = instrs[i]
        parts = insn.op_str.split(',')
        if parts and parts[0].strip() in ('x1', 'w1'):
            x1_source = (insn.address, insn.mnemonic, insn.op_str)
            break
    result["x1_source"] = x1_source
    
    # Check for bounds checks between function start and call site
    bounds = check_bounds_between(instrs, 0, call_idx, 'w2')
    result["bounds_checks_on_x2"] = bounds
    
    # Also check for any CMP in the function that might bound the size indirectly
    all_cmps = []
    for i in range(len(instrs)):
        insn = instrs[i]
        if insn.mnemonic in ('cmp', 'cmn', 'subs'):
            all_cmps.append((insn.address, insn.mnemonic, insn.op_str))
    result["all_comparisons"] = all_cmps
    
    # Check for heap_alloc calls before the memcpy (allocation for destination?)
    alloc_calls = []
    for i in range(call_idx):
        insn = instrs[i]
        if insn.mnemonic == 'bl':
            bl_target = int(insn.op_str.replace('#', ''), 16)
            if bl_target == HEAP_ALLOC:
                alloc_calls.append((insn.address, "heap_alloc"))
    result["alloc_before_memcpy"] = alloc_calls
    
    return result

def full_disasm_func(func_start, max_size=2048):
    """Full disassembly of a function."""
    func_end = find_func_end(func_start, max_size)
    instrs = disasm_range(func_start, func_end)
    lines = []
    for insn in instrs:
        raw = rom_read(insn.address, insn.size)
        hexbytes = raw.hex() if raw else "????????"
        lines.append(f"  0x{insn.address:09X}: {hexbytes:8s}  {insn.mnemonic:8s} {insn.op_str}")
    return lines, instrs

print("=" * 120)
print("  CRITICAL MEMCPY BACKWARD-SLICE VERIFICATION")
print("  Target: A12 T8020 B1 SecureROM")
print("=" * 120)

# =====================================================================================
# PHASE 1: Quick classification of all 23 CRITICAL sites
# =====================================================================================
print(f"\n{'#' * 120}")
print(f"# PHASE 1: QUICK CLASSIFICATION OF ALL {len(CRITICAL_SITES)} CRITICAL SITES")
print(f"{'#' * 120}\n")

exploitable_candidates = []
safe_sites = []

for func_start, call_site, target, desc in CRITICAL_SITES:
    print(f"\n  --- 0x{call_site:X} in 0x{func_start:X}: {desc} ---")
    
    result = analyze_function_context(func_start, call_site, target)
    if result is None or "error" in result:
        print(f"    ERROR: {result}")
        continue
    
    # Show setup context
    for line in result["setup_context"]:
        print(line)
    
    # Show x2 trace
    print(f"\n    x2 TRACE (backward from call):")
    for addr, mn, ops, analysis in result["x2_trace"]:
        print(f"      0x{addr:X}: {mn} {ops}")
        print(f"        → {analysis}")
    
    print(f"\n    VERDICT: {result['verdict']}")
    if result['verdict_detail']:
        print(f"    DETAIL: {result['verdict_detail']}")
    
    # Show x0/x1 sources
    if result["x0_source"]:
        a, m, o = result["x0_source"]
        print(f"    x0 (dst): 0x{a:X}: {m} {o}")
    if result["x1_source"]:
        a, m, o = result["x1_source"]
        print(f"    x1 (src): 0x{a:X}: {m} {o}")
    
    # Bounds checks
    if result["bounds_checks_on_x2"]:
        print(f"    BOUNDS on x2: {len(result['bounds_checks_on_x2'])} check(s)")
        for a, m, o in result["bounds_checks_on_x2"]:
            print(f"      0x{a:X}: {m} {o}")
    else:
        print(f"    BOUNDS on x2: NONE!")
    
    # Heap allocs
    if result["alloc_before_memcpy"]:
        print(f"    Heap allocs before call: {len(result['alloc_before_memcpy'])}")
        for a, d in result["alloc_before_memcpy"]:
            print(f"      0x{a:X}: {d}")
    
    # Classification
    if result["verdict"] == "SAFE_CONSTANT":
        val = result["verdict_detail"]
        print(f"    ➤ SAFE: x2 is hardcoded constant {val} (0x{val:x})")
        safe_sites.append((func_start, call_site, desc, f"constant={val}"))
    elif result["verdict"] == "SAFE_POINTER":
        print(f"    ➤ SAFE: x2 is a code/data pointer")
        safe_sites.append((func_start, call_site, desc, "pointer"))
    elif result["verdict"] == "ATTACKER_CONTROLLED_LOAD":
        print(f"    ➤ !!!! POTENTIALLY EXPLOITABLE: x2 loaded from memory!")
        exploitable_candidates.append((func_start, call_site, desc, result))
    elif result["verdict"] == "FUNC_RETURN":
        print(f"    ➤ NEEDS DEEPER ANALYSIS: x2 from function return")
        exploitable_candidates.append((func_start, call_site, desc, result))
    else:
        print(f"    ➤ UNKNOWN — needs manual review")
        exploitable_candidates.append((func_start, call_site, desc, result))

# =====================================================================================
# PHASE 2: Deep dive on exploitable candidates
# =====================================================================================
print(f"\n\n{'#' * 120}")
print(f"# PHASE 2: DEEP ANALYSIS OF {len(exploitable_candidates)} EXPLOITABLE CANDIDATES")
print(f"{'#' * 120}")

print(f"\n  Safe sites (hardcoded constants): {len(safe_sites)}")
for func, call, desc, reason in safe_sites:
    print(f"    0x{call:X}: {desc} — {reason}")

print(f"\n  Potentially exploitable: {len(exploitable_candidates)}")
for func, call, desc, _ in exploitable_candidates:
    print(f"    0x{call:X} in 0x{func:X}: {desc}")

# For each exploitable candidate, do full function disassembly
for func_start, call_site, desc, result in exploitable_candidates:
    print(f"\n\n  {'=' * 100}")
    print(f"  FULL DISASSEMBLY: 0x{func_start:X} — {desc}")
    print(f"  {'=' * 100}")
    
    lines, instrs = full_disasm_func(func_start)
    for line in lines:
        # Highlight the call site
        addr_hex = f"0x{call_site:09X}"
        if addr_hex.lower() in line.lower():
            print(f"{line}  <<<< CRITICAL MEMCPY HERE")
        elif "0x100010bd0" in line.lower() or "0x100010e00" in line.lower() or "0x100010d80" in line.lower():
            print(f"{line}  <<<< memcpy/memset/bzero")
        elif "0x10000f1ec" in line.lower():
            print(f"{line}  <<<< heap_alloc")
        elif any(mn in line for mn in [' cmp ', ' cmn ', ' subs ', ' cbz ', ' cbnz ']):
            print(f"{line}  <<<< CHECK")
        elif any(mn in line for mn in [' ldrb ', ' ldrh ', ' ldrsh ', ' ldrsb ']):
            print(f"{line}  <<<< SMALL LOAD")
        else:
            print(line)
    
    # All comparisons in the function
    print(f"\n    ALL COMPARISONS in 0x{func_start:X}:")
    for a, m, o in result["all_comparisons"]:
        print(f"      0x{a:X}: {m} {o}")

# =====================================================================================
# PHASE 3: The most critical — 0x10000D0A8 (DER entry from IO, only 72 bytes)
# =====================================================================================
print(f"\n\n{'#' * 120}")
print(f"# PHASE 3: DEEP RE OF 0x10000D0A8 — DER ENTRY POINT FROM IO/TRANSPORT")
print(f"# Called by: 0x10000A4F4, 0x10000A5A0 (img4 loading path)")
print(f"# Only 72 bytes — complete manual RE possible")
print(f"{'#' * 120}")

# Full disassembly with register annotations
instrs_entry = disasm_range(0x10000D0A8, 0x10000D0A8 + 128)
print(f"\n  Complete disassembly (with annotations):\n")
for insn in instrs_entry:
    raw = rom_read(insn.address, insn.size)
    hexbytes = raw.hex() if raw else "????????"
    
    annotation = ""
    if insn.mnemonic == 'bl':
        bl_target = int(insn.op_str.replace('#',''), 16)
        if bl_target == MEMCPY:
            annotation = "  // memcpy(x0=dst, x1=src, x2=size)"
        elif bl_target == MEMSET:
            annotation = "  // memset(x0=dst, x1=val, x2=size)"
        elif bl_target == BZERO:
            annotation = "  // bzero(x0=dst, x2=size)"
        elif bl_target == HEAP_ALLOC:
            annotation = "  // heap_alloc(x0=size, x1=align)"
    elif insn.mnemonic == 'ret':
        annotation = "  // RETURN"
    elif insn.mnemonic in ('stp', 'ldp') and 'x29' in insn.op_str:
        annotation = "  // prologue/epilogue"
    
    print(f"  0x{insn.address:09X}: {hexbytes:8s}  {insn.mnemonic:8s} {insn.op_str:40s}{annotation}")
    
    # Stop after ret
    if insn.mnemonic == 'ret' and insn.address > 0x10000D0A8 + 16:
        break

# Now trace the callers
print(f"\n\n  CALLERS OF 0x10000D0A8:")
for caller_addr in [0x10000A4F4, 0x10000A5A0]:
    print(f"\n  --- Caller 0x{caller_addr:X} context (20 instructions before + after call) ---")
    # Find the context around the call
    context_instrs = disasm_range(caller_addr - 80, caller_addr + 80)
    for insn in context_instrs:
        raw = rom_read(insn.address, insn.size)
        hexbytes = raw.hex() if raw else "????????"
        marker = " <<<<" if insn.address == caller_addr else ""
        print(f"    0x{insn.address:09X}: {hexbytes:8s}  {insn.mnemonic:8s} {insn.op_str}{marker}")

# =====================================================================================
# PHASE 4: Deep analysis of 0x10000D5EC (DER core parser, 820B)
# Most complex function with heap_alloc + memcpy + unchecked length arithmetic
# =====================================================================================
print(f"\n\n{'#' * 120}")
print(f"# PHASE 4: CRITICAL DATA FLOW IN 0x10000D5EC (820B DER CORE PARSER)")
print(f"# Has: 2x memcpy, 2x heap_alloc, unchecked length arithmetic, width mismatch")
print(f"{'#' * 120}")

# Find all heap_alloc calls and their x0 (size) setup
instrs_d5ec = disasm_range(0x10000D5EC, 0x10000D5EC + 900)
print(f"\n  KEY OPERATIONS in 0x10000D5EC:\n")

for i, insn in enumerate(instrs_d5ec):
    if insn.mnemonic == 'bl':
        bl_target = int(insn.op_str.replace('#',''), 16)
        if bl_target in (MEMCPY, MEMSET, BZERO, HEAP_ALLOC):
            # Show 6 instructions before
            name = {MEMCPY: "memcpy", MEMSET: "memset", BZERO: "bzero", HEAP_ALLOC: "heap_alloc"}[bl_target]
            print(f"\n    >>> {name} call at 0x{insn.address:X}:")
            for j in range(max(0, i-8), i+1):
                ins = instrs_d5ec[j]
                marker = " <<<" if j == i else ""
                print(f"      0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str}{marker}")

# =====================================================================================
# PHASE 5: 0x10001489C (ASN.1 main decoder, 1268B, 8x memcpy)
# =====================================================================================
print(f"\n\n{'#' * 120}")
print(f"# PHASE 5: 0x10001489C — ASN.1 MAIN DECODER (8x memcpy, 1268 bytes)")
print(f"{'#' * 120}")

instrs_489c = disasm_range(0x10001489C, 0x10001489C + 1400)
print(f"\n  ALL memcpy CALL SITES with x2 setup:\n")

for i, insn in enumerate(instrs_489c):
    if insn.mnemonic == 'bl':
        bl_target = int(insn.op_str.replace('#',''), 16)
        if bl_target == MEMCPY:
            print(f"\n    >>> memcpy at 0x{insn.address:X}:")
            for j in range(max(0, i-10), min(len(instrs_489c), i+3)):
                ins = instrs_489c[j]
                marker = " <<<" if j == i else ""
                # Check if this sets x2/w2
                parts = ins.op_str.split(',')
                if parts and parts[0].strip() in ('x2', 'w2'):
                    marker += " [SETS x2]"
                print(f"      0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str}{marker}")

# =====================================================================================
# PHASE 6: HIGH-value combined findings
# Functions with BOTH unchecked arithmetic AND memcpy
# =====================================================================================
print(f"\n\n{'#' * 120}")
print(f"# PHASE 6: CONVERGENT FINDINGS — Functions with MULTIPLE vulnerability types")
print(f"{'#' * 120}")

convergent_funcs = {
    0x10000D5EC: "2 CRITICAL memcpy + 1 unchecked length arith + 1 width mismatch + 2 heap_alloc",
    0x100014108: "4 CRITICAL memcpy + 3 unchecked length arith + 1 SUB underflow + 1 width mismatch",
    0x100014440: "1 CRITICAL memcpy + 3 unchecked length arith + 4 SUB underflow",
    0x10000E9F0: "2 CRITICAL memcpy + 2 SUB underflow",
    0x10000ECD0: "3 CRITICAL memcpy + 2 SUB underflow",
}

for func, desc in convergent_funcs.items():
    print(f"\n  0x{func:X}: {desc}")
    
    # Count all CMP and conditional branches
    instrs = disasm_range(func, func + 2048)
    cmp_count = 0
    branch_count = 0
    memcpy_count = 0
    load_count = 0
    
    for insn in instrs:
        if insn.address >= func + 2048:
            break
        if insn.mnemonic in ('cmp', 'cmn', 'subs'):
            cmp_count += 1
        if insn.mnemonic.startswith('b.') or insn.mnemonic in ('cbz', 'cbnz', 'tbz', 'tbnz'):
            branch_count += 1
        if insn.mnemonic == 'bl':
            try:
                t = int(insn.op_str.replace('#',''), 16)
                if t in (MEMCPY, MEMSET, BZERO):
                    memcpy_count += 1
            except:
                pass
        if insn.mnemonic in ('ldrb', 'ldrh', 'ldrsh', 'ldrsb'):
            load_count += 1
    
    print(f"    Comparisons: {cmp_count}, Branches: {branch_count}, memcpy/set: {memcpy_count}, Small loads: {load_count}")

# =====================================================================================
# FINAL VERDICT
# =====================================================================================
print(f"\n\n{'#' * 120}")
print(f"# FINAL EXPLOITATION ASSESSMENT")
print(f"{'#' * 120}")

print(f"""
  SUMMARY:
    Total CRITICAL sites analyzed: {len(CRITICAL_SITES)}
    Confirmed SAFE (hardcoded constant): {len(safe_sites)}
    Need deeper verification: {len(exploitable_candidates)}
    
  KEY INSIGHT:
    If x2 in ANY memcpy call derives from a DER length field that was parsed
    from the input buffer WITHOUT being bounded against the destination buffer
    size, we have a classic heap/stack overflow.
    
    The DER format encodes lengths as:
      - Short form: 1 byte, value 0-127
      - Long form: first byte = 0x80 | num_length_bytes, then N bytes of length
    
    An attacker controls these length fields in malicious DER/img4 data sent
    over USB during DFU mode.
    
  MOST PROMISING TARGETS (in order):
    1. 0x10000D0A8 — DER entry point from IO. Only 72 bytes. Direct from USB.
    2. 0x10000D5EC — DER core parser. heap_alloc(attacker_size) then memcpy(x2=??).
       If alloc size != memcpy size → heap overflow.
    3. 0x100014108 — 4 memcpy + unchecked arithmetic in cert parsing.
    4. 0x10001489C — ASN.1 decoder with 8 memcpy calls in a loop.
""")

print("=" * 120)
print("  ANALYSIS COMPLETE")
print("=" * 120)
