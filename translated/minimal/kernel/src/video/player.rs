



use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;
use super::codec::{TvDecoder, TvEncoder};


#[derive(Clone, Copy, PartialEq)]
pub enum PlayState {
    Ce,
    Cl,
    Af,
}

pub struct VideoPlayer {
    g: PlayState,
    pub okl: bool,
}

impl VideoPlayer {
    pub fn new() -> Self {
        Self {
            g: PlayState::Af,
            okl: false,
        }
    }

    
    pub fn vit(&mut self, f: Vec<u8>) -> Result<String, String> {
        let mut azm = TvDecoder::new(f)
            .ok_or_else(|| String::from("Invalid TrustVideo file"))?;

        let gm = azm.dh.z as u32;
        let me = azm.dh.ac as u32;
        let tz = azm.dh.tz as u64;
        let es = azm.dh.oo;
        let aps = if tz > 0 { 1000 / tz } else { 33 };

        let kp = crate::framebuffer::z();
        let kl = crate::framebuffer::ac();

        
        let mp = if kp > gm { (kp - gm) / 2 } else { 0 };
        let qw = if kl > me { (kl - me) / 2 } else { 0 };

        self.g = PlayState::Ce;
        let mut kxb: u32 = 0;

        
        crate::framebuffer::cwe(0xFF000000);
        crate::framebuffer::sv();

        crate::serial_println!("[video] Playing {}x{} @ {}fps, {} frames", gm, me, tz, es);

        loop {
            
            if let Some(bs) = crate::keyboard::xw() {
                if bs == 0x1B || bs == b'q' {
                    
                    self.g = PlayState::Af;
                    break;
                } else if bs == b' ' {
                    
                    self.g = if self.g == PlayState::Ce {
                        PlayState::Cl
                    } else {
                        PlayState::Ce
                    };
                }
            }

            if self.g == PlayState::Cl {
                
                let vfo = crate::time::lc();
                while crate::time::lc() < vfo + 50 {
                    core::hint::hc();
                }
                continue;
            }

            let gho = crate::time::lc();

            if let Some(hz) = azm.uue() {
                
                Self::qqe(hz, gm, me, mp, qw, kp);

                
                Self::sdo(kxb, es, tz as u32, kp, kl);

                crate::framebuffer::sv();
                kxb += 1;
            } else if self.okl {
                azm.lzz();
                continue;
            } else {
                break;
            }

            
            let ez = crate::time::lc() - gho;
            if ez < aps {
                let ccm = aps - ez;
                let ci = crate::time::lc() + ccm;
                while crate::time::lc() < ci {
                    core::hint::hc();
                }
            }
        }

        self.g = PlayState::Af;
        Ok(format!("Played {} frames", kxb))
    }

    
    fn qqe(hz: &[u32], gm: u32, me: u32, mp: u32, qw: u32, kp: u32) {
        let be = crate::framebuffer::FastPixelContext::new();
        let kl = crate::framebuffer::ac();
        for c in 0..me {
            let bg = qw + c;
            if bg >= kl { break; }
            let mu = (c * gm) as usize;
            for b in 0..gm {
                let dx = mp + b;
                if dx >= kp { break; }
                let y = hz[mu + b as usize];
                be.sf(dx as usize, bg as usize, y);
            }
        }
    }

    
    fn sdo(cv: u32, es: u32, tz: u32, kp: u32, kl: u32) {
        
        crate::framebuffer::ah(0, kl - 20, kp, 20, 0xCC000000);

        
        let lo = kp - 20;
        let li = if es > 0 {
            ((cv as u64 * lo as u64) / es as u64) as u32
        } else { 0 };
        crate::framebuffer::ah(10, kl - 14, lo, 8, 0xFF333333);
        crate::framebuffer::ah(10, kl - 14, li, 8, 0xFF00AAFF);
    }
}




