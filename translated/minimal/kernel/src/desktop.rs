









use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;
use crate::framebuffer::{self, B_, G_, AU_, Q_, MG_};
use crate::apps::text_editor::{EditorState, ehm};
use core::sync::atomic::{AtomicBool, Ordering};
use crate::math::ahn;


#[inline]
fn bei(adg: u32, tnp: u32) -> u32 {
    crate::accessibility::qee(adg, tnp)
}


static MX_: AtomicBool = AtomicBool::new(false);





static AXN_: Mutex<Option<String>> = Mutex::new(None);

static AXO_: Mutex<Option<Vec<String>>> = Mutex::new(None);

static UH_: AtomicBool = AtomicBool::new(false);





static ZR_: Mutex<Option<String>> = Mutex::new(None);

static ALX_: Mutex<Option<Result<(String, u16, Vec<(String, String)>, Vec<u8>), String>>> = Mutex::new(None);

static RR_: AtomicBool = AtomicBool::new(false);


fn naf(mse: u64) -> i32 {
    let url = {
        let mut aln = ZR_.lock();
        aln.take()
    };
    let url = match url {
        Some(tm) => tm,
        None => {
            RR_.store(false, Ordering::SeqCst);
            return 0;
        }
    };

    
    let amf = crate::browser::gnx(&url, "");
    crate::serial_println!("[BROWSER-BG] Fetching: {}", amf);

    let result = if amf.cj("https://") {
        match crate::netstack::https::get(&amf) {
            Ok(m) => Ok((amf, m.wt, m.zk, m.gj)),
            Err(aa) => Err(alloc::format!("HTTPS error: {}", aa)),
        }
    } else {
        match crate::netstack::http::get(&amf) {
            Ok(m) => Ok((amf, m.wt, m.zk, m.gj)),
            Err(aa) => Err(alloc::format!("Network error: {}", aa)),
        }
    };

    {
        let mut lnn = ALX_.lock();
        *lnn = Some(result);
    }
    RR_.store(false, Ordering::SeqCst);
    0
}


fn uad(mse: u64) -> i32 {
    
    let query = {
        let mut aln = AXN_.lock();
        aln.take()
    };
    let query = match query {
        Some(fm) => fm,
        None => {
            UH_.store(false, Ordering::SeqCst);
            return 0;
        }
    };

    
    crate::shell::jsk(); 
    crate::shell::DE_.store(true, Ordering::SeqCst);
    crate::shell::azu(&query);
    crate::shell::DE_.store(false, Ordering::SeqCst);
    let bjm = crate::shell::jsk();

    
    let mut ak = Vec::new();
    for line in bjm.ak() {
        ak.push(String::from(line));
    }
    {
        let mut result = AXO_.lock();
        *result = Some(ak);
    }
    UH_.store(false, Ordering::SeqCst);
    0
}








const RK_: u32 = 0xFF050606;          
const JJ_: u32 = 0xFF070B09;             
const BLB_: u32 = 0xFF0A0F0C;           
const ALV_: u32 = 0xFF0D1310;            


const I_: u32 = 0xFF00FF66;       
const AG_: u32 = 0xFF00CC55;     
const BK_: u32 = 0xFF00AA44;      
const X_: u32 = 0xFF008844;         
const BH_: u32 = 0xFF006633;        
const P_: u32 = 0xFF003B1A;         


const EC_: u32 = 0xFFB0B2B0;       
const GC_: u32 = 0xFF8C8E8C;          
const AT_: u32 = 0xFF606260;          
const AJ_: u32 = 0xFF3A3C3A;        


const FY_: u32 = 0xFFFFD166;        
const DC_: u32 = 0xFFFF5555;          
const QV_: u32 = 0xFF4ECDC4;         


const DBN_: u32 = 0xFF0A0F0C;
const EKM_: u32 = 0xFF070B09;
const EIF_: u32 = 0xFF080C09;
const EIG_: u32 = 0xFF060908;
const EIE_: u32 = X_;


const AQN_: u32 = 0xFF060908;
const BSA_: u32 = 0xFF0D1310;
const DJS_: u32 = 0xFF101815;


const BAI_: u32 = 0xFF080C09;
const BAJ_: u32 = 0xFF0D1310;
const CGH_: u32 = 0xFF1A2A20;
const DTX_: u32 = 0xFF1A2A20;


const AMA_: u32 = 0xFF3A2828;
const DDY_: u32 = 0xFFFF5555;
const AMB_: u32 = 0xFF2A2A20;
const DDZ_: u32 = 0xFFFFD166;
const AMC_: u32 = 0xFF283028;
const DEA_: u32 = 0xFF00CC55;


const AC_: u32 = 0xFFE0E8E4;
const N_: u32 = 0xFF8A9890;
const PY_: u32 = 0xFF00CC55;


const CXL_: u32 = AQN_;
const EHG_: u32 = BSA_;
const DEF_: u32 = AMA_;
const DEG_: u32 = AMB_;
const DED_: u32 = AMC_;
const DGS_: u32 = BAI_;
const DGU_: u32 = BAJ_;
const DGT_: u32 = CGH_;
const DJD_: u32 = RK_;
const DJC_: u32 = 0xFF020303;


const W_: u32 = 48;
const J_: u32 = 28;             
const DBO_: u32 = 8;          
const EKO_: u32 = 16;
const BU_: u32 = 32;               
const BY_: u32 = 72;                   


const DLO_: u8 = 8;






static ZA_: Mutex<bool> = Mutex::new(true);
static GA_: Mutex<f32> = Mutex::new(1.0); 


const BKG_: u32 = 12;      
const BKD_: u32 = 8;      
const BKF_: u32 = 10;  
const BKE_: u32 = 10;  


#[derive(Clone, Copy, PartialEq)]
pub enum AnimationState {
    None,
    Akv,      
    Pb,      
    Akh,   
    Avk,   
    Ayc,    
}


#[derive(Clone)]
pub struct WindowAnimation {
    pub g: AnimationState,
    pub li: f32,           
    pub ql: i32,
    pub vc: i32,
    pub fvo: u32,
    pub fvm: u32,
    pub ayw: i32,
    pub ayx: i32,
    pub fwh: u32,
    pub fwd: u32,
    pub dw: f32,              
}

impl WindowAnimation {
    pub fn new() -> Self {
        Self {
            g: AnimationState::None,
            li: 0.0,
            ql: 0,
            vc: 0,
            fvo: 0,
            fvm: 0,
            ayw: 0,
            ayx: 0,
            fwh: 0,
            fwd: 0,
            dw: 1.0,
        }
    }
    
    
    pub fn wsy(&mut self, b: i32, c: i32, z: u32, ac: u32) {
        self.g = AnimationState::Akv;
        self.li = 0.0;
        
        self.ql = b + z as i32 / 2 - 10;
        self.vc = c + ac as i32 / 2 - 10;
        self.fvo = 20;
        self.fvm = 20;
        self.ayw = b;
        self.ayx = c;
        self.fwh = z;
        self.fwd = ac;
        self.dw = 0.0;
    }
    
    
    pub fn wso(&mut self, b: i32, c: i32, z: u32, ac: u32) {
        self.g = AnimationState::Pb;
        self.li = 0.0;
        self.ql = b;
        self.vc = c;
        self.fvo = z;
        self.fvm = ac;
        
        self.ayw = b + z as i32 / 2 - 10;
        self.ayx = c + ac as i32 / 2 - 10;
        self.fwh = 20;
        self.fwd = 20;
        self.dw = 1.0;
    }
    
    
    pub fn wsx(&mut self, b: i32, c: i32, z: u32, ac: u32, mkb: i32, ejr: i32) {
        self.g = AnimationState::Akh;
        self.li = 0.0;
        self.ql = b;
        self.vc = c;
        self.fvo = z;
        self.fvm = ac;
        self.ayw = mkb;
        self.ayx = ejr;
        self.fwh = 48;
        self.fwd = 32;
        self.dw = 1.0;
    }
    
    
    pub fn wsv(&mut self, b: i32, c: i32, z: u32, ac: u32, hrh: u32, hra: u32) {
        self.g = AnimationState::Avk;
        self.li = 0.0;
        self.ql = b;
        self.vc = c;
        self.fvo = z;
        self.fvm = ac;
        self.ayw = 0;
        self.ayx = 0;
        self.fwh = hrh;
        self.fwd = hra - W_;
        self.dw = 1.0;
    }
    
    
    pub fn wta(&mut self, rrt: i32, rru: i32, rrs: u32, rrr: u32,
                         exy: i32, exz: i32, wcy: u32, wcx: u32) {
        self.g = AnimationState::Ayc;
        self.li = 0.0;
        self.ql = rrt;
        self.vc = rru;
        self.fvo = rrs;
        self.fvm = rrr;
        self.ayw = exy;
        self.ayx = exz;
        self.fwh = wcy;
        self.fwd = wcx;
        self.dw = 1.0;
    }
    
    
    pub fn qs(&mut self) -> bool {
        if self.g == AnimationState::None {
            return false;
        }
        
        let ig = *GA_.lock();
        let avr = match self.g {
            AnimationState::Akv => BKG_,
            AnimationState::Pb => BKD_,
            AnimationState::Akh => BKF_,
            AnimationState::Avk | AnimationState::Ayc => BKE_,
            AnimationState::None => return false,
        };
        
        let gu = ig / avr as f32;
        self.li += gu;
        
        
        match self.g {
            AnimationState::Akv => {
                self.dw = npa(self.li);
            }
            AnimationState::Pb | AnimationState::Akh => {
                self.dw = 1.0 - noz(self.li);
            }
            _ => {}
        }
        
        if self.li >= 1.0 {
            self.li = 1.0;
            let rna = self.g;
            self.g = AnimationState::None;
            return rna == AnimationState::Pb;
        }
        
        false 
    }
    
    
    pub fn tdf(&self) -> (i32, i32, u32, u32, f32) {
        let ab = match self.g {
            AnimationState::Akv | AnimationState::Ayc => sig(self.li),
            AnimationState::Pb => sif(self.li),
            AnimationState::Akh => noz(self.li),
            AnimationState::Avk => npa(self.li),
            AnimationState::None => 1.0,
        };
        
        let b = oiv(self.ql, self.ayw, ab);
        let c = oiv(self.vc, self.ayx, ab);
        let d = oiw(self.fvo, self.fwh, ab);
        let i = oiw(self.fvm, self.fwd, ab);
        
        (b, c, d, i, self.dw)
    }
    
    
    pub fn lfw(&self) -> bool {
        self.g != AnimationState::None
    }
}






fn oiv(q: i32, o: i32, ab: f32) -> i32 {
    (q as f32 + (o - q) as f32 * ab) as i32
}


fn oiw(q: u32, o: u32, ab: f32) -> u32 {
    if q > o {
        (q as f32 - (q - o) as f32 * ab) as u32
    } else {
        (q as f32 + (o - q) as f32 * ab) as u32
    }
}


fn npa(ab: f32) -> f32 {
    let ab = ab.qp(0.0, 1.0);
    1.0 - (1.0 - ab) * (1.0 - ab) * (1.0 - ab)
}


fn noz(ab: f32) -> f32 {
    let ab = ab.qp(0.0, 1.0);
    ab * ab * ab
}


fn sig(ab: f32) -> f32 {
    let ab = ab.qp(0.0, 1.0);
    let rw: f32 = 1.70158;
    let der = rw + 1.0;
    let idt = ab - 1.0;
    1.0 + der * idt * idt * idt + rw * idt * idt
}


fn sif(ab: f32) -> f32 {
    let ab = ab.qp(0.0, 1.0);
    let rw: f32 = 1.70158;
    let der = rw + 1.0;
    der * ab * ab * ab - rw * ab * ab
}


pub fn col() -> bool {
    *ZA_.lock()
}


pub fn jop(iq: bool) {
    *ZA_.lock() = iq;
    crate::serial_println!("[ANIM] Animations {}", if iq { "ENABLED" } else { "DISABLED" });
}


pub fn xir() {
    let mut iq = ZA_.lock();
    *iq = !*iq;
    crate::serial_println!("[ANIM] Animations {}", if *iq { "ENABLED" } else { "DISABLED" });
}


pub fn pio(ig: f32) {
    *GA_.lock() = ig.qp(0.25, 4.0);
    crate::serial_println!("[ANIM] Speed set to {}x", ig);
}


pub fn hlf() -> f32 {
    *GA_.lock()
}


static BBM_: Mutex<u32> = Mutex::new(1);


#[derive(Clone)]
pub struct An {
    pub cu: String,
    pub hr: ContextAction,
}


#[derive(Clone, Copy, PartialEq)]
pub enum ContextAction {
    Ck,
    Awl,
    Jj,
    Axv,
    Ady,
    Axs,
    Awa,
    Bnh,
    Bdt,
    Aql,
    Copy,
    Awr,
    Bvt,
    Cpl,
    Cpk,
    Azb,
    Cnf,
    Btf,
    Bow,
    Azu,
    Hl,
}




#[derive(Clone)]
pub struct CellPixels {
    pub hz: [u32; 128],  
}

impl CellPixels {
    pub const fn mzj() -> Self {
        CellPixels { hz: [0; 128] }
    }

    
    pub fn sxv(r: char, s: u32) -> Self {
        let ka = crate::framebuffer::font::ada(r);
        let mut y = [0u32; 128];
        for br in 0..16 {
            let fs = ka[br];
            for ga in 0..8u8 {
                if fs & (0x80 >> ga) != 0 {
                    y[br * 8 + ga as usize] = s;
                }
            }
        }
        CellPixels { hz: y }
    }

    
    #[inline]
    pub fn oj(&mut self, b: u8, c: u8, s: u32) {
        if b < 8 && c < 16 {
            self.hz[c as usize * 8 + b as usize] = s;
        }
    }

    
    #[inline]
    pub fn get(&self, b: u8, c: u8) -> u32 {
        if b < 8 && c < 16 { self.hz[c as usize * 8 + b as usize] } else { 0 }
    }

    
    pub fn vi(&mut self, s: u32) {
        self.hz = [s; 128];
    }
}






pub struct MatrixProjection {
    pub b: u32,
    pub c: u32,
    pub z: u32,
    pub ac: u32,
    pub hz: Vec<u32>,  
    pub gh: bool,
}

impl MatrixProjection {
    pub const fn azs() -> Self {
        MatrixProjection {
            b: 0,
            c: 0,
            z: 0,
            ac: 0,
            hz: Vec::new(),
            gh: false,
        }
    }

    
    pub fn tco(z: u32, ac: u32) -> Vec<u32> {
        let d = z as usize;
        let i = ac as usize;
        let mut hz = vec![0u32; d * i];

        for x in 0..i {
            for y in 0..d {
                
                let tm = y as f32 / d as f32;
                let p = x as f32 / i as f32;
                
                let cx = tm * 2.0 - 1.0;
                let ae = p * 2.0 - 1.0;

                
                
                fn ahn(b: f32) -> f32 {
                    if b <= 0.0 { return 0.0; }
                    let mut anj = b * 0.5;
                    anj = 0.5 * (anj + b / anj);
                    anj = 0.5 * (anj + b / anj);
                    anj
                }
                let bc = ahn(cx * cx + ae * ae); 

                
                fn lz(b: f32) -> f32 {
                    
                    let b = b % 6.2832;
                    let b = if b > 3.1416 { b - 6.2832 } else if b < -3.1416 { b + 6.2832 } else { b };
                    
                    if b < 0.0 {
                        1.27323954 * b + 0.405284735 * b * b
                    } else {
                        1.27323954 * b - 0.405284735 * b * b
                    }
                }

                let bic = lz(tm * 10.0 + p * 6.0) * 0.5 + 0.5;
                let cuc = lz(bc * 12.0 - p * 4.0) * 0.5 + 0.5;
                let hym = lz((cx + ae) * 8.0) * 0.5 + 0.5;

                let mut m = (bic * 0.5 + cuc * 0.3 + hym * 0.2).v(1.0);
                let mut at = (cuc * 0.5 + hym * 0.3 + bic * 0.2).v(1.0);
                let mut o = (hym * 0.5 + bic * 0.3 + cuc * 0.2).v(1.0);

                

                
                let irc = cx.gp() + ae.gp();
                if irc < 0.35 {
                    let ab = 1.0 - irc / 0.35;
                    m = m * (1.0 - ab * 0.8) + 0.1 * ab;
                    at = at * (1.0 - ab * 0.5) + 1.0 * ab * 0.5 + at * ab * 0.5;
                    o = o * (1.0 - ab * 0.5) + 1.0 * ab * 0.5 + o * ab * 0.5;
                }

                
                let pdf = (bc - 0.5).gp();
                if pdf < 0.04 {
                    let ab = 1.0 - pdf / 0.04;
                    m = (m + ab * 0.9).v(1.0);
                    at = at * (1.0 - ab * 0.6);
                    o = (o + ab * 0.8).v(1.0);
                }
                let pdg = (bc - 0.75).gp();
                if pdg < 0.03 {
                    let ab = 1.0 - pdg / 0.03;
                    m = (m + ab * 0.3).v(1.0);
                    at = (at + ab * 0.9).v(1.0);
                    o = at * (1.0 - ab * 0.3);
                }

                
                let puo = (1.0 - tm) + (1.0 - p);
                if puo > 1.7 {
                    let ab = ((puo - 1.7) / 0.3).v(1.0);
                    m = (m + ab * 0.6).v(1.0);
                    at = (at + ab * 0.3).v(1.0);
                    o = o * (1.0 - ab * 0.4);
                }
                let mzv = tm + p;
                if mzv > 1.7 {
                    let ab = ((mzv - 1.7) / 0.3).v(1.0);
                    m = m * (1.0 - ab * 0.3);
                    at = (at + ab * 0.4).v(1.0);
                    o = (o + ab * 0.7).v(1.0);
                }

                
                if ae.gp() < 0.012 || cx.gp() < 0.012 {
                    m = (m * 0.5 + 0.5).v(1.0);
                    at = (at * 0.5 + 0.5).v(1.0);
                    o = (o * 0.5 + 0.5).v(1.0);
                }

                
                let fyi: f32 = if (1.0 - bc * 0.7) > 0.0 { 1.0 - bc * 0.7 } else { 0.0 };
                m *= fyi;
                at *= fyi;
                o *= fyi;

                
                m = (m * 1.3).v(1.0);
                at = (at * 1.2).v(1.0);
                o = (o * 1.3).v(1.0);

                let jl = (m * 255.0) as u32;
                let iwz = (at * 255.0) as u32;
                let cvv = (o * 255.0) as u32;
                hz[x * d + y] = 0xFF000000 | (jl << 16) | (iwz << 8) | cvv;
            }
        }
        hz
    }
}


#[derive(Clone)]
pub struct Wb {
    pub iw: bool,
    pub b: i32,
    pub c: i32,
    pub pj: Vec<An>,
    pub acm: usize,
    pub fwe: Option<usize>,  
    pub ejm: Option<String>, 
}


#[derive(Clone)]
pub struct Aqv {
    pub j: String,
    pub ecz: crate::icons::IconType,
    pub b: u32,
    pub c: u32,
    pub hr: IconAction,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IconAction {
    Tu,
    Aks,
    Akt,
    Akq,
    Bod,
    Bnw,
    Boe,
    Bnz,
    Bny,
    Cii,
    Bnv,
    Boc,
    Cij,
    #[cfg(feature = "emulators")]
    Cik,
    #[cfg(feature = "emulators")]
    Boa,
    #[cfg(feature = "emulators")]
    Bob,
}


#[derive(Clone, Copy, PartialEq)]
pub enum WindowType {
    Ay,
    Qx,
    Jf,
    Jl,
    Calculator,
    Ak,
    Ag,
    Hy,
    Gn,
    Bp,
    Is,
    Pj,
    Aqu,  
    Io,    
    Browser, 
    Fp, 
    So,  
    Gs,   
    Ih, 
    #[cfg(feature = "emulators")]
    Xt,  
    #[cfg(feature = "emulators")]
    Sp, 
    #[cfg(feature = "emulators")]
    Abx, 
    Ro, 
    Td,      
    #[cfg(feature = "emulators")]
    Lm,      
    Lw,  
    Afs, 
    Aft, 
}


#[derive(Clone)]
pub struct Window {
    pub ad: u32,
    pub dq: String,
    pub b: i32,
    pub c: i32,
    pub z: u32,
    pub ac: u32,
    pub czx: u32,
    pub dtg: u32,
    pub iw: bool,
    pub ja: bool,
    pub aat: bool,
    pub bkk: bool,
    pub cka: bool,
    pub dlg: ResizeEdge,
    pub dgp: i32,
    pub dgq: i32,
    
    pub exy: i32,
    pub exz: i32,
    pub gri: u32,
    pub grh: u32,
    pub ld: WindowType,
    pub ca: Vec<String>,
    pub wn: Option<String>,
    pub acm: usize,
    pub px: usize,
    
    pub cvp: WindowAnimation,
    pub egj: bool,  
}


#[derive(Clone, Copy, PartialEq)]
pub enum ResizeEdge {
    None,
    Ap,
    Ca,
    Jd,
    Hk,
    Dp,
    Dq,
    Dt,
    Du,
}


#[derive(Clone, Copy, PartialEq)]
enum CursorMode {
    Ov,
    Axz,     
    Ayb,     
    Aeh,  
    Aeg,  
    Cep,        
}


#[derive(Clone, Copy, PartialEq)]
pub enum FileManagerViewMode {
    Px,
    Sz,
    Aaz,
    Oh,
}


#[derive(Clone, Copy, PartialEq)]
pub enum Cxf {
    Dfa,
    Djt,
}


pub struct FileManagerState {
    
    pub adv: Vec<String>,
    
    pub cym: usize,
    
    pub ian: bool,
    
    pub iao: u32,
    
    pub wnv: usize,
    
    pub pks: i32,
    
    pub jkw: Vec<(String, String)>, 
    
    pub eit: u8,
    
    pub dcc: bool,
    
    pub ocf: Option<usize>,
    
    pub bla: String,
    
    pub chp: bool,
    
    pub rma: [u32; 4],
}

impl FileManagerState {
    pub fn new() -> Self {
        Self {
            adv: Vec::new(),
            cym: 0,
            ian: false,
            iao: 180,
            wnv: 0,
            pks: -1,
            jkw: vec![
                (String::from("Desktop"), String::from("/")),
                (String::from("Documents"), String::from("/documents")),
                (String::from("Downloads"), String::from("/downloads")),
                (String::from("Music"), String::from("/music")),
                (String::from("Pictures"), String::from("/pictures")),
            ],
            eit: 0,
            dcc: true,
            ocf: None,
            bla: String::new(),
            chp: false,
            rma: [200, 80, 80, 120],
        }
    }
    
    pub fn lwg(&mut self, path: &str) {
        
        if self.cym + 1 < self.adv.len() {
            self.adv.dmu(self.cym + 1);
        }
        self.adv.push(String::from(path));
        self.cym = self.adv.len() - 1;
    }
    
    pub fn nbo(&self) -> bool {
        self.cym > 0
    }
    
    pub fn nbp(&self) -> bool {
        self.cym + 1 < self.adv.len()
    }
    
    pub fn tgm(&mut self) -> Option<&str> {
        if self.cym > 0 {
            self.cym -= 1;
            Some(&self.adv[self.cym])
        } else {
            None
        }
    }
    
    pub fn tgn(&mut self) -> Option<&str> {
        if self.cym + 1 < self.adv.len() {
            self.cym += 1;
            Some(&self.adv[self.cym])
        } else {
            None
        }
    }
}


pub struct ImageViewerState {
    pub hz: Vec<u32>,
    pub esh: u32,
    pub flc: u32,
    pub ddn: u32,     
    pub hud: i32,
    pub hue: i32,
}

impl ImageViewerState {
    pub fn new() -> Self {
        Self { hz: Vec::new(), esh: 0, flc: 0, ddn: 100, hud: 0, hue: 0 }
    }
}


pub struct Bgx {
    pub path: String,
    pub j: String,
    pub jbe: bool,
}


pub struct Bez {
    pub mgm: String,
    pub it: String,
    pub ta: bool,
    pub ql: i32,
    pub vc: i32,
    pub aua: i32,
    pub bbi: i32,
    pub pmc: u32,
    pub gh: bool,
}

impl Window {
    pub fn new(dq: &str, b: i32, c: i32, z: u32, ac: u32, ash: WindowType) -> Self {
        let mut ocz = BBM_.lock();
        let ad = *ocz;
        *ocz += 1;
        
        Window {
            ad,
            dq: String::from(dq),
            b,
            c,
            z,
            ac,
            czx: 200,
            dtg: 150,
            iw: true,
            ja: false,
            aat: false,
            bkk: false,
            cka: false,
            dlg: ResizeEdge::None,
            dgp: 0,
            dgq: 0,
            exy: b,
            exz: c,
            gri: z,
            grh: ac,
            ld: ash,
            ca: Vec::new(),
            wn: None,
            acm: 0,
            px: 0,
            cvp: WindowAnimation::new(),
            egj: false,
        }
    }
    
    
    pub fn qis(&mut self) {
        if col() {
            self.cvp.wsy(self.b, self.c, self.z, self.ac);
        }
    }
    
    
    pub fn qiq(&mut self) -> bool {
        if col() {
            self.cvp.wso(self.b, self.c, self.z, self.ac);
            self.egj = true;
            true 
        } else {
            false 
        }
    }
    
    
    pub fn qir(&mut self, ejr: i32) {
        if col() {
            let mkb = 100; 
            self.cvp.wsx(self.b, self.c, self.z, self.ac, mkb, ejr);
        }
    }
    
    
    pub fn yev(&mut self, wf: u32, aav: u32) {
        if col() {
            self.cvp.wsv(self.b, self.c, self.z, self.ac, wf, aav);
        }
    }
    
    
    pub fn yew(&mut self) {
        if col() {
            self.cvp.wta(
                self.b, self.c, self.z, self.ac,
                self.exy, self.exz, self.gri, self.grh
            );
        }
    }
    
    
    pub fn xor(&mut self) -> bool {
        if self.cvp.lfw() {
            let pkk = self.cvp.qs();
            
            
            if !self.cvp.lfw() && !pkk {
                
            }
            
            return pkk && self.egj;
        }
        false
    }
    
    
    pub fn ytr(&self) -> (i32, i32, u32, u32, f32) {
        if self.cvp.lfw() {
            self.cvp.tdf()
        } else {
            (self.b, self.c, self.z, self.ac, 1.0)
        }
    }
    
    
    pub fn contains(&self, y: i32, x: i32) -> bool {
        if self.aat { return false; }
        y >= self.b && y < self.b + self.z as i32 &&
        x >= self.c && x < self.c + self.ac as i32
    }
    
    
    pub fn odw(&self, y: i32, x: i32) -> bool {
        y >= self.b && y < self.b + self.z as i32 - 90 &&
        x >= self.c && x < self.c + J_ as i32
    }
    
    
    pub fn uxw(&self, y: i32, x: i32) -> bool {
        let pm = 28i32;
        let qx = J_ as i32;
        let bx = self.b + self.z as i32 - pm - 1;
        let je = self.c + 1;
        y >= bx && y < bx + pm && x >= je && x < je + qx
    }
    
    
    pub fn uyg(&self, y: i32, x: i32) -> bool {
        let pm = 28i32;
        let qx = J_ as i32;
        let bx = self.b + self.z as i32 - pm * 2 - 1;
        let je = self.c + 1;
        y >= bx && y < bx + pm && x >= je && x < je + qx
    }
    
    
    pub fn uyh(&self, y: i32, x: i32) -> bool {
        let pm = 28i32;
        let qx = J_ as i32;
        let bx = self.b + self.z as i32 - pm * 3 - 1;
        let je = self.c + 1;
        y >= bx && y < bx + pm && x >= je && x < je + qx
    }
    
    
    pub fn lqj(&self, y: i32, x: i32) -> ResizeEdge {
        if self.bkk { return ResizeEdge::None; }
        
        let jmg = 12i32;
        let oip = self.b;
        let pde = self.b + self.z as i32;
        let pum = self.c;
        let mzw = self.c + self.ac as i32;
        
        let lqi = y >= oip && y < oip + jmg;
        let lqk = y >= pde - jmg && y < pde;
        let lqm = x >= pum && x < pum + jmg;
        let lqf = x >= mzw - jmg && x < mzw;
        
        if lqm && lqi { ResizeEdge::Dp }
        else if lqm && lqk { ResizeEdge::Dq }
        else if lqf && lqi { ResizeEdge::Dt }
        else if lqf && lqk { ResizeEdge::Du }
        else if lqi { ResizeEdge::Ap }
        else if lqk { ResizeEdge::Ca }
        else if lqm { ResizeEdge::Jd }
        else if lqf { ResizeEdge::Hk }
        else { ResizeEdge::None }
    }
    
    
    pub fn idy(&mut self, anv: u32, akr: u32) {
        if self.bkk {
            
            self.b = self.exy;
            self.c = self.exz;
            self.z = self.gri;
            self.ac = self.grh;
            self.bkk = false;
        } else {
            
            self.exy = self.b;
            self.exz = self.c;
            self.gri = self.z;
            self.grh = self.ac;
            
            self.b = BY_ as i32;
            self.c = 0;
            self.z = anv.ao(BY_);
            self.ac = akr.ao(W_);
            self.bkk = true;
        }
    }
}


use crate::graphics::{compositor, Compositor, CompositorTheme, WindowSurface, Easing};






#[derive(Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Af,
    Ce,
    Cl,
}


pub struct MusicPlayerState {
    pub g: PlaybackState,
    
    pub audio: Option<Vec<i16>>,
    
    pub gst: String,
    
    pub dfl: usize,
    
    pub alm: usize,
    
    pub bph: usize,
    
    pub cry: u32,
    
    pub dyo: bool,
    
    pub enx: usize,
    
    pub axs: usize,
    
    pub ekp: u32,
    
    pub oz: u64,
    
    pub grw: u64,
    
    pub alu: u64,
    
    pub buh: [f32; 1024],
    pub ceo: [f32; 1024],
    
    pub brp: f32,
    
    pub ato: f32,
    pub aee: f32,
    pub vs: f32,
    pub axg: f32,
    
    pub rf: f32,
    
    pub abo: f32,
    
    pub ewu: f32,
    
    pub cxk: [f32; 43],
    pub drs: usize,
    pub cab: usize,
    
    pub ve: [f32; 128],
    pub ihd: usize,
    
    pub hq: u32,
    
    pub emk: i32,
    
    pub gkg: bool,
    
    pub dxb: Vec<String>,
    
    pub mmq: usize,
}

impl MusicPlayerState {
    pub fn new() -> Self {
        Self {
            g: PlaybackState::Af,
            audio: None,
            gst: String::from("No Track"),
            dfl: 0,
            alm: 0,
            bph: 0,
            cry: 0,
            dyo: false,
            enx: 0,
            axs: 0,
            ekp: 0,
            oz: 0,
            grw: 0,
            alu: 0,
            buh: [0.0; 1024],
            ceo: [0.0; 1024],
            brp: 1.0,
            ato: 0.0,
            aee: 0.0,
            vs: 0.0,
            axg: 0.0,
            rf: 0.0,
            abo: 0.0,
            ewu: 0.0,
            cxk: [0.0; 43],
            drs: 0,
            cab: 0,
            ve: [0.0; 128],
            ihd: 0,
            hq: 75,
            emk: 0,
            gkg: false,
            dxb: Vec::new(),
            mmq: 0,
        }
    }

    
    pub fn ojz(&mut self) {
        self.dxb = crate::trustdaw::disk_audio::tey();
        self.alm = self.dxb.len();
        crate::serial_println!("[MUSIC] Track list: {} tracks", self.alm);
        for (a, j) in self.dxb.iter().cf() {
            crate::serial_println!("[MUSIC]   {}: {}", a, j);
        }
    }

    
    pub fn luh(&mut self) {
        self.dkp(self.dfl);
    }

    
    pub fn dkp(&mut self, zx: usize) {
        
        
        self.qg();

        
        if self.alm == 0 {
            self.ojz();
        }

        if self.alm == 0 {
            self.gst = String::from("No tracks found");
            crate::serial_println!("[MUSIC] No tracks available on disk");
            return;
        }

        let w = zx % self.alm;
        self.dfl = w;

        crate::serial_println!("[MUSIC] Loading track {} — heap free: {} KB",
            w, crate::memory::heap::aez() / 1024);

        
        match crate::trustdaw::disk_audio::ojy(w) {
            Ok((dla, j)) => {
                
                if dla.len() >= 12 {
                    crate::serial_println!("[MUSIC] Track {} raw header: {:02X} {:02X} {:02X} {:02X} ... {:02X} {:02X} {:02X} {:02X}",
                        w, dla[0], dla[1], dla[2], dla[3],
                        dla[8], dla[9], dla[10], dla[11]);
                }
                match crate::trustdaw::audio_viz::hfq(&dla) {
                    Ok(audio) => {
                        crate::serial_println!("[MUSIC] Decoded track {}: '{}' → {} samples", w, j, audio.len());
                        
                        drop(dla);
                        self.wsz(audio, &j);
                        return;
                    }
                    Err(aa) => {
                        crate::serial_println!("[MUSIC] Track {} decode error: {}", w, aa);
                    }
                }
            }
            Err(aa) => {
                crate::serial_println!("[MUSIC] Track {} load error: {}", w, aa);
            }
        }

        
        let j = if w < self.dxb.len() {
            self.dxb[w].clone()
        } else {
            alloc::format!("Track {}", w + 1)
        };
        self.gst = alloc::format!("{} (load failed)", j);
    }

    
    pub fn loq(&mut self) {
        if self.alm > 1 {
            let next = (self.dfl + 1) % self.alm;
            self.dkp(next);
        } else {
            
            self.dkp(self.dfl);
        }
    }

    
    pub fn oxm(&mut self) {
        if self.alm > 1 {
            let vo = if self.dfl == 0 { self.alm - 1 } else { self.dfl - 1 };
            self.dkp(vo);
        } else {
            
            self.dkp(self.dfl);
        }
    }

    
    fn wsz(&mut self, audio: Vec<i16>, dq: &str) {
        self.gst = String::from(dq);
        let agc = audio.len() / 2;
        self.alu = (agc as u64 * 1000) / 48000;

        
        crate::audio::init().bq();

        
        let axs = crate::drivers::hda::gic()
            .map(|(_, r)| r)
            .unwrap_or(0);
        if axs == 0 {
            crate::serial_println!("[MUSIC] No DMA buffer available");
            return;
        }

        
        let _ = crate::drivers::hda::qg();
        crate::drivers::hda::jmf();

        
        let cfo = audio.len().v(axs);
        if let Ok(()) = crate::drivers::hda::dcg(&audio[0..cfo]) {
            self.bph = cfo;
            self.axs = axs;
            self.audio = Some(audio);
            self.g = PlaybackState::Ce;
            self.dyo = false;
            self.enx = 0;
            self.grw = 0;
            self.ekp = 0;
            self.oz = 0;

            
            let bvg = crate::drivers::hda::hlj();
            let ghq = (axs * 2) as u32;
            let hmi = ghq / 2;
            let gly = if bvg >= ghq { 0 } else { bvg };
            self.cry = if gly < hmi { 0 } else { 1 };

            
            let _ = crate::drivers::hda::chv(self.hq.v(100) as u8);

            
            self.brp = 1.0;
            self.ato = 0.0;
            self.aee = 0.0;
            self.vs = 0.0;
            self.axg = 0.0;
            self.rf = 0.0;
            self.abo = 0.0;
            self.ewu = 0.0;
            self.cxk = [0.0; 43];
            self.drs = 0;
            self.cab = 0;
            self.ve = [0.0; 128];
            self.ihd = 0;
            crate::serial_println!("[MUSIC] Playing '{}', {}ms, DMA={}", self.gst, self.alu, axs);
        } else {
            crate::serial_println!("[MUSIC] start_looped_playback failed");
        }
    }

    
    pub fn qg(&mut self) {
        if self.g != PlaybackState::Af {
            let _ = crate::drivers::hda::qg();
            crate::drivers::hda::jmf();
            self.g = PlaybackState::Af;
            self.audio = None;
            self.bph = 0;
            self.enx = 0;
            self.axs = 0;
            self.grw = 0;
            self.ekp = 0;
            self.oz = 0;
            self.gkg = false;
            
            self.rf = 0.0;
            self.abo = 0.0;
            self.ato = 0.0;
            self.aee = 0.0;
            self.vs = 0.0;
            self.axg = 0.0;
            crate::serial_println!("[MUSIC] Stopped");
        }
    }

    
    pub fn mlq(&mut self) {
        match self.g {
            PlaybackState::Ce => {
                let _ = crate::drivers::hda::qg();
                self.g = PlaybackState::Cl;
                crate::serial_println!("[MUSIC] Paused at {}ms", self.oz);
            }
            PlaybackState::Cl => {
                
                self.pcx();
            }
            _ => {}
        }
    }

    
    
    fn pcx(&mut self) {
        
        let audio = match self.audio.take() {
            Some(q) => q,
            None => return,
        };
        let axs = crate::drivers::hda::gic()
            .map(|(_, r)| r)
            .unwrap_or(0);
        if axs == 0 {
            self.audio = Some(audio);
            return;
        }

        
        let dvj = ((self.oz as usize * 48000 * 2) / 1000).v(audio.len());

        
        let _ = crate::drivers::hda::qg();
        crate::drivers::hda::jmf();

        
        let cfo = audio.len().ao(dvj).v(axs);
        if cfo == 0 {
            self.audio = Some(audio);
            self.qg();
            return;
        }
        if let Ok(()) = crate::drivers::hda::dcg(&audio[dvj..dvj + cfo]) {
            self.bph = dvj + cfo;
            self.axs = axs;
            self.enx = 0;
            self.grw = self.oz;
            self.g = PlaybackState::Ce;
            self.dyo = false;

            
            self.cry = 0;

            let _ = crate::drivers::hda::chv(self.hq.v(100) as u8);
            crate::serial_println!("[MUSIC] Resumed at {}ms, sample={}", self.oz, dvj);
        } else {
            crate::serial_println!("[MUSIC] Resume start_looped_playback failed");
        }
        
        self.audio = Some(audio);
    }

    
    pub fn wgm(&mut self, icy: u64) {
        let icy = icy.v(self.alu);
        self.oz = icy;

        
        if self.g == PlaybackState::Cl {
            crate::serial_println!("[MUSIC] Seek (paused) to {}ms", icy);
            return;
        }

        
        if self.g == PlaybackState::Ce {
            self.pcx();
            crate::serial_println!("[MUSIC] Seek (playing) to {}ms", icy);
        }
    }

    
    pub fn or(&mut self) {
        if self.g != PlaybackState::Ce { return; }
        let audio = match &self.audio {
            Some(q) => q,
            None => return,
        };

        
        if let Some((dqc, axs)) = crate::drivers::hda::gic() {
            let baa = axs / 2;
            let hmi = (baa * 2) as u32;
            let ghq = (axs * 2) as u32;

            crate::drivers::hda::ndi();
            crate::drivers::hda::nqi();

            let bvg = crate::drivers::hda::hlj();
            let gly = if bvg >= ghq { 0 } else { bvg };
            let heu = if gly < hmi { 0u32 } else { 1u32 };

            if heu != self.cry {
                
                self.enx += baa;

                if self.bph < audio.len() {
                    let cpx = self.cry as usize * baa;
                    let ia = audio.len() - self.bph;
                    let acq = ia.v(baa);
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            audio.fq().add(self.bph),
                            dqc.add(cpx),
                            acq,
                        );
                        if acq < baa {
                            
                            if self.gkg && !audio.is_empty() {
                                let mut adu = acq;
                                while adu < baa {
                                    let ihj = (baa - adu).v(audio.len());
                                    core::ptr::copy_nonoverlapping(
                                        audio.fq(),
                                        dqc.add(cpx + adu),
                                        ihj,
                                    );
                                    adu += ihj;
                                }
                            } else {
                                core::ptr::ahx(dqc.add(cpx + acq), 0, baa - acq);
                            }
                        }
                    }
                    self.bph += acq;
                    if self.bph >= audio.len() {
                        if self.gkg {
                            self.bph = 0; 
                        } else {
                            self.dyo = true;
                        }
                    }
                } else if self.gkg && !audio.is_empty() {
                    
                    self.bph = 0;
                    let cpx = self.cry as usize * baa;
                    let acq = audio.len().v(baa);
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            audio.fq(),
                            dqc.add(cpx),
                            acq,
                        );
                        if acq < baa {
                            let mut adu = acq;
                            while adu < baa {
                                let ihj = (baa - adu).v(audio.len());
                                core::ptr::copy_nonoverlapping(
                                    audio.fq(),
                                    dqc.add(cpx + adu),
                                    ihj,
                                );
                                adu += ihj;
                            }
                        }
                    }
                    self.bph += acq;
                } else {
                    let cpx = self.cry as usize * baa;
                    unsafe { core::ptr::ahx(dqc.add(cpx), 0, baa); }
                }
                self.cry = heu;
            }

            
            
            
            let ljz = (gly / 2) as usize; 
            
            
            
            
            
            
            let tog = if self.enx + ljz >= axs {
                self.enx + ljz - axs
            } else {
                ljz 
            };
            
            
            self.oz = self.grw + (tog as u64 * 1000) / (48000 * 2);
        }

        self.ekp += 1;

        
        if self.oz >= self.alu || self.dyo {
            if self.gkg {
                
                self.bph = 0;
                self.dyo = false;
                self.enx = 0;
                self.grw = 0;
                self.oz = 0;
                return;
            }
            
            if self.alm > 1 {
                let next = (self.dfl + 1) % self.alm;
                crate::serial_println!("[MUSIC] Track ended, auto-advancing to track {}", next);
                self.dkp(next);
            } else {
                self.qg();
            }
            return;
        }

        
        let xsc = (self.oz as i64 + self.emk as i64).am(0).v(self.alu as i64) as u64;
        let gan = (xsc as usize * 48000 * 2 / 1000).v(audio.len().ao(2));

        
        let bbp = 256usize;
        let hru = gan.ao(bbp * 2) & !1;
        let mut awd: f32 = 0.0;
        for a in 0..bbp {
            let w = hru + a * 2;
            let e = if w < audio.len() { audio[w] as f32 } else { 0.0 };
            self.buh[a] = e;
            self.ceo[a] = 0.0;
            let q = if e >= 0.0 { e } else { -e };
            if q > awd { awd = q; }
        }
        
        if awd > self.brp {
            self.brp += (awd - self.brp) * 0.3;
        } else {
            self.brp *= 0.9995;
        }
        let dqz = if self.brp > 100.0 { 16000.0 / self.brp } else { 1.0 };
        
        for a in 0..bbp {
            let ab = a as f32 / bbp as f32;
            let hmm = 0.5 * (1.0 - libm::zq(2.0 * core::f32::consts::Eu * ab));
            self.buh[a] *= hmm * dqz / 32768.0;
        }
        
        {
            let ath = &mut self.buh[..bbp];
            let aum = &mut self.ceo[..bbp];
            
            let mut fb = 0usize;
            for a in 0..bbp {
                if a < fb { ath.swap(a, fb); aum.swap(a, fb); }
                let mut ef = bbp >> 1;
                while ef >= 1 && fb >= ef { fb -= ef; ef >>= 1; }
                fb += ef;
            }
            
            let mut gu = 2;
            while gu <= bbp {
                let iv = gu / 2;
                let qii = -core::f32::consts::Eu * 2.0 / gu as f32;
                for eh in 0..iv {
                    let q = qii * eh as f32;
                    let bfu = libm::zq(q);
                    let yi = libm::st(q);
                    let mut cfm = eh;
                    while cfm < bbp {
                        let crw = cfm + iv;
                        let agd = bfu * ath[crw] - yi * aum[crw];
                        let ezs = bfu * aum[crw] + yi * ath[crw];
                        ath[crw] = ath[cfm] - agd; aum[crw] = aum[cfm] - ezs;
                        ath[cfm] += agd; aum[cfm] += ezs;
                        cfm += gu;
                    }
                }
                gu <<= 1;
            }
        }
        
        let efa = |hh: usize, gd: usize| -> f32 {
            let mut e = 0.0f32;
            for a in hh..gd.v(128) {
                e += libm::bon(self.buh[a] * self.buh[a] + self.ceo[a] * self.ceo[a]);
            }
            e / (gd - hh).am(1) as f32
        };
        let frz = efa(1, 2);   
        let dkz = efa(2, 4);  
        let exe = efa(4, 16);  
        let hwr = efa(16, 60); 
        let lxc = frz * 1.5 + dkz * 1.2 + exe * 0.5 + hwr * 0.2;

        
        let bie = |vo: f32, new: f32, q: f32, m: f32| -> f32 {
            if new > vo { vo + (new - vo) * q } else { vo + (new - vo) * m }
        };
        self.ato = bie(self.ato, frz.v(1.0), 0.75, 0.10);
        self.aee = bie(self.aee, dkz.v(1.0), 0.70, 0.10);
        self.vs = bie(self.vs, exe.v(1.0), 0.60, 0.12);
        self.axg = bie(self.axg, hwr.v(1.0), 0.70, 0.16);
        self.abo = bie(self.abo, lxc.v(1.5), 0.65, 0.10);

        
        let dee = frz + dkz * 0.8;
        self.cxk[self.drs] = dee;
        self.drs = (self.drs + 1) % 43;
        if self.cab < 43 { self.cab += 1; }
        let adu = self.cab.am(1) as f32;
        let abl: f32 = self.cxk.iter().take(self.cab).sum::<f32>() / adu;
        let mut fax = 0.0f32;
        for a in 0..self.cab { let bc = self.cxk[a] - abl; fax += bc * bc; }
        let igh = fax / adu;
        let bxm = (-15.0 * igh + 1.45f32).am(1.05).v(1.5);
        let lqp = dee - self.ewu;
        if dee > abl * bxm && lqp > 0.002 && self.cab > 5 {
            let ccc = ((dee - abl * bxm) / abl.am(0.001)).v(1.0);
            self.rf = (0.6 + ccc * 0.4).v(1.0);
        } else {
            self.rf *= 0.88;
            if self.rf < 0.02 { self.rf = 0.0; }
        }
        self.ewu = dee;

        
        if !audio.is_empty() {
            let w = gan.v(audio.len() - 1) & !1;
            let yr = audio[w] as f32 / 32768.0;
            self.ve[self.ihd % 128] = yr;
            self.ihd += 1;
        }
    }
}






#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    
    Apy,
    
    Ks,
    
    
    Atd,
}




#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum DesktopTier {
    
    Aap,
    
    Gy,
    
    Gc,
    
    Bv,
}

pub struct Desktop {
    pub ee: Vec<Window>,
    pub icons: Vec<Aqv>,
    pub lf: i32,
    pub ot: i32,
    pub cwu: bool,
    pub z: u32,
    pub ac: u32,
    oo: u64,
    ajo: bool,
    pub xn: String,
    pub btx: bool,
    pub aka: Wb,
    
    gbz: String,
    hcd: String,
    jcq: u64,
    
    doh: bool,
    bex: bool,
    gkt: i32,
    gku: i32,
    
    etz: usize,
    ety: bool,
    etw: bool,
    
    pub che: RenderMode,
    pub kkf: CompositorTheme,
    
    dpz: [(u32, u32, u32, u32); 32], 
    eov: usize,
    kzn: u32,  
    
    pub browser: Option<crate::browser::Browser>,
    pub ado: String,
    pub aef: usize,
    pub btn: bool,
    pub cdj: bool,
    
    pub cxh: BTreeMap<u32, EditorState>,
    
    pub djq: BTreeMap<u32, crate::model_editor::ModelEditorState>,
    
    pub enf: BTreeMap<u32, CalculatorState>,
    
    pub eyq: BTreeMap<u32, SnakeState>,
    
    pub dra: BTreeMap<u32, crate::game3d::Game3DState>,
    
    pub dou: BTreeMap<u32, crate::chess::ChessState>,
    
    pub cwd: BTreeMap<u32, crate::chess3d::Chess3DState>,
    
    pub fdi: BTreeMap<u32, crate::apps::binary_viewer::BinaryViewerState>,
    
    #[cfg(feature = "emulators")]
    pub dtk: BTreeMap<u32, crate::nes::NesEmulator>,
    
    #[cfg(feature = "emulators")]
    pub arf: BTreeMap<u32, crate::gameboy::GameBoyEmulator>,
    
    pub dso: BTreeMap<u32, crate::lab_mode::LabState>,
    
    pub ano: BTreeMap<u32, MusicPlayerState>,
    
    #[cfg(feature = "emulators")]
    pub azy: BTreeMap<u32, crate::game_lab::GameLabState>,
    
    #[cfg(feature = "emulators")]
    pub ghu: BTreeMap<u32, u32>,
    
    pub mcc: u32,
    
    car: Vec<u8>,
    awc: Vec<i32>,
    czn: Vec<u32>,
    fnr: Vec<u32>,
    lkn: bool,
    hqx: u32,
    lko: bool,
    
    pub eup: u8,
    
    
    
    pub dsz: BTreeMap<usize, CellPixels>,
    
    pub djj: MatrixProjection,
    
    visualizer: crate::visualizer::VisualizerState,
    
    drone_swarm: crate::drone_swarm::DroneSwarmState,
    
    
    drh: Vec<f32>,
    erf: Vec<f32>,
    erh: f32,
    ere: f32,
    erg: f32,
    eri: f32,
    ecd: f32,
    drg: f32,
    fju: f32,
    kze: f32,
    gij: Vec<f32>,
    ixd: usize,
    fjt: usize,
    fjs: bool,
    
    fwk: usize,
    
    bqa: Vec<String>,
    ari: Option<usize>,
    hyr: String,
    
    pub bij: String,
    
    pub bsl: i32,
    
    hcz: Option<(usize, bool)>,
    
    
    
    pub eqn: BTreeMap<u32, FileManagerViewMode>,
    
    pub avu: BTreeMap<u32, FileManagerState>,
    
    pub gjo: BTreeMap<u32, ImageViewerState>,
    
    pub iuk: Option<Bgx>,
    
    pub eaz: Option<Bez>,
    
    pub dvt: u8,
    
    pub efr: u8,
    
    pub eug: bool,
    
    pub djb: String,
    
    pub eeu: u32,
    
    pub gtw: u32,
    
    pub icm: u32,
    
    pub mje: bool,
    
    
    
    pub jwu: usize,
    
    pub gws: usize,
    
    pub ddl: String,
    
    pub ihf: String,
    
    pub fyt: bool,
    
    pub xum: bool,
    
    pub fbj: Option<String>,
    
    
    
    kye: crate::gesture::GestureRecognizer,
    
    hle: crate::gesture::GestureBuffer,
    
    pub pvl: bool,
    
    pub ud: crate::mobile::MobileState,
    
    
    hkf: u64,
    
    ivo: u32,
    
    pub cya: u32,
    
    pub swn: bool,
    
    pub asr: DesktopTier,
    
    bqt: u32,
    
    cqt: u32,
    
    lel: DesktopTier,
    
    pub dcq: bool,
    
    dwc: Option<SnapDir>,
    
    iak: bool,
    
    
    mfr: bool,
    
    mfs: u64,
    
    pkp: u8,
}


pub struct CalculatorState {
    
    pub xz: String,
    
    pub display: String,
    
    pub cle: bool,
    
    pub mck: bool,
}

impl CalculatorState {
    pub fn new() -> Self {
        CalculatorState {
            xz: String::new(),
            display: String::from("0"),
            cle: false,
            mck: false,
        }
    }
    
    pub fn oxb(&mut self, bc: char) {
        if self.cle {
            self.xz.clear();
            self.cle = false;
        }
        if self.xz.len() < 64 {
            self.xz.push(bc);
            self.display = self.xz.clone();
        }
    }
    
    pub fn oxc(&mut self) {
        if self.cle {
            self.xz = String::from("0");
            self.cle = false;
        }
        self.xz.push('.');
        self.display = self.xz.clone();
    }
    
    pub fn duq(&mut self, op: char) {
        if self.cle {
            
            self.cle = false;
        }
        if !self.xz.is_empty() {
            self.xz.push(op);
            self.display = self.xz.clone();
        }
    }
    
    pub fn jju(&mut self, ai: char) {
        if self.cle && ai == '(' {
            self.xz.clear();
            self.cle = false;
        }
        self.xz.push(ai);
        self.display = self.xz.clone();
    }
    
    pub fn vkz(&mut self, j: &str) {
        if self.cle {
            self.xz.clear();
            self.cle = false;
        }
        self.xz.t(j);
        self.xz.push('(');
        self.display = self.xz.clone();
    }
    
    pub fn oxd(&mut self) {
        let result = Self::itb(&self.xz);
        self.display = Self::hkd(result);
        
        self.xz = self.display.clone();
        self.cle = true;
    }
    
    pub fn oxa(&mut self) {
        self.xz.clear();
        self.display = String::from("0");
        self.cle = false;
    }
    
    pub fn owz(&mut self) {
        if !self.xz.is_empty() {
            
            let hks = ["sqrt(", "sin(", "cos(", "tan(", "abs(", "ln("];
            let mut pbs = false;
            for bb in hks {
                if self.xz.pp(bb) {
                    for _ in 0..bb.len() { self.xz.pop(); }
                    pbs = true;
                    break;
                }
            }
            if !pbs {
                self.xz.pop();
            }
            if self.xz.is_empty() {
                self.display = String::from("0");
            } else {
                self.display = self.xz.clone();
            }
        }
    }
    
    
    pub fn zsz(&mut self) {
        self.mck = !self.mck;
    }
    
    
    
    
    
    
    
    fn itb(expr: &str) -> f64 {
        let eb = Self::fwz(expr);
        let mut u = 0;
        let result = Self::bey(&eb, &mut u);
        result
    }
    
    fn fwz(expr: &str) -> Vec<CalcToken> {
        let mut eb = Vec::new();
        let bw: Vec<char> = expr.bw().collect();
        let mut a = 0;
        while a < bw.len() {
            let bm = bw[a];
            if bm.atb() || bm == '.' {
                
                let ay = a;
                while a < bw.len() && (bw[a].atb() || bw[a] == '.') {
                    a += 1;
                }
                let ajh: String = bw[ay..a].iter().collect();
                eb.push(CalcToken::Adn(Self::lsj(&ajh)));
            } else if bm.gke() {
                
                let ay = a;
                while a < bw.len() && bw[a].gke() {
                    a += 1;
                }
                let j: String = bw[ay..a].iter().collect();
                eb.push(CalcToken::Bhm(j));
            } else if bm == '(' {
                eb.push(CalcToken::Kr);
                a += 1;
            } else if bm == ')' {
                eb.push(CalcToken::Jv);
                a += 1;
            } else if bm == '+' || bm == '-' || bm == '*' || bm == '/' || bm == '%' {
                eb.push(CalcToken::Op(bm));
                a += 1;
            } else {
                a += 1; 
            }
        }
        eb
    }
    
    fn bey(eb: &[CalcToken], u: &mut usize) -> f64 {
        let mut fd = Self::fqi(eb, u);
        while *u < eb.len() {
            match &eb[*u] {
                CalcToken::Op('+') => { *u += 1; fd += Self::fqi(eb, u); }
                CalcToken::Op('-') => { *u += 1; fd -= Self::fqi(eb, u); }
                _ => break,
            }
        }
        fd
    }
    
    fn fqi(eb: &[CalcToken], u: &mut usize) -> f64 {
        let mut fd = Self::dkn(eb, u);
        while *u < eb.len() {
            match &eb[*u] {
                CalcToken::Op('*') => { *u += 1; fd *= Self::dkn(eb, u); }
                CalcToken::Op('/') => { *u += 1; let m = Self::dkn(eb, u); fd = if m != 0.0 { fd / m } else { 0.0 }; }
                CalcToken::Op('%') => { *u += 1; let m = Self::dkn(eb, u); fd = if m != 0.0 { fd % m } else { 0.0 }; }
                _ => break,
            }
        }
        fd
    }
    
    fn dkn(eb: &[CalcToken], u: &mut usize) -> f64 {
        
        if *u < eb.len() {
            if let CalcToken::Op('-') = &eb[*u] {
                *u += 1;
                return -Self::otx(eb, u);
            }
        }
        Self::otx(eb, u)
    }
    
    fn otx(eb: &[CalcToken], u: &mut usize) -> f64 {
        if *u >= eb.len() { return 0.0; }
        
        match &eb[*u] {
            CalcToken::Adn(bo) => {
                let p = *bo;
                *u += 1;
                p
            }
            CalcToken::Kr => {
                *u += 1; 
                let p = Self::bey(eb, u);
                if *u < eb.len() {
                    if let CalcToken::Jv = &eb[*u] { *u += 1; }
                }
                p
            }
            CalcToken::Bhm(j) => {
                let ebt = j.clone();
                *u += 1; 
                
                if *u < eb.len() {
                    if let CalcToken::Kr = &eb[*u] { *u += 1; }
                }
                let ji = Self::bey(eb, u);
                if *u < eb.len() {
                    if let CalcToken::Jv = &eb[*u] { *u += 1; }
                }
                Self::qjt(&ebt, ji)
            }
            _ => {
                *u += 1;
                0.0
            }
        }
    }
    
    fn qjt(j: &str, b: f64) -> f64 {
        match j {
            "sqrt" => {
                if b >= 0.0 { Self::bfj(b) } else { 0.0 }
            }
            "sin" => Self::boj(b),
            "cos" => Self::byz(b),
            "tan" => {
                let r = Self::byz(b);
                if r.gp() > 1e-10 { Self::boj(b) / r } else { 0.0 }
            }
            "abs" => if b < 0.0 { -b } else { b },
            "ln" => Self::ees(b),
            _ => b,
        }
    }
    
    
    
    fn bfj(b: f64) -> f64 {
        if b <= 0.0 { return 0.0; }
        let mut anj = b / 2.0;
        for _ in 0..20 {
            anj = (anj + b / anj) / 2.0;
        }
        anj
    }
    
    fn boj(b: f64) -> f64 {
        
        let akk = 3.14159265358979323846;
        let mut b = b % (2.0 * akk);
        if b > akk { b -= 2.0 * akk; }
        if b < -akk { b += 2.0 * akk; }
        
        let hy = b * b;
        let ajr = hy * b;
        let fbw = ajr * hy;
        let fyz = fbw * hy;
        let jxm = fyz * hy;
        let xwb = jxm * hy;
        b - ajr / 6.0 + fbw / 120.0 - fyz / 5040.0 + jxm / 362880.0 - xwb / 39916800.0
    }
    
    fn byz(b: f64) -> f64 {
        let akk = 3.14159265358979323846;
        Self::boj(b + akk / 2.0)
    }
    
    fn ees(b: f64) -> f64 {
        if b <= 0.0 { return 0.0; }
        
        let c = (b - 1.0) / (b + 1.0);
        let jz = c * c;
        let mut result = c;
        let mut asc = c;
        for bo in 1..30 {
            asc *= jz;
            result += asc / (2 * bo + 1) as f64;
        }
        result * 2.0
    }
    
    fn lsj(e: &str) -> f64 {
        let mut result: f64 = 0.0;
        let mut njx = false;
        let mut njw = 0.1;
        let mut opf = false;
        for (a, bm) in e.bw().cf() {
            if bm == '-' && a == 0 {
                opf = true;
            } else if bm == '.' {
                njx = true;
            } else if bm.atb() {
                let dpy = (bm as u8 - b'0') as f64;
                if njx {
                    result += dpy * njw;
                    njw *= 0.1;
                } else {
                    result = result * 10.0 + dpy;
                }
            }
        }
        if opf { -result } else { result }
    }
    
    fn hkd(bo: f64) -> String {
        if bo == (bo as i64) as f64 && bo.gp() < 1e15 {
            format!("{}", bo as i64)
        } else {
            let e = format!("{:.6}", bo);
            let e = e.bdd('0');
            let e = e.bdd('.');
            String::from(e)
        }
    }
}


#[derive(Clone)]
enum CalcToken {
    Adn(f64),
    Op(char),
    Kr,
    Jv,
    Bhm(String),
}


pub struct SnakeState {
    pub atl: Vec<(i32, i32)>,
    pub sz: (i32, i32),
    pub ghh: (i32, i32),
    pub ol: u32,
    pub cev: bool,
    pub ant: bool,
    pub crf: u32,
    pub auk: i32,
    pub bhc: i32,
    pub jtf: u32,
    pub ig: u32,
    pub ajn: u32,
}

impl SnakeState {
    pub fn new() -> Self {
        let mut g = SnakeState {
            atl: Vec::new(),
            sz: (1, 0),
            ghh: (12, 5),
            ol: 0,
            cev: false,
            ant: false,
            crf: 0,
            auk: 20,
            bhc: 15,
            jtf: 0,
            ig: 8, 
            ajn: 42,
        };
        
        for a in 0..4 {
            g.atl.push((10 - a, 7));
        }
        g
    }
    
    fn lop(&mut self) -> u32 {
        self.ajn ^= self.ajn << 13;
        self.ajn ^= self.ajn >> 17;
        self.ajn ^= self.ajn << 5;
        self.ajn
    }
    
    pub fn wqo(&mut self) {
        
        let xjv = (self.auk * self.bhc) as usize;
        if self.atl.len() >= xjv {
            
            self.cev = true;
            if self.ol > self.crf { self.crf = self.ol; }
            return;
        }
        for _ in 0..1000 {
            let jf = (self.lop() % self.auk as u32) as i32;
            let sc = (self.lop() % self.bhc as u32) as i32;
            if !self.atl.iter().any(|&(cr, cq)| cr == jf && cq == sc) {
                self.ghh = (jf, sc);
                return;
            }
        }
        
        for ub in 0..self.bhc {
            for qz in 0..self.auk {
                if !self.atl.iter().any(|&(cr, cq)| cr == qz && cq == ub) {
                    self.ghh = (qz, ub);
                    return;
                }
            }
        }
        
        self.cev = true;
        if self.ol > self.crf { self.crf = self.ol; }
    }
    
    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, AH_, AI_};
        
        if bs == b'p' || bs == b'P' || bs == 0x1B {
            if !self.cev {
                self.ant = !self.ant;
                return;
            }
        }
        if self.cev {
            if bs == b' ' || bs == 0x0D {
                
                let lcu = self.crf;
                *self = SnakeState::new();
                self.crf = lcu;
            }
            return;
        }
        if self.ant { return; }
        match bs {
            V_    if self.sz != (0, 1)  => self.sz = (0, -1),
            U_  if self.sz != (0, -1) => self.sz = (0, 1),
            AH_  if self.sz != (1, 0)  => self.sz = (-1, 0),
            AI_ if self.sz != (-1, 0) => self.sz = (1, 0),
            _ => {}
        }
    }
    
    pub fn or(&mut self) {
        if self.cev || self.ant { return; }
        self.jtf += 1;
        if self.jtf < self.ig { return; }
        self.jtf = 0;
        
        let ale = self.atl[0];
        let efs = (ale.0 + self.sz.0, ale.1 + self.sz.1);
        
        
        if efs.0 < 0 || efs.0 >= self.auk || efs.1 < 0 || efs.1 >= self.bhc {
            self.cev = true;
            if self.ol > self.crf { self.crf = self.ol; }
            return;
        }
        
        
        if self.atl.iter().any(|&e| e == efs) {
            self.cev = true;
            if self.ol > self.crf { self.crf = self.ol; }
            return;
        }
        
        self.atl.insert(0, efs);
        
        
        if efs == self.ghh {
            self.ol += 10;
            self.wqo();
            
            if self.ol % 50 == 0 && self.ig > 3 {
                self.ig -= 1;
            }
        } else {
            self.atl.pop();
        }
    }
}

impl Desktop {
    pub const fn new() -> Self {
        Desktop {
            ee: Vec::new(),
            icons: Vec::new(),
            lf: 640,
            ot: 400,
            cwu: true,
            z: 1280,
            ac: 800,
            oo: 0,
            ajo: false,
            xn: String::new(),
            btx: true,
            aka: Wb {
                iw: false,
                b: 0,
                c: 0,
                pj: Vec::new(),
                acm: 0,
                fwe: None,
                ejm: None,
            },
            gbz: String::new(),
            hcd: String::new(),
            jcq: 0,
            doh: false,
            bex: true,
            gkt: 640,
            gku: 400,
            etz: 0,
            ety: false,
            etw: false,
            che: RenderMode::Apy,
            kkf: CompositorTheme::Xq,
            dpz: [(0, 0, 0, 0); 32],
            eov: 0,
            kzn: 0,
            browser: None,
            ado: String::new(),
            aef: 0,
            btn: false,
            cdj: false,
            cxh: BTreeMap::new(),
            djq: BTreeMap::new(),
            enf: BTreeMap::new(),
            eyq: BTreeMap::new(),
            dra: BTreeMap::new(),
            dou: BTreeMap::new(),
            cwd: BTreeMap::new(),
            #[cfg(feature = "emulators")]
            dtk: BTreeMap::new(),
            #[cfg(feature = "emulators")]
            arf: BTreeMap::new(),
            fdi: BTreeMap::new(),
            dso: BTreeMap::new(),
            ano: BTreeMap::new(),
            #[cfg(feature = "emulators")]
            azy: BTreeMap::new(),
            #[cfg(feature = "emulators")]
            ghu: BTreeMap::new(),
            mcc: 1,
            car: Vec::new(),
            awc: Vec::new(),
            czn: Vec::new(),
            fnr: Vec::new(),
            lkn: false,
            hqx: 0,
            lko: false,
            eup: 0, 
            dsz: BTreeMap::new(),
            djj: MatrixProjection::azs(),
            visualizer: crate::visualizer::VisualizerState::new(),
            drone_swarm: crate::drone_swarm::DroneSwarmState::new(),
            
            drh: Vec::new(),
            erf: Vec::new(),
            erh: 0.0,
            ere: 0.0,
            erg: 0.0,
            eri: 0.0,
            ecd: 0.0,
            drg: 0.0,
            fju: 0.0,
            kze: 0.0,
            gij: Vec::new(),
            ixd: 0,
            fjt: 0,
            fjs: false,
            fwk: 0,
            bqa: Vec::new(),
            ari: None,
            hyr: String::new(),
            bij: String::new(),
            bsl: -1,
            hcz: None,
            
            eqn: BTreeMap::new(),
            avu: BTreeMap::new(),
            gjo: BTreeMap::new(),
            iuk: None,
            eaz: None,
            dvt: 0,
            efr: 0,
            eug: false,
            djb: String::new(),
            eeu: 0,
            gtw: 75,
            icm: 85,
            mje: true,
            
            jwu: 0,
            gws: 0,
            ddl: String::new(),
            ihf: String::new(),
            fyt: false,
            xum: false,
            fbj: None,
            
            kye: crate::gesture::GestureRecognizer::new(1280, 800),
            hle: crate::gesture::GestureBuffer::new(),
            pvl: false,
            
            ud: crate::mobile::MobileState::new(),
            
            hkf: 0,
            ivo: 0,
            cya: 0,
            swn: true,
            asr: DesktopTier::Bv,
            bqt: 0,
            cqt: 0,
            lel: DesktopTier::Bv,
            dcq: false,
            dwc: None,
            iak: false,
            
            mfr: false,
            mfs: 0,
            pkp: 0,
        }
    }
    
    
    pub fn init(&mut self, z: u32, ac: u32) {
        crate::serial_println!("[Desktop] init start: {}x{} (clearing {} windows, {} icons)", 
            z, ac, self.ee.len(), self.icons.len());
        
        
        
        self.ud = crate::mobile::MobileState::new();
        
        self.ee.clear();
        self.icons.clear();
        self.cxh.clear();
        self.djq.clear();
        self.enf.clear();
        self.eyq.clear();
        self.dra.clear();
        self.cwd.clear();
        #[cfg(feature = "emulators")]
        self.dtk.clear();
        #[cfg(feature = "emulators")]
        self.arf.clear();
        self.fdi.clear();
        self.dso.clear();
        self.ano.clear();
        
        self.browser = None;
        self.ado.clear();
        self.aef = 0;
        self.btn = false;
        self.cdj = false;
        
        self.xn.clear();
        self.ajo = false;
        self.bij.clear();
        self.btx = false;
        self.aka.iw = false;
        self.aka.pj.clear();
        self.aka.acm = 0;
        self.aka.fwe = None;
        self.aka.ejm = None;
        
        self.oo = 0;
        self.fwk = 0;
        self.bqa.clear();
        self.ari = None;
        self.hyr.clear();
        self.etz = 0;
        self.ety = false;
        self.etw = false;
        self.jcq = 0;
        self.gbz.clear();
        self.hcd.clear();
        
        *BBM_.lock() = 1;
        
        crate::serial_println!("[Desktop] state cleared, windows={} icons={}", 
            self.ee.len(), self.icons.len());
        
        self.z = z;
        self.ac = ac;
        self.lf = (z / 2) as i32;
        self.ot = (ac / 2) as i32;
        
        
        crate::graphics::scaling::init(z, ac);
        self.mcc = crate::graphics::scaling::ckv();
        crate::serial_println!("[Desktop] UI scale factor: {}x", self.mcc);
        
        
        crate::touch::init();
        crate::touch::dbw(z, ac);
        self.kye.dbw(z as i32, ac as i32);
        crate::serial_println!("[Desktop] Touch input initialized");
        
        
        crate::serial_println!("[Desktop] init_double_buffer...");
        framebuffer::beo();
        
        
        if framebuffer::nxr().is_some() {
            framebuffer::afi(true);
            crate::serial_println!("[Desktop] double buffer: OK");
        } else {
            framebuffer::afi(false);
            crate::serial_println!("[Desktop] WARNING: backbuffer alloc failed, using direct FB mode");
        }
        
        
        crate::serial_println!("[Desktop] init_background_cache...");
        framebuffer::tte();
        
        
        crate::serial_println!("[Desktop] init_compositor...");
        compositor::ttg(z, ac);
        compositor::pis(self.kkf);
        
        
        crate::serial_println!("[Desktop] init_desktop_icons...");
        self.tti();
        
        
        self.doh = false;
        self.bex = true;
        
        
        self.ttt();
        
        
        self.rww();
        
        
        
        crate::serial_println!("[Desktop] init complete (tier={:?})", self.asr);
    }
    
    
    fn ttt(&mut self) {
        
        const R_: usize = 256;
        const AGC_: usize = 4;
        const EL_: usize = 40;   
        const Bcx: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
        
        let es = R_ * AGC_;
        self.car = vec![0u8; es * EL_];
        self.awc = vec![0i32; es];
        self.czn = vec![2u32; es];
        self.fnr = vec![0u32; es];
        
        let ac = self.ac.ao(W_);
        
        for bj in 0..R_ {
            for fl in 0..AGC_ {
                let w = bj * AGC_ + fl;
                let dv = (bj as u32).hx(2654435761)
                    ^ 0xDEADBEEF
                    ^ ((fl as u32).hx(0x9E3779B9));
                for a in 0..EL_ {
                    let des = dv.cn((a as u32).hx(7919));
                    self.car[w * EL_ + a] = Bcx[(des as usize) % Bcx.len()];
                }
                
                let eyw = ac / 2 + (fl as u32) * ac / 6;
                self.awc[w] = -((dv % eyw.am(1)) as i32);
                self.czn[w] = 2 + (dv % 4);
                self.fnr[w] = dv;
            }
        }
        self.lkn = true;
        
        crate::drone_swarm::init(&mut self.drone_swarm, self.z, ac);
        
        
    }

    
    
    pub fn yeg(&mut self) {
        let lvw: u32 = 256;
        let lvv: u32 = 256;
        let wf = self.z;
        let aav = self.ac.ao(W_);
        let frh = (wf / 2).ao(lvw / 2);
        let fri = (aav / 2).ao(lvv / 2);
        let hz = MatrixProjection::tco(lvw, lvv);
        self.djj = MatrixProjection {
            b: frh,
            c: fri,
            z: lvw,
            ac: lvv,
            hz,
            gh: true,
        };
    }

    
    pub fn ylg(&mut self) {
        self.djj.gh = false;
    }

    
    
    
    
    
    const R_: usize = 256;
    const AZJ_: usize = 4;
    const AET_: usize = 40;

    
    pub fn mev(&mut self, akl: u8) {
        self.eup = akl.v(2);
        crate::serial_println!("[RAIN] Speed preset set to {}", ["slow", "mid", "fast"][self.eup as usize]);
    }

    
    #[inline]
    fn jfd(bj: usize, fl: usize, cim: usize) -> usize {
        (bj * Self::AZJ_ + fl) * Self::AET_ + cim
    }

    
    
    
    pub fn oll(&mut self, bj: usize, fl: usize, cim: usize, cell: CellPixels) -> &mut CellPixels {
        let bs = Self::jfd(bj, fl, cim);
        self.dsz.insert(bs, cell);
        self.dsz.ds(&bs).unwrap()
    }

    
    
    pub fn zch(&mut self, bj: usize, fl: usize, cim: usize, s: u32) -> &mut CellPixels {
        let w = bj * Self::AZJ_ + fl;
        let gcl = w * Self::AET_ + cim;
        let r = if gcl < self.car.len() {
            self.car[gcl] as char
        } else {
            '#'
        };
        let cell = CellPixels::sxv(r, s);
        self.oll(bj, fl, cim, cell)
    }

    
    pub fn zcg(&mut self, bj: usize, fl: usize, cim: usize) -> Option<&mut CellPixels> {
        let bs = Self::jfd(bj, fl, cim);
        self.dsz.ds(&bs)
    }

    
    pub fn zcd(&mut self, bj: usize, fl: usize, cim: usize, y: u8, x: u8, s: u32) {
        let bs = Self::jfd(bj, fl, cim);
        let cell = self.dsz.bt(bs).clq(CellPixels::mzj);
        cell.oj(y, x, s);
    }

    
    pub fn zce(&mut self, bj: usize, fl: usize, cim: usize) {
        let bs = Self::jfd(bj, fl, cim);
        self.dsz.remove(&bs);
    }

    
    pub fn zcf(&mut self) {
        self.dsz.clear();
    }

    
    
    
    
    pub fn zcc(&mut self, bii: usize, fl: usize, wtg: usize,
                              amn: &[u32], ars: usize, afv: usize) {
        
        let qxn = (ars + 7) / 8;
        let qxm = (afv + 15) / 16;
        
        for ae in 0..qxm {
            for cx in 0..qxn {
                let bj = bii + cx;
                let cim = wtg + ae;
                if bj >= Self::R_ || cim >= Self::AET_ { continue; }
                
                let mut cell = CellPixels::mzj();
                for x in 0..16u8 {
                    for y in 0..8u8 {
                        let blg = cx * 8 + y as usize;
                        let bih = ae * 16 + x as usize;
                        if blg < ars && bih < afv {
                            let s = amn[bih * ars + blg];
                            if s & 0xFF000000 != 0 {  
                                cell.oj(y, x, s);
                            }
                        }
                    }
                }
                self.oll(bj, fl, cim, cell);
            }
        }
    }

    
    fn zek(&mut self) {
        
        let mbn = r#"//! TrustOS — A Modern Operating System in Rust
//!
//! This file demonstrates TrustCode's syntax highlighting

use core::fmt;

/// Main kernel entry point
pub fn kernel_main() -> ! {
    let message = "Hello from TrustOS!";
    serial_println!("{}", message);

    // Initialize hardware
    let cpu_count: u32 = 4;
    let memory_mb: u64 = 256;

    for i in 0..cpu_count {
        init_cpu(i);
    }

    // Start the desktop environment
    let mut desktop = Desktop::new();
    desktop.init(1280, 800);

    loop {
        desktop.render();
        desktop.handle_input();
    }
}

/// Initialize a CPU core
fn init_cpu(id: u32) {
    // Setup GDT, IDT, APIC
    serial_println!("CPU {} initialized", id);
}

#[derive(Debug, Clone)]
struct AppConfig {
    name: String,
    version: (u8, u8, u8),
    features: Vec<&'static str>,
}
"#;
        
        let _ = crate::ramfs::fh(|fs| {
            fs.ns("/demo.rs", mbn.as_bytes())
        });
        
        let ad = self.xl("TrustCode: demo.rs", 160, 50, 780, 560, WindowType::Ag);
        if let Some(editor) = self.cxh.ds(&ad) {
            editor.dsu("demo.rs");
        }
        
        self.dhh(ad);
        crate::serial_println!("[TrustCode] Demo editor opened");
    }
    
    
    pub fn dvr(&mut self, ev: RenderMode) {
        self.che = ev;
        self.bex = true;
        self.doh = false;
        
        if ev == RenderMode::Ks {
            
            self.wxc();
        }
    }
    
    
    
    pub fn rww(&mut self) {
        let clx = crate::memory::fxc() / (1024 * 1024);
        let drq = crate::memory::heap::aez() / (1024 * 1024);
        let hz = (self.z as u64) * (self.ac as u64);
        let cdv = crate::cpu::smp::aao().am(1) as u64;
        
        
        let ifd = crate::cpu::mnh() / 1_000_000;
        
        
        
        
        
        
        
        let vpy = ((clx / 256) as i64).v(8);
        let rpw = if ifd > 0 { (ifd / 400) as i64 } else { 2 };
        let rpc = (cdv as i64) * 2;
        let vxo = ((hz as i64) - 1_000_000) / 1_000_000;
        let ol = vpy + rpw + rpc - vxo;
        
        
        
        
        
        let ngp = ifd > 0 && ifd < 1500 && cdv <= 1;
        
        let mks = if clx < 128 || drq < 8 {
            DesktopTier::Aap
        } else if ol <= 4 || clx < 256 {
            DesktopTier::Gy
        } else if ol <= 8 || clx < 512 || ngp {
            DesktopTier::Gc
        } else {
            DesktopTier::Bv
        };
        
        self.asr = mks;
        self.lel = mks;
        self.bqt = 0;
        self.cqt = 0;
        
        crate::serial_println!(
            "[Desktop] Tier={:?} (score={}, RAM={}MB, heap={}MB, CPUs={}, TSC={}MHz, {}x{}, cpu_limited={})",
            mks, ol, clx, drq, cdv, ifd, self.z, self.ac, ngp
        );
    }
    
    
    
    
    fn qli(&mut self) {
        
        if self.dcq { return; }
        
        
        if self.oo < 120 { return; }
        
        
        
        
        if self.cya < 40 {
            
            let tsq = if self.cya <= 2 { 60 } else if self.cya < 18 { 4 } else { 1 };
            self.bqt += tsq;
            self.cqt = 0;
        } else if self.cya >= 50 {
            
            self.cqt += 1;
            if self.bqt > 0 {
                self.bqt = self.bqt.ao(4);
            }
        } else {
            
            if self.bqt > 0 {
                self.bqt = self.bqt.ao(2);
            }
            self.cqt = 0;
        }
        
        
        if self.bqt >= 120 {
            let aft = self.asr;
            
            let clm = if self.cya <= 2 {
                match aft {
                    DesktopTier::Bv | DesktopTier::Gc => DesktopTier::Gy,
                    _ => aft,
                }
            } else {
                match aft {
                    DesktopTier::Bv => DesktopTier::Gc,
                    DesktopTier::Gc => DesktopTier::Gy,
                    _ => aft,
                }
            };
            if clm != aft {
                self.asr = clm;
                self.bqt = 0;
                self.cqt = 0;
                self.bex = true;
                self.doh = false;
                crate::serial_println!(
                    "[Desktop] Auto-downgrade: {:?} -> {:?} (FPS was {})",
                    aft, clm, self.cya
                );
            }
        }
        
        
        if self.cqt >= 300 {
            let aft = self.asr;
            let clm = match aft {
                DesktopTier::Gy => DesktopTier::Gc,
                DesktopTier::Gc => DesktopTier::Bv,
                _ => aft,
            };
            
            if clm != aft && clm <= self.lel {
                self.asr = clm;
                self.cqt = 0;
                self.bqt = 0;
                self.bex = true;
                self.doh = false;
                crate::serial_println!(
                    "[Desktop] Auto-upgrade: {:?} -> {:?} (FPS was {})",
                    aft, clm, self.cya
                );
            } else {
                self.cqt = 0;
            }
        }
    }
    
    
    pub fn bxb(&mut self, theme: CompositorTheme) {
        self.kkf = theme;
        compositor::pis(theme);
        self.bex = true;
    }
    
    
    fn wxc(&self) {
        let mut dfd = compositor::compositor();
        dfd.axa.clear();
        
        for bh in &self.ee {
            if bh.iw {
                let mut surface = WindowSurface::new(
                    bh.ad,
                    bh.b as f32,
                    bh.c as f32,
                    bh.z as f32,
                    bh.ac as f32,
                );
                surface.ell = 0;
                surface.ja = bh.ja;
                surface.iw = !bh.aat;
                dfd.axa.push(surface);
            }
        }
    }
    
    
    fn tti(&mut self) {
        use crate::icons::IconType;
        
        
        let crk = 50u32; 
        let vc = 12u32;
        let fg = 12u32;
        
        let kqm: &[(&str, IconType, IconAction)] = &[
            ("Terminal", IconType::Ay, IconAction::Tu),
            ("Files", IconType::Aig, IconAction::Aks),
            ("Editor", IconType::Ahq, IconAction::Bny),
            ("Calc", IconType::Calculator, IconAction::Bnw),
            ("NetScan", IconType::As, IconAction::Boe),
            ("Chess 3D", IconType::Gs, IconAction::Bnz),

            ("Browser", IconType::Browser, IconAction::Bnv),
            ("TrustEd", IconType::Fp, IconAction::Boc),
            ("Settings", IconType::Gn, IconAction::Akt),
            ("Music", IconType::Bmt, IconAction::Bod),
            #[cfg(feature = "emulators")]
            ("GameBoy", IconType::Aij, IconAction::Boa),
            #[cfg(feature = "emulators")]
            ("GameLab", IconType::Lm, IconAction::Bob),
        ];
        
        for (a, (j, ecz, hr)) in kqm.iter().cf() {
            self.icons.push(Aqv {
                j: String::from(*j),
                ecz: *ecz,
                b: fg,
                c: vc + a as u32 * crk,
                hr: *hr,
            });
        }
    }
    
    
    fn yhx(&self, b: i32, c: i32) -> Option<IconAction> {
        
        if b < 0 || b >= (BY_ + 10) as i32 { return None; }
        
        let cjz = self.ac.ao(W_);
        let eva = self.icons.len().am(1) as u32;
        let ob = 12u32;
        let bfz = cjz.ao(ob * 2);
        let crk = (bfz / eva) as i32;
        let vc = (ob + (bfz - crk as u32 * eva) / 2) as i32;
        
        for (a, pa) in self.icons.iter().cf() {
            let og = vc + a as i32 * crk;
            if c >= og - 3 && c < og + crk as i32 {
                return Some(pa.hr);
            }
        }
        None
    }
    
    
    pub fn xl(&mut self, dq: &str, b: i32, c: i32, z: u32, ac: u32, ash: WindowType) -> u32 {
        
        let xpg = self.z.ao(BY_ + 4);
        let xpf = self.ac.ao(W_ + J_);
        let d = z.v(xpg).am(120);
        let i = ac.v(xpf).am(80);
        
        let cso = BY_ as i32 + 2;
        let bvj = (self.z as i32 - d as i32).am(cso);
        let csl = (self.ac as i32 - W_ as i32 - i as i32).am(0);
        let cx = b.am(cso).v(bvj);
        let ae = c.am(0).v(csl);

        let mut bh = Window::new(dq, cx, ae, d, i, ash);
        
        
        match ash {
            WindowType::Ay => {
                bh.ca.push(String::from("\x01HTrustOS Terminal v1.0"));
                bh.ca.push(String::from("\x01MType \x01Ghelp\x01M for available commands."));
                bh.ca.push(String::from(""));
                bh.ca.push(Self::csi("_"));
            },
            WindowType::Qx => {
                bh.ca.push(String::from("=== System Information ==="));
                bh.ca.push(String::from(""));
                bh.ca.push(format!("OS: TrustOS v0.2.0"));
                bh.ca.push(format!("Arch: x86_64"));
                bh.ca.push(format!("Display: {}x{}", self.z, self.ac));
                bh.ca.push(String::from("Kernel: trustos_kernel"));
            },
            WindowType::Jf => {
                bh.ca.push(String::from("TrustOS"));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("A modern operating system"));
                bh.ca.push(String::from("written in Rust"));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("(c) 2026 Nathan"));
            },
            WindowType::Calculator => {
                self.enf.insert(bh.ad, CalculatorState::new());
            },
            WindowType::Ak => {
                bh.ca.push(String::from("=== File Manager ==="));
                bh.ca.push(String::from("Path: /"));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("  Name              Type       Size    Program"));
                bh.ca.push(String::from("  ────────────────────────────────────────────"));
                bh.wn = Some(String::from("/"));
                
                let mut nvh = FileManagerState::new();
                nvh.lwg("/");
                self.avu.insert(bh.ad, nvh);
                
                if let Ok(ch) = crate::ramfs::fh(|fs| fs.awb(Some("/"))) {
                    for (j, are, aw) in ch.iter().take(50) {
                        let pa = if *are == crate::ramfs::FileType::K { 
                            "[D]" 
                        } else { 
                            crate::file_assoc::iwq(j)
                        };
                        let ctl = if *are == crate::ramfs::FileType::K {
                            String::from("---")
                        } else {
                            String::from(crate::file_assoc::gih(j).j())
                        };
                        let kxm = if *are == crate::ramfs::FileType::K { "DIR" } else { "FILE" };
                        bh.ca.push(format!("  {} {:<14} {:<10} {:<7} {}", pa, j, kxm, aw, ctl));
                    }
                }
                if bh.ca.len() <= 5 {
                    bh.ca.push(String::from("  (empty directory)"));
                }
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("  [Enter] Open | [Up/Down] Navigate"));
            },
            WindowType::Ag => {
                
                
                let mut editor = EditorState::new();
                let ntn = self.cxh.len() + 1;
                let rvc = if ntn == 1 {
                    String::from("untitled.rs")
                } else {
                    alloc::format!("untitled_{}.rs", ntn)
                };
                editor.wn = Some(rvc);
                editor.eej = crate::apps::text_editor::Language::Rust;
                self.cxh.insert(bh.ad, editor);
            },
            WindowType::Hy => {
                
            },
            WindowType::Gn => {
                bh.ca.push(String::from("=== Settings ==="));
                bh.ca.push(String::from(""));
                bh.ca.push(format!("Resolution: {}x{}", self.z, self.ac));
                bh.ca.push(String::from("Theme: Dark Green"));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("--- Animations ---"));
                let gyq = if col() { "ON " } else { "OFF" };
                let kao = *GA_.lock();
                bh.ca.push(format!("[1] Animations: {}", gyq));
                bh.ca.push(format!("[2] Speed: {:.1}x", kao));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("--- Accessibility ---"));
                let bei = if crate::accessibility::edv() { "ON " } else { "OFF" };
                bh.ca.push(format!("[5] High Contrast: {}", bei));
                bh.ca.push(format!("[6] Font Size: {}", crate::accessibility::gid().cu()));
                bh.ca.push(format!("[7] Cursor Size: {}", crate::accessibility::gib().cu()));
                bh.ca.push(format!("[8] Sticky Keys: {}", if crate::accessibility::dsj() { "ON" } else { "OFF" }));
                bh.ca.push(format!("[9] Mouse Speed: {}", crate::accessibility::gig().cu()));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("--- Other ---"));
                bh.ca.push(String::from("[3] File Associations"));
                bh.ca.push(String::from("[4] About System"));
            },
            WindowType::Bp => {
                bh.ca.push(String::from("=== Image Viewer ==="));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("No image loaded"));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("Supported: PNG, JPG, BMP, GIF"));
            },
            WindowType::Is => {
                bh.ca.push(String::from("=== Hex Viewer ==="));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("No file loaded"));
            },
            WindowType::Aqu => {
                bh.ca.push(String::from("=== 3D Graphics Demo ==="));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("TrustOS Graphics Engine"));
                bh.ca.push(String::from("Software 3D Renderer"));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("Features:"));
                bh.ca.push(String::from("- Wireframe/Solid/Mixed modes"));
                bh.ca.push(String::from("- Z-buffer depth testing"));
                bh.ca.push(String::from("- Flat shading with lighting"));
                bh.ca.push(String::from("- Perspective projection"));
                bh.ca.push(String::from("- Backface culling"));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("[Rotating Cube Demo Below]"));
            },
            WindowType::Pj => {
                bh.ca.push(String::from("=== File Associations ==="));
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("Extension | Program       | Type"));
                bh.ca.push(String::from("----------|---------------|-------------"));
                
                let kbi = crate::file_assoc::ojn();
                for (wm, ctl, desc) in kbi.iter().take(15) {
                    bh.ca.push(format!(".{:<8} | {:<13} | {}", wm, ctl, desc));
                }
                bh.ca.push(String::from(""));
                bh.ca.push(String::from("Click extension to change program"));
            },
            WindowType::Browser => {
                
                if self.browser.is_none() {
                    self.browser = Some(crate::browser::Browser::new(z, ac));
                }
                self.ado = String::from("http://example.com");
                    self.aef = self.ado.len();
            },
            WindowType::Fp => {
                let g = crate::model_editor::ModelEditorState::new();
                self.djq.insert(bh.ad, g);
            },
            WindowType::Io => {
                self.eyq.insert(bh.ad, SnakeState::new());
            },
            WindowType::So => {
                self.dra.insert(bh.ad, crate::game3d::Game3DState::new());
            },
            WindowType::Gs => {
                self.dou.insert(bh.ad, crate::chess::ChessState::new());
            },
            WindowType::Ih => {
                self.cwd.insert(bh.ad, crate::chess3d::Chess3DState::new());
            },
            #[cfg(feature = "emulators")]
            WindowType::Xt => {
                let mut cw = crate::nes::NesEmulator::new();
                
                if let Some(maq) = crate::embedded_roms::usa() {
                    cw.ljk(maq);
                }
                self.dtk.insert(bh.ad, cw);
            },
            #[cfg(feature = "emulators")]
            WindowType::Sp => {
                let mut cw = crate::gameboy::GameBoyEmulator::new();
                
                if let Some(maq) = crate::embedded_roms::tag() {
                    cw.ljk(maq);
                }
                self.arf.insert(bh.ad, cw);
            },
            WindowType::Ro => {
                
            },
            WindowType::Td => {
                self.dso.insert(bh.ad, crate::lab_mode::LabState::new());
            },
            #[cfg(feature = "emulators")]
            WindowType::Lm => {
                self.azy.insert(bh.ad, crate::game_lab::GameLabState::new());
            },
            WindowType::Lw => {
                crate::serial_println!("[Desktop] Creating MusicPlayer state for window {}", bh.ad);
                let mut sn = MusicPlayerState::new();
                sn.ojz();
                self.ano.insert(bh.ad, sn);
                crate::serial_println!("[Desktop] MusicPlayer state created OK");
            },
            WindowType::Afs => {
                self.jwu = 0;
                self.gws = 0;
                self.fbj = None;
                let _ = crate::drivers::net::wifi::pod();
            },
            WindowType::Aft => {
                self.ddl.clear();
                self.fyt = false;
                self.fbj = None;
            },
            _ => {}
        }
        
        
        bh.qis();
        
        let ad = bh.ad;
        self.ee.push(bh);
        ad
    }
    
    
    pub fn iod(&mut self, ad: u32) {
        crate::serial_println!("[GUI] close_window({}) start", ad);
        if let Some(d) = self.ee.el().du(|d| d.ad == ad) {
            if d.qiq() {
                crate::serial_println!("[GUI] close_window({}) animate path", ad);
                
                
                #[cfg(feature = "emulators")]
                self.arf.remove(&ad);
                #[cfg(feature = "emulators")]
                self.dtk.remove(&ad);
                self.dra.remove(&ad);
                self.cwd.remove(&ad);
                #[cfg(feature = "emulators")]
                self.azy.remove(&ad);
                self.dso.remove(&ad);
                
                if let Some(sn) = self.ano.ds(&ad) {
                    crate::serial_println!("[GUI] close_window({}) stopping music...", ad);
                    sn.qg();
                    crate::serial_println!("[GUI] close_window({}) music stopped", ad);
                }
                crate::serial_println!("[GUI] close_window({}) removing mp state...", ad);
                self.ano.remove(&ad);
                crate::serial_println!("[GUI] close_window({}) animate path done", ad);
                return;
            }
        }
        crate::serial_println!("[GUI] close_window({}) immediate remove path", ad);
        
        self.ee.ajm(|d| d.ad != ad);
        
        self.cxh.remove(&ad);
        self.djq.remove(&ad);
        self.enf.remove(&ad);
        self.eyq.remove(&ad);
        self.dra.remove(&ad);
        self.dou.remove(&ad);
        self.cwd.remove(&ad);
        #[cfg(feature = "emulators")]
        self.dtk.remove(&ad);
        #[cfg(feature = "emulators")]
        self.arf.remove(&ad);
        self.fdi.remove(&ad);
        self.dso.remove(&ad);
        if let Some(sn) = self.ano.ds(&ad) {
            crate::serial_println!("[GUI] close_window({}) stopping music (imm)...", ad);
            sn.qg();
            crate::serial_println!("[GUI] close_window({}) music stopped (imm)", ad);
        }
        crate::serial_println!("[GUI] close_window({}) removing mp state (imm)...", ad);
        self.ano.remove(&ad);
        crate::serial_println!("[GUI] close_window({}) immediate path done", ad);
        #[cfg(feature = "emulators")]
        self.azy.remove(&ad);
        #[cfg(feature = "emulators")]
        self.ghu.remove(&ad);
    }
    
    
    pub fn llz(&mut self, ad: u32) {
        let ejr = (self.ac - W_) as i32;
        if let Some(d) = self.ee.el().du(|d| d.ad == ad) {
            if !d.aat {
                d.qir(ejr);
            }
            d.aat = !d.aat;
        }
    }
    
    
    pub fn xos(&mut self) {
        let mut cik = Vec::new();
        
        for d in &mut self.ee {
            if d.xor() {
                
                cik.push(d.ad);
            }
        }
        
        
        for ad in cik {
            self.ee.ajm(|d| d.ad != ad);
            self.cxh.remove(&ad);
            self.djq.remove(&ad);
            self.dra.remove(&ad);
            self.cwd.remove(&ad);
            #[cfg(feature = "emulators")]
            self.dtk.remove(&ad);
            #[cfg(feature = "emulators")]
            self.arf.remove(&ad);
            #[cfg(feature = "emulators")]
            self.azy.remove(&ad);
        }
    }
    
    
    pub fn dhh(&mut self, ad: u32) {
        for d in &mut self.ee {
            d.ja = false;
        }
        if let Some(w) = self.ee.iter().qf(|d| d.ad == ad) {
            let mut bh = self.ee.remove(w);
            bh.ja = true;
            bh.aat = false;
            self.ee.push(bh);
        }
    }
    
    
    
    
    
    
    pub fn anv(&self) -> u32 { self.z }
    pub fn akr(&self) -> u32 { self.ac }
    
    
    pub fn ndo(&mut self) {
        if let Some(ad) = self.ee.iter().vv().du(|d| d.ja).map(|d| d.ad) {
            self.iod(ad);
        }
    }
    
    
    pub fn uoo(&mut self) {
        if let Some(ad) = self.ee.iter().vv().du(|d| d.ja).map(|d| d.ad) {
            self.llz(ad);
        }
    }
    
    
    pub fn xiw(&mut self) {
        if let Some(ad) = self.ee.iter().vv().du(|d| d.ja).map(|d| d.ad) {
            let (kp, kl) = (self.z, self.ac);
            if let Some(d) = self.ee.el().du(|d| d.ad == ad) {
                d.idy(kp, kl);
            }
        }
    }
    
    
    pub fn mgh(&mut self, te: SnapDir) {
        if let Some(d) = self.ee.el().vv().du(|d| d.ja) {
            let mqu = self.ac.ao(W_);
            let cvi = BY_ as i32;
            let mqv = self.z.ao(BY_);
            let abd = mqv / 2;
            let wp = mqu / 2;
            
            match te {
                SnapDir::Ap => {
                    d.b = cvi;
                    d.c = 0;
                    d.z = abd;
                    d.ac = mqu;
                }
                SnapDir::Ca => {
                    d.b = cvi + abd as i32;
                    d.c = 0;
                    d.z = abd;
                    d.ac = mqu;
                }
                SnapDir::Dp => {
                    d.b = cvi;
                    d.c = 0;
                    d.z = abd;
                    d.ac = wp;
                }
                SnapDir::Dq => {
                    d.b = cvi + abd as i32;
                    d.c = 0;
                    d.z = abd;
                    d.ac = wp;
                }
                SnapDir::Dt => {
                    d.b = cvi;
                    d.c = wp as i32;
                    d.z = abd;
                    d.ac = wp;
                }
                SnapDir::Du => {
                    d.b = cvi + abd as i32;
                    d.c = wp as i32;
                    d.z = abd;
                    d.ac = wp;
                }
            }
            d.bkk = false;
        }
    }
    
    
    pub fn puc(&mut self) {
        
        let qgj = self.ee.iter().xx(|d| d.aat);
        
        
        for d in &mut self.ee {
            d.aat = !qgj;
        }
    }
    
    
    pub fn svi(&mut self, index: usize) {
        if index < self.ee.len() {
            
            let iw: Vec<u32> = self.ee.iter()
                .hi(|d| !d.aat)
                .map(|d| d.ad)
                .collect();
            
            if index < iw.len() {
                self.dhh(iw[index]);
            }
        }
    }
    
    
    pub fn yuk(&self) -> Vec<String> {
        self.ee.iter()
            .hi(|d| !d.aat)
            .map(|d| d.dq.clone())
            .collect()
    }
    
    
    pub fn tfe(&self) -> Vec<(String, WindowType)> {
        self.ee.iter()
            .hi(|d| !d.aat)
            .map(|d| (d.dq.clone(), d.ld.clone()))
            .collect()
    }
    
    
    pub fn zej(&mut self) {
        let ad = self.xl("Terminal", 100, 60, 780, 540, WindowType::Ay);
        self.dhh(ad);
    }

    
    pub fn ago(&mut self, b: i32, c: i32, vn: bool) {
        
        if self.ud.gh {
            let fp = self.ud.dxp;
            let iz = self.ud.ddi;
            let gm = self.ud.att as i32;
            let me = self.ud.azc as i32;
            
            if b >= fp && b < fp + gm && c >= iz && c < iz + me {
                let bhi = b - fp;
                let alk = c - iz;
                let ebi = if vn {
                    crate::mobile::GestureEvent::Btz(bhi, alk)
                } else {
                    crate::mobile::GestureEvent::Bua(bhi, alk)
                };
                let hr = crate::mobile::tjt(&mut self.ud, ebi);
                self.qjy(hr);
            }
            return;
        }
        
        
        if self.eug { return; }
        
        
        if !vn && self.eaz.is_some() {
            self.nuo(b, c);
            return;
        }
        if vn && self.eaz.is_some() {
            self.pxb(b, c);
        }
        
        if vn {
            
            if self.aka.iw {
                if let Some(hr) = self.qyq(b, c) {
                    self.sol(hr);
                }
                self.aka.iw = false;
                return;
            }
            
            
            crate::mouse::vtf();
            
            
            if self.ajo {
                if let Some(hr) = self.qzw(b, c) {
                    self.ajo = false;
                    self.bij.clear();
                    self.law(hr);
                    return;
                }
                
                if c < (self.ac - W_) as i32 || b >= 108 {
                    self.ajo = false;
                    self.bij.clear();
                    return;
                }
            }
            
            
            if c >= (self.ac - W_) as i32 {
                self.tlh(b, c);
                return;
            }
            
            
            for a in (0..self.ee.len()).vv() {
                if self.ee[a].contains(b, c) {
                    let ad = self.ee[a].ad;
                    
                    if self.ee[a].uxw(b, c) {
                        self.iod(ad);
                        return;
                    }
                    
                    if self.ee[a].uyg(b, c) {
                        let (kp, kl) = (self.z, self.ac);
                        if let Some(d) = self.ee.el().du(|d| d.ad == ad) {
                            d.idy(kp, kl);
                        }
                        return;
                    }
                    
                    if self.ee[a].uyh(b, c) {
                        self.llz(ad);
                        return;
                    }
                    
                    
                    
                    let lzq = self.ee[a].lqj(b, c);
                    let ogw = oh!(lzq, ResizeEdge::Jd | ResizeEdge::Dp | ResizeEdge::Dq);
                    if lzq != ResizeEdge::None && !ogw {
                        self.ee[a].dlg = lzq;
                        self.ee[a].dgp = b;
                        self.ee[a].dgq = c;
                        self.dhh(ad);
                        return;
                    }
                    
                    
                    if self.ee[a].odw(b, c) || ogw {
                        
                        if crate::mouse::jbf() {
                            crate::mouse::pcp();
                            let (kp, kl) = (self.z, self.ac);
                            if let Some(d) = self.ee.el().du(|d| d.ad == ad) {
                                d.idy(kp, kl);
                            }
                            return;
                        }
                        
                        let abx = self.ee[a].b;
                        let aha = self.ee[a].c;
                        self.ee[a].cka = true;
                        self.ee[a].dgp = b - abx;
                        self.ee[a].dgq = c - aha;
                    }
                    
                    
                    if self.ee[a].ld == WindowType::Browser {
                        crate::serial_println!("[CLICK-DBG] Browser window {} clicked at ({},{})", self.ee[a].ad, b, c);
                        let bx = self.ee[a].b;
                        let je = self.ee[a].c;
                        let nm = self.ee[a].z;
                        self.tja(b, c, bx, je, nm);
                    }
                    
                    
                    if self.ee[a].ld == WindowType::Ak {
                        let sve = self.ee[a].ad;
                        self.tjp(b, c, sve);
                    }
                    
                    
                    if self.ee[a].ld == WindowType::Fp {
                        let ep = &self.ee[a];
                        let fp = b - ep.b;
                        let iz = c - ep.c - J_ as i32;
                        let gm = ep.z as usize;
                        let me = ep.ac.ao(J_) as usize;
                        let nr = ep.ad;
                        if iz >= 0 {
                            if let Some(g) = self.djq.ds(&nr) {
                                g.ago(fp, iz, gm, me, true);
                            }
                        }
                    }
                    
                    
                    if self.ee[a].ld == WindowType::Gs {
                        let ep = &self.ee[a];
                        let avx = ep.b as i32 + 8;
                        let aui = ep.c as i32 + J_ as i32 + 4;
                        let bqu = ep.z.ao(16) as i32;
                        let ny: i32 = 48;
                        let aly = ny * 8;
                        let aoj = avx + (bqu - aly) / 2;
                        let apl = aui + 28;
                        
                        let bj = (b - aoj) / ny;
                        let br = (c - apl) / ny;
                        
                        if b >= aoj && b < aoj + aly && c >= apl && c < apl + aly && bj >= 0 && bj < 8 && br >= 0 && br < 8 {
                            let nr = ep.ad;
                            if let Some(chess) = self.dou.ds(&nr) {
                                chess.oai(bj, br);
                                chess.pxc(b, c);
                            }
                        }
                    }
                    
                    
                    if self.ee[a].ld == WindowType::Ih {
                        let ep = &self.ee[a];
                        let tc = ep.b as i32;
                        let gl = ep.c as i32 + J_ as i32;
                        let ur = ep.z as i32;
                        let nd = ep.ac.ao(J_) as i32;
                        let amr = b - tc;
                        let aio = c - gl;
                        if amr >= 0 && aio >= 0 && amr < ur && aio < nd {
                            let nr = ep.ad;
                            if let Some(g) = self.cwd.ds(&nr) {
                                g.ago(amr, aio, ur, nd);
                            }
                        }
                    }
                    
                    
                    if self.ee[a].ld == WindowType::Td {
                        let ep = &self.ee[a];
                        let amr = b - ep.b;
                        let aio = c - ep.c;
                        let nr = ep.ad;
                        let hk = ep.z;
                        let mg = ep.ac;
                        if let Some(abg) = self.dso.ds(&nr) {
                            abg.ago(amr, aio, hk, mg);
                        }
                    }

                    
                    if self.ee[a].ld == WindowType::Afs {
                        let ep = &self.ee[a];
                        let nr = ep.ad;
                        self.tlr(b, c, nr);
                    }

                    
                    if self.ee[a].ld == WindowType::Aft {
                        let ep = &self.ee[a];
                        let nr = ep.ad;
                        self.tls(b, c, nr);
                    }

                    
                    #[cfg(feature = "emulators")]
                    if self.ee[a].ld == WindowType::Sp {
                        let ep = &self.ee[a];
                        let tc = ep.b as u32;
                        let gl = (ep.c + J_ as i32) as u32;
                        let ur = ep.z;
                        let aje: u32 = 22;
                        let nr = ep.ad;
                        let abx = ep.b;
                        let aha = ep.c;
                        let aog = ep.z;
                        let biz = ep.ac;
                        let hl = b as u32;
                        let ir = c as u32;
                        
                        
                        if ir >= gl && ir < gl + aje {
                            
                            let fln: u32 = 48;
                            let edh = tc + ur - fln - 4;
                            if hl >= edh && hl < edh + fln {
                                
                                let tux = abx;
                                let tuy = aha + biz as i32 + 2;
                                let tuw = self.xl("GB Input", tux, tuy, aog.v(480), 160, WindowType::Abx);
                                self.ghu.insert(tuw, nr);
                            }
                            
                            
                            let fml: u32 = 32;
                            let fmm = edh - fml - 6;
                            if hl >= fmm && hl < fmm + fml {
                                
                                let kp = self.z;
                                let kl = self.ac;
                                let hpn = abx + aog as i32 + 4;
                                let lhq = (kp as i32 - hpn).am(400) as u32;
                                let lhp = kl - W_;
                                let ets = self.xl("Game Lab", hpn, 0, lhq, lhp, WindowType::Lm);
                                if let Some(abg) = self.azy.ds(&ets) {
                                    abg.fnb = Some(nr);
                                }
                                self.dhh(ets);
                            }
                        }
                    }

                    
                    #[cfg(feature = "emulators")]
                    if self.ee[a].ld == WindowType::Abx {
                        let ep = &self.ee[a];
                        let cx = ep.b as u32;
                        let ae = (ep.c + J_ as i32) as u32;
                        let dt = ep.z;
                        let bm = ep.ac.ao(J_);
                        let nr = ep.ad;
                        let hl = b as u32;
                        let ir = c as u32;
                        
                        let fnc = self.ghu.get(&nr).hu();
                        let cjk = crate::game_lab::tdt(cx, ae, dt, bm);
                        for &(bx, je, nm, adn, bs) in &cjk {
                            if hl >= bx && hl < bx + nm && ir >= je && ir < je + adn {
                                
                                let ebe = fnc.or_else(|| self.arf.cai().next().hu());
                                if let Some(dqn) = ebe {
                                    if let Some(cw) = self.arf.ds(&dqn) {
                                        cw.vr(bs);
                                    }
                                }
                                break;
                            }
                        }
                    }

                    
                    #[cfg(feature = "emulators")]
                    if self.ee[a].ld == WindowType::Lm {
                        let ep = &self.ee[a];
                        let amr = b - ep.b;
                        let aio = c - ep.c;
                        let nr = ep.ad;
                        let hk = ep.z;
                        let mg = ep.ac;
                        if let Some(abg) = self.azy.ds(&nr) {
                            
                            let jnm = hk as i32 - 120;
                            if aio >= J_ as i32 + 2 && aio < J_ as i32 + 18 {
                                if amr >= jnm && amr < jnm + 48 {
                                    
                                    let ebe = abg.fnb
                                        .or_else(|| self.arf.cai().next().hu());
                                    if let Some(dqn) = ebe {
                                        if let Some(cw) = self.arf.get(&dqn) {
                                            if let Some(fjr) = self.azy.ds(&nr) {
                                                fjr.wcv(cw);
                                                crate::serial_println!("[GameLab] State saved (click)");
                                            }
                                        }
                                    }
                                } else if amr >= jnm + 54 && amr < jnm + 102 {
                                    
                                    let ebe = abg.fnb
                                        .or_else(|| self.arf.cai().next().hu());
                                    if let Some(dqn) = ebe {
                                        let blq = self.azy.get(&nr)
                                            .map(|dm| dm.fto.blq).unwrap_or(false);
                                        if blq {
                                            if let Some(cw) = self.arf.ds(&dqn) {
                                                if let Some(fjr) = self.azy.get(&nr) {
                                                    fjr.uhb(cw);
                                                    crate::serial_println!("[GameLab] State loaded (click)");
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                abg.ago(amr, aio, hk, mg);
                            }
                        }
                    }

                    
                    if self.ee[a].ld == WindowType::Gn {
                        let ep = &self.ee[a];
                        let fx = ep.b;
                        let lw = ep.c;
                        let gl = lw + J_ as i32;
                        let aiq = 140i32;
                        let ali = 32i32;
                        
                        
                        if b >= fx && b < fx + aiq && c >= gl + 8 {
                            let w = ((c - gl - 8) / ali) as u8;
                            if w <= 7 {
                                self.dvt = w;
                            }
                        }
                        
                        
                        if self.dvt == 0 && b >= fx + aiq {
                            let y = fx + aiq + 20;
                            let gy = 22i32;
                            
                            
                            
                            
                            
                            let dec = gl + 16;
                            let lmi = dec + (gy + 8) + gy + (gy + 8) + gy + (gy + 8) + gy + gy;
                            let otk = lmi + gy;
                            
                            
                            if c >= lmi && c < lmi + gy {
                                let clm = match self.asr {
                                    DesktopTier::Bv => DesktopTier::Gc,
                                    DesktopTier::Gc => DesktopTier::Gy,
                                    DesktopTier::Gy | DesktopTier::Aap => DesktopTier::Bv,
                                };
                                self.asr = clm;
                                self.dcq = true;
                                self.bqt = 0;
                                self.cqt = 0;
                                self.bex = true;
                                self.doh = false;
                                crate::serial_println!("[Desktop] Manual tier change (click): {:?}", clm);
                            }
                            
                            if c >= otk && c < otk + gy {
                                self.dcq = !self.dcq;
                                self.bqt = 0;
                                self.cqt = 0;
                                crate::serial_println!("[Desktop] Manual override toggle (click): {}", self.dcq);
                            }
                        }
                    }
                    
                    
                    if self.ee[a].ld == WindowType::Ro {
                        let ep = &self.ee[a];
                        let amr = b - ep.b;
                        let aio = c - ep.c;
                        let nr = ep.ad;
                        let hk = ep.z;
                        let mg = ep.ac;
                        if let Some(cve) = self.fdi.ds(&nr) {
                            cve.ago(amr, aio, hk, mg);
                        }
                    }
                    
                    
                    if self.ee[a].ld == WindowType::Calculator {
                        let ep = &self.ee[a];
                        let rsn = ep.b as u32 + 4;
                        let rsq = ep.c as u32 + J_ + 4;
                        let dt = ep.z.ao(8);
                        let bm = ep.ac.ao(J_ + 8);
                        let cjy = 56u32;
                        let imf = rsq + cjy + 12;
                        let hbl = 4u32;
                        let hbm = 5u32;
                        let aib = 4u32;
                        let pm = (dt - 12 - aib * (hbl - 1)) / hbl;
                        let qx = ((bm - cjy - 20 - aib * (hbm - 1)) / hbm).v(40);
                        
                        let agi = b as u32;
                        let bbf = c as u32;
                        
                        if bbf >= imf {
                            let cjk = [
                                ['C', '(', ')', '%'],
                                ['7', '8', '9', '/'],
                                ['4', '5', '6', '*'],
                                ['1', '2', '3', '-'],
                                ['0', '.', '=', '+'],
                            ];
                            
                            for (br, kfd) in cjk.iter().cf() {
                                for (bj, &cu) in kfd.iter().cf() {
                                    let bx = rsn + 4 + bj as u32 * (pm + aib);
                                    let je = imf + br as u32 * (qx + aib);
                                    
                                    if agi >= bx && agi < bx + pm && bbf >= je && bbf < je + qx {
                                        let nr = ep.ad;
                                        if let Some(akz) = self.enf.ds(&nr) {
                                            match cu {
                                                '0'..='9' => akz.oxb(cu),
                                                '.' => akz.oxc(),
                                                '+' => akz.duq('+'),
                                                '-' => akz.duq('-'),
                                                '*' => akz.duq('*'),
                                                '/' => akz.duq('/'),
                                                '%' => akz.duq('%'),
                                                '=' => akz.oxd(),
                                                'C' => akz.oxa(),
                                                '(' => akz.jju('('),
                                                ')' => akz.jju(')'),
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    
                    if self.ee[a].ld == WindowType::Lw {
                        let ep = &self.ee[a];
                        let fx = ep.b as u32;
                        let lw = ep.c as u32 + J_;
                        let hk = ep.z;
                        let ov = 10u32;
                        let yz = fx + ov;
                        let aii = hk.ao(ov * 2);

                        let agi = b as u32;
                        let bbf = c as u32;
                        let nr = ep.ad;

                        
                        let alm = self.ano.get(&nr)
                            .map(|sn| sn.alm).unwrap_or(0);
                        let hpz = lw + 6;
                        let ou = hpz + 16;
                        let ayf = 5usize;
                        let ph = 20u32;
                        let bae = if alm == 0 { ph } else { (alm.v(ayf) as u32) * ph };

                        let jhk = ou + bae + 10;
                        let iaz = jhk + 16;
                        let uo = iaz + 16;
                        let ctm = uo + 18;
                        let dxm = ctm + 12;
                        let ekq = 60u32;
                        let fdb = dxm + ekq + 4;
                        let tn = 14u32;
                        let cdw = fdb + tn + 8;
                        let qx = 28u32;
                        let cuk = 36u32;
                        let fra = 64u32;
                        let qi = 4u32;

                        
                        if alm > 0
                            && agi >= yz && agi < yz + aii
                            && bbf >= ou && bbf < ou + bae
                        {
                            let jc = self.ano.get(&nr)
                                .map(|sn| sn.mmq.v(sn.alm.ao(ayf)))
                                .unwrap_or(0);
                            let bwv = ((bbf - ou) / ph) as usize;
                            let zx = jc + bwv;
                            if zx < alm {
                                crate::serial_println!("[MUSIC] Track list click: track {}", zx);
                                if let Some(sn) = self.ano.ds(&nr) {
                                    sn.dkp(zx);
                                }
                                self.bqt = 0; 
                            }
                        }

                        
                        let mmo = cuk * 3 + fra + qi * 3;
                        let mmy = yz + (aii.ao(mmo)) / 2;
                        if bbf >= cdw && bbf < cdw + qx {
                            
                            let bwb = mmy;
                            if agi >= bwb && agi < bwb + cuk {
                                if let Some(sn) = self.ano.ds(&nr) {
                                    sn.oxm();
                                }
                            }
                            
                            let gpk = bwb + cuk + qi;
                            if agi >= gpk && agi < gpk + fra {
                                if let Some(sn) = self.ano.ds(&nr) {
                                    match sn.g {
                                        PlaybackState::Af => {
                                            sn.dkp(sn.dfl);
                                            self.bqt = 0;
                                        },
                                        PlaybackState::Ce | PlaybackState::Cl => sn.mlq(),
                                    }
                                }
                            }
                            
                            let gti = gpk + fra + qi;
                            if agi >= gti && agi < gti + cuk {
                                if let Some(sn) = self.ano.ds(&nr) {
                                    sn.qg();
                                }
                            }
                            
                            let jhb = gti + cuk + qi;
                            if agi >= jhb && agi < jhb + cuk {
                                if let Some(sn) = self.ano.ds(&nr) {
                                    sn.loq();
                                }
                            }
                        }

                        
                        if agi >= yz && agi < yz + aii
                            && bbf >= ctm.ao(3) && bbf < ctm + 8 {
                            if let Some(sn) = self.ano.ds(&nr) {
                                if sn.alu > 0 && sn.g != PlaybackState::Af {
                                    let adj = (agi - yz) as f32 / aii.am(1) as f32;
                                    let utc = (adj * sn.alu as f32) as u64;
                                    sn.wgm(utc);
                                }
                            }
                        }

                        
                        let ccl = cdw + qx + 8;
                        let igt = 10u32;
                        let mpw = yz + 30;
                        let pyt = aii.ao(72);
                        if agi >= mpw && agi < mpw + pyt
                            && bbf >= ccl.ao(4) && bbf < ccl + igt + 4 {
                            let adj = (agi - mpw) as f32 / pyt.am(1) as f32;
                            let oqh = (adj * 100.0).am(0.0).v(100.0) as u32;
                            if let Some(sn) = self.ano.ds(&nr) {
                                sn.hq = oqh;
                                let _ = crate::drivers::hda::chv(oqh.v(100) as u8);
                            }
                        }

                        
                        let iwg = ccl + igt + 10;
                        let iwf = iwg + 4;
                        let azx = 24u32;
                        let aju = 24u32;
                        let caj = 36u32;
                        let cyc = yz + caj + 4;

                        
                        let dmh = iwf + 16;
                        if bbf >= dmh && bbf < dmh + azx {
                            
                            if agi >= cyc && agi < cyc + aju {
                                if let Some(sn) = self.ano.ds(&nr) {
                                    sn.emk = (sn.emk - 10).am(-500);
                                }
                            }
                            let gtv = cyc + aju + 4 + 52 + 4;
                            
                            if agi >= gtv && agi < gtv + aju {
                                if let Some(sn) = self.ano.ds(&nr) {
                                    sn.emk = (sn.emk + 10).v(500);
                                }
                            }
                            
                            let jrz = gtv + aju + 4;
                            if agi >= jrz && agi < jrz + aju {
                                if let Some(sn) = self.ano.ds(&nr) {
                                    sn.emk = 0;
                                }
                            }
                        }

                        
                        let dxn = dmh + azx + 4;
                        if bbf >= dxn && bbf < dxn + azx {
                            let gwf = aii.ao(caj + 4 + aju * 2 + 12);
                            
                            if agi >= cyc && agi < cyc + aju {
                                let ef = self.visualizer.ev;
                                self.visualizer.ev = if ef == 0 { crate::visualizer::IR_ - 1 } else { ef - 1 };
                            }
                            
                            let jvv = cyc + aju + 4 + gwf + 4;
                            if agi >= jvv && agi < jvv + aju {
                                self.visualizer.ev = (self.visualizer.ev + 1) % crate::visualizer::IR_;
                            }
                        }

                        
                        let dud = dxn + azx + 4;
                        if bbf >= dud && bbf < dud + azx {
                            let gos = aii.ao(caj + 4 + aju * 2 + 12);
                            
                            if agi >= cyc && agi < cyc + aju {
                                let ai = self.visualizer.aim;
                                self.visualizer.aim = if ai == 0 { crate::visualizer::AGE_ - 1 } else { ai - 1 };
                            }
                            
                            let jim = cyc + aju + 4 + gos + 4;
                            if agi >= jim && agi < jim + aju {
                                self.visualizer.aim = (self.visualizer.aim + 1) % crate::visualizer::AGE_;
                            }
                        }

                        
                        let duv = dud + azx + 4;
                        if bbf >= duv && bbf < duv + azx {
                            let hwn = aii.ao(caj + 4 + aju * 2 + 12);
                            
                            if agi >= cyc && agi < cyc + aju {
                                let ai = self.eup;
                                self.mev(if ai == 0 { 2 } else { ai - 1 });
                            }
                            
                            let jlb = cyc + aju + 4 + hwn + 4;
                            if agi >= jlb && agi < jlb + aju {
                                self.mev((self.eup + 1) % 3);
                            }
                        }
                    }
                    
                    self.dhh(ad);
                    return;
                }
            }
            
            
            if let Some(w) = self.ncp(b, c) {
                let hr = self.icons[w].hr;
                self.oag(hr);
                return;
            }
            
            self.ajo = false;
            self.bij.clear();
        } else {
            
            let mgg = self.dwc.take();
            let mut wqf: Option<u32> = None;
            for d in &mut self.ee {
                if d.cka {
                    if let Some(te) = mgg {
                        
                        let bxw = self.ac.ao(W_);
                        let cvi = BY_ as i32;
                        let mqv = self.z.ao(BY_);
                        let abd = mqv / 2;
                        let wp = bxw / 2;
                        match te {
                            SnapDir::Ap => { d.b = cvi; d.c = 0; d.z = abd; d.ac = bxw; }
                            SnapDir::Ca => { d.b = cvi + abd as i32; d.c = 0; d.z = abd; d.ac = bxw; }
                            SnapDir::Dp => { d.b = cvi; d.c = 0; d.z = abd; d.ac = wp; }
                            SnapDir::Dq => { d.b = cvi + abd as i32; d.c = 0; d.z = abd; d.ac = wp; }
                            SnapDir::Dt => { d.b = cvi; d.c = wp as i32; d.z = abd; d.ac = wp; }
                            SnapDir::Du => { d.b = cvi + abd as i32; d.c = wp as i32; d.z = abd; d.ac = wp; }
                        }
                        d.bkk = false;
                        wqf = Some(d.ad);
                    }
                }
                d.cka = false;
                d.dlg = ResizeEdge::None;
            }
            
            
            if self.eaz.is_some() {
                self.nuo(b, c);
            }
            
            
            let upg: Vec<u32> = self.ee.iter()
                .hi(|d| d.ld == WindowType::Fp && d.ja)
                .map(|d| d.ad)
                .collect();
            for ad in upg {
                if let Some(g) = self.djq.ds(&ad) {
                    g.ago(0, 0, 0, 0, false);
                }
            }
            
            
            let khj: Vec<u32> = self.ee.iter()
                .hi(|d| d.ld == WindowType::Gs && d.ja)
                .map(|d| d.ad)
                .collect();
            for ad in khj {
                if let Some(chess) = self.dou.ds(&ad) {
                    if chess.dgo.is_some() {
                        
                        if let Some(ep) = self.ee.iter().du(|d| d.ad == ad) {
                            let avx = ep.b as i32 + 8;
                            let aui = ep.c as i32 + J_ as i32 + 4;
                            let bqu = ep.z.ao(16) as i32;
                            let ny: i32 = 48;
                            let aly = ny * 8;
                            let aoj = avx + (bqu - aly) / 2;
                            let apl = aui + 28;
                            
                            let bj = (b - aoj) / ny;
                            let br = (c - apl) / ny;
                            chess.lay(bj, br);
                        }
                    }
                }
            }
            
            
            let rad: Vec<u32> = self.ee.iter()
                .hi(|d| d.ld == WindowType::Ih && d.ja)
                .map(|d| d.ad)
                .collect();
            for ad in rad {
                if let Some(g) = self.cwd.ds(&ad) {
                    g.lay();
                }
            }
            
            
            #[cfg(feature = "emulators")]
            {
            let tva: Vec<(u32, Option<u32>)> = self.ee.iter()
                .hi(|d| d.ld == WindowType::Abx && d.ja)
                .map(|d| (d.ad, self.ghu.get(&d.ad).hu()))
                .collect();
            for (xzy, fnc) in tva {
                let ebe = fnc.or_else(|| self.arf.cai().next().hu());
                if let Some(dqn) = ebe {
                    if let Some(cw) = self.arf.ds(&dqn) {
                        cw.avy(b'w');
                        cw.avy(b'a');
                        cw.avy(b's');
                        cw.avy(b'd');
                        cw.avy(b'x');
                        cw.avy(b'z');
                        cw.avy(b'c');
                        cw.avy(b'\r');
                    }
                }
            }
            }
        }
    }
    
    
    pub fn hmk(&mut self, b: i32, c: i32, vn: bool) {
        if !vn {
            return; 
        }
        
        
        self.aka.iw = false;
        self.ajo = false;
        self.bij.clear();
        
        
        if let Some(svf) = self.ee.iter().du(|d| {
            d.ld == WindowType::Ak
            && b >= d.b && b < d.b + d.z as i32
            && c >= d.c + J_ as i32 + 36 + 1 + 24 
            && c < d.c + d.ac as i32
        }).map(|d| (d.ad, d.b, d.c, d.z, d.ac, d.wn.clone(), d.acm, d.ca.len())) {
            let (ajq, fx, lw, hk, qec, kvs, mdj, byy) = svf;
            let aiq = self.avu.get(&ajq).map(|bb| if bb.ian { 0i32 } else { bb.iao as i32 }).unwrap_or(180);
            
            
            if b >= fx + aiq {
                
                let gl = lw + J_ as i32;
                let asj = gl + 36 + 1;
                let lix = asj + 24 + 1;
                let ph = 26i32;
                let asu = 5usize.v(byy);
                let bec = if byy > asu + 2 { byy - asu - 2 } else { 0 };
                let aio = c - lix;
                let jc = self.ee.iter().du(|d| d.ad == ajq).map(|d| d.px).unwrap_or(0);
                
                let cwf = if aio >= 0 { Some(jc + (aio / ph) as usize) } else { None };
                let osl = cwf.map(|a| a < bec).unwrap_or(false);
                
                
                if let Some(w) = cwf {
                    if w < bec {
                        if let Some(d) = self.ee.el().du(|d| d.ad == ajq) {
                            d.acm = w;
                        }
                    }
                }
                
                
                let ejm = if osl {
                    if let Some(d) = self.ee.iter().du(|d| d.ad == ajq) {
                        let baw = asu + cwf.unwrap_or(0);
                        if baw < d.ca.len().ao(2) {
                            let line = &d.ca[baw];
                            let j = Self::cxp(line);
                            if j != ".." { Some(String::from(j)) } else { None }
                        } else { None }
                    } else { None }
                } else { None };
                
                if osl && ejm.is_some() {
                    
                    self.aka = Wb {
                        iw: true,
                        b, c,
                        pj: alloc::vec![
                            An { cu: String::from("  Open          Enter"), hr: ContextAction::Ck },
                            An { cu: String::from("  Open With..."), hr: ContextAction::Awl },
                            An { cu: String::from("─────────────────────"), hr: ContextAction::Hl },
                            An { cu: String::from("  Cut          Ctrl+X"), hr: ContextAction::Aql },
                            An { cu: String::from("  Copy         Ctrl+C"), hr: ContextAction::Copy },
                            An { cu: String::from("  Copy Path"), hr: ContextAction::Bdt },
                            An { cu: String::from("─────────────────────"), hr: ContextAction::Hl },
                            An { cu: String::from("  Rename            F2"), hr: ContextAction::Axv },
                            An { cu: String::from("  Delete           Del"), hr: ContextAction::Jj },
                            An { cu: String::from("─────────────────────"), hr: ContextAction::Hl },
                            An { cu: String::from("  Properties"), hr: ContextAction::Ady },
                        ],
                        acm: 0,
                        fwe: None,
                        ejm,
                    };
                } else {
                    
                    self.aka = Wb {
                        iw: true,
                        b, c,
                        pj: alloc::vec![
                            An { cu: String::from("  New File         N"), hr: ContextAction::Awa },
                            An { cu: String::from("  New Folder       D"), hr: ContextAction::Bnh },
                            An { cu: String::from("─────────────────────"), hr: ContextAction::Hl },
                            An { cu: String::from("  Paste        Ctrl+V"), hr: ContextAction::Awr },
                            An { cu: String::from("─────────────────────"), hr: ContextAction::Hl },
                            An { cu: String::from("  Sort by Name"), hr: ContextAction::Azb },
                            An { cu: String::from("  Sort by Size"), hr: ContextAction::Btf },
                            An { cu: String::from("─────────────────────"), hr: ContextAction::Hl },
                            An { cu: String::from("  Refresh          F5"), hr: ContextAction::Axs },
                            An { cu: String::from("  Open in Terminal"), hr: ContextAction::Azu },
                            An { cu: String::from("  Properties"), hr: ContextAction::Ady },
                        ],
                        acm: 0,
                        fwe: None,
                        ejm: kvs,
                    };
                }
                return;
            }
        }
        
        
        if let Some(w) = self.ncp(b, c) {
            self.wni(b, c, w);
            return;
        }
        
        
        if c < (self.ac - W_) as i32 {
            self.wng(b, c);
        }
    }
    
    
    fn wni(&mut self, b: i32, c: i32, trf: usize) {
        self.aka = Wb {
            iw: true,
            b,
            c,
            pj: alloc::vec![
                An { cu: String::from("  Open          Enter"), hr: ContextAction::Ck },
                An { cu: String::from("  Open With..."), hr: ContextAction::Awl },
                An { cu: String::from("─────────────────────"), hr: ContextAction::Hl },
                An { cu: String::from("  Cut          Ctrl+X"), hr: ContextAction::Aql },
                An { cu: String::from("  Copy         Ctrl+C"), hr: ContextAction::Copy },
                An { cu: String::from("─────────────────────"), hr: ContextAction::Hl },
                An { cu: String::from("  Rename            F2"), hr: ContextAction::Axv },
                An { cu: String::from("  Delete           Del"), hr: ContextAction::Jj },
                An { cu: String::from("─────────────────────"), hr: ContextAction::Hl },
                An { cu: String::from("  Properties"), hr: ContextAction::Ady },
            ],
            acm: 0,
            fwe: Some(trf),
            ejm: None,
        };
    }
    
    
    fn wng(&mut self, b: i32, c: i32) {
        self.aka = Wb {
            iw: true,
            b,
            c,
            pj: alloc::vec![
                An { cu: String::from("  View              >"), hr: ContextAction::Bvt },
                An { cu: String::from("  Sort by           >"), hr: ContextAction::Azb },
                An { cu: String::from("  Refresh          F5"), hr: ContextAction::Axs },
                An { cu: String::from("─────────────────────"), hr: ContextAction::Hl },
                An { cu: String::from("  Paste        Ctrl+V"), hr: ContextAction::Awr },
                An { cu: String::from("─────────────────────"), hr: ContextAction::Hl },
                An { cu: String::from("  New               >"), hr: ContextAction::Awa },
                An { cu: String::from("─────────────────────"), hr: ContextAction::Hl },
                An { cu: String::from("  Open in Terminal"), hr: ContextAction::Azu },
                An { cu: String::from("  Personalize"), hr: ContextAction::Bow },
                An { cu: String::from("  Properties"), hr: ContextAction::Ady },
            ],
            acm: 0,
            fwe: None,
            ejm: None,
        };
    }
    
    
    fn qyq(&self, b: i32, c: i32) -> Option<ContextAction> {
        if !self.aka.iw {
            return None;
        }
        
        let rs = self.aka.b;
        let xp = self.aka.c;
        let djn = 150;
        let crv = 22;
        let gmo = self.aka.pj.len() as i32 * crv;
        
        if b >= rs && b < rs + djn && c >= xp && c < xp + gmo {
            let w = ((c - xp) / crv) as usize;
            if w < self.aka.pj.len() {
                return Some(self.aka.pj[w].hr);
            }
        }
        
        None
    }
    
    
    fn sol(&mut self, hr: ContextAction) {
        let l = (self.ee.len() as i32 * 25) % 200;
        
        
        let nvi = self.aka.ejm.clone();
        let fiw = self.aka.fwe;
        
        
        let cza = nvi.is_some() && fiw.is_none();
        
        match hr {
            ContextAction::Ck => {
                if cza {
                    
                    if let Some(bh) = self.ee.iter().du(|d| d.ja && d.ld == WindowType::Ak) {
                        let asu = 5usize.v(bh.ca.len());
                        let baw = asu + bh.acm;
                        if baw < bh.ca.len().ao(2) {
                            let line = &bh.ca[baw];
                            let ta = line.contains("[D]");
                            let j = String::from(Self::cxp(line));
                            if ta {
                                self.jgm(&j);
                            } else {
                                self.gol(&j);
                            }
                        }
                    }
                } else if let Some(w) = fiw {
                    let tra = self.icons[w].hr;
                    self.oag(tra);
                }
            },
            ContextAction::Awl => {
                self.xl("Open With", 300 + l, 200 + l, 400, 300, WindowType::Pj);
            },
            ContextAction::Axs => {
                if cza {
                    if let Some(path) = nvi {
                        self.brz(&path);
                    }
                }
                crate::serial_println!("[GUI] Refreshed");
            },
            ContextAction::Awa => {
                if cza {
                    let rp = self.ee.iter()
                        .du(|d| d.ja && d.ld == WindowType::Ak)
                        .and_then(|d| d.wn.clone())
                        .unwrap_or_else(|| String::from("/"));
                    let j = format!("new_file_{}.txt", self.oo % 1000);
                    let wo = if rp == "/" { format!("/{}", j) } else { format!("{}/{}", rp, j) };
                    let _ = crate::ramfs::fh(|fs| fs.touch(&wo));
                    crate::serial_println!("[FM] Created file: {}", wo);
                    self.brz(&rp);
                } else {
                    let it = format!("/desktop/newfile_{}.txt", self.oo);
                    crate::ramfs::fh(|fs| { let _ = fs.ns(&it, b"New file created from desktop"); });
                }
            },
            ContextAction::Bnh => {
                if cza {
                    let rp = self.ee.iter()
                        .du(|d| d.ja && d.ld == WindowType::Ak)
                        .and_then(|d| d.wn.clone())
                        .unwrap_or_else(|| String::from("/"));
                    let j = format!("folder_{}", self.oo % 1000);
                    let wo = if rp == "/" { format!("/{}", j) } else { format!("{}/{}", rp, j) };
                    let _ = crate::ramfs::fh(|fs| fs.ut(&wo));
                    crate::serial_println!("[FM] Created folder: {}", wo);
                    self.brz(&rp);
                } else {
                    let fgu = format!("/desktop/folder_{}", self.oo);
                    crate::ramfs::fh(|fs| { let _ = fs.ut(&fgu); });
                }
            },
            ContextAction::Ady => {
                let (d, i) = (self.z, self.ac);
                let xun = self.ee.len();
                let tre = self.icons.len();
                let nr = self.xl("Properties", 350 + l, 250 + l, 320, 220, WindowType::Jf);
                if let Some(bh) = self.ee.el().du(|xuz| xuz.ad == nr) {
                    bh.ca.clear();
                    bh.ca.push(String::from("═══════ System Properties ═══════"));
                    bh.ca.push(String::new());
                    bh.ca.push(format!("Display: {}x{}", d, i));
                    bh.ca.push(format!("Windows open: {}", xun + 1));
                    bh.ca.push(format!("Desktop icons: {}", tre));
                    bh.ca.push(String::new());
                    bh.ca.push(String::from("Theme: GitHub Dark"));
                    bh.ca.push(String::from("OS: TrustOS v0.9.4"));
                }
            },
            ContextAction::Aql => {
                if cza {
                    self.iul(true);
                } else if let Some(w) = fiw {
                    self.hcz = Some((w, true));
                    let j = self.icons[w].j.clone();
                    crate::keyboard::eno(&j);
                }
            },
            ContextAction::Copy => {
                if cza {
                    self.iul(false);
                } else if let Some(w) = fiw {
                    self.hcz = Some((w, false));
                    let j = self.icons[w].j.clone();
                    crate::keyboard::eno(&j);
                }
            },
            ContextAction::Awr => {
                if cza {
                    self.ntj();
                } else if let Some((blf, jbe)) = self.hcz.take() {
                    if blf < self.icons.len() {
                        if !jbe {
                            let cy = self.icons[blf].clone();
                            let gno = format!("{} (copy)", cy.j);
                            let usv = Aqv {
                                j: gno.clone(),
                                ecz: cy.ecz,
                                b: cy.b + 10,
                                c: cy.c + 10,
                                hr: cy.hr,
                            };
                            self.icons.push(usv);
                        }
                    }
                }
            },
            ContextAction::Bdt => {
                if cza {
                    if let Some(bh) = self.ee.iter().du(|d| d.ja && d.ld == WindowType::Ak) {
                        let rp = bh.wn.clone().unwrap_or_else(|| String::from("/"));
                        let asu = 5usize.v(bh.ca.len());
                        let baw = asu + bh.acm;
                        if baw < bh.ca.len().ao(2) {
                            let j = Self::cxp(&bh.ca[baw]);
                            let auh = if rp == "/" { format!("/{}", j) } else { format!("{}/{}", rp, j) };
                            crate::keyboard::eno(&auh);
                            crate::serial_println!("[FM] Copied path: {}", auh);
                        }
                    }
                } else if let Some(w) = fiw {
                    if w < self.icons.len() {
                        let path = format!("/desktop/{}", self.icons[w].j);
                        crate::keyboard::eno(&path);
                    }
                }
            },
            ContextAction::Jj => {
                if cza {
                    if let Some(bh) = self.ee.iter().du(|d| d.ja && d.ld == WindowType::Ak) {
                        let rp = bh.wn.clone().unwrap_or_else(|| String::from("/"));
                        let asu = 5usize.v(bh.ca.len());
                        let baw = asu + bh.acm;
                        if baw < bh.ca.len().ao(2) {
                            let j = String::from(Self::cxp(&bh.ca[baw]));
                            if j != ".." {
                                let wo = if rp == "/" { format!("/{}", j) } else { format!("{}/{}", rp, j) };
                                let _ = crate::ramfs::fh(|fs| fs.hb(&wo));
                                crate::serial_println!("[FM] Deleted: {}", wo);
                            }
                        }
                        let bza = rp.clone();
                        drop(bh);
                        self.brz(&bza);
                    }
                } else if let Some(w) = fiw {
                    if w < self.icons.len() {
                        self.icons.remove(w);
                        self.hcz = None;
                    }
                }
            },
            ContextAction::Axv => {
                if cza {
                    
                    if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ak) {
                        let asu = 5usize.v(bh.ca.len());
                        let baw = asu + bh.acm;
                        if baw < bh.ca.len().ao(2) {
                            let j = String::from(Self::cxp(&bh.ca[baw]));
                            if j != ".." {
                                self.xn = j.clone();
                                bh.dq = format!("RENAME:{}", j);
                            }
                        }
                    }
                } else if let Some(w) = fiw {
                    if w < self.icons.len() {
                        crate::serial_println!("[GUI] Rename icon: {}", self.icons[w].j);
                    }
                }
            },
            ContextAction::Azb => {
                if cza {
                    if let Some(ajq) = self.ee.iter().du(|d| d.ja && d.ld == WindowType::Ak).map(|d| d.ad) {
                        if let Some(xa) = self.avu.ds(&ajq) {
                            if xa.eit == 0 { xa.dcc = !xa.dcc; } else { xa.eit = 0; xa.dcc = true; }
                        }
                        if let Some(path) = self.ee.iter().du(|d| d.ad == ajq).and_then(|d| d.wn.clone()) {
                            self.brz(&path);
                        }
                    }
                }
            },
            ContextAction::Btf => {
                if cza {
                    if let Some(ajq) = self.ee.iter().du(|d| d.ja && d.ld == WindowType::Ak).map(|d| d.ad) {
                        if let Some(xa) = self.avu.ds(&ajq) {
                            if xa.eit == 2 { xa.dcc = !xa.dcc; } else { xa.eit = 2; xa.dcc = true; }
                        }
                        if let Some(path) = self.ee.iter().du(|d| d.ad == ajq).and_then(|d| d.wn.clone()) {
                            self.brz(&path);
                        }
                    }
                }
            },
            ContextAction::Cnf => {
                crate::serial_println!("[GUI] Sort by date (not yet supported)");
            },
            ContextAction::Bvt | ContextAction::Cpl | ContextAction::Cpk => {
                crate::serial_println!("[GUI] View mode changed");
            },
            ContextAction::Bow => {
                self.xl("Personalization", 250 + l, 150 + l, 400, 300, WindowType::Gn);
            },
            ContextAction::Azu => {
                self.xl("Terminal", 200 + l, 120 + l, 500, 350, WindowType::Ay);
            },
            ContextAction::Hl => {},
        }
    }
    
    
    fn ncp(&self, b: i32, c: i32) -> Option<usize> {
        
        if b < 0 || b >= (BY_ + 10) as i32 {
            return None;
        }
        let cjz = self.ac.ao(W_);
        let eva = self.icons.len().am(1) as u32;
        let ob = 12u32;
        let bfz = cjz.ao(ob * 2);
        let crk = bfz / eva;
        let vc = ob + (bfz - crk * eva) / 2;
        
        for (w, xzx) in self.icons.iter().cf() {
            let og = (vc + w as u32 * crk) as i32;
            if c >= og && c < og + crk as i32 {
                return Some(w);
            }
        }
        None
    }
    
    
    fn oag(&mut self, hr: IconAction) {
        let l = (self.ee.len() as i32 * 25) % 200;
        let ad = match hr {
            IconAction::Tu => {
                self.xl("Terminal", 120 + l, 60 + l, 640, 440, WindowType::Ay)
            },
            IconAction::Aks => {
                self.xl("Files", 140 + l, 80 + l, 520, 420, WindowType::Ak)
            },
            IconAction::Bnw => {
                self.xl("Calculator", 350 + l, 100 + l, 300, 380, WindowType::Calculator)
            },
            IconAction::Boe => {
                self.xl("NetScan", 140 + l, 80 + l, 640, 440, WindowType::Hy)
            },
            IconAction::Akt => {
                self.xl("Settings", 250 + l, 120 + l, 440, 340, WindowType::Gn)
            },
            IconAction::Akq => {
                self.xl("About TrustOS", 300 + l, 140 + l, 420, 280, WindowType::Jf)
            },
            IconAction::Bod => {
                let gne = self.z.ao(340) as i32;
                let gnf = self.ac.ao(W_ + 600) as i32;
                self.xl("Music Player", gne, gnf.am(20), 320, 580, WindowType::Lw)
            },
            IconAction::Bnz => {
                let kp = self.z;
                let kl = self.ac;
                let ad = self.xl("TrustChess 3D", 0, 0, kp, kl - W_, WindowType::Ih);
                if let Some(d) = self.ee.el().du(|d| d.ad == ad) {
                    d.bkk = true;
                }
                ad
            },
            IconAction::Bny => {
                self.xl("TrustCode", 120 + l, 50 + l, 780, 560, WindowType::Ag)
            },
            IconAction::Cii => {
                self.xl("TrustGL 3D Demo", 120 + l, 50 + l, 500, 420, WindowType::Aqu)
            },
            IconAction::Bnv => {
                self.xl("TrustBrowser", 100 + l, 40 + l, 720, 520, WindowType::Browser)
            },
            IconAction::Boc => {
                self.xl("TrustEdit 3D", 80 + l, 40 + l, 780, 560, WindowType::Fp)
            },
            IconAction::Cij => {
                self.xl("TrustDoom 3D", 60 + l, 30 + l, 720, 540, WindowType::So)
            },
            #[cfg(feature = "emulators")]
            IconAction::Cik => {
                self.xl("NES Emulator", 80 + l, 50 + l, 512, 480, WindowType::Xt)
            },
            #[cfg(feature = "emulators")]
            IconAction::Boa => {
                self.xl("Game Boy", 100 + l, 60 + l, 480, 432, WindowType::Sp)
            },
            #[cfg(feature = "emulators")]
            IconAction::Bob => {
                let kp = self.z;
                let kl = self.ac;
                
                let hpn = 490i32;
                let lhq = (kp as i32 - hpn).am(400) as u32;
                let lhp = kl - W_;
                let ets = self.xl("Game Lab", hpn, 0, lhq, lhp, WindowType::Lm);
                ets
            },
        };
        
        self.dhh(ad);
    }
    
    fn tlh(&mut self, b: i32, iij: i32) {
        
        if b >= (self.z - 8) as i32 {
            self.puc();
            crate::serial_println!("[GUI] Show Desktop corner clicked");
            return;
        }
        
        
        if b >= 4 && b < 120 {
            self.ajo = !self.ajo;
            if !self.ajo {
                self.bij.clear();
            }
            return;
        }
        
        
        let guy = self.z - 120;
        let pjm = guy - 44;
        if b >= pjm as i32 && b < (pjm + 40) as i32 {
            self.lqt();
            return;
        }

        
        let jue = guy;
        if b >= jue as i32 && b < (jue + 20) as i32 {
            
            for d in &self.ee {
                if d.ld == WindowType::Afs {
                    let ad = d.ad;
                    self.dhh(ad);
                    return;
                }
            }
            self.xl("WiFi Networks", 200, 100, 420, 500, WindowType::Afs);
            return;
        }
        
        
        let ied = self.ee.len();
        if ied > 0 {
            let pm = 96u32;
            let aib = 6u32;
            let aza = ied as u32 * (pm + aib) - aib;
            let ql = (self.z.ao(aza)) / 2;
            
            for (a, d) in self.ee.iter().cf() {
                let axp = ql + a as u32 * (pm + aib);
                if b >= axp as i32 && b < (axp + pm) as i32 {
                    let ad = d.ad;
                    
                    if d.ja && !d.aat {
                        self.llz(ad);
                    } else {
                        self.dhh(ad);
                    }
                    return;
                }
            }
        }
    }
    
    
    fn lqt(&mut self) {
        
        for d in &self.ee {
            if d.ld == WindowType::Gn {
                let ad = d.ad;
                self.dhh(ad);
                return;
            }
        }
        
        self.xl("Settings", 180, 80, 620, 440, WindowType::Gn);
    }
    
    
    fn qzw(&self, b: i32, c: i32) -> Option<u8> {
        
        let afr = 480u32;
        let aje = 680u32;
        let rs = 4i32;
        let xp = (self.ac - W_ - aje - 8) as i32;
        
        
        if b < rs || b >= rs + afr as i32 || c < xp || c >= xp + aje as i32 {
            return None;
        }
        
        
        let hpg = xp + 78;
        
        
        let qjn: [&str; 15] = [
            "Terminal", "Files", "Calculator", "Network", "Text Editor",
            "TrustEdit 3D", "Browser", "Snake", "Chess", "Chess 3D",
            "NES Emulator", "Game Boy", "TrustLab", "Music Player", "Settings",
        ];
        let mvy: [u8; 15] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14];
        
        
        let vkl: [&str; 3] = ["Exit Desktop", "Shutdown", "Reboot"];
        let vki: [u8; 3] = [15, 16, 17];
        
        let anw = self.bij.em();
        let cbp: String = anw.bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
        
        
        let doz = 2u32;
        let ezu = (afr - 24) / doz;
        let dwt = 44u32;
        let ezt = 4u32;
        
        let ssr: alloc::vec::Vec<u8> = if anw.is_empty() {
            mvy.ip()
        } else {
            mvy.iter().hi(|&&w| {
                let cu: String = qjn[w as usize].bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
                cu.contains(cbp.as_str())
            }).hu().collect()
        };
        
        
        if c >= hpg && c < xp + aje as i32 - 110 {
            for (fhb, &com) in ssr.iter().cf() {
                let bj = (fhb % doz as usize) as i32;
                let br = (fhb / doz as usize) as i32;
                let dsk = rs + 10 + bj * (ezu + ezt) as i32;
                let ajd = hpg + br * (dwt + ezt) as i32;
                
                if b >= dsk && b < dsk + ezu as i32
                    && c >= ajd && c < ajd + dwt as i32 {
                    return Some(com);
                }
            }
        }
        
        
        let jjs = xp + aje as i32 - 106;
        let owv = jjs + 8;
        if c >= owv {
            for (akk, &vhu) in vki.iter().cf() {
                if !cbp.is_empty() {
                    let glq: String = vkl[akk].bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
                    if !glq.contains(cbp.as_str()) { continue; }
                }
                let ohd = owv + (akk as i32 * 30);
                if c >= ohd && c < ohd + 28 {
                    return Some(vhu);
                }
            }
        }
        
        None
    }
    
    fn law(&mut self, hr: u8) {
        
        
        
        match hr {
            0 => { 
                let b = 100 + (self.ee.len() as i32 * 30);
                let c = 60 + (self.ee.len() as i32 * 20);
                self.xl("Terminal", b, c, 640, 440, WindowType::Ay);
            },
            1 => { 
                self.xl("File Explorer", 100, 60, 780, 520, WindowType::Ak);
            },
            2 => { 
                self.xl("Calculator", 350, 100, 300, 380, WindowType::Calculator);
            },
            3 => { 
                self.xl("NetScan", 140, 80, 640, 440, WindowType::Hy);
            },
            4 => { 
                self.xl("TrustCode", 120, 50, 780, 560, WindowType::Ag);
            },
            5 => { 
                self.xl("TrustEdit 3D", 80, 40, 780, 560, WindowType::Fp);
            },
            6 => { 
                self.xl("TrustBrowser", 100, 40, 720, 520, WindowType::Browser);
            },
            7 => { 
                let kp = self.z;
                let kl = self.ac;
                let ad = self.xl("TrustChess 3D", 0, 0, kp, kl - W_, WindowType::Ih);
                
                if let Some(d) = self.ee.el().du(|d| d.ad == ad) {
                    d.bkk = true;
                }
            },
            8 => { 
                self.xl("TrustChess", 180, 60, 520, 560, WindowType::Gs);
            },
            9 => { 
                self.xl("Snake Game", 220, 80, 380, 400, WindowType::Io);
            },
            10 => { 
                #[cfg(feature = "emulators")]
                self.xl("NES Emulator", 80, 40, 560, 520, WindowType::Xt);
            },
            11 => { 
                #[cfg(feature = "emulators")]
                self.xl("Game Boy", 100, 40, 520, 480, WindowType::Sp);
            },
            12 => { 
                self.osq();
            },
            13 => { 
                crate::serial_println!("[GUI] Opening Music Player...");
                let gne = self.z.ao(320) as i32;
                let gnf = self.ac.ao(W_ + 600) as i32;
                crate::serial_println!("[GUI] Music Player pos: {}x{}", gne, gnf.am(20));
                self.xl("Music Player", gne, gnf.am(20), 320, 580, WindowType::Lw);
                crate::serial_println!("[GUI] Music Player window created OK");
            },
            14 => { 
                self.lqt();
            },
            15 => { 
                crate::serial_println!("[GUI] Exit Desktop from start menu");
                MX_.store(true, Ordering::SeqCst);
            },
            16 => { 
                crate::serial_println!("[SYSTEM] Shutdown sequence initiated");
                self.mfr = true;
                self.mfs = crate::logger::lh();
                self.pkp = 0;
                
                self.ajo = false;
                self.bij.clear();
            },
            17 => { 
                crate::serial_println!("[SYSTEM] Reboot requested");
                
                unsafe {
                    let mut port = crate::arch::Port::<u8>::new(0x64);
                    port.write(0xFE);
                }
                loop { crate::arch::bhd(); }
            },
            _ => {}
        }
    }
    
    
    pub fn oah(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_};
        crate::serial_println!("[KBD-DBG] handle_keyboard_input key={} (0x{:02X}) lock={} start_menu={}",
            bs, bs, self.eug, self.ajo);
        
        
        if self.eug {
            self.tki(bs);
            return;
        }
        
        
        if self.ajo {
            match bs {
                0x1B => { 
                    self.ajo = false;
                    self.bij.clear();
                    self.bsl = -1;
                },
                0x08 | 0x7F => { 
                    self.bij.pop();
                    self.bsl = -1; 
                },
                eh if eh == V_ => { 
                    if self.bsl > 0 {
                        self.bsl -= 1;
                    } else {
                        
                        self.bsl = 16;
                    }
                },
                eh if eh == U_ => { 
                    if self.bsl < 16 {
                        self.bsl += 1;
                    } else {
                        self.bsl = 0;
                    }
                },
                0x0D | 0x0A => { 
                    if self.bsl >= 0 && self.bsl <= 16 {
                        
                        let hr = self.bsl as u8;
                        self.ajo = false;
                        self.bij.clear();
                        self.bsl = -1;
                        self.law(hr);
                        return;
                    }
                    
                    let qgi: [&str; 17] = [
                        "Terminal", "Files", "Calculator", "Network", "Text Editor",
                        "TrustEdit 3D", "Browser", "Snake", "Chess", "Chess 3D",
                        "NES Emulator", "Game Boy", "TrustLab",
                        "Settings", "Exit Desktop", "Shutdown", "Reboot",
                    ];
                    let anw = self.bij.em();
                    if !anw.is_empty() {
                        let cbp: String = anw.bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
                        for (a, cu) in qgi.iter().cf() {
                            let hpp: String = cu.bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
                            if hpp.contains(cbp.as_str()) {
                                self.ajo = false;
                                self.bij.clear();
                                self.bsl = -1;
                                self.law(a as u8);
                                return;
                            }
                        }
                    }
                },
                b' '..=b'~' => { 
                    if self.bij.len() < 32 {
                        self.bij.push(bs as char);
                        self.bsl = -1; 
                    }
                },
                _ => {}
            }
            return;
        }

        
        let ivf = self.ee.iter().du(|d| d.ja).map(|d| (d.ld, d.ad));
        crate::serial_println!("[KBD-DBG] focused_info={:?} n_windows={}",
            ivf.map(|(_, ad)| ad), self.ee.len());
        
        if let Some((ash, nr)) = ivf {
            match ash {
                WindowType::Ay => {
                    self.tlj(bs);
                },
                WindowType::Ak => {
                    
                    let db = crate::keyboard::alh(0x1D);
                    if db && (bs == 3 || bs == b'c' || bs == b'C') {
                        self.iul(false);
                        return;
                    }
                    if db && (bs == 24 || bs == b'x' || bs == b'X') {
                        self.iul(true);
                        return;
                    }
                    if db && (bs == 22 || bs == b'v' || bs == b'V') {
                        self.ntj();
                        return;
                    }
                    
                    if bs == b'v' || bs == b'V' {
                        let cv = self.eqn.get(&nr).hu().unwrap_or(FileManagerViewMode::Px);
                        let opy = match cv {
                            FileManagerViewMode::Px => FileManagerViewMode::Sz,
                            FileManagerViewMode::Sz => FileManagerViewMode::Aaz,
                            FileManagerViewMode::Aaz => FileManagerViewMode::Oh,
                            FileManagerViewMode::Oh => FileManagerViewMode::Px,
                        };
                        self.eqn.insert(nr, opy);
                        crate::serial_println!("[FM] View mode: {:?}-like for window {}", 
                            match opy { FileManagerViewMode::Px => "List", FileManagerViewMode::Sz => "Grid", FileManagerViewMode::Aaz => "Details", FileManagerViewMode::Oh => "Tiles" },
                            nr);
                        return;
                    }
                    self.tjr(bs);
                },
                WindowType::Bp => {
                    self.tjv(bs);
                },
                WindowType::Pj => {
                    self.tjq(bs);
                },
                WindowType::Gn => {
                    self.tkx(bs);
                },
                WindowType::Hy => {
                    self.tkm(bs);
                },
                WindowType::Ag => {
                    
                    if let Some(editor) = self.cxh.ds(&nr) {
                        editor.vr(bs);
                    }
                },
                WindowType::Fp => {
                    
                    if let Some(g) = self.djq.ds(&nr) {
                        g.vr(bs);
                    }
                },
                WindowType::Calculator => {
                    if let Some(akz) = self.enf.ds(&nr) {
                        match bs {
                            b'0'..=b'9' => akz.oxb(bs as char),
                            b'.' => akz.oxc(),
                            b'+' => akz.duq('+'),
                            b'-' => akz.duq('-'),
                            b'*' => akz.duq('*'),
                            b'/' => akz.duq('/'),
                            b'%' => akz.duq('%'),
                            b'(' => akz.jju('('),
                            b')' => akz.jju(')'),
                            b'=' | 0x0D | 0x0A => akz.oxd(), 
                            b'c' | b'C' => akz.oxa(),
                            0x08 => akz.owz(), 
                            0x7F => akz.owz(), 
                            b's' => akz.vkz("sqrt"), 
                            _ => {}
                        }
                    }
                },
                WindowType::Io => {
                    if let Some(atl) = self.eyq.ds(&nr) {
                        atl.vr(bs);
                    }
                },
                WindowType::So => {
                    if let Some(kxo) = self.dra.ds(&nr) {
                        kxo.vr(bs);
                    }
                },
                WindowType::Gs => {
                    if let Some(chess) = self.dou.ds(&nr) {
                        chess.vr(bs);
                    }
                },
                WindowType::Ih => {
                    if let Some(g) = self.cwd.ds(&nr) {
                        g.vr(bs);
                    }
                },
                #[cfg(feature = "emulators")]
                WindowType::Xt => {
                    if let Some(cw) = self.dtk.ds(&nr) {
                        cw.vr(bs);
                    }
                },
                #[cfg(feature = "emulators")]
                WindowType::Sp => {
                    if let Some(cw) = self.arf.ds(&nr) {
                        cw.vr(bs);
                    }
                },
                WindowType::Ro => {
                    if let Some(cve) = self.fdi.ds(&nr) {
                        use crate::keyboard::{V_, U_, AH_, AI_, AM_, AQ_, CQ_, CP_};
                        match bs {
                            V_ => cve.crc(0x48),
                            U_ => cve.crc(0x50),
                            AH_ => cve.crc(0x4B),
                            AI_ => cve.crc(0x4D),
                            AM_ => cve.crc(0x49),
                            AQ_ => cve.crc(0x51),
                            CQ_ => cve.crc(0x47),
                            CP_ => cve.crc(0x4F),
                            0x09 => cve.crc(0x0F), 
                            0x0D | 0x0A => cve.crc(0x1C), 
                            _ => cve.vr(bs as char),
                        }
                    }
                },
                WindowType::Td => {
                    if let Some(abg) = self.dso.ds(&nr) {
                        
                        if bs >= 0x20 && bs < 0x7F {
                            abg.fka(bs as char);
                        } else {
                            abg.vr(bs);
                        }
                    }
                },
                #[cfg(feature = "emulators")]
                WindowType::Lm => {
                    if let Some(abg) = self.azy.ds(&nr) {
                        
                        if bs == 0x0D || bs == 0x0A {
                            if abg.ahd == crate::game_lab::LabTab::Ys {
                                
                                let ebe = abg.fnb
                                    .or_else(|| self.arf.cai().next().hu());
                                if let Some(dqn) = ebe {
                                    let rzv = !abg.bcn;
                                    if rzv {
                                        if let Some(cw) = self.arf.get(&dqn) {
                                            if let Some(fjr) = self.azy.ds(&nr) {
                                                fjr.wfn(cw);
                                            }
                                        }
                                    } else {
                                        if let Some(cw) = self.arf.get(&dqn) {
                                            if let Some(fjr) = self.azy.ds(&nr) {
                                                fjr.wfm(cw);
                                            }
                                        }
                                    }
                                }
                                return;
                            }
                        }
                        abg.vr(bs);
                    }
                },
                WindowType::Browser => {
                    use crate::keyboard::{AH_, AI_, CQ_, CP_, CX_, AM_, AQ_};
                    let db = crate::keyboard::alh(0x1D);
                    crate::serial_println!("[BROWSER] Key received: {} (0x{:02X}) cursor={} url_len={} sel={}", 
                        if bs >= 0x20 && bs < 0x7F { bs as char } else { '?' }, bs,
                        self.aef, self.ado.len(), self.cdj);
                    
                    
                    if self.cdj {
                        match bs {
                            0x08 | _ if bs == CX_ => {
                                
                                self.ado.clear();
                                self.aef = 0;
                                self.cdj = false;
                                return;
                            },
                            0x1B => {
                                
                                self.cdj = false;
                                return;
                            },
                            0x0D | 0x0A => {
                                
                                self.cdj = false;
                            },
                            32..=126 => {
                                
                                self.ado.clear();
                                self.ado.push(bs as char);
                                self.aef = 1;
                                self.cdj = false;
                                return;
                            },
                            _ => {
                                
                                self.cdj = false;
                            }
                        }
                    }
                    
                    
                    if db && (bs == b'a' || bs == b'A') {
                        self.cdj = true;
                        self.aef = self.ado.len();
                        return;
                    }
                    
                    
                    if self.btn && bs != 0x1B {
                        crate::serial_println!("[BROWSER] Key ignored: loading in progress");
                    } else {
                    match bs {
                        0x08 => { 
                            if self.aef > 0 {
                                self.aef -= 1;
                                if self.aef < self.ado.len() {
                                    self.ado.remove(self.aef);
                                }
                            }
                        },
                        0x0D | 0x0A => { 
                            if !self.ado.is_empty() && !self.btn {
                                self.btn = true;
                                let url = self.ado.clone();
                                crate::serial_println!("[DESKTOP] Browser navigate async: {}", url);
                                {
                                    let mut aln = ZR_.lock();
                                    *aln = Some(url);
                                }
                                RR_.store(true, Ordering::SeqCst);
                                crate::thread::jqu("browser-nav", naf, 0);
                            }
                        },
                        0x1B => { 
                            if self.btn {
                                self.btn = false;
                            } else {
                                self.ado.clear();
                                self.aef = 0;
                            }
                        },
                        _ if bs == AH_ => {
                            if db {
                                
                                while self.aef > 0 {
                                    self.aef -= 1;
                                    if self.aef > 0 {
                                        let r = self.ado.as_bytes()[self.aef - 1];
                                        if r == b' ' || r == b'/' || r == b'.' || r == b':' {
                                            break;
                                        }
                                    }
                                }
                            } else if self.aef > 0 {
                                self.aef -= 1;
                            }
                        },
                        _ if bs == AI_ => {
                            if db {
                                
                                let len = self.ado.len();
                                while self.aef < len {
                                    self.aef += 1;
                                    if self.aef < len {
                                        let r = self.ado.as_bytes()[self.aef];
                                        if r == b' ' || r == b'/' || r == b'.' || r == b':' {
                                            break;
                                        }
                                    }
                                }
                            } else if self.aef < self.ado.len() {
                                self.aef += 1;
                            }
                        },
                        _ if bs == CQ_ => {
                            self.aef = 0;
                        },
                        _ if bs == CP_ => {
                            self.aef = self.ado.len();
                        },
                        _ if bs == CX_ => {
                            if self.aef < self.ado.len() {
                                self.ado.remove(self.aef);
                            }
                        },
                        _ if bs == AM_ => {
                            
                            if let Some(ref mut browser) = self.browser {
                                browser.jc(-200);
                            }
                        },
                        _ if bs == AQ_ => {
                            
                            if let Some(ref mut browser) = self.browser {
                                browser.jc(200);
                            }
                        },
                        _ if db && (bs == b'l' || bs == b'L') => {
                            
                            self.cdj = true;
                            self.aef = self.ado.len();
                        },
                        _ if db && (bs == b'r' || bs == b'R') => {
                            
                            if let Some(ref mut browser) = self.browser {
                                let _ = browser.gqr();
                            }
                        },
                        _ if db && (bs == b'a' || bs == b'A') => {
                            
                            self.cdj = true;
                            self.aef = self.ado.len();
                        },
                        _ if bs == b'\t' => {
                            
                            if !self.ado.contains("://") && !self.ado.is_empty() {
                                self.ado = alloc::format!("http://{}", self.ado);
                                self.aef = self.ado.len();
                            }
                        },
                        32..=126 => { 
                            if self.ado.len() < 512 {
                                if self.aef >= self.ado.len() {
                                    self.ado.push(bs as char);
                                } else {
                                    self.ado.insert(self.aef, bs as char);
                                }
                                self.aef += 1;
                            }
                        },
                        _ => {}
                    }
                    }
                },
                WindowType::Is => {
                    use crate::keyboard::{V_, U_, AM_, AQ_, CQ_, CP_};
                    if let Some(bh) = self.ee.el().du(|d| d.ad == nr) {
                        let act = ((bh.ac.ao(J_ + 20)) / 16) as usize;
                        let aye = bh.ca.len().ao(act);
                        match bs {
                            V_ => bh.px = bh.px.ao(1),
                            U_ => bh.px = (bh.px + 1).v(aye),
                            AM_ => bh.px = bh.px.ao(act),
                            AQ_ => bh.px = (bh.px + act).v(aye),
                            CQ_ => bh.px = 0,
                            CP_ => bh.px = aye,
                            _ => {}
                        }
                    }
                },
                WindowType::Aft => {
                    match bs {
                        0x1B => { 
                            self.ee.ajm(|d| d.ad != nr);
                        },
                        0x08 | 0x7F => { 
                            self.ddl.pop();
                        },
                        0x0D | 0x0A => { 
                            if !self.ddl.is_empty() {
                                crate::drivers::net::wifi::lzk(
                                    &self.ihf,
                                    &self.ddl,
                                );
                                self.ee.ajm(|d| d.ad != nr);
                            } else {
                                self.fbj = Some(String::from("Password cannot be empty"));
                            }
                        },
                        b' '..=b'~' => { 
                            if self.ddl.len() < 128 {
                                self.ddl.push(bs as char);
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }
    
    
    fn tjr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, CX_};
        
        
        {
            let svk = self.ee.iter()
                .du(|d| d.ja && d.ld == WindowType::Ak)
                .map(|d| d.ad);
            if let Some(ajq) = svk {
                let chp = self.avu.get(&ajq).map(|bb| bb.chp).unwrap_or(false);
                if chp {
                    if bs == 0x1B { 
                        if let Some(xa) = self.avu.ds(&ajq) {
                            xa.chp = false;
                            xa.bla.clear();
                        }
                        let path = self.ee.iter().du(|d| d.ad == ajq)
                            .and_then(|d| d.wn.clone()).unwrap_or_else(|| String::from("/"));
                        self.brz(&path);
                        return;
                    } else if bs == 0x08 { 
                        if let Some(xa) = self.avu.ds(&ajq) {
                            xa.bla.pop();
                        }
                        let path = self.ee.iter().du(|d| d.ad == ajq)
                            .and_then(|d| d.wn.clone()).unwrap_or_else(|| String::from("/"));
                        self.brz(&path);
                        return;
                    } else if bs == 0x0D || bs == 0x0A { 
                        if let Some(xa) = self.avu.ds(&ajq) {
                            xa.chp = false;
                        }
                        return;
                    } else if bs >= 0x20 && bs < 0x7F {
                        if let Some(xa) = self.avu.ds(&ajq) {
                            if xa.bla.len() < 32 {
                                xa.bla.push(bs as char);
                            }
                        }
                        let path = self.ee.iter().du(|d| d.ad == ajq)
                            .and_then(|d| d.wn.clone()).unwrap_or_else(|| String::from("/"));
                        self.brz(&path);
                        return;
                    }
                    return;
                }
            }
        }
        
        let mut hr: Option<(String, bool)> = None; 
        let mut nkg: Option<String> = None;
        let mut gnm = false;
        let mut opt = false;
        let mut vvb = false;
        
        let mut pbt: Option<(String, String, String)> = None; 
        
        if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ak) {
            
            if bh.dq.cj("RENAME:") {
                if bs == 0x0D || bs == 0x0A { 
                    let jhq = String::from(&bh.dq[7..]);
                    let gno = self.xn.clone();
                    self.xn.clear();
                    bh.dq = String::from("File Manager");
                    let rp = bh.wn.clone().unwrap_or_else(|| String::from("/"));
                    pbt = Some((jhq, gno, rp));
                } else if bs == 0x08 { 
                    self.xn.pop();
                    return;
                } else if bs == 0x1B { 
                    self.xn.clear();
                    bh.dq = String::from("File Manager");
                    return;
                } else if bs >= 0x20 && bs < 0x7F {
                    self.xn.push(bs as char);
                    return;
                }
                return;
            }
            
            
            let bec = bh.ca.len().ao(7); 
            
            if bs == V_ {
                if bh.acm > 0 {
                    bh.acm -= 1;
                }
            } else if bs == U_ {
                if bh.acm < bec.ao(1) {
                    bh.acm += 1;
                }
            } else if bs == 0x08 { 
                hr = Some((String::from(".."), true));
            } else if bs == CX_ { 
                let w = bh.acm + 5;
                if w < bh.ca.len().ao(2) {
                    let line = &bh.ca[w];
                    if let Some(akj) = line.du(']') {
                        if akj + 2 < line.len() {
                            let kr = &line[akj + 2..];
                            if let Some(bew) = kr.du(' ') {
                                let it = String::from(kr[..bew].em());
                                if it != ".." {
                                    nkg = Some(it);
                                }
                            }
                        }
                    }
                }
            } else if bs == b'n' || bs == b'N' { 
                gnm = true;
            } else if bs == b'd' || bs == b'D' { 
                opt = true;
            } else if bs == b'r' || bs == b'R' { 
                vvb = true;
                let w = bh.acm + 5;
                if w < bh.ca.len().ao(2) {
                    let line = &bh.ca[w];
                    if let Some(akj) = line.du(']') {
                        if akj + 2 < line.len() {
                            let kr = &line[akj + 2..];
                            if let Some(bew) = kr.du(' ') {
                                let it = String::from(kr[..bew].em());
                                if it != ".." {
                                    self.xn = it.clone();
                                    bh.dq = format!("RENAME:{}", it);
                                }
                            }
                        }
                    }
                }
            } else if bs == 0x0D || bs == 0x0A { 
                
                let w = bh.acm + 5; 
                if w < bh.ca.len().ao(2) { 
                    let line = &bh.ca[w];
                    
                    if let Some(akj) = line.du(']') {
                        if akj + 2 < line.len() {
                            let kr = &line[akj + 2..];
                            if let Some(bew) = kr.du(' ') {
                                let it = String::from(kr[..bew].em());
                                let ta = line.contains("[D]");
                                hr = Some((it, ta));
                            }
                        }
                    }
                }
            }
        }
        
        
        if let Some((jhq, gno, rp)) = pbt {
            let osj = if rp == "/" { format!("/{}", jhq) } else { format!("{}/{}", rp, jhq) };
            let oqb = if rp == "/" { format!("/{}", gno) } else { format!("{}/{}", rp, gno) };
            let _ = crate::ramfs::fh(|fs| fs.euz(&osj, &oqb));
            crate::serial_println!("[FM] Renamed: {} -> {}", osj, oqb);
            self.brz(&rp);
            return;
        }
        
        
        if let Some(it) = nkg {
            let rp = self.ee.iter()
                .du(|d| d.ja && d.ld == WindowType::Ak)
                .and_then(|d| d.wn.clone())
                .unwrap_or_else(|| String::from("/"));
            let wo = if rp == "/" { format!("/{}", it) } else { format!("{}/{}", rp, it) };
            let _ = crate::ramfs::fh(|fs| fs.hb(&wo));
            crate::serial_println!("[FM] Deleted: {}", wo);
            self.brz(&rp);
            return;
        }
        
        
        if gnm {
            let rp = self.ee.iter()
                .du(|d| d.ja && d.ld == WindowType::Ak)
                .and_then(|d| d.wn.clone())
                .unwrap_or_else(|| String::from("/"));
            let j = format!("new_file_{}.txt", self.oo % 1000);
            let wo = if rp == "/" { format!("/{}", j) } else { format!("{}/{}", rp, j) };
            let _ = crate::ramfs::fh(|fs| fs.touch(&wo));
            crate::serial_println!("[FM] Created file: {}", wo);
            self.brz(&rp);
            return;
        }
        
        
        if opt {
            let rp = self.ee.iter()
                .du(|d| d.ja && d.ld == WindowType::Ak)
                .and_then(|d| d.wn.clone())
                .unwrap_or_else(|| String::from("/"));
            let j = format!("folder_{}", self.oo % 1000);
            let wo = if rp == "/" { format!("/{}", j) } else { format!("{}/{}", rp, j) };
            let _ = crate::ramfs::fh(|fs| fs.ut(&wo));
            crate::serial_println!("[FM] Created folder: {}", wo);
            self.brz(&rp);
            return;
        }
        
        
        if let Some((it, ta)) = hr {
            if ta {
                
                self.jgm(&it);
            } else {
                self.gol(&it);
            }
        }
    }
    
    
    fn brz(&mut self, path: &str) {
        
        let (iba, mgk, wfp) = {
            let ajq = self.ee.iter()
                .du(|d| d.ja && d.ld == WindowType::Ak)
                .map(|d| d.ad);
            if let Some(ajq) = ajq {
                if let Some(xa) = self.avu.get(&ajq) {
                    (xa.eit, xa.dcc, xa.bla.clone())
                } else { (0, true, String::new()) }
            } else { return; }
        };
        
        if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ak) {
            bh.ca.clear();
            bh.ca.push(String::from("=== File Manager ==="));
            bh.ca.push(format!("Path: {}", path));
            bh.ca.push(String::from(""));
            bh.ca.push(String::from("  Name              Type       Size    Program"));
            bh.ca.push(String::from("  ────────────────────────────────────────────"));
            
            if path != "/" {
                bh.ca.push(String::from("  [D] ..             DIR        ---     ---"));
            }
            
            let vev = if path == "/" { Some("/") } else { Some(path) };
            if let Ok(ch) = crate::ramfs::fh(|fs| fs.awb(vev)) {
                
                let cbp: String = wfp.bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
                let mut aud: Vec<&(String, crate::ramfs::FileType, usize)> = if cbp.is_empty() {
                    ch.iter().collect()
                } else {
                    ch.iter().hi(|(j, _, _)| {
                        let hsk: String = j.bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
                        hsk.contains(cbp.as_str())
                    }).collect()
                };
                
                
                aud.bxe(|q, o| {
                    
                    let mtc = q.1 == crate::ramfs::FileType::K;
                    let qmd = o.1 == crate::ramfs::FileType::K;
                    if mtc != qmd {
                        return if mtc { core::cmp::Ordering::Tg } else { core::cmp::Ordering::Ss };
                    }
                    let osz = match iba {
                        1 => { 
                            let spn = q.0.cmm('.').next().unwrap_or("");
                            let spo = o.0.cmm('.').next().unwrap_or("");
                            spn.cmp(spo)
                        }
                        2 => q.2.cmp(&o.2), 
                        _ => q.0.cmp(&o.0), 
                    };
                    if mgk { osz } else { osz.dbh() }
                });
                
                for (j, are, aw) in aud.iter().take(200) {
                    let pa = if *are == crate::ramfs::FileType::K { 
                        "[D]" 
                    } else { 
                        crate::file_assoc::iwq(j)
                    };
                    let ctl = if *are == crate::ramfs::FileType::K {
                        String::from("---")
                    } else {
                        String::from(crate::file_assoc::gih(j).j())
                    };
                    let kxm = if *are == crate::ramfs::FileType::K { "DIR" } else { "FILE" };
                    bh.ca.push(format!("  {} {:<14} {:<10} {:<7} {}", pa, j, kxm, aw, ctl));
                }
            }
            if bh.ca.len() <= 5 + if path != "/" { 1 } else { 0 } {
                bh.ca.push(String::from("  (empty directory)"));
            }
            bh.ca.push(String::from(""));
            bh.ca.push(String::from("  [Del] Delete | [N] New File | [D] New Folder | [F2] Rename"));
            
            bh.wn = Some(String::from(path));
            bh.acm = 0;
            bh.px = 0;
        }
    }
    
    
    fn jgm(&mut self, fgu: &str) {
        
        let (dag, ajq) = {
            if let Some(bh) = self.ee.iter().du(|d| d.ja && d.ld == WindowType::Ak) {
                let rp = bh.wn.clone().unwrap_or_else(|| String::from("/"));
                let dag = if fgu == ".." {
                    if rp == "/" {
                        String::from("/")
                    } else {
                        let ux = rp.bdd('/');
                        match ux.bhx('/') {
                            Some(0) => String::from("/"),
                            Some(u) => String::from(&ux[..u]),
                            None => String::from("/"),
                        }
                    }
                } else if rp == "/" {
                    format!("/{}", fgu)
                } else {
                    format!("{}/{}", rp.bdd('/'), fgu)
                };
                crate::serial_println!("[FM] Navigate: {} -> {}", rp, dag);
                (dag, bh.ad)
            } else { return; }
        };
        
        
        if let Some(xa) = self.avu.ds(&ajq) {
            xa.bla.clear();
            xa.chp = false;
        }
        
        
        if let Some(bh) = self.ee.el().du(|d| d.ad == ajq) {
            bh.wn = Some(dag.clone());
        }
        
        
        self.brz(&dag);
        
        
        if let Some(xa) = self.avu.ds(&ajq) {
            xa.lwg(&dag);
        }
    }
    
    
    fn gol(&mut self, it: &str) {
        use crate::file_assoc::{gih, Program};
        
        let alo = gih(it);
        let l = (self.ee.len() as i32 * 25) % 150;
        
        match alo {
            Program::Ag => {
                let ad = self.xl(&format!("TrustCode: {}", it), 150 + l, 80 + l, 700, 500, WindowType::Ag);
                
                if let Some(editor) = self.cxh.ds(&ad) {
                    editor.dsu(it);
                }
                crate::serial_println!("[TrustCode] Opened: {}", it);
            },
            Program::Bp => {
                let ad = self.xl(&format!("View: {}", it), 180 + l, 100 + l, 500, 420, WindowType::Bp);
                if let Some(bh) = self.ee.el().du(|d| d.ad == ad) {
                    bh.wn = Some(String::from(it));
                    bh.ca.clear();
                    
                    
                    let wn = format!("/{}", it);
                    if let Ok(bal) = crate::ramfs::fh(|fs| fs.mq(&wn).map(|bc| bc.ip())) {
                        
                        if let Some(th) = crate::theme::bmp::hqf(&bal) {
                            let mut g = ImageViewerState::new();
                            g.esh = th.z;
                            g.flc = th.ac;
                            g.hz = th.hz;
                            
                            let suh = (480 * 100) / th.z.am(1);
                            let sug = (360 * 100) / th.ac.am(1);
                            g.ddn = suh.v(sug).v(200);
                            self.gjo.insert(ad, g);
                            crate::serial_println!("[ImageViewer] Loaded BMP: {}x{}", th.z, th.ac);
                            bh.ca.push(format!("Image: {} ({}x{} BMP)", it, th.z, th.ac));
                        } else {
                            
                            bh.ca.push(format!("=== Image: {} ===", it));
                            bh.ca.push(format!("Size: {} bytes", bal.len()));
                            if bal.len() >= 2 && &bal[0..2] == b"BM" {
                                bh.ca.push(String::from("BMP detected but failed to parse"));
                            } else {
                                bh.ca.push(String::from("Format not supported (BMP only)"));
                            }
                            self.gjo.insert(ad, ImageViewerState::new());
                        }
                    } else {
                        bh.ca.push(String::from("Failed to read file"));
                        self.gjo.insert(ad, ImageViewerState::new());
                    }
                }
            },
            Program::Is => {
                let ad = self.xl(&format!("Hex: {}", it), 160 + l, 80 + l, 500, 350, WindowType::Is);
                if let Some(bh) = self.ee.el().du(|d| d.ad == ad) {
                    bh.wn = Some(String::from(it));
                    bh.ca.clear();
                    bh.ca.push(format!("=== Hex View: {} ===", it));
                    bh.ca.push(String::new());
                    bh.ca.push(String::from("Offset   00 01 02 03 04 05 06 07  ASCII"));
                    bh.ca.push(String::from("──────── ─────────────────────── ────────"));
                    
                    let wn = format!("/{}", it);
                    if let Ok(ca) = crate::ramfs::fh(|fs| fs.mq(&wn).map(|bc| bc.ip())) {
                        let xv = ca.len();
                        for (a, jj) in ca.btq(8).cf() {
                            let l = a * 8;
                            let nu: String = jj.iter()
                                .map(|o| format!("{:02X} ", o))
                                .collect();
                            let ascii: String = jj.iter()
                                .map(|&o| if o >= 0x20 && o < 0x7F { o as char } else { '.' })
                                .collect();
                            bh.ca.push(format!("{:08X} {:<24} {}", l, nu, ascii));
                        }
                        bh.ca.push(String::new());
                        bh.ca.push(format!("Total: {} bytes ({} lines)", xv, bh.ca.len() - 4));
                    }
                    bh.px = 0;
                }
            },
            Program::Ay => {
                
                crate::serial_println!("[EXEC] Would execute: {}", it);
                let ad = self.xl("Execution", 200 + l, 150 + l, 400, 200, WindowType::Ay);
                if let Some(bh) = self.ee.el().du(|d| d.ad == ad) {
                    bh.ca.clear();
                    bh.ca.push(format!("Executing: {}", it));
                    bh.ca.push(String::from(""));
                    bh.ca.push(String::from("(ELF execution not yet integrated in GUI)"));
                }
            },
            _ => {
                
                crate::serial_println!("[OPEN] No handler for: {}", it);
            }
        }
    }
    
    
    fn tkx(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_};
        
        
        if bs == V_ {
            if self.dvt > 0 {
                self.dvt -= 1;
            }
            return;
        }
        if bs == U_ {
            if self.dvt < 7 {
                self.dvt += 1;
            }
            return;
        }
        
        match self.dvt {
            0 => { 
                if bs == b'1' {
                    xir();
                } else if bs == b'2' {
                    let cv = *GA_.lock();
                    let next = if cv <= 0.5 { 1.0 } else if cv <= 1.0 { 2.0 } else { 0.5 };
                    *GA_.lock() = next;
                } else if bs == b'3' {
                    
                    let clm = match self.asr {
                        DesktopTier::Bv => DesktopTier::Gc,
                        DesktopTier::Gc => DesktopTier::Gy,
                        DesktopTier::Gy | DesktopTier::Aap => DesktopTier::Bv,
                    };
                    self.asr = clm;
                    self.dcq = true;
                    self.bqt = 0;
                    self.cqt = 0;
                    self.bex = true;
                    self.doh = false;
                    crate::serial_println!("[Desktop] Manual tier change: {:?} (override=ON)", clm);
                } else if bs == b'4' {
                    
                    self.dcq = !self.dcq;
                    self.bqt = 0;
                    self.cqt = 0;
                    crate::serial_println!("[Desktop] Manual override: {}", if self.dcq { "ON" } else { "OFF (auto)" });
                }
            },
            1 => { 
                if bs == b'1' {
                    let api = &mut Aa.lock().gtw;
                    *api = (*api + 10).v(100);
                } else if bs == b'2' {
                    let api = &mut Aa.lock().gtw;
                    *api = api.ao(10);
                }
            },
            2 => { 
                
            },
            3 => { 
                if bs == b'1' {
                    
                    let cv = crate::theme::Ib.read().j.clone();
                    let next = if cv == "windows11_dark" { "dark" } else { "windows11" };
                    crate::theme::piq(next);
                    self.bex = true;
                    self.doh = false;
                }
            },
            4 => { 
                if bs == b'1' {
                    crate::accessibility::mln();
                    self.bex = true;
                    self.doh = false;
                } else if bs == b'2' {
                    crate::accessibility::niy();
                } else if bs == b'3' {
                    crate::accessibility::nix();
                } else if bs == b'4' {
                    crate::accessibility::pud();
                } else if bs == b'5' {
                    crate::accessibility::niz();
                }
            },
            5 => { 
                
            },
            6 => { 
                if bs == b'3' || bs == 0x0D {
                    let l = (self.ee.len() as i32 * 20) % 100;
                    self.xl("File Associations", 250 + l, 130 + l, 500, 400, WindowType::Pj);
                }
            },
            7 => { 
                
            },
            _ => {}
        }
    }
    
    
    fn sfj(&self, bh: &Window) {
        let fx = bh.b;
        let lw = bh.c;
        let hk = bh.z;
        let mg = bh.ac;
        
        if hk < 200 || mg < 160 { return; }
        
        let gl = lw + J_ as i32;
        let nd = mg.ao(J_);
        let zb = fx.am(0) as u32;
        
        let hoy = crate::accessibility::edv();
        let gba = if hoy { 0xFF0A0A0A } else { 0xFF060E08 };
        let kcx = if hoy { 0xFF000000 } else { 0xFF0A140C };
        let aek = 0xFF2A6A3Au32;
        let agn = I_;
        let adm = 0xFF88AA88;
        let agb = 0xFFBBDDBB;
        let ahv = 0xFF446644;
        
        
        let aiq = 140u32;
        framebuffer::ah(zb, gl as u32, aiq, nd, gba);
        
        let fej = [
            ("Display",      "@"),
            ("Sound",        "~"),
            ("Taskbar",      "_"),
            ("Personal.",    "*"),
            ("Access.",      "A"),
            ("Network",      "N"),
            ("Apps",         "#"),
            ("About",        "?"),
        ];
        
        let ali = 32i32;
        let mut cq = gl + 8;
        for (a, (cu, pa)) in fej.iter().cf() {
            let rl = a as u8 == self.dvt;
            
            if rl {
                mf(zb as i32 + 4, cq - 1, aiq - 8, ali as u32 - 2, 4, 0xFF0C2A14);
                framebuffer::ah(zb + 2, (cq + 2) as u32, 3, (ali - 6) as u32, agn);
            }
            
            let r = if rl { agn } else { adm };
            self.en(zb as i32 + 14, cq + 8, pa, if rl { agn } else { ahv });
            self.en(zb as i32 + 28, cq + 8, cu, r);
            cq += ali;
        }
        
        
        framebuffer::ah(zb + aiq - 1, gl as u32, 1, nd, 0xFF1A3A1A);
        
        
        let cx = zb + aiq;
        let dt = hk.ao(aiq);
        framebuffer::ah(cx, gl as u32, dt, nd, kcx);
        
        let y = cx as i32 + 20; 
        let mut x = gl + 16;  
        let gy = 22i32;
        
        match self.dvt {
            0 => { 
                self.en(y, x, "Display", agn);
                self.en(y + 1, x, "Display", agn); 
                x += gy + 8;
                
                self.en(y, x, "Resolution", adm);
                self.en(y + 120, x, &alloc::format!("{}x{}", self.z, self.ac), agb);
                x += gy;
                
                let ezr = crate::theme::Ib.read().j.clone();
                self.en(y, x, "Theme", adm);
                self.en(y + 120, x, &ezr, agb);
                x += gy + 8;
                
                
                let qio = col();
                self.fgz(y, x, "[1] Animations", qio);
                x += gy;
                
                
                let ig = *GA_.lock();
                self.en(y, x, "[2] Anim Speed", adm);
                self.en(y + 180, x, &alloc::format!("{:.1}x", ig), agb);
                x += gy + 8;
                
                
                framebuffer::zs((y) as u32, (x + 2) as u32, dt.ao(40), 0xFF1A3A1A);
                x += gy;
                self.en(y, x, "Desktop Mode", agn);
                x += gy;
                
                let xgy = match self.asr {
                    DesktopTier::Bv => "Full",
                    DesktopTier::Gc => "Standard",
                    DesktopTier::Gy => "Minimal",
                    DesktopTier::Aap => "CLI Only",
                };
                self.en(y, x, "[3] Mode", adm);
                self.en(y + 120, x, xgy, agb);
                x += gy;
                
                self.fgz(y, x, "[4] Manual Override", self.dcq);
                x += gy;
                
                if !self.dcq {
                    self.en(y + 12, x, "(auto-adjusts based on FPS)", ahv);
                } else {
                    self.en(y + 12, x, "(locked, no auto-downgrade)", agb);
                }
                x += gy;
            },
            1 => { 
                self.en(y, x, "Sound", agn);
                self.en(y + 1, x, "Sound", agn);
                x += gy + 8;
                
                self.en(y, x, "Master Volume", adm);
                let api = self.gtw;
                self.sfl(y + 140, x, dt.ao(180) as i32, api, 100);
                x += gy;
                
                self.en(y, x, "[1] Volume +  [2] Volume -", ahv);
                x += gy + 8;
                
                
                self.en(y, x, "Audio Device", adm);
                x += gy;
                let kro = if crate::drivers::hda::ky() { "Intel HDA (active)" } else { "Not detected" };
                self.en(y + 12, x, kro, ahv);
            },
            2 => { 
                self.en(y, x, "Taskbar", agn);
                self.en(y + 1, x, "Taskbar", agn);
                x += gy + 8;
                
                let bov = crate::theme::bou();
                self.en(y, x, "Position", adm);
                let dar = match bov.qf {
                    crate::theme::TaskbarPosition::Hk => "Bottom",
                    crate::theme::TaskbarPosition::Jd => "Top",
                    crate::theme::TaskbarPosition::Ap => "Left",
                    crate::theme::TaskbarPosition::Ca => "Right",
                };
                self.en(y + 120, x, dar, agb);
                x += gy;
                
                self.en(y, x, "Height", adm);
                self.en(y + 120, x, &alloc::format!("{}px", bov.ac), agb);
                x += gy;
                
                self.fgz(y, x, "Show Clock", bov.iai);
                x += gy;
                
                self.fgz(y, x, "Show Date", bov.jqa);
                x += gy;
                
                self.fgz(y, x, "Centered Icons", bov.gch);
            },
            3 => { 
                self.en(y, x, "Personalization", agn);
                self.en(y + 1, x, "Personalization", agn);
                x += gy + 8;
                
                let ezr = crate::theme::Ib.read().j.clone();
                self.en(y, x, "[1] Theme", adm);
                self.en(y + 120, x, &ezr, agb);
                x += gy;
                
                self.en(y, x, "Available themes:", ahv);
                x += gy;
                let xgg = ["dark_green", "windows11_dark"];
                let cze = ["TrustOS Dark", "Windows 11 Dark"];
                for (a, cu) in cze.iter().cf() {
                    let afb = ezr == xgg[a];
                    let r = if afb { agn } else { adm };
                    let marker = if afb { " *" } else { "  " };
                    self.en(y + 16, x, &alloc::format!("{}{}", marker, cu), r);
                    x += gy;
                }
                x += 8;
                
                let colors = crate::theme::colors();
                self.en(y, x, "Accent Color", adm);
                
                framebuffer::ah((y + 120) as u32, x as u32, 20, 14, colors.mm);
                x += gy;
                
                self.en(y, x, "Background", adm);
                framebuffer::ah((y + 120) as u32, x as u32, 20, 14, colors.cop);
            },
            4 => { 
                self.en(y, x, "Accessibility", agn);
                self.en(y + 1, x, "Accessibility", agn);
                x += gy + 8;
                
                self.fgz(y, x, "[1] High Contrast", crate::accessibility::edv());
                x += gy;
                
                self.en(y, x, "[2] Font Size", adm);
                self.en(y + 160, x, crate::accessibility::gid().cu(), agb);
                x += gy;
                
                self.en(y, x, "[3] Cursor Size", adm);
                self.en(y + 160, x, crate::accessibility::gib().cu(), agb);
                x += gy;
                
                self.fgz(y, x, "[4] Sticky Keys", crate::accessibility::dsj());
                x += gy;
                
                self.en(y, x, "[5] Mouse Speed", adm);
                self.en(y + 160, x, crate::accessibility::gig().cu(), agb);
            },
            5 => { 
                self.en(y, x, "Network", agn);
                self.en(y + 1, x, "Network", agn);
                x += gy + 8;
                
                
                self.en(y, x, "Interface", adm);
                x += gy;
                
                if let Some(ed) = crate::network::ckt() {
                    self.en(y + 12, x, &alloc::format!("MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", ed[0], ed[1], ed[2], ed[3], ed[4], ed[5]), agb);
                } else {
                    self.en(y + 12, x, "MAC: Not available", ahv);
                }
                x += gy;
                
                if let Some((ip, hs, nt)) = crate::network::aou() {
                    self.en(y + 12, x, &alloc::format!("IP:   {}", ip), agb);
                    x += gy;
                    self.en(y + 12, x, &alloc::format!("Mask: {}", hs), agb);
                    x += gy;
                    if let Some(at) = nt {
                        self.en(y + 12, x, &alloc::format!("GW:   {}", at), agb);
                    }
                } else {
                    self.en(y + 12, x, "IP: Waiting for DHCP...", ahv);
                }
                x += gy + 8;
                
                let rj = if crate::virtio_net::ky() { "virtio-net (active)" }
                    else if crate::drivers::net::bzy() { "RTL8169/e1000 (active)" }
                    else { "No driver loaded" };
                self.en(y, x, "Driver", adm);
                self.en(y + 80, x, rj, agb);
            },
            6 => { 
                self.en(y, x, "Default Apps", agn);
                self.en(y + 1, x, "Default Apps", agn);
                x += gy + 8;
                
                let kbi = crate::file_assoc::ojn();
                self.en(y, x, "Extension", ahv);
                self.en(y + 100, x, "Program", ahv);
                self.en(y + 220, x, "Type", ahv);
                x += 4;
                framebuffer::zs((y) as u32, (x + 12) as u32, dt.ao(40), 0xFF1A3A1A);
                x += gy;
                
                for (wm, ctl, desc) in kbi.iter().take(10) {
                    self.en(y, x, &alloc::format!(".{}", wm), agb);
                    self.en(y + 100, x, ctl, adm);
                    self.en(y + 220, x, desc, ahv);
                    x += gy;
                }
                x += 8;
                self.en(y, x, "[3] Edit File Associations...", adm);
            },
            7 => { 
                self.en(y, x, "About TrustOS", agn);
                self.en(y + 1, x, "About TrustOS", agn);
                x += gy + 8;
                
                self.en(y, x, "TrustOS", 0xFFCCEECC);
                self.en(y + 1, x, "TrustOS", 0xFFCCEECC);
                x += gy;
                self.en(y, x, "Version 0.2.0", agb);
                x += gy;
                self.en(y, x, "Bare-metal OS written in Rust", adm);
                x += gy + 8;
                
                self.en(y, x, "Kernel", ahv);
                self.en(y + 80, x, "trustos_kernel (x86_64)", agb);
                x += gy;
                
                self.en(y, x, "Arch", ahv);
                self.en(y + 80, x, "x86_64", agb);
                x += gy;
                
                self.en(y, x, "Display", ahv);
                self.en(y + 80, x, &alloc::format!("{}x{}", self.z, self.ac), agb);
                x += gy;
                
                self.en(y, x, "AI", ahv);
                self.en(y + 80, x, "JARVIS (Transformer 4.4M params)", agb);
                x += gy + 8;
                
                self.en(y, x, "(c) 2026 Nathan", adm);
            },
            _ => {}
        }
    }
    
    
    fn fgz(&self, b: i32, c: i32, cu: &str, iq: bool) {
        let adm = 0xFF88AA88;
        let agn = I_;
        self.en(b, c, cu, adm);
        
        let gx = b + 180;
        let qd = 36u32;
        let ejt = 16u32;
        let xlh = if iq { 0xFF1A5A2A } else { 0xFF1A1A1A };
        mf(gx, c, qd, ejt, 8, xlh);
        tf(gx, c, qd, ejt, 8, if iq { agn } else { 0xFF333333 });
        
        let etq = if iq { gx + qd as i32 - 14 } else { gx + 2 };
        let ubo = if iq { agn } else { 0xFF666666 };
        for bg in 0..12u32 {
            for dx in 0..12u32 {
                let ym = dx as i32 - 6;
                let wl = bg as i32 - 6;
                if ym * ym + wl * wl <= 36 {
                    framebuffer::ii((etq + dx as i32) as u32, (c as u32 + 2 + bg), ubo);
                }
            }
        }
    }
    
    
    fn sfl(&self, b: i32, c: i32, z: i32, bn: u32, aki: u32) {
        let ekb = z.am(40) as u32;
        let bdc = 6u32;
        let ty = c + 5;
        
        
        mf(b, ty, ekb, bdc, 3, 0xFF1A1A1A);
        
        
        let akd = ((bn as u64 * ekb as u64) / aki.am(1) as u64) as u32;
        if akd > 0 {
            mf(b, ty, akd.v(ekb), bdc, 3, 0xFF1A5A2A);
        }
        
        
        let etq = b + akd as i32;
        for bg in 0..10u32 {
            for dx in 0..10u32 {
                let ym = dx as i32 - 5;
                let wl = bg as i32 - 5;
                if ym * ym + wl * wl <= 25 {
                    framebuffer::ii((etq + dx as i32 - 5).am(0) as u32, (ty as u32 - 2 + bg), I_);
                }
            }
        }
        
        
        self.en(b + ekb as i32 + 8, c, &alloc::format!("{}", bn), 0xFFBBDDBB);
    }
    
    
    fn tkm(&mut self, bs: u8) {
        
        if bs >= b'1' && bs <= b'6' {
            self.efr = bs - b'1';
            return;
        }
        use crate::keyboard::{AH_, AI_};
        if bs == AH_ {
            self.efr = self.efr.ao(1);
            return;
        }
        if bs == AI_ {
            if self.efr < 5 { self.efr += 1; }
            return;
        }
        
        match self.efr {
            1 => { 
                if bs == b's' || bs == b'S' {
                    if let Some((fcc, elo, nt)) = crate::network::aou() {
                        if let Some(at) = nt {
                            let cd = *at.as_bytes();
                            let (hd, cm) = crate::netscan::port_scanner::oyv(cd);
                            if let Some(bh) = self.ee.el().du(|d| d.ld == WindowType::Hy) {
                                bh.ca.clear();
                                bh.ca.push(alloc::format!("Scan: {} | Open: {} | Closed: {} | {:.0}ms",
                                    crate::netscan::aot(cd), cm.aji, cm.cwg, cm.oz));
                                for oc in &hd {
                                    let boo = match oc.g {
                                        crate::netscan::port_scanner::PortState::Ck => "OPEN",
                                        crate::netscan::port_scanner::PortState::Dk => "closed",
                                        crate::netscan::port_scanner::PortState::Kl => "filtered",
                                        _ => "unknown",
                                    };
                                    bh.ca.push(alloc::format!("  Port {}: {} ({})", oc.port, boo, oc.xi));
                                }
                                if hd.is_empty() {
                                    bh.ca.push(String::from("  No open ports found"));
                                }
                            }
                        }
                    }
                }
            },
            2 => { 
                if bs == b'd' || bs == b'D' {
                    let bab = crate::netscan::discovery::kbb(3000);
                    if let Some(bh) = self.ee.el().du(|d| d.ld == WindowType::Hy) {
                        bh.ca.clear();
                        bh.ca.push(alloc::format!("ARP Sweep: {} hosts found", bab.len()));
                        for kh in &bab {
                            let djg = match kh.ed {
                                Some(ef) => crate::netscan::eqs(ef),
                                None => String::from("??:??:??:??:??:??"),
                            };
                            bh.ca.push(alloc::format!("  {} - {} ({}ms)",
                                crate::netscan::aot(kh.ip), djg, kh.bcj));
                        }
                        if bab.is_empty() {
                            bh.ca.push(String::from("  No hosts discovered"));
                        }
                    }
                }
            },
            3 => { 
                if bs == b's' || bs == b'S' {
                    if crate::netscan::sniffer::edu() {
                        crate::netscan::sniffer::gth();
                    } else {
                        crate::netscan::sniffer::gtb();
                    }
                }
            },
            4 => { 
                if bs == b't' || bs == b'T' {
                    if let Some((fcc, elo, nt)) = crate::network::aou() {
                        if let Some(at) = nt {
                            let cd = *at.as_bytes();
                            let cyn = crate::netscan::traceroute::trace(cd, 30, 5000);
                            let fiy = crate::netscan::traceroute::swb(&cyn);
                            if let Some(bh) = self.ee.el().du(|d| d.ld == WindowType::Hy) {
                                bh.ca.clear();
                                for line in fiy.ak() {
                                    bh.ca.push(String::from(line));
                                }
                            }
                        }
                    }
                }
            },
            5 => { 
                if bs == b'v' || bs == b'V' {
                    if let Some((fcc, elo, nt)) = crate::network::aou() {
                        if let Some(at) = nt {
                            let cd = *at.as_bytes();
                            
                            let (vjz, _) = crate::netscan::port_scanner::oyv(cd);
                            let dkf: alloc::vec::Vec<u16> = vjz.iter()
                                .hi(|ai| oh!(ai.g, crate::netscan::port_scanner::PortState::Ck))
                                .map(|ai| ai.port)
                                .collect();
                            let hd = crate::netscan::vuln::arx(cd, &dkf);
                            let report = crate::netscan::vuln::fix(cd, &hd);
                            if let Some(bh) = self.ee.el().du(|d| d.ld == WindowType::Hy) {
                                bh.ca.clear();
                                for line in report.ak() {
                                    bh.ca.push(String::from(line));
                                }
                            }
                        }
                    }
                }
            },
            _ => {}
        }
    }
    
    
    fn sej(&self, bh: &Window) {
        let fx = bh.b;
        let lw = bh.c;
        let hk = bh.z;
        let mg = bh.ac;
        
        if hk < 200 || mg < 120 { return; }
        
        let gl = lw + J_ as i32;
        let nd = mg.ao(J_);
        let zb = fx.am(0) as u32;
        
        let ei = 0xFF0A140Cu32;
        let xaf = 0xFF060E08u32;
        let xae = 0xFF0C2A14u32;
        let agn = I_;
        let adm = 0xFF88AA88u32;
        let agb = 0xFFBBDDBBu32;
        let ahv = 0xFF446644u32;
        let aia = 0xFF1A3A1Au32;
        
        
        framebuffer::ah(zb, gl as u32, hk, nd, ei);
        
        
        let dwo = 28u32;
        framebuffer::ah(zb, gl as u32, hk, dwo, xaf);
        framebuffer::ah(zb, (gl + dwo as i32) as u32, hk, 1, aia);
        
        let bio = ["Dashboard", "PortScan", "Discovery", "Sniffer", "Traceroute", "VulnScan"];
        let axb = (hk / bio.len() as u32).am(80);
        
        for (a, cu) in bio.iter().cf() {
            let gx = zb + (a as u32 * axb);
            let rl = a as u8 == self.efr;
            
            if rl {
                framebuffer::ah(gx, gl as u32, axb, dwo, xae);
                
                framebuffer::ah(gx + 4, (gl + dwo as i32 - 2) as u32, axb - 8, 2, agn);
            }
            
            let r = if rl { agn } else { ahv };
            
            let bda = cu.len() as i32 * 8;
            let wg = gx as i32 + (axb as i32 - bda) / 2;
            self.en(wg, gl + 7, cu, r);
        }
        
        
        let cx = zb as i32 + 16;
        let mut ae = gl + dwo as i32 + 12;
        let gy = 20i32;
        let ijy = hk.ao(32);
        
        match self.efr {
            0 => { 
                self.en(cx, ae, "Network Dashboard", agn);
                self.en(cx + 1, ae, "Network Dashboard", agn);
                ae += gy + 8;
                
                
                let dzr = crate::virtio_net::ky() || crate::drivers::net::bzy();
                let dch = if dzr { 0xFF33DD66u32 } else { 0xFFDD3333u32 };
                let fvq = if dzr { "Connected" } else { "Disconnected" };
                self.en(cx, ae, "Status:", adm);
                
                for bg in 0..8u32 {
                    for dx in 0..8u32 {
                        let ym = dx as i32 - 4;
                        let wl = bg as i32 - 4;
                        if ym * ym + wl * wl <= 16 {
                            framebuffer::ii((cx + 70 + dx as i32) as u32, (ae + 4 + bg as i32) as u32, dch);
                        }
                    }
                }
                self.en(cx + 84, ae, fvq, dch);
                ae += gy;
                
                
                let rj = if crate::virtio_net::ky() { "virtio-net" }
                    else if crate::drivers::net::bzy() { "RTL8169/e1000" }
                    else { "None" };
                self.en(cx, ae, "Driver:", adm);
                self.en(cx + 70, ae, rj, agb);
                ae += gy;
                
                
                if let Some(ed) = crate::network::ckt() {
                    self.en(cx, ae, "MAC:", adm);
                    self.en(cx + 70, ae, &alloc::format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", ed[0], ed[1], ed[2], ed[3], ed[4], ed[5]), agb);
                }
                ae += gy;
                
                
                if let Some((ip, hs, nt)) = crate::network::aou() {
                    self.en(cx, ae, "IP:", adm);
                    self.en(cx + 70, ae, &alloc::format!("{}", ip), agb);
                    ae += gy;
                    self.en(cx, ae, "Subnet:", adm);
                    self.en(cx + 70, ae, &alloc::format!("{}", hs), agb);
                    ae += gy;
                    if let Some(at) = nt {
                        self.en(cx, ae, "Gateway:", adm);
                        self.en(cx + 70, ae, &alloc::format!("{}", at), agb);
                        ae += gy;
                    }
                } else {
                    self.en(cx, ae, "IPv4:", adm);
                    self.en(cx + 70, ae, "Waiting for DHCP...", ahv);
                    ae += gy;
                }
                
                ae += 8;
                
                let cm = crate::network::asx();
                self.en(cx, ae, "Packets", ahv);
                ae += gy;
                self.en(cx + 8, ae, &alloc::format!("TX: {}  RX: {}", cm.egc, cm.dub), agb);
                ae += gy;
                self.en(cx + 8, ae, &alloc::format!("Bytes TX: {}  RX: {}", cm.feb, cm.cdm), agb);
                
                ae += gy + 8;
                self.en(cx, ae, "Use tabs [1-6] or Left/Right to navigate", ahv);
            },
            1 => { 
                self.en(cx, ae, "Port Scanner", agn);
                self.en(cx + 1, ae, "Port Scanner", agn);
                ae += gy + 8;
                
                if let Some((fcc, elo, nt)) = crate::network::aou() {
                    if let Some(at) = nt {
                        self.en(cx, ae, "Target:", adm);
                        self.en(cx + 70, ae, &alloc::format!("{} (gateway)", at), agb);
                        ae += gy + 4;
                    }
                }
                
                self.en(cx, ae, "[S] Start Quick Scan", adm);
                ae += gy + 8;
                
                
                if !bh.ca.is_empty() {
                    framebuffer::ah(zb + 8, ae as u32, ijy, 1, aia);
                    ae += 6;
                    self.en(cx, ae, "Results:", ahv);
                    ae += gy;
                    for line in bh.ca.iter() {
                        if ae > lw + mg as i32 - 20 { break; }
                        let r = if line.contains("OPEN") { 0xFF33DD66u32 } else { agb };
                        self.en(cx + 8, ae, line, r);
                        ae += gy;
                    }
                }
            },
            2 => { 
                self.en(cx, ae, "Network Discovery", agn);
                self.en(cx + 1, ae, "Network Discovery", agn);
                ae += gy + 8;
                
                self.en(cx, ae, "[D] Run ARP Sweep", adm);
                ae += gy + 8;
                
                if !bh.ca.is_empty() {
                    framebuffer::ah(zb + 8, ae as u32, ijy, 1, aia);
                    ae += 6;
                    for line in bh.ca.iter() {
                        if ae > lw + mg as i32 - 20 { break; }
                        self.en(cx + 8, ae, line, agb);
                        ae += gy;
                    }
                }
            },
            3 => { 
                self.en(cx, ae, "Packet Sniffer", agn);
                self.en(cx + 1, ae, "Packet Sniffer", agn);
                ae += gy + 8;
                
                let kgn = crate::netscan::sniffer::edu();
                let status = if kgn { "Capturing..." } else { "Idle" };
                let jt = if kgn { 0xFF33DD66u32 } else { ahv };
                self.en(cx, ae, "Status:", adm);
                self.en(cx + 70, ae, status, jt);
                ae += gy;
                
                let xiv = if kgn { "[S] Stop Capture" } else { "[S] Start Capture" };
                self.en(cx, ae, xiv, adm);
                ae += gy + 8;
                
                let (xkn, xv, cox) = crate::netscan::sniffer::asx();
                self.en(cx, ae, "Captured:", adm);
                self.en(cx + 80, ae, &alloc::format!("{} packets", xkn), agb);
                ae += gy;
                self.en(cx, ae, "Bytes:", adm);
                self.en(cx + 80, ae, &alloc::format!("{}", xv), agb);
                ae += gy;
                self.en(cx, ae, "Buffered:", adm);
                self.en(cx + 80, ae, &alloc::format!("{}", cox), agb);
            },
            4 => { 
                self.en(cx, ae, "Traceroute", agn);
                self.en(cx + 1, ae, "Traceroute", agn);
                ae += gy + 8;
                
                if let Some((fcc, elo, nt)) = crate::network::aou() {
                    if let Some(at) = nt {
                        self.en(cx, ae, "Target:", adm);
                        self.en(cx + 70, ae, &alloc::format!("{}", at), agb);
                        ae += gy + 4;
                    }
                }
                
                self.en(cx, ae, "[T] Run Traceroute", adm);
                ae += gy + 8;
                
                if !bh.ca.is_empty() {
                    framebuffer::ah(zb + 8, ae as u32, ijy, 1, aia);
                    ae += 6;
                    for line in bh.ca.iter() {
                        if ae > lw + mg as i32 - 20 { break; }
                        self.en(cx + 8, ae, line, agb);
                        ae += gy;
                    }
                }
            },
            5 => { 
                self.en(cx, ae, "Vulnerability Scanner", agn);
                self.en(cx + 1, ae, "Vulnerability Scanner", agn);
                ae += gy + 8;
                
                if let Some((fcc, elo, nt)) = crate::network::aou() {
                    if let Some(at) = nt {
                        self.en(cx, ae, "Target:", adm);
                        self.en(cx + 70, ae, &alloc::format!("{}", at), agb);
                        ae += gy + 4;
                    }
                }
                
                self.en(cx, ae, "[V] Run Vulnerability Scan", adm);
                ae += gy + 8;
                
                if !bh.ca.is_empty() {
                    framebuffer::ah(zb + 8, ae as u32, ijy, 1, aia);
                    ae += 6;
                    for line in bh.ca.iter() {
                        if ae > lw + mg as i32 - 20 { break; }
                        let r = if line.contains("VULN") || line.contains("HIGH") { 0xFFDD3333u32 }
                            else if line.contains("WARN") || line.contains("MEDIUM") { 0xFFDDAA33u32 }
                            else { agb };
                        self.en(cx + 8, ae, line, r);
                        ae += gy;
                    }
                }
            },
            _ => {}
        }
    }
    
    
    fn ziz(&mut self) {
        if let Some(bh) = self.ee.el().du(|d| d.ld == WindowType::Gn) {
            bh.ca.clear();
            bh.ca.push(String::from("=== Settings ==="));
            bh.ca.push(String::from(""));
            bh.ca.push(format!("Resolution: {}x{}", self.z, self.ac));
            bh.ca.push(String::from("Theme: Dark Green"));
            bh.ca.push(String::from(""));
            bh.ca.push(String::from("--- Animations ---"));
            let gyq = if col() { "ON " } else { "OFF" };
            let kao = *GA_.lock();
            bh.ca.push(format!("[1] Animations: {}", gyq));
            bh.ca.push(format!("[2] Speed: {:.1}x", kao));
            bh.ca.push(String::from(""));
            bh.ca.push(String::from("--- Accessibility ---"));
            let bei = if crate::accessibility::edv() { "ON " } else { "OFF" };
            bh.ca.push(format!("[5] High Contrast: {}", bei));
            bh.ca.push(format!("[6] Font Size: {}", crate::accessibility::gid().cu()));
            bh.ca.push(format!("[7] Cursor Size: {}", crate::accessibility::gib().cu()));
            bh.ca.push(format!("[8] Sticky Keys: {}", if crate::accessibility::dsj() { "ON" } else { "OFF" }));
            bh.ca.push(format!("[9] Mouse Speed: {}", crate::accessibility::gig().cu()));
            bh.ca.push(String::from(""));
            bh.ca.push(String::from("--- Other ---"));
            bh.ca.push(String::from("[3] File Associations"));
            bh.ca.push(String::from("[4] About System"));
        }
    }
    
    
    fn tjq(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_};
        
        if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Pj) {
            let ojq = 4; 
            let hpx = bh.ca.len().ao(2);
            let liw = hpx.ao(ojq);
            
            if bs == V_ && bh.acm > 0 {
                bh.acm -= 1;
            } else if bs == U_ && bh.acm < liw.ao(1) {
                bh.acm += 1;
            } else if bs == 0x0D || bs == 0x0A {
                
                let w = ojq + bh.acm;
                if w < hpx {
                    
                    let line = &bh.ca[w];
                    if let Some(spr) = line.du('|') {
                        let wm = line[1..spr].em().tl('.');
                        
                        use crate::file_assoc::{Program, jpe, gih};
                        let cv = gih(&format!("test.{}", wm));
                        let next = match cv {
                            Program::Ag => Program::Bp,
                            Program::Bp => Program::Is,
                            Program::Is => Program::Ay,
                            Program::Ay => Program::Ag,
                            _ => Program::Ag,
                        };
                        jpe(wm, next.clone());
                        
                        crate::serial_println!("[ASSOC] {} -> {}", wm, next.j());
                    }
                }
            }
        }
    }
    
    
    fn rbg(&mut self) {
        if self.fwk > 0 {
            if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ay) {
                for _ in 0..self.fwk {
                    bh.ca.pop();
                }
            }
            self.fwk = 0;
        }
    }
    
    
    fn wnp(&mut self) {
        if self.xn.is_empty() {
            return;
        }
        let ewc = self.xn.as_str();
        let commands = crate::shell::AHV_;
        let oh: Vec<&str> = commands.iter().hu()
            .hi(|r| r.cj(ewc) && *r != ewc)
            .collect();
        if oh.is_empty() {
            return;
        }
        if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ay) {
            
            let display: Vec<&str> = oh.iter().hu().take(6).collect();
            let line = format!("  \x01M> {}", display.rr("  "));
            bh.ca.push(line);
            self.fwk = 1;
            
            if oh.len() > 6 {
                bh.ca.push(format!("    \x01M... +{} more", oh.len() - 6));
                self.fwk = 2;
            }
        }
    }
    
    
    fn csi(cif: &str) -> String {
        let os = crate::rtc::cgz();
        let jv = crate::ramfs::fh(|fs| {
            let ai = fs.dau();
            String::from(ai)
        });
        let ryn = if jv == "/" { String::from("~") } else { jv };
        format!("\x01B[{:02}:{:02}:{:02}] \x01Rroot\x01M@trustos\x01M:\x01B{}\x01M$ \x01G{}", os.bek, os.bri, os.chr, ryn, cif)
    }
    
    
    fn tlj(&mut self, bs: u8) {
        use crate::keyboard::{AM_, AQ_};
        
        self.rbg();
        
        
        if bs == AM_ || bs == AQ_ {
            if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ay) {
                let acg = 16usize;
                let ffq = (bh.ac as usize).ao(J_ as usize + 16);
                let act = if acg > 0 { ffq / acg } else { 1 };
                let aye = bh.ca.len().ao(act);
                if bs == AM_ {
                    bh.px = bh.px.ao(act);
                } else {
                    bh.px = (bh.px + act).v(aye);
                }
            }
            return;
        }
        
        if bs == 0x08 { 
            if !self.xn.is_empty() {
                self.xn.pop();
                if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ay) {
                    if let Some(qv) = bh.ca.dsq() {
                        *qv = Self::csi(&format!("{}_", self.xn));
                    }
                }
            }
        } else if bs == 0x09 { 
            let ewc = self.xn.clone();
            if !ewc.is_empty() {
                
                if let Some(pmd) = ewc.bhx(' ') {
                    
                    let nto = &ewc[pmd + 1..];
                    if !nto.is_empty() {
                        
                        let mut hjg: Vec<String> = Vec::new();
                        if let Ok(ch) = crate::ramfs::fh(|fs| fs.awb(Some("/"))) {
                            for (j, are, dds) in ch.iter() {
                                let hsk: String = j.bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
                                let veo: String = nto.bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
                                if hsk.cj(&veo) {
                                    let cif = if *are == crate::ramfs::FileType::K { "/" } else { "" };
                                    hjg.push(format!("{}{}", j, cif));
                                }
                            }
                        }
                        if hjg.len() == 1 {
                            let rhf = &ewc[..=pmd];
                            self.xn = format!("{}{}", rhf, hjg[0]);
                            if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ay) {
                                if let Some(qv) = bh.ca.dsq() {
                                    *qv = Self::csi(&format!("{}_", self.xn));
                                }
                            }
                        } else if hjg.len() > 1 {
                            let lkm: String = hjg.rr("  ");
                            if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ay) {
                                bh.ca.push(lkm);
                                bh.ca.push(Self::csi(&format!("{}_", self.xn)));
                            }
                        }
                    }
                } else {
                    
                    let commands = crate::shell::AHV_;
                    let vep = ewc.as_str();
                    let oh: Vec<&str> = commands.iter().hu().hi(|r| r.cj(vep)).collect();
                    if oh.len() == 1 {
                        self.xn = String::from(oh[0]);
                        if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ay) {
                            if let Some(qv) = bh.ca.dsq() {
                                *qv = Self::csi(&format!("{}_", self.xn));
                            }
                        }
                    } else if oh.len() > 1 {
                        let lkm: String = oh.rr("  ");
                        if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ay) {
                            bh.ca.push(lkm);
                            bh.ca.push(Self::csi(&format!("{}_", self.xn)));
                        }
                    }
                }
            }
        } else if bs == 0xF0 { 
            if !self.bqa.is_empty() {
                match self.ari {
                    None => {
                        
                        self.hyr = self.xn.clone();
                        let w = self.bqa.len() - 1;
                        self.ari = Some(w);
                        self.xn = self.bqa[w].clone();
                    }
                    Some(a) if a > 0 => {
                        let w = a - 1;
                        self.ari = Some(w);
                        self.xn = self.bqa[w].clone();
                    }
                    _ => {} 
                }
                if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ay) {
                    if let Some(qv) = bh.ca.dsq() {
                        *qv = Self::csi(&format!("{}_", self.xn));
                    }
                }
            }
        } else if bs == 0xF1 { 
            if let Some(a) = self.ari {
                if a + 1 < self.bqa.len() {
                    let w = a + 1;
                    self.ari = Some(w);
                    self.xn = self.bqa[w].clone();
                } else {
                    
                    self.ari = None;
                    self.xn = self.hyr.clone();
                }
                if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ay) {
                    if let Some(qv) = bh.ca.dsq() {
                        *qv = Self::csi(&format!("{}_", self.xn));
                    }
                }
            }
        } else if bs == 0x0D || bs == 0x0A { 
            let cmd = self.xn.clone();
            self.xn.clear();
            
            if !cmd.em().is_empty() {
                let ksa = self.bqa.qv().map(|i| i == &cmd).unwrap_or(false);
                if !ksa {
                    self.bqa.push(cmd.clone());
                }
            }
            self.ari = None;
            self.hyr.clear();
            
            let an = Self::sok(&cmd);
            
            
            let nei = cmd.em();
            if nei.cj("play ") {
                let ji = nei.blj("play ").unwrap_or("").em();
                match ji {
                    "u2" | "untitled2" | "lofi" | "untitled" => {
                        
                        let gne = self.z.ao(320) as i32;
                        let gnf = self.ac.ao(W_ + 600) as i32;
                        let ajq = self.xl("Music Player", gne, gnf.am(20), 320, 580, WindowType::Lw);
                        if let Some(upz) = self.ano.ds(&ajq) {
                            upz.dkp(0);
                        }
                    },
                    _ => {},
                }
            }
            
            if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ay) {
                
                if cmd.em() == "clear" {
                    bh.ca.clear();
                    bh.ca.push(Self::csi("_"));
                    bh.px = 0;
                } else {
                    
                    if bh.ca.qv().map(|e| e.contains("$ ")).unwrap_or(false) {
                        bh.ca.pop();
                    }
                    
                    bh.ca.push(Self::csi(&cmd));
                    
                    for line in an {
                        bh.ca.push(line);
                    }
                    
                    
                    bh.ca.push(Self::csi("_"));
                    
                    
                    let acg = 16usize;
                    let ffq = (bh.ac as usize).ao(J_ as usize + 16);
                    let act = if acg > 0 { ffq / acg } else { 1 };
                    if bh.ca.len() > act {
                        bh.px = bh.ca.len() - act;
                    } else {
                        bh.px = 0;
                    }
                }
            }
        } else if bs >= 0x20 && bs < 0x7F {
            self.xn.push(bs as char);
            
            if let Some(bh) = self.ee.el().du(|d| d.ja && d.ld == WindowType::Ay) {
                if let Some(qv) = bh.ca.dsq() {
                    *qv = Self::csi(&format!("{}_", self.xn));
                }
            }
        }
        
        
        self.wnp();
    }
    
    
    fn sok(cmd: &str) -> Vec<String> {
        let mut an = Vec::new();
        let cmd = cmd.em();
        
        
        crate::serial_println!("[TERM] Executing command: '{}' len={}", cmd, cmd.len());
        
        if cmd.is_empty() {
            return an;
        }
        
        match cmd {
            "help" => {
                an.push(String::from("\x01HTrustOS Terminal \x01M- Available Commands"));
                an.push(String::from(""));
                
                an.push(String::from("\x01Y[File System]"));
                an.push(String::from("  \x01Gls \x01B[dir]      \x01WList directory contents"));
                an.push(String::from("  \x01Gcd \x01B<dir>      \x01WChange directory"));
                an.push(String::from("  \x01Gpwd            \x01WPrint working directory"));
                an.push(String::from("  \x01Gcat \x01B<file>    \x01WShow file contents"));
                an.push(String::from("  \x01Gmkdir \x01B<name>  \x01WCreate directory"));
                an.push(String::from("  \x01Gtouch \x01B<name>  \x01WCreate empty file"));
                an.push(String::from("  \x01Grm \x01B<file>     \x01WRemove file"));
                an.push(String::from("  \x01Gcp \x01B<src> <dst>\x01WCopy file"));
                an.push(String::from("  \x01Gmv \x01B<src> <dst>\x01WMove/rename file"));
                an.push(String::from("  \x01Gtree \x01B[path]   \x01WDirectory tree"));
                an.push(String::from("  \x01Gfind \x01B<p> <n>  \x01WSearch files by name"));
                an.push(String::from("  \x01Gstat \x01B<file>   \x01WFile metadata"));
                an.push(String::from(""));
                
                an.push(String::from("\x01Y[System]"));
                an.push(String::from("  \x01Gdate           \x01WCurrent date and time"));
                an.push(String::from("  \x01Guname          \x01WSystem information"));
                an.push(String::from("  \x01Gfree           \x01WMemory usage"));
                an.push(String::from("  \x01Gps             \x01WList processes"));
                an.push(String::from("  \x01Guptime         \x01WSystem uptime"));
                an.push(String::from("  \x01Gdf             \x01WDisk usage"));
                an.push(String::from("  \x01Gwhoami         \x01WCurrent user"));
                an.push(String::from("  \x01Ghostname       \x01WSystem hostname"));
                an.push(String::from("  \x01Gneofetch       \x01WSystem info banner"));
                an.push(String::from("  \x01Ghistory        \x01WCommand history"));
                an.push(String::from("  \x01Genv            \x01WEnvironment variables"));
                an.push(String::from("  \x01Gdmesg          \x01WKernel log messages"));
                an.push(String::from(""));
                
                an.push(String::from("\x01Y[Network]"));
                an.push(String::from("  \x01Gnet            \x01WNetwork interface status"));
                an.push(String::from("  \x01Gifconfig       \x01WNetwork configuration"));
                an.push(String::from("  \x01Gping \x01B<host>   \x01WICMP echo test"));
                an.push(String::from("  \x01Gcurl \x01B<url>    \x01WHTTP client"));
                an.push(String::from("  \x01Gnslookup \x01B<h>  \x01WDNS lookup"));
                an.push(String::from("  \x01Gnetstat        \x01WActive connections"));
                an.push(String::from(""));
                
                an.push(String::from("\x01Y[Graphics & Demos]"));
                an.push(String::from("  \x01Gshader \x01B<name>  \x01WGPU shader (plasma/fire/tunnel...)"));
                an.push(String::from("  \x01Gmatrix3d       \x01W3D Matrix tunnel"));
                an.push(String::from("  \x01Gshowcase3d     \x01W3D cinematic demo"));
                an.push(String::from("  \x01Gfilled3d       \x01WFilled 3D test"));
                an.push(String::from("  \x01Gchess          \x01WChess game vs AI"));
                an.push(String::from("  \x01Gchess3d        \x01W3D chess (Matrix style)"));
                an.push(String::from("  \x01Ggameboy        \x01WGame Boy emulator"));
                an.push(String::from("  \x01Gmatrix         \x01WMatrix rain animation"));
                an.push(String::from("  \x01Gneofetch       \x01WASCII system info"));
                an.push(String::from(""));
                
                an.push(String::from("\x01Y[Audio]"));
                an.push(String::from("  \x01Gplay \x01B<track>  \x01WPlay music (u2, lofi)"));
                an.push(String::from("  \x01Gsynth \x01B<cmd>   \x01WPolyphonic synthesizer"));
                an.push(String::from("  \x01Gdaw \x01B<cmd>     \x01WDigital audio workstation"));
                an.push(String::from("  \x01Gbeep \x01B[hz] [ms]\x01WPlay a tone"));
                an.push(String::from(""));
                
                an.push(String::from("\x01Y[AI / Jarvis]"));
                an.push(String::from("  \x01Gjarvis \x01B<cmd>  \x01WAI assistant (chat/brain/hw)"));
                an.push(String::from("  \x01Gj \x01B<query>     \x01WJarvis shortcut"));
                an.push(String::from(""));
                
                an.push(String::from("\x01Y[Text Processing]"));
                an.push(String::from("  \x01Ggrep \x01B<pat> <f>\x01WSearch pattern in file"));
                an.push(String::from("  \x01Gsort \x01B<file>   \x01WSort lines"));
                an.push(String::from("  \x01Gdiff \x01B<a> <b>  \x01WCompare files"));
                an.push(String::from("  \x01Ghexdump \x01B<f>   \x01WHex dump file"));
                an.push(String::from(""));
                
                an.push(String::from("\x01Y[Shell]"));
                an.push(String::from("  \x01Ghelp           \x01WShow this help"));
                an.push(String::from("  \x01Gecho \x01B<text>   \x01WPrint text"));
                an.push(String::from("  \x01Gclear          \x01WClear terminal"));
                an.push(String::from("  \x01Gexit           \x01WClose terminal"));
                an.push(String::from(""));
                an.push(String::from("\x01M  All boot shell commands also available (220+)"));
            },
            
            "matrix3d" | "tunnel" | "holomatrix" | "3d" => {
                an.push(String::from("✓ Matrix Tunnel 3D - ESC to exit"));
                
                
                let pq = crate::framebuffer::kyq();
                let z = crate::framebuffer::z();
                let ac = crate::framebuffer::ac();
                
                
                crate::gpu_emu::init(pq, z, ac);
                if let Some(fuo) = crate::gpu_emu::kyx("tunnel") {
                    crate::gpu_emu::hzy(fuo);
                }
                
                let mut vj = 0u32;
                loop {
                    if let Some(bs) = crate::keyboard::xw() {
                        if bs == 27 { break; }
                    }
                    
                    #[cfg(target_arch = "x86_64")]
                    crate::gpu_emu::krk();
                    #[cfg(not(target_arch = "x86_64"))]
                    crate::gpu_emu::po();
                    
                    crate::gpu_emu::or(16);
                    vj += 1;
                    
                    if vj % 60 == 0 {
                        crate::framebuffer::cb("MATRIX 3D TUNNEL | ESC=exit", 10, 10, 0xFF00FF00);
                    }
                }
                an.push(format!("Tunnel ended ({} frames)", vj));
            },
            "ls" => {
                let jv = crate::ramfs::fh(|fs| String::from(fs.dau()));
                an.push(format!("\x01MDirectory: \x01B{}", jv));
                if let Ok(ch) = crate::ramfs::fh(|fs| fs.awb(None)) {
                    for (j, are, aw) in ch.iter().take(20) {
                        let pa = if *are == crate::ramfs::FileType::K { "\x01B" } else { "\x01M" };
                        let bde = if *are == crate::ramfs::FileType::K { "/" } else { "" };
                        an.push(format!("  {}{}{}  \x01M{} bytes", pa, j, bde, aw));
                    }
                    if ch.is_empty() {
                        an.push(String::from("\x01M  (empty directory)"));
                    }
                }
            },
            "ls /" => {
                an.push(String::from("\x01MDirectory: \x01B/"));
                if let Ok(ch) = crate::ramfs::fh(|fs| fs.awb(Some("/"))) {
                    for (j, are, aw) in ch.iter().take(20) {
                        let pa = if *are == crate::ramfs::FileType::K { "\x01B" } else { "\x01M" };
                        let bde = if *are == crate::ramfs::FileType::K { "/" } else { "" };
                        an.push(format!("  {}{}{}  \x01M{} bytes", pa, j, bde, aw));
                    }
                    if ch.is_empty() {
                        an.push(String::from("\x01M  (empty directory)"));
                    }
                }
            },
            _ if cmd.cj("ls ") => {
                let path = &cmd[3..];
                an.push(format!("\x01MDirectory: \x01B{}", path));
                if let Ok(ch) = crate::ramfs::fh(|fs| fs.awb(Some(path))) {
                    for (j, are, aw) in ch.iter().take(20) {
                        let pa = if *are == crate::ramfs::FileType::K { "\x01B" } else { "\x01M" };
                        let bde = if *are == crate::ramfs::FileType::K { "/" } else { "" };
                        an.push(format!("  {}{}{}  \x01M{} bytes", pa, j, bde, aw));
                    }
                    if ch.is_empty() {
                        an.push(String::from("\x01M  (empty directory)"));
                    }
                } else {
                    an.push(format!("\x01Rls: cannot access '{}': No such file or directory", path));
                }
            },
            "pwd" => {
                let jv = crate::ramfs::fh(|fs| String::from(fs.dau()));
                an.push(format!("\x01B{}", jv));
            },
            "clear" => {
                
            },
            "date" | "time" => {
                let os = crate::rtc::cgz();
                an.push(format!("\x01B{:04}-{:02}-{:02} \x01W{:02}:{:02}:{:02}", 
                    os.ccq, os.caw, os.cjw, os.bek, os.bri, os.chr));
            },
            "uname" | "uname -a" | "version" => {
                an.push(String::from("\x01GTrustOS \x01W0.1.1 \x01Bx86_64 \x01MRust Kernel"));
                an.push(format!("\x01MHeap: \x01W{} MB", crate::memory::cre() / 1024 / 1024));
            },
            "neofetch" => {
                an.push(String::from("\x01G  _____               _    ___  ___"));
                an.push(String::from("\x01G |_   _| __ _   _ ___| |_ / _ \\/ __|"));
                an.push(String::from("\x01G   | || '__| | | / __| __| | | \\__ \\"));
                an.push(String::from("\x01G   | || |  | |_| \\__ \\ |_| |_| |__) |"));
                an.push(String::from("\x01G   |_||_|   \\__,_|___/\\__|\\___/|___/"));
                an.push(String::from(""));
                an.push(String::from("\x01BOS\x01M:      \x01WTrustOS 0.1.1"));
                an.push(String::from("\x01BKernel\x01M:  \x01Wtrustos_kernel"));
                an.push(String::from("\x01BArch\x01M:    \x01Wx86_64"));
                an.push(format!("\x01BUptime\x01M:  \x01W{}m {}s", crate::logger::lh() / 100 / 60, (crate::logger::lh() / 100) % 60));
                an.push(format!("\x01BMemory\x01M:  \x01W{} MB", crate::memory::cre() / 1024 / 1024));
                an.push(format!("\x01BShell\x01M:   \x01Wtrustsh"));
                an.push(format!("\x01BDisplay\x01M: \x01W{}x{}", crate::framebuffer::z(), crate::framebuffer::ac()));
            },
            "whoami" | "user" | "users" | "id" => {
                an.push(String::from("\x01Groot"));
            },
            "hostname" => {
                an.push(String::from("\x01Gtrustos"));
            },
            "history" => {
                
                let crg = crate::desktop::Aa.lock().bqa.clone();
                if crg.is_empty() {
                    an.push(String::from("\x01M  (no history yet)"));
                } else {
                    for (a, bt) in crg.iter().cf() {
                        an.push(format!("\x01M  {}  {}", a + 1, bt));
                    }
                }
            },
            "free" | "mem" => {
                let tod = crate::memory::cre() / 1024 / 1024;
                an.push(String::from("\x01YMemory Usage:"));
                an.push(format!("  \x01BHeap Size: \x01W{} MB", tod));
                an.push(String::from("  \x01BKernel:   \x01GActive"));
            },
            "net" | "ifconfig" | "ip" | "ipconfig" => {
                an.push(String::from("\x01YNetwork Status:"));
                if crate::network::anl() {
                    if let Some((ed, ip, gxl)) = crate::network::gif() {
                        an.push(format!("  \x01BMAC: \x01W{}", ed));
                        if let Some(ip) = ip {
                            an.push(format!("  \x01BIP:  \x01W{}", ip));
                        }
                        an.push(String::from("  \x01BStatus: \x01GConnected"));
                    }
                } else {
                    an.push(String::from("  \x01BStatus: \x01RNo network"));
                }
            },
            _ if cmd.cj("cat ") => {
                let it = &cmd[4..].em();
                if let Ok(ca) = crate::ramfs::fh(|fs| fs.mq(it).map(|bc| bc.ip())) {
                    if let Ok(text) = core::str::jg(&ca) {
                        for line in text.ak().take(20) {
                            an.push(String::from(line));
                        }
                    } else {
                        an.push(format!("cat: {}: binary file", it));
                    }
                } else {
                    an.push(format!("cat: {}: No such file", it));
                }
            },
            _ if cmd.cj("echo ") => {
                an.push(String::from(&cmd[5..]));
            },
            "cd" => {
                
                let _ = crate::ramfs::fh(|fs| fs.fem("/"));
            },
            _ if cmd.cj("cd ") => {
                let path = &cmd[3..].em();
                match crate::ramfs::fh(|fs| fs.fem(path)) {
                    Ok(()) => {
                        let jv = crate::ramfs::fh(|fs| String::from(fs.dau()));
                        an.push(format!("\x01B{}", jv));
                    },
                    Err(aa) => an.push(format!("\x01Rcd: {}: {}", path, aa.as_str())),
                }
            },
            _ if cmd.cj("mkdir ") => {
                let path = cmd[6..].em();
                match crate::ramfs::fh(|fs| fs.ut(path)) {
                    Ok(()) => an.push(format!("\x01Gmkdir: \x01Wcreated '\x01B{}\x01W'", path)),
                    Err(aa) => an.push(format!("\x01Rmkdir: {}: {}", path, aa.as_str())),
                }
            },
            _ if cmd.cj("touch ") => {
                let path = cmd[6..].em();
                match crate::ramfs::fh(|fs| fs.touch(path)) {
                    Ok(()) => an.push(format!("\x01Gtouch: \x01Wcreated '\x01B{}\x01W'", path)),
                    Err(aa) => an.push(format!("\x01Rtouch: {}: {}", path, aa.as_str())),
                }
            },
            _ if cmd.cj("rm ") || cmd.cj("del ") => {
                let path = if cmd.cj("rm ") { cmd[3..].em() } else { cmd[4..].em() };
                match crate::ramfs::fh(|fs| fs.hb(path)) {
                    Ok(()) => an.push(format!("\x01Grm: \x01Wremoved '\x01B{}\x01W'", path)),
                    Err(aa) => an.push(format!("\x01Rrm: {}: {}", path, aa.as_str())),
                }
            },
            "shader" | "vgpu" => {
                an.push(String::from("╔═══════════════════════════════════════╗"));
                an.push(String::from("║     Virtual GPU - Shader Demo         ║"));
                an.push(String::from("╠═══════════════════════════════════════╣"));
                an.push(String::from("║ shader plasma    - Plasma waves       ║"));
                an.push(String::from("║ shader fire      - Fire effect        ║"));
                an.push(String::from("║ shader mandelbrot- Fractal zoom       ║"));
                an.push(String::from("║ shader matrix    - Matrix rain        ║"));
                an.push(String::from("║ shader tunnel    - 3D HOLOMATRIX      ║"));
                an.push(String::from("║ shader shapes    - 3D OBJECTS         ║"));
                an.push(String::from("║ shader parallax  - Depth layers       ║"));
                an.push(String::from("║ shader gradient  - Test gradient      ║"));
                an.push(String::from("╚═══════════════════════════════════════╝"));
                an.push(String::from("Press ESC to exit shader demo"));
            },
            _ if cmd.cj("shader ") => {
                let dvv = cmd.tl("shader ").em();
                if let Some(fuo) = crate::gpu_emu::kyx(dvv) {
                    an.push(format!("✓ Starting shader: {} (ESC to exit)", dvv));
                    
                    
                    let pq = crate::framebuffer::kyq();
                    let z = crate::framebuffer::z();
                    let ac = crate::framebuffer::ac();
                    
                    
                    crate::gpu_emu::init(pq, z, ac);
                    crate::gpu_emu::hzy(fuo);
                    
                    
                    let mut vj = 0u32;
                    
                    loop {
                        
                        if let Some(bs) = crate::keyboard::xw() {
                            if bs == 27 { break; }
                        }
                        
                        
                        #[cfg(target_arch = "x86_64")]
                        crate::gpu_emu::krk();
                        #[cfg(not(target_arch = "x86_64"))]
                        crate::gpu_emu::po();
                        
                        
                        crate::gpu_emu::or(16);
                        vj += 1;
                        
                        
                        if vj % 60 == 0 {
                            crate::framebuffer::cb(&format!("FPS: ~60 | {} | ESC=exit", dvv), 10, 10, 0xFFFFFFFF);
                        }
                    }
                    
                    an.push(format!("Shader ended ({} frames)", vj));
                } else {
                    an.push(format!("Unknown shader: {}", dvv));
                    an.push(String::from("Available: plasma, fire, mandelbrot, matrix, tunnel, parallax, gradient"));
                }
            },
            "ps" | "procs" | "top" => {
                an.push(String::from("\x01BPID  \x01BSTATE    \x01BNAME"));
                an.push(String::from("  \x01W1  \x01GRunning  \x01Winit"));
                an.push(String::from("  \x01W2  \x01GRunning  \x01Wdesktop"));
                an.push(String::from("  \x01W3  \x01GRunning  \x01Wterminal"));
            },
            "uptime" => {
                let qb = crate::logger::lh();
                let tv = qb / 100;
                let bbz = tv / 60;
                an.push(format!("\x01BUptime: \x01W{}m {}s", bbz, tv % 60));
            },
            "df" | "lsblk" => {
                an.push(String::from("\x01BFilesystem      Size  Used  Avail Use%"));
                an.push(String::from("\x01Wramfs           32M   1M    31M   3%"));
            },
            "showcase3d" | "demo3d" => {
                an.push(String::from("\u{2713} Showcase 3D Cinematic - ESC to skip scenes"));
                drop(an);
                crate::shell::desktop::neh();
                return Vec::new();
            },
            "filled3d" => {
                an.push(String::from("\u{2713} Filled 3D Test - ESC to exit"));
                drop(an);
                crate::shell::desktop::ndy();
                return Vec::new();
            },
            "exit" | "quit" => {
                an.push(String::from("\x01MUse the X button to close the terminal"));
            },
            "desktop" | "gui" | "mobile" => {
                an.push(String::from("\x01MAlready in desktop mode."));
            },
            "chess" => {
                an.push(String::from("\x01G\u{265A} TrustChess \x01M— Opening chess window..."));
                an.push(String::from("\x01MPlay vs AI (Black). Arrow keys, Enter, Esc."));
            },
            "chess3d" => {
                an.push(String::from("\x01G\u{265A} TrustChess 3D \x01M— Opening 3D chess window..."));
                an.push(String::from("\x01MWASD:Camera  ZX:Zoom  O:Auto-rotate  Click:Move"));
            },
            "gameboy" | "gb" => {
                an.push(String::from("\x01G\u{1F3AE} Game Boy \x01M— Opening Game Boy window..."));
                an.push(String::from("\x01MWASD:D-Pad X/Space:A Z:B C:Select Enter:Start"));
            },
            _ if cmd.cj("play ") || cmd == "play" => {
                let ji = cmd.blj("play ").unwrap_or("").em();
                if ji.is_empty() {
                    an.push(String::from("\x01Y\u{266B} Usage: \x01Gplay u2"));
                    an.push(String::from("\x01MTracks: u2, untitled2, lofi"));
                } else {
                    match ji {
                        "u2" | "untitled2" | "lofi" | "untitled" => {
                            an.push(String::from("\x01G\u{266B} Playing Untitled (2) — Lo-Fi"));
                            an.push(String::from("\x01MOpening Music Player widget..."));
                            
                        },
                        _ => {
                            an.push(format!("\x01RTrack not found: \x01W{}", ji));
                            an.push(String::from("\x01MAvailable: u2, untitled2, lofi"));
                        },
                    }
                }
            },
            _ if cmd.cj("j ") || cmd.cj("jarvis ") || cmd == "j" || cmd == "jarvis" => {
                
                if UH_.load(core::sync::atomic::Ordering::SeqCst) {
                    an.push(String::from("\x01Y[Jarvis] \x01MStill thinking... please wait."));
                } else {
                    UH_.store(true, core::sync::atomic::Ordering::SeqCst);
                    {
                        let mut aln = AXN_.lock();
                        *aln = Some(String::from(cmd));
                    }
                    crate::thread::jqu("jarvis-bg", uad, 0);
                    an.push(String::from("\x01Y[Jarvis] \x01M\u{1F4AD} Thinking..."));
                }
            },
            _ => {
                
                
                crate::shell::jsk(); 
                crate::shell::DE_.store(true, core::sync::atomic::Ordering::SeqCst);
                crate::shell::azu(cmd);
                crate::shell::DE_.store(false, core::sync::atomic::Ordering::SeqCst);
                let bjm = crate::shell::jsk();
                if !bjm.is_empty() {
                    for line in bjm.ak() {
                        an.push(String::from(line));
                    }
                }
            },
        }
        
        an
    }
    
    
    pub fn tkj(&mut self, b: i32, c: i32) {
        self.lf = b.qp(0, self.z as i32 - 1);
        self.ot = c.qp(0, self.ac as i32 - 1);
        
        
        if self.eaz.is_some() {
            self.pxb(b, c);
        }
        
        for d in &mut self.ee {
            
            if d.cka && !d.bkk {
                d.b = (b - d.dgp).am(0).v(self.z as i32 - 50);
                d.c = (c - d.dgq).am(0).v(self.ac as i32 - W_ as i32 - J_ as i32);
                
                
                let epq = 16i32;
                let kp = self.z as i32;
                let kl = (self.ac - W_) as i32;
                let wp = kl / 2;
                
                if b <= epq && c <= epq + wp / 4 {
                    self.dwc = Some(SnapDir::Dp);
                } else if b <= epq && c >= kl - wp / 4 {
                    self.dwc = Some(SnapDir::Dt);
                } else if b <= epq {
                    self.dwc = Some(SnapDir::Ap);
                } else if b >= kp - epq && c <= epq + wp / 4 {
                    self.dwc = Some(SnapDir::Dq);
                } else if b >= kp - epq && c >= kl - wp / 4 {
                    self.dwc = Some(SnapDir::Du);
                } else if b >= kp - epq {
                    self.dwc = Some(SnapDir::Ca);
                } else {
                    self.dwc = None;
                }
            }
            
            
            if d.dlg != ResizeEdge::None {
                let dx = b - d.dgp;
                let bg = c - d.dgq;
                
                
                match d.dlg {
                    ResizeEdge::Ca | ResizeEdge::Du | ResizeEdge::Dq => {
                        let cst = (d.z as i32 + dx).am(d.czx as i32) as u32;
                        d.z = cst.v(self.z - d.b as u32);
                        d.dgp = b;
                    }
                    _ => {}
                }
                
                
                match d.dlg {
                    ResizeEdge::Ap | ResizeEdge::Dt | ResizeEdge::Dp => {
                        let cst = (d.z as i32 - dx).am(d.czx as i32) as u32;
                        if cst != d.z as u32 {
                            d.b += (d.z as i32 - cst as i32);
                            d.z = cst;
                        }
                        d.dgp = b;
                    }
                    _ => {}
                }
                
                
                match d.dlg {
                    ResizeEdge::Hk | ResizeEdge::Du | ResizeEdge::Dt => {
                        let csr = (d.ac as i32 + bg).am(d.dtg as i32) as u32;
                        d.ac = csr.v(self.ac - W_ - d.c as u32);
                        d.dgq = c;
                    }
                    _ => {}
                }
                
                
                match d.dlg {
                    ResizeEdge::Jd | ResizeEdge::Dp | ResizeEdge::Dq => {
                        let csr = (d.ac as i32 - bg).am(d.dtg as i32) as u32;
                        if csr != d.ac as u32 {
                            d.c += (d.ac as i32 - csr as i32);
                            d.ac = csr;
                        }
                        d.dgq = c;
                    }
                    _ => {}
                }
            }
        }
        
        
        let lmj: Option<(u32, i32, i32, u32, u32)> = self.ee.iter()
            .du(|d| d.ja && !d.aat && d.ld == WindowType::Fp)
            .map(|d| (d.ad, d.b, d.c, d.z, d.ac));
        if let Some((nr, fx, lw, hk, mg)) = lmj {
            let fp = b - fx;
            let iz = c - lw - J_ as i32;
            let gm = hk as usize;
            let me = mg.ao(J_) as usize;
            if let Some(g) = self.djq.ds(&nr) {
                g.lax(fp, iz, gm, me);
            }
        }
        
        
        let raf: Option<u32> = self.ee.iter()
            .du(|d| d.ja && !d.aat && d.ld == WindowType::Gs)
            .map(|d| d.ad);
        if let Some(nr) = raf {
            if let Some(chess) = self.dou.ds(&nr) {
                if chess.dgo.is_some() {
                    chess.pxc(b, c);
                }
            }
        }
        
        
        let khi: Option<(u32, i32, i32)> = self.ee.iter()
            .du(|d| d.ja && !d.aat && d.ld == WindowType::Ih)
            .map(|d| (d.ad, d.b, d.c));
        if let Some((nr, fx, lw)) = khi {
            if let Some(g) = self.cwd.ds(&nr) {
                let amr = b - fx;
                let aio = c - lw - J_ as i32;
                g.lax(amr, aio);
            }
        }
    }
    
    
    pub fn ers(&mut self, aaq: i8) {
        
        let lmj = self.ee.iter().vv().du(|d| d.ja && !d.aat && d.ld == WindowType::Fp).map(|d| d.ad);
        if let Some(nr) = lmj {
            if let Some(g) = self.djq.ds(&nr) {
                g.ers(aaq);
            }
            return;
        }
        
        let khi = self.ee.iter().vv().du(|d| d.ja && !d.aat && d.ld == WindowType::Ih).map(|d| d.ad);
        if let Some(nr) = khi {
            if let Some(g) = self.cwd.ds(&nr) {
                g.ers(aaq);
            }
            return;
        }
        
        if let Some(bh) = self.ee.el().vv().du(|d| d.ja && !d.aat) {
            match bh.ld {
                WindowType::Ak | WindowType::Ag | WindowType::Is | 
                WindowType::Pj | WindowType::Ay => {
                    let aye = if bh.ca.len() > 10 {
                        bh.ca.len() - 10
                    } else {
                        0
                    };
                    
                    if aaq > 0 {
                        
                        if bh.px > 0 {
                            bh.px = bh.px.ao(3);
                        }
                    } else if aaq < 0 {
                        
                        bh.px = (bh.px + 3).v(aye);
                    }
                },
                _ => {}
            }
        }
    }
    
    
    
    pub fn vmu(&mut self) {
        
        self.hle.clear();
        self.kye.vml(&mut self.hle);
        
        
        if !self.hle.is_empty() {
            self.pvl = true; 
        }
        
        
        let mut cqv: [(u8, i32, i32, i32, i32, i32); 8] = [(0, 0, 0, 0, 0, 0); 8];
        let mut cex = 0usize;
        
        for gesture in self.hle.iter() {
            if cex >= 8 { break; }
            
            match gesture {
                crate::gesture::GestureEvent::Bty { b, c } => {
                    cqv[cex] = (1, *b, *c, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Bey { b, c } => {
                    cqv[cex] = (2, *b, *c, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Blp { b, c } => {
                    cqv[cex] = (3, *b, *c, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Btt { sz, ql, vc, cqe, hic, .. } => {
                    let kpy = match sz {
                        crate::gesture::SwipeDirection::Ap => 0,
                        crate::gesture::SwipeDirection::Ca => 1,
                        crate::gesture::SwipeDirection::Ek => 2,
                        crate::gesture::SwipeDirection::Fm => 3,
                    };
                    cqv[cex] = (4, *ql, *vc, *cqe, *hic, kpy);
                }
                crate::gesture::GestureEvent::Abp { atf, li } => {
                    let uzf = match atf {
                        crate::gesture::EdgeOrigin::Hk => 0,
                        crate::gesture::EdgeOrigin::Jd => 1,
                        crate::gesture::EdgeOrigin::Ap => 2,
                        crate::gesture::EdgeOrigin::Ca => 3,
                    };
                    cqv[cex] = (5, uzf, *li, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Boz { yv, uq, bv } => {
                    cqv[cex] = (6, *yv, *uq, *bv, 0, 0);
                }
                crate::gesture::GestureEvent::Yq { iqw, iqx } => {
                    cqv[cex] = (7, *iqw, *iqx, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Bui { sz } => {
                    let kpy = match sz {
                        crate::gesture::SwipeDirection::Ap => 0,
                        crate::gesture::SwipeDirection::Ca => 1,
                        crate::gesture::SwipeDirection::Ek => 2,
                        crate::gesture::SwipeDirection::Fm => 3,
                    };
                    cqv[cex] = (8, kpy, 0, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Bum { b, c } => {
                    cqv[cex] = (9, *b, *c, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Bun { b, c } => {
                    cqv[cex] = (10, *b, *c, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Qy { b, c } => {
                    cqv[cex] = (11, *b, *c, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Arf { b, c, ql, vc } => {
                    cqv[cex] = (12, *b, *c, *ql, *vc, 0);
                }
            }
            cex += 1;
        }
        
        
        for a in 0..cex {
            let (tht, q, o, r, bc, aa) = cqv[a];
            match tht {
                1 => { 
                    self.fas(q, o);
                    self.ago(q, o, true);
                    self.ago(q, o, false);
                }
                2 => { 
                    self.fas(q, o);
                    self.ago(q, o, true);
                    self.ago(q, o, false);
                    self.ago(q, o, true);
                    self.ago(q, o, false);
                }
                3 => { 
                    self.fas(q, o);
                    self.hmk(q, o, true);
                    self.hmk(q, o, false);
                }
                4 => { 
                    match aa {
                        0 => {  }
                        1 => {  }
                        2 => {  }
                        3 => {  }
                        _ => {}
                    }
                }
                5 => { 
                    match q {
                        0 => { 
                            if !self.ajo {
                                self.ajo = true;
                            }
                        }
                        1 => { 
                        }
                        _ => {}
                    }
                }
                6 => { 
                    
                    
                }
                7 => { 
                    let jnw = if o > 0 { -1i8 } else if o < 0 { 1i8 } else { 0i8 };
                    if jnw != 0 {
                        self.ers(jnw);
                    }
                }
                8 => { 
                    
                    self.rss();
                }
                9 => { 
                    self.fas(q, o);
                }
                10 => { 
                    self.fas(q, o);
                }
                11 => { 
                    
                }
                12 => { 
                    self.fas(q, o);
                }
                _ => {}
            }
        }
    }
    
    
    fn fas(&mut self, b: i32, c: i32) {
        self.lf = b.qp(0, self.z as i32 - 1);
        self.ot = c.qp(0, self.ac as i32 - 1);
    }
    
    
    fn rss(&mut self) {
        if self.ee.len() < 2 {
            return;
        }
        
        let svj = self.ee.iter().qf(|d| d.ja);
        if let Some(w) = svj {
            let next = (w + 1) % self.ee.len();
            for d in self.ee.el() {
                d.ja = false;
            }
            self.ee[next].ja = true;
        }
    }
    
    
    pub fn po(&mut self) {
        self.oo += 1;
        
        
        if self.mfr {
            self.sfo();
            return;
        }
        
        
        
        
        
        if self.oo <= 3 {
            crate::serial_println!("[Desktop] safe frame {} / 3", self.oo);
            
            let mouse = crate::mouse::drd();
            
            framebuffer::cwe(0xFF010200);
            framebuffer::ili();
            
            self.hgt();
            self.hgx();
            self.dqf();
            
            self.gkt = mouse.b;
            self.gku = mouse.c;
            self.etz = self.ee.len();
            self.ety = self.ajo;
            self.etw = self.aka.iw;
            framebuffer::gge();
            framebuffer::sv();
            crate::serial_println!("[Desktop] safe frame {} done", self.oo);
            return;
        }
        
        
        self.qli();
        
        
        self.ivo += 1;
        let lpb = crate::logger::lh();
        if self.hkf == 0 { self.hkf = lpb; }
        let ez = lpb.ao(self.hkf);
        
        if ez >= 100 {
            self.cya = ((self.ivo as u64 * 100) / ez.am(1)) as u32;
            self.ivo = 0;
            self.hkf = lpb;
        }
        
        
        self.xos();
        
        
        crate::drivers::net::wifi::poll();
        self.mje = crate::drivers::net::wifi::lfz();
        
        
        let wqa: Vec<u32> = self.eyq.cai().hu().collect();
        for ad in wqa {
            let rl = self.ee.iter().any(|d| d.ad == ad && d.ja && d.iw && !d.aat);
            if rl {
                if let Some(atl) = self.eyq.ds(&ad) {
                    atl.or();
                }
            }
        }
        
        
        let upy: Vec<u32> = self.ano.cai().hu().collect();
        for ad in upy {
            if let Some(sn) = self.ano.ds(&ad) {
                sn.or();
            }
        }
        
        
        let taa: Vec<u32> = self.dra.cai().hu().collect();
        for ad in taa {
            let rl = self.ee.iter().any(|d| d.ad == ad && d.ja && d.iw && !d.aat);
            if rl {
                if let Some(kxo) = self.dra.ds(&ad) {
                    kxo.or();
                }
            }
        }
        
        
        #[cfg(feature = "emulators")]
        {
        let urz: Vec<u32> = self.dtk.cai().hu().collect();
        for ad in urz {
            let rl = self.ee.iter().any(|d| d.ad == ad && d.ja && d.iw && !d.aat);
            if rl {
                if let Some(cw) = self.dtk.ds(&ad) {
                    cw.or();
                }
            }
        }
        }
        
        
        
        #[cfg(feature = "emulators")]
        {
        let taf: Vec<u32> = self.arf.cai().hu().collect();
        for ad in taf {
            let rl = self.ee.iter().any(|d| d.ad == ad && d.iw && !d.aat && !d.egj);
            if rl {
                
                let ets = self.azy.iter()
                    .du(|(_, abg)| abg.fnb == Some(ad))
                    .map(|(&czg, _)| czg)
                    .or_else(|| self.azy.cai().next().hu());
                
                
                let (ant, mut dwj, mut dwi, cmx, eka) =
                    if let Some(czg) = ets {
                        if let Some(abg) = self.azy.get(&czg) {
                            (abg.ant, abg.dwj, abg.dwi, abg.cmx, abg.eka)
                        } else { (false, false, false, 2, false) }
                    } else { (false, false, false, 2, false) };
                
                
                if ant && !dwj && !dwi {
                    
                    if let Some(cw) = self.arf.ds(&ad) {
                        if !crate::keyboard::alh(0x11) { cw.avy(b'w'); }
                        if !crate::keyboard::alh(0x1E) { cw.avy(b'a'); }
                        if !crate::keyboard::alh(0x1F) { cw.avy(b's'); }
                        if !crate::keyboard::alh(0x20) { cw.avy(b'd'); }
                        if !crate::keyboard::alh(0x2D) { cw.avy(b'x'); }
                        if !crate::keyboard::alh(0x2C) { cw.avy(b'z'); }
                        if !crate::keyboard::alh(0x2E) { cw.avy(b'c'); }
                        if !crate::keyboard::alh(0x1C) { cw.avy(b'\r'); }
                    }
                    continue;
                }

                
                let qb = match cmx {
                    0 => if self.oo % 4 == 0 { 1 } else { 0 }, 
                    1 => if self.oo % 2 == 0 { 1 } else { 0 }, 
                    2 => 1, 
                    3 => 2, 
                    4 => 4, 
                    _ => 1,
                };

                if let Some(cw) = self.arf.ds(&ad) {
                    
                    if !crate::keyboard::alh(0x11) { cw.avy(b'w'); }
                    if !crate::keyboard::alh(0x1E) { cw.avy(b'a'); }
                    if !crate::keyboard::alh(0x1F) { cw.avy(b's'); }
                    if !crate::keyboard::alh(0x20) { cw.avy(b'd'); }
                    if !crate::keyboard::alh(0x2D) { cw.avy(b'x'); }
                    if !crate::keyboard::alh(0x2C) { cw.avy(b'z'); }
                    if !crate::keyboard::alh(0x2E) { cw.avy(b'c'); }
                    if !crate::keyboard::alh(0x1C) { cw.avy(b'\r'); }

                    
                    if eka {
                        if let Some(czg) = ets {
                            
                            let fz = cw.cpu.fz;
                            let q = cw.cpu.q;
                            let bb = cw.cpu.bb;
                            let sp = cw.cpu.sp;
                            let opcode = crate::game_lab::boa(cw, fz);
                            drop(cw); 
                            if let Some(abg) = self.azy.ds(&czg) {
                                if abg.trace.len() >= 64 { abg.trace.remove(0); }
                                abg.trace.push(crate::game_lab::Azx { fz, opcode, q, bb, sp });
                            }
                            
                            if let Some(cw) = self.arf.ds(&ad) {
                                for _ in 0..qb { cw.or(); }
                            }
                            
                            if dwj || dwi {
                                if let Some(abg) = self.azy.ds(&czg) {
                                    abg.dwj = false;
                                    abg.dwi = false;
                                }
                            }
                            
                            if let Some(cw) = self.arf.get(&ad) {
                                if let Some(abg) = self.azy.ds(&czg) {
                                    abg.pxk(cw);
                                    crate::game_lab::pxh(abg, cw);
                                    
                                    if abg.pkj(cw.cpu.fz) {
                                        abg.ant = true;
                                    }
                                }
                            }
                            continue; 
                        }
                    }

                    
                    for _ in 0..qb { cw.or(); }
                }

                
                if let Some(czg) = ets {
                    if dwj || dwi {
                        if let Some(abg) = self.azy.ds(&czg) {
                            abg.dwj = false;
                            abg.dwi = false;
                        }
                    }
                    if let Some(cw) = self.arf.get(&ad) {
                        if let Some(abg) = self.azy.ds(&czg) {
                            abg.pxk(cw);
                            crate::game_lab::pxh(abg, cw);
                            if abg.pkj(cw.cpu.fz) {
                                abg.ant = true;
                            }
                        }
                    }
                }
            }
        }
        }
        
        
        let khj: Vec<u32> = self.dou.cai().hu().collect();
        for ad in khj {
            let rl = self.ee.iter().any(|d| d.ad == ad && d.iw && !d.aat);
            if rl {
                if let Some(chess) = self.dou.ds(&ad) {
                    chess.xgt(16); 
                }
            }
        }
        
        
        let ubv: Vec<u32> = self.dso.cai().hu().collect();
        for ad in ubv {
            let rl = self.ee.iter().any(|d| d.ad == ad && d.iw && !d.aat);
            if rl {
                if let Some(abg) = self.dso.ds(&ad) {
                    abg.or();
                }
            }
        }
        
        
        #[cfg(feature = "emulators")]
        {
        let tac: Vec<u32> = self.azy.cai().hu().collect();
        for ad in tac {
            let rl = self.ee.iter().any(|d| d.ad == ad && d.iw && !d.aat);
            if rl {
                if let Some(abg) = self.azy.ds(&ad) {
                    abg.or();
                }
            }
        }
        }
        
        
        if self.oo % 9 == 0 {
            self.btx = !self.btx;
        }
        
        
        let mouse = crate::mouse::drd();
        
        
        if self.che == RenderMode::Ks {
            self.sem();
            return;
        }
        
        
        if self.che == RenderMode::Atd {
            self.sdm();
            return;
        }
        
        
        
        
        let mqs = self.ee.len() != self.etz;
        let omy = self.ajo != self.ety 
                        || self.aka.iw != self.etw;
        
        
        if mqs {
            self.bex = true;
        }
        
        
        if self.ud.gh {
            self.nni();
            framebuffer::sv();
            return;
        }
        
        
        
        
        
        
        framebuffer::ili(); 
        self.gff();
        self.hgt();
        
        
        let ywp = self.ee.iter().any(|d| d.iw && !d.aat);
        for bh in &self.ee {
            if bh.iw && !bh.aat {
                self.nnu(bh);
            }
        }
        
        
        self.nmy();
        
        
        self.nnj();
        
        
        self.nmz();
        
        
        self.nmv();
        
        
        #[cfg(feature = "emulators")]
        self.nnk();
        #[cfg(feature = "emulators")]
        self.nna();
        
        
        if let Some(mgg) = self.dwc {
            let bxw = self.ac - W_;
            let abd = self.z / 2;
            let wp = bxw / 2;
            let (cr, cq, kp, kl) = match mgg {
                SnapDir::Ap       => (0, 0, abd, bxw),
                SnapDir::Ca      => (abd, 0, abd, bxw),
                SnapDir::Dp    => (0, 0, abd, wp),
                SnapDir::Dq   => (abd, 0, abd, wp),
                SnapDir::Dt => (0, wp, abd, wp),
                SnapDir::Du => (abd, wp, abd, wp),
            };
            
            framebuffer::ih(cr, cq, kp, kl, 0x00FF66, 18);
            
            framebuffer::lx(cr + 2, cq + 2, kp.ao(4), kl.ao(4), X_);
            framebuffer::lx(cr + 3, cq + 3, kp.ao(6), kl.ao(6), P_);
        }
        
        
        self.hgx();
        
        
        if self.ajo {
            self.krl();
        }
        
        
        if self.aka.iw {
            self.kqx();
        }
        
        
        self.nmx();
        
        
        if self.eug {
            self.nng();
        }

        
        self.dqf();
        
        
        self.gkt = mouse.b;
        self.gku = mouse.c;
        self.etz = self.ee.len();
        self.ety = self.ajo;
        self.etw = self.aka.iw;
        
        
        framebuffer::gge();
        framebuffer::sv();
    }
    
    
    fn sem(&mut self) {
        use crate::graphics::opengl::*;
        
        
        let os = 1.0 / 60.0; 
        
        
        {
            let mut dfd = compositor::compositor();
            dfd.qs(os);
            
            
            for bh in &self.ee {
                if let Some(surface) = dfd.teu(bh.ad) {
                    surface.b = bh.b as f32;
                    surface.c = bh.c as f32;
                    surface.z = bh.z as f32;
                    surface.ac = bh.ac as f32;
                    surface.ja = bh.ja;
                    surface.iw = bh.iw && !bh.aat;
                }
            }
        }
        
        
        compositor::vvm();
        
        
        
        self.hgx();
        
        if self.ajo {
            self.krl();
        }
        
        if self.aka.iw {
            self.kqx();
        }
        
        
        self.hgt();
        
        
        self.dqf();
        
        
        self.etz = self.ee.len();
        self.ety = self.ajo;
        self.etw = self.aka.iw;
        
        
        framebuffer::sv();
    }
    
    
    
    
    
    
    
    fn fzt(&mut self, b: u32, c: u32, d: u32, i: u32) {
        if self.eov < 32 {
            self.dpz[self.eov] = (b, c, d, i);
            self.eov += 1;
        }
    }
    
    
    fn sdm(&mut self) {
        let mouse = crate::mouse::drd();
        let mqs = self.ee.len() != self.etz;
        let omy = self.ajo != self.ety
                        || self.aka.iw != self.etw;
        let rsf = mouse.b != self.gkt || mouse.c != self.gku;
        
        
        self.eov = 0;
        
        
        if mqs || omy || self.bex {
            
            self.fzt(0, 0, self.z, self.ac);
            self.bex = false;
        } else {
            
            if rsf {
                
                let lqc = (self.gkt.am(0) as u32).ao(2);
                let lqd = (self.gku.am(0) as u32).ao(2);
                self.fzt(lqc, lqd, 24, 24);
                
                let evh = (mouse.b.am(0) as u32).ao(2);
                let bhn = (mouse.c.am(0) as u32).ao(2);
                self.fzt(evh, bhn, 24, 24);
            }
            
            
            let xuo: Vec<(u32, u32, u32, u32)> = self.ee.iter()
                .hi(|d| d.iw && !d.aat)
                .map(|d| (d.b.am(0) as u32, d.c.am(0) as u32, d.z, d.ac))
                .collect();
            for (fx, lw, hk, mg) in xuo {
                self.fzt(fx, lw, hk, mg);
            }
            
            if self.oo % 60 == 0 {
                self.fzt(0, self.ac.ao(40), self.z, 40);
            }
        }
        
        
        if self.ud.gh {
            self.nni();
        } else {
            framebuffer::cwe(0xFF000000);
            self.gff();
            self.hgt();
            
            for bh in &self.ee {
                if bh.iw && !bh.aat {
                    self.nnu(bh);
                }
            }
            self.nmy();
            self.nnj();
            self.nmz();
            self.nmv();
            #[cfg(feature = "emulators")]
            self.nnk();
            #[cfg(feature = "emulators")]
            self.nna();
            self.hgx();
            if self.ajo { self.krl(); }
            if self.aka.iw { self.kqx(); }
            self.nmx();
            if self.eug { self.nng(); }
            self.dqf();
        }
        
        
        self.gkt = mouse.b;
        self.gku = mouse.c;
        self.etz = self.ee.len();
        self.ety = self.ajo;
        self.etw = self.aka.iw;
        
        
        if crate::drivers::virtio_gpu::anl() && self.eov > 0 && self.eov < 32 {
            
            crate::drivers::virtio_gpu::vkt(
                &self.dpz[..self.eov]
            );
            
            framebuffer::wwj();
        } else {
            framebuffer::sv();
        }
        
        self.kzn = self.kzn.cn(1);
    }
    
    
    fn kqx(&self) {
        let rs = self.aka.b;
        let xp = self.aka.c;
        let djn = 200i32;
        let crv = 28;
        let gmo = self.aka.pj.len() as i32 * crv + 8;
        let ob = 4;
        let dfg: u32 = 8;
        
        
        for a in (1..=6).vv() {
            let dw = (18 - a * 2).am(4) as u32;
            let dls = dw << 24;
            mf(
                rs + a, xp + a + 2,
                djn as u32, gmo as u32,
                dfg + 2, dls,
            );
        }
        
        
        mf(
            rs, xp,
            djn as u32, gmo as u32,
            dfg, 0xFF0C1210,
        );
        
        
        
        for br in 0..gmo.v(20) {
            let ixc = (12 - br * 12 / 20).am(0) as u32;
            if ixc > 0 {
                let cte = (ixc << 24) | 0x00FFFFFF;
                
                let flp = if br < dfg as i32 { (dfg as i32 - ggu((dfg as i32 * dfg as i32) - (dfg as i32 - br) * (dfg as i32 - br))) } else { 0 };
                let mj = rs + flp;
                let zv = djn - flp * 2;
                if zv > 0 {
                    crate::framebuffer::ah(mj as u32, (xp + br) as u32, zv as u32, 1, cte);
                }
            }
        }
        
        
        tf(
            rs, xp,
            djn as u32, gmo as u32,
            dfg, GC_,
        );
        
        
        crate::framebuffer::ah(
            (rs + dfg as i32) as u32, xp as u32,
            (djn - dfg as i32 * 2) as u32, 1, EC_,
        );
        
        
        for (w, item) in self.aka.pj.iter().cf() {
            let ajd = xp + ob + w as i32 * crv;
            
            let apx = self.lf >= rs && self.lf < rs + djn
                && self.ot >= ajd && self.ot < ajd + crv;
            
            if apx && item.hr != ContextAction::Hl && !item.cu.cj("─") {
                
                mf(
                    rs + 4, ajd,
                    (djn - 8) as u32, (crv - 2) as u32,
                    6, P_,
                );
                mf(
                    rs + 6, ajd + 1,
                    (djn - 12) as u32, (crv - 4) as u32,
                    5, ALV_,
                );
                
                mf(
                    rs + 4, ajd + 4,
                    2, (crv - 10) as u32,
                    1, I_,
                );
            }
            
            
            if item.cu.cj("─") {
                framebuffer::ah(
                    (rs + 12) as u32, (ajd + crv / 2) as u32,
                    (djn - 24) as u32, 1,
                    P_
                );
            } else {
                let agx = if apx { AG_ } else { BK_ };
                self.cb(rs + 16, ajd + 6, &item.cu, agx);
            }
        }
    }

    
    
    
    fn qhz(&mut self) {
        
        if self.drh.len() < 256 {
            self.drh.cmg(256, 0.0);
            self.erf.cmg(256, 0.0);
        }
        if self.gij.len() < 43 {
            self.gij.cmg(43, 0.0);
        }

        
        if !crate::drivers::hda::lgj() {
            
            self.erh *= 0.92;
            self.ere *= 0.92;
            self.erg *= 0.92;
            self.eri *= 0.92;
            self.ecd *= 0.92;
            self.drg *= 0.85;
            if self.ecd < 0.001 {
                self.fjs = false;
            }
            return;
        }

        
        let rzj = crate::drivers::hda::gic();
        let bvg = crate::drivers::hda::hlj();
        
        let (aeg, gbq) = match rzj {
            Some((ai, r)) if !ai.abq() && r > 512 => (ai, r),
            _ => return,
        };

        
        
        let ljy = (bvg as usize) / 2;
        
        let bbp = 256usize;
        let vsn = if ljy >= bbp * 2 {
            ljy - bbp * 2
        } else {
            
            gbq.ao(bbp * 2 - ljy)
        };

        let mut awd: f32 = 0.0;
        for a in 0..bbp {
            let w = (vsn + a * 2) % gbq; 
            let e = unsafe { *aeg.add(w) } as f32;
            self.drh[a] = e;
            self.erf[a] = 0.0;
            let q = if e >= 0.0 { e } else { -e };
            if q > awd { awd = q; }
        }

        
        if awd < 10.0 {
            self.fjs = false;
            self.erh *= 0.92;
            self.ere *= 0.92;
            self.erg *= 0.92;
            self.eri *= 0.92;
            self.ecd *= 0.92;
            self.drg *= 0.85;
            return;
        }
        self.fjs = true;

        
        if awd > self.fju {
            self.fju += (awd - self.fju) * 0.3;
        } else {
            self.fju *= 0.9995;
        }
        let dqz = if self.fju > 100.0 { 16000.0 / self.fju } else { 1.0 };

        
        for a in 0..bbp {
            let ab = a as f32 / bbp as f32;
            let hmm = 0.5 * (1.0 - libm::zq(2.0 * core::f32::consts::Eu * ab));
            self.drh[a] *= hmm * dqz / 32768.0;
        }

        
        {
            let ath = &mut self.drh[..bbp];
            let aum = &mut self.erf[..bbp];
            
            let mut fb = 0usize;
            for a in 0..bbp {
                if a < fb { ath.swap(a, fb); aum.swap(a, fb); }
                let mut ef = bbp >> 1;
                while ef >= 1 && fb >= ef { fb -= ef; ef >>= 1; }
                fb += ef;
            }
            
            let mut gu = 2usize;
            while gu <= bbp {
                let iv = gu >> 1;
                let hg = -core::f32::consts::Eu / iv as f32;
                let (ciw, ekv) = (libm::st(hg), libm::zq(hg));
                for eh in (0..bbp).akt(gu) {
                    let (mut bfu, mut yi) = (1.0f32, 0.0f32);
                    for ef in 0..iv {
                        let cfm = eh + ef;
                        let crw = cfm + iv;
                        let agd = bfu * ath[crw] - yi * aum[crw];
                        let ezs = bfu * aum[crw] + yi * ath[crw];
                        ath[crw] = ath[cfm] - agd; aum[crw] = aum[cfm] - ezs;
                        ath[cfm] += agd; aum[cfm] += ezs;
                        let utz = bfu * ekv - yi * ciw;
                        yi = bfu * ciw + yi * ekv;
                        bfu = utz;
                    }
                }
                gu <<= 1;
            }
        }

        
        let efa = |ath: &[f32], aum: &[f32], hh: usize, gd: usize| -> f32 {
            let mut e = 0.0f32;
            for a in hh..gd.v(128) {
                e += libm::bon(ath[a] * ath[a] + aum[a] * aum[a]);
            }
            e / (gd - hh).am(1) as f32
        };
        let frz = efa(&self.drh, &self.erf, 1, 2);
        let dkz = efa(&self.drh, &self.erf, 2, 4);
        let exe = efa(&self.drh, &self.erf, 4, 16);
        let hwr = efa(&self.drh, &self.erf, 16, 60);
        let lxc = frz * 1.5 + dkz * 1.2 + exe * 0.5 + hwr * 0.2;

        
        let bie = |vo: f32, new: f32, q: f32, m: f32| -> f32 {
            if new > vo { vo + (new - vo) * q } else { vo + (new - vo) * m }
        };
        self.erh = bie(self.erh, frz.v(1.0), 0.75, 0.10);
        self.ere = bie(self.ere, dkz.v(1.0), 0.70, 0.10);
        self.erg = bie(self.erg, exe.v(1.0), 0.60, 0.12);
        self.eri = bie(self.eri, hwr.v(1.0), 0.70, 0.16);
        self.ecd = bie(self.ecd, lxc.v(1.5), 0.65, 0.10);

        
        let dee = frz + dkz * 0.8;
        self.gij[self.ixd] = dee;
        self.ixd = (self.ixd + 1) % 43;
        if self.fjt < 43 { self.fjt += 1; }
        let adu = self.fjt.am(1) as f32;
        let abl: f32 = self.gij.iter().take(self.fjt).sum::<f32>() / adu;
        let mut fax = 0.0f32;
        for a in 0..self.fjt {
            let bc = self.gij[a] - abl;
            fax += bc * bc;
        }
        let igh = fax / adu;
        let bxm = (-15.0 * igh + 1.45f32).am(1.05).v(1.5);
        let lqp = dee - self.kze;
        if dee > abl * bxm && lqp > 0.002 && self.fjt > 5 {
            let ccc = ((dee - abl * bxm) / abl.am(0.001)).v(1.0);
            self.drg = (0.6 + ccc * 0.4).v(1.0);
        } else {
            self.drg *= 0.88;
            if self.drg < 0.02 { self.drg = 0.0; }
        }
        self.kze = dee;
    }
    
    
    
    
    
    fn nni(&mut self) {
        
        let (fp, iz, gm, me) = crate::mobile::nbi(self.z, self.ac);
        self.ud.dxp = fp;
        self.ud.ddi = iz;
        self.ud.att = gm;
        self.ud.azc = me;

        
        framebuffer::cwe(0xFF000000);

        
        self.gff();

        
        if fp > 0 {
            framebuffer::ah(0, 0, fp as u32, self.ac, 0xFF020202);
            framebuffer::ah((fp + gm as i32) as u32, 0, (self.z as i32 - fp - gm as i32).am(0) as u32, self.ac, 0xFF020202);
        }

        
        crate::mobile::sex(fp, iz, gm, me);

        
        crate::mobile::xgr(&mut self.ud);

        
        if self.oo % 60 == 0 || self.ud.bso.is_empty() {
            let os = crate::rtc::cgz();
            use core::fmt::Write;
            self.ud.bso.clear();
            let _ = core::write!(self.ud.bso, "{:02}:{:02}", os.bek, os.bri);
        }

        let frame = self.ud.dod;
        let bls = self.ud.bls;
        let bso = self.ud.bso.clone();
        let abe = self.ud.hmx;

        
        crate::mobile::hgw(fp, iz, gm, me, &bso, frame);

        match bls {
            crate::mobile::MobileView::Lo => {
                crate::mobile::nnc(fp, iz, gm, me, abe, frame);
                
                let qlc = crate::mobile::Adc {
                    uu: self.fjs,
                    rf: self.drg,
                    abo: self.ecd,
                    ato: self.erh,
                    aee: self.ere,
                    vs: self.erg,
                    axg: self.eri,
                    frame: self.oo,
                };
                crate::mobile::sdz(fp, iz, gm, me, &qlc,
                    self.ud.gnh,
                    self.ud.lnc);
                crate::mobile::irs(fp, iz, gm, me, -1, frame);
                crate::mobile::irt(fp, iz, gm, me);
            }
            crate::mobile::MobileView::Zv => {
                
                let com = self.ud.fcj.unwrap_or(0);
                let kas = if (com as usize) < crate::mobile::qjm() {
                    crate::mobile::kas(com as usize)
                } else { "App" };
                crate::mobile::sbe(fp, iz, gm, kas, frame);
                
                let qld = crate::mobile::Adc {
                    uu: self.fjs,
                    rf: self.drg,
                    abo: self.ecd,
                    ato: self.erh,
                    aee: self.ere,
                    vs: self.erg,
                    axg: self.eri,
                    frame: self.oo,
                };
                crate::mobile::sdy(
                    fp, iz, gm, me,
                    com, self.oo, &qld,
                    &self.ud,
                );
                
                crate::mobile::irt(fp, iz, gm, me);
            }
            crate::mobile::MobileView::Apa => {
                crate::mobile::sbo(fp, iz, gm, me, &[], 0, frame);
                crate::mobile::irt(fp, iz, gm, me);
            }
            crate::mobile::MobileView::Wc => {
                crate::mobile::nnc(fp, iz, gm, me, abe, frame);
                crate::mobile::irs(fp, iz, gm, me, -1, frame);
                crate::mobile::sck(fp, iz, gm, me, self.ud.dzl, frame);
                crate::mobile::irt(fp, iz, gm, me);
            }
        }

        
        self.dqf();
    }

    
    fn qjy(&mut self, hr: crate::mobile::MobileAction) {
        use crate::mobile::MobileAction;
        match hr {
            MobileAction::None => {}
            MobileAction::Atc => {
                self.ud.bls = crate::mobile::MobileView::Lo;
                self.ud.fcj = None;
            }
            MobileAction::Bof => {
                self.ud.bls = crate::mobile::MobileView::Apa;
            }
            MobileAction::Bnx => {
                self.ud.bls = crate::mobile::MobileView::Wc;
                self.ud.dzl = 1; 
            }
            MobileAction::Bdp => {
                self.ud.bls = crate::mobile::MobileView::Lo;
            }
            MobileAction::Bkr(w) => {
                self.ud.bls = crate::mobile::MobileView::Zv;
                self.ud.fcj = Some(w as u32);
                crate::serial_println!("[Mobile] Launch app #{}", w);
            }
            MobileAction::Bks(gk) => {
                let w = crate::mobile::rzy(gk as usize);
                self.ud.bls = crate::mobile::MobileView::Zv;
                self.ud.fcj = Some(w as u32);
                crate::serial_println!("[Mobile] Launch dock app slot={} -> idx={}", gk, w);
            }
            MobileAction::Bcm => {
                self.ud.bls = crate::mobile::MobileView::Lo;
                self.ud.fcj = None;
            }
            MobileAction::Bzn(ad) => {
                self.ud.kid.push((ad, 255));
            }
            MobileAction::Bmx => {
                
                const FO_: u32 = 0xFFFF_FFFE;
                if !self.ano.bgm(&FO_) {
                    self.ano.insert(FO_, MusicPlayerState::new());
                }
                if let Some(sn) = self.ano.ds(&FO_) {
                    match sn.g {
                        PlaybackState::Af => sn.dkp(0),
                        PlaybackState::Ce | PlaybackState::Cl => sn.mlq(),
                    }
                }
            }
            MobileAction::Chp => {
                const FO_: u32 = 0xFFFF_FFFE;
                if let Some(sn) = self.ano.ds(&FO_) {
                    sn.qg();
                }
            }
            MobileAction::Bmw => {
                self.ud.gnh = !self.ud.gnh;
            }
            MobileAction::Bmv(ev) => {
                self.ud.lnc = ev;
                self.ud.gnh = false;
                
                self.visualizer.ev = ev;
                crate::serial_println!("[Mobile] Viz mode set to {} ({})", ev,
                    crate::visualizer::OG_[ev as usize % crate::visualizer::IR_ as usize]);
            }
            MobileAction::CalcButton(bmc) => {
                
                let jn = &mut self.ud;
                match bmc {
                    16 => { 
                        jn.aqu.clear();
                        jn.fec = 0;
                        jn.coz = 0;
                        jn.dzk = false;
                    }
                    17 => { 
                        if !jn.aqu.is_empty() && jn.aqu != "0" {
                            if jn.aqu.cj('-') {
                                jn.aqu.remove(0);
                            } else {
                                jn.aqu.insert(0, '-');
                            }
                        }
                    }
                    18 => { 
                        if let Ok(p) = jn.aqu.parse::<i64>() {
                            let result = p / 100;
                            jn.aqu.clear();
                            use core::fmt::Write;
                            let _ = core::write!(jn.aqu, "{}", result);
                        }
                    }
                    10 => { 
                        if jn.dzk { jn.aqu.clear(); jn.dzk = false; }
                        if !jn.aqu.contains('.') {
                            if jn.aqu.is_empty() { jn.aqu.push('0'); }
                            jn.aqu.push('.');
                        }
                    }
                    15 => { 
                        let cv = jn.aqu.parse::<i64>().unwrap_or(0);
                        let result = match jn.fec {
                            1 => jn.coz + cv,
                            2 => jn.coz - cv,
                            3 => jn.coz * cv,
                            4 => if cv != 0 { jn.coz / cv } else { 0 },
                            _ => cv,
                        };
                        jn.aqu.clear();
                        use core::fmt::Write;
                        let _ = core::write!(jn.aqu, "{}", result);
                        jn.fec = 0;
                        jn.coz = 0;
                        jn.dzk = true;
                    }
                    11 | 12 | 13 | 14 => { 
                        let cv = jn.aqu.parse::<i64>().unwrap_or(0);
                        
                        if jn.fec > 0 && !jn.dzk {
                            let result = match jn.fec {
                                1 => jn.coz + cv,
                                2 => jn.coz - cv,
                                3 => jn.coz * cv,
                                4 => if cv != 0 { jn.coz / cv } else { 0 },
                                _ => cv,
                            };
                            jn.coz = result;
                            jn.aqu.clear();
                            use core::fmt::Write;
                            let _ = core::write!(jn.aqu, "{}", result);
                        } else {
                            jn.coz = cv;
                        }
                        jn.fec = bmc - 10; 
                        jn.dzk = true;
                    }
                    0..=9 => { 
                        if jn.dzk {
                            jn.aqu.clear();
                            jn.dzk = false;
                        }
                        if jn.aqu == "0" { jn.aqu.clear(); }
                        if jn.aqu.len() < 15 {
                            jn.aqu.push((b'0' + bmc) as char);
                        }
                    }
                    _ => {}
                }
                crate::serial_println!("[Mobile] Calc: display={}", jn.aqu);
            }
            MobileAction::Bhb(w) => {
                let jn = &mut self.ud;
                jn.gha = w as i32;
                
                if w < 4 && jn.dqu == 0 {
                    jn.dqu = 1;
                    jn.gha = -1;
                }
                crate::serial_println!("[Mobile] Files: tap idx={} depth={}", w, jn.dqu);
            }
            MobileAction::Bha => {
                self.ud.dqu = self.ud.dqu.ao(1);
                self.ud.gha = -1;
            }
            MobileAction::Bso(w) => {
                let jn = &mut self.ud;
                jn.mey = w as i32;
                if (w as usize) < jn.gsk.len() {
                    jn.gsk[w as usize] = !jn.gsk[w as usize];
                }
                crate::serial_println!("[Mobile] Settings: toggled idx={}", w);
            }
            MobileAction::Bhu(w) => {
                self.ud.kxp = w as i32;
                crate::serial_println!("[Mobile] Games: selected idx={}", w);
            }
            MobileAction::Agv(awl) => {
                self.ud.hbi = awl;
                crate::serial_println!("[Mobile] Browser: page={}", awl);
            }
            MobileAction::Bfu(line) => {
                self.ud.isk = line as u32;
            }
            MobileAction::Arm(acp) => {
                self.ud.gfy = acp;
            }
            MobileAction::Bdl(im) => {
                let jn = &mut self.ud;
                if jn.cpd == im as i32 {
                    jn.cpd = -1; 
                } else if jn.cpd >= 0 {
                    
                    jn.hct = 1 - jn.hct;
                    jn.cpd = -1;
                    crate::serial_println!("[Mobile] Chess: move to sq={}", im);
                } else {
                    jn.cpd = im as i32;
                }
            }
            MobileAction::Bmu => {
                const FO_: u32 = 0xFFFF_FFFE;
                if !self.ano.bgm(&FO_) {
                    self.ano.insert(FO_, MusicPlayerState::new());
                }
                if let Some(sn) = self.ano.ds(&FO_) {
                    match sn.g {
                        PlaybackState::Af => sn.dkp(0),
                        PlaybackState::Ce | PlaybackState::Cl => sn.mlq(),
                    }
                }
            }
            MobileAction::Buc => {
                let jn = &mut self.ud;
                
                let commands = ["help", "uname", "ls", "pwd", "whoami", "date", "free -h", "uptime"];
                let rfg = jn.dwq.len() / 2; 
                let cmd = commands[rfg % commands.len()];
                jn.dwq.push(alloc::format!("$ {}", cmd));
                let mk = match cmd {
                    "help" => "Available: help, ls, pwd, date, uname, whoami, free, uptime",
                    "ls" => "Documents  Downloads  Music  Pictures  config.toml",
                    "pwd" => "/home/user",
                    "uname" => "TrustOS 2.0 aarch64 #1 SMP",
                    "date" => "2026-03-05 12:00:00 UTC",
                    "whoami" => "user@trustos",
                    "free -h" => "  total: 8.0G  used: 2.1G  free: 5.9G",
                    "uptime" => "up 4h 23m, 1 user, load: 0.12",
                    _ => "command not found",
                };
                jn.dwq.push(alloc::string::String::from(mk));
                
                if jn.dwq.len() > 40 {
                    jn.dwq.bbk(0..2);
                }
            }
        }
    }

    
    
    
    
    
    
    
    
    
    fn sfo(&mut self) {
        let iu = crate::logger::lh();
        let ez = iu.ao(self.mfs); 
        
        
        const OW_: u64 = 50;   
        const OX_: u64 = 100;  
        const GZ_: u64 = 140;  
        const WE_: u64 = 180;  
        const CJK_: u64 = 200;  
        
        let z = self.z;
        let ac = self.ac;
        
        framebuffer::cwe(0xFF010200);
        framebuffer::ili();
        
        
        
        
        let ifs = if ez >= OW_ {
            255u8
        } else {
            ((ez * 255) / OW_) as u8
        };
        
        
        let hqw = if ez < OW_ {
            0u8
        } else if ez >= OX_ {
            255u8
        } else {
            (((ez - OW_) * 255) / (OX_ - OW_)) as u8
        };
        
        
        let jvu = if ez < OX_ {
            0u8
        } else if ez >= GZ_ {
            255u8
        } else {
            (((ez - OX_) * 255) / (GZ_ - OX_)) as u8
        };
        
        
        let oke = if ez < GZ_ {
            0u8
        } else if ez >= WE_ {
            255u8
        } else {
            (((ez - GZ_) * 255) / (WE_ - GZ_)) as u8
        };
        
        
        
        
        if hqw < 255 {
            
            self.gff();
            
            
            if hqw > 0 {
                framebuffer::ih(0, 0, z, ac, 0x000000, hqw as u32);
            }
            
            
            if jvu > 0 && jvu < 255 {
                
                
                let ang = jvu.ao(hqw);
                if ang > 0 {
                    
                    let cx = z / 4;
                    let ae = ac / 4;
                    framebuffer::ih(cx, ae, z / 2, ac / 2, 0x000000, ang as u32);
                }
            }
            
            
            if oke > 0 {
                let cfz = crate::logo_bitmap::AY_ as u32;
                let cfy = crate::logo_bitmap::BL_ as u32;
                let euh = (z / 2).ao(cfz / 2);
                let eui = (ac / 2).ao(cfy / 2);
                let ang = oke.ao(jvu.am(hqw));
                if ang > 0 {
                    let ov = 20u32;
                    framebuffer::ih(
                        euh.ao(ov), eui.ao(ov),
                        cfz + ov * 2, cfy + ov * 2,
                        0x000000, ang as u32,
                    );
                }
            }
        }
        
        
        
        if ifs < 255 {
            
            let wnu = (ifs as i32 * BY_ as i32) / 255;
            let xbe = (ifs as i32 * W_ as i32) / 255;
            
            
            if wnu < BY_ as i32 {
                
                self.hgt();
                
                framebuffer::ih(0, 0, BY_ as u32 + 10, ac, 0x000000, ifs as u32);
            }
            
            
            if xbe < W_ as i32 {
                self.hgx();
                framebuffer::ih(0, ac.ao(W_ as u32), z, W_ as u32, 0x000000, ifs as u32);
            }
        }
        
        
        if ez >= GZ_ {
            let xfq = if ez < WE_ {
                (((ez - GZ_) * 255) / (WE_ - GZ_)) as u8
            } else {
                255u8
            };
            let at = (xfq as u32 * 0xAA) / 255;
            let s = 0xFF000000 | (at << 8);
            let fr = "Shutting down...";
            let nk = 8u32;
            let bda = fr.len() as u32 * nk;
            let gx = (z / 2).ao(bda / 2);
            let ty = ac / 2 + 40;
            let mut cx = gx;
            for bm in fr.bw() {
                framebuffer::afn(cx, ty, bm, s);
                cx += nk;
            }
        }
        
        framebuffer::gge();
        framebuffer::sv();
        
        
        if ez >= CJK_ {
            crate::serial_println!("[SHUTDOWN] Animation complete, powering off...");
            
            framebuffer::cwe(0xFF000000);
            framebuffer::ili();
            framebuffer::gge();
            framebuffer::sv();
            
            
            for (ddq, sn) in self.ano.el() {
                sn.g = PlaybackState::Af;
            }
            
            
            crate::acpi::cbu();
            
        }
    }

    fn gff(&mut self) {
        
        
        
        
        const R_: usize = 256;
        const EL_: usize = 40;
        let bkq: usize = if self.asr >= DesktopTier::Bv {
            4
        } else if self.asr >= DesktopTier::Gc {
            2
        } else {
            1  
        };
        
        
        
        
        
        
        const CED_: [usize; 4]  = [28, 20, 14, 8];
        const CDX_: [f32; 4]       = [0.28, 0.50, 0.78, 1.0];
        const CEB_: [[f32; 4]; 3] = [
            [0.44, 0.88, 1.62, 2.25],
            [0.75, 1.50, 2.75, 3.75],
            [1.19, 2.38, 4.38, 6.25],
        ];
        let akl = (self.eup as usize).v(2);
        let udj: [f32; 4] = CEB_[akl];
        const CEC_: [f32; 4]      = [0.0, 0.3, 1.0, 2.0];
        const CDW_: [usize; 4] = [2, 3, 4, 6];
        const CDV_: [i16; 4]    = [ 0,  0,  0,  0];
        const CDU_: [i16; 4]    = [-4,  0,  4,  8];
        const CDT_: [i16; 4]    = [ 0,  0,  0,  0];
        const CDY_: [f32; 4]       = [0.0, 1.5, 3.5, 6.0];
        const CEA_: [u32; 4]    = [5, 7, 10, 14];
        const CDZ_: [u32; 4]    = [10, 14, 20, 28];
        const CGT_: [u32; 4]   = [3, 4, 6, 7];
        const CGS_: [u32; 4]   = [6, 8, 12, 14];
        const CGR_: [usize; 4] = [2, 3, 6, 8];
        let hoz = self.ud.gh;
        
        let ac = self.ac.ao(W_);
        let z = self.z;
        
        
        self.qhz();
        let eez = self.drg;
        let fnk = self.ecd;
        let jel = self.erh;
        let jej = self.ere;
        let jek = self.erg;
        let jem = self.eri;
        let csf = self.fjs;
        
        
        let myl = csf && eez > 0.5;
        if myl && !self.lko {
            self.hqx = self.hqx.cn(1);
        }
        self.lko = myl;
        
        let pys = csf && (self.hqx % 8 == 7);
        
        
        framebuffer::ah(0, 0, z, ac, 0xFF010200);
        
        let jlw = ac * 88 / 100;
        if jlw < ac {
            framebuffer::ah(0, jlw, z, ac - jlw, 0xFF020300);
        }
        
        
        
        
        
        if self.asr >= DesktopTier::Gc
            && (self.asr >= DesktopTier::Bv || self.oo % 3 == 0)
        {
            let wsl = self.oo as u32;
            let gu = 12u32; 
            let mut cq = 0u32;
            while cq < ac {
                let mut cr = 0u32;
                while cr < z {
                    
                    let i = (cr.hx(2654435761)).cn(cq.hx(340573321));
                    let i = i ^ (i >> 16);
                    if i % 97 == 0 {
                        
                        let mp = (i >> 8) % gu;
                        let qw = (i >> 14) % gu;
                        let y = cr + mp;
                        let x = cq + qw;
                        if y < z && x < ac && x < jlw {
                            
                            let ib = wsl.cn(i & 0xFF).hx(3);
                            let xmg = ((ib & 255) as i32 - 128).eki(); 
                            let fni = 40 + (xmg * 60 / 128) as u32; 
                            let r = 0xFF000000 | (fni << 16) | (fni << 8) | fni;
                            framebuffer::ii(y, x, r);
                        }
                    }
                    cr += gu;
                }
                cq += gu;
            }
        }
        
        if !self.lkn {
            return;
        }
        
        
        
        if self.oo < 5 { crate::serial_println!("[FRAME] #{} start", self.oo); }
        
        
        
        if self.asr >= DesktopTier::Bv {
            crate::visualizer::qs(
                &mut self.visualizer,
                z, ac,
                R_,
                eez, fnk,
                jel, jej, jek, jem,
                csf,
            );
        }
        
        if self.oo < 5 { crate::serial_println!("[FRAME] #{} viz done", self.oo); }
        
        
        if self.asr >= DesktopTier::Bv {
            crate::drone_swarm::qs(&mut self.drone_swarm);
        }
        
        
        let okf = ac / 2;
        let uib = z / 2;
        
        let ivb = 300.0f32;
        let nvf = 250.0f32; 
        
        
        let byt = z / R_ as u32;
        let oab = R_ as f32 / 2.0;
        
        let ebs = self.oo as f32 * 0.008;
        
        if self.oo < 5 { crate::serial_println!("[FRAME] #{} rain start", self.oo); }
        
        let (ggv, nsx, xze) = framebuffer::sww();
        let tmc = !self.dsz.is_empty();
        for fl in 0..bkq {
        
        
        let pqh = CEC_[fl];
        let wwn = match fl { 4 => 0.010f32, 5 => 0.014, 3 => 0.006, _ => 0.0 };
        let wwo = if pqh > 0.0 {
            let ib = (self.oo as f32) * wwn;
            
            let ayq = crate::graphics::holomatrix::cuh;
            let bic = ayq(ib);
            let cuc = ayq(ib * 1.7 + 2.0) * 0.4;
            ((bic + cuc) * pqh) as i32
        } else { 0i32 };
        let nes = if hoz { CGR_[fl] } else { CDW_[fl] };
        let qkw = CDV_[fl];
        let mwl = CDU_[fl];
        let qkv = CDT_[fl];
        let nve = CDY_[fl];
        
        let nvd = if self.visualizer.ev == 7 { nve * 2.5 } else { nve };
        let cyg = if hoz { CGT_[fl] } else { CEA_[fl] };
        let cyf = if hoz { CGS_[fl] } else { CDZ_[fl] };
        
        for bj in 0..R_.v(self.awc.len() / bkq.am(1)) {
            
            if nes > 1 && (bj % nes) != 0 { continue; }
            
            let w = bj * bkq.am(1) + fl;
            let ig = self.czn[w];
            let dv = self.fnr[w];
            
            let fde = (bj as u32 * byt) + byt / 2;
            let b = (fde as i32 + wwo).am(0).v(z as i32 - 1) as u32;
            
            
            let lic = CED_[fl];
            let udf: f32 = CDX_[fl];
            let oil: f32 = udj[fl];
            
            
            
            let ecb = ((bj as f32) - oab).gp() / oab;
            let kwv = (ecb * ecb).v(1.0);
            
            let swh = (kwv * (cyf as f32 * 0.15)) as u32;
            let nph: u32 = cyf + swh;
            
            let swk: i32 = (100.0 - kwv * 12.0) as i32;
            
            let swg: f32 = 1.0 - kwv * 0.04;
            
            
            
            let sxh = (self.fnr[bj * bkq.am(1)] >> 3) % 4;
            
            let (mxp, qmo, mxq, yfj) = if csf {
                match sxh {
                    0 => (jel,  0u8, 180u8, 0u8),   
                    1 => (jej,      0u8, 200u8, 0u8),   
                    2 => (jek,       0u8, 220u8, 0u8),   
                    _ => (jem,    0u8, 210u8, 0u8),   
                }
            } else {
                (0.0, 0u8, 200u8, 0u8) 
            };
            
            let sxj = if csf {
                (0.3 + mxp * 1.2).v(1.5)
            } else { 1.0 };
            
            
            
            let rly = pys && ((bj.hx(7) ^ self.hqx as usize) % 16 == 0);
            
            
            let kco = if csf {
                let ab = (bj as u32 * 2) % (R_ as u32);
                let rlx = if ab < R_ as u32 {
                    ab as f32 / R_ as f32
                } else {
                    2.0 - ab as f32 / R_ as f32
                };
                let uhs = eez * (0.5 + rlx * 0.5);
                (uhs * 6.0 + mxp * 4.0) as i32
            } else { 0 };
            
            
            
            let tfl = crate::visualizer::nez(&self.visualizer, bj) as i32;
            let wqv = ((ig as f32) * oil) as i32;
            let sjh = (((wqv + kco) * tfl / 100) * swk / 100).am(1);
            let bhn = self.awc[w] + sjh;
            if bhn > ac as i32 + (lic as i32 * nph as i32) {
                let gnp = dv.hx(1103515245).cn(12345);
                self.fnr[w] = gnp;
                self.awc[w] = -((gnp % (ac / 3)) as i32);
                let bw: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
                for a in 0..lic.v(EL_) {
                    let aap = gnp.cn((a as u32).hx(7919));
                    self.car[w * EL_ + a] = bw[(aap as usize) % bw.len()];
                }
            } else {
                self.awc[w] = bhn;
            }
            
            
            if rly { continue; }
            
            let buu = self.awc[w];
            
            
            let kjv = (ig as f32) * oil;
            let ibd = (kjv / 5.0).v(1.0); 
            let slr = if csf { fnk * 0.3 } else { 0.0 };
            let qom = if csf { eez * 0.15 } else { 0.0 };
            let fdr = ((0.3 + ibd * 0.7 + slr + qom) * swg * udf).v(1.5);
            
            
            let tfk = csf && bj < self.visualizer.anc.len() && {
                let (uj, wv) = self.visualizer.anc[bj];
                uj >= 0 && wv > uj
            };
            
            
            
            let ffk = ((bj as u32).hx(2654435761u32)) >> 20; 
            let rlz = 0.55 + (ffk % 100) as f32 / 110.0; 
            let wra = ((lic as f32) * (0.5 + ibd * 0.5) * rlz) as usize;
            let npi = wra.am(4).v(EL_);
            
            for a in 0..npi {
                
                if fl == 0 && (a & 1) == 1 { continue; }
                
                let avl = buu - (a as i32 * nph as i32);
                if avl < 0 || avl >= ac as i32 { continue; }
                
                
                if pys && (a % 5 == 0) && a > 3 { continue; }
                
                
                let pvt = if csf { (fnk * 30.0) as u8 } else { 0 };
                let sqs = (200u8 as u16 / (npi as u16).am(1)) as u8;
                
                let pmh = (ibd * 30.0) as u8;
                let ar = if a == 0 { 255u8 }
                    else if a == 1 { (230u8 + pmh / 2).v(255).akq(pvt / 2) }
                    else { (210u8 + pvt / 3 + pmh / 3).ao((a as u8).mbq(sqs.am(3))) };
                if ar < (if tfk { 2 } else { 3 }) { continue; }
                
                let kt = ((ar as f32) * fdr).v(255.0) as u8;
                
                
                
                
                let (m, at, o) = if a == 0 {
                    
                    let jwt = (0.50 + ibd * 0.45).v(0.95);
                    let mxr = 1.0 - jwt;
                    
                    let gjg = ((qmo as f32 * mxr + 180.0 * jwt) * fdr).v(190.0) as i16;
                    let gjb = ((mxq as f32 * mxr + 255.0 * jwt) * fdr).v(255.0) as i16;
                    let gix = ((180.0 * jwt) * fdr).v(190.0) as i16;
                    let kcq = if csf { (eez * 8.0).v(15.0) as i16 } else { 0 };
                    
                    let xb = (gjg + kcq / 4 + qkw).am(0).v(190) as u8;
                    let lp = (gjb + kcq + mwl).am(0).v(255) as u8;
                    let pq = (gix + kcq / 4 + qkv).am(0).v(190) as u8;
                    
                    let xb = xb.v(lp);
                    let pq = pq.v(lp);
                    (xb, lp, pq)
                } else {
                    
                    let yx = kt as f32 / 255.0;
                    let cqk = sxj;
                    if self.visualizer.aim == 23 {
                        
                        let (btu, bmh, aiv) = crate::visualizer::vpv(
                            bj, a, self.fnr[w],
                        );
                        let xb = (btu as f32 * yx * cqk).v(255.0) as u8;
                        let lp = (bmh as f32 * yx * cqk).v(255.0) as u8;
                        let pq = (aiv as f32 * yx * cqk).v(255.0) as u8;
                        (xb, lp, pq)
                    } else {
                        let wqw = 0.8 + ibd * 0.4; 
                        let agd = 0i16; 
                        let ejs = ((mxq as f32 * cqk * yx * wqw).v(255.0)) as i16;
                        let bov = 0i16; 
                        
                        let xb = 0u8; 
                        let lp = (ejs + mwl).am(0).v(255) as u8;
                        let pq = 0u8; 
                        (xb, lp, pq)
                    }
                };
                
                
                let (mut m, mut at, mut o) = (m, at, o);
                let mut hlo: u8 = 0;
                let mut kzc: u8 = 128;
                
                
                if fl >= 1 {
                    let jf = crate::visualizer::khf(
                        &self.visualizer, bj, avl,
                        self.visualizer.ana, fnk,
                    );
                    if jf.tq > 0 || jf.apb > 0 || jf.aug > 0 || jf.amv > 0
                        || jf.ys > 0 || jf.ata > 0 || jf.zc > 0 {
                        let (hsb, csn, csm) = crate::visualizer::lmn(
                            m, at, o, jf.tq, jf.eo, jf.apb,
                            jf.aug, jf.amv,
                            jf.atv, jf.axo, jf.ys, jf.ata, jf.zc,
                            eez, fnk,
                            jel, jej, jek, jem,
                            self.visualizer.aim,
                        );
                        m = hsb; at = csn; o = csm;
                        hlo = jf.bst;
                        kzc = jf.eo;
                    }
                    
                    if jf.dcm > 0 {
                        let ab = jf.dcm as f32 / 255.0;
                        let wq = 1.0 - ab;
                        m = (m as f32 * wq + jf.ejq as f32 * ab) as u8;
                        at = (at as f32 * wq + jf.ejn as f32 * ab) as u8;
                        o = (o as f32 * wq + jf.ejl as f32 * ab) as u8;
                        hlo = jf.bst;
                    }
                    
                    if jf.tp > 0 {
                        let eec = 1.0 - (jf.tp as f32 / 255.0);
                        m = (m as f32 * eec) as u8;
                        at = (at as f32 * eec) as u8;
                        o = (o as f32 * eec) as u8;
                    }
                }
                
                if hlo > 0 {
                    let bma = 1.0 + hlo as f32 / 100.0;
                    m = (m as f32 * bma).v(255.0) as u8;
                    at = (at as f32 * bma).v(255.0) as u8;
                    o = (o as f32 * bma).v(255.0) as u8;
                }
                
                
                
                
                let sux = if nvd > 0.0 && fl >= 3 {
                    let dx = b as f32 - uib as f32;
                    let bg = avl as f32 - okf as f32;
                    let ass = dx * dx + bg * bg;
                    
                    let bwl = ivb * ivb;
                    let otg = (ivb + nvf) * (ivb + nvf);
                    let duu = if ass < bwl {
                        1.0
                    } else if ass < otg {
                        
                        1.0 - (ass - bwl) / (otg - bwl)
                    } else {
                        0.0
                    };
                    if duu > 0.01 {
                        let ae = avl as f32;
                        let cx = bj as f32;
                        let ayq = crate::graphics::holomatrix::cuh;
                        let csy = ayq(ae * 0.0045 + cx * 0.13 + ebs);
                        let csz = ayq(ae * 0.012 + cx * 0.07 + ebs * 1.6 + 3.0) * 0.4;
                        let cta = ayq(ae * 0.028 + cx * 0.21 + ebs * 2.3 + 1.5) * 0.15;
                        ((csy + csz + cta) * nvd * duu) as i32
                    } else { 0 }
                } else { 0 };
                let fzb = (b as i32 + sux).am(0).v(z as i32 - 1) as u32;
                
                
                
                let gfk = if fl >= 3 {
                    crate::drone_swarm::query(
                        &self.drone_swarm, fzb as f32, avl as f32,
                    )
                } else {
                    crate::drone_swarm::DroneInteraction { kt: 1.0, cpl: 0, cwj: 0, cwi: 0 }
                };
                if gfk.kt != 1.0 || gfk.cpl != 0 {
                    let dyv = gfk.kt;
                    m = ((m as f32 * dyv).v(255.0)) as u8;
                    at = ((at as f32 * dyv).v(255.0)) as u8;
                    o = ((o as f32 * dyv).v(255.0)) as u8;
                    m = ((m as i16 + gfk.cpl).am(0).v(255)) as u8;
                    at = ((at as i16 + gfk.cwj).am(0).v(255)) as u8;
                    o = ((o as i16 + gfk.cwi).am(0).v(255)) as u8;
                }
                
                let s = 0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32);
                
                
                
                let oet = hlo > 30;
                let uqs = if oet { 10u32 } else { 28u32 };
                let des = dv.cn((a as u32 * 7919) ^ (self.oo as u32 / uqs));
                let bw: &[u8] = if oet {
                    if kzc > 180 {
                        
                        b"@#$%&WM8BOXZNHK"
                    } else if kzc < 80 {
                        
                        b".:;~-'`"
                    } else {
                        
                        b"0123456789ABCDEF"
                    }
                } else {
                    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|"
                };
                let r = bw[(des as usize) % bw.len()] as char;
                let ka = crate::framebuffer::font::ada(r);
                
                
                
                let odt = self.djj.gh
                    && fzb + cyg > self.djj.b
                    && fzb < self.djj.b + self.djj.z
                    && (avl as u32) + cyf > self.djj.c
                    && (avl as u32) < self.djj.c + self.djj.ac;

                
                let nbx = w * EL_ + a;
                let tmw = !odt && tmc && self.dsz.bgm(&nbx);
                
                let btu = ((s >> 16) & 0xFF) as u8;
                let bmh = ((s >> 8) & 0xFF) as u8;
                let aiv = (s & 0xFF) as u8;
                
                if odt {
                    
                    
                    
                    
                    let aci = &self.djj;
                    let hj = kt as f32 / 255.0;
                    for erm in 0..cyf as usize {
                        let x = avl as u32 + erm as u32;
                        if x >= ac { continue; }
                        if x < aci.c || x >= aci.c + aci.ac { continue; }
                        let ys: f32 = if x & 1 == 0 { 1.0 } else { 0.96 };
                        let hnu = x > ac * 88 / 100;
                        let fle = (x - aci.c) as usize;
                        for ga in 0..cyg {
                            let y = fzb + ga;
                            if y >= z { continue; }
                            if y < aci.b || y >= aci.b + aci.z { continue; }
                            let fld = (y - aci.b) as usize;
                            let egu = aci.hz[fle * aci.z as usize + fld];
                            if egu & 0xFF000000 == 0 { continue; }
                            let oc = ((egu >> 16) & 0xFF) as f32;
                            let bah = ((egu >> 8) & 0xFF) as f32;
                            let ue = (egu & 0xFF) as f32;
                            let mut xb = (oc * hj).v(255.0) as u8;
                            let mut lp = (bah * hj).v(255.0) as u8;
                            let mut pq = (ue * hj).v(255.0) as u8;
                            xb = ((xb as f32 * ys).v(255.0)) as u8;
                            lp = ((lp as f32 * ys).v(255.0)) as u8;
                            pq = ((pq as f32 * ys).v(255.0)) as u8;
                            if hnu {
                                lp = (lp as u16 + 10).v(255) as u8;
                            }
                            let gc = 0xFF000000 | ((xb as u32) << 16) | ((lp as u32) << 8) | (pq as u32);
                            framebuffer::ii(y, x, gc);
                        }
                    }
                } else if tmw {
                    
                    let qxh = &self.dsz[&nbx];
                    for erm in 0..16usize {
                        let x = avl as u32 + erm as u32;
                        if x >= ac { continue; }
                        let ys: f32 = if x & 1 == 0 { 1.0 } else { 0.96 };
                        let hnu = x > ac * 88 / 100;
                        for ga in 0..8u32 {
                            let egu = qxh.hz[erm * 8 + ga as usize];
                            if egu & 0xFF000000 == 0 { continue; } 
                            let y = fzb + ga;
                            if y >= z { continue; }
                            
                            let oc = ((egu >> 16) & 0xFF) as f32;
                            let bah = ((egu >> 8) & 0xFF) as f32;
                            let ue = (egu & 0xFF) as f32;
                            
                            
                            let hj = kt as f32 / 255.0;
                            let mut xb = (oc * hj).v(255.0) as u8;
                            let mut lp = (bah * hj).v(255.0) as u8;
                            let mut pq = (ue * hj).v(255.0) as u8;
                            if fl > 0 {
                                lp = (lp as u16 + 30u16).v(255) as u8;
                            }
                            xb = ((xb as f32 * ys).v(255.0)) as u8;
                            lp = ((lp as f32 * ys).v(255.0)) as u8;
                            pq = ((pq as f32 * ys).v(255.0)) as u8;
                            if hnu {
                                lp = (lp as u16 + 10).v(255) as u8;
                            }
                            let gc = 0xFF000000 | ((xb as u32) << 16) | ((lp as u32) << 8) | (pq as u32);
                            framebuffer::ii(y, x, gc);
                        }
                    }
                } else {
                    
                    
                    let srx = if fl > 0 { 30u16 } else { 0u16 };
                    let gyw = btu;
                    let jzs = (bmh as u16 + srx).v(255) as u8;
                    let jyr = aiv;
                    let rmg = 0xFF000000 | ((gyw as u32) << 16) | ((jzs as u32) << 8) | (jyr as u32);
                    let adz = (gyw as u16 * 245 >> 8) as u8;
                    let bsi = (jzs as u16 * 245 >> 8) as u8;
                    let is = (jyr as u16 * 245 >> 8) as u8;
                    let rmi = 0xFF000000 | ((adz as u32) << 16) | ((bsi as u32) << 8) | (is as u32);
                    let vtr = ac * 88 / 100;
                    let vyq = (jzs as u16 + 10).v(255) as u8;
                    let vyr = (bsi as u16 + 10).v(255) as u8;
                    let rmj = 0xFF000000 | ((gyw as u32) << 16) | ((vyq as u32) << 8) | (jyr as u32);
                    let rmk = 0xFF000000 | ((adz as u32) << 16) | ((vyr as u32) << 8) | (is as u32);
                    let xpl = !ggv.abq();
                    for cq in 0..cyf {
                        let x = avl as u32 + cq;
                        if x >= ac { continue; }
                        let bxg = ((cq * 16) / cyf).v(15) as usize;
                        let fs = ka[bxg];
                        if fs == 0 { continue; }
                        let tyk = x & 1 != 0;
                        let hnu = x > vtr;
                        let gc = match (tyk, hnu) {
                            (false, false) => rmg,
                            (true, false) => rmi,
                            (false, true) => rmj,
                            (true, true) => rmk,
                        };
                        if xpl {
                            
                            let dvh = x as usize * nsx as usize;
                            for cr in 0..cyg {
                                let mhb = ((cr * 8) / cyg).v(7);
                                if fs & (0x80 >> mhb) != 0 {
                                    let y = fzb + cr;
                                    if y < z {
                                        unsafe { *ggv.add(dvh + y as usize) = gc; }
                                    }
                                }
                            }
                        } else {
                            for cr in 0..cyg {
                                let mhb = ((cr * 8) / cyg).v(7);
                                if fs & (0x80 >> mhb) != 0 {
                                    let y = fzb + cr;
                                    if y < z {
                                        framebuffer::ii(y, x, gc);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } 
        } 
        
        
        
        
        
        
        if !hoz && self.asr >= DesktopTier::Bv {
            const CIC_: usize = 4;
            const ACE_: usize = 16;
            const ACD_: u32 = 16;  
            
            for bj in 0..R_.v(self.visualizer.anc.len()) {
                
                let (uj, wv) = self.visualizer.anc[bj];
                if uj < 0 || wv <= uj { continue; }
                
                let b = (bj as u32 * byt) + byt / 2;
                
                for vi in 0..CIC_ {
                    let kvw = (bj as u32).hx(2654435761)
                        ^ ((vi as u32 + 17).hx(0x9E3779B9));
                    let ssm = 1 + (kvw % 3);
                    
                    
                    let aku = (ac + ACE_ as u32 * ACD_) as u32;
                    let vqk = (self.oo as u32)
                        .hx(ssm)
                        .cn(kvw);
                    let xrx = (vqk % aku.am(1)) as i32
                        - (ACE_ as i32 * ACD_ as i32);
                    
                    for a in 0..ACE_ {
                        let avl = xrx - (a as i32 * ACD_ as i32);
                        if avl < 0 || avl >= ac as i32 { continue; }
                        
                        
                        let adf = 12i32;
                        if avl < uj - adf || avl > wv + adf { continue; }
                        
                        let jf = crate::visualizer::khf(
                            &self.visualizer, bj, avl,
                            self.visualizer.ana, fnk,
                        );
                        
                        
                        if jf.tq == 0 && jf.dcm == 0 { continue; }
                        
                        
                        let ar = if a == 0 { 180u8 }
                            else { 120u8.ao((a as u8).mbq(7)) };
                        if ar < 10 { continue; }
                        
                        
                        let kpx = (ar as u32 / 8) as u8;
                        let kpw = (ar as u32 / 3) as u8;
                        let kpv = (ar as u32 / 7) as u8;
                        let (mut hsb, mut csn, mut csm) = crate::visualizer::lmn(
                            kpx, kpw, kpv,
                            jf.tq, jf.eo, jf.apb,
                            jf.aug, jf.amv,
                            jf.atv, jf.axo, jf.ys, jf.ata, jf.zc,
                            eez, fnk,
                            jel, jej, jek, jem,
                            self.visualizer.aim,
                        );
                        
                        if jf.dcm > 0 {
                            let ab = jf.dcm as f32 / 255.0;
                            let wq = 1.0 - ab;
                            hsb = (hsb as f32 * wq + jf.ejq as f32 * ab) as u8;
                            csn = (csn as f32 * wq + jf.ejn as f32 * ab) as u8;
                            csm = (csm as f32 * wq + jf.ejl as f32 * ab) as u8;
                        }
                        
                        let s = 0xFF000000 | ((hsb as u32) << 16) | ((csn as u32) << 8) | (csm as u32);
                        
                        
                        let aap = kvw.cn(
                            (a as u32 * 7919) ^ (self.oo as u32 / 8)
                        );
                        let ntr: &[u8] = b"@#$%&WM8BOX0ZNHK";
                        let r = ntr[(aap as usize) % ntr.len()] as char;
                        let ka = crate::framebuffer::font::ada(r);
                        
                        for (erm, &fs) in ka.iter().cf() {
                            let x = avl as u32 + erm as u32;
                            if x >= ac || fs == 0 { continue; }
                            if !ggv.abq() {
                                let dvh = x as usize * nsx as usize;
                                for ga in 0..8u32 {
                                    if fs & (0x80 >> ga) != 0 {
                                        let y = b + ga;
                                        if y < z {
                                            unsafe { *ggv.add(dvh + y as usize) = s; }
                                        }
                                    }
                                }
                            } else {
                                for ga in 0..8u32 {
                                    if fs & (0x80 >> ga) != 0 {
                                        let y = b + ga;
                                        if y < z {
                                            framebuffer::ii(y, x, s);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if self.oo < 5 { crate::serial_println!("[FRAME] #{} rain+fill done", self.oo); }
        
        
        
        if self.oo < 5 { crate::serial_println!("[FRAME] #{} logo start", self.oo); }
        
        
        
        if !hoz {
            let cfz = crate::logo_bitmap::AY_ as u32;
            let cfy = crate::logo_bitmap::BL_ as u32;
            let euh = (z / 2).ao(cfz / 2);
            let eui = okf.ao(cfy / 2);
            
            
            
            
            if self.asr >= DesktopTier::Bv && self.oo % 4 == 0 {
                for ct in (0..cfy).akt(2) {
                    for mj in 0..cfz {
                        if !crate::logo_bitmap::hqj(mj as usize, ct as usize) { continue; }
                        let y = euh + mj;
                        let x = eui + ct;
                        if y >= z || x >= ac { continue; }
                        let kzg: u32 = if csf { 35 + (eez * 50.0) as u32 } else { 30 };
                        
                        framebuffer::ii(y, x, 0xFF000000 | (kzg.v(255) << 8));
                    }
                }
            }
            
            
            for ct in 0..cfy {
                for mj in 0..cfz {
                    let bax = crate::logo_bitmap::djc(mj as usize, ct as usize);
                    let q = (bax >> 24) & 0xFF;
                    let m = (bax >> 16) & 0xFF;
                    let at = (bax >> 8) & 0xFF;
                    let o = bax & 0xFF;
                    
                    if q < 20 { continue; }
                    let hqm = (m * 77 + at * 150 + o * 29) >> 8;
                    if hqm < 30 { continue; }
                    
                    let y = euh + mj;
                    let x = eui + ct;
                    if y >= z || x >= ac { continue; }
                    
                    if hqm >= 60 {
                        let kcn = if csf { (eez * 20.0).v(30.0) as u32 } else { 0 };
                        let oc = (m + kcn).v(255);
                        let bah = (at + kcn).v(255);
                        let ue = (o + kcn).v(255);
                        framebuffer::ii(y, x, 0xFF000000 | (oc << 16) | (bah << 8) | ue);
                    } else {
                        let dw = ((hqm as u32) * 255 / 60).v(255);
                        let ei = framebuffer::iwt(y, x);
                        let wq = 255 - dw;
                        let nr = (m * dw + ((ei >> 16) & 0xFF) * wq) / 255;
                        let csu = (at * dw + ((ei >> 8) & 0xFF) * wq) / 255;
                        let csq = (o * dw + (ei & 0xFF) * wq) / 255;
                        framebuffer::ii(y, x, 0xFF000000 | (nr << 16) | (csu << 8) | csq);
                    }
                }
            }
        }
        
        
    }
    
    
    fn ynp(&self, bja: &crate::theme::Uy, akr: u32) {
        use crate::theme::WallpaperMode;
        let ev = crate::theme::Ib.read().bsx.ev;
        
        match ev {
            WallpaperMode::Uq => {
                
                let gww = bja.z;
                let pzt = bja.ac;
                let pgk = self.z;
                
                for cq in 0..akr {
                    
                    let pmy = (cq as u64 * ((pzt as u64 - 1) << 8)) / akr as u64;
                    let fo = (pmy >> 8) as u32;
                    let dp = (fo + 1).v(pzt - 1);
                    let sc = (pmy & 0xFF) as u32; 
                    let gjm = 256 - sc;
                    
                    for cr in 0..pgk {
                        let pmx = (cr as u64 * ((gww as u64 - 1) << 8)) / pgk as u64;
                        let fy = (pmx >> 8) as u32;
                        let dn = (fy + 1).v(gww - 1);
                        let jf = (pmx & 0xFF) as u32;
                        let gjl = 256 - jf;
                        
                        
                        let tqu = (fo * gww + fy) as usize;
                        let tqw = (fo * gww + dn) as usize;
                        let tqv = (dp * gww + fy) as usize;
                        let oct = (dp * gww + dn) as usize;
                        
                        if oct < bja.hz.len() {
                            let byn = bja.hz[tqu];
                            let cdn = bja.hz[tqw];
                            let byo = bja.hz[tqv];
                            let cdo = bja.hz[oct];
                            
                            
                            let m = ( ((byn >> 16) & 0xFF) * gjl * gjm
                                    + ((cdn >> 16) & 0xFF) * jf * gjm
                                    + ((byo >> 16) & 0xFF) * gjl * sc
                                    + ((cdo >> 16) & 0xFF) * jf * sc ) >> 16;
                            let at = ( ((byn >> 8) & 0xFF) * gjl * gjm
                                    + ((cdn >> 8) & 0xFF) * jf * gjm
                                    + ((byo >> 8) & 0xFF) * gjl * sc
                                    + ((cdo >> 8) & 0xFF) * jf * sc ) >> 16;
                            let o = ( (byn & 0xFF) * gjl * gjm
                                    + (cdn & 0xFF) * jf * gjm
                                    + (byo & 0xFF) * gjl * sc
                                    + (cdo & 0xFF) * jf * sc ) >> 16;
                            
                            framebuffer::ii(cr, cq, 0xFF000000 | (m << 16) | (at << 8) | o);
                        }
                    }
                }
            }
            WallpaperMode::Eo => {
                
                let ei = crate::theme::colors().cop;
                framebuffer::ah(0, 0, self.z, akr, ei);
                
                let dtw = self.z.ao(bja.z) / 2;
                let dtx = akr.ao(bja.ac) / 2;
                
                for c in 0..bja.ac.v(akr) {
                    for b in 0..bja.z.v(self.z) {
                        let w = (c * bja.z + b) as usize;
                        if w < bja.hz.len() {
                            framebuffer::ii(dtw + b, dtx + c, bja.hz[w]);
                        }
                    }
                }
            }
            WallpaperMode::Azw => {
                
                let mut bg = 0;
                while bg < akr {
                    let mut dx = 0;
                    while dx < self.z {
                        for c in 0..bja.ac {
                            if bg + c >= akr { break; }
                            for b in 0..bja.z {
                                if dx + b >= self.z { break; }
                                let w = (c * bja.z + b) as usize;
                                if w < bja.hz.len() {
                                    framebuffer::ii(dx + b, bg + c, bja.hz[w]);
                                }
                            }
                        }
                        dx += bja.z;
                    }
                    bg += bja.ac;
                }
            }
            _ => {
                
                let s = crate::theme::Ib.read().bsx.hiv;
                framebuffer::ah(0, 0, self.z, akr, s);
            }
        }
    }
    
    
    
    
    fn yng(&self) {
        let yv = self.z / 2;
        let uq = (self.ac - W_) / 2 - 30;
        
        
        let cyh = 0xFF50E050u32;  
        let thm = 0xFF1A6B1Au32;    
        let kdn = 0xFF080808u32;     
        let qaw = 0xFFC0E020u32;  
        let hcu = 0xFF40C040u32; 
        let oqt = 0xFF60E060u32;     
        let zsj = 0xFF999999u32;      
        let zsk = 0xFF40CC40u32;     
        let lrc = 0xFF30A030u32; 
        
        
        let cbt = 80u32;
        let dlu = 100u32;
        let cr = yv - cbt / 2;
        let cq = uq - dlu / 2;
        
        for c in 0..dlu {
            let bkx = c as f32 / dlu as f32;
            let mqp = if bkx < 0.45 {
                1.0
            } else {
                let ab = (bkx - 0.45) / 0.55;
                (1.0 - ab * ab).am(0.0)
            };
            let d = (cbt as f32 * mqp).am(2.0) as u32;
            let aze = (cbt - d) / 2;
            
            for dx in 0..d {
                let y = cr + aze + dx;
                let x = cq + c;
                
                let bhi = aze + dx;
                let kpu = (bhi as f32 / cbt as f32) + (bkx * 0.2);
                let vi = if kpu < 0.5 { kdn } else { thm };
                framebuffer::ii(y, x, vi);
            }
            
            
            if d > 2 {
                framebuffer::ii(cr + aze, cq + c, lrc);
                framebuffer::ii(cr + aze + d - 1, cq + c, lrc);
            }
        }
        
        framebuffer::zs(cr, cq, cbt, lrc);
        
        
        let dsv = yv;
        let hqg = cq + 30;
        
        for bg in 0..14u32 {
            for dx in 0..20u32 {
                let ym = dx as i32 - 10;
                let wl = bg as i32;
                let bwk = 10i32;
                let dut = 6i32;
                if wl <= bwk && (ym * ym + (wl - bwk) * (wl - bwk)) <= bwk * bwk
                   && (ym * ym + (wl - bwk) * (wl - bwk)) >= dut * dut {
                    framebuffer::ii(dsv - 10 + dx, hqg - 14 + bg, qaw);
                }
            }
        }
        
        framebuffer::ah(dsv - 12, hqg, 24, 18, qaw);
        
        for bg in 0..6u32 {
            for dx in 0..6u32 {
                let ym = dx as i32 - 3;
                let wl = bg as i32 - 3;
                if ym * ym + wl * wl <= 9 {
                    framebuffer::ii(dsv - 3 + dx, hqg + 4 + bg, kdn);
                }
            }
        }
        
        framebuffer::ah(dsv - 1, hqg + 9, 3, 5, kdn);
        
        
        let fmh = hqg + 18;
        let lhd = cq + dlu + 50;
        
        
        for lhn in fmh..lhd {
            framebuffer::ii(dsv - 1, lhn, hcu);
            framebuffer::ii(dsv, lhn, hcu);
            framebuffer::ii(dsv + 1, lhn, hcu);
        }
        
        
        let ket: &[(u32, i32, u32)] = &[
            (fmh + 8, -20, 6),   
            (fmh + 8, 18, 6),    
            (fmh + 22, -25, 5),  
            (fmh + 22, 22, 5),   
            (fmh + 36, -15, 4),  
            (fmh + 36, 15, 4),   
        ];
        
        for &(je, hbw, gnu) in ket {
            if je >= self.ac.ao(W_) { continue; }
            
            let iaq: i32 = if hbw < 0 { -1 } else { 1 };
            let qeo = if hbw < 0 { -hbw } else { hbw };
            for dx in 0..qeo {
                let y = (dsv as i32 + iaq * dx) as u32;
                if y < self.z {
                    framebuffer::ii(y, je, hcu);
                    framebuffer::ii(y, je + 1, hcu);
                }
            }
            
            let uuw = (dsv as i32 + hbw) as u32;
            for opb in 0..gnu {
                for opa in 0..gnu {
                    let ym = opa as i32 - gnu as i32 / 2;
                    let wl = opb as i32 - gnu as i32 / 2;
                    if ym * ym + wl * wl <= (gnu as i32 / 2) * (gnu as i32 / 2) {
                        let y = uuw + opa;
                        let x = je + opb;
                        if y < self.z && x < self.ac.ao(W_) {
                            framebuffer::ii(y, x, oqt);
                        }
                    }
                }
            }
        }
        
        
        if lhd + 4 < self.ac.ao(W_) {
            for bg in 0..8u32 {
                for dx in 0..8u32 {
                    let ym = dx as i32 - 4;
                    let wl = bg as i32 - 4;
                    if ym * ym + wl * wl <= 16 {
                        framebuffer::ii(dsv - 4 + dx, lhd + bg, oqt);
                    }
                }
            }
        }
        
    }
    
    fn hgt(&self) {
        
        
        
        
        let cjz = self.ac.ao(W_);
        
        
        
        for bg in 0..cjz {
            for dx in 0..(BY_ + 10) {
                let xy = framebuffer::iwt(dx, bg);
                let bqm = ((xy >> 16) & 0xFF) as u32;
                let fhj = ((xy >> 8) & 0xFF) as u32;
                let ebc = (xy & 0xFF) as u32;
                
                let nr = (bqm * 25 / 100 + 4 * 75 / 100).v(255);
                let csu = (fhj * 25 / 100 + 8 * 75 / 100).v(255);
                let csq = (ebc * 25 / 100 + 4 * 75 / 100).v(255);
                framebuffer::ii(dx, bg, 0xFF000000 | (nr << 16) | (csu << 8) | csq);
            }
        }
        
        framebuffer::ah(BY_ + 9, 0, 1, cjz, AJ_);
        
        let aih = 36u32;
        let eva = self.icons.len().am(1) as u32;
        let ob = 12u32;
        let bfz = cjz.ao(ob * 2);
        let crk = bfz / eva;
        let vc = ob + (bfz - crk * eva) / 2;
        
        for (a, pa) in self.icons.iter().cf() {
            let fg = 12u32;
            let og = vc + (a as u32) * crk;
            if og + aih > cjz { break; }
            
            
            let apx = self.lf >= 0 && self.lf < (BY_ + 10) as i32
                && self.ot >= og as i32 && self.ot < (og + crk) as i32;
            
            
            let xd = if apx { I_ } else { BH_ };
            let bbw = if apx { I_ } else { 0xFF556655 };
            
            
            if apx {
                
                let cye = 6u32;
                let qz = fg.ao(cye);
                let ub = og.ao(cye);
                let nt = aih + cye * 2;
                let bjz = aih + 20 + cye * 2;
                for hlb in 0..bjz {
                    for hla in 0..nt {
                        let y = qz + hla;
                        let x = ub + hlb;
                        if y >= BY_ + 10 || x >= cjz { continue; }
                        
                        let yz = if hla < cye { cye - hla } 
                            else if hla > nt - cye { hla - (nt - cye) } 
                            else { 0 };
                        let jae = if hlb < cye { cye - hlb }
                            else if hlb > bjz - cye { hlb - (bjz - cye) }
                            else { 0 };
                        let la = yz.am(jae);
                        if la > 0 {
                            let hj = (20u32.ao(la * 4)).v(20) as u8;
                            if hj > 0 {
                                let xy = framebuffer::iwt(y, x);
                                let fhj = ((xy >> 8) & 0xFF) as u8;
                                let fou = fhj.akq(hj);
                                let dei = (xy & 0xFFFF00FF) | ((fou as u32) << 8);
                                framebuffer::ii(y, x, dei);
                            }
                        }
                    }
                }
                
                mf((fg as i32) - 3, (og as i32) - 2, aih + 6, aih + 16, 6, 0xFF001A0A);
                tf((fg as i32) - 3, (og as i32) - 2, aih + 6, aih + 16, 6, GC_);
            }
            
            
            let axm = match pa.ecz {
                IconType::Ay => 0xFF20CC60u32,  
                IconType::Aig => 0xFFDDAA30u32,    
                IconType::Ahq => 0xFF5090E0u32,    
                IconType::Calculator => 0xFFCC6633u32, 
                IconType::As => 0xFF40AADDu32,    
                IconType::Io => 0xFFCC4444u32,       
                IconType::Gs => 0xFFEECC88u32,      
                IconType::Gn => 0xFF9988BBu32,   
                IconType::Browser => 0xFF4488DDu32,    
                IconType::Aij => 0xFF88BB44u32,    
                _ => xd,
            };
            
            mf(fg as i32, og as i32, aih, aih, 6, 0xFF060A06);
            if apx {
                
                tf(fg as i32, og as i32, aih, aih, 6, axm);
            } else {
                tf(fg as i32, og as i32, aih, aih, 6, AJ_);
            }
            
            
            let ads = if apx { axm } else { xd };
            
            
            let cx = fg + aih / 2;
            let ae = og + aih / 2;
            use crate::icons::IconType;
            match pa.ecz {
                IconType::Ay => {
                    
                    
                    mf((cx - 15) as i32, (ae - 11) as i32, 30, 22, 3, ads);
                    
                    framebuffer::ah(cx - 13, ae - 9, 26, 16, 0xFF050A05);
                    
                    framebuffer::zs(cx - 13, ae - 9, 26, axm);
                    
                    self.cb((cx - 9) as i32, (ae - 5) as i32, "$", 0xFF40FF60);
                    framebuffer::ah(cx - 3, ae - 3, 8, 2, 0xFF40FF60);
                    
                    framebuffer::ah(cx - 3, ae + 8, 6, 3, ads);
                    framebuffer::ah(cx - 6, ae + 10, 12, 2, ads);
                },
                IconType::Aig => {
                    
                    
                    mf((cx - 14) as i32, (ae - 10) as i32, 14, 6, 2, ads);
                    
                    mf((cx - 14) as i32, (ae - 5) as i32, 28, 18, 2, ads);
                    
                    framebuffer::ah(cx - 12, ae - 3, 24, 13, 0xFF0A0A06);
                    
                    framebuffer::ah(cx - 8, ae, 14, 1, 0xFF404020);
                    framebuffer::ah(cx - 8, ae + 3, 10, 1, 0xFF404020);
                    framebuffer::ah(cx - 8, ae + 6, 16, 1, 0xFF404020);
                    
                    framebuffer::ah(cx - 2, ae - 5, 4, 2, axm);
                },
                IconType::Ahq => {
                    
                    
                    mf((cx - 11) as i32, (ae - 13) as i32, 22, 26, 2, ads);
                    
                    framebuffer::ah(cx + 5, ae - 13, 6, 6, 0xFF0A0A0A);
                    framebuffer::zs(cx + 5, ae - 13, 1, ads);
                    framebuffer::axt(cx + 5, ae - 13, 6, ads);
                    framebuffer::zs(cx + 5, ae - 8, 6, ads);
                    
                    framebuffer::ah(cx - 9, ae - 7, 18, 18, 0xFF080C08);
                    
                    for br in 0..5u32 {
                        framebuffer::ah(cx - 8, ae - 5 + br * 3, 2, 1, 0xFF335533);
                    }
                    
                    framebuffer::ah(cx - 4, ae - 5, 7, 1, 0xFF6688CC);  
                    framebuffer::ah(cx - 4, ae - 2, 10, 1, ads);  
                    framebuffer::ah(cx - 4, ae + 1, 6, 1, 0xFFCC8844);  
                    framebuffer::ah(cx - 4, ae + 4, 12, 1, ads);  
                    framebuffer::ah(cx - 4, ae + 7, 5, 1, 0xFF88BB44);  
                },
                IconType::Calculator => {
                    
                    mf((cx - 11) as i32, (ae - 13) as i32, 22, 26, 3, ads);
                    
                    framebuffer::ah(cx - 9, ae - 11, 18, 22, 0xFF0C0C0A);
                    
                    mf((cx - 8) as i32, (ae - 10) as i32, 16, 7, 1, 0xFF1A3320);
                    self.cb((cx - 5) as i32, (ae - 10) as i32, "42", 0xFF40FF40);
                    
                    for br in 0..3u32 {
                        for bj in 0..4u32 {
                            let bx = cx - 8 + bj * 5;
                            let je = ae - 0 + br * 4;
                            let qsi = if bj == 3 { axm } else { ads };
                            framebuffer::ah(bx, je, 3, 2, qsi);
                        }
                    }
                },
                IconType::As => {
                    
                    let qke = cx as i32;
                    let mwd = (ae + 6) as i32;
                    
                    for mz in 0..3u32 {
                        let m = 5 + mz * 4;
                        let uv = (m * m) as i32;
                        let jkx = ((m.ao(2)) * (m.ao(2))) as i32;
                        for bg in -(m as i32)..=0 {
                            for dx in -(m as i32)..=(m as i32) {
                                let dgk = dx * dx + bg * bg;
                                if dgk <= uv && dgk >= jkx {
                                    let y = (qke + dx) as u32;
                                    let x = (mwd + bg) as u32;
                                    if y >= fg && y < fg + aih && x >= og && x < og + aih {
                                        let s = if mz == 0 { 
                                            if apx { axm } else { P_ }
                                        } else if mz == 1 { 
                                            if apx { axm } else { BH_ }
                                        } else { 
                                            ads 
                                        };
                                        framebuffer::ii(y, x, s);
                                    }
                                }
                            }
                        }
                    }
                    
                    for bg in -1..=1i32 {
                        for dx in -1..=1i32 {
                            if dx*dx+bg*bg <= 1 {
                                framebuffer::ii((cx as i32 + dx) as u32, (mwd + bg) as u32, ads);
                            }
                        }
                    }
                },
                IconType::Io => {
                    
                    
                    mf((cx - 15) as i32, (ae - 6) as i32, 30, 16, 5, ads);
                    
                    framebuffer::ah(cx - 13, ae - 4, 26, 12, 0xFF0A0A0A);
                    
                    mf((cx - 15) as i32, (ae - 2) as i32, 6, 10, 2, ads);
                    
                    mf((cx + 9) as i32, (ae - 2) as i32, 6, 10, 2, ads);
                    
                    framebuffer::ah(cx - 10, ae - 1, 7, 2, ads);
                    framebuffer::ah(cx - 8, ae - 3, 2, 7, ads);
                    
                    framebuffer::ah(cx + 4, ae - 3, 3, 3, 0xFF4488DD);  
                    framebuffer::ah(cx + 8, ae - 1, 3, 3, DC_);  
                    framebuffer::ah(cx + 4, ae + 1, 3, 3, 0xFF44DD44);  
                    framebuffer::ah(cx + 1, ae - 1, 3, 3, 0xFFDDDD44);  
                },
                IconType::Gs => {
                    
                    let fz = if apx { 0xFFFFDD88 } else { ads };
                    
                    framebuffer::ah(cx - 8, ae + 6, 16, 4, fz);
                    
                    framebuffer::ah(cx - 6, ae + 2, 12, 4, fz);
                    
                    framebuffer::ah(cx - 4, ae - 6, 8, 8, fz);
                    
                    framebuffer::ah(cx - 6, ae - 10, 3, 5, fz);
                    framebuffer::ah(cx - 1, ae - 12, 2, 7, fz);
                    framebuffer::ah(cx + 3, ae - 10, 3, 5, fz);
                    
                    framebuffer::ah(cx - 1, ae - 14, 2, 4, fz);
                    framebuffer::ah(cx - 2, ae - 13, 4, 2, fz);
                },
                IconType::Gn => {
                    
                    for bg in 0..20u32 {
                        for dx in 0..20u32 {
                            let ym = dx as i32 - 10;
                            let wl = bg as i32 - 10;
                            let ass = ym * ym + wl * wl;
                            
                            if ass >= 36 && ass <= 72 {
                                framebuffer::ii(cx - 10 + dx, ae - 10 + bg, ads);
                            }
                            
                            if ass <= 12 {
                                framebuffer::ii(cx - 10 + dx, ae - 10 + bg, ads);
                            }
                            
                            if ass >= 10 && ass <= 16 {
                                framebuffer::ii(cx - 10 + dx, ae - 10 + bg, axm);
                            }
                        }
                    }
                    
                    let idb: &[(i32, i32)] = &[(0, -10), (0, 10), (-10, 0), (10, 0), (-7, -7), (7, -7), (-7, 7), (7, 7)];
                    for &(gx, ty) in idb {
                        let y = (cx as i32 + gx) as u32;
                        let x = (ae as i32 + ty) as u32;
                        framebuffer::ah(y.ao(2), x.ao(1), 4, 3, ads);
                    }
                },
                IconType::Browser => {
                    
                    for bg in 0..22u32 {
                        for dx in 0..22u32 {
                            let ym = dx as i32 - 11;
                            let wl = bg as i32 - 11;
                            let ass = ym * ym + wl * wl;
                            
                            if ass <= 110 {
                                framebuffer::ii(cx - 11 + dx, ae - 11 + bg, 0xFF0A1A2A);
                            }
                            
                            if ass >= 100 && ass <= 121 {
                                framebuffer::ii(cx - 11 + dx, ae - 11 + bg, ads);
                            }
                        }
                    }
                    
                    framebuffer::ah(cx - 10, ae, 20, 1, ads);
                    
                    framebuffer::ah(cx, ae - 10, 1, 20, ads);
                    
                    for bg in 0..20u32 {
                        let wl = bg as i32 - 10;
                        let ap = 100 - wl * wl;
                        if ap > 0 {
                            let iqg = (ggu(ap) * 2 / 5) as u32;
                            if cx + iqg < fg + aih {
                                framebuffer::ii(cx + iqg, ae - 10 + bg, ads);
                            }
                            if cx >= iqg + fg {
                                framebuffer::ii(cx.ao(iqg), ae - 10 + bg, ads);
                            }
                        }
                    }
                    
                    framebuffer::ah(cx - 9, ae - 5, 18, 1, ads);
                    framebuffer::ah(cx - 9, ae + 5, 18, 1, ads);
                },
                IconType::Aij => {
                    
                    mf((cx - 10) as i32, (ae - 13) as i32, 20, 26, 3, ads);
                    framebuffer::ah(cx - 8, ae - 11, 16, 22, 0xFF1A1A1A);
                    
                    mf((cx - 7) as i32, (ae - 10) as i32, 14, 11, 1, 0xFF1A3320);
                    
                    framebuffer::ah(cx - 2, ae - 8, 4, 4, 0xFF40CC40);
                    framebuffer::ah(cx - 3, ae - 4, 6, 2, 0xFF40CC40);
                    
                    framebuffer::ah(cx - 7, ae + 4, 5, 2, 0xFF333333);
                    framebuffer::ah(cx - 5, ae + 2, 2, 6, 0xFF333333);
                    
                    framebuffer::ah(cx + 3, ae + 3, 3, 3, DC_);
                    framebuffer::ah(cx + 1, ae + 5, 3, 3, 0xFF4488DD);
                    
                    for a in 0..3u32 {
                        framebuffer::ah(cx + 2 + a * 3, ae + 10, 1, 2, 0xFF333333);
                    }
                },
                IconType::Jf => {
                    
                    for bg in 0..20u32 {
                        for dx in 0..20u32 {
                            let ym = dx as i32 - 10;
                            let wl = bg as i32 - 10;
                            let ass = ym * ym + wl * wl;
                            if ass >= 72 && ass <= 100 {
                                framebuffer::ii(cx - 10 + dx, ae - 10 + bg, ads);
                            }
                        }
                    }
                    
                    framebuffer::ah(cx - 1, ae - 6, 2, 2, axm); 
                    framebuffer::ah(cx - 1, ae - 2, 2, 8, axm); 
                    framebuffer::ah(cx - 3, ae + 5, 6, 1, axm); 
                },
                IconType::Fp => {
                    
                    
                    let ggq = cx as i32 - 8;
                    let ggr = ae as i32 - 2;
                    framebuffer::ah(ggq as u32, ggr as u32, 14, 12, 0xFF162016);
                    framebuffer::lx(ggq as u32, ggr as u32, 14, 12, ads);
                    
                    for a in 0..14i32 {
                        framebuffer::ii((ggq + a + 4) as u32, (ggr - 4) as u32, ads);
                        framebuffer::ii((ggq + a + 2) as u32, (ggr - 2) as u32, axm);
                    }
                    
                    framebuffer::axt((ggq + 17) as u32, (ggr - 4) as u32, 12, ads);
                    for fb in 0..4u32 {
                        framebuffer::ii((ggq + 14 + fb as i32) as u32, (ggr + fb as i32 - 4) as u32, ads);
                    }
                },
                IconType::Lm => {
                    
                    framebuffer::ah(cx - 3, ae - 12, 6, 8, ads); 
                    framebuffer::ah(cx - 5, ae - 12, 10, 2, ads); 
                    
                    for br in 0..10u32 {
                        let abd = 3 + br;
                        framebuffer::ah(cx.ao(abd), ae - 4 + br, abd * 2, 1, ads);
                    }
                    
                    for br in 4..10u32 {
                        let abd = br;
                        framebuffer::ah(cx.ao(abd) + 1, ae - 4 + br, (abd * 2).ao(2), 1, axm);
                    }
                    
                    framebuffer::ah(cx - 2, ae + 1, 2, 2, 0xFF80FF80);
                    framebuffer::ah(cx + 1, ae + 3, 2, 2, 0xFF80FF80);
                },
                _ => {
                    
                    tf((cx - 10) as i32, (ae - 10) as i32, 20, 20, 3, ads);
                    for a in 0..6i32 {
                        framebuffer::ii((cx as i32 + a) as u32, (ae as i32 - a) as u32, axm);
                        framebuffer::ii((cx as i32 - a) as u32, (ae as i32 - a) as u32, axm);
                        framebuffer::ii((cx as i32 + a) as u32, (ae as i32 + a) as u32, axm);
                        framebuffer::ii((cx as i32 - a) as u32, (ae as i32 + a) as u32, axm);
                    }
                },
            }
            
            
            let j = &pa.j;
            let bda = j.len() as u32 * 8;
            let wg = fg + (aih / 2).ao(bda / 2);
            self.en(wg as i32, (og + aih + 2) as i32, j, bbw);
        }
    }
    
    fn hgx(&mut self) {
        let c = self.ac - W_;
        
        
        
        
        
        
        {
            let dy = 6u32;
            let jl = dy as i32;
            let uv = jl * jl;
            let d = self.z;
            
            
            for br in 0..dy {
                let igl = jl - br as i32;
                let iyo = ggu(uv - igl * igl) as u32;
                let oiq = dy - iyo;
                let pyj = d.ao(oiq * 2);
                if pyj > 0 {
                    framebuffer::ih(oiq, c + br, pyj, 1, 0x040A06, 165);
                }
            }
            
            framebuffer::ih(0, c + dy, d, W_ - dy, 0x040A06, 165);
            
            framebuffer::ih(0, c, d, W_, 0x00AA44, 10);
            
            if d > dy * 2 {
                for y in dy..(d - dy) {
                    framebuffer::ii(y, c, AT_);
                }
            }
            
            
            for br in 0..dy {
                let igl = jl - br as i32;
                let iyo = ggu(uv - igl * igl) as u32;
                let eua = dy - iyo;
                let dvc = d - dy + iyo;
                if eua < d {
                    framebuffer::ii(eua, c + br, AT_);
                }
                if dvc > 0 && dvc - 1 < d {
                    framebuffer::ii(dvc - 1, c + br, AT_);
                }
            }
        }
        
        
        let jrl = self.lf >= 4 && self.lf < 120 && self.ot >= c as i32;
        if jrl || self.ajo {
            mf(6, (c + 7) as i32, 110, 34, 10, 0xFF003318);
            framebuffer::ih(6, c + 7, 110, 34, 0x00CC66, 60);
            
            framebuffer::ih(4, c + 5, 114, 1, 0x00FF66, 25);
        }
        let aia = if jrl || self.ajo { EC_ } else { AJ_ };
        tf(6, (c + 7) as i32, 110, 34, 10, aia);
        let pwr = if jrl || self.ajo { I_ } else { AG_ };
        self.en(20, (c + 15) as i32, "TrustOS", pwr);
        
        if jrl || self.ajo {
            self.en(21, (c + 15) as i32, "TrustOS", pwr);
        }
        
        
        let ied = self.ee.len();
        let pm = 96u32;
        let qx = 34u32;
        let aib = 6u32;
        let aza = if ied > 0 { ied as u32 * (pm + aib) - aib } else { 0 };
        let ql = (self.z.ao(aza)) / 2;
        
        for (a, d) in self.ee.iter().cf() {
            let axp = ql + a as u32 * (pm + aib);
            let kn = c + 7;
            
            let jbk = self.lf >= axp as i32 && self.lf < (axp + pm) as i32
                && self.ot >= c as i32;
            
            
            if d.ja {
                mf(axp as i32, kn as i32, pm, qx, 8, 0xFF001A0A);
                framebuffer::ih(axp, kn, pm, qx, 0x00AA44, 70);
                
                framebuffer::ih(axp + 4, kn, pm - 8, 1, 0x00FF66, 35);
            } else if jbk {
                mf(axp as i32, kn as i32, pm, qx, 8, 0xFF000D05);
                framebuffer::ih(axp, kn, pm, qx, 0x008833, 50);
            }
            
            let qok = if d.ja { EC_ } else if jbk { GC_ } else { AJ_ };
            tf(axp as i32, kn as i32, pm, qx, 8, qok);
            
            
            let hnp = match d.ld {
                WindowType::Ay => ">_",
                WindowType::Ak => "[]",
                WindowType::Calculator => "##",
                WindowType::Browser => "WW",
                WindowType::Ag => "Tx",
                WindowType::Io => "Sk",
                WindowType::Lw => "Mu",
                _ => "::",
            };
            let xd = if d.ja { I_ } else { P_ };
            self.en((axp + 8) as i32, (kn + 10) as i32, hnp, xd);
            
            
            let xhw = 7;
            let dq: String = d.dq.bw().take(xhw).collect();
            let agx = if d.ja { I_ } else { BK_ };
            self.en((axp + 28) as i32, (kn + 10) as i32, &dq, agx);
            
            
            if d.ja {
                let ldx = 60u32.v(pm - 14);
                let oed = axp + (pm - ldx) / 2;
                mf((oed) as i32, (c + W_ - 5) as i32, ldx, 3, 1, I_);
                framebuffer::ih(oed.ao(2), c + W_ - 7, ldx + 4, 2, I_, 50);
            } else if !d.aat {
                let hgo = axp + pm / 2 - 2;
                framebuffer::ah(hgo, c + W_ - 4, 4, 2, BH_);
            }
        }
        
        
        
        let gux = 12u32; 
        
        
        let mut dms = self.z - 8 - 8 - gux; 
        
        
        let taq = 20u32;
        let fji = dms - taq;
        let iwl = c + 16;
        dms = fji - gux;
        
        
        let rbs = 64u32;
        let ioc = dms - rbs;
        let time = self.tev();
        self.en(ioc as i32, (c + 10) as i32, &time, bei(I_, 0xFFFFFFFF));
        
        self.en((ioc + 1) as i32, (c + 10) as i32, &time, bei(I_, 0xFFFFFFFF));
        let hff = self.tdg();
        self.en(ioc as i32, (c + 27) as i32, &hff, bei(BK_, 0xFFCCCCCC));
        dms = ioc - gux;
        
        
        let qnb = 36u32;
        let dij = dms - qnb;
        let gjv = c + 8;
        let ngo = ((self.oo % 7) + 2).v(6) as u32;
        self.cb(dij as i32, (gjv + 2) as i32, "C", P_);
        let ikt = dij + 12;
        for pk in 0..8u32 {
            let mde = if pk < ngo {
                if ngo > 6 { DC_ } else { I_ }
            } else { P_ };
            framebuffer::ah(ikt + pk * 3, gjv + 3, 2, 8, mde);
        }
        let omt = {
            let es = 16u32;
            let mr = ((self.ee.len() as u32 * 2) + 4).v(es);
            (mr * 8 / es).v(8)
        };
        self.cb(dij as i32, (gjv + 17) as i32, "M", P_);
        for pk in 0..8u32 {
            let mde = if pk < omt {
                if omt > 6 { FY_ } else { I_ }
            } else { P_ };
            framebuffer::ah(ikt + pk * 3, gjv + 18, 2, 8, mde);
        }
        dms = dij - gux;
        
        
        let ghn = format!("{}fps", self.cya);
        let kwy = if self.cya >= 55 { AG_ } else if self.cya >= 30 { FY_ } else { DC_ };
        let swp = (ghn.len() as u32) * 8 + 4;
        let hkg = dms - swp;
        self.en(hkg as i32, (c + 17) as i32, &ghn, kwy);
        dms = hkg - gux;
        
        
        let jyo = crate::accessibility::wts();
        if !jyo.is_empty() {
            let qef = (jyo.len() as u32) * 8 + 4;
            let mta = dms - qef;
            self.en(mta as i32, (c + 17) as i32, &jyo, bei(FY_, 0xFFFFFF00));
            dms = mta - gux;
        }
        
        
        let xmd = 100u32;
        let jue = dms - xmd;
        self.sfy(jue, c + 10);
        let nxi = self.lf >= (fji as i32 - 4) && self.lf < (fji as i32 + 20)
            && self.ot >= c as i32;
        let kxx = if nxi { I_ } else { BK_ };
        if nxi {
            framebuffer::ih(fji - 2, iwl - 2, 20, 20, 0x00CC66, 30);
        }
        for bg in 0..16u32 {
            for dx in 0..16u32 {
                let ym = dx as i32 - 8;
                let wl = bg as i32 - 8;
                let ass = ym * ym + wl * wl;
                if ass >= 25 && ass <= 56 {
                    framebuffer::ii(fji + dx, iwl + bg, kxx);
                }
                if ass <= 6 {
                    framebuffer::ii(fji + dx, iwl + bg, kxx);
                }
            }
        }
        let idb: &[(i32, i32)] = &[(0, -8), (0, 8), (-8, 0), (8, 0), (-6, -6), (6, -6), (-6, 6), (6, 6)];
        for &(gx, ty) in idb {
            let y = (fji as i32 + 8 + gx) as u32;
            let x = (iwl as i32 + 8 + ty) as u32;
            framebuffer::ah(y.ao(1), x.ao(1), 3, 3, kxx);
        }
        
        
        let mct = self.z - 8;
        let wfe = 8u32;
        let wfd = self.lf >= mct as i32 && self.ot >= c as i32;
        let wfc = if wfd { X_ } else { P_ };
        framebuffer::ah(mct, c, wfe, W_, wfc);
        framebuffer::ah(mct, c + 6, 1, W_ - 12, BH_);
    }
    
    fn tev(&mut self) -> String {
        
        
        if self.oo - self.jcq >= 60 || self.gbz.is_empty() {
            let os = crate::rtc::cgz();
            self.gbz = format!("{:02}:{:02}", os.bek, os.bri);
            self.hcd = format!("{:02}/{:02}", os.caw, os.cjw);
            self.jcq = self.oo;
        }
        self.gbz.clone()
    }
    
    fn tdg(&self) -> String {
        self.hcd.clone()
    }
    
    fn krl(&self) {
        let afr = 480u32;
        let aje = 680u32;
        let rs = 4i32;
        let xp = (self.ac - W_ - aje - 8) as i32;
        
        let hoy = crate::accessibility::edv();
        
        
        
        
        
        
        if hoy {
            framebuffer::ah(rs as u32, xp as u32, afr, aje, 0xFF000000);
        } else {
            mf(rs, xp, afr, aje, 14, 0xFF060A08);
            framebuffer::ih(rs as u32, xp as u32, afr, aje, 0x060A08, 185);
        }
        
        
        let qri = bei(GC_, 0xFFFFFFFF);
        tf(rs, xp, afr, aje, 14, qri);
        
        framebuffer::ih((rs + 14) as u32, xp as u32, afr - 28, 1, 0x00FF66, 20);
        
        
        if hoy {
            framebuffer::ah((rs + 2) as u32, (xp + 2) as u32, afr - 4, 28, 0xFF1A1A1A);
        } else {
            framebuffer::ih((rs + 2) as u32, (xp + 2) as u32, afr - 4, 28, 0x002200, 160);
        }
        self.en(rs + 14, xp + 8, "TrustOS Menu", bei(I_, 0xFFFFFF00));
        self.en(rs + 15, xp + 8, "TrustOS Menu", bei(I_, 0xFFFFFF00)); 
        
        
        framebuffer::zs((rs + 2) as u32, (xp + 30) as u32, afr - 4, P_);
        
        
        let blb = xp + 34;
        let ftv = 36u32;
        let grt = 12i32;
        let bco = afr - grt as u32 * 2;
        mf(rs + grt, blb, bco, ftv, 10, 0xFF0A120A);
        tf(rs + grt, blb, bco, ftv, 10, P_);
        
        framebuffer::ih((rs + grt + 4) as u32, blb as u32, bco - 8, 1, 0x00FF66, 15);
        
        
        let fnl = rs + grt + 12;
        let fnm = blb + 10;
        for bg in 0..10u32 {
            for dx in 0..10u32 {
                let ym = dx as i32 - 5;
                let wl = bg as i32 - 5;
                let la = ym * ym + wl * wl;
                if la >= 12 && la <= 25 {
                    framebuffer::ii((fnl + dx as i32) as u32, (fnm + bg as i32) as u32, BK_);
                }
            }
        }
        framebuffer::ah((fnl + 8) as u32, (fnm + 8) as u32, 4, 2, BK_);
        
        
        let mcw = rs + grt + 26;
        if self.bij.is_empty() {
            self.en(mcw, blb + 12, "Search apps...", P_);
        } else {
            self.en(mcw, blb + 12, &self.bij, I_);
            let lf = mcw + (self.bij.len() as i32 * 8);
            if self.btx {
                framebuffer::ah(lf as u32, (blb + 10) as u32, 2, 16, I_);
            }
        }
        
        let hpg = blb + ftv as i32 + 8;
        
        
        let pj: [(&str, &str, bool); 18] = [
            (">_", "Terminal", false),
            ("[]", "Files", false),
            ("##", "Calculator", false),
            ("~~", "NetScan", false),
            ("Tx", "Text Editor", false),
            ("/\\", "TrustEdit 3D", false),
            ("WW", "Browser", false),
            ("C3", "Chess 3D", false),
            ("Kk", "Chess 2D", false),
            ("Sk", "Snake", false),
            ("NE", "NES Emulator", false),
            ("GB", "Game Boy", false),
            ("Lb", "TrustLab", false),
            ("Mu", "Music Player", false),
            ("@)", "Settings", false),
            ("<-", "Exit Desktop", true),
            ("!!", "Shutdown", true),
            (">>", "Reboot", true),
        ];
        
        
        let anw = self.bij.em();
        let cbp: alloc::string::String = anw.bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
        
        
        let doz = 2u32;
        let ezu = (afr - 24) / doz;
        let dwt = 44u32;
        let ezt = 4u32;
        let mut fhb = 0usize;
        
        for (cfm, (pa, cu, tyz)) in pj.iter().cf() {
            if *tyz { continue; }
            
            if !cbp.is_empty() {
                let hpp: alloc::string::String = cu.bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
                if !hpp.contains(cbp.as_str()) {
                    continue;
                }
            }
            
            let bj = (fhb % doz as usize) as u32;
            let br = (fhb / doz as usize) as u32;
            let dsk = rs + 10 + bj as i32 * (ezu + ezt) as i32;
            let ajd = hpg + (br as i32 * (dwt + ezt) as i32);
            fhb += 1;
            
            
            if ajd + dwt as i32 > xp + aje as i32 - 110 { break; }
            
            let apx = self.lf >= dsk 
                && self.lf < dsk + ezu as i32
                && self.ot >= ajd 
                && self.ot < ajd + dwt as i32;
            let qe = self.bsl == cfm as i32;
            
            
            if apx || qe {
                mf(dsk, ajd, ezu, dwt, 8, 0xFF0A2A14);
                framebuffer::ih(dsk as u32, ajd as u32, ezu, dwt, 0x00AA44, if qe { 70 } else { 50 });
                tf(dsk, ajd, ezu, dwt, 8, P_);
            }
            
            
            let ese = dsk + 22;
            let esf = ajd + dwt as i32 / 2;
            let cyr = 14i32;
            let ocy = cyr * cyr;
            let trc = if apx || qe { 0xFF0A3A1A } else { 0xFF0C1810 };
            for bg in -cyr..=cyr {
                for dx in -cyr..=cyr {
                    if dx * dx + bg * bg <= ocy {
                        framebuffer::ii((ese + dx) as u32, (esf + bg) as u32, trc);
                    }
                }
            }
            
            for bg in -cyr..=cyr {
                for dx in -cyr..=cyr {
                    let us = dx * dx + bg * bg;
                    if us >= (cyr - 1) * (cyr - 1) && us <= ocy {
                        let atw = if apx || qe { X_ } else { P_ };
                        framebuffer::ii((ese + dx) as u32, (esf + bg) as u32, atw);
                    }
                }
            }
            
            
            let xd = if apx || qe { I_ } else { AG_ };
            self.en(ese - 8, esf - 6, pa, xd);
            
            
            let bbw = if apx || qe { I_ } else { AC_ };
            self.en(dsk + 42, esf - 6, cu, bbw);
        }
        
        
        let vkk = if cbp.is_empty() { true } else {
            pj[14..].iter().any(|(_, cu, _)| {
                let glq: String = cu.bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
                glq.contains(cbp.as_str())
            })
        };
        if fhb == 0 && !vkk && !cbp.is_empty() {
            let uuv = hpg + 12;
            self.en(rs + 40, uuv, "No results found", P_);
        }
        
        
        let jjs = xp + aje as i32 - 106;
        framebuffer::zs((rs + 12) as u32, jjs as u32, afr - 24, P_);
        
        let vkj: [(&str, &str, u8); 3] = [
            ("<-", "Exit Desktop", 14),
            ("!!", "Shutdown", 15),
            (">>", "Reboot", 16),
        ];
        
        for (akk, (pa, cu, w)) in vkj.iter().cf() {
            if !cbp.is_empty() {
                let hpp: String = cu.bw().map(|r| if r.crs() { (r as u8 + 32) as char } else { r }).collect();
                if !hpp.contains(cbp.as_str()) {
                    continue;
                }
            }
            
            let ajd = jjs + 8 + (akk as i32 * 30);
            let ali = 28u32;
            
            let apx = self.lf >= rs 
                && self.lf < rs + afr as i32
                && self.ot >= ajd 
                && self.ot < ajd + ali as i32;
            let qe = self.bsl == *w as i32;
            
            if apx || qe {
                mf(rs + 8, ajd, afr - 16, ali, 6, 0xFF1A0808);
                framebuffer::ih((rs + 8) as u32, ajd as u32, afr - 16, ali, 0xAA2222, if qe { 50 } else { 35 });
            }
            
            let xd = if apx || qe { DC_ } else { 0xFF994444 };
            self.en(rs + 18, ajd + 8, pa, xd);
            
            let bbw = if apx || qe { DC_ } else { 0xFFAA4444 };
            self.en(rs + 44, ajd + 8, cu, bbw);
        }
        
        
        let pyd = xp + aje as i32 - 22;
        framebuffer::zs((rs + 8) as u32, pyd as u32, afr - 16, P_);
        self.cb(rs + 14, pyd + 6, "TrustOS v0.4.2", BH_);
    }
    
    fn nnu(&self, bh: &Window) {
        let b = bh.b;
        let c = bh.c;
        let d = bh.z;
        let i = bh.ac;
        
        
        
        
        
        let avn = if bh.bkk { 0u32 } else { DBO_ };
        
        
        
        if !bh.bkk && d > 4 && i > 4 {
            if self.asr >= DesktopTier::Bv {
                framebuffer::ih((b + 10) as u32, (c + 10) as u32, d + 2, i + 2, 0x000000, 14);
                framebuffer::ih((b + 7) as u32, (c + 7) as u32, d + 2, i + 2, 0x000000, 18);
                framebuffer::ih((b + 5) as u32, (c + 5) as u32, d, i, 0x000000, 22);
                framebuffer::ih((b + 3) as u32, (c + 3) as u32, d, i, 0x000000, 16);
                framebuffer::ih((b + 1) as u32, (c + 1) as u32, d + 2, i + 2, 0x000000, 8);
            } else if self.asr >= DesktopTier::Gc {
                framebuffer::ih((b + 5) as u32, (c + 5) as u32, d, i, 0x000000, 30);
                framebuffer::ih((b + 2) as u32, (c + 2) as u32, d + 1, i + 1, 0x000000, 15);
            } else {
                
                framebuffer::ah((b + 4) as u32, (c + 4) as u32, d, i, 0xFF080808);
            }
            if bh.ja {
                
                framebuffer::ih((b - 1) as u32, (c - 1) as u32, d + 2, i + 2, 0x00FF66, 10);
            }
        }
        
        
        
        if self.asr >= DesktopTier::Bv {
            if avn > 0 {
                sfh(b, c, d, i, avn, 0x080C08, 160);
            } else {
                framebuffer::ih(b as u32, c as u32, d, i, 0x080C08, 160);
            }
        } else {
            if avn > 0 {
                mf(b, c, d, i, avn, 0xFF0A0E0A);
            } else {
                framebuffer::ah(b as u32, c as u32, d, i, 0xFF0A0E0A);
            }
        }
        
        
        let aia = if bh.ja {
            bei(GC_, 0xFFFFFFFF)
        } else {
            bei(AJ_, 0xFF888888)
        };
        let gbm = if bh.ja { P_ } else { AJ_ };
        if self.asr >= DesktopTier::Bv {
            
            if avn > 0 {
                tf(b, c, d, i, avn, aia);
                tf(b + 1, c + 1, d.ao(2), i.ao(2), avn.ao(1), gbm);
                tf(b + 2, c + 2, d.ao(4), i.ao(4), avn.ao(2), aia);
                tf(b + 3, c + 3, d.ao(6), i.ao(6), avn.ao(3), gbm);
            } else {
                framebuffer::lx(b as u32, c as u32, d, i, aia);
                framebuffer::lx((b + 1) as u32, (c + 1) as u32, d.ao(2), i.ao(2), gbm);
                framebuffer::lx((b + 2) as u32, (c + 2) as u32, d.ao(4), i.ao(4), aia);
                framebuffer::lx((b + 3) as u32, (c + 3) as u32, d.ao(6), i.ao(6), gbm);
            }
        } else {
            
            if avn > 0 {
                tf(b, c, d, i, avn, aia);
                tf(b + 1, c + 1, d.ao(2), i.ao(2), avn.ao(1), gbm);
            } else {
                framebuffer::lx(b as u32, c as u32, d, i, aia);
                framebuffer::lx((b + 1) as u32, (c + 1) as u32, d.ao(2), i.ao(2), gbm);
            }
        }
        
        
        if bh.ja && !bh.bkk && d > 20 && i > 20 {
            let amd = bh.lqj(self.lf, self.ot);
            let bzv = 0x00FF66u32;
            let erj = 40u32;
            let gt = 4u32;
            let bjz = if i > 4 { i - 4 } else { 1 };
            let nt = if d > 4 { d - 4 } else { 1 };
            match amd {
                ResizeEdge::Ap | ResizeEdge::Dp | ResizeEdge::Dt => {
                    framebuffer::ih(b as u32, (c + 2) as u32, gt, bjz, bzv, erj);
                }
                _ => {}
            }
            match amd {
                ResizeEdge::Ca | ResizeEdge::Dq | ResizeEdge::Du => {
                    framebuffer::ih((b + d as i32 - gt as i32) as u32, (c + 2) as u32, gt, bjz, bzv, erj);
                }
                _ => {}
            }
            match amd {
                ResizeEdge::Jd | ResizeEdge::Dp | ResizeEdge::Dq => {
                    framebuffer::ih((b + 2) as u32, c as u32, nt, gt, bzv, erj);
                }
                _ => {}
            }
            match amd {
                ResizeEdge::Hk | ResizeEdge::Dt | ResizeEdge::Du => {
                    framebuffer::ih((b + 2) as u32, (c + i as i32 - gt as i32) as u32, nt, gt, bzv, erj);
                }
                _ => {}
            }
        }

        
        
        
        let dwx = J_;
        let gug = (b + 3) as u32;
        let guf = d.ao(6);
        if self.asr >= DesktopTier::Bv {
            
            if bh.ja {
                framebuffer::ih(gug, (c + 3) as u32, guf, dwx - 3, 0x0E2210, 190);
                framebuffer::ih(gug, (c + 3) as u32, guf, 1, 0x00FF66, 30);
                framebuffer::ih(gug, (c + 4) as u32, guf, 1, 0x00CC55, 15);
            } else {
                framebuffer::ih(gug, (c + 3) as u32, guf, dwx - 3, 0x080C08, 175);
            }
        } else {
            
            if bh.ja {
                framebuffer::ah(gug, (c + 3) as u32, guf, dwx - 3, 0xFF0E2210);
            } else {
                framebuffer::ah(gug, (c + 3) as u32, guf, dwx - 3, 0xFF080C08);
            }
        }
        
        
        framebuffer::zs((b + 3) as u32, (c + dwx as i32) as u32, d.ao(6), 
            if bh.ja { P_ } else { AJ_ });

        
        
        
        let pm = 28u32;
        let qx = dwx - 4;
        let kn = (c + 3) as u32;
        let hl = self.lf;
        let ir = self.ot;
        
        
        let bdr = b + d as i32 - pm as i32 - 3;
        let ndp = hl >= bdr && hl < bdr + pm as i32 
            && ir >= kn as i32 && ir < kn as i32 + qx as i32;
        let enp = if ndp { 0xFFCC3333 } else if bh.ja { 0xFF2A1414 } else { 0xFF1A1A1A };
        framebuffer::ah(bdr as u32, kn, pm, qx, enp);
        
        let iqh = bdr + pm as i32 / 2;
        let iqi = kn as i32 + qx as i32 / 2;
        let jxn = if ndp { 0xFFFFFFFF } else if bh.ja { 0xFFCC4444 } else { 0xFF666666 };
        for a in -3..=3i32 {
            framebuffer::ii((iqh + a) as u32, (iqi + a) as u32, jxn);
            framebuffer::ii((iqh + a) as u32, (iqi - a) as u32, jxn);
            
            framebuffer::ii((iqh + a + 1) as u32, (iqi + a) as u32, jxn);
            framebuffer::ii((iqh + a + 1) as u32, (iqi - a) as u32, jxn);
        }
        
        
        let bvj = bdr - pm as i32;
        let olv = hl >= bvj && hl < bvj + pm as i32 
            && ir >= kn as i32 && ir < kn as i32 + qx as i32;
        let lks = if olv { 0xFF1A3A20 } else { 0xFF0E0E0E };
        framebuffer::ah(bvj as u32, kn, pm, qx, lks);
        let dpn = bvj + pm as i32 / 2;
        let dpo = kn as i32 + qx as i32 / 2;
        let dsy = if olv { 0xFF44DD66 } else if bh.ja { 0xFF227744 } else { 0xFF555555 };
        if bh.bkk {
            
            for a in -2..=1i32 {
                framebuffer::ii((dpn + a + 1) as u32, (dpo - 3) as u32, dsy);
                framebuffer::ii((dpn + 3) as u32, (dpo + a - 1) as u32, dsy);
            }
            for a in -2..=2i32 {
                framebuffer::ii((dpn + a - 1) as u32, (dpo - 1) as u32, dsy);
                framebuffer::ii((dpn + a - 1) as u32, (dpo + 3) as u32, dsy);
                framebuffer::ii((dpn - 3) as u32, (dpo + a + 1) as u32, dsy);
                framebuffer::ii((dpn + 1) as u32, (dpo + a + 1) as u32, dsy);
            }
        } else {
            
            for a in -3..=3i32 {
                framebuffer::ii((dpn + a) as u32, (dpo - 3) as u32, dsy);
                framebuffer::ii((dpn + a) as u32, (dpo + 3) as u32, dsy);
                framebuffer::ii((dpn - 3) as u32, (dpo + a) as u32, dsy);
                framebuffer::ii((dpn + 3) as u32, (dpo + a) as u32, dsy);
            }
        }
        
        
        let cso = bvj - pm as i32;
        let onj = hl >= cso && hl < cso + pm as i32 
            && ir >= kn as i32 && ir < kn as i32 + qx as i32;
        let llv = if onj { 0xFF2A2A10 } else { 0xFF0E0E0E };
        framebuffer::ah(cso as u32, kn, pm, qx, llv);
        let nit = cso + pm as i32 / 2;
        let niv = kn as i32 + qx as i32 / 2;
        let oow = if onj { 0xFFFFBB33 } else if bh.ja { 0xFF886622 } else { 0xFF555555 };
        
        for a in -3..=3i32 {
            framebuffer::ii((nit + a) as u32, niv as u32, oow);
            framebuffer::ii((nit + a) as u32, (niv + 1) as u32, oow);
        }
        
        
        framebuffer::ah(cso as u32, kn, 1, qx, AJ_);
        framebuffer::ah(bvj as u32, kn, 1, qx, AJ_);
        framebuffer::ah(bdr as u32, kn, 1, qx, AJ_);
        
        
        let bel = b + 10;
        let hnp = match bh.ld {
            WindowType::Ay => ">_",
            WindowType::Ak => "[]",
            WindowType::Calculator => "##",
            WindowType::Browser => "WW",
            WindowType::Fp => "/\\",
            WindowType::Ag => "Tx",
            WindowType::Io => "Sk",
            WindowType::Gs => "Kk",
            WindowType::Ih => "C3",
            WindowType::Lw => "Mu",
            _ => "::",
        };
        let xd = if bh.ja { I_ } else { BK_ };
        self.en(bel, c + (dwx as i32 / 2) - 6, hnp, xd);
        
        
        let agx = if bh.ja {
            bei(AC_, 0xFFFFFFFF)
        } else {
            bei(N_, 0xFFCCCCCC)
        };
        let xhx = bh.dq.len() as i32 * 8;
        let xhs = b + (d as i32 / 2) - (xhx / 2);
        let cnf = xhs.am(bel + 24);
        self.en(cnf, c + (dwx as i32 / 2) - 6, &bh.dq, agx);
        
        
        
        
        let gl = c + dwx as i32;
        let nd = i - dwx;
        
        
        if self.asr >= DesktopTier::Bv {
            framebuffer::ah((b + 3) as u32, (gl + 1) as u32, d.ao(6), nd.ao(4), 0xFF080808);
            framebuffer::ih((b + 3) as u32, (gl + 1) as u32, d.ao(6), nd.ao(4), 0x060A06, 210);
        } else {
            framebuffer::ah((b + 3) as u32, (gl + 1) as u32, d.ao(6), nd.ao(4), 0xFF080A08);
        }
        
        
        let bgi = (b + 3).am(0) as u32;
        let yk = (gl + 1).am(0) as u32;
        let axq = d.ao(6);
        let aom = nd.ao(4);
        framebuffer::pir(bgi, yk, axq, aom);
        
        
        self.sgp(bh);
        
        
        framebuffer::nde();
    }
    
    
    fn yni(&self, b: u32, c: u32, aw: u32, s: u32, asy: bool) {
        if asy {
            
            framebuffer::ah(b.ao(1), c.ao(1), aw + 2, aw + 2, 
                (s & 0x00FFFFFF) | 0x40000000);
        }
        framebuffer::ah(b, c, aw, aw, s);
    }
    
    
    fn mzk(&self, rw: u32, tx: u32, ab: f32) -> u32 {
        let aqh = ((rw >> 16) & 0xFF) as f32;
        let cyd = ((rw >> 8) & 0xFF) as f32;
        let of = (rw & 0xFF) as f32;
        let uv = ((tx >> 16) & 0xFF) as f32;
        let cqu = ((tx >> 8) & 0xFF) as f32;
        let tb = (tx & 0xFF) as f32;
        
        let m = (aqh + (uv - aqh) * ab) as u32;
        let at = (cyd + (cqu - cyd) * ab) as u32;
        let o = (of + (tb - of) * ab) as u32;
        
        0xFF000000 | (m << 16) | (at << 8) | o
    }
    
    fn sgp(&self, bh: &Window) {
        let tc = bh.b + 8;
        let gl = bh.c + J_ as i32 + 8;
        
        
        if bh.ld == WindowType::Ag {
            return;
        }
        
        
        if bh.ld == WindowType::Fp {
            return;
        }
        
        
        if bh.ld == WindowType::So {
            return;
        }

        
        #[cfg(feature = "emulators")]
        if bh.ld == WindowType::Sp
            || bh.ld == WindowType::Abx
            || bh.ld == WindowType::Xt
        {
            return;
        }
        
        
        if bh.ld == WindowType::Calculator {
            self.scb(bh);
            return;
        }
        
        
        
        
        if bh.ld == WindowType::Lw {
            self.sef(bh);
            return;
        }
        
        
        
        
        if bh.ld == WindowType::Afs {
            self.sgn(bh);
            return;
        }
        
        
        
        
        if bh.ld == WindowType::Aft {
            self.sgo(bh);
            return;
        }
        
        
        
        
        if bh.ld == WindowType::Ak {
            self.scv(bh);
            return;
        }
        
        
        
        
        if bh.ld == WindowType::Bp {
            self.sdp(bh);
            return;
        }
        
        
        if bh.ld == WindowType::Aqu {
            self.sba(bh);
            return;
        }
        
        
        if bh.ld == WindowType::Io {
            self.sfq(bh);
            return;
        }
        
        
        if bh.ld == WindowType::Gs {
            self.scg(bh);
            return;
        }
        
        
        if bh.ld == WindowType::Ih {
            return;
        }
        
        
        if bh.ld == WindowType::Ro {
            self.kqv(bh);
            return;
        }
        
        
        if bh.ld == WindowType::Td {
            if let Some(g) = self.dso.get(&bh.ad) {
                crate::lab_mode::sds(g, bh.b, bh.c, bh.z, bh.ac);
            }
            return;
        }
        
        
        #[cfg(feature = "emulators")]
        if bh.ld == WindowType::Lm {
            if let Some(ohy) = self.azy.get(&bh.ad) {
                
                let skl = if let Some(fnc) = ohy.fnb {
                    self.arf.get(&fnc)
                } else {
                    
                    self.arf.alv().next()
                };
                crate::game_lab::sdd(ohy, skl, bh.b, bh.c, bh.z, bh.ac);
            }
            return;
        }
        
        
        if bh.ld == WindowType::Browser {
            self.sby(bh);
            return;
        }
        
        
        
        
        if bh.ld == WindowType::Gn {
            self.sfj(bh);
            return;
        }
        
        
        
        
        if bh.ld == WindowType::Hy {
            self.sej(bh);
            return;
        }
        
        
        
        
        if bh.ld == WindowType::Ay {
            let acg = 16i32;
            let ffq = (bh.ac as i32 - J_ as i32 - 16).am(0) as usize;
            let act = if acg as usize > 0 { ffq / acg as usize } else { 0 };
            let bss = bh.ca.len();
            
            
            let jc = bh.px;
            let ay = jc;
            let ci = (ay + act).v(bss);
            
            for w in ay..ci {
                let line = &bh.ca[w];
                let brg = gl + ((w - ay) as i32 * acg);
                if brg >= bh.c + bh.ac as i32 - 8 {
                    break;
                }
                
                
                
                
                if line.contains('\x01') {
                    let mut cx = tc;
                    let mut dpk = B_;
                    let mut bw = line.bw().ltk();
                    while let Some(bm) = bw.next() {
                        if bm == '\x01' {
                            if let Some(&aj) = bw.amm() {
                                bw.next();
                                dpk = match aj {
                                    'R' => DC_,
                                    'G' => I_,
                                    'B' => QV_,
                                    'W' => AC_,
                                    'Y' => FY_,
                                    'M' => X_,
                                    'D' => P_,
                                    'N' => B_,
                                    'H' => 0xFF00FFAA,
                                    'A' => BK_,
                                    'S' => BH_,
                                    _ => dpk,
                                };
                            }
                        } else {
                            crate::framebuffer::afn(cx as u32, brg as u32, bm, dpk);
                            cx += 8;
                        }
                    }
                } else {
                    
                    let ux = line.ifa();
                    if ux.cj("root@trustos") || ux.cj("$") {
                        
                        let mut cx = tc;
                        if let Some(nmm) = line.du('$') {
                            
                            let cvu = &line[..nmm];
                            
                            if let Some(ikc) = cvu.du('@') {
                                
                                for bm in cvu[..ikc].bw() {
                                    crate::framebuffer::afn(cx as u32, brg as u32, bm, I_);
                                    cx += 8;
                                }
                                
                                crate::framebuffer::afn(cx as u32, brg as u32, '@', P_);
                                cx += 8;
                                
                                let iyq = &cvu[ikc + 1..];
                                
                                if let Some(dfa) = iyq.du(':') {
                                    for bm in iyq[..dfa].bw() {
                                        crate::framebuffer::afn(cx as u32, brg as u32, bm, QV_);
                                        cx += 8;
                                    }
                                    crate::framebuffer::afn(cx as u32, brg as u32, ':', P_);
                                    cx += 8;
                                    
                                    for bm in iyq[dfa + 1..].bw() {
                                        crate::framebuffer::afn(cx as u32, brg as u32, bm, FY_);
                                        cx += 8;
                                    }
                                } else {
                                    for bm in iyq.bw() {
                                        crate::framebuffer::afn(cx as u32, brg as u32, bm, QV_);
                                        cx += 8;
                                    }
                                }
                            } else {
                                for bm in cvu.bw() {
                                    crate::framebuffer::afn(cx as u32, brg as u32, bm, AG_);
                                    cx += 8;
                                }
                            }
                            
                            crate::framebuffer::afn(cx as u32, brg as u32, '$', I_);
                            cx += 8;
                            
                            for bm in line[nmm + 1..].bw() {
                                crate::framebuffer::afn(cx as u32, brg as u32, bm, AC_);
                                cx += 8;
                            }
                        } else {
                            self.cb(tc, brg, line, B_);
                        }
                    } else {
                        
                        self.cb(tc, brg, line, B_);
                    }
                }
            }
            
            
            let jny = 6u32;
            let ftr = (bh.b + bh.z as i32 - jny as i32 - 3) as u32;
            let ekc = (bh.c + J_ as i32 + 2) as u32;
            let bdc = bh.ac.ao(J_ + 4);
            
            if bss > act {
                
                framebuffer::ih(ftr, ekc, jny, bdc, 0x0A1A0F, 80);
                
                
                let axd = ((act as u32 * bdc) / bss as u32).am(20);
                let aye = bss.ao(act);
                let bsm = if aye > 0 {
                    ekc + ((jc as u32 * (bdc - axd)) / aye as u32)
                } else {
                    ekc
                };
                
                mf(ftr as i32, bsm as i32, jny, axd, 3, X_);
                
                framebuffer::ih(ftr + 1, bsm + 1, jny - 2, 1, 0x00FF66, 30);
            }
            
            return;
        }
        
        
        let urt = oh!(bh.ld, 
            WindowType::Ak | WindowType::Pj);
        
        
        let (grx, jod) = match bh.ld {
            WindowType::Ak => (5, bh.ca.len().ao(2)),
            WindowType::Pj => (4, bh.ca.len().ao(2)),
            _ => (0, 0),
        };
        
        
        let jc = if bh.ld == WindowType::Is {
            bh.px
        } else {
            0
        };
        
        for (w, line) in bh.ca.iter().cf().chz(jc) {
            let a = w - jc;
            let brg = gl + (a as i32 * 16);
            if brg >= bh.c + bh.ac as i32 - 8 {
                break;
            }
            
            
            let qe = urt 
                && w >= grx 
                && w < jod 
                && (w - grx) == bh.acm;
            
            if qe {
                
                framebuffer::ah(
                    tc as u32 - 4, 
                    brg as u32 - 2, 
                    bh.z - 16, 
                    18, 
                    0xFF003300
                );
                self.cb(tc, brg, line, G_);
            } else {
                self.cb(tc, brg, line, B_);
            }
        }
    }
    
    
    fn nmy(&mut self) {
        
        let kst: Vec<(u32, i32, i32, u32, u32)> = self.ee.iter()
            .hi(|d| d.ld == WindowType::Ag && d.iw && !d.aat)
            .map(|d| (d.ad, d.b, d.c, d.z, d.ac))
            .collect();
        
        for (nr, fx, lw, hk, mg) in kst {
            if let Some(editor) = self.cxh.ds(&nr) {
                let tc = fx;
                let gl = lw + J_ as i32;
                let ur = hk;
                let nd = mg.ao(J_);
                
                
                framebuffer::pir(
                    (tc + 2).am(0) as u32,
                    gl.am(0) as u32,
                    ur.ao(4),
                    nd,
                );
                
                ehm(
                    editor,
                    tc, gl, ur, nd,
                    &|b, c, text, s| {
                        
                        for (a, bm) in text.bw().cf() {
                            let cx = (b + (a as i32 * 8)) as u32;
                            let ae = c as u32;
                            crate::framebuffer::afn(cx, ae, bm, s);
                        }
                    },
                    &|b, c, bm, s| {
                        crate::framebuffer::afn(b as u32, c as u32, bm, s);
                    },
                );
                
                framebuffer::nde();
            }
        }
    }
    
    
    fn nnj(&mut self) {
        
        let kst: Vec<(u32, i32, i32, u32, u32)> = self.ee.iter()
            .hi(|d| d.ld == WindowType::Fp && d.iw && !d.aat)
            .map(|d| (d.ad, d.b, d.c, d.z, d.ac))
            .collect();
        
        for (nr, fx, lw, hk, mg) in kst {
            if let Some(g) = self.djq.ds(&nr) {
                let tc = fx as u32;
                let gl = (lw + J_ as i32) as u32;
                let ur = hk;
                let nd = mg.ao(J_);
                
                if ur < 80 || nd < 80 { continue; }
                
                
                let ahe = ur as usize;
                let asl = nd as usize;
                let mut k = alloc::vec![0u32; ahe * asl];
                
                g.tj(&mut k, ahe, asl);
                
                
                for x in 0..asl {
                    for y in 0..ahe {
                        let s = k[x * ahe + y];
                        let cr = tc + y as u32;
                        let cq = gl + x as u32;
                        framebuffer::ii(cr, cq, s);
                    }
                }
            }
        }
    }
    
    
    fn nmz(&mut self) {
        let tab: Vec<(u32, i32, i32, u32, u32)> = self.ee.iter()
            .hi(|d| d.ld == WindowType::So && d.iw && !d.aat)
            .map(|d| (d.ad, d.b, d.c, d.z, d.ac))
            .collect();
        
        for (nr, fx, lw, hk, mg) in tab {
            if let Some(g) = self.dra.ds(&nr) {
                let tc = fx as u32;
                let gl = (lw + J_ as i32) as u32;
                let ur = hk;
                let nd = mg.ao(J_);
                
                if ur < 80 || nd < 60 { continue; }
                
                let ahe = ur as usize;
                let asl = nd as usize;
                let mut k = alloc::vec![0u32; ahe * asl];
                
                g.tj(&mut k, ahe, asl);
                
                
                for x in 0..asl {
                    for y in 0..ahe {
                        let s = k[x * ahe + y];
                        let cr = tc + y as u32;
                        let cq = gl + x as u32;
                        framebuffer::ii(cr, cq, s);
                    }
                }
            }
        }
    }
    
    
    fn nmv(&mut self) {
        let rae: Vec<(u32, i32, i32, u32, u32)> = self.ee.iter()
            .hi(|d| d.ld == WindowType::Ih && d.iw && !d.aat && !d.egj)
            .map(|d| (d.ad, d.b, d.c, d.z, d.ac))
            .collect();
        
        for (nr, fx, lw, hk, mg) in rae {
            if let Some(g) = self.cwd.ds(&nr) {
                let tc = fx as u32;
                let gl = (lw + J_ as i32) as u32;
                let ur = hk;
                let nd = mg.ao(J_);
                
                if ur < 100 || nd < 100 { continue; }
                
                g.or();
                
                let ahe = ur as usize;
                let asl = nd as usize;
                let mut k = alloc::vec![0u32; ahe * asl];
                
                g.tj(&mut k, ahe, asl);
                
                
                for x in 0..asl {
                    for y in 0..ahe {
                        let s = k[x * ahe + y];
                        let cr = tc + y as u32;
                        let cq = gl + x as u32;
                        framebuffer::ii(cr, cq, s);
                    }
                }
            }
        }
    }
    
    
    #[cfg(feature = "emulators")]
    fn nnk(&mut self) {
        let usb: Vec<(u32, i32, i32, u32, u32)> = self.ee.iter()
            .hi(|d| d.ld == WindowType::Xt && d.iw && !d.aat && !d.egj)
            .map(|d| (d.ad, d.b, d.c, d.z, d.ac))
            .collect();
        
        for (nr, fx, lw, hk, mg) in usb {
            if let Some(cw) = self.dtk.ds(&nr) {
                let tc = fx as u32;
                let gl = (lw + J_ as i32) as u32;
                let ur = hk;
                let nd = mg.ao(J_);
                
                if ur < 80 || nd < 60 { continue; }
                
                let ahe = ur as usize;
                let asl = nd as usize;
                let mut k = alloc::vec![0u32; ahe * asl];
                
                cw.tj(&mut k, ahe, asl);
                
                for x in 0..asl {
                    for y in 0..ahe {
                        let s = k[x * ahe + y];
                        let cr = tc + y as u32;
                        let cq = gl + x as u32;
                        framebuffer::ii(cr, cq, s);
                    }
                }
            }
        }
    }
    
    
    #[cfg(feature = "emulators")]
    fn nna(&mut self) {
        let tah: Vec<(u32, i32, i32, u32, u32, bool)> = self.ee.iter()
            .hi(|d| d.ld == WindowType::Sp && d.iw && !d.aat && !d.egj)
            .map(|d| (d.ad, d.b, d.c, d.z, d.ac, d.ja))
            .collect();
        
        let aje: u32 = 22;
        
        for (nr, fx, lw, hk, mg, xzi) in tah {
            if let Some(cw) = self.arf.ds(&nr) {
                let tc = fx as u32;
                let gl = (lw + J_ as i32) as u32;
                let ur = hk;
                let nd = mg.ao(J_);
                
                if ur < 80 || nd < 60 { continue; }
                
                
                framebuffer::ah(tc, gl, ur, aje, 0xFF0E1418);
                framebuffer::ah(tc, gl + aje - 1, ur, 1, 0xFF1E3028);
                
                
                let vfq = alloc::format!("PC:{:04X}", cw.cpu.fz);
                let uiz = alloc::format!("LY:{:3}", cw.gpu.ct);
                let upb = match cw.gpu.ev {
                    0 => "HBL",
                    1 => "VBL",
                    2 => "OAM",
                    3 => "DRW",
                    _ => "???",
                };
                let qms = alloc::format!("BK:{}", cw.on.bwu);
                
                let mut gx = tc + 4;
                for bm in vfq.bw() { framebuffer::afn(gx, gl + 4, bm, 0xFF58A6FF); gx += 8; }
                gx += 8;
                for bm in uiz.bw() { framebuffer::afn(gx, gl + 4, bm, 0xFF80FFAA); gx += 8; }
                gx += 8;
                for bm in upb.bw() { framebuffer::afn(gx, gl + 4, bm, 0xFFD29922); gx += 8; }
                gx += 8;
                for bm in qms.bw() { framebuffer::afn(gx, gl + 4, bm, 0xFF9CD8B0); gx += 8; }
                
                if cw.atz {
                    gx += 8;
                    let mgn = if cw.beq & 0x80 != 0 { "2x" } else { "1x" };
                    for bm in "CGB".bw() { framebuffer::afn(gx, gl + 4, bm, 0xFF00FF88); gx += 8; }
                    gx += 4;
                    for bm in mgn.bw() { framebuffer::afn(gx, gl + 4, bm, 0xFF79C0FF); gx += 8; }
                }
                
                
                
                let fln: u32 = 48;
                let edh = tc + ur - fln - 4;
                framebuffer::ah(edh, gl + 2, fln, aje - 4, 0xFF1A3028);
                framebuffer::ah(edh, gl + 2, fln, 1, 0xFF2A4A38);
                framebuffer::ah(edh, gl + aje - 3, fln, 1, 0xFF2A4A38);
                let uab = edh + 4;
                for (a, bm) in "INPUT".bw().cf() {
                    framebuffer::afn(uab + a as u32 * 8, gl + 5, bm, 0xFF00FF88);
                }
                
                
                let fml: u32 = 32;
                let fmm = edh - fml - 6;
                framebuffer::ah(fmm, gl + 2, fml, aje - 4, 0xFF1A2838);
                framebuffer::ah(fmm, gl + 2, fml, 1, 0xFF2A3A58);
                framebuffer::ah(fmm, gl + aje - 3, fml, 1, 0xFF2A3A58);
                let uit = fmm + 4;
                for (a, bm) in "LAB".bw().cf() {
                    framebuffer::afn(uit + a as u32 * 8, gl + 5, bm, 0xFF58A6FF);
                }
                
                
                let aui = gl + aje;
                let bum = nd.ao(aje);
                
                if bum < 40 { continue; }
                
                let ahe = ur as usize;
                let asl = bum as usize;
                let mut k = alloc::vec![0u32; ahe * asl];
                
                cw.tj(&mut k, ahe, asl);
                
                for x in 0..asl {
                    for y in 0..ahe {
                        let s = k[x * ahe + y];
                        let cr = tc + y as u32;
                        let cq = aui + x as u32;
                        framebuffer::ii(cr, cq, s);
                    }
                }
            }
        }
        
        
        let tvd: Vec<(u32, i32, i32, u32, u32, Option<u32>)> = self.ee.iter()
            .hi(|d| d.ld == WindowType::Abx && d.iw && !d.aat && !d.egj)
            .map(|d| {
                let ufg = self.ghu.get(&d.ad).hu();
                (d.ad, d.b, d.c, d.z, d.ac, ufg)
            })
            .collect();
        
        for (ydx, fx, lw, hk, mg, fnc) in tvd {
            let cx = fx as u32;
            let ae = (lw + J_ as i32) as u32;
            let dt = hk;
            let bm = mg.ao(J_);
            
            if dt < 60 || bm < 40 { continue; }
            
            
            let skk = if let Some(czg) = fnc {
                self.arf.get(&czg)
            } else {
                self.arf.alv().next()
            };
            
            crate::game_lab::sdq(skk, cx, ae, dt, bm);
        }
    }
    
    
    fn sba(&self, bh: &Window) {
        use crate::graphics::opengl::*;
        use crate::graphics::texture;
        
        let cww = bh.b as u32 + 10;
        let dgd = bh.c as u32 + J_ + 10;
        let bqe = bh.z.ao(20);
        let cea = bh.ac.ao(J_ + 20);
        
        if bqe < 80 || cea < 80 {
            return;
        }
        
        
        nyy(bqe, cea);
        tfw(cww as i32, dgd as i32, bqe, cea);
        
        
        nyx(0.04, 0.06, 0.04, 1.0);
        nyw(ACW_ | ACX_);
        
        
        kzd(TL_);
        
        
        let dyk = bqe as f32 / cea as f32;
        ixa(NF_);
        hlp();
        tgf(45.0, dyk, 0.1, 100.0);
        
        
        ixa(ACY_);
        hlp();
        nzf(
            3.0, 2.0, 4.0,   
            0.0, 0.0, 0.0,   
            0.0, 1.0, 0.0    
        );
        
        
        let hg = (self.oo as f32 * 0.5) % 360.0;
        ixb(hg, 0.0, 1.0, 0.0);
        ixb(hg * 0.3, 1.0, 0.0, 0.0);
        
        
        let e = 0.8;
        
        cfa(KG_);
        
        
        drf(1.0, 0.2, 0.2);
        bnc(0.0, 0.0, 1.0);
        jx(-e, -e, e);
        jx(e, -e, e);
        jx(e, e, e);
        jx(-e, e, e);
        
        
        drf(0.2, 1.0, 0.2);
        bnc(0.0, 0.0, -1.0);
        jx(e, -e, -e);
        jx(-e, -e, -e);
        jx(-e, e, -e);
        jx(e, e, -e);
        
        
        drf(0.2, 0.2, 1.0);
        bnc(0.0, 1.0, 0.0);
        jx(-e, e, e);
        jx(e, e, e);
        jx(e, e, -e);
        jx(-e, e, -e);
        
        
        drf(1.0, 1.0, 0.2);
        bnc(0.0, -1.0, 0.0);
        jx(-e, -e, -e);
        jx(e, -e, -e);
        jx(e, -e, e);
        jx(-e, -e, e);
        
        
        drf(1.0, 0.2, 1.0);
        bnc(1.0, 0.0, 0.0);
        jx(e, -e, e);
        jx(e, -e, -e);
        jx(e, e, -e);
        jx(e, e, e);
        
        
        drf(0.2, 1.0, 1.0);
        bnc(-1.0, 0.0, 0.0);
        jx(-e, -e, -e);
        jx(-e, -e, e);
        jx(-e, e, e);
        jx(-e, e, -e);
        
        cfb();
        
        
        nza();
        tfv(2.5, 0.0, 0.0); 
        ixb(hg * 0.7, 0.3, 1.0, 0.2);
        
        
        static mut APY_: u32 = 0;
        static mut BGZ_: bool = false;
        unsafe {
            if !BGZ_ {
                rvm(&mut APY_);
                BGZ_ = true;
            }
            rvn(0.0, APY_);
        }
        nyz();
        
        
        hlp();
        nzf(3.0, 2.0, 4.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        
        cfa(TO_);
        
        drf(1.0, 0.0, 0.0);
        jx(0.0, 0.0, 0.0);
        jx(2.0, 0.0, 0.0);
        
        drf(0.0, 1.0, 0.0);
        jx(0.0, 0.0, 0.0);
        jx(0.0, 2.0, 0.0);
        
        drf(0.0, 0.0, 1.0);
        jx(0.0, 0.0, 0.0);
        jx(0.0, 0.0, 2.0);
        cfb();
        
        
        self.cb(cww as i32 + 8, dgd as i32 + 8, "TrustGL OpenGL Demo", AG_);
        self.cb(cww as i32 + 8, dgd as i32 + 24, "Software 3D + Textures", BK_);
        
        
        let dwf = dgd as i32 + cea as i32 - 24;
        self.cb(cww as i32 + 8, dwf, "Left: Color Cube | Right: Textured Cube", X_);
        self.cb(cww as i32 + 8, dwf, "Vertices: 8 | Edges: 12 | Faces: 6", X_);
    }
    
    
    fn yne(&self, fy: i32, fo: i32, dn: i32, dp: i32, s: u32) {
        let dx = (dn - fy).gp();
        let bg = -(dp - fo).gp();
        let cr = if fy < dn { 1 } else { -1 };
        let cq = if fo < dp { 1 } else { -1 };
        let mut rq = dx + bg;
        let mut b = fy;
        let mut c = fo;
        
        loop {
            if b >= 0 && c >= 0 && (b as u32) < self.z && (c as u32) < self.ac {
                framebuffer::ii(b as u32, c as u32, s);
            }
            if b == dn && c == dp { break; }
            let agl = 2 * rq;
            if agl >= bg {
                rq += bg;
                b += cr;
            }
            if agl <= dx {
                rq += dx;
                c += cq;
            }
        }
    }
    
    
    fn sfq(&self, bh: &Window) {
        let avx = bh.b as u32 + 10;
        let aui = bh.c as u32 + J_ + 10;
        let bqu = bh.z.ao(20);
        let bum = bh.ac.ao(J_ + 20);
        
        if bqu < 80 || bum < 80 {
            return;
        }
        
        
        framebuffer::ah(avx, aui, bqu, bum, 0xFF0A0E0B);
        
        
        for a in 0..bqu {
            framebuffer::ii(avx + a, aui, X_);
            framebuffer::ii(avx + a, aui + bum - 1, X_);
        }
        for a in 0..bum {
            framebuffer::ii(avx, aui + a, X_);
            framebuffer::ii(avx + bqu - 1, aui + a, X_);
        }
        
        
        if let Some(atl) = self.eyq.get(&bh.ad) {
            let ny: u32 = 14;
            let lam = avx + 10;
            let lan = aui + 36;
            
            
            for ub in 0..atl.bhc {
                for qz in 0..atl.auk {
                    let y = lam + qz as u32 * ny;
                    let x = lan + ub as u32 * ny;
                    if y + ny < avx + bqu && x + ny < aui + bum {
                        let ei = if (qz + ub) % 2 == 0 { 0xFF0D120E } else { 0xFF0B100C };
                        framebuffer::ah(y, x, ny, ny, ei);
                    }
                }
            }
            
            
            for (a, &(cr, cq)) in atl.atl.iter().cf() {
                let y = lam + cr as u32 * ny;
                let x = lan + cq as u32 * ny;
                let s = if a == 0 { 
                    0xFF00FF00 
                } else {
                    let yx = (0xCC - (a as u32 * 8).v(0x80)) as u32;
                    0xFF000000 | (yx << 8) 
                };
                
                if y + ny < avx + bqu && x + ny < aui + bum {
                    framebuffer::ah(y + 1, x + 1, ny - 2, ny - 2, s);
                    
                    if a == 0 {
                        let (snx, sqm, sny, sqn) = match atl.sz {
                            (1, 0) => (ny-4, 3, ny-4, ny-5), 
                            (-1, 0) => (2, 3, 2, ny-5),                     
                            (0, -1) => (3, 2, ny-5, 2),                     
                            _ => (3, ny-4, ny-5, ny-4),       
                        };
                        framebuffer::ii(y + snx, x + sqm, 0xFF000000);
                        framebuffer::ii(y + sny, x + sqn, 0xFF000000);
                    }
                }
            }
            
            
            let jf = lam + atl.ghh.0 as u32 * ny;
            let sc = lan + atl.ghh.1 as u32 * ny;
            if jf + ny < avx + bqu && sc + ny < aui + bum {
                framebuffer::ah(jf + 2, sc + 2, ny - 4, ny - 4, 0xFFFF4444);
                framebuffer::ii(jf + ny/2, sc + 1, 0xFF00AA00); 
            }
            
            
            self.cb(avx as i32 + 8, aui as i32 + 8, "SNAKE", G_);
            
            
            let hzb = if atl.crf > 0 {
                format!("Score: {}  Best: {}", atl.ol, atl.crf)
            } else {
                format!("Score: {}", atl.ol)
            };
            self.cb(avx as i32 + bqu as i32 - 170, aui as i32 + 8, &hzb, AG_);
            
            if atl.cev {
                
                let mp = avx + bqu / 2 - 60;
                let qw = aui + bum / 2 - 20;
                framebuffer::ah(mp - 4, qw - 4, 128, 58, 0xCC000000);
                self.cb(mp as i32, qw as i32, "GAME OVER", 0xFFFF4444);
                let ssu = format!("Score: {}", atl.ol);
                self.cb(mp as i32 + 4, qw as i32 + 18, &ssu, AG_);
                self.cb(mp as i32 - 8, qw as i32 + 36, "Press ENTER", BK_);
            } else if atl.ant {
                
                let mp = avx + bqu / 2 - 50;
                let qw = aui + bum / 2 - 20;
                framebuffer::ah(mp - 4, qw - 4, 110, 48, 0xCC000000);
                self.cb(mp as i32 + 8, qw as i32, "PAUSED", 0xFFFFCC00);
                self.cb(mp as i32 - 4, qw as i32 + 20, "P to resume", BK_);
            } else {
                
                self.cb(avx as i32 + 8, aui as i32 + bum as i32 - 18, 
                               "Arrows to move | P pause", BK_);
            }
        }
    }
    
    
    fn nmw(y: u32, x: u32, xe: i8) {
        let mtf = if xe < 0 { -xe } else { xe };
        let aun = xe > 0;

        let vi = if aun { 0xFFE8E0D0_u32 } else { 0xFF2A2A2A_u32 };
        let oth = if aun { 0xFF1A1A1A_u32 } else { 0xFF888888_u32 };

        
        
        let ek: &[(u32, u32, u32, u32)] = match mtf {
            1 => &[ 
                (20, 12, 8, 7),   
                (22, 19, 4, 3),   
                (19, 22, 10, 3),  
                (16, 25, 16, 3),  
                (14, 28, 20, 3),  
                (12, 31, 24, 4),  
            ],
            2 => &[ 
                (21, 8, 6, 3),    
                (17, 11, 14, 4),  
                (13, 15, 14, 3),  
                (13, 18, 8, 2),   
                (19, 17, 10, 5),  
                (21, 22, 8, 5),   
                (16, 27, 16, 3),  
                (13, 30, 22, 3),  
                (11, 33, 26, 3),  
            ],
            3 => &[ 
                (23, 6, 2, 3),    
                (21, 9, 6, 4),    
                (19, 13, 10, 4),  
                (21, 17, 6, 5),   
                (18, 22, 12, 4),  
                (15, 26, 18, 3),  
                (13, 29, 22, 3),  
                (11, 32, 26, 4),  
            ],
            4 => &[ 
                (15, 7, 4, 4),    
                (22, 7, 4, 4),    
                (29, 7, 4, 4),    
                (15, 11, 18, 3),  
                (17, 14, 14, 12), 
                (15, 26, 18, 3),  
                (13, 29, 22, 3),  
                (11, 32, 26, 4),  
            ],
            5 => &[ 
                (23, 4, 2, 3),    
                (17, 7, 2, 3),    
                (23, 6, 2, 3),    
                (29, 7, 2, 3),    
                (16, 10, 16, 4),  
                (20, 14, 8, 4),   
                (17, 18, 14, 6),  
                (15, 24, 18, 3),  
                (13, 27, 22, 3),  
                (11, 30, 26, 4),  
            ],
            6 => &[ 
                (23, 4, 2, 6),    
                (20, 6, 8, 2),    
                (18, 10, 12, 4),  
                (20, 14, 8, 3),   
                (17, 17, 14, 7),  
                (15, 24, 18, 3),  
                (13, 27, 22, 3),  
                (11, 30, 26, 4),  
            ],
            _ => return,
        };

        
        for &(b, c, d, i) in ek {
            framebuffer::ah(y + b - 1, x + c - 1, d + 2, i + 2, oth);
        }
        
        for &(b, c, d, i) in ek {
            framebuffer::ah(y + b, x + c, d, i, vi);
        }
        
        let abe = if aun { 0x66FFFFFF_u32 } else { 0x44FFFFFF_u32 };
        for &(b, c, d, i) in ek {
            if d > 4 && i > 2 {
                framebuffer::ah(y + b + 1, x + c + 1, 1, i - 2, abe);
            }
        }

        
        if mtf == 3 {
            let plj = oth;
            framebuffer::ah(y + 22, x + 14, 4, 1, plj);
            framebuffer::ah(y + 21, x + 15, 4, 1, plj);
        }
    }

    
    fn scg(&self, bh: &Window) {
        let avx = bh.b as u32 + 8;
        let aui = bh.c as u32 + J_ + 4;
        let bqu = bh.z.ao(16);
        let bum = bh.ac.ao(J_ + 8);
        
        if bqu < 200 || bum < 200 {
            return;
        }
        
        
        framebuffer::ah(avx, aui, bqu, bum, 0xFF0A0E0B);
        
        if let Some(chess) = self.dou.get(&bh.ad) {
            
            let ny: u32 = 48;
            let aly = ny * 8;
            let aoj = avx + (bqu.ao(aly)) / 2;
            let apl = aui + 28;
            
            
            self.cb(avx as i32 + 8, aui as i32 + 6, "TRUSTCHESS", I_);
            
            
            let ol = chess.oli();
            let wel = if ol > 0 {
                format!("+{}", ol / 100)
            } else if ol < 0 {
                format!("{}", ol / 100)
            } else {
                String::from("=")
            };
            let mcl = if ol > 0 { 0xFFFFFFFF } else if ol < 0 { 0xFFCC4444 } else { X_ };
            
            self.cb(avx as i32 + 96, aui as i32 + 6, &wel, mcl);
            
            
            let rxj = match chess.fzz { 1 => "Easy", 2 => "Med", _ => "Hard" };
            self.cb(avx as i32 + 130, aui as i32 + 6, rxj, X_);
            
            
            if chess.ezv {
                let qse = crate::chess::ChessState::ivj(chess.fdl);
                let xvy = crate::chess::ChessState::ivj(chess.fyr);
                
                let xhg = if !chess.axi && chess.ezw { 0xFFCC4444 } else { X_ };
                self.cb(aoj as i32 + aly as i32 + 8, apl as i32 + 4, &qse, xhg);
                crate::framebuffer::afn(aoj + aly + 8, apl + 14, 'B', 0xFFCC4444);
                
                let xhh = if chess.axi && chess.ezw { 0xFFFFFFFF } else { X_ };
                self.cb(aoj as i32 + aly as i32 + 8, apl as i32 + aly as i32 - 20, &xvy, xhh);
                crate::framebuffer::afn(aoj + aly + 8, apl + aly - 10, 'W', 0xFFFFFFFF);
            }
            
            
            for br in 0..8u32 {
                for bj in 0..8u32 {
                    let im = (br * 8 + bj) as usize;
                    let y = aoj + bj * ny;
                    let x = apl + br * ny;
                    
                    
                    let dio = (br + bj) % 2 == 0;
                    let mut ei = if dio { 0xFF3D5A3D } else { 0xFF1A2E1A };
                    
                    
                    if chess.na == Some(im) {
                        ei = 0xFF5A7A2A; 
                    }
                    
                    
                    if chess.blr.contains(&im) {
                        ei = if dio { 0xFF4A8A4A } else { 0xFF2A6A2A };
                    }
                    
                    
                    if chess.jcn == Some(im) || chess.jco == Some(im) {
                        ei = if dio { 0xFF5A6A3A } else { 0xFF3A4A2A };
                    }
                    
                    
                    if chess.gi == im {
                        ei = 0xFF00AA44; 
                    }
                    
                    framebuffer::ah(y, x, ny, ny, ei);
                    
                    
                    let xe = chess.mn[im];
                    let lfx = chess.dgo == Some(im) && chess.epb.is_some();
                    if xe != 0 && !lfx {
                        Self::nmw(y, x, xe);
                    }
                    
                    
                    if chess.blr.contains(&im) && (xe == 0 || lfx) {
                        let hgo = y + ny / 2 - 3;
                        let kqo = x + ny / 2 - 3;
                        framebuffer::ah(hgo, kqo, 6, 6, 0xFF00FF66);
                    }
                    
                    
                    if chess.blr.contains(&im) && xe != 0 && !lfx {
                        
                        for dx in 0..4u32 {
                            framebuffer::ii(y + dx, x, 0xFF00FF66);
                            framebuffer::ii(y, x + dx, 0xFF00FF66);
                            framebuffer::ii(y + ny - 1 - dx, x, 0xFF00FF66);
                            framebuffer::ii(y + ny - 1, x + dx, 0xFF00FF66);
                            framebuffer::ii(y + dx, x + ny - 1, 0xFF00FF66);
                            framebuffer::ii(y, x + ny - 1 - dx, 0xFF00FF66);
                            framebuffer::ii(y + ny - 1 - dx, x + ny - 1, 0xFF00FF66);
                            framebuffer::ii(y + ny - 1, x + ny - 1 - dx, 0xFF00FF66);
                        }
                    }
                }
            }
            
            
            if let (Some(msh), Some(sar)) = (chess.dgo, chess.epb) {
                let dx = chess.kqq;
                let bg = chess.kqr;
                if dx > 24 && bg > 24 {
                    Self::nmw(dx as u32 - 24, bg as u32 - 24, sar);
                }
            }
            
            
            for a in 0..aly {
                framebuffer::ii(aoj + a, apl, X_);
                framebuffer::ii(aoj + a, apl + aly, X_);
            }
            for a in 0..aly + 1 {
                framebuffer::ii(aoj, apl + a, X_);
                framebuffer::ii(aoj + aly, apl + a, X_);
            }
            
            
            for r in 0..8u32 {
                let cu = (b'a' + r as u8) as char;
                crate::framebuffer::afn(aoj + r * ny + ny / 2 - 4, apl + aly + 4, cu, BK_);
            }
            
            for m in 0..8u32 {
                let cu = (b'8' - m as u8) as char;
                crate::framebuffer::afn(aoj - 14, apl + m * ny + ny / 2 - 6, cu, BK_);
            }
            
            
            let pl = apl + aly + 18;
            let lo = aly;
            let tn = 6u32;
            framebuffer::ah(aoj, pl, lo, tn, 0xFF1A1A1A);
            
            let eus = 2000i32; 
            let feu = ol.qp(-eus, eus);
            let pn = aoj + lo / 2;
            if feu > 0 {
                let akd = ((feu as u32) * (lo / 2)) / eus as u32;
                framebuffer::ah(pn, pl, akd.v(lo / 2), tn, 0xFFFFFFFF);
            } else if feu < 0 {
                let akd = (((-feu) as u32) * (lo / 2)) / eus as u32;
                let akd = akd.v(lo / 2);
                framebuffer::ah(pn - akd, pl, akd, tn, 0xFFCC4444);
            }
            
            framebuffer::ah(pn, pl, 1, tn, X_);
            
            
            let uo = pl + tn + 6;
            let uqe = match chess.ib {
                crate::chess::GamePhase::Aam => DC_,
                crate::chess::GamePhase::Mw => 0xFFFF4444,
                crate::chess::GamePhase::Up => FY_,
                crate::chess::GamePhase::Yg => QV_,
                _ => I_,
            };
            self.cb(aoj as i32, uo as i32, &chess.message, uqe);
            
            
            let gvd = if chess.axi { "White" } else { "Black" };
            let ifg = if chess.axi { 0xFFFFFFFF } else { 0xFFCC4444 };
            self.cb(aoj as i32 + aly as i32 - 60, uo as i32, gvd, ifg);
            
            
            let tox = uo as u32 + 18;
            let obv = if chess.gnd.len() > 6 { chess.gnd.len() - 6 } else { 0 };
            let mut bng = aoj as i32;
            for (a, ef) in chess.gnd[obv..].iter().cf() {
                let num = obv + a + 1;
                let bt = format!("{}. {} ", num, ef);
                self.cb(bng, tox as i32, &bt, X_);
                bng += bt.len() as i32 * 8 + 4;
                if bng > aoj as i32 + aly as i32 - 40 {
                    break; 
                }
            }
            
            
            let iyj = aui + bum - 30;
            self.cb(avx as i32 + 4, iyj as i32,
                           "Mouse:Click/Drag  Arrows:Move  Enter:Select", BK_);
            self.cb(avx as i32 + 4, iyj as i32 + 12,
                           "Esc:Desel  R:Reset  T:Timer  D:Difficulty", BK_);
        }
    }
    
    
    fn kqv(&self, bh: &Window) {
        if let Some(g) = self.fdi.get(&bh.ad) {
            let cb = |b: i32, c: i32, text: &str, s: u32| {
                self.cb(b, c, text, s);
            };
            crate::apps::binary_viewer::kqv(
                g,
                bh.b, bh.c,
                bh.z, bh.ac,
                &cb,
            );
        }
    }

    
    pub fn uyv(&mut self, path: &str) -> Result<u32, &'static str> {
        let ln = crate::binary_analysis::mvr(path)?;
        let g = crate::apps::binary_viewer::BinaryViewerState::new(ln, path);
        
        let mld = alloc::format!("TrustView — {}", path);
        
        let ad = self.xl(&mld, 50, 50, 1100, 650, WindowType::Ro);
        self.fdi.insert(ad, g);
        Ok(ad)
    }

    
    pub fn osq(&mut self) -> u32 {
        let ad = self.xl("TrustLab \u{2014} OS Introspection", 30, 30, 1200, 700, WindowType::Td);
        
        let kp = crate::framebuffer::z() as u32;
        let kl = crate::framebuffer::ac() as u32;
        if let Some(d) = self.ee.el().du(|d| d.ad == ad) {
            d.exy = d.b;
            d.exz = d.c;
            d.gri = d.z;
            d.grh = d.ac;
            d.b = 0;
            d.c = 0;
            d.z = kp;
            d.ac = kl - W_;
            d.bkk = true;
        }
        self.dhh(ad);
        ad
    }

    
    
    
    
    fn sdp(&self, bh: &Window) {
        let fx = bh.b;
        let lw = bh.c;
        let hk = bh.z;
        let mg = bh.ac;
        if hk < 60 || mg < 80 { return; }
        
        let gl = lw + J_ as i32;
        let nd = mg.ao(J_ + 28); 
        let zb = if fx < 0 { 0u32 } else { fx as u32 };
        let wci = gl as u32;
        
        
        framebuffer::ah(zb + 2, wci, hk.ao(4), nd, 0xFF080808);
        
        if let Some(g) = self.gjo.get(&bh.ad) {
            if g.esh > 0 && g.flc > 0 && !g.hz.is_empty() {
                
                let qbc = g.ddn as u32;
                let dgi = (g.esh * qbc) / 100;
                let eaw = (g.flc * qbc) / 100;
                
                
                let dtw = (hk as i32 - dgi as i32) / 2 + g.hud;
                let dtx = (nd as i32 - eaw as i32) / 2 + g.hue;
                
                
                let wf = framebuffer::z();
                let aav = framebuffer::ac();
                
                for bg in 0..eaw {
                    let abi = gl + dtx + bg as i32;
                    if abi < gl || abi >= gl + nd as i32 { continue; }
                    if abi < 0 || abi >= aav as i32 { continue; }
                    
                    
                    let bih = (bg * g.flc) / eaw.am(1);
                    if bih >= g.flc { continue; }
                    
                    for dx in 0..dgi {
                        let xu = fx + dtw + dx as i32;
                        if xu < fx + 2 || xu >= fx + hk as i32 - 2 { continue; }
                        if xu < 0 || xu >= wf as i32 { continue; }
                        
                        let blg = (dx * g.esh) / dgi.am(1);
                        if blg >= g.esh { continue; }
                        
                        let il = g.hz[(bih * g.esh + blg) as usize];
                        
                        if (il >> 24) == 0 { continue; }
                        framebuffer::ii(xu as u32, abi as u32, il | 0xFF000000);
                    }
                }
                
                
                let uo = (gl + nd as i32) as u32;
                framebuffer::ah(zb + 2, uo, hk.ao(4), 24, 0xFF0A1A12);
                framebuffer::zs(zb + 2, uo, hk.ao(4), 0xFF1A2A1A);
                
                let co = alloc::format!("{}x{} | Zoom: {}% | +/- to zoom | Arrows to pan", 
                    g.esh, g.flc, g.ddn);
                self.en(fx + 10, uo as i32 + 5, &co, BH_);
            } else {
                
                self.en(fx + hk as i32 / 2 - 60, gl + nd as i32 / 2, "No image loaded", P_);
                self.en(fx + hk as i32 / 2 - 80, gl + nd as i32 / 2 + 20, "Open a .bmp file to view it", P_);
            }
        } else {
            self.en(fx + 20, gl + 30, "Image Viewer — open a file", P_);
        }
    }
    
    fn tjv(&mut self, bs: u8) {
        let nr = match self.ee.iter().du(|d| d.ja && d.ld == WindowType::Bp) {
            Some(d) => d.ad,
            None => return,
        };
        if let Some(g) = self.gjo.ds(&nr) {
            match bs {
                b'+' | b'=' => { g.ddn = (g.ddn + 10).v(500); }
                b'-' => { g.ddn = g.ddn.ao(10).am(10); }
                b'0' => { g.ddn = 100; g.hud = 0; g.hue = 0; } 
                _ => {
                    if bs == crate::keyboard::V_ { g.hue += 20; }
                    else if bs == crate::keyboard::U_ { g.hue -= 20; }
                    else if bs == crate::keyboard::AH_ { g.hud += 20; }
                    else if bs == crate::keyboard::AI_ { g.hud -= 20; }
                }
            }
        }
    }

    
    
    
    
    fn scw(&self, bh: &Window) {
        let fx = bh.b;
        let lw = bh.c;
        let hk = bh.z;
        let mg = bh.ac;
        if hk < 80 || mg < 100 { return; }
        
        let ipe = lw + J_ as i32;
        let zb = if fx < 0 { 0u32 } else { fx as u32 };
        
        
        let aiq = self.avu.get(&bh.ad).map(|bb| if bb.ian { 0u32 } else { bb.iao }).unwrap_or(180);
        let bqw = fx + aiq as i32;
        let auk = hk.ao(aiq);
        
        
        let qpd = 0xFF0A120Cu32;
        let ham = 0xFF0C140Cu32;
        let gba = 0xFF081008u32;
        let hak = 0xFF0A3818u32;
        let jsz = 0xFF80CC90u32;
        let fkx = 0xFFDDAA30u32;
        let lda = 0xFF60AA80u32;
        let hzr = 0xFF142014u32;
        
        
        let bpb = 36u32;
        framebuffer::ah(zb, ipe as u32, hk, bpb, ham);
        
        let kn = ipe + 7;
        let acv = 22u32;
        
        let imw = self.avu.get(&bh.ad).map(|bb| bb.nbo()).unwrap_or(false);
        let qmg = if imw { AG_ } else { 0xFF1A2A1A };
        mf(fx + 8, kn, acv, acv, 4, 0xFF101810);
        self.cb(fx + 14, kn + 4, "<", qmg);
        
        let imx = self.avu.get(&bh.ad).map(|bb| bb.nbp()).unwrap_or(false);
        let szp = if imx { AG_ } else { 0xFF1A2A1A };
        mf(fx + 34, kn, acv, acv, 4, 0xFF101810);
        self.cb(fx + 40, kn + 4, ">", szp);
        
        mf(fx + 60, kn, acv, acv, 4, 0xFF101810);
        tf(fx + 60, kn, acv, acv, 4, P_);
        self.cb(fx + 66, kn + 4, "^", BH_);
        
        
        let ewg = fx + 90;
        let fql = (hk as i32).ao(106);
        if fql > 10 {
            mf(ewg, kn, fql as u32, acv, 6, 0xFF080E08);
            tf(ewg, kn, fql as u32, acv, 6, hzr);
            let rp = bh.wn.ahz().unwrap_or("/");
            self.en(ewg + 10, kn + 5, rp, I_);
        }
        
        framebuffer::zs(zb, (ipe + bpb as i32) as u32, hk, hzr);
        
        
        let asj = ipe + bpb as i32 + 1;
        let byl = mg.ao(J_ + bpb + 1 + 26);
        
        if aiq > 0 && byl > 20 {
            framebuffer::ah(zb, asj as u32, aiq, byl, gba);
            let mut cq = asj + 8;
            let ali = 24i32;
            let cr = fx + 6;
            let plc = aiq.ao(12);
            
            self.en(cr + 4, cq, "Quick Access", 0xFF3A7A4A);
            cq += 20;
            if let Some(xa) = self.avu.get(&bh.ad) {
                for (j, path) in xa.jkw.iter() {
                    if cq + ali > asj + byl as i32 - 40 { break; }
                    let afb = bh.wn.ahz() == Some(path.as_str());
                    if afb {
                        mf(cr, cq - 2, plc, ali as u32, 4, 0xFF0C2810);
                        framebuffer::ah(zb + 2, cq as u32, 3, (ali - 4) as u32, I_);
                    }
                    let drv = (cr + 12) as u32;
                    let drw = (cq + 2) as u32;
                    framebuffer::ah(drv, drw, 6, 2, fkx);
                    framebuffer::ah(drv, drw + 2, 12, 8, fkx);
                    let r = if afb { I_ } else { 0xFF50AA60 };
                    self.en(cr + 30, cq + 3, j, r);
                    cq += ali;
                }
            }
            cq += 6;
            framebuffer::zs(zb + 10, cq as u32, aiq.ao(20), hzr);
            cq += 10;
            self.en(cr + 4, cq, "This PC", 0xFF3A7A4A);
            cq += 20;
            let bzh = [("Local Disk (C:)", "/"), ("RAM Disk", "/tmp"), ("Devices", "/dev"), ("System", "/proc")];
            for (j, path) in &bzh {
                if cq + ali > asj + byl as i32 - 4 { break; }
                let afb = bh.wn.ahz() == Some(*path);
                if afb {
                    mf(cr, cq - 2, plc, ali as u32, 4, 0xFF0C2810);
                    framebuffer::ah(zb + 2, cq as u32, 3, (ali - 4) as u32, I_);
                }
                let r = if afb { I_ } else { 0xFF50AA60 };
                self.en(cr + 30, cq + 3, j, r);
                cq += ali;
            }
            framebuffer::ah(zb + aiq - 1, asj as u32, 1, byl, hzr);
        }
        
        
        let drk = asj as u32;
        let ero = byl.ao(2);
        if ero < 8 { return; }
        framebuffer::ah(bqw.am(0) as u32, drk, auk, ero, qpd);
        
        
        let cyq = 90u32;
        let esd = 80u32;
        let ec = ((auk.ao(20)) / cyq).am(1);
        let lrt = (auk.ao(ec * cyq)) / 2;
        
        
        let asu = 5usize.v(bh.ca.len());
        let fin = if bh.ca.len() > asu + 2 { bh.ca.len() - 2 } else { bh.ca.len() };
        let cql: Vec<&str> = if fin > asu {
            bh.ca[asu..fin].iter().map(|e| e.as_str()).collect()
        } else { Vec::new() };
        
        if cql.is_empty() {
            self.en(bqw + 40, drk as i32 + 30, "This folder is empty.", P_);
        }
        
        let umc = (ero / esd).am(1) as usize;
        let dbt = bh.px / ec as usize;
        
        for (w, bt) in cql.iter().cf() {
            let br = w / ec as usize;
            let bj = w % ec as usize;
            
            
            if br < dbt { continue; }
            let iri = br - dbt;
            if iri >= umc { break; }
            
            let dzm = bqw.am(0) as u32 + lrt + bj as u32 * cyq;
            let bmg = drk + iri as u32 * esd;
            if bmg + esd > drk + ero { break; }
            
            let qe = w == bh.acm;
            let ta = bt.contains("[D]");
            
            
            if qe {
                mf(dzm as i32 + 4, bmg as i32 + 2, cyq - 8, esd - 4, 6, hak);
                tf(dzm as i32 + 4, bmg as i32 + 2, cyq - 8, esd - 4, 6, 0xFF1A5A2A);
            }
            
            
            let bel = dzm + (cyq - 32) / 2;
            let bem = bmg + 6;
            if ta {
                
                let gc = if qe { 0xFFEEBB40 } else { fkx };
                framebuffer::ah(bel, bem, 16, 6, gc);
                framebuffer::ah(bel, bem + 6, 32, 20, gc);
                framebuffer::ah(bel + 2, bem + 10, 28, 14, 0xFF0A0A04);
                framebuffer::ah(bel + 6, bem + 14, 16, 2, 0xFF302A10);
                framebuffer::ah(bel + 6, bem + 18, 12, 2, 0xFF302A10);
            } else {
                
                let wm = Self::cxp(bt);
                let (gc, kbw, nsb) = if wm.pp(".rs") || wm.pp(".c") || wm.pp(".h") {
                    (if qe { 0xFFFFAA66 } else { 0xFFDD7733 }, 0xFFFF6633, "RS")
                } else if wm.pp(".txt") || wm.pp(".md") || wm.pp(".log") {
                    (if qe { 0xFF88BBEE } else { 0xFF4488CC }, 0xFF4488CC, if wm.pp(".md") { "MD" } else { "TXT" })
                } else if wm.pp(".toml") || wm.pp(".json") || wm.pp(".cfg") {
                    (if qe { 0xFFEEDD66 } else { 0xFFDDAA00 }, 0xFFDDAA00, "CFG")
                } else if wm.pp(".bmp") || wm.pp(".png") || wm.pp(".jpg") {
                    (if qe { 0xFF66DD88 } else { 0xFF33BB66 }, 0xFF33BB66, "IMG")
                } else if wm.pp(".wav") || wm.pp(".mp3") {
                    (if qe { 0xFFFF88CC } else { 0xFFEE55AA }, 0xFFEE55AA, "SND")
                } else if wm.pp(".sh") || wm.pp(".elf") {
                    (if qe { 0xFFCC88FF } else { 0xFF9966DD }, 0xFF9966DD, "EXE")
                } else {
                    (if qe { 0xFF80DD99 } else { lda }, 0xFF60AA80, "")
                };
                framebuffer::ah(bel, bem, 28, 28, gc);
                framebuffer::ah(bel + 18, bem, 10, 10, 0xFF0A140A);
                framebuffer::ah(bel + 18, bem, 2, 10, gc);
                framebuffer::ah(bel + 18, bem + 8, 10, 2, gc);
                framebuffer::ah(bel + 3, bem + 12, 22, 14, 0xFF040A04);
                
                framebuffer::ah(bel, bem, 3, 28, kbw);
                
                if !nsb.is_empty() {
                    self.cb((bel + 5) as i32, (bem + 15) as i32, nsb, 0xFF203020);
                }
            }
            
            
            let j = Self::cxp(bt);
            let aem = (cyq / 8).v(10) as usize;
            let gez: String = if j.len() > aem {
                let mut e: String = j.bw().take(aem - 2).collect();
                e.t("..");
                e
            } else {
                String::from(j)
            };
            let dac = dzm as i32 + (cyq as i32 - gez.len() as i32 * 8) / 2;
            let urj = (bmg + esd - 20) as i32;
            let csp = if qe { I_ } else { jsz };
            self.en(dac, urj, &gez, csp);
        }
        
        
        let uo = (lw + mg as i32).ao(24) as u32;
        framebuffer::ah(zb, uo, hk, 24, ham);
        framebuffer::zs(zb, uo, hk, hzr);
        let hpf = cql.len();
        let fvq = if hpf == 1 { String::from("1 item") } else { alloc::format!("{} items", hpf) };
        self.en(bqw + 10, uo as i32 + 6, &fvq, 0xFF406850);
    }
    
    fn cxp(bt: &str) -> &str {
        let ux = bt.em();
        if let Some(hbe) = ux.du(']') {
            let jzr = if hbe + 1 < ux.len() { &ux[hbe + 1..] } else { "" };
            let ek: Vec<&str> = jzr.ayt().collect();
            if !ek.is_empty() { ek[0] } else { "???" }
        } else {
            ux
        }
    }

    
    
    
    
    fn tjp(&mut self, b: i32, c: i32, apj: u32) {
        let (ash, fx, lw, hk, mg, kvs, byy, joh) = {
            if let Some(d) = self.ee.iter().du(|d| d.ad == apj && d.ld == WindowType::Ak) {
                (d.ld, d.b, d.c, d.z, d.ac, d.wn.clone(), d.ca.len(), d.acm)
            } else { return; }
        };
        
        let gl = lw + J_ as i32;
        let bpb = 36i32;
        let aiq = self.avu.get(&apj).map(|bb| if bb.ian { 0i32 } else { bb.iao as i32 }).unwrap_or(180);
        
        
        let kn = gl + 7;
        let acv = 22i32;
        
        
        if b >= fx + 8 && b < fx + 8 + acv && c >= kn && c < kn + acv {
            
            let qmi = self.avu.ds(&apj).and_then(|bb| bb.tgm().map(|e| String::from(e)));
            if let Some(path) = qmi {
                self.jgn(apj, &path);
            }
            return;
        }
        
        if b >= fx + 34 && b < fx + 34 + acv && c >= kn && c < kn + acv {
            let szr = self.avu.ds(&apj).and_then(|bb| bb.tgn().map(|e| String::from(e)));
            if let Some(path) = szr {
                self.jgn(apj, &path);
            }
            return;
        }
        
        if b >= fx + 60 && b < fx + 60 + acv && c >= kn && c < kn + acv {
            self.jgm("..");
            return;
        }
        
        
        let bco = if hk > 400 { 180i32 } else if hk > 300 { 120i32 } else { 0i32 };
        if bco > 0 {
            let cr = fx + hk as i32 - bco - 8;
            if b >= cr && b < cr + bco && c >= kn && c < kn + acv {
                if let Some(xa) = self.avu.ds(&apj) {
                    xa.chp = true;
                }
                return;
            } else {
                
                if let Some(xa) = self.avu.ds(&apj) {
                    xa.chp = false;
                }
            }
        }
        
        
        let asj = gl + bpb + 1;
        let cpi = 24i32;
        let tc = fx + aiq;
        let ur = hk as i32 - aiq;
        if c >= asj && c < asj + cpi && b >= tc {
            let gcx = tc + (ur * 52 / 100);
            let gcw = tc + (ur * 68 / 100);
            let hdo = tc + (ur * 82 / 100);
            
            let ndk: u8 = if ur > 420 && b >= hdo { 3 }
                else if ur > 300 && b >= gcw { 2 }
                else if ur > 200 && b >= gcx { 1 }
                else { 0 };
            
            if let Some(xa) = self.avu.ds(&apj) {
                if xa.eit == ndk {
                    xa.dcc = !xa.dcc;
                } else {
                    xa.eit = ndk;
                    xa.dcc = true;
                }
            }
            
            let path = kvs.clone().unwrap_or_else(|| String::from("/"));
            self.brz(&path);
            return;
        }
        
        
        let byl = mg.ao(J_ + bpb as u32 + 1 + 26);
        let uo = gl + bpb + 1 + byl as i32;
        if c >= uo && c < uo + 24 && hk > 300 {
            let bdg = fx + hk as i32 - 120;
            let bfq = 24i32;
            
            if b >= bdg && b < bdg + bfq {
                self.eqn.insert(apj, FileManagerViewMode::Px);
                return;
            }
            
            if b >= bdg + bfq + 4 && b < bdg + bfq * 2 + 4 {
                self.eqn.insert(apj, FileManagerViewMode::Sz);
                return;
            }
            
            if b >= bdg + (bfq + 4) * 2 && b < bdg + (bfq + 4) * 2 + bfq {
                self.eqn.insert(apj, FileManagerViewMode::Aaz);
                return;
            }
        }
        
        
        let asj = gl + bpb + 1;
        if aiq > 0 && b >= fx && b < fx + aiq {
            let ali = 24i32;
            let mut cq = asj + 28; 
            
            
            if let Some(xa) = self.avu.get(&apj) {
                let vot: Vec<String> = xa.jkw.iter().map(|(_, ai)| ai.clone()).collect();
                for (a, path) in vot.iter().cf() {
                    if c >= cq && c < cq + ali {
                        crate::serial_println!("[FM] Sidebar click: Quick Access -> {}", path);
                        self.jgn(apj, path);
                        return;
                    }
                    cq += ali;
                }
            }
            
            
            cq += 36; 
            
            
            let bzh = ["/", "/tmp", "/dev", "/proc"];
            for path in &bzh {
                if c >= cq && c < cq + ali {
                    crate::serial_println!("[FM] Sidebar click: Drive -> {}", path);
                    self.jgn(apj, path);
                    return;
                }
                cq += ali;
            }
            return; 
        }
        
        
        let txp = self.eqn.get(&apj).hu().unwrap_or(FileManagerViewMode::Px) == FileManagerViewMode::Sz;
        let asu = 5usize.v(byy);
        let fin = if byy > asu + 2 { byy - 2 } else { byy };
        let bec = fin.ao(asu);
        
        if txp {
            
            let drk = gl + bpb + 1;
            let cyq = 90i32;
            let esd = 80i32;
            let tc = fx + aiq;
            let auk = hk as i32 - aiq;
            let ec = ((auk - 20) / cyq).am(1);
            let lrt = (auk - ec * cyq) / 2;
            let dbt = (self.ee.iter().du(|d| d.ad == apj).map(|d| d.px).unwrap_or(0) / ec as usize) as i32;
            
            let amr = b - tc - lrt;
            let aio = c - drk;
            if amr >= 0 && aio >= 0 {
                let bj = amr / cyq;
                let iri = aio / esd;
                let fcl = iri + dbt;
                let w = fcl * ec + bj;
                if w >= 0 && (w as usize) < bec {
                    let cwf = w as usize;
                    if cwf == joh && crate::mouse::jbf() {
                        self.osr(apj, cwf);
                        return;
                    }
                    if let Some(d) = self.ee.el().du(|d| d.ad == apj) {
                        d.acm = cwf;
                    }
                }
            }
        } else {
            
            let tc = fx + aiq;
            let cpi = 24i32;
            let lix = asj + cpi + 1;
            let ph = 26i32;
            
            let aio = c - lix;
            if aio >= 0 && b >= tc {
                let px = self.ee.iter().du(|d| d.ad == apj).map(|d| d.px).unwrap_or(0);
                let cwf = px + (aio / ph) as usize;
                if cwf < bec {
                    if cwf == joh && crate::mouse::jbf() {
                        self.osr(apj, cwf);
                        return;
                    }
                    if let Some(d) = self.ee.el().du(|d| d.ad == apj) {
                        d.acm = cwf;
                    }
                }
            }
        }
    }
    
    
    fn jgn(&mut self, apj: u32, path: &str) {
        
        let mqi: Vec<u32> = self.ee.iter().hi(|d| d.ja).map(|d| d.ad).collect();
        for d in &mut self.ee {
            d.ja = d.ad == apj;
        }
        
        if let Some(bh) = self.ee.el().du(|d| d.ad == apj) {
            bh.wn = Some(String::from(path));
        }
        self.brz(path);
        
        if let Some(xa) = self.avu.ds(&apj) {
            xa.lwg(path);
        }
        
        for d in &mut self.ee {
            d.ja = mqi.contains(&d.ad);
        }
    }
    
    fn osr(&mut self, apj: u32, bea: usize) {
        let (it, ta) = {
            if let Some(d) = self.ee.iter().du(|d| d.ad == apj) {
                let asu = 5usize.v(d.ca.len());
                let baw = asu + bea;
                if baw < d.ca.len().ao(2) {
                    let line = &d.ca[baw];
                    let ta = line.contains("[D]");
                    let j = Self::cxp(line);
                    (String::from(j), ta)
                } else { return; }
            } else { return; }
        };
        
        if ta {
            self.jgm(&it);
        } else {
            self.gol(&it);
        }
    }

    
    
    
    
    fn zpn(&mut self, apj: u32) {
        if let Some(d) = self.ee.iter().du(|d| d.ad == apj && d.ld == WindowType::Ak) {
            let asu = 5usize.v(d.ca.len());
            let baw = asu + d.acm;
            if baw < d.ca.len().ao(2) {
                let line = &d.ca[baw];
                let ta = line.contains("[D]");
                let j = Self::cxp(line);
                if j == ".." { return; }
                let rp = d.wn.clone().unwrap_or_else(|| String::from("/"));
                let wo = if rp == "/" {
                    alloc::format!("/{}", j)
                } else {
                    alloc::format!("{}/{}", rp, j)
                };
                self.eaz = Some(Bez {
                    mgm: wo,
                    it: String::from(j),
                    ta,
                    ql: self.lf,
                    vc: self.ot,
                    aua: self.lf,
                    bbi: self.ot,
                    pmc: apj,
                    gh: true,
                });
                crate::serial_println!("[DnD] Started drag: {}", j);
            }
        }
    }
    
    fn pxb(&mut self, b: i32, c: i32) {
        if let Some(ref mut bzf) = self.eaz {
            bzf.aua = b;
            bzf.bbi = c;
        }
    }
    
    fn nuo(&mut self, b: i32, c: i32) {
        let sax = self.eaz.take();
        if let Some(bzf) = sax {
            
            let xba = self.ee.iter()
                .hi(|d| d.ld == WindowType::Ak && d.ad != bzf.pmc)
                .du(|d| b >= d.b && b < d.b + d.z as i32 && c >= d.c && c < d.c + d.ac as i32);
            
            if let Some(cd) = xba {
                let mjz = cd.wn.clone().unwrap_or_else(|| String::from("/"));
                let dge = if mjz == "/" {
                    alloc::format!("/{}", bzf.it)
                } else {
                    alloc::format!("{}/{}", mjz, bzf.it)
                };
                
                
                if !bzf.ta {
                    if let Ok(f) = crate::ramfs::fh(|fs| fs.mq(&bzf.mgm).map(|bc| bc.ip())) {
                        let _ = crate::ramfs::fh(|fs| fs.ns(&dge, &f));
                        crate::serial_println!("[DnD] Copied {} -> {}", bzf.mgm, dge);
                    }
                } else {
                    let _ = crate::ramfs::fh(|fs| fs.ut(&dge));
                    crate::serial_println!("[DnD] Created dir: {}", dge);
                }
                
                
                self.vtv(cd.ad, &mjz);
            } else if c >= (self.ac - W_) as i32 {
                
                crate::serial_println!("[DnD] Dropped on taskbar, ignoring");
            } else {
                
                crate::serial_println!("[DnD] Dropped on desktop: {}", bzf.it);
            }
        }
    }
    
    fn nmx(&self) {
        if let Some(ref bzf) = self.eaz {
            if !bzf.gh { return; }
            let qz = bzf.aua;
            let ub = bzf.bbi;
            
            
            framebuffer::ih(qz as u32, ub as u32, 70, 22, 0x0C1410, 180);
            tf(qz, ub, 70, 22, 4, I_);
            
            
            if bzf.ta {
                framebuffer::ah((qz + 4) as u32, (ub + 4) as u32, 14, 14, 0xFFDDAA30);
            } else {
                framebuffer::ah((qz + 4) as u32, (ub + 4) as u32, 12, 14, 0xFF60AA80);
            }
            
            
            let aem = 6;
            let j: String = bzf.it.bw().take(aem).collect();
            self.cb(qz + 22, ub + 5, &j, I_);
        }
    }
    
    fn vtv(&mut self, ajq: u32, path: &str) {
        
        let mqi: Vec<u32> = self.ee.iter().hi(|d| d.ja).map(|d| d.ad).collect();
        for d in &mut self.ee {
            d.ja = d.ad == ajq;
        }
        self.brz(path);
        
        for d in &mut self.ee {
            d.ja = mqi.contains(&d.ad);
        }
    }

    
    
    
    
    fn iul(&mut self, niq: bool) {
        if let Some(d) = self.ee.iter().du(|d| d.ja && d.ld == WindowType::Ak) {
            let asu = 5usize.v(d.ca.len());
            let baw = asu + d.acm;
            if baw < d.ca.len().ao(2) {
                let line = &d.ca[baw];
                let j = Self::cxp(line);
                if j == ".." { return; }
                let rp = d.wn.clone().unwrap_or_else(|| String::from("/"));
                let wo = if rp == "/" {
                    alloc::format!("/{}", j)
                } else {
                    alloc::format!("{}/{}", rp, j)
                };
                let op = if niq { "Cut" } else { "Copied" };
                crate::serial_println!("[FM] {} file: {}", op, wo);
                self.iuk = Some(Bgx {
                    path: wo,
                    j: String::from(j),
                    jbe: niq,
                });
                
                crate::keyboard::eno(j);
            }
        }
    }
    
    fn ntj(&mut self) {
        let fex = self.iuk.take();
        if let Some(bt) = fex {
            let rp = self.ee.iter()
                .du(|d| d.ja && d.ld == WindowType::Ak)
                .and_then(|d| d.wn.clone())
                .unwrap_or_else(|| String::from("/"));
            
            let aac = if rp == "/" {
                alloc::format!("/{}", bt.j)
            } else {
                alloc::format!("{}/{}", rp, bt.j)
            };
            
            if bt.jbe {
                
                let _ = crate::ramfs::fh(|fs| fs.euz(&bt.path, &aac));
                crate::serial_println!("[FM] Moved {} -> {}", bt.path, aac);
            } else {
                
                if let Ok(f) = crate::ramfs::fh(|fs| fs.mq(&bt.path).map(|bc| bc.ip())) {
                    let _ = crate::ramfs::fh(|fs| fs.ns(&aac, &f));
                    crate::serial_println!("[FM] Pasted {} -> {}", bt.path, aac);
                }
                
                self.iuk = Some(bt);
            }
            
            self.brz(&rp);
        }
    }

    
    
    
    
    fn nng(&self) {
        let kp = self.z;
        let kl = self.ac;
        
        
        framebuffer::ah(0, 0, kp, kl, 0xFF040808);
        
        
        let ec = kp / 10;
        for r in 0..ec {
            let dv = r.hx(7919).cn(self.oo as u32);
            let cpi = (dv % 20) + 3;
            let ffl = r * 10;
            let rmc = (dv.hx(13) % kl) as i32;
            for m in 0..cpi {
                let ix = rmc + m as i32 * 14;
                if ix >= 0 && ix < kl as i32 - 14 {
                    let kt = (255 - m * 12).am(20);
                    let s = (kt << 8) | 0xFF000000;
                    let qya = ((dv.cn(m)) % 26 + 65) as u8 as char;
                    let mut k = [0u8; 4];
                    let khb = qya.hia(&mut k);
                    framebuffer::cb(khb, ffl, ix as u32, s);
                }
            }
        }
        
        
        let yd = 360u32;
        let ans = 280u32;
        let y = (kp - yd) / 2;
        let x = (kl - ans) / 2;
        let wmh = if self.eeu > 0 {
            let dyg = (self.eeu as i32 * 3) % 13 - 6;
            dyg
        } else { 0 };
        let y = (y as i32 + wmh) as u32;
        
        
        framebuffer::ih(y, x, yd, ans, 0x0C1A12, 200);
        mf(y as i32, x as i32, yd, ans, 12, 0xFF0A1A0F);
        tf(y as i32, x as i32, yd, ans, 12, X_);
        
        
        let cnf = (y + yd / 2).ao(40) as i32;
        self.en(cnf, (x + 30) as i32, "TrustOS", I_);
        
        
        let cli = y + yd / 2 - 12;
        let bhj = x + 70;
        
        framebuffer::ah(cli, bhj, 24, 3, BH_);
        framebuffer::ah(cli, bhj, 3, 16, BH_);
        framebuffer::ah(cli + 21, bhj, 3, 16, BH_);
        
        framebuffer::ah(cli - 4, bhj + 16, 32, 22, X_);
        framebuffer::ah(cli + 8, bhj + 22, 8, 10, 0xFF040A08);
        
        
        let uhv = (y + yd / 2).ao(24) as i32;
        self.en(uhv, (bhj + 48) as i32, "Locked", BK_);
        
        
        let time = &self.gbz;
        if !time.is_empty() {
            let mku = (y + yd / 2).ao((time.len() as u32 * 8) / 2) as i32;
            self.en(mku, (x + 150) as i32, time, I_);
        }
        let hff = &self.hcd;
        if !hff.is_empty() {
            let rts = (y + yd / 2).ao((hff.len() as u32 * 8) / 2) as i32;
            self.en(rts, (x + 170) as i32, hff, BK_);
        }
        
        
        let alf = x + 200;
        let ess = 200u32;
        let cky = y + (yd - ess) / 2;
        mf(cky as i32, alf as i32, ess, 30, 6, 0xFF081208);
        tf(cky as i32, alf as i32, ess, 30, 6, P_);
        
        
        let hgp: String = self.djb.bw().map(|_| '*').collect();
        if hgp.is_empty() {
            self.en((cky + 8) as i32, (alf + 8) as i32, "Enter PIN...", P_);
        } else {
            self.en((cky + 8) as i32, (alf + 8) as i32, &hgp, I_);
        }
        
        
        if self.btx {
            let cx = cky + 8 + hgp.len() as u32 * 8;
            framebuffer::ah(cx, alf + 6, 2, 18, I_);
        }
        
        
        self.en((y + yd / 2 - 80) as i32, (alf + 42) as i32, "Press Enter to unlock", P_);
        
        
        if self.eeu > 0 {
            self.en((y + yd / 2 - 50) as i32, (alf + 60) as i32, "Wrong PIN!", 0xFFCC4444);
        }
    }
    
    fn tki(&mut self, bs: u8) {
        if self.eeu > 0 {
            self.eeu = self.eeu.ao(1);
        }
        
        if bs == 0x0D || bs == 0x0A { 
            
            if self.djb.is_empty() || self.djb == "0000" || self.djb == "1234" {
                self.eug = false;
                self.djb.clear();
                crate::serial_println!("[LOCK] Screen unlocked");
            } else {
                
                self.eeu = 15;
                self.djb.clear();
                crate::serial_println!("[LOCK] Wrong PIN");
            }
        } else if bs == 0x08 { 
            self.djb.pop();
        } else if bs >= 0x20 && bs < 0x7F && self.djb.len() < 16 {
            self.djb.push(bs as char);
        }
    }

    
    
    
    
    fn sfy(&self, guy: u32, mmz: u32) {
        
        let pzj = guy;
        let pzk = mmz + 2;
        let mqq = if self.mje { I_ } else { 0xFF553333 };
        
        for kau in 0..3u32 {
            let m = 3 + kau * 3;
            let cx = pzj + 8;
            let ae = pzk + 12;
            
            for gyp in 0..8u32 {
                let dx = (gyp * m) / 8;
                let hhm = (m * m).ao(dx * dx);
                
                let mut bg = 0u32;
                while (bg + 1) * (bg + 1) <= hhm { bg += 1; }
                let y = cx + dx;
                let x = ae.ao(bg);
                if y > 0 && x > 0 {
                    framebuffer::ii(y, x, mqq);
                    
                    if cx >= dx {
                        framebuffer::ii(cx - dx, x, mqq);
                    }
                }
            }
        }
        
        framebuffer::ah(pzj + 7, pzk + 11, 3, 3, mqq);
        
        
        let igv = guy + 22;
        let ccl = mmz + 3;
        let mpu = BH_;
        
        framebuffer::ah(igv, ccl + 4, 4, 6, mpu);
        framebuffer::ah(igv + 4, ccl + 2, 3, 10, mpu);
        
        let xuc = (self.gtw / 34).v(3); 
        for d in 0..xuc {
            let xvz = igv + 9 + d * 3;
            let pzh = 2 + d * 2;
            let xwa = ccl + 7;
            framebuffer::ah(xvz, xwa.ao(pzh), 1, pzh * 2, mpu);
        }
        if self.gtw == 0 {
            
            framebuffer::ah(igv + 9, ccl + 3, 1, 8, 0xFFCC4444);
            framebuffer::ah(igv + 12, ccl + 3, 1, 8, 0xFFCC4444);
        }
        
        
        let ila = guy + 44;
        let ilb = mmz + 4;
        let ikz = 18u32;
        let mye = 8u32;
        
        framebuffer::lx(ila, ilb, ikz, mye, P_);
        
        framebuffer::ah(ila + ikz, ilb + 2, 2, 4, P_);
        
        let akd = ((self.icm as u32 * (ikz - 2)) / 100).am(1);
        let qny = if self.icm > 50 { I_ }
            else if self.icm > 20 { FY_ }
            else { DC_ };
        framebuffer::ah(ila + 1, ilb + 1, akd, mye - 2, qny);
        
        
        let qnz = alloc::format!("{}%", self.icm);
        self.cb((ila + ikz + 5) as i32, ilb as i32, &qnz, P_);
    }

    
    fn scv(&self, bh: &Window) {
        
        let jvm = self.eqn.get(&bh.ad).hu().unwrap_or(FileManagerViewMode::Px);
        if jvm == FileManagerViewMode::Sz {
            self.scw(bh);
            return;
        }
        
        let fx = bh.b;
        let lw = bh.c;
        let hk = bh.z;
        let mg = bh.ac;
        
        if hk < 120 || mg < 140 { return; }
        
        let gl = lw + J_ as i32;
        let zb = fx.am(0) as u32;
        
        
        let gba     = 0xFF081008u32;  
        let mzb = 0xFF0C2810u32;  
        let qpl = 0xFF0A1C0Cu32;  
        let kcx     = 0xFF0A120Cu32;  
        let ham     = 0xFF0C140Cu32;  
        let kcz      = 0xFF0C180Eu32;  
        let kdb    = 0xFF0A120Cu32;
        let kdd     = 0xFF0C140Cu32;
        let kdc   = 0xFF0E1E10u32;  
        let hak    = 0xFF0A3818u32;  
        let qpk      = 0xFF060C06u32;  
        let psu   = 0xFF50AA60u32;
        let mkm = 0xFF3A7A4Au32;  
        let jta    = 0xFF50CC70u32;
        let jsz      = 0xFF80CC90u32;
        let ahv       = 0xFF406850u32;
        let fkx    = 0xFFDDAA30u32;
        let lda      = 0xFF60AA80u32;
        let dvo      = 0xFF142014u32;
        let mm         = I_;
        
        
        let xa = self.avu.get(&bh.ad);
        let aiq = xa.map(|bb| if bb.ian { 0u32 } else { bb.iao }).unwrap_or(180);
        let tqb = xa.and_then(|bb| bb.ocf);
        
        
        let bpb = 36u32;
        framebuffer::ah(zb, gl as u32, hk, bpb, ham);
        
        let kn = gl + 7;
        let acv = 22u32;
        
        
        let imw = xa.map(|bb| bb.nbo()).unwrap_or(false);
        let qmh = if imw { AG_ } else { 0xFF1A2A1A };
        mf(fx + 8, kn, acv, acv, 4, 0xFF101810);
        if imw { tf(fx + 8, kn, acv, acv, 4, P_); }
        self.cb(fx + 14, kn + 4, "<", qmh);
        
        
        let imx = xa.map(|bb| bb.nbp()).unwrap_or(false);
        let szq = if imx { AG_ } else { 0xFF1A2A1A };
        mf(fx + 34, kn, acv, acv, 4, 0xFF101810);
        if imx { tf(fx + 34, kn, acv, acv, 4, P_); }
        self.cb(fx + 40, kn + 4, ">", szq);
        
        
        mf(fx + 60, kn, acv, acv, 4, 0xFF101810);
        tf(fx + 60, kn, acv, acv, 4, P_);
        self.cb(fx + 66, kn + 4, "^", BH_);
        
        
        let ewg = fx + 90;
        let bco = if hk > 400 { 180i32 } else if hk > 300 { 120i32 } else { 0i32 };
        let fql = (hk as i32 - 100 - bco - 10).am(60);
        
        mf(ewg, kn, fql as u32, acv, 6, 0xFF080E08);
        tf(ewg, kn, fql as u32, acv, 6, dvo);
        
        
        let rp = bh.wn.ahz().unwrap_or("/");
        let mut y = ewg + 10;
        let ek: Vec<&str> = rp.adk('/').hi(|e| !e.is_empty()).collect();
        
        
        self.en(y, kn + 5, "\x07", 0xFF40AA50); 
        y += 12;
        
        if ek.is_empty() {
            self.en(y, kn + 5, "This PC", mm);
        } else {
            self.en(y, kn + 5, "This PC", BH_);
            y += 56;
            for (a, vu) in ek.iter().cf() {
                if y > ewg + fql - 30 { 
                    self.en(y, kn + 5, "...", P_);
                    break; 
                }
                
                self.en(y, kn + 5, ">", 0xFF2A4A30);
                y += 12;
                let fmd = a == ek.len() - 1;
                let r = if fmd { mm } else { BH_ };
                self.en(y, kn + 5, vu, r);
                y += (vu.len() as i32) * 8 + 6;
            }
        }
        
        
        if bco > 0 {
            let cr = fx + hk as i32 - bco - 8;
            let chp = xa.map(|bb| bb.chp).unwrap_or(false);
            let wda = if chp { 0xFF081008 } else { qpk };
            let wdb = if chp { mm } else { dvo };
            mf(cr, kn, bco as u32, acv, 6, wda);
            tf(cr, kn, bco as u32, acv, 6, wdb);
            
            self.en(cr + 8, kn + 5, "\x0F", if chp { mm } else { P_ });
            let query = xa.map(|bb| bb.bla.as_str()).unwrap_or("");
            if query.is_empty() {
                self.en(cr + 22, kn + 5, "Search", ahv);
            } else {
                self.en(cr + 22, kn + 5, query, jsz);
            }
            
            if chp && (self.oo / 30) % 2 == 0 {
                let lf = cr + 22 + (query.len() as i32) * 8;
                framebuffer::ah(lf as u32, (kn + 4) as u32, 1, 14, mm);
            }
        }
        
        
        framebuffer::zs(zb, (gl + bpb as i32) as u32, hk, dvo);
        
        
        let asj = gl + bpb as i32 + 1;
        let byl = mg.ao(J_ + bpb + 1 + 26); 
        
        if aiq > 0 && byl > 20 {
            framebuffer::ah(zb, asj as u32, aiq, byl, gba);
            
            let mut cq = asj + 8;
            let ali = 24i32;
            let eis = fx + 6;
            let mfu = aiq.ao(12);
            
            
            self.en(eis + 4, cq, "Quick Access", mkm);
            
            self.en(eis as i32 + mfu as i32 - 8, cq, "v", mkm);
            cq += 20;
            
            if let Some(nvg) = xa {
                for (a, (j, path)) in nvg.jkw.iter().cf() {
                    if cq + ali > asj + byl as i32 - 40 { break; }
                    
                    let afb = bh.wn.ahz() == Some(path.as_str());
                    let apx = nvg.pks == a as i32;
                    
                    
                    let hyc = if afb { mzb } else if apx { qpl } else { gba };
                    if afb || apx {
                        mf(eis, cq - 2, mfu, ali as u32, 4, hyc);
                    }
                    
                    if afb {
                        framebuffer::ah(zb + 2, cq as u32, 3, (ali - 4) as u32, mm);
                    }
                    
                    
                    let drv = (eis + 12) as u32;
                    let drw = (cq + 2) as u32;
                    framebuffer::ah(drv, drw, 6, 2, fkx);
                    framebuffer::ah(drv, drw + 2, 12, 8, fkx);
                    framebuffer::ah(drv + 1, drw + 4, 10, 5, 0xFF0A0A04);
                    
                    let csp = if afb { mm } else { psu };
                    self.en(eis + 30, cq + 3, j, csp);
                    
                    cq += ali;
                }
            }
            
            
            cq += 6;
            framebuffer::zs(zb + 10, cq as u32, aiq.ao(20), dvo);
            cq += 10;
            
            
            self.en(eis + 4, cq, "This PC", mkm);
            cq += 20;
            
            
            let bzh = [
                ("\x07", "Local Disk (C:)", "/"),
                ("\x07", "RAM Disk",        "/tmp"),
                ("\x07", "Devices",         "/dev"),
                ("\x07", "System",          "/proc"),
            ];
            
            for (pa, j, path) in &bzh {
                if cq + ali > asj + byl as i32 - 4 { break; }
                let afb = bh.wn.ahz() == Some(*path);
                
                if afb {
                    mf(eis, cq - 2, mfu, ali as u32, 4, mzb);
                    framebuffer::ah(zb + 2, cq as u32, 3, (ali - 4) as u32, mm);
                }
                
                
                let drv = (eis + 12) as u32;
                let drw = (cq + 2) as u32;
                framebuffer::ah(drv, drw, 12, 10, 0xFF406050);
                framebuffer::ah(drv + 1, drw + 1, 10, 3, 0xFF60AA80);
                framebuffer::ah(drv + 4, drw + 5, 4, 3, 0xFF80CC90);
                
                let r = if afb { mm } else { psu };
                self.en(eis + 30, cq + 3, j, r);
                cq += ali;
            }
            
            
            framebuffer::ah(zb + aiq - 1, asj as u32, 1, byl, dvo);
        }
        
        
        let tc = fx + aiq as i32;
        let ur = hk.ao(aiq);
        
        
        let cpi = 24u32;
        framebuffer::ah((tc.am(0)) as u32, asj as u32, ur, cpi, kcz);
        
        
        let ner = tc + 36;
        let gcx = tc + (ur as i32 * 52 / 100);
        let gcw = tc + (ur as i32 * 68 / 100);
        let hdo = tc + (ur as i32 * 82 / 100);
        
        let crj = asj + 5;
        
        
        let iba = xa.map(|bb| bb.eit).unwrap_or(0);
        let mgk = xa.map(|bb| bb.dcc).unwrap_or(true);
        let mgj = if mgk { "v" } else { "^" };
        
        
        self.en(ner, crj, "Name", jta);
        if iba == 0 { self.en(ner + 40, crj, mgj, P_); }
        
        if ur > 200 {
            framebuffer::ah(gcx as u32 - 2, asj as u32 + 4, 1, cpi - 8, dvo);
            self.en(gcx, crj, "Type", jta);
            if iba == 1 { self.en(gcx + 36, crj, mgj, P_); }
        }
        if ur > 300 {
            framebuffer::ah(gcw as u32 - 2, asj as u32 + 4, 1, cpi - 8, dvo);
            self.en(gcw, crj, "Size", jta);
            if iba == 2 { self.en(gcw + 36, crj, mgj, P_); }
        }
        if ur > 420 {
            framebuffer::ah(hdo as u32 - 2, asj as u32 + 4, 1, cpi - 8, dvo);
            self.en(hdo, crj, "Open with", jta);
        }
        
        framebuffer::zs((tc.am(0)) as u32, (asj + cpi as i32) as u32, ur, dvo);
        
        
        let ou = asj + cpi as i32 + 1;
        let bae = byl.ao(cpi + 27); 
        if bae < 8 { return; }
        
        framebuffer::ah((tc.am(0)) as u32, ou as u32, ur, bae, kcx);
        
        let ph = 26u32; 
        let ayf = (bae / ph).am(1) as usize;
        
        
        let asu = 5usize.v(bh.ca.len());
        let fin = if bh.ca.len() > asu + 2 { bh.ca.len() - 2 } else { bh.ca.len() };
        let cql: Vec<&str> = if fin > asu {
            bh.ca[asu..fin].iter().map(|e| e.as_str()).collect()
        } else { Vec::new() };
        
        if cql.is_empty() {
            self.en(tc + 30, ou + 30, "This folder is empty.", ahv);
            self.en(tc + 30, ou + 50, "Press N to create a file, D for a folder.", P_);
        }
        
        let jc = bh.px;
        let mpl = cql.len().v(ayf);
        
        for afj in 0..mpl {
            let bea = jc + afj;
            if bea >= cql.len() { break; }
            let line = cql[bea];
            let ix = ou as u32 + (afj as u32) * ph;
            if ix + ph > ou as u32 + bae { break; }
            
            let qe = bea == bh.acm;
            let ta = line.contains("[D]");
            let apx = tqb == Some(bea);
            
            
            let hyc = if qe {
                hak
            } else if apx {
                kdc
            } else if afj % 2 == 0 {
                kdb
            } else {
                kdd
            };
            framebuffer::ah((tc.am(0)) as u32, ix, ur, ph, hyc);
            
            
            if qe {
                
                framebuffer::ah((tc.am(0)) as u32, ix + 3, 3, ph - 6, mm);
                
                tf(tc, ix as i32, ur, ph, 3, 0xFF1A4A28);
            }
            
            let sl = (ix + 6) as i32;
            let peg = if qe { mm } else { jsz };
            
            
            let fg = (tc + 10).am(0) as u32;
            let og = ix + 3;
            let cfl = 20u32;
            
            if ta {
                
                let gc = if qe { 0xFFEECC50 } else { fkx };
                let iuc = if qe { 0xFFCCAA30 } else { 0xFFBB8820 };
                
                framebuffer::ah(fg, og, cfl / 2, 4, gc);
                
                framebuffer::ah(fg, og + 4, cfl, cfl - 4, gc);
                
                framebuffer::ah(fg + 2, og + 7, cfl - 4, cfl - 10, iuc);
                
                framebuffer::ah(fg + 4, og + 9, cfl - 8, 1, 0xFF0A0A04);
                framebuffer::ah(fg + 4, og + 12, cfl / 2, 1, 0xFF0A0A04);
            } else {
                
                let gc = if qe { 0xFF80DDAA } else { lda };
                let iuc = 0xFF0A140A;
                
                framebuffer::ah(fg + 2, og, cfl - 6, cfl, gc);
                
                framebuffer::ah(fg + cfl - 8, og, 4, 6, iuc);
                framebuffer::ah(fg + cfl - 8, og, 1, 6, gc);
                framebuffer::ah(fg + cfl - 8, og + 5, 4, 1, gc);
                
                framebuffer::ah(fg + 4, og + 8, cfl - 10, cfl - 10, iuc);
                
                framebuffer::ah(fg + 5, og + 10, 8, 1, 0xFF1A3A1A);
                framebuffer::ah(fg + 5, og + 13, 6, 1, 0xFF1A3A1A);
                framebuffer::ah(fg + 5, og + 16, 7, 1, 0xFF1A3A1A);
                
                
                let cqi = Self::cxp(line);
                let kbw = if cqi.pp(".rs") { 0xFFFF6633 }       
                    else if cqi.pp(".txt") { 0xFF4488CC }               
                    else if cqi.pp(".md") { 0xFF5599DD }                
                    else if cqi.pp(".toml") { 0xFF8866BB }              
                    else if cqi.pp(".json") { 0xFFDDAA00 }              
                    else if cqi.pp(".html") || cqi.pp(".htm") { 0xFFEE6633 }
                    else if cqi.pp(".css") { 0xFF3399EE }
                    else if cqi.pp(".png") || cqi.pp(".jpg") || cqi.pp(".bmp") { 0xFF33BB66 }
                    else if cqi.pp(".mp3") || cqi.pp(".wav") { 0xFFEE55AA }
                    else { 0xFF446644 };
                framebuffer::ah(fg + 3, og + cfl - 5, 6, 4, kbw);
            }
            
            
            let ux = line.em();
            let (amj, bde, als, vna) = if let Some(hbe) = ux.du(']') {
                let jzr = if hbe + 1 < ux.len() { &ux[hbe + 1..] } else { "" };
                let ek: Vec<&str> = jzr.ayt().collect();
                (
                    if !ek.is_empty() { ek[0] } else { "???" },
                    if ek.len() > 1 { ek[1] } else { "" },
                    if ek.len() > 2 { ek[2] } else { "" },
                    if ek.len() > 3 { ek[3] } else { "" },
                )
            } else {
                (ux, "", "", "")
            };
            
            
            
            let dac = tc + 36;
            if let Some(fgw) = amj.bhx('.') {
                let ar = &amj[..fgw];
                let wm = &amj[fgw..];
                self.en(dac, sl, ar, peg);
                let spx = dac + (ar.len() as i32) * 8;
                self.en(spx, sl, wm, if qe { BH_ } else { ahv });
            } else {
                self.en(dac, sl, amj, peg);
            }
            
            
            if ur > 200 {
                let mnr = if ta { "File folder" } else {
                    match amj.cmm('.').next() {
                        Some("rs") => "Rust Source",
                        Some("txt") => "Text Document",
                        Some("md") => "Markdown",
                        Some("toml") => "TOML Config",
                        Some("json") => "JSON File",
                        Some("html") | Some("htm") => "HTML Document",
                        Some("css") => "Stylesheet",
                        Some("png") | Some("jpg") | Some("bmp") => "Image",
                        Some("mp3") | Some("wav") => "Audio",
                        Some("sh") => "Shell Script",
                        _ => bde,
                    }
                };
                let asb = if qe { BH_ } else { 0xFF50886A };
                self.en(gcx, sl, mnr, asb);
            }
            
            
            if ur > 300 {
                let wpa = if ta {
                    String::from("")
                } else if let Ok(bf) = als.parse::<u64>() {
                    if bf < 1024 { alloc::format!("{} B", bf) }
                    else if bf < 1024 * 1024 { alloc::format!("{} KB", bf / 1024) }
                    else { alloc::format!("{:.1} MB", bf as f64 / (1024.0 * 1024.0)) }
                } else {
                    String::from(als)
                };
                let jt = if qe { BH_ } else { 0xFF50886A };
                self.en(gcw, sl, &wpa, jt);
            }
            
            
            if ur > 420 {
                let fz = if qe { P_ } else { 0xFF406050 };
                self.en(hdo, sl, vna, fz);
            }
            
            
            framebuffer::zs((tc.am(0)) as u32, ix + ph - 1, ur, 0xFF0E160E);
        }
        
        
        if cql.len() > ayf && bae > 20 {
            let mby = 5u32;
            let auz = (tc as u32 + ur).ao(mby + 2);
            let bdc = bae.ao(4);
            framebuffer::ah(auz, ou as u32 + 2, mby, bdc, 0xFF0A160C);
            let es = cql.len() as u32;
            let iw = ayf as u32;
            let axd = ((iw * bdc) / es.am(1)).am(20).v(bdc);
            let aye = es.ao(iw);
            let bsm = if aye > 0 {
                ou as u32 + 2 + ((jc as u32 * bdc.ao(axd)) / aye)
            } else {
                ou as u32 + 2
            };
            mf(auz as i32, bsm as i32, mby, axd, 2, 0xFF204030);
        }
        
        
        let uo = (asj + byl as i32) as u32;
        let bfm = 24u32;
        framebuffer::ah(zb, uo, hk, bfm, ham);
        framebuffer::zs(zb, uo, hk, dvo);
        
        
        let hpf = cql.len();
        let fvq = if hpf == 1 {
            String::from("1 item")
        } else {
            alloc::format!("{} items", hpf)
        };
        self.en(fx + aiq as i32 + 10, uo as i32 + 6, &fvq, ahv);
        
        
        if bh.acm < cql.len() {
            let pho = Self::cxp(cql[bh.acm]);
            if pho != ".." {
                let wgr = alloc::format!("| {}", pho);
                self.en(fx + aiq as i32 + 80, uo as i32 + 6, &wgr, P_);
            }
        }
        
        
        if hk > 300 {
            let ccj = uo as i32 + 3;
            let bdg = fx + hk as i32 - 120;
            let bfq = 24i32;
            
            
            let ojl = jvm == FileManagerViewMode::Px;
            let liv = if ojl { mm } else { P_ };
            mf(bdg, ccj, bfq as u32, 18, 3, if ojl { 0xFF102810 } else { 0xFF0A140A });
            
            framebuffer::ah((bdg + 5) as u32, (ccj + 4) as u32, 14, 2, liv);
            framebuffer::ah((bdg + 5) as u32, (ccj + 8) as u32, 14, 2, liv);
            framebuffer::ah((bdg + 5) as u32, (ccj + 12) as u32, 14, 2, liv);
            
            
            let nzq = jvm == FileManagerViewMode::Sz;
            let ixg = if nzq { mm } else { P_ };
            mf(bdg + bfq + 4, ccj, bfq as u32, 18, 3, if nzq { 0xFF102810 } else { 0xFF0A140A });
            
            framebuffer::ah((bdg + bfq + 8) as u32, (ccj + 4) as u32, 6, 5, ixg);
            framebuffer::ah((bdg + bfq + 16) as u32, (ccj + 4) as u32, 6, 5, ixg);
            framebuffer::ah((bdg + bfq + 8) as u32, (ccj + 11) as u32, 6, 5, ixg);
            framebuffer::ah((bdg + bfq + 16) as u32, (ccj + 11) as u32, 6, 5, ixg);
            
            
            let nkv = jvm == FileManagerViewMode::Aaz;
            let geo = if nkv { mm } else { P_ };
            mf(bdg + (bfq + 4) * 2, ccj, bfq as u32, 18, 3, if nkv { 0xFF102810 } else { 0xFF0A140A });
            
            framebuffer::ah((bdg + (bfq + 4) * 2 + 5) as u32, (ccj + 4) as u32, 3, 2, geo);
            framebuffer::ah((bdg + (bfq + 4) * 2 + 10) as u32, (ccj + 4) as u32, 8, 2, geo);
            framebuffer::ah((bdg + (bfq + 4) * 2 + 5) as u32, (ccj + 8) as u32, 3, 2, geo);
            framebuffer::ah((bdg + (bfq + 4) * 2 + 10) as u32, (ccj + 8) as u32, 8, 2, geo);
            framebuffer::ah((bdg + (bfq + 4) * 2 + 5) as u32, (ccj + 12) as u32, 3, 2, geo);
            framebuffer::ah((bdg + (bfq + 4) * 2 + 10) as u32, (ccj + 12) as u32, 8, 2, geo);
        }
    }

    
    
    
    fn sef(&self, bh: &Window) {
        let fx = bh.b as u32;
        let lw = bh.c as u32 + J_;
        let hk = bh.z;
        let mg = bh.ac.ao(J_);

        if hk < 80 || mg < 80 { return; }

        
        framebuffer::ih(fx, lw, hk, mg, 0x060D0A, 210);
        
        framebuffer::ih(fx + 1, lw + 1, hk - 2, 1, 0x00FF66, 30);
        framebuffer::ih(fx + 1, lw + mg - 1, hk - 2, 1, 0x00FF66, 18);
        framebuffer::ih(fx, lw + 1, 1, mg - 2, 0x00FF66, 22);
        framebuffer::ih(fx + hk - 1, lw + 1, 1, mg - 2, 0x00FF66, 22);

        let g = match self.ano.get(&bh.ad) {
            Some(e) => e,
            None => return,
        };

        let ov = 10u32;
        let yz = fx + ov;
        let aii = hk.ao(ov * 2);
        let dt = crate::graphics::scaling::bmi() as u32;

        
        
        
        let hpz = lw + 6;
        
        self.cb(yz as i32, hpz as i32, "LIBRARY", 0xFF44886A);
        if g.alm > 0 {
            let ffy = alloc::format!("{} tracks", g.alm);
            let kkx = (yz + aii).ao(ffy.len() as u32 * dt);
            self.cb(kkx as i32, hpz as i32, &ffy, 0xFF336655);
        }

        let ou = hpz + 16;
        let ayf = 5usize;
        let ph = 20u32;
        let bae = if g.alm == 0 { ph } else { (g.alm.v(ayf) as u32) * ph };

        
        framebuffer::ih(yz, ou, aii, bae, 0x0A1A12, 180);
        
        framebuffer::ih(yz, ou, aii, 1, 0x00FF66, 18);
        framebuffer::ih(yz, ou + bae - 1, aii, 1, 0x00FF66, 12);

        if g.alm == 0 {
            self.cb(yz as i32 + 8, (ou + 4) as i32, "No tracks found", 0xFF556655);
        } else {
            let jc = g.mmq.v(g.alm.ao(ayf));
            for afj in 0..ayf {
                let ieq = jc + afj;
                if ieq >= g.alm { break; }
                let ix = ou + afj as u32 * ph;
                let afb = ieq == g.dfl && g.g != PlaybackState::Af;

                
                if afb {
                    framebuffer::ih(yz + 1, ix + 1, aii - 2, ph - 2, 0x00AA44, 40);
                }

                
                let ajh = alloc::format!("{}.", ieq + 1);
                let htc = if afb { 0xFF00FFAA } else { 0xFF446655 };
                self.cb(yz as i32 + 6, (ix + 3) as i32, &ajh, htc);

                
                let j = if ieq < g.dxb.len() {
                    &g.dxb[ieq]
                } else {
                    "Unknown"
                };
                let aem = ((aii - 30) / dt) as usize;
                let gez = if j.len() > aem {
                    &j[..aem.v(j.len())]
                } else {
                    j
                };
                let csp = if afb { 0xFF00FFCC } else { 0xFF88BBAA };
                self.cb(yz as i32 + 26, (ix + 3) as i32, gez, csp);

                
                if afb && g.g == PlaybackState::Ce {
                    self.cb(yz as i32 + aii as i32 - 14, (ix + 3) as i32, ">", 0xFF00FF88);
                }
            }
        }

        
        
        
        let jhk = ou + bae + 10;
        self.cb(yz as i32, jhk as i32, "NOW PLAYING", 0xFF336655);

        
        let iaz = jhk + 16;
        let dq = &g.gst;
        self.cb(yz as i32, iaz as i32, dq, 0xFF00FFAA);
        self.cb(yz as i32 + 1, iaz as i32, dq, 0xFF00FFAA);

        
        let uo = iaz + 16;
        let status = match g.g {
            PlaybackState::Ce => "PLAYING",
            PlaybackState::Cl  => "PAUSED",
            PlaybackState::Af => "STOPPED",
        };
        let dch = match g.g {
            PlaybackState::Ce => 0xFF00CC66,
            PlaybackState::Cl  => 0xFF00AA88,
            PlaybackState::Af => 0xFF666666,
        };
        self.cb(yz as i32, uo as i32, status, dch);

        
        let cxi = (g.oz / 1000) as u32;
        let dcx = (g.alu / 1000) as u32;
        let bso = alloc::format!(
            "{}:{:02} / {}:{:02}",
            cxi / 60, cxi % 60,
            dcx / 60, dcx % 60
        );
        let mku = (yz + aii).ao(bso.len() as u32 * dt);
        self.cb(mku as i32, uo as i32, &bso, 0xFF88CCAA);

        
        let ctm = uo + 18;
        let lvu = 4u32;
        framebuffer::ih(yz, ctm, aii, lvu, 0x1A3322, 200);
        if g.alu > 0 {
            let akd = ((g.oz as u64 * aii as u64) / g.alu.am(1) as u64) as u32;
            let akd = akd.v(aii);
            if akd > 0 {
                framebuffer::ah(yz, ctm, akd, lvu, 0xFF00FF88);
                if akd > 2 {
                    framebuffer::ih(yz + akd - 2, ctm.ao(1), 4, lvu + 2, 0x00FF88, 120);
                }
            }
        }

        
        let dxm = ctm + 12;
        let ekq = 60u32;
        framebuffer::ih(yz, dxm, aii, ekq, 0x030908, 160);
        framebuffer::ih(yz, dxm, aii, 1, 0x00FF66, 20);
        framebuffer::ih(yz, dxm + ekq - 1, aii, 1, 0x00FF66, 12);

        let bkl = dxm + ekq / 2;
        let wp = (ekq / 2 - 3) as f32;

        if g.g == PlaybackState::Ce || g.g == PlaybackState::Cl {
            let jgk = aii.v(128) as usize;
            let ilh = g.rf;
            for a in 0..jgk {
                let xtz = (g.ihd + a) % 128;
                let yr = g.ve[xtz];
                let byf = yr * (1.0 + ilh * 0.5);
                let mrw = (byf * wp).am(-wp).v(wp) as i32;
                let y = yz + a as u32;
                let x = (bkl as i32 + mrw) as u32;
                let x = x.am(dxm + 2).v(dxm + ekq - 3);

                let nwy = 0xCCu32;
                let mxg = (ilh * 180.0) as u32;
                let oza = (g.abo * 60.0).v(60.0) as u32;
                let pn = bkl;
                if x < pn {
                    for dxu in x..pn {
                        let yx = 1.0 - ((pn - dxu) as f32 / wp).v(1.0) * 0.4;
                        let r = 0xFF000000 | (((oza as f32 * yx) as u32).v(0xFF) << 16)
                            | (((nwy as f32 * yx) as u32).v(0xFF) << 8)
                            | ((mxg as f32 * yx) as u32).v(0xFF);
                        framebuffer::ii(y, dxu, r);
                    }
                } else {
                    for dxu in pn..=x {
                        let yx = 1.0 - ((dxu - pn) as f32 / wp).v(1.0) * 0.4;
                        let r = 0xFF000000 | (((oza as f32 * yx) as u32).v(0xFF) << 16)
                            | (((nwy as f32 * yx) as u32).v(0xFF) << 8)
                            | ((mxg as f32 * yx) as u32).v(0xFF);
                        framebuffer::ii(y, dxu, r);
                    }
                }
                framebuffer::ii(y, x, 0xFF00FFCC);
            }
            if ilh > 0.3 {
                let suo = ((ilh - 0.3) * 50.0) as u32;
                framebuffer::ih(yz, dxm, aii, ekq, 0x00FF88, suo);
            }
        } else {
            framebuffer::ah(yz + 4, bkl, aii - 8, 1, 0xFF223322);
        }

        
        let fdb = dxm + ekq + 4;
        let tn = 14u32;
        let lo = aii / 4 - 3;
        let cdc = [
            (g.ato, 0xFF00FF44, "SB"),
            (g.aee, 0xFF00CC88, "BA"),
            (g.vs, 0xFF00AACC, "MD"),
            (g.axg, 0xFF8866FF, "TR"),
        ];
        for (cvv, (jy, s, cu)) in cdc.iter().cf() {
            let bx = yz + cvv as u32 * (lo + 3);
            framebuffer::ih(bx, fdb, lo, tn, 0x0E1E14, 160);
            let vi = (jy.v(1.0) * lo as f32) as u32;
            if vi > 0 {
                framebuffer::ah(bx, fdb, vi, tn, *s);
                framebuffer::ih(bx, fdb, vi, tn, 0xFFFFFF, 12);
            }
            self.cb(bx as i32 + 2, fdb as i32 + 2, cu, 0xFF99BB99);
        }

        
        
        
        let cdw = fdb + tn + 8;
        let qx = 28u32;

        
        let cuk = 36u32;
        let fra = 64u32;
        let qi = 4u32;
        let mmo = cuk * 3 + fra + qi * 3;
        let mmy = yz + (aii.ao(mmo)) / 2;

        
        fn bgt(xgi: &Desktop, bx: u32, je: u32, nm: u32, adn: u32, cu: &str, ei: u32, acu: u32, fwm: u32) {
            let dt = crate::graphics::scaling::bmi() as u32;
            
            framebuffer::ih(bx, je, nm, adn, ei, 210);
            
            framebuffer::ih(bx + 1, je, nm - 2, 1, acu, 80);
            
            framebuffer::ih(bx + 1, je + adn - 1, nm - 2, 1, 0x000000, 60);
            
            framebuffer::ih(bx, je + 1, 1, adn - 2, acu, 30);
            framebuffer::ih(bx + nm - 1, je + 1, 1, adn - 2, acu, 30);
            
            framebuffer::ih(bx + 1, je + 1, nm - 2, 2, 0xFFFFFF, 12);
            
            let bda = cu.len() as u32 * dt;
            let gx = bx + (nm.ao(bda)) / 2;
            let ty = je + (adn.ao(12)) / 2;
            xgi.cb(gx as i32, ty as i32, cu, fwm);
        }

        
        let bwb = mmy;
        bgt(self, bwb, cdw, cuk, qx, "|<", 0x142820, 0x00AA88, 0xFF88CCAA);

        
        let gpk = bwb + cuk + qi;
        let vix = match g.g {
            PlaybackState::Ce => "PAUSE",
            _ => "PLAY",
        };
        let vir = match g.g {
            PlaybackState::Ce => 0x0A5530,
            _ => 0x084428,
        };
        bgt(self, gpk, cdw, fra, qx, vix, vir, 0x00FF88, 0xFF00FFAA);

        
        let gti = gpk + fra + qi;
        bgt(self, gti, cdw, cuk, qx, "STOP", 0x2A1610, 0xCC6633, 0xFFFF8844);

        
        let jhb = gti + cuk + qi;
        bgt(self, jhb, cdw, cuk, qx, ">|", 0x142820, 0x00AA88, 0xFF88CCAA);

        
        let ccl = cdw + qx + 8;
        let igt = 10u32;
        self.cb(yz as i32, ccl as i32, "VOL", 0xFF44886A);

        let ier = yz + 30;
        let ekb = aii.ao(72);
        framebuffer::ih(ier, ccl + 3, ekb, 4, 0x1A3322, 200);
        let mpv = (g.hq as u32 * ekb) / 100;
        if mpv > 0 {
            framebuffer::ah(ier, ccl + 3, mpv, 4, 0xFF00CC88);
        }
        
        let etq = ier + mpv;
        if etq + 4 <= ier + ekb + 4 {
            framebuffer::ah(etq, ccl, 4, igt, 0xFF00FFAA);
        }
        let igu = alloc::format!("{}%", g.hq);
        let xst = ier + ekb + 6;
        self.cb(xst as i32, ccl as i32, &igu, 0xFF88CCAA);

        
        
        
        let iwg = ccl + igt + 10;
        
        framebuffer::ih(yz, iwg, aii, 1, 0x00FF66, 20);
        let iwf = iwg + 4;
        self.cb(yz as i32, iwf as i32, "EFFECTS", 0xFF336655);

        let azx = 24u32;
        let aju = 24u32;
        let caj = 36u32;

        
        let dmh = iwf + 16;
        self.cb(yz as i32, dmh as i32 + 4, "SYNC", 0xFF44886A);
        let pqo = yz + caj + 4;
        
        bgt(self, pqo, dmh, aju, azx, "-", 0x142820, 0x00AA88, 0xFF88CCAA);
        
        let pqp = alloc::format!("{}ms", g.emk);
        let min = pqo + aju + 4;
        let mim = 52u32;
        framebuffer::ih(min, dmh, mim, azx, 0x0A1A12, 180);
        let wwf = min + (mim.ao(pqp.len() as u32 * dt)) / 2;
        self.cb(wwf as i32, dmh as i32 + 5, &pqp, 0xFF88CCAA);
        
        let gtv = min + mim + 4;
        bgt(self, gtv, dmh, aju, azx, "+", 0x142820, 0x00AA88, 0xFF88CCAA);
        
        let jrz = gtv + aju + 4;
        bgt(self, jrz, dmh, aju, azx, "0", 0x1A1A14, 0x888855, 0xFFCCAA66);

        
        let dxn = dmh + azx + 4;
        self.cb(yz as i32, dxn as i32 + 4, "VIZ", 0xFF44886A);
        let pyk = yz + caj + 4;
        
        bgt(self, pyk, dxn, aju, azx, "<", 0x142820, 0x00AA88, 0xFF88CCAA);
        
        let igo = self.visualizer.ev as usize % crate::visualizer::IR_ as usize;
        let mpm = crate::visualizer::OG_[igo];
        let mpn = pyk + aju + 4;
        let gwf = aii.ao(caj + 4 + aju * 2 + 12);
        framebuffer::ih(mpn, dxn, gwf, azx, 0x0A1A12, 180);
        let olx = (gwf / dt) as usize;
        let pyl = if mpm.len() > olx { &mpm[..olx] } else { mpm };
        let xss = mpn + (gwf.ao(pyl.len() as u32 * dt)) / 2;
        self.cb(xss as i32, dxn as i32 + 5, pyl, 0xFF00DDAA);
        
        let jvv = mpn + gwf + 4;
        bgt(self, jvv, dxn, aju, azx, ">", 0x142820, 0x00AA88, 0xFF88CCAA);

        
        let dud = dxn + azx + 4;
        self.cb(yz as i32, dud as i32 + 4, "PAL", 0xFF44886A);
        let otr = yz + caj + 4;
        
        bgt(self, otr, dud, aju, azx, "<", 0x142820, 0x8866CC, 0xFFAA88EE);
        
        let vbe = self.visualizer.aim as usize % crate::visualizer::AGE_ as usize;
        let lru = crate::visualizer::CIM_[vbe];
        let lrv = otr + aju + 4;
        let gos = aii.ao(caj + 4 + aju * 2 + 12);
        framebuffer::ih(lrv, dud, gos, azx, 0x0A1A12, 180);
        let omb = (gos / dt) as usize;
        let ots = if lru.len() > omb { &lru[..omb] } else { lru };
        let vjq = lrv + (gos.ao(ots.len() as u32 * dt)) / 2;
        self.cb(vjq as i32, dud as i32 + 5, ots, 0xFFCC88FF);
        
        let jim = lrv + gos + 4;
        bgt(self, jim, dud, aju, azx, ">", 0x142820, 0x8866CC, 0xFFAA88EE);

        
        let duv = dud + azx + 4;
        self.cb(yz as i32, duv as i32 + 4, "RAIN", 0xFF44886A);
        let ozd = yz + caj + 4;
        
        bgt(self, ozd, duv, aju, azx, "<", 0x142820, 0x00AA88, 0xFF88CCAA);
        
        let vpu = (self.eup as usize).v(2);
        let vpt = ["Slow", "Mid", "Fast"];
        let oze = vpt[vpu];
        let lwx = ozd + aju + 4;
        let hwn = aii.ao(caj + 4 + aju * 2 + 12);
        framebuffer::ih(lwx, duv, hwn, azx, 0x0A1A12, 180);
        let vzr = lwx + (hwn.ao(oze.len() as u32 * dt)) / 2;
        self.cb(vzr as i32, duv as i32 + 5, oze, 0xFF88DDAA);
        
        let jlb = lwx + hwn + 4;
        bgt(self, jlb, duv, aju, azx, ">", 0x142820, 0x00AA88, 0xFF88CCAA);
    }

    
    fn scb(&self, bh: &Window) {
        let cx = bh.b as u32 + 4;
        let ae = bh.c as u32 + J_ + 4;
        let dt = bh.z.ao(8);
        let bm = bh.ac.ao(J_ + 8);
        
        if dt < 100 || bm < 120 {
            return;
        }
        
        
        
        
        
        
        
        let cjy = 72u32;
        
        framebuffer::ah(cx + 6, ae + 6, dt - 12, cjy, 0xFF0D0D1A);
        framebuffer::ih(cx + 6, ae + 6, dt - 12, cjy / 2, 0x1A1A3E, 60);
        
        tf((cx + 6) as i32, (ae + 6) as i32, dt - 12, cjy, 6, P_);
        
        framebuffer::ih(cx + 7, ae + 7, dt - 14, 1, 0x4444AA, 40);
        
        
        let dgj = if let Some(akz) = self.enf.get(&bh.ad) {
            &akz.display
        } else {
            "0"
        };
        
        
        let xfv = dgj.len() as i32;
        let nk = 12; 
        let wg = cx as i32 + dt as i32 - 18 - xfv * nk;
        for (a, bm) in dgj.bw().cf() {
            let y = wg + a as i32 * nk;
            let x = ae as i32 + 28;
            let mut k = [0u8; 4];
            let e = bm.hia(&mut k);
            
            self.cb(y, x, e, 0xFFFFFFFF);
            self.cb(y + 1, x, e, 0xFFFFFFFF);
            self.cb(y, x + 1, e, 0xFFEEEEEE);
        }
        
        
        if let Some(akz) = self.enf.get(&bh.ad) {
            if akz.cle && !akz.xz.is_empty() {
                self.cb(cx as i32 + 14, ae as i32 + 14, "=", I_);
            }
        }
        
        
        let imf = ae + cjy + 16;
        let hbm = 5u32;
        let hbl = 4u32;
        let aib = 6u32;
        let qlu = dt.ao(16);
        let qls = bm.ao(cjy + 28);
        let pm = (qlu - aib * (hbl - 1)) / hbl;
        let qx = ((qls - aib * (hbm - 1)) / hbm).v(52);
        
        let cjk = [
            ["C", "(", ")", "%"],
            ["7", "8", "9", "/"],
            ["4", "5", "6", "*"],
            ["1", "2", "3", "-"],
            ["0", ".", "=", "+"],
        ];
        
        for (br, kfd) in cjk.iter().cf() {
            for (bj, cu) in kfd.iter().cf() {
                let bx = cx + 6 + bj as u32 * (pm + aib);
                let je = imf + br as u32 * (qx + aib);
                
                let lgi = oh!(*cu, "+" | "-" | "*" | "/" | "%" | "=");
                let lfy = *cu == "C" || *cu == "(" || *cu == ")";
                
                
                let (enb, qsh) = if lgi {
                    if *cu == "=" {
                        (X_, I_)
                    } else {
                        (0xFF1A2A22, P_)
                    }
                } else if lfy {
                    (0xFF2A1A28, 0xFF442244)
                } else {
                    (0xFF181C20, 0xFF2A2E34)
                };
                
                
                let ocd = self.lf >= bx as i32 && self.lf < (bx + pm) as i32
                    && self.ot >= je as i32 && self.ot < (je + qx) as i32;
                
                let ei = if ocd {
                    
                    let m = ((enb >> 16) & 0xFF).v(220) + 30;
                    let at = ((enb >> 8) & 0xFF).v(220) + 30;
                    let o = (enb & 0xFF).v(220) + 30;
                    0xFF000000 | (m << 16) | (at << 8) | o
                } else {
                    enb
                };
                
                
                if qx > 8 {
                    framebuffer::ih(bx + 2, je + 2, pm, qx, 0x000000, 30);
                }
                
                
                let nai = 8u32.v(qx / 3);
                mf(bx as i32, je as i32, pm, qx, nai, ei);
                tf(bx as i32, je as i32, pm, qx, nai, 
                    if ocd { I_ } else { qsh });
                
                
                if qx > 12 {
                    framebuffer::ih(bx + 3, je + 1, pm.ao(6), 1, 0xFFFFFF, 15);
                }
                
                
                let zv = cu.len() as u32 * 8;
                let mj = bx + (pm.ao(zv)) / 2;
                let ct = je + (qx / 2).ao(5);
                let agx = if *cu == "=" { 
                    0xFF000000 
                } else if lgi { 
                    I_ 
                } else if lfy {
                    FY_
                } else { 
                    AC_ 
                };
                self.cb(mj as i32, ct as i32, cu, agx);
                
                if lgi || lfy {
                    self.cb(mj as i32 + 1, ct as i32, cu, agx);
                }
            }
        }
    }
    
    
    
    
    const RS_: u32 = 28;
    const JM_: u32 = 38;
    const ALY_: u32 = 20;
    const ZQ_: u32 = 2; 

    
    
    fn nae(&self, bh: &Window)
        -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32)
    {
        let bx = bh.b as u32 + Self::ZQ_;
        let je = bh.c as u32 + J_;
        let nm = bh.z.ao(Self::ZQ_ * 2);
        let adn = bh.ac.ao(J_ + Self::ZQ_);

        let bxl = je;                                 
        let dae = bxl + Self::RS_;    
        let dtj: u32 = 28;                      
        
        let ooy = dtj * 3 + 6 * 3;
        let blp = bx + 8 + ooy + 4;
        let aoe = dae + 4;
        let cno = Self::JM_ - 8;
        let cio = nm.ao(ooy + 20 + 40); 

        let gl = dae + Self::JM_;
        let nd = adn.ao(Self::RS_ + Self::JM_ + Self::ALY_);
        let uo = gl + nd;

        (bx, je, nm, adn, bxl, dae, blp, aoe, cio, cno, gl, nd, uo, dtj)
    }

    
    
    

    fn sgn(&self, bh: &Window) {
        let fx = bh.b;
        let lw = bh.c;
        let hk = bh.z;
        let mg = bh.ac;
        if hk < 200 || mg < 200 { return; }

        let gl = lw + J_ as i32;
        let zb = fx.am(0) as u32;

        let kda      = 0xFF0A120Cu32;
        let kcz     = 0xFF0C180Eu32;
        let kdb   = 0xFF0A120Cu32;
        let kdd    = 0xFF0C140Cu32;
        let kdc  = 0xFF0E1E10u32;
        let hak   = 0xFF0A3818u32;
        let jsy   = 0xFF80CC90u32;
        let ahv      = 0xFF406850u32;
        let psr  = 0xFF50CC70u32;
        let gxo  = I_;
        let mfw   = 0xFF00CC66u32;
        let iar     = 0xFFCCAA00u32;
        let ias   = 0xFFCC4444u32;

        
        framebuffer::ah(zb, gl as u32, hk, mg - J_, kda);

        
        let bbs = 36u32;
        framebuffer::ah(zb, gl as u32, hk, bbs, kcz);
        framebuffer::zs(zb, (gl + bbs as i32) as u32, hk, P_);

        
        let biy = crate::drivers::net::wifi::g();
        let boo = match biy {
            crate::drivers::net::wifi::WifiState::Bni => "No WiFi Hardware",
            crate::drivers::net::wifi::WifiState::Aqx => "WiFi Disabled",
            crate::drivers::net::wifi::WifiState::Lg => "WiFi Disconnected",
            crate::drivers::net::wifi::WifiState::Uj => "Scanning...",
            crate::drivers::net::wifi::WifiState::Aas => "Connecting...",
            crate::drivers::net::wifi::WifiState::Bcf => "Authenticating...",
            crate::drivers::net::wifi::WifiState::Dl => "Connected",
            crate::drivers::net::wifi::WifiState::Kk => "Connection Failed",
        };

        self.en((fx + 12) as i32, (gl + 10) as i32, "WiFi Networks", psr);
        self.en((fx + 13) as i32, (gl + 10) as i32, "WiFi Networks", psr);

        
        let wty = (fx as u32 + hk).ao(boo.len() as u32 * 8 + 16);
        let dch = match biy {
            crate::drivers::net::wifi::WifiState::Dl => mfw,
            crate::drivers::net::wifi::WifiState::Uj => iar,
            crate::drivers::net::wifi::WifiState::Kk => ias,
            _ => ahv,
        };
        self.en(wty as i32, (gl + 10) as i32, boo, dch);

        
        let mut ou = gl + bbs as i32 + 2;
        if let Some(bfk) = crate::drivers::net::wifi::cwo() {
            let gzo = 40u32;
            framebuffer::ih(zb + 4, ou as u32, hk - 8, gzo, 0x003310, 180);
            tf((fx + 4) as i32, ou, hk - 8, gzo, 6, gxo);

            self.en((fx + 14) as i32, ou + 6, ">>", mfw);
            self.en((fx + 34) as i32, ou + 6, &format!("Connected: {}", bfk), gxo);

            if let Some(sig) = crate::drivers::net::wifi::jqh() {
                let woe = format!("{} dBm", sig);
                self.en((fx + 34) as i32, ou + 22, &woe, ahv);
            }

            
            let gev = (fx as u32 + hk).ao(100);
            mf(gev as i32, ou + 8, 80, 24, 6, 0xFF331111);
            tf(gev as i32, ou + 8, 80, 24, 6, ias);
            self.en((gev + 8) as i32, ou + 14, "Disconnect", ias);

            ou += gzo as i32 + 4;
        }

        
        let ftq = 80u32;
        let grj = (fx as u32 + hk).ao(ftq + 8);
        let mcf = ou;
        let gki = biy == crate::drivers::net::wifi::WifiState::Uj;
        let wdo = if gki { 0xFF0A2A10u32 } else { 0xFF0C1C0Cu32 };
        mf(grj as i32, mcf, ftq, 26, 6, wdo);
        tf(grj as i32, mcf, ftq, 26, 6, if gki { iar } else { gxo });
        let pgf = if gki { "Scanning" } else { "Scan" };
        let wdx = grj + (ftq - pgf.len() as u32 * 8) / 2;
        self.en(wdx as i32, mcf + 7, pgf, if gki { iar } else { gxo });

        self.en((fx + 12) as i32, ou + 6, "Available Networks", ahv);
        ou += 30;

        framebuffer::zs(zb + 4, ou as u32, hk - 8, P_);
        ou += 2;

        
        let hso = crate::drivers::net::wifi::nym();
        let ph = 44i32;
        let bpd = ((mg as i32 - (ou - lw) - 8) / ph).am(1) as usize;

        if hso.is_empty() && !gki {
            self.en((fx + 20) as i32, ou + 20, "No networks found. Click Scan to search.", ahv);
        } else if hso.is_empty() && gki {
            let hgp = match (self.oo / 15) % 4 {
                0 => ".",
                1 => "..",
                2 => "...",
                _ => "",
            };
            self.en((fx + 20) as i32, ou + 20, &format!("Scanning for networks{}", hgp), iar);
        }

        for (a, net) in hso.iter().cf().chz(self.gws).take(bpd) {
            let afy = ou + ((a - self.gws) as i32 * ph);
            let qe = a == self.jwu;
            let jbk = self.lf >= fx && self.lf < fx + hk as i32
                && self.ot >= afy && self.ot < afy + ph;

            let hyc = if qe { hak }
                else if jbk { kdc }
                else if a % 2 == 0 { kdb }
                else { kdd };
            framebuffer::ah(zb + 4, afy as u32, hk - 8, ph as u32, hyc);

            
            let cjf = net.wob();
            let ajx = fx + 12;
            for o in 0..4u32 {
                let mxw = 4 + o * 4;
                let qmy = afy + 30;
                let emn = if o < cjf as u32 {
                    if cjf >= 3 { mfw } else if cjf >= 2 { iar } else { ias }
                } else {
                    0xFF1A2A1Au32
                };
                framebuffer::ah(
                    (ajx + o as i32 * 6) as u32,
                    (qmy - mxw as i32) as u32,
                    4,
                    mxw,
                    emn,
                );
            }

            
            let pnb = fx + 42;
            let wrx = if qe { gxo } else { jsy };
            let wry = if net.bfk.is_empty() { "(Hidden Network)" } else { &net.bfk };
            self.en(pnb, afy + 8, wry, wrx);

            
            let ldz = format!("{} | Ch {} | {} MHz | {} dBm",
                net.security.as_str(), net.channel, net.sxk, net.dlv);
            self.en(pnb, afy + 24, &ldz, ahv);

            
            if net.security != crate::drivers::net::wifi::WifiSecurity::Ck {
                let cli = (fx as u32 + hk).ao(30);
                framebuffer::lx(cli + 2, (afy + 8) as u32, 8, 6, ahv);
                framebuffer::ah(cli, (afy + 14) as u32, 12, 10, ahv);
            }

            framebuffer::zs(zb + 8, (afy + ph - 1) as u32, hk - 16, 0xFF142014);
        }

        
        if let Some(ref fr) = self.fbj {
            let ita = (lw as u32 + mg).ao(30);
            framebuffer::ih(zb + 4, ita, hk - 8, 24, 0x331111, 200);
            self.en((fx + 12) as i32, (ita + 6) as i32, fr, ias);
        }
    }

    
    
    

    fn sgo(&self, bh: &Window) {
        let fx = bh.b;
        let lw = bh.c;
        let hk = bh.z;
        let mg = bh.ac;
        if hk < 200 || mg < 150 { return; }

        let gl = lw + J_ as i32;
        let zb = fx.am(0) as u32;

        let kda     = 0xFF0A120Cu32;
        let jsy = 0xFF80CC90u32;
        let ahv    = 0xFF406850u32;
        let mm      = I_;
        let tuz    = 0xFF060C06u32;
        let qus   = 0xFF0A3818u32;
        let quq  = 0xFF1A0A0Au32;

        
        framebuffer::ah(zb, gl as u32, hk, mg - J_, kda);

        
        let ese = fx as u32 + hk / 2;
        let esf = gl as u32 + 30;
        for kau in 0..4u32 {
            let m = 6 + kau * 6;
            for gyp in 0..16u32 {
                let dx = (gyp * m) / 16;
                let hhm = (m * m).ao(dx * dx);
                let mut bg = 0u32;
                while (bg + 1) * (bg + 1) <= hhm { bg += 1; }
                framebuffer::ii(ese + dx, esf.ao(bg), mm);
                if ese >= dx {
                    framebuffer::ii(ese - dx, esf.ao(bg), mm);
                }
            }
        }
        framebuffer::ah(ese - 2, esf, 5, 5, mm);

        
        let pna = format!("Connect to: {}", self.ihf);
        let dis = (fx as u32 + (hk - pna.len() as u32 * 8) / 2) as i32;
        self.en(dis, gl + 60, &pna, jsy);

        
        self.en(fx + 20, gl + 90, "Password:", ahv);

        
        let alf = (gl + 108) as u32;
        let ess = hk - 32;
        let hob = 32u32;
        mf((fx + 16) as i32, alf as i32, ess, hob, 6, tuz);
        tf((fx + 16) as i32, alf as i32, ess, hob, 6, P_);

        
        let dgj = if self.fyt {
            self.ddl.clone()
        } else {
            "*".afd(self.ddl.len())
        };
        let wg = fx + 24;
        let sl = alf as i32 + 9;
        if dgj.is_empty() {
            self.en(wg, sl, "Enter password...", P_);
        } else {
            let aem = ((ess as usize).ao(40)) / 8;
            let iw: String = dgj.bw().vv().take(aem).collect::<String>().bw().vv().collect();
            self.en(wg, sl, &iw, jsy);
        }

        
        if self.btx {
            let aem = ((ess as usize).ao(40)) / 8;
            let lf = wg + dgj.len().v(aem) as i32 * 8;
            framebuffer::ah(lf as u32, (sl - 1) as u32, 2, 14, mm);
        }

        
        let faa = alf as i32 + hob as i32 + 8;
        let qyo = if self.fyt { mm } else { P_ };
        framebuffer::lx((fx + 20) as u32, faa as u32, 14, 14, qyo);
        if self.fyt {
            framebuffer::ah((fx + 23) as u32, (faa + 7) as u32, 3, 3, mm);
            framebuffer::ah((fx + 26) as u32, (faa + 5) as u32, 3, 3, mm);
            framebuffer::ah((fx + 29) as u32, (faa + 3) as u32, 3, 3, mm);
        }
        self.en(fx + 40, faa + 2, "Show password", ahv);

        
        let cwb = (lw as u32 + mg).ao(50);
        let bym = 100u32;
        let doq = 32u32;
        let hbu = 16u32;
        let mmc = bym * 2 + hbu;
        let end = (fx as u32 + (hk - mmc) / 2) as i32;

        
        mf(end, cwb as i32, bym, doq, 8, qus);
        tf(end, cwb as i32, bym, doq, 8, mm);
        let rny = end + (bym as i32 - 56) / 2;
        self.en(rny, cwb as i32 + 9, "Connect", mm);

        
        let gcd = end + bym as i32 + hbu as i32;
        mf(gcd, cwb as i32, bym, doq, 8, quq);
        tf(gcd, cwb as i32, bym, doq, 8, 0xFFCC4444);
        let qvx = gcd + (bym as i32 - 48) / 2;
        self.en(qvx, cwb as i32 + 9, "Cancel", 0xFFCC4444);

        
        if let Some(ref fr) = self.fbj {
            let ita = cwb - 24;
            let snh = (fx as u32 + (hk - fr.len() as u32 * 8) / 2) as i32;
            self.en(snh, ita as i32, fr, 0xFFCC4444);
        }
    }

    
    fn sby(&self, bh: &Window) {
        let (bx, qbk, nm, adn, bxl, dae,
             blp, aoe, cio, cno,
             gl, nd, uo, dtj)
            = self.nae(bh);

        if nm < 120 || adn < 100 { return; }

        let dt = crate::graphics::scaling::bmi() as i32;
        let bm = crate::graphics::scaling::fep();

        
        framebuffer::ah(bx, bxl, nm, Self::RS_, 0xFF202124);
        
        let guc = bx + 8;
        let axb: u32 = 200.v(nm.ao(60));
        let dwo = Self::RS_ - 4;
        
        framebuffer::ah(guc + 2, bxl + 4, axb - 4, dwo, 0xFF35363A);
        framebuffer::ah(guc, bxl + 6, 2, dwo - 2, 0xFF35363A);
        framebuffer::ah(guc + axb - 2, bxl + 6, 2, dwo - 2, 0xFF35363A);
        
        let mjq = if let Some(ref browser) = self.browser {
            if let Some(ref doc) = browser.ama {
                if doc.dq.is_empty() { alloc::string::String::from("New Tab") } else { doc.dq.clone() }
            } else { alloc::string::String::from("New Tab") }
        } else { alloc::string::String::from("New Tab") };
        let ryq: alloc::string::String = if mjq.len() > 22 {
            let e: alloc::string::String = mjq.bw().take(20).collect();
            alloc::format!("{}...", e)
        } else { mjq };
        self.cb(guc as i32 + 10, (bxl + 8) as i32, &ryq, 0xFFE8EAED);
        
        self.cb((guc + axb - 18) as i32, (bxl + 8) as i32, "x", 0xFF999999);
        
        let owe = guc + axb + 6;
        framebuffer::ah(owe, bxl + 6, 24, dwo, 0xFF2A2A2E);
        self.cb(owe as i32 + 8, (bxl + 8) as i32, "+", 0xFF999999);

        
        framebuffer::ah(bx, dae, nm, Self::JM_, 0xFF35363A);
        
        framebuffer::ah(bx, dae, nm, 1, 0xFF4A4A4E);

        
        let dze = dae + Self::JM_ / 2; 
        let cjj = dtj / 2;
        let mut hbv = bx + 12u32;
        
        let kre = |cx: u32, ae: u32, m: u32, oce: u32| {
            
            let flp = m / 3;
            framebuffer::ah(cx - m + flp, ae - m, (m - flp) * 2, m * 2, oce);
            framebuffer::ah(cx - m, ae - m + flp, m * 2, (m - flp) * 2, oce);
        };
        
        let mxi = hbv + cjj;
        kre(mxi, dze, cjj, 0xFF4A4A4E);
        self.cb((mxi - 4) as i32, (dze - 6) as i32, "<", 0xFFE8EAED);
        hbv += dtj + 6;
        
        let nww = hbv + cjj;
        kre(nww, dze, cjj, 0xFF4A4A4E);
        self.cb((nww - 4) as i32, (dze - 6) as i32, ">", 0xFFE8EAED);
        hbv += dtj + 6;
        
        let lyk = hbv + cjj;
        kre(lyk, dze, cjj, 0xFF4A4A4E);
        if self.btn {
            self.cb((lyk - 4) as i32, (dze - 6) as i32, "X", 0xFFE8EAED);
        } else {
            self.cb((lyk - 4) as i32, (dze - 6) as i32, "R", 0xFFE8EAED);
        }

        
        
        let blo = cno / 2; 
        framebuffer::ah(blp + blo, aoe, cio.ao(blo * 2), cno, 0xFF202124);
        
        framebuffer::ah(blp, aoe + blo / 2, blo, cno - blo, 0xFF202124);
        framebuffer::ah(blp + 1, aoe + blo / 4, blo - 1, blo / 2, 0xFF202124);
        
        framebuffer::ah(blp + cio - blo, aoe + blo / 2, blo, cno - blo, 0xFF202124);
        framebuffer::ah(blp + cio - blo, aoe + blo / 4, blo - 1, blo / 2, 0xFF202124);
        
        if bh.ja {
            framebuffer::ah(blp + blo, aoe, cio.ao(blo * 2), 1, 0xFF8AB4F8);
            framebuffer::ah(blp + blo, aoe + cno - 1, cio.ao(blo * 2), 1, 0xFF8AB4F8);
        }

        
        let bel = blp as i32 + 8;
        let sl = aoe as i32 + (cno as i32 - bm as i32) / 2;
        let tmq = self.ado.cj("https://");
        if tmq {
            self.cb(bel, sl, "S", 0xFF81C995);
        } else {
            
            self.cb(bel + 1, sl, "i", 0xFF999999);
        }
        
        framebuffer::ah((blp + 22) as u32, aoe + 5, 1, cno - 10, 0xFF3C3C3C);

        
        let gvr = blp as i32 + 28;
        let pxn = if self.ado.is_empty() {
            "Search or enter URL"
        } else {
            &self.ado
        };
        let agx = if self.ado.is_empty() { 0xFF9AA0A6 } else { 0xFFE8EAED };

        
        let xfr = (cio as i32).ao(42);
        let ayf = if dt > 0 { (xfr / dt).am(1) as usize } else { 40 };
        let pxm = pxn.len();
        let jnx = if self.aef > ayf {
            self.aef - ayf + 1
        } else { 0 };
        let xry = (jnx + ayf).v(pxm);
        let ign = if jnx < pxm { &pxn[jnx..xry] } else { "" };

        if self.btn {
            self.cb(gvr, sl, "Loading...", 0xFF8AB4F8);
        } else {
            self.cb(gvr, sl, ign, agx);
        }

        
        if !self.btn && self.cdj && !self.ado.is_empty() {
            let php = (ign.len() as u32) * dt as u32;
            if php > 0 {
                framebuffer::ah(gvr as u32, aoe + 3, php.v(cio - 34), cno - 6, 0xFF3574E0);
                
                self.cb(gvr, sl, ign, 0xFFFFFFFF);
            }
        }

        
        if !self.btn && bh.ja {
            if self.btx {
                let rlo = self.aef.ao(jnx);
                let cx = gvr + (rlo as i32) * dt;
                if cx >= gvr && cx < (blp + cio - 8) as i32 {
                    framebuffer::ah(cx as u32, aoe + 4, 2, cno - 8, 0xFF8AB4F8);
                }
            }
        }

        
        let rs = blp + cio + 6;
        let xp = dae + 8;
        let gfe: u32 = 3;
        let tyq = self.browser.as_ref().map(|o| o.dca).unwrap_or(false);
        let llp = if tyq { 0xFF8AB4F8 } else { 0xFF999999 };
        framebuffer::ah(rs + 4, xp + 2, gfe, gfe, llp);
        framebuffer::ah(rs + 4, xp + 8, gfe, gfe, llp);
        framebuffer::ah(rs + 4, xp + 14, gfe, gfe, llp);

        
        if let Some(ref browser) = self.browser {
            if browser.dca && !browser.bfc.is_empty() {
                framebuffer::ah(bx, gl, nm, nd, 0xFF1E1E1E);
                self.sez(bx as i32, gl as i32, nm, nd, &browser.bfc, browser.ug);
            } else if let Some(ref doc) = browser.ama {
                crate::browser::vvy(doc, bx as i32, gl as i32, nm, nd, browser.ug);
            } else {
                self.nmu(bx, gl, nm, nd);
            }
        } else {
            self.nmu(bx, gl, nm, nd);
        }

        
        framebuffer::ah(bx, uo, nm, Self::ALY_, 0xFF202124);
        let fvq = if let Some(ref browser) = self.browser {
            match &browser.status {
                crate::browser::BrowserStatus::Cv => alloc::string::String::from("Ready"),
                crate::browser::BrowserStatus::Py => alloc::string::String::from("Loading..."),
                crate::browser::BrowserStatus::At => {
                    if !browser.bhw.is_empty() {
                        alloc::format!("Done  ({} resources)", browser.bhw.len())
                    } else {
                        alloc::string::String::from("Done")
                    }
                },
                crate::browser::BrowserStatus::Q(aa) => aa.clone(),
            }
        } else { alloc::string::String::from("Ready") };
        self.cb(bx as i32 + 8, uo as i32 + 3, &fvq, 0xFF9AA0A6);
    }

    
    fn nmu(&self, bx: u32, ae: u32, nm: u32, bm: u32) {
        
        framebuffer::ah(bx, ae, nm, bm, 0xFFFFFFFF);

        let cgd = bx as i32 + nm as i32 / 2;
        let bkl = ae as i32 + bm as i32 / 2 - 50;

        
        let dq = "TrustBrowser";
        let jsr = crate::graphics::scaling::bmi() as i32;
        let gx = cgd - (dq.len() as i32 * jsr) / 2;
        self.cb(gx, bkl, dq, 0xFF202124);

        
        let dom: u32 = 360.v(nm.ao(40));
        let del: u32 = 34;
        let btm = (cgd - dom as i32 / 2).am(bx as i32 + 4) as u32;
        let bjk = (bkl + 30) as u32;
        framebuffer::ah(btm + 4, bjk, dom - 8, del, 0xFFF1F3F4);
        framebuffer::ah(btm, bjk + 4, 4, del - 8, 0xFFF1F3F4);
        framebuffer::ah(btm + dom - 4, bjk + 4, 4, del - 8, 0xFFF1F3F4);
        
        framebuffer::ah(btm + 4, bjk, dom - 8, 1, 0xFFDFE1E5);
        framebuffer::ah(btm + 4, bjk + del - 1, dom - 8, 1, 0xFFDFE1E5);
        
        self.cb(btm as i32 + 14, bjk as i32 + 9, "Search or type a URL", 0xFF9AA0A6);

        
        let eue = bjk as i32 + del as i32 + 24;
        let liu = ["example.com", "10.0.2.2", "google.com"];
        let glm: i32 = 100;
        let aza = liu.len() as i32 * glm + (liu.len() as i32 - 1) * 12;
        let mut mj = cgd - aza / 2;
        for cu in &liu {
            
            framebuffer::ah(mj as u32, eue as u32, glm as u32, 28, 0xFFF1F3F4);
            framebuffer::ah(mj as u32, eue as u32, glm as u32, 1, 0xFFDFE1E5);
            framebuffer::ah(mj as u32, (eue + 27) as u32, glm as u32, 1, 0xFFDFE1E5);
            let qd = cu.len() as i32 * jsr;
            self.cb(mj + (glm - qd) / 2, eue + 7, cu, 0xFF1A73E8);
            mj += glm + 12;
        }
    }
    
    
    fn sez(&self, b: i32, c: i32, z: u32, ac: u32, brb: &str, ug: i32) {
        let dt = crate::graphics::scaling::bmi() as i32;
        let acg = crate::graphics::scaling::fep() as i32 + 2;
        let aem = if dt > 0 { (z as usize).ao(56) / dt as usize } else { 60 };

        let mut hgz = c + 8 - ug;
        let csl = c + ac as i32 - 8;
        let mut csd = 1;

        for line in brb.ak() {
            if hgz > csl { break; }
            if hgz >= c - acg {
                let lis = alloc::format!("{:4} ", csd);
                self.cb(b + 4, hgz, &lis, 0xFF6E7681);
                let ryo: alloc::string::String = if line.len() > aem {
                    let ab: alloc::string::String = line.bw().take(aem.ao(3)).collect();
                    alloc::format!("{}...", ab)
                } else { alloc::string::String::from(line) };
                self.sfw(b + 5 * dt + 8, hgz, &ryo);
            }
            hgz += acg;
            csd += 1;
        }
    }

    
    fn sfw(&self, b: i32, c: i32, line: &str) {
        let dt = crate::graphics::scaling::bmi() as i32;
        let mut aua = b;
        let mut izu = false;
        let mut cyv = false;
        let mut ldq = false;
        let mut ppa = '"';

        let bw: alloc::vec::Vec<char> = line.bw().collect();
        let mut a = 0;
        while a < bw.len() {
            let r = bw[a];
            let s = if cyv {
                0xFFCE9178
            } else if r == '<' || r == '>' || r == '/' {
                izu = r == '<';
                if r == '>' { ldq = false; }
                0xFF569CD6
            } else if izu && r == '=' {
                ldq = true;
                0xFF9CDCFE
            } else if izu && (r == '"' || r == '\'') {
                cyv = true;
                ppa = r;
                0xFFCE9178
            } else if ldq && !r.fme() {
                0xFF4EC9B0
            } else if izu && !r.fme() && r != '=' {
                0xFF569CD6
            } else {
                0xFFD4D4D4
            };
            if cyv && a > 0 && r == ppa && bw[a-1] != '\\' {
                cyv = false;
            }
            let e = alloc::format!("{}", r);
            self.cb(aua, c, &e, s);
            aua += dt;
            a += 1;
        }
    }
    
    
    fn tja(&mut self, b: i32, c: i32, abx: i32, aha: i32, aog: u32) {
        crate::serial_println!("[BROWSER-DBG] handle_browser_click x={} y={} win_x={} win_y={} win_w={}",
            b, c, abx, aha, aog);
        
        
        let xif = Window {
            ad: 0, dq: String::new(),
            b: abx, c: aha,
            z: aog,
            ac: self.ee.iter()
                .du(|d| d.b == abx && d.c == aha && d.z == aog)
                .map(|d| d.ac).unwrap_or(500),
            czx: 0, dtg: 0,
            iw: true, ja: true, aat: false, bkk: false,
            cka: false, dlg: ResizeEdge::None,
            dgp: 0, dgq: 0,
            exy: 0, exz: 0, gri: 0, grh: 0,
            ld: WindowType::Browser,
            ca: Vec::new(), wn: None,
            acm: 0, px: 0,
            cvp: WindowAnimation::new(), egj: false,
        };
        let (qbj, qbk, nm, xyb, ydh, dae,
             blp, aoe, cio, cno,
             xyn, xym, ydb, dtj)
            = self.nae(&xif);

        crate::serial_println!("[BROWSER-DBG] layout: url_bar=({},{} {}x{}) nav_y={} bw={} click=({},{})",
            blp, aoe, cio, cno, dae, nm, b, c);

        if nm < 120 { return; }

        let cx = b as u32;
        let ae = c as u32;

        
        let cjj = dtj / 2;
        let dze = dae + Self::JM_ / 2;
        let mut axp = qbj + 12 + cjj;
        
        let lcf = |qut: u32| -> bool {
            let dx = (cx as i32 - qut as i32).eki();
            let bg = (ae as i32 - dze as i32).eki();
            dx <= cjj && bg <= cjj
        };
        
        if lcf(axp) {
            crate::serial_println!("[BROWSER] Back button clicked");
            if let Some(ref mut browser) = self.browser { let _ = browser.qmf(); }
            return;
        }
        axp += dtj + 6;
        
        if lcf(axp) {
            crate::serial_println!("[BROWSER] Forward button clicked");
            if let Some(ref mut browser) = self.browser { let _ = browser.fiz(); }
            return;
        }
        axp += dtj + 6;
        
        if lcf(axp) {
            crate::serial_println!("[BROWSER] Refresh button clicked");
            if let Some(ref mut browser) = self.browser { let _ = browser.gqr(); }
            return;
        }

        
        if cx >= blp && cx < blp + cio
            && ae >= aoe && ae < aoe + cno
        {
            
            if crate::mouse::jbf() {
                crate::mouse::pcp();
                self.cdj = true;
                self.aef = self.ado.len();
                crate::serial_println!("[BROWSER] URL bar double-clicked, select all");
                return;
            }
            
            self.cdj = false;
            let dt = crate::graphics::scaling::bmi();
            if dt > 0 {
                let xgb = blp + 26;
                let amr = cx.ao(xgb);
                let hcr = (amr / dt) as usize;
                self.aef = hcr.v(self.ado.len());
                crate::serial_println!("[BROWSER] URL bar clicked, cursor={}", self.aef);
            }
            return;
        }

        
        let rs = blp + cio + 6;
        let xp = dae + 8;
        if cx >= rs && cx < rs + 16 && ae >= xp && ae < xp + 22 {
            crate::serial_println!("[BROWSER] Menu (view toggle) clicked");
            if let Some(ref mut browser) = self.browser {
                browser.xja();
            }
            return;
        }
    }
    
    fn dqf(&self) {
        
        let mut eag = CursorMode::Ov;
        
        
        for d in self.ee.iter().vv() {
            if d.aat || d.bkk { continue; }
            let amd = d.lqj(self.lf, self.ot);
            match amd {
                ResizeEdge::Ap | ResizeEdge::Ca => { eag = CursorMode::Axz; break; },
                ResizeEdge::Jd | ResizeEdge::Hk => { eag = CursorMode::Ayb; break; },
                ResizeEdge::Dp | ResizeEdge::Du => { eag = CursorMode::Aeh; break; },
                ResizeEdge::Dq | ResizeEdge::Dt => { eag = CursorMode::Aeg; break; },
                _ => {},
            }
            
            if self.lf >= d.b && self.lf < d.b + d.z as i32
                && self.ot >= d.c && self.ot < d.c + d.ac as i32 {
                break;
            }
        }
        
        
        for d in &self.ee {
            match d.dlg {
                ResizeEdge::Ap | ResizeEdge::Ca => { eag = CursorMode::Axz; break; },
                ResizeEdge::Jd | ResizeEdge::Hk => { eag = CursorMode::Ayb; break; },
                ResizeEdge::Dp | ResizeEdge::Du => { eag = CursorMode::Aeh; break; },
                ResizeEdge::Dq | ResizeEdge::Dt => { eag = CursorMode::Aeg; break; },
                _ => {},
            }
        }
        
        match eag {
            CursorMode::Ov | CursorMode::Cep => self.sbs(),
            CursorMode::Axz => self.sfb(),
            CursorMode::Ayb => self.sfe(),
            CursorMode::Aeh => self.sfd(),
            CursorMode::Aeg => self.sfc(),
        }
    }
    
    
    fn sbs(&self) {
        let aap = crate::accessibility::gib().bv();
        let bei = crate::accessibility::edv();
        
        
        let dls = 0x40000000u32;
        for l in 1..=(2 * aap as i32) {
            let cr = self.lf + l;
            let cq = self.ot + l;
            if cr >= 0 && cq >= 0 && cr < self.z as i32 && cq < self.ac as i32 {
                for bg in 0..(12 * aap as i32) {
                    let x = (cq + bg) as u32;
                    let y = cr as u32;
                    if x < self.ac && y < self.z {
                        framebuffer::ii(y, x, dls);
                    }
                }
            }
        }
        
        
        let dua = if bei { 0xFF000000u32 } else { X_ };
        let ebo = if bei { 0xFFFFFFFFu32 } else { AG_ };
        
        
        let gi: [[u8; 12]; 16] = [
            [1,0,0,0,0,0,0,0,0,0,0,0],
            [1,1,0,0,0,0,0,0,0,0,0,0],
            [1,2,1,0,0,0,0,0,0,0,0,0],
            [1,2,2,1,0,0,0,0,0,0,0,0],
            [1,2,2,2,1,0,0,0,0,0,0,0],
            [1,2,2,2,2,1,0,0,0,0,0,0],
            [1,2,2,2,2,2,1,0,0,0,0,0],
            [1,2,2,2,2,2,2,1,0,0,0,0],
            [1,2,2,2,2,2,2,2,1,0,0,0],
            [1,2,2,2,2,2,2,2,2,1,0,0],
            [1,2,2,2,2,2,1,1,1,1,1,0],
            [1,2,2,1,2,2,1,0,0,0,0,0],
            [1,2,1,0,1,2,2,1,0,0,0,0],
            [1,1,0,0,1,2,2,1,0,0,0,0],
            [1,0,0,0,0,1,2,2,1,0,0,0],
            [0,0,0,0,0,1,1,1,1,0,0,0],
        ];
        
        for (ae, br) in gi.iter().cf() {
            for (cx, &il) in br.iter().cf() {
                if il == 0 { continue; }
                let s = match il {
                    1 => dua,
                    2 => ebo,
                    _ => continue,
                };
                
                for cq in 0..aap {
                    for cr in 0..aap {
                        let y = (self.lf + cx as i32 * aap as i32 + cr as i32) as u32;
                        let x = (self.ot + ae as i32 * aap as i32 + cq as i32) as u32;
                        if y < self.z && x < self.ac {
                            framebuffer::ii(y, x, s);
                        }
                    }
                }
            }
        }
    }
    
    
    fn sfb(&self) {
        let hl = self.lf;
        let ir = self.ot;
        
        
        for a in 0..7i32 {
            let y = (hl - 7 + a) as u32;
            let x = ir as u32;
            if y < self.z && x < self.ac {
                framebuffer::ii(y, x, I_);
                if x > 0 { framebuffer::ii(y, x - 1, X_); }
                if x + 1 < self.ac { framebuffer::ii(y, x + 1, X_); }
            }
        }
        
        for a in 0..7i32 {
            let y = (hl + 1 + a) as u32;
            let x = ir as u32;
            if y < self.z && x < self.ac {
                framebuffer::ii(y, x, I_);
                if x > 0 { framebuffer::ii(y, x - 1, X_); }
                if x + 1 < self.ac { framebuffer::ii(y, x + 1, X_); }
            }
        }
        
        for bc in 1..=4i32 {
            let y = (hl - 7 + bc) as u32;
            if y < self.z {
                if (ir - bc) >= 0 { framebuffer::ii(y, (ir - bc) as u32, I_); }
                if (ir + bc) < self.ac as i32 { framebuffer::ii(y, (ir + bc) as u32, I_); }
            }
        }
        
        for bc in 1..=4i32 {
            let y = (hl + 7 - bc) as u32;
            if y < self.z {
                if (ir - bc) >= 0 { framebuffer::ii(y, (ir - bc) as u32, I_); }
                if (ir + bc) < self.ac as i32 { framebuffer::ii(y, (ir + bc) as u32, I_); }
            }
        }
        
        if hl >= 0 && ir >= 0 && (hl as u32) < self.z && (ir as u32) < self.ac {
            framebuffer::ii(hl as u32, ir as u32, 0xFFFFFFFF);
        }
    }
    
    
    fn sfe(&self) {
        let hl = self.lf;
        let ir = self.ot;
        
        for a in 0..7i32 {
            let y = hl as u32;
            let x = (ir - 7 + a) as u32;
            if y < self.z && x < self.ac {
                framebuffer::ii(y, x, I_);
                if y > 0 { framebuffer::ii(y - 1, x, X_); }
                if y + 1 < self.z { framebuffer::ii(y + 1, x, X_); }
            }
        }
        for a in 0..7i32 {
            let y = hl as u32;
            let x = (ir + 1 + a) as u32;
            if y < self.z && x < self.ac {
                framebuffer::ii(y, x, I_);
                if y > 0 { framebuffer::ii(y - 1, x, X_); }
                if y + 1 < self.z { framebuffer::ii(y + 1, x, X_); }
            }
        }
        
        for bc in 1..=4i32 {
            let x = (ir - 7 + bc) as u32;
            if x < self.ac {
                if (hl - bc) >= 0 { framebuffer::ii((hl - bc) as u32, x, I_); }
                if (hl + bc) < self.z as i32 { framebuffer::ii((hl + bc) as u32, x, I_); }
            }
        }
        
        for bc in 1..=4i32 {
            let x = (ir + 7 - bc) as u32;
            if x < self.ac {
                if (hl - bc) >= 0 { framebuffer::ii((hl - bc) as u32, x, I_); }
                if (hl + bc) < self.z as i32 { framebuffer::ii((hl + bc) as u32, x, I_); }
            }
        }
        if hl >= 0 && ir >= 0 && (hl as u32) < self.z && (ir as u32) < self.ac {
            framebuffer::ii(hl as u32, ir as u32, 0xFFFFFFFF);
        }
    }
    
    
    fn sfd(&self) {
        let hl = self.lf;
        let ir = self.ot;
        
        for a in -6..=6i32 {
            let y = (hl + a) as u32;
            let x = (ir + a) as u32;
            if y < self.z && x < self.ac {
                framebuffer::ii(y, x, I_);
                if y + 1 < self.z { framebuffer::ii(y + 1, x, X_); }
                if x + 1 < self.ac { framebuffer::ii(y, x + 1, X_); }
            }
        }
        
        for bc in 1..=3i32 {
            let bx = hl - 6 + bc;
            let je = ir - 6;
            if bx >= 0 && (je as u32) < self.ac { framebuffer::ii(bx as u32, je as u32, I_); }
            let dep = hl - 6;
            let deq = ir - 6 + bc;
            if dep >= 0 && deq >= 0 { framebuffer::ii(dep as u32, deq as u32, I_); }
        }
        
        for bc in 1..=3i32 {
            let bx = hl + 6 - bc;
            let je = ir + 6;
            if (bx as u32) < self.z && (je as u32) < self.ac { framebuffer::ii(bx as u32, je as u32, I_); }
            let dep = hl + 6;
            let deq = ir + 6 - bc;
            if (dep as u32) < self.z && (deq as u32) < self.ac { framebuffer::ii(dep as u32, deq as u32, I_); }
        }
    }
    
    
    fn sfc(&self) {
        let hl = self.lf;
        let ir = self.ot;
        
        for a in -6..=6i32 {
            let y = (hl + a) as u32;
            let x = (ir - a) as u32;
            if y < self.z && x < self.ac {
                framebuffer::ii(y, x, I_);
                if y > 0 { framebuffer::ii(y - 1, x, X_); }
                if x + 1 < self.ac { framebuffer::ii(y, x + 1, X_); }
            }
        }
        
        for bc in 1..=3i32 {
            let bx = hl + 6 - bc;
            let je = ir - 6;
            if (bx as u32) < self.z && je >= 0 { framebuffer::ii(bx as u32, je as u32, I_); }
            let dep = hl + 6;
            let deq = ir - 6 + bc;
            if (dep as u32) < self.z && deq >= 0 { framebuffer::ii(dep as u32, deq as u32, I_); }
        }
        
        for bc in 1..=3i32 {
            let bx = hl - 6 + bc;
            let je = ir + 6;
            if bx >= 0 && (je as u32) < self.ac { framebuffer::ii(bx as u32, je as u32, I_); }
            let dep = hl - 6;
            let deq = ir + 6 - bc;
            if dep >= 0 && (deq as u32) < self.ac { framebuffer::ii(dep as u32, deq as u32, I_); }
        }
    }
    
    fn cb(&self, b: i32, c: i32, text: &str, s: u32) {
        
        let lpy = framebuffer::hlh();
        framebuffer::dbv(s);
        
        let dt = crate::graphics::scaling::bmi() as i32;
        for (a, r) in text.bw().cf() {
            let y = b + (a as i32 * dt);
            if y >= 0 && y < self.z as i32 && c >= 0 && c < self.ac as i32 {
                crate::graphics::scaling::krh(y as u32, c as u32, r, s);
            }
        }
        
        framebuffer::dbv(lpy);
    }
    
    fn ahi(&self, b: u32, c: u32, r: char, s: u32) {
        
        crate::graphics::scaling::krh(b, c, r, s);
    }
    
    
    fn en(&self, b: i32, c: i32, text: &str, s: u32) {
        let dt = crate::graphics::scaling::bmi() as i32;
        let pv = crate::graphics::scaling::ckv();
        let qbm = 16u32 * pv;
        let xzm = 8u32 * pv;
        let gz = self.z;
        let kc = self.ac;
        
        let ebm = ((s >> 16) & 0xFF) as u32;
        let ebl = ((s >> 8) & 0xFF) as u32;
        let ebk = (s & 0xFF) as u32;
        
        for (a, r) in text.bw().cf() {
            let cx = b + (a as i32 * dt);
            if cx < 0 || cx >= gz as i32 || c < 0 || c >= kc as i32 { continue; }
            
            let ka = framebuffer::font::ada(r);
            
            for br in 0..16u32 {
                let fs = ka[br as usize];
                let vo = if br > 0 { ka[br as usize - 1] } else { 0u8 };
                let next = if br < 15 { ka[br as usize + 1] } else { 0u8 };
                
                for bj in 0..8u32 {
                    let hs = 0x80u8 >> bj;
                    let lgm = fs & hs != 0;
                    
                    if lgm {
                        
                        for cq in 0..pv {
                            for cr in 0..pv {
                                let y = cx as u32 + bj * pv + cr;
                                let x = c as u32 + br * pv + cq;
                                if y < gz && x < kc {
                                    framebuffer::ii(y, x, s);
                                }
                            }
                        }
                    } else {
                        
                        let fd  = bj > 0 && (fs & (hs << 1)) != 0;
                        let hw = bj < 7 && (fs & (hs >> 1)) != 0;
                        let qc   = vo & hs != 0;
                        let bjj   = next & hs != 0;
                        
                        let mle = bj > 0 && (vo & (hs << 1)) != 0;
                        let agd = bj < 7 && (vo & (hs >> 1)) != 0;
                        let bl = bj > 0 && (next & (hs << 1)) != 0;
                        let avi = bj < 7 && (next & (hs >> 1)) != 0;
                        
                        
                        let qwk = (fd as u32) + (hw as u32) + (qc as u32) + (bjj as u32);
                        let kpu = (mle as u32) + (agd as u32) + (bl as u32) + (avi as u32);
                        let ol = qwk * 2 + kpu; 
                        
                        if ol > 0 {
                            
                            let dw = if ol >= 6 { 140u32 }
                                else if ol >= 4 { 100u32 }
                                else if ol >= 2 { 60u32 }
                                else { 35u32 };
                            let wq = 255 - dw;
                            for cq in 0..pv {
                                for cr in 0..pv {
                                    let y = cx as u32 + bj * pv + cr;
                                    let x = c as u32 + br * pv + cq;
                                    if y < gz && x < kc {
                                        let ei = framebuffer::iwt(y, x);
                                        let cos = (ei >> 16) & 0xFF;
                                        let cor = (ei >> 8) & 0xFF;
                                        let coq = ei & 0xFF;
                                        let m = (ebm * dw + cos * wq) / 255;
                                        let at = (ebl * dw + cor * wq) / 255;
                                        let o = (ebk * dw + coq * wq) / 255;
                                        framebuffer::ii(y, x, 0xFF000000 | (m << 16) | (at << 8) | o);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn tlr(&mut self, b: i32, c: i32, nr: u32) {
        let bh = match self.ee.iter().du(|d| d.ad == nr) {
            Some(d) => d,
            None => return,
        };
        let fx = bh.b;
        let lw = bh.c;
        let hk = bh.z;
        let mg = bh.ac;
        let gl = lw + J_ as i32;
        let bbs = 36i32;

        
        let mut ou = gl + bbs + 2;
        if crate::drivers::net::wifi::cwo().is_some() {
            let gzo = 40i32;
            
            let gev = (fx as i32 + hk as i32) - 100;
            if b >= gev && b < gev + 80 && c >= ou + 8 && c < ou + 32 {
                crate::drivers::net::wifi::irg();
                return;
            }
            ou += gzo + 4;
        }

        
        let ftq = 80i32;
        let grj = (fx as i32 + hk as i32) - ftq - 8;
        if b >= grj && b < grj + ftq
            && c >= ou && c < ou + 26 {
            crate::drivers::net::wifi::pod();
            return;
        }

        ou += 32; 

        
        let hso = crate::drivers::net::wifi::nym();
        let ph = 44i32;
        for (a, net) in hso.iter().cf().chz(self.gws) {
            let afy = ou + ((a - self.gws) as i32 * ph);
            if afy + ph > lw + mg as i32 { break; }
            if c >= afy && c < afy + ph && b >= fx && b < fx + hk as i32 {
                self.jwu = a;
                
                if net.security == crate::drivers::net::wifi::WifiSecurity::Ck {
                    crate::drivers::net::wifi::lzk(&net.bfk, "");
                } else {
                    self.ihf = net.bfk.clone();
                    self.ddl.clear();
                    self.fbj = None;
                    self.xl("WiFi Password", 250, 150, 360, 300, WindowType::Aft);
                }
                return;
            }
        }
    }

    fn tls(&mut self, b: i32, c: i32, nr: u32) {
        let bh = match self.ee.iter().du(|d| d.ad == nr) {
            Some(d) => d,
            None => return,
        };
        let fx = bh.b;
        let lw = bh.c;
        let hk = bh.z;
        let mg = bh.ac;
        let gl = lw + J_ as i32;

        let alf = gl + 108;
        let hob = 32;

        
        let faa = alf + hob + 8;
        if b >= fx + 20 && b < fx + 34 && c >= faa && c < faa + 14 {
            self.fyt = !self.fyt;
            return;
        }

        
        let cwb = (lw as i32 + mg as i32) - 50;
        let bym = 100i32;
        let doq = 32i32;
        let hbu = 16i32;
        let mmc = bym * 2 + hbu;
        let end = fx + (hk as i32 - mmc) / 2;

        
        if b >= end && b < end + bym
            && c >= cwb && c < cwb + doq {
            if self.ddl.is_empty() {
                self.fbj = Some(String::from("Password cannot be empty"));
            } else {
                crate::drivers::net::wifi::lzk(
                    &self.ihf,
                    &self.ddl,
                );
                
                self.ee.ajm(|d| d.ad != nr);
            }
            return;
        }

        
        let gcd = end + bym + hbu;
        if b >= gcd && b < gcd + bym
            && c >= cwb && c < cwb + doq {
            self.ee.ajm(|d| d.ad != nr);
            return;
        }
    }
}


fn yss(r: char) -> [u8; 16] {
    
    match r {
        'A' => [0x00,0x18,0x3C,0x66,0x66,0x7E,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00],
        'B' => [0x00,0x7C,0x66,0x66,0x7C,0x66,0x66,0x66,0x7C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'C' => [0x00,0x3C,0x66,0x60,0x60,0x60,0x60,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'D' => [0x00,0x78,0x6C,0x66,0x66,0x66,0x66,0x6C,0x78,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'E' => [0x00,0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'F' => [0x00,0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'G' => [0x00,0x3C,0x66,0x60,0x60,0x6E,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'H' => [0x00,0x66,0x66,0x66,0x7E,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'I' => [0x00,0x3C,0x18,0x18,0x18,0x18,0x18,0x18,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'J' => [0x00,0x1E,0x0C,0x0C,0x0C,0x0C,0x6C,0x6C,0x38,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'K' => [0x00,0x66,0x6C,0x78,0x70,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'L' => [0x00,0x60,0x60,0x60,0x60,0x60,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'M' => [0x00,0x63,0x77,0x7F,0x6B,0x63,0x63,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'N' => [0x00,0x66,0x76,0x7E,0x7E,0x6E,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'O' => [0x00,0x3C,0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'P' => [0x00,0x7C,0x66,0x66,0x7C,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Q' => [0x00,0x3C,0x66,0x66,0x66,0x66,0x6E,0x3C,0x0E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'R' => [0x00,0x7C,0x66,0x66,0x7C,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'S' => [0x00,0x3C,0x66,0x60,0x3C,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'T' => [0x00,0x7E,0x18,0x18,0x18,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'U' => [0x00,0x66,0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'V' => [0x00,0x66,0x66,0x66,0x66,0x66,0x3C,0x3C,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'W' => [0x00,0x63,0x63,0x63,0x6B,0x7F,0x77,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'X' => [0x00,0x66,0x66,0x3C,0x18,0x3C,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Y' => [0x00,0x66,0x66,0x66,0x3C,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Z' => [0x00,0x7E,0x06,0x0C,0x18,0x30,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'a' => [0x00,0x00,0x00,0x3C,0x06,0x3E,0x66,0x66,0x3E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'b' => [0x00,0x60,0x60,0x7C,0x66,0x66,0x66,0x66,0x7C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'c' => [0x00,0x00,0x00,0x3C,0x66,0x60,0x60,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'd' => [0x00,0x06,0x06,0x3E,0x66,0x66,0x66,0x66,0x3E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'e' => [0x00,0x00,0x00,0x3C,0x66,0x7E,0x60,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'f' => [0x00,0x1C,0x36,0x30,0x7C,0x30,0x30,0x30,0x30,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'g' => [0x00,0x00,0x00,0x3E,0x66,0x66,0x3E,0x06,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'h' => [0x00,0x60,0x60,0x7C,0x66,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'i' => [0x00,0x18,0x00,0x38,0x18,0x18,0x18,0x18,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'j' => [0x00,0x0C,0x00,0x1C,0x0C,0x0C,0x0C,0x6C,0x38,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'k' => [0x00,0x60,0x60,0x66,0x6C,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'l' => [0x00,0x38,0x18,0x18,0x18,0x18,0x18,0x18,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'm' => [0x00,0x00,0x00,0x76,0x7F,0x6B,0x6B,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'n' => [0x00,0x00,0x00,0x7C,0x66,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'o' => [0x00,0x00,0x00,0x3C,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'p' => [0x00,0x00,0x00,0x7C,0x66,0x66,0x7C,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'q' => [0x00,0x00,0x00,0x3E,0x66,0x66,0x3E,0x06,0x06,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'r' => [0x00,0x00,0x00,0x7C,0x66,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        's' => [0x00,0x00,0x00,0x3E,0x60,0x3C,0x06,0x06,0x7C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        't' => [0x00,0x30,0x30,0x7C,0x30,0x30,0x30,0x36,0x1C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'u' => [0x00,0x00,0x00,0x66,0x66,0x66,0x66,0x66,0x3E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'v' => [0x00,0x00,0x00,0x66,0x66,0x66,0x3C,0x3C,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'w' => [0x00,0x00,0x00,0x63,0x63,0x6B,0x7F,0x77,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'x' => [0x00,0x00,0x00,0x66,0x3C,0x18,0x3C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'y' => [0x00,0x00,0x00,0x66,0x66,0x3E,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'z' => [0x00,0x00,0x00,0x7E,0x0C,0x18,0x30,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '0' => [0x00,0x3C,0x66,0x6E,0x76,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '1' => [0x00,0x18,0x38,0x18,0x18,0x18,0x18,0x18,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '2' => [0x00,0x3C,0x66,0x06,0x0C,0x18,0x30,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '3' => [0x00,0x3C,0x66,0x06,0x1C,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '4' => [0x00,0x0C,0x1C,0x3C,0x6C,0x7E,0x0C,0x0C,0x0C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '5' => [0x00,0x7E,0x60,0x7C,0x06,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '6' => [0x00,0x3C,0x66,0x60,0x7C,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '7' => [0x00,0x7E,0x06,0x0C,0x18,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '8' => [0x00,0x3C,0x66,0x66,0x3C,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '9' => [0x00,0x3C,0x66,0x66,0x3E,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '-' => [0x00,0x00,0x00,0x00,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        ' ' => [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        _ => [0x00,0x3C,0x42,0x42,0x42,0x42,0x42,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00], 
    }
}


pub static Aa: Mutex<Desktop> = Mutex::new(Desktop::new());


pub fn init(z: u32, ac: u32) {
    Aa.lock().init(z, ac);
    crate::serial_println!("[GUI] Desktop initialized: {}x{} (double-buffered)", z, ac);
}


pub fn xl(dq: &str, b: i32, c: i32, z: u32, ac: u32) -> u32 {
    Aa.lock().xl(dq, b, c, z, ac, WindowType::Jl)
}


pub fn yks(b: i32, c: i32) -> u32 {
    Aa.lock().xl("Terminal", b, c, 640, 440, WindowType::Ay)
}


pub fn ykr(b: i32, c: i32) -> u32 {
    Aa.lock().xl("System Info", b, c, 300, 220, WindowType::Qx)
}


pub fn iod(ad: u32) {
    Aa.lock().iod(ad);
}


pub fn fas(b: i32, c: i32) {
    Aa.lock().tkj(b, c);
}


pub fn ago(b: i32, c: i32, vn: bool) {
    Aa.lock().ago(b, c, vn);
}


pub fn po() {
    Aa.lock().po();
}


pub fn tke(bs: u8) {
    Aa.lock().oah(bs);
}


pub fn hmk(b: i32, c: i32, vn: bool) {
    Aa.lock().hmk(b, c, vn);
}


pub fn ers(aaq: i8) {
    Aa.lock().ers(aaq);
}


pub fn vw() {
    use crate::gui::engine::{self, HotkeyAction};
    MX_.store(false, Ordering::SeqCst);
    
    
    engine::oep();
    crate::gui::vsync::init();
    
    
    
    unsafe {
        
        static mut COO_: bool = true;
        COO_ = true;
    }
    
    {
        let mut bc = Aa.lock();
        bc.aka.iw = false;
    }
    
    crate::serial_println!("[GUI] Starting desktop environment...");
    crate::serial_println!("[GUI] Hotkeys: Alt+Tab, Win+Arrows, Alt+F4, Win=Start");
    crate::serial_println!("[GUI] Target: ~60 FPS (16.6ms) with spin-loop frame limiting");
    
    
    let mut okj: u32 = 0;

    loop {
        
        if MX_.load(Ordering::SeqCst) {
            crate::serial_println!("[GUI] Desktop exit requested, returning to shell");
            break;
        }
        let gho = engine::awf();
        
        
        
        
        
        
        let mouse = crate::mouse::drd();
        fas(mouse.b, mouse.c);
        
        
        
        {
            let kai = crate::keyboard::alh(0x38);
            let mqr = crate::keyboard::alh(0x5B);
            let kme = crate::keyboard::alh(0x1D);
            let prl = crate::keyboard::alh(0x0F);
            static mut BDN_: bool = false;
            unsafe {
                if (kai || mqr || kme) && prl && !BDN_ {
                    if !engine::hot() {
                        engine::pny();
                    } else {
                        engine::mvg();
                    }
                }
                BDN_ = prl;
            }
        }
        
        
        
        
        
        
        let mut ohw = 0u32;
        while let Some(bs) = crate::keyboard::auw() {
            ohw += 1;
            if ohw > 32 { break; }
            crate::serial_println!("[INPUT-DBG] key={} (0x{:02X})", bs, bs);
            
            let bdj = crate::keyboard::alh(0x38);
            let msg = crate::keyboard::alh(0x1D);
            let ep = crate::keyboard::alh(0x5B);
            
            
            
            
            if bs == 27 {
                crate::serial_println!("[GUI] ESC pressed");
                
                if let Some(mut bc) = Aa.try_lock() {
                    
                    if bc.ud.gh {
                        crate::serial_println!("[GUI] ESC: mobile mode, exiting to shell");
                        drop(bc);
                        MX_.store(true, Ordering::SeqCst);
                        continue;
                    }
                    
                    if bc.ajo {
                        bc.ajo = false;
                        bc.bij.clear();
                        bc.bsl = -1;
                        crate::serial_println!("[GUI] ESC: closed start menu");
                        drop(bc);
                        continue;
                    }
                    
                    let sjb = {
                        let ja = bc.ee.iter().du(|d| d.ja && !d.aat);
                        if let Some(d) = ja {
                            if d.ld == WindowType::Ag {
                                if let Some(editor) = bc.cxh.get(&d.ad) {
                                    editor.cqn.is_some() || editor.dri.is_some()
                                } else { false }
                            } else { false }
                        } else { false }
                    };
                    if sjb {
                        let ajq = bc.ee.iter().du(|d| d.ja && !d.aat).map(|d| d.ad);
                        if let Some(ad) = ajq {
                            if let Some(editor) = bc.cxh.ds(&ad) {
                                editor.vr(27);
                            }
                        }
                        drop(bc);
                        continue;
                    }
                    
                    
                    let qsc = {
                        let ja = bc.ee.iter().du(|d| d.ja && !d.aat);
                        ja.map(|d| d.ld == WindowType::Browser).unwrap_or(false)
                    };
                    if qsc {
                        bc.oah(27);
                        drop(bc);
                        continue;
                    }
                    
                    let ivf = bc.ee.iter().du(|d| d.ja && !d.aat).map(|d| d.ad);
                    if let Some(ajq) = ivf {
                        crate::serial_println!("[GUI] ESC: closing window {}", ajq);
                        bc.ndo();
                        crate::serial_println!("[GUI] ESC: window closed OK");
                    } else {
                        crate::serial_println!("[GUI] ESC: no focused window, ignoring");
                    }
                    drop(bc);
                } else {
                    crate::serial_println!("[GUI] ESC: lock busy, skipping");
                }
                continue;
            }
            
            
            
            {
                static mut ABZ_: bool = false;
                let nsq = crate::keyboard::alh(0x3B);
                unsafe {
                    if nsq && !bdj && !ep && !ABZ_ {
                        ABZ_ = true;
                        let mut bc = Aa.lock();
                        bc.iak = !bc.iak;
                        crate::serial_println!("[GUI] F1: shortcuts overlay = {}", bc.iak);
                        drop(bc);
                    }
                    if !nsq { ABZ_ = false; }
                }
            }
            
            
            if (bdj || ep || msg) && bs == 9 {
                if !engine::hot() {
                    engine::pny();
                } else {
                    engine::mvg();
                }
                continue;
            }
            
            
            if ep && bs == crate::keyboard::AH_ {
                Aa.lock().mgh(SnapDir::Ap);
                unsafe { DB_ = true; }
                continue;
            }
            
            if ep && bs == crate::keyboard::AI_ {
                Aa.lock().mgh(SnapDir::Ca);
                unsafe { DB_ = true; }
                continue;
            }
            
            if ep && bs == crate::keyboard::V_ {
                Aa.lock().xiw();
                unsafe { DB_ = true; }
                continue;
            }
            
            if ep && bs == crate::keyboard::U_ {
                Aa.lock().uoo();
                unsafe { DB_ = true; }
                continue;
            }
            
            
            if ep && (bs == b'd' || bs == b'D') {
                Aa.lock().puc();
                unsafe { DB_ = true; }
                crate::serial_println!("[GUI] Win+D: toggle show desktop");
                continue;
            }
            
            
            if ep && (bs == b'e' || bs == b'E') {
                Aa.lock().xl("File Explorer", 100, 60, 780, 520, WindowType::Ak);
                unsafe { DB_ = true; }
                crate::serial_println!("[GUI] Win+E: open file manager");
                continue;
            }
            
            
            if ep && (bs == b'i' || bs == b'I') {
                Aa.lock().lqt();
                unsafe { DB_ = true; }
                crate::serial_println!("[GUI] Win+I: open settings");
                continue;
            }
            
            
            if ep && (bs == b'h' || bs == b'H') {
                crate::accessibility::mln();
                let mut bc = Aa.lock();
                bc.bex = true;
                bc.doh = false;
                drop(bc);
                unsafe { DB_ = true; }
                crate::serial_println!("[GUI] Win+H: toggle high contrast");
                continue;
            }
            
            
            if ep && (bs == b'l' || bs == b'L') {
                let mut bc = Aa.lock();
                bc.eug = true;
                bc.djb.clear();
                bc.eeu = 0;
                drop(bc);
                unsafe { DB_ = true; }
                crate::serial_println!("[GUI] Win+L: lock screen");
                continue;
            }
            
            
            if ep && bs != 0 {
                unsafe { DB_ = true; }
            }
            
            
            
            if bdj && crate::keyboard::alh(0x3E) {
                let mut bc = Aa.lock();
                let tmm = bc.ee.iter().any(|d| d.ja && !d.aat);
                if tmm {
                    bc.ndo();
                    crate::serial_println!("[GUI] Alt+F4: closed focused window");
                } else {
                    crate::serial_println!("[GUI] Alt+F4: no window, exiting desktop");
                    MX_.store(true, Ordering::SeqCst);
                }
                drop(bc);
                continue;
            }
            
            
            crate::serial_println!("[MAIN-DBG] passing key {} (0x{:02X}) to handle_keyboard", bs, bs);
            tke(bs);
        }
        
        
        if engine::hot() {
            let kai = crate::keyboard::alh(0x38);
            let mqr = crate::keyboard::alh(0x5B);
            let kme = crate::keyboard::alh(0x1D);
            if !kai && !mqr && !kme {
                let na = engine::stv();
                Aa.lock().svi(na as usize);
            }
        }
        
        
        
        static mut AEJ_: bool = false;
        static mut DB_: bool = false;
        {
            let jww = crate::keyboard::alh(0x5B);
            unsafe {
                if jww && !AEJ_ {
                    
                    DB_ = false;
                }
                if jww {
                    
                    if engine::hot() {
                        DB_ = true;
                    }
                }
                if !jww && AEJ_ && !DB_ {
                    
                    let mut bc = Aa.lock();
                    bc.ajo = !bc.ajo;
                }
                AEJ_ = jww;
            }
        }
        
        
        static mut AYU_: bool = false;
        let fd = mouse.jda;
        unsafe {
            if fd != AYU_ {
                if fd {
                    crate::serial_println!("[INPUT-DBG] mouse click at ({},{})", mouse.b, mouse.c);
                }
                
                if fd {
                    let mut bc = Aa.lock();
                    if bc.ajo {
                        
                    }
                    drop(bc);
                }
                ago(mouse.b, mouse.c, fd);
                AYU_ = fd;
            }
        }
        
        
        static mut AEG_: bool = false;
        static mut BEM_: bool = false;
        let hw = mouse.pdd;
        unsafe {
            if !BEM_ {
                
                AEG_ = hw;
                BEM_ = true;
            }
            if hw != AEG_ {
                hmk(mouse.b, mouse.c, hw);
                AEG_ = hw;
            }
        }
        
        
        let jc = crate::mouse::teo();
        if jc != 0 {
            ers(jc);
        }
        
        
        
        
        {
            let mut bc = Aa.lock();
            bc.vmu();
            drop(bc);
        }
        
        
        
        
        {
            let result = {
                let mut m = AXO_.lock();
                m.take()
            };
            if let Some(ak) = result {
                let mut bc = Aa.lock();
                
                if let Some(bh) = bc.ee.el().du(|d| d.ld == WindowType::Ay) {
                    
                    if bh.ca.qv().map(|e| e.contains("$ ")).unwrap_or(false) {
                        bh.ca.pop();
                    }
                    
                    for line in &ak {
                        bh.ca.push(line.clone());
                    }
                    
                    bh.ca.push(Desktop::csi("_"));
                    
                    let acg = 16usize;
                    let ffq = (bh.ac as usize).ao(J_ as usize + 16);
                    let act = if acg > 0 { ffq / acg } else { 1 };
                    if bh.ca.len() > act {
                        bh.px = bh.ca.len() - act;
                    } else {
                        bh.px = 0;
                    }
                }
                drop(bc);
            }
        }

        
        
        
        {
            let result = {
                let mut m = ALX_.lock();
                m.take()
            };
            if let Some(lnn) = result {
                let mut bc = Aa.lock();
                match lnn {
                    Ok((hjl, wt, zk, gj)) => {
                        crate::serial_println!("[BROWSER-BG] Received {} bytes, status {}", gj.len(), wt);
                        if let Some(ref mut browser) = bc.browser {
                            if wt >= 400 {
                                browser.status = crate::browser::BrowserStatus::Q(alloc::format!("HTTP {}", wt));
                                browser.bfc = alloc::format!(
                                    "<html><body><h1>HTTP Error {}</h1><p>The server returned an error for {}</p></body></html>",
                                    wt, hjl
                                );
                                browser.ama = Some(crate::browser::due(&browser.bfc));
                            } else if wt >= 300 && wt < 400 {
                                
                                let cse = zk.iter()
                                    .du(|(eh, _)| eh.aqn() == "location")
                                    .map(|(_, p)| p.clone());
                                if let Some(euf) = cse {
                                    crate::serial_println!("[BROWSER-BG] Redirect {} -> {}", wt, euf);
                                    bc.ado = euf.clone();
                                    bc.aef = bc.ado.len();
                                    
                                    {
                                        let mut aln = ZR_.lock();
                                        *aln = Some(euf);
                                    }
                                    RR_.store(true, Ordering::SeqCst);
                                    crate::thread::jqu("browser-nav", naf, 0);
                                    drop(bc);
                                    
                                    continue;
                                }
                            } else {
                                
                                let brb = core::str::jg(&gj).unwrap_or("");
                                browser.bfc = String::from(brb);
                                browser.lvt(&zk, &hjl);
                                browser.ama = Some(crate::browser::due(brb));
                                browser.nrp();
                                browser.nsm(&hjl);
                                
                                if browser.ari < browser.adv.len() {
                                    browser.adv.dmu(browser.ari);
                                }
                                browser.adv.push(hjl.clone());
                                browser.ari = browser.adv.len();
                                browser.bdv = hjl.clone();
                                browser.ug = 0;
                                browser.status = crate::browser::BrowserStatus::At;
                            }
                            bc.ado = browser.bdv.clone();
                            bc.aef = bc.ado.len();
                        }
                    }
                    Err(aa) => {
                        crate::serial_println!("[BROWSER-BG] Navigation error: {}", aa);
                        if let Some(ref mut browser) = bc.browser {
                            browser.status = crate::browser::BrowserStatus::Q(aa.clone());
                            browser.bfc = alloc::format!(
                                "<html><body><h1>Error</h1><p>{}</p></body></html>", aa
                            );
                            browser.ama = Some(crate::browser::due(&browser.bfc));
                            browser.ug = 0;
                        }
                    }
                }
                bc.btn = false;
                drop(bc);
            }
        }

        
        
        
        
        
        po();
        
        
        if engine::hot() {
            vvc();
        }
        
        
        
        
        
        
        
        
        {
            let bc = Aa.lock();
            let iah = bc.iak;
            drop(bc);
            if iah {
                vwn();
            }
        }
        
        
        vwg();
        
        
        {
            let bc = Aa.lock();
            let d = bc.z;
            let i = bc.ac;
            drop(bc);
            let swx = engine::awf().ao(gho);
            crate::devtools::vvp(d, i, swx);
        }
        
        
        #[cfg(yli)]
        {
            let tz = engine::kyp();
            
        }
        
        
        
        
        let pce = engine::awf().ao(gho);
        
        {
            let bc = Aa.lock();
            let gc = bc.oo;
            drop(bc);
            if gc % 120 == 0 && gc > 0 {
                let tz = crate::gui::vsync::tz();
                crate::serial_println!("[PERF] frame={} render={}us fps={}", gc, pce, tz);
            }
        }
        crate::gui::vsync::swy(gho);
        okj = okj.akq(1);
    }
    
    
    
    
    crate::serial_println!("[GUI] Desktop exiting, cleaning up...");
    
    {
        let mut bc = Aa.lock();
        for (ddq, sn) in bc.ano.el() {
            sn.qg();
        }
        crate::serial_println!("[GUI] All music players stopped");
    }
    crate::framebuffer::afi(false);
    crate::framebuffer::clear();
    crate::serial_println!("[GUI] Desktop exited cleanly");
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SnapDir {
    Ap,
    Ca,
    Dp,
    Dq,
    Dt,
    Du,
}


fn vvc() {
    let desktop = Aa.lock();
    let ihg = desktop.tfe();
    if ihg.is_empty() { return; }
    
    let wf = desktop.z;
    let aav = desktop.ac;
    drop(desktop);
    
    let gry = crate::gui::engine::qhl();
    let az = ihg.len() as i32;
    let w = ((gry % az) + az) % az;
    
    
    let bpw: u32 = 150;
    let bgg: u32 = 100;
    let qi: u32 = 12;
    let ayf: u32 = 6; 
    let mpl = (ihg.len() as u32).v(ayf);
    let aza = mpl * (bpw + qi) + qi;
    let xhq: u32 = 30;
    let aku = bgg + qi * 2 + xhq + 14;
    
    let mp = (wf as i32 - aza as i32) / 2;
    let qw = (aav as i32 - aku as i32) / 2;
    
    
    mf(mp - 2, qw - 2, aza + 4, aku + 4, 14, 0x40000000);
    mf(mp, qw, aza, aku, 12, 0xE8101420);
    
    tf(mp, qw, aza, aku, 12, 0x3000FF66);
    
    cqm(mp + 14, qw + 1, aza as i32 - 28, 1, 0x20FFFFFF);
    
    
    np(mp + aza as i32 / 2, qw + 8, "Switch Window", 0xFF888888);
    
    
    for (a, (dq, ash)) in ihg.iter().cf() {
        if a as u32 >= ayf { break; }
        let cx = mp + qi as i32 + a as i32 * (bpw + qi) as i32;
        let ae = qw + qi as i32 + 22;
        
        let qe = a as i32 == w;
        
        
        if qe {
            
            mf(cx - 2, ae - 2, bpw + 4, bgg + 4, 8, 0x3000FF66);
            mf(cx, ae, bpw, bgg, 6, 0xFF1A2A20);
            
            tf(cx, ae, bpw, bgg, 6, 0xFF00CC55);
        } else {
            mf(cx, ae, bpw, bgg, 6, 0xFF1A1E28);
            tf(cx, ae, bpw, bgg, 6, 0xFF2A2E38);
        }
        
        
        let pa = xur(*ash);
        let bel = cx + (bpw as i32 - crate::graphics::scaling::clj(pa) as i32) / 2;
        let bem = ae + 20;
        let xd = if qe { 0xFF00FF66 } else { 0xFF667766 };
        cb(bel, bem, pa, xd);
        
        
        let mnr = xus(*ash);
        let bbw = if qe { 0xFF00CC55 } else { 0xFF555555 };
        np(cx + bpw as i32 / 2, ae + bgg as i32 - 22, mnr, bbw);
        
        
        let iag: alloc::string::String = dq.bw().take(16).collect();
        let ejy = if qe { 0xFFFFFFFF } else { 0xFF999999 };
        np(cx + bpw as i32 / 2, ae + bgg as i32 + 6, &iag, ejy);
    }
    
    
    if ihg.len() > 1 {
        np(mp + aza as i32 / 2, qw + aku as i32 - 18, 
            "Tab: next  |  Release Alt: select", 0xFF555555);
    }
}


fn vwn() {
    let desktop = Aa.lock();
    let wf = desktop.z;
    let aav = desktop.ac;
    drop(desktop);

    
    let fej: &[(&str, &[(&str, &str)])] = &[
        ("Navigation", &[
            ("Win", "Toggle Start Menu"),
            ("Alt+Tab", "Switch Windows"),
            ("Win+D", "Show Desktop"),
            ("Win+L", "Lock Screen"),
            ("ESC", "Close Window / Menu"),
            ("Alt+F4", "Force Close Window"),
        ]),
        ("Windows", &[
            ("Win+Left", "Snap Left"),
            ("Win+Right", "Snap Right"),
            ("Win+Up", "Maximize"),
            ("Win+Down", "Minimize"),
        ]),
        ("Apps", &[
            ("Win+E", "File Manager"),
            ("Win+I", "Settings"),
            ("Win+H", "High Contrast"),
        ]),
        ("Editor", &[
            ("Ctrl+S", "Save"),
            ("Ctrl+F", "Find"),
            ("Ctrl+G", "Go to Line"),
            ("Ctrl+C/X/V", "Copy / Cut / Paste"),
        ]),
        ("File Manager", &[
            ("N / D", "New File / New Folder"),
            ("R", "Rename"),
            ("Del", "Delete"),
            ("V", "Toggle View"),
        ]),
    ];

    
    let doz: u32 = 2;
    let oy: u32 = 300;
    let ph: u32 = 18;
    let kgu: u32 = 8;
    let bbs: u32 = 40;
    let nvm: u32 = 24;
    let adf: u32 = 20;

    
    let mut fxd: u32 = 0;
    for (_, ch) in fej.iter() {
        fxd += 1 + ch.len() as u32; 
    }
    
    let peh = (fxd + 1) / 2;
    let nd = peh * ph + ((fej.len() as u32 + 1) / 2) * kgu;
    let yd = doz * oy + adf * 3;
    let ans = bbs + nd + nvm + adf;

    let mp = (wf as i32 - yd as i32) / 2;
    let qw = (aav as i32 - ans as i32) / 2;

    
    mf(mp - 2, qw - 2, yd + 4, ans + 4, 14, 0x50000000);
    mf(mp, qw, yd, ans, 12, 0xF0101420);
    tf(mp, qw, yd, ans, 12, 0x5000FF66);
    
    cqm(mp + 14, qw + 1, yd as i32 - 28, 1, 0x20FFFFFF);

    
    np(mp + yd as i32 / 2, qw + 12, "Keyboard Shortcuts", 0xFF00FF66);
    
    cqm(mp + adf as i32, qw + bbs as i32 - 6, yd as i32 - adf as i32 * 2, 1, 0x3000FF66);

    
    let mut bj = 0u32;
    let mut grd = 0u32;
    let mut kgt = 0usize;

    for (hci, ch) in fej.iter() {
        
        let lnr = 1 + ch.len() as u32;
        if grd + lnr > peh && bj < doz - 1 {
            bj += 1;
            grd = 0;
        }

        let cx = mp + adf as i32 + bj as i32 * (oy as i32 + adf as i32);
        let ae = qw + bbs as i32 + grd as i32 * ph as i32 + kgt as i32 * kgu as i32;

        
        cb(cx, ae, hci, 0xFF00CC55);
        grd += 1;

        
        for (bs, desc) in ch.iter() {
            let ahm = qw + bbs as i32 + grd as i32 * ph as i32 + kgt as i32 * kgu as i32;
            
            let diq = crate::graphics::scaling::clj(bs) as i32 + 10;
            mf(cx + 4, ahm - 1, diq as u32, 16, 4, 0xFF1A2A20);
            tf(cx + 4, ahm - 1, diq as u32, 16, 4, 0xFF00AA44);
            cb(cx + 9, ahm + 1, bs, 0xFF00FF66);
            
            cb(cx + diq + 14, ahm + 1, desc, 0xFFAAAAAA);
            grd += 1;
        }
        kgt += 1;
    }

    
    np(mp + yd as i32 / 2, qw + ans as i32 - nvm as i32 + 4,
        "Press F1 to close", 0xFF555555);
}


fn xur(ash: WindowType) -> &'static str {
    match ash {
        WindowType::Ay => ">_",
        WindowType::Qx => "[i]",
        WindowType::Jf => "(?)",
        WindowType::Calculator => "[#]",
        WindowType::Ak => "[/]",
        WindowType::Ag => "[=]",
        WindowType::Hy => "[~]",
        WindowType::Gn => "{*}",
        WindowType::Bp => "[^]",
        WindowType::Browser => "</>",
        WindowType::Io => "[*]",
        WindowType::Gs | WindowType::Ih => "[K]",
        WindowType::Fp => "[3D]",
        WindowType::So => "[3D]",
        WindowType::Lw => "[~]",
        WindowType::Td => "{L}",
        WindowType::Ro => "0x",
        _ => "[.]",
    }
}


fn xus(ash: WindowType) -> &'static str {
    match ash {
        WindowType::Ay => "Terminal",
        WindowType::Qx => "System",
        WindowType::Jf => "About",
        WindowType::Calculator => "Calc",
        WindowType::Ak => "Files",
        WindowType::Ag => "Editor",
        WindowType::Hy => "NetScan",
        WindowType::Gn => "Settings",
        WindowType::Bp => "Images",
        WindowType::Browser => "Browser",
        WindowType::Io => "Snake",
        WindowType::Gs => "Chess",
        WindowType::Ih => "Chess 3D",
        WindowType::Fp => "3D Edit",
        WindowType::So => "FPS",
        WindowType::Lw => "Music",
        WindowType::Td => "Lab",
        WindowType::Ro => "BinView",
        _ => "Window",
    }
}


fn zjm() {
    use crate::gui::engine::{nyp, StartAction};
    
    let desktop = Aa.lock();
    let aav = desktop.ac;
    drop(desktop);
    
    let afr: u32 = 280;
    let aje: u32 = 350;
    let b: i32 = 10;
    let c: i32 = aav as i32 - W_ as i32 - aje as i32 - 5;
    
    
    mf(b, c, afr, aje, 12, 0xF0101520);
    mf(b + 1, c + 1, afr - 2, aje - 2, 11, 0xF0181C25);
    
    
    cb(b + 20, c + 15, "TrustOS", 0xFF00FF66);
    cb(b + 90, c + 15, "v0.1", 0xFF606060);
    
    
    ahj(b + 15, c + 35, b + afr as i32 - 15, c + 35, 0xFF303540);
    
    
    let pj = nyp();
    let mut og = c + 45;
    
    for item in pj.iter() {
        if item.pa == 255 {
            
            ahj(b + 15, og + 5, b + afr as i32 - 15, og + 5, 0xFF303540);
            og += 12;
        } else {
            
            cb(b + 40, og, &item.j, 0xFFCCCCCC);
            og += 28;
        }
    }
    
    
    let blb = c + aje as i32 - 45;
    mf(b + 15, blb, afr - 30, 30, 6, 0xFF252A35);
    cb(b + 25, blb + 7, "Search apps...", 0xFF606060);
}


fn vwg() {
    use crate::gui::engine::{nyg, NotifyPriority};
    
    let desktop = Aa.lock();
    let wf = desktop.z;
    drop(desktop);
    
    let csv = nyg();
    if csv.is_empty() { return; }
    
    let mut c: i32 = 55; 
    
    for ezz in csv.iter() {
        let d: u32 = 320;
        let tmy = ezz.li.is_some();
        let i: u32 = if tmy { 78 } else { 64 };
        let adh = ezz.adh();
        if adh == 0 { continue; }
        
        
        let ez = ezz.oz();
        let wpp = if ez < 300 {
            ((300 - ez) * 40 / 300) as i32
        } else {
            0
        };
        let b = wf as i32 - d as i32 - 15 + wpp;
        
        
        let qpb = (adh as u32 * 0xF0 / 255) << 24;
        let vp = qpb | 0x00141820;
        
        
        let erj = (adh as u32 * 0x18 / 255) << 24;
        mf(b - 1, c - 1, d + 2, i + 2, 11, erj | 0x00000000);
        
        
        mf(b, c, d, i, 10, vp);
        
        
        let ixc = (adh as u32 * 0x15 / 255) << 24;
        cqm(b + 12, c + 1, d as i32 - 24, 1, ixc | 0x00FFFFFF);
        
        
        let axm = ezz.tda();
        let qer = (adh as u32 * ((axm >> 24) & 0xFF) / 255) << 24;
        let mti = axm & 0x00FFFFFF;
        cqm(b + 2, c + 8, 3, i as i32 - 16, qer | mti);
        
        
        let pa = match ezz.abv {
            NotifyPriority::V => "[i]",
            NotifyPriority::Oo => "/!\\",
            NotifyPriority::Q => "[X]",
            NotifyPriority::Hf => "[v]",
        };
        let trb = (adh as u32 * 0xFF / 255) << 24;
        cb(b + 14, c + 12, pa, trb | mti);
        
        
        let xhp = (adh as u32 * 0xFF / 255) << 24;
        let xhy: alloc::string::String = ezz.dq.bw().take(28).collect();
        cb(b + 48, c + 12, &xhy, xhp | 0x00EEEEEE);
        
        
        let uqd = (adh as u32 * 0xBB / 255) << 24;
        let uqf: alloc::string::String = ezz.message.bw().take(36).collect();
        cb(b + 14, c + 34, &uqf, uqd | 0x00999999);
        
        
        if let Some(egl) = ezz.li {
            let pl = c + 54;
            let lo = d - 28;
            let kby = (adh as u32 * 0xFF / 255) << 24;
            mf(b + 14, pl, lo, 8, 3, kby | 0x00252A35);
            let akd = (lo * egl as u32 / 100).am(1);
            if akd > 4 {
                mf(b + 14, pl, akd, 8, 3, kby | 0x0000CC55);
            }
            
            let jiy = alloc::format!("{}%", egl);
            cb(b + d as i32 - 40, pl - 1, &jiy, kby | 0x00777777);
        }
        
        
        let qrj = (adh as u32 * 0x10 / 255) << 24;
        cqm(b + 10, c + i as i32 - 1, d as i32 - 20, 1, qrj | 0x00FFFFFF);
        
        c += i as i32 + 8;
    }
}


fn cb(b: i32, c: i32, text: &str, s: u32) {
    crate::graphics::scaling::kri(b, c, text, s);
}


fn np(cx: i32, c: i32, text: &str, s: u32) {
    let d = crate::graphics::scaling::clj(text) as i32;
    cb(cx - d / 2, c, text, s);
}


fn ahj(dn: i32, dp: i32, hy: i32, jz: i32, s: u32) {
    
    if dp == jz {
        let (ihq, fza) = if dn < hy { (dn, hy) } else { (hy, dn) };
        for b in ihq..=fza {
            if b >= 0 {
                crate::framebuffer::draw_pixel(b as u32, dp as u32, s);
            }
        }
    }
}


fn lx(b: i32, c: i32, d: u32, i: u32, s: u32) {
    for bg in 0..i {
        for dx in 0..d {
            crate::framebuffer::draw_pixel((b + dx as i32) as u32, (c + bg as i32) as u32, s);
        }
    }
}


fn sfh(b: i32, c: i32, d: u32, i: u32, dy: u32, s: u32, dw: u32) {
    if d == 0 || i == 0 { return; }
    let m = dy.v(d / 2).v(i / 2);

    if m == 0 {
        if b >= 0 && c >= 0 {
            crate::framebuffer::ih(b as u32, c as u32, d, i, s, dw);
        }
        return;
    }

    let yi = d as i32;
    let gd = i as i32;
    let jl = m as i32;

    fiq(b, c + jl, yi, gd - jl * 2, s, dw);
    fiq(b + jl, c, yi - jl * 2, jl, s, dw);
    fiq(b + jl, c + gd - jl, yi - jl * 2, jl, s, dw);

    let uv = jl * jl;
    for bg in 0..jl {
        let dx = ggu(uv - bg * bg);
        fiq(b + jl - dx, c + jl - bg - 1, dx, 1, s, dw);
        fiq(b + yi - jl, c + jl - bg - 1, dx, 1, s, dw);
        fiq(b + jl - dx, c + gd - jl + bg, dx, 1, s, dw);
        fiq(b + yi - jl, c + gd - jl + bg, dx, 1, s, dw);
    }
}


fn mf(b: i32, c: i32, d: u32, i: u32, dy: u32, s: u32) {
    if d == 0 || i == 0 { return; }
    let m = dy.v(d / 2).v(i / 2);

    if m == 0 {
        
        if b >= 0 && c >= 0 {
            crate::framebuffer::ah(b as u32, c as u32, d, i, s);
        }
        return;
    }

    let yi = d as i32;
    let gd = i as i32;
    let jl = m as i32;

    
    
    cqm(b, c + jl, yi, gd - jl * 2, s);
    
    cqm(b + jl, c, yi - jl * 2, jl, s);
    
    cqm(b + jl, c + gd - jl, yi - jl * 2, jl, s);

    
    
    let uv = jl * jl;
    for bg in 0..jl {
        
        let dx = ggu(uv - bg * bg);
        
        cqm(b + jl - dx, c + jl - bg - 1, dx, 1, s);
        
        cqm(b + yi - jl, c + jl - bg - 1, dx, 1, s);
        
        cqm(b + jl - dx, c + gd - jl + bg, dx, 1, s);
        
        cqm(b + yi - jl, c + gd - jl + bg, dx, 1, s);
    }
}


fn tf(b: i32, c: i32, d: u32, i: u32, dy: u32, s: u32) {
    if d == 0 || i == 0 { return; }
    let m = dy.v(d / 2).v(i / 2);
    let yi = d as i32;
    let gd = i as i32;
    let jl = m as i32;

    if m == 0 {
        if b >= 0 && c >= 0 {
            crate::framebuffer::lx(b as u32, c as u32, d, i, s);
        }
        return;
    }

    
    for y in jl..yi - jl {
        dat(b + y, c, s);            
        dat(b + y, c + gd - 1, s);   
    }
    for x in jl..gd - jl {
        dat(b, c + x, s);            
        dat(b + yi - 1, c + x, s);   
    }

    
    let mut cx = jl;
    let mut ae = 0i32;
    let mut rq = 0i32;
    while cx >= ae {
        
        dat(b + jl - cx, c + jl - ae, s);
        dat(b + jl - ae, c + jl - cx, s);
        
        dat(b + yi - 1 - jl + cx, c + jl - ae, s);
        dat(b + yi - 1 - jl + ae, c + jl - cx, s);
        
        dat(b + jl - cx, c + gd - 1 - jl + ae, s);
        dat(b + jl - ae, c + gd - 1 - jl + cx, s);
        
        dat(b + yi - 1 - jl + cx, c + gd - 1 - jl + ae, s);
        dat(b + yi - 1 - jl + ae, c + gd - 1 - jl + cx, s);

        ae += 1;
        rq += 1 + 2 * ae;
        if 2 * (rq - cx) + 1 > 0 {
            cx -= 1;
            rq += 1 - 2 * cx;
        }
    }
}


#[inline]
fn cqm(b: i32, c: i32, d: i32, i: i32, s: u32) {
    if d <= 0 || i <= 0 { return; }
    let y = b.am(0) as u32;
    let x = c.am(0) as u32;
    let dt = if b < 0 { (d + b).am(0) as u32 } else { d as u32 };
    let bm = if c < 0 { (i + c).am(0) as u32 } else { i as u32 };
    if dt > 0 && bm > 0 {
        crate::framebuffer::ah(y, x, dt, bm, s);
    }
}


fn fiq(b: i32, c: i32, d: i32, i: i32, s: u32, dw: u32) {
    if d <= 0 || i <= 0 { return; }
    let y = b.am(0) as u32;
    let x = c.am(0) as u32;
    let dt = if b < 0 { (d + b).am(0) as u32 } else { d as u32 };
    let bm = if c < 0 { (i + c).am(0) as u32 } else { i as u32 };
    if dt > 0 && bm > 0 {
        crate::framebuffer::ih(y, x, dt, bm, s, dw);
    }
}


#[inline]
fn dat(b: i32, c: i32, s: u32) {
    if b >= 0 && c >= 0 {
        crate::framebuffer::draw_pixel(b as u32, c as u32, s);
    }
}


#[inline]
fn ggu(p: i32) -> i32 {
    if p <= 0 { return 0; }
    let mut b = p;
    let mut c = (b + 1) / 2;
    while c < b {
        b = c;
        c = (b + p / b) / 2;
    }
    b
}


#[inline]
fn ow() -> u64 {
    crate::arch::aea()
}


pub fn dvr(ev: RenderMode) {
    Aa.lock().dvr(ev);
    let czz = match ev {
        RenderMode::Apy => "Classic",
        RenderMode::Ks => "OpenGL Compositor",
        RenderMode::Atd => "GPU Accelerated",
    };
    crate::serial_println!("[GUI] Render mode: {}", czz);
}


pub fn bxb(theme: CompositorTheme) {
    Aa.lock().bxb(theme);
    let ezr = match theme {
        CompositorTheme::Aif => "Flat",
        CompositorTheme::Xq => "Modern",
        CompositorTheme::Ait => "Glass",
        CompositorTheme::Tp => "Neon",
        CompositorTheme::Gy => "Minimal",
    };
    crate::serial_println!("[GUI] Compositor theme: {}", ezr);
}


pub fn yts() -> RenderMode {
    Aa.lock().che
}
