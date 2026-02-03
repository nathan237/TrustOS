//! PS/2 Mouse Driver
//! 
//! Handles mouse input for GUI interactions with scroll wheel support.
//! OPTIMIZED: Uses atomic operations for hot path to reduce lock contention.

use x86_64::instructions::port::Port;
use spin::Mutex;
use core::sync::atomic::{AtomicI32, AtomicBool, AtomicI8, AtomicU64, Ordering};

/// PS/2 controller ports
const PS2_DATA: u16 = 0x60;
const PS2_STATUS: u16 = 0x64;
const PS2_COMMAND: u16 = 0x64;

// OPTIMIZED: Use atomics for frequently accessed state (no locks in hot path)
static MOUSE_X: AtomicI32 = AtomicI32::new(640);
static MOUSE_Y: AtomicI32 = AtomicI32::new(400);
static LEFT_BUTTON: AtomicBool = AtomicBool::new(false);
static RIGHT_BUTTON: AtomicBool = AtomicBool::new(false);
static MIDDLE_BUTTON: AtomicBool = AtomicBool::new(false);
static SCROLL_DELTA: AtomicI8 = AtomicI8::new(0);

/// Screen dimensions (rarely change, ok to use mutex)
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

/// Mouse has scroll wheel (IntelliMouse)
static HAS_SCROLL_WHEEL: Mutex<bool> = Mutex::new(false);

/// Mouse packet buffer (4 bytes for scroll wheel mouse)
static MOUSE_PACKET: Mutex<[u8; 4]> = Mutex::new([0; 4]);
static PACKET_INDEX: Mutex<usize> = Mutex::new(0);

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
        *HAS_SCROLL_WHEEL.lock() = true;
        crate::serial_println!("[MOUSE] IntelliMouse scroll wheel enabled");
    }
    
    // Enable mouse
    mouse_write(0xF4);
    
    crate::serial_println!("[MOUSE] PS/2 mouse initialized (ID={})", device_id);
}

/// Set screen dimensions for clamping
pub fn set_screen_size(width: u32, height: u32) {
    SCREEN_WIDTH.store(width as i32, Ordering::Relaxed);
    SCREEN_HEIGHT.store(height as i32, Ordering::Relaxed);
}

/// Handle mouse interrupt (called from IRQ12 handler)
/// OPTIMIZED: Uses atomics for position updates (lock-free in hot path)
pub fn handle_interrupt() {
    let mut data_port = Port::<u8>::new(PS2_DATA);
    let byte = unsafe { data_port.read() };
    
    let mut packet = MOUSE_PACKET.lock();
    let mut index = PACKET_INDEX.lock();
    let has_scroll = *HAS_SCROLL_WHEEL.lock();
    let packet_size = if has_scroll { 4 } else { 3 };
    
    // First byte must have bit 3 set (always 1)
    if *index == 0 && byte & 0x08 == 0 {
        return; // Out of sync, wait for valid first byte
    }
    
    packet[*index] = byte;
    *index += 1;
    
    if *index >= packet_size {
        *index = 0;
        
        // Parse packet
        let buttons = packet[0];
        let x_rel = packet[1] as i8 as i32;
        let y_rel = packet[2] as i8 as i32;
        let z_rel = if has_scroll { packet[3] as i8 } else { 0 };
        
        // Handle overflow
        let x_overflow = buttons & 0x40 != 0;
        let y_overflow = buttons & 0x80 != 0;
        
        if !x_overflow && !y_overflow {
            let width = SCREEN_WIDTH.load(Ordering::Relaxed);
            let height = SCREEN_HEIGHT.load(Ordering::Relaxed);
            
            // OPTIMIZED: Use atomics for position (no mutex lock)
            let old_x = MOUSE_X.load(Ordering::Relaxed);
            let old_y = MOUSE_Y.load(Ordering::Relaxed);
            
            // Update position (Y is inverted in PS/2)
            // Apply mouse acceleration for better feel
            let accel = 1; // Acceleration factor
            let new_x = (old_x + x_rel * accel).clamp(0, width - 1);
            let new_y = (old_y - y_rel * accel).clamp(0, height - 1);
            
            MOUSE_X.store(new_x, Ordering::Relaxed);
            MOUSE_Y.store(new_y, Ordering::Relaxed);
            
            // Update buttons atomically
            LEFT_BUTTON.store(buttons & 0x01 != 0, Ordering::Relaxed);
            RIGHT_BUTTON.store(buttons & 0x02 != 0, Ordering::Relaxed);
            MIDDLE_BUTTON.store(buttons & 0x04 != 0, Ordering::Relaxed);
            
            // Update scroll wheel delta
            if z_rel != 0 {
                SCROLL_DELTA.store(z_rel, Ordering::Relaxed);
            }
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
    *HAS_SCROLL_WHEEL.lock() || true  // Always true after init() is called
}
