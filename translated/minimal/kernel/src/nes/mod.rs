

#![allow(bgr)]

pub mod cartridge;
pub mod cpu;
pub mod ppu;

use alloc::vec;
use alloc::vec::Vec;
use cartridge::Cartridge;
use cpu::{Cpu, Ch};
use ppu::Ppu;

const BBH_: usize = 256;
const CHL_: usize = 240;
const BQV_: u32 = 114; 


pub struct NesEmulator {
    cpu: Cpu,
    ppu: Ppu,
    ajl: [u8; 2048],
    on: Cartridge,
    dvf: bool,

    
    cwq: [u8; 2],
    cjr: [u8; 2],
    eoa: bool,

    
    dqb: u8,
    dgm: bool,

    
    hkj: bool,
    oo: u64,

    
    bbv: u8, 
}

struct Lf<'a> {
    ajl: &'a mut [u8; 2048],
    ppu: &'a mut Ppu,
    on: &'a mut Cartridge,
    cwq: &'a [u8; 2],
    cjr: &'a mut [u8; 2],
    eoa: &'a bool,
    dqb: &'a mut u8,
    dgm: &'a mut bool,
}

impl<'a> Ch for Lf<'a> {
    fn mc(&mut self, ag: u16) -> u8 {
        match ag {
            0x0000..=0x1FFF => self.ajl[(ag & 0x07FF) as usize],
            0x2000..=0x3FFF => self.ppu.gql(ag, self.on),
            0x4016 => {
                let ap = (self.cjr[0] & 0x80) >> 7;
                self.cjr[0] <<= 1;
                ap | 0x40
            }
            0x4017 => {
                let ap = (self.cjr[1] & 0x80) >> 7;
                self.cjr[1] <<= 1;
                ap | 0x40
            }
            0x4000..=0x4015 => 0, 
            0x4018..=0x5FFF => 0, 
            0x6000..=0xFFFF => self.on.mc(ag),
            _ => 0,
        }
    }

    fn ok(&mut self, ag: u16, ap: u8) {
        match ag {
            0x0000..=0x1FFF => self.ajl[(ag & 0x07FF) as usize] = ap,
            0x2000..=0x3FFF => self.ppu.ihl(ag, ap, self.on),
            0x4014 => {
                
                *self.dqb = ap;
                *self.dgm = true;
            }
            0x4016 => {
                if ap & 1 != 0 {
                    
                    self.cjr[0] = self.cwq[0];
                    self.cjr[1] = self.cwq[1];
                }
            }
            0x4000..=0x4015 | 0x4017 => {} 
            0x4018..=0x5FFF => {} 
            0x6000..=0xFFFF => self.on.ok(ag, ap),
            _ => {}
        }
    }
}

