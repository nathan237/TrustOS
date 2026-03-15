






















use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use super::ir::*;


fn ccy(ijz: u8) -> Reg {
    match ijz {
        0  => Reg::Je, 
        1  => Reg::Zo, 
        2  => Reg::Afw, 
        3  => Reg::Va, 
        4  => Reg::Vb, 
        5  => Reg::Vc, 
        6  => Reg::Bav, 
        7  => Reg::Vd, 
        8  => Reg::Aol, 
        9  => Reg::Bg,  
        10 => Reg::Mp,  
        11 => Reg::Aom,  
        12 => Reg::Bay, 
        13 => Reg::Baz, 
        14 => Reg::Bba, 
        15 => Reg::Bbd,  
        16 => Reg::Mp,  
        17 => Reg::Aom,  
        18 => Reg::Bbb,  
        19 => Reg::Afx, 
        20 => Reg::Afy, 
        21 => Reg::Afz, 
        22 => Reg::Aog, 
        23 => Reg::Aoh, 
        24 => Reg::Aoi, 
        25 => Reg::Aoj, 
        26 => Reg::Aok, 
        27 => Reg::Baw, 
        28 => Reg::Bax, 
        29 => Reg::Aon,  
        30 => Reg::Oq,  
        31 => Reg::Bt,  
        _  => Reg::Bt,
    }
}


fn kba(ijz: u8, tyy: bool) -> Reg {
    if ijz == 31 {
        if tyy { Reg::Ds } else { Reg::Bt }
    } else {
        ccy(ijz)
    }
}


pub struct ArmDecoder {
    
    aj: Vec<u8>,
    
    sm: u64,
    
    l: usize,
    
    pub cm: TranslationStats,
}

impl ArmDecoder {
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
        let mut block = TranslatedBlock::new(cbz, SourceArch::Fg);

        let eff = 256;
        let mut az = 0;

