




use core::sync::atomic::{AtomicU8, AtomicBool, Ordering};
use spin::Mutex;






static OK_: AtomicBool = AtomicBool::new(false);


static UM_: AtomicU8 = AtomicU8::new(1);


static TS_: AtomicU8 = AtomicU8::new(0);


static AKD_: AtomicBool = AtomicBool::new(false);


static WR_: AtomicU8 = AtomicU8::new(1);






#[derive(Clone, Copy, PartialEq)]
pub enum StickyState {
    Off,
    Latched,  
    Locked,   
}


static QO_: Mutex<StickyState> = Mutex::new(StickyState::Off);
static QN_: Mutex<StickyState> = Mutex::new(StickyState::Off);
static QP_: Mutex<StickyState> = Mutex::new(StickyState::Off);






pub struct Alb;
impl Alb {
    pub const SM_: u32     = 0xFF000000;
    pub const DW_: u32        = 0xFF000000;
    pub const BNT_: u32      = 0xFF0A0A0A;
    pub const ANM_: u32       = 0xFF1A1A1A;
    pub const Bal: u32        = 0xFFFFFF00; 
    pub const Bbn: u32      = 0xFFFFFFFF; 
    pub const Bdq: u32       = 0xFFCCCCCC;
    pub const Azk: u32          = 0xFF888888;
    pub const Bdb: u32         = 0xFF666666;
    pub const Awr: u32          = 0xFF444444;
    pub const DFW_: u32    = 0xFFFF8800; 
    pub const DFT_: u32   = 0xFFFF0000; 
    pub const DFU_: u32    = 0xFF00CCFF; 
    pub const AB_: u32   = 0xFFFFFFFF; 
    pub const O_: u32 = 0xFFCCCCCC;
    pub const QV_: u32    = 0xFFFFFF00; 
    pub const DFF_: u32      = 0xFF000000;
    pub const ENZ_: u32  = 0xFFFFFFFF;
    pub const ELU_: u32      = 0xFF1A1A1A;
    pub const ASQ_: u32        = 0xFF0A0A0A;
    pub const BCK_: u32        = 0xFF0A0A0A;
    pub const BCL_: u32     = 0xFF333300;
    pub const AOE_: u32      = 0xFFFF0000;
    pub const AOF_: u32   = 0xFFFFFF00;
    pub const AOG_: u32   = 0xFF00FF00;
}






pub fn btq() -> bool {
    OK_.load(Ordering::Relaxed)
}

pub fn qwa(enabled: bool) {
    OK_.store(enabled, Ordering::Relaxed);
    crate::serial_println!("[A11Y] High contrast: {}", if enabled { "ON" } else { "OFF" });
}

pub fn gzg() {
    let fey = OK_.load(Ordering::Relaxed);
    OK_.store(!fey, Ordering::Relaxed);
    crate::serial_println!("[A11Y] High contrast toggled: {}", if !fey { "ON" } else { "OFF" });
}



#[inline]
pub fn jsw(normal: u32, ads: u32) -> u32 {
    if OK_.load(Ordering::Relaxed) { ads } else { normal }
}



#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum FontSize {
    Small = 0,
    Medium = 1,
    Large = 2,
    ExtraLarge = 3,
}

impl FontSize {
    pub fn atw(v: u8) -> Self {
        match v {
            0 => FontSize::Small,
            1 => FontSize::Medium,
            2 => FontSize::Large,
            3 => FontSize::ExtraLarge,
            _ => FontSize::Medium,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FontSize::Small => "Small",
            FontSize::Medium => "Medium",
            FontSize::Large => "Large",
            FontSize::ExtraLarge => "XL",
        }
    }

    
    
    
    pub fn ceh(&self) -> (u32, u32) {
        match self {
            FontSize::Small => (7, 8),
            FontSize::Medium => (1, 1),
            FontSize::Large => (5, 4),
            FontSize::ExtraLarge => (3, 2),
        }
    }
}

pub fn cyn() -> FontSize {
    FontSize::atw(UM_.load(Ordering::Relaxed))
}