impl NesEmulator {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            ajl: [0; 2048],
            on: Cartridge::azs(),
            dvf: false,
            cwq: [0; 2],
            cjr: [0; 2],
            eoa: false,
            dqb: 0,
            dgm: false,
            hkj: false,
            oo: 0,
            bbv: 0,
        }
    }

    
    pub fn ljk(&mut self, f: &[u8]) -> bool {
        if let Some(on) = Cartridge::sxy(f) {
            self.on = on;
            self.dvf = true;
            
            self.cpu = Cpu::new();
            self.ppu = Ppu::new();
            self.ajl = [0; 2048];
            {
                let mut aq = Lf {
                    ajl: &mut self.ajl,
                    ppu: &mut self.ppu,
                    on: &mut self.on,
                    cwq: &self.cwq,
                    cjr: &mut self.cjr,
                    eoa: &self.eoa,
                    dqb: &mut self.dqb,
                    dgm: &mut self.dgm,
                };
                self.cpu.apa(&mut aq);
            }
            crate::serial_println!("[NES] ROM loaded, PC={:#06X}", self.cpu.fz);
            true
        } else {
            false
        }
    }

    fn ujl(&mut self) -> Lf<'_> {
        Lf {
            ajl: &mut self.ajl,
            ppu: &mut self.ppu,
            on: &mut self.on,
            cwq: &self.cwq,
            cjr: &mut self.cjr,
            eoa: &self.eoa,
            dqb: &mut self.dqb,
            dgm: &mut self.dgm,
        }
    }

    

    
    pub fn vr(&mut self, bs: u8) {
        match bs {
            b'x' | b'X' => self.bbv |= 0x80, 
            b'z' | b'Z' => self.bbv |= 0x40, 
            b'c' | b'C' => self.bbv |= 0x20, 
            b'\r' | 10  => self.bbv |= 0x10, 
            b'w' | b'W' | 0xF0 => self.bbv |= 0x08, 
            b's' | b'S' | 0xF1 => self.bbv |= 0x04, 
            b'a' | b'A' | 0xF2 => self.bbv |= 0x02, 
            b'd' | b'D' | 0xF3 => self.bbv |= 0x01, 
            b' '        => self.bbv |= 0x80, 
            _ => {}
        }
        self.cwq[0] = self.bbv;
    }

    pub fn avy(&mut self, bs: u8) {
        match bs {
            b'x' | b'X' => self.bbv &= !0x80,
            b'z' | b'Z' => self.bbv &= !0x40,
            b'c' | b'C' => self.bbv &= !0x20,
            b'\r' | 10  => self.bbv &= !0x10,
            b'w' | b'W' | 0xF0 => self.bbv &= !0x08,
            b's' | b'S' | 0xF1 => self.bbv &= !0x04,
            b'a' | b'A' | 0xF2 => self.bbv &= !0x02,
            b'd' | b'D' | 0xF3 => self.bbv &= !0x01,
            b' '        => self.bbv &= !0x80,
            _ => {}
        }
        self.cwq[0] = self.bbv;
    }

    

    
    pub fn or(&mut self) {
        if !self.dvf { return; }

        
        for _ in 0..262 {
            
            let mut mch: u32 = 0;
            while mch < BQV_ {
                
                if self.dgm {
                    self.dgm = false;
                    let ar = (self.dqb as u16) << 8;
                    let mut nmd = [0u8; 256];
                    for a in 0..256u16 {
                        let ag = ar | a;
                        nmd[a as usize] = match ag {
                            0x0000..=0x1FFF => self.ajl[(ag & 0x07FF) as usize],
                            0x6000..=0xFFFF => self.on.mc(ag),
                            _ => 0,
                        };
                    }
                    self.ppu.uwt(&nmd);
                    mch += 513;
                    continue;
                }

                let mut aq = Lf {
                    ajl: &mut self.ajl,
                    ppu: &mut self.ppu,
                    on: &mut self.on,
                    cwq: &self.cwq,
                    cjr: &mut self.cjr,
                    eoa: &self.eoa,
                    dqb: &mut self.dqb,
                    dgm: &mut self.dgm,
                };
                let yl = self.cpu.gu(&mut aq);
                mch += yl;
            }

            
            let evi = self.ppu.wud(&self.on);
            if evi {
                self.cpu.jhc = true;
            }
        }

        self.oo += 1;
    }

    

    
    pub fn tj(&self, an: &mut [u32], efz: usize, fpz: usize) {
        if !self.dvf {
            self.vwf(an, efz, fpz);
            return;
        }

        
        for qw in 0..fpz {
            let cq = qw * CHL_ / fpz;
            for mp in 0..efz {
                let cr = mp * BBH_ / efz;
                let s = self.ppu.framebuffer[cq * BBH_ + cr];
                an[qw * efz + mp] = s;
            }
        }
    }

    fn vwf(&self, an: &mut [u32], d: usize, i: usize) {
        
        for a in 0..d * i {
            an[a] = 0xFF0F0F23;
        }

        
        let dq = "TrustNES";
        let atp = "Insert ROM to play";
        let nfu = "WASD:Dpad X:A Z:B C:Select Enter:Start";

        let cx = d / 2;
        let ty = i / 3;
        self.cb(an, d, i, cx - dq.len() * 4, ty, dq, 0xFFFF4444);
        self.cb(an, d, i, cx - atp.len() * 4, ty + 30, atp, 0xFF888888);
        self.cb(an, d, i, cx - nfu.len() * 4, ty + 55, nfu, 0xFF666666);

        
        let ae = i / 2 + 30;
        let bx = cx - 30;
        
        for bg in 0..20u32 {
            for dx in 0..60u32 {
                let y = bx + dx as usize;
                let x = ae + bg as usize;
                if y < d && x < i {
                    an[x * d + y] = 0xFF333333;
                }
            }
        }
        
        for bc in 0..5u32 {
            let y = bx + 12; let x = ae + 5 + bc as usize;
            if y < d && x < i { an[x * d + y] = 0xFF666666; }
            let y = bx + 10 + bc as usize; let x = ae + 7;
            if y < d && x < i { an[x * d + y] = 0xFF666666; }
        }
        
        for axp in [bx + 42, bx + 50] {
            for bg in 0..4u32 {
                for dx in 0..4u32 {
                    let y = axp + dx as usize;
                    let x = ae + 6 + bg as usize;
                    if y < d && x < i {
                        an[x * d + y] = 0xFFCC2222;
                    }
                }
            }
        }
    }

    fn cb(&self, k: &mut [u32], d: usize, i: usize, b: usize, c: usize, text: &str, s: u32) {
        
        const Cdd: [u64; 128] = {
            let mut bb = [0u64; 128];
            bb[b'A' as usize] = 0x4A_EA_CE; bb[b'B' as usize] = 0xCA_CA_CE;
            bb[b'C' as usize] = 0x68_88_6E; bb[b'D' as usize] = 0xCA_AA_CE;
            bb[b'E' as usize] = 0xE8_C8_EE; bb[b'F' as usize] = 0xE8_C8_80;
            bb[b'G' as usize] = 0x68_A8_6E; bb[b'H' as usize] = 0xAA_EA_AE;
            bb[b'I' as usize] = 0xE4_44_EE; bb[b'J' as usize] = 0x22_2A_4E;
            bb[b'K' as usize] = 0xAA_CA_AE; bb[b'L' as usize] = 0x88_88_EE;
            bb[b'M' as usize] = 0xAE_EA_AE; bb[b'N' as usize] = 0xAE_EA_AE;
            bb[b'O' as usize] = 0x4A_AA_4E; bb[b'P' as usize] = 0xCA_C8_80;
            bb[b'Q' as usize] = 0x4A_AE_6E; bb[b'R' as usize] = 0xCA_CA_AE;
            bb[b'S' as usize] = 0x68_42_CE; bb[b'T' as usize] = 0xE4_44_40;
            bb[b'U' as usize] = 0xAA_AA_EE; bb[b'V' as usize] = 0xAA_AA_40;
            bb[b'W' as usize] = 0xAA_EE_AE; bb[b'X' as usize] = 0xAA_4A_AE;
            bb[b'Y' as usize] = 0xAA_44_40; bb[b'Z' as usize] = 0xE2_48_EE;
            bb[b'0' as usize] = 0x4A_AA_4E; bb[b'1' as usize] = 0x4C_44_EE;
            bb[b'2' as usize] = 0xC2_48_EE; bb[b'3' as usize] = 0xC2_42_CE;
            bb[b'4' as usize] = 0xAA_E2_2E; bb[b'5' as usize] = 0xE8_C2_CE;
            bb[b'6' as usize] = 0x68_CA_6E; bb[b'7' as usize] = 0xE2_24_40;
            bb[b'8' as usize] = 0x6A_4A_6E; bb[b'9' as usize] = 0x6A_62_CE;
            bb[b':' as usize] = 0x04_04_00; bb[b' ' as usize] = 0x00_00_00;
            bb[b'.' as usize] = 0x00_00_40;
            bb
        };

        let mut cx = b;
        for bm in text.bf() {
            let w = bm as usize;
            let ka = if w < 128 { Cdd[w] } else { 0 };
            if ka != 0 || bm == b' ' {
                for br in 0..5 {
                    for bj in 0..4 {
                        let ga = (ka >> (20 - br * 4 - bj)) & 1;
                        if ga != 0 {
                            let y = cx + bj as usize;
                            let x = c + br as usize;
                            if y < d && x < i {
                                k[x * d + y] = s;
                            }
                        }
                    }
                }
            }
            cx += 5;
        }
    }
}
