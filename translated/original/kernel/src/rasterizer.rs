//! Software Rasterizer for TrustOS
//! 
//! Provides optimized 2D rendering with:
//! - Alpha blending (transparency)
//! - Antialiased primitives
//! - Gradient fills
//! - Dirty rectangle optimization

use alloc::vec::Vec;

// Math helpers for no_std environment
fn floor_f32(x: f32) -> f32 {
    let xi = x as i32;
    if x < xi as f32 { (xi - 1) as f32 } else { xi as f32 }
}

fn round_f32(x: f32) -> f32 {
    floor_f32(x + 0.5)
}

fn fract_f32(x: f32) -> f32 {
    x - floor_f32(x)
}

fn abs_f32(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

fn clamp_f32(x: f32, min: f32, max: f32) -> f32 {
    if x < min { min } else if x > max { max } else { x }
}

fn sqrt_f32(x: f32) -> f32 { crate::math::fast_sqrt(x) }

fn sin_f32(x: f32) -> f32 { crate::math::fast_sin(x) }

fn cos_f32(x: f32) -> f32 { crate::math::fast_cos(x) }

fn tan_f32(x: f32) -> f32 { crate::math::fast_tan(x) }

/// RGBA color with alpha channel
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    pub const fn from_u32(c: u32) -> Self {
        Self {
            a: ((c >> 24) & 0xFF) as u8,
            r: ((c >> 16) & 0xFF) as u8,
            g: ((c >> 8) & 0xFF) as u8,
            b: (c & 0xFF) as u8,
        }
    }
    
    pub const fn to_u32(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
    
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    /// Predefined colors
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const RED: Self = Self::new(255, 0, 0, 255);
    pub const GREEN: Self = Self::new(0, 255, 0, 255);
    pub const BLUE: Self = Self::new(0, 0, 255, 255);
}

/// Dirty rectangle for optimization
#[derive(Clone, Copy, Debug)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl DirtyRect {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }
    
    /// Merge two rectangles into one that contains both
    pub fn union(&self, other: &DirtyRect) -> DirtyRect {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.w).max(other.x + other.w);
        let y2 = (self.y + self.h).max(other.y + other.h);
        DirtyRect::new(x1, y1, x2 - x1, y2 - y1)
    }
    
    /// Check if rectangles intersect
    pub fn intersects(&self, other: &DirtyRect) -> bool {
        !(self.x + self.w <= other.x || other.x + other.w <= self.x ||
          self.y + self.h <= other.y || other.y + other.h <= self.y)
    }
}

/// Software rasterizer with double buffering
pub struct Rasterizer {
    pub width: u32,
    pub height: u32,
    pub front_buffer: Vec<u32>,
    pub back_buffer: Vec<u32>,
    pub dirty_rects: Vec<DirtyRect>,
    pub full_redraw: bool,
}

