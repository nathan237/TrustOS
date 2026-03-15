//! x86_64 Timer (TSC + PIT)
//!
//! Timestamp counter and programmable interval timer.

use super::cpu;

/// Read the Time Stamp Counter (RDTSC)
#[inline(always)]
pub fn timestamp() -> u64 {
    cpu::rdtsc()
}

/// PIT (Programmable Interval Timer) I/O ports
const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const PIT_FREQUENCY: u32 = 1_193_182; // Hz

/// Set PIT frequency for one-shot or periodic mode
pub fn pit_set_frequency(hz: u32) {
    let divisor = if hz == 0 { 0xFFFF } else { PIT_FREQUENCY / hz };
    let divisor = divisor.min(0xFFFF) as u16;
    
    unsafe {
        // Channel 0, lobyte/hibyte, rate generator (mode 2)
        cpu::outb(PIT_COMMAND, 0x34);
        cpu::outb(PIT_CHANNEL0, (divisor & 0xFF) as u8);
        cpu::outb(PIT_CHANNEL0, (divisor >> 8) as u8);
    }
}

/// Read PIT current count (for calibration)
pub fn pit_read_count() -> u16 {
    unsafe {
        cpu::outb(PIT_COMMAND, 0x00); // Latch channel 0
        let lo = cpu::inb(PIT_CHANNEL0) as u16;
        let hi = cpu::inb(PIT_CHANNEL0) as u16;
        (hi << 8) | lo
    }
}
