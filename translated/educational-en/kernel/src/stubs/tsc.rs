//! CPU TSC stub for non-x86_64 architectures
//! Provides timing functions using the architecture's timer.

static mut FREQUENCY: u64 = 1_000_000_000;

// Public structure — visible outside this module.
pub struct Stopwatch {
    start: u64,
}

// Implementation block — defines methods for the type above.
impl Stopwatch {
        // Public function — callable from other modules.
pub fn start() -> Self {
        Self { start: crate::arch::timestamp() }
    }
        // Public function — callable from other modules.
pub fn elapsed_nanos(&self) -> u64 {
        let elapsed = crate::arch::timestamp().wrapping_sub(self.start);
        cycles_to_nanos(elapsed)
    }
        // Public function — callable from other modules.
pub fn elapsed_micros(&self) -> u64 { self.elapsed_nanos() / 1_000 }
        // Public function — callable from other modules.
pub fn elapsed_millis(&self) -> u64 { self.elapsed_nanos() / 1_000_000 }
        // Public function — callable from other modules.
pub fn elapsed_cycles(&self) -> u64 { crate::arch::timestamp().wrapping_sub(self.start) }
        // Public function — callable from other modules.
pub fn lap_nanos(&mut self) -> u64 {
        let now = crate::arch::timestamp();
        let elapsed = cycles_to_nanos(now.wrapping_sub(self.start));
        self.start = now;
        elapsed
    }
}

// Public function — callable from other modules.
pub fn init(frequency_hz: u64) {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { FREQUENCY = frequency_hz; }
}

// Public function — callable from other modules.
pub fn read_tsc() -> u64 { crate::arch::timestamp() }
// Public function — callable from other modules.
pub fn read_tsc_serialized() -> u64 { crate::arch::timestamp() }
// Public function — callable from other modules.
pub fn read_tscp() -> (u64, u32) { (crate::arch::timestamp(), 0) }
// Public function — callable from other modules.
pub fn frequency_hz() -> u64 { // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { FREQUENCY } }

// Public function — callable from other modules.
pub fn cycles_to_nanos(cycles: u64) -> u64 {
    let freq = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { FREQUENCY };
    if freq == 0 { return 0; }
    (cycles as u128 * 1_000_000_000 / freq as u128) as u64
}
// Public function — callable from other modules.
pub fn cycles_to_micros(cycles: u64) -> u64 { cycles_to_nanos(cycles) / 1_000 }
// Public function — callable from other modules.
pub fn cycles_to_millis(cycles: u64) -> u64 { cycles_to_nanos(cycles) / 1_000_000 }

// Public function — callable from other modules.
pub fn now_nanos() -> u64 { cycles_to_nanos(crate::arch::timestamp()) }
// Public function — callable from other modules.
pub fn now_micros() -> u64 { now_nanos() / 1_000 }
// Public function — callable from other modules.
pub fn now_millis() -> u64 { now_nanos() / 1_000_000 }

// Public function — callable from other modules.
pub fn delay_nanos(nanos: u64) {
    let start = now_nanos();
    while now_nanos().wrapping_sub(start) < nanos {
        core::hint::spin_loop();
    }
}
// Public function — callable from other modules.
pub fn delay_micros(micros: u64) { delay_nanos(micros * 1_000); }
// Public function — callable from other modules.
pub fn delay_millis(millis: u64) { delay_nanos(millis * 1_000_000); }
// Public function — callable from other modules.
pub fn pit_delay_mouse(millis: u64) { delay_millis(millis); }
// Public function — callable from other modules.
pub fn calibrate_tsc() -> u64 { // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { FREQUENCY } }
