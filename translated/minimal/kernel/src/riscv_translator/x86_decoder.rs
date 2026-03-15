

















use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use super::ir::*;


fn ahb(xwf: u8) -> Reg {
    match xwf {
        0  => Reg::Je,  
        1  => Reg::Zo,  
        2  => Reg::Afw,  
        3  => Reg::Va,  
        4  => Reg::Ds,   
        5  => Reg::Aon,   
        6  => Reg::Vb,  
        7  => Reg::Vc,  
        8  => Reg::Afx,  
        9  => Reg::Afy,  
        10 => Reg::Afz,  
        11 => Reg::Aog,  
        12 => Reg::Aoh,  
        13 => Reg::Aoi,  
        14 => Reg::Aoj,  
        15 => Reg::Aok,  
        _  => Reg::Bg,   
    }
}


pub struct X86Decoder {
    
    aj: Vec<u8>,
    
    sm: u64,
    
    l: usize,
    
    pub cm: TranslationStats,
}

impl X86Decoder {
    pub fn new(aj: &[u8], sm: u64) -> Self {
        Self {
            aj: aj.ip(),
            sm,
            l: 0,
            cm: TranslationStats::default(),
        }
    }

    
    pub fn mmw(&mut self, cun: usize) -> TranslatedBlock {
        self.l = cun;
        let cbz = self.sm + cun as u64;
        let mut block = TranslatedBlock::new(cbz, SourceArch::BT_);

        let eff = 256;
        let mut az = 0;

        while self.l < self.aj.len() && az < eff {
            let yyg = self.l;
            let mkf = self.hfp(&mut block);
            block.jrg += 1;
            self.cm.esv += 1;
            az += 1;

            if mkf {
                break;
            }
        }

        self.cm.ilv += 1;
        self.cm.hyi += block.instructions.len() as u64;
        block
    }

    
    pub fn iev(&mut self) -> Vec<TranslatedBlock> {
        let mut xk = Vec::new();
        let mut fyw: Vec<usize> = Vec::new();
        let mut bxs: Vec<u64> = Vec::new();

        fyw.push(0);

        while let Some(l) = fyw.pop() {
            let ag = self.sm + l as u64;
            if bxs.contains(&ag) {
                continue;
            }
            bxs.push(ag);

            let block = self.mmw(l);

            
            for &fvt in &block.bil {
                if fvt >= self.sm {
                    let ice = (fvt - self.sm) as usize;
                    if ice < self.aj.len() && !bxs.contains(&fvt) {
                        fyw.push(ice);
                    }
                }
            }

            xk.push(block);
        }

        xk
    }

    
    
