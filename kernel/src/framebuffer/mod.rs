//! Framebuffer Console
//!
//! Provides text rendering on the Limine framebuffer.
//! Replaces VGA text mode for UEFI boot.
//! Supports double buffering for flicker-free GUI rendering.

pub mod font;
pub mod logo;

use core::fmt;
use spin::Mutex;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use core::sync::atomic::{AtomicPtr, AtomicU64, AtomicBool, Ordering};

/// Framebuffer info stored after initialization
struct FramebufferInfo {
    addr: *mut u8,
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
}

/// Console state
struct Console {
    cursor_x: usize,
    cursor_y: usize,
    fg_color: u32,
    bg_color: u32,
}

// Static storage
static FB_ADDR: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static FB_WIDTH: AtomicU64 = AtomicU64::new(0);
static FB_HEIGHT: AtomicU64 = AtomicU64::new(0);
static FB_PITCH: AtomicU64 = AtomicU64::new(0);

// Double buffering
static BACKBUFFER: Mutex<Option<Box<[u32]>>> = Mutex::new(None);
static USE_BACKBUFFER: AtomicBool = AtomicBool::new(false);

// ==================== SCROLLBACK BUFFER ====================
// Store terminal history for scroll up/down functionality
const SCROLLBACK_MAX_LINES: usize = 1000;  // Store up to 1000 lines of history
const SCROLLBACK_LINE_WIDTH: usize = 256; // Max characters per line

/// A single line in the scrollback buffer
#[derive(Clone)]
struct ScrollbackLine {
    chars: [char; SCROLLBACK_LINE_WIDTH],
    colors: [(u32, u32); SCROLLBACK_LINE_WIDTH], // (fg, bg) for each char
    len: usize,
}

impl ScrollbackLine {
    const fn new() -> Self {
        ScrollbackLine {
            chars: [' '; SCROLLBACK_LINE_WIDTH],
            colors: [(0xFFFFFFFF, 0xFF000000); SCROLLBACK_LINE_WIDTH],
            len: 0,
        }
    }
}

/// Scrollback buffer state
struct ScrollbackBuffer {
    lines: Vec<ScrollbackLine>,
    current_line: ScrollbackLine,
    scroll_offset: usize,  // How many lines we've scrolled back (0 = at bottom)
    is_scrolled: bool,     // Are we in scroll mode?
}

impl ScrollbackBuffer {
    fn new() -> Self {
        ScrollbackBuffer {
            lines: Vec::with_capacity(SCROLLBACK_MAX_LINES),
            current_line: ScrollbackLine::new(),
            scroll_offset: 0,
            is_scrolled: false,
        }
    }
    
    /// Add a character to the current line
    fn push_char(&mut self, c: char, fg: u32, bg: u32) {
        if c == '\n' {
            // Commit current line and start new one
            self.commit_line();
        } else if c == '\r' {
            // Carriage return - reset to beginning of line
            self.current_line.len = 0;
        } else if c != '\x08' && c.is_ascii_graphic() || c == ' ' {
            if self.current_line.len < SCROLLBACK_LINE_WIDTH {
                self.current_line.chars[self.current_line.len] = c;
                self.current_line.colors[self.current_line.len] = (fg, bg);
                self.current_line.len += 1;
            }
        }
    }
    
    /// Commit current line to history
    fn commit_line(&mut self) {
        if self.lines.len() >= SCROLLBACK_MAX_LINES {
            self.lines.remove(0); // Remove oldest line
        }
        self.lines.push(self.current_line.clone());
        self.current_line = ScrollbackLine::new();
    }
    
    /// Get total lines in history
    fn total_lines(&self) -> usize {
        self.lines.len()
    }
}

static SCROLLBACK: Mutex<Option<ScrollbackBuffer>> = Mutex::new(None);
static SCROLLBACK_ENABLED: AtomicBool = AtomicBool::new(false);

// ==================== BACKGROUND CACHE ====================
// Cache the static background to avoid recomputing every frame
static BACKGROUND_CACHE: Mutex<Option<Box<[u32]>>> = Mutex::new(None);
static BACKGROUND_VALID: AtomicBool = AtomicBool::new(false);

// ==================== DIRTY RECTANGLES ====================
// Only redraw regions that have changed
pub const MAX_DIRTY_RECTS: usize = 32;

#[derive(Clone, Copy, Default)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl DirtyRect {
    pub const fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        DirtyRect { x, y, w, h }
    }
    
    /// Check if rectangle is valid (non-zero size)
    pub fn is_valid(&self) -> bool {
        self.w > 0 && self.h > 0
    }
    
    /// Merge with another rectangle (union)
    pub fn merge(&self, other: &DirtyRect) -> DirtyRect {
        if !self.is_valid() { return *other; }
        if !other.is_valid() { return *self; }
        
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.w).max(other.x + other.w);
        let y2 = (self.y + self.h).max(other.y + other.h);
        
        DirtyRect { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }
    }
    
    /// Check if overlaps with another rectangle
    pub fn overlaps(&self, other: &DirtyRect) -> bool {
        !(self.x + self.w <= other.x || other.x + other.w <= self.x ||
          self.y + self.h <= other.y || other.y + other.h <= self.y)
    }
}