impl Rasterizer {
    /// Create a new rasterizer with given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            front_buffer: alloc::vec![0xFF000000; size],
            back_buffer: alloc::vec![0xFF000000; size],
            dirty_rects: Vec::new(),
            full_redraw: true,
        }
    }
    
    /// Mark a region as dirty (needs redraw)
    pub fn mark_dirty(&mut self, x: u32, y: u32, w: u32, h: u32) {
        let rect = DirtyRect::new(
            x.min(self.width),
            y.min(self.height),
            w.min(self.width - x.min(self.width)),
            h.min(self.height - y.min(self.height)),
        );
        
        // Merge with existing rects if they overlap
        let mut merged = false;
        for existing in self.dirty_rects.iter_mut() {
            if existing.intersects(&rect) {
                *existing = existing.union(&rect);
                merged = true;
                break;
            }
        }
        
        if !merged {
            self.dirty_rects.push(rect);
        }
    }
    
    /// Clear the back buffer (SSE2-optimized on x86_64)
    pub fn clear(&mut self, color: u32) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use core::arch::x86_64::*;
            let fill = _mm_set1_epi32(color as i32);
            let ptr = self.back_buffer.as_mut_ptr() as *mut __m128i;
            let count = self.back_buffer.len() / 4;
            for i in 0..count {
                _mm_storeu_si128(ptr.add(i), fill);
            }
            for i in (count * 4)..self.back_buffer.len() {
                self.back_buffer[i] = color;
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            for pixel in self.back_buffer.iter_mut() {
                *pixel = color;
            }
        }
        self.full_redraw = true;
    }
    
    /// Get pixel index
    #[inline(always)]
    fn pixel_index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }
    
    /// Set a pixel with alpha blending
    #[inline(always)]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if let Some(idx) = self.pixel_index(x, y) {
            let alpha = (color >> 24) & 0xFF;
            
            if alpha == 255 {
                // Fully opaque - direct write
                self.back_buffer[idx] = color;
            } else if alpha > 0 {
                // Alpha blend
                self.back_buffer[idx] = Self::blend_pixel(self.back_buffer[idx], color);
            }
            // alpha == 0: fully transparent, do nothing
        }
    }
    
    /// Alpha blend two colors (src over dst)
    #[inline(always)]
    pub fn blend_pixel(dst: u32, src: u32) -> u32 {
        let sa = ((src >> 24) & 0xFF) as u32;
        if sa == 0 { return dst; }
        if sa == 255 { return src; }
        
        let sr = ((src >> 16) & 0xFF) as u32;
        let sg = ((src >> 8) & 0xFF) as u32;
        let sb = (src & 0xFF) as u32;
        
        let da = ((dst >> 24) & 0xFF) as u32;
        let dr = ((dst >> 16) & 0xFF) as u32;
        let dg = ((dst >> 8) & 0xFF) as u32;
        let db = (dst & 0xFF) as u32;
        
        // Fast alpha blend: out = src * sa + dst * (255 - sa)
        let inv_sa = 255 - sa;
        let or = (sr * sa + dr * inv_sa) / 255;
        let og = (sg * sa + dg * inv_sa) / 255;
        let ob = (sb * sa + db * inv_sa) / 255;
        let oa = sa + (da * inv_sa) / 255;
        
        (oa << 24) | (or << 16) | (og << 8) | ob
    }
    
    /// Draw a filled rectangle with optional alpha (SSE2-optimized for opaque)
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        let x0 = x.max(0) as u32;
        let y0 = y.max(0) as u32;
        let x1 = ((x + w as i32) as u32).min(self.width);
        let y1 = ((y + h as i32) as u32).min(self.height);
        
        let alpha = (color >> 24) & 0xFF;
        let row_width = (x1 - x0) as usize;
        
        if alpha == 255 && row_width >= 4 {
            // Fast path: SSE2 fill for opaque rects
            #[cfg(target_arch = "x86_64")]
            unsafe {
                use core::arch::x86_64::*;
                let fill = _mm_set1_epi32(color as i32);
                for py in y0..y1 {
                    let row_start = (py * self.width + x0) as usize;
                    let ptr = self.back_buffer.as_mut_ptr().add(row_start) as *mut __m128i;
                    let chunks = row_width / 4;
                    for i in 0..chunks {
                        _mm_storeu_si128(ptr.add(i), fill);
                    }
                    for i in (chunks * 4)..row_width {
                        self.back_buffer[row_start + i] = color;
                    }
                }
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                for py in y0..y1 {
                    let row_start = (py * self.width + x0) as usize;
                    for i in 0..row_width {
                        self.back_buffer[row_start + i] = color;
                    }
                }
            }
        } else {
            for py in y0..y1 {
                let row_start = (py * self.width) as usize;
                for px in x0..x1 {
                    let idx = row_start + px as usize;
                    if alpha == 255 {
                        self.back_buffer[idx] = color;
                    } else if alpha > 0 {
                        self.back_buffer[idx] = Self::blend_pixel(self.back_buffer[idx], color);
                    }
                }
            }
        }
        
        self.mark_dirty(x0, y0, x1 - x0, y1 - y0);
    }
    
    /// Draw a rectangle outline
    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        // Top and bottom lines
        self.fill_rect(x, y, w, 1, color);
        self.fill_rect(x, y + h as i32 - 1, w, 1, color);
        // Left and right lines
        self.fill_rect(x, y, 1, h, color);
        self.fill_rect(x + w as i32 - 1, y, 1, h, color);
    }
    
    /// Draw an antialiased line using Xiaolin Wu's algorithm
    pub fn draw_line_aa(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: u32) {
        let steep = abs_f32(y1 - y0) > abs_f32(x1 - x0);
        
        let (x0, y0, x1, y1) = if steep {
            (y0, x0, y1, x1)
        } else {
            (x0, y0, x1, y1)
        };
        
        let (x0, y0, x1, y1) = if x0 > x1 {
            (x1, y1, x0, y0)
        } else {
            (x0, y0, x1, y1)
        };
        
        let dx = x1 - x0;
        let dy = y1 - y0;
        let gradient = if dx == 0.0 { 1.0 } else { dy / dx };
        
        // Handle first endpoint
        let xend = round_f32(x0);
        let yend = y0 + gradient * (xend - x0);
        let xgap = 1.0 - fract_f32(x0 + 0.5);
        let xpxl1 = xend as i32;
        let ypxl1 = floor_f32(yend) as i32;
        
        if steep {
            self.plot_aa(ypxl1, xpxl1, color, (1.0 - fract_f32(yend)) * xgap);
            self.plot_aa(ypxl1 + 1, xpxl1, color, fract_f32(yend) * xgap);
        } else {
            self.plot_aa(xpxl1, ypxl1, color, (1.0 - fract_f32(yend)) * xgap);
            self.plot_aa(xpxl1, ypxl1 + 1, color, fract_f32(yend) * xgap);
        }
        
        let mut intery = yend + gradient;
        
        // Handle second endpoint
        let xend = round_f32(x1);
        let yend = y1 + gradient * (xend - x1);
        let xgap = fract_f32(x1 + 0.5);
        let xpxl2 = xend as i32;
        let ypxl2 = floor_f32(yend) as i32;
        
        if steep {
            self.plot_aa(ypxl2, xpxl2, color, (1.0 - fract_f32(yend)) * xgap);
            self.plot_aa(ypxl2 + 1, xpxl2, color, fract_f32(yend) * xgap);
        } else {
            self.plot_aa(xpxl2, ypxl2, color, (1.0 - fract_f32(yend)) * xgap);
            self.plot_aa(xpxl2, ypxl2 + 1, color, fract_f32(yend) * xgap);
        }
        
        // Main loop
        for x in (xpxl1 + 1)..xpxl2 {
            if steep {
                self.plot_aa(floor_f32(intery) as i32, x, color, 1.0 - fract_f32(intery));
                self.plot_aa(floor_f32(intery) as i32 + 1, x, color, fract_f32(intery));
            } else {
                self.plot_aa(x, floor_f32(intery) as i32, color, 1.0 - fract_f32(intery));
                self.plot_aa(x, floor_f32(intery) as i32 + 1, color, fract_f32(intery));
            }
            intery += gradient;
        }
    }
    
    /// Plot a pixel with fractional intensity for antialiasing
    #[inline(always)]
    fn plot_aa(&mut self, x: i32, y: i32, color: u32, intensity: f32) {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            let base_alpha = ((color >> 24) & 0xFF) as f32;
            let new_alpha = (base_alpha * clamp_f32(intensity, 0.0, 1.0)) as u32;
            let blended_color = (new_alpha << 24) | (color & 0x00FFFFFF);
            self.set_pixel(x as u32, y as u32, blended_color);
        }
    }
    
    /// Draw an antialiased circle using midpoint algorithm with AA
    pub fn draw_circle_aa(&mut self, cx: i32, cy: i32, radius: u32, color: u32) {
        let r = radius as f32;
        let _r2 = r * r;
        
        for y in -(radius as i32)..=(radius as i32) {
            for x in -(radius as i32)..=(radius as i32) {
                let dist2 = (x * x + y * y) as f32;
                let dist = sqrt_f32(dist2);
                
                // Distance from edge
                let edge_dist = abs_f32(dist - r);
                
                if edge_dist < 1.5 {
                    // Near the edge - calculate AA intensity
                    let v = abs_f32(dist - r + 0.5);
                    let intensity = 1.0 - if v < 1.0 { v } else { 1.0 };
                    if intensity > 0.0 {
                        self.plot_aa(cx + x, cy + y, color, intensity);
                    }
                }
            }
        }
    }
    
    /// Draw a filled circle with antialiased edge
    pub fn fill_circle_aa(&mut self, cx: i32, cy: i32, radius: u32, color: u32) {
        let r = radius as f32;
        
        for y in -(radius as i32 + 1)..=(radius as i32 + 1) {
            for x in -(radius as i32 + 1)..=(radius as i32 + 1) {
                let dist = sqrt_f32((x * x + y * y) as f32);
                
                if dist <= r - 0.5 {
                    // Inside - full color
                    self.set_pixel((cx + x) as u32, (cy + y) as u32, color);
                } else if dist < r + 0.5 {
                    // Edge - antialiased
                    let intensity = 1.0 - clamp_f32(dist - r + 0.5, 0.0, 1.0);
                    self.plot_aa(cx + x, cy + y, color, intensity);
                }
            }
        }
        
        self.mark_dirty(
            (cx - radius as i32 - 1).max(0) as u32,
            (cy - radius as i32 - 1).max(0) as u32,
            radius * 2 + 3,
            radius * 2 + 3,
        );
    }
    
    /// Draw a horizontal gradient
    pub fn fill_gradient_h(&mut self, x: i32, y: i32, w: u32, h: u32, color1: u32, color2: u32) {
        let x0 = x.max(0) as u32;
        let y0 = y.max(0) as u32;
        let x1 = ((x + w as i32) as u32).min(self.width);
        let y1 = ((y + h as i32) as u32).min(self.height);
        
        let c1 = Color::from_u32(color1);
        let c2 = Color::from_u32(color2);
        
        for py in y0..y1 {
            let row_start = (py * self.width) as usize;
            for px in x0..x1 {
                let t = (px - x0) as f32 / w as f32;
                let r = (c1.r as f32 * (1.0 - t) + c2.r as f32 * t) as u8;
                let g = (c1.g as f32 * (1.0 - t) + c2.g as f32 * t) as u8;
                let b = (c1.b as f32 * (1.0 - t) + c2.b as f32 * t) as u8;
                let a = (c1.a as f32 * (1.0 - t) + c2.a as f32 * t) as u8;
                
                let color = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                let idx = row_start + px as usize;
                
                if a == 255 {
                    self.back_buffer[idx] = color;
                } else if a > 0 {
                    self.back_buffer[idx] = Self::blend_pixel(self.back_buffer[idx], color);
                }
            }
        }
        
        self.mark_dirty(x0, y0, x1 - x0, y1 - y0);
    }
    
    /// Draw a vertical gradient
    pub fn fill_gradient_v(&mut self, x: i32, y: i32, w: u32, h: u32, color1: u32, color2: u32) {
        let x0 = x.max(0) as u32;
        let y0 = y.max(0) as u32;
        let x1 = ((x + w as i32) as u32).min(self.width);
        let y1 = ((y + h as i32) as u32).min(self.height);
        
        let c1 = Color::from_u32(color1);
        let c2 = Color::from_u32(color2);
        
        for py in y0..y1 {
            let t = (py - y0) as f32 / h as f32;
            let r = (c1.r as f32 * (1.0 - t) + c2.r as f32 * t) as u8;
            let g = (c1.g as f32 * (1.0 - t) + c2.g as f32 * t) as u8;
            let b = (c1.b as f32 * (1.0 - t) + c2.b as f32 * t) as u8;
            let a = (c1.a as f32 * (1.0 - t) + c2.a as f32 * t) as u8;
            
            let color = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            let row_start = (py * self.width) as usize;
            
            for px in x0..x1 {
                let idx = row_start + px as usize;
                if a == 255 {
                    self.back_buffer[idx] = color;
                } else if a > 0 {
                    self.back_buffer[idx] = Self::blend_pixel(self.back_buffer[idx], color);
                }
            }
        }
        
        self.mark_dirty(x0, y0, x1 - x0, y1 - y0);
    }
    
    /// Draw a rounded rectangle with antialiasing
    pub fn fill_rounded_rect(&mut self, x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
        let r = radius.min(w / 2).min(h / 2);
        
        // Fill main body (excluding corners)
        self.fill_rect(x + r as i32, y, w - r * 2, h, color);
        self.fill_rect(x, y + r as i32, r, h - r * 2, color);
        self.fill_rect(x + w as i32 - r as i32, y + r as i32, r, h - r * 2, color);
        
        // Draw antialiased corners
        // Top-left
        self.fill_corner_aa(x + r as i32, y + r as i32, r, color, 2);
        // Top-right
        self.fill_corner_aa(x + w as i32 - r as i32 - 1, y + r as i32, r, color, 1);
        // Bottom-left
        self.fill_corner_aa(x + r as i32, y + h as i32 - r as i32 - 1, r, color, 3);
        // Bottom-right
        self.fill_corner_aa(x + w as i32 - r as i32 - 1, y + h as i32 - r as i32 - 1, r, color, 0);
    }
    
    /// Fill a corner with antialiasing (quadrant: 0=BR, 1=TR, 2=TL, 3=BL)
    fn fill_corner_aa(&mut self, cx: i32, cy: i32, radius: u32, color: u32, quadrant: u8) {
        let r = radius as f32;
        
        let (x_range, y_range): (core::ops::RangeInclusive<i32>, core::ops::RangeInclusive<i32>) = match quadrant {
            0 => (0..=(radius as i32), 0..=(radius as i32)),       // Bottom-right
            1 => (0..=(radius as i32), -(radius as i32)..=0),      // Top-right
            2 => (-(radius as i32)..=0, -(radius as i32)..=0),     // Top-left
            3 => (-(radius as i32)..=0, 0..=(radius as i32)),      // Bottom-left
            _ => return,
        };
        
        for dy in y_range {
            for dx in x_range.clone() {
                let dist = sqrt_f32((dx * dx + dy * dy) as f32);
                
                if dist <= r - 0.5 {
                    self.set_pixel((cx + dx) as u32, (cy + dy) as u32, color);
                } else if dist < r + 0.5 {
                    let intensity = 1.0 - clamp_f32(dist - r + 0.5, 0.0, 1.0);
                    self.plot_aa(cx + dx, cy + dy, color, intensity);
                }
            }
        }
    }
    
    /// Draw a drop shadow (blurred rectangle)
    pub fn draw_shadow(&mut self, x: i32, y: i32, w: u32, h: u32, blur: u32, color: u32) {
        let base_alpha = ((color >> 24) & 0xFF) as f32;
        let shadow_color = color & 0x00FFFFFF;
        
        for b in 0..blur {
            let t = (blur - b) as f32 / blur as f32;
            let alpha = (base_alpha * t * 0.5) as u32;
            let c = (alpha << 24) | shadow_color;
            
            self.fill_rect(
                x - b as i32,
                y - b as i32,
                w + b * 2,
                h + b * 2,
                c,
            );
        }
    }
    
    /// Swap buffers - copy only dirty regions to front buffer
    pub fn swap_buffers(&mut self) {
        if self.full_redraw {
            // Copy entire buffer
            self.front_buffer.copy_from_slice(&self.back_buffer);
            self.full_redraw = false;
        } else {
            // Copy only dirty regions
            for rect in &self.dirty_rects {
                for y in rect.y..(rect.y + rect.h).min(self.height) {
                    let start = (y * self.width + rect.x) as usize;
                    let end = (y * self.width + (rect.x + rect.w).min(self.width)) as usize;
                    self.front_buffer[start..end].copy_from_slice(&self.back_buffer[start..end]);
                }
            }
        }
        
        self.dirty_rects.clear();
    }
    
    /// Blit front buffer to framebuffer
    pub fn blit_to_framebuffer(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                crate::framebuffer::draw_pixel(x, y, self.front_buffer[idx]);
            }
        }
    }
    
    /// Blit only dirty regions to framebuffer
    pub fn blit_dirty_to_framebuffer(&self) {
        for rect in &self.dirty_rects {
            for y in rect.y..(rect.y + rect.h).min(self.height) {
                for x in rect.x..(rect.x + rect.w).min(self.width) {
                    let idx = (y * self.width + x) as usize;
                    crate::framebuffer::draw_pixel(x, y, self.front_buffer[idx]);
                }
            }
        }
    }
}

