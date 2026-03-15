//! aarch64 CPU Primitives
//!
//! ARM system registers, CPU identification, and low-level operations.

/// Read the stack pointer (SP)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_stack_pointer() -> u64 {
    let sp: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("mov {}, sp", out(reg) sp, options(nomem, nostack, preserves_flags));
    }
    sp
}

/// Read the frame pointer (X29 / FP)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_frame_pointer() -> u64 {
    let fp: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("mov {}, x29", out(reg) fp, options(nomem, nostack, preserves_flags));
    }
    fp
}

/// Read DAIF (interrupt mask flags)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_daif() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("mrs {}, DAIF", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// I/O wait — use ISB (instruction synchronization barrier) as delay
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn io_wait() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("isb", options(nomem, nostack, preserves_flags));
    }
}

/// Software breakpoint (BRK #0)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn breakpoint() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("brk #0", options(nomem, nostack));
    }
}

/// Data Synchronization Barrier (DSB SY)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn dsb_sy() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("dsb sy", options(nomem, nostack, preserves_flags));
    }
}

/// Instruction Synchronization Barrier (ISB)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn isb() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("isb", options(nomem, nostack, preserves_flags));
    }
}

/// Data Memory Barrier (DMB SY)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn dmb_sy() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("dmb sy", options(nomem, nostack, preserves_flags));
    }
}

/// Read MPIDR_EL1 (Multiprocessor Affinity Register — CPU ID)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_mpidr() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("mrs {}, MPIDR_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Read MIDR_EL1 (Main ID Register — CPU model)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_midr() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("mrs {}, MIDR_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Read current Exception Level (from CurrentEL register)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn current_el() -> u8 {
    let el: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("mrs {}, CurrentEL", out(reg) el, options(nomem, nostack, preserves_flags));
    }
    ((el >> 2) & 0x3) as u8
}

/// Read SCTLR_EL1 (System Control Register)
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn read_sctlr_el1() -> u64 {
    let value: u64;
    core::arch::asm!("mrs {}, SCTLR_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

/// Write SCTLR_EL1
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn write_sctlr_el1(value: u64) {
    core::arch::asm!("msr SCTLR_EL1, {}", in(reg) value, options(nomem, nostack, preserves_flags));
    isb();
}

/// Read TCR_EL1 (Translation Control Register)
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn read_tcr_el1() -> u64 {
    let value: u64;
    core::arch::asm!("mrs {}, TCR_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

/// Write TCR_EL1
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn write_tcr_el1(value: u64) {
    core::arch::asm!("msr TCR_EL1, {}", in(reg) value, options(nomem, nostack, preserves_flags));
    isb();
}

/// Read VBAR_EL1 (Vector Base Address Register)
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn read_vbar_el1() -> u64 {
    let value: u64;
    core::arch::asm!("mrs {}, VBAR_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

/// Write VBAR_EL1
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn write_vbar_el1(value: u64) {
    core::arch::asm!("msr VBAR_EL1, {}", in(reg) value, options(nomem, nostack, preserves_flags));
    isb();
}

/// Read ESR_EL1 (Exception Syndrome Register — fault info)
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn read_esr_el1() -> u64 {
    let value: u64;
    core::arch::asm!("mrs {}, ESR_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

/// Read FAR_EL1 (Fault Address Register)
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn read_far_el1() -> u64 {
    let value: u64;
    core::arch::asm!("mrs {}, FAR_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

/// Read ELR_EL1 (Exception Link Register — return address)
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn read_elr_el1() -> u64 {
    let value: u64;
    core::arch::asm!("mrs {}, ELR_EL1", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

/// Read SP_EL0 (User stack pointer)
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn read_sp_el0() -> u64 {
    let value: u64;
    core::arch::asm!("mrs {}, SP_EL0", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

/// Write SP_EL0
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn write_sp_el0(value: u64) {
    core::arch::asm!("msr SP_EL0, {}", in(reg) value, options(nomem, nostack, preserves_flags));
}

// ============================================================================
// MMIO (Memory-Mapped I/O) — ARM has no I/O ports, everything is MMIO
// ============================================================================

/// Read a 32-bit value from an MMIO address
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn mmio_read32(address: u64) -> u32 {
    let value: u32;
    core::arch::asm!(
        "ldr {val:w}, [{addr}]",
        address = in(reg) address,
        value = out(reg) value,
        options(nostack, preserves_flags)
    );
    value
}

/// Write a 32-bit value to an MMIO address
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn mmio_write32(address: u64, value: u32) {
    core::arch::asm!(
        "str {val:w}, [{addr}]",
        address = in(reg) address,
        value = in(reg) value,
        options(nostack, preserves_flags)
    );
}

/// Read an 8-bit value from an MMIO address
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn mmio_read8(address: u64) -> u8 {
    let value: u32;
    core::arch::asm!(
        "ldrb {val:w}, [{addr}]",
        address = in(reg) address,
        value = out(reg) value,
        options(nostack, preserves_flags)
    );
    value as u8
}

/// Write an 8-bit value to an MMIO address
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn mmio_write8(address: u64, value: u8) {
    core::arch::asm!(
        "strb {val:w}, [{addr}]",
        address = in(reg) address,
        value = in(reg) value as u32,
        options(nostack, preserves_flags)
    );
}
