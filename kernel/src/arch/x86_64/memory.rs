//! x86_64 Memory Management Primitives
//!
//! Page table root (CR3), TLB flush, NXE enable, optional PCID support.

use super::cpu;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Tracks whether CR4.PCIDE has been enabled (set by `enable_pcid()`).
static PCID_ENABLED: AtomicBool = AtomicBool::new(false);

/// TLB shootdown statistics: number of remote-CPU invalidations skipped
/// thanks to PCID (when PCID is on we keep stale entries tagged so a
/// process switch back doesn't pay full TLB refill cost).
static TLB_LOCAL_FLUSHES: AtomicU64 = AtomicU64::new(0);
static TLB_FULL_FLUSHES: AtomicU64 = AtomicU64::new(0);

/// Enable PCID (Process-Context Identifiers) if the CPU supports it.
/// Returns `true` if PCID is now active.
///
/// Must be called *after* paging is on (CR0.PG=1) and *before* any
/// CR3 write that uses the PCID bit.
pub fn enable_pcid() -> bool {
    let caps = match crate::cpu::capabilities() {
        Some(c) => c,
        None => return false,
    };
    if !caps.pcid {
        return false;
    }
    unsafe {
        let mut cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack));
        // Set CR4.PCIDE (bit 17). Must not be cleared once set without first
        // clearing CR3.PCID to 0, so we do this exactly once at boot.
        cr4 |= 1 << 17;
        core::arch::asm!("mov cr4, {}", in(reg) cr4, options(nomem, nostack));
    }
    PCID_ENABLED.store(true, Ordering::Release);
    crate::serial_println!("[MM] PCID enabled (CR4.PCIDE=1)");
    true
}

/// Flush a single TLB entry (INVLPG)
#[inline(always)]
pub fn flush_tlb(addr: u64) {
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
    }
    TLB_LOCAL_FLUSHES.fetch_add(1, Ordering::Relaxed);
}

/// Flush the entire TLB.
/// With PCID enabled, this preserves global pages (kernel mappings stay hot).
#[inline(always)]
pub fn flush_tlb_all() {
    unsafe {
        let cr3 = cpu::read_cr3();
        // Writing CR3 with PCID bit 63=0 only flushes the current PCID's
        // non-global entries; global pages survive (CR4.PGE behavior).
        cpu::write_cr3(cr3);
    }
    TLB_FULL_FLUSHES.fetch_add(1, Ordering::Relaxed);
}

/// Get TLB flush counters: (local INVLPG, full CR3 reloads).
pub fn tlb_stats() -> (u64, u64) {
    (TLB_LOCAL_FLUSHES.load(Ordering::Relaxed),
     TLB_FULL_FLUSHES.load(Ordering::Relaxed))
}

/// Returns `true` if PCID is currently enabled on this CPU.
#[inline(always)]
pub fn pcid_enabled() -> bool {
    PCID_ENABLED.load(Ordering::Relaxed)
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
