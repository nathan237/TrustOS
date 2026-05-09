//! WOA Renderer — Internal 1280×800 backbuffer with blit
//!
//! All game drawing happens to the internal buffer.
//! `present()` blits to native framebuffer (1:1 or nearest-neighbor upscale).

use alloc::vec;
use alloc::vec::Vec;

/// Internal game resolution — 1:1 with native framebuffer
pub const INTERNAL_W: u32 = 1280;
pub const INTERNAL_H: u32 = 800;

pub struct Renderer {
    /// Internal pixel buffer (ARGB u32)
    buffer: Vec<u32>,
    width: u32,
    height: u32,
}

impl Renderer {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            buffer: vec![0xFF000000; (w * h) as usize],
            width: w,
            height: h,
        }
    }

    /// Clear entire buffer to a solid color
    #[inline]
    pub fn clear(&mut self, color: u32) {
        for px in self.buffer.iter_mut() {
            *px = color;
        }
    }

    /// Set a single pixel (bounds-checked)
    #[inline]
    pub fn put_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.buffer[(y * self.width + x) as usize] = color;
        }
    }

    /// Fill a rectangle (clipped to buffer bounds)
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let x1 = x.min(self.width);
        let y1 = y.min(self.height);
        let x2 = (x + w).min(self.width);
        let y2 = (y + h).min(self.height);
        for row in y1..y2 {
            let start = (row * self.width + x1) as usize;
            let end = (row * self.width + x2) as usize;
            for px in &mut self.buffer[start..end] {
                *px = color;
            }
        }
    }

    /// Draw a 1px outline rectangle
    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        if w == 0 || h == 0 { return; }
        // Top & bottom
        for i in x..x + w {
            self.put_pixel(i, y, color);
            self.put_pixel(i, y + h - 1, color);
        }
        // Left & right
        for j in y..y + h {
            self.put_pixel(x, j, color);
            self.put_pixel(x + w - 1, j, color);
        }
    }

    /// Draw horizontal line
    #[inline]
    pub fn hline(&mut self, x: u32, y: u32, len: u32, color: u32) {
        for i in x..(x + len).min(self.width) {
            self.put_pixel(i, y, color);
        }
    }

    /// Draw vertical line
    #[inline]
    pub fn vline(&mut self, x: u32, y: u32, len: u32, color: u32) {
        for j in y..(y + len).min(self.height) {
            self.put_pixel(x, j, color);
        }
    }

    /// Draw a sprite from raw ARGB data (skip transparent pixels: alpha == 0)
    pub fn blit_sprite(&mut self, x: i32, y: i32, w: u32, h: u32, data: &[u32]) {
        for sy in 0..h {
            let dy = y + sy as i32;
            if dy < 0 || dy >= self.height as i32 { continue; }
            for sx in 0..w {
                let dx = x + sx as i32;
                if dx < 0 || dx >= self.width as i32 { continue; }
                let src_idx = (sy * w + sx) as usize;
                if src_idx < data.len() {
                    let color = data[src_idx];
                    if color & 0xFF000000 != 0 {
                        self.buffer[(dy as u32 * self.width + dx as u32) as usize] = color;
                    }
                }
            }
        }
    }

    /// Upscale and blit internal buffer to native framebuffer, then swap
    pub fn present(&self) {
        let fb_w = crate::framebuffer::width();
        let fb_h = crate::framebuffer::height();
        if fb_w == 0 || fb_h == 0 { return; }

        let scale = core::cmp::min(fb_w / self.width, fb_h / self.height);
        if scale == 0 { return; }
        let off_x = (fb_w - self.width * scale) / 2;
        let off_y = (fb_h - self.height * scale) / 2;

        crate::framebuffer::begin_frame();

        // Black borders
        if off_x > 0 || off_y > 0 {
            crate::framebuffer::clear_backbuffer(0xFF000000);
        }

        // Upscale blit — write scale×scale blocks
        for iy in 0..self.height {
            let row_base = (iy * self.width) as usize;
            let dy_base = off_y + iy * scale;
            for ix in 0..self.width {
                let color = self.buffer[row_base + ix as usize];
                let dx_base = off_x + ix * scale;
                for sy in 0..scale {
                    for sx in 0..scale {
                        crate::framebuffer::put_pixel_fast(dx_base + sx, dy_base + sy, color);
                    }
                }
            }
        }

        crate::framebuffer::end_frame();
        crate::framebuffer::swap_buffers();
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
}
