








use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;






const DZW_: usize = 8;


const GQ_: usize = 64;


const TE_: usize = 128;






pub struct FastSurface {
    
    pub f: Box<[u32]>,
    
    pub z: u32,
    
    pub ac: u32,
    
    no: DirtyRegion,
}

impl FastSurface {
    
    pub fn new(z: u32, ac: u32) -> Self {
        let aw = (z * ac) as usize;
        Self {
            f: alloc::vec![0u32; aw].dsd(),
            z,
            ac,
            no: DirtyRegion::new(z, ac),
        }
    }

    
    #[inline]
    pub fn clear(&mut self, s: u32) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            crate::graphics::simd::bed(
                self.f.mw(),
                self.f.len(),
                s
            );
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.f.vi(s);
        }
        self.no.olb();
    }

    
    #[inline]
    pub fn ah(&mut self, b: i32, c: i32, d: u32, i: u32, s: u32) {
        let dn = b.am(0) as u32;
        let dp = c.am(0) as u32;
        let hy = ((b + d as i32) as u32).v(self.z);
        let jz = ((c + i as i32) as u32).v(self.ac);
        
        if hy <= dn || jz <= dp { return; }
        
        let cml = (hy - dn) as usize;
        let oq = self.z as usize;
        
        for x in dp..jz {
            let mu = x as usize * oq + dn as usize;
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::bed(
                    self.f.mw().add(mu),
                    cml,
                    s
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                self.f[mu..mu + cml].vi(s);
            }
        }
        
        self.no.gxx(dn, dp, hy - dn, jz - dp);
    }

    
    #[inline(always)]
    pub fn aht(&mut self, b: u32, c: u32, s: u32) {
        if b < self.z && c < self.ac {
            self.f[(c * self.z + b) as usize] = s;
        }
    }

    
    #[inline(always)]
    pub fn beg(&self, b: u32, c: u32) -> u32 {
        if b < self.z && c < self.ac {
            self.f[(c * self.z + b) as usize]
        } else {
            0
        }
    }

    
    #[inline]
    pub fn obx(&mut self, b: i32, c: i32, len: u32, s: u32) {
        if c < 0 || c >= self.ac as i32 { return; }
        
        let dn = b.am(0) as u32;
        let hy = ((b + len as i32) as u32).v(self.z);
        if hy <= dn { return; }
        
        let c = c as u32;
        let ay = (c * self.z + dn) as usize;
        let hjj = (hy - dn) as usize;
        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            crate::graphics::simd::bed(
                self.f.mw().add(ay),
                hjj,
                s,
            );
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.f[ay..ay + hjj].vi(s);
        }
        
        self.no.gxx(dn, c, hy - dn, 1);
    }

    
    #[inline]
    pub fn pym(&mut self, b: i32, c: i32, len: u32, s: u32) {
        if b < 0 || b >= self.z as i32 { return; }
        
        let dp = c.am(0) as u32;
        let jz = ((c + len as i32) as u32).v(self.ac);
        if jz <= dp { return; }
        
        let b = b as u32;
        let oq = self.z;
        for x in dp..jz {
            self.f[(x * oq + b) as usize] = s;
        }
        
        self.no.gxx(b, dp, 1, jz - dp);
    }

    
    pub fn lx(&mut self, b: i32, c: i32, d: u32, i: u32, s: u32) {
        self.obx(b, c, d, s);
        self.obx(b, c + i as i32 - 1, d, s);
        self.pym(b, c, i, s);
        self.pym(b + d as i32 - 1, c, i, s);
    }

    
    pub fn bge(&mut self, cy: &FastSurface, buc: i32, bqg: i32) {
        self.qqf(cy, 0, 0, cy.z, cy.ac, buc, bqg);
    }

    
    pub fn qqf(&mut self, cy: &FastSurface, 
                       blg: u32, bih: u32, jri: u32, mhc: u32,
                       buc: i32, bqg: i32) {
        
        let asa = blg.v(cy.z);
        let bos = bih.v(cy.ac);
        let amy = (blg + jri).v(cy.z);
        let bcw = (bih + mhc).v(cy.ac);
        
        if amy <= asa || bcw <= bos { return; }
        
        
        let mut dx = buc;
        let mut bg = bqg;
        let mut aoo = (amy - asa) as i32;
        let mut bbg = (bcw - bos) as i32;
        let mut pmu = 0i32;
        let mut pmv = 0i32;
        
        
        if dx < 0 {
            pmu = -dx;
            aoo += dx;
            dx = 0;
        }
        
        if bg < 0 {
            pmv = -bg;
            bbg += bg;
            bg = 0;
        }
        
        if dx + aoo > self.z as i32 {
            aoo = self.z as i32 - dx;
        }
        
        if bg + bbg > self.ac as i32 {
            bbg = self.ac as i32 - bg;
        }
        
        if aoo <= 0 || bbg <= 0 { return; }
        
        let aoo = aoo as usize;
        let bbg = bbg as usize;
        let dx = dx as usize;
        let bg = bg as usize;
        let qfd = (asa as i32 + pmu) as usize;
        let qfe = (bos as i32 + pmv) as usize;
        
        
        let cid = cy.z as usize;
        let epi = self.z as usize;
        
        for br in 0..bbg {
            let mhd = (qfe + br) * cid + qfd;
            let krw = (bg + br) * epi + dx;
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::dpd(
                    self.f.mw().add(krw),
                    cy.f.fq().add(mhd),
                    aoo
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                self.f[krw..krw + aoo]
                    .dg(&cy.f[mhd..mhd + aoo]);
            }
        }
        
        self.no.gxx(dx as u32, bg as u32, aoo as u32, bbg as u32);
    }

    
    pub fn qqd(&mut self, cy: &FastSurface, buc: i32, bqg: i32) {
        let hhj = buc.am(0) as u32;
        let ksd = bqg.am(0) as u32;
        let jrj = if buc < 0 { (-buc) as u32 } else { 0 };
        let pmz = if bqg < 0 { (-bqg) as u32 } else { 0 };
        
        let aoo = (cy.z - jrj).v(self.z - hhj);
        let bbg = (cy.ac - pmz).v(self.ac - ksd);
        
        if aoo == 0 || bbg == 0 { return; }
        
        let cid = cy.z as usize;
        let epi = self.z as usize;
        
        for br in 0..bbg {
            let bxg = (pmz + br) as usize * cid;
            let hhd = (ksd + br) as usize * epi;
            
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                let aob = cy.f.fq().add(bxg + jrj as usize);
                let alc = self.f.mw().add(hhd + hhj as usize);
                crate::graphics::simd::kdu(alc, aob, aoo as usize);
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                
                let mut b = 0u32;
                while b + 8 <= aoo {
                    for a in 0..8 {
                        let blf = bxg + (jrj + b + a) as usize;
                        let bbm = hhd + (hhj + b + a) as usize;
                        let fvh = cy.f[blf];
                        let dw = fvh >> 24;
                        
                        if dw == 255 {
                            self.f[bbm] = fvh;
                        } else if dw > 0 {
                            self.f[bbm] = kds(fvh, self.f[bbm]);
                        }
                    }
                    b += 8;
                }
                
                
                while b < aoo {
                    let blf = bxg + (jrj + b) as usize;
                    let bbm = hhd + (hhj + b) as usize;
                    let fvh = cy.f[blf];
                    let dw = fvh >> 24;
                
                    if dw == 255 {
                        self.f[bbm] = fvh;
                    } else if dw > 0 {
                        self.f[bbm] = kds(fvh, self.f[bbm]);
                    }
                    b += 1;
                }
            }
        }
        
        self.no.gxx(hhj, ksd, aoo, bbg);
    }

    
    pub fn afp(&mut self, b: i32, c: i32, d: u32, i: u32, m: u32, s: u32) {
        let m = m.v(d / 2).v(i / 2);
        
        
        self.ah(b + m as i32, c, d - 2 * m, i, s);
        
        
        self.ah(b, c + m as i32, m, i - 2 * m, s);
        self.ah(b + d as i32 - m as i32, c + m as i32, m, i - 2 * m, s);
        
        
        self.iun(b + m as i32, c + m as i32, m, s, Corner::Dp);
        self.iun(b + d as i32 - m as i32 - 1, c + m as i32, m, s, Corner::Dq);
        self.iun(b + m as i32, c + i as i32 - m as i32 - 1, m, s, Corner::Dt);
        self.iun(b + d as i32 - m as i32 - 1, c + i as i32 - m as i32 - 1, m, s, Corner::Du);
    }

    fn iun(&mut self, cx: i32, ae: i32, m: u32, s: u32, hea: Corner) {
        let m = m as i32;
        let bwl = m * m;
        
        for bg in 0..=m {
            for dx in 0..=m {
                if dx * dx + bg * bg <= bwl {
                    let (y, x) = match hea {
                        Corner::Dp => (cx - dx, ae - bg),
                        Corner::Dq => (cx + dx, ae - bg),
                        Corner::Dt => (cx - dx, ae + bg),
                        Corner::Du => (cx + dx, ae + bg),
                    };
                    if y >= 0 && x >= 0 && y < self.z as i32 && x < self.ac as i32 {
                        self.aht(y as u32, x as u32, s);
                    }
                }
            }
        }
    }

    
    pub fn tdi(&self) -> &DirtyRegion {
        &self.no
    }

    
    pub fn yij(&mut self) {
        self.no.clear();
    }

    
    pub fn suz(&mut self) {
        if self.no.asw {
            self.svc();
        } else {
            for a in 0..self.no.az {
                let ha = self.no.akn[a];
                self.svb(ha.b, ha.c, ha.d, ha.i);
            }
        }
        self.no.clear();
    }

    
    pub fn svc(&self) {
        let (lu, qh) = crate::framebuffer::yn();
        let aoo = self.z.v(lu) as usize;
        let bbg = self.ac.v(qh) as usize;
        
        let dqt = crate::framebuffer::iwp();
        let fij = crate::framebuffer::kyo();
        
        if dqt.abq() { return; }
        
        let cid = self.z as usize;
        
        for br in 0..bbg {
            let big = br * cid;
            let bgu = br * fij;
            
            unsafe {
                let cy = self.f.fq().add(big);
                let cs = dqt.add(bgu) as *mut u32;
                core::ptr::copy_nonoverlapping(cy, cs, aoo);
            }
        }
    }

    
    fn svb(&self, b: u32, c: u32, d: u32, i: u32) {
        let (lu, qh) = crate::framebuffer::yn();
        
        let dn = b.v(lu).v(self.z);
        let dp = c.v(qh).v(self.ac);
        let hy = (b + d).v(lu).v(self.z);
        let jz = (c + i).v(qh).v(self.ac);
        
        if hy <= dn || jz <= dp { return; }
        
        let dqt = crate::framebuffer::iwp();
        let fij = crate::framebuffer::kyo();
        
        if dqt.abq() { return; }
        
        let aoo = (hy - dn) as usize;
        let cid = self.z as usize;
        
        for br in dp..jz {
            let big = br as usize * cid + dn as usize;
            let bgu = br as usize * fij + dn as usize * 4;
            
            unsafe {
                let cy = self.f.fq().add(big);
                let cs = dqt.add(bgu) as *mut u32;
                core::ptr::copy_nonoverlapping(cy, cs, aoo);
            }
        }
    }
}

