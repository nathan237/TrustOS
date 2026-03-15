







use alloc::vec::Vec;


fn eql(b: f32) -> f32 {
    let mrp = b as i32;
    if b < mrp as f32 { (mrp - 1) as f32 } else { mrp as f32 }
}

fn pec(b: f32) -> f32 {
    eql(b + 0.5)
}

fn ckn(b: f32) -> f32 {
    b - eql(b)
}

fn iip(b: f32) -> f32 {
    if b < 0.0 { -b } else { b }
}

fn khs(b: f32, v: f32, am: f32) -> f32 {
    if b < v { v } else if b > am { am } else { b }
}

fn jre(b: f32) -> f32 { crate::math::ahn(b) }

fn pkz(b: f32) -> f32 { crate::math::lz(b) }

fn ngf(b: f32) -> f32 { crate::math::rk(b) }

fn xaq(b: f32) -> f32 { crate::math::nsw(b) }


#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub m: u8,
    pub at: u8,
    pub o: u8,
    pub q: u8,
}

impl Color {
    pub const fn new(m: u8, at: u8, o: u8, q: u8) -> Self {
        Self { m, at, o, q }
    }
    
    pub const fn zi(r: u32) -> Self {
        Self {
            q: ((r >> 24) & 0xFF) as u8,
            m: ((r >> 16) & 0xFF) as u8,
            at: ((r >> 8) & 0xFF) as u8,
            o: (r & 0xFF) as u8,
        }
    }
    
    pub const fn lv(self) -> u32 {
        ((self.q as u32) << 24) | ((self.m as u32) << 16) | ((self.at as u32) << 8) | (self.o as u32)
    }
    
    pub const fn xt(m: u8, at: u8, o: u8) -> Self {
        Self { m, at, o, q: 255 }
    }
    
    pub const fn dbi(m: u8, at: u8, o: u8, q: u8) -> Self {
        Self { m, at, o, q }
    }
    
    
    pub const Anl: Self = Self::new(0, 0, 0, 0);
    pub const Ox: Self = Self::new(0, 0, 0, 255);
    pub const Zm: Self = Self::new(255, 255, 255, 255);
    pub const Bqa: Self = Self::new(255, 0, 0, 255);
    pub const Bht: Self = Self::new(0, 255, 0, 255);
    pub const Bci: Self = Self::new(0, 0, 255, 255);
}


#[derive(Clone, Copy, Debug)]
pub struct DirtyRect {
    pub b: u32,
    pub c: u32,
    pub d: u32,
    pub i: u32,
}

impl DirtyRect {
    pub fn new(b: u32, c: u32, d: u32, i: u32) -> Self {
        Self { b, c, d, i }
    }
    
    
    pub fn far(&self, gq: &DirtyRect) -> DirtyRect {
        let dn = self.b.v(gq.b);
        let dp = self.c.v(gq.c);
        let hy = (self.b + self.d).am(gq.b + gq.d);
        let jz = (self.c + self.i).am(gq.c + gq.i);
        DirtyRect::new(dn, dp, hy - dn, jz - dp)
    }
    
    
    pub fn jao(&self, gq: &DirtyRect) -> bool {
        !(self.b + self.d <= gq.b || gq.b + gq.d <= self.b ||
          self.c + self.i <= gq.c || gq.c + gq.i <= self.c)
    }
}


pub struct Rasterizer {
    pub z: u32,
    pub ac: u32,
    pub hkn: Vec<u32>,
    pub aqt: Vec<u32>,
    pub dpz: Vec<DirtyRect>,
    pub asw: bool,
}

impl Rasterizer {
    
