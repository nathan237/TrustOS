






use alloc::vec::Vec;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Reg {
    J = 0, Fe = 1, Axm = 2, Ckf = 3,
    Qo = 4, Qn = 5, Brf = 6, Bql = 7,
    Alo = 8,  Alp = 9,  Alj = 10, Alk = 11,
    All = 12, Alm = 13, Aln = 14, Aec = 15,
}

impl Reg {
    
    #[inline]
    pub fn ael(self) -> u8 { (self as u8) & 0x07 }
    
    #[inline]
    pub fn evf(self) -> bool { (self as u8) >= 8 }
}


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Cc {
    Se  = 0x04, 
    Adl = 0x05, 
    Aur  = 0x0C, 
    Wr = 0x0D, 
    Te = 0x0E, 
    Aii  = 0x0F, 
    Byb  = 0x02, 
    Crj = 0x03, 
    Cry = 0x06, 
    Bxd  = 0x07, 
}


#[derive(Debug, Clone, Copy)]
pub struct Dy(pub usize);


#[derive(Debug)]
pub struct Ala {
    pub l: usize,  
    pub cu: Dy,    
}


pub struct X86Asm {
    pub aj: Vec<u8>,
    pub cze: Vec<Option<usize>>, 
    pub huo: Vec<Ala>,
}

