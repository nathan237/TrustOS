

#![allow(bgr)]

use alloc::vec;
use alloc::vec::Vec;

pub struct Cartridge {
    pub awv: Vec<u8>,
    pub ajl: Vec<u8>,
    pub fnz: MbcType,
    pub bwu: u16,
    pub brv: u8,
    pub ctr: bool,
    pub ev: u8, 
    pub dq: [u8; 16],
    pub eni: u8, 
}

#[derive(Clone, Copy, PartialEq)]
pub enum MbcType {
    None,   
    Acw,   
    Acx,   
    Acy,   
}

impl Cartridge {
    pub fn azs() -> Self {
        Self {
            awv: vec![0u8; 32768],
            ajl: vec![0u8; 8192],
            fnz: MbcType::None,
            bwu: 1,
            brv: 0,
            ctr: false,
            ev: 0,
            dq: [0; 16],
            eni: 0,
        }
    }

    pub fn syh(f: &[u8]) -> Option<Self> {
        if f.len() < 0x150 { return None; }

        
        let mut dq = [0u8; 16];
        for a in 0..16 {
            dq[a] = f[0x134 + a];
        }

        
        let eni = f[0x143];

        
        let nbt = f[0x147];
        let fnz = match nbt {
            0x00 | 0x08 | 0x09 => MbcType::None,
            0x01..=0x03 => MbcType::Acw,
            0x0F..=0x13 => MbcType::Acx,
            0x19..=0x1E => MbcType::Acy,
            _ => MbcType::None,
        };

        
        let mar = match f[0x148] {
            0 => 32 * 1024,
            1 => 64 * 1024,
            2 => 128 * 1024,
            3 => 256 * 1024,
            4 => 512 * 1024,
            5 => 1024 * 1024,
            6 => 2048 * 1024,
            7 => 4096 * 1024,
            _ => f.len(),
        };

        
        let cbf = match f[0x149] {
            0 => 0,
            1 => 2 * 1024,
            2 => 8 * 1024,
            3 => 32 * 1024,
            4 => 128 * 1024,
            5 => 64 * 1024,
            _ => 8 * 1024,
        };

        let awv = if f.len() >= mar {
            f[..mar].ip()
        } else {
            let mut m = f.ip();
            m.cmg(mar, 0);
            m
        };

        let ajl = vec![0u8; cbf.am(8192)];

        let mld: Vec<u8> = dq.iter().hu().fwc(|&r| r != 0 && r >= 0x20).collect();
        crate::serial_println!("[GB] ROM: \"{}\" type={:#04X} mbc={:?} ROM={}KB RAM={}KB CGB={:#04X}",
            core::str::jg(&mld).unwrap_or("???"),
            nbt,
            match fnz { MbcType::None => "None", MbcType::Acw => "MBC1", MbcType::Acx => "MBC3", MbcType::Acy => "MBC5" },
            awv.len() / 1024,
            cbf / 1024,
            eni);

        Some(Self {
            awv,
            ajl,
            fnz,
            bwu: 1,
            brv: 0,
            ctr: false,
            ev: 0,
            dq,
            eni,
        })
    }

    pub fn read(&self, ag: u16) -> u8 {
        match self.fnz {
            MbcType::None => self.umh(ag),
            MbcType::Acw => self.umj(ag),
            MbcType::Acx => self.uml(ag),
            MbcType::Acy => self.umn(ag),
        }
    }

