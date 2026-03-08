#!/usr/bin/env python3
"""
T8020 B1 SecureROM — COMPLETE IMG4/DER/ASN.1 PARSER AUDIT
============================================================
Date: 2026-03-04
Target: ALL parser code in the SecureROM, prioritized by exploitability

Coverage map (what this script analyzes):
  Region A: 0x100004800 - 0x100006000  IMG4 container / IM4P / boot image   (~6KB)
  Region B: 0x10000A700 - 0x10000AB00  img4_verify + image validation        (~1KB)
  Region C: 0x10000D000 - 0x10000F000  DER/ASN.1 core parser                 (~8KB)
  Region D: 0x100012000 - 0x100015000  X.509/Certificate deep parser          (~12KB)
  Region E: 0x100017000 - 0x100018200  IMG4 manifest / payload extraction     (~5KB)
  Region F: 0x100019000 - 0x10001A000  Crypto primitives (SHA/AES wrappers)   (~4KB)
  Region G: 0x100020000 - 0x100022000  Dispatch tables + data region          (~8KB)

For EACH function:
  1. Full disassembly with annotations
  2. Length field load tracking (every ldrb/ldrh that reads parsed data)
  3. Bounds check mapping (which lengths ARE checked, which are NOT)
  4. Integer arithmetic analysis (overflow/underflow detection)
  5. Buffer access analysis (size vs actual access)
  6. Data flow from DFU input → parser → memory write

Output: Structured vulnerability report with prioritized findings
"""

import struct, sys, os, json
from collections import defaultdict, OrderedDict
from pathlib import Path

# ---- Capstone setup ----
try:
    from capstone import *
    from capstone.arm64 import *
except ImportError:
    print("ERROR: pip install capstone")
    sys.exit(1)

ROM_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "t8020_B1_securerom.bin")
ROM_BASE = 0x100000000

with open(ROM_PATH, "rb") as f:
    rom = f.read()

ROM_SIZE = len(rom)
md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

W = 120
SEP = "=" * W
HSEP = "-" * W

# Known function identities from prior RE
KNOWN_FUNCS = {
    0x100008978: "log_printf",
    0x100008B58: "panic",
    0x100010BD0: "memcpy_safe",      # memmove-like
    0x100010D80: "memset_zero",      # specialized memset
    0x100010E00: "memset",
    0x10000F1EC: "malloc",
    0x10000F3E4: "malloc_internal",
    0x10000F468: "free",
    0x100011004: "strlen",
    0x10000AD7C: "platform_get_raw",
    0x100009438: "get_current_task",
    0x100006774: "debug_event_log",
    0x100006754: "debug_event_simple",
    0x100011C70: "lock_acquire",
    0x100011CBC: "lock_release",
    0x10000A704: "img4_verify",
    0x100005480: "img4_verify_internal",
    0x10000A86C: "img4_alloc_context",
    0x10000A8C0: "img4_free_context",
    0x10000D0A8: "der_init_context",
    0x10000D100: "der_boot_init",
}

# Regions of interest for the parser audit
AUDIT_REGIONS = [
    (0x100004800, 0x100006200, "IMG4_CONTAINER"),
    (0x10000A700, 0x10000AB00, "IMG4_VERIFY"),
    (0x10000D000, 0x10000F000, "DER_ASN1_CORE"),
    (0x100012000, 0x100015000, "X509_CERT_DEEP"),
    (0x100017000, 0x100018200, "IMG4_MANIFEST"),
    (0x100019000, 0x10001A200, "CRYPTO_WRAPPERS"),
]

# Functions that copy memory (need size validation before calls)
MEMCPY_FUNCS = {0x100010BD0, 0x100010D80, 0x100010E00, 0x100019364, 0x100019388}
ALLOC_FUNCS = {0x10000F1EC, 0x10000F3E4}
FREE_FUNCS = {0x10000F468}

# ============================================================
# PHASE 0: Build infrastructure
# ============================================================
def read32(off):
    return struct.unpack('<I', rom[off:off+4])[0]

def read64(off):
    return struct.unpack('<Q', rom[off:off+8])[0]

def disasm_range(start, end):
    """Disassemble a range, yielding (addr, raw, insn) tuples."""
    off = start - ROM_BASE
    end_off = end - ROM_BASE
    for pos in range(off, min(end_off, ROM_SIZE - 4), 4):
        raw = read32(pos)
        addr = ROM_BASE + pos
        dis = list(md.disasm(rom[pos:pos+4], addr, 1))
        if dis:
            yield addr, raw, dis[0]
        else:
            yield addr, raw, None

