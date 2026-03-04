#!/usr/bin/env python3
"""
T8020 B1 SecureROM — DEEP RE: CERTIFICATE / X.509 / ASN.1 PARSER
=================================================================
Target regions:
  - DER/ASN.1 Core:     0x10000D000 - 0x10000EFFF (40 functions)
  - ASN.1/Cert/DER Deep: 0x100012000 - 0x100014FFF (39 functions)
  - Dispatch tables:     0x1000211A0 - 0x100021988 (200+ entries)
  
Goal: Full disassembly + logic analysis of EVERY function in these regions.
Look for:
  1. Integer overflow in length/size calculations
  2. Unchecked recursion depth
  3. Type confusion via dispatch tables
  4. Buffer over-read/write in field extraction
  5. Signed/unsigned mismatches in comparisons
  6. Missing bounds checks on parsed lengths
  7. Off-by-one in buffer copy operations
"""

import struct
from collections import defaultdict
from capstone import *
from capstone.arm64 import *

ROM_PATH = "securerom/t8020_B1_securerom.bin"
ROM_BASE = 0x100000000

with open(ROM_PATH, "rb") as f:
    rom = f.read()

ROM_SIZE = len(rom)
md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

WIDTH = 120
SEP = "=" * WIDTH

def disasm_func(start_addr, max_bytes=8000):
    """Disassemble a function from start_addr until RET or max_bytes."""
    off = start_addr - ROM_BASE
    instructions = []
    calls = []
    branches = []
    mem_accesses = []
    comparisons = []
    
    for pos in range(off, min(off + max_bytes, ROM_SIZE - 4), 4):
        raw = struct.unpack('<I', rom[pos:pos+4])[0]
        addr = ROM_BASE + pos
        
        dis = list(md.disasm(rom[pos:pos+4], addr, 1))
        if not dis:
            instructions.append((addr, raw, "???", ""))
            continue
        
        insn = dis[0]
        instructions.append((addr, raw, insn.mnemonic, insn.op_str))
        
        # Track BL calls
        if insn.mnemonic == 'bl':
            try:
                target = int(insn.op_str.replace('#', ''), 16)
                calls.append((addr, target))
            except:
                calls.append((addr, 0))
        
        # Track branches
        if insn.mnemonic in ('b', 'b.eq', 'b.ne', 'b.lt', 'b.le', 'b.gt', 'b.ge',
                             'b.hi', 'b.lo', 'b.hs', 'b.ls', 'b.mi', 'b.pl',
                             'b.cs', 'b.cc', 'b.vs', 'b.vc',
                             'cbz', 'cbnz', 'tbz', 'tbnz'):
            branches.append((addr, insn.mnemonic, insn.op_str))
        
        # Track comparisons
        if insn.mnemonic in ('cmp', 'cmn', 'tst', 'ccmp'):
            comparisons.append((addr, insn.mnemonic, insn.op_str))
        
        # Track memory accesses with sizes
        if insn.mnemonic in ('ldr', 'ldrb', 'ldrh', 'ldrsb', 'ldrsh', 'ldrsw',
                             'str', 'strb', 'strh', 'ldp', 'stp', 'ldar', 'stlr'):
            mem_accesses.append((addr, insn.mnemonic, insn.op_str))
        
        # Stop at RET
        if raw == 0xD65F03C0:  # RET
            break
    
    return instructions, calls, branches, mem_accesses, comparisons

def find_func_end(start_addr, max_bytes=8000):
    """Find the end of a function (RET instruction)."""
    off = start_addr - ROM_BASE
    for pos in range(off, min(off + max_bytes, ROM_SIZE - 4), 4):
        if struct.unpack('<I', rom[pos:pos+4])[0] == 0xD65F03C0:
            return ROM_BASE + pos
    return start_addr + max_bytes

# ============================================================
# Build function list for target regions
# ============================================================
functions = []
for off in range(0, ROM_SIZE - 4, 4):
    insn = struct.unpack('<I', rom[off:off+4])[0]
    if (insn & 0xFFC003E0) == 0xA90003E0:
        rt = insn & 0x1F
        rt2 = (insn >> 10) & 0x1F
        if rt == 29 and rt2 == 30:
            functions.append(ROM_BASE + off)
    elif (insn & 0xFFC003E0) == 0xA98003E0:
        rt = insn & 0x1F
        rt2 = (insn >> 10) & 0x1F
        if rt == 29 and rt2 == 30:
            functions.append(ROM_BASE + off)
