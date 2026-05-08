//! aarch64 CPU Primitives
//!
//! ARM system registers, CPU identification, and low-level operations.

/// Read the stack pointer (SP)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_stack_pointer() -> u64 {
    let sp: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("mov {}, sp", out(reg) sp, options(nomem, nostack, preserves_flags));
    }
    sp
}

/// Read the frame pointer (X29 / FP)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_frame_pointer() -> u64 {
    let fp: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("mov {}, x29", out(reg) fp, options(nomem, nostack, preserves_flags));
    }
    fp
}

/// Read DAIF (interrupt mask flags)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_daif() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("mrs {}, DAIF", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// I/O wait — use ISB (instruction synchronization barrier) as delay
#[inline(always)]
// Public function — callable from other modules.
pub fn io_wait() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("isb", options(nomem, nostack, preserves_flags));
    }
}

/// Software breakpoint (BRK #0)
#[inline(always)]
// Public function — callable from other modules.
pub fn breakpoint() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("brk #0", options(nomem, nostack));
    }
}

/// Data Synchronization Barrier (DSB SY)
#[inline(always)]
// Public function — callable from other modules.
pub fn dsb_sy() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("dsb sy", options(nomem, nostack, preserves_flags));
    }
}

/// Instruction Synchronization Barrier (ISB)
#[inline(always)]
// Public function — callable from other modules.
pub fn isb() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("isb", options(nomem, nostack, preserves_flags));
    }
}

/// Data Memory Barrier (DMB SY)
#[inline(always)]
// Public function — callable from other modules.
pub fn dmb_sy() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("dmb sy", options(nomem, nostack, preserves_flags));
    }
}

/// Read MPIDR_EL1 (Multiprocessor Affinity Register — CPU ID)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_mpidr() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("mrs {}, MPIDR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read MIDR_EL1 (Main ID Register — CPU model)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_midr() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("mrs {}, MIDR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read current Exception Level (from CurrentEL register)
#[inline(always)]
// Public function — callable from other modules.
pub fn current_el() -> u8 {
    let el: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("mrs {}, CurrentEL", out(reg) el, options(nomem, nostack, preserves_flags));
    }
    ((el >> 2) & 0x3) as u8
}

/// Read SCTLR_EL1 (System Control Register)
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn read_sctlr_el1() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, SCTLR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}

/// Write SCTLR_EL1
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_sctlr_el1(val: u64) {
    core::arch::asm!("msr SCTLR_EL1, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    isb();
}

/// Read TCR_EL1 (Translation Control Register)
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn read_tcr_el1() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, TCR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}

/// Write TCR_EL1
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_tcr_el1(val: u64) {
    core::arch::asm!("msr TCR_EL1, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    isb();
}

/// Read VBAR_EL1 (Vector Base Address Register)
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn read_vbar_el1() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, VBAR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}

/// Write VBAR_EL1
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_vbar_el1(val: u64) {
    core::arch::asm!("msr VBAR_EL1, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    isb();
}

/// Read ESR_EL1 (Exception Syndrome Register — fault info)
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn read_esr_el1() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, ESR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}

/// Read FAR_EL1 (Fault Address Register)
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn read_far_el1() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, FAR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}

/// Read ELR_EL1 (Exception Link Register — return address)
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn read_elr_el1() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, ELR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}

/// Read SP_EL0 (User stack pointer)
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn read_sp_el0() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, SP_EL0", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}

/// Write SP_EL0
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_sp_el0(val: u64) {
    core::arch::asm!("msr SP_EL0, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

// ============================================================================
// MMIO (Memory-Mapped I/O) — ARM has no I/O ports, everything is MMIO
// ============================================================================

/// Read a 32-bit value from an MMIO address
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn mmio_read32(addr: u64) -> u32 {
    let val: u32;
    core::arch::asm!(
        "ldr {val:w}, [{addr}]",
        addr = in(reg) addr,
        val = out(reg) val,
        options(nostack, preserves_flags)
    );
    val
}

/// Write a 32-bit value to an MMIO address
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn mmio_write32(addr: u64, val: u32) {
    core::arch::asm!(
        "str {val:w}, [{addr}]",
        addr = in(reg) addr,
        val = in(reg) val,
        options(nostack, preserves_flags)
    );
}

/// Read an 8-bit value from an MMIO address
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn mmio_read8(addr: u64) -> u8 {
    let val: u32;
    core::arch::asm!(
        "ldrb {val:w}, [{addr}]",
        addr = in(reg) addr,
        val = out(reg) val,
        options(nostack, preserves_flags)
    );
    val as u8
}

/// Write an 8-bit value to an MMIO address
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn mmio_write8(addr: u64, val: u8) {
    core::arch::asm!(
        "strb {val:w}, [{addr}]",
        addr = in(reg) addr,
        val = in(reg) val as u32,
        options(nostack, preserves_flags)
    );
}