def find_all_functions():
    """Find all function prologues in the ROM.
    Uses multiple heuristics:
    1. STP x29, x30 (standard frame pointer save)
    2. STP with other callee-saved registers (x28,x27 etc) as first instruction
    3. SUB sp, sp followed by STP
    4. PACIBSP marker
    5. BL targets (any address called via BL is a function entry)
    6. Known functions from prior reverse engineering
    """
    funcs = set()
    
    for off in range(0, ROM_SIZE - 4, 4):
        # Method 1: STP x29, x30, [sp, #imm]! or [sp, #imm]
        if rom[off] == 0xFD and rom[off+1] == 0x7B and rom[off+3] == 0xA9:
            funcs.add(ROM_BASE + off)
            continue
        
        # Method 2: Any STP [sp, #imm]! (pre-indexed, saves callee regs)
        # Encoding: byte3 = 0xA9 and Rn field (bits 9:5) = SP (11111)
        raw = read32(off)
        if (raw >> 24) == 0xA9:
            rn = (raw >> 5) & 0x1F
            if rn == 31:  # SP
                imm7 = (raw >> 15) & 0x7F
                # Pre-indexed: bit 23 set
                if raw & (1 << 23):
                    funcs.add(ROM_BASE + off)
                    continue
        
        # Method 3: SUB sp, sp, #imm followed by STP to sp
        if (raw & 0xFF0003FF) == 0xD10003FF:
            if off + 4 < ROM_SIZE - 3:
                if rom[off+7] == 0xA9:
                    nxt = read32(off + 4)
                    rn = (nxt >> 5) & 0x1F
                    if rn == 31:
                        funcs.add(ROM_BASE + off)
                        continue
        
        # Method 4: PACIBSP (A12+ PAC marker)
        if raw == 0xD503237F:
            funcs.add(ROM_BASE + off)
            continue
    
    # Method 5: BL targets (every BL destination is a function entry)
    for off in range(0, ROM_SIZE - 4, 4):
        raw = read32(off)
        if (raw & 0xFC000000) == 0x94000000:  # BL
            imm26 = raw & 0x03FFFFFF
            if imm26 & 0x02000000:
                imm26 = struct.unpack('<i', struct.pack('<I', imm26 | 0xFC000000))[0]
            caller = ROM_BASE + off
            target = caller + (imm26 * 4)
            if ROM_BASE <= target < ROM_BASE + ROM_SIZE:
                funcs.add(target)
    
    # Method 6: Known functions from prior RE
    for addr in KNOWN_FUNCS:
        funcs.add(addr)
    
    return sorted(funcs)

def disasm_function(start_addr, max_instrs=600):
    """Disassemble one function until RET (with tail call detection)."""
    off = start_addr - ROM_BASE
    instructions = []
    ret_count = 0
    for pos in range(off, min(off + max_instrs * 4, ROM_SIZE - 4), 4):
        raw = read32(pos)
        addr = ROM_BASE + pos
        dis = list(md.disasm(rom[pos:pos+4], addr, 1))
        if dis:
            insn = dis[0]
            instructions.append((addr, raw, insn.mnemonic, insn.op_str, insn))
            if raw == 0xD65F03C0:  # RET
                ret_count += 1
                # Check if next instruction is a new prologue
                if pos + 4 < ROM_SIZE:
                    next_raw = read32(pos + 4)
                    if (next_raw & 0xFFC003FF) == 0xA9BF7BFD or next_raw == 0xD503237F:
                        break
                    # If we find SPT x29, x30 pattern, new function
                    if (next_raw & 0xFFE003E0) == 0xA98003E0:
                        rt = next_raw & 0x1F
                        rt2 = (next_raw >> 10) & 0x1F
                        if rt == 29 and rt2 == 30:
                            break
                # After 2 RETs, stop (likely hit next function in padding gaps)
                if ret_count >= 3:
                    break
        else:
            instructions.append((addr, raw, ".word", f"0x{raw:08X}", None))
            # Data in code = function boundary
            if ret_count > 0:
                break
    return instructions

def build_call_xrefs():
    """Build a complete cross-reference map of BL instructions."""
    xrefs = defaultdict(list)
    for off in range(0, ROM_SIZE - 4, 4):
        raw = read32(off)
        if (raw & 0xFC000000) == 0x94000000:  # BL
            imm26 = raw & 0x03FFFFFF
            if imm26 & 0x02000000:
                imm26 = struct.unpack('<i', struct.pack('<I', imm26 | 0xFC000000))[0]
            caller = ROM_BASE + off
            target = caller + (imm26 * 4)
            xrefs[target].append(caller)
    return xrefs

# ============================================================
# PHASE 1: Discover all functions in parser regions
# ============================================================
print(SEP)
print("  T8020 B1 SecureROM — COMPLETE IMG4/DER PARSER VULNERABILITY AUDIT")
print(f"  ROM: {ROM_SIZE} bytes | Base: 0x{ROM_BASE:X}")
print(f"  Audit date: 2026-03-04")
print(SEP)

print("\n[*] Finding all functions...")
all_functions = find_all_functions()
print(f"    Total functions in ROM: {len(all_functions)}")

print("[*] Building cross-reference map...")
xrefs = build_call_xrefs()

# Collect functions per region
region_funcs = OrderedDict()
for rstart, rend, rname in AUDIT_REGIONS:
    funcs_in_region = [f for f in all_functions if rstart <= f < rend]
    region_funcs[rname] = (rstart, rend, funcs_in_region)
    print(f"    {rname}: {len(funcs_in_region)} functions in 0x{rstart:X}-0x{rend:X}")

total_target = sum(len(v[2]) for v in region_funcs.values())
print(f"\n    TOTAL TARGET FUNCTIONS: {total_target}")

# ============================================================
# PHASE 2: Deep analysis of every function
# ============================================================
findings = []  # (severity, type, func_addr, instr_addr, detail)