        while self.l + 4 <= self.aj.len() && az < eff {
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
                    if ice + 4 <= self.aj.len() && !bxs.contains(&fvt) {
                        fyw.push(ice);
                    }
                }
            }

            xk.push(block);
        }

        xk
    }

    
    fn hjd(&mut self) -> u32 {
        if self.l + 4 > self.aj.len() {
            return 0;
        }
        let fi = u32::dj([
            self.aj[self.l],
            self.aj[self.l + 1],
            self.aj[self.l + 2],
            self.aj[self.l + 3],
        ]);
        self.l += 4;
        fi
    }

    
    
    fn hfp(&mut self, block: &mut TranslatedBlock) -> bool {
        let dik = self.sm + self.l as u64;
        let fi = self.hjd();

        if fi == 0 {
            block.fj(RvInst::Fq);
            return true;
        }

        
        let uym = (fi >> 25) & 0xF;

        match uym {
            
            0b1000 | 0b1001 => self.ruj(fi, dik, block),

            
            0b1010 | 0b1011 => return self.ruh(fi, dik, block),

            
            0b0100 | 0b0110 | 0b1100 | 0b1110 => self.ruo(fi, dik, block),

            
            0b0101 | 0b1101 => self.ruk(fi, dik, block),

            _ => {
                self.cm.ddf += 1;
                block.fj(RvInst::Od {
                    arch: SourceArch::Fg,
                    ag: dik,
                    text: format!("unsupported: 0x{:08X}", fi),
                });
                block.fj(RvInst::Fq);
            }
        }

        false
    }

    
    fn ruj(&mut self, fi: u32, ag: u64, block: &mut TranslatedBlock) {
        let fps = (fi >> 23) & 0x7;
        let eim = (fi >> 31) & 1; 
        let ck = (fi & 0x1F) as u8;
        let dve = ((fi >> 5) & 0x1F) as u8;

        match fps {
            
            0b010 | 0b011 => {
                let acn = ((fi >> 22) & 1) as u8;
                let izn = ((fi >> 10) & 0xFFF) as i64;
                let gf = if acn == 1 { izn << 12 } else { izn };
                let jbv = (fi >> 30) & 1 == 1;
                let gsh = (fi >> 29) & 1 == 1;

                let aen = kba(ck, !gsh);
                let we = kba(dve, true);

                if jbv {
                    block.fj(RvInst::Gf { ck: aen, cp: we, gf: -gf });
                } else {
                    block.fj(RvInst::Gf { ck: aen, cp: we, gf });
                }

                if gsh {
                    block.fj(RvInst::Ed { cp: aen, et: Reg::Bt });
                }
            }

            
            0b100 | 0b101 => {
                let avz = ((fi >> 21) & 0x3) as u8;
                let buy = ((fi >> 5) & 0xFFFF) as i64;
                let uyt = (fi >> 29) & 0x3;
                let aen = ccy(ck);

                let dvy = buy << (avz * 16);

                match uyt {
                    0b00 => { 
                        block.fj(RvInst::Hu { ck: aen, gf: !dvy });
                    }
                    0b10 => { 
                        block.fj(RvInst::Hu { ck: aen, gf: dvy });
                    }
                    0b11 => { 
                        let hs = !(0xFFFF_i64 << (avz * 16));
                        block.fj(RvInst::Hu { ck: Reg::Bg, gf: hs });
                        block.fj(RvInst::Ex { ck: aen, cp: aen, et: Reg::Bg });
                        block.fj(RvInst::Hu { ck: Reg::Bg, gf: dvy });
                        block.fj(RvInst::Fx { ck: aen, cp: aen, et: Reg::Bg });
                    }
                    _ => {
                        block.fj(RvInst::Fq);
                    }
                }
            }

            
            0b110 => {
                
                let aen = ccy(ck);
                let we = ccy(dve);
                let hqh = (fi >> 29) & 0x3;
                
                let izo = ruf(fi, eim == 1);

                match hqh {
                    0b00 => block.fj(RvInst::Ou { ck: aen, cp: we, gf: izo }),
                    0b01 => block.fj(RvInst::Akw { ck: aen, cp: we, gf: izo }),
                    0b10 => block.fj(RvInst::Aoq { ck: aen, cp: we, gf: izo }),
                    0b11 => { 
                        block.fj(RvInst::Ou { ck: aen, cp: we, gf: izo });
                        block.fj(RvInst::Ed { cp: aen, et: Reg::Bt });
                    }
                    _ => {}
                }
            }

            
            0b000 | 0b001 => {
                let aen = ccy(ck);
                let tsd = ((fi >> 5) & 0x7FFFF) as i64;
                let tse = ((fi >> 29) & 0x3) as i64;
                let twq = (fi >> 31) & 1 == 1;
                let mut gf = (tsd << 2) | tse;
                
                if gf & (1 << 20) != 0 { gf |= !0x1FFFFF; }
                if twq { gf <<= 12; }
                let cd = ag as i64 + gf;
                block.fj(RvInst::Hu { ck: aen, gf: cd });
            }

            _ => {
                self.cm.ddf += 1;
                block.fj(RvInst::Fq);
            }
        }
    }

    
    fn ruh(&mut self, fi: u32, ag: u64, block: &mut TranslatedBlock) -> bool {
        let uyn = (fi >> 29) & 0x7;

        match uyn {
            
            0b000 | 0b100 => {
                let mut ldm = (fi & 0x03FF_FFFF) as i64;
                if ldm & (1 << 25) != 0 { ldm |= !0x03FF_FFFF; }
                let cd = ag as i64 + (ldm << 2);
                let ofq = (fi >> 31) & 1 == 1;

                if ofq {
                    
                    block.fj(RvInst::Xh { ck: Reg::Oq, l: cd });
                } else {
                    
                    block.fj(RvInst::Xh { ck: Reg::Bt, l: cd });
                }

                block.bil.push(cd as u64);
                if ofq {
                    block.bil.push(ag + 4);
                }
                true
            }

            
            0b010 => {
                let mut flf = ((fi >> 5) & 0x7FFFF) as i64;
                if flf & (1 << 18) != 0 { flf |= !0x7FFFF; }
                let cd = ag as i64 + (flf << 2);
                let mo = (fi & 0xF) as u8;

                let suk = qkn(mo);
                block.fj(RvInst::Aad { mo: suk, l: cd });
                block.bil.push(cd as u64);
                block.bil.push(ag + 4);
                true
            }

            
            0b001 | 0b101 => {
                let dbl = (fi & 0x1F) as u8;
                let mut flf = ((fi >> 5) & 0x7FFFF) as i64;
                if flf & (1 << 18) != 0 { flf |= !0x7FFFF; }
                let cd = ag as i64 + (flf << 2);
                let twv = (fi >> 24) & 1 == 1;
                let bck = ccy(dbl);

                if twv {
                    block.fj(RvInst::Ags { cp: bck, et: Reg::Bt, l: cd });
                } else {
                    block.fj(RvInst::Agp { cp: bck, et: Reg::Bt, l: cd });
                }

                block.bil.push(cd as u64);
                block.bil.push(ag + 4);
                true
            }

            
            0b110 => {
                let fpq = (fi >> 21) & 0x7;
                if fpq == 0 {
                    
                    let fps = (fi >> 21) & 0x7;
                    let glq = fi & 0x3;
                    if glq == 1 {
                        
                        block.fj(RvInst::Od {
                            arch: SourceArch::Fg,
                            ag,
                            text: String::from("SVC #0 (syscall)"),
                        });
                        
                        
                        
                        block.fj(RvInst::Gl { ck: Reg::Vd, acl: Reg::Aol });
                        
                        block.fj(RvInst::Wk);
                        
                        return false;
                    } else if glq == 0 && fps == 0 {
                        
                        block.fj(RvInst::Wk);
                        return false;
                    }
                }

                
                if (fi & 0xFFFFFC1F) == 0xD65F0000 {
                    block.fj(RvInst::Ama);
                    return true;
                }

                
                if (fi & 0xFFFFFC00) == 0xD61F0000 {
                    let dve = ((fi >> 5) & 0x1F) as u8;
                    let we = ccy(dve);
                    block.fj(RvInst::Xi { ck: Reg::Bt, cp: we, l: 0 });
                    return true;
                }

                
                if (fi & 0xFFFFFC00) == 0xD63F0000 {
                    let dve = ((fi >> 5) & 0x1F) as u8;
                    let we = ccy(dve);
                    block.fj(RvInst::Xi { ck: Reg::Oq, cp: we, l: 0 });
                    block.bil.push(ag + 4);
                    return true;
                }

                
                if fi == 0xD503201F { 
                    block.fj(RvInst::Fq);
                    return false;
                }
                if fi == 0xD503207F { 
                    block.fj(RvInst::Fq);
                    return false;
                }

                self.cm.ddf += 1;
                block.fj(RvInst::Fq);
                false
            }

            
            0b011 | 0b111 => {
                let dbl = (fi & 0x1F) as u8;
                let ga = ((fi >> 19) & 0x1F) as u8 | (((fi >> 31) & 1) as u8) << 5;
                let mut ldl = ((fi >> 5) & 0x3FFF) as i64;
                if ldl & (1 << 13) != 0 { ldl |= !0x3FFF; }
                let cd = ag as i64 + (ldl << 2);
                let tzd = (fi >> 24) & 1 == 1;

                let bck = ccy(dbl);
                
                block.fj(RvInst::Hu { ck: Reg::Bg, gf: 1 << ga });
                block.fj(RvInst::Ex { ck: Reg::Bg, cp: bck, et: Reg::Bg });

                if tzd {
                    block.fj(RvInst::Ags { cp: Reg::Bg, et: Reg::Bt, l: cd });
                } else {
                    block.fj(RvInst::Agp { cp: Reg::Bg, et: Reg::Bt, l: cd });
                }

                block.bil.push(cd as u64);
                block.bil.push(ag + 4);
                true
            }

            _ => {
                self.cm.ddf += 1;
                block.fj(RvInst::Fq);
                false
            }
        }
    }

    
    fn ruo(&mut self, fi: u32, ag: u64, block: &mut TranslatedBlock) {
        let aw = (fi >> 30) & 0x3;
        let fps = (fi >> 22) & 0x3;
        let dve = ((fi >> 5) & 0x1F) as u8;
        let dbl = (fi & 0x1F) as u8;

        let bck = ccy(dbl);
        let we = kba(dve, true);

        
        if (fi & 0x3B000000) == 0x39000000 {
            let izn = ((fi >> 10) & 0xFFF) as i64;
            let bv = aw as i64;
            let l = izn << bv;
            let gkf = (fps & 1) == 1;

            if gkf {
                match aw {
                    0 => block.fj(RvInst::Ajr { ck: bck, cp: we, l }),
                    1 => block.fj(RvInst::Ajs { ck: bck, cp: we, l }),
                    2 => block.fj(RvInst::Aka { ck: bck, cp: we, l }),
                    3 => block.fj(RvInst::Pt { ck: bck, cp: we, l }),
                    _ => {}
                }
            } else {
                match aw {
                    0 => block.fj(RvInst::Amf { et: bck, cp: we, l }),
                    1 => block.fj(RvInst::Amo { et: bck, cp: we, l }),
                    2 => block.fj(RvInst::Ang { et: bck, cp: we, l }),
                    3 => block.fj(RvInst::Mi { et: bck, cp: we, l }),
                    _ => {}
                }
            }
            return;
        }

        
        if (fi & 0x3B200C00) == 0x38000000 || (fi & 0x3B200C00) == 0x38000400 {
            let mut cyu = ((fi >> 12) & 0x1FF) as i64;
            if cyu & (1 << 8) != 0 { cyu |= !0x1FF; }
            let dsh = (fi >> 11) & 1 == 1;
            let gkf = (fps & 1) == 1;

            if dsh {
                
                block.fj(RvInst::Gf { ck: we, cp: we, gf: cyu });
            }

            if gkf {
                match aw {
                    0 => block.fj(RvInst::Ajr { ck: bck, cp: we, l: if dsh { 0 } else { cyu } }),
                    1 => block.fj(RvInst::Ajs { ck: bck, cp: we, l: if dsh { 0 } else { cyu } }),
                    2 => block.fj(RvInst::Aka { ck: bck, cp: we, l: if dsh { 0 } else { cyu } }),
                    3 => block.fj(RvInst::Pt { ck: bck, cp: we, l: if dsh { 0 } else { cyu } }),
                    _ => {}
                }
            } else {
                match aw {
                    0 => block.fj(RvInst::Amf { et: bck, cp: we, l: if dsh { 0 } else { cyu } }),
                    1 => block.fj(RvInst::Amo { et: bck, cp: we, l: if dsh { 0 } else { cyu } }),
                    2 => block.fj(RvInst::Ang { et: bck, cp: we, l: if dsh { 0 } else { cyu } }),
                    3 => block.fj(RvInst::Mi { et: bck, cp: we, l: if dsh { 0 } else { cyu } }),
                    _ => {}
                }
            }

            if !dsh {
                
                block.fj(RvInst::Gf { ck: we, cp: we, gf: cyu });
            }
            return;
        }

        
        if (fi & 0x3E000000) == 0x28000000 || (fi & 0x3E000000) == 0x2C000000 {
            let wba = ((fi >> 10) & 0x1F) as u8;
            let mut ldn = ((fi >> 15) & 0x7F) as i64;
            if ldn & (1 << 6) != 0 { ldn |= !0x7F; }
            let bv = if (fi >> 31) & 1 == 1 { 3 } else { 2 };
            let l = ldn << bv;
            let gkf = (fi >> 22) & 1 == 1;

            let pet = ccy(wba);

            if gkf {
                block.fj(RvInst::Pt { ck: bck, cp: we, l });
                block.fj(RvInst::Pt { ck: pet, cp: we, l: l + 8 });
            } else {
                block.fj(RvInst::Mi { et: bck, cp: we, l });
                block.fj(RvInst::Mi { et: pet, cp: we, l: l + 8 });
            }
            return;
        }

        
        self.cm.ddf += 1;
        block.fj(RvInst::Od {
            arch: SourceArch::Fg,
            ag,
            text: format!("unsupported ldst: 0x{:08X}", fi),
        });
        block.fj(RvInst::Fq);
    }

    
    fn ruk(&mut self, fi: u32, ag: u64, block: &mut TranslatedBlock) {
        let ck = (fi & 0x1F) as u8;
        let dve = ((fi >> 5) & 0x1F) as u8;
        let hb = ((fi >> 16) & 0x1F) as u8;
        let fps = (fi >> 29) & 0x7;

        let aen = ccy(ck);
        let we = ccy(dve);
        let aww = ccy(hb);

        
        if (fi & 0x1F000000) == 0x0A000000 {
            let hqh = (fi >> 29) & 0x3;
            let bo = (fi >> 21) & 1;
            let gsh = hqh == 0b11;

            match (hqh, bo) {
                (0b00, 0) => block.fj(RvInst::Ex { ck: aen, cp: we, et: aww }),
                (0b01, 0) => block.fj(RvInst::Fx { ck: aen, cp: we, et: aww }),
                (0b10, 0) => block.fj(RvInst::Aga { ck: aen, cp: we, et: aww }),
                (0b11, 0) => { 
                    block.fj(RvInst::Ex { ck: aen, cp: we, et: aww });
                }
                
                (_, 1) => {
                    block.fj(RvInst::Aoq { ck: Reg::Bg, cp: aww, gf: -1 });
                    match hqh {
                        0b00 => block.fj(RvInst::Ex { ck: aen, cp: we, et: Reg::Bg }),
                        0b01 => block.fj(RvInst::Fx { ck: aen, cp: we, et: Reg::Bg }),
                        0b10 => block.fj(RvInst::Aga { ck: aen, cp: we, et: Reg::Bg }),
                        0b11 => block.fj(RvInst::Ex { ck: aen, cp: we, et: Reg::Bg }),
                        _ => {}
                    }
                }
                _ => { block.fj(RvInst::Fq); }
            }

            if gsh {
                block.fj(RvInst::Ed { cp: aen, et: Reg::Bt });
            }
            return;
        }

        
        if (fi & 0x1F000000) == 0x0B000000 {
            let jbv = (fi >> 30) & 1 == 1;
            let gsh = (fi >> 29) & 1 == 1;

            
            let wmw = ((fi >> 22) & 0x3) as u8;
            let jpw = ((fi >> 10) & 0x3F) as u8;

            if jpw > 0 {
                match wmw {
                    0 => block.fj(RvInst::Ayv { ck: Reg::Bg, cp: aww, bcp: jpw }),
                    1 => block.fj(RvInst::Aze { ck: Reg::Bg, cp: aww, bcp: jpw }),
                    2 => block.fj(RvInst::Azd { ck: Reg::Bg, cp: aww, bcp: jpw }),
                    _ => block.fj(RvInst::Gl { ck: Reg::Bg, acl: aww }),
                }
                if jbv {
                    block.fj(RvInst::Sub { ck: aen, cp: we, et: Reg::Bg });
                } else {
                    block.fj(RvInst::Add { ck: aen, cp: we, et: Reg::Bg });
                }
            } else {
                if jbv {
                    block.fj(RvInst::Sub { ck: aen, cp: we, et: aww });
                } else {
                    block.fj(RvInst::Add { ck: aen, cp: we, et: aww });
                }
            }

            if gsh {
                block.fj(RvInst::Ed { cp: aen, et: Reg::Bt });
            }
            return;
        }

        
        if (fi & 0x7FE00000) == 0x1B000000 {
            let hwl = ((fi >> 10) & 0x1F) as u8;
            let tyg = (fi >> 15) & 1 == 1;

            if hwl == 31 {
                
                block.fj(RvInst::Mul { ck: aen, cp: we, et: aww });
            } else {
                let pes = ccy(hwl);
                block.fj(RvInst::Mul { ck: Reg::Bg, cp: we, et: aww });
                if tyg {
                    block.fj(RvInst::Sub { ck: aen, cp: pes, et: Reg::Bg });
                } else {
                    block.fj(RvInst::Add { ck: aen, cp: pes, et: Reg::Bg });
                }
            }
            return;
        }

        
        if (fi & 0x7FE0FC00) == 0x1AC00800 {
            let tzg = (fi >> 10) & 1 == 0;
            if tzg {
                block.fj(RvInst::Arb { ck: aen, cp: we, et: aww });
            } else {
                block.fj(RvInst::Div { ck: aen, cp: we, et: aww });
            }
            return;
        }

        
        if (fi & 0x7FE0F000) == 0x1AC02000 {
            let wmv = ((fi >> 10) & 0x3) as u8;
            match wmv {
                0 => block.fj(RvInst::Amt { ck: aen, cp: we, et: aww }),
                1 => block.fj(RvInst::Amx { ck: aen, cp: we, et: aww }),
                2 => block.fj(RvInst::Azc { ck: aen, cp: we, et: aww }),
                
                3 => {
                    block.fj(RvInst::Amx { ck: Reg::Bg, cp: we, et: aww });
                    block.fj(RvInst::Hu { ck: Reg::Mp, gf: 64 });
                    block.fj(RvInst::Sub { ck: Reg::Mp, cp: Reg::Mp, et: aww });
                    block.fj(RvInst::Amt { ck: Reg::Mp, cp: we, et: Reg::Mp });
                    block.fj(RvInst::Fx { ck: aen, cp: Reg::Bg, et: Reg::Mp });
                }
                _ => {}
            }
            return;
        }

        
        self.cm.ddf += 1;
        block.fj(RvInst::Od {
            arch: SourceArch::Fg,
            ag,
            text: format!("unsupported dp_reg: 0x{:08X}", fi),
        });
        block.fj(RvInst::Fq);
    }
}


