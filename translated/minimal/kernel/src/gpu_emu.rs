









use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use alloc::vec::Vec;



pub const YJ_: usize = 32;


pub const DBP_: usize = 64;


pub const DTO_: usize = 256;






#[derive(Clone, Copy)]
#[repr(C)]
pub struct Fy {
    pub b: u32,
    pub c: u32,
    pub z: u32,
    pub ac: u32,
    pub time: f32,
    pub frame: u32,
}


#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct PixelOutput {
    pub m: u8,
    pub at: u8,
    pub o: u8,
    pub q: u8,
}

impl PixelOutput {
    #[inline]
    pub fn lv(&self) -> u32 {
        ((self.q as u32) << 24) | ((self.m as u32) << 16) | ((self.at as u32) << 8) | (self.o as u32)
    }
    
    #[inline]
    pub fn bul(m: u8, at: u8, o: u8) -> Self {
        Self { m, at, o, q: 255 }
    }
    
    #[inline]
    pub fn syf(m: u8, at: u8, o: u8, q: u8) -> Self {
        Self { m, at, o, q }
    }
}


pub type Ty = fn(input: Fy) -> PixelOutput;


pub type Ctl = fn(yuz: u32, zbn: u32, zty: *const u8) -> u32;






pub struct VirtualGpu {
    
    frame: AtomicU32,
    
    ejx: AtomicU32,
    
    iiv: Option<Ty>,
    
    framebuffer: *mut u32,
    
    z: u32,
    ac: u32,
    
    oq: u32,
    
    rpa: [AtomicBool; YJ_],
    
    dng: AtomicU32,
}

unsafe impl Send for VirtualGpu {}
unsafe impl Sync for VirtualGpu {}

impl VirtualGpu {
    
    pub const fn new() -> Self {
        const CBR_: AtomicBool = AtomicBool::new(false);
        Self {
            frame: AtomicU32::new(0),
            ejx: AtomicU32::new(0),
            iiv: None,
            framebuffer: core::ptr::null_mut(),
            z: 0,
            ac: 0,
            oq: 0,
            rpa: [CBR_; YJ_],
            dng: AtomicU32::new(0),
        }
    }
    
    
    
    pub fn init(&mut self, framebuffer: *mut u32, z: u32, ac: u32, oq: u32) {
        self.framebuffer = framebuffer;
        self.z = z;
        self.ac = ac;
        self.oq = oq;
    }
    
    
    pub fn hzy(&mut self, bfg: Ty) {
        self.iiv = Some(bfg);
    }
    
    
    
    pub fn nlu(&self) {
        let Some(bfg) = self.iiv else { return };
        
        let z = self.z;
        let ac = self.ac;
        let jtv = (z * ac) as usize;
        let time = self.ejx.load(Ordering::Relaxed) as f32 / 1000.0;
        let frame = self.frame.load(Ordering::Relaxed);
        
        
        let aao = crate::cpu::smp::boc() as usize;
        let aao = aao.am(1);
        
        
        let zfi = (jtv + aao - 1) / aao;
        
        
        self.dng.store(0, Ordering::Release);
        
        
        let be = Kx {
            bfg,
            framebuffer: self.framebuffer,
            z,
            ac,
            oq: self.oq,
            time,
            frame,
        };
        
        
        
        crate::cpu::smp::daj(
            ac as usize,
            nlv,
            &be as *const Kx as *mut u8,
        );
        
        
        self.frame.fetch_add(1, Ordering::Relaxed);
    }
    
    
    #[cfg(target_arch = "x86_64")]
    pub fn ryh(&self) {
        let Some(bfg) = self.iiv else { return };
        
        let z = self.z;
        let ac = self.ac;
        let time = self.ejx.load(Ordering::Relaxed) as f32 / 1000.0;
        let frame = self.frame.load(Ordering::Relaxed);
        
        let be = Kx {
            bfg,
            framebuffer: self.framebuffer,
            z,
            ac,
            oq: self.oq,
            time,
            frame,
        };
        
        
        crate::cpu::smp::daj(
            ac as usize,
            nlw,
            &be as *const Kx as *mut u8,
        );
        
        self.frame.fetch_add(1, Ordering::Relaxed);
    }
    
    
    pub fn or(&self, koy: u32) {
        self.ejx.fetch_add(koy, Ordering::Relaxed);
    }
    
    
    pub fn frame(&self) -> u32 {
        self.frame.load(Ordering::Relaxed)
    }
    
    
    pub fn time(&self) -> f32 {
        self.ejx.load(Ordering::Relaxed) as f32 / 1000.0
    }
}














#[inline]
pub fn nlu(
    framebuffer: *mut u32,
    z: u32,
    ac: u32,
    time: f32,
    frame: u32,
    bfg: Ty,
) {
    ryi(framebuffer, z, ac, z, time, frame, bfg);
}


pub fn ryi(
    framebuffer: *mut u32,
    z: u32,
    ac: u32,
    oq: u32,
    time: f32,
    frame: u32,
    bfg: Ty,
) {
    let be = Kx {
        bfg,
        framebuffer,
        z,
        ac,
        oq,
        time,
        frame,
    };

    
    #[cfg(target_arch = "x86_64")]
    {
        crate::cpu::smp::daj(
            ac as usize,
            nlw,
            &be as *const Kx as *mut u8,
        );
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        crate::cpu::smp::daj(
            ac as usize,
            nlv,
            &be as *const Kx as *mut u8,
        );
    }
}






#[repr(C)]
struct Kx {
    bfg: Ty,
    framebuffer: *mut u32,
    z: u32,
    ac: u32,
    oq: u32,  
    time: f32,
    frame: u32,
}

unsafe impl Send for Kx {}
unsafe impl Sync for Kx {}


fn nlv(ay: usize, ci: usize, f: *mut u8) {
    let be = unsafe { &*(f as *const Kx) };
    let bfg = be.bfg;
    let pq = be.framebuffer;
    let z = be.z;
    let ac = be.ac;
    let oq = be.oq as usize;

    for c in ay..ci {
        let afg = c * oq;
        for b in 0..z as usize {
            let input = Fy {
                b: b as u32,
                c: c as u32,
                z,
                ac,
                time: be.time,
                frame: be.frame,
            };

            let an = bfg(input);
            unsafe {
                *pq.add(afg + b) = an.lv();
            }
        }
    }
}


