//! Wayland Seat (Input Devices)
//!
//! wl_seat represents a group of input devices (keyboard, pointer, touch).

use alloc::string::String;
use alloc::vec::Vec;

/// Keyboard key state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyState {
    Released = 0,
    Pressed = 1,
}

/// Pointer button state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Released = 0,
    Pressed = 1,
}

/// Keyboard modifier state
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierState {
    pub depressed: u32,
    pub latched: u32,
    pub locked: u32,
    pub group: u32,
}

/// Input seat managing keyboard, pointer, and touch
pub struct Seat {
    /// Seat name
    pub name: String,
    
    /// Capabilities
    pub capabilities: u32,
    
    /// Pointer state
    pub pointer: PointerState,
    
    /// Keyboard state
    pub keyboard: KeyboardState,
    
    /// Currently focused surface
    pub focused_surface: Option<u32>,
    
    /// Serial counter for input events
    serial: u32,
}

impl Seat {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            capabilities: 0b11, // Pointer + Keyboard
            pointer: PointerState::new(),
            keyboard: KeyboardState::new(),
            focused_surface: None,
            serial: 1,
        }
    }
    
    /// Get next event serial
    pub fn next_serial(&mut self) -> u32 {
        let s = self.serial;
        self.serial = self.serial.wrapping_add(1);
        s
    }
    
    /// Set keyboard focus to a surface
    pub fn set_keyboard_focus(&mut self, surface_id: Option<u32>) {
        self.focused_surface = surface_id;
    }
    
    /// Check if this seat has pointer capability
    pub fn has_pointer(&self) -> bool {
        self.capabilities & 1 != 0
    }
    
    /// Check if this seat has keyboard capability
    pub fn has_keyboard(&self) -> bool {
        self.capabilities & 2 != 0
    }
    
    /// Check if this seat has touch capability
    pub fn has_touch(&self) -> bool {
        self.capabilities & 4 != 0
    }
}

/// Pointer device state
pub struct PointerState {
    /// Current X position
    pub x: f64,
    
    /// Current Y position
    pub y: f64,
    
    /// Surface the pointer is over
    pub focus: Option<u32>,
    
    /// Surface-local coordinates
    pub surface_x: f64,
    pub surface_y: f64,
    
    /// Button states
    pub buttons: u32,
    
    /// Current cursor surface
    pub cursor_surface: Option<u32>,
    pub cursor_hotspot_x: i32,
    pub cursor_hotspot_y: i32,
}

impl PointerState {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            focus: None,
            surface_x: 0.0,
            surface_y: 0.0,
            buttons: 0,
            cursor_surface: None,
            cursor_hotspot_x: 0,
            cursor_hotspot_y: 0,
        }
    }
    
    /// Update pointer position
    pub fn move_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }
    
    /// Update button state
    pub fn set_button(&mut self, button: u32, pressed: bool) {
        if pressed {
            self.buttons |= 1 << button;
        } else {
            self.buttons &= !(1 << button);
        }
    }
    
    /// Check if a button is pressed
    pub fn is_button_pressed(&self, button: u32) -> bool {
        self.buttons & (1 << button) != 0
    }
    
    /// Set cursor surface
    pub fn set_cursor(&mut self, surface: Option<u32>, hotspot_x: i32, hotspot_y: i32) {
        self.cursor_surface = surface;
        self.cursor_hotspot_x = hotspot_x;
        self.cursor_hotspot_y = hotspot_y;
    }
}

impl Default for PointerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Keyboard device state
pub struct KeyboardState {
    /// Currently pressed keys (scancodes)
    pub pressed_keys: Vec<u32>,
    
    /// Modifier state
    pub modifiers: ModifierState,
    
    /// Repeat rate (chars per second, 0 = disabled)
    pub repeat_rate: i32,
    
    /// Repeat delay (milliseconds)
    pub repeat_delay: i32,
    
    /// Surface with keyboard focus
    pub focus: Option<u32>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            pressed_keys: Vec::new(),
            modifiers: ModifierState::default(),
            repeat_rate: 25,
            repeat_delay: 400,
            focus: None,
        }
    }
    
    /// Key press
    pub fn key_press(&mut self, key: u32) {
        if !self.pressed_keys.contains(&key) {
            self.pressed_keys.push(key);
        }
    }
    
    /// Key release
    pub fn key_release(&mut self, key: u32) {
        self.pressed_keys.retain(|&k| k != key);
    }
    
    /// Check if key is pressed
    pub fn is_key_pressed(&self, key: u32) -> bool {
        self.pressed_keys.contains(&key)
    }
    
    /// Update modifiers
    pub fn set_modifiers(&mut self, depressed: u32, latched: u32, locked: u32, group: u32) {
        self.modifiers = ModifierState {
            depressed,
            latched,
            locked,
            group,
        };
    }
    
    /// Clear all pressed keys
    pub fn clear(&mut self) {
        self.pressed_keys.clear();
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// INPUT EVENT TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Pointer motion event
#[derive(Debug, Clone)]
pub struct PointerMotionEvent {
    pub time: u32,
    pub surface_x: f64,
    pub surface_y: f64,
}

/// Pointer button event
#[derive(Debug, Clone)]
pub struct PointerButtonEvent {
    pub serial: u32,
    pub time: u32,
    pub button: u32,
    pub state: ButtonState,
}

/// Pointer axis event (scroll)
#[derive(Debug, Clone)]
pub struct PointerAxisEvent {
    pub time: u32,
    pub axis: u32, // 0 = vertical, 1 = horizontal
    pub value: f64,
}

/// Keyboard key event
#[derive(Debug, Clone)]
pub struct KeyboardKeyEvent {
    pub serial: u32,
    pub time: u32,
    pub key: u32,
    pub state: KeyState,
}

/// Keyboard modifiers event
#[derive(Debug, Clone)]
pub struct KeyboardModifiersEvent {
    pub serial: u32,
    pub mods_depressed: u32,
    pub mods_latched: u32,
    pub mods_locked: u32,
    pub group: u32,
}