class VulnFinder:
    """Automated vulnerability pattern detector for AArch64 parser code."""
    
    def __init__(self):
        self.findings = []
        self.length_loads = []      # Where length fields are loaded from parsed data
        self.bounds_checks = []     # Where bounds are verified
        self.unchecked_paths = []   # Where lengths flow to mem ops without check
        self.arithmetic_ops = []    # Add/sub on parsed lengths
        
    def analyze_function(self, func_addr, instructions, region_name):
        """Run all vulnerability patterns on a function."""
        self.cur_func = func_addr
        self.cur_region = region_name
        instrs = instructions  # list of (addr, raw, mnem, ops, insn_obj)
        
        # Track register state for data flow
        reg_source = {}  # reg -> ('length_load', addr) or ('immediate', val) etc
        
        for i, (addr, raw, mnem, ops, insn) in enumerate(instrs):
            # ── TRACK: length field loads from parsed data ──
            if mnem in ('ldrb', 'ldrh', 'ldrsb', 'ldrsh'):
                dest = ops.split(',')[0].strip()
                src = ','.join(ops.split(',')[1:]).strip()
                # If loading from a pointer (not stack/frame), this is parser input
                if 'sp' not in src and 'x29' not in src:
                    reg_source[dest] = ('length_load', addr, mnem)
                    self.length_loads.append((func_addr, addr, mnem, ops))
            
            # Also track ldr (32-bit or 64-bit loads that might be lengths)
            if mnem in ('ldr',) and not ops.startswith(('x29', 'x30')):
                dest = ops.split(',')[0].strip()
                src = ','.join(ops.split(',')[1:]).strip()
                if 'sp' not in src and 'x29' not in src:
                    reg_source[dest] = ('data_load', addr, mnem)
            
            # ── TRACK: bounds checks ──
            if mnem in ('cmp', 'cmn', 'tst', 'ccmp'):
                parts = [p.strip() for p in ops.split(',')]
                self.bounds_checks.append((func_addr, addr, mnem, ops))
                # Mark the compared register as "checked"
                if parts[0] in reg_source:
                    reg_source[parts[0]] = ('checked', addr, mnem)
            
            # ── PATTERN 1: Small load → arithmetic without bounds check ──
            if mnem in ('ldrb', 'ldrh', 'ldrsb', 'ldrsh'):
                dest = ops.split(',')[0].strip()
                self._check_unchecked_arithmetic(instrs, i, dest, addr, mnem, ops)
            
            # ── PATTERN 2: ADD/SUB on length without overflow check ──
            if mnem in ('add', 'sub') and '#' not in ops:
                parts = [p.strip() for p in ops.split(',')]
                if len(parts) >= 3:
                    # Check if any operand came from a length load
                    for p in parts[1:]:
                        if p in reg_source and reg_source[p][0] == 'length_load':
                            # Is the result checked before memory use?
                            self._check_overflow_propagation(instrs, i, parts[0], addr, mnem, ops, p)
            
            # ── PATTERN 3: Integer truncation (ldr xN → cmp wN or ldr xN used as wN) ──
            if mnem == 'cmp':
                parts = [p.strip() for p in ops.split(',')]
                if len(parts) >= 2:
                    self._check_width_truncation(instrs, i, parts, addr)
            
            # ── PATTERN 4: Signed vs unsigned comparison on length ──
            if mnem.startswith('b.') and mnem in ('b.lt', 'b.le', 'b.gt', 'b.ge'):
                # Find the preceding CMP
                for j in range(i-1, max(i-4, 0), -1):
                    if instrs[j][2] == 'cmp':
                        cmp_reg = instrs[j][3].split(',')[0].strip()
                        # Was this register loaded from parsed data?
                        if cmp_reg in reg_source and reg_source[cmp_reg][0] in ('length_load', 'data_load'):
                            self.findings.append(('HIGH', 'SIGNED_CMP_ON_PARSED_LENGTH',
                                func_addr, addr,
                                f"Signed branch {mnem} on CMP {instrs[j][3]} — "
                                f"register {cmp_reg} loaded from parsed data at 0x{reg_source[cmp_reg][1]:X}. "
                                f"Negative length could bypass bounds check!"))
                        break
            
            # ── PATTERN 5: memcpy/memset call without size check ──
            if mnem == 'bl':
                try:
                    target = int(ops.replace('#', '').replace('0x', ''), 16)
                except:
                    target = 0
                if target in MEMCPY_FUNCS:
                    self._check_memcpy_bounds(instrs, i, addr, target)
            
            # ── PATTERN 6: malloc with attacker-controlled size ──
            if mnem == 'bl':
                try:
                    target = int(ops.replace('#', '').replace('0x', ''), 16)
                except:
                    target = 0
                if target in ALLOC_FUNCS:
                    self._check_alloc_size_source(instrs, i, addr, reg_source)
            
            # ── PATTERN 7: Self-recursion (ASN.1 depth bomb) ──
            if mnem == 'bl':
                try:
                    target = int(ops.replace('#', '').replace('0x', ''), 16)
                except:
                    target = 0
                if target == func_addr:
                    # Check if there's a depth counter
                    has_depth = False
                    for j in range(max(0, i-20), i):
                        if instrs[j][2] in ('subs', 'sub') and '#' in instrs[j][3]:
                            for k in range(j+1, min(j+3, i)):
                                if instrs[k][2] in ('cbz', 'cbnz', 'b.eq', 'b.le', 'b.ls'):
                                    has_depth = True
                                    break
                    if not has_depth:
                        self.findings.append(('CRITICAL', 'UNBOUNDED_RECURSION',
                            func_addr, addr,
                            f"Self-recursive call without depth limit — "
                            f"ASN.1 nesting depth bomb → stack overflow!"))
                    else:
                        self.findings.append(('INFO', 'BOUNDED_RECURSION',
                            func_addr, addr,
                            f"Self-recursive call WITH depth counter (safer)"))
            
            # ── PATTERN 8: Indexed array access without bounds check ──
            if mnem in ('ldr', 'ldrb', 'ldrh', 'str', 'strb', 'strh'):
                if insn and len(insn.operands) >= 2:
                    mem_op = insn.operands[-1]
                    if mem_op.type == ARM64_OP_MEM and mem_op.mem.index != 0:
                        # Register-indexed access: ldr x0, [x1, x2]
                        idx_reg = insn.reg_name(mem_op.mem.index)
                        if idx_reg and idx_reg in reg_source:
                            src_type = reg_source[idx_reg][0]
                            if src_type in ('length_load', 'data_load'):
                                self.findings.append(('HIGH', 'UNCHECKED_INDEX_FROM_PARSED',
                                    func_addr, addr,
                                    f"Array access {mnem} {ops} uses index {idx_reg} "
                                    f"loaded from parsed data at 0x{reg_source[idx_reg][1]:X} — "
                                    f"OOB read/write if not bounded!"))
            
            # ── PATTERN 9: Return value from sub-parser not checked ──
            if mnem == 'bl':
                try:
                    target = int(ops.replace('#', '').replace('0x', ''), 16)
                except:
                    target = 0
                # Skip known safe functions
                if target in (0x100008978, 0x100008B58, 0x100006774, 0x100006754,
                              0x100011C70, 0x100011CBC, 0x100009438):
                    continue
                # Check if return is verified
                if i + 1 < len(instrs):
                    checked = False
                    for j in range(i+1, min(i+5, len(instrs))):
                        mn2, op2 = instrs[j][2], instrs[j][3]
                        if mn2 in ('cbz', 'cbnz') and ('x0' in op2 or 'w0' in op2):
                            checked = True; break
                        if mn2 in ('cmp', 'tst') and ('x0' in op2 or 'w0' in op2):
                            checked = True; break
                        if mn2 == 'mov' and op2.startswith(('x0', 'w0')):
                            break  # x0 overwritten = checking lost
                        if mn2 == 'mov' and ('x0' in op2.split(',')[1:] if ',' in op2 else False):
                            checked = True; break  # saved for later check
                    # Only flag for parser-region targets
                    if not checked and any(rs <= target < re for rs, re, _ in AUDIT_REGIONS):
                        func_name = KNOWN_FUNCS.get(target, f"sub_{target:X}")
                        self.findings.append(('MEDIUM', 'PARSER_RETURN_UNCHECKED',
                            func_addr, addr,
                            f"Return from {func_name} (0x{target:X}) not checked — "
                            f"error condition may propagate silently"))
            
            # ── PATTERN 10: DER length decode (the critical pattern) ──
            # Standard DER: byte 0 = tag, byte 1 = length
            # If length byte >= 0x80: long form (0x81=1 byte, 0x82=2 bytes, etc)
            if mnem == 'ldrb':
                dest = ops.split(',')[0].strip()
                # Look for the DER length decode pattern
                for j in range(i+1, min(i+8, len(instrs))):
                    mn2, op2 = instrs[j][2], instrs[j][3]
                    if mn2 == 'cmp' and dest in op2 and '#0x80' in op2:
                        # Found DER length check! Now trace what happens on long form
                        self._trace_der_length_decode(instrs, j, dest, addr, func_addr)
                        break
                    if mn2 == 'and' and dest in op2 and '#0x7f' in op2:
                        # Masking off high bit = extracting long-form count
                        self._trace_der_long_form(instrs, j, dest, addr, func_addr)
                        break
            
            # ── Update register tracking for MOV ──
            if mnem == 'mov':
                parts = [p.strip() for p in ops.split(',')]
                if len(parts) == 2:
                    dst, src = parts
                    if src in reg_source:
                        reg_source[dst] = reg_source[src]
                    elif src.startswith('#'):
                        try:
                            val = int(src.replace('#', '').replace('0x', ''), 16)
                            reg_source[dst] = ('immediate', val, addr)
                        except:
                            pass
            
            # Clear on function call (callee-saved regs preserved, but x0-x7 clobbered)
            if mnem == 'bl':
                for r in ['x0','x1','x2','x3','x4','x5','x6','x7',
                          'w0','w1','w2','w3','w4','w5','w6','w7']:
                    reg_source.pop(r, None)
                # x0 now holds return value
                reg_source['x0'] = ('call_return', addr, mnem)
                reg_source['w0'] = ('call_return', addr, mnem)
    
    def _check_unchecked_arithmetic(self, instrs, load_idx, dest, load_addr, load_mnem, load_ops):
        """Check if a small load (ldrb/ldrh) flows into arithmetic without bounds check."""
        has_cmp = False
        for j in range(load_idx + 1, min(load_idx + 15, len(instrs))):
            mn2, op2 = instrs[j][2], instrs[j][3]
            if mn2 in ('cmp', 'cmn', 'tst') and dest in op2:
                has_cmp = True
                break
            if mn2 in ('add', 'sub', 'lsl', 'mul', 'madd', 'umull') and dest in op2:
                if not has_cmp:
                    self.findings.append(('HIGH', 'LENGTH_ARITH_NO_CHECK',
                        self.cur_func, load_addr,
                        f"{load_mnem} {load_ops} → {mn2} {op2} at 0x{instrs[j][0]:X} "
                        f"WITHOUT bounds check — attacker-controlled arithmetic!"))
                return
            if mn2 == 'ret' or mn2 == 'bl':
                break
    
    def _check_overflow_propagation(self, instrs, arith_idx, result_reg, arith_addr, mnem, ops, source_reg):
        """Check if arithmetic result is used in memory access without overflow check."""
        # ADD/SUB that doesn't set flags → no overflow detection
        if mnem in ('add', 'sub'):  # (not adds/subs)
            for j in range(arith_idx + 1, min(arith_idx + 10, len(instrs))):
                mn2, op2 = instrs[j][2], instrs[j][3]
                if mn2 in ('ldr', 'str', 'ldrb', 'strb', 'ldrh', 'strh', 'ldp', 'stp'):
                    if result_reg in op2:
                        self.findings.append(('HIGH', 'OVERFLOW_TO_MEMACCESS',
                            self.cur_func, arith_addr,
                            f"{mnem} {ops} (no flags!) → memory access {mn2} {op2} "
                            f"at 0x{instrs[j][0]:X} — integer overflow → OOB access"))
                        return
                if mn2 in ('bl',):
                    # If result goes to a function as size argument
                    if result_reg in ('w0', 'x0', 'w2', 'x2'):
                        try:
                            target = int(op2.replace('#', '').replace('0x', ''), 16)
                        except:
                            target = 0
                        if target in MEMCPY_FUNCS:
                            self.findings.append(('CRITICAL', 'OVERFLOW_SIZE_TO_MEMCPY',
                                self.cur_func, arith_addr,
                                f"{mnem} {ops} → size for memcpy at 0x{instrs[j][0]:X} — "
                                f"integer overflow → heap overflow!"))
                            return
    
    def _check_width_truncation(self, instrs, cmp_idx, parts, cmp_addr):
        """Detect 64→32 bit truncation in comparisons."""
        if len(parts) < 2:
            return
        reg = parts[0]
        if not reg.startswith('w'):
            return
        # Check if the corresponding x-register was loaded with 64-bit value
        x_reg = 'x' + reg[1:]
        for j in range(cmp_idx - 1, max(cmp_idx - 15, 0), -1):
            mn2, op2 = instrs[j][2], instrs[j][3]
            if op2.startswith(f'{x_reg},') and mn2 in ('ldr', 'mov', 'add', 'sub'):
                self.findings.append(('HIGH', 'WIDTH_TRUNCATION_CMP',
                    self.cur_func, cmp_addr,
                    f"CMP {','.join(parts)} but {x_reg} was set by {mn2} {op2} "
                    f"at 0x{instrs[j][0]:X} — upper 32 bits silently ignored!"))
                return
            if op2.startswith(f'{reg},'):
                return  # w-register was explicitly set, no truncation
    
    def _check_memcpy_bounds(self, instrs, call_idx, call_addr, target):
        """Check if memcpy/memset size parameter (x2) is validated."""
        # Scan back for size check on x2/w2
        has_size_check = False
        x2_source = None
        for j in range(call_idx - 1, max(call_idx - 20, 0), -1):
            mn2, op2 = instrs[j][2], instrs[j][3]
            if mn2 in ('cmp', 'cmn', 'tst'):
                if 'x2' in op2 or 'w2' in op2:
                    has_size_check = True
                    break
            if mn2 == 'mov' and op2.startswith(('x2,', 'w2,')):
                src = op2.split(',')[1].strip()
                if src.startswith('#'):
                    has_size_check = True  # Constant size = safe
                else:
                    x2_source = (instrs[j][0], src)
                break
            if mn2 in ('ldrb', 'ldrh', 'ldr') and op2.startswith(('x2,', 'w2,')):
                x2_source = (instrs[j][0], 'memory')
                break
        
        if not has_size_check and x2_source:
            func_name = KNOWN_FUNCS.get(target, f"0x{target:X}")
            self.findings.append(('CRITICAL', 'MEMCPY_UNCHECKED_SIZE',
                self.cur_func, call_addr,
                f"Call to {func_name} — size (x2) from {x2_source[1]} "
                f"at 0x{x2_source[0]:X} without bounds validation!"))
    
    def _check_alloc_size_source(self, instrs, call_idx, call_addr, reg_source):
        """Check if malloc size comes from parsed data."""
        # x0 is the size argument
        if 'w0' in reg_source and reg_source['w0'][0] == 'length_load':
            self.findings.append(('HIGH', 'ALLOC_FROM_PARSED_LENGTH',
                self.cur_func, call_addr,
                f"malloc() with size from parsed data loaded at "
                f"0x{reg_source['w0'][1]:X} — attacker controls allocation size"))
        elif 'x0' in reg_source and reg_source['x0'][0] == 'length_load':
            self.findings.append(('HIGH', 'ALLOC_FROM_PARSED_LENGTH',
                self.cur_func, call_addr,
                f"malloc() with size from parsed data loaded at "
                f"0x{reg_source['x0'][1]:X} — attacker controls allocation size"))
    
    def _trace_der_length_decode(self, instrs, cmp_idx, length_reg, load_addr, func_addr):
        """Trace DER length decoding after the cmp #0x80 check."""
        # After cmp reg, #0x80:
        # - b.lo → short form (length = byte value directly)
        # - b.hs → long form (byte & 0x7F = number of length bytes following)
        for j in range(cmp_idx + 1, min(cmp_idx + 4, len(instrs))):
            mn2, op2 = instrs[j][2], instrs[j][3]
            if mn2 in ('b.lo', 'b.hs', 'b.ls', 'b.hi', 'b.cc', 'b.cs'):
                # This is the DER length form branch — look for the long-form path
                self.findings.append(('INFO', 'DER_LENGTH_DECODE',
                    func_addr, load_addr,
                    f"DER length decode: ldrb at 0x{load_addr:X} → "
                    f"cmp {length_reg}, #0x80 → {mn2} at 0x{instrs[j][0]:X}. "
                    f"Long-form length follows."))
                
                # Now trace the long-form path for overflow
                # In long form: length_bytes = reg & 0x7F, then read that many bytes
                # KEY VULN: if length_bytes > 4, the multi-byte length can overflow 32-bit
                for k in range(j + 1, min(j + 20, len(instrs))):
                    mn3, op3 = instrs[k][2], instrs[k][3]
                    # Look for: and reg, reg, #0x7f → gives count of length bytes
                    if mn3 == 'and' and '#0x7f' in op3:
                        count_reg = op3.split(',')[0].strip()
                        # Check if count_reg is bounded (must be ≤ 4 for 32-bit or ≤ 8 for 64-bit)
                        bounded = False
                        for m in range(k + 1, min(k + 8, len(instrs))):
                            mn4, op4 = instrs[m][2], instrs[m][3]
                            if mn4 == 'cmp' and count_reg in op4:
                                bounded = True
                                # Extract the limit
                                if '#' in op4:
                                    limit_str = op4.split('#')[1].strip()
                                    try:
                                        limit = int(limit_str.replace('0x', ''), 16)
                                        if limit > 4:
                                            self.findings.append(('HIGH', 'DER_LENGTH_OVERLARGE',
                                                func_addr, instrs[m][0],
                                                f"DER long-form length count allowed up to {limit} bytes — "
                                                f"can express lengths > 2^32 → integer overflow in size calc!"))
                                    except:
                                        pass
                                break
                        if not bounded:
                            self.findings.append(('CRITICAL', 'DER_LENGTH_COUNT_UNBOUNDED',
                                func_addr, instrs[k][0],
                                f"DER long-form byte count ({count_reg} = lenbyte & 0x7F) "
                                f"is NEVER bounded! A crafted tag with length byte 0x88 → "
                                f"8 bytes of length → 64-bit length value → wrap to small!"))
                        break
                break
    
    def _trace_der_long_form(self, instrs, and_idx, dest_reg, load_addr, func_addr):
        """Trace DER long-form handling after AND #0x7F."""
        # The AND #0x7F extracts the count of subsequent length bytes
        # Look for a loop that reads length bytes
        for j in range(and_idx + 1, min(and_idx + 25, len(instrs))):
            mn2, op2 = instrs[j][2], instrs[j][3]
            # LSL in a loop = building multi-byte integer → overflow risk
            if mn2 in ('lsl', 'orr', 'add') and j > and_idx + 2:
                # Check if this is in a loop (backward branch nearby)
                for k in range(j + 1, min(j + 10, len(instrs))):
                    mn3, op3 = instrs[k][2], instrs[k][3]
                    if mn3 in ('b', 'cbnz', 'b.ne', 'b.gt', 'b.hi'):
                        try:
                            target_str = op3.split('#')[-1].strip() if '#' in op3 else op3.split(',')[-1].strip().lstrip('#')
                            target = int(target_str.replace('0x', ''), 16)
                            if target <= instrs[j][0]:  # Backward branch = loop
                                self.findings.append(('HIGH', 'DER_LENGTH_ACCUMULATE_LOOP',
                                    func_addr, instrs[j][0],
                                    f"DER long-form length accumulated in loop via {mn2} at "
                                    f"0x{instrs[j][0]:X} — if length byte count is unbounded, "
                                    f"accumulator can overflow!"))
                                return
                        except:
                            pass
                    break

    def get_report(self):
        """Generate prioritized vulnerability report."""
        severity_rank = {'CRITICAL': 0, 'HIGH': 1, 'MEDIUM': 2, 'LOW': 3, 'INFO': 4}
        self.findings.sort(key=lambda x: (severity_rank.get(x[0], 99), x[2]))
        return self.findings

