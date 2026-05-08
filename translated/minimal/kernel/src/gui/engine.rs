










use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicI32, Ordering};
use spin::Mutex;






const BJB_: u64 = 16_666;


static GL_: AtomicU64 = AtomicU64::new(3_000_000_000); 


static AUM_: AtomicU64 = AtomicU64::new(0);
static BAU_: AtomicU64 = AtomicU64::new(0);
static ARS_: AtomicU64 = AtomicU64::new(0);


pub fn igw() {
    
    let freq = crate::cpu::hac();
    GL_.store(freq, Ordering::SeqCst);
    crate::serial_println!("[GUI] Frame timing init: TSC {} MHz", freq / 1_000_000);
}


#[inline]
fn pog(gx: u64) -> u64 {
    let freq = GL_.load(Ordering::Relaxed);
    if freq == 0 { return 0; }
    (gx * 1_000_000) / freq
}


#[inline]
pub fn yy() -> u64 {
    pog(ey())
}


#[inline]
fn ey() -> u64 {
    crate::arch::timestamp()
}


pub fn rch(frame_start_us: u64) {
    let bb = yy().saturating_sub(frame_start_us);
    
    if bb < BJB_ {
        let target = frame_start_us + BJB_;
        let mut dif = 0u32;
        while yy() < target {
            dif += 1;
            if dif >= 2_000_000 { break; } 
            core::hint::spin_loop();
        }
    }
    
    
    let count = AUM_.fetch_add(1, Ordering::Relaxed);
    let cy = yy();
    let last = BAU_.load(Ordering::Relaxed);
    if cy - last >= 1_000_000 {
        ARS_.store(count, Ordering::Relaxed);
        AUM_.store(0, Ordering::Relaxed);
        BAU_.store(cy, Ordering::Relaxed);
    }
}


pub fn fyp() -> u64 {
    ARS_.load(Ordering::Relaxed)
}






static BCY_: AtomicBool = AtomicBool::new(false);
static BCZ_: AtomicBool = AtomicBool::new(false);
static CKK_: AtomicBool = AtomicBool::new(false);
static BDA_: AtomicBool = AtomicBool::new(false);


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HotkeyAction {
    None,
    
    CloseWindow,      
    SwitchWindow,     
    SnapLeft,         
    SnapRight,        
    Maximize,         
    Minimize,         
    ShowDesktop,      
    
    OpenFileManager,  
    OpenTerminal,     
    OpenRun,          
    
    LockScreen,       
    OpenStartMenu,    
    Screenshot,       
    ToggleDevPanel,   
}


pub mod scancode {
    pub const Wd: u8 = 0x38;
    pub const Xc: u8 = 0x1D;
    pub const Adr: u8 = 0x2A;
    pub const Agj: u8 = 0x5B;  
    pub const Aqg: u8 = 0x0F;
    pub const Ajm: u8 = 0x3E;
    pub const Amg: u8 = 0x4B;
    pub const Aoc: u8 = 0x4D;
    pub const Np: u8 = 0x48;
    pub const Ail: u8 = 0x50;
    pub const Aid: u8 = 0x20;
    pub const Hq: u8 = 0x12;
    pub const T: u8 = 0x14;
    pub const U: u8 = 0x13;
    pub const Th: u8 = 0x26;
    pub const Auw: u8 = 0x01;
    pub const Ajl: u8 = 0x58;
}


pub fn rbr(scancode: u8, pressed: bool) {
    match scancode {
        scancode::Wd => BCY_.store(pressed, Ordering::Relaxed),
        scancode::Xc => BCZ_.store(pressed, Ordering::Relaxed),
        scancode::Adr => CKK_.store(pressed, Ordering::Relaxed),
        scancode::Agj => BDA_.store(pressed, Ordering::Relaxed),
        _ => {}
    }
}