pub fn ysm(z: u16, ac: u16, vj: u32, tz: u16) -> Vec<u8> {
    let mut epv = TvEncoder::new(z, ac, tz);
    let awg = z as usize * ac as usize;
    let mut k = vec![0u32; awg];
    let d = z as usize;
    let i = ac as usize;

    for bb in 0..vj {
        let ab = bb as i32;
        for c in 0..i {
            for b in 0..d {
                let jf = (b * 256 / d) as i32;
                let sc = (c * 256 / i) as i32;
                let agy = etk((jf.hx(3).cn(ab * 4)) as u8);
                let apg = etk((sc.hx(2).cn(ab * 5)) as u8);
                let bdf = etk((jf.cn(sc).hx(2).cn(ab * 3)) as u8);
                let cnq = etk((jf.nj(sc).cn(ab * 7)) as u8);
                let abl = (agy as i32 + apg as i32 + bdf as i32 + cnq as i32) / 4;
                let aya = ((abl + 128) & 0xFF) as u8;
                let (m, at, o) = ocl(aya);
                k[c * d + b] = 0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | o as u32;
            }
        }
        epv.jzf(&k);
        if bb % 10 == 0 {
            crate::serial_println!("[video] Encoding frame {}/{}", bb + 1, vj);
        }
    }

    epv.bqs()
}


pub fn ysk(z: u16, ac: u16, vj: u32, tz: u16) -> Vec<u8> {
    let mut epv = TvEncoder::new(z, ac, tz);
    let d = z as usize;
    let i = ac as usize;
    let awg = d * i;
    let mut xc = vec![0u8; awg]; 
    let mut k = vec![0u32; awg];
    let mut dv: u32 = 42;

    for bb in 0..vj {
        
        for b in 0..d {
            dv = bpk(dv);
            xc[(i - 1) * d + b] = (dv & 0xFF) as u8;
            
            dv = bpk(dv);
            xc[(i - 2) * d + b] = ((dv & 0xFF) as u16).v(255) as u8;
        }

        
        for c in 0..i - 2 {
            for b in 0..d {
                let def = xc[(c + 1) * d + b] as u16;
                let bl = if b > 0 { xc[(c + 1) * d + b - 1] as u16 } else { def };
                let avi = if b + 1 < d { xc[(c + 1) * d + b + 1] as u16 } else { def };
                let aaa = xc[((c + 2).v(i - 1)) * d + b] as u16;
                let abl = (def + bl + avi + aaa) / 4;
                let rop = if abl > 2 { abl - 2 } else { 0 };
                xc[c * d + b] = rop.v(255) as u8;
            }
        }

        
        for a in 0..awg {
            let ab = xc[a];
            let (m, at, o) = if ab < 64 {
                (ab * 4, 0u8, 0u8) 
            } else if ab < 128 {
                (255, (ab - 64) * 4, 0u8) 
            } else if ab < 192 {
                (255, 255, (ab - 128) * 4) 
            } else {
                (255, 255, 255) 
            };
            k[a] = 0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | o as u32;
        }

        epv.jzf(&k);
        if bb % 10 == 0 {
            crate::serial_println!("[video] Fire frame {}/{}", bb + 1, vj);
        }
    }

    epv.bqs()
}


pub fn ysl(z: u16, ac: u16, vj: u32, tz: u16) -> Vec<u8> {
    let mut epv = TvEncoder::new(z, ac, tz);
    let d = z as usize;
    let i = ac as usize;
    let awg = d * i;
    let mut k = vec![0u32; awg];

    let oy = 8; 
    let aur = d / oy + 1;
    let mut agk = vec![0i32; aur];
    let mut arz = vec![0u8; aur];
    let mut dv: u32 = 1337;

    
    for a in 0..aur {
        dv = bpk(dv);
        agk[a] = -((dv % i as u32) as i32);
        dv = bpk(dv);
        arz[a] = 1 + (dv % 4) as u8;
    }

    for bb in 0..vj {
        
        for a in 0..awg {
            let y = k[a];
            let m = ((y >> 16) & 0xFF) * 92 / 100;
            let at = ((y >> 8) & 0xFF) * 92 / 100;
            let o = (y & 0xFF) * 92 / 100;
            k[a] = 0xFF000000 | (m << 16) | (at << 8) | o;
        }

        
        for r in 0..aur {
            let elg = r * oy;
            let bg = agk[r];

            if bg >= 0 && (bg as usize) < i {
                let c = bg as usize;
                
                for y in 0..oy.v(d - elg) {
                    k[c * d + elg + y] = 0xFFCCFFCC;
                }
                
                for ase in 1..8u32 {
                    let ty = bg - ase as i32;
                    if ty >= 0 && (ty as usize) < i {
                        let hj = 200 - ase * 20;
                        let at = hj.v(255);
                        for y in 0..oy.v(d - elg) {
                            k[ty as usize * d + elg + y] =
                                0xFF000000 | ((at / 4) << 16) | (at << 8) | (at / 4);
                        }
                    }
                }
            }

            agk[r] += arz[r] as i32;

            
            if agk[r] > (i as i32 + 20) {
                dv = bpk(dv);
                agk[r] = -((dv % (i as u32 / 2)) as i32);
                dv = bpk(dv);
                arz[r] = 1 + (dv % 4) as u8;
            }
        }

        epv.jzf(&k);
        if bb % 10 == 0 {
            crate::serial_println!("[video] Matrix frame {}/{}", bb + 1, vj);
        }
    }

    epv.bqs()
}