functions.sort()

# Build cross-reference map
call_xrefs = defaultdict(list)
for off in range(0, ROM_SIZE - 4, 4):
    raw = struct.unpack('<I', rom[off:off+4])[0]
    if (raw & 0xFC000000) == 0x94000000:
        imm26 = raw & 0x03FFFFFF
        if imm26 & 0x02000000:
            imm26 = struct.unpack('<i', struct.pack('<I', imm26 | 0xFC000000))[0]
        caller = ROM_BASE + off
        target = caller + (imm26 * 4)
        call_xrefs[target].append(caller)

# Target regions for cert/ASN.1 parsing
CERT_REGIONS = [
    (0x10000D000, 0x10000F000, "DER/ASN.1 CORE"),
    (0x100012000, 0x100015000, "CERT/X.509/DER DEEP"),
]

# Collect all functions in target regions
target_funcs = []
for region_start, region_end, region_name in CERT_REGIONS:
    for f in functions:
        if region_start <= f < region_end:
            target_funcs.append((f, region_name))

print(SEP)
print(f"  T8020 B1 SecureROM — CERTIFICATE PARSER DEEP REVERSE ENGINEERING")
print(f"  Target: {len(target_funcs)} functions in DER/ASN.1/X.509 regions")
print(SEP)

# ============================================================
# PART 1: Full disassembly of EVERY function
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 1: FULL DISASSEMBLY OF ALL CERT/ASN.1 FUNCTIONS")
print(f"{'#' * WIDTH}\n")

all_func_data = {}

for func_addr, region in target_funcs:
    instructions, calls, branches, mem_accesses, comparisons = disasm_func(func_addr)
    func_end = func_addr + len(instructions) * 4
    
    xrefs = call_xrefs.get(func_addr, [])
    
    all_func_data[func_addr] = {
        'instructions': instructions,
        'calls': calls,
        'branches': branches,
        'mem_accesses': mem_accesses,
        'comparisons': comparisons,
        'xrefs': xrefs,
        'region': region,
        'size': len(instructions) * 4,
    }
    
    print(f"\n{'=' * 90}")
    print(f"  FUNCTION 0x{func_addr:X} ({region})")
    print(f"  Size: {len(instructions)} instructions ({len(instructions)*4} bytes)")
    print(f"  Calls: {len(calls)} | Branches: {len(branches)} | Comparisons: {len(comparisons)}")
    print(f"  Called from: {len(xrefs)} places: {', '.join(f'0x{x:X}' for x in xrefs[:8])}")
    print(f"{'=' * 90}")
    
    for addr, raw, mnem, ops in instructions:
        marker = ""
        if mnem == 'bl':
            marker = " <<< CALL"
        elif mnem == 'ret':
            marker = " <<< RETURN"
        elif mnem in ('cbz', 'cbnz', 'tbz', 'tbnz'):
            marker = " <<< BRANCH"
        elif mnem.startswith('b.'):
            marker = " <<< COND"
        elif mnem in ('cmp', 'cmn', 'tst'):
            marker = " <<< COMPARE"
        elif mnem in ('adds', 'subs') and '#' in ops:
            marker = " <<< ARITHMETIC"
        elif 'ldr' in mnem and 'sp' not in ops and 'x29' not in ops:
            marker = " <<< LOAD"
        elif 'str' in mnem and 'sp' not in ops and 'x29' not in ops:
            marker = " <<< STORE"
        
        print(f"    0x{addr:X}: {mnem:8s} {ops:40s} ; 0x{raw:08X}{marker}")

# ============================================================
# PART 2: VULNERABILITY PATTERN SCAN
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 2: VULNERABILITY PATTERN SCAN")
print(f"{'#' * WIDTH}\n")

vuln_findings = []

