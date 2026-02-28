//! x86_64 Interrupt Control
//!
//! Enable, disable, and manage hardware interrupts via the FLAGS register.

/// Enable interrupts (STI)
#[inline(always)]
pub fn enable() {
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
}

/// Disable interrupts (CLI)
#[inline(always)]
pub fn disable() {
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack));
    }
}

/// Check if interrupts are enabled (IF flag in RFLAGS)
#[inline(always)]
pub fn are_enabled() -> bool {
    let flags: u64;
    unsafe {
        core::arch::asm!(
            "pushfq",
            "pop {}",
            out(reg) flags,
            options(nomem, preserves_flags)
        );
    }
    flags & (1 << 9) != 0 // IF flag is bit 9 of RFLAGS
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