    pub fn write(&mut self, ag: u16, ap: u8) {
        match self.fnz {
            MbcType::None => self.umi(ag, ap),
            MbcType::Acw => self.umk(ag, ap),
            MbcType::Acx => self.umm(ag, ap),
            MbcType::Acy => self.umo(ag, ap),
        }
    }

    
    fn umh(&self, ag: u16) -> u8 {
        match ag {
            0x0000..=0x7FFF => {
                if (ag as usize) < self.awv.len() { self.awv[ag as usize] } else { 0xFF }
            }
            0xA000..=0xBFFF => {
                let w = (ag - 0xA000) as usize;
                if w < self.ajl.len() { self.ajl[w] } else { 0xFF }
            }
            _ => 0xFF,
        }
    }
    fn umi(&mut self, ag: u16, ap: u8) {
        if ag >= 0xA000 && ag <= 0xBFFF {
            let w = (ag - 0xA000) as usize;
            if w < self.ajl.len() { self.ajl[w] = ap; }
        }
    }

    
    fn umj(&self, ag: u16) -> u8 {
        match ag {
            0x0000..=0x3FFF => {
                if self.ev == 1 {
                    let om = ((self.brv as usize & 3) << 5) % (self.awv.len() / 16384).am(1);
                    let w = om * 16384 + ag as usize;
                    if w < self.awv.len() { self.awv[w] } else { 0xFF }
                } else {
                    if (ag as usize) < self.awv.len() { self.awv[ag as usize] } else { 0xFF }
                }
            }
            0x4000..=0x7FFF => {
                let om = if self.bwu == 0 { 1 } else { self.bwu as usize };
                let sys = (om | ((self.brv as usize & 3) << 5)) % (self.awv.len() / 16384).am(1);
                let w = sys * 16384 + (ag as usize - 0x4000);
                if w < self.awv.len() { self.awv[w] } else { 0xFF }
            }
            0xA000..=0xBFFF => {
                if !self.ctr { return 0xFF; }
                let om = if self.ev == 1 { self.brv as usize } else { 0 };
                let w = om * 8192 + (ag as usize - 0xA000);
                if w < self.ajl.len() { self.ajl[w] } else { 0xFF }
            }
            _ => 0xFF,
        }
    }
    fn umk(&mut self, ag: u16, ap: u8) {
        match ag {
            0x0000..=0x1FFF => self.ctr = (ap & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                let mut om = (ap & 0x1F) as u16;
                if om == 0 { om = 1; }
                self.bwu = om;
            }
            0x4000..=0x5FFF => self.brv = ap & 3,
            0x6000..=0x7FFF => self.ev = ap & 1,
            0xA000..=0xBFFF => {
                if !self.ctr { return; }
                let om = if self.ev == 1 { self.brv as usize } else { 0 };
                let w = om * 8192 + (ag as usize - 0xA000);
                if w < self.ajl.len() { self.ajl[w] = ap; }
            }
            _ => {}
        }
    }

    
    fn uml(&self, ag: u16) -> u8 {
        match ag {
            0x0000..=0x3FFF => {
                if (ag as usize) < self.awv.len() { self.awv[ag as usize] } else { 0xFF }
            }
            0x4000..=0x7FFF => {
                let om = if self.bwu == 0 { 1 } else { self.bwu as usize };
                let w = (om % (self.awv.len() / 16384).am(1)) * 16384 + (ag as usize - 0x4000);
                if w < self.awv.len() { self.awv[w] } else { 0xFF }
            }
            0xA000..=0xBFFF => {
                if !self.ctr { return 0xFF; }
                if self.brv <= 3 {
                    let w = self.brv as usize * 8192 + (ag as usize - 0xA000);
                    if w < self.ajl.len() { self.ajl[w] } else { 0xFF }
                } else {
                    0 
                }
            }
            _ => 0xFF,
        }
    }
    fn umm(&mut self, ag: u16, ap: u8) {
        match ag {
            0x0000..=0x1FFF => self.ctr = (ap & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                let om = (ap & 0x7F) as u16;
                self.bwu = if om == 0 { 1 } else { om };
            }
            0x4000..=0x5FFF => self.brv = ap,
            0x6000..=0x7FFF => {} 
            0xA000..=0xBFFF => {
                if !self.ctr { return; }
                if self.brv <= 3 {
                    let w = self.brv as usize * 8192 + (ag as usize - 0xA000);
                    if w < self.ajl.len() { self.ajl[w] = ap; }
                }
            }
            _ => {}
        }
    }

    
    fn umn(&self, ag: u16) -> u8 {
        match ag {
            0x0000..=0x3FFF => {
                if (ag as usize) < self.awv.len() { self.awv[ag as usize] } else { 0xFF }
            }
            0x4000..=0x7FFF => {
                let om = self.bwu as usize;
                let w = (om % (self.awv.len() / 16384).am(1)) * 16384 + (ag as usize - 0x4000);
                if w < self.awv.len() { self.awv[w] } else { 0xFF }
            }
            0xA000..=0xBFFF => {
                if !self.ctr { return 0xFF; }
                let w = (self.brv as usize & 0x0F) * 8192 + (ag as usize - 0xA000);
                if w < self.ajl.len() { self.ajl[w] } else { 0xFF }
            }
            _ => 0xFF,
        }
    }
    fn umo(&mut self, ag: u16, ap: u8) {
        match ag {
            0x0000..=0x1FFF => self.ctr = (ap & 0x0F) == 0x0A,
            0x2000..=0x2FFF => self.bwu = (self.bwu & 0x100) | ap as u16,
            0x3000..=0x3FFF => self.bwu = (self.bwu & 0xFF) | (((ap & 1) as u16) << 8),
            0x4000..=0x5FFF => self.brv = ap & 0x0F,
            0xA000..=0xBFFF => {
                if !self.ctr { return; }
                let w = (self.brv as usize & 0x0F) * 8192 + (ag as usize - 0xA000);
                if w < self.ajl.len() { self.ajl[w] = ap; }
            }
            _ => {}
        }
    }
}