for func_addr, data in all_func_data.items():
    instructions = data['instructions']
    comparisons = data['comparisons']
    calls = data['calls']
    branches = data['branches']
    
    # PATTERN 1: Length field loaded as byte/half then used in arithmetic without bounds check
    # Look for: ldrb/ldrh followed by add/sub without cmp before use
    for i, (addr, raw, mnem, ops) in enumerate(instructions):
        # Small load (byte or halfword)
        if mnem in ('ldrb', 'ldrh', 'ldrsb', 'ldrsh'):
            dest_reg = ops.split(',')[0].strip()
            # Check next 10 instructions for arithmetic use without bounds check
            has_cmp = False
            arith_use = None
            for j in range(i+1, min(i+12, len(instructions))):
                _, _, mn2, op2 = instructions[j]
                if mn2 in ('cmp', 'cmn', 'tst') and dest_reg in op2:
                    has_cmp = True
                    break
                if mn2 in ('add', 'adds', 'sub', 'subs') and dest_reg in op2:
                    arith_use = (instructions[j][0], mn2, op2)
                    break
            if arith_use and not has_cmp:
                vuln_findings.append({
                    'type': 'UNCHECKED_LENGTH_ARITHMETIC',
                    'func': func_addr,
                    'addr': addr,
                    'detail': f"Small load {mnem} {ops} -> arithmetic at 0x{arith_use[0]:X}: {arith_use[1]} {arith_use[2]} WITHOUT bounds check",
                    'severity': 'HIGH'
                })
        
        # PATTERN 2: ADD without overflow check (no ADDS or no branch on carry)
        if mnem == 'add' and '#' not in ops:
            # Two-register add without immediate = potential overflow
            parts = ops.split(',')
            if len(parts) >= 3:
                dest = parts[0].strip()
                # Check if this feeds into a memory access
                for j in range(i+1, min(i+8, len(instructions))):
                    _, _, mn2, op2 = instructions[j]
                    if 'str' in mn2 or 'ldr' in mn2:
                        if dest in op2 and '[' in op2:
                            vuln_findings.append({
                                'type': 'UNCHECKED_ADD_TO_MEMACCESS',
                                'func': func_addr,
                                'addr': addr,
                                'detail': f"ADD {ops} feeds memory access {mn2} {op2} at 0x{instructions[j][0]:X}",
                                'severity': 'MEDIUM'
                            })
                            break
        
        # PATTERN 3: Comparison width mismatch (cmp w vs operand from x register)
        if mnem == 'cmp':
            parts = ops.split(',')
            if len(parts) == 2:
                reg1 = parts[0].strip()
                reg2 = parts[1].strip()
                if reg1.startswith('w') and not reg2.startswith('#'):
                    # Look back for where reg1 was set from 64-bit source
                    reg_x = 'x' + reg1[1:]
                    for j in range(max(0, i-10), i):
                        _, _, mn2, op2 = instructions[j]
                        if reg_x in op2 and mn2 in ('mov', 'ldr', 'add'):
                            vuln_findings.append({
                                'type': 'WIDTH_MISMATCH_CMP',
                                'func': func_addr,
                                'addr': addr,
                                'detail': f"CMP {ops} but register may hold 64-bit value from {mn2} {op2} at 0x{instructions[j][0]:X}",
                                'severity': 'HIGH'
                            })
                            break
        
        # PATTERN 4: Sub without underflow check (for length remaining)
        if mnem == 'subs' or mnem == 'sub':
            parts = ops.split(',')
            if len(parts) >= 2:
                # Check if this is a "remaining length" pattern
                if mnem == 'sub':  # No flags set = no underflow check possible
                    for j in range(i+1, min(i+6, len(instructions))):
                        _, _, mn2, op2 = instructions[j]
                        if 'ldr' in mn2 or 'str' in mn2:
                            if parts[0].strip() in op2:
                                vuln_findings.append({
                                    'type': 'SUB_NO_UNDERFLOW_CHECK',
                                    'func': func_addr, 
                                    'addr': addr,
                                    'detail': f"SUB {ops} (no flags!) followed by mem access using result",
                                    'severity': 'HIGH'
                                })
                                break
        
        # PATTERN 5: Unsigned comparison where signed needed (or vice versa)
        # After CMP, check if branch uses signed vs unsigned condition
        if mnem == 'cmp':
            # Check next few instructions for branch type
            for j in range(i+1, min(i+4, len(instructions))):
                _, _, mn2, op2 = instructions[j]
                # Signed branches: b.lt, b.le, b.gt, b.ge
                # Unsigned branches: b.lo, b.ls, b.hi, b.hs (b.cc, b.cs)
                # If comparing what looks like a length (from ldr), should be unsigned
                if mn2 in ('b.lt', 'b.le', 'b.gt', 'b.ge'):
                    # Look back for the source of compared register
                    cmp_reg = ops.split(',')[0].strip()
                    for k in range(max(0, i-8), i):
                        _, _, mn3, op3 = instructions[k]
                        if ('ldr' in mn3 or 'add' in mn3) and cmp_reg in op3.split(',')[0]:
                            vuln_findings.append({
                                'type': 'SIGNED_BRANCH_ON_LENGTH',
                                'func': func_addr,
                                'addr': instructions[j][0],
                                'detail': f"Signed branch {mn2} after CMP {ops} — length field should use unsigned!",
                                'severity': 'MEDIUM'
                            })
                            break
                    break
        
        # PATTERN 6: Memory copy without size validation
        if mnem == 'bl':
            try:
                target = int(ops.replace('#', ''), 16)
            except:
                target = 0
            # Known memcpy-like functions
            MEMCPY_FUNCS = {0x100010BD0, 0x100010D80, 0x100010E00, 0x100011040}
            if target in MEMCPY_FUNCS:
                # Check if x2 (size param) was bounded before the call
                has_size_check = False
                for j in range(max(0, i-15), i):
                    _, _, mn2, op2 = instructions[j]
                    if mn2 in ('cmp', 'cmn', 'tst') and ('x2' in op2 or 'w2' in op2):
                        has_size_check = True
                        break
                if not has_size_check:
                    vuln_findings.append({
                        'type': 'MEMCPY_NO_SIZE_CHECK',
                        'func': func_addr,
                        'addr': addr,
                        'detail': f"Call to memcpy-like 0x{target:X} without size validation on x2",
                        'severity': 'CRITICAL'
                    })
    
    # PATTERN 7: Recursion detection (function calls itself)
    for call_addr, call_target in calls:
        if call_target == func_addr:
            vuln_findings.append({
                'type': 'RECURSIVE_FUNCTION',
                'func': func_addr,
                'addr': call_addr,
                'detail': f"Function calls itself! Recursive parser without depth limit?",
                'severity': 'CRITICAL'
            })
    
    # PATTERN 8: No return value check after sub-call
    for i, (addr, raw, mnem, ops) in enumerate(instructions):
        if mnem == 'bl' and i + 1 < len(instructions):
            next_addr, _, next_mnem, next_ops = instructions[i+1]
            # If next instruction doesn't check w0/x0
            if next_mnem not in ('cbz', 'cbnz', 'cmp', 'tst', 'mov') or \
               ('w0' not in next_ops and 'x0' not in next_ops):
                # Check next 3 instructions
                checked = False
                for j in range(i+1, min(i+4, len(instructions))):
                    _, _, mn2, op2 = instructions[j]
                    if ('w0' in op2 or 'x0' in op2) and mn2 in ('cbz', 'cbnz', 'cmp', 'tst', 'mov', 'str', 'stp'):
                        checked = True
                        break
                if not checked:
                    try:
                        target = int(ops.replace('#', ''), 16)
                    except:
                        target = 0
                    # Skip panic and logging
                    if target not in (0x100008978, 0x100008B58):
                        vuln_findings.append({
                            'type': 'UNCHECKED_RETURN',
                            'func': func_addr,
                            'addr': addr,
                            'detail': f"Return from 0x{target:X} not checked",
                            'severity': 'LOW'
                        })

