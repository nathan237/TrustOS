//! GUI Window Manager
//! 
//! Modern windowing system with double-buffered rendering,
//! taskbar, mouse cursor, context menus, and scroll wheel support.
//! 
//! Design: TrustOS Modern Dark Theme
//! Inspired by: macOS, Windows 11, Terminal apps
//! 
//! Now using the TrustOS Graphics Engine for optimized rendering

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;
use crate::framebuffer::{self, COLOR_GREEN, COLOR_BRIGHT_GREEN, COLOR_DARK_GREEN, COLOR_WHITE, COLOR_BLACK};
use crate::apps::text_editor::{EditorState, render_editor};
use core::sync::atomic::{AtomicBool, Ordering};
use crate::math::fast_sqrt;

/// Get a color with high-contrast fallback
#[inline]
fn hc(normal: u32, hc_replacement: u32) -> u32 {
    crate::accessibility::a11y_color(normal, hc_replacement)
}

/// Module-level flag to signal desktop exit (accessible from run() and handle_menu_action)
static EXIT_DESKTOP_FLAG: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// JARVIS async inference — run on background thread, poll result in render loop
// ═══════════════════════════════════════════════════════════════════════════════
/// Pending JARVIS query (set by terminal, consumed by background thread)
static JARVIS_PENDING_QUERY: Mutex<Option<String>> = Mutex::new(None);
/// Completed JARVIS result lines (set by background thread, consumed by render loop)
static JARVIS_RESULT: Mutex<Option<Vec<String>>> = Mutex::new(None);
/// Flag: JARVIS is currently thinking
static JARVIS_BUSY: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// Browser async navigation — run HTTP fetch on background thread
// ═══════════════════════════════════════════════════════════════════════════════
/// Pending browser URL to navigate (set by keyboard handler, consumed by worker)
static BROWSER_PENDING_URL: Mutex<Option<String>> = Mutex::new(None);
/// Completed browser navigation result: Ok((final_url, status, headers, body)) or Err(msg)
static BROWSER_NAV_RESULT: Mutex<Option<Result<(String, u16, Vec<(String, String)>, Vec<u8>), String>>> = Mutex::new(None);
/// Flag: browser navigation in progress
static BROWSER_NAV_BUSY: AtomicBool = AtomicBool::new(false);

/// Background thread entry point for browser HTTP fetch
fn browser_nav_worker(_arg: u64) -> i32 {
    let url = {
        let mut pending = BROWSER_PENDING_URL.lock();
        pending.take()
    };
    let url = match url {
        Some(u) => u,
        None => {
            BROWSER_NAV_BUSY.store(false, Ordering::SeqCst);
            return 0;
        }
    };

    // Normalize URL
    let full_url = crate::browser::normalize_url(&url, "");
    crate::serial_println!("[BROWSER-BG] Fetching: {}", full_url);

    let result = if full_url.starts_with("https://") {
        match crate::netstack::https::get(&full_url) {
            Ok(r) => Ok((full_url, r.status_code, r.headers, r.body)),
            Err(e) => Err(alloc::format!("HTTPS error: {}", e)),
        }
    } else {
        match crate::netstack::http::get(&full_url) {
            Ok(r) => Ok((full_url, r.status_code, r.headers, r.body)),
            Err(e) => Err(alloc::format!("Network error: {}", e)),
        }
    };

    {
        let mut nav_result = BROWSER_NAV_RESULT.lock();
        *nav_result = Some(result);
    }
    BROWSER_NAV_BUSY.store(false, Ordering::SeqCst);
    0
}

/// Background thread entry point for JARVIS inference
fn jarvis_worker(_arg: u64) -> i32 {
    // Take the pending query
    let query = {
        let mut pending = JARVIS_PENDING_QUERY.lock();
        pending.take()
    };
    let query = match query {
        Some(q) => q,
        None => {
            JARVIS_BUSY.store(false, Ordering::SeqCst);
            return 0;
        }
    };

    // Run the JARVIS command through shell capture
    crate::shell::take_captured(); // clear stale
    crate::shell::CAPTURE_MODE.store(true, Ordering::SeqCst);
    crate::shell::execute_command(&query);
    crate::shell::CAPTURE_MODE.store(false, Ordering::SeqCst);
    let captured = crate::shell::take_captured();

    // Store result
    let mut lines = Vec::new();
    for line in captured.lines() {
        lines.push(String::from(line));
    }
    {
        let mut result = JARVIS_RESULT.lock();
        *result = Some(lines);
    }
    JARVIS_BUSY.store(false, Ordering::SeqCst);
    0
}


// ═══════════════════════════════════════════════════════════════════════════════
// 🎨 TRUSTOS OFFICIAL PALETTE - Matrix-like professional dark theme
// ═══════════════════════════════════════════════════════════════════════════════
// STRICT COMPLIANCE: No pure white (#FFFFFF), green accent hierarchy
// ═══════════════════════════════════════════════════════════════════════════════

// Primary backgrounds
const BG_DEEPEST: u32 = 0xFF050606;          // Deepest black
const BG_DARK: u32 = 0xFF070B09;             // Panel background
const BG_MEDIUM: u32 = 0xFF0A0F0C;           // Window background
const BG_LIGHT: u32 = 0xFF0D1310;            // Alternate/hover

// Green hierarchy (stratified)
const GREEN_PRIMARY: u32 = 0xFF00FF66;       // Bright accent, focus
const GREEN_SECONDARY: u32 = 0xFF00CC55;     // Normal text, icons
const GREEN_TERTIARY: u32 = 0xFF00AA44;      // Intermediate
const GREEN_MUTED: u32 = 0xFF008844;         // Borders, separators  
const GREEN_SUBTLE: u32 = 0xFF006633;        // Subtle elements
const GREEN_GHOST: u32 = 0xFF003B1A;         // Shadows, grid

// Chrome/silver from TrustOS logo — shining grey for borders
const CHROME_BRIGHT: u32 = 0xFFB0B2B0;       // Bright chrome highlight (logo highlight)
const CHROME_MID: u32 = 0xFF8C8E8C;          // Mid chrome (logo main grey)
const CHROME_DIM: u32 = 0xFF606260;          // Subtle chrome (logo shadow edge)
const CHROME_GHOST: u32 = 0xFF3A3C3A;        // Very faint chrome (unfocused borders)

// Status colors (minimal use)
const ACCENT_AMBER: u32 = 0xFFFFD166;        // Warnings
const ACCENT_RED: u32 = 0xFFFF5555;          // Errors (rare)
const ACCENT_BLUE: u32 = 0xFF4ECDC4;         // Info/links (minimal)

// UI Elements
const WINDOW_BG: u32 = 0xFF0A0F0C;
const WINDOW_CONTENT_BG: u32 = 0xFF070B09;
const TITLE_BAR_ACTIVE: u32 = 0xFF080C09;
const TITLE_BAR_INACTIVE: u32 = 0xFF060908;
const TITLE_BAR_ACCENT: u32 = GREEN_MUTED;

// Dock (left sidebar)
const DOCK_BG: u32 = 0xFF060908;
const DOCK_ITEM_HOVER: u32 = 0xFF0D1310;
const DOCK_ITEM_ACTIVE: u32 = 0xFF101815;

// Context menu
const MENU_BG: u32 = 0xFF080C09;
const MENU_HOVER: u32 = 0xFF0D1310;
const MENU_BORDER: u32 = 0xFF1A2A20;
const MENU_SEPARATOR: u32 = 0xFF1A2A20;

// Window control buttons (desaturated - professional)
const BTN_CLOSE: u32 = 0xFF3A2828;
const BTN_CLOSE_HOVER: u32 = 0xFFFF5555;
const BTN_MAXIMIZE: u32 = 0xFF2A2A20;
const BTN_MAXIMIZE_HOVER: u32 = 0xFFFFD166;
const BTN_MINIMIZE: u32 = 0xFF283028;
const BTN_MINIMIZE_HOVER: u32 = 0xFF00CC55;

// Text colors
const TEXT_PRIMARY: u32 = 0xFFE0E8E4;
const TEXT_SECONDARY: u32 = 0xFF8A9890;
const TEXT_ACCENT: u32 = 0xFF00CC55;

// Legacy aliases for compatibility
const TASKBAR_BG: u32 = DOCK_BG;
const TASKBAR_ACCENT: u32 = DOCK_ITEM_HOVER;
const BUTTON_RED: u32 = BTN_CLOSE;
const BUTTON_YELLOW: u32 = BTN_MAXIMIZE;
const BUTTON_GREEN_BTN: u32 = BTN_MINIMIZE;
const CONTEXT_MENU_BG: u32 = MENU_BG;
const CONTEXT_MENU_HOVER: u32 = MENU_HOVER;
const CONTEXT_MENU_BORDER: u32 = MENU_BORDER;
const DESKTOP_BG_TOP: u32 = BG_DEEPEST;
const DESKTOP_BG_BOTTOM: u32 = 0xFF020303;

// Layout constants (official spec — v2 visual overhaul)
const TASKBAR_HEIGHT: u32 = 48;
const TITLE_BAR_HEIGHT: u32 = 28;             // Classic title bar height
const WINDOW_BORDER_RADIUS: u32 = 8;          // Moderate rounded corners
const WINDOW_SHADOW_BLUR: u32 = 16;
const DOCK_ICON_SIZE: u32 = 32;               // Larger dock icons
const DOCK_WIDTH: u32 = 72;                   // Widened to fit label text (e.g. Settings)

// Animation state (minimal - no flashy effects)
const FADE_STEPS: u8 = 8;

// ═══════════════════════════════════════════════════════════════════════════════
// 🎬 ANIMATION SYSTEM - Smooth transitions for modern desktop feel
// ═══════════════════════════════════════════════════════════════════════════════

/// Global animation settings
static ANIMATIONS_ENABLED: Mutex<bool> = Mutex::new(true);
static ANIMATION_SPEED: Mutex<f32> = Mutex::new(1.0); // 1.0 = normal, 2.0 = fast, 0.5 = slow

/// Animation duration in frames (at 60 FPS)
const ANIM_DURATION_OPEN: u32 = 12;      // 200ms window open
const ANIM_DURATION_CLOSE: u32 = 8;      // 133ms window close
const ANIM_DURATION_MINIMIZE: u32 = 10;  // 166ms minimize
const ANIM_DURATION_MAXIMIZE: u32 = 10;  // 166ms maximize

/// Animation state for a window
#[derive(Clone, Copy, PartialEq)]
pub enum AnimationState {
    None,
    Opening,      // Scale up from center
    Closing,      // Scale down + fade out
    Minimizing,   // Move to taskbar
    Maximizing,   // Expand to full screen
    Restoring,    // Restore from maximized/minimized
}

/// Animation data for interpolation
#[derive(Clone)]
pub struct WindowAnimation {
    pub state: AnimationState,
    pub progress: f32,           // 0.0 to 1.0
    pub start_x: i32,
    pub start_y: i32,
    pub start_width: u32,
    pub start_height: u32,
    pub target_x: i32,
    pub target_y: i32,
    pub target_width: u32,
    pub target_height: u32,
    pub alpha: f32,              // 0.0 to 1.0 for fade effects
}

impl WindowAnimation {
    pub fn new() -> Self {
        Self {
            state: AnimationState::None,
            progress: 0.0,
            start_x: 0,
            start_y: 0,
            start_width: 0,
            start_height: 0,
            target_x: 0,
            target_y: 0,
            target_width: 0,
            target_height: 0,
            alpha: 1.0,
        }
    }
    
    /// Start opening animation
    pub fn start_open(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.state = AnimationState::Opening;
        self.progress = 0.0;
        // Start from center, small
        self.start_x = x + width as i32 / 2 - 10;
        self.start_y = y + height as i32 / 2 - 10;
        self.start_width = 20;
        self.start_height = 20;
        self.target_x = x;
        self.target_y = y;
        self.target_width = width;
        self.target_height = height;
        self.alpha = 0.0;
    }
    
    /// Start closing animation
    pub fn start_close(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.state = AnimationState::Closing;
        self.progress = 0.0;
        self.start_x = x;
        self.start_y = y;
        self.start_width = width;
        self.start_height = height;
        // End at center, small
        self.target_x = x + width as i32 / 2 - 10;
        self.target_y = y + height as i32 / 2 - 10;
        self.target_width = 20;
        self.target_height = 20;
        self.alpha = 1.0;
    }
    
    /// Start minimize animation
    pub fn start_minimize(&mut self, x: i32, y: i32, width: u32, height: u32, taskbar_x: i32, taskbar_y: i32) {
        self.state = AnimationState::Minimizing;
        self.progress = 0.0;
        self.start_x = x;
        self.start_y = y;
        self.start_width = width;
        self.start_height = height;
        self.target_x = taskbar_x;
        self.target_y = taskbar_y;
        self.target_width = 48;
        self.target_height = 32;
        self.alpha = 1.0;
    }
    
    /// Start maximize animation  
    pub fn start_maximize(&mut self, x: i32, y: i32, width: u32, height: u32, max_w: u32, max_h: u32) {
        self.state = AnimationState::Maximizing;
        self.progress = 0.0;
        self.start_x = x;
        self.start_y = y;
        self.start_width = width;
        self.start_height = height;
        self.target_x = 0;
        self.target_y = 0;
        self.target_width = max_w;
        self.target_height = max_h - TASKBAR_HEIGHT;
        self.alpha = 1.0;
    }
    
    /// Start restore animation
    pub fn start_restore(&mut self, curr_x: i32, curr_y: i32, curr_w: u32, curr_h: u32,
                         saved_x: i32, saved_y: i32, saved_w: u32, saved_h: u32) {
        self.state = AnimationState::Restoring;
        self.progress = 0.0;
        self.start_x = curr_x;
        self.start_y = curr_y;
        self.start_width = curr_w;
        self.start_height = curr_h;
        self.target_x = saved_x;
        self.target_y = saved_y;
        self.target_width = saved_w;
        self.target_height = saved_h;
        self.alpha = 1.0;
    }
    
    /// Update animation progress (call each frame)
    pub fn update(&mut self) -> bool {
        if self.state == AnimationState::None {
            return false;
        }
        
        let speed = *ANIMATION_SPEED.lock();
        let duration = match self.state {
            AnimationState::Opening => ANIM_DURATION_OPEN,
            AnimationState::Closing => ANIM_DURATION_CLOSE,
            AnimationState::Minimizing => ANIM_DURATION_MINIMIZE,
            AnimationState::Maximizing | AnimationState::Restoring => ANIM_DURATION_MAXIMIZE,
            AnimationState::None => return false,
        };
        
        let step = speed / duration as f32;
        self.progress += step;
        
        // Update alpha for fade effects
        match self.state {
            AnimationState::Opening => {
                self.alpha = ease_out_cubic(self.progress);
            }
            AnimationState::Closing | AnimationState::Minimizing => {
                self.alpha = 1.0 - ease_in_cubic(self.progress);
            }
            _ => {}
        }
        
        if self.progress >= 1.0 {
            self.progress = 1.0;
            let completed_state = self.state;
            self.state = AnimationState::None;
            return completed_state == AnimationState::Closing;
        }
        
        false // Animation still running
    }
    
    /// Get current interpolated position and size
    pub fn get_current(&self) -> (i32, i32, u32, u32, f32) {
        let t = match self.state {
            AnimationState::Opening | AnimationState::Restoring => ease_out_back(self.progress),
            AnimationState::Closing => ease_in_back(self.progress),
            AnimationState::Minimizing => ease_in_cubic(self.progress),
            AnimationState::Maximizing => ease_out_cubic(self.progress),
            AnimationState::None => 1.0,
        };
        
        let x = lerp_i32(self.start_x, self.target_x, t);
        let y = lerp_i32(self.start_y, self.target_y, t);
        let w = lerp_u32(self.start_width, self.target_width, t);
        let h = lerp_u32(self.start_height, self.target_height, t);
        
        (x, y, w, h, self.alpha)
    }
    
    /// Check if currently animating
    pub fn is_animating(&self) -> bool {
        self.state != AnimationState::None
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🎯 EASING FUNCTIONS - Smooth animation curves
// ═══════════════════════════════════════════════════════════════════════════════

/// Linear interpolation for i32
fn lerp_i32(a: i32, b: i32, t: f32) -> i32 {
    (a as f32 + (b - a) as f32 * t) as i32
}

/// Linear interpolation for u32
fn lerp_u32(a: u32, b: u32, t: f32) -> u32 {
    if a > b {
        (a as f32 - (a - b) as f32 * t) as u32
    } else {
        (a as f32 + (b - a) as f32 * t) as u32
    }
}

/// Ease-out cubic: decelerating end
fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t)
}

/// Ease-in cubic: accelerating start
fn ease_in_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t
}

/// Ease-out back: overshoot then settle (bouncy opening)
fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1: f32 = 1.70158;
    let c3 = c1 + 1.0;
    let tm1 = t - 1.0;
    1.0 + c3 * tm1 * tm1 * tm1 + c1 * tm1 * tm1
}

/// Ease-in back: pull back before moving (anticipation)
fn ease_in_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1: f32 = 1.70158;
    let c3 = c1 + 1.0;
    c3 * t * t * t - c1 * t * t
}

/// Check if animations are enabled
pub fn animations_enabled() -> bool {
    *ANIMATIONS_ENABLED.lock()
}

/// Enable/disable animations
pub fn set_animations_enabled(enabled: bool) {
    *ANIMATIONS_ENABLED.lock() = enabled;
    crate::serial_println!("[ANIM] Animations {}", if enabled { "ENABLED" } else { "DISABLED" });
}

/// Toggle animations on/off
pub fn toggle_animations() {
    let mut enabled = ANIMATIONS_ENABLED.lock();
    *enabled = !*enabled;
    crate::serial_println!("[ANIM] Animations {}", if *enabled { "ENABLED" } else { "DISABLED" });
}

/// Set animation speed multiplier
pub fn set_animation_speed(speed: f32) {
    *ANIMATION_SPEED.lock() = speed.clamp(0.25, 4.0);
    crate::serial_println!("[ANIM] Speed set to {}x", speed);
}

/// Get animation speed
pub fn get_animation_speed() -> f32 {
    *ANIMATION_SPEED.lock()
}

/// Window ID counter
static NEXT_WINDOW_ID: Mutex<u32> = Mutex::new(1);

/// Context menu item
#[derive(Clone)]
pub struct ContextMenuItem {
    pub label: String,
    pub action: ContextAction,
}

/// Context menu actions
#[derive(Clone, Copy, PartialEq)]
pub enum ContextAction {
    Open,
    OpenWith,
    Delete,
    Rename,
    Properties,
    Refresh,
    NewFile,
    NewFolder,
    CopyPath,
    Cut,
    Copy,
    Paste,
    ViewLargeIcons,
    ViewSmallIcons,
    ViewList,
    SortByName,
    SortByDate,
    SortBySize,
    Personalize,
    TerminalHere,
    Cancel,
}

/// Per-cell pixel override: 8×16 = 128 ARGB pixels.
/// Each pixel is independently colored. 0x00000000 = transparent (skip).
/// Used to render arbitrary pixel content through the matrix rain.
#[derive(Clone)]
pub struct CellPixels {
    pub pixels: [u32; 128],  // row-major: pixels[row * 8 + col]
}

impl CellPixels {
    pub const fn blank() -> Self {
        CellPixels { pixels: [0; 128] }
    }

    /// Initialize from a glyph: lit pixels get `color`, unlit stay 0 (transparent).
    pub fn from_glyph(c: char, color: u32) -> Self {
        let glyph = crate::framebuffer::font::get_glyph(c);
        let mut px = [0u32; 128];
        for row in 0..16 {
            let bits = glyph[row];
            for bit in 0..8u8 {
                if bits & (0x80 >> bit) != 0 {
                    px[row * 8 + bit as usize] = color;
                }
            }
        }
        CellPixels { pixels: px }
    }

    /// Set a single pixel (0..7, 0..15) to a color.
    #[inline]
    pub fn set(&mut self, x: u8, y: u8, color: u32) {
        if x < 8 && y < 16 {
            self.pixels[y as usize * 8 + x as usize] = color;
        }
    }

    /// Get a single pixel.
    #[inline]
    pub fn get(&self, x: u8, y: u8) -> u32 {
        if x < 8 && y < 16 { self.pixels[y as usize * 8 + x as usize] } else { 0 }
    }

    /// Fill all pixels with one color.
    pub fn fill(&mut self, color: u32) {
        self.pixels = [color; 128];
    }
}

/// Screen-space projection zone: a fixed rectangular area where the matrix rain
/// reveals a colorful image as it falls through the zone.
/// Unlike CellPixels (trail-indexed, scrolls with rain), this is anchored to
/// screen coordinates. Rain intensity modulates how brightly each image pixel
/// is revealed — creating a holographic reveal effect.
pub struct MatrixProjection {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,  // row-major ARGB: pixels[y * width + x]
    pub active: bool,
}

impl MatrixProjection {
    pub const fn empty() -> Self {
        MatrixProjection {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            pixels: Vec::new(),
            active: false,
        }
    }

    /// Generate a colorful procedural test image: plasma gradient with geometric shapes.
    pub fn generate_test_image(width: u32, height: u32) -> Vec<u32> {
        let w = width as usize;
        let h = height as usize;
        let mut pixels = vec![0u32; w * h];

        for py in 0..h {
            for px in 0..w {
                // Normalized coordinates (0.0 - 1.0)
                let u = px as f32 / w as f32;
                let v = py as f32 / h as f32;
                // Center-relative (-1.0 to 1.0)
                let cx = u * 2.0 - 1.0;
                let cy = v * 2.0 - 1.0;

                // ── Plasma base: overlapping sine waves ──
                // Fast approximate sqrt for no_std (Newton-Raphson, 2 iterations)
                fn fast_sqrt(x: f32) -> f32 {
                    if x <= 0.0 { return 0.0; }
                    let mut guess = x * 0.5;
                    guess = 0.5 * (guess + x / guess);
                    guess = 0.5 * (guess + x / guess);
                    guess
                }
                let d = fast_sqrt(cx * cx + cy * cy); // distance from center

                // Fast integer-approximated sine using parabolic approximation
                fn fast_sin(x: f32) -> f32 {
                    // Reduce to [-PI, PI]
                    let x = x % 6.2832;
                    let x = if x > 3.1416 { x - 6.2832 } else if x < -3.1416 { x + 6.2832 } else { x };
                    // Parabolic approximation
                    if x < 0.0 {
                        1.27323954 * x + 0.405284735 * x * x
                    } else {
                        1.27323954 * x - 0.405284735 * x * x
                    }
                }

                let s1 = fast_sin(u * 10.0 + v * 6.0) * 0.5 + 0.5;
                let s2 = fast_sin(d * 12.0 - v * 4.0) * 0.5 + 0.5;
                let s3 = fast_sin((cx + cy) * 8.0) * 0.5 + 0.5;

                let mut r = (s1 * 0.5 + s2 * 0.3 + s3 * 0.2).min(1.0);
                let mut g = (s2 * 0.5 + s3 * 0.3 + s1 * 0.2).min(1.0);
                let mut b = (s3 * 0.5 + s1 * 0.3 + s2 * 0.2).min(1.0);

                // ── Geometric shapes overlay ──

                // Central diamond: bright cyan
                let diamond = cx.abs() + cy.abs();
                if diamond < 0.35 {
                    let t = 1.0 - diamond / 0.35;
                    r = r * (1.0 - t * 0.8) + 0.1 * t;
                    g = g * (1.0 - t * 0.5) + 1.0 * t * 0.5 + g * t * 0.5;
                    b = b * (1.0 - t * 0.5) + 1.0 * t * 0.5 + b * t * 0.5;
                }

                // Concentric rings: magenta glow
                let ring1 = (d - 0.5).abs();
                if ring1 < 0.04 {
                    let t = 1.0 - ring1 / 0.04;
                    r = (r + t * 0.9).min(1.0);
                    g = g * (1.0 - t * 0.6);
                    b = (b + t * 0.8).min(1.0);
                }
                let ring2 = (d - 0.75).abs();
                if ring2 < 0.03 {
                    let t = 1.0 - ring2 / 0.03;
                    r = (r + t * 0.3).min(1.0);
                    g = (g + t * 0.9).min(1.0);
                    b = g * (1.0 - t * 0.3);
                }

                // Corner accents: warm orange triangles
                let top_left = (1.0 - u) + (1.0 - v);
                if top_left > 1.7 {
                    let t = ((top_left - 1.7) / 0.3).min(1.0);
                    r = (r + t * 0.6).min(1.0);
                    g = (g + t * 0.3).min(1.0);
                    b = b * (1.0 - t * 0.4);
                }
                let bot_right = u + v;
                if bot_right > 1.7 {
                    let t = ((bot_right - 1.7) / 0.3).min(1.0);
                    r = r * (1.0 - t * 0.3);
                    g = (g + t * 0.4).min(1.0);
                    b = (b + t * 0.7).min(1.0);
                }

                // Cross-hair lines through center: white with glow
                if cy.abs() < 0.012 || cx.abs() < 0.012 {
                    r = (r * 0.5 + 0.5).min(1.0);
                    g = (g * 0.5 + 0.5).min(1.0);
                    b = (b * 0.5 + 0.5).min(1.0);
                }

                // Vignette: darken edges
                let vig: f32 = if (1.0 - d * 0.7) > 0.0 { 1.0 - d * 0.7 } else { 0.0 };
                r *= vig;
                g *= vig;
                b *= vig;

                // Boost saturation a bit
                r = (r * 1.3).min(1.0);
                g = (g * 1.2).min(1.0);
                b = (b * 1.3).min(1.0);

                let ri = (r * 255.0) as u32;
                let gi = (g * 255.0) as u32;
                let bi = (b * 255.0) as u32;
                pixels[py * w + px] = 0xFF000000 | (ri << 16) | (gi << 8) | bi;
            }
        }
        pixels
    }
}

/// Context menu structure
#[derive(Clone)]
pub struct ContextMenu {
    pub visible: bool,
    pub x: i32,
    pub y: i32,
    pub items: Vec<ContextMenuItem>,
    pub selected_index: usize,
    pub target_icon: Option<usize>,  // Index of icon right-clicked
    pub target_file: Option<String>, // File path right-clicked
}

/// Desktop icon structure
#[derive(Clone)]
pub struct DesktopIcon {
    pub name: String,
    pub icon_type: crate::icons::IconType,
    pub x: u32,
    pub y: u32,
    pub action: IconAction,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IconAction {
    OpenTerminal,
    OpenFileManager,
    OpenSettings,
    OpenAbout,
    OpenMusicPlayer,
    OpenCalculator,
    OpenNetwork,
    OpenGame,
    OpenEditor,
    OpenGL3D,
    OpenBrowser,
    OpenModelEditor,
    OpenGame3D,
    #[cfg(feature = "emulators")]
    OpenNes,
    #[cfg(feature = "emulators")]
    OpenGameBoy,
    #[cfg(feature = "emulators")]
    OpenGameLab,
}

/// Window type for content
#[derive(Clone, Copy, PartialEq)]
pub enum WindowType {
    Terminal,
    SystemInfo,
    About,
    Empty,
    Calculator,
    FileManager,
    TextEditor,
    NetworkInfo,
    Settings,
    ImageViewer,
    HexViewer,
    FileAssociations,
    Demo3D,  // New: 3D graphics demo
    Game,    // Snake game
    Browser, // Web browser
    ModelEditor, // TrustEdit 3D model editor
    Game3D,  // 3D FPS raycasting game
    Chess,   // Chess game vs AI
    Chess3D, // 3D Matrix-style chess
    #[cfg(feature = "emulators")]
    NesEmu,  // NES emulator
    #[cfg(feature = "emulators")]
    GameBoyEmu, // Game Boy emulator
    #[cfg(feature = "emulators")]
    GameBoyInput, // Game Boy input display (separate window)
    BinaryViewer, // TrustView binary analyzer
    LabMode,      // TrustLab introspection laboratory
    #[cfg(feature = "emulators")]
    GameLab,      // Game Boy emulator analysis dashboard
    MusicPlayer,  // Music player widget with pulse wave
}

/// Window structure
#[derive(Clone)]
pub struct Window {
    pub id: u32,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub visible: bool,
    pub focused: bool,
    pub minimized: bool,
    pub maximized: bool,
    pub dragging: bool,
    pub resizing: ResizeEdge,
    pub drag_offset_x: i32,
    pub drag_offset_y: i32,
    // Saved position before maximize
    pub saved_x: i32,
    pub saved_y: i32,
    pub saved_width: u32,
    pub saved_height: u32,
    pub window_type: WindowType,
    pub content: Vec<String>,
    pub file_path: Option<String>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    // Animation state
    pub animation: WindowAnimation,
    pub pending_close: bool,  // Window should close after animation
}

/// Resize edge being dragged
#[derive(Clone, Copy, PartialEq)]
pub enum ResizeEdge {
    None,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Cursor display mode (changes dynamically based on context)
#[derive(Clone, Copy, PartialEq)]
enum CursorMode {
    Arrow,
    ResizeH,     // Horizontal ←→
    ResizeV,     // Vertical ↕
    ResizeNWSE,  // Diagonal ↘↖
    ResizeNESW,  // Diagonal ↗↙
    Grab,        // Grabbing/dragging
}

/// File manager view mode
#[derive(Clone, Copy, PartialEq)]
pub enum FileManagerViewMode {
    List,
    IconGrid,
    Details,
    Tiles,
}

/// File manager sidebar section
#[derive(Clone, Copy, PartialEq)]
pub enum FmSidebarSection {
    QuickAccess,
    ThisPC,
}

/// File manager state — Windows Explorer-like persistent state per window
pub struct FileManagerState {
    /// Navigation history (paths visited)
    pub history: Vec<String>,
    /// Current index in history (for back/forward)
    pub history_idx: usize,
    /// Sidebar collapsed or not
    pub sidebar_collapsed: bool,
    /// Sidebar width in pixels
    pub sidebar_width: u32,
    /// Sidebar scroll offset
    pub sidebar_scroll: usize,
    /// Selected sidebar item (-1 = none)
    pub sidebar_selected: i32,
    /// Quick access pinned paths
    pub quick_access: Vec<(String, String)>, // (display_name, path)
    /// Sort column (0=name, 1=type, 2=size, 3=program)
    pub sort_column: u8,
    /// Sort ascending
    pub sort_ascending: bool,
    /// Hover row index (for mouse hover highlight)
    pub hover_index: Option<usize>,
    /// Search query (for the search box)
    pub search_query: String,
    /// Search box focused 
    pub search_focused: bool,
    /// Column widths (resizable) - name, type, size, program
    pub col_widths: [u32; 4],
}

impl FileManagerState {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            history_idx: 0,
            sidebar_collapsed: false,
            sidebar_width: 180,
            sidebar_scroll: 0,
            sidebar_selected: -1,
            quick_access: vec![
                (String::from("Desktop"), String::from("/")),
                (String::from("Documents"), String::from("/documents")),
                (String::from("Downloads"), String::from("/downloads")),
                (String::from("Music"), String::from("/music")),
                (String::from("Pictures"), String::from("/pictures")),
            ],
            sort_column: 0,
            sort_ascending: true,
            hover_index: None,
            search_query: String::new(),
            search_focused: false,
            col_widths: [200, 80, 80, 120],
        }
    }
    
    pub fn push_history(&mut self, path: &str) {
        // Trim forward history when navigating to new path
        if self.history_idx + 1 < self.history.len() {
            self.history.truncate(self.history_idx + 1);
        }
        self.history.push(String::from(path));
        self.history_idx = self.history.len() - 1;
    }
    
    pub fn can_go_back(&self) -> bool {
        self.history_idx > 0
    }
    
    pub fn can_go_forward(&self) -> bool {
        self.history_idx + 1 < self.history.len()
    }
    
    pub fn go_back(&mut self) -> Option<&str> {
        if self.history_idx > 0 {
            self.history_idx -= 1;
            Some(&self.history[self.history_idx])
        } else {
            None
        }
    }
    
    pub fn go_forward(&mut self) -> Option<&str> {
        if self.history_idx + 1 < self.history.len() {
            self.history_idx += 1;
            Some(&self.history[self.history_idx])
        } else {
            None
        }
    }
}

/// Image viewer state — holds decoded pixel data for BMP display
pub struct ImageViewerState {
    pub pixels: Vec<u32>,
    pub img_width: u32,
    pub img_height: u32,
    pub zoom: u32,     // percentage: 100 = 1:1
    pub pan_x: i32,
    pub pan_y: i32,
}

impl ImageViewerState {
    pub fn new() -> Self {
        Self { pixels: Vec::new(), img_width: 0, img_height: 0, zoom: 100, pan_x: 0, pan_y: 0 }
    }
}

/// File clipboard entry for copy/paste in file manager
pub struct FileClipboardEntry {
    pub path: String,
    pub name: String,
    pub is_cut: bool,
}

/// Drag-and-drop state
pub struct DragState {
    pub source_path: String,
    pub filename: String,
    pub is_dir: bool,
    pub start_x: i32,
    pub start_y: i32,
    pub current_x: i32,
    pub current_y: i32,
    pub source_window_id: u32,
    pub active: bool,
}

impl Window {
    pub fn new(title: &str, x: i32, y: i32, width: u32, height: u32, wtype: WindowType) -> Self {
        let mut id_lock = NEXT_WINDOW_ID.lock();
        let id = *id_lock;
        *id_lock += 1;
        
        Window {
            id,
            title: String::from(title),
            x,
            y,
            width,
            height,
            min_width: 200,
            min_height: 150,
            visible: true,
            focused: false,
            minimized: false,
            maximized: false,
            dragging: false,
            resizing: ResizeEdge::None,
            drag_offset_x: 0,
            drag_offset_y: 0,
            saved_x: x,
            saved_y: y,
            saved_width: width,
            saved_height: height,
            window_type: wtype,
            content: Vec::new(),
            file_path: None,
            selected_index: 0,
            scroll_offset: 0,
            animation: WindowAnimation::new(),
            pending_close: false,
        }
    }
    
    /// Start open animation if animations are enabled
    pub fn animate_open(&mut self) {
        if animations_enabled() {
            self.animation.start_open(self.x, self.y, self.width, self.height);
        }
    }
    
    /// Start close animation if animations are enabled
    pub fn animate_close(&mut self) -> bool {
        if animations_enabled() {
            self.animation.start_close(self.x, self.y, self.width, self.height);
            self.pending_close = true;
            true // Animation started
        } else {
            false // Close immediately
        }
    }
    
    /// Start minimize animation if animations are enabled
    pub fn animate_minimize(&mut self, taskbar_y: i32) {
        if animations_enabled() {
            let taskbar_x = 100; // Approximate position in taskbar
            self.animation.start_minimize(self.x, self.y, self.width, self.height, taskbar_x, taskbar_y);
        }
    }
    
    /// Start maximize animation if animations are enabled
    pub fn animate_maximize(&mut self, screen_w: u32, screen_h: u32) {
        if animations_enabled() {
            self.animation.start_maximize(self.x, self.y, self.width, self.height, screen_w, screen_h);
        }
    }
    
    /// Start restore animation if animations are enabled
    pub fn animate_restore(&mut self) {
        if animations_enabled() {
            self.animation.start_restore(
                self.x, self.y, self.width, self.height,
                self.saved_x, self.saved_y, self.saved_width, self.saved_height
            );
        }
    }
    
    /// Update animation state (call each frame)
    pub fn update_animation(&mut self) -> bool {
        if self.animation.is_animating() {
            let should_close = self.animation.update();
            
            // If animation completed and it was maximizing/restoring, apply final state
            if !self.animation.is_animating() && !should_close {
                // Animation completed normally
            }
            
            return should_close && self.pending_close;
        }
        false
    }
    
    /// Get render position (considering animation)
    pub fn get_render_bounds(&self) -> (i32, i32, u32, u32, f32) {
        if self.animation.is_animating() {
            self.animation.get_current()
        } else {
            (self.x, self.y, self.width, self.height, 1.0)
        }
    }
    
    /// Check if point is inside window
    pub fn contains(&self, px: i32, py: i32) -> bool {
        if self.minimized { return false; }
        px >= self.x && px < self.x + self.width as i32 &&
        py >= self.y && py < self.y + self.height as i32
    }
    
    /// Check if point is in title bar
    pub fn in_title_bar(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width as i32 - 90 &&
        py >= self.y && py < self.y + TITLE_BAR_HEIGHT as i32
    }
    
    /// Check if point is on close button (Windows-style: rightmost, top-right)
    pub fn on_close_button(&self, px: i32, py: i32) -> bool {
        let btn_w = 28i32;
        let btn_h = TITLE_BAR_HEIGHT as i32;
        let bx = self.x + self.width as i32 - btn_w - 1;
        let by = self.y + 1;
        px >= bx && px < bx + btn_w && py >= by && py < by + btn_h
    }
    
    /// Check if point is on maximize button (Windows-style: second from right)
    pub fn on_maximize_button(&self, px: i32, py: i32) -> bool {
        let btn_w = 28i32;
        let btn_h = TITLE_BAR_HEIGHT as i32;
        let bx = self.x + self.width as i32 - btn_w * 2 - 1;
        let by = self.y + 1;
        px >= bx && px < bx + btn_w && py >= by && py < by + btn_h
    }
    
    /// Check if point is on minimize button (Windows-style: third from right)
    pub fn on_minimize_button(&self, px: i32, py: i32) -> bool {
        let btn_w = 28i32;
        let btn_h = TITLE_BAR_HEIGHT as i32;
        let bx = self.x + self.width as i32 - btn_w * 3 - 1;
        let by = self.y + 1;
        px >= bx && px < bx + btn_w && py >= by && py < by + btn_h
    }
    
    /// Check if point is on resize edge
    pub fn on_resize_edge(&self, px: i32, py: i32) -> ResizeEdge {
        if self.maximized { return ResizeEdge::None; }
        
        let resize_margin = 12i32;
        let left_edge = self.x;
        let right_edge = self.x + self.width as i32;
        let top_edge = self.y;
        let bottom_edge = self.y + self.height as i32;
        
        let on_left = px >= left_edge && px < left_edge + resize_margin;
        let on_right = px >= right_edge - resize_margin && px < right_edge;
        let on_top = py >= top_edge && py < top_edge + resize_margin;
        let on_bottom = py >= bottom_edge - resize_margin && py < bottom_edge;
        
        if on_top && on_left { ResizeEdge::TopLeft }
        else if on_top && on_right { ResizeEdge::TopRight }
        else if on_bottom && on_left { ResizeEdge::BottomLeft }
        else if on_bottom && on_right { ResizeEdge::BottomRight }
        else if on_left { ResizeEdge::Left }
        else if on_right { ResizeEdge::Right }
        else if on_top { ResizeEdge::Top }
        else if on_bottom { ResizeEdge::Bottom }
        else { ResizeEdge::None }
    }
    
    /// Toggle maximize state
    pub fn toggle_maximize(&mut self, screen_width: u32, screen_height: u32) {
        if self.maximized {
            // Restore
            self.x = self.saved_x;
            self.y = self.saved_y;
            self.width = self.saved_width;
            self.height = self.saved_height;
            self.maximized = false;
        } else {
            // Save current position
            self.saved_x = self.x;
            self.saved_y = self.y;
            self.saved_width = self.width;
            self.saved_height = self.height;
            // Maximize to usable area (right of dock, above taskbar)
            self.x = DOCK_WIDTH as i32;
            self.y = 0;
            self.width = screen_width.saturating_sub(DOCK_WIDTH);
            self.height = screen_height.saturating_sub(TASKBAR_HEIGHT);
            self.maximized = true;
        }
    }
}

/// Desktop manager
use crate::graphics::{compositor, Compositor, CompositorTheme, WindowSurface, Easing};

// ═══════════════════════════════════════════════════════════════════════════════
// � MUSIC PLAYER STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Playback state machine
#[derive(Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Music player widget state — manages non-blocking audio playback + FFT visualization.
pub struct MusicPlayerState {
    pub state: PlaybackState,
    /// Decoded PCM (48kHz stereo i16)
    pub audio: Option<Vec<i16>>,
    /// Current song title
    pub song_title: String,
    /// Current track index on the audio disk
    pub current_track: usize,
    /// Total tracks available on disk
    pub num_tracks: usize,
    /// DMA write cursor (samples written so far)
    pub write_cursor: usize,
    /// Which DMA half was last refilled (0 or 1)
    pub last_half: u32,
    /// Audio exhausted flag
    pub audio_exhausted: bool,
    /// Total DMA-consumed samples (stereo i16, incremented on each half-buffer flip)
    pub consumed_samples: usize,
    /// DMA buffer capacity in i16 samples (cached after start)
    pub dma_cap: usize,
    /// Visual frame counter (local)
    pub vis_frame: u32,
    /// LPIB-based elapsed milliseconds (sync'd to hardware)
    pub elapsed_ms: u64,
    /// Base offset for seek (added to LPIB-derived time)
    pub seek_base_ms: u64,
    /// Total duration in ms
    pub total_ms: u64,
    /// FFT scratch buffers
    pub fft_re: [f32; 1024],
    pub fft_im: [f32; 1024],
    /// Peak RMS for auto-gain
    pub peak_rms: f32,
    /// Smoothed band energies (sub-bass, bass, mid, treble)
    pub sub_bass: f32,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    /// Beat pulse (0.0–1.0)
    pub beat: f32,
    /// Smoothed overall energy
    pub energy: f32,
    /// Previous energy for onset detection
    pub prev_energy: f32,
    /// Energy history ring buffer for adaptive threshold
    pub energy_hist: [f32; 43],
    pub hist_idx: usize,
    pub hist_count: usize,
    /// Waveform ring buffer (128 samples for pulse visualization)
    pub waveform: [f32; 128],
    pub wave_idx: usize,
    /// Volume level (0–100)
    pub volume: u32,
    /// Audio/visual sync offset in ms (positive = shift visual later, negative = shift visual earlier)
    pub av_offset_ms: i32,
    /// True when playing procedural audio (short loop that repeats indefinitely)
    pub is_looping: bool,
    /// Track names loaded from disk (for clickable list)
    pub track_names: Vec<String>,
    /// Scroll offset for track list
    pub track_list_scroll: usize,
}

impl MusicPlayerState {
    pub fn new() -> Self {
        Self {
            state: PlaybackState::Stopped,
            audio: None,
            song_title: String::from("No Track"),
            current_track: 0,
            num_tracks: 0,
            write_cursor: 0,
            last_half: 0,
            audio_exhausted: false,
            consumed_samples: 0,
            dma_cap: 0,
            vis_frame: 0,
            elapsed_ms: 0,
            seek_base_ms: 0,
            total_ms: 0,
            fft_re: [0.0; 1024],
            fft_im: [0.0; 1024],
            peak_rms: 1.0,
            sub_bass: 0.0,
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            beat: 0.0,
            energy: 0.0,
            prev_energy: 0.0,
            energy_hist: [0.0; 43],
            hist_idx: 0,
            hist_count: 0,
            waveform: [0.0; 128],
            wave_idx: 0,
            volume: 75,
            av_offset_ms: 0,
            is_looping: false,
            track_names: Vec::new(),
            track_list_scroll: 0,
        }
    }

    /// Load track names from disk for the UI list.
    pub fn load_track_list(&mut self) {
        self.track_names = crate::trustdaw::disk_audio::get_track_names();
        self.num_tracks = self.track_names.len();
        crate::serial_println!("[MUSIC] Track list: {} tracks", self.num_tracks);
        for (i, name) in self.track_names.iter().enumerate() {
            crate::serial_println!("[MUSIC]   {}: {}", i, name);
        }
    }

    /// Start playback of the current track (or default).
    pub fn play_untitled2(&mut self) {
        self.play_track(self.current_track);
    }

    /// Play a specific track by index.
    pub fn play_track(&mut self, track_idx: usize) {
        // Fully stop and release old audio buffer BEFORE loading new track.
        // This frees ~30-50MB of PCM data, preventing OOM when allocating the new track.
        self.stop();

        // Detect track count on first play
        if self.num_tracks == 0 {
            self.load_track_list();
        }

        if self.num_tracks == 0 {
            self.song_title = String::from("No tracks found");
            crate::serial_println!("[MUSIC] No tracks available on disk");
            return;
        }

        let idx = track_idx % self.num_tracks;
        self.current_track = idx;

        crate::serial_println!("[MUSIC] Loading track {} — heap free: {} KB",
            idx, crate::memory::heap::free() / 1024);

        // Load track from TWAV disk
        match crate::trustdaw::disk_audio::load_track_from_disk(idx) {
            Ok((raw_wav, name)) => {
                // Debug: log first 12 bytes to verify WAV header
                if raw_wav.len() >= 12 {
                    crate::serial_println!("[MUSIC] Track {} raw header: {:02X} {:02X} {:02X} {:02X} ... {:02X} {:02X} {:02X} {:02X}",
                        idx, raw_wav[0], raw_wav[1], raw_wav[2], raw_wav[3],
                        raw_wav[8], raw_wav[9], raw_wav[10], raw_wav[11]);
                }
                match crate::trustdaw::audio_viz::decode_wav_to_pcm(&raw_wav) {
                    Ok(audio) => {
                        crate::serial_println!("[MUSIC] Decoded track {}: '{}' → {} samples", idx, name, audio.len());
                        // Drop raw WAV to free memory before playback setup
                        drop(raw_wav);
                        self.start_playback_with_audio(audio, &name);
                        return;
                    }
                    Err(e) => {
                        crate::serial_println!("[MUSIC] Track {} decode error: {}", idx, e);
                    }
                }
            }
            Err(e) => {
                crate::serial_println!("[MUSIC] Track {} load error: {}", idx, e);
            }
        }

        // If we get here, loading failed
        let name = if idx < self.track_names.len() {
            self.track_names[idx].clone()
        } else {
            alloc::format!("Track {}", idx + 1)
        };
        self.song_title = alloc::format!("{} (load failed)", name);
    }

    /// Switch to next track and start playback.
    pub fn next_track(&mut self) {
        if self.num_tracks > 1 {
            let next = (self.current_track + 1) % self.num_tracks;
            self.play_track(next);
        } else {
            // Only 1 track (or procedural) — restart from beginning
            self.play_track(self.current_track);
        }
    }

    /// Switch to previous track and start playback.
    pub fn prev_track(&mut self) {
        if self.num_tracks > 1 {
            let prev = if self.current_track == 0 { self.num_tracks - 1 } else { self.current_track - 1 };
            self.play_track(prev);
        } else {
            // Only 1 track — restart from beginning
            self.play_track(self.current_track);
        }
    }

    /// Shared playback setup for decoded audio.
    fn start_playback_with_audio(&mut self, audio: Vec<i16>, title: &str) {
        self.song_title = String::from(title);
        let total_frames = audio.len() / 2;
        self.total_ms = (total_frames as u64 * 1000) / 48000;

        // Init HDA audio (idempotent)
        crate::audio::init().ok();

        // Get DMA buffer capacity
        let dma_cap = crate::drivers::hda::get_dma_buffer_info()
            .map(|(_, c)| c)
            .unwrap_or(0);
        if dma_cap == 0 {
            crate::serial_println!("[MUSIC] No DMA buffer available");
            return;
        }

        // Reset stream fully: stop, SRST (clears LPIB), reconfigure
        let _ = crate::drivers::hda::stop();
        crate::drivers::hda::reset_stream();

        // Fill initial DMA buffer and start playback
        let initial = audio.len().min(dma_cap);
        if let Ok(()) = crate::drivers::hda::start_looped_playback(&audio[0..initial]) {
            self.write_cursor = initial;
            self.dma_cap = dma_cap;
            self.audio = Some(audio);
            self.state = PlaybackState::Playing;
            self.audio_exhausted = false;
            self.consumed_samples = 0;
            self.seek_base_ms = 0;
            self.vis_frame = 0;
            self.elapsed_ms = 0;

            // Read actual LPIB to sync half-buffer tracking
            let lpib = crate::drivers::hda::get_playback_position();
            let full_bytes = (dma_cap * 2) as u32;
            let half_bytes = full_bytes / 2;
            let lpib_clamped = if lpib >= full_bytes { 0 } else { lpib };
            self.last_half = if lpib_clamped < half_bytes { 0 } else { 1 };

            // Apply saved volume
            let _ = crate::drivers::hda::set_volume(self.volume.min(100) as u8);

            // Reset FFT / beat state
            self.peak_rms = 1.0;
            self.sub_bass = 0.0;
            self.bass = 0.0;
            self.mid = 0.0;
            self.treble = 0.0;
            self.beat = 0.0;
            self.energy = 0.0;
            self.prev_energy = 0.0;
            self.energy_hist = [0.0; 43];
            self.hist_idx = 0;
            self.hist_count = 0;
            self.waveform = [0.0; 128];
            self.wave_idx = 0;
            crate::serial_println!("[MUSIC] Playing '{}', {}ms, DMA={}", self.song_title, self.total_ms, dma_cap);
        } else {
            crate::serial_println!("[MUSIC] start_looped_playback failed");
        }
    }

    /// Stop playback
    pub fn stop(&mut self) {
        if self.state != PlaybackState::Stopped {
            let _ = crate::drivers::hda::stop();
            crate::drivers::hda::reset_stream();
            self.state = PlaybackState::Stopped;
            self.audio = None;
            self.write_cursor = 0;
            self.consumed_samples = 0;
            self.dma_cap = 0;
            self.seek_base_ms = 0;
            self.vis_frame = 0;
            self.elapsed_ms = 0;
            self.is_looping = false;
            // Decay visual state
            self.beat = 0.0;
            self.energy = 0.0;
            self.sub_bass = 0.0;
            self.bass = 0.0;
            self.mid = 0.0;
            self.treble = 0.0;
            crate::serial_println!("[MUSIC] Stopped");
        }
    }

    /// Toggle pause/resume
    pub fn toggle_pause(&mut self) {
        match self.state {
            PlaybackState::Playing => {
                let _ = crate::drivers::hda::stop();
                self.state = PlaybackState::Paused;
                crate::serial_println!("[MUSIC] Paused at {}ms", self.elapsed_ms);
            }
            PlaybackState::Paused => {
                // Resume by refilling DMA from current position and restarting
                self.resume_from_current_pos();
            }
            _ => {}
        }
    }

    /// Resume playback from current elapsed_ms position
    /// Refills DMA buffer from the audio at that position and restarts HDA.
    fn resume_from_current_pos(&mut self) {
        // Take audio out temporarily to avoid clone (we put it back after)
        let audio = match self.audio.take() {
            Some(a) => a,
            None => return,
        };
        let dma_cap = crate::drivers::hda::get_dma_buffer_info()
            .map(|(_, c)| c)
            .unwrap_or(0);
        if dma_cap == 0 {
            self.audio = Some(audio);
            return;
        }

        // Compute audio sample position from elapsed_ms
        let sample_pos = ((self.elapsed_ms as usize * 48000 * 2) / 1000).min(audio.len());

        // Stop + full stream reset (SRST clears LPIB to 0, reconfigures stream)
        let _ = crate::drivers::hda::stop();
        crate::drivers::hda::reset_stream();

        // Fill DMA buffer from new position and start
        let initial = audio.len().saturating_sub(sample_pos).min(dma_cap);
        if initial == 0 {
            self.audio = Some(audio);
            self.stop();
            return;
        }
        if let Ok(()) = crate::drivers::hda::start_looped_playback(&audio[sample_pos..sample_pos + initial]) {
            self.write_cursor = sample_pos + initial;
            self.dma_cap = dma_cap;
            self.consumed_samples = 0;
            self.seek_base_ms = self.elapsed_ms;
            self.state = PlaybackState::Playing;
            self.audio_exhausted = false;

            // After SRST, LPIB starts at 0 — always start in half 0
            self.last_half = 0;

            let _ = crate::drivers::hda::set_volume(self.volume.min(100) as u8);
            crate::serial_println!("[MUSIC] Resumed at {}ms, sample={}", self.elapsed_ms, sample_pos);
        } else {
            crate::serial_println!("[MUSIC] Resume start_looped_playback failed");
        }
        // Put audio back
        self.audio = Some(audio);
    }

    /// Seek to a specific millisecond position. Works from Playing or Paused.
    pub fn seek_to(&mut self, target_ms: u64) {
        let target_ms = target_ms.min(self.total_ms);
        self.elapsed_ms = target_ms;

        // If paused, just update position — DMA will be refilled on resume
        if self.state == PlaybackState::Paused {
            crate::serial_println!("[MUSIC] Seek (paused) to {}ms", target_ms);
            return;
        }

        // If playing, do a live seek: refill DMA from new position
        if self.state == PlaybackState::Playing {
            self.resume_from_current_pos();
            crate::serial_println!("[MUSIC] Seek (playing) to {}ms", target_ms);
        }
    }

    /// Tick: DMA refill + FFT analysis. Called every desktop frame (~16ms).
    pub fn tick(&mut self) {
        if self.state != PlaybackState::Playing { return; }
        let audio = match &self.audio {
            Some(a) => a,
            None => return,
        };

        // DMA refill using LPIB-based half-buffer ping-pong
        if let Some((dma_ptr, dma_cap)) = crate::drivers::hda::get_dma_buffer_info() {
            let half_i16 = dma_cap / 2;
            let half_bytes = (half_i16 * 2) as u32;
            let full_bytes = (dma_cap * 2) as u32;

            crate::drivers::hda::clear_stream_status();
            crate::drivers::hda::ensure_running();

            let lpib = crate::drivers::hda::get_playback_position();
            let lpib_clamped = if lpib >= full_bytes { 0 } else { lpib };
            let current_half = if lpib_clamped < half_bytes { 0u32 } else { 1u32 };

            if current_half != self.last_half {
                // A half-buffer has been consumed — track it
                self.consumed_samples += half_i16;

                if self.write_cursor < audio.len() {
                    let dest_offset = self.last_half as usize * half_i16;
                    let remaining = audio.len() - self.write_cursor;
                    let to_copy = remaining.min(half_i16);
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            audio.as_ptr().add(self.write_cursor),
                            dma_ptr.add(dest_offset),
                            to_copy,
                        );
                        if to_copy < half_i16 {
                            // If looping, wrap around to fill the rest from the start
                            if self.is_looping && !audio.is_empty() {
                                let mut filled = to_copy;
                                while filled < half_i16 {
                                    let wrap_copy = (half_i16 - filled).min(audio.len());
                                    core::ptr::copy_nonoverlapping(
                                        audio.as_ptr(),
                                        dma_ptr.add(dest_offset + filled),
                                        wrap_copy,
                                    );
                                    filled += wrap_copy;
                                }
                            } else {
                                core::ptr::write_bytes(dma_ptr.add(dest_offset + to_copy), 0, half_i16 - to_copy);
                            }
                        }
                    }
                    self.write_cursor += to_copy;
                    if self.write_cursor >= audio.len() {
                        if self.is_looping {
                            self.write_cursor = 0; // wrap around for next refill
                        } else {
                            self.audio_exhausted = true;
                        }
                    }
                } else if self.is_looping && !audio.is_empty() {
                    // Looping: refill from start of audio
                    self.write_cursor = 0;
                    let dest_offset = self.last_half as usize * half_i16;
                    let to_copy = audio.len().min(half_i16);
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            audio.as_ptr(),
                            dma_ptr.add(dest_offset),
                            to_copy,
                        );
                        if to_copy < half_i16 {
                            let mut filled = to_copy;
                            while filled < half_i16 {
                                let wrap_copy = (half_i16 - filled).min(audio.len());
                                core::ptr::copy_nonoverlapping(
                                    audio.as_ptr(),
                                    dma_ptr.add(dest_offset + filled),
                                    wrap_copy,
                                );
                                filled += wrap_copy;
                            }
                        }
                    }
                    self.write_cursor += to_copy;
                } else {
                    let dest_offset = self.last_half as usize * half_i16;
                    unsafe { core::ptr::write_bytes(dma_ptr.add(dest_offset), 0, half_i16); }
                }
                self.last_half = current_half;
            }

            // ── LPIB-based timing (synced to hardware) ──
            // consumed_samples counts STEREO i16 samples consumed by DMA.
            // But LPIB also tells us exactly where in the current half we are.
            let lpib_samples = (lpib_clamped / 2) as usize; // bytes → i16
            // Total hardware-consumed position = consumed_samples - remaining_in_current_half
            // consumed_samples tracks how many samples have been FULLY played (past halves),
            // plus lpib_samples tells us how far into the current DMA buffer we are.
            // Since consumed_samples is incremented when the PREVIOUS half completes,
            // the actual playback position is: initial_dma_fill + consumed - dma_cap + lpib_samples
            // Simplified: the source audio position being heard RIGHT NOW is:
            let heard_pos = if self.consumed_samples + lpib_samples >= dma_cap {
                self.consumed_samples + lpib_samples - dma_cap
            } else {
                lpib_samples // still within initial DMA fill
            };
            // Convert stereo samples → milliseconds (48kHz stereo = 96000 samples/sec)
            // Add seek_base_ms so seek position is preserved
            self.elapsed_ms = self.seek_base_ms + (heard_pos as u64 * 1000) / (48000 * 2);
        }

        self.vis_frame += 1;

        // Check if done
        if self.elapsed_ms >= self.total_ms || self.audio_exhausted {
            if self.is_looping {
                // Looping mode: restart same track
                self.write_cursor = 0;
                self.audio_exhausted = false;
                self.consumed_samples = 0;
                self.seek_base_ms = 0;
                self.elapsed_ms = 0;
                return;
            }
            // Auto-advance to next track if multiple tracks available
            if self.num_tracks > 1 {
                let next = (self.current_track + 1) % self.num_tracks;
                crate::serial_println!("[MUSIC] Track ended, auto-advancing to track {}", next);
                self.play_track(next);
            } else {
                self.stop();
            }
            return;
        }

        // Audio position for FFT — use LPIB-synced position + A/V sync offset
        let visual_ms = (self.elapsed_ms as i64 + self.av_offset_ms as i64).max(0).min(self.total_ms as i64) as u64;
        let audio_pos = (visual_ms as usize * 48000 * 2 / 1000).min(audio.len().saturating_sub(2));

        // ── Mini FFT (256-point for speed — enough for widget) ──
        let fft_n = 256usize;
        let mono_start = audio_pos.saturating_sub(fft_n * 2) & !1;
        let mut max_abs: f32 = 0.0;
        for i in 0..fft_n {
            let idx = mono_start + i * 2;
            let s = if idx < audio.len() { audio[idx] as f32 } else { 0.0 };
            self.fft_re[i] = s;
            self.fft_im[i] = 0.0;
            let a = if s >= 0.0 { s } else { -s };
            if a > max_abs { max_abs = a; }
        }
        // Auto-gain
        if max_abs > self.peak_rms {
            self.peak_rms += (max_abs - self.peak_rms) * 0.3;
        } else {
            self.peak_rms *= 0.9995;
        }
        let gain = if self.peak_rms > 100.0 { 16000.0 / self.peak_rms } else { 1.0 };
        // Hann + normalize
        for i in 0..fft_n {
            let t = i as f32 / fft_n as f32;
            let hann = 0.5 * (1.0 - libm::cosf(2.0 * core::f32::consts::PI * t));
            self.fft_re[i] *= hann * gain / 32768.0;
        }
        // In-place FFT (256-point)
        {
            let re = &mut self.fft_re[..fft_n];
            let im = &mut self.fft_im[..fft_n];
            // Bit-reversal
            let mut j = 0usize;
            for i in 0..fft_n {
                if i < j { re.swap(i, j); im.swap(i, j); }
                let mut m = fft_n >> 1;
                while m >= 1 && j >= m { j -= m; m >>= 1; }
                j += m;
            }
            // Butterfly
            let mut step = 2;
            while step <= fft_n {
                let half = step / 2;
                let ang = -core::f32::consts::PI * 2.0 / step as f32;
                for k in 0..half {
                    let a = ang * k as f32;
                    let wr = libm::cosf(a);
                    let wi = libm::sinf(a);
                    let mut ii = k;
                    while ii < fft_n {
                        let jj = ii + half;
                        let tr = wr * re[jj] - wi * im[jj];
                        let ti = wr * im[jj] + wi * re[jj];
                        re[jj] = re[ii] - tr; im[jj] = im[ii] - ti;
                        re[ii] += tr; im[ii] += ti;
                        ii += step;
                    }
                }
                step <<= 1;
            }
        }
        // Band energies (256-pt FFT at 48kHz: bin = index * 187.5 Hz)
        let mag = |lo: usize, hi: usize| -> f32 {
            let mut s = 0.0f32;
            for i in lo..hi.min(128) {
                s += libm::sqrtf(self.fft_re[i] * self.fft_re[i] + self.fft_im[i] * self.fft_im[i]);
            }
            s / (hi - lo).max(1) as f32
        };
        let raw_sub = mag(1, 2);   // ~188Hz
        let raw_bass = mag(2, 4);  // 375-750Hz
        let raw_mid = mag(4, 16);  // 750-3000Hz
        let raw_tre = mag(16, 60); // 3000-11.25kHz
        let raw_e = raw_sub * 1.5 + raw_bass * 1.2 + raw_mid * 0.5 + raw_tre * 0.2;

        // Smooth
        let sm = |prev: f32, new: f32, a: f32, r: f32| -> f32 {
            if new > prev { prev + (new - prev) * a } else { prev + (new - prev) * r }
        };
        self.sub_bass = sm(self.sub_bass, raw_sub.min(1.0), 0.75, 0.10);
        self.bass = sm(self.bass, raw_bass.min(1.0), 0.70, 0.10);
        self.mid = sm(self.mid, raw_mid.min(1.0), 0.60, 0.12);
        self.treble = sm(self.treble, raw_tre.min(1.0), 0.70, 0.16);
        self.energy = sm(self.energy, raw_e.min(1.5), 0.65, 0.10);

        // Beat detection
        let be = raw_sub + raw_bass * 0.8;
        self.energy_hist[self.hist_idx] = be;
        self.hist_idx = (self.hist_idx + 1) % 43;
        if self.hist_count < 43 { self.hist_count += 1; }
        let filled = self.hist_count.max(1) as f32;
        let avg: f32 = self.energy_hist.iter().take(self.hist_count).sum::<f32>() / filled;
        let mut var_sum = 0.0f32;
        for i in 0..self.hist_count { let d = self.energy_hist[i] - avg; var_sum += d * d; }
        let variance = var_sum / filled;
        let threshold = (-15.0 * variance + 1.45f32).max(1.05).min(1.5);
        let onset = be - self.prev_energy;
        if be > avg * threshold && onset > 0.002 && self.hist_count > 5 {
            let strength = ((be - avg * threshold) / avg.max(0.001)).min(1.0);
            self.beat = (0.6 + strength * 0.4).min(1.0);
        } else {
            self.beat *= 0.88;
            if self.beat < 0.02 { self.beat = 0.0; }
        }
        self.prev_energy = be;

        // Update waveform ring buffer (sample every few frames for smooth viz)
        if !audio.is_empty() {
            let idx = audio_pos.min(audio.len() - 1) & !1;
            let sample = audio[idx] as f32 / 32768.0;
            self.waveform[self.wave_idx % 128] = sample;
            self.wave_idx += 1;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// �🎨 RENDER MODE - Choose between classic and OpenGL compositor
// ═══════════════════════════════════════════════════════════════════════════════

/// Rendering backend for the desktop
#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    /// Classic framebuffer rendering (stable, fast)
    Classic,
    /// OpenGL compositor with effects (modern, customizable)
    OpenGL,
    /// GPU-accelerated classic: same rendering pipeline but with dirty-rect
    /// tracking and VirtIO GPU partial flush for 2-5x speedup on idle frames
    GpuAccelerated,
}

/// Adaptive desktop complexity tier — chosen based on host capabilities.
/// Higher tiers unlock more visual effects; if FPS drops below 30 for
/// 3 consecutive seconds the tier auto-downgrades.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum DesktopTier {
    /// CLI only — returned to shell, no GUI launched (< 128 MB RAM or no FB)
    CliOnly,
    /// Minimal desktop: solid background, taskbar, windows — no rain, no effects
    Minimal,
    /// Standard desktop: matrix rain at low density, basic animations
    Standard,
    /// Full desktop: 4-layer matrix rain, visualizer, drone swarm, all effects
    Full,
}

pub struct Desktop {
    pub windows: Vec<Window>,
    pub icons: Vec<DesktopIcon>,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub cursor_visible: bool,
    pub width: u32,
    pub height: u32,
    frame_count: u64,
    start_menu_open: bool,
    pub input_buffer: String,
    pub cursor_blink: bool,
    pub context_menu: ContextMenu,
    // Performance: cached RTC (read once/sec, not every frame)
    cached_time_str: String,
    cached_date_str: String,
    last_rtc_frame: u64,
    // Performance: cached background
    background_cached: bool,
    needs_full_redraw: bool,
    last_cursor_x: i32,
    last_cursor_y: i32,
    // Track state changes
    last_window_count: usize,
    last_start_menu_open: bool,
    last_context_menu_visible: bool,
    // OpenGL compositor
    pub render_mode: RenderMode,
    pub compositor_theme: CompositorTheme,
    // GPU-accelerated dirty rect tracking (Upgrade #3)
    dirty_rects: [(u32, u32, u32, u32); 32], // (x, y, w, h) — max 32 regions
    dirty_rect_count: usize,
    gpu_frame_skip: u32,  // frames since last full redraw
    // Browser state
    pub browser: Option<crate::browser::Browser>,
    pub browser_url_input: String,
    pub browser_url_cursor: usize,
    pub browser_loading: bool,
    pub browser_url_select_all: bool,
    // Editor states (window_id -> EditorState)
    pub editor_states: BTreeMap<u32, EditorState>,
    // Model editor states (window_id -> ModelEditorState)
    pub model_editor_states: BTreeMap<u32, crate::model_editor::ModelEditorState>,
    // Calculator states (window_id -> CalculatorState)
    pub calculator_states: BTreeMap<u32, CalculatorState>,
    // Snake game states (window_id -> SnakeState)
    pub snake_states: BTreeMap<u32, SnakeState>,
    // 3D Game states (window_id -> Game3DState)
    pub game3d_states: BTreeMap<u32, crate::game3d::Game3DState>,
    // Chess game states (window_id -> ChessState)
    pub chess_states: BTreeMap<u32, crate::chess::ChessState>,
    // 3D Chess states (window_id -> Chess3DState)
    pub chess3d_states: BTreeMap<u32, crate::chess3d::Chess3DState>,
    // Binary viewer states (window_id -> BinaryViewerState)
    pub binary_viewer_states: BTreeMap<u32, crate::apps::binary_viewer::BinaryViewerState>,
    // NES emulator states
    #[cfg(feature = "emulators")]
    pub nes_states: BTreeMap<u32, crate::nes::NesEmulator>,
    // Game Boy emulator states
    #[cfg(feature = "emulators")]
    pub gameboy_states: BTreeMap<u32, crate::gameboy::GameBoyEmulator>,
    // Lab mode states (window_id -> LabState)
    pub lab_states: BTreeMap<u32, crate::lab_mode::LabState>,
    // Music player states (window_id -> MusicPlayerState)
    pub music_player_states: BTreeMap<u32, MusicPlayerState>,
    // GameLab states (window_id -> GameLabState)
    #[cfg(feature = "emulators")]
    pub gamelab_states: BTreeMap<u32, crate::game_lab::GameLabState>,
    // GameBoy Input window links (input_window_id -> gb_window_id)
    #[cfg(feature = "emulators")]
    pub gb_input_links: BTreeMap<u32, u32>,
    // UI scale factor (1 = native, 2 = HiDPI, 3 = ultra)
    pub scale_factor: u32,
    // Matrix rain state (depth-parallax advancing effect)
    matrix_chars: Vec<u8>,
    matrix_heads: Vec<i32>,
    matrix_speeds: Vec<u32>,
    matrix_seeds: Vec<u32>,
    matrix_initialized: bool,
    matrix_beat_count: u32,
    matrix_last_beat: bool,
    /// Rain speed preset: 0=slow, 1=mid, 2=fast
    pub matrix_rain_preset: u8,
    /// Per-cell pixel overrides: key = flat cell index (idx * MAX_TRAIL + trail_i).
    /// When a cell has an override, its custom pixel block is rendered instead of
    /// the standard glyph. Rain color/intensity modulation still applies.
    pub matrix_overrides: BTreeMap<usize, CellPixels>,
    /// Screen-space projection zone: static image revealed by rain.
    pub matrix_projection: MatrixProjection,
    // Visualizer: multi-mode 3D wireframes revealed by rain collision
    visualizer: crate::visualizer::VisualizerState,
    // Drone swarm: holographic wireframe formations rendered through rain
    drone_swarm: crate::drone_swarm::DroneSwarmState,
    // ── Global audio analyzer (reacts to ANY HDA output, not just music player) ──
    // Uses Vec instead of fixed arrays to avoid bloating the static struct size
    global_fft_re: Vec<f32>,
    global_fft_im: Vec<f32>,
    global_sub_bass: f32,
    global_bass: f32,
    global_mid: f32,
    global_treble: f32,
    global_energy: f32,
    global_beat: f32,
    global_peak_rms: f32,
    global_prev_energy: f32,
    global_energy_hist: Vec<f32>,
    global_hist_idx: usize,
    global_hist_count: usize,
    global_audio_active: bool,
    // Terminal auto-suggestions: how many suggestion lines added after prompt
    terminal_suggestion_count: usize,
    // Terminal command history (Up/Down arrows)
    command_history: Vec<String>,
    history_index: Option<usize>,
    saved_input: String,
    // Start menu search
    pub start_menu_search: String,
    // Start menu keyboard navigation (selected item index, -1 = none)
    pub start_menu_selected: i32,
    /// Desktop clipboard: (icon_index, is_cut)
    clipboard_icon: Option<(usize, bool)>,
    
    // ══════ NEW FEATURES ══════
    /// File manager view mode per window (window_id -> mode)
    pub fm_view_modes: BTreeMap<u32, FileManagerViewMode>,
    /// File manager explorer states per window (window_id -> FileManagerState)
    pub fm_states: BTreeMap<u32, FileManagerState>,
    /// Image viewer states per window (window_id -> state with pixel data)
    pub image_viewer_states: BTreeMap<u32, ImageViewerState>,
    /// File clipboard for copy/paste in file manager
    pub file_clipboard: Option<FileClipboardEntry>,
    /// Drag-and-drop state
    pub drag_state: Option<DragState>,
    /// Settings panel: active category index (0=Display, 1=Sound, 2=Taskbar, 3=Personalization, 4=Accessibility, 5=Network, 6=Apps, 7=About)
    pub settings_category: u8,
    /// NetScan: active tab index (0=Dashboard, 1=PortScan, 2=Discovery, 3=Sniffer, 4=Traceroute, 5=VulnScan)
    pub netscan_tab: u8,
    /// Lock screen active
    pub lock_screen_active: bool,
    /// Lock screen PIN input buffer
    pub lock_screen_input: String,
    /// Lock screen wrong PIN shake animation
    pub lock_screen_shake: u32,
    /// System tray: simulated volume level (0-100)
    pub sys_volume: u32,
    /// System tray: simulated battery level (0-100)
    pub sys_battery: u32,
    /// System tray: simulated wifi connected
    pub sys_wifi_connected: bool,
    
    // ══════ TOUCH INPUT ══════
    /// Gesture recognizer state machine
    gesture_recognizer: crate::gesture::GestureRecognizer,
    /// Per-frame gesture output buffer
    gesture_buffer: crate::gesture::GestureBuffer,
    /// Touch-based cursor mode (true when last input was touch)
    pub touch_mode: bool,
    /// Mobile UI state (portrait mode, same style as desktop)
    pub mobile_state: crate::mobile::MobileState,
    // ══════ FPS TRACKING ══════
    /// Tick value at last FPS measurement
    fps_last_tick: u64,
    /// Frames rendered since last FPS measurement
    fps_frame_accum: u32,
    /// Current measured FPS
    pub fps_current: u32,
    /// Show FPS overlay on desktop
    pub fps_display: bool,
    /// Adaptive desktop complexity tier
    pub desktop_tier: DesktopTier,
    /// Consecutive low-FPS frames (for auto-downgrade)
    fps_low_count: u32,
    /// Consecutive high-FPS frames (for auto-upgrade)
    fps_high_count: u32,
    /// The initial tier detected at boot (ceiling for auto-upgrade)
    initial_tier: DesktopTier,
    /// Snap preview zone (shown while dragging a window near screen edges)
    snap_preview: Option<SnapDir>,
    /// Shortcut help overlay visible (F1 toggle)
    show_shortcuts: bool,
}

/// Calculator state for interactive calculator windows
pub struct CalculatorState {
    /// The full expression string (e.g. "2*(3+4)")
    pub expression: String,
    /// Display text (expression while typing, result after =)
    pub display: String,
    /// True after pressing = (next digit starts new expression)
    pub just_evaluated: bool,
    /// Scientific mode toggle
    pub scientific: bool,
}

impl CalculatorState {
    pub fn new() -> Self {
        CalculatorState {
            expression: String::new(),
            display: String::from("0"),
            just_evaluated: false,
            scientific: false,
        }
    }
    
    pub fn press_digit(&mut self, d: char) {
        if self.just_evaluated {
            self.expression.clear();
            self.just_evaluated = false;
        }
        if self.expression.len() < 64 {
            self.expression.push(d);
            self.display = self.expression.clone();
        }
    }
    
    pub fn press_dot(&mut self) {
        if self.just_evaluated {
            self.expression = String::from("0");
            self.just_evaluated = false;
        }
        self.expression.push('.');
        self.display = self.expression.clone();
    }
    
    pub fn press_operator(&mut self, op: char) {
        if self.just_evaluated {
            // Continue from result — expression starts with the result
            self.just_evaluated = false;
        }
        if !self.expression.is_empty() {
            self.expression.push(op);
            self.display = self.expression.clone();
        }
    }
    
    pub fn press_paren(&mut self, p: char) {
        if self.just_evaluated && p == '(' {
            self.expression.clear();
            self.just_evaluated = false;
        }
        self.expression.push(p);
        self.display = self.expression.clone();
    }
    
    pub fn press_func(&mut self, name: &str) {
        if self.just_evaluated {
            self.expression.clear();
            self.just_evaluated = false;
        }
        self.expression.push_str(name);
        self.expression.push('(');
        self.display = self.expression.clone();
    }
    
    pub fn press_equals(&mut self) {
        let result = Self::eval_expression(&self.expression);
        self.display = Self::format_number(result);
        // Keep the result as expression for chaining
        self.expression = self.display.clone();
        self.just_evaluated = true;
    }
    
    pub fn press_clear(&mut self) {
        self.expression.clear();
        self.display = String::from("0");
        self.just_evaluated = false;
    }
    
    pub fn press_backspace(&mut self) {
        if !self.expression.is_empty() {
            // If expression ends with a function name like "sqrt(", remove the whole thing
            let funcs = ["sqrt(", "sin(", "cos(", "tan(", "abs(", "ln("];
            let mut removed_func = false;
            for f in funcs {
                if self.expression.ends_with(f) {
                    for _ in 0..f.len() { self.expression.pop(); }
                    removed_func = true;
                    break;
                }
            }
            if !removed_func {
                self.expression.pop();
            }
            if self.expression.is_empty() {
                self.display = String::from("0");
            } else {
                self.display = self.expression.clone();
            }
        }
    }
    
    /// Toggle scientific mode
    pub fn toggle_scientific(&mut self) {
        self.scientific = !self.scientific;
    }
    
    // ── Expression evaluator with parentheses and precedence ──
    // Grammar: expr = term (('+' | '-') term)*
    //          term = factor (('*' | '/' | '%') factor)*
    //          factor = ['-'] atom
    //          atom = number | '(' expr ')' | func '(' expr ')'
    
    fn eval_expression(expr: &str) -> f64 {
        let tokens = Self::tokenize(expr);
        let mut pos = 0;
        let result = Self::parse_expr(&tokens, &mut pos);
        result
    }
    
    fn tokenize(expr: &str) -> Vec<CalcToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = expr.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let ch = chars[i];
            if ch.is_ascii_digit() || ch == '.' {
                // Parse number
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                tokens.push(CalcToken::Num(Self::parse_float(&num_str)));
            } else if ch.is_ascii_alphabetic() {
                // Parse function name
                let start = i;
                while i < chars.len() && chars[i].is_ascii_alphabetic() {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                tokens.push(CalcToken::Func(name));
            } else if ch == '(' {
                tokens.push(CalcToken::LParen);
                i += 1;
            } else if ch == ')' {
                tokens.push(CalcToken::RParen);
                i += 1;
            } else if ch == '+' || ch == '-' || ch == '*' || ch == '/' || ch == '%' {
                tokens.push(CalcToken::Op(ch));
                i += 1;
            } else {
                i += 1; // skip unknown chars
            }
        }
        tokens
    }
    
    fn parse_expr(tokens: &[CalcToken], pos: &mut usize) -> f64 {
        let mut left = Self::parse_term(tokens, pos);
        while *pos < tokens.len() {
            match &tokens[*pos] {
                CalcToken::Op('+') => { *pos += 1; left += Self::parse_term(tokens, pos); }
                CalcToken::Op('-') => { *pos += 1; left -= Self::parse_term(tokens, pos); }
                _ => break,
            }
        }
        left
    }
    
    fn parse_term(tokens: &[CalcToken], pos: &mut usize) -> f64 {
        let mut left = Self::parse_factor(tokens, pos);
        while *pos < tokens.len() {
            match &tokens[*pos] {
                CalcToken::Op('*') => { *pos += 1; left *= Self::parse_factor(tokens, pos); }
                CalcToken::Op('/') => { *pos += 1; let r = Self::parse_factor(tokens, pos); left = if r != 0.0 { left / r } else { 0.0 }; }
                CalcToken::Op('%') => { *pos += 1; let r = Self::parse_factor(tokens, pos); left = if r != 0.0 { left % r } else { 0.0 }; }
                _ => break,
            }
        }
        left
    }
    
    fn parse_factor(tokens: &[CalcToken], pos: &mut usize) -> f64 {
        // Handle unary minus
        if *pos < tokens.len() {
            if let CalcToken::Op('-') = &tokens[*pos] {
                *pos += 1;
                return -Self::parse_atom(tokens, pos);
            }
        }
        Self::parse_atom(tokens, pos)
    }
    
    fn parse_atom(tokens: &[CalcToken], pos: &mut usize) -> f64 {
        if *pos >= tokens.len() { return 0.0; }
        
        match &tokens[*pos] {
            CalcToken::Num(n) => {
                let v = *n;
                *pos += 1;
                v
            }
            CalcToken::LParen => {
                *pos += 1; // skip (
                let v = Self::parse_expr(tokens, pos);
                if *pos < tokens.len() {
                    if let CalcToken::RParen = &tokens[*pos] { *pos += 1; }
                }
                v
            }
            CalcToken::Func(name) => {
                let fname = name.clone();
                *pos += 1; // skip func name
                // Expect (
                if *pos < tokens.len() {
                    if let CalcToken::LParen = &tokens[*pos] { *pos += 1; }
                }
                let arg = Self::parse_expr(tokens, pos);
                if *pos < tokens.len() {
                    if let CalcToken::RParen = &tokens[*pos] { *pos += 1; }
                }
                Self::apply_func(&fname, arg)
            }
            _ => {
                *pos += 1;
                0.0
            }
        }
    }
    
    fn apply_func(name: &str, x: f64) -> f64 {
        match name {
            "sqrt" => {
                if x >= 0.0 { Self::sqrt_approx(x) } else { 0.0 }
            }
            "sin" => Self::sin_approx(x),
            "cos" => Self::cos_approx(x),
            "tan" => {
                let c = Self::cos_approx(x);
                if c.abs() > 1e-10 { Self::sin_approx(x) / c } else { 0.0 }
            }
            "abs" => if x < 0.0 { -x } else { x },
            "ln" => Self::ln_approx(x),
            _ => x,
        }
    }
    
    // ── Math approximations (no libm in no_std) ──
    
    fn sqrt_approx(x: f64) -> f64 {
        if x <= 0.0 { return 0.0; }
        let mut guess = x / 2.0;
        for _ in 0..20 {
            guess = (guess + x / guess) / 2.0;
        }
        guess
    }
    
    fn sin_approx(x: f64) -> f64 {
        // Normalize to [-PI, PI]
        let pi = 3.14159265358979323846;
        let mut x = x % (2.0 * pi);
        if x > pi { x -= 2.0 * pi; }
        if x < -pi { x += 2.0 * pi; }
        // Taylor series: sin(x) = x - x^3/6 + x^5/120 - x^7/5040 + ...
        let x2 = x * x;
        let x3 = x2 * x;
        let x5 = x3 * x2;
        let x7 = x5 * x2;
        let x9 = x7 * x2;
        let x11 = x9 * x2;
        x - x3 / 6.0 + x5 / 120.0 - x7 / 5040.0 + x9 / 362880.0 - x11 / 39916800.0
    }
    
    fn cos_approx(x: f64) -> f64 {
        let pi = 3.14159265358979323846;
        Self::sin_approx(x + pi / 2.0)
    }
    
    fn ln_approx(x: f64) -> f64 {
        if x <= 0.0 { return 0.0; }
        // Use: ln(x) = 2 * atanh((x-1)/(x+1)) with series expansion
        let y = (x - 1.0) / (x + 1.0);
        let y2 = y * y;
        let mut result = y;
        let mut term = y;
        for n in 1..30 {
            term *= y2;
            result += term / (2 * n + 1) as f64;
        }
        result * 2.0
    }
    
    fn parse_float(s: &str) -> f64 {
        let mut result: f64 = 0.0;
        let mut decimal_part = false;
        let mut decimal_factor = 0.1;
        let mut negative = false;
        for (i, ch) in s.chars().enumerate() {
            if ch == '-' && i == 0 {
                negative = true;
            } else if ch == '.' {
                decimal_part = true;
            } else if ch.is_ascii_digit() {
                let digit = (ch as u8 - b'0') as f64;
                if decimal_part {
                    result += digit * decimal_factor;
                    decimal_factor *= 0.1;
                } else {
                    result = result * 10.0 + digit;
                }
            }
        }
        if negative { -result } else { result }
    }
    
    fn format_number(n: f64) -> String {
        if n == (n as i64) as f64 && n.abs() < 1e15 {
            format!("{}", n as i64)
        } else {
            let s = format!("{:.6}", n);
            let s = s.trim_end_matches('0');
            let s = s.trim_end_matches('.');
            String::from(s)
        }
    }
}

/// Token for calculator expression parser
#[derive(Clone)]
enum CalcToken {
    Num(f64),
    Op(char),
    LParen,
    RParen,
    Func(String),
}

/// Snake game state for interactive game windows
pub struct SnakeState {
    pub snake: Vec<(i32, i32)>,
    pub direction: (i32, i32),
    pub food: (i32, i32),
    pub score: u32,
    pub game_over: bool,
    pub paused: bool,
    pub high_score: u32,
    pub grid_w: i32,
    pub grid_h: i32,
    pub tick_counter: u32,
    pub speed: u32,
    pub rng_state: u32,
}

impl SnakeState {
    pub fn new() -> Self {
        let mut state = SnakeState {
            snake: Vec::new(),
            direction: (1, 0),
            food: (12, 5),
            score: 0,
            game_over: false,
            paused: false,
            high_score: 0,
            grid_w: 20,
            grid_h: 15,
            tick_counter: 0,
            speed: 8, // Move every 8 frames
            rng_state: 42,
        };
        // Initial snake in the middle
        for i in 0..4 {
            state.snake.push((10 - i, 7));
        }
        state
    }
    
    fn next_rng(&mut self) -> u32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        self.rng_state
    }
    
    pub fn spawn_food(&mut self) {
        // Check if there are any free cells left (avoid infinite loop)
        let total_cells = (self.grid_w * self.grid_h) as usize;
        if self.snake.len() >= total_cells {
            // Grid is full — player wins!
            self.game_over = true;
            if self.score > self.high_score { self.high_score = self.score; }
            return;
        }
        for _ in 0..1000 {
            let fx = (self.next_rng() % self.grid_w as u32) as i32;
            let fy = (self.next_rng() % self.grid_h as u32) as i32;
            if !self.snake.iter().any(|&(sx, sy)| sx == fx && sy == fy) {
                self.food = (fx, fy);
                return;
            }
        }
        // Fallback: linear scan for any free cell
        for gy in 0..self.grid_h {
            for gx in 0..self.grid_w {
                if !self.snake.iter().any(|&(sx, sy)| sx == gx && sy == gy) {
                    self.food = (gx, gy);
                    return;
                }
            }
        }
        // Truly full — game over (win)
        self.game_over = true;
        if self.score > self.high_score { self.high_score = self.score; }
    }
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT};
        // Pause toggle with P or Escape
        if key == b'p' || key == b'P' || key == 0x1B {
            if !self.game_over {
                self.paused = !self.paused;
                return;
            }
        }
        if self.game_over {
            if key == b' ' || key == 0x0D {
                // Restart — preserve high score
                let hs = self.high_score;
                *self = SnakeState::new();
                self.high_score = hs;
            }
            return;
        }
        if self.paused { return; }
        match key {
            KEY_UP    if self.direction != (0, 1)  => self.direction = (0, -1),
            KEY_DOWN  if self.direction != (0, -1) => self.direction = (0, 1),
            KEY_LEFT  if self.direction != (1, 0)  => self.direction = (-1, 0),
            KEY_RIGHT if self.direction != (-1, 0) => self.direction = (1, 0),
            _ => {}
        }
    }
    
    pub fn tick(&mut self) {
        if self.game_over || self.paused { return; }
        self.tick_counter += 1;
        if self.tick_counter < self.speed { return; }
        self.tick_counter = 0;
        
        let head = self.snake[0];
        let new_head = (head.0 + self.direction.0, head.1 + self.direction.1);
        
        // Wall collision
        if new_head.0 < 0 || new_head.0 >= self.grid_w || new_head.1 < 0 || new_head.1 >= self.grid_h {
            self.game_over = true;
            if self.score > self.high_score { self.high_score = self.score; }
            return;
        }
        
        // Self collision
        if self.snake.iter().any(|&s| s == new_head) {
            self.game_over = true;
            if self.score > self.high_score { self.high_score = self.score; }
            return;
        }
        
        self.snake.insert(0, new_head);
        
        // Check food
        if new_head == self.food {
            self.score += 10;
            self.spawn_food();
            // Speed up slightly every 50 points
            if self.score % 50 == 0 && self.speed > 3 {
                self.speed -= 1;
            }
        } else {
            self.snake.pop();
        }
    }
}

impl Desktop {
    pub const fn new() -> Self {
        Desktop {
            windows: Vec::new(),
            icons: Vec::new(),
            cursor_x: 640,
            cursor_y: 400,
            cursor_visible: true,
            width: 1280,
            height: 800,
            frame_count: 0,
            start_menu_open: false,
            input_buffer: String::new(),
            cursor_blink: true,
            context_menu: ContextMenu {
                visible: false,
                x: 0,
                y: 0,
                items: Vec::new(),
                selected_index: 0,
                target_icon: None,
                target_file: None,
            },
            cached_time_str: String::new(),
            cached_date_str: String::new(),
            last_rtc_frame: 0,
            background_cached: false,
            needs_full_redraw: true,
            last_cursor_x: 640,
            last_cursor_y: 400,
            last_window_count: 0,
            last_start_menu_open: false,
            last_context_menu_visible: false,
            render_mode: RenderMode::Classic,
            compositor_theme: CompositorTheme::Modern,
            dirty_rects: [(0, 0, 0, 0); 32],
            dirty_rect_count: 0,
            gpu_frame_skip: 0,
            browser: None,
            browser_url_input: String::new(),
            browser_url_cursor: 0,
            browser_loading: false,
            browser_url_select_all: false,
            editor_states: BTreeMap::new(),
            model_editor_states: BTreeMap::new(),
            calculator_states: BTreeMap::new(),
            snake_states: BTreeMap::new(),
            game3d_states: BTreeMap::new(),
            chess_states: BTreeMap::new(),
            chess3d_states: BTreeMap::new(),
            #[cfg(feature = "emulators")]
            nes_states: BTreeMap::new(),
            #[cfg(feature = "emulators")]
            gameboy_states: BTreeMap::new(),
            binary_viewer_states: BTreeMap::new(),
            lab_states: BTreeMap::new(),
            music_player_states: BTreeMap::new(),
            #[cfg(feature = "emulators")]
            gamelab_states: BTreeMap::new(),
            #[cfg(feature = "emulators")]
            gb_input_links: BTreeMap::new(),
            scale_factor: 1,
            matrix_chars: Vec::new(),
            matrix_heads: Vec::new(),
            matrix_speeds: Vec::new(),
            matrix_seeds: Vec::new(),
            matrix_initialized: false,
            matrix_beat_count: 0,
            matrix_last_beat: false,
            matrix_rain_preset: 0, // default = slow
            matrix_overrides: BTreeMap::new(),
            matrix_projection: MatrixProjection::empty(),
            visualizer: crate::visualizer::VisualizerState::new(),
            drone_swarm: crate::drone_swarm::DroneSwarmState::new(),
            // Global audio analyzer (heap-allocated to avoid static bloat)
            global_fft_re: Vec::new(),
            global_fft_im: Vec::new(),
            global_sub_bass: 0.0,
            global_bass: 0.0,
            global_mid: 0.0,
            global_treble: 0.0,
            global_energy: 0.0,
            global_beat: 0.0,
            global_peak_rms: 0.0,
            global_prev_energy: 0.0,
            global_energy_hist: Vec::new(),
            global_hist_idx: 0,
            global_hist_count: 0,
            global_audio_active: false,
            terminal_suggestion_count: 0,
            command_history: Vec::new(),
            history_index: None,
            saved_input: String::new(),
            start_menu_search: String::new(),
            start_menu_selected: -1,
            clipboard_icon: None,
            // New features
            fm_view_modes: BTreeMap::new(),
            fm_states: BTreeMap::new(),
            image_viewer_states: BTreeMap::new(),
            file_clipboard: None,
            drag_state: None,
            settings_category: 0,
            netscan_tab: 0,
            lock_screen_active: false,
            lock_screen_input: String::new(),
            lock_screen_shake: 0,
            sys_volume: 75,
            sys_battery: 85,
            sys_wifi_connected: true,
            // Touch input
            gesture_recognizer: crate::gesture::GestureRecognizer::new(1280, 800),
            gesture_buffer: crate::gesture::GestureBuffer::new(),
            touch_mode: false,
            // Mobile UI
            mobile_state: crate::mobile::MobileState::new(),
            // FPS tracking
            fps_last_tick: 0,
            fps_frame_accum: 0,
            fps_current: 0,
            fps_display: true,
            desktop_tier: DesktopTier::Full,
            fps_low_count: 0,
            fps_high_count: 0,
            initial_tier: DesktopTier::Full,
            snap_preview: None,
            show_shortcuts: false,
        }
    }
    
    /// Initialize desktop with double buffering
    pub fn init(&mut self, width: u32, height: u32) {
        crate::serial_println!("[Desktop] init start: {}x{} (clearing {} windows, {} icons)", 
            width, height, self.windows.len(), self.icons.len());
        
        // ===== FULL STATE RESET (prevents duplication on re-entry) =====
        // Reset mobile state so "desktop" command always boots in desktop mode
        self.mobile_state = crate::mobile::MobileState::new();
        // Data collections
        self.windows.clear();
        self.icons.clear();
        self.editor_states.clear();
        self.model_editor_states.clear();
        self.calculator_states.clear();
        self.snake_states.clear();
        self.game3d_states.clear();
        self.chess3d_states.clear();
        #[cfg(feature = "emulators")]
        self.nes_states.clear();
        #[cfg(feature = "emulators")]
        self.gameboy_states.clear();
        self.binary_viewer_states.clear();
        self.lab_states.clear();
        self.music_player_states.clear();
        // Browser
        self.browser = None;
        self.browser_url_input.clear();
        self.browser_url_cursor = 0;
        self.browser_loading = false;
        self.browser_url_select_all = false;
        // Input / UI state
        self.input_buffer.clear();
        self.start_menu_open = false;
        self.start_menu_search.clear();
        self.cursor_blink = false;
        self.context_menu.visible = false;
        self.context_menu.items.clear();
        self.context_menu.selected_index = 0;
        self.context_menu.target_icon = None;
        self.context_menu.target_file = None;
        // Counters / tracking
        self.frame_count = 0;
        self.terminal_suggestion_count = 0;
        self.command_history.clear();
        self.history_index = None;
        self.saved_input.clear();
        self.last_window_count = 0;
        self.last_start_menu_open = false;
        self.last_context_menu_visible = false;
        self.last_rtc_frame = 0;
        self.cached_time_str.clear();
        self.cached_date_str.clear();
        // Reset window ID counter so IDs don't grow unbounded
        *NEXT_WINDOW_ID.lock() = 1;
        
        crate::serial_println!("[Desktop] state cleared, windows={} icons={}", 
            self.windows.len(), self.icons.len());
        
        self.width = width;
        self.height = height;
        self.cursor_x = (width / 2) as i32;
        self.cursor_y = (height / 2) as i32;
        
        // Initialize UI scaling based on resolution
        crate::graphics::scaling::init(width, height);
        self.scale_factor = crate::graphics::scaling::get_scale_factor();
        crate::serial_println!("[Desktop] UI scale factor: {}x", self.scale_factor);
        
        // Initialize touch input subsystem
        crate::touch::init();
        crate::touch::set_screen_size(width, height);
        self.gesture_recognizer.set_screen_size(width as i32, height as i32);
        crate::serial_println!("[Desktop] Touch input initialized");
        
        // Initialize double buffering
        crate::serial_println!("[Desktop] init_double_buffer...");
        framebuffer::init_double_buffer();
        // Verify backbuffer was actually allocated — if it failed (OOM),
        // fall back to direct framebuffer mode to avoid invisible desktop
        if framebuffer::get_backbuffer_ptr().is_some() {
            framebuffer::set_double_buffer_mode(true);
            crate::serial_println!("[Desktop] double buffer: OK");
        } else {
            framebuffer::set_double_buffer_mode(false);
            crate::serial_println!("[Desktop] WARNING: backbuffer alloc failed, using direct FB mode");
        }
        
        // Initialize background cache for fast redraws
        crate::serial_println!("[Desktop] init_background_cache...");
        framebuffer::init_background_cache();
        
        // Initialize OpenGL compositor
        crate::serial_println!("[Desktop] init_compositor...");
        compositor::init_compositor(width, height);
        compositor::set_compositor_theme(self.compositor_theme);
        
        // Create desktop icons like Windows
        crate::serial_println!("[Desktop] init_desktop_icons...");
        self.init_desktop_icons();
        
        // Mark that we need to render background on first frame
        self.background_cached = false;
        self.needs_full_redraw = true;
        
        // Initialize matrix rain
        self.init_matrix_rain();
        
        // Detect optimal desktop tier based on host capabilities
        self.detect_tier();
        
        // Music player is available from the Start menu — not auto-opened
        
        crate::serial_println!("[Desktop] init complete (tier={:?})", self.desktop_tier);
    }
    
    /// Initialize matrix rain background data (depth-parallax advancing effect)
    fn init_matrix_rain(&mut self) {
        // 256 columns × 4 layers: depth-layered rain with far=dense/slow, near=sparse/fast
        const MATRIX_COLS: usize = 256;
        const NUM_LAYERS: usize = 4;
        const MAX_TRAIL: usize = 40;   // must match draw_background
        const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
        
        let total = MATRIX_COLS * NUM_LAYERS;
        self.matrix_chars = vec![0u8; total * MAX_TRAIL];
        self.matrix_heads = vec![0i32; total];
        self.matrix_speeds = vec![2u32; total];
        self.matrix_seeds = vec![0u32; total];
        
        let height = self.height.saturating_sub(TASKBAR_HEIGHT);
        
        for col in 0..MATRIX_COLS {
            for layer in 0..NUM_LAYERS {
                let idx = col * NUM_LAYERS + layer;
                let seed = (col as u32).wrapping_mul(2654435761)
                    ^ 0xDEADBEEF
                    ^ ((layer as u32).wrapping_mul(0x9E3779B9));
                for i in 0..MAX_TRAIL {
                    let char_seed = seed.wrapping_add((i as u32).wrapping_mul(7919));
                    self.matrix_chars[idx * MAX_TRAIL + i] = CHARS[(char_seed as usize) % CHARS.len()];
                }
                // Stagger start positions so layers don't overlap initially
                let spread = height / 2 + (layer as u32) * height / 6;
                self.matrix_heads[idx] = -((seed % spread.max(1)) as i32);
                self.matrix_speeds[idx] = 2 + (seed % 4);
                self.matrix_seeds[idx] = seed;
            }
        }
        self.matrix_initialized = true;
        // Initialize drone swarm holographic formations
        crate::drone_swarm::init(&mut self.drone_swarm, self.width, height);
        // Activate test projection image (centered 256×256 zone)
        // self.activate_test_projection(); // TODO: re-enable when projection art is ready
    }

    /// Activate a colorful test image projection zone centered on screen.
    /// The matrix rain will reveal this image as it falls through the zone.
    pub fn activate_test_projection(&mut self) {
        let proj_w: u32 = 256;
        let proj_h: u32 = 256;
        let screen_w = self.width;
        let screen_h = self.height.saturating_sub(TASKBAR_HEIGHT);
        let proj_x = (screen_w / 2).saturating_sub(proj_w / 2);
        let proj_y = (screen_h / 2).saturating_sub(proj_h / 2);
        let pixels = MatrixProjection::generate_test_image(proj_w, proj_h);
        self.matrix_projection = MatrixProjection {
            x: proj_x,
            y: proj_y,
            width: proj_w,
            height: proj_h,
            pixels,
            active: true,
        };
    }

    /// Deactivate the projection zone.
    pub fn deactivate_projection(&mut self) {
        self.matrix_projection.active = false;
    }

    // ═══════════════════════════════════════════════════════════════════
    // MATRIX CELL PIXEL OVERRIDE API
    // ═══════════════════════════════════════════════════════════════════
    
    /// Matrix layout constants (must match draw_background)
    const MATRIX_COLS: usize = 256;
    const MATRIX_LAYERS: usize = 4;
    const MATRIX_MAX_TRAIL: usize = 40;

    /// Set rain speed preset: 0=slow, 1=mid, 2=fast
    pub fn set_rain_preset(&mut self, preset: u8) {
        self.matrix_rain_preset = preset.min(2);
        crate::serial_println!("[RAIN] Speed preset set to {}", ["slow", "mid", "fast"][self.matrix_rain_preset as usize]);
    }

    /// Compute the flat cell index for (col, layer, trail_pos).
    #[inline]
    fn matrix_cell_key(col: usize, layer: usize, trail_i: usize) -> usize {
        (col * Self::MATRIX_LAYERS + layer) * Self::MATRIX_MAX_TRAIL + trail_i
    }

    /// Override a cell with a custom pixel block.
    /// `col` 0..320, `layer` 0..3, `trail_i` 0..40.
    /// Returns a mutable reference to the pixel block for further editing.
    pub fn matrix_override_cell(&mut self, col: usize, layer: usize, trail_i: usize, cell: CellPixels) -> &mut CellPixels {
        let key = Self::matrix_cell_key(col, layer, trail_i);
        self.matrix_overrides.insert(key, cell);
        self.matrix_overrides.get_mut(&key).unwrap()
    }

    /// Override a cell, initializing it from the current glyph character at that position.
    /// Gives you the glyph shape as a starting point for per-pixel editing.
    pub fn matrix_override_from_glyph(&mut self, col: usize, layer: usize, trail_i: usize, color: u32) -> &mut CellPixels {
        let idx = col * Self::MATRIX_LAYERS + layer;
        let char_idx = idx * Self::MATRIX_MAX_TRAIL + trail_i;
        let c = if char_idx < self.matrix_chars.len() {
            self.matrix_chars[char_idx] as char
        } else {
            '#'
        };
        let cell = CellPixels::from_glyph(c, color);
        self.matrix_override_cell(col, layer, trail_i, cell)
    }

    /// Get a mutable reference to an existing cell override (None if not overridden).
    pub fn matrix_get_cell_mut(&mut self, col: usize, layer: usize, trail_i: usize) -> Option<&mut CellPixels> {
        let key = Self::matrix_cell_key(col, layer, trail_i);
        self.matrix_overrides.get_mut(&key)
    }

    /// Set a single pixel in a cell override. Creates the override if needed (blank).
    pub fn matrix_cell_set_pixel(&mut self, col: usize, layer: usize, trail_i: usize, px: u8, py: u8, color: u32) {
        let key = Self::matrix_cell_key(col, layer, trail_i);
        let cell = self.matrix_overrides.entry(key).or_insert_with(CellPixels::blank);
        cell.set(px, py, color);
    }

    /// Remove the override for a specific cell (reverts to normal glyph rendering).
    pub fn matrix_clear_cell(&mut self, col: usize, layer: usize, trail_i: usize) {
        let key = Self::matrix_cell_key(col, layer, trail_i);
        self.matrix_overrides.remove(&key);
    }

    /// Clear ALL cell overrides.
    pub fn matrix_clear_overrides(&mut self) {
        self.matrix_overrides.clear();
    }

    /// Set a rectangular block of cells to custom pixel blocks.
    /// Useful for drawing images/sprites across multiple cells.
    /// `pixel_data` is a flat ARGB buffer of `pw × ph` pixels.
    /// The image is tiled across the matrix cell grid starting at (start_col, layer, start_trail).
    pub fn matrix_blit_image(&mut self, start_col: usize, layer: usize, start_trail: usize,
                              pixel_data: &[u32], pw: usize, ph: usize) {
        // Number of cells needed
        let cells_w = (pw + 7) / 8;
        let cells_h = (ph + 15) / 16;
        
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                let col = start_col + cx;
                let trail_i = start_trail + cy;
                if col >= Self::MATRIX_COLS || trail_i >= Self::MATRIX_MAX_TRAIL { continue; }
                
                let mut cell = CellPixels::blank();
                for py in 0..16u8 {
                    for px in 0..8u8 {
                        let src_x = cx * 8 + px as usize;
                        let src_y = cy * 16 + py as usize;
                        if src_x < pw && src_y < ph {
                            let color = pixel_data[src_y * pw + src_x];
                            if color & 0xFF000000 != 0 {  // non-transparent
                                cell.set(px, py, color);
                            }
                        }
                    }
                }
                self.matrix_override_cell(col, layer, trail_i, cell);
            }
        }
    }

    /// Open TrustCode with a demo Rust file
    fn open_trustcode_demo(&mut self) {
        // Create a sample Rust file in ramfs
        let sample_code = r#"//! TrustOS — A Modern Operating System in Rust
//!
//! This file demonstrates TrustCode's syntax highlighting

use core::fmt;

/// Main kernel entry point
pub fn kernel_main() -> ! {
    let message = "Hello from TrustOS!";
    serial_println!("{}", message);

    // Initialize hardware
    let cpu_count: u32 = 4;
    let memory_mb: u64 = 256;

    for i in 0..cpu_count {
        init_cpu(i);
    }

    // Start the desktop environment
    let mut desktop = Desktop::new();
    desktop.init(1280, 800);

    loop {
        desktop.render();
        desktop.handle_input();
    }
}

/// Initialize a CPU core
fn init_cpu(id: u32) {
    // Setup GDT, IDT, APIC
    serial_println!("CPU {} initialized", id);
}

#[derive(Debug, Clone)]
struct AppConfig {
    name: String,
    version: (u8, u8, u8),
    features: Vec<&'static str>,
}
"#;
        
        let _ = crate::ramfs::with_fs(|fs| {
            fs.write_file("/demo.rs", sample_code.as_bytes())
        });
        
        let id = self.create_window("TrustCode: demo.rs", 160, 50, 780, 560, WindowType::TextEditor);
        if let Some(editor) = self.editor_states.get_mut(&id) {
            editor.load_file("demo.rs");
        }
        // Focus the editor window
        self.focus_window(id);
        crate::serial_println!("[TrustCode] Demo editor opened");
    }
    
    /// Set render mode (Classic or OpenGL)
    pub fn set_render_mode(&mut self, mode: RenderMode) {
        self.render_mode = mode;
        self.needs_full_redraw = true;
        self.background_cached = false;
        
        if mode == RenderMode::OpenGL {
            // Sync windows to compositor
            self.sync_compositor_surfaces();
        }
    }
    
    /// Detect the optimal desktop tier based on host capabilities.
    /// Called once during init; can also be re-evaluated at runtime.
    pub fn detect_tier(&mut self) {
        let phys_mb = crate::memory::total_physical_memory() / (1024 * 1024);
        let heap_free_mb = crate::memory::heap::free() / (1024 * 1024);
        let pixels = (self.width as u64) * (self.height as u64);
        let cpus = crate::cpu::smp::cpu_count().max(1) as u64;
        
        // Estimate CPU speed via TSC (in MHz)
        let tsc_mhz = crate::cpu::tsc_frequency() / 1_000_000;
        
        // Compute a capability score weighted toward CPU throughput.
        // Full tier requires real rendering power — RAM alone isn't enough.
        //   RAM contribution: 1 point per 256 MB (capped at 8 — diminishing returns)
        //   CPU contribution: 1 point per 400 MHz (weighted more than RAM)
        //   Core contribution: 2 points per core (parallelism matters for rendering)
        //   Resolution penalty: -1 per million pixels above 1M
        let ram_score = ((phys_mb / 256) as i64).min(8);
        let cpu_score = if tsc_mhz > 0 { (tsc_mhz / 400) as i64 } else { 2 };
        let core_score = (cpus as i64) * 2;
        let res_penalty = ((pixels as i64) - 1_000_000) / 1_000_000;
        let score = ram_score + cpu_score + core_score - res_penalty;
        
        // Hard cap: very old single-core CPUs (< 1.5 GHz) cannot sustain Full tier.
        // The 4-layer matrix rain + visualizer + drone swarm need at least
        // ~2 GHz × 2 cores or equivalent throughput to hit stable 30 FPS.
        // Core 2 Duo (~2.2 GHz) is perfectly capable of Standard tier with 2-layer rain.
        let cpu_limited = tsc_mhz > 0 && tsc_mhz < 1500 && cpus <= 1;
        
        let tier = if phys_mb < 128 || heap_free_mb < 8 {
            DesktopTier::CliOnly
        } else if score <= 4 || phys_mb < 256 {
            DesktopTier::Minimal
        } else if score <= 8 || phys_mb < 512 || cpu_limited {
            DesktopTier::Standard
        } else {
            DesktopTier::Full
        };
        
        self.desktop_tier = tier;
        self.initial_tier = tier;
        self.fps_low_count = 0;
        self.fps_high_count = 0;
        
        crate::serial_println!(
            "[Desktop] Tier={:?} (score={}, RAM={}MB, heap={}MB, CPUs={}, TSC={}MHz, {}x{}, cpu_limited={})",
            tier, score, phys_mb, heap_free_mb, cpus, tsc_mhz, self.width, self.height, cpu_limited
        );
    }
    
    /// Auto-adjust tier: downgrade if FPS stays below 18 for ~6 seconds,
    /// upgrade back if FPS stays above 35 for ~5 seconds.
    /// Called once per frame from draw().
    fn auto_adjust_tier(&mut self) {
        // Skip auto-adjust during the first 120 frames (FPS not yet stable,
        // especially on slower hardware like T61 where boot takes longer)
        if self.frame_count < 120 { return; }
        
        // ── Downgrade: sustained low FPS ──
        // Include fps_current == 0 (frame > 1s) as critically low
        if self.fps_current < 18 {
            // Critical: if FPS is 0-2, count faster (each frame ≈ seconds of wall time)
            let increment = if self.fps_current <= 2 { 60 } else { 1 };
            self.fps_low_count += increment;
            self.fps_high_count = 0;
        } else if self.fps_current >= 35 {
            // ── Upgrade candidate: sustained >= 35 FPS ──
            self.fps_high_count += 1;
            if self.fps_low_count > 0 {
                self.fps_low_count = self.fps_low_count.saturating_sub(4);
            }
        } else {
            // In between (18-34 fps): slowly decay low count, reset high count
            if self.fps_low_count > 0 {
                self.fps_low_count = self.fps_low_count.saturating_sub(2);
            }
            self.fps_high_count = 0;
        }
        
        // ~6 seconds of sustained < 18 FPS → downgrade
        if self.fps_low_count >= 360 {
            let old = self.desktop_tier;
            // Emergency: if FPS <= 2, skip directly to Minimal
            let new_tier = if self.fps_current <= 2 {
                match old {
                    DesktopTier::Full | DesktopTier::Standard => DesktopTier::Minimal,
                    _ => old,
                }
            } else {
                match old {
                    DesktopTier::Full => DesktopTier::Standard,
                    DesktopTier::Standard => DesktopTier::Minimal,
                    _ => old,
                }
            };
            if new_tier != old {
                self.desktop_tier = new_tier;
                self.fps_low_count = 0;
                self.fps_high_count = 0;
                self.needs_full_redraw = true;
                self.background_cached = false;
                crate::serial_println!(
                    "[Desktop] Auto-downgrade: {:?} -> {:?} (FPS was {})",
                    old, new_tier, self.fps_current
                );
            }
        }
        
        // ~5 seconds of sustained >= 35 FPS → upgrade (up to initial tier)
        if self.fps_high_count >= 300 {
            let old = self.desktop_tier;
            let new_tier = match old {
                DesktopTier::Minimal => DesktopTier::Standard,
                DesktopTier::Standard => DesktopTier::Full,
                _ => old,
            };
            // Never upgrade beyond what the system initially qualified for
            if new_tier != old && new_tier <= self.initial_tier {
                self.desktop_tier = new_tier;
                self.fps_high_count = 0;
                self.fps_low_count = 0;
                self.needs_full_redraw = true;
                self.background_cached = false;
                crate::serial_println!(
                    "[Desktop] Auto-upgrade: {:?} -> {:?} (FPS was {})",
                    old, new_tier, self.fps_current
                );
            } else {
                self.fps_high_count = 0;
            }
        }
    }
    
    /// Set compositor visual theme
    pub fn set_theme(&mut self, theme: CompositorTheme) {
        self.compositor_theme = theme;
        compositor::set_compositor_theme(theme);
        self.needs_full_redraw = true;
    }
    
    /// Sync window state to compositor surfaces
    fn sync_compositor_surfaces(&self) {
        let mut comp = compositor::compositor();
        comp.surfaces.clear();
        
        for window in &self.windows {
            if window.visible {
                let mut surface = WindowSurface::new(
                    window.id,
                    window.x as f32,
                    window.y as f32,
                    window.width as f32,
                    window.height as f32,
                );
                surface.z_order = 0;
                surface.focused = window.focused;
                surface.visible = !window.minimized;
                comp.surfaces.push(surface);
            }
        }
    }
    
    /// Initialize desktop icons (positioned for left dock sidebar)
    fn init_desktop_icons(&mut self) {
        use crate::icons::IconType;
        
        // Dock layout: icon_size=36, gap=14, ix=12, start_y=12
        let icon_total = 50u32; // icon_size(36) + gap(14)
        let start_y = 12u32;
        let ix = 12u32;
        
        let dock_items: &[(&str, IconType, IconAction)] = &[
            ("Terminal", IconType::Terminal, IconAction::OpenTerminal),
            ("Files", IconType::Folder, IconAction::OpenFileManager),
            ("Editor", IconType::Editor, IconAction::OpenEditor),
            ("Calc", IconType::Calculator, IconAction::OpenCalculator),
            ("NetScan", IconType::Network, IconAction::OpenNetwork),
            ("Chess 3D", IconType::Chess, IconAction::OpenGame),

            ("Browser", IconType::Browser, IconAction::OpenBrowser),
            ("TrustEd", IconType::ModelEditor, IconAction::OpenModelEditor),
            ("Settings", IconType::Settings, IconAction::OpenSettings),
            ("Music", IconType::Music, IconAction::OpenMusicPlayer),
            #[cfg(feature = "emulators")]
            ("GameBoy", IconType::GameBoy, IconAction::OpenGameBoy),
            #[cfg(feature = "emulators")]
            ("GameLab", IconType::GameLab, IconAction::OpenGameLab),
        ];
        
        for (i, (name, icon_type, action)) in dock_items.iter().enumerate() {
            self.icons.push(DesktopIcon {
                name: String::from(*name),
                icon_type: *icon_type,
                x: ix,
                y: start_y + i as u32 * icon_total,
                action: *action,
            });
        }
    }
    
    /// Check if click is on a dock icon
    fn check_icon_click(&self, x: i32, y: i32) -> Option<IconAction> {
        // Dock hit area: full dock strip width
        if x < 0 || x >= (DOCK_WIDTH + 10) as i32 { return None; }
        
        let dock_h = self.height.saturating_sub(TASKBAR_HEIGHT);
        let n_icons = self.icons.len().max(1) as u32;
        let padding = 12u32;
        let available = dock_h.saturating_sub(padding * 2);
        let icon_total = (available / n_icons) as i32;
        let start_y = (padding + (available - icon_total as u32 * n_icons) / 2) as i32;
        
        for (i, icon) in self.icons.iter().enumerate() {
            let iy = start_y + i as i32 * icon_total;
            if y >= iy - 3 && y < iy + icon_total as i32 {
                return Some(icon.action);
            }
        }
        None
    }
    
    /// Create a new window with type
    pub fn create_window(&mut self, title: &str, x: i32, y: i32, width: u32, height: u32, wtype: WindowType) -> u32 {
        // Clamp window size to fit available screen area (minus dock and taskbar)
        let usable_w = self.width.saturating_sub(DOCK_WIDTH + 4);
        let usable_h = self.height.saturating_sub(TASKBAR_HEIGHT + TITLE_BAR_HEIGHT);
        let w = width.min(usable_w).max(120);
        let h = height.min(usable_h).max(80);
        // Clamp position so the window stays on-screen
        let min_x = DOCK_WIDTH as i32 + 2;
        let max_x = (self.width as i32 - w as i32).max(min_x);
        let max_y = (self.height as i32 - TASKBAR_HEIGHT as i32 - h as i32).max(0);
        let cx = x.max(min_x).min(max_x);
        let cy = y.max(0).min(max_y);

        let mut window = Window::new(title, cx, cy, w, h, wtype);
        
        // Initialize content based on type
        match wtype {
            WindowType::Terminal => {
                window.content.push(String::from("\x01HTrustOS Terminal v1.0"));
                window.content.push(String::from("\x01MType \x01Ghelp\x01M for available commands."));
                window.content.push(String::from(""));
                window.content.push(Self::make_prompt("_"));
            },
            WindowType::SystemInfo => {
                window.content.push(String::from("=== System Information ==="));
                window.content.push(String::from(""));
                window.content.push(format!("OS: TrustOS v0.2.0"));
                window.content.push(format!("Arch: x86_64"));
                window.content.push(format!("Display: {}x{}", self.width, self.height));
                window.content.push(String::from("Kernel: trustos_kernel"));
            },
            WindowType::About => {
                window.content.push(String::from("TrustOS"));
                window.content.push(String::from(""));
                window.content.push(String::from("A modern operating system"));
                window.content.push(String::from("written in Rust"));
                window.content.push(String::from(""));
                window.content.push(String::from("(c) 2026 Nathan"));
            },
            WindowType::Calculator => {
                self.calculator_states.insert(window.id, CalculatorState::new());
            },
            WindowType::FileManager => {
                window.content.push(String::from("=== File Manager ==="));
                window.content.push(String::from("Path: /"));
                window.content.push(String::from(""));
                window.content.push(String::from("  Name              Type       Size    Program"));
                window.content.push(String::from("  ────────────────────────────────────────────"));
                window.file_path = Some(String::from("/"));
                // Initialize Explorer-style state
                let mut fm_state = FileManagerState::new();
                fm_state.push_history("/");
                self.fm_states.insert(window.id, fm_state);
                // List actual files from ramfs with file type info
                if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(Some("/"))) {
                    for (name, ftype, size) in entries.iter().take(50) {
                        let icon = if *ftype == crate::ramfs::FileType::Directory { 
                            "[D]" 
                        } else { 
                            crate::file_assoc::get_file_icon(name)
                        };
                        let prog = if *ftype == crate::ramfs::FileType::Directory {
                            String::from("---")
                        } else {
                            String::from(crate::file_assoc::get_program_for_file(name).name())
                        };
                        let ftype_str = if *ftype == crate::ramfs::FileType::Directory { "DIR" } else { "FILE" };
                        window.content.push(format!("  {} {:<14} {:<10} {:<7} {}", icon, name, ftype_str, size, prog));
                    }
                }
                if window.content.len() <= 5 {
                    window.content.push(String::from("  (empty directory)"));
                }
                window.content.push(String::from(""));
                window.content.push(String::from("  [Enter] Open | [Up/Down] Navigate"));
            },
            WindowType::TextEditor => {
                // Editor state is managed in editor_states BTreeMap
                // Initialize with a new empty editor, assign a default file path
                let mut editor = EditorState::new();
                let file_num = self.editor_states.len() + 1;
                let default_name = if file_num == 1 {
                    String::from("untitled.rs")
                } else {
                    alloc::format!("untitled_{}.rs", file_num)
                };
                editor.file_path = Some(default_name);
                editor.language = crate::apps::text_editor::Language::Rust;
                self.editor_states.insert(window.id, editor);
            },
            WindowType::NetworkInfo => {
                // NetScan uses custom GUI drawing — no static content needed
            },
            WindowType::Settings => {
                window.content.push(String::from("=== Settings ==="));
                window.content.push(String::from(""));
                window.content.push(format!("Resolution: {}x{}", self.width, self.height));
                window.content.push(String::from("Theme: Dark Green"));
                window.content.push(String::from(""));
                window.content.push(String::from("--- Animations ---"));
                let anim_status = if animations_enabled() { "ON " } else { "OFF" };
                let anim_speed = *ANIMATION_SPEED.lock();
                window.content.push(format!("[1] Animations: {}", anim_status));
                window.content.push(format!("[2] Speed: {:.1}x", anim_speed));
                window.content.push(String::from(""));
                window.content.push(String::from("--- Accessibility ---"));
                let hc = if crate::accessibility::is_high_contrast() { "ON " } else { "OFF" };
                window.content.push(format!("[5] High Contrast: {}", hc));
                window.content.push(format!("[6] Font Size: {}", crate::accessibility::get_font_size().label()));
                window.content.push(format!("[7] Cursor Size: {}", crate::accessibility::get_cursor_size().label()));
                window.content.push(format!("[8] Sticky Keys: {}", if crate::accessibility::is_sticky_keys() { "ON" } else { "OFF" }));
                window.content.push(format!("[9] Mouse Speed: {}", crate::accessibility::get_mouse_speed().label()));
                window.content.push(String::from(""));
                window.content.push(String::from("--- Other ---"));
                window.content.push(String::from("[3] File Associations"));
                window.content.push(String::from("[4] About System"));
            },
            WindowType::ImageViewer => {
                window.content.push(String::from("=== Image Viewer ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("No image loaded"));
                window.content.push(String::from(""));
                window.content.push(String::from("Supported: PNG, JPG, BMP, GIF"));
            },
            WindowType::HexViewer => {
                window.content.push(String::from("=== Hex Viewer ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("No file loaded"));
            },
            WindowType::Demo3D => {
                window.content.push(String::from("=== 3D Graphics Demo ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("TrustOS Graphics Engine"));
                window.content.push(String::from("Software 3D Renderer"));
                window.content.push(String::from(""));
                window.content.push(String::from("Features:"));
                window.content.push(String::from("- Wireframe/Solid/Mixed modes"));
                window.content.push(String::from("- Z-buffer depth testing"));
                window.content.push(String::from("- Flat shading with lighting"));
                window.content.push(String::from("- Perspective projection"));
                window.content.push(String::from("- Backface culling"));
                window.content.push(String::from(""));
                window.content.push(String::from("[Rotating Cube Demo Below]"));
            },
            WindowType::FileAssociations => {
                window.content.push(String::from("=== File Associations ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("Extension | Program       | Type"));
                window.content.push(String::from("----------|---------------|-------------"));
                // Load associations
                let assocs = crate::file_assoc::list_associations();
                for (ext, prog, desc) in assocs.iter().take(15) {
                    window.content.push(format!(".{:<8} | {:<13} | {}", ext, prog, desc));
                }
                window.content.push(String::from(""));
                window.content.push(String::from("Click extension to change program"));
            },
            WindowType::Browser => {
                // Initialize browser if not already done
                if self.browser.is_none() {
                    self.browser = Some(crate::browser::Browser::new(width, height));
                }
                self.browser_url_input = String::from("http://example.com");
                    self.browser_url_cursor = self.browser_url_input.len();
            },
            WindowType::ModelEditor => {
                let state = crate::model_editor::ModelEditorState::new();
                self.model_editor_states.insert(window.id, state);
            },
            WindowType::Game => {
                self.snake_states.insert(window.id, SnakeState::new());
            },
            WindowType::Game3D => {
                self.game3d_states.insert(window.id, crate::game3d::Game3DState::new());
            },
            WindowType::Chess => {
                self.chess_states.insert(window.id, crate::chess::ChessState::new());
            },
            WindowType::Chess3D => {
                self.chess3d_states.insert(window.id, crate::chess3d::Chess3DState::new());
            },
            #[cfg(feature = "emulators")]
            WindowType::NesEmu => {
                let mut emu = crate::nes::NesEmulator::new();
                // Auto-load embedded ROM if available
                if let Some(rom_data) = crate::embedded_roms::nes_rom() {
                    emu.load_rom(rom_data);
                }
                self.nes_states.insert(window.id, emu);
            },
            #[cfg(feature = "emulators")]
            WindowType::GameBoyEmu => {
                let mut emu = crate::gameboy::GameBoyEmulator::new();
                // Auto-load embedded ROM if available
                if let Some(rom_data) = crate::embedded_roms::gb_rom() {
                    emu.load_rom(rom_data);
                }
                self.gameboy_states.insert(window.id, emu);
            },
            WindowType::BinaryViewer => {
                // State is inserted externally via open_binary_viewer()
            },
            WindowType::LabMode => {
                self.lab_states.insert(window.id, crate::lab_mode::LabState::new());
            },
            #[cfg(feature = "emulators")]
            WindowType::GameLab => {
                self.gamelab_states.insert(window.id, crate::game_lab::GameLabState::new());
            },
            WindowType::MusicPlayer => {
                crate::serial_println!("[Desktop] Creating MusicPlayer state for window {}", window.id);
                let mut mp = MusicPlayerState::new();
                mp.load_track_list();
                self.music_player_states.insert(window.id, mp);
                crate::serial_println!("[Desktop] MusicPlayer state created OK");
            },
            _ => {}
        }
        
        // Start opening animation
        window.animate_open();
        
        let id = window.id;
        self.windows.push(window);
        id
    }
    
    /// Close a window (with animation if enabled)
    pub fn close_window(&mut self, id: u32) {
        crate::serial_println!("[GUI] close_window({}) start", id);
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            if w.animate_close() {
                crate::serial_println!("[GUI] close_window({}) animate path", id);
                // Animation started — immediately free heavyweight emulator/game states
                // (the window chrome will still animate, but we don't need the state)
                #[cfg(feature = "emulators")]
                self.gameboy_states.remove(&id);
                #[cfg(feature = "emulators")]
                self.nes_states.remove(&id);
                self.game3d_states.remove(&id);
                self.chess3d_states.remove(&id);
                #[cfg(feature = "emulators")]
                self.gamelab_states.remove(&id);
                self.lab_states.remove(&id);
                // Stop music playback on close
                if let Some(mp) = self.music_player_states.get_mut(&id) {
                    crate::serial_println!("[GUI] close_window({}) stopping music...", id);
                    mp.stop();
                    crate::serial_println!("[GUI] close_window({}) music stopped", id);
                }
                crate::serial_println!("[GUI] close_window({}) removing mp state...", id);
                self.music_player_states.remove(&id);
                crate::serial_println!("[GUI] close_window({}) animate path done", id);
                return;
            }
        }
        crate::serial_println!("[GUI] close_window({}) immediate remove path", id);
        // No animation, remove immediately
        self.windows.retain(|w| w.id != id);
        // Clean up states
        self.editor_states.remove(&id);
        self.model_editor_states.remove(&id);
        self.calculator_states.remove(&id);
        self.snake_states.remove(&id);
        self.game3d_states.remove(&id);
        self.chess_states.remove(&id);
        self.chess3d_states.remove(&id);
        #[cfg(feature = "emulators")]
        self.nes_states.remove(&id);
        #[cfg(feature = "emulators")]
        self.gameboy_states.remove(&id);
        self.binary_viewer_states.remove(&id);
        self.lab_states.remove(&id);
        if let Some(mp) = self.music_player_states.get_mut(&id) {
            crate::serial_println!("[GUI] close_window({}) stopping music (imm)...", id);
            mp.stop();
            crate::serial_println!("[GUI] close_window({}) music stopped (imm)", id);
        }
        crate::serial_println!("[GUI] close_window({}) removing mp state (imm)...", id);
        self.music_player_states.remove(&id);
        crate::serial_println!("[GUI] close_window({}) immediate path done", id);
        #[cfg(feature = "emulators")]
        self.gamelab_states.remove(&id);
        #[cfg(feature = "emulators")]
        self.gb_input_links.remove(&id);
    }
    
    /// Minimize/restore a window (with animation)
    pub fn minimize_window(&mut self, id: u32) {
        let taskbar_y = (self.height - TASKBAR_HEIGHT) as i32;
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            if !w.minimized {
                w.animate_minimize(taskbar_y);
            }
            w.minimized = !w.minimized;
        }
    }
    
    /// Update all window animations (call each frame)
    pub fn update_animations(&mut self) {
        let mut to_remove = Vec::new();
        
        for w in &mut self.windows {
            if w.update_animation() {
                // Animation completed and window should close
                to_remove.push(w.id);
            }
        }
        
        // Remove windows that finished closing animation
        for id in to_remove {
            self.windows.retain(|w| w.id != id);
            self.editor_states.remove(&id);
            self.model_editor_states.remove(&id);
            self.game3d_states.remove(&id);
            self.chess3d_states.remove(&id);
            #[cfg(feature = "emulators")]
            self.nes_states.remove(&id);
            #[cfg(feature = "emulators")]
            self.gameboy_states.remove(&id);
            #[cfg(feature = "emulators")]
            self.gamelab_states.remove(&id);
        }
    }
    
    /// Focus a window (bring to front)
    pub fn focus_window(&mut self, id: u32) {
        for w in &mut self.windows {
            w.focused = false;
        }
        if let Some(idx) = self.windows.iter().position(|w| w.id == id) {
            let mut window = self.windows.remove(idx);
            window.focused = true;
            window.minimized = false;
            self.windows.push(window);
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════════════
    // NEW: Windows-like features for hotkeys and GUI engine
    // ═══════════════════════════════════════════════════════════════════════════════
    
    /// Get screen dimensions
    pub fn screen_width(&self) -> u32 { self.width }
    pub fn screen_height(&self) -> u32 { self.height }
    
    /// Close the currently focused window (Alt+F4)
    pub fn close_focused_window(&mut self) {
        if let Some(id) = self.windows.iter().rev().find(|w| w.focused).map(|w| w.id) {
            self.close_window(id);
        }
    }
    
    /// Minimize the currently focused window (Win+Down)
    pub fn minimize_focused_window(&mut self) {
        if let Some(id) = self.windows.iter().rev().find(|w| w.focused).map(|w| w.id) {
            self.minimize_window(id);
        }
    }
    
    /// Toggle maximize on focused window (Win+Up)
    pub fn toggle_maximize_focused(&mut self) {
        if let Some(id) = self.windows.iter().rev().find(|w| w.focused).map(|w| w.id) {
            let (sw, sh) = (self.width, self.height);
            if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                w.toggle_maximize(sw, sh);
            }
        }
    }
    
    /// Snap focused window to left/right/quadrant (Win+Arrow or drag-to-edge)
    pub fn snap_focused_window(&mut self, dir: SnapDir) {
        if let Some(w) = self.windows.iter_mut().rev().find(|w| w.focused) {
            let work_height = self.height.saturating_sub(TASKBAR_HEIGHT);
            let work_x = DOCK_WIDTH as i32;
            let work_w = self.width.saturating_sub(DOCK_WIDTH);
            let half_w = work_w / 2;
            let half_h = work_height / 2;
            
            match dir {
                SnapDir::Left => {
                    w.x = work_x;
                    w.y = 0;
                    w.width = half_w;
                    w.height = work_height;
                }
                SnapDir::Right => {
                    w.x = work_x + half_w as i32;
                    w.y = 0;
                    w.width = half_w;
                    w.height = work_height;
                }
                SnapDir::TopLeft => {
                    w.x = work_x;
                    w.y = 0;
                    w.width = half_w;
                    w.height = half_h;
                }
                SnapDir::TopRight => {
                    w.x = work_x + half_w as i32;
                    w.y = 0;
                    w.width = half_w;
                    w.height = half_h;
                }
                SnapDir::BottomLeft => {
                    w.x = work_x;
                    w.y = half_h as i32;
                    w.width = half_w;
                    w.height = half_h;
                }
                SnapDir::BottomRight => {
                    w.x = work_x + half_w as i32;
                    w.y = half_h as i32;
                    w.width = half_w;
                    w.height = half_h;
                }
            }
            w.maximized = false;
        }
    }
    
    /// Toggle show desktop (Win+D) - minimize/restore all windows
    pub fn toggle_show_desktop(&mut self) {
        // Check if all windows are minimized
        let all_minimized = self.windows.iter().all(|w| w.minimized);
        
        // Toggle all windows
        for w in &mut self.windows {
            w.minimized = !all_minimized;
        }
    }
    
    /// Focus window by index (for Alt+Tab)
    pub fn focus_window_by_index(&mut self, index: usize) {
        if index < self.windows.len() {
            // Get visible (non-minimized) windows
            let visible: Vec<u32> = self.windows.iter()
                .filter(|w| !w.minimized)
                .map(|w| w.id)
                .collect();
            
            if index < visible.len() {
                self.focus_window(visible[index]);
            }
        }
    }
    
    /// Get list of window titles for Alt+Tab display
    pub fn get_window_titles(&self) -> Vec<String> {
        self.windows.iter()
            .filter(|w| !w.minimized)
            .map(|w| w.title.clone())
            .collect()
    }
    
    /// Get window info (title, type) for Alt+Tab overlay
    pub fn get_window_info(&self) -> Vec<(String, WindowType)> {
        self.windows.iter()
            .filter(|w| !w.minimized)
            .map(|w| (w.title.clone(), w.window_type.clone()))
            .collect()
    }
    
    /// Open a new terminal window
    pub fn open_terminal(&mut self) {
        let id = self.create_window("Terminal", 100, 60, 780, 540, WindowType::Terminal);
        self.focus_window(id);
    }

    /// Handle mouse click
    pub fn handle_click(&mut self, x: i32, y: i32, pressed: bool) {
        // ════════ MOBILE MODE: route clicks to mobile gesture system ════════
        if self.mobile_state.active {
            let vx = self.mobile_state.vp_x;
            let vy = self.mobile_state.vp_y;
            let vw = self.mobile_state.vp_w as i32;
            let vh = self.mobile_state.vp_h as i32;
            // Check if click is inside viewport
            if x >= vx && x < vx + vw && y >= vy && y < vy + vh {
                let local_x = x - vx;
                let local_y = y - vy;
                let evt = if pressed {
                    crate::mobile::GestureEvent::TapDown(local_x, local_y)
                } else {
                    crate::mobile::GestureEvent::TapUp(local_x, local_y)
                };
                let action = crate::mobile::handle_gesture(&mut self.mobile_state, evt);
                self.apply_mobile_action(action);
            }
            return;
        }
        
        // Lock screen blocks all mouse interaction
        if self.lock_screen_active { return; }
        
        // Update drag position if dragging a file
        if !pressed && self.drag_state.is_some() {
            self.finish_drag(x, y);
            return;
        }
        if pressed && self.drag_state.is_some() {
            self.update_drag(x, y);
        }
        
        if pressed {
            // Close context menu on any left click
            if self.context_menu.visible {
                if let Some(action) = self.check_context_menu_click(x, y) {
                    self.execute_context_action(action);
                }
                self.context_menu.visible = false;
                return;
            }
            
            // Record click for double-click detection
            crate::mouse::record_click();
            
            // Check start menu clicks first
            if self.start_menu_open {
                if let Some(action) = self.check_start_menu_click(x, y) {
                    self.start_menu_open = false;
                    self.start_menu_search.clear();
                    self.handle_menu_action(action);
                    return;
                }
                // Click outside menu (but not on taskbar TrustOS button) → close menu
                if y < (self.height - TASKBAR_HEIGHT) as i32 || x >= 108 {
                    self.start_menu_open = false;
                    self.start_menu_search.clear();
                    return;
                }
            }
            
            // Check taskbar first
            if y >= (self.height - TASKBAR_HEIGHT) as i32 {
                self.handle_taskbar_click(x, y);
                return;
            }
            
            // Check windows from top to bottom
            for i in (0..self.windows.len()).rev() {
                if self.windows[i].contains(x, y) {
                    let id = self.windows[i].id;
                    
                    if self.windows[i].on_close_button(x, y) {
                        self.close_window(id);
                        return;
                    }
                    
                    if self.windows[i].on_maximize_button(x, y) {
                        let (sw, sh) = (self.width, self.height);
                        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                            w.toggle_maximize(sw, sh);
                        }
                        return;
                    }
                    
                    if self.windows[i].on_minimize_button(x, y) {
                        self.minimize_window(id);
                        return;
                    }
                    
                    // Check for resize edge — only side/bottom edges
                    // Top border is treated as drag area for easier window moving
                    let resize_edge = self.windows[i].on_resize_edge(x, y);
                    let is_top_border = matches!(resize_edge, ResizeEdge::Top | ResizeEdge::TopLeft | ResizeEdge::TopRight);
                    if resize_edge != ResizeEdge::None && !is_top_border {
                        self.windows[i].resizing = resize_edge;
                        self.windows[i].drag_offset_x = x;
                        self.windows[i].drag_offset_y = y;
                        self.focus_window(id);
                        return;
                    }
                    
                    // Title bar OR top border → drag to move (double-click to maximize)
                    if self.windows[i].in_title_bar(x, y) || is_top_border {
                        // Double-click to toggle maximize
                        if crate::mouse::is_double_click() {
                            crate::mouse::reset_click_count();
                            let (sw, sh) = (self.width, self.height);
                            if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                                w.toggle_maximize(sw, sh);
                            }
                            return;
                        }
                        
                        let win_x = self.windows[i].x;
                        let win_y = self.windows[i].y;
                        self.windows[i].dragging = true;
                        self.windows[i].drag_offset_x = x - win_x;
                        self.windows[i].drag_offset_y = y - win_y;
                    }
                    
                    // Handle browser content clicks
                    if self.windows[i].window_type == WindowType::Browser {
                        crate::serial_println!("[CLICK-DBG] Browser window {} clicked at ({},{})", self.windows[i].id, x, y);
                        let bx = self.windows[i].x;
                        let by = self.windows[i].y;
                        let bw = self.windows[i].width;
                        self.handle_browser_click(x, y, bx, by, bw);
                    }
                    
                    // Handle file manager content clicks (mouse navigation + double-click open)
                    if self.windows[i].window_type == WindowType::FileManager {
                        let fm_id = self.windows[i].id;
                        self.handle_file_manager_click(x, y, fm_id);
                    }
                    
                    // Handle model editor clicks
                    if self.windows[i].window_type == WindowType::ModelEditor {
                        let win = &self.windows[i];
                        let vx = x - win.x;
                        let vy = y - win.y - TITLE_BAR_HEIGHT as i32;
                        let vw = win.width as usize;
                        let vh = win.height.saturating_sub(TITLE_BAR_HEIGHT) as usize;
                        let win_id = win.id;
                        if vy >= 0 {
                            if let Some(state) = self.model_editor_states.get_mut(&win_id) {
                                state.handle_click(vx, vy, vw, vh, true);
                            }
                        }
                    }
                    
                    // Handle chess board clicks (mouse selection & drag start)
                    if self.windows[i].window_type == WindowType::Chess {
                        let win = &self.windows[i];
                        let game_x = win.x as i32 + 8;
                        let game_y = win.y as i32 + TITLE_BAR_HEIGHT as i32 + 4;
                        let game_w = win.width.saturating_sub(16) as i32;
                        let cell_size: i32 = 48;
                        let board_size = cell_size * 8;
                        let board_x = game_x + (game_w - board_size) / 2;
                        let board_y = game_y + 28;
                        
                        let col = (x - board_x) / cell_size;
                        let row = (y - board_y) / cell_size;
                        
                        if x >= board_x && x < board_x + board_size && y >= board_y && y < board_y + board_size && col >= 0 && col < 8 && row >= 0 && row < 8 {
                            let win_id = win.id;
                            if let Some(chess) = self.chess_states.get_mut(&win_id) {
                                chess.handle_mouse_click(col, row);
                                chess.update_drag_position(x, y);
                            }
                        }
                    }
                    
                    // Handle 3D chess board clicks
                    if self.windows[i].window_type == WindowType::Chess3D {
                        let win = &self.windows[i];
                        let content_x = win.x as i32;
                        let content_y = win.y as i32 + TITLE_BAR_HEIGHT as i32;
                        let content_w = win.width as i32;
                        let content_h = win.height.saturating_sub(TITLE_BAR_HEIGHT) as i32;
                        let rel_x = x - content_x;
                        let rel_y = y - content_y;
                        if rel_x >= 0 && rel_y >= 0 && rel_x < content_w && rel_y < content_h {
                            let win_id = win.id;
                            if let Some(state) = self.chess3d_states.get_mut(&win_id) {
                                state.handle_click(rel_x, rel_y, content_w, content_h);
                            }
                        }
                    }
                    
                    // Handle lab mode panel clicks
                    if self.windows[i].window_type == WindowType::LabMode {
                        let win = &self.windows[i];
                        let rel_x = x - win.x;
                        let rel_y = y - win.y;
                        let win_id = win.id;
                        let ww = win.width;
                        let wh = win.height;
                        if let Some(lab) = self.lab_states.get_mut(&win_id) {
                            lab.handle_click(rel_x, rel_y, ww, wh);
                        }
                    }

                    // Handle Game Boy menu bar clicks (LAB / INPUT buttons)
                    #[cfg(feature = "emulators")]
                    if self.windows[i].window_type == WindowType::GameBoyEmu {
                        let win = &self.windows[i];
                        let content_x = win.x as u32;
                        let content_y = (win.y + TITLE_BAR_HEIGHT as i32) as u32;
                        let content_w = win.width;
                        let menu_h: u32 = 22;
                        let win_id = win.id;
                        let win_x = win.x;
                        let win_y = win.y;
                        let win_w = win.width;
                        let win_h = win.height;
                        let mx = x as u32;
                        let my = y as u32;
                        
                        // Check if click is in menu bar area
                        if my >= content_y && my < content_y + menu_h {
                            // [INPUT] button (right side)
                            let inp_btn_w: u32 = 48;
                            let inp_btn_x = content_x + content_w - inp_btn_w - 4;
                            if mx >= inp_btn_x && mx < inp_btn_x + inp_btn_w {
                                // Open input window below the GB window
                                let inp_x = win_x;
                                let inp_y = win_y + win_h as i32 + 2;
                                let inp_id = self.create_window("GB Input", inp_x, inp_y, win_w.min(480), 160, WindowType::GameBoyInput);
                                self.gb_input_links.insert(inp_id, win_id);
                            }
                            
                            // [LAB] button
                            let lab_btn_w: u32 = 32;
                            let lab_btn_x = inp_btn_x - lab_btn_w - 6;
                            if mx >= lab_btn_x && mx < lab_btn_x + lab_btn_w {
                                // Open GameLab to the right of the Game Boy window
                                let sw = self.width;
                                let sh = self.height;
                                let lab_x = win_x + win_w as i32 + 4;
                                let lab_w = (sw as i32 - lab_x).max(400) as u32;
                                let lab_h = sh - TASKBAR_HEIGHT;
                                let lab_id = self.create_window("Game Lab", lab_x, 0, lab_w, lab_h, WindowType::GameLab);
                                if let Some(lab) = self.gamelab_states.get_mut(&lab_id) {
                                    lab.linked_gb_id = Some(win_id);
                                }
                                self.focus_window(lab_id);
                            }
                        }
                    }

                    // Handle Game Boy Input window clicks (forward to linked emulator)
                    #[cfg(feature = "emulators")]
                    if self.windows[i].window_type == WindowType::GameBoyInput {
                        let win = &self.windows[i];
                        let cx = win.x as u32;
                        let cy = (win.y + TITLE_BAR_HEIGHT as i32) as u32;
                        let cw = win.width;
                        let ch = win.height.saturating_sub(TITLE_BAR_HEIGHT);
                        let win_id = win.id;
                        let mx = x as u32;
                        let my = y as u32;
                        
                        let linked_id = self.gb_input_links.get(&win_id).copied();
                        let buttons = crate::game_lab::get_input_buttons(cx, cy, cw, ch);
                        for &(bx, by, bw, bh, key) in &buttons {
                            if mx >= bx && mx < bx + bw && my >= by && my < by + bh {
                                // Find the linked GB emulator
                                let emu_id = linked_id.or_else(|| self.gameboy_states.keys().next().copied());
                                if let Some(eid) = emu_id {
                                    if let Some(emu) = self.gameboy_states.get_mut(&eid) {
                                        emu.handle_key(key);
                                    }
                                }
                                break;
                            }
                        }
                    }

                    // Handle GameLab panel clicks
                    #[cfg(feature = "emulators")]
                    if self.windows[i].window_type == WindowType::GameLab {
                        let win = &self.windows[i];
                        let rel_x = x - win.x;
                        let rel_y = y - win.y;
                        let win_id = win.id;
                        let ww = win.width;
                        let wh = win.height;
                        if let Some(lab) = self.gamelab_states.get_mut(&win_id) {
                            // Check Save/Load header button clicks
                            let save_rx = ww as i32 - 120;
                            if rel_y >= TITLE_BAR_HEIGHT as i32 + 2 && rel_y < TITLE_BAR_HEIGHT as i32 + 18 {
                                if rel_x >= save_rx && rel_x < save_rx + 48 {
                                    // SAVE click
                                    let emu_id = lab.linked_gb_id
                                        .or_else(|| self.gameboy_states.keys().next().copied());
                                    if let Some(eid) = emu_id {
                                        if let Some(emu) = self.gameboy_states.get(&eid) {
                                            if let Some(gl) = self.gamelab_states.get_mut(&win_id) {
                                                gl.save_from(emu);
                                                crate::serial_println!("[GameLab] State saved (click)");
                                            }
                                        }
                                    }
                                } else if rel_x >= save_rx + 54 && rel_x < save_rx + 102 {
                                    // LOAD click
                                    let emu_id = lab.linked_gb_id
                                        .or_else(|| self.gameboy_states.keys().next().copied());
                                    if let Some(eid) = emu_id {
                                        let valid = self.gamelab_states.get(&win_id)
                                            .map(|l| l.save_state.valid).unwrap_or(false);
                                        if valid {
                                            if let Some(emu) = self.gameboy_states.get_mut(&eid) {
                                                if let Some(gl) = self.gamelab_states.get(&win_id) {
                                                    gl.load_into(emu);
                                                    crate::serial_println!("[GameLab] State loaded (click)");
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                lab.handle_click(rel_x, rel_y, ww, wh);
                            }
                        }
                    }

                    // Handle binary viewer clicks
                    if self.windows[i].window_type == WindowType::BinaryViewer {
                        let win = &self.windows[i];
                        let rel_x = x - win.x;
                        let rel_y = y - win.y;
                        let win_id = win.id;
                        let ww = win.width;
                        let wh = win.height;
                        if let Some(viewer) = self.binary_viewer_states.get_mut(&win_id) {
                            viewer.handle_click(rel_x, rel_y, ww, wh);
                        }
                    }
                    
                    // Handle calculator button clicks
                    if self.windows[i].window_type == WindowType::Calculator {
                        let win = &self.windows[i];
                        let cx_start = win.x as u32 + 4;
                        let cy_start = win.y as u32 + TITLE_BAR_HEIGHT + 4;
                        let cw = win.width.saturating_sub(8);
                        let ch = win.height.saturating_sub(TITLE_BAR_HEIGHT + 8);
                        let display_h = 56u32;
                        let btn_area_y = cy_start + display_h + 12;
                        let btn_cols = 4u32;
                        let btn_rows = 5u32;
                        let btn_gap = 4u32;
                        let btn_w = (cw - 12 - btn_gap * (btn_cols - 1)) / btn_cols;
                        let btn_h = ((ch - display_h - 20 - btn_gap * (btn_rows - 1)) / btn_rows).min(40);
                        
                        let click_x = x as u32;
                        let click_y = y as u32;
                        
                        if click_y >= btn_area_y {
                            let buttons = [
                                ['C', '(', ')', '%'],
                                ['7', '8', '9', '/'],
                                ['4', '5', '6', '*'],
                                ['1', '2', '3', '-'],
                                ['0', '.', '=', '+'],
                            ];
                            
                            for (row, btn_row) in buttons.iter().enumerate() {
                                for (col, &label) in btn_row.iter().enumerate() {
                                    let bx = cx_start + 4 + col as u32 * (btn_w + btn_gap);
                                    let by = btn_area_y + row as u32 * (btn_h + btn_gap);
                                    
                                    if click_x >= bx && click_x < bx + btn_w && click_y >= by && click_y < by + btn_h {
                                        let win_id = win.id;
                                        if let Some(calc) = self.calculator_states.get_mut(&win_id) {
                                            match label {
                                                '0'..='9' => calc.press_digit(label),
                                                '.' => calc.press_dot(),
                                                '+' => calc.press_operator('+'),
                                                '-' => calc.press_operator('-'),
                                                '*' => calc.press_operator('*'),
                                                '/' => calc.press_operator('/'),
                                                '%' => calc.press_operator('%'),
                                                '=' => calc.press_equals(),
                                                'C' => calc.press_clear(),
                                                '(' => calc.press_paren('('),
                                                ')' => calc.press_paren(')'),
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Handle music player clicks (track list + transport + volume)
                    if self.windows[i].window_type == WindowType::MusicPlayer {
                        let win = &self.windows[i];
                        let wx = win.x as u32;
                        let wy = win.y as u32 + TITLE_BAR_HEIGHT;
                        let ww = win.width;
                        let pad = 10u32;
                        let inner_x = wx + pad;
                        let inner_w = ww.saturating_sub(pad * 2);

                        let click_x = x as u32;
                        let click_y = y as u32;
                        let win_id = win.id;

                        // ── Layout positions (must match draw_music_player) ──
                        let num_tracks = self.music_player_states.get(&win_id)
                            .map(|mp| mp.num_tracks).unwrap_or(0);
                        let list_header_y = wy + 6;
                        let list_y = list_header_y + 16;
                        let max_visible = 5usize;
                        let row_h = 20u32;
                        let list_h = if num_tracks == 0 { row_h } else { (num_tracks.min(max_visible) as u32) * row_h };

                        let np_y = list_y + list_h + 10;
                        let song_y = np_y + 16;
                        let status_y = song_y + 16;
                        let prog_y = status_y + 18;
                        let viz_y = prog_y + 12;
                        let viz_h = 60u32;
                        let bars_y = viz_y + viz_h + 4;
                        let bar_h = 14u32;
                        let ctrl_y = bars_y + bar_h + 8;
                        let btn_h = 28u32;
                        let small_btn_w = 36u32;
                        let play_btn_w = 64u32;
                        let gap = 4u32;

                        // ── Track list click ──
                        if num_tracks > 0
                            && click_x >= inner_x && click_x < inner_x + inner_w
                            && click_y >= list_y && click_y < list_y + list_h
                        {
                            let scroll = self.music_player_states.get(&win_id)
                                .map(|mp| mp.track_list_scroll.min(mp.num_tracks.saturating_sub(max_visible)))
                                .unwrap_or(0);
                            let row_idx = ((click_y - list_y) / row_h) as usize;
                            let track_idx = scroll + row_idx;
                            if track_idx < num_tracks {
                                crate::serial_println!("[MUSIC] Track list click: track {}", track_idx);
                                if let Some(mp) = self.music_player_states.get_mut(&win_id) {
                                    mp.play_track(track_idx);
                                }
                                self.fps_low_count = 0; // disk I/O dip is transient
                            }
                        }

                        // ── Transport buttons (centered, fixed-size) ──
                        let total_transport_w = small_btn_w * 3 + play_btn_w + gap * 3;
                        let transport_x = inner_x + (inner_w.saturating_sub(total_transport_w)) / 2;
                        if click_y >= ctrl_y && click_y < ctrl_y + btn_h {
                            // |< Previous
                            let prev_x = transport_x;
                            if click_x >= prev_x && click_x < prev_x + small_btn_w {
                                if let Some(mp) = self.music_player_states.get_mut(&win_id) {
                                    mp.prev_track();
                                }
                            }
                            // PLAY/PAUSE
                            let play_x = prev_x + small_btn_w + gap;
                            if click_x >= play_x && click_x < play_x + play_btn_w {
                                if let Some(mp) = self.music_player_states.get_mut(&win_id) {
                                    match mp.state {
                                        PlaybackState::Stopped => {
                                            mp.play_track(mp.current_track);
                                            self.fps_low_count = 0;
                                        },
                                        PlaybackState::Playing | PlaybackState::Paused => mp.toggle_pause(),
                                    }
                                }
                            }
                            // STOP
                            let stop_x = play_x + play_btn_w + gap;
                            if click_x >= stop_x && click_x < stop_x + small_btn_w {
                                if let Some(mp) = self.music_player_states.get_mut(&win_id) {
                                    mp.stop();
                                }
                            }
                            // >| Next
                            let next_x = stop_x + small_btn_w + gap;
                            if click_x >= next_x && click_x < next_x + small_btn_w {
                                if let Some(mp) = self.music_player_states.get_mut(&win_id) {
                                    mp.next_track();
                                }
                            }
                        }

                        // ── Progress bar seek ──
                        if click_x >= inner_x && click_x < inner_x + inner_w
                            && click_y >= prog_y.saturating_sub(3) && click_y < prog_y + 8 {
                            if let Some(mp) = self.music_player_states.get_mut(&win_id) {
                                if mp.total_ms > 0 && mp.state != PlaybackState::Stopped {
                                    let rel = (click_x - inner_x) as f32 / inner_w.max(1) as f32;
                                    let new_ms = (rel * mp.total_ms as f32) as u64;
                                    mp.seek_to(new_ms);
                                }
                            }
                        }

                        // ── Volume slider ──
                        let vol_y = ctrl_y + btn_h + 8;
                        let vol_h = 10u32;
                        let vol_track_x = inner_x + 30;
                        let vol_track_w = inner_w.saturating_sub(72);
                        if click_x >= vol_track_x && click_x < vol_track_x + vol_track_w
                            && click_y >= vol_y.saturating_sub(4) && click_y < vol_y + vol_h + 4 {
                            let rel = (click_x - vol_track_x) as f32 / vol_track_w.max(1) as f32;
                            let new_vol = (rel * 100.0).max(0.0).min(100.0) as u32;
                            if let Some(mp) = self.music_player_states.get_mut(&win_id) {
                                mp.volume = new_vol;
                                let _ = crate::drivers::hda::set_volume(new_vol.min(100) as u8);
                            }
                        }

                        // ── Effects controls (SYNC / VIZ / PAL / RAIN) ──
                        let fx_y = vol_y + vol_h + 10;
                        let fx_start_y = fx_y + 4;
                        let fx_row_h = 24u32;
                        let arrow_w = 24u32;
                        let label_w = 36u32;
                        let fx_ctrl_x = inner_x + label_w + 4;

                        // SYNC row
                        let sync_y = fx_start_y + 16;
                        if click_y >= sync_y && click_y < sync_y + fx_row_h {
                            // [-] button
                            if click_x >= fx_ctrl_x && click_x < fx_ctrl_x + arrow_w {
                                if let Some(mp) = self.music_player_states.get_mut(&win_id) {
                                    mp.av_offset_ms = (mp.av_offset_ms - 10).max(-500);
                                }
                            }
                            let sync_plus_x = fx_ctrl_x + arrow_w + 4 + 52 + 4;
                            // [+] button
                            if click_x >= sync_plus_x && click_x < sync_plus_x + arrow_w {
                                if let Some(mp) = self.music_player_states.get_mut(&win_id) {
                                    mp.av_offset_ms = (mp.av_offset_ms + 10).min(500);
                                }
                            }
                            // [0] reset button
                            let sync_reset_x = sync_plus_x + arrow_w + 4;
                            if click_x >= sync_reset_x && click_x < sync_reset_x + arrow_w {
                                if let Some(mp) = self.music_player_states.get_mut(&win_id) {
                                    mp.av_offset_ms = 0;
                                }
                            }
                        }

                        // VIZ row
                        let viz_y2 = sync_y + fx_row_h + 4;
                        if click_y >= viz_y2 && click_y < viz_y2 + fx_row_h {
                            let viz_name_w = inner_w.saturating_sub(label_w + 4 + arrow_w * 2 + 12);
                            // [<] prev mode
                            if click_x >= fx_ctrl_x && click_x < fx_ctrl_x + arrow_w {
                                let m = self.visualizer.mode;
                                self.visualizer.mode = if m == 0 { crate::visualizer::NUM_MODES - 1 } else { m - 1 };
                            }
                            // [>] next mode
                            let viz_next_x = fx_ctrl_x + arrow_w + 4 + viz_name_w + 4;
                            if click_x >= viz_next_x && click_x < viz_next_x + arrow_w {
                                self.visualizer.mode = (self.visualizer.mode + 1) % crate::visualizer::NUM_MODES;
                            }
                        }

                        // PAL row
                        let pal_y = viz_y2 + fx_row_h + 4;
                        if click_y >= pal_y && click_y < pal_y + fx_row_h {
                            let pal_name_w = inner_w.saturating_sub(label_w + 4 + arrow_w * 2 + 12);
                            // [<] prev palette
                            if click_x >= fx_ctrl_x && click_x < fx_ctrl_x + arrow_w {
                                let p = self.visualizer.palette;
                                self.visualizer.palette = if p == 0 { crate::visualizer::NUM_PALETTES - 1 } else { p - 1 };
                            }
                            // [>] next palette
                            let pal_next_x = fx_ctrl_x + arrow_w + 4 + pal_name_w + 4;
                            if click_x >= pal_next_x && click_x < pal_next_x + arrow_w {
                                self.visualizer.palette = (self.visualizer.palette + 1) % crate::visualizer::NUM_PALETTES;
                            }
                        }

                        // RAIN row
                        let rain_y = pal_y + fx_row_h + 4;
                        if click_y >= rain_y && click_y < rain_y + fx_row_h {
                            let rain_name_w = inner_w.saturating_sub(label_w + 4 + arrow_w * 2 + 12);
                            // [<] prev preset
                            if click_x >= fx_ctrl_x && click_x < fx_ctrl_x + arrow_w {
                                let p = self.matrix_rain_preset;
                                self.set_rain_preset(if p == 0 { 2 } else { p - 1 });
                            }
                            // [>] next preset
                            let rain_next_x = fx_ctrl_x + arrow_w + 4 + rain_name_w + 4;
                            if click_x >= rain_next_x && click_x < rain_next_x + arrow_w {
                                self.set_rain_preset((self.matrix_rain_preset + 1) % 3);
                            }
                        }
                    }
                    
                    self.focus_window(id);
                    return;
                }
            }
            
            // Check desktop icons - single click to open
            if let Some(idx) = self.check_icon_index(x, y) {
                let action = self.icons[idx].action;
                self.handle_icon_action(action);
                return;
            }
            
            self.start_menu_open = false;
            self.start_menu_search.clear();
        } else {
            // Mouse released - stop dragging and snap if preview active
            let snap_dir = self.snap_preview.take();
            let mut snapped_id: Option<u32> = None;
            for w in &mut self.windows {
                if w.dragging {
                    if let Some(dir) = snap_dir {
                        // Apply snap: resize window to preview zone (accounting for dock)
                        let work_h = self.height.saturating_sub(TASKBAR_HEIGHT);
                        let work_x = DOCK_WIDTH as i32;
                        let work_w = self.width.saturating_sub(DOCK_WIDTH);
                        let half_w = work_w / 2;
                        let half_h = work_h / 2;
                        match dir {
                            SnapDir::Left => { w.x = work_x; w.y = 0; w.width = half_w; w.height = work_h; }
                            SnapDir::Right => { w.x = work_x + half_w as i32; w.y = 0; w.width = half_w; w.height = work_h; }
                            SnapDir::TopLeft => { w.x = work_x; w.y = 0; w.width = half_w; w.height = half_h; }
                            SnapDir::TopRight => { w.x = work_x + half_w as i32; w.y = 0; w.width = half_w; w.height = half_h; }
                            SnapDir::BottomLeft => { w.x = work_x; w.y = half_h as i32; w.width = half_w; w.height = half_h; }
                            SnapDir::BottomRight => { w.x = work_x + half_w as i32; w.y = half_h as i32; w.width = half_w; w.height = half_h; }
                        }
                        w.maximized = false;
                        snapped_id = Some(w.id);
                    }
                }
                w.dragging = false;
                w.resizing = ResizeEdge::None;
            }
            
            // Handle file drag-and-drop release
            if self.drag_state.is_some() {
                self.finish_drag(x, y);
            }
            
            // Notify model editors about mouse release
            let model_ids: Vec<u32> = self.windows.iter()
                .filter(|w| w.window_type == WindowType::ModelEditor && w.focused)
                .map(|w| w.id)
                .collect();
            for id in model_ids {
                if let Some(state) = self.model_editor_states.get_mut(&id) {
                    state.handle_click(0, 0, 0, 0, false);
                }
            }
            
            // Notify chess about mouse release (drag & drop)
            let chess_ids: Vec<u32> = self.windows.iter()
                .filter(|w| w.window_type == WindowType::Chess && w.focused)
                .map(|w| w.id)
                .collect();
            for id in chess_ids {
                if let Some(chess) = self.chess_states.get_mut(&id) {
                    if chess.drag_from.is_some() {
                        // Find the window to compute board coordinates
                        if let Some(win) = self.windows.iter().find(|w| w.id == id) {
                            let game_x = win.x as i32 + 8;
                            let game_y = win.y as i32 + TITLE_BAR_HEIGHT as i32 + 4;
                            let game_w = win.width.saturating_sub(16) as i32;
                            let cell_size: i32 = 48;
                            let board_size = cell_size * 8;
                            let board_x = game_x + (game_w - board_size) / 2;
                            let board_y = game_y + 28;
                            
                            let col = (x - board_x) / cell_size;
                            let row = (y - board_y) / cell_size;
                            chess.handle_mouse_release(col, row);
                        }
                    }
                }
            }
            
            // Notify Chess3D about mouse release (stop drag rotation)
            let chess3d_ids: Vec<u32> = self.windows.iter()
                .filter(|w| w.window_type == WindowType::Chess3D && w.focused)
                .map(|w| w.id)
                .collect();
            for id in chess3d_ids {
                if let Some(state) = self.chess3d_states.get_mut(&id) {
                    state.handle_mouse_release();
                }
            }
            
            // Release all input buttons on Game Boy Input windows on mouse up
            #[cfg(feature = "emulators")]
            {
            let input_ids: Vec<(u32, Option<u32>)> = self.windows.iter()
                .filter(|w| w.window_type == WindowType::GameBoyInput && w.focused)
                .map(|w| (w.id, self.gb_input_links.get(&w.id).copied()))
                .collect();
            for (_iid, linked_id) in input_ids {
                let emu_id = linked_id.or_else(|| self.gameboy_states.keys().next().copied());
                if let Some(eid) = emu_id {
                    if let Some(emu) = self.gameboy_states.get_mut(&eid) {
                        emu.handle_key_release(b'w');
                        emu.handle_key_release(b'a');
                        emu.handle_key_release(b's');
                        emu.handle_key_release(b'd');
                        emu.handle_key_release(b'x');
                        emu.handle_key_release(b'z');
                        emu.handle_key_release(b'c');
                        emu.handle_key_release(b'\r');
                    }
                }
            }
            }
        }
    }
    
    /// Handle right-click (context menu)
    pub fn handle_right_click(&mut self, x: i32, y: i32, pressed: bool) {
        if !pressed {
            return; // Only handle press, not release
        }
        
        // Close any existing context menu
        self.context_menu.visible = false;
        self.start_menu_open = false;
        self.start_menu_search.clear();
        
        // Check if right-click inside a File Manager window's content area
        if let Some(fm_info) = self.windows.iter().find(|w| {
            w.window_type == WindowType::FileManager
            && x >= w.x && x < w.x + w.width as i32
            && y >= w.y + TITLE_BAR_HEIGHT as i32 + 36 + 1 + 24 // below column headers
            && y < w.y + w.height as i32
        }).map(|w| (w.id, w.x, w.y, w.width, w.height, w.file_path.clone(), w.selected_index, w.content.len())) {
            let (wid, wx, wy, ww, _wh, file_path_opt, sel_idx, content_len) = fm_info;
            let sidebar_w = self.fm_states.get(&wid).map(|f| if f.sidebar_collapsed { 0i32 } else { f.sidebar_width as i32 }).unwrap_or(180);
            
            // Only show context menu if click is in the file list area (right of sidebar)
            if x >= wx + sidebar_w {
                // Determine which file is under the cursor
                let content_y = wy + TITLE_BAR_HEIGHT as i32;
                let body_y = content_y + 36 + 1;
                let list_start_y = body_y + 24 + 1;
                let row_h = 26i32;
                let file_start_idx = 5usize.min(content_len);
                let file_count = if content_len > file_start_idx + 2 { content_len - file_start_idx - 2 } else { 0 };
                let rel_y = y - list_start_y;
                let scroll = self.windows.iter().find(|w| w.id == wid).map(|w| w.scroll_offset).unwrap_or(0);
                
                let click_idx = if rel_y >= 0 { Some(scroll + (rel_y / row_h) as usize) } else { None };
                let on_file = click_idx.map(|i| i < file_count).unwrap_or(false);
                
                // Select the clicked file
                if let Some(idx) = click_idx {
                    if idx < file_count {
                        if let Some(w) = self.windows.iter_mut().find(|w| w.id == wid) {
                            w.selected_index = idx;
                        }
                    }
                }
                
                // Get target file name
                let target_file = if on_file {
                    if let Some(w) = self.windows.iter().find(|w| w.id == wid) {
                        let actual_idx = file_start_idx + click_idx.unwrap_or(0);
                        if actual_idx < w.content.len().saturating_sub(2) {
                            let line = &w.content[actual_idx];
                            let name = Self::extract_name_from_entry(line);
                            if name != ".." { Some(String::from(name)) } else { None }
                        } else { None }
                    } else { None }
                } else { None };
                
                if on_file && target_file.is_some() {
                    // Right-click on a file/folder
                    self.context_menu = ContextMenu {
                        visible: true,
                        x, y,
                        items: alloc::vec![
                            ContextMenuItem { label: String::from("  Open          Enter"), action: ContextAction::Open },
                            ContextMenuItem { label: String::from("  Open With..."), action: ContextAction::OpenWith },
                            ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                            ContextMenuItem { label: String::from("  Cut          Ctrl+X"), action: ContextAction::Cut },
                            ContextMenuItem { label: String::from("  Copy         Ctrl+C"), action: ContextAction::Copy },
                            ContextMenuItem { label: String::from("  Copy Path"), action: ContextAction::CopyPath },
                            ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                            ContextMenuItem { label: String::from("  Rename            F2"), action: ContextAction::Rename },
                            ContextMenuItem { label: String::from("  Delete           Del"), action: ContextAction::Delete },
                            ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                            ContextMenuItem { label: String::from("  Properties"), action: ContextAction::Properties },
                        ],
                        selected_index: 0,
                        target_icon: None,
                        target_file,
                    };
                } else {
                    // Right-click on empty area inside FM
                    self.context_menu = ContextMenu {
                        visible: true,
                        x, y,
                        items: alloc::vec![
                            ContextMenuItem { label: String::from("  New File         N"), action: ContextAction::NewFile },
                            ContextMenuItem { label: String::from("  New Folder       D"), action: ContextAction::NewFolder },
                            ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                            ContextMenuItem { label: String::from("  Paste        Ctrl+V"), action: ContextAction::Paste },
                            ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                            ContextMenuItem { label: String::from("  Sort by Name"), action: ContextAction::SortByName },
                            ContextMenuItem { label: String::from("  Sort by Size"), action: ContextAction::SortBySize },
                            ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                            ContextMenuItem { label: String::from("  Refresh          F5"), action: ContextAction::Refresh },
                            ContextMenuItem { label: String::from("  Open in Terminal"), action: ContextAction::TerminalHere },
                            ContextMenuItem { label: String::from("  Properties"), action: ContextAction::Properties },
                        ],
                        selected_index: 0,
                        target_icon: None,
                        target_file: file_path_opt,
                    };
                }
                return;
            }
        }
        
        // Check if right-click on desktop icon
        if let Some(idx) = self.check_icon_index(x, y) {
            self.show_icon_context_menu(x, y, idx);
            return;
        }
        
        // Check if right-click on desktop (empty area)
        if y < (self.height - TASKBAR_HEIGHT) as i32 {
            self.show_desktop_context_menu(x, y);
        }
    }
    
    /// Show context menu for a desktop icon
    fn show_icon_context_menu(&mut self, x: i32, y: i32, icon_index: usize) {
        self.context_menu = ContextMenu {
            visible: true,
            x,
            y,
            items: alloc::vec![
                ContextMenuItem { label: String::from("  Open          Enter"), action: ContextAction::Open },
                ContextMenuItem { label: String::from("  Open With..."), action: ContextAction::OpenWith },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Cut          Ctrl+X"), action: ContextAction::Cut },
                ContextMenuItem { label: String::from("  Copy         Ctrl+C"), action: ContextAction::Copy },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Rename            F2"), action: ContextAction::Rename },
                ContextMenuItem { label: String::from("  Delete           Del"), action: ContextAction::Delete },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Properties"), action: ContextAction::Properties },
            ],
            selected_index: 0,
            target_icon: Some(icon_index),
            target_file: None,
        };
    }
    
    /// Show context menu for desktop (empty area) - Windows style
    fn show_desktop_context_menu(&mut self, x: i32, y: i32) {
        self.context_menu = ContextMenu {
            visible: true,
            x,
            y,
            items: alloc::vec![
                ContextMenuItem { label: String::from("  View              >"), action: ContextAction::ViewLargeIcons },
                ContextMenuItem { label: String::from("  Sort by           >"), action: ContextAction::SortByName },
                ContextMenuItem { label: String::from("  Refresh          F5"), action: ContextAction::Refresh },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Paste        Ctrl+V"), action: ContextAction::Paste },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  New               >"), action: ContextAction::NewFile },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Open in Terminal"), action: ContextAction::TerminalHere },
                ContextMenuItem { label: String::from("  Personalize"), action: ContextAction::Personalize },
                ContextMenuItem { label: String::from("  Properties"), action: ContextAction::Properties },
            ],
            selected_index: 0,
            target_icon: None,
            target_file: None,
        };
    }
    
    /// Check if click is on context menu and return action
    fn check_context_menu_click(&self, x: i32, y: i32) -> Option<ContextAction> {
        if !self.context_menu.visible {
            return None;
        }
        
        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let menu_width = 150;
        let item_height = 22;
        let menu_height = self.context_menu.items.len() as i32 * item_height;
        
        if x >= menu_x && x < menu_x + menu_width && y >= menu_y && y < menu_y + menu_height {
            let idx = ((y - menu_y) / item_height) as usize;
            if idx < self.context_menu.items.len() {
                return Some(self.context_menu.items[idx].action);
            }
        }
        
        None
    }
    
    /// Execute context menu action
    fn execute_context_action(&mut self, action: ContextAction) {
        let offset = (self.windows.len() as i32 * 25) % 200;
        
        // Check if this context action targets a File Manager file (not a desktop icon)
        let fm_target = self.context_menu.target_file.clone();
        let fm_icon_target = self.context_menu.target_icon;
        
        // If target_file is set and target_icon is None, this came from File Manager
        let is_fm_ctx = fm_target.is_some() && fm_icon_target.is_none();
        
        match action {
            ContextAction::Open => {
                if is_fm_ctx {
                    // Open from file manager — open the selected file in the focused FM
                    if let Some(window) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager) {
                        let file_start_idx = 5usize.min(window.content.len());
                        let actual_idx = file_start_idx + window.selected_index;
                        if actual_idx < window.content.len().saturating_sub(2) {
                            let line = &window.content[actual_idx];
                            let is_dir = line.contains("[D]");
                            let name = String::from(Self::extract_name_from_entry(line));
                            if is_dir {
                                self.navigate_file_manager(&name);
                            } else {
                                self.open_file(&name);
                            }
                        }
                    }
                } else if let Some(idx) = fm_icon_target {
                    let icon_action = self.icons[idx].action;
                    self.handle_icon_action(icon_action);
                }
            },
            ContextAction::OpenWith => {
                self.create_window("Open With", 300 + offset, 200 + offset, 400, 300, WindowType::FileAssociations);
            },
            ContextAction::Refresh => {
                if is_fm_ctx {
                    if let Some(path) = fm_target {
                        self.refresh_file_manager(&path);
                    }
                }
                crate::serial_println!("[GUI] Refreshed");
            },
            ContextAction::NewFile => {
                if is_fm_ctx {
                    let current_path = self.windows.iter()
                        .find(|w| w.focused && w.window_type == WindowType::FileManager)
                        .and_then(|w| w.file_path.clone())
                        .unwrap_or_else(|| String::from("/"));
                    let name = format!("new_file_{}.txt", self.frame_count % 1000);
                    let full_path = if current_path == "/" { format!("/{}", name) } else { format!("{}/{}", current_path, name) };
                    let _ = crate::ramfs::with_fs(|fs| fs.touch(&full_path));
                    crate::serial_println!("[FM] Created file: {}", full_path);
                    self.refresh_file_manager(&current_path);
                } else {
                    let filename = format!("/desktop/newfile_{}.txt", self.frame_count);
                    crate::ramfs::with_fs(|fs| { let _ = fs.write_file(&filename, b"New file created from desktop"); });
                }
            },
            ContextAction::NewFolder => {
                if is_fm_ctx {
                    let current_path = self.windows.iter()
                        .find(|w| w.focused && w.window_type == WindowType::FileManager)
                        .and_then(|w| w.file_path.clone())
                        .unwrap_or_else(|| String::from("/"));
                    let name = format!("folder_{}", self.frame_count % 1000);
                    let full_path = if current_path == "/" { format!("/{}", name) } else { format!("{}/{}", current_path, name) };
                    let _ = crate::ramfs::with_fs(|fs| fs.mkdir(&full_path));
                    crate::serial_println!("[FM] Created folder: {}", full_path);
                    self.refresh_file_manager(&current_path);
                } else {
                    let dirname = format!("/desktop/folder_{}", self.frame_count);
                    crate::ramfs::with_fs(|fs| { let _ = fs.mkdir(&dirname); });
                }
            },
            ContextAction::Properties => {
                let (w, h) = (self.width, self.height);
                let win_count = self.windows.len();
                let icon_count = self.icons.len();
                let win_id = self.create_window("Properties", 350 + offset, 250 + offset, 320, 220, WindowType::About);
                if let Some(window) = self.windows.iter_mut().find(|wnd| wnd.id == win_id) {
                    window.content.clear();
                    window.content.push(String::from("═══════ System Properties ═══════"));
                    window.content.push(String::new());
                    window.content.push(format!("Display: {}x{}", w, h));
                    window.content.push(format!("Windows open: {}", win_count + 1));
                    window.content.push(format!("Desktop icons: {}", icon_count));
                    window.content.push(String::new());
                    window.content.push(String::from("Theme: GitHub Dark"));
                    window.content.push(String::from("OS: TrustOS v0.9.4"));
                }
            },
            ContextAction::Cut => {
                if is_fm_ctx {
                    self.file_clipboard_copy(true);
                } else if let Some(idx) = fm_icon_target {
                    self.clipboard_icon = Some((idx, true));
                    let name = self.icons[idx].name.clone();
                    crate::keyboard::clipboard_set(&name);
                }
            },
            ContextAction::Copy => {
                if is_fm_ctx {
                    self.file_clipboard_copy(false);
                } else if let Some(idx) = fm_icon_target {
                    self.clipboard_icon = Some((idx, false));
                    let name = self.icons[idx].name.clone();
                    crate::keyboard::clipboard_set(&name);
                }
            },
            ContextAction::Paste => {
                if is_fm_ctx {
                    self.file_clipboard_paste();
                } else if let Some((src_idx, is_cut)) = self.clipboard_icon.take() {
                    if src_idx < self.icons.len() {
                        if !is_cut {
                            let src = self.icons[src_idx].clone();
                            let new_name = format!("{} (copy)", src.name);
                            let new_icon = DesktopIcon {
                                name: new_name.clone(),
                                icon_type: src.icon_type,
                                x: src.x + 10,
                                y: src.y + 10,
                                action: src.action,
                            };
                            self.icons.push(new_icon);
                        }
                    }
                }
            },
            ContextAction::CopyPath => {
                if is_fm_ctx {
                    if let Some(window) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager) {
                        let current_path = window.file_path.clone().unwrap_or_else(|| String::from("/"));
                        let file_start_idx = 5usize.min(window.content.len());
                        let actual_idx = file_start_idx + window.selected_index;
                        if actual_idx < window.content.len().saturating_sub(2) {
                            let name = Self::extract_name_from_entry(&window.content[actual_idx]);
                            let full = if current_path == "/" { format!("/{}", name) } else { format!("{}/{}", current_path, name) };
                            crate::keyboard::clipboard_set(&full);
                            crate::serial_println!("[FM] Copied path: {}", full);
                        }
                    }
                } else if let Some(idx) = fm_icon_target {
                    if idx < self.icons.len() {
                        let path = format!("/desktop/{}", self.icons[idx].name);
                        crate::keyboard::clipboard_set(&path);
                    }
                }
            },
            ContextAction::Delete => {
                if is_fm_ctx {
                    if let Some(window) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager) {
                        let current_path = window.file_path.clone().unwrap_or_else(|| String::from("/"));
                        let file_start_idx = 5usize.min(window.content.len());
                        let actual_idx = file_start_idx + window.selected_index;
                        if actual_idx < window.content.len().saturating_sub(2) {
                            let name = String::from(Self::extract_name_from_entry(&window.content[actual_idx]));
                            if name != ".." {
                                let full_path = if current_path == "/" { format!("/{}", name) } else { format!("{}/{}", current_path, name) };
                                let _ = crate::ramfs::with_fs(|fs| fs.rm(&full_path));
                                crate::serial_println!("[FM] Deleted: {}", full_path);
                            }
                        }
                        let cp = current_path.clone();
                        drop(window);
                        self.refresh_file_manager(&cp);
                    }
                } else if let Some(idx) = fm_icon_target {
                    if idx < self.icons.len() {
                        self.icons.remove(idx);
                        self.clipboard_icon = None;
                    }
                }
            },
            ContextAction::Rename => {
                if is_fm_ctx {
                    // Enter rename mode
                    if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileManager) {
                        let file_start_idx = 5usize.min(window.content.len());
                        let actual_idx = file_start_idx + window.selected_index;
                        if actual_idx < window.content.len().saturating_sub(2) {
                            let name = String::from(Self::extract_name_from_entry(&window.content[actual_idx]));
                            if name != ".." {
                                self.input_buffer = name.clone();
                                window.title = format!("RENAME:{}", name);
                            }
                        }
                    }
                } else if let Some(idx) = fm_icon_target {
                    if idx < self.icons.len() {
                        crate::serial_println!("[GUI] Rename icon: {}", self.icons[idx].name);
                    }
                }
            },
            ContextAction::SortByName => {
                if is_fm_ctx {
                    if let Some(wid) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager).map(|w| w.id) {
                        if let Some(fm) = self.fm_states.get_mut(&wid) {
                            if fm.sort_column == 0 { fm.sort_ascending = !fm.sort_ascending; } else { fm.sort_column = 0; fm.sort_ascending = true; }
                        }
                        if let Some(path) = self.windows.iter().find(|w| w.id == wid).and_then(|w| w.file_path.clone()) {
                            self.refresh_file_manager(&path);
                        }
                    }
                }
            },
            ContextAction::SortBySize => {
                if is_fm_ctx {
                    if let Some(wid) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager).map(|w| w.id) {
                        if let Some(fm) = self.fm_states.get_mut(&wid) {
                            if fm.sort_column == 2 { fm.sort_ascending = !fm.sort_ascending; } else { fm.sort_column = 2; fm.sort_ascending = true; }
                        }
                        if let Some(path) = self.windows.iter().find(|w| w.id == wid).and_then(|w| w.file_path.clone()) {
                            self.refresh_file_manager(&path);
                        }
                    }
                }
            },
            ContextAction::SortByDate => {
                crate::serial_println!("[GUI] Sort by date (not yet supported)");
            },
            ContextAction::ViewLargeIcons | ContextAction::ViewSmallIcons | ContextAction::ViewList => {
                crate::serial_println!("[GUI] View mode changed");
            },
            ContextAction::Personalize => {
                self.create_window("Personalization", 250 + offset, 150 + offset, 400, 300, WindowType::Settings);
            },
            ContextAction::TerminalHere => {
                self.create_window("Terminal", 200 + offset, 120 + offset, 500, 350, WindowType::Terminal);
            },
            ContextAction::Cancel => {},
        }
    }
    
    /// Get icon index at position — uses same dynamic layout as draw_desktop_icons
    fn check_icon_index(&self, x: i32, y: i32) -> Option<usize> {
        // Must be within dock strip
        if x < 0 || x >= (DOCK_WIDTH + 10) as i32 {
            return None;
        }
        let dock_h = self.height.saturating_sub(TASKBAR_HEIGHT);
        let n_icons = self.icons.len().max(1) as u32;
        let padding = 12u32;
        let available = dock_h.saturating_sub(padding * 2);
        let icon_total = available / n_icons;
        let start_y = padding + (available - icon_total * n_icons) / 2;
        
        for (idx, _icon) in self.icons.iter().enumerate() {
            let iy = (start_y + idx as u32 * icon_total) as i32;
            if y >= iy && y < iy + icon_total as i32 {
                return Some(idx);
            }
        }
        None
    }
    
    /// Handle desktop icon action
    fn handle_icon_action(&mut self, action: IconAction) {
        let offset = (self.windows.len() as i32 * 25) % 200;
        let id = match action {
            IconAction::OpenTerminal => {
                self.create_window("Terminal", 120 + offset, 60 + offset, 640, 440, WindowType::Terminal)
            },
            IconAction::OpenFileManager => {
                self.create_window("Files", 140 + offset, 80 + offset, 520, 420, WindowType::FileManager)
            },
            IconAction::OpenCalculator => {
                self.create_window("Calculator", 350 + offset, 100 + offset, 300, 380, WindowType::Calculator)
            },
            IconAction::OpenNetwork => {
                self.create_window("NetScan", 140 + offset, 80 + offset, 640, 440, WindowType::NetworkInfo)
            },
            IconAction::OpenSettings => {
                self.create_window("Settings", 250 + offset, 120 + offset, 440, 340, WindowType::Settings)
            },
            IconAction::OpenAbout => {
                self.create_window("About TrustOS", 300 + offset, 140 + offset, 420, 280, WindowType::About)
            },
            IconAction::OpenMusicPlayer => {
                let mp_x = self.width.saturating_sub(340) as i32;
                let mp_y = self.height.saturating_sub(TASKBAR_HEIGHT + 600) as i32;
                self.create_window("Music Player", mp_x, mp_y.max(20), 320, 580, WindowType::MusicPlayer)
            },
            IconAction::OpenGame => {
                let sw = self.width;
                let sh = self.height;
                let id = self.create_window("TrustChess 3D", 0, 0, sw, sh - TASKBAR_HEIGHT, WindowType::Chess3D);
                if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                    w.maximized = true;
                }
                id
            },
            IconAction::OpenEditor => {
                self.create_window("TrustCode", 120 + offset, 50 + offset, 780, 560, WindowType::TextEditor)
            },
            IconAction::OpenGL3D => {
                self.create_window("TrustGL 3D Demo", 120 + offset, 50 + offset, 500, 420, WindowType::Demo3D)
            },
            IconAction::OpenBrowser => {
                self.create_window("TrustBrowser", 100 + offset, 40 + offset, 720, 520, WindowType::Browser)
            },
            IconAction::OpenModelEditor => {
                self.create_window("TrustEdit 3D", 80 + offset, 40 + offset, 780, 560, WindowType::ModelEditor)
            },
            IconAction::OpenGame3D => {
                self.create_window("TrustDoom 3D", 60 + offset, 30 + offset, 720, 540, WindowType::Game3D)
            },
            #[cfg(feature = "emulators")]
            IconAction::OpenNes => {
                self.create_window("NES Emulator", 80 + offset, 50 + offset, 512, 480, WindowType::NesEmu)
            },
            #[cfg(feature = "emulators")]
            IconAction::OpenGameBoy => {
                self.create_window("Game Boy", 100 + offset, 60 + offset, 480, 432, WindowType::GameBoyEmu)
            },
            #[cfg(feature = "emulators")]
            IconAction::OpenGameLab => {
                let sw = self.width;
                let sh = self.height;
                // Open on the right side, leaving room for Game Boy (480px)
                let lab_x = 490i32;
                let lab_w = (sw as i32 - lab_x).max(400) as u32;
                let lab_h = sh - TASKBAR_HEIGHT;
                let lab_id = self.create_window("Game Lab", lab_x, 0, lab_w, lab_h, WindowType::GameLab);
                lab_id
            },
        };
        // Auto-focus newly created window
        self.focus_window(id);
    }
    
    fn handle_taskbar_click(&mut self, x: i32, _y: i32) {
        // Show Desktop button (far right corner, 8px wide)
        if x >= (self.width - 8) as i32 {
            self.toggle_show_desktop();
            crate::serial_println!("[GUI] Show Desktop corner clicked");
            return;
        }
        
        // TrustOS button (left side, matches draw_taskbar v2)
        if x >= 4 && x < 120 {
            self.start_menu_open = !self.start_menu_open;
            if !self.start_menu_open {
                self.start_menu_search.clear();
            }
            return;
        }
        
        // Settings button in system tray (gear icon)
        let tray_x = self.width - 120;
        let settings_x = tray_x - 44;
        if x >= settings_x as i32 && x < (settings_x + 40) as i32 {
            self.open_settings_panel();
            return;
        }
        
        // Window buttons — must match the centered layout in draw_taskbar v2
        let total_btns = self.windows.len();
        if total_btns > 0 {
            let btn_w = 96u32;
            let btn_gap = 6u32;
            let total_w = total_btns as u32 * (btn_w + btn_gap) - btn_gap;
            let start_x = (self.width.saturating_sub(total_w)) / 2;
            
            for (i, w) in self.windows.iter().enumerate() {
                let btn_x = start_x + i as u32 * (btn_w + btn_gap);
                if x >= btn_x as i32 && x < (btn_x + btn_w) as i32 {
                    let id = w.id;
                    // Click on focused window → minimize; click on other → focus/unminimize
                    if w.focused && !w.minimized {
                        self.minimize_window(id);
                    } else {
                        self.focus_window(id);
                    }
                    return;
                }
            }
        }
    }
    
    /// Open or focus the settings panel
    fn open_settings_panel(&mut self) {
        // Check if settings window already exists
        for w in &self.windows {
            if w.window_type == WindowType::Settings {
                let id = w.id;
                self.focus_window(id);
                return;
            }
        }
        // Create new settings window (wider for sidebar layout)
        self.create_window("Settings", 180, 80, 620, 440, WindowType::Settings);
    }
    
    /// Menu actions enum — must match draw_start_menu layout exactly
    fn check_start_menu_click(&self, x: i32, y: i32) -> Option<u8> {
        // Same dimensions as draw_start_menu() v2
        let menu_w = 480u32;
        let menu_h = 680u32;
        let menu_x = 4i32;
        let menu_y = (self.height - TASKBAR_HEIGHT - menu_h - 8) as i32;
        
        // Check if click is inside the start menu at all
        if x < menu_x || x >= menu_x + menu_w as i32 || y < menu_y || y >= menu_y + menu_h as i32 {
            return None;
        }
        
        // Search bar: menu_y + 34, height 36 → items start at menu_y + 78
        let items_start_y = menu_y + 78;
        
        // App labels (indices 0-14, non-special)
        let app_labels: [&str; 15] = [
            "Terminal", "Files", "Calculator", "Network", "Text Editor",
            "TrustEdit 3D", "Browser", "Snake", "Chess", "Chess 3D",
            "NES Emulator", "Game Boy", "TrustLab", "Music Player", "Settings",
        ];
        let app_indices: [u8; 15] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14];
        
        // Power labels (indices 15-17, bottom-anchored)
        let power_labels: [&str; 3] = ["Exit Desktop", "Shutdown", "Reboot"];
        let power_indices: [u8; 3] = [15, 16, 17];
        
        let search = self.start_menu_search.trim();
        let search_lower: String = search.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
        
        // Grid layout: 2 columns, tile_h=44, tile_gap=4
        let col_count = 2u32;
        let tile_w = (menu_w - 24) / col_count;
        let tile_h = 44u32;
        let tile_gap = 4u32;
        
        let filtered_apps: alloc::vec::Vec<u8> = if search.is_empty() {
            app_indices.to_vec()
        } else {
            app_indices.iter().filter(|&&idx| {
                let label: String = app_labels[idx as usize].chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                label.contains(search_lower.as_str())
            }).copied().collect()
        };
        
        // Check app grid items
        if y >= items_start_y && y < menu_y + menu_h as i32 - 110 {
            for (drawn, &app_idx) in filtered_apps.iter().enumerate() {
                let col = (drawn % col_count as usize) as i32;
                let row = (drawn / col_count as usize) as i32;
                let item_x = menu_x + 10 + col * (tile_w + tile_gap) as i32;
                let item_y = items_start_y + row * (tile_h + tile_gap) as i32;
                
                if x >= item_x && x < item_x + tile_w as i32
                    && y >= item_y && y < item_y + tile_h as i32 {
                    return Some(app_idx);
                }
            }
        }
        
        // Check power items (bottom-anchored)
        let power_y = menu_y + menu_h as i32 - 106;
        let power_start_y = power_y + 8;
        if y >= power_start_y {
            for (pi, &pidx) in power_indices.iter().enumerate() {
                if !search_lower.is_empty() {
                    let ll: String = power_labels[pi].chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                    if !ll.contains(search_lower.as_str()) { continue; }
                }
                let item_top = power_start_y + (pi as i32 * 30);
                if y >= item_top && y < item_top + 28 {
                    return Some(pidx);
                }
            }
        }
        
        None
    }
    
    fn handle_menu_action(&mut self, action: u8) {
        // Matches draw_start_menu items array order:
        // 0=Terminal, 1=Files, 2=Calculator, 3=Network, 4=TextEditor,
        // 5=TrustEdit3D, 6=Browser, 7=Chess3D, 8=Chess2D, 9=Snake, 10=NES, 11=GameBoy, 12=TrustLab, 13=MusicPlayer, 14=Settings, 15=Exit Desktop, 16=Shutdown, 17=Reboot
        match action {
            0 => { // Terminal
                let x = 100 + (self.windows.len() as i32 * 30);
                let y = 60 + (self.windows.len() as i32 * 20);
                self.create_window("Terminal", x, y, 640, 440, WindowType::Terminal);
            },
            1 => { // Files
                self.create_window("File Explorer", 100, 60, 780, 520, WindowType::FileManager);
            },
            2 => { // Calculator
                self.create_window("Calculator", 350, 100, 300, 380, WindowType::Calculator);
            },
            3 => { // NetScan
                self.create_window("NetScan", 140, 80, 640, 440, WindowType::NetworkInfo);
            },
            4 => { // Text Editor (TrustCode)
                self.create_window("TrustCode", 120, 50, 780, 560, WindowType::TextEditor);
            },
            5 => { // TrustEdit 3D
                self.create_window("TrustEdit 3D", 80, 40, 780, 560, WindowType::ModelEditor);
            },
            6 => { // Browser
                self.create_window("TrustBrowser", 100, 40, 720, 520, WindowType::Browser);
            },
            7 => { // Chess 3D — open fullscreen
                let sw = self.width;
                let sh = self.height;
                let id = self.create_window("TrustChess 3D", 0, 0, sw, sh - TASKBAR_HEIGHT, WindowType::Chess3D);
                // Mark as maximized
                if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                    w.maximized = true;
                }
            },
            8 => { // Chess 2D
                self.create_window("TrustChess", 180, 60, 520, 560, WindowType::Chess);
            },
            9 => { // Snake
                self.create_window("Snake Game", 220, 80, 380, 400, WindowType::Game);
            },
            10 => { // NES Emulator
                #[cfg(feature = "emulators")]
                self.create_window("NES Emulator", 80, 40, 560, 520, WindowType::NesEmu);
            },
            11 => { // Game Boy
                #[cfg(feature = "emulators")]
                self.create_window("Game Boy", 100, 40, 520, 480, WindowType::GameBoyEmu);
            },
            12 => { // TrustLab
                self.open_lab_mode();
            },
            13 => { // Music Player
                crate::serial_println!("[GUI] Opening Music Player...");
                let mp_x = self.width.saturating_sub(320) as i32;
                let mp_y = self.height.saturating_sub(TASKBAR_HEIGHT + 600) as i32;
                crate::serial_println!("[GUI] Music Player pos: {}x{}", mp_x, mp_y.max(20));
                self.create_window("Music Player", mp_x, mp_y.max(20), 320, 580, WindowType::MusicPlayer);
                crate::serial_println!("[GUI] Music Player window created OK");
            },
            14 => { // Settings
                self.open_settings_panel();
            },
            15 => { // Exit Desktop
                crate::serial_println!("[GUI] Exit Desktop from start menu");
                EXIT_DESKTOP_FLAG.store(true, Ordering::SeqCst);
            },
            16 => { // Shutdown
                crate::println!("\n\n=== SYSTEM SHUTDOWN ===");
                crate::println!("Goodbye!");
                loop { crate::arch::halt(); }
            },
            17 => { // Reboot
                crate::serial_println!("[SYSTEM] Reboot requested");
                // Triple fault reboot
                unsafe {
                    let mut port = crate::arch::Port::<u8>::new(0x64);
                    port.write(0xFE);
                }
                loop { crate::arch::halt(); }
            },
            _ => {}
        }
    }
    
    /// Handle keyboard input for the focused window
    pub fn handle_keyboard_input(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN};
        crate::serial_println!("[KBD-DBG] handle_keyboard_input key={} (0x{:02X}) lock={} start_menu={}",
            key, key, self.lock_screen_active, self.start_menu_open);
        
        // If lock screen is active, route all keys there
        if self.lock_screen_active {
            self.handle_lock_screen_key(key);
            return;
        }
        
        // If start menu is open, route keyboard to search bar + navigation
        if self.start_menu_open {
            match key {
                0x1B => { // Escape — close menu
                    self.start_menu_open = false;
                    self.start_menu_search.clear();
                    self.start_menu_selected = -1;
                },
                0x08 | 0x7F => { // Backspace / Delete
                    self.start_menu_search.pop();
                    self.start_menu_selected = -1; // Reset selection on search change
                },
                k if k == KEY_UP => { // Arrow Up — navigate menu items
                    if self.start_menu_selected > 0 {
                        self.start_menu_selected -= 1;
                    } else {
                        // Wrap to last item (total including power items = 17)
                        self.start_menu_selected = 16;
                    }
                },
                k if k == KEY_DOWN => { // Arrow Down — navigate menu items
                    if self.start_menu_selected < 16 {
                        self.start_menu_selected += 1;
                    } else {
                        self.start_menu_selected = 0;
                    }
                },
                0x0D | 0x0A => { // Enter — launch selected or first match
                    if self.start_menu_selected >= 0 && self.start_menu_selected <= 16 {
                        // Launch the selected item
                        let action = self.start_menu_selected as u8;
                        self.start_menu_open = false;
                        self.start_menu_search.clear();
                        self.start_menu_selected = -1;
                        self.handle_menu_action(action);
                        return;
                    }
                    // Fallback: launch first matching item by search
                    let all_labels: [&str; 17] = [
                        "Terminal", "Files", "Calculator", "Network", "Text Editor",
                        "TrustEdit 3D", "Browser", "Snake", "Chess", "Chess 3D",
                        "NES Emulator", "Game Boy", "TrustLab",
                        "Settings", "Exit Desktop", "Shutdown", "Reboot",
                    ];
                    let search = self.start_menu_search.trim();
                    if !search.is_empty() {
                        let search_lower: String = search.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                        for (i, label) in all_labels.iter().enumerate() {
                            let label_lower: String = label.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                            if label_lower.contains(search_lower.as_str()) {
                                self.start_menu_open = false;
                                self.start_menu_search.clear();
                                self.start_menu_selected = -1;
                                self.handle_menu_action(i as u8);
                                return;
                            }
                        }
                    }
                },
                b' '..=b'~' => { // Printable ASCII
                    if self.start_menu_search.len() < 32 {
                        self.start_menu_search.push(key as char);
                        self.start_menu_selected = -1; // Reset selection on search change
                    }
                },
                _ => {}
            }
            return;
        }

        // Extract type and id to avoid borrow conflict
        let focused_info = self.windows.iter().find(|w| w.focused).map(|w| (w.window_type, w.id));
        crate::serial_println!("[KBD-DBG] focused_info={:?} n_windows={}",
            focused_info.map(|(_, id)| id), self.windows.len());
        
        if let Some((wtype, win_id)) = focused_info {
            match wtype {
                WindowType::Terminal => {
                    self.handle_terminal_key(key);
                },
                WindowType::FileManager => {
                    // Ctrl+C/X/V for file clipboard
                    let ctrl = crate::keyboard::is_key_pressed(0x1D);
                    if ctrl && (key == 3 || key == b'c' || key == b'C') {
                        self.file_clipboard_copy(false);
                        return;
                    }
                    if ctrl && (key == 24 || key == b'x' || key == b'X') {
                        self.file_clipboard_copy(true);
                        return;
                    }
                    if ctrl && (key == 22 || key == b'v' || key == b'V') {
                        self.file_clipboard_paste();
                        return;
                    }
                    // V to toggle view mode (cycle: List -> Grid -> Details -> Tiles)
                    if key == b'v' || key == b'V' {
                        let current = self.fm_view_modes.get(&win_id).copied().unwrap_or(FileManagerViewMode::List);
                        let new_mode = match current {
                            FileManagerViewMode::List => FileManagerViewMode::IconGrid,
                            FileManagerViewMode::IconGrid => FileManagerViewMode::Details,
                            FileManagerViewMode::Details => FileManagerViewMode::Tiles,
                            FileManagerViewMode::Tiles => FileManagerViewMode::List,
                        };
                        self.fm_view_modes.insert(win_id, new_mode);
                        crate::serial_println!("[FM] View mode: {:?}-like for window {}", 
                            match new_mode { FileManagerViewMode::List => "List", FileManagerViewMode::IconGrid => "Grid", FileManagerViewMode::Details => "Details", FileManagerViewMode::Tiles => "Tiles" },
                            win_id);
                        return;
                    }
                    self.handle_filemanager_key(key);
                },
                WindowType::ImageViewer => {
                    self.handle_image_viewer_key(key);
                },
                WindowType::FileAssociations => {
                    self.handle_fileassoc_key(key);
                },
                WindowType::Settings => {
                    self.handle_settings_key(key);
                },
                WindowType::NetworkInfo => {
                    self.handle_netscan_key(key);
                },
                WindowType::TextEditor => {
                    // Forward to TrustCode editor state
                    if let Some(editor) = self.editor_states.get_mut(&win_id) {
                        editor.handle_key(key);
                    }
                },
                WindowType::ModelEditor => {
                    // Forward to model editor state
                    if let Some(state) = self.model_editor_states.get_mut(&win_id) {
                        state.handle_key(key);
                    }
                },
                WindowType::Calculator => {
                    if let Some(calc) = self.calculator_states.get_mut(&win_id) {
                        match key {
                            b'0'..=b'9' => calc.press_digit(key as char),
                            b'.' => calc.press_dot(),
                            b'+' => calc.press_operator('+'),
                            b'-' => calc.press_operator('-'),
                            b'*' => calc.press_operator('*'),
                            b'/' => calc.press_operator('/'),
                            b'%' => calc.press_operator('%'),
                            b'(' => calc.press_paren('('),
                            b')' => calc.press_paren(')'),
                            b'=' | 0x0D | 0x0A => calc.press_equals(), // = or Enter
                            b'c' | b'C' => calc.press_clear(),
                            0x08 => calc.press_backspace(), // Backspace
                            0x7F => calc.press_backspace(), // Delete
                            b's' => calc.press_func("sqrt"), // s for sqrt
                            _ => {}
                        }
                    }
                },
                WindowType::Game => {
                    if let Some(snake) = self.snake_states.get_mut(&win_id) {
                        snake.handle_key(key);
                    }
                },
                WindowType::Game3D => {
                    if let Some(game) = self.game3d_states.get_mut(&win_id) {
                        game.handle_key(key);
                    }
                },
                WindowType::Chess => {
                    if let Some(chess) = self.chess_states.get_mut(&win_id) {
                        chess.handle_key(key);
                    }
                },
                WindowType::Chess3D => {
                    if let Some(state) = self.chess3d_states.get_mut(&win_id) {
                        state.handle_key(key);
                    }
                },
                #[cfg(feature = "emulators")]
                WindowType::NesEmu => {
                    if let Some(emu) = self.nes_states.get_mut(&win_id) {
                        emu.handle_key(key);
                    }
                },
                #[cfg(feature = "emulators")]
                WindowType::GameBoyEmu => {
                    if let Some(emu) = self.gameboy_states.get_mut(&win_id) {
                        emu.handle_key(key);
                    }
                },
                WindowType::BinaryViewer => {
                    if let Some(viewer) = self.binary_viewer_states.get_mut(&win_id) {
                        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT, KEY_PGUP, KEY_PGDOWN, KEY_HOME, KEY_END};
                        match key {
                            KEY_UP => viewer.handle_scancode(0x48),
                            KEY_DOWN => viewer.handle_scancode(0x50),
                            KEY_LEFT => viewer.handle_scancode(0x4B),
                            KEY_RIGHT => viewer.handle_scancode(0x4D),
                            KEY_PGUP => viewer.handle_scancode(0x49),
                            KEY_PGDOWN => viewer.handle_scancode(0x51),
                            KEY_HOME => viewer.handle_scancode(0x47),
                            KEY_END => viewer.handle_scancode(0x4F),
                            0x09 => viewer.handle_scancode(0x0F), // Tab
                            0x0D | 0x0A => viewer.handle_scancode(0x1C), // Enter
                            _ => viewer.handle_key(key as char),
                        }
                    }
                },
                WindowType::LabMode => {
                    if let Some(lab) = self.lab_states.get_mut(&win_id) {
                        // Printable ASCII → handle_char, control keys → handle_key
                        if key >= 0x20 && key < 0x7F {
                            lab.handle_char(key as char);
                        } else {
                            lab.handle_key(key);
                        }
                    }
                },
                #[cfg(feature = "emulators")]
                WindowType::GameLab => {
                    if let Some(lab) = self.gamelab_states.get_mut(&win_id) {
                        // Handle Enter for search scan/filter
                        if key == 0x0D || key == 0x0A {
                            if lab.active_tab == crate::game_lab::LabTab::Search {
                                // Find linked emulator
                                let emu_id = lab.linked_gb_id
                                    .or_else(|| self.gameboy_states.keys().next().copied());
                                if let Some(eid) = emu_id {
                                    let do_initial = !lab.search_active;
                                    if do_initial {
                                        if let Some(emu) = self.gameboy_states.get(&eid) {
                                            if let Some(gl) = self.gamelab_states.get_mut(&win_id) {
                                                gl.search_initial(emu);
                                            }
                                        }
                                    } else {
                                        if let Some(emu) = self.gameboy_states.get(&eid) {
                                            if let Some(gl) = self.gamelab_states.get_mut(&win_id) {
                                                gl.search_filter(emu);
                                            }
                                        }
                                    }
                                }
                                return;
                            }
                        }
                        lab.handle_key(key);
                    }
                },
                WindowType::Browser => {
                    use crate::keyboard::{KEY_LEFT, KEY_RIGHT, KEY_HOME, KEY_END, KEY_DELETE, KEY_PGUP, KEY_PGDOWN};
                    let ctrl = crate::keyboard::is_key_pressed(0x1D);
                    crate::serial_println!("[BROWSER] Key received: {} (0x{:02X}) cursor={} url_len={} sel={}", 
                        if key >= 0x20 && key < 0x7F { key as char } else { '?' }, key,
                        self.browser_url_cursor, self.browser_url_input.len(), self.browser_url_select_all);
                    
                    // If all text is selected, handle replacement
                    if self.browser_url_select_all {
                        match key {
                            0x08 | _ if key == KEY_DELETE => {
                                // Backspace/Delete with select-all → clear all
                                self.browser_url_input.clear();
                                self.browser_url_cursor = 0;
                                self.browser_url_select_all = false;
                                return;
                            },
                            0x1B => {
                                // Escape → just deselect
                                self.browser_url_select_all = false;
                                return;
                            },
                            0x0D | 0x0A => {
                                // Enter → navigate (deselect, fall through)
                                self.browser_url_select_all = false;
                            },
                            32..=126 => {
                                // Printable char → replace entire URL with this char
                                self.browser_url_input.clear();
                                self.browser_url_input.push(key as char);
                                self.browser_url_cursor = 1;
                                self.browser_url_select_all = false;
                                return;
                            },
                            _ => {
                                // Arrow keys etc. → just deselect, fall through
                                self.browser_url_select_all = false;
                            }
                        }
                    }
                    
                    // Ctrl+A: select all
                    if ctrl && (key == b'a' || key == b'A') {
                        self.browser_url_select_all = true;
                        self.browser_url_cursor = self.browser_url_input.len();
                        return;
                    }
                    
                    // Don't process keys while loading (except Escape to cancel)
                    if self.browser_loading && key != 0x1B {
                        crate::serial_println!("[BROWSER] Key ignored: loading in progress");
                    } else {
                    match key {
                        0x08 => { // Backspace - delete char before cursor
                            if self.browser_url_cursor > 0 {
                                self.browser_url_cursor -= 1;
                                if self.browser_url_cursor < self.browser_url_input.len() {
                                    self.browser_url_input.remove(self.browser_url_cursor);
                                }
                            }
                        },
                        0x0D | 0x0A => { // Enter - navigate (async)
                            if !self.browser_url_input.is_empty() && !self.browser_loading {
                                self.browser_loading = true;
                                let url = self.browser_url_input.clone();
                                crate::serial_println!("[DESKTOP] Browser navigate async: {}", url);
                                {
                                    let mut pending = BROWSER_PENDING_URL.lock();
                                    *pending = Some(url);
                                }
                                BROWSER_NAV_BUSY.store(true, Ordering::SeqCst);
                                crate::thread::spawn_kernel("browser-nav", browser_nav_worker, 0);
                            }
                        },
                        0x1B => { // Escape - cancel loading or clear URL
                            if self.browser_loading {
                                self.browser_loading = false;
                            } else {
                                self.browser_url_input.clear();
                                self.browser_url_cursor = 0;
                            }
                        },
                        _ if key == KEY_LEFT => {
                            if ctrl {
                                // Ctrl+Left: jump to previous word boundary
                                while self.browser_url_cursor > 0 {
                                    self.browser_url_cursor -= 1;
                                    if self.browser_url_cursor > 0 {
                                        let c = self.browser_url_input.as_bytes()[self.browser_url_cursor - 1];
                                        if c == b' ' || c == b'/' || c == b'.' || c == b':' {
                                            break;
                                        }
                                    }
                                }
                            } else if self.browser_url_cursor > 0 {
                                self.browser_url_cursor -= 1;
                            }
                        },
                        _ if key == KEY_RIGHT => {
                            if ctrl {
                                // Ctrl+Right: jump to next word boundary
                                let len = self.browser_url_input.len();
                                while self.browser_url_cursor < len {
                                    self.browser_url_cursor += 1;
                                    if self.browser_url_cursor < len {
                                        let c = self.browser_url_input.as_bytes()[self.browser_url_cursor];
                                        if c == b' ' || c == b'/' || c == b'.' || c == b':' {
                                            break;
                                        }
                                    }
                                }
                            } else if self.browser_url_cursor < self.browser_url_input.len() {
                                self.browser_url_cursor += 1;
                            }
                        },
                        _ if key == KEY_HOME => {
                            self.browser_url_cursor = 0;
                        },
                        _ if key == KEY_END => {
                            self.browser_url_cursor = self.browser_url_input.len();
                        },
                        _ if key == KEY_DELETE => {
                            if self.browser_url_cursor < self.browser_url_input.len() {
                                self.browser_url_input.remove(self.browser_url_cursor);
                            }
                        },
                        _ if key == KEY_PGUP => {
                            // Page Up - scroll browser content up
                            if let Some(ref mut browser) = self.browser {
                                browser.scroll(-200);
                            }
                        },
                        _ if key == KEY_PGDOWN => {
                            // Page Down - scroll browser content down
                            if let Some(ref mut browser) = self.browser {
                                browser.scroll(200);
                            }
                        },
                        _ if ctrl && (key == b'l' || key == b'L') => {
                            // Ctrl+L: select all URL text (focus omnibox)
                            self.browser_url_select_all = true;
                            self.browser_url_cursor = self.browser_url_input.len();
                        },
                        _ if ctrl && (key == b'r' || key == b'R') => {
                            // Ctrl+R or F5: refresh
                            if let Some(ref mut browser) = self.browser {
                                let _ = browser.refresh();
                            }
                        },
                        _ if ctrl && (key == b'a' || key == b'A') => {
                            // Ctrl+A: select all — handled above, but fallback
                            self.browser_url_select_all = true;
                            self.browser_url_cursor = self.browser_url_input.len();
                        },
                        _ if key == b'\t' => {
                            // Tab: auto-complete common domains
                            if !self.browser_url_input.contains("://") && !self.browser_url_input.is_empty() {
                                self.browser_url_input = alloc::format!("http://{}", self.browser_url_input);
                                self.browser_url_cursor = self.browser_url_input.len();
                            }
                        },
                        32..=126 => { // Printable ASCII - insert at cursor
                            if self.browser_url_input.len() < 512 {
                                if self.browser_url_cursor >= self.browser_url_input.len() {
                                    self.browser_url_input.push(key as char);
                                } else {
                                    self.browser_url_input.insert(self.browser_url_cursor, key as char);
                                }
                                self.browser_url_cursor += 1;
                            }
                        },
                        _ => {}
                    }
                    }
                },
                WindowType::HexViewer => {
                    use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_PGUP, KEY_PGDOWN, KEY_HOME, KEY_END};
                    if let Some(window) = self.windows.iter_mut().find(|w| w.id == win_id) {
                        let visible_lines = ((window.height.saturating_sub(TITLE_BAR_HEIGHT + 20)) / 16) as usize;
                        let max_scroll = window.content.len().saturating_sub(visible_lines);
                        match key {
                            KEY_UP => window.scroll_offset = window.scroll_offset.saturating_sub(1),
                            KEY_DOWN => window.scroll_offset = (window.scroll_offset + 1).min(max_scroll),
                            KEY_PGUP => window.scroll_offset = window.scroll_offset.saturating_sub(visible_lines),
                            KEY_PGDOWN => window.scroll_offset = (window.scroll_offset + visible_lines).min(max_scroll),
                            KEY_HOME => window.scroll_offset = 0,
                            KEY_END => window.scroll_offset = max_scroll,
                            _ => {}
                        }
                    }
                },
                _ => {}
            }
        }
    }
    
    /// Handle file manager keyboard input
    fn handle_filemanager_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_DELETE};
        
        // ── Check if search box is focused → route keys there ──
        {
            let focused_wid = self.windows.iter()
                .find(|w| w.focused && w.window_type == WindowType::FileManager)
                .map(|w| w.id);
            if let Some(wid) = focused_wid {
                let search_focused = self.fm_states.get(&wid).map(|f| f.search_focused).unwrap_or(false);
                if search_focused {
                    if key == 0x1B { // Escape — unfocus search
                        if let Some(fm) = self.fm_states.get_mut(&wid) {
                            fm.search_focused = false;
                            fm.search_query.clear();
                        }
                        let path = self.windows.iter().find(|w| w.id == wid)
                            .and_then(|w| w.file_path.clone()).unwrap_or_else(|| String::from("/"));
                        self.refresh_file_manager(&path);
                        return;
                    } else if key == 0x08 { // Backspace
                        if let Some(fm) = self.fm_states.get_mut(&wid) {
                            fm.search_query.pop();
                        }
                        let path = self.windows.iter().find(|w| w.id == wid)
                            .and_then(|w| w.file_path.clone()).unwrap_or_else(|| String::from("/"));
                        self.refresh_file_manager(&path);
                        return;
                    } else if key == 0x0D || key == 0x0A { // Enter — unfocus
                        if let Some(fm) = self.fm_states.get_mut(&wid) {
                            fm.search_focused = false;
                        }
                        return;
                    } else if key >= 0x20 && key < 0x7F {
                        if let Some(fm) = self.fm_states.get_mut(&wid) {
                            if fm.search_query.len() < 32 {
                                fm.search_query.push(key as char);
                            }
                        }
                        let path = self.windows.iter().find(|w| w.id == wid)
                            .and_then(|w| w.file_path.clone()).unwrap_or_else(|| String::from("/"));
                        self.refresh_file_manager(&path);
                        return;
                    }
                    return;
                }
            }
        }
        
        let mut action: Option<(String, bool)> = None; // (filename, is_dir)
        let mut delete_target: Option<String> = None;
        let mut new_file = false;
        let mut new_folder = false;
        let mut rename_start = false;
        
        let mut rename_action: Option<(String, String, String)> = None; // (old_name, new_name, current_path)
        
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileManager) {
            // Check if we're in rename mode
            if window.title.starts_with("RENAME:") {
                if key == 0x0D || key == 0x0A { // Enter — confirm rename
                    let old_name = String::from(&window.title[7..]);
                    let new_name = self.input_buffer.clone();
                    self.input_buffer.clear();
                    window.title = String::from("File Manager");
                    let current_path = window.file_path.clone().unwrap_or_else(|| String::from("/"));
                    rename_action = Some((old_name, new_name, current_path));
                } else if key == 0x08 { // Backspace in rename
                    self.input_buffer.pop();
                    return;
                } else if key == 0x1B { // Escape — cancel rename
                    self.input_buffer.clear();
                    window.title = String::from("File Manager");
                    return;
                } else if key >= 0x20 && key < 0x7F {
                    self.input_buffer.push(key as char);
                    return;
                }
                return;
            }
            
            // Get file list (skip header lines - first 5 lines)
            let file_count = window.content.len().saturating_sub(7); // header + footer
            
            if key == KEY_UP {
                if window.selected_index > 0 {
                    window.selected_index -= 1;
                }
            } else if key == KEY_DOWN {
                if window.selected_index < file_count.saturating_sub(1) {
                    window.selected_index += 1;
                }
            } else if key == 0x08 { // Backspace - navigate up
                action = Some((String::from(".."), true));
            } else if key == KEY_DELETE { // Delete selected file/folder
                let idx = window.selected_index + 5;
                if idx < window.content.len().saturating_sub(2) {
                    let line = &window.content[idx];
                    if let Some(name_start) = line.find(']') {
                        if name_start + 2 < line.len() {
                            let rest = &line[name_start + 2..];
                            if let Some(name_end) = rest.find(' ') {
                                let filename = String::from(rest[..name_end].trim());
                                if filename != ".." {
                                    delete_target = Some(filename);
                                }
                            }
                        }
                    }
                }
            } else if key == b'n' || key == b'N' { // New file
                new_file = true;
            } else if key == b'd' || key == b'D' { // New folder (D for Directory)
                new_folder = true;
            } else if key == b'r' || key == b'R' { // R for rename
                rename_start = true;
                let idx = window.selected_index + 5;
                if idx < window.content.len().saturating_sub(2) {
                    let line = &window.content[idx];
                    if let Some(name_start) = line.find(']') {
                        if name_start + 2 < line.len() {
                            let rest = &line[name_start + 2..];
                            if let Some(name_end) = rest.find(' ') {
                                let filename = String::from(rest[..name_end].trim());
                                if filename != ".." {
                                    self.input_buffer = filename.clone();
                                    window.title = format!("RENAME:{}", filename);
                                }
                            }
                        }
                    }
                }
            } else if key == 0x0D || key == 0x0A { // Enter - open file
                // Get selected file
                let idx = window.selected_index + 5; // Skip header
                if idx < window.content.len().saturating_sub(2) { // Skip footer
                    let line = &window.content[idx];
                    // Parse filename from line format: "  [icon] filename..."
                    if let Some(name_start) = line.find(']') {
                        if name_start + 2 < line.len() {
                            let rest = &line[name_start + 2..];
                            if let Some(name_end) = rest.find(' ') {
                                let filename = String::from(rest[..name_end].trim());
                                let is_dir = line.contains("[D]");
                                action = Some((filename, is_dir));
                            }
                        }
                    }
                }
            }
        }
        
        // Handle rename action
        if let Some((old_name, new_name, current_path)) = rename_action {
            let old_path = if current_path == "/" { format!("/{}", old_name) } else { format!("{}/{}", current_path, old_name) };
            let new_path_str = if current_path == "/" { format!("/{}", new_name) } else { format!("{}/{}", current_path, new_name) };
            let _ = crate::ramfs::with_fs(|fs| fs.mv(&old_path, &new_path_str));
            crate::serial_println!("[FM] Renamed: {} -> {}", old_path, new_path_str);
            self.refresh_file_manager(&current_path);
            return;
        }
        
        // Handle delete outside borrow
        if let Some(filename) = delete_target {
            let current_path = self.windows.iter()
                .find(|w| w.focused && w.window_type == WindowType::FileManager)
                .and_then(|w| w.file_path.clone())
                .unwrap_or_else(|| String::from("/"));
            let full_path = if current_path == "/" { format!("/{}", filename) } else { format!("{}/{}", current_path, filename) };
            let _ = crate::ramfs::with_fs(|fs| fs.rm(&full_path));
            crate::serial_println!("[FM] Deleted: {}", full_path);
            self.refresh_file_manager(&current_path);
            return;
        }
        
        // Handle new file
        if new_file {
            let current_path = self.windows.iter()
                .find(|w| w.focused && w.window_type == WindowType::FileManager)
                .and_then(|w| w.file_path.clone())
                .unwrap_or_else(|| String::from("/"));
            let name = format!("new_file_{}.txt", self.frame_count % 1000);
            let full_path = if current_path == "/" { format!("/{}", name) } else { format!("{}/{}", current_path, name) };
            let _ = crate::ramfs::with_fs(|fs| fs.touch(&full_path));
            crate::serial_println!("[FM] Created file: {}", full_path);
            self.refresh_file_manager(&current_path);
            return;
        }
        
        // Handle new folder
        if new_folder {
            let current_path = self.windows.iter()
                .find(|w| w.focused && w.window_type == WindowType::FileManager)
                .and_then(|w| w.file_path.clone())
                .unwrap_or_else(|| String::from("/"));
            let name = format!("folder_{}", self.frame_count % 1000);
            let full_path = if current_path == "/" { format!("/{}", name) } else { format!("{}/{}", current_path, name) };
            let _ = crate::ramfs::with_fs(|fs| fs.mkdir(&full_path));
            crate::serial_println!("[FM] Created folder: {}", full_path);
            self.refresh_file_manager(&current_path);
            return;
        }
        
        // Handle action outside the borrow
        if let Some((filename, is_dir)) = action {
            if is_dir {
                // Navigate into directory
                self.navigate_file_manager(&filename);
            } else {
                self.open_file(&filename);
            }
        }
    }
    
    /// Refresh file manager at current directory
    fn refresh_file_manager(&mut self, path: &str) {
        // Read sort/search settings before borrowing windows mutably
        let (sort_col, sort_asc, search_q) = {
            let wid = self.windows.iter()
                .find(|w| w.focused && w.window_type == WindowType::FileManager)
                .map(|w| w.id);
            if let Some(wid) = wid {
                if let Some(fm) = self.fm_states.get(&wid) {
                    (fm.sort_column, fm.sort_ascending, fm.search_query.clone())
                } else { (0, true, String::new()) }
            } else { return; }
        };
        
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileManager) {
            window.content.clear();
            window.content.push(String::from("=== File Manager ==="));
            window.content.push(format!("Path: {}", path));
            window.content.push(String::from(""));
            window.content.push(String::from("  Name              Type       Size    Program"));
            window.content.push(String::from("  ────────────────────────────────────────────"));
            
            if path != "/" {
                window.content.push(String::from("  [D] ..             DIR        ---     ---"));
            }
            
            let path_arg = if path == "/" { Some("/") } else { Some(path) };
            if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(path_arg)) {
                // Filter by search query
                let search_lower: String = search_q.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                let mut filtered: Vec<&(String, crate::ramfs::FileType, usize)> = if search_lower.is_empty() {
                    entries.iter().collect()
                } else {
                    entries.iter().filter(|(name, _, _)| {
                        let name_lower: String = name.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                        name_lower.contains(search_lower.as_str())
                    }).collect()
                };
                
                // Sort entries
                filtered.sort_by(|a, b| {
                    // Directories always first
                    let a_is_dir = a.1 == crate::ramfs::FileType::Directory;
                    let b_is_dir = b.1 == crate::ramfs::FileType::Directory;
                    if a_is_dir != b_is_dir {
                        return if a_is_dir { core::cmp::Ordering::Less } else { core::cmp::Ordering::Greater };
                    }
                    let ord = match sort_col {
                        1 => { // Sort by type (extension)
                            let ext_a = a.0.rsplit('.').next().unwrap_or("");
                            let ext_b = b.0.rsplit('.').next().unwrap_or("");
                            ext_a.cmp(ext_b)
                        }
                        2 => a.2.cmp(&b.2), // Sort by size
                        _ => a.0.cmp(&b.0), // Sort by name (default)
                    };
                    if sort_asc { ord } else { ord.reverse() }
                });
                
                for (name, ftype, size) in filtered.iter().take(200) {
                    let icon = if *ftype == crate::ramfs::FileType::Directory { 
                        "[D]" 
                    } else { 
                        crate::file_assoc::get_file_icon(name)
                    };
                    let prog = if *ftype == crate::ramfs::FileType::Directory {
                        String::from("---")
                    } else {
                        String::from(crate::file_assoc::get_program_for_file(name).name())
                    };
                    let ftype_str = if *ftype == crate::ramfs::FileType::Directory { "DIR" } else { "FILE" };
                    window.content.push(format!("  {} {:<14} {:<10} {:<7} {}", icon, name, ftype_str, size, prog));
                }
            }
            if window.content.len() <= 5 + if path != "/" { 1 } else { 0 } {
                window.content.push(String::from("  (empty directory)"));
            }
            window.content.push(String::from(""));
            window.content.push(String::from("  [Del] Delete | [N] New File | [D] New Folder | [F2] Rename"));
            
            window.file_path = Some(String::from(path));
            window.selected_index = 0;
            window.scroll_offset = 0;
        }
    }
    
    /// Navigate file manager to a directory
    fn navigate_file_manager(&mut self, dirname: &str) {
        // Build new path first
        let (new_path, wid) = {
            if let Some(window) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager) {
                let current_path = window.file_path.clone().unwrap_or_else(|| String::from("/"));
                let new_path = if dirname == ".." {
                    if current_path == "/" {
                        String::from("/")
                    } else {
                        let trimmed = current_path.trim_end_matches('/');
                        match trimmed.rfind('/') {
                            Some(0) => String::from("/"),
                            Some(pos) => String::from(&trimmed[..pos]),
                            None => String::from("/"),
                        }
                    }
                } else if current_path == "/" {
                    format!("/{}", dirname)
                } else {
                    format!("{}/{}", current_path.trim_end_matches('/'), dirname)
                };
                crate::serial_println!("[FM] Navigate: {} -> {}", current_path, new_path);
                (new_path, window.id)
            } else { return; }
        };
        
        // Clear search on navigation
        if let Some(fm) = self.fm_states.get_mut(&wid) {
            fm.search_query.clear();
            fm.search_focused = false;
        }
        
        // Set path on window
        if let Some(window) = self.windows.iter_mut().find(|w| w.id == wid) {
            window.file_path = Some(new_path.clone());
        }
        
        // Use refresh to populate with sort/filter
        self.refresh_file_manager(&new_path);
        
        // Track navigation history
        if let Some(fm) = self.fm_states.get_mut(&wid) {
            fm.push_history(&new_path);
        }
    }
    
    /// Open file with associated program
    fn open_file(&mut self, filename: &str) {
        use crate::file_assoc::{get_program_for_file, Program};
        
        let program = get_program_for_file(filename);
        let offset = (self.windows.len() as i32 * 25) % 150;
        
        match program {
            Program::TextEditor => {
                let id = self.create_window(&format!("TrustCode: {}", filename), 150 + offset, 80 + offset, 700, 500, WindowType::TextEditor);
                // Load file into editor state
                if let Some(editor) = self.editor_states.get_mut(&id) {
                    editor.load_file(filename);
                }
                crate::serial_println!("[TrustCode] Opened: {}", filename);
            },
            Program::ImageViewer => {
                let id = self.create_window(&format!("View: {}", filename), 180 + offset, 100 + offset, 500, 420, WindowType::ImageViewer);
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.file_path = Some(String::from(filename));
                    window.content.clear();
                    
                    // Try to load BMP image data
                    let file_path = format!("/{}", filename);
                    if let Ok(raw_data) = crate::ramfs::with_fs(|fs| fs.read_file(&file_path).map(|d| d.to_vec())) {
                        // Try BMP parser
                        if let Some(img) = crate::theme::bmp::load_bmp_from_bytes(&raw_data) {
                            let mut state = ImageViewerState::new();
                            state.img_width = img.width;
                            state.img_height = img.height;
                            state.pixels = img.pixels;
                            // Auto-zoom to fit window (500x420 content area ~480x360)
                            let fit_w = (480 * 100) / img.width.max(1);
                            let fit_h = (360 * 100) / img.height.max(1);
                            state.zoom = fit_w.min(fit_h).min(200);
                            self.image_viewer_states.insert(id, state);
                            crate::serial_println!("[ImageViewer] Loaded BMP: {}x{}", img.width, img.height);
                            window.content.push(format!("Image: {} ({}x{} BMP)", filename, img.width, img.height));
                        } else {
                            // Not a BMP or parse failed — show file info
                            window.content.push(format!("=== Image: {} ===", filename));
                            window.content.push(format!("Size: {} bytes", raw_data.len()));
                            if raw_data.len() >= 2 && &raw_data[0..2] == b"BM" {
                                window.content.push(String::from("BMP detected but failed to parse"));
                            } else {
                                window.content.push(String::from("Format not supported (BMP only)"));
                            }
                            self.image_viewer_states.insert(id, ImageViewerState::new());
                        }
                    } else {
                        window.content.push(String::from("Failed to read file"));
                        self.image_viewer_states.insert(id, ImageViewerState::new());
                    }
                }
            },
            Program::HexViewer => {
                let id = self.create_window(&format!("Hex: {}", filename), 160 + offset, 80 + offset, 500, 350, WindowType::HexViewer);
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.file_path = Some(String::from(filename));
                    window.content.clear();
                    window.content.push(format!("=== Hex View: {} ===", filename));
                    window.content.push(String::new());
                    window.content.push(String::from("Offset   00 01 02 03 04 05 06 07  ASCII"));
                    window.content.push(String::from("──────── ─────────────────────── ────────"));
                    // Load and show hex
                    let file_path = format!("/{}", filename);
                    if let Ok(content) = crate::ramfs::with_fs(|fs| fs.read_file(&file_path).map(|d| d.to_vec())) {
                        let total_bytes = content.len();
                        for (i, chunk) in content.chunks(8).enumerate() {
                            let offset = i * 8;
                            let hex: String = chunk.iter()
                                .map(|b| format!("{:02X} ", b))
                                .collect();
                            let ascii: String = chunk.iter()
                                .map(|&b| if b >= 0x20 && b < 0x7F { b as char } else { '.' })
                                .collect();
                            window.content.push(format!("{:08X} {:<24} {}", offset, hex, ascii));
                        }
                        window.content.push(String::new());
                        window.content.push(format!("Total: {} bytes ({} lines)", total_bytes, window.content.len() - 4));
                    }
                    window.scroll_offset = 0;
                }
            },
            Program::Terminal => {
                // Execute file
                crate::serial_println!("[EXEC] Would execute: {}", filename);
                let id = self.create_window("Execution", 200 + offset, 150 + offset, 400, 200, WindowType::Terminal);
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.content.clear();
                    window.content.push(format!("Executing: {}", filename));
                    window.content.push(String::from(""));
                    window.content.push(String::from("(ELF execution not yet integrated in GUI)"));
                }
            },
            _ => {
                // Unknown - open with hex viewer
                crate::serial_println!("[OPEN] No handler for: {}", filename);
            }
        }
    }
    
    /// Handle settings keyboard input
    fn handle_settings_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN};
        
        // Left sidebar navigation: Up/Down to switch category
        if key == KEY_UP {
            if self.settings_category > 0 {
                self.settings_category -= 1;
            }
            return;
        }
        if key == KEY_DOWN {
            if self.settings_category < 7 {
                self.settings_category += 1;
            }
            return;
        }
        
        match self.settings_category {
            0 => { // Display
                if key == b'1' {
                    toggle_animations();
                } else if key == b'2' {
                    let current = *ANIMATION_SPEED.lock();
                    let next = if current <= 0.5 { 1.0 } else if current <= 1.0 { 2.0 } else { 0.5 };
                    *ANIMATION_SPEED.lock() = next;
                }
            },
            1 => { // Sound
                if key == b'1' {
                    let vol = &mut DESKTOP.lock().sys_volume;
                    *vol = (*vol + 10).min(100);
                } else if key == b'2' {
                    let vol = &mut DESKTOP.lock().sys_volume;
                    *vol = vol.saturating_sub(10);
                }
            },
            2 => { // Taskbar
                // Taskbar options handled through theme
            },
            3 => { // Personalization
                if key == b'1' {
                    // Cycle between dark_green and windows11_dark
                    let current = crate::theme::THEME.read().name.clone();
                    let next = if current == "windows11_dark" { "dark" } else { "windows11" };
                    crate::theme::set_builtin_theme(next);
                    self.needs_full_redraw = true;
                    self.background_cached = false;
                }
            },
            4 => { // Accessibility
                if key == b'1' {
                    crate::accessibility::toggle_high_contrast();
                    self.needs_full_redraw = true;
                    self.background_cached = false;
                } else if key == b'2' {
                    crate::accessibility::cycle_font_size();
                } else if key == b'3' {
                    crate::accessibility::cycle_cursor_size();
                } else if key == b'4' {
                    crate::accessibility::toggle_sticky_keys();
                } else if key == b'5' {
                    crate::accessibility::cycle_mouse_speed();
                }
            },
            5 => { // Network
                // Read-only status
            },
            6 => { // Apps
                if key == b'3' || key == 0x0D {
                    let offset = (self.windows.len() as i32 * 20) % 100;
                    self.create_window("File Associations", 250 + offset, 130 + offset, 500, 400, WindowType::FileAssociations);
                }
            },
            7 => { // About
                // Read-only info
            },
            _ => {}
        }
    }
    
    /// Draw the Settings GUI panel with sidebar + content
    fn draw_settings_gui(&self, window: &Window) {
        let wx = window.x;
        let wy = window.y;
        let ww = window.width;
        let wh = window.height;
        
        if ww < 200 || wh < 160 { return; }
        
        let content_y = wy + TITLE_BAR_HEIGHT as i32;
        let content_h = wh.saturating_sub(TITLE_BAR_HEIGHT);
        let safe_x = wx.max(0) as u32;
        
        let is_hc = crate::accessibility::is_high_contrast();
        let bg_sidebar = if is_hc { 0xFF0A0A0A } else { 0xFF060E08 };
        let bg_content = if is_hc { 0xFF000000 } else { 0xFF0A140C };
        let green_dim = 0xFF2A6A3Au32;
        let green_accent = GREEN_PRIMARY;
        let text_label = 0xFF88AA88;
        let text_value = 0xFFBBDDBB;
        let text_dim = 0xFF446644;
        
        // ── Sidebar ──
        let sidebar_w = 140u32;
        framebuffer::fill_rect(safe_x, content_y as u32, sidebar_w, content_h, bg_sidebar);
        
        let categories = [
            ("Display",      "@"),
            ("Sound",        "~"),
            ("Taskbar",      "_"),
            ("Personal.",    "*"),
            ("Access.",      "A"),
            ("Network",      "N"),
            ("Apps",         "#"),
            ("About",        "?"),
        ];
        
        let item_h = 32i32;
        let mut sy = content_y + 8;
        for (i, (label, icon)) in categories.iter().enumerate() {
            let is_active = i as u8 == self.settings_category;
            
            if is_active {
                draw_rounded_rect(safe_x as i32 + 4, sy - 1, sidebar_w - 8, item_h as u32 - 2, 4, 0xFF0C2A14);
                framebuffer::fill_rect(safe_x + 2, (sy + 2) as u32, 3, (item_h - 6) as u32, green_accent);
            }
            
            let c = if is_active { green_accent } else { text_label };
            self.draw_text_smooth(safe_x as i32 + 14, sy + 8, icon, if is_active { green_accent } else { text_dim });
            self.draw_text_smooth(safe_x as i32 + 28, sy + 8, label, c);
            sy += item_h;
        }
        
        // Separator
        framebuffer::fill_rect(safe_x + sidebar_w - 1, content_y as u32, 1, content_h, 0xFF1A3A1A);
        
        // ── Content area ──
        let cx = safe_x + sidebar_w;
        let cw = ww.saturating_sub(sidebar_w);
        framebuffer::fill_rect(cx, content_y as u32, cw, content_h, bg_content);
        
        let px = cx as i32 + 20; // padding x
        let mut py = content_y + 16;  // current y
        let line_h = 22i32;
        
        match self.settings_category {
            0 => { // Display
                self.draw_text_smooth(px, py, "Display", green_accent);
                self.draw_text_smooth(px + 1, py, "Display", green_accent); // bold
                py += line_h + 8;
                
                self.draw_text_smooth(px, py, "Resolution", text_label);
                self.draw_text_smooth(px + 120, py, &alloc::format!("{}x{}", self.width, self.height), text_value);
                py += line_h;
                
                let theme_name = crate::theme::THEME.read().name.clone();
                self.draw_text_smooth(px, py, "Theme", text_label);
                self.draw_text_smooth(px + 120, py, &theme_name, text_value);
                py += line_h + 8;
                
                // Toggle: Animations
                let anim_on = animations_enabled();
                self.draw_settings_toggle(px, py, "[1] Animations", anim_on);
                py += line_h;
                
                // Speed
                let speed = *ANIMATION_SPEED.lock();
                self.draw_text_smooth(px, py, "[2] Anim Speed", text_label);
                self.draw_text_smooth(px + 180, py, &alloc::format!("{:.1}x", speed), text_value);
                py += line_h;
            },
            1 => { // Sound
                self.draw_text_smooth(px, py, "Sound", green_accent);
                self.draw_text_smooth(px + 1, py, "Sound", green_accent);
                py += line_h + 8;
                
                self.draw_text_smooth(px, py, "Master Volume", text_label);
                let vol = self.sys_volume;
                self.draw_settings_slider(px + 140, py, cw.saturating_sub(180) as i32, vol, 100);
                py += line_h;
                
                self.draw_text_smooth(px, py, "[1] Volume +  [2] Volume -", text_dim);
                py += line_h + 8;
                
                // Audio driver info
                self.draw_text_smooth(px, py, "Audio Device", text_label);
                py += line_h;
                let driver_name = if crate::drivers::hda::is_initialized() { "Intel HDA (active)" } else { "Not detected" };
                self.draw_text_smooth(px + 12, py, driver_name, text_dim);
            },
            2 => { // Taskbar
                self.draw_text_smooth(px, py, "Taskbar", green_accent);
                self.draw_text_smooth(px + 1, py, "Taskbar", green_accent);
                py += line_h + 8;
                
                let tb = crate::theme::taskbar();
                self.draw_text_smooth(px, py, "Position", text_label);
                let pos_str = match tb.position {
                    crate::theme::TaskbarPosition::Bottom => "Bottom",
                    crate::theme::TaskbarPosition::Top => "Top",
                    crate::theme::TaskbarPosition::Left => "Left",
                    crate::theme::TaskbarPosition::Right => "Right",
                };
                self.draw_text_smooth(px + 120, py, pos_str, text_value);
                py += line_h;
                
                self.draw_text_smooth(px, py, "Height", text_label);
                self.draw_text_smooth(px + 120, py, &alloc::format!("{}px", tb.height), text_value);
                py += line_h;
                
                self.draw_settings_toggle(px, py, "Show Clock", tb.show_clock);
                py += line_h;
                
                self.draw_settings_toggle(px, py, "Show Date", tb.show_date);
                py += line_h;
                
                self.draw_settings_toggle(px, py, "Centered Icons", tb.centered_icons);
            },
            3 => { // Personalization
                self.draw_text_smooth(px, py, "Personalization", green_accent);
                self.draw_text_smooth(px + 1, py, "Personalization", green_accent);
                py += line_h + 8;
                
                let theme_name = crate::theme::THEME.read().name.clone();
                self.draw_text_smooth(px, py, "[1] Theme", text_label);
                self.draw_text_smooth(px + 120, py, &theme_name, text_value);
                py += line_h;
                
                self.draw_text_smooth(px, py, "Available themes:", text_dim);
                py += line_h;
                let themes = ["dark_green", "windows11_dark"];
                let labels = ["TrustOS Dark", "Windows 11 Dark"];
                for (i, label) in labels.iter().enumerate() {
                    let is_current = theme_name == themes[i];
                    let c = if is_current { green_accent } else { text_label };
                    let marker = if is_current { " *" } else { "  " };
                    self.draw_text_smooth(px + 16, py, &alloc::format!("{}{}", marker, label), c);
                    py += line_h;
                }
                py += 8;
                
                let colors = crate::theme::colors();
                self.draw_text_smooth(px, py, "Accent Color", text_label);
                // Draw color swatch
                framebuffer::fill_rect((px + 120) as u32, py as u32, 20, 14, colors.accent);
                py += line_h;
                
                self.draw_text_smooth(px, py, "Background", text_label);
                framebuffer::fill_rect((px + 120) as u32, py as u32, 20, 14, colors.background);
            },
            4 => { // Accessibility
                self.draw_text_smooth(px, py, "Accessibility", green_accent);
                self.draw_text_smooth(px + 1, py, "Accessibility", green_accent);
                py += line_h + 8;
                
                self.draw_settings_toggle(px, py, "[1] High Contrast", crate::accessibility::is_high_contrast());
                py += line_h;
                
                self.draw_text_smooth(px, py, "[2] Font Size", text_label);
                self.draw_text_smooth(px + 160, py, crate::accessibility::get_font_size().label(), text_value);
                py += line_h;
                
                self.draw_text_smooth(px, py, "[3] Cursor Size", text_label);
                self.draw_text_smooth(px + 160, py, crate::accessibility::get_cursor_size().label(), text_value);
                py += line_h;
                
                self.draw_settings_toggle(px, py, "[4] Sticky Keys", crate::accessibility::is_sticky_keys());
                py += line_h;
                
                self.draw_text_smooth(px, py, "[5] Mouse Speed", text_label);
                self.draw_text_smooth(px + 160, py, crate::accessibility::get_mouse_speed().label(), text_value);
            },
            5 => { // Network
                self.draw_text_smooth(px, py, "Network", green_accent);
                self.draw_text_smooth(px + 1, py, "Network", green_accent);
                py += line_h + 8;
                
                // Interface info
                self.draw_text_smooth(px, py, "Interface", text_label);
                py += line_h;
                
                if let Some(mac) = crate::network::get_mac_address() {
                    self.draw_text_smooth(px + 12, py, &alloc::format!("MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]), text_value);
                } else {
                    self.draw_text_smooth(px + 12, py, "MAC: Not available", text_dim);
                }
                py += line_h;
                
                if let Some((ip, mask, gw)) = crate::network::get_ipv4_config() {
                    self.draw_text_smooth(px + 12, py, &alloc::format!("IP:   {}", ip), text_value);
                    py += line_h;
                    self.draw_text_smooth(px + 12, py, &alloc::format!("Mask: {}", mask), text_value);
                    py += line_h;
                    if let Some(g) = gw {
                        self.draw_text_smooth(px + 12, py, &alloc::format!("GW:   {}", g), text_value);
                    }
                } else {
                    self.draw_text_smooth(px + 12, py, "IP: Waiting for DHCP...", text_dim);
                }
                py += line_h + 8;
                
                let driver = if crate::virtio_net::is_initialized() { "virtio-net (active)" }
                    else if crate::drivers::net::has_driver() { "RTL8169/e1000 (active)" }
                    else { "No driver loaded" };
                self.draw_text_smooth(px, py, "Driver", text_label);
                self.draw_text_smooth(px + 80, py, driver, text_value);
            },
            6 => { // Apps
                self.draw_text_smooth(px, py, "Default Apps", green_accent);
                self.draw_text_smooth(px + 1, py, "Default Apps", green_accent);
                py += line_h + 8;
                
                let assocs = crate::file_assoc::list_associations();
                self.draw_text_smooth(px, py, "Extension", text_dim);
                self.draw_text_smooth(px + 100, py, "Program", text_dim);
                self.draw_text_smooth(px + 220, py, "Type", text_dim);
                py += 4;
                framebuffer::draw_hline((px) as u32, (py + 12) as u32, cw.saturating_sub(40), 0xFF1A3A1A);
                py += line_h;
                
                for (ext, prog, desc) in assocs.iter().take(10) {
                    self.draw_text_smooth(px, py, &alloc::format!(".{}", ext), text_value);
                    self.draw_text_smooth(px + 100, py, prog, text_label);
                    self.draw_text_smooth(px + 220, py, desc, text_dim);
                    py += line_h;
                }
                py += 8;
                self.draw_text_smooth(px, py, "[3] Edit File Associations...", text_label);
            },
            7 => { // About
                self.draw_text_smooth(px, py, "About TrustOS", green_accent);
                self.draw_text_smooth(px + 1, py, "About TrustOS", green_accent);
                py += line_h + 8;
                
                self.draw_text_smooth(px, py, "TrustOS", 0xFFCCEECC);
                self.draw_text_smooth(px + 1, py, "TrustOS", 0xFFCCEECC);
                py += line_h;
                self.draw_text_smooth(px, py, "Version 0.2.0", text_value);
                py += line_h;
                self.draw_text_smooth(px, py, "Bare-metal OS written in Rust", text_label);
                py += line_h + 8;
                
                self.draw_text_smooth(px, py, "Kernel", text_dim);
                self.draw_text_smooth(px + 80, py, "trustos_kernel (x86_64)", text_value);
                py += line_h;
                
                self.draw_text_smooth(px, py, "Arch", text_dim);
                self.draw_text_smooth(px + 80, py, "x86_64", text_value);
                py += line_h;
                
                self.draw_text_smooth(px, py, "Display", text_dim);
                self.draw_text_smooth(px + 80, py, &alloc::format!("{}x{}", self.width, self.height), text_value);
                py += line_h;
                
                self.draw_text_smooth(px, py, "AI", text_dim);
                self.draw_text_smooth(px + 80, py, "JARVIS (Transformer 4.4M params)", text_value);
                py += line_h + 8;
                
                self.draw_text_smooth(px, py, "(c) 2026 Nathan", text_label);
            },
            _ => {}
        }
    }
    
    /// Draw a toggle switch widget
    fn draw_settings_toggle(&self, x: i32, y: i32, label: &str, enabled: bool) {
        let text_label = 0xFF88AA88;
        let green_accent = GREEN_PRIMARY;
        self.draw_text_smooth(x, y, label, text_label);
        
        let tx = x + 180;
        let tw = 36u32;
        let th = 16u32;
        let track_color = if enabled { 0xFF1A5A2A } else { 0xFF1A1A1A };
        draw_rounded_rect(tx, y, tw, th, 8, track_color);
        draw_rounded_rect_border(tx, y, tw, th, 8, if enabled { green_accent } else { 0xFF333333 });
        
        let knob_x = if enabled { tx + tw as i32 - 14 } else { tx + 2 };
        let knob_color = if enabled { green_accent } else { 0xFF666666 };
        for dy in 0..12u32 {
            for dx in 0..12u32 {
                let ddx = dx as i32 - 6;
                let ddy = dy as i32 - 6;
                if ddx * ddx + ddy * ddy <= 36 {
                    framebuffer::put_pixel_fast((knob_x + dx as i32) as u32, (y as u32 + 2 + dy), knob_color);
                }
            }
        }
    }
    
    /// Draw a horizontal slider widget
    fn draw_settings_slider(&self, x: i32, y: i32, width: i32, value: u32, max_val: u32) {
        let track_w = width.max(40) as u32;
        let track_h = 6u32;
        let ty = y + 5;
        
        // Track background
        draw_rounded_rect(x, ty, track_w, track_h, 3, 0xFF1A1A1A);
        
        // Filled portion
        let fill_w = ((value as u64 * track_w as u64) / max_val.max(1) as u64) as u32;
        if fill_w > 0 {
            draw_rounded_rect(x, ty, fill_w.min(track_w), track_h, 3, 0xFF1A5A2A);
        }
        
        // Knob
        let knob_x = x + fill_w as i32;
        for dy in 0..10u32 {
            for dx in 0..10u32 {
                let ddx = dx as i32 - 5;
                let ddy = dy as i32 - 5;
                if ddx * ddx + ddy * ddy <= 25 {
                    framebuffer::put_pixel_fast((knob_x + dx as i32 - 5).max(0) as u32, (ty as u32 - 2 + dy), GREEN_PRIMARY);
                }
            }
        }
        
        // Value label
        self.draw_text_smooth(x + track_w as i32 + 8, y, &alloc::format!("{}", value), 0xFFBBDDBB);
    }
    
    /// Handle NetScan keyboard input
    fn handle_netscan_key(&mut self, key: u8) {
        // Tab switching: 1-6 for tabs, Left/Right arrows
        if key >= b'1' && key <= b'6' {
            self.netscan_tab = key - b'1';
            return;
        }
        use crate::keyboard::{KEY_LEFT, KEY_RIGHT};
        if key == KEY_LEFT {
            self.netscan_tab = self.netscan_tab.saturating_sub(1);
            return;
        }
        if key == KEY_RIGHT {
            if self.netscan_tab < 5 { self.netscan_tab += 1; }
            return;
        }
        
        match self.netscan_tab {
            1 => { // PortScan
                if key == b's' || key == b'S' {
                    if let Some((_ip, _mask, gw)) = crate::network::get_ipv4_config() {
                        if let Some(g) = gw {
                            let target = *g.as_bytes();
                            let (results, stats) = crate::netscan::port_scanner::quick_scan(target);
                            if let Some(window) = self.windows.iter_mut().find(|w| w.window_type == WindowType::NetworkInfo) {
                                window.content.clear();
                                window.content.push(alloc::format!("Scan: {} | Open: {} | Closed: {} | {:.0}ms",
                                    crate::netscan::format_ip(target), stats.open, stats.closed, stats.elapsed_ms));
                                for pr in &results {
                                    let state_str = match pr.state {
                                        crate::netscan::port_scanner::PortState::Open => "OPEN",
                                        crate::netscan::port_scanner::PortState::Closed => "closed",
                                        crate::netscan::port_scanner::PortState::Filtered => "filtered",
                                        _ => "unknown",
                                    };
                                    window.content.push(alloc::format!("  Port {}: {} ({})", pr.port, state_str, pr.service));
                                }
                                if results.is_empty() {
                                    window.content.push(String::from("  No open ports found"));
                                }
                            }
                        }
                    }
                }
            },
            2 => { // Discovery
                if key == b'd' || key == b'D' {
                    let hosts = crate::netscan::discovery::arp_sweep_local(3000);
                    if let Some(window) = self.windows.iter_mut().find(|w| w.window_type == WindowType::NetworkInfo) {
                        window.content.clear();
                        window.content.push(alloc::format!("ARP Sweep: {} hosts found", hosts.len()));
                        for host in &hosts {
                            let mac_str = match host.mac {
                                Some(m) => crate::netscan::format_mac(m),
                                None => String::from("??:??:??:??:??:??"),
                            };
                            window.content.push(alloc::format!("  {} - {} ({}ms)",
                                crate::netscan::format_ip(host.ip), mac_str, host.rtt_ms));
                        }
                        if hosts.is_empty() {
                            window.content.push(String::from("  No hosts discovered"));
                        }
                    }
                }
            },
            3 => { // Sniffer
                if key == b's' || key == b'S' {
                    if crate::netscan::sniffer::is_capturing() {
                        crate::netscan::sniffer::stop_capture();
                    } else {
                        crate::netscan::sniffer::start_capture();
                    }
                }
            },
            4 => { // Traceroute
                if key == b't' || key == b'T' {
                    if let Some((_ip, _mask, gw)) = crate::network::get_ipv4_config() {
                        if let Some(g) = gw {
                            let target = *g.as_bytes();
                            let hops = crate::netscan::traceroute::trace(target, 30, 5000);
                            let formatted = crate::netscan::traceroute::format_trace(&hops);
                            if let Some(window) = self.windows.iter_mut().find(|w| w.window_type == WindowType::NetworkInfo) {
                                window.content.clear();
                                for line in formatted.lines() {
                                    window.content.push(String::from(line));
                                }
                            }
                        }
                    }
                }
            },
            5 => { // VulnScan
                if key == b'v' || key == b'V' {
                    if let Some((_ip, _mask, gw)) = crate::network::get_ipv4_config() {
                        if let Some(g) = gw {
                            let target = *g.as_bytes();
                            // First do a quick scan to get open ports, then vuln scan those
                            let (port_results, _) = crate::netscan::port_scanner::quick_scan(target);
                            let open_ports: alloc::vec::Vec<u16> = port_results.iter()
                                .filter(|p| matches!(p.state, crate::netscan::port_scanner::PortState::Open))
                                .map(|p| p.port)
                                .collect();
                            let results = crate::netscan::vuln::scan(target, &open_ports);
                            let report = crate::netscan::vuln::format_report(target, &results);
                            if let Some(window) = self.windows.iter_mut().find(|w| w.window_type == WindowType::NetworkInfo) {
                                window.content.clear();
                                for line in report.lines() {
                                    window.content.push(String::from(line));
                                }
                            }
                        }
                    }
                }
            },
            _ => {}
        }
    }
    
    /// Draw the NetScan tabbed GUI
    fn draw_netscan_gui(&self, window: &Window) {
        let wx = window.x;
        let wy = window.y;
        let ww = window.width;
        let wh = window.height;
        
        if ww < 200 || wh < 120 { return; }
        
        let content_y = wy + TITLE_BAR_HEIGHT as i32;
        let content_h = wh.saturating_sub(TITLE_BAR_HEIGHT);
        let safe_x = wx.max(0) as u32;
        
        let bg = 0xFF0A140Cu32;
        let tab_bg = 0xFF060E08u32;
        let tab_active_bg = 0xFF0C2A14u32;
        let green_accent = GREEN_PRIMARY;
        let text_label = 0xFF88AA88u32;
        let text_value = 0xFFBBDDBBu32;
        let text_dim = 0xFF446644u32;
        let border_color = 0xFF1A3A1Au32;
        
        // ── Background ──
        framebuffer::fill_rect(safe_x, content_y as u32, ww, content_h, bg);
        
        // ── Tab bar ──
        let tab_h = 28u32;
        framebuffer::fill_rect(safe_x, content_y as u32, ww, tab_h, tab_bg);
        framebuffer::fill_rect(safe_x, (content_y + tab_h as i32) as u32, ww, 1, border_color);
        
        let tabs = ["Dashboard", "PortScan", "Discovery", "Sniffer", "Traceroute", "VulnScan"];
        let tab_w = (ww / tabs.len() as u32).max(80);
        
        for (i, label) in tabs.iter().enumerate() {
            let tx = safe_x + (i as u32 * tab_w);
            let is_active = i as u8 == self.netscan_tab;
            
            if is_active {
                framebuffer::fill_rect(tx, content_y as u32, tab_w, tab_h, tab_active_bg);
                // Green underline for active tab
                framebuffer::fill_rect(tx + 4, (content_y + tab_h as i32 - 2) as u32, tab_w - 8, 2, green_accent);
            }
            
            let c = if is_active { green_accent } else { text_dim };
            // Center text in tab
            let text_w = label.len() as i32 * 8;
            let text_x = tx as i32 + (tab_w as i32 - text_w) / 2;
            self.draw_text_smooth(text_x, content_y + 7, label, c);
        }
        
        // ── Content area ──
        let cx = safe_x as i32 + 16;
        let mut cy = content_y + tab_h as i32 + 12;
        let line_h = 20i32;
        let area_w = ww.saturating_sub(32);
        
        match self.netscan_tab {
            0 => { // Dashboard
                self.draw_text_smooth(cx, cy, "Network Dashboard", green_accent);
                self.draw_text_smooth(cx + 1, cy, "Network Dashboard", green_accent);
                cy += line_h + 8;
                
                // Connection status
                let connected = crate::virtio_net::is_initialized() || crate::drivers::net::has_driver();
                let status_color = if connected { 0xFF33DD66u32 } else { 0xFFDD3333u32 };
                let status_text = if connected { "Connected" } else { "Disconnected" };
                self.draw_text_smooth(cx, cy, "Status:", text_label);
                // Status dot
                for dy in 0..8u32 {
                    for dx in 0..8u32 {
                        let ddx = dx as i32 - 4;
                        let ddy = dy as i32 - 4;
                        if ddx * ddx + ddy * ddy <= 16 {
                            framebuffer::put_pixel_fast((cx + 70 + dx as i32) as u32, (cy + 4 + dy as i32) as u32, status_color);
                        }
                    }
                }
                self.draw_text_smooth(cx + 84, cy, status_text, status_color);
                cy += line_h;
                
                // Driver
                let driver = if crate::virtio_net::is_initialized() { "virtio-net" }
                    else if crate::drivers::net::has_driver() { "RTL8169/e1000" }
                    else { "None" };
                self.draw_text_smooth(cx, cy, "Driver:", text_label);
                self.draw_text_smooth(cx + 70, cy, driver, text_value);
                cy += line_h;
                
                // MAC
                if let Some(mac) = crate::network::get_mac_address() {
                    self.draw_text_smooth(cx, cy, "MAC:", text_label);
                    self.draw_text_smooth(cx + 70, cy, &alloc::format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]), text_value);
                }
                cy += line_h;
                
                // IPv4
                if let Some((ip, mask, gw)) = crate::network::get_ipv4_config() {
                    self.draw_text_smooth(cx, cy, "IP:", text_label);
                    self.draw_text_smooth(cx + 70, cy, &alloc::format!("{}", ip), text_value);
                    cy += line_h;
                    self.draw_text_smooth(cx, cy, "Subnet:", text_label);
                    self.draw_text_smooth(cx + 70, cy, &alloc::format!("{}", mask), text_value);
                    cy += line_h;
                    if let Some(g) = gw {
                        self.draw_text_smooth(cx, cy, "Gateway:", text_label);
                        self.draw_text_smooth(cx + 70, cy, &alloc::format!("{}", g), text_value);
                        cy += line_h;
                    }
                } else {
                    self.draw_text_smooth(cx, cy, "IPv4:", text_label);
                    self.draw_text_smooth(cx + 70, cy, "Waiting for DHCP...", text_dim);
                    cy += line_h;
                }
                
                cy += 8;
                // Packet stats
                let stats = crate::network::get_stats();
                self.draw_text_smooth(cx, cy, "Packets", text_dim);
                cy += line_h;
                self.draw_text_smooth(cx + 8, cy, &alloc::format!("TX: {}  RX: {}", stats.packets_sent, stats.packets_received), text_value);
                cy += line_h;
                self.draw_text_smooth(cx + 8, cy, &alloc::format!("Bytes TX: {}  RX: {}", stats.bytes_sent, stats.bytes_received), text_value);
                
                cy += line_h + 8;
                self.draw_text_smooth(cx, cy, "Use tabs [1-6] or Left/Right to navigate", text_dim);
            },
            1 => { // PortScan
                self.draw_text_smooth(cx, cy, "Port Scanner", green_accent);
                self.draw_text_smooth(cx + 1, cy, "Port Scanner", green_accent);
                cy += line_h + 8;
                
                if let Some((_ip, _mask, gw)) = crate::network::get_ipv4_config() {
                    if let Some(g) = gw {
                        self.draw_text_smooth(cx, cy, "Target:", text_label);
                        self.draw_text_smooth(cx + 70, cy, &alloc::format!("{} (gateway)", g), text_value);
                        cy += line_h + 4;
                    }
                }
                
                self.draw_text_smooth(cx, cy, "[S] Start Quick Scan", text_label);
                cy += line_h + 8;
                
                // Display results from window.content
                if !window.content.is_empty() {
                    framebuffer::fill_rect(safe_x + 8, cy as u32, area_w, 1, border_color);
                    cy += 6;
                    self.draw_text_smooth(cx, cy, "Results:", text_dim);
                    cy += line_h;
                    for line in window.content.iter() {
                        if cy > wy + wh as i32 - 20 { break; }
                        let c = if line.contains("OPEN") { 0xFF33DD66u32 } else { text_value };
                        self.draw_text_smooth(cx + 8, cy, line, c);
                        cy += line_h;
                    }
                }
            },
            2 => { // Discovery
                self.draw_text_smooth(cx, cy, "Network Discovery", green_accent);
                self.draw_text_smooth(cx + 1, cy, "Network Discovery", green_accent);
                cy += line_h + 8;
                
                self.draw_text_smooth(cx, cy, "[D] Run ARP Sweep", text_label);
                cy += line_h + 8;
                
                if !window.content.is_empty() {
                    framebuffer::fill_rect(safe_x + 8, cy as u32, area_w, 1, border_color);
                    cy += 6;
                    for line in window.content.iter() {
                        if cy > wy + wh as i32 - 20 { break; }
                        self.draw_text_smooth(cx + 8, cy, line, text_value);
                        cy += line_h;
                    }
                }
            },
            3 => { // Sniffer
                self.draw_text_smooth(cx, cy, "Packet Sniffer", green_accent);
                self.draw_text_smooth(cx + 1, cy, "Packet Sniffer", green_accent);
                cy += line_h + 8;
                
                let capturing = crate::netscan::sniffer::is_capturing();
                let status = if capturing { "Capturing..." } else { "Idle" };
                let sc = if capturing { 0xFF33DD66u32 } else { text_dim };
                self.draw_text_smooth(cx, cy, "Status:", text_label);
                self.draw_text_smooth(cx + 70, cy, status, sc);
                cy += line_h;
                
                let toggle_label = if capturing { "[S] Stop Capture" } else { "[S] Start Capture" };
                self.draw_text_smooth(cx, cy, toggle_label, text_label);
                cy += line_h + 8;
                
                let (total_pkts, total_bytes, buffered) = crate::netscan::sniffer::get_stats();
                self.draw_text_smooth(cx, cy, "Captured:", text_label);
                self.draw_text_smooth(cx + 80, cy, &alloc::format!("{} packets", total_pkts), text_value);
                cy += line_h;
                self.draw_text_smooth(cx, cy, "Bytes:", text_label);
                self.draw_text_smooth(cx + 80, cy, &alloc::format!("{}", total_bytes), text_value);
                cy += line_h;
                self.draw_text_smooth(cx, cy, "Buffered:", text_label);
                self.draw_text_smooth(cx + 80, cy, &alloc::format!("{}", buffered), text_value);
            },
            4 => { // Traceroute
                self.draw_text_smooth(cx, cy, "Traceroute", green_accent);
                self.draw_text_smooth(cx + 1, cy, "Traceroute", green_accent);
                cy += line_h + 8;
                
                if let Some((_ip, _mask, gw)) = crate::network::get_ipv4_config() {
                    if let Some(g) = gw {
                        self.draw_text_smooth(cx, cy, "Target:", text_label);
                        self.draw_text_smooth(cx + 70, cy, &alloc::format!("{}", g), text_value);
                        cy += line_h + 4;
                    }
                }
                
                self.draw_text_smooth(cx, cy, "[T] Run Traceroute", text_label);
                cy += line_h + 8;
                
                if !window.content.is_empty() {
                    framebuffer::fill_rect(safe_x + 8, cy as u32, area_w, 1, border_color);
                    cy += 6;
                    for line in window.content.iter() {
                        if cy > wy + wh as i32 - 20 { break; }
                        self.draw_text_smooth(cx + 8, cy, line, text_value);
                        cy += line_h;
                    }
                }
            },
            5 => { // VulnScan
                self.draw_text_smooth(cx, cy, "Vulnerability Scanner", green_accent);
                self.draw_text_smooth(cx + 1, cy, "Vulnerability Scanner", green_accent);
                cy += line_h + 8;
                
                if let Some((_ip, _mask, gw)) = crate::network::get_ipv4_config() {
                    if let Some(g) = gw {
                        self.draw_text_smooth(cx, cy, "Target:", text_label);
                        self.draw_text_smooth(cx + 70, cy, &alloc::format!("{}", g), text_value);
                        cy += line_h + 4;
                    }
                }
                
                self.draw_text_smooth(cx, cy, "[V] Run Vulnerability Scan", text_label);
                cy += line_h + 8;
                
                if !window.content.is_empty() {
                    framebuffer::fill_rect(safe_x + 8, cy as u32, area_w, 1, border_color);
                    cy += 6;
                    for line in window.content.iter() {
                        if cy > wy + wh as i32 - 20 { break; }
                        let c = if line.contains("VULN") || line.contains("HIGH") { 0xFFDD3333u32 }
                            else if line.contains("WARN") || line.contains("MEDIUM") { 0xFFDDAA33u32 }
                            else { text_value };
                        self.draw_text_smooth(cx + 8, cy, line, c);
                        cy += line_h;
                    }
                }
            },
            _ => {}
        }
    }
    
    /// Refresh settings window content to show current values
    fn refresh_settings_window(&mut self) {
        if let Some(window) = self.windows.iter_mut().find(|w| w.window_type == WindowType::Settings) {
            window.content.clear();
            window.content.push(String::from("=== Settings ==="));
            window.content.push(String::from(""));
            window.content.push(format!("Resolution: {}x{}", self.width, self.height));
            window.content.push(String::from("Theme: Dark Green"));
            window.content.push(String::from(""));
            window.content.push(String::from("--- Animations ---"));
            let anim_status = if animations_enabled() { "ON " } else { "OFF" };
            let anim_speed = *ANIMATION_SPEED.lock();
            window.content.push(format!("[1] Animations: {}", anim_status));
            window.content.push(format!("[2] Speed: {:.1}x", anim_speed));
            window.content.push(String::from(""));
            window.content.push(String::from("--- Accessibility ---"));
            let hc = if crate::accessibility::is_high_contrast() { "ON " } else { "OFF" };
            window.content.push(format!("[5] High Contrast: {}", hc));
            window.content.push(format!("[6] Font Size: {}", crate::accessibility::get_font_size().label()));
            window.content.push(format!("[7] Cursor Size: {}", crate::accessibility::get_cursor_size().label()));
            window.content.push(format!("[8] Sticky Keys: {}", if crate::accessibility::is_sticky_keys() { "ON" } else { "OFF" }));
            window.content.push(format!("[9] Mouse Speed: {}", crate::accessibility::get_mouse_speed().label()));
            window.content.push(String::from(""));
            window.content.push(String::from("--- Other ---"));
            window.content.push(String::from("[3] File Associations"));
            window.content.push(String::from("[4] About System"));
        }
    }
    
    /// Handle file associations keyboard input
    fn handle_fileassoc_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN};
        
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileAssociations) {
            let list_start = 4; // After header
            let list_end = window.content.len().saturating_sub(2);
            let list_len = list_end.saturating_sub(list_start);
            
            if key == KEY_UP && window.selected_index > 0 {
                window.selected_index -= 1;
            } else if key == KEY_DOWN && window.selected_index < list_len.saturating_sub(1) {
                window.selected_index += 1;
            } else if key == 0x0D || key == 0x0A {
                // Cycle through programs for selected extension
                let idx = list_start + window.selected_index;
                if idx < list_end {
                    // Get extension from line
                    let line = &window.content[idx];
                    if let Some(ext_end) = line.find('|') {
                        let ext = line[1..ext_end].trim().trim_start_matches('.');
                        // Cycle to next program
                        use crate::file_assoc::{Program, set_program, get_program_for_file};
                        let current = get_program_for_file(&format!("test.{}", ext));
                        let next = match current {
                            Program::TextEditor => Program::ImageViewer,
                            Program::ImageViewer => Program::HexViewer,
                            Program::HexViewer => Program::Terminal,
                            Program::Terminal => Program::TextEditor,
                            _ => Program::TextEditor,
                        };
                        set_program(ext, next.clone());
                        // Update display
                        crate::serial_println!("[ASSOC] {} -> {}", ext, next.name());
                    }
                }
            }
        }
    }
    
    /// Remove any existing auto-suggestion lines from the terminal window
    fn clear_terminal_suggestions(&mut self) {
        if self.terminal_suggestion_count > 0 {
            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                for _ in 0..self.terminal_suggestion_count {
                    window.content.pop();
                }
            }
            self.terminal_suggestion_count = 0;
        }
    }
    
    /// Show auto-suggestions below the prompt based on current input
    fn show_terminal_suggestions(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }
        let partial = self.input_buffer.as_str();
        let commands = crate::shell::SHELL_COMMANDS;
        let matches: Vec<&str> = commands.iter().copied()
            .filter(|c| c.starts_with(partial) && *c != partial)
            .collect();
        if matches.is_empty() {
            return;
        }
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
            // Show up to 6 suggestions on one line
            let display: Vec<&str> = matches.iter().copied().take(6).collect();
            let line = format!("  \x01M> {}", display.join("  "));
            window.content.push(line);
            self.terminal_suggestion_count = 1;
            // If many matches, show count on a second line
            if matches.len() > 6 {
                window.content.push(format!("    \x01M... +{} more", matches.len() - 6));
                self.terminal_suggestion_count = 2;
            }
        }
    }
    
    /// Build a colored prompt string with timestamp
    fn make_prompt(suffix: &str) -> String {
        let dt = crate::rtc::read_rtc();
        let cwd = crate::ramfs::with_fs(|fs| {
            let p = fs.pwd();
            String::from(p)
        });
        let display_cwd = if cwd == "/" { String::from("~") } else { cwd };
        format!("\x01B[{:02}:{:02}:{:02}] \x01Rroot\x01M@trustos\x01M:\x01B{}\x01M$ \x01G{}", dt.hour, dt.minute, dt.second, display_cwd, suffix)
    }
    
    /// Handle terminal keyboard input
    fn handle_terminal_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_PGUP, KEY_PGDOWN};
        // Clear old suggestion lines so content.last() is the prompt again
        self.clear_terminal_suggestions();
        
        // PageUp / PageDown — scroll terminal output
        if key == KEY_PGUP || key == KEY_PGDOWN {
            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                let line_height = 16usize;
                let content_area_h = (window.height as usize).saturating_sub(TITLE_BAR_HEIGHT as usize + 16);
                let visible_lines = if line_height > 0 { content_area_h / line_height } else { 1 };
                let max_scroll = window.content.len().saturating_sub(visible_lines);
                if key == KEY_PGUP {
                    window.scroll_offset = window.scroll_offset.saturating_sub(visible_lines);
                } else {
                    window.scroll_offset = (window.scroll_offset + visible_lines).min(max_scroll);
                }
            }
            return;
        }
        
        if key == 0x08 { // Backspace
            if !self.input_buffer.is_empty() {
                self.input_buffer.pop();
                if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                    if let Some(last) = window.content.last_mut() {
                        *last = Self::make_prompt(&format!("{}_", self.input_buffer));
                    }
                }
            }
        } else if key == 0x09 { // Tab — autosuggestion (commands + file names)
            let partial = self.input_buffer.clone();
            if !partial.is_empty() {
                // Check if we're completing a command argument (contains space) or a command name
                if let Some(space_pos) = partial.rfind(' ') {
                    // Completing a filename argument
                    let file_partial = &partial[space_pos + 1..];
                    if !file_partial.is_empty() {
                        // List files from ramfs and find matches
                        let mut file_matches: Vec<String> = Vec::new();
                        if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(Some("/"))) {
                            for (name, ftype, _size) in entries.iter() {
                                let name_lower: String = name.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                                let partial_lower: String = file_partial.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                                if name_lower.starts_with(&partial_lower) {
                                    let suffix = if *ftype == crate::ramfs::FileType::Directory { "/" } else { "" };
                                    file_matches.push(format!("{}{}", name, suffix));
                                }
                            }
                        }
                        if file_matches.len() == 1 {
                            let cmd_prefix = &partial[..=space_pos];
                            self.input_buffer = format!("{}{}", cmd_prefix, file_matches[0]);
                            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                                if let Some(last) = window.content.last_mut() {
                                    *last = Self::make_prompt(&format!("{}_", self.input_buffer));
                                }
                            }
                        } else if file_matches.len() > 1 {
                            let match_str: String = file_matches.join("  ");
                            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                                window.content.push(match_str);
                                window.content.push(Self::make_prompt(&format!("{}_", self.input_buffer)));
                            }
                        }
                    }
                } else {
                    // Completing a command name
                    let commands = crate::shell::SHELL_COMMANDS;
                    let partial_str = partial.as_str();
                    let matches: Vec<&str> = commands.iter().copied().filter(|c| c.starts_with(partial_str)).collect();
                    if matches.len() == 1 {
                        self.input_buffer = String::from(matches[0]);
                        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                            if let Some(last) = window.content.last_mut() {
                                *last = Self::make_prompt(&format!("{}_", self.input_buffer));
                            }
                        }
                    } else if matches.len() > 1 {
                        let match_str: String = matches.join("  ");
                        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                            window.content.push(match_str);
                            window.content.push(Self::make_prompt(&format!("{}_", self.input_buffer)));
                        }
                    }
                }
            }
        } else if key == 0xF0 { // Up arrow — history previous
            if !self.command_history.is_empty() {
                match self.history_index {
                    None => {
                        // Start browsing: save current input, go to last command
                        self.saved_input = self.input_buffer.clone();
                        let idx = self.command_history.len() - 1;
                        self.history_index = Some(idx);
                        self.input_buffer = self.command_history[idx].clone();
                    }
                    Some(i) if i > 0 => {
                        let idx = i - 1;
                        self.history_index = Some(idx);
                        self.input_buffer = self.command_history[idx].clone();
                    }
                    _ => {} // already at oldest
                }
                if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                    if let Some(last) = window.content.last_mut() {
                        *last = Self::make_prompt(&format!("{}_", self.input_buffer));
                    }
                }
            }
        } else if key == 0xF1 { // Down arrow — history next
            if let Some(i) = self.history_index {
                if i + 1 < self.command_history.len() {
                    let idx = i + 1;
                    self.history_index = Some(idx);
                    self.input_buffer = self.command_history[idx].clone();
                } else {
                    // Past the end: restore saved input
                    self.history_index = None;
                    self.input_buffer = self.saved_input.clone();
                }
                if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                    if let Some(last) = window.content.last_mut() {
                        *last = Self::make_prompt(&format!("{}_", self.input_buffer));
                    }
                }
            }
        } else if key == 0x0D || key == 0x0A { // Enter
            let cmd = self.input_buffer.clone();
            self.input_buffer.clear();
            // Save to history (skip duplicates of last entry)
            if !cmd.trim().is_empty() {
                let dup = self.command_history.last().map(|h| h == &cmd).unwrap_or(false);
                if !dup {
                    self.command_history.push(cmd.clone());
                }
            }
            self.history_index = None;
            self.saved_input.clear();
            
            let output = Self::execute_command_static(&cmd);
            
            // Post-command: handle commands that need &mut self (window creation, etc.)
            let cmd_trimmed = cmd.trim();
            if cmd_trimmed.starts_with("play ") {
                let arg = cmd_trimmed.strip_prefix("play ").unwrap_or("").trim();
                match arg {
                    "u2" | "untitled2" | "lofi" | "untitled" => {
                        // Create music player widget and start playback
                        let mp_x = self.width.saturating_sub(320) as i32;
                        let mp_y = self.height.saturating_sub(TASKBAR_HEIGHT + 600) as i32;
                        let wid = self.create_window("Music Player", mp_x, mp_y.max(20), 320, 580, WindowType::MusicPlayer);
                        if let Some(mp_state) = self.music_player_states.get_mut(&wid) {
                            mp_state.play_track(0);
                        }
                    },
                    _ => {},
                }
            }
            
            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                // Handle "clear" specially — wipe all content
                if cmd.trim() == "clear" {
                    window.content.clear();
                    window.content.push(Self::make_prompt("_"));
                    window.scroll_offset = 0;
                } else {
                    // Remove cursor line
                    if window.content.last().map(|s| s.contains("$ ")).unwrap_or(false) {
                        window.content.pop();
                    }
                    // Show executed command with prompt
                    window.content.push(Self::make_prompt(&cmd));
                    
                    for line in output {
                        window.content.push(line);
                    }
                    
                    // Add new prompt
                    window.content.push(Self::make_prompt("_"));
                    
                    // Auto-scroll to bottom
                    let line_height = 16usize;
                    let content_area_h = (window.height as usize).saturating_sub(TITLE_BAR_HEIGHT as usize + 16);
                    let visible_lines = if line_height > 0 { content_area_h / line_height } else { 1 };
                    if window.content.len() > visible_lines {
                        window.scroll_offset = window.content.len() - visible_lines;
                    } else {
                        window.scroll_offset = 0;
                    }
                }
            }
        } else if key >= 0x20 && key < 0x7F {
            self.input_buffer.push(key as char);
            
            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                if let Some(last) = window.content.last_mut() {
                    *last = Self::make_prompt(&format!("{}_", self.input_buffer));
                }
            }
        }
        
        // Show auto-suggestions below prompt (except after Enter — buffer is empty)
        self.show_terminal_suggestions();
    }
    
    /// Execute terminal command (static to avoid borrow issues)
    fn execute_command_static(cmd: &str) -> Vec<String> {
        let mut output = Vec::new();
        let cmd = cmd.trim();
        
        // Debug: log command to serial
        crate::serial_println!("[TERM] Executing command: '{}' len={}", cmd, cmd.len());
        
        if cmd.is_empty() {
            return output;
        }
        
        match cmd {
            "help" => {
                output.push(String::from("\x01HTrustOS Terminal \x01M- Available Commands"));
                output.push(String::from(""));
                // File System
                output.push(String::from("\x01Y[File System]"));
                output.push(String::from("  \x01Gls \x01B[dir]      \x01WList directory contents"));
                output.push(String::from("  \x01Gcd \x01B<dir>      \x01WChange directory"));
                output.push(String::from("  \x01Gpwd            \x01WPrint working directory"));
                output.push(String::from("  \x01Gcat \x01B<file>    \x01WShow file contents"));
                output.push(String::from("  \x01Gmkdir \x01B<name>  \x01WCreate directory"));
                output.push(String::from("  \x01Gtouch \x01B<name>  \x01WCreate empty file"));
                output.push(String::from("  \x01Grm \x01B<file>     \x01WRemove file"));
                output.push(String::from(""));
                // System Info
                output.push(String::from("\x01Y[System]"));
                output.push(String::from("  \x01Gdate           \x01WCurrent date and time"));
                output.push(String::from("  \x01Guname          \x01WSystem information"));
                output.push(String::from("  \x01Gfree           \x01WMemory usage"));
                output.push(String::from("  \x01Gps             \x01WList processes"));
                output.push(String::from("  \x01Guptime         \x01WSystem uptime"));
                output.push(String::from("  \x01Gdf             \x01WDisk usage"));
                output.push(String::from("  \x01Gwhoami         \x01WCurrent user"));
                output.push(String::from("  \x01Ghostname       \x01WSystem hostname"));
                output.push(String::from("  \x01Gneofetch       \x01WSystem info banner"));
                output.push(String::from(""));
                // Network
                output.push(String::from("\x01Y[Network]"));
                output.push(String::from("  \x01Gnet            \x01WNetwork interface status"));
                output.push(String::from("  \x01Gifconfig       \x01WNetwork configuration"));
                output.push(String::from(""));
                // Graphics & Demos
                output.push(String::from("\x01Y[Graphics & Demos]"));
                output.push(String::from("  \x01Gshader \x01B<name>  \x01WRun GPU shader demo"));
                output.push(String::from("  \x01Gmatrix3d       \x01W3D Matrix tunnel"));
                output.push(String::from("  \x01Gshowcase3d     \x01W3D cinematic demo"));
                output.push(String::from("  \x01Gfilled3d       \x01WFilled 3D test"));
                output.push(String::from("  \x01Gchess          \x01WChess game vs AI"));
                output.push(String::from("  \x01Gchess3d        \x01W3D chess (Matrix style)"));
                output.push(String::from(""));
                // Audio
                output.push(String::from("\x01Y[Audio]"));
                output.push(String::from("  \x01Gplay \x01B<track>  \x01WPlay music (u2, lofi)"));
                output.push(String::from(""));
                // Shell
                output.push(String::from("\x01Y[Shell]"));
                output.push(String::from("  \x01Ghelp           \x01WShow this help"));
                output.push(String::from("  \x01Gecho \x01B<text>   \x01WPrint text"));
                output.push(String::from("  \x01Gclear          \x01WClear terminal"));
                output.push(String::from("  \x01Gexit           \x01WClose terminal"));
            },
            // Direct shortcut for 3D Matrix tunnel
            "matrix3d" | "tunnel" | "holomatrix" | "3d" => {
                output.push(String::from("✓ Matrix Tunnel 3D - ESC to exit"));
                
                // Get framebuffer info
                let fb = crate::framebuffer::get_framebuffer();
                let width = crate::framebuffer::width();
                let height = crate::framebuffer::height();
                
                // Init virtual GPU with tunnel shader
                crate::gpu_emu::init(fb, width, height);
                if let Some(shader_fn) = crate::gpu_emu::get_shader("tunnel") {
                    crate::gpu_emu::set_shader(shader_fn);
                }
                
                let mut frames = 0u32;
                loop {
                    if let Some(key) = crate::keyboard::try_read_key() {
                        if key == 27 { break; }
                    }
                    
                    #[cfg(target_arch = "x86_64")]
                    crate::gpu_emu::draw_simd();
                    #[cfg(not(target_arch = "x86_64"))]
                    crate::gpu_emu::draw();
                    
                    crate::gpu_emu::tick(16);
                    frames += 1;
                    
                    if frames % 60 == 0 {
                        crate::framebuffer::draw_text("MATRIX 3D TUNNEL | ESC=exit", 10, 10, 0xFF00FF00);
                    }
                }
                output.push(format!("Tunnel ended ({} frames)", frames));
            },
            "ls" => {
                let cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
                output.push(format!("\x01MDirectory: \x01B{}", cwd));
                if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(None)) {
                    for (name, ftype, size) in entries.iter().take(20) {
                        let icon = if *ftype == crate::ramfs::FileType::Directory { "\x01B" } else { "\x01M" };
                        let type_str = if *ftype == crate::ramfs::FileType::Directory { "/" } else { "" };
                        output.push(format!("  {}{}{}  \x01M{} bytes", icon, name, type_str, size));
                    }
                    if entries.is_empty() {
                        output.push(String::from("\x01M  (empty directory)"));
                    }
                }
            },
            "ls /" => {
                output.push(String::from("\x01MDirectory: \x01B/"));
                if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(Some("/"))) {
                    for (name, ftype, size) in entries.iter().take(20) {
                        let icon = if *ftype == crate::ramfs::FileType::Directory { "\x01B" } else { "\x01M" };
                        let type_str = if *ftype == crate::ramfs::FileType::Directory { "/" } else { "" };
                        output.push(format!("  {}{}{}  \x01M{} bytes", icon, name, type_str, size));
                    }
                    if entries.is_empty() {
                        output.push(String::from("\x01M  (empty directory)"));
                    }
                }
            },
            _ if cmd.starts_with("ls ") => {
                let path = &cmd[3..];
                output.push(format!("\x01MDirectory: \x01B{}", path));
                if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(Some(path))) {
                    for (name, ftype, size) in entries.iter().take(20) {
                        let icon = if *ftype == crate::ramfs::FileType::Directory { "\x01B" } else { "\x01M" };
                        let type_str = if *ftype == crate::ramfs::FileType::Directory { "/" } else { "" };
                        output.push(format!("  {}{}{}  \x01M{} bytes", icon, name, type_str, size));
                    }
                    if entries.is_empty() {
                        output.push(String::from("\x01M  (empty directory)"));
                    }
                } else {
                    output.push(format!("\x01Rls: cannot access '{}': No such file or directory", path));
                }
            },
            "pwd" => {
                let cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
                output.push(format!("\x01B{}", cwd));
            },
            "clear" => {
                // Will be handled specially
            },
            "date" | "time" => {
                let dt = crate::rtc::read_rtc();
                output.push(format!("\x01B{:04}-{:02}-{:02} \x01W{:02}:{:02}:{:02}", 
                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second));
            },
            "uname" | "uname -a" | "version" => {
                output.push(String::from("\x01GTrustOS \x01W0.1.1 \x01Bx86_64 \x01MRust Kernel"));
                output.push(format!("\x01MHeap: \x01W{} MB", crate::memory::heap_size() / 1024 / 1024));
            },
            "neofetch" => {
                output.push(String::from("\x01G  _____               _    ___  ___"));
                output.push(String::from("\x01G |_   _| __ _   _ ___| |_ / _ \\/ __|"));
                output.push(String::from("\x01G   | || '__| | | / __| __| | | \\__ \\"));
                output.push(String::from("\x01G   | || |  | |_| \\__ \\ |_| |_| |__) |"));
                output.push(String::from("\x01G   |_||_|   \\__,_|___/\\__|\\___/|___/"));
                output.push(String::from(""));
                output.push(String::from("\x01BOS\x01M:      \x01WTrustOS 0.1.1"));
                output.push(String::from("\x01BKernel\x01M:  \x01Wtrustos_kernel"));
                output.push(String::from("\x01BArch\x01M:    \x01Wx86_64"));
                output.push(format!("\x01BUptime\x01M:  \x01W{}m {}s", crate::logger::get_ticks() / 100 / 60, (crate::logger::get_ticks() / 100) % 60));
                output.push(format!("\x01BMemory\x01M:  \x01W{} MB", crate::memory::heap_size() / 1024 / 1024));
                output.push(format!("\x01BShell\x01M:   \x01Wtrustsh"));
                output.push(format!("\x01BDisplay\x01M: \x01W{}x{}", crate::framebuffer::width(), crate::framebuffer::height()));
            },
            "whoami" | "user" | "users" | "id" => {
                output.push(String::from("\x01Groot"));
            },
            "hostname" => {
                output.push(String::from("\x01Gtrustos"));
            },
            "history" => {
                // Show real command history from DESKTOP
                let hist = crate::desktop::DESKTOP.lock().command_history.clone();
                if hist.is_empty() {
                    output.push(String::from("\x01M  (no history yet)"));
                } else {
                    for (i, entry) in hist.iter().enumerate() {
                        output.push(format!("\x01M  {}  {}", i + 1, entry));
                    }
                }
            },
            "free" | "mem" => {
                let heap_mb = crate::memory::heap_size() / 1024 / 1024;
                output.push(String::from("\x01YMemory Usage:"));
                output.push(format!("  \x01BHeap Size: \x01W{} MB", heap_mb));
                output.push(String::from("  \x01BKernel:   \x01GActive"));
            },
            "net" | "ifconfig" | "ip" | "ipconfig" => {
                output.push(String::from("\x01YNetwork Status:"));
                if crate::network::is_available() {
                    if let Some((mac, ip, _state)) = crate::network::get_interface() {
                        output.push(format!("  \x01BMAC: \x01W{}", mac));
                        if let Some(ip) = ip {
                            output.push(format!("  \x01BIP:  \x01W{}", ip));
                        }
                        output.push(String::from("  \x01BStatus: \x01GConnected"));
                    }
                } else {
                    output.push(String::from("  \x01BStatus: \x01RNo network"));
                }
            },
            _ if cmd.starts_with("cat ") => {
                let filename = &cmd[4..].trim();
                if let Ok(content) = crate::ramfs::with_fs(|fs| fs.read_file(filename).map(|d| d.to_vec())) {
                    if let Ok(text) = core::str::from_utf8(&content) {
                        for line in text.lines().take(20) {
                            output.push(String::from(line));
                        }
                    } else {
                        output.push(format!("cat: {}: binary file", filename));
                    }
                } else {
                    output.push(format!("cat: {}: No such file", filename));
                }
            },
            _ if cmd.starts_with("echo ") => {
                output.push(String::from(&cmd[5..]));
            },
            "cd" => {
                // cd alone = go to root
                let _ = crate::ramfs::with_fs(|fs| fs.cd("/"));
            },
            _ if cmd.starts_with("cd ") => {
                let path = &cmd[3..].trim();
                match crate::ramfs::with_fs(|fs| fs.cd(path)) {
                    Ok(()) => {
                        let cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
                        output.push(format!("\x01B{}", cwd));
                    },
                    Err(e) => output.push(format!("\x01Rcd: {}: {}", path, e.as_str())),
                }
            },
            _ if cmd.starts_with("mkdir ") => {
                let path = cmd[6..].trim();
                match crate::ramfs::with_fs(|fs| fs.mkdir(path)) {
                    Ok(()) => output.push(format!("\x01Gmkdir: \x01Wcreated '\x01B{}\x01W'", path)),
                    Err(e) => output.push(format!("\x01Rmkdir: {}: {}", path, e.as_str())),
                }
            },
            _ if cmd.starts_with("touch ") => {
                let path = cmd[6..].trim();
                match crate::ramfs::with_fs(|fs| fs.touch(path)) {
                    Ok(()) => output.push(format!("\x01Gtouch: \x01Wcreated '\x01B{}\x01W'", path)),
                    Err(e) => output.push(format!("\x01Rtouch: {}: {}", path, e.as_str())),
                }
            },
            _ if cmd.starts_with("rm ") || cmd.starts_with("del ") => {
                let path = if cmd.starts_with("rm ") { cmd[3..].trim() } else { cmd[4..].trim() };
                match crate::ramfs::with_fs(|fs| fs.rm(path)) {
                    Ok(()) => output.push(format!("\x01Grm: \x01Wremoved '\x01B{}\x01W'", path)),
                    Err(e) => output.push(format!("\x01Rrm: {}: {}", path, e.as_str())),
                }
            },
            "shader" | "vgpu" => {
                output.push(String::from("╔═══════════════════════════════════════╗"));
                output.push(String::from("║     Virtual GPU - Shader Demo         ║"));
                output.push(String::from("╠═══════════════════════════════════════╣"));
                output.push(String::from("║ shader plasma    - Plasma waves       ║"));
                output.push(String::from("║ shader fire      - Fire effect        ║"));
                output.push(String::from("║ shader mandelbrot- Fractal zoom       ║"));
                output.push(String::from("║ shader matrix    - Matrix rain        ║"));
                output.push(String::from("║ shader tunnel    - 3D HOLOMATRIX      ║"));
                output.push(String::from("║ shader shapes    - 3D OBJECTS         ║"));
                output.push(String::from("║ shader parallax  - Depth layers       ║"));
                output.push(String::from("║ shader gradient  - Test gradient      ║"));
                output.push(String::from("╚═══════════════════════════════════════╝"));
                output.push(String::from("Press ESC to exit shader demo"));
            },
            _ if cmd.starts_with("shader ") => {
                let shader_name = cmd.trim_start_matches("shader ").trim();
                if let Some(shader_fn) = crate::gpu_emu::get_shader(shader_name) {
                    output.push(format!("✓ Starting shader: {} (ESC to exit)", shader_name));
                    
                    // Get framebuffer info
                    let fb = crate::framebuffer::get_framebuffer();
                    let width = crate::framebuffer::width();
                    let height = crate::framebuffer::height();
                    
                    // Init virtual GPU
                    crate::gpu_emu::init(fb, width, height);
                    crate::gpu_emu::set_shader(shader_fn);
                    
                    // Run shader demo loop
                    let mut frames = 0u32;
                    
                    loop {
                        // Check for ESC key
                        if let Some(key) = crate::keyboard::try_read_key() {
                            if key == 27 { break; }
                        }
                        
                        // Draw shader
                        #[cfg(target_arch = "x86_64")]
                        crate::gpu_emu::draw_simd();
                        #[cfg(not(target_arch = "x86_64"))]
                        crate::gpu_emu::draw();
                        
                        // Update time (~16ms per frame target)
                        crate::gpu_emu::tick(16);
                        frames += 1;
                        
                        // Show FPS every 60 frames
                        if frames % 60 == 0 {
                            crate::framebuffer::draw_text(&format!("FPS: ~60 | {} | ESC=exit", shader_name), 10, 10, 0xFFFFFFFF);
                        }
                    }
                    
                    output.push(format!("Shader ended ({} frames)", frames));
                } else {
                    output.push(format!("Unknown shader: {}", shader_name));
                    output.push(String::from("Available: plasma, fire, mandelbrot, matrix, tunnel, parallax, gradient"));
                }
            },
            "ps" | "procs" | "top" => {
                output.push(String::from("\x01BPID  \x01BSTATE    \x01BNAME"));
                output.push(String::from("  \x01W1  \x01GRunning  \x01Winit"));
                output.push(String::from("  \x01W2  \x01GRunning  \x01Wdesktop"));
                output.push(String::from("  \x01W3  \x01GRunning  \x01Wterminal"));
            },
            "uptime" => {
                let ticks = crate::logger::get_ticks();
                let secs = ticks / 100;
                let mins = secs / 60;
                output.push(format!("\x01BUptime: \x01W{}m {}s", mins, secs % 60));
            },
            "df" | "lsblk" => {
                output.push(String::from("\x01BFilesystem      Size  Used  Avail Use%"));
                output.push(String::from("\x01Wramfs           32M   1M    31M   3%"));
            },
            "showcase3d" | "demo3d" => {
                output.push(String::from("\u{2713} Showcase 3D Cinematic - ESC to skip scenes"));
                drop(output);
                crate::shell::desktop::cmd_showcase3d();
                return Vec::new();
            },
            "filled3d" => {
                output.push(String::from("\u{2713} Filled 3D Test - ESC to exit"));
                drop(output);
                crate::shell::desktop::cmd_filled3d();
                return Vec::new();
            },
            "exit" | "quit" => {
                output.push(String::from("\x01MUse the X button to close the terminal"));
            },
            "desktop" | "gui" | "mobile" => {
                output.push(String::from("\x01MAlready in desktop mode."));
            },
            "chess" => {
                output.push(String::from("\x01G\u{265A} TrustChess \x01M— Opening chess window..."));
                output.push(String::from("\x01MPlay vs AI (Black). Arrow keys, Enter, Esc."));
            },
            "chess3d" => {
                output.push(String::from("\x01G\u{265A} TrustChess 3D \x01M— Opening 3D chess window..."));
                output.push(String::from("\x01MWASD:Camera  ZX:Zoom  O:Auto-rotate  Click:Move"));
            },
            "gameboy" | "gb" => {
                output.push(String::from("\x01G\u{1F3AE} Game Boy \x01M— Opening Game Boy window..."));
                output.push(String::from("\x01MWASD:D-Pad X/Space:A Z:B C:Select Enter:Start"));
            },
            _ if cmd.starts_with("play ") || cmd == "play" => {
                let arg = cmd.strip_prefix("play ").unwrap_or("").trim();
                if arg.is_empty() {
                    output.push(String::from("\x01Y\u{266B} Usage: \x01Gplay u2"));
                    output.push(String::from("\x01MTracks: u2, untitled2, lofi"));
                } else {
                    match arg {
                        "u2" | "untitled2" | "lofi" | "untitled" => {
                            output.push(String::from("\x01G\u{266B} Playing Untitled (2) — Lo-Fi"));
                            output.push(String::from("\x01MOpening Music Player widget..."));
                            // Will be handled by terminal command dispatch in handle_terminal_command
                        },
                        _ => {
                            output.push(format!("\x01RTrack not found: \x01W{}", arg));
                            output.push(String::from("\x01MAvailable: u2, untitled2, lofi"));
                        },
                    }
                }
            },
            _ if cmd.starts_with("j ") || cmd.starts_with("jarvis ") || cmd == "j" || cmd == "jarvis" => {
                // JARVIS commands: run async on background thread to avoid freezing desktop
                if JARVIS_BUSY.load(core::sync::atomic::Ordering::SeqCst) {
                    output.push(String::from("\x01Y[Jarvis] \x01MStill thinking... please wait."));
                } else {
                    JARVIS_BUSY.store(true, core::sync::atomic::Ordering::SeqCst);
                    {
                        let mut pending = JARVIS_PENDING_QUERY.lock();
                        *pending = Some(String::from(cmd));
                    }
                    crate::thread::spawn_kernel("jarvis-bg", jarvis_worker, 0);
                    output.push(String::from("\x01Y[Jarvis] \x01M\u{1F4AD} Thinking..."));
                }
            },
            _ => {
                // Route through the real shell engine so ALL commands are available
                // Clear captured output, enable capture, run command, collect output
                crate::shell::take_captured(); // clear any stale data
                crate::shell::CAPTURE_MODE.store(true, core::sync::atomic::Ordering::SeqCst);
                crate::shell::execute_command(cmd);
                crate::shell::CAPTURE_MODE.store(false, core::sync::atomic::Ordering::SeqCst);
                let captured = crate::shell::take_captured();
                if !captured.is_empty() {
                    for line in captured.lines() {
                        output.push(String::from(line));
                    }
                }
            },
        }
        
        output
    }
    
    /// Handle mouse move
    pub fn handle_move(&mut self, x: i32, y: i32) {
        self.cursor_x = x.clamp(0, self.width as i32 - 1);
        self.cursor_y = y.clamp(0, self.height as i32 - 1);
        
        // Update drag-and-drop position
        if self.drag_state.is_some() {
            self.update_drag(x, y);
        }
        
        for w in &mut self.windows {
            // Handle window dragging
            if w.dragging && !w.maximized {
                w.x = (x - w.drag_offset_x).max(0).min(self.width as i32 - 50);
                w.y = (y - w.drag_offset_y).max(0).min(self.height as i32 - TASKBAR_HEIGHT as i32 - TITLE_BAR_HEIGHT as i32);
                
                // Detect snap zone while dragging
                let edge_margin = 16i32;
                let sw = self.width as i32;
                let sh = (self.height - TASKBAR_HEIGHT) as i32;
                let half_h = sh / 2;
                
                if x <= edge_margin && y <= edge_margin + half_h / 4 {
                    self.snap_preview = Some(SnapDir::TopLeft);
                } else if x <= edge_margin && y >= sh - half_h / 4 {
                    self.snap_preview = Some(SnapDir::BottomLeft);
                } else if x <= edge_margin {
                    self.snap_preview = Some(SnapDir::Left);
                } else if x >= sw - edge_margin && y <= edge_margin + half_h / 4 {
                    self.snap_preview = Some(SnapDir::TopRight);
                } else if x >= sw - edge_margin && y >= sh - half_h / 4 {
                    self.snap_preview = Some(SnapDir::BottomRight);
                } else if x >= sw - edge_margin {
                    self.snap_preview = Some(SnapDir::Right);
                } else {
                    self.snap_preview = None;
                }
            }
            
            // Handle window resizing (all edges)
            if w.resizing != ResizeEdge::None {
                let dx = x - w.drag_offset_x;
                let dy = y - w.drag_offset_y;
                
                // Right edge: expand width
                match w.resizing {
                    ResizeEdge::Right | ResizeEdge::BottomRight | ResizeEdge::TopRight => {
                        let new_width = (w.width as i32 + dx).max(w.min_width as i32) as u32;
                        w.width = new_width.min(self.width - w.x as u32);
                        w.drag_offset_x = x;
                    }
                    _ => {}
                }
                
                // Left edge: move x and shrink width
                match w.resizing {
                    ResizeEdge::Left | ResizeEdge::BottomLeft | ResizeEdge::TopLeft => {
                        let new_width = (w.width as i32 - dx).max(w.min_width as i32) as u32;
                        if new_width != w.width as u32 {
                            w.x += (w.width as i32 - new_width as i32);
                            w.width = new_width;
                        }
                        w.drag_offset_x = x;
                    }
                    _ => {}
                }
                
                // Bottom edge: expand height
                match w.resizing {
                    ResizeEdge::Bottom | ResizeEdge::BottomRight | ResizeEdge::BottomLeft => {
                        let new_height = (w.height as i32 + dy).max(w.min_height as i32) as u32;
                        w.height = new_height.min(self.height - TASKBAR_HEIGHT - w.y as u32);
                        w.drag_offset_y = y;
                    }
                    _ => {}
                }
                
                // Top edge: move y and shrink height
                match w.resizing {
                    ResizeEdge::Top | ResizeEdge::TopLeft | ResizeEdge::TopRight => {
                        let new_height = (w.height as i32 - dy).max(w.min_height as i32) as u32;
                        if new_height != w.height as u32 {
                            w.y += (w.height as i32 - new_height as i32);
                            w.height = new_height;
                        }
                        w.drag_offset_y = y;
                    }
                    _ => {}
                }
            }
        }
        
        // Forward mouse move to focused model editor
        let model_info: Option<(u32, i32, i32, u32, u32)> = self.windows.iter()
            .find(|w| w.focused && !w.minimized && w.window_type == WindowType::ModelEditor)
            .map(|w| (w.id, w.x, w.y, w.width, w.height));
        if let Some((win_id, wx, wy, ww, wh)) = model_info {
            let vx = x - wx;
            let vy = y - wy - TITLE_BAR_HEIGHT as i32;
            let vw = ww as usize;
            let vh = wh.saturating_sub(TITLE_BAR_HEIGHT) as usize;
            if let Some(state) = self.model_editor_states.get_mut(&win_id) {
                state.handle_mouse_move(vx, vy, vw, vh);
            }
        }
        
        // Update chess drag position
        let chess_info: Option<u32> = self.windows.iter()
            .find(|w| w.focused && !w.minimized && w.window_type == WindowType::Chess)
            .map(|w| w.id);
        if let Some(win_id) = chess_info {
            if let Some(chess) = self.chess_states.get_mut(&win_id) {
                if chess.drag_from.is_some() {
                    chess.update_drag_position(x, y);
                }
            }
        }
        
        // Forward mouse move to Chess3D (drag rotation)
        let chess3d_info: Option<(u32, i32, i32)> = self.windows.iter()
            .find(|w| w.focused && !w.minimized && w.window_type == WindowType::Chess3D)
            .map(|w| (w.id, w.x, w.y));
        if let Some((win_id, wx, wy)) = chess3d_info {
            if let Some(state) = self.chess3d_states.get_mut(&win_id) {
                let rel_x = x - wx;
                let rel_y = y - wy - TITLE_BAR_HEIGHT as i32;
                state.handle_mouse_move(rel_x, rel_y);
            }
        }
    }
    
    /// Handle scroll wheel
    pub fn handle_scroll(&mut self, delta: i8) {
        // Handle model editor scroll (zoom) separately
        let model_info = self.windows.iter().rev().find(|w| w.focused && !w.minimized && w.window_type == WindowType::ModelEditor).map(|w| w.id);
        if let Some(win_id) = model_info {
            if let Some(state) = self.model_editor_states.get_mut(&win_id) {
                state.handle_scroll(delta);
            }
            return;
        }
        // Handle Chess3D scroll (zoom) separately
        let chess3d_info = self.windows.iter().rev().find(|w| w.focused && !w.minimized && w.window_type == WindowType::Chess3D).map(|w| w.id);
        if let Some(win_id) = chess3d_info {
            if let Some(state) = self.chess3d_states.get_mut(&win_id) {
                state.handle_scroll(delta);
            }
            return;
        }
        // Scroll focused window content if it's a scrollable type
        if let Some(window) = self.windows.iter_mut().rev().find(|w| w.focused && !w.minimized) {
            match window.window_type {
                WindowType::FileManager | WindowType::TextEditor | WindowType::HexViewer | 
                WindowType::FileAssociations | WindowType::Terminal => {
                    let max_scroll = if window.content.len() > 10 {
                        window.content.len() - 10
                    } else {
                        0
                    };
                    
                    if delta > 0 {
                        // Scroll up
                        if window.scroll_offset > 0 {
                            window.scroll_offset = window.scroll_offset.saturating_sub(3);
                        }
                    } else if delta < 0 {
                        // Scroll down
                        window.scroll_offset = (window.scroll_offset + 3).min(max_scroll);
                    }
                },
                _ => {}
            }
        }
    }
    
    /// Process touch input events and gestures.
    /// Called each frame from the run() loop, after mouse input.
    pub fn process_touch_input(&mut self) {
        // Process all pending touch events through the gesture recognizer
        self.gesture_buffer.clear();
        self.gesture_recognizer.process_all(&mut self.gesture_buffer);
        
        // Handle each recognized gesture
        if !self.gesture_buffer.is_empty() {
            self.touch_mode = true; // Switch to touch mode
        }
        
        // Collect gestures into a temp array to avoid borrow issues
        let mut gesture_list: [(u8, i32, i32, i32, i32, i32); 8] = [(0, 0, 0, 0, 0, 0); 8];
        let mut gesture_count = 0usize;
        
        for gesture in self.gesture_buffer.iter() {
            if gesture_count >= 8 { break; }
            // Pack gesture type and key data into the array
            match gesture {
                crate::gesture::GestureEvent::Tap { x, y } => {
                    gesture_list[gesture_count] = (1, *x, *y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::DoubleTap { x, y } => {
                    gesture_list[gesture_count] = (2, *x, *y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::LongPress { x, y } => {
                    gesture_list[gesture_count] = (3, *x, *y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Swipe { direction, start_x, start_y, end_x, end_y, .. } => {
                    let dir_code = match direction {
                        crate::gesture::SwipeDirection::Left => 0,
                        crate::gesture::SwipeDirection::Right => 1,
                        crate::gesture::SwipeDirection::Up => 2,
                        crate::gesture::SwipeDirection::Down => 3,
                    };
                    gesture_list[gesture_count] = (4, *start_x, *start_y, *end_x, *end_y, dir_code);
                }
                crate::gesture::GestureEvent::EdgeSwipe { origin, progress } => {
                    let origin_code = match origin {
                        crate::gesture::EdgeOrigin::Bottom => 0,
                        crate::gesture::EdgeOrigin::Top => 1,
                        crate::gesture::EdgeOrigin::Left => 2,
                        crate::gesture::EdgeOrigin::Right => 3,
                    };
                    gesture_list[gesture_count] = (5, origin_code, *progress, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Pinch { center_x, center_y, scale } => {
                    gesture_list[gesture_count] = (6, *center_x, *center_y, *scale, 0, 0);
                }
                crate::gesture::GestureEvent::Scroll { delta_x, delta_y } => {
                    gesture_list[gesture_count] = (7, *delta_x, *delta_y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::ThreeFingerSwipe { direction } => {
                    let dir_code = match direction {
                        crate::gesture::SwipeDirection::Left => 0,
                        crate::gesture::SwipeDirection::Right => 1,
                        crate::gesture::SwipeDirection::Up => 2,
                        crate::gesture::SwipeDirection::Down => 3,
                    };
                    gesture_list[gesture_count] = (8, dir_code, 0, 0, 0, 0);
                }
                crate::gesture::GestureEvent::TouchDown { x, y } => {
                    gesture_list[gesture_count] = (9, *x, *y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::TouchMove { x, y } => {
                    gesture_list[gesture_count] = (10, *x, *y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::TouchUp { x, y } => {
                    gesture_list[gesture_count] = (11, *x, *y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Drag { x, y, start_x, start_y } => {
                    gesture_list[gesture_count] = (12, *x, *y, *start_x, *start_y, 0);
                }
            }
            gesture_count += 1;
        }
        
        // Now dispatch each gesture (no longer borrowing gesture_buffer)
        for i in 0..gesture_count {
            let (gtype, a, b, c, d, e) = gesture_list[i];
            match gtype {
                1 => { // Tap → left click
                    self.update_cursor(a, b);
                    self.handle_click(a, b, true);
                    self.handle_click(a, b, false);
                }
                2 => { // DoubleTap → double click
                    self.update_cursor(a, b);
                    self.handle_click(a, b, true);
                    self.handle_click(a, b, false);
                    self.handle_click(a, b, true);
                    self.handle_click(a, b, false);
                }
                3 => { // LongPress → right click (context menu)
                    self.update_cursor(a, b);
                    self.handle_right_click(a, b, true);
                    self.handle_right_click(a, b, false);
                }
                4 => { // Swipe
                    match e {
                        0 => { /* Swipe Left — could map to browser back */ }
                        1 => { /* Swipe Right — could map to browser forward */ }
                        2 => { /* Swipe Up */ }
                        3 => { /* Swipe Down */ }
                        _ => {}
                    }
                }
                5 => { // EdgeSwipe
                    match a {
                        0 => { // Bottom edge → open start menu (launcher)
                            if !self.start_menu_open {
                                self.start_menu_open = true;
                            }
                        }
                        1 => { // Top edge → (notification panel placeholder)
                        }
                        _ => {}
                    }
                }
                6 => { // Pinch → zoom (scale factor)
                    // c = scale (100 = no change)
                    // Could be used by image viewer, maps, etc. in future
                }
                7 => { // Two-finger scroll
                    let scroll_delta = if b > 0 { -1i8 } else if b < 0 { 1i8 } else { 0i8 };
                    if scroll_delta != 0 {
                        self.handle_scroll(scroll_delta);
                    }
                }
                8 => { // Three-finger swipe → app switch (Alt+Tab equiv)
                    // a = direction code (0=left, 1=right)
                    self.cycle_windows();
                }
                9 => { // TouchDown — move cursor to touch position
                    self.update_cursor(a, b);
                }
                10 => { // TouchMove — track finger as cursor
                    self.update_cursor(a, b);
                }
                11 => { // TouchUp
                    // Nothing extra needed
                }
                12 => { // Drag — move cursor  
                    self.update_cursor(a, b);
                }
                _ => {}
            }
        }
    }
    
    /// Update cursor position (used by both mouse and touch)
    fn update_cursor(&mut self, x: i32, y: i32) {
        self.cursor_x = x.clamp(0, self.width as i32 - 1);
        self.cursor_y = y.clamp(0, self.height as i32 - 1);
    }
    
    /// Cycle to next window (used by 3-finger swipe / Alt+Tab)
    fn cycle_windows(&mut self) {
        if self.windows.len() < 2 {
            return;
        }
        // Find currently focused window and move focus to next
        let focused_idx = self.windows.iter().position(|w| w.focused);
        if let Some(idx) = focused_idx {
            let next = (idx + 1) % self.windows.len();
            for w in self.windows.iter_mut() {
                w.focused = false;
            }
            self.windows[next].focused = true;
        }
    }
    
    /// Draw the desktop with double buffering
    pub fn draw(&mut self) {
        self.frame_count += 1;
        
        // ── Safe startup: first 3 frames do MINIMAL rendering ──
        // Eliminates all heavy code (matrix rain, audio analysis, game ticks,
        // animations) from the initial frames. If this fixes the T61 freeze,
        // the bug is in one of the skipped subsystems.
        if self.frame_count <= 3 {
            crate::serial_println!("[Desktop] safe frame {} / 3", self.frame_count);
            // Get mouse state for cursor
            let mouse = crate::mouse::get_state();
            
            framebuffer::clear_backbuffer(0xFF010200);
            framebuffer::begin_frame();
            // Just draw icons + taskbar + cursor — no rain, no audio, no effects
            self.draw_desktop_icons();
            self.draw_taskbar();
            self.draw_cursor();
            // Update tracking state
            self.last_cursor_x = mouse.x;
            self.last_cursor_y = mouse.y;
            self.last_window_count = self.windows.len();
            self.last_start_menu_open = self.start_menu_open;
            self.last_context_menu_visible = self.context_menu.visible;
            framebuffer::end_frame();
            framebuffer::swap_buffers();
            crate::serial_println!("[Desktop] safe frame {} done", self.frame_count);
            return;
        }
        
        // ── Auto-adjust tier based on sustained FPS ──
        self.auto_adjust_tier();
        
        // ── FPS measurement (tick-based, updated every ~1 second) ──
        self.fps_frame_accum += 1;
        let now_tick = crate::logger::get_ticks();
        if self.fps_last_tick == 0 { self.fps_last_tick = now_tick; }
        let elapsed = now_tick.saturating_sub(self.fps_last_tick);
        // Timer interrupt fires at ~100 Hz (1 tick = ~10ms), so 100 ticks ≈ 1 sec
        if elapsed >= 100 {
            self.fps_current = ((self.fps_frame_accum as u64 * 100) / elapsed.max(1)) as u32;
            self.fps_frame_accum = 0;
            self.fps_last_tick = now_tick;
        }
        
        // Update animations each frame
        self.update_animations();
        
        // Tick snake games — only when window is focused and visible
        let snake_ids: Vec<u32> = self.snake_states.keys().copied().collect();
        for id in snake_ids {
            let is_active = self.windows.iter().any(|w| w.id == id && w.focused && w.visible && !w.minimized);
            if is_active {
                if let Some(snake) = self.snake_states.get_mut(&id) {
                    snake.tick();
                }
            }
        }
        
        // Tick music players — always tick if playing (even minimized, for DMA refill)
        let mp_ids: Vec<u32> = self.music_player_states.keys().copied().collect();
        for id in mp_ids {
            if let Some(mp) = self.music_player_states.get_mut(&id) {
                mp.tick();
            }
        }
        
        // Tick 3D game — only when window is focused and visible
        let game3d_ids: Vec<u32> = self.game3d_states.keys().copied().collect();
        for id in game3d_ids {
            let is_active = self.windows.iter().any(|w| w.id == id && w.focused && w.visible && !w.minimized);
            if is_active {
                if let Some(game) = self.game3d_states.get_mut(&id) {
                    game.tick();
                }
            }
        }
        
        // Tick NES emulator — only when window is focused and visible
        #[cfg(feature = "emulators")]
        {
        let nes_ids: Vec<u32> = self.nes_states.keys().copied().collect();
        for id in nes_ids {
            let is_active = self.windows.iter().any(|w| w.id == id && w.focused && w.visible && !w.minimized);
            if is_active {
                if let Some(emu) = self.nes_states.get_mut(&id) {
                    emu.tick();
                }
            }
        }
        }
        
        // Tick Game Boy emulator — runs whenever visible (not just focused)
        // Integrates GameLab speed control, pausing, breakpoints, trace
        #[cfg(feature = "emulators")]
        {
        let gb_ids: Vec<u32> = self.gameboy_states.keys().copied().collect();
        for id in gb_ids {
            let is_active = self.windows.iter().any(|w| w.id == id && w.visible && !w.minimized && !w.pending_close);
            if is_active {
                // Find linked GameLab (if any)
                let lab_id = self.gamelab_states.iter()
                    .find(|(_, lab)| lab.linked_gb_id == Some(id))
                    .map(|(&lid, _)| lid)
                    .or_else(|| self.gamelab_states.keys().next().copied());
                
                // Read lab state
                let (paused, mut step_one, mut step_frame, speed_idx, trace_enabled) =
                    if let Some(lid) = lab_id {
                        if let Some(lab) = self.gamelab_states.get(&lid) {
                            (lab.paused, lab.step_one, lab.step_frame, lab.speed_idx, lab.trace_enabled)
                        } else { (false, false, false, 2, false) }
                    } else { (false, false, false, 2, false) };
                
                // Handle pause
                if paused && !step_one && !step_frame {
                    // Still poll key releases
                    if let Some(emu) = self.gameboy_states.get_mut(&id) {
                        if !crate::keyboard::is_key_pressed(0x11) { emu.handle_key_release(b'w'); }
                        if !crate::keyboard::is_key_pressed(0x1E) { emu.handle_key_release(b'a'); }
                        if !crate::keyboard::is_key_pressed(0x1F) { emu.handle_key_release(b's'); }
                        if !crate::keyboard::is_key_pressed(0x20) { emu.handle_key_release(b'd'); }
                        if !crate::keyboard::is_key_pressed(0x2D) { emu.handle_key_release(b'x'); }
                        if !crate::keyboard::is_key_pressed(0x2C) { emu.handle_key_release(b'z'); }
                        if !crate::keyboard::is_key_pressed(0x2E) { emu.handle_key_release(b'c'); }
                        if !crate::keyboard::is_key_pressed(0x1C) { emu.handle_key_release(b'\r'); }
                    }
                    continue;
                }

                // Speed control: accumulate ticks
                let ticks = match speed_idx {
                    0 => if self.frame_count % 4 == 0 { 1 } else { 0 }, // 0.25x
                    1 => if self.frame_count % 2 == 0 { 1 } else { 0 }, // 0.5x
                    2 => 1, // 1x
                    3 => 2, // 2x
                    4 => 4, // 4x
                    _ => 1,
                };

                if let Some(emu) = self.gameboy_states.get_mut(&id) {
                    // Poll keyboard
                    if !crate::keyboard::is_key_pressed(0x11) { emu.handle_key_release(b'w'); }
                    if !crate::keyboard::is_key_pressed(0x1E) { emu.handle_key_release(b'a'); }
                    if !crate::keyboard::is_key_pressed(0x1F) { emu.handle_key_release(b's'); }
                    if !crate::keyboard::is_key_pressed(0x20) { emu.handle_key_release(b'd'); }
                    if !crate::keyboard::is_key_pressed(0x2D) { emu.handle_key_release(b'x'); }
                    if !crate::keyboard::is_key_pressed(0x2C) { emu.handle_key_release(b'z'); }
                    if !crate::keyboard::is_key_pressed(0x2E) { emu.handle_key_release(b'c'); }
                    if !crate::keyboard::is_key_pressed(0x1C) { emu.handle_key_release(b'\r'); }

                    // Record trace before tick
                    if trace_enabled {
                        if let Some(lid) = lab_id {
                            // Need to split borrow — record PC info then pass to lab
                            let pc = emu.cpu.pc;
                            let a = emu.cpu.a;
                            let f = emu.cpu.f;
                            let sp = emu.cpu.sp;
                            let opcode = crate::game_lab::read_emu_byte(emu, pc);
                            drop(emu); // release borrow
                            if let Some(lab) = self.gamelab_states.get_mut(&lid) {
                                if lab.trace.len() >= 64 { lab.trace.remove(0); }
                                lab.trace.push(crate::game_lab::TraceEntry { pc, opcode, a, f, sp });
                            }
                            // Re-borrow emu for tick
                            if let Some(emu) = self.gameboy_states.get_mut(&id) {
                                for _ in 0..ticks { emu.tick(); }
                            }
                            // Clear step flags
                            if step_one || step_frame {
                                if let Some(lab) = self.gamelab_states.get_mut(&lid) {
                                    lab.step_one = false;
                                    lab.step_frame = false;
                                }
                            }
                            // Update watches + mem_diff
                            if let Some(emu) = self.gameboy_states.get(&id) {
                                if let Some(lab) = self.gamelab_states.get_mut(&lid) {
                                    lab.update_watches(emu);
                                    crate::game_lab::update_mem_diff(lab, emu);
                                    // Check breakpoints
                                    if lab.should_break(emu.cpu.pc) {
                                        lab.paused = true;
                                    }
                                }
                            }
                            continue; // skip the code below since we handled everything
                        }
                    }

                    // Normal tick (no trace)
                    for _ in 0..ticks { emu.tick(); }
                }

                // Clear step flags + update watches (no trace path)
                if let Some(lid) = lab_id {
                    if step_one || step_frame {
                        if let Some(lab) = self.gamelab_states.get_mut(&lid) {
                            lab.step_one = false;
                            lab.step_frame = false;
                        }
                    }
                    if let Some(emu) = self.gameboy_states.get(&id) {
                        if let Some(lab) = self.gamelab_states.get_mut(&lid) {
                            lab.update_watches(emu);
                            crate::game_lab::update_mem_diff(lab, emu);
                            if lab.should_break(emu.cpu.pc) {
                                lab.paused = true;
                            }
                        }
                    }
                }
            }
        }
        }
        
        // Tick chess timers (~60fps → ~16ms per frame)
        let chess_ids: Vec<u32> = self.chess_states.keys().copied().collect();
        for id in chess_ids {
            let is_active = self.windows.iter().any(|w| w.id == id && w.visible && !w.minimized);
            if is_active {
                if let Some(chess) = self.chess_states.get_mut(&id) {
                    chess.tick_timer(16); // ~16ms per frame at 60fps
                }
            }
        }
        
        // Tick TrustLab states — live data refresh
        let lab_ids: Vec<u32> = self.lab_states.keys().copied().collect();
        for id in lab_ids {
            let is_active = self.windows.iter().any(|w| w.id == id && w.visible && !w.minimized);
            if is_active {
                if let Some(lab) = self.lab_states.get_mut(&id) {
                    lab.tick();
                }
            }
        }
        
        // Tick GameLab states
        #[cfg(feature = "emulators")]
        {
        let gamelab_ids: Vec<u32> = self.gamelab_states.keys().copied().collect();
        for id in gamelab_ids {
            let is_active = self.windows.iter().any(|w| w.id == id && w.visible && !w.minimized);
            if is_active {
                if let Some(lab) = self.gamelab_states.get_mut(&id) {
                    lab.tick();
                }
            }
        }
        }
        
        // Toggle cursor blink every ~9 frames (~500ms at 18fps)
        if self.frame_count % 9 == 0 {
            self.cursor_blink = !self.cursor_blink;
        }
        
        // Get mouse state
        let mouse = crate::mouse::get_state();
        
        // Use OpenGL compositor if enabled
        if self.render_mode == RenderMode::OpenGL {
            self.draw_opengl();
            return;
        }
        
        // Use GPU-accelerated mode with dirty rect tracking
        if self.render_mode == RenderMode::GpuAccelerated {
            self.draw_gpu_accelerated();
            return;
        }
        
        // === CLASSIC RENDERING MODE ===
        
        // Detect state changes that require taskbar/background recache
        let windows_changed = self.windows.len() != self.last_window_count;
        let menu_changed = self.start_menu_open != self.last_start_menu_open 
                        || self.context_menu.visible != self.last_context_menu_visible;
        
        // If windows changed, invalidate taskbar in background cache
        if windows_changed {
            self.needs_full_redraw = true;
        }
        
        // ════════ MOBILE MODE: render portrait UI instead of desktop ════════
        if self.mobile_state.active {
            self.draw_mobile_mode();
            framebuffer::swap_buffers();
            return;
        }
        
        // OPTIMIZATION 1: Background rendering — tier-adaptive
        // Full: animated matrix rain every frame (4 layers + visualizer + drones)
        // Standard: 2-layer rain, no visualizer/drones
        // Minimal: 1-layer simplified rain (lightweight, any CPU can handle it)
        framebuffer::clear_backbuffer(0xFF010200);
        framebuffer::begin_frame(); // Cache BB pointer — all put_pixel_fast calls are zero-lock
        self.draw_background();
        self.draw_desktop_icons();
        
        // Draw windows (these change, so always redraw)
        let has_visible_windows = self.windows.iter().any(|w| w.visible && !w.minimized);
        for window in &self.windows {
            if window.visible && !window.minimized {
                self.draw_window(window);
            }
        }
        
        // Second pass: render editor content (needs &mut for blink counter)
        self.draw_editor_windows();
        
        // Third pass: render model editor windows (needs &mut for state)
        self.draw_model_editor_windows();
        
        // Fourth pass: render 3D game windows (needs &mut for state)
        self.draw_game3d_windows();
        
        // Fifth pass: render 3D chess windows (needs &mut for state)
        self.draw_chess3d_windows();
        
        // Sixth pass: render emulator windows
        #[cfg(feature = "emulators")]
        self.draw_nes_windows();
        #[cfg(feature = "emulators")]
        self.draw_gameboy_windows();
        
        // Draw snap preview overlay (translucent green zone while dragging to edge)
        if let Some(snap_dir) = self.snap_preview {
            let work_h = self.height - TASKBAR_HEIGHT;
            let half_w = self.width / 2;
            let half_h = work_h / 2;
            let (sx, sy, sw, sh) = match snap_dir {
                SnapDir::Left       => (0, 0, half_w, work_h),
                SnapDir::Right      => (half_w, 0, half_w, work_h),
                SnapDir::TopLeft    => (0, 0, half_w, half_h),
                SnapDir::TopRight   => (half_w, 0, half_w, half_h),
                SnapDir::BottomLeft => (0, half_h, half_w, half_h),
                SnapDir::BottomRight => (half_w, half_h, half_w, half_h),
            };
            // Translucent green fill
            framebuffer::fill_rect_alpha(sx, sy, sw, sh, 0x00FF66, 18);
            // Green border outline
            framebuffer::draw_rect(sx + 2, sy + 2, sw.saturating_sub(4), sh.saturating_sub(4), GREEN_MUTED);
            framebuffer::draw_rect(sx + 3, sy + 3, sw.saturating_sub(6), sh.saturating_sub(6), GREEN_GHOST);
        }
        
        // ALWAYS draw taskbar last (on top of everything, never covered by windows)
        self.draw_taskbar();
        
        // Draw start menu if open
        if self.start_menu_open {
            self.draw_start_menu();
        }
        
        // Draw context menu if visible
        if self.context_menu.visible {
            self.draw_context_menu();
        }
        
        // Draw drag-and-drop ghost
        self.draw_drag_ghost();
        
        // Draw lock screen if active (covers everything)
        if self.lock_screen_active {
            self.draw_lock_screen();
        }

        // Draw cursor last
        self.draw_cursor();
        
        // Update tracking state
        self.last_cursor_x = mouse.x;
        self.last_cursor_y = mouse.y;
        self.last_window_count = self.windows.len();
        self.last_start_menu_open = self.start_menu_open;
        self.last_context_menu_visible = self.context_menu.visible;
        
        // End frame caching and swap buffers
        framebuffer::end_frame();
        framebuffer::swap_buffers();
    }
    
    /// Draw the desktop using OpenGL compositor
    fn draw_opengl(&mut self) {
        use crate::graphics::opengl::*;
        
        // Update compositor state
        let dt = 1.0 / 60.0; // Assume 60 FPS
        
        // Sync windows to compositor surfaces
        {
            let mut comp = compositor::compositor();
            comp.update(dt);
            
            // Update surface positions from windows
            for window in &self.windows {
                if let Some(surface) = comp.get_surface_mut(window.id) {
                    surface.x = window.x as f32;
                    surface.y = window.y as f32;
                    surface.width = window.width as f32;
                    surface.height = window.height as f32;
                    surface.focused = window.focused;
                    surface.visible = window.visible && !window.minimized;
                }
            }
        }
        
        // Render using compositor
        compositor::render_compositor_frame();
        
        // Draw additional UI elements on top (taskbar, menus, cursor)
        // These are rendered classically on top of the OpenGL content
        self.draw_taskbar();
        
        if self.start_menu_open {
            self.draw_start_menu();
        }
        
        if self.context_menu.visible {
            self.draw_context_menu();
        }
        
        // Draw desktop icons
        self.draw_desktop_icons();
        
        // Draw cursor
        self.draw_cursor();
        
        // Update state
        self.last_window_count = self.windows.len();
        self.last_start_menu_open = self.start_menu_open;
        self.last_context_menu_visible = self.context_menu.visible;
        
        // Swap buffers
        framebuffer::swap_buffers();
    }
    
    // ════════════════════════════════════════════════════════════════════════
    // GPU-ACCELERATED RENDERING (Upgrade #2 + #3)
    // Same classic pipeline but with dirty-rect tracking and VirtIO partial flush
    // ════════════════════════════════════════════════════════════════════════
    
    /// Add a dirty rectangle for GPU-accelerated partial flush
    fn add_dirty_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        if self.dirty_rect_count < 32 {
            self.dirty_rects[self.dirty_rect_count] = (x, y, w, h);
            self.dirty_rect_count += 1;
        }
    }
    
    /// GPU-accelerated draw: classic rendering + dirty rect tracking + VirtIO partial flush
    fn draw_gpu_accelerated(&mut self) {
        let mouse = crate::mouse::get_state();
        let windows_changed = self.windows.len() != self.last_window_count;
        let menu_changed = self.start_menu_open != self.last_start_menu_open
                        || self.context_menu.visible != self.last_context_menu_visible;
        let cursor_moved = mouse.x != self.last_cursor_x || mouse.y != self.last_cursor_y;
        
        // Reset dirty rects for this frame
        self.dirty_rect_count = 0;
        
        // Track what's dirty this frame
        if windows_changed || menu_changed || self.needs_full_redraw {
            // Full redraw needed — mark entire screen dirty
            self.add_dirty_rect(0, 0, self.width, self.height);
            self.needs_full_redraw = false;
        } else {
            // Partial: only mark changed regions
            if cursor_moved {
                // Old cursor position
                let old_x = (self.last_cursor_x.max(0) as u32).saturating_sub(2);
                let old_y = (self.last_cursor_y.max(0) as u32).saturating_sub(2);
                self.add_dirty_rect(old_x, old_y, 24, 24);
                // New cursor position
                let new_x = (mouse.x.max(0) as u32).saturating_sub(2);
                let new_y = (mouse.y.max(0) as u32).saturating_sub(2);
                self.add_dirty_rect(new_x, new_y, 24, 24);
            }
            // Mark dirty rects for each visible window (content may animate)
            // Collect window rects first to avoid borrow conflict
            let win_rects: Vec<(u32, u32, u32, u32)> = self.windows.iter()
                .filter(|w| w.visible && !w.minimized)
                .map(|w| (w.x.max(0) as u32, w.y.max(0) as u32, w.width, w.height))
                .collect();
            for (wx, wy, ww, wh) in win_rects {
                self.add_dirty_rect(wx, wy, ww, wh);
            }
            // Taskbar is always dirty (clock updates)
            if self.frame_count % 60 == 0 {
                self.add_dirty_rect(0, self.height.saturating_sub(40), self.width, 40);
            }
        }
        
        // Render using the classic pipeline (draws to backbuffer)
        if self.mobile_state.active {
            self.draw_mobile_mode();
        } else {
            framebuffer::clear_backbuffer(0xFF000000);
            self.draw_background();
            self.draw_desktop_icons();
            
            for window in &self.windows {
                if window.visible && !window.minimized {
                    self.draw_window(window);
                }
            }
            self.draw_editor_windows();
            self.draw_model_editor_windows();
            self.draw_game3d_windows();
            self.draw_chess3d_windows();
            #[cfg(feature = "emulators")]
            self.draw_nes_windows();
            #[cfg(feature = "emulators")]
            self.draw_gameboy_windows();
            self.draw_taskbar();
            if self.start_menu_open { self.draw_start_menu(); }
            if self.context_menu.visible { self.draw_context_menu(); }
            self.draw_drag_ghost();
            if self.lock_screen_active { self.draw_lock_screen(); }
            self.draw_cursor();
        }
        
        // Update tracking
        self.last_cursor_x = mouse.x;
        self.last_cursor_y = mouse.y;
        self.last_window_count = self.windows.len();
        self.last_start_menu_open = self.start_menu_open;
        self.last_context_menu_visible = self.context_menu.visible;
        
        // Use VirtIO GPU partial flush if available (Upgrade #3)
        if crate::drivers::virtio_gpu::is_available() && self.dirty_rect_count > 0 && self.dirty_rect_count < 32 {
            // Copy backbuffer to GPU, then partial flush only dirty regions
            crate::drivers::virtio_gpu::present_dirty_rects(
                &self.dirty_rects[..self.dirty_rect_count]
            );
            // Also MMIO fallback for VGA
            framebuffer::swap_buffers_mmio_only();
        } else {
            framebuffer::swap_buffers();
        }
        
        self.gpu_frame_skip = self.gpu_frame_skip.wrapping_add(1);
    }
    
    /// Draw context menu - Windows 11 style with rounded corners and glass
    fn draw_context_menu(&self) {
        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let menu_width = 200i32;
        let item_height = 28;
        let menu_height = self.context_menu.items.len() as i32 * item_height + 8;
        let padding = 4;
        let corner_r: u32 = 8;
        
        // Soft rounded shadow (multiple layers, increasingly offset)
        for i in (1..=6).rev() {
            let alpha = (18 - i * 2).max(4) as u32;
            let shadow_color = alpha << 24;
            draw_rounded_rect(
                menu_x + i, menu_y + i + 2,
                menu_width as u32, menu_height as u32,
                corner_r + 2, shadow_color,
            );
        }
        
        // Background: dark glass panel with rounded corners
        draw_rounded_rect(
            menu_x, menu_y,
            menu_width as u32, menu_height as u32,
            corner_r, 0xFF0C1210,
        );
        
        // Glass-like gradient overlay (lighter at top, fading down)
        // We draw thin horizontal slices clipped to the rounded shape
        for row in 0..menu_height.min(20) {
            let glass_alpha = (12 - row * 12 / 20).max(0) as u32;
            if glass_alpha > 0 {
                let overlay = (glass_alpha << 24) | 0x00FFFFFF;
                // Inset to respect rounded corners
                let inset = if row < corner_r as i32 { (corner_r as i32 - fast_sqrt_i32((corner_r as i32 * corner_r as i32) - (corner_r as i32 - row) * (corner_r as i32 - row))) } else { 0 };
                let lx = menu_x + inset;
                let lw = menu_width - inset * 2;
                if lw > 0 {
                    crate::framebuffer::fill_rect(lx as u32, (menu_y + row) as u32, lw as u32, 1, overlay);
                }
            }
        }
        
        // Chrome border (TrustOS logo style)
        draw_rounded_rect_border(
            menu_x, menu_y,
            menu_width as u32, menu_height as u32,
            corner_r, CHROME_MID,
        );
        
        // Bright chrome top edge (slightly inset for rounded look)
        crate::framebuffer::fill_rect(
            (menu_x + corner_r as i32) as u32, menu_y as u32,
            (menu_width - corner_r as i32 * 2) as u32, 1, CHROME_BRIGHT,
        );
        
        // Draw items
        for (idx, item) in self.context_menu.items.iter().enumerate() {
            let item_y = menu_y + padding + idx as i32 * item_height;
            
            let is_hovered = self.cursor_x >= menu_x && self.cursor_x < menu_x + menu_width
                && self.cursor_y >= item_y && self.cursor_y < item_y + item_height;
            
            if is_hovered && item.action != ContextAction::Cancel && !item.label.starts_with("─") {
                // Hover with rounded pill shape
                draw_rounded_rect(
                    menu_x + 4, item_y,
                    (menu_width - 8) as u32, (item_height - 2) as u32,
                    6, GREEN_GHOST,
                );
                draw_rounded_rect(
                    menu_x + 6, item_y + 1,
                    (menu_width - 12) as u32, (item_height - 4) as u32,
                    5, BG_LIGHT,
                );
                // Left accent bar (rounded)
                draw_rounded_rect(
                    menu_x + 4, item_y + 4,
                    2, (item_height - 10) as u32,
                    1, GREEN_PRIMARY,
                );
            }
            
            // Separator line or text
            if item.label.starts_with("─") {
                framebuffer::fill_rect(
                    (menu_x + 12) as u32, (item_y + item_height / 2) as u32,
                    (menu_width - 24) as u32, 1,
                    GREEN_GHOST
                );
            } else {
                let text_color = if is_hovered { GREEN_SECONDARY } else { GREEN_TERTIARY };
                self.draw_text(menu_x + 16, item_y + 6, &item.label, text_color);
            }
        }
    }

    /// Analyze audio directly from the HDA DMA buffer — source-agnostic.
    /// Any audio playing through HDA (music player, Game Boy, system sounds, etc.)
    /// will drive the matrix rain visualization.
    fn analyze_global_audio(&mut self) {
        // Lazy-init Vec buffers on first call
        if self.global_fft_re.len() < 256 {
            self.global_fft_re.resize(256, 0.0);
            self.global_fft_im.resize(256, 0.0);
        }
        if self.global_energy_hist.len() < 43 {
            self.global_energy_hist.resize(43, 0.0);
        }

        // Check if HDA is playing anything
        if !crate::drivers::hda::is_playing() {
            // Decay all values when nothing is playing
            self.global_sub_bass *= 0.92;
            self.global_bass *= 0.92;
            self.global_mid *= 0.92;
            self.global_treble *= 0.92;
            self.global_energy *= 0.92;
            self.global_beat *= 0.85;
            if self.global_energy < 0.001 {
                self.global_audio_active = false;
            }
            return;
        }

        // Get DMA buffer and playback position
        let dma_info = crate::drivers::hda::get_dma_buffer_info();
        let lpib = crate::drivers::hda::get_playback_position();
        
        let (buf_ptr, buf_cap) = match dma_info {
            Some((p, c)) if !p.is_null() && c > 512 => (p, c),
            _ => return,
        };

        // Read 256 mono samples from around the current playback position
        // LPIB is in bytes; convert to sample index (i16 = 2 bytes)
        let lpib_sample = (lpib as usize) / 2;
        // We want samples slightly behind LPIB (what's being heard now)
        let fft_n = 256usize;
        let read_start = if lpib_sample >= fft_n * 2 {
            lpib_sample - fft_n * 2
        } else {
            // Wrap around the circular DMA buffer
            buf_cap.saturating_sub(fft_n * 2 - lpib_sample)
        };

        let mut max_abs: f32 = 0.0;
        for i in 0..fft_n {
            let idx = (read_start + i * 2) % buf_cap; // stereo: skip every other (take left channel)
            let s = unsafe { *buf_ptr.add(idx) } as f32;
            self.global_fft_re[i] = s;
            self.global_fft_im[i] = 0.0;
            let a = if s >= 0.0 { s } else { -s };
            if a > max_abs { max_abs = a; }
        }

        // Check if there's actual signal (not just silence)
        if max_abs < 10.0 {
            self.global_audio_active = false;
            self.global_sub_bass *= 0.92;
            self.global_bass *= 0.92;
            self.global_mid *= 0.92;
            self.global_treble *= 0.92;
            self.global_energy *= 0.92;
            self.global_beat *= 0.85;
            return;
        }
        self.global_audio_active = true;

        // Auto-gain
        if max_abs > self.global_peak_rms {
            self.global_peak_rms += (max_abs - self.global_peak_rms) * 0.3;
        } else {
            self.global_peak_rms *= 0.9995;
        }
        let gain = if self.global_peak_rms > 100.0 { 16000.0 / self.global_peak_rms } else { 1.0 };

        // Hann window + normalize
        for i in 0..fft_n {
            let t = i as f32 / fft_n as f32;
            let hann = 0.5 * (1.0 - libm::cosf(2.0 * core::f32::consts::PI * t));
            self.global_fft_re[i] *= hann * gain / 32768.0;
        }

        // In-place radix-2 FFT (256-point)
        {
            let re = &mut self.global_fft_re[..fft_n];
            let im = &mut self.global_fft_im[..fft_n];
            // Bit-reversal
            let mut j = 0usize;
            for i in 0..fft_n {
                if i < j { re.swap(i, j); im.swap(i, j); }
                let mut m = fft_n >> 1;
                while m >= 1 && j >= m { j -= m; m >>= 1; }
                j += m;
            }
            // Butterfly
            let mut step = 2usize;
            while step <= fft_n {
                let half = step >> 1;
                let angle = -core::f32::consts::PI / half as f32;
                let (ws, wc) = (libm::sinf(angle), libm::cosf(angle));
                for k in (0..fft_n).step_by(step) {
                    let (mut wr, mut wi) = (1.0f32, 0.0f32);
                    for m in 0..half {
                        let ii = k + m;
                        let jj = ii + half;
                        let tr = wr * re[jj] - wi * im[jj];
                        let ti = wr * im[jj] + wi * re[jj];
                        re[jj] = re[ii] - tr; im[jj] = im[ii] - ti;
                        re[ii] += tr; im[ii] += ti;
                        let new_wr = wr * wc - wi * ws;
                        wi = wr * ws + wi * wc;
                        wr = new_wr;
                    }
                }
                step <<= 1;
            }
        }

        // Band energies (256-pt FFT at 48kHz: bin = index * 187.5 Hz)
        let mag = |re: &[f32], im: &[f32], lo: usize, hi: usize| -> f32 {
            let mut s = 0.0f32;
            for i in lo..hi.min(128) {
                s += libm::sqrtf(re[i] * re[i] + im[i] * im[i]);
            }
            s / (hi - lo).max(1) as f32
        };
        let raw_sub = mag(&self.global_fft_re, &self.global_fft_im, 1, 2);
        let raw_bass = mag(&self.global_fft_re, &self.global_fft_im, 2, 4);
        let raw_mid = mag(&self.global_fft_re, &self.global_fft_im, 4, 16);
        let raw_tre = mag(&self.global_fft_re, &self.global_fft_im, 16, 60);
        let raw_e = raw_sub * 1.5 + raw_bass * 1.2 + raw_mid * 0.5 + raw_tre * 0.2;

        // Smooth
        let sm = |prev: f32, new: f32, a: f32, r: f32| -> f32 {
            if new > prev { prev + (new - prev) * a } else { prev + (new - prev) * r }
        };
        self.global_sub_bass = sm(self.global_sub_bass, raw_sub.min(1.0), 0.75, 0.10);
        self.global_bass = sm(self.global_bass, raw_bass.min(1.0), 0.70, 0.10);
        self.global_mid = sm(self.global_mid, raw_mid.min(1.0), 0.60, 0.12);
        self.global_treble = sm(self.global_treble, raw_tre.min(1.0), 0.70, 0.16);
        self.global_energy = sm(self.global_energy, raw_e.min(1.5), 0.65, 0.10);

        // Beat detection
        let be = raw_sub + raw_bass * 0.8;
        self.global_energy_hist[self.global_hist_idx] = be;
        self.global_hist_idx = (self.global_hist_idx + 1) % 43;
        if self.global_hist_count < 43 { self.global_hist_count += 1; }
        let filled = self.global_hist_count.max(1) as f32;
        let avg: f32 = self.global_energy_hist.iter().take(self.global_hist_count).sum::<f32>() / filled;
        let mut var_sum = 0.0f32;
        for i in 0..self.global_hist_count {
            let d = self.global_energy_hist[i] - avg;
            var_sum += d * d;
        }
        let variance = var_sum / filled;
        let threshold = (-15.0 * variance + 1.45f32).max(1.05).min(1.5);
        let onset = be - self.global_prev_energy;
        if be > avg * threshold && onset > 0.002 && self.global_hist_count > 5 {
            let strength = ((be - avg * threshold) / avg.max(0.001)).min(1.0);
            self.global_beat = (0.6 + strength * 0.4).min(1.0);
        } else {
            self.global_beat *= 0.88;
            if self.global_beat < 0.02 { self.global_beat = 0.0; }
        }
        self.global_prev_energy = be;
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // MOBILE MODE: portrait UI within a centered viewport
    // Uses EXACT same style: matrix rain, chrome borders, glass panels
    // ═══════════════════════════════════════════════════════════════════
    fn draw_mobile_mode(&mut self) {
        // Calculate viewport (iPhone 16 aspect ratio centered on screen)
        let (vx, vy, vw, vh) = crate::mobile::calculate_viewport(self.width, self.height);
        self.mobile_state.vp_x = vx;
        self.mobile_state.vp_y = vy;
        self.mobile_state.vp_w = vw;
        self.mobile_state.vp_h = vh;

        // Clear entire screen to deep black
        framebuffer::clear_backbuffer(0xFF000000);

        // Draw matrix rain background (same as desktop, full screen)
        self.draw_background();

        // Darken area outside viewport for phone-frame effect
        if vx > 0 {
            framebuffer::fill_rect(0, 0, vx as u32, self.height, 0xFF020202);
            framebuffer::fill_rect((vx + vw as i32) as u32, 0, (self.width as i32 - vx - vw as i32).max(0) as u32, self.height, 0xFF020202);
        }

        // Phone chrome frame (same border style as desktop windows)
        crate::mobile::draw_phone_frame(vx, vy, vw, vh);

        // Tick animations
        crate::mobile::tick_animations(&mut self.mobile_state);

        // Update time string from RTC cache
        if self.frame_count % 60 == 0 || self.mobile_state.time_str.is_empty() {
            let dt = crate::rtc::read_rtc();
            use core::fmt::Write;
            self.mobile_state.time_str.clear();
            let _ = core::write!(self.mobile_state.time_str, "{:02}:{:02}", dt.hour, dt.minute);
        }

        let frame = self.mobile_state.anim_frame;
        let view = self.mobile_state.view;
        let time_str = self.mobile_state.time_str.clone();
        let hl = self.mobile_state.highlighted_icon;

        // Status bar (always visible)
        crate::mobile::draw_status_bar(vx, vy, vw, vh, &time_str, frame);

        match view {
            crate::mobile::MobileView::Home => {
                crate::mobile::draw_home_screen(vx, vy, vw, vh, hl, frame);
                // Music widget above dock
                let audio_info = crate::mobile::MobileAudioInfo {
                    playing: self.global_audio_active,
                    beat: self.global_beat,
                    energy: self.global_energy,
                    sub_bass: self.global_sub_bass,
                    bass: self.global_bass,
                    mid: self.global_mid,
                    treble: self.global_treble,
                    frame: self.frame_count,
                };
                crate::mobile::draw_mobile_music_widget(vx, vy, vw, vh, &audio_info,
                    self.mobile_state.music_dropdown_open,
                    self.mobile_state.music_viz_mode);
                crate::mobile::draw_dock(vx, vy, vw, vh, -1, frame);
                crate::mobile::draw_gesture_bar(vx, vy, vw, vh);
            }
            crate::mobile::MobileView::AppFullscreen => {
                // Draw app content area with real app UI
                let app_idx = self.mobile_state.active_app_id.unwrap_or(0);
                let app_name = if (app_idx as usize) < crate::mobile::app_count() {
                    crate::mobile::app_name(app_idx as usize)
                } else { "App" };
                crate::mobile::draw_app_bar(vx, vy, vw, app_name, frame);
                // Render actual app content
                let audio_info_app = crate::mobile::MobileAudioInfo {
                    playing: self.global_audio_active,
                    beat: self.global_beat,
                    energy: self.global_energy,
                    sub_bass: self.global_sub_bass,
                    bass: self.global_bass,
                    mid: self.global_mid,
                    treble: self.global_treble,
                    frame: self.frame_count,
                };
                crate::mobile::draw_mobile_app_content(
                    vx, vy, vw, vh,
                    app_idx, self.frame_count, &audio_info_app,
                    &self.mobile_state,
                );
                // Draw gesture bar at bottom
                crate::mobile::draw_gesture_bar(vx, vy, vw, vh);
            }
            crate::mobile::MobileView::AppSwitcher => {
                crate::mobile::draw_app_switcher(vx, vy, vw, vh, &[], 0, frame);
                crate::mobile::draw_gesture_bar(vx, vy, vw, vh);
            }
            crate::mobile::MobileView::ControlCenter => {
                crate::mobile::draw_home_screen(vx, vy, vw, vh, hl, frame);
                crate::mobile::draw_dock(vx, vy, vw, vh, -1, frame);
                crate::mobile::draw_control_center(vx, vy, vw, vh, self.mobile_state.cc_progress, frame);
                crate::mobile::draw_gesture_bar(vx, vy, vw, vh);
            }
        }

        // Draw cursor on top
        self.draw_cursor();
    }

    /// Apply a mobile gesture action
    fn apply_mobile_action(&mut self, action: crate::mobile::MobileAction) {
        use crate::mobile::MobileAction;
        match action {
            MobileAction::None => {}
            MobileAction::GoHome => {
                self.mobile_state.view = crate::mobile::MobileView::Home;
                self.mobile_state.active_app_id = None;
            }
            MobileAction::OpenSwitcher => {
                self.mobile_state.view = crate::mobile::MobileView::AppSwitcher;
            }
            MobileAction::OpenControlCenter => {
                self.mobile_state.view = crate::mobile::MobileView::ControlCenter;
                self.mobile_state.cc_progress = 1; // start animating
            }
            MobileAction::CloseControlCenter => {
                self.mobile_state.view = crate::mobile::MobileView::Home;
            }
            MobileAction::LaunchApp(idx) => {
                self.mobile_state.view = crate::mobile::MobileView::AppFullscreen;
                self.mobile_state.active_app_id = Some(idx as u32);
                crate::serial_println!("[Mobile] Launch app #{}", idx);
            }
            MobileAction::LaunchDockApp(slot) => {
                let idx = crate::mobile::dock_app_index(slot as usize);
                self.mobile_state.view = crate::mobile::MobileView::AppFullscreen;
                self.mobile_state.active_app_id = Some(idx as u32);
                crate::serial_println!("[Mobile] Launch dock app slot={} -> idx={}", slot, idx);
            }
            MobileAction::BackFromApp => {
                self.mobile_state.view = crate::mobile::MobileView::Home;
                self.mobile_state.active_app_id = None;
            }
            MobileAction::CloseSwitcherCard(id) => {
                self.mobile_state.closing_cards.push((id, 255));
            }
            MobileAction::MusicTogglePlay => {
                // Use a dedicated key for mobile music player
                const MOBILE_MP_KEY: u32 = 0xFFFF_FFFE;
                if !self.music_player_states.contains_key(&MOBILE_MP_KEY) {
                    self.music_player_states.insert(MOBILE_MP_KEY, MusicPlayerState::new());
                }
                if let Some(mp) = self.music_player_states.get_mut(&MOBILE_MP_KEY) {
                    match mp.state {
                        PlaybackState::Stopped => mp.play_track(0),
                        PlaybackState::Playing | PlaybackState::Paused => mp.toggle_pause(),
                    }
                }
            }
            MobileAction::MusicStop => {
                const MOBILE_MP_KEY: u32 = 0xFFFF_FFFE;
                if let Some(mp) = self.music_player_states.get_mut(&MOBILE_MP_KEY) {
                    mp.stop();
                }
            }
            MobileAction::MusicToggleDropdown => {
                self.mobile_state.music_dropdown_open = !self.mobile_state.music_dropdown_open;
            }
            MobileAction::MusicSetVizMode(mode) => {
                self.mobile_state.music_viz_mode = mode;
                self.mobile_state.music_dropdown_open = false;
                // Apply to the actual desktop visualizer
                self.visualizer.mode = mode;
                crate::serial_println!("[Mobile] Viz mode set to {} ({})", mode,
                    crate::visualizer::MODE_NAMES[mode as usize % crate::visualizer::NUM_MODES as usize]);
            }
            MobileAction::CalcButton(btn) => {
                // Full calculator logic
                let ms = &mut self.mobile_state;
                match btn {
                    16 => { // C (clear)
                        ms.calc_display.clear();
                        ms.calc_op = 0;
                        ms.calc_operand = 0;
                        ms.calc_fresh = false;
                    }
                    17 => { // +/- (negate)
                        if !ms.calc_display.is_empty() && ms.calc_display != "0" {
                            if ms.calc_display.starts_with('-') {
                                ms.calc_display.remove(0);
                            } else {
                                ms.calc_display.insert(0, '-');
                            }
                        }
                    }
                    18 => { // %
                        if let Ok(v) = ms.calc_display.parse::<i64>() {
                            let result = v / 100;
                            ms.calc_display.clear();
                            use core::fmt::Write;
                            let _ = core::write!(ms.calc_display, "{}", result);
                        }
                    }
                    10 => { // . (dot)
                        if ms.calc_fresh { ms.calc_display.clear(); ms.calc_fresh = false; }
                        if !ms.calc_display.contains('.') {
                            if ms.calc_display.is_empty() { ms.calc_display.push('0'); }
                            ms.calc_display.push('.');
                        }
                    }
                    15 => { // = (equals)
                        let current = ms.calc_display.parse::<i64>().unwrap_or(0);
                        let result = match ms.calc_op {
                            1 => ms.calc_operand + current,
                            2 => ms.calc_operand - current,
                            3 => ms.calc_operand * current,
                            4 => if current != 0 { ms.calc_operand / current } else { 0 },
                            _ => current,
                        };
                        ms.calc_display.clear();
                        use core::fmt::Write;
                        let _ = core::write!(ms.calc_display, "{}", result);
                        ms.calc_op = 0;
                        ms.calc_operand = 0;
                        ms.calc_fresh = true;
                    }
                    11 | 12 | 13 | 14 => { // +, -, *, /
                        let current = ms.calc_display.parse::<i64>().unwrap_or(0);
                        // Chain operations
                        if ms.calc_op > 0 && !ms.calc_fresh {
                            let result = match ms.calc_op {
                                1 => ms.calc_operand + current,
                                2 => ms.calc_operand - current,
                                3 => ms.calc_operand * current,
                                4 => if current != 0 { ms.calc_operand / current } else { 0 },
                                _ => current,
                            };
                            ms.calc_operand = result;
                            ms.calc_display.clear();
                            use core::fmt::Write;
                            let _ = core::write!(ms.calc_display, "{}", result);
                        } else {
                            ms.calc_operand = current;
                        }
                        ms.calc_op = btn - 10; // 1=+, 2=-, 3=*, 4=/
                        ms.calc_fresh = true;
                    }
                    0..=9 => { // Digits
                        if ms.calc_fresh {
                            ms.calc_display.clear();
                            ms.calc_fresh = false;
                        }
                        if ms.calc_display == "0" { ms.calc_display.clear(); }
                        if ms.calc_display.len() < 15 {
                            ms.calc_display.push((b'0' + btn) as char);
                        }
                    }
                    _ => {}
                }
                crate::serial_println!("[Mobile] Calc: display={}", ms.calc_display);
            }
            MobileAction::FilesTap(idx) => {
                let ms = &mut self.mobile_state;
                ms.files_selected = idx as i32;
                // If it's a directory (first 4 entries), go deeper
                if idx < 4 && ms.files_depth == 0 {
                    ms.files_depth = 1;
                    ms.files_selected = -1;
                }
                crate::serial_println!("[Mobile] Files: tap idx={} depth={}", idx, ms.files_depth);
            }
            MobileAction::FilesBack => {
                self.mobile_state.files_depth = self.mobile_state.files_depth.saturating_sub(1);
                self.mobile_state.files_selected = -1;
            }
            MobileAction::SettingsTap(idx) => {
                let ms = &mut self.mobile_state;
                ms.settings_selected = idx as i32;
                if (idx as usize) < ms.settings_toggles.len() {
                    ms.settings_toggles[idx as usize] = !ms.settings_toggles[idx as usize];
                }
                crate::serial_println!("[Mobile] Settings: toggled idx={}", idx);
            }
            MobileAction::GamesTap(idx) => {
                self.mobile_state.games_selected = idx as i32;
                crate::serial_println!("[Mobile] Games: selected idx={}", idx);
            }
            MobileAction::BrowserNav(page) => {
                self.mobile_state.browser_page = page;
                crate::serial_println!("[Mobile] Browser: page={}", page);
            }
            MobileAction::EditorTap(line) => {
                self.mobile_state.editor_cursor_line = line as u32;
            }
            MobileAction::EditorSwitchTab(tab) => {
                self.mobile_state.editor_tab = tab;
            }
            MobileAction::ChessTap(sq) => {
                let ms = &mut self.mobile_state;
                if ms.chess_selected == sq as i32 {
                    ms.chess_selected = -1; // Deselect
                } else if ms.chess_selected >= 0 {
                    // "Move" piece: toggle turn, deselect
                    ms.chess_turn = 1 - ms.chess_turn;
                    ms.chess_selected = -1;
                    crate::serial_println!("[Mobile] Chess: move to sq={}", sq);
                } else {
                    ms.chess_selected = sq as i32;
                }
            }
            MobileAction::MusicAppToggle => {
                const MOBILE_MP_KEY: u32 = 0xFFFF_FFFE;
                if !self.music_player_states.contains_key(&MOBILE_MP_KEY) {
                    self.music_player_states.insert(MOBILE_MP_KEY, MusicPlayerState::new());
                }
                if let Some(mp) = self.music_player_states.get_mut(&MOBILE_MP_KEY) {
                    match mp.state {
                        PlaybackState::Stopped => mp.play_track(0),
                        PlaybackState::Playing | PlaybackState::Paused => mp.toggle_pause(),
                    }
                }
            }
            MobileAction::TermSubmit => {
                let ms = &mut self.mobile_state;
                // Since there's no keyboard on mobile, cycle through demo commands
                let commands = ["help", "uname", "ls", "pwd", "whoami", "date", "free -h", "uptime"];
                let cmd_idx = ms.term_lines.len() / 2; // every 2 lines = 1 command+response
                let cmd = commands[cmd_idx % commands.len()];
                ms.term_lines.push(alloc::format!("$ {}", cmd));
                let response = match cmd {
                    "help" => "Available: help, ls, pwd, date, uname, whoami, free, uptime",
                    "ls" => "Documents  Downloads  Music  Pictures  config.toml",
                    "pwd" => "/home/user",
                    "uname" => "TrustOS 2.0 aarch64 #1 SMP",
                    "date" => "2026-03-05 12:00:00 UTC",
                    "whoami" => "user@trustos",
                    "free -h" => "  total: 8.0G  used: 2.1G  free: 5.9G",
                    "uptime" => "up 4h 23m, 1 user, load: 0.12",
                    _ => "command not found",
                };
                ms.term_lines.push(alloc::string::String::from(response));
                // Keep history manageable
                if ms.term_lines.len() > 40 {
                    ms.term_lines.drain(0..2);
                }
            }
        }
    }

    fn draw_background(&mut self) {
        // ═══════════════════════════════════════════════════════════════
        // MATRIX RAIN — Multi-color, beat-synced, depth-layered
        // Logo: faithful chrome/silver rendering from bitmap
        // Tier-adaptive: Standard=2 layers, Full=4 layers
        // ═══════════════════════════════════════════════════════════════
        const MATRIX_COLS: usize = 256;
        const MAX_TRAIL: usize = 40;
        let num_layers: usize = if self.desktop_tier >= DesktopTier::Full {
            4
        } else if self.desktop_tier >= DesktopTier::Standard {
            2
        } else {
            1  // Minimal: single layer — lightweight rain even on old CPUs
        };
        
        // ── Per-layer depth parameters (4 layers: far→near) ──
        // Layer 0 = FAR BG      (slow, dark, long trails)
        // Layer 1 = MID          (moderate, medium chars)
        // Layer 2 = NEAR         (faster, bright, vivid colors)
        // Layer 3 = FOREGROUND   (fastest, brightest, sparse)
        const LAYER_TRAIL: [usize; 4]  = [28, 20, 14, 8];
        const LAYER_DIM: [f32; 4]       = [0.28, 0.50, 0.78, 1.0];
        const LAYER_SPEED_PRESETS: [[f32; 4]; 3] = [
            [0.44, 0.88, 1.62, 2.25],
            [0.75, 1.50, 2.75, 3.75],
            [1.19, 2.38, 4.38, 6.25],
        ];
        let preset = (self.matrix_rain_preset as usize).min(2);
        let layer_speed: [f32; 4] = LAYER_SPEED_PRESETS[preset];
        const LAYER_SWAY: [f32; 4]      = [0.0, 0.3, 1.0, 2.0];
        const LAYER_COL_SKIP: [usize; 4] = [2, 3, 4, 6];
        const LAYER_ATMO_R: [i16; 4]    = [ 0,  0,  0,  0];
        const LAYER_ATMO_G: [i16; 4]    = [-4,  0,  4,  8];
        const LAYER_ATMO_B: [i16; 4]    = [ 0,  0,  0,  0];
        const LAYER_FLOW: [f32; 4]       = [0.0, 1.5, 3.5, 6.0];
        const LAYER_GLYPH_W: [u32; 4]    = [5, 7, 10, 14];
        const LAYER_GLYPH_H: [u32; 4]    = [10, 14, 20, 28];
        const MOBILE_GLYPH_W: [u32; 4]   = [3, 4, 6, 7];
        const MOBILE_GLYPH_H: [u32; 4]   = [6, 8, 12, 14];
        const MOBILE_COL_SKIP: [usize; 4] = [2, 3, 6, 8];
        let is_mobile = self.mobile_state.active;
        
        let height = self.height.saturating_sub(TASKBAR_HEIGHT);
        let width = self.width;
        
        // ── Analyze ALL audio from HDA DMA buffer (source-agnostic) ──
        self.analyze_global_audio();
        let m_beat = self.global_beat;
        let m_energy = self.global_energy;
        let m_sub_bass = self.global_sub_bass;
        let m_bass = self.global_bass;
        let m_mid = self.global_mid;
        let m_treble = self.global_treble;
        let m_playing = self.global_audio_active;
        
        // ── Beat counter for mode switching ──
        let beat_on = m_playing && m_beat > 0.5;
        if beat_on && !self.matrix_last_beat {
            self.matrix_beat_count = self.matrix_beat_count.wrapping_add(1);
        }
        self.matrix_last_beat = beat_on;
        // Void mode: activates every 8th beat → very subtle, preserves density
        let void_mode = m_playing && (self.matrix_beat_count % 8 == 7);
        
        // Pure black background with micro green tint
        framebuffer::fill_rect(0, 0, width, height, 0xFF010200);
        // Reflection zone: bottom 12% faint green glow (no blue)
        let refl_start = height * 88 / 100;
        if refl_start < height {
            framebuffer::fill_rect(0, refl_start, width, height - refl_start, 0xFF020300);
        }
        
        // ── Star field: sparse single white pixels ──
        // Deterministic grid with hash-based selection — ~0.15% of pixels twinkle
        // Stars are fixed positions but brightness gently oscillates per frame
        {
            let star_tick = self.frame_count as u32;
            let step = 12u32; // check every 12th pixel in both axes
            let mut sy = 0u32;
            while sy < height {
                let mut sx = 0u32;
                while sx < width {
                    // Fast integer hash to decide if this grid cell has a star
                    let h = (sx.wrapping_mul(2654435761)).wrapping_add(sy.wrapping_mul(340573321));
                    let h = h ^ (h >> 16);
                    if h % 97 == 0 {
                        // Sub-pixel offset within the cell (deterministic)
                        let ox = (h >> 8) % step;
                        let oy = (h >> 14) % step;
                        let px = sx + ox;
                        let py = sy + oy;
                        if px < width && py < height && py < refl_start {
                            // Twinkle: fast integer triangle wave (no sinf)
                            let phase = star_tick.wrapping_add(h & 0xFF).wrapping_mul(3);
                            let tri = ((phase & 255) as i32 - 128).unsigned_abs(); // 0..128
                            let lum = 40 + (tri * 60 / 128) as u32; // 40..100 brightness
                            let c = 0xFF000000 | (lum << 16) | (lum << 8) | lum;
                            framebuffer::put_pixel_fast(px, py, c);
                        }
                    }
                    sx += step;
                }
                sy += step;
            }
        }
        
        if !self.matrix_initialized {
            return;
        }
        
        // Desktop oscilloscope removed — waveform is shown inside the music player widget
        
        if self.frame_count < 5 { crate::serial_println!("[FRAME] #{} start", self.frame_count); }
        
        // ── Visualizer: update multi-mode 3D shape projection ──
        // Full tier only — wireframe deformation costs per-column trig
        if self.desktop_tier >= DesktopTier::Full {
            crate::visualizer::update(
                &mut self.visualizer,
                width, height,
                MATRIX_COLS,
                m_beat, m_energy,
                m_sub_bass, m_bass, m_mid, m_treble,
                m_playing,
            );
        }
        
        if self.frame_count < 5 { crate::serial_println!("[FRAME] #{} viz done", self.frame_count); }
        // ── Drone Swarm: advance choreography, project, render glow buffer ──
        // Full tier only — expensive per-drone physics + glow
        if self.desktop_tier >= DesktopTier::Full {
            crate::drone_swarm::update(&mut self.drone_swarm);
        }
        
        // Logo centered vertically in the available space
        let logo_center_y = height / 2;
        let logo_center_x = width / 2;
        // Flow field radius: strongest within ~250px of logo, fades to zero at ~500px
        let flow_radius = 300.0f32;
        let flow_fade = 250.0f32; // fade zone beyond radius
        
        // ── Render matrix rain with color variety ──
        let col_width = width / MATRIX_COLS as u32;
        let half_cols = MATRIX_COLS as f32 / 2.0;
        // Flow field phase: slow scrolling offset so the field evolves
        let flow_time = self.frame_count as f32 * 0.008;
        
        if self.frame_count < 5 { crate::serial_println!("[FRAME] #{} rain start", self.frame_count); }
        // Get raw framebuffer pointer once — eliminates 3 atomic loads per pixel
        let (fb_ptr, fb_stride, _fb_height) = framebuffer::frame_context();
        let has_any_overrides = !self.matrix_overrides.is_empty();
        for layer in 0..num_layers {
        // ── Parallax sway: horizontal drift based on frame time ──
        // Each layer oscillates at a different frequency for organic motion
        let sway_amp = LAYER_SWAY[layer];
        let sway_freq = match layer { 4 => 0.010f32, 5 => 0.014, 3 => 0.006, _ => 0.0 };
        let sway_offset = if sway_amp > 0.0 {
            let phase = (self.frame_count as f32) * sway_freq;
            // Use fast sin approx for non-repetitive motion
            let sin = crate::graphics::holomatrix::sin_approx_pub;
            let s1 = sin(phase);
            let s2 = sin(phase * 1.7 + 2.0) * 0.4;
            ((s1 + s2) * sway_amp) as i32
        } else { 0i32 };
        let col_skip = if is_mobile { MOBILE_COL_SKIP[layer] } else { LAYER_COL_SKIP[layer] };
        let atmo_r = LAYER_ATMO_R[layer];
        let atmo_g = LAYER_ATMO_G[layer];
        let atmo_b = LAYER_ATMO_B[layer];
        let flow_amp_base = LAYER_FLOW[layer];
        // FlowField visualizer mode (7): amplify flow field by 2.5×
        let flow_amp = if self.visualizer.mode == 7 { flow_amp_base * 2.5 } else { flow_amp_base };
        let glyph_w = if is_mobile { MOBILE_GLYPH_W[layer] } else { LAYER_GLYPH_W[layer] };
        let glyph_h = if is_mobile { MOBILE_GLYPH_H[layer] } else { LAYER_GLYPH_H[layer] };
        
        for col in 0..MATRIX_COLS.min(self.matrix_heads.len() / num_layers.max(1)) {
            // Column density: skip columns based on layer
            if col_skip > 1 && (col % col_skip) != 0 { continue; }
            
            let idx = col * num_layers.max(1) + layer;
            let speed = self.matrix_speeds[idx];
            let seed = self.matrix_seeds[idx];
            // Apply horizontal parallax sway to x position
            let base_x = (col as u32 * col_width) + col_width / 2;
            let x = (base_x as i32 + sway_offset).max(0).min(width as i32 - 1) as u32;
            
            // ── Per-layer depth parameters ──
            let layer_trail = LAYER_TRAIL[layer];
            let layer_dim: f32 = LAYER_DIM[layer];
            let layer_spd: f32 = layer_speed[layer];
            
            // ── FOV depth: columns at screen edges → more spaced, slower ──
            // fov_t: 0.0 at center, 1.0 at extreme edges (quadratic)
            let from_center = ((col as f32) - half_cols).abs() / half_cols;
            let fov_t = (from_center * from_center).min(1.0);
            // Char spacing: use glyph_h as base + minimal FOV expansion at edges
            let fov_extra = (fov_t * (glyph_h as f32 * 0.15)) as u32;
            let eff_char_h: u32 = glyph_h + fov_extra;
            // Speed reduction: edges ~12% slower (subtle convergence)
            let fov_speed_pct: i32 = (100.0 - fov_t * 12.0) as i32;
            // Minimal dim at periphery (preserve density at edges)
            let fov_dim: f32 = 1.0 - fov_t * 0.04;
            
            // ── Per-column frequency band assignment (seeded) ──
            // Same band for all layers in a column (use layer-0 seed)
            let freq_band = (self.matrix_seeds[col * num_layers.max(1)] >> 3) % 4;
            // Get the amplitude for this column's band (0.0 - 1.0+)
            let (band_amp, band_r_base, band_g_base, band_b_base) = if m_playing {
                match freq_band {
                    0 => (m_sub_bass,  0u8, 180u8, 0u8),   // Sub-bass → dark green
                    1 => (m_bass,      0u8, 200u8, 0u8),   // Bass → medium green
                    2 => (m_mid,       0u8, 220u8, 0u8),   // Mid → pure green
                    _ => (m_treble,    0u8, 210u8, 0u8),   // Treble → bright green
                }
            } else {
                (0.0, 0u8, 200u8, 0u8) // No music → pure green
            };
            // Intensity multiplier from band amplitude (0.3 = idle, up to 1.5 at max)
            let freq_intensity = if m_playing {
                (0.3 + band_amp * 1.2).min(1.5)
            } else { 1.0 };
            
            // ── Beat-synced void mode: suppress a few columns on 8th beat ──
            // Suppress only ~6% of columns in void mode (subtle pulse)
            let col_suppressed = void_mode && ((col.wrapping_mul(7) ^ self.matrix_beat_count as usize) % 16 == 0);
            
            // ── Music-reactive speed boost ──
            let beat_boost = if m_playing {
                let t = (col as u32 * 2) % (MATRIX_COLS as u32);
                let col_phase = if t < MATRIX_COLS as u32 {
                    t as f32 / MATRIX_COLS as f32
                } else {
                    2.0 - t as f32 / MATRIX_COLS as f32
                };
                let local_beat = m_beat * (0.5 + col_phase * 0.5);
                (local_beat * 6.0 + band_amp * 4.0) as i32
            } else { 0 };
            
            // Visualizer: slow rain in columns that intersect the 3D shape
            // Always active (idle mode shows shapes even without music)
            let ghost_slow = crate::visualizer::column_slow_factor(&self.visualizer, col) as i32;
            let speed_adj = ((speed as f32) * layer_spd) as i32;
            let effective_advance = (((speed_adj + beat_boost) * ghost_slow / 100) * fov_speed_pct / 100).max(1);
            let new_y = self.matrix_heads[idx] + effective_advance;
            if new_y > height as i32 + (layer_trail as i32 * eff_char_h as i32) {
                let new_seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                self.matrix_seeds[idx] = new_seed;
                self.matrix_heads[idx] = -((new_seed % (height / 3)) as i32);
                let chars: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
                for i in 0..layer_trail.min(MAX_TRAIL) {
                    let cs = new_seed.wrapping_add((i as u32).wrapping_mul(7919));
                    self.matrix_chars[idx * MAX_TRAIL + i] = chars[(cs as usize) % chars.len()];
                }
            } else {
                self.matrix_heads[idx] = new_y;
            }
            
            // Skip rendering for suppressed columns (position still advances)
            if col_suppressed { continue; }
            
            let head_y = self.matrix_heads[idx];
            
            // Speed-based brightness: faster drops = brighter
            let col_speed = (speed as f32) * layer_spd;
            let speed_norm = (col_speed / 5.0).min(1.0); // 0..1 normalized to max speed
            let energy_boost = if m_playing { m_energy * 0.3 } else { 0.0 };
            let beat_bright = if m_playing { m_beat * 0.15 } else { 0.0 };
            let brightness_mult = ((0.3 + speed_norm * 0.7 + energy_boost + beat_bright) * fov_dim * layer_dim).min(1.5);
            
            // Visualizer: columns inside shape allow dimmer trail chars through
            let ghost_col_inside = m_playing && col < self.visualizer.column_bounds.len() && {
                let (bmin, bmax) = self.visualizer.column_bounds[col];
                bmin >= 0 && bmax > bmin
            };
            
            // Per-column trail variation: each column gets a unique trail multiplier
            // so rain drops have different lengths (short splashes vs long streaks)
            let col_hash = ((col as u32).wrapping_mul(2654435761u32)) >> 20; // Knuth hash
            let col_trail_mult = 0.55 + (col_hash % 100) as f32 / 110.0; // 0.55..1.46
            let speed_trail = ((layer_trail as f32) * (0.5 + speed_norm * 0.5) * col_trail_mult) as usize;
            let eff_trail = speed_trail.max(4).min(MAX_TRAIL);
            
            for i in 0..eff_trail {
                // Far layer: render every other trail char (barely visible anyway)
                if layer == 0 && (i & 1) == 1 { continue; }
                
                let char_y = head_y - (i as i32 * eff_char_h as i32);
                if char_y < 0 || char_y >= height as i32 { continue; }
                
                // In void mode, skip every 5th character (very subtle)
                if void_mode && (i % 5 == 0) && i > 3 { continue; }
                
                // Trail fading — faster drops fade slower (longer visible trail)
                let trail_ext = if m_playing { (m_energy * 30.0) as u8 } else { 0 };
                let fade_rate = (200u8 as u16 / (eff_trail as u16).max(1)) as u8;
                // Speed bonus: fast drops sustain brightness longer
                let speed_sustain = (speed_norm * 30.0) as u8;
                let base = if i == 0 { 255u8 }
                    else if i == 1 { (230u8 + speed_sustain / 2).min(255).saturating_add(trail_ext / 2) }
                    else { (210u8 + trail_ext / 3 + speed_sustain / 3).saturating_sub((i as u8).saturating_mul(fade_rate.max(3))) };
                if base < (if ghost_col_inside { 2 } else { 3 }) { continue; }
                
                let brightness = ((base as f32) * brightness_mult).min(255.0) as u8;
                
                // ══ Speed-based green→white gradient coloring ══
                // Faster drops → brighter, whiter, longer visible trail
                // Each layer progressively reduces intensity
                let (r, g, b) = if i == 0 {
                    // HEAD: bright white-green (Matrix palette: white → green → black)
                    let white_mix = (0.50 + speed_norm * 0.45).min(0.95);
                    let band_mix = 1.0 - white_mix;
                    // White in Matrix = equal R/G/B, green-biased
                    let hr = ((band_r_base as f32 * band_mix + 180.0 * white_mix) * brightness_mult).min(190.0) as i16;
                    let hg = ((band_g_base as f32 * band_mix + 255.0 * white_mix) * brightness_mult).min(255.0) as i16;
                    let hb = ((180.0 * white_mix) * brightness_mult).min(190.0) as i16;
                    let beat_w = if m_playing { (m_beat * 8.0).min(15.0) as i16 } else { 0 };
                    // Ensure R never exceeds G (prevents yellow)
                    let fr = (hr + beat_w / 4 + atmo_r).max(0).min(190) as u8;
                    let fg = (hg + beat_w + atmo_g).max(0).min(255) as u8;
                    let fb = (hb + beat_w / 4 + atmo_b).max(0).min(190) as u8;
                    // Clamp: R must be ≤ G to prevent any yellow tint
                    let fr = fr.min(fg);
                    let fb = fb.min(fg);
                    (fr, fg, fb)
                } else {
                    // TRAIL: speed drives brightness floor + green purity
                    let fade = brightness as f32 / 255.0;
                    let fi = freq_intensity;
                    if self.visualizer.palette == 23 {
                        // Random palette: vivid random color per character
                        let (cr, cg, cb) = crate::visualizer::rain_random_color(
                            col, i, self.matrix_seeds[idx],
                        );
                        let fr = (cr as f32 * fade * fi).min(255.0) as u8;
                        let fg = (cg as f32 * fade * fi).min(255.0) as u8;
                        let fb = (cb as f32 * fade * fi).min(255.0) as u8;
                        (fr, fg, fb)
                    } else {
                        let speed_green = 0.8 + speed_norm * 0.4; // faster = greener
                        let tr = 0i16; // Zero red in trail — pure green only
                        let tg = ((band_g_base as f32 * fi * fade * speed_green).min(255.0)) as i16;
                        let tb = 0i16; // No blue in trail
                        // Atmospheric shift
                        let fr = 0u8; // Absolutely no red in trail
                        let fg = (tg + atmo_g).max(0).min(255) as u8;
                        let fb = 0u8; // Absolutely no blue in trail
                        (fr, fg, fb)
                    }
                };
                
                // ── Visualizer: modulate rain through invisible 3D shape ──
                let (mut r, mut g, mut b) = (r, g, b);
                let mut ghost_trail_boost: u8 = 0;
                let mut ghost_depth: u8 = 128;
                // Far layer 0: skip expensive collision — barely visible
                // Always active: shapes visible in idle mode too
                if layer >= 1 {
                    let fx = crate::visualizer::check_rain_collision(
                        &self.visualizer, col, char_y,
                        self.visualizer.beat_pulse, m_energy,
                    );
                    if fx.glow > 0 || fx.ripple > 0 || fx.fresnel > 0 || fx.specular > 0
                        || fx.scanline > 0 || fx.inner_glow > 0 || fx.shadow > 0 {
                        let (mr, mg, mb) = crate::visualizer::modulate_rain_color(
                            r, g, b, fx.glow, fx.depth, fx.ripple,
                            fx.fresnel, fx.specular,
                            fx.ao, fx.bloom, fx.scanline, fx.inner_glow, fx.shadow,
                            m_beat, m_energy,
                            m_sub_bass, m_bass, m_mid, m_treble,
                            self.visualizer.palette,
                        );
                        r = mr; g = mg; b = mb;
                        ghost_trail_boost = fx.trail_boost;
                        ghost_depth = fx.depth;
                    }
                    // Image mode: blend rain color toward image pixel color
                    if fx.target_blend > 0 {
                        let t = fx.target_blend as f32 / 255.0;
                        let inv = 1.0 - t;
                        r = (r as f32 * inv + fx.target_r as f32 * t) as u8;
                        g = (g as f32 * inv + fx.target_g as f32 * t) as u8;
                        b = (b as f32 * inv + fx.target_b as f32 * t) as u8;
                        ghost_trail_boost = fx.trail_boost;
                    }
                    // Contrast dim zone: darken rain just outside the shape
                    if fx.dim > 0 {
                        let keep = 1.0 - (fx.dim as f32 / 255.0);
                        r = (r as f32 * keep) as u8;
                        g = (g as f32 * keep) as u8;
                        b = (b as f32 * keep) as u8;
                    }
                }
                // Trail extension: boost brightness for trailing chars near shape
                if ghost_trail_boost > 0 {
                    let boost = 1.0 + ghost_trail_boost as f32 / 100.0;
                    r = (r as f32 * boost).min(255.0) as u8;
                    g = (g as f32 * boost).min(255.0) as u8;
                    b = (b as f32 * boost).min(255.0) as u8;
                }
                
                // ── Flow field: organic horizontal displacement ──
                // Radial: strongest near the TrustOS logo, fades to zero far away
                // Skip for far layers (0-2) — not noticeable at their dim brightness
                let flow_offset = if flow_amp > 0.0 && layer >= 3 {
                    let dx = x as f32 - logo_center_x as f32;
                    let dy = char_y as f32 - logo_center_y as f32;
                    let dist_sq = dx * dx + dy * dy;
                    // Radial falloff using squared distances (no sqrtf)
                    let r_sq = flow_radius * flow_radius;
                    let outer_sq = (flow_radius + flow_fade) * (flow_radius + flow_fade);
                    let radial = if dist_sq < r_sq {
                        1.0
                    } else if dist_sq < outer_sq {
                        // Linear approx in squared space
                        1.0 - (dist_sq - r_sq) / (outer_sq - r_sq)
                    } else {
                        0.0
                    };
                    if radial > 0.01 {
                        let cy = char_y as f32;
                        let cx = col as f32;
                        let sin = crate::graphics::holomatrix::sin_approx_pub;
                        let o1 = sin(cy * 0.0045 + cx * 0.13 + flow_time);
                        let o2 = sin(cy * 0.012 + cx * 0.07 + flow_time * 1.6 + 3.0) * 0.4;
                        let o3 = sin(cy * 0.028 + cx * 0.21 + flow_time * 2.3 + 1.5) * 0.15;
                        ((o1 + o2 + o3) * flow_amp * radial) as i32
                    } else { 0 }
                } else { 0 };
                let x_flow = (x as i32 + flow_offset).max(0).min(width as i32 - 1) as u32;
                
                // ── Drone Swarm: holographic wireframe formations ──
                // Skip for far layers (0-2) — subtle effect invisible at low brightness
                let drone_fx = if layer >= 3 {
                    crate::drone_swarm::query(
                        &self.drone_swarm, x_flow as f32, char_y as f32,
                    )
                } else {
                    crate::drone_swarm::DroneInteraction { brightness: 1.0, color_r: 0, color_g: 0, color_b: 0 }
                };
                if drone_fx.brightness != 1.0 || drone_fx.color_r != 0 {
                    let bf = drone_fx.brightness;
                    r = ((r as f32 * bf).min(255.0)) as u8;
                    g = ((g as f32 * bf).min(255.0)) as u8;
                    b = ((b as f32 * bf).min(255.0)) as u8;
                    r = ((r as i16 + drone_fx.color_r).max(0).min(255)) as u8;
                    g = ((g as i16 + drone_fx.color_g).max(0).min(255)) as u8;
                    b = ((b as i16 + drone_fx.color_b).max(0).min(255)) as u8;
                }
                
                let color = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                
                // Character selection: depth-of-field glyph selection
                // Front face → dense/heavy glyphs, back face → light/thin glyphs
                let inside_shape = ghost_trail_boost > 30;
                let mutation_speed = if inside_shape { 10u32 } else { 28u32 };
                let char_seed = seed.wrapping_add((i as u32 * 7919) ^ (self.frame_count as u32 / mutation_speed));
                let chars: &[u8] = if inside_shape {
                    if ghost_depth > 180 {
                        // Front face: heavy/dense glyphs (sharp focus)
                        b"@#$%&WM8BOXZNHK"
                    } else if ghost_depth < 80 {
                        // Back face: light/thin glyphs (out of focus)
                        b".:;~-'`"
                    } else {
                        // Mid-depth: medium weight glyphs
                        b"0123456789ABCDEF"
                    }
                } else {
                    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|"
                };
                let c = chars[(char_seed as usize) % chars.len()] as char;
                let glyph = crate::framebuffer::font::get_glyph(c);
                
                // ── Scaled glyph rendering + CRT scanline ──
                // Check for screen-space projection zone overlap
                let in_proj = self.matrix_projection.active
                    && x_flow + glyph_w > self.matrix_projection.x
                    && x_flow < self.matrix_projection.x + self.matrix_projection.width
                    && (char_y as u32) + glyph_h > self.matrix_projection.y
                    && (char_y as u32) < self.matrix_projection.y + self.matrix_projection.height;

                // Check for per-cell pixel override (skip if projection takes priority)
                let cell_key = idx * MAX_TRAIL + i;
                let has_override = !in_proj && has_any_overrides && self.matrix_overrides.contains_key(&cell_key);
                
                let cr = ((color >> 16) & 0xFF) as u8;
                let cg = ((color >> 8) & 0xFF) as u8;
                let cb = (color & 0xFF) as u8;
                
                if in_proj {
                    // ── PROJECTION MODE: reveal image pixels through rain ──
                    // Every pixel in the cell block is sampled from the projection
                    // image and modulated by rain intensity (brightness).
                    // Rain head = full brightness, trail = fading reveal.
                    let proj = &self.matrix_projection;
                    let intensity = brightness as f32 / 255.0;
                    for gr in 0..glyph_h as usize {
                        let py = char_y as u32 + gr as u32;
                        if py >= height { continue; }
                        if py < proj.y || py >= proj.y + proj.height { continue; }
                        let scanline: f32 = if py & 1 == 0 { 1.0 } else { 0.96 };
                        let in_reflection = py > height * 88 / 100;
                        let img_y = (py - proj.y) as usize;
                        for bit in 0..glyph_w {
                            let px = x_flow + bit;
                            if px >= width { continue; }
                            if px < proj.x || px >= proj.x + proj.width { continue; }
                            let img_x = (px - proj.x) as usize;
                            let pixel_color = proj.pixels[img_y * proj.width as usize + img_x];
                            if pixel_color & 0xFF000000 == 0 { continue; }
                            let pr = ((pixel_color >> 16) & 0xFF) as f32;
                            let pg = ((pixel_color >> 8) & 0xFF) as f32;
                            let pb = (pixel_color & 0xFF) as f32;
                            let mut fr = (pr * intensity).min(255.0) as u8;
                            let mut fg = (pg * intensity).min(255.0) as u8;
                            let mut fb = (pb * intensity).min(255.0) as u8;
                            fr = ((fr as f32 * scanline).min(255.0)) as u8;
                            fg = ((fg as f32 * scanline).min(255.0)) as u8;
                            fb = ((fb as f32 * scanline).min(255.0)) as u8;
                            if in_reflection {
                                fg = (fg as u16 + 10).min(255) as u8;
                            }
                            let fc = 0xFF000000 | ((fr as u32) << 16) | ((fg as u32) << 8) | (fb as u32);
                            framebuffer::put_pixel_fast(px, py, fc);
                        }
                    }
                } else if has_override {
                    // ── OVERRIDE MODE: render per-pixel from CellPixels ──
                    let cell_pixels = &self.matrix_overrides[&cell_key];
                    for gr in 0..16usize {
                        let py = char_y as u32 + gr as u32;
                        if py >= height { continue; }
                        let scanline: f32 = if py & 1 == 0 { 1.0 } else { 0.96 };
                        let in_reflection = py > height * 88 / 100;
                        for bit in 0..8u32 {
                            let pixel_color = cell_pixels.pixels[gr * 8 + bit as usize];
                            if pixel_color & 0xFF000000 == 0 { continue; } // transparent
                            let px = x_flow + bit;
                            if px >= width { continue; }
                            // Extract pixel's own RGB, then modulate by rain intensity
                            let pr = ((pixel_color >> 16) & 0xFF) as f32;
                            let pg = ((pixel_color >> 8) & 0xFF) as f32;
                            let pb = (pixel_color & 0xFF) as f32;
                            // Modulate: blend the override pixel toward the rain color/intensity
                            // intensity = brightness / 255 (how bright this trail position is)
                            let intensity = brightness as f32 / 255.0;
                            let mut fr = (pr * intensity).min(255.0) as u8;
                            let mut fg = (pg * intensity).min(255.0) as u8;
                            let mut fb = (pb * intensity).min(255.0) as u8;
                            if layer > 0 {
                                fg = (fg as u16 + 30u16).min(255) as u8;
                            }
                            fr = ((fr as f32 * scanline).min(255.0)) as u8;
                            fg = ((fg as f32 * scanline).min(255.0)) as u8;
                            fb = ((fb as f32 * scanline).min(255.0)) as u8;
                            if in_reflection {
                                fg = (fg as u16 + 10).min(255) as u8;
                            }
                            let fc = 0xFF000000 | ((fr as u32) << 16) | ((fg as u32) << 8) | (fb as u32);
                            framebuffer::put_pixel_fast(px, py, fc);
                        }
                    }
                } else {
                    // ── GLYPH RENDERING (all layers) — direct pointer writes ──
                    // Pre-compute colors for even/odd scanlines (avoid per-pixel float math)
                    let fg_boost = if layer > 0 { 30u16 } else { 0u16 };
                    let ar = cr;
                    let ag = (cg as u16 + fg_boost).min(255) as u8;
                    let ab = cb;
                    let color_even = 0xFF000000 | ((ar as u32) << 16) | ((ag as u32) << 8) | (ab as u32);
                    let sr = (ar as u16 * 245 >> 8) as u8;
                    let sg = (ag as u16 * 245 >> 8) as u8;
                    let sb = (ab as u16 * 245 >> 8) as u8;
                    let color_odd = 0xFF000000 | ((sr as u32) << 16) | ((sg as u32) << 8) | (sb as u32);
                    let refl_y = height * 88 / 100;
                    let rg_even = (ag as u16 + 10).min(255) as u8;
                    let rg_odd = (sg as u16 + 10).min(255) as u8;
                    let color_refl_even = 0xFF000000 | ((ar as u32) << 16) | ((rg_even as u32) << 8) | (ab as u32);
                    let color_refl_odd = 0xFF000000 | ((sr as u32) << 16) | ((rg_odd as u32) << 8) | (sb as u32);
                    let use_direct = !fb_ptr.is_null();
                    for sy in 0..glyph_h {
                        let py = char_y as u32 + sy;
                        if py >= height { continue; }
                        let src_row = ((sy * 16) / glyph_h).min(15) as usize;
                        let bits = glyph[src_row];
                        if bits == 0 { continue; }
                        let is_odd = py & 1 != 0;
                        let in_reflection = py > refl_y;
                        let fc = match (is_odd, in_reflection) {
                            (false, false) => color_even,
                            (true, false) => color_odd,
                            (false, true) => color_refl_even,
                            (true, true) => color_refl_odd,
                        };
                        if use_direct {
                            // Direct pointer write — zero atomic loads per pixel
                            let row_base = py as usize * fb_stride as usize;
                            for sx in 0..glyph_w {
                                let src_col = ((sx * 8) / glyph_w).min(7);
                                if bits & (0x80 >> src_col) != 0 {
                                    let px = x_flow + sx;
                                    if px < width {
                                        unsafe { *fb_ptr.add(row_base + px as usize) = fc; }
                                    }
                                }
                            }
                        } else {
                            for sx in 0..glyph_w {
                                let src_col = ((sx * 8) / glyph_w).min(7);
                                if bits & (0x80 >> src_col) != 0 {
                                    let px = x_flow + sx;
                                    if px < width {
                                        framebuffer::put_pixel_fast(px, py, fc);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } // for col
        } // for layer
        
        // ═════════════════════════════════════════════════════════════
        // GHOST FILL LAYERS — invisible rain that only renders inside
        // the 3D shape, filling gaps for a much denser silhouette
        // (skip on mobile — too heavy for small viewport)
        // ═════════════════════════════════════════════════════════════
        if !is_mobile {
            const NUM_FILL: usize = 4;
            const FILL_TRAIL: usize = 16;
            const FILL_CH: u32 = 16;  // Fill layers use standard 8×16 glyphs
            
            for col in 0..MATRIX_COLS.min(self.visualizer.column_bounds.len()) {
                // Skip columns with no shape at all (free)
                let (bmin, bmax) = self.visualizer.column_bounds[col];
                if bmin < 0 || bmax <= bmin { continue; }
                
                let x = (col as u32 * col_width) + col_width / 2;
                
                for fill in 0..NUM_FILL {
                    let fill_seed = (col as u32).wrapping_mul(2654435761)
                        ^ ((fill as u32 + 17).wrapping_mul(0x9E3779B9));
                    let fill_speed = 1 + (fill_seed % 3);
                    
                    // Virtual scrolling head: wraps within a range covering the screen
                    let total_h = (height + FILL_TRAIL as u32 * FILL_CH) as u32;
                    let raw_pos = (self.frame_count as u32)
                        .wrapping_mul(fill_speed)
                        .wrapping_add(fill_seed);
                    let virtual_head = (raw_pos % total_h.max(1)) as i32
                        - (FILL_TRAIL as i32 * FILL_CH as i32);
                    
                    for i in 0..FILL_TRAIL {
                        let char_y = virtual_head - (i as i32 * FILL_CH as i32);
                        if char_y < 0 || char_y >= height as i32 { continue; }
                        
                        // Quick bounds reject: only check collision near the shape
                        let margin = 12i32;
                        if char_y < bmin - margin || char_y > bmax + margin { continue; }
                        
                        let fx = crate::visualizer::check_rain_collision(
                            &self.visualizer, col, char_y,
                            self.visualizer.beat_pulse, m_energy,
                        );
                        // Only render if truly inside or on an edge — invisible otherwise
                        // (image mode uses target_blend instead of glow)
                        if fx.glow == 0 && fx.target_blend == 0 { continue; }
                        
                        // Trail fade for fill chars
                        let base = if i == 0 { 180u8 }
                            else { 120u8.saturating_sub((i as u8).saturating_mul(7)) };
                        if base < 10 { continue; }
                        
                        // Start from a dim base tinted toward the shape color
                        let dim_r = (base as u32 / 8) as u8;
                        let dim_g = (base as u32 / 3) as u8;
                        let dim_b = (base as u32 / 7) as u8;
                        let (mut mr, mut mg, mut mb) = crate::visualizer::modulate_rain_color(
                            dim_r, dim_g, dim_b,
                            fx.glow, fx.depth, fx.ripple,
                            fx.fresnel, fx.specular,
                            fx.ao, fx.bloom, fx.scanline, fx.inner_glow, fx.shadow,
                            m_beat, m_energy,
                            m_sub_bass, m_bass, m_mid, m_treble,
                            self.visualizer.palette,
                        );
                        // Image mode: blend fill chars toward image color too
                        if fx.target_blend > 0 {
                            let t = fx.target_blend as f32 / 255.0;
                            let inv = 1.0 - t;
                            mr = (mr as f32 * inv + fx.target_r as f32 * t) as u8;
                            mg = (mg as f32 * inv + fx.target_g as f32 * t) as u8;
                            mb = (mb as f32 * inv + fx.target_b as f32 * t) as u8;
                        }
                        
                        let color = 0xFF000000 | ((mr as u32) << 16) | ((mg as u32) << 8) | (mb as u32);
                        
                        // Dense glyphs for fill (shape-revealing characters)
                        let cs = fill_seed.wrapping_add(
                            (i as u32 * 7919) ^ (self.frame_count as u32 / 8)
                        );
                        let fill_chars: &[u8] = b"@#$%&WM8BOX0ZNHK";
                        let c = fill_chars[(cs as usize) % fill_chars.len()] as char;
                        let glyph = crate::framebuffer::font::get_glyph(c);
                        
                        for (gr, &bits) in glyph.iter().enumerate() {
                            let py = char_y as u32 + gr as u32;
                            if py >= height || bits == 0 { continue; }
                            if !fb_ptr.is_null() {
                                let row_base = py as usize * fb_stride as usize;
                                for bit in 0..8u32 {
                                    if bits & (0x80 >> bit) != 0 {
                                        let px = x + bit;
                                        if px < width {
                                            unsafe { *fb_ptr.add(row_base + px as usize) = color; }
                                        }
                                    }
                                }
                            } else {
                                for bit in 0..8u32 {
                                    if bits & (0x80 >> bit) != 0 {
                                        let px = x + bit;
                                        if px < width {
                                            framebuffer::put_pixel_fast(px, py, color);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if self.frame_count < 5 { crate::serial_println!("[FRAME] #{} rain+fill done", self.frame_count); }
        // Ghost mesh: no overlay drawing — rain IS the only renderer
        
        // ═══════════════════════════════════════════════════════════════
        if self.frame_count < 5 { crate::serial_println!("[FRAME] #{} logo start", self.frame_count); }
        // LOGO — Faithful chrome/silver rendering from bitmap
        // Skip on mobile — covers app grid
        // ═══════════════════════════════════════════════════════════════
        if !is_mobile {
            let logo_w = crate::logo_bitmap::LOGO_W as u32;
            let logo_h = crate::logo_bitmap::LOGO_H as u32;
            let logo_x = (width / 2).saturating_sub(logo_w / 2);
            let logo_y = logo_center_y.saturating_sub(logo_h / 2);
            
            // No vignette — rain behind logo provides sufficient contrast
            
            // Green glow behind chrome edges — render every 4th frame for performance
            if self.frame_count % 4 == 0 {
                for ly in (0..logo_h).step_by(2) {
                    for lx in 0..logo_w {
                        if !crate::logo_bitmap::logo_edge_pixel(lx as usize, ly as usize) { continue; }
                        let px = logo_x + lx;
                        let py = logo_y + ly;
                        if px >= width || py >= height { continue; }
                        let glow_g: u32 = if m_playing { 35 + (m_beat * 50.0) as u32 } else { 30 };
                        // Single pixel glow (skip 3×3)
                        framebuffer::put_pixel_fast(px, py, 0xFF000000 | (glow_g.min(255) << 8));
                    }
                }
            }
            
            // Draw actual logo pixels — chrome/silver faithfully, skip near-black
            for ly in 0..logo_h {
                for lx in 0..logo_w {
                    let argb = crate::logo_bitmap::logo_pixel(lx as usize, ly as usize);
                    let a = (argb >> 24) & 0xFF;
                    let r = (argb >> 16) & 0xFF;
                    let g = (argb >> 8) & 0xFF;
                    let b = argb & 0xFF;
                    
                    if a < 20 { continue; }
                    let luminance = (r * 77 + g * 150 + b * 29) >> 8;
                    if luminance < 30 { continue; }
                    
                    let px = logo_x + lx;
                    let py = logo_y + ly;
                    if px >= width || py >= height { continue; }
                    
                    if luminance >= 60 {
                        let beat_add = if m_playing { (m_beat * 20.0).min(30.0) as u32 } else { 0 };
                        let pr = (r + beat_add).min(255);
                        let pg = (g + beat_add).min(255);
                        let pb = (b + beat_add).min(255);
                        framebuffer::put_pixel_fast(px, py, 0xFF000000 | (pr << 16) | (pg << 8) | pb);
                    } else {
                        let alpha = ((luminance as u32) * 255 / 60).min(255);
                        let bg = framebuffer::get_pixel_fast(px, py);
                        let inv = 255 - alpha;
                        let nr = (r * alpha + ((bg >> 16) & 0xFF) * inv) / 255;
                        let ng = (g * alpha + ((bg >> 8) & 0xFF) * inv) / 255;
                        let nb = (b * alpha + (bg & 0xFF) * inv) / 255;
                        framebuffer::put_pixel_fast(px, py, 0xFF000000 | (nr << 16) | (ng << 8) | nb);
                    }
                }
            }
        }
        
        // (Beat flash overlay removed — was too aggressive)
    }
    
    /// Draw wallpaper from loaded image data
    fn draw_wallpaper_image(&self, wp_data: &crate::theme::WallpaperData, screen_height: u32) {
        use crate::theme::WallpaperMode;
        let mode = crate::theme::THEME.read().wallpaper.mode;
        
        match mode {
            WallpaperMode::Stretch => {
                // Bilinear interpolation for smooth scaling
                let wp_w = wp_data.width;
                let wp_h = wp_data.height;
                let scr_w = self.width;
                
                for sy in 0..screen_height {
                    // Fixed-point source coordinate (8-bit fractional)
                    let src_y_fp = (sy as u64 * ((wp_h as u64 - 1) << 8)) / screen_height as u64;
                    let y0 = (src_y_fp >> 8) as u32;
                    let y1 = (y0 + 1).min(wp_h - 1);
                    let fy = (src_y_fp & 0xFF) as u32; // 0..255 fractional part
                    let ify = 256 - fy;
                    
                    for sx in 0..scr_w {
                        let src_x_fp = (sx as u64 * ((wp_w as u64 - 1) << 8)) / scr_w as u64;
                        let x0 = (src_x_fp >> 8) as u32;
                        let x1 = (x0 + 1).min(wp_w - 1);
                        let fx = (src_x_fp & 0xFF) as u32;
                        let ifx = 256 - fx;
                        
                        // 4 neighboring pixels
                        let i00 = (y0 * wp_w + x0) as usize;
                        let i10 = (y0 * wp_w + x1) as usize;
                        let i01 = (y1 * wp_w + x0) as usize;
                        let i11 = (y1 * wp_w + x1) as usize;
                        
                        if i11 < wp_data.pixels.len() {
                            let c00 = wp_data.pixels[i00];
                            let c10 = wp_data.pixels[i10];
                            let c01 = wp_data.pixels[i01];
                            let c11 = wp_data.pixels[i11];
                            
                            // Bilinear blend each channel
                            let r = ( ((c00 >> 16) & 0xFF) * ifx * ify
                                    + ((c10 >> 16) & 0xFF) * fx * ify
                                    + ((c01 >> 16) & 0xFF) * ifx * fy
                                    + ((c11 >> 16) & 0xFF) * fx * fy ) >> 16;
                            let g = ( ((c00 >> 8) & 0xFF) * ifx * ify
                                    + ((c10 >> 8) & 0xFF) * fx * ify
                                    + ((c01 >> 8) & 0xFF) * ifx * fy
                                    + ((c11 >> 8) & 0xFF) * fx * fy ) >> 16;
                            let b = ( (c00 & 0xFF) * ifx * ify
                                    + (c10 & 0xFF) * fx * ify
                                    + (c01 & 0xFF) * ifx * fy
                                    + (c11 & 0xFF) * fx * fy ) >> 16;
                            
                            framebuffer::put_pixel_fast(sx, sy, 0xFF000000 | (r << 16) | (g << 8) | b);
                        }
                    }
                }
            }
            WallpaperMode::Center => {
                // Center the image
                let bg = crate::theme::colors().background;
                framebuffer::fill_rect(0, 0, self.width, screen_height, bg);
                
                let offset_x = self.width.saturating_sub(wp_data.width) / 2;
                let offset_y = screen_height.saturating_sub(wp_data.height) / 2;
                
                for y in 0..wp_data.height.min(screen_height) {
                    for x in 0..wp_data.width.min(self.width) {
                        let idx = (y * wp_data.width + x) as usize;
                        if idx < wp_data.pixels.len() {
                            framebuffer::put_pixel_fast(offset_x + x, offset_y + y, wp_data.pixels[idx]);
                        }
                    }
                }
            }
            WallpaperMode::Tile => {
                // Tile the image
                let mut dy = 0;
                while dy < screen_height {
                    let mut dx = 0;
                    while dx < self.width {
                        for y in 0..wp_data.height {
                            if dy + y >= screen_height { break; }
                            for x in 0..wp_data.width {
                                if dx + x >= self.width { break; }
                                let idx = (y * wp_data.width + x) as usize;
                                if idx < wp_data.pixels.len() {
                                    framebuffer::put_pixel_fast(dx + x, dy + y, wp_data.pixels[idx]);
                                }
                            }
                        }
                        dx += wp_data.width;
                    }
                    dy += wp_data.height;
                }
            }
            _ => {
                // Solid color fallback
                let color = crate::theme::THEME.read().wallpaper.fallback_color;
                framebuffer::fill_rect(0, 0, self.width, screen_height, color);
            }
        }
    }
    
    /// Draw TrustOS logo watermark — shield + padlock + circuit key
    /// Based on logo image: black/green shield, yellow-green padlock,
    /// circuit-board key extending downward with branch nodes, "TrustOS" text
    fn draw_logo_watermark(&self) {
        let center_x = self.width / 2;
        let center_y = (self.height - TASKBAR_HEIGHT) / 2 - 30;
        
        // Colors matching the logo image
        let green_bright = 0xFF50E050u32;  // Bright green
        let green_dark = 0xFF1A6B1Au32;    // Dark green for shield right half
        let black_fill = 0xFF080808u32;     // Near-black for shield left half
        let yellow_green = 0xFFC0E020u32;  // Yellow-green for padlock
        let circuit_green = 0xFF40C040u32; // Circuit line green
        let node_color = 0xFF60E060u32;     // Branch node dots
        let text_gray = 0xFF999999u32;      // "Trust" in gray
        let text_green = 0xFF40CC40u32;     // "OS" in green
        let outline_green = 0xFF30A030u32; // Shield outline
        
        // ── Shield shape: 80x100, split diagonally (left=black, right=green) ──
        let shield_w = 80u32;
        let shield_h = 100u32;
        let sx = center_x - shield_w / 2;
        let sy = center_y - shield_h / 2;
        
        for y in 0..shield_h {
            let ratio = y as f32 / shield_h as f32;
            let width_factor = if ratio < 0.45 {
                1.0
            } else {
                let t = (ratio - 0.45) / 0.55;
                (1.0 - t * t).max(0.0)
            };
            let w = (shield_w as f32 * width_factor).max(2.0) as u32;
            let x_off = (shield_w - w) / 2;
            
            for dx in 0..w {
                let px = sx + x_off + dx;
                let py = sy + y;
                // Diagonal split: left of center = black, right = dark green
                let local_x = x_off + dx;
                let diagonal = (local_x as f32 / shield_w as f32) + (ratio * 0.2);
                let fill = if diagonal < 0.5 { black_fill } else { green_dark };
                framebuffer::put_pixel_fast(px, py, fill);
            }
            
            // Shield outline (both edges)
            if w > 2 {
                framebuffer::put_pixel_fast(sx + x_off, sy + y, outline_green);
                framebuffer::put_pixel_fast(sx + x_off + w - 1, sy + y, outline_green);
            }
        }
        // Top edge
        framebuffer::draw_hline(sx, sy, shield_w, outline_green);
        
        // ── Padlock centered in upper shield ──
        let lock_cx = center_x;
        let lock_cy = sy + 30;
        // Shackle (rounded arch)
        for dy in 0..14u32 {
            for dx in 0..20u32 {
                let ddx = dx as i32 - 10;
                let ddy = dy as i32;
                let r_outer = 10i32;
                let r_inner = 6i32;
                if ddy <= r_outer && (ddx * ddx + (ddy - r_outer) * (ddy - r_outer)) <= r_outer * r_outer
                   && (ddx * ddx + (ddy - r_outer) * (ddy - r_outer)) >= r_inner * r_inner {
                    framebuffer::put_pixel_fast(lock_cx - 10 + dx, lock_cy - 14 + dy, yellow_green);
                }
            }
        }
        // Lock body (rectangle)
        framebuffer::fill_rect(lock_cx - 12, lock_cy, 24, 18, yellow_green);
        // Keyhole in lock body
        for dy in 0..6u32 {
            for dx in 0..6u32 {
                let ddx = dx as i32 - 3;
                let ddy = dy as i32 - 3;
                if ddx * ddx + ddy * ddy <= 9 {
                    framebuffer::put_pixel_fast(lock_cx - 3 + dx, lock_cy + 4 + dy, black_fill);
                }
            }
        }
        // Keyhole slot
        framebuffer::fill_rect(lock_cx - 1, lock_cy + 9, 3, 5, black_fill);
        
        // ── Circuit-board key extending downward from lock ──
        let key_start_y = lock_cy + 18;
        let key_end_y = sy + shield_h + 50;
        
        // Main vertical stem
        for ky in key_start_y..key_end_y {
            framebuffer::put_pixel_fast(lock_cx - 1, ky, circuit_green);
            framebuffer::put_pixel_fast(lock_cx, ky, circuit_green);
            framebuffer::put_pixel_fast(lock_cx + 1, ky, circuit_green);
        }
        
        // Branch connections at regular intervals
        let branches: &[(u32, i32, u32)] = &[
            (key_start_y + 8, -20, 6),   // Left branch
            (key_start_y + 8, 18, 6),    // Right branch
            (key_start_y + 22, -25, 5),  // Left branch 2
            (key_start_y + 22, 22, 5),   // Right branch 2
            (key_start_y + 36, -15, 4),  // Left branch 3
            (key_start_y + 36, 15, 4),   // Right branch 3
        ];
        
        for &(by, bx_off, node_r) in branches {
            if by >= self.height.saturating_sub(TASKBAR_HEIGHT) { continue; }
            // Draw branch line
            let sign: i32 = if bx_off < 0 { -1 } else { 1 };
            let abs_off = if bx_off < 0 { -bx_off } else { bx_off };
            for dx in 0..abs_off {
                let px = (lock_cx as i32 + sign * dx) as u32;
                if px < self.width {
                    framebuffer::put_pixel_fast(px, by, circuit_green);
                    framebuffer::put_pixel_fast(px, by + 1, circuit_green);
                }
            }
            // Node dot at end of branch
            let node_x = (lock_cx as i32 + bx_off) as u32;
            for ndy in 0..node_r {
                for ndx in 0..node_r {
                    let ddx = ndx as i32 - node_r as i32 / 2;
                    let ddy = ndy as i32 - node_r as i32 / 2;
                    if ddx * ddx + ddy * ddy <= (node_r as i32 / 2) * (node_r as i32 / 2) {
                        let px = node_x + ndx;
                        let py = by + ndy;
                        if px < self.width && py < self.height.saturating_sub(TASKBAR_HEIGHT) {
                            framebuffer::put_pixel_fast(px, py, node_color);
                        }
                    }
                }
            }
        }
        
        // Bottom terminator node (larger)
        if key_end_y + 4 < self.height.saturating_sub(TASKBAR_HEIGHT) {
            for dy in 0..8u32 {
                for dx in 0..8u32 {
                    let ddx = dx as i32 - 4;
                    let ddy = dy as i32 - 4;
                    if ddx * ddx + ddy * ddy <= 16 {
                        framebuffer::put_pixel_fast(lock_cx - 4 + dx, key_end_y + dy, node_color);
                    }
                }
            }
        }
        
    }
    
    fn draw_desktop_icons(&self) {
        // ═══════════════════════════════════════════════════════════════
        // LEFT DOCK SIDEBAR — Dark translucent panel with glow effects
        // Icons dynamically fill the full sidebar height
        // ═══════════════════════════════════════════════════════════════
        let dock_h = self.height.saturating_sub(TASKBAR_HEIGHT);
        
        // Frosted dark dock background — very dark with slight green tint
        // Draw column by column with opacity blending over matrix rain
        for dy in 0..dock_h {
            for dx in 0..(DOCK_WIDTH + 10) {
                let existing = framebuffer::get_pixel_fast(dx, dy);
                let er = ((existing >> 16) & 0xFF) as u32;
                let eg = ((existing >> 8) & 0xFF) as u32;
                let eb = (existing & 0xFF) as u32;
                // 75% opacity dark overlay: blend toward 0x040804
                let nr = (er * 25 / 100 + 4 * 75 / 100).min(255);
                let ng = (eg * 25 / 100 + 8 * 75 / 100).min(255);
                let nb = (eb * 25 / 100 + 4 * 75 / 100).min(255);
                framebuffer::put_pixel_fast(dx, dy, 0xFF000000 | (nr << 16) | (ng << 8) | nb);
            }
        }
        // Right edge: chrome separator
        framebuffer::fill_rect(DOCK_WIDTH + 9, 0, 1, dock_h, CHROME_GHOST);
        
        let icon_size = 36u32;
        let n_icons = self.icons.len().max(1) as u32;
        let padding = 12u32;
        let available = dock_h.saturating_sub(padding * 2);
        let icon_total = available / n_icons;
        let start_y = padding + (available - icon_total * n_icons) / 2;
        
        for (i, icon) in self.icons.iter().enumerate() {
            let ix = 12u32;
            let iy = start_y + (i as u32) * icon_total;
            if iy + icon_size > dock_h { break; }
            
            // Hit test
            let is_hovered = self.cursor_x >= 0 && self.cursor_x < (DOCK_WIDTH + 10) as i32
                && self.cursor_y >= iy as i32 && self.cursor_y < (iy + icon_total) as i32;
            
            // Darker muted colors normally, vivid glow on hover
            let icon_color = if is_hovered { GREEN_PRIMARY } else { GREEN_SUBTLE };
            let label_color = if is_hovered { GREEN_PRIMARY } else { 0xFF556655 };
            
            // Glow effect on hover: soft green glow around icon area
            if is_hovered {
                // Outer glow (soft spread)
                let glow_pad = 6u32;
                let gx = ix.saturating_sub(glow_pad);
                let gy = iy.saturating_sub(glow_pad);
                let gw = icon_size + glow_pad * 2;
                let gh = icon_size + 20 + glow_pad * 2;
                for gdy in 0..gh {
                    for gdx in 0..gw {
                        let px = gx + gdx;
                        let py = gy + gdy;
                        if px >= DOCK_WIDTH + 10 || py >= dock_h { continue; }
                        // Distance from edge of icon area for falloff
                        let inner_x = if gdx < glow_pad { glow_pad - gdx } 
                            else if gdx > gw - glow_pad { gdx - (gw - glow_pad) } 
                            else { 0 };
                        let inner_y = if gdy < glow_pad { glow_pad - gdy }
                            else if gdy > gh - glow_pad { gdy - (gh - glow_pad) }
                            else { 0 };
                        let dist = inner_x.max(inner_y);
                        if dist > 0 {
                            let intensity = (20u32.saturating_sub(dist * 4)).min(20) as u8;
                            if intensity > 0 {
                                let existing = framebuffer::get_pixel_fast(px, py);
                                let eg = ((existing >> 8) & 0xFF) as u8;
                                let new_g = eg.saturating_add(intensity);
                                let blended = (existing & 0xFFFF00FF) | ((new_g as u32) << 8);
                                framebuffer::put_pixel_fast(px, py, blended);
                            }
                        }
                    }
                }
                // Inner highlight (rounded to match icon shape)
                draw_rounded_rect((ix as i32) - 3, (iy as i32) - 2, icon_size + 6, icon_size + 16, 6, 0xFF001A0A);
                draw_rounded_rect_border((ix as i32) - 3, (iy as i32) - 2, icon_size + 6, icon_size + 16, 6, CHROME_MID);
            }
            
            // Icon background — rounded dark square with colored accent glow
            let accent_color = match icon.icon_type {
                IconType::Terminal => 0xFF20CC60u32,  // Bright green
                IconType::Folder => 0xFFDDAA30u32,    // Warm amber/gold
                IconType::Editor => 0xFF5090E0u32,    // Soft blue  
                IconType::Calculator => 0xFFCC6633u32, // Orange
                IconType::Network => 0xFF40AADDu32,    // Cyan
                IconType::Game => 0xFFCC4444u32,       // Red
                IconType::Chess => 0xFFEECC88u32,      // Gold/ivory
                IconType::Settings => 0xFF9988BBu32,   // Purple/lilac
                IconType::Browser => 0xFF4488DDu32,    // Blue
                IconType::GameBoy => 0xFF88BB44u32,    // Yellow-green
                _ => icon_color,
            };
            // Rounded icon background
            draw_rounded_rect(ix as i32, iy as i32, icon_size, icon_size, 6, 0xFF060A06);
            if is_hovered {
                // Colored accent border on hover
                draw_rounded_rect_border(ix as i32, iy as i32, icon_size, icon_size, 6, accent_color);
            } else {
                draw_rounded_rect_border(ix as i32, iy as i32, icon_size, icon_size, 6, CHROME_GHOST);
            }
            
            // Use accent color for hovered icons, muted version for normal
            let draw_color = if is_hovered { accent_color } else { icon_color };
            
            // Pixel-art icon inside square — v2 refined icons
            let cx = ix + icon_size / 2;
            let cy = iy + icon_size / 2;
            use crate::icons::IconType;
            match icon.icon_type {
                IconType::Terminal => {
                    // Terminal: monitor shape with prompt
                    // Outer screen body
                    draw_rounded_rect((cx - 15) as i32, (cy - 11) as i32, 30, 22, 3, draw_color);
                    // Dark inner screen
                    framebuffer::fill_rect(cx - 13, cy - 9, 26, 16, 0xFF050A05);
                    // Screen border highlight
                    framebuffer::draw_hline(cx - 13, cy - 9, 26, accent_color);
                    // Prompt: $ _
                    self.draw_text((cx - 9) as i32, (cy - 5) as i32, "$", 0xFF40FF60);
                    framebuffer::fill_rect(cx - 3, cy - 3, 8, 2, 0xFF40FF60);
                    // Stand/base
                    framebuffer::fill_rect(cx - 3, cy + 8, 6, 3, draw_color);
                    framebuffer::fill_rect(cx - 6, cy + 10, 12, 2, draw_color);
                },
                IconType::Folder => {
                    // Folder: detailed with tab, clasp, inner files
                    // Tab
                    draw_rounded_rect((cx - 14) as i32, (cy - 10) as i32, 14, 6, 2, draw_color);
                    // Main body
                    draw_rounded_rect((cx - 14) as i32, (cy - 5) as i32, 28, 18, 2, draw_color);
                    // Inner dark area
                    framebuffer::fill_rect(cx - 12, cy - 3, 24, 13, 0xFF0A0A06);
                    // Document lines peek out
                    framebuffer::fill_rect(cx - 8, cy, 14, 1, 0xFF404020);
                    framebuffer::fill_rect(cx - 8, cy + 3, 10, 1, 0xFF404020);
                    framebuffer::fill_rect(cx - 8, cy + 6, 16, 1, 0xFF404020);
                    // Clasp on front
                    framebuffer::fill_rect(cx - 2, cy - 5, 4, 2, accent_color);
                },
                IconType::Editor => {
                    // Code editor: document with syntax-colored lines
                    // Page body
                    draw_rounded_rect((cx - 11) as i32, (cy - 13) as i32, 22, 26, 2, draw_color);
                    // Dog-ear
                    framebuffer::fill_rect(cx + 5, cy - 13, 6, 6, 0xFF0A0A0A);
                    framebuffer::draw_hline(cx + 5, cy - 13, 1, draw_color);
                    framebuffer::draw_vline(cx + 5, cy - 13, 6, draw_color);
                    framebuffer::draw_hline(cx + 5, cy - 8, 6, draw_color);
                    // Dark code area
                    framebuffer::fill_rect(cx - 9, cy - 7, 18, 18, 0xFF080C08);
                    // Gutter line numbers
                    for row in 0..5u32 {
                        framebuffer::fill_rect(cx - 8, cy - 5 + row * 3, 2, 1, 0xFF335533);
                    }
                    // Syntax-colored code lines
                    framebuffer::fill_rect(cx - 4, cy - 5, 7, 1, 0xFF6688CC);  // blue keyword
                    framebuffer::fill_rect(cx - 4, cy - 2, 10, 1, draw_color);  // normal
                    framebuffer::fill_rect(cx - 4, cy + 1, 6, 1, 0xFFCC8844);  // orange string
                    framebuffer::fill_rect(cx - 4, cy + 4, 12, 1, draw_color);  // normal
                    framebuffer::fill_rect(cx - 4, cy + 7, 5, 1, 0xFF88BB44);  // green comment
                },
                IconType::Calculator => {
                    // Calculator: screen + button grid, more detailed
                    draw_rounded_rect((cx - 11) as i32, (cy - 13) as i32, 22, 26, 3, draw_color);
                    // Inner body
                    framebuffer::fill_rect(cx - 9, cy - 11, 18, 22, 0xFF0C0C0A);
                    // LED display
                    draw_rounded_rect((cx - 8) as i32, (cy - 10) as i32, 16, 7, 1, 0xFF1A3320);
                    self.draw_text((cx - 5) as i32, (cy - 10) as i32, "42", 0xFF40FF40);
                    // Buttons: 4×3 grid
                    for row in 0..3u32 {
                        for col in 0..4u32 {
                            let bx = cx - 8 + col * 5;
                            let by = cy - 0 + row * 4;
                            let btn_col = if col == 3 { accent_color } else { draw_color };
                            framebuffer::fill_rect(bx, by, 3, 2, btn_col);
                        }
                    }
                },
                IconType::Network => {
                    // Wi-Fi: proper arc shapes with dot
                    let arc_cx = cx as i32;
                    let arc_cy = (cy + 6) as i32;
                    // 3 arcs from outside in
                    for ring in 0..3u32 {
                        let r = 5 + ring * 4;
                        let r2 = (r * r) as i32;
                        let r2_inner = ((r.saturating_sub(2)) * (r.saturating_sub(2))) as i32;
                        for dy in -(r as i32)..=0 {
                            for dx in -(r as i32)..=(r as i32) {
                                let dist2 = dx * dx + dy * dy;
                                if dist2 <= r2 && dist2 >= r2_inner {
                                    let px = (arc_cx + dx) as u32;
                                    let py = (arc_cy + dy) as u32;
                                    if px >= ix && px < ix + icon_size && py >= iy && py < iy + icon_size {
                                        let color = if ring == 0 { 
                                            if is_hovered { accent_color } else { GREEN_GHOST }
                                        } else if ring == 1 { 
                                            if is_hovered { accent_color } else { GREEN_SUBTLE }
                                        } else { 
                                            draw_color 
                                        };
                                        framebuffer::put_pixel_fast(px, py, color);
                                    }
                                }
                            }
                        }
                    }
                    // Center dot
                    for dy in -1..=1i32 {
                        for dx in -1..=1i32 {
                            if dx*dx+dy*dy <= 1 {
                                framebuffer::put_pixel_fast((cx as i32 + dx) as u32, (arc_cy + dy) as u32, draw_color);
                            }
                        }
                    }
                },
                IconType::Game => {
                    // Gamepad: wider body with grips, d-pad and colored buttons
                    // Body
                    draw_rounded_rect((cx - 15) as i32, (cy - 6) as i32, 30, 16, 5, draw_color);
                    // Darker interior
                    framebuffer::fill_rect(cx - 13, cy - 4, 26, 12, 0xFF0A0A0A);
                    // Left grip
                    draw_rounded_rect((cx - 15) as i32, (cy - 2) as i32, 6, 10, 2, draw_color);
                    // Right grip
                    draw_rounded_rect((cx + 9) as i32, (cy - 2) as i32, 6, 10, 2, draw_color);
                    // D-pad (left side)
                    framebuffer::fill_rect(cx - 10, cy - 1, 7, 2, draw_color);
                    framebuffer::fill_rect(cx - 8, cy - 3, 2, 7, draw_color);
                    // Action buttons (right side) — ABXY style
                    framebuffer::fill_rect(cx + 4, cy - 3, 3, 3, 0xFF4488DD);  // blue (top)
                    framebuffer::fill_rect(cx + 8, cy - 1, 3, 3, ACCENT_RED);  // red (right)
                    framebuffer::fill_rect(cx + 4, cy + 1, 3, 3, 0xFF44DD44);  // green (bottom)
                    framebuffer::fill_rect(cx + 1, cy - 1, 3, 3, 0xFFDDDD44);  // yellow (left)
                },
                IconType::Chess => {
                    // Chess king piece — crown silhouette
                    let pc = if is_hovered { 0xFFFFDD88 } else { draw_color };
                    // Base
                    framebuffer::fill_rect(cx - 8, cy + 6, 16, 4, pc);
                    // Pedestal
                    framebuffer::fill_rect(cx - 6, cy + 2, 12, 4, pc);
                    // Body (narrower)
                    framebuffer::fill_rect(cx - 4, cy - 6, 8, 8, pc);
                    // Crown prongs
                    framebuffer::fill_rect(cx - 6, cy - 10, 3, 5, pc);
                    framebuffer::fill_rect(cx - 1, cy - 12, 2, 7, pc);
                    framebuffer::fill_rect(cx + 3, cy - 10, 3, 5, pc);
                    // Cross on top
                    framebuffer::fill_rect(cx - 1, cy - 14, 2, 4, pc);
                    framebuffer::fill_rect(cx - 2, cy - 13, 4, 2, pc);
                },
                IconType::Settings => {
                    // Gear: proper circle with teeth
                    for dy in 0..20u32 {
                        for dx in 0..20u32 {
                            let ddx = dx as i32 - 10;
                            let ddy = dy as i32 - 10;
                            let dist_sq = ddx * ddx + ddy * ddy;
                            // Outer ring (gear body)
                            if dist_sq >= 36 && dist_sq <= 72 {
                                framebuffer::put_pixel_fast(cx - 10 + dx, cy - 10 + dy, draw_color);
                            }
                            // Inner hole
                            if dist_sq <= 12 {
                                framebuffer::put_pixel_fast(cx - 10 + dx, cy - 10 + dy, draw_color);
                            }
                            // Inner ring border
                            if dist_sq >= 10 && dist_sq <= 16 {
                                framebuffer::put_pixel_fast(cx - 10 + dx, cy - 10 + dy, accent_color);
                            }
                        }
                    }
                    // 8 gear teeth (wider)
                    let teeth: &[(i32, i32)] = &[(0, -10), (0, 10), (-10, 0), (10, 0), (-7, -7), (7, -7), (-7, 7), (7, 7)];
                    for &(tx, ty) in teeth {
                        let px = (cx as i32 + tx) as u32;
                        let py = (cy as i32 + ty) as u32;
                        framebuffer::fill_rect(px.saturating_sub(2), py.saturating_sub(1), 4, 3, draw_color);
                    }
                },
                IconType::Browser => {
                    // Globe: circle with meridians and parallels
                    for dy in 0..22u32 {
                        for dx in 0..22u32 {
                            let ddx = dx as i32 - 11;
                            let ddy = dy as i32 - 11;
                            let dist_sq = ddx * ddx + ddy * ddy;
                            // Filled circle (globe body)
                            if dist_sq <= 110 {
                                framebuffer::put_pixel_fast(cx - 11 + dx, cy - 11 + dy, 0xFF0A1A2A);
                            }
                            // Outer ring
                            if dist_sq >= 100 && dist_sq <= 121 {
                                framebuffer::put_pixel_fast(cx - 11 + dx, cy - 11 + dy, draw_color);
                            }
                        }
                    }
                    // Equator
                    framebuffer::fill_rect(cx - 10, cy, 20, 1, draw_color);
                    // Vertical meridian
                    framebuffer::fill_rect(cx, cy - 10, 1, 20, draw_color);
                    // Curved meridian (ellipse)
                    for dy in 0..20u32 {
                        let ddy = dy as i32 - 10;
                        let val = 100 - ddy * ddy;
                        if val > 0 {
                            let curve_x = (fast_sqrt_i32(val) * 2 / 5) as u32;
                            if cx + curve_x < ix + icon_size {
                                framebuffer::put_pixel_fast(cx + curve_x, cy - 10 + dy, draw_color);
                            }
                            if cx >= curve_x + ix {
                                framebuffer::put_pixel_fast(cx.saturating_sub(curve_x), cy - 10 + dy, draw_color);
                            }
                        }
                    }
                    // Parallels
                    framebuffer::fill_rect(cx - 9, cy - 5, 18, 1, draw_color);
                    framebuffer::fill_rect(cx - 9, cy + 5, 18, 1, draw_color);
                },
                IconType::GameBoy => {
                    // Game Boy: handheld console shape
                    draw_rounded_rect((cx - 10) as i32, (cy - 13) as i32, 20, 26, 3, draw_color);
                    framebuffer::fill_rect(cx - 8, cy - 11, 16, 22, 0xFF1A1A1A);
                    // Green screen
                    draw_rounded_rect((cx - 7) as i32, (cy - 10) as i32, 14, 11, 1, 0xFF1A3320);
                    // Pixel character on screen
                    framebuffer::fill_rect(cx - 2, cy - 8, 4, 4, 0xFF40CC40);
                    framebuffer::fill_rect(cx - 3, cy - 4, 6, 2, 0xFF40CC40);
                    // D-pad
                    framebuffer::fill_rect(cx - 7, cy + 4, 5, 2, 0xFF333333);
                    framebuffer::fill_rect(cx - 5, cy + 2, 2, 6, 0xFF333333);
                    // A/B buttons
                    framebuffer::fill_rect(cx + 3, cy + 3, 3, 3, ACCENT_RED);
                    framebuffer::fill_rect(cx + 1, cy + 5, 3, 3, 0xFF4488DD);
                    // Speaker grilles (bottom)
                    for i in 0..3u32 {
                        framebuffer::fill_rect(cx + 2 + i * 3, cy + 10, 1, 2, 0xFF333333);
                    }
                },
                IconType::About => {
                    // Info circle: (i) icon
                    for dy in 0..20u32 {
                        for dx in 0..20u32 {
                            let ddx = dx as i32 - 10;
                            let ddy = dy as i32 - 10;
                            let dist_sq = ddx * ddx + ddy * ddy;
                            if dist_sq >= 72 && dist_sq <= 100 {
                                framebuffer::put_pixel_fast(cx - 10 + dx, cy - 10 + dy, draw_color);
                            }
                        }
                    }
                    // "i" letter
                    framebuffer::fill_rect(cx - 1, cy - 6, 2, 2, accent_color); // dot
                    framebuffer::fill_rect(cx - 1, cy - 2, 2, 8, accent_color); // body
                    framebuffer::fill_rect(cx - 3, cy + 5, 6, 1, accent_color); // serif
                },
                IconType::ModelEditor => {
                    // 3D cube: wireframe with colored faces
                    // Front face
                    let f_x = cx as i32 - 8;
                    let f_y = cy as i32 - 2;
                    framebuffer::fill_rect(f_x as u32, f_y as u32, 14, 12, 0xFF162016);
                    framebuffer::draw_rect(f_x as u32, f_y as u32, 14, 12, draw_color);
                    // Top face (parallelogram)
                    for i in 0..14i32 {
                        framebuffer::put_pixel_fast((f_x + i + 4) as u32, (f_y - 4) as u32, draw_color);
                        framebuffer::put_pixel_fast((f_x + i + 2) as u32, (f_y - 2) as u32, accent_color);
                    }
                    // Side edges
                    framebuffer::draw_vline((f_x + 17) as u32, (f_y - 4) as u32, 12, draw_color);
                    for j in 0..4u32 {
                        framebuffer::put_pixel_fast((f_x + 14 + j as i32) as u32, (f_y + j as i32 - 4) as u32, draw_color);
                    }
                },
                IconType::GameLab => {
                    // Flask/beaker shape
                    framebuffer::fill_rect(cx - 3, cy - 12, 6, 8, draw_color); // neck
                    framebuffer::fill_rect(cx - 5, cy - 12, 10, 2, draw_color); // rim
                    // Body (triangle shape)
                    for row in 0..10u32 {
                        let half_w = 3 + row;
                        framebuffer::fill_rect(cx.saturating_sub(half_w), cy - 4 + row, half_w * 2, 1, draw_color);
                    }
                    // Liquid inside
                    for row in 4..10u32 {
                        let half_w = row;
                        framebuffer::fill_rect(cx.saturating_sub(half_w) + 1, cy - 4 + row, (half_w * 2).saturating_sub(2), 1, accent_color);
                    }
                    // Bubbles
                    framebuffer::fill_rect(cx - 2, cy + 1, 2, 2, 0xFF80FF80);
                    framebuffer::fill_rect(cx + 1, cy + 3, 2, 2, 0xFF80FF80);
                },
                _ => {
                    // Generic: bordered square with inner diamond
                    draw_rounded_rect_border((cx - 10) as i32, (cy - 10) as i32, 20, 20, 3, draw_color);
                    for i in 0..6i32 {
                        framebuffer::put_pixel_fast((cx as i32 + i) as u32, (cy as i32 - i) as u32, accent_color);
                        framebuffer::put_pixel_fast((cx as i32 - i) as u32, (cy as i32 - i) as u32, accent_color);
                        framebuffer::put_pixel_fast((cx as i32 + i) as u32, (cy as i32 + i) as u32, accent_color);
                        framebuffer::put_pixel_fast((cx as i32 - i) as u32, (cy as i32 + i) as u32, accent_color);
                    }
                },
            }
            
            // Label under icon (anti-aliased)
            let name = &icon.name;
            let text_w = name.len() as u32 * 8;
            let text_x = ix + (icon_size / 2).saturating_sub(text_w / 2);
            self.draw_text_smooth(text_x as i32, (iy + icon_size + 2) as i32, name, label_color);
        }
    }
    
    fn draw_taskbar(&mut self) {
        let y = self.height - TASKBAR_HEIGHT;
        
        // ═══════════════════════════════════════════════════════════════
        // MODERN TASKBAR — v2: taller, glass morphism, larger icons
        // ═══════════════════════════════════════════════════════════════
        
        // Rounded translucent background with deeper glass effect
        {
            let radius = 6u32;
            let ri = radius as i32;
            let r2 = ri * ri;
            let w = self.width;
            
            // Top rows: rounded corners (same alpha as main body for uniform look)
            for row in 0..radius {
                let vert_dist = ri - row as i32;
                let horiz = fast_sqrt_i32(r2 - vert_dist * vert_dist) as u32;
                let left_indent = radius - horiz;
                let visible_w = w.saturating_sub(left_indent * 2);
                if visible_w > 0 {
                    framebuffer::fill_rect_alpha(left_indent, y + row, visible_w, 1, 0x040A06, 190);
                }
            }
            // Main body (uniform alpha with rounded top)
            framebuffer::fill_rect_alpha(0, y + radius, w, TASKBAR_HEIGHT - radius, 0x040A06, 190);
            // Green tint overlay
            framebuffer::fill_rect_alpha(0, y, w, TASKBAR_HEIGHT, 0x00AA44, 8);
            // Glass top highlight
            if w > radius * 2 {
                for px in radius..(w - radius) {
                    framebuffer::put_pixel_fast(px, y, CHROME_DIM);
                }
            }
            
            // Rounded top border
            for row in 0..radius {
                let vert_dist = ri - row as i32;
                let horiz = fast_sqrt_i32(r2 - vert_dist * vert_dist) as u32;
                let left_x = radius - horiz;
                let right_x = w - radius + horiz;
                if left_x < w {
                    framebuffer::put_pixel_fast(left_x, y + row, CHROME_DIM);
                }
                if right_x > 0 && right_x - 1 < w {
                    framebuffer::put_pixel_fast(right_x - 1, y + row, CHROME_DIM);
                }
            }
        }
        
        // ── TrustOS button (left) — pill shape with glow ──
        let start_hover = self.cursor_x >= 4 && self.cursor_x < 120 && self.cursor_y >= y as i32;
        if start_hover || self.start_menu_open {
            draw_rounded_rect(6, (y + 7) as i32, 110, 34, 10, 0xFF003318);
            framebuffer::fill_rect_alpha(6, y + 7, 110, 34, 0x00CC66, 60);
            // Glow effect
            framebuffer::fill_rect_alpha(4, y + 5, 114, 1, 0x00FF66, 25);
        }
        let border_color = if start_hover || self.start_menu_open { CHROME_BRIGHT } else { CHROME_GHOST };
        draw_rounded_rect_border(6, (y + 7) as i32, 110, 34, 10, border_color);
        let txt_color = if start_hover || self.start_menu_open { GREEN_PRIMARY } else { GREEN_SECONDARY };
        self.draw_text_smooth(20, (y + 15) as i32, "TrustOS", txt_color);
        // Subtle bold effect for TrustOS label
        if start_hover || self.start_menu_open {
            self.draw_text_smooth(21, (y + 15) as i32, "TrustOS", txt_color);
        }
        
        // ── Window buttons (centered) — taller pills ──
        let total_btns = self.windows.len();
        let btn_w = 96u32;
        let btn_h = 34u32;
        let btn_gap = 6u32;
        let total_w = if total_btns > 0 { total_btns as u32 * (btn_w + btn_gap) - btn_gap } else { 0 };
        let start_x = (self.width.saturating_sub(total_w)) / 2;
        
        for (i, w) in self.windows.iter().enumerate() {
            let btn_x = start_x + i as u32 * (btn_w + btn_gap);
            let btn_y = y + 7;
            
            let is_hover = self.cursor_x >= btn_x as i32 && self.cursor_x < (btn_x + btn_w) as i32
                && self.cursor_y >= y as i32;
            
            // Button background — rounded glass pill
            if w.focused {
                draw_rounded_rect(btn_x as i32, btn_y as i32, btn_w, btn_h, 8, 0xFF001A0A);
                framebuffer::fill_rect_alpha(btn_x, btn_y, btn_w, btn_h, 0x00AA44, 70);
                // Top glass highlight
                framebuffer::fill_rect_alpha(btn_x + 4, btn_y, btn_w - 8, 1, 0x00FF66, 35);
            } else if is_hover {
                draw_rounded_rect(btn_x as i32, btn_y as i32, btn_w, btn_h, 8, 0xFF000D05);
                framebuffer::fill_rect_alpha(btn_x, btn_y, btn_w, btn_h, 0x008833, 50);
            }
            // Border
            let bdr = if w.focused { CHROME_BRIGHT } else if is_hover { CHROME_MID } else { CHROME_GHOST };
            draw_rounded_rect_border(btn_x as i32, btn_y as i32, btn_w, btn_h, 8, bdr);
            
            // Window icon (2-char)
            let icon_str = match w.window_type {
                WindowType::Terminal => ">_",
                WindowType::FileManager => "[]",
                WindowType::Calculator => "##",
                WindowType::Browser => "WW",
                WindowType::TextEditor => "Tx",
                WindowType::Game => "Sk",
                WindowType::MusicPlayer => "Mu",
                _ => "::",
            };
            let icon_color = if w.focused { GREEN_PRIMARY } else { GREEN_GHOST };
            self.draw_text_smooth((btn_x + 8) as i32, (btn_y + 10) as i32, icon_str, icon_color);
            
            // Window title (truncated)
            let title_max = 7;
            let title: String = w.title.chars().take(title_max).collect();
            let text_color = if w.focused { GREEN_PRIMARY } else { GREEN_TERTIARY };
            self.draw_text_smooth((btn_x + 28) as i32, (btn_y + 10) as i32, &title, text_color);
            
            // Active indicator (green glow bar at bottom)
            if w.focused {
                let indicator_w = 60u32.min(btn_w - 14);
                let indicator_x = btn_x + (btn_w - indicator_w) / 2;
                draw_rounded_rect((indicator_x) as i32, (y + TASKBAR_HEIGHT - 5) as i32, indicator_w, 3, 1, GREEN_PRIMARY);
                framebuffer::fill_rect_alpha(indicator_x.saturating_sub(2), y + TASKBAR_HEIGHT - 7, indicator_w + 4, 2, GREEN_PRIMARY, 50);
            } else if !w.minimized {
                let dot_x = btn_x + btn_w / 2 - 2;
                framebuffer::fill_rect(dot_x, y + TASKBAR_HEIGHT - 4, 4, 2, GREEN_SUBTLE);
            }
        }
        
        // ── System tray (right side) — layout right-to-left with proper spacing ──
        // Layout anchors (right to left): ShowDesktop(8) | Gear(20) | Clock(64) | CPU/MEM(36) | FPS(~44) | A11y(~50) | WiFi/Vol/Bat(100)
        let tray_gap = 12u32; // gap between each tray element
        
        // Cursor starts after ShowDesktop button (8px + gap)
        let mut tray_cursor = self.width - 8 - 8 - tray_gap; // skip ShowDesktop zone
        
        // ── Settings gear icon ──
        let gear_w = 20u32;
        let gear_x = tray_cursor - gear_w;
        let gear_y = y + 16;
        tray_cursor = gear_x - tray_gap;
        
        // ── Clock + Date ──
        let clock_w = 64u32;
        let clock_x = tray_cursor - clock_w;
        let time = self.get_time_string();
        self.draw_text_smooth(clock_x as i32, (y + 10) as i32, &time, hc(GREEN_PRIMARY, 0xFFFFFFFF));
        // Bold effect for clock
        self.draw_text_smooth((clock_x + 1) as i32, (y + 10) as i32, &time, hc(GREEN_PRIMARY, 0xFFFFFFFF));
        let date = self.get_date_string();
        self.draw_text_smooth(clock_x as i32, (y + 27) as i32, &date, hc(GREEN_TERTIARY, 0xFFCCCCCC));
        tray_cursor = clock_x - tray_gap;
        
        // ── CPU + MEM mini-bars ──
        let bars_w = 36u32;
        let ind_x = tray_cursor - bars_w;
        let ind_y = y + 8;
        let cpu_level = ((self.frame_count % 7) + 2).min(6) as u32;
        self.draw_text(ind_x as i32, (ind_y + 2) as i32, "C", GREEN_GHOST);
        let bar_start_x = ind_x + 12;
        for seg in 0..8u32 {
            let seg_color = if seg < cpu_level {
                if cpu_level > 6 { ACCENT_RED } else { GREEN_PRIMARY }
            } else { GREEN_GHOST };
            framebuffer::fill_rect(bar_start_x + seg * 3, ind_y + 3, 2, 8, seg_color);
        }
        let mem_level = {
            let total = 16u32;
            let used = ((self.windows.len() as u32 * 2) + 4).min(total);
            (used * 8 / total).min(8)
        };
        self.draw_text(ind_x as i32, (ind_y + 17) as i32, "M", GREEN_GHOST);
        for seg in 0..8u32 {
            let seg_color = if seg < mem_level {
                if mem_level > 6 { ACCENT_AMBER } else { GREEN_PRIMARY }
            } else { GREEN_GHOST };
            framebuffer::fill_rect(bar_start_x + seg * 3, ind_y + 18, 2, 8, seg_color);
        }
        tray_cursor = ind_x - tray_gap;
        
        // ── FPS counter ──
        let fps_str = format!("{}fps", self.fps_current);
        let fps_color = if self.fps_current >= 55 { GREEN_SECONDARY } else if self.fps_current >= 30 { ACCENT_AMBER } else { ACCENT_RED };
        let fps_w = (fps_str.len() as u32) * 8 + 4;
        let fps_x = tray_cursor - fps_w;
        self.draw_text_smooth(fps_x as i32, (y + 17) as i32, &fps_str, fps_color);
        tray_cursor = fps_x - tray_gap;
        
        // ── Accessibility status indicators ──
        let a11y_str = crate::accessibility::status_indicators();
        if !a11y_str.is_empty() {
            let a11y_w = (a11y_str.len() as u32) * 8 + 4;
            let a11y_x = tray_cursor - a11y_w;
            self.draw_text_smooth(a11y_x as i32, (y + 17) as i32, &a11y_str, hc(ACCENT_AMBER, 0xFFFFFF00));
            tray_cursor = a11y_x - tray_gap;
        }
        
        // ── System tray indicators (WiFi, Volume, Battery) ──
        let tray_icons_w = 100u32;
        let tray_icons_x = tray_cursor - tray_icons_w;
        self.draw_sys_tray_indicators(tray_icons_x, y + 10);
        let gear_hover = self.cursor_x >= (gear_x as i32 - 4) && self.cursor_x < (gear_x as i32 + 20)
            && self.cursor_y >= y as i32;
        let gear_color = if gear_hover { GREEN_PRIMARY } else { GREEN_TERTIARY };
        if gear_hover {
            framebuffer::fill_rect_alpha(gear_x - 2, gear_y - 2, 20, 20, 0x00CC66, 30);
        }
        for dy in 0..16u32 {
            for dx in 0..16u32 {
                let ddx = dx as i32 - 8;
                let ddy = dy as i32 - 8;
                let dist_sq = ddx * ddx + ddy * ddy;
                if dist_sq >= 25 && dist_sq <= 56 {
                    framebuffer::put_pixel_fast(gear_x + dx, gear_y + dy, gear_color);
                }
                if dist_sq <= 6 {
                    framebuffer::put_pixel_fast(gear_x + dx, gear_y + dy, gear_color);
                }
            }
        }
        let teeth: &[(i32, i32)] = &[(0, -8), (0, 8), (-8, 0), (8, 0), (-6, -6), (6, -6), (-6, 6), (6, 6)];
        for &(tx, ty) in teeth {
            let px = (gear_x as i32 + 8 + tx) as u32;
            let py = (gear_y as i32 + 8 + ty) as u32;
            framebuffer::fill_rect(px.saturating_sub(1), py.saturating_sub(1), 3, 3, gear_color);
        }
        
        // ── Show Desktop button (far right) ──
        let sd_x = self.width - 8;
        let sd_w = 8u32;
        let sd_hover = self.cursor_x >= sd_x as i32 && self.cursor_y >= y as i32;
        let sd_color = if sd_hover { GREEN_MUTED } else { GREEN_GHOST };
        framebuffer::fill_rect(sd_x, y, sd_w, TASKBAR_HEIGHT, sd_color);
        framebuffer::fill_rect(sd_x, y + 6, 1, TASKBAR_HEIGHT - 12, GREEN_SUBTLE);
    }
    
    fn get_time_string(&mut self) -> String {
        // Cache RTC reads: only read once per ~60 frames (~1 second)
        // Avoids ~960 CMOS port I/O ops/sec that crash VirtualBox (VT-x VM exits)
        if self.frame_count - self.last_rtc_frame >= 60 || self.cached_time_str.is_empty() {
            let dt = crate::rtc::read_rtc();
            self.cached_time_str = format!("{:02}:{:02}", dt.hour, dt.minute);
            self.cached_date_str = format!("{:02}/{:02}", dt.month, dt.day);
            self.last_rtc_frame = self.frame_count;
        }
        self.cached_time_str.clone()
    }
    
    fn get_date_string(&self) -> String {
        self.cached_date_str.clone()
    }
    
    fn draw_start_menu(&self) {
        let menu_w = 480u32;
        let menu_h = 680u32;
        let menu_x = 4i32;
        let menu_y = (self.height - TASKBAR_HEIGHT - menu_h - 8) as i32;
        
        let is_hc = crate::accessibility::is_high_contrast();
        
        // ═══════════════════════════════════════════════════════════════
        // MODERN START MENU — v2: glass panel, icon grid, rounded search
        // ═══════════════════════════════════════════════════════════════
        
        // Frosted glass background (deeper alpha)
        if is_hc {
            framebuffer::fill_rect(menu_x as u32, menu_y as u32, menu_w, menu_h, 0xFF000000);
        } else {
            draw_rounded_rect(menu_x, menu_y, menu_w, menu_h, 14, 0xFF060A08);
            framebuffer::fill_rect_alpha(menu_x as u32, menu_y as u32, menu_w, menu_h, 0x060A08, 220);
        }
        
        // Single chrome border with rounded corners
        let border1 = hc(CHROME_MID, 0xFFFFFFFF);
        draw_rounded_rect_border(menu_x, menu_y, menu_w, menu_h, 14, border1);
        // Top highlight
        framebuffer::fill_rect_alpha((menu_x + 14) as u32, menu_y as u32, menu_w - 28, 1, 0x00FF66, 20);
        
        // Title bar
        if is_hc {
            framebuffer::fill_rect((menu_x + 2) as u32, (menu_y + 2) as u32, menu_w - 4, 28, 0xFF1A1A1A);
        } else {
            framebuffer::fill_rect_alpha((menu_x + 2) as u32, (menu_y + 2) as u32, menu_w - 4, 28, 0x002200, 180);
        }
        self.draw_text_smooth(menu_x + 14, menu_y + 8, "TrustOS Menu", hc(GREEN_PRIMARY, 0xFFFFFF00));
        self.draw_text_smooth(menu_x + 15, menu_y + 8, "TrustOS Menu", hc(GREEN_PRIMARY, 0xFFFFFF00)); // bold
        
        // Separator
        framebuffer::draw_hline((menu_x + 2) as u32, (menu_y + 30) as u32, menu_w - 4, GREEN_GHOST);
        
        // ── Rounded search bar ──
        let search_y = menu_y + 34;
        let search_h = 36u32;
        let search_pad = 12i32;
        let search_w = menu_w - search_pad as u32 * 2;
        draw_rounded_rect(menu_x + search_pad, search_y, search_w, search_h, 10, 0xFF0A120A);
        draw_rounded_rect_border(menu_x + search_pad, search_y, search_w, search_h, 10, GREEN_GHOST);
        // Glass highlight on search bar
        framebuffer::fill_rect_alpha((menu_x + search_pad + 4) as u32, search_y as u32, search_w - 8, 1, 0x00FF66, 15);
        
        // Search icon (magnifying glass)
        let mag_x = menu_x + search_pad + 12;
        let mag_y = search_y + 10;
        for dy in 0..10u32 {
            for dx in 0..10u32 {
                let ddx = dx as i32 - 5;
                let ddy = dy as i32 - 5;
                let dist = ddx * ddx + ddy * ddy;
                if dist >= 12 && dist <= 25 {
                    framebuffer::put_pixel_fast((mag_x + dx as i32) as u32, (mag_y + dy as i32) as u32, GREEN_TERTIARY);
                }
            }
        }
        framebuffer::fill_rect((mag_x + 8) as u32, (mag_y + 8) as u32, 4, 2, GREEN_TERTIARY);
        
        // Search text
        let search_text_x = menu_x + search_pad + 26;
        if self.start_menu_search.is_empty() {
            self.draw_text_smooth(search_text_x, search_y + 12, "Search apps...", GREEN_GHOST);
        } else {
            self.draw_text_smooth(search_text_x, search_y + 12, &self.start_menu_search, GREEN_PRIMARY);
            let cursor_x = search_text_x + (self.start_menu_search.len() as i32 * 8);
            if self.cursor_blink {
                framebuffer::fill_rect(cursor_x as u32, (search_y + 10) as u32, 2, 16, GREEN_PRIMARY);
            }
        }
        
        let items_start_y = search_y + search_h as i32 + 8;
        
        // ── App items — 2-column ICON GRID ──
        let items: [(&str, &str, bool); 18] = [
            (">_", "Terminal", false),
            ("[]", "Files", false),
            ("##", "Calculator", false),
            ("~~", "NetScan", false),
            ("Tx", "Text Editor", false),
            ("/\\", "TrustEdit 3D", false),
            ("WW", "Browser", false),
            ("C3", "Chess 3D", false),
            ("Kk", "Chess 2D", false),
            ("Sk", "Snake", false),
            ("NE", "NES Emulator", false),
            ("GB", "Game Boy", false),
            ("Lb", "TrustLab", false),
            ("Mu", "Music Player", false),
            ("@)", "Settings", false),
            ("<-", "Exit Desktop", true),
            ("!!", "Shutdown", true),
            (">>", "Reboot", true),
        ];
        
        // Filter items by search
        let search = self.start_menu_search.trim();
        let search_lower: alloc::string::String = search.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
        
        // ── Draw app grid (non-special items in 2 columns) ──
        let col_count = 2u32;
        let tile_w = (menu_w - 24) / col_count;
        let tile_h = 44u32;
        let tile_gap = 4u32;
        let mut drawn = 0usize;
        
        for (ii, (icon, label, is_special)) in items.iter().enumerate() {
            if *is_special { continue; }
            
            if !search_lower.is_empty() {
                let label_lower: alloc::string::String = label.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                if !label_lower.contains(search_lower.as_str()) {
                    continue;
                }
            }
            
            let col = (drawn % col_count as usize) as u32;
            let row = (drawn / col_count as usize) as u32;
            let item_x = menu_x + 10 + col as i32 * (tile_w + tile_gap) as i32;
            let item_y = items_start_y + (row as i32 * (tile_h + tile_gap) as i32);
            drawn += 1;
            
            // Don't overflow into power section
            if item_y + tile_h as i32 > menu_y + menu_h as i32 - 110 { break; }
            
            let is_hovered = self.cursor_x >= item_x 
                && self.cursor_x < item_x + tile_w as i32
                && self.cursor_y >= item_y 
                && self.cursor_y < item_y + tile_h as i32;
            let is_selected = self.start_menu_selected == ii as i32;
            
            // Tile background (rounded, with hover glow)
            if is_hovered || is_selected {
                draw_rounded_rect(item_x, item_y, tile_w, tile_h, 8, 0xFF0A2A14);
                framebuffer::fill_rect_alpha(item_x as u32, item_y as u32, tile_w, tile_h, 0x00AA44, if is_selected { 70 } else { 50 });
                draw_rounded_rect_border(item_x, item_y, tile_w, tile_h, 8, GREEN_GHOST);
            }
            
            // Icon circle background
            let icon_cx = item_x + 22;
            let icon_cy = item_y + tile_h as i32 / 2;
            let icon_r = 14i32;
            let icon_r2 = icon_r * icon_r;
            let icon_bg = if is_hovered || is_selected { 0xFF0A3A1A } else { 0xFF0C1810 };
            for dy in -icon_r..=icon_r {
                for dx in -icon_r..=icon_r {
                    if dx * dx + dy * dy <= icon_r2 {
                        framebuffer::put_pixel_fast((icon_cx + dx) as u32, (icon_cy + dy) as u32, icon_bg);
                    }
                }
            }
            // Icon border circle
            for dy in -icon_r..=icon_r {
                for dx in -icon_r..=icon_r {
                    let d2 = dx * dx + dy * dy;
                    if d2 >= (icon_r - 1) * (icon_r - 1) && d2 <= icon_r2 {
                        let bc = if is_hovered || is_selected { GREEN_MUTED } else { GREEN_GHOST };
                        framebuffer::put_pixel_fast((icon_cx + dx) as u32, (icon_cy + dy) as u32, bc);
                    }
                }
            }
            
            // Icon text (centered in circle)
            let icon_color = if is_hovered || is_selected { GREEN_PRIMARY } else { GREEN_SECONDARY };
            self.draw_text_smooth(icon_cx - 8, icon_cy - 6, icon, icon_color);
            
            // Label text (right of circle)
            let label_color = if is_hovered || is_selected { GREEN_PRIMARY } else { TEXT_PRIMARY };
            self.draw_text_smooth(item_x + 42, icon_cy - 6, label, label_color);
        }
        
        // "No results" when search yields nothing
        let power_items_visible = if search_lower.is_empty() { true } else {
            items[14..].iter().any(|(_, label, _)| {
                let ll: String = label.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                ll.contains(search_lower.as_str())
            })
        };
        if drawn == 0 && !power_items_visible && !search_lower.is_empty() {
            let no_y = items_start_y + 12;
            self.draw_text_smooth(menu_x + 40, no_y, "No results found", GREEN_GHOST);
        }
        
        // ── Power section (bottom-anchored) ──
        let power_y = menu_y + menu_h as i32 - 106;
        framebuffer::draw_hline((menu_x + 12) as u32, power_y as u32, menu_w - 24, GREEN_GHOST);
        
        let power_items: [(&str, &str, u8); 3] = [
            ("<-", "Exit Desktop", 14),
            ("!!", "Shutdown", 15),
            (">>", "Reboot", 16),
        ];
        
        for (pi, (icon, label, idx)) in power_items.iter().enumerate() {
            if !search_lower.is_empty() {
                let label_lower: String = label.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                if !label_lower.contains(search_lower.as_str()) {
                    continue;
                }
            }
            
            let item_y = power_y + 8 + (pi as i32 * 30);
            let item_h = 28u32;
            
            let is_hovered = self.cursor_x >= menu_x 
                && self.cursor_x < menu_x + menu_w as i32
                && self.cursor_y >= item_y 
                && self.cursor_y < item_y + item_h as i32;
            let is_selected = self.start_menu_selected == *idx as i32;
            
            if is_hovered || is_selected {
                draw_rounded_rect(menu_x + 8, item_y, menu_w - 16, item_h, 6, 0xFF1A0808);
                framebuffer::fill_rect_alpha((menu_x + 8) as u32, item_y as u32, menu_w - 16, item_h, 0xAA2222, if is_selected { 50 } else { 35 });
            }
            
            let icon_color = if is_hovered || is_selected { ACCENT_RED } else { 0xFF994444 };
            self.draw_text_smooth(menu_x + 18, item_y + 8, icon, icon_color);
            
            let label_color = if is_hovered || is_selected { ACCENT_RED } else { 0xFFAA4444 };
            self.draw_text_smooth(menu_x + 44, item_y + 8, label, label_color);
        }
        
        // Bottom: version info
        let ver_y = menu_y + menu_h as i32 - 22;
        framebuffer::draw_hline((menu_x + 8) as u32, ver_y as u32, menu_w - 16, GREEN_GHOST);
        self.draw_text(menu_x + 14, ver_y + 6, "TrustOS v0.4.2", GREEN_SUBTLE);
    }
    
    fn draw_window(&self, window: &Window) {
        let x = window.x;
        let y = window.y;
        let w = window.width;
        let h = window.height;
        
        // ═══════════════════════════════════════════════════════════════
        // CLASSIC WINDOW — Thick borders, transparency, right-side buttons
        // ═══════════════════════════════════════════════════════════════
        
        let corner_radius = if window.maximized { 0u32 } else { WINDOW_BORDER_RADIUS };
        
        // Drop shadow (5-layer)
        if !window.maximized && w > 4 && h > 4 {
            framebuffer::fill_rect_alpha((x + 10) as u32, (y + 10) as u32, w + 2, h + 2, 0x000000, 14);
            framebuffer::fill_rect_alpha((x + 7) as u32, (y + 7) as u32, w + 2, h + 2, 0x000000, 18);
            framebuffer::fill_rect_alpha((x + 5) as u32, (y + 5) as u32, w, h, 0x000000, 22);
            framebuffer::fill_rect_alpha((x + 3) as u32, (y + 3) as u32, w, h, 0x000000, 16);
            framebuffer::fill_rect_alpha((x + 1) as u32, (y + 1) as u32, w + 2, h + 2, 0x000000, 8);
            if window.focused {
                // Green edge glow for focused windows
                framebuffer::fill_rect_alpha((x - 1) as u32, (y - 1) as u32, w + 2, h + 2, 0x00FF66, 10);
            }
        }
        
        // Window background with slight transparency (alpha blend over desktop)
        if corner_radius > 0 {
            draw_rounded_rect(x, y, w, h, corner_radius, 0xFF0A0A0A);
            // Transparency overlay — blend a semi-transparent layer
            framebuffer::fill_rect_alpha(x as u32, y as u32, w, h, 0x0A0E0A, 235);
        } else {
            framebuffer::fill_rect(x as u32, y as u32, w, h, 0xFF0A0A0A);
            framebuffer::fill_rect_alpha(x as u32, y as u32, w, h, 0x0A0E0A, 235);
        }
        
        // Thick border (4px — bold modern look)
        let border_color = if window.focused {
            hc(CHROME_MID, 0xFFFFFFFF)
        } else {
            hc(CHROME_GHOST, 0xFF888888)
        };
        let bright_border = if window.focused { GREEN_GHOST } else { CHROME_GHOST };
        if corner_radius > 0 {
            // 4-layer border for thickness
            draw_rounded_rect_border(x, y, w, h, corner_radius, border_color);
            draw_rounded_rect_border(x + 1, y + 1, w.saturating_sub(2), h.saturating_sub(2), corner_radius.saturating_sub(1), bright_border);
            draw_rounded_rect_border(x + 2, y + 2, w.saturating_sub(4), h.saturating_sub(4), corner_radius.saturating_sub(2), border_color);
            draw_rounded_rect_border(x + 3, y + 3, w.saturating_sub(6), h.saturating_sub(6), corner_radius.saturating_sub(3), bright_border);
        } else {
            framebuffer::draw_rect(x as u32, y as u32, w, h, border_color);
            framebuffer::draw_rect((x + 1) as u32, (y + 1) as u32, w.saturating_sub(2), h.saturating_sub(2), bright_border);
            framebuffer::draw_rect((x + 2) as u32, (y + 2) as u32, w.saturating_sub(4), h.saturating_sub(4), border_color);
            framebuffer::draw_rect((x + 3) as u32, (y + 3) as u32, w.saturating_sub(6), h.saturating_sub(6), bright_border);
        }
        
        // Visual resize edge indicators (glow strips when hovering)
        if window.focused && !window.maximized && w > 20 && h > 20 {
            let edge = window.on_resize_edge(self.cursor_x, self.cursor_y);
            let glow_color = 0x00FF66u32;
            let glow_alpha = 40u32;
            let gt = 4u32;
            let gh = if h > 4 { h - 4 } else { 1 };
            let gw = if w > 4 { w - 4 } else { 1 };
            match edge {
                ResizeEdge::Left | ResizeEdge::TopLeft | ResizeEdge::BottomLeft => {
                    framebuffer::fill_rect_alpha(x as u32, (y + 2) as u32, gt, gh, glow_color, glow_alpha);
                }
                _ => {}
            }
            match edge {
                ResizeEdge::Right | ResizeEdge::TopRight | ResizeEdge::BottomRight => {
                    framebuffer::fill_rect_alpha((x + w as i32 - gt as i32) as u32, (y + 2) as u32, gt, gh, glow_color, glow_alpha);
                }
                _ => {}
            }
            match edge {
                ResizeEdge::Top | ResizeEdge::TopLeft | ResizeEdge::TopRight => {
                    framebuffer::fill_rect_alpha((x + 2) as u32, y as u32, gw, gt, glow_color, glow_alpha);
                }
                _ => {}
            }
            match edge {
                ResizeEdge::Bottom | ResizeEdge::BottomLeft | ResizeEdge::BottomRight => {
                    framebuffer::fill_rect_alpha((x + 2) as u32, (y + h as i32 - gt as i32) as u32, gw, gt, glow_color, glow_alpha);
                }
                _ => {}
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // TITLE BAR — Glass gradient with transparency
        // ═══════════════════════════════════════════════════════════════
        let titlebar_h = TITLE_BAR_HEIGHT;
        let tb_x = (x + 3) as u32;
        let tb_w = w.saturating_sub(6);
        if window.focused {
            framebuffer::fill_rect_alpha(tb_x, (y + 3) as u32, tb_w, titlebar_h - 3, 0x0E2210, 220);
            // Top glass highlight
            framebuffer::fill_rect_alpha(tb_x, (y + 3) as u32, tb_w, 1, 0x00FF66, 30);
            framebuffer::fill_rect_alpha(tb_x, (y + 4) as u32, tb_w, 1, 0x00CC55, 15);
        } else {
            framebuffer::fill_rect_alpha(tb_x, (y + 3) as u32, tb_w, titlebar_h - 3, 0x080C08, 200);
        }
        
        // Title bar bottom separator
        framebuffer::draw_hline((x + 3) as u32, (y + titlebar_h as i32) as u32, w.saturating_sub(6), 
            if window.focused { GREEN_GHOST } else { CHROME_GHOST });

        // ═══════════════════════════════════════════════════════════════
        // Windows-style BUTTONS (right side: minimize / maximize / close)
        // ═══════════════════════════════════════════════════════════════
        let btn_w = 28u32;
        let btn_h = titlebar_h - 4;
        let btn_y = (y + 3) as u32;
        let mx = self.cursor_x;
        let my = self.cursor_y;
        
        // Close button (rightmost, red)
        let close_x = x + w as i32 - btn_w as i32 - 3;
        let close_hover = mx >= close_x && mx < close_x + btn_w as i32 
            && my >= btn_y as i32 && my < btn_y as i32 + btn_h as i32;
        let close_bg = if close_hover { 0xFFCC3333 } else if window.focused { 0xFF2A1414 } else { 0xFF1A1A1A };
        framebuffer::fill_rect(close_x as u32, btn_y, btn_w, btn_h, close_bg);
        // X icon
        let cx_c = close_x + btn_w as i32 / 2;
        let cy_c = btn_y as i32 + btn_h as i32 / 2;
        let x_color = if close_hover { 0xFFFFFFFF } else if window.focused { 0xFFCC4444 } else { 0xFF666666 };
        for i in -3..=3i32 {
            framebuffer::put_pixel_fast((cx_c + i) as u32, (cy_c + i) as u32, x_color);
            framebuffer::put_pixel_fast((cx_c + i) as u32, (cy_c - i) as u32, x_color);
            // Bold: offset by 1 pixel
            framebuffer::put_pixel_fast((cx_c + i + 1) as u32, (cy_c + i) as u32, x_color);
            framebuffer::put_pixel_fast((cx_c + i + 1) as u32, (cy_c - i) as u32, x_color);
        }
        
        // Maximize button (second from right, green)
        let max_x = close_x - btn_w as i32;
        let max_hover = mx >= max_x && mx < max_x + btn_w as i32 
            && my >= btn_y as i32 && my < btn_y as i32 + btn_h as i32;
        let max_bg = if max_hover { 0xFF1A3A20 } else { 0xFF0E0E0E };
        framebuffer::fill_rect(max_x as u32, btn_y, btn_w, btn_h, max_bg);
        let cx_m = max_x + btn_w as i32 / 2;
        let cy_m = btn_y as i32 + btn_h as i32 / 2;
        let m_color = if max_hover { 0xFF44DD66 } else if window.focused { 0xFF227744 } else { 0xFF555555 };
        if window.maximized {
            // Overlapping squares (restore icon)
            for i in -2..=1i32 {
                framebuffer::put_pixel_fast((cx_m + i + 1) as u32, (cy_m - 3) as u32, m_color);
                framebuffer::put_pixel_fast((cx_m + 3) as u32, (cy_m + i - 1) as u32, m_color);
            }
            for i in -2..=2i32 {
                framebuffer::put_pixel_fast((cx_m + i - 1) as u32, (cy_m - 1) as u32, m_color);
                framebuffer::put_pixel_fast((cx_m + i - 1) as u32, (cy_m + 3) as u32, m_color);
                framebuffer::put_pixel_fast((cx_m - 3) as u32, (cy_m + i + 1) as u32, m_color);
                framebuffer::put_pixel_fast((cx_m + 1) as u32, (cy_m + i + 1) as u32, m_color);
            }
        } else {
            // Square icon
            for i in -3..=3i32 {
                framebuffer::put_pixel_fast((cx_m + i) as u32, (cy_m - 3) as u32, m_color);
                framebuffer::put_pixel_fast((cx_m + i) as u32, (cy_m + 3) as u32, m_color);
                framebuffer::put_pixel_fast((cx_m - 3) as u32, (cy_m + i) as u32, m_color);
                framebuffer::put_pixel_fast((cx_m + 3) as u32, (cy_m + i) as u32, m_color);
            }
        }
        
        // Minimize button (third from right, amber)
        let min_x = max_x - btn_w as i32;
        let min_hover = mx >= min_x && mx < min_x + btn_w as i32 
            && my >= btn_y as i32 && my < btn_y as i32 + btn_h as i32;
        let min_bg = if min_hover { 0xFF2A2A10 } else { 0xFF0E0E0E };
        framebuffer::fill_rect(min_x as u32, btn_y, btn_w, btn_h, min_bg);
        let cx_n = min_x + btn_w as i32 / 2;
        let cy_n = btn_y as i32 + btn_h as i32 / 2;
        let n_color = if min_hover { 0xFFFFBB33 } else if window.focused { 0xFF886622 } else { 0xFF555555 };
        // Dash icon —
        for i in -3..=3i32 {
            framebuffer::put_pixel_fast((cx_n + i) as u32, cy_n as u32, n_color);
            framebuffer::put_pixel_fast((cx_n + i) as u32, (cy_n + 1) as u32, n_color);
        }
        
        // Button separator lines
        framebuffer::fill_rect(min_x as u32, btn_y, 1, btn_h, CHROME_GHOST);
        framebuffer::fill_rect(max_x as u32, btn_y, 1, btn_h, CHROME_GHOST);
        framebuffer::fill_rect(close_x as u32, btn_y, 1, btn_h, CHROME_GHOST);
        
        // Window icon (left side)
        let icon_x = x + 10;
        let icon_str = match window.window_type {
            WindowType::Terminal => ">_",
            WindowType::FileManager => "[]",
            WindowType::Calculator => "##",
            WindowType::Browser => "WW",
            WindowType::ModelEditor => "/\\",
            WindowType::TextEditor => "Tx",
            WindowType::Game => "Sk",
            WindowType::Chess => "Kk",
            WindowType::Chess3D => "C3",
            WindowType::MusicPlayer => "Mu",
            _ => "::",
        };
        let icon_color = if window.focused { GREEN_PRIMARY } else { GREEN_TERTIARY };
        self.draw_text_smooth(icon_x, y + (titlebar_h as i32 / 2) - 6, icon_str, icon_color);
        
        // Title text (centered in title bar)
        let text_color = if window.focused {
            hc(TEXT_PRIMARY, 0xFFFFFFFF)
        } else {
            hc(TEXT_SECONDARY, 0xFFCCCCCC)
        };
        let title_pixel_w = window.title.len() as i32 * 8;
        let title_center_x = x + (w as i32 / 2) - (title_pixel_w / 2);
        let title_x = title_center_x.max(icon_x + 24);
        self.draw_text_smooth(title_x, y + (titlebar_h as i32 / 2) - 6, &window.title, text_color);
        
        // ═══════════════════════════════════════════════════════════════
        // Content Area
        // ═══════════════════════════════════════════════════════════════
        let content_y = y + titlebar_h as i32;
        let content_h = h - titlebar_h;
        framebuffer::fill_rect((x + 3) as u32, (content_y + 1) as u32, w.saturating_sub(6), content_h.saturating_sub(4), 0xFF080808);
        
        // Set clip rect to window content area
        let clip_x = (x + 3).max(0) as u32;
        let clip_y = (content_y + 1).max(0) as u32;
        let clip_w = w.saturating_sub(6);
        let clip_h = content_h.saturating_sub(4);
        framebuffer::set_clip_rect(clip_x, clip_y, clip_w, clip_h);
        
        // Draw window content
        self.draw_window_content(window);
        
        // Clear clip rect
        framebuffer::clear_clip_rect();
    }
    
    /// Draw a modern minimal button (legacy - kept for compatibility)
    fn draw_modern_button(&self, x: u32, y: u32, size: u32, color: u32, hovered: bool) {
        if hovered {
            // Glow effect on hover
            framebuffer::fill_rect(x.saturating_sub(1), y.saturating_sub(1), size + 2, size + 2, 
                (color & 0x00FFFFFF) | 0x40000000);
        }
        framebuffer::fill_rect(x, y, size, size, color);
    }
    
    /// Blend two colors with a factor (0.0 = first color, 1.0 = second color)
    fn blend_colors(&self, c1: u32, c2: u32, t: f32) -> u32 {
        let r1 = ((c1 >> 16) & 0xFF) as f32;
        let g1 = ((c1 >> 8) & 0xFF) as f32;
        let b1 = (c1 & 0xFF) as f32;
        let r2 = ((c2 >> 16) & 0xFF) as f32;
        let g2 = ((c2 >> 8) & 0xFF) as f32;
        let b2 = (c2 & 0xFF) as f32;
        
        let r = (r1 + (r2 - r1) * t) as u32;
        let g = (g1 + (g2 - g1) * t) as u32;
        let b = (b1 + (b2 - b1) * t) as u32;
        
        0xFF000000 | (r << 16) | (g << 8) | b
    }
    
    fn draw_window_content(&self, window: &Window) {
        let content_x = window.x + 8;
        let content_y = window.y + TITLE_BAR_HEIGHT as i32 + 8;
        
        // TextEditor rendering is handled separately in draw_editor_windows
        if window.window_type == WindowType::TextEditor {
            return;
        }
        
        // ModelEditor is handled separately (needs &mut self)
        if window.window_type == WindowType::ModelEditor {
            return;
        }
        
        // Game3D is handled separately (needs &mut self)
        if window.window_type == WindowType::Game3D {
            return;
        }

        // Emulator windows are rendered in separate late passes
        #[cfg(feature = "emulators")]
        if window.window_type == WindowType::GameBoyEmu
            || window.window_type == WindowType::GameBoyInput
            || window.window_type == WindowType::NesEmu
        {
            return;
        }
        
        // Calculator is handled separately
        if window.window_type == WindowType::Calculator {
            self.draw_calculator(window);
            return;
        }
        
        // ═══════════════════════════════════════════════════════════════
        // MUSIC PLAYER — Glass-like widget with pulse wave visualization
        // ═══════════════════════════════════════════════════════════════
        if window.window_type == WindowType::MusicPlayer {
            self.draw_music_player(window);
            return;
        }
        
        // ═══════════════════════════════════════════════════════════════
        // FILE MANAGER — Windows Explorer-style graphical rendering
        // ═══════════════════════════════════════════════════════════════
        if window.window_type == WindowType::FileManager {
            self.draw_file_manager_gui(window);
            return;
        }
        
        // ═══════════════════════════════════════════════════════════════
        // IMAGE VIEWER — Render actual BMP image pixels
        // ═══════════════════════════════════════════════════════════════
        if window.window_type == WindowType::ImageViewer {
            self.draw_image_viewer(window);
            return;
        }
        
        // Special rendering for 3D demo window
        if window.window_type == WindowType::Demo3D {
            self.draw_3d_demo(window);
            return;
        }
        
        // Special rendering for Snake game
        if window.window_type == WindowType::Game {
            self.draw_snake_game(window);
            return;
        }
        
        // Special rendering for Chess game
        if window.window_type == WindowType::Chess {
            self.draw_chess_game(window);
            return;
        }
        
        // Special rendering for 3D Chess game (rendered in draw_chess3d_windows)
        if window.window_type == WindowType::Chess3D {
            return;
        }
        
        // Binary Viewer handled here
        if window.window_type == WindowType::BinaryViewer {
            self.draw_binary_viewer(window);
            return;
        }
        
        // TrustLab Mode
        if window.window_type == WindowType::LabMode {
            if let Some(state) = self.lab_states.get(&window.id) {
                crate::lab_mode::draw_lab(state, window.x, window.y, window.width, window.height);
            }
            return;
        }
        
        // GameLab — Game Boy emulator analysis dashboard
        #[cfg(feature = "emulators")]
        if window.window_type == WindowType::GameLab {
            if let Some(lab_state) = self.gamelab_states.get(&window.id) {
                // Find the linked Game Boy emulator (use linked_gb_id or first available)
                let emu_ref = if let Some(linked_id) = lab_state.linked_gb_id {
                    self.gameboy_states.get(&linked_id)
                } else {
                    // Auto-link to first active Game Boy emulator
                    self.gameboy_states.values().next()
                };
                crate::game_lab::draw_game_lab(lab_state, emu_ref, window.x, window.y, window.width, window.height);
            }
            return;
        }
        
        // Special rendering for Browser
        if window.window_type == WindowType::Browser {
            self.draw_browser(window);
            return;
        }
        
        // ═══════════════════════════════════════════════════════════════
        // SETTINGS — Graphical panel with sidebar categories
        // ═══════════════════════════════════════════════════════════════
        if window.window_type == WindowType::Settings {
            self.draw_settings_gui(window);
            return;
        }
        
        // ═══════════════════════════════════════════════════════════════
        // NETSCAN — Tabbed network toolkit GUI
        // ═══════════════════════════════════════════════════════════════
        if window.window_type == WindowType::NetworkInfo {
            self.draw_netscan_gui(window);
            return;
        }
        
        // ═══════════════════════════════════════════════════════════════
        // TERMINAL — v2 visual overhaul: styled prompt, modern scrollbar
        // ═══════════════════════════════════════════════════════════════
        if window.window_type == WindowType::Terminal {
            let line_height = 16i32;
            let content_area_h = (window.height as i32 - TITLE_BAR_HEIGHT as i32 - 16).max(0) as usize;
            let visible_lines = if line_height as usize > 0 { content_area_h / line_height as usize } else { 0 };
            let total_lines = window.content.len();
            
            // Determine scroll range
            let scroll = window.scroll_offset;
            let start = scroll;
            let end = (start + visible_lines).min(total_lines);
            
            for idx in start..end {
                let line = &window.content[idx];
                let line_y = content_y + ((idx - start) as i32 * line_height);
                if line_y >= window.y + window.height as i32 - 8 {
                    break;
                }
                
                // Parse color markers: \x01R = red, \x01G = green, \x01B = blue/cyan,
                // \x01W = white, \x01Y = yellow, \x01M = muted, \x01D = dim, \x01N = normal green
                // \x01H = header bright, \x01A = amber
                if line.contains('\x01') {
                    let mut cx = content_x;
                    let mut current_color = COLOR_GREEN;
                    let mut chars = line.chars().peekable();
                    while let Some(ch) = chars.next() {
                        if ch == '\x01' {
                            if let Some(&code) = chars.peek() {
                                chars.next();
                                current_color = match code {
                                    'R' => ACCENT_RED,
                                    'G' => GREEN_PRIMARY,
                                    'B' => ACCENT_BLUE,
                                    'W' => TEXT_PRIMARY,
                                    'Y' => ACCENT_AMBER,
                                    'M' => GREEN_MUTED,
                                    'D' => GREEN_GHOST,
                                    'N' => COLOR_GREEN,
                                    'H' => 0xFF00FFAA,
                                    'A' => GREEN_TERTIARY,
                                    'S' => GREEN_SUBTLE,
                                    _ => current_color,
                                };
                            }
                        } else {
                            crate::framebuffer::draw_char_at(cx as u32, line_y as u32, ch, current_color);
                            cx += 8;
                        }
                    }
                } else {
                    // Detect prompt lines (starts with "root@trustos" or "$")
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("root@trustos") || trimmed.starts_with("$") {
                        // Styled prompt: user@host in cyan, path in amber, $ in green
                        let mut cx = content_x;
                        if let Some(dollar_pos) = line.find('$') {
                            // Draw everything before $
                            let before = &line[..dollar_pos];
                            // Find @ to split user@host
                            if let Some(at_pos) = before.find('@') {
                                // "root" in bright green
                                for ch in before[..at_pos].chars() {
                                    crate::framebuffer::draw_char_at(cx as u32, line_y as u32, ch, GREEN_PRIMARY);
                                    cx += 8;
                                }
                                // "@" in dim
                                crate::framebuffer::draw_char_at(cx as u32, line_y as u32, '@', GREEN_GHOST);
                                cx += 8;
                                // "trustos" in cyan
                                let host_part = &before[at_pos + 1..];
                                // Find : separator for path
                                if let Some(colon_pos) = host_part.find(':') {
                                    for ch in host_part[..colon_pos].chars() {
                                        crate::framebuffer::draw_char_at(cx as u32, line_y as u32, ch, ACCENT_BLUE);
                                        cx += 8;
                                    }
                                    crate::framebuffer::draw_char_at(cx as u32, line_y as u32, ':', GREEN_GHOST);
                                    cx += 8;
                                    // Path in amber
                                    for ch in host_part[colon_pos + 1..].chars() {
                                        crate::framebuffer::draw_char_at(cx as u32, line_y as u32, ch, ACCENT_AMBER);
                                        cx += 8;
                                    }
                                } else {
                                    for ch in host_part.chars() {
                                        crate::framebuffer::draw_char_at(cx as u32, line_y as u32, ch, ACCENT_BLUE);
                                        cx += 8;
                                    }
                                }
                            } else {
                                for ch in before.chars() {
                                    crate::framebuffer::draw_char_at(cx as u32, line_y as u32, ch, GREEN_SECONDARY);
                                    cx += 8;
                                }
                            }
                            // $ in bright green
                            crate::framebuffer::draw_char_at(cx as u32, line_y as u32, '$', GREEN_PRIMARY);
                            cx += 8;
                            // Rest of line (user input) in white
                            for ch in line[dollar_pos + 1..].chars() {
                                crate::framebuffer::draw_char_at(cx as u32, line_y as u32, ch, TEXT_PRIMARY);
                                cx += 8;
                            }
                        } else {
                            self.draw_text(content_x, line_y, line, COLOR_GREEN);
                        }
                    } else {
                        // Plain green text (output)
                        self.draw_text(content_x, line_y, line, COLOR_GREEN);
                    }
                }
            }
            
            // ── Modern Scrollbar (rounded thumb) ──
            let scrollbar_w = 6u32;
            let scrollbar_x = (window.x + window.width as i32 - scrollbar_w as i32 - 3) as u32;
            let track_y = (window.y + TITLE_BAR_HEIGHT as i32 + 2) as u32;
            let track_h = window.height.saturating_sub(TITLE_BAR_HEIGHT + 4);
            
            if total_lines > visible_lines {
                // Track (very subtle)
                framebuffer::fill_rect_alpha(scrollbar_x, track_y, scrollbar_w, track_h, 0x0A1A0F, 80);
                
                // Rounded thumb
                let thumb_h = ((visible_lines as u32 * track_h) / total_lines as u32).max(20);
                let max_scroll = total_lines.saturating_sub(visible_lines);
                let thumb_y = if max_scroll > 0 {
                    track_y + ((scroll as u32 * (track_h - thumb_h)) / max_scroll as u32)
                } else {
                    track_y
                };
                
                draw_rounded_rect(scrollbar_x as i32, thumb_y as i32, scrollbar_w, thumb_h, 3, GREEN_MUTED);
                // Top/bottom highlight
                framebuffer::fill_rect_alpha(scrollbar_x + 1, thumb_y + 1, scrollbar_w - 2, 1, 0x00FF66, 30);
            }
            
            return;
        }
        
        // Check if this window type needs selection highlighting
        let needs_selection = matches!(window.window_type, 
            WindowType::FileManager | WindowType::FileAssociations);
        
        // Calculate which lines are selectable (skip headers)
        let (sel_start, sel_end) = match window.window_type {
            WindowType::FileManager => (5, window.content.len().saturating_sub(2)),
            WindowType::FileAssociations => (4, window.content.len().saturating_sub(2)),
            _ => (0, 0),
        };
        
        // Apply scroll_offset for HexViewer (and other scrollable generic windows)
        let scroll = if window.window_type == WindowType::HexViewer {
            window.scroll_offset
        } else {
            0
        };
        
        for (idx, line) in window.content.iter().enumerate().skip(scroll) {
            let i = idx - scroll;
            let line_y = content_y + (i as i32 * 16);
            if line_y >= window.y + window.height as i32 - 8 {
                break;
            }
            
            // Check if this line is selected
            let is_selected = needs_selection 
                && idx >= sel_start 
                && idx < sel_end 
                && (idx - sel_start) == window.selected_index;
            
            if is_selected {
                // Draw selection highlight
                framebuffer::fill_rect(
                    content_x as u32 - 4, 
                    line_y as u32 - 2, 
                    window.width - 16, 
                    18, 
                    0xFF003300
                );
                self.draw_text(content_x, line_y, line, COLOR_BRIGHT_GREEN);
            } else {
                self.draw_text(content_x, line_y, line, COLOR_GREEN);
            }
        }
    }
    
    /// Render all TextEditor windows (separate pass because we need &mut for editor state)
    fn draw_editor_windows(&mut self) {
        // Collect editor window info first to avoid borrow issues
        let editor_windows: Vec<(u32, i32, i32, u32, u32)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::TextEditor && w.visible && !w.minimized)
            .map(|w| (w.id, w.x, w.y, w.width, w.height))
            .collect();
        
        for (win_id, wx, wy, ww, wh) in editor_windows {
            if let Some(editor) = self.editor_states.get_mut(&win_id) {
                let content_x = wx;
                let content_y = wy + TITLE_BAR_HEIGHT as i32;
                let content_w = ww;
                let content_h = wh.saturating_sub(TITLE_BAR_HEIGHT);
                
                // Clip to window content area
                framebuffer::set_clip_rect(
                    (content_x + 2).max(0) as u32,
                    content_y.max(0) as u32,
                    content_w.saturating_sub(4),
                    content_h,
                );
                
                render_editor(
                    editor,
                    content_x, content_y, content_w, content_h,
                    &|x, y, text, color| {
                        // Use the font renderer char by char
                        for (i, ch) in text.chars().enumerate() {
                            let cx = (x + (i as i32 * 8)) as u32;
                            let cy = y as u32;
                            crate::framebuffer::draw_char_at(cx, cy, ch, color);
                        }
                    },
                    &|x, y, ch, color| {
                        crate::framebuffer::draw_char_at(x as u32, y as u32, ch, color);
                    },
                );
                
                framebuffer::clear_clip_rect();
            }
        }
    }
    
    /// Render all ModelEditor windows (separate pass because we need &mut for editor state)
    fn draw_model_editor_windows(&mut self) {
        // Collect model editor window info to avoid borrow issues
        let editor_windows: Vec<(u32, i32, i32, u32, u32)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::ModelEditor && w.visible && !w.minimized)
            .map(|w| (w.id, w.x, w.y, w.width, w.height))
            .collect();
        
        for (win_id, wx, wy, ww, wh) in editor_windows {
            if let Some(state) = self.model_editor_states.get_mut(&win_id) {
                let content_x = wx as u32;
                let content_y = (wy + TITLE_BAR_HEIGHT as i32) as u32;
                let content_w = ww;
                let content_h = wh.saturating_sub(TITLE_BAR_HEIGHT);
                
                if content_w < 80 || content_h < 80 { continue; }
                
                // Render into a buffer then blit
                let buf_w = content_w as usize;
                let buf_h = content_h as usize;
                let mut buf = alloc::vec![0u32; buf_w * buf_h];
                
                state.render(&mut buf, buf_w, buf_h);
                
                // Blit buffer to framebuffer
                for py in 0..buf_h {
                    for px in 0..buf_w {
                        let color = buf[py * buf_w + px];
                        let sx = content_x + px as u32;
                        let sy = content_y + py as u32;
                        framebuffer::put_pixel_fast(sx, sy, color);
                    }
                }
            }
        }
    }
    
    /// Render all 3D Game windows (separate pass because we need &mut for game state)
    fn draw_game3d_windows(&mut self) {
        let game_windows: Vec<(u32, i32, i32, u32, u32)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::Game3D && w.visible && !w.minimized)
            .map(|w| (w.id, w.x, w.y, w.width, w.height))
            .collect();
        
        for (win_id, wx, wy, ww, wh) in game_windows {
            if let Some(state) = self.game3d_states.get_mut(&win_id) {
                let content_x = wx as u32;
                let content_y = (wy + TITLE_BAR_HEIGHT as i32) as u32;
                let content_w = ww;
                let content_h = wh.saturating_sub(TITLE_BAR_HEIGHT);
                
                if content_w < 80 || content_h < 60 { continue; }
                
                let buf_w = content_w as usize;
                let buf_h = content_h as usize;
                let mut buf = alloc::vec![0u32; buf_w * buf_h];
                
                state.render(&mut buf, buf_w, buf_h);
                
                // Blit buffer to framebuffer
                for py in 0..buf_h {
                    for px in 0..buf_w {
                        let color = buf[py * buf_w + px];
                        let sx = content_x + px as u32;
                        let sy = content_y + py as u32;
                        framebuffer::put_pixel_fast(sx, sy, color);
                    }
                }
            }
        }
    }
    
    /// Render 3D chess windows (needs &mut self for state mutation during render)
    fn draw_chess3d_windows(&mut self) {
        let chess3d_windows: Vec<(u32, i32, i32, u32, u32)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::Chess3D && w.visible && !w.minimized && !w.pending_close)
            .map(|w| (w.id, w.x, w.y, w.width, w.height))
            .collect();
        
        for (win_id, wx, wy, ww, wh) in chess3d_windows {
            if let Some(state) = self.chess3d_states.get_mut(&win_id) {
                let content_x = wx as u32;
                let content_y = (wy + TITLE_BAR_HEIGHT as i32) as u32;
                let content_w = ww;
                let content_h = wh.saturating_sub(TITLE_BAR_HEIGHT);
                
                if content_w < 100 || content_h < 100 { continue; }
                
                state.tick();
                
                let buf_w = content_w as usize;
                let buf_h = content_h as usize;
                let mut buf = alloc::vec![0u32; buf_w * buf_h];
                
                state.render(&mut buf, buf_w, buf_h);
                
                // Blit buffer to framebuffer
                for py in 0..buf_h {
                    for px in 0..buf_w {
                        let color = buf[py * buf_w + px];
                        let sx = content_x + px as u32;
                        let sy = content_y + py as u32;
                        framebuffer::put_pixel_fast(sx, sy, color);
                    }
                }
            }
        }
    }
    
    /// Render NES emulator windows
    #[cfg(feature = "emulators")]
    fn draw_nes_windows(&mut self) {
        let nes_windows: Vec<(u32, i32, i32, u32, u32)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::NesEmu && w.visible && !w.minimized && !w.pending_close)
            .map(|w| (w.id, w.x, w.y, w.width, w.height))
            .collect();
        
        for (win_id, wx, wy, ww, wh) in nes_windows {
            if let Some(emu) = self.nes_states.get_mut(&win_id) {
                let content_x = wx as u32;
                let content_y = (wy + TITLE_BAR_HEIGHT as i32) as u32;
                let content_w = ww;
                let content_h = wh.saturating_sub(TITLE_BAR_HEIGHT);
                
                if content_w < 80 || content_h < 60 { continue; }
                
                let buf_w = content_w as usize;
                let buf_h = content_h as usize;
                let mut buf = alloc::vec![0u32; buf_w * buf_h];
                
                emu.render(&mut buf, buf_w, buf_h);
                
                for py in 0..buf_h {
                    for px in 0..buf_w {
                        let color = buf[py * buf_w + px];
                        let sx = content_x + px as u32;
                        let sy = content_y + py as u32;
                        framebuffer::put_pixel_fast(sx, sy, color);
                    }
                }
            }
        }
    }
    
    /// Render Game Boy emulator windows
    #[cfg(feature = "emulators")]
    fn draw_gameboy_windows(&mut self) {
        let gb_windows: Vec<(u32, i32, i32, u32, u32, bool)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::GameBoyEmu && w.visible && !w.minimized && !w.pending_close)
            .map(|w| (w.id, w.x, w.y, w.width, w.height, w.focused))
            .collect();
        
        let menu_h: u32 = 22;
        
        for (win_id, wx, wy, ww, wh, _focused) in gb_windows {
            if let Some(emu) = self.gameboy_states.get_mut(&win_id) {
                let content_x = wx as u32;
                let content_y = (wy + TITLE_BAR_HEIGHT as i32) as u32;
                let content_w = ww;
                let content_h = wh.saturating_sub(TITLE_BAR_HEIGHT);
                
                if content_w < 80 || content_h < 60 { continue; }
                
                // ── Menu bar at top ────────────────────────────────────────────
                framebuffer::fill_rect(content_x, content_y, content_w, menu_h, 0xFF0E1418);
                framebuffer::fill_rect(content_x, content_y + menu_h - 1, content_w, 1, 0xFF1E3028);
                
                // Game info (real-time)
                let pc_s = alloc::format!("PC:{:04X}", emu.cpu.pc);
                let ly_s = alloc::format!("LY:{:3}", emu.gpu.ly);
                let mode_s = match emu.gpu.mode {
                    0 => "HBL",
                    1 => "VBL",
                    2 => "OAM",
                    3 => "DRW",
                    _ => "???",
                };
                let bank_s = alloc::format!("BK:{}", emu.cart.rom_bank);
                
                let mut tx = content_x + 4;
                for ch in pc_s.chars() { framebuffer::draw_char_at(tx, content_y + 4, ch, 0xFF58A6FF); tx += 8; }
                tx += 8;
                for ch in ly_s.chars() { framebuffer::draw_char_at(tx, content_y + 4, ch, 0xFF80FFAA); tx += 8; }
                tx += 8;
                for ch in mode_s.chars() { framebuffer::draw_char_at(tx, content_y + 4, ch, 0xFFD29922); tx += 8; }
                tx += 8;
                for ch in bank_s.chars() { framebuffer::draw_char_at(tx, content_y + 4, ch, 0xFF9CD8B0); tx += 8; }
                
                if emu.cgb_mode {
                    tx += 8;
                    let spd = if emu.key1 & 0x80 != 0 { "2x" } else { "1x" };
                    for ch in "CGB".chars() { framebuffer::draw_char_at(tx, content_y + 4, ch, 0xFF00FF88); tx += 8; }
                    tx += 4;
                    for ch in spd.chars() { framebuffer::draw_char_at(tx, content_y + 4, ch, 0xFF79C0FF); tx += 8; }
                }
                
                // Menu buttons (right-aligned)
                // [INPUT] button
                let inp_btn_w: u32 = 48;
                let inp_btn_x = content_x + content_w - inp_btn_w - 4;
                framebuffer::fill_rect(inp_btn_x, content_y + 2, inp_btn_w, menu_h - 4, 0xFF1A3028);
                framebuffer::fill_rect(inp_btn_x, content_y + 2, inp_btn_w, 1, 0xFF2A4A38);
                framebuffer::fill_rect(inp_btn_x, content_y + menu_h - 3, inp_btn_w, 1, 0xFF2A4A38);
                let itx = inp_btn_x + 4;
                for (i, ch) in "INPUT".chars().enumerate() {
                    framebuffer::draw_char_at(itx + i as u32 * 8, content_y + 5, ch, 0xFF00FF88);
                }
                
                // [LAB] button
                let lab_btn_w: u32 = 32;
                let lab_btn_x = inp_btn_x - lab_btn_w - 6;
                framebuffer::fill_rect(lab_btn_x, content_y + 2, lab_btn_w, menu_h - 4, 0xFF1A2838);
                framebuffer::fill_rect(lab_btn_x, content_y + 2, lab_btn_w, 1, 0xFF2A3A58);
                framebuffer::fill_rect(lab_btn_x, content_y + menu_h - 3, lab_btn_w, 1, 0xFF2A3A58);
                let ltx = lab_btn_x + 4;
                for (i, ch) in "LAB".chars().enumerate() {
                    framebuffer::draw_char_at(ltx + i as u32 * 8, content_y + 5, ch, 0xFF58A6FF);
                }
                
                // ── Game rendering below menu ──────────────────────────
                let game_y = content_y + menu_h;
                let game_h = content_h.saturating_sub(menu_h);
                
                if game_h < 40 { continue; }
                
                let buf_w = content_w as usize;
                let buf_h = game_h as usize;
                let mut buf = alloc::vec![0u32; buf_w * buf_h];
                
                emu.render(&mut buf, buf_w, buf_h);
                
                for py in 0..buf_h {
                    for px in 0..buf_w {
                        let color = buf[py * buf_w + px];
                        let sx = content_x + px as u32;
                        let sy = game_y + py as u32;
                        framebuffer::put_pixel_fast(sx, sy, color);
                    }
                }
            }
        }
        
        // ── Render GameBoyInput windows ────────────────────────────────
        let input_windows: Vec<(u32, i32, i32, u32, u32, Option<u32>)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::GameBoyInput && w.visible && !w.minimized && !w.pending_close)
            .map(|w| {
                let linked = self.gb_input_links.get(&w.id).copied();
                (w.id, w.x, w.y, w.width, w.height, linked)
            })
            .collect();
        
        for (_win_id, wx, wy, ww, wh, linked_id) in input_windows {
            let cx = wx as u32;
            let cy = (wy + TITLE_BAR_HEIGHT as i32) as u32;
            let cw = ww;
            let ch = wh.saturating_sub(TITLE_BAR_HEIGHT);
            
            if cw < 60 || ch < 40 { continue; }
            
            // Find the linked emulator
            let emu_opt = if let Some(lid) = linked_id {
                self.gameboy_states.get(&lid)
            } else {
                self.gameboy_states.values().next()
            };
            
            crate::game_lab::draw_input_window(emu_opt, cx, cy, cw, ch);
        }
    }
    
    /// Draw 3D graphics demo using TrustGL (OpenGL-like API)
    fn draw_3d_demo(&self, window: &Window) {
        use crate::graphics::opengl::*;
        use crate::graphics::texture;
        
        let demo_x = window.x as u32 + 10;
        let demo_y = window.y as u32 + TITLE_BAR_HEIGHT + 10;
        let demo_w = window.width.saturating_sub(20);
        let demo_h = window.height.saturating_sub(TITLE_BAR_HEIGHT + 20);
        
        if demo_w < 80 || demo_h < 80 {
            return;
        }
        
        // Initialize TrustGL for this viewport
        gl_init(demo_w, demo_h);
        gl_viewport(demo_x as i32, demo_y as i32, demo_w, demo_h);
        
        // Clear with dark background
        gl_clear_color(0.04, 0.06, 0.04, 1.0);
        gl_clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        
        // Enable depth testing
        gl_enable(GL_DEPTH_TEST);
        
        // Set up projection matrix
        let aspect = demo_w as f32 / demo_h as f32;
        gl_matrix_mode(GL_PROJECTION);
        gl_load_identity();
        glu_perspective(45.0, aspect, 0.1, 100.0);
        
        // Set up modelview matrix
        gl_matrix_mode(GL_MODELVIEW);
        gl_load_identity();
        glu_look_at(
            3.0, 2.0, 4.0,   // eye position
            0.0, 0.0, 0.0,   // look at center
            0.0, 1.0, 0.0    // up vector
        );
        
        // Simple rotation based on a static angle (could be animated with time)
        let angle = (self.frame_count as f32 * 0.5) % 360.0;
        gl_rotatef(angle, 0.0, 1.0, 0.0);
        gl_rotatef(angle * 0.3, 1.0, 0.0, 0.0);
        
        // Draw a colorful cube using immediate mode
        let s = 0.8;
        
        gl_begin(GL_QUADS);
        
        // Front face (red)
        gl_color3f(1.0, 0.2, 0.2);
        gl_normal3f(0.0, 0.0, 1.0);
        gl_vertex3f(-s, -s, s);
        gl_vertex3f(s, -s, s);
        gl_vertex3f(s, s, s);
        gl_vertex3f(-s, s, s);
        
        // Back face (green)
        gl_color3f(0.2, 1.0, 0.2);
        gl_normal3f(0.0, 0.0, -1.0);
        gl_vertex3f(s, -s, -s);
        gl_vertex3f(-s, -s, -s);
        gl_vertex3f(-s, s, -s);
        gl_vertex3f(s, s, -s);
        
        // Top face (blue)
        gl_color3f(0.2, 0.2, 1.0);
        gl_normal3f(0.0, 1.0, 0.0);
        gl_vertex3f(-s, s, s);
        gl_vertex3f(s, s, s);
        gl_vertex3f(s, s, -s);
        gl_vertex3f(-s, s, -s);
        
        // Bottom face (yellow)
        gl_color3f(1.0, 1.0, 0.2);
        gl_normal3f(0.0, -1.0, 0.0);
        gl_vertex3f(-s, -s, -s);
        gl_vertex3f(s, -s, -s);
        gl_vertex3f(s, -s, s);
        gl_vertex3f(-s, -s, s);
        
        // Right face (magenta)
        gl_color3f(1.0, 0.2, 1.0);
        gl_normal3f(1.0, 0.0, 0.0);
        gl_vertex3f(s, -s, s);
        gl_vertex3f(s, -s, -s);
        gl_vertex3f(s, s, -s);
        gl_vertex3f(s, s, s);
        
        // Left face (cyan)
        gl_color3f(0.2, 1.0, 1.0);
        gl_normal3f(-1.0, 0.0, 0.0);
        gl_vertex3f(-s, -s, -s);
        gl_vertex3f(-s, -s, s);
        gl_vertex3f(-s, s, s);
        gl_vertex3f(-s, s, -s);
        
        gl_end();
        
        // === TEXTURED CUBE (offset to the right) ===
        gl_push_matrix();
        gl_translatef(2.5, 0.0, 0.0); // Offset to the right
        gl_rotatef(angle * 0.7, 0.3, 1.0, 0.2);
        
        // Create/use checkerboard texture
        static mut DEMO_TEX_ID: u32 = 0;
        static mut TEX_INIT: bool = false;
        unsafe {
            if !TEX_INIT {
                demo_init_checkerboard_texture(&mut DEMO_TEX_ID);
                TEX_INIT = true;
            }
            demo_render_textured_cube(0.0, DEMO_TEX_ID);
        }
        gl_pop_matrix();
        
        // Draw coordinate axes with lines
        gl_load_identity();
        glu_look_at(3.0, 2.0, 4.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        
        gl_begin(GL_LINES);
        // X axis (red)
        gl_color3f(1.0, 0.0, 0.0);
        gl_vertex3f(0.0, 0.0, 0.0);
        gl_vertex3f(2.0, 0.0, 0.0);
        // Y axis (green)
        gl_color3f(0.0, 1.0, 0.0);
        gl_vertex3f(0.0, 0.0, 0.0);
        gl_vertex3f(0.0, 2.0, 0.0);
        // Z axis (blue)
        gl_color3f(0.0, 0.0, 1.0);
        gl_vertex3f(0.0, 0.0, 0.0);
        gl_vertex3f(0.0, 0.0, 2.0);
        gl_end();
        
        // Draw text overlay
        self.draw_text(demo_x as i32 + 8, demo_y as i32 + 8, "TrustGL OpenGL Demo", GREEN_SECONDARY);
        self.draw_text(demo_x as i32 + 8, demo_y as i32 + 24, "Software 3D + Textures", GREEN_TERTIARY);
        
        // Stats
        let stats_y = demo_y as i32 + demo_h as i32 - 24;
        self.draw_text(demo_x as i32 + 8, stats_y, "Left: Color Cube | Right: Textured Cube", GREEN_MUTED);
        self.draw_text(demo_x as i32 + 8, stats_y, "Vertices: 8 | Edges: 12 | Faces: 6", GREEN_MUTED);
    }
    
    /// Draw a simple line using Bresenham's algorithm
    fn draw_line_simple(&self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        
        loop {
            if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
                framebuffer::put_pixel_fast(x as u32, y as u32, color);
            }
            if x == x1 && y == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
    
    /// Draw Snake game window content
    fn draw_snake_game(&self, window: &Window) {
        let game_x = window.x as u32 + 10;
        let game_y = window.y as u32 + TITLE_BAR_HEIGHT + 10;
        let game_w = window.width.saturating_sub(20);
        let game_h = window.height.saturating_sub(TITLE_BAR_HEIGHT + 20);
        
        if game_w < 80 || game_h < 80 {
            return;
        }
        
        // Draw game area background
        framebuffer::fill_rect(game_x, game_y, game_w, game_h, 0xFF0A0E0B);
        
        // Draw border
        for i in 0..game_w {
            framebuffer::put_pixel_fast(game_x + i, game_y, GREEN_MUTED);
            framebuffer::put_pixel_fast(game_x + i, game_y + game_h - 1, GREEN_MUTED);
        }
        for i in 0..game_h {
            framebuffer::put_pixel_fast(game_x, game_y + i, GREEN_MUTED);
            framebuffer::put_pixel_fast(game_x + game_w - 1, game_y + i, GREEN_MUTED);
        }
        
        // Get snake state
        if let Some(snake) = self.snake_states.get(&window.id) {
            let cell_size: u32 = 14;
            let grid_offset_x = game_x + 10;
            let grid_offset_y = game_y + 36;
            
            // Draw grid background (subtle)
            for gy in 0..snake.grid_h {
                for gx in 0..snake.grid_w {
                    let px = grid_offset_x + gx as u32 * cell_size;
                    let py = grid_offset_y + gy as u32 * cell_size;
                    if px + cell_size < game_x + game_w && py + cell_size < game_y + game_h {
                        let bg = if (gx + gy) % 2 == 0 { 0xFF0D120E } else { 0xFF0B100C };
                        framebuffer::fill_rect(px, py, cell_size, cell_size, bg);
                    }
                }
            }
            
            // Draw snake
            for (i, &(sx, sy)) in snake.snake.iter().enumerate() {
                let px = grid_offset_x + sx as u32 * cell_size;
                let py = grid_offset_y + sy as u32 * cell_size;
                let color = if i == 0 { 
                    0xFF00FF00 // Head is bright green
                } else {
                    let fade = (0xCC - (i as u32 * 8).min(0x80)) as u32;
                    0xFF000000 | (fade << 8) // Body fades
                };
                
                if px + cell_size < game_x + game_w && py + cell_size < game_y + game_h {
                    framebuffer::fill_rect(px + 1, py + 1, cell_size - 2, cell_size - 2, color);
                    // Head has eyes
                    if i == 0 {
                        let (ex1, ey1, ex2, ey2) = match snake.direction {
                            (1, 0) => (cell_size-4, 3, cell_size-4, cell_size-5), // Right
                            (-1, 0) => (2, 3, 2, cell_size-5),                     // Left
                            (0, -1) => (3, 2, cell_size-5, 2),                     // Up
                            _ => (3, cell_size-4, cell_size-5, cell_size-4),       // Down
                        };
                        framebuffer::put_pixel_fast(px + ex1, py + ey1, 0xFF000000);
                        framebuffer::put_pixel_fast(px + ex2, py + ey2, 0xFF000000);
                    }
                }
            }
            
            // Draw food
            let fx = grid_offset_x + snake.food.0 as u32 * cell_size;
            let fy = grid_offset_y + snake.food.1 as u32 * cell_size;
            if fx + cell_size < game_x + game_w && fy + cell_size < game_y + game_h {
                framebuffer::fill_rect(fx + 2, fy + 2, cell_size - 4, cell_size - 4, 0xFFFF4444);
                framebuffer::put_pixel_fast(fx + cell_size/2, fy + 1, 0xFF00AA00); // stem
            }
            
            // Title
            self.draw_text(game_x as i32 + 8, game_y as i32 + 8, "SNAKE", COLOR_BRIGHT_GREEN);
            
            // Score + High Score
            let score_str = if snake.high_score > 0 {
                format!("Score: {}  Best: {}", snake.score, snake.high_score)
            } else {
                format!("Score: {}", snake.score)
            };
            self.draw_text(game_x as i32 + game_w as i32 - 170, game_y as i32 + 8, &score_str, GREEN_SECONDARY);
            
            if snake.game_over {
                // Game over overlay
                let ox = game_x + game_w / 2 - 60;
                let oy = game_y + game_h / 2 - 20;
                framebuffer::fill_rect(ox - 4, oy - 4, 128, 58, 0xCC000000);
                self.draw_text(ox as i32, oy as i32, "GAME OVER", 0xFFFF4444);
                let final_str = format!("Score: {}", snake.score);
                self.draw_text(ox as i32 + 4, oy as i32 + 18, &final_str, GREEN_SECONDARY);
                self.draw_text(ox as i32 - 8, oy as i32 + 36, "Press ENTER", GREEN_TERTIARY);
            } else if snake.paused {
                // Pause overlay
                let ox = game_x + game_w / 2 - 50;
                let oy = game_y + game_h / 2 - 20;
                framebuffer::fill_rect(ox - 4, oy - 4, 110, 48, 0xCC000000);
                self.draw_text(ox as i32 + 8, oy as i32, "PAUSED", 0xFFFFCC00);
                self.draw_text(ox as i32 - 4, oy as i32 + 20, "P to resume", GREEN_TERTIARY);
            } else {
                // Instructions
                self.draw_text(game_x as i32 + 8, game_y as i32 + game_h as i32 - 18, 
                               "Arrows to move | P pause", GREEN_TERTIARY);
            }
        }
    }
    
    /// Draw a chess piece silhouette at pixel position (px, py) within a 48x48 cell
    fn draw_chess_piece_sprite(px: u32, py: u32, piece: i8) {
        let abs_p = if piece < 0 { -piece } else { piece };
        let is_white = piece > 0;

        let fill = if is_white { 0xFFE8E0D0_u32 } else { 0xFF2A2A2A_u32 };
        let outline = if is_white { 0xFF1A1A1A_u32 } else { 0xFF888888_u32 };

        // Parts: (x_offset, y_offset, width, height) relative to cell (px, py)
        // Pieces designed for 48x48 cells, centered horizontally
        let parts: &[(u32, u32, u32, u32)] = match abs_p {
            1 => &[ // PAWN — round head, narrow neck, flared base
                (20, 12, 8, 7),   // head ball
                (22, 19, 4, 3),   // neck
                (19, 22, 10, 3),  // collar
                (16, 25, 16, 3),  // skirt
                (14, 28, 20, 3),  // base top
                (12, 31, 24, 4),  // base bottom
            ],
            2 => &[ // KNIGHT — horse head profile (faces left)
                (21, 8, 6, 3),    // ear
                (17, 11, 14, 4),  // upper head
                (13, 15, 14, 3),  // muzzle extends left
                (13, 18, 8, 2),   // lower jaw
                (19, 17, 10, 5),  // neck upper
                (21, 22, 8, 5),   // neck lower
                (16, 27, 16, 3),  // shoulder
                (13, 30, 22, 3),  // base
                (11, 33, 26, 3),  // footer
            ],
            3 => &[ // BISHOP — pointed mitre with slit
                (23, 6, 2, 3),    // point
                (21, 9, 6, 4),    // upper mitre
                (19, 13, 10, 4),  // mid mitre
                (21, 17, 6, 5),   // stem
                (18, 22, 12, 4),  // lower body
                (15, 26, 18, 3),  // base top
                (13, 29, 22, 3),  // base
                (11, 32, 26, 4),  // footer
            ],
            4 => &[ // ROOK — tower with crenellations
                (15, 7, 4, 4),    // left merlon
                (22, 7, 4, 4),    // center merlon
                (29, 7, 4, 4),    // right merlon
                (15, 11, 18, 3),  // top wall
                (17, 14, 14, 12), // body (tall)
                (15, 26, 18, 3),  // lower rim
                (13, 29, 22, 3),  // base
                (11, 32, 26, 4),  // footer
            ],
            5 => &[ // QUEEN — crown with points + round body
                (23, 4, 2, 3),    // top jewel
                (17, 7, 2, 3),    // left crown point
                (23, 6, 2, 3),    // center crown point
                (29, 7, 2, 3),    // right crown point
                (16, 10, 16, 4),  // crown base
                (20, 14, 8, 4),   // neck
                (17, 18, 14, 6),  // body
                (15, 24, 18, 3),  // lower
                (13, 27, 22, 3),  // base
                (11, 30, 26, 4),  // footer
            ],
            6 => &[ // KING — cross on top + tall body
                (23, 4, 2, 6),    // cross vertical
                (20, 6, 8, 2),    // cross horizontal
                (18, 10, 12, 4),  // crown
                (20, 14, 8, 3),   // neck
                (17, 17, 14, 7),  // body
                (15, 24, 18, 3),  // lower
                (13, 27, 22, 3),  // base
                (11, 30, 26, 4),  // footer
            ],
            _ => return,
        };

        // Outline pass — draw each part inflated by 1px
        for &(x, y, w, h) in parts {
            framebuffer::fill_rect(px + x - 1, py + y - 1, w + 2, h + 2, outline);
        }
        // Fill pass
        for &(x, y, w, h) in parts {
            framebuffer::fill_rect(px + x, py + y, w, h, fill);
        }
        // Highlight pass — thin bright line on left side for 3D effect
        let hl = if is_white { 0x66FFFFFF_u32 } else { 0x44FFFFFF_u32 };
        for &(x, y, w, h) in parts {
            if w > 4 && h > 2 {
                framebuffer::fill_rect(px + x + 1, py + y + 1, 1, h - 2, hl);
            }
        }

        // Bishop slit — distinctive diagonal cut
        if abs_p == 3 {
            let slit_color = outline;
            framebuffer::fill_rect(px + 22, py + 14, 4, 1, slit_color);
            framebuffer::fill_rect(px + 21, py + 15, 4, 1, slit_color);
        }
    }

    /// Draw chess game board
    fn draw_chess_game(&self, window: &Window) {
        let game_x = window.x as u32 + 8;
        let game_y = window.y as u32 + TITLE_BAR_HEIGHT + 4;
        let game_w = window.width.saturating_sub(16);
        let game_h = window.height.saturating_sub(TITLE_BAR_HEIGHT + 8);
        
        if game_w < 200 || game_h < 200 {
            return;
        }
        
        // Background
        framebuffer::fill_rect(game_x, game_y, game_w, game_h, 0xFF0A0E0B);
        
        if let Some(chess) = self.chess_states.get(&window.id) {
            // Board dimensions
            let cell_size: u32 = 48;
            let board_size = cell_size * 8;
            let board_x = game_x + (game_w.saturating_sub(board_size)) / 2;
            let board_y = game_y + 28;
            
            // ── Title ──
            self.draw_text(game_x as i32 + 8, game_y as i32 + 6, "TRUSTCHESS", GREEN_PRIMARY);
            
            // ── Score display (material advantage) ──
            let score = chess.material_score();
            let score_text = if score > 0 {
                format!("+{}", score / 100)
            } else if score < 0 {
                format!("{}", score / 100)
            } else {
                String::from("=")
            };
            let score_color = if score > 0 { 0xFFFFFFFF } else if score < 0 { 0xFFCC4444 } else { GREEN_MUTED };
            // Score bar next to title
            self.draw_text(game_x as i32 + 96, game_y as i32 + 6, &score_text, score_color);
            
            // Difficulty label
            let diff_label = match chess.ai_depth { 1 => "Easy", 2 => "Med", _ => "Hard" };
            self.draw_text(game_x as i32 + 130, game_y as i32 + 6, diff_label, GREEN_MUTED);
            
            // ── Timer display ──
            if chess.timer_enabled {
                let btime = crate::chess::ChessState::format_time(chess.black_time_ms);
                let wtime = crate::chess::ChessState::format_time(chess.white_time_ms);
                // Black timer (top-right)
                let timer_color_b = if !chess.white_turn && chess.timer_started { 0xFFCC4444 } else { GREEN_MUTED };
                self.draw_text(board_x as i32 + board_size as i32 + 8, board_y as i32 + 4, &btime, timer_color_b);
                crate::framebuffer::draw_char_at(board_x + board_size + 8, board_y + 14, 'B', 0xFFCC4444);
                // White timer (bottom-right)
                let timer_color_w = if chess.white_turn && chess.timer_started { 0xFFFFFFFF } else { GREEN_MUTED };
                self.draw_text(board_x as i32 + board_size as i32 + 8, board_y as i32 + board_size as i32 - 20, &wtime, timer_color_w);
                crate::framebuffer::draw_char_at(board_x + board_size + 8, board_y + board_size - 10, 'W', 0xFFFFFFFF);
            }
            
            // ── Draw board ──
            for row in 0..8u32 {
                for col in 0..8u32 {
                    let sq = (row * 8 + col) as usize;
                    let px = board_x + col * cell_size;
                    let py = board_y + row * cell_size;
                    
                    // Square colors — dark/light alternating
                    let is_light = (row + col) % 2 == 0;
                    let mut bg = if is_light { 0xFF3D5A3D } else { 0xFF1A2E1A };
                    
                    // Highlight selected piece
                    if chess.selected == Some(sq) {
                        bg = 0xFF5A7A2A; // Gold-green
                    }
                    
                    // Highlight valid moves
                    if chess.valid_moves.contains(&sq) {
                        bg = if is_light { 0xFF4A8A4A } else { 0xFF2A6A2A };
                    }
                    
                    // Highlight last move
                    if chess.last_move_from == Some(sq) || chess.last_move_to == Some(sq) {
                        bg = if is_light { 0xFF5A6A3A } else { 0xFF3A4A2A };
                    }
                    
                    // Highlight cursor
                    if chess.cursor == sq {
                        bg = 0xFF00AA44; // Bright green cursor
                    }
                    
                    framebuffer::fill_rect(px, py, cell_size, cell_size, bg);
                    
                    // Draw piece (skip if being dragged from this square)
                    let piece = chess.board[sq];
                    let is_being_dragged = chess.drag_from == Some(sq) && chess.dragging_piece.is_some();
                    if piece != 0 && !is_being_dragged {
                        Self::draw_chess_piece_sprite(px, py, piece);
                    }
                    
                    // Valid move dots (for empty target squares)
                    if chess.valid_moves.contains(&sq) && (piece == 0 || is_being_dragged) {
                        let dot_x = px + cell_size / 2 - 3;
                        let dot_y = py + cell_size / 2 - 3;
                        framebuffer::fill_rect(dot_x, dot_y, 6, 6, 0xFF00FF66);
                    }
                    
                    // Valid move capture indicator (circle corners on enemy-occupied squares)
                    if chess.valid_moves.contains(&sq) && piece != 0 && !is_being_dragged {
                        // Draw corner triangles to indicate capturable
                        for dx in 0..4u32 {
                            framebuffer::put_pixel_fast(px + dx, py, 0xFF00FF66);
                            framebuffer::put_pixel_fast(px, py + dx, 0xFF00FF66);
                            framebuffer::put_pixel_fast(px + cell_size - 1 - dx, py, 0xFF00FF66);
                            framebuffer::put_pixel_fast(px + cell_size - 1, py + dx, 0xFF00FF66);
                            framebuffer::put_pixel_fast(px + dx, py + cell_size - 1, 0xFF00FF66);
                            framebuffer::put_pixel_fast(px, py + cell_size - 1 - dx, 0xFF00FF66);
                            framebuffer::put_pixel_fast(px + cell_size - 1 - dx, py + cell_size - 1, 0xFF00FF66);
                            framebuffer::put_pixel_fast(px + cell_size - 1, py + cell_size - 1 - dx, 0xFF00FF66);
                        }
                    }
                }
            }
            
            // ── Draw dragged piece at mouse cursor ──
            if let (Some(_from), Some(dp)) = (chess.drag_from, chess.dragging_piece) {
                let dx = chess.drag_pixel_x;
                let dy = chess.drag_pixel_y;
                if dx > 24 && dy > 24 {
                    Self::draw_chess_piece_sprite(dx as u32 - 24, dy as u32 - 24, dp);
                }
            }
            
            // ── Board border ──
            for i in 0..board_size {
                framebuffer::put_pixel_fast(board_x + i, board_y, GREEN_MUTED);
                framebuffer::put_pixel_fast(board_x + i, board_y + board_size, GREEN_MUTED);
            }
            for i in 0..board_size + 1 {
                framebuffer::put_pixel_fast(board_x, board_y + i, GREEN_MUTED);
                framebuffer::put_pixel_fast(board_x + board_size, board_y + i, GREEN_MUTED);
            }
            
            // ── File labels (a-h) ──
            for c in 0..8u32 {
                let label = (b'a' + c as u8) as char;
                crate::framebuffer::draw_char_at(board_x + c * cell_size + cell_size / 2 - 4, board_y + board_size + 4, label, GREEN_TERTIARY);
            }
            // ── Rank labels (8-1) ──
            for r in 0..8u32 {
                let label = (b'8' - r as u8) as char;
                crate::framebuffer::draw_char_at(board_x - 14, board_y + r * cell_size + cell_size / 2 - 6, label, GREEN_TERTIARY);
            }
            
            // ── Material score bar (visual) ──
            let bar_y = board_y + board_size + 18;
            let bar_w = board_size;
            let bar_h = 6u32;
            framebuffer::fill_rect(board_x, bar_y, bar_w, bar_h, 0xFF1A1A1A);
            // Fill proportional to score — center = equal, left = black advantage, right = white
            let max_score = 2000i32; // Clamp range
            let clamped = score.clamp(-max_score, max_score);
            let center = board_x + bar_w / 2;
            if clamped > 0 {
                let fill_w = ((clamped as u32) * (bar_w / 2)) / max_score as u32;
                framebuffer::fill_rect(center, bar_y, fill_w.min(bar_w / 2), bar_h, 0xFFFFFFFF);
            } else if clamped < 0 {
                let fill_w = (((-clamped) as u32) * (bar_w / 2)) / max_score as u32;
                let fill_w = fill_w.min(bar_w / 2);
                framebuffer::fill_rect(center - fill_w, bar_y, fill_w, bar_h, 0xFFCC4444);
            }
            // Center tick mark
            framebuffer::fill_rect(center, bar_y, 1, bar_h, GREEN_MUTED);
            
            // ── Status message ──
            let status_y = bar_y + bar_h + 6;
            let msg_color = match chess.phase {
                crate::chess::GamePhase::Check => ACCENT_RED,
                crate::chess::GamePhase::Checkmate => 0xFFFF4444,
                crate::chess::GamePhase::Stalemate => ACCENT_AMBER,
                crate::chess::GamePhase::Promotion => ACCENT_BLUE,
                _ => GREEN_PRIMARY,
            };
            self.draw_text(board_x as i32, status_y as i32, &chess.message, msg_color);
            
            // ── Turn indicator ──
            let turn_text = if chess.white_turn { "White" } else { "Black" };
            let turn_color = if chess.white_turn { 0xFFFFFFFF } else { 0xFFCC4444 };
            self.draw_text(board_x as i32 + board_size as i32 - 60, status_y as i32, turn_text, turn_color);
            
            // ── Move history (last 6) ──
            let hist_y = status_y as u32 + 18;
            let hist_start = if chess.move_history.len() > 6 { chess.move_history.len() - 6 } else { 0 };
            let mut hx = board_x as i32;
            for (i, m) in chess.move_history[hist_start..].iter().enumerate() {
                let num = hist_start + i + 1;
                let entry = format!("{}. {} ", num, m);
                self.draw_text(hx, hist_y as i32, &entry, GREEN_MUTED);
                hx += entry.len() as i32 * 8 + 4;
                if hx > board_x as i32 + board_size as i32 - 40 {
                    break; // Don't overflow
                }
            }
            
            // ── Controls hint ──
            let hint_y = game_y + game_h - 30;
            self.draw_text(game_x as i32 + 4, hint_y as i32,
                           "Mouse:Click/Drag  Arrows:Move  Enter:Select", GREEN_TERTIARY);
            self.draw_text(game_x as i32 + 4, hint_y as i32 + 12,
                           "Esc:Desel  R:Reset  T:Timer  D:Difficulty", GREEN_TERTIARY);
        }
    }
    
    /// Draw TrustView binary viewer
    fn draw_binary_viewer(&self, window: &Window) {
        if let Some(state) = self.binary_viewer_states.get(&window.id) {
            let draw_text = |x: i32, y: i32, text: &str, color: u32| {
                self.draw_text(x, y, text, color);
            };
            crate::apps::binary_viewer::draw_binary_viewer(
                state,
                window.x, window.y,
                window.width, window.height,
                &draw_text,
            );
        }
    }

    /// Open binary viewer for a file path
    pub fn open_binary_viewer(&mut self, path: &str) -> Result<u32, &'static str> {
        let analysis = crate::binary_analysis::analyze_path(path)?;
        let state = crate::apps::binary_viewer::BinaryViewerState::new(analysis, path);
        
        let title_str = alloc::format!("TrustView — {}", path);
        // Large window for multi-panel view
        let id = self.create_window(&title_str, 50, 50, 1100, 650, WindowType::BinaryViewer);
        self.binary_viewer_states.insert(id, state);
        Ok(id)
    }

    /// Open TrustLab mode window
    pub fn open_lab_mode(&mut self) -> u32 {
        let id = self.create_window("TrustLab \u{2014} OS Introspection", 30, 30, 1200, 700, WindowType::LabMode);
        // Force fullscreen (maximized)
        let sw = crate::framebuffer::width() as u32;
        let sh = crate::framebuffer::height() as u32;
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.saved_x = w.x;
            w.saved_y = w.y;
            w.saved_width = w.width;
            w.saved_height = w.height;
            w.x = 0;
            w.y = 0;
            w.width = sw;
            w.height = sh - TASKBAR_HEIGHT;
            w.maximized = true;
        }
        self.focus_window(id);
        id
    }

    // ═══════════════════════════════════════════════════════════════════════
    // IMAGE VIEWER — Renders actual BMP pixel data in-window
    // ═══════════════════════════════════════════════════════════════════════
    
    fn draw_image_viewer(&self, window: &Window) {
        let wx = window.x;
        let wy = window.y;
        let ww = window.width;
        let wh = window.height;
        if ww < 60 || wh < 80 { return; }
        
        let content_y = wy + TITLE_BAR_HEIGHT as i32;
        let content_h = wh.saturating_sub(TITLE_BAR_HEIGHT + 28); // 28px for status bar
        let safe_x = if wx < 0 { 0u32 } else { wx as u32 };
        let safe_y = content_y as u32;
        
        // Dark background
        framebuffer::fill_rect(safe_x + 2, safe_y, ww.saturating_sub(4), content_h, 0xFF080808);
        
        if let Some(state) = self.image_viewer_states.get(&window.id) {
            if state.img_width > 0 && state.img_height > 0 && !state.pixels.is_empty() {
                // Calculate display size with zoom
                let zoom_f = state.zoom as u32;
                let disp_w = (state.img_width * zoom_f) / 100;
                let disp_h = (state.img_height * zoom_f) / 100;
                
                // Center image in window
                let offset_x = (ww as i32 - disp_w as i32) / 2 + state.pan_x;
                let offset_y = (content_h as i32 - disp_h as i32) / 2 + state.pan_y;
                
                // Draw pixel-by-pixel (nearest-neighbor scaling)
                let screen_w = framebuffer::width();
                let screen_h = framebuffer::height();
                
                for dy in 0..disp_h {
                    let screen_y = content_y + offset_y + dy as i32;
                    if screen_y < content_y || screen_y >= content_y + content_h as i32 { continue; }
                    if screen_y < 0 || screen_y >= screen_h as i32 { continue; }
                    
                    // Source row
                    let src_y = (dy * state.img_height) / disp_h.max(1);
                    if src_y >= state.img_height { continue; }
                    
                    for dx in 0..disp_w {
                        let screen_x = wx + offset_x + dx as i32;
                        if screen_x < wx + 2 || screen_x >= wx + ww as i32 - 2 { continue; }
                        if screen_x < 0 || screen_x >= screen_w as i32 { continue; }
                        
                        let src_x = (dx * state.img_width) / disp_w.max(1);
                        if src_x >= state.img_width { continue; }
                        
                        let pixel = state.pixels[(src_y * state.img_width + src_x) as usize];
                        // Skip fully transparent pixels
                        if (pixel >> 24) == 0 { continue; }
                        framebuffer::put_pixel_fast(screen_x as u32, screen_y as u32, pixel | 0xFF000000);
                    }
                }
                
                // ── Status bar ──
                let status_y = (content_y + content_h as i32) as u32;
                framebuffer::fill_rect(safe_x + 2, status_y, ww.saturating_sub(4), 24, 0xFF0A1A12);
                framebuffer::draw_hline(safe_x + 2, status_y, ww.saturating_sub(4), 0xFF1A2A1A);
                
                let info = alloc::format!("{}x{} | Zoom: {}% | +/- to zoom | Arrows to pan", 
                    state.img_width, state.img_height, state.zoom);
                self.draw_text_smooth(wx + 10, status_y as i32 + 5, &info, GREEN_SUBTLE);
            } else {
                // No image data — show placeholder
                self.draw_text_smooth(wx + ww as i32 / 2 - 60, content_y + content_h as i32 / 2, "No image loaded", GREEN_GHOST);
                self.draw_text_smooth(wx + ww as i32 / 2 - 80, content_y + content_h as i32 / 2 + 20, "Open a .bmp file to view it", GREEN_GHOST);
            }
        } else {
            self.draw_text_smooth(wx + 20, content_y + 30, "Image Viewer — open a file", GREEN_GHOST);
        }
    }
    
    fn handle_image_viewer_key(&mut self, key: u8) {
        let win_id = match self.windows.iter().find(|w| w.focused && w.window_type == WindowType::ImageViewer) {
            Some(w) => w.id,
            None => return,
        };
        if let Some(state) = self.image_viewer_states.get_mut(&win_id) {
            match key {
                b'+' | b'=' => { state.zoom = (state.zoom + 10).min(500); }
                b'-' => { state.zoom = state.zoom.saturating_sub(10).max(10); }
                b'0' => { state.zoom = 100; state.pan_x = 0; state.pan_y = 0; } // Reset
                _ => {
                    if key == crate::keyboard::KEY_UP { state.pan_y += 20; }
                    else if key == crate::keyboard::KEY_DOWN { state.pan_y -= 20; }
                    else if key == crate::keyboard::KEY_LEFT { state.pan_x += 20; }
                    else if key == crate::keyboard::KEY_RIGHT { state.pan_x -= 20; }
                }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FILE MANAGER ICON/GRID VIEW
    // ═══════════════════════════════════════════════════════════════════════
    
    fn draw_file_manager_icon_grid(&self, window: &Window) {
        let wx = window.x;
        let wy = window.y;
        let ww = window.width;
        let wh = window.height;
        if ww < 80 || wh < 100 { return; }
        
        let content_y_start = wy + TITLE_BAR_HEIGHT as i32;
        let safe_x = if wx < 0 { 0u32 } else { wx as u32 };
        
        // Sidebar offset
        let sidebar_w = self.fm_states.get(&window.id).map(|f| if f.sidebar_collapsed { 0u32 } else { f.sidebar_width }).unwrap_or(180);
        let grid_x = wx + sidebar_w as i32;
        let grid_w = ww.saturating_sub(sidebar_w);
        
        // Colors
        let bg_dark = 0xFF0A120Cu32;
        let bg_toolbar = 0xFF0C140Cu32;
        let bg_sidebar = 0xFF081008u32;
        let bg_selected = 0xFF0A3818u32;
        let text_file = 0xFF80CC90u32;
        let icon_folder = 0xFFDDAA30u32;
        let icon_file = 0xFF60AA80u32;
        let separator = 0xFF142014u32;
        
        // ── Toolbar (same as list view — reuse draw logic) ──
        let toolbar_h = 36u32;
        framebuffer::fill_rect(safe_x, content_y_start as u32, ww, toolbar_h, bg_toolbar);
        
        let btn_y = content_y_start + 7;
        let btn_sz = 22u32;
        // Back/Forward/Up buttons
        let can_back = self.fm_states.get(&window.id).map(|f| f.can_go_back()).unwrap_or(false);
        let back_c = if can_back { GREEN_SECONDARY } else { 0xFF1A2A1A };
        draw_rounded_rect(wx + 8, btn_y, btn_sz, btn_sz, 4, 0xFF101810);
        self.draw_text(wx + 14, btn_y + 4, "<", back_c);
        
        let can_fwd = self.fm_states.get(&window.id).map(|f| f.can_go_forward()).unwrap_or(false);
        let fwd_c = if can_fwd { GREEN_SECONDARY } else { 0xFF1A2A1A };
        draw_rounded_rect(wx + 34, btn_y, btn_sz, btn_sz, 4, 0xFF101810);
        self.draw_text(wx + 40, btn_y + 4, ">", fwd_c);
        
        draw_rounded_rect(wx + 60, btn_y, btn_sz, btn_sz, 4, 0xFF101810);
        draw_rounded_rect_border(wx + 60, btn_y, btn_sz, btn_sz, 4, GREEN_GHOST);
        self.draw_text(wx + 66, btn_y + 4, "^", GREEN_SUBTLE);
        
        // Path bar
        let path_x = wx + 90;
        let path_w = (ww as i32).saturating_sub(106);
        if path_w > 10 {
            draw_rounded_rect(path_x, btn_y, path_w as u32, btn_sz, 6, 0xFF080E08);
            draw_rounded_rect_border(path_x, btn_y, path_w as u32, btn_sz, 6, separator);
            let current_path = window.file_path.as_deref().unwrap_or("/");
            self.draw_text_smooth(path_x + 10, btn_y + 5, current_path, GREEN_PRIMARY);
        }
        
        framebuffer::draw_hline(safe_x, (content_y_start + toolbar_h as i32) as u32, ww, separator);
        
        // ── Sidebar (same as list view) ──
        let body_y = content_y_start + toolbar_h as i32 + 1;
        let body_h = wh.saturating_sub(TITLE_BAR_HEIGHT + toolbar_h + 1 + 26);
        
        if sidebar_w > 0 && body_h > 20 {
            framebuffer::fill_rect(safe_x, body_y as u32, sidebar_w, body_h, bg_sidebar);
            let mut sy = body_y + 8;
            let item_h = 24i32;
            let sx = wx + 6;
            let siw = sidebar_w.saturating_sub(12);
            
            self.draw_text_smooth(sx + 4, sy, "Quick Access", 0xFF3A7A4A);
            sy += 20;
            if let Some(fm) = self.fm_states.get(&window.id) {
                for (name, path) in fm.quick_access.iter() {
                    if sy + item_h > body_y + body_h as i32 - 40 { break; }
                    let is_current = window.file_path.as_deref() == Some(path.as_str());
                    if is_current {
                        draw_rounded_rect(sx, sy - 2, siw, item_h as u32, 4, 0xFF0C2810);
                        framebuffer::fill_rect(safe_x + 2, sy as u32, 3, (item_h - 4) as u32, GREEN_PRIMARY);
                    }
                    let ic_x = (sx + 12) as u32;
                    let ic_y = (sy + 2) as u32;
                    framebuffer::fill_rect(ic_x, ic_y, 6, 2, icon_folder);
                    framebuffer::fill_rect(ic_x, ic_y + 2, 12, 8, icon_folder);
                    let c = if is_current { GREEN_PRIMARY } else { 0xFF50AA60 };
                    self.draw_text_smooth(sx + 30, sy + 3, name, c);
                    sy += item_h;
                }
            }
            sy += 6;
            framebuffer::draw_hline(safe_x + 10, sy as u32, sidebar_w.saturating_sub(20), separator);
            sy += 10;
            self.draw_text_smooth(sx + 4, sy, "This PC", 0xFF3A7A4A);
            sy += 20;
            let drives = [("Local Disk (C:)", "/"), ("RAM Disk", "/tmp"), ("Devices", "/dev"), ("System", "/proc")];
            for (name, path) in &drives {
                if sy + item_h > body_y + body_h as i32 - 4 { break; }
                let is_current = window.file_path.as_deref() == Some(*path);
                if is_current {
                    draw_rounded_rect(sx, sy - 2, siw, item_h as u32, 4, 0xFF0C2810);
                    framebuffer::fill_rect(safe_x + 2, sy as u32, 3, (item_h - 4) as u32, GREEN_PRIMARY);
                }
                let c = if is_current { GREEN_PRIMARY } else { 0xFF50AA60 };
                self.draw_text_smooth(sx + 30, sy + 3, name, c);
                sy += item_h;
            }
            framebuffer::fill_rect(safe_x + sidebar_w - 1, body_y as u32, 1, body_h, separator);
        }
        
        // ── Grid area ──
        let grid_start_y = body_y as u32;
        let grid_area_h = body_h.saturating_sub(2);
        if grid_area_h < 8 { return; }
        framebuffer::fill_rect(grid_x.max(0) as u32, grid_start_y, grid_w, grid_area_h, bg_dark);
        
        // Grid parameters
        let icon_cell_w = 90u32;
        let icon_cell_h = 80u32;
        let cols = ((grid_w.saturating_sub(20)) / icon_cell_w).max(1);
        let padding_x = (grid_w.saturating_sub(cols * icon_cell_w)) / 2;
        
        // Parse file entries
        let file_start_idx = 5usize.min(window.content.len());
        let file_end_idx = if window.content.len() > file_start_idx + 2 { window.content.len() - 2 } else { window.content.len() };
        let file_entries: Vec<&str> = if file_end_idx > file_start_idx {
            window.content[file_start_idx..file_end_idx].iter().map(|s| s.as_str()).collect()
        } else { Vec::new() };
        
        if file_entries.is_empty() {
            self.draw_text_smooth(grid_x + 40, grid_start_y as i32 + 30, "This folder is empty.", GREEN_GHOST);
        }
        
        let max_visible_rows = (grid_area_h / icon_cell_h).max(1) as usize;
        let scroll_row = window.scroll_offset / cols as usize;
        
        for (idx, entry) in file_entries.iter().enumerate() {
            let row = idx / cols as usize;
            let col = idx % cols as usize;
            
            // Scroll check
            if row < scroll_row { continue; }
            let display_row = row - scroll_row;
            if display_row >= max_visible_rows { break; }
            
            let cell_x = grid_x.max(0) as u32 + padding_x + col as u32 * icon_cell_w;
            let cell_y = grid_start_y + display_row as u32 * icon_cell_h;
            if cell_y + icon_cell_h > grid_start_y + grid_area_h { break; }
            
            let is_selected = idx == window.selected_index;
            let is_dir = entry.contains("[D]");
            
            // Selection background
            if is_selected {
                draw_rounded_rect(cell_x as i32 + 4, cell_y as i32 + 2, icon_cell_w - 8, icon_cell_h - 4, 6, bg_selected);
                draw_rounded_rect_border(cell_x as i32 + 4, cell_y as i32 + 2, icon_cell_w - 8, icon_cell_h - 4, 6, 0xFF1A5A2A);
            }
            
            // Draw icon (larger in grid view)
            let icon_x = cell_x + (icon_cell_w - 32) / 2;
            let icon_y = cell_y + 6;
            if is_dir {
                // Large folder icon
                let fc = if is_selected { 0xFFEEBB40 } else { icon_folder };
                framebuffer::fill_rect(icon_x, icon_y, 16, 6, fc);
                framebuffer::fill_rect(icon_x, icon_y + 6, 32, 20, fc);
                framebuffer::fill_rect(icon_x + 2, icon_y + 10, 28, 14, 0xFF0A0A04);
                framebuffer::fill_rect(icon_x + 6, icon_y + 14, 16, 2, 0xFF302A10);
                framebuffer::fill_rect(icon_x + 6, icon_y + 18, 12, 2, 0xFF302A10);
            } else {
                // Large file icon — color-coded by extension
                let ext = Self::extract_name_from_entry(entry);
                let (fc, badge_color, ext_label) = if ext.ends_with(".rs") || ext.ends_with(".c") || ext.ends_with(".h") {
                    (if is_selected { 0xFFFFAA66 } else { 0xFFDD7733 }, 0xFFFF6633, "RS")
                } else if ext.ends_with(".txt") || ext.ends_with(".md") || ext.ends_with(".log") {
                    (if is_selected { 0xFF88BBEE } else { 0xFF4488CC }, 0xFF4488CC, if ext.ends_with(".md") { "MD" } else { "TXT" })
                } else if ext.ends_with(".toml") || ext.ends_with(".json") || ext.ends_with(".cfg") {
                    (if is_selected { 0xFFEEDD66 } else { 0xFFDDAA00 }, 0xFFDDAA00, "CFG")
                } else if ext.ends_with(".bmp") || ext.ends_with(".png") || ext.ends_with(".jpg") {
                    (if is_selected { 0xFF66DD88 } else { 0xFF33BB66 }, 0xFF33BB66, "IMG")
                } else if ext.ends_with(".wav") || ext.ends_with(".mp3") {
                    (if is_selected { 0xFFFF88CC } else { 0xFFEE55AA }, 0xFFEE55AA, "SND")
                } else if ext.ends_with(".sh") || ext.ends_with(".elf") {
                    (if is_selected { 0xFFCC88FF } else { 0xFF9966DD }, 0xFF9966DD, "EXE")
                } else {
                    (if is_selected { 0xFF80DD99 } else { icon_file }, 0xFF60AA80, "")
                };
                framebuffer::fill_rect(icon_x, icon_y, 28, 28, fc);
                framebuffer::fill_rect(icon_x + 18, icon_y, 10, 10, 0xFF0A140A);
                framebuffer::fill_rect(icon_x + 18, icon_y, 2, 10, fc);
                framebuffer::fill_rect(icon_x + 18, icon_y + 8, 10, 2, fc);
                framebuffer::fill_rect(icon_x + 3, icon_y + 12, 22, 14, 0xFF040A04);
                // Badge color stripe on left side
                framebuffer::fill_rect(icon_x, icon_y, 3, 28, badge_color);
                // File type hint
                if !ext_label.is_empty() {
                    self.draw_text((icon_x + 5) as i32, (icon_y + 15) as i32, ext_label, 0xFF203020);
                }
            }
            
            // Filename (centered, truncated)
            let name = Self::extract_name_from_entry(entry);
            let max_chars = (icon_cell_w / 8).min(10) as usize;
            let display_name: String = if name.len() > max_chars {
                let mut s: String = name.chars().take(max_chars - 2).collect();
                s.push_str("..");
                s
            } else {
                String::from(name)
            };
            let name_x = cell_x as i32 + (icon_cell_w as i32 - display_name.len() as i32 * 8) / 2;
            let name_y = (cell_y + icon_cell_h - 20) as i32;
            let name_color = if is_selected { GREEN_PRIMARY } else { text_file };
            self.draw_text_smooth(name_x, name_y, &display_name, name_color);
        }
        
        // ── Status bar ──
        let status_y = (wy + wh as i32).saturating_sub(24) as u32;
        framebuffer::fill_rect(safe_x, status_y, ww, 24, bg_toolbar);
        framebuffer::draw_hline(safe_x, status_y, ww, separator);
        let item_count = file_entries.len();
        let status_text = if item_count == 1 { String::from("1 item") } else { alloc::format!("{} items", item_count) };
        self.draw_text_smooth(grid_x + 10, status_y as i32 + 6, &status_text, 0xFF406850);
    }
    
    fn extract_name_from_entry(entry: &str) -> &str {
        let trimmed = entry.trim();
        if let Some(bracket_end) = trimmed.find(']') {
            let after_icon = if bracket_end + 1 < trimmed.len() { &trimmed[bracket_end + 1..] } else { "" };
            let parts: Vec<&str> = after_icon.split_whitespace().collect();
            if !parts.is_empty() { parts[0] } else { "???" }
        } else {
            trimmed
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FILE MANAGER MOUSE CLICK HANDLING
    // ═══════════════════════════════════════════════════════════════════════
    
    fn handle_file_manager_click(&mut self, x: i32, y: i32, window_id: u32) {
        let (wtype, wx, wy, ww, wh, file_path_opt, content_len, selected_idx) = {
            if let Some(w) = self.windows.iter().find(|w| w.id == window_id && w.window_type == WindowType::FileManager) {
                (w.window_type, w.x, w.y, w.width, w.height, w.file_path.clone(), w.content.len(), w.selected_index)
            } else { return; }
        };
        
        let content_y = wy + TITLE_BAR_HEIGHT as i32;
        let toolbar_h = 36i32;
        let sidebar_w = self.fm_states.get(&window_id).map(|f| if f.sidebar_collapsed { 0i32 } else { f.sidebar_width as i32 }).unwrap_or(180);
        
        // ── Toolbar buttons ──
        let btn_y = content_y + 7;
        let btn_sz = 22i32;
        
        // Back button (◄)
        if x >= wx + 8 && x < wx + 8 + btn_sz && y >= btn_y && y < btn_y + btn_sz {
            // Use history-based back navigation
            let back_path = self.fm_states.get_mut(&window_id).and_then(|f| f.go_back().map(|s| String::from(s)));
            if let Some(path) = back_path {
                self.navigate_file_manager_to(window_id, &path);
            }
            return;
        }
        // Forward button (►)
        if x >= wx + 34 && x < wx + 34 + btn_sz && y >= btn_y && y < btn_y + btn_sz {
            let fwd_path = self.fm_states.get_mut(&window_id).and_then(|f| f.go_forward().map(|s| String::from(s)));
            if let Some(path) = fwd_path {
                self.navigate_file_manager_to(window_id, &path);
            }
            return;
        }
        // Up button (▲)
        if x >= wx + 60 && x < wx + 60 + btn_sz && y >= btn_y && y < btn_y + btn_sz {
            self.navigate_file_manager("..");
            return;
        }
        
        // ── Search box click (focus toggle) ──
        let search_w = if ww > 400 { 180i32 } else if ww > 300 { 120i32 } else { 0i32 };
        if search_w > 0 {
            let sx = wx + ww as i32 - search_w - 8;
            if x >= sx && x < sx + search_w && y >= btn_y && y < btn_y + btn_sz {
                if let Some(fm) = self.fm_states.get_mut(&window_id) {
                    fm.search_focused = true;
                }
                return;
            } else {
                // Click outside search box → unfocus
                if let Some(fm) = self.fm_states.get_mut(&window_id) {
                    fm.search_focused = false;
                }
            }
        }
        
        // ── Column header clicks (sorting) ──
        let body_y = content_y + toolbar_h + 1;
        let col_h = 24i32;
        let content_x = wx + sidebar_w;
        let content_w = ww as i32 - sidebar_w;
        if y >= body_y && y < body_y + col_h && x >= content_x {
            let col_type_x = content_x + (content_w * 52 / 100);
            let col_size_x = content_x + (content_w * 68 / 100);
            let col_date_x = content_x + (content_w * 82 / 100);
            
            let clicked_col: u8 = if content_w > 420 && x >= col_date_x { 3 }
                else if content_w > 300 && x >= col_size_x { 2 }
                else if content_w > 200 && x >= col_type_x { 1 }
                else { 0 };
            
            if let Some(fm) = self.fm_states.get_mut(&window_id) {
                if fm.sort_column == clicked_col {
                    fm.sort_ascending = !fm.sort_ascending;
                } else {
                    fm.sort_column = clicked_col;
                    fm.sort_ascending = true;
                }
            }
            // Re-sort by refreshing
            let path = file_path_opt.clone().unwrap_or_else(|| String::from("/"));
            self.refresh_file_manager(&path);
            return;
        }
        
        // ── Status bar view mode buttons (bottom-right) ──
        let body_h = wh.saturating_sub(TITLE_BAR_HEIGHT + toolbar_h as u32 + 1 + 26);
        let status_y = content_y + toolbar_h + 1 + body_h as i32;
        if y >= status_y && y < status_y + 24 && ww > 300 {
            let vb_x = wx + ww as i32 - 120;
            let vb_w = 24i32;
            // List button
            if x >= vb_x && x < vb_x + vb_w {
                self.fm_view_modes.insert(window_id, FileManagerViewMode::List);
                return;
            }
            // Grid button
            if x >= vb_x + vb_w + 4 && x < vb_x + vb_w * 2 + 4 {
                self.fm_view_modes.insert(window_id, FileManagerViewMode::IconGrid);
                return;
            }
            // Details button
            if x >= vb_x + (vb_w + 4) * 2 && x < vb_x + (vb_w + 4) * 2 + vb_w {
                self.fm_view_modes.insert(window_id, FileManagerViewMode::Details);
                return;
            }
        }
        
        // ── Sidebar clicks (Quick Access and This PC) ──
        let body_y = content_y + toolbar_h + 1;
        if sidebar_w > 0 && x >= wx && x < wx + sidebar_w {
            let item_h = 24i32;
            let mut sy = body_y + 28; // After "Quick Access" header
            
            // Quick Access items
            if let Some(fm) = self.fm_states.get(&window_id) {
                let qa_paths: Vec<String> = fm.quick_access.iter().map(|(_, p)| p.clone()).collect();
                for (i, path) in qa_paths.iter().enumerate() {
                    if y >= sy && y < sy + item_h {
                        crate::serial_println!("[FM] Sidebar click: Quick Access -> {}", path);
                        self.navigate_file_manager_to(window_id, path);
                        return;
                    }
                    sy += item_h;
                }
            }
            
            // Skip separator space
            sy += 36; // gap + "This PC" header
            
            // This PC drive items
            let drives = ["/", "/tmp", "/dev", "/proc"];
            for path in &drives {
                if y >= sy && y < sy + item_h {
                    crate::serial_println!("[FM] Sidebar click: Drive -> {}", path);
                    self.navigate_file_manager_to(window_id, path);
                    return;
                }
                sy += item_h;
            }
            return; // Click was in sidebar but didn't hit an item
        }
        
        // ── Click on file in content area ──
        let is_grid = self.fm_view_modes.get(&window_id).copied().unwrap_or(FileManagerViewMode::List) == FileManagerViewMode::IconGrid;
        let file_start_idx = 5usize.min(content_len);
        let file_end_idx = if content_len > file_start_idx + 2 { content_len - 2 } else { content_len };
        let file_count = file_end_idx.saturating_sub(file_start_idx);
        
        if is_grid {
            // Grid view click
            let grid_start_y = content_y + toolbar_h + 1;
            let icon_cell_w = 90i32;
            let icon_cell_h = 80i32;
            let content_x = wx + sidebar_w;
            let grid_w = ww as i32 - sidebar_w;
            let cols = ((grid_w - 20) / icon_cell_w).max(1);
            let padding_x = (grid_w - cols * icon_cell_w) / 2;
            let scroll_row = (self.windows.iter().find(|w| w.id == window_id).map(|w| w.scroll_offset).unwrap_or(0) / cols as usize) as i32;
            
            let rel_x = x - content_x - padding_x;
            let rel_y = y - grid_start_y;
            if rel_x >= 0 && rel_y >= 0 {
                let col = rel_x / icon_cell_w;
                let display_row = rel_y / icon_cell_h;
                let actual_row = display_row + scroll_row;
                let idx = actual_row * cols + col;
                if idx >= 0 && (idx as usize) < file_count {
                    let click_idx = idx as usize;
                    if click_idx == selected_idx && crate::mouse::is_double_click() {
                        self.open_selected_file_at(window_id, click_idx);
                        return;
                    }
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == window_id) {
                        w.selected_index = click_idx;
                    }
                }
            }
        } else {
            // List/Details view click
            let content_x = wx + sidebar_w;
            let col_h = 24i32;
            let list_start_y = body_y + col_h + 1;
            let row_h = 26i32;
            
            let rel_y = y - list_start_y;
            if rel_y >= 0 && x >= content_x {
                let scroll_offset = self.windows.iter().find(|w| w.id == window_id).map(|w| w.scroll_offset).unwrap_or(0);
                let click_idx = scroll_offset + (rel_y / row_h) as usize;
                if click_idx < file_count {
                    if click_idx == selected_idx && crate::mouse::is_double_click() {
                        self.open_selected_file_at(window_id, click_idx);
                        return;
                    }
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == window_id) {
                        w.selected_index = click_idx;
                    }
                }
            }
        }
    }
    
    /// Navigate a specific file manager window to a path (by window id)
    fn navigate_file_manager_to(&mut self, window_id: u32, path: &str) {
        // Focus this window, then use navigate logic
        let was_focused: Vec<u32> = self.windows.iter().filter(|w| w.focused).map(|w| w.id).collect();
        for w in &mut self.windows {
            w.focused = w.id == window_id;
        }
        // Set path directly and refresh
        if let Some(window) = self.windows.iter_mut().find(|w| w.id == window_id) {
            window.file_path = Some(String::from(path));
        }
        self.refresh_file_manager(path);
        // Push history
        if let Some(fm) = self.fm_states.get_mut(&window_id) {
            fm.push_history(path);
        }
        // Restore focus
        for w in &mut self.windows {
            w.focused = was_focused.contains(&w.id);
        }
    }
    
    fn open_selected_file_at(&mut self, window_id: u32, entry_idx: usize) {
        let (filename, is_dir) = {
            if let Some(w) = self.windows.iter().find(|w| w.id == window_id) {
                let file_start_idx = 5usize.min(w.content.len());
                let actual_idx = file_start_idx + entry_idx;
                if actual_idx < w.content.len().saturating_sub(2) {
                    let line = &w.content[actual_idx];
                    let is_dir = line.contains("[D]");
                    let name = Self::extract_name_from_entry(line);
                    (String::from(name), is_dir)
                } else { return; }
            } else { return; }
        };
        
        if is_dir {
            self.navigate_file_manager(&filename);
        } else {
            self.open_file(&filename);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DRAG AND DROP
    // ═══════════════════════════════════════════════════════════════════════
    
    fn start_file_drag(&mut self, window_id: u32) {
        if let Some(w) = self.windows.iter().find(|w| w.id == window_id && w.window_type == WindowType::FileManager) {
            let file_start_idx = 5usize.min(w.content.len());
            let actual_idx = file_start_idx + w.selected_index;
            if actual_idx < w.content.len().saturating_sub(2) {
                let line = &w.content[actual_idx];
                let is_dir = line.contains("[D]");
                let name = Self::extract_name_from_entry(line);
                if name == ".." { return; }
                let current_path = w.file_path.clone().unwrap_or_else(|| String::from("/"));
                let full_path = if current_path == "/" {
                    alloc::format!("/{}", name)
                } else {
                    alloc::format!("{}/{}", current_path, name)
                };
                self.drag_state = Some(DragState {
                    source_path: full_path,
                    filename: String::from(name),
                    is_dir,
                    start_x: self.cursor_x,
                    start_y: self.cursor_y,
                    current_x: self.cursor_x,
                    current_y: self.cursor_y,
                    source_window_id: window_id,
                    active: true,
                });
                crate::serial_println!("[DnD] Started drag: {}", name);
            }
        }
    }
    
    fn update_drag(&mut self, x: i32, y: i32) {
        if let Some(ref mut drag) = self.drag_state {
            drag.current_x = x;
            drag.current_y = y;
        }
    }
    
    fn finish_drag(&mut self, x: i32, y: i32) {
        let drag_info = self.drag_state.take();
        if let Some(drag) = drag_info {
            // Check if dropped on another FileManager window
            let target_window = self.windows.iter()
                .filter(|w| w.window_type == WindowType::FileManager && w.id != drag.source_window_id)
                .find(|w| x >= w.x && x < w.x + w.width as i32 && y >= w.y && y < w.y + w.height as i32);
            
            if let Some(target) = target_window {
                let target_path = target.file_path.clone().unwrap_or_else(|| String::from("/"));
                let dest_path = if target_path == "/" {
                    alloc::format!("/{}", drag.filename)
                } else {
                    alloc::format!("{}/{}", target_path, drag.filename)
                };
                
                // Copy file
                if !drag.is_dir {
                    if let Ok(data) = crate::ramfs::with_fs(|fs| fs.read_file(&drag.source_path).map(|d| d.to_vec())) {
                        let _ = crate::ramfs::with_fs(|fs| fs.write_file(&dest_path, &data));
                        crate::serial_println!("[DnD] Copied {} -> {}", drag.source_path, dest_path);
                    }
                } else {
                    let _ = crate::ramfs::with_fs(|fs| fs.mkdir(&dest_path));
                    crate::serial_println!("[DnD] Created dir: {}", dest_path);
                }
                
                // Refresh target file manager
                self.refresh_file_manager_by_id(target.id, &target_path);
            } else if y >= (self.height - TASKBAR_HEIGHT) as i32 {
                // Dropped on taskbar — ignore
                crate::serial_println!("[DnD] Dropped on taskbar, ignoring");
            } else {
                // Dropped on desktop — create desktop shortcut path
                crate::serial_println!("[DnD] Dropped on desktop: {}", drag.filename);
            }
        }
    }
    
    fn draw_drag_ghost(&self) {
        if let Some(ref drag) = self.drag_state {
            if !drag.active { return; }
            let gx = drag.current_x;
            let gy = drag.current_y;
            
            // Semi-transparent ghost icon
            framebuffer::fill_rect_alpha(gx as u32, gy as u32, 70, 22, 0x0C1410, 180);
            draw_rounded_rect_border(gx, gy, 70, 22, 4, GREEN_PRIMARY);
            
            // Icon
            if drag.is_dir {
                framebuffer::fill_rect((gx + 4) as u32, (gy + 4) as u32, 14, 14, 0xFFDDAA30);
            } else {
                framebuffer::fill_rect((gx + 4) as u32, (gy + 4) as u32, 12, 14, 0xFF60AA80);
            }
            
            // Filename
            let max_chars = 6;
            let name: String = drag.filename.chars().take(max_chars).collect();
            self.draw_text(gx + 22, gy + 5, &name, GREEN_PRIMARY);
        }
    }
    
    fn refresh_file_manager_by_id(&mut self, wid: u32, path: &str) {
        // Temporarily focus this window to use refresh_file_manager
        let was_focused: Vec<u32> = self.windows.iter().filter(|w| w.focused).map(|w| w.id).collect();
        for w in &mut self.windows {
            w.focused = w.id == wid;
        }
        self.refresh_file_manager(path);
        // Restore focus
        for w in &mut self.windows {
            w.focused = was_focused.contains(&w.id);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FILE CLIPBOARD (Ctrl+C/X/V in file manager)
    // ═══════════════════════════════════════════════════════════════════════
    
    fn file_clipboard_copy(&mut self, cut: bool) {
        if let Some(w) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager) {
            let file_start_idx = 5usize.min(w.content.len());
            let actual_idx = file_start_idx + w.selected_index;
            if actual_idx < w.content.len().saturating_sub(2) {
                let line = &w.content[actual_idx];
                let name = Self::extract_name_from_entry(line);
                if name == ".." { return; }
                let current_path = w.file_path.clone().unwrap_or_else(|| String::from("/"));
                let full_path = if current_path == "/" {
                    alloc::format!("/{}", name)
                } else {
                    alloc::format!("{}/{}", current_path, name)
                };
                let op = if cut { "Cut" } else { "Copied" };
                crate::serial_println!("[FM] {} file: {}", op, full_path);
                self.file_clipboard = Some(FileClipboardEntry {
                    path: full_path,
                    name: String::from(name),
                    is_cut: cut,
                });
                // Also set text clipboard
                crate::keyboard::clipboard_set(name);
            }
        }
    }
    
    fn file_clipboard_paste(&mut self) {
        let clipboard = self.file_clipboard.take();
        if let Some(entry) = clipboard {
            let current_path = self.windows.iter()
                .find(|w| w.focused && w.window_type == WindowType::FileManager)
                .and_then(|w| w.file_path.clone())
                .unwrap_or_else(|| String::from("/"));
            
            let dest = if current_path == "/" {
                alloc::format!("/{}", entry.name)
            } else {
                alloc::format!("{}/{}", current_path, entry.name)
            };
            
            if entry.is_cut {
                // Move: rename
                let _ = crate::ramfs::with_fs(|fs| fs.mv(&entry.path, &dest));
                crate::serial_println!("[FM] Moved {} -> {}", entry.path, dest);
            } else {
                // Copy: read + write
                if let Ok(data) = crate::ramfs::with_fs(|fs| fs.read_file(&entry.path).map(|d| d.to_vec())) {
                    let _ = crate::ramfs::with_fs(|fs| fs.write_file(&dest, &data));
                    crate::serial_println!("[FM] Pasted {} -> {}", entry.path, dest);
                }
                // Put back in clipboard for repeated paste
                self.file_clipboard = Some(entry);
            }
            
            self.refresh_file_manager(&current_path);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // LOCK SCREEN
    // ═══════════════════════════════════════════════════════════════════════
    
    fn draw_lock_screen(&self) {
        let sw = self.width;
        let sh = self.height;
        
        // Full-screen dark overlay
        framebuffer::fill_rect(0, 0, sw, sh, 0xFF040808);
        
        // Matrix-style rain effect (subtle, using frame_count)
        let cols = sw / 10;
        for c in 0..cols {
            let seed = c.wrapping_mul(7919).wrapping_add(self.frame_count as u32);
            let col_h = (seed % 20) + 3;
            let col_x = c * 10;
            let col_y_start = (seed.wrapping_mul(13) % sh) as i32;
            for r in 0..col_h {
                let ry = col_y_start + r as i32 * 14;
                if ry >= 0 && ry < sh as i32 - 14 {
                    let brightness = (255 - r * 12).max(20);
                    let color = (brightness << 8) | 0xFF000000;
                    let ch_val = ((seed.wrapping_add(r)) % 26 + 65) as u8 as char;
                    let mut buf = [0u8; 4];
                    let ch_str = ch_val.encode_utf8(&mut buf);
                    framebuffer::draw_text(ch_str, col_x, ry as u32, color);
                }
            }
        }
        
        // Center panel
        let panel_w = 360u32;
        let panel_h = 280u32;
        let px = (sw - panel_w) / 2;
        let py = (sh - panel_h) / 2;
        let shake_off = if self.lock_screen_shake > 0 {
            let amplitude = (self.lock_screen_shake as i32 * 3) % 13 - 6;
            amplitude
        } else { 0 };
        let px = (px as i32 + shake_off) as u32;
        
        // Panel background (frosted glass)
        framebuffer::fill_rect_alpha(px, py, panel_w, panel_h, 0x0C1A12, 200);
        draw_rounded_rect(px as i32, py as i32, panel_w, panel_h, 12, 0xFF0A1A0F);
        draw_rounded_rect_border(px as i32, py as i32, panel_w, panel_h, 12, GREEN_MUTED);
        
        // TrustOS logo text
        let title_x = (px + panel_w / 2).saturating_sub(40) as i32;
        self.draw_text_smooth(title_x, (py + 30) as i32, "TrustOS", GREEN_PRIMARY);
        
        // Lock icon (padlock shape)
        let lock_x = px + panel_w / 2 - 12;
        let lock_y = py + 70;
        // Shackle (arc)
        framebuffer::fill_rect(lock_x, lock_y, 24, 3, GREEN_SUBTLE);
        framebuffer::fill_rect(lock_x, lock_y, 3, 16, GREEN_SUBTLE);
        framebuffer::fill_rect(lock_x + 21, lock_y, 3, 16, GREEN_SUBTLE);
        // Body
        framebuffer::fill_rect(lock_x - 4, lock_y + 16, 32, 22, GREEN_MUTED);
        framebuffer::fill_rect(lock_x + 8, lock_y + 22, 8, 10, 0xFF040A08);
        
        // "Locked" text
        let lock_text_x = (px + panel_w / 2).saturating_sub(24) as i32;
        self.draw_text_smooth(lock_text_x, (lock_y + 48) as i32, "Locked", GREEN_TERTIARY);
        
        // Clock (large)
        let time = &self.cached_time_str;
        if !time.is_empty() {
            let time_x = (px + panel_w / 2).saturating_sub((time.len() as u32 * 8) / 2) as i32;
            self.draw_text_smooth(time_x, (py + 150) as i32, time, GREEN_PRIMARY);
        }
        let date = &self.cached_date_str;
        if !date.is_empty() {
            let date_x = (px + panel_w / 2).saturating_sub((date.len() as u32 * 8) / 2) as i32;
            self.draw_text_smooth(date_x, (py + 170) as i32, date, GREEN_TERTIARY);
        }
        
        // PIN input field
        let input_y = py + 200;
        let input_w = 200u32;
        let input_x = px + (panel_w - input_w) / 2;
        draw_rounded_rect(input_x as i32, input_y as i32, input_w, 30, 6, 0xFF081208);
        draw_rounded_rect_border(input_x as i32, input_y as i32, input_w, 30, 6, GREEN_GHOST);
        
        // Show dots for each character entered
        let dots: String = self.lock_screen_input.chars().map(|_| '*').collect();
        if dots.is_empty() {
            self.draw_text_smooth((input_x + 8) as i32, (input_y + 8) as i32, "Enter PIN...", GREEN_GHOST);
        } else {
            self.draw_text_smooth((input_x + 8) as i32, (input_y + 8) as i32, &dots, GREEN_PRIMARY);
        }
        
        // Cursor blink
        if self.cursor_blink {
            let cx = input_x + 8 + dots.len() as u32 * 8;
            framebuffer::fill_rect(cx, input_y + 6, 2, 18, GREEN_PRIMARY);
        }
        
        // Hint
        self.draw_text_smooth((px + panel_w / 2 - 80) as i32, (input_y + 42) as i32, "Press Enter to unlock", GREEN_GHOST);
        
        // Wrong PIN message
        if self.lock_screen_shake > 0 {
            self.draw_text_smooth((px + panel_w / 2 - 50) as i32, (input_y + 60) as i32, "Wrong PIN!", 0xFFCC4444);
        }
    }
    
    fn handle_lock_screen_key(&mut self, key: u8) {
        if self.lock_screen_shake > 0 {
            self.lock_screen_shake = self.lock_screen_shake.saturating_sub(1);
        }
        
        if key == 0x0D || key == 0x0A { // Enter
            // PIN is "0000" or empty (for demo, any input unlocks)
            if self.lock_screen_input.is_empty() || self.lock_screen_input == "0000" || self.lock_screen_input == "1234" {
                self.lock_screen_active = false;
                self.lock_screen_input.clear();
                crate::serial_println!("[LOCK] Screen unlocked");
            } else {
                // Wrong PIN
                self.lock_screen_shake = 15;
                self.lock_screen_input.clear();
                crate::serial_println!("[LOCK] Wrong PIN");
            }
        } else if key == 0x08 { // Backspace
            self.lock_screen_input.pop();
        } else if key >= 0x20 && key < 0x7F && self.lock_screen_input.len() < 16 {
            self.lock_screen_input.push(key as char);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SYSTEM TRAY INDICATORS (volume, battery, wifi)
    // ═══════════════════════════════════════════════════════════════════════
    
    fn draw_sys_tray_indicators(&self, tray_x: u32, tray_y: u32) {
        // ── WiFi indicator ──
        let wifi_x = tray_x;
        let wifi_y = tray_y + 2;
        let wifi_color = if self.sys_wifi_connected { GREEN_PRIMARY } else { 0xFF553333 };
        // Draw wifi arcs (3 arcs of increasing size)
        for arc in 0..3u32 {
            let r = 3 + arc * 3;
            let cx = wifi_x + 8;
            let cy = wifi_y + 12;
            // Quarter arc (top-right quadrant pointing up)
            for angle_step in 0..8u32 {
                let dx = (angle_step * r) / 8;
                let dy_sq = (r * r).saturating_sub(dx * dx);
                // Integer sqrt approximation
                let mut dy = 0u32;
                while (dy + 1) * (dy + 1) <= dy_sq { dy += 1; }
                let px = cx + dx;
                let py = cy.saturating_sub(dy);
                if px > 0 && py > 0 {
                    framebuffer::put_pixel_fast(px, py, wifi_color);
                    // Mirror to left
                    if cx >= dx {
                        framebuffer::put_pixel_fast(cx - dx, py, wifi_color);
                    }
                }
            }
        }
        // Center dot
        framebuffer::fill_rect(wifi_x + 7, wifi_y + 11, 3, 3, wifi_color);
        
        // ── Volume indicator ──
        let vol_x = tray_x + 22;
        let vol_y = tray_y + 3;
        let vol_color = GREEN_SUBTLE;
        // Speaker body
        framebuffer::fill_rect(vol_x, vol_y + 4, 4, 6, vol_color);
        framebuffer::fill_rect(vol_x + 4, vol_y + 2, 3, 10, vol_color);
        // Sound waves proportional to volume
        let waves = (self.sys_volume / 34).min(3); // 0-3 waves
        for w in 0..waves {
            let wx_off = vol_x + 9 + w * 3;
            let wh_half = 2 + w * 2;
            let wy_center = vol_y + 7;
            framebuffer::fill_rect(wx_off, wy_center.saturating_sub(wh_half), 1, wh_half * 2, vol_color);
        }
        if self.sys_volume == 0 {
            // Muted X
            framebuffer::fill_rect(vol_x + 9, vol_y + 3, 1, 8, 0xFFCC4444);
            framebuffer::fill_rect(vol_x + 12, vol_y + 3, 1, 8, 0xFFCC4444);
        }
        
        // ── Battery indicator ──
        let bat_x = tray_x + 44;
        let bat_y = tray_y + 4;
        let bat_w = 18u32;
        let bat_h = 8u32;
        // Battery outline
        framebuffer::draw_rect(bat_x, bat_y, bat_w, bat_h, GREEN_GHOST);
        // Battery tip
        framebuffer::fill_rect(bat_x + bat_w, bat_y + 2, 2, 4, GREEN_GHOST);
        // Fill level
        let fill_w = ((self.sys_battery as u32 * (bat_w - 2)) / 100).max(1);
        let bat_color = if self.sys_battery > 50 { GREEN_PRIMARY }
            else if self.sys_battery > 20 { ACCENT_AMBER }
            else { ACCENT_RED };
        framebuffer::fill_rect(bat_x + 1, bat_y + 1, fill_w, bat_h - 2, bat_color);
        
        // Battery % text
        let bat_str = alloc::format!("{}%", self.sys_battery);
        self.draw_text((bat_x + bat_w + 5) as i32, bat_y as i32, &bat_str, GREEN_GHOST);
    }

    /// Draw Windows Explorer-style file manager — full redesign
    fn draw_file_manager_gui(&self, window: &Window) {
        // Check view mode — dispatch to grid view if needed
        let view_mode = self.fm_view_modes.get(&window.id).copied().unwrap_or(FileManagerViewMode::List);
        if view_mode == FileManagerViewMode::IconGrid {
            self.draw_file_manager_icon_grid(window);
            return;
        }
        
        let wx = window.x;
        let wy = window.y;
        let ww = window.width;
        let wh = window.height;
        
        if ww < 120 || wh < 140 { return; }
        
        let content_y = wy + TITLE_BAR_HEIGHT as i32;
        let safe_x = wx.max(0) as u32;
        
        // ════════════════ COLORS (Windows 11-inspired + TrustOS green) ════════════════
        let bg_sidebar     = 0xFF081008u32;  // Very dark green-black sidebar
        let bg_sidebar_sel = 0xFF0C2810u32;  // Sidebar selected item
        let bg_sidebar_hov = 0xFF0A1C0Cu32;  // Sidebar hover
        let bg_content     = 0xFF0A120Cu32;  // Main content area
        let bg_toolbar     = 0xFF0C140Cu32;  // Toolbar bg
        let bg_header      = 0xFF0C180Eu32;  // Column header
        let bg_row_even    = 0xFF0A120Cu32;
        let bg_row_odd     = 0xFF0C140Cu32;
        let bg_row_hover   = 0xFF0E1E10u32;  // Row hover highlight
        let bg_selected    = 0xFF0A3818u32;  // Selected row
        let bg_search      = 0xFF060C06u32;  // Search box bg
        let text_sidebar   = 0xFF50AA60u32;
        let text_sidebar_h = 0xFF3A7A4Au32;  // Section header
        let text_header    = 0xFF50CC70u32;
        let text_file      = 0xFF80CC90u32;
        let text_dim       = 0xFF406850u32;
        let icon_folder    = 0xFFDDAA30u32;
        let icon_file      = 0xFF60AA80u32;
        let sep_color      = 0xFF142014u32;
        let accent         = GREEN_PRIMARY;
        
        // Get FM state
        let fm = self.fm_states.get(&window.id);
        let sidebar_w = fm.map(|f| if f.sidebar_collapsed { 0u32 } else { f.sidebar_width }).unwrap_or(180);
        let hover_idx = fm.and_then(|f| f.hover_index);
        
        // ════════════════ TOOLBAR (Windows 11 style nav bar) ════════════════
        let toolbar_h = 36u32;
        framebuffer::fill_rect(safe_x, content_y as u32, ww, toolbar_h, bg_toolbar);
        
        let btn_y = content_y + 7;
        let btn_sz = 22u32;
        
        // ── Back button ◄
        let can_back = fm.map(|f| f.can_go_back()).unwrap_or(false);
        let back_color = if can_back { GREEN_SECONDARY } else { 0xFF1A2A1A };
        draw_rounded_rect(wx + 8, btn_y, btn_sz, btn_sz, 4, 0xFF101810);
        if can_back { draw_rounded_rect_border(wx + 8, btn_y, btn_sz, btn_sz, 4, GREEN_GHOST); }
        self.draw_text(wx + 14, btn_y + 4, "<", back_color);
        
        // ── Forward button ►
        let can_fwd = fm.map(|f| f.can_go_forward()).unwrap_or(false);
        let fwd_color = if can_fwd { GREEN_SECONDARY } else { 0xFF1A2A1A };
        draw_rounded_rect(wx + 34, btn_y, btn_sz, btn_sz, 4, 0xFF101810);
        if can_fwd { draw_rounded_rect_border(wx + 34, btn_y, btn_sz, btn_sz, 4, GREEN_GHOST); }
        self.draw_text(wx + 40, btn_y + 4, ">", fwd_color);
        
        // ── Up button ▲
        draw_rounded_rect(wx + 60, btn_y, btn_sz, btn_sz, 4, 0xFF101810);
        draw_rounded_rect_border(wx + 60, btn_y, btn_sz, btn_sz, 4, GREEN_GHOST);
        self.draw_text(wx + 66, btn_y + 4, "^", GREEN_SUBTLE);
        
        // ── Breadcrumb path bar (Windows 11 style)
        let path_x = wx + 90;
        let search_w = if ww > 400 { 180i32 } else if ww > 300 { 120i32 } else { 0i32 };
        let path_w = (ww as i32 - 100 - search_w - 10).max(60);
        
        draw_rounded_rect(path_x, btn_y, path_w as u32, btn_sz, 6, 0xFF080E08);
        draw_rounded_rect_border(path_x, btn_y, path_w as u32, btn_sz, 6, sep_color);
        
        // Draw breadcrumb path segments
        let current_path = window.file_path.as_deref().unwrap_or("/");
        let mut px = path_x + 10;
        let parts: Vec<&str> = current_path.split('/').filter(|s| !s.is_empty()).collect();
        
        // Root icon
        self.draw_text_smooth(px, btn_y + 5, "\x07", 0xFF40AA50); // "drive" icon
        px += 12;
        
        if parts.is_empty() {
            self.draw_text_smooth(px, btn_y + 5, "This PC", accent);
        } else {
            self.draw_text_smooth(px, btn_y + 5, "This PC", GREEN_SUBTLE);
            px += 56;
            for (i, part) in parts.iter().enumerate() {
                if px > path_x + path_w - 30 { 
                    self.draw_text_smooth(px, btn_y + 5, "...", GREEN_GHOST);
                    break; 
                }
                // Separator chevron ›
                self.draw_text_smooth(px, btn_y + 5, ">", 0xFF2A4A30);
                px += 12;
                let is_last = i == parts.len() - 1;
                let c = if is_last { accent } else { GREEN_SUBTLE };
                self.draw_text_smooth(px, btn_y + 5, part, c);
                px += (part.len() as i32) * 8 + 6;
            }
        }
        
        // ── Search box (Windows 11 style, right side)
        if search_w > 0 {
            let sx = wx + ww as i32 - search_w - 8;
            let search_focused = fm.map(|f| f.search_focused).unwrap_or(false);
            let sb_bg = if search_focused { 0xFF081008 } else { bg_search };
            let sb_border = if search_focused { accent } else { sep_color };
            draw_rounded_rect(sx, btn_y, search_w as u32, btn_sz, 6, sb_bg);
            draw_rounded_rect_border(sx, btn_y, search_w as u32, btn_sz, 6, sb_border);
            // Search icon (magnifying glass)
            self.draw_text_smooth(sx + 8, btn_y + 5, "\x0F", if search_focused { accent } else { GREEN_GHOST });
            let query = fm.map(|f| f.search_query.as_str()).unwrap_or("");
            if query.is_empty() {
                self.draw_text_smooth(sx + 22, btn_y + 5, "Search", text_dim);
            } else {
                self.draw_text_smooth(sx + 22, btn_y + 5, query, text_file);
            }
            // Blinking cursor when focused
            if search_focused && (self.frame_count / 30) % 2 == 0 {
                let cursor_x = sx + 22 + (query.len() as i32) * 8;
                framebuffer::fill_rect(cursor_x as u32, (btn_y + 4) as u32, 1, 14, accent);
            }
        }
        
        // ── Toolbar bottom separator
        framebuffer::draw_hline(safe_x, (content_y + toolbar_h as i32) as u32, ww, sep_color);
        
        // ════════════════ SIDEBAR (Windows 11 Navigation Pane) ════════════════
        let body_y = content_y + toolbar_h as i32 + 1;
        let body_h = wh.saturating_sub(TITLE_BAR_HEIGHT + toolbar_h + 1 + 26); // 26 = status bar
        
        if sidebar_w > 0 && body_h > 20 {
            framebuffer::fill_rect(safe_x, body_y as u32, sidebar_w, body_h, bg_sidebar);
            
            let mut sy = body_y + 8;
            let item_h = 24i32;
            let sidebar_x = wx + 6;
            let sidebar_inner_w = sidebar_w.saturating_sub(12);
            
            // ── Quick Access section
            self.draw_text_smooth(sidebar_x + 4, sy, "Quick Access", text_sidebar_h);
            // Expand/collapse chevron
            self.draw_text_smooth(sidebar_x as i32 + sidebar_inner_w as i32 - 8, sy, "v", text_sidebar_h);
            sy += 20;
            
            if let Some(fm_s) = fm {
                for (i, (name, path)) in fm_s.quick_access.iter().enumerate() {
                    if sy + item_h > body_y + body_h as i32 - 40 { break; }
                    
                    let is_current = window.file_path.as_deref() == Some(path.as_str());
                    let is_hovered = fm_s.sidebar_selected == i as i32;
                    
                    // Highlight current/hovered
                    let row_bg = if is_current { bg_sidebar_sel } else if is_hovered { bg_sidebar_hov } else { bg_sidebar };
                    if is_current || is_hovered {
                        draw_rounded_rect(sidebar_x, sy - 2, sidebar_inner_w, item_h as u32, 4, row_bg);
                    }
                    // Left accent bar for current
                    if is_current {
                        framebuffer::fill_rect(safe_x + 2, sy as u32, 3, (item_h - 4) as u32, accent);
                    }
                    
                    // Folder icon (small)
                    let ic_x = (sidebar_x + 12) as u32;
                    let ic_y = (sy + 2) as u32;
                    framebuffer::fill_rect(ic_x, ic_y, 6, 2, icon_folder);
                    framebuffer::fill_rect(ic_x, ic_y + 2, 12, 8, icon_folder);
                    framebuffer::fill_rect(ic_x + 1, ic_y + 4, 10, 5, 0xFF0A0A04);
                    
                    let name_color = if is_current { accent } else { text_sidebar };
                    self.draw_text_smooth(sidebar_x + 30, sy + 3, name, name_color);
                    
                    sy += item_h;
                }
            }
            
            // ── Separator
            sy += 6;
            framebuffer::draw_hline(safe_x + 10, sy as u32, sidebar_w.saturating_sub(20), sep_color);
            sy += 10;
            
            // ── This PC section
            self.draw_text_smooth(sidebar_x + 4, sy, "This PC", text_sidebar_h);
            sy += 20;
            
            // Drive items
            let drives = [
                ("\x07", "Local Disk (C:)", "/"),
                ("\x07", "RAM Disk",        "/tmp"),
                ("\x07", "Devices",         "/dev"),
                ("\x07", "System",          "/proc"),
            ];
            
            for (icon, name, path) in &drives {
                if sy + item_h > body_y + body_h as i32 - 4 { break; }
                let is_current = window.file_path.as_deref() == Some(*path);
                
                if is_current {
                    draw_rounded_rect(sidebar_x, sy - 2, sidebar_inner_w, item_h as u32, 4, bg_sidebar_sel);
                    framebuffer::fill_rect(safe_x + 2, sy as u32, 3, (item_h - 4) as u32, accent);
                }
                
                // Drive icon (small disk)
                let ic_x = (sidebar_x + 12) as u32;
                let ic_y = (sy + 2) as u32;
                framebuffer::fill_rect(ic_x, ic_y, 12, 10, 0xFF406050);
                framebuffer::fill_rect(ic_x + 1, ic_y + 1, 10, 3, 0xFF60AA80);
                framebuffer::fill_rect(ic_x + 4, ic_y + 5, 4, 3, 0xFF80CC90);
                
                let c = if is_current { accent } else { text_sidebar };
                self.draw_text_smooth(sidebar_x + 30, sy + 3, name, c);
                sy += item_h;
            }
            
            // ── Vertical separator line between sidebar and content
            framebuffer::fill_rect(safe_x + sidebar_w - 1, body_y as u32, 1, body_h, sep_color);
        }
        
        // ════════════════ MAIN CONTENT AREA ════════════════
        let content_x = wx + sidebar_w as i32;
        let content_w = ww.saturating_sub(sidebar_w);
        
        // ── Column headers (Windows 11 style)
        let col_h = 24u32;
        framebuffer::fill_rect((content_x.max(0)) as u32, body_y as u32, content_w, col_h, bg_header);
        
        // Column positions (proportional)
        let col_name_x = content_x + 36;
        let col_type_x = content_x + (content_w as i32 * 52 / 100);
        let col_size_x = content_x + (content_w as i32 * 68 / 100);
        let col_date_x = content_x + (content_w as i32 * 82 / 100);
        
        let hy = body_y + 5;
        
        // Sort indicator
        let sort_col = fm.map(|f| f.sort_column).unwrap_or(0);
        let sort_asc = fm.map(|f| f.sort_ascending).unwrap_or(true);
        let sort_arrow = if sort_asc { "v" } else { "^" };
        
        // Name header
        self.draw_text_smooth(col_name_x, hy, "Name", text_header);
        if sort_col == 0 { self.draw_text_smooth(col_name_x + 40, hy, sort_arrow, GREEN_GHOST); }
        
        if content_w > 200 {
            framebuffer::fill_rect(col_type_x as u32 - 2, body_y as u32 + 4, 1, col_h - 8, sep_color);
            self.draw_text_smooth(col_type_x, hy, "Type", text_header);
            if sort_col == 1 { self.draw_text_smooth(col_type_x + 36, hy, sort_arrow, GREEN_GHOST); }
        }
        if content_w > 300 {
            framebuffer::fill_rect(col_size_x as u32 - 2, body_y as u32 + 4, 1, col_h - 8, sep_color);
            self.draw_text_smooth(col_size_x, hy, "Size", text_header);
            if sort_col == 2 { self.draw_text_smooth(col_size_x + 36, hy, sort_arrow, GREEN_GHOST); }
        }
        if content_w > 420 {
            framebuffer::fill_rect(col_date_x as u32 - 2, body_y as u32 + 4, 1, col_h - 8, sep_color);
            self.draw_text_smooth(col_date_x, hy, "Open with", text_header);
        }
        
        framebuffer::draw_hline((content_x.max(0)) as u32, (body_y + col_h as i32) as u32, content_w, sep_color);
        
        // ── File list area
        let list_y = body_y + col_h as i32 + 1;
        let list_h = body_h.saturating_sub(col_h + 27); // reserve for status bar
        if list_h < 8 { return; }
        
        framebuffer::fill_rect((content_x.max(0)) as u32, list_y as u32, content_w, list_h, bg_content);
        
        let row_h = 26u32; // Slightly taller rows like Windows 11
        let max_visible = (list_h / row_h).max(1) as usize;
        
        // Parse file entries
        let file_start_idx = 5usize.min(window.content.len());
        let file_end_idx = if window.content.len() > file_start_idx + 2 { window.content.len() - 2 } else { window.content.len() };
        let file_entries: Vec<&str> = if file_end_idx > file_start_idx {
            window.content[file_start_idx..file_end_idx].iter().map(|s| s.as_str()).collect()
        } else { Vec::new() };
        
        if file_entries.is_empty() {
            self.draw_text_smooth(content_x + 30, list_y + 30, "This folder is empty.", text_dim);
            self.draw_text_smooth(content_x + 30, list_y + 50, "Press N to create a file, D for a folder.", GREEN_GHOST);
        }
        
        let scroll = window.scroll_offset;
        let visible_count = file_entries.len().min(max_visible);
        
        for vi in 0..visible_count {
            let entry_idx = scroll + vi;
            if entry_idx >= file_entries.len() { break; }
            let line = file_entries[entry_idx];
            let ry = list_y as u32 + (vi as u32) * row_h;
            if ry + row_h > list_y as u32 + list_h { break; }
            
            let is_selected = entry_idx == window.selected_index;
            let is_dir = line.contains("[D]");
            let is_hovered = hover_idx == Some(entry_idx);
            
            // ── Row background
            let row_bg = if is_selected {
                bg_selected
            } else if is_hovered {
                bg_row_hover
            } else if vi % 2 == 0 {
                bg_row_even
            } else {
                bg_row_odd
            };
            framebuffer::fill_rect((content_x.max(0)) as u32, ry, content_w, row_h, row_bg);
            
            // ── Selection highlight (Windows 11: subtle rounded highlight + left accent)
            if is_selected {
                // Left accent bar (blue in Windows, green in TrustOS)
                framebuffer::fill_rect((content_x.max(0)) as u32, ry + 3, 3, row_h - 6, accent);
                // Subtle border around selected row
                draw_rounded_rect_border(content_x, ry as i32, content_w, row_h, 3, 0xFF1A4A28);
            }
            
            let text_y = (ry + 6) as i32;
            let row_text_color = if is_selected { accent } else { text_file };
            
            // ── Draw icon (Windows 11 style: larger, more detailed)
            let ix = (content_x + 10).max(0) as u32;
            let iy = ry + 3;
            let icon_sz = 20u32;
            
            if is_dir {
                // Folder icon — Windows 11 style yellow folder
                let fc = if is_selected { 0xFFEECC50 } else { icon_folder };
                let fc_dark = if is_selected { 0xFFCCAA30 } else { 0xFFBB8820 };
                // Tab part
                framebuffer::fill_rect(ix, iy, icon_sz / 2, 4, fc);
                // Body  
                framebuffer::fill_rect(ix, iy + 4, icon_sz, icon_sz - 4, fc);
                // Inner shadow
                framebuffer::fill_rect(ix + 2, iy + 7, icon_sz - 4, icon_sz - 10, fc_dark);
                // Doc line hints
                framebuffer::fill_rect(ix + 4, iy + 9, icon_sz - 8, 1, 0xFF0A0A04);
                framebuffer::fill_rect(ix + 4, iy + 12, icon_sz / 2, 1, 0xFF0A0A04);
            } else {
                // File icon — Windows 11 style document
                let fc = if is_selected { 0xFF80DDAA } else { icon_file };
                let fc_dark = 0xFF0A140A;
                // Main body
                framebuffer::fill_rect(ix + 2, iy, icon_sz - 6, icon_sz, fc);
                // Folded corner (top-right)
                framebuffer::fill_rect(ix + icon_sz - 8, iy, 4, 6, fc_dark);
                framebuffer::fill_rect(ix + icon_sz - 8, iy, 1, 6, fc);
                framebuffer::fill_rect(ix + icon_sz - 8, iy + 5, 4, 1, fc);
                // Inner area  
                framebuffer::fill_rect(ix + 4, iy + 8, icon_sz - 10, icon_sz - 10, fc_dark);
                // Text lines inside file
                framebuffer::fill_rect(ix + 5, iy + 10, 8, 1, 0xFF1A3A1A);
                framebuffer::fill_rect(ix + 5, iy + 13, 6, 1, 0xFF1A3A1A);
                framebuffer::fill_rect(ix + 5, iy + 16, 7, 1, 0xFF1A3A1A);
                
                // File type color badge (small colored square for extension)
                let ext_name = Self::extract_name_from_entry(line);
                let badge_color = if ext_name.ends_with(".rs") { 0xFFFF6633 }       // Rust orange
                    else if ext_name.ends_with(".txt") { 0xFF4488CC }               // Blue text
                    else if ext_name.ends_with(".md") { 0xFF5599DD }                // Markdown blue
                    else if ext_name.ends_with(".toml") { 0xFF8866BB }              // Purple config
                    else if ext_name.ends_with(".json") { 0xFFDDAA00 }              // Yellow json
                    else if ext_name.ends_with(".html") || ext_name.ends_with(".htm") { 0xFFEE6633 }
                    else if ext_name.ends_with(".css") { 0xFF3399EE }
                    else if ext_name.ends_with(".png") || ext_name.ends_with(".jpg") || ext_name.ends_with(".bmp") { 0xFF33BB66 }
                    else if ext_name.ends_with(".mp3") || ext_name.ends_with(".wav") { 0xFFEE55AA }
                    else { 0xFF446644 };
                framebuffer::fill_rect(ix + 3, iy + icon_sz - 5, 6, 4, badge_color);
            }
            
            // ── Parse entry fields
            let trimmed = line.trim();
            let (name_str, type_str, size_str, prog_str) = if let Some(bracket_end) = trimmed.find(']') {
                let after_icon = if bracket_end + 1 < trimmed.len() { &trimmed[bracket_end + 1..] } else { "" };
                let parts: Vec<&str> = after_icon.split_whitespace().collect();
                (
                    if !parts.is_empty() { parts[0] } else { "???" },
                    if parts.len() > 1 { parts[1] } else { "" },
                    if parts.len() > 2 { parts[2] } else { "" },
                    if parts.len() > 3 { parts[3] } else { "" },
                )
            } else {
                (trimmed, "", "", "")
            };
            
            // ── Draw columns
            // Name (with extension dimmed like Windows)
            let name_x = content_x + 36;
            if let Some(dot_pos) = name_str.rfind('.') {
                let base = &name_str[..dot_pos];
                let ext = &name_str[dot_pos..];
                self.draw_text_smooth(name_x, text_y, base, row_text_color);
                let ext_x = name_x + (base.len() as i32) * 8;
                self.draw_text_smooth(ext_x, text_y, ext, if is_selected { GREEN_SUBTLE } else { text_dim });
            } else {
                self.draw_text_smooth(name_x, text_y, name_str, row_text_color);
            }
            
            // Type column
            if content_w > 200 {
                let type_label = if is_dir { "File folder" } else {
                    match name_str.rsplit('.').next() {
                        Some("rs") => "Rust Source",
                        Some("txt") => "Text Document",
                        Some("md") => "Markdown",
                        Some("toml") => "TOML Config",
                        Some("json") => "JSON File",
                        Some("html") | Some("htm") => "HTML Document",
                        Some("css") => "Stylesheet",
                        Some("png") | Some("jpg") | Some("bmp") => "Image",
                        Some("mp3") | Some("wav") => "Audio",
                        Some("sh") => "Shell Script",
                        _ => type_str,
                    }
                };
                let tc = if is_selected { GREEN_SUBTLE } else { 0xFF50886A };
                self.draw_text_smooth(col_type_x, text_y, type_label, tc);
            }
            
            // Size column (human-readable like Windows)
            if content_w > 300 {
                let size_display = if is_dir {
                    String::from("")
                } else if let Ok(bytes) = size_str.parse::<u64>() {
                    if bytes < 1024 { alloc::format!("{} B", bytes) }
                    else if bytes < 1024 * 1024 { alloc::format!("{} KB", bytes / 1024) }
                    else { alloc::format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)) }
                } else {
                    String::from(size_str)
                };
                let sc = if is_selected { GREEN_SUBTLE } else { 0xFF50886A };
                self.draw_text_smooth(col_size_x, text_y, &size_display, sc);
            }
            
            // Open-with column
            if content_w > 420 {
                let pc = if is_selected { GREEN_GHOST } else { 0xFF406050 };
                self.draw_text_smooth(col_date_x, text_y, prog_str, pc);
            }
            
            // ── Row bottom line (very subtle, every row)
            framebuffer::draw_hline((content_x.max(0)) as u32, ry + row_h - 1, content_w, 0xFF0E160E);
        }
        
        // ── Scrollbar (Windows 11 thin style)
        if file_entries.len() > max_visible && list_h > 20 {
            let sb_w = 5u32;
            let sb_x = (content_x as u32 + content_w).saturating_sub(sb_w + 2);
            let track_h = list_h.saturating_sub(4);
            framebuffer::fill_rect(sb_x, list_y as u32 + 2, sb_w, track_h, 0xFF0A160C);
            let total = file_entries.len() as u32;
            let visible = max_visible as u32;
            let thumb_h = ((visible * track_h) / total.max(1)).max(20).min(track_h);
            let max_scroll = total.saturating_sub(visible);
            let thumb_y = if max_scroll > 0 {
                list_y as u32 + 2 + ((scroll as u32 * track_h.saturating_sub(thumb_h)) / max_scroll)
            } else {
                list_y as u32 + 2
            };
            draw_rounded_rect(sb_x as i32, thumb_y as i32, sb_w, thumb_h, 2, 0xFF204030);
        }
        
        // ════════════════ STATUS BAR (Windows 11 style) ════════════════
        let status_y = (body_y + body_h as i32) as u32;
        let status_h = 24u32;
        framebuffer::fill_rect(safe_x, status_y, ww, status_h, bg_toolbar);
        framebuffer::draw_hline(safe_x, status_y, ww, sep_color);
        
        // Item count
        let item_count = file_entries.len();
        let status_text = if item_count == 1 {
            String::from("1 item")
        } else {
            alloc::format!("{} items", item_count)
        };
        self.draw_text_smooth(wx + sidebar_w as i32 + 10, status_y as i32 + 6, &status_text, text_dim);
        
        // Selected item info
        if window.selected_index < file_entries.len() {
            let sel_name = Self::extract_name_from_entry(file_entries[window.selected_index]);
            if sel_name != ".." {
                let sel_info = alloc::format!("| {}", sel_name);
                self.draw_text_smooth(wx + sidebar_w as i32 + 80, status_y as i32 + 6, &sel_info, GREEN_GHOST);
            }
        }
        
        // View mode buttons (right side of status bar) — List/Grid/Details/Tiles
        if ww > 300 {
            let vb_y = status_y as i32 + 3;
            let vb_x = wx + ww as i32 - 120;
            let vb_w = 24i32;
            
            // List view button
            let list_active = view_mode == FileManagerViewMode::List;
            let list_c = if list_active { accent } else { GREEN_GHOST };
            draw_rounded_rect(vb_x, vb_y, vb_w as u32, 18, 3, if list_active { 0xFF102810 } else { 0xFF0A140A });
            // List icon (3 horizontal lines)
            framebuffer::fill_rect((vb_x + 5) as u32, (vb_y + 4) as u32, 14, 2, list_c);
            framebuffer::fill_rect((vb_x + 5) as u32, (vb_y + 8) as u32, 14, 2, list_c);
            framebuffer::fill_rect((vb_x + 5) as u32, (vb_y + 12) as u32, 14, 2, list_c);
            
            // Grid view button  
            let grid_active = view_mode == FileManagerViewMode::IconGrid;
            let grid_c = if grid_active { accent } else { GREEN_GHOST };
            draw_rounded_rect(vb_x + vb_w + 4, vb_y, vb_w as u32, 18, 3, if grid_active { 0xFF102810 } else { 0xFF0A140A });
            // Grid icon (4 squares)
            framebuffer::fill_rect((vb_x + vb_w + 8) as u32, (vb_y + 4) as u32, 6, 5, grid_c);
            framebuffer::fill_rect((vb_x + vb_w + 16) as u32, (vb_y + 4) as u32, 6, 5, grid_c);
            framebuffer::fill_rect((vb_x + vb_w + 8) as u32, (vb_y + 11) as u32, 6, 5, grid_c);
            framebuffer::fill_rect((vb_x + vb_w + 16) as u32, (vb_y + 11) as u32, 6, 5, grid_c);
            
            // Details view button
            let det_active = view_mode == FileManagerViewMode::Details;
            let det_c = if det_active { accent } else { GREEN_GHOST };
            draw_rounded_rect(vb_x + (vb_w + 4) * 2, vb_y, vb_w as u32, 18, 3, if det_active { 0xFF102810 } else { 0xFF0A140A });
            // Details icon (lines with dots)
            framebuffer::fill_rect((vb_x + (vb_w + 4) * 2 + 5) as u32, (vb_y + 4) as u32, 3, 2, det_c);
            framebuffer::fill_rect((vb_x + (vb_w + 4) * 2 + 10) as u32, (vb_y + 4) as u32, 8, 2, det_c);
            framebuffer::fill_rect((vb_x + (vb_w + 4) * 2 + 5) as u32, (vb_y + 8) as u32, 3, 2, det_c);
            framebuffer::fill_rect((vb_x + (vb_w + 4) * 2 + 10) as u32, (vb_y + 8) as u32, 8, 2, det_c);
            framebuffer::fill_rect((vb_x + (vb_w + 4) * 2 + 5) as u32, (vb_y + 12) as u32, 3, 2, det_c);
            framebuffer::fill_rect((vb_x + (vb_w + 4) * 2 + 10) as u32, (vb_y + 12) as u32, 8, 2, det_c);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // 🎵 MUSIC PLAYER — Polished glass widget with track list + visualization
    // ═══════════════════════════════════════════════════════════════════════════
    fn draw_music_player(&self, window: &Window) {
        let wx = window.x as u32;
        let wy = window.y as u32 + TITLE_BAR_HEIGHT;
        let ww = window.width;
        let wh = window.height.saturating_sub(TITLE_BAR_HEIGHT);

        if ww < 80 || wh < 80 { return; }

        // ── Glass background ──
        framebuffer::fill_rect_alpha(wx, wy, ww, wh, 0x060D0A, 210);
        // Subtle border glow
        framebuffer::fill_rect_alpha(wx + 1, wy + 1, ww - 2, 1, 0x00FF66, 30);
        framebuffer::fill_rect_alpha(wx + 1, wy + wh - 1, ww - 2, 1, 0x00FF66, 18);
        framebuffer::fill_rect_alpha(wx, wy + 1, 1, wh - 2, 0x00FF66, 22);
        framebuffer::fill_rect_alpha(wx + ww - 1, wy + 1, 1, wh - 2, 0x00FF66, 22);

        let state = match self.music_player_states.get(&window.id) {
            Some(s) => s,
            None => return,
        };

        let pad = 10u32;
        let inner_x = wx + pad;
        let inner_w = ww.saturating_sub(pad * 2);
        let cw = crate::graphics::scaling::char_width() as u32;

        // ═══════════════════════════════════════════════
        // TRACK LIST
        // ═══════════════════════════════════════════════
        let list_header_y = wy + 6;
        // Header: "LIBRARY" + track count
        self.draw_text(inner_x as i32, list_header_y as i32, "LIBRARY", 0xFF44886A);
        if state.num_tracks > 0 {
            let count_str = alloc::format!("{} tracks", state.num_tracks);
            let count_x = (inner_x + inner_w).saturating_sub(count_str.len() as u32 * cw);
            self.draw_text(count_x as i32, list_header_y as i32, &count_str, 0xFF336655);
        }

        let list_y = list_header_y + 16;
        let max_visible = 5usize;
        let row_h = 20u32;
        let list_h = if state.num_tracks == 0 { row_h } else { (state.num_tracks.min(max_visible) as u32) * row_h };

        // List background
        framebuffer::fill_rect_alpha(inner_x, list_y, inner_w, list_h, 0x0A1A12, 180);
        // List border
        framebuffer::fill_rect_alpha(inner_x, list_y, inner_w, 1, 0x00FF66, 18);
        framebuffer::fill_rect_alpha(inner_x, list_y + list_h - 1, inner_w, 1, 0x00FF66, 12);

        if state.num_tracks == 0 {
            self.draw_text(inner_x as i32 + 8, (list_y + 4) as i32, "No tracks found", 0xFF556655);
        } else {
            let scroll = state.track_list_scroll.min(state.num_tracks.saturating_sub(max_visible));
            for vi in 0..max_visible {
                let track_i = scroll + vi;
                if track_i >= state.num_tracks { break; }
                let ry = list_y + vi as u32 * row_h;
                let is_current = track_i == state.current_track && state.state != PlaybackState::Stopped;

                // Highlight current track
                if is_current {
                    framebuffer::fill_rect_alpha(inner_x + 1, ry + 1, inner_w - 2, row_h - 2, 0x00AA44, 40);
                }

                // Track number
                let num_str = alloc::format!("{}.", track_i + 1);
                let num_color = if is_current { 0xFF00FFAA } else { 0xFF446655 };
                self.draw_text(inner_x as i32 + 6, (ry + 3) as i32, &num_str, num_color);

                // Track name (truncate if too long)
                let name = if track_i < state.track_names.len() {
                    &state.track_names[track_i]
                } else {
                    "Unknown"
                };
                let max_chars = ((inner_w - 30) / cw) as usize;
                let display_name = if name.len() > max_chars {
                    &name[..max_chars.min(name.len())]
                } else {
                    name
                };
                let name_color = if is_current { 0xFF00FFCC } else { 0xFF88BBAA };
                self.draw_text(inner_x as i32 + 26, (ry + 3) as i32, display_name, name_color);

                // Playing indicator
                if is_current && state.state == PlaybackState::Playing {
                    self.draw_text(inner_x as i32 + inner_w as i32 - 14, (ry + 3) as i32, ">", 0xFF00FF88);
                }
            }
        }

        // ═══════════════════════════════════════════════
        // NOW PLAYING
        // ═══════════════════════════════════════════════
        let np_y = list_y + list_h + 10;
        self.draw_text(inner_x as i32, np_y as i32, "NOW PLAYING", 0xFF336655);

        // Song title (bold)
        let song_y = np_y + 16;
        let title = &state.song_title;
        self.draw_text(inner_x as i32, song_y as i32, title, 0xFF00FFAA);
        self.draw_text(inner_x as i32 + 1, song_y as i32, title, 0xFF00FFAA);

        // Status + time
        let status_y = song_y + 16;
        let status = match state.state {
            PlaybackState::Playing => "PLAYING",
            PlaybackState::Paused  => "PAUSED",
            PlaybackState::Stopped => "STOPPED",
        };
        let status_color = match state.state {
            PlaybackState::Playing => 0xFF00CC66,
            PlaybackState::Paused  => 0xFF00AA88,
            PlaybackState::Stopped => 0xFF666666,
        };
        self.draw_text(inner_x as i32, status_y as i32, status, status_color);

        // Time display (right-aligned)
        let elapsed_s = (state.elapsed_ms / 1000) as u32;
        let total_s = (state.total_ms / 1000) as u32;
        let time_str = alloc::format!(
            "{}:{:02} / {}:{:02}",
            elapsed_s / 60, elapsed_s % 60,
            total_s / 60, total_s % 60
        );
        let time_x = (inner_x + inner_w).saturating_sub(time_str.len() as u32 * cw);
        self.draw_text(time_x as i32, status_y as i32, &time_str, 0xFF88CCAA);

        // ── Progress bar ──
        let prog_y = status_y + 18;
        let prog_h = 4u32;
        framebuffer::fill_rect_alpha(inner_x, prog_y, inner_w, prog_h, 0x1A3322, 200);
        if state.total_ms > 0 {
            let fill_w = ((state.elapsed_ms as u64 * inner_w as u64) / state.total_ms.max(1) as u64) as u32;
            let fill_w = fill_w.min(inner_w);
            if fill_w > 0 {
                framebuffer::fill_rect(inner_x, prog_y, fill_w, prog_h, 0xFF00FF88);
                if fill_w > 2 {
                    framebuffer::fill_rect_alpha(inner_x + fill_w - 2, prog_y.saturating_sub(1), 4, prog_h + 2, 0x00FF88, 120);
                }
            }
        }

        // ── Waveform visualization (compact) ──
        let viz_y = prog_y + 12;
        let viz_h = 60u32;
        framebuffer::fill_rect_alpha(inner_x, viz_y, inner_w, viz_h, 0x030908, 160);
        framebuffer::fill_rect_alpha(inner_x, viz_y, inner_w, 1, 0x00FF66, 20);
        framebuffer::fill_rect_alpha(inner_x, viz_y + viz_h - 1, inner_w, 1, 0x00FF66, 12);

        let mid_y = viz_y + viz_h / 2;
        let half_h = (viz_h / 2 - 3) as f32;

        if state.state == PlaybackState::Playing || state.state == PlaybackState::Paused {
            let n_points = inner_w.min(128) as usize;
            let beat_glow = state.beat;
            for i in 0..n_points {
                let wave_i = (state.wave_idx + i) % 128;
                let sample = state.waveform[wave_i];
                let amp = sample * (1.0 + beat_glow * 0.5);
                let y_offset = (amp * half_h).max(-half_h).min(half_h) as i32;
                let px = inner_x + i as u32;
                let py = (mid_y as i32 + y_offset) as u32;
                let py = py.max(viz_y + 2).min(viz_y + viz_h - 3);

                let g_base = 0xCCu32;
                let b_shift = (beat_glow * 180.0) as u32;
                let r_shift = (state.energy * 60.0).min(60.0) as u32;
                let center = mid_y;
                if py < center {
                    for yy in py..center {
                        let fade = 1.0 - ((center - yy) as f32 / half_h).min(1.0) * 0.4;
                        let c = 0xFF000000 | (((r_shift as f32 * fade) as u32).min(0xFF) << 16)
                            | (((g_base as f32 * fade) as u32).min(0xFF) << 8)
                            | ((b_shift as f32 * fade) as u32).min(0xFF);
                        framebuffer::put_pixel_fast(px, yy, c);
                    }
                } else {
                    for yy in center..=py {
                        let fade = 1.0 - ((yy - center) as f32 / half_h).min(1.0) * 0.4;
                        let c = 0xFF000000 | (((r_shift as f32 * fade) as u32).min(0xFF) << 16)
                            | (((g_base as f32 * fade) as u32).min(0xFF) << 8)
                            | ((b_shift as f32 * fade) as u32).min(0xFF);
                        framebuffer::put_pixel_fast(px, yy, c);
                    }
                }
                framebuffer::put_pixel_fast(px, py, 0xFF00FFCC);
            }
            if beat_glow > 0.3 {
                let flash_alpha = ((beat_glow - 0.3) * 50.0) as u32;
                framebuffer::fill_rect_alpha(inner_x, viz_y, inner_w, viz_h, 0x00FF88, flash_alpha);
            }
        } else {
            framebuffer::fill_rect(inner_x + 4, mid_y, inner_w - 8, 1, 0xFF223322);
        }

        // ── Frequency bars (mini) ──
        let bars_y = viz_y + viz_h + 4;
        let bar_h = 14u32;
        let bar_w = inner_w / 4 - 3;
        let bands = [
            (state.sub_bass, 0xFF00FF44, "SB"),
            (state.bass, 0xFF00CC88, "BA"),
            (state.mid, 0xFF00AACC, "MD"),
            (state.treble, 0xFF8866FF, "TR"),
        ];
        for (bi, (level, color, label)) in bands.iter().enumerate() {
            let bx = inner_x + bi as u32 * (bar_w + 3);
            framebuffer::fill_rect_alpha(bx, bars_y, bar_w, bar_h, 0x0E1E14, 160);
            let fill = (level.min(1.0) * bar_w as f32) as u32;
            if fill > 0 {
                framebuffer::fill_rect(bx, bars_y, fill, bar_h, *color);
                framebuffer::fill_rect_alpha(bx, bars_y, fill, bar_h, 0xFFFFFF, 12);
            }
            self.draw_text(bx as i32 + 2, bars_y as i32 + 2, label, 0xFF99BB99);
        }

        // ═══════════════════════════════════════════════
        // TRANSPORT CONTROLS (fixed-size beautiful buttons)
        // ═══════════════════════════════════════════════
        let ctrl_y = bars_y + bar_h + 8;
        let btn_h = 28u32;

        // Fixed-size buttons: all same width except PLAY which is wider
        let small_btn_w = 36u32;
        let play_btn_w = 64u32;
        let gap = 4u32;
        let total_transport_w = small_btn_w * 3 + play_btn_w + gap * 3;
        let transport_x = inner_x + (inner_w.saturating_sub(total_transport_w)) / 2;

        // Helper: draw a styled button with glass effect
        fn draw_btn(this: &Desktop, bx: u32, by: u32, bw: u32, bh: u32, label: &str, bg: u32, border: u32, text_col: u32) {
            let cw = crate::graphics::scaling::char_width() as u32;
            // Body
            framebuffer::fill_rect_alpha(bx, by, bw, bh, bg, 210);
            // Top highlight
            framebuffer::fill_rect_alpha(bx + 1, by, bw - 2, 1, border, 80);
            // Bottom shadow
            framebuffer::fill_rect_alpha(bx + 1, by + bh - 1, bw - 2, 1, 0x000000, 60);
            // Left/right edges
            framebuffer::fill_rect_alpha(bx, by + 1, 1, bh - 2, border, 30);
            framebuffer::fill_rect_alpha(bx + bw - 1, by + 1, 1, bh - 2, border, 30);
            // Inner glow (subtle)
            framebuffer::fill_rect_alpha(bx + 1, by + 1, bw - 2, 2, 0xFFFFFF, 12);
            // Centered text
            let text_w = label.len() as u32 * cw;
            let tx = bx + (bw.saturating_sub(text_w)) / 2;
            let ty = by + (bh.saturating_sub(12)) / 2;
            this.draw_text(tx as i32, ty as i32, label, text_col);
        }

        // |< Previous
        let prev_x = transport_x;
        draw_btn(self, prev_x, ctrl_y, small_btn_w, btn_h, "|<", 0x142820, 0x00AA88, 0xFF88CCAA);

        // PLAY / PAUSE
        let play_x = prev_x + small_btn_w + gap;
        let play_label = match state.state {
            PlaybackState::Playing => "PAUSE",
            _ => "PLAY",
        };
        let play_bg = match state.state {
            PlaybackState::Playing => 0x0A5530,
            _ => 0x084428,
        };
        draw_btn(self, play_x, ctrl_y, play_btn_w, btn_h, play_label, play_bg, 0x00FF88, 0xFF00FFAA);

        // STOP
        let stop_x = play_x + play_btn_w + gap;
        draw_btn(self, stop_x, ctrl_y, small_btn_w, btn_h, "STOP", 0x2A1610, 0xCC6633, 0xFFFF8844);

        // >| Next
        let next_x = stop_x + small_btn_w + gap;
        draw_btn(self, next_x, ctrl_y, small_btn_w, btn_h, ">|", 0x142820, 0x00AA88, 0xFF88CCAA);

        // ── Volume control ──
        let vol_y = ctrl_y + btn_h + 8;
        let vol_h = 10u32;
        self.draw_text(inner_x as i32, vol_y as i32, "VOL", 0xFF44886A);

        let track_x = inner_x + 30;
        let track_w = inner_w.saturating_sub(72);
        framebuffer::fill_rect_alpha(track_x, vol_y + 3, track_w, 4, 0x1A3322, 200);
        let vol_fill = (state.volume as u32 * track_w) / 100;
        if vol_fill > 0 {
            framebuffer::fill_rect(track_x, vol_y + 3, vol_fill, 4, 0xFF00CC88);
        }
        // Knob
        let knob_x = track_x + vol_fill;
        if knob_x + 4 <= track_x + track_w + 4 {
            framebuffer::fill_rect(knob_x, vol_y, 4, vol_h, 0xFF00FFAA);
        }
        let vol_str = alloc::format!("{}%", state.volume);
        let vol_txt_x = track_x + track_w + 6;
        self.draw_text(vol_txt_x as i32, vol_y as i32, &vol_str, 0xFF88CCAA);

        // ═══════════════════════════════════════════════
        // EFFECTS CONTROLS (SYNC / VIZ / PAL / RAIN)
        // ═══════════════════════════════════════════════
        let fx_y = vol_y + vol_h + 10;
        // Separator line
        framebuffer::fill_rect_alpha(inner_x, fx_y, inner_w, 1, 0x00FF66, 20);
        let fx_start_y = fx_y + 4;
        self.draw_text(inner_x as i32, fx_start_y as i32, "EFFECTS", 0xFF336655);

        let fx_row_h = 24u32;
        let arrow_w = 24u32;
        let label_w = 36u32;

        // ── SYNC row ──
        let sync_y = fx_start_y + 16;
        self.draw_text(inner_x as i32, sync_y as i32 + 4, "SYNC", 0xFF44886A);
        let sync_ctrl_x = inner_x + label_w + 4;
        // [-] button
        draw_btn(self, sync_ctrl_x, sync_y, arrow_w, fx_row_h, "-", 0x142820, 0x00AA88, 0xFF88CCAA);
        // value display
        let sync_val_str = alloc::format!("{}ms", state.av_offset_ms);
        let sync_val_x = sync_ctrl_x + arrow_w + 4;
        let sync_val_w = 52u32;
        framebuffer::fill_rect_alpha(sync_val_x, sync_y, sync_val_w, fx_row_h, 0x0A1A12, 180);
        let svtx = sync_val_x + (sync_val_w.saturating_sub(sync_val_str.len() as u32 * cw)) / 2;
        self.draw_text(svtx as i32, sync_y as i32 + 5, &sync_val_str, 0xFF88CCAA);
        // [+] button
        let sync_plus_x = sync_val_x + sync_val_w + 4;
        draw_btn(self, sync_plus_x, sync_y, arrow_w, fx_row_h, "+", 0x142820, 0x00AA88, 0xFF88CCAA);
        // [0] reset
        let sync_reset_x = sync_plus_x + arrow_w + 4;
        draw_btn(self, sync_reset_x, sync_y, arrow_w, fx_row_h, "0", 0x1A1A14, 0x888855, 0xFFCCAA66);

        // ── VIZ row ──
        let viz_y2 = sync_y + fx_row_h + 4;
        self.draw_text(inner_x as i32, viz_y2 as i32 + 4, "VIZ", 0xFF44886A);
        let viz_ctrl_x = inner_x + label_w + 4;
        // [<] button
        draw_btn(self, viz_ctrl_x, viz_y2, arrow_w, fx_row_h, "<", 0x142820, 0x00AA88, 0xFF88CCAA);
        // mode name display
        let viz_mode = self.visualizer.mode as usize % crate::visualizer::NUM_MODES as usize;
        let viz_name = crate::visualizer::MODE_NAMES[viz_mode];
        let viz_name_x = viz_ctrl_x + arrow_w + 4;
        let viz_name_w = inner_w.saturating_sub(label_w + 4 + arrow_w * 2 + 12);
        framebuffer::fill_rect_alpha(viz_name_x, viz_y2, viz_name_w, fx_row_h, 0x0A1A12, 180);
        let max_name_chars = (viz_name_w / cw) as usize;
        let viz_display = if viz_name.len() > max_name_chars { &viz_name[..max_name_chars] } else { viz_name };
        let vntx = viz_name_x + (viz_name_w.saturating_sub(viz_display.len() as u32 * cw)) / 2;
        self.draw_text(vntx as i32, viz_y2 as i32 + 5, viz_display, 0xFF00DDAA);
        // [>] button
        let viz_next_x = viz_name_x + viz_name_w + 4;
        draw_btn(self, viz_next_x, viz_y2, arrow_w, fx_row_h, ">", 0x142820, 0x00AA88, 0xFF88CCAA);

        // ── PAL row ──
        let pal_y = viz_y2 + fx_row_h + 4;
        self.draw_text(inner_x as i32, pal_y as i32 + 4, "PAL", 0xFF44886A);
        let pal_ctrl_x = inner_x + label_w + 4;
        // [<] button
        draw_btn(self, pal_ctrl_x, pal_y, arrow_w, fx_row_h, "<", 0x142820, 0x8866CC, 0xFFAA88EE);
        // palette name display
        let pal_idx = self.visualizer.palette as usize % crate::visualizer::NUM_PALETTES as usize;
        let pal_name = crate::visualizer::PALETTE_NAMES[pal_idx];
        let pal_name_x = pal_ctrl_x + arrow_w + 4;
        let pal_name_w = inner_w.saturating_sub(label_w + 4 + arrow_w * 2 + 12);
        framebuffer::fill_rect_alpha(pal_name_x, pal_y, pal_name_w, fx_row_h, 0x0A1A12, 180);
        let max_pal_chars = (pal_name_w / cw) as usize;
        let pal_display = if pal_name.len() > max_pal_chars { &pal_name[..max_pal_chars] } else { pal_name };
        let pntx = pal_name_x + (pal_name_w.saturating_sub(pal_display.len() as u32 * cw)) / 2;
        self.draw_text(pntx as i32, pal_y as i32 + 5, pal_display, 0xFFCC88FF);
        // [>] button
        let pal_next_x = pal_name_x + pal_name_w + 4;
        draw_btn(self, pal_next_x, pal_y, arrow_w, fx_row_h, ">", 0x142820, 0x8866CC, 0xFFAA88EE);

        // ── RAIN row ──
        let rain_y = pal_y + fx_row_h + 4;
        self.draw_text(inner_x as i32, rain_y as i32 + 4, "RAIN", 0xFF44886A);
        let rain_ctrl_x = inner_x + label_w + 4;
        // [<] button
        draw_btn(self, rain_ctrl_x, rain_y, arrow_w, fx_row_h, "<", 0x142820, 0x00AA88, 0xFF88CCAA);
        // speed name display
        let rain_preset = (self.matrix_rain_preset as usize).min(2);
        let rain_names = ["Slow", "Mid", "Fast"];
        let rain_name = rain_names[rain_preset];
        let rain_name_x = rain_ctrl_x + arrow_w + 4;
        let rain_name_w = inner_w.saturating_sub(label_w + 4 + arrow_w * 2 + 12);
        framebuffer::fill_rect_alpha(rain_name_x, rain_y, rain_name_w, fx_row_h, 0x0A1A12, 180);
        let rntx = rain_name_x + (rain_name_w.saturating_sub(rain_name.len() as u32 * cw)) / 2;
        self.draw_text(rntx as i32, rain_y as i32 + 5, rain_name, 0xFF88DDAA);
        // [>] button
        let rain_next_x = rain_name_x + rain_name_w + 4;
        draw_btn(self, rain_next_x, rain_y, arrow_w, fx_row_h, ">", 0x142820, 0x00AA88, 0xFF88CCAA);
    }

    /// Draw interactive Calculator
    fn draw_calculator(&self, window: &Window) {
        let cx = window.x as u32 + 4;
        let cy = window.y as u32 + TITLE_BAR_HEIGHT + 4;
        let cw = window.width.saturating_sub(8);
        let ch = window.height.saturating_sub(TITLE_BAR_HEIGHT + 8);
        
        if cw < 100 || ch < 120 {
            return;
        }
        
        // ═══════════════════════════════════════════════════════════════
        // MODERN CALCULATOR — v2 visual overhaul
        // Larger display, bigger rounded buttons, glass look
        // ═══════════════════════════════════════════════════════════════
        
        // Display area (taller, with gradient)
        let display_h = 72u32;
        // Glass display background
        framebuffer::fill_rect(cx + 6, cy + 6, cw - 12, display_h, 0xFF0D0D1A);
        framebuffer::fill_rect_alpha(cx + 6, cy + 6, cw - 12, display_h / 2, 0x1A1A3E, 60);
        // Display border
        draw_rounded_rect_border((cx + 6) as i32, (cy + 6) as i32, cw - 12, display_h, 6, GREEN_GHOST);
        // Top inner highlight
        framebuffer::fill_rect_alpha(cx + 7, cy + 7, cw - 14, 1, 0x4444AA, 40);
        
        // Get calculator state
        let display_text = if let Some(calc) = self.calculator_states.get(&window.id) {
            &calc.display
        } else {
            "0"
        };
        
        // Draw display text (right-aligned, large — draw each char with bold effect)
        let text_len = display_text.len() as i32;
        let char_w = 12; // wider spacing for larger look
        let text_x = cx as i32 + cw as i32 - 18 - text_len * char_w;
        for (i, ch) in display_text.chars().enumerate() {
            let px = text_x + i as i32 * char_w;
            let py = cy as i32 + 28;
            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            // Triple-draw for bolder look
            self.draw_text(px, py, s, 0xFFFFFFFF);
            self.draw_text(px + 1, py, s, 0xFFFFFFFF);
            self.draw_text(px, py + 1, s, 0xFFEEEEEE);
        }
        
        // Expression indicator
        if let Some(calc) = self.calculator_states.get(&window.id) {
            if calc.just_evaluated && !calc.expression.is_empty() {
                self.draw_text(cx as i32 + 14, cy as i32 + 14, "=", GREEN_PRIMARY);
            }
        }
        
        // ── Button grid (bigger, rounded, with shadows) ──
        let btn_area_y = cy + display_h + 16;
        let btn_rows = 5u32;
        let btn_cols = 4u32;
        let btn_gap = 6u32;
        let available_w = cw.saturating_sub(16);
        let available_h = ch.saturating_sub(display_h + 28);
        let btn_w = (available_w - btn_gap * (btn_cols - 1)) / btn_cols;
        let btn_h = ((available_h - btn_gap * (btn_rows - 1)) / btn_rows).min(52);
        
        let buttons = [
            ["C", "(", ")", "%"],
            ["7", "8", "9", "/"],
            ["4", "5", "6", "*"],
            ["1", "2", "3", "-"],
            ["0", ".", "=", "+"],
        ];
        
        for (row, btn_row) in buttons.iter().enumerate() {
            for (col, label) in btn_row.iter().enumerate() {
                let bx = cx + 6 + col as u32 * (btn_w + btn_gap);
                let by = btn_area_y + row as u32 * (btn_h + btn_gap);
                
                let is_operator = matches!(*label, "+" | "-" | "*" | "/" | "%" | "=");
                let is_clear = *label == "C" || *label == "(" || *label == ")";
                
                // Button colors (modern, with depth)
                let (btn_bg, btn_border) = if is_operator {
                    if *label == "=" {
                        (GREEN_MUTED, GREEN_PRIMARY)
                    } else {
                        (0xFF1A2A22, GREEN_GHOST)
                    }
                } else if is_clear {
                    (0xFF2A1A28, 0xFF442244)
                } else {
                    (0xFF181C20, 0xFF2A2E34)
                };
                
                // Hover detection
                let hover = self.cursor_x >= bx as i32 && self.cursor_x < (bx + btn_w) as i32
                    && self.cursor_y >= by as i32 && self.cursor_y < (by + btn_h) as i32;
                
                let bg = if hover {
                    // Brighten on hover
                    let r = ((btn_bg >> 16) & 0xFF).min(220) + 30;
                    let g = ((btn_bg >> 8) & 0xFF).min(220) + 30;
                    let b = (btn_bg & 0xFF).min(220) + 30;
                    0xFF000000 | (r << 16) | (g << 8) | b
                } else {
                    btn_bg
                };
                
                // Button shadow
                if btn_h > 8 {
                    framebuffer::fill_rect_alpha(bx + 2, by + 2, btn_w, btn_h, 0x000000, 30);
                }
                
                // Draw rounded button
                let btn_radius = 8u32.min(btn_h / 3);
                draw_rounded_rect(bx as i32, by as i32, btn_w, btn_h, btn_radius, bg);
                draw_rounded_rect_border(bx as i32, by as i32, btn_w, btn_h, btn_radius, 
                    if hover { GREEN_PRIMARY } else { btn_border });
                
                // Top highlight (glass effect)
                if btn_h > 12 {
                    framebuffer::fill_rect_alpha(bx + 3, by + 1, btn_w.saturating_sub(6), 1, 0xFFFFFF, 15);
                }
                
                // Label centered (larger spacing)
                let lw = label.len() as u32 * 8;
                let lx = bx + (btn_w.saturating_sub(lw)) / 2;
                let ly = by + (btn_h / 2).saturating_sub(5);
                let text_color = if *label == "=" { 
                    0xFF000000 
                } else if is_operator { 
                    GREEN_PRIMARY 
                } else if is_clear {
                    ACCENT_AMBER
                } else { 
                    TEXT_PRIMARY 
                };
                self.draw_text(lx as i32, ly as i32, label, text_color);
                // Bold effect for operators
                if is_operator || is_clear {
                    self.draw_text(lx as i32 + 1, ly as i32, label, text_color);
                }
            }
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // Browser layout constants (shared between draw & click)
    // ═══════════════════════════════════════════════════════════════════
    const BROWSER_TAB_BAR_H: u32 = 28;
    const BROWSER_NAV_BAR_H: u32 = 38;
    const BROWSER_STATUS_H: u32 = 20;
    const BROWSER_PAD: u32 = 2; // inner window padding

    /// Compute common browser layout geometry from a window.
    /// Returns (bx, by, bw, bh, tab_y, nav_y, url_bar_x, url_bar_y, url_bar_w, url_bar_h, content_y, content_h, status_y, nav_btn_size)
    fn browser_layout(&self, window: &Window)
        -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32)
    {
        let bx = window.x as u32 + Self::BROWSER_PAD;
        let by = window.y as u32 + TITLE_BAR_HEIGHT;
        let bw = window.width.saturating_sub(Self::BROWSER_PAD * 2);
        let bh = window.height.saturating_sub(TITLE_BAR_HEIGHT + Self::BROWSER_PAD);

        let tab_y = by;                                 // tab bar top
        let nav_y = tab_y + Self::BROWSER_TAB_BAR_H;    // navigation bar top
        let nav_btn_size: u32 = 28;                      // circular nav buttons
        // 3 nav buttons + gaps = 3*28 + 3*6 = 102
        let nav_btns_w = nav_btn_size * 3 + 6 * 3;
        let url_bar_x = bx + 8 + nav_btns_w + 4;
        let url_bar_y = nav_y + 4;
        let url_bar_h = Self::BROWSER_NAV_BAR_H - 8;
        let url_bar_w = bw.saturating_sub(nav_btns_w + 20 + 40); // 40 for menu btn on right

        let content_y = nav_y + Self::BROWSER_NAV_BAR_H;
        let content_h = bh.saturating_sub(Self::BROWSER_TAB_BAR_H + Self::BROWSER_NAV_BAR_H + Self::BROWSER_STATUS_H);
        let status_y = content_y + content_h;

        (bx, by, bw, bh, tab_y, nav_y, url_bar_x, url_bar_y, url_bar_w, url_bar_h, content_y, content_h, status_y, nav_btn_size)
    }

    /// Draw Browser window content — Chrome / Edge style
    fn draw_browser(&self, window: &Window) {
        let (bx, _by, bw, bh, tab_y, nav_y,
             url_bar_x, url_bar_y, url_bar_w, url_bar_h,
             content_y, content_h, status_y, nav_btn_size)
            = self.browser_layout(window);

        if bw < 120 || bh < 100 { return; }

        let cw = crate::graphics::scaling::char_width() as i32;
        let ch = crate::graphics::scaling::char_height();

        // ── Tab bar (dark, like Chrome's #202124) ──────────────────────
        framebuffer::fill_rect(bx, tab_y, bw, Self::BROWSER_TAB_BAR_H, 0xFF202124);
        // Active tab pill
        let tab_x = bx + 8;
        let tab_w: u32 = 200.min(bw.saturating_sub(60));
        let tab_h = Self::BROWSER_TAB_BAR_H - 4;
        // Rounded-ish tab: draw body + 1px lighter top corners
        framebuffer::fill_rect(tab_x + 2, tab_y + 4, tab_w - 4, tab_h, 0xFF35363A);
        framebuffer::fill_rect(tab_x, tab_y + 6, 2, tab_h - 2, 0xFF35363A);
        framebuffer::fill_rect(tab_x + tab_w - 2, tab_y + 6, 2, tab_h - 2, 0xFF35363A);
        // Tab title
        let tab_title = if let Some(ref browser) = self.browser {
            if let Some(ref doc) = browser.document {
                if doc.title.is_empty() { alloc::string::String::from("New Tab") } else { doc.title.clone() }
            } else { alloc::string::String::from("New Tab") }
        } else { alloc::string::String::from("New Tab") };
        let display_title: alloc::string::String = if tab_title.len() > 22 {
            let s: alloc::string::String = tab_title.chars().take(20).collect();
            alloc::format!("{}...", s)
        } else { tab_title };
        self.draw_text(tab_x as i32 + 10, (tab_y + 8) as i32, &display_title, 0xFFE8EAED);
        // Close button on tab (tiny x)
        self.draw_text((tab_x + tab_w - 18) as i32, (tab_y + 8) as i32, "x", 0xFF999999);
        // "+" New tab button
        let plus_x = tab_x + tab_w + 6;
        framebuffer::fill_rect(plus_x, tab_y + 6, 24, tab_h, 0xFF2A2A2E);
        self.draw_text(plus_x as i32 + 8, (tab_y + 8) as i32, "+", 0xFF999999);

        // ── Navigation bar (slightly lighter #35363A) ─────────────────
        framebuffer::fill_rect(bx, nav_y, bw, Self::BROWSER_NAV_BAR_H, 0xFF35363A);
        // 1px separator between tab bar and nav bar
        framebuffer::fill_rect(bx, nav_y, bw, 1, 0xFF4A4A4E);

        // Circular nav buttons (back / forward / refresh)
        let btn_cy = nav_y + Self::BROWSER_NAV_BAR_H / 2; // vertical center
        let btn_r = nav_btn_size / 2;
        let mut bx_cursor = bx + 12u32;
        // Helper: draw a filled circle-ish button (octagon approx)
        let draw_nav_btn = |cx: u32, cy: u32, r: u32, hover_col: u32| {
            // Approximate circle: center rect + 4 corner rects
            let inset = r / 3;
            framebuffer::fill_rect(cx - r + inset, cy - r, (r - inset) * 2, r * 2, hover_col);
            framebuffer::fill_rect(cx - r, cy - r + inset, r * 2, (r - inset) * 2, hover_col);
        };
        // Back ◀
        let back_cx = bx_cursor + btn_r;
        draw_nav_btn(back_cx, btn_cy, btn_r, 0xFF4A4A4E);
        self.draw_text((back_cx - 4) as i32, (btn_cy - 6) as i32, "<", 0xFFE8EAED);
        bx_cursor += nav_btn_size + 6;
        // Forward ▶
        let fwd_cx = bx_cursor + btn_r;
        draw_nav_btn(fwd_cx, btn_cy, btn_r, 0xFF4A4A4E);
        self.draw_text((fwd_cx - 4) as i32, (btn_cy - 6) as i32, ">", 0xFFE8EAED);
        bx_cursor += nav_btn_size + 6;
        // Refresh ⟳  (or stop X while loading)
        let ref_cx = bx_cursor + btn_r;
        draw_nav_btn(ref_cx, btn_cy, btn_r, 0xFF4A4A4E);
        if self.browser_loading {
            self.draw_text((ref_cx - 4) as i32, (btn_cy - 6) as i32, "X", 0xFFE8EAED);
        } else {
            self.draw_text((ref_cx - 4) as i32, (btn_cy - 6) as i32, "R", 0xFFE8EAED);
        }

        // ── URL / Omnibox  (rounded, dark input field like Chrome) ────
        // Rounded rect: body + left/right caps
        let ur = url_bar_h / 2; // corner radius
        framebuffer::fill_rect(url_bar_x + ur, url_bar_y, url_bar_w.saturating_sub(ur * 2), url_bar_h, 0xFF202124);
        // Left cap
        framebuffer::fill_rect(url_bar_x, url_bar_y + ur / 2, ur, url_bar_h - ur, 0xFF202124);
        framebuffer::fill_rect(url_bar_x + 1, url_bar_y + ur / 4, ur - 1, ur / 2, 0xFF202124);
        // Right cap
        framebuffer::fill_rect(url_bar_x + url_bar_w - ur, url_bar_y + ur / 2, ur, url_bar_h - ur, 0xFF202124);
        framebuffer::fill_rect(url_bar_x + url_bar_w - ur, url_bar_y + ur / 4, ur - 1, ur / 2, 0xFF202124);
        // Focused: highlight border
        if window.focused {
            framebuffer::fill_rect(url_bar_x + ur, url_bar_y, url_bar_w.saturating_sub(ur * 2), 1, 0xFF8AB4F8);
            framebuffer::fill_rect(url_bar_x + ur, url_bar_y + url_bar_h - 1, url_bar_w.saturating_sub(ur * 2), 1, 0xFF8AB4F8);
        }

        // Lock / info icon placeholder
        let icon_x = url_bar_x as i32 + 8;
        let text_y = url_bar_y as i32 + (url_bar_h as i32 - ch as i32) / 2;
        let has_https = self.browser_url_input.starts_with("https://");
        if has_https {
            self.draw_text(icon_x, text_y, "S", 0xFF81C995);
        } else {
            // Draw a small circle with "i" for info icon, centered
            self.draw_text(icon_x + 1, text_y, "i", 0xFF999999);
        }
        // Separator line between icon and URL
        framebuffer::fill_rect((url_bar_x + 22) as u32, url_bar_y + 5, 1, url_bar_h - 10, 0xFF3C3C3C);

        // URL text
        let url_text_x = url_bar_x as i32 + 28;
        let url_text = if self.browser_url_input.is_empty() {
            "Search or enter URL"
        } else {
            &self.browser_url_input
        };
        let text_color = if self.browser_url_input.is_empty() { 0xFF9AA0A6 } else { 0xFFE8EAED };

        // Visible portion with scroll
        let text_area_px = (url_bar_w as i32).saturating_sub(42);
        let max_visible = if cw > 0 { (text_area_px / cw).max(1) as usize } else { 40 };
        let url_len = url_text.len();
        let scroll_off = if self.browser_url_cursor > max_visible {
            self.browser_url_cursor - max_visible + 1
        } else { 0 };
        let vis_end = (scroll_off + max_visible).min(url_len);
        let visible_text = if scroll_off < url_len { &url_text[scroll_off..vis_end] } else { "" };

        if self.browser_loading {
            self.draw_text(url_text_x, text_y, "Loading...", 0xFF8AB4F8);
        } else {
            self.draw_text(url_text_x, text_y, visible_text, text_color);
        }

        // Selection highlight (select-all draws blue background behind text)
        if !self.browser_loading && self.browser_url_select_all && !self.browser_url_input.is_empty() {
            let sel_w = (visible_text.len() as u32) * cw as u32;
            if sel_w > 0 {
                framebuffer::fill_rect(url_text_x as u32, url_bar_y + 3, sel_w.min(url_bar_w - 34), url_bar_h - 6, 0xFF3574E0);
                // Redraw text on top of selection in white
                self.draw_text(url_text_x, text_y, visible_text, 0xFFFFFFFF);
            }
        }

        // Blinking cursor
        if !self.browser_loading && window.focused {
            if self.cursor_blink {
                let coff = self.browser_url_cursor.saturating_sub(scroll_off);
                let cx = url_text_x + (coff as i32) * cw;
                if cx >= url_text_x && cx < (url_bar_x + url_bar_w - 8) as i32 {
                    framebuffer::fill_rect(cx as u32, url_bar_y + 4, 2, url_bar_h - 8, 0xFF8AB4F8);
                }
            }
        }

        // ── Menu button (3 dots) on the right ─────────────────────────
        let menu_x = url_bar_x + url_bar_w + 6;
        let menu_y = nav_y + 8;
        let dot_size: u32 = 3;
        let is_raw = self.browser.as_ref().map(|b| b.show_raw_html).unwrap_or(false);
        let menu_col = if is_raw { 0xFF8AB4F8 } else { 0xFF999999 };
        framebuffer::fill_rect(menu_x + 4, menu_y + 2, dot_size, dot_size, menu_col);
        framebuffer::fill_rect(menu_x + 4, menu_y + 8, dot_size, dot_size, menu_col);
        framebuffer::fill_rect(menu_x + 4, menu_y + 14, dot_size, dot_size, menu_col);

        // ── Content area ──────────────────────────────────────────────
        if let Some(ref browser) = self.browser {
            if browser.show_raw_html && !browser.raw_html.is_empty() {
                framebuffer::fill_rect(bx, content_y, bw, content_h, 0xFF1E1E1E);
                self.draw_raw_html_view(bx as i32, content_y as i32, bw, content_h, &browser.raw_html, browser.scroll_y);
            } else if let Some(ref doc) = browser.document {
                crate::browser::render_html(doc, bx as i32, content_y as i32, bw, content_h, browser.scroll_y);
            } else {
                self.draw_browser_welcome(bx, content_y, bw, content_h);
            }
        } else {
            self.draw_browser_welcome(bx, content_y, bw, content_h);
        }

        // ── Status bar ────────────────────────────────────────────────
        framebuffer::fill_rect(bx, status_y, bw, Self::BROWSER_STATUS_H, 0xFF202124);
        let status_text = if let Some(ref browser) = self.browser {
            match &browser.status {
                crate::browser::BrowserStatus::Idle => alloc::string::String::from("Ready"),
                crate::browser::BrowserStatus::Loading => alloc::string::String::from("Loading..."),
                crate::browser::BrowserStatus::Ready => {
                    if !browser.resources.is_empty() {
                        alloc::format!("Done  ({} resources)", browser.resources.len())
                    } else {
                        alloc::string::String::from("Done")
                    }
                },
                crate::browser::BrowserStatus::Error(e) => e.clone(),
            }
        } else { alloc::string::String::from("Ready") };
        self.draw_text(bx as i32 + 8, status_y as i32 + 3, &status_text, 0xFF9AA0A6);
    }

    /// Chrome-style welcome / new-tab page
    fn draw_browser_welcome(&self, bx: u32, cy: u32, bw: u32, ch: u32) {
        // Soft off-white background
        framebuffer::fill_rect(bx, cy, bw, ch, 0xFFFFFFFF);

        let mid_x = bx as i32 + bw as i32 / 2;
        let mid_y = cy as i32 + ch as i32 / 2 - 50;

        // "TrustBrowser" title
        let title = "TrustBrowser";
        let tcw = crate::graphics::scaling::char_width() as i32;
        let tx = mid_x - (title.len() as i32 * tcw) / 2;
        self.draw_text(tx, mid_y, title, 0xFF202124);

        // Fake search box (centered rounded rect)
        let box_w: u32 = 360.min(bw.saturating_sub(40));
        let box_h: u32 = 34;
        let box_x = (mid_x - box_w as i32 / 2).max(bx as i32 + 4) as u32;
        let box_y = (mid_y + 30) as u32;
        framebuffer::fill_rect(box_x + 4, box_y, box_w - 8, box_h, 0xFFF1F3F4);
        framebuffer::fill_rect(box_x, box_y + 4, 4, box_h - 8, 0xFFF1F3F4);
        framebuffer::fill_rect(box_x + box_w - 4, box_y + 4, 4, box_h - 8, 0xFFF1F3F4);
        // Border
        framebuffer::fill_rect(box_x + 4, box_y, box_w - 8, 1, 0xFFDFE1E5);
        framebuffer::fill_rect(box_x + 4, box_y + box_h - 1, box_w - 8, 1, 0xFFDFE1E5);
        // Placeholder text
        self.draw_text(box_x as i32 + 14, box_y as i32 + 9, "Search or type a URL", 0xFF9AA0A6);

        // Quick links row
        let links_y = box_y as i32 + box_h as i32 + 24;
        let link_labels = ["example.com", "10.0.2.2", "google.com"];
        let link_w: i32 = 100;
        let total_w = link_labels.len() as i32 * link_w + (link_labels.len() as i32 - 1) * 12;
        let mut lx = mid_x - total_w / 2;
        for label in &link_labels {
            // Rounded chip
            framebuffer::fill_rect(lx as u32, links_y as u32, link_w as u32, 28, 0xFFF1F3F4);
            framebuffer::fill_rect(lx as u32, links_y as u32, link_w as u32, 1, 0xFFDFE1E5);
            framebuffer::fill_rect(lx as u32, (links_y + 27) as u32, link_w as u32, 1, 0xFFDFE1E5);
            let tw = label.len() as i32 * tcw;
            self.draw_text(lx + (link_w - tw) / 2, links_y + 7, label, 0xFF1A73E8);
            lx += link_w + 12;
        }
    }
    
    /// Draw raw HTML source code view
    fn draw_raw_html_view(&self, x: i32, y: i32, width: u32, height: u32, html: &str, scroll_y: i32) {
        let cw = crate::graphics::scaling::char_width() as i32;
        let line_height = crate::graphics::scaling::char_height() as i32 + 2;
        let max_chars = if cw > 0 { (width as usize).saturating_sub(56) / cw as usize } else { 60 };

        let mut draw_y = y + 8 - scroll_y;
        let max_y = y + height as i32 - 8;
        let mut line_num = 1;

        for line in html.lines() {
            if draw_y > max_y { break; }
            if draw_y >= y - line_height {
                let line_str = alloc::format!("{:4} ", line_num);
                self.draw_text(x + 4, draw_y, &line_str, 0xFF6E7681);
                let display_line: alloc::string::String = if line.len() > max_chars {
                    let t: alloc::string::String = line.chars().take(max_chars.saturating_sub(3)).collect();
                    alloc::format!("{}...", t)
                } else { alloc::string::String::from(line) };
                self.draw_syntax_highlighted(x + 5 * cw + 8, draw_y, &display_line);
            }
            draw_y += line_height;
            line_num += 1;
        }
    }

    /// Draw syntax-highlighted HTML
    fn draw_syntax_highlighted(&self, x: i32, y: i32, line: &str) {
        let cw = crate::graphics::scaling::char_width() as i32;
        let mut current_x = x;
        let mut in_tag = false;
        let mut in_string = false;
        let mut in_attr = false;
        let mut string_char = '"';

        let chars: alloc::vec::Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            let color = if in_string {
                0xFFCE9178
            } else if c == '<' || c == '>' || c == '/' {
                in_tag = c == '<';
                if c == '>' { in_attr = false; }
                0xFF569CD6
            } else if in_tag && c == '=' {
                in_attr = true;
                0xFF9CDCFE
            } else if in_tag && (c == '"' || c == '\'') {
                in_string = true;
                string_char = c;
                0xFFCE9178
            } else if in_attr && !c.is_whitespace() {
                0xFF4EC9B0
            } else if in_tag && !c.is_whitespace() && c != '=' {
                0xFF569CD6
            } else {
                0xFFD4D4D4
            };
            if in_string && i > 0 && c == string_char && chars[i-1] != '\\' {
                in_string = false;
            }
            let s = alloc::format!("{}", c);
            self.draw_text(current_x, y, &s, color);
            current_x += cw;
            i += 1;
        }
    }
    
    /// Handle mouse click inside browser window (Chrome-style layout)
    fn handle_browser_click(&mut self, x: i32, y: i32, win_x: i32, win_y: i32, win_w: u32) {
        crate::serial_println!("[BROWSER-DBG] handle_browser_click x={} y={} win_x={} win_y={} win_w={}",
            x, y, win_x, win_y, win_w);
        // Build a dummy Window to reuse browser_layout geometry
        // We only need x, y, width, height and focused fields
        let tmp_win = Window {
            id: 0, title: String::new(),
            x: win_x, y: win_y,
            width: win_w,
            height: self.windows.iter()
                .find(|w| w.x == win_x && w.y == win_y && w.width == win_w)
                .map(|w| w.height).unwrap_or(500),
            min_width: 0, min_height: 0,
            visible: true, focused: true, minimized: false, maximized: false,
            dragging: false, resizing: ResizeEdge::None,
            drag_offset_x: 0, drag_offset_y: 0,
            saved_x: 0, saved_y: 0, saved_width: 0, saved_height: 0,
            window_type: WindowType::Browser,
            content: Vec::new(), file_path: None,
            selected_index: 0, scroll_offset: 0,
            animation: WindowAnimation::new(), pending_close: false,
        };
        let (_bx, _by, bw, _bh, _tab_y, nav_y,
             url_bar_x, url_bar_y, url_bar_w, url_bar_h,
             _content_y, _content_h, _status_y, nav_btn_size)
            = self.browser_layout(&tmp_win);

        crate::serial_println!("[BROWSER-DBG] layout: url_bar=({},{} {}x{}) nav_y={} bw={} click=({},{})",
            url_bar_x, url_bar_y, url_bar_w, url_bar_h, nav_y, bw, x, y);

        if bw < 120 { return; }

        let cx = x as u32;
        let cy = y as u32;

        // ── Navigation buttons (in nav bar row) ──
        let btn_r = nav_btn_size / 2;
        let btn_cy = nav_y + Self::BROWSER_NAV_BAR_H / 2;
        let mut btn_x = _bx + 12 + btn_r;
        // Check circular hit (use square approximation)
        let hit_btn = |bx_c: u32| -> bool {
            let dx = (cx as i32 - bx_c as i32).unsigned_abs();
            let dy = (cy as i32 - btn_cy as i32).unsigned_abs();
            dx <= btn_r && dy <= btn_r
        };
        // Back
        if hit_btn(btn_x) {
            crate::serial_println!("[BROWSER] Back button clicked");
            if let Some(ref mut browser) = self.browser { let _ = browser.back(); }
            return;
        }
        btn_x += nav_btn_size + 6;
        // Forward
        if hit_btn(btn_x) {
            crate::serial_println!("[BROWSER] Forward button clicked");
            if let Some(ref mut browser) = self.browser { let _ = browser.forward(); }
            return;
        }
        btn_x += nav_btn_size + 6;
        // Refresh
        if hit_btn(btn_x) {
            crate::serial_println!("[BROWSER] Refresh button clicked");
            if let Some(ref mut browser) = self.browser { let _ = browser.refresh(); }
            return;
        }

        // ── URL bar click — full rectangle hit area ──
        if cx >= url_bar_x && cx < url_bar_x + url_bar_w
            && cy >= url_bar_y && cy < url_bar_y + url_bar_h
        {
            // Double-click → select all URL text
            if crate::mouse::is_double_click() {
                crate::mouse::reset_click_count();
                self.browser_url_select_all = true;
                self.browser_url_cursor = self.browser_url_input.len();
                crate::serial_println!("[BROWSER] URL bar double-clicked, select all");
                return;
            }
            // Single click → position cursor
            self.browser_url_select_all = false;
            let cw = crate::graphics::scaling::char_width();
            if cw > 0 {
                let text_start_x = url_bar_x + 26;
                let rel_x = cx.saturating_sub(text_start_x);
                let char_pos = (rel_x / cw) as usize;
                self.browser_url_cursor = char_pos.min(self.browser_url_input.len());
                crate::serial_println!("[BROWSER] URL bar clicked, cursor={}", self.browser_url_cursor);
            }
            return;
        }

        // ── Menu button (three dots) — toggle RAW/HTML view ──
        let menu_x = url_bar_x + url_bar_w + 6;
        let menu_y = nav_y + 8;
        if cx >= menu_x && cx < menu_x + 16 && cy >= menu_y && cy < menu_y + 22 {
            crate::serial_println!("[BROWSER] Menu (view toggle) clicked");
            if let Some(ref mut browser) = self.browser {
                browser.toggle_view_mode();
            }
            return;
        }
    }
    
    fn draw_cursor(&self) {
        // Determine cursor type based on context (resize edges, etc.)
        let mut cursor_mode = CursorMode::Arrow;
        
        // Check if hovering over a resize edge of any non-maximized window
        for w in self.windows.iter().rev() {
            if w.minimized || w.maximized { continue; }
            let edge = w.on_resize_edge(self.cursor_x, self.cursor_y);
            match edge {
                ResizeEdge::Left | ResizeEdge::Right => { cursor_mode = CursorMode::ResizeH; break; },
                ResizeEdge::Top | ResizeEdge::Bottom => { cursor_mode = CursorMode::ResizeV; break; },
                ResizeEdge::TopLeft | ResizeEdge::BottomRight => { cursor_mode = CursorMode::ResizeNWSE; break; },
                ResizeEdge::TopRight | ResizeEdge::BottomLeft => { cursor_mode = CursorMode::ResizeNESW; break; },
                _ => {},
            }
            // If point is inside window bounds, stop checking further
            if self.cursor_x >= w.x && self.cursor_x < w.x + w.width as i32
                && self.cursor_y >= w.y && self.cursor_y < w.y + w.height as i32 {
                break;
            }
        }
        
        // Also check if actively resizing
        for w in &self.windows {
            match w.resizing {
                ResizeEdge::Left | ResizeEdge::Right => { cursor_mode = CursorMode::ResizeH; break; },
                ResizeEdge::Top | ResizeEdge::Bottom => { cursor_mode = CursorMode::ResizeV; break; },
                ResizeEdge::TopLeft | ResizeEdge::BottomRight => { cursor_mode = CursorMode::ResizeNWSE; break; },
                ResizeEdge::TopRight | ResizeEdge::BottomLeft => { cursor_mode = CursorMode::ResizeNESW; break; },
                _ => {},
            }
        }
        
        match cursor_mode {
            CursorMode::Arrow | CursorMode::Grab => self.draw_arrow_cursor(),
            CursorMode::ResizeH => self.draw_resize_cursor_h(),
            CursorMode::ResizeV => self.draw_resize_cursor_v(),
            CursorMode::ResizeNWSE => self.draw_resize_cursor_nwse(),
            CursorMode::ResizeNESW => self.draw_resize_cursor_nesw(),
        }
    }
    
    /// Default arrow cursor
    fn draw_arrow_cursor(&self) {
        let cs = crate::accessibility::get_cursor_size().scale();
        let hc = crate::accessibility::is_high_contrast();
        
        // Simple drop shadow (scaled)
        let shadow_color = 0x40000000u32;
        for offset in 1..=(2 * cs as i32) {
            let sx = self.cursor_x + offset;
            let sy = self.cursor_y + offset;
            if sx >= 0 && sy >= 0 && sx < self.width as i32 && sy < self.height as i32 {
                for dy in 0..(12 * cs as i32) {
                    let py = (sy + dy) as u32;
                    let px = sx as u32;
                    if py < self.height && px < self.width {
                        framebuffer::put_pixel_fast(px, py, shadow_color);
                    }
                }
            }
        }
        
        // Cursor colors: high contrast uses white/black for maximum visibility
        let outline_color = if hc { 0xFF000000u32 } else { GREEN_MUTED };
        let fill_color = if hc { 0xFFFFFFFFu32 } else { GREEN_SECONDARY };
        
        // Modern arrow cursor with green accent (scaled by cursor size)
        let cursor: [[u8; 12]; 16] = [
            [1,0,0,0,0,0,0,0,0,0,0,0],
            [1,1,0,0,0,0,0,0,0,0,0,0],
            [1,2,1,0,0,0,0,0,0,0,0,0],
            [1,2,2,1,0,0,0,0,0,0,0,0],
            [1,2,2,2,1,0,0,0,0,0,0,0],
            [1,2,2,2,2,1,0,0,0,0,0,0],
            [1,2,2,2,2,2,1,0,0,0,0,0],
            [1,2,2,2,2,2,2,1,0,0,0,0],
            [1,2,2,2,2,2,2,2,1,0,0,0],
            [1,2,2,2,2,2,2,2,2,1,0,0],
            [1,2,2,2,2,2,1,1,1,1,1,0],
            [1,2,2,1,2,2,1,0,0,0,0,0],
            [1,2,1,0,1,2,2,1,0,0,0,0],
            [1,1,0,0,1,2,2,1,0,0,0,0],
            [1,0,0,0,0,1,2,2,1,0,0,0],
            [0,0,0,0,0,1,1,1,1,0,0,0],
        ];
        
        for (cy, row) in cursor.iter().enumerate() {
            for (cx, &pixel) in row.iter().enumerate() {
                if pixel == 0 { continue; }
                let color = match pixel {
                    1 => outline_color,
                    2 => fill_color,
                    _ => continue,
                };
                // Draw cs×cs block per cursor pixel for accessibility scaling
                for sy in 0..cs {
                    for sx in 0..cs {
                        let px = (self.cursor_x + cx as i32 * cs as i32 + sx as i32) as u32;
                        let py = (self.cursor_y + cy as i32 * cs as i32 + sy as i32) as u32;
                        if px < self.width && py < self.height {
                            framebuffer::put_pixel_fast(px, py, color);
                        }
                    }
                }
            }
        }
    }
    
    /// Horizontal resize cursor (←→)
    fn draw_resize_cursor_h(&self) {
        let mx = self.cursor_x;
        let my = self.cursor_y;
        // Horizontal double arrow: ←→ centered on cursor
        // Left arrow
        for i in 0..7i32 {
            let px = (mx - 7 + i) as u32;
            let py = my as u32;
            if px < self.width && py < self.height {
                framebuffer::put_pixel_fast(px, py, GREEN_PRIMARY);
                if py > 0 { framebuffer::put_pixel_fast(px, py - 1, GREEN_MUTED); }
                if py + 1 < self.height { framebuffer::put_pixel_fast(px, py + 1, GREEN_MUTED); }
            }
        }
        // Right arrow  
        for i in 0..7i32 {
            let px = (mx + 1 + i) as u32;
            let py = my as u32;
            if px < self.width && py < self.height {
                framebuffer::put_pixel_fast(px, py, GREEN_PRIMARY);
                if py > 0 { framebuffer::put_pixel_fast(px, py - 1, GREEN_MUTED); }
                if py + 1 < self.height { framebuffer::put_pixel_fast(px, py + 1, GREEN_MUTED); }
            }
        }
        // Left arrowhead
        for d in 1..=4i32 {
            let px = (mx - 7 + d) as u32;
            if px < self.width {
                if (my - d) >= 0 { framebuffer::put_pixel_fast(px, (my - d) as u32, GREEN_PRIMARY); }
                if (my + d) < self.height as i32 { framebuffer::put_pixel_fast(px, (my + d) as u32, GREEN_PRIMARY); }
            }
        }
        // Right arrowhead
        for d in 1..=4i32 {
            let px = (mx + 7 - d) as u32;
            if px < self.width {
                if (my - d) >= 0 { framebuffer::put_pixel_fast(px, (my - d) as u32, GREEN_PRIMARY); }
                if (my + d) < self.height as i32 { framebuffer::put_pixel_fast(px, (my + d) as u32, GREEN_PRIMARY); }
            }
        }
        // Center dot
        if mx >= 0 && my >= 0 && (mx as u32) < self.width && (my as u32) < self.height {
            framebuffer::put_pixel_fast(mx as u32, my as u32, 0xFFFFFFFF);
        }
    }
    
    /// Vertical resize cursor (↕)
    fn draw_resize_cursor_v(&self) {
        let mx = self.cursor_x;
        let my = self.cursor_y;
        // Vertical double arrow
        for i in 0..7i32 {
            let px = mx as u32;
            let py = (my - 7 + i) as u32;
            if px < self.width && py < self.height {
                framebuffer::put_pixel_fast(px, py, GREEN_PRIMARY);
                if px > 0 { framebuffer::put_pixel_fast(px - 1, py, GREEN_MUTED); }
                if px + 1 < self.width { framebuffer::put_pixel_fast(px + 1, py, GREEN_MUTED); }
            }
        }
        for i in 0..7i32 {
            let px = mx as u32;
            let py = (my + 1 + i) as u32;
            if px < self.width && py < self.height {
                framebuffer::put_pixel_fast(px, py, GREEN_PRIMARY);
                if px > 0 { framebuffer::put_pixel_fast(px - 1, py, GREEN_MUTED); }
                if px + 1 < self.width { framebuffer::put_pixel_fast(px + 1, py, GREEN_MUTED); }
            }
        }
        // Top arrowhead
        for d in 1..=4i32 {
            let py = (my - 7 + d) as u32;
            if py < self.height {
                if (mx - d) >= 0 { framebuffer::put_pixel_fast((mx - d) as u32, py, GREEN_PRIMARY); }
                if (mx + d) < self.width as i32 { framebuffer::put_pixel_fast((mx + d) as u32, py, GREEN_PRIMARY); }
            }
        }
        // Bottom arrowhead
        for d in 1..=4i32 {
            let py = (my + 7 - d) as u32;
            if py < self.height {
                if (mx - d) >= 0 { framebuffer::put_pixel_fast((mx - d) as u32, py, GREEN_PRIMARY); }
                if (mx + d) < self.width as i32 { framebuffer::put_pixel_fast((mx + d) as u32, py, GREEN_PRIMARY); }
            }
        }
        if mx >= 0 && my >= 0 && (mx as u32) < self.width && (my as u32) < self.height {
            framebuffer::put_pixel_fast(mx as u32, my as u32, 0xFFFFFFFF);
        }
    }
    
    /// NW-SE diagonal resize cursor (↘↖)
    fn draw_resize_cursor_nwse(&self) {
        let mx = self.cursor_x;
        let my = self.cursor_y;
        // Diagonal line NW→SE
        for i in -6..=6i32 {
            let px = (mx + i) as u32;
            let py = (my + i) as u32;
            if px < self.width && py < self.height {
                framebuffer::put_pixel_fast(px, py, GREEN_PRIMARY);
                if px + 1 < self.width { framebuffer::put_pixel_fast(px + 1, py, GREEN_MUTED); }
                if py + 1 < self.height { framebuffer::put_pixel_fast(px, py + 1, GREEN_MUTED); }
            }
        }
        // NW arrowhead
        for d in 1..=3i32 {
            let bx = mx - 6 + d;
            let by = my - 6;
            if bx >= 0 && (by as u32) < self.height { framebuffer::put_pixel_fast(bx as u32, by as u32, GREEN_PRIMARY); }
            let bx2 = mx - 6;
            let by2 = my - 6 + d;
            if bx2 >= 0 && by2 >= 0 { framebuffer::put_pixel_fast(bx2 as u32, by2 as u32, GREEN_PRIMARY); }
        }
        // SE arrowhead
        for d in 1..=3i32 {
            let bx = mx + 6 - d;
            let by = my + 6;
            if (bx as u32) < self.width && (by as u32) < self.height { framebuffer::put_pixel_fast(bx as u32, by as u32, GREEN_PRIMARY); }
            let bx2 = mx + 6;
            let by2 = my + 6 - d;
            if (bx2 as u32) < self.width && (by2 as u32) < self.height { framebuffer::put_pixel_fast(bx2 as u32, by2 as u32, GREEN_PRIMARY); }
        }
    }
    
    /// NE-SW diagonal resize cursor (↗↙)
    fn draw_resize_cursor_nesw(&self) {
        let mx = self.cursor_x;
        let my = self.cursor_y;
        // Diagonal line NE→SW
        for i in -6..=6i32 {
            let px = (mx + i) as u32;
            let py = (my - i) as u32;
            if px < self.width && py < self.height {
                framebuffer::put_pixel_fast(px, py, GREEN_PRIMARY);
                if px > 0 { framebuffer::put_pixel_fast(px - 1, py, GREEN_MUTED); }
                if py + 1 < self.height { framebuffer::put_pixel_fast(px, py + 1, GREEN_MUTED); }
            }
        }
        // NE arrowhead
        for d in 1..=3i32 {
            let bx = mx + 6 - d;
            let by = my - 6;
            if (bx as u32) < self.width && by >= 0 { framebuffer::put_pixel_fast(bx as u32, by as u32, GREEN_PRIMARY); }
            let bx2 = mx + 6;
            let by2 = my - 6 + d;
            if (bx2 as u32) < self.width && by2 >= 0 { framebuffer::put_pixel_fast(bx2 as u32, by2 as u32, GREEN_PRIMARY); }
        }
        // SW arrowhead
        for d in 1..=3i32 {
            let bx = mx - 6 + d;
            let by = my + 6;
            if bx >= 0 && (by as u32) < self.height { framebuffer::put_pixel_fast(bx as u32, by as u32, GREEN_PRIMARY); }
            let bx2 = mx - 6;
            let by2 = my + 6 - d;
            if bx2 >= 0 && (by2 as u32) < self.height { framebuffer::put_pixel_fast(bx2 as u32, by2 as u32, GREEN_PRIMARY); }
        }
    }
    
    fn draw_text(&self, x: i32, y: i32, text: &str, color: u32) {
        // Use scaled text rendering
        let old_fg = framebuffer::get_fg_color();
        framebuffer::set_fg_color(color);
        
        let cw = crate::graphics::scaling::char_width() as i32;
        for (i, c) in text.chars().enumerate() {
            let px = x + (i as i32 * cw);
            if px >= 0 && px < self.width as i32 && y >= 0 && y < self.height as i32 {
                crate::graphics::scaling::draw_scaled_char(px as u32, y as u32, c, color);
            }
        }
        
        framebuffer::set_fg_color(old_fg);
    }
    
    fn draw_char(&self, x: u32, y: u32, c: char, color: u32) {
        // Use scaled character rendering
        crate::graphics::scaling::draw_scaled_char(x, y, c, color);
    }
    
    /// Draw text with sub-pixel anti-aliasing — reads current pixels to blend edges
    fn draw_text_smooth(&self, x: i32, y: i32, text: &str, color: u32) {
        let cw = crate::graphics::scaling::char_width() as i32;
        let factor = crate::graphics::scaling::get_scale_factor();
        let _ch = 16u32 * factor;
        let _fw = 8u32 * factor;
        let fb_w = self.width;
        let fb_h = self.height;
        
        let fg_r = ((color >> 16) & 0xFF) as u32;
        let fg_g = ((color >> 8) & 0xFF) as u32;
        let fg_b = (color & 0xFF) as u32;
        
        for (i, c) in text.chars().enumerate() {
            let cx = x + (i as i32 * cw);
            if cx < 0 || cx >= fb_w as i32 || y < 0 || y >= fb_h as i32 { continue; }
            
            let glyph = framebuffer::font::get_glyph(c);
            
            for row in 0..16u32 {
                let bits = glyph[row as usize];
                let prev = if row > 0 { glyph[row as usize - 1] } else { 0u8 };
                let next = if row < 15 { glyph[row as usize + 1] } else { 0u8 };
                
                for col in 0..8u32 {
                    let mask = 0x80u8 >> col;
                    let is_set = bits & mask != 0;
                    
                    if is_set {
                        // Draw foreground block
                        for sy in 0..factor {
                            for sx in 0..factor {
                                let px = cx as u32 + col * factor + sx;
                                let py = y as u32 + row * factor + sy;
                                if px < fb_w && py < fb_h {
                                    framebuffer::put_pixel_fast(px, py, color);
                                }
                            }
                        }
                    } else {
                        // Check 8-connected neighbors for improved AA
                        let left  = col > 0 && (bits & (mask << 1)) != 0;
                        let right = col < 7 && (bits & (mask >> 1)) != 0;
                        let top   = prev & mask != 0;
                        let bot   = next & mask != 0;
                        // Diagonal neighbors (weighted at 0.7x)
                        let tl = col > 0 && (prev & (mask << 1)) != 0;
                        let tr = col < 7 && (prev & (mask >> 1)) != 0;
                        let bl = col > 0 && (next & (mask << 1)) != 0;
                        let br = col < 7 && (next & (mask >> 1)) != 0;
                        
                        // Cardinal adjacency counts as 1.0, diagonal as 0.5
                        let cardinal = (left as u32) + (right as u32) + (top as u32) + (bot as u32);
                        let diagonal = (tl as u32) + (tr as u32) + (bl as u32) + (br as u32);
                        let score = cardinal * 2 + diagonal; // max = 8+4 = 12
                        
                        if score > 0 {
                            // Smoother alpha curve based on neighbor density
                            let alpha = if score >= 6 { 140u32 }
                                else if score >= 4 { 100u32 }
                                else if score >= 2 { 60u32 }
                                else { 35u32 };
                            let inv = 255 - alpha;
                            for sy in 0..factor {
                                for sx in 0..factor {
                                    let px = cx as u32 + col * factor + sx;
                                    let py = y as u32 + row * factor + sy;
                                    if px < fb_w && py < fb_h {
                                        let bg = framebuffer::get_pixel_fast(px, py);
                                        let bg_r = (bg >> 16) & 0xFF;
                                        let bg_g = (bg >> 8) & 0xFF;
                                        let bg_b = bg & 0xFF;
                                        let r = (fg_r * alpha + bg_r * inv) / 255;
                                        let g = (fg_g * alpha + bg_g * inv) / 255;
                                        let b = (fg_b * alpha + bg_b * inv) / 255;
                                        framebuffer::put_pixel_fast(px, py, 0xFF000000 | (r << 16) | (g << 8) | b);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Get simple glyph for character (8x16 bitmap)
fn get_char_glyph(c: char) -> [u8; 16] {
    // Very simple glyphs for basic characters
    match c {
        'A' => [0x00,0x18,0x3C,0x66,0x66,0x7E,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00],
        'B' => [0x00,0x7C,0x66,0x66,0x7C,0x66,0x66,0x66,0x7C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'C' => [0x00,0x3C,0x66,0x60,0x60,0x60,0x60,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'D' => [0x00,0x78,0x6C,0x66,0x66,0x66,0x66,0x6C,0x78,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'E' => [0x00,0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'F' => [0x00,0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'G' => [0x00,0x3C,0x66,0x60,0x60,0x6E,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'H' => [0x00,0x66,0x66,0x66,0x7E,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'I' => [0x00,0x3C,0x18,0x18,0x18,0x18,0x18,0x18,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'J' => [0x00,0x1E,0x0C,0x0C,0x0C,0x0C,0x6C,0x6C,0x38,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'K' => [0x00,0x66,0x6C,0x78,0x70,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'L' => [0x00,0x60,0x60,0x60,0x60,0x60,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'M' => [0x00,0x63,0x77,0x7F,0x6B,0x63,0x63,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'N' => [0x00,0x66,0x76,0x7E,0x7E,0x6E,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'O' => [0x00,0x3C,0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'P' => [0x00,0x7C,0x66,0x66,0x7C,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Q' => [0x00,0x3C,0x66,0x66,0x66,0x66,0x6E,0x3C,0x0E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'R' => [0x00,0x7C,0x66,0x66,0x7C,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'S' => [0x00,0x3C,0x66,0x60,0x3C,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'T' => [0x00,0x7E,0x18,0x18,0x18,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'U' => [0x00,0x66,0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'V' => [0x00,0x66,0x66,0x66,0x66,0x66,0x3C,0x3C,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'W' => [0x00,0x63,0x63,0x63,0x6B,0x7F,0x77,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'X' => [0x00,0x66,0x66,0x3C,0x18,0x3C,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Y' => [0x00,0x66,0x66,0x66,0x3C,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Z' => [0x00,0x7E,0x06,0x0C,0x18,0x30,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'a' => [0x00,0x00,0x00,0x3C,0x06,0x3E,0x66,0x66,0x3E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'b' => [0x00,0x60,0x60,0x7C,0x66,0x66,0x66,0x66,0x7C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'c' => [0x00,0x00,0x00,0x3C,0x66,0x60,0x60,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'd' => [0x00,0x06,0x06,0x3E,0x66,0x66,0x66,0x66,0x3E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'e' => [0x00,0x00,0x00,0x3C,0x66,0x7E,0x60,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'f' => [0x00,0x1C,0x36,0x30,0x7C,0x30,0x30,0x30,0x30,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'g' => [0x00,0x00,0x00,0x3E,0x66,0x66,0x3E,0x06,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'h' => [0x00,0x60,0x60,0x7C,0x66,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'i' => [0x00,0x18,0x00,0x38,0x18,0x18,0x18,0x18,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'j' => [0x00,0x0C,0x00,0x1C,0x0C,0x0C,0x0C,0x6C,0x38,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'k' => [0x00,0x60,0x60,0x66,0x6C,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'l' => [0x00,0x38,0x18,0x18,0x18,0x18,0x18,0x18,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'm' => [0x00,0x00,0x00,0x76,0x7F,0x6B,0x6B,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'n' => [0x00,0x00,0x00,0x7C,0x66,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'o' => [0x00,0x00,0x00,0x3C,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'p' => [0x00,0x00,0x00,0x7C,0x66,0x66,0x7C,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'q' => [0x00,0x00,0x00,0x3E,0x66,0x66,0x3E,0x06,0x06,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'r' => [0x00,0x00,0x00,0x7C,0x66,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        's' => [0x00,0x00,0x00,0x3E,0x60,0x3C,0x06,0x06,0x7C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        't' => [0x00,0x30,0x30,0x7C,0x30,0x30,0x30,0x36,0x1C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'u' => [0x00,0x00,0x00,0x66,0x66,0x66,0x66,0x66,0x3E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'v' => [0x00,0x00,0x00,0x66,0x66,0x66,0x3C,0x3C,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'w' => [0x00,0x00,0x00,0x63,0x63,0x6B,0x7F,0x77,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'x' => [0x00,0x00,0x00,0x66,0x3C,0x18,0x3C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'y' => [0x00,0x00,0x00,0x66,0x66,0x3E,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'z' => [0x00,0x00,0x00,0x7E,0x0C,0x18,0x30,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '0' => [0x00,0x3C,0x66,0x6E,0x76,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '1' => [0x00,0x18,0x38,0x18,0x18,0x18,0x18,0x18,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '2' => [0x00,0x3C,0x66,0x06,0x0C,0x18,0x30,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '3' => [0x00,0x3C,0x66,0x06,0x1C,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '4' => [0x00,0x0C,0x1C,0x3C,0x6C,0x7E,0x0C,0x0C,0x0C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '5' => [0x00,0x7E,0x60,0x7C,0x06,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '6' => [0x00,0x3C,0x66,0x60,0x7C,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '7' => [0x00,0x7E,0x06,0x0C,0x18,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '8' => [0x00,0x3C,0x66,0x66,0x3C,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '9' => [0x00,0x3C,0x66,0x66,0x3E,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '-' => [0x00,0x00,0x00,0x00,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        ' ' => [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        _ => [0x00,0x3C,0x42,0x42,0x42,0x42,0x42,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00], // Box for unknown
    }
}

/// Global desktop
pub static DESKTOP: Mutex<Desktop> = Mutex::new(Desktop::new());

/// Initialize GUI with double buffering
pub fn init(width: u32, height: u32) {
    DESKTOP.lock().init(width, height);
    crate::serial_println!("[GUI] Desktop initialized: {}x{} (double-buffered)", width, height);
}

/// Create a window (legacy, empty type)
pub fn create_window(title: &str, x: i32, y: i32, width: u32, height: u32) -> u32 {
    DESKTOP.lock().create_window(title, x, y, width, height, WindowType::Empty)
}

/// Create a terminal window
pub fn create_terminal(x: i32, y: i32) -> u32 {
    DESKTOP.lock().create_window("Terminal", x, y, 640, 440, WindowType::Terminal)
}

/// Create a system info window
pub fn create_sysinfo(x: i32, y: i32) -> u32 {
    DESKTOP.lock().create_window("System Info", x, y, 300, 220, WindowType::SystemInfo)
}

/// Close a window
pub fn close_window(id: u32) {
    DESKTOP.lock().close_window(id);
}

/// Update cursor position
pub fn update_cursor(x: i32, y: i32) {
    DESKTOP.lock().handle_move(x, y);
}

/// Handle click
pub fn handle_click(x: i32, y: i32, pressed: bool) {
    DESKTOP.lock().handle_click(x, y, pressed);
}

/// Draw everything
pub fn draw() {
    DESKTOP.lock().draw();
}

/// Handle keyboard input for focused window
pub fn handle_keyboard(key: u8) {
    DESKTOP.lock().handle_keyboard_input(key);
}

/// Handle right-click
pub fn handle_right_click(x: i32, y: i32, pressed: bool) {
    DESKTOP.lock().handle_right_click(x, y, pressed);
}

/// Handle scroll wheel
pub fn handle_scroll(delta: i8) {
    DESKTOP.lock().handle_scroll(delta);
}

/// Run GUI loop with double-buffered rendering and vsync-like timing
pub fn run() {
    use crate::gui::engine::{self, HotkeyAction};
    EXIT_DESKTOP_FLAG.store(false, Ordering::SeqCst);
    
    // Initialize GUI timing + vsync
    engine::init_timing();
    crate::gui::vsync::init();
    
    // Reset mouse button edge-detection state so stale state
    // from a previous session doesn't trigger false clicks
    unsafe {
        // These statics are in the loop body; we mirror-reset here
        static mut RESET_MOUSE_INIT: bool = true;
        RESET_MOUSE_INIT = true;
    }
    // Also ensure context menu is hidden on fresh entry
    {
        let mut d = DESKTOP.lock();
        d.context_menu.visible = false;
    }
    
    crate::serial_println!("[GUI] Starting desktop environment...");
    crate::serial_println!("[GUI] Hotkeys: Alt+Tab, Win+Arrows, Alt+F4, Win=Start");
    crate::serial_println!("[GUI] Target: ~60 FPS (16.6ms) with spin-loop frame limiting");
    
    // Frame counter for safe startup (skip heavy work on first few frames)
    let mut loop_frame: u32 = 0;

    loop {
        // Check exit flag
        if EXIT_DESKTOP_FLAG.load(Ordering::SeqCst) {
            crate::serial_println!("[GUI] Desktop exit requested, returning to shell");
            break;
        }
        let frame_start = engine::now_us();
        
        // ═══════════════════════════════════════════════════════════════
        // Input Processing
        // ═══════════════════════════════════════════════════════════════
        
        // Process mouse
        let mouse = crate::mouse::get_state();
        update_cursor(mouse.x, mouse.y);
        
        // ── Scancode-level Alt+Tab detection (poll raw key state) ──
        // This catches Alt+Tab even if VirtualBox or host OS eats the ASCII
        {
            let alt_held = crate::keyboard::is_key_pressed(0x38);
            let win_held = crate::keyboard::is_key_pressed(0x5B);
            let ctrl_held = crate::keyboard::is_key_pressed(0x1D);
            let tab_held = crate::keyboard::is_key_pressed(0x0F);
            static mut PREV_TAB_RAW: bool = false;
            unsafe {
                if (alt_held || win_held || ctrl_held) && tab_held && !PREV_TAB_RAW {
                    if !engine::is_alt_tab_active() {
                        engine::start_alt_tab();
                    } else {
                        engine::alt_tab_next();
                    }
                }
                PREV_TAB_RAW = tab_held;
            }
        }
        
        // Handle keyboard input
        // NOTE: read_char() returns ASCII values, NOT scancodes.
        // The interrupt handler converts scancodes→ASCII and strips releases.
        // Use keyboard::is_key_pressed(scancode) to check modifier/key state.
        // Cap at 32 keys per frame to prevent infinite loop from noisy UART/keyboard
        let mut keys_this_frame = 0u32;
        while let Some(key) = crate::keyboard::read_char() {
            keys_this_frame += 1;
            if keys_this_frame > 32 { break; }
            crate::serial_println!("[INPUT-DBG] key={} (0x{:02X})", key, key);
            // Check modifier state from interrupt handler (tracks raw scancodes)
            let alt = crate::keyboard::is_key_pressed(0x38);
            let _ctrl = crate::keyboard::is_key_pressed(0x1D);
            let win = crate::keyboard::is_key_pressed(0x5B);
            
            // ESC (ASCII 27) → route to handle_keyboard_input which handles
            // start menu close, browser cancel, rename cancel, etc.
            // If no specific handler catches it, close the focused window.
            if key == 27 {
                crate::serial_println!("[GUI] ESC pressed");
                // Try-lock to avoid blocking the render loop; skip if locked
                if let Some(mut d) = DESKTOP.try_lock() {
                    // In mobile mode, ESC always exits back to shell
                    if d.mobile_state.active {
                        crate::serial_println!("[GUI] ESC: mobile mode, exiting to shell");
                        drop(d);
                        EXIT_DESKTOP_FLAG.store(true, Ordering::SeqCst);
                        continue;
                    }
                    // If start menu is open, close it
                    if d.start_menu_open {
                        d.start_menu_open = false;
                        d.start_menu_search.clear();
                        d.start_menu_selected = -1;
                        crate::serial_println!("[GUI] ESC: closed start menu");
                        drop(d);
                        continue;
                    }
                    // If focused window is TextEditor with active dialog, route ESC to editor
                    let editor_needs_esc = {
                        let focused = d.windows.iter().find(|w| w.focused && !w.minimized);
                        if let Some(w) = focused {
                            if w.window_type == WindowType::TextEditor {
                                if let Some(editor) = d.editor_states.get(&w.id) {
                                    editor.find_query.is_some() || editor.goto_line_input.is_some()
                                } else { false }
                            } else { false }
                        } else { false }
                    };
                    if editor_needs_esc {
                        let wid = d.windows.iter().find(|w| w.focused && !w.minimized).map(|w| w.id);
                        if let Some(id) = wid {
                            if let Some(editor) = d.editor_states.get_mut(&id) {
                                editor.handle_key(27);
                            }
                        }
                        drop(d);
                        continue;
                    }
                    // If focused window is Browser, route ESC to browser handler
                    // (cancel loading or clear URL) instead of closing
                    let browser_needs_esc = {
                        let focused = d.windows.iter().find(|w| w.focused && !w.minimized);
                        focused.map(|w| w.window_type == WindowType::Browser).unwrap_or(false)
                    };
                    if browser_needs_esc {
                        d.handle_keyboard_input(27);
                        drop(d);
                        continue;
                    }
                    // Otherwise close focused window (if any)
                    let focused_info = d.windows.iter().find(|w| w.focused && !w.minimized).map(|w| w.id);
                    if let Some(wid) = focused_info {
                        crate::serial_println!("[GUI] ESC: closing window {}", wid);
                        d.close_focused_window();
                        crate::serial_println!("[GUI] ESC: window closed OK");
                    } else {
                        crate::serial_println!("[GUI] ESC: no focused window, ignoring");
                    }
                    drop(d);
                } else {
                    crate::serial_println!("[GUI] ESC: lock busy, skipping");
                }
                continue;
            }
            
            // F1 → toggle shortcut overlay
            // F1 scancode 0x3B — check via is_key_pressed
            {
                static mut F1_HANDLED: bool = false;
                let f1_down = crate::keyboard::is_key_pressed(0x3B);
                unsafe {
                    if f1_down && !alt && !win && !F1_HANDLED {
                        F1_HANDLED = true;
                        let mut d = DESKTOP.lock();
                        d.show_shortcuts = !d.show_shortcuts;
                        crate::serial_println!("[GUI] F1: shortcuts overlay = {}", d.show_shortcuts);
                        drop(d);
                    }
                    if !f1_down { F1_HANDLED = false; }
                }
            }
            
            // Alt+Tab, Win+Tab, or Ctrl+Tab → window switcher
            if (alt || win || _ctrl) && key == 9 {
                if !engine::is_alt_tab_active() {
                    engine::start_alt_tab();
                } else {
                    engine::alt_tab_next();
                }
                continue;
            }
            
            // Win+Left Arrow → snap window to left half
            if win && key == crate::keyboard::KEY_LEFT {
                DESKTOP.lock().snap_focused_window(SnapDir::Left);
                unsafe { WIN_USED_COMBO = true; }
                continue;
            }
            // Win+Right Arrow → snap window to right half
            if win && key == crate::keyboard::KEY_RIGHT {
                DESKTOP.lock().snap_focused_window(SnapDir::Right);
                unsafe { WIN_USED_COMBO = true; }
                continue;
            }
            // Win+Up Arrow → maximize focused window
            if win && key == crate::keyboard::KEY_UP {
                DESKTOP.lock().toggle_maximize_focused();
                unsafe { WIN_USED_COMBO = true; }
                continue;
            }
            // Win+Down Arrow → minimize focused window
            if win && key == crate::keyboard::KEY_DOWN {
                DESKTOP.lock().minimize_focused_window();
                unsafe { WIN_USED_COMBO = true; }
                continue;
            }
            
            // Win+D → toggle show desktop (minimize/restore all)
            if win && (key == b'd' || key == b'D') {
                DESKTOP.lock().toggle_show_desktop();
                unsafe { WIN_USED_COMBO = true; }
                crate::serial_println!("[GUI] Win+D: toggle show desktop");
                continue;
            }
            
            // Win+E → open file manager
            if win && (key == b'e' || key == b'E') {
                DESKTOP.lock().create_window("File Explorer", 100, 60, 780, 520, WindowType::FileManager);
                unsafe { WIN_USED_COMBO = true; }
                crate::serial_println!("[GUI] Win+E: open file manager");
                continue;
            }
            
            // Win+I → open settings
            if win && (key == b'i' || key == b'I') {
                DESKTOP.lock().open_settings_panel();
                unsafe { WIN_USED_COMBO = true; }
                crate::serial_println!("[GUI] Win+I: open settings");
                continue;
            }
            
            // Win+H → toggle high contrast (accessibility)
            if win && (key == b'h' || key == b'H') {
                crate::accessibility::toggle_high_contrast();
                let mut d = DESKTOP.lock();
                d.needs_full_redraw = true;
                d.background_cached = false;
                drop(d);
                unsafe { WIN_USED_COMBO = true; }
                crate::serial_println!("[GUI] Win+H: toggle high contrast");
                continue;
            }
            
            // Win+L → lock screen
            if win && (key == b'l' || key == b'L') {
                let mut d = DESKTOP.lock();
                d.lock_screen_active = true;
                d.lock_screen_input.clear();
                d.lock_screen_shake = 0;
                drop(d);
                unsafe { WIN_USED_COMBO = true; }
                crate::serial_println!("[GUI] Win+L: lock screen");
                continue;
            }
            
            // Mark any Win+key combo as used so Win release doesn't toggle start menu
            if win && key != 0 {
                unsafe { WIN_USED_COMBO = true; }
            }
            
            // Alt+F4 → close focused window
            // F4 scancode is 0x3E — check via is_key_pressed since F4 has no ASCII
            if alt && crate::keyboard::is_key_pressed(0x3E) {
                let mut d = DESKTOP.lock();
                let has_focused = d.windows.iter().any(|w| w.focused && !w.minimized);
                if has_focused {
                    d.close_focused_window();
                    crate::serial_println!("[GUI] Alt+F4: closed focused window");
                } else {
                    crate::serial_println!("[GUI] Alt+F4: no window, exiting desktop");
                    EXIT_DESKTOP_FLAG.store(true, Ordering::SeqCst);
                }
                drop(d);
                continue;
            }
            
            // Pass key to focused window
            crate::serial_println!("[MAIN-DBG] passing key {} (0x{:02X}) to handle_keyboard", key, key);
            handle_keyboard(key);
        }
        
        // Handle Alt/Win/Ctrl release to finish Alt+Tab / Win+Tab / Ctrl+Tab
        if engine::is_alt_tab_active() {
            let alt_held = crate::keyboard::is_key_pressed(0x38);
            let win_held = crate::keyboard::is_key_pressed(0x5B);
            let ctrl_held = crate::keyboard::is_key_pressed(0x1D);
            if !alt_held && !win_held && !ctrl_held {
                let selected = engine::finish_alt_tab();
                DESKTOP.lock().focus_window_by_index(selected as usize);
            }
        }
        
        // Win key alone (press & release) → toggle start menu
        // (statics shared with keyboard loop above)
        static mut LAST_WIN: bool = false;
        static mut WIN_USED_COMBO: bool = false;
        {
            let win_now = crate::keyboard::is_key_pressed(0x5B);
            unsafe {
                if win_now && !LAST_WIN {
                    // Win just pressed — reset combo flag
                    WIN_USED_COMBO = false;
                }
                if win_now {
                    // If Tab is pressed while Win held, mark as combo
                    if engine::is_alt_tab_active() {
                        WIN_USED_COMBO = true;
                    }
                }
                if !win_now && LAST_WIN && !WIN_USED_COMBO {
                    // Win released, was not part of a combo → toggle start menu
                    let mut d = DESKTOP.lock();
                    d.start_menu_open = !d.start_menu_open;
                }
                LAST_WIN = win_now;
            }
        }
        
        // Handle left mouse button
        static mut LAST_LEFT: bool = false;
        let left = mouse.left_button;
        unsafe {
            if left != LAST_LEFT {
                if left {
                    crate::serial_println!("[INPUT-DBG] mouse click at ({},{})", mouse.x, mouse.y);
                }
                // Close start menu on click outside
                if left {
                    let mut d = DESKTOP.lock();
                    if d.start_menu_open {
                        // Will be handled by Desktop's own click handler
                    }
                    drop(d);
                }
                handle_click(mouse.x, mouse.y, left);
                LAST_LEFT = left;
            }
        }
        
        // Handle right mouse button
        static mut LAST_RIGHT: bool = false;
        static mut RIGHT_INIT: bool = false;
        let right = mouse.right_button;
        unsafe {
            if !RIGHT_INIT {
                // Sync initial state to prevent false trigger on first frame
                LAST_RIGHT = right;
                RIGHT_INIT = true;
            }
            if right != LAST_RIGHT {
                handle_right_click(mouse.x, mouse.y, right);
                LAST_RIGHT = right;
            }
        }
        
        // Handle scroll wheel
        let scroll = crate::mouse::get_scroll_delta();
        if scroll != 0 {
            handle_scroll(scroll);
        }
        
        // ═══════════════════════════════════════════════════════════════
        // Touch Input Processing
        // ═══════════════════════════════════════════════════════════════
        {
            let mut d = DESKTOP.lock();
            d.process_touch_input();
            drop(d);
        }
        
        // ═══════════════════════════════════════════════════════════════
        // Poll async JARVIS result
        // ═══════════════════════════════════════════════════════════════
        {
            let result = {
                let mut r = JARVIS_RESULT.lock();
                r.take()
            };
            if let Some(lines) = result {
                let mut d = DESKTOP.lock();
                // Find the focused terminal window and append JARVIS output
                if let Some(window) = d.windows.iter_mut().find(|w| w.window_type == WindowType::Terminal) {
                    // Remove the cursor/prompt line before appending
                    if window.content.last().map(|s| s.contains("$ ")).unwrap_or(false) {
                        window.content.pop();
                    }
                    // Append result lines
                    for line in &lines {
                        window.content.push(line.clone());
                    }
                    // Re-add prompt
                    window.content.push(Desktop::make_prompt("_"));
                    // Auto-scroll to bottom
                    let line_height = 16usize;
                    let content_area_h = (window.height as usize).saturating_sub(TITLE_BAR_HEIGHT as usize + 16);
                    let visible_lines = if line_height > 0 { content_area_h / line_height } else { 1 };
                    if window.content.len() > visible_lines {
                        window.scroll_offset = window.content.len() - visible_lines;
                    } else {
                        window.scroll_offset = 0;
                    }
                }
                drop(d);
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // Poll async browser navigation result
        // ═══════════════════════════════════════════════════════════════
        {
            let result = {
                let mut r = BROWSER_NAV_RESULT.lock();
                r.take()
            };
            if let Some(nav_result) = result {
                let mut d = DESKTOP.lock();
                match nav_result {
                    Ok((final_url, status_code, headers, body)) => {
                        crate::serial_println!("[BROWSER-BG] Received {} bytes, status {}", body.len(), status_code);
                        if let Some(ref mut browser) = d.browser {
                            if status_code >= 400 {
                                browser.status = crate::browser::BrowserStatus::Error(alloc::format!("HTTP {}", status_code));
                                browser.raw_html = alloc::format!(
                                    "<html><body><h1>HTTP Error {}</h1><p>The server returned an error for {}</p></body></html>",
                                    status_code, final_url
                                );
                                browser.document = Some(crate::browser::parse_html(&browser.raw_html));
                            } else if status_code >= 300 && status_code < 400 {
                                // Redirect — check Location header
                                let location = headers.iter()
                                    .find(|(k, _)| k.to_lowercase() == "location")
                                    .map(|(_, v)| v.clone());
                                if let Some(loc) = location {
                                    crate::serial_println!("[BROWSER-BG] Redirect {} -> {}", status_code, loc);
                                    d.browser_url_input = loc.clone();
                                    d.browser_url_cursor = d.browser_url_input.len();
                                    // Re-queue the redirect URL
                                    {
                                        let mut pending = BROWSER_PENDING_URL.lock();
                                        *pending = Some(loc);
                                    }
                                    BROWSER_NAV_BUSY.store(true, Ordering::SeqCst);
                                    crate::thread::spawn_kernel("browser-nav", browser_nav_worker, 0);
                                    drop(d);
                                    // Don't clear browser_loading — still navigating
                                    continue;
                                }
                            } else {
                                // Success — parse HTML and update browser state
                                let html = core::str::from_utf8(&body).unwrap_or("");
                                browser.raw_html = String::from(html);
                                browser.process_set_cookies(&headers, &final_url);
                                browser.document = Some(crate::browser::parse_html(html));
                                browser.execute_scripts();
                                browser.extract_resources(&final_url);
                                // Update history
                                if browser.history_index < browser.history.len() {
                                    browser.history.truncate(browser.history_index);
                                }
                                browser.history.push(final_url.clone());
                                browser.history_index = browser.history.len();
                                browser.current_url = final_url.clone();
                                browser.scroll_y = 0;
                                browser.status = crate::browser::BrowserStatus::Ready;
                            }
                            d.browser_url_input = browser.current_url.clone();
                            d.browser_url_cursor = d.browser_url_input.len();
                        }
                    }
                    Err(e) => {
                        crate::serial_println!("[BROWSER-BG] Navigation error: {}", e);
                        if let Some(ref mut browser) = d.browser {
                            browser.status = crate::browser::BrowserStatus::Error(e.clone());
                            browser.raw_html = alloc::format!(
                                "<html><body><h1>Error</h1><p>{}</p></body></html>", e
                            );
                            browser.document = Some(crate::browser::parse_html(&browser.raw_html));
                            browser.scroll_y = 0;
                        }
                    }
                }
                d.browser_loading = false;
                drop(d);
            }
        }

        // ═══════════════════════════════════════════════════════════════
        // Rendering
        // ═══════════════════════════════════════════════════════════════
        
        // Render main desktop
        draw();
        
        // Render Alt+Tab overlay if active
        if engine::is_alt_tab_active() {
            render_alt_tab_overlay();
        }
        
        // Render Start Menu if open (Desktop's own start menu handles this)
        // Note: gui::engine start menu disabled to avoid dual-menu conflict
        // if engine::is_start_menu_open() {
        //     render_start_menu();
        // }
        
        // Render shortcut overlay if active
        {
            let d = DESKTOP.lock();
            let show = d.show_shortcuts;
            drop(d);
            if show {
                render_shortcut_overlay();
            }
        }
        
        // Render notifications
        render_notifications();
        
        // Render developer panel overlay (F12 toggle)
        {
            let d = DESKTOP.lock();
            let w = d.width;
            let h = d.height;
            drop(d);
            let frame_elapsed = engine::now_us().saturating_sub(frame_start);
            crate::devtools::render_devpanel(w, h, frame_elapsed);
        }
        
        // Render FPS counter (debug)
        #[cfg(debug_assertions)]
        {
            let fps = engine::get_fps();
            // Draw FPS in corner (optional)
        }
        
        // ═══════════════════════════════════════════════════════════════
        // VSync frame pacing (spin-loop sleep with bail-out)
        // ═══════════════════════════════════════════════════════════════
        let render_time_us = engine::now_us().saturating_sub(frame_start);
        // Log FPS to serial every 120 frames for performance monitoring
        {
            let d = DESKTOP.lock();
            let fc = d.frame_count;
            drop(d);
            if fc % 120 == 0 && fc > 0 {
                let fps = crate::gui::vsync::fps();
                crate::serial_println!("[PERF] frame={} render={}us fps={}", fc, render_time_us, fps);
            }
        }
        crate::gui::vsync::frame_end(frame_start);
        loop_frame = loop_frame.saturating_add(1);
    }
    
    // ═══════════════════════════════════════════════════════════════
    // Cleanup before returning to shell
    // ═══════════════════════════════════════════════════════════════
    crate::serial_println!("[GUI] Desktop exiting, cleaning up...");
    // Stop all music playback to prevent HDA DMA writing to freed memory
    {
        let mut d = DESKTOP.lock();
        for (_id, mp) in d.music_player_states.iter_mut() {
            mp.stop();
        }
        crate::serial_println!("[GUI] All music players stopped");
    }
    crate::framebuffer::set_double_buffer_mode(false);
    crate::framebuffer::clear();
    crate::serial_println!("[GUI] Desktop exited cleanly");
}

/// Snap direction for window snapping
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SnapDir {
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Render Alt+Tab overlay — polished window switcher
fn render_alt_tab_overlay() {
    let desktop = DESKTOP.lock();
    let win_info = desktop.get_window_info();
    if win_info.is_empty() { return; }
    
    let screen_w = desktop.width;
    let screen_h = desktop.height;
    drop(desktop);
    
    let selection = crate::gui::engine::alt_tab_selection();
    let count = win_info.len() as i32;
    let idx = ((selection % count) + count) % count;
    
    // Card dimensions
    let card_w: u32 = 150;
    let card_h: u32 = 100;
    let gap: u32 = 12;
    let max_visible: u32 = 6; // Max cards shown at once
    let visible_count = (win_info.len() as u32).min(max_visible);
    let total_w = visible_count * (card_w + gap) + gap;
    let title_area: u32 = 30;
    let total_h = card_h + gap * 2 + title_area + 14;
    
    let ox = (screen_w as i32 - total_w as i32) / 2;
    let oy = (screen_h as i32 - total_h as i32) / 2;
    
    // ── Backdrop (dark glass) ──
    draw_rounded_rect(ox - 2, oy - 2, total_w + 4, total_h + 4, 14, 0x40000000);
    draw_rounded_rect(ox, oy, total_w, total_h, 12, 0xE8101420);
    // Subtle border
    draw_rounded_rect_border(ox, oy, total_w, total_h, 12, 0x3000FF66);
    // Glass highlight (top edge)
    fill_rect_signed(ox + 14, oy + 1, total_w as i32 - 28, 1, 0x20FFFFFF);
    
    // ── Title ──
    draw_text_centered(ox + total_w as i32 / 2, oy + 8, "Switch Window", 0xFF888888);
    
    // ── Window cards ──
    for (i, (title, wtype)) in win_info.iter().enumerate() {
        if i as u32 >= max_visible { break; }
        let cx = ox + gap as i32 + i as i32 * (card_w + gap) as i32;
        let cy = oy + gap as i32 + 22;
        
        let is_selected = i as i32 == idx;
        
        // Card background
        if is_selected {
            // Glow behind selected
            draw_rounded_rect(cx - 2, cy - 2, card_w + 4, card_h + 4, 8, 0x3000FF66);
            draw_rounded_rect(cx, cy, card_w, card_h, 6, 0xFF1A2A20);
            // Green accent border
            draw_rounded_rect_border(cx, cy, card_w, card_h, 6, 0xFF00CC55);
        } else {
            draw_rounded_rect(cx, cy, card_w, card_h, 6, 0xFF1A1E28);
            draw_rounded_rect_border(cx, cy, card_w, card_h, 6, 0xFF2A2E38);
        }
        
        // Window type icon (centered in card, large)
        let icon = window_type_icon(*wtype);
        let icon_x = cx + (card_w as i32 - crate::graphics::scaling::measure_text_width(icon) as i32) / 2;
        let icon_y = cy + 20;
        let icon_color = if is_selected { 0xFF00FF66 } else { 0xFF667766 };
        draw_text(icon_x, icon_y, icon, icon_color);
        
        // Type label (small, below icon)
        let type_label = window_type_label(*wtype);
        let label_color = if is_selected { 0xFF00CC55 } else { 0xFF555555 };
        draw_text_centered(cx + card_w as i32 / 2, cy + card_h as i32 - 22, type_label, label_color);
        
        // Window title (below card)
        let short: alloc::string::String = title.chars().take(16).collect();
        let title_color = if is_selected { 0xFFFFFFFF } else { 0xFF999999 };
        draw_text_centered(cx + card_w as i32 / 2, cy + card_h as i32 + 6, &short, title_color);
    }
    
    // ── Navigation hint ──
    if win_info.len() > 1 {
        draw_text_centered(ox + total_w as i32 / 2, oy + total_h as i32 - 18, 
            "Tab: next  |  Release Alt: select", 0xFF555555);
    }
}

/// Render shortcut overlay — keyboard shortcut cheat sheet (F1 toggle)
fn render_shortcut_overlay() {
    let desktop = DESKTOP.lock();
    let screen_w = desktop.width;
    let screen_h = desktop.height;
    drop(desktop);

    // Shortcut categories with entries: (category, &[(key, description)])
    let categories: &[(&str, &[(&str, &str)])] = &[
        ("Navigation", &[
            ("Win", "Toggle Start Menu"),
            ("Alt+Tab", "Switch Windows"),
            ("Win+D", "Show Desktop"),
            ("Win+L", "Lock Screen"),
            ("ESC", "Close Window / Menu"),
            ("Alt+F4", "Force Close Window"),
        ]),
        ("Windows", &[
            ("Win+Left", "Snap Left"),
            ("Win+Right", "Snap Right"),
            ("Win+Up", "Maximize"),
            ("Win+Down", "Minimize"),
        ]),
        ("Apps", &[
            ("Win+E", "File Manager"),
            ("Win+I", "Settings"),
            ("Win+H", "High Contrast"),
        ]),
        ("Editor", &[
            ("Ctrl+S", "Save"),
            ("Ctrl+F", "Find"),
            ("Ctrl+G", "Go to Line"),
            ("Ctrl+C/X/V", "Copy / Cut / Paste"),
        ]),
        ("File Manager", &[
            ("N / D", "New File / New Folder"),
            ("R", "Rename"),
            ("Del", "Delete"),
            ("V", "Toggle View"),
        ]),
    ];

    // Layout: 2-column grid
    let col_count: u32 = 2;
    let col_w: u32 = 300;
    let row_h: u32 = 18;
    let cat_pad: u32 = 8;
    let header_h: u32 = 40;
    let footer_h: u32 = 24;
    let margin: u32 = 20;

    // Calculate total height
    let mut total_rows: u32 = 0;
    for (_, entries) in categories.iter() {
        total_rows += 1 + entries.len() as u32; // category header + entries
    }
    // Split into 2 columns
    let rows_per_col = (total_rows + 1) / 2;
    let content_h = rows_per_col * row_h + ((categories.len() as u32 + 1) / 2) * cat_pad;
    let panel_w = col_count * col_w + margin * 3;
    let panel_h = header_h + content_h + footer_h + margin;

    let ox = (screen_w as i32 - panel_w as i32) / 2;
    let oy = (screen_h as i32 - panel_h as i32) / 2;

    // ── Backdrop (dark glass) ──
    draw_rounded_rect(ox - 2, oy - 2, panel_w + 4, panel_h + 4, 14, 0x50000000);
    draw_rounded_rect(ox, oy, panel_w, panel_h, 12, 0xF0101420);
    draw_rounded_rect_border(ox, oy, panel_w, panel_h, 12, 0x5000FF66);
    // Glass highlight
    fill_rect_signed(ox + 14, oy + 1, panel_w as i32 - 28, 1, 0x20FFFFFF);

    // ── Title ──
    draw_text_centered(ox + panel_w as i32 / 2, oy + 12, "Keyboard Shortcuts", 0xFF00FF66);
    // Underline
    fill_rect_signed(ox + margin as i32, oy + header_h as i32 - 6, panel_w as i32 - margin as i32 * 2, 1, 0x3000FF66);

    // ── Content: 2-column layout ──
    let mut col = 0u32;
    let mut row_in_col = 0u32;
    let mut cat_idx = 0usize;

    for (cat_name, entries) in categories.iter() {
        // Check if this category would overflow current column
        let needed = 1 + entries.len() as u32;
        if row_in_col + needed > rows_per_col && col < col_count - 1 {
            col += 1;
            row_in_col = 0;
        }

        let cx = ox + margin as i32 + col as i32 * (col_w as i32 + margin as i32);
        let cy = oy + header_h as i32 + row_in_col as i32 * row_h as i32 + cat_idx as i32 * cat_pad as i32;

        // Category header
        draw_text(cx, cy, cat_name, 0xFF00CC55);
        row_in_col += 1;

        // Entries
        for (key, desc) in entries.iter() {
            let ey = oy + header_h as i32 + row_in_col as i32 * row_h as i32 + cat_idx as i32 * cat_pad as i32;
            // Key badge
            let key_w = crate::graphics::scaling::measure_text_width(key) as i32 + 10;
            draw_rounded_rect(cx + 4, ey - 1, key_w as u32, 16, 4, 0xFF1A2A20);
            draw_rounded_rect_border(cx + 4, ey - 1, key_w as u32, 16, 4, 0xFF00AA44);
            draw_text(cx + 9, ey + 1, key, 0xFF00FF66);
            // Description
            draw_text(cx + key_w + 14, ey + 1, desc, 0xFFAAAAAA);
            row_in_col += 1;
        }
        cat_idx += 1;
    }

    // ── Footer ──
    draw_text_centered(ox + panel_w as i32 / 2, oy + panel_h as i32 - footer_h as i32 + 4,
        "Press F1 to close", 0xFF555555);
}

/// Get icon for a window type
fn window_type_icon(wtype: WindowType) -> &'static str {
    match wtype {
        WindowType::Terminal => ">_",
        WindowType::SystemInfo => "[i]",
        WindowType::About => "(?)",
        WindowType::Calculator => "[#]",
        WindowType::FileManager => "[/]",
        WindowType::TextEditor => "[=]",
        WindowType::NetworkInfo => "[~]",
        WindowType::Settings => "{*}",
        WindowType::ImageViewer => "[^]",
        WindowType::Browser => "</>",
        WindowType::Game => "[*]",
        WindowType::Chess | WindowType::Chess3D => "[K]",
        WindowType::ModelEditor => "[3D]",
        WindowType::Game3D => "[3D]",
        WindowType::MusicPlayer => "[~]",
        WindowType::LabMode => "{L}",
        WindowType::BinaryViewer => "0x",
        _ => "[.]",
    }
}

/// Get label for a window type
fn window_type_label(wtype: WindowType) -> &'static str {
    match wtype {
        WindowType::Terminal => "Terminal",
        WindowType::SystemInfo => "System",
        WindowType::About => "About",
        WindowType::Calculator => "Calc",
        WindowType::FileManager => "Files",
        WindowType::TextEditor => "Editor",
        WindowType::NetworkInfo => "NetScan",
        WindowType::Settings => "Settings",
        WindowType::ImageViewer => "Images",
        WindowType::Browser => "Browser",
        WindowType::Game => "Snake",
        WindowType::Chess => "Chess",
        WindowType::Chess3D => "Chess 3D",
        WindowType::ModelEditor => "3D Edit",
        WindowType::Game3D => "FPS",
        WindowType::MusicPlayer => "Music",
        WindowType::LabMode => "Lab",
        WindowType::BinaryViewer => "BinView",
        _ => "Window",
    }
}

/// Render Start Menu
fn render_start_menu() {
    use crate::gui::engine::{get_start_menu_items, StartAction};
    
    let desktop = DESKTOP.lock();
    let screen_h = desktop.height;
    drop(desktop);
    
    let menu_w: u32 = 280;
    let menu_h: u32 = 350;
    let x: i32 = 10;
    let y: i32 = screen_h as i32 - TASKBAR_HEIGHT as i32 - menu_h as i32 - 5;
    
    // Background with blur effect simulation
    draw_rounded_rect(x, y, menu_w, menu_h, 12, 0xF0101520);
    draw_rounded_rect(x + 1, y + 1, menu_w - 2, menu_h - 2, 11, 0xF0181C25);
    
    // TrustOS logo/header
    draw_text(x + 20, y + 15, "TrustOS", 0xFF00FF66);
    draw_text(x + 90, y + 15, "v0.1", 0xFF606060);
    
    // Separator
    draw_line(x + 15, y + 35, x + menu_w as i32 - 15, y + 35, 0xFF303540);
    
    // Menu items
    let items = get_start_menu_items();
    let mut iy = y + 45;
    
    for item in items.iter() {
        if item.icon == 255 {
            // Separator
            draw_line(x + 15, iy + 5, x + menu_w as i32 - 15, iy + 5, 0xFF303540);
            iy += 12;
        } else {
            // Menu item
            draw_text(x + 40, iy, &item.name, 0xFFCCCCCC);
            iy += 28;
        }
    }
    
    // Search bar at bottom
    let search_y = y + menu_h as i32 - 45;
    draw_rounded_rect(x + 15, search_y, menu_w - 30, 30, 6, 0xFF252A35);
    draw_text(x + 25, search_y + 7, "Search apps...", 0xFF606060);
}

/// Render toast notifications — polished with slide-in/fade-out
fn render_notifications() {
    use crate::gui::engine::{get_notifications, NotifyPriority};
    
    let desktop = DESKTOP.lock();
    let screen_w = desktop.width;
    drop(desktop);
    
    let notifs = get_notifications();
    if notifs.is_empty() { return; }
    
    let mut y: i32 = 55; // Below any top UI
    
    for toast in notifs.iter() {
        let w: u32 = 320;
        let has_progress = toast.progress.is_some();
        let h: u32 = if has_progress { 78 } else { 64 };
        let opacity = toast.opacity();
        if opacity == 0 { continue; }
        
        // Slide-in from right: first 300ms slides from +40px offset
        let elapsed = toast.elapsed_ms();
        let slide_offset = if elapsed < 300 {
            ((300 - elapsed) * 40 / 300) as i32
        } else {
            0
        };
        let x = screen_w as i32 - w as i32 - 15 + slide_offset;
        
        // Alpha-adjusted background
        let bg_alpha = (opacity as u32 * 0xF0 / 255) << 24;
        let bg_color = bg_alpha | 0x00141820;
        
        // Outer glow (subtle)
        let glow_alpha = (opacity as u32 * 0x18 / 255) << 24;
        draw_rounded_rect(x - 1, y - 1, w + 2, h + 2, 11, glow_alpha | 0x00000000);
        
        // Card background
        draw_rounded_rect(x, y, w, h, 10, bg_color);
        
        // Glass highlight (top edge)
        let glass_alpha = (opacity as u32 * 0x15 / 255) << 24;
        fill_rect_signed(x + 12, y + 1, w as i32 - 24, 1, glass_alpha | 0x00FFFFFF);
        
        // Accent side bar (4px, rounded look via 2 rects)
        let accent_color = toast.get_color();
        let accent_alpha = (opacity as u32 * ((accent_color >> 24) & 0xFF) / 255) << 24;
        let accent_rgb = accent_color & 0x00FFFFFF;
        fill_rect_signed(x + 2, y + 8, 3, h as i32 - 16, accent_alpha | accent_rgb);
        
        // Priority icon
        let icon = match toast.priority {
            NotifyPriority::Info => "[i]",
            NotifyPriority::Warning => "/!\\",
            NotifyPriority::Error => "[X]",
            NotifyPriority::Success => "[v]",
        };
        let icon_alpha = (opacity as u32 * 0xFF / 255) << 24;
        draw_text(x + 14, y + 12, icon, icon_alpha | accent_rgb);
        
        // Title (bold-ish, white)
        let title_alpha = (opacity as u32 * 0xFF / 255) << 24;
        let title_short: alloc::string::String = toast.title.chars().take(28).collect();
        draw_text(x + 48, y + 12, &title_short, title_alpha | 0x00EEEEEE);
        
        // Message (dimmer)
        let msg_alpha = (opacity as u32 * 0xBB / 255) << 24;
        let msg_short: alloc::string::String = toast.message.chars().take(36).collect();
        draw_text(x + 14, y + 34, &msg_short, msg_alpha | 0x00999999);
        
        // Progress bar if present
        if let Some(percent) = toast.progress {
            let bar_y = y + 54;
            let bar_w = w - 28;
            let bar_alpha = (opacity as u32 * 0xFF / 255) << 24;
            draw_rounded_rect(x + 14, bar_y, bar_w, 8, 3, bar_alpha | 0x00252A35);
            let fill_w = (bar_w * percent as u32 / 100).max(1);
            if fill_w > 4 {
                draw_rounded_rect(x + 14, bar_y, fill_w, 8, 3, bar_alpha | 0x0000CC55);
            }
            // Percentage text
            let pct_str = alloc::format!("{}%", percent);
            draw_text(x + w as i32 - 40, bar_y - 1, &pct_str, bar_alpha | 0x00777777);
        }
        
        // Subtle bottom border
        let border_alpha = (opacity as u32 * 0x10 / 255) << 24;
        fill_rect_signed(x + 10, y + h as i32 - 1, w as i32 - 20, 1, border_alpha | 0x00FFFFFF);
        
        y += h as i32 + 8;
    }
}

/// Helper: Draw text (wrapper) — uses scaling module
fn draw_text(x: i32, y: i32, text: &str, color: u32) {
    crate::graphics::scaling::draw_scaled_text(x, y, text, color);
}

/// Helper: Draw centered text — uses scaling module
fn draw_text_centered(cx: i32, y: i32, text: &str, color: u32) {
    let w = crate::graphics::scaling::measure_text_width(text) as i32;
    draw_text(cx - w / 2, y, text, color);
}

/// Helper: Draw line
fn draw_line(x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
    // Simple horizontal/vertical line
    if y1 == y2 {
        let (x_start, x_end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        for x in x_start..=x_end {
            if x >= 0 {
                crate::framebuffer::draw_pixel(x as u32, y1 as u32, color);
            }
        }
    }
}

/// Helper: Draw rect
fn draw_rect(x: i32, y: i32, w: u32, h: u32, color: u32) {
    for dy in 0..h {
        for dx in 0..w {
            crate::framebuffer::draw_pixel((x + dx as i32) as u32, (y + dy as i32) as u32, color);
        }
    }
}

/// Helper: Draw filled rounded rect with proper quarter-circle corners and alpha support
fn draw_rounded_rect(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    let r = radius.min(w / 2).min(h / 2);

    if r == 0 {
        // No rounding — just fill
        if x >= 0 && y >= 0 {
            crate::framebuffer::fill_rect(x as u32, y as u32, w, h, color);
        }
        return;
    }

    let wi = w as i32;
    let hi = h as i32;
    let ri = r as i32;

    // ── Center body (3 rectangles that avoid corners) ──
    // Middle band full width
    fill_rect_signed(x, y + ri, wi, hi - ri * 2, color);
    // Top band between corners
    fill_rect_signed(x + ri, y, wi - ri * 2, ri, color);
    // Bottom band between corners
    fill_rect_signed(x + ri, y + hi - ri, wi - ri * 2, ri, color);

    // ── Quarter-circle corners ──
    // Use filled scanline approach (fast: one hline per dy)
    let r2 = ri * ri;
    for dy in 0..ri {
        // Number of pixels from corner inward that are inside the circle
        let dx = fast_sqrt_i32(r2 - dy * dy);
        // Top-left corner
        fill_rect_signed(x + ri - dx, y + ri - dy - 1, dx, 1, color);
        // Top-right corner
        fill_rect_signed(x + wi - ri, y + ri - dy - 1, dx, 1, color);
        // Bottom-left corner
        fill_rect_signed(x + ri - dx, y + hi - ri + dy, dx, 1, color);
        // Bottom-right corner
        fill_rect_signed(x + wi - ri, y + hi - ri + dy, dx, 1, color);
    }
}

/// Helper: Draw rounded rectangle border (outline only)
fn draw_rounded_rect_border(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    let r = radius.min(w / 2).min(h / 2);
    let wi = w as i32;
    let hi = h as i32;
    let ri = r as i32;

    if r == 0 {
        if x >= 0 && y >= 0 {
            crate::framebuffer::draw_rect(x as u32, y as u32, w, h, color);
        }
        return;
    }

    // Straight edges
    for px in ri..wi - ri {
        put_pixel_signed(x + px, y, color);            // top
        put_pixel_signed(x + px, y + hi - 1, color);   // bottom
    }
    for py in ri..hi - ri {
        put_pixel_signed(x, y + py, color);            // left
        put_pixel_signed(x + wi - 1, y + py, color);   // right
    }

    // Corner arcs (Bresenham midpoint)
    let mut cx = ri;
    let mut cy = 0i32;
    let mut err = 0i32;
    while cx >= cy {
        // Top-left
        put_pixel_signed(x + ri - cx, y + ri - cy, color);
        put_pixel_signed(x + ri - cy, y + ri - cx, color);
        // Top-right
        put_pixel_signed(x + wi - 1 - ri + cx, y + ri - cy, color);
        put_pixel_signed(x + wi - 1 - ri + cy, y + ri - cx, color);
        // Bottom-left
        put_pixel_signed(x + ri - cx, y + hi - 1 - ri + cy, color);
        put_pixel_signed(x + ri - cy, y + hi - 1 - ri + cx, color);
        // Bottom-right
        put_pixel_signed(x + wi - 1 - ri + cx, y + hi - 1 - ri + cy, color);
        put_pixel_signed(x + wi - 1 - ri + cy, y + hi - 1 - ri + cx, color);

        cy += 1;
        err += 1 + 2 * cy;
        if 2 * (err - cx) + 1 > 0 {
            cx -= 1;
            err += 1 - 2 * cx;
        }
    }
}

/// Signed-coord fill_rect helper (clips negative coords)
#[inline]
fn fill_rect_signed(x: i32, y: i32, w: i32, h: i32, color: u32) {
    if w <= 0 || h <= 0 { return; }
    let px = x.max(0) as u32;
    let py = y.max(0) as u32;
    let cw = if x < 0 { (w + x).max(0) as u32 } else { w as u32 };
    let ch = if y < 0 { (h + y).max(0) as u32 } else { h as u32 };
    if cw > 0 && ch > 0 {
        crate::framebuffer::fill_rect(px, py, cw, ch, color);
    }
}

/// Signed-coord pixel helper
#[inline]
fn put_pixel_signed(x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 {
        crate::framebuffer::draw_pixel(x as u32, y as u32, color);
    }
}

/// Integer square root for corner calculations
#[inline]
fn fast_sqrt_i32(v: i32) -> i32 {
    if v <= 0 { return 0; }
    let mut x = v;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + v / x) / 2;
    }
    x
}

/// Read CPU timestamp counter for timing
#[inline]
fn read_tsc() -> u64 {
    crate::arch::timestamp()
}

/// Set the render mode (Classic or OpenGL)
pub fn set_render_mode(mode: RenderMode) {
    DESKTOP.lock().set_render_mode(mode);
    let mode_name = match mode {
        RenderMode::Classic => "Classic",
        RenderMode::OpenGL => "OpenGL Compositor",
        RenderMode::GpuAccelerated => "GPU Accelerated",
    };
    crate::serial_println!("[GUI] Render mode: {}", mode_name);
}

/// Set the compositor theme
pub fn set_theme(theme: CompositorTheme) {
    DESKTOP.lock().set_theme(theme);
    let theme_name = match theme {
        CompositorTheme::Flat => "Flat",
        CompositorTheme::Modern => "Modern",
        CompositorTheme::Glass => "Glass",
        CompositorTheme::Neon => "Neon",
        CompositorTheme::Minimal => "Minimal",
    };
    crate::serial_println!("[GUI] Compositor theme: {}", theme_name);
}

/// Get current render mode
pub fn get_render_mode() -> RenderMode {
    DESKTOP.lock().render_mode
}
