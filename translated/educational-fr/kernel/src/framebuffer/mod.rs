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
use crate::math::fast_sqrt;

/// Framebuffer info stored after initialization
struct FramebufferInformation {
    address: *mut u8,
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

// Static storage - pub for compositor access
pub // Variable atomique — accès thread-safe sans verrou.
static FRAMEBUFFER_ADDRESS: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
pub // Variable atomique — accès thread-safe sans verrou.
static FRAMEBUFFER_WIDTH: AtomicU64 = AtomicU64::new(0);
pub // Variable atomique — accès thread-safe sans verrou.
static FRAMEBUFFER_HEIGHT: AtomicU64 = AtomicU64::new(0);
pub // Variable atomique — accès thread-safe sans verrou.
static FRAMEBUFFER_PITCH: AtomicU64 = AtomicU64::new(0);
pub // Variable atomique — accès thread-safe sans verrou.
static FRAMEBUFFER_BPP: AtomicU64 = AtomicU64::new(32); // bits per pixel (default 32)

// Double buffering
static BACKBUFFER: Mutex<Option<Box<[u32]>>> = Mutex::new(None);
// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static PREVIOUS_FRAME: Mutex<Option<Box<[u32]>>> = Mutex::new(None);
// Variable atomique — accès thread-safe sans verrou.
static USE_BACKBUFFER: AtomicBool = AtomicBool::new(false);

// ==================== FRAME-CACHED BACKBUFFER ACCESS ====================
// Lock the backbuffer ONCE per frame, then all pixel ops use the cached pointer.
// Eliminates ~300k mutex lock/unlock per frame in the matrix rain.
static FRAME_BB_POINTER: AtomicPtr<u32> = AtomicPtr::new(core::ptr::null_mut());
// Variable atomique — accès thread-safe sans verrou.
static FRAME_BB_STRIDE: AtomicU64 = AtomicU64::new(0);
// Variable atomique — accès thread-safe sans verrou.
static FRAME_BB_HEIGHT: AtomicU64 = AtomicU64::new(0);

// ==================== CLIP RECTANGLE ====================
// Global clip rect for window content rendering.
// When enabled, all pixel writes are clipped to this rectangle.
// Uses atomics for safety (accessed from rendering path which could be interrupted).
use core::sync::atomic::AtomicU32;

// Variable atomique — accès thread-safe sans verrou.
static CLIP_ACTIVE: AtomicBool = AtomicBool::new(false);
// Variable atomique — accès thread-safe sans verrou.
static CLIP_X1: AtomicU32 = AtomicU32::new(0);
// Variable atomique — accès thread-safe sans verrou.
static CLIP_Y1: AtomicU32 = AtomicU32::new(0);
// Variable atomique — accès thread-safe sans verrou.
static CLIP_X2: AtomicU32 = AtomicU32::new(u32::MAX);
// Variable atomique — accès thread-safe sans verrou.
static CLIP_Y2: AtomicU32 = AtomicU32::new(u32::MAX);

/// Set the clip rectangle. All pixel writes will be clipped to this area.
pub fn set_clip_rect(x: u32, y: u32, w: u32, h: u32) {
    CLIP_X1.store(x, Ordering::Relaxed);
    CLIP_Y1.store(y, Ordering::Relaxed);
    CLIP_X2.store(x.saturating_add(w), Ordering::Relaxed);
    CLIP_Y2.store(y.saturating_add(h), Ordering::Relaxed);
    CLIP_ACTIVE.store(true, Ordering::Release);
}

/// Clear the clip rectangle (disable clipping).
pub fn clear_clip_rect() {
    CLIP_ACTIVE.store(false, Ordering::Release);
}

/// Check if a pixel is inside the current clip rectangle.
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn clip_test(x: u32, y: u32) -> bool {
    if !CLIP_ACTIVE.load(Ordering::Relaxed) {
        return true;
    }
    x >= CLIP_X1.load(Ordering::Relaxed) && x < CLIP_X2.load(Ordering::Relaxed) &&
    y >= CLIP_Y1.load(Ordering::Relaxed) && y < CLIP_Y2.load(Ordering::Relaxed)
}

/// Call at the start of each frame to cache the backbuffer pointer.
/// All subsequent `put_pixel_fast`/`get_pixel_fast` calls will use this
/// cached pointer with ZERO mutex overhead.
pub fn begin_frame() {
    if let Some(ref mut buffer) = *BACKBUFFER.lock() {
        FRAME_BB_POINTER.store(buffer.as_mut_pointer(), Ordering::Release);
        FRAME_BB_STRIDE.store(FRAMEBUFFER_WIDTH.load(Ordering::Relaxed), Ordering::Release);
        FRAME_BB_HEIGHT.store(FRAMEBUFFER_HEIGHT.load(Ordering::Relaxed), Ordering::Release);
    }
}

/// Call at the end of each frame (before swap_buffers).
pub fn end_frame() {
    FRAME_BB_POINTER.store(core::ptr::null_mut(), Ordering::Release);
}

/// Get raw frame context for batch pixel writes.
/// Returns (ptr, stride_pixels, height) — caller must ensure begin_frame() was called.
/// Eliminates per-pixel atomic loads when writing many pixels in a loop.
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn frame_context() -> (*mut u32, u32, u32) {
    let ptr = FRAME_BB_POINTER.load(Ordering::Relaxed);
    let stride = FRAME_BB_STRIDE.load(Ordering::Relaxed) as u32;
    let height = FRAME_BB_HEIGHT.load(Ordering::Relaxed) as u32;
    (ptr, stride, height)
}

/// Ultra-fast pixel write — uses cached backbuffer pointer, no mutex per call.
/// Must be called between begin_frame() and end_frame().
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn put_pixel_fast(x: u32, y: u32, color: u32) {
    let ptr = FRAME_BB_POINTER.load(Ordering::Relaxed);
    if ptr.is_null() { put_pixel(x, y, color); return; }
    let stride = FRAME_BB_STRIDE.load(Ordering::Relaxed) as u32;
    let height = FRAME_BB_HEIGHT.load(Ordering::Relaxed) as u32;
    if x >= stride || y >= height { return; }
    if !clip_test(x, y) { return; }
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *ptr.add(y as usize * stride as usize + x as usize) = color; }
}

/// Ultra-fast pixel read — uses cached backbuffer pointer, no mutex per call.
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn get_pixel_fast(x: u32, y: u32) -> u32 {
    let ptr = FRAME_BB_POINTER.load(Ordering::Relaxed);
    if ptr.is_null() { return get_pixel(x, y); }
    let stride = FRAME_BB_STRIDE.load(Ordering::Relaxed) as u32;
    let height = FRAME_BB_HEIGHT.load(Ordering::Relaxed) as u32;
    if x >= stride || y >= height { return 0; }
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *ptr.add(y as usize * stride as usize + x as usize) }
}

// ==================== SCROLLBACK BUFFER ====================
// Store terminal history for scroll up/down functionality
const SCROLLBACK_MAXIMUM_LINES: usize = 1000;  // Store up to 1000 lines of history
const SCROLLBACK_LINE_WIDTH: usize = 256; // Max characters per line

