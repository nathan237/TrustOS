

#![allow(bgr)]

use alloc::vec;
use alloc::vec::Vec;


pub const KF_: [u32; 4] = [
    0xFFE0F8D0, 
    0xFF88C070, 
    0xFF346856, 
    0xFF081820, 
];

pub const EQ_: usize = 160;
pub const AHM_: usize = 144;

pub struct Gpu {
    pub aof: [u8; 8192],     
    pub dnb: [u8; 8192],    
    pub awh: [u8; 160],       
    pub framebuffer: Vec<u32>, 

    
    pub amh: u8,   
    pub hm: u8,   
    pub eyf: u8,    
    pub eye: u8,    
    pub ct: u8,     
    pub eey: u8,    
    pub emt: u8,    
    pub fpm: u8,   
    pub fpn: u8,   
    pub lw: u8,     
    pub fx: u8,     

    pub ev: u8,       
    pub yl: u32,    
    pub hkj: bool,
    pub eza: bool,  
    pub jvi: bool, 

    
    pub ekz: u8,     

    
    pub atz: bool,        
    pub fbb: u8,         
    pub bdo: [u8; 64],  
    pub fpk: [u8; 64], 
    pub doj: u8,              
    pub dtv: u8,              
    
    dyw: [u8; 160],    
}

impl Gpu {
    pub fn new() -> Self {
        Self {
            aof: [0; 8192],
            dnb: [0; 8192],
            awh: [0; 160],
            framebuffer: vec![KF_[0]; EQ_ * AHM_],
            amh: 0x91,
            hm: 0x00,
            eyf: 0,
            eye: 0,
            ct: 0,
            eey: 0,
            emt: 0xFC,
            fpm: 0xFF,
            fpn: 0xFF,
            lw: 0,
            fx: 0,
            ev: 2,
            yl: 0,
            hkj: false,
            eza: false,
            jvi: false,
            ekz: 0,
            atz: false,
            fbb: 0,
            bdo: [0xFF; 64],
            fpk: [0xFF; 64],
            doj: 0,
            dtv: 0,
            dyw: [0; 160],
        }
    }

    
    pub fn gu(&mut self, rpm: u32) {
        if self.amh & 0x80 == 0 {
            
            return;
        }

        self.yl += rpm * 4; 

        match self.ev {
            2 => { 
                if self.yl >= 80 {
                    self.yl -= 80;
                    self.ev = 3;
                }
            }
            3 => { 
                if self.yl >= 172 {
                    self.yl -= 172;
                    self.ev = 0;

                    
                    self.lzh();

                    
                    if self.hm & 0x08 != 0 {
                        self.eza = true;
                    }
                }
            }
            0 => { 
                if self.yl >= 204 {
                    self.yl -= 204;
                    self.ct += 1;

                    if self.ct == 144 {
                        
                        self.ev = 1;
                        self.jvi = true;
                        self.hkj = true;
                        self.ekz = 0;

                        
                        if self.hm & 0x10 != 0 {
                            self.eza = true;
                        }
                    } else {
                        self.ev = 2;
                        
                        if self.hm & 0x20 != 0 {
                            self.eza = true;
                        }
                    }

                    self.ncq();
                }
            }
            1 => { 
                if self.yl >= 456 {
                    self.yl -= 456;
                    self.ct += 1;

                    if self.ct >= 154 {
                        self.ct = 0;
                        self.ev = 2;

                        
                        if self.hm & 0x20 != 0 {
                            self.eza = true;
                        }
                    }

                    self.ncq();
                }
            }
            _ => {}
        }
    }

    fn ncq(&mut self) {
        if self.ct == self.eey {
            self.hm |= 0x04; 
            if self.hm & 0x40 != 0 {
                self.eza = true;
            }
        } else {
            self.hm &= !0x04;
        }
    }

    pub fn vso(&self) -> u8 {
        (self.hm & 0xF8) | (if self.ct == self.eey { 0x04 } else { 0 }) | self.ev
    }

    
    fn lzh(&mut self) {
        let ct = self.ct as usize;
        if ct >= AHM_ { return; }

        let l = ct * EQ_;

        
        for b in 0..EQ_ {
            self.framebuffer[l + b] = if self.atz {
                Self::ine(&self.bdo, 0, 0)
            } else {
                KF_[0]
            };
            self.dyw[b] = 0;
        }

        
        if self.atz || self.amh & 0x01 != 0 {
            if self.atz {
                self.vvf(ct, l);
            } else {
                self.vve(ct, l);
            }
        }

        
        if self.amh & 0x20 != 0 && (self.atz || self.amh & 0x01 != 0) {
            if self.atz {
                self.vwz(ct, l);
            } else {
                self.vwy(ct, l);
            }
        }

        
        if self.amh & 0x02 != 0 {
            if self.atz {
                self.vwp(ct, l);
            } else {
                self.vwo(ct, l);
            }
        }
    }

