//! TrustOS GUI Engine - Windows-like Desktop Experience
//!
//! Optimized GUI with:
//! - True 60 FPS with HLT-based frame limiting (low CPU)
//! - Window snapping (Win+Arrow)
//! - Alt+Tab window switcher
//! - Start Menu
//! - Toast notifications
//! - Keyboard shortcuts
//! - Contextual cursors

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicI32, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════════
// FRAME TIMING - True 60 FPS with CPU sleep
// ═══════════════════════════════════════════════════════════════════════════════

/// Target frame time in microseconds (16.666ms = 60 FPS)
const TARGET_FRAME_US: u64 = 16_666;

/// TSC frequency in Hz (set during init)
static TSC_FREQ_HZ: AtomicU64 = AtomicU64::new(3_000_000_000); // Default 3 GHz

/// Frame counter for FPS calculation
static FRAME_COUNT: AtomicU64 = AtomicU64::new(0);
static LAST_FPS_TIME: AtomicU64 = AtomicU64::new(0);
static CURRENT_FPS: AtomicU64 = AtomicU64::new(0);

/// Initialize frame timing with TSC frequency
pub fn init_timing() {
    // Get TSC frequency from CPU module
    let freq = crate::cpu::tsc_frequency();
    TSC_FREQ_HZ.store(freq, Ordering::SeqCst);
    crate::serial_println!("[GUI] Frame timing init: TSC {} MHz", freq / 1_000_000);
}

/// Convert TSC ticks to microseconds
#[inline]
fn tsc_to_us(ticks: u64) -> u64 {
    let freq = TSC_FREQ_HZ.load(Ordering::Relaxed);
    if freq == 0 { return 0; }
    (ticks * 1_000_000) / freq
}

/// Get current time in microseconds (from TSC)
#[inline]
pub fn now_us() -> u64 {
    tsc_to_us(read_tsc())
}

/// Read TSC
#[inline]
fn read_tsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

/// Sleep until next frame (uses HLT to save CPU)
pub fn wait_for_next_frame(frame_start_us: u64) {
    let elapsed = now_us().saturating_sub(frame_start_us);
    
    if elapsed < TARGET_FRAME_US {
        let wait_us = TARGET_FRAME_US - elapsed;
        
        // Use HLT for long waits (>1ms), spin for short waits
        if wait_us > 1000 {
            // HLT will wake on next interrupt (timer, keyboard, mouse)
            // This drops CPU from 100% to ~1%
            unsafe {
                // Enable interrupts and halt until interrupt
                core::arch::asm!("sti; hlt", options(nomem, nostack));
            }
        } else {
            // Short spin for precise timing
            let target = frame_start_us + TARGET_FRAME_US;
            while now_us() < target {
                core::hint::spin_loop();
            }
        }
    }
    
    // Update FPS counter
    let count = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
    let now = now_us();
    let last = LAST_FPS_TIME.load(Ordering::Relaxed);
    if now - last >= 1_000_000 {
        CURRENT_FPS.store(count, Ordering::Relaxed);
        FRAME_COUNT.store(0, Ordering::Relaxed);
        LAST_FPS_TIME.store(now, Ordering::Relaxed);
    }
}

/// Get current FPS
pub fn get_fps() -> u64 {
    CURRENT_FPS.load(Ordering::Relaxed)
}

// ═══════════════════════════════════════════════════════════════════════════════
// KEYBOARD SHORTCUTS - Windows-like hotkeys
// ═══════════════════════════════════════════════════════════════════════════════

/// Modifier keys state
static MOD_ALT: AtomicBool = AtomicBool::new(false);
static MOD_CTRL: AtomicBool = AtomicBool::new(false);
static MOD_SHIFT: AtomicBool = AtomicBool::new(false);
static MOD_WIN: AtomicBool = AtomicBool::new(false);

/// Hotkey action
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HotkeyAction {
    None,
    // Window management
    CloseWindow,      // Alt+F4
    SwitchWindow,     // Alt+Tab
    SnapLeft,         // Win+Left
    SnapRight,        // Win+Right
    Maximize,         // Win+Up
    Minimize,         // Win+Down
    ShowDesktop,      // Win+D
    // Apps
    OpenFileManager,  // Win+E
    OpenTerminal,     // Win+T or Ctrl+Alt+T
    OpenRun,          // Win+R
    // System
    LockScreen,       // Win+L
    OpenStartMenu,    // Win key alone
    Screenshot,       // PrintScreen
    ToggleDevPanel,   // F12
}

