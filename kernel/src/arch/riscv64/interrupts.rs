//! RISC-V 64 Interrupt Control
//!
//! Uses the SIE bit in sstatus CSR to enable/disable supervisor interrupts.

use super::cpu;

/// Enable interrupts (set SIE bit in sstatus)
#[inline(always)]
pub fn enable() {
    unsafe {
        core::arch::asm!("csrsi sstatus, 0x2", options(nomem, nostack, preserves_flags));
    }
}

/// Disable interrupts (clear SIE bit in sstatus)
#[inline(always)]
pub fn disable() {
    unsafe {
        core::arch::asm!("csrci sstatus, 0x2", options(nomem, nostack, preserves_flags));
    }
}

/// Check if interrupts are enabled (SIE bit in sstatus)
#[inline(always)]
pub fn are_enabled() -> bool {
    let sstatus = cpu::read_sstatus();
    sstatus & cpu::sstatus::SIE != 0
}

/// Run a closure with interrupts disabled, restoring previous state after
#[inline(always)]
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let were_enabled = are_enabled();
    if were_enabled {
        disable();
    }
    let result = f();
    if were_enabled {
        enable();
    }
    result
}
