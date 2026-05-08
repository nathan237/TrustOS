








use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;






const EDN_: usize = 8;


const HH_: usize = 64;


const UL_: usize = 128;






pub struct FastSurface {
    
    pub data: Box<[u32]>,
    
    pub width: u32,
    
    pub height: u32,
    
    dirty: DirtyRegion,
}

impl FastSurface {
    
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            data: alloc::vec![0u32; size].into_boxed_slice(),
            width,
            height,
            dirty: DirtyRegion::new(width, height),
        }
    }

    
    #[inline]
    pub fn clear(&mut self, color: u32) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            crate::graphics::simd::adq(
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

    
    #[inline]
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        let x1 = x.max(0) as u32;
        let y1 = y.max(0) as u32;
        let x2 = ((x + w as i32) as u32).min(self.width);
        let y2 = ((y + h as i32) as u32).min(self.height);
        
        if x2 <= x1 || y2 <= y1 { return; }
        
        let auv = (x2 - x1) as usize;
        let stride = self.width as usize;
        
        for o in y1..y2 {
            let fk = o as usize * stride + x1 as usize;
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::adq(
                    self.data.as_mut_ptr().add(fk),
                    auv,
                    color
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                self.data[fk..fk + auv].fill(color);
            }
        }
        
        self.dirty.add_rect(x1, y1, x2 - x1, y2 - y1);
    }

    
    #[inline(always)]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = color;
        }
    }

    
    #[inline(always)]
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize]
        } else {
            0
        }
    }

    
    #[inline]
    pub fn hline(&mut self, x: i32, y: i32, len: u32, color: u32) {
        if y < 0 || y >= self.height as i32 { return; }
        
        let x1 = x.max(0) as u32;
        let x2 = ((x + len as i32) as u32).min(self.width);
        if x2 <= x1 { return; }
        
        let y = y as u32;
        let start = (y * self.width + x1) as usize;
        let dpn = (x2 - x1) as usize;
        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            crate::graphics::simd::adq(
                self.data.as_mut_ptr().add(start),
                dpn,
                color,
            );
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.data[start..start + dpn].fill(color);
        }
        
        self.dirty.add_rect(x1, y, x2 - x1, 1);
    }

    
    #[inline]
    pub fn vline(&mut self, x: i32, y: i32, len: u32, color: u32) {
        if x < 0 || x >= self.width as i32 { return; }
        
        let y1 = y.max(0) as u32;
        let y2 = ((y + len as i32) as u32).min(self.height);
        if y2 <= y1 { return; }
        
        let x = x as u32;
        let stride = self.width;
        for o in y1..y2 {
            self.data[(o * stride + x) as usize] = color;
        }
        
        self.dirty.add_rect(x, y1, 1, y2 - y1);
    }

    
    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        self.hline(x, y, w, color);
        self.hline(x, y + h as i32 - 1, w, color);
        self.vline(x, y, h, color);
        self.vline(x + w as i32 - 1, y, h, color);
    }

    
    pub fn blit(&mut self, src: &FastSurface, dst_x: i32, dst_y: i32) {
        self.blit_region(src, 0, 0, src.width, src.height, dst_x, dst_y);
    }

    
    pub fn blit_region(&mut self, src: &FastSurface, 
                       ahc: u32, aft: u32, fbk: u32, gvv: u32,
                       dst_x: i32, dst_y: i32) {
        
        let wn = ahc.min(src.width);
        let aiu = aft.min(src.height);
        let tq = (ahc + fbk).min(src.width);
        let acv = (aft + gvv).min(src.height);
        
        if tq <= wn || acv <= aiu { return; }
        
        
        let mut dx = dst_x;
        let mut ad = dst_y;
        let mut ut = (tq - wn) as i32;
        let mut abw = (acv - aiu) as i32;
        let mut jhr = 0i32;
        let mut jhs = 0i32;
        
        
        if dx < 0 {
            jhr = -dx;
            ut += dx;
            dx = 0;
        }
        
        if ad < 0 {
            jhs = -ad;
            abw += ad;
            ad = 0;
        }
        
        if dx + ut > self.width as i32 {
            ut = self.width as i32 - dx;
        }
        
        if ad + abw > self.height as i32 {
            abw = self.height as i32 - ad;
        }
        
        if ut <= 0 || abw <= 0 { return; }
        
        let ut = ut as usize;
        let abw = abw as usize;
        let dx = dx as usize;
        let ad = ad as usize;
        let jtq = (wn as i32 + jhr) as usize;
        let jtr = (aiu as i32 + jhs) as usize;
        
        
        let src_stride = src.width as usize;
        let dst_stride = self.width as usize;
        
        for row in 0..abw {
            let gvy = (jtr + row) * src_stride + jtq;
            let ftk = (ad + row) * dst_stride + dx;
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::blg(
                    self.data.as_mut_ptr().add(ftk),
                    src.data.as_ptr().add(gvy),
                    ut
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                self.data[ftk..ftk + ut]
                    .copy_from_slice(&src.data[gvy..gvy + ut]);
            }
        }
        
        self.dirty.add_rect(dx as u32, ad as u32, ut as u32, abw as u32);
    }

    
    pub fn blit_alpha(&mut self, src: &FastSurface, dst_x: i32, dst_y: i32) {
        let dod = dst_x.max(0) as u32;
        let ftp = dst_y.max(0) as u32;
        let fbl = if dst_x < 0 { (-dst_x) as u32 } else { 0 };
        let jhv = if dst_y < 0 { (-dst_y) as u32 } else { 0 };
        
        let ut = (src.width - fbl).min(self.width - dod);
        let abw = (src.height - jhv).min(self.height - ftp);
        
        if ut == 0 || abw == 0 { return; }
        
        let src_stride = src.width as usize;
        let dst_stride = self.width as usize;
        
        for row in 0..abw {
            let amv = (jhv + row) as usize * src_stride;
            let dnw = (ftp + row) as usize * dst_stride;
            
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                let ps = src.data.as_ptr().add(amv + fbl as usize);
                let nt = self.data.as_mut_ptr().add(dnw + dod as usize);
                crate::graphics::simd::egy(nt, ps, ut as usize);
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                
                let mut x = 0u32;
                while x + 8 <= ut {
                    for i in 0..8 {
                        let amu = amv + (fbl + x + i) as usize;
                        let ajm = dnw + (dod + x + i) as usize;
                        let cra = src.data[amu];
                        let alpha = cra >> 24;
                        
                        if alpha == 255 {
                            self.data[ajm] = cra;
                        } else if alpha > 0 {
                            self.data[ajm] = fji(cra, self.data[ajm]);
                        }
                    }
                    x += 8;
                }
                
                
                while x < ut {
                    let amu = amv + (fbl + x) as usize;
                    let ajm = dnw + (dod + x) as usize;
                    let cra = src.data[amu];
                    let alpha = cra >> 24;
                
                    if alpha == 255 {
                        self.data[ajm] = cra;
                    } else if alpha > 0 {
                        self.data[ajm] = fji(cra, self.data[ajm]);
                    }
                    x += 1;
                }
            }
        }
        
        self.dirty.add_rect(dod, ftp, ut, abw);
    }

    
    pub fn fill_rounded_rect(&mut self, x: i32, y: i32, w: u32, h: u32, r: u32, color: u32) {
        let r = r.min(w / 2).min(h / 2);
        
        
        self.fill_rect(x + r as i32, y, w - 2 * r, h, color);
        
        
        self.fill_rect(x, y + r as i32, r, h - 2 * r, color);
        self.fill_rect(x + w as i32 - r as i32, y + r as i32, r, h - 2 * r, color);
        
        
        self.fill_corner(x + r as i32, y + r as i32, r, color, Corner::TopLeft);
        self.fill_corner(x + w as i32 - r as i32 - 1, y + r as i32, r, color, Corner::TopRight);
        self.fill_corner(x + r as i32, y + h as i32 - r as i32 - 1, r, color, Corner::BottomLeft);
        self.fill_corner(x + w as i32 - r as i32 - 1, y + h as i32 - r as i32 - 1, r, color, Corner::BottomRight);
    }

    fn fill_corner(&mut self, cx: i32, u: i32, r: u32, color: u32, corner: Corner) {
        let r = r as i32;
        let amn = r * r;
        
        for ad in 0..=r {
            for dx in 0..=r {
                if dx * dx + ad * ad <= amn {
                    let (p, o) = match corner {
                        Corner::TopLeft => (cx - dx, u - ad),
                        Corner::TopRight => (cx + dx, u - ad),
                        Corner::BottomLeft => (cx - dx, u + ad),
                        Corner::BottomRight => (cx + dx, u + ad),
                    };
                    if p >= 0 && o >= 0 && p < self.width as i32 && o < self.height as i32 {
                        self.set_pixel(p as u32, o as u32, color);
                    }
                }
            }
        }
    }

    
    pub fn mcw(&self) -> &DirtyRegion {
        &self.dirty
    }

    
    pub fn pzx(&mut self) {
        self.dirty.clear();
    }

    
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

    
    pub fn flush_to_fb(&self) {
        let (fb_width, fb_height) = crate::framebuffer::kv();
        let ut = self.width.min(fb_width) as usize;
        let abw = self.height.min(fb_height) as usize;
        
        let bme = crate::framebuffer::eob();
        let cjh = crate::framebuffer::fyo();
        
        if bme.is_null() { return; }
        
        let src_stride = self.width as usize;
        
        for row in 0..abw {
            let zl = row * src_stride;
            let afd = row * cjh;
            
            unsafe {
                let src = self.data.as_ptr().add(zl);
                let dst = bme.add(afd) as *mut u32;
                core::ptr::copy_nonoverlapping(src, dst, ut);
            }
        }
    }

    
    fn flush_rect_to_fb(&self, x: u32, y: u32, w: u32, h: u32) {
        let (fb_width, fb_height) = crate::framebuffer::kv();
        
        let x1 = x.min(fb_width).min(self.width);
        let y1 = y.min(fb_height).min(self.height);
        let x2 = (x + w).min(fb_width).min(self.width);
        let y2 = (y + h).min(fb_height).min(self.height);
        
        if x2 <= x1 || y2 <= y1 { return; }
        
        let bme = crate::framebuffer::eob();
        let cjh = crate::framebuffer::fyo();
        
        if bme.is_null() { return; }
        
        let ut = (x2 - x1) as usize;
        let src_stride = self.width as usize;
        
        for row in y1..y2 {
            let zl = row as usize * src_stride + x1 as usize;
            let afd = row as usize * cjh + x1 as usize * 4;
            
            unsafe {
                let src = self.data.as_ptr().add(zl);
                let dst = bme.add(afd) as *mut u32;
                core::ptr::copy_nonoverlapping(src, dst, ut);
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





#[derive(Clone, Copy, Default)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub struct DirtyRegion {
    pub rects: [Rect; HH_],
    pub count: usize,
    pub full_redraw: bool,
    screen_w: u32,
    screen_h: u32,
}

impl DirtyRegion {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            rects: [Rect::default(); HH_],
            count: 0,
            full_redraw: true, 
            screen_w: width,
            screen_h: height,
        }
    }

    pub fn add_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        if self.full_redraw { return; }
        if w == 0 || h == 0 { return; }
        
        let gjg = Rect { x, y, w, h };
        
        
        for i in 0..self.count {
            if ods(&self.rects[i], &gjg) {
                self.rects[i] = nem(&self.rects[i], &gjg);
                return;
            }
        }
        
        
        if self.count < HH_ {
            self.rects[self.count] = gjg;
            self.count += 1;
        } else {
            
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

fn ods(a: &Rect, b: &Rect) -> bool {
    !(a.x + a.w <= b.x || b.x + b.w <= a.x ||
      a.y + a.h <= b.y || b.y + b.h <= a.y)
}

fn nem(a: &Rect, b: &Rect) -> Rect {
    let x1 = a.x.min(b.x);
    let y1 = a.y.min(b.y);
    let x2 = (a.x + a.w).max(b.x + b.w);
    let y2 = (a.y + a.h).max(b.y + b.h);
    Rect { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }
}






#[inline(always)]
fn fji(src: u32, dst: u32) -> u32 {
    let alpha = (src >> 24) & 0xFF;
    if alpha == 0 { return dst; }
    if alpha == 255 { return src; }
    
    let sg = 255 - alpha;
    
    
    let pb = (src >> 16) & 0xFF;
    let akl = (src >> 8) & 0xFF;
    let cv = src & 0xFF;
    
    let qw = (dst >> 16) & 0xFF;
    let afb = (dst >> 8) & 0xFF;
    let fu = dst & 0xFF;
    
    
    let r = (pb * alpha + qw * sg + 127) / 255;
    let g = (akl * alpha + afb * sg + 127) / 255;
    let b = (cv * alpha + fu * sg + 127) / 255;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}






pub struct GlyphCache {
    
    glyphs: [[u32; 128]; UL_], 
    fg_color: u32,
    initialized: bool,
}

impl GlyphCache {
    pub const fn new() -> Self {
        Self {
            glyphs: [[0u32; 128]; UL_],
            fg_color: 0xFFFFFFFF,
            initialized: false,
        }
    }

    
    pub fn init(&mut self, fg_color: u32) {
        self.fg_color = fg_color;
        
        for c in 0..UL_ {
            let dqw = crate::framebuffer::font::ol(c as u8 as char);
            let mut aza = 0;
            
            for row in 0..16 {
                let bits = dqw[row];
                for col in 0..8 {
                    if (bits >> (7 - col)) & 1 == 1 {
                        self.glyphs[c][aza] = fg_color;
                    } else {
                        self.glyphs[c][aza] = 0; 
                    }
                    aza += 1;
                }
            }
        }
        
        self.initialized = true;
    }

    
    pub fn draw_glyph(&self, surface: &mut FastSurface, c: char, x: i32, y: i32) {
        let idx = (c as usize).min(UL_ - 1);
        let du = &self.glyphs[idx];
        
        let mut aza = 0;
        for row in 0..16 {
            let o = y + row;
            if o >= 0 && o < surface.height as i32 {
                for col in 0..8 {
                    let p = x + col;
                    if p >= 0 && p < surface.width as i32 {
                        let color = du[aza];
                        if color != 0 {
                            surface.set_pixel(p as u32, o as u32, color);
                        }
                    }
                    aza += 1;
                }
            } else {
                aza += 8;
            }
        }
    }

    
    pub fn draw_string(&self, surface: &mut FastSurface, j: &str, x: i32, y: i32) {
        let mut cx = x;
        for c in j.chars() {
            if cx >= surface.width as i32 { break; }
            self.draw_glyph(surface, c, cx, y);
            cx += 8;
        }
    }
}


static AUJ_: Mutex<GlyphCache> = Mutex::new(GlyphCache::new());


pub fn qlb(fg_color: u32) {
    AUJ_.lock().init(fg_color);
}


pub fn draw_text(surface: &mut FastSurface, j: &str, x: i32, y: i32) {
    AUJ_.lock().draw_string(surface, j, x, y);
}






pub struct Layer {
    pub surface: FastSurface,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub visible: bool,
    pub id: u32,
}


pub struct Akc {
    
    pub output: FastSurface,
    
    layers: Vec<Layer>,
    
    next_id: u32,
    
    bg_color: u32,
}

impl Akc {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            output: FastSurface::new(width, height),
            layers: Vec::new(),
            next_id: 1,
            bg_color: 0xFF101010,
        }
    }

    
    pub fn ooo(&mut self, color: u32) {
        self.bg_color = color;
    }

    
    pub fn qbs(&mut self, width: u32, height: u32, x: i32, y: i32, z: i32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let bj = Layer {
            surface: FastSurface::new(width, height),
            x,
            y,
            z,
            visible: true,
            id,
        };
        
        self.layers.push(bj);
        self.sort_layers();
        
        id
    }

    
    pub fn get_layer(&mut self, id: u32) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    
    pub fn oez(&mut self, id: u32) {
        self.layers.retain(|l| l.id != id);
    }

    
    pub fn qpd(&mut self, id: u32, x: i32, y: i32) {
        if let Some(bj) = self.get_layer(id) {
            bj.x = x;
            bj.y = y;
        }
    }

    
    pub fn qrr(&mut self, id: u32) {
        let ndp = self.layers.iter().map(|l| l.z).max().unwrap_or(0);
        if let Some(bj) = self.get_layer(id) {
            bj.z = ndp + 1;
        }
        self.sort_layers();
    }

    fn sort_layers(&mut self) {
        self.layers.sort_by_key(|l| l.z);
    }

    
    pub fn composite(&mut self) {
        
        self.output.clear(self.bg_color);
        
        
        for bj in &self.layers {
            if bj.visible {
                self.output.blit_alpha(&bj.surface, bj.x, bj.y);
            }
        }
    }

    
    pub fn present(&mut self) {
        self.output.flush_dirty_to_fb();
    }

    
    pub fn render(&mut self) {
        self.composite();
        self.present();
    }
}
