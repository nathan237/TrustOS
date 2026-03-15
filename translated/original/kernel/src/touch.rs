//! Touch Input Driver
//!
//! Multitouch input handling for touchscreen displays.
//! Lock-free design (same pattern as mouse.rs) — safe from interrupt context.
//!
//! Supports:
//! - USB HID multitouch (I2C-HID over USB, used on tablets/phones)
//! - Virtio-input touch (QEMU testing)
//! - Up to 10 simultaneous touch points
//!
//! The driver exposes raw touch events. Gesture recognition is in `gesture.rs`.

use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering};

// ═════════════════════════════════════════════════════════════════════════════
// Constants
// ═════════════════════════════════════════════════════════════════════════════

/// Maximum simultaneous touch points
pub const MAX_TOUCH_POINTS: usize = 10;

/// Touch event ring buffer size (power of 2 for fast masking)
const EVENT_BUFFER_SIZE: usize = 64;
const EVENT_BUFFER_MASK: usize = EVENT_BUFFER_SIZE - 1;

// ═════════════════════════════════════════════════════════════════════════════
// Types
// ═════════════════════════════════════════════════════════════════════════════

/// State of a single touch contact
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TouchPhase {
    /// Finger just made contact
    Down = 0,
    /// Finger is moving on the surface
    Moved = 1,
    /// Finger was lifted
    Up = 2,
    /// Touch was cancelled (e.g. palm rejection)
    Cancelled = 3,
}

impl TouchPhase {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Down,
            1 => Self::Moved,
            2 => Self::Up,
            3 => Self::Cancelled,
            _ => Self::Cancelled,
        }
    }
}

/// A single touch contact point
#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    /// Tracking ID (identifies the same finger across events)
    pub id: u16,
    /// X coordinate in screen pixels
    pub x: i32,
    /// Y coordinate in screen pixels
    pub y: i32,
    /// Contact pressure (0-255, 0 = not available)
    pub pressure: u8,
    /// Contact phase
    pub phase: TouchPhase,
    /// Timestamp in microseconds (from arch::timestamp())
    pub timestamp_us: u64,
}

impl Default for TouchPoint {
    fn default() -> Self {
        Self {
            id: 0,
            x: 0,
            y: 0,
            pressure: 0,
            phase: TouchPhase::Up,
            timestamp_us: 0,
        }
    }
}

/// Snapshot of all active touch contacts
#[derive(Debug, Clone)]
pub struct TouchState {
    /// Active touch points (only first `count` entries are valid)
    pub points: [TouchPoint; MAX_TOUCH_POINTS],
    /// Number of currently active touch points (fingers on screen)
    pub count: u8,
    /// Timestamp of last touch event
    pub timestamp_us: u64,
}

impl Default for TouchState {
    fn default() -> Self {
        Self {
            points: [TouchPoint::default(); MAX_TOUCH_POINTS],
            count: 0,
            timestamp_us: 0,
        }
    }
}

/// A touch event (queued for consumption by the gesture recognizer)
#[derive(Debug, Clone, Copy)]
pub struct TouchEvent {
    pub point: TouchPoint,
}

// ═════════════════════════════════════════════════════════════════════════════
// Atomic State (lock-free, interrupt-safe)
// ═════════════════════════════════════════════════════════════════════════════

/// Whether the touch subsystem has been initialized
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Whether a touchscreen device was detected
static DEVICE_PRESENT: AtomicBool = AtomicBool::new(false);

/// Screen dimensions for coordinate mapping
static SCREEN_WIDTH: AtomicU32 = AtomicU32::new(1280);
static SCREEN_HEIGHT: AtomicU32 = AtomicU32::new(800);

/// Touch device raw resolution (for coordinate scaling)
static DEVICE_MAX_X: AtomicU32 = AtomicU32::new(4096);
static DEVICE_MAX_Y: AtomicU32 = AtomicU32::new(4096);

/// Number of currently active touches
static ACTIVE_COUNT: AtomicU8 = AtomicU8::new(0);

/// Per-slot atomic state (up to MAX_TOUCH_POINTS slots)
/// Each slot corresponds to one finger tracking ID.
struct AtomicTouchSlot {
    active: AtomicBool,
    id: AtomicU16,
    x: AtomicI32,
    y: AtomicI32,
    pressure: AtomicU8,
    phase: AtomicU8,
    timestamp_us: AtomicU64,
}