enum Corner {
    Dp,
    Dq,
    Dt,
    Du,
}





#[derive(Clone, Copy, Default)]
pub struct Rect {
    pub b: u32,
    pub c: u32,
    pub d: u32,
    pub i: u32,
}

pub struct DirtyRegion {
    pub akn: [Rect; GQ_],
    pub az: usize,
    pub asw: bool,
    wf: u32,
    aav: u32,
}

impl DirtyRegion {
    pub fn new(z: u32, ac: u32) -> Self {
        Self {
            akn: [Rect::default(); GQ_],
            az: 0,
            asw: true, 
            wf: z,
            aav: ac,
        }
    }

    pub fn gxx(&mut self, b: u32, c: u32, d: u32, i: u32) {
        if self.asw { return; }
        if d == 0 || i == 0 { return; }
        
        let lob = Rect { b, c, d, i };
        
        
        for a in 0..self.az {
            if vtj(&self.akn[a], &lob) {
                self.akn[a] = ung(&self.akn[a], &lob);
                return;
            }
        }
        
        
        if self.az < GQ_ {
            self.akn[self.az] = lob;
            self.az += 1;
        } else {
            
            self.asw = true;
        }
    }

    pub fn olb(&mut self) {
        self.asw = true;
    }

    pub fn clear(&mut self) {
        self.az = 0;
        self.asw = false;
    }
}