# ============================================================
# PHASE 3: Run the audit on every function
# ============================================================
vf = VulnFinder()
func_count = 0
total_instrs = 0

for region_name, (rstart, rend, funcs) in region_funcs.items():
    print(f"\n\n{'#' * W}")
    print(f"# REGION: {region_name} (0x{rstart:X} — 0x{rend:X})")
    print(f"# Functions: {len(funcs)}")
    print(f"{'#' * W}")
    
    for func_addr in funcs:
        instrs = disasm_function(func_addr)
        func_count += 1
        total_instrs += len(instrs)
        
        # Compute call/branch/cmp stats
        calls = [(a, o) for a, r, m, o, _ in instrs if m == 'bl']
        branches = [(a, m, o) for a, r, m, o, _ in instrs if m in ('cbz','cbnz','tbz','tbnz') or m.startswith('b.')]
        compares = [(a, m, o) for a, r, m, o, _ in instrs if m in ('cmp','cmn','tst','ccmp')]
        mem_ops = [(a, m, o) for a, r, m, o, _ in instrs if m in 
                   ('ldr','ldrb','ldrh','ldrsb','ldrsh','ldrsw','str','strb','strh','stp','ldp')]
        
        func_name = KNOWN_FUNCS.get(func_addr, "")
        func_xrefs = xrefs.get(func_addr, [])
        
        print(f"\n{HSEP}")
        print(f"  FUNC 0x{func_addr:X} {func_name}")
        print(f"  {len(instrs)} instrs | {len(calls)} calls | {len(branches)} branches | "
              f"{len(compares)} cmps | {len(mem_ops)} mem_ops")
        print(f"  Called from {len(func_xrefs)} sites: {', '.join(f'0x{x:X}' for x in func_xrefs[:6])}"
              f"{'...' if len(func_xrefs) > 6 else ''}")
        print(HSEP)
        
        # Full disassembly with annotations
        for addr, raw, mnem, ops, insn_obj in instrs:
            markers = []
            
            # Annotate known function calls
            if mnem == 'bl':
                try:
                    target = int(ops.replace('#', '').replace('0x', ''), 16)
                    name = KNOWN_FUNCS.get(target, "")
                    if name:
                        markers.append(f"→ {name}")
                    elif target in MEMCPY_FUNCS:
                        markers.append("→ MEMCPY⚠")
                    elif target in ALLOC_FUNCS:
                        markers.append("→ MALLOC⚠")
                    elif target in FREE_FUNCS:
                        markers.append("→ FREE⚠")
                except:
                    pass
            
            # Annotate parsed-data loads
            if mnem in ('ldrb', 'ldrh') and 'sp' not in ops and 'x29' not in ops:
                markers.append("◀ PARSED_DATA_LOAD")
            
            # Annotate comparisons that look like bounds checks
            if mnem == 'cmp' and '#' in ops:
                markers.append("■ BOUNDS_CHECK")
            
            # Annotate DER magic
            if mnem == 'cmp' and ('#0x80' in ops or '#128' in ops):
                markers.append("★ DER_LENGTH_FORM_CHECK")
            if mnem == 'and' and '#0x7f' in ops:
                markers.append("★ DER_LONG_FORM_EXTRACT")
            if mnem == 'cmp' and '#0x1f' in ops:
                markers.append("★ DER_TAG_CLASS")
            
            # Annotate size-related constants
            if mnem == 'cmp' and '#0x801' in ops:
                markers.append("★ DFU_LIMIT_CHECK")
            
            # Annotate potential issues
            if mnem == 'ret':
                markers.append("◀ RETURN")
            
            marker_str = "  " + " | ".join(markers) if markers else ""
            print(f"    0x{addr:X}: {mnem:8s} {ops:44s} ; {raw:08X}{marker_str}")
        
        # Run vulnerability patterns
        vf.analyze_function(func_addr, instrs, region_name)

