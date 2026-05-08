//! RISC-V 64 CPU Primitives
//!
//! CSR (Control and Status Register) access and low-level operations.

/// Read the stack pointer (SP / x2)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_stack_pointer() -> u64 {
    let sp: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("mv {}, sp", out(reg) sp, options(nomem, nostack, preserves_flags));
    }
    sp
}

/// Read the frame pointer (S0/FP / x8)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_frame_pointer() -> u64 {
    let fp: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("mv {}, s0", out(reg) fp, options(nomem, nostack, preserves_flags));
    }
    fp
}

/// I/O wait — use fence instruction as a barrier/delay
#[inline(always)]
// Public function — callable from other modules.
pub fn io_wait() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("fence iorw, iorw", options(nomem, nostack, preserves_flags));
    }
}

/// Software breakpoint (EBREAK)
#[inline(always)]
// Public function — callable from other modules.
pub fn breakpoint() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("ebreak", options(nomem, nostack));
    }
}

/// Fence (memory barrier)
#[inline(always)]
// Public function — callable from other modules.
pub fn fence() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("fence iorw, iorw", options(nomem, nostack, preserves_flags));
    }
}

/// Fence.I (instruction barrier)
#[inline(always)]
// Public function — callable from other modules.
pub fn fence_i() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("fence.i", options(nomem, nostack, preserves_flags));
    }
}

// ============================================================================
// CSR (Control and Status Register) Access
// ============================================================================

/// Read sstatus CSR (Supervisor Status)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_sstatus() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("csrr {}, sstatus", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write sstatus CSR
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_sstatus(val: u64) {
    core::arch::asm!("csrw sstatus, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read sie CSR (Supervisor Interrupt Enable)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_sie() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("csrr {}, sie", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write sie CSR
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_sie(val: u64) {
    core::arch::asm!("csrw sie, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read sip CSR (Supervisor Interrupt Pending)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_sip() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("csrr {}, sip", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read stvec CSR (Supervisor Trap Vector Base)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_stvec() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("csrr {}, stvec", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write stvec CSR
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_stvec(val: u64) {
    core::arch::asm!("csrw stvec, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read sepc CSR (Supervisor Exception Program Counter)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_sepc() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("csrr {}, sepc", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write sepc CSR
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_sepc(val: u64) {
    core::arch::asm!("csrw sepc, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read scause CSR (Supervisor Cause)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_scause() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read stval CSR (Supervisor Trap Value — fault address)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_stval() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("csrr {}, stval", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read satp CSR (Supervisor Address Translation and Protection)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_satp() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("csrr {}, satp", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write satp CSR
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_satp(val: u64) {
    core::arch::asm!("csrw satp, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    // Flush pipeline after changing satp
    core::arch::asm!("sfence.vma", options(nomem, nostack, preserves_flags));
}

/// Read sscratch CSR (Supervisor Scratch — kernel stack pointer storage)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_sscratch() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("csrr {}, sscratch", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write sscratch CSR
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_sscratch(val: u64) {
    core::arch::asm!("csrw sscratch, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read cycle counter (rdcycle)
#[inline(always)]
// Public function — callable from other modules.
pub fn rdcycle() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("rdcycle {}", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read time counter (rdtime) — real-time clock
#[inline(always)]
// Public function — callable from other modules.
pub fn rdtime() -> u64 {
    let val: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("rdtime {}", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read hart ID (mhartid) — only accessible in M-mode, use sscratch in S-mode
pub fn hart_id() -> u64 {
    // In S-mode, we typically store hart ID in sscratch or tp
    let tp: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
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
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn mmio_read32(addr: u64) -> u32 {
    let val: u32;
    core::arch::asm!(
        "lw {val}, 0({addr})",
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
        "sw {val}, 0({addr})",
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
        "lbu {val}, 0({addr})",
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
        "sb {val}, 0({addr})",
        addr = in(reg) addr,
        val = in(reg) val,
        options(nostack, preserves_flags)
    );
}

// ============================================================================
// CSR bit constants
// ============================================================================

/// sstatus bits
pub mod sstatus {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SIE: u64 = 1 << 1;   // Supervisor Interrupt Enable
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SPIE: u64 = 1 << 5;  // Previous SIE value
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SPP: u64 = 1 << 8;   // Previous privilege (0=User, 1=Supervisor)
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SUM: u64 = 1 << 18;  // Supervisor User Memory access
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const MXR: u64 = 1 << 19;  // Make eXecutable Readable
}

/// sie bits (interrupt enable)
pub mod sie_bits {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SSIE: u64 = 1 << 1;  // Supervisor Software Interrupt Enable
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const STIE: u64 = 1 << 5;  // Supervisor Timer Interrupt Enable
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SEIE: u64 = 1 << 9;  // Supervisor External Interrupt Enable
}

/// satp modes
pub mod satp_mode {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const BARE: u64 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SV39: u64 = 8 << 60;  // 39-bit virtual address (3-level page tables)
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SV48: u64 = 9 << 60;  // 48-bit virtual address (4-level page tables)
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SV57: u64 = 10 << 60; // 57-bit virtual address (5-level page tables)
}
