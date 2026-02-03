//! 2D Rendering with embedded-graphics
//!
//! Provides a DrawTarget implementation for the framebuffer
//! and high-level 2D drawing primitives.

use alloc::vec::Vec;
use alloc::boxed::Box;
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{Dimensions, OriginDimensions, Size as EgSize},
    pixelcolor::{Rgb888, RgbColor},
    Pixel,
};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, Ellipse, Line, Rectangle, RoundedRectangle, Triangle,
    PrimitiveStyle, PrimitiveStyleBuilder, StrokeAlignment,
    CornerRadii,
};
use embedded_graphics::mono_font::{ascii::FONT_8X13, MonoTextStyle};
use embedded_graphics::text::Text;

// ═══════════════════════════════════════════════════════════════════════════════
// FRAMEBUFFER DRAW TARGET
// ═══════════════════════════════════════════════════════════════════════════════

/// Wraps a raw framebuffer as an embedded-graphics DrawTarget
pub struct FramebufferTarget {
    pub buffer: *mut u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32, // Pixels per row (may be > width for alignment)
}

impl FramebufferTarget {
    /// Create a new framebuffer target
    /// # Safety
    /// The buffer pointer must be valid for the lifetime of this object
    pub unsafe fn new(buffer: *mut u32, width: u32, height: u32, stride: u32) -> Self {
        Self { buffer, width, height, stride }
    }

    /// Create from a slice
    pub fn from_slice(buffer: &mut [u32], width: u32, height: u32) -> Self {
        Self {
            buffer: buffer.as_mut_ptr(),
            width,
            height,
            stride: width,
        }
    }

    /// Set a pixel directly
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            unsafe {
                let offset = (y * self.stride + x) as isize;
                *self.buffer.offset(offset) = color;
            }
        }
    }

    /// Get a pixel
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            unsafe {
                let offset = (y * self.stride + x) as isize;
                *self.buffer.offset(offset)
            }
        } else {
            0
        }
    }

    /// Clear with a color
    pub fn clear_color(&mut self, color: u32) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x, y, color);
            }
        }
    }

    /// Get the width
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }
}

// Implement embedded-graphics traits
impl OriginDimensions for FramebufferTarget {
    fn size(&self) -> EgSize {
        EgSize::new(self.width, self.height)
    }
}

impl DrawTarget for FramebufferTarget {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= 0 && coord.y >= 0 
                && (coord.x as u32) < self.width 
                && (coord.y as u32) < self.height 
            {
                let c = ((color.r() as u32) << 16)
                    | ((color.g() as u32) << 8)
                    | (color.b() as u32)
                    | 0xFF000000;
                self.set_pixel(coord.x as u32, coord.y as u32, c);
            }
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HIGH-LEVEL 2D DRAWING API
// ═══════════════════════════════════════════════════════════════════════════════

/// Color for 2D drawing
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Color2D {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color2D {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
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

    pub fn to_rgb888(self) -> Rgb888 {
        Rgb888::new(self.r, self.g, self.b)
    }

    // Predefined colors
    pub const BLACK: Color2D = Color2D::rgb(0, 0, 0);
    pub const WHITE: Color2D = Color2D::rgb(255, 255, 255);
    pub const RED: Color2D = Color2D::rgb(255, 0, 0);
    pub const GREEN: Color2D = Color2D::rgb(0, 255, 0);
    pub const BLUE: Color2D = Color2D::rgb(0, 0, 255);
    pub const YELLOW: Color2D = Color2D::rgb(255, 255, 0);
    pub const CYAN: Color2D = Color2D::rgb(0, 255, 255);
    pub const MAGENTA: Color2D = Color2D::rgb(255, 0, 255);
    pub const GRAY: Color2D = Color2D::rgb(128, 128, 128);
    pub const DARK_GRAY: Color2D = Color2D::rgb(64, 64, 64);
    pub const LIGHT_GRAY: Color2D = Color2D::rgb(192, 192, 192);
    pub const ORANGE: Color2D = Color2D::rgb(255, 165, 0);
    pub const PURPLE: Color2D = Color2D::rgb(128, 0, 128);
    pub const TRANSPARENT: Color2D = Color2D::new(0, 0, 0, 0);
}

/// 2D Renderer using embedded-graphics
pub struct Renderer2D<'a> {
    target: &'a mut FramebufferTarget,
}

impl<'a> Renderer2D<'a> {
    pub fn new(target: &'a mut FramebufferTarget) -> Self {
        Self { target }
    }

    /// Clear screen with color
    pub fn clear(&mut self, color: Color2D) {
        self.target.clear_color(color.to_u32());
    }

