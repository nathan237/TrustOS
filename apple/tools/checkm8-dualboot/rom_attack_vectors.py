#!/usr/bin/env python3
"""
T8020 B1 SecureROM — ATTACK VECTOR VERIFICATION
=================================================
Systematically verify all 4 remaining theoretical vectors:
  1. Heap pool corruption (UAF, double-free, overflow)
  2. DFU lock race conditions (missing unlock, re-entrant)
  3. DWC2 MMIO write timing (missing barriers, race windows)
  4. Vtable/BLR poisoning (who writes the function pointers?)
"""

import struct, os, sys
from collections import defaultdict, OrderedDict
from capstone import *
from capstone.arm64 import *

ROM_PATH = os.path.join(os.path.dirname(__file__), "securerom", "t8020_B1_securerom.bin")
ROM_BASE = 0x100000000
ACTIVE_END = 0x25000  # Active code region

# Known function addresses
POOL_ALLOC  = 0x10000F3B0
POOL_FREE   = 0x10000F468
HEAP_ALLOC  = 0x10000F1EC
HEAP_FREE   = 0x10000F2D8  # Estimate — will verify
MEMCPY      = 0x100010BD0
MEMSET      = 0x100010E00
BZERO       = 0x100010D80
PANIC       = 0x100008978
STACK_CHK   = 0x100008B58

# Lock/unlock function addresses (from DFU analysis)
ACQUIRE_LOCK    = 0x100001D14
ENTER_CRITICAL  = 0x100001F0C
RELEASE_LOCK    = 0x100001F84
SIGNAL_EVENT    = 0x100001F3C
WAIT_EVENT      = 0x100001F54
SIGNAL_OR_WAIT  = 0x100001F6C

# Memory barrier
MEM_BARRIER = 0x10000612C

# DFU context getter
GET_DFU_CTX = 0x100004454

def load_rom():
    with open(ROM_PATH, "rb") as f:
        return f.read()

def get_md():
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    return md

def find_functions(rom, md):
    """Find all function boundaries via multiple methods."""
    funcs = set()

    # Method 1: Scan 4 bytes at a time for STP x29,x30 prologues
    for off in range(0, ACTIVE_END, 4):
        code = rom[off:off+4]
        for ins in md.disasm(code, ROM_BASE + off):
            if ins.mnemonic == 'stp' and 'x29, x30' in ins.op_str and '[sp' in ins.op_str:
                funcs.add(ins.address)

    # Method 2: Scan for all BL targets (called functions)
    for off in range(0, ACTIVE_END, 4):
        code = rom[off:off+4]
        for ins in md.disasm(code, ROM_BASE + off):
            if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
                target = ins.operands[0].imm
                if ROM_BASE <= target < ROM_BASE + ACTIVE_END:
                    funcs.add(target)

    # Method 3: Add all known critical functions explicitly
    known = [
        0x100001D14, 0x100001F0C, 0x100001F24, 0x100001F3C,
        0x100001F54, 0x100001F6C, 0x100001F84,
        0x100002120, 0x1000021E4, 0x10000226C, 0x1000022F4,
        0x1000023A0, 0x100002368, 0x1000023FC, 0x1000026A0,
        0x100002D34, 0x100002D38, 0x100002DB0, 0x100002E30,
        0x100002E38, 0x100002EDC, 0x100002F78, 0x100002F84,
        0x1000030E0, 0x100003184, 0x1000031C0,
        0x100003CF8, 0x100003ED8, 0x100003F88, 0x100003F9C,
        0x1000040C0, 0x100004174, 0x100004240, 0x100004368,
        0x100004454, 0x1000044A8, 0x100004650, 0x1000047BC,
        0x1000049D8, 0x100004B24, 0x100004B60, 0x100004BC4,
        0x100004CB8, 0x10000612C,
        0x100006754, 0x100006774, 0x1000068A4,
        0x100007368, 0x100007DE0, 0x100007DE8,
        0x100008370, 0x1000083B8, 0x100008978, 0x100008B58,
        0x100009B64, 0x100009B78, 0x100009BA8,
        0x10000AA38,
        0x10000F1EC, 0x10000F3B0, 0x10000F468,
        0x100010014, 0x100010BD0, 0x100010D80, 0x100010E00, 0x100010EA4,
        0x100011004, 0x1000113B4,
        0x1000126FC, 0x100012924, 0x100012A6C, 0x100012AD8, 0x100012B44,
        0x100013AAC,
    ]
    funcs.update(known)

    return sorted(funcs)

