//! COSMIC Renderer - tiny-skia based rendering to framebuffer
//!
//! Provides high-quality 2D rendering with:
//! - Anti-aliased shapes
//! - Rounded rectangles
//! - Drop shadows
//! - Gradient fills
//! - Path rendering

use tiny_skia::{Pixmap, PixmapMut, Paint, PathBuilder, Transform, FillRule, Stroke, LineCap, Color as SkiaColor};
use tiny_skia_path::Path;
use super::{Color, Rect, Point, CosmicTheme, theme};

/// COSMIC Renderer using tiny-skia
pub struct CosmicRenderer {
    pixmap: Pixmap,
    width: u32,
    height: u32,
}

impl CosmicRenderer {
    /// Create a new renderer with given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let pixmap = Pixmap::new(width, height).expect("Failed to create pixmap");
        Self { pixmap, width, height }
    }
    
    /// Get dimensions
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    /// Clear with a color - FAST direct pixel write
    pub fn clear(&mut self, color: Color) {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        
        let data = self.pixmap.data_mut();
        let pixels = (self.width * self.height) as usize;
        
        // Write 4 bytes per pixel directly
        for i in 0..pixels {
            let idx = i * 4;
            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = a;
        }
    }
    
    /// Fill a rectangle - FAST direct pixel write (no anti-aliasing)
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        
        let x0 = (rect.x as i32).max(0) as u32;
        let y0 = (rect.y as i32).max(0) as u32;
        let x1 = ((rect.x + rect.width) as u32).min(self.width);
        let y1 = ((rect.y + rect.height) as u32).min(self.height);
        
        let data = self.pixmap.data_mut();
        let stride = self.width as usize * 4;
        
        if a == 255 {
            // Opaque: direct write
            for y in y0..y1 {
                let row_start = y as usize * stride;
                for x in x0..x1 {
                    let idx = row_start + x as usize * 4;
                    data[idx] = r;
                    data[idx + 1] = g;
                    data[idx + 2] = b;
                    data[idx + 3] = a;
                }
            }
        } else {
            // Alpha blend
            let alpha = a as u32;
            let inv_alpha = 255 - alpha;
            for y in y0..y1 {
                let row_start = y as usize * stride;
                for x in x0..x1 {
                    let idx = row_start + x as usize * 4;
                    data[idx] = ((r as u32 * alpha + data[idx] as u32 * inv_alpha) / 255) as u8;
                    data[idx + 1] = ((g as u32 * alpha + data[idx + 1] as u32 * inv_alpha) / 255) as u8;
                    data[idx + 2] = ((b as u32 * alpha + data[idx + 2] as u32 * inv_alpha) / 255) as u8;
                    data[idx + 3] = 255;
                }
            }
        }
    }
    
    /// Fill a rounded rectangle - uses fast rect for speed, corners are approximate
    pub fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        // For speed, just draw a regular rect if radius is small
        if radius <= 2.0 {
            self.fill_rect(rect, color);
            return;
        }
        
        // Draw main body (fast)
        self.fill_rect(Rect::new(rect.x + radius, rect.y, rect.width - radius * 2.0, rect.height), color);
        self.fill_rect(Rect::new(rect.x, rect.y + radius, rect.width, rect.height - radius * 2.0), color);
        
        // Draw corners with tiny-skia for anti-aliasing (only 4 small circles)
        let r = radius;
        self.fill_circle(Point::new(rect.x + r, rect.y + r), r, color);
        self.fill_circle(Point::new(rect.x + rect.width - r, rect.y + r), r, color);
        self.fill_circle(Point::new(rect.x + r, rect.y + rect.height - r), r, color);
        self.fill_circle(Point::new(rect.x + rect.width - r, rect.y + rect.height - r), r, color);
    }
    
    /// Stroke a rounded rectangle border
    pub fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color, width: f32) {
        let path = rounded_rect_path(rect, radius);
        let mut paint = Paint::default();
        paint.set_color(to_skia_color(color));
        paint.anti_alias = true;
        
        let stroke = Stroke {
            width,
            line_cap: LineCap::Round,
            ..Default::default()
        };
        
        self.pixmap.stroke_path(
            &path,
            &paint,
            &stroke,
            Transform::identity(),
            None,
        );
    }
    
    /// Fill a circle - FAST for small circles, tiny-skia for large
    pub fn fill_circle(&mut self, center: Point, radius: f32, color: Color) {
        // For very small circles, use fast direct pixel draw
        if radius <= 10.0 {
            self.fill_circle_fast(center, radius, color);
            return;
        }
        
        // Large circles use tiny-skia for quality
        let path = circle_path(center, radius);
        let mut paint = Paint::default();
        paint.set_color(to_skia_color(color));
        paint.anti_alias = true;
        
        self.pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
    
    /// Fast circle drawing using midpoint algorithm (no anti-aliasing)
    fn fill_circle_fast(&mut self, center: Point, radius: f32, color: Color) {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        
        let cx = center.x as i32;
        let cy = center.y as i32;
        let rad = radius as i32;
        let rad_sq = rad * rad;
        
        let data = self.pixmap.data_mut();
        let stride = self.width as usize * 4;
        let w = self.width as i32;
        let h = self.height as i32;
        
        // Simple filled circle using squared distance check
        for dy in -rad..=rad {
            let py = cy + dy;
            if py < 0 || py >= h { continue; }
            
            let row_start = py as usize * stride;
            let dy_sq = dy * dy;
            
            for dx in -rad..=rad {
                // Check if point is inside circle
                if dx * dx + dy_sq > rad_sq { continue; }
                
                let px = cx + dx;
                if px < 0 || px >= w { continue; }
                
                let idx = row_start + px as usize * 4;
                if a == 255 {
                    data[idx] = r;
                    data[idx + 1] = g;
                    data[idx + 2] = b;
                    data[idx + 3] = a;
                } else {
                    let alpha = a as u32;
                    let inv_alpha = 255 - alpha;
                    data[idx] = ((r as u32 * alpha + data[idx] as u32 * inv_alpha) / 255) as u8;
                    data[idx + 1] = ((g as u32 * alpha + data[idx + 1] as u32 * inv_alpha) / 255) as u8;
                    data[idx + 2] = ((b as u32 * alpha + data[idx + 2] as u32 * inv_alpha) / 255) as u8;
                    data[idx + 3] = 255;
                }
            }
        }
    }
    
    /// Draw a line
    pub fn draw_line(&mut self, from: Point, to: Point, color: Color, width: f32) {
        let mut pb = PathBuilder::new();
        pb.move_to(from.x, from.y);
        pb.line_to(to.x, to.y);
        
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(to_skia_color(color));
            paint.anti_alias = true;
            
            let stroke = Stroke {
                width,
                line_cap: LineCap::Round,
                ..Default::default()
            };
            
            self.pixmap.stroke_path(
                &path,
                &paint,
                &stroke,
                Transform::identity(),
                None,
            );
        }
    }
    
    /// Draw shadow behind a rounded rect
    pub fn draw_shadow(&mut self, rect: Rect, radius: f32, blur: f32, color: Color) {
        // Simple multi-layer shadow effect
        let layers = (blur / 2.0) as i32;
        for i in 0..layers {
            let expand = i as f32;
            let alpha = color.a * (1.0 - (i as f32 / layers as f32));
            let shadow_color = Color::new(color.r, color.g, color.b, alpha * 0.3);
            
            let shadow_rect = Rect::new(
                rect.x - expand + blur / 2.0,
                rect.y - expand + blur,
                rect.width + expand * 2.0,
                rect.height + expand * 2.0,
            );
            
            self.fill_rounded_rect(shadow_rect, radius + expand, shadow_color);
        }
    }
    
    /// Draw vertical gradient
    pub fn fill_gradient_v(&mut self, rect: Rect, top: Color, bottom: Color) {
        let steps = rect.height as i32;
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let color = top.blend(bottom, t);
            let line_rect = Rect::new(rect.x, rect.y + i as f32, rect.width, 1.0);
            self.fill_rect(line_rect, color);
        }
    }
    
    /// Copy to framebuffer - Simple and fast scalar version
    pub fn present_to_framebuffer(&self) {
        let data = self.pixmap.data();
        let fb_addr = crate::framebuffer::get_fb_addr();
        let fb_pitch = crate::framebuffer::get_fb_pitch();
        
        // Simple row-by-row copy with RGBA->ARGB conversion
        for y in 0..self.height {
            let src_offset = (y * self.width) as usize * 4;
            let dst_offset = y as usize * fb_pitch;
            
            unsafe {
                let src = data.as_ptr().add(src_offset) as *const u32;
                let dst = fb_addr.add(dst_offset) as *mut u32;
                
                for x in 0..self.width as usize {
                    let rgba = *src.add(x);
                    // RGBA to ARGB: swap R and B
                    let r = (rgba >> 0) & 0xFF;
                    let g = (rgba >> 8) & 0xFF;  
                    let b = (rgba >> 16) & 0xFF;
                    let a = (rgba >> 24) & 0xFF;
                    *dst.add(x) = (a << 24) | (r << 16) | (g << 8) | b;
                }
            }
        }
    }
    
    /// Blit to backbuffer for double buffering - OPTIMIZED
    pub fn present_to_backbuffer(&self) {
        let data = self.pixmap.data();
        
        if let Some((ptr, bb_width, bb_height, _stride)) = crate::framebuffer::get_backbuffer_info() {
            let pixels = self.width.min(bb_width) * self.height.min(bb_height);
            
            unsafe {
                let src_u32 = data.as_ptr() as *const u32;
                let dst_u32 = ptr as *mut u32;
                
                for i in 0..pixels as usize {
                    let rgba = *src_u32.add(i);
                    let r = (rgba >> 0) & 0xFF;
                    let g = (rgba >> 8) & 0xFF;
                    let b = (rgba >> 16) & 0xFF;
                    let a = (rgba >> 24) & 0xFF;
                    let argb = (a << 24) | (r << 16) | (g << 8) | b;
                    *dst_u32.add(i) = argb;
                }
            }
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════════════
    // COSMIC Widget rendering helpers
    // ═══════════════════════════════════════════════════════════════════════════════
    
    /// Draw a COSMIC-style button
    pub fn draw_button(&mut self, rect: Rect, label: &str, state: ButtonState) {
        let t = theme();
        
        let bg = match state {
            ButtonState::Normal => t.button_bg,
            ButtonState::Hovered => t.button_hover,
            ButtonState::Pressed => t.button_pressed,
            ButtonState::Suggested => t.button_suggested,
            ButtonState::Destructive => t.button_destructive,
        };
        
        // Shadow for elevated look
        if matches!(state, ButtonState::Hovered | ButtonState::Suggested) {
            self.draw_shadow(rect, t.corner_radius, 4.0, Color::BLACK.with_alpha(0.3));
        }
        
        // Background
        self.fill_rounded_rect(rect, t.corner_radius, bg);
        
        // Border
        if matches!(state, ButtonState::Normal | ButtonState::Hovered) {
            self.stroke_rounded_rect(rect, t.corner_radius, t.border, 1.0);
        }
        
        // Text would go here (need font rendering)
        // For now, we'll just indicate with a centered dot
        let cx = rect.x + rect.width / 2.0;
        let cy = rect.y + rect.height / 2.0;
        self.fill_circle(Point::new(cx, cy), 3.0, t.text_primary);
    }
    
    /// Draw a COSMIC-style header/titlebar
    pub fn draw_header(&mut self, rect: Rect, title: &str, focused: bool) {
        let t = theme();
        
        // Background
        let bg = if focused { t.header_bg } else { t.header_bg.darken(0.05) };
        self.fill_rect(rect, bg);
        
        // Bottom border
        self.draw_line(
            Point::new(rect.x, rect.y + rect.height - 1.0),
            Point::new(rect.x + rect.width, rect.y + rect.height - 1.0),
            t.bg_divider,
            1.0,
        );
        
        // Window controls (right side)
        let btn_size = 14.0;
        let btn_y = rect.y + (rect.height - btn_size) / 2.0;
        let btn_spacing = 8.0;
        
        // Close button (rightmost)
        let close_x = rect.x + rect.width - btn_size - 12.0;
        self.fill_circle(Point::new(close_x + btn_size/2.0, btn_y + btn_size/2.0), btn_size/2.0, t.close_bg);
        
        // Maximize button
        let max_x = close_x - btn_size - btn_spacing;
        self.fill_circle(Point::new(max_x + btn_size/2.0, btn_y + btn_size/2.0), btn_size/2.0, t.maximize_bg);
        
        // Minimize button  
        let min_x = max_x - btn_size - btn_spacing;
        self.fill_circle(Point::new(min_x + btn_size/2.0, btn_y + btn_size/2.0), btn_size/2.0, t.minimize_bg);
    }
    
    /// Draw COSMIC-style panel (top bar)
    pub fn draw_panel(&mut self, rect: Rect) {
        let t = theme();
        
        // Semi-transparent background
        self.fill_rect(rect, t.panel_bg);
        
        // Bottom border
        self.draw_line(
            Point::new(rect.x, rect.y + rect.height),
            Point::new(rect.x + rect.width, rect.y + rect.height),
            t.bg_divider,
            1.0,
        );
    }
    
    /// Draw COSMIC-style dock
    pub fn draw_dock(&mut self, rect: Rect, items: &[DockItem]) {
        let t = theme();
        
        // Background with rounded corners
        self.fill_rounded_rect(rect, 12.0, t.panel_bg);
        
        // Draw each dock item
        let item_size = 48.0;
        let padding = 8.0;
        let mut y = rect.y + padding;
        
        for item in items {
            let item_rect = Rect::new(rect.x + padding, y, item_size, item_size);
            
            if item.active {
                // Active indicator
                self.fill_rounded_rect(item_rect, 8.0, t.accent.with_alpha(0.3));
            } else if item.hovered {
                self.fill_rounded_rect(item_rect, 8.0, t.panel_hover);
            }
            
            // Icon placeholder (circle)
            let cx = item_rect.x + item_rect.width / 2.0;
            let cy = item_rect.y + item_rect.height / 2.0;
            self.fill_circle(Point::new(cx, cy), 16.0, t.text_secondary);
            
            // Running indicator dot
            if item.running {
                self.fill_circle(
                    Point::new(rect.x + rect.width - 4.0, cy),
                    3.0,
                    t.accent,
                );
            }
            
            y += item_size + padding;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Suggested,
    Destructive,
}

pub struct DockItem {
    pub name: &'static str,
    pub active: bool,
    pub hovered: bool,
    pub running: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PATH BUILDERS
// ═══════════════════════════════════════════════════════════════════════════════

fn to_skia_color(c: Color) -> SkiaColor {
    SkiaColor::from_rgba(c.r, c.g, c.b, c.a).unwrap_or(SkiaColor::BLACK)
}

fn rect_path(r: Rect) -> Path {
    let mut pb = PathBuilder::new();
    pb.move_to(r.x, r.y);
    pb.line_to(r.x + r.width, r.y);
    pb.line_to(r.x + r.width, r.y + r.height);
    pb.line_to(r.x, r.y + r.height);
    pb.close();
    pb.finish().unwrap()
}

fn rounded_rect_path(r: Rect, radius: f32) -> Path {
    let mut pb = PathBuilder::new();
    let rad = radius.min(r.width / 2.0).min(r.height / 2.0);
    
    // Start at top-left after corner
    pb.move_to(r.x + rad, r.y);
    
    // Top edge
    pb.line_to(r.x + r.width - rad, r.y);
    // Top-right corner
    pb.quad_to(r.x + r.width, r.y, r.x + r.width, r.y + rad);
    
    // Right edge
    pb.line_to(r.x + r.width, r.y + r.height - rad);
    // Bottom-right corner
    pb.quad_to(r.x + r.width, r.y + r.height, r.x + r.width - rad, r.y + r.height);
    
    // Bottom edge
    pb.line_to(r.x + rad, r.y + r.height);
    // Bottom-left corner
    pb.quad_to(r.x, r.y + r.height, r.x, r.y + r.height - rad);
    
    // Left edge
    pb.line_to(r.x, r.y + rad);
    // Top-left corner
    pb.quad_to(r.x, r.y, r.x + rad, r.y);
    
    pb.close();
    pb.finish().unwrap()
}

fn circle_path(center: Point, radius: f32) -> Path {
    let mut pb = PathBuilder::new();
    
    // Approximate circle with bezier curves
    let k = 0.5522847498; // Magic number for circle approximation
    let kr = k * radius;
    
    pb.move_to(center.x + radius, center.y);
    pb.cubic_to(
        center.x + radius, center.y + kr,
        center.x + kr, center.y + radius,
        center.x, center.y + radius,
    );
    pb.cubic_to(
        center.x - kr, center.y + radius,
        center.x - radius, center.y + kr,
        center.x - radius, center.y,
    );
    pb.cubic_to(
        center.x - radius, center.y - kr,
        center.x - kr, center.y - radius,
        center.x, center.y - radius,
    );
    pb.cubic_to(
        center.x + kr, center.y - radius,
        center.x + radius, center.y - kr,
        center.x + radius, center.y,
    );
    pb.close();
    
    pb.finish().unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEXT RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

impl CosmicRenderer {
    /// Draw text at position using 8x16 bitmap font
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, color: Color) {
        let mut cx = x as i32;
        let cy = y as i32;
        
        for c in text.chars() {
            if c == ' ' {
                cx += 8;
                continue;
            }
            self.draw_char(cx, cy, c, color);
            cx += 8;
        }
    }
    
    /// Draw text centered in a rect
    pub fn draw_text_centered(&mut self, text: &str, rect: Rect, color: Color) {
        let text_width = (text.len() * 8) as f32;
        let text_height = 16.0f32;
        let x = rect.x + (rect.width - text_width) / 2.0;
        let y = rect.y + (rect.height - text_height) / 2.0;
        self.draw_text(text, x, y, color);
    }
    
    /// Draw a single character using 8x16 bitmap font
    fn draw_char(&mut self, x: i32, y: i32, c: char, color: Color) {
        let glyph = crate::framebuffer::font::get_glyph(c);
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        
        let data = self.pixmap.data_mut();
        let stride = self.width as usize * 4;
        
        for (row, &glyph_byte) in glyph.iter().enumerate() {
            let py = y + row as i32;
            if py < 0 || py >= self.height as i32 {
                continue;
            }
            
            for bit in 0..8 {
                let px = x + bit;
                if px < 0 || px >= self.width as i32 {
                    continue;
                }
                
                if (glyph_byte >> (7 - bit)) & 1 != 0 {
                    let idx = py as usize * stride + px as usize * 4;
                    if idx + 3 < data.len() {
                        data[idx] = r;
                        data[idx + 1] = g;
                        data[idx + 2] = b;
                        data[idx + 3] = a;
                    }
                }
            }
        }
    }
    
    /// Draw text with shadow for better readability
    pub fn draw_text_shadow(&mut self, text: &str, x: f32, y: f32, color: Color, shadow: Color) {
        self.draw_text(text, x + 1.0, y + 1.0, shadow);
        self.draw_text(text, x, y, color);
    }
    
    /// Draw a triangle (for arrows/indicators)
    pub fn fill_triangle(&mut self, p1: Point, p2: Point, p3: Point, color: Color) {
        let mut pb = PathBuilder::new();
        pb.move_to(p1.x, p1.y);
        pb.line_to(p2.x, p2.y);
        pb.line_to(p3.x, p3.y);
        pb.close();
        
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(to_skia_color(color));
            paint.anti_alias = true;
            
            self.pixmap.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }
    
    /// Draw a progress bar
    pub fn draw_progress_bar(&mut self, rect: Rect, progress: f32, bg: Color, fg: Color, border: Color) {
        // Background
        self.fill_rounded_rect(rect, 4.0, bg);
        
        // Progress fill
        let fill_width = rect.width * progress.clamp(0.0, 1.0);
        if fill_width > 0.0 {
            let fill_rect = Rect::new(rect.x, rect.y, fill_width, rect.height);
            self.fill_rounded_rect(fill_rect, 4.0, fg);
        }
        
        // Border
        self.stroke_rounded_rect(rect, 4.0, border, 1.0);
    }
}
