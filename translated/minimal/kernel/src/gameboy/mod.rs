

#![allow(bgr)]

use alloc::vec;
use alloc::vec::Vec;
pub mod cartridge;
pub mod gpu;
pub mod timer;
pub mod cpu;

use cpu::Kn;


const BVM_: u32 = 17556;

pub struct GameBoyEmulator {
    pub cpu: cpu::Cpu,
    pub gpu: gpu::Gpu,
    pub timer: timer::Timer,
    pub on: cartridge::Cartridge,

    
    pub aec: Vec<u8>,       
    pub bux: [u8; 127],    

    
    pub brc: u8,          
    pub bhf: u8,          
    pub cfu: u8,      
    pub aow: u8,  
    pub aox: u8,     
    pub cht: u8,     
    pub chs: u8,     

    pub dvf: bool,
    pub bbv: u32,

    
    pub atz: bool,
    pub axj: u8,       
    pub beq: u8,            
    pub ecn: u8,           
    pub eco: u8,           
    pub ecp: u8,           
    pub ecq: u8,           
    pub ecr: u8,           
    pub fkj: bool,
}

impl GameBoyEmulator {
    pub fn new() -> Self {
        Self {
            cpu: cpu::Cpu::new(),
            gpu: gpu::Gpu::new(),
            timer: timer::Timer::new(),
            on: cartridge::Cartridge::azs(),
            aec: vec![0u8; 32768], 
            bux: [0; 127],
            brc: 0,
            bhf: 0,
            cfu: 0xCF,
            aow: 0x0F,
            aox: 0x0F,
            cht: 0,
            chs: 0,
            dvf: false,
            bbv: 0,
            atz: false,
            axj: 1,
            beq: 0,
            ecn: 0xFF,
            eco: 0xFF,
            ecp: 0xFF,
            ecq: 0xFF,
            ecr: 0xFF,
            fkj: false,
        }
    }

    pub fn ljk(&mut self, f: &[u8]) -> bool {
        if let Some(on) = cartridge::Cartridge::syh(f) {
            
            let hov = on.eni == 0x80 || on.eni == 0xC0;
            self.atz = hov;
            self.on = on;
            self.cpu = cpu::Cpu::new();
            if hov {
                
                self.cpu.q = 0x11;
                self.cpu.bb = 0x80; 
                self.cpu.o = 0x00;
                self.cpu.r = 0x00;
                self.cpu.bc = 0xFF;
                self.cpu.aa = 0x56;
                self.cpu.i = 0x00;
                self.cpu.dm = 0x0D;
                crate::serial_println!("[GB] CGB mode enabled (A=0x11)");
            }
            self.gpu = gpu::Gpu::new();
            self.gpu.atz = hov;
            if hov {
                
                
                self.gpu.bdo[0] = 0xFF; self.gpu.bdo[1] = 0x7F;
                
                self.gpu.bdo[2] = 0xB5; self.gpu.bdo[3] = 0x56;
                
                self.gpu.bdo[4] = 0x4A; self.gpu.bdo[5] = 0x29;
                
                self.gpu.bdo[6] = 0x00; self.gpu.bdo[7] = 0x00;
                
                for a in 0..8 {
                    self.gpu.fpk[a] = self.gpu.bdo[a];
                }
            }
            self.timer = timer::Timer::new();
            for o in self.aec.el() { *o = 0; }
            self.bux = [0; 127];
            self.brc = 0;
            self.bhf = 0;
            self.axj = 1;
            self.beq = 0;
            self.fkj = false;
            self.dvf = true;
            crate::serial_println!("[GB] ROM loaded successfully (CGB={})", hov);
            true
        } else {
            crate::serial_println!("[GB] Failed to load ROM");
            false
        }
    }

