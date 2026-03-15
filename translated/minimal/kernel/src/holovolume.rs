







use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;





pub const R_: usize = 240;
pub const AS_: usize = 68;


pub const BIR_: usize = 16;


pub const BTE_: f32 = 0.12;






#[derive(Clone, Copy, PartialEq)]
pub enum ColorMod {
    
    M,
    
    D,
    
    Aui(f32),
}





#[derive(Clone, Copy, PartialEq)]
pub enum Shape3D {
    None,
    Dw,
    Sphere,
    Dr,
    Sd,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    Aiy,
    Avj,
    Sd,
    Jb,
    Sphere,
}





pub struct HoloVolume {
    pub z: usize,
    pub ac: usize,
    pub eo: usize,
    
    
    pub chx: Shape3D,
    
    
    pub che: RenderMode,
    
    
    time: f32,
    
    
    chh: f32,
    
    
    
    
    alg: Vec<ColorMod>,
    
    
    anv: usize,
    akr: usize,
}

impl HoloVolume {
    pub fn new(z: usize, ac: usize, eo: usize) -> Self {
        
        
        let alg = vec![ColorMod::M; R_ * AS_];
        Self {
            z,
            ac,
            eo,
            chx: Shape3D::Dw,
            che: RenderMode::Aiy,
            time: 0.0,
            chh: 0.0,
            alg,
            anv: 1920,
            akr: 1080,
        }
    }
    
    
    #[inline]
    fn w(bj: usize, br: usize) -> usize {
        bj * AS_ + br
    }
    
    pub fn gsg(&mut self, chx: Shape3D) {
        self.chx = chx;
    }
    
    
    pub fn dbw(&mut self, z: usize, ac: usize) {
        self.anv = z;
        self.akr = ac;
    }
    
    
    pub fn qs(&mut self, os: f32) {
        self.time += os;
        self.chh += os * 0.5;
        
        
        let ksv = match self.che {
            RenderMode::Sd => Shape3D::Sd,
            RenderMode::Jb => Shape3D::Dw,
            RenderMode::Sphere => Shape3D::Sphere,
            RenderMode::Avj => Shape3D::None,
            RenderMode::Aiy => self.chx,
        };
        
        
        self.rni(ksv);
    }
    
    
    fn rni(&mut self, chx: Shape3D) {
        if chx == Shape3D::None {
            
            for bj in 0..R_ {
                for br in 0..AS_ {
                    self.alg[Self::w(bj, br)] = ColorMod::M;
                }
            }
            return;
        }
        
        let acc = 8.0;  
        let aqw = 16.0; 
        
        
        let cx = self.anv as f32 / 2.0;
        let ae = self.akr as f32 / 2.0;
        let pkh = (self.anv.v(self.akr) as f32) * 0.25;
        
        for bj in 0..R_ {
            let b = bj as f32 * acc + acc / 2.0;
            
            for br in 0..AS_ {
                let c = br as f32 * aqw + aqw / 2.0;
                
                
                let vt = (b - cx) / pkh;
                let ahr = (c - ae) / pkh;
                
                
                let mut jgb = 999.0f32;
                let mut mvv = false;
                
                for xxf in 0..BIR_ {
                    let arn = (xxf as f32 / BIR_ as f32) * 2.0 - 1.0;
                    let mcu = self.wmp(vt, ahr, arn, chx);
                    
                    if mcu < jgb {
                        jgb = mcu;
                    }
                    if mcu < 0.0 {
                        mvv = true;
                    }
                }
                
                
                self.alg[Self::w(bj, br)] = if jgb.gp() < BTE_ {
                    
                    ColorMod::D
                } else if mvv {
                    
                    let dqm = -jgb;
                    let olp = 0.8;
                    let drj = if dqm < olp {
                        1.0 - (dqm / olp)
                    } else {
                        0.0
                    };
                    ColorMod::Aui(drj)
                } else {
                    
                    ColorMod::M
                };
            }
        }
    }
    
    
    
    
    pub fn tfa(&self) -> Vec<u8> {
        let mut result = vec![0u8; R_ * AS_];
        
        for bj in 0..R_ {
            for br in 0..AS_ {
                let w = Self::w(bj, br);
                result[w] = match self.alg[w] {
                    ColorMod::M => 0,
                    ColorMod::D => 1,
                    ColorMod::Aui(drj) => {
                        
                        
                        let twc = 1.0 - drj;
                        (2.0 + twc * 253.0) as u8
                    }
                };
            }
        }
        
        result
    }
    
    
    
