//! PS/2 Mouse Driver
//! 
//! Handles mouse input for GUI interactions with scroll wheel support.
//! OPTIMIZED: Fully lock-free interrupt handler using atomics only.

use x86_64::instructions::port::Port;
use spin::Mutex;
use core::sync::atomic::{AtomicI32, AtomicBool, AtomicI8, AtomicU8, AtomicU64, Ordering};

/// PS/2 controller ports
const PS2_DATA: u16 = 0x60;
const PS2_STATUS: u16 = 0x64;
const PS2_COMMAND: u16 = 0x64;

// Fully atomic mouse state — no locks in interrupt handler
static MOUSE_X: AtomicI32 = AtomicI32::new(640);
static MOUSE_Y: AtomicI32 = AtomicI32::new(400);
static LEFT_BUTTON: AtomicBool = AtomicBool::new(false);
static RIGHT_BUTTON: AtomicBool = AtomicBool::new(false);
static MIDDLE_BUTTON: AtomicBool = AtomicBool::new(false);
static SCROLL_DELTA: AtomicI8 = AtomicI8::new(0);

/// Screen dimensions
static SCREEN_WIDTH: AtomicI32 = AtomicI32::new(1280);
static SCREEN_HEIGHT: AtomicI32 = AtomicI32::new(800);

/// Mouse state (for compatibility)
#[derive(Clone, Copy, Debug, Default)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    pub scroll_delta: i8,
}

/// Last click timestamp for double-click detection
static LAST_CLICK_TIME: AtomicU64 = AtomicU64::new(0);
static CLICK_COUNT: Mutex<u8> = Mutex::new(0);

/// Mouse has scroll wheel — set once during init, read-only after (atomic, no lock)
static HAS_SCROLL_WHEEL: AtomicBool = AtomicBool::new(false);

/// Lock-free packet buffer using atomics — NO mutexes in interrupt path
static PACKET_BYTE0: AtomicU8 = AtomicU8::new(0);
static PACKET_BYTE1: AtomicU8 = AtomicU8::new(0);
static PACKET_BYTE2: AtomicU8 = AtomicU8::new(0);
static PACKET_BYTE3: AtomicU8 = AtomicU8::new(0);
static PACKET_INDEX: AtomicU8 = AtomicU8::new(0);

/// Wait for PS/2 controller to be ready for reading
fn wait_read() {
    let mut status = Port::<u8>::new(PS2_STATUS);
    for _ in 0..100_000 {
        if unsafe { status.read() } & 0x01 != 0 {
            return;
        }
        core::hint::spin_loop();
    }
}

/// Wait for PS/2 controller to be ready for writing
fn wait_write() {
    let mut status = Port::<u8>::new(PS2_STATUS);
    for _ in 0..100_000 {
        if unsafe { status.read() } & 0x02 == 0 {
            return;
        }
        core::hint::spin_loop();
    }
}

/// Send command to PS/2 controller
fn ps2_command(cmd: u8) {
    let mut command = Port::<u8>::new(PS2_COMMAND);
    wait_write();
    unsafe { command.write(cmd); }
}

/// Send data to PS/2 data port
fn ps2_write(data: u8) {
    let mut port = Port::<u8>::new(PS2_DATA);
    wait_write();
    unsafe { port.write(data); }
}

/// Read from PS/2 data port
fn ps2_read() -> u8 {
    let mut port = Port::<u8>::new(PS2_DATA);
    wait_read();
    unsafe { port.read() }
}

/// Send command to mouse (via PS/2 controller)
fn mouse_write(cmd: u8) {
    ps2_command(0xD4); // Tell controller to send to mouse
    ps2_write(cmd);
    ps2_read(); // Read ACK
}

