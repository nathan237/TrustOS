//! x86_64 Memory Management Primitives
//!
//! Page table root (CR3), TLB flush, and NXE enable.

use super::cpu;

/// Flush a single TLB entry (INVLPG)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn flush_tlb(addr: u64) {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
    }
}

/// Flush the entire TLB by reloading CR3
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn flush_tlb_all() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let cr3 = cpu::read_cr3();
        cpu::write_cr3(cr3);
    }
}

/// Read the page table root (CR3)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn read_page_table_root() -> u64 {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { cpu::read_cr3() }
}

/// Write the page table root (CR3)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn write_page_table_root(val: u64) {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { cpu::write_cr3(val); }
}

/// Enable the NX (No-Execute) bit via EFER MSR
pub fn enable_nx() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let efer = cpu::rdmsr(cpu::msr::IA32_EFER);
        cpu::wrmsr(cpu::msr::IA32_EFER, efer | cpu::msr::EFER_NXE);
    }
}

/// Check if NX bit is enabled
pub fn is_nx_enabled() -> bool {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let efer = cpu::rdmsr(cpu::msr::IA32_EFER);
        efer & cpu::msr::EFER_NXE != 0
    }
}