def disasm_function(rom, md, start, max_bytes=4096):
    """Disassemble from start to first RET, tracking nested branches."""
    off = start - ROM_BASE
    if off < 0 or off >= len(rom):
        return []
    code = rom[off:off+max_bytes]
    result = []
    for ins in md.disasm(code, start):
        result.append(ins)
        if ins.mnemonic == 'ret':
            break
    return result

def resolve_adrp_chain(instrs):
    """Track ADRP+ADD pairs to resolve full addresses."""
    adrp_regs = {}
    resolved = {}  # ins_addr -> resolved_address
    for ins in instrs:
        if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
            if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_IMM:
                adrp_regs[ins.operands[0].reg] = ins.operands[1].imm
        elif ins.mnemonic == 'add' and len(ins.operands) >= 3:
            if (ins.operands[1].type == ARM64_OP_REG and
                ins.operands[2].type == ARM64_OP_IMM and
                ins.operands[1].reg in adrp_regs):
                full = adrp_regs[ins.operands[1].reg] + ins.operands[2].imm
                adrp_regs[ins.operands[0].reg] = full
                resolved[ins.address] = full
    return resolved


# ============================================================================
# VECTOR 1: HEAP POOL CORRUPTION
# ============================================================================

def analyze_heap_pool(rom, md, funcs):
    print("=" * 100)
    print("VECTOR 1: HEAP / POOL CORRUPTION ANALYSIS")
    print("=" * 100)
    print("""
    Looking for:
    - pool_alloc() without matching pool_free() → memory leak (not exploitable)
    - pool_free() without prior pool_alloc() → double-free / UAF ★
    - pool_alloc() result used after pool_free() → use-after-free ★
    - heap_alloc() size controlled by attacker data → heap overflow ★
    - Missing size validation before memcpy into heap buffer ★
    """)

    alloc_sites = []   # (func_addr, call_addr, alloc_type, args_info)
    free_sites = []    # (func_addr, call_addr, free_type)
    memcpy_after_alloc = []  # (func_addr, alloc_addr, memcpy_addr)

    for func_start in funcs:
        instrs = disasm_function(rom, md, func_start)
        if not instrs:
            continue

        # Track state within function
        has_alloc = False
        alloc_addr = 0
        has_free = False
        free_addr = 0
        alloc_reg = None  # Which register holds the allocated pointer
        last_alloc_args = {}
        bl_targets = []

        for i, ins in enumerate(instrs):
            if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
                target = ins.operands[0].imm
                bl_targets.append((ins.address, target))

                if target == POOL_ALLOC:
                    has_alloc = True
                    alloc_addr = ins.address
                    alloc_reg = 'x0'  # return value
                    # Backtrack to find args (w0=type, w1=size, x2=flags)
                    args = {}
                    for j in range(max(0, i-6), i):
                        prev = instrs[j]
                        if prev.mnemonic in ('mov', 'movz') and len(prev.operands) >= 2:
                            if prev.operands[1].type == ARM64_OP_IMM:
                                reg_name = prev.reg_name(prev.operands[0].reg)
                                args[reg_name] = prev.operands[1].imm
                    alloc_sites.append((func_start, ins.address, 'pool_alloc', args))
                    last_alloc_args = args

                elif target == POOL_FREE:
                    has_free = True
                    free_addr = ins.address
                    free_sites.append((func_start, ins.address, 'pool_free'))

                elif target == HEAP_ALLOC:
                    has_alloc = True
                    alloc_addr = ins.address
                    alloc_reg = 'x0'
                    args = {}
                    for j in range(max(0, i-6), i):
                        prev = instrs[j]
                        if prev.mnemonic in ('mov', 'movz') and len(prev.operands) >= 2:
                            if prev.operands[1].type == ARM64_OP_IMM:
                                reg_name = prev.reg_name(prev.operands[0].reg)
                                args[reg_name] = prev.operands[1].imm
                    alloc_sites.append((func_start, ins.address, 'heap_alloc', args))
                    last_alloc_args = args

                elif target == MEMCPY and has_alloc:
                    memcpy_after_alloc.append((func_start, alloc_addr, ins.address))

        # Check for free-without-alloc in same function
        if has_free and not has_alloc:
            print(f"  ⚠️  FREE WITHOUT LOCAL ALLOC: func=0x{func_start:X} free@0x{free_addr:X}")

        # Check for alloc-then-free-then-use pattern
        if has_alloc and has_free:
            # Check ordering
            if free_addr < alloc_addr:
                print(f"  ⚠️  FREE BEFORE ALLOC in same func: func=0x{func_start:X} "
                      f"free@0x{free_addr:X} alloc@0x{alloc_addr:X}")

    print(f"\n  --- ALLOCATION SITES ({len(alloc_sites)}) ---")
    for func, addr, atype, args in alloc_sites:
        args_str = ', '.join(f'{k}=0x{v:X}' for k, v in sorted(args.items()))
        print(f"  func=0x{func:X}  call@0x{addr:X}  type={atype}  args=({args_str})")

    print(f"\n  --- FREE SITES ({len(free_sites)}) ---")
    for func, addr, ftype in free_sites:
        print(f"  func=0x{func:X}  call@0x{addr:X}  type={ftype}")

    print(f"\n  --- MEMCPY AFTER ALLOC ({len(memcpy_after_alloc)}) ---")
    for func, aaddr, maddr in memcpy_after_alloc:
        print(f"  func=0x{func:X}  alloc@0x{aaddr:X}  memcpy@0x{maddr:X}")

    # Deep analysis: pool_alloc/pool_free internals
    print("\n  --- POOL ALLOCATOR INTERNALS ---")
    pool_instrs = disasm_function(rom, md, POOL_ALLOC, 512)
    print(f"  pool_alloc @ 0x{POOL_ALLOC:X}: {len(pool_instrs)} instructions")
    # Look for size validation, overflow checks
    has_size_check = False
    has_overflow_check = False
    for ins in pool_instrs:
        if ins.mnemonic == 'cmp' and len(ins.operands) >= 2:
            has_size_check = True
        if ins.mnemonic in ('adds', 'subs') or (ins.mnemonic == 'cmp' and
            len(ins.operands) >= 2 and ins.operands[1].type == ARM64_OP_IMM):
            has_overflow_check = True
    print(f"  Has size validation: {has_size_check}")
    print(f"  Has overflow check:  {has_overflow_check}")

    # Dump pool_alloc disassembly
    print(f"\n  pool_alloc full disassembly:")
    adrp_regs = {}
    for ins in pool_instrs:
        ann = ""
        if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
            if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_IMM:
                adrp_regs[ins.operands[0].reg] = ins.operands[1].imm
                ann = f"  ; page=0x{ins.operands[1].imm:X}"
        if ins.mnemonic == 'add' and len(ins.operands) >= 3:
            if (ins.operands[1].type == ARM64_OP_REG and ins.operands[2].type == ARM64_OP_IMM and
                ins.operands[1].reg in adrp_regs):
                full = adrp_regs[ins.operands[1].reg] + ins.operands[2].imm
                ann = f"  ; =0x{full:X}"
        if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
            t = ins.operands[0].imm
            names = {PANIC: 'panic', MEMCPY: 'memcpy', MEMSET: 'memset', BZERO: 'bzero',
                     0x10000F1EC: 'heap_alloc'}
            ann = f"  ; CALL {names.get(t, f'sub_{t:X}')}"
        print(f"    0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str:40s}{ann}")

    # pool_free internals
    pf_instrs = disasm_function(rom, md, POOL_FREE, 512)
    print(f"\n  pool_free @ 0x{POOL_FREE:X}: {len(pf_instrs)} instructions")
    print(f"  pool_free full disassembly:")
    adrp_regs = {}
    for ins in pf_instrs:
        ann = ""
        if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
            if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_IMM:
                adrp_regs[ins.operands[0].reg] = ins.operands[1].imm
                ann = f"  ; page=0x{ins.operands[1].imm:X}"
        if ins.mnemonic == 'add' and len(ins.operands) >= 3:
            if (ins.operands[1].type == ARM64_OP_REG and ins.operands[2].type == ARM64_OP_IMM and
                ins.operands[1].reg in adrp_regs):
                full = adrp_regs[ins.operands[1].reg] + ins.operands[2].imm
                ann = f"  ; =0x{full:X}"
        if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
            t = ins.operands[0].imm
            names = {PANIC: 'panic', BZERO: 'bzero', MEMSET: 'memset'}
            ann = f"  ; CALL {names.get(t, f'sub_{t:X}')}"
        print(f"    0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str:40s}{ann}")

    # Also check heap_alloc
    ha_instrs = disasm_function(rom, md, HEAP_ALLOC, 512)
    print(f"\n  heap_alloc @ 0x{HEAP_ALLOC:X}: {len(ha_instrs)} instructions")
    print(f"  heap_alloc full disassembly:")
    for ins in ha_instrs:
        ann = ""
        if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
            t = ins.operands[0].imm
            ann = f"  ; CALL sub_{t:X}"
        print(f"    0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str:40s}{ann}")

    # Look for heap_free candidates near heap_alloc
    print(f"\n  --- SEARCHING FOR HEAP FREE FUNCTION ---")
    for candidate in range(HEAP_ALLOC - 0x200, HEAP_ALLOC + 0x400, 4):
        off = candidate - ROM_BASE
        if off < 0 or off + 4 > len(rom):
            continue
        word = struct.unpack_from('<I', rom, off)[0]
        # STP x29, x30 prologue
        if (word & 0xFFC003E0) == 0xA9007BFD or (word & 0xFFC003E0) == 0xA98003E0:
            ci = disasm_function(rom, md, candidate, 200)
            if not ci:
                continue
            # check if it calls bzero or memset (typical free pattern)
            for cins in ci:
                if cins.mnemonic == 'bl' and cins.operands and cins.operands[0].type == ARM64_OP_IMM:
                    target = cins.operands[0].imm
                    if target in (BZERO, MEMSET):
                        size = ci[-1].address - candidate + 4 if ci else 0
                        print(f"  Candidate free @ 0x{candidate:X} ({size}B) calls {'bzero' if target == BZERO else 'memset'}")


