








use tiny_skia::{Pixmap, PixmapMut, Paint, PathBuilder, Transform, FillRule, Stroke, LineCap, Color as SkiaColor};
use tiny_skia_path::Path;
use super::{Color, Rect, Point, CosmicTheme, theme};


pub struct CosmicRenderer {
    pixmap: Pixmap,
    width: u32,
    height: u32,
}

impl CosmicRenderer {
    
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let pixmap = Pixmap::new(width, height)?;
        Some(Self { pixmap, width, height })
    }
    
    
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    
    pub fn clear(&mut self, color: Color) {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        
        let data = self.pixmap.data_mut();
        let pixels = (self.width * self.height) as usize;
        
        
        for i in 0..pixels {
            let idx = i * 4;
            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = a;
        }
    }
    
    
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        
        let bm = (rect.x as i32).max(0) as u32;
        let az = (rect.y as i32).max(0) as u32;
        let x1 = ((rect.x + rect.width) as u32).min(self.width);
        let y1 = ((rect.y + rect.height) as u32).min(self.height);
        
        let data = self.pixmap.data_mut();
        let stride = self.width as usize * 4;
        
        if a == 255 {
            
            for y in az..y1 {
                let fk = y as usize * stride;
                for x in bm..x1 {
                    let idx = fk + x as usize * 4;
                    data[idx] = r;
                    data[idx + 1] = g;
                    data[idx + 2] = b;
                    data[idx + 3] = a;
                }
            }
        } else {
            
            let alpha = a as u32;
            let sg = 255 - alpha;
            for y in az..y1 {
                let fk = y as usize * stride;
                for x in bm..x1 {
                    let idx = fk + x as usize * 4;
                    data[idx] = ((r as u32 * alpha + data[idx] as u32 * sg) / 255) as u8;
                    data[idx + 1] = ((g as u32 * alpha + data[idx + 1] as u32 * sg) / 255) as u8;
                    data[idx + 2] = ((b as u32 * alpha + data[idx + 2] as u32 * sg) / 255) as u8;
                    data[idx + 3] = 255;
                }
            }
        }
    }
    
    
    pub fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        
        if radius <= 2.0 {
            self.fill_rect(rect, color);
            return;
        }
        
        
        self.fill_rect(Rect::new(rect.x + radius, rect.y, rect.width - radius * 2.0, rect.height), color);
        self.fill_rect(Rect::new(rect.x, rect.y + radius, rect.width, rect.height - radius * 2.0), color);
        
        
        let r = radius;
        self.fill_circle(Point::new(rect.x + r, rect.y + r), r, color);
        self.fill_circle(Point::new(rect.x + rect.width - r, rect.y + r), r, color);
        self.fill_circle(Point::new(rect.x + r, rect.y + rect.height - r), r, color);
        self.fill_circle(Point::new(rect.x + rect.width - r, rect.y + rect.height - r), r, color);
    }
    
    
    pub fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color, width: f32) {
        let path = match oie(rect, radius) {
            Some(aa) => aa,
            None => return,
        };
        let mut ars = Paint::default();
        ars.set_color(fdc(color));
        ars.anti_alias = true;
        
        let gwo = Stroke {
            width,
            line_cap: LineCap::Round,
            ..Default::default()
        };
        
        self.pixmap.stroke_path(
            &path,
            &ars,
            &gwo,
            Transform::identity(),
            None,
        );
    }
    
    
    pub fn fill_circle(&mut self, center: Point, radius: f32, color: Color) {
        
        if radius <= 10.0 {
            self.fill_circle_fast(center, radius, color);
            return;
        }
        
        
        let path = match kkn(center, radius) {
            Some(aa) => aa,
            None => return,
        };
        let mut ars = Paint::default();
        ars.set_color(fdc(color));
        ars.anti_alias = true;
        
        self.pixmap.fill_path(
            &path,
            &ars,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
    
    
    fn fill_circle_fast(&mut self, center: Point, radius: f32, color: Color) {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        
        let cx = center.x as i32;
        let u = center.y as i32;
        let abf = radius as i32;
        let oav = abf * abf;
        
        let data = self.pixmap.data_mut();
        let stride = self.width as usize * 4;
        let w = self.width as i32;
        let h = self.height as i32;
        
        
        for ad in -abf..=abf {
            let o = u + ad;
            if o < 0 || o >= h { continue; }
            
            let fk = o as usize * stride;
            let dof = ad * ad;
            
            for dx in -abf..=abf {
                
                if dx * dx + dof > oav { continue; }
                
                let p = cx + dx;
                if p < 0 || p >= w { continue; }
                
                let idx = fk + p as usize * 4;
                if a == 255 {
                    data[idx] = r;
                    data[idx + 1] = g;
                    data[idx + 2] = b;
                    data[idx + 3] = a;
                } else {
                    let alpha = a as u32;
                    let sg = 255 - alpha;
                    data[idx] = ((r as u32 * alpha + data[idx] as u32 * sg) / 255) as u8;
                    data[idx + 1] = ((g as u32 * alpha + data[idx + 1] as u32 * sg) / 255) as u8;
                    data[idx + 2] = ((b as u32 * alpha + data[idx + 2] as u32 * sg) / 255) as u8;
                    data[idx + 3] = 255;
                }
            }
        }
    }
    
    
    pub fn draw_line(&mut self, from: Point, to: Point, color: Color, width: f32) {
        let mut ji = PathBuilder::new();
        ji.move_to(from.x, from.y);
        ji.line_to(to.x, to.y);
        
        if let Some(path) = ji.finish() {
            let mut ars = Paint::default();
            ars.set_color(fdc(color));
            ars.anti_alias = true;
            
            let gwo = Stroke {
                width,
                line_cap: LineCap::Round,
                ..Default::default()
            };
            
            self.pixmap.stroke_path(
                &path,
                &ars,
                &gwo,
                Transform::identity(),
                None,
            );
        }
    }
    
    
    pub fn draw_shadow(&mut self, rect: Rect, radius: f32, awi: f32, color: Color) {
        
        let layers = (awi / 2.0) as i32;
        for i in 0..layers {
            let wd = i as f32;
            let alpha = color.a * (1.0 - (i as f32 / layers as f32));
            let bjd = Color::new(color.r, color.g, color.b, alpha * 0.3);
            
            let ori = Rect::new(
                rect.x - wd + awi / 2.0,
                rect.y - wd + awi,
                rect.width + wd * 2.0,
                rect.height + wd * 2.0,
            );
            
            self.fill_rounded_rect(ori, radius + wd, bjd);
        }
    }
    
    
    pub fn fill_gradient_v(&mut self, rect: Rect, top: Color, bottom: Color) {
        let steps = rect.height as i32;
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let color = top.blend(bottom, t);
            let myn = Rect::new(rect.x, rect.y + i as f32, rect.width, 1.0);
            self.fill_rect(myn, color);
        }
    }
    
    
    pub fn present_to_framebuffer(&self) {
        let data = self.pixmap.data();
        let bme = crate::framebuffer::eob();
        let cjh = crate::framebuffer::fyo();
        
        
        for y in 0..self.height {
            let azu = (y * self.width) as usize * 4;
            let afd = y as usize * cjh;
            
            unsafe {
                let src = data.as_ptr().add(azu) as *const u32;
                let dst = bme.add(afd) as *mut u32;
                
                for x in 0..self.width as usize {
                    let bdl = *src.add(x);
                    
                    let r = (bdl >> 0) & 0xFF;
                    let g = (bdl >> 8) & 0xFF;  
                    let b = (bdl >> 16) & 0xFF;
                    let a = (bdl >> 24) & 0xFF;
                    *dst.add(x) = (a << 24) | (r << 16) | (g << 8) | b;
                }
            }
        }
    }
    
    
    pub fn qqy(&self) {
        let data = self.pixmap.data();
        
        if let Some((ptr, bb_width, bb_height, _stride)) = crate::framebuffer::aqr() {
            let pixels = self.width.min(bb_width) * self.height.min(bb_height);
            
            unsafe {
                let ovk = data.as_ptr() as *const u32;
                let lls = ptr as *mut u32;
                
                for i in 0..pixels as usize {
                    let bdl = *ovk.add(i);
                    let r = (bdl >> 0) & 0xFF;
                    let g = (bdl >> 8) & 0xFF;
                    let b = (bdl >> 16) & 0xFF;
                    let a = (bdl >> 24) & 0xFF;
                    let abq = (a << 24) | (r << 16) | (g << 8) | b;
                    *lls.add(i) = abq;
                }
            }
        }
    }
    
    
    
    
    
    
    pub fn draw_button(&mut self, rect: Rect, label: &str, state: ButtonState) {
        let t = theme();
        
        let bg = match state {
            ButtonState::Normal => t.button_bg,
            ButtonState::Hovered => t.button_hover,
            ButtonState::Pressed => t.button_pressed,
            ButtonState::Suggested => t.button_suggested,
            ButtonState::Destructive => t.button_destructive,
        };
        
        
        if matches!(state, ButtonState::Hovered | ButtonState::Suggested) {
            self.draw_shadow(rect, t.corner_radius, 4.0, Color::BLACK.with_alpha(0.3));
        }
        
        
        self.fill_rounded_rect(rect, t.corner_radius, bg);
        
        
        if matches!(state, ButtonState::Normal | ButtonState::Hovered) {
            self.stroke_rounded_rect(rect, t.corner_radius, t.border, 1.0);
        }
        
        
        
        let cx = rect.x + rect.width / 2.0;
        let u = rect.y + rect.height / 2.0;
        self.fill_circle(Point::new(cx, u), 3.0, t.text_primary);
    }
    
    
    pub fn draw_header(&mut self, rect: Rect, title: &str, focused: bool) {
        let t = theme();
        
        
        let bg = if focused { t.header_bg } else { t.header_bg.darken(0.05) };
        self.fill_rect(rect, bg);
        
        
        self.draw_line(
            Point::new(rect.x, rect.y + rect.height - 1.0),
            Point::new(rect.x + rect.width, rect.y + rect.height - 1.0),
            t.bg_divider,
            1.0,
        );
        
        
        let wv = 14.0;
        let ed = rect.y + (rect.height - wv) / 2.0;
        let hiy = 8.0;
        
        
        let adl = rect.x + rect.width - wv - 12.0;
        self.fill_circle(Point::new(adl + wv/2.0, ed + wv/2.0), wv/2.0, t.close_bg);
        
        
        let aly = adl - wv - hiy;
        self.fill_circle(Point::new(aly + wv/2.0, ed + wv/2.0), wv/2.0, t.maximize_bg);
        
        
        let ayg = aly - wv - hiy;
        self.fill_circle(Point::new(ayg + wv/2.0, ed + wv/2.0), wv/2.0, t.minimize_bg);
    }
    
    
    pub fn draw_panel(&mut self, rect: Rect) {
        let t = theme();
        
        
        self.fill_rect(rect, t.panel_bg);
        
        
        self.draw_line(
            Point::new(rect.x, rect.y + rect.height),
            Point::new(rect.x + rect.width, rect.y + rect.height),
            t.bg_divider,
            1.0,
        );
    }
    
    
    pub fn draw_dock(&mut self, rect: Rect, items: &[Ln]) {
        let t = theme();
        
        
        self.fill_rounded_rect(rect, 12.0, t.panel_bg);
        
        
        let geg = 48.0;
        let padding = 8.0;
        let mut y = rect.y + padding;
        
        for item in items {
            let dad = Rect::new(rect.x + padding, y, geg, geg);
            
            if item.active {
                
                self.fill_rounded_rect(dad, 8.0, t.accent.with_alpha(0.3));
            } else if item.hovered {
                self.fill_rounded_rect(dad, 8.0, t.panel_hover);
            }
            
            
            let cx = dad.x + dad.width / 2.0;
            let u = dad.y + dad.height / 2.0;
            self.fill_circle(Point::new(cx, u), 16.0, t.text_secondary);
            
            
            if item.running {
                self.fill_circle(
                    Point::new(rect.x + rect.width - 4.0, u),
                    3.0,
                    t.accent,
                );
            }
            
            y += geg + padding;
        }
    }
}