/// Scancode constants
pub mod scancode {
    pub const ALT: u8 = 0x38;
    pub const CTRL: u8 = 0x1D;
    pub const SHIFT: u8 = 0x2A;
    pub const WIN: u8 = 0x5B;  // Left Win
    pub const TAB: u8 = 0x0F;
    pub const F4: u8 = 0x3E;
    pub const LEFT: u8 = 0x4B;
    pub const RIGHT: u8 = 0x4D;
    pub const UP: u8 = 0x48;
    pub const DOWN: u8 = 0x50;
    pub const D: u8 = 0x20;
    pub const E: u8 = 0x12;
    pub const T: u8 = 0x14;
    pub const R: u8 = 0x13;
    pub const L: u8 = 0x26;
    pub const ESC: u8 = 0x01;
    pub const F12: u8 = 0x58;
}

/// Update modifier state from scancode
pub fn update_modifiers(scancode: u8, pressed: bool) {
    match scancode {
        scancode::ALT => MOD_ALT.store(pressed, Ordering::Relaxed),
        scancode::CTRL => MOD_CTRL.store(pressed, Ordering::Relaxed),
        scancode::SHIFT => MOD_SHIFT.store(pressed, Ordering::Relaxed),
        scancode::WIN => MOD_WIN.store(pressed, Ordering::Relaxed),
        _ => {}
    }
}

/// Check for hotkey and return action
pub fn check_hotkey(scancode: u8) -> HotkeyAction {
    let alt = MOD_ALT.load(Ordering::Relaxed);
    let ctrl = MOD_CTRL.load(Ordering::Relaxed);
    let win = MOD_WIN.load(Ordering::Relaxed);
    
    // Alt+F4 = Close window
    if alt && scancode == scancode::F4 {
        return HotkeyAction::CloseWindow;
    }
    
    // Alt+Tab = Switch window
    if alt && scancode == scancode::TAB {
        return HotkeyAction::SwitchWindow;
    }
    
    // Win+Arrow keys = Snap
    if win {
        match scancode {
            scancode::LEFT => return HotkeyAction::SnapLeft,
            scancode::RIGHT => return HotkeyAction::SnapRight,
            scancode::UP => return HotkeyAction::Maximize,
            scancode::DOWN => return HotkeyAction::Minimize,
            scancode::D => return HotkeyAction::ShowDesktop,
            scancode::E => return HotkeyAction::OpenFileManager,
            scancode::T => return HotkeyAction::OpenTerminal,
            scancode::R => return HotkeyAction::OpenRun,
            scancode::L => return HotkeyAction::LockScreen,
            _ => {}
        }
    }
    
    // Ctrl+Alt+T = Terminal (Linux style)
    if ctrl && alt && scancode == scancode::T {
        return HotkeyAction::OpenTerminal;
    }
    
    // F12 = Toggle DevPanel overlay
    if scancode == scancode::F12 {
        return HotkeyAction::ToggleDevPanel;
    }
    
    HotkeyAction::None
}

/// Check if Win key was tapped (pressed and released alone)
static WIN_PRESSED_ALONE: AtomicBool = AtomicBool::new(false);

pub fn check_start_menu_trigger(scancode: u8, pressed: bool) -> bool {
    if scancode == scancode::WIN {
        if pressed {
            WIN_PRESSED_ALONE.store(true, Ordering::Relaxed);
        } else {
            // Win released - check if it was pressed alone
            if WIN_PRESSED_ALONE.load(Ordering::Relaxed) {
                WIN_PRESSED_ALONE.store(false, Ordering::Relaxed);
                return true; // Trigger start menu
            }
        }
    } else if pressed {
        // Any other key pressed while Win is down = not alone
        WIN_PRESSED_ALONE.store(false, Ordering::Relaxed);
    }
    false
}

// ═══════════════════════════════════════════════════════════════════════════════
// WINDOW SNAPPING - Win+Arrow
// ═══════════════════════════════════════════════════════════════════════════════