# Print findings sorted by severity  
print(f"\n  Total findings: {len(vuln_findings)}")
severity_order = {'CRITICAL': 0, 'HIGH': 1, 'MEDIUM': 2, 'LOW': 3}
vuln_findings.sort(key=lambda x: severity_order.get(x['severity'], 99))

for f in vuln_findings:
    sev = f['severity']
    marker = {'CRITICAL': '!!!!', 'HIGH': '!!!', 'MEDIUM': '!!', 'LOW': '!'}[sev]
    print(f"\n  [{sev:8s}] {marker} 0x{f['addr']:X} in func 0x{f['func']:X}")
    print(f"    Type: {f['type']}")
    print(f"    {f['detail']}")

# ============================================================
# PART 3: DISPATCH TABLE ANALYSIS
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 3: ASN.1 DISPATCH TABLE REVERSE ENGINEERING")
print(f"{'#' * WIDTH}\n")

# The massive dispatch table at 0x1000211A0 - 0x100021988
DISPATCH_START = 0x1000211A0
DISPATCH_END = 0x100021990

dispatch_off = DISPATCH_START - ROM_BASE
handler_counts = defaultdict(int)
all_handlers = []

for off in range(dispatch_off, DISPATCH_END - ROM_BASE, 8):
    if off + 8 > ROM_SIZE:
        break
    ptr = struct.unpack('<Q', rom[off:off+8])[0]
    if ROM_BASE <= ptr < ROM_BASE + ROM_SIZE:
        handler_counts[ptr] += 1
        all_handlers.append((ROM_BASE + off, ptr))

