







use alloc::vec::Vec;


fn bzt(x: f32) -> f32 {
    let hcx = x as i32;
    if x < hcx as f32 { (hcx - 1) as f32 } else { hcx as f32 }
}

fn jbk(x: f32) -> f32 {
    bzt(x + 0.5)
}

fn atu(x: f32) -> f32 {
    x - bzt(x)
}

fn eev(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

fn fls(x: f32, min: f32, max: f32) -> f32 {
    if x < min { min } else if x > max { max } else { x }
}

fn fbi(x: f32) -> f32 { crate::math::ra(x) }

fn jgj(x: f32) -> f32 { crate::math::eu(x) }

fn hny(x: f32) -> f32 { crate::math::hr(x) }

fn pda(x: f32) -> f32 { crate::math::hxv(x) }


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
    
    pub const fn bdl(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const Acz: Self = Self::new(255, 0, 0, 255);
    pub const Zf: Self = Self::new(0, 255, 0, 255);
    pub const Wn: Self = Self::new(0, 0, 255, 255);
}


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
    
    
    pub fn union(&self, other: &DirtyRect) -> DirtyRect {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.w).max(other.x + other.w);
        let y2 = (self.y + self.h).max(other.y + other.h);
        DirtyRect::new(x1, y1, x2 - x1, y2 - y1)
    }
    
    
    pub fn intersects(&self, other: &DirtyRect) -> bool {
        !(self.x + self.w <= other.x || other.x + other.w <= self.x ||
          self.y + self.h <= other.y || other.y + other.h <= self.y)
    }
}


pub struct Rasterizer {
    pub width: u32,
    pub height: u32,
    pub front_buffer: Vec<u32>,
    pub back_buffer: Vec<u32>,
    pub dirty_rects: Vec<DirtyRect>,
    pub full_redraw: bool,
}

