//! aarch64 Timer (ARM Generic Timer)
//!
//! Uses the ARM Generic Timer (CNTPCT_EL0) for timestamps and CNTP_* for scheduling.

/// Read the physical counter (CNTPCT_EL0) — monotonic timestamp
#[inline(always)]
pub fn timestamp() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, CNTPCT_EL0", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Read the counter frequency (CNTFRQ_EL0) — ticks per second
#[inline(always)]
pub fn frequency() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, CNTFRQ_EL0", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

/// Set the physical timer compare value (CNTP_CVAL_EL0)
pub fn set_timer_compare(value: u64) {
    unsafe {
        core::arch::asm!(
            "msr CNTP_CVAL_EL0, {}",
            in(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Enable the physical timer (CNTP_CTL_EL0)
pub fn enable_timer() {
    unsafe {
        core::arch::asm!(
            "msr CNTP_CTL_EL0, {}",
            in(reg) 1u64, // ENABLE bit
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Disable the physical timer
pub fn disable_timer() {
    unsafe {
        core::arch::asm!(
            "msr CNTP_CTL_EL0, {}",
            in(reg) 0u64,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Set a one-shot timer to fire after `ticks` counter ticks
pub fn set_oneshot(ticks: u64) {
    let current = timestamp();
    set_timer_compare(current + ticks);
    enable_timer();
}

/// Set a one-shot timer to fire after `us` microseconds
pub fn set_oneshot_us(us: u64) {
    let freq = frequency();
    let ticks = (us * freq) / 1_000_000;
    set_oneshot(ticks);
}
