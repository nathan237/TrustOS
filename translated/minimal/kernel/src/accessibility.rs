




use core::sync::atomic::{AtomicU8, AtomicBool, Ordering};
use spin::Mutex;






static NJ_: AtomicBool = AtomicBool::new(false);


static TF_: AtomicU8 = AtomicU8::new(1);


static SM_: AtomicU8 = AtomicU8::new(0);


static AIH_: AtomicBool = AtomicBool::new(false);


static VI_: AtomicU8 = AtomicU8::new(1);






#[derive(Clone, Copy, PartialEq)]
pub enum StickyState {
    Iy,
    Aco,  
    Bln,   
}


static PR_: Mutex<StickyState> = Mutex::new(StickyState::Iy);
static PQ_: Mutex<StickyState> = Mutex::new(StickyState::Iy);
static PS_: Mutex<StickyState> = Mutex::new(StickyState::Iy);






pub struct Cez;
impl Cez {
    pub const RK_: u32     = 0xFF000000;
    pub const JJ_: u32        = 0xFF000000;
    pub const BLB_: u32      = 0xFF0A0A0A;
    pub const ALV_: u32       = 0xFF1A1A1A;
    pub const Deb: u32        = 0xFFFFFF00; 
    pub const Dgl: u32      = 0xFFFFFFFF; 
    pub const Djg: u32       = 0xFFCCCCCC;
    pub const Dcf: u32          = 0xFF888888;
    pub const Dhz: u32         = 0xFF666666;
    pub const Cxv: u32          = 0xFF444444;
    pub const DCB_: u32    = 0xFFFF8800; 
    pub const DBY_: u32   = 0xFFFF0000; 
    pub const DBZ_: u32    = 0xFF00CCFF; 
    pub const AC_: u32   = 0xFFFFFFFF; 
    pub const N_: u32 = 0xFFCCCCCC;
    pub const PY_: u32    = 0xFFFFFF00; 
    pub const DBN_: u32      = 0xFF000000;
    pub const EKL_: u32  = 0xFFFFFFFF;
    pub const EID_: u32      = 0xFF1A1A1A;
    pub const AQN_: u32        = 0xFF0A0A0A;
    pub const BAI_: u32        = 0xFF0A0A0A;
    pub const BAJ_: u32     = 0xFF333300;
    pub const AMA_: u32      = 0xFFFF0000;
    pub const AMB_: u32   = 0xFFFFFF00;
    pub const AMC_: u32   = 0xFF00FF00;
}






pub fn edv() -> bool {
    NJ_.load(Ordering::Relaxed)
}

pub fn znd(iq: bool) {
    NJ_.store(iq, Ordering::Relaxed);
    crate::serial_println!("[A11Y] High contrast: {}", if iq { "ON" } else { "OFF" });
}

pub fn mln() {
    let jwk = NJ_.load(Ordering::Relaxed);
    NJ_.store(!jwk, Ordering::Relaxed);
    crate::serial_println!("[A11Y] High contrast toggled: {}", if !jwk { "ON" } else { "OFF" });
}



#[inline]
pub fn qee(adg: u32, bei: u32) -> u32 {
    if NJ_.load(Ordering::Relaxed) { bei } else { adg }
}



#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum FontSize {
    Ew = 0,
    Bc = 1,
    Ht = 2,
    Arv = 3,
}

impl FontSize {
    pub fn ckp(p: u8) -> Self {
        match p {
            0 => FontSize::Ew,
            1 => FontSize::Bc,
            2 => FontSize::Ht,
            3 => FontSize::Arv,
            _ => FontSize::Bc,
        }
    }

    pub fn cu(&self) -> &'static str {
        match self {
            FontSize::Ew => "Small",
            FontSize::Bc => "Medium",
            FontSize::Ht => "Large",
            FontSize::Arv => "XL",
        }
    }

    
    
    
    pub fn ezq(&self) -> (u32, u32) {
        match self {
            FontSize::Ew => (7, 8),
            FontSize::Bc => (1, 1),
            FontSize::Ht => (5, 4),
            FontSize::Arv => (3, 2),
        }
    }
}

pub fn gid() -> FontSize {
    FontSize::ckp(TF_.load(Ordering::Relaxed))
}

pub fn zna(aw: FontSize) {
    TF_.store(aw as u8, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Font size: {}", aw.cu());
}

pub fn niy() {
    let cv = TF_.load(Ordering::Relaxed);
    let next = (cv + 1) % 4;
    TF_.store(next, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Font size: {}", FontSize::ckp(next).cu());
}



#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum CursorSize {
    Ew = 0,
    Bc = 1,
    Ht = 2,
}

impl CursorSize {
    pub fn ckp(p: u8) -> Self {
        match p {
            0 => CursorSize::Ew,
            1 => CursorSize::Bc,
            2 => CursorSize::Ht,
            _ => CursorSize::Ew,
        }
    }

    pub fn cu(&self) -> &'static str {
        match self {
            CursorSize::Ew => "Small",
            CursorSize::Bc => "Medium",
            CursorSize::Ht => "Large",
        }
    }

    
    pub fn bv(&self) -> u32 {
        match self {
            CursorSize::Ew => 1,
            CursorSize::Bc => 2,
            CursorSize::Ht => 3,
        }
    }
}