/// Snap position for window
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SnapPosition {
    None,
    Left,       // 50% left
    Right,      // 50% right
    Maximized,  // Full screen
    TopLeft,    // 25% corner
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Calculate snapped window bounds
pub fn calculate_snap_bounds(
    snap: SnapPosition,
    screen_w: u32,
    screen_h: u32,
    taskbar_h: u32,
) -> (i32, i32, u32, u32) {
    let work_h = screen_h - taskbar_h;
    
    match snap {
        SnapPosition::Left => (0, 0, screen_w / 2, work_h),
        SnapPosition::Right => ((screen_w / 2) as i32, 0, screen_w / 2, work_h),
        SnapPosition::Maximized => (0, 0, screen_w, work_h),
        SnapPosition::TopLeft => (0, 0, screen_w / 2, work_h / 2),
        SnapPosition::TopRight => ((screen_w / 2) as i32, 0, screen_w / 2, work_h / 2),
        SnapPosition::BottomLeft => (0, (work_h / 2) as i32, screen_w / 2, work_h / 2),
        SnapPosition::BottomRight => ((screen_w / 2) as i32, (work_h / 2) as i32, screen_w / 2, work_h / 2),
        SnapPosition::None => (0, 0, 400, 300), // Default
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ALT+TAB WINDOW SWITCHER
// ═══════════════════════════════════════════════════════════════════════════════

/// Alt+Tab state
static ALT_TAB_ACTIVE: AtomicBool = AtomicBool::new(false);
static ALT_TAB_INDEX: AtomicI32 = AtomicI32::new(0);

/// Start Alt+Tab mode
pub fn start_alt_tab() {
    ALT_TAB_ACTIVE.store(true, Ordering::Relaxed);
    ALT_TAB_INDEX.store(0, Ordering::Relaxed);
}

/// Cycle to next window in Alt+Tab
pub fn alt_tab_next() {
    ALT_TAB_INDEX.fetch_add(1, Ordering::Relaxed);
}

/// Cycle to previous window in Alt+Tab (Shift+Tab)
pub fn alt_tab_prev() {
    ALT_TAB_INDEX.fetch_sub(1, Ordering::Relaxed);
}

/// Finish Alt+Tab (select current)
pub fn finish_alt_tab() -> i32 {
    ALT_TAB_ACTIVE.store(false, Ordering::Relaxed);
    ALT_TAB_INDEX.load(Ordering::Relaxed)
}

/// Check if Alt+Tab is active
pub fn is_alt_tab_active() -> bool {
    ALT_TAB_ACTIVE.load(Ordering::Relaxed)
}

/// Get current Alt+Tab selection index
pub fn alt_tab_selection() -> i32 {
    ALT_TAB_INDEX.load(Ordering::Relaxed)
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTEXTUAL CURSORS
// ═══════════════════════════════════════════════════════════════════════════════

/// Cursor type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CursorType {
    Arrow,
    Hand,           // Clickable
    Text,           // I-beam for text
    ResizeNS,       // North-South resize
    ResizeEW,       // East-West resize
    ResizeNWSE,     // Diagonal NW-SE
    ResizeNESW,     // Diagonal NE-SW
    Move,           // Move/drag
    Wait,           // Hourglass/spinner
    Crosshair,      // Precision select
}

static CURRENT_CURSOR: Mutex<CursorType> = Mutex::new(CursorType::Arrow);

/// Set current cursor type
pub fn set_cursor(cursor: CursorType) {
    *CURRENT_CURSOR.lock() = cursor;
}

/// Get current cursor type
pub fn get_cursor() -> CursorType {
    *CURRENT_CURSOR.lock()
}

/// Cursor bitmaps (8x8 simple cursors)
pub fn get_cursor_bitmap(cursor: CursorType) -> &'static [u8; 64] {
    match cursor {
        CursorType::Arrow => &CURSOR_ARROW,
        CursorType::Hand => &CURSOR_HAND,
        CursorType::Text => &CURSOR_TEXT,
        CursorType::ResizeNS => &CURSOR_RESIZE_NS,
        CursorType::ResizeEW => &CURSOR_RESIZE_EW,
        CursorType::ResizeNWSE => &CURSOR_RESIZE_NWSE,
        CursorType::ResizeNESW => &CURSOR_RESIZE_NESW,
        CursorType::Move => &CURSOR_MOVE,
        CursorType::Wait => &CURSOR_WAIT,
        CursorType::Crosshair => &CURSOR_CROSSHAIR,
    }
}

// Cursor bitmaps (1 = white, 2 = black outline, 0 = transparent)
static CURSOR_ARROW: [u8; 64] = [
    2,0,0,0,0,0,0,0,
    2,2,0,0,0,0,0,0,
    2,1,2,0,0,0,0,0,
    2,1,1,2,0,0,0,0,
    2,1,1,1,2,0,0,0,
    2,1,1,1,1,2,0,0,
    2,1,1,2,2,0,0,0,
    2,2,2,0,0,0,0,0,
];

static CURSOR_HAND: [u8; 64] = [
    0,0,2,2,0,0,0,0,
    0,2,1,1,2,0,0,0,
    0,2,1,1,2,0,0,0,
    0,2,1,1,2,2,2,0,
    2,2,1,1,1,1,1,2,
    2,1,1,1,1,1,1,2,
    2,1,1,1,1,1,1,2,
    0,2,2,2,2,2,2,0,
];

static CURSOR_TEXT: [u8; 64] = [
    0,2,2,2,2,2,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,2,2,2,2,2,0,0,
];

static CURSOR_RESIZE_NS: [u8; 64] = [
    0,0,0,2,0,0,0,0,
    0,0,2,1,2,0,0,0,
    0,2,1,1,1,2,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,2,1,1,1,2,0,0,
    0,0,2,1,2,0,0,0,
    0,0,0,2,0,0,0,0,
];

static CURSOR_RESIZE_EW: [u8; 64] = [
    0,0,0,0,0,0,0,0,
    0,0,2,0,0,2,0,0,
    0,2,1,2,2,1,2,0,
    2,1,1,1,1,1,1,2,
    0,2,1,2,2,1,2,0,
    0,0,2,0,0,2,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
];

static CURSOR_RESIZE_NWSE: [u8; 64] = [
    2,2,2,2,0,0,0,0,
    2,1,1,2,0,0,0,0,
    2,1,2,0,0,0,0,0,
    2,2,0,2,0,0,0,0,
    0,0,0,0,2,0,2,2,
    0,0,0,0,0,2,1,2,
    0,0,0,0,2,1,1,2,
    0,0,0,0,2,2,2,2,
];

static CURSOR_RESIZE_NESW: [u8; 64] = [
    0,0,0,0,2,2,2,2,
    0,0,0,0,2,1,1,2,
    0,0,0,0,0,2,1,2,
    0,0,0,0,2,0,2,2,
    2,2,0,2,0,0,0,0,
    2,1,2,0,0,0,0,0,
    2,1,1,2,0,0,0,0,
    2,2,2,2,0,0,0,0,
];

static CURSOR_MOVE: [u8; 64] = [
    0,0,0,2,0,0,0,0,
    0,0,2,1,2,0,0,0,
    0,0,0,2,0,0,0,0,
    2,2,2,2,2,2,2,0,
    0,0,0,2,0,0,0,0,
    0,0,2,1,2,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,0,0,0,0,0,
];

static CURSOR_WAIT: [u8; 64] = [
    2,2,2,2,2,2,0,0,
    2,1,1,1,1,2,0,0,
    0,2,1,1,2,0,0,0,
    0,0,2,2,0,0,0,0,
    0,0,2,2,0,0,0,0,
    0,2,1,1,2,0,0,0,
    2,1,1,1,1,2,0,0,
    2,2,2,2,2,2,0,0,
];

static CURSOR_CROSSHAIR: [u8; 64] = [
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    2,2,2,1,2,2,2,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,0,0,0,0,0,
];

// ═══════════════════════════════════════════════════════════════════════════════
// TOAST NOTIFICATIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Notification priority
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NotifyPriority {
    Info,
    Warning,
    Error,
    Success,
}

/// A toast notification
pub struct Toast {
    pub title: String,
    pub message: String,
    pub priority: NotifyPriority,
    pub created_at: u64,
    pub duration_ms: u64,
    pub progress: Option<u8>, // 0-100 for progress bar
}

impl Toast {
    pub fn new(title: &str, message: &str, priority: NotifyPriority) -> Self {
        Self {
            title: String::from(title),
            message: String::from(message),
            priority,
            created_at: now_us(),
            duration_ms: 5000,
            progress: None,
        }
    }
    
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
    
    pub fn with_progress(mut self, percent: u8) -> Self {
        self.progress = Some(percent.min(100));
        self
    }
    
    pub fn is_expired(&self) -> bool {
        let elapsed = (now_us() - self.created_at) / 1000;
        elapsed >= self.duration_ms
    }
    
    pub fn get_color(&self) -> u32 {
        match self.priority {
            NotifyPriority::Info => 0xFF3498DB,    // Blue
            NotifyPriority::Warning => 0xFFF39C12, // Orange
            NotifyPriority::Error => 0xFFE74C3C,   // Red
            NotifyPriority::Success => 0xFF27AE60, // Green
        }
    }
}

/// Notification manager
static NOTIFICATIONS: Mutex<Vec<Toast>> = Mutex::new(Vec::new());
const MAX_NOTIFICATIONS: usize = 5;

/// Show a toast notification
pub fn show_toast(title: &str, message: &str, priority: NotifyPriority) {
    let mut notifs = NOTIFICATIONS.lock();
    
    // Remove old notifications
    notifs.retain(|n| !n.is_expired());
    
    // Limit count
    while notifs.len() >= MAX_NOTIFICATIONS {
        notifs.remove(0);
    }
    
    notifs.push(Toast::new(title, message, priority));
}

/// Show progress notification
pub fn show_progress(title: &str, message: &str, percent: u8) {
    let mut notifs = NOTIFICATIONS.lock();
    
    // Update existing progress notification with same title
    for n in notifs.iter_mut() {
        if n.title == title && n.progress.is_some() {
            n.progress = Some(percent.min(100));
            n.message = String::from(message);
            return;
        }
    }
    
    // Create new
    notifs.push(Toast::new(title, message, NotifyPriority::Info)
        .with_progress(percent)
        .with_duration(30000)); // Long timeout for progress
}

/// Get active notifications for rendering
pub fn get_notifications() -> Vec<Toast> {
    let mut notifs = NOTIFICATIONS.lock();
    notifs.retain(|n| !n.is_expired());
    notifs.clone()
}

impl Clone for Toast {
    fn clone(&self) -> Self {
        Self {
            title: self.title.clone(),
            message: self.message.clone(),
            priority: self.priority,
            created_at: self.created_at,
            duration_ms: self.duration_ms,
            progress: self.progress,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// START MENU
// ═══════════════════════════════════════════════════════════════════════════════

/// Start menu state
static START_MENU_OPEN: AtomicBool = AtomicBool::new(false);

/// Start menu entry
#[derive(Clone)]
pub struct StartMenuItem {
    pub name: String,
    pub icon: u8,      // Icon index
    pub action: StartAction,
}

#[derive(Clone, Copy, Debug)]
pub enum StartAction {
    OpenApp(&'static str),
    OpenTerminal,
    OpenFiles,
    OpenSettings,
    OpenAbout,
    Shutdown,
    Restart,
    Lock,
}

/// Toggle start menu
pub fn toggle_start_menu() {
    let current = START_MENU_OPEN.load(Ordering::Relaxed);
    START_MENU_OPEN.store(!current, Ordering::Relaxed);
}

/// Close start menu
pub fn close_start_menu() {
    START_MENU_OPEN.store(false, Ordering::Relaxed);
}

/// Check if start menu is open
pub fn is_start_menu_open() -> bool {
    START_MENU_OPEN.load(Ordering::Relaxed)
}

/// Get start menu items
pub fn get_start_menu_items() -> Vec<StartMenuItem> {
    vec![
        StartMenuItem { name: String::from("Terminal"), icon: 0, action: StartAction::OpenTerminal },
        StartMenuItem { name: String::from("Files"), icon: 1, action: StartAction::OpenFiles },
        StartMenuItem { name: String::from("Settings"), icon: 2, action: StartAction::OpenSettings },
        StartMenuItem { name: String::from("About"), icon: 3, action: StartAction::OpenAbout },
        StartMenuItem { name: String::from("───────────"), icon: 255, action: StartAction::OpenAbout },
        StartMenuItem { name: String::from("Lock"), icon: 4, action: StartAction::Lock },
        StartMenuItem { name: String::from("Restart"), icon: 5, action: StartAction::Restart },
        StartMenuItem { name: String::from("Shutdown"), icon: 6, action: StartAction::Shutdown },
    ]
}

// ═══════════════════════════════════════════════════════════════════════════════
// FAST ALPHA BLENDING with lookup table
// ═══════════════════════════════════════════════════════════════════════════════

/// Precomputed alpha blend table (256 * 256 = 64KB)
/// BLEND_TABLE[alpha][value] = (value * alpha) / 255
static BLEND_TABLE: [[u8; 256]; 256] = {
    let mut table = [[0u8; 256]; 256];
    let mut alpha = 0usize;
    while alpha < 256 {
        let mut value = 0usize;
        while value < 256 {
            table[alpha][value] = ((value * alpha + 127) / 255) as u8;
            value += 1;
        }
        alpha += 1;
    }
    table
};

/// Fast alpha blend using lookup table
#[inline(always)]
pub fn blend_fast(src: u32, dst: u32) -> u32 {
    let alpha = ((src >> 24) & 0xFF) as usize;
    if alpha == 0 { return dst; }
    if alpha == 255 { return src; }
    
    let inv_alpha = 255 - alpha;
    
    let sr = ((src >> 16) & 0xFF) as usize;
    let sg = ((src >> 8) & 0xFF) as usize;
    let sb = (src & 0xFF) as usize;
    
    let dr = ((dst >> 16) & 0xFF) as usize;
    let dg = ((dst >> 8) & 0xFF) as usize;
    let db = (dst & 0xFF) as usize;
    
    let r = BLEND_TABLE[alpha][sr] as u32 + BLEND_TABLE[inv_alpha][dr] as u32;
    let g = BLEND_TABLE[alpha][sg] as u32 + BLEND_TABLE[inv_alpha][dg] as u32;
    let b = BLEND_TABLE[alpha][sb] as u32 + BLEND_TABLE[inv_alpha][db] as u32;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}

// ═══════════════════════════════════════════════════════════════════════════════
// OPTIMIZED DIRTY RECTANGLES
// ═══════════════════════════════════════════════════════════════════════════════

/// Rectangle
#[derive(Clone, Copy, Default, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }
    
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w as i32 &&
        self.x + self.w as i32 > other.x &&
        self.y < other.y + other.h as i32 &&
        self.y + self.h as i32 > other.y
    }
    
    pub fn union(&self, other: &Rect) -> Rect {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.w as i32).max(other.x + other.w as i32);
        let y2 = (self.y + self.h as i32).max(other.y + other.h as i32);
        Rect {
            x: x1,
            y: y1,
            w: (x2 - x1) as u32,
            h: (y2 - y1) as u32,
        }
    }
    
    pub fn area(&self) -> u32 {
        self.w * self.h
    }
}

/// Optimized dirty region tracker with smart merging
pub struct DirtyTracker {
    rects: Vec<Rect>,
    full_redraw: bool,
    screen_w: u32,
    screen_h: u32,
}

impl DirtyTracker {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            rects: Vec::with_capacity(64),
            full_redraw: true,
            screen_w: w,
            screen_h: h,
        }
    }
    
    /// Mark rectangle as dirty
    pub fn mark(&mut self, rect: Rect) {
        if self.full_redraw { return; }
        if rect.w == 0 || rect.h == 0 { return; }
        
        // Try to merge with overlapping rectangles
        for i in 0..self.rects.len() {
            if self.rects[i].intersects(&rect) {
                // Check if merging is worth it (doesn't waste too much area)
                let merged = self.rects[i].union(&rect);
                let wasted = merged.area() as i64 - 
                    (self.rects[i].area() + rect.area()) as i64;
                
                // Merge if wasted area is less than 50% of smaller rect
                let smaller = self.rects[i].area().min(rect.area());
                if wasted < (smaller / 2) as i64 {
                    self.rects[i] = merged;
                    return;
                }
            }
        }
        
        // Add as new rectangle
        if self.rects.len() < 64 {
            self.rects.push(rect);
        } else {
            // Too many rectangles, merge into full redraw
            self.full_redraw = true;
        }
    }
    
    /// Mark full screen dirty
    pub fn mark_full(&mut self) {
        self.full_redraw = true;
    }
    
    /// Clear all dirty regions
    pub fn clear(&mut self) {
        self.rects.clear();
        self.full_redraw = false;
    }
    
    /// Get dirty rectangles (or full screen rect if full_redraw)
    pub fn get_dirty(&self) -> &[Rect] {
        &self.rects
    }
    
    /// Check if full redraw needed
    pub fn needs_full_redraw(&self) -> bool {
        self.full_redraw
    }
}
