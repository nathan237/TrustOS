//! RISC-V 64 Timer
//!
//! Uses rdtime CSR for timestamps and SBI timer for scheduling.

use super::cpu;

/// Read the time counter (rdtime) — monotonic timestamp
#[inline(always)]
pub fn timestamp() -> u64 {
    cpu::rdtime()
}

/// Read cycle counter (rdcycle) — CPU cycles (may not be available in S-mode)
#[inline(always)]
pub fn cycles() -> u64 {
    cpu::rdcycle()
}

/// Set the timer compare value via SBI (Supervisor Binary Interface)
///
/// On RISC-V, timer management goes through SBI ecalls to M-mode firmware.
/// SBI extension ID 0x54494D45 ("TIME"), function 0
pub fn set_timer(stime_value: u64) {
    unsafe {
        // SBI call: ecall with a7=extension_id, a6=function_id, a0=arg
        // Legacy SBI set_timer: extension 0, function 0
        core::arch::asm!(
            "ecall",
            in("a7") 0x54494D45u64, // SBI Timer extension
            in("a6") 0u64,           // set_timer function
            in("a0") stime_value,
            options(nostack)
        );
    }
}

/// Set a one-shot timer to fire after `delta` time ticks
pub fn set_oneshot(delta: u64) {
    let current = timestamp();
    set_timer(current + delta);
}

/// Timer frequency — typically 10 MHz on QEMU virt
/// In practice, read from the device tree (FDT)
pub fn frequency() -> u64 {
    // QEMU virt default: 10 MHz
    // Real hardware: read from device tree /cpus/timebase-frequency
    10_000_000
}