#[derive(Clone, Copy, PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Suggested,
    Destructive,
}

pub struct Ln {
    pub name: &'static str,
    pub active: bool,
    pub hovered: bool,
    pub running: bool,
}





fn fdc(c: Color) -> SkiaColor {
    SkiaColor::from_rgba(c.r, c.g, c.b, c.a).unwrap_or(SkiaColor::BLACK)
}

fn qth(r: Rect) -> Option<Path> {
    let mut ji = PathBuilder::new();
    ji.move_to(r.x, r.y);
    ji.line_to(r.x + r.width, r.y);
    ji.line_to(r.x + r.width, r.y + r.height);
    ji.line_to(r.x, r.y + r.height);
    ji.close();
    ji.finish()
}

fn oie(r: Rect, radius: f32) -> Option<Path> {
    let mut ji = PathBuilder::new();
    let abf = radius.min(r.width / 2.0).min(r.height / 2.0);
    
    
    ji.move_to(r.x + abf, r.y);
    
    
    ji.line_to(r.x + r.width - abf, r.y);
    
    ji.quad_to(r.x + r.width, r.y, r.x + r.width, r.y + abf);
    
    
    ji.line_to(r.x + r.width, r.y + r.height - abf);
    
    ji.quad_to(r.x + r.width, r.y + r.height, r.x + r.width - abf, r.y + r.height);
    
    
    ji.line_to(r.x + abf, r.y + r.height);
    
    ji.quad_to(r.x, r.y + r.height, r.x, r.y + r.height - abf);
    
    
    ji.line_to(r.x, r.y + abf);
    
    ji.quad_to(r.x, r.y, r.x + abf, r.y);
    
    ji.close();
    ji.finish()
}