# ============================================================
# PHASE 4: Also analyze the dispatch tables
# ============================================================
print(f"\n\n{'#' * W}")
print(f"# DISPATCH TABLE ANALYSIS")
print(f"{'#' * W}\n")

# Table at 0x100020000+ area
# Look for contiguous pointer arrays
for table_base in [0x1000211A0, 0x100021038, 0x100020F00]:
    off = table_base - ROM_BASE
    if off < 0 or off >= ROM_SIZE - 8:
        continue
    
    entries = []
    for i in range(256):  # max 256 entries
        ptr_off = off + i * 8
        if ptr_off + 8 > ROM_SIZE:
            break
        ptr = read64(ptr_off)
        if ptr == 0 or not (ROM_BASE <= ptr < ROM_BASE + ROM_SIZE):
            break
        entries.append((ROM_BASE + ptr_off, ptr))
    
    if entries:
        unique = set(e[1] for e in entries)
        print(f"\n  Table at 0x{table_base:X}: {len(entries)} entries, {len(unique)} unique handlers")
        
        # Check: how is this table indexed? Find references to the table base
        table_refs = []
        for toff in range(0, ROM_SIZE - 4, 4):
            raw = read32(toff)
            addr = ROM_BASE + toff
            # ADRP + ADD pattern pointing near table_base
            if (raw & 0x9F000000) == 0x90000000:  # ADRP
                immhi = (raw >> 5) & 0x7FFFF
                immlo = (raw >> 29) & 0x3
                imm = (immhi << 2) | immlo
                if imm & 0x100000:
                    imm -= 0x200000
                page = (addr & ~0xFFF) + (imm << 12)
                if abs(page - (table_base & ~0xFFF)) < 0x1000:
                    table_refs.append(addr)
        
        if table_refs:
            print(f"  Referenced from: {', '.join(f'0x{r:X}' for r in table_refs[:10])}")
            # For each reference, check if the index is bounded
            for ref_addr in table_refs[:5]:
                ctx_instrs = disasm_function(ref_addr & ~0x3, max_instrs=30)
                for idx, (a, r, m, o, _) in enumerate(ctx_instrs):
                    if a >= ref_addr and m in ('ldr', 'ldrsw') and 'lsl' in o:
                        # This is likely the table lookup: ldr xN, [xBase, xIndex, lsl #3]
                        # Check if index was bounded before
                        bounded = False
                        for j in range(max(0, idx-10), idx):
                            if ctx_instrs[j][2] in ('cmp', 'and', 'ubfx'):
                                if any(ctx_instrs[j][3].startswith(f'{p},') 
                                       for p in ['w0','w1','w2','w3','w8','w9','w10']):
                                    bounded = True
                                    break
                        if not bounded:
                            vf.findings.append(('CRITICAL', 'DISPATCH_TABLE_UNBOUNDED',
                                ref_addr, a,
                                f"Dispatch table at 0x{table_base:X} indexed without bounds check! "
                                f"Attacker tag value → arbitrary function pointer!"))