struct DirtyRectList {
    rects: [DirtyRect; MAX_DIRTY_RECTS],
    count: usize,
    full_redraw: bool,
}

impl DirtyRectList {
    const fn new() -> Self {
        DirtyRectList {
            rects: [DirtyRect { x: 0, y: 0, w: 0, h: 0 }; MAX_DIRTY_RECTS],
            count: 0,
            full_redraw: true, // Start with full redraw
        }
    }
    
    fn add(&mut self, rect: DirtyRect) {
        if self.full_redraw { return; } // Already doing full redraw
        if !rect.is_valid() { return; }
        
        // Try to merge with existing rectangles
        for i in 0..self.count {
            if self.rects[i].overlaps(&rect) {
                self.rects[i] = self.rects[i].merge(&rect);
                return;
            }
        }
        
        // Add as new rectangle
        if self.count < MAX_DIRTY_RECTS {
            self.rects[self.count] = rect;
            self.count += 1;
        } else {
            // Too many dirty rects, do full redraw
            self.full_redraw = true;
        }
    }
    
    fn clear(&mut self) {
        self.count = 0;
        self.full_redraw = false;
    }
    
    fn mark_full_redraw(&mut self) {
        self.full_redraw = true;
    }
}

static DIRTY_RECTS: Mutex<DirtyRectList> = Mutex::new(DirtyRectList::new());

static CONSOLE: Mutex<Console> = Mutex::new(Console {
    cursor_x: 0,
    cursor_y: 0,
    fg_color: 0xFFFFFFFF, // White
    bg_color: 0xFF000000, // Black
});

/// Character dimensions
const CHAR_WIDTH: usize = 8;
const CHAR_HEIGHT: usize = 16;

/// Initialize framebuffer with Limine info
/// NOTE: This does NOT allocate memory. Call init_scrollback() after heap is ready.
pub fn init(addr: *mut u8, width: u64, height: u64, pitch: u64, bpp: u16) {
    FB_ADDR.store(addr, Ordering::SeqCst);
    FB_WIDTH.store(width, Ordering::SeqCst);
    FB_HEIGHT.store(height, Ordering::SeqCst);
    FB_PITCH.store(pitch, Ordering::SeqCst);
    
    // NOTE: Do NOT call init_scrollback() here! It allocates memory.
    // Call it after heap initialization in main.rs
    
    // Clear screen (no allocation - just writes to framebuffer directly)
    clear();
}

/// Initialize the scrollback buffer
pub fn init_scrollback() {
    *SCROLLBACK.lock() = Some(ScrollbackBuffer::new());
    SCROLLBACK_ENABLED.store(true, Ordering::SeqCst);
    crate::serial_println!("[FB] Scrollback buffer initialized ({} lines max)", SCROLLBACK_MAX_LINES);
}

/// Check if framebuffer is initialized
pub fn is_initialized() -> bool {
    !FB_ADDR.load(Ordering::SeqCst).is_null()
}

/// Clear the screen
pub fn clear() {
    let addr = FB_ADDR.load(Ordering::SeqCst);
    if addr.is_null() {
        return;
    }
    
    let height = FB_HEIGHT.load(Ordering::SeqCst) as usize;
    let pitch = FB_PITCH.load(Ordering::SeqCst) as usize;
    let bg_color = CONSOLE.lock().bg_color;
    
    for y in 0..height {
        let row = unsafe { addr.add(y * pitch) };
        for x in 0..(pitch / 4) {
            unsafe {
                row.add(x * 4).cast::<u32>().write_volatile(bg_color);
            }
        }
    }
    
    let mut console = CONSOLE.lock();
    console.cursor_x = 0;
    console.cursor_y = 0;
}

/// Set foreground color
pub fn set_fg_color(color: u32) {
    CONSOLE.lock().fg_color = color;
}

/// Set background color  
pub fn set_bg_color(color: u32) {
    CONSOLE.lock().bg_color = color;
}

/// Put a pixel at (x, y) - public for GUI use
pub fn put_pixel(x: u32, y: u32, color: u32) {
    let addr = FB_ADDR.load(Ordering::SeqCst);
    if addr.is_null() {
        return;
    }
    
    let width = FB_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as u32;
    let pitch = FB_PITCH.load(Ordering::SeqCst) as usize;
    
    if x >= width || y >= height {
        return;
    }
    
    // Write to backbuffer if enabled, otherwise direct to framebuffer
    if USE_BACKBUFFER.load(Ordering::SeqCst) {
        if let Some(ref mut buf) = *BACKBUFFER.lock() {
            let idx = y as usize * width as usize + x as usize;
            if idx < buf.len() {
                buf[idx] = color;
            }
        }
    } else {
        let offset = y as usize * pitch + x as usize * 4;
        unsafe {
            addr.add(offset).cast::<u32>().write_volatile(color);
        }
    }
}

