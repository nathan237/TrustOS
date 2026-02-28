//! aarch64 Interrupt Control
//!
//! Uses DAIF (Debug/Abort/IRQ/FIQ) mask bits in PSTATE.

/// Enable interrupts (clear IRQ mask in DAIF)
#[inline(always)]
pub fn enable() {
    unsafe {
        core::arch::asm!("msr DAIFClr, #0x2", options(nomem, nostack, preserves_flags));
    }
}

/// Disable interrupts (set IRQ mask in DAIF)
#[inline(always)]
pub fn disable() {
    unsafe {
        core::arch::asm!("msr DAIFSet, #0x2", options(nomem, nostack, preserves_flags));
    }
}

/// Check if IRQ interrupts are enabled (DAIF.I bit clear)
#[inline(always)]
pub fn are_enabled() -> bool {
    let daif: u64;
    unsafe {
        core::arch::asm!("mrs {}, DAIF", out(reg) daif, options(nomem, nostack, preserves_flags));
    }
    // DAIF bit 7 (I) = IRQ mask; 0 = enabled, 1 = masked
    daif & (1 << 7) == 0
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
