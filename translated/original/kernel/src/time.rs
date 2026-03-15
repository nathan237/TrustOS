//! Time utilities

use core::sync::atomic::{AtomicU64, Ordering};
use alloc::vec::Vec;
use spin::Mutex;

/// System uptime in milliseconds
static UPTIME_MS: AtomicU64 = AtomicU64::new(0);

/// Pending timed wake-ups: (deadline_ns, thread_tid)
static WAKEUP_QUEUE: Mutex<Vec<(u64, u64)>> = Mutex::new(Vec::new());

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

    // Check and fire expired wakeups
    process_wakeups();
}

/// Register a timed wakeup for a thread.
/// When `now_ns() >= deadline_ns`, the thread will be woken.
pub fn register_wakeup(tid: u64, deadline_ns: u64) {
    let mut q = WAKEUP_QUEUE.lock();
    q.push((deadline_ns, tid));
}

/// Process expired wakeups — called on every timer tick.
fn process_wakeups() {
    let now = now_ns();
    let mut q = WAKEUP_QUEUE.lock();
    // Drain expired entries and wake threads
    let mut i = 0;
    while i < q.len() {
        if now >= q[i].0 {
            let tid = q[i].1;
            q.swap_remove(i);
            // Wake the thread (it may have already been woken by futex_wake, etc.;
            // wake() is a no-op if the thread isn't Blocked).
            crate::thread::wake(tid);
        } else {
            i += 1;
        }
    }
}

/// Get current time in seconds
pub fn uptime_secs() -> u64 {
    uptime_ms() / 1000
}