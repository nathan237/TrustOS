//! TrustOS Accessibility Module
//!
//! Central accessibility settings: high contrast, font size, cursor size,
//! sticky keys, mouse speed. All settings are runtime-configurable.

use core::sync::atomic::{AtomicU8, AtomicBool, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════════
// ACCESSIBILITY SETTINGS — Global state
// ═══════════════════════════════════════════════════════════════════════════════

/// High contrast mode
static HIGH_CONTRAST: AtomicBool = AtomicBool::new(false);

/// Font size: 0=Small, 1=Medium (default), 2=Large, 3=XL
static FONT_SIZE: AtomicU8 = AtomicU8::new(1);

/// Cursor size: 0=Small (default), 1=Medium, 2=Large
static CURSOR_SIZE: AtomicU8 = AtomicU8::new(0);

/// Sticky keys enabled
static STICKY_KEYS: AtomicBool = AtomicBool::new(false);

/// Mouse speed: 0=Slow, 1=Normal (default), 2=Fast, 3=Very Fast
static MOUSE_SPEED: AtomicU8 = AtomicU8::new(1);

// ═══════════════════════════════════════════════════════════════════════════════
// STICKY KEYS STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Sticky key states: None → Latched (next key applies) → Locked (persistent)
#[derive(Clone, Copy, PartialEq)]
pub enum StickyState {
    Off,
    Latched,  // Active for next keypress only
    Locked,   // Persistent until pressed again
}

/// Sticky modifier states
static STICKY_CTRL: Mutex<StickyState> = Mutex::new(StickyState::Off);
static STICKY_ALT: Mutex<StickyState> = Mutex::new(StickyState::Off);
static STICKY_SHIFT: Mutex<StickyState> = Mutex::new(StickyState::Off);

// ═══════════════════════════════════════════════════════════════════════════════
// HIGH CONTRAST COLOR PALETTE
// ═══════════════════════════════════════════════════════════════════════════════

/// High-contrast colors — white on pure black, bold yellow highlights
pub struct HcColors;
impl HcColors {
    pub const BG_DEEPEST: u32     = 0xFF000000;
    pub const BG_DARK: u32        = 0xFF000000;
    pub const BG_MEDIUM: u32      = 0xFF0A0A0A;
    pub const BG_LIGHT: u32       = 0xFF1A1A1A;
    pub const PRIMARY: u32        = 0xFFFFFF00; // Bright yellow
    pub const SECONDARY: u32      = 0xFFFFFFFF; // Pure white
    pub const TERTIARY: u32       = 0xFFCCCCCC;
    pub const MUTED: u32          = 0xFF888888;
    pub const SUBTLE: u32         = 0xFF666666;
    pub const GHOST: u32          = 0xFF444444;
    pub const ACCENT_WARN: u32    = 0xFFFF8800; // Orange warnings
    pub const ACCENT_ERROR: u32   = 0xFFFF0000; // Pure red errors
    pub const ACCENT_INFO: u32    = 0xFF00CCFF; // Cyan info
    pub const TEXT_PRIMARY: u32   = 0xFFFFFFFF; // Pure white text
    pub const TEXT_SECONDARY: u32 = 0xFFCCCCCC;
    pub const TEXT_ACCENT: u32    = 0xFFFFFF00; // Yellow accent
    pub const WINDOW_BG: u32      = 0xFF000000;
    pub const WINDOW_BORDER: u32  = 0xFFFFFFFF;
    pub const TITLE_BAR: u32      = 0xFF1A1A1A;
    pub const DOCK_BG: u32        = 0xFF0A0A0A;
    pub const MENU_BG: u32        = 0xFF0A0A0A;
    pub const MENU_HOVER: u32     = 0xFF333300;
    pub const BTN_CLOSE: u32      = 0xFFFF0000;
    pub const BTN_MAXIMIZE: u32   = 0xFFFFFF00;
    pub const BTN_MINIMIZE: u32   = 0xFF00FF00;
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

// --- High Contrast ---
pub fn is_high_contrast() -> bool {
    HIGH_CONTRAST.load(Ordering::Relaxed)
}

pub fn set_high_contrast(enabled: bool) {
    HIGH_CONTRAST.store(enabled, Ordering::Relaxed);
    crate::serial_println!("[A11Y] High contrast: {}", if enabled { "ON" } else { "OFF" });
}

pub fn toggle_high_contrast() {
    let was = HIGH_CONTRAST.load(Ordering::Relaxed);
    HIGH_CONTRAST.store(!was, Ordering::Relaxed);
    crate::serial_println!("[A11Y] High contrast toggled: {}", if !was { "ON" } else { "OFF" });
}

/// Get color with high-contrast fallback
/// Usage: `a11y_color(NORMAL_COLOR, HcColors::REPLACEMENT)`
#[inline]
pub fn a11y_color(normal: u32, hc: u32) -> u32 {
    if HIGH_CONTRAST.load(Ordering::Relaxed) { hc } else { normal }
}

// --- Font Size ---

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum FontSize {
    Small = 0,
    Medium = 1,
    Large = 2,
    ExtraLarge = 3,
}

impl FontSize {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => FontSize::Small,
            1 => FontSize::Medium,
            2 => FontSize::Large,
            3 => FontSize::ExtraLarge,
            _ => FontSize::Medium,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FontSize::Small => "Small",
            FontSize::Medium => "Medium",
            FontSize::Large => "Large",
            FontSize::ExtraLarge => "XL",
        }
    }

    /// Extra scale multiplier applied on top of DPI scaling
    /// Small=0.875x (~7px), Medium=1.0x (8px), Large=1.25x (~10px), XL=1.5x (~12px)
    /// Returns (numerator, denominator) for integer math
    pub fn text_scale(&self) -> (u32, u32) {
        match self {
            FontSize::Small => (7, 8),
            FontSize::Medium => (1, 1),
            FontSize::Large => (5, 4),
            FontSize::ExtraLarge => (3, 2),
        }
    }
}