pub fn qvx(size: FontSize) {
    UM_.store(size as u8, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Font size: {}", size.label());
}

pub fn hqf() {
    let current = UM_.load(Ordering::Relaxed);
    let next = (current + 1) % 4;
    UM_.store(next, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Font size: {}", FontSize::atw(next).label());
}



#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum CursorSize {
    Small = 0,
    Medium = 1,
    Large = 2,
}

impl CursorSize {
    pub fn atw(v: u8) -> Self {
        match v {
            0 => CursorSize::Small,
            1 => CursorSize::Medium,
            2 => CursorSize::Large,
            _ => CursorSize::Small,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CursorSize::Small => "Small",
            CursorSize::Medium => "Medium",
            CursorSize::Large => "Large",
        }
    }

    
    pub fn scale(&self) -> u32 {
        match self {
            CursorSize::Small => 1,
            CursorSize::Medium => 2,
            CursorSize::Large => 3,
        }
    }
}

pub fn cyl() -> CursorSize {
    CursorSize::atw(TS_.load(Ordering::Relaxed))
}

pub fn qvq(size: CursorSize) {
    TS_.store(size as u8, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Cursor size: {}", size.label());
}

pub fn hqe() {
    let current = TS_.load(Ordering::Relaxed);
    let next = (current + 1) % 3;
    TS_.store(next, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Cursor size: {}", CursorSize::atw(next).label());
}



pub fn bnc() -> bool {
    AKD_.load(Ordering::Relaxed)
}

pub fn opl(enabled: bool) {
    AKD_.store(enabled, Ordering::Relaxed);
    if !enabled {
        
        *QO_.lock() = StickyState::Off;
        *QN_.lock() = StickyState::Off;
        *QP_.lock() = StickyState::Off;
    }
    crate::serial_println!("[A11Y] Sticky keys: {}", if enabled { "ON" } else { "OFF" });
}

pub fn jnd() {
    let fey = AKD_.load(Ordering::Relaxed);
    opl(!fey);
}



pub fn eak(modifier: StickyModifier) -> bool {
    if !bnc() { return false; }
    let ows = match modifier {
        StickyModifier::Ctrl => &QO_,
        StickyModifier::Alt => &QN_,
        StickyModifier::Shift => &QP_,
    };
    let mut state = ows.lock();
    *state = match *state {
        StickyState::Off => StickyState::Latched,
        StickyState::Latched => StickyState::Locked,
        StickyState::Locked => StickyState::Off,
    };
    true
}


pub fn oxi() {
    if !bnc() { return; }
    let mut ctrl = QO_.lock();
    if *ctrl == StickyState::Latched { *ctrl = StickyState::Off; }
    drop(ctrl);
    let mut adf = QN_.lock();
    if *adf == StickyState::Latched { *adf = StickyState::Off; }
    drop(adf);
    let mut no = QP_.lock();
    if *no == StickyState::Latched { *no = StickyState::Off; }
}


pub fn erv(modifier: StickyModifier) -> bool {
    if !bnc() { return false; }
    let state = match modifier {
        StickyModifier::Ctrl => *QO_.lock(),
        StickyModifier::Alt => *QN_.lock(),
        StickyModifier::Shift => *QP_.lock(),
    };
    state != StickyState::Off
}


pub fn qio(modifier: StickyModifier) -> StickyState {
    if !bnc() { return StickyState::Off; }
    match modifier {
        StickyModifier::Ctrl => *QO_.lock(),
        StickyModifier::Alt => *QN_.lock(),
        StickyModifier::Shift => *QP_.lock(),
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum StickyModifier {
    Ctrl,
    Alt,
    Shift,
}



#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum MouseSpeed {
    Slow = 0,
    Normal = 1,
    Fast = 2,
    VeryFast = 3,
}

impl MouseSpeed {
    pub fn atw(v: u8) -> Self {
        match v {
            0 => MouseSpeed::Slow,
            1 => MouseSpeed::Normal,
            2 => MouseSpeed::Fast,
            3 => MouseSpeed::VeryFast,
            _ => MouseSpeed::Normal,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MouseSpeed::Slow => "Slow",
            MouseSpeed::Normal => "Normal",
            MouseSpeed::Fast => "Fast",
            MouseSpeed::VeryFast => "Very Fast",
        }
    }

    
    pub fn multiplier(&self) -> (i32, i32) {
        match self {
            MouseSpeed::Slow => (1, 2),       
            MouseSpeed::Normal => (1, 1),      
            MouseSpeed::Fast => (3, 2),        
            MouseSpeed::VeryFast => (2, 1),    
        }
    }
}

pub fn cyq() -> MouseSpeed {
    MouseSpeed::atw(WR_.load(Ordering::Relaxed))
}

pub fn qwi(speed: MouseSpeed) {
    WR_.store(speed as u8, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Mouse speed: {}", speed.label());
}

pub fn hqg() {
    let current = WR_.load(Ordering::Relaxed);
    let next = (current + 1) % 4;
    WR_.store(next, Ordering::Relaxed);
    crate::serial_println!("[A11Y] Mouse speed: {}", MouseSpeed::atw(next).label());
}


pub fn jxe(dx: i32, ad: i32) -> (i32, i32) {
    let (num, anz) = cyq().multiplier();
    ((dx * num) / anz, (ad * num) / anz)
}






pub fn oww() -> alloc::string::String {
    use alloc::string::String;
    use alloc::format;
    let mut au: alloc::vec::Vec<&str> = alloc::vec::Vec::new();
    if btq() { au.push("HC"); }
    if bnc() {
        au.push("SK");
    }
    let fs = cyn();
    if fs != FontSize::Medium {
        
    }
    if au.is_empty() && fs == FontSize::Medium {
        return String::new();
    }
    let mut j = String::from("[");
    for (i, aa) in au.iter().enumerate() {
        if i > 0 { j.push(' '); }
        j.push_str(aa);
    }
    if fs != FontSize::Medium {
        if !au.is_empty() { j.push(' '); }
        j.push_str("F:");
        j.push_str(fs.label());
    }
    j.push(']');
    j
}