# ============================================================================
# VECTOR 2: DFU LOCK RACE CONDITIONS
# ============================================================================

def analyze_lock_races(rom, md, funcs):
    print("\n\n" + "=" * 100)
    print("VECTOR 2: DFU LOCK / RACE CONDITION ANALYSIS")
    print("=" * 100)
    print("""
    Looking for:
    - acquire_lock() without matching release_lock() → deadlock on re-entry
    - release_lock() without acquire_lock() → double-release ★
    - Multiple functions holding lock simultaneously → re-entrant race ★
    - Error paths that skip release_lock() → lock leak ★
    - Time window between lock release and resource deallocation ★
    """)

    lock_funcs = {
        ACQUIRE_LOCK: 'acquire_lock',
        ENTER_CRITICAL: 'enter_critical',
        RELEASE_LOCK: 'release_lock',
        SIGNAL_EVENT: 'signal_event',
        WAIT_EVENT: 'wait_event',
        SIGNAL_OR_WAIT: 'signal_or_wait',
    }

    lock_analysis = []  # (func_addr, [(call_addr, lock_type)])

    for func_start in funcs:
        instrs = disasm_function(rom, md, func_start)
        if not instrs:
            continue

        lock_calls = []
        for ins in instrs:
            if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
                target = ins.operands[0].imm
                if target in lock_funcs:
                    lock_calls.append((ins.address, lock_funcs[target]))

        if lock_calls:
            lock_analysis.append((func_start, lock_calls))

    print(f"\n  --- FUNCTIONS WITH LOCK OPERATIONS ({len(lock_analysis)}) ---")
    for func, calls in lock_analysis:
        acquires = [(a, n) for a, n in calls if 'acquire' in n or 'enter' in n]
        releases = [(a, n) for a, n in calls if 'release' in n]
        signals  = [(a, n) for a, n in calls if 'signal' in n or 'wait' in n]

        status = "OK"
        if len(acquires) > 0 and len(releases) == 0:
            status = "⚠️ ACQUIRE WITHOUT RELEASE"
        elif len(releases) > 0 and len(acquires) == 0:
            status = "⚠️ RELEASE WITHOUT ACQUIRE"
        elif len(acquires) != len(releases):
            status = "⚠️ MISMATCH"

        print(f"\n  func=0x{func:X}  [{status}]")
        for addr, name in calls:
            print(f"    0x{addr:X}: {name}()")

    # Check for error paths that skip unlock
    print(f"\n  --- ERROR PATH ANALYSIS ---")
    for func, calls in lock_analysis:
        acquires = [a for a, n in calls if 'acquire' in n or 'enter' in n]
        releases = [a for a, n in calls if 'release' in n]
        if not acquires:
            continue

        instrs = disasm_function(rom, md, func)
        if not instrs:
            continue

        # Find all RET instructions and check if release_lock is called before each
        ret_addrs = [ins.address for ins in instrs if ins.mnemonic == 'ret']
        last_acquire = max(acquires)
        last_release = max(releases) if releases else 0

        # Find early returns (branches to RET before release)
        for ins in instrs:
            if ins.mnemonic in ('b.ne', 'b.eq', 'b.lo', 'b.hi', 'b.ge', 'b.lt',
                                'b.ls', 'b.hs', 'cbz', 'cbnz', 'tbz', 'tbnz'):
                # Get branch target
                target = 0
                for op in ins.operands:
                    if op.type == ARM64_OP_IMM:
                        target = op.imm
                        break

                # Check if branch goes past all releases but before ret
                if target > 0 and ins.address > last_acquire:
                    if last_release > 0 and ins.address < last_release and target > last_release:
                        # This branch SKIPS the release
                        pass  # Normal — the target code likely has its own release
                    elif last_release > 0 and target in ret_addrs and ins.address < last_release:
                        print(f"  ⚠️ POTENTIAL LOCK LEAK: func=0x{func:X}")
                        print(f"    Branch at 0x{ins.address:X} ({ins.mnemonic}) → 0x{target:X} (RET)")
                        print(f"    Acquires lock at 0x{last_acquire:X}, release at 0x{last_release:X}")
                        print(f"    Branch BEFORE release, target is RET → may skip unlock!")

    # Check re-entrant call chains
    print(f"\n  --- RE-ENTRANT CALL CHAIN CHECK ---")
    lock_holding_funcs = set()
    for func, calls in lock_analysis:
        acquires = [a for a, n in calls if 'acquire' in n or 'enter' in n]
        if acquires:
            lock_holding_funcs.add(func)

    for func in lock_holding_funcs:
        instrs = disasm_function(rom, md, func)
        if not instrs:
            continue
        for ins in instrs:
            if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
                target = ins.operands[0].imm
                if target in lock_holding_funcs and target != func:
                    print(f"  ⚠️ RE-ENTRANT: 0x{func:X} (holds lock) calls 0x{target:X} (also acquires lock)")