fn kkn(center: Point, radius: f32) -> Option<Path> {
    let mut ji = PathBuilder::new();
    
    
    let k = 0.5522847498; 
    let bhm = k * radius;
    
    ji.move_to(center.x + radius, center.y);
    ji.cubic_to(
        center.x + radius, center.y + bhm,
        center.x + bhm, center.y + radius,
        center.x, center.y + radius,
    );
    ji.cubic_to(
        center.x - bhm, center.y + radius,
        center.x - radius, center.y + bhm,
        center.x - radius, center.y,
    );
    ji.cubic_to(
        center.x - radius, center.y - bhm,
        center.x - bhm, center.y - radius,
        center.x, center.y - radius,
    );
    ji.cubic_to(
        center.x + bhm, center.y - radius,
        center.x + radius, center.y - bhm,
        center.x + radius, center.y,
    );
    ji.close();
    
    ji.finish()
}





impl CosmicRenderer {
    
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, color: Color) {
        let mut cx = x as i32;
        let u = y as i32;
        
        for c in text.chars() {
            if c == ' ' {
                cx += 8;
                continue;
            }
            self.draw_char(cx, u, c, color);
            cx += 8;
        }
    }
    
    
    pub fn draw_text_centered(&mut self, text: &str, rect: Rect, color: Color) {
        let ebn = (text.len() * 8) as f32;
        let pih = 16.0f32;
        let x = rect.x + (rect.width - ebn) / 2.0;
        let y = rect.y + (rect.height - pih) / 2.0;
        self.draw_text(text, x, y, color);
    }
    
    
    fn draw_char(&mut self, x: i32, y: i32, c: char, color: Color) {
        let du = crate::framebuffer::font::ol(c);
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        
        let data = self.pixmap.data_mut();
        let stride = self.width as usize * 4;
        
        for (row, &glyph_byte) in du.iter().enumerate() {
            let o = y + row as i32;
            if o < 0 || o >= self.height as i32 {
                continue;
            }
            
            for bf in 0..8 {
                let p = x + bf;
                if p < 0 || p >= self.width as i32 {
                    continue;
                }
                
                if (glyph_byte >> (7 - bf)) & 1 != 0 {
                    let idx = o as usize * stride + p as usize * 4;
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
    
    
    pub fn qea(&mut self, text: &str, x: f32, y: f32, color: Color, shadow: Color) {
        self.draw_text(text, x + 1.0, y + 1.0, shadow);
        self.draw_text(text, x, y, color);
    }
    
    
    pub fn fwt(&mut self, gw: Point, gn: Point, aih: Point, color: Color) {
        let mut ji = PathBuilder::new();
        ji.move_to(gw.x, gw.y);
        ji.line_to(gn.x, gn.y);
        ji.line_to(aih.x, aih.y);
        ji.close();
        
        if let Some(path) = ji.finish() {
            let mut ars = Paint::default();
            ars.set_color(fdc(color));
            ars.anti_alias = true;
            
            self.pixmap.fill_path(
                &path,
                &ars,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }
    
    
    pub fn bly(&mut self, rect: Rect, progress: f32, bg: Color, fg: Color, border: Color) {
        
        self.fill_rounded_rect(rect, 4.0, bg);
        
        
        let bzr = rect.width * progress.clamp(0.0, 1.0);
        if bzr > 0.0 {
            let fill_rect = Rect::new(rect.x, rect.y, bzr, rect.height);
            self.fill_rounded_rect(fill_rect, 4.0, fg);
        }
        
        
        self.stroke_rounded_rect(rect, 4.0, border, 1.0);
    }
}