/// A single line in the scrollback buffer
#[derive(Clone)]
struct ScrollbackLine {
    chars: [char; SCROLLBACK_LINE_WIDTH],
    colors: [(u32, u32); SCROLLBACK_LINE_WIDTH], // (fg, bg) for each char
    len: usize,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
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

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl ScrollbackBuffer {
    fn new() -> Self {
        ScrollbackBuffer {
            lines: Vec::with_capacity(SCROLLBACK_MAXIMUM_LINES),
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
        } else if c == '\x08' {
            // Backspace — remove last character from current line
            if self.current_line.len > 0 {
                self.current_line.len -= 1;
            }
        } else if c.is_ascii_graphic() || c == ' ' {
            if self.current_line.len < SCROLLBACK_LINE_WIDTH {
                self.current_line.chars[self.current_line.len] = c;
                self.current_line.colors[self.current_line.len] = (fg, bg);
                self.current_line.len += 1;
            }
        }
    }
    
    /// Commit current line to history
    fn commit_line(&mut self) {
        if self.lines.len() >= SCROLLBACK_MAXIMUM_LINES {
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

// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static SCROLLBACK: Mutex<Option<ScrollbackBuffer>> = Mutex::new(None);
// Variable atomique — accès thread-safe sans verrou.
static SCROLLBACK_ENABLED: AtomicBool = AtomicBool::new(false);

// ==================== BACKGROUND CACHE ====================
// Cache the static background to avoid recomputing every frame
static BACKGROUND_CACHE: Mutex<Option<Box<[u32]>>> = Mutex::new(None);
// Variable atomique — accès thread-safe sans verrou.
static BACKGROUND_VALID: AtomicBool = AtomicBool::new(false);

// ==================== DIRTY RECTANGLES ====================
// Only redraw regions that have changed
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAXIMUM_DIRTY_RECTS: usize = 32;

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy, Default)]
// Structure publique — visible à l'extérieur de ce module.
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
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
        
        let x1 = self.x.minimum(other.x);
        let y1 = self.y.minimum(other.y);
        let x2 = (self.x + self.w).maximum(other.x + other.w);
        let y2 = (self.y + self.h).maximum(other.y + other.h);
        
        DirtyRect { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }
    }
    
    /// Check if overlaps with another rectangle
    pub fn overlaps(&self, other: &DirtyRect) -> bool {
        !(self.x + self.w <= other.x || other.x + other.w <= self.x ||
          self.y + self.h <= other.y || other.y + other.h <= self.y)
    }
}

struct DirtyRectList {
    rects: [DirtyRect; MAXIMUM_DIRTY_RECTS],
    count: usize,
    full_redraw: bool,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl DirtyRectList {
    const fn new() -> Self {
        DirtyRectList {
            rects: [DirtyRect { x: 0, y: 0, w: 0, h: 0 }; MAXIMUM_DIRTY_RECTS],
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
        if self.count < MAXIMUM_DIRTY_RECTS {
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

// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static DIRTY_RECTS: Mutex<DirtyRectList> = Mutex::new(DirtyRectList::new());

// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static CONSOLE: Mutex<Console> = Mutex::new(Console {
    cursor_x: 0,
    cursor_y: 0,
    fg_color: 0xFFFFFFFF, // White
    bg_color: 0xFF000000, // Black
});

/// Character dimensions
const CHAR_WIDTH: usize = 8;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CHAR_HEIGHT: usize = 16;

/// Maximum framebuffer size for backbuffer allocation (16 MB = ~2560×1600×4)
const MAXIMUM_BACKBUFFER_BYTES: usize = 16 * 1024 * 1024;

/// Initialize framebuffer with Limine info
/// NOTE: This does NOT allocate memory. Call init_scrollback() after heap is ready.
pub fn init(address: *mut u8, width: u64, height: u64, pitch: u64, bpp: u16) {
    // Safety: only support 32bpp framebuffers (all pixel code uses u32)
    if bpp != 32 {
        crate::serial_println!("[FB] WARNING: unsupported bpp={}, forcing 32bpp interpretation (may have color artifacts)", bpp);
    }

    FRAMEBUFFER_ADDRESS.store(address, Ordering::SeqCst);
    FRAMEBUFFER_WIDTH.store(width, Ordering::SeqCst);
    FRAMEBUFFER_HEIGHT.store(height, Ordering::SeqCst);
    FRAMEBUFFER_PITCH.store(pitch, Ordering::SeqCst);
    FRAMEBUFFER_BPP.store(bpp as u64, Ordering::SeqCst);
    
    // NOTE: Do NOT call init_scrollback() here! It allocates memory.
    // Call it after heap initialization in main.rs
    
    // Clear screen (no allocation - just writes to framebuffer directly)
    clear();
}

/// Initialize the scrollback buffer
pub fn initialize_scrollback() {
    *SCROLLBACK.lock() = Some(ScrollbackBuffer::new());
    SCROLLBACK_ENABLED.store(true, Ordering::SeqCst);
    crate::serial_println!("[FB] Scrollback buffer initialized ({} lines max)", SCROLLBACK_MAXIMUM_LINES);
}

/// Check if framebuffer is initialized
pub fn is_initialized() -> bool {
    !FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst).is_null()
}

/// Get framebuffer width
pub fn width() -> u32 {
    FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as u32
}

/// Get framebuffer height
pub fn height() -> u32 {
    FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as u32
}

/// Get raw framebuffer pointer as u32 pixels
pub fn get_framebuffer() -> *mut u32 {
    FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst) as *mut u32
}

/// Clear the screen
pub fn clear() {
    let address = FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst);
    if address.is_null() {
        return;
    }
    
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    let pitch = FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize;
    let bg_color = CONSOLE.lock().bg_color;
    
    for y in 0..height {
        let row = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { address.add(y * pitch) };
        for x in 0..(pitch / 4) {
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
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
    let address = FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst);
    if address.is_null() {
        return;
    }
    
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as u32;
    let pitch = FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize;
    
    if x >= width || y >= height {
        return;
    }
    if !clip_test(x, y) { return; }
    
    // Write to backbuffer if enabled, otherwise direct to framebuffer
    if USE_BACKBUFFER.load(Ordering::SeqCst) {
        if let Some(ref mut buffer) = *BACKBUFFER.lock() {
            let index = y as usize * width as usize + x as usize;
            if index < buffer.len() {
                buffer[index] = color;
            }
        }
    } else {
        let offset = y as usize * pitch + x as usize * 4;
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            address.add(offset).cast::<u32>().write_volatile(color);
        }
    }
}

/// Get a pixel at (x, y) - for alpha blending and image compositing
pub fn get_pixel(x: u32, y: u32) -> u32 {
    let address = FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst);
    if address.is_null() {
        return 0;
    }
    
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as u32;
    let pitch = FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize;
    
    if x >= width || y >= height {
        return 0;
    }
    
    // Read from backbuffer if enabled, otherwise from framebuffer
    if USE_BACKBUFFER.load(Ordering::SeqCst) {
        if let Some(ref buffer) = *BACKBUFFER.lock() {
            let index = y as usize * width as usize + x as usize;
            if index < buffer.len() {
                return buffer[index];
            }
        }
        0
    } else {
        let offset = y as usize * pitch + x as usize * 4;
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            address.add(offset).cast::<u32>().read_volatile()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FAST PIXEL CONTEXT - Cache framebuffer params to avoid atomic loads per pixel
// ═══════════════════════════════════════════════════════════════════════════════

/// Cached framebuffer context for fast pixel operations
/// Use when drawing many pixels to avoid 4 atomic loads per put_pixel call
pub struct FastPixelContext {
    pub address: *mut u8,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub backbuffer: bool,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl FastPixelContext {
    /// Create a new fast pixel context by caching current FB state
    #[inline]
        // Fonction publique — appelable depuis d'autres modules.
pub fn new() -> Self {
        FastPixelContext {
            address: FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst),
            width: FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize,
            height: FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize,
            pitch: FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize,
            backbuffer: USE_BACKBUFFER.load(Ordering::SeqCst),
        }
    }
    
    /// Put pixel (no bounds check for maximum speed)
    /// Safety: Caller must ensure x < width and y < height
    #[inline(always)]
    pub     // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn put_pixel_unchecked(&self, x: usize, y: usize, color: u32) {
        if self.backbuffer {
            // Fast path for backbuffer
            if let Some(ref mut buffer) = *BACKBUFFER.lock() {
                let index = y * self.width + x;
                *buffer.get_unchecked_mut(index) = color;
            }
        } else {
            let offset = y * self.pitch + x * 4;
            (self.address.add(offset) as *mut u32).write_volatile(color);
        }
    }
    
    /// Put pixel with bounds check
    #[inline(always)]
        // Fonction publique — appelable depuis d'autres modules.
pub fn put_pixel(&self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height { return; }
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { self.put_pixel_unchecked(x, y, color); }
    }
    
    /// Fill a horizontal span of pixels
    #[inline]
        // Fonction publique — appelable depuis d'autres modules.
pub fn fill_hspan(&self, x: usize, y: usize, len: usize, color: u32) {
        if y >= self.height || x >= self.width { return; }
        let actual_length = len.minimum(self.width - x);
        
        if self.backbuffer {
            if let Some(ref mut buffer) = *BACKBUFFER.lock() {
                let start = y * self.width + x;
                #[cfg(target_arch = "x86_64")]
                                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                    crate::graphics::simd::fill_row_sse2(
                        buffer.as_mut_pointer().add(start),
                        actual_length,
                        color
                    );
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    buffer[start..start + actual_length].fill(color);
                }
            }
        } else {
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                let ptr = (self.address.add(y * self.pitch + x * 4)) as *mut u32;
                #[cfg(target_arch = "x86_64")]
                {
                    crate::graphics::simd::fill_row_sse2(ptr, actual_length, color);
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    for i in 0..actual_length {
                        ptr.add(i).write_volatile(color);
                    }
                }
            }
        }
    }
    
    /// Get a mutable slice to the backbuffer (if enabled)
    /// Returns None if backbuffer is not available
    pub fn get_backbuffer_slice(&self) -> Option<alloc::boxed::Box<[u32]>> {
        if self.backbuffer {
            if let Some(ref buffer) = *BACKBUFFER.lock() {
                // We need to clone, can't return reference
                return Some(buffer.clone());
            }
        }
        None
    }
}

