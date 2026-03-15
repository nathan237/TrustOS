




use alloc::vec::Vec;
use alloc::boxed::Box;
use embedded_graphics_core::{
    draw_target::Cba,
    geometry::{Cuj, Cim, Size as EgSize},
    pixelcolor::{Rgb888, Dgh},
    Bpb,
};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, Ellipse, Line, Rectangle, RoundedRectangle, Triangle,
    PrimitiveStyle, Dev, Dja,
    CornerRadii,
};
use embedded_graphics::mono_font::{ascii::BVI_, MonoTextStyle};
use embedded_graphics::text::Text;






pub struct Sn {
    pub bi: *mut u32,
    pub z: u32,
    pub ac: u32,
    pub oq: u32, 
}

impl Sn {
    
    
    
    pub unsafe fn new(bi: *mut u32, z: u32, ac: u32, oq: u32) -> Self {
        Self { bi, z, ac, oq }
    }

    
    pub fn yrt(bi: &mut [u32], z: u32, ac: u32) -> Self {
        Self {
            bi: bi.mw(),
            z,
            ac,
            oq: z,
        }
    }

    
    #[inline]
    pub fn aht(&mut self, b: u32, c: u32, s: u32) {
        if b < self.z && c < self.ac {
            unsafe {
                let l = (c * self.oq + b) as isize;
                *self.bi.l(l) = s;
            }
        }
    }

    
    #[inline]
    pub fn beg(&self, b: u32, c: u32) -> u32 {
        if b < self.z && c < self.ac {
            unsafe {
                let l = (c * self.oq + b) as isize;
                *self.bi.l(l)
            }
        } else {
            0
        }
    }

    
    pub fn hcv(&mut self, s: u32) {
        for c in 0..self.ac {
            for b in 0..self.z {
                self.aht(b, c, s);
            }
        }
    }

    
    #[inline]
    pub fn z(&self) -> u32 {
        self.z
    }

    
    #[inline]
    pub fn ac(&self) -> u32 {
        self.ac
    }
}


impl Cim for Sn {
    fn aw(&self) -> EgSize {
        EgSize::new(self.z, self.ac)
    }
}

impl Cba for Sn {
    type Color = Rgb888;
    type Q = core::convert::Czr;

    fn draw_iter<Bix>(&mut self, hz: Bix) -> Result<(), Self::Q>
    where
        Bix: IntoIterator<Item = Bpb<Self::Color>>,
    {
        for Bpb(dff, s) in hz {
            if dff.b >= 0 && dff.c >= 0 
                && (dff.b as u32) < self.z 
                && (dff.c as u32) < self.ac 
            {
                let r = ((s.m() as u32) << 16)
                    | ((s.at() as u32) << 8)
                    | (s.o() as u32)
                    | 0xFF000000;
                self.aht(dff.b as u32, dff.c as u32, r);
            }
        }
        Ok(())
    }
}






#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Color2D {
    pub m: u8,
    pub at: u8,
    pub o: u8,
    pub q: u8,
}

impl Color2D {
    pub const fn new(m: u8, at: u8, o: u8, q: u8) -> Self {
        Self { m, at, o, q }
    }

    pub const fn xt(m: u8, at: u8, o: u8) -> Self {
        Self { m, at, o, q: 255 }
    }

