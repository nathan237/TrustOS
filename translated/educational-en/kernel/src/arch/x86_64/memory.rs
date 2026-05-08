//! x86_64 Memory Management Primitives
//!
//! Page table root (CR3), TLB flush, and NXE enable.

use super::cpu;

/// Flush a single TLB entry (INVLPG)
#[inline(always)]
// Public function — callable from other modules.
pub fn flush_tlb(addr: u64) {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
    }
}

/// Flush the entire TLB by reloading CR3
#[inline(always)]
// Public function — callable from other modules.
pub fn flush_tlb_all() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let cr3 = cpu::read_cr3();
        cpu::write_cr3(cr3);
    }
}

/// Read the page table root (CR3)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_page_table_root() -> u64 {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { cpu::read_cr3() }
}

/// Write the page table root (CR3)
#[inline(always)]
// Public function — callable from other modules.
pub fn write_page_table_root(val: u64) {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { cpu::write_cr3(val); }
}

/// Enable the NX (No-Execute) bit via EFER MSR
pub fn enable_nx() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let efer = cpu::rdmsr(cpu::msr::IA32_EFER);
        cpu::wrmsr(cpu::msr::IA32_EFER, efer | cpu::msr::EFER_NXE);
    }
}

/// Check if NX bit is enabled
pub fn is_nx_enabled() -> bool {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let efer = cpu::rdmsr(cpu::msr::IA32_EFER);
        efer & cpu::msr::EFER_NXE != 0
    }
}