pub fn pzl(scancode: u8) -> HotkeyAction {
    let adf = BCY_.load(Ordering::Relaxed);
    let ctrl = BCZ_.load(Ordering::Relaxed);
    let aw = BDA_.load(Ordering::Relaxed);
    
    
    if adf && scancode == scancode::Ajm {
        return HotkeyAction::CloseWindow;
    }
    
    
    if adf && scancode == scancode::Aqg {
        return HotkeyAction::SwitchWindow;
    }
    
    
    if aw {
        match scancode {
            scancode::Amg => return HotkeyAction::SnapLeft,
            scancode::Aoc => return HotkeyAction::SnapRight,
            scancode::Np => return HotkeyAction::Maximize,
            scancode::Ail => return HotkeyAction::Minimize,
            scancode::Aid => return HotkeyAction::ShowDesktop,
            scancode::Hq => return HotkeyAction::OpenFileManager,
            scancode::T => return HotkeyAction::OpenTerminal,
            scancode::U => return HotkeyAction::OpenRun,
            scancode::Th => return HotkeyAction::LockScreen,
            _ => {}
        }
    }
    
    
    if ctrl && adf && scancode == scancode::T {
        return HotkeyAction::OpenTerminal;
    }
    
    
    if scancode == scancode::Ajl {
        return HotkeyAction::ToggleDevPanel;
    }
    
    HotkeyAction::None
}


static ZU_: AtomicBool = AtomicBool::new(false);

pub fn pzq(scancode: u8, pressed: bool) -> bool {
    if scancode == scancode::Agj {
        if pressed {
            ZU_.store(true, Ordering::Relaxed);
        } else {
            
            if ZU_.load(Ordering::Relaxed) {
                ZU_.store(false, Ordering::Relaxed);
                return true; 
            }
        }
    } else if pressed {
        
        ZU_.store(false, Ordering::Relaxed);
    }
    false
}






#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SnapPosition {
    None,
    Left,       
    Right,      
    Maximized,  
    TopLeft,    
    TopRight,
    BottomLeft,
    BottomRight,
}


pub fn pzc(
    jp: SnapPosition,
    screen_w: u32,
    screen_h: u32,
    taskbar_h: u32,
) -> (i32, i32, u32, u32) {
    let ang = screen_h - taskbar_h;
    
    match jp {
        SnapPosition::Left => (0, 0, screen_w / 2, ang),
        SnapPosition::Right => ((screen_w / 2) as i32, 0, screen_w / 2, ang),
        SnapPosition::Maximized => (0, 0, screen_w, ang),
        SnapPosition::TopLeft => (0, 0, screen_w / 2, ang / 2),
        SnapPosition::TopRight => ((screen_w / 2) as i32, 0, screen_w / 2, ang / 2),
        SnapPosition::BottomLeft => (0, (ang / 2) as i32, screen_w / 2, ang / 2),
        SnapPosition::BottomRight => ((screen_w / 2) as i32, (ang / 2) as i32, screen_w / 2, ang / 2),
        SnapPosition::None => (0, 0, 400, 300), 
    }
}






static AAE_: AtomicBool = AtomicBool::new(false);
static MM_: AtomicI32 = AtomicI32::new(0);


pub fn jig() {
    AAE_.store(true, Ordering::Relaxed);
    MM_.store(0, Ordering::Relaxed);
}


pub fn hfb() {
    MM_.fetch_add(1, Ordering::Relaxed);
}


pub fn pyb() {
    MM_.fetch_sub(1, Ordering::Relaxed);
}


pub fn lwe() -> i32 {
    AAE_.store(false, Ordering::Relaxed);
    MM_.load(Ordering::Relaxed)
}


pub fn dsk() -> bool {
    AAE_.load(Ordering::Relaxed)
}


pub fn jvj() -> i32 {
    MM_.load(Ordering::Relaxed)
}






#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CursorType {
    Arrow,
    Hand,           
    Text,           
    ResizeNS,       
    ResizeEW,       
    ResizeNWSE,     
    ResizeNESW,     
    Move,           
    Wait,           
    Crosshair,      
}

static ARR_: Mutex<CursorType> = Mutex::new(CursorType::Arrow);