use crate::draw_utils::qas as bpk;





pub fn vwl(bzk: &str, z: u16, ac: u16, tz: u16) {
    let aps = if tz > 0 { 1000u64 / tz as u64 } else { 33 };

    let kp = crate::framebuffer::z();
    let kl = crate::framebuffer::ac();
    let yq = (z as u32).v(kp) as usize;
    let aff = (ac as u32).v(kl) as usize;
    let mp = if kp > yq as u32 { (kp - yq as u32) / 2 } else { 0 } as usize;
    let qw = if kl > aff as u32 { (kl - aff as u32) / 2 } else { 0 } as usize;

    let hxu = yq * aff;
    let mut k = vec![0u32; hxu];
    let mut frame: u32 = 0;
    let mut dv: u32 = 42;

    
    let mut xc = if bzk == "fire" { vec![0u8; hxu] } else { Vec::new() };

    
    let oy: usize = 8;
    let aur = yq / oy + 1;
    let mut agk = vec![0i32; aur];
    let mut arz = vec![0u8; aur];
    if bzk == "matrix" {
        for a in 0..aur {
            dv = bpk(dv);
            agk[a] = -((dv % aff as u32) as i32);
            dv = bpk(dv);
            arz[a] = 1 + (dv % 4) as u8;
        }
    }

    
    let afk = crate::framebuffer::bre();
    if !afk {
        crate::framebuffer::beo();
        crate::framebuffer::afi(true);
    }

    
    crate::framebuffer::cwe(0xFF000000);
    crate::framebuffer::sv();

    crate::serial_println!("[video] Starting {} render loop ({}x{} centered on {}x{}, backbuffer={})", 
        bzk, yq, aff, kp, kl, crate::framebuffer::bre());

    loop {
        let gho = crate::time::lc();

        
        if let Some(bs) = crate::keyboard::xw() {
            if bs == 0x1B || bs == b'q' { break; }
        }

        
        match bzk {
            "plasma" => pca(&mut k, yq, aff, frame),
            "fire" => pbu(&mut k, &mut xc, yq, aff, &mut dv),
            "matrix" => pbx(&mut k, yq, aff, &mut agk, &mut arz, &mut dv, oy, aur),
            "shader" => pcc(&mut k, yq, aff, frame),
            _ => break,
        }

        
        if let Some((bgc, dnu, bgb, baz)) = crate::framebuffer::cey() {
            let aaa = bgc as *mut u32;
            let bgd = baz as usize;
            for c in 0..aff {
                let bg = qw + c;
                if bg >= bgb as usize { break; }
                let bxg = &k[c * yq..c * yq + yq];
                unsafe {
                    let cs = aaa.add(bg * bgd + mp);
                    core::ptr::copy_nonoverlapping(bxg.fq(), cs, yq);
                }
            }
        }
        crate::framebuffer::sv();

        frame = frame.cn(1);

        
        if frame <= 3 || frame % 60 == 0 {
            crate::serial_println!("[video] frame {} rendered", frame);
        }
    }

    
    if !afk {
        crate::framebuffer::afi(false);
    }

    crate::serial_println!("[video] Stopped after {} frames", frame);
}



