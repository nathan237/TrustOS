
#![allow(bgr)]


const AX_: u8 = 0x01; 
const CE_: u8 = 0x02; 
const GL_: u8 = 0x04; 
const ASD_: u8 = 0x08; 
const KB_: u8 = 0x10; 
const GM_: u8 = 0x20; 
const KC_: u8 = 0x40; 
const DR_: u8 = 0x80; 

pub trait Ch {
    fn mc(&mut self, ag: u16) -> u8;
    fn ok(&mut self, ag: u16, ap: u8);
}

#[derive(Clone)]
pub struct Cpu {
    pub q: u8,
    pub b: u8,
    pub c: u8,
    pub sp: u8,
    pub fz: u16,
    pub status: u8,
    pub yl: u64,
    pub jhc: bool,
    pub jay: bool,
    pub ibp: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            q: 0, b: 0, c: 0,
            sp: 0xFD,
            fz: 0,
            status: GM_ | GL_,
            yl: 0,
            jhc: false,
            jay: false,
            ibp: 0,
        }
    }

    pub fn apa(&mut self, aq: &mut impl Ch) {
        let hh = aq.mc(0xFFFC) as u16;
        let gd = aq.mc(0xFFFD) as u16;
        self.fz = (gd << 8) | hh;
        self.sp = 0xFD;
        self.status = GM_ | GL_;
    }

    

    fn cqp(&self, bb: u8) -> bool { self.status & bb != 0 }
    fn ayp(&mut self, bb: u8, ea: bool) { if ea { self.status |= bb; } else { self.status &= !bb; } }
    fn nh(&mut self, p: u8) { self.ayp(CE_, p == 0); self.ayp(DR_, p & 0x80 != 0); }

    fn frt(&mut self, aq: &mut impl Ch, ap: u8) {
        aq.ok(0x0100 | self.sp as u16, ap);
        self.sp = self.sp.nj(1);
    }
    fn bcf(&mut self, aq: &mut impl Ch, ap: u16) {
        self.frt(aq, (ap >> 8) as u8);
        self.frt(aq, ap as u8);
    }
    fn hwd(&mut self, aq: &mut impl Ch) -> u8 {
        self.sp = self.sp.cn(1);
        aq.mc(0x0100 | self.sp as u16)
    }
    fn oyk(&mut self, aq: &mut impl Ch) -> u16 {
        let hh = self.hwd(aq) as u16;
        let gd = self.hwd(aq) as u16;
        (gd << 8) | hh
    }

    

    fn akm(&mut self, aq: &mut impl Ch) -> u8 {
        let p = aq.mc(self.fz); self.fz = self.fz.cn(1); p
    }
    fn aym(&mut self, aq: &mut impl Ch) -> u16 {
        let hh = aq.mc(self.fz) as u16;
        let gd = aq.mc(self.fz.cn(1)) as u16;
        self.fz = self.fz.cn(2);
        (gd << 8) | hh
    }
    fn vrf(&self, aq: &mut impl Ch, ag: u16) -> u16 {
        
        let hh = aq.mc(ag) as u16;
        let tom = (ag & 0xFF00) | ((ag + 1) & 0x00FF);
        let gd = aq.mc(tom) as u16;
        (gd << 8) | hh
    }

    
    fn gf(&mut self, aq: &mut impl Ch) -> u8 { self.akm(aq) }
    fn cvk(&mut self, aq: &mut impl Ch) -> u8 { let q = self.akm(aq) as u16; aq.mc(q) }
    fn fcb(&mut self, aq: &mut impl Ch) -> u8 { let q = self.akm(aq).cn(self.b) as u16; aq.mc(q) }
    fn qbe(&mut self, aq: &mut impl Ch) -> u8 { let q = self.akm(aq).cn(self.c) as u16; aq.mc(q) }
    fn cod(&mut self, aq: &mut impl Ch) -> u8 { let q = self.aym(aq); aq.mc(q) }
    fn dxy(&mut self, aq: &mut impl Ch) -> (u8, u32) {
        let ar = self.aym(aq); let q = ar.cn(self.b as u16);
        let ai = if (ar & 0xFF00) != (q & 0xFF00) { 1 } else { 0 };
        (aq.mc(q), ai)
    }
    fn dxz(&mut self, aq: &mut impl Ch) -> (u8, u32) {
        let ar = self.aym(aq); let q = ar.cn(self.c as u16);
        let ai = if (ar & 0xFF00) != (q & 0xFF00) { 1 } else { 0 };
        (aq.mc(q), ai)
    }
    fn etm(&mut self, aq: &mut impl Ch) -> u8 {
        let av = self.akm(aq).cn(self.b);
        let hh = aq.mc(av as u16) as u16;
        let gd = aq.mc(av.cn(1) as u16) as u16;
        aq.mc((gd << 8) | hh)
    }
    fn etn(&mut self, aq: &mut impl Ch) -> (u8, u32) {
        let av = self.akm(aq);
        let hh = aq.mc(av as u16) as u16;
        let gd = aq.mc(av.cn(1) as u16) as u16;
        let ar = (gd << 8) | hh;
        let q = ar.cn(self.c as u16);
        let ai = if (ar & 0xFF00) != (q & 0xFF00) { 1 } else { 0 };
        (aq.mc(q), ai)
    }

    
    fn ccr(&mut self, aq: &mut impl Ch) -> u16 { self.akm(aq) as u16 }
    fn coc(&mut self, aq: &mut impl Ch) -> u16 { self.akm(aq).cn(self.b) as u16 }
    fn qbd(&mut self, aq: &mut impl Ch) -> u16 { self.akm(aq).cn(self.c) as u16 }
    fn byc(&mut self, aq: &mut impl Ch) -> u16 { self.aym(aq) }
    fn coe(&mut self, aq: &mut impl Ch) -> u16 { let o = self.aym(aq); o.cn(self.b as u16) }
    fn fcg(&mut self, aq: &mut impl Ch) -> u16 { let o = self.aym(aq); o.cn(self.c as u16) }
    fn etl(&mut self, aq: &mut impl Ch) -> u16 {
        let av = self.akm(aq).cn(self.b);
        let hh = aq.mc(av as u16) as u16;
        let gd = aq.mc(av.cn(1) as u16) as u16;
        (gd << 8) | hh
    }
    fn fmf(&mut self, aq: &mut impl Ch) -> u16 {
        let av = self.akm(aq);
        let hh = aq.mc(av as u16) as u16;
        let gd = aq.mc(av.cn(1) as u16) as u16;
        let ar = (gd << 8) | hh;
        ar.cn(self.c as u16)
    }

    

    fn byd(&mut self, p: u8) {
        let q = self.q as u16;
        let ef = p as u16;
        let r = if self.cqp(AX_) { 1u16 } else { 0 };
        let sum = q + ef + r;
        self.ayp(AX_, sum > 0xFF);
        let result = sum as u8;
        self.ayp(KC_, (!(self.q ^ p) & (self.q ^ result)) & 0x80 != 0);
        self.q = result;
        self.nh(self.q);
    }

    fn cho(&mut self, p: u8) { self.byd(!p); }

    fn bgk(&mut self, reg: u8, p: u8) {
        let m = reg.nj(p);
        self.ayp(AX_, reg >= p);
        self.nh(m);
    }

    fn emz(&mut self, aq: &mut impl Ch, mo: bool) -> u32 {
        let l = self.akm(aq) as i8;
        if mo {
            let oqc = self.fz.cn(l as u16);
            let vgo = if (self.fz & 0xFF00) != (oqc & 0xFF00) { 1 } else { 0 };
            self.fz = oqc;
            3 + vgo
        } else { 2 }
    }

    fn cvs(&mut self, p: u8) -> u8 {
        self.ayp(AX_, p & 0x80 != 0);
        let m = p << 1; self.nh(m); m
    }
    fn czj(&mut self, p: u8) -> u8 {
        self.ayp(AX_, p & 0x01 != 0);
        let m = p >> 1; self.nh(m); m
    }
    fn dbj(&mut self, p: u8) -> u8 {
        let r = if self.cqp(AX_) { 1u8 } else { 0 };
        self.ayp(AX_, p & 0x80 != 0);
        let m = (p << 1) | r; self.nh(m); m
    }
    fn dbk(&mut self, p: u8) -> u8 {
        let r = if self.cqp(AX_) { 0x80u8 } else { 0 };
        self.ayp(AX_, p & 0x01 != 0);
        let m = (p >> 1) | r; self.nh(m); m
    }

    

    pub fn gu(&mut self, aq: &mut impl Ch) -> u32 {
        if self.ibp > 0 { self.ibp -= 1; return 1; }

        
        if self.jhc {
            self.jhc = false;
            self.bcf(aq, self.fz);
            self.frt(aq, (self.status | GM_) & !KB_);
            self.ayp(GL_, true);
            let hh = aq.mc(0xFFFA) as u16;
            let gd = aq.mc(0xFFFB) as u16;
            self.fz = (gd << 8) | hh;
            self.yl += 7;
            return 7;
        }

        
        if self.jay && !self.cqp(GL_) {
            self.jay = false;
            self.bcf(aq, self.fz);
            self.frt(aq, (self.status | GM_) & !KB_);
            self.ayp(GL_, true);
            let hh = aq.mc(0xFFFE) as u16;
            let gd = aq.mc(0xFFFF) as u16;
            self.fz = (gd << 8) | hh;
            self.yl += 7;
            return 7;
        }

        let opcode = aq.mc(self.fz);
        self.fz = self.fz.cn(1);

        let yl = match opcode {
            
            0xA9 => { let p = self.gf(aq); self.q = p; self.nh(p); 2 }
            0xA5 => { let p = self.cvk(aq); self.q = p; self.nh(p); 3 }
            0xB5 => { let p = self.fcb(aq); self.q = p; self.nh(p); 4 }
            0xAD => { let p = self.cod(aq); self.q = p; self.nh(p); 4 }
            0xBD => { let (p, ai) = self.dxy(aq); self.q = p; self.nh(p); 4 + ai }
            0xB9 => { let (p, ai) = self.dxz(aq); self.q = p; self.nh(p); 4 + ai }
            0xA1 => { let p = self.etm(aq); self.q = p; self.nh(p); 6 }
            0xB1 => { let (p, ai) = self.etn(aq); self.q = p; self.nh(p); 5 + ai }

            
            0xA2 => { let p = self.gf(aq); self.b = p; self.nh(p); 2 }
            0xA6 => { let p = self.cvk(aq); self.b = p; self.nh(p); 3 }
            0xB6 => { let p = self.qbe(aq); self.b = p; self.nh(p); 4 }
            0xAE => { let p = self.cod(aq); self.b = p; self.nh(p); 4 }
            0xBE => { let (p, ai) = self.dxz(aq); self.b = p; self.nh(p); 4 + ai }

            
            0xA0 => { let p = self.gf(aq); self.c = p; self.nh(p); 2 }
            0xA4 => { let p = self.cvk(aq); self.c = p; self.nh(p); 3 }
            0xB4 => { let p = self.fcb(aq); self.c = p; self.nh(p); 4 }
            0xAC => { let p = self.cod(aq); self.c = p; self.nh(p); 4 }
            0xBC => { let (p, ai) = self.dxy(aq); self.c = p; self.nh(p); 4 + ai }

            
            0x85 => { let q = self.ccr(aq); aq.ok(q, self.q); 3 }
            0x95 => { let q = self.coc(aq); aq.ok(q, self.q); 4 }
            0x8D => { let q = self.byc(aq); aq.ok(q, self.q); 4 }
            0x9D => { let q = self.coe(aq); aq.ok(q, self.q); 5 }
            0x99 => { let q = self.fcg(aq); aq.ok(q, self.q); 5 }
            0x81 => { let q = self.etl(aq); aq.ok(q, self.q); 6 }
            0x91 => { let q = self.fmf(aq); aq.ok(q, self.q); 6 }

            
            0x86 => { let q = self.ccr(aq); aq.ok(q, self.b); 3 }
            0x96 => { let q = self.qbd(aq); aq.ok(q, self.b); 4 }
            0x8E => { let q = self.byc(aq); aq.ok(q, self.b); 4 }

            
            0x84 => { let q = self.ccr(aq); aq.ok(q, self.c); 3 }
            0x94 => { let q = self.coc(aq); aq.ok(q, self.c); 4 }
            0x8C => { let q = self.byc(aq); aq.ok(q, self.c); 4 }

            
            0x69 => { let p = self.gf(aq); self.byd(p); 2 }
            0x65 => { let p = self.cvk(aq); self.byd(p); 3 }
            0x75 => { let p = self.fcb(aq); self.byd(p); 4 }
            0x6D => { let p = self.cod(aq); self.byd(p); 4 }
            0x7D => { let (p, ai) = self.dxy(aq); self.byd(p); 4 + ai }
            0x79 => { let (p, ai) = self.dxz(aq); self.byd(p); 4 + ai }
            0x61 => { let p = self.etm(aq); self.byd(p); 6 }
            0x71 => { let (p, ai) = self.etn(aq); self.byd(p); 5 + ai }

            
            0xE9 | 0xEB => { let p = self.gf(aq); self.cho(p); 2 }
            0xE5 => { let p = self.cvk(aq); self.cho(p); 3 }
            0xF5 => { let p = self.fcb(aq); self.cho(p); 4 }
            0xED => { let p = self.cod(aq); self.cho(p); 4 }
            0xFD => { let (p, ai) = self.dxy(aq); self.cho(p); 4 + ai }
            0xF9 => { let (p, ai) = self.dxz(aq); self.cho(p); 4 + ai }
            0xE1 => { let p = self.etm(aq); self.cho(p); 6 }
            0xF1 => { let (p, ai) = self.etn(aq); self.cho(p); 5 + ai }

            
            0x29 => { let p = self.gf(aq); self.q &= p; self.nh(self.q); 2 }
            0x25 => { let p = self.cvk(aq); self.q &= p; self.nh(self.q); 3 }
            0x35 => { let p = self.fcb(aq); self.q &= p; self.nh(self.q); 4 }
            0x2D => { let p = self.cod(aq); self.q &= p; self.nh(self.q); 4 }
            0x3D => { let (p, ai) = self.dxy(aq); self.q &= p; self.nh(self.q); 4 + ai }
            0x39 => { let (p, ai) = self.dxz(aq); self.q &= p; self.nh(self.q); 4 + ai }
            0x21 => { let p = self.etm(aq); self.q &= p; self.nh(self.q); 6 }
            0x31 => { let (p, ai) = self.etn(aq); self.q &= p; self.nh(self.q); 5 + ai }

            
            0x09 => { let p = self.gf(aq); self.q |= p; self.nh(self.q); 2 }
            0x05 => { let p = self.cvk(aq); self.q |= p; self.nh(self.q); 3 }
            0x15 => { let p = self.fcb(aq); self.q |= p; self.nh(self.q); 4 }
            0x0D => { let p = self.cod(aq); self.q |= p; self.nh(self.q); 4 }
            0x1D => { let (p, ai) = self.dxy(aq); self.q |= p; self.nh(self.q); 4 + ai }
            0x19 => { let (p, ai) = self.dxz(aq); self.q |= p; self.nh(self.q); 4 + ai }
            0x01 => { let p = self.etm(aq); self.q |= p; self.nh(self.q); 6 }
            0x11 => { let (p, ai) = self.etn(aq); self.q |= p; self.nh(self.q); 5 + ai }

            
            0x49 => { let p = self.gf(aq); self.q ^= p; self.nh(self.q); 2 }
            0x45 => { let p = self.cvk(aq); self.q ^= p; self.nh(self.q); 3 }
            0x55 => { let p = self.fcb(aq); self.q ^= p; self.nh(self.q); 4 }
            0x4D => { let p = self.cod(aq); self.q ^= p; self.nh(self.q); 4 }
            0x5D => { let (p, ai) = self.dxy(aq); self.q ^= p; self.nh(self.q); 4 + ai }
            0x59 => { let (p, ai) = self.dxz(aq); self.q ^= p; self.nh(self.q); 4 + ai }
            0x41 => { let p = self.etm(aq); self.q ^= p; self.nh(self.q); 6 }
            0x51 => { let (p, ai) = self.etn(aq); self.q ^= p; self.nh(self.q); 5 + ai }

            
            0xC9 => { let p = self.gf(aq); self.bgk(self.q, p); 2 }
            0xC5 => { let p = self.cvk(aq); self.bgk(self.q, p); 3 }
            0xD5 => { let p = self.fcb(aq); self.bgk(self.q, p); 4 }
            0xCD => { let p = self.cod(aq); self.bgk(self.q, p); 4 }
            0xDD => { let (p, ai) = self.dxy(aq); self.bgk(self.q, p); 4 + ai }
            0xD9 => { let (p, ai) = self.dxz(aq); self.bgk(self.q, p); 4 + ai }
            0xC1 => { let p = self.etm(aq); self.bgk(self.q, p); 6 }
            0xD1 => { let (p, ai) = self.etn(aq); self.bgk(self.q, p); 5 + ai }

            
            0xE0 => { let p = self.gf(aq); self.bgk(self.b, p); 2 }
            0xE4 => { let p = self.cvk(aq); self.bgk(self.b, p); 3 }
            0xEC => { let p = self.cod(aq); self.bgk(self.b, p); 4 }

            
            0xC0 => { let p = self.gf(aq); self.bgk(self.c, p); 2 }
            0xC4 => { let p = self.cvk(aq); self.bgk(self.c, p); 3 }
            0xCC => { let p = self.cod(aq); self.bgk(self.c, p); 4 }

            
            0x24 => { let p = self.cvk(aq); self.ayp(CE_, self.q & p == 0); self.ayp(KC_, p & 0x40 != 0); self.ayp(DR_, p & 0x80 != 0); 3 }
            0x2C => { let p = self.cod(aq); self.ayp(CE_, self.q & p == 0); self.ayp(KC_, p & 0x40 != 0); self.ayp(DR_, p & 0x80 != 0); 4 }

            
            0x0A => { self.q = self.cvs(self.q); 2 }
            0x06 => { let q = self.ccr(aq); let p = aq.mc(q); let m = self.cvs(p); aq.ok(q, m); 5 }
            0x16 => { let q = self.coc(aq); let p = aq.mc(q); let m = self.cvs(p); aq.ok(q, m); 6 }
            0x0E => { let q = self.byc(aq); let p = aq.mc(q); let m = self.cvs(p); aq.ok(q, m); 6 }
            0x1E => { let q = self.coe(aq); let p = aq.mc(q); let m = self.cvs(p); aq.ok(q, m); 7 }

            
            0x4A => { self.q = self.czj(self.q); 2 }
            0x46 => { let q = self.ccr(aq); let p = aq.mc(q); let m = self.czj(p); aq.ok(q, m); 5 }
            0x56 => { let q = self.coc(aq); let p = aq.mc(q); let m = self.czj(p); aq.ok(q, m); 6 }
            0x4E => { let q = self.byc(aq); let p = aq.mc(q); let m = self.czj(p); aq.ok(q, m); 6 }
            0x5E => { let q = self.coe(aq); let p = aq.mc(q); let m = self.czj(p); aq.ok(q, m); 7 }

            
            0x2A => { self.q = self.dbj(self.q); 2 }
            0x26 => { let q = self.ccr(aq); let p = aq.mc(q); let m = self.dbj(p); aq.ok(q, m); 5 }
            0x36 => { let q = self.coc(aq); let p = aq.mc(q); let m = self.dbj(p); aq.ok(q, m); 6 }
            0x2E => { let q = self.byc(aq); let p = aq.mc(q); let m = self.dbj(p); aq.ok(q, m); 6 }
            0x3E => { let q = self.coe(aq); let p = aq.mc(q); let m = self.dbj(p); aq.ok(q, m); 7 }

            
            0x6A => { self.q = self.dbk(self.q); 2 }
            0x66 => { let q = self.ccr(aq); let p = aq.mc(q); let m = self.dbk(p); aq.ok(q, m); 5 }
            0x76 => { let q = self.coc(aq); let p = aq.mc(q); let m = self.dbk(p); aq.ok(q, m); 6 }
            0x6E => { let q = self.byc(aq); let p = aq.mc(q); let m = self.dbk(p); aq.ok(q, m); 6 }
            0x7E => { let q = self.coe(aq); let p = aq.mc(q); let m = self.dbk(p); aq.ok(q, m); 7 }

            
            0xE6 => { let q = self.ccr(aq); let p = aq.mc(q).cn(1); self.nh(p); aq.ok(q, p); 5 }
            0xF6 => { let q = self.coc(aq); let p = aq.mc(q).cn(1); self.nh(p); aq.ok(q, p); 6 }
            0xEE => { let q = self.byc(aq); let p = aq.mc(q).cn(1); self.nh(p); aq.ok(q, p); 6 }
            0xFE => { let q = self.coe(aq); let p = aq.mc(q).cn(1); self.nh(p); aq.ok(q, p); 7 }

            
            0xC6 => { let q = self.ccr(aq); let p = aq.mc(q).nj(1); self.nh(p); aq.ok(q, p); 5 }
            0xD6 => { let q = self.coc(aq); let p = aq.mc(q).nj(1); self.nh(p); aq.ok(q, p); 6 }
            0xCE => { let q = self.byc(aq); let p = aq.mc(q).nj(1); self.nh(p); aq.ok(q, p); 6 }
            0xDE => { let q = self.coe(aq); let p = aq.mc(q).nj(1); self.nh(p); aq.ok(q, p); 7 }

            
            0xE8 => { self.b = self.b.cn(1); self.nh(self.b); 2 }
            0xC8 => { self.c = self.c.cn(1); self.nh(self.c); 2 }
            0xCA => { self.b = self.b.nj(1); self.nh(self.b); 2 }
            0x88 => { self.c = self.c.nj(1); self.nh(self.c); 2 }

            
            0xAA => { self.b = self.q; self.nh(self.b); 2 }  
            0xA8 => { self.c = self.q; self.nh(self.c); 2 }  
            0x8A => { self.q = self.b; self.nh(self.q); 2 }  
            0x98 => { self.q = self.c; self.nh(self.q); 2 }  
            0xBA => { self.b = self.sp; self.nh(self.b); 2 }  
            0x9A => { self.sp = self.b; 2 }                        

            
            0x90 => self.emz(aq, !self.cqp(AX_)),  
            0xB0 => self.emz(aq, self.cqp(AX_)),   
            0xF0 => self.emz(aq, self.cqp(CE_)),   
            0xD0 => self.emz(aq, !self.cqp(CE_)),  
            0x30 => self.emz(aq, self.cqp(DR_)),   
            0x10 => self.emz(aq, !self.cqp(DR_)),  
            0x50 => self.emz(aq, !self.cqp(KC_)),  
            0x70 => self.emz(aq, self.cqp(KC_)),   

            
            0x4C => { self.fz = self.aym(aq); 3 }
            0x6C => { let q = self.aym(aq); self.fz = self.vrf(aq, q); 5 }

            
            0x20 => { let q = self.aym(aq); self.bcf(aq, self.fz.nj(1)); self.fz = q; 6 }
            0x60 => { self.fz = self.oyk(aq).cn(1); 6 }
            0x40 => {
                self.status = (self.hwd(aq) & !KB_) | GM_;
                self.fz = self.oyk(aq);
                4
            }

            
            0x48 => { self.frt(aq, self.q); 3 }           
            0x08 => { self.frt(aq, self.status | KB_ | GM_); 3 } 
            0x68 => { self.q = self.hwd(aq); self.nh(self.q); 4 }  
            0x28 => { self.status = (self.hwd(aq) & !KB_) | GM_; 4 } 

            
            0x18 => { self.ayp(AX_, false); 2 } 
            0x38 => { self.ayp(AX_, true); 2 }  
            0x58 => { self.ayp(GL_, false); 2 } 
            0x78 => { self.ayp(GL_, true); 2 }  
            0xD8 => { self.ayp(ASD_, false); 2 } 
            0xF8 => { self.ayp(ASD_, true); 2 }  
            0xB8 => { self.ayp(KC_, false); 2 } 

            
            0x00 => {
                self.fz = self.fz.cn(1);
                self.bcf(aq, self.fz);
                self.frt(aq, self.status | KB_ | GM_);
                self.ayp(GL_, true);
                let hh = aq.mc(0xFFFE) as u16;
                let gd = aq.mc(0xFFFF) as u16;
                self.fz = (gd << 8) | hh;
                7
            }

            
            0xEA => 2,

            
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => 2,
            0x04 | 0x44 | 0x64 => { self.fz = self.fz.cn(1); 3 }
            0x0C => { self.fz = self.fz.cn(2); 4 }
            0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => { self.fz = self.fz.cn(1); 4 }
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                let (_, ai) = self.dxy(aq); 4 + ai
            }
            0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => { self.fz = self.fz.cn(1); 2 }

            
            0xA7 => { let p = self.cvk(aq); self.q = p; self.b = p; self.nh(p); 3 }
            0xB7 => { let p = self.qbe(aq); self.q = p; self.b = p; self.nh(p); 4 }
            0xAF => { let p = self.cod(aq); self.q = p; self.b = p; self.nh(p); 4 }
            0xBF => { let (p, ai) = self.dxz(aq); self.q = p; self.b = p; self.nh(p); 4 + ai }
            0xA3 => { let p = self.etm(aq); self.q = p; self.b = p; self.nh(p); 6 }
            0xB3 => { let (p, ai) = self.etn(aq); self.q = p; self.b = p; self.nh(p); 5 + ai }

            
            0x87 => { let q = self.ccr(aq); aq.ok(q, self.q & self.b); 3 }
            0x97 => { let q = self.qbd(aq); aq.ok(q, self.q & self.b); 4 }
            0x8F => { let q = self.byc(aq); aq.ok(q, self.q & self.b); 4 }
            0x83 => { let q = self.etl(aq); aq.ok(q, self.q & self.b); 6 }

            
            0xC7 => { let q = self.ccr(aq); let p = aq.mc(q).nj(1); aq.ok(q, p); self.bgk(self.q, p); 5 }
            0xD7 => { let q = self.coc(aq); let p = aq.mc(q).nj(1); aq.ok(q, p); self.bgk(self.q, p); 6 }
            0xCF => { let q = self.byc(aq); let p = aq.mc(q).nj(1); aq.ok(q, p); self.bgk(self.q, p); 6 }
            0xDF => { let q = self.coe(aq); let p = aq.mc(q).nj(1); aq.ok(q, p); self.bgk(self.q, p); 7 }
            0xDB => { let q = self.fcg(aq); let p = aq.mc(q).nj(1); aq.ok(q, p); self.bgk(self.q, p); 7 }
            0xC3 => { let q = self.etl(aq); let p = aq.mc(q).nj(1); aq.ok(q, p); self.bgk(self.q, p); 8 }
            0xD3 => { let q = self.fmf(aq); let p = aq.mc(q).nj(1); aq.ok(q, p); self.bgk(self.q, p); 8 }

            
            0xE7 => { let q = self.ccr(aq); let p = aq.mc(q).cn(1); aq.ok(q, p); self.cho(p); 5 }
            0xF7 => { let q = self.coc(aq); let p = aq.mc(q).cn(1); aq.ok(q, p); self.cho(p); 6 }
            0xEF => { let q = self.byc(aq); let p = aq.mc(q).cn(1); aq.ok(q, p); self.cho(p); 6 }
            0xFF => { let q = self.coe(aq); let p = aq.mc(q).cn(1); aq.ok(q, p); self.cho(p); 7 }
            0xFB => { let q = self.fcg(aq); let p = aq.mc(q).cn(1); aq.ok(q, p); self.cho(p); 7 }
            0xE3 => { let q = self.etl(aq); let p = aq.mc(q).cn(1); aq.ok(q, p); self.cho(p); 8 }
            0xF3 => { let q = self.fmf(aq); let p = aq.mc(q).cn(1); aq.ok(q, p); self.cho(p); 8 }

            
            0x07 => { let q = self.ccr(aq); let p = aq.mc(q); let m = self.cvs(p); aq.ok(q, m); self.q |= m; self.nh(self.q); 5 }
            0x17 => { let q = self.coc(aq); let p = aq.mc(q); let m = self.cvs(p); aq.ok(q, m); self.q |= m; self.nh(self.q); 6 }
            0x0F => { let q = self.byc(aq); let p = aq.mc(q); let m = self.cvs(p); aq.ok(q, m); self.q |= m; self.nh(self.q); 6 }
            0x1F => { let q = self.coe(aq); let p = aq.mc(q); let m = self.cvs(p); aq.ok(q, m); self.q |= m; self.nh(self.q); 7 }
            0x1B => { let q = self.fcg(aq); let p = aq.mc(q); let m = self.cvs(p); aq.ok(q, m); self.q |= m; self.nh(self.q); 7 }
            0x03 => { let q = self.etl(aq); let p = aq.mc(q); let m = self.cvs(p); aq.ok(q, m); self.q |= m; self.nh(self.q); 8 }
            0x13 => { let q = self.fmf(aq); let p = aq.mc(q); let m = self.cvs(p); aq.ok(q, m); self.q |= m; self.nh(self.q); 8 }

            
            0x27 => { let q = self.ccr(aq); let p = aq.mc(q); let m = self.dbj(p); aq.ok(q, m); self.q &= m; self.nh(self.q); 5 }
            0x37 => { let q = self.coc(aq); let p = aq.mc(q); let m = self.dbj(p); aq.ok(q, m); self.q &= m; self.nh(self.q); 6 }
            0x2F => { let q = self.byc(aq); let p = aq.mc(q); let m = self.dbj(p); aq.ok(q, m); self.q &= m; self.nh(self.q); 6 }
            0x3F => { let q = self.coe(aq); let p = aq.mc(q); let m = self.dbj(p); aq.ok(q, m); self.q &= m; self.nh(self.q); 7 }
            0x3B => { let q = self.fcg(aq); let p = aq.mc(q); let m = self.dbj(p); aq.ok(q, m); self.q &= m; self.nh(self.q); 7 }
            0x23 => { let q = self.etl(aq); let p = aq.mc(q); let m = self.dbj(p); aq.ok(q, m); self.q &= m; self.nh(self.q); 8 }
            0x33 => { let q = self.fmf(aq); let p = aq.mc(q); let m = self.dbj(p); aq.ok(q, m); self.q &= m; self.nh(self.q); 8 }

            
            0x47 => { let q = self.ccr(aq); let p = aq.mc(q); let m = self.czj(p); aq.ok(q, m); self.q ^= m; self.nh(self.q); 5 }
            0x57 => { let q = self.coc(aq); let p = aq.mc(q); let m = self.czj(p); aq.ok(q, m); self.q ^= m; self.nh(self.q); 6 }
            0x4F => { let q = self.byc(aq); let p = aq.mc(q); let m = self.czj(p); aq.ok(q, m); self.q ^= m; self.nh(self.q); 6 }
            0x5F => { let q = self.coe(aq); let p = aq.mc(q); let m = self.czj(p); aq.ok(q, m); self.q ^= m; self.nh(self.q); 7 }
            0x5B => { let q = self.fcg(aq); let p = aq.mc(q); let m = self.czj(p); aq.ok(q, m); self.q ^= m; self.nh(self.q); 7 }
            0x43 => { let q = self.etl(aq); let p = aq.mc(q); let m = self.czj(p); aq.ok(q, m); self.q ^= m; self.nh(self.q); 8 }
            0x53 => { let q = self.fmf(aq); let p = aq.mc(q); let m = self.czj(p); aq.ok(q, m); self.q ^= m; self.nh(self.q); 8 }

            
            0x67 => { let q = self.ccr(aq); let p = aq.mc(q); let m = self.dbk(p); aq.ok(q, m); self.byd(m); 5 }
            0x77 => { let q = self.coc(aq); let p = aq.mc(q); let m = self.dbk(p); aq.ok(q, m); self.byd(m); 6 }
            0x6F => { let q = self.byc(aq); let p = aq.mc(q); let m = self.dbk(p); aq.ok(q, m); self.byd(m); 6 }
            0x7F => { let q = self.coe(aq); let p = aq.mc(q); let m = self.dbk(p); aq.ok(q, m); self.byd(m); 7 }
            0x7B => { let q = self.fcg(aq); let p = aq.mc(q); let m = self.dbk(p); aq.ok(q, m); self.byd(m); 7 }
            0x63 => { let q = self.etl(aq); let p = aq.mc(q); let m = self.dbk(p); aq.ok(q, m); self.byd(m); 8 }
            0x73 => { let q = self.fmf(aq); let p = aq.mc(q); let m = self.dbk(p); aq.ok(q, m); self.byd(m); 8 }

            
            _ => {
                
                1
            }
        };

        self.yl += yl as u64;
        yl
    }
}