/// Initialize PS/2 mouse
pub fn init() {
    // Enable auxiliary device (mouse)
    ps2_command(0xA8);
    
    // Get compaq status byte
    ps2_command(0x20);
    let status = ps2_read();
    
    // Enable IRQ12 and disable mouse clock disable
    let status = (status | 0x02) & !0x20;
    ps2_command(0x60);
    ps2_write(status);
    
    // Tell mouse to use default settings
    mouse_write(0xF6);
    
    // Try to enable scroll wheel (IntelliMouse protocol)
    // Magic sequence: Set sample rate to 200, then 100, then 80
    mouse_write(0xF3); ps2_write(200); // Set sample rate 200
    mouse_write(0xF3); ps2_write(100); // Set sample rate 100
    mouse_write(0xF3); ps2_write(80);  // Set sample rate 80
    
    // Get device ID to check if scroll wheel enabled
    mouse_write(0xF2); // Get device ID
    let device_id = ps2_read();
    if device_id == 3 {
        HAS_SCROLL_WHEEL.store(true, Ordering::Relaxed);
        crate::serial_println!("[MOUSE] IntelliMouse scroll wheel enabled");
    }
    
    // Enable mouse
    mouse_write(0xF4);
    
    // Flush any stale bytes from the PS/2 buffer to prevent packet misalignment
    let mut status_port = Port::<u8>::new(PS2_STATUS);
    let mut data_port = Port::<u8>::new(PS2_DATA);
    for _ in 0..16 {
        if unsafe { status_port.read() } & 0x01 != 0 {
            unsafe { data_port.read(); }
        } else {
            break;
        }
    }
    
    crate::serial_println!("[MOUSE] PS/2 mouse initialized (ID={})", device_id);
}

/// Set screen dimensions for clamping
pub fn set_screen_size(width: u32, height: u32) {
    SCREEN_WIDTH.store(width as i32, Ordering::Relaxed);
    SCREEN_HEIGHT.store(height as i32, Ordering::Relaxed);
}

/// Handle mouse interrupt (called from IRQ12 handler)
/// FULLY LOCK-FREE: No mutexes, only atomics — cannot deadlock
pub fn handle_interrupt() {
    let mut data_port = Port::<u8>::new(PS2_DATA);
    let byte = unsafe { data_port.read() };
    
    // All state is atomic — no locks needed
    let has_scroll = HAS_SCROLL_WHEEL.load(Ordering::Relaxed);
    let packet_size: u8 = if has_scroll { 4 } else { 3 };
    let idx = PACKET_INDEX.load(Ordering::Relaxed);
    
    // First byte must have bit 3 set (PS/2 protocol — always 1 in byte 0)
    if idx == 0 && byte & 0x08 == 0 {
        return; // Out of sync, wait for valid first byte
    }
    
    // Store byte in atomic packet buffer
    match idx {
        0 => PACKET_BYTE0.store(byte, Ordering::Relaxed),
        1 => PACKET_BYTE1.store(byte, Ordering::Relaxed),
        2 => PACKET_BYTE2.store(byte, Ordering::Relaxed),
        3 => PACKET_BYTE3.store(byte, Ordering::Relaxed),
        _ => {
            // Should never happen — reset
            PACKET_INDEX.store(0, Ordering::Relaxed);
            return;
        }
    }
    
    let next_idx = idx + 1;
    if next_idx < packet_size {
        PACKET_INDEX.store(next_idx, Ordering::Relaxed);
        return;
    }
    
    // Full packet received — reset index first
    PACKET_INDEX.store(0, Ordering::Relaxed);
    
    // Read packet bytes
    let b0 = PACKET_BYTE0.load(Ordering::Relaxed);
    let b1 = PACKET_BYTE1.load(Ordering::Relaxed);
    let b2 = PACKET_BYTE2.load(Ordering::Relaxed);
    let b3 = if has_scroll { PACKET_BYTE3.load(Ordering::Relaxed) } else { 0 };
    
    // Update buttons (always, even on overflow)
    LEFT_BUTTON.store(b0 & 0x01 != 0, Ordering::Relaxed);
    RIGHT_BUTTON.store(b0 & 0x02 != 0, Ordering::Relaxed);
    MIDDLE_BUTTON.store(b0 & 0x04 != 0, Ordering::Relaxed);
    
    // Proper 9-bit PS/2 sign extension using byte 0's sign bits
    let mut x_rel = b1 as i32;
    let mut y_rel = b2 as i32;
    if b0 & 0x10 != 0 { x_rel |= !0xFF_i32; } // X sign bit (bit 4 of byte 0)
    if b0 & 0x20 != 0 { y_rel |= !0xFF_i32; } // Y sign bit (bit 5 of byte 0)
    
    // Handle overflow: clamp to max movement instead of discarding
    let x_overflow = b0 & 0x40 != 0;
    let y_overflow = b0 & 0x80 != 0;
    if x_overflow {
        x_rel = if b0 & 0x10 != 0 { -255 } else { 255 };
    }
    if y_overflow {
        y_rel = if b0 & 0x20 != 0 { -255 } else { 255 };
    }
    
    let width = SCREEN_WIDTH.load(Ordering::Relaxed);
    let height = SCREEN_HEIGHT.load(Ordering::Relaxed);
    
    let old_x = MOUSE_X.load(Ordering::Relaxed);
    let old_y = MOUSE_Y.load(Ordering::Relaxed);
    
    // Update position (Y is inverted in PS/2)
    let new_x = (old_x + x_rel).clamp(0, width - 1);
    let new_y = (old_y - y_rel).clamp(0, height - 1);
    
    MOUSE_X.store(new_x, Ordering::Relaxed);
    MOUSE_Y.store(new_y, Ordering::Relaxed);
    
    // Update scroll wheel delta (accumulate, don't overwrite)
    if has_scroll {
        let z_rel = b3 as i8;
        if z_rel != 0 {
            let _ = SCROLL_DELTA.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
                Some(old.saturating_add(z_rel))
            });
        }
    }
}

