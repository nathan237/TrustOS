

#![allow(bgr)]

use alloc::vec;
use alloc::vec::Vec;
use super::cartridge::Cartridge;


pub const AFV_: [u32; 64] = [
    0xFF666666, 0xFF002A88, 0xFF1412A7, 0xFF3B00A4, 0xFF5C007E, 0xFF6E0040, 0xFF6C0600, 0xFF561D00,
    0xFF333500, 0xFF0B4800, 0xFF005200, 0xFF004F08, 0xFF00404D, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFADADAD, 0xFF155FD9, 0xFF4240FF, 0xFF7527FE, 0xFFA01ACC, 0xFFB71E7B, 0xFFB53120, 0xFF994E00,
    0xFF6B6D00, 0xFF388700, 0xFF0C9300, 0xFF008F32, 0xFF007C8D, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFFFFEFF, 0xFF64B0FF, 0xFF9290FF, 0xFFC676FF, 0xFFF36AFF, 0xFFFE6ECC, 0xFFFE8170, 0xFFEA9E22,
    0xFFBCBE00, 0xFF88D800, 0xFF5CE430, 0xFF45E082, 0xFF48CDDE, 0xFF4F4F4F, 0xFF000000, 0xFF000000,
    0xFFFFFEFF, 0xFFC0DFFF, 0xFFD3D2FF, 0xFFE8C8FF, 0xFFFBC2FF, 0xFFFEC4EA, 0xFFFECCC5, 0xFFF7D8A5,
    0xFFE4E594, 0xFFCFEF96, 0xFFBDF4AB, 0xFFB3F3CC, 0xFFB5EBF2, 0xFFB8B8B8, 0xFF000000, 0xFF000000,
];

pub struct Ppu {
    
