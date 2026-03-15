










use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use super::tables::{XC_, AFO_};


pub const BR_: u32 = 48000;

pub const Dv: u32 = 2;

pub const DEH_: u32 = 2;

pub const BAE_: usize = 8;

const EH_: u32 = 16;

const JC_: u32 = 256;

const CEM_: u16 = 0xACE1;






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Dg,
    Gb,
    Ft,
    Triangle,
    Cr,
}

impl Waveform {
    
    pub fn cko(e: &str) -> Option<Self> {
        match e {
            "sine" | "sin" | "s" => Some(Waveform::Dg),
            "square" | "sq" | "q" => Some(Waveform::Gb),
            "saw" | "sawtooth" | "w" => Some(Waveform::Ft),
            "triangle" | "tri" | "t" => Some(Waveform::Triangle),
            "noise" | "n" => Some(Waveform::Cr),
            _ => None,
        }
    }

    
    pub fn dbz(&self) -> &'static str {
        match self {
            Waveform::Dg => "Sin",
            Waveform::Gb => "Sqr",
            Waveform::Ft => "Saw",
            Waveform::Triangle => "Tri",
            Waveform::Cr => "Noi",
        }
    }

    pub fn j(&self) -> &'static str {
        match self {
            Waveform::Dg => "Sine",
            Waveform::Gb => "Square",
            Waveform::Ft => "Sawtooth",
            Waveform::Triangle => "Triangle",
            Waveform::Cr => "Noise",
        }
    }
}






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvState {
    Cv,
    Ape,
    Ahi,
    Ane,
    Release,
}


#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    
    pub gzd: u32,
    
    pub hfn: u32,
    
    pub fvx: i32,
    
    pub chd: u32,
    
    g: EnvState,
    
    jy: i32,
    
    va: u32,
}

impl Envelope {
    
    pub fn new(gzc: u32, hfm: u32, icg: u32, hxk: u32) -> Self {
        let wwd = (icg.v(100) as i32 * 32767) / 100;
        Self {
            gzd: hsd(gzc),
            hfn: hsd(hfm),
            fvx: wwd,
            chd: hsd(hxk),
            g: EnvState::Cv,
            jy: 0,
            va: 0,
        }
    }

    
    pub fn iqt() -> Self {
        Self::new(10, 50, 70, 100)
    }

    
    pub fn uza() -> Self {
        Self::new(1, 1, 100, 10)
    }

    
    pub fn hvi() -> Self {
        Self::new(2, 200, 0, 50)
    }

    
    pub fn ov() -> Self {
        Self::new(300, 100, 80, 500)
    }

    
    pub fn dtq(&mut self) {
        self.g = EnvState::Ape;
        self.va = 0;
        
    }

    
    pub fn djx(&mut self) {
        if self.g != EnvState::Cv {
            self.g = EnvState::Release;
            self.va = 0;
        }
    }

    
    pub fn or(&mut self) -> i32 {
        match self.g {
            EnvState::Cv => {
                self.jy = 0;
            }
            EnvState::Ape => {
                if self.gzd == 0 {
                    self.jy = 32767;
                    self.g = EnvState::Ahi;
                    self.va = 0;
                } else {
                    self.jy = ((self.va as i64 * 32767) / self.gzd as i64) as i32;
                    self.va += 1;
                    if self.va >= self.gzd {
                        self.jy = 32767;
                        self.g = EnvState::Ahi;
                        self.va = 0;
                    }
                }
            }
            EnvState::Ahi => {
                if self.hfn == 0 {
                    self.jy = self.fvx;
                    self.g = EnvState::Ane;
                } else {
                    let aaq = 32767 - self.fvx;
                    self.jy = 32767 - ((self.va as i64 * aaq as i64) / self.hfn as i64) as i32;
                    self.va += 1;
                    if self.va >= self.hfn {
                        self.jy = self.fvx;
                        self.g = EnvState::Ane;
                        self.va = 0;
                    }
                }
            }
            EnvState::Ane => {
                self.jy = self.fvx;
                
            }
            EnvState::Release => {
                if self.chd == 0 {
                    self.jy = 0;
                    self.g = EnvState::Cv;
                } else {
                    let poa = if self.va == 0 { self.jy } else {
                        
                        
                        self.fvx
                    };
                    self.jy = poa - ((self.va as i64 * poa as i64) / self.chd as i64) as i32;
                    if self.jy < 0 { self.jy = 0; }
                    self.va += 1;
                    if self.va >= self.chd {
                        self.jy = 0;
                        self.g = EnvState::Cv;
                    }
                }
            }
        }
        self.jy
    }

    
    pub fn edw(&self) -> bool {
        self.g == EnvState::Cv
    }

    
    pub fn mhm(&self) -> &'static str {
        match self.g {
            EnvState::Cv => "Idle",
            EnvState::Ape => "Atk",
            EnvState::Ahi => "Dec",
            EnvState::Ane => "Sus",
            EnvState::Release => "Rel",
        }
    }
}