pub fn afr(cursor: CursorType) {
    *ARR_.lock() = cursor;
}


pub fn cyk() -> CursorType {
    *ARR_.lock()
}


pub fn qhm(cursor: CursorType) -> &'static [u8; 64] {
    match cursor {
        CursorType::Arrow => &BTG_,
        CursorType::Hand => &BTI_,
        CursorType::Text => &BTO_,
        CursorType::ResizeNS => &BTM_,
        CursorType::ResizeEW => &BTK_,
        CursorType::ResizeNWSE => &BTN_,
        CursorType::ResizeNESW => &BTL_,
        CursorType::Move => &BTJ_,
        CursorType::Wait => &BTP_,
        CursorType::Crosshair => &BTH_,
    }
}


static BTG_: [u8; 64] = [
    2,0,0,0,0,0,0,0,
    2,2,0,0,0,0,0,0,
    2,1,2,0,0,0,0,0,
    2,1,1,2,0,0,0,0,
    2,1,1,1,2,0,0,0,
    2,1,1,1,1,2,0,0,
    2,1,1,2,2,0,0,0,
    2,2,2,0,0,0,0,0,
];

static BTI_: [u8; 64] = [
    0,0,2,2,0,0,0,0,
    0,2,1,1,2,0,0,0,
    0,2,1,1,2,0,0,0,
    0,2,1,1,2,2,2,0,
    2,2,1,1,1,1,1,2,
    2,1,1,1,1,1,1,2,
    2,1,1,1,1,1,1,2,
    0,2,2,2,2,2,2,0,
];

static BTO_: [u8; 64] = [
    0,2,2,2,2,2,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,2,2,2,2,2,0,0,
];

static BTM_: [u8; 64] = [
    0,0,0,2,0,0,0,0,
    0,0,2,1,2,0,0,0,
    0,2,1,1,1,2,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,2,1,1,1,2,0,0,
    0,0,2,1,2,0,0,0,
    0,0,0,2,0,0,0,0,
];

static BTK_: [u8; 64] = [
    0,0,0,0,0,0,0,0,
    0,0,2,0,0,2,0,0,
    0,2,1,2,2,1,2,0,
    2,1,1,1,1,1,1,2,
    0,2,1,2,2,1,2,0,
    0,0,2,0,0,2,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
];

static BTN_: [u8; 64] = [
    2,2,2,2,0,0,0,0,
    2,1,1,2,0,0,0,0,
    2,1,2,0,0,0,0,0,
    2,2,0,2,0,0,0,0,
    0,0,0,0,2,0,2,2,
    0,0,0,0,0,2,1,2,
    0,0,0,0,2,1,1,2,
    0,0,0,0,2,2,2,2,
];

static BTL_: [u8; 64] = [
    0,0,0,0,2,2,2,2,
    0,0,0,0,2,1,1,2,
    0,0,0,0,0,2,1,2,
    0,0,0,0,2,0,2,2,
    2,2,0,2,0,0,0,0,
    2,1,2,0,0,0,0,0,
    2,1,1,2,0,0,0,0,
    2,2,2,2,0,0,0,0,
];

static BTJ_: [u8; 64] = [
    0,0,0,2,0,0,0,0,
    0,0,2,1,2,0,0,0,
    0,0,0,2,0,0,0,0,
    2,2,2,2,2,2,2,0,
    0,0,0,2,0,0,0,0,
    0,0,2,1,2,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,0,0,0,0,0,
];

static BTP_: [u8; 64] = [
    2,2,2,2,2,2,0,0,
    2,1,1,1,1,2,0,0,
    0,2,1,1,2,0,0,0,
    0,0,2,2,0,0,0,0,
    0,0,2,2,0,0,0,0,
    0,2,1,1,2,0,0,0,
    2,1,1,1,1,2,0,0,
    2,2,2,2,2,2,0,0,
];

static BTH_: [u8; 64] = [
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    2,2,2,1,2,2,2,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,0,0,0,0,0,
];






#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NotifyPriority {
    Info,
    Warning,
    Error,
    Success,
}