fn qkn(nn: u8) -> FlagCond {
    match nn {
        0x0 => FlagCond::Eq,     
        0x1 => FlagCond::Adl,     
        0x2 => FlagCond::Atb,    
        0x3 => FlagCond::Auz,    
        0x4 => FlagCond::Neg,    
        0x5 => FlagCond::Pos,    
        0x6 => FlagCond::Awn,    
        0x7 => FlagCond::Awc,  
        0x8 => FlagCond::Jn,     
        0x9 => FlagCond::Te,     
        0xA => FlagCond::Wr,     
        0xB => FlagCond::Lt,     
        0xC => FlagCond::Jn,     
        0xD => FlagCond::Te,     
        0xE => FlagCond::Eq,     
        _   => FlagCond::Eq,
    }
}


fn ruf(fi: u32, yad: bool) -> i64 {
    let bo = (fi >> 22) & 1;
    let tsf = ((fi >> 16) & 0x3F) as u32;
    let odl = ((fi >> 10) & 0x3F) as u32;

    
    let len = if bo == 1 { 6 } else {
        
        let jhi = !odl & 0x3F;
        if jhi & 0x20 != 0 { 5 }
        else if jhi & 0x10 != 0 { 4 }
        else if jhi & 0x08 != 0 { 3 }
        else if jhi & 0x04 != 0 { 2 }
        else { 1 }
    };

    let aw = 1u64 << len;
    let hs = aw - 1;
    let e = (odl & hs as u32) as u64;
    let m = (tsf & hs as u32) as u64;

    let mut ihe: u64 = (1u64 << (e + 1)) - 1;
    
    if m > 0 {
        ihe = (ihe >> m) | (ihe << (aw - m));
        ihe &= (1u64 << aw) - 1;
    }

    
    let mut result = ihe;
    let mut kmo = aw;
    while kmo < 64 {
        result |= result << kmo;
        kmo *= 2;
    }

    result as i64
}