    fn vve(&mut self, ct: usize, l: usize) {
        let dmm = if self.amh & 0x10 != 0 { 0x0000usize } else { 0x0800 };
        let eju = if self.amh & 0x08 != 0 { 0x1C00usize } else { 0x1800 };
        let fur = self.amh & 0x10 == 0;

        let c = (self.eyf as usize + ct) & 0xFF;
        let fws = c / 8;
        let egv = c % 8;

        for b in 0..EQ_ {
            let hxe = (self.eye as usize + b) & 0xFF;
            let fwr = hxe / 8;
            let egt = hxe % 8;

            let djh = fws * 32 + fwr;
            let cup = self.aof[eju + djh];

            let bsn = if fur {
                let fus = cup as i8 as i32;
                (dmm as i32 + (fus + 128) * 16) as usize
            } else {
                dmm + cup as usize * 16
            };

            let chj = bsn + egv * 2;
            if chj + 1 >= self.aof.len() { continue; }

            let hh = self.aof[chj];
            let gd = self.aof[chj + 1];
            let ga = 7 - egt;
            let bts = ((gd >> ga) & 1) << 1 | ((hh >> ga) & 1);

            let clr = (self.emt >> (bts * 2)) & 3;
            self.framebuffer[l + b] = KF_[clr as usize];
        }
    }

    fn vwy(&mut self, ct: usize, l: usize) {
        if ct < self.lw as usize { return; }
        let fx = self.fx as i32 - 7;

        let dmm = if self.amh & 0x10 != 0 { 0x0000usize } else { 0x0800 };
        let eju = if self.amh & 0x40 != 0 { 0x1C00usize } else { 0x1800 };
        let fur = self.amh & 0x10 == 0;

        let aha = self.ekz as usize;
        let fws = aha / 8;
        let egv = aha % 8;

        let mut hxm = false;

        for b in 0..EQ_ {
            let abx = b as i32 - fx;
            if abx < 0 { continue; }
            hxm = true;

            let fwr = abx as usize / 8;
            let egt = abx as usize % 8;
            let djh = fws * 32 + fwr;
            if djh >= 1024 { continue; }

            let cup = self.aof[eju + djh];

            let bsn = if fur {
                let fus = cup as i8 as i32;
                (dmm as i32 + (fus + 128) * 16) as usize
            } else {
                dmm + cup as usize * 16
            };

            let chj = bsn + egv * 2;
            if chj + 1 >= self.aof.len() { continue; }

            let hh = self.aof[chj];
            let gd = self.aof[chj + 1];
            let ga = 7 - egt;
            let bts = ((gd >> ga) & 1) << 1 | ((hh >> ga) & 1);

            let clr = (self.emt >> (bts * 2)) & 3;
            self.framebuffer[l + b] = KF_[clr as usize];
        }

        if hxm {
            self.ekz += 1;
        }
    }

    fn vwo(&mut self, ct: usize, l: usize) {
        let cby = if self.amh & 0x04 != 0 { 16 } else { 8 };

        
        let mut ibh: [(u8, u8, u8, u8, usize); 10] = [(0, 0, 0, 0, 0); 10];
        let mut az = 0usize;

        for a in 0..40 {
            let cq = self.awh[a * 4] as i32 - 16;
            let cr = self.awh[a * 4 + 1] as i32 - 8;
            let ccd = self.awh[a * 4 + 2];
            let flags = self.awh[a * 4 + 3];

            if ct as i32 >= cq && (ct as i32) < cq + cby as i32 {
                if az < 10 {
                    ibh[az] = (
                        self.awh[a * 4],
                        self.awh[a * 4 + 1],
                        ccd,
                        flags,
                        a,
                    );
                    az += 1;
                }
            }
        }

        
        for a in (0..az).vv() {
            let (mik, mij, mut ccd, flags, ybn) = ibh[a];
            let cq = mik as i32 - 16;
            let cr = mij as i32 - 8;
            let fit = flags & 0x20 != 0;
            let fiu = flags & 0x40 != 0;
            let kcr = flags & 0x80 != 0;
            let aim = if flags & 0x10 != 0 { self.fpn } else { self.fpm };

            let mut br = ct as i32 - cq;
            if fiu { br = cby as i32 - 1 - br; }

            if cby == 16 {
                ccd &= 0xFE; 
                if br >= 8 {
                    ccd += 1;
                    br -= 8;
                }
            }

            let bsn = ccd as usize * 16 + br as usize * 2;
            if bsn + 1 >= self.aof.len() { continue; }

            let hh = self.aof[bsn];
            let gd = self.aof[bsn + 1];

            for y in 0..8i32 {
                let xu = cr + y;
                if xu < 0 || xu >= EQ_ as i32 { continue; }

                let ga = if fit { y } else { 7 - y } as u8;
                let bts = ((gd >> ga) & 1) << 1 | ((hh >> ga) & 1);
                if bts == 0 { continue; } 

                let jnv = l + xu as usize;

                
                if kcr {
                    let vp = self.framebuffer[jnv];
                    if vp != KF_[0] { continue; }
                }

                let clr = (aim >> (bts * 2)) & 3;
                self.framebuffer[jnv] = KF_[clr as usize];
            }
        }
    }

    