pub struct Toast {
    pub title: String,
    pub message: String,
    pub priority: NotifyPriority,
    pub created_at: u64,
    pub duration_ms: u64,
    pub progress: Option<u8>, 
}

impl Toast {
    pub fn new(title: &str, message: &str, priority: NotifyPriority) -> Self {
        Self {
            title: String::from(title),
            message: String::from(message),
            priority,
            created_at: yy(),
            duration_ms: 5000,
            progress: None,
        }
    }
    
    pub fn with_duration(mut self, dh: u64) -> Self {
        self.duration_ms = dh;
        self
    }
    
    pub fn with_progress(mut self, bup: u8) -> Self {
        self.progress = Some(bup.min(100));
        self
    }
    
    pub fn is_expired(&self) -> bool {
        let bb = (yy() - self.created_at) / 1000;
        bb >= self.duration_ms
    }
    
    
    pub fn elapsed_ms(&self) -> u64 {
        (yy() - self.created_at) / 1000
    }
    
    
    pub fn opacity(&self) -> u8 {
        let bb = self.elapsed_ms();
        
        if bb < 300 {
            return ((bb * 255) / 300).min(255) as u8;
        }
        
        if self.duration_ms > 500 && bb > self.duration_ms - 500 {
            let ck = self.duration_ms.saturating_sub(bb);
            return ((ck * 255) / 500).min(255) as u8;
        }
        255
    }
    
    pub fn get_color(&self) -> u32 {
        match self.priority {
            NotifyPriority::Info => 0xFF3498DB,    
            NotifyPriority::Warning => 0xFFF39C12, 
            NotifyPriority::Error => 0xFFE74C3C,   
            NotifyPriority::Success => 0xFF27AE60, 
        }
    }
}


static Tv: Mutex<Vec<Toast>> = Mutex::new(Vec::new());
const CIU_: usize = 5;


pub fn osf(title: &str, message: &str, priority: NotifyPriority) {
    let mut ayp = Tv.lock();
    
    
    ayp.retain(|ae| !ae.is_expired());
    
    
    while ayp.len() >= CIU_ {
        ayp.remove(0);
    }
    
    ayp.push(Toast::new(title, message, priority));
}


pub fn qwx(title: &str, message: &str, bup: u8) {
    let mut ayp = Tv.lock();
    
    
    for ae in ayp.iter_mut() {
        if ae.title == title && ae.progress.is_some() {
            ae.progress = Some(bup.min(100));
            ae.message = String::from(message);
            return;
        }
    }
    
    
    ayp.push(Toast::new(title, message, NotifyPriority::Info)
        .with_progress(bup)
        .with_duration(30000)); 
}


pub fn ibr() -> Vec<Toast> {
    let mut ayp = Tv.lock();
    ayp.retain(|ae| !ae.is_expired());
    ayp.clone()
}

impl Clone for Toast {
    fn clone(&self) -> Self {
        Self {
            title: self.title.clone(),
            message: self.message.clone(),
            priority: self.priority,
            created_at: self.created_at,
            duration_ms: self.duration_ms,
            progress: self.progress,
        }
    }
}






static YP_: AtomicBool = AtomicBool::new(false);


#[derive(Clone)]
pub struct Gb {
    pub name: String,
    pub icon: u8,      
    pub action: StartAction,
}

