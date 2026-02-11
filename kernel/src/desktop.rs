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
use crate::graphics::desktop_gfx;
use crate::apps::text_editor::{EditorState, render_editor};
use core::sync::atomic::{AtomicBool, Ordering};

/// Module-level flag to signal desktop exit (accessible from run() and handle_menu_action)
static EXIT_DESKTOP_FLAG: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// 🧮 MATH UTILITIES for no_std environment
// ═══════════════════════════════════════════════════════════════════════════════

/// Fast approximate square root using Newton-Raphson method
fn fast_sqrt(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let mut guess = x / 2.0;
    for _ in 0..5 {
        guess = (guess + x / guess) / 2.0;
    }
    guess
}

/// Approximate x^2
#[allow(dead_code)]
fn square(x: f32) -> f32 {
    x * x
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

// Layout constants (official spec)
const TASKBAR_HEIGHT: u32 = 40;
const TITLE_BAR_HEIGHT: u32 = 28;             // Official: 28px
const WINDOW_BORDER_RADIUS: u32 = 6;          // Official: 6px rounded corners
const WINDOW_SHADOW_BLUR: u32 = 12;
const DOCK_ICON_SIZE: u32 = 24;               // Official icon size
const DOCK_WIDTH: u32 = 60;                   // Official: 56-64px

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
    OpenCalculator,
    OpenNetwork,
    OpenGame,
    OpenEditor,
    OpenGL3D,
    OpenBrowser,
    OpenModelEditor,
    OpenGame3D,
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
        px >= self.x && px < self.x + self.width as i32 - 60 &&
        py >= self.y && py < self.y + TITLE_BAR_HEIGHT as i32
    }
    
    /// Check if point is on close button
    pub fn on_close_button(&self, px: i32, py: i32) -> bool {
        let btn_x = self.x + self.width as i32 - 24;
        px >= btn_x && px < btn_x + 18 &&
        py >= self.y + 5 && py < self.y + 23
    }
    
    /// Check if point is on maximize button
    pub fn on_maximize_button(&self, px: i32, py: i32) -> bool {
        let btn_x = self.x + self.width as i32 - 46;
        px >= btn_x && px < btn_x + 18 &&
        py >= self.y + 5 && py < self.y + 23
    }
    
    /// Check if point is on minimize button
    pub fn on_minimize_button(&self, px: i32, py: i32) -> bool {
        let btn_x = self.x + self.width as i32 - 68;
        px >= btn_x && px < btn_x + 18 &&
        py >= self.y + 5 && py < self.y + 23
    }
    
    /// Check if point is on resize edge
    pub fn on_resize_edge(&self, px: i32, py: i32) -> ResizeEdge {
        if self.maximized { return ResizeEdge::None; }
        
        let resize_margin = 8i32;
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
            // Maximize
            self.x = 0;
            self.y = 0;
            self.width = screen_width;
            self.height = screen_height - TASKBAR_HEIGHT;
            self.maximized = true;
        }
    }
}

/// Desktop manager
use crate::graphics::{compositor, Compositor, CompositorTheme, WindowSurface, Easing};

// ═══════════════════════════════════════════════════════════════════════════════
// 🎨 RENDER MODE - Choose between classic and OpenGL compositor
// ═══════════════════════════════════════════════════════════════════════════════

/// Rendering backend for the desktop
#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    /// Classic framebuffer rendering (stable, fast)
    Classic,
    /// OpenGL compositor with effects (modern, customizable)
    OpenGL,
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
    // Browser state
    pub browser: Option<crate::browser::Browser>,
    pub browser_url_input: String,
    pub browser_url_cursor: usize,
    pub browser_loading: bool,
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
    // UI scale factor (1 = native, 2 = HiDPI, 3 = ultra)
    pub scale_factor: u32,
    // Matrix rain state (depth-parallax advancing effect)
    matrix_chars: Vec<u8>,
    matrix_heads: Vec<i32>,
    matrix_speeds: Vec<u32>,
    matrix_seeds: Vec<u32>,
    matrix_initialized: bool,
    // Terminal auto-suggestions: how many suggestion lines added after prompt
    terminal_suggestion_count: usize,
}

/// Calculator state for interactive calculator windows
pub struct CalculatorState {
    pub display: String,
    pub accumulator: f64,
    pub current_input: String,
    pub operator: Option<char>,
    pub just_evaluated: bool,
}

impl CalculatorState {
    pub fn new() -> Self {
        CalculatorState {
            display: String::from("0"),
            accumulator: 0.0,
            current_input: String::new(),
            operator: None,
            just_evaluated: false,
        }
    }
    
    pub fn press_digit(&mut self, d: char) {
        if self.just_evaluated {
            self.current_input.clear();
            self.just_evaluated = false;
        }
        if self.current_input.len() < 12 {
            self.current_input.push(d);
            self.display = self.current_input.clone();
        }
    }
    
    pub fn press_dot(&mut self) {
        if self.just_evaluated {
            self.current_input = String::from("0");
            self.just_evaluated = false;
        }
        if !self.current_input.contains('.') {
            if self.current_input.is_empty() {
                self.current_input.push('0');
            }
            self.current_input.push('.');
            self.display = self.current_input.clone();
        }
    }
    
    pub fn press_operator(&mut self, op: char) {
        if !self.current_input.is_empty() {
            let val = self.parse_input();
            if let Some(prev_op) = self.operator {
                self.accumulator = self.evaluate(self.accumulator, prev_op, val);
            } else {
                self.accumulator = val;
            }
            self.display = self.format_number(self.accumulator);
            self.current_input.clear();
        }
        self.operator = Some(op);
        self.just_evaluated = false;
    }
    
    pub fn press_equals(&mut self) {
        if !self.current_input.is_empty() {
            let val = self.parse_input();
            if let Some(op) = self.operator {
                self.accumulator = self.evaluate(self.accumulator, op, val);
            } else {
                self.accumulator = val;
            }
        }
        self.display = self.format_number(self.accumulator);
        self.current_input.clear();
        self.operator = None;
        self.just_evaluated = true;
    }
    
    pub fn press_clear(&mut self) {
        self.display = String::from("0");
        self.accumulator = 0.0;
        self.current_input.clear();
        self.operator = None;
        self.just_evaluated = false;
    }
    
    pub fn press_backspace(&mut self) {
        if !self.current_input.is_empty() {
            self.current_input.pop();
            if self.current_input.is_empty() {
                self.display = String::from("0");
            } else {
                self.display = self.current_input.clone();
            }
        }
    }
    
