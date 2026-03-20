//! WOA Input — Game-oriented keyboard state using PS/2 scancode bitmap
//!
//! Uses kernel's `keyboard::is_key_pressed(scancode)` for continuous hold detection.
//! Drains the char buffer each frame to prevent overflow.

/// PS/2 Scancode Set 1 constants for game keys
pub const SC_ESC: u8    = 0x01;
pub const SC_SPACE: u8  = 0x39;
pub const SC_ENTER: u8  = 0x1C;
pub const SC_W: u8      = 0x11;
pub const SC_A: u8      = 0x1E;
pub const SC_S: u8      = 0x1F;
pub const SC_D: u8      = 0x20;
pub const SC_Z: u8      = 0x2C;
pub const SC_Q: u8      = 0x10;
pub const SC_E: u8      = 0x12;
pub const SC_P: u8      = 0x19;
pub const SC_UP: u8     = 0x48;  // Arrow up (& numpad 8)
pub const SC_DOWN: u8   = 0x50;  // Arrow down (& numpad 2)
pub const SC_LEFT: u8   = 0x4B;  // Arrow left (& numpad 4)
pub const SC_RIGHT: u8  = 0x4D;  // Arrow right (& numpad 6)
pub const SC_LSHIFT: u8 = 0x2A;
pub const SC_TAB: u8    = 0x0F;

pub struct Input {
    /// Keys pressed this frame (edge detection)
    prev_state: [bool; 24],
}

/// Mapping from index to scancode for edge detection
const TRACKED_KEYS: [u8; 24] = [
    SC_ESC, SC_SPACE, SC_ENTER,
    SC_W, SC_A, SC_S, SC_D,
    SC_Z, SC_Q, SC_E, SC_P,
    SC_UP, SC_DOWN, SC_LEFT, SC_RIGHT,
    SC_LSHIFT, SC_TAB,
    0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // 1-7 keys
];

impl Input {
    pub fn new() -> Self {
        Self {
            prev_state: [false; 24],
        }
    }

    /// Call once per frame — drains keyboard buffer and updates edge state
    pub fn poll(&mut self) {
        // Drain the keyboard char buffer to prevent overflow
        while crate::keyboard::read_char().is_some() {}

        // prev_state is updated at end of frame via end_frame()
    }

    /// Update previous state — call at END of frame
    pub fn end_frame(&mut self) {
        for (i, &sc) in TRACKED_KEYS.iter().enumerate() {
            self.prev_state[i] = crate::keyboard::is_key_pressed(sc);
        }
    }

    /// Is key currently held down? (continuous)
    #[inline]
    pub fn is_held(&self, scancode: u8) -> bool {
        crate::keyboard::is_key_pressed(scancode)
    }

    /// Was key just pressed this frame? (edge: not held last frame, held now)
    pub fn is_pressed(&self, scancode: u8) -> bool {
        let now = crate::keyboard::is_key_pressed(scancode);
        if !now { return false; }
        // Find in tracked keys
        for (i, &sc) in TRACKED_KEYS.iter().enumerate() {
            if sc == scancode {
                return !self.prev_state[i];
            }
        }
        // Not tracked — fallback to held
        now
    }

    /// Was key just released this frame? (edge: held last frame, not held now)
    pub fn is_released(&self, scancode: u8) -> bool {
        let now = crate::keyboard::is_key_pressed(scancode);
        if now { return false; }
        for (i, &sc) in TRACKED_KEYS.iter().enumerate() {
            if sc == scancode {
                return self.prev_state[i];
            }
        }
        false
    }

    /// Any tracked key pressed this frame?
    pub fn any_pressed(&self) -> bool {
        for &sc in TRACKED_KEYS.iter() {
            if crate::keyboard::is_key_pressed(sc) {
                return true;
            }
        }
        false
    }
}