    fn vvf(&mut self, ct: usize, l: usize) {
        let dmm = if self.amh & 0x10 != 0 { 0x0000usize } else { 0x0800 };
        let eju = if self.amh & 0x08 != 0 { 0x1C00usize } else { 0x1800 };
        let fur = self.amh & 0x10 == 0;

        let c = (self.eyf as usize + ct) & 0xFF;
        let fws = c / 8;
        let egv = c % 8;

        for b in 0..EQ_ {
            let hxe = (self.eye as usize + b) & 0xFF;
            let fwr = hxe / 8;
            let egt = hxe % 8;

            let djh = fws * 32 + fwr;
            let cup = self.aof[eju + djh];
            
            let ddy = self.dnb[eju + djh];
            let hcp = (ddy & 0x07) as usize;
            let idn = (ddy >> 3) & 1;
            let fit = ddy & 0x20 != 0;
            let fiu = ddy & 0x40 != 0;
            let dyw = ddy & 0x80 != 0;

            let bsn = if fur {
                let fus = cup as i8 as i32;
                (dmm as i32 + (fus + 128) * 16) as usize
            } else {
                dmm + cup as usize * 16
            };

            let fcl = if fiu { 7 - egv } else { egv };
            let chj = bsn + fcl * 2;
            let dnc = if idn == 1 { &self.dnb } else { &self.aof };
            if chj + 1 >= dnc.len() { continue; }

            let hh = dnc[chj];
            let gd = dnc[chj + 1];
            let ga = if fit { egt } else { 7 - egt };
            let bts = ((gd >> ga) & 1) << 1 | ((hh >> ga) & 1);

            let s = Self::ine(&self.bdo, hcp, bts as usize);
            self.framebuffer[l + b] = s;
            
            self.dyw[b] = (if bts != 0 { 1 } else { 0 })
                | (if dyw { 2 } else { 0 });
        }
    }

    fn vwz(&mut self, ct: usize, l: usize) {
        if ct < self.lw as usize { return; }
        let fx = self.fx as i32 - 7;

        let dmm = if self.amh & 0x10 != 0 { 0x0000usize } else { 0x0800 };
        let eju = if self.amh & 0x40 != 0 { 0x1C00usize } else { 0x1800 };
        let fur = self.amh & 0x10 == 0;

        let aha = self.ekz as usize;
        let fws = aha / 8;
        let egv = aha % 8;

        let mut hxm = false;

        for b in 0..EQ_ {
            let abx = b as i32 - fx;
            if abx < 0 { continue; }
            hxm = true;

            let fwr = abx as usize / 8;
            let egt = abx as usize % 8;
            let djh = fws * 32 + fwr;
            if djh >= 1024 { continue; }

            let cup = self.aof[eju + djh];
            let ddy = self.dnb[eju + djh];
            let hcp = (ddy & 0x07) as usize;
            let idn = (ddy >> 3) & 1;
            let fit = ddy & 0x20 != 0;
            let fiu = ddy & 0x40 != 0;
            let dyw = ddy & 0x80 != 0;

            let bsn = if fur {
                let fus = cup as i8 as i32;
                (dmm as i32 + (fus + 128) * 16) as usize
            } else {
                dmm + cup as usize * 16
            };

            let fcl = if fiu { 7 - egv } else { egv };
            let chj = bsn + fcl * 2;
            let dnc = if idn == 1 { &self.dnb } else { &self.aof };
            if chj + 1 >= dnc.len() { continue; }

            let hh = dnc[chj];
            let gd = dnc[chj + 1];
            let ga = if fit { egt } else { 7 - egt };
            let bts = ((gd >> ga) & 1) << 1 | ((hh >> ga) & 1);

            let s = Self::ine(&self.bdo, hcp, bts as usize);
            self.framebuffer[l + b] = s;
            self.dyw[b] = (if bts != 0 { 1 } else { 0 })
                | (if dyw { 2 } else { 0 });
        }

        if hxm {
            self.ekz += 1;
        }
    }