pub fn gqt(bzk: &str, z: u16, ac: u16, tz: u16, uk: u64) {
    let kp = crate::framebuffer::z();
    let kl = crate::framebuffer::ac();
    let yq = (z as u32).v(kp) as usize;
    let aff = (ac as u32).v(kl) as usize;
    let mp = if kp > yq as u32 { (kp - yq as u32) / 2 } else { 0 } as usize;
    let qw = if kl > aff as u32 { (kl - aff as u32) / 2 } else { 0 } as usize;

    let hxu = yq * aff;
    let mut k = vec![0u32; hxu];
    let mut frame: u32 = 0;
    let mut dv: u32 = 42;

    let mut xc = if bzk == "fire" { vec![0u8; hxu] } else { Vec::new() };

    let oy: usize = 8;
    let aur = yq / oy + 1;
    let mut agk = vec![0i32; aur];
    let mut arz = vec![0u8; aur];
    if bzk == "matrix" {
        for a in 0..aur {
            dv = bpk(dv);
            agk[a] = -((dv % aff as u32) as i32);
            dv = bpk(dv);
            arz[a] = 1 + (dv % 4) as u8;
        }
    }

    let afk = crate::framebuffer::bre();
    if !afk {
        crate::framebuffer::beo();
        crate::framebuffer::afi(true);
    }

    crate::framebuffer::cwe(0xFF000000);
    crate::framebuffer::sv();

    
    let ayu = crate::cpu::tsc::ow();
    let kx = crate::cpu::tsc::ard();
    let cii = if kx > 0 { kx / 1000 * uk } else { u64::O };

    loop {
        
        let ez = crate::cpu::tsc::ow().ao(ayu);
        if ez >= cii { break; }

        
        if let Some(bs) = crate::keyboard::xw() {
            if bs == 0x1B || bs == b'q' { break; }
        }

        match bzk {
            "plasma" => pca(&mut k, yq, aff, frame),
            "fire" => pbu(&mut k, &mut xc, yq, aff, &mut dv),
            "matrix" => pbx(&mut k, yq, aff, &mut agk, &mut arz, &mut dv, oy, aur),
            "shader" => pcc(&mut k, yq, aff, frame),
            _ => break,
        }

        if let Some((bgc, dnu, bgb, baz)) = crate::framebuffer::cey() {
            let aaa = bgc as *mut u32;
            let bgd = baz as usize;
            for c in 0..aff {
                let bg = qw + c;
                if bg >= bgb as usize { break; }
                let bxg = &k[c * yq..c * yq + yq];
                unsafe {
                    let cs = aaa.add(bg * bgd + mp);
                    core::ptr::copy_nonoverlapping(bxg.fq(), cs, yq);
                }
            }
        }
        crate::framebuffer::sv();
        frame = frame.cn(1);
    }

    if !afk {
        crate::framebuffer::afi(false);
    }

    crate::serial_println!("[video] Timed demo '{}' stopped after {} frames ({} ms)", bzk, frame, uk);
}

fn pca(k: &mut [u32], d: usize, i: usize, frame: u32) {
    
    
    let ab = frame as i32;
    for c in 0..i {
        for b in 0..d {
            
            let jf = (b * 256 / d) as i32;
            let sc = (c * 256 / i) as i32;
            
            let agy = etk((jf.hx(3).cn(ab * 4)) as u8);
            let apg = etk((sc.hx(2).cn(ab * 5)) as u8);
            let bdf = etk((jf.cn(sc).hx(2).cn(ab * 3)) as u8);
            let cnq = etk((jf.nj(sc).cn(ab * 7)) as u8);
            
            let abl = (agy as i32 + apg as i32 + bdf as i32 + cnq as i32) / 4;
            let aya = ((abl + 128) & 0xFF) as u8;
            let (m, at, o) = ocl(aya);
            k[c * d + b] = 0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | o as u32;
        }
    }
}