    pub db: u8,       
    pub hs: u8,       
    pub status: u8,     
    pub dkd: u8,   

    
    pub p: u16,         
    pub ab: u16,         
    pub kwg: u8,     
    pub d: bool,        
    pub iqo: u8,   

    
    pub awh: [u8; 256],        
    pub aof: [u8; 2048],      
    pub aim: [u8; 32],     

    
    pub ys: i32,
    pub amb: u32,
    pub oo: u64,
    pub uus: bool,
    pub wrl: bool,

    
    jrc: [u8; 8],
    gsu: u8,

    
    pub framebuffer: Vec<u32>,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            db: 0, hs: 0, status: 0, dkd: 0,
            p: 0, ab: 0, kwg: 0, d: false, iqo: 0,
            awh: [0; 256],
            aof: [0; 2048],
            aim: [0; 32],
            ys: -1,
            amb: 0,
            oo: 0,
            uus: false,
            wrl: false,
            jrc: [0xFF; 8],
            gsu: 0,
            framebuffer: vec![0u32; 256 * 240],
        }
    }

    

    pub fn gql(&mut self, ag: u16, on: &Cartridge) -> u8 {
        match ag & 7 {
            2 => { 
                let ap = (self.status & 0xE0) | (self.iqo & 0x1F);
                self.status &= !0x80; 
                self.d = false;
                ap
            }
            4 => { 
                self.awh[self.dkd as usize]
            }
            7 => { 
                let ag = self.p & 0x3FFF;
                let ap = if ag >= 0x3F00 {
                    self.ott(ag)
                } else {
                    let cox = self.iqo;
                    self.iqo = self.egx(ag, on);
                    cox
                };
                self.p = self.p.cn(if self.db & 0x04 != 0 { 32 } else { 1 });
                ap
            }
            _ => 0,
        }
    }

    pub fn ihl(&mut self, ag: u16, ap: u8, on: &mut Cartridge) {
        match ag & 7 {
            0 => { 
                self.db = ap;
                self.ab = (self.ab & 0xF3FF) | (((ap as u16) & 3) << 10);
            }
            1 => self.hs = ap,
            3 => self.dkd = ap,
            4 => { 
                self.awh[self.dkd as usize] = ap;
                self.dkd = self.dkd.cn(1);
            }
            5 => { 
                if !self.d {
                    self.ab = (self.ab & 0xFFE0) | ((ap as u16) >> 3);
                    self.kwg = ap & 7;
                } else {
                    self.ab = (self.ab & 0x8C1F)
                        | (((ap as u16) & 0xF8) << 2)
                        | (((ap as u16) & 7) << 12);
                }
                self.d = !self.d;
            }
            6 => { 
                if !self.d {
                    self.ab = (self.ab & 0x00FF) | (((ap as u16) & 0x3F) << 8);
                } else {
                    self.ab = (self.ab & 0xFF00) | (ap as u16);
                    self.p = self.ab;
                }
                self.d = !self.d;
            }
            7 => { 
                let q = self.p & 0x3FFF;
                self.lva(q, ap, on);
                self.p = self.p.cn(if self.db & 0x04 != 0 { 32 } else { 1 });
            }
            _ => {}
        }
    }

    

    fn egx(&self, ag: u16, on: &Cartridge) -> u8 {
        match ag {
            0x0000..=0x1FFF => on.egx(ag),
            0x2000..=0x3EFF => {
                let htb = on.ono(ag - 0x2000);
                self.aof[htb as usize]
            }
            0x3F00..=0x3FFF => self.ott(ag),
            _ => 0,
        }
    }

    fn lva(&mut self, ag: u16, ap: u8, on: &mut Cartridge) {
        match ag {
            0x0000..=0x1FFF => on.lva(ag, ap),
            0x2000..=0x3EFF => {
                let htb = on.ono(ag - 0x2000);
                self.aof[htb as usize] = ap;
            }
            0x3F00..=0x3FFF => {
                let w = (ag & 0x1F) as usize;
                self.aim[w] = ap & 0x3F;
                
                if w & 3 == 0 {
                    self.aim[w ^ 0x10] = ap & 0x3F;
                }
            }
            _ => {}
        }
    }

    fn ott(&self, ag: u16) -> u8 {
        let mut w = (ag & 0x1F) as usize;
        if w >= 16 && w & 3 == 0 { w -= 16; }
        self.aim[w] & 0x3F
    }

    

    
    pub fn wud(&mut self, on: &Cartridge) -> bool {
        let mut pwd = false;
        let pcf = self.hs & 0x18 != 0;

        match self.ys {
            0..=239 => {
                
                if pcf {
                    self.snr(on);
                    self.lzh(on);
                }
            }
            241 => {
                
                self.status |= 0x80;
                if self.db & 0x80 != 0 {
                    pwd = true;
                }
            }
            261 => {
                
                self.status &= !0xE0; 
                if pcf {
                    
                    self.p = (self.p & 0x041F) | (self.ab & 0x7BE0);
                }
            }
            _ => {}
        }

        self.ys += 1;
        if self.ys > 261 {
            self.ys = 0;
            self.oo += 1;
        }

        pwd
    }

    fn lzh(&mut self, on: &Cartridge) {
        let c = self.ys as usize;
        if c >= 240 { return; }

        let qpe = self.hs & 0x08 != 0;
        let wrh = self.hs & 0x10 != 0;
        let qpi = self.hs & 0x02 != 0;
        let wri = self.hs & 0x04 != 0;

        let haj = if self.db & 0x10 != 0 { 0x1000u16 } else { 0u16 };
        let mgv = if self.db & 0x08 != 0 { 0x1000u16 } else { 0u16 };
        let xap = self.db & 0x20 != 0;
        let ibg = if xap { 16 } else { 8 };

        
        let rlc = self.p & 0x1F;
        let kjo = (self.p >> 5) & 0x1F;
        let iur = (self.p >> 12) & 7;
        let uwa = (self.p >> 10) & 3;

        for b in 0..256usize {
            let xu = b;
            let ewr = b as u16 + self.kwg as u16;
            let pte = (rlc as u16 + ewr / 8) as u16;
            let stq = (ewr % 8) as u8;

            
            let (vp, bdo) = if qpe && (qpi || b >= 8) {
                let jze = pte & 0x1F;
                let uvy = if pte >= 32 { 1u16 } else { 0 };
                let uvx = uwa ^ uvy;
                let orh = 0x2000 + uvx * 0x400;

                let htb = orh + (kjo + iur / 8) * 32 + jze;
                let cup = self.egx(htb, on) as u16;

                let qla = orh + 0x03C0 + ((kjo + iur / 8) / 4) * 8 + jze / 4;
                let qn = self.egx(qla, on);
                let acn = ((((kjo + iur / 8) & 2)) | ((jze & 2) >> 1)) * 2;
                let lrx = (qn >> acn) & 3;

                let huq = haj + cup * 16 + (iur & 7);
                let hh = self.egx(huq, on);
                let gd = self.egx(huq + 8, on);
                let ga = 7 - stq;
                let s = ((hh >> ga) & 1) | (((gd >> ga) & 1) << 1);

                (s, lrx)
            } else {
                (0, 0)
            };

            
            let (mgu, wrj, wrk, tza) = if wrh && (wri || b >= 8) {
                self.ter(b as u8, c as u8, mgv, ibg, on)
            } else {
                (0, 0, false, false)
            };

            
            if tza && vp != 0 && mgu != 0 && b < 255 {
                self.status |= 0x40;
            }

            
            let hjk = if mgu != 0 && (vp == 0 || !wrk) {
                
                let w = self.aim[16 + wrj as usize * 4 + mgu as usize] as usize;
                AFV_[w & 0x3F]
            } else if vp != 0 {
                let w = self.aim[bdo as usize * 4 + vp as usize] as usize;
                AFV_[w & 0x3F]
            } else {
                AFV_[self.aim[0] as usize & 0x3F]
            };

            self.framebuffer[c * 256 + xu] = hjk;
        }

        
        if self.ys < 240 {
            self.tsr();
            
            self.p = (self.p & !0x041F) | (self.ab & 0x041F);
        }
    }

    fn ter(&self, b: u8, c: u8, mgv: u16, ibg: u8, on: &Cartridge) -> (u8, u8, bool, bool) {
        for a in 0..self.gsu as usize {
            let w = self.jrc[a] as usize * 4;
            let mgx = self.awh[w] as i16;
            let mgw = self.awh[w + 1];
            let jra = self.awh[w + 2];
            let jrb = self.awh[w + 3] as i16;

            if (b as i16) < jrb || (b as i16) >= jrb + 8 { continue; }

            let sut = jra & 0x40 != 0;
            let suu = jra & 0x80 != 0;
            let abv = jra & 0x20 != 0; 
            let lrx = jra & 3;

            let mut br = c as i16 - mgx - 1;
            if suu { br = (ibg as i16 - 1) - br; }
            if br < 0 || br >= ibg as i16 { continue; }

            let (ccd, vfb) = if ibg == 16 {
                let om = (mgw as u16 & 1) * 0x1000;
                let ptc = mgw & 0xFE;
                if br < 8 {
                    (ptc as u16, om)
                } else {
                    ((ptc + 1) as u16, om)
                }
            } else {
                (mgw as u16, mgv)
            };

            let fcl = (br % 8) as u16;
            let huq = vfb + ccd * 16 + fcl;
            let hh = self.egx(huq, on);
            let gd = self.egx(huq + 8, on);

            let bj = if sut { b as i16 - jrb } else { 7 - (b as i16 - jrb) };
            let s = ((hh >> bj) & 1) | (((gd >> bj) & 1) << 1);

            if s != 0 {
                return (s, lrx, abv, self.jrc[a] == 0);
            }
        }
        (0, 0, false, false)
    }

    fn snr(&mut self, xye: &Cartridge) {
        self.gsu = 0;
        let c = self.ys as u8;
        let i = if self.db & 0x20 != 0 { 16i16 } else { 8i16 };

        for a in 0..64u8 {
            let mgx = self.awh[a as usize * 4] as i16;
            let wz = c as i16 - mgx;
            if wz >= 1 && wz <= i {
                if self.gsu < 8 {
                    self.jrc[self.gsu as usize] = a;
                    self.gsu += 1;
                } else {
                    self.status |= 0x20; 
                    break;
                }
            }
        }
    }

    fn tsr(&mut self) {
        if (self.p & 0x7000) != 0x7000 {
            self.p += 0x1000; 
        } else {
            self.p &= !0x7000; 
            let mut c = (self.p & 0x03E0) >> 5;
            if c == 29 {
                c = 0;
                self.p ^= 0x0800; 
            } else if c == 31 {
                c = 0;
            } else {
                c += 1;
            }
            self.p = (self.p & !0x03E0) | (c << 5);
        }
    }

    
    pub fn uwt(&mut self, f: &[u8; 256]) {
        self.awh.dg(f);
    }
}