/// Get framebuffer dimensions
pub fn get_dimensions() -> (u32, u32) {
    (FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as u32, FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as u32)
}

/// Execute a closure with direct mutable access to the backbuffer.
/// The backbuffer lock is held for the entire duration — zero per-pixel overhead.
/// The closure receives (buffer_ptr, width, height, stride).
/// Returns true if the closure executed, false if backbuffer unavailable.
/// 
/// Usage: `with_backbuffer_mut(|ptr, w, h, s| { unsafe { *ptr.add(y*s+x) = color; } })`
#[inline]
// Fonction publique — appelable depuis d'autres modules.
pub fn with_backbuffer_mut<F: FnOnce(*mut u32, usize, usize, usize)>(f: F) -> bool {
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 { return false; }
    if let Some(ref mut buffer) = *BACKBUFFER.lock() {
        f(buffer.as_mut_pointer(), width, height, width);
        true
    } else {
        false
    }
}

/// Get raw framebuffer address for direct access
pub fn get_framebuffer_address() -> *mut u8 {
    FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst)
}

/// Get framebuffer pitch (bytes per row)
pub fn get_framebuffer_pitch() -> usize {
    FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize
}

// ==================== DOUBLE BUFFERING ====================

/// Initialize double buffering (allocate backbuffer)
pub fn initialize_double_buffer() {
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    
    if width == 0 || height == 0 {
        return;
    }
    
    let size = width * height;
    let size_bytes = size * 4; // u32 per pixel
    
    if size_bytes > MAXIMUM_BACKBUFFER_BYTES {
        crate::serial_println!("[FB] WARNING: Framebuffer {}x{} = {} KB too large for backbuffer (max {} KB), disabling double buffer",
            width, height, size_bytes / 1024, MAXIMUM_BACKBUFFER_BYTES / 1024);
        return;
    }
    
    // Use try_reserve to avoid OOM panic on memory-constrained hardware
    let mut buffer = alloc::vec::Vec::new();
    if buffer.try_reserve_exact(size).is_err() {
        crate::serial_println!("[FB] WARNING: Failed to allocate backbuffer {} KB — OOM, desktop will use direct mode",
            size_bytes / 1024);
        return;
    }
    buffer.resize(size, 0u32);
    let buffer = buffer.into_boxed_slice();
    
    *BACKBUFFER.lock() = Some(buffer);
    
    // Allocate shadow buffer for row-diff MMIO (only copies changed rows)
    let mut previous_buffer = alloc::vec::Vec::new();
    if previous_buffer.try_reserve_exact(size).is_ok() {
        previous_buffer.resize(size, 0u32);
        *PREVIOUS_FRAME.lock() = Some(previous_buffer.into_boxed_slice());
        crate::serial_println!("[FB] Row-diff shadow buffer allocated: {} KB", size_bytes / 1024);
    }
    
    crate::serial_println!("[FB] Double buffer allocated: {}x{} ({} KB)", width, height, size_bytes / 1024);
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
pub fn get_backbuffer_information() -> Option<(*mut u8, u32, u32, u32)> {
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as u32;
    
    if width == 0 || height == 0 {
        return None;
    }
    
    let backbuffer = BACKBUFFER.lock();
    if let Some(ref buffer) = *backbuffer {
        // Return pointer to the boxed slice's data
        let ptr = buffer.as_pointer() as *mut u8;
        Some((ptr, width, height, width)) // stride = width for backbuffer
    } else {
        None
    }
}

/// Swap buffers - copy backbuffer to display
/// Upgrade #1: When VirtIO GPU is available, uses DMA transfer to host GPU
/// instead of slow MMIO writes. Falls back to SSE2 MMIO copy otherwise.
/// Upgrade #5: Collects dirty rects and only transfers changed regions to VirtIO GPU.
/// Upgrade #6: Row-diff MMIO — only copies rows that changed since last frame,
/// reducing MMIO volume by 40-70% for typical desktop animations.
pub fn swap_buffers() {
    let address = FRAMEBUFFER_ADDRESS.load(Ordering::Relaxed);
    let width = FRAMEBUFFER_WIDTH.load(Ordering::Relaxed) as usize;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::Relaxed) as usize;
    let pitch = FRAMEBUFFER_PITCH.load(Ordering::Relaxed) as usize;
    
    if width == 0 || height == 0 { return; }
    
    // ── VirtIO GPU DMA fast path ──
    // Copy backbuffer → GPU backing buffer, then DMA to host display
    // Upgrade #4: Uses double-buffered present for tear-free rendering
    if crate::drivers::virtio_gpu::is_available() {
        // Prefer back buffer (double-buffered), fall back to raw buffer
        let gpu_buffer = crate::drivers::virtio_gpu::get_back_buffer()
            .or_else(|| crate::drivers::virtio_gpu::get_raw_buffer());
        if let Some((gpu_pointer, gpu_w, gpu_h)) = gpu_buffer {
            if let Some(ref buffer) = *BACKBUFFER.lock() {
                let copy_w = width.minimum(gpu_w as usize);
                let copy_h = height.minimum(gpu_h as usize);
                                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                    for y in 0..copy_h {
                        let source = buffer.as_pointer().add(y * width);
                        let destination = gpu_pointer.add(y * gpu_w as usize);
                        #[cfg(target_arch = "x86_64")]
                        crate::graphics::simd::copy_row_sse2(destination, source, copy_w);
                        #[cfg(not(target_arch = "x86_64"))]
                        core::ptr::copy_nonoverlapping(source, destination, copy_w);
                    }
                }
            }
            // DMA transfer + flush + swap (double-buffered when available)
            let _ = crate::drivers::virtio_gpu::present_frame_double_buffered();
            // Skip MMIO fallback — VirtIO GPU DMA is the display path;
            // the redundant full-screen memcpy was halving frame rate.
            return;
        }
    }
    
    // ── MMIO fallback path ──
    if address.is_null() { return; }
    // Use row-diff version: only copies changed rows (40-70% reduction on matrix rain)
    swap_buffers_mmio_diff(address, width, height, pitch);
}

