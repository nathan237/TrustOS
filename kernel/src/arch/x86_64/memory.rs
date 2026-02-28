//! x86_64 Memory Management Primitives
//!
//! Page table root (CR3), TLB flush, and NXE enable.

use super::cpu;

/// Flush a single TLB entry (INVLPG)
#[inline(always)]
pub fn flush_tlb(addr: u64) {
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
    }
}

/// Flush the entire TLB by reloading CR3
#[inline(always)]
pub fn flush_tlb_all() {
    unsafe {
        let cr3 = cpu::read_cr3();
        cpu::write_cr3(cr3);
    }
}

/// Read the page table root (CR3)
#[inline(always)]
pub fn read_page_table_root() -> u64 {
    unsafe { cpu::read_cr3() }
}

/// Write the page table root (CR3)
#[inline(always)]
pub fn write_page_table_root(val: u64) {
    unsafe { cpu::write_cr3(val); }
}

/// Enable the NX (No-Execute) bit via EFER MSR
pub fn enable_nx() {
    unsafe {
        let efer = cpu::rdmsr(cpu::msr::IA32_EFER);
        cpu::wrmsr(cpu::msr::IA32_EFER, efer | cpu::msr::EFER_NXE);
    }
}

/// Check if NX bit is enabled
pub fn is_nx_enabled() -> bool {
    unsafe {
        let efer = cpu::rdmsr(cpu::msr::IA32_EFER);
        efer & cpu::msr::EFER_NXE != 0
    }
}