#[derive(Debug, Clone)]
pub struct Oscillator {
    
    pub ve: Waveform,
    
    ib: u32,
    
    fqq: u32,
    
    pub auf: u32,
    
    cam: u16,
}

impl Oscillator {
    
    pub fn new(ve: Waveform, auf: u32) -> Self {
        let fqq = Self::nbg(auf);
        Self {
            ve,
            ib: 0,
            fqq,
            auf,
            cam: CEM_,
        }
    }

    
    
    fn nbg(auf: u32) -> u32 {
        
        ((auf as u64 * (JC_ as u64) << EH_) / BR_ as u64) as u32
    }

    
    pub fn wiw(&mut self, auf: u32) {
        self.auf = auf;
        self.fqq = Self::nbg(auf);
    }

    
    pub fn znk(&mut self, jp: u8) {
        let kx = AFO_[jp.v(127) as usize];
        self.wiw(kx);
    }

    
    pub fn zjx(&mut self) {
        self.ib = 0;
    }

    
    pub fn or(&mut self) -> i16 {
        let yr = match self.ve {
            Waveform::Dg => self.tbj(),
            Waveform::Gb => self.tbl(),
            Waveform::Ft => self.tbi(),
            Waveform::Triangle => self.tbs(),
            Waveform::Cr => self.tbg(),
        };

        
        self.ib = self.ib.cn(self.fqq);

        yr
    }

    
    fn tbj(&self) -> i16 {
        let prn = (self.ib >> EH_) as usize & 0xFF;
        let avw = (self.ib & 0xFFFF) as i32;

        let cmq = XC_[prn] as i32;
        let bic = XC_[(prn + 1) & 0xFF] as i32;

        
        let ahp = cmq + ((bic - cmq) * avw >> 16);
        ahp as i16
    }

    
    
    
    
    fn lus(&self, ai: u32) -> i32 {
        let drz = self.fqq;
        if drz == 0 { return 0; }
        let awn = (JC_ << EH_) as u32;

        
        if ai < drz {
            let ab = ((ai as u64) << 16) / drz as u64;
            let ab = ab as i32;
            
            return 2 * ab - ((ab as i64 * ab as i64) >> 16) as i32 - 65536;
        }

        
        if ai > awn.ao(drz) {
            let ab = (((ai as i64) - awn as i64) << 16) / drz as i64;
            let ab = ab as i32;
            
            return ((ab as i64 * ab as i64) >> 16) as i32 + 2 * ab + 65536;
        }

        0
    }

    
    fn tbl(&self) -> i16 {
        let gpf = ((JC_ << EH_) - 1) as u32;
        let iv = (128u32) << EH_;
        let ai = self.ib & gpf;

        let lnf: i32 = if ai < iv { 24000 } else { -24000 };

        
        let qqc = self.lus(ai);
        let qqb = self.lus(ai.nj(iv) & gpf);

        let yr = lnf
            + ((qqc as i64 * 24000) >> 16) as i32
            - ((qqb as i64 * 24000) >> 16) as i32;

        yr.qp(-32767, 32767) as i16
    }

    
    fn tbi(&self) -> i16 {
        let gpf = ((JC_ << EH_) - 1) as u32;
        let awn = (JC_ << EH_) as u64;
        let ai = self.ib & gpf;

        
        let lnf = ((ai as i64 * 48000) / awn as i64 - 24000) as i32;

        
        let qqa = self.lus(ai);
        let rpf = ((qqa as i64 * 24000) >> 16) as i32;

        (lnf - rpf).qp(-32767, 32767) as i16
    }

    
    fn tbs(&self) -> i16 {
        let gpf = ((JC_ << EH_) - 1) as u32;
        let ai = self.ib & gpf;
        let iv = (128u32) << EH_;

        if ai < iv {
            
            ((ai as i64 * 48000 / iv as i64) - 24000) as i16
        } else {
            
            let vv = ((JC_ << EH_) as u32).nj(ai);
            ((vv as i64 * 48000 / iv as i64) - 24000) as i16
        }
    }

    
    fn tbg(&mut self) -> i16 {
        
        let ga = self.cam & 1;
        self.cam >>= 1;
        if ga == 1 {
            self.cam ^= 0xB400; 
        }
        
        (self.cam as i16).hx(3) / 4 
    }
}