impl X86Asm {
    pub fn new() -> Self {
        Self {
            aj: Vec::fc(4096),
            cze: Vec::new(),
            huo: Vec::new(),
        }
    }

    
    #[inline]
    pub fn l(&self) -> usize { self.aj.len() }

    
    pub fn dtl(&mut self) -> Dy {
        let ad = self.cze.len();
        self.cze.push(None);
        Dy(ad)
    }

    
    pub fn deg(&mut self, cu: Dy) {
        self.cze[cu.0] = Some(self.aj.len());
    }

    
    pub fn vxw(&mut self) -> Result<(), &'static str> {
        for jiw in &self.huo {
            let cd = self.cze[jiw.cu.0].ok_or("unresolved label")?;
            let adj = cd as i64 - (jiw.l as i64 + 4); 
            let vui = adj as i32;
            let bf = vui.ho();
            self.aj[jiw.l..jiw.l + 4].dg(&bf);
        }
        Ok(())
    }

    

    
    fn ako(&mut self, hb: Reg, reg: Reg) {
        let mut aip: u8 = 0x48;
        if reg.evf() { aip |= 0x04; } 
        if hb.evf()  { aip |= 0x01; } 
        self.aj.push(aip);
    }

    fn ehr(&mut self, hb: Reg) {
        let mut aip: u8 = 0x48;
        if hb.evf() { aip |= 0x01; }
        self.aj.push(aip);
    }

    

    pub fn ms(ev: u8, reg: u8, hb: u8) -> u8 {
        (ev << 6) | ((reg & 7) << 3) | (hb & 7)
    }

    

    
    pub fn dkx(&mut self, m: Reg) {
        if m.evf() { self.aj.push(0x41); }
        self.aj.push(0x50 + m.ael());
    }

    
    pub fn clz(&mut self, m: Reg) {
        if m.evf() { self.aj.push(0x41); }
        self.aj.push(0x58 + m.ael());
    }

    
    pub fn lmu(&mut self, cs: Reg, gf: i64) {
        self.ehr(cs);
        self.aj.push(0xB8 + cs.ael());
        self.aj.bk(&gf.ho());
    }

    
    pub fn gmz(&mut self, cs: Reg, gf: i32) {
        self.ehr(cs);
        self.aj.push(0xC7);
        self.aj.push(Self::ms(0b11, 0, cs.ael()));
        self.aj.bk(&gf.ho());
    }

    
    pub fn jgg(&mut self, cs: Reg, cy: Reg) {
        self.ako(cs, cy);
        self.aj.push(0x89);
        self.aj.push(Self::ms(0b11, cy.ael(), cs.ael()));
    }

    
    pub fn hrz(&mut self, cs: Reg, l: i32) {
        self.ako(Reg::Qn, cs);
        self.aj.push(0x8B);
        if l >= -128 && l <= 127 {
            self.aj.push(Self::ms(0b01, cs.ael(), 0x05));
            self.aj.push(l as u8);
        } else {
            self.aj.push(Self::ms(0b10, cs.ael(), 0x05));
            self.aj.bk(&l.ho());
        }
    }

    
    pub fn gna(&mut self, l: i32, cy: Reg) {
        self.ako(Reg::Qn, cy);
        self.aj.push(0x89);
        if l >= -128 && l <= 127 {
            self.aj.push(Self::ms(0b01, cy.ael(), 0x05));
            self.aj.push(l as u8);
        } else {
            self.aj.push(Self::ms(0b10, cy.ael(), 0x05));
            self.aj.bk(&l.ho());
        }
    }

    
    pub fn qfm(&mut self, cs: Reg, cy: Reg) {
        self.ako(cs, cy);
        self.aj.push(0x01);
        self.aj.push(Self::ms(0b11, cy.ael(), cs.ael()));
    }

    
    pub fn wvo(&mut self, cs: Reg, cy: Reg) {
        self.ako(cs, cy);
        self.aj.push(0x29);
        self.aj.push(Self::ms(0b11, cy.ael(), cs.ael()));
    }

    
    pub fn tsk(&mut self, cs: Reg, cy: Reg) {
        self.ako(cy, cs);
        self.aj.push(0x0F);
        self.aj.push(0xAF);
        self.aj.push(Self::ms(0b11, cs.ael(), cy.ael()));
    }

    
    pub fn ngw(&mut self) {
        self.aj.push(0x48);
        self.aj.push(0x99);
    }

    
    pub fn odd(&mut self, cy: Reg) {
        self.ehr(cy);
        self.aj.push(0xF7);
        self.aj.push(Self::ms(0b11, 7, cy.ael()));
    }

    
    pub fn ury(&mut self, m: Reg) {
        self.ehr(m);
        self.aj.push(0xF7);
        self.aj.push(Self::ms(0b11, 3, m.ael()));
    }

    
    pub fn mvs(&mut self, cs: Reg, cy: Reg) {
        self.ako(cs, cy);
        self.aj.push(0x21);
        self.aj.push(Self::ms(0b11, cy.ael(), cs.ael()));
    }

    
    pub fn osw(&mut self, cs: Reg, cy: Reg) {
        self.ako(cs, cy);
        self.aj.push(0x09);
        self.aj.push(Self::ms(0b11, cy.ael(), cs.ael()));
    }

    
    pub fn xwp(&mut self, cs: Reg, cy: Reg) {
        self.ako(cs, cy);
        self.aj.push(0x31);
        self.aj.push(Self::ms(0b11, cy.ael(), cs.ael()));
    }

    
    pub fn wmy(&mut self, cs: Reg) {
        self.ehr(cs);
        self.aj.push(0xD3);
        self.aj.push(Self::ms(0b11, 4, cs.ael()));
    }

    
    pub fn wcr(&mut self, cs: Reg) {
        self.ehr(cs);
        self.aj.push(0xD3);
        self.aj.push(Self::ms(0b11, 7, cs.ael()));
    }

    
    pub fn fff(&mut self, q: Reg, o: Reg) {
        self.ako(q, o);
        self.aj.push(0x39);
        self.aj.push(Self::ms(0b11, o.ael(), q.ael()));
    }

    
    pub fn yjk(&mut self, m: Reg, gf: i32) {
        self.ehr(m);
        if m == Reg::J {
            self.aj.push(0x3D);
        } else {
            self.aj.push(0x81);
            self.aj.push(Self::ms(0b11, 7, m.ael()));
        }
        self.aj.bk(&gf.ho());
    }

    
    pub fn mkj(&mut self, q: Reg, o: Reg) {
        self.ako(q, o);
        self.aj.push(0x85);
        self.aj.push(Self::ms(0b11, o.ael(), q.ael()));
    }

    
    pub fn ful(&mut self, nn: Cc, cs: Reg) {
        if cs.evf() {
            self.aj.push(0x41);
        } else {
            
            self.aj.push(0x40);
        }
        self.aj.push(0x0F);
        self.aj.push(0x90 + nn as u8);
        self.aj.push(Self::ms(0b11, 0, cs.ael()));
    }

    
    pub fn fop(&mut self, cs: Reg, cy: Reg) {
        self.ako(cy, cs);
        self.aj.push(0x0F);
        self.aj.push(0xB6);
        self.aj.push(Self::ms(0b11, cs.ael(), cy.ael()));
    }

    
    pub fn jzg(&mut self, cs: Reg, gf: i32) {
        self.ehr(cs);
        if cs == Reg::J {
            self.aj.push(0x05);
        } else {
            self.aj.push(0x81);
            self.aj.push(Self::ms(0b11, 0, cs.ael()));
        }
        self.aj.bk(&gf.ho());
    }

    
    pub fn ppl(&mut self, cs: Reg, gf: i32) {
        self.ehr(cs);
        if cs == Reg::J {
            self.aj.push(0x2D);
        } else {
            self.aj.push(0x81);
            self.aj.push(Self::ms(0b11, 5, cs.ael()));
        }
        self.aj.bk(&gf.ho());
    }

    
    pub fn gko(&mut self, cu: Dy) {
        self.aj.push(0xE9);
        let dz = self.aj.len();
        self.aj.bk(&0i32.ho()); 
        self.huo.push(Ala { l: dz, cu });
    }

    
    pub fn lgu(&mut self, nn: Cc, cu: Dy) {
        self.aj.push(0x0F);
        self.aj.push(0x80 + nn as u8);
        let dz = self.aj.len();
        self.aj.bk(&0i32.ho()); 
        self.huo.push(Ala { l: dz, cu });
    }

    
    pub fn nbl(&mut self, cu: Dy) {
        self.aj.push(0xE8);
        let dz = self.aj.len();
        self.aj.bk(&0i32.ho());
        self.huo.push(Ala { l: dz, cu });
    }

    
    pub fn qvo(&mut self, m: Reg) {
        if m.evf() { self.aj.push(0x41); }
        self.aj.push(0xFF);
        self.aj.push(Self::ms(0b11, 2, m.ael()));
    }

    
    pub fn aux(&mut self) {
        self.aj.push(0xC3);
    }

    
    pub fn oqw(&mut self) {
        self.aj.push(0x90);
    }

    
    pub fn zam(&mut self, cs: Reg, l: i32) {
        self.ako(Reg::Qn, cs);
        self.aj.push(0x8D);
        if l >= -128 && l <= 127 {
            self.aj.push(Self::ms(0b01, cs.ael(), 0x05));
            self.aj.push(l as u8);
        } else {
            self.aj.push(Self::ms(0b10, cs.ael(), 0x05));
            self.aj.bk(&l.ho());
        }
    }

    

    
    pub fn vne(&mut self, bzt: i32) {
        self.dkx(Reg::Qn);
        self.jgg(Reg::Qn, Reg::Qo);
        if bzt > 0 {
            self.ppl(Reg::Qo, bzt);
        }
    }

    
    pub fn nqw(&mut self) {
        self.jgg(Reg::Qo, Reg::Qn);
        self.clz(Reg::Qn);
        self.aux();
    }
}
