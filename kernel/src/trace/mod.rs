//! Event Tracing Subsystem
//! 
//! Lock-free ring buffer for kernel event tracing.
//! Supports deterministic debugging and replay.

use spin::Mutex;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

/// Ring buffer size (power of 2 for efficient wrapping)
const TRACE_BUFFER_SIZE: usize = 4096;

/// Trace buffer
static TRACE_BUFFER: Mutex<[TraceEvent; TRACE_BUFFER_SIZE]> = 
    Mutex::new([TraceEvent::EMPTY; TRACE_BUFFER_SIZE]);

/// Write index
static WRITE_INDEX: AtomicU64 = AtomicU64::new(0);

/// Tracing enabled flag
static TRACING_ENABLED: AtomicBool = AtomicBool::new(true);

/// Deterministic mode flag
static DETERMINISTIC_MODE: AtomicBool = AtomicBool::new(false);

/// Event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    None = 0,
    TimerTick = 1,
    ContextSwitch = 2,
    IpcSend = 3,
    IpcReceive = 4,
    PageFault = 5,
    Interrupt = 6,
    SyscallEntry = 7,
    SyscallExit = 8,
    SecurityViolation = 9,
    MemoryAlloc = 10,
    MemoryFree = 11,
    KeyboardInput = 12,
    Custom = 255,
}

/// Single trace event
#[derive(Debug, Clone, Copy)]
pub struct TraceEvent {
    /// Timestamp (kernel ticks)
    pub timestamp: u64,
    /// CPU that generated event
    pub cpu_id: u8,
    /// Event type
    pub event_type: EventType,
    /// Event-specific payload
    pub payload: u64,
}

impl TraceEvent {
    pub const EMPTY: Self = Self {
        timestamp: 0,
        cpu_id: 0,
        event_type: EventType::None,
        payload: 0,
    };
}

/// Initialize tracing subsystem
pub fn init() {
    // Check for deterministic mode environment
    // In real implementation, this would come from boot params
    #[cfg(feature = "deterministic")]
    {
        DETERMINISTIC_MODE.store(true, Ordering::SeqCst);
        crate::log_debug!("Deterministic mode ENABLED");
    }
    
    crate::log_debug!("Trace buffer initialized ({} entries)", TRACE_BUFFER_SIZE);
}

/// Record a trace event
pub fn record_event(event_type: EventType, payload: u64) {
    if !TRACING_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    
    let timestamp = crate::logger::get_timestamp();
    let cpu_id = 0; // TODO: Get actual CPU ID
    
    let event = TraceEvent {
        timestamp,
        cpu_id,
        event_type,
        payload,
    };
    
    let index = WRITE_INDEX.fetch_add(1, Ordering::Relaxed) as usize;
    let slot = index % TRACE_BUFFER_SIZE;
    
    TRACE_BUFFER.lock()[slot] = event;
}

/// Enable/disable tracing
pub fn set_enabled(enabled: bool) {
    TRACING_ENABLED.store(enabled, Ordering::SeqCst);
}

/// Check if tracing is enabled
pub fn is_enabled() -> bool {
    TRACING_ENABLED.load(Ordering::Relaxed)
}

/// Enable deterministic mode
pub fn enable_deterministic_mode() {
    DETERMINISTIC_MODE.store(true, Ordering::SeqCst);
    crate::log!("Deterministic mode ENABLED");
}

/// Check if in deterministic mode
pub fn is_deterministic() -> bool {
    DETERMINISTIC_MODE.load(Ordering::Relaxed)
}

/// Dump trace buffer on panic
pub fn dump_on_panic() {
    let buffer = TRACE_BUFFER.lock();
    let current = WRITE_INDEX.load(Ordering::Relaxed) as usize;
    
    crate::serial::_print(format_args!("\n=== TRACE DUMP (last 32 events) ===\n"));
    
    // Dump last 32 events
    let start = current.saturating_sub(32);
    for i in start..current {
        let event = &buffer[i % TRACE_BUFFER_SIZE];
        if event.event_type != EventType::None {
            crate::serial::_print(format_args!(
                "[{:>10}][CPU{}] {:?} payload={:#x}\n",
                event.timestamp,
                event.cpu_id,
                event.event_type,
                event.payload
            ));
        }
    }
    
    crate::serial::_print(format_args!("=== END TRACE DUMP ===\n"));
}

/// Export trace buffer to userland
pub fn export_buffer() -> alloc::vec::Vec<TraceEvent> {
    let buffer = TRACE_BUFFER.lock();
    let current = WRITE_INDEX.load(Ordering::Relaxed) as usize;
    let count = current.min(TRACE_BUFFER_SIZE);
    
    let mut events = alloc::vec::Vec::with_capacity(count);
    let start = if current > TRACE_BUFFER_SIZE {
        current - TRACE_BUFFER_SIZE
    } else {
        0
    };
    
    for i in start..current {
        events.push(buffer[i % TRACE_BUFFER_SIZE]);
    }
    
    events
}

/// Get trace statistics
pub fn stats() -> TraceStats {
    TraceStats {
        events_recorded: WRITE_INDEX.load(Ordering::Relaxed),
        buffer_size: TRACE_BUFFER_SIZE,
        tracing_enabled: TRACING_ENABLED.load(Ordering::Relaxed),
        deterministic_mode: DETERMINISTIC_MODE.load(Ordering::Relaxed),
    }
}

/// Trace statistics
#[derive(Debug, Clone)]
pub struct TraceStats {
    pub events_recorded: u64,
    pub buffer_size: usize,
    pub tracing_enabled: bool,
    pub deterministic_mode: bool,
}
