




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






pub struct Hy {
    pub buffer: *mut u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32, 
}

impl Hy {
    
    
    
    pub unsafe fn new(buffer: *mut u32, width: u32, height: u32, stride: u32) -> Self {
        Self { buffer, width, height, stride }
    }

    
    pub fn qgn(buffer: &mut [u32], width: u32, height: u32) -> Self {
        Self {
            buffer: buffer.as_mut_ptr(),
            width,
            height,
            stride: width,
        }
    }

    
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            unsafe {
                let offset = (y * self.stride + x) as isize;
                *self.buffer.offset(offset) = color;
            }
        }
    }

    
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

    
    pub fn clear_color(&mut self, color: u32) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x, y, color);
            }
        }
    }

    
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }
}


impl OriginDimensions for Hy {
    fn size(&self) -> EgSize {
        EgSize::new(self.width, self.height)
    }
}

impl DrawTarget for Hy {
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

    pub const fn bdl(r: u8, g: u8, b: u8, a: u8) -> Self {
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

    
    pub const BLACK: Color2D = Color2D::rgb(0, 0, 0);
    pub const WHITE: Color2D = Color2D::rgb(255, 255, 255);
    pub const Acz: Color2D = Color2D::rgb(255, 0, 0);
    pub const Zf: Color2D = Color2D::rgb(0, 255, 0);
    pub const Wn: Color2D = Color2D::rgb(0, 0, 255);
    pub const Asf: Color2D = Color2D::rgb(255, 255, 0);
    pub const Ahy: Color2D = Color2D::rgb(0, 255, 255);
    pub const Amm: Color2D = Color2D::rgb(255, 0, 255);
    pub const Awt: Color2D = Color2D::rgb(128, 128, 128);
    pub const DMC_: Color2D = Color2D::rgb(64, 64, 64);
    pub const DWH_: Color2D = Color2D::rgb(192, 192, 192);
    pub const Azw: Color2D = Color2D::rgb(255, 165, 0);
    pub const Bao: Color2D = Color2D::rgb(128, 0, 128);
    pub const TRANSPARENT: Color2D = Color2D::new(0, 0, 0, 0);
}


pub struct Aon<'a> {
    target: &'a mut Hy,
}