# ============================================================
# PHASE 5: VULNERABILITY REPORT
# ============================================================
print(f"\n\n{'#' * W}")
print(f"# ╔══════════════════════════════════════════════════════╗")
print(f"# ║     VULNERABILITY AUDIT REPORT — T8020 B1 SecureROM  ║")
print(f"# ╚══════════════════════════════════════════════════════╝")
print(f"{'#' * W}\n")

report = vf.get_report()
severity_counts = defaultdict(int)
for sev, vtype, faddr, iaddr, detail in report:
    severity_counts[sev] += 1

print(f"  Functions audited: {func_count}")
print(f"  Instructions analyzed: {total_instrs}")
print(f"  Length loads tracked: {len(vf.length_loads)}")
print(f"  Bounds checks found: {len(vf.bounds_checks)}")
print(f"")
print(f"  ┌─────────────────────────────────────┐")
print(f"  │ FINDINGS SUMMARY                     │")
print(f"  │   CRITICAL: {severity_counts.get('CRITICAL', 0):4d}                      │")
print(f"  │   HIGH:     {severity_counts.get('HIGH', 0):4d}                      │")
print(f"  │   MEDIUM:   {severity_counts.get('MEDIUM', 0):4d}                      │")
print(f"  │   LOW:      {severity_counts.get('LOW', 0):4d}                      │")
print(f"  │   INFO:     {severity_counts.get('INFO', 0):4d}                      │")
print(f"  └─────────────────────────────────────┘\n")