/// Get a pixel at (x, y) - for alpha blending and image compositing
pub fn get_pixel(x: u32, y: u32) -> u32 {
    let addr = FB_ADDR.load(Ordering::SeqCst);
    if addr.is_null() {
        return 0;
    }
    
    let width = FB_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as u32;
    let pitch = FB_PITCH.load(Ordering::SeqCst) as usize;
    
    if x >= width || y >= height {
        return 0;
    }
    
    // Read from backbuffer if enabled, otherwise from framebuffer
    if USE_BACKBUFFER.load(Ordering::SeqCst) {
        if let Some(ref buf) = *BACKBUFFER.lock() {
            let idx = y as usize * width as usize + x as usize;
            if idx < buf.len() {
                return buf[idx];
            }
        }
        0
    } else {
        let offset = y as usize * pitch + x as usize * 4;
        unsafe {
            addr.add(offset).cast::<u32>().read_volatile()
        }
    }
}

/// Get framebuffer dimensions
pub fn get_dimensions() -> (u32, u32) {
    (FB_WIDTH.load(Ordering::SeqCst) as u32, FB_HEIGHT.load(Ordering::SeqCst) as u32)
}

/// Get raw framebuffer address for direct access
pub fn get_fb_addr() -> *mut u8 {
    FB_ADDR.load(Ordering::SeqCst)
}

/// Get framebuffer pitch (bytes per row)
pub fn get_fb_pitch() -> usize {
    FB_PITCH.load(Ordering::SeqCst) as usize
}

// ==================== DOUBLE BUFFERING ====================

/// Initialize double buffering (allocate backbuffer)
pub fn init_double_buffer() {
    let width = FB_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as usize;
    
    if width == 0 || height == 0 {
        return;
    }
    
    let size = width * height;
    let buffer = alloc::vec![0u32; size].into_boxed_slice();
    
    *BACKBUFFER.lock() = Some(buffer);
    crate::serial_println!("[FB] Double buffer allocated: {}x{} ({} KB)", width, height, size * 4 / 1024);
}

/// Enable/disable double buffering mode
pub fn set_double_buffer_mode(enabled: bool) {
    USE_BACKBUFFER.store(enabled, Ordering::SeqCst);
}

/// Check if double buffering is enabled
pub fn is_double_buffer_enabled() -> bool {
    USE_BACKBUFFER.load(Ordering::SeqCst)
}

/// Get backbuffer info for external rendering
/// Returns (pointer, width, height, stride) if backbuffer is initialized
pub fn get_backbuffer_info() -> Option<(*mut u8, u32, u32, u32)> {
    let width = FB_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as u32;
    
    if width == 0 || height == 0 {
        return None;
    }
    
    let backbuffer = BACKBUFFER.lock();
    if let Some(ref buf) = *backbuffer {
        // Return pointer to the boxed slice's data
        let ptr = buf.as_ptr() as *mut u8;
        Some((ptr, width, height, width)) // stride = width for backbuffer
    } else {
        None
    }
}

/// Swap buffers - copy backbuffer to framebuffer (fast memcpy)
pub fn swap_buffers() {
    let addr = FB_ADDR.load(Ordering::SeqCst);
    if addr.is_null() {
        return;
    }
    
    let width = FB_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as usize;
    let pitch = FB_PITCH.load(Ordering::SeqCst) as usize;
    
    if let Some(ref buf) = *BACKBUFFER.lock() {
        // Fast copy row by row (handles pitch != width*4)
        for y in 0..height {
            let src_offset = y * width;
            let dst_offset = y * pitch;
            
            unsafe {
                let src = buf.as_ptr().add(src_offset);
                let dst = addr.add(dst_offset) as *mut u32;
                core::ptr::copy_nonoverlapping(src, dst, width);
            }
        }
    }
}

/// Clear backbuffer with color (optimized with slice fill)
pub fn clear_backbuffer(color: u32) {
    if let Some(ref mut buf) = *BACKBUFFER.lock() {
        // Using fill() is faster than iterating
        buf.fill(color);
    }
}

/// Draw filled rectangle to backbuffer (fast)
/// Draw filled rectangle to backbuffer (optimized with slice fill)
pub fn fill_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    let width = FB_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as u32;
    
    let x1 = x.min(width);
    let y1 = y.min(height);
    let x2 = (x + w).min(width);
    let y2 = (y + h).min(height);
    
    if x2 <= x1 || y2 <= y1 { return; }
    
    if USE_BACKBUFFER.load(Ordering::SeqCst) {
        if let Some(ref mut buf) = *BACKBUFFER.lock() {
            let rect_width = (x2 - x1) as usize;
            for py in y1..y2 {
                let row_start = py as usize * width as usize + x1 as usize;
                if row_start + rect_width <= buf.len() {
                    // Use slice fill for each row (much faster than loop)
                    buf[row_start..row_start + rect_width].fill(color);
                }
            }
        }
    } else {
        for py in y1..y2 {
            for px in x1..x2 {
                put_pixel(px, py, color);
            }
        }
    }
}