    pub const fn dbi(m: u8, at: u8, o: u8, q: u8) -> Self {
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

    pub fn dcv(self) -> Rgb888 {
        Rgb888::new(self.m, self.at, self.o)
    }

    
    pub const Ox: Color2D = Color2D::xt(0, 0, 0);
    pub const Zm: Color2D = Color2D::xt(255, 255, 255);
    pub const Bqa: Color2D = Color2D::xt(255, 0, 0);
    pub const Bht: Color2D = Color2D::xt(0, 255, 0);
    pub const Bci: Color2D = Color2D::xt(0, 0, 255);
    pub const Cqs: Color2D = Color2D::xt(255, 255, 0);
    pub const Bzg: Color2D = Color2D::xt(0, 255, 255);
    pub const Cgz: Color2D = Color2D::xt(255, 0, 255);
    pub const Cxx: Color2D = Color2D::xt(128, 128, 128);
    pub const DIN_: Color2D = Color2D::xt(64, 64, 64);
    pub const DSO_: Color2D = Color2D::xt(192, 192, 192);
    pub const Ddj: Color2D = Color2D::xt(255, 165, 0);
    pub const Dee: Color2D = Color2D::xt(128, 0, 128);
    pub const Anl: Color2D = Color2D::new(0, 0, 0, 0);
}


pub struct Ckk<'a> {
    cd: &'a mut Sn,
}