impl AtomicTouchSlot {
    const fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            id: AtomicU16::new(0),
            x: AtomicI32::new(0),
            y: AtomicI32::new(0),
            pressure: AtomicU8::new(0),
            phase: AtomicU8::new(TouchPhase::Up as u8),
            timestamp_us: AtomicU64::new(0),
        }
    }

    fn load(&self) -> TouchPoint {
        TouchPoint {
            id: self.id.load(Ordering::Relaxed),
            x: self.x.load(Ordering::Relaxed),
            y: self.y.load(Ordering::Relaxed),
            pressure: self.pressure.load(Ordering::Relaxed),
            phase: TouchPhase::from_u8(self.phase.load(Ordering::Relaxed)),
            timestamp_us: self.timestamp_us.load(Ordering::Relaxed),
        }
    }

    fn store(&self, point: &TouchPoint) {
        self.id.store(point.id, Ordering::Relaxed);
        self.x.store(point.x, Ordering::Relaxed);
        self.y.store(point.y, Ordering::Relaxed);
        self.pressure.store(point.pressure, Ordering::Relaxed);
        self.phase.store(point.phase as u8, Ordering::Relaxed);
        self.timestamp_us.store(point.timestamp_us, Ordering::Relaxed);
        self.active.store(point.phase != TouchPhase::Up && point.phase != TouchPhase::Cancelled, Ordering::Relaxed);
    }

    fn clear(&self) {
        self.active.store(false, Ordering::Relaxed);
        self.phase.store(TouchPhase::Up as u8, Ordering::Relaxed);
    }
}

/// Static array of touch slots
static TOUCH_SLOTS: [AtomicTouchSlot; MAX_TOUCH_POINTS] = [
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
];

// ═════════════════════════════════════════════════════════════════════════════
// Event Ring Buffer (lock-free SPSC: interrupt → main thread)
// ═════════════════════════════════════════════════════════════════════════════

/// Ring buffer for touch events (producer: IRQ handler, consumer: gesture engine)
struct EventRingBuffer {
    /// Packed event data: [id:16 | x:32 | y:32 | pressure:8 | phase:8 | timestamp:64]
    /// We store events as parallel arrays of atomics for lock-free access.
    ids: [AtomicU16; EVENT_BUFFER_SIZE],
    xs: [AtomicI32; EVENT_BUFFER_SIZE],
    ys: [AtomicI32; EVENT_BUFFER_SIZE],
    pressures: [AtomicU8; EVENT_BUFFER_SIZE],
    phases: [AtomicU8; EVENT_BUFFER_SIZE],
    timestamps: [AtomicU64; EVENT_BUFFER_SIZE],
    /// Write index (only modified by producer/IRQ)
    write_idx: AtomicU32,
    /// Read index (only modified by consumer/main)
    read_idx: AtomicU32,
}

// Helper macro for initializing atomic arrays
macro_rules! atomic_array {
    ($type:ty, $val:expr, $n:expr) => {{
        // Workaround: const array init doesn't work for AtomicXXX in static,
        // so we use a const helper.
        const INIT: $type = $val;
        [INIT; $n]
    }};
}

static EVENT_RING: EventRingBuffer = EventRingBuffer {
    ids: atomic_array!(AtomicU16, AtomicU16::new(0), EVENT_BUFFER_SIZE),
    xs: atomic_array!(AtomicI32, AtomicI32::new(0), EVENT_BUFFER_SIZE),
    ys: atomic_array!(AtomicI32, AtomicI32::new(0), EVENT_BUFFER_SIZE),
    pressures: atomic_array!(AtomicU8, AtomicU8::new(0), EVENT_BUFFER_SIZE),
    phases: atomic_array!(AtomicU8, AtomicU8::new(0), EVENT_BUFFER_SIZE),
    timestamps: atomic_array!(AtomicU64, AtomicU64::new(0), EVENT_BUFFER_SIZE),
    write_idx: AtomicU32::new(0),
    read_idx: AtomicU32::new(0),
};

