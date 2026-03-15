

#![allow(bgr)]

pub const CE_: u8 = 0x80;
pub const DR_: u8 = 0x40;
pub const HY_: u8 = 0x20;
pub const AX_: u8 = 0x10;

pub trait Kn {
    fn read(&mut self, ag: u16) -> u8;
    fn write(&mut self, ag: u16, ap: u8);
}

pub struct Cpu {
    pub q: u8, pub bb: u8,
    pub o: u8, pub r: u8,
    pub bc: u8, pub aa: u8,
    pub i: u8, pub dm: u8,
    pub sp: u16,
    pub fz: u16,
    pub dih: bool,
    pub izm: bool,
    pub dhv: bool,
    pub yl: u64,
}

impl Cpu {
    pub fn new() -> Self {
        
        Self {
            q: 0x01, bb: 0xB0,
            o: 0x00, r: 0x13,
            bc: 0x00, aa: 0xD8,
            i: 0x01, dm: 0x4D,
            sp: 0xFFFE,
            fz: 0x0100,
            dih: true,
            izm: false,
            dhv: false,
            yl: 0,
        }
    }

    
    pub fn qfv(&self) -> u16 { (self.q as u16) << 8 | self.bb as u16 }
    pub fn atw(&self) -> u16 { (self.o as u16) << 8 | self.r as u16 }
    pub fn gef(&self) -> u16 { (self.bc as u16) << 8 | self.aa as u16 }
    pub fn abe(&self) -> u16 { (self.i as u16) << 8 | self.dm as u16 }
    fn wie(&mut self, p: u16) { self.q = (p >> 8) as u8; self.bb = (p & 0xF0) as u8; }
    fn joq(&mut self, p: u16) { self.o = (p >> 8) as u8; self.r = (p & 0xFF) as u8; }
    fn jot(&mut self, p: u16) { self.bc = (p >> 8) as u8; self.aa = (p & 0xFF) as u8; }
    fn dvp(&mut self, p: u16) { self.i = (p >> 8) as u8; self.dm = (p & 0xFF) as u8; }

    
    fn aca(&self) -> bool { self.bb & CE_ != 0 }
    fn gns(&self) -> bool { self.bb & DR_ != 0 }
    fn obu(&self) -> bool { self.bb & HY_ != 0 }
    fn vq(&self) -> bool { self.bb & AX_ != 0 }
    fn bff(&mut self, av: bool, bo: bool, i: bool, r: bool) {
        self.bb = (if av { CE_ } else { 0 })
               | (if bo { DR_ } else { 0 })
               | (if i { HY_ } else { 0 })
               | (if r { AX_ } else { 0 });
    }

    
    fn auc(&mut self, aq: &mut impl Kn) -> u8 {
        let p = aq.read(self.fz); self.fz = self.fz.cn(1); p
    }
    fn bug(&mut self, aq: &mut impl Kn) -> u16 {
        let hh = aq.read(self.fz) as u16;
        let gd = aq.read(self.fz.cn(1)) as u16;
        self.fz = self.fz.cn(2);
        hh | (gd << 8)
    }

    
    fn bcf(&mut self, aq: &mut impl Kn, ap: u16) {
        self.sp = self.sp.nj(1); aq.write(self.sp, (ap >> 8) as u8);
        self.sp = self.sp.nj(1); aq.write(self.sp, ap as u8);
    }
    fn dup(&mut self, aq: &mut impl Kn) -> u16 {
        let hh = aq.read(self.sp) as u16; self.sp = self.sp.cn(1);
        let gd = aq.read(self.sp) as u16; self.sp = self.sp.cn(1);
        hh | (gd << 8)
    }

    
    fn bzu(&self, m: u8, aq: &mut impl Kn) -> u8 {
        match m & 7 {
            0 => self.o, 1 => self.r, 2 => self.bc, 3 => self.aa,
            4 => self.i, 5 => self.dm, 6 => aq.read(self.abe()), 7 => self.q,
            _ => 0,
        }
    }
    fn bxa(&mut self, m: u8, p: u8, aq: &mut impl Kn) {
        match m & 7 {
            0 => self.o = p, 1 => self.r = p, 2 => self.bc = p, 3 => self.aa = p,
            4 => self.i = p, 5 => self.dm = p, 6 => aq.write(self.abe(), p), 7 => self.q = p,
            _ => {}
        }
    }

    
    fn mvi(&mut self, p: u8) {
        let q = self.q; let m = q.cn(p);
        self.bff(m == 0, false, (q & 0xF) + (p & 0xF) > 0xF, (q as u16 + p as u16) > 0xFF);
        self.q = m;
    }
    fn mvh(&mut self, p: u8) {
        let q = self.q; let r = if self.vq() { 1u8 } else { 0 };
        let m = q.cn(p).cn(r);
        self.bff(m == 0, false, (q & 0xF) + (p & 0xF) + r > 0xF, q as u16 + p as u16 + r as u16 > 0xFF);
        self.q = m;
    }
    fn mvn(&mut self, p: u8) {
        let q = self.q; let m = q.nj(p);
        self.bff(m == 0, true, (q & 0xF) < (p & 0xF), q < p);
        self.q = m;
    }
    fn mvm(&mut self, p: u8) {
        let q = self.q; let r = if self.vq() { 1u8 } else { 0 };
        let m = q.nj(p).nj(r);
        self.bff(m == 0, true, (q & 0xF) < (p & 0xF).cn(r), (q as u16) < p as u16 + r as u16);
        self.q = m;
    }
    fn mvj(&mut self, p: u8) { self.q &= p; self.bff(self.q == 0, false, true, false); }
    fn mvo(&mut self, p: u8) { self.q ^= p; self.bff(self.q == 0, false, false, false); }
    fn mvl(&mut self, p: u8)  { self.q |= p; self.bff(self.q == 0, false, false, false); }
    fn mvk(&mut self, p: u8) {
        let q = self.q; let m = q.nj(p);
        self.bff(m == 0, true, (q & 0xF) < (p & 0xF), q < p);
    }

    
    fn qws(&mut self, p: u8) -> u8 { let r = p >> 7; let m = (p << 1) | r; self.bff(m == 0, false, false, r != 0); m }
    fn qwu(&mut self, p: u8) -> u8 { let r = p & 1; let m = (p >> 1) | (r << 7); self.bff(m == 0, false, false, r != 0); m }
    fn qwr(&mut self, p: u8) -> u8 { let dtu = if self.vq() { 1u8 } else { 0 }; let r = p >> 7; let m = (p << 1) | dtu; self.bff(m == 0, false, false, r != 0); m }
    fn qwt(&mut self, p: u8) -> u8 { let dtu = if self.vq() { 1u8 } else { 0 }; let r = p & 1; let m = (p >> 1) | (dtu << 7); self.bff(m == 0, false, false, r != 0); m }
    fn qwv(&mut self, p: u8) -> u8 { let r = p >> 7; let m = p << 1; self.bff(m == 0, false, false, r != 0); m }
    fn qww(&mut self, p: u8) -> u8 { let r = p & 1; let m = (p >> 1) | (p & 0x80); self.bff(m == 0, false, false, r != 0); m }
    fn qwy(&mut self, p: u8) -> u8 { let m = (p >> 4) | (p << 4); self.bff(m == 0, false, false, false); m }
    fn qwx(&mut self, p: u8) -> u8 { let r = p & 1; let m = p >> 1; self.bff(m == 0, false, false, r != 0); m }

    
    fn esm(&mut self, p: u8) -> u8 {
        let m = p.cn(1);
        self.bb = (if m == 0 { CE_ } else { 0 }) | (if (p & 0xF) + 1 > 0xF { HY_ } else { 0 }) | (self.bb & AX_);
        m
    }
    fn eoo(&mut self, p: u8) -> u8 {
        let m = p.nj(1);
        self.bb = (if m == 0 { CE_ } else { 0 }) | DR_ | (if p & 0xF == 0 { HY_ } else { 0 }) | (self.bb & AX_);
        m
    }
    fn iiy(&mut self, p: u16) {
        let abe = self.abe(); let m = abe.cn(p);
        self.bb = (self.bb & CE_) | (if (abe & 0xFFF) + (p & 0xFFF) > 0xFFF { HY_ } else { 0 }) | (if abe as u32 + p as u32 > 0xFFFF { AX_ } else { 0 });
        self.dvp(m);
    }
    fn rtf(&mut self) {
        let mut q = self.q as i32;
        if self.gns() {
            if self.obu() { q = (q.nj(6)) & 0xFF; }
            if self.vq() { q = q.nj(0x60); }
        } else {
            if self.obu() || (q & 0xF) > 9 { q = q.cn(6); }
            if self.vq() || q > 0x9F { q = q.cn(0x60); }
        }
        let r = self.vq() || q > 0xFF;
        self.q = q as u8;
        self.bb = (if self.q == 0 { CE_ } else { 0 }) | (self.bb & DR_) | (if r { AX_ } else { 0 });
    }

    
    
    
    pub fn gu(&mut self, aq: &mut impl Kn) -> u32 {
        
        if self.izm { self.dih = true; self.izm = false; }

        
        if self.dih || self.dhv {
            let hnr = aq.read(0xFFFF);
            let dry = aq.read(0xFF0F);
            let aln = hnr & dry & 0x1F;
            if aln != 0 {
                self.dhv = false;
                if self.dih {
                    self.dih = false;
                    for ga in 0u16..5 {
                        if aln & (1 << ga) != 0 {
                            aq.write(0xFF0F, dry & !(1 << ga as u8));
                            self.bcf(aq, self.fz);
                            self.fz = 0x0040 + ga * 8;
                            self.yl += 5;
                            return 5;
                        }
                    }
                }
            }
        }
        if self.dhv { self.yl += 1; return 1; }

        let op = self.auc(aq);
        let ef = self.bna(op, aq);
        self.yl += ef as u64;
        ef
    }