# Print each finding
for idx, (sev, vtype, faddr, iaddr, detail) in enumerate(report):
    if sev == 'INFO':
        continue  # Skip info-level in main report
    icon = {'CRITICAL': '🔴', 'HIGH': '🟠', 'MEDIUM': '🟡', 'LOW': '🔵'}.get(sev, '⚪')
    func_name = KNOWN_FUNCS.get(faddr, f"func_{faddr:X}")
    print(f"\n  [{idx+1:3d}] {icon} {sev:8s} | {vtype}")
    print(f"        Function: 0x{faddr:X} ({func_name})")
    print(f"        Address:  0x{iaddr:X}")
    print(f"        {detail}")

# ============================================================
# PHASE 6: DER LENGTH FIELD ANALYSIS SUMMARY
# ============================================================
print(f"\n\n{'#' * W}")
print(f"# DER LENGTH FIELD TRACE SUMMARY")
print(f"{'#' * W}\n")

print(f"  {len(vf.length_loads)} parsed data loads tracked across all parser functions:\n")
for func_addr, load_addr, mnem, ops in vf.length_loads:
    func_name = KNOWN_FUNCS.get(func_addr, "")
    # Check if this load has an associated bounds check
    has_check = any(ca == func_addr and abs(ba - load_addr) < 40 
                    for ca, ba, _, _ in vf.bounds_checks)
    status = "✓ CHECKED" if has_check else "✗ UNCHECKED"
    print(f"    0x{load_addr:X} in 0x{func_addr:X} {func_name:20s}: {mnem:5s} {ops:40s} [{status}]")