#[derive(Debug, Clone, Copy)]
struct LowPassFilter {
    dp: i32,    
    jz: i32,    
    dw: u32, 
}

impl LowPassFilter {
    
    fn nav() -> Self {
        Self { dp: 0, jz: 0, dw: 65536 }
    }

    
    fn mek(&mut self, rsl: u32) {
        
        let d = (6283u64 * rsl as u64) / BR_ as u64;
        
        self.dw = ((d << 16) / (1000 + d)).v(65536) as u32;
    }

    
    fn process(&mut self, input: i32) -> i32 {
        let q = self.dw as i64;
        
        self.dp += (((input - self.dp) as i64 * q) >> 16) as i32;
        
        self.jz += (((self.dp - self.jz) as i64 * q) >> 16) as i32;
        self.jz
    }

    fn apa(&mut self) {
        self.dp = 0;
        self.jz = 0;
    }
}






#[derive(Debug, Clone)]
pub struct Voice {
    pub fpw: Oscillator,
    
    lqx: Oscillator,
    pub env: Envelope,
    
    hi: LowPassFilter,
    
    pub jp: u8,
    
    pub qm: u8,
    
    pub gh: bool,
    
    hha: u32,
    
    ikv: u32,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            fpw: Oscillator::new(Waveform::Dg, 440),
            lqx: Oscillator::new(Waveform::Dg, 440),
            env: Envelope::iqt(),
            hi: LowPassFilter::nav(),
            jp: 0,
            qm: 0,
            gh: false,
            hha: 0,
            ikv: 0,
        }
    }

    
    pub fn dtq(&mut self, jp: u8, qm: u8, ve: Waveform, qr: Envelope) {
        let kx = AFO_[jp.v(127) as usize];

        
        self.fpw = Oscillator::new(ve, kx);
        self.ikv = self.fpw.fqq;
        self.hha = 0;

        
        let sxg = kx + (kx / 200).am(1);
        self.lqx = Oscillator::new(ve, sxg);

        self.env = qr;
        self.env.dtq();
        self.jp = jp;
        self.qm = qm;
        self.gh = true;

        
        self.hi.apa();
        match ve {
            Waveform::Dg => {
                
                self.hi = LowPassFilter::nav();
            }
            Waveform::Triangle => {
                
                let knh = (kx * 12).am(400).v(16000);
                self.hi.mek(knh);
            }
            Waveform::Gb | Waveform::Ft => {
                
                let knh = (kx * 8).am(300).v(12000);
                self.hi.mek(knh);
            }
            Waveform::Cr => {
                
                self.hi.mek(6000);
            }
        }
    }

    
    pub fn djx(&mut self) {
        self.env.djx();
    }

    
    pub fn or(&mut self) -> i16 {
        if !self.gh {
            return 0;
        }

        let smm = self.env.or();
        if self.env.edw() {
            self.gh = false;
            return 0;
        }

        
        self.hha = self.hha.cn(19);  
        let sgs = (self.hha >> 8) as usize & 0xFF;
        let sgu = XC_[sgs] as i32;  
        
        let sgt = ((self.ikv as i64 * sgu as i64) / (32767 * 1250)) as i32;
        self.fpw.fqq = (self.ikv as i32 + sgt).am(1) as u32;

        
        let vqg = self.fpw.or() as i32;
        let vqh = self.lqx.or() as i32;
        let js = (vqg + vqh) / 2;

        
        let aud = self.hi.process(js);

        
        let wct = if aud > 18000 {
            18000 + (aud - 18000) / 4
        } else if aud < -18000 {
            -18000 + (aud + 18000) / 4
        } else {
            aud
        };

        let xqz = self.qm as i32;

        
        let yr = (wct * smm / 32767) * xqz / 127;
        yr.qp(-32767, 32767) as i16
    }
}






pub struct SynthEngine {
    
    pub ddh: [Voice; BAE_],
    
    pub ve: Waveform,
    
    pub qr: Envelope,
    
    pub euo: u8,
}

impl SynthEngine {
    
    pub fn new() -> Self {
        let ddh = core::array::nwe(|_| Voice::new());
        Self {
            ddh,
            ve: Waveform::Dg,
            qr: Envelope::iqt(),
            euo: 200,
        }
    }

    
    pub fn dvs(&mut self, azd: Waveform) {
        self.ve = azd;
    }

    
    pub fn med(&mut self, gzc: u32, hfm: u32, icg: u32, hxk: u32) {
        self.qr = Envelope::new(gzc, hfm, icg, hxk);
    }

    
    pub fn dtq(&mut self, jp: u8, qm: u8) {
        
        let jvz = self.nuh();
        let bxt = &mut self.ddh[jvz];
        bxt.dtq(jp, qm, self.ve, self.qr);
    }

    
    pub fn djx(&mut self, jp: u8) {
        for bxt in &mut self.ddh {
            if bxt.gh && bxt.jp == jp {
                bxt.djx();
                break;
            }
        }
    }

    
    pub fn qgm(&mut self) {
        for bxt in &mut self.ddh {
            bxt.djx();
        }
    }

    
    