#[cfg(target_arch = "x86_64")]
fn nlw(ay: usize, ci: usize, f: *mut u8) {
    use core::arch::x86_64::*;
    
    let be = unsafe { &*(f as *const Kx) };
    let bfg = be.bfg;
    let pq = be.framebuffer;
    let z = be.z as usize;
    let ac = be.ac;
    let oq = be.oq as usize;
    
    for c in ay..ci {
        let afg = c * oq;
        
        
        let mut b = 0;
        while b + 4 <= z {
            
            let mut colors = [0u32; 4];
            
            for a in 0..4 {
                let input = Fy {
                    b: (b + a) as u32,
                    c: c as u32,
                    z: z as u32,
                    ac,
                    time: be.time,
                    frame: be.frame,
                };
                colors[a] = bfg(input).lv();
            }
            
            
            unsafe {
                let hz = byb(colors.fq() as *const acb);
                ccs(pq.add(afg + b) as *mut acb, hz);
            }
            
            b += 4;
        }
        
        
        while b < z {
            let input = Fy {
                b: b as u32,
                c: c as u32,
                z: z as u32,
                ac,
                time: be.time,
                frame: be.frame,
            };
            unsafe {
                *pq.add(afg + b) = bfg(input).lv();
            }
            b += 1;
        }
    }
}






pub fn wmd(input: Fy) -> PixelOutput {
    let b = input.b as f32 / input.z as f32;
    let c = input.c as f32 / input.ac as f32;
    let ab = input.time;
    
    
    let agy = lz(b * 10.0 + ab);
    let apg = lz(c * 10.0 + ab * 1.5);
    let bdf = lz((b + c) * 5.0 + ab * 0.7);
    let cnq = lz(ahn((b - 0.5) * (b - 0.5) + (c - 0.5) * (c - 0.5)) * 10.0 - ab * 2.0);
    
    let p = (agy + apg + bdf + cnq) / 4.0;
    
    
    let m = ((p + 1.0) * 0.5 * 255.0) as u8;
    let at = ((lz(p * 3.14159 + ab) + 1.0) * 0.5 * 255.0) as u8;
    let o = ((lz(p * 3.14159 * 2.0 + ab * 1.3) + 1.0) * 0.5 * 255.0) as u8;
    
    PixelOutput::bul(m, at, o)
}


pub fn wlz(input: Fy) -> PixelOutput {
    let b = input.b;
    let c = input.c;
    let ab = input.time;
    let d = input.z;
    let i = input.ac;
    
    
    let acc = 8u32;
    let aqw = 16u32;
    let bj = b / acc;
    let br = c / aqw;
    let bhi = b % acc;
    let alk = c % aqw;
    
    
    let dpa = bj.hx(2654435761);
    let ffk = (dpa & 0xFFFF) as f32 / 65535.0;
    let rlv = ((dpa >> 8) & 0xFFFF) as f32 / 65535.0;
    
    
    let ig = 3.0 + ffk * 8.0;           
    let l = rlv * 50.0;               
    let acr = 8.0 + ffk * 12.0;       
    
    
    let lbr = ((ab * ig + l) % ((i / aqw + 30) as f32)) as i32 - 15;
    let mat = br as i32;
    
    
    let la = lbr - mat;
    
    if la < 0 || la > acr as i32 {
        
        return PixelOutput::bul(0, 0, 0);
    }
    
    
    let cuv = la as f32 / acr;
    let kt = if la == 0 {
        1.0  
    } else {
        (1.0 - cuv) * 0.7
    };
    
    
    let feo = (bj.hx(31337) ^ br.hx(7919) ^ (input.frame / 3)) as f32;
    let tgh = vnv(bhi, alk, feo as u32);
    
    if !tgh {
        
        let tq = kt * 0.15;
        let at = (tq * 255.0) as u8;
        return PixelOutput::bul(0, at / 2, at / 4);
    }
    
    
    let bji = kt * 255.0;
    let m = if la == 0 { (bji * 0.8) as u8 } else { 0 };  
    let at = bji as u8;
    let o = if la == 0 { (bji * 0.8) as u8 } else { (bji * 0.2) as u8 };
    
    PixelOutput::bul(m, at, o)
}


#[inline]
fn vnv(mj: u32, ct: u32, dv: u32) -> bool {
    
    let hash = dv
        .hx(2654435761)
        .cn(mj.hx(7919))
        .cn(ct.hx(31337));
    
    
    let vfl = (dv / 7) % 8;
    
    match vfl {
        0 => {
            
            ct % 4 < 2 && mj > 1 && mj < 6
        },
        1 => {
            
            mj == 3 || mj == 4 || (ct % 5 == 0 && mj > 1)
        },
        2 => {
            
            (ct == 2 || ct == 13) && mj > 1 && mj < 6 ||
            (mj == 2 || mj == 5) && ct > 2 && ct < 13
        },
        3 => {
            
            let wz = if mj > ct / 2 { mj - ct / 2 } else { ct / 2 - mj };
            wz < 2
        },
        4 => {
            
            (mj == 3 || mj == 4) && ct > 2 && ct < 14 ||
            (ct == 7 || ct == 8) && mj > 0 && mj < 7
        },
        5 => {
            
            (hash % 3) == 0 && mj > 0 && mj < 7 && ct > 1 && ct < 14
        },
        6 => {
            
            let vs = 4i32;
            let cml = (ct as i32 - 2).am(0).v(6);
            let rzc = (mj as i32 - vs).gp();
            ct > 2 && ct < 14 && rzc <= cml / 2
        },
        _ => {
            
            mj > 0 && mj < 7 && ct > 1 && ct < 14 && (mj + ct) % 3 != 0
        }
    }
}


pub fn wly(input: Fy) -> PixelOutput {
    let ddn = 2.5 + lz(input.time * 0.3) * 0.5;
    let cx = (input.b as f32 / input.z as f32 - 0.7) * ddn;
    let ae = (input.c as f32 / input.ac as f32 - 0.5) * ddn;
    
    let mut gxd = 0.0f32;
    let mut gxe = 0.0f32;
    let mut iter = 0u32;
    const AFB_: u32 = 64;
    
    while gxd * gxd + gxe * gxe < 4.0 && iter < AFB_ {
        let gup = gxd * gxd - gxe * gxe + cx;
        gxe = 2.0 * gxd * gxe + ae;
        gxd = gup;
        iter += 1;
    }
    
    if iter == AFB_ {
        PixelOutput::bul(0, 0, 0)
    } else {
        let ab = iter as f32 / AFB_ as f32;
        let m = (ab * 255.0) as u8;
        let at = (srb(ab * 2.0) * 255.0) as u8;
        let o = ((1.0 - ab) * 255.0) as u8;
        PixelOutput::bul(m, at, o)
    }
}