impl EventRingBuffer {
    /// Push a touch event (called from IRQ / injection context)
    fn push(&self, point: &TouchPoint) {
        let w = self.write_idx.load(Ordering::Relaxed);
        let idx = (w as usize) & EVENT_BUFFER_MASK;

        self.ids[idx].store(point.id, Ordering::Relaxed);
        self.xs[idx].store(point.x, Ordering::Relaxed);
        self.ys[idx].store(point.y, Ordering::Relaxed);
        self.pressures[idx].store(point.pressure, Ordering::Relaxed);
        self.phases[idx].store(point.phase as u8, Ordering::Relaxed);
        self.timestamps[idx].store(point.timestamp_us, Ordering::Relaxed);

        // Advance write index (Release ensures data is visible before index)
        self.write_idx.store(w.wrapping_add(1), Ordering::Release);
    }

    /// Pop a touch event (called from main/gesture thread)
    fn pop(&self) -> Option<TouchEvent> {
        let r = self.read_idx.load(Ordering::Relaxed);
        let w = self.write_idx.load(Ordering::Acquire);

        if r == w {
            return None; // Empty
        }

        let idx = (r as usize) & EVENT_BUFFER_MASK;
        let point = TouchPoint {
            id: self.ids[idx].load(Ordering::Relaxed),
            x: self.xs[idx].load(Ordering::Relaxed),
            y: self.ys[idx].load(Ordering::Relaxed),
            pressure: self.pressures[idx].load(Ordering::Relaxed),
            phase: TouchPhase::from_u8(self.phases[idx].load(Ordering::Relaxed)),
            timestamp_us: self.timestamps[idx].load(Ordering::Relaxed),
        };

        self.read_idx.store(r.wrapping_add(1), Ordering::Release);
        Some(TouchEvent { point })
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Public API
// ═════════════════════════════════════════════════════════════════════════════

/// Initialize the touch subsystem
pub fn init() {
    // Reset all slots
    for slot in &TOUCH_SLOTS {
        slot.clear();
    }
    ACTIVE_COUNT.store(0, Ordering::Relaxed);
    INITIALIZED.store(true, Ordering::Relaxed);
    crate::serial_println!("[TOUCH] Touch subsystem initialized (max {} points)", MAX_TOUCH_POINTS);
}

/// Set screen dimensions for coordinate mapping
pub fn set_screen_size(width: u32, height: u32) {
    SCREEN_WIDTH.store(width, Ordering::Relaxed);
    SCREEN_HEIGHT.store(height, Ordering::Relaxed);
}

/// Set device raw resolution (for scaling raw→screen coords)
pub fn set_device_resolution(max_x: u32, max_y: u32) {
    DEVICE_MAX_X.store(max_x, Ordering::Relaxed);
    DEVICE_MAX_Y.store(max_y, Ordering::Relaxed);
}

/// Report that a touchscreen device was detected
pub fn set_device_present(present: bool) {
    DEVICE_PRESENT.store(present, Ordering::Relaxed);
    if present {
        crate::serial_println!("[TOUCH] Touchscreen device detected");
    }
}

/// Check if a touchscreen is available
pub fn is_available() -> bool {
    INITIALIZED.load(Ordering::Relaxed) && DEVICE_PRESENT.load(Ordering::Relaxed)
}

/// Check if the touch subsystem is initialized (even without a device)
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}

/// Get the number of fingers currently on screen
pub fn active_count() -> u8 {
    ACTIVE_COUNT.load(Ordering::Relaxed)
}

/// Get a snapshot of all active touch contacts
pub fn get_state() -> TouchState {
    let mut state = TouchState::default();
    let mut count = 0u8;

    for slot in &TOUCH_SLOTS {
        if slot.active.load(Ordering::Relaxed) && (count as usize) < MAX_TOUCH_POINTS {
            state.points[count as usize] = slot.load();
            count += 1;
        }
    }

    state.count = count;
    state.timestamp_us = crate::gui::engine::now_us();
    state
}

/// Pop the next touch event from the queue (returns None if empty)
pub fn poll_event() -> Option<TouchEvent> {
    EVENT_RING.pop()
}

/// Drain all pending events into a callback
pub fn drain_events<F: FnMut(TouchEvent)>(mut f: F) {
    while let Some(evt) = EVENT_RING.pop() {
        f(evt);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Injection API (used by USB HID / virtio-input drivers)
// ═════════════════════════════════════════════════════════════════════════════

/// Inject a raw touch event from a hardware driver.
///
/// `raw_x` / `raw_y` are in device coordinates (0..DEVICE_MAX_X/Y).
/// They will be scaled to screen coordinates automatically.
pub fn inject_raw(id: u16, raw_x: u32, raw_y: u32, pressure: u8, phase: TouchPhase) {
    let screen_w = SCREEN_WIDTH.load(Ordering::Relaxed);
    let screen_h = SCREEN_HEIGHT.load(Ordering::Relaxed);
    let dev_max_x = DEVICE_MAX_X.load(Ordering::Relaxed).max(1);
    let dev_max_y = DEVICE_MAX_Y.load(Ordering::Relaxed).max(1);

    // Scale device coordinates → screen pixels
    let x = ((raw_x as u64 * screen_w as u64) / dev_max_x as u64) as i32;
    let y = ((raw_y as u64 * screen_h as u64) / dev_max_y as u64) as i32;

    inject_screen(id, x, y, pressure, phase);
}

/// Inject a touch event already in screen coordinates.
///
/// Used by virtio-input (which reports in screen pixels directly)
/// or for software-generated touch events (testing).
pub fn inject_screen(id: u16, x: i32, y: i32, pressure: u8, phase: TouchPhase) {
    let screen_w = SCREEN_WIDTH.load(Ordering::Relaxed) as i32;
    let screen_h = SCREEN_HEIGHT.load(Ordering::Relaxed) as i32;

    let point = TouchPoint {
        id,
        x: x.clamp(0, screen_w - 1),
        y: y.clamp(0, screen_h - 1),
        pressure,
        phase,
        timestamp_us: crate::gui::engine::now_us(),
    };

    // Update slot
    let slot_idx = find_or_alloc_slot(id, phase);
    if let Some(idx) = slot_idx {
        TOUCH_SLOTS[idx].store(&point);
    }

    // Update active count
    let mut count = 0u8;
    for slot in &TOUCH_SLOTS {
        if slot.active.load(Ordering::Relaxed) {
            count += 1;
        }
    }
    ACTIVE_COUNT.store(count, Ordering::Relaxed);

    // Push to event ring buffer
    EVENT_RING.push(&point);
}

/// Find existing slot for `id`, or allocate a new one on Down.
/// Returns None if all slots are full.
fn find_or_alloc_slot(id: u16, phase: TouchPhase) -> Option<usize> {
    // First: find existing slot with this ID
    for (i, slot) in TOUCH_SLOTS.iter().enumerate() {
        if slot.active.load(Ordering::Relaxed) && slot.id.load(Ordering::Relaxed) == id {
            return Some(i);
        }
    }

    // If this is a Down event, allocate a new slot
    if phase == TouchPhase::Down {
        for (i, slot) in TOUCH_SLOTS.iter().enumerate() {
            if !slot.active.load(Ordering::Relaxed) {
                return Some(i);
            }
        }
    }

    // For Move/Up events on an unknown ID, try slot 0 as fallback
    if phase == TouchPhase::Up || phase == TouchPhase::Cancelled {
        // Just find any inactive slot to record the event
        for (i, slot) in TOUCH_SLOTS.iter().enumerate() {
            if slot.id.load(Ordering::Relaxed) == id {
                return Some(i);
            }
        }
    }

    None
}

// ═════════════════════════════════════════════════════════════════════════════
// Mouse Emulation (touch → mouse for apps that don't know about touch)
// ═════════════════════════════════════════════════════════════════════════════

/// Convert single-finger touch to mouse events.
/// Called from the desktop's input processing loop.
/// Returns the primary touch point if any finger is down.
pub fn emulate_mouse() -> Option<(i32, i32, bool)> {
    // Use the first active touch point as the mouse cursor
    for slot in &TOUCH_SLOTS {
        if slot.active.load(Ordering::Relaxed) {
            let x = slot.x.load(Ordering::Relaxed);
            let y = slot.y.load(Ordering::Relaxed);
            let phase = TouchPhase::from_u8(slot.phase.load(Ordering::Relaxed));
            let pressed = phase == TouchPhase::Down || phase == TouchPhase::Moved;
            return Some((x, y, pressed));
        }
    }
    None
}