/// Simple 3D vector for future 3D rendering
#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    
    pub fn length(&self) -> f32 {
        sqrt_f32(self.x * self.x + self.y * self.y + self.z * self.z)
    }
    
    pub fn normalize(&self) -> Vec3 {
        let len = self.length();
        if len > 0.0 {
            Vec3 {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }
}

/// 4x4 Matrix for transformations
#[derive(Clone, Copy, Debug)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Mat4 {
    pub const fn identity() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    
    /// Create rotation matrix around Y axis
    pub fn rotation_y(angle: f32) -> Self {
        let c = cos_f32(angle);
        let s = sin_f32(angle);
        Self {
            m: [
                [c, 0.0, s, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-s, 0.0, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    
    /// Create rotation matrix around X axis
    pub fn rotation_x(angle: f32) -> Self {
        let c = cos_f32(angle);
        let s = sin_f32(angle);
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, c, -s, 0.0],
                [0.0, s, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    
    /// Create perspective projection matrix
    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / tan_f32(fov / 2.0);
        Self {
            m: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (far + near) / (near - far), -1.0],
                [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
            ],
        }
    }
    
    /// Transform a 3D point
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        let w = self.m[0][3] * p.x + self.m[1][3] * p.y + self.m[2][3] * p.z + self.m[3][3];
        Vec3 {
            x: (self.m[0][0] * p.x + self.m[1][0] * p.y + self.m[2][0] * p.z + self.m[3][0]) / w,
            y: (self.m[0][1] * p.x + self.m[1][1] * p.y + self.m[2][1] * p.z + self.m[3][1]) / w,
            z: (self.m[0][2] * p.x + self.m[1][2] * p.y + self.m[2][2] * p.z + self.m[3][2]) / w,
        }
    }
    
    /// Multiply two matrices
    pub fn mul(&self, other: &Mat4) -> Mat4 {
        let mut result = Mat4::identity();
        for i in 0..4 {
            for j in 0..4 {
                result.m[i][j] = 0.0;
                for k in 0..4 {
                    result.m[i][j] += self.m[i][k] * other.m[k][j];
                }
            }
        }
        result
    }
}

/// Simple 3D wireframe renderer
pub struct Renderer3D {
    pub width: u32,
    pub height: u32,
    pub z_buffer: Vec<f32>,
}

impl Renderer3D {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            z_buffer: alloc::vec![f32::MAX; (width * height) as usize],
        }
    }
    
    pub fn clear_z_buffer(&mut self) {
        for z in self.z_buffer.iter_mut() {
            *z = f32::MAX;
        }
    }
    
    /// Project 3D point to 2D screen coordinates
    pub fn project(&self, p: Vec3, camera_z: f32) -> Option<(i32, i32, f32)> {
        let z = p.z + camera_z;
        if z <= 0.1 { return None; }
        
        let scale = 200.0 / z;
        let sx = (self.width as f32 / 2.0 + p.x * scale) as i32;
        let sy = (self.height as f32 / 2.0 - p.y * scale) as i32;
        
        Some((sx, sy, z))
    }
    
    /// Draw a 3D line with depth testing
    pub fn draw_line_3d(&mut self, rast: &mut Rasterizer, p1: Vec3, p2: Vec3, camera_z: f32, color: u32) {
        if let (Some((x1, y1, _)), Some((x2, y2, _))) = (self.project(p1, camera_z), self.project(p2, camera_z)) {
            rast.draw_line_aa(x1 as f32, y1 as f32, x2 as f32, y2 as f32, color);
        }
    }
    
    /// Draw a wireframe cube
    pub fn draw_cube(&mut self, rast: &mut Rasterizer, center: Vec3, size: f32, rotation: &Mat4, color: u32) {
        let s = size / 2.0;
        
        // 8 vertices of cube
        let vertices = [
            Vec3::new(-s, -s, -s),
            Vec3::new(s, -s, -s),
            Vec3::new(s, s, -s),
            Vec3::new(-s, s, -s),
            Vec3::new(-s, -s, s),
            Vec3::new(s, -s, s),
            Vec3::new(s, s, s),
            Vec3::new(-s, s, s),
        ];
        
        // Transform vertices
        let transformed: Vec<Vec3> = vertices.iter()
            .map(|v| {
                let rotated = rotation.transform_point(*v);
                Vec3::new(
                    rotated.x + center.x,
                    rotated.y + center.y,
                    rotated.z + center.z,
                )
            })
            .collect();
        
        // 12 edges of cube
        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 0),  // Front face
            (4, 5), (5, 6), (6, 7), (7, 4),  // Back face
            (0, 4), (1, 5), (2, 6), (3, 7),  // Connecting edges
        ];
        
        for (i1, i2) in edges {
            self.draw_line_3d(rast, transformed[i1], transformed[i2], 5.0, color);
        }
    }
}