    fn ujl(&mut self) -> Lf<'_> {
        Lf {
            aec: &mut self.aec,
            bux: &mut self.bux,
            gpu: &mut self.gpu,
            timer: &mut self.timer,
            on: &mut self.on,
            brc: &mut self.brc,
            bhf: &mut self.bhf,
            cfu: &mut self.cfu,
            aow: &self.aow,
            aox: &self.aox,
            cht: &mut self.cht,
            chs: &mut self.chs,
            atz: self.atz,
            axj: &mut self.axj,
            beq: &mut self.beq,
            ecn: &mut self.ecn,
            eco: &mut self.eco,
            ecp: &mut self.ecp,
            ecq: &mut self.ecq,
            ecr: &mut self.ecr,
            fkj: &mut self.fkj,
        }
    }

    
    
    
    pub fn vr(&mut self, bs: u8) {
        match bs {
            b'd' | b'D' | 0xF3 => self.aox &= !0x01, 
            b'a' | b'A' | 0xF2 => self.aox &= !0x02, 
            b'w' | b'W' | 0xF0 => self.aox &= !0x04, 
            b's' | b'S' | 0xF1 => self.aox &= !0x08, 
            b'x' | b'X' | b' ' => self.aow &= !0x01, 
            b'z' | b'Z'        => self.aow &= !0x02, 
            b'\r' | 10         => self.aow &= !0x08, 
            b'c' | b'C'        => self.aow &= !0x04, 
            _ => {}
        }
        self.bhf |= 0x10; 
    }

    pub fn avy(&mut self, bs: u8) {
        match bs {
            b'd' | b'D' | 0xF3 => self.aox |= 0x01,
            b'a' | b'A' | 0xF2 => self.aox |= 0x02,
            b'w' | b'W' | 0xF0 => self.aox |= 0x04,
            b's' | b'S' | 0xF1 => self.aox |= 0x08,
            b'x' | b'X' | b' ' => self.aow |= 0x01,
            b'z' | b'Z'        => self.aow |= 0x02,
            b'\r' | 10         => self.aow |= 0x08,
            b'c' | b'C'        => self.aow |= 0x04,
            _ => {}
        }
    }

    
    pub fn or(&mut self) {
        if !self.dvf { return; }

        self.gpu.hkj = false;
        let mut nvy: u32 = 0;
        let mut pfe: u32 = 0;
        const CFI_: u32 = 200_000; 

        while nvy < BVM_ {
            pfe += 1;
            if pfe > CFI_ {
                
                break;
            }

            let ef = {
                let mut aq = Lf {
                    aec: &mut self.aec,
                    bux: &mut self.bux,
                    gpu: &mut self.gpu,
                    timer: &mut self.timer,
                    on: &mut self.on,
                    brc: &mut self.brc,
                    bhf: &mut self.bhf,
                    cfu: &mut self.cfu,
                    aow: &self.aow,
                    aox: &self.aox,
                    cht: &mut self.cht,
                    chs: &mut self.chs,
                    atz: self.atz,
                    axj: &mut self.axj,
                    beq: &mut self.beq,
                    ecn: &mut self.ecn,
                    eco: &mut self.eco,
                    ecp: &mut self.ecp,
                    ecq: &mut self.ecq,
                    ecr: &mut self.ecr,
                    fkj: &mut self.fkj,
                };
                self.cpu.gu(&mut aq)
            };

            
            self.gpu.gu(ef);
            self.timer.gu(ef);

            
            if self.gpu.jvi {
                self.bhf |= 0x01;
                self.gpu.jvi = false;
            }
            if self.gpu.eza {
                self.bhf |= 0x02;
                self.gpu.eza = false;
            }
            if self.timer.gkb {
                self.bhf |= 0x04;
                self.timer.gkb = false;
            }

            nvy += ef;
        }
    }

    
    pub fn tj(&self, bd: &mut [u32], efz: usize, fpz: usize) {
        if !self.dvf {
            self.vwe(bd, efz, fpz);
            return;
        }

        let jri = gpu::EQ_;
        let mhc = gpu::AHM_;

        for c in 0..fpz {
            let cq = c * mhc / fpz;
            for b in 0..efz {
                let cr = b * jri / efz;
                let si = cq * jri + cr;
                bd[c * efz + b] = if si < self.gpu.framebuffer.len() {
                    self.gpu.framebuffer[si]
                } else {
                    0xFF081820
                };
            }
        }
    }

    fn vwe(&self, bd: &mut [u32], d: usize, i: usize) {
        let ei = 0xFF081820u32;  
        let lp = 0xFFE0F8D0u32;  
        let csn = 0xFF346856u32;  

        for ai in bd.el() { *ai = ei; }

        
        cb(bd, d, i, "GAME BOY", d / 2 - 32, i / 6, lp, 2);
        cb(bd, d, i, "EMULATOR", d / 2 - 32, i / 6 + 20, csn, 2);

        
        cb(bd, d, i, "INSERT ROM", d / 2 - 40, i / 2 - 10, lp, 2);

        
        let cx = d / 2;
        let je = i * 5 / 8;
        let nm = 60usize;
        let adn = 80usize;
        for b in (cx - nm/2)..=(cx + nm/2) {
            if b < d {
                if je < i { bd[je * d + b] = csn; }
                if je + adn < i { bd[(je + adn) * d + b] = csn; }
            }
        }
        for c in je..=(je + adn) {
            if c < i {
                if cx - nm/2 < d { bd[c * d + (cx - nm/2)] = csn; }
                if cx + nm/2 < d { bd[c * d + (cx + nm/2)] = csn; }
            }
        }
        
        let kp = 40usize;
        let kl = 36usize;
        let cr = cx - kp / 2;
        let cq = je + 8;
        for c in cq..(cq + kl).v(i) {
            for b in cr..(cr + kp).v(d) {
                bd[c * d + b] = 0xFF88C070;
            }
        }

        
        cb(bd, d, i, "WASD:DPAD", d / 2 - 36, i - 50, csn, 1);
        cb(bd, d, i, "X:A Z:B ENTER:START", d / 2 - 72, i - 38, csn, 1);
    }
}


