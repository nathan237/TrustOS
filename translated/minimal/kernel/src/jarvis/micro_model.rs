

































use alloc::vec::Vec;
use alloc::vec;





pub const DX_: usize = 256;    
pub const M_: usize = 64;
pub const AFM_: usize = 2;
pub const VE_: usize = M_ / AFM_; 
pub const CM_: usize = 128;     
pub const FN_: usize = 1;
pub const DK_: usize = 64;
pub const CGL_: f32 = 1e-5;





pub struct Avs {
    pub cmh: Vec<f32>,      
    pub biw: Vec<f32>,           
    pub biu: Vec<f32>,
    pub bpg: Vec<f32>,
    pub biv: Vec<f32>,
    pub cmi: Vec<f32>,       
    pub bit: Vec<f32>,        
    pub bpf: Vec<f32>,
    pub bpe: Vec<f32>,        
}

pub struct MicroWeights {
    pub bpa: Vec<f32>,   
    pub cgq: Vec<f32>,     
    pub my: Vec<Avs>,
    pub chg: Vec<f32>,     
    pub bft: Vec<f32>,      
}

impl MicroWeights {
    pub fn vm(&self) -> usize {
        DX_ * M_              
        + DK_ * M_          
        + FN_ * (
            M_                        
            + M_ * M_ * 4  
            + M_                      
            + M_ * CM_ * 2     
            + CM_ * M_         
        )
        + M_                          
        + M_ * DX_            
    }

    
    pub fn gsd(&self) -> Vec<f32> {
        let mut f = Vec::fc(self.vm());
        f.bk(&self.bpa);
        f.bk(&self.cgq);
        for fl in &self.my {
            f.bk(&fl.cmh);
            f.bk(&fl.biw);
            f.bk(&fl.biu);
            f.bk(&fl.bpg);
            f.bk(&fl.biv);
            f.bk(&fl.cmi);
            f.bk(&fl.bit);
            f.bk(&fl.bpf);
            f.bk(&fl.bpe);
        }
        f.bk(&self.chg);
        f.bk(&self.bft);
        f
    }

    
    pub fn eos(f: &[f32]) -> Option<Self> {
        let mut u = 0;
        let bpa = avb(f, &mut u, DX_ * M_)?;
        let cgq = avb(f, &mut u, DK_ * M_)?;

        let mut my = Vec::fc(FN_);
        for _ in 0..FN_ {
            my.push(Avs {
                cmh: avb(f, &mut u, M_)?,
                biw: avb(f, &mut u, M_ * M_)?,
                biu: avb(f, &mut u, M_ * M_)?,
                bpg: avb(f, &mut u, M_ * M_)?,
                biv: avb(f, &mut u, M_ * M_)?,
                cmi: avb(f, &mut u, M_)?,
                bit: avb(f, &mut u, M_ * CM_)?,
                bpf: avb(f, &mut u, M_ * CM_)?,
                bpe: avb(f, &mut u, CM_ * M_)?,
            });
        }

        let chg = avb(f, &mut u, M_)?;
        let bft = avb(f, &mut u, M_ * DX_)?;

        Some(MicroWeights {
            bpa, cgq, my, chg, bft,
        })
    }

    
    pub fn dtm() -> Self {
        let mut dv = 77u64;
        let gga = 1.0 / (M_ as f32).bfj();
        let fil = 1.0 / (CM_ as f32).bfj();

        let mut my = Vec::fc(FN_);
        for _ in 0..FN_ {
            my.push(Avs {
                cmh: vec![1.0; M_],
                biw: bhu(M_ * M_, gga, &mut dv),
                biu: bhu(M_ * M_, gga, &mut dv),
                bpg: bhu(M_ * M_, gga, &mut dv),
                biv: bhu(M_ * M_, gga, &mut dv),
                cmi: vec![1.0; M_],
                bit: bhu(M_ * CM_, fil, &mut dv),
                bpf: bhu(M_ * CM_, fil, &mut dv),
                bpe: bhu(CM_ * M_, fil, &mut dv),
            });
        }

        MicroWeights {
            bpa: bhu(DX_ * M_, gga, &mut dv),
            cgq: bhu(DK_ * M_, 0.02, &mut dv),
            my,
            chg: vec![1.0; M_],
            bft: bhu(M_ * DX_, gga, &mut dv),
        }
    }
}