/// MMIO framebuffer copy with row-diff — only copies rows that changed.
/// Compares each row against the previous frame's shadow buffer.
/// On typical matrix rain frames, 40-70% of rows are unchanged and skipped.
fn swap_buffers_mmio_diff(address: *mut u8, width: usize, height: usize, pitch: usize) {
    let bb_guard = BACKBUFFER.lock();
    let mut pf_guard = PREVIOUS_FRAME.lock();
    
    let (bb, pf) = // Correspondance de motifs — branchement exhaustif de Rust.
match (bb_guard.as_ref(), pf_guard.as_mut()) {
        (Some(b), Some(p)) => (b, p),
        // No shadow buffer — fall back to full copy
        (Some(b), None) => {
            for y in 0..height {
                let source_offset = y * width;
                let destination_offset = y * pitch;
                                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                    let source = b.as_pointer().add(source_offset);
                    let destination = address.add(destination_offset) as *mut u32;
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::copy_row_sse2_nt(destination, source, width);
                    #[cfg(not(target_arch = "x86_64"))]
                    core::ptr::copy_nonoverlapping(source, destination, width);
                }
            }
            return;
        }
        _ => return,
    };
    
    for y in 0..height {
        let offset = y * width;
        let bb_row = &bb[offset..offset + width];
        let pf_row = &mut pf[offset..offset + width];
        
        // Fast 64-bit comparison: check 8 pixels at a time
        let mut changed = false;
        let bb8 = bb_row.as_pointer() as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64;
        let pf8 = pf_row.as_pointer() as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64;
        let pairs = width / 2;
        
        // Check in chunks of 8 pairs (64 bytes = 1 cache line) for early exit
        let chunks = pairs / 8;
        let mut i = 0usize;
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            for _ in 0..chunks {
                if *bb8.add(i) != *pf8.add(i)
                    || *bb8.add(i+1) != *pf8.add(i+1)
                    || *bb8.add(i+2) != *pf8.add(i+2)
                    || *bb8.add(i+3) != *pf8.add(i+3)
                    || *bb8.add(i+4) != *pf8.add(i+4)
                    || *bb8.add(i+5) != *pf8.add(i+5)
                    || *bb8.add(i+6) != *pf8.add(i+6)
                    || *bb8.add(i+7) != *pf8.add(i+7)
                {
                    changed = true;
                    break;
                }
                i += 8;
            }
            // Check remaining pairs
            if !changed {
                while i < pairs {
                    if *bb8.add(i) != *pf8.add(i) {
                        changed = true;
                        break;
                    }
                    i += 1;
                }
            }
            // Check odd tail pixel
            if !changed && (width & 1) != 0 {
                if bb_row[width - 1] != pf_row[width - 1] {
                    changed = true;
                }
            }
        }
        
        if changed {
            // Copy row to MMIO VRAM (NT stores — bypass cache for 2-4x faster writes)
            let destination_offset = y * pitch;
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                let source = bb_row.as_pointer();
                let destination = address.add(destination_offset) as *mut u32;
                #[cfg(target_arch = "x86_64")]
                crate::graphics::simd::copy_row_sse2_nt(destination, source, width);
                #[cfg(not(target_arch = "x86_64"))]
                core::ptr::copy_nonoverlapping(source, destination, width);
            }
            // Update shadow buffer for this row
            pf_row.copy_from_slice(bb_row);
        }
    }
}

/// MMIO framebuffer copy — uses non-temporal stores (movnti) for optimal VRAM writes.
/// NT stores bypass CPU cache and use write-combining, which is 2-4x faster for
/// memory-mapped framebuffers (VGA, SVGA, VBox VRAM) and reduces cache pollution.
fn swap_buffers_mmio(address: *mut u8, width: usize, height: usize, pitch: usize) {
    if let Some(ref buffer) = *BACKBUFFER.lock() {
        for y in 0..height {
            let source_offset = y * width;
            let destination_offset = y * pitch;
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                let source = buffer.as_pointer().add(source_offset);
                let destination = address.add(destination_offset) as *mut u32;
                #[cfg(target_arch = "x86_64")]
                crate::graphics::simd::copy_row_sse2_nt(destination, source, width);
                #[cfg(not(target_arch = "x86_64"))]
                core::ptr::copy_nonoverlapping(source, destination, width);
            }
        }
    }
}

/// Get a raw pointer to the backbuffer data (for GPU copy operations)
/// Returns None if backbuffer is not allocated.
pub fn get_backbuffer_pointer() -> Option<*// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32> {
    let bb = BACKBUFFER.lock();
    bb.as_ref().map(|buffer| buffer.as_pointer())
}

/// MMIO-only swap: copy backbuffer to MMIO framebuffer without VirtIO GPU path.
/// Used when VirtIO GPU present is handled separately (dirty rect path).
pub fn swap_buffers_mmio_only() {
    let address = FRAMEBUFFER_ADDRESS.load(Ordering::Relaxed);
    if address.is_null() { return; }
    let width = FRAMEBUFFER_WIDTH.load(Ordering::Relaxed) as usize;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::Relaxed) as usize;
    let pitch = FRAMEBUFFER_PITCH.load(Ordering::Relaxed) as usize;
    if width == 0 || height == 0 { return; }
    swap_buffers_mmio(address, width, height, pitch);
}

/// Clear backbuffer with color (SSE2 optimized)
pub fn clear_backbuffer(color: u32) {
    if let Some(ref mut buffer) = *BACKBUFFER.lock() {
        #[cfg(target_arch = "x86_64")]
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            crate::graphics::simd::fill_row_sse2(buffer.as_mut_pointer(), buffer.len(), color);
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            buffer.fill(color);
        }
    }
}

// ============================================================================
// SMP PARALLEL BLIT - Copy buffer directly to MMIO framebuffer using all cores
// ============================================================================

/// Context for parallel blit operation
#[repr(C)]
struct ParallelBlitContext {
    source: *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32,
    destination: *mut u8,
    source_stride: usize,   // source width in u32s
    destination_pitch: usize,    // destination pitch in bytes
    width: usize,        // copy width in u32s
}

// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Send for ParallelBlitContext {}
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Sync for ParallelBlitContext {}

/// Worker function for parallel blit (called on each core)
fn blit_rows_worker(start: usize, end: usize, data: *mut u8) {
    let context = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &*(data as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ParallelBlitContext) };
    for y in start..end {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let source = context.source.add(y * context.source_stride);
            let destination = context.destination.add(y * context.destination_pitch) as *mut u32;
            #[cfg(target_arch = "x86_64")]
            {
                crate::graphics::simd::copy_row_sse2_nt(destination, source, context.width);
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                core::ptr::copy_nonoverlapping(source, destination, context.width);
            }
        }
    }
}

/// Blit an external buffer directly to the MMIO framebuffer using SMP.
/// Splits the row copies across all available CPU cores.
/// This combines the backbuffer-write + swap into a single parallel operation.
/// Robust: BSP fills any rows that APs missed (timeout protection).
///
/// `src`: pointer to ARGB pixel buffer (w * h)
/// `w`: width in pixels
/// `h`: height in pixels
pub fn blit_to_framebuffer_parallel(source: *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32, w: usize, h: usize) {
    let address = FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst);
    if address.is_null() { return; }

    let framebuffer_w = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    let pitch = FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize;
    let framebuffer_h = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;

    let copy_w = w.minimum(framebuffer_w);
    let copy_h = h.minimum(framebuffer_h);

    let context = ParallelBlitContext {
        source,
        destination: address,
        source_stride: w,
        destination_pitch: pitch,
        width: copy_w,
    };

    // Try SMP parallel blit (sends IPI to wake APs)
    crate::cpu::smp::parallel_for(
        copy_h,
        blit_rows_worker,
        &context as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ParallelBlitContext as *mut u8,
    );

    // Safety net: BSP re-blits ALL rows to fill any that APs missed.
    // Writing the same pixels twice is idempotent, so this is safe.
    // Cost: ~1ms extra on BSP, but guarantees no missing rows.
    blit_rows_worker(0, copy_h, &context as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ParallelBlitContext as *mut u8);
}

