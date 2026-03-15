

#![allow(bgr)]

use alloc::vec;
use alloc::vec::Vec;

const CBP_: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

#[derive(Clone, Copy, PartialEq)]
pub enum Mirror {
    Po,
    On,
    Bsv,
    Bsw,
    Bhk,
}

pub struct Cartridge {
    pub bce: Vec<u8>,
    pub cpe: Vec<u8>,
    pub inn: bool,
    pub gpu: [u8; 8192],
    pub fnq: u8,
    pub djo: Mirror,
    
    fnj: u8,
    gma: u8,
    glz: u8,
    hqo: u8,
    jef: u8,
    jeg: u8,
    
    jeh: u8,
    jei: u8,
}

impl Cartridge {
    pub fn azs() -> Self {
        Self {
            bce: vec![0u8; 32768],
            cpe: vec![0u8; 8192],
            inn: true,
            gpu: [0; 8192],
            fnq: 0,
            djo: Mirror::Po,
            fnj: 0x10,
            gma: 0,
            glz: 0x0C,
            hqo: 0,
            jef: 0,
            jeg: 0,
            jeh: 0,
            jei: 0,
        }
    }

    pub fn sxy(f: &[u8]) -> Option<Self> {
        if f.len() < 16 { return None; }
        if f[0..4] != CBP_ { return None; }

        let lvk = f[4] as usize;
        let rao = f[5] as usize;
        let iuy = f[6];
        let sul = f[7];
        let fnq = (sul & 0xF0) | (iuy >> 4);
        let djo = if iuy & 0x08 != 0 {
            Mirror::Bhk
        } else if iuy & 0x01 != 0 {
            Mirror::On
        } else {
            Mirror::Po
        };
        let tnk = iuy & 0x04 != 0;
        let l = 16 + if tnk { 512 } else { 0 };
        let hwa = lvk * 16384;
        let kho = rao * 8192;

        if f.len() < l + hwa + kho { return None; }

        let bce = f[l..l + hwa].ip();
        let (cpe, inn) = if kho > 0 {
            (f[l + hwa..l + hwa + kho].ip(), false)
        } else {
            (vec![0u8; 8192], true)
        };

        crate::serial_println!("[NES] ROM: mapper={} PRG={}KB CHR={}KB mirror={:?}",
            fnq, hwa / 1024, cpe.len() / 1024,
            if djo == Mirror::On { "V" } else { "H" });

        Some(Self {
            bce,
            cpe,
            inn,
            gpu: [0; 8192],
            fnq,
            djo,
            fnj: 0x10,
            gma: 0,
            glz: 0x0C,
            hqo: 0,
            jef: 0,
            jeg: 0,
            jeh: 0,
            jei: 0,
        })
    }

    

    pub fn mc(&self, ag: u16) -> u8 {
        match self.fnq {
            0 => self.lkj(ag),
            1 => self.uju(ag),
            2 => self.ujx(ag),
            3 => self.ujz(ag),
            _ => self.lkj(ag),
        }
    }

    pub fn ok(&mut self, ag: u16, ap: u8) {
        match self.fnq {
            0 => {} 
            1 => self.ujv(ag, ap),
            2 => self.ujy(ag, ap),
            3 => self.uka(ag, ap),
            _ => {}
        }
        
        if ag >= 0x6000 && ag < 0x8000 {
            self.gpu[(ag - 0x6000) as usize] = ap;
        }
    }

    

    pub fn egx(&self, ag: u16) -> u8 {
        match self.fnq {
            3 => {
                let qmr = (self.jei as usize) * 8192;
                let w = qmr + (ag as usize & 0x1FFF);
                if w < self.cpe.len() { self.cpe[w] } else { 0 }
            }
            1 => self.ujw(ag),
            _ => {
                let w = ag as usize & (self.cpe.len() - 1).am(0x1FFF);
                if w < self.cpe.len() { self.cpe[w] } else { 0 }
            }
        }
    }

    pub fn lva(&mut self, ag: u16, ap: u8) {
        if self.inn {
            let w = ag as usize & 0x1FFF;
            if w < self.cpe.len() {
                self.cpe[w] = ap;
            }
        }
    }

    pub fn ono(&self, ag: u16) -> u16 {
        let ag = ag & 0x0FFF;
        match self.djo {
            Mirror::Po => {
                
                let gg = (ag >> 11) & 1;
                (gg * 0x400) | (ag & 0x03FF)
            }
            Mirror::On => {
                
                ag & 0x07FF
            }
            Mirror::Bsv => ag & 0x03FF,
            Mirror::Bsw => 0x400 | (ag & 0x03FF),
            Mirror::Bhk => ag & 0x0FFF,
        }
    }

    