print(f"  Dispatch table entries: {len(all_handlers)}")
print(f"  Unique handlers: {len(handler_counts)}")
print(f"\n  Handler frequency:")
for handler, count in sorted(handler_counts.items(), key=lambda x: -x[1]):
    print(f"    0x{handler:X}: {count}x entries")

# Now disassemble each unique handler
print(f"\n  Disassembly of each unique handler:")
for handler in sorted(handler_counts.keys()):
    count = handler_counts[handler]
    print(f"\n  --- Handler 0x{handler:X} ({count}x in dispatch table) ---")
    
    off = handler - ROM_BASE
    if off < 0 or off >= ROM_SIZE - 4:
        print(f"    [out of ROM range]")
        continue
    
    # Disassemble up to 40 instructions or RET
    for pos in range(off, min(off + 160, ROM_SIZE - 4), 4):
        raw = struct.unpack('<I', rom[pos:pos+4])[0]
        addr = ROM_BASE + pos
        dis = list(md.disasm(rom[pos:pos+4], addr, 1))
        if dis:
            insn = dis[0]
            marker = ""
            if insn.mnemonic == 'bl':
                marker = " <<< CALL"
            elif insn.mnemonic == 'ret':
                marker = " <<< RET"
            elif insn.mnemonic in ('cbz', 'cbnz', 'tbz', 'tbnz') or insn.mnemonic.startswith('b.'):
                marker = " <<< BRANCH"
            elif insn.mnemonic in ('cmp', 'tst'):
                marker = " <<< CMP"
            print(f"    0x{addr:X}: {insn.mnemonic:8s} {insn.op_str:40s}{marker}")
        else:
            print(f"    0x{addr:X}: .word 0x{raw:08X}")
        
        if raw == 0xD65F03C0:  # RET
            break

# ============================================================
# PART 4: SECOND DISPATCH TABLE (IO/Transport at 0x100021038)
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 4: IO/TRANSPORT DISPATCH TABLE")
print(f"{'#' * WIDTH}\n")

IO_DISPATCH_START = 0x100021038
IO_DISPATCH_END = 0x1000210D0

io_off = IO_DISPATCH_START - ROM_BASE
io_handlers = []
for off in range(io_off, IO_DISPATCH_END - ROM_BASE, 8):
    if off + 8 > ROM_SIZE:
        break
    ptr = struct.unpack('<Q', rom[off:off+8])[0]
    if ROM_BASE <= ptr < ROM_BASE + ROM_SIZE:
        io_handlers.append((ROM_BASE + off, ptr))

print(f"  IO dispatch entries: {len(io_handlers)}")
seen = set()
for table_addr, handler in io_handlers:
    if handler not in seen:
        seen.add(handler)
        print(f"\n  IO Handler 0x{handler:X}:")
        off = handler - ROM_BASE
        for pos in range(off, min(off + 80, ROM_SIZE - 4), 4):
            raw = struct.unpack('<I', rom[pos:pos+4])[0]
            addr = ROM_BASE + pos
            dis = list(md.disasm(rom[pos:pos+4], addr, 1))
            if dis:
                insn = dis[0]
                print(f"    0x{addr:X}: {insn.mnemonic:8s} {insn.op_str}")
            if raw == 0xD65F03C0:
                break

# ============================================================  
# PART 5: CALL GRAPH — What calls the cert parser?
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 5: CALL GRAPH — WHO CALLS THE CERT/ASN.1 FUNCTIONS?")
print(f"{'#' * WIDTH}\n")

for func_addr, data in sorted(all_func_data.items()):
    xrefs = data['xrefs']
    calls = data['calls']
    if xrefs or calls:
        print(f"\n  0x{func_addr:X} ({data['region']}, {data['size']}B):")
        if xrefs:
            # Determine which region each caller is in
            for caller in xrefs:
                caller_off = caller - ROM_BASE
                if caller_off < 0x3000:
                    caller_region = "BOOT"
                elif caller_off < 0x5000:
                    caller_region = "DFU"
                elif caller_off < 0xA000:
                    caller_region = "VERIFY/CONFIG"
                elif caller_off < 0xD000:
                    caller_region = "IO/TRANSPORT"
                elif caller_off < 0xF000:
                    caller_region = "DER/ASN.1"
                elif caller_off < 0x12000:
                    caller_region = "HEAP/MEM"
                elif caller_off < 0x15000:
                    caller_region = "CERT/X509"
                elif caller_off < 0x1C000:
                    caller_region = "CRYPTO"
                else:
                    caller_region = "DATA"
            print(f"    CALLED BY: {', '.join(f'0x{x:X}({caller_region})' for x in xrefs)}")
        if calls:
            print(f"    CALLS: {', '.join(f'0x{t:X}' for _, t in calls[:15])}")
            if len(calls) > 15:
                print(f"           ... +{len(calls)-15} more")