pub fn get_font_size() -> FontSize {
    FontSize::from_u8(FONT_SIZE.load(Ordering::Relaxed))
}

pub fn set_font_size(size: FontSize) {
    FONT_SIZE.store(size as u8, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Font size: {}", size.label());
}

pub fn cycle_font_size() {
    let current = FONT_SIZE.load(Ordering::Relaxed);
    let next = (current + 1) % 4;
    FONT_SIZE.store(next, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Font size: {}", FontSize::from_u8(next).label());
}

// --- Cursor Size ---

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum CursorSize {
    Small = 0,
    Medium = 1,
    Large = 2,
}

impl CursorSize {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => CursorSize::Small,
            1 => CursorSize::Medium,
            2 => CursorSize::Large,
            _ => CursorSize::Small,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CursorSize::Small => "Small",
            CursorSize::Medium => "Medium",
            CursorSize::Large => "Large",
        }
    }

    /// Pixel scale factor for cursor rendering
    pub fn scale(&self) -> u32 {
        match self {
            CursorSize::Small => 1,
            CursorSize::Medium => 2,
            CursorSize::Large => 3,
        }
    }
}

pub fn get_cursor_size() -> CursorSize {
    CursorSize::from_u8(CURSOR_SIZE.load(Ordering::Relaxed))
}