# ============================================================
# PHASE 7: ATTACK SURFACE MAP
# ============================================================
print(f"\n\n{'#' * W}")
print(f"# ATTACK SURFACE MAP: DFU → PARSER DATA FLOW")
print(f"{'#' * W}\n")

print("""
  DFU DNLOAD (USB EP0 OUT)
       │
       ▼
  io_buffer (0x800 bytes at heap)
       │
       ▼
  DFU state machine (dfuDNLOAD-IDLE → dfuMANIFEST)
       │
       ▼
  Image4 entry: img4_verify (0x10000A704)
       │
       ├──▶ Magic check: IM4P (0x696D3470) or Memz (0x4D656D7A)
       │
       ├──▶ img4_verify_internal (0x100005480)
       │         │
       │         ├──▶ DER tag parse    (0x10000D000+ region)
       │         ├──▶ ASN.1 sequence   (0x10000D100+ region)
       │         ├──▶ X.509 cert       (0x100012000+ region)
       │         └──▶ IMG4 manifest    (0x100017000+ region)
       │
       ▼
  Signature verification
       │
       ▼
  Boot image (or REJECT → dfuERROR)
  
  ATTACKER CONTROLS: All bytes in io_buffer (up to 0x800 per DNLOAD block)
  TARGET: Any parser function that processes this data without full validation
""")

# ============================================================
# PHASE 8: Export structured data for further analysis
# ============================================================
output_data = {
    "audit_date": "2026-03-04",
    "rom_info": {
        "chip": "T8020", "revision": "B1",
        "version": "iBoot-3865.0.0.4.7",
        "size": ROM_SIZE, "base": hex(ROM_BASE)
    },
    "stats": {
        "functions_audited": func_count,
        "instructions_analyzed": total_instrs,
        "length_loads": len(vf.length_loads),
        "bounds_checks": len(vf.bounds_checks),
    },
    "findings": [
        {
            "severity": sev, "type": vtype,
            "function": hex(faddr), "address": hex(iaddr),
            "detail": detail
        }
        for sev, vtype, faddr, iaddr, detail in report
        if sev != 'INFO'
    ],
    "length_loads": [
        {"function": hex(fa), "address": hex(la), "instruction": f"{m} {o}"}
        for fa, la, m, o in vf.length_loads
    ],
    "severity_summary": dict(severity_counts)
}

json_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "img4_audit_results.json")
with open(json_path, 'w') as f:
    json.dump(output_data, f, indent=2)
print(f"\n  [*] Structured results exported to: {json_path}")

print(f"\n{SEP}")
print(f"  IMG4/DER PARSER AUDIT COMPLETE — {func_count} functions, {total_instrs} instructions")
print(f"{SEP}")