struct Lf<'a> {
    aec: &'a mut Vec<u8>,
    bux: &'a mut [u8; 127],
    gpu: &'a mut gpu::Gpu,
    timer: &'a mut timer::Timer,
    on: &'a mut cartridge::Cartridge,
    brc: &'a mut u8,
    bhf: &'a mut u8,
    cfu: &'a mut u8,
    aow: &'a u8,
    aox: &'a u8,
    cht: &'a mut u8,
    chs: &'a mut u8,
    
    atz: bool,
    axj: &'a mut u8,
    beq: &'a mut u8,
    ecn: &'a mut u8,
    eco: &'a mut u8,
    ecp: &'a mut u8,
    ecq: &'a mut u8,
    ecr: &'a mut u8,
    fkj: &'a mut bool,
}

impl Kn for Lf<'_> {
    fn read(&mut self, ag: u16) -> u8 {
        match ag {
            
            0x0000..=0x7FFF => self.on.read(ag),
            
            0x8000..=0x9FFF => self.gpu.jlp(ag),
            
            0xA000..=0xBFFF => self.on.read(ag),
            
            0xC000..=0xCFFF => self.aec[(ag as usize - 0xC000)],
            
            0xD000..=0xDFFF => {
                let om = if self.atz { (*self.axj).am(1) as usize } else { 1 };
                let l = om * 0x1000 + (ag as usize - 0xD000);
                if l < self.aec.len() { self.aec[l] } else { 0xFF }
            },
            
            0xE000..=0xEFFF => self.aec[(ag as usize - 0xE000)],
            0xF000..=0xFDFF => {
                let om = if self.atz { (*self.axj).am(1) as usize } else { 1 };
                let l = om * 0x1000 + (ag as usize - 0xF000);
                if l < self.aec.len() { self.aec[l] } else { 0xFF }
            },
            
            0xFE00..=0xFE9F => self.gpu.pai(ag),
            
            0xFEA0..=0xFEFF => 0xFF,
            
            0xFF00 => {
                let mut ap = *self.cfu & 0x30;
                if ap & 0x10 == 0 { ap |= *self.aox; }
                if ap & 0x20 == 0 { ap |= *self.aow; }
                ap | 0xC0
            }
            0xFF01 => *self.cht,
            0xFF02 => *self.chs,
            0xFF04 => self.timer.pac(),
            0xFF05 => self.timer.ejw,
            0xFF06 => self.timer.fww,
            0xFF07 => self.timer.ezl,
            0xFF0F => *self.bhf,
            
            0xFF10..=0xFF3F => 0xFF,
            
            0xFF40 => self.gpu.amh,
            0xFF41 => self.gpu.vso(),
            0xFF42 => self.gpu.eyf,
            0xFF43 => self.gpu.eye,
            0xFF44 => if self.gpu.amh & 0x80 != 0 { self.gpu.ct } else { 0 },
            0xFF45 => self.gpu.eey,
            0xFF46 => 0, 
            0xFF47 => self.gpu.emt,
            0xFF48 => self.gpu.fpm,
            0xFF49 => self.gpu.fpn,
            0xFF4A => self.gpu.lw,
            0xFF4B => self.gpu.fx,
            
            0xFF4D => *self.beq,                      
            0xFF4F => self.gpu.fbb | 0xFE,       
            0xFF51 => *self.ecn,
            0xFF52 => *self.eco,
            0xFF53 => *self.ecp,
            0xFF54 => *self.ecq,
            0xFF55 => *self.ecr,
            0xFF68 => self.gpu.doj,                   
            0xFF69 => self.gpu.vrg(),            
            0xFF6A => self.gpu.dtv,                   
            0xFF6B => self.gpu.vse(),            
            0xFF70 => *self.axj,                 
            
            0xFF80..=0xFFFE => self.bux[(ag - 0xFF80) as usize],
            
            0xFFFF => *self.brc,
            _ => 0xFF,
        }
    }

    fn write(&mut self, ag: u16, ap: u8) {
        match ag {
            
            0x0000..=0x7FFF => self.on.write(ag, ap),
            
            0x8000..=0x9FFF => self.gpu.mrd(ag, ap),
            
            0xA000..=0xBFFF => self.on.write(ag, ap),
            
            0xC000..=0xCFFF => self.aec[(ag as usize - 0xC000)] = ap,
            
            0xD000..=0xDFFF => {
                let om = if self.atz { (*self.axj).am(1) as usize } else { 1 };
                let l = om * 0x1000 + (ag as usize - 0xD000);
                if l < self.aec.len() { self.aec[l] = ap; }
            },
            
            0xE000..=0xEFFF => self.aec[(ag as usize - 0xE000)] = ap,
            0xF000..=0xFDFF => {
                let om = if self.atz { (*self.axj).am(1) as usize } else { 1 };
                let l = om * 0x1000 + (ag as usize - 0xF000);
                if l < self.aec.len() { self.aec[l] = ap; }
            },
            
            0xFE00..=0xFE9F => self.gpu.pzz(ag, ap),
            
            0xFEA0..=0xFEFF => {}
            
            0xFF00 => *self.cfu = ap & 0x30,
            0xFF01 => *self.cht = ap,
            0xFF02 => *self.chs = ap,
            0xFF04 => self.timer.xvi(),
            0xFF05 => self.timer.ejw = ap,
            0xFF06 => self.timer.fww = ap,
            0xFF07 => self.timer.ezl = ap,
            0xFF0F => *self.bhf = ap,
            
            0xFF10..=0xFF3F => {}
            
            0xFF40 => {
                let aft = self.gpu.amh;
                self.gpu.amh = ap;
                
                if ap & 0x80 != 0 && aft & 0x80 == 0 {
                    self.gpu.ct = 0;
                    self.gpu.yl = 0;
                    self.gpu.ev = 2;
                    self.gpu.ekz = 0;
                }
            }
            0xFF41 => self.gpu.hm = (self.gpu.hm & 0x07) | (ap & 0xF8),
            0xFF42 => self.gpu.eyf = ap,
            0xFF43 => self.gpu.eye = ap,
            0xFF44 => {} 
            0xFF45 => self.gpu.eey = ap,
            0xFF46 => {
                
                let ar = (ap as u16) << 8;
                for a in 0..160u16 {
                    let hf = match ar + a {
                        q @ 0x0000..=0x7FFF => self.on.read(q),
                        q @ 0x8000..=0x9FFF => self.gpu.jlp(q),
                        q @ 0xA000..=0xBFFF => self.on.read(q),
                        q @ 0xC000..=0xCFFF => self.aec[(q as usize - 0xC000)],
                        q @ 0xD000..=0xDFFF => {
                            let om = if self.atz { (*self.axj).am(1) as usize } else { 1 };
                            let l = om * 0x1000 + (q as usize - 0xD000);
                            if l < self.aec.len() { self.aec[l] } else { 0 }
                        },
                        _ => 0,
                    };
                    self.gpu.pzz(0xFE00 + a, hf);
                }
            }
            0xFF47 => self.gpu.emt = ap,
            0xFF48 => self.gpu.fpm = ap,
            0xFF49 => self.gpu.fpn = ap,
            0xFF4A => self.gpu.lw = ap,
            0xFF4B => self.gpu.fx = ap,
            
            0xFF4D => *self.beq = (*self.beq & 0x80) | (ap & 0x01), 
            0xFF4F => self.gpu.fbb = ap & 0x01,                  
            0xFF51 => *self.ecn = ap,
            0xFF52 => *self.eco = ap & 0xF0,
            0xFF53 => *self.ecp = ap & 0x1F,
            0xFF54 => *self.ecq = ap & 0xF0,
            0xFF55 => {
                
                if self.atz {
                    let cy = ((*self.ecn as u16) << 8) | (*self.eco as u16);
                    let cs = 0x8000 | (((*self.ecp as u16) << 8) | (*self.ecq as u16));
                    let len = ((ap as u16 & 0x7F) + 1) * 16;
                    
                    if ap & 0x80 == 0 {
                        
                        for a in 0..len {
                            let hf = match cy.cn(a) {
                                q @ 0x0000..=0x7FFF => self.on.read(q),
                                q @ 0x8000..=0x9FFF => self.gpu.jlp(q),
                                q @ 0xA000..=0xBFFF => self.on.read(q),
                                q @ 0xC000..=0xCFFF => self.aec[(q as usize - 0xC000)],
                                q @ 0xD000..=0xDFFF => {
                                    let om = (*self.axj).am(1) as usize;
                                    let l = om * 0x1000 + (q as usize - 0xD000);
                                    if l < self.aec.len() { self.aec[l] } else { 0 }
                                },
                                _ => 0xFF,
                            };
                            self.gpu.mrd(cs.cn(a), hf);
                        }
                        *self.ecr = 0xFF; 
                    } else {
                        
                        for a in 0..len {
                            let hf = match cy.cn(a) {
                                q @ 0x0000..=0x7FFF => self.on.read(q),
                                q @ 0xA000..=0xBFFF => self.on.read(q),
                                q @ 0xC000..=0xCFFF => self.aec[(q as usize - 0xC000)],
                                q @ 0xD000..=0xDFFF => {
                                    let om = (*self.axj).am(1) as usize;
                                    let l = om * 0x1000 + (q as usize - 0xD000);
                                    if l < self.aec.len() { self.aec[l] } else { 0 }
                                },
                                _ => 0xFF,
                            };
                            self.gpu.mrd(cs.cn(a), hf);
                        }
                        *self.ecr = 0xFF;
                    }
                }
            }
            0xFF68 => self.gpu.doj = ap,               
            0xFF69 => self.gpu.xvh(ap),           
            0xFF6A => self.gpu.dtv = ap,               
            0xFF6B => self.gpu.xvo(ap),           
            0xFF70 => {
                
                *self.axj = ap & 0x07;
                if *self.axj == 0 { *self.axj = 1; }
            }
            
            0xFF80..=0xFFFE => self.bux[(ag - 0xFF80) as usize] = ap,
            
            0xFFFF => *self.brc = ap,
            _ => {}
        }
    }
}