# ============================================================================
# VECTOR 3: DWC2 MMIO WRITE TIMING
# ============================================================================

def analyze_mmio_timing(rom, md, funcs):
    print("\n\n" + "=" * 100)
    print("VECTOR 3: DWC2 MMIO WRITE TIMING ANALYSIS")
    print("=" * 100)
    print("""
    Looking for:
    - MMIO writes without preceding DSB/ISB memory barrier ★
    - Back-to-back MMIO writes without barrier between them ★
    - MMIO read-modify-write without atomicity ★
    - Gap between MMIO config write and data transfer start ★
    - USB reset handling during MMIO write sequence ★
    """)

    # DWC2 MMIO is accessed via:
    # 1. MOV+MOVK to build 0x235008008 (inline construction)
    # 2. Context table at ROM[0x1C5A0] with 0x235100000 pointers
    # 3. Indirect through registers loaded from context

    # Find all STR to registers that could be MMIO
    mmio_writes = []
    barrier_calls = []
    movk_chains = defaultdict(list)  # func -> [(addr, constructed_value)]

    for func_start in funcs:
        instrs = disasm_function(rom, md, func_start)
        if not instrs:
            continue

        # Track MOV+MOVK chain to find 0x2350xxxxx addresses
        mov_regs = {}
        for ins in instrs:
            if ins.mnemonic in ('mov', 'movz') and len(ins.operands) >= 2:
                if (ins.operands[0].type == ARM64_OP_REG and
                    ins.operands[1].type == ARM64_OP_IMM):
                    mov_regs[ins.operands[0].reg] = ins.operands[1].imm

            elif ins.mnemonic == 'movk' and len(ins.operands) >= 2:
                if (ins.operands[0].type == ARM64_OP_REG and
                    ins.operands[1].type == ARM64_OP_IMM):
                    reg = ins.operands[0].reg
                    val = ins.operands[1].imm
                    shift = 0
                    if ins.operands[1].shift.type != 0:
                        shift = ins.operands[1].shift.value
                    if reg in mov_regs:
                        mov_regs[reg] = (mov_regs[reg] & ~(0xFFFF << shift)) | (val << shift)
                        full = mov_regs[reg]
                        if 0x200000000 <= full < 0x300000000:
                            movk_chains[func_start].append((ins.address, full))

            # Track STR to MMIO-range registers
            elif ins.mnemonic in ('str', 'strb', 'strh') and len(ins.operands) >= 2:
                mem_op = ins.operands[1] if ins.operands[1].type == ARM64_OP_MEM else None
                if mem_op and mem_op.mem.base in mov_regs:
                    base = mov_regs[mem_op.mem.base]
                    if 0x200000000 <= base < 0x300000000:
                        addr = base + mem_op.mem.disp
                        mmio_writes.append((func_start, ins.address, addr, ins.mnemonic))

            # Track barrier calls
            elif ins.mnemonic == 'bl' and ins.operands:
                if ins.operands[0].type == ARM64_OP_IMM:
                    if ins.operands[0].imm == MEM_BARRIER:
                        barrier_calls.append((func_start, ins.address))

            # Track DSB/ISB instructions directly
            elif ins.mnemonic in ('dsb', 'isb', 'dmb'):
                barrier_calls.append((func_start, ins.address))

    print(f"\n  --- MOV+MOVK MMIO ADDRESS CONSTRUCTION ({sum(len(v) for v in movk_chains.values())}) ---")
    for func, chains in sorted(movk_chains.items()):
        for addr, val in chains:
            print(f"  func=0x{func:X}  @0x{addr:X}: constructed MMIO 0x{val:X}")

    print(f"\n  --- MMIO WRITE SITES ({len(mmio_writes)}) ---")
    for func, addr, mmio_addr, mnem in mmio_writes:
        print(f"  func=0x{func:X}  @0x{addr:X}: {mnem} → [0x{mmio_addr:X}]")

    print(f"\n  --- MEMORY BARRIER SITES ({len(barrier_calls)}) ---")
    for func, addr in barrier_calls:
        print(f"  func=0x{func:X}  @0x{addr:X}: barrier")

    # Check: are MMIO writes protected by barriers?
    print(f"\n  --- BARRIER PROTECTION CHECK ---")
    for func, write_addr, mmio_addr, mnem in mmio_writes:
        func_barriers = [ba for bf, ba in barrier_calls if bf == func]
        # Find nearest barrier BEFORE this write
        before = [b for b in func_barriers if b < write_addr]
        after = [b for b in func_barriers if b > write_addr]
        nearest_before = max(before) if before else None
        nearest_after = min(after) if after else None

        gap_before = (write_addr - nearest_before) // 4 if nearest_before else None
        gap_after = (nearest_after - write_addr) // 4 if nearest_after else None

        if nearest_before is None and nearest_after is None:
            print(f"  ⚠️ NO BARRIER: @0x{write_addr:X} → [0x{mmio_addr:X}] — zero barriers in function!")
        elif nearest_before is None:
            print(f"  ⚠️ NO BARRIER BEFORE: @0x{write_addr:X} → [0x{mmio_addr:X}] (next after: +{gap_after} insns)")
        else:
            if gap_before and gap_before > 20:
                print(f"  ⚠️ DISTANT BARRIER: @0x{write_addr:X} → [0x{mmio_addr:X}] (barrier {gap_before} insns before)")
            else:
                print(f"  ✓ Protected: @0x{write_addr:X} → [0x{mmio_addr:X}] (barrier {gap_before} insns before)")

    # Check back-to-back writes
    print(f"\n  --- BACK-TO-BACK MMIO WRITE CHECK ---")
    by_func = defaultdict(list)
    for func, addr, mmio_addr, mnem in mmio_writes:
        by_func[func].append((addr, mmio_addr))
    for func, writes in by_func.items():
        writes.sort()
        for i in range(len(writes) - 1):
            gap = (writes[i+1][0] - writes[i][0]) // 4
            func_barriers = [ba for bf, ba in barrier_calls if bf == func]
            barriers_between = [b for b in func_barriers if writes[i][0] < b < writes[i+1][0]]
            if gap <= 4 and not barriers_between:
                print(f"  ⚠️ BACK-TO-BACK @0x{func:X}: "
                      f"[0x{writes[i][1]:X}]@0x{writes[i][0]:X} → "
                      f"[0x{writes[i+1][1]:X}]@0x{writes[i+1][0]:X} "
                      f"({gap} insns apart, NO barrier)")

    # Also find MMIO accesses via context table (indirect through loaded pointers)
    print(f"\n  --- INDIRECT MMIO VIA CONTEXT TABLE ---")
    print("  The DFU context at ROM[0x1C5A0] contains 14 DWC2 register pointers.")
    print("  Functions that call get_dfu_context (0x100004454) and then STR through result:")
    for func_start in funcs:
        instrs = disasm_function(rom, md, func_start)
        if not instrs:
            continue
        calls_ctx = False
        str_through_ctx = []
        for ins in instrs:
            if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
                if ins.operands[0].imm == GET_DFU_CTX:
                    calls_ctx = True
            if calls_ctx and ins.mnemonic in ('str', 'strb', 'strh'):
                # Check if storing through a register loaded from context
                for op in ins.operands:
                    if op.type == ARM64_OP_MEM:
                        str_through_ctx.append(ins.address)
                        break
        if calls_ctx and len(str_through_ctx) > 2:
            print(f"  func=0x{func_start:X}: calls get_dfu_context(), {len(str_through_ctx)} stores via context regs")


