//! Per-CPU lock-free single-producer / single-consumer ring buffer.
//!
//! Producer = the CPU that owns the ring (called from any context, IRQ included).
//! Consumer = the drainer (shell command or future background thread).
//!
//! Single-producer means we don't need a CAS on `head` — a `Relaxed` load +
//! `Release` store is sufficient. The consumer uses `Acquire` to observe a
//! committed event. Overflow is non-blocking: the event is dropped and a
//! per-ring counter is incremented.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU64, Ordering};

use super::event::TraceEvent;

pub const RING_CAPACITY:    usize = 2048;
pub const RING_MASK:        u64   = (RING_CAPACITY as u64) - 1;
pub const MAX_TRACE_CPUS:   usize = 16;

/// Cache-line aligned ring; head and tail live on separate cache lines to
/// avoid false sharing between producer and consumer.
#[repr(C, align(64))]
pub struct PerCpuRing {
    pub head:    AtomicU64,
    _pad1:       [u8; 56],
    pub tail:    AtomicU64,
    _pad2:       [u8; 56],
    pub dropped: AtomicU64,
    pub written: AtomicU64,
    pub buffer:  UnsafeCell<[TraceEvent; RING_CAPACITY]>,
}

unsafe impl Sync for PerCpuRing {}

impl PerCpuRing {
    pub const fn new() -> Self {
        Self {
            head:    AtomicU64::new(0),
            _pad1:   [0; 56],
            tail:    AtomicU64::new(0),
            _pad2:   [0; 56],
            dropped: AtomicU64::new(0),
            written: AtomicU64::new(0),
            buffer:  UnsafeCell::new([TraceEvent::ZERO; RING_CAPACITY]),
        }
    }

    /// Push an event. Single-producer per ring — caller MUST be the CPU that
    /// owns this ring (or interrupts/preemption disabled while writing).
    /// Returns `true` if stored, `false` if dropped due to overflow.
    #[inline(always)]
    pub fn push(&self, ev: TraceEvent) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head.wrapping_sub(tail) >= RING_CAPACITY as u64 {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        unsafe {
            let slot = (*self.buffer.get()).as_mut_ptr().add((head & RING_MASK) as usize);
            core::ptr::write_volatile(slot, ev);
        }
        // Release: the event store is visible BEFORE head moves.
        self.head.store(head.wrapping_add(1), Ordering::Release);
        self.written.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// Drain up to `out.len()` events into `out`. Returns count drained.
    pub fn drain(&self, out: &mut [TraceEvent]) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let mut tail = self.tail.load(Ordering::Relaxed);
        let mut n = 0usize;
        while tail != head && n < out.len() {
            unsafe {
                let slot = (*self.buffer.get()).as_ptr().add((tail & RING_MASK) as usize);
                out[n] = core::ptr::read_volatile(slot);
            }
            tail = tail.wrapping_add(1);
            n += 1;
        }
        self.tail.store(tail, Ordering::Release);
        n
    }

    pub fn len(&self) -> u64 {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }
}

/// Static array of per-CPU rings.
#[allow(clippy::declare_interior_mutable_const)]
const RING_INIT: PerCpuRing = PerCpuRing::new();
pub static RINGS: [PerCpuRing; MAX_TRACE_CPUS] = [RING_INIT; MAX_TRACE_CPUS];