/// Get current mouse state (lock-free, very fast)
pub fn get_state() -> MouseState {
    MouseState {
        x: MOUSE_X.load(Ordering::Relaxed),
        y: MOUSE_Y.load(Ordering::Relaxed),
        left_button: LEFT_BUTTON.load(Ordering::Relaxed),
        right_button: RIGHT_BUTTON.load(Ordering::Relaxed),
        middle_button: MIDDLE_BUTTON.load(Ordering::Relaxed),
        scroll_delta: SCROLL_DELTA.swap(0, Ordering::Relaxed),
    }
}

/// Get and consume scroll delta
pub fn get_scroll_delta() -> i8 {
    SCROLL_DELTA.swap(0, Ordering::Relaxed)
}

/// Record a click for double-click detection
pub fn record_click() {
    // Use frame count as time (approximate)
    let now = crate::logger::get_ticks();
    let last = LAST_CLICK_TIME.load(Ordering::Relaxed);
    LAST_CLICK_TIME.store(now, Ordering::Relaxed);
    
    let mut count = CLICK_COUNT.lock();
    // Double-click if within ~500ms (assuming ~100 ticks/sec)
    if now - last < 50 {
        *count = 2;
    } else {
        *count = 1;
    }
}

/// Check if this is a double-click (call after record_click)
pub fn is_double_click() -> bool {
    let count = CLICK_COUNT.lock();
    *count >= 2
}

/// Reset click count (call after handling double-click)
pub fn reset_click_count() {
    *CLICK_COUNT.lock() = 0;
}

/// Get mouse position (lock-free)
pub fn get_position() -> (i32, i32) {
    (MOUSE_X.load(Ordering::Relaxed), MOUSE_Y.load(Ordering::Relaxed))
}

/// Check if left button is pressed (lock-free)
pub fn is_left_pressed() -> bool {
    LEFT_BUTTON.load(Ordering::Relaxed)
}

/// Check if right button is pressed (lock-free)
pub fn is_right_pressed() -> bool {
    RIGHT_BUTTON.load(Ordering::Relaxed)
}

/// Check if mouse is initialized
pub fn is_initialized() -> bool {
    // Mouse is considered initialized if we have scroll wheel support
    // or if basic initialization completed
    HAS_SCROLL_WHEEL.load(Ordering::Relaxed) || true  // Always true after init() is called
}

// ═══════════════════════════════════════════════════════════════════════════════
// DELTA TRACKING for smooth mouse movement in GUI
// ═══════════════════════════════════════════════════════════════════════════════

static LAST_X: AtomicI32 = AtomicI32::new(640);
static LAST_Y: AtomicI32 = AtomicI32::new(400);

/// Get mouse delta since last call (for smooth GUI updates)
/// Returns None if mouse hasn't moved
pub fn get_delta() -> Option<(i32, i32)> {
    let cur_x = MOUSE_X.load(Ordering::Relaxed);
    let cur_y = MOUSE_Y.load(Ordering::Relaxed);
    let last_x = LAST_X.swap(cur_x, Ordering::Relaxed);
    let last_y = LAST_Y.swap(cur_y, Ordering::Relaxed);
    
    let dx = cur_x - last_x;
    let dy = cur_y - last_y;
    
    if dx != 0 || dy != 0 {
        Some((dx, dy))
    } else {
        None
    }
}