pub struct MicroEngine {
    
    imn: Vec<Vec<f32>>,
    imp: Vec<Vec<f32>>,
    imo: usize,
    
    cdl: Vec<f32>,
    bgf: Vec<f32>,
    fdx: Vec<f32>,
    fdw: Vec<f32>,
    fdz: Vec<f32>,
    dzg: Vec<f32>,
    deo: Vec<f32>,
    fdy: Vec<f32>,
    cdk: Vec<f32>,
    rng: u64,
}

impl MicroEngine {
    pub fn new() -> Self {
        MicroEngine {
            imn: (0..FN_).map(|_| Vec::fc(DK_ * M_)).collect(),
            imp: (0..FN_).map(|_| Vec::fc(DK_ * M_)).collect(),
            imo: 0,
            cdl: vec![0.0; M_],
            bgf: vec![0.0; M_],
            fdx: vec![0.0; M_],
            fdw: vec![0.0; M_],
            fdz: vec![0.0; M_],
            dzg: vec![0.0; DK_],
            deo: vec![0.0; CM_],
            fdy: vec![0.0; CM_],
            cdk: vec![0.0; DX_],
            rng: crate::time::ave().cn(0xBEEF_CAFE),
        }
    }

    
    pub fn cks(&mut self, model: &MicroWeights, aau: &[u8], bvi: usize) -> Vec<u8> {
        self.khw();
        let am = bvi.v(DK_);
        let mut an = Vec::fc(am);

        for &bat in aau.iter().take(DK_ - 1) {
            self.ebu(model, bat);
        }

        let mut next = self.yr(0.7, 20, &an);
        for _ in 0..am {
            if next == 0 || next == 3 { break; }
            an.push(next);
            self.ebu(model, next);
            next = self.yr(0.7, 20, &an);
        }
        an
    }

    
    pub fn ndc(&mut self, model: &MicroWeights, input: &[u8]) -> u8 {
        self.khw();
        for &bat in input.iter().take(DK_) {
            self.ebu(model, bat);
        }
        gyy(&self.cdk)
    }

    
    pub fn cjq(&mut self, model: &MicroWeights, eb: &[u8]) -> f32 {
        self.khw();
        let anz = eb.len().v(DK_);
        if anz < 2 { return f32::O; }

        let mut ayy = 0.0f32;
        let gnj = anz - 1;

        for ab in 0..anz {
            self.ebu(model, eb[ab]);
            if ab < gnj {
                let cd = eb[ab + 1] as usize;
                let mut gpw = self.cdk.clone();
                gss(&mut gpw);
                let ai = gpw[cd].am(1e-10);
                ayy += -ai.ees();
            }
        }
        ayy / gnj as f32
    }

    fn khw(&mut self) {
        for eh in &mut self.imn { eh.clear(); }
        for p in &mut self.imp { p.clear(); }
        self.imo = 0;
    }