    fn parse_input(&self) -> f64 {
        // Simple integer/float parser for no_std
        let s = &self.current_input;
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
    
    fn evaluate(&self, a: f64, op: char, b: f64) -> f64 {
        match op {
            '+' => a + b,
            '-' => a - b,
            '*' => a * b,
            '/' => if b != 0.0 { a / b } else { 0.0 },
            '%' => if b != 0.0 { a % b } else { 0.0 },
            _ => b,
        }
    }
    
    fn format_number(&self, n: f64) -> String {
        // Check if it's an integer
        if n == (n as i64) as f64 && n.abs() < 1e15 {
            format!("{}", n as i64)
        } else {
            // Format with up to 6 decimal places, trimming trailing zeros
            let s = format!("{:.6}", n);
            let s = s.trim_end_matches('0');
            let s = s.trim_end_matches('.');
            String::from(s)
        }
    }
}

/// Snake game state for interactive game windows
pub struct SnakeState {
    pub snake: Vec<(i32, i32)>,
    pub direction: (i32, i32),
    pub food: (i32, i32),
    pub score: u32,
    pub game_over: bool,
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
        loop {
            let fx = (self.next_rng() % self.grid_w as u32) as i32;
            let fy = (self.next_rng() % self.grid_h as u32) as i32;
            if !self.snake.iter().any(|&(sx, sy)| sx == fx && sy == fy) {
                self.food = (fx, fy);
                break;
            }
        }
    }
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT};
        if self.game_over {
            if key == b' ' || key == 0x0D {
                // Restart
                *self = SnakeState::new();
            }
            return;
        }
        match key {
            KEY_UP    if self.direction != (0, 1)  => self.direction = (0, -1),
            KEY_DOWN  if self.direction != (0, -1) => self.direction = (0, 1),
            KEY_LEFT  if self.direction != (1, 0)  => self.direction = (-1, 0),
            KEY_RIGHT if self.direction != (-1, 0) => self.direction = (1, 0),
            _ => {}
        }
    }
    
    pub fn tick(&mut self) {
        if self.game_over { return; }
        self.tick_counter += 1;
        if self.tick_counter < self.speed { return; }
        self.tick_counter = 0;
        
        let head = self.snake[0];
        let new_head = (head.0 + self.direction.0, head.1 + self.direction.1);
        
        // Wall collision
        if new_head.0 < 0 || new_head.0 >= self.grid_w || new_head.1 < 0 || new_head.1 >= self.grid_h {
            self.game_over = true;
            return;
        }
        
        // Self collision
        if self.snake.iter().any(|&s| s == new_head) {
            self.game_over = true;
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
            browser: None,
            browser_url_input: String::new(),
            browser_url_cursor: 0,
            browser_loading: false,
            editor_states: BTreeMap::new(),
            model_editor_states: BTreeMap::new(),
            calculator_states: BTreeMap::new(),
            snake_states: BTreeMap::new(),
            game3d_states: BTreeMap::new(),
            scale_factor: 1,
            matrix_chars: Vec::new(),
            matrix_heads: Vec::new(),
            matrix_speeds: Vec::new(),
            matrix_seeds: Vec::new(),
            matrix_initialized: false,
            terminal_suggestion_count: 0,
        }
    }
    
    /// Initialize desktop with double buffering
    pub fn init(&mut self, width: u32, height: u32) {
        crate::serial_println!("[Desktop] init start: {}x{} (clearing {} windows, {} icons)", 
            width, height, self.windows.len(), self.icons.len());
        
        // ===== FULL STATE RESET (prevents duplication on re-entry) =====
        // Data collections
        self.windows.clear();
        self.icons.clear();
        self.editor_states.clear();
        self.model_editor_states.clear();
        self.calculator_states.clear();
        self.snake_states.clear();
        self.game3d_states.clear();
        // Browser
        self.browser = None;
        self.browser_url_input.clear();
        self.browser_url_cursor = 0;
        self.browser_loading = false;
        // Input / UI state
        self.input_buffer.clear();
        self.start_menu_open = false;
        self.cursor_blink = false;
        self.context_menu.visible = false;
        self.context_menu.items.clear();
        self.context_menu.selected_index = 0;
        self.context_menu.target_icon = None;
        self.context_menu.target_file = None;
        // Counters / tracking
        self.frame_count = 0;
        self.terminal_suggestion_count = 0;
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
        
        // Initialize double buffering
        crate::serial_println!("[Desktop] init_double_buffer...");
        framebuffer::init_double_buffer();
        framebuffer::set_double_buffer_mode(true);
        
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
        
        crate::serial_println!("[Desktop] init complete");
    }
    
    /// Initialize matrix rain background data (depth-parallax advancing effect)
    fn init_matrix_rain(&mut self) {
        // 160 columns for depth-parallax advancing matrix rain
        // Speeds 2-6: slow=far(dim), fast=near(bright) → illusion of advancing
        const MATRIX_COLS: usize = 160;
        const TRAIL_LEN: usize = 30;
        const CHAR_H: usize = 16;
        const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
        
        // Pre-generate character set for trail rendering
        self.matrix_chars = vec![0u8; MATRIX_COLS * TRAIL_LEN];
        self.matrix_heads = vec![0i32; MATRIX_COLS];
        self.matrix_speeds = vec![2u32; MATRIX_COLS];
        self.matrix_seeds = vec![0u32; MATRIX_COLS];
        
        let height = self.height.saturating_sub(TASKBAR_HEIGHT);
        
        for col in 0..MATRIX_COLS {
            let seed = (col as u32).wrapping_mul(2654435761) ^ 0xDEADBEEF;
            // Pre-gen chars for this column's trail
            for i in 0..TRAIL_LEN {
                let char_seed = seed.wrapping_add((i as u32).wrapping_mul(7919));
                self.matrix_chars[col * TRAIL_LEN + i] = CHARS[(char_seed as usize) % CHARS.len()];
            }
            // Start position (randomized, many start off-screen)
            self.matrix_heads[col] = -((seed % (height / 2)) as i32);
            // Speed 1-3 (determines depth: 1=far/dim, 3=near/bright) — slower rain
            self.matrix_speeds[col] = 1 + (seed % 3);
            self.matrix_seeds[col] = seed;
        }
        self.matrix_initialized = true;
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
        
        let id = self.create_window("TrustCode: demo.rs", 200, 80, 720, 520, WindowType::TextEditor);
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
            ("Network", IconType::Network, IconAction::OpenNetwork),
            ("Games", IconType::Game, IconAction::OpenGame),
            ("Browser", IconType::Browser, IconAction::OpenBrowser),
            ("TrustEd", IconType::ModelEditor, IconAction::OpenModelEditor),
            ("Settings", IconType::Settings, IconAction::OpenSettings),
            ("About", IconType::About, IconAction::OpenAbout),
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
        let mut window = Window::new(title, x, y, width, height, wtype);
        
        // Initialize content based on type
        match wtype {
            WindowType::Terminal => {
                window.content.push(String::from("TrustOS Terminal v1.0"));
                window.content.push(String::from(""));
                window.content.push(String::from("root@trustos:~$ _"));
            },
            WindowType::SystemInfo => {
                window.content.push(String::from("=== System Information ==="));
                window.content.push(String::from(""));
                window.content.push(format!("OS: TrustOS v0.1.0"));
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
                // List actual files from ramfs with file type info
                if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(Some("/"))) {
                    for (name, ftype, size) in entries.iter().take(12) {
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
                // Initialize with a new empty editor
                let mut editor = EditorState::new();
                editor.language = crate::apps::text_editor::Language::Plain;
                self.editor_states.insert(window.id, editor);
            },
            WindowType::NetworkInfo => {
                window.content.push(String::from("=== Network Status ==="));
                window.content.push(String::from(""));
                if let Some(mac) = crate::network::get_mac_address() {
                    window.content.push(format!("MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
                } else {
                    window.content.push(String::from("MAC: Not available"));
                }
                window.content.push(String::from("IP: Waiting for DHCP..."));
                window.content.push(String::from(""));
                if crate::virtio_net::is_initialized() {
                    window.content.push(String::from("Driver: virtio-net (active)"));
                } else {
                    window.content.push(String::from("Driver: none"));
                }
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
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            if w.animate_close() {
                // Animation started, window will be removed after animation
                return;
            }
        }
        // No animation, remove immediately
        self.windows.retain(|w| w.id != id);
        // Clean up states
        self.editor_states.remove(&id);
        self.model_editor_states.remove(&id);
        self.calculator_states.remove(&id);
        self.snake_states.remove(&id);
        self.game3d_states.remove(&id);
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
    
    /// Snap focused window to left/right (Win+Arrow)
    pub fn snap_focused_window(&mut self, dir: SnapDir) {
        if let Some(w) = self.windows.iter_mut().rev().find(|w| w.focused) {
            let work_height = self.height - TASKBAR_HEIGHT;
            
            match dir {
                SnapDir::Left => {
                    w.x = 0;
                    w.y = 0;
                    w.width = self.width / 2;
                    w.height = work_height;
                }
                SnapDir::Right => {
                    w.x = (self.width / 2) as i32;
                    w.y = 0;
                    w.width = self.width / 2;
                    w.height = work_height;
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
    
    /// Open a new terminal window
    pub fn open_terminal(&mut self) {
        let id = self.create_window("Terminal", 100, 100, 700, 500, WindowType::Terminal);
        self.focus_window(id);
    }

    /// Handle mouse click
    pub fn handle_click(&mut self, x: i32, y: i32, pressed: bool) {
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
                    self.handle_menu_action(action);
                    return;
                }
                // Click outside menu (but not on taskbar TrustOS button) → close menu
                if y < (self.height - TASKBAR_HEIGHT) as i32 || x >= 108 {
                    self.start_menu_open = false;
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
                        self.handle_browser_click(x, y, &self.windows[i].clone());
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
                                                _ => {}
                                            }
                                        }
                                    }
                                }
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
        } else {
            // Mouse released - stop dragging and resizing
            for w in &mut self.windows {
                w.dragging = false;
                w.resizing = ResizeEdge::None;
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
                ContextMenuItem { label: String::from("  Open"), action: ContextAction::Open },
                ContextMenuItem { label: String::from("  Open With..."), action: ContextAction::OpenWith },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Cut"), action: ContextAction::Cut },
                ContextMenuItem { label: String::from("  Copy"), action: ContextAction::Copy },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Rename"), action: ContextAction::Rename },
                ContextMenuItem { label: String::from("  Delete"), action: ContextAction::Delete },
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
                ContextMenuItem { label: String::from("  Refresh"), action: ContextAction::Refresh },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Paste"), action: ContextAction::Paste },
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
        
        match action {
            ContextAction::Open => {
                if let Some(idx) = self.context_menu.target_icon {
                    let icon_action = self.icons[idx].action;
                    self.handle_icon_action(icon_action);
                }
            },
            ContextAction::OpenWith => {
                // Show file associations window
                self.create_window("Open With", 300 + offset, 200 + offset, 400, 300, WindowType::FileAssociations);
            },
            ContextAction::Refresh => {
                // Redraw desktop - just logs for now
                crate::serial_println!("[GUI] Desktop refreshed");
            },
            ContextAction::NewFile => {
                // Create a new file in ramfs
                let filename = format!("/desktop/newfile_{}.txt", self.frame_count);
                crate::ramfs::with_fs(|fs| {
                    let _ = fs.write_file(&filename, b"New file created from desktop");
                });
                crate::serial_println!("[GUI] Created new file: {}", filename);
            },
            ContextAction::NewFolder => {
                let dirname = format!("/desktop/folder_{}", self.frame_count);
                crate::ramfs::with_fs(|fs| {
                    let _ = fs.mkdir(&dirname);
                });
                crate::serial_println!("[GUI] Created new folder: {}", dirname);
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
                    window.content.push(String::from("OS: TrustOS v0.1.0"));
                }
            },
            ContextAction::Cut => {
                crate::serial_println!("[GUI] Cut (not implemented)");
            },
            ContextAction::Copy => {
                crate::serial_println!("[GUI] Copy (not implemented)");
            },
            ContextAction::Paste => {
                crate::serial_println!("[GUI] Paste (not implemented)");
            },
            ContextAction::ViewLargeIcons | ContextAction::ViewSmallIcons | ContextAction::ViewList => {
                crate::serial_println!("[GUI] View mode changed");
            },
            ContextAction::SortByName | ContextAction::SortByDate | ContextAction::SortBySize => {
                crate::serial_println!("[GUI] Sort order changed");
            },
            ContextAction::Personalize => {
                self.create_window("Personalization", 250 + offset, 150 + offset, 400, 300, WindowType::Settings);
            },
            ContextAction::TerminalHere => {
                self.create_window("Terminal", 200 + offset, 120 + offset, 500, 350, WindowType::Terminal);
            },
            ContextAction::Delete | ContextAction::Rename | ContextAction::CopyPath => {
                crate::serial_println!("[GUI] Action not implemented yet");
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
                self.create_window("Terminal", 150 + offset, 100 + offset, 500, 350, WindowType::Terminal)
            },
            IconAction::OpenFileManager => {
                self.create_window("Files", 180 + offset, 120 + offset, 400, 350, WindowType::FileManager)
            },
            IconAction::OpenCalculator => {
                self.create_window("Calculator", 450 + offset, 150 + offset, 200, 220, WindowType::Calculator)
            },
            IconAction::OpenNetwork => {
                self.create_window("Network", 200 + offset, 140 + offset, 320, 200, WindowType::NetworkInfo)
            },
            IconAction::OpenSettings => {
                self.create_window("Settings", 300 + offset, 160 + offset, 350, 250, WindowType::Settings)
            },
            IconAction::OpenAbout => {
                self.create_window("About TrustOS", 350 + offset, 180 + offset, 350, 200, WindowType::About)
            },
            IconAction::OpenGame => {
                self.create_window("Snake Game", 250 + offset, 120 + offset, 320, 320, WindowType::Game)
            },
            IconAction::OpenEditor => {
                self.create_window("TrustCode", 150 + offset, 80 + offset, 700, 500, WindowType::TextEditor)
            },
            IconAction::OpenGL3D => {
                self.create_window("TrustGL 3D Demo", 150 + offset, 80 + offset, 400, 350, WindowType::Demo3D)
            },
            IconAction::OpenBrowser => {
                self.create_window("TrustBrowser", 120 + offset, 60 + offset, 600, 450, WindowType::Browser)
            },
            IconAction::OpenModelEditor => {
                self.create_window("TrustEdit 3D", 100 + offset, 60 + offset, 700, 500, WindowType::ModelEditor)
            },
            IconAction::OpenGame3D => {
                self.create_window("TrustDoom 3D", 80 + offset, 50 + offset, 640, 480, WindowType::Game3D)
            },
        };
        // Auto-focus newly created window
        self.focus_window(id);
    }
    
    fn handle_taskbar_click(&mut self, x: i32, _y: i32) {
        // TrustOS button (left side, matches draw_taskbar)
        if x >= 4 && x < 112 {
            self.start_menu_open = !self.start_menu_open;
            return;
        }
        
        // Settings button in system tray (gear icon)
        let tray_x = self.width - 120;
        let settings_x = tray_x - 44;
        if x >= settings_x as i32 && x < (settings_x + 40) as i32 {
            self.open_settings_panel();
            return;
        }
        
        // Window buttons — must match the centered layout in draw_taskbar
        let total_btns = self.windows.len();
        if total_btns > 0 {
            let btn_w = 84u32;
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
        // Create new settings window
        self.create_window("Settings", 250, 140, 400, 350, WindowType::Settings);
    }
    
    /// Menu actions enum — must match draw_start_menu layout exactly
    fn check_start_menu_click(&self, x: i32, y: i32) -> Option<u8> {
        // Same dimensions as draw_start_menu()
        let menu_w = 280u32;
        let menu_h = 472u32;
        let menu_x = 4i32;
        let menu_y = (self.height - TASKBAR_HEIGHT - menu_h - 8) as i32;
        
        // Check if click is inside the start menu at all
        if x < menu_x || x >= menu_x + menu_w as i32 || y < menu_y || y >= menu_y + menu_h as i32 {
            return None;
        }
        
        // 13 list items at menu_y + 30 + (i * 32), each 30px tall
        // Matches draw_start_menu items array:
        // 0=Terminal, 1=Files, 2=Calculator, 3=Network, 4=TextEditor,
        // 5=TrustEdit3D, 6=Browser, 7=Snake, 8=TrustDoom3D, 9=Settings, 10=Exit Desktop, 11=Shutdown, 12=Reboot
        let items_start_y = menu_y + 30;
        let item_spacing = 32;
        let item_h = 30;
        
        if y >= items_start_y {
            let idx = ((y - items_start_y) / item_spacing) as u8;
            if idx < 13 {
                // Verify within item height (not in gap)
                let item_top = items_start_y + (idx as i32 * item_spacing);
                if y < item_top + item_h {
                    return Some(idx);
                }
            }
        }
        
        None
    }
    
    fn handle_menu_action(&mut self, action: u8) {
        // Matches draw_start_menu items array order:
        // 0=Terminal, 1=Files, 2=Calculator, 3=Network, 4=TextEditor,
        // 5=TrustEdit3D, 6=Browser, 7=Snake, 8=TrustDoom3D, 9=Settings, 10=Exit Desktop, 11=Shutdown, 12=Reboot
        match action {
            0 => { // Terminal
                let x = 100 + (self.windows.len() as i32 * 30);
                let y = 80 + (self.windows.len() as i32 * 20);
                self.create_window("Terminal", x, y, 500, 350, WindowType::Terminal);
            },
            1 => { // Files
                self.create_window("Files", 150, 100, 400, 350, WindowType::FileManager);
            },
            2 => { // Calculator
                self.create_window("Calculator", 400, 150, 280, 350, WindowType::Calculator);
            },
            3 => { // Network
                self.create_window("Network", 200, 120, 320, 200, WindowType::NetworkInfo);
            },
            4 => { // Text Editor (TrustCode)
                self.create_window("TrustCode", 150, 80, 700, 500, WindowType::TextEditor);
            },
            5 => { // TrustEdit 3D
                self.create_window("TrustEdit 3D", 100, 60, 700, 500, WindowType::ModelEditor);
            },
            6 => { // Browser
                self.create_window("TrustBrowser", 120, 60, 600, 450, WindowType::Browser);
            },
            7 => { // Snake
                self.create_window("Snake Game", 250, 120, 340, 360, WindowType::Game);
            },
            8 => { // TrustDoom 3D
                self.create_window("TrustDoom 3D", 80, 50, 640, 480, WindowType::Game3D);
            },
            9 => { // Settings
                self.open_settings_panel();
            },
            10 => { // Exit Desktop
                crate::serial_println!("[GUI] Exit Desktop from start menu");
                EXIT_DESKTOP_FLAG.store(true, Ordering::SeqCst);
            },
            11 => { // Shutdown
                crate::println!("\n\n=== SYSTEM SHUTDOWN ===");
                crate::println!("Goodbye!");
                loop { x86_64::instructions::hlt(); }
            },
            12 => { // Reboot
                crate::serial_println!("[SYSTEM] Reboot requested");
                // Triple fault reboot
                unsafe {
                    let mut port = x86_64::instructions::port::Port::<u8>::new(0x64);
                    port.write(0xFE);
                }
                loop { x86_64::instructions::hlt(); }
            },
            _ => {}
        }
    }
    
    /// Handle keyboard input for the focused window
    pub fn handle_keyboard_input(&mut self, key: u8) {
        // Extract type and id to avoid borrow conflict
        let focused_info = self.windows.iter().find(|w| w.focused).map(|w| (w.window_type, w.id));
        
        if let Some((wtype, win_id)) = focused_info {
            match wtype {
                WindowType::Terminal => {
                    self.handle_terminal_key(key);
                },
                WindowType::FileManager => {
                    self.handle_filemanager_key(key);
                },
                WindowType::FileAssociations => {
                    self.handle_fileassoc_key(key);
                },
                WindowType::Settings => {
                    self.handle_settings_key(key);
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
                            b'=' | 0x0D | 0x0A => calc.press_equals(), // = or Enter
                            b'c' | b'C' => calc.press_clear(),
                            0x08 => calc.press_backspace(), // Backspace
                            0x7F => calc.press_backspace(), // Delete
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
                WindowType::Browser => {
                    use crate::keyboard::{KEY_LEFT, KEY_RIGHT, KEY_HOME, KEY_END, KEY_DELETE, KEY_PGUP, KEY_PGDOWN};
                    let ctrl = crate::keyboard::is_key_pressed(0x1D);
                    
                    // Don't process keys while loading (except Escape to cancel)
                    if self.browser_loading && key != 0x1B {
                        // Skip input during navigation
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
                        0x0D | 0x0A => { // Enter - navigate
                            if !self.browser_url_input.is_empty() && !self.browser_loading {
                                self.browser_loading = true;
                                let url = self.browser_url_input.clone();
                                if let Some(ref mut browser) = self.browser {
                                    match browser.navigate(&url) {
                                        Ok(()) => {
                                            crate::serial_println!("[DESKTOP] Browser navigated OK");
                                            // Update URL bar with final URL (after redirects)
                                            self.browser_url_input = browser.current_url.clone();
                                            self.browser_url_cursor = self.browser_url_input.len();
                                        }
                                        Err(e) => {
                                            crate::serial_println!("[DESKTOP] Browser navigate error: {}", e);
                                        }
                                    }
                                }
                                self.browser_loading = false;
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
                            // Ctrl+L: select all URL text
                            self.browser_url_cursor = self.browser_url_input.len();
                        },
                        _ if ctrl && (key == b'r' || key == b'R') => {
                            // Ctrl+R or F5: refresh
                            if let Some(ref mut browser) = self.browser {
                                let _ = browser.refresh();
                            }
                        },
                        _ if ctrl && (key == b'a' || key == b'A') => {
                            // Ctrl+A: select all (move cursor to end)
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
                _ => {}
            }
        }
    }
    
    /// Handle file manager keyboard input
    fn handle_filemanager_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN};
        
        let mut action: Option<(String, bool)> = None; // (filename, is_dir)
        
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileManager) {
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
            } else if key == 0x0D || key == 0x0A { // Enter - open file
                // Get selected file
                let idx = window.selected_index + 5; // Skip header
                if idx < window.content.len() - 2 { // Skip footer
                    let line = &window.content[idx];
                    // Parse filename from line format: "  [icon] filename..."
                    if let Some(name_start) = line.find(']') {
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
        
        // Handle action outside the borrow
        if let Some((filename, is_dir)) = action {
            if is_dir {
                // TODO: Navigate into directory
                crate::serial_println!("[FM] Navigate to dir: {}", filename);
            } else {
                self.open_file(&filename);
            }
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
                let id = self.create_window(&format!("View: {}", filename), 180 + offset, 100 + offset, 400, 320, WindowType::ImageViewer);
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.file_path = Some(String::from(filename));
                    window.content.clear();
                    window.content.push(format!("=== Image: {} ===", filename));
                    window.content.push(String::new());
                    // For now, just show file info
                    let file_path = format!("/{}", filename);
                    if let Ok(content) = crate::ramfs::with_fs(|fs| fs.read_file(&file_path).map(|d| d.to_vec())) {
                        window.content.push(format!("Size: {} bytes", content.len()));
                        window.content.push(String::from("Format: Detected from header"));
                        // Check PNG signature
                        if content.len() >= 8 && &content[0..8] == b"\x89PNG\r\n\x1a\n" {
                            window.content.push(String::from("Type: PNG Image"));
                        } else if content.len() >= 2 && &content[0..2] == b"\xFF\xD8" {
                            window.content.push(String::from("Type: JPEG Image"));
                        } else if content.len() >= 2 && &content[0..2] == b"BM" {
                            window.content.push(String::from("Type: BMP Image"));
                        } else {
                            window.content.push(String::from("Type: Unknown"));
                        }
                        window.content.push(String::new());
                        window.content.push(String::from("(Image preview not implemented)"));
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
                        for (i, chunk) in content.chunks(8).enumerate().take(16) {
                            let offset = i * 8;
                            let hex: String = chunk.iter()
                                .map(|b| format!("{:02X} ", b))
                                .collect();
                            let ascii: String = chunk.iter()
                                .map(|&b| if b >= 0x20 && b < 0x7F { b as char } else { '.' })
                                .collect();
                            window.content.push(format!("{:08X} {:<24} {}", offset, hex, ascii));
                        }
                    }
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
        if key == b'1' {
            // Toggle animations
            toggle_animations();
            // Refresh settings window content
            self.refresh_settings_window();
        } else if key == b'2' {
            // Cycle animation speed: 0.5 -> 1.0 -> 2.0 -> 0.5
            let current = *ANIMATION_SPEED.lock();
            let next = if current <= 0.5 { 1.0 } else if current <= 1.0 { 2.0 } else { 0.5 };
            *ANIMATION_SPEED.lock() = next;
            self.refresh_settings_window();
        } else if key == b'3' {
            // Open file associations
            let offset = (self.windows.len() as i32 * 20) % 100;
            self.create_window("File Associations", 250 + offset, 130 + offset, 500, 400, WindowType::FileAssociations);
        } else if key == b'4' {
            // About
            let offset = (self.windows.len() as i32 * 20) % 100;
            self.create_window("About TrustOS", 280 + offset, 150 + offset, 350, 200, WindowType::About);
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
            let line = format!("  > {}", display.join("  "));
            window.content.push(line);
            self.terminal_suggestion_count = 1;
            // If many matches, show count on a second line
            if matches.len() > 6 {
                window.content.push(format!("    ... +{} more", matches.len() - 6));
                self.terminal_suggestion_count = 2;
            }
            while window.content.len() > 20 {
                window.content.remove(0);
            }
        }
    }
    
    /// Handle terminal keyboard input
    fn handle_terminal_key(&mut self, key: u8) {
        // Clear old suggestion lines so content.last() is the prompt again
        self.clear_terminal_suggestions();
        
        if key == 0x08 { // Backspace
            if !self.input_buffer.is_empty() {
                self.input_buffer.pop();
                // Update terminal display after backspace
                if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                    if let Some(last) = window.content.last_mut() {
                        *last = format!("root@trustos:~$ {}_", self.input_buffer);
                    }
                }
            }
        } else if key == 0x09 { // Tab — autosuggestion
            let partial = self.input_buffer.clone();
            if !partial.is_empty() {
                let commands = crate::shell::SHELL_COMMANDS;
                let partial_str = partial.as_str();
                let matches: Vec<&str> = commands.iter().copied().filter(|c| c.starts_with(partial_str)).collect();
                if matches.len() == 1 {
                    self.input_buffer = String::from(matches[0]);
                    if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                        if let Some(last) = window.content.last_mut() {
                            *last = format!("root@trustos:~$ {}_", self.input_buffer);
                        }
                    }
                } else if matches.len() > 1 {
                    // Show all matching commands
                    let match_str: String = matches.join("  ");
                    if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                        window.content.push(match_str);
                        window.content.push(format!("root@trustos:~$ {}_", self.input_buffer));
                        while window.content.len() > 18 { window.content.remove(0); }
                    }
                }
            }
        } else if key == 0x0D || key == 0x0A { // Enter
            let cmd = self.input_buffer.clone();
            self.input_buffer.clear();
            
            // Execute command first (before borrowing windows)
            let output = Self::execute_command_static(&cmd);
            
            // Add command to terminal output
            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                // Remove cursor line
                if window.content.last().map(|s| s.ends_with("$ _")).unwrap_or(false) {
                    window.content.pop();
                }
                window.content.push(format!("root@trustos:~$ {}", cmd));
                
                for line in output {
                    window.content.push(line);
                }
                
                // Add new prompt
                window.content.push(String::from("root@trustos:~$ _"));
                
                // Scroll if needed (keep last 20 lines visible)
                while window.content.len() > 18 {
                    window.content.remove(0);
                }
            }
        } else if key >= 0x20 && key < 0x7F {
            self.input_buffer.push(key as char);
            
            // Update terminal display
            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                // Update prompt line with current input
                if let Some(last) = window.content.last_mut() {
                    *last = format!("root@trustos:~$ {}_", self.input_buffer);
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
                output.push(String::from("TrustOS GUI Terminal - Commands:"));
                output.push(String::from("  help      - Show this help"));
                output.push(String::from("  ls [dir]  - List directory contents"));
                output.push(String::from("  cd <dir>  - Change directory"));
                output.push(String::from("  pwd       - Print working directory"));
                output.push(String::from("  cat <file>- Show file contents"));
                output.push(String::from("  mkdir     - Create directory"));
                output.push(String::from("  touch     - Create file"));
                output.push(String::from("  rm        - Remove file"));
                output.push(String::from("  date      - Show current date/time"));
                output.push(String::from("  uname     - System information"));
                output.push(String::from("  free      - Memory usage"));
                output.push(String::from("  net       - Network status"));
                output.push(String::from("  ps        - List processes"));
                output.push(String::from("  uptime    - System uptime"));
                output.push(String::from("  matrix3d  - 3D Matrix tunnel (ESC=exit)"));
                output.push(String::from("  shader    - Graphics demos"));
                output.push(String::from("  showcase3d- 3D cinematic demo"));
                output.push(String::from("  filled3d  - Filled 3D test (flat shading)"));
                output.push(String::from("  clear     - Clear terminal"));
                output.push(String::from("  exit      - Close terminal"));
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
            "ls" | "ls /" => {
                output.push(String::from("Directory: /"));
                if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(Some("/"))) {
                    for (name, ftype, size) in entries.iter().take(15) {
                        let icon = if *ftype == crate::ramfs::FileType::Directory { "📁" } else { "📄" };
                        output.push(format!("  {} {} ({} bytes)", icon, name, size));
                    }
                    if entries.is_empty() {
                        output.push(String::from("  (empty directory)"));
                    }
                }
            },
            _ if cmd.starts_with("ls ") => {
                let path = &cmd[3..];
                output.push(format!("Directory: {}", path));
                if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(Some(path))) {
                    for (name, ftype, size) in entries.iter().take(15) {
                        let icon = if *ftype == crate::ramfs::FileType::Directory { "📁" } else { "📄" };
                        output.push(format!("  {} {} ({} bytes)", icon, name, size));
                    }
                } else {
                    output.push(format!("  Error: cannot access '{}'", path));
                }
            },
            "pwd" => {
                output.push(String::from("/"));
            },
            "clear" => {
                // Will be handled specially
            },
            "date" => {
                let dt = crate::rtc::read_rtc();
                output.push(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", 
                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second));
            },
            "uname" | "uname -a" => {
                output.push(String::from("TrustOS 0.1.1 x86_64 Rust Kernel"));
                output.push(format!("Heap: {} MB", crate::memory::heap_size() / 1024 / 1024));
            },
            "whoami" => {
                output.push(String::from("root"));
            },
            "free" | "mem" => {
                let heap_mb = crate::memory::heap_size() / 1024 / 1024;
                output.push(String::from("Memory Usage:"));
                output.push(format!("  Heap Size: {} MB", heap_mb));
                output.push(String::from("  Kernel:   Active"));
            },
            "net" | "ifconfig" | "ip" => {
                output.push(String::from("Network Status:"));
                if crate::network::is_available() {
                    if let Some((mac, ip, _state)) = crate::network::get_interface() {
                        output.push(format!("  MAC: {}", mac));
                        if let Some(ip) = ip {
                            output.push(format!("  IP:  {}", ip));
                        }
                        output.push(String::from("  Status: Connected"));
                    }
                } else {
                    output.push(String::from("  Status: No network"));
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
            _ if cmd.starts_with("cd ") => {
                let path = &cmd[3..];
                match crate::ramfs::with_fs(|fs| fs.cd(path)) {
                    Ok(()) => output.push(format!("Changed to: {}", path)),
                    Err(e) => output.push(format!("cd: {}: {}", path, e.as_str())),
                }
            },
            _ if cmd.starts_with("mkdir ") => {
                let path = &cmd[6..];
                match crate::ramfs::with_fs(|fs| fs.mkdir(path)) {
                    Ok(()) => output.push(format!("Created directory: {}", path)),
                    Err(e) => output.push(format!("mkdir: {}: {}", path, e.as_str())),
                }
            },
            _ if cmd.starts_with("touch ") => {
                let path = &cmd[6..];
                match crate::ramfs::with_fs(|fs| fs.touch(path)) {
                    Ok(()) => output.push(format!("Created file: {}", path)),
                    Err(e) => output.push(format!("touch: {}: {}", path, e.as_str())),
                }
            },
            _ if cmd.starts_with("rm ") => {
                let path = &cmd[3..];
                match crate::ramfs::with_fs(|fs| fs.rm(path)) {
                    Ok(()) => output.push(format!("Removed: {}", path)),
                    Err(e) => output.push(format!("rm: {}: {}", path, e.as_str())),
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
            "ps" | "procs" => {
                output.push(String::from("PID  STATE    NAME"));
                output.push(String::from("  1  Running  init"));
                output.push(String::from("  2  Running  desktop"));
            },
            "uptime" => {
                let ticks = crate::logger::get_ticks();
                let secs = ticks / 100;
                let mins = secs / 60;
                output.push(format!("Uptime: {}m {}s", mins, secs % 60));
            },
            "df" => {
                output.push(String::from("Filesystem      Size  Used  Avail Use%"));
                output.push(String::from("ramfs           32M   1M    31M   3%"));
            },
            "showcase3d" | "demo3d" => {
                output.push(String::from("\u{2713} Showcase 3D Cinematic - ESC to skip scenes"));
                drop(output);
                crate::shell::cmd_showcase3d();
                return Vec::new();
            },
            "filled3d" => {
                output.push(String::from("\u{2713} Filled 3D Test - ESC to exit"));
                drop(output);
                crate::shell::cmd_filled3d();
                return Vec::new();
            },
            "exit" | "quit" => {
                output.push(String::from("Use the X button to close the terminal"));
            },
            _ => {
                output.push(format!("Command not found: {}", cmd));
                output.push(String::from("Type 'help' for available commands"));
            },
        }
        
        output
    }
    
    /// Handle mouse move
    pub fn handle_move(&mut self, x: i32, y: i32) {
        self.cursor_x = x.clamp(0, self.width as i32 - 1);
        self.cursor_y = y.clamp(0, self.height as i32 - 1);
        
        for w in &mut self.windows {
            // Handle window dragging
            if w.dragging && !w.maximized {
                w.x = (x - w.drag_offset_x).max(0).min(self.width as i32 - 50);
                w.y = (y - w.drag_offset_y).max(0).min(self.height as i32 - TASKBAR_HEIGHT as i32 - TITLE_BAR_HEIGHT as i32);
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
    
    /// Draw the desktop with double buffering
    pub fn draw(&mut self) {
        self.frame_count += 1;
        
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
        
        // Toggle cursor blink every ~45 frames (slower for readability)
        if self.frame_count % 45 == 0 {
            self.cursor_blink = !self.cursor_blink;
        }
        
        // Get mouse state
        let mouse = crate::mouse::get_state();
        
        // Use OpenGL compositor if enabled
        if self.render_mode == RenderMode::OpenGL {
            self.draw_opengl();
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
        
        // OPTIMIZATION 1: Background caching
        // Only draw background once, then cache it
        // Matrix rain is animated — redraw background every frame
        framebuffer::clear_backbuffer(0xFF000000);
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
        
        // Draw cursor last
        self.draw_cursor();
        
        // Update tracking state
        self.last_cursor_x = mouse.x;
        self.last_cursor_y = mouse.y;
        self.last_window_count = self.windows.len();
        self.last_start_menu_open = self.start_menu_open;
        self.last_context_menu_visible = self.context_menu.visible;
        
        // Swap buffers for flicker-free display
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
        
        // Rounded border
        draw_rounded_rect_border(
            menu_x, menu_y,
            menu_width as u32, menu_height as u32,
            corner_r, GREEN_MUTED,
        );
        
        // Bright top edge (slightly inset for rounded look)
        crate::framebuffer::fill_rect(
            (menu_x + corner_r as i32) as u32, menu_y as u32,
            (menu_width - corner_r as i32 * 2) as u32, 1, GREEN_TERTIARY,
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
    
    fn draw_background(&mut self) {
        // ═══════════════════════════════════════════════════════════════
        // MATRIX RAIN — Slow, atmospheric depth-parallax
        // Slow columns = FAR (dim, desaturated)
        // Fast columns = NEAR (bright, vivid green)
        // Center: "TrustOS" text that lights up on matrix contact
        // ═══════════════════════════════════════════════════════════════
        const MATRIX_COLS: usize = 160;
        const TRAIL_LEN: usize = 30;
        const CHAR_H: u32 = 16;
        
        let height = self.height.saturating_sub(TASKBAR_HEIGHT);
        let width = self.width;
        
        // Clear background to pure black
        framebuffer::fill_rect(0, 0, width, height, 0xFF000000);
        
        if !self.matrix_initialized {
            return;
        }
        
        // ── Build "TrustOS" text bitmap in center ──
        // Large block letters, each char ~20px wide, ~24px tall
        // "TRUSTOS" = 7 chars → ~140px wide at center
        let text_str = "TrustOS";
        let text_px_w = text_str.len() as u32 * 16; // 16px per char (scaled 2x from 8px glyphs)
        let text_px_h = 32u32; // 2x scale of 16px glyph
        let text_x0 = (width / 2).saturating_sub(text_px_w / 2);
        let text_y0 = (height / 2).saturating_sub(text_px_h / 2);
        
        // Build text mask: 1 = pixel belongs to "TrustOS" text
        // Use 2x scaled glyphs
        let mut text_mask = [[false; 224]; 32]; // 7*16*2=224 wide, 32 tall max
        for (ci, ch) in text_str.chars().enumerate() {
            let glyph = crate::framebuffer::font::get_glyph(ch);
            for (row, &bits) in glyph.iter().enumerate() {
                for bit in 0..8u32 {
                    if bits & (0x80 >> bit) != 0 {
                        // Scale 2x
                        let mx = ci * 16 + (bit as usize) * 2;
                        let my = row * 2;
                        if mx < 224 && my < 32 {
                            text_mask[my][mx] = true;
                            if mx + 1 < 224 { text_mask[my][mx + 1] = true; }
                            if my + 1 < 32 {
                                text_mask[my + 1][mx] = true;
                                if mx + 1 < 224 { text_mask[my + 1][mx + 1] = true; }
                            }
                        }
                    }
                }
            }
        }
        
        // ── Render matrix rain ──
        let col_width = width / MATRIX_COLS as u32;
        
        for col in 0..MATRIX_COLS.min(self.matrix_heads.len()) {
            let speed = self.matrix_speeds[col];
            let seed = self.matrix_seeds[col];
            let x = (col as u32 * col_width) + col_width / 2;
            
            // Update position (slower: speed is now 1-3)
            let new_y = self.matrix_heads[col] + speed as i32;
            if new_y > height as i32 + (TRAIL_LEN as i32 * CHAR_H as i32) {
                let new_seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                self.matrix_seeds[col] = new_seed;
                self.matrix_heads[col] = -((new_seed % (height / 2)) as i32);
                let chars: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
                for i in 0..TRAIL_LEN {
                    let cs = new_seed.wrapping_add((i as u32).wrapping_mul(7919));
                    self.matrix_chars[col * TRAIL_LEN + i] = chars[(cs as usize) % chars.len()];
                }
            } else {
                self.matrix_heads[col] = new_y;
            }
            
            let head_y = self.matrix_heads[col];
            
            // Depth from speed: 1=far(dim), 3=near(bright)
            let depth_factor = (speed as f32 - 1.0) / 2.0; // 0.0=far, 1.0=near
            let brightness_mult = 0.3 + depth_factor * 0.7; // 30%→100%
            let saturation = 0.2 + depth_factor * 0.8; // 20%→100%
            
            for i in 0..TRAIL_LEN {
                let char_y = head_y - (i as i32 * CHAR_H as i32);
                if char_y < 0 || char_y >= height as i32 { continue; }
                
                // Trail fading
                let base = if i == 0 { 255u8 }
                    else if i == 1 { 200u8 }
                    else { 160u8.saturating_sub((i as u8).saturating_mul(7)) };
                if base < 15 { continue; }
                
                let brightness = ((base as f32) * brightness_mult) as u8;
                
                // Check if this rain drop touches the "TrustOS" text zone
                let in_text_zone = x >= text_x0 && x < text_x0 + text_px_w
                    && (char_y as u32) >= text_y0 && (char_y as u32) < text_y0 + text_px_h;
                
                let mut text_hit = false;
                if in_text_zone {
                    let tx = (x - text_x0) as usize;
                    let ty = (char_y as u32 - text_y0) as usize;
                    if ty < 32 && tx < 224 && text_mask[ty][tx] {
                        text_hit = true;
                    }
                }
                
                // Color computation
                let (r, g, b) = if text_hit {
                    // Text pixel: bright white-green glow when rain touches
                    let glow = brightness.max(180);
                    let white = (glow as u32 * 3 / 4).min(220) as u8;
                    (white, glow, white)
                } else if i == 0 {
                    // Head: white-ish glow
                    let w = (140.0 * brightness_mult) as u8;
                    (w, brightness.max(w), w)
                } else {
                    // Trail: green with depth-based atmospheric tint
                    let gray_tint = ((15.0 * (1.0 - saturation)) as u8).min(40);
                    let blue_tint = ((30.0 * (1.0 - saturation)) as u8).min(50);
                    (gray_tint, brightness, blue_tint)
                };
                
                let color = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                
                // Character selection with slow mutation
                let char_seed = seed.wrapping_add((i as u32 * 7919) ^ (self.frame_count as u32 / 12));
                let chars: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
                let c = chars[(char_seed as usize) % chars.len()] as char;
                let glyph = crate::framebuffer::font::get_glyph(c);
                
                for (gr, &bits) in glyph.iter().enumerate() {
                    let py = char_y as u32 + gr as u32;
                    if py >= height || bits == 0 { continue; }
                    for bit in 0..8u32 {
                        if bits & (0x80 >> bit) != 0 {
                            let px = x + bit;
                            if px < width {
                                framebuffer::put_pixel(px, py, color);
                            }
                        }
                    }
                }
            }
        }
        
        // ── Draw "TrustOS" outline when NOT being rained on ──
        // Very subtle dark green text that's always visible as ghost
        for (ci, ch) in text_str.chars().enumerate() {
            let glyph = crate::framebuffer::font::get_glyph(ch);
            for (row, &bits) in glyph.iter().enumerate() {
                for bit in 0..8u32 {
                    if bits & (0x80 >> bit) != 0 {
                        for sy in 0..2u32 {
                            for sx in 0..2u32 {
                                let px = text_x0 + (ci as u32) * 16 + bit * 2 + sx;
                                let py = text_y0 + (row as u32) * 2 + sy;
                                if px < width && py < height {
                                    // Only draw ghost if pixel is currently black/very dark
                                    // This creates the "text appears only when rain hits" effect
                                    // with a faint outline always visible
                                    let existing = framebuffer::get_pixel(px, py);
                                    let eg = (existing >> 8) & 0xFF;
                                    if eg < 20 {
                                        framebuffer::put_pixel(px, py, 0xFF001A0A);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
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
                            
                            framebuffer::put_pixel(sx, sy, 0xFF000000 | (r << 16) | (g << 8) | b);
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
                            framebuffer::put_pixel(offset_x + x, offset_y + y, wp_data.pixels[idx]);
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
                                    framebuffer::put_pixel(dx + x, dy + y, wp_data.pixels[idx]);
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
                framebuffer::put_pixel(px, py, fill);
            }
            
            // Shield outline (both edges)
            if w > 2 {
                framebuffer::put_pixel(sx + x_off, sy + y, outline_green);
                framebuffer::put_pixel(sx + x_off + w - 1, sy + y, outline_green);
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
                    framebuffer::put_pixel(lock_cx - 10 + dx, lock_cy - 14 + dy, yellow_green);
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
                    framebuffer::put_pixel(lock_cx - 3 + dx, lock_cy + 4 + dy, black_fill);
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
            framebuffer::put_pixel(lock_cx - 1, ky, circuit_green);
            framebuffer::put_pixel(lock_cx, ky, circuit_green);
            framebuffer::put_pixel(lock_cx + 1, ky, circuit_green);
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
                    framebuffer::put_pixel(px, by, circuit_green);
                    framebuffer::put_pixel(px, by + 1, circuit_green);
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
                            framebuffer::put_pixel(px, py, node_color);
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
                        framebuffer::put_pixel(lock_cx - 4 + dx, key_end_y + dy, node_color);
                    }
                }
            }
        }
        
        // ── "TrustOS" text below circuit key ──
        let text_y_pos = (key_end_y + 16) as i32;
        if (text_y_pos + 16) < self.height.saturating_sub(TASKBAR_HEIGHT) as i32 {
            // "Trust" in gray
            let trust_x = (center_x as i32) - 28;
            self.draw_text(trust_x, text_y_pos, "Trust", text_gray);
            // "OS" in green, right after
            let os_x = trust_x + 40;
            self.draw_text(os_x, text_y_pos, "OS", text_green);
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
                let existing = framebuffer::get_pixel(dx, dy);
                let er = ((existing >> 16) & 0xFF) as u32;
                let eg = ((existing >> 8) & 0xFF) as u32;
                let eb = (existing & 0xFF) as u32;
                // 75% opacity dark overlay: blend toward 0x040804
                let nr = (er * 25 / 100 + 4 * 75 / 100).min(255);
                let ng = (eg * 25 / 100 + 8 * 75 / 100).min(255);
                let nb = (eb * 25 / 100 + 4 * 75 / 100).min(255);
                framebuffer::put_pixel(dx, dy, 0xFF000000 | (nr << 16) | (ng << 8) | nb);
            }
        }
        // Right edge: subtle dark green separator
        framebuffer::fill_rect(DOCK_WIDTH + 9, 0, 1, dock_h, 0xFF002210);
        
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
                                let existing = framebuffer::get_pixel(px, py);
                                let eg = ((existing >> 8) & 0xFF) as u8;
                                let new_g = eg.saturating_add(intensity);
                                let blended = (existing & 0xFFFF00FF) | ((new_g as u32) << 8);
                                framebuffer::put_pixel(px, py, blended);
                            }
                        }
                    }
                }
                // Inner highlight
                framebuffer::fill_rect(ix - 3, iy - 2, icon_size + 6, icon_size + 16, 0xFF001A0A);
                framebuffer::draw_rect(ix - 3, iy - 2, icon_size + 6, icon_size + 16, 0xFF00AA44);
            }
            
            // Icon square background — very dark
            framebuffer::fill_rect(ix, iy, icon_size, icon_size, 0xFF050805);
            framebuffer::draw_rect(ix, iy, icon_size, icon_size, if is_hovered { 0xFF00CC55 } else { 0xFF002A15 });
            
            // Pixel-art icon inside square
            let cx = ix + icon_size / 2;
            let cy = iy + icon_size / 2;
            use crate::icons::IconType;
            match icon.icon_type {
                IconType::Terminal => {
                    // Terminal: rect with >_ prompt
                    framebuffer::draw_rect(cx - 14, cy - 10, 28, 20, icon_color);
                    self.draw_text((cx - 8) as i32, (cy - 4) as i32, ">", icon_color);
                    framebuffer::fill_rect(cx - 2, cy - 2, 10, 2, icon_color);
                },
                IconType::Folder => {
                    // Files: folder shape
                    framebuffer::fill_rect(cx - 12, cy - 2, 24, 14, icon_color);
                    framebuffer::fill_rect(cx - 14, cy - 6, 10, 6, icon_color);
                },
                IconType::Editor => {
                    // Editor: document with text lines
                    framebuffer::fill_rect(cx - 10, cy - 12, 20, 24, icon_color);
                    framebuffer::fill_rect(cx - 8, cy - 10, 16, 20, 0xFF0A0A0A);
                    framebuffer::fill_rect(cx - 6, cy - 6, 12, 2, icon_color);
                    framebuffer::fill_rect(cx - 6, cy - 2, 12, 2, icon_color);
                    framebuffer::fill_rect(cx - 6, cy + 2, 8, 2, icon_color);
                },
                IconType::Calculator => {
                    // Calculator: grid squares
                    framebuffer::draw_rect(cx - 10, cy - 10, 20, 20, icon_color);
                    framebuffer::fill_rect(cx - 8, cy - 8, 6, 4, icon_color);
                    framebuffer::fill_rect(cx + 2, cy - 8, 6, 4, icon_color);
                    framebuffer::fill_rect(cx - 8, cy + 2, 6, 4, icon_color);
                    framebuffer::fill_rect(cx + 2, cy + 2, 6, 4, icon_color);
                },
                IconType::Network => {
                    // Network: signal bars
                    framebuffer::fill_rect(cx - 10, cy + 4, 4, 8, icon_color);
                    framebuffer::fill_rect(cx - 4, cy - 0, 4, 12, icon_color);
                    framebuffer::fill_rect(cx + 2, cy - 4, 4, 16, icon_color);
                    framebuffer::fill_rect(cx + 8, cy - 8, 4, 20, icon_color);
                },
                IconType::Game => {
                    // Game: play triangle
                    for dy in 0..16u32 {
                        let w = (dy.min(16 - dy) + 1) as u32;
                        framebuffer::fill_rect(cx - 6, cy - 8 + dy, w, 1, icon_color);
                    }
                },
                IconType::Settings => {
                    // Settings: gear circle
                    for dy in 0..12u32 {
                        for dx in 0..12u32 {
                            let ddx = dx as i32 - 6;
                            let ddy = dy as i32 - 6;
                            if ddx * ddx + ddy * ddy <= 36 && ddx * ddx + ddy * ddy >= 16 {
                                framebuffer::put_pixel(cx - 6 + dx, cy - 6 + dy, icon_color);
                            }
                        }
                    }
                },
                IconType::Browser => {
                    // Browser: globe
                    for dy in 0..20u32 {
                        for dx in 0..20u32 {
                            let ddx = dx as i32 - 10;
                            let ddy = dy as i32 - 10;
                            if ddx * ddx + ddy * ddy <= 100 && ddx * ddx + ddy * ddy >= 64 {
                                framebuffer::put_pixel(cx - 10 + dx, cy - 10 + dy, icon_color);
                            }
                        }
                    }
                    framebuffer::fill_rect(cx - 10, cy - 1, 20, 2, icon_color);
                    framebuffer::fill_rect(cx - 1, cy - 10, 2, 20, icon_color);
                },
                _ => {
                    // Generic: bordered square
                    framebuffer::draw_rect(cx - 10, cy - 10, 20, 20, icon_color);
                    framebuffer::fill_rect(cx - 4, cy - 4, 8, 8, icon_color);
                },
            }
            
            // Label under icon
            let name = &icon.name;
            let text_w = name.len() as u32 * 8;
            let text_x = ix + (icon_size / 2).saturating_sub(text_w / 2);
            self.draw_text(text_x as i32, (iy + icon_size + 2) as i32, name, label_color);
        }
    }
    
    fn draw_taskbar(&mut self) {
        let y = self.height - TASKBAR_HEIGHT;
        
        // ═══════════════════════════════════════════════════════════════
        // REFINED TRANSLUCENT TASKBAR — Frosted glass with smooth text
        // ═══════════════════════════════════════════════════════════════
        
        // Translucent dark glass background (opaque base + subtle green tint)
        // Use opaque fill first, then translucent green overlay for glass effect
        framebuffer::fill_rect(0, y, self.width, TASKBAR_HEIGHT, 0xFF080C0A);
        framebuffer::fill_rect_alpha(0, y, self.width, TASKBAR_HEIGHT, 0x00AA44, 15);
        
        // Top border: soft gradient line (2px, fading green)
        framebuffer::fill_rect(0, y, self.width, 1, 0xFF0D3D1A);
        framebuffer::fill_rect_alpha(0, y + 1, self.width, 1, 0x00AA44, 25);
        
        // ── TrustOS button (left) ──
        let start_hover = self.cursor_x >= 4 && self.cursor_x < 112 && self.cursor_y >= y as i32;
        if start_hover || self.start_menu_open {
            framebuffer::fill_rect_alpha(4, y + 5, 104, 30, 0x00CC66, 50);
        }
        // Rounded-feel border (subtle glow)
        let border_color = if start_hover || self.start_menu_open { GREEN_PRIMARY } else { GREEN_GHOST };
        framebuffer::draw_rect(4, y + 5, 104, 30, border_color);
        let txt_color = if start_hover || self.start_menu_open { GREEN_PRIMARY } else { GREEN_SECONDARY };
        self.draw_text_smooth(16, (y + 11) as i32, "TrustOS", txt_color);
        
        // ── Window buttons (centered) ──
        let total_btns = self.windows.len();
        let btn_w = 84u32;
        let btn_gap = 6u32;
        let total_w = if total_btns > 0 { total_btns as u32 * (btn_w + btn_gap) - btn_gap } else { 0 };
        let start_x = (self.width.saturating_sub(total_w)) / 2;
        
        for (i, w) in self.windows.iter().enumerate() {
            let btn_x = start_x + i as u32 * (btn_w + btn_gap);
            let btn_y = y + 5;
            
            let is_hover = self.cursor_x >= btn_x as i32 && self.cursor_x < (btn_x + btn_w) as i32
                && self.cursor_y >= y as i32;
            
            // Button background — translucent glass
            if w.focused {
                framebuffer::fill_rect_alpha(btn_x, btn_y, btn_w, 30, 0x00AA44, 70);
            } else if is_hover {
                framebuffer::fill_rect_alpha(btn_x, btn_y, btn_w, 30, 0x008833, 45);
            }
            // Border
            let bdr = if w.focused { GREEN_PRIMARY } else if is_hover { GREEN_MUTED } else { GREEN_GHOST };
            framebuffer::draw_rect(btn_x, btn_y, btn_w, 30, bdr);
            
            // Window title (truncated, anti-aliased)
            let title_max = 8;
            let title: String = w.title.chars().take(title_max).collect();
            let text_color = if w.focused { GREEN_PRIMARY } else { GREEN_TERTIARY };
            self.draw_text_smooth((btn_x + 8) as i32, (btn_y + 9) as i32, &title, text_color);
            
            // Active indicator (green glow line at bottom)
            if w.focused {
                let indicator_w = 60u32.min(btn_w - 10);
                let indicator_x = btn_x + (btn_w - indicator_w) / 2;
                framebuffer::fill_rect(indicator_x, y + TASKBAR_HEIGHT - 4, indicator_w, 3, GREEN_PRIMARY);
                // Soft glow under indicator
                framebuffer::fill_rect_alpha(indicator_x.saturating_sub(2), y + TASKBAR_HEIGHT - 6, indicator_w + 4, 2, GREEN_PRIMARY, 60);
            } else if !w.minimized {
                // Small green dot for open windows
                let dot_x = btn_x + btn_w / 2 - 2;
                framebuffer::fill_rect(dot_x, y + TASKBAR_HEIGHT - 3, 4, 2, GREEN_SUBTLE);
            }
        }
        
        // ── System tray (right side) ──
        // FPS counter (dimmed)
        let fps_str = format!("{}fps", (60u64).min(self.frame_count.max(1)));
        self.draw_text_smooth((self.width - 200) as i32, (y + 14) as i32, &fps_str, GREEN_GHOST);
        
        // Clock (anti-aliased, bright)
        let time = self.get_time_string();
        self.draw_text_smooth((self.width - 130) as i32, (y + 14) as i32, &time, GREEN_PRIMARY);

        // Status circles with soft glow
        let cx = self.width - 30;
        let cy = y + TASKBAR_HEIGHT / 2;
        // Green circle + glow
        framebuffer::fill_rect_alpha(cx.saturating_sub(16), cy.saturating_sub(6), 12, 12, GREEN_PRIMARY, 30);
        for dy in 0..6u32 {
            for dx in 0..6u32 {
                if (dx as i32 - 3) * (dx as i32 - 3) + (dy as i32 - 3) * (dy as i32 - 3) <= 9 {
                    framebuffer::put_pixel(cx - 12 + dx, cy - 3 + dy, GREEN_PRIMARY);
                }
            }
        }
        // Amber circle + glow
        framebuffer::fill_rect_alpha(cx.saturating_sub(4), cy.saturating_sub(6), 12, 12, ACCENT_AMBER, 30);
        for dy in 0..6u32 {
            for dx in 0..6u32 {
                if (dx as i32 - 3) * (dx as i32 - 3) + (dy as i32 - 3) * (dy as i32 - 3) <= 9 {
                    framebuffer::put_pixel(cx + dx, cy - 3 + dy, ACCENT_AMBER);
                }
            }
        }
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
        let menu_w = 280u32;
        let menu_h = 472u32;
        let menu_x = 4i32;
        let menu_y = (self.height - TASKBAR_HEIGHT - menu_h - 8) as i32;
        
        // ═══════════════════════════════════════════════════════════════
        // MATRIX HACKER STYLE START MENU — Frosted glass popup
        // ═══════════════════════════════════════════════════════════════
        
        // Frosted dark glass background
        framebuffer::fill_rect_alpha(menu_x as u32, menu_y as u32, menu_w, menu_h, 0x060A08, 210);
        
        // Double green border
        framebuffer::draw_rect(menu_x as u32, menu_y as u32, menu_w, menu_h, GREEN_PRIMARY);
        framebuffer::draw_rect((menu_x + 1) as u32, (menu_y + 1) as u32, menu_w - 2, menu_h - 2, GREEN_SUBTLE);
        
        // Title bar: translucent dark green
        framebuffer::fill_rect_alpha((menu_x + 2) as u32, (menu_y + 2) as u32, menu_w - 4, 24, 0x002200, 180);
        self.draw_text_smooth(menu_x + 10, menu_y + 6, "TrustOS Menu", GREEN_PRIMARY);
        
        // Separator
        framebuffer::draw_hline((menu_x + 2) as u32, (menu_y + 26) as u32, menu_w - 4, GREEN_MUTED);
        
        // Menu items
        let items: [(&str, &str, bool); 13] = [
            (">_", "Terminal", false),
            ("[]", "Files", false),
            ("##", "Calculator", false),
            ("~~", "Network", false),
            ("Tx", "Text Editor", false),
            ("/\\", "TrustEdit 3D", false),
            ("WW", "Browser", false),
            ("Sk", "Snake", false),
            ("3D", "TrustDoom 3D", false),
            ("@)", "Settings", false),
            ("<-", "Exit Desktop", true),
            ("!!", "Shutdown", true),
            (">>", "Reboot", true),
        ];
        
        for (i, (icon, label, is_special)) in items.iter().enumerate() {
            let item_y = menu_y + 30 + (i as i32 * 32);
            let item_h = 30u32;
            
            // Hover detection
            let is_hovered = self.cursor_x >= menu_x 
                && self.cursor_x < menu_x + menu_w as i32
                && self.cursor_y >= item_y 
                && self.cursor_y < item_y + item_h as i32;
            
            if is_hovered {
                // Translucent green highlight background
                framebuffer::fill_rect_alpha((menu_x + 3) as u32, item_y as u32, menu_w - 6, item_h, 0x00AA44, 50);
                // Left accent bar
                framebuffer::fill_rect((menu_x + 3) as u32, (item_y + 4) as u32, 2, item_h - 8, 
                    if *is_special { ACCENT_RED } else { GREEN_PRIMARY });
            }
            
            // Icon
            let icon_color = if is_hovered {
                if *is_special { ACCENT_RED } else { GREEN_PRIMARY }
            } else {
                if *is_special { 0xFF994444 } else { GREEN_TERTIARY }
            };
            self.draw_text_smooth(menu_x + 14, item_y + 8, icon, icon_color);
            
            // Label
            let label_color = if is_hovered {
                if *is_special { ACCENT_RED } else { GREEN_SECONDARY }
            } else {
                if *is_special { 0xFFAA4444 } else { GREEN_SECONDARY }
            };
            self.draw_text_smooth(menu_x + 40, item_y + 8, label, label_color);
        }
        
        // Bottom: version info
        let ver_y = menu_y + menu_h as i32 - 20;
        framebuffer::draw_hline((menu_x + 4) as u32, ver_y as u32, menu_w - 8, GREEN_GHOST);
        self.draw_text(menu_x + 10, ver_y + 6, "TrustOS v0.1.0", GREEN_SUBTLE);
    }
    
    fn draw_window(&self, window: &Window) {
        let x = window.x;
        let y = window.y;
        let w = window.width;
        let h = window.height;
        
        // ═══════════════════════════════════════════════════════════════
        // MATRIX HACKER STYLE WINDOW — Green borders, dark background
        // ═══════════════════════════════════════════════════════════════
        
        // Window background: near-black
        framebuffer::fill_rect(x as u32, y as u32, w, h, 0xFF0A0A0A);
        
        // Double green border
        let border_color = if window.focused { GREEN_PRIMARY } else { GREEN_SUBTLE };
        framebuffer::draw_rect(x as u32, y as u32, w, h, border_color);
        if w > 4 && h > 4 {
            framebuffer::draw_rect((x + 1) as u32, (y + 1) as u32, w - 2, h - 2, 
                if window.focused { GREEN_MUTED } else { GREEN_GHOST });
        }
        
        // Title bar: frosted glass effect
        let titlebar_h = TITLE_BAR_HEIGHT;
        if window.focused {
            framebuffer::fill_rect_alpha((x + 2) as u32, (y + 2) as u32, w - 4, titlebar_h - 2, 0x0A1A0A, 220);
        } else {
            framebuffer::fill_rect_alpha((x + 2) as u32, (y + 2) as u32, w - 4, titlebar_h - 2, 0x080C08, 200);
        }
        
        // Title bar bottom separator
        framebuffer::draw_hline((x + 2) as u32, (y + titlebar_h as i32) as u32, w - 4, 
            if window.focused { GREEN_MUTED } else { GREEN_GHOST });
        
        // Window icon (2-char representation)
        let icon_str = match window.window_type {
            WindowType::Terminal => ">_",
            WindowType::FileManager => "[]",
            WindowType::Calculator => "##",
            WindowType::Browser => "WW",
            WindowType::ModelEditor => "/\\",
            WindowType::TextEditor => "Tx",
            WindowType::Game => "Sk",
            _ => "::",
        };
        let icon_color = if window.focused { GREEN_PRIMARY } else { GREEN_TERTIARY };
        self.draw_text_smooth(x + 10, y + 7, icon_str, icon_color);
        
        // Title text
        let text_color = if window.focused { TEXT_PRIMARY } else { TEXT_SECONDARY };
        self.draw_text_smooth(x + 32, y + 7, &window.title, text_color);
        
        // ═══════════════════════════════════════════════════════════════
        // Control Buttons (macOS-style colored circles)
        // ═══════════════════════════════════════════════════════════════
        let btn_r = 5u32;
        let btn_y_center = y + titlebar_h as i32 / 2;
        let mx = self.cursor_x;
        let my = self.cursor_y;
        
        // Close button (red circle)
        let close_cx = x + w as i32 - 20;
        let close_hover = (mx - close_cx).abs() <= btn_r as i32 + 2 && (my - btn_y_center).abs() <= btn_r as i32 + 2;
        let close_color = if close_hover { 0xFFFF4444 } else { BTN_CLOSE };
        for dy in 0..btn_r * 2 + 1 {
            for dx in 0..btn_r * 2 + 1 {
                let ddx = dx as i32 - btn_r as i32;
                let ddy = dy as i32 - btn_r as i32;
                if ddx * ddx + ddy * ddy <= (btn_r * btn_r) as i32 {
                    framebuffer::put_pixel((close_cx + ddx) as u32, (btn_y_center + ddy) as u32, close_color);
                }
            }
        }
        if close_hover {
            // X inside
            for i in -2..=2i32 {
                framebuffer::put_pixel((close_cx + i) as u32, (btn_y_center + i) as u32, 0xFFFFFFFF);
                framebuffer::put_pixel((close_cx + i) as u32, (btn_y_center - i) as u32, 0xFFFFFFFF);
            }
        }
        
        // Minimize button (yellow circle)
        let min_cx = close_cx - 18;
        let min_hover = (mx - min_cx).abs() <= btn_r as i32 + 2 && (my - btn_y_center).abs() <= btn_r as i32 + 2;
        let min_color = if min_hover { 0xFFFFCC00 } else { BTN_MAXIMIZE };
        for dy in 0..btn_r * 2 + 1 {
            for dx in 0..btn_r * 2 + 1 {
                let ddx = dx as i32 - btn_r as i32;
                let ddy = dy as i32 - btn_r as i32;
                if ddx * ddx + ddy * ddy <= (btn_r * btn_r) as i32 {
                    framebuffer::put_pixel((min_cx + ddx) as u32, (btn_y_center + ddy) as u32, min_color);
                }
            }
        }
        if min_hover {
            framebuffer::fill_rect((min_cx - 3) as u32, btn_y_center as u32, 6, 1, 0xFF000000);
        }
        
        // Maximize button (green circle)
        let max_cx = min_cx - 18;
        let max_hover = (mx - max_cx).abs() <= btn_r as i32 + 2 && (my - btn_y_center).abs() <= btn_r as i32 + 2;
        let max_color = if max_hover { 0xFF44DD44 } else { BTN_MINIMIZE };
        for dy in 0..btn_r * 2 + 1 {
            for dx in 0..btn_r * 2 + 1 {
                let ddx = dx as i32 - btn_r as i32;
                let ddy = dy as i32 - btn_r as i32;
                if ddx * ddx + ddy * ddy <= (btn_r * btn_r) as i32 {
                    framebuffer::put_pixel((max_cx + ddx) as u32, (btn_y_center + ddy) as u32, max_color);
                }
            }
        }
        if max_hover {
            framebuffer::draw_rect((max_cx - 2) as u32, (btn_y_center - 2) as u32, 5, 5, 0xFF000000);
        }
        
        // ═══════════════════════════════════════════════════════════════
        // Content Area
        // ═══════════════════════════════════════════════════════════════
        let content_y = y + titlebar_h as i32;
        let content_h = h - titlebar_h;
        framebuffer::fill_rect((x + 2) as u32, (content_y + 1) as u32, w - 4, content_h.saturating_sub(3), 0xFF080808);
        
        // Draw window content
        self.draw_window_content(window);
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
        
        // Calculator is handled separately
        if window.window_type == WindowType::Calculator {
            self.draw_calculator(window);
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
        
        // Special rendering for Browser
        if window.window_type == WindowType::Browser {
            self.draw_browser(window);
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
        
        for (i, line) in window.content.iter().enumerate() {
            let line_y = content_y + (i as i32 * 16);
            if line_y >= window.y + window.height as i32 - 8 {
                break;
            }
            
            // Check if this line is selected
            let is_selected = needs_selection 
                && i >= sel_start 
                && i < sel_end 
                && (i - sel_start) == window.selected_index;
            
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
                        framebuffer::put_pixel(sx, sy, color);
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
                        framebuffer::put_pixel(sx, sy, color);
                    }
                }
            }
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
                framebuffer::put_pixel(x as u32, y as u32, color);
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
            framebuffer::put_pixel(game_x + i, game_y, GREEN_MUTED);
            framebuffer::put_pixel(game_x + i, game_y + game_h - 1, GREEN_MUTED);
        }
        for i in 0..game_h {
            framebuffer::put_pixel(game_x, game_y + i, GREEN_MUTED);
            framebuffer::put_pixel(game_x + game_w - 1, game_y + i, GREEN_MUTED);
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
                        framebuffer::put_pixel(px + ex1, py + ey1, 0xFF000000);
                        framebuffer::put_pixel(px + ex2, py + ey2, 0xFF000000);
                    }
                }
            }
            
            // Draw food
            let fx = grid_offset_x + snake.food.0 as u32 * cell_size;
            let fy = grid_offset_y + snake.food.1 as u32 * cell_size;
            if fx + cell_size < game_x + game_w && fy + cell_size < game_y + game_h {
                framebuffer::fill_rect(fx + 2, fy + 2, cell_size - 4, cell_size - 4, 0xFFFF4444);
                framebuffer::put_pixel(fx + cell_size/2, fy + 1, 0xFF00AA00); // stem
            }
            
            // Title
            self.draw_text(game_x as i32 + 8, game_y as i32 + 8, "SNAKE", COLOR_BRIGHT_GREEN);
            
            // Score
            let score_str = format!("Score: {}", snake.score);
            self.draw_text(game_x as i32 + game_w as i32 - 90, game_y as i32 + 8, &score_str, GREEN_SECONDARY);
            
            if snake.game_over {
                // Game over overlay
                let ox = game_x + game_w / 2 - 60;
                let oy = game_y + game_h / 2 - 20;
                framebuffer::fill_rect(ox - 4, oy - 4, 128, 48, 0xCC000000);
                self.draw_text(ox as i32, oy as i32, "GAME OVER", 0xFFFF4444);
                self.draw_text(ox as i32 - 8, oy as i32 + 20, "Press ENTER", GREEN_TERTIARY);
            } else {
                // Instructions
                self.draw_text(game_x as i32 + 8, game_y as i32 + game_h as i32 - 18, 
                               "Arrow Keys to move", GREEN_TERTIARY);
            }
        }
    }
    
    /// Draw interactive Calculator
    fn draw_calculator(&self, window: &Window) {
        use crate::gui::windows11::colors;
        
        let cx = window.x as u32 + 4;
        let cy = window.y as u32 + TITLE_BAR_HEIGHT + 4;
        let cw = window.width.saturating_sub(8);
        let ch = window.height.saturating_sub(TITLE_BAR_HEIGHT + 8);
        
        if cw < 100 || ch < 120 {
            return;
        }
        
        // Display area
        let display_h = 56u32;
        framebuffer::fill_rect(cx + 4, cy + 4, cw - 8, display_h, 0xFF1A1A2E);
        framebuffer::fill_rect(cx + 5, cy + 5, cw - 10, display_h - 2, 0xFF0D0D1A);
        
        // Get calculator state
        let display_text = if let Some(calc) = self.calculator_states.get(&window.id) {
            &calc.display
        } else {
            "0"
        };
        
        // Draw display text (right-aligned, large)
        let text_len = display_text.len() as i32;
        let text_x = cx as i32 + cw as i32 - 16 - text_len * 10;
        // Draw each char slightly larger (using 2 copies offset)
        for (i, ch) in display_text.chars().enumerate() {
            let px = text_x + i as i32 * 10;
            let py = cy as i32 + 20;
            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            self.draw_text(px, py, s, 0xFFFFFFFF);
            self.draw_text(px + 1, py, s, 0xFFFFFFFF); // Bold effect
        }
        
        // Operator indicator
        if let Some(calc) = self.calculator_states.get(&window.id) {
            if let Some(op) = calc.operator {
                let mut buf = [0u8; 4];
                let s = op.encode_utf8(&mut buf);
                self.draw_text(cx as i32 + 10, cy as i32 + 12, s, colors::ACCENT);
            }
        }
        
        // Button grid
        let btn_area_y = cy + display_h + 12;
        let btn_rows = 5;
        let btn_cols = 4;
        let btn_gap = 4u32;
        let btn_w = (cw - 12 - btn_gap * (btn_cols - 1)) / btn_cols;
        let btn_h = (ch - display_h - 20 - btn_gap * (btn_rows - 1)) / btn_rows;
        let btn_h = btn_h.min(40);
        
        let buttons = [
            ["C", "(", ")", "%"],
            ["7", "8", "9", "/"],
            ["4", "5", "6", "*"],
            ["1", "2", "3", "-"],
            ["0", ".", "=", "+"],
        ];
        
        for (row, btn_row) in buttons.iter().enumerate() {
            for (col, label) in btn_row.iter().enumerate() {
                let bx = cx + 4 + col as u32 * (btn_w + btn_gap);
                let by = btn_area_y + row as u32 * (btn_h + btn_gap);
                
                // Button color based on type
                let is_operator = matches!(*label, "+" | "-" | "*" | "/" | "%" | "=");
                let is_clear = *label == "C" || *label == "(" || *label == ")";
                
                let btn_bg = if is_operator {
                    if *label == "=" { colors::ACCENT } else { 0xFF2A3A30 }
                } else if is_clear {
                    0xFF2A2030
                } else {
                    0xFF1E2228
                };
                
                // Hover detection
                let hover = self.cursor_x >= bx as i32 && self.cursor_x < (bx + btn_w) as i32
                    && self.cursor_y >= by as i32 && self.cursor_y < (by + btn_h) as i32;
                
                let bg = if hover { 
                    (btn_bg & 0x00FFFFFF) | 0xFF000000 // Brighten slightly
                } else { 
                    btn_bg 
                };
                
                // Draw button
                crate::gui::windows11::draw_rounded_rect(bx as i32, by as i32, btn_w, btn_h, 4, bg);
                if hover {
                    crate::gui::windows11::draw_rounded_rect_border(bx as i32, by as i32, btn_w, btn_h, 4, colors::ACCENT);
                }
                
                // Label centered
                let lw = label.len() as u32 * 8;
                let lx = bx + (btn_w - lw) / 2;
                let ly = by + (btn_h / 2) - 4;
                let text_color = if *label == "=" { 0xFF000000 } else { colors::TEXT_PRIMARY };
                self.draw_text(lx as i32, ly as i32, label, text_color);
            }
        }
        
        // Instructions at bottom
        self.draw_text(cx as i32 + 4, (cy + ch - 14) as i32, "Keys: 0-9 +*/- = Enter C", GREEN_TERTIARY);
    }
    
    /// Draw Browser window content
    fn draw_browser(&self, window: &Window) {
        let toolbar_height: u32 = 36;
        let browser_x = window.x as u32 + 4;
        let browser_y = window.y as u32 + TITLE_BAR_HEIGHT + 4;
        let browser_w = window.width.saturating_sub(8);
        let browser_h = window.height.saturating_sub(TITLE_BAR_HEIGHT + 8);
        
        if browser_w < 100 || browser_h < 80 {
            return;
        }
        
        // Draw toolbar background
        framebuffer::fill_rect(browser_x, browser_y, browser_w, toolbar_height, 0xFF303030);
        
        // Navigation buttons
        let btn_y = browser_y + 6;
        let btn_size: u32 = 24;
        
        // Back button (◀)
        framebuffer::fill_rect(browser_x + 8, btn_y, btn_size, btn_size, 0xFF404040);
        self.draw_text(browser_x as i32 + 14, btn_y as i32 + 4, "<", 0xFFCCCCCC);
        
        // Forward button (▶)
        framebuffer::fill_rect(browser_x + 8 + btn_size + 4, btn_y, btn_size, btn_size, 0xFF404040);
        self.draw_text(browser_x as i32 + 14 + btn_size as i32 + 4, btn_y as i32 + 4, ">", 0xFFCCCCCC);
        
        // Refresh button (⟳)
        framebuffer::fill_rect(browser_x + 8 + (btn_size + 4) * 2, btn_y, btn_size, btn_size, 0xFF404040);
        self.draw_text(browser_x as i32 + 14 + (btn_size as i32 + 4) * 2, btn_y as i32 + 4, "R", 0xFFCCCCCC);
        
        // Parse/Raw toggle button
        let toggle_btn_x = browser_x + 8 + (btn_size + 4) * 3;
        let toggle_btn_w: u32 = 40;
        let is_raw = self.browser.as_ref().map(|b| b.show_raw_html).unwrap_or(false);
        let toggle_color = if is_raw { 0xFF0066CC } else { 0xFF404040 };
        framebuffer::fill_rect(toggle_btn_x, btn_y, toggle_btn_w, btn_size, toggle_color);
        let toggle_text = if is_raw { "RAW" } else { "HTML" };
        self.draw_text(toggle_btn_x as i32 + 6, btn_y as i32 + 4, toggle_text, 0xFFFFFFFF);
        
        // URL bar (after toggle button)
        let url_bar_x = toggle_btn_x + toggle_btn_w + 8;
        let url_bar_w = browser_w.saturating_sub(url_bar_x - browser_x + 8);
        framebuffer::fill_rect(url_bar_x, btn_y, url_bar_w, btn_size, 0xFF1A1A1A);
        
        // Draw current URL or placeholder
        let url_text = if self.browser_url_input.is_empty() {
            "Enter URL..."
        } else {
            &self.browser_url_input
        };
        let text_color = if self.browser_url_input.is_empty() { 0xFF666666 } else { 0xFFFFFFFF };
        
        // Draw loading indicator
        if self.browser_loading {
            self.draw_text(url_bar_x as i32 + 8, btn_y as i32 + 4, "Loading...", 0xFF00CC66);
        } else {
            self.draw_text(url_bar_x as i32 + 8, btn_y as i32 + 4, url_text, text_color);
        }
        
        // Draw blinking cursor in URL bar
        if !self.browser_loading && !self.browser_url_input.is_empty() || self.browser_url_input.is_empty() {
            let ticks = crate::logger::get_ticks();
            if (ticks / 500) % 2 == 0 { // Blink every 500ms
                let cursor_x = url_bar_x as i32 + 8 + (self.browser_url_cursor as i32) * 7;
                if cursor_x < (url_bar_x + url_bar_w - 4) as i32 {
                    framebuffer::fill_rect(cursor_x as u32, btn_y + 3, 2, btn_size - 6, 0xFF00CC66);
                }
            }
        }
        
        // Content area
        let content_y = browser_y + toolbar_height + 2;
        let content_h = browser_h.saturating_sub(toolbar_height + 4);
        
        // Draw browser content
        if let Some(ref browser) = self.browser {
            if browser.show_raw_html && !browser.raw_html.is_empty() {
                // RAW HTML view - display source code
                framebuffer::fill_rect(browser_x, content_y, browser_w, content_h, 0xFF1A1A1A);
                self.draw_raw_html_view(browser_x as i32, content_y as i32, browser_w, content_h, &browser.raw_html, browser.scroll_y);
            } else if let Some(ref doc) = browser.document {
                // PARSED view - use the browser renderer
                crate::browser::render_html(
                    doc,
                    browser_x as i32,
                    content_y as i32,
                    browser_w,
                    content_h,
                    browser.scroll_y,
                );
            } else {
                // No document loaded - show welcome page
                framebuffer::fill_rect(browser_x, content_y, browser_w, content_h, 0xFFFFFFFF);
                
                // Centered welcome text
                let center_x = browser_x as i32 + browser_w as i32 / 2 - 100;
                let center_y = content_y as i32 + content_h as i32 / 2 - 40;
                
                self.draw_text(center_x, center_y, "TrustBrowser", 0xFF000000);
                self.draw_text(center_x - 20, center_y + 24, "Enter a URL to get started", 0xFF666666);
                self.draw_text(center_x - 10, center_y + 48, "Try: http://example.com", 0xFF0066CC);
            }
        } else {
            // Browser not initialized - show blank page
            framebuffer::fill_rect(browser_x, content_y, browser_w, content_h, 0xFFFFFFFF);
            let center_x = browser_x as i32 + browser_w as i32 / 2 - 80;
            let center_y = content_y as i32 + content_h as i32 / 2;
            self.draw_text(center_x, center_y, "Welcome to TrustBrowser", 0xFF000000);
        }
        
        // Status bar at bottom - show resources info
        let status_y = browser_y + browser_h - 18;
        framebuffer::fill_rect(browser_x, status_y, browser_w, 18, 0xFF2A2A2A);
        
        let status_text = if let Some(ref browser) = self.browser {
            let resources_info = if !browser.pending_resources.is_empty() {
                alloc::format!(" | {} resources pending", browser.pending_resources.len())
            } else if !browser.resources.is_empty() {
                alloc::format!(" | {} resources loaded", browser.resources.len())
            } else {
                alloc::string::String::new()
            };
            match &browser.status {
                crate::browser::BrowserStatus::Idle => alloc::format!("Ready{}", resources_info),
                crate::browser::BrowserStatus::Loading => alloc::format!("Loading...{}", resources_info),
                crate::browser::BrowserStatus::Ready => alloc::format!("Done{}", resources_info),
                crate::browser::BrowserStatus::Error(e) => e.clone(),
            }
        } else {
            alloc::string::String::from("Ready")
        };
        self.draw_text(browser_x as i32 + 8, status_y as i32 + 2, &status_text, 0xFF999999);
        
        // Shortcuts hint on right side of status bar
        let hint = "PgUp/Dn:Scroll  Ctrl+R:Refresh  Tab:http://  Esc:Clear";
        let hint_x = (browser_x + browser_w) as i32 - (hint.len() as i32 * 7) - 8;
        if hint_x > browser_x as i32 + 150 {
            self.draw_text(hint_x, status_y as i32 + 2, hint, 0xFF666666);
        }
    }
    
    /// Draw raw HTML source code view
    fn draw_raw_html_view(&self, x: i32, y: i32, width: u32, height: u32, html: &str, scroll_y: i32) {
        let line_height = 14;
        let char_width = 7;
        let max_chars = (width as usize).saturating_sub(20) / char_width;
        
        let mut draw_y = y + 8 - scroll_y;
        let max_y = y + height as i32 - 8;
        let mut line_num = 1;
        
        for line in html.lines() {
            if draw_y > max_y {
                break;
            }
            
            if draw_y >= y - line_height {
                // Line number (gray)
                let line_str = alloc::format!("{:4} ", line_num);
                self.draw_text(x + 4, draw_y, &line_str, 0xFF666666);
                
                // Truncate long lines
                let display_line: alloc::string::String = if line.len() > max_chars {
                    let truncated: alloc::string::String = line.chars().take(max_chars.saturating_sub(3)).collect();
                    alloc::format!("{}...", truncated)
                } else {
                    alloc::string::String::from(line)
                };
                
                // Syntax highlight: tags in blue, attributes in green, strings in orange
                self.draw_syntax_highlighted(x + 40, draw_y, &display_line);
            }
            
            draw_y += line_height;
            line_num += 1;
        }
    }
    
    /// Draw syntax-highlighted HTML
    fn draw_syntax_highlighted(&self, x: i32, y: i32, line: &str) {
        let mut current_x = x;
        let char_width = 7;
        let mut in_tag = false;
        let mut in_string = false;
        let mut in_attr = false;
        let mut string_char = '"';
        
        let chars: alloc::vec::Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let c = chars[i];
            let color = if in_string {
                0xFFE9967A // Orange for strings
            } else if c == '<' || c == '>' || c == '/' {
                in_tag = c == '<';
                if c == '>' { in_attr = false; }
                0xFF569CD6 // Blue for < > /
            } else if in_tag && c == '=' {
                in_attr = true;
                0xFF9CDCFE // Light blue for =
            } else if in_tag && (c == '"' || c == '\'') {
                in_string = true;
                string_char = c;
                0xFFE9967A // Orange
            } else if in_attr && !c.is_whitespace() {
                0xFF4EC9B0 // Teal for attribute names
            } else if in_tag && !c.is_whitespace() && c != '=' {
                0xFF569CD6 // Blue for tag names
            } else {
                0xFFD4D4D4 // Light gray for text
            };
            
            // Check for end of string
            if in_string && i > 0 && c == string_char && chars[i-1] != '\\' {
                in_string = false;
            }
            
            // Draw character
            let s = alloc::format!("{}", c);
            self.draw_text(current_x, y, &s, color);
            current_x += char_width as i32;
            i += 1;
        }
    }
    
    /// Handle mouse click inside browser window
    fn handle_browser_click(&mut self, x: i32, y: i32, window: &Window) {
        let toolbar_height: u32 = 36;
        let browser_x = window.x as u32 + 4;
        let browser_y = window.y as u32 + TITLE_BAR_HEIGHT + 4;
        let browser_w = window.width.saturating_sub(8);
        
        if browser_w < 100 {
            return;
        }
        
        let btn_y = browser_y + 6;
        let btn_size: u32 = 24;
        
        let click_x = x as u32;
        let click_y = y as u32;
        
        // Check if click is in toolbar area
        if click_y >= btn_y && click_y < btn_y + btn_size {
            // Back button
            let back_x = browser_x + 8;
            if click_x >= back_x && click_x < back_x + btn_size {
                crate::serial_println!("[BROWSER] Back button clicked");
                if let Some(ref mut browser) = self.browser {
                    let _ = browser.back();
                }
                return;
            }
            
            // Forward button
            let fwd_x = browser_x + 8 + btn_size + 4;
            if click_x >= fwd_x && click_x < fwd_x + btn_size {
                crate::serial_println!("[BROWSER] Forward button clicked");
                if let Some(ref mut browser) = self.browser {
                    let _ = browser.forward();
                }
                return;
            }
            
            // Refresh button
            let refresh_x = browser_x + 8 + (btn_size + 4) * 2;
            if click_x >= refresh_x && click_x < refresh_x + btn_size {
                crate::serial_println!("[BROWSER] Refresh button clicked");
                if let Some(ref mut browser) = self.browser {
                    let _ = browser.refresh();
                }
                return;
            }
            
            // Parse/Raw toggle button
            let toggle_btn_x = browser_x + 8 + (btn_size + 4) * 3;
            let toggle_btn_w: u32 = 40;
            if click_x >= toggle_btn_x && click_x < toggle_btn_x + toggle_btn_w {
                crate::serial_println!("[BROWSER] View toggle clicked");
                if let Some(ref mut browser) = self.browser {
                    browser.toggle_view_mode();
                }
                return;
            }
        }
    }
    
    fn draw_cursor(&self) {
        // OPTIMIZED: Simple cursor without expensive glow calculations
        // The glow effect was causing ~256 sqrt calculations per frame!
        
        // Simple 3-pixel drop shadow (very fast)
        let shadow_color = 0x40000000u32;
        for offset in 1..=2 {
            let sx = self.cursor_x + offset;
            let sy = self.cursor_y + offset;
            if sx >= 0 && sy >= 0 && sx < self.width as i32 && sy < self.height as i32 {
                for dy in 0..12 {
                    let py = (sy + dy) as u32;
                    let px = sx as u32;
                    if py < self.height && px < self.width {
                        framebuffer::put_pixel(px, py, shadow_color);
                    }
                }
            }
        }
        
        // Modern arrow cursor with green accent
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
                let px = (self.cursor_x + cx as i32) as u32;
                let py = (self.cursor_y + cy as i32) as u32;
                if px < self.width && py < self.height {
                    let color = match pixel {
                        1 => GREEN_MUTED,      // Outline in muted green
                        2 => GREEN_SECONDARY,  // Fill in bright green
                        _ => continue,
                    };
                    framebuffer::put_pixel(px, py, color);
                }
            }
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
        let ch = 16u32 * factor;
        let fw = 8u32 * factor;
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
                if bits == 0 { continue; }
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
                                    framebuffer::put_pixel(px, py, color);
                                }
                            }
                        }
                    } else {
                        // Check neighbors for AA edge
                        let left  = col > 0 && (bits & (mask << 1)) != 0;
                        let right = col < 7 && (bits & (mask >> 1)) != 0;
                        let top   = prev & mask != 0;
                        let bot   = next & mask != 0;
                        let adj = (left as u8) + (right as u8) + (top as u8) + (bot as u8);
                        
                        if adj > 0 {
                            // Edge pixel: blend at 30-60% depending on adjacency
                            let alpha = if adj >= 2 { 150u32 } else { 75u32 };
                            let inv = 255 - alpha;
                            for sy in 0..factor {
                                for sx in 0..factor {
                                    let px = cx as u32 + col * factor + sx;
                                    let py = y as u32 + row * factor + sy;
                                    if px < fb_w && py < fb_h {
                                        let bg = framebuffer::get_pixel(px, py);
                                        let br = (bg >> 16) & 0xFF;
                                        let bg_g = (bg >> 8) & 0xFF;
                                        let bb = bg & 0xFF;
                                        let r = (fg_r * alpha + br * inv) / 255;
                                        let g = (fg_g * alpha + bg_g * inv) / 255;
                                        let b = (fg_b * alpha + bb * inv) / 255;
                                        framebuffer::put_pixel(px, py, 0xFF000000 | (r << 16) | (g << 8) | b);
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
    DESKTOP.lock().create_window("Terminal", x, y, 500, 350, WindowType::Terminal)
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
    
    // Initialize GUI timing
    engine::init_timing();
    
    crate::serial_println!("[GUI] Starting desktop environment...");
    crate::serial_println!("[GUI] Hotkeys: Alt+Tab, Win+Arrows, Alt+F4, Win=Start");
    crate::serial_println!("[GUI] Target: 60 FPS with HLT-based frame limiting");
    
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
        
        // Handle keyboard input
        // NOTE: read_char() returns ASCII values, NOT scancodes.
        // The interrupt handler converts scancodes→ASCII and strips releases.
        // Use keyboard::is_key_pressed(scancode) to check modifier/key state.
        while let Some(key) = crate::keyboard::read_char() {
            // Check modifier state from interrupt handler (tracks raw scancodes)
            let alt = crate::keyboard::is_key_pressed(0x38);
            let _ctrl = crate::keyboard::is_key_pressed(0x1D);
            let win = crate::keyboard::is_key_pressed(0x5B);
            
            // ESC (ASCII 27) → close focused window, or exit desktop if none
            if key == 27 {
                let mut d = DESKTOP.lock();
                let has_focused = d.windows.iter().any(|w| w.focused && !w.minimized);
                if has_focused {
                    d.close_focused_window();
                    crate::serial_println!("[GUI] ESC: closed focused window");
                } else {
                    crate::serial_println!("[GUI] ESC: no window open, exiting desktop");
                    EXIT_DESKTOP_FLAG.store(true, Ordering::SeqCst);
                }
                drop(d);
                continue;
            }
            
            // Alt+Tab or Win+Tab (Tab = ASCII 9) → window switcher
            if (alt || win) && key == 9 {
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
                continue;
            }
            // Win+Right Arrow → snap window to right half
            if win && key == crate::keyboard::KEY_RIGHT {
                DESKTOP.lock().snap_focused_window(SnapDir::Right);
                continue;
            }
            // Win+Up Arrow → maximize focused window
            if win && key == crate::keyboard::KEY_UP {
                DESKTOP.lock().toggle_maximize_focused();
                continue;
            }
            // Win+Down Arrow → minimize focused window
            if win && key == crate::keyboard::KEY_DOWN {
                DESKTOP.lock().minimize_focused_window();
                continue;
            }
            
            // Alt+F4 → close focused window  (F4 key sends no ASCII via read_char,
            // so this is handled in the scancode check below)
            
            // Pass key to focused window
            handle_keyboard(key);
        }
        
        // Handle Alt/Win release to finish Alt+Tab / Win+Tab
        if engine::is_alt_tab_active() {
            // Check if both Alt and Win are released → select the window
            let alt_held = crate::keyboard::is_key_pressed(0x38);
            let win_held = crate::keyboard::is_key_pressed(0x5B);
            if !alt_held && !win_held {
                let selected = engine::finish_alt_tab();
                DESKTOP.lock().focus_window_by_index(selected as usize);
            }
        }
        
        // Win key alone (press & release) → toggle start menu
        {
            static mut LAST_WIN: bool = false;
            static mut WIN_USED_COMBO: bool = false;
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
        let right = mouse.right_button;
        unsafe {
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
        // Frame Limiting with HLT (saves CPU!)
        // ═══════════════════════════════════════════════════════════════
        engine::wait_for_next_frame(frame_start);
    }
    
    // ═══════════════════════════════════════════════════════════════
    // Cleanup before returning to shell
    // ═══════════════════════════════════════════════════════════════
    crate::serial_println!("[GUI] Desktop exiting, cleaning up...");
    crate::framebuffer::set_double_buffer_mode(false);
    crate::framebuffer::clear();
    crate::serial_println!("[GUI] Desktop exited cleanly");
}

/// Snap direction for window snapping
#[derive(Clone, Copy)]
pub enum SnapDir {
    Left,
    Right,
}

/// Render Alt+Tab overlay
fn render_alt_tab_overlay() {
    // Get window list
    let desktop = DESKTOP.lock();
    let windows = desktop.get_window_titles();
    if windows.is_empty() { return; }
    
    let screen_w = desktop.width;
    let screen_h = desktop.height;
    drop(desktop);
    
    let selection = crate::gui::engine::alt_tab_selection();
    let count = windows.len() as i32;
    let idx = ((selection % count) + count) % count; // Wrap around
    
    // Calculate overlay size
    let item_w: u32 = 120;
    let item_h: u32 = 80;
    let padding: u32 = 10;
    let total_w = windows.len() as u32 * (item_w + padding) + padding;
    let total_h = item_h + padding * 2 + 20;
    
    let x = (screen_w as i32 - total_w as i32) / 2;
    let y = (screen_h as i32 - total_h as i32) / 2;
    
    // Draw background
    draw_rounded_rect(x, y, total_w, total_h, 8, 0xE0101520);
    draw_rounded_rect(x + 1, y + 1, total_w - 2, total_h - 2, 7, 0xE0202530);
    
    // Draw window previews
    for (i, title) in windows.iter().enumerate() {
        let ix = x + padding as i32 + i as i32 * (item_w + padding) as i32;
        let iy = y + padding as i32;
        
        // Highlight selected
        let bg = if i as i32 == idx { 0xFF00CC55 } else { 0xFF303540 };
        draw_rounded_rect(ix, iy, item_w, item_h, 4, bg);
        
        // Window title (truncated)
        let short: String = title.chars().take(12).collect();
        draw_text_centered(ix + item_w as i32 / 2, iy + item_h as i32 + 5, &short, 0xFFFFFFFF);
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

/// Render toast notifications
fn render_notifications() {
    use crate::gui::engine::{get_notifications, NotifyPriority};
    
    let desktop = DESKTOP.lock();
    let screen_w = desktop.width;
    drop(desktop);
    
    let notifs = get_notifications();
    let mut y = 60; // Below taskbar
    
    for toast in notifs.iter() {
        let w: u32 = 300;
        let h: u32 = if toast.progress.is_some() { 70 } else { 60 };
        let x = screen_w as i32 - w as i32 - 15;
        
        // Background
        draw_rounded_rect(x, y, w, h, 8, 0xF0181C25);
        
        // Accent bar
        let accent_color = toast.get_color();
        draw_rect(x, y, 4, h, accent_color);
        
        // Title
        draw_text(x + 15, y + 10, &toast.title, accent_color);
        
        // Message
        draw_text(x + 15, y + 30, &toast.message, 0xFFAAAAAA);
        
        // Progress bar if present
        if let Some(percent) = toast.progress {
            let bar_y = y + 50;
            let bar_w = w - 30;
            draw_rounded_rect(x + 15, bar_y, bar_w, 8, 3, 0xFF303540);
            let fill_w = (bar_w as u32 * percent as u32 / 100) as u32;
            if fill_w > 0 {
                draw_rounded_rect(x + 15, bar_y, fill_w.max(6), 8, 3, 0xFF00CC55);
            }
        }
        
        y += h as i32 + 10;
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
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
}

/// Set the render mode (Classic or OpenGL)
pub fn set_render_mode(mode: RenderMode) {
    DESKTOP.lock().set_render_mode(mode);
    let mode_name = match mode {
        RenderMode::Classic => "Classic",
        RenderMode::OpenGL => "OpenGL Compositor",
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