/// Draw filled rectangle to backbuffer (SSE2 optimized)
pub fn fill_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as u32;
    
    let x1 = x.minimum(width);
    let y1 = y.minimum(height);
    let x2 = (x + w).minimum(width);
    let y2 = (y + h).minimum(height);
    
    if x2 <= x1 || y2 <= y1 { return; }
    
    // Fast path: use frame-cached pointer if available (zero mutex overhead)
    let ptr = FRAME_BB_POINTER.load(Ordering::Relaxed);
    if !ptr.is_null() {
        let stride = FRAME_BB_STRIDE.load(Ordering::Relaxed) as usize;
        let rect_width = (x2 - x1) as usize;
        for py in y1..y2 {
            let row_start = py as usize * stride + x1 as usize;
            #[cfg(target_arch = "x86_64")]
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                crate::graphics::simd::fill_row_sse2(
                    ptr.add(row_start),
                    rect_width,
                    color
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                for i in 0..rect_width {
                    *ptr.add(row_start + i) = color;
                }
            }
        }
        return;
    }
    
    if USE_BACKBUFFER.load(Ordering::SeqCst) {
        if let Some(ref mut buffer) = *BACKBUFFER.lock() {
            let rect_width = (x2 - x1) as usize;
            for py in y1..y2 {
                let row_start = py as usize * width as usize + x1 as usize;
                if row_start + rect_width <= buffer.len() {
                    // Use SSE2 for each row (much faster)
                    #[cfg(target_arch = "x86_64")]
                                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                        crate::graphics::simd::fill_row_sse2(
                            buffer.as_mut_pointer().add(row_start),
                            rect_width,
                            color
                        );
                    }
                    #[cfg(not(target_arch = "x86_64"))]
                    {
                        buffer[row_start..row_start + rect_width].fill(color);
                    }
                }
            }
        }
    } else {
        for py in y1..y2 {
            for pixel in x1..x2 {
                put_pixel(pixel, py, color);
            }
        }
    }
}

/// Draw a single pixel (bounds checked) - uses backbuffer if enabled
pub fn draw_pixel(x: u32, y: u32, color: u32) {
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as u32;
    
    if x >= width || y >= height { return; }
    
    // Use backbuffer if double buffering is enabled
    if USE_BACKBUFFER.load(Ordering::SeqCst) {
        if let Some(ref mut buffer) = *BACKBUFFER.lock() {
            let offset = y as usize * width as usize + x as usize;
            if offset < buffer.len() {
                buffer[offset] = color;
            }
        }
    } else {
        let address = FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst);
        if address.is_null() { return; }
        let pitch = FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize;
        let offset = y as usize * pitch + x as usize * 4;
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let ptr = address.add(offset) as *mut u32;
            *ptr = color;
        }
    }
}

/// Draw horizontal line (optimized)
pub fn draw_hline(x: u32, y: u32, len: u32, color: u32) {
    fill_rect(x, y, len, 1, color);
}

