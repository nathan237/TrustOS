//! Time utilities

use core::sync::atomic::{AtomicU64, Ordering};

/// System uptime in milliseconds
static UPTIME_MS: AtomicU64 = AtomicU64::new(0);

/// Initialize time system
pub fn init() {
    // Time is updated by timer interrupts
}

/// Get current uptime in milliseconds
pub fn uptime_ms() -> u64 {
    UPTIME_MS.load(Ordering::Relaxed)
}

/// Get current time in nanoseconds
pub fn now_ns() -> u64 {
    uptime_ms() * 1_000_000
}

/// Get current uptime in ticks (same as ms for compatibility)
pub fn uptime_ticks() -> u64 {
    UPTIME_MS.load(Ordering::Relaxed)
}

/// Update uptime (called by timer interrupt)
pub fn tick() {
    UPTIME_MS.fetch_add(10, Ordering::Relaxed); // 10ms per tick
}

/// Get current time in seconds
pub fn uptime_secs() -> u64 {
    uptime_ms() / 1000
}