pub fn wlv(input: Fy) -> PixelOutput {
    let m = (input.b * 255 / input.z) as u8;
    let at = (input.c * 255 / input.ac) as u8;
    let o = ((input.time * 50.0) as u32 % 256) as u8;
    PixelOutput::bul(m, at, o)
}


pub fn wlu(input: Fy) -> PixelOutput {
    let b = input.b as f32;
    let c = input.c as f32;
    let i = input.ac as f32;
    let ab = input.time;
    
    
    let uuy = lz(b * 0.1 + ab * 3.0) * 0.5 + 0.5;
    let uuz = lz(b * 0.17 + ab * 2.3) * 0.5 + 0.5;
    let uva = lz(b * 0.23 + c * 0.1 + ab * 1.7) * 0.5 + 0.5;
    
    let qnk = 1.0 - (c / i);
    let xc = qnk * (0.5 + uuy * 0.2 + uuz * 0.2 + uva * 0.1);
    let xc = xc.am(0.0).v(1.0);
    
    
    let (m, at, o) = if xc < 0.2 {
        let ab = xc / 0.2;
        ((ab * 128.0) as u8, 0, 0)
    } else if xc < 0.5 {
        let ab = (xc - 0.2) / 0.3;
        (128 + (ab * 127.0) as u8, (ab * 100.0) as u8, 0)
    } else if xc < 0.8 {
        let ab = (xc - 0.5) / 0.3;
        (255, 100 + (ab * 155.0) as u8, (ab * 50.0) as u8)
    } else {
        let ab = (xc - 0.8) / 0.2;
        (255, 255, 50 + (ab * 205.0) as u8)
    };
    
    PixelOutput::bul(m, at, o)
}







pub fn wlx(input: Fy) -> PixelOutput {
    let d = input.z as f32;
    let i = input.ac as f32;
    let ab = input.time;
    
    
    let cx = (input.b as f32 - d * 0.5) / (i * 0.5);  
    let ae = (input.c as f32 - i * 0.5) / (i * 0.5);
    
    
    let dy = ahn(cx * cx + ae * ae).am(0.001);
    let hg = itz(ae, cx);
    
    
    
    let eo = 1.0 / dy;
    
    
    let av = eo + ab * 2.5;
    
    
    
    let cnz = (hg / 6.28318 + 0.5) % 1.0;
    let cnz = if cnz < 0.0 { cnz + 1.0 } else { cnz };
    
    
    let xto = av % 1.0;
    
    
    let kgy = 0.08;
    let nby = 0.12;  
    
    let dzm = (cnz / kgy) as u32;
    let bmg = (av / nby) as u32;
    let bhi = (cnz % kgy) / kgy;
    let alk = (xto / nby) % 1.0;
    
    
    let feo = dzm.hx(31337) ^ bmg.hx(7919);
    let nzi = pwk(bhi, alk, feo);
    
    
    
    let cer = (dy * 1.5).v(1.0);
    let rvs = (lz(av * 3.0 + ab * 4.0) * 0.15 + 0.85);
    let kt = cer * rvs;
    
    
    let ys = if (input.c % 3) == 0 { 0.85 } else { 1.0 };
    
    
    let nzs = xng(cnz, av, ab);
    
    
    let gzv = if nzi {
        kt * ys
    } else {
        kt * 0.08 * ys  
    };
    
    
    let mmg = (gzv + nzs * 0.4).v(1.0);
    
    
    
    let nko = (1.0 - cer) * 0.3;  
    
    let m = (mmg * (80.0 + nko * 100.0) * kt) as u8;
    let at = (mmg * 255.0) as u8;
    let o = (mmg * (60.0 + nko * 150.0 + nzs * 80.0)) as u8;
    
    
    let nva = (av * 8.0 + ab * 10.0) % 1.0;
    if nzi && nva < 0.05 && dy < 0.8 {
        let ceq = (1.0 - nva / 0.05) * kt;
        let xb = (m as f32 + ceq * 200.0).v(255.0) as u8;
        let lp = (at as f32 + ceq * 50.0).v(255.0) as u8;
        let pq = (o as f32 + ceq * 200.0).v(255.0) as u8;
        return PixelOutput::bul(xb, lp, pq);
    }
    
    PixelOutput::bul(m, at, o)
}


#[inline]
fn pwk(mj: f32, ct: f32, dv: u32) -> bool {
    let pattern = dv % 12;
    let y = (mj * 8.0) as u32;
    let x = (ct * 12.0) as u32;
    
    match pattern {
        0 => x > 2 && x < 10 && (y == 2 || y == 5),  
        1 => x == 3 || x == 8 || (y == 4 && x > 2 && x < 10),  
        2 => (y + x) % 3 == 0,  
        3 => y > 1 && y < 6 && (x == 2 || x == 9),  
        4 => (y == 3 || y == 4) && x > 1 && x < 11,  
        5 => x > 2 && x < 10 && y > 1 && y < 6 && (x - 2) % 2 == 0,  
        6 => {  
            (x == 2 || x == 9) && y > 1 && y < 6 ||
            (y == 2 || y == 5) && x > 2 && x < 9
        },
        7 => y == 3 && x > 1 && x < 11 || x == 6 && y > 0 && y < 7,  
        8 => (y + x / 2) % 4 == 0,  
        9 => x > 3 && x < 9 && ((y > 1 && y < 4) || (y > 4 && y < 7)),  
        10 => {  
            let pn = 3.5;
            let la = if y as f32 > pn { y as f32 - pn } else { pn - y as f32 };
            x > 2 && x < 10 && la < (x - 2) as f32 * 0.4
        },
        _ => (dv.hx(y) ^ x) % 3 == 0,  
    }
}


#[inline]
fn xng(cnz: f32, av: f32, ab: f32) -> f32 {
    
    let vpp = 16.0;
    let ozc = cnz * vpp;
    let ozb = axv(ozc - (ozc as i32) as f32 - 0.5);
    let vpq = if ozb < 0.08 { (0.08 - ozb) / 0.08 } else { 0.0 };
    
    
    let vze = 0.3;
    let pdl = av / vze;
    let vyz = pdl - (pdl as i32) as f32;
    let pdi = axv(vyz - 0.5);
    let vzb = if pdi < 0.05 { (0.05 - pdi) / 0.05 * 0.5 } else { 0.0 };
    
    
    let xg = lz(av * 2.0 - ab * 8.0) * 0.3 + 0.7;
    
    (vpq * 0.6 + vzb * xg) * 0.8
}