    /// Draw a line
    pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color2D, thickness: u32) {
        let style = PrimitiveStyle::with_stroke(color.to_rgb888(), thickness);
        let _ = Line::new(Point::new(x1, y1), Point::new(x2, y2))
            .into_styled(style)
            .draw(self.target);
    }

    /// Draw a rectangle outline
    pub fn rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color2D, thickness: u32) {
        let style = PrimitiveStyle::with_stroke(color.to_rgb888(), thickness);
        let _ = Rectangle::new(Point::new(x, y), EgSize::new(w, h))
            .into_styled(style)
            .draw(self.target);
    }

    /// Fill a rectangle
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color2D) {
        let style = PrimitiveStyle::with_fill(color.to_rgb888());
        let _ = Rectangle::new(Point::new(x, y), EgSize::new(w, h))
            .into_styled(style)
            .draw(self.target);
    }

    /// Draw a rounded rectangle
    pub fn rounded_rect(&mut self, x: i32, y: i32, w: u32, h: u32, radius: u32, color: Color2D, thickness: u32) {
        let style = PrimitiveStyle::with_stroke(color.to_rgb888(), thickness);
        let _ = RoundedRectangle::new(
            Rectangle::new(Point::new(x, y), EgSize::new(w, h)),
            CornerRadii::new(EgSize::new(radius, radius)),
        )
        .into_styled(style)
        .draw(self.target);
    }

    /// Fill a rounded rectangle
    pub fn fill_rounded_rect(&mut self, x: i32, y: i32, w: u32, h: u32, radius: u32, color: Color2D) {
        let style = PrimitiveStyle::with_fill(color.to_rgb888());
        let _ = RoundedRectangle::new(
            Rectangle::new(Point::new(x, y), EgSize::new(w, h)),
            CornerRadii::new(EgSize::new(radius, radius)),
        )
        .into_styled(style)
        .draw(self.target);
    }

    /// Draw a circle outline
    pub fn circle(&mut self, cx: i32, cy: i32, radius: u32, color: Color2D, thickness: u32) {
        let style = PrimitiveStyle::with_stroke(color.to_rgb888(), thickness);
        let _ = Circle::new(Point::new(cx - radius as i32, cy - radius as i32), radius * 2)
            .into_styled(style)
            .draw(self.target);
    }

    /// Fill a circle
    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: u32, color: Color2D) {
        let style = PrimitiveStyle::with_fill(color.to_rgb888());
        let _ = Circle::new(Point::new(cx - radius as i32, cy - radius as i32), radius * 2)
            .into_styled(style)
            .draw(self.target);
    }

    /// Draw an ellipse outline
    pub fn ellipse(&mut self, cx: i32, cy: i32, rx: u32, ry: u32, color: Color2D, thickness: u32) {
        let style = PrimitiveStyle::with_stroke(color.to_rgb888(), thickness);
        let _ = Ellipse::new(Point::new(cx - rx as i32, cy - ry as i32), EgSize::new(rx * 2, ry * 2))
            .into_styled(style)
            .draw(self.target);
    }

    /// Fill an ellipse
    pub fn fill_ellipse(&mut self, cx: i32, cy: i32, rx: u32, ry: u32, color: Color2D) {
        let style = PrimitiveStyle::with_fill(color.to_rgb888());
        let _ = Ellipse::new(Point::new(cx - rx as i32, cy - ry as i32), EgSize::new(rx * 2, ry * 2))
            .into_styled(style)
            .draw(self.target);
    }

    /// Draw a triangle outline
    pub fn triangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, color: Color2D, thickness: u32) {
        let style = PrimitiveStyle::with_stroke(color.to_rgb888(), thickness);
        let _ = Triangle::new(
            Point::new(x1, y1),
            Point::new(x2, y2),
            Point::new(x3, y3),
        )
        .into_styled(style)
        .draw(self.target);
    }

    /// Fill a triangle
    pub fn fill_triangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, color: Color2D) {
        let style = PrimitiveStyle::with_fill(color.to_rgb888());
        let _ = Triangle::new(
            Point::new(x1, y1),
            Point::new(x2, y2),
            Point::new(x3, y3),
        )
        .into_styled(style)
        .draw(self.target);
    }

    /// Draw text
    pub fn text(&mut self, x: i32, y: i32, text: &str, color: Color2D) {
        let style = MonoTextStyle::new(&FONT_8X13, color.to_rgb888());
        let _ = Text::new(text, Point::new(x, y), style).draw(self.target);
    }

    /// Draw a gradient rectangle (vertical)
    pub fn gradient_rect_v(&mut self, x: i32, y: i32, w: u32, h: u32, top: Color2D, bottom: Color2D) {
        for row in 0..h {
            let t = row as f32 / h as f32;
            let r = (top.r as f32 + (bottom.r as f32 - top.r as f32) * t) as u8;
            let g = (top.g as f32 + (bottom.g as f32 - top.g as f32) * t) as u8;
            let b = (top.b as f32 + (bottom.b as f32 - top.b as f32) * t) as u8;
            let color = Color2D::rgb(r, g, b);
            
            for col in 0..w {
                self.target.set_pixel((x + col as i32) as u32, (y + row as i32) as u32, color.to_u32());
            }
        }
    }

    /// Draw a gradient rectangle (horizontal)
    pub fn gradient_rect_h(&mut self, x: i32, y: i32, w: u32, h: u32, left: Color2D, right: Color2D) {
        for col in 0..w {
            let t = col as f32 / w as f32;
            let r = (left.r as f32 + (right.r as f32 - left.r as f32) * t) as u8;
            let g = (left.g as f32 + (right.g as f32 - left.g as f32) * t) as u8;
            let b = (left.b as f32 + (right.b as f32 - left.b as f32) * t) as u8;
            let color = Color2D::rgb(r, g, b);
            
            for row in 0..h {
                self.target.set_pixel((x + col as i32) as u32, (y + row as i32) as u32, color.to_u32());
            }
        }
    }

    /// Draw a shadow effect
    pub fn shadow(&mut self, x: i32, y: i32, w: u32, h: u32, blur: u32, color: Color2D) {
        let alpha_step = color.a as f32 / blur as f32;
        for i in 0..blur {
            let alpha = (color.a as f32 - alpha_step * i as f32) as u8;
            let shadow_color = Color2D::new(color.r, color.g, color.b, alpha);
            self.rect(
                x - i as i32,
                y - i as i32,
                w + i * 2,
                h + i * 2,
                shadow_color,
                1,
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SPRITE / IMAGE
// ═══════════════════════════════════════════════════════════════════════════════

/// Simple sprite/image
pub struct Sprite {
    pub width: u32,
    pub height: u32,
    pub pixels: Box<[u32]>,
}

impl Sprite {
    /// Create empty sprite
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            pixels: alloc::vec![0u32; size].into_boxed_slice(),
        }
    }

    /// Create from pixel data
    pub fn from_pixels(width: u32, height: u32, pixels: Vec<u32>) -> Self {
        Self {
            width,
            height,
            pixels: pixels.into_boxed_slice(),
        }
    }

    /// Get pixel at position
    pub fn get(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize]
        } else {
            0
        }
    }

    /// Set pixel at position
    pub fn set(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize] = color;
        }
    }

    /// Draw sprite to framebuffer
    pub fn draw(&self, target: &mut FramebufferTarget, x: i32, y: i32) {
        for sy in 0..self.height {
            for sx in 0..self.width {
                let px = x + sx as i32;
                let py = y + sy as i32;
                if px >= 0 && py >= 0 {
                    let color = self.get(sx, sy);
                    if (color >> 24) > 0 { // Check alpha
                        target.set_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }
    }

    /// Draw sprite with alpha blending
    pub fn draw_blended(&self, target: &mut FramebufferTarget, x: i32, y: i32) {
        for sy in 0..self.height {
            for sx in 0..self.width {
                let px = x + sx as i32;
                let py = y + sy as i32;
                if px >= 0 && py >= 0 && (px as u32) < target.width && (py as u32) < target.height {
                    let src = self.get(sx, sy);
                    let src_a = ((src >> 24) & 0xFF) as f32 / 255.0;
                    
                    if src_a > 0.0 {
                        if src_a >= 1.0 {
                            target.set_pixel(px as u32, py as u32, src);
                        } else {
                            let dst = target.get_pixel(px as u32, py as u32);
                            let dst_a = 1.0 - src_a;
                            
                            let r = (((src >> 16) & 0xFF) as f32 * src_a + ((dst >> 16) & 0xFF) as f32 * dst_a) as u32;
                            let g = (((src >> 8) & 0xFF) as f32 * src_a + ((dst >> 8) & 0xFF) as f32 * dst_a) as u32;
                            let b = ((src & 0xFF) as f32 * src_a + (dst & 0xFF) as f32 * dst_a) as u32;
                            
                            let blended = 0xFF000000 | (r << 16) | (g << 8) | b;
                            target.set_pixel(px as u32, py as u32, blended);
                        }
                    }
                }
            }
        }
    }

    /// Scale sprite (nearest neighbor)
    pub fn scale(&self, new_width: u32, new_height: u32) -> Self {
        let mut scaled = Sprite::new(new_width, new_height);
        
        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x * self.width / new_width).min(self.width - 1);
                let src_y = (y * self.height / new_height).min(self.height - 1);
                scaled.set(x, y, self.get(src_x, src_y));
            }
        }
        
        scaled
    }
}