fn vtj(q: &Rect, o: &Rect) -> bool {
    !(q.b + q.d <= o.b || o.b + o.d <= q.b ||
      q.c + q.i <= o.c || o.c + o.i <= q.c)
}

fn ung(q: &Rect, o: &Rect) -> Rect {
    let dn = q.b.v(o.b);
    let dp = q.c.v(o.c);
    let hy = (q.b + q.d).am(o.b + o.d);
    let jz = (q.c + q.i).am(o.c + o.i);
    Rect { b: dn, c: dp, d: hy - dn, i: jz - dp }
}






#[inline(always)]
fn kds(cy: u32, cs: u32) -> u32 {
    let dw = (cy >> 24) & 0xFF;
    if dw == 0 { return cs; }
    if dw == 255 { return cy; }
    
    let akg = 255 - dw;
    
    
    let adz = (cy >> 16) & 0xFF;
    let bsi = (cy >> 8) & 0xFF;
    let is = cy & 0xFF;
    
    let ahh = (cs >> 16) & 0xFF;
    let bgs = (cs >> 8) & 0xFF;
    let ng = cs & 0xFF;
    
    
    let m = (adz * dw + ahh * akg + 127) / 255;
    let at = (bsi * dw + bgs * akg + 127) / 255;
    let o = (is * dw + ng * akg + 127) / 255;
    
    0xFF000000 | (m << 16) | (at << 8) | o
}