/// Draw a single pixel (bounds checked)
pub fn draw_pixel(x: u32, y: u32, color: u32) {
    let addr = FB_ADDR.load(Ordering::SeqCst);
    if addr.is_null() { return; }
    
    let width = FB_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as u32;
    let pitch = FB_PITCH.load(Ordering::SeqCst) as usize;
    
    if x < width && y < height {
        let offset = y as usize * pitch + x as usize * 4;
        unsafe {
            let ptr = addr.add(offset) as *mut u32;
            *ptr = color;
        }
    }
}

/// Draw horizontal line (optimized)
pub fn draw_hline(x: u32, y: u32, len: u32, color: u32) {
    fill_rect(x, y, len, 1, color);
}

/// Draw vertical line (optimized)
pub fn draw_vline(x: u32, y: u32, len: u32, color: u32) {
    fill_rect(x, y, 1, len, color);
}

/// Draw rectangle outline
pub fn draw_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    draw_hline(x, y, w, color);
    draw_hline(x, y + h - 1, w, color);
    draw_vline(x, y, h, color);
    draw_vline(x + w - 1, y, h, color);
}

/// Draw a character at pixel position (private)
fn draw_char(c: char, x: usize, y: usize, fg: u32, bg: u32) {
    let glyph = font::get_glyph(c);
    
    for row in 0..CHAR_HEIGHT {
        let bits = glyph[row];
        for col in 0..CHAR_WIDTH {
            let color = if (bits >> (7 - col)) & 1 == 1 { fg } else { bg };
            put_pixel((x + col) as u32, (y + row) as u32, color);
        }
    }
}

/// Draw a character at pixel position with transparent background
pub fn draw_char_at(x: u32, y: u32, c: char, color: u32) {
    let glyph = font::get_glyph(c);
    
    for row in 0..CHAR_HEIGHT {
        let bits = glyph[row];
        for col in 0..CHAR_WIDTH {
            if (bits >> (7 - col)) & 1 == 1 {
                draw_pixel(x + col as u32, y + row as u32, color);
            }
        }
    }
}

/// Write a character to the console
fn write_char(c: char) {
    let width = FB_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as usize;
    
    if width == 0 || height == 0 {
        return;
    }
    
    let cols = width / CHAR_WIDTH;
    let rows = height / CHAR_HEIGHT;
    
    let mut console = CONSOLE.lock();
    let fg = console.fg_color;
    let bg = console.bg_color;
    
    // Store character in scrollback buffer (if not in scroll mode)
    if SCROLLBACK_ENABLED.load(Ordering::SeqCst) {
        if let Some(ref mut scrollback) = *SCROLLBACK.lock() {
            if !scrollback.is_scrolled {
                scrollback.push_char(c, fg, bg);
            }
        }
    }
    
    match c {
        '\x08' => {
            if console.cursor_x > 0 {
                console.cursor_x -= 1;
            } else if console.cursor_y > 0 {
                console.cursor_y -= 1;
                console.cursor_x = cols.saturating_sub(1);
            } else {
                return;
            }

            let px = console.cursor_x * CHAR_WIDTH;
            let py = console.cursor_y * CHAR_HEIGHT;
            let fg = console.fg_color;
            let bg = console.bg_color;
            drop(console);
            draw_char(' ', px, py, fg, bg);
            return;
        }
        '\n' => {
            console.cursor_x = 0;
            console.cursor_y += 1;
        }
        '\r' => {
            console.cursor_x = 0;
        }
        '\t' => {
            console.cursor_x = (console.cursor_x + 4) & !3;
        }
        _ => {
            if console.cursor_x >= cols {
                console.cursor_x = 0;
                console.cursor_y += 1;
            }
            
            let px = console.cursor_x * CHAR_WIDTH;
            let py = console.cursor_y * CHAR_HEIGHT;
            
            // Drop lock before drawing (to avoid holding it too long)
            let x = console.cursor_x;
            console.cursor_x += 1;
            drop(console);
            
            draw_char(c, px, py, fg, bg);
            return;
        }
    }
    
    // Handle scrolling
    if console.cursor_y >= rows {
        drop(console);
        scroll_up();
        CONSOLE.lock().cursor_y = rows - 1;
    }
}

/// Scroll the screen up by one line
fn scroll_up() {
    let addr = FB_ADDR.load(Ordering::SeqCst);
    if addr.is_null() {
        return;
    }
    
    let height = FB_HEIGHT.load(Ordering::SeqCst) as usize;
    let pitch = FB_PITCH.load(Ordering::SeqCst) as usize;
    let bg_color = CONSOLE.lock().bg_color;
    
    // Copy each line up
    for y in CHAR_HEIGHT..height {
        unsafe {
            let src = addr.add(y * pitch);
            let dst = addr.add((y - CHAR_HEIGHT) * pitch);
            core::ptr::copy(src, dst, pitch);
        }
    }
    
    // Clear last line
    for y in (height - CHAR_HEIGHT)..height {
        let row = unsafe { addr.add(y * pitch) };
        for x in 0..(pitch / 4) {
            unsafe {
                row.add(x * 4).cast::<u32>().write_volatile(bg_color);
            }
        }
    }
}