impl<'a> Aon<'a> {
    pub fn new(target: &'a mut Hy) -> Self {
        Self { target }
    }

    
    pub fn clear(&mut self, color: Color2D) {
        self.target.clear_color(color.to_u32());
    }

    
    pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color2D, rh: u32) {
        let style = PrimitiveStyle::with_stroke(color.to_rgb888(), rh);
        let _ = Line::new(Point::new(x1, y1), Point::new(x2, y2))
            .into_styled(style)
            .draw(self.target);
    }

    
    pub fn rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color2D, rh: u32) {
        let style = PrimitiveStyle::with_stroke(color.to_rgb888(), rh);
        let _ = Rectangle::new(Point::new(x, y), EgSize::new(w, h))
            .into_styled(style)
            .draw(self.target);
    }

    
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color2D) {
        let style = PrimitiveStyle::with_fill(color.to_rgb888());
        let _ = Rectangle::new(Point::new(x, y), EgSize::new(w, h))
            .into_styled(style)
            .draw(self.target);
    }

    
    pub fn qum(&mut self, x: i32, y: i32, w: u32, h: u32, radius: u32, color: Color2D, rh: u32) {
        let style = PrimitiveStyle::with_stroke(color.to_rgb888(), rh);
        let _ = RoundedRectangle::new(
            Rectangle::new(Point::new(x, y), EgSize::new(w, h)),
            CornerRadii::new(EgSize::new(radius, radius)),
        )
        .into_styled(style)
        .draw(self.target);
    }

    
    pub fn fill_rounded_rect(&mut self, x: i32, y: i32, w: u32, h: u32, radius: u32, color: Color2D) {
        let style = PrimitiveStyle::with_fill(color.to_rgb888());
        let _ = RoundedRectangle::new(
            Rectangle::new(Point::new(x, y), EgSize::new(w, h)),
            CornerRadii::new(EgSize::new(radius, radius)),
        )
        .into_styled(style)
        .draw(self.target);
    }

    
    pub fn pzs(&mut self, cx: i32, u: i32, radius: u32, color: Color2D, rh: u32) {
        let style = PrimitiveStyle::with_stroke(color.to_rgb888(), rh);
        let _ = Circle::new(Point::new(cx - radius as i32, u - radius as i32), radius * 2)
            .into_styled(style)
            .draw(self.target);
    }

    
    pub fn fill_circle(&mut self, cx: i32, u: i32, radius: u32, color: Color2D) {
        let style = PrimitiveStyle::with_fill(color.to_rgb888());
        let _ = Circle::new(Point::new(cx - radius as i32, u - radius as i32), radius * 2)
            .into_styled(style)
            .draw(self.target);
    }

    
    pub fn qeo(&mut self, cx: i32, u: i32, da: u32, cm: u32, color: Color2D, rh: u32) {
        let style = PrimitiveStyle::with_stroke(color.to_rgb888(), rh);
        let _ = Ellipse::new(Point::new(cx - da as i32, u - cm as i32), EgSize::new(da * 2, cm * 2))
            .into_styled(style)
            .draw(self.target);
    }

    
    pub fn qfn(&mut self, cx: i32, u: i32, da: u32, cm: u32, color: Color2D) {
        let style = PrimitiveStyle::with_fill(color.to_rgb888());
        let _ = Ellipse::new(Point::new(cx - da as i32, u - cm as i32), EgSize::new(da * 2, cm * 2))
            .into_styled(style)
            .draw(self.target);
    }

    
    pub fn ray(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, bkf: i32, color: Color2D, rh: u32) {
        let style = PrimitiveStyle::with_stroke(color.to_rgb888(), rh);
        let _ = Triangle::new(
            Point::new(x1, y1),
            Point::new(x2, y2),
            Point::new(x3, bkf),
        )
        .into_styled(style)
        .draw(self.target);
    }

    
    pub fn fwt(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, bkf: i32, color: Color2D) {
        let style = PrimitiveStyle::with_fill(color.to_rgb888());
        let _ = Triangle::new(
            Point::new(x1, y1),
            Point::new(x2, y2),
            Point::new(x3, bkf),
        )
        .into_styled(style)
        .draw(self.target);
    }

    
    pub fn text(&mut self, x: i32, y: i32, text: &str, color: Color2D) {
        let style = MonoTextStyle::new(&FONT_8X13, color.to_rgb888());
        let _ = Text::new(text, Point::new(x, y), style).draw(self.target);
    }

    
    pub fn qju(&mut self, x: i32, y: i32, w: u32, h: u32, top: Color2D, bottom: Color2D) {
        if h == 0 || w == 0 { return; }
        for row in 0..h {
            let t = row as f32 / h as f32;
            let r = (top.r as f32 + (bottom.r as f32 - top.r as f32) * t) as u8;
            let g = (top.g as f32 + (bottom.g as f32 - top.g as f32) * t) as u8;
            let b = (top.b as f32 + (bottom.b as f32 - top.b as f32) * t) as u8;
            let color = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            
            let o = y + row as i32;
            if o < 0 || o >= self.target.height as i32 { continue; }
            let bdd = x.max(0) as u32;
            let dwz = ((x + w as i32) as u32).min(self.target.width);
            if dwz <= bdd { continue; }
            
            let pq = (o as u32 * self.target.stride + bdd) as usize;
            let dpn = (dwz - bdd) as usize;
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::adq(
                    self.target.buffer.add(pq),
                    dpn,
                    color,
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                for col in 0..dpn {
                    unsafe { *self.target.buffer.add(pq + col) = color; }
                }
            }
        }
    }

    
    pub fn qjt(&mut self, x: i32, y: i32, w: u32, h: u32, left: Color2D, right: Color2D) {
        if h == 0 || w == 0 { return; }
        
        let mut fzi = alloc::vec![0u32; w as usize];
        for col in 0..w {
            let t = col as f32 / w as f32;
            let r = (left.r as f32 + (right.r as f32 - left.r as f32) * t) as u8;
            let g = (left.g as f32 + (right.g as f32 - left.g as f32) * t) as u8;
            let b = (left.b as f32 + (right.b as f32 - left.b as f32) * t) as u8;
            fzi[col as usize] = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        }
        
        for row in 0..h {
            let o = y + row as i32;
            if o < 0 || o >= self.target.height as i32 { continue; }
            let bdd = x.max(0) as u32;
            let dwz = ((x + w as i32) as u32).min(self.target.width);
            if dwz <= bdd { continue; }
            
            let jhq = (bdd as i32 - x).max(0) as usize;
            let mb = (dwz - bdd) as usize;
            let afd = (o as u32 * self.target.stride + bdd) as usize;
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::blg(
                    self.target.buffer.add(afd),
                    fzi.as_ptr().add(jhq),
                    mb,
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                for i in 0..mb {
                    unsafe { *self.target.buffer.add(afd + i) = fzi[jhq + i]; }
                }
            }
        }
    }

    
    pub fn shadow(&mut self, x: i32, y: i32, w: u32, h: u32, awi: u32, color: Color2D) {
        let jve = color.a as f32 / awi as f32;
        for i in 0..awi {
            let alpha = (color.a as f32 - jve * i as f32) as u8;
            let bjd = Color2D::new(color.r, color.g, color.b, alpha);
            self.rect(
                x - i as i32,
                y - i as i32,
                w + i * 2,
                h + i * 2,
                bjd,
                1,
            );
        }
    }
}