impl<'a> Ckk<'a> {
    pub fn new(cd: &'a mut Sn) -> Self {
        Self { cd }
    }

    
    pub fn clear(&mut self, s: Color2D) {
        self.cd.hcv(s.lv());
    }

    
    pub fn line(&mut self, dn: i32, dp: i32, hy: i32, jz: i32, s: Color2D, ahw: u32) {
        let amx = PrimitiveStyle::ihi(s.dcv(), ahw);
        let _ = Line::new(Point::new(dn, dp), Point::new(hy, jz))
            .dsf(amx)
            .po(self.cd);
    }

    
    pub fn ha(&mut self, b: i32, c: i32, d: u32, i: u32, s: Color2D, ahw: u32) {
        let amx = PrimitiveStyle::ihi(s.dcv(), ahw);
        let _ = Rectangle::new(Point::new(b, c), EgSize::new(d, i))
            .dsf(amx)
            .po(self.cd);
    }

    
    pub fn ah(&mut self, b: i32, c: i32, d: u32, i: u32, s: Color2D) {
        let amx = PrimitiveStyle::jwz(s.dcv());
        let _ = Rectangle::new(Point::new(b, c), EgSize::new(d, i))
            .dsf(amx)
            .po(self.cd);
    }

    
    pub fn zkl(&mut self, b: i32, c: i32, d: u32, i: u32, dy: u32, s: Color2D, ahw: u32) {
        let amx = PrimitiveStyle::ihi(s.dcv(), ahw);
        let _ = RoundedRectangle::new(
            Rectangle::new(Point::new(b, c), EgSize::new(d, i)),
            CornerRadii::new(EgSize::new(dy, dy)),
        )
        .dsf(amx)
        .po(self.cd);
    }

    
    pub fn afp(&mut self, b: i32, c: i32, d: u32, i: u32, dy: u32, s: Color2D) {
        let amx = PrimitiveStyle::jwz(s.dcv());
        let _ = RoundedRectangle::new(
            Rectangle::new(Point::new(b, c), EgSize::new(d, i)),
            CornerRadii::new(EgSize::new(dy, dy)),
        )
        .dsf(amx)
        .po(self.cd);
    }

    
    pub fn yie(&mut self, cx: i32, ae: i32, dy: u32, s: Color2D, ahw: u32) {
        let amx = PrimitiveStyle::ihi(s.dcv(), ahw);
        let _ = Circle::new(Point::new(cx - dy as i32, ae - dy as i32), dy * 2)
            .dsf(amx)
            .po(self.cd);
    }

    
    pub fn abc(&mut self, cx: i32, ae: i32, dy: u32, s: Color2D) {
        let amx = PrimitiveStyle::jwz(s.dcv());
        let _ = Circle::new(Point::new(cx - dy as i32, ae - dy as i32), dy * 2)
            .dsf(amx)
            .po(self.cd);
    }

    
    pub fn you(&mut self, cx: i32, ae: i32, kb: u32, ix: u32, s: Color2D, ahw: u32) {
        let amx = PrimitiveStyle::ihi(s.dcv(), ahw);
        let _ = Ellipse::new(Point::new(cx - kb as i32, ae - ix as i32), EgSize::new(kb * 2, ix * 2))
            .dsf(amx)
            .po(self.cd);
    }

    
    pub fn yql(&mut self, cx: i32, ae: i32, kb: u32, ix: u32, s: Color2D) {
        let amx = PrimitiveStyle::jwz(s.dcv());
        let _ = Ellipse::new(Point::new(cx - kb as i32, ae - ix as i32), EgSize::new(kb * 2, ix * 2))
            .dsf(amx)
            .po(self.cd);
    }

    
    pub fn ztp(&mut self, dn: i32, dp: i32, hy: i32, jz: i32, ajr: i32, dnn: i32, s: Color2D, ahw: u32) {
        let amx = PrimitiveStyle::ihi(s.dcv(), ahw);
        let _ = Triangle::new(
            Point::new(dn, dp),
            Point::new(hy, jz),
            Point::new(ajr, dnn),
        )
        .dsf(amx)
        .po(self.cd);
    }

    
    pub fn kvx(&mut self, dn: i32, dp: i32, hy: i32, jz: i32, ajr: i32, dnn: i32, s: Color2D) {
        let amx = PrimitiveStyle::jwz(s.dcv());
        let _ = Triangle::new(
            Point::new(dn, dp),
            Point::new(hy, jz),
            Point::new(ajr, dnn),
        )
        .dsf(amx)
        .po(self.cd);
    }

    
    pub fn text(&mut self, b: i32, c: i32, text: &str, s: Color2D) {
        let amx = MonoTextStyle::new(&BVI_, s.dcv());
        let _ = Text::new(text, Point::new(b, c), amx).po(self.cd);
    }

    
    pub fn yvk(&mut self, b: i32, c: i32, d: u32, i: u32, qc: Color2D, abm: Color2D) {
        if i == 0 || d == 0 { return; }
        for br in 0..i {
            let ab = br as f32 / i as f32;
            let m = (qc.m as f32 + (abm.m as f32 - qc.m as f32) * ab) as u8;
            let at = (qc.at as f32 + (abm.at as f32 - qc.at as f32) * ab) as u8;
            let o = (qc.o as f32 + (abm.o as f32 - qc.o as f32) * ab) as u8;
            let s = 0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32);
            
            let x = c + br as i32;
            if x < 0 || x >= self.cd.ac as i32 { continue; }
            let dav = b.am(0) as u32;
            let hwe = ((b + d as i32) as u32).v(self.cd.z);
            if hwe <= dav { continue; }
            
            let afg = (x as u32 * self.cd.oq + dav) as usize;
            let hjj = (hwe - dav) as usize;
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::bed(
                    self.cd.bi.add(afg),
                    hjj,
                    s,
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                for bj in 0..hjj {
                    unsafe { *self.cd.bi.add(afg + bj) = s; }
                }
            }
        }
    }

    
    pub fn yvj(&mut self, b: i32, c: i32, d: u32, i: u32, fd: Color2D, hw: Color2D) {
        if i == 0 || d == 0 { return; }
        
        let mut lah = alloc::vec![0u32; d as usize];
        for bj in 0..d {
            let ab = bj as f32 / d as f32;
            let m = (fd.m as f32 + (hw.m as f32 - fd.m as f32) * ab) as u8;
            let at = (fd.at as f32 + (hw.at as f32 - fd.at as f32) * ab) as u8;
            let o = (fd.o as f32 + (hw.o as f32 - fd.o as f32) * ab) as u8;
            lah[bj as usize] = 0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32);
        }
        
        for br in 0..i {
            let x = c + br as i32;
            if x < 0 || x >= self.cd.ac as i32 { continue; }
            let dav = b.am(0) as u32;
            let hwe = ((b + d as i32) as u32).v(self.cd.z);
            if hwe <= dav { continue; }
            
            let pmt = (dav as i32 - b).am(0) as usize;
            let zg = (hwe - dav) as usize;
            let bgu = (x as u32 * self.cd.oq + dav) as usize;
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::dpd(
                    self.cd.bi.add(bgu),
                    lah.fq().add(pmt),
                    zg,
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                for a in 0..zg {
                    unsafe { *self.cd.bi.add(bgu + a) = lah[pmt + a]; }
                }
            }
        }
    }

    
    pub fn zc(&mut self, b: i32, c: i32, d: u32, i: u32, cou: u32, s: Color2D) {
        let qhg = s.q as f32 / cou as f32;
        for a in 0..cou {
            let dw = (s.q as f32 - qhg * a as f32) as u8;
            let dls = Color2D::new(s.m, s.at, s.o, dw);
            self.ha(
                b - a as i32,
                c - a as i32,
                d + a * 2,
                i + a * 2,
                dls,
                1,
            );
        }
    }
}