/// Writer struct for fmt::Write
pub struct Writer;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            write_char(c);
        }
        Ok(())
    }
}

/// Internal print function
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    Writer.write_fmt(args).unwrap();
    crate::serial::_print(args);
}

/// Print to framebuffer console
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::framebuffer::_print(format_args!($($arg)*))
    };
}

/// Print to framebuffer console with newline
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

// === COLOR CONSTANTS (ARGB format) ===
pub const COLOR_BLACK: u32 = 0xFF000000;
pub const COLOR_WHITE: u32 = 0xFFFFFFFF;
pub const COLOR_GREEN: u32 = 0xFF00FF00;
pub const COLOR_BRIGHT_GREEN: u32 = 0xFF00FF66;
pub const COLOR_DARK_GREEN: u32 = 0xFF00AA00;
pub const COLOR_RED: u32 = 0xFFFF0000;
pub const COLOR_BLUE: u32 = 0xFF0000FF;
pub const COLOR_YELLOW: u32 = 0xFFFFFF00;
pub const COLOR_CYAN: u32 = 0xFF00FFFF;
pub const COLOR_MAGENTA: u32 = 0xFFFF00FF;
pub const COLOR_GRAY: u32 = 0xFF888888;

/// Print with specific color (temporarily changes fg, then restores)
#[macro_export]
macro_rules! print_color {
    ($color:expr, $($arg:tt)*) => {{
        let old = $crate::framebuffer::get_fg_color();
        $crate::framebuffer::set_fg_color($color);
        $crate::print!($($arg)*);
        $crate::framebuffer::set_fg_color(old);
    }};
}

/// Print with color and newline
#[macro_export]
macro_rules! println_color {
    ($color:expr, $($arg:tt)*) => {{
        let old = $crate::framebuffer::get_fg_color();
        $crate::framebuffer::set_fg_color($color);
        $crate::println!($($arg)*);
        $crate::framebuffer::set_fg_color(old);
    }};
}

/// Get current foreground color
pub fn get_fg_color() -> u32 {
    CONSOLE.lock().fg_color
}

/// Set colors to Matrix-style theme (green on black)
pub fn set_matrix_theme() {
    let mut console = CONSOLE.lock();
    console.fg_color = COLOR_GREEN;
    console.bg_color = COLOR_BLACK;
}

/// Set colors to classic white on black
pub fn set_classic_theme() {
    let mut console = CONSOLE.lock();
    console.fg_color = COLOR_WHITE;
    console.bg_color = COLOR_BLACK;
}

// ==================== ENHANCED UI FUNCTIONS ====================

/// Draw text at specific pixel position with custom colors
pub fn draw_text_at(text: &str, x: u32, y: u32, fg: u32, bg: u32) {
    for (i, c) in text.chars().enumerate() {
        let px = x + (i as u32) * CHAR_WIDTH as u32;
        draw_char(c, px as usize, y as usize, fg, bg);
    }
}

/// Draw centered text at y position
pub fn draw_text_centered(text: &str, y: u32, fg: u32) {
    let (width, _) = get_dimensions();
    let text_width = text.len() as u32 * CHAR_WIDTH as u32;
    let x = (width.saturating_sub(text_width)) / 2;
    draw_text_at(text, x, y, fg, COLOR_BLACK);
}

/// Draw a horizontal separator line
pub fn draw_separator(y: u32, color: u32) {
    let (width, _) = get_dimensions();
    let margin = width / 10;
    draw_hline(margin, y, width - 2 * margin, color);
}

/// Set cursor position (in character coordinates)
pub fn set_cursor(col: usize, row: usize) {
    let mut console = CONSOLE.lock();
    console.cursor_x = col;
    console.cursor_y = row;
}

/// Get cursor position
pub fn get_cursor() -> (usize, usize) {
    let console = CONSOLE.lock();
    (console.cursor_x, console.cursor_y)
}

/// Draw progress bar
pub fn draw_progress_bar(x: u32, y: u32, width: u32, progress: u32, fg: u32, bg: u32) {
    let filled = (width * progress.min(100)) / 100;
    
    // Background
    fill_rect(x, y, width, 16, bg);
    
    // Filled portion
    if filled > 0 {
        fill_rect(x, y, filled, 16, fg);
    }
    
    // Border
    draw_rect(x, y, width, 16, fg);
}