#[inline(always)]
fn itz(c: f32, b: f32) -> f32 { crate::math::itz(c, b) }






pub fn wlw(input: Fy) -> PixelOutput {
    let b = input.b;
    let c = input.c;
    let ab = input.time;
    let i = input.ac;
    
    
    let mut iem = 0.0f32;
    let mut ieh = 0.0f32;
    let mut iec = 0.0f32;
    
    
    let (ctp, szv, wu) = jio(b, c, ab, i, 0.4, 0.15, 6, 12, 0);
    iem += ctp * 0.3;
    ieh += szv * 0.3;
    iec += wu * 0.5;  
    
    
    let (aqh, cyd, of) = jio(b, c, ab, i, 0.7, 0.35, 7, 14, 100);
    iem += aqh * 0.5;
    ieh += cyd * 0.5;
    iec += of * 0.35;
    
    
    let (uv, cqu, tb) = jio(b, c, ab, i, 1.0, 0.65, 8, 16, 200);
    iem += uv * 0.7;
    ieh += cqu * 0.7;
    iec += tb * 0.25;
    
    
    let (ctq, szw, ajw) = jio(b, c, ab, i, 1.5, 1.0, 10, 20, 300);
    iem += ctq;
    ieh += szw;
    iec += ajw * 0.2;
    
    
    let ys = if (c % 2) == 0 { 0.92 } else { 1.0 };
    
    let m = (iem * ys).v(255.0) as u8;
    let at = (ieh * ys).v(255.0) as u8;
    let o = (iec * ys).v(60.0) as u8;  
    
    PixelOutput::bul(m, at, o)
}


fn jio(b: u32, c: u32, ab: f32, i: u32, ig: f32, kt: f32, 
                  acc: u32, aqw: u32, phl: u32) -> (f32, f32, f32) {
    let bj = b / acc;
    let br = c / aqw;
    let bhi = b % acc;
    let alk = c % aqw;
    
    
    let dpa = bj.cn(phl).hx(2654435761);
    let ffk = (dpa & 0xFFFF) as f32 / 65535.0;
    
    let kjv = ig * (0.7 + ffk * 0.6);
    let kju = ((dpa >> 16) & 0xFFFF) as f32 / 65535.0 * 50.0;
    let acr = 12.0 + ffk * 20.0;
    
    
    let lbr = ((ab * kjv * 8.0 + kju) % ((i / aqw + 40) as f32)) as i32 - 20;
    let mat = br as i32;
    let la = lbr - mat;
    
    if la < 0 || la > acr as i32 {
        return (0.0, 0.0, 0.0);
    }
    
    
    let cuv = la as f32 / acr;
    let hj = if la == 0 {
        kt * 255.0
    } else {
        kt * (1.0 - cuv * cuv) * 180.0
    };
    
    
    let feo = bj.hx(31337) ^ br.hx(7919) ^ phl;
    if !pwk(bhi as f32 / acc as f32, 
                              alk as f32 / aqw as f32, feo) {
        return (hj * 0.05, hj * 0.1, hj * 0.03);
    }
    
    
    let (m, at, o) = if la == 0 {
        (hj * 0.9, hj, hj * 0.9)  
    } else if la < 3 {
        (hj * 0.3, hj, hj * 0.1)  
    } else {
        (0.0, hj, hj * 0.05)  
    };
    
    (m, at, o)
}





use crate::math::{ahn, axv, lz, rk, jdi};


#[inline(always)]
fn srb(b: f32) -> f32 {
    b - (b as i32) as f32
}







pub fn wmc(input: Fy) -> PixelOutput {
    let d = input.z as f32;
    let i = input.ac as f32;
    let ab = input.time;
    
    
    let tm = (input.b as f32 / d - 0.5) * 2.0 * (d / i);  
    let p = (input.c as f32 / i - 0.5) * 2.0;
    
    
    let fef = -3.0;
    let lxg = tm;
    let lxh = p;
    let lxi = 1.5;  
    
    
    let gqh = ahn(lxg * lxg + lxh * lxh + lxi * lxi);
    let vqv = lxg / gqh;
    let vqw = lxh / gqh;
    let vqx = lxi / gqh;
    
    
    let vzs = 0.0;
    let vzt = 0.0;
    let vzu = fef;
    
    
    let mut la = 0.0f32;
    let mut lch = 0u8;  
    let mut lcg = (0.0f32, 0.0f32, 0.0f32);
    
    for ydc in 0..24 {  
        let y = vzs + vqv * la;
        let x = vzt + vqw * la;
        let cbe = vzu + vqx * la;
        
        
        let (pkg, wmk) = hzd(y, x, cbe, ab);
        
        if pkg < 0.02 {  
            lch = wmk;
            lcg = (y, x, cbe);
            break;
        }
        
        la += pkg;
        if la > 20.0 { break; }
    }
    
    
    let (cos, cor, coq) = ukn(input.b, input.c, d, i, ab);
    
    if lch == 0 {
        
        return PixelOutput::bul(cos, cor, coq);
    }
    
    
    
    let btz = (1.0 - (la - 1.0) / 10.0).qp(0.2, 1.0);
    let cpz = btz;
    
    
    let kss = axv(lcg.0) > 0.4;
    let siy = axv(lcg.1) > 0.4;
    let isj = if kss || siy { 0.5 } else { 0.0 };
    
    
    let cer = ((la - 1.0) / 6.0).qp(0.0, 0.6);
    
    
    let (wmm, wmj, wmi) = match lch {
        1 => {  
            let m = (80.0 * cpz + isj * 200.0) as u8;
            let at = (255.0 * cpz + isj * 100.0) as u8;
            let o = (180.0 * cpz + isj * 255.0) as u8;
            (m, at, o)
        },
        2 => {  
            let xg = lz(ab * 3.0) * 0.2 + 0.8;
            let m = (60.0 * cpz * xg) as u8;
            let at = (255.0 * cpz * xg) as u8;
            let o = (100.0 * cpz * xg) as u8;
            (m, at, o)
        },
        _ => (cos, cor, coq)
    };
    
    
    let adh = 1.0 - cer;
    let m = jdi(cos, wmm, adh);
    let at = jdi(cor, wmj, adh);
    let o = jdi(coq, wmi, adh);
    
    
    let arx = if input.c % 3 == 0 { 0.9 } else { 1.0 };
    let m = ((m as f32) * arx) as u8;
    let at = ((at as f32) * arx) as u8;
    let o = ((o as f32) * arx) as u8;
    
    PixelOutput::bul(m, at, o)
}