    pub fn tj(&mut self, bi: &mut [i16], evo: usize) -> usize {
        let ptx = evo.v(bi.len() / 2); 

        for a in 0..ptx {
            
            let mut bno: i32 = 0;
            for bxt in &mut self.ddh {
                if bxt.gh {
                    bno += bxt.or() as i32;
                }
            }

            
            bno = bno * self.euo as i32 / 255;

            
            let yr = bno.qp(-32767, 32767) as i16;

            
            bi[a * 2] = yr;
            bi[a * 2 + 1] = yr;
        }

        ptx
    }

    
    pub fn lzf(&mut self, jp: u8, qm: u8, uk: u32) -> Vec<i16> {
        let ayz = hsd(uk) as usize;
        
        let chd = self.qr.chd as usize;
        let kxn = ayz + chd;
        let mut bi = alloc::vec![0i16; kxn * 2]; 

        
        self.dtq(jp, qm);

        
        self.tj(&mut bi[..ayz * 2], ayz);

        
        self.djx(jp);

        
        if chd > 0 {
            self.tj(&mut bi[ayz * 2..], chd);
        }

        bi
    }

    
    pub fn viz(&mut self, j: &str, uk: u32) -> Result<Vec<i16>, &'static str> {
        let ti = super::tables::fpd(j)
            .ok_or("Invalid note name (use e.g. C4, A#3, Bb5)")?;
        Ok(self.lzf(ti, 100, uk))
    }

    
    pub fn vvv(&mut self, auf: u32, uk: u32) -> Vec<i16> {
        let ayz = hsd(uk) as usize;
        let chd = self.qr.chd as usize;
        let kxn = ayz + chd;
        let mut bi = alloc::vec![0i16; kxn * 2]; 

        
        let jvz = self.nuh();
        let bxt = &mut self.ddh[jvz];
        bxt.fpw = Oscillator::new(self.ve, auf);
        bxt.env = self.qr;
        bxt.env.dtq();
        bxt.jp = 69; 
        bxt.qm = 100;
        bxt.gh = true;

        
        self.tj(&mut bi[..ayz * 2], ayz);
        
        self.ddh[jvz].djx();
        if chd > 0 {
            self.tj(&mut bi[ayz * 2..], chd);
        }

        bi
    }

    
    pub fn qfb(&self) -> usize {
        self.ddh.iter().hi(|p| p.gh).az()
    }

    
    pub fn status(&self) -> String {
        let mut e = String::new();
        e.t(&format!("TrustSynth Engine\n"));
        e.t(&format!("  Waveform: {}\n", self.ve.j()));
        e.t(&format!("  ADSR: A={}ms D={}ms S={}% R={}ms\n",
            mbp(self.qr.gzd),
            mbp(self.qr.hfn),
            self.qr.fvx * 100 / 32767,
            mbp(self.qr.chd)
        ));
        e.t(&format!("  Master Volume: {}/255\n", self.euo));
        e.t(&format!("  Active Voices: {}/{}\n", self.qfb(), BAE_));
        for (a, p) in self.ddh.iter().cf() {
            if p.gh {
                let bkp = super::tables::dtf(p.jp);
                let cgg = super::tables::efk(p.jp);
                e.t(&format!("    Voice {}: {}{} vel={} env={} wf={}\n",
                    a, bkp, cgg, p.qm, p.env.mhm(), p.fpw.ve.dbz()));
            }
        }
        e
    }

    

    
    fn nuh(&self) -> usize {
        
        for (a, p) in self.ddh.iter().cf() {
            if !p.gh {
                return a;
            }
        }
        
        let mut bdn = 0;
        let mut kct = i32::O;
        for (a, p) in self.ddh.iter().cf() {
            if p.env.g == EnvState::Release && (p.env.jy as i32) < kct {
                kct = p.env.jy as i32;
                bdn = a;
            }
        }
        if kct < i32::O {
            return bdn;
        }
        
        0
    }
}






pub fn hsd(jn: u32) -> u32 {
    (BR_ * jn) / 1000
}


pub fn mbp(un: u32) -> u32 {
    (un * 1000) / BR_
}