/// Print styled boot message [OK], [--], [!!]
pub fn print_boot_status(msg: &str, status: BootStatus) {
    let (status_str, color) = match status {
        BootStatus::Ok => ("[OK]", COLOR_GREEN),
        BootStatus::Skip => ("[--]", COLOR_GRAY),
        BootStatus::Fail => ("[!!]", COLOR_RED),
        BootStatus::Info => ("[..]", COLOR_CYAN),
    };
    
    // Print status with color
    let old_fg = get_fg_color();
    set_fg_color(color);
    crate::print!("{} ", status_str);
    set_fg_color(old_fg);
    crate::println!("{}", msg);
}

/// Boot status enum for styled messages
#[derive(Clone, Copy)]
pub enum BootStatus {
    Ok,
    Skip,
    Fail,
    Info,
}

/// Draw the boot splash screen with logo
pub fn draw_boot_splash() {
    logo::draw_boot_splash();
}

/// Clear screen and show Matrix-styled boot header
pub fn show_boot_header() {
    clear();
    set_matrix_theme();
    
    let (width, _height) = get_dimensions();
    
    // Draw top border
    draw_separator(0, COLOR_GREEN);
    
    // Draw ASCII art banner
    let banner_y = 16u32;
    draw_text_centered("╔════════════════════════════════════════════════════════════╗", banner_y, COLOR_BRIGHT_GREEN);
    draw_text_centered("║                                                            ║", banner_y + 16, COLOR_BRIGHT_GREEN);
    draw_text_centered("║   ████████╗██████╗ ██╗   ██╗███████╗████████╗ ██████╗ ███████╗  ║", banner_y + 32, COLOR_GREEN);
    draw_text_centered("║   ╚══██╔══╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔═══██╗██╔════╝  ║", banner_y + 48, COLOR_GREEN);
    draw_text_centered("║      ██║   ██████╔╝██║   ██║███████╗   ██║   ██║   ██║███████╗  ║", banner_y + 64, COLOR_BRIGHT_GREEN);
    draw_text_centered("║      ██║   ██╔══██╗██║   ██║╚════██║   ██║   ██║   ██║╚════██║  ║", banner_y + 80, COLOR_GREEN);
    draw_text_centered("║      ██║   ██║  ██║╚██████╔╝███████║   ██║   ╚██████╔╝███████║  ║", banner_y + 96, COLOR_BRIGHT_GREEN);
    draw_text_centered("║      ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝ ╚══════╝  ║", banner_y + 112, COLOR_GREEN);
    draw_text_centered("║                                                            ║", banner_y + 128, COLOR_BRIGHT_GREEN);
    draw_text_centered("║            FAST  •  SECURE  •  RELIABLE                    ║", banner_y + 144, COLOR_DARK_GREEN);
    draw_text_centered("║                                                            ║", banner_y + 160, COLOR_BRIGHT_GREEN);
    draw_text_centered("╚════════════════════════════════════════════════════════════╝", banner_y + 176, COLOR_BRIGHT_GREEN);
    
    // Draw bottom separator
    draw_separator(banner_y + 200, COLOR_GREEN);
    
    // Set cursor below the banner for boot messages
    let start_row = ((banner_y + 220) / 16) as usize;
    set_cursor(0, start_row);
}