impl Rasterizer {
    
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
    
    
    pub fn mark_dirty(&mut self, x: u32, y: u32, w: u32, h: u32) {
        let rect = DirtyRect::new(
            x.min(self.width),
            y.min(self.height),
            w.min(self.width - x.min(self.width)),
            h.min(self.height - y.min(self.height)),
        );
        
        
        let mut duf = false;
        for ku in self.dirty_rects.iter_mut() {
            if ku.intersects(&rect) {
                *ku = ku.union(&rect);
                duf = true;
                break;
            }
        }
        
        if !duf {
            self.dirty_rects.push(rect);
        }
    }
    
    
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
            for ct in self.back_buffer.iter_mut() {
                *ct = color;
            }
        }
        self.full_redraw = true;
    }
    
    
    #[inline(always)]
    fn pixel_index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }
    
    
    #[inline(always)]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if let Some(idx) = self.pixel_index(x, y) {
            let alpha = (color >> 24) & 0xFF;
            
            if alpha == 255 {
                
                self.back_buffer[idx] = color;
            } else if alpha > 0 {
                
                self.back_buffer[idx] = Self::egx(self.back_buffer[idx], color);
            }
            
        }
    }
    
    
    #[inline(always)]
    pub fn egx(dst: u32, src: u32) -> u32 {
        let acl = ((src >> 24) & 0xFF) as u32;
        if acl == 0 { return dst; }
        if acl == 255 { return src; }
        
        let pb = ((src >> 16) & 0xFF) as u32;
        let akl = ((src >> 8) & 0xFF) as u32;
        let cv = (src & 0xFF) as u32;
        
        let lbi = ((dst >> 24) & 0xFF) as u32;
        let qw = ((dst >> 16) & 0xFF) as u32;
        let afb = ((dst >> 8) & 0xFF) as u32;
        let fu = (dst & 0xFF) as u32;
        
        
        let eqx = 255 - acl;
        let or = (pb * acl + qw * eqx) / 255;
        let nml = (akl * acl + afb * eqx) / 255;
        let nmc = (cv * acl + fu * eqx) / 255;
        let nmb = acl + (lbi * eqx) / 255;
        
        (nmb << 24) | (or << 16) | (nml << 8) | nmc
    }
    
    
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        let bm = x.max(0) as u32;
        let az = y.max(0) as u32;
        let x1 = ((x + w as i32) as u32).min(self.width);
        let y1 = ((y + h as i32) as u32).min(self.height);
        
        let alpha = (color >> 24) & 0xFF;
        let auv = (x1 - bm) as usize;
        
        if alpha == 255 && auv >= 4 {
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                use core::arch::x86_64::*;
                let fill = _mm_set1_epi32(color as i32);
                for o in az..y1 {
                    let fk = (o * self.width + bm) as usize;
                    let ptr = self.back_buffer.as_mut_ptr().add(fk) as *mut __m128i;
                    let chunks = auv / 4;
                    for i in 0..chunks {
                        _mm_storeu_si128(ptr.add(i), fill);
                    }
                    for i in (chunks * 4)..auv {
                        self.back_buffer[fk + i] = color;
                    }
                }
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                for o in az..y1 {
                    let fk = (o * self.width + bm) as usize;
                    for i in 0..auv {
                        self.back_buffer[fk + i] = color;
                    }
                }
            }
        } else {
            for o in az..y1 {
                let fk = (o * self.width) as usize;
                for p in bm..x1 {
                    let idx = fk + p as usize;
                    if alpha == 255 {
                        self.back_buffer[idx] = color;
                    } else if alpha > 0 {
                        self.back_buffer[idx] = Self::egx(self.back_buffer[idx], color);
                    }
                }
            }
        }
        
        self.mark_dirty(bm, az, x1 - bm, y1 - az);
    }
    
    
    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        
        self.fill_rect(x, y, w, 1, color);
        self.fill_rect(x, y + h as i32 - 1, w, 1, color);
        
        self.fill_rect(x, y, 1, h, color);
        self.fill_rect(x + w as i32 - 1, y, 1, h, color);
    }
    
    
    pub fn draw_line_aa(&mut self, bm: f32, az: f32, x1: f32, y1: f32, color: u32) {
        let fbr = eev(y1 - az) > eev(x1 - bm);
        
        let (bm, az, x1, y1) = if fbr {
            (az, bm, y1, x1)
        } else {
            (bm, az, x1, y1)
        };
        
        let (bm, az, x1, y1) = if bm > x1 {
            (x1, y1, bm, az)
        } else {
            (bm, az, x1, y1)
        };
        
        let dx = x1 - bm;
        let ad = y1 - az;
        let bmo = if dx == 0.0 { 1.0 } else { ad / dx };
        
        
        let eel = jbk(bm);
        let bei = az + bmo * (eel - bm);
        let bxe = 1.0 - atu(bm + 0.5);
        let een = eel as i32;
        let ffq = bzt(bei) as i32;
        
        if fbr {
            self.plot_aa(ffq, een, color, (1.0 - atu(bei)) * bxe);
            self.plot_aa(ffq + 1, een, color, atu(bei) * bxe);
        } else {
            self.plot_aa(een, ffq, color, (1.0 - atu(bei)) * bxe);
            self.plot_aa(een, ffq + 1, color, atu(bei) * bxe);
        }
        
        let mut btm = bei + bmo;
        
        
        let eel = jbk(x1);
        let bei = y1 + bmo * (eel - x1);
        let bxe = atu(x1 + 0.5);
        let eeo = eel as i32;
        let ffr = bzt(bei) as i32;
        
        if fbr {
            self.plot_aa(ffr, eeo, color, (1.0 - atu(bei)) * bxe);
            self.plot_aa(ffr + 1, eeo, color, atu(bei) * bxe);
        } else {
            self.plot_aa(eeo, ffr, color, (1.0 - atu(bei)) * bxe);
            self.plot_aa(eeo, ffr + 1, color, atu(bei) * bxe);
        }
        
        
        for x in (een + 1)..eeo {
            if fbr {
                self.plot_aa(bzt(btm) as i32, x, color, 1.0 - atu(btm));
                self.plot_aa(bzt(btm) as i32 + 1, x, color, atu(btm));
            } else {
                self.plot_aa(x, bzt(btm) as i32, color, 1.0 - atu(btm));
                self.plot_aa(x, bzt(btm) as i32 + 1, color, atu(btm));
            }
            btm += bmo;
        }
    }
    
    
    #[inline(always)]
    fn plot_aa(&mut self, x: i32, y: i32, color: u32, intensity: f32) {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            let fih = ((color >> 24) & 0xFF) as f32;
            let niq = (fih * fls(intensity, 0.0, 1.0)) as u32;
            let kcg = (niq << 24) | (color & 0x00FFFFFF);
            self.set_pixel(x as u32, y as u32, kcg);
        }
    }
    
    
    pub fn qdo(&mut self, cx: i32, u: i32, radius: u32, color: u32) {
        let r = radius as f32;
        let pxg = r * r;
        
        for y in -(radius as i32)..=(radius as i32) {
            for x in -(radius as i32)..=(radius as i32) {
                let bgb = (x * x + y * y) as f32;
                let em = fbi(bgb);
                
                
                let bma = eev(em - r);
                
                if bma < 1.5 {
                    
                    let v = eev(em - r + 0.5);
                    let intensity = 1.0 - if v < 1.0 { v } else { 1.0 };
                    if intensity > 0.0 {
                        self.plot_aa(cx + x, u + y, color, intensity);
                    }
                }
            }
        }
    }
    
    
    pub fn fill_circle_aa(&mut self, cx: i32, u: i32, radius: u32, color: u32) {
        let r = radius as f32;
        
        for y in -(radius as i32 + 1)..=(radius as i32 + 1) {
            for x in -(radius as i32 + 1)..=(radius as i32 + 1) {
                let em = fbi((x * x + y * y) as f32);
                
                if em <= r - 0.5 {
                    
                    self.set_pixel((cx + x) as u32, (u + y) as u32, color);
                } else if em < r + 0.5 {
                    
                    let intensity = 1.0 - fls(em - r + 0.5, 0.0, 1.0);
                    self.plot_aa(cx + x, u + y, color, intensity);
                }
            }
        }
        
        self.mark_dirty(
            (cx - radius as i32 - 1).max(0) as u32,
            (u - radius as i32 - 1).max(0) as u32,
            radius * 2 + 3,
            radius * 2 + 3,
        );
    }
    
    
    pub fn fill_gradient_h(&mut self, x: i32, y: i32, w: u32, h: u32, agh: u32, ale: u32) {
        let bm = x.max(0) as u32;
        let az = y.max(0) as u32;
        let x1 = ((x + w as i32) as u32).min(self.width);
        let y1 = ((y + h as i32) as u32).min(self.height);
        
        let hw = Color::from_u32(agh);
        let jf = Color::from_u32(ale);
        
        for o in az..y1 {
            let fk = (o * self.width) as usize;
            for p in bm..x1 {
                let t = (p - bm) as f32 / w as f32;
                let r = (hw.r as f32 * (1.0 - t) + jf.r as f32 * t) as u8;
                let g = (hw.g as f32 * (1.0 - t) + jf.g as f32 * t) as u8;
                let b = (hw.b as f32 * (1.0 - t) + jf.b as f32 * t) as u8;
                let a = (hw.a as f32 * (1.0 - t) + jf.a as f32 * t) as u8;
                
                let color = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                let idx = fk + p as usize;
                
                if a == 255 {
                    self.back_buffer[idx] = color;
                } else if a > 0 {
                    self.back_buffer[idx] = Self::egx(self.back_buffer[idx], color);
                }
            }
        }
        
        self.mark_dirty(bm, az, x1 - bm, y1 - az);
    }
    
    
    pub fn fill_gradient_v(&mut self, x: i32, y: i32, w: u32, h: u32, agh: u32, ale: u32) {
        let bm = x.max(0) as u32;
        let az = y.max(0) as u32;
        let x1 = ((x + w as i32) as u32).min(self.width);
        let y1 = ((y + h as i32) as u32).min(self.height);
        
        let hw = Color::from_u32(agh);
        let jf = Color::from_u32(ale);
        
        for o in az..y1 {
            let t = (o - az) as f32 / h as f32;
            let r = (hw.r as f32 * (1.0 - t) + jf.r as f32 * t) as u8;
            let g = (hw.g as f32 * (1.0 - t) + jf.g as f32 * t) as u8;
            let b = (hw.b as f32 * (1.0 - t) + jf.b as f32 * t) as u8;
            let a = (hw.a as f32 * (1.0 - t) + jf.a as f32 * t) as u8;
            
            let color = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            let fk = (o * self.width) as usize;
            
            for p in bm..x1 {
                let idx = fk + p as usize;
                if a == 255 {
                    self.back_buffer[idx] = color;
                } else if a > 0 {
                    self.back_buffer[idx] = Self::egx(self.back_buffer[idx], color);
                }
            }
        }
        
        self.mark_dirty(bm, az, x1 - bm, y1 - az);
    }
    
    
    pub fn fill_rounded_rect(&mut self, x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
        let r = radius.min(w / 2).min(h / 2);
        
        
        self.fill_rect(x + r as i32, y, w - r * 2, h, color);
        self.fill_rect(x, y + r as i32, r, h - r * 2, color);
        self.fill_rect(x + w as i32 - r as i32, y + r as i32, r, h - r * 2, color);
        
        
        
        self.fill_corner_aa(x + r as i32, y + r as i32, r, color, 2);
        
        self.fill_corner_aa(x + w as i32 - r as i32 - 1, y + r as i32, r, color, 1);
        
        self.fill_corner_aa(x + r as i32, y + h as i32 - r as i32 - 1, r, color, 3);
        
        self.fill_corner_aa(x + w as i32 - r as i32 - 1, y + h as i32 - r as i32 - 1, r, color, 0);
    }
    
    
    fn fill_corner_aa(&mut self, cx: i32, u: i32, radius: u32, color: u32, quadrant: u8) {
        let r = radius as f32;
        
        let (x_range, ctb): (core::ops::RangeInclusive<i32>, core::ops::RangeInclusive<i32>) = match quadrant {
            0 => (0..=(radius as i32), 0..=(radius as i32)),       
            1 => (0..=(radius as i32), -(radius as i32)..=0),      
            2 => (-(radius as i32)..=0, -(radius as i32)..=0),     
            3 => (-(radius as i32)..=0, 0..=(radius as i32)),      
            _ => return,
        };
        
        for ad in ctb {
            for dx in x_range.clone() {
                let em = fbi((dx * dx + ad * ad) as f32);
                
                if em <= r - 0.5 {
                    self.set_pixel((cx + dx) as u32, (u + ad) as u32, color);
                } else if em < r + 0.5 {
                    let intensity = 1.0 - fls(em - r + 0.5, 0.0, 1.0);
                    self.plot_aa(cx + dx, u + ad, color, intensity);
                }
            }
        }
    }
    
    
    pub fn draw_shadow(&mut self, x: i32, y: i32, w: u32, h: u32, awi: u32, color: u32) {
        let fih = ((color >> 24) & 0xFF) as f32;
        let bjd = color & 0x00FFFFFF;
        
        for b in 0..awi {
            let t = (awi - b) as f32 / awi as f32;
            let alpha = (fih * t * 0.5) as u32;
            let c = (alpha << 24) | bjd;
            
            self.fill_rect(
                x - b as i32,
                y - b as i32,
                w + b * 2,
                h + b * 2,
                c,
            );
        }
    }
    
    
    pub fn ii(&mut self) {
        if self.full_redraw {
            
            self.front_buffer.copy_from_slice(&self.back_buffer);
            self.full_redraw = false;
        } else {
            
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
    
    
    pub fn pyx(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                crate::framebuffer::draw_pixel(x, y, self.front_buffer[idx]);
            }
        }
    }
    
    
    pub fn pyu(&self) {
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
        fbi(self.x * self.x + self.y * self.y + self.z * self.z)
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
    
    
    pub fn rotation_y(cc: f32) -> Self {
        let c = hny(cc);
        let j = jgj(cc);
        Self {
            m: [
                [c, 0.0, j, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-j, 0.0, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    
    
    pub fn rotation_x(cc: f32) -> Self {
        let c = hny(cc);
        let j = jgj(cc);
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, c, -j, 0.0],
                [0.0, j, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    
    
    pub fn vq(fov: f32, bqh: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / pda(fov / 2.0);
        Self {
            m: [
                [f / bqh, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (far + near) / (near - far), -1.0],
                [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
            ],
        }
    }
    
    
    pub fn transform_point(&self, aa: Vec3) -> Vec3 {
        let w = self.m[0][3] * aa.x + self.m[1][3] * aa.y + self.m[2][3] * aa.z + self.m[3][3];
        Vec3 {
            x: (self.m[0][0] * aa.x + self.m[1][0] * aa.y + self.m[2][0] * aa.z + self.m[3][0]) / w,
            y: (self.m[0][1] * aa.x + self.m[1][1] * aa.y + self.m[2][1] * aa.z + self.m[3][1]) / w,
            z: (self.m[0][2] * aa.x + self.m[1][2] * aa.y + self.m[2][2] * aa.z + self.m[3][2]) / w,
        }
    }
    
    
    pub fn mul(&self, other: &Mat4) -> Mat4 {
        let mut result = Mat4::identity();
        for i in 0..4 {
            for ay in 0..4 {
                result.m[i][ay] = 0.0;
                for k in 0..4 {
                    result.m[i][ay] += self.m[i][k] * other.m[k][ay];
                }
            }
        }
        result
    }
}


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
    
    
    pub fn project(&self, aa: Vec3, camera_z: f32) -> Option<(i32, i32, f32)> {
        let z = aa.z + camera_z;
        if z <= 0.1 { return None; }
        
        let scale = 200.0 / z;
        let am = (self.width as f32 / 2.0 + aa.x * scale) as i32;
        let ak = (self.height as f32 / 2.0 - aa.y * scale) as i32;
        
        Some((am, ak, z))
    }
    
    
    pub fn draw_line_3d(&mut self, zh: &mut Rasterizer, gw: Vec3, gn: Vec3, camera_z: f32, color: u32) {
        if let (Some((x1, y1, _)), Some((x2, y2, _))) = (self.project(gw, camera_z), self.project(gn, camera_z)) {
            zh.draw_line_aa(x1 as f32, y1 as f32, x2 as f32, y2 as f32, color);
        }
    }
    
    
    pub fn draw_cube(&mut self, zh: &mut Rasterizer, center: Vec3, size: f32, rotation: &Mat4, color: u32) {
        let j = size / 2.0;
        
        
        let vertices = [
            Vec3::new(-j, -j, -j),
            Vec3::new(j, -j, -j),
            Vec3::new(j, j, -j),
            Vec3::new(-j, j, -j),
            Vec3::new(-j, -j, j),
            Vec3::new(j, -j, j),
            Vec3::new(j, j, j),
            Vec3::new(-j, j, j),
        ];
        
        
        let bpr: Vec<Vec3> = vertices.iter()
            .map(|v| {
                let auu = rotation.transform_point(*v);
                Vec3::new(
                    auu.x + center.x,
                    auu.y + center.y,
                    auu.z + center.z,
                )
            })
            .collect();
        
        
        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 0),  
            (4, 5), (5, 6), (6, 7), (7, 4),  
            (0, 4), (1, 5), (2, 6), (3, 7),  
        ];
        
        for (i1, i2) in edges {
            self.draw_line_3d(zh, bpr[i1], bpr[i2], 5.0, color);
        }
    }
}
