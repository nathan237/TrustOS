








use alloc::string::String;
use alloc::vec::Vec;

pub mod bmp;
pub mod ppm;
pub mod png;

pub use bmp::*;
pub use ppm::*;
pub use png::*;


#[derive(Clone)]
pub struct Image {
    pub z: u32,
    pub ac: u32,
    pub hz: Vec<u32>,  
}

impl Image {
    
    pub fn new(z: u32, ac: u32) -> Self {
        let aw = (z * ac) as usize;
        Self {
            z,
            ac,
            hz: alloc::vec![0xFF000000; aw],
        }
    }
    
    
    pub fn fjd(z: u32, ac: u32, hz: Vec<u32>) -> Self {
        Self { z, ac, hz }
    }
    
    
    pub fn beg(&self, b: u32, c: u32) -> u32 {
        if b < self.z && c < self.ac {
            self.hz[(c * self.z + b) as usize]
        } else {
            0
        }
    }
    
    
    pub fn aht(&mut self, b: u32, c: u32, s: u32) {
        if b < self.z && c < self.ac {
            self.hz[(c * self.z + b) as usize] = s;
        }
    }
    
    
    pub fn vi(&mut self, s: u32) {
        for il in &mut self.hz {
            *il = s;
        }
    }
    
    
    pub fn po(&self, b: i32, c: i32) {
        self.nnp(b, c, self.z, self.ac);
    }
    
    
    pub fn nnp(&self, b: i32, c: i32, fgq: u32, fgp: u32) {
        let (lu, qh) = crate::framebuffer::yn();
        
        for bg in 0..fgp {
            let abi = c + bg as i32;
            if abi < 0 || abi >= qh as i32 { continue; }
            
            
            let bih = (bg as u64 * self.ac as u64 / fgp as u64) as u32;
            
            for dx in 0..fgq {
                let xu = b + dx as i32;
                if xu < 0 || xu >= lu as i32 { continue; }
                
                
                let blg = (dx as u64 * self.z as u64 / fgq as u64) as u32;
                
                let il = self.beg(blg, bih);
                let dw = (il >> 24) & 0xFF;
                
                if dw == 255 {
                    
                    crate::framebuffer::sf(xu as u32, abi as u32, il);
                } else if dw > 0 {
                    
                    let ei = crate::framebuffer::beg(xu as u32, abi as u32);
                    let dei = mzm(il, ei);
                    crate::framebuffer::sf(xu as u32, abi as u32, dei);
                }
                
            }
        }
    }
    
    
    pub fn sbv(&self, b: i32, c: i32, tfy: u8) {
        let (lu, qh) = crate::framebuffer::yn();
        
        for cq in 0..self.ac {
            let abi = c + cq as i32;
            if abi < 0 || abi >= qh as i32 { continue; }
            
            for cr in 0..self.z {
                let xu = b + cr as i32;
                if xu < 0 || xu >= lu as i32 { continue; }
                
                let il = self.beg(cr, cq);
                let vif = ((il >> 24) & 0xFF) as u32;
                let nfa = (vif * tfy as u32) / 255;
                
                if nfa > 0 {
                    let bvl = (nfa << 24) | (il & 0x00FFFFFF);
                    let ei = crate::framebuffer::beg(xu as u32, abi as u32);
                    let dei = mzm(bvl, ei);
                    crate::framebuffer::sf(xu as u32, abi as u32, dei);
                }
            }
        }
    }
    
    
    pub fn bv(&self, cst: u32, csr: u32) -> Image {
        let mut hyu = Image::new(cst, csr);
        
        for bg in 0..csr {
            let bih = (bg as u64 * self.ac as u64 / csr as u64) as u32;
            for dx in 0..cst {
                let blg = (dx as u64 * self.z as u64 / cst as u64) as u32;
                hyu.aht(dx, bg, self.beg(blg, bih));
            }
        }
        
        hyu
    }
    
    
    pub fn ykw(&self, b: u32, c: u32, d: u32, i: u32) -> Image {
        let mut nhn = Image::new(d, i);
        
        for bg in 0..i {
            for dx in 0..d {
                nhn.aht(dx, bg, self.beg(b + dx, c + bg));
            }
        }
        
        nhn
    }
}


