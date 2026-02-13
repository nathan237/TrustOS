//! TrustLab Event Bus — Zero-cost kernel event broadcasting
//!
//! When LAB_ACTIVE is true, kernel subsystems push events here.
//! The Lab UI reads them for real-time display.
//! When LAB_ACTIVE is false, all calls are no-ops (zero overhead).

extern crate alloc;

use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

/// Maximum events in the ring buffer
const EVENT_RING_SIZE: usize = 512;

/// Event categories (colored differently in UI)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EventCategory {
    Interrupt = 0,
    Scheduler = 1,
    Memory = 2,
    FileSystem = 3,
    Syscall = 4,
    Keyboard = 5,
    Network = 6,
    Security = 7,
    Custom = 8,
}

impl EventCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Interrupt  => "IRQ",
            Self::Scheduler  => "SCHED",
            Self::Memory     => "MEM",
            Self::FileSystem => "VFS",
            Self::Syscall    => "SYS",
            Self::Keyboard   => "KBD",
            Self::Network    => "NET",
            Self::Security   => "SEC",
            Self::Custom     => "USR",
        }
    }
    
    pub fn color(&self) -> u32 {
        match self {
            Self::Interrupt  => 0xFFD18616, // orange
            Self::Scheduler  => 0xFFD29922, // yellow
            Self::Memory     => 0xFF3FB950, // green
            Self::FileSystem => 0xFF79C0FF, // cyan
            Self::Syscall    => 0xFFBC8CFF, // purple
            Self::Keyboard   => 0xFF58A6FF, // blue
            Self::Network    => 0xFF79C0FF, // cyan
            Self::Security   => 0xFFF85149, // red
            Self::Custom     => 0xFFE6EDF3, // white
        }
    }
}

/// A single Lab event
#[derive(Clone)]
pub struct LabEvent {
    /// Timestamp in milliseconds since boot
    pub timestamp_ms: u64,
    /// Category
    pub category: EventCategory,
    /// Short description (e.g. "timer tick", "page fault @ 0x1000")
    pub message: String,
    /// Optional numeric payload
    pub payload: u64,
}

/// Global event ring buffer
static EVENT_RING: Mutex<EventRing> = Mutex::new(EventRing::new());

/// Write index (monotonically increasing)
static WRITE_IDX: AtomicU64 = AtomicU64::new(0);

/// Read index for each consumer (Lab window)
/// Multiple windows can read at their own pace
static TOTAL_EVENTS: AtomicU64 = AtomicU64::new(0);

struct EventRing {
    buffer: [Option<LabEvent>; EVENT_RING_SIZE],
}

// Const-initializable
impl EventRing {
    const fn new() -> Self {
        // Can't use [None; N] for non-Copy, so use a const block trick
        const NONE: Option<LabEvent> = None;
        Self {
            buffer: [NONE; EVENT_RING_SIZE],
        }
    }
}

// LabEvent is not Copy, but we need const NONE above — this works because 
// Option<LabEvent> has a None variant that is trivially constructible.

/// Emit an event (called from kernel subsystems, gated by LAB_ACTIVE)
pub fn emit(category: EventCategory, message: String, payload: u64) {
    if !super::LAB_ACTIVE.load(Ordering::Relaxed) {
        return; // Zero-cost when Lab is inactive
    }
    
    let ts = crate::time::uptime_ms();
    let event = LabEvent {
        timestamp_ms: ts,
        category,
        message,
        payload,
    };
    
    let idx = WRITE_IDX.fetch_add(1, Ordering::Relaxed) as usize;
    let slot = idx % EVENT_RING_SIZE;
    
    if let Some(mut ring) = EVENT_RING.try_lock() {
        ring.buffer[slot] = Some(event);
    }
    TOTAL_EVENTS.fetch_add(1, Ordering::Relaxed);
}

/// Emit with a static string (avoids allocation in hot paths)
#[inline]
pub fn emit_static(category: EventCategory, msg: &'static str, payload: u64) {
    if !super::LAB_ACTIVE.load(Ordering::Relaxed) {
        return;
    }
    emit(category, String::from(msg), payload);
}

/// Read recent events (returns up to `count` most recent events)
pub fn read_recent(count: usize) -> alloc::vec::Vec<LabEvent> {
    let mut result = alloc::vec::Vec::new();
    let total = TOTAL_EVENTS.load(Ordering::Relaxed) as usize;
    if total == 0 {
        return result;
    }
    
    let start = if total > count { total - count } else { 0 };
    let ring = EVENT_RING.lock();
    
    for i in start..total {
        let slot = i % EVENT_RING_SIZE;
        if let Some(ref event) = ring.buffer[slot] {
            result.push(event.clone());
        }
    }
    
    result
}

/// Get total event count
pub fn total_count() -> u64 {
    TOTAL_EVENTS.load(Ordering::Relaxed)
}

/// Read events newer than a given index (for incremental updates)
pub fn read_since(since_idx: u64, max: usize) -> (alloc::vec::Vec<LabEvent>, u64) {
    let total = TOTAL_EVENTS.load(Ordering::Relaxed);
    if total <= since_idx {
        return (alloc::vec::Vec::new(), total);
    }
    
    let mut result = alloc::vec::Vec::new();
    let start = since_idx as usize;
    let end = total as usize;
    let ring = EVENT_RING.lock();
    
    let actual_start = if end - start > max { end - max } else { start };
    for i in actual_start..end {
        let slot = i % EVENT_RING_SIZE;
        if let Some(ref event) = ring.buffer[slot] {
            result.push(event.clone());
        }
    }
    
    (result, total)
}