/// Draw filled rectangle with alpha blending (0=transparent, 255=opaque)
/// Reads existing pixels and blends the given color on top.
pub fn fill_rect_alpha(x: u32, y: u32, w: u32, h: u32, color: u32, alpha: u32) {
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as u32;
    
    let x1 = x.minimum(width);
    let y1 = y.minimum(height);
    let x2 = (x + w).minimum(width);
    let y2 = (y + h).minimum(height);
    if x2 <= x1 || y2 <= y1 { return; }
    
    let alpha = alpha.minimum(255);
    let inv = 255 - alpha;
    let sr = (color >> 16) & 0xFF;
    let sg = (color >> 8) & 0xFF;
    let sb = color & 0xFF;
    
    // Fast path: use frame-cached pointer if available (zero mutex overhead)
    let ptr = FRAME_BB_POINTER.load(Ordering::Relaxed);
    if !ptr.is_null() {
        let stride = FRAME_BB_STRIDE.load(Ordering::Relaxed) as usize;
        for py in y1..y2 {
            let row_pointer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { ptr.add(py as usize * stride + x1 as usize) };
            let row_w = (x2 - x1) as usize;
            #[cfg(target_arch = "x86_64")]
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { crate::graphics::simd::blend_fill_row_sse2(row_pointer, row_w, color, alpha); }
            #[cfg(not(target_arch = "x86_64"))]
            for pixel in x1..x2 {
                let index = py as usize * stride + pixel as usize;
                                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                    let existing = *ptr.add(index);
                    let dr = (existing >> 16) & 0xFF;
                    let dg = (existing >> 8) & 0xFF;
                    let db = existing & 0xFF;
                    let r = ((sr * alpha + dr * inv + 128) >> 8).minimum(255);
                    let g = ((sg * alpha + dg * inv + 128) >> 8).minimum(255);
                    let b = ((sb * alpha + db * inv + 128) >> 8).minimum(255);
                    *ptr.add(index) = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }
        }
        return;
    }
    
    // Fallback: use mutex-locked backbuffer
    if USE_BACKBUFFER.load(Ordering::SeqCst) {
        if let Some(ref mut buffer) = *BACKBUFFER.lock() {
            let buffer_length = buffer.len();
            for py in y1..y2 {
                let row = py as usize * width as usize;
                let row_start = row + x1 as usize;
                let row_w = (x2 - x1) as usize;
                if row_start + row_w <= buffer_length {
                    #[cfg(target_arch = "x86_64")]
                                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { crate::graphics::simd::blend_fill_row_sse2(buffer.as_mut_pointer().add(row_start), row_w, color, alpha); }
                    #[cfg(not(target_arch = "x86_64"))]
                    for pixel in x1..x2 {
                        let index = row + pixel as usize;
                        let existing = buffer[index];
                        let dr = (existing >> 16) & 0xFF;
                        let dg = (existing >> 8) & 0xFF;
                        let db = existing & 0xFF;
                        let r = ((sr * alpha + dr * inv + 128) >> 8).minimum(255);
                        let g = ((sg * alpha + dg * inv + 128) >> 8).minimum(255);
                        let b = ((sb * alpha + db * inv + 128) >> 8).minimum(255);
                        buffer[index] = 0xFF000000 | (r << 16) | (g << 8) | b;
                    }
                }
            }
        }
    } else {
        for py in y1..y2 {
            for pixel in x1..x2 {
                let existing = get_pixel(pixel, py);
                let dr = (existing >> 16) & 0xFF;
                let dg = (existing >> 8) & 0xFF;
                let db = existing & 0xFF;
                let r = ((sr * alpha + dr * inv + 128) >> 8).minimum(255);
                let g = ((sg * alpha + dg * inv + 128) >> 8).minimum(255);
                let b = ((sb * alpha + db * inv + 128) >> 8).minimum(255);
                put_pixel(pixel, py, 0xFF000000 | (r << 16) | (g << 8) | b);
            }
        }
    }
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

/// Draw filled circle (fast Midpoint circle algorithm)
pub fn fill_circle(cx: u32, cy: u32, radius: u32, color: u32) {
    if radius == 0 { return; }
    
    // Fast horizontal line fill using squared distance
    let r2 = (radius * radius) as i32;
    for dy in 0..=radius {
        let dx = fast_sqrt((r2 - (dy * dy) as i32) as f32) as u32;
        if dx > 0 {
            // Upper half
            if cy >= dy {
                fill_rect(cx.saturating_sub(dx), cy - dy, dx * 2 + 1, 1, color);
            }
            // Lower half
            fill_rect(cx.saturating_sub(dx), cy + dy, dx * 2 + 1, 1, color);
        }
    }
}

/// Draw filled rounded rectangle (corners with radius)
pub fn fill_rounded_rect(x: u32, y: u32, w: u32, h: u32, radius: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    
    let r = radius.minimum(w / 2).minimum(h / 2);
    
    if r == 0 {
        fill_rect(x, y, w, h, color);
        return;
    }
    
    // Center rectangle (full width, reduced height)
    fill_rect(x, y + r, w, h - r * 2, color);
    // Top rectangle (reduced width)
    fill_rect(x + r, y, w - r * 2, r, color);
    // Bottom rectangle (reduced width)
    fill_rect(x + r, y + h - r, w - r * 2, r, color);
    
    // Four corners using filled quarter circles
    let r2 = (r * r) as i32;
    for dy in 0..r {
        let dx = fast_sqrt((r2 - (dy * dy) as i32) as f32) as u32;
        if dx > 0 {
            // Top-left corner
            fill_rect(x + r - dx, y + r - dy - 1, dx, 1, color);
            // Top-right corner
            fill_rect(x + w - r, y + r - dy - 1, dx, 1, color);
            // Bottom-left corner
            fill_rect(x + r - dx, y + h - r + dy, dx, 1, color);
            // Bottom-right corner
            fill_rect(x + w - r, y + h - r + dy, dx, 1, color);
        }
    }
}

/// Draw rounded rectangle stroke (outline only)
pub fn stroke_rounded_rect(x: u32, y: u32, w: u32, h: u32, radius: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    
    let r = radius.minimum(w / 2).minimum(h / 2);
    
    if r == 0 {
        draw_rect(x, y, w, h, color);
        return;
    }
    
    // Horizontal edges
    draw_hline(x + r, y, w - r * 2, color);           // Top
    draw_hline(x + r, y + h - 1, w - r * 2, color);   // Bottom
    // Vertical edges
    draw_vline(x, y + r, h - r * 2, color);           // Left
    draw_vline(x + w - 1, y + r, h - r * 2, color);   // Right
    
    // Draw corner arcs using Midpoint circle algorithm
    let mut pixel = r as i32;
    let mut py = 0i32;
    let mut error = 0i32;
    
    while pixel >= py {
        // Top-left corner
        draw_pixel(x + r - pixel as u32, y + r - py as u32, color);
        draw_pixel(x + r - py as u32, y + r - pixel as u32, color);
        // Top-right corner  
        draw_pixel(x + w - 1 - r + pixel as u32, y + r - py as u32, color);
        draw_pixel(x + w - 1 - r + py as u32, y + r - pixel as u32, color);
        // Bottom-left corner
        draw_pixel(x + r - pixel as u32, y + h - 1 - r + py as u32, color);
        draw_pixel(x + r - py as u32, y + h - 1 - r + pixel as u32, color);
        // Bottom-right corner
        draw_pixel(x + w - 1 - r + pixel as u32, y + h - 1 - r + py as u32, color);
        draw_pixel(x + w - 1 - r + py as u32, y + h - 1 - r + pixel as u32, color);
        
        py += 1;
        error += 1 + 2 * py;
        if 2 * (error - pixel) + 1 > 0 {
            pixel -= 1;
            error += 1 - 2 * pixel;
        }
    }
}

/// Draw text using bitmap font with transparent background
pub fn draw_text(text: &str, x: u32, y: u32, color: u32) {
    let mut cx = x;
    for c in text.chars() {
        draw_char_at(cx, y, c, color);
        cx += CHAR_WIDTH as u32;
    }
}

/// Draw a character at pixel position (private)
fn draw_char(c: char, x: usize, y: usize, fg: u32, bg: u32) {
    let glyph = font::get_glyph(c);
    
    for row in 0..CHAR_HEIGHT {
        let bits = glyph[row];
        for column in 0..CHAR_WIDTH {
            let color = if (bits >> (7 - column)) & 1 == 1 { fg } else { bg };
            put_pixel((x + column) as u32, (y + row) as u32, color);
        }
    }
}

/// Draw a character at pixel position with transparent background
/// Optimized: takes lock once instead of per-pixel
pub fn draw_char_at(x: u32, y: u32, c: char, color: u32) {
    let glyph = font::get_glyph(c);
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as u32;
    
    if x >= width || y >= height { return; }
    
    // Fast path: write directly to backbuffer with single lock
    if USE_BACKBUFFER.load(Ordering::SeqCst) {
        if let Some(ref mut buffer) = *BACKBUFFER.lock() {
            let stride = width as usize;
            for row in 0..CHAR_HEIGHT {
                let py = y as usize + row;
                if py >= height as usize { break; }
                let bits = glyph[row];
                let row_offset = py * stride;
                for column in 0..CHAR_WIDTH {
                    if (bits >> (7 - column)) & 1 == 1 {
                        let pixel = x as usize + column;
                        if pixel < width as usize && clip_test(pixel as u32, py as u32) {
                            let offset = row_offset + pixel;
                            if offset < buffer.len() {
                                buffer[offset] = color;
                            }
                        }
                    }
                }
            }
        }
    } else {
        // Fallback: direct framebuffer
        let address = FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst);
        if address.is_null() { return; }
        let pitch = FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize;
        for row in 0..CHAR_HEIGHT {
            let py = y as usize + row;
            if py >= height as usize { break; }
            let bits = glyph[row];
            for column in 0..CHAR_WIDTH {
                if (bits >> (7 - column)) & 1 == 1 {
                    let pixel = x as usize + column;
                    if pixel < width as usize && clip_test(pixel as u32, py as u32) {
                        let offset = py * pitch + pixel * 4;
                                                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                            let ptr = address.add(offset) as *mut u32;
                            *ptr = color;
                        }
                    }
                }
            }
        }
    }
}

/// Write a character to the console
fn write_char(c: char) {
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    
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
    
        // Correspondance de motifs — branchement exhaustif de Rust.
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

            let pixel = console.cursor_x * CHAR_WIDTH;
            let py = console.cursor_y * CHAR_HEIGHT;
            let fg = console.fg_color;
            let bg = console.bg_color;
            drop(console);
            draw_char(' ', pixel, py, fg, bg);
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
            
            let pixel = console.cursor_x * CHAR_WIDTH;
            let py = console.cursor_y * CHAR_HEIGHT;
            
            // Drop lock before drawing (to avoid holding it too long)
            let x = console.cursor_x;
            console.cursor_x += 1;
            drop(console);
            
            draw_char(c, pixel, py, fg, bg);
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
pub fn scroll_up() {
    let address = FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst);
    if address.is_null() {
        return;
    }
    
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    let pitch = FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize;
    let bg_color = CONSOLE.lock().bg_color;
    
    // Copy each line up
    for y in CHAR_HEIGHT..height {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let source = address.add(y * pitch);
            let destination = address.add((y - CHAR_HEIGHT) * pitch);
            core::ptr::copy(source, destination, pitch);
        }
    }
    
    // Clear last line
    for y in (height - CHAR_HEIGHT)..height {
        let row = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { address.add(y * pitch) };
        for x in 0..(pitch / 4) {
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                row.add(x * 4).cast::<u32>().write_volatile(bg_color);
            }
        }
    }
}

/// Writer struct for fmt::Write
pub struct Writer;

// Implémentation de trait — remplit un contrat comportemental.
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            write_char(c);
        }
        Ok(())
    }
}

/// When true, println!/print! skip framebuffer write (serial only).
/// Used by inttest to avoid slow QEMU framebuffer scrolling.
static SERIAL_ONLY_MODE: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

/// Enable serial-only mode (println writes to serial, skips framebuffer)
pub fn set_serial_only(on: bool) {
    SERIAL_ONLY_MODE.store(on, core::sync::atomic::Ordering::Relaxed);
}

/// Internal print function (framebuffer + serial)
#[doc(hidden)]
// Fonction publique — appelable depuis d'autres modules.
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    // When pipe capture mode is active, redirect output to capture buffer
    if crate::shell::is_capturing() {
        let mut s = alloc::string::String::new();
        let _ = core::fmt::write(&mut s, args);
        crate::shell::capture_write(&s);
        return;
    }
    if !SERIAL_ONLY_MODE.load(core::sync::atomic::Ordering::Relaxed) {
        Writer.write_fmt(args).unwrap();
    }
    crate::serial::_print(args);
}