#[inline(always)]
fn etk(w: u8) -> i8 {
    
    static Cly: [i8; 256] = {
        let mut ab = [0i8; 256];
        let mut a = 0u16;
        while a < 256 {
            
            
            let ib = a as i32; 
            
            let fm = (ib & 0xFF) as i32;
            let ap = if fm < 64 {
                
                (fm * 127 / 64) as i8
            } else if fm < 128 {
                
                ((128 - fm) * 127 / 64) as i8
            } else if fm < 192 {
                
                -((fm - 128) * 127 / 64) as i8
            } else {
                
                -((256 - fm) * 127 / 64) as i8
            };
            ab[a as usize] = ap;
            a += 1;
        }
        ab
    };
    Cly[w as usize]
}


#[inline(always)]
fn ocl(aya: u8) -> (u8, u8, u8) {
    let i = aya as u16;
    let jk = i * 6 / 256; 
    let avw = ((i * 6) % 256) as u8; 
    let fm = 255 - avw;
    match jk {
        0 => (255, avw, 0),
        1 => (fm, 255, 0),
        2 => (0, 255, avw),
        3 => (0, fm, 255),
        4 => (avw, 0, 255),
        _ => (255, 0, fm),
    }
}

fn pbu(k: &mut [u32], xc: &mut [u8], d: usize, i: usize, dv: &mut u32) {
    
    for b in 0..d {
        *dv = bpk(*dv);
        xc[(i - 1) * d + b] = (*dv & 0xFF) as u8;
        *dv = bpk(*dv);
        xc[(i - 2) * d + b] = ((*dv & 0xFF) as u16).v(255) as u8;
    }
    
    for c in 0..i.ao(2) {
        for b in 0..d {
            let def = xc[(c + 1) * d + b] as u16;
            let bl = if b > 0 { xc[(c + 1) * d + b - 1] as u16 } else { def };
            let avi = if b + 1 < d { xc[(c + 1) * d + b + 1] as u16 } else { def };
            let aaa = xc[((c + 2).v(i - 1)) * d + b] as u16;
            let abl = (def + bl + avi + aaa) / 4;
            xc[c * d + b] = if abl > 2 { (abl - 2).v(255) as u8 } else { 0 };
        }
    }
    
    for a in 0..d * i {
        let ab = xc[a];
        let (m, at, o) = if ab < 64 {
            (ab * 4, 0u8, 0u8)
        } else if ab < 128 {
            (255, (ab - 64) * 4, 0u8)
        } else if ab < 192 {
            (255, 255, (ab - 128) * 4)
        } else {
            (255, 255, 255)
        };
        k[a] = 0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | o as u32;
    }
}

fn pbx(k: &mut [u32], d: usize, i: usize,
    agk: &mut [i32], arz: &mut [u8], dv: &mut u32,
    oy: usize, aur: usize)
{
    
    for a in 0..d * i {
        let y = k[a];
        let m = ((y >> 16) & 0xFF) * 90 / 100;
        let at = ((y >> 8) & 0xFF) * 90 / 100;
        let o = (y & 0xFF) * 90 / 100;
        k[a] = 0xFF000000 | (m << 16) | (at << 8) | o;
    }
    
    for r in 0..aur {
        let elg = r * oy;
        let bg = agk[r];
        if bg >= 0 && (bg as usize) < i {
            let c = bg as usize;
            for y in 0..oy.v(d.ao(elg)) {
                k[c * d + elg + y] = 0xFFCCFFCC;
            }
            for ase in 1..8u32 {
                let ty = bg - ase as i32;
                if ty >= 0 && (ty as usize) < i {
                    let hj = (200u32).ao(ase * 20);
                    let at = hj.v(255);
                    for y in 0..oy.v(d.ao(elg)) {
                        k[ty as usize * d + elg + y] =
                            0xFF000000 | ((at / 4) << 16) | (at << 8) | (at / 4);
                    }
                }
            }
        }
        agk[r] += arz[r] as i32;
        if agk[r] > (i as i32 + 20) {
            *dv = bpk(*dv);
            agk[r] = -((*dv % (i as u32 / 2)) as i32);
            *dv = bpk(*dv);
            arz[r] = 1 + (*dv % 4) as u8;
        }
    }
}