    fn lkj(&self, ag: u16) -> u8 {
        match ag {
            0x6000..=0x7FFF => self.gpu[(ag - 0x6000) as usize],
            0x8000..=0xFFFF => {
                let w = (ag - 0x8000) as usize;
                if self.bce.len() <= 16384 {
                    self.bce[w & 0x3FFF] 
                } else {
                    self.bce[w & (self.bce.len() - 1)]
                }
            }
            _ => 0,
        }
    }

    

    fn uju(&self, ag: u16) -> u8 {
        match ag {
            0x6000..=0x7FFF => self.gpu[(ag - 0x6000) as usize],
            0x8000..=0xFFFF => {
                let vld = (self.glz >> 2) & 3;
                let om = self.jeg as usize & 0x0F;
                let lvk = self.bce.len() / 16384;
                match vld {
                    0 | 1 => {
                        
                        let ar = (om & !1) * 16384;
                        let w = ar + (ag as usize - 0x8000);
                        self.bce[w % self.bce.len()]
                    }
                    2 => {
                        
                        if ag < 0xC000 {
                            self.bce[(ag as usize - 0x8000) % self.bce.len()]
                        } else {
                            let ar = om * 16384;
                            self.bce[(ar + (ag as usize - 0xC000)) % self.bce.len()]
                        }
                    }
                    _ => {
                        
                        if ag < 0xC000 {
                            let ar = om * 16384;
                            self.bce[(ar + (ag as usize - 0x8000)) % self.bce.len()]
                        } else {
                            let ar = (lvk - 1) * 16384;
                            self.bce[(ar + (ag as usize - 0xC000)) % self.bce.len()]
                        }
                    }
                }
            }
            _ => 0,
        }
    }

    fn ujv(&mut self, ag: u16, ap: u8) {
        if ag < 0x8000 { return; }

        if ap & 0x80 != 0 {
            self.fnj = 0x10;
            self.gma = 0;
            self.glz |= 0x0C;
            return;
        }

        self.fnj = (self.fnj >> 1) | ((ap & 1) << 4);
        self.gma += 1;

        if self.gma == 5 {
            let bn = self.fnj;
            match ag {
                0x8000..=0x9FFF => {
                    self.glz = bn;
                    self.djo = match bn & 3 {
                        0 => Mirror::Bsv,
                        1 => Mirror::Bsw,
                        2 => Mirror::On,
                        _ => Mirror::Po,
                    };
                }
                0xA000..=0xBFFF => self.hqo = bn,
                0xC000..=0xDFFF => self.jef = bn,
                0xE000..=0xFFFF => self.jeg = bn & 0x0F,
                _ => {}
            }
            self.fnj = 0x10;
            self.gma = 0;
        }
    }

    fn ujw(&self, ag: u16) -> u8 {
        let rap = (self.glz >> 4) & 1;
        let w = if rap == 0 {
            
            let om = (self.hqo as usize & !1) * 4096;
            om + (ag as usize & 0x1FFF)
        } else {
            
            if ag < 0x1000 {
                (self.hqo as usize) * 4096 + (ag as usize & 0x0FFF)
            } else {
                (self.jef as usize) * 4096 + (ag as usize & 0x0FFF)
            }
        };
        if w < self.cpe.len() { self.cpe[w] } else { 0 }
    }

    

    fn ujx(&self, ag: u16) -> u8 {
        match ag {
            0x6000..=0x7FFF => self.gpu[(ag - 0x6000) as usize],
            0x8000..=0xBFFF => {
                let ar = (self.jeh as usize) * 16384;
                self.bce[(ar + (ag as usize - 0x8000)) % self.bce.len()]
            }
            0xC000..=0xFFFF => {
                let uca = (self.bce.len() / 16384).ao(1);
                let ar = uca * 16384;
                self.bce[(ar + (ag as usize - 0xC000)) % self.bce.len()]
            }
            _ => 0,
        }
    }

    fn ujy(&mut self, ag: u16, ap: u8) {
        if ag >= 0x8000 {
            self.jeh = ap;
        }
    }

    

    fn ujz(&self, ag: u16) -> u8 {
        self.lkj(ag) 
    }

    fn uka(&mut self, ag: u16, ap: u8) {
        if ag >= 0x8000 {
            self.jei = ap & 0x03;
        }
    }
}