    pub fn new(z: u32, ac: u32) -> Self {
        let aw = (z * ac) as usize;
        Self {
            z,
            ac,
            hkn: alloc::vec![0xFF000000; aw],
            aqt: alloc::vec![0xFF000000; aw],
            dpz: Vec::new(),
            asw: true,
        }
    }
    
    
    pub fn jey(&mut self, b: u32, c: u32, d: u32, i: u32) {
        let ha = DirtyRect::new(
            b.v(self.z),
            c.v(self.ac),
            d.v(self.z - b.v(self.z)),
            i.v(self.ac - c.v(self.ac)),
        );
        
        
        let mut hrj = false;
        for xy in self.dpz.el() {
            if xy.jao(&ha) {
                *xy = xy.far(&ha);
                hrj = true;
                break;
            }
        }
        
        if !hrj {
            self.dpz.push(ha);
        }
    }
    
    
    pub fn clear(&mut self, s: u32) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use core::arch::x86_64::*;
            let vi = els(s as i32);
            let ptr = self.aqt.mw() as *mut acb;
            let az = self.aqt.len() / 4;
            for a in 0..az {
                ccs(ptr.add(a), vi);
            }
            for a in (az * 4)..self.aqt.len() {
                self.aqt[a] = s;
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            for il in self.aqt.el() {
                *il = s;
            }
        }
        self.asw = true;
    }
    
    
    #[inline(always)]
    fn vih(&self, b: u32, c: u32) -> Option<usize> {
        if b < self.z && c < self.ac {
            Some((c * self.z + b) as usize)
        } else {
            None
        }
    }
    
    
    #[inline(always)]
    pub fn aht(&mut self, b: u32, c: u32, s: u32) {
        if let Some(w) = self.vih(b, c) {
            let dw = (s >> 24) & 0xFF;
            
            if dw == 255 {
                
                self.aqt[w] = s;
            } else if dw > 0 {
                
                self.aqt[w] = Self::ilr(self.aqt[w], s);
            }
            
        }
    }
    
    
    #[inline(always)]
    pub fn ilr(cs: u32, cy: u32) -> u32 {
        let bcm = ((cy >> 24) & 0xFF) as u32;
        if bcm == 0 { return cs; }
        if bcm == 255 { return cy; }
        
        let adz = ((cy >> 16) & 0xFF) as u32;
        let bsi = ((cy >> 8) & 0xFF) as u32;
        let is = (cy & 0xFF) as u32;
        
        let rte = ((cs >> 24) & 0xFF) as u32;
        let ahh = ((cs >> 16) & 0xFF) as u32;
        let bgs = ((cs >> 8) & 0xFF) as u32;
        let ng = (cs & 0xFF) as u32;
        
        
        let jaq = 255 - bcm;
        let efx = (adz * bcm + ahh * jaq) / 255;
        let uxl = (bsi * bcm + bgs * jaq) / 255;
        let uwu = (is * bcm + ng * jaq) / 255;
        let uws = bcm + (rte * jaq) / 255;
        
        (uws << 24) | (efx << 16) | (uxl << 8) | uwu
    }
    
    
    pub fn ah(&mut self, b: i32, c: i32, d: u32, i: u32, s: u32) {
        let fy = b.am(0) as u32;
        let fo = c.am(0) as u32;
        let dn = ((b + d as i32) as u32).v(self.z);
        let dp = ((c + i as i32) as u32).v(self.ac);
        
        let dw = (s >> 24) & 0xFF;
        let cml = (dn - fy) as usize;
        
        if dw == 255 && cml >= 4 {
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                use core::arch::x86_64::*;
                let vi = els(s as i32);
                for x in fo..dp {
                    let mu = (x * self.z + fy) as usize;
                    let ptr = self.aqt.mw().add(mu) as *mut acb;
                    let btq = cml / 4;
                    for a in 0..btq {
                        ccs(ptr.add(a), vi);
                    }
                    for a in (btq * 4)..cml {
                        self.aqt[mu + a] = s;
                    }
                }
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                for x in fo..dp {
                    let mu = (x * self.z + fy) as usize;
                    for a in 0..cml {
                        self.aqt[mu + a] = s;
                    }
                }
            }
        } else {
            for x in fo..dp {
                let mu = (x * self.z) as usize;
                for y in fy..dn {
                    let w = mu + y as usize;
                    if dw == 255 {
                        self.aqt[w] = s;
                    } else if dw > 0 {
                        self.aqt[w] = Self::ilr(self.aqt[w], s);
                    }
                }
            }
        }
        
        self.jey(fy, fo, dn - fy, dp - fo);
    }
    
    
    pub fn lx(&mut self, b: i32, c: i32, d: u32, i: u32, s: u32) {
        
        self.ah(b, c, d, 1, s);
        self.ah(b, c + i as i32 - 1, d, 1, s);
        
        self.ah(b, c, 1, i, s);
        self.ah(b + d as i32 - 1, c, 1, i, s);
    }
    
    
    pub fn krd(&mut self, fy: f32, fo: f32, dn: f32, dp: f32, s: u32) {
        let jrs = iip(dp - fo) > iip(dn - fy);
        
        let (fy, fo, dn, dp) = if jrs {
            (fo, fy, dp, dn)
        } else {
            (fy, fo, dn, dp)
        };
        
        let (fy, fo, dn, dp) = if fy > dn {
            (dn, dp, fy, fo)
        } else {
            (fy, fo, dn, dp)
        };
        
        let dx = dn - fy;
        let bg = dp - fo;
        let drj = if dx == 0.0 { 1.0 } else { bg / dx };
        
        
        let ihs = pec(fy);
        let ddm = fo + drj * (ihs - fy);
        let elh = 1.0 - ckn(fy + 0.5);
        let ihu = ihs as i32;
        let jxq = eql(ddm) as i32;
        
        if jrs {
            self.cgo(jxq, ihu, s, (1.0 - ckn(ddm)) * elh);
            self.cgo(jxq + 1, ihu, s, ckn(ddm) * elh);
        } else {
            self.cgo(ihu, jxq, s, (1.0 - ckn(ddm)) * elh);
            self.cgo(ihu, jxq + 1, s, ckn(ddm) * elh);
        }
        
        let mut edn = ddm + drj;
        
        
        let ihs = pec(dn);
        let ddm = dp + drj * (ihs - dn);
        let elh = ckn(dn + 0.5);
        let ihv = ihs as i32;
        let jxr = eql(ddm) as i32;
        
        if jrs {
            self.cgo(jxr, ihv, s, (1.0 - ckn(ddm)) * elh);
            self.cgo(jxr + 1, ihv, s, ckn(ddm) * elh);
        } else {
            self.cgo(ihv, jxr, s, (1.0 - ckn(ddm)) * elh);
            self.cgo(ihv, jxr + 1, s, ckn(ddm) * elh);
        }
        
        
        for b in (ihu + 1)..ihv {
            if jrs {
                self.cgo(eql(edn) as i32, b, s, 1.0 - ckn(edn));
                self.cgo(eql(edn) as i32 + 1, b, s, ckn(edn));
            } else {
                self.cgo(b, eql(edn) as i32, s, 1.0 - ckn(edn));
                self.cgo(b, eql(edn) as i32 + 1, s, ckn(edn));
            }
            edn += drj;
        }
    }
    
    
    #[inline(always)]
    fn cgo(&mut self, b: i32, c: i32, s: u32, hj: f32) {
        if b >= 0 && c >= 0 && (b as u32) < self.z && (c as u32) < self.ac {
            let kce = ((s >> 24) & 0xFF) as f32;
            let usl = (kce * khs(hj, 0.0, 1.0)) as u32;
            let qpx = (usl << 24) | (s & 0x00FFFFFF);
            self.aht(b as u32, c as u32, qpx);
        }
    }
    
    
    pub fn ymz(&mut self, cx: i32, ae: i32, dy: u32, s: u32) {
        let m = dy as f32;
        let yca = m * m;
        
        for c in -(dy as i32)..=(dy as i32) {
            for b in -(dy as i32)..=(dy as i32) {
                let dgk = (b * b + c * c) as f32;
                let la = jre(dgk);
                
                
                let dqm = iip(la - m);
                
                if dqm < 1.5 {
                    
                    let p = iip(la - m + 0.5);
                    let hj = 1.0 - if p < 1.0 { p } else { 1.0 };
                    if hj > 0.0 {
                        self.cgo(cx + b, ae + c, s, hj);
                    }
                }
            }
        }
    }
    
    
    pub fn hji(&mut self, cx: i32, ae: i32, dy: u32, s: u32) {
        let m = dy as f32;
        
        for c in -(dy as i32 + 1)..=(dy as i32 + 1) {
            for b in -(dy as i32 + 1)..=(dy as i32 + 1) {
                let la = jre((b * b + c * c) as f32);
                
                if la <= m - 0.5 {
                    
                    self.aht((cx + b) as u32, (ae + c) as u32, s);
                } else if la < m + 0.5 {
                    
                    let hj = 1.0 - khs(la - m + 0.5, 0.0, 1.0);
                    self.cgo(cx + b, ae + c, s, hj);
                }
            }
        }
        
        self.jey(
            (cx - dy as i32 - 1).am(0) as u32,
            (ae - dy as i32 - 1).am(0) as u32,
            dy * 2 + 3,
            dy * 2 + 3,
        );
    }
    
    
    pub fn nts(&mut self, b: i32, c: i32, d: u32, i: u32, bjo: u32, btr: u32) {
        let fy = b.am(0) as u32;
        let fo = c.am(0) as u32;
        let dn = ((b + d as i32) as u32).v(self.z);
        let dp = ((c + i as i32) as u32).v(self.ac);
        
        let rw = Color::zi(bjo);
        let tx = Color::zi(btr);
        
        for x in fo..dp {
            let mu = (x * self.z) as usize;
            for y in fy..dn {
                let ab = (y - fy) as f32 / d as f32;
                let m = (rw.m as f32 * (1.0 - ab) + tx.m as f32 * ab) as u8;
                let at = (rw.at as f32 * (1.0 - ab) + tx.at as f32 * ab) as u8;
                let o = (rw.o as f32 * (1.0 - ab) + tx.o as f32 * ab) as u8;
                let q = (rw.q as f32 * (1.0 - ab) + tx.q as f32 * ab) as u8;
                
                let s = ((q as u32) << 24) | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32);
                let w = mu + y as usize;
                
                if q == 255 {
                    self.aqt[w] = s;
                } else if q > 0 {
                    self.aqt[w] = Self::ilr(self.aqt[w], s);
                }
            }
        }
        
        self.jey(fy, fo, dn - fy, dp - fo);
    }
    
    
    pub fn kvv(&mut self, b: i32, c: i32, d: u32, i: u32, bjo: u32, btr: u32) {
        let fy = b.am(0) as u32;
        let fo = c.am(0) as u32;
        let dn = ((b + d as i32) as u32).v(self.z);
        let dp = ((c + i as i32) as u32).v(self.ac);
        
        let rw = Color::zi(bjo);
        let tx = Color::zi(btr);
        
        for x in fo..dp {
            let ab = (x - fo) as f32 / i as f32;
            let m = (rw.m as f32 * (1.0 - ab) + tx.m as f32 * ab) as u8;
            let at = (rw.at as f32 * (1.0 - ab) + tx.at as f32 * ab) as u8;
            let o = (rw.o as f32 * (1.0 - ab) + tx.o as f32 * ab) as u8;
            let q = (rw.q as f32 * (1.0 - ab) + tx.q as f32 * ab) as u8;
            
            let s = ((q as u32) << 24) | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32);
            let mu = (x * self.z) as usize;
            
            for y in fy..dn {
                let w = mu + y as usize;
                if q == 255 {
                    self.aqt[w] = s;
                } else if q > 0 {
                    self.aqt[w] = Self::ilr(self.aqt[w], s);
                }
            }
        }
        
        self.jey(fy, fo, dn - fy, dp - fo);
    }
    
    
    pub fn afp(&mut self, b: i32, c: i32, d: u32, i: u32, dy: u32, s: u32) {
        let m = dy.v(d / 2).v(i / 2);
        
        
        self.ah(b + m as i32, c, d - m * 2, i, s);
        self.ah(b, c + m as i32, m, i - m * 2, s);
        self.ah(b + d as i32 - m as i32, c + m as i32, m, i - m * 2, s);
        
        
        
        self.iuo(b + m as i32, c + m as i32, m, s, 2);
        
        self.iuo(b + d as i32 - m as i32 - 1, c + m as i32, m, s, 1);
        
        self.iuo(b + m as i32, c + i as i32 - m as i32 - 1, m, s, 3);
        
        self.iuo(b + d as i32 - m as i32 - 1, c + i as i32 - m as i32 - 1, m, s, 0);
    }
    
    
    fn iuo(&mut self, cx: i32, ae: i32, dy: u32, s: u32, vov: u8) {
        let m = dy as f32;
        
        let (xwi, fze): (core::ops::Cke<i32>, core::ops::Cke<i32>) = match vov {
            0 => (0..=(dy as i32), 0..=(dy as i32)),       
            1 => (0..=(dy as i32), -(dy as i32)..=0),      
            2 => (-(dy as i32)..=0, -(dy as i32)..=0),     
            3 => (-(dy as i32)..=0, 0..=(dy as i32)),      
            _ => return,
        };
        
        for bg in fze {
            for dx in xwi.clone() {
                let la = jre((dx * dx + bg * bg) as f32);
                
                if la <= m - 0.5 {
                    self.aht((cx + dx) as u32, (ae + bg) as u32, s);
                } else if la < m + 0.5 {
                    let hj = 1.0 - khs(la - m + 0.5, 0.0, 1.0);
                    self.cgo(cx + dx, ae + bg, s, hj);
                }
            }
        }
    }
    
    
    pub fn gfj(&mut self, b: i32, c: i32, d: u32, i: u32, cou: u32, s: u32) {
        let kce = ((s >> 24) & 0xFF) as f32;
        let dls = s & 0x00FFFFFF;
        
        for o in 0..cou {
            let ab = (cou - o) as f32 / cou as f32;
            let dw = (kce * ab * 0.5) as u32;
            let r = (dw << 24) | dls;
            
            self.ah(
                b - o as i32,
                c - o as i32,
                d + o * 2,
                i + o * 2,
                r,
            );
        }
    }
    
    
    pub fn sv(&mut self) {
        if self.asw {
            
            self.hkn.dg(&self.aqt);
            self.asw = false;
        } else {
            
            for ha in &self.dpz {
                for c in ha.c..(ha.c + ha.i).v(self.ac) {
                    let ay = (c * self.z + ha.b) as usize;
                    let ci = (c * self.z + (ha.b + ha.d).v(self.z)) as usize;
                    self.hkn[ay..ci].dg(&self.aqt[ay..ci]);
                }
            }
        }
        
        self.dpz.clear();
    }
    
    
    pub fn ygp(&self) {
        for c in 0..self.ac {
            for b in 0..self.z {
                let w = (c * self.z + b) as usize;
                crate::framebuffer::draw_pixel(b, c, self.hkn[w]);
            }
        }
    }
    
    
    pub fn ygm(&self) {
        for ha in &self.dpz {
            for c in ha.c..(ha.c + ha.i).v(self.ac) {
                for b in ha.b..(ha.b + ha.d).v(self.z) {
                    let w = (c * self.z + b) as usize;
                    crate::framebuffer::draw_pixel(b, c, self.hkn[w]);
                }
            }
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub b: f32,
    pub c: f32,
    pub av: f32,
}

impl Vec3 {
    pub const fn new(b: f32, c: f32, av: f32) -> Self {
        Self { b, c, av }
    }
    
    pub fn amb(&self, gq: &Vec3) -> f32 {
        self.b * gq.b + self.c * gq.c + self.av * gq.av
    }
    
    pub fn bjr(&self, gq: &Vec3) -> Vec3 {
        Vec3 {
            b: self.c * gq.av - self.av * gq.c,
            c: self.av * gq.b - self.b * gq.av,
            av: self.b * gq.c - self.c * gq.b,
        }
    }
    
    pub fn go(&self) -> f32 {
        jre(self.b * self.b + self.c * self.c + self.av * self.av)
    }
    
    pub fn all(&self) -> Vec3 {
        let len = self.go();
        if len > 0.0 {
            Vec3 {
                b: self.b / len,
                c: self.c / len,
                av: self.av / len,
            }
        } else {
            *self
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Mat4 {
    pub ef: [[f32; 4]; 4],
}

impl Mat4 {
    pub const fn fky() -> Self {
        Self {
            ef: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    
    
    pub fn chi(hg: f32) -> Self {
        let r = ngf(hg);
        let e = pkz(hg);
        Self {
            ef: [
                [r, 0.0, e, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-e, 0.0, r, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    
    
    pub fn dlk(hg: f32) -> Self {
        let r = ngf(hg);
        let e = pkz(hg);
        Self {
            ef: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, r, -e, 0.0],
                [0.0, e, r, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    
    
    pub fn aqf(ckm: f32, dyk: f32, bhl: f32, adt: f32) -> Self {
        let bb = 1.0 / xaq(ckm / 2.0);
        Self {
            ef: [
                [bb / dyk, 0.0, 0.0, 0.0],
                [0.0, bb, 0.0, 0.0],
                [0.0, 0.0, (adt + bhl) / (bhl - adt), -1.0],
                [0.0, 0.0, (2.0 * adt * bhl) / (bhl - adt), 0.0],
            ],
        }
    }
    
    
    pub fn pvx(&self, ai: Vec3) -> Vec3 {
        let d = self.ef[0][3] * ai.b + self.ef[1][3] * ai.c + self.ef[2][3] * ai.av + self.ef[3][3];
        Vec3 {
            b: (self.ef[0][0] * ai.b + self.ef[1][0] * ai.c + self.ef[2][0] * ai.av + self.ef[3][0]) / d,
            c: (self.ef[0][1] * ai.b + self.ef[1][1] * ai.c + self.ef[2][1] * ai.av + self.ef[3][1]) / d,
            av: (self.ef[0][2] * ai.b + self.ef[1][2] * ai.c + self.ef[2][2] * ai.av + self.ef[3][2]) / d,
        }
    }
    
    
    pub fn mul(&self, gq: &Mat4) -> Mat4 {
        let mut result = Mat4::fky();
        for a in 0..4 {
            for fb in 0..4 {
                result.ef[a][fb] = 0.0;
                for eh in 0..4 {
                    result.ef[a][fb] += self.ef[a][eh] * gq.ef[eh][fb];
                }
            }
        }
        result
    }
}


pub struct Renderer3D {
    pub z: u32,
    pub ac: u32,
    pub cob: Vec<f32>,
}

impl Renderer3D {
    pub fn new(z: u32, ac: u32) -> Self {
        Self {
            z,
            ac,
            cob: alloc::vec![f32::O; (z * ac) as usize],
        }
    }
    
    pub fn rbj(&mut self) {
        for av in self.cob.el() {
            *av = f32::O;
        }
    }
    
    
    pub fn nv(&self, ai: Vec3, imt: f32) -> Option<(i32, i32, f32)> {
        let av = ai.av + imt;
        if av <= 0.1 { return None; }
        
        let bv = 200.0 / av;
        let cr = (self.z as f32 / 2.0 + ai.b * bv) as i32;
        let cq = (self.ac as f32 / 2.0 - ai.c * bv) as i32;
        
        Some((cr, cq, av))
    }
    
    
    pub fn bzg(&mut self, awq: &mut Rasterizer, pr: Vec3, pf: Vec3, imt: f32, s: u32) {
        if let (Some((dn, dp, _)), Some((hy, jz, _))) = (self.nv(pr, imt), self.nv(pf, imt)) {
            awq.krd(dn as f32, dp as f32, hy as f32, jz as f32, s);
        }
    }
    
    
    pub fn gfg(&mut self, awq: &mut Rasterizer, pn: Vec3, aw: f32, chh: &Mat4, s: u32) {
        let e = aw / 2.0;
        
        
        let lm = [
            Vec3::new(-e, -e, -e),
            Vec3::new(e, -e, -e),
            Vec3::new(e, e, -e),
            Vec3::new(-e, e, -e),
            Vec3::new(-e, -e, e),
            Vec3::new(e, -e, e),
            Vec3::new(e, e, e),
            Vec3::new(-e, e, e),
        ];
        
        
        let dxc: Vec<Vec3> = lm.iter()
            .map(|p| {
                let cmk = chh.pvx(*p);
                Vec3::new(
                    cmk.b + pn.b,
                    cmk.c + pn.c,
                    cmk.av + pn.av,
                )
            })
            .collect();
        
        
        let bu = [
            (0, 1), (1, 2), (2, 3), (3, 0),  
            (4, 5), (5, 6), (6, 7), (7, 4),  
            (0, 4), (1, 5), (2, 6), (3, 7),  
        ];
        
        for (hnh, hni) in bu {
            self.bzg(awq, dxc[hnh], dxc[hni], 5.0, s);
        }
    }
}