fn mzm(lp: u32, ei: u32) -> u32 {
    let dw = ((lp >> 24) & 0xFF) as u32;
    if dw == 0 { return ei; }
    if dw == 255 { return lp; }
    
    let akg = 255 - dw;
    
    let ebm = ((lp >> 16) & 0xFF) as u32;
    let ebl = ((lp >> 8) & 0xFF) as u32;
    let ebk = (lp & 0xFF) as u32;
    
    let cos = ((ei >> 16) & 0xFF) as u32;
    let cor = ((ei >> 8) & 0xFF) as u32;
    let coq = (ei & 0xFF) as u32;
    
    let m = (ebm * dw + cos * akg) / 255;
    let at = (ebl * dw + cor * akg) / 255;
    let o = (ebk * dw + coq * akg) / 255;
    
    0xFF000000 | (m << 16) | (at << 8) | o
}






pub fn load(path: &str) -> Option<Image> {
    
    let hql = path.ptw();
    
    if hql.pp(".bmp") {
        bmp::jdu(path)
    } else if hql.pp(".ppm") || hql.pp(".pnm") {
        ppm::ojv(path)
    } else if hql.pp(".raw") || hql.pp(".rgba") {
        
        if let Ok(f) = crate::vfs::mq(path) {
            
            if f.len() >= 8 {
                let z = u32::dj([f[0], f[1], f[2], f[3]]);
                let ac = u32::dj([f[4], f[5], f[6], f[7]]);
                let amn = &f[8..];
                
                if amn.len() >= (z * ac * 4) as usize {
                    let mut hz = Vec::fc((z * ac) as usize);
                    for a in 0..(z * ac) as usize {
                        let l = a * 4;
                        let il = u32::dj([
                            amn[l],
                            amn[l + 1],
                            amn[l + 2],
                            amn[l + 3],
                        ]);
                        hz.push(il);
                    }
                    return Some(Image::fjd(z, ac, hz));
                }
            }
        }
        None
    } else {
        
        bmp::jdu(path).or_else(|| ppm::ojv(path))
    }
}


pub fn ljf(f: &[u8], format: ImageFormat) -> Option<Image> {
    match format {
        ImageFormat::Vp => bmp::hqf(f),
        ImageFormat::Yf => ppm::ljj(f),
        ImageFormat::Axl { z, ac } => {
            if f.len() >= (z * ac * 4) as usize {
                let mut hz = Vec::fc((z * ac) as usize);
                for a in 0..(z * ac) as usize {
                    let l = a * 4;
                    let il = u32::dj([
                        f[l],
                        f[l + 1],
                        f[l + 2],
                        f[l + 3],
                    ]);
                    hz.push(il);
                }
                Some(Image::fjd(z, ac, hz))
            } else {
                None
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum ImageFormat {
    Vp,
    Yf,
    Axl { z: u32, ac: u32 },
}


trait Cob {
    fn ptw(&self) -> String;
}

impl Cob for str {
    fn ptw(&self) -> String {
        self.bw().map(|r| {
            if r >= 'A' && r <= 'Z' {
                (r as u8 + 32) as char
            } else {
                r
            }
        }).collect()
    }
}






pub fn rqs(z: u32, ac: u32, s: u32) -> Image {
    let mut th = Image::new(z, ac);
    th.vi(s);
    th
}


pub fn nhd(z: u32, ac: u32, idz: u32, hba: u32) -> Image {
    let mut th = Image::new(z, ac);
    
    let agd = ((idz >> 16) & 0xFF) as i32;
    let ejs = ((idz >> 8) & 0xFF) as i32;
    let bov = (idz & 0xFF) as i32;
    
    let avi = ((hba >> 16) & 0xFF) as i32;
    let ei = ((hba >> 8) & 0xFF) as i32;
    let aaa = (hba & 0xFF) as i32;
    
    for c in 0..ac {
        let ab = c as f32 / ac as f32;
        let m = (agd as f32 * (1.0 - ab) + avi as f32 * ab) as u32;
        let at = (ejs as f32 * (1.0 - ab) + ei as f32 * ab) as u32;
        let o = (bov as f32 * (1.0 - ab) + aaa as f32 * ab) as u32;
        let s = 0xFF000000 | (m << 16) | (at << 8) | o;
        
        for b in 0..z {
            th.aht(b, c, s);
        }
    }
    
    th
}


pub fn rql(z: u32, ac: u32, aw: u32, bjo: u32, btr: u32) -> Image {
    let mut th = Image::new(z, ac);
    
    for c in 0..ac {
        for b in 0..z {
            let rac = ((b / aw) + (c / aw)) % 2 == 0;
            th.aht(b, c, if rac { bjo } else { btr });
        }
    }
    
    th
}
