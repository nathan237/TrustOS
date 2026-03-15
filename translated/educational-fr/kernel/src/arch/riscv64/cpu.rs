//! RISC-V 64 CPU Primitives
//!
//! CSR (Control and Status Register) access and low-level operations.

/// Read the stack pointer (SP / x2)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_stack_pointer() -> u64 {
    let sp: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("mv {}, sp", out(reg) sp, options(nomem, nostack, preserves_flags));
    }
    sp
}

/// Read the frame pointer (S0/FP / x8)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_frame_pointer() -> u64 {
    let fp: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("mv {}, s0", out(reg) fp, options(nomem, nostack, preserves_flags));
    }
    fp
}

/// I/O wait — use fence instruction as a barrier/delay
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn io_wait() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("fence iorw, iorw", options(nomem, nostack, preserves_flags));
    }
}

/// Software breakpoint (EBREAK)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn breakpoint() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("ebreak", options(nomem, nostack));
    }
}

/// Fence (memory barrier)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn fence() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("fence iorw, iorw", options(nomem, nostack, preserves_flags));
    }
}

/// Fence.I (instruction barrier)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn fence_i() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("fence.i", options(nomem, nostack, preserves_flags));
    }
}

// ============================================================================
// CSR (Control and Status Register) Access
// ============================================================================

/// Read sstatus CSR (Supervisor Status)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_sstatus() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("csrr {}, sstatus", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Write sstatus CSR
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn write_sstatus(value: u64) {
    core::arch::asm!("csrw sstatus, {}", in(reg) value, options(nomem, nostack, preserves_flags));
}

/// Read sie CSR (Supervisor Interrupt Enable)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_sie() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("csrr {}, sie", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Write sie CSR
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn write_sie(value: u64) {
    core::arch::asm!("csrw sie, {}", in(reg) value, options(nomem, nostack, preserves_flags));
}

/// Read sip CSR (Supervisor Interrupt Pending)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_sip() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("csrr {}, sip", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Read stvec CSR (Supervisor Trap Vector Base)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_stvec() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("csrr {}, stvec", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Write stvec CSR
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn write_stvec(value: u64) {
    core::arch::asm!("csrw stvec, {}", in(reg) value, options(nomem, nostack, preserves_flags));
}

/// Read sepc CSR (Supervisor Exception Program Counter)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_sepc() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("csrr {}, sepc", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Write sepc CSR
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn write_sepc(value: u64) {
    core::arch::asm!("csrw sepc, {}", in(reg) value, options(nomem, nostack, preserves_flags));
}

/// Read scause CSR (Supervisor Cause)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_scause() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Read stval CSR (Supervisor Trap Value — fault address)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_stval() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("csrr {}, stval", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Read satp CSR (Supervisor Address Translation and Protection)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_satp() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("csrr {}, satp", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Write satp CSR
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn write_satp(value: u64) {
    core::arch::asm!("csrw satp, {}", in(reg) value, options(nomem, nostack, preserves_flags));
    // Flush pipeline after changing satp
    core::arch::asm!("sfence.vma", options(nomem, nostack, preserves_flags));
}

/// Read sscratch CSR (Supervisor Scratch — kernel stack pointer storage)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_sscratch() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("csrr {}, sscratch", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Write sscratch CSR
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn write_sscratch(value: u64) {
    core::arch::asm!("csrw sscratch, {}", in(reg) value, options(nomem, nostack, preserves_flags));
}

/// Read cycle counter (rdcycle)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn rdcycle() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("rdcycle {}", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Read time counter (rdtime) — real-time clock
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn rdtime() -> u64 {
    let value: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("rdtime {}", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Read hart ID (mhartid) — only accessible in M-mode, use sscratch in S-mode
pub fn hart_id() -> u64 {
    // In S-mode, we typically store hart ID in sscratch or tp
    let tp: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("mv {}, tp", out(reg) tp, options(nomem, nostack, preserves_flags));
    }
    tp
}

// ============================================================================
// MMIO (Memory-Mapped I/O) — RISC-V uses MMIO for all device access
// ============================================================================

/// Read a 32-bit value from an MMIO address
#[inline(always)]
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn mmio_read32(address: u64) -> u32 {
    let value: u32;
    core::arch::asm!(
        "lw {val}, 0({addr})",
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
        "sw {val}, 0({addr})",
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
        "lbu {val}, 0({addr})",
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
        "sb {val}, 0({addr})",
        address = in(reg) address,
        value = in(reg) value,
        options(nostack, preserves_flags)
    );
}

// ============================================================================
// CSR bit constants
// ============================================================================

/// sstatus bits
pub mod sstatus {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SIE: u64 = 1 << 1;   // Supervisor Interrupt Enable
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SPIE: u64 = 1 << 5;  // Previous SIE value
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SPP: u64 = 1 << 8;   // Previous privilege (0=User, 1=Supervisor)
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SUM: u64 = 1 << 18;  // Supervisor User Memory access
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MXR: u64 = 1 << 19;  // Make eXecutable Readable
}

/// sie bits (interrupt enable)
pub mod sie_bits {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SSIE: u64 = 1 << 1;  // Supervisor Software Interrupt Enable
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const STIE: u64 = 1 << 5;  // Supervisor Timer Interrupt Enable
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SEIE: u64 = 1 << 9;  // Supervisor External Interrupt Enable
}

/// satp modes
pub mod satp_mode {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BARE: u64 = 0;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SV39: u64 = 8 << 60;  // 39-bit virtual address (3-level page tables)
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SV48: u64 = 9 << 60;  // 48-bit virtual address (4-level page tables)
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SV57: u64 = 10 << 60; // 57-bit virtual address (5-level page tables)
}