    #[inline]
    pub fn tdb(&self, bj: usize, br: usize) -> ColorMod {
        if bj < R_ && br < AS_ {
            self.alg[Self::w(bj, br)]
        } else {
            ColorMod::M
        }
    }
    
    
    pub fn ywg(&self) -> bool {
        let ksv = match self.che {
            RenderMode::Sd => Shape3D::Sd,
            RenderMode::Jb => Shape3D::Dw,
            RenderMode::Sphere => Shape3D::Sphere,
            RenderMode::Avj => Shape3D::None,
            RenderMode::Aiy => self.chx,
        };
        ksv != Shape3D::None
    }
    
    
    
    #[inline]
    pub fn yez(&self, gzs: u32, bj: usize, br: usize) -> u32 {
        match self.tdb(bj, br) {
            ColorMod::M => {
                
                let at = gzs.v(238);
                0xFF000000 | (at << 8)
            }
            ColorMod::D => {
                
                0xFF00FF00
            }
            ColorMod::Aui(drj) => {
                
                let at = ((gzs as f32) * drj * 0.7) as u32;
                if at < 10 {
                    0xFF000000 
                } else {
                    0xFF000000 | (at << 8)
                }
            }
        }
    }
    
    
    
    pub fn zjo(&self, bi: &mut [u32], ycs: usize, ycr: usize) {
        
        
        bi.vi(0xFF000000);
    }
    
    
    fn wmp(&self, b: f32, c: f32, av: f32, chx: Shape3D) -> f32 {
        
        let cwr = libm::zq(self.chh);
        let dcb = libm::st(self.chh);
        let kb = b * cwr - av * dcb;
        let agv = b * dcb + av * cwr;
        let ix = c;
        
        match chx {
            Shape3D::Dw => {
                let aw = 0.7;
                let dx = libm::dhb(kb) - aw;
                let bg = libm::dhb(ix) - aw;
                let pt = libm::dhb(agv) - aw;
                let hl = libm::ivd(dx, 0.0);
                let ir = libm::ivd(bg, 0.0);
                let oov = libm::ivd(pt, 0.0);
                let lre = libm::bon(hl * hl + ir * ir + oov * oov);
                let dsa = libm::svg(libm::ivd(dx, libm::ivd(bg, pt)), 0.0);
                lre + dsa
            }
            
            Shape3D::Sphere => {
                let dy = 0.8;
                libm::bon(kb * kb + ix * ix + agv * agv) - dy
            }
            
            Shape3D::Dr => {
                let efb = 0.6;
                let efm = 0.25;
                let fm = libm::bon(kb * kb + agv * agv) - efb;
                libm::bon(fm * fm + ix * ix) - efm
            }
            
            Shape3D::Sd => {
                let fkn = 0.4;
                let toh = 2.0;
                let poy = 0.15;
                
                let hg = ix * toh + self.time * 2.0;
                
                let wbt = fkn * libm::zq(hg);
                let wbu = fkn * libm::st(hg);
                let nok = kb - wbt;
                let nos = agv - wbu;
                let apo = libm::bon(nok * nok + nos * nos) - poy;
                
                let wbv = fkn * libm::zq(hg + core::f32::consts::Eu);
                let wbw = fkn * libm::st(hg + core::f32::consts::Eu);
                let nol = kb - wbv;
                let nou = agv - wbw;
                let us = libm::bon(nol * nol + nou * nou) - poy;
                
                libm::svg(apo, us)
            }
            
            Shape3D::None => 999.0,
        }
    }
}