pub struct Sprite {
    pub z: u32,
    pub ac: u32,
    pub hz: Box<[u32]>,
}

impl Sprite {
    
    pub fn new(z: u32, ac: u32) -> Self {
        let aw = (z * ac) as usize;
        Self {
            z,
            ac,
            hz: alloc::vec![0u32; aw].dsd(),
        }
    }

    
    pub fn fjd(z: u32, ac: u32, hz: Vec<u32>) -> Self {
        Self {
            z,
            ac,
            hz: hz.dsd(),
        }
    }

    
    pub fn get(&self, b: u32, c: u32) -> u32 {
        if b < self.z && c < self.ac {
            self.hz[(c * self.z + b) as usize]
        } else {
            0
        }
    }

    
    pub fn oj(&mut self, b: u32, c: u32, s: u32) {
        if b < self.z && c < self.ac {
            self.hz[(c * self.z + b) as usize] = s;
        }
    }

    
    pub fn po(&self, cd: &mut Sn, b: i32, c: i32) {
        for cq in 0..self.ac {
            for cr in 0..self.z {
                let y = b + cr as i32;
                let x = c + cq as i32;
                if y >= 0 && x >= 0 {
                    let s = self.get(cr, cq);
                    if (s >> 24) > 0 { 
                        cd.aht(y as u32, x as u32, s);
                    }
                }
            }
        }
    }

    
    pub fn sbv(&self, cd: &mut Sn, b: i32, c: i32) {
        for cq in 0..self.ac {
            for cr in 0..self.z {
                let y = b + cr as i32;
                let x = c + cq as i32;
                if y >= 0 && x >= 0 && (y as u32) < cd.z && (x as u32) < cd.ac {
                    let cy = self.get(cr, cq);
                    let gsw = ((cy >> 24) & 0xFF) as f32 / 255.0;
                    
                    if gsw > 0.0 {
                        if gsw >= 1.0 {
                            cd.aht(y as u32, x as u32, cy);
                        } else {
                            let cs = cd.beg(y as u32, x as u32);
                            let krr = 1.0 - gsw;
                            
                            let m = (((cy >> 16) & 0xFF) as f32 * gsw + ((cs >> 16) & 0xFF) as f32 * krr) as u32;
                            let at = (((cy >> 8) & 0xFF) as f32 * gsw + ((cs >> 8) & 0xFF) as f32 * krr) as u32;
                            let o = ((cy & 0xFF) as f32 * gsw + (cs & 0xFF) as f32 * krr) as u32;
                            
                            let dei = 0xFF000000 | (m << 16) | (at << 8) | o;
                            cd.aht(y as u32, x as u32, dei);
                        }
                    }
                }
            }
        }
    }

    
    pub fn bv(&self, cst: u32, csr: u32) -> Self {
        let mut hyu = Sprite::new(cst, csr);
        
        for c in 0..csr {
            for b in 0..cst {
                let blg = (b * self.z / cst).v(self.z - 1);
                let bih = (c * self.ac / csr).v(self.ac - 1);
                hyu.oj(b, c, self.get(blg, bih));
            }
        }
        
        hyu
    }
}