pub struct Sprite {
    pub width: u32,
    pub height: u32,
    pub pixels: Box<[u32]>,
}

impl Sprite {
    
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            pixels: alloc::vec![0u32; size].into_boxed_slice(),
        }
    }

    
    pub fn cjv(width: u32, height: u32, pixels: Vec<u32>) -> Self {
        Self {
            width,
            height,
            pixels: pixels.into_boxed_slice(),
        }
    }

    
    pub fn get(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize]
        } else {
            0
        }
    }

    
    pub fn set(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize] = color;
        }
    }

    
    pub fn draw(&self, target: &mut Hy, x: i32, y: i32) {
        for ak in 0..self.height {
            for am in 0..self.width {
                let p = x + am as i32;
                let o = y + ak as i32;
                if p >= 0 && o >= 0 {
                    let color = self.get(am, ak);
                    if (color >> 24) > 0 { 
                        target.set_pixel(p as u32, o as u32, color);
                    }
                }
            }
        }
    }

    
    pub fn lic(&self, target: &mut Hy, x: i32, y: i32) {
        for ak in 0..self.height {
            for am in 0..self.width {
                let p = x + am as i32;
                let o = y + ak as i32;
                if p >= 0 && o >= 0 && (p as u32) < target.width && (o as u32) < target.height {
                    let src = self.get(am, ak);
                    let des = ((src >> 24) & 0xFF) as f32 / 255.0;
                    
                    if des > 0.0 {
                        if des >= 1.0 {
                            target.set_pixel(p as u32, o as u32, src);
                        } else {
                            let dst = target.get_pixel(p as u32, o as u32);
                            let fth = 1.0 - des;
                            
                            let r = (((src >> 16) & 0xFF) as f32 * des + ((dst >> 16) & 0xFF) as f32 * fth) as u32;
                            let g = (((src >> 8) & 0xFF) as f32 * des + ((dst >> 8) & 0xFF) as f32 * fth) as u32;
                            let b = ((src & 0xFF) as f32 * des + (dst & 0xFF) as f32 * fth) as u32;
                            
                            let bex = 0xFF000000 | (r << 16) | (g << 8) | b;
                            target.set_pixel(p as u32, o as u32, bex);
                        }
                    }
                }
            }
        }
    }

    
    pub fn scale(&self, aym: u32, ayk: u32) -> Self {
        let mut dyk = Sprite::new(aym, ayk);
        
        for y in 0..ayk {
            for x in 0..aym {
                let ahc = (x * self.width / aym).min(self.width - 1);
                let aft = (y * self.height / ayk).min(self.height - 1);
                dyk.set(x, y, self.get(ahc, aft));
            }
        }
        
        dyk
    }
}