    fn hfp(&mut self, block: &mut TranslatedBlock) -> bool {
        if self.l >= self.aj.len() {
            return true;
        }

        let dik = self.sm + self.l as u64;

        
        let mut aip: u8 = 0;
        let mut kf = false;
        let mut uyr = false;

        loop {
            if self.l >= self.aj.len() { return true; }
            let o = self.aj[self.l];
            match o {
                0x66 => { uyr = true; self.l += 1; }
                0x40..=0x4F => { aip = o; kf = true; self.l += 1; }
                0xF0 | 0xF2 | 0xF3 | 0x2E | 0x3E | 0x26 | 0x64 | 0x65 | 0x36 => {
                    self.l += 1; 
                }
                _ => break,
            }
        }

        if self.l >= self.aj.len() { return true; }

        let ako = kf && (aip & 0x08) != 0;
        let nx = kf && (aip & 0x04) != 0;
        let pg = kf && (aip & 0x02) != 0;
        let ic = kf && (aip & 0x01) != 0;

        let opcode = self.aj[self.l];
        self.l += 1;

        match opcode {
            
            0x90 => {
                block.fj(RvInst::Fq);
                false
            }

            
            0xC7 => {
                let (hb, _) = self.bmt(ic, nx);
                let gf = self.amq() as i64;
                let ck = ahb(hb);
                block.fj(RvInst::Hu { ck, gf });
                false
            }

            
            0xB8..=0xBF => {
                let reg = (opcode - 0xB8) + if ic { 8 } else { 0 };
                let ck = ahb(reg);
                let gf = if ako {
                    self.jll()
                } else {
                    self.amq() as i64
                };
                block.fj(RvInst::Hu { ck, gf });
                false
            }

            
            0xB0..=0xB7 => {
                let reg = (opcode - 0xB0) + if ic { 8 } else { 0 };
                let ck = ahb(reg);
                let gf = self.ady() as i64;
                
                block.fj(RvInst::Ou { ck, cp: ck, gf: !0xFF });
                block.fj(RvInst::Akw { ck, cp: ck, gf: gf & 0xFF });
                false
            }

            
            0x89 => {
                let (hb, reg) = self.bmt(ic, nx);
                let acl = ahb(reg);
                let ck = ahb(hb);
                block.fj(RvInst::Gl { ck, acl });
                false
            }
            0x8B => {
                let (hb, reg) = self.bmt(ic, nx);
                let acl = ahb(hb);
                let ck = ahb(reg);
                block.fj(RvInst::Gl { ck, acl });
                false
            }

            
            0x50..=0x57 => {
                let reg = (opcode - 0x50) + if ic { 8 } else { 0 };
                let acl = ahb(reg);
                
                block.fj(RvInst::Gf { ck: Reg::Ds, cp: Reg::Ds, gf: -8 });
                block.fj(RvInst::Mi { et: acl, cp: Reg::Ds, l: 0 });
                false
            }

            
            0x58..=0x5F => {
                let reg = (opcode - 0x58) + if ic { 8 } else { 0 };
                let ck = ahb(reg);
                
                block.fj(RvInst::Pt { ck, cp: Reg::Ds, l: 0 });
                block.fj(RvInst::Gf { ck: Reg::Ds, cp: Reg::Ds, gf: 8 });
                false
            }

            
            0x01 => {
                let (hb, reg) = self.bmt(ic, nx);
                let ck = ahb(hb);
                let acl = ahb(reg);
                block.fj(RvInst::Add { ck, cp: ck, et: acl });
                block.fj(RvInst::Ed { cp: ck, et: Reg::Bt });
                false
            }

            
            0x03 => {
                let (hb, reg) = self.bmt(ic, nx);
                let ck = ahb(reg);
                let acl = ahb(hb);
                block.fj(RvInst::Add { ck, cp: ck, et: acl });
                block.fj(RvInst::Ed { cp: ck, et: Reg::Bt });
                false
            }

            
            0x29 => {
                let (hb, reg) = self.bmt(ic, nx);
                let ck = ahb(hb);
                let acl = ahb(reg);
                block.fj(RvInst::Sub { ck, cp: ck, et: acl });
                block.fj(RvInst::Ed { cp: ck, et: Reg::Bt });
                false
            }

            
            0x2B => {
                let (hb, reg) = self.bmt(ic, nx);
                let ck = ahb(reg);
                let acl = ahb(hb);
                block.fj(RvInst::Sub { ck, cp: ck, et: acl });
                block.fj(RvInst::Ed { cp: ck, et: Reg::Bt });
                false
            }

            
            0x21 => {
                let (hb, reg) = self.bmt(ic, nx);
                let ck = ahb(hb);
                let acl = ahb(reg);
                block.fj(RvInst::Ex { ck, cp: ck, et: acl });
                block.fj(RvInst::Ed { cp: ck, et: Reg::Bt });
                false
            }

            
            0x09 => {
                let (hb, reg) = self.bmt(ic, nx);
                let ck = ahb(hb);
                let acl = ahb(reg);
                block.fj(RvInst::Fx { ck, cp: ck, et: acl });
                block.fj(RvInst::Ed { cp: ck, et: Reg::Bt });
                false
            }

            
            0x31 => {
                let (hb, reg) = self.bmt(ic, nx);
                let ck = ahb(hb);
                let acl = ahb(reg);
                block.fj(RvInst::Aga { ck, cp: ck, et: acl });
                block.fj(RvInst::Ed { cp: ck, et: Reg::Bt });
                false
            }

            
            0x39 => {
                let (hb, reg) = self.bmt(ic, nx);
                let aww = ahb(hb);
                let hyj = ahb(reg);
                
                block.fj(RvInst::Ed { cp: aww, et: hyj });
                false
            }

            
            0x3B => {
                let (hb, reg) = self.bmt(ic, nx);
                let aww = ahb(hb);
                let hyj = ahb(reg);
                block.fj(RvInst::Ed { cp: hyj, et: aww });
                false
            }

            
            0x83 => {
                let (hb, lqq) = self.bmt(ic, nx);
                let gf = self.cmd() as i64;
                let ck = ahb(hb);
                match lqq {
                    0 => { 
                        block.fj(RvInst::Gf { ck, cp: ck, gf });
                        block.fj(RvInst::Ed { cp: ck, et: Reg::Bt });
                    }
                    4 => { 
                        block.fj(RvInst::Ou { ck, cp: ck, gf });
                        block.fj(RvInst::Ed { cp: ck, et: Reg::Bt });
                    }
                    5 => { 
                        block.fj(RvInst::Gf { ck, cp: ck, gf: -gf });
                        block.fj(RvInst::Ed { cp: ck, et: Reg::Bt });
                    }
                    7 => { 
                        block.fj(RvInst::Hu { ck: Reg::Bg, gf });
                        block.fj(RvInst::Ed { cp: ck, et: Reg::Bg });
                    }
                    _ => {
                        self.cm.ddf += 1;
                        block.fj(RvInst::Fq);
                    }
                }
                false
            }

            
            0x85 => {
                let (hb, reg) = self.bmt(ic, nx);
                let aww = ahb(hb);
                let hyj = ahb(reg);
                
                block.fj(RvInst::Ex { ck: Reg::Bg, cp: aww, et: hyj });
                block.fj(RvInst::Ed { cp: Reg::Bg, et: Reg::Bt });
                false
            }

            
            0x8D => {
                let (hb, reg) = self.bmt(ic, nx);
                let ck = ahb(reg);
                let acl = ahb(hb);
                
                block.fj(RvInst::Gl { ck, acl });
                false
            }

            
            0xFF => {
                let (hb, lqq) = self.bmt(ic, nx);
                let ck = ahb(hb);
                match lqq {
                    0 => { 
                        block.fj(RvInst::Gf { ck, cp: ck, gf: 1 });
                        block.fj(RvInst::Ed { cp: ck, et: Reg::Bt });
                    }
                    1 => { 
                        block.fj(RvInst::Gf { ck, cp: ck, gf: -1 });
                        block.fj(RvInst::Ed { cp: ck, et: Reg::Bt });
                    }
                    2 => { 
                        
                        block.fj(RvInst::Gf { ck: Reg::Ds, cp: Reg::Ds, gf: -8 });
                        let hsu = self.sm + self.l as u64;
                        block.fj(RvInst::Hu { ck: Reg::Bg, gf: hsu as i64 });
                        block.fj(RvInst::Mi { et: Reg::Bg, cp: Reg::Ds, l: 0 });
                        block.fj(RvInst::Xi { ck: Reg::Oq, cp: ck, l: 0 });
                        block.bil.push(0); 
                        return true;
                    }
                    4 => { 
                        block.fj(RvInst::Xi { ck: Reg::Bt, cp: ck, l: 0 });
                        return true;
                    }
                    6 => { 
                        block.fj(RvInst::Gf { ck: Reg::Ds, cp: Reg::Ds, gf: -8 });
                        block.fj(RvInst::Mi { et: ck, cp: Reg::Ds, l: 0 });
                    }
                    _ => {
                        self.cm.ddf += 1;
                        block.fj(RvInst::Fq);
                    }
                }
                false
            }

            
            0x70..=0x7F => {
                let adj = self.cmd() as i64;
                let cd = self.sm as i64 + self.l as i64 + adj;
                let mo = qal(opcode & 0x0F);
                block.fj(RvInst::Aad { mo, l: cd });
                let kve = self.sm + self.l as u64;
                block.bil.push(cd as u64);
                block.bil.push(kve);
                true
            }

            
            0xEB => {
                let adj = self.cmd() as i64;
                let cd = self.sm as i64 + self.l as i64 + adj;
                block.fj(RvInst::Xh { ck: Reg::Bt, l: cd });
                block.bil.push(cd as u64);
                true
            }

            
            0xE9 => {
                let adj = self.amq() as i64;
                let cd = self.sm as i64 + self.l as i64 + adj;
                block.fj(RvInst::Xh { ck: Reg::Bt, l: cd });
                block.bil.push(cd as u64);
                true
            }

            
            0xE8 => {
                let adj = self.amq() as i64;
                let cd = self.sm as i64 + self.l as i64 + adj;
                
                block.fj(RvInst::Gf { ck: Reg::Ds, cp: Reg::Ds, gf: -8 });
                let dbg = self.sm + self.l as u64;
                block.fj(RvInst::Hu { ck: Reg::Bg, gf: dbg as i64 });
                block.fj(RvInst::Mi { et: Reg::Bg, cp: Reg::Ds, l: 0 });
                block.fj(RvInst::En { l: cd });
                block.bil.push(cd as u64);
                block.bil.push(dbg);
                true
            }

            
            0xC3 => {
                
                block.fj(RvInst::Pt { ck: Reg::Oq, cp: Reg::Ds, l: 0 });
                block.fj(RvInst::Gf { ck: Reg::Ds, cp: Reg::Ds, gf: 8 });
                block.fj(RvInst::Ama);
                true
            }

            
            0x0F => {
                if self.l < self.aj.len() {
                    let naw = self.aj[self.l];
                    self.l += 1;

                    match naw {
                        0x05 => {
                            
                            
                            
                            block.fj(RvInst::Od {
                                arch: SourceArch::BT_,
                                ag: dik,
                                text: String::from("syscall"),
                            });
                            
                            block.fj(RvInst::Gl { ck: Reg::Vd, acl: Reg::Je });
                            
                            
                            block.fj(RvInst::Gl { ck: Reg::Bg, acl: Reg::Vc });  
                            block.fj(RvInst::Gl { ck: Reg::Je, acl: Reg::Bg });  
                            
                            block.fj(RvInst::Gl { ck: Reg::Zo, acl: Reg::Vb });
                            
                            
                            block.fj(RvInst::Gl { ck: Reg::Va, acl: Reg::Afz });
                            
                            block.fj(RvInst::Gl { ck: Reg::Vb, acl: Reg::Afx });
                            
                            block.fj(RvInst::Gl { ck: Reg::Vc, acl: Reg::Afy });
                            block.fj(RvInst::Wk);
                            
                            false
                        }

                        
                        0x80..=0x8F => {
                            let adj = self.amq() as i64;
                            let cd = self.sm as i64 + self.l as i64 + adj;
                            let mo = qal(naw & 0x0F);
                            block.fj(RvInst::Aad { mo, l: cd });
                            let kve = self.sm + self.l as u64;
                            block.bil.push(cd as u64);
                            block.bil.push(kve);
                            true
                        }

                        
                        0xB6 => {
                            let (hb, reg) = self.bmt(ic, nx);
                            let ck = ahb(reg);
                            let acl = ahb(hb);
                            block.fj(RvInst::Ou { ck, cp: acl, gf: 0xFF });
                            false
                        }
                        0xB7 => {
                            let (hb, reg) = self.bmt(ic, nx);
                            let ck = ahb(reg);
                            let acl = ahb(hb);
                            block.fj(RvInst::Ou { ck, cp: acl, gf: 0xFFFF });
                            false
                        }

                        
                        0xAF => {
                            let (hb, reg) = self.bmt(ic, nx);
                            let ck = ahb(reg);
                            let acl = ahb(hb);
                            block.fj(RvInst::Mul { ck, cp: ck, et: acl });
                            false
                        }

                        _ => {
                            self.cm.ddf += 1;
                            block.fj(RvInst::Fq);
                            false
                        }
                    }
                } else {
                    true
                }
            }

            
            0xCD => {
                let tvl = self.ady();
                if tvl == 0x80 {
                    
                    block.fj(RvInst::Od {
                        arch: SourceArch::BT_,
                        ag: dik,
                        text: String::from("int 0x80 (legacy syscall)"),
                    });
                    block.fj(RvInst::Gl { ck: Reg::Vd, acl: Reg::Je }); 
                    block.fj(RvInst::Gl { ck: Reg::Je, acl: Reg::Va }); 
                    
                    
                    block.fj(RvInst::Wk);
                }
                false
            }

            
            _ => {
                self.cm.ddf += 1;
                block.fj(RvInst::Od {
                    arch: SourceArch::BT_,
                    ag: dik,
                    text: format!("unsupported opcode: 0x{:02X}", opcode),
                });
                block.fj(RvInst::Fq);
                false
            }
        }
    }

    

