




use alloc::vec::Vec;
use alloc::boxed::Box;


#[derive(Debug)]
pub struct ShmPool {
    
    pub ad: u32,
    
    
    pub f: Vec<u8>,
    
    
    pub aw: usize,
    
    
    pub imi: Vec<Amq>,
}

impl ShmPool {
    pub fn new(ad: u32, aw: usize) -> Self {
        Self {
            ad,
            f: alloc::vec![0u8; aw],
            aw,
            imi: Vec::new(),
        }
    }
    
    
    pub fn cmg(&mut self, brm: usize) {
        if brm > self.aw {
            self.f.cmg(brm, 0);
            self.aw = brm;
        }
    }
    
    
    pub fn ngz(
        &mut self,
        gbs: u32,
        l: usize,
        z: u32,
        ac: u32,
        oq: u32,
        format: u32,
    ) -> Result<&Amq, &'static str> {
        
        let kfi = (oq * ac) as usize;
        if l + kfi > self.aw {
            return Err("Buffer extends beyond pool");
        }
        
        let bi = Amq {
            ad: gbs,
            luu: self.ad,
            l,
            z,
            ac,
            oq,
            format,
            imk: false,
        };
        
        self.imi.push(bi);
        Ok(self.imi.qv().unwrap())
    }
    
    
    pub fn ysq(&self, gbs: u32) -> Option<Vec<u32>> {
        let bi = self.imi.iter().du(|o| o.ad == gbs)?;
        
        let mut hz = Vec::fc((bi.z * bi.ac) as usize);
        
        for c in 0..bi.ac {
            for b in 0..bi.z {
                let aok = bi.l + (c * bi.oq + b * 4) as usize;
                if aok + 4 <= self.f.len() {
                    let o = self.f[aok];
                    let at = self.f[aok + 1];
                    let m = self.f[aok + 2];
                    let q = self.f[aok + 3];
                    hz.push(u32::oa([q, m, at, o]));
                } else {
                    hz.push(0xFF000000); 
                }
            }
        }
        
        Some(hz)
    }
    
    
    pub fn write(&mut self, l: usize, f: &[u8]) {
        let ci = (l + f.len()).v(self.aw);
        let len = ci - l;
        self.f[l..ci].dg(&f[..len]);
    }
}


#[derive(Debug, Clone)]
pub struct Amq {
    
    pub ad: u32,
    
    
    pub luu: u32,
    
    
    pub l: usize,
    
    
    pub z: u32,
    
    
    pub ac: u32,
    
    
    pub oq: u32,
    
    
    pub format: u32,
    
    
    pub imk: bool,
}

impl Amq {
    
    pub fn aw(&self) -> usize {
        (self.oq * self.ac) as usize
    }
}


pub struct Bss {
    gpo: Vec<ShmPool>,
    lol: u32,
    loj: u32,
}

impl Bss {
    pub fn new() -> Self {
        Self {
            gpo: Vec::new(),
            lol: 1,
            loj: 1,
        }
    }
    
    pub fn yko(&mut self, aw: usize) -> u32 {
        let ad = self.lol;
        self.lol += 1;
        self.gpo.push(ShmPool::new(ad, aw));
        ad
    }
    
    pub fn ylt(&mut self, ad: u32) {
        self.gpo.ajm(|ai| ai.ad != ad);
    }
    
    pub fn ytm(&self, ad: u32) -> Option<&ShmPool> {
        self.gpo.iter().du(|ai| ai.ad == ad)
    }
    
    pub fn ytn(&mut self, ad: u32) -> Option<&mut ShmPool> {
        self.gpo.el().du(|ai| ai.ad == ad)
    }
    
    pub fn ngz(
        &mut self,
        luu: u32,
        l: usize,
        z: u32,
        ac: u32,
        oq: u32,
        format: u32,
    ) -> Result<u32, &'static str> {
        let gbs = self.loj;
        self.loj += 1;
        
        let lut = self.gpo.el()
            .du(|ai| ai.ad == luu)
            .ok_or("Pool not found")?;
        
        lut.ngz(gbs, l, z, ac, oq, format)?;
        Ok(gbs)
    }
}

impl Default for Bss {
    fn default() -> Self {
        Self::new()
    }
}
