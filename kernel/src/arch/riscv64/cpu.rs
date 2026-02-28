//! RISC-V 64 CPU Primitives
//!
//! CSR (Control and Status Register) access and low-level operations.

/// Read the stack pointer (SP / x2)
#[inline(always)]
pub fn read_stack_pointer() -> u64 {
    let sp: u64;
    unsafe {
        core::arch::asm!("mv {}, sp", out(reg) sp, options(nomem, nostack, preserves_flags));
    }
    sp
}

/// Read the frame pointer (S0/FP / x8)
#[inline(always)]
pub fn read_frame_pointer() -> u64 {
    let fp: u64;
    unsafe {
        core::arch::asm!("mv {}, s0", out(reg) fp, options(nomem, nostack, preserves_flags));
    }
    fp
}

/// I/O wait — use fence instruction as a barrier/delay
#[inline(always)]
pub fn io_wait() {
    unsafe {
        core::arch::asm!("fence iorw, iorw", options(nomem, nostack, preserves_flags));
    }
}

/// Software breakpoint (EBREAK)
#[inline(always)]
pub fn breakpoint() {
    unsafe {
        core::arch::asm!("ebreak", options(nomem, nostack));
    }
}

/// Fence (memory barrier)
#[inline(always)]
pub fn fence() {
    unsafe {
        core::arch::asm!("fence iorw, iorw", options(nomem, nostack, preserves_flags));
    }
}

/// Fence.I (instruction barrier)
#[inline(always)]
pub fn fence_i() {
    unsafe {
        core::arch::asm!("fence.i", options(nomem, nostack, preserves_flags));
    }
}

// ============================================================================
// CSR (Control and Status Register) Access
// ============================================================================

/// Read sstatus CSR (Supervisor Status)
#[inline(always)]
pub fn read_sstatus() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, sstatus", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write sstatus CSR
#[inline(always)]
pub unsafe fn write_sstatus(val: u64) {
    core::arch::asm!("csrw sstatus, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read sie CSR (Supervisor Interrupt Enable)
#[inline(always)]
pub fn read_sie() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, sie", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write sie CSR
#[inline(always)]
pub unsafe fn write_sie(val: u64) {
    core::arch::asm!("csrw sie, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read sip CSR (Supervisor Interrupt Pending)
#[inline(always)]
pub fn read_sip() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, sip", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read stvec CSR (Supervisor Trap Vector Base)
#[inline(always)]
pub fn read_stvec() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, stvec", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write stvec CSR
#[inline(always)]
pub unsafe fn write_stvec(val: u64) {
    core::arch::asm!("csrw stvec, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read sepc CSR (Supervisor Exception Program Counter)
#[inline(always)]
pub fn read_sepc() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, sepc", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write sepc CSR
#[inline(always)]
pub unsafe fn write_sepc(val: u64) {
    core::arch::asm!("csrw sepc, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read scause CSR (Supervisor Cause)
#[inline(always)]
pub fn read_scause() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read stval CSR (Supervisor Trap Value — fault address)
#[inline(always)]
pub fn read_stval() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, stval", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read satp CSR (Supervisor Address Translation and Protection)
#[inline(always)]
pub fn read_satp() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, satp", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write satp CSR
#[inline(always)]
pub unsafe fn write_satp(val: u64) {
    core::arch::asm!("csrw satp, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    // Flush pipeline after changing satp
    core::arch::asm!("sfence.vma", options(nomem, nostack, preserves_flags));
}

/// Read sscratch CSR (Supervisor Scratch — kernel stack pointer storage)
#[inline(always)]
pub fn read_sscratch() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, sscratch", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write sscratch CSR
#[inline(always)]
pub unsafe fn write_sscratch(val: u64) {
    core::arch::asm!("csrw sscratch, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read cycle counter (rdcycle)
#[inline(always)]
pub fn rdcycle() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("rdcycle {}", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read time counter (rdtime) — real-time clock
#[inline(always)]
pub fn rdtime() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("rdtime {}", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read hart ID (mhartid) — only accessible in M-mode, use sscratch in S-mode
pub fn hart_id() -> u64 {
    // In S-mode, we typically store hart ID in sscratch or tp
    let tp: u64;
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
pub unsafe fn mmio_read32(addr: u64) -> u32 {
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
pub unsafe fn mmio_write32(addr: u64, val: u32) {
    core::arch::asm!(
        "sw {val}, 0({addr})",
        addr = in(reg) addr,
        val = in(reg) val,
        options(nostack, preserves_flags)
    );
}

/// Read an 8-bit value from an MMIO address
#[inline(always)]
pub unsafe fn mmio_read8(addr: u64) -> u8 {
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
pub unsafe fn mmio_write8(addr: u64, val: u8) {
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
    pub const SIE: u64 = 1 << 1;   // Supervisor Interrupt Enable
    pub const SPIE: u64 = 1 << 5;  // Previous SIE value
    pub const SPP: u64 = 1 << 8;   // Previous privilege (0=User, 1=Supervisor)
    pub const SUM: u64 = 1 << 18;  // Supervisor User Memory access
    pub const MXR: u64 = 1 << 19;  // Make eXecutable Readable
}

/// sie bits (interrupt enable)
pub mod sie_bits {
    pub const SSIE: u64 = 1 << 1;  // Supervisor Software Interrupt Enable
    pub const STIE: u64 = 1 << 5;  // Supervisor Timer Interrupt Enable
    pub const SEIE: u64 = 1 << 9;  // Supervisor External Interrupt Enable
}

/// satp modes
pub mod satp_mode {
    pub const BARE: u64 = 0;
    pub const SV39: u64 = 8 << 60;  // 39-bit virtual address (3-level page tables)
    pub const SV48: u64 = 9 << 60;  // 48-bit virtual address (4-level page tables)
    pub const SV57: u64 = 10 << 60; // 57-bit virtual address (5-level page tables)
}