#[inline(always)]
fn cxr(b: f32) -> f32 {
    
    let b = if b > 10.0 { 10.0 } else if b < -10.0 { -10.0 } else { b };
    
    let q = (1 << 23) as f32 / core::f32::consts::IG_;
    let o = (1 << 23) as f32 * (127.0 - 0.04367744890362246);
    let p = (q * b + o) as i32;
    f32::bhb(if p > 0 { p as u32 } else { 0 })
}


#[inline(always)]
fn fii(b: f32) -> f32 {
    if b > 5.0 { return 1.0; }
    if b < -5.0 { return -1.0; }
    let hy = b * b;
    b / (1.0 + b.gp() + hy * 0.28)
}

fn pcc(k: &mut [u32], d: usize, i: usize, frame: u32) {
    
    let bv = 4usize;
    let kp = d / bv;
    let kl = i / bv;

    let ab = frame as f32 * 0.03;
    let ix = kl as f32;
    let kb = kp as f32;

    
    static Bry: [f32; 256] = {
        let mut djf = [0.0f32; 256];
        let mut a = 0;
        while a < 256 {
            
            let hg = (a as f64) * 6.283185307179586 / 256.0;
            
            let q = hg % 6.283185307179586;
            let b = if q > 3.14159265358979 { q - 6.283185307179586 } else { q };
            let hy = b * b;
            let ajr = hy * b;
            let fbw = ajr * hy;
            let fyz = fbw * hy;
            let jxm = fyz * hy;
            let e = b - ajr / 6.0 + fbw / 120.0 - fyz / 5040.0 + jxm / 362880.0;
            djf[a] = e as f32;
            a += 1;
        }
        djf
    };
    
    #[inline(always)]
    fn kxl(b: f32) -> f32 {
        
        let w = ((b * (256.0 / 6.2831853)) as i32 & 255) as u8;
        Bry[w as usize]
    }
    
    #[inline(always)]
    fn nsy(b: f32) -> f32 {
        let w = (((b + 1.5707963) * (256.0 / 6.2831853)) as i32 & 255) as u8;
        Bry[w as usize]
    }

    for cq in 0..kl {
        let cth = (cq as f32 * 2.0 - ix) / ix;
        
        let ksh = cxr(cth);
        let ksi = cxr(-cth);
        let ksj = cxr(cth * -2.0);

        for cr in 0..kp {
            let fqb = (cr as f32 * 2.0 - kb) / ix;

            let kqn = fqb * fqb + cth * cth;
            let dm = (0.7 - kqn).gp();

            let e = (1.0 - dm) * 5.0; 
            let mut fp = fqb * e;
            let mut iz = cth * e;

            let mut htk: f32 = 0.0;
            let mut htj: f32 = 0.0;
            let mut hti: f32 = 0.0;

            
            let mut a: f32 = 1.0;
            while a <= 6.0 {
                let hok = 1.0 / a;
                fp += nsy(iz * a + ab) * hok + 0.7;
                iz += nsy(fp * a + a + ab) * hok + 0.7;

                let wz = (fp - iz).gp() * 0.2;
                htk += (kxl(fp) + 1.0) * wz;
                htj += (kxl(iz) + 1.0) * wz;
                hti += (kxl(iz) + 1.0) * wz;
                a += 1.0;
            }

            let duu = cxr(-4.0 * dm);
            let xb = fii(ksh * duu / (htk + 0.001));
            let lp = fii(ksi * duu / (htj + 0.001));
            let pq = fii(ksj * duu / (hti + 0.001));

            let m = (xb.gp() * 255.0).v(255.0) as u32;
            let at = (lp.gp() * 255.0).v(255.0) as u32;
            let o = (pq.gp() * 255.0).v(255.0) as u32;
            let s = 0xFF000000 | (m << 16) | (at << 8) | o;

            
            for bg in 0..bv {
                let x = cq * bv + bg;
                if x >= i { break; }
                let fte = x * d + cr * bv;
                for dx in 0..bv {
                    let y = cr * bv + dx;
                    if y < d {
                        k[fte + dx] = s;
                    }
                }
            }
        }
    }
}
