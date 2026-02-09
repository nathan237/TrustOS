//! Fast Software Rendering Engine
//!
//! Optimized 2D rendering techniques used by modern OS:
//! - SIMD-like batch pixel operations (8 pixels at a time)
//! - Dirty rectangle tracking
//! - Font glyph caching
//! - Optimized alpha blending
//! - Row-based memcpy for blitting

use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Process 8 pixels at a time for better cache utilization
const PIXEL_BATCH_SIZE: usize = 8;

/// Maximum dirty rectangles before forcing full redraw
const MAX_DIRTY_RECTS: usize = 64;

/// Font cache size (ASCII printable characters)
const FONT_CACHE_SIZE: usize = 128;

// ═══════════════════════════════════════════════════════════════════════════════
// FAST SURFACE - Optimized pixel buffer
// ═══════════════════════════════════════════════════════════════════════════════

/// A fast software-rendered surface
pub struct FastSurface {
    /// Pixel data (ARGB8888)
    pub data: Box<[u32]>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels  
    pub height: u32,
    /// Dirty region tracking
    dirty: DirtyRegion,
}

impl FastSurface {
    /// Create a new surface
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            data: alloc::vec![0u32; size].into_boxed_slice(),
            width,
            height,
            dirty: DirtyRegion::new(width, height),
        }
    }

    /// Clear entire surface (SSE2 optimized)
    #[inline]
    pub fn clear(&mut self, color: u32) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            crate::graphics::simd::fill_row_sse2(
                self.data.as_mut_ptr(),
                self.data.len(),
                color
            );
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.data.fill(color);
        }
        self.dirty.mark_full();
    }

    /// Fill rectangle - SSE2 optimized row-based fill
    #[inline]
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        let x1 = x.max(0) as u32;
        let y1 = y.max(0) as u32;
        let x2 = ((x + w as i32) as u32).min(self.width);
        let y2 = ((y + h as i32) as u32).min(self.height);
        
        if x2 <= x1 || y2 <= y1 { return; }
        
        let row_width = (x2 - x1) as usize;
        let stride = self.width as usize;
        
        for py in y1..y2 {
            let row_start = py as usize * stride + x1 as usize;
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::fill_row_sse2(
                    self.data.as_mut_ptr().add(row_start),
                    row_width,
                    color
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                self.data[row_start..row_start + row_width].fill(color);
            }
        }
        
        self.dirty.add_rect(x1, y1, x2 - x1, y2 - y1);
    }

    /// Set single pixel
    #[inline(always)]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = color;
        }
    }

    /// Get single pixel
    #[inline(always)]
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize]
        } else {
            0
        }
    }

    /// Draw horizontal line (very fast - single fill)
    #[inline]
    pub fn hline(&mut self, x: i32, y: i32, len: u32, color: u32) {
        if y < 0 || y >= self.height as i32 { return; }
        
        let x1 = x.max(0) as u32;
        let x2 = ((x + len as i32) as u32).min(self.width);
        if x2 <= x1 { return; }
        
        let y = y as u32;
        let start = (y * self.width + x1) as usize;
        let end = (y * self.width + x2) as usize;
        self.data[start..end].fill(color);
        
        self.dirty.add_rect(x1, y, x2 - x1, 1);
    }

    /// Draw vertical line
    #[inline]
    pub fn vline(&mut self, x: i32, y: i32, len: u32, color: u32) {
        if x < 0 || x >= self.width as i32 { return; }
        
        let y1 = y.max(0) as u32;
        let y2 = ((y + len as i32) as u32).min(self.height);
        if y2 <= y1 { return; }
        
        let x = x as u32;
        let stride = self.width;
        for py in y1..y2 {
            self.data[(py * stride + x) as usize] = color;
        }
        
        self.dirty.add_rect(x, y1, 1, y2 - y1);
    }

    /// Draw rectangle outline
    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        self.hline(x, y, w, color);
        self.hline(x, y + h as i32 - 1, w, color);
        self.vline(x, y, h, color);
        self.vline(x + w as i32 - 1, y, h, color);
    }

    /// Blit from another surface (optimized row copy)
    pub fn blit(&mut self, src: &FastSurface, dst_x: i32, dst_y: i32) {
        self.blit_region(src, 0, 0, src.width, src.height, dst_x, dst_y);
    }

    /// Blit a region from another surface
    pub fn blit_region(&mut self, src: &FastSurface, 
                       src_x: u32, src_y: u32, src_w: u32, src_h: u32,
                       dst_x: i32, dst_y: i32) {
        // Clip source region
        let sx1 = src_x.min(src.width);
        let sy1 = src_y.min(src.height);
        let sx2 = (src_x + src_w).min(src.width);
        let sy2 = (src_y + src_h).min(src.height);
        
        if sx2 <= sx1 || sy2 <= sy1 { return; }
        
        // Clip destination
        let mut dx = dst_x;
        let mut dy = dst_y;
        let mut copy_w = (sx2 - sx1) as i32;
        let mut copy_h = (sy2 - sy1) as i32;
        let mut src_offset_x = 0i32;
        let mut src_offset_y = 0i32;
        
        // Clip left
        if dx < 0 {
            src_offset_x = -dx;
            copy_w += dx;
            dx = 0;
        }
        // Clip top
        if dy < 0 {
            src_offset_y = -dy;
            copy_h += dy;
            dy = 0;
        }
        // Clip right
        if dx + copy_w > self.width as i32 {
            copy_w = self.width as i32 - dx;
        }
        // Clip bottom
        if dy + copy_h > self.height as i32 {
            copy_h = self.height as i32 - dy;
        }
        
        if copy_w <= 0 || copy_h <= 0 { return; }
        
        let copy_w = copy_w as usize;
        let copy_h = copy_h as usize;
        let dx = dx as usize;
        let dy = dy as usize;
        let actual_sx = (sx1 as i32 + src_offset_x) as usize;
        let actual_sy = (sy1 as i32 + src_offset_y) as usize;
        
        // Copy row by row using SSE2
        let src_stride = src.width as usize;
        let dst_stride = self.width as usize;
        
        for row in 0..copy_h {
            let src_row_start = (actual_sy + row) * src_stride + actual_sx;
            let dst_row_start = (dy + row) * dst_stride + dx;
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::copy_row_sse2(
                    self.data.as_mut_ptr().add(dst_row_start),
                    src.data.as_ptr().add(src_row_start),
                    copy_w
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                self.data[dst_row_start..dst_row_start + copy_w]
                    .copy_from_slice(&src.data[src_row_start..src_row_start + copy_w]);
            }
        }
        
        self.dirty.add_rect(dx as u32, dy as u32, copy_w as u32, copy_h as u32);
    }

    /// Blit with alpha blending (SSE2 optimized)
    pub fn blit_alpha(&mut self, src: &FastSurface, dst_x: i32, dst_y: i32) {
        let dx_start = dst_x.max(0) as u32;
        let dy_start = dst_y.max(0) as u32;
        let src_x_start = if dst_x < 0 { (-dst_x) as u32 } else { 0 };
        let src_y_start = if dst_y < 0 { (-dst_y) as u32 } else { 0 };
        
        let copy_w = (src.width - src_x_start).min(self.width - dx_start);
        let copy_h = (src.height - src_y_start).min(self.height - dy_start);
        
        if copy_w == 0 || copy_h == 0 { return; }
        
        let src_stride = src.width as usize;
        let dst_stride = self.width as usize;
        
        for row in 0..copy_h {
            let src_row = (src_y_start + row) as usize * src_stride;
            let dst_row = (dy_start + row) as usize * dst_stride;
            
            // Use SSE2 blend when available
            #[cfg(target_arch = "x86_64")]
            unsafe {
                let src_ptr = src.data.as_ptr().add(src_row + src_x_start as usize);
                let dst_ptr = self.data.as_mut_ptr().add(dst_row + dx_start as usize);
                crate::graphics::simd::blend_row_sse2(dst_ptr, src_ptr, copy_w as usize);
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                // Process 8 pixels at a time when possible
                let mut x = 0u32;
                while x + 8 <= copy_w {
                    for i in 0..8 {
                        let src_idx = src_row + (src_x_start + x + i) as usize;
                        let dst_idx = dst_row + (dx_start + x + i) as usize;
                        let src_pixel = src.data[src_idx];
                        let alpha = src_pixel >> 24;
                        
                        if alpha == 255 {
                            self.data[dst_idx] = src_pixel;
                        } else if alpha > 0 {
                            self.data[dst_idx] = blend_fast(src_pixel, self.data[dst_idx]);
                        }
                    }
                    x += 8;
                }
                
                // Handle remaining pixels
                while x < copy_w {
                    let src_idx = src_row + (src_x_start + x) as usize;
                    let dst_idx = dst_row + (dx_start + x) as usize;
                    let src_pixel = src.data[src_idx];
                    let alpha = src_pixel >> 24;
                
                    if alpha == 255 {
                        self.data[dst_idx] = src_pixel;
                    } else if alpha > 0 {
                        self.data[dst_idx] = blend_fast(src_pixel, self.data[dst_idx]);
                    }
                    x += 1;
                }
            }
        }
        
        self.dirty.add_rect(dx_start, dy_start, copy_w, copy_h);
    }

    /// Draw rounded rectangle (optimized)
    pub fn fill_rounded_rect(&mut self, x: i32, y: i32, w: u32, h: u32, r: u32, color: u32) {
        let r = r.min(w / 2).min(h / 2);
        
        // Center rectangle (no corners)
        self.fill_rect(x + r as i32, y, w - 2 * r, h, color);
        
        // Left and right strips
        self.fill_rect(x, y + r as i32, r, h - 2 * r, color);
        self.fill_rect(x + w as i32 - r as i32, y + r as i32, r, h - 2 * r, color);
        
        // Draw corners using precomputed circle
        self.fill_corner(x + r as i32, y + r as i32, r, color, Corner::TopLeft);
        self.fill_corner(x + w as i32 - r as i32 - 1, y + r as i32, r, color, Corner::TopRight);
        self.fill_corner(x + r as i32, y + h as i32 - r as i32 - 1, r, color, Corner::BottomLeft);
        self.fill_corner(x + w as i32 - r as i32 - 1, y + h as i32 - r as i32 - 1, r, color, Corner::BottomRight);
    }

    fn fill_corner(&mut self, cx: i32, cy: i32, r: u32, color: u32, corner: Corner) {
        let r = r as i32;
        let r_sq = r * r;
        
        for dy in 0..=r {
            for dx in 0..=r {
                if dx * dx + dy * dy <= r_sq {
                    let (px, py) = match corner {
                        Corner::TopLeft => (cx - dx, cy - dy),
                        Corner::TopRight => (cx + dx, cy - dy),
                        Corner::BottomLeft => (cx - dx, cy + dy),
                        Corner::BottomRight => (cx + dx, cy + dy),
                    };
                    if px >= 0 && py >= 0 && px < self.width as i32 && py < self.height as i32 {
                        self.set_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }
    }

    /// Get dirty region for partial updates
    pub fn get_dirty(&self) -> &DirtyRegion {
        &self.dirty
    }

    /// Clear dirty tracking
    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    /// Copy only dirty regions to framebuffer (massive optimization!)
    pub fn flush_dirty_to_fb(&mut self) {
        if self.dirty.full_redraw {
            self.flush_to_fb();
        } else {
            for i in 0..self.dirty.count {
                let rect = self.dirty.rects[i];
                self.flush_rect_to_fb(rect.x, rect.y, rect.w, rect.h);
            }
        }
        self.dirty.clear();
    }

    /// Flush entire surface to framebuffer
    pub fn flush_to_fb(&self) {
        let (fb_width, fb_height) = crate::framebuffer::get_dimensions();
        let copy_w = self.width.min(fb_width) as usize;
        let copy_h = self.height.min(fb_height) as usize;
        
        let fb_addr = crate::framebuffer::get_fb_addr();
        let fb_pitch = crate::framebuffer::get_fb_pitch();
        
        if fb_addr.is_null() { return; }
        
        let src_stride = self.width as usize;
        
        for row in 0..copy_h {
            let src_start = row * src_stride;
            let dst_offset = row * fb_pitch;
            
            unsafe {
                let src = self.data.as_ptr().add(src_start);
                let dst = fb_addr.add(dst_offset) as *mut u32;
                core::ptr::copy_nonoverlapping(src, dst, copy_w);
            }
        }
    }

    /// Flush only a rectangle to framebuffer
    fn flush_rect_to_fb(&self, x: u32, y: u32, w: u32, h: u32) {
        let (fb_width, fb_height) = crate::framebuffer::get_dimensions();
        
        let x1 = x.min(fb_width).min(self.width);
        let y1 = y.min(fb_height).min(self.height);
        let x2 = (x + w).min(fb_width).min(self.width);
        let y2 = (y + h).min(fb_height).min(self.height);
        
        if x2 <= x1 || y2 <= y1 { return; }
        
        let fb_addr = crate::framebuffer::get_fb_addr();
        let fb_pitch = crate::framebuffer::get_fb_pitch();
        
        if fb_addr.is_null() { return; }
        
        let copy_w = (x2 - x1) as usize;
        let src_stride = self.width as usize;
        
        for row in y1..y2 {
            let src_start = row as usize * src_stride + x1 as usize;
            let dst_offset = row as usize * fb_pitch + x1 as usize * 4;
            
            unsafe {
                let src = self.data.as_ptr().add(src_start);
                let dst = fb_addr.add(dst_offset) as *mut u32;
                core::ptr::copy_nonoverlapping(src, dst, copy_w);
            }
        }
    }
}

enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

// ═══════════════════════════════════════════════════════════════════════════════
// DIRTY REGION TRACKING
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Default)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub struct DirtyRegion {
    pub rects: [Rect; MAX_DIRTY_RECTS],
    pub count: usize,
    pub full_redraw: bool,
    screen_w: u32,
    screen_h: u32,
}

impl DirtyRegion {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            rects: [Rect::default(); MAX_DIRTY_RECTS],
            count: 0,
            full_redraw: true, // Start with full redraw
            screen_w: width,
            screen_h: height,
        }
    }

    pub fn add_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        if self.full_redraw { return; }
        if w == 0 || h == 0 { return; }
        
        let new_rect = Rect { x, y, w, h };
        
        // Try to merge with existing
        for i in 0..self.count {
            if rects_overlap(&self.rects[i], &new_rect) {
                self.rects[i] = merge_rects(&self.rects[i], &new_rect);
                return;
            }
        }
        
        // Add as new
        if self.count < MAX_DIRTY_RECTS {
            self.rects[self.count] = new_rect;
            self.count += 1;
        } else {
            // Too many rects, do full redraw
            self.full_redraw = true;
        }
    }

    pub fn mark_full(&mut self) {
        self.full_redraw = true;
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.full_redraw = false;
    }
}