/// Internal print function (framebuffer ONLY, no serial output)
/// Used for UI elements like autocomplete suggestions that shouldn't pollute serial
#[doc(hidden)]
// Fonction publique — appelable depuis d'autres modules.
pub fn _print_framebuffer_only(args: fmt::Arguments) {
    use core::fmt::Write;
    Writer.write_fmt(args).unwrap();
}

/// Print to framebuffer console
#[macro_export]
macro_rules! print {
    ($($argument:tt)*) => {
        $crate::framebuffer::_print(format_args!($($argument)*))
    };
}

/// Print to framebuffer ONLY (no serial)
#[macro_export]
macro_rules! print_framebuffer_only {
    ($($argument:tt)*) => {
        $crate::framebuffer::_print_framebuffer_only(format_args!($($argument)*))
    };
}

/// Print to framebuffer console with newline
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($argument:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($argument)*));
}

// === COLOR CONSTANTS (ARGB format) ===
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLOR_BLACK: u32 = 0xFF000000;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLOR_WHITE: u32 = 0xFFFFFFFF;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLOR_GREEN: u32 = 0xFF00FF00;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLOR_BRIGHT_GREEN: u32 = 0xFF00FF66;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLOR_DARK_GREEN: u32 = 0xFF00AA00;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLOR_RED: u32 = 0xFFFF0000;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLOR_BLUE: u32 = 0xFF0000FF;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLOR_YELLOW: u32 = 0xFFFFFF00;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLOR_CYAN: u32 = 0xFF00FFFF;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLOR_MAGENTA: u32 = 0xFFFF00FF;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLOR_GRAY: u32 = 0xFF888888;

/// Print with specific color (temporarily changes fg, then restores)
#[macro_export]
macro_rules! print_color {
    ($color:expr, $($argument:tt)*) => {{
        let old = $crate::framebuffer::get_fg_color();
        $crate::framebuffer::set_fg_color($color);
        $crate::print!($($argument)*);
        $crate::framebuffer::set_fg_color(old);
    }};
}

