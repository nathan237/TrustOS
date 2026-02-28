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
        px >= self.x && px < self.x + self.width as i32 - 60 &&
        py >= self.y && py < self.y + TITLE_BAR_HEIGHT as i32
    }
    
    /// Check if point is on close button
    pub fn on_close_button(&self, px: i32, py: i32) -> bool {
        // Windows-style: close is rightmost button (32px wide)
        let btn_w = 32i32;
        let btn_x = self.x + self.width as i32 - btn_w - 2;
        px >= btn_x && px < btn_x + btn_w &&
        py >= self.y + 2 && py < self.y + TITLE_BAR_HEIGHT as i32
    }
    
    /// Check if point is on maximize button
    pub fn on_maximize_button(&self, px: i32, py: i32) -> bool {
        // Windows-style: maximize is middle button
        let btn_w = 32i32;
        let btn_x = self.x + self.width as i32 - btn_w * 2 - 2;
        px >= btn_x && px < btn_x + btn_w &&
        py >= self.y + 2 && py < self.y + TITLE_BAR_HEIGHT as i32
    }
    
    /// Check if point is on minimize button
    pub fn on_minimize_button(&self, px: i32, py: i32) -> bool {
        // Windows-style: minimize is leftmost of the three buttons
        let btn_w = 32i32;
        let btn_x = self.x + self.width as i32 - btn_w * 3 - 2;
        px >= btn_x && px < btn_x + btn_w &&
        py >= self.y + 2 && py < self.y + TITLE_BAR_HEIGHT as i32
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
    /// Image viewer states per window (window_id -> state with pixel data)
    pub image_viewer_states: BTreeMap<u32, ImageViewerState>,
    /// File clipboard for copy/paste in file manager
    pub file_clipboard: Option<FileClipboardEntry>,
    /// Drag-and-drop state
    pub drag_state: Option<DragState>,
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
            browser: None,
            browser_url_input: String::new(),
            browser_url_cursor: 0,
            browser_loading: false,
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
            terminal_suggestion_count: 0,
            command_history: Vec::new(),
            history_index: None,
            saved_input: String::new(),
            start_menu_search: String::new(),
            start_menu_selected: -1,
            clipboard_icon: None,
            // New features
            fm_view_modes: BTreeMap::new(),
            image_viewer_states: BTreeMap::new(),
            file_clipboard: None,
            drag_state: None,
            lock_screen_active: false,
            lock_screen_input: String::new(),
            lock_screen_shake: 0,
            sys_volume: 75,
            sys_battery: 85,
            sys_wifi_connected: true,
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
        self.chess3d_states.clear();
        #[cfg(feature = "emulators")]
        self.nes_states.clear();
        #[cfg(feature = "emulators")]
        self.gameboy_states.clear();
        self.binary_viewer_states.clear();
        self.lab_states.clear();
        // Browser
        self.browser = None;
        self.browser_url_input.clear();
        self.browser_url_cursor = 0;
        self.browser_loading = false;
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
        let mut window = Window::new(title, x, y, width, height, wtype);
        
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
        self.chess_states.remove(&id);
        self.chess3d_states.remove(&id);
        #[cfg(feature = "emulators")]
        self.nes_states.remove(&id);
        #[cfg(feature = "emulators")]
        self.gameboy_states.remove(&id);
        self.binary_viewer_states.remove(&id);
        self.lab_states.remove(&id);
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
                        self.handle_browser_click(x, y, &self.windows[i].clone());
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
            // Mouse released - stop dragging and resizing
            for w in &mut self.windows {
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
                    window.content.push(String::from("OS: TrustOS v0.2.0"));
                }
            },
            ContextAction::Cut => {
                if let Some(idx) = self.context_menu.target_icon {
                    self.clipboard_icon = Some((idx, true));
                    // Also store icon name in text clipboard
                    let name = self.icons[idx].name.clone();
                    crate::keyboard::clipboard_set(&name);
                    crate::serial_println!("[GUI] Cut icon: {}", name);
                }
            },
            ContextAction::Copy => {
                if let Some(idx) = self.context_menu.target_icon {
                    self.clipboard_icon = Some((idx, false));
                    let name = self.icons[idx].name.clone();
                    crate::keyboard::clipboard_set(&name);
                    crate::serial_println!("[GUI] Copied icon: {}", name);
                }
            },
            ContextAction::Paste => {
                if let Some((src_idx, is_cut)) = self.clipboard_icon.take() {
                    if src_idx < self.icons.len() {
                        if is_cut {
                            // Move: icon already exists, just log
                            crate::serial_println!("[GUI] Pasted (moved) icon: {}", self.icons[src_idx].name);
                        } else {
                            // Copy: duplicate the icon
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
                            crate::serial_println!("[GUI] Pasted (copied) icon: {}", new_name);
                        }
                    }
                }
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
            ContextAction::Delete => {
                if let Some(idx) = self.context_menu.target_icon {
                    if idx < self.icons.len() {
                        let name = self.icons[idx].name.clone();
                        self.icons.remove(idx);
                        // Clear clipboard if it referenced this or a later icon
                        self.clipboard_icon = None;
                        crate::serial_println!("[GUI] Deleted icon: {}", name);
                    }
                }
            },
            ContextAction::Rename => {
                if let Some(idx) = self.context_menu.target_icon {
                    if idx < self.icons.len() {
                        crate::serial_println!("[GUI] Rename icon: {} (inline rename not yet supported)", self.icons[idx].name);
                    }
                }
            },
            ContextAction::CopyPath => {
                if let Some(idx) = self.context_menu.target_icon {
                    if idx < self.icons.len() {
                        let path = format!("/desktop/{}", self.icons[idx].name);
                        crate::keyboard::clipboard_set(&path);
                        crate::serial_println!("[GUI] Copied path: {}", path);
                    }
                }
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
        
        // TrustOS button (left side, matches draw_taskbar)
        if x >= 4 && x < 112 {
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
        let menu_w = 420u32;
        let menu_h = 640u32;
        let menu_x = 4i32;
        let menu_y = (self.height - TASKBAR_HEIGHT - menu_h - 8) as i32;
        
        // Check if click is inside the start menu at all
        if x < menu_x || x >= menu_x + menu_w as i32 || y < menu_y || y >= menu_y + menu_h as i32 {
            return None;
        }
        
        // Search bar is at menu_y + 30, height 32 — clicking there focuses search (no action)
        let search_bar_bottom = menu_y + 66;
        let items_start_y = search_bar_bottom + 4;
        let item_spacing = 28;
        let item_h = 26;
        
        // App labels (indices 0-13, non-special)
        let app_labels: [&str; 14] = [
            "Terminal", "Files", "Calculator", "Network", "Text Editor",
            "TrustEdit 3D", "Browser", "Snake", "Chess", "Chess 3D",
            "NES Emulator", "Game Boy", "TrustLab", "Settings",
        ];
        let app_indices: [u8; 14] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13];
        
        // Power labels (indices 14-16, bottom-anchored)
        let power_labels: [&str; 3] = ["Exit Desktop", "Shutdown", "Reboot"];
        let power_indices: [u8; 3] = [14, 15, 16];
        
        let search = self.start_menu_search.trim();
        let search_lower: String = search.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
        
        // Check app items first
        let filtered_apps: alloc::vec::Vec<u8> = if search.is_empty() {
            app_indices.to_vec()
        } else {
            app_indices.iter().filter(|&&idx| {
                let label: String = app_labels[idx as usize].chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                label.contains(search_lower.as_str())
            }).copied().collect()
        };
        
        if y >= items_start_y && y < menu_y + menu_h as i32 - 106 {
            let clicked_row = ((y - items_start_y) / item_spacing) as usize;
            if clicked_row < filtered_apps.len() {
                let item_top = items_start_y + (clicked_row as i32 * item_spacing);
                if y < item_top + item_h {
                    return Some(filtered_apps[clicked_row]);
                }
            }
        }
        
        // Check power items (bottom-anchored)
        let power_y = menu_y + menu_h as i32 - 106;
        let power_start_y = power_y + 6;
        if y >= power_start_y {
            for (pi, &pidx) in power_indices.iter().enumerate() {
                // Filter by search
                if !search_lower.is_empty() {
                    let ll: String = power_labels[pi].chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                    if !ll.contains(search_lower.as_str()) { continue; }
                }
                let item_top = power_start_y + (pi as i32 * item_spacing);
                if y >= item_top && y < item_top + item_h {
                    return Some(pidx);
                }
            }
        }
        
        None
    }
    
    fn handle_menu_action(&mut self, action: u8) {
        // Matches draw_start_menu items array order:
        // 0=Terminal, 1=Files, 2=Calculator, 3=Network, 4=TextEditor,
        // 5=TrustEdit3D, 6=Browser, 7=Snake, 8=Chess, 9=Chess3D, 10=NES, 11=GameBoy, 12=TrustLab, 13=Settings, 14=Exit Desktop, 15=Shutdown, 16=Reboot
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
            8 => { // Chess
                self.create_window("TrustChess", 200, 80, 480, 520, WindowType::Chess);
            },
            9 => { // Chess 3D — open fullscreen
                let sw = self.width;
                let sh = self.height;
                let id = self.create_window("TrustChess 3D", 0, 0, sw, sh - TASKBAR_HEIGHT, WindowType::Chess3D);
                // Mark as maximized
                if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                    w.maximized = true;
                }
            },
            10 => { // NES Emulator
                #[cfg(feature = "emulators")]
                self.create_window("NES Emulator", 80, 50, 512, 480, WindowType::NesEmu);
            },
            11 => { // Game Boy
                #[cfg(feature = "emulators")]
                self.create_window("Game Boy", 100, 60, 480, 432, WindowType::GameBoyEmu);
            },
            12 => { // TrustLab
                self.open_lab_mode();
            },
            13 => { // Settings
                self.open_settings_panel();
            },
            14 => { // Exit Desktop
                crate::serial_println!("[GUI] Exit Desktop from start menu");
                EXIT_DESKTOP_FLAG.store(true, Ordering::SeqCst);
            },
            15 => { // Shutdown
                crate::println!("\n\n=== SYSTEM SHUTDOWN ===");
                crate::println!("Goodbye!");
                loop { crate::arch::halt(); }
            },
            16 => { // Reboot
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
                    // V to toggle view mode
                    if key == b'v' || key == b'V' {
                        let current = self.fm_view_modes.get(&win_id).copied().unwrap_or(FileManagerViewMode::List);
                        let new_mode = match current {
                            FileManagerViewMode::List => FileManagerViewMode::IconGrid,
                            FileManagerViewMode::IconGrid => FileManagerViewMode::List,
                        };
                        self.fm_view_modes.insert(win_id, new_mode);
                        crate::serial_println!("[FM] Toggled view mode for window {}", win_id);
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
        // Set file_path temporarily to know current path, then navigate to "."
        // We just re-call navigate with the same path by navigating to a dummy then back
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileManager) {
            let cp = window.file_path.clone().unwrap_or_else(|| String::from("/"));
            // Rebuild content in-place
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
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileManager) {
            // Build new path
            let current_path = window.file_path.clone().unwrap_or_else(|| String::from("/"));
            let new_path = if dirname == ".." {
                // Go up one level
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
            
            // Rebuild window content
            window.content.clear();
            window.content.push(String::from("=== File Manager ==="));
            window.content.push(format!("Path: {}", new_path));
            window.content.push(String::from(""));
            window.content.push(String::from("  Name              Type       Size    Program"));
            window.content.push(String::from("  ────────────────────────────────────────────"));
            
            // Add ".." entry if not at root
            if new_path != "/" {
                window.content.push(String::from("  [D] ..             DIR        ---     ---"));
            }
            
            // List actual files from ramfs
            let path_arg = if new_path == "/" { Some("/") } else { Some(new_path.as_str()) };
            if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(path_arg)) {
                for (name, ftype, size) in entries.iter().take(20) {
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
            if window.content.len() <= 5 + if new_path != "/" { 1 } else { 0 } {
                window.content.push(String::from("  (empty directory)"));
            }
            window.content.push(String::from(""));
            window.content.push(String::from("  [Del] Delete | [N] New File | [D] New Folder | [R] Rename"));
            
            window.file_path = Some(new_path);
            window.selected_index = 0;
            window.scroll_offset = 0;
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
        if key == b'1' {
            // Toggle animations
            toggle_animations();
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
        } else if key == b'5' {
            // Toggle high contrast
            crate::accessibility::toggle_high_contrast();
            self.needs_full_redraw = true;
            self.background_cached = false;
            self.refresh_settings_window();
        } else if key == b'6' {
            // Cycle font size
            crate::accessibility::cycle_font_size();
            self.refresh_settings_window();
        } else if key == b'7' {
            // Cycle cursor size
            crate::accessibility::cycle_cursor_size();
            self.refresh_settings_window();
        } else if key == b'8' {
            // Toggle sticky keys
            crate::accessibility::toggle_sticky_keys();
            self.refresh_settings_window();
        } else if key == b'9' {
            // Cycle mouse speed
            crate::accessibility::cycle_mouse_speed();
            self.refresh_settings_window();
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
        // Clear old suggestion lines so content.last() is the prompt again
        self.clear_terminal_suggestions();
        
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
            _ => {
                output.push(format!("\x01Rbash: \x01Wcommand not found: \x01G{}", cmd));
                output.push(String::from("\x01MType '\x01Ghelp\x01M' for available commands"));
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
        
        // Fifth pass: render 3D chess windows (needs &mut for state)
        self.draw_chess3d_windows();
        
        // Sixth pass: render emulator windows
        #[cfg(feature = "emulators")]
        self.draw_nes_windows();
        #[cfg(feature = "emulators")]
        self.draw_gameboy_windows();
        
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
        
        // Logo centered vertically in the available space
        let logo_center_y = height / 2;
        
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
                
                // Color computation
                let (r, g, b) = if i == 0 {
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
        
        // ── Draw TrustOS fleur-de-lis logo — centered on screen ──
        // Transparent interior: only outline/edge pixels drawn, matrix rain visible through
        {
            let logo_w = crate::logo_bitmap::LOGO_W as u32;
            let logo_h = crate::logo_bitmap::LOGO_H as u32;
            let logo_x = (width / 2).saturating_sub(logo_w / 2);
            // Center logo vertically in available space
            let logo_y = logo_center_y.saturating_sub(logo_h / 2);
            
            for ly in 0..logo_h {
                for lx in 0..logo_w {
                    let argb = crate::logo_bitmap::logo_pixel(lx as usize, ly as usize);
                    let a = (argb >> 24) & 0xFF;
                    let r = (argb >> 16) & 0xFF;
                    let g = (argb >> 8) & 0xFF;
                    let b = argb & 0xFF;
                    
                    // Skip near-black / transparent pixels
                    if a < 20 { continue; }
                    let luminance = (r * 77 + g * 150 + b * 29) >> 8;
                    if luminance < 15 { continue; }
                    
                    let px = logo_x + lx;
                    let py = logo_y + ly;
                    if px >= width || py >= height { continue; }
                    
                    let is_edge = crate::logo_bitmap::logo_edge_pixel(lx as usize, ly as usize);
                    
                    // Check if near edge (within 3px) for softer falloff
                    let near_edge = if !is_edge {
                        let mut min_dist = 99u32;
                        for dy in 0..5u32 {
                            for dx in 0..5u32 {
                                let nx = lx as i32 + dx as i32 - 2;
                                let ny = ly as i32 + dy as i32 - 2;
                                if nx >= 0 && ny >= 0 && nx < logo_w as i32 && ny < logo_h as i32 {
                                    if crate::logo_bitmap::logo_edge_pixel(nx as usize, ny as usize) {
                                        let d = ((dx as i32 - 2).unsigned_abs() + (dy as i32 - 2).unsigned_abs()) as u32;
                                        if d < min_dist { min_dist = d; }
                                    }
                                }
                            }
                        }
                        min_dist
                    } else { 0 };
                    
                    if is_edge {
                        // Edge pixels: bright green glow corona (3px)
                        for gdy in 0..5u32 {
                            for gdx in 0..5u32 {
                                let gx = px + gdx;
                                let gy = py + gdy;
                                if gx >= 2 && gy >= 2 && gx - 2 < width && gy - 2 < height {
                                    let existing = framebuffer::get_pixel(gx - 2, gy - 2);
                                    let eg = (existing >> 8) & 0xFF;
                                    if eg < 40 {
                                        framebuffer::put_pixel(gx - 2, gy - 2, 0xFF002A10);
                                    }
                                }
                            }
                        }
                        // Draw edge pixel: use original color but boost green channel
                        let edge_r = (r / 3).min(60);
                        let edge_g = (g.max(luminance) + 40).min(255);
                        let edge_b = (b / 3).min(60);
                        framebuffer::put_pixel(px, py, 0xFF000000 | (edge_r << 16) | (edge_g << 8) | edge_b);
                    } else if near_edge <= 2 {
                        // Near-edge: very subtle tint (15-30% alpha), matrix visible
                        let alpha_val = if near_edge == 1 { 45u32 } else { 20u32 };
                        let bg = framebuffer::get_pixel(px, py);
                        let inv = 255 - alpha_val;
                        let edge_g2 = g.max(60).min(200);
                        let nr = ((r / 4) * alpha_val + ((bg >> 16) & 0xFF) * inv) / 255;
                        let ng = (edge_g2 * alpha_val + ((bg >> 8) & 0xFF) * inv) / 255;
                        let nb = ((b / 4) * alpha_val + (bg & 0xFF) * inv) / 255;
                        framebuffer::put_pixel(px, py, 0xFF000000 | (nr << 16) | (ng << 8) | nb);
                    }
                    // Interior pixels (far from edge): skip entirely → matrix rain shows through
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
                // Inner highlight (rounded to match icon shape)
                draw_rounded_rect((ix as i32) - 3, (iy as i32) - 2, icon_size + 6, icon_size + 16, 6, 0xFF001A0A);
                draw_rounded_rect_border((ix as i32) - 3, (iy as i32) - 2, icon_size + 6, icon_size + 16, 6, 0xFF00AA44);
            }
            
            // Icon background — rounded dark square with colored accent glow
            let accent_color = match icon.icon_type {
                IconType::Terminal => 0xFF20CC60u32,  // Bright green
                IconType::Folder => 0xFFDDAA30u32,    // Warm amber/gold
                IconType::Editor => 0xFF5090E0u32,    // Soft blue  
                IconType::Calculator => 0xFFCC6633u32, // Orange
                IconType::Network => 0xFF40AADDu32,    // Cyan
                IconType::Game => 0xFFCC4444u32,       // Red
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
                draw_rounded_rect_border(ix as i32, iy as i32, icon_size, icon_size, 6, 0xFF1A2A1A);
            }
            
            // Use accent color for hovered icons, muted version for normal
            let draw_color = if is_hovered { accent_color } else { icon_color };
            
            // Pixel-art icon inside square
            let cx = ix + icon_size / 2;
            let cy = iy + icon_size / 2;
            use crate::icons::IconType;
            match icon.icon_type {
                IconType::Terminal => {
                    // Terminal: rounded rect with >_ prompt and blinking cursor
                    draw_rounded_rect_border((cx - 14) as i32, (cy - 10) as i32, 28, 20, 3, draw_color);
                    // Top bar of terminal window
                    framebuffer::fill_rect(cx - 13, cy - 9, 26, 3, draw_color);
                    // 3 tiny dots in top bar (traffic lights)
                    framebuffer::fill_rect(cx - 11, cy - 8, 2, 1, 0xFF0A0A0A);
                    framebuffer::fill_rect(cx - 8, cy - 8, 2, 1, 0xFF0A0A0A);
                    framebuffer::fill_rect(cx - 5, cy - 8, 2, 1, 0xFF0A0A0A);
                    // Prompt >_
                    self.draw_text((cx - 8) as i32, (cy - 2) as i32, ">", draw_color);
                    framebuffer::fill_rect(cx - 2, cy, 8, 2, draw_color);
                },
                IconType::Folder => {
                    // Files: folder with tab and inner shadow
                    // Folder tab
                    framebuffer::fill_rect(cx - 14, cy - 8, 12, 5, draw_color);
                    // Main folder body
                    framebuffer::fill_rect(cx - 14, cy - 3, 28, 15, draw_color);
                    // Inner shadow (darker interior)
                    framebuffer::fill_rect(cx - 12, cy - 1, 24, 11, 0xFF0A0A0A);
                    // File hint lines inside
                    framebuffer::fill_rect(cx - 8, cy + 2, 16, 1, 0xFF303020);
                    framebuffer::fill_rect(cx - 8, cy + 5, 12, 1, 0xFF303020);
                },
                IconType::Editor => {
                    // Editor: document with code lines (syntax colored)
                    framebuffer::fill_rect(cx - 10, cy - 12, 20, 24, draw_color);
                    // Dog-eared corner
                    framebuffer::fill_rect(cx + 4, cy - 12, 6, 6, 0xFF0A0A0A);
                    framebuffer::fill_rect(cx + 4, cy - 12, 1, 6, draw_color);
                    framebuffer::fill_rect(cx + 4, cy - 7, 6, 1, draw_color);
                    // Dark interior
                    framebuffer::fill_rect(cx - 8, cy - 6, 16, 16, 0xFF0A0A0A);
                    // Code-like lines with colors
                    framebuffer::fill_rect(cx - 6, cy - 4, 6, 1, 0xFF6688CC); // blue keyword
                    framebuffer::fill_rect(cx - 6, cy - 2, 10, 1, draw_color);
                    framebuffer::fill_rect(cx - 6, cy + 0, 8, 1, 0xFFCC8844); // orange string
                    framebuffer::fill_rect(cx - 6, cy + 2, 12, 1, draw_color);
                    framebuffer::fill_rect(cx - 6, cy + 4, 4, 1, 0xFF88BB44); // green comment
                    framebuffer::fill_rect(cx - 6, cy + 6, 9, 1, draw_color);
                },
                IconType::Calculator => {
                    // Calculator: screen + colored button grid
                    draw_rounded_rect_border((cx - 10) as i32, (cy - 12) as i32, 20, 24, 2, draw_color);
                    // LED screen
                    framebuffer::fill_rect(cx - 8, cy - 10, 16, 6, 0xFF1A3320);
                    self.draw_text((cx - 4) as i32, (cy - 10) as i32, "42", 0xFF40FF40);
                    // Button grid (3x3)
                    for row in 0..3u32 {
                        for col in 0..3u32 {
                            let bx = cx - 8 + col * 6;
                            let by = cy - 1 + row * 5;
                            let btn_col = if row == 2 && col == 2 { 0xFFCC6633 } else { draw_color };
                            framebuffer::fill_rect(bx, by, 4, 3, btn_col);
                        }
                    }
                },
                IconType::Network => {
                    // Network: Wi-Fi arcs + signal strength
                    let arc_cx = cx as i32;
                    let arc_cy = (cy + 4) as i32;
                    // Signal arcs (3 concentric)
                    for ring in 0..3u32 {
                        let r = 4 + ring * 4;
                        let r2 = (r * r) as i32;
                        let r2_inner = ((r.saturating_sub(2)) * (r.saturating_sub(2))) as i32;
                        for dy in 0..r as i32 + 1 {
                            for dx in -(r as i32)..=(r as i32) {
                                let dist2 = dx * dx + dy * dy;
                                if dist2 <= r2 && dist2 >= r2_inner && dy <= 0 {
                                    let px = (arc_cx + dx) as u32;
                                    let py = (arc_cy + dy) as u32;
                                    if px >= ix && px < ix + icon_size && py >= iy && py < iy + icon_size {
                                        let fade = if ring == 0 { draw_color } 
                                            else if ring == 1 { if is_hovered { draw_color } else { GREEN_GHOST } }
                                            else { GREEN_GHOST };
                                        framebuffer::put_pixel(px, py, fade);
                                    }
                                }
                            }
                        }
                    }
                    // Center dot
                    framebuffer::fill_rect(cx - 1, cy + 3, 3, 3, draw_color);
                },
                IconType::Game => {
                    // Game: game controller / gamepad
                    // Controller body (wide rounded shape)
                    framebuffer::fill_rect(cx - 12, cy - 4, 24, 12, draw_color);
                    framebuffer::fill_rect(cx - 14, cy - 2, 4, 8, draw_color);
                    framebuffer::fill_rect(cx + 10, cy - 2, 4, 8, draw_color);
                    // Dark interior
                    framebuffer::fill_rect(cx - 11, cy - 3, 22, 10, 0xFF0A0A0A);
                    // D-pad (left)
                    framebuffer::fill_rect(cx - 9, cy - 1, 5, 1, draw_color);
                    framebuffer::fill_rect(cx - 7, cy - 3, 1, 5, draw_color);
                    // Action buttons (right) — colored dots
                    framebuffer::fill_rect(cx + 5, cy - 2, 2, 2, 0xFF4488DD); // blue
                    framebuffer::fill_rect(cx + 8, cy - 1, 2, 2, ACCENT_RED); // red
                    framebuffer::fill_rect(cx + 5, cy + 1, 2, 2, 0xFF44DD44); // green
                    framebuffer::fill_rect(cx + 8, cy + 2, 2, 2, 0xFFDDDD44); // yellow
                },
                IconType::Settings => {
                    // Settings: detailed gear with teeth
                    for dy in 0..18u32 {
                        for dx in 0..18u32 {
                            let ddx = dx as i32 - 9;
                            let ddy = dy as i32 - 9;
                            let dist_sq = ddx * ddx + ddy * ddy;
                            // Outer ring
                            if dist_sq >= 30 && dist_sq <= 64 {
                                framebuffer::put_pixel(cx - 9 + dx, cy - 9 + dy, draw_color);
                            }
                            // Inner circle
                            if dist_sq <= 9 {
                                framebuffer::put_pixel(cx - 9 + dx, cy - 9 + dy, draw_color);
                            }
                        }
                    }
                    // 8 gear teeth
                    let teeth: &[(i32, i32)] = &[(0, -9), (0, 9), (-9, 0), (9, 0), (-7, -7), (7, -7), (-7, 7), (7, 7)];
                    for &(tx, ty) in teeth {
                        let px = (cx as i32 + tx) as u32;
                        let py = (cy as i32 + ty) as u32;
                        framebuffer::fill_rect(px.saturating_sub(1), py.saturating_sub(1), 3, 3, draw_color);
                    }
                },
                IconType::Browser => {
                    // Browser: globe with meridians and parallels
                    for dy in 0..20u32 {
                        for dx in 0..20u32 {
                            let ddx = dx as i32 - 10;
                            let ddy = dy as i32 - 10;
                            let dist_sq = ddx * ddx + ddy * ddy;
                            // Outer circle
                            if dist_sq >= 72 && dist_sq <= 100 {
                                framebuffer::put_pixel(cx - 10 + dx, cy - 10 + dy, draw_color);
                            }
                        }
                    }
                    // Equator
                    framebuffer::fill_rect(cx - 10, cy - 1, 20, 2, draw_color);
                    // Vertical meridian
                    framebuffer::fill_rect(cx - 1, cy - 10, 2, 20, draw_color);
                    // Elliptical meridian (curved) — integer sqrt approximation
                    for dy in 0..20u32 {
                        let ddy = dy as i32 - 10;
                        let val = 100 - ddy * ddy;
                        if val > 0 {
                            let curve_x = (fast_sqrt_i32(val) * 2 / 5) as u32; // ~0.4 * sqrt
                            let px1 = cx + curve_x;
                            let px2 = cx.saturating_sub(curve_x);
                            if px1 > ix && px1 < ix + icon_size {
                                framebuffer::put_pixel(px1, cy - 10 + dy, draw_color);
                            }
                            if px2 > ix && px2 < ix + icon_size {
                                framebuffer::put_pixel(px2, cy - 10 + dy, draw_color);
                            }
                        }
                    }
                    // Parallels (latitude lines)
                    framebuffer::fill_rect(cx - 8, cy - 5, 16, 1, draw_color);
                    framebuffer::fill_rect(cx - 8, cy + 5, 16, 1, draw_color);
                },
                IconType::GameBoy => {
                    // Game Boy: handheld console (detailed)
                    draw_rounded_rect_border((cx - 10) as i32, (cy - 12) as i32, 20, 24, 2, draw_color);
                    framebuffer::fill_rect(cx - 9, cy - 11, 18, 22, draw_color);
                    // Screen area (green-tinted)
                    framebuffer::fill_rect(cx - 7, cy - 9, 14, 10, 0xFF1A3320);
                    // Pixel character on screen
                    framebuffer::fill_rect(cx - 2, cy - 7, 4, 4, 0xFF40CC40);
                    framebuffer::fill_rect(cx - 3, cy - 3, 6, 2, 0xFF40CC40);
                    // D-pad
                    framebuffer::fill_rect(cx - 7, cy + 3, 5, 2, 0xFF0A0A0A);
                    framebuffer::fill_rect(cx - 5, cy + 1, 1, 6, 0xFF0A0A0A);
                    // A/B buttons (colored)
                    framebuffer::fill_rect(cx + 4, cy + 2, 3, 3, ACCENT_RED);
                    framebuffer::fill_rect(cx + 1, cy + 4, 3, 3, 0xFF4488DD);
                },
                _ => {
                    // Generic: bordered square with inner pattern
                    draw_rounded_rect_border((cx - 10) as i32, (cy - 10) as i32, 20, 20, 3, draw_color);
                    framebuffer::fill_rect(cx - 4, cy - 4, 8, 8, draw_color);
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
        // TRANSLUCENT ROUNDED TASKBAR — Frosted glass, matrix shows through
        // ═══════════════════════════════════════════════════════════════
        
        // Rounded translucent background — alpha-blended over matrix rain
        {
            let radius = 12u32;
            let ri = radius as i32;
            let r2 = ri * ri;
            let w = self.width;
            
            // Top rows: rounded corners (per-row scanline)
            for row in 0..radius {
                let vert_dist = ri - row as i32;
                let horiz = fast_sqrt_i32(r2 - vert_dist * vert_dist) as u32;
                let left_indent = radius - horiz;
                let visible_w = w.saturating_sub(left_indent * 2);
                if visible_w > 0 {
                    framebuffer::fill_rect_alpha(left_indent, y + row, visible_w, 1, 0x040A06, 155);
                }
            }
            // Main body below rounded zone
            framebuffer::fill_rect_alpha(0, y + radius, w, TASKBAR_HEIGHT - radius, 0x040A06, 155);
            // Subtle green tint overlay
            framebuffer::fill_rect_alpha(0, y + radius, w, TASKBAR_HEIGHT - radius, 0x00AA44, 8);
            
            // Rounded top border — glowing green arc
            for row in 0..radius {
                let vert_dist = ri - row as i32;
                let horiz = fast_sqrt_i32(r2 - vert_dist * vert_dist) as u32;
                let left_x = radius - horiz;
                let right_x = w - radius + horiz;
                if left_x < w {
                    framebuffer::put_pixel(left_x, y + row, 0xFF0D5D2A);
                }
                if right_x > 0 && right_x - 1 < w {
                    framebuffer::put_pixel(right_x - 1, y + row, 0xFF0D5D2A);
                }
            }
            // Straight top border between corners
            if w > radius * 2 {
                for px in radius..(w - radius) {
                    framebuffer::put_pixel(px, y, 0xFF0D5D2A);
                }
            }
        }
        
        // ── TrustOS button (left) — rounded pill shape ──
        let start_hover = self.cursor_x >= 4 && self.cursor_x < 112 && self.cursor_y >= y as i32;
        if start_hover || self.start_menu_open {
            draw_rounded_rect(4, (y + 5) as i32, 104, 30, 8, 0xFF003318);
            framebuffer::fill_rect_alpha(4, y + 5, 104, 30, 0x00CC66, 50);
        }
        // Rounded border
        let border_color = if start_hover || self.start_menu_open { GREEN_PRIMARY } else { GREEN_GHOST };
        draw_rounded_rect_border(4, (y + 5) as i32, 104, 30, 8, border_color);
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
            
            // Button background — rounded translucent glass pill
            if w.focused {
                draw_rounded_rect(btn_x as i32, btn_y as i32, btn_w, 30, 6, 0xFF001A0A);
                framebuffer::fill_rect_alpha(btn_x, btn_y, btn_w, 30, 0x00AA44, 60);
            } else if is_hover {
                draw_rounded_rect(btn_x as i32, btn_y as i32, btn_w, 30, 6, 0xFF000D05);
                framebuffer::fill_rect_alpha(btn_x, btn_y, btn_w, 30, 0x008833, 40);
            }
            // Rounded border
            let bdr = if w.focused { GREEN_PRIMARY } else if is_hover { GREEN_MUTED } else { GREEN_GHOST };
            draw_rounded_rect_border(btn_x as i32, btn_y as i32, btn_w, 30, 6, bdr);
            
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
        self.draw_text_smooth((self.width - 130) as i32, (y + 8) as i32, &time, hc(GREEN_PRIMARY, 0xFFFFFFFF));
        
        // Date below clock
        let date = self.get_date_string();
        self.draw_text_smooth((self.width - 130) as i32, (y + 22) as i32, &date, hc(GREEN_TERTIARY, 0xFFCCCCCC));
        
        // Accessibility status indicators (left of clock)
        let a11y_str = crate::accessibility::status_indicators();
        if !a11y_str.is_empty() {
            let a11y_x = (self.width - 210) as i32;
            self.draw_text_smooth(a11y_x, (y + 14) as i32, &a11y_str, hc(ACCENT_AMBER, 0xFFFFFF00));
        }
        
        // ── System tray indicators (WiFi, Volume, Battery) ──
        self.draw_sys_tray_indicators(self.width - 290, y + 6);

        // System indicators (CPU + MEM mini-bars)
        let ind_x = self.width - 50;
        let ind_y = y + 6;
        // CPU indicator — simulated activity based on frame timing
        let cpu_level = ((self.frame_count % 7) + 2).min(6) as u32; // 2-6 of 8 segments
        self.draw_text((ind_x - 24) as i32, (ind_y + 1) as i32, "C", GREEN_GHOST);
        for seg in 0..8u32 {
            let seg_color = if seg < cpu_level {
                if cpu_level > 6 { ACCENT_RED } else { GREEN_PRIMARY }
            } else { GREEN_GHOST };
            framebuffer::fill_rect(ind_x + seg * 3, ind_y + 2, 2, 8, seg_color);
        }
        // MEM indicator — gets actual memory usage from allocator stats
        let mem_level = {
            let total = 16u32; // normalized to 16 segments (represents "total")
            let used = ((self.windows.len() as u32 * 2) + 4).min(total); // rough estimate
            (used * 8 / total).min(8)
        };
        self.draw_text((ind_x - 24) as i32, (ind_y + 15) as i32, "M", GREEN_GHOST);
        for seg in 0..8u32 {
            let seg_color = if seg < mem_level {
                if mem_level > 6 { ACCENT_AMBER } else { GREEN_PRIMARY }
            } else { GREEN_GHOST };
            framebuffer::fill_rect(ind_x + seg * 3, ind_y + 16, 2, 8, seg_color);
        }
        
        // ── Settings gear icon (left of indicators, no overlap) ──
        let gear_x = self.width - 80;
        let gear_y = y + 12;
        let gear_hover = self.cursor_x >= (gear_x as i32 - 4) && self.cursor_x < (gear_x as i32 + 20)
            && self.cursor_y >= y as i32;
        let gear_color = if gear_hover { GREEN_PRIMARY } else { GREEN_TERTIARY };
        if gear_hover {
            // Subtle hover glow background
            framebuffer::fill_rect_alpha(gear_x - 2, gear_y - 2, 20, 20, 0x00CC66, 30);
        }
        // Outer gear ring with teeth
        for dy in 0..16u32 {
            for dx in 0..16u32 {
                let ddx = dx as i32 - 8;
                let ddy = dy as i32 - 8;
                let dist_sq = ddx * ddx + ddy * ddy;
                // Outer ring
                if dist_sq >= 25 && dist_sq <= 56 {
                    framebuffer::put_pixel(gear_x + dx, gear_y + dy, gear_color);
                }
                // Inner dot
                if dist_sq <= 6 {
                    framebuffer::put_pixel(gear_x + dx, gear_y + dy, gear_color);
                }
            }
        }
        // Gear teeth (small rects at cardinal + diagonal directions)
        let teeth: &[(i32, i32)] = &[(0, -8), (0, 8), (-8, 0), (8, 0), (-6, -6), (6, -6), (-6, 6), (6, 6)];
        for &(tx, ty) in teeth {
            let px = (gear_x as i32 + 8 + tx) as u32;
            let py = (gear_y as i32 + 8 + ty) as u32;
            framebuffer::fill_rect(px.saturating_sub(1), py.saturating_sub(1), 3, 3, gear_color);
        }
        
        // ── Show Desktop button (far right corner, Windows-style) ──
        let sd_x = self.width - 8;
        let sd_w = 8u32;
        let sd_hover = self.cursor_x >= sd_x as i32 && self.cursor_y >= y as i32;
        let sd_color = if sd_hover { GREEN_MUTED } else { GREEN_GHOST };
        framebuffer::fill_rect(sd_x, y, sd_w, TASKBAR_HEIGHT, sd_color);
        // Thin separator line
        framebuffer::fill_rect(sd_x, y + 4, 1, TASKBAR_HEIGHT - 8, GREEN_SUBTLE);
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
        let menu_w = 420u32;
        let menu_h = 640u32;
        let menu_x = 4i32;
        let menu_y = (self.height - TASKBAR_HEIGHT - menu_h - 8) as i32;
        
        let is_hc = crate::accessibility::is_high_contrast();
        
        // ═══════════════════════════════════════════════════════════════
        // MATRIX HACKER STYLE START MENU — Wide frosted glass popup with search
        // ═══════════════════════════════════════════════════════════════
        
        // Frosted dark glass background
        if is_hc {
            framebuffer::fill_rect(menu_x as u32, menu_y as u32, menu_w, menu_h, 0xFF000000);
        } else {
            framebuffer::fill_rect_alpha(menu_x as u32, menu_y as u32, menu_w, menu_h, 0x060A08, 210);
        }
        
        // Double border
        let border1 = hc(GREEN_PRIMARY, 0xFFFFFFFF);
        let border2 = hc(GREEN_SUBTLE, 0xFF888888);
        framebuffer::draw_rect(menu_x as u32, menu_y as u32, menu_w, menu_h, border1);
        framebuffer::draw_rect((menu_x + 1) as u32, (menu_y + 1) as u32, menu_w - 2, menu_h - 2, border2);
        
        // Title bar
        if is_hc {
            framebuffer::fill_rect((menu_x + 2) as u32, (menu_y + 2) as u32, menu_w - 4, 24, 0xFF1A1A1A);
        } else {
            framebuffer::fill_rect_alpha((menu_x + 2) as u32, (menu_y + 2) as u32, menu_w - 4, 24, 0x002200, 180);
        }
        self.draw_text_smooth(menu_x + 10, menu_y + 6, "TrustOS Menu", hc(GREEN_PRIMARY, 0xFFFFFF00));
        
        // Separator
        framebuffer::draw_hline((menu_x + 2) as u32, (menu_y + 26) as u32, menu_w - 4, GREEN_MUTED);
        
        // ── Search bar ──
        let search_y = menu_y + 30;
        let search_h = 32u32;
        let search_pad = 8i32;
        // Search bar background
        framebuffer::fill_rect((menu_x + search_pad) as u32, search_y as u32, menu_w - search_pad as u32 * 2, search_h, 0xFF0A120A);
        framebuffer::draw_rect((menu_x + search_pad) as u32, search_y as u32, menu_w - search_pad as u32 * 2, search_h, GREEN_MUTED);
        
        // Search icon (magnifying glass)
        let mag_x = menu_x + search_pad + 8;
        let mag_y = search_y + 8;
        // Circle part
        for dy in 0..10u32 {
            for dx in 0..10u32 {
                let ddx = dx as i32 - 5;
                let ddy = dy as i32 - 5;
                let dist = ddx * ddx + ddy * ddy;
                if dist >= 12 && dist <= 25 {
                    framebuffer::put_pixel((mag_x + dx as i32) as u32, (mag_y + dy as i32) as u32, GREEN_TERTIARY);
                }
            }
        }
        // Handle part
        framebuffer::fill_rect((mag_x + 8) as u32, (mag_y + 8) as u32, 4, 2, GREEN_TERTIARY);
        
        // Search text or placeholder
        let search_text_x = menu_x + search_pad + 22;
        if self.start_menu_search.is_empty() {
            self.draw_text_smooth(search_text_x, search_y + 10, "Search apps...", GREEN_GHOST);
        } else {
            self.draw_text_smooth(search_text_x, search_y + 10, &self.start_menu_search, GREEN_PRIMARY);
            // Cursor blink
            let cursor_x = search_text_x + (self.start_menu_search.len() as i32 * 8);
            if (self.frame_count / 30) % 2 == 0 {
                framebuffer::fill_rect(cursor_x as u32, (search_y + 8) as u32, 2, 16, GREEN_PRIMARY);
            }
        }
        
        let items_start_y = search_y + search_h as i32 + 4;
        
        // Menu items — full list
        let items: [(&str, &str, bool); 17] = [
            (">_", "Terminal", false),
            ("[]", "Files", false),
            ("##", "Calculator", false),
            ("~~", "Network", false),
            ("Tx", "Text Editor", false),
            ("/\\", "TrustEdit 3D", false),
            ("WW", "Browser", false),
            ("Sk", "Snake", false),
            ("Kk", "Chess", false),
            ("C3", "Chess 3D", false),
            ("NE", "NES Emulator", false),
            ("GB", "Game Boy", false),
            ("Lb", "TrustLab", false),
            ("@)", "Settings", false),
            ("<-", "Exit Desktop", true),
            ("!!", "Shutdown", true),
            (">>", "Reboot", true),
        ];
        
        // Filter items by search
        let search = self.start_menu_search.trim();
        let search_lower: alloc::string::String = search.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
        
        // Draw app items (non-special, indices 0-13)
        let mut drawn = 0usize;
        for (ii, (icon, label, is_special)) in items.iter().enumerate() {
            if *is_special { continue; } // Power items drawn separately
            
            // Filter: skip items that don't match search
            if !search_lower.is_empty() {
                let label_lower: alloc::string::String = label.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                if !label_lower.contains(search_lower.as_str()) {
                    continue;
                }
            }
            
            let item_y = items_start_y + (drawn as i32 * 28);
            let item_h = 26u32;
            drawn += 1;
            
            // Don't draw past the power section
            if item_y + item_h as i32 > menu_y + menu_h as i32 - 110 { break; }
            
            // Hover or keyboard selection detection
            let is_hovered = self.cursor_x >= menu_x 
                && self.cursor_x < menu_x + menu_w as i32
                && self.cursor_y >= item_y 
                && self.cursor_y < item_y + item_h as i32;
            let is_selected = self.start_menu_selected == ii as i32;
            
            if is_hovered || is_selected {
                framebuffer::fill_rect_alpha((menu_x + 3) as u32, item_y as u32, menu_w - 6, item_h, 0x00AA44, if is_selected { 70 } else { 50 });
                framebuffer::fill_rect((menu_x + 3) as u32, (item_y + 3) as u32, 2, item_h - 6, GREEN_PRIMARY);
            }
            
            // Icon
            let icon_color = if is_hovered || is_selected { GREEN_PRIMARY } else { GREEN_TERTIARY };
            self.draw_text_smooth(menu_x + 14, item_y + 6, icon, icon_color);
            
            // Label
            let label_color = if is_hovered || is_selected { GREEN_SECONDARY } else { GREEN_SECONDARY };
            self.draw_text_smooth(menu_x + 40, item_y + 6, label, label_color);
            
            // Show search match highlight
            if !search_lower.is_empty() && is_hovered {
                let kw_x = menu_x + 40 + (label.len() as i32 * 8) + 8;
                self.draw_text(kw_x, item_y + 8, "*", GREEN_GHOST);
            }
        }
        
        // "No results" when search yields nothing (and no power items match either)
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
        
        // ── Power section (bottom-anchored with separator) ──
        let power_y = menu_y + menu_h as i32 - 106;
        // Separator line
        framebuffer::draw_hline((menu_x + 8) as u32, power_y as u32, menu_w - 16, GREEN_MUTED);
        
        let power_items: [(&str, &str, u8); 3] = [
            ("<-", "Exit Desktop", 14),
            ("!!", "Shutdown", 15),
            (">>", "Reboot", 16),
        ];
        
        for (pi, (icon, label, idx)) in power_items.iter().enumerate() {
            // Filter by search
            if !search_lower.is_empty() {
                let label_lower: String = label.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                if !label_lower.contains(search_lower.as_str()) {
                    continue;
                }
            }
            
            let item_y = power_y + 6 + (pi as i32 * 28);
            let item_h = 26u32;
            
            let is_hovered = self.cursor_x >= menu_x 
                && self.cursor_x < menu_x + menu_w as i32
                && self.cursor_y >= item_y 
                && self.cursor_y < item_y + item_h as i32;
            let is_selected = self.start_menu_selected == *idx as i32;
            
            if is_hovered || is_selected {
                framebuffer::fill_rect_alpha((menu_x + 3) as u32, item_y as u32, menu_w - 6, item_h, 0x00AA44, if is_selected { 70 } else { 50 });
                framebuffer::fill_rect((menu_x + 3) as u32, (item_y + 3) as u32, 2, item_h - 6, ACCENT_RED);
            }
            
            let icon_color = if is_hovered || is_selected { ACCENT_RED } else { 0xFF994444 };
            self.draw_text_smooth(menu_x + 14, item_y + 6, icon, icon_color);
            
            let label_color = if is_hovered || is_selected { ACCENT_RED } else { 0xFFAA4444 };
            self.draw_text_smooth(menu_x + 40, item_y + 6, label, label_color);
        }
        
        // Bottom: version info
        let ver_y = menu_y + menu_h as i32 - 20;
        framebuffer::draw_hline((menu_x + 4) as u32, ver_y as u32, menu_w - 8, GREEN_GHOST);
        self.draw_text(menu_x + 10, ver_y + 6, "TrustOS v0.4.2", GREEN_SUBTLE);
    }
    
    fn draw_window(&self, window: &Window) {
        let x = window.x;
        let y = window.y;
        let w = window.width;
        let h = window.height;
        
        // ═══════════════════════════════════════════════════════════════
        // MATRIX HACKER STYLE WINDOW — Green borders, dark background
        // ═══════════════════════════════════════════════════════════════
        
        // Drop shadow (subtle depth effect, only for non-maximized windows)
        let corner_radius = if window.maximized { 0u32 } else { 8u32 };
        if !window.maximized && w > 4 && h > 4 {
            // 3-layer shadow: decreasing opacity at increasing offset
            framebuffer::fill_rect_alpha((x + 6) as u32, (y + 6) as u32, w, h, 0x000000, 30);
            framebuffer::fill_rect_alpha((x + 4) as u32, (y + 4) as u32, w, h, 0x000000, 22);
            framebuffer::fill_rect_alpha((x + 2) as u32, (y + 2) as u32, w + 2, h + 2, 0x000000, 15);
        }
        
        // Window background: near-black with rounded corners (increased radius from 6→8)
        let win_bg = hc(0xFF0A0A0A, 0xFF000000);
        if corner_radius > 0 {
            draw_rounded_rect(x, y, w, h, corner_radius, win_bg);
        } else {
            framebuffer::fill_rect(x as u32, y as u32, w, h, win_bg);
        }
        
        // Green border (rounded)
        let border_color = if window.focused {
            hc(GREEN_PRIMARY, 0xFFFFFFFF)
        } else {
            hc(GREEN_SUBTLE, 0xFF888888)
        };
        if corner_radius > 0 {
            draw_rounded_rect_border(x, y, w, h, corner_radius, border_color);
            if w > 4 && h > 4 {
                draw_rounded_rect_border(x + 1, y + 1, w - 2, h - 2, corner_radius.saturating_sub(1), 
                    if window.focused { GREEN_MUTED } else { GREEN_GHOST });
            }
        } else {
            framebuffer::draw_rect(x as u32, y as u32, w, h, border_color);
            if w > 4 && h > 4 {
                framebuffer::draw_rect((x + 1) as u32, (y + 1) as u32, w - 2, h - 2, 
                    if window.focused { GREEN_MUTED } else { GREEN_GHOST });
            }
        }
        
        // Visual resize edge indicators (glow strips when hovering)
        if window.focused && !window.maximized && w > 20 && h > 20 {
            let edge = window.on_resize_edge(self.cursor_x, self.cursor_y);
            let glow_color = 0x00FF66u32; // bright green glow
            let glow_alpha = 40u32;
            let gt = 3u32;
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

        // Title bar: frosted glass effect with subtle gradient
        let titlebar_h = TITLE_BAR_HEIGHT;
        if window.focused {
            framebuffer::fill_rect_alpha((x + 2) as u32, (y + 2) as u32, w - 4, titlebar_h - 2, 0x0A1A0A, 220);
            // Subtle highlight at top of title bar
            framebuffer::fill_rect_alpha((x + 2) as u32, (y + 2) as u32, w - 4, 1, 0x00FF66, 15);
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
            WindowType::Chess => "Kk",
            WindowType::Chess3D => "C3",
            _ => "::",
        };
        let icon_color = if window.focused { GREEN_PRIMARY } else { GREEN_TERTIARY };
        self.draw_text_smooth(x + 10, y + 7, icon_str, icon_color);
        
        // Title text
        let text_color = if window.focused {
            hc(TEXT_PRIMARY, 0xFFFFFFFF)
        } else {
            hc(TEXT_SECONDARY, 0xFFCCCCCC)
        };
        self.draw_text_smooth(x + 32, y + 7, &window.title, text_color);
        
        // ═══════════════════════════════════════════════════════════════
        // Control Buttons (Windows-style: minimize | maximize | close)
        // ═══════════════════════════════════════════════════════════════
        let btn_w = 32u32;
        let btn_h = titlebar_h - 2;
        let btn_y_top = y + 2;
        let btn_y_center = y + titlebar_h as i32 / 2;
        let mx = self.cursor_x;
        let my = self.cursor_y;
        
        // Close button (rightmost, red on hover)
        let close_x = x + w as i32 - btn_w as i32 - 2;
        let close_hover = mx >= close_x && mx < close_x + btn_w as i32 && my >= btn_y_top && my < btn_y_top + btn_h as i32;
        if close_hover {
            framebuffer::fill_rect(close_x as u32, btn_y_top as u32, btn_w, btn_h, 0xFFCC3333);
        }
        // X icon
        let cx = close_x + btn_w as i32 / 2;
        let cy = btn_y_center;
        let x_color = if close_hover { 0xFFFFFFFF } else { if window.focused { GREEN_SECONDARY } else { GREEN_GHOST } };
        for i in -3..=3i32 {
            framebuffer::put_pixel((cx + i) as u32, (cy + i) as u32, x_color);
            framebuffer::put_pixel((cx + i) as u32, (cy - i) as u32, x_color);
            // Thicken
            framebuffer::put_pixel((cx + i + 1) as u32, (cy + i) as u32, x_color);
            framebuffer::put_pixel((cx + i + 1) as u32, (cy - i) as u32, x_color);
        }
        
        // Maximize button (middle)
        let max_x = close_x - btn_w as i32;
        let max_hover = mx >= max_x && mx < max_x + btn_w as i32 && my >= btn_y_top && my < btn_y_top + btn_h as i32;
        if max_hover {
            framebuffer::fill_rect(max_x as u32, btn_y_top as u32, btn_w, btn_h, GREEN_GHOST);
        }
        // Square icon
        let sq_color = if max_hover { GREEN_PRIMARY } else { if window.focused { GREEN_SECONDARY } else { GREEN_GHOST } };
        let sq_x = max_x + btn_w as i32 / 2 - 4;
        let sq_y = cy - 4;
        if window.maximized {
            // Overlapping squares for restore icon
            framebuffer::draw_rect((sq_x + 2) as u32, (sq_y) as u32, 6, 6, sq_color);
            framebuffer::draw_rect((sq_x) as u32, (sq_y + 2) as u32, 6, 6, sq_color);
        } else {
            framebuffer::draw_rect(sq_x as u32, sq_y as u32, 8, 8, sq_color);
        }
        
        // Minimize button (leftmost of the three)
        let min_x = max_x - btn_w as i32;
        let min_hover = mx >= min_x && mx < min_x + btn_w as i32 && my >= btn_y_top && my < btn_y_top + btn_h as i32;
        if min_hover {
            framebuffer::fill_rect(min_x as u32, btn_y_top as u32, btn_w, btn_h, GREEN_GHOST);
        }
        // Dash icon (horizontal line)
        let dash_color = if min_hover { GREEN_PRIMARY } else { if window.focused { GREEN_SECONDARY } else { GREEN_GHOST } };
        framebuffer::fill_rect((min_x + btn_w as i32 / 2 - 4) as u32, cy as u32, 8, 1, dash_color);
        framebuffer::fill_rect((min_x + btn_w as i32 / 2 - 4) as u32, (cy + 1) as u32, 8, 1, dash_color);
        
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
        // TERMINAL — special rendering with colored text + scrollbar
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
                                    'R' => ACCENT_RED,           // Red (root, errors)
                                    'G' => GREEN_PRIMARY,        // Bright green  
                                    'B' => ACCENT_BLUE,          // Cyan/blue
                                    'W' => TEXT_PRIMARY,          // White
                                    'Y' => ACCENT_AMBER,         // Yellow/amber
                                    'M' => GREEN_MUTED,          // Muted green
                                    'D' => GREEN_GHOST,          // Dim
                                    'N' => COLOR_GREEN,          // Normal green
                                    'H' => 0xFF00FFAA,           // Header bright
                                    'A' => GREEN_TERTIARY,       // Accent tertiary
                                    'S' => GREEN_SUBTLE,         // Subtle
                                    _ => current_color,
                                };
                            }
                        } else {
                            crate::framebuffer::draw_char_at(cx as u32, line_y as u32, ch, current_color);
                            cx += 8;
                        }
                    }
                } else {
                    // Plain green text (legacy)
                    self.draw_text(content_x, line_y, line, COLOR_GREEN);
                }
            }
            
            // ── Scrollbar ──
            let scrollbar_w = 6u32;
            let scrollbar_x = (window.x + window.width as i32 - scrollbar_w as i32 - 3) as u32;
            let track_y = (window.y + TITLE_BAR_HEIGHT as i32 + 2) as u32;
            let track_h = window.height.saturating_sub(TITLE_BAR_HEIGHT + 4);
            
            if total_lines > visible_lines {
                // Draw track
                framebuffer::fill_rect(scrollbar_x, track_y, scrollbar_w, track_h, 0xFF0A1A0F);
                
                // Draw thumb
                let thumb_h = ((visible_lines as u32 * track_h) / total_lines as u32).max(16);
                let max_scroll = total_lines.saturating_sub(visible_lines);
                let thumb_y = if max_scroll > 0 {
                    track_y + ((scroll as u32 * (track_h - thumb_h)) / max_scroll as u32)
                } else {
                    track_y
                };
                
                framebuffer::fill_rect(scrollbar_x, thumb_y, scrollbar_w, thumb_h, GREEN_MUTED);
                // Thumb border highlight
                framebuffer::fill_rect(scrollbar_x, thumb_y, 1, thumb_h, GREEN_SUBTLE);
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
                        framebuffer::put_pixel(sx, sy, color);
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
                        framebuffer::put_pixel(sx, sy, color);
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
                        framebuffer::put_pixel(sx, sy, color);
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
                            framebuffer::put_pixel(px + dx, py, 0xFF00FF66);
                            framebuffer::put_pixel(px, py + dx, 0xFF00FF66);
                            framebuffer::put_pixel(px + cell_size - 1 - dx, py, 0xFF00FF66);
                            framebuffer::put_pixel(px + cell_size - 1, py + dx, 0xFF00FF66);
                            framebuffer::put_pixel(px + dx, py + cell_size - 1, 0xFF00FF66);
                            framebuffer::put_pixel(px, py + cell_size - 1 - dx, 0xFF00FF66);
                            framebuffer::put_pixel(px + cell_size - 1 - dx, py + cell_size - 1, 0xFF00FF66);
                            framebuffer::put_pixel(px + cell_size - 1, py + cell_size - 1 - dx, 0xFF00FF66);
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
                framebuffer::put_pixel(board_x + i, board_y, GREEN_MUTED);
                framebuffer::put_pixel(board_x + i, board_y + board_size, GREEN_MUTED);
            }
            for i in 0..board_size + 1 {
                framebuffer::put_pixel(board_x, board_y + i, GREEN_MUTED);
                framebuffer::put_pixel(board_x + board_size, board_y + i, GREEN_MUTED);
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
                        framebuffer::put_pixel(screen_x as u32, screen_y as u32, pixel | 0xFF000000);
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
        
        // Colors
        let bg_dark = 0xFF0C1410u32;
        let bg_toolbar = 0xFF0A1A12u32;
        let bg_selected = 0xFF0A3A1Au32;
        let text_file = 0xFF80CC90u32;
        let icon_folder = 0xFFDDAA30u32;
        let icon_file = 0xFF60AA80u32;
        let separator = 0xFF1A2A1Au32;
        
        // ── Toolbar (same as list view) ──
        let toolbar_h = 32u32;
        framebuffer::fill_rect_alpha(safe_x + 2, content_y_start as u32, ww.saturating_sub(4), toolbar_h, 0x0A1A12, 230);
        
        let btn_y = content_y_start + 6;
        let btn_size = 20u32;
        if btn_y > 0 {
            draw_rounded_rect(wx + 8, btn_y, btn_size, btn_size, 3, 0xFF1A2A1A);
            draw_rounded_rect_border(wx + 8, btn_y, btn_size, btn_size, 3, GREEN_GHOST);
            self.draw_text(wx + 13, btn_y + 3, "<", GREEN_SUBTLE);
            draw_rounded_rect(wx + 32, btn_y, btn_size, btn_size, 3, 0xFF1A2A1A);
            draw_rounded_rect_border(wx + 32, btn_y, btn_size, btn_size, 3, GREEN_GHOST);
            self.draw_text(wx + 37, btn_y + 3, "^", GREEN_SUBTLE);
            
            // View mode toggle button [Grid] → highlight active
            let view_btn_x = wx + ww as i32 - 60;
            draw_rounded_rect(view_btn_x, btn_y, 50, btn_size, 3, 0xFF1A3A1A);
            draw_rounded_rect_border(view_btn_x, btn_y, 50, btn_size, 3, GREEN_PRIMARY);
            self.draw_text(view_btn_x + 6, btn_y + 3, "List", GREEN_PRIMARY);
        }
        
        // Path bar
        let path_x = wx + 58;
        let path_w = (ww as i32).saturating_sub(126);
        if path_w > 10 && btn_y > 0 {
            draw_rounded_rect(path_x, btn_y, path_w as u32, btn_size, 4, 0xFF081208);
            draw_rounded_rect_border(path_x, btn_y, path_w as u32, btn_size, 4, GREEN_GHOST);
            let current_path = window.file_path.as_deref().unwrap_or("/");
            self.draw_text_smooth(path_x + 8, btn_y + 4, current_path, GREEN_PRIMARY);
        }
        
        // Toolbar separator
        let grid_start_y = content_y_start as u32 + toolbar_h + 1;
        framebuffer::draw_hline(safe_x + 2, grid_start_y.saturating_sub(1), ww.saturating_sub(4), separator);
        
        // ── Grid area ──
        let grid_area_h = wh.saturating_sub(TITLE_BAR_HEIGHT + toolbar_h + 30);
        if grid_area_h < 8 { return; }
        framebuffer::fill_rect(safe_x + 2, grid_start_y, ww.saturating_sub(4), grid_area_h, bg_dark);
        
        // Grid parameters
        let icon_cell_w = 90u32;
        let icon_cell_h = 80u32;
        let cols = ((ww.saturating_sub(20)) / icon_cell_w).max(1);
        let padding_x = (ww.saturating_sub(cols * icon_cell_w)) / 2;
        
        // Parse file entries
        let file_start_idx = 5usize.min(window.content.len());
        let file_end_idx = if window.content.len() > file_start_idx + 2 { window.content.len() - 2 } else { window.content.len() };
        let file_entries: Vec<&str> = if file_end_idx > file_start_idx {
            window.content[file_start_idx..file_end_idx].iter().map(|s| s.as_str()).collect()
        } else { Vec::new() };
        
        if file_entries.is_empty() {
            self.draw_text_smooth(wx + 40, grid_start_y as i32 + 30, "(empty)", GREEN_GHOST);
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
            
            let cell_x = safe_x + padding_x + col as u32 * icon_cell_w;
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
                // Large file icon
                let fc = if is_selected { 0xFF80DD99 } else { icon_file };
                framebuffer::fill_rect(icon_x, icon_y, 28, 28, fc);
                framebuffer::fill_rect(icon_x + 18, icon_y, 10, 10, 0xFF0A140A);
                framebuffer::fill_rect(icon_x + 18, icon_y, 2, 10, fc);
                framebuffer::fill_rect(icon_x + 18, icon_y + 8, 10, 2, fc);
                framebuffer::fill_rect(icon_x + 3, icon_y + 12, 22, 14, 0xFF040A04);
                // File type hint
                let ext = Self::extract_name_from_entry(entry);
                let ext_label = if ext.ends_with(".rs") { "RS" }
                    else if ext.ends_with(".txt") { "TXT" }
                    else if ext.ends_with(".bmp") { "BMP" }
                    else if ext.ends_with(".sh") { "SH" }
                    else if ext.ends_with(".toml") { "TML" }
                    else { "" };
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
        framebuffer::fill_rect(safe_x + 2, status_y, ww.saturating_sub(4), 20, bg_toolbar);
        framebuffer::draw_hline(safe_x + 2, status_y, ww.saturating_sub(4), separator);
        let item_count = file_entries.len();
        let status_text = alloc::format!("{} items | V:toggle view | Ctrl+C/X/V:clipboard", item_count);
        self.draw_text_smooth(wx + 10, status_y as i32 + 4, &status_text, GREEN_SUBTLE);
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
        
        let content_y_start = wy + TITLE_BAR_HEIGHT as i32;
        let toolbar_h = 32i32;
        
        // Check toolbar buttons
        let btn_y = content_y_start + 6;
        let btn_size = 20i32;
        
        // Back button
        if x >= wx + 8 && x < wx + 8 + btn_size && y >= btn_y && y < btn_y + btn_size {
            self.navigate_file_manager("..");
            return;
        }
        // Up button
        if x >= wx + 32 && x < wx + 32 + btn_size && y >= btn_y && y < btn_y + btn_size {
            self.navigate_file_manager("..");
            return;
        }
        
        // View mode toggle button (top-right)
        let view_btn_x = wx + ww as i32 - 60;
        if x >= view_btn_x && x < view_btn_x + 50 && y >= btn_y && y < btn_y + btn_size {
            let current = self.fm_view_modes.get(&window_id).copied().unwrap_or(FileManagerViewMode::List);
            let new_mode = match current {
                FileManagerViewMode::List => FileManagerViewMode::IconGrid,
                FileManagerViewMode::IconGrid => FileManagerViewMode::List,
            };
            self.fm_view_modes.insert(window_id, new_mode);
            crate::serial_println!("[FM] Toggled view mode");
            return;
        }
        
        // Click on file in content area
        let is_grid = self.fm_view_modes.get(&window_id).copied().unwrap_or(FileManagerViewMode::List) == FileManagerViewMode::IconGrid;
        let file_start_idx = 5usize.min(content_len);
        let file_end_idx = if content_len > file_start_idx + 2 { content_len - 2 } else { content_len };
        let file_count = file_end_idx.saturating_sub(file_start_idx);
        
        if is_grid {
            // Grid view click
            let grid_start_y = content_y_start + toolbar_h + 1;
            let icon_cell_w = 90i32;
            let icon_cell_h = 80i32;
            let cols = ((ww as i32 - 20) / icon_cell_w).max(1);
            let padding_x = (ww as i32 - cols * icon_cell_w) / 2;
            
            let rel_x = x - wx - padding_x;
            let rel_y = y - grid_start_y;
            if rel_x >= 0 && rel_y >= 0 {
                let col = rel_x / icon_cell_w;
                let row = rel_y / icon_cell_h;
                let idx = row * cols + col;
                if idx >= 0 && (idx as usize) < file_count {
                    let click_idx = idx as usize;
                    // Double-click to open
                    if click_idx == selected_idx && crate::mouse::is_double_click() {
                        // Open the file/dir
                        self.open_selected_file_at(window_id, click_idx);
                        return;
                    }
                    // Single click — select
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == window_id) {
                        w.selected_index = click_idx;
                    }
                }
            }
        } else {
            // List view click
            let header_y = content_y_start + toolbar_h;
            let col_header_h = 22i32;
            let list_start_y = header_y + col_header_h + 1;
            let row_h = 24i32;
            
            let rel_y = y - list_start_y;
            if rel_y >= 0 {
                let scroll_offset = self.windows.iter().find(|w| w.id == window_id).map(|w| w.scroll_offset).unwrap_or(0);
                let click_idx = scroll_offset + (rel_y / row_h) as usize;
                if click_idx < file_count {
                    // Double-click to open
                    if click_idx == selected_idx && crate::mouse::is_double_click() {
                        self.open_selected_file_at(window_id, click_idx);
                        return;
                    }
                    // Single click — select
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == window_id) {
                        w.selected_index = click_idx;
                    }
                }
            }
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
                    framebuffer::put_pixel(px, py, wifi_color);
                    // Mirror to left
                    if cx >= dx {
                        framebuffer::put_pixel(cx - dx, py, wifi_color);
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

    /// Draw Windows Explorer-style file manager
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
        
        // Guard against too-small windows
        if ww < 80 || wh < 100 {
            return;
        }
        
        let content_y_start = wy + TITLE_BAR_HEIGHT as i32;
        
        // ── Colors ──
        let bg_dark = 0xFF0C1410u32;
        let bg_toolbar = 0xFF0A1A12u32;
        let bg_header = 0xFF0E1E14u32;
        let bg_row_even = 0xFF0A140Fu32;
        let bg_row_odd = 0xFF0C180Fu32;
        let bg_selected = 0xFF0A3A1Au32;
        let text_header = 0xFF60CC80u32;
        let text_file = 0xFF80CC90u32;
        let text_path = GREEN_PRIMARY;
        let icon_folder = 0xFFDDAA30u32;
        let icon_file = 0xFF60AA80u32;
        let separator = 0xFF1A2A1Au32;
        
        // Safe u32 coordinate helpers
        let safe_x = if wx < 0 { 0u32 } else { wx as u32 };
        let safe_y = if wy < 0 { 0u32 } else { wy as u32 };
        
        // ── Toolbar area (path bar + nav buttons) ──
        let toolbar_h = 32u32;
        framebuffer::fill_rect_alpha(safe_x + 2, content_y_start as u32, ww.saturating_sub(4), toolbar_h, 0x0A1A12, 230);
        
        // Back button [<]
        let btn_y = content_y_start + 6;
        let btn_size = 20u32;
        if btn_y > 0 {
            draw_rounded_rect(wx + 8, btn_y, btn_size, btn_size, 3, 0xFF1A2A1A);
            draw_rounded_rect_border(wx + 8, btn_y, btn_size, btn_size, 3, GREEN_GHOST);
            self.draw_text(wx + 13, btn_y + 3, "<", GREEN_SUBTLE);
            
            // Up button [^]
            draw_rounded_rect(wx + 32, btn_y, btn_size, btn_size, 3, 0xFF1A2A1A);
            draw_rounded_rect_border(wx + 32, btn_y, btn_size, btn_size, 3, GREEN_GHOST);
            self.draw_text(wx + 37, btn_y + 3, "^", GREEN_SUBTLE);
            
            // View mode toggle button [Grid]
            let view_btn_x = wx + ww as i32 - 60;
            draw_rounded_rect(view_btn_x, btn_y, 50, btn_size, 3, 0xFF1A2A1A);
            draw_rounded_rect_border(view_btn_x, btn_y, 50, btn_size, 3, GREEN_GHOST);
            self.draw_text(view_btn_x + 6, btn_y + 3, "Grid", GREEN_SUBTLE);
        }
        
        // Path bar (breadcrumb style)
        let path_x = wx + 58;
        let path_w = (ww as i32).saturating_sub(126);
        if path_w > 10 && btn_y > 0 {
            draw_rounded_rect(path_x, btn_y, path_w as u32, btn_size, 4, 0xFF081208);
            draw_rounded_rect_border(path_x, btn_y, path_w as u32, btn_size, 4, GREEN_GHOST);
            let current_path = window.file_path.as_deref().unwrap_or("/");
            let mut px = path_x + 8;
            let parts: Vec<&str> = current_path.split('/').filter(|s| !s.is_empty()).collect();
            if parts.is_empty() {
                self.draw_text_smooth(px, btn_y + 4, "/", text_path);
            } else {
                self.draw_text_smooth(px, btn_y + 4, "/", GREEN_GHOST);
                px += 10;
                for (i, part) in parts.iter().enumerate() {
                    if px > path_x + path_w - 16 { break; } // don't overflow path bar
                    let is_last = i == parts.len() - 1;
                    let color = if is_last { text_path } else { GREEN_SUBTLE };
                    self.draw_text_smooth(px, btn_y + 4, part, color);
                    px += (part.len() as i32) * 8 + 4;
                    if !is_last {
                        self.draw_text_smooth(px, btn_y + 4, ">", GREEN_GHOST);
                        px += 12;
                    }
                }
            }
        }
        
        // Toolbar bottom separator
        let header_y = content_y_start as u32 + toolbar_h;
        framebuffer::draw_hline(safe_x + 2, header_y, ww.saturating_sub(4), separator);
        
        // ── Column headers ──
        let col_header_h = 22u32;
        framebuffer::fill_rect(safe_x + 2, header_y + 1, ww.saturating_sub(4), col_header_h, bg_header);
        
        let col_name_x = wx + 34;
        let col_type_x = wx + (ww as i32 * 55 / 100);
        let col_size_x = wx + (ww as i32 * 72 / 100);
        let col_prog_x = wx + (ww as i32 * 85 / 100);
        
        let hy = (header_y + 4) as i32;
        self.draw_text_smooth(col_name_x, hy, "Name", text_header);
        if ww > 200 {
            self.draw_text_smooth(col_type_x, hy, "Type", text_header);
        }
        if ww > 280 {
            self.draw_text_smooth(col_size_x, hy, "Size", text_header);
        }
        if ww > 360 {
            self.draw_text_smooth(col_prog_x, hy, "Open with", text_header);
        }
        
        // Header bottom separator + column separators
        let list_start_y = header_y + col_header_h + 1;
        framebuffer::draw_hline(safe_x + 2, list_start_y.saturating_sub(1), ww.saturating_sub(4), separator);
        if ww > 200 && col_type_x > 4 {
            framebuffer::fill_rect(col_type_x as u32 - 4, header_y + 1, 1, col_header_h, separator);
        }
        if ww > 280 && col_size_x > 4 {
            framebuffer::fill_rect(col_size_x as u32 - 4, header_y + 1, 1, col_header_h, separator);
        }
        if ww > 360 && col_prog_x > 4 {
            framebuffer::fill_rect(col_prog_x as u32 - 4, header_y + 1, 1, col_header_h, separator);
        }
        
        // ── File list area ──
        let list_area_h = wh.saturating_sub(TITLE_BAR_HEIGHT + toolbar_h + col_header_h + 30);
        if list_area_h < 8 { return; }
        framebuffer::fill_rect(safe_x + 2, list_start_y, ww.saturating_sub(4), list_area_h, bg_dark);
        
        let row_h = 24u32;
        let max_visible = (list_area_h / row_h).max(1) as usize;
        
        // Parse file entries from content (skip header lines, last 2 footer)
        // Guard: ensure content has enough lines
        let file_start_idx = 5usize.min(window.content.len());
        let file_end_idx = if window.content.len() > file_start_idx + 2 { 
            window.content.len() - 2 
        } else { 
            window.content.len() 
        };
        
        let file_entries: Vec<&str> = if file_end_idx > file_start_idx {
            window.content[file_start_idx..file_end_idx]
                .iter()
                .map(|s| s.as_str())
                .collect()
        } else {
            Vec::new()
        };
        
        if file_entries.is_empty() {
            // Show empty directory message
            self.draw_text_smooth(wx + 40, list_start_y as i32 + 20, "(empty)", GREEN_GHOST);
        }
        
        let scroll = window.scroll_offset;
        let visible_count = file_entries.len().min(max_visible);
        
        for vi in 0..visible_count {
            let entry_idx = scroll + vi;
            if entry_idx >= file_entries.len() { break; }
            let line = file_entries[entry_idx];
            let ry = list_start_y + (vi as u32) * row_h;
            if ry + row_h > list_start_y + list_area_h { break; }
            
            let is_selected = entry_idx == window.selected_index;
            let is_dir = line.contains("[D]");
            
            // Row background
            let row_bg = if is_selected {
                bg_selected
            } else if vi % 2 == 0 {
                bg_row_even
            } else {
                bg_row_odd
            };
            framebuffer::fill_rect(safe_x + 2, ry, ww.saturating_sub(4), row_h, row_bg);
            
            // Selection indicator
            if is_selected {
                framebuffer::fill_rect(safe_x + 2, ry, 3, row_h, GREEN_PRIMARY);
                draw_rounded_rect_border(wx + 2, ry as i32, ww.saturating_sub(4), row_h, 2, 0xFF1A5A2A);
            }
            
            let text_y = (ry + 5) as i32;
            let row_text_color = if is_selected { GREEN_PRIMARY } else { text_file };
            
            // ── Draw folder/file icon ──
            let ix = safe_x + 10;
            let iy = ry + 3;
            if is_dir {
                let fc = if is_selected { 0xFFEEBB40 } else { icon_folder };
                framebuffer::fill_rect(ix, iy, 8, 3, fc);
                framebuffer::fill_rect(ix, iy + 3, 16, 11, fc);
                framebuffer::fill_rect(ix + 1, iy + 5, 14, 7, 0xFF0A0A04);
                framebuffer::fill_rect(ix + 3, iy + 7, 8, 1, 0xFF302A10);
                framebuffer::fill_rect(ix + 3, iy + 9, 6, 1, 0xFF302A10);
            } else {
                let fc = if is_selected { 0xFF80DD99 } else { icon_file };
                framebuffer::fill_rect(ix, iy, 14, 16, fc);
                framebuffer::fill_rect(ix + 9, iy, 5, 5, 0xFF0A140A);
                framebuffer::fill_rect(ix + 9, iy, 1, 5, fc);
                framebuffer::fill_rect(ix + 9, iy + 4, 5, 1, fc);
                framebuffer::fill_rect(ix + 2, iy + 6, 10, 8, 0xFF040A04);
                framebuffer::fill_rect(ix + 3, iy + 7, 8, 1, 0xFF203020);
                framebuffer::fill_rect(ix + 3, iy + 9, 6, 1, 0xFF203020);
                framebuffer::fill_rect(ix + 3, iy + 11, 7, 1, 0xFF203020);
            }
            
            // ── Parse entry fields ──
            let trimmed = line.trim();
            let name_str;
            let type_str;
            let size_str;
            let prog_str;
            
            if let Some(bracket_end) = trimmed.find(']') {
                let after_icon = if bracket_end + 1 < trimmed.len() { &trimmed[bracket_end + 1..] } else { "" };
                let parts: Vec<&str> = after_icon.split_whitespace().collect();
                name_str = if !parts.is_empty() { parts[0] } else { "???" };
                type_str = if parts.len() > 1 { parts[1] } else { "" };
                size_str = if parts.len() > 2 { parts[2] } else { "" };
                prog_str = if parts.len() > 3 { parts[3] } else { "" };
            } else {
                name_str = trimmed;
                type_str = "";
                size_str = "";
                prog_str = "";
            }
            
            // Draw columns
            self.draw_text_smooth(col_name_x, text_y, name_str, row_text_color);
            if ww > 200 {
                let type_color = if is_dir { 0xFF55AA70 } else { 0xFF558866 };
                self.draw_text_smooth(col_type_x, text_y, type_str, if is_selected { GREEN_PRIMARY } else { type_color });
            }
            if ww > 280 {
                self.draw_text_smooth(col_size_x, text_y, size_str, if is_selected { GREEN_SUBTLE } else { 0xFF558866 });
            }
            if ww > 360 {
                self.draw_text_smooth(col_prog_x, text_y, prog_str, if is_selected { GREEN_SUBTLE } else { 0xFF446655 });
            }
            
            // Row bottom separator
            framebuffer::draw_hline(safe_x + 6, ry + row_h - 1, ww.saturating_sub(12), 0xFF0E1A12);
        }
        
        // ── Scrollbar ──
        if file_entries.len() > max_visible && list_area_h > 20 {
            let sb_w = 6u32;
            let sb_x = safe_x + ww.saturating_sub(sb_w + 4);
            framebuffer::fill_rect(sb_x, list_start_y + 2, sb_w, list_area_h.saturating_sub(4), 0xFF0A1A0F);
            let total = file_entries.len() as u32;
            let visible = max_visible as u32;
            let track_h = list_area_h.saturating_sub(4);
            let thumb_h = ((visible * track_h) / total.max(1)).max(16).min(track_h);
            let max_scroll = total.saturating_sub(visible);
            let thumb_y = if max_scroll > 0 {
                list_start_y + 2 + ((scroll as u32 * track_h.saturating_sub(thumb_h)) / max_scroll)
            } else {
                list_start_y + 2
            };
            draw_rounded_rect(sb_x as i32, thumb_y as i32, sb_w, thumb_h, 2, GREEN_MUTED);
        }
        
        // ── Status bar at bottom ──
        let status_y = safe_y + wh.saturating_sub(24);
        framebuffer::fill_rect(safe_x + 2, status_y, ww.saturating_sub(4), 20, bg_toolbar);
        framebuffer::draw_hline(safe_x + 2, status_y, ww.saturating_sub(4), separator);
        
        let item_count = file_entries.len();
        let status_text = if item_count == 1 {
            String::from("1 item")
        } else {
            alloc::format!("{} items", item_count)
        };
        self.draw_text_smooth(wx + 10, status_y as i32 + 4, &status_text, GREEN_SUBTLE);
        if ww > 280 {
            self.draw_text_smooth(wx + ww as i32 - 300, status_y as i32 + 4, "V:View Ctrl+C/X/V:Clip Enter:Open Bksp:Up", GREEN_GHOST);
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
        
        // Expression indicator (show expression in small text above result)
        if let Some(calc) = self.calculator_states.get(&window.id) {
            if calc.just_evaluated && !calc.expression.is_empty() {
                // Show a small "=" indicator
                self.draw_text(cx as i32 + 10, cy as i32 + 12, "=", colors::ACCENT);
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
                        framebuffer::put_pixel(px, py, shadow_color);
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
                            framebuffer::put_pixel(px, py, color);
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
                framebuffer::put_pixel(px, py, GREEN_PRIMARY);
                if py > 0 { framebuffer::put_pixel(px, py - 1, GREEN_MUTED); }
                if py + 1 < self.height { framebuffer::put_pixel(px, py + 1, GREEN_MUTED); }
            }
        }
        // Right arrow  
        for i in 0..7i32 {
            let px = (mx + 1 + i) as u32;
            let py = my as u32;
            if px < self.width && py < self.height {
                framebuffer::put_pixel(px, py, GREEN_PRIMARY);
                if py > 0 { framebuffer::put_pixel(px, py - 1, GREEN_MUTED); }
                if py + 1 < self.height { framebuffer::put_pixel(px, py + 1, GREEN_MUTED); }
            }
        }
        // Left arrowhead
        for d in 1..=4i32 {
            let px = (mx - 7 + d) as u32;
            if px < self.width {
                if (my - d) >= 0 { framebuffer::put_pixel(px, (my - d) as u32, GREEN_PRIMARY); }
                if (my + d) < self.height as i32 { framebuffer::put_pixel(px, (my + d) as u32, GREEN_PRIMARY); }
            }
        }
        // Right arrowhead
        for d in 1..=4i32 {
            let px = (mx + 7 - d) as u32;
            if px < self.width {
                if (my - d) >= 0 { framebuffer::put_pixel(px, (my - d) as u32, GREEN_PRIMARY); }
                if (my + d) < self.height as i32 { framebuffer::put_pixel(px, (my + d) as u32, GREEN_PRIMARY); }
            }
        }
        // Center dot
        if mx >= 0 && my >= 0 && (mx as u32) < self.width && (my as u32) < self.height {
            framebuffer::put_pixel(mx as u32, my as u32, 0xFFFFFFFF);
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
                framebuffer::put_pixel(px, py, GREEN_PRIMARY);
                if px > 0 { framebuffer::put_pixel(px - 1, py, GREEN_MUTED); }
                if px + 1 < self.width { framebuffer::put_pixel(px + 1, py, GREEN_MUTED); }
            }
        }
        for i in 0..7i32 {
            let px = mx as u32;
            let py = (my + 1 + i) as u32;
            if px < self.width && py < self.height {
                framebuffer::put_pixel(px, py, GREEN_PRIMARY);
                if px > 0 { framebuffer::put_pixel(px - 1, py, GREEN_MUTED); }
                if px + 1 < self.width { framebuffer::put_pixel(px + 1, py, GREEN_MUTED); }
            }
        }
        // Top arrowhead
        for d in 1..=4i32 {
            let py = (my - 7 + d) as u32;
            if py < self.height {
                if (mx - d) >= 0 { framebuffer::put_pixel((mx - d) as u32, py, GREEN_PRIMARY); }
                if (mx + d) < self.width as i32 { framebuffer::put_pixel((mx + d) as u32, py, GREEN_PRIMARY); }
            }
        }
        // Bottom arrowhead
        for d in 1..=4i32 {
            let py = (my + 7 - d) as u32;
            if py < self.height {
                if (mx - d) >= 0 { framebuffer::put_pixel((mx - d) as u32, py, GREEN_PRIMARY); }
                if (mx + d) < self.width as i32 { framebuffer::put_pixel((mx + d) as u32, py, GREEN_PRIMARY); }
            }
        }
        if mx >= 0 && my >= 0 && (mx as u32) < self.width && (my as u32) < self.height {
            framebuffer::put_pixel(mx as u32, my as u32, 0xFFFFFFFF);
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
                framebuffer::put_pixel(px, py, GREEN_PRIMARY);
                if px + 1 < self.width { framebuffer::put_pixel(px + 1, py, GREEN_MUTED); }
                if py + 1 < self.height { framebuffer::put_pixel(px, py + 1, GREEN_MUTED); }
            }
        }
        // NW arrowhead
        for d in 1..=3i32 {
            let bx = mx - 6 + d;
            let by = my - 6;
            if bx >= 0 && (by as u32) < self.height { framebuffer::put_pixel(bx as u32, by as u32, GREEN_PRIMARY); }
            let bx2 = mx - 6;
            let by2 = my - 6 + d;
            if bx2 >= 0 && by2 >= 0 { framebuffer::put_pixel(bx2 as u32, by2 as u32, GREEN_PRIMARY); }
        }
        // SE arrowhead
        for d in 1..=3i32 {
            let bx = mx + 6 - d;
            let by = my + 6;
            if (bx as u32) < self.width && (by as u32) < self.height { framebuffer::put_pixel(bx as u32, by as u32, GREEN_PRIMARY); }
            let bx2 = mx + 6;
            let by2 = my + 6 - d;
            if (bx2 as u32) < self.width && (by2 as u32) < self.height { framebuffer::put_pixel(bx2 as u32, by2 as u32, GREEN_PRIMARY); }
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
                framebuffer::put_pixel(px, py, GREEN_PRIMARY);
                if px > 0 { framebuffer::put_pixel(px - 1, py, GREEN_MUTED); }
                if py + 1 < self.height { framebuffer::put_pixel(px, py + 1, GREEN_MUTED); }
            }
        }
        // NE arrowhead
        for d in 1..=3i32 {
            let bx = mx + 6 - d;
            let by = my - 6;
            if (bx as u32) < self.width && by >= 0 { framebuffer::put_pixel(bx as u32, by as u32, GREEN_PRIMARY); }
            let bx2 = mx + 6;
            let by2 = my - 6 + d;
            if (bx2 as u32) < self.width && by2 >= 0 { framebuffer::put_pixel(bx2 as u32, by2 as u32, GREEN_PRIMARY); }
        }
        // SW arrowhead
        for d in 1..=3i32 {
            let bx = mx - 6 + d;
            let by = my + 6;
            if bx >= 0 && (by as u32) < self.height { framebuffer::put_pixel(bx as u32, by as u32, GREEN_PRIMARY); }
            let bx2 = mx - 6;
            let by2 = my + 6 - d;
            if bx2 >= 0 && (by2 as u32) < self.height { framebuffer::put_pixel(bx2 as u32, by2 as u32, GREEN_PRIMARY); }
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
                                    framebuffer::put_pixel(px, py, color);
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
                                        let bg = framebuffer::get_pixel(px, py);
                                        let bg_r = (bg >> 16) & 0xFF;
                                        let bg_g = (bg >> 8) & 0xFF;
                                        let bg_b = bg & 0xFF;
                                        let r = (fg_r * alpha + bg_r * inv) / 255;
                                        let g = (fg_g * alpha + bg_g * inv) / 255;
                                        let b = (fg_b * alpha + bg_b * inv) / 255;
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
                DESKTOP.lock().create_window("Files", 150, 100, 400, 350, WindowType::FileManager);
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
    crate::arch::timestamp()
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