fn cb(bd: &mut [u32], d: usize, i: usize, text: &str, b: usize, c: usize, s: u32, bv: usize) {
    let mut cx = b;
    for bm in text.bf() {
        let ka = ada(bm);
        for br in 0..5usize {
            for bj in 0..3usize {
                if ka[br] & (1 << (2 - bj)) != 0 {
                    for cq in 0..bv {
                        for cr in 0..bv {
                            let y = cx + bj * bv + cr;
                            let x = c + br * bv + cq;
                            if y < d && x < i { bd[x * d + y] = s; }
                        }
                    }
                }
            }
        }
        cx += (3 + 1) * bv;
    }
}

fn ada(bm: u8) -> [u8; 5] {
    match bm {
        b'A' => [0b111, 0b101, 0b111, 0b101, 0b101],
        b'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        b'C' => [0b111, 0b100, 0b100, 0b100, 0b111],
        b'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        b'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        b'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        b'G' => [0b111, 0b100, 0b101, 0b101, 0b111],
        b'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        b'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        b'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
        b'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        b'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        b'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        b'N' => [0b101, 0b111, 0b111, 0b111, 0b101],
        b'O' => [0b111, 0b101, 0b101, 0b101, 0b111],
        b'P' => [0b111, 0b101, 0b111, 0b100, 0b100],
        b'Q' => [0b111, 0b101, 0b101, 0b111, 0b001],
        b'R' => [0b111, 0b101, 0b111, 0b110, 0b101],
        b'S' => [0b111, 0b100, 0b111, 0b001, 0b111],
        b'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        b'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        b'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        b'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        b'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        b'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        b'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        b'0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        b'1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        b'2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        b'3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        b'4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        b'5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        b'6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        b'7' => [0b111, 0b001, 0b001, 0b001, 0b001],
        b'8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        b'9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        b':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        b' ' => [0b000, 0b000, 0b000, 0b000, 0b000],
        _ => [0b111, 0b111, 0b111, 0b111, 0b111],
    }
}