fn rects_overlap(a: &Rect, b: &Rect) -> bool {
    !(a.x + a.w <= b.x || b.x + b.w <= a.x ||
      a.y + a.h <= b.y || b.y + b.h <= a.y)
}

fn merge_rects(a: &Rect, b: &Rect) -> Rect {
    let x1 = a.x.min(b.x);
    let y1 = a.y.min(b.y);
    let x2 = (a.x + a.w).max(b.x + b.w);
    let y2 = (a.y + a.h).max(b.y + b.h);
    Rect { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FAST ALPHA BLENDING
// ═══════════════════════════════════════════════════════════════════════════════

/// Fast alpha blend using integer math only
#[inline(always)]
fn blend_fast(src: u32, dst: u32) -> u32 {
    let alpha = (src >> 24) & 0xFF;
    if alpha == 0 { return dst; }
    if alpha == 255 { return src; }
    
    let inv_alpha = 255 - alpha;
    
    // Unpack
    let sr = (src >> 16) & 0xFF;
    let sg = (src >> 8) & 0xFF;
    let sb = src & 0xFF;
    
    let dr = (dst >> 16) & 0xFF;
    let dg = (dst >> 8) & 0xFF;
    let db = dst & 0xFF;
    
    // Blend with rounding
    let r = (sr * alpha + dr * inv_alpha + 127) / 255;
    let g = (sg * alpha + dg * inv_alpha + 127) / 255;
    let b = (sb * alpha + db * inv_alpha + 127) / 255;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}

// ═══════════════════════════════════════════════════════════════════════════════
// FONT CACHE
// ═══════════════════════════════════════════════════════════════════════════════

/// Cached glyph bitmap
pub struct GlyphCache {
    /// Pre-rendered glyphs (8x16 bitmap expanded to 8x16 u32 array)
    glyphs: [[u32; 128]; FONT_CACHE_SIZE], // [char_index][pixel_index] = color or 0
    fg_color: u32,
    initialized: bool,
}

impl GlyphCache {
    pub const fn new() -> Self {
        Self {
            glyphs: [[0u32; 128]; FONT_CACHE_SIZE],
            fg_color: 0xFFFFFFFF,
            initialized: false,
        }
    }

    /// Initialize cache with a foreground color
    pub fn init(&mut self, fg_color: u32) {
        self.fg_color = fg_color;
        
        for c in 0..FONT_CACHE_SIZE {
            let glyph_data = crate::framebuffer::font::get_glyph(c as u8 as char);
            let mut pixel_idx = 0;
            
            for row in 0..16 {
                let bits = glyph_data[row];
                for col in 0..8 {
                    if (bits >> (7 - col)) & 1 == 1 {
                        self.glyphs[c][pixel_idx] = fg_color;
                    } else {
                        self.glyphs[c][pixel_idx] = 0; // Transparent
                    }
                    pixel_idx += 1;
                }
            }
        }
        
        self.initialized = true;
    }

    /// Draw cached glyph to surface (fast - no per-pixel bit testing)
    pub fn draw_glyph(&self, surface: &mut FastSurface, c: char, x: i32, y: i32) {
        let idx = (c as usize).min(FONT_CACHE_SIZE - 1);
        let glyph = &self.glyphs[idx];
        
        let mut pixel_idx = 0;
        for row in 0..16 {
            let py = y + row;
            if py >= 0 && py < surface.height as i32 {
                for col in 0..8 {
                    let px = x + col;
                    if px >= 0 && px < surface.width as i32 {
                        let color = glyph[pixel_idx];
                        if color != 0 {
                            surface.set_pixel(px as u32, py as u32, color);
                        }
                    }
                    pixel_idx += 1;
                }
            } else {
                pixel_idx += 8;
            }
        }
    }

    /// Draw string using cached glyphs
    pub fn draw_string(&self, surface: &mut FastSurface, s: &str, x: i32, y: i32) {
        let mut cx = x;
        for c in s.chars() {
            if cx >= surface.width as i32 { break; }
            self.draw_glyph(surface, c, cx, y);
            cx += 8;
        }
    }
}

// Global font cache
static FONT_CACHE: Mutex<GlyphCache> = Mutex::new(GlyphCache::new());

/// Initialize global font cache
pub fn init_font_cache(fg_color: u32) {
    FONT_CACHE.lock().init(fg_color);
}

/// Draw text using cached font
pub fn draw_text(surface: &mut FastSurface, s: &str, x: i32, y: i32) {
    FONT_CACHE.lock().draw_string(surface, s, x, y);
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMPOSITOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Layer in the compositor
pub struct Layer {
    pub surface: FastSurface,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub visible: bool,
    pub id: u32,
}

/// Fast software compositor
pub struct FastCompositor {
    /// Output surface (what gets shown on screen)
    pub output: FastSurface,
    /// Layers sorted by z-order
    layers: Vec<Layer>,
    /// Next layer ID
    next_id: u32,
    /// Background color
    bg_color: u32,
}

impl FastCompositor {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            output: FastSurface::new(width, height),
            layers: Vec::new(),
            next_id: 1,
            bg_color: 0xFF101010,
        }
    }

    /// Set background color
    pub fn set_background(&mut self, color: u32) {
        self.bg_color = color;
    }

    /// Create a new layer
    pub fn create_layer(&mut self, width: u32, height: u32, x: i32, y: i32, z: i32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let layer = Layer {
            surface: FastSurface::new(width, height),
            x,
            y,
            z,
            visible: true,
            id,
        };
        
        self.layers.push(layer);
        self.sort_layers();
        
        id
    }

    /// Get layer by ID
    pub fn get_layer(&mut self, id: u32) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    /// Remove layer
    pub fn remove_layer(&mut self, id: u32) {
        self.layers.retain(|l| l.id != id);
    }

    /// Move layer to new position
    pub fn move_layer(&mut self, id: u32, x: i32, y: i32) {
        if let Some(layer) = self.get_layer(id) {
            layer.x = x;
            layer.y = y;
        }
    }

    /// Raise layer to top
    pub fn raise_layer(&mut self, id: u32) {
        let max_z = self.layers.iter().map(|l| l.z).max().unwrap_or(0);
        if let Some(layer) = self.get_layer(id) {
            layer.z = max_z + 1;
        }
        self.sort_layers();
    }

    fn sort_layers(&mut self) {
        self.layers.sort_by_key(|l| l.z);
    }

    /// Composite all layers to output
    pub fn composite(&mut self) {
        // Clear output with background
        self.output.clear(self.bg_color);
        
        // Blit each visible layer (back to front)
        for layer in &self.layers {
            if layer.visible {
                self.output.blit_alpha(&layer.surface, layer.x, layer.y);
            }
        }
    }

    /// Flush output to framebuffer
    pub fn present(&mut self) {
        self.output.flush_dirty_to_fb();
    }

    /// Full render and present
    pub fn render(&mut self) {
        self.composite();
        self.present();
    }
}