pub fn gib() -> CursorSize {
    CursorSize::ckp(SM_.load(Ordering::Relaxed))
}

pub fn zmt(aw: CursorSize) {
    SM_.store(aw as u8, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Cursor size: {}", aw.cu());
}

pub fn nix() {
    let cv = SM_.load(Ordering::Relaxed);
    let next = (cv + 1) % 3;
    SM_.store(next, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Cursor size: {}", CursorSize::ckp(next).cu());
}



pub fn dsj() -> bool {
    AIH_.load(Ordering::Relaxed)
}

pub fn wjp(iq: bool) {
    AIH_.store(iq, Ordering::Relaxed);
    if !iq {
        
        *PR_.lock() = StickyState::Iy;
        *PQ_.lock() = StickyState::Iy;
        *PS_.lock() = StickyState::Iy;
    }
    crate::serial_println!("[A11Y] Sticky keys: {}", if iq { "ON" } else { "OFF" });
}

pub fn pud() {
    let jwk = AIH_.load(Ordering::Relaxed);
    wjp(!jwk);
}



pub fn ibv(hrs: StickyModifier) -> bool {
    if !dsj() { return false; }
    let wto = match hrs {
        StickyModifier::Wd => &PR_,
        StickyModifier::Vh => &PQ_,
        StickyModifier::Yv => &PS_,
    };
    let mut g = wto.lock();
    *g = match *g {
        StickyState::Iy => StickyState::Aco,
        StickyState::Aco => StickyState::Bln,
        StickyState::Bln => StickyState::Iy,
    };
    true
}


pub fn wuh() {
    if !dsj() { return; }
    let mut db = PR_.lock();
    if *db == StickyState::Aco { *db = StickyState::Iy; }
    drop(db);
    let mut bdj = PQ_.lock();
    if *bdj == StickyState::Aco { *bdj = StickyState::Iy; }
    drop(bdj);
    let mut acn = PS_.lock();
    if *acn == StickyState::Aco { *acn = StickyState::Iy; }
}


pub fn jbu(hrs: StickyModifier) -> bool {
    if !dsj() { return false; }
    let g = match hrs {
        StickyModifier::Wd => *PR_.lock(),
        StickyModifier::Vh => *PQ_.lock(),
        StickyModifier::Yv => *PS_.lock(),
    };
    g != StickyState::Iy
}


pub fn ytx(hrs: StickyModifier) -> StickyState {
    if !dsj() { return StickyState::Iy; }
    match hrs {
        StickyModifier::Wd => *PR_.lock(),
        StickyModifier::Vh => *PQ_.lock(),
        StickyModifier::Yv => *PS_.lock(),
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum StickyModifier {
    Wd,
    Vh,
    Yv,
}



#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum MouseSpeed {
    Ayw = 0,
    M = 1,
    Arz = 2,
    Bak = 3,
}

impl MouseSpeed {
    pub fn ckp(p: u8) -> Self {
        match p {
            0 => MouseSpeed::Ayw,
            1 => MouseSpeed::M,
            2 => MouseSpeed::Arz,
            3 => MouseSpeed::Bak,
            _ => MouseSpeed::M,
        }
    }

    pub fn cu(&self) -> &'static str {
        match self {
            MouseSpeed::Ayw => "Slow",
            MouseSpeed::M => "Normal",
            MouseSpeed::Arz => "Fast",
            MouseSpeed::Bak => "Very Fast",
        }
    }

    
    pub fn uqp(&self) -> (i32, i32) {
        match self {
            MouseSpeed::Ayw => (1, 2),       
            MouseSpeed::M => (1, 1),      
            MouseSpeed::Arz => (3, 2),        
            MouseSpeed::Bak => (2, 1),    
        }
    }
}

pub fn gig() -> MouseSpeed {
    MouseSpeed::ckp(VI_.load(Ordering::Relaxed))
}

pub fn znm(ig: MouseSpeed) {
    VI_.store(ig as u8, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Mouse speed: {}", ig.cu());
}

pub fn niz() {
    let cv = VI_.load(Ordering::Relaxed);
    let next = (cv + 1) % 4;
    VI_.store(next, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Mouse speed: {}", MouseSpeed::ckp(next).cu());
}


pub fn qjz(dx: i32, bg: i32) -> (i32, i32) {
    let (num, bzd) = gig().uqp();
    ((dx * num) / bzd, (bg * num) / bzd)
}






pub fn wts() -> alloc::string::String {
    use alloc::string::String;
    use alloc::format;
    let mut ek: alloc::vec::Vec<&str> = alloc::vec::Vec::new();
    if edv() { ek.push("HC"); }
    if dsj() {
        ek.push("SK");
    }
    let fs = gid();
    if fs != FontSize::Bc {
        
    }
    if ek.is_empty() && fs == FontSize::Bc {
        return String::new();
    }
    let mut e = String::from("[");
    for (a, ai) in ek.iter().cf() {
        if a > 0 { e.push(' '); }
        e.t(ai);
    }
    if fs != FontSize::Bc {
        if !ek.is_empty() { e.push(' '); }
        e.t("F:");
        e.t(fs.cu());
    }
    e.push(']');
    e
}