# ============================================================
# PART 6: LENGTH FIELD EXTRACTION PATTERNS
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 6: LENGTH FIELD HANDLING — How does the parser read lengths?")
print(f"{'#' * WIDTH}\n")

# In DER encoding, length is encoded as:
# - 1 byte if < 0x80
# - 0x81 NN for 1-byte length
# - 0x82 NN NN for 2-byte length
# Look for patterns: ldrb → cmp #0x80 → branch → multi-byte length read

for func_addr, data in all_func_data.items():
    instructions = data['instructions']
    length_patterns = []
    
    for i, (addr, raw, mnem, ops) in enumerate(instructions):
        # Pattern: LDRB followed by CMP with #0x80 or AND with #0x80
        if mnem == 'ldrb':
            dest = ops.split(',')[0].strip()
            for j in range(i+1, min(i+6, len(instructions))):
                _, _, mn2, op2 = instructions[j]
                if mn2 in ('cmp', 'tst', 'and') and dest in op2:
                    if '#0x80' in op2 or '#128' in op2 or '#0x7f' in op2:
                        length_patterns.append((addr, instructions[j][0], 
                            f"DER length decode: {mnem} {ops} -> {mn2} {op2}"))
                        break
        
        # Pattern: LDRB followed by AND #0x1f or #0x3f (tag extraction)
        if mnem == 'ldrb':
            dest = ops.split(',')[0].strip()
            for j in range(i+1, min(i+4, len(instructions))):
                _, _, mn2, op2 = instructions[j]
                if mn2 == 'and' and dest in op2:
                    if '#0x1f' in op2 or '#0x3f' in op2 or '#0x20' in op2 or '#0xc0' in op2:
                        length_patterns.append((addr, instructions[j][0],
                            f"DER tag decode: {mnem} {ops} -> {mn2} {op2}"))
                        break
    
    if length_patterns:
        print(f"\n  Function 0x{func_addr:X}:")
        for load_addr, check_addr, desc in length_patterns:
            print(f"    0x{load_addr:X} -> 0x{check_addr:X}: {desc}")

# ============================================================
# PART 7: CRITICAL FINDINGS SUMMARY
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PART 7: CRITICAL FINDINGS SUMMARY")
print(f"{'#' * WIDTH}\n")

critical = [f for f in vuln_findings if f['severity'] in ('CRITICAL', 'HIGH')]
print(f"  CRITICAL + HIGH findings: {len(critical)}")
for f in critical:
    print(f"\n  {'!'*4 if f['severity']=='CRITICAL' else '!'*3} [{f['severity']}] in 0x{f['func']:X} at 0x{f['addr']:X}")
    print(f"      Type: {f['type']}")
    print(f"      {f['detail']}")

# Summary stats
print(f"\n  STATS:")
print(f"    Functions analyzed: {len(all_func_data)}")
print(f"    Total instructions: {sum(len(d['instructions']) for d in all_func_data.values())}")
print(f"    Total sub-calls: {sum(len(d['calls']) for d in all_func_data.values())}")
print(f"    Total comparisons: {sum(len(d['comparisons']) for d in all_func_data.values())}")
print(f"    Total memory accesses: {sum(len(d['mem_accesses']) for d in all_func_data.values())}")
print(f"    Vulnerability findings: {len(vuln_findings)}")
print(f"      CRITICAL: {sum(1 for f in vuln_findings if f['severity']=='CRITICAL')}")
print(f"      HIGH:     {sum(1 for f in vuln_findings if f['severity']=='HIGH')}")
print(f"      MEDIUM:   {sum(1 for f in vuln_findings if f['severity']=='MEDIUM')}")
print(f"      LOW:      {sum(1 for f in vuln_findings if f['severity']=='LOW')}")

print(f"\n{SEP}")
print(f"  CERT PARSER ANALYSIS COMPLETE")
print(f"{SEP}")