    fn bmt(&mut self, ic: bool, nx: bool) -> (u8, u8) {
        if self.l >= self.aj.len() {
            return (0, 0);
        }
        let ms = self.aj[self.l];
        self.l += 1;

        let omm = (ms >> 6) & 3;
        let mut reg = (ms >> 3) & 7;
        let mut hb = ms & 7;

        if nx { reg += 8; }
        if ic { hb += 8; }

        
        if omm != 3 && hb == 4 {
            if self.l < self.aj.len() {
                self.l += 1; 
            }
        }

        
        match omm {
            0 => {
                if hb == 5 {
                    self.l += 4; 
                }
            }
            1 => { self.l += 1; } 
            2 => { self.l += 4; } 
            _ => {} 
        }

        (hb, reg)
    }

    fn ady(&mut self) -> u8 {
        if self.l >= self.aj.len() { return 0; }
        let p = self.aj[self.l];
        self.l += 1;
        p
    }

    fn cmd(&mut self) -> i8 {
        self.ady() as i8
    }

    fn amq(&mut self) -> i32 {
        if self.l + 4 > self.aj.len() { return 0; }
        let p = i32::dj([
            self.aj[self.l],
            self.aj[self.l + 1],
            self.aj[self.l + 2],
            self.aj[self.l + 3],
        ]);
        self.l += 4;
        p
    }

    fn jll(&mut self) -> i64 {
        if self.l + 8 > self.aj.len() { return 0; }
        let p = i64::dj([
            self.aj[self.l], self.aj[self.l + 1],
            self.aj[self.l + 2], self.aj[self.l + 3],
            self.aj[self.l + 4], self.aj[self.l + 5],
            self.aj[self.l + 6], self.aj[self.l + 7],
        ]);
        self.l += 8;
        p
    }
}


fn qal(nn: u8) -> FlagCond {
    match nn {
        0x0 => FlagCond::Awn,    
        0x1 => FlagCond::Awc,  
        0x2 => FlagCond::Auz,    
        0x3 => FlagCond::Atb,    
        0x4 => FlagCond::Eq,     
        0x5 => FlagCond::Adl,     
        0x6 => FlagCond::Te,     
        0x7 => FlagCond::Jn,     
        0x8 => FlagCond::Neg,    
        0x9 => FlagCond::Pos,    
        0xC => FlagCond::Lt,     
        0xD => FlagCond::Wr,     
        0xE => FlagCond::Te,     
        0xF => FlagCond::Jn,     
        _   => FlagCond::Eq,     
    }
}