    fn bna(&mut self, op: u8, aq: &mut impl Kn) -> u32 {
        match op {
            
            0x00 => 1,
            0x01 => { let p = self.bug(aq); self.joq(p); 3 }
            0x02 => { aq.write(self.atw(), self.q); 2 }
            0x03 => { let p = self.atw().cn(1); self.joq(p); 2 }
            0x04 => { self.o = self.esm(self.o); 1 }
            0x05 => { self.o = self.eoo(self.o); 1 }
            0x06 => { self.o = self.auc(aq); 2 }
            0x07 => { let r = self.q >> 7; self.q = (self.q << 1) | r; self.bff(false, false, false, r != 0); 1 }
            0x08 => { let q = self.bug(aq); aq.write(q, self.sp as u8); aq.write(q.cn(1), (self.sp >> 8) as u8); 5 }
            0x09 => { self.iiy(self.atw()); 2 }
            0x0A => { self.q = aq.read(self.atw()); 2 }
            0x0B => { let p = self.atw().nj(1); self.joq(p); 2 }
            0x0C => { self.r = self.esm(self.r); 1 }
            0x0D => { self.r = self.eoo(self.r); 1 }
            0x0E => { self.r = self.auc(aq); 2 }
            0x0F => { let r = self.q & 1; self.q = (self.q >> 1) | (r << 7); self.bff(false, false, false, r != 0); 1 }

            
            0x10 => {
                
                let beq = aq.read(0xFF4D);
                if beq & 0x01 != 0 {
                    
                    aq.write(0xFF4D, (beq ^ 0x80) & !0x01);
                }
                self.fz = self.fz.cn(1);
                1
            }
            0x11 => { let p = self.bug(aq); self.jot(p); 3 }
            0x12 => { aq.write(self.gef(), self.q); 2 }
            0x13 => { let p = self.gef().cn(1); self.jot(p); 2 }
            0x14 => { self.bc = self.esm(self.bc); 1 }
            0x15 => { self.bc = self.eoo(self.bc); 1 }
            0x16 => { self.bc = self.auc(aq); 2 }
            0x17 => { let dtu = if self.vq() { 1u8 } else { 0 }; let r = self.q >> 7; self.q = (self.q << 1) | dtu; self.bff(false, false, false, r != 0); 1 }
            0x18 => { let aa = self.auc(aq) as i8; self.fz = self.fz.cn(aa as u16); 3 }
            0x19 => { self.iiy(self.gef()); 2 }
            0x1A => { self.q = aq.read(self.gef()); 2 }
            0x1B => { let p = self.gef().nj(1); self.jot(p); 2 }
            0x1C => { self.aa = self.esm(self.aa); 1 }
            0x1D => { self.aa = self.eoo(self.aa); 1 }
            0x1E => { self.aa = self.auc(aq); 2 }
            0x1F => { let dtu = if self.vq() { 1u8 } else { 0 }; let r = self.q & 1; self.q = (self.q >> 1) | (dtu << 7); self.bff(false, false, false, r != 0); 1 }

            
            0x20 => { let aa = self.auc(aq) as i8; if !self.aca() { self.fz = self.fz.cn(aa as u16); 3 } else { 2 } }
            0x21 => { let p = self.bug(aq); self.dvp(p); 3 }
            0x22 => { let abe = self.abe(); aq.write(abe, self.q); self.dvp(abe.cn(1)); 2 }
            0x23 => { let p = self.abe().cn(1); self.dvp(p); 2 }
            0x24 => { self.i = self.esm(self.i); 1 }
            0x25 => { self.i = self.eoo(self.i); 1 }
            0x26 => { self.i = self.auc(aq); 2 }
            0x27 => { self.rtf(); 1 }
            0x28 => { let aa = self.auc(aq) as i8; if self.aca() { self.fz = self.fz.cn(aa as u16); 3 } else { 2 } }
            0x29 => { let abe = self.abe(); self.iiy(abe); 2 }
            0x2A => { let abe = self.abe(); self.q = aq.read(abe); self.dvp(abe.cn(1)); 2 }
            0x2B => { let p = self.abe().nj(1); self.dvp(p); 2 }
            0x2C => { self.dm = self.esm(self.dm); 1 }
            0x2D => { self.dm = self.eoo(self.dm); 1 }
            0x2E => { self.dm = self.auc(aq); 2 }
            0x2F => { self.q = !self.q; self.bb = (self.bb & (CE_ | AX_)) | DR_ | HY_; 1 }

            
            0x30 => { let aa = self.auc(aq) as i8; if !self.vq() { self.fz = self.fz.cn(aa as u16); 3 } else { 2 } }
            0x31 => { self.sp = self.bug(aq); 3 }
            0x32 => { let abe = self.abe(); aq.write(abe, self.q); self.dvp(abe.nj(1)); 2 }
            0x33 => { self.sp = self.sp.cn(1); 2 }
            0x34 => { let abe = self.abe(); let p = self.esm(aq.read(abe)); aq.write(abe, p); 3 }
            0x35 => { let abe = self.abe(); let p = self.eoo(aq.read(abe)); aq.write(abe, p); 3 }
            0x36 => { let p = self.auc(aq); aq.write(self.abe(), p); 3 }
            0x37 => { self.bb = (self.bb & CE_) | AX_; 1 }
            0x38 => { let aa = self.auc(aq) as i8; if self.vq() { self.fz = self.fz.cn(aa as u16); 3 } else { 2 } }
            0x39 => { self.iiy(self.sp); 2 }
            0x3A => { let abe = self.abe(); self.q = aq.read(abe); self.dvp(abe.nj(1)); 2 }
            0x3B => { self.sp = self.sp.nj(1); 2 }
            0x3C => { self.q = self.esm(self.q); 1 }
            0x3D => { self.q = self.eoo(self.q); 1 }
            0x3E => { self.q = self.auc(aq); 2 }
            0x3F => { let r = !self.vq(); self.bb = (self.bb & CE_) | if r { AX_ } else { 0 }; 1 }

            
            0x76 => { self.dhv = true; 1 }
            0x40..=0x75 | 0x77..=0x7F => {
                let cs = (op >> 3) & 7;
                let cy = op & 7;
                let p = self.bzu(cy, aq);
                self.bxa(cs, p, aq);
                if cy == 6 || cs == 6 { 2 } else { 1 }
            }

            
            0x80..=0x87 => { let p = self.bzu(op & 7, aq); self.mvi(p); if op & 7 == 6 { 2 } else { 1 } }
            0x88..=0x8F => { let p = self.bzu(op & 7, aq); self.mvh(p); if op & 7 == 6 { 2 } else { 1 } }
            0x90..=0x97 => { let p = self.bzu(op & 7, aq); self.mvn(p); if op & 7 == 6 { 2 } else { 1 } }
            0x98..=0x9F => { let p = self.bzu(op & 7, aq); self.mvm(p); if op & 7 == 6 { 2 } else { 1 } }
            0xA0..=0xA7 => { let p = self.bzu(op & 7, aq); self.mvj(p); if op & 7 == 6 { 2 } else { 1 } }
            0xA8..=0xAF => { let p = self.bzu(op & 7, aq); self.mvo(p); if op & 7 == 6 { 2 } else { 1 } }
            0xB0..=0xB7 => { let p = self.bzu(op & 7, aq); self.mvl(p); if op & 7 == 6 { 2 } else { 1 } }
            0xB8..=0xBF => { let p = self.bzu(op & 7, aq); self.mvk(p); if op & 7 == 6 { 2 } else { 1 } }

            
            0xC0 => { if !self.aca() { self.fz = self.dup(aq); 5 } else { 2 } }
            0xC1 => { let p = self.dup(aq); self.joq(p); 3 }
            0xC2 => { let q = self.bug(aq); if !self.aca() { self.fz = q; 4 } else { 3 } }
            0xC3 => { self.fz = self.bug(aq); 4 }
            0xC4 => { let q = self.bug(aq); if !self.aca() { self.bcf(aq, self.fz); self.fz = q; 6 } else { 3 } }
            0xC5 => { let p = self.atw(); self.bcf(aq, p); 4 }
            0xC6 => { let p = self.auc(aq); self.mvi(p); 2 }
            0xC7 => { self.bcf(aq, self.fz); self.fz = 0x00; 4 }
            0xC8 => { if self.aca() { self.fz = self.dup(aq); 5 } else { 2 } }
            0xC9 => { self.fz = self.dup(aq); 4 }
            0xCA => { let q = self.bug(aq); if self.aca() { self.fz = q; 4 } else { 3 } }
            0xCB => { return self.soj(aq); }
            0xCC => { let q = self.bug(aq); if self.aca() { self.bcf(aq, self.fz); self.fz = q; 6 } else { 3 } }
            0xCD => { let q = self.bug(aq); self.bcf(aq, self.fz); self.fz = q; 6 }
            0xCE => { let p = self.auc(aq); self.mvh(p); 2 }
            0xCF => { self.bcf(aq, self.fz); self.fz = 0x08; 4 }

            0xD0 => { if !self.vq() { self.fz = self.dup(aq); 5 } else { 2 } }
            0xD1 => { let p = self.dup(aq); self.jot(p); 3 }
            0xD2 => { let q = self.bug(aq); if !self.vq() { self.fz = q; 4 } else { 3 } }
            0xD4 => { let q = self.bug(aq); if !self.vq() { self.bcf(aq, self.fz); self.fz = q; 6 } else { 3 } }
            0xD5 => { let p = self.gef(); self.bcf(aq, p); 4 }
            0xD6 => { let p = self.auc(aq); self.mvn(p); 2 }
            0xD7 => { self.bcf(aq, self.fz); self.fz = 0x10; 4 }
            0xD8 => { if self.vq() { self.fz = self.dup(aq); 5 } else { 2 } }
            0xD9 => { self.fz = self.dup(aq); self.dih = true; 4 }
            0xDA => { let q = self.bug(aq); if self.vq() { self.fz = q; 4 } else { 3 } }
            0xDC => { let q = self.bug(aq); if self.vq() { self.bcf(aq, self.fz); self.fz = q; 6 } else { 3 } }
            0xDE => { let p = self.auc(aq); self.mvm(p); 2 }
            0xDF => { self.bcf(aq, self.fz); self.fz = 0x18; 4 }

            0xE0 => { let bo = self.auc(aq); aq.write(0xFF00 | bo as u16, self.q); 3 }
            0xE1 => { let p = self.dup(aq); self.dvp(p); 3 }
            0xE2 => { aq.write(0xFF00 | self.r as u16, self.q); 2 }
            0xE5 => { let p = self.abe(); self.bcf(aq, p); 4 }
            0xE6 => { let p = self.auc(aq); self.mvj(p); 2 }
            0xE7 => { self.bcf(aq, self.fz); self.fz = 0x20; 4 }
            0xE8 => {
                let aa = self.auc(aq) as i8 as i16 as u16;
                let sp = self.sp;
                self.bff(false, false, (sp & 0xF) + (aa & 0xF) > 0xF, (sp & 0xFF) + (aa & 0xFF) > 0xFF);
                self.sp = sp.cn(aa);
                4
            }
            0xE9 => { self.fz = self.abe(); 1 }
            0xEA => { let q = self.bug(aq); aq.write(q, self.q); 4 }
            0xEE => { let p = self.auc(aq); self.mvo(p); 2 }
            0xEF => { self.bcf(aq, self.fz); self.fz = 0x28; 4 }

            0xF0 => { let bo = self.auc(aq); self.q = aq.read(0xFF00 | bo as u16); 3 }
            0xF1 => { let p = self.dup(aq); self.wie(p); 3 }
            0xF2 => { self.q = aq.read(0xFF00 | self.r as u16); 2 }
            0xF3 => { self.dih = false; 1 }
            0xF5 => { let p = self.qfv(); self.bcf(aq, p); 4 }
            0xF6 => { let p = self.auc(aq); self.mvl(p); 2 }
            0xF7 => { self.bcf(aq, self.fz); self.fz = 0x30; 4 }
            0xF8 => {
                let aa = self.auc(aq) as i8 as i16 as u16;
                let sp = self.sp;
                self.bff(false, false, (sp & 0xF) + (aa & 0xF) > 0xF, (sp & 0xFF) + (aa & 0xFF) > 0xFF);
                self.dvp(sp.cn(aa));
                3
            }
            0xF9 => { self.sp = self.abe(); 2 }
            0xFA => { let q = self.bug(aq); self.q = aq.read(q); 4 }
            0xFB => { self.izm = true; 1 }
            0xFE => { let p = self.auc(aq); self.mvk(p); 2 }
            0xFF => { self.bcf(aq, self.fz); self.fz = 0x38; 4 }

            _ => 1, 
        }
    }

    
    fn soj(&mut self, aq: &mut impl Kn) -> u32 {
        let aiv = self.auc(aq);
        let m = aiv & 7;
        let p = self.bzu(m, aq);
        let fmb = m == 6;

        match aiv {
            0x00..=0x07 => { let chf = self.qws(p); self.bxa(m, chf, aq); }
            0x08..=0x0F => { let chf = self.qwu(p); self.bxa(m, chf, aq); }
            0x10..=0x17 => { let chf = self.qwr(p); self.bxa(m, chf, aq); }
            0x18..=0x1F => { let chf = self.qwt(p); self.bxa(m, chf, aq); }
            0x20..=0x27 => { let chf = self.qwv(p); self.bxa(m, chf, aq); }
            0x28..=0x2F => { let chf = self.qww(p); self.bxa(m, chf, aq); }
            0x30..=0x37 => { let chf = self.qwy(p); self.bxa(m, chf, aq); }
            0x38..=0x3F => { let chf = self.qwx(p); self.bxa(m, chf, aq); }
            0x40..=0x7F => {
                let ga = (aiv >> 3) & 7;
                let av = (p >> ga) & 1 == 0;
                self.bb = (if av { CE_ } else { 0 }) | HY_ | (self.bb & AX_);
                return if fmb { 3 } else { 2 };
            }
            0x80..=0xBF => {
                let ga = (aiv >> 3) & 7;
                self.bxa(m, p & !(1 << ga), aq);
            }
            0xC0..=0xFF => {
                let ga = (aiv >> 3) & 7;
                self.bxa(m, p | (1 << ga), aq);
            }
        }
        if fmb { 4 } else { 2 }
    }
}