/// Simple boot header for faster boot
pub fn show_simple_boot_header() {
    clear();
    set_matrix_theme();
    
    crate::println!("╔══════════════════════════════════════════════════════════════╗");
    crate::println!("║                      T R U S T - O S                         ║");
    crate::println!("║                 FAST • SECURE • RELIABLE                     ║");
    crate::println!("╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
}

// ==================== BACKGROUND CACHE FUNCTIONS ====================

/// Initialize background cache buffer
pub fn init_background_cache() {
    let width = FB_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as usize;
    
    if width == 0 || height == 0 {
        return;
    }
    
    let size = width * height;
    let buffer = alloc::vec![0u32; size].into_boxed_slice();
    
    *BACKGROUND_CACHE.lock() = Some(buffer);
    BACKGROUND_VALID.store(false, Ordering::SeqCst);
    crate::serial_println!("[FB] Background cache allocated: {} KB", size * 4 / 1024);
}

/// Mark background cache as valid (after rendering background to it)
pub fn validate_background_cache() {
    BACKGROUND_VALID.store(true, Ordering::SeqCst);
}

/// Invalidate background cache (force re-render)
pub fn invalidate_background_cache() {
    BACKGROUND_VALID.store(false, Ordering::SeqCst);
}

/// Check if background cache is valid
pub fn is_background_cached() -> bool {
    BACKGROUND_VALID.load(Ordering::SeqCst)
}

/// Blit background cache to backbuffer (fast memcpy)
pub fn restore_background_to_backbuffer() {
    let width = FB_WIDTH.load(Ordering::SeqCst) as usize;
    
    let bg_guard = BACKGROUND_CACHE.lock();
    if let Some(ref bg_buf) = *bg_guard {
        if let Some(ref mut back_buf) = *BACKBUFFER.lock() {
            // Fast copy entire background
            let len = bg_buf.len().min(back_buf.len());
            unsafe {
                core::ptr::copy_nonoverlapping(bg_buf.as_ptr(), back_buf.as_mut_ptr(), len);
            }
        }
    }
}

/// Restore only a rectangular region from background cache
pub fn restore_background_rect(x: u32, y: u32, w: u32, h: u32) {
    let width = FB_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as u32;
    
    let x1 = x.min(width);
    let y1 = y.min(height);
    let x2 = (x + w).min(width);
    let y2 = (y + h).min(height);
    
    let bg_guard = BACKGROUND_CACHE.lock();
    if let Some(ref bg_buf) = *bg_guard {
        if let Some(ref mut back_buf) = *BACKBUFFER.lock() {
            for py in y1..y2 {
                let src_start = py as usize * width as usize + x1 as usize;
                let src_end = py as usize * width as usize + x2 as usize;
                let dst_start = src_start;
                
                if src_end <= bg_buf.len() && src_end <= back_buf.len() {
                    unsafe {
                        let src = bg_buf.as_ptr().add(src_start);
                        let dst = back_buf.as_mut_ptr().add(dst_start);
                        core::ptr::copy_nonoverlapping(src, dst, (x2 - x1) as usize);
                    }
                }
            }
        }
    }
}

/// Draw to background cache instead of backbuffer (call before drawing background)
pub fn draw_to_background_cache<F: FnOnce()>(draw_fn: F) {
    // Draw to backbuffer first
    draw_fn();
    
    // Then copy backbuffer to background cache
    cache_current_background();
}

/// Copy current backbuffer to background cache
pub fn cache_current_background() {
    // Get dimensions first
    let width = FB_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as usize;
    let size = width * height;
    
    if size == 0 { return; }
    
    // Copy from backbuffer to background cache
    // Must lock both, but be careful about order
    let mut bg_guard = BACKGROUND_CACHE.lock();
    let back_guard = BACKBUFFER.lock();
    
    if let (Some(ref mut bg_buf), Some(ref back_buf)) = (&mut *bg_guard, &*back_guard) {
        let len = back_buf.len().min(bg_buf.len());
        unsafe {
            core::ptr::copy_nonoverlapping(back_buf.as_ptr(), bg_buf.as_mut_ptr(), len);
        }
    }
    
    drop(back_guard);
    drop(bg_guard);
    
    BACKGROUND_VALID.store(true, Ordering::SeqCst);
}

// ==================== DIRTY RECTANGLE FUNCTIONS ====================

/// Add a dirty rectangle
pub fn add_dirty_rect(x: u32, y: u32, w: u32, h: u32) {
    DIRTY_RECTS.lock().add(DirtyRect::new(x, y, w, h));
}

/// Mark entire screen as dirty (force full redraw)
pub fn mark_full_redraw() {
    DIRTY_RECTS.lock().mark_full_redraw();
}

/// Clear dirty rectangles (call after processing them)
pub fn clear_dirty_rects() {
    DIRTY_RECTS.lock().clear();
}

/// Check if we need full redraw
pub fn needs_full_redraw() -> bool {
    DIRTY_RECTS.lock().full_redraw
}

/// Get dirty rectangles for partial update
pub fn get_dirty_rects() -> ([DirtyRect; MAX_DIRTY_RECTS], usize, bool) {
    let guard = DIRTY_RECTS.lock();
    (guard.rects, guard.count, guard.full_redraw)
}

/// Swap only dirty regions to framebuffer (optimized swap)
pub fn swap_dirty_regions() {
    let addr = FB_ADDR.load(Ordering::SeqCst);
    if addr.is_null() {
        return;
    }
    
    let width = FB_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as usize;
    let pitch = FB_PITCH.load(Ordering::SeqCst) as usize;
    
    let (rects, count, full_redraw) = get_dirty_rects();
    
    if full_redraw {
        // Fall back to full swap
        swap_buffers();
        clear_dirty_rects();
        return;
    }
    
    if let Some(ref buf) = *BACKBUFFER.lock() {
        for i in 0..count {
            let rect = &rects[i];
            if !rect.is_valid() { continue; }
            
            let x1 = (rect.x as usize).min(width);
            let y1 = (rect.y as usize).min(height);
            let x2 = ((rect.x + rect.w) as usize).min(width);
            let y2 = ((rect.y + rect.h) as usize).min(height);
            
            for y in y1..y2 {
                let src_offset = y * width + x1;
                let dst_offset = y * pitch / 4 + x1; // pitch is in bytes, divide by 4 for u32
                let len = x2 - x1;
                
                unsafe {
                    let src = buf.as_ptr().add(src_offset);
                    let dst = (addr as *mut u32).add(y * pitch / 4 + x1);
                    core::ptr::copy_nonoverlapping(src, dst, len);
                }
            }
        }
    }
    
    clear_dirty_rects();
}

/// Get the current cursor row (in character cells)
pub fn get_cursor_row() -> usize {
    CONSOLE.lock().cursor_y
}

/// Get the current cursor column (in character cells)
pub fn get_cursor_col() -> usize {
    CONSOLE.lock().cursor_x
}

/// Scroll the terminal view up (PageUp) - shows older content
pub fn scroll_up_lines(lines: usize) {
    if !SCROLLBACK_ENABLED.load(Ordering::SeqCst) {
        return;
    }
    
    let width = FB_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 {
        return;
    }
    
    let visible_rows = height / CHAR_HEIGHT;
    
    let mut scrollback = SCROLLBACK.lock();
    if let Some(ref mut sb) = *scrollback {
        let max_scroll = sb.total_lines().saturating_sub(visible_rows);
        sb.scroll_offset = (sb.scroll_offset + lines).min(max_scroll);
        sb.is_scrolled = sb.scroll_offset > 0;
        
        if sb.is_scrolled {
            // Redraw screen with scrollback content
            let offset = sb.scroll_offset;
            let total = sb.total_lines();
            drop(scrollback);
            redraw_from_scrollback(offset, total, visible_rows);
        }
    }
}

/// Scroll the terminal view down (PageDown) - shows newer content
pub fn scroll_down(lines: usize) {
    if !SCROLLBACK_ENABLED.load(Ordering::SeqCst) {
        return;
    }
    
    let width = FB_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FB_HEIGHT.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 {
        return;
    }
    
    let visible_rows = height / CHAR_HEIGHT;
    
    let mut scrollback = SCROLLBACK.lock();
    if let Some(ref mut sb) = *scrollback {
        sb.scroll_offset = sb.scroll_offset.saturating_sub(lines);
        sb.is_scrolled = sb.scroll_offset > 0;
        
        if sb.is_scrolled {
            let offset = sb.scroll_offset;
            let total = sb.total_lines();
            drop(scrollback);
            redraw_from_scrollback(offset, total, visible_rows);
        } else {
            // Back to live view
            drop(scrollback);
            // Just let normal output resume
        }
    }
}

/// Reset scroll position to bottom (live view)
pub fn scroll_to_bottom() {
    if let Some(ref mut sb) = *SCROLLBACK.lock() {
        sb.scroll_offset = 0;
        sb.is_scrolled = false;
    }
}

/// Check if we're in scrollback mode
pub fn is_scrolled_back() -> bool {
    if let Some(ref sb) = *SCROLLBACK.lock() {
        sb.is_scrolled
    } else {
        false
    }
}

/// Get current scroll info: (offset, total_lines)
pub fn get_scroll_info() -> (usize, usize) {
    if let Some(ref sb) = *SCROLLBACK.lock() {
        (sb.scroll_offset, sb.total_lines())
    } else {
        (0, 0)
    }
}

/// Redraw the screen using scrollback buffer content
fn redraw_from_scrollback(scroll_offset: usize, total_lines: usize, visible_rows: usize) {
    let width = FB_WIDTH.load(Ordering::SeqCst) as usize;
    let cols = width / CHAR_WIDTH;
    
    // Clear screen first
    let bg = CONSOLE.lock().bg_color;
    clear_with_color(bg);
    
    // Calculate which lines to display
    // scroll_offset=0 means we're at the bottom (most recent)
    // scroll_offset=N means we've scrolled up N lines
    let start_line = total_lines.saturating_sub(visible_rows + scroll_offset);
    let end_line = total_lines.saturating_sub(scroll_offset);
    
    let scrollback = SCROLLBACK.lock();
    if let Some(ref sb) = *scrollback {
        for (screen_row, line_idx) in (start_line..end_line).enumerate() {
            if line_idx >= sb.lines.len() {
                continue;
            }
            let line = &sb.lines[line_idx];
            for (col, i) in (0..line.len.min(cols)).enumerate() {
                let c = line.chars[i];
                let (fg, bg) = line.colors[i];
                let px = col * CHAR_WIDTH;
                let py = screen_row * CHAR_HEIGHT;
                draw_char(c, px, py, fg, bg);
            }
        }
    }
    
    // Show scroll indicator at top right
    if scroll_offset > 0 {
        let indicator = format!("-- SCROLL: +{} lines --", scroll_offset);
        let start_col = cols.saturating_sub(indicator.len() + 2);
        for (i, ch) in indicator.chars().enumerate() {
            let px = (start_col + i) * CHAR_WIDTH;
            draw_char(ch, px, 0, 0xFFFFFF00, 0xFF000000); // Yellow on black
        }
    }
}

/// Clear screen with specific color
fn clear_with_color(color: u32) {
    let addr = FB_ADDR.load(Ordering::SeqCst);
    if addr.is_null() {
        return;
    }
    
    let height = FB_HEIGHT.load(Ordering::SeqCst) as usize;
    let pitch = FB_PITCH.load(Ordering::SeqCst) as usize;
    
    for y in 0..height {
        let row = unsafe { addr.add(y * pitch) };
        for x in 0..(pitch / 4) {
            unsafe {
                row.add(x * 4).cast::<u32>().write_volatile(color);
            }
        }
    }
}
