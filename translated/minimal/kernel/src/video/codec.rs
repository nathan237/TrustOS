















use alloc::vec::Vec;
use alloc::vec;


const BHU_: u32 = 0x5476_5264;
const ASK_: u8 = 0;
const BVL_: u8 = 1;


#[repr(C)]
#[derive(Clone, Copy)]
pub struct TvHeader {
    pub sj: u32,
    pub dk: u16,
    pub z: u16,
    pub ac: u16,
    pub tz: u16,
    pub oo: u32,
    pub gkq: u16, 
    pub asi: [u8; 6],
}

impl TvHeader {
    pub fn new(z: u16, ac: u16, tz: u16, oo: u32) -> Self {
        Self {
            sj: BHU_,
            dk: 1,
            z,
            ac,
            tz,
            oo,
            gkq: 30,
            asi: [0; 6],
        }
    }

    pub fn pts(&self) -> Vec<u8> {
        let mut k = Vec::fc(24);
        k.bk(&self.sj.ho());
        k.bk(&self.dk.ho());
        k.bk(&self.z.ho());
        k.bk(&self.ac.ho());
        k.bk(&self.tz.ho());
        k.bk(&self.oo.ho());
        k.bk(&self.gkq.ho());
        k.bk(&self.asi);
        k
    }

    pub fn eca(f: &[u8]) -> Option<Self> {
        if f.len() < 24 { return None; }
        let sj = u32::dj([f[0], f[1], f[2], f[3]]);
        if sj != BHU_ { return None; }
        Some(Self {
            sj,
            dk: u16::dj([f[4], f[5]]),
            z: u16::dj([f[6], f[7]]),
            ac: u16::dj([f[8], f[9]]),
            tz: u16::dj([f[10], f[11]]),
            oo: u32::dj([f[12], f[13], f[14], f[15]]),
            gkq: u16::dj([f[16], f[17]]),
            asi: [f[18], f[19], f[20], f[21], f[22], f[23]],
        })
    }
}




fn pds(hz: &[u32]) -> Vec<u8> {
    let mut bd = Vec::new();
    if hz.is_empty() { return bd; }
    let mut a = 0;
    while a < hz.len() {
        let ap = hz[a];
        let mut vw: u8 = 0; 
        while a + (vw as usize) + 1 < hz.len()
            && hz[a + (vw as usize) + 1] == ap
            && vw < 255
        {
            vw += 1;
        }
        bd.push(vw);
        bd.bk(&ap.ho());
        a += vw as usize + 1;
    }
    bd
}


fn pdr(f: &[u8], fqv: usize) -> Vec<u32> {
    let mut hz = Vec::fc(fqv);
    let mut a = 0;
    while a + 4 < f.len() && hz.len() < fqv {
        let vw = f[a] as usize + 1;
        let ap = u32::dj([f[a + 1], f[a + 2], f[a + 3], f[a + 4]]);
        for _ in 0..vw {
            if hz.len() >= fqv { break; }
            hz.push(ap);
        }
        a += 5;
    }
    hz
}



pub struct TvEncoder {
    pub dh: TvHeader,
    jjv: Vec<u32>,
    hkl: u32,
    pub f: Vec<u8>,
}

impl TvEncoder {
    pub fn new(z: u16, ac: u16, tz: u16) -> Self {
        let awg = z as usize * ac as usize;
        Self {
            dh: TvHeader::new(z, ac, tz, 0),
            jjv: vec![0u32; awg],
            hkl: 0,
            f: Vec::new(),
        }
    }

    
    pub fn jzf(&mut self, hz: &[u32]) {
        let awg = self.dh.z as usize * self.dh.ac as usize;
        let txw = self.hkl == 0
            || (self.dh.gkq > 0
                && self.hkl % self.dh.gkq as u32 == 0);

        if txw {
            
            let ahf = pds(&hz[..awg]);
            let bzt = 1 + ahf.len(); 
            self.f.bk(&(bzt as u32).ho());
            self.f.push(ASK_);
            self.f.bk(&ahf);
            self.jjv[..awg].dg(&hz[..awg]);
        } else {
            
            let mut aaq = vec![0u32; awg];
            for a in 0..awg {
                aaq[a] = hz[a] ^ self.jjv[a];
            }
            let ahf = pds(&aaq);
            let bzt = 1 + ahf.len();
            self.f.bk(&(bzt as u32).ho());
            self.f.push(BVL_);
            self.f.bk(&ahf);
            self.jjv[..awg].dg(&hz[..awg]);
        }
        self.hkl += 1;
        self.dh.oo = self.hkl;
    }

    
    pub fn bqs(&self) -> Vec<u8> {
        let mut bd = self.dh.pts();
        bd.bk(&self.f);
        bd
    }
}



pub struct TvDecoder {
    pub dh: TvHeader,
    f: Vec<u8>,
    l: usize,
    het: Vec<u32>,
    pub hkk: u32,
}

impl TvDecoder {
    pub fn new(cxw: Vec<u8>) -> Option<Self> {
        let dh = TvHeader::eca(&cxw)?;
        let awg = dh.z as usize * dh.ac as usize;
        Some(Self {
            dh,
            f: cxw,
            l: 24, 
            het: vec![0u32; awg],
            hkk: 0,
        })
    }

    
    pub fn uue(&mut self) -> Option<&[u32]> {
        if self.hkk >= self.dh.oo { return None; }
        if self.l + 5 > self.f.len() { return None; }

        let bzt = u32::dj([
            self.f[self.l],
            self.f[self.l + 1],
            self.f[self.l + 2],
            self.f[self.l + 3],
        ]) as usize;
        self.l += 4;

        if self.l + bzt > self.f.len() { return None; }

        let swz = self.f[self.l];
        let pdq = &self.f[self.l + 1..self.l + bzt];
        let awg = self.dh.z as usize * self.dh.ac as usize;

        if swz == ASK_ {
            let hz = pdr(pdq, awg);
            self.het[..awg].dg(&hz[..awg.v(hz.len())]);
        } else {
            
            let aaq = pdr(pdq, awg);
            for a in 0..awg.v(aaq.len()) {
                self.het[a] ^= aaq[a];
            }
        }

        self.l += bzt;
        self.hkk += 1;
        Some(&self.het)
    }

    
    pub fn lzz(&mut self) {
        self.l = 24;
        self.hkk = 0;
        self.het.vi(0);
    }

    
    pub fn ywj(&self) -> bool {
        self.hkk < self.dh.oo && self.l + 5 <= self.f.len()
    }
}