/// Print with color and newline
#[macro_export]
macro_rules! println_color {
    ($color:expr, $($argument:tt)*) => {{
        let old = $crate::framebuffer::get_fg_color();
        $crate::framebuffer::set_fg_color($color);
        $crate::println!($($argument)*);
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
        let pixel = x + (i as u32) * CHAR_WIDTH as u32;
        draw_char(c, pixel as usize, y as usize, fg, bg);
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

/// Clear a character row directly with background pixels — bypasses Writer/scrollback.
/// Use this for UI decorations (suggestions, etc.) that shouldn't affect scrollback state.
pub fn clear_char_row(row: usize) {
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 { return; }
    let py = row * CHAR_HEIGHT;
    if py + CHAR_HEIGHT > height { return; }
    let bg = CONSOLE.lock().bg_color;
    fill_rect(0, py as u32, width as u32, CHAR_HEIGHT as u32, bg);
}

/// Draw text at a character row/col directly — bypasses Writer/scrollback.
/// Returns the number of characters drawn.
pub fn draw_text_raw(column: usize, row: usize, text: &str, fg: u32, bg: u32) -> usize {
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 { return 0; }
    let cols = width / CHAR_WIDTH;
    let py = row * CHAR_HEIGHT;
    if py + CHAR_HEIGHT > height { return 0; }
    let mut count = 0;
    for (i, character) in text.chars().enumerate() {
        let c = column + i;
        if c >= cols { break; }
        draw_char(character, c * CHAR_WIDTH, py, fg, bg);
        count += 1;
    }
    count
}

/// Set cursor position (in character coordinates)
pub fn set_cursor(column: usize, row: usize) {
    let mut console = CONSOLE.lock();
    console.cursor_x = column;
    console.cursor_y = row;
}

/// Get cursor position
pub fn get_cursor() -> (usize, usize) {
    let console = CONSOLE.lock();
    (console.cursor_x, console.cursor_y)
}

/// Draw progress bar
pub fn draw_progress_bar(x: u32, y: u32, width: u32, progress: u32, fg: u32, bg: u32) {
    let filled = (width * progress.minimum(100)) / 100;
    
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
pub fn print_boot_status(message: &str, status: BootStatus) {
    let (status_str, color) = // Correspondance de motifs — branchement exhaustif de Rust.
match status {
        BootStatus::Ok => ("[OK]", COLOR_GREEN),
        BootStatus::Skip => ("[--]", COLOR_GRAY),
        BootStatus::Fail => ("[!!]", COLOR_RED),
        BootStatus::Information => ("[..]", COLOR_CYAN),
    };
    
    // Print status with color
    let old_fg = get_fg_color();
    set_fg_color(color);
    crate::print!("{} ", status_str);
    set_fg_color(old_fg);
    crate::println!("{}", message);
}

/// Boot status enum for styled messages
#[derive(Clone, Copy)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum BootStatus {
    Ok,
    Skip,
    Fail,
    Information,
}

/// Draw the boot splash screen with logo
pub fn draw_boot_splash() {
    logo::draw_boot_splash();
}

/// Initialize the graphical boot splash (logo + progress bar frame)
pub fn initialize_boot_splash() {
    logo::initialize_boot_splash();
}

/// Update boot splash progress bar and phase message
pub fn update_boot_splash(phase: u32, message: &str) {
    logo::update_boot_splash(phase, message);
}

/// Fade out splash screen before transitioning to shell
pub fn fade_out_splash() {
    logo::fade_out_splash();
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
pub fn initialize_background_cache() {
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    
    if width == 0 || height == 0 {
        return;
    }
    
    let size = width * height;
    // Use try_reserve to avoid OOM panic on memory-constrained hardware
    let mut buffer = alloc::vec::Vec::new();
    if buffer.try_reserve_exact(size).is_err() {
        crate::serial_println!("[FB] WARNING: Failed to allocate background cache {} KB — OOM",
            size * 4 / 1024);
        return;
    }
    buffer.resize(size, 0u32);
    let buffer = buffer.into_boxed_slice();
    
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
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    
    let bg_guard = BACKGROUND_CACHE.lock();
    if let Some(ref bg_buffer) = *bg_guard {
        if let Some(ref mut back_buffer) = *BACKBUFFER.lock() {
            // Fast copy entire background
            let len = bg_buffer.len().minimum(back_buffer.len());
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                core::ptr::copy_nonoverlapping(bg_buffer.as_pointer(), back_buffer.as_mut_pointer(), len);
            }
        }
    }
}

/// Restore only a rectangular region from background cache
pub fn restore_background_rect(x: u32, y: u32, w: u32, h: u32) {
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as u32;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as u32;
    
    let x1 = x.minimum(width);
    let y1 = y.minimum(height);
    let x2 = (x + w).minimum(width);
    let y2 = (y + h).minimum(height);
    
    let bg_guard = BACKGROUND_CACHE.lock();
    if let Some(ref bg_buffer) = *bg_guard {
        if let Some(ref mut back_buffer) = *BACKBUFFER.lock() {
            for py in y1..y2 {
                let source_start = py as usize * width as usize + x1 as usize;
                let source_end = py as usize * width as usize + x2 as usize;
                let destination_start = source_start;
                
                if source_end <= bg_buffer.len() && source_end <= back_buffer.len() {
                                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                        let source = bg_buffer.as_pointer().add(source_start);
                        let destination = back_buffer.as_mut_pointer().add(destination_start);
                        core::ptr::copy_nonoverlapping(source, destination, (x2 - x1) as usize);
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
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    let size = width * height;
    
    if size == 0 { return; }
    
    // Copy from backbuffer to background cache
    // Must lock both, but be careful about order
    let mut bg_guard = BACKGROUND_CACHE.lock();
    let back_guard = BACKBUFFER.lock();
    
    if let (Some(ref mut bg_buffer), Some(ref back_buffer)) = (&mut *bg_guard, &*back_guard) {
        let len = back_buffer.len().minimum(bg_buffer.len());
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            core::ptr::copy_nonoverlapping(back_buffer.as_pointer(), bg_buffer.as_mut_pointer(), len);
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
pub fn get_dirty_rects() -> ([DirtyRect; MAXIMUM_DIRTY_RECTS], usize, bool) {
    let guard = DIRTY_RECTS.lock();
    (guard.rects, guard.count, guard.full_redraw)
}

/// Swap only dirty regions to framebuffer (optimized swap)
pub fn swap_dirty_regions() {
    let address = FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst);
    if address.is_null() {
        return;
    }
    
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    let pitch = FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize;
    
    let (rects, count, full_redraw) = get_dirty_rects();
    
    if full_redraw {
        // Fall back to full swap
        swap_buffers();
        clear_dirty_rects();
        return;
    }
    
    if let Some(ref buffer) = *BACKBUFFER.lock() {
        for i in 0..count {
            let rect = &rects[i];
            if !rect.is_valid() { continue; }
            
            let x1 = (rect.x as usize).minimum(width);
            let y1 = (rect.y as usize).minimum(height);
            let x2 = ((rect.x + rect.w) as usize).minimum(width);
            let y2 = ((rect.y + rect.h) as usize).minimum(height);
            
            for y in y1..y2 {
                let source_offset = y * width + x1;
                let destination_offset = y * pitch / 4 + x1; // pitch is in bytes, divide by 4 for u32
                let len = x2 - x1;
                
                                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                    let source = buffer.as_pointer().add(source_offset);
                    let destination = (address as *mut u32).add(y * pitch / 4 + x1);
                    core::ptr::copy_nonoverlapping(source, destination, len);
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
pub fn get_cursor_column() -> usize {
    CONSOLE.lock().cursor_x
}

/// Scroll the terminal view up (PageUp) - shows older content
pub fn scroll_up_lines(lines: usize) {
    if !SCROLLBACK_ENABLED.load(Ordering::SeqCst) {
        return;
    }
    
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 {
        return;
    }
    
    let visible_rows = height / CHAR_HEIGHT;
    
    let mut scrollback = SCROLLBACK.lock();
    if let Some(ref mut sb) = *scrollback {
        let maximum_scroll = sb.total_lines().saturating_sub(visible_rows);
        sb.scroll_offset = (sb.scroll_offset + lines).minimum(maximum_scroll);
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
    
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 {
        return;
    }
    
    let visible_rows = height / CHAR_HEIGHT;
    
    let mut scrollback = SCROLLBACK.lock();
    if let Some(ref mut sb) = *scrollback {
        sb.scroll_offset = sb.scroll_offset.saturating_sub(lines);
        sb.is_scrolled = sb.scroll_offset > 0;
        
        let offset = sb.scroll_offset;
        let total = sb.total_lines();
        drop(scrollback);
        // Redraw at new position (including live view at offset=0)
        redraw_from_scrollback(offset, total, visible_rows);
    }
}

/// Reset scroll position to bottom (live view)
pub fn scroll_to_bottom() {
    if let Some(ref mut sb) = *SCROLLBACK.lock() {
        sb.scroll_offset = 0;
        sb.is_scrolled = false;
    }
}

/// Restore live view: snap to bottom and fully redraw the screen
/// Returns the (col, row) of the cursor after redraw
pub fn restore_live_view() -> (usize, usize) {
    let total = {
        let mut scrollback = SCROLLBACK.lock();
        if let Some(ref mut sb) = *scrollback {
            sb.scroll_offset = 0;
            sb.is_scrolled = false;
            sb.total_lines()
        } else {
            return (0, 0);
        }
    };

    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    if height == 0 {
        return (0, 0);
    }
    let visible_rows = height / CHAR_HEIGHT;
    redraw_from_scrollback(0, total, visible_rows);

    let console = CONSOLE.lock();
    (console.cursor_x, console.cursor_y)
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
pub fn get_scroll_information() -> (usize, usize) {
    if let Some(ref sb) = *SCROLLBACK.lock() {
        (sb.scroll_offset, sb.total_lines())
    } else {
        (0, 0)
    }
}

/// Redraw the screen using scrollback buffer content
fn redraw_from_scrollback(scroll_offset: usize, total_lines: usize, visible_rows: usize) {
    let width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
    let cols = width / CHAR_WIDTH;
    
    // Clear screen first
    let bg = CONSOLE.lock().bg_color;
    clear_with_color(bg);
    
    // Calculate which lines to display
    // scroll_offset=0 means we're at the bottom (most recent)
    // scroll_offset=N means we've scrolled up N lines
    //
    // At live view (offset=0), reserve the last row for the current_line
    // (prompt + typed input). Without this, when total_lines >= visible_rows,
    // all rows are filled with history and the input line is never drawn.
    let history_rows = if scroll_offset == 0 {
        visible_rows.saturating_sub(1)
    } else {
        visible_rows
    };
    let start_line = total_lines.saturating_sub(history_rows + scroll_offset);
    let end_line = total_lines.saturating_sub(scroll_offset);
    
    let mut live_cursor_column = 0usize;
    let mut live_cursor_row = 0usize;
    let mut has_current_line = false;
    
    let scrollback = SCROLLBACK.lock();
    if let Some(ref sb) = *scrollback {
        for (screen_row, line_index) in (start_line..end_line).enumerate() {
            if line_index >= sb.lines.len() {
                continue;
            }
            let line = &sb.lines[line_index];
            for (column, i) in (0..line.len.minimum(cols)).enumerate() {
                let c = line.chars[i];
                let (fg, bg) = line.colors[i];
                let pixel = column * CHAR_WIDTH;
                let py = screen_row * CHAR_HEIGHT;
                draw_char(c, pixel, py, fg, bg);
            }
        }
        
        // When at live view (scroll_offset=0), also draw the current uncommitted line
        // This is the prompt + any partially typed input
        if scroll_offset == 0 {
            let screen_row = end_line.saturating_sub(start_line);
            if screen_row < visible_rows && sb.current_line.len > 0 {
                for column in 0..sb.current_line.len.minimum(cols) {
                    let c = sb.current_line.chars[column];
                    let (fg, bg) = sb.current_line.colors[column];
                    let pixel = column * CHAR_WIDTH;
                    let py = screen_row * CHAR_HEIGHT;
                    draw_char(c, pixel, py, fg, bg);
                }
                live_cursor_row = screen_row;
                live_cursor_column = sb.current_line.len.minimum(cols);
                has_current_line = true;
            } else if screen_row < visible_rows {
                // Empty current line - cursor at start of row
                live_cursor_row = screen_row;
                live_cursor_column = 0;
                has_current_line = true;
            }
        }
    }
    drop(scrollback);
    
    // Update cursor position for live view
    if scroll_offset == 0 && has_current_line {
        let mut console = CONSOLE.lock();
        console.cursor_y = live_cursor_row;
        console.cursor_x = live_cursor_column;
    }
    
    // Show scroll indicator at top right
    if scroll_offset > 0 {
        let indicator = format!("-- SCROLL: +{} lines --", scroll_offset);
        let start_column = cols.saturating_sub(indicator.len() + 2);
        for (i, character) in indicator.chars().enumerate() {
            let pixel = (start_column + i) * CHAR_WIDTH;
            draw_char(character, pixel, 0, 0xFFFFFF00, 0xFF000000); // Yellow on black
        }
    }
}

/// Clear screen with specific color
fn clear_with_color(color: u32) {
    let address = FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst);
    if address.is_null() {
        return;
    }
    
    let height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
    let pitch = FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize;
    
    for y in 0..height {
        let row = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { address.add(y * pitch) };
        for x in 0..(pitch / 4) {
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                row.add(x * 4).cast::<u32>().write_volatile(color);
            }
        }
    }
}