#[inline(always)]
fn hzd(b: f32, c: f32, av: f32, ab: f32) -> (f32, u8) {
    
    let rro = 2.0 + lz(ab * 0.5) * 0.3;
    
    
    let aev = ab * 0.5;
    let (kb, ix, agv) = wac(b, c, av - rro, 0.0, aev);
    let nhw = wff(kb, ix, agv, 0.6);
    
    
    let wrc = rk(ab * 0.7) * 1.0;
    let wrd = lz(ab * 0.5) * 0.6;
    let wre = 2.5;
    let pmi = wfg(b - wrc, c - wrd, av - wre, 0.4);
    
    
    if nhw < pmi {
        (nhw, 1)
    } else {
        (pmi, 2)
    }
}


#[inline(always)]
fn wfg(b: f32, c: f32, av: f32, m: f32) -> f32 {
    ahn(b * b + c * c + av * av) - m
}


#[inline(always)]
fn wff(b: f32, c: f32, av: f32, e: f32) -> f32 {
    let dx = axv(b) - e;
    let bg = axv(c) - e;
    let pt = axv(av) - e;
    
    let lre = ahn(
        dx.am(0.0) * dx.am(0.0) + 
        bg.am(0.0) * bg.am(0.0) + 
        pt.am(0.0) * pt.am(0.0)
    );
    let dsa = dx.am(bg).am(pt).v(0.0);
    lre + dsa
}


#[inline(always)]
fn zlw(b: f32, c: f32, av: f32, lww: f32, jkz: f32) -> f32 {
    let fm = ahn(b * b + av * av) - lww;
    ahn(fm * fm + c * c) - jkz
}


#[inline(always)]
fn wac(b: f32, c: f32, av: f32, ax: f32, bga: f32) -> (f32, f32, f32) {
    
    let bmo = rk(bga);
    let bol = lz(bga);
    let hy = b * bmo - av * bol;
    let ahc = b * bol + av * bmo;
    
    
    let bmn = rk(ax);
    let bok = lz(ax);
    let jz = c * bmn - ahc * bok;
    let eli = c * bok + ahc * bmn;
    
    (hy, jz, eli)
}


fn zlv(b: f32, c: f32, av: f32, ab: f32) -> (f32, f32, f32) {
    let cel = 0.001;
    let (bc, _) = hzd(b, c, av, ab);
    let (dx, _) = hzd(b + cel, c, av, ab);
    let (bg, _) = hzd(b, c + cel, av, ab);
    let (pt, _) = hzd(b, c, av + cel, ab);
    
    let vt = dx - bc;
    let ahr = bg - bc;
    let arn = pt - bc;
    let len = ahn(vt * vt + ahr * ahr + arn * arn).am(0.0001);
    (vt / len, ahr / len, arn / len)
}


fn isj(b: f32, c: f32, av: f32, ydg: f32) -> f32 {
    let gfx = 0.02;
    let ax = axv(b);
    let bga = axv(c);
    let gzl = axv(av);
    
    
    let uxz = (axv(ax - 0.5) < gfx) && (axv(bga - 0.5) < gfx);
    let uya = (axv(ax - 0.5) < gfx) && (axv(gzl - 0.5) < gfx);
    let uyb = (axv(bga - 0.5) < gfx) && (axv(gzl - 0.5) < gfx);
    
    if uxz || uya || uyb {
        1.0
    } else {
        0.0
    }
}


fn ukn(b: u32, c: u32, d: f32, i: f32, ab: f32) -> (u8, u8, u8) {
    let bj = (b as f32 / 16.0) as u32;
    let br = (c as f32 / 18.0) as u32;
    
    
    let dv = bj.hx(31337) ^ 0xDEAD;
    let sqy = 0.3 + ((dv % 100) as f32 / 100.0) * 0.7;
    let vhj = (dv % 1000) as f32 / 100.0;
    
    
    let itv = (ab * sqy + vhj) % 1.5;
    let jmx = c as f32 / i;
    
    
    let hms = (itv - jmx).gp();
    let jbi = hms < 0.03;
    
    
    let ies = 0.3;
    let odx = jmx < itv && (itv - jmx) < ies;
    let fad = if odx { 1.0 - (itv - jmx) / ies } else { 0.0 };
    
    
    let feo = bj.hx(7919) ^ br.hx(31337);
    let nzg = (feo % 3) != 0;
    
    if jbi && nzg {
        (220, 255, 220)  
    } else if odx && nzg {
        let at = (200.0 * fad) as u8;
        let m = (50.0 * fad) as u8;
        let o = (80.0 * fad) as u8;
        (m, at, o)
    } else {
        (5, 15, 8)  
    }
}





use spin::Mutex;

static Rd: Mutex<VirtualGpu> = Mutex::new(VirtualGpu::new());



pub fn init(framebuffer: *mut u32, z: u32, ac: u32) {
    Rd.lock().init(framebuffer, z, ac, z);
    crate::serial_println!("[VGPU] Initialized {}x{} virtual GPU ({} virtual cores)", 
        z, ac, YJ_);
}


pub fn ttx(framebuffer: *mut u32, z: u32, ac: u32, oq: u32) {
    Rd.lock().init(framebuffer, z, ac, oq);
    crate::serial_println!("[VGPU] Initialized {}x{} stride={} virtual GPU ({} virtual cores)", 
        z, ac, oq, YJ_);
}


pub fn hzy(bfg: Ty) {
    Rd.lock().hzy(bfg);
}