#[derive(Clone, Copy, Debug)]
pub enum StartAction {
    OpenApp(&'static str),
    OpenTerminal,
    OpenFiles,
    OpenSettings,
    OpenAbout,
    Shutdown,
    Restart,
    Lock,
}


pub fn rao() {
    let current = YP_.load(Ordering::Relaxed);
    YP_.store(!current, Ordering::Relaxed);
}


pub fn qai() {
    YP_.store(false, Ordering::Relaxed);
}


pub fn qmy() -> bool {
    YP_.load(Ordering::Relaxed)
}


pub fn ibw() -> Vec<Gb> {
    vec![
        Gb { name: String::from("Terminal"), icon: 0, action: StartAction::OpenTerminal },
        Gb { name: String::from("Files"), icon: 1, action: StartAction::OpenFiles },
        Gb { name: String::from("Settings"), icon: 2, action: StartAction::OpenSettings },
        Gb { name: String::from("About"), icon: 3, action: StartAction::OpenAbout },
        Gb { name: String::from("───────────"), icon: 255, action: StartAction::OpenAbout },
        Gb { name: String::from("Lock"), icon: 4, action: StartAction::Lock },
        Gb { name: String::from("Restart"), icon: 5, action: StartAction::Restart },
        Gb { name: String::from("Shutdown"), icon: 6, action: StartAction::Shutdown },
    ]
}







static KC_: [[u8; 256]; 256] = {
    let mut bs = [[0u8; 256]; 256];
    let mut alpha = 0usize;
    while alpha < 256 {
        let mut value = 0usize;
        while value < 256 {
            bs[alpha][value] = ((value * alpha + 127) / 255) as u8;
            value += 1;
        }
        alpha += 1;
    }
    bs
};


#[inline(always)]
pub fn fji(src: u32, dst: u32) -> u32 {
    let alpha = ((src >> 24) & 0xFF) as usize;
    if alpha == 0 { return dst; }
    if alpha == 255 { return src; }
    
    let sg = 255 - alpha;
    
    let pb = ((src >> 16) & 0xFF) as usize;
    let akl = ((src >> 8) & 0xFF) as usize;
    let cv = (src & 0xFF) as usize;
    
    let qw = ((dst >> 16) & 0xFF) as usize;
    let afb = ((dst >> 8) & 0xFF) as usize;
    let fu = (dst & 0xFF) as usize;
    
    let r = KC_[alpha][pb] as u32 + KC_[sg][qw] as u32;
    let g = KC_[alpha][akl] as u32 + KC_[sg][afb] as u32;
    let b = KC_[alpha][cv] as u32 + KC_[sg][fu] as u32;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}






#[derive(Clone, Copy, Default, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }
    
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w as i32 &&
        self.x + self.w as i32 > other.x &&
        self.y < other.y + other.h as i32 &&
        self.y + self.h as i32 > other.y
    }
    
    pub fn union(&self, other: &Rect) -> Rect {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.w as i32).max(other.x + other.w as i32);
        let y2 = (self.y + self.h as i32).max(other.y + other.h as i32);
        Rect {
            x: x1,
            y: y1,
            w: (x2 - x1) as u32,
            h: (y2 - y1) as u32,
        }
    }
    
    pub fn area(&self) -> u32 {
        self.w * self.h
    }
}


pub struct Ais {
    rects: Vec<Rect>,
    full_redraw: bool,
    screen_w: u32,
    screen_h: u32,
}

impl Ais {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            rects: Vec::with_capacity(64),
            full_redraw: true,
            screen_w: w,
            screen_h: h,
        }
    }
    
    
    pub fn qok(&mut self, rect: Rect) {
        if self.full_redraw { return; }
        if rect.w == 0 || rect.h == 0 { return; }
        
        
        for i in 0..self.rects.len() {
            if self.rects[i].intersects(&rect) {
                
                let duf = self.rects[i].union(&rect);
                let ptx = duf.area() as i64 - 
                    (self.rects[i].area() + rect.area()) as i64;
                
                
                let otw = self.rects[i].area().min(rect.area());
                if ptx < (otw / 2) as i64 {
                    self.rects[i] = duf;
                    return;
                }
            }
        }
        
        
        if self.rects.len() < 64 {
            self.rects.push(rect);
        } else {
            
            self.full_redraw = true;
        }
    }
    
    
    pub fn mark_full(&mut self) {
        self.full_redraw = true;
    }
    
    
    pub fn clear(&mut self) {
        self.rects.clear();
        self.full_redraw = false;
    }
    
    
    pub fn mcw(&self) -> &[Rect] {
        &self.rects
    }
    
    
    pub fn needs_full_redraw(&self) -> bool {
        self.full_redraw
    }
}