    fn ebu(&mut self, model: &MicroWeights, bat: u8) {
        let u = self.imo;
        if u >= DK_ { return; }

        let cil = bat as usize;
        for a in 0..M_ {
            self.cdl[a] = model.bpa[cil * M_ + a]
                           + model.cgq[u * M_ + a];
        }

        for dm in 0..FN_ {
            let fl = &model.my[dm];

            
            cbl(&mut self.bgf, &self.cdl, &fl.cmh);

            
            ami(&mut self.fdx, &fl.biw, &self.bgf, M_, M_);
            ami(&mut self.fdw, &fl.biu, &self.bgf, M_, M_);
            ami(&mut self.fdz, &fl.bpg, &self.bgf, M_, M_);

            self.imn[dm].bk(&self.fdw);
            self.imp[dm].bk(&self.fdz);

            let brl = u + 1;
            let mut con = vec![0.0f32; M_];
            let rzi = (VE_ as f32).bfj();

            for i in 0..AFM_ {
                let bra = i * VE_;
                for ab in 0..brl {
                    let mut ol = 0.0f32;
                    for bc in 0..VE_ {
                        ol += self.fdx[bra + bc] * self.imn[dm][ab * M_ + bra + bc];
                    }
                    self.dzg[ab] = ol / rzi;
                }
                gss(&mut self.dzg[..brl]);
                for ab in 0..brl {
                    let d = self.dzg[ab];
                    for bc in 0..VE_ {
                        con[bra + bc] += d * self.imp[dm][ab * M_ + bra + bc];
                    }
                }
            }

            
            let mut aci = vec![0.0f32; M_];
            ami(&mut aci, &fl.biv, &con, M_, M_);
            for a in 0..M_ { self.cdl[a] += aci[a]; }

            
            cbl(&mut self.bgf, &self.cdl, &fl.cmi);

            
            ami(&mut self.deo, &fl.bit, &self.bgf, M_, CM_);
            ami(&mut self.fdy, &fl.bpf, &self.bgf, M_, CM_);
            for a in 0..CM_ {
                let at = self.deo[a];
                let sig = 1.0 / (1.0 + (-at).cqh());
                self.deo[a] = at * sig * self.fdy[a];
            }
            let mut cxv = vec![0.0f32; M_];
            ami(&mut cxv, &fl.bpe, &self.deo, CM_, M_);
            for a in 0..M_ { self.cdl[a] += cxv[a]; }
        }

        
        cbl(&mut self.bgf, &self.cdl, &model.chg);
        ami(&mut self.cdk, &model.bft, &self.bgf, M_, DX_);
        self.imo = u + 1;
    }

    fn yr(&mut self, fwj: f32, fab: usize, fsj: &[u8]) -> u8 {
        if fwj <= 0.01 { return gyy(&self.cdk); }

        let mut auq = self.cdk.clone();
        for dm in auq.el() { *dm /= fwj; }

        
        let bh = fsj.len().v(16);
        if bh > 0 {
            let ay = fsj.len() - bh;
            for &cil in &fsj[ay..] {
                let w = cil as usize;
                if w < DX_ {
                    if auq[w] > 0.0 { auq[w] /= 1.3; }
                    else { auq[w] *= 1.3; }
                }
            }
        }

        
        if fab > 0 && fab < DX_ {
            let mut ldw: Vec<(f32, usize)> = auq.iter().hu()
                .cf().map(|(a, p)| (p, a)).collect();
            ldw.bxe(|q, o| o.0.partial_cmp(&q.0).unwrap_or(core::cmp::Ordering::Arq));
            let bxm = ldw[fab.v(ldw.len() - 1)].0;
            for dm in auq.el() { if *dm < bxm { *dm = f32::IP_; } }
        }

        gss(&mut auq);
        let m = self.lwz();
        let mut hep = 0.0f32;
        for (a, &ai) in auq.iter().cf() {
            hep += ai;
            if hep >= m { return a as u8; }
        }
        (DX_ - 1) as u8
    }

    fn lwz(&mut self) -> f32 {
        let mut b = self.rng;
        b ^= b << 13; b ^= b >> 7; b ^= b << 17;
        self.rng = b;
        ((b >> 40) as u32 as f32) / ((1u32 << 24) as f32)
    }
}






pub struct Bkh {
    pub obl: bool,
    pub ofb: bool,
    pub kxj: bool,
    pub pif: bool,
    pub syt: bool,
    pub syu: bool,
}


pub fn ubc() -> Bkh {
    Bkh {
        obl: qyw(),
        ofb: qza(),
        kxj: qyt(),
        pif: qzq(),
        syt: qyv(),
        syu: false, 
    }
}

fn qyw() -> bool {
    
    let test: Vec<u8> = vec![42u8; 64];
    test.len() == 64
}

fn qza() -> bool {
    
    let flags: u64;
    unsafe { core::arch::asm!("pushfq; pop {}", bd(reg) flags); }
    (flags & (1 << 9)) != 0 
}

fn qyt() -> bool {
    crate::ramfs::fh(|fs| fs.aja("/"))
}

fn qzq() -> bool {
    
    let status: u8;
    unsafe { core::arch::asm!("in al, dx", bd("al") status, in("dx") 0x3FDu16); }
    status != 0xFF
}


