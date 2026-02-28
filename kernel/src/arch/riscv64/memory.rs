//! RISC-V 64 Memory Management Primitives
//!
//! Uses satp CSR for page table base and sfence.vma for TLB flush.
//! Supports Sv39 (3-level), Sv48 (4-level), and Sv57 (5-level) paging.

use super::cpu;

/// Flush a single TLB entry (sfence.vma with specific address)
#[inline(always)]
pub fn flush_tlb(addr: u64) {
    unsafe {
        core::arch::asm!(
            "sfence.vma {}, zero",
            in(reg) addr,
            options(nostack, preserves_flags)
        );
    }
}

/// Flush the entire TLB (sfence.vma with no arguments)
#[inline(always)]
pub fn flush_tlb_all() {
    unsafe {
        core::arch::asm!("sfence.vma", options(nomem, nostack, preserves_flags));
    }
}

/// Read the page table root (satp CSR)
///
/// satp format (Sv48):
/// - Bits 63:60 = Mode (0=Bare, 8=Sv39, 9=Sv48)
/// - Bits 59:44 = ASID
/// - Bits 43:0  = PPN (Physical Page Number)
#[inline(always)]
pub fn read_page_table_root() -> u64 {
    cpu::read_satp()
}

/// Write the page table root (satp CSR)
#[inline(always)]
pub fn write_page_table_root(val: u64) {
    unsafe { cpu::write_satp(val); }
}

/// Build a satp value for Sv48 mode
pub fn make_satp_sv48(root_ppn: u64, asid: u16) -> u64 {
    cpu::satp_mode::SV48 | ((asid as u64) << 44) | root_ppn
}

/// Build a satp value for Sv39 mode
pub fn make_satp_sv39(root_ppn: u64, asid: u16) -> u64 {
    cpu::satp_mode::SV39 | ((asid as u64) << 44) | root_ppn
}

/// Extract the PPN from a satp value
pub fn satp_ppn(satp: u64) -> u64 {
    satp & 0x00000FFFFFFFFFFF // Bits 43:0
}