pub fn wma(input: Fy) -> PixelOutput {
    let d = input.z as f32;
    let i = input.ac as f32;
    let ab = input.time;
    
    
    let cx = (input.b as f32 - d * 0.5) / (i * 0.5);
    let ae = (input.c as f32 - i * 0.5) / (i * 0.5);
    
    
    let dy = ahn(cx * cx + ae * ae).am(0.001);
    let hg = itz(ae, cx);
    
    
    let eo = 1.0 / dy;
    
    
    let av = eo + ab * 3.0;
    
    
    
    let oro = 32.0;
    let nen = (hg + 3.14159) / 6.28318;  
    let column = (nen * oro) as u32;
    let odn = (nen * oro) % 1.0;  
    
    
    let dpa = column.hx(48271);
    let kju = (dpa % 1000) as f32 / 1000.0 * 10.0;
    
    
    let nzh = 0.15;
    let jzo = av + kju;
    let tgk = (jzo / nzh) as u32;
    let hls = (jzo / nzh) % 1.0;
    
    
    
    let rls = axv(odn - 0.5);
    let tgl = hls > 0.2 && hls < 0.8;
    
    
    let byt = 0.3 + dy * 0.2;
    let uxx = rls < byt;
    
    
    let amg = tgk.hx(31337) ^ column.hx(48271);
    let tgj = ukl(odn, hls, amg);
    
    
    
    let cer = (dy * 1.8).v(1.0);
    
    
    let jbi = hls < 0.25;
    let fad = if jbi { 1.0 } else { 1.0 - (hls - 0.25) / 0.6 };
    
    
    let ys = if input.c % 2 == 0 { 0.9 } else { 1.0 };
    
    
    if uxx && tgl && tgj {
        
        let hj = cer * fad * ys;
        
        if jbi {
            
            let xg = lz(ab * 8.0 + jzo * 4.0) * 0.2 + 0.8;
            let m = (200.0 * hj * xg) as u8;
            let at = (255.0 * hj) as u8;
            let o = (220.0 * hj * xg) as u8;
            PixelOutput::bul(m, at, o)
        } else {
            
            let m = (40.0 * hj) as u8;
            let at = (255.0 * hj * fad) as u8;
            let o = (80.0 * hj) as u8;
            PixelOutput::bul(m, at, o)
        }
    } else {
        
        let qpf = (cer * 0.05 * ys) as f32;
        let ei = (qpf * 40.0) as u8;
        PixelOutput::bul(0, ei, ei / 2)
    }
}


#[inline]
fn ukl(mj: f32, ct: f32, dv: u32) -> bool {
    let y = (mj * 6.0) as u32;
    let x = (ct * 8.0) as u32;
    let pattern = dv % 10;
    
    match pattern {
        0 => y > 1 && y < 5,                              
        1 => x == 2 || x == 5,                            
        2 => (y + x) % 2 == 0,                            
        3 => y == 3 || x == 4,                            
        4 => x > 1 && x < 7 && y > 1 && y < 5,          
        5 => (y == 2 || y == 4) && x > 1 && x < 7,      
        6 => x == 3 || (y == 3 && x > 1 && x < 7),      
        7 => (y + x / 2) % 3 == 0,                        
        8 => x < 4 && y > 1 && y < 5,                    
        _ => (dv.hx(y + 1) ^ (x + 1)) % 3 == 0, 
    }
}


pub fn po() {
    Rd.lock().nlu();
}


#[cfg(target_arch = "x86_64")]
pub fn krk() {
    Rd.lock().ryh();
}


pub fn or(koy: u32) {
    Rd.lock().or(koy);
}


pub fn frame() -> u32 {
    Rd.lock().frame()
}


pub fn time() -> f32 {
    Rd.lock().time()
}








pub fn wlt(input: Fy) -> PixelOutput {
    let d = input.z as f32;
    let i = input.ac as f32;
    let ab = input.time;
    
    
    let fqb = (input.b as f32 * 2.0 - d) / i;
    let cth = (input.c as f32 * 2.0 - i) / i;
    
    let kqn = fqb * fqb + cth * cth;
    let dm = axv(0.7 - kqn);
    
    let e = (1.0 - dm) * 5.0;
    let mut fp = fqb * e;
    let mut iz = cth * e;
    
    let mut htk: f32 = 0.0;
    let mut htj: f32 = 0.0;
    let mut hti: f32 = 0.0;
    
    
    let mut a: f32 = 1.0;
    while a <= 6.0 {
        let hok = 1.0 / a;
        fp += rk(iz * a + ab) * hok + 0.7;
        iz += rk(fp * a + a + ab) * hok + 0.7;
        
        let wz = axv(fp - iz) * 0.2;
        htk += (lz(fp) + 1.0) * wz;
        htj += (lz(iz) + 1.0) * wz;
        hti += (lz(iz) + 1.0) * wz;
        a += 1.0;
    }
    
    
    let duu = cxr(-4.0 * dm);
    let ksh = cxr(cth);
    let ksi = cxr(-cth);
    let ksj = cxr(cth * -2.0);
    
    let xb = fii(ksh * duu / (htk + 0.001));
    let lp = fii(ksi * duu / (htj + 0.001));
    let pq = fii(ksj * duu / (hti + 0.001));
    
    let m = (axv(xb) * 255.0).v(255.0) as u8;
    let at = (axv(lp) * 255.0).v(255.0) as u8;
    let o = (axv(pq) * 255.0).v(255.0) as u8;
    PixelOutput::bul(m, at, o)
}


#[inline(always)]
fn fii(b: f32) -> f32 {
    let hy = b * b;
    b / (1.0 + b.gp() + hy * 0.28)
}


#[inline(always)]
fn cxr(b: f32) -> f32 {
    let b = b.qp(-10.0, 10.0);
    let ab = 1.0 + b / 256.0;
    let mut m = ab;
    
    m = m * m; m = m * m; m = m * m; m = m * m;
    m = m * m; m = m * m; m = m * m; m = m * m;
    m
}


pub fn kyx(j: &str) -> Option<Ty> {
    match j.aqn().as_str() {
        "plasma" => Some(wmd),
        "matrix" | "rain" => Some(wlz),
        "mandelbrot" | "fractal" => Some(wly),
        "gradient" | "test" => Some(wlv),
        "fire" => Some(wlu),
        "tunnel" | "holotunnel" | "3d" => Some(wlx),
        "parallax" | "holoparallax" | "depth" => Some(wlw),
        "shapes" | "objects" | "cubes" | "matrix3dshapes" => Some(wmc),
        "rain3d" | "matrix3d" | "matrixrain3d" | "fly" => Some(wma),
        "cosmic" | "deform" | "vortex" | "complex" => Some(wlt),
        _ => None,
    }
}


pub fn zbb() -> &'static [&'static str] {
    &["plasma", "matrix", "mandelbrot", "gradient", "fire", "tunnel", "parallax", "shapes", "rain3d", "cosmic"]
}






















const Xn: usize = 8;

const OE_: usize = 240;

const CGK_: usize = 150;

const Act: usize = 4;

const BBC_: usize = 10;
const OJ_: usize = 45;

const CGQ_: usize = 64;


static CGM_: [u8; 64] = [
    255, 250, 244, 238, 232, 225, 218, 211,
    204, 196, 189, 181, 174, 166, 158, 150,
    143, 135, 128, 121, 114, 107, 100,  94,
     88,  82,  76,  71,  66,  61,  56,  52,
     48,  44,  40,  37,  34,  31,  28,  26,
     24,  22,  20,  18,  16,  15,  14,  13,
     12,  11,  10,   9,   8,   7,   6,   5,
      5,   4,   4,   3,   3,   3,   2,   2,
];



