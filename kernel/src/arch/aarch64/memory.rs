//! aarch64 Memory Management Primitives
//!
//! ARM64 uses TTBR0_EL1/TTBR1_EL1 for page tables and TLBI for TLB flush.
//! TTBR0 = user space (lower VA range), TTBR1 = kernel space (upper VA range).

use super::cpu;

/// Flush a single TLB entry by virtual address (TLBI VAE1IS)
#[inline(always)]
pub fn flush_tlb(addr: u64) {
    unsafe {
        // TLBI VAE1IS — invalidate by VA, EL1, inner shareable
        // The address must be shifted right by 12 (page-aligned)
        let va = addr >> 12;
        core::arch::asm!(
            "tlbi vae1is, {}",
            "dsb ish",
            "isb",
            in(reg) va,
            options(nostack, preserves_flags)
        );
    }
}

/// Flush the entire TLB (TLBI VMALLE1IS)
#[inline(always)]
pub fn flush_tlb_all() {
    unsafe {
        core::arch::asm!(
            "tlbi vmalle1is",
            "dsb ish",
            "isb",
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Read the page table root — TTBR0_EL1 (user space page table base)
#[inline(always)]
pub fn read_page_table_root() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, TTBR0_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write the page table root — TTBR0_EL1
#[inline(always)]
pub fn write_page_table_root(val: u64) {
    unsafe {
        core::arch::asm!(
            "msr TTBR0_EL1, {}",
            "isb",
            in(reg) val,
            options(nostack, preserves_flags)
        );
    }
}

/// Read the kernel page table root — TTBR1_EL1
#[inline(always)]
pub fn read_kernel_page_table() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, TTBR1_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Write the kernel page table root — TTBR1_EL1
#[inline(always)]
pub fn write_kernel_page_table(val: u64) {
    unsafe {
        core::arch::asm!(
            "msr TTBR1_EL1, {}",
            "isb",
            in(reg) val,
            options(nostack, preserves_flags)
        );
    }
}

/// Enable the MMU and caches via SCTLR_EL1
pub fn enable_mmu() {
    unsafe {
        let mut sctlr = cpu::read_sctlr_el1();
        sctlr |= 1 << 0;  // M bit — enable MMU
        sctlr |= 1 << 2;  // C bit — enable data cache
        sctlr |= 1 << 12; // I bit — enable instruction cache
        cpu::write_sctlr_el1(sctlr);
    }
}

/// Check if MMU is enabled
pub fn is_mmu_enabled() -> bool {
    unsafe {
        let sctlr = cpu::read_sctlr_el1();
        sctlr & 1 != 0
    }
}