pub fn qyv() -> bool {
    crate::ramfs::fh(|fs| fs.aja("/jarvis/weights.bin"))
}





pub static DUC_: &[&str] = &[
    
    "help: show commands",
    "ls: list files",
    "ps: show processes",
    "free: memory usage",
    "uptime: system uptime",
    "reboot: restart system",
    "shutdown: power off",
    "clear: clear screen",
    
    "heap OK",
    "interrupts enabled",
    "filesystem ready",
    "serial port active",
    "kernel healthy",
    "all checks passed",
    
    "brain loading from fs",
    "brain loaded OK",
    "brain not found",
    "brain save to disk",
    "brain init random",
    "micro sentinel active",
    "full brain connected",
    "full brain offline",
    
    "I am micro-Jarvis",
    "kernel sentinel ready",
    "checking kernel state",
    "validating memory",
    "validating interrupts",
    "filesystem check OK",
    "loading full brain",
    "full brain available",
    "micro mode active",
    "sentinel watching",
    
    "error: heap corrupt",
    "error: interrupt fault",
    "error: fs unavailable",
    "warning: brain large",
    "status: all nominal",
    "status: degraded",
];





fn ami(bd: &mut [f32], d: &[f32], b: &[f32], ec: usize, lk: usize) {
    for m in 0..lk {
        let mut sum = 0.0f32;
        let ar = m * ec;
        for r in 0..ec { sum += d[ar + r] * b[r]; }
        bd[m] = sum;
    }
}

fn cbl(bd: &mut [f32], b: &[f32], amz: &[f32]) {
    let bo = b.len();
    let rv: f32 = b.iter().map(|p| p * p).sum();
    let wq = 1.0 / (rv / bo as f32 + CGL_).bfj();
    for a in 0..bo { bd[a] = b[a] * wq * amz[a]; }
}

fn gss(f: &mut [f32]) {
    if f.is_empty() { return; }
    let am = f.iter().hu().cqs(f32::IP_, f32::am);
    for p in f.el() { *p = (*p - am).cqh(); }
    let sum: f32 = f.iter().sum();
    if sum > 0.0 { let wq = 1.0 / sum; for p in f.el() { *p *= wq; } }
}

fn gyy(f: &[f32]) -> u8 {
    let mut bdn = 0;
    for a in 1..f.len() { if f[a] > f[bdn] { bdn = a; } }
    bdn as u8
}

fn bhu(len: usize, bv: f32, dv: &mut u64) -> Vec<f32> {
    (0..len).map(|_| mrs(dv) * bv).collect()
}

fn mrs(g: &mut u64) -> f32 {
    let mut b = *g;
    b ^= b << 13; b ^= b >> 7; b ^= b << 17;
    *g = b;
    let fs = (b >> 40) as u32;
    (fs as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
}

fn avb(f: &[f32], u: &mut usize, az: usize) -> Option<Vec<f32>> {
    if *u + az > f.len() { return None; }
    let p = f[*u..*u + az].ip();
    *u += az;
    Some(p)
}





trait Wo {
    fn cqh(self) -> f32;
    fn bfj(self) -> f32;
    fn ees(self) -> f32;
}

impl Wo for f32 {
    fn cqh(self) -> f32 {
        let b = self.qp(-88.0, 88.0);
        let q = 12102203.0f32;
        let o = 1065353216.0f32;
        let fs = ((q * b + o) as i32).am(0) as u32;
        f32::bhb(fs)
    }

    fn bfj(self) -> f32 {
        if self <= 0.0 { return 0.0; }
        let fs = self.bsr();
        let anj = f32::bhb((fs >> 1) + 0x1FBD_1DF5);
        (anj + self / anj) * 0.5
    }

    fn ees(self) -> f32 {
        if self <= 0.0 { return -88.0; }
        let fs = self.bsr();
        let aa = ((fs >> 23) & 0xFF) as f32 - 127.0;
        let ef = f32::bhb((fs & 0x007FFFFF) | 0x3F800000);
        (aa + (ef - 1.0) * 1.4427) * core::f32::consts::IG_
    }
}