const fn tbc() -> [u32; 256] {
    let mut djf = [0xFF010201u32; 256]; 
    let mut a = 1u32;
    while a < 256 {
        let r = if a > 250 {
            
            let d = 200 + ((a - 250) * 10) as u32;
            let d = if d > 255 { 255 } else { d };
            (0xFF << 24) | (d << 16) | (255 << 8) | d
        } else if a > 200 {
            
            let bb = a - 200;
            let m = bb * 3 / 2;
            let at = 200 + bb;
            let o = bb / 2;
            (0xFF << 24) | (m << 16) | (at << 8) | o
        } else if a > 140 {
            
            let at = 130 + (a - 140) * 7 / 6;
            let m = (a - 140) / 6;
            let o = (a - 140) / 8;
            (0xFF << 24) | (m << 16) | (at << 8) | o
        } else if a > 80 {
            
            let at = 60 + (a - 80) * 7 / 6;
            let o = (a - 80) / 4;
            (0xFF << 24) | (at << 8) | o
        } else if a > 30 {
            
            let at = 20 + (a - 30) * 4 / 5;
            (0xFF << 24) | (at << 8)
        } else if a > 10 {
            
            let at = 6 + (a - 10) * 7 / 10;
            (0xFF << 24) | (at << 8)
        } else {
            
            let at = 2 + a / 2;
            (0xFF << 24) | (at << 8)
        };
        djf[a as usize] = r;
        a += 1;
    }
    djf
}

static CFZ_: [u32; 256] = tbc();


#[inline(always)]
fn oml(hj: u8) -> u32 {
    CFZ_[hj as usize]
}



const fn tbd() -> [u32; 256] {
    let mut djf = [0xFF010201u32; 256];
    let mut a = 1u32;
    while a < 256 {
        
        let r = if a > 240 {
            
            let d = 220 + ((a - 240) * 2);
            let d = if d > 255 { 255 } else { d };
            (0xFF << 24) | (d << 16) | (255 << 8) | d
        } else if a > 180 {
            
            let bb = a - 180;
            let m = 100 + bb;
            let at = 200 + bb / 2;
            let o = 140 + bb;
            let m = if m > 255 { 255 } else { m };
            let at = if at > 255 { 255 } else { at };
            let o = if o > 255 { 255 } else { o };
            (0xFF << 24) | (m << 16) | (at << 8) | o
        } else if a > 120 {
            
            let bb = a - 120;
            let m = 30 + bb / 2;
            let at = 130 + bb;
            let o = 60 + bb;
            (0xFF << 24) | (m << 16) | (at << 8) | o
        } else if a > 60 {
            
            let bb = a - 60;
            let at = 60 + bb;
            let o = 30 + bb / 2;
            let m = bb / 4;
            (0xFF << 24) | (m << 16) | (at << 8) | o
        } else if a > 20 {
            
            let at = 20 + (a - 20);
            let o = 10 + (a - 20) / 2;
            (0xFF << 24) | (at << 8) | o
        } else {
            
            let at = 4 + a / 2;
            let o = 2 + a / 3;
            (0xFF << 24) | (at << 8) | o
        };
        djf[a as usize] = r;
        a += 1;
    }
    djf
}

static CGJ_: [u32; 256] = tbd();


#[inline(always)]
fn uod(hj: u8) -> u32 {
    CGJ_[hj as usize]
}


#[derive(Clone, Copy)]
struct MDrop {
    c: i16,              
    ig: u8,           
    va: u8,         
    acr: u8,       
    amg: u32,     
    gh: bool,
    
    
    glu: u64,          
    glt: [u8; 48],   
}

impl MDrop {
    const fn new() -> Self {
        Self {
            c: -100, ig: 2, va: 0, acr: 20,
            amg: 0, gh: false,
            glu: 0, glt: [0u8; 48],
        }
    }

    
    #[inline(always)]
    fn kzh(&self, aaz: usize) -> usize {
        if aaz < 48 && (self.glu >> aaz) & 1 != 0 {
            self.glt[aaz] as usize
        } else {
            let ckx = self.amg.cn(aaz as u32 * 2654435761);
            (ckx % CGQ_ as u32) as usize
        }
    }

    
    #[inline(always)]
    fn ogg(&self, aaz: usize) -> bool {
        aaz < 48 && (self.glu >> aaz) & 1 != 0
    }
}


pub struct ShaderMatrixState {
    agk: [[MDrop; Act]; OE_],
    cwh: [u8; OE_],   
    ajg: usize,
    bnr: usize,
    frame: u32,
    rng: u32,
    jr: bool,
}