    fn vwp(&mut self, ct: usize, l: usize) {
        let cby = if self.amh & 0x04 != 0 { 16 } else { 8 };

        let mut ibh: [(u8, u8, u8, u8, usize); 10] = [(0, 0, 0, 0, 0); 10];
        let mut az = 0usize;

        for a in 0..40 {
            let cq = self.awh[a * 4] as i32 - 16;
            if ct as i32 >= cq && (ct as i32) < cq + cby as i32 {
                if az < 10 {
                    ibh[az] = (
                        self.awh[a * 4],
                        self.awh[a * 4 + 1],
                        self.awh[a * 4 + 2],
                        self.awh[a * 4 + 3],
                        a,
                    );
                    az += 1;
                }
            }
        }

        
        for a in (0..az).vv() {
            let (mik, mij, mut ccd, flags, _) = ibh[a];
            let cq = mik as i32 - 16;
            let cr = mij as i32 - 8;
            let fit = flags & 0x20 != 0;
            let fiu = flags & 0x40 != 0;
            let kcr = flags & 0x80 != 0;
            let hcp = (flags & 0x07) as usize;
            let idn = (flags >> 3) & 1;

            let mut br = ct as i32 - cq;
            if fiu { br = cby as i32 - 1 - br; }

            if cby == 16 {
                ccd &= 0xFE;
                if br >= 8 { ccd += 1; br -= 8; }
            }

            let bsn = ccd as usize * 16 + br as usize * 2;
            let dnc = if idn == 1 { &self.dnb } else { &self.aof };
            if bsn + 1 >= dnc.len() { continue; }

            let hh = dnc[bsn];
            let gd = dnc[bsn + 1];

            for y in 0..8i32 {
                let xu = cr + y;
                if xu < 0 || xu >= EQ_ as i32 { continue; }

                let ga = if fit { y } else { 7 - y } as u8;
                let bts = ((gd >> ga) & 1) << 1 | ((hh >> ga) & 1);
                if bts == 0 { continue; } 

                let pqj = xu as usize;
                let jnv = l + pqj;

                
                
                if self.amh & 0x01 != 0 {
                    let mza = self.dyw[pqj];
                    if (kcr || mza & 2 != 0) && mza & 1 != 0 {
                        continue;
                    }
                }

                let s = Self::ine(&self.fpk, hcp, bts as usize);
                self.framebuffer[jnv] = s;
            }
        }
    }

    
    pub fn jlp(&self, ag: u16) -> u8 {
        let w = (ag & 0x1FFF) as usize;
        if self.fbb == 1 { self.dnb[w] } else { self.aof[w] }
    }
    pub fn mrd(&mut self, ag: u16, ap: u8) {
        let w = (ag & 0x1FFF) as usize;
        if self.fbb == 1 { self.dnb[w] = ap; } else { self.aof[w] = ap; }
    }
    
    pub fn zim(&self, ag: u16, om: u8) -> u8 {
        let w = (ag & 0x1FFF) as usize;
        if om == 1 { self.dnb[w] } else { self.aof[w] }
    }

    
    pub fn vrg(&self) -> u8 {
        let w = (self.doj & 0x3F) as usize;
        self.bdo[w]
    }
    pub fn xvh(&mut self, ap: u8) {
        let w = (self.doj & 0x3F) as usize;
        self.bdo[w] = ap;
        if self.doj & 0x80 != 0 {
            self.doj = 0x80 | ((self.doj + 1) & 0x3F);
        }
    }
    pub fn vse(&self) -> u8 {
        let w = (self.dtv & 0x3F) as usize;
        self.fpk[w]
    }
    pub fn xvo(&mut self, ap: u8) {
        let w = (self.dtv & 0x3F) as usize;
        self.fpk[w] = ap;
        if self.dtv & 0x80 != 0 {
            self.dtv = 0x80 | ((self.dtv + 1) & 0x3F);
        }
    }

    
    fn ine(lrw: &[u8], vbf: usize, rmh: usize) -> u32 {
        let l = vbf * 8 + rmh * 2;
        if l + 1 >= lrw.len() { return 0xFF000000; }
        let hh = lrw[l] as u16;
        let gd = lrw[l + 1] as u16;
        let fsv = hh | (gd << 8);
        let daw = (fsv & 0x1F) as u8;
        let nwx = ((fsv >> 5) & 0x1F) as u8;
        let bjh = ((fsv >> 10) & 0x1F) as u8;
        
        let m = (daw << 3) | (daw >> 2);
        let at = (nwx << 3) | (nwx >> 2);
        let o = (bjh << 3) | (bjh >> 2);
        0xFF000000 | (m as u32) << 16 | (at as u32) << 8 | o as u32
    }

    
    pub fn pai(&self, ag: u16) -> u8 {
        let w = (ag - 0xFE00) as usize;
        if w < 160 { self.awh[w] } else { 0xFF }
    }
    pub fn pzz(&mut self, ag: u16, ap: u8) {
        let w = (ag - 0xFE00) as usize;
        if w < 160 { self.awh[w] = ap; }
    }
}