# ============================================================================
# VECTOR 4: VTABLE / BLR POISONING
# ============================================================================

def analyze_vtable_poisoning(rom, md, funcs):
    print("\n\n" + "=" * 100)
    print("VECTOR 4: VTABLE / INDIRECT CALL (BLR) POISONING ANALYSIS")
    print("=" * 100)
    print("""
    Looking for:
    - All BLR instructions (indirect calls through register) ★
    - Where the register value comes from (ROM data vs SRAM vs computed)
    - If SRAM-sourced: who can WRITE to that SRAM address?
    - If the pointer is validated before the BLR call
    - PAC (Pointer Authentication) usage
    """)

    blr_sites = []  # (func, addr, reg, source_info)
    br_sites = []   # (func, addr, reg, source_info)

    for func_start in funcs:
        instrs = disasm_function(rom, md, func_start)
        if not instrs:
            continue

        # Resolve ADRP chains for this function
        resolved = resolve_adrp_chain(instrs)

        # Track register sources
        reg_sources = {}  # reg -> (source_type, address_or_info)

        for i, ins in enumerate(instrs):
            # Track LDR from known addresses
            if ins.mnemonic in ('ldr', 'ldrsw') and len(ins.operands) >= 2:
                if ins.operands[1].type == ARM64_OP_MEM:
                    base_reg = ins.operands[1].mem.base
                    disp = ins.operands[1].mem.disp
                    dst_reg = ins.operands[0].reg

                    if base_reg in reg_sources:
                        src_type, src_addr = reg_sources[base_reg]
                        if isinstance(src_addr, int):
                            full = src_addr + disp
                            reg_sources[dst_reg] = ('mem', full)

            # Track ADRP+ADD resolved addresses
            if ins.mnemonic == 'add' and ins.address in resolved:
                dst_reg = ins.operands[0].reg
                reg_sources[dst_reg] = ('resolved', resolved[ins.address])

            # Track MOV between registers
            if ins.mnemonic == 'mov' and len(ins.operands) >= 2:
                if ins.operands[1].type == ARM64_OP_REG:
                    src = ins.operands[1].reg
                    dst = ins.operands[0].reg
                    if src in reg_sources:
                        reg_sources[dst] = reg_sources[src]

            # Find BLR
            if ins.mnemonic == 'blr' and len(ins.operands) >= 1:
                reg = ins.operands[0].reg
                reg_name = ins.reg_name(reg)
                source = reg_sources.get(reg, ('unknown', None))
                blr_sites.append((func_start, ins.address, reg_name, source))

            # Find BR (tail call)
            if ins.mnemonic == 'br' and len(ins.operands) >= 1:
                reg = ins.operands[0].reg
                reg_name = ins.reg_name(reg)
                source = reg_sources.get(reg, ('unknown', None))
                br_sites.append((func_start, ins.address, reg_name, source))

    print(f"\n  --- BLR (INDIRECT CALL) SITES ({len(blr_sites)}) ---")
    sram_blr = []
    for func, addr, reg, (src_type, src_addr) in blr_sites:
        if src_type == 'mem' and isinstance(src_addr, int):
            if 0x19C000000 <= src_addr < 0x1A0000000:
                tag = "★ SRAM"
                sram_blr.append((func, addr, reg, src_addr))
            elif 0x100000000 <= src_addr < 0x100100000:
                tag = "ROM"
            else:
                tag = f"MEM"
            print(f"  func=0x{func:X}  @0x{addr:X}: BLR {reg}  ← [{tag} 0x{src_addr:X}]")
        elif src_type == 'resolved' and isinstance(src_addr, int):
            print(f"  func=0x{func:X}  @0x{addr:X}: BLR {reg}  ← resolved 0x{src_addr:X}")
        else:
            print(f"  func=0x{func:X}  @0x{addr:X}: BLR {reg}  ← {src_type}")

    print(f"\n  --- BR (INDIRECT JUMP) SITES ({len(br_sites)}) ---")
    for func, addr, reg, (src_type, src_addr) in br_sites:
        if src_type == 'mem' and isinstance(src_addr, int):
            print(f"  func=0x{func:X}  @0x{addr:X}: BR {reg}   ← [0x{src_addr:X}]")
        else:
            print(f"  func=0x{func:X}  @0x{addr:X}: BR {reg}   ← {src_type}")

    # CRITICAL: For SRAM-sourced BLR, find all WRITE paths to those addresses
    print(f"\n  --- ★ SRAM-SOURCED BLR: WHO WRITES THE POINTERS? ---")
    sram_targets = set(addr for _, _, _, addr in sram_blr)
    for sram_addr in sorted(sram_targets):
        print(f"\n  Target SRAM address: 0x{sram_addr:X}")
        # Find all STR/STRB instructions that write to this address
        writers = []
        for func_start in funcs:
            instrs = disasm_function(rom, md, func_start)
            if not instrs:
                continue
            resolved = resolve_adrp_chain(instrs)
            reg_sources = {}
            for ins in instrs:
                if ins.mnemonic == 'add' and ins.address in resolved:
                    reg_sources[ins.operands[0].reg] = resolved[ins.address]
                if ins.mnemonic in ('str', 'stp') and len(ins.operands) >= 2:
                    for op in ins.operands:
                        if op.type == ARM64_OP_MEM:
                            base = op.mem.base
                            disp = op.mem.disp
                            if base in reg_sources:
                                write_addr = reg_sources[base] + disp
                                if write_addr == sram_addr or abs(write_addr - sram_addr) < 16:
                                    writers.append((func_start, ins.address, write_addr))

        if writers:
            for wfunc, waddr, wdest in writers:
                print(f"    WRITER: func=0x{wfunc:X}  @0x{waddr:X} → [0x{wdest:X}]")
        else:
            print(f"    NO DIRECT WRITERS FOUND via ADRP — may be written via register indirect")

    # Check for PAC instructions
    print(f"\n  --- POINTER AUTHENTICATION (PAC) CHECK ---")
    pac_count = 0
    for func_start in funcs:
        instrs = disasm_function(rom, md, func_start)
        for ins in instrs:
            if ins.mnemonic.startswith('pac') or ins.mnemonic.startswith('aut') or \
               ins.mnemonic in ('braa', 'brab', 'blraa', 'blrab', 'retaa', 'retab',
                                'xpacd', 'xpaci'):
                pac_count += 1
                print(f"  PAC instruction: func=0x{func_start:X}  @0x{ins.address:X}: {ins.mnemonic} {ins.op_str}")

    if pac_count == 0:
        print("  ❌ NO PAC INSTRUCTIONS FOUND IN ENTIRE ROM")
        print("     All indirect calls (BLR/BR) are UNGUARDED by pointer authentication.")
        print("     If an attacker can corrupt a function pointer in SRAM, the BLR will")
        print("     execute arbitrary code with NO PAC validation.")


# ============================================================================
# MAIN
# ============================================================================

def main():
    rom = load_rom()
    md = get_md()

    print("T8020 B1 SecureROM — ATTACK VECTOR VERIFICATION")
    print(f"ROM size: {len(rom)} bytes, active code: 0x0-0x{ACTIVE_END:X}")
    print()

    # Find all function prologues
    print("Finding function boundaries...")
    funcs = find_functions(rom, md)
    print(f"Found {len(funcs)} functions\n")

    analyze_heap_pool(rom, md, funcs)
    analyze_lock_races(rom, md, funcs)
    analyze_mmio_timing(rom, md, funcs)
    analyze_vtable_poisoning(rom, md, funcs)

    # Final summary
    print("\n\n" + "=" * 100)
    print("FINAL ATTACK VECTOR SUMMARY")
    print("=" * 100)

if __name__ == "__main__":
    main()