pub struct GlyphCache {
    
    cqz: [[u32; 128]; TE_], 
    axw: u32,
    jr: bool,
}

impl GlyphCache {
    pub const fn new() -> Self {
        Self {
            cqz: [[0u32; 128]; TE_],
            axw: 0xFFFFFFFF,
            jr: false,
        }
    }

    
    pub fn init(&mut self, axw: u32) {
        self.axw = axw;
        
        for r in 0..TE_ {
            let hlr = crate::framebuffer::font::ada(r as u8 as char);
            let mut ctj = 0;
            
            for br in 0..16 {
                let fs = hlr[br];
                for bj in 0..8 {
                    if (fs >> (7 - bj)) & 1 == 1 {
                        self.cqz[r][ctj] = axw;
                    } else {
                        self.cqz[r][ctj] = 0; 
                    }
                    ctj += 1;
                }
            }
        }
        
        self.jr = true;
    }

    
    pub fn sdi(&self, surface: &mut FastSurface, r: char, b: i32, c: i32) {
        let w = (r as usize).v(TE_ - 1);
        let ka = &self.cqz[w];
        
        let mut ctj = 0;
        for br in 0..16 {
            let x = c + br;
            if x >= 0 && x < surface.ac as i32 {
                for bj in 0..8 {
                    let y = b + bj;
                    if y >= 0 && y < surface.z as i32 {
                        let s = ka[ctj];
                        if s != 0 {
                            surface.aht(y as u32, x as u32, s);
                        }
                    }
                    ctj += 1;
                }
            } else {
                ctj += 8;
            }
        }
    }

    
    pub fn sft(&self, surface: &mut FastSurface, e: &str, b: i32, c: i32) {
        let mut cx = b;
        for r in e.bw() {
            if cx >= surface.z as i32 { break; }
            self.sdi(surface, r, cx, c);
            cx += 8;
        }
    }
}


static ASF_: Mutex<GlyphCache> = Mutex::new(GlyphCache::new());


pub fn yxq(axw: u32) {
    ASF_.lock().init(axw);
}


pub fn cb(surface: &mut FastSurface, e: &str, b: i32, c: i32) {
    ASF_.lock().sft(surface, e, b, c);
}






pub struct Layer {
    pub surface: FastSurface,
    pub b: i32,
    pub c: i32,
    pub av: i32,
    pub iw: bool,
    pub ad: u32,
}


pub struct Cdk {
    
    pub an: FastSurface,
    
    my: Vec<Layer>,
    
    bcb: u32,
    
    vp: u32,
}

impl Cdk {
    pub fn new(z: u32, ac: u32) -> Self {
        Self {
            an: FastSurface::new(z, ac),
            my: Vec::new(),
            bcb: 1,
            vp: 0xFF101010,
        }
    }

    
    pub fn wig(&mut self, s: u32) {
        self.vp = s;
    }

    
    pub fn ykl(&mut self, z: u32, ac: u32, b: i32, c: i32, av: i32) -> u32 {
        let ad = self.bcb;
        self.bcb += 1;
        
        let fl = Layer {
            surface: FastSurface::new(z, ac),
            b,
            c,
            av,
            iw: true,
            ad,
        };
        
        self.my.push(fl);
        self.pmb();
        
        ad
    }

    
    pub fn iws(&mut self, ad: u32) -> Option<&mut Layer> {
        self.my.el().du(|dm| dm.ad == ad)
    }

    
    pub fn vux(&mut self, ad: u32) {
        self.my.ajm(|dm| dm.ad != ad);
    }

    
    pub fn zdb(&mut self, ad: u32, b: i32, c: i32) {
        if let Some(fl) = self.iws(ad) {
            fl.b = b;
            fl.c = c;
        }
    }

    
    pub fn zhf(&mut self, ad: u32) {
        let umd = self.my.iter().map(|dm| dm.av).am().unwrap_or(0);
        if let Some(fl) = self.iws(ad) {
            fl.av = umd + 1;
        }
        self.pmb();
    }

    fn pmb(&mut self) {
        self.my.bxf(|dm| dm.av);
    }

    
    pub fn iov(&mut self) {
        
        self.an.clear(self.vp);
        
        
        for fl in &self.my {
            if fl.iw {
                self.an.qqd(&fl.surface, fl.b, fl.c);
            }
        }
    }

    
    pub fn brs(&mut self) {
        self.an.suz();
    }

    
    pub fn tj(&mut self) {
        self.iov();
        self.brs();
    }
}