pub fn set_cursor_size(size: CursorSize) {
    CURSOR_SIZE.store(size as u8, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Cursor size: {}", size.label());
}

pub fn cycle_cursor_size() {
    let current = CURSOR_SIZE.load(Ordering::Relaxed);
    let next = (current + 1) % 3;
    CURSOR_SIZE.store(next, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Cursor size: {}", CursorSize::from_u8(next).label());
}

// --- Sticky Keys ---

pub fn is_sticky_keys() -> bool {
    STICKY_KEYS.load(Ordering::Relaxed)
}

pub fn set_sticky_keys(enabled: bool) {
    STICKY_KEYS.store(enabled, Ordering::Relaxed);
    if !enabled {
        // Clear all sticky states
        *STICKY_CTRL.lock() = StickyState::Off;
        *STICKY_ALT.lock() = StickyState::Off;
        *STICKY_SHIFT.lock() = StickyState::Off;
    }
    crate::serial_println!("[A11Y] Sticky keys: {}", if enabled { "ON" } else { "OFF" });
}

pub fn toggle_sticky_keys() {
    let was = STICKY_KEYS.load(Ordering::Relaxed);
    set_sticky_keys(!was);
}

/// Called when a modifier key is pressed. Returns whether the event was consumed.
/// Press once → Latched; press again while latched → Locked; press while locked → Off
pub fn sticky_modifier_press(modifier: StickyModifier) -> bool {
    if !is_sticky_keys() { return false; }
    let state_lock = match modifier {
        StickyModifier::Ctrl => &STICKY_CTRL,
        StickyModifier::Alt => &STICKY_ALT,
        StickyModifier::Shift => &STICKY_SHIFT,
    };
    let mut state = state_lock.lock();
    *state = match *state {
        StickyState::Off => StickyState::Latched,
        StickyState::Latched => StickyState::Locked,
        StickyState::Locked => StickyState::Off,
    };
    true
}

/// Called after a non-modifier key is pressed. Clears latched modifiers.
pub fn sticky_consume_latched() {
    if !is_sticky_keys() { return; }
    let mut ctrl = STICKY_CTRL.lock();
    if *ctrl == StickyState::Latched { *ctrl = StickyState::Off; }
    drop(ctrl);
    let mut alt = STICKY_ALT.lock();
    if *alt == StickyState::Latched { *alt = StickyState::Off; }
    drop(alt);
    let mut shift = STICKY_SHIFT.lock();
    if *shift == StickyState::Latched { *shift = StickyState::Off; }
}

/// Check if a modifier is active via sticky keys (latched or locked)
pub fn is_sticky_active(modifier: StickyModifier) -> bool {
    if !is_sticky_keys() { return false; }
    let state = match modifier {
        StickyModifier::Ctrl => *STICKY_CTRL.lock(),
        StickyModifier::Alt => *STICKY_ALT.lock(),
        StickyModifier::Shift => *STICKY_SHIFT.lock(),
    };
    state != StickyState::Off
}

/// Get sticky state for UI indicator
pub fn get_sticky_state(modifier: StickyModifier) -> StickyState {
    if !is_sticky_keys() { return StickyState::Off; }
    match modifier {
        StickyModifier::Ctrl => *STICKY_CTRL.lock(),
        StickyModifier::Alt => *STICKY_ALT.lock(),
        StickyModifier::Shift => *STICKY_SHIFT.lock(),
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum StickyModifier {
    Ctrl,
    Alt,
    Shift,
}

// --- Mouse Speed ---

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum MouseSpeed {
    Slow = 0,
    Normal = 1,
    Fast = 2,
    VeryFast = 3,
}

impl MouseSpeed {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => MouseSpeed::Slow,
            1 => MouseSpeed::Normal,
            2 => MouseSpeed::Fast,
            3 => MouseSpeed::VeryFast,
            _ => MouseSpeed::Normal,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MouseSpeed::Slow => "Slow",
            MouseSpeed::Normal => "Normal",
            MouseSpeed::Fast => "Fast",
            MouseSpeed::VeryFast => "Very Fast",
        }
    }

    /// Speed multiplier: (numerator, denominator)
    pub fn multiplier(&self) -> (i32, i32) {
        match self {
            MouseSpeed::Slow => (1, 2),       // 0.5x
            MouseSpeed::Normal => (1, 1),      // 1.0x
            MouseSpeed::Fast => (3, 2),        // 1.5x
            MouseSpeed::VeryFast => (2, 1),    // 2.0x
        }
    }
}

pub fn get_mouse_speed() -> MouseSpeed {
    MouseSpeed::from_u8(MOUSE_SPEED.load(Ordering::Relaxed))
}

pub fn set_mouse_speed(speed: MouseSpeed) {
    MOUSE_SPEED.store(speed as u8, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Mouse speed: {}", speed.label());
}

pub fn cycle_mouse_speed() {
    let current = MOUSE_SPEED.load(Ordering::Relaxed);
    let next = (current + 1) % 4;
    MOUSE_SPEED.store(next, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Mouse speed: {}", MouseSpeed::from_u8(next).label());
}

/// Apply mouse speed multiplier to raw delta
pub fn apply_mouse_speed(dx: i32, dy: i32) -> (i32, i32) {
    let (num, den) = get_mouse_speed().multiplier();
    ((dx * num) / den, (dy * num) / den)
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUMMARY
// ═══════════════════════════════════════════════════════════════════════════════

/// Get a summary string for the status bar
pub fn status_indicators() -> alloc::string::String {
    use alloc::string::String;
    use alloc::format;
    let mut parts: alloc::vec::Vec<&str> = alloc::vec::Vec::new();
    if is_high_contrast() { parts.push("HC"); }
    if is_sticky_keys() {
        parts.push("SK");
    }
    let fs = get_font_size();
    if fs != FontSize::Medium {
        // Will add inline below
    }
    if parts.is_empty() && fs == FontSize::Medium {
        return String::new();
    }
    let mut s = String::from("[");
    for (i, p) in parts.iter().enumerate() {
        if i > 0 { s.push(' '); }
        s.push_str(p);
    }
    if fs != FontSize::Medium {
        if !parts.is_empty() { s.push(' '); }
        s.push_str("F:");
        s.push_str(fs.label());
    }
    s.push(']');
    s
}