impl ShaderMatrixState {
    pub const fn new() -> Self {
        Self {
            agk: [[MDrop::new(); Act]; OE_],
            cwh: [128u8; OE_],
            ajg: 0,
            bnr: 0,
            frame: 0,
            rng: 0xDEADBEEF,
            jr: false,
        }
    }

    
    pub fn init(&mut self, wf: usize, aav: usize) {
        self.ajg = (wf / Xn).v(OE_);
        self.bnr = (aav / Xn).v(CGK_);
        self.frame = 0;
        self.rng = 0xDEADBEEF;

        
        for bj in 0..self.ajg {
            self.rng = self.rng.hx(1103515245).cn(12345);
            
            let pattern = ((bj * 17 + 53) % 97) as i32 - 48; 
            let lxa = (self.rng % 100) as i32 - 50;          
            let eo = (145i32 + pattern + lxa).qp(20, 255) as u8;
            self.cwh[bj] = eo;
        }

        
        for bj in 0..self.ajg {
            let eo = self.cwh[bj];
            let eat = eo as u32; 

            let mut foz: i32 = 0;
            for di in 0..Act {
                self.rng = self.rng.hx(1103515245).cn(12345);

                
                let foi = BBC_ as u32 + eat / 8;           
                let llf = (OJ_ as u32).v(foi + 20);  
                let ase = foi + (self.rng % (llf - foi + 1));

                self.rng = self.rng.hx(1103515245).cn(12345);

                
                let tae = 3u32.ao(eat / 128);    
                let hkw = 2 + (255 - eat) / 50;             
                let qi = tae + (self.rng % hkw);

                let vc = foz - (self.rng % 6) as i32;
                foz = vc - ase as i32 - qi as i32;

                self.rng = self.rng.hx(1103515245).cn(12345);

                
                let fvc = 1 + (255u32.ao(eat)) / 128; 
                let fvd = 1 + (255u32.ao(eat)) / 80; 
                let ig = fvc + (self.rng % fvd);

                self.rng = self.rng.hx(1103515245).cn(12345);

                self.agk[bj][di] = MDrop {
                    c: vc as i16,
                    ig: ig.v(8) as u8,
                    va: (self.rng % ig) as u8,
                    acr: ase.v(OJ_ as u32) as u8,
                    amg: self.rng,
                    gh: true,
                    glu: 0,
                    glt: [0u8; 48],
                };
            }
        }

        self.jr = true;
    }

    
    pub fn qs(&mut self) {
        self.frame = self.frame.cn(1);
        let csl = self.bnr as i32 + OJ_ as i32 + 10;

        for bj in 0..self.ajg {
            let eo = self.cwh[bj];
            let eat = eo as u32;

            for di in 0..Act {
                let drop = &mut self.agk[bj][di];
                if !drop.gh { continue; }

                
                drop.va = drop.va.cn(1);
                if drop.va >= drop.ig {
                    drop.va = 0;
                    drop.c += 1;
                    
                    drop.amg = drop.amg.hx(1103515245).cn(12345);

                    
                    
                    
                    
                    let mlf = drop.acr as usize;
                    if mlf >= 2 && mlf <= 48 {
                        let mut oxh = drop.kzh(0) as u8;
                        for aaz in 1..mlf {
                            let ipw = drop.kzh(aaz) as u8;
                            if ipw == oxh {
                                
                                drop.glu |= (1u64 << (aaz - 1)) | (1u64 << aaz);
                                drop.glt[aaz - 1] = ipw;
                                drop.glt[aaz] = ipw;
                            }
                            oxh = ipw;
                        }
                    }
                }

                
                if drop.c as i32 > csl {
                    self.rng = self.rng.hx(1103515245).cn(12345);

                    let foi = BBC_ as u32 + eat / 8;
                    let llf = (OJ_ as u32).v(foi + 20);
                    let ase = foi + (self.rng % (llf - foi + 1));

                    self.rng = self.rng.hx(1103515245).cn(12345);
                    let qi = 2 + (self.rng % 6);
                    let bhn = -(ase as i32) - qi as i32 - (self.rng % 8) as i32;

                    self.rng = self.rng.hx(1103515245).cn(12345);
                    let fvc = 1 + (255u32.ao(eat)) / 128;
                    let fvd = 1 + (255u32.ao(eat)) / 80;
                    let ig = fvc + (self.rng % fvd);

                    self.rng = self.rng.hx(1103515245).cn(12345);
                    drop.c = bhn as i16;
                    drop.ig = ig.v(8) as u8;
                    drop.va = 0;
                    drop.acr = ase.v(OJ_ as u32) as u8;
                    drop.amg = self.rng;
                    
                    drop.glu = 0;
                    drop.glt = [0u8; 48];
                }
            }
        }
    }
}



#[repr(C)]
struct Acv {
    g: *const ShaderMatrixState,
    pq: *mut u32,
    lu: usize,
    qh: usize,
}

unsafe impl Send for Acv {}
unsafe impl Sync for Acv {}



fn ukm(ay: usize, ci: usize, f: *mut u8) {
    let be = unsafe { &*(f as *const Acv) };
    let g = unsafe { &*be.g };
    let pq = be.pq;
    let ua = be.lu;
    let iuj = be.qh;
    let bnr = g.bnr;

    
    let cqz = &crate::matrix_fast::CFC_;

    for bj in ay..ci {
        let eo = g.cwh[bj] as u32;
        
        let hfu = 100 + (eo * 155 / 255);

        for di in 0..Act {
            let drop = &g.agk[bj][di];
            if !drop.gh { continue; }

            let buu = drop.c as i32;
            let acr = drop.acr as usize;

            for aaz in 0..acr {
                let bmg = buu - aaz as i32;
                if bmg < 0 || bmg >= bnr as i32 { continue; }

                
                let uiu = (aaz * 63) / acr.am(1);
                let qnm = CGM_[uiu.v(63)] as u32;
                let mut hj = ((qnm * hfu) / 255).v(255) as u8;
                if hj < 2 { continue; }

                
                let caq = drop.ogg(aaz);
                let cqy = drop.kzh(aaz);
                let ka = &cqz[cqy];

                
                
                
                
                let s = if aaz == 0 {
                    oml(hj.am(250))
                } else if caq {
                    
                    hj = hj.akq(60).v(255);
                    uod(hj)
                } else {
                    oml(hj)
                };

                
                let y = bj * Xn + 1;
                let x = bmg as usize * Xn + 1;

                
                if x + 6 <= iuj && y + 6 <= ua {
                    
                    for br in 0..6 {
                        let fs = ka[br];
                        if fs == 0 { continue; }
                        let dvh = (x + br) * ua + y;
                        unsafe {
                            if fs & 0b000001 != 0 { *pq.add(dvh)     = s; }
                            if fs & 0b000010 != 0 { *pq.add(dvh + 1) = s; }
                            if fs & 0b000100 != 0 { *pq.add(dvh + 2) = s; }
                            if fs & 0b001000 != 0 { *pq.add(dvh + 3) = s; }
                            if fs & 0b010000 != 0 { *pq.add(dvh + 4) = s; }
                            if fs & 0b100000 != 0 { *pq.add(dvh + 5) = s; }
                        }
                    }
                }
            }
        }
    }
}



static CSQ_: spin::Mutex<ShaderMatrixState> =
    spin::Mutex::new(ShaderMatrixState::new());










pub fn wmb(pq: *mut u32, z: usize, ac: usize) {
    let mut g = CSQ_.lock();

    
    if !g.jr || g.ajg != z / Xn || g.bnr != ac / Xn {
        g.init(z, ac);
    }

    
    g.qs();

    
    let jtv = z * ac;
    unsafe {
        #[cfg(target_arch = "x86_64")]
        crate::graphics::simd::bed(pq, jtv, 0xFF010201);
        #[cfg(not(target_arch = "x86_64"))]
        {
            for a in 0..jtv {
                *pq.add(a) = 0xFF010201u32;
            }
        }
    }

    
    let ajg = g.ajg;
    let be = Acv {
        g: &*g as *const ShaderMatrixState,
        pq,
        lu: z,
        qh: ac,
    };

    crate::cpu::smp::daj(
        ajg,
        ukm,
        &be as *const Acv as *mut u8,
    );
}