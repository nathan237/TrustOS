









use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;
use crate::framebuffer::{self, B_, G_, AX_, R_, NE_};
use crate::apps::text_editor::{EditorState, bvh};
use core::sync::atomic::{AtomicBool, Ordering};
use crate::math::ra;


#[inline]
fn ads(normal: u32, hc_replacement: u32) -> u32 {
    crate::accessibility::jsw(normal, hc_replacement)
}


static NW_: AtomicBool = AtomicBool::new(false);







use core::sync::atomic::{AtomicPtr, AtomicU64};





static ANK_: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());



static AAQ_: Mutex<Option<Vec<u32>>> = Mutex::new(None);
static AAR_: Mutex<Option<Vec<u32>>> = Mutex::new(None);


static AAU_: AtomicU64 = AtomicU64::new(0);

static AAT_: AtomicU64 = AtomicU64::new(0);

static AAS_: AtomicU64 = AtomicU64::new(0);

static ANQ_: AtomicBool = AtomicBool::new(false);

static GQ_: AtomicBool = AtomicBool::new(false);


fn kbv(_arg: u64) -> i32 {
    crate::serial_println!("[BG-THREAD] Started on CPU {}", crate::cpu::smp::bll());
    GQ_.store(true, Ordering::SeqCst);
    
    let width = framebuffer::width();
    let height = framebuffer::height();
    let stride = width as usize;
    let ate = stride * height as usize;
    
    if ate == 0 {
        crate::serial_println!("[BG-THREAD] ERROR: zero framebuffer size");
        GQ_.store(false, Ordering::SeqCst);
        return 1;
    }
    
    let mut hzx: u64 = 0;
    
    while !ANQ_.load(Ordering::Relaxed) {
        
        let ready = AAT_.load(Ordering::Relaxed);
        let fog = AAS_.load(Ordering::Relaxed);
        if ready > fog + 1 {
            
            crate::thread::ajc();
            continue;
        }
        
        
        let fxu = AAU_.load(Ordering::Acquire);
        let jrr = fxu == 1; 
        
        
        let buf_ptr = if jrr {
            let jg = AAQ_.lock();
            match &*jg {
                Some(buf) => buf.as_ptr() as *mut u32,
                None => { crate::thread::ajc(); continue; }
            }
        } else {
            let jg = AAR_.lock();
            match &*jg {
                Some(buf) => buf.as_ptr() as *mut u32,
                None => { crate::thread::ajc(); continue; }
            }
        };
        
        
        let hrp = ANK_.load(Ordering::Acquire);
        if hrp.is_null() {
            crate::thread::ajc();
            continue;
        }
        
        
        
        
        let desktop = unsafe { &mut *(hrp as *mut Desktop) };
        
        
        framebuffer::odv(buf_ptr);
        
        
        let okh = desktop.desktop_tier;
        desktop.desktop_tier = DesktopTier::Full;
        
        
        desktop.frame_count += 1;
        desktop.draw_background();
        
        
        desktop.desktop_tier = okh;
        
        
        framebuffer::ogf();
        
        
        let niz = if jrr { 0u64 } else { 1u64 };
        AAU_.store(niz, Ordering::Release);
        hzx += 1;
        AAT_.store(hzx, Ordering::Release);
    }
    
    crate::serial_println!("[BG-THREAD] Stopped");
    GQ_.store(false, Ordering::SeqCst);
    0
}



fn moz(desktop: &mut Desktop) {
    let width = framebuffer::width() as usize;
    let height = framebuffer::height() as usize;
    let size = width * height;
    if size == 0 { return; }
    
    
    let mut bey = alloc::vec::Vec::new();
    let mut bez = alloc::vec::Vec::new();
    if bey.try_reserve_exact(size).is_err() || bez.try_reserve_exact(size).is_err() {
        crate::serial_println!("[BG-THREAD] Failed to allocate double buffers ({} KB each)", size * 4 / 1024);
        return;
    }
    bey.resize(size, 0xFF010200u32); 
    bez.resize(size, 0xFF010200u32);
    *AAQ_.lock() = Some(bey);
    *AAR_.lock() = Some(bez);
    crate::serial_println!("[BG-THREAD] Double buffers allocated: 2 × {} KB", size * 4 / 1024);
    
    
    let ptr = desktop as *mut Desktop as *mut u8;
    ANK_.store(ptr, Ordering::Release);
    
    
    ANQ_.store(false, Ordering::SeqCst);
    let hdl = crate::thread::dzu("bg-render", kbv, 0);
    crate::serial_println!("[BG-THREAD] Spawned render thread (tid={})", hdl);
}




fn eha() -> bool {
    let ready = AAT_.load(Ordering::Acquire);
    let fog = AAS_.load(Ordering::Relaxed);
    if ready <= fog {
        return false; 
    }
    
    let fxu = AAU_.load(Ordering::Acquire);
    let jhn = if fxu == 0 { AAQ_.lock() } else { AAR_.lock() };
    
    if let Some(ref src_buf) = *jhn {
        let ptr = framebuffer::lyj();
        if !ptr.is_null() {
            let len = src_buf.len();
            unsafe {
                core::ptr::copy_nonoverlapping(src_buf.as_ptr(), ptr, len);
            }
        }
    }
    drop(jhn);
    
    AAS_.store(ready, Ordering::Release);
    true
}





static AZQ_: Mutex<Option<String>> = Mutex::new(None);

static AZR_: Mutex<Option<Vec<String>>> = Mutex::new(None);

static VQ_: AtomicBool = AtomicBool::new(false);





static ABC_: Mutex<Option<String>> = Mutex::new(None);

static ANT_: Mutex<Option<Result<(String, u16, Vec<(String, String)>, Vec<u8>), String>>> = Mutex::new(None);

static ST_: AtomicBool = AtomicBool::new(false);


fn hir(_arg: u64) -> i32 {
    let url = {
        let mut pending = ABC_.lock();
        pending.take()
    };
    let url = match url {
        Some(iy) => iy,
        None => {
            ST_.store(false, Ordering::SeqCst);
            return 0;
        }
    };

    
    let ti = crate::browser::normalize_url(&url, "");
    crate::serial_println!("[BROWSER-BG] Fetching: {}", ti);

    let result = if ti.starts_with("https://") {
        match crate::netstack::https::get(&ti) {
            Ok(r) => Ok((ti, r.status_code, r.headers, r.body)),
            Err(e) => Err(alloc::format!("HTTPS error: {}", e)),
        }
    } else {
        match crate::netstack::http::get(&ti) {
            Ok(r) => Ok((ti, r.status_code, r.headers, r.body)),
            Err(e) => Err(alloc::format!("Network error: {}", e)),
        }
    };

    {
        let mut gir = ANT_.lock();
        *gir = Some(result);
    }
    ST_.store(false, Ordering::SeqCst);
    0
}


fn muv(_arg: u64) -> i32 {
    
    let query = {
        let mut pending = AZQ_.lock();
        pending.take()
    };
    let query = match query {
        Some(q) => q,
        None => {
            VQ_.store(false, Ordering::SeqCst);
            return 0;
        }
    };

    
    crate::shell::fcj(); 
    crate::shell::DL_.store(true, Ordering::SeqCst);
    crate::shell::aav(&query);
    crate::shell::DL_.store(false, Ordering::SeqCst);
    let captured = crate::shell::fcj();

    
    let mut lines = Vec::new();
    for line in captured.lines() {
        lines.push(String::from(line));
    }
    {
        let mut result = AZR_.lock();
        *result = Some(lines);
    }
    VQ_.store(false, Ordering::SeqCst);
    0
}








const SM_: u32 = 0xFF050606;          
const DW_: u32 = 0xFF070B09;             
const BNT_: u32 = 0xFF0A0F0C;           
const ANM_: u32 = 0xFF0D1310;            


const I_: u32 = 0xFF00FF66;       
const AH_: u32 = 0xFF00CC55;     
const BM_: u32 = 0xFF00AA44;      
const Y_: u32 = 0xFF008844;         
const BJ_: u32 = 0xFF006633;        
const Q_: u32 = 0xFF003B1A;         


const EP_: u32 = 0xFFB0B2B0;       
const GR_: u32 = 0xFF8C8E8C;          
const AW_: u32 = 0xFF606260;          
const AP_: u32 = 0xFF3A3C3A;        


const GN_: u32 = 0xFFFFD166;        
const DJ_: u32 = 0xFFFF5555;          
const RQ_: u32 = 0xFF4ECDC4;         


const DFF_: u32 = 0xFF0A0F0C;
const EOA_: u32 = 0xFF070B09;
const ELW_: u32 = 0xFF080C09;
const ELX_: u32 = 0xFF060908;
const ELV_: u32 = Y_;


const ASQ_: u32 = 0xFF060908;
const BUW_: u32 = 0xFF0D1310;
const DNG_: u32 = 0xFF101815;


const BCK_: u32 = 0xFF080C09;
const BCL_: u32 = 0xFF0D1310;
const CJR_: u32 = 0xFF1A2A20;
const DXO_: u32 = 0xFF1A2A20;


const AOE_: u32 = 0xFF3A2828;
const DHS_: u32 = 0xFFFF5555;
const AOF_: u32 = 0xFF2A2A20;
const DHT_: u32 = 0xFFFFD166;
const AOG_: u32 = 0xFF283028;
const DHU_: u32 = 0xFF00CC55;


const AB_: u32 = 0xFFE0E8E4;
const O_: u32 = 0xFF8A9890;
const QV_: u32 = 0xFF00CC55;


const DBD_: u32 = ASQ_;
const EKX_: u32 = BUW_;
const DHZ_: u32 = AOE_;
const DIA_: u32 = AOF_;
const DHX_: u32 = AOG_;
const DKL_: u32 = BCK_;
const DKN_: u32 = BCL_;
const DKM_: u32 = CJR_;
const DMS_: u32 = SM_;
const DMR_: u32 = 0xFF020303;



const SG_: u32 = 48;
const SH_: u32 = 24;
const SI_: u32 = 6;
const SJ_: u32 = 10;
const SE_: u32 = 28;
const SF_: u32 = 64;


#[inline(always)]
#[allow(non_snake_case)]
fn V_() -> u32 { crate::graphics::scaling::ddt(SG_) }
#[inline(always)]
#[allow(non_snake_case)]
fn J_() -> u32 { crate::graphics::scaling::ddt(SH_) }
#[inline(always)]
#[allow(non_snake_case)]
fn DFG_() -> u32 { crate::graphics::scaling::ddt(SI_) }
#[inline(always)]
#[allow(non_snake_case)]
fn EOC_() -> u32 { crate::graphics::scaling::ddt(SJ_) }
#[inline(always)]
#[allow(non_snake_case)]
fn BV_() -> u32 { crate::graphics::scaling::ddt(SE_) }
#[inline(always)]
#[allow(non_snake_case)]
fn BW_() -> u32 { crate::graphics::scaling::ddt(SF_) }


const DPD_: u8 = 8;






static AAF_: Mutex<bool> = Mutex::new(true);
static GP_: Mutex<f32> = Mutex::new(1.0); 


const BMQ_: u32 = 12;      
const BMN_: u32 = 8;      
const BMP_: u32 = 10;  
const BMO_: u32 = 10;  


#[derive(Clone, Copy, PartialEq)]
pub enum AnimationState {
    None,
    Opening,      
    Closing,      
    Minimizing,   
    Maximizing,   
    Restoring,    
}


#[derive(Clone)]
pub struct WindowAnimation {
    pub state: AnimationState,
    pub progress: f32,           
    pub start_x: i32,
    pub start_y: i32,
    pub start_width: u32,
    pub start_height: u32,
    pub target_x: i32,
    pub target_y: i32,
    pub target_width: u32,
    pub target_height: u32,
    pub alpha: f32,              
}

impl WindowAnimation {
    pub fn new() -> Self {
        Self {
            state: AnimationState::None,
            progress: 0.0,
            start_x: 0,
            start_y: 0,
            start_width: 0,
            start_height: 0,
            target_x: 0,
            target_y: 0,
            target_width: 0,
            target_height: 0,
            alpha: 1.0,
        }
    }
    
    
    pub fn start_open(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.state = AnimationState::Opening;
        self.progress = 0.0;
        
        self.start_x = x + width as i32 / 2 - 10;
        self.start_y = y + height as i32 / 2 - 10;
        self.start_width = 20;
        self.start_height = 20;
        self.target_x = x;
        self.target_y = y;
        self.target_width = width;
        self.target_height = height;
        self.alpha = 0.0;
    }
    
    
    pub fn start_close(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.state = AnimationState::Closing;
        self.progress = 0.0;
        self.start_x = x;
        self.start_y = y;
        self.start_width = width;
        self.start_height = height;
        
        self.target_x = x + width as i32 / 2 - 10;
        self.target_y = y + height as i32 / 2 - 10;
        self.target_width = 20;
        self.target_height = 20;
        self.alpha = 1.0;
    }
    
    
    pub fn start_minimize(&mut self, x: i32, y: i32, width: u32, height: u32, gyd: i32, bwh: i32) {
        self.state = AnimationState::Minimizing;
        self.progress = 0.0;
        self.start_x = x;
        self.start_y = y;
        self.start_width = width;
        self.start_height = height;
        self.target_x = gyd;
        self.target_y = bwh;
        self.target_width = 48;
        self.target_height = 32;
        self.alpha = 1.0;
    }
    
    
    pub fn start_maximize(&mut self, x: i32, y: i32, width: u32, height: u32, max_w: u32, dua: u32) {
        self.state = AnimationState::Maximizing;
        self.progress = 0.0;
        self.start_x = x;
        self.start_y = y;
        self.start_width = width;
        self.start_height = height;
        self.target_x = 0;
        self.target_y = 0;
        self.target_width = max_w;
        self.target_height = dua - V_();
        self.alpha = 1.0;
    }
    
    
    pub fn start_restore(&mut self, curr_x: i32, curr_y: i32, curr_w: u32, curr_h: u32,
                         saved_x: i32, saved_y: i32, saved_w: u32, saved_h: u32) {
        self.state = AnimationState::Restoring;
        self.progress = 0.0;
        self.start_x = curr_x;
        self.start_y = curr_y;
        self.start_width = curr_w;
        self.start_height = curr_h;
        self.target_x = saved_x;
        self.target_y = saved_y;
        self.target_width = saved_w;
        self.target_height = saved_h;
        self.alpha = 1.0;
    }
    
    
    pub fn update(&mut self) -> bool {
        if self.state == AnimationState::None {
            return false;
        }
        
        let speed = *GP_.lock();
        let yq = match self.state {
            AnimationState::Opening => BMQ_,
            AnimationState::Closing => BMN_,
            AnimationState::Minimizing => BMP_,
            AnimationState::Maximizing | AnimationState::Restoring => BMO_,
            AnimationState::None => return false,
        };
        
        let step = speed / yq as f32;
        self.progress += step;
        
        
        match self.state {
            AnimationState::Opening => {
                self.alpha = huv(self.progress);
            }
            AnimationState::Closing | AnimationState::Minimizing => {
                self.alpha = 1.0 - huu(self.progress);
            }
            _ => {}
        }
        
        if self.progress >= 1.0 {
            self.progress = 1.0;
            let kwg = self.state;
            self.state = AnimationState::None;
            return kwg == AnimationState::Closing;
        }
        
        false 
    }
    
    
    pub fn get_current(&self) -> (i32, i32, u32, u32, f32) {
        let t = match self.state {
            AnimationState::Opening | AnimationState::Restoring => lnh(self.progress),
            AnimationState::Closing => lng(self.progress),
            AnimationState::Minimizing => huu(self.progress),
            AnimationState::Maximizing => huv(self.progress),
            AnimationState::None => 1.0,
        };
        
        let x = ijx(self.start_x, self.target_x, t);
        let y = ijx(self.start_y, self.target_y, t);
        let w = ijy(self.start_width, self.target_width, t);
        let h = ijy(self.start_height, self.target_height, t);
        
        (x, y, w, h, self.alpha)
    }
    
    
    pub fn is_animating(&self) -> bool {
        self.state != AnimationState::None
    }
}






fn ijx(a: i32, b: i32, t: f32) -> i32 {
    (a as f32 + (b - a) as f32 * t) as i32
}


fn ijy(a: u32, b: u32, t: f32) -> u32 {
    if a > b {
        (a as f32 - (a - b) as f32 * t) as u32
    } else {
        (a as f32 + (b - a) as f32 * t) as u32
    }
}


fn huv(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t)
}


fn huu(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t
}


fn lnh(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let hw: f32 = 1.70158;
    let bfc = hw + 1.0;
    let ebv = t - 1.0;
    1.0 + bfc * ebv * ebv * ebv + hw * ebv * ebv
}


fn lng(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let hw: f32 = 1.70158;
    let bfc = hw + 1.0;
    bfc * t * t * t - hw * t * t
}


pub fn awb() -> bool {
    *AAF_.lock()
}


pub fn fae(enabled: bool) {
    *AAF_.lock() = enabled;
    crate::serial_println!("[ANIM] Animations {}", if enabled { "ENABLED" } else { "DISABLED" });
}


pub fn pkq() {
    let mut enabled = AAF_.lock();
    *enabled = !*enabled;
    crate::serial_println!("[ANIM] Animations {}", if *enabled { "ENABLED" } else { "DISABLED" });
}


pub fn jew(speed: f32) {
    *GP_.lock() = speed.clamp(0.25, 4.0);
    crate::serial_println!("[ANIM] Speed set to {}x", speed);
}


pub fn dqn() -> f32 {
    *GP_.lock()
}


static BDP_: Mutex<u32> = Mutex::new(1);


#[derive(Clone)]
pub struct Aa {
    pub label: String,
    pub action: ContextAction,
}


#[derive(Clone, Copy, PartialEq)]
pub enum ContextAction {
    Open,
    OpenWith,
    Delete,
    Rename,
    Properties,
    Refresh,
    NewFile,
    NewFolder,
    CopyPath,
    Cut,
    Copy,
    Paste,
    ViewLargeIcons,
    ViewSmallIcons,
    ViewList,
    SortByName,
    SortByDate,
    SortBySize,
    Personalize,
    TerminalHere,
    Cancel,
}




#[derive(Clone)]
pub struct CellPixels {
    pub pixels: [u32; 128],  
}

impl CellPixels {
    pub const fn hhz() -> Self {
        CellPixels { pixels: [0; 128] }
    }

    
    pub fn lzf(c: char, color: u32) -> Self {
        let du = crate::framebuffer::font::ol(c);
        let mut p = [0u32; 128];
        for row in 0..16 {
            let bits = du[row];
            for bf in 0..8u8 {
                if bits & (0x80 >> bf) != 0 {
                    p[row * 8 + bf as usize] = color;
                }
            }
        }
        CellPixels { pixels: p }
    }

    
    #[inline]
    pub fn set(&mut self, x: u8, y: u8, color: u32) {
        if x < 8 && y < 16 {
            self.pixels[y as usize * 8 + x as usize] = color;
        }
    }

    
    #[inline]
    pub fn get(&self, x: u8, y: u8) -> u32 {
        if x < 8 && y < 16 { self.pixels[y as usize * 8 + x as usize] } else { 0 }
    }

    
    pub fn fill(&mut self, color: u32) {
        self.pixels = [color; 128];
    }
}






pub struct MatrixProjection {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,  
    pub active: bool,
}

impl MatrixProjection {
    pub const fn empty() -> Self {
        MatrixProjection {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            pixels: Vec::new(),
            active: false,
        }
    }

    
    pub fn mcl(width: u32, height: u32) -> Vec<u32> {
        let w = width as usize;
        let h = height as usize;
        let mut pixels = vec![0u32; w * h];

        for o in 0..h {
            for p in 0..w {
                
                let iy = p as f32 / w as f32;
                let v = o as f32 / h as f32;
                
                let cx = iy * 2.0 - 1.0;
                let u = v * 2.0 - 1.0;

                
                
                fn ra(x: f32) -> f32 {
                    if x <= 0.0 { return 0.0; }
                    let mut uc = x * 0.5;
                    uc = 0.5 * (uc + x / uc);
                    uc = 0.5 * (uc + x / uc);
                    uc
                }
                let d = ra(cx * cx + u * u); 

                
                fn eu(x: f32) -> f32 {
                    
                    let x = x % 6.2832;
                    let x = if x > 3.1416 { x - 6.2832 } else if x < -3.1416 { x + 6.2832 } else { x };
                    
                    if x < 0.0 {
                        1.27323954 * x + 0.405284735 * x * x
                    } else {
                        1.27323954 * x - 0.405284735 * x * x
                    }
                }

                let afq = eu(iy * 10.0 + v * 6.0) * 0.5 + 0.5;
                let azn = eu(d * 12.0 - v * 4.0) * 0.5 + 0.5;
                let dyf = eu((cx + u) * 8.0) * 0.5 + 0.5;

                let mut r = (afq * 0.5 + azn * 0.3 + dyf * 0.2).min(1.0);
                let mut g = (azn * 0.5 + dyf * 0.3 + afq * 0.2).min(1.0);
                let mut b = (dyf * 0.5 + afq * 0.3 + azn * 0.2).min(1.0);

                

                
                let ekd = cx.abs() + u.abs();
                if ekd < 0.35 {
                    let t = 1.0 - ekd / 0.35;
                    r = r * (1.0 - t * 0.8) + 0.1 * t;
                    g = g * (1.0 - t * 0.5) + 1.0 * t * 0.5 + g * t * 0.5;
                    b = b * (1.0 - t * 0.5) + 1.0 * t * 0.5 + b * t * 0.5;
                }

                
                let jas = (d - 0.5).abs();
                if jas < 0.04 {
                    let t = 1.0 - jas / 0.04;
                    r = (r + t * 0.9).min(1.0);
                    g = g * (1.0 - t * 0.6);
                    b = (b + t * 0.8).min(1.0);
                }
                let jat = (d - 0.75).abs();
                if jat < 0.03 {
                    let t = 1.0 - jat / 0.03;
                    r = (r + t * 0.3).min(1.0);
                    g = (g + t * 0.9).min(1.0);
                    b = g * (1.0 - t * 0.3);
                }

                
                let jnm = (1.0 - iy) + (1.0 - v);
                if jnm > 1.7 {
                    let t = ((jnm - 1.7) / 0.3).min(1.0);
                    r = (r + t * 0.6).min(1.0);
                    g = (g + t * 0.3).min(1.0);
                    b = b * (1.0 - t * 0.4);
                }
                let hij = iy + v;
                if hij > 1.7 {
                    let t = ((hij - 1.7) / 0.3).min(1.0);
                    r = r * (1.0 - t * 0.3);
                    g = (g + t * 0.4).min(1.0);
                    b = (b + t * 0.7).min(1.0);
                }

                
                if u.abs() < 0.012 || cx.abs() < 0.012 {
                    r = (r * 0.5 + 0.5).min(1.0);
                    g = (g * 0.5 + 0.5).min(1.0);
                    b = (b * 0.5 + 0.5).min(1.0);
                }

                
                let csk: f32 = if (1.0 - d * 0.7) > 0.0 { 1.0 - d * 0.7 } else { 0.0 };
                r *= csk;
                g *= csk;
                b *= csk;

                
                r = (r * 1.3).min(1.0);
                g = (g * 1.2).min(1.0);
                b = (b * 1.3).min(1.0);

                let dk = (r * 255.0) as u32;
                let eoh = (g * 255.0) as u32;
                let bal = (b * 255.0) as u32;
                pixels[o * w + p] = 0xFF000000 | (dk << 16) | (eoh << 8) | bal;
            }
        }
        pixels
    }
}


#[derive(Clone)]
pub struct Jr {
    pub visible: bool,
    pub x: i32,
    pub y: i32,
    pub items: Vec<Aa>,
    pub selected_index: usize,
    pub target_icon: Option<usize>,  
    pub target_file: Option<String>, 
}


#[derive(Clone)]
pub struct Rr {
    pub name: String,
    pub icon_type: crate::icons::IconType,
    pub x: u32,
    pub y: u32,
    pub action: IconAction,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IconAction {
    OpenTerminal,
    OpenFileManager,
    OpenSettings,
    OpenAbout,
    OpenMusicPlayer,
    OpenCalculator,
    OpenNetwork,
    OpenGame,
    OpenEditor,
    OpenGL3D,
    OpenBrowser,
    OpenModelEditor,
    OpenGame3D,
    #[cfg(feature = "emulators")]
    OpenNes,
    #[cfg(feature = "emulators")]
    OpenGameBoy,
    #[cfg(feature = "emulators")]
    OpenGameLab,
}


#[derive(Clone, Copy, PartialEq)]
pub enum WindowType {
    Terminal,
    SystemInfo,
    About,
    Empty,
    Calculator,
    FileManager,
    TextEditor,
    Cn,
    Settings,
    ImageViewer,
    HexViewer,
    FileAssociations,
    Demo3D,  
    Game,    
    Browser, 
    ModelEditor, 
    Game3D,  
    Chess,   
    Chess3D, 
    #[cfg(feature = "emulators")]
    NesEmu,  
    #[cfg(feature = "emulators")]
    GameBoyEmu, 
    #[cfg(feature = "emulators")]
    GameBoyInput, 
    BinaryViewer, 
    LabMode,      
    #[cfg(feature = "emulators")]
    GameLab,      
    MusicPlayer,  
    WifiNetworks, 
    WifiPassword, 
    WifiAnalyzer, 
}


#[derive(Clone)]
pub struct Window {
    pub id: u32,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub visible: bool,
    pub focused: bool,
    pub minimized: bool,
    pub maximized: bool,
    pub dragging: bool,
    pub resizing: ResizeEdge,
    pub drag_offset_x: i32,
    pub drag_offset_y: i32,
    
    pub saved_x: i32,
    pub saved_y: i32,
    pub saved_width: u32,
    pub saved_height: u32,
    pub window_type: WindowType,
    pub content: Vec<String>,
    pub file_path: Option<String>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    
    pub animation: WindowAnimation,
    pub pending_close: bool,  
    
    pub dirty: bool,
}


#[derive(Clone, Copy, PartialEq)]
pub enum ResizeEdge {
    None,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}


#[derive(Clone, Copy, PartialEq)]
enum CursorMode {
    Arrow,
    ResizeH,     
    ResizeV,     
    ResizeNWSE,  
    ResizeNESW,  
    Grab,        
}


#[derive(Clone, Copy, PartialEq)]
pub enum FileManagerViewMode {
    List,
    IconGrid,
    Details,
    Tiles,
}


#[derive(Clone, Copy, PartialEq)]
pub enum Awe {
    QuickAccess,
    ThisPC,
}


pub struct FileManagerState {
    
    pub history: Vec<String>,
    
    pub history_idx: usize,
    
    pub sidebar_collapsed: bool,
    
    pub sidebar_width: u32,
    
    pub sidebar_scroll: usize,
    
    pub sidebar_selected: i32,
    
    pub quick_access: Vec<(String, String)>, 
    
    pub sort_column: u8,
    
    pub sort_ascending: bool,
    
    pub hover_index: Option<usize>,
    
    pub search_query: String,
    
    pub search_focused: bool,
    
    pub col_widths: [u32; 4],
}

impl FileManagerState {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            history_idx: 0,
            sidebar_collapsed: false,
            sidebar_width: 180,
            sidebar_scroll: 0,
            sidebar_selected: -1,
            quick_access: vec![
                (String::from("Desktop"), String::from("/")),
                (String::from("Documents"), String::from("/documents")),
                (String::from("Downloads"), String::from("/downloads")),
                (String::from("Music"), String::from("/music")),
                (String::from("Pictures"), String::from("/pictures")),
            ],
            sort_column: 0,
            sort_ascending: true,
            hover_index: None,
            search_query: String::new(),
            search_focused: false,
            col_widths: [200, 80, 80, 120],
        }
    }
    
    pub fn push_history(&mut self, path: &str) {
        
        if self.history_idx + 1 < self.history.len() {
            self.history.truncate(self.history_idx + 1);
        }
        self.history.push(String::from(path));
        self.history_idx = self.history.len() - 1;
    }
    
    pub fn can_go_back(&self) -> bool {
        self.history_idx > 0
    }
    
    pub fn can_go_forward(&self) -> bool {
        self.history_idx + 1 < self.history.len()
    }
    
    pub fn go_back(&mut self) -> Option<&str> {
        if self.history_idx > 0 {
            self.history_idx -= 1;
            Some(&self.history[self.history_idx])
        } else {
            None
        }
    }
    
    pub fn go_forward(&mut self) -> Option<&str> {
        if self.history_idx + 1 < self.history.len() {
            self.history_idx += 1;
            Some(&self.history[self.history_idx])
        } else {
            None
        }
    }
}


pub struct ImageViewerState {
    pub pixels: Vec<u32>,
    pub img_width: u32,
    pub img_height: u32,
    pub zoom: u32,     
    pub pan_x: i32,
    pub pan_y: i32,
}

impl ImageViewerState {
    pub fn new() -> Self {
        Self { pixels: Vec::new(), img_width: 0, img_height: 0, zoom: 100, pan_x: 0, pan_y: 0 }
    }
}


pub struct Yv {
    pub path: String,
    pub name: String,
    pub is_cut: bool,
}


pub struct Xx {
    pub source_path: String,
    pub filename: String,
    pub is_dir: bool,
    pub start_x: i32,
    pub start_y: i32,
    pub current_x: i32,
    pub current_y: i32,
    pub source_window_id: u32,
    pub active: bool,
}

impl Window {
    pub fn new(title: &str, x: i32, y: i32, width: u32, height: u32, wt: WindowType) -> Self {
        let mut ifm = BDP_.lock();
        let id = *ifm;
        *ifm += 1;
        
        Window {
            id,
            title: String::from(title),
            x,
            y,
            width,
            height,
            min_width: 200,
            min_height: 150,
            visible: true,
            focused: false,
            minimized: false,
            maximized: false,
            dragging: false,
            resizing: ResizeEdge::None,
            drag_offset_x: 0,
            drag_offset_y: 0,
            saved_x: x,
            saved_y: y,
            saved_width: width,
            saved_height: height,
            window_type: wt,
            content: Vec::new(),
            file_path: None,
            selected_index: 0,
            scroll_offset: 0,
            animation: WindowAnimation::new(),
            pending_close: false,
            dirty: true,
        }
    }
    
    
    pub fn animate_open(&mut self) {
        if awb() {
            self.animation.start_open(self.x, self.y, self.width, self.height);
        }
    }
    
    
    pub fn animate_close(&mut self) -> bool {
        if awb() {
            self.animation.start_close(self.x, self.y, self.width, self.height);
            self.pending_close = true;
            true 
        } else {
            false 
        }
    }
    
    
    pub fn animate_minimize(&mut self, bwh: i32) {
        if awb() {
            let gyd = 100; 
            self.animation.start_minimize(self.x, self.y, self.width, self.height, gyd, bwh);
        }
    }
    
    
    pub fn pyc(&mut self, screen_w: u32, screen_h: u32) {
        if awb() {
            self.animation.start_maximize(self.x, self.y, self.width, self.height, screen_w, screen_h);
        }
    }
    
    
    pub fn pyd(&mut self) {
        if awb() {
            self.animation.start_restore(
                self.x, self.y, self.width, self.height,
                self.saved_x, self.saved_y, self.saved_width, self.saved_height
            );
        }
    }
    
    
    pub fn update_animation(&mut self) -> bool {
        if self.animation.is_animating() {
            let jgf = self.animation.update();
            
            
            if !self.animation.is_animating() && !jgf {
                
            }
            
            return jgf && self.pending_close;
        }
        false
    }
    
    
    pub fn qii(&self) -> (i32, i32, u32, u32, f32) {
        if self.animation.is_animating() {
            self.animation.get_current()
        } else {
            (self.x, self.y, self.width, self.height, 1.0)
        }
    }
    
    
    pub fn contains(&self, p: i32, o: i32) -> bool {
        if self.minimized { return false; }
        p >= self.x && p < self.x + self.width as i32 &&
        o >= self.y && o < self.y + self.height as i32
    }
    
    
    pub fn in_title_bar(&self, p: i32, o: i32) -> bool {
        p >= self.x && p < self.x + self.width as i32 - 90 &&
        o >= self.y && o < self.y + J_() as i32
    }
    
    
    pub fn on_close_button(&self, p: i32, o: i32) -> bool {
        let gu = 28i32;
        let hn = J_() as i32;
        let bx = self.x + self.width as i32 - gu - 1;
        let dc = self.y + 1;
        p >= bx && p < bx + gu && o >= dc && o < dc + hn
    }
    
    
    pub fn on_maximize_button(&self, p: i32, o: i32) -> bool {
        let gu = 28i32;
        let hn = J_() as i32;
        let bx = self.x + self.width as i32 - gu * 2 - 1;
        let dc = self.y + 1;
        p >= bx && p < bx + gu && o >= dc && o < dc + hn
    }
    
    
    pub fn on_minimize_button(&self, p: i32, o: i32) -> bool {
        let gu = 28i32;
        let hn = J_() as i32;
        let bx = self.x + self.width as i32 - gu * 3 - 1;
        let dc = self.y + 1;
        p >= bx && p < bx + gu && o >= dc && o < dc + hn
    }
    
    
    pub fn on_resize_edge(&self, p: i32, o: i32) -> ResizeEdge {
        if self.maximized { return ResizeEdge::None; }
        
        let eyo = 12i32;
        let ijt = self.x;
        let jar = self.x + self.width as i32;
        let jnl = self.y;
        let hik = self.y + self.height as i32;
        
        let gkv = p >= ijt && p < ijt + eyo;
        let gkw = p >= jar - eyo && p < jar;
        let gky = o >= jnl && o < jnl + eyo;
        let gku = o >= hik - eyo && o < hik;
        
        if gky && gkv { ResizeEdge::TopLeft }
        else if gky && gkw { ResizeEdge::TopRight }
        else if gku && gkv { ResizeEdge::BottomLeft }
        else if gku && gkw { ResizeEdge::BottomRight }
        else if gkv { ResizeEdge::Left }
        else if gkw { ResizeEdge::Right }
        else if gky { ResizeEdge::Top }
        else if gku { ResizeEdge::Bottom }
        else { ResizeEdge::None }
    }
    
    
    pub fn toggle_maximize(&mut self, screen_width: u32, screen_height: u32) {
        if self.maximized {
            
            self.x = self.saved_x;
            self.y = self.saved_y;
            self.width = self.saved_width;
            self.height = self.saved_height;
            self.maximized = false;
        } else {
            
            self.saved_x = self.x;
            self.saved_y = self.y;
            self.saved_width = self.width;
            self.saved_height = self.height;
            
            self.x = BW_() as i32;
            self.y = 0;
            self.width = screen_width.saturating_sub(BW_());
            self.height = screen_height.saturating_sub(V_());
            self.maximized = true;
        }
    }
}


use crate::graphics::{compositor, Compositor, CompositorTheme, WindowSurface, Easing};






#[derive(Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}


pub struct MusicPlayerState {
    pub state: PlaybackState,
    
    pub audio: Option<Vec<i16>>,
    
    pub song_title: String,
    
    pub current_track: usize,
    
    pub num_tracks: usize,
    
    pub write_cursor: usize,
    
    pub last_half: u32,
    
    pub audio_exhausted: bool,
    
    pub consumed_samples: usize,
    
    pub dma_cap: usize,
    
    pub vis_frame: u32,
    
    pub elapsed_ms: u64,
    
    pub seek_base_ms: u64,
    
    pub total_ms: u64,
    
    pub fft_re: [f32; 1024],
    pub fft_im: [f32; 1024],
    
    pub peak_rms: f32,
    
    pub sub_bass: f32,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    
    pub beat: f32,
    
    pub energy: f32,
    
    pub prev_energy: f32,
    
    pub energy_hist: [f32; 43],
    pub hist_idx: usize,
    pub hist_count: usize,
    
    pub waveform: [f32; 128],
    pub wave_idx: usize,
    
    pub volume: u32,
    
    pub av_offset_ms: i32,
    
    pub is_looping: bool,
    
    pub track_names: Vec<String>,
    
    pub track_list_scroll: usize,
}

impl MusicPlayerState {
    pub fn new() -> Self {
        Self {
            state: PlaybackState::Stopped,
            audio: None,
            song_title: String::from("No Track"),
            current_track: 0,
            num_tracks: 0,
            write_cursor: 0,
            last_half: 0,
            audio_exhausted: false,
            consumed_samples: 0,
            dma_cap: 0,
            vis_frame: 0,
            elapsed_ms: 0,
            seek_base_ms: 0,
            total_ms: 0,
            fft_re: [0.0; 1024],
            fft_im: [0.0; 1024],
            peak_rms: 1.0,
            sub_bass: 0.0,
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            beat: 0.0,
            energy: 0.0,
            prev_energy: 0.0,
            energy_hist: [0.0; 43],
            hist_idx: 0,
            hist_count: 0,
            waveform: [0.0; 128],
            wave_idx: 0,
            volume: 75,
            av_offset_ms: 0,
            is_looping: false,
            track_names: Vec::new(),
            track_list_scroll: 0,
        }
    }

    
    pub fn load_track_list(&mut self) {
        self.track_names = crate::trustdaw::disk_audio::mdz();
        self.num_tracks = self.track_names.len();
        crate::serial_println!("[MUSIC] Track list: {} tracks", self.num_tracks);
        for (i, name) in self.track_names.iter().enumerate() {
            crate::serial_println!("[MUSIC]   {}: {}", i, name);
        }
    }

    
    pub fn gni(&mut self) {
        self.play_track(self.current_track);
    }

    
    pub fn play_track(&mut self, mp: usize) {
        
        
        self.stop();

        
        if self.num_tracks == 0 {
            self.load_track_list();
        }

        if self.num_tracks == 0 {
            self.song_title = String::from("No tracks found");
            crate::serial_println!("[MUSIC] No tracks available on disk");
            return;
        }

        let idx = mp % self.num_tracks;
        self.current_track = idx;

        crate::serial_println!("[MUSIC] Loading track {} — heap free: {} KB",
            idx, crate::memory::heap::free() / 1024);

        
        match crate::trustdaw::disk_audio::etf(idx) {
            Ok((raw_wav, name)) => {
                
                if raw_wav.len() >= 12 {
                    crate::serial_println!("[MUSIC] Track {} raw header: {:02X} {:02X} {:02X} {:02X} ... {:02X} {:02X} {:02X} {:02X}",
                        idx, raw_wav[0], raw_wav[1], raw_wav[2], raw_wav[3],
                        raw_wav[8], raw_wav[9], raw_wav[10], raw_wav[11]);
                }
                match crate::trustdaw::audio_viz::byq(&raw_wav) {
                    Ok(audio) => {
                        crate::serial_println!("[MUSIC] Decoded track {}: '{}' → {} samples", idx, name, audio.len());
                        
                        drop(raw_wav);
                        self.start_playback_with_audio(audio, &name);
                        return;
                    }
                    Err(e) => {
                        crate::serial_println!("[MUSIC] Track {} decode error: {}", idx, e);
                    }
                }
            }
            Err(e) => {
                crate::serial_println!("[MUSIC] Track {} load error: {}", idx, e);
            }
        }

        
        let name = if idx < self.track_names.len() {
            self.track_names[idx].clone()
        } else {
            alloc::format!("Track {}", idx + 1)
        };
        self.song_title = alloc::format!("{} (load failed)", name);
    }

    
    pub fn next_track(&mut self) {
        if self.num_tracks > 1 {
            let next = (self.current_track + 1) % self.num_tracks;
            self.play_track(next);
        } else {
            
            self.play_track(self.current_track);
        }
    }

    
    pub fn prev_track(&mut self) {
        if self.num_tracks > 1 {
            let prev = if self.current_track == 0 { self.num_tracks - 1 } else { self.current_track - 1 };
            self.play_track(prev);
        } else {
            
            self.play_track(self.current_track);
        }
    }

    
    fn start_playback_with_audio(&mut self, audio: Vec<i16>, title: &str) {
        self.song_title = String::from(title);
        let total_frames = audio.len() / 2;
        self.total_ms = (total_frames as u64 * 1000) / 48000;

        
        crate::audio::init().ok();

        
        let dma_cap = crate::drivers::hda::cym()
            .map(|(_, c)| c)
            .unwrap_or(0);
        if dma_cap == 0 {
            crate::serial_println!("[MUSIC] No DMA buffer available");
            return;
        }

        
        let _ = crate::drivers::hda::stop();
        crate::drivers::hda::eyn();

        
        let are = audio.len().min(dma_cap);
        if let Ok(()) = crate::drivers::hda::bdu(&audio[0..are]) {
            self.write_cursor = are;
            self.dma_cap = dma_cap;
            self.audio = Some(audio);
            self.state = PlaybackState::Playing;
            self.audio_exhausted = false;
            self.consumed_samples = 0;
            self.seek_base_ms = 0;
            self.vis_frame = 0;
            self.elapsed_ms = 0;

            
            let alw = crate::drivers::hda::dqq();
            let cye = (dma_cap * 2) as u32;
            let drd = cye / 2;
            let dav = if alw >= cye { 0 } else { alw };
            self.last_half = if dav < drd { 0 } else { 1 };

            
            let _ = crate::drivers::hda::set_volume(self.volume.min(100) as u8);

            
            self.peak_rms = 1.0;
            self.sub_bass = 0.0;
            self.bass = 0.0;
            self.mid = 0.0;
            self.treble = 0.0;
            self.beat = 0.0;
            self.energy = 0.0;
            self.prev_energy = 0.0;
            self.energy_hist = [0.0; 43];
            self.hist_idx = 0;
            self.hist_count = 0;
            self.waveform = [0.0; 128];
            self.wave_idx = 0;
            crate::serial_println!("[MUSIC] Playing '{}', {}ms, DMA={}", self.song_title, self.total_ms, dma_cap);
        } else {
            crate::serial_println!("[MUSIC] start_looped_playback failed");
        }
    }

    
    pub fn stop(&mut self) {
        if self.state != PlaybackState::Stopped {
            let _ = crate::drivers::hda::stop();
            crate::drivers::hda::eyn();
            self.state = PlaybackState::Stopped;
            self.audio = None;
            self.write_cursor = 0;
            self.consumed_samples = 0;
            self.dma_cap = 0;
            self.seek_base_ms = 0;
            self.vis_frame = 0;
            self.elapsed_ms = 0;
            self.is_looping = false;
            
            self.beat = 0.0;
            self.energy = 0.0;
            self.sub_bass = 0.0;
            self.bass = 0.0;
            self.mid = 0.0;
            self.treble = 0.0;
            crate::serial_println!("[MUSIC] Stopped");
        }
    }

    
    pub fn toggle_pause(&mut self) {
        match self.state {
            PlaybackState::Playing => {
                let _ = crate::drivers::hda::stop();
                self.state = PlaybackState::Paused;
                crate::serial_println!("[MUSIC] Paused at {}ms", self.elapsed_ms);
            }
            PlaybackState::Paused => {
                
                self.resume_from_current_pos();
            }
            _ => {}
        }
    }

    
    
    fn resume_from_current_pos(&mut self) {
        
        let audio = match self.audio.take() {
            Some(a) => a,
            None => return,
        };
        let dma_cap = crate::drivers::hda::cym()
            .map(|(_, c)| c)
            .unwrap_or(0);
        if dma_cap == 0 {
            self.audio = Some(audio);
            return;
        }

        
        let sample_pos = ((self.elapsed_ms as usize * 48000 * 2) / 1000).min(audio.len());

        
        let _ = crate::drivers::hda::stop();
        crate::drivers::hda::eyn();

        
        let are = audio.len().saturating_sub(sample_pos).min(dma_cap);
        if are == 0 {
            self.audio = Some(audio);
            self.stop();
            return;
        }
        if let Ok(()) = crate::drivers::hda::bdu(&audio[sample_pos..sample_pos + are]) {
            self.write_cursor = sample_pos + are;
            self.dma_cap = dma_cap;
            self.consumed_samples = 0;
            self.seek_base_ms = self.elapsed_ms;
            self.state = PlaybackState::Playing;
            self.audio_exhausted = false;

            
            self.last_half = 0;

            let _ = crate::drivers::hda::set_volume(self.volume.min(100) as u8);
            crate::serial_println!("[MUSIC] Resumed at {}ms, sample={}", self.elapsed_ms, sample_pos);
        } else {
            crate::serial_println!("[MUSIC] Resume start_looped_playback failed");
        }
        
        self.audio = Some(audio);
    }

    
    pub fn seek_to(&mut self, ebh: u64) {
        let ebh = ebh.min(self.total_ms);
        self.elapsed_ms = ebh;

        
        if self.state == PlaybackState::Paused {
            crate::serial_println!("[MUSIC] Seek (paused) to {}ms", ebh);
            return;
        }

        
        if self.state == PlaybackState::Playing {
            self.resume_from_current_pos();
            crate::serial_println!("[MUSIC] Seek (playing) to {}ms", ebh);
        }
    }

    
    pub fn tick(&mut self) {
        if self.state != PlaybackState::Playing { return; }
        let audio = match &self.audio {
            Some(a) => a,
            None => return,
        };

        
        if let Some((dma_ptr, dma_cap)) = crate::drivers::hda::cym() {
            let aaz = dma_cap / 2;
            let drd = (aaz * 2) as u32;
            let cye = (dma_cap * 2) as u32;

            crate::drivers::hda::hli();
            crate::drivers::hda::hwa();

            let alw = crate::drivers::hda::dqq();
            let dav = if alw >= cye { 0 } else { alw };
            let dlx = if dav < drd { 0u32 } else { 1u32 };

            if dlx != self.last_half {
                
                self.consumed_samples += aaz;

                if self.write_cursor < audio.len() {
                    let awv = self.last_half as usize * aaz;
                    let ck = audio.len() - self.write_cursor;
                    let od = ck.min(aaz);
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            audio.as_ptr().add(self.write_cursor),
                            dma_ptr.add(awv),
                            od,
                        );
                        if od < aaz {
                            
                            if self.is_looping && !audio.is_empty() {
                                let mut oz = od;
                                while oz < aaz {
                                    let eeh = (aaz - oz).min(audio.len());
                                    core::ptr::copy_nonoverlapping(
                                        audio.as_ptr(),
                                        dma_ptr.add(awv + oz),
                                        eeh,
                                    );
                                    oz += eeh;
                                }
                            } else {
                                core::ptr::write_bytes(dma_ptr.add(awv + od), 0, aaz - od);
                            }
                        }
                    }
                    self.write_cursor += od;
                    if self.write_cursor >= audio.len() {
                        if self.is_looping {
                            self.write_cursor = 0; 
                        } else {
                            self.audio_exhausted = true;
                        }
                    }
                } else if self.is_looping && !audio.is_empty() {
                    
                    self.write_cursor = 0;
                    let awv = self.last_half as usize * aaz;
                    let od = audio.len().min(aaz);
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            audio.as_ptr(),
                            dma_ptr.add(awv),
                            od,
                        );
                        if od < aaz {
                            let mut oz = od;
                            while oz < aaz {
                                let eeh = (aaz - oz).min(audio.len());
                                core::ptr::copy_nonoverlapping(
                                    audio.as_ptr(),
                                    dma_ptr.add(awv + oz),
                                    eeh,
                                );
                                oz += eeh;
                            }
                        }
                    }
                    self.write_cursor += od;
                } else {
                    let awv = self.last_half as usize * aaz;
                    unsafe { core::ptr::write_bytes(dma_ptr.add(awv), 0, aaz); }
                }
                self.last_half = dlx;
            }

            
            
            
            let ggj = (dav / 2) as usize; 
            
            
            
            
            
            
            let mkx = if self.consumed_samples + ggj >= dma_cap {
                self.consumed_samples + ggj - dma_cap
            } else {
                ggj 
            };
            
            
            self.elapsed_ms = self.seek_base_ms + (mkx as u64 * 1000) / (48000 * 2);
        }

        self.vis_frame += 1;

        
        if self.elapsed_ms >= self.total_ms || self.audio_exhausted {
            if self.is_looping {
                
                self.write_cursor = 0;
                self.audio_exhausted = false;
                self.consumed_samples = 0;
                self.seek_base_ms = 0;
                self.elapsed_ms = 0;
                return;
            }
            
            if self.num_tracks > 1 {
                let next = (self.current_track + 1) % self.num_tracks;
                crate::serial_println!("[MUSIC] Track ended, auto-advancing to track {}", next);
                self.play_track(next);
            } else {
                self.stop();
            }
            return;
        }

        
        let psl = (self.elapsed_ms as i64 + self.av_offset_ms as i64).max(0).min(self.total_ms as i64) as u64;
        let cts = (psl as usize * 48000 * 2 / 1000).min(audio.len().saturating_sub(2));

        
        let aca = 256usize;
        let dun = cts.saturating_sub(aca * 2) & !1;
        let mut yw: f32 = 0.0;
        for i in 0..aca {
            let idx = dun + i * 2;
            let j = if idx < audio.len() { audio[idx] as f32 } else { 0.0 };
            self.fft_re[i] = j;
            self.fft_im[i] = 0.0;
            let a = if j >= 0.0 { j } else { -j };
            if a > yw { yw = a; }
        }
        
        if yw > self.peak_rms {
            self.peak_rms += (yw - self.peak_rms) * 0.3;
        } else {
            self.peak_rms *= 0.9995;
        }
        let bmi = if self.peak_rms > 100.0 { 16000.0 / self.peak_rms } else { 1.0 };
        
        for i in 0..aca {
            let t = i as f32 / aca as f32;
            let drf = 0.5 * (1.0 - libm::cosf(2.0 * core::f32::consts::PI * t));
            self.fft_re[i] *= drf * bmi / 32768.0;
        }
        
        {
            let xh = &mut self.fft_re[..aca];
            let xq = &mut self.fft_im[..aca];
            
            let mut ay = 0usize;
            for i in 0..aca {
                if i < ay { xh.swap(i, ay); xq.swap(i, ay); }
                let mut m = aca >> 1;
                while m >= 1 && ay >= m { ay -= m; m >>= 1; }
                ay += m;
            }
            
            let mut step = 2;
            while step <= aca {
                let cw = step / 2;
                let jwb = -core::f32::consts::PI * 2.0 / step as f32;
                for k in 0..cw {
                    let a = jwb * k as f32;
                    let aep = libm::cosf(a);
                    let ld = libm::sinf(a);
                    let mut ard = k;
                    while ard < aca {
                        let axt = ard + cw;
                        let tr = aep * xh[axt] - ld * xq[axt];
                        let cej = aep * xq[axt] + ld * xh[axt];
                        xh[axt] = xh[ard] - tr; xq[axt] = xq[ard] - cej;
                        xh[ard] += tr; xq[ard] += cej;
                        ard += step;
                    }
                }
                step <<= 1;
            }
        }
        
        let bue = |lo: usize, hi: usize| -> f32 {
            let mut j = 0.0f32;
            for i in lo..hi.min(128) {
                j += libm::sqrtf(self.fft_re[i] * self.fft_re[i] + self.fft_im[i] * self.fft_im[i]);
            }
            j / (hi - lo).max(1) as f32
        };
        let cov = bue(1, 2);   
        let biu = bue(2, 4);  
        let cda = bue(4, 16);  
        let dxh = bue(16, 60); 
        let gpr = cov * 1.5 + biu * 1.2 + cda * 0.5 + dxh * 0.2;

        
        let afs = |prev: f32, new: f32, a: f32, r: f32| -> f32 {
            if new > prev { prev + (new - prev) * a } else { prev + (new - prev) * r }
        };
        self.sub_bass = afs(self.sub_bass, cov.min(1.0), 0.75, 0.10);
        self.bass = afs(self.bass, biu.min(1.0), 0.70, 0.10);
        self.mid = afs(self.mid, cda.min(1.0), 0.60, 0.12);
        self.treble = afs(self.treble, dxh.min(1.0), 0.70, 0.16);
        self.energy = afs(self.energy, gpr.min(1.5), 0.65, 0.10);

        
        let beu = cov + biu * 0.8;
        self.energy_hist[self.hist_idx] = beu;
        self.hist_idx = (self.hist_idx + 1) % 43;
        if self.hist_count < 43 { self.hist_count += 1; }
        let oz = self.hist_count.max(1) as f32;
        let ns: f32 = self.energy_hist.iter().take(self.hist_count).sum::<f32>() / oz;
        let mut cex = 0.0f32;
        for i in 0..self.hist_count { let d = self.energy_hist[i] - ns; cex += d * d; }
        let edo = cex / oz;
        let amz = (-15.0 * edo + 1.45f32).max(1.05).min(1.5);
        let glb = beu - self.prev_energy;
        if beu > ns * amz && glb > 0.002 && self.hist_count > 5 {
            let strength = ((beu - ns * amz) / ns.max(0.001)).min(1.0);
            self.beat = (0.6 + strength * 0.4).min(1.0);
        } else {
            self.beat *= 0.88;
            if self.beat < 0.02 { self.beat = 0.0; }
        }
        self.prev_energy = beu;

        
        if !audio.is_empty() {
            let idx = cts.min(audio.len() - 1) & !1;
            let sample = audio[idx] as f32 / 32768.0;
            self.waveform[self.wave_idx % 128] = sample;
            self.wave_idx += 1;
        }
    }
}






#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    
    Classic,
    
    OpenGL,
    
    
    GpuAccelerated,
}




#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum DesktopTier {
    
    CliOnly,
    
    Minimal,
    
    Standard,
    
    Full,
}

pub struct Desktop {
    pub windows: Vec<Window>,
    pub icons: Vec<Rr>,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub cursor_visible: bool,
    pub width: u32,
    pub height: u32,
    frame_count: u64,
    start_menu_open: bool,
    pub input_buffer: String,
    pub cursor_blink: bool,
    pub context_menu: Jr,
    
    cached_time_str: String,
    cached_date_str: String,
    last_rtc_frame: u64,
    
    background_cached: bool,
    needs_full_redraw: bool,
    last_cursor_x: i32,
    last_cursor_y: i32,
    
    last_window_count: usize,
    last_start_menu_open: bool,
    last_context_menu_visible: bool,
    
    pub render_mode: RenderMode,
    pub compositor_theme: CompositorTheme,
    
    dirty_rects: [(u32, u32, u32, u32); 32], 
    dirty_rect_count: usize,
    gpu_frame_skip: u32,  
    
    pub browser: Option<crate::browser::Browser>,
    pub browser_url_input: String,
    pub browser_url_cursor: usize,
    pub browser_loading: bool,
    pub browser_url_select_all: bool,
    
    pub editor_states: BTreeMap<u32, EditorState>,
    
    pub model_editor_states: BTreeMap<u32, crate::model_editor::ModelEditorState>,
    
    pub calculator_states: BTreeMap<u32, CalculatorState>,
    
    pub snake_states: BTreeMap<u32, SnakeState>,
    
    pub game3d_states: BTreeMap<u32, crate::game3d::Game3DState>,
    
    pub chess_states: BTreeMap<u32, crate::chess::ChessState>,
    
    pub chess3d_states: BTreeMap<u32, crate::chess3d::Chess3DState>,
    
    pub binary_viewer_states: BTreeMap<u32, crate::apps::binary_viewer::BinaryViewerState>,
    
    #[cfg(feature = "emulators")]
    pub nes_states: BTreeMap<u32, crate::nes::NesEmulator>,
    
    #[cfg(feature = "emulators")]
    pub gameboy_states: BTreeMap<u32, crate::gameboy::GameBoyEmulator>,
    
    pub lab_states: BTreeMap<u32, crate::lab_mode::LabState>,
    
    pub music_player_states: BTreeMap<u32, MusicPlayerState>,
    
    #[cfg(feature = "emulators")]
    pub gamelab_states: BTreeMap<u32, crate::game_lab::GameLabState>,
    
    #[cfg(feature = "emulators")]
    pub gb_input_links: BTreeMap<u32, u32>,
    
    pub wifi_analyzer_states: BTreeMap<u32, crate::wifi_analyzer::WifiAnalyzerState>,
    
    pub scale_factor: u32,
    
    matrix_cols: usize,
    matrix_chars: Vec<u8>,
    matrix_heads: Vec<i32>,
    matrix_speeds: Vec<u32>,
    matrix_seeds: Vec<u32>,
    matrix_initialized: bool,
    matrix_beat_count: u32,
    matrix_last_beat: bool,
    
    pub matrix_rain_preset: u8,
    
    
    
    pub matrix_overrides: BTreeMap<usize, CellPixels>,
    
    pub matrix_projection: MatrixProjection,
    
    visualizer: crate::visualizer::VisualizerState,
    
    drone_swarm: crate::drone_swarm::DroneSwarmState,
    
    
    global_fft_re: Vec<f32>,
    global_fft_im: Vec<f32>,
    global_sub_bass: f32,
    global_bass: f32,
    global_mid: f32,
    global_treble: f32,
    global_energy: f32,
    global_beat: f32,
    global_peak_rms: f32,
    global_prev_energy: f32,
    global_energy_hist: Vec<f32>,
    global_hist_idx: usize,
    global_hist_count: usize,
    global_audio_active: bool,
    
    terminal_suggestion_count: usize,
    
    command_history: Vec<String>,
    history_index: Option<usize>,
    saved_input: String,
    
    pub start_menu_search: String,
    
    pub start_menu_selected: i32,
    
    clipboard_icon: Option<(usize, bool)>,
    
    
    
    pub fm_view_modes: BTreeMap<u32, FileManagerViewMode>,
    
    pub fm_states: BTreeMap<u32, FileManagerState>,
    
    pub image_viewer_states: BTreeMap<u32, ImageViewerState>,
    
    pub file_clipboard: Option<Yv>,
    
    pub drag_state: Option<Xx>,
    
    pub settings_category: u8,
    
    pub netscan_tab: u8,
    
    pub lock_screen_active: bool,
    
    pub lock_screen_input: String,
    
    pub lock_screen_shake: u32,
    
    pub sys_volume: u32,
    
    pub sys_battery: u32,
    
    pub sys_wifi_connected: bool,
    
    
    
    pub wifi_selected_index: usize,
    
    pub wifi_scroll_offset: usize,
    
    pub wifi_password_input: String,
    
    pub wifi_connecting_ssid: String,
    
    pub wifi_show_password: bool,
    
    pub wifi_scan_requested: bool,
    
    pub wifi_error_msg: Option<String>,
    
    
    
    gesture_recognizer: crate::gesture::GestureRecognizer,
    
    gesture_buffer: crate::gesture::GestureBuffer,
    
    pub touch_mode: bool,
    
    pub mobile_state: crate::mobile::MobileState,
    
    
    fps_last_tick: u64,
    
    fps_frame_accum: u32,
    
    pub fps_current: u32,
    
    pub fps_display: bool,
    
    pub desktop_tier: DesktopTier,
    
    fps_low_count: u32,
    
    fps_high_count: u32,
    
    initial_tier: DesktopTier,
    
    tier_cooldown: u32,
    
    pub tier_manual_override: bool,
    
    snap_preview: Option<SnapDir>,
    
    show_shortcuts: bool,
    
    windows_dirty: bool,
    
    prev_hover_window_id: Option<u32>,
    
    
    shutdown_active: bool,
    
    shutdown_start_tick: u64,
    
    shutdown_phase: u8,
}


pub struct CalculatorState {
    
    pub expression: String,
    
    pub display: String,
    
    pub just_evaluated: bool,
    
    pub scientific: bool,
}

impl CalculatorState {
    pub fn new() -> Self {
        CalculatorState {
            expression: String::new(),
            display: String::from("0"),
            just_evaluated: false,
            scientific: false,
        }
    }
    
    pub fn press_digit(&mut self, d: char) {
        if self.just_evaluated {
            self.expression.clear();
            self.just_evaluated = false;
        }
        if self.expression.len() < 64 {
            self.expression.push(d);
            self.display = self.expression.clone();
        }
    }
    
    pub fn press_dot(&mut self) {
        if self.just_evaluated {
            self.expression = String::from("0");
            self.just_evaluated = false;
        }
        self.expression.push('.');
        self.display = self.expression.clone();
    }
    
    pub fn press_operator(&mut self, op: char) {
        if self.just_evaluated {
            
            self.just_evaluated = false;
        }
        if !self.expression.is_empty() {
            self.expression.push(op);
            self.display = self.expression.clone();
        }
    }
    
    pub fn press_paren(&mut self, aa: char) {
        if self.just_evaluated && aa == '(' {
            self.expression.clear();
            self.just_evaluated = false;
        }
        self.expression.push(aa);
        self.display = self.expression.clone();
    }
    
    pub fn press_func(&mut self, name: &str) {
        if self.just_evaluated {
            self.expression.clear();
            self.just_evaluated = false;
        }
        self.expression.push_str(name);
        self.expression.push('(');
        self.display = self.expression.clone();
    }
    
    pub fn press_equals(&mut self) {
        let result = Self::elp(&self.expression);
        self.display = Self::format_number(result);
        
        self.expression = self.display.clone();
        self.just_evaluated = true;
    }
    
    pub fn press_clear(&mut self) {
        self.expression.clear();
        self.display = String::from("0");
        self.just_evaluated = false;
    }
    
    pub fn press_backspace(&mut self) {
        if !self.expression.is_empty() {
            
            let dqj = ["sqrt(", "sin(", "cos(", "tan(", "abs(", "ln("];
            let mut izn = false;
            for f in dqj {
                if self.expression.ends_with(f) {
                    for _ in 0..f.len() { self.expression.pop(); }
                    izn = true;
                    break;
                }
            }
            if !izn {
                self.expression.pop();
            }
            if self.expression.is_empty() {
                self.display = String::from("0");
            } else {
                self.display = self.expression.clone();
            }
        }
    }
    
    
    pub fn ran(&mut self) {
        self.scientific = !self.scientific;
    }
    
    
    
    
    
    
    
    fn elp(expr: &str) -> f64 {
        let tokens = Self::crv(expr);
        let mut pos = 0;
        let result = Self::parse_expr(&tokens, &mut pos);
        result
    }
    
    fn crv(expr: &str) -> Vec<CalcToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = expr.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let ch = chars[i];
            if ch.is_ascii_digit() || ch == '.' {
                
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let rw: String = chars[start..i].iter().collect();
                tokens.push(CalcToken::Num(Self::gmi(&rw)));
            } else if ch.is_ascii_alphabetic() {
                
                let start = i;
                while i < chars.len() && chars[i].is_ascii_alphabetic() {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                tokens.push(CalcToken::Func(name));
            } else if ch == '(' {
                tokens.push(CalcToken::LParen);
                i += 1;
            } else if ch == ')' {
                tokens.push(CalcToken::RParen);
                i += 1;
            } else if ch == '+' || ch == '-' || ch == '*' || ch == '/' || ch == '%' {
                tokens.push(CalcToken::Op(ch));
                i += 1;
            } else {
                i += 1; 
            }
        }
        tokens
    }
    
    fn parse_expr(tokens: &[CalcToken], pos: &mut usize) -> f64 {
        let mut left = Self::cnv(tokens, pos);
        while *pos < tokens.len() {
            match &tokens[*pos] {
                CalcToken::Op('+') => { *pos += 1; left += Self::cnv(tokens, pos); }
                CalcToken::Op('-') => { *pos += 1; left -= Self::cnv(tokens, pos); }
                _ => break,
            }
        }
        left
    }
    
    fn cnv(tokens: &[CalcToken], pos: &mut usize) -> f64 {
        let mut left = Self::bip(tokens, pos);
        while *pos < tokens.len() {
            match &tokens[*pos] {
                CalcToken::Op('*') => { *pos += 1; left *= Self::bip(tokens, pos); }
                CalcToken::Op('/') => { *pos += 1; let r = Self::bip(tokens, pos); left = if r != 0.0 { left / r } else { 0.0 }; }
                CalcToken::Op('%') => { *pos += 1; let r = Self::bip(tokens, pos); left = if r != 0.0 { left % r } else { 0.0 }; }
                _ => break,
            }
        }
        left
    }
    
    fn bip(tokens: &[CalcToken], pos: &mut usize) -> f64 {
        
        if *pos < tokens.len() {
            if let CalcToken::Op('-') = &tokens[*pos] {
                *pos += 1;
                return -Self::itm(tokens, pos);
            }
        }
        Self::itm(tokens, pos)
    }
    
    fn itm(tokens: &[CalcToken], pos: &mut usize) -> f64 {
        if *pos >= tokens.len() { return 0.0; }
        
        match &tokens[*pos] {
            CalcToken::Num(ae) => {
                let v = *ae;
                *pos += 1;
                v
            }
            CalcToken::LParen => {
                *pos += 1; 
                let v = Self::parse_expr(tokens, pos);
                if *pos < tokens.len() {
                    if let CalcToken::RParen = &tokens[*pos] { *pos += 1; }
                }
                v
            }
            CalcToken::Func(name) => {
                let bsr = name.clone();
                *pos += 1; 
                
                if *pos < tokens.len() {
                    if let CalcToken::LParen = &tokens[*pos] { *pos += 1; }
                }
                let db = Self::parse_expr(tokens, pos);
                if *pos < tokens.len() {
                    if let CalcToken::RParen = &tokens[*pos] { *pos += 1; }
                }
                Self::jxa(&bsr, db)
            }
            _ => {
                *pos += 1;
                0.0
            }
        }
    }
    
    fn jxa(name: &str, x: f64) -> f64 {
        match name {
            "sqrt" => {
                if x >= 0.0 { Self::sqrt_approx(x) } else { 0.0 }
            }
            "sin" => Self::aip(x),
            "cos" => Self::anx(x),
            "tan" => {
                let c = Self::anx(x);
                if c.abs() > 1e-10 { Self::aip(x) / c } else { 0.0 }
            }
            "abs" => if x < 0.0 { -x } else { x },
            "ln" => Self::ln_approx(x),
            _ => x,
        }
    }
    
    
    
    fn sqrt_approx(x: f64) -> f64 {
        if x <= 0.0 { return 0.0; }
        let mut uc = x / 2.0;
        for _ in 0..20 {
            uc = (uc + x / uc) / 2.0;
        }
        uc
    }
    
    fn aip(x: f64) -> f64 {
        
        let pi = 3.14159265358979323846;
        let mut x = x % (2.0 * pi);
        if x > pi { x -= 2.0 * pi; }
        if x < -pi { x += 2.0 * pi; }
        
        let x2 = x * x;
        let x3 = x2 * x;
        let cfo = x3 * x2;
        let csy = cfo * x2;
        let ffn = csy * x2;
        let pvm = ffn * x2;
        x - x3 / 6.0 + cfo / 120.0 - csy / 5040.0 + ffn / 362880.0 - pvm / 39916800.0
    }
    
    fn anx(x: f64) -> f64 {
        let pi = 3.14159265358979323846;
        Self::aip(x + pi / 2.0)
    }
    
    fn ln_approx(x: f64) -> f64 {
        if x <= 0.0 { return 0.0; }
        
        let y = (x - 1.0) / (x + 1.0);
        let y2 = y * y;
        let mut result = y;
        let mut wp = y;
        for ae in 1..30 {
            wp *= y2;
            result += wp / (2 * ae + 1) as f64;
        }
        result * 2.0
    }
    
    fn gmi(j: &str) -> f64 {
        let mut result: f64 = 0.0;
        let mut hqx = false;
        let mut hqw = 0.1;
        let mut ipl = false;
        for (i, ch) in j.chars().enumerate() {
            if ch == '-' && i == 0 {
                ipl = true;
            } else if ch == '.' {
                hqx = true;
            } else if ch.is_ascii_digit() {
                let blu = (ch as u8 - b'0') as f64;
                if hqx {
                    result += blu * hqw;
                    hqw *= 0.1;
                } else {
                    result = result * 10.0 + blu;
                }
            }
        }
        if ipl { -result } else { result }
    }
    
    fn format_number(ae: f64) -> String {
        if ae == (ae as i64) as f64 && ae.abs() < 1e15 {
            format!("{}", ae as i64)
        } else {
            let j = format!("{:.6}", ae);
            let j = j.trim_end_matches('0');
            let j = j.trim_end_matches('.');
            String::from(j)
        }
    }
}


#[derive(Clone)]
enum CalcToken {
    Num(f64),
    Op(char),
    LParen,
    RParen,
    Func(String),
}


pub struct SnakeState {
    pub snake: Vec<(i32, i32)>,
    pub direction: (i32, i32),
    pub food: (i32, i32),
    pub score: u32,
    pub game_over: bool,
    pub paused: bool,
    pub high_score: u32,
    pub grid_w: i32,
    pub grid_h: i32,
    pub tick_counter: u32,
    pub speed: u32,
    pub rng_state: u32,
}

impl SnakeState {
    pub fn new() -> Self {
        let mut state = SnakeState {
            snake: Vec::new(),
            direction: (1, 0),
            food: (12, 5),
            score: 0,
            game_over: false,
            paused: false,
            high_score: 0,
            grid_w: 20,
            grid_h: 15,
            tick_counter: 0,
            speed: 8, 
            rng_state: 42,
        };
        
        for i in 0..4 {
            state.snake.push((10 - i, 7));
        }
        state
    }
    
    fn next_rng(&mut self) -> u32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        self.rng_state
    }
    
    pub fn spawn_food(&mut self) {
        
        let pln = (self.grid_w * self.grid_h) as usize;
        if self.snake.len() >= pln {
            
            self.game_over = true;
            if self.score > self.high_score { self.high_score = self.score; }
            return;
        }
        for _ in 0..1000 {
            let dg = (self.next_rng() % self.grid_w as u32) as i32;
            let hj = (self.next_rng() % self.grid_h as u32) as i32;
            if !self.snake.iter().any(|&(am, ak)| am == dg && ak == hj) {
                self.food = (dg, hj);
                return;
            }
        }
        
        for jh in 0..self.grid_h {
            for hc in 0..self.grid_w {
                if !self.snake.iter().any(|&(am, ak)| am == hc && ak == jh) {
                    self.food = (hc, jh);
                    return;
                }
            }
        }
        
        self.game_over = true;
        if self.score > self.high_score { self.high_score = self.score; }
    }
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_, AI_, AJ_};
        
        if key == b'p' || key == b'P' || key == 0x1B {
            if !self.game_over {
                self.paused = !self.paused;
                return;
            }
        }
        if self.game_over {
            if key == b' ' || key == 0x0D {
                
                let gbf = self.high_score;
                *self = SnakeState::new();
                self.high_score = gbf;
            }
            return;
        }
        if self.paused { return; }
        match key {
            T_    if self.direction != (0, 1)  => self.direction = (0, -1),
            S_  if self.direction != (0, -1) => self.direction = (0, 1),
            AI_  if self.direction != (1, 0)  => self.direction = (-1, 0),
            AJ_ if self.direction != (-1, 0) => self.direction = (1, 0),
            _ => {}
        }
    }
    
    pub fn tick(&mut self) {
        if self.game_over || self.paused { return; }
        self.tick_counter += 1;
        if self.tick_counter < self.speed { return; }
        self.tick_counter = 0;
        
        let su = self.snake[0];
        let bum = (su.0 + self.direction.0, su.1 + self.direction.1);
        
        
        if bum.0 < 0 || bum.0 >= self.grid_w || bum.1 < 0 || bum.1 >= self.grid_h {
            self.game_over = true;
            if self.score > self.high_score { self.high_score = self.score; }
            return;
        }
        
        
        if self.snake.iter().any(|&j| j == bum) {
            self.game_over = true;
            if self.score > self.high_score { self.high_score = self.score; }
            return;
        }
        
        self.snake.insert(0, bum);
        
        
        if bum == self.food {
            self.score += 10;
            self.spawn_food();
            
            if self.score % 50 == 0 && self.speed > 3 {
                self.speed -= 1;
            }
        } else {
            self.snake.pop();
        }
    }
}

impl Desktop {
    pub const fn new() -> Self {
        Desktop {
            windows: Vec::new(),
            icons: Vec::new(),
            cursor_x: 640,
            cursor_y: 400,
            cursor_visible: true,
            width: 1280,
            height: 800,
            frame_count: 0,
            start_menu_open: false,
            input_buffer: String::new(),
            cursor_blink: true,
            context_menu: Jr {
                visible: false,
                x: 0,
                y: 0,
                items: Vec::new(),
                selected_index: 0,
                target_icon: None,
                target_file: None,
            },
            cached_time_str: String::new(),
            cached_date_str: String::new(),
            last_rtc_frame: 0,
            background_cached: false,
            needs_full_redraw: true,
            last_cursor_x: 640,
            last_cursor_y: 400,
            last_window_count: 0,
            last_start_menu_open: false,
            last_context_menu_visible: false,
            render_mode: RenderMode::Classic,
            compositor_theme: CompositorTheme::Modern,
            dirty_rects: [(0, 0, 0, 0); 32],
            dirty_rect_count: 0,
            gpu_frame_skip: 0,
            browser: None,
            browser_url_input: String::new(),
            browser_url_cursor: 0,
            browser_loading: false,
            browser_url_select_all: false,
            editor_states: BTreeMap::new(),
            model_editor_states: BTreeMap::new(),
            calculator_states: BTreeMap::new(),
            snake_states: BTreeMap::new(),
            game3d_states: BTreeMap::new(),
            chess_states: BTreeMap::new(),
            chess3d_states: BTreeMap::new(),
            #[cfg(feature = "emulators")]
            nes_states: BTreeMap::new(),
            #[cfg(feature = "emulators")]
            gameboy_states: BTreeMap::new(),
            binary_viewer_states: BTreeMap::new(),
            lab_states: BTreeMap::new(),
            music_player_states: BTreeMap::new(),
            #[cfg(feature = "emulators")]
            gamelab_states: BTreeMap::new(),
            #[cfg(feature = "emulators")]
            gb_input_links: BTreeMap::new(),
            wifi_analyzer_states: BTreeMap::new(),
            scale_factor: 1,
            matrix_cols: 256,
            matrix_chars: Vec::new(),
            matrix_heads: Vec::new(),
            matrix_speeds: Vec::new(),
            matrix_seeds: Vec::new(),
            matrix_initialized: false,
            matrix_beat_count: 0,
            matrix_last_beat: false,
            matrix_rain_preset: 0, 
            matrix_overrides: BTreeMap::new(),
            matrix_projection: MatrixProjection::empty(),
            visualizer: crate::visualizer::VisualizerState::new(),
            drone_swarm: crate::drone_swarm::DroneSwarmState::new(),
            
            global_fft_re: Vec::new(),
            global_fft_im: Vec::new(),
            global_sub_bass: 0.0,
            global_bass: 0.0,
            global_mid: 0.0,
            global_treble: 0.0,
            global_energy: 0.0,
            global_beat: 0.0,
            global_peak_rms: 0.0,
            global_prev_energy: 0.0,
            global_energy_hist: Vec::new(),
            global_hist_idx: 0,
            global_hist_count: 0,
            global_audio_active: false,
            terminal_suggestion_count: 0,
            command_history: Vec::new(),
            history_index: None,
            saved_input: String::new(),
            start_menu_search: String::new(),
            start_menu_selected: -1,
            clipboard_icon: None,
            
            fm_view_modes: BTreeMap::new(),
            fm_states: BTreeMap::new(),
            image_viewer_states: BTreeMap::new(),
            file_clipboard: None,
            drag_state: None,
            settings_category: 0,
            netscan_tab: 0,
            lock_screen_active: false,
            lock_screen_input: String::new(),
            lock_screen_shake: 0,
            sys_volume: 75,
            sys_battery: 85,
            sys_wifi_connected: true,
            
            wifi_selected_index: 0,
            wifi_scroll_offset: 0,
            wifi_password_input: String::new(),
            wifi_connecting_ssid: String::new(),
            wifi_show_password: false,
            wifi_scan_requested: false,
            wifi_error_msg: None,
            
            gesture_recognizer: crate::gesture::GestureRecognizer::new(1280, 800),
            gesture_buffer: crate::gesture::GestureBuffer::new(),
            touch_mode: false,
            
            mobile_state: crate::mobile::MobileState::new(),
            
            fps_last_tick: 0,
            fps_frame_accum: 0,
            fps_current: 0,
            fps_display: true,
            desktop_tier: DesktopTier::Full,
            fps_low_count: 0,
            fps_high_count: 0,
            initial_tier: DesktopTier::Full,
            tier_cooldown: 0,
            tier_manual_override: false,
            snap_preview: None,
            show_shortcuts: false,
            windows_dirty: true,
            prev_hover_window_id: None,
            
            shutdown_active: false,
            shutdown_start_tick: 0,
            shutdown_phase: 0,
        }
    }
    
    
    pub fn init(&mut self, width: u32, height: u32) {
        crate::serial_println!("[Desktop] init start: {}x{} (clearing {} windows, {} icons)", 
            width, height, self.windows.len(), self.icons.len());
        
        
        
        self.mobile_state = crate::mobile::MobileState::new();
        
        self.windows.clear();
        self.icons.clear();
        self.editor_states.clear();
        self.model_editor_states.clear();
        self.calculator_states.clear();
        self.snake_states.clear();
        self.game3d_states.clear();
        self.chess3d_states.clear();
        #[cfg(feature = "emulators")]
        self.nes_states.clear();
        #[cfg(feature = "emulators")]
        self.gameboy_states.clear();
        self.binary_viewer_states.clear();
        self.lab_states.clear();
        self.music_player_states.clear();
        
        self.browser = None;
        self.browser_url_input.clear();
        self.browser_url_cursor = 0;
        self.browser_loading = false;
        self.browser_url_select_all = false;
        
        self.input_buffer.clear();
        self.start_menu_open = false;
        self.start_menu_search.clear();
        self.cursor_blink = false;
        self.context_menu.visible = false;
        self.context_menu.items.clear();
        self.context_menu.selected_index = 0;
        self.context_menu.target_icon = None;
        self.context_menu.target_file = None;
        
        self.frame_count = 0;
        self.terminal_suggestion_count = 0;
        self.command_history.clear();
        self.history_index = None;
        self.saved_input.clear();
        self.last_window_count = 0;
        self.last_start_menu_open = false;
        self.last_context_menu_visible = false;
        self.last_rtc_frame = 0;
        self.cached_time_str.clear();
        self.cached_date_str.clear();
        
        *BDP_.lock() = 1;
        
        crate::serial_println!("[Desktop] state cleared, windows={} icons={}", 
            self.windows.len(), self.icons.len());
        
        self.width = width;
        self.height = height;
        self.cursor_x = (width / 2) as i32;
        self.cursor_y = (height / 2) as i32;
        
        
        crate::graphics::scaling::init(width, height);
        crate::graphics::scaling::mpq(width, height);
        self.scale_factor = crate::graphics::scaling::aqv();
        crate::serial_println!("[Desktop] Text scale: {}x, UI chrome: TASKBAR={}px DOCK={}px TITLE={}px",
            self.scale_factor, V_(), BW_(), J_());
        
        
        crate::touch::init();
        crate::touch::set_screen_size(width, height);
        self.gesture_recognizer.set_screen_size(width as i32, height as i32);
        crate::serial_println!("[Desktop] Touch input initialized");
        
        
        crate::serial_println!("[Desktop] init_double_buffer...");
        framebuffer::adw();
        
        
        if framebuffer::ibg().is_some() {
            framebuffer::pr(true);
            crate::serial_println!("[Desktop] double buffer: OK");
        } else {
            framebuffer::pr(false);
            crate::serial_println!("[Desktop] WARNING: backbuffer alloc failed, using direct FB mode");
        }
        
        
        crate::serial_println!("[Desktop] init_background_cache...");
        framebuffer::mox();
        
        
        framebuffer::mpr();
        
        
        framebuffer::moy();
        
        
        moz(self);


        crate::serial_println!("[Desktop] init_compositor...");
        compositor::mpa(width, height);
        compositor::jez(self.compositor_theme);
        
        
        crate::serial_println!("[Desktop] init_desktop_icons...");
        self.init_desktop_icons();
        
        
        self.background_cached = false;
        self.needs_full_redraw = true;
        
        
        self.init_matrix_rain();
        
        
        self.detect_tier();
        
        
        
        crate::serial_println!("[Desktop] init complete (tier={:?})", self.desktop_tier);
    }
    
    
    fn init_matrix_rain(&mut self) {
        
        
        
        const ANJ_: usize = 256;
        const BNR_: usize = 1280;
        let matrix_cols = ((self.width as usize * ANJ_) / BNR_).max(ANJ_);
        self.matrix_cols = matrix_cols;
        const AHW_: usize = 4;
        const EZ_: usize = 40;   
        const Wu: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
        
        crate::serial_println!("[Rain] {}px wide -> {} cols (base=256@1280)", self.width, matrix_cols);
        let av = matrix_cols * AHW_;
        self.matrix_chars = vec![0u8; av * EZ_];
        self.matrix_heads = vec![0i32; av];
        self.matrix_speeds = vec![2u32; av];
        self.matrix_seeds = vec![0u32; av];
        
        let height = self.height.saturating_sub(V_());
        
        for col in 0..matrix_cols {
            for bj in 0..AHW_ {
                let idx = col * AHW_ + bj;
                let seed = (col as u32).wrapping_mul(2654435761)
                    ^ 0xDEADBEEF
                    ^ ((bj as u32).wrapping_mul(0x9E3779B9));
                for i in 0..EZ_ {
                    let bfe = seed.wrapping_add((i as u32).wrapping_mul(7919));
                    self.matrix_chars[idx * EZ_ + i] = Wu[(bfe as usize) % Wu.len()];
                }
                
                let cdv = height / 2 + (bj as u32) * height / 6;
                self.matrix_heads[idx] = -((seed % cdv.max(1)) as i32);
                self.matrix_speeds[idx] = 2 + (seed % 4);
                self.matrix_seeds[idx] = seed;
            }
        }
        self.matrix_initialized = true;
        
        crate::drone_swarm::init(&mut self.drone_swarm, self.width, height);
        
        
    }

    
    
    pub fn pxp(&mut self) {
        let gor: u32 = 256;
        let goq: u32 = 256;
        let screen_w = self.width;
        let screen_h = self.height.saturating_sub(V_());
        let proj_x = (screen_w / 2).saturating_sub(gor / 2);
        let proj_y = (screen_h / 2).saturating_sub(goq / 2);
        let pixels = MatrixProjection::mcl(gor, goq);
        self.matrix_projection = MatrixProjection {
            x: proj_x,
            y: proj_y,
            width: gor,
            height: goq,
            pixels,
            active: true,
        };
    }

    
    pub fn qcg(&mut self) {
        self.matrix_projection.active = false;
    }

    
    
    
    
    
    
    const BBK_: usize = 4;
    const AGN_: usize = 40;

    
    pub fn set_rain_preset(&mut self, preset: u8) {
        self.matrix_rain_preset = preset.min(2);
        crate::serial_println!("[RAIN] Speed preset set to {}", ["slow", "mid", "fast"][self.matrix_rain_preset as usize]);
    }

    
    #[inline]
    fn etx(col: usize, bj: usize, asn: usize) -> usize {
        (col * Self::BBK_ + bj) * Self::AGN_ + asn
    }

    
    
    
    pub fn matrix_override_cell(&mut self, col: usize, bj: usize, asn: usize, cell: CellPixels) -> &mut CellPixels {
        let key = Self::etx(col, bj, asn);
        self.matrix_overrides.insert(key, cell);
        
        self.matrix_overrides.get_mut(&key).unwrap_or_else(|| unreachable!())
    }

    
    
    pub fn qos(&mut self, col: usize, bj: usize, asn: usize, color: u32) -> &mut CellPixels {
        let idx = col * Self::BBK_ + bj;
        let cuz = idx * Self::AGN_ + asn;
        let c = if cuz < self.matrix_chars.len() {
            self.matrix_chars[cuz] as char
        } else {
            '#'
        };
        let cell = CellPixels::lzf(c, color);
        self.matrix_override_cell(col, bj, asn, cell)
    }

    
    pub fn qor(&mut self, col: usize, bj: usize, asn: usize) -> Option<&mut CellPixels> {
        let key = Self::etx(col, bj, asn);
        self.matrix_overrides.get_mut(&key)
    }

    
    pub fn qoo(&mut self, col: usize, bj: usize, asn: usize, p: u8, o: u8, color: u32) {
        let key = Self::etx(col, bj, asn);
        let cell = self.matrix_overrides.entry(key).or_insert_with(CellPixels::hhz);
        cell.set(p, o, color);
    }

    
    pub fn qop(&mut self, col: usize, bj: usize, asn: usize) {
        let key = Self::etx(col, bj, asn);
        self.matrix_overrides.remove(&key);
    }

    
    pub fn qoq(&mut self) {
        self.matrix_overrides.clear();
    }

    
    
    
    
    pub fn qon(&mut self, afu: usize, bj: usize, start_trail: usize,
                              tm: &[u32], wl: usize, qc: usize) {
        
        let kid = (wl + 7) / 8;
        let kic = (qc + 15) / 16;
        
        for u in 0..kic {
            for cx in 0..kid {
                let col = afu + cx;
                let asn = start_trail + u;
                if col >= self.matrix_cols || asn >= Self::AGN_ { continue; }
                
                let mut cell = CellPixels::hhz();
                for o in 0..16u8 {
                    for p in 0..8u8 {
                        let ahc = cx * 8 + p as usize;
                        let aft = u * 16 + o as usize;
                        if ahc < wl && aft < qc {
                            let color = tm[aft * wl + ahc];
                            if color & 0xFF000000 != 0 {  
                                cell.set(p, o, color);
                            }
                        }
                    }
                }
                self.matrix_override_cell(col, bj, asn, cell);
            }
        }
    }

    
    fn qqa(&mut self) {
        
        let gsh = r#"//! TrustOS — A Modern Operating System in Rust
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
        
        let _ = crate::ramfs::bh(|fs| {
            fs.write_file("/demo.rs", gsh.as_bytes())
        });
        
        let id = self.create_window("TrustCode: demo.rs", 160, 50, 780, 560, WindowType::TextEditor);
        if let Some(editor) = self.editor_states.get_mut(&id) {
            editor.load_file("demo.rs");
        }
        
        self.focus_window(id);
        crate::serial_println!("[TrustCode] Demo editor opened");
    }
    
    
    pub fn set_render_mode(&mut self, mode: RenderMode) {
        self.render_mode = mode;
        self.needs_full_redraw = true;
        self.background_cached = false;
        
        if mode == RenderMode::OpenGL {
            
            self.sync_compositor_surfaces();
        }
    }
    
    
    
    pub fn detect_tier(&mut self) {
        let aun = crate::memory::ceo() / (1024 * 1024);
        let bmt = crate::memory::heap::free() / (1024 * 1024);
        let pixels = (self.width as u64) * (self.height as u64);
        let cpus = crate::cpu::smp::cpu_count().max(1) as u64;
        
        
        let ecr = crate::cpu::hac() / 1_000_000;
        
        
        
        
        
        
        
        let obd = ((aun / 256) as i64).min(8);
        let kyr = if ecr > 0 { (ecr / 400) as i64 } else { 2 };
        let kxz = (cpus as i64) * 2;
        let ogd = ((pixels as i64) - 1_000_000) / 1_000_000;
        let score = obd + kyr + kxz - ogd;
        
        
        
        
        
        let hoh = ecr > 0 && ecr < 1500 && cpus <= 1;
        
        let gys = if aun < 128 || bmt < 8 {
            DesktopTier::CliOnly
        } else if score <= 4 || aun < 256 {
            DesktopTier::Minimal
        } else if score <= 8 || aun < 512 || hoh {
            DesktopTier::Standard
        } else {
            DesktopTier::Full
        };
        
        self.desktop_tier = gys;
        self.initial_tier = gys;
        self.fps_low_count = 0;
        self.fps_high_count = 0;
        
        crate::serial_println!(
            "[Desktop] Tier={:?} (score={}, RAM={}MB, heap={}MB, CPUs={}, TSC={}MHz, {}x{}, cpu_limited={})",
            gys, score, aun, bmt, cpus, ecr, self.width, self.height, hoh
        );
    }
    
    
    
    
    fn auto_adjust_tier(&mut self) {
        
        if self.tier_manual_override { return; }
        
        
        
        if GQ_.load(Ordering::Relaxed) { return; }
        
        if self.fps_current == 0 { return; }
        
        
        if self.frame_count < 120 { return; }
        
        if self.tier_cooldown > 0 {
            self.tier_cooldown -= 1;
            self.fps_low_count = 0;
            self.fps_high_count = 0;
            return;
        }
        
        
        
        
        if self.fps_current < 18 {
            
            let moo = if self.fps_current <= 2 { 60 } else if self.fps_current < 10 { 4 } else { 1 };
            self.fps_low_count += moo;
            self.fps_high_count = 0;
        } else if self.fps_current >= 35 {
            
            self.fps_high_count += 1;
            if self.fps_low_count > 0 {
                self.fps_low_count = self.fps_low_count.saturating_sub(4);
            }
        } else {
            
            if self.fps_low_count > 0 {
                self.fps_low_count = self.fps_low_count.saturating_sub(2);
            }
            self.fps_high_count = 0;
        }
        
        
        if self.fps_low_count >= 120 {
            let qb = self.desktop_tier;
            
            let akg = if self.fps_current <= 2 {
                match qb {
                    DesktopTier::Full | DesktopTier::Standard => DesktopTier::Minimal,
                    _ => qb,
                }
            } else {
                match qb {
                    DesktopTier::Full => DesktopTier::Standard,
                    DesktopTier::Standard => DesktopTier::Minimal,
                    _ => qb,
                }
            };
            if akg != qb {
                self.desktop_tier = akg;
                
                
                self.initial_tier = akg;
                self.fps_low_count = 0;
                self.fps_high_count = 0;
                self.tier_cooldown = 180;
                self.needs_full_redraw = true;
                self.background_cached = false;
                framebuffer::ihi();
                crate::serial_println!(
                    "[Desktop] Auto-downgrade: {:?} -> {:?} (FPS was {}, ceiling now {:?})",
                    qb, akg, self.fps_current, self.initial_tier
                );
            }
        }
        
        
        if self.fps_high_count >= 200 {
            let qb = self.desktop_tier;
            let akg = match qb {
                DesktopTier::Minimal => DesktopTier::Standard,
                DesktopTier::Standard => DesktopTier::Full,
                _ => qb,
            };
            
            if akg != qb && akg <= self.initial_tier {
                self.desktop_tier = akg;
                self.fps_high_count = 0;
                self.fps_low_count = 0;
                self.tier_cooldown = 180; 
                self.needs_full_redraw = true;
                self.background_cached = false;
                framebuffer::ihi();
                crate::serial_println!(
                    "[Desktop] Auto-upgrade: {:?} -> {:?} (FPS was {})",
                    qb, akg, self.fps_current
                );
            } else {
                self.fps_high_count = 0;
            }
        }
    }
    
    
    pub fn set_theme(&mut self, theme: CompositorTheme) {
        self.compositor_theme = theme;
        compositor::jez(theme);
        self.needs_full_redraw = true;
    }
    
    
    fn sync_compositor_surfaces(&self) {
        let mut bfm = compositor::compositor();
        bfm.surfaces.clear();
        
        for window in &self.windows {
            if window.visible {
                let mut surface = WindowSurface::new(
                    window.id,
                    window.x as f32,
                    window.y as f32,
                    window.width as f32,
                    window.height as f32,
                );
                surface.z_order = 0;
                surface.focused = window.focused;
                surface.visible = !window.minimized;
                bfm.surfaces.push(surface);
            }
        }
    }
    
    
    fn init_desktop_icons(&mut self) {
        use crate::icons::IconType;
        
        
        let axn = 50u32; 
        let start_y = 12u32;
        let bi = 12u32;
        
        let fsr: &[(&str, IconType, IconAction)] = &[
            ("Terminal", IconType::Terminal, IconAction::OpenTerminal),
            ("Files", IconType::Folder, IconAction::OpenFileManager),
            ("Editor", IconType::Editor, IconAction::OpenEditor),
            ("Calc", IconType::Calculator, IconAction::OpenCalculator),
            ("NetScan", IconType::Network, IconAction::OpenNetwork),
            ("Chess 3D", IconType::Chess, IconAction::OpenGame),

            ("Browser", IconType::Browser, IconAction::OpenBrowser),
            ("TrustEd", IconType::ModelEditor, IconAction::OpenModelEditor),
            ("Settings", IconType::Settings, IconAction::OpenSettings),
            ("Music", IconType::Music, IconAction::OpenMusicPlayer),
            #[cfg(feature = "emulators")]
            ("GameBoy", IconType::GameBoy, IconAction::OpenGameBoy),
            #[cfg(feature = "emulators")]
            ("GameLab", IconType::GameLab, IconAction::OpenGameLab),
        ];
        
        for (i, (name, icon_type, action)) in fsr.iter().enumerate() {
            self.icons.push(Rr {
                name: String::from(*name),
                icon_type: *icon_type,
                x: bi,
                y: start_y + i as u32 * axn,
                action: *action,
            });
        }
    }
    
    
    fn pzm(&self, x: i32, y: i32) -> Option<IconAction> {
        
        if x < 0 || x >= (BW_() + 10) as i32 { return None; }
        
        let atn = self.height.saturating_sub(V_());
        let cbu = self.icons.len().max(1) as u32;
        let padding = 12u32;
        let available = atn.saturating_sub(padding * 2);
        let axn = (available / cbu) as i32;
        let start_y = (padding + (available - axn as u32 * cbu) / 2) as i32;
        
        for (i, icon) in self.icons.iter().enumerate() {
            let gg = start_y + i as i32 * axn;
            if y >= gg - 3 && y < gg + axn as i32 {
                return Some(icon.action);
            }
        }
        None
    }
    
    
    pub fn create_window(&mut self, title: &str, x: i32, y: i32, width: u32, height: u32, wt: WindowType) -> u32 {
        
        let pqe = self.width.saturating_sub(BW_() + 4);
        let pqd = self.height.saturating_sub(V_() + J_());
        let w = width.min(pqe).max(120);
        let h = height.min(pqd).max(80);
        
        let ayg = BW_() as i32 + 2;
        let aly = (self.width as i32 - w as i32).max(ayg);
        let aye = (self.height as i32 - V_() as i32 - h as i32).max(0);
        let cx = x.max(ayg).min(aly);
        let u = y.max(0).min(aye);

        let mut window = Window::new(title, cx, u, w, h, wt);
        
        
        match wt {
            WindowType::Terminal => {
                window.content.push(String::from("\x01HTrustOS Terminal v1.0"));
                window.content.push(String::from("\x01MType \x01Ghelp\x01M for available commands."));
                window.content.push(String::from(""));
                window.content.push(Self::aya("_"));
            },
            WindowType::SystemInfo => {
                window.content.push(String::from("=== System Information ==="));
                window.content.push(String::from(""));
                window.content.push(format!("OS: TrustOS v0.2.0"));
                window.content.push(format!("Arch: x86_64"));
                window.content.push(format!("Display: {}x{}", self.width, self.height));
                window.content.push(String::from("Kernel: trustos_kernel"));
            },
            WindowType::About => {
                window.content.push(String::from("TrustOS"));
                window.content.push(String::from(""));
                window.content.push(String::from("A modern operating system"));
                window.content.push(String::from("written in Rust"));
                window.content.push(String::from(""));
                window.content.push(String::from("(c) 2026 Nathan"));
            },
            WindowType::Calculator => {
                self.calculator_states.insert(window.id, CalculatorState::new());
            },
            WindowType::FileManager => {
                window.content.push(String::from("=== File Manager ==="));
                window.content.push(String::from("Path: /"));
                window.content.push(String::from(""));
                window.content.push(String::from("  Name              Type       Size    Program"));
                window.content.push(String::from("  ────────────────────────────────────────────"));
                window.file_path = Some(String::from("/"));
                
                let mut hzj = FileManagerState::new();
                hzj.push_history("/");
                self.fm_states.insert(window.id, hzj);
                
                if let Ok(entries) = crate::ramfs::bh(|fs| fs.ls(Some("/"))) {
                    for (name, wf, size) in entries.iter().take(50) {
                        let icon = if *wf == crate::ramfs::FileType::Directory { 
                            "[D]" 
                        } else { 
                            crate::file_assoc::get_file_icon(name)
                        };
                        let azb = if *wf == crate::ramfs::FileType::Directory {
                            String::from("---")
                        } else {
                            String::from(crate::file_assoc::cyr(name).name())
                        };
                        let fxy = if *wf == crate::ramfs::FileType::Directory { "DIR" } else { "FILE" };
                        window.content.push(format!("  {} {:<14} {:<10} {:<7} {}", icon, name, fxy, size, azb));
                    }
                }
                if window.content.len() <= 5 {
                    window.content.push(String::from("  (empty directory)"));
                }
                window.content.push(String::from(""));
                window.content.push(String::from("  [Enter] Open | [Up/Down] Navigate"));
            },
            WindowType::TextEditor => {
                
                
                let mut editor = EditorState::new();
                let hyg = self.editor_states.len() + 1;
                let lcz = if hyg == 1 {
                    String::from("untitled.rs")
                } else {
                    alloc::format!("untitled_{}.rs", hyg)
                };
                editor.file_path = Some(lcz);
                editor.language = crate::apps::text_editor::Language::Rust;
                self.editor_states.insert(window.id, editor);
            },
            WindowType::Cn => {
                
            },
            WindowType::Settings => {
                window.content.push(String::from("=== Settings ==="));
                window.content.push(String::from(""));
                window.content.push(format!("Resolution: {}x{}", self.width, self.height));
                window.content.push(String::from("Theme: Dark Green"));
                window.content.push(String::from(""));
                window.content.push(String::from("--- Animations ---"));
                let dhs = if awb() { "ON " } else { "OFF" };
                let fhc = *GP_.lock();
                window.content.push(format!("[1] Animations: {}", dhs));
                window.content.push(format!("[2] Speed: {:.1}x", fhc));
                window.content.push(String::from(""));
                window.content.push(String::from("--- Accessibility ---"));
                let ads = if crate::accessibility::btq() { "ON " } else { "OFF" };
                window.content.push(format!("[5] High Contrast: {}", ads));
                window.content.push(format!("[6] Font Size: {}", crate::accessibility::cyn().label()));
                window.content.push(format!("[7] Cursor Size: {}", crate::accessibility::cyl().label()));
                window.content.push(format!("[8] Sticky Keys: {}", if crate::accessibility::bnc() { "ON" } else { "OFF" }));
                window.content.push(format!("[9] Mouse Speed: {}", crate::accessibility::cyq().label()));
                window.content.push(String::from(""));
                window.content.push(String::from("--- Other ---"));
                window.content.push(String::from("[3] File Associations"));
                window.content.push(String::from("[4] About System"));
            },
            WindowType::ImageViewer => {
                window.content.push(String::from("=== Image Viewer ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("No image loaded"));
                window.content.push(String::from(""));
                window.content.push(String::from("Supported: PNG, JPG, BMP, GIF"));
            },
            WindowType::HexViewer => {
                window.content.push(String::from("=== Hex Viewer ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("No file loaded"));
            },
            WindowType::Demo3D => {
                window.content.push(String::from("=== 3D Graphics Demo ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("TrustOS Graphics Engine"));
                window.content.push(String::from("Software 3D Renderer"));
                window.content.push(String::from(""));
                window.content.push(String::from("Features:"));
                window.content.push(String::from("- Wireframe/Solid/Mixed modes"));
                window.content.push(String::from("- Z-buffer depth testing"));
                window.content.push(String::from("- Flat shading with lighting"));
                window.content.push(String::from("- Perspective projection"));
                window.content.push(String::from("- Backface culling"));
                window.content.push(String::from(""));
                window.content.push(String::from("[Rotating Cube Demo Below]"));
            },
            WindowType::FileAssociations => {
                window.content.push(String::from("=== File Associations ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("Extension | Program       | Type"));
                window.content.push(String::from("----------|---------------|-------------"));
                
                let fhq = crate::file_assoc::iko();
                for (ext, azb, desc) in fhq.iter().take(15) {
                    window.content.push(format!(".{:<8} | {:<13} | {}", ext, azb, desc));
                }
                window.content.push(String::from(""));
                window.content.push(String::from("Click extension to change program"));
            },
            WindowType::Browser => {
                
                if self.browser.is_none() {
                    self.browser = Some(crate::browser::Browser::new(width, height));
                }
                self.browser_url_input = String::from("http://example.com");
                    self.browser_url_cursor = self.browser_url_input.len();
            },
            WindowType::ModelEditor => {
                let state = crate::model_editor::ModelEditorState::new();
                self.model_editor_states.insert(window.id, state);
            },
            WindowType::Game => {
                self.snake_states.insert(window.id, SnakeState::new());
            },
            WindowType::Game3D => {
                self.game3d_states.insert(window.id, crate::game3d::Game3DState::new());
            },
            WindowType::Chess => {
                self.chess_states.insert(window.id, crate::chess::ChessState::new());
            },
            WindowType::Chess3D => {
                self.chess3d_states.insert(window.id, crate::chess3d::Chess3DState::new());
            },
            #[cfg(feature = "emulators")]
            WindowType::NesEmu => {
                let mut an = crate::nes::NesEmulator::new();
                
                if let Some(rom_data) = crate::embedded_roms::nif() {
                    an.load_rom(rom_data);
                }
                self.nes_states.insert(window.id, an);
            },
            #[cfg(feature = "emulators")]
            WindowType::GameBoyEmu => {
                let mut an = crate::gameboy::GameBoyEmulator::new();
                
                if let Some(rom_data) = crate::embedded_roms::mbe() {
                    an.load_rom(rom_data);
                }
                self.gameboy_states.insert(window.id, an);
            },
            WindowType::BinaryViewer => {
                
            },
            WindowType::LabMode => {
                self.lab_states.insert(window.id, crate::lab_mode::LabState::new());
            },
            #[cfg(feature = "emulators")]
            WindowType::GameLab => {
                self.gamelab_states.insert(window.id, crate::game_lab::GameLabState::new());
            },
            WindowType::MusicPlayer => {
                crate::serial_println!("[Desktop] Creating MusicPlayer state for window {}", window.id);
                let mut ic = MusicPlayerState::new();
                ic.load_track_list();
                self.music_player_states.insert(window.id, ic);
                crate::serial_println!("[Desktop] MusicPlayer state created OK");
            },
            WindowType::WifiNetworks => {
                self.wifi_selected_index = 0;
                self.wifi_scroll_offset = 0;
                self.wifi_error_msg = None;
                let _ = crate::drivers::net::wifi::eaj();
            },
            WindowType::WifiPassword => {
                self.wifi_password_input.clear();
                self.wifi_show_password = false;
                self.wifi_error_msg = None;
            },
            WindowType::WifiAnalyzer => {
                self.wifi_analyzer_states.insert(window.id, crate::wifi_analyzer::WifiAnalyzerState::new());
            },
            _ => {}
        }
        
        
        window.animate_open();
        
        let id = window.id;
        self.windows.push(window);
        self.windows_dirty = true;
        framebuffer::eqy();
        id
    }
    
    
    pub fn close_window(&mut self, id: u32) {
        crate::serial_println!("[GUI] close_window({}) start", id);
        self.windows_dirty = true;
        framebuffer::eqy();
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            if w.animate_close() {
                crate::serial_println!("[GUI] close_window({}) animate path", id);
                
                
                #[cfg(feature = "emulators")]
                self.gameboy_states.remove(&id);
                #[cfg(feature = "emulators")]
                self.nes_states.remove(&id);
                self.game3d_states.remove(&id);
                self.chess3d_states.remove(&id);
                #[cfg(feature = "emulators")]
                self.gamelab_states.remove(&id);
                self.lab_states.remove(&id);
                self.wifi_analyzer_states.remove(&id);
                
                if let Some(ic) = self.music_player_states.get_mut(&id) {
                    crate::serial_println!("[GUI] close_window({}) stopping music...", id);
                    ic.stop();
                    crate::serial_println!("[GUI] close_window({}) music stopped", id);
                }
                crate::serial_println!("[GUI] close_window({}) removing mp state...", id);
                self.music_player_states.remove(&id);
                crate::serial_println!("[GUI] close_window({}) animate path done", id);
                return;
            }
        }
        crate::serial_println!("[GUI] close_window({}) immediate remove path", id);
        
        self.windows.retain(|w| w.id != id);
        
        self.editor_states.remove(&id);
        self.model_editor_states.remove(&id);
        self.calculator_states.remove(&id);
        self.snake_states.remove(&id);
        self.game3d_states.remove(&id);
        self.chess_states.remove(&id);
        self.chess3d_states.remove(&id);
        #[cfg(feature = "emulators")]
        self.nes_states.remove(&id);
        #[cfg(feature = "emulators")]
        self.gameboy_states.remove(&id);
        self.binary_viewer_states.remove(&id);
        self.lab_states.remove(&id);
        self.wifi_analyzer_states.remove(&id);
        if let Some(ic) = self.music_player_states.get_mut(&id) {
            crate::serial_println!("[GUI] close_window({}) stopping music (imm)...", id);
            ic.stop();
            crate::serial_println!("[GUI] close_window({}) music stopped (imm)", id);
        }
        crate::serial_println!("[GUI] close_window({}) removing mp state (imm)...", id);
        self.music_player_states.remove(&id);
        crate::serial_println!("[GUI] close_window({}) immediate path done", id);
        #[cfg(feature = "emulators")]
        self.gamelab_states.remove(&id);
        #[cfg(feature = "emulators")]
        self.gb_input_links.remove(&id);
    }
    
    
    pub fn minimize_window(&mut self, id: u32) {
        let bwh = (self.height - V_()) as i32;
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            if !w.minimized {
                w.animate_minimize(bwh);
            }
            w.minimized = !w.minimized;
        }
    }
    
    
    pub fn update_animations(&mut self) {
        let mut aph = Vec::new();
        let mut hfh = false;
        
        for w in &mut self.windows {
            if w.animation.state != AnimationState::None {
                hfh = true;
                w.dirty = true;
            }
            if w.update_animation() {
                
                aph.push(w.id);
            }
        }
        
        if hfh || !aph.is_empty() {
            self.windows_dirty = true;
            framebuffer::eqy();
        }
        
        
        for id in aph {
            self.windows.retain(|w| w.id != id);
            self.editor_states.remove(&id);
            self.model_editor_states.remove(&id);
            self.game3d_states.remove(&id);
            self.chess3d_states.remove(&id);
            #[cfg(feature = "emulators")]
            self.nes_states.remove(&id);
            #[cfg(feature = "emulators")]
            self.gameboy_states.remove(&id);
            #[cfg(feature = "emulators")]
            self.gamelab_states.remove(&id);
        }
    }
    
    
    pub fn focus_window(&mut self, id: u32) {
        for w in &mut self.windows {
            w.focused = false;
            w.dirty = true;
        }
        if let Some(idx) = self.windows.iter().position(|w| w.id == id) {
            let mut window = self.windows.remove(idx);
            window.focused = true;
            window.minimized = false;
            window.dirty = true;
            self.windows.push(window);
            self.windows_dirty = true;
        }
    }
    
    
    
    
    
    
    pub fn screen_width(&self) -> u32 { self.width }
    pub fn screen_height(&self) -> u32 { self.height }
    
    
    pub fn close_focused_window(&mut self) {
        if let Some(id) = self.windows.iter().rev().find(|w| w.focused).map(|w| w.id) {
            self.close_window(id);
        }
    }
    
    
    pub fn minimize_focused_window(&mut self) {
        if let Some(id) = self.windows.iter().rev().find(|w| w.focused).map(|w| w.id) {
            self.minimize_window(id);
        }
    }
    
    
    pub fn toggle_maximize_focused(&mut self) {
        if let Some(id) = self.windows.iter().rev().find(|w| w.focused).map(|w| w.id) {
            let (dy, dw) = (self.width, self.height);
            if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                w.toggle_maximize(dy, dw);
            }
        }
    }
    
    
    pub fn snap_focused_window(&mut self, it: SnapDir) {
        if let Some(w) = self.windows.iter_mut().rev().find(|w| w.focused) {
            let hcl = self.height.saturating_sub(V_());
            let bah = BW_() as i32;
            let hcm = self.width.saturating_sub(BW_());
            let nk = hcm / 2;
            let kh = hcl / 2;
            
            match it {
                SnapDir::Left => {
                    w.x = bah;
                    w.y = 0;
                    w.width = nk;
                    w.height = hcl;
                }
                SnapDir::Right => {
                    w.x = bah + nk as i32;
                    w.y = 0;
                    w.width = nk;
                    w.height = hcl;
                }
                SnapDir::TopLeft => {
                    w.x = bah;
                    w.y = 0;
                    w.width = nk;
                    w.height = kh;
                }
                SnapDir::TopRight => {
                    w.x = bah + nk as i32;
                    w.y = 0;
                    w.width = nk;
                    w.height = kh;
                }
                SnapDir::BottomLeft => {
                    w.x = bah;
                    w.y = kh as i32;
                    w.width = nk;
                    w.height = kh;
                }
                SnapDir::BottomRight => {
                    w.x = bah + nk as i32;
                    w.y = kh as i32;
                    w.width = nk;
                    w.height = kh;
                }
            }
            w.maximized = false;
        }
    }
    
    
    pub fn toggle_show_desktop(&mut self) {
        
        let jun = self.windows.iter().all(|w| w.minimized);
        
        
        for w in &mut self.windows {
            w.minimized = !jun;
        }
    }
    
    
    pub fn focus_window_by_index(&mut self, index: usize) {
        if index < self.windows.len() {
            
            let visible: Vec<u32> = self.windows.iter()
                .filter(|w| !w.minimized)
                .map(|w| w.id)
                .collect();
            
            if index < visible.len() {
                self.focus_window(visible[index]);
            }
        }
    }
    
    
    pub fn qja(&self) -> Vec<String> {
        self.windows.iter()
            .filter(|w| !w.minimized)
            .map(|w| w.title.clone())
            .collect()
    }
    
    
    pub fn get_window_info(&self) -> Vec<(String, WindowType)> {
        self.windows.iter()
            .filter(|w| !w.minimized)
            .map(|w| (w.title.clone(), w.window_type.clone()))
            .collect()
    }
    
    
    pub fn qpz(&mut self) {
        let id = self.create_window("Terminal", 100, 60, 780, 540, WindowType::Terminal);
        self.focus_window(id);
    }

    
    pub fn handle_click(&mut self, x: i32, y: i32, pressed: bool) {
        
        self.windows_dirty = true;
        
        if self.mobile_state.active {
            let vx = self.mobile_state.vp_x;
            let vy = self.mobile_state.vp_y;
            let bt = self.mobile_state.vp_w as i32;
            let ex = self.mobile_state.vp_h as i32;
            
            if x >= vx && x < vx + bt && y >= vy && y < vy + ex {
                let afh = x - vx;
                let ta = y - vy;
                let bsj = if pressed {
                    crate::mobile::GestureEvent::TapDown(afh, ta)
                } else {
                    crate::mobile::GestureEvent::TapUp(afh, ta)
                };
                let action = crate::mobile::mhv(&mut self.mobile_state, bsj);
                self.apply_mobile_action(action);
            }
            return;
        }
        
        
        if self.lock_screen_active { return; }
        
        
        if !pressed && self.drag_state.is_some() {
            self.finish_drag(x, y);
            return;
        }
        if pressed && self.drag_state.is_some() {
            self.update_drag(x, y);
        }
        
        if pressed {
            
            if self.context_menu.visible {
                if let Some(action) = self.check_context_menu_click(x, y) {
                    self.execute_context_action(action);
                }
                self.context_menu.visible = false;
                return;
            }
            
            
            crate::mouse::odr();
            
            
            if self.start_menu_open {
                if let Some(action) = self.check_start_menu_click(x, y) {
                    self.start_menu_open = false;
                    self.start_menu_search.clear();
                    self.handle_menu_action(action);
                    return;
                }
                
                if y < (self.height - V_()) as i32 || x >= 108 {
                    self.start_menu_open = false;
                    self.start_menu_search.clear();
                    return;
                }
            }
            
            
            if y >= (self.height - V_()) as i32 {
                self.handle_taskbar_click(x, y);
                return;
            }
            
            
            for i in (0..self.windows.len()).rev() {
                if self.windows[i].contains(x, y) {
                    let id = self.windows[i].id;
                    
                    if self.windows[i].on_close_button(x, y) {
                        self.close_window(id);
                        return;
                    }
                    
                    if self.windows[i].on_maximize_button(x, y) {
                        let (dy, dw) = (self.width, self.height);
                        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                            w.toggle_maximize(dy, dw);
                        }
                        return;
                    }
                    
                    if self.windows[i].on_minimize_button(x, y) {
                        self.minimize_window(id);
                        return;
                    }
                    
                    
                    
                    let grh = self.windows[i].on_resize_edge(x, y);
                    let iil = matches!(grh, ResizeEdge::Top | ResizeEdge::TopLeft | ResizeEdge::TopRight);
                    if grh != ResizeEdge::None && !iil {
                        self.windows[i].resizing = grh;
                        self.windows[i].drag_offset_x = x;
                        self.windows[i].drag_offset_y = y;
                        self.focus_window(id);
                        return;
                    }
                    
                    
                    if self.windows[i].in_title_bar(x, y) || iil {
                        
                        if crate::mouse::erj() {
                            crate::mouse::jah();
                            let (dy, dw) = (self.width, self.height);
                            if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                                w.toggle_maximize(dy, dw);
                            }
                            return;
                        }
                        
                        let nw = self.windows[i].x;
                        let qr = self.windows[i].y;
                        self.windows[i].dragging = true;
                        self.windows[i].drag_offset_x = x - nw;
                        self.windows[i].drag_offset_y = y - qr;
                    }
                    
                    
                    if self.windows[i].window_type == WindowType::Browser {
                        crate::serial_println!("[CLICK-DBG] Browser window {} clicked at ({},{})", self.windows[i].id, x, y);
                        let bx = self.windows[i].x;
                        let dc = self.windows[i].y;
                        let fv = self.windows[i].width;
                        self.handle_browser_click(x, y, bx, dc, fv);
                    }
                    
                    
                    if self.windows[i].window_type == WindowType::FileManager {
                        let lxe = self.windows[i].id;
                        self.handle_file_manager_click(x, y, lxe);
                    }
                    
                    
                    if self.windows[i].window_type == WindowType::ModelEditor {
                        let aw = &self.windows[i];
                        let vx = x - aw.x;
                        let vy = y - aw.y - J_() as i32;
                        let bt = aw.width as usize;
                        let ex = aw.height.saturating_sub(J_()) as usize;
                        let fr = aw.id;
                        if vy >= 0 {
                            if let Some(state) = self.model_editor_states.get_mut(&fr) {
                                state.handle_click(vx, vy, bt, ex, true);
                            }
                        }
                    }
                    
                    
                    if self.windows[i].window_type == WindowType::Chess {
                        let aw = &self.windows[i];
                        let yu = aw.x as i32 + 8;
                        let xp = aw.y as i32 + J_() as i32 + 4;
                        let ajs = aw.width.saturating_sub(16) as i32;
                        let cell_size: i32 = 48;
                        let tg = cell_size * 8;
                        let un = yu + (ajs - tg) / 2;
                        let ve = xp + 28;
                        
                        let col = (x - un) / cell_size;
                        let row = (y - ve) / cell_size;
                        
                        if x >= un && x < un + tg && y >= ve && y < ve + tg && col >= 0 && col < 8 && row >= 0 && row < 8 {
                            let fr = aw.id;
                            if let Some(chess) = self.chess_states.get_mut(&fr) {
                                chess.handle_mouse_click(col, row);
                                chess.update_drag_position(x, y);
                            }
                        }
                    }
                    
                    
                    if self.windows[i].window_type == WindowType::Chess3D {
                        let aw = &self.windows[i];
                        let ho = aw.x as i32;
                        let bn = aw.y as i32 + J_() as i32;
                        let hy = aw.width as i32;
                        let en = aw.height.saturating_sub(J_()) as i32;
                        let sk = x - ho;
                        let qn = y - bn;
                        if sk >= 0 && qn >= 0 && sk < hy && qn < en {
                            let fr = aw.id;
                            if let Some(state) = self.chess3d_states.get_mut(&fr) {
                                state.handle_click(sk, qn, hy, en);
                            }
                        }
                    }
                    
                    
                    if self.windows[i].window_type == WindowType::LabMode {
                        let aw = &self.windows[i];
                        let sk = x - aw.x;
                        let qn = y - aw.y;
                        let fr = aw.id;
                        let ca = aw.width;
                        let er = aw.height;
                        if let Some(lab) = self.lab_states.get_mut(&fr) {
                            lab.handle_click(sk, qn, ca, er);
                        }
                    }

                    
                    if self.windows[i].window_type == WindowType::WifiAnalyzer {
                        let aw = &self.windows[i];
                        let sk = x - aw.x;
                        let qn = y - aw.y;
                        let fr = aw.id;
                        let ca = aw.width;
                        let er = aw.height;
                        if let Some(apn) = self.wifi_analyzer_states.get_mut(&fr) {
                            apn.handle_click(sk, qn, ca, er);
                        }
                    }

                    
                    if self.windows[i].window_type == WindowType::WifiNetworks {
                        let aw = &self.windows[i];
                        let fr = aw.id;
                        self.handle_wifi_networks_click(x, y, fr);
                    }

                    
                    if self.windows[i].window_type == WindowType::WifiPassword {
                        let aw = &self.windows[i];
                        let fr = aw.id;
                        self.handle_wifi_password_click(x, y, fr);
                    }

                    
                    #[cfg(feature = "emulators")]
                    if self.windows[i].window_type == WindowType::GameBoyEmu {
                        let aw = &self.windows[i];
                        let ho = aw.x as u32;
                        let bn = (aw.y + J_() as i32) as u32;
                        let hy = aw.width;
                        let rv: u32 = 22;
                        let fr = aw.id;
                        let nw = aw.x;
                        let qr = aw.y;
                        let ul = aw.width;
                        let afy = aw.height;
                        let cg = x as u32;
                        let cr = y as u32;
                        
                        
                        if cr >= bn && cr < bn + rv {
                            
                            let cld: u32 = 48;
                            let btk = ho + hy - cld - 4;
                            if cg >= btk && cg < btk + cld {
                                
                                let mqk = nw;
                                let mql = qr + afy as i32 + 2;
                                let mqj = self.create_window("GB Input", mqk, mql, ul.min(480), 160, WindowType::GameBoyInput);
                                self.gb_input_links.insert(mqj, fr);
                            }
                            
                            
                            let clu: u32 = 32;
                            let clv = btk - clu - 6;
                            if cg >= clv && cg < clv + clu {
                                
                                let dy = self.width;
                                let dw = self.height;
                                let dtd = nw + ul as i32 + 4;
                                let gex = (dy as i32 - dtd).max(400) as u32;
                                let gew = dw - V_();
                                let cbh = self.create_window("Game Lab", dtd, 0, gex, gew, WindowType::GameLab);
                                if let Some(lab) = self.gamelab_states.get_mut(&cbh) {
                                    lab.linked_gb_id = Some(fr);
                                }
                                self.focus_window(cbh);
                            }
                        }
                    }

                    
                    #[cfg(feature = "emulators")]
                    if self.windows[i].window_type == WindowType::GameBoyInput {
                        let aw = &self.windows[i];
                        let cx = aw.x as u32;
                        let u = (aw.y + J_() as i32) as u32;
                        let aq = aw.width;
                        let ch = aw.height.saturating_sub(J_());
                        let fr = aw.id;
                        let cg = x as u32;
                        let cr = y as u32;
                        
                        let cmh = self.gb_input_links.get(&fr).copied();
                        let buttons = crate::game_lab::mdf(cx, u, aq, ch);
                        for &(bx, dc, fv, ov, key) in &buttons {
                            if cg >= bx && cg < bx + fv && cr >= dc && cr < dc + ov {
                                
                                let bsf = cmh.or_else(|| self.gameboy_states.keys().next().copied());
                                if let Some(bbh) = bsf {
                                    if let Some(an) = self.gameboy_states.get_mut(&bbh) {
                                        an.handle_key(key);
                                    }
                                }
                                break;
                            }
                        }
                    }

                    
                    #[cfg(feature = "emulators")]
                    if self.windows[i].window_type == WindowType::GameLab {
                        let aw = &self.windows[i];
                        let sk = x - aw.x;
                        let qn = y - aw.y;
                        let fr = aw.id;
                        let ca = aw.width;
                        let er = aw.height;
                        if let Some(lab) = self.gamelab_states.get_mut(&fr) {
                            
                            let ezj = ca as i32 - 120;
                            if qn >= J_() as i32 + 2 && qn < J_() as i32 + 18 {
                                if sk >= ezj && sk < ezj + 48 {
                                    
                                    let bsf = lab.linked_gb_id
                                        .or_else(|| self.gameboy_states.keys().next().copied());
                                    if let Some(bbh) = bsf {
                                        if let Some(an) = self.gameboy_states.get(&bbh) {
                                            if let Some(gl) = self.gamelab_states.get_mut(&fr) {
                                                gl.save_from(an);
                                                crate::serial_println!("[GameLab] State saved (click)");
                                            }
                                        }
                                    }
                                } else if sk >= ezj + 54 && sk < ezj + 102 {
                                    
                                    let bsf = lab.linked_gb_id
                                        .or_else(|| self.gameboy_states.keys().next().copied());
                                    if let Some(bbh) = bsf {
                                        let valid = self.gamelab_states.get(&fr)
                                            .map(|l| l.save_state.valid).unwrap_or(false);
                                        if valid {
                                            if let Some(an) = self.gameboy_states.get_mut(&bbh) {
                                                if let Some(gl) = self.gamelab_states.get(&fr) {
                                                    gl.load_into(an);
                                                    crate::serial_println!("[GameLab] State loaded (click)");
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                lab.handle_click(sk, qn, ca, er);
                            }
                        }
                    }

                    
                    if self.windows[i].window_type == WindowType::Settings {
                        let aw = &self.windows[i];
                        let wx = aw.x;
                        let wy = aw.y;
                        let bn = wy + J_() as i32;
                        let rq = 140i32;
                        let sy = 32i32;
                        
                        
                        if x >= wx && x < wx + rq && y >= bn + 8 {
                            let idx = ((y - bn - 8) / sy) as u8;
                            if idx <= 7 {
                                self.settings_category = idx;
                            }
                        }
                        
                        
                        if self.settings_category == 0 && x >= wx + rq {
                            let p = wx + rq + 20;
                            let bw = 22i32;
                            
                            
                            
                            
                            
                            let bet = bn + 16;
                            let ghy = bet + (bw + 8) + bw + (bw + 8) + bw + (bw + 8) + bw + bw;
                            let itb = ghy + bw;
                            
                            
                            if y >= ghy && y < ghy + bw {
                                let akg = match self.desktop_tier {
                                    DesktopTier::Full => DesktopTier::Standard,
                                    DesktopTier::Standard => DesktopTier::Minimal,
                                    DesktopTier::Minimal | DesktopTier::CliOnly => DesktopTier::Full,
                                };
                                self.desktop_tier = akg;
                                self.tier_manual_override = true;
                                self.fps_low_count = 0;
                                self.fps_high_count = 0;
                                self.needs_full_redraw = true;
                                self.background_cached = false;
                                crate::serial_println!("[Desktop] Manual tier change (click): {:?}", akg);
                            }
                            
                            if y >= itb && y < itb + bw {
                                self.tier_manual_override = !self.tier_manual_override;
                                self.fps_low_count = 0;
                                self.fps_high_count = 0;
                                crate::serial_println!("[Desktop] Manual override toggle (click): {}", self.tier_manual_override);
                            }
                        }
                    }
                    
                    
                    if self.windows[i].window_type == WindowType::BinaryViewer {
                        let aw = &self.windows[i];
                        let sk = x - aw.x;
                        let qn = y - aw.y;
                        let fr = aw.id;
                        let ca = aw.width;
                        let er = aw.height;
                        if let Some(viewer) = self.binary_viewer_states.get_mut(&fr) {
                            viewer.handle_click(sk, qn, ca, er);
                        }
                    }
                    
                    
                    if self.windows[i].window_type == WindowType::Calculator {
                        let aw = &self.windows[i];
                        let lav = aw.x as u32 + 4;
                        let lax = aw.y as u32 + J_() + 4;
                        let aq = aw.width.saturating_sub(8);
                        let ch = aw.height.saturating_sub(J_() + 8);
                        let atm = 56u32;
                        let ehf = lax + atm + 12;
                        let djs = 4u32;
                        let djt = 5u32;
                        let rj = 4u32;
                        let gu = (aq - 12 - rj * (djs - 1)) / djs;
                        let hn = ((ch - atm - 20 - rj * (djt - 1)) / djt).min(40);
                        
                        let qh = x as u32;
                        let abv = y as u32;
                        
                        if abv >= ehf {
                            let buttons = [
                                ['C', '(', ')', '%'],
                                ['7', '8', '9', '/'],
                                ['4', '5', '6', '*'],
                                ['1', '2', '3', '-'],
                                ['0', '.', '=', '+'],
                            ];
                            
                            for (row, btn_row) in buttons.iter().enumerate() {
                                for (col, &label) in btn_row.iter().enumerate() {
                                    let bx = lav + 4 + col as u32 * (gu + rj);
                                    let dc = ehf + row as u32 * (hn + rj);
                                    
                                    if qh >= bx && qh < bx + gu && abv >= dc && abv < dc + hn {
                                        let fr = aw.id;
                                        if let Some(sq) = self.calculator_states.get_mut(&fr) {
                                            match label {
                                                '0'..='9' => sq.press_digit(label),
                                                '.' => sq.press_dot(),
                                                '+' => sq.press_operator('+'),
                                                '-' => sq.press_operator('-'),
                                                '*' => sq.press_operator('*'),
                                                '/' => sq.press_operator('/'),
                                                '%' => sq.press_operator('%'),
                                                '=' => sq.press_equals(),
                                                'C' => sq.press_clear(),
                                                '(' => sq.press_paren('('),
                                                ')' => sq.press_paren(')'),
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    
                    if self.windows[i].window_type == WindowType::MusicPlayer {
                        let aw = &self.windows[i];
                        let wx = aw.x as u32;
                        let wy = aw.y as u32 + J_();
                        let ca = aw.width;
                        let pad = 10u32;
                        let lp = wx + pad;
                        let rn = ca.saturating_sub(pad * 2);

                        let qh = x as u32;
                        let abv = y as u32;
                        let fr = aw.id;

                        
                        let num_tracks = self.music_player_states.get(&fr)
                            .map(|ic| ic.num_tracks).unwrap_or(0);
                        let dtk = wy + 6;
                        let gc = dtk + 16;
                        let aac = 5usize;
                        let ep = 20u32;
                        let abc = if num_tracks == 0 { ep } else { (num_tracks.min(aac) as u32) * ep };

                        let evi = gc + abc + 10;
                        let dzs = evi + 16;
                        let status_y = dzs + 16;
                        let azc = status_y + 18;
                        let bpy = azc + 12;
                        let bwu = 60u32;
                        let cgd = bpy + bwu + 4;
                        let hs = 14u32;
                        let aqc = cgd + hs + 8;
                        let hn = 28u32;
                        let azt = 36u32;
                        let cog = 64u32;
                        let gap = 4u32;

                        
                        if num_tracks > 0
                            && qh >= lp && qh < lp + rn
                            && abv >= gc && abv < gc + abc
                        {
                            let scroll = self.music_player_states.get(&fr)
                                .map(|ic| ic.track_list_scroll.min(ic.num_tracks.saturating_sub(aac)))
                                .unwrap_or(0);
                            let amq = ((abv - gc) / ep) as usize;
                            let mp = scroll + amq;
                            if mp < num_tracks {
                                crate::serial_println!("[MUSIC] Track list click: track {}", mp);
                                if let Some(ic) = self.music_player_states.get_mut(&fr) {
                                    ic.play_track(mp);
                                }
                                self.fps_low_count = 0; 
                            }
                        }

                        
                        let gzu = azt * 3 + cog + gap * 3;
                        let haa = lp + (rn.saturating_sub(gzu)) / 2;
                        if abv >= aqc && abv < aqc + hn {
                            
                            let amh = haa;
                            if qh >= amh && qh < amh + azt {
                                if let Some(ic) = self.music_player_states.get_mut(&fr) {
                                    ic.prev_track();
                                }
                            }
                            
                            let dct = amh + azt + gap;
                            if qh >= dct && qh < dct + cog {
                                if let Some(ic) = self.music_player_states.get_mut(&fr) {
                                    match ic.state {
                                        PlaybackState::Stopped => {
                                            ic.play_track(ic.current_track);
                                            self.fps_low_count = 0;
                                        },
                                        PlaybackState::Playing | PlaybackState::Paused => ic.toggle_pause(),
                                    }
                                }
                            }
                            
                            let dey = dct + cog + gap;
                            if qh >= dey && qh < dey + azt {
                                if let Some(ic) = self.music_player_states.get_mut(&fr) {
                                    ic.stop();
                                }
                            }
                            
                            let evc = dey + azt + gap;
                            if qh >= evc && qh < evc + azt {
                                if let Some(ic) = self.music_player_states.get_mut(&fr) {
                                    ic.next_track();
                                }
                            }
                        }

                        
                        if qh >= lp && qh < lp + rn
                            && abv >= azc.saturating_sub(3) && abv < azc + 8 {
                            if let Some(ic) = self.music_player_states.get_mut(&fr) {
                                if ic.total_ms > 0 && ic.state != PlaybackState::Stopped {
                                    let ot = (qh - lp) as f32 / rn.max(1) as f32;
                                    let njf = (ot * ic.total_ms as f32) as u64;
                                    ic.seek_to(njf);
                                }
                            }
                        }

                        
                        let apm = aqc + hn + 8;
                        let edw = 10u32;
                        let hbx = lp + 30;
                        let jqn = rn.saturating_sub(72);
                        if qh >= hbx && qh < hbx + jqn
                            && abv >= apm.saturating_sub(4) && abv < apm + edw + 4 {
                            let ot = (qh - hbx) as f32 / jqn.max(1) as f32;
                            let iqk = (ot * 100.0).max(0.0).min(100.0) as u32;
                            if let Some(ic) = self.music_player_states.get_mut(&fr) {
                                ic.volume = iqk;
                                let _ = crate::drivers::hda::set_volume(iqk.min(100) as u8);
                            }
                        }

                        
                        let ent = apm + edw + 10;
                        let ens = ent + 4;
                        let aax = 24u32;
                        let sb = 24u32;
                        let aok = 36u32;
                        let bbt = lp + aok + 4;

                        
                        let bjm = ens + 16;
                        if abv >= bjm && abv < bjm + aax {
                            
                            if qh >= bbt && qh < bbt + sb {
                                if let Some(ic) = self.music_player_states.get_mut(&fr) {
                                    ic.av_offset_ms = (ic.av_offset_ms - 10).max(-500);
                                }
                            }
                            let dfd = bbt + sb + 4 + 52 + 4;
                            
                            if qh >= dfd && qh < dfd + sb {
                                if let Some(ic) = self.music_player_states.get_mut(&fr) {
                                    ic.av_offset_ms = (ic.av_offset_ms + 10).min(500);
                                }
                            }
                            
                            let fby = dfd + sb + 4;
                            if qh >= fby && qh < fby + sb {
                                if let Some(ic) = self.music_player_states.get_mut(&fr) {
                                    ic.av_offset_ms = 0;
                                }
                            }
                        }

                        
                        let bpz = bjm + aax + 4;
                        if abv >= bpz && abv < bpz + aax {
                            let dgj = rn.saturating_sub(aok + 4 + sb * 2 + 12);
                            
                            if qh >= bbt && qh < bbt + sb {
                                let m = self.visualizer.mode;
                                self.visualizer.mode = if m == 0 { crate::visualizer::JJ_ - 1 } else { m - 1 };
                            }
                            
                            let fen = bbt + sb + 4 + dgj + 4;
                            if qh >= fen && qh < fen + sb {
                                self.visualizer.mode = (self.visualizer.mode + 1) % crate::visualizer::JJ_;
                            }
                        }

                        
                        let bod = bpz + aax + 4;
                        if abv >= bod && abv < bod + aax {
                            let dcf = rn.saturating_sub(aok + 4 + sb * 2 + 12);
                            
                            if qh >= bbt && qh < bbt + sb {
                                let aa = self.visualizer.palette;
                                self.visualizer.palette = if aa == 0 { crate::visualizer::AHY_ - 1 } else { aa - 1 };
                            }
                            
                            let ewb = bbt + sb + 4 + dcf + 4;
                            if qh >= ewb && qh < ewb + sb {
                                self.visualizer.palette = (self.visualizer.palette + 1) % crate::visualizer::AHY_;
                            }
                        }

                        
                        let bok = bod + aax + 4;
                        if abv >= bok && abv < bok + aax {
                            let dxg = rn.saturating_sub(aok + 4 + sb * 2 + 12);
                            
                            if qh >= bbt && qh < bbt + sb {
                                let aa = self.matrix_rain_preset;
                                self.set_rain_preset(if aa == 0 { 2 } else { aa - 1 });
                            }
                            
                            let exp = bbt + sb + 4 + dxg + 4;
                            if qh >= exp && qh < exp + sb {
                                self.set_rain_preset((self.matrix_rain_preset + 1) % 3);
                            }
                        }
                    }
                    
                    self.focus_window(id);
                    return;
                }
            }
            
            
            if let Some(idx) = self.check_icon_index(x, y) {
                let action = self.icons[idx].action;
                self.handle_icon_action(action);
                return;
            }
            
            self.start_menu_open = false;
            self.start_menu_search.clear();
        } else {
            
            let gvm = self.snap_preview.take();
            let mut oug: Option<u32> = None;
            for w in &mut self.windows {
                if w.dragging {
                    if let Some(it) = gvm {
                        
                        let ang = self.height.saturating_sub(V_());
                        let bah = BW_() as i32;
                        let hcm = self.width.saturating_sub(BW_());
                        let nk = hcm / 2;
                        let kh = ang / 2;
                        match it {
                            SnapDir::Left => { w.x = bah; w.y = 0; w.width = nk; w.height = ang; }
                            SnapDir::Right => { w.x = bah + nk as i32; w.y = 0; w.width = nk; w.height = ang; }
                            SnapDir::TopLeft => { w.x = bah; w.y = 0; w.width = nk; w.height = kh; }
                            SnapDir::TopRight => { w.x = bah + nk as i32; w.y = 0; w.width = nk; w.height = kh; }
                            SnapDir::BottomLeft => { w.x = bah; w.y = kh as i32; w.width = nk; w.height = kh; }
                            SnapDir::BottomRight => { w.x = bah + nk as i32; w.y = kh as i32; w.width = nk; w.height = kh; }
                        }
                        w.maximized = false;
                        oug = Some(w.id);
                    }
                }
                w.dragging = false;
                w.resizing = ResizeEdge::None;
            }
            
            
            if self.drag_state.is_some() {
                self.finish_drag(x, y);
            }
            
            
            let nfy: Vec<u32> = self.windows.iter()
                .filter(|w| w.window_type == WindowType::ModelEditor && w.focused)
                .map(|w| w.id)
                .collect();
            for id in nfy {
                if let Some(state) = self.model_editor_states.get_mut(&id) {
                    state.handle_click(0, 0, 0, 0, false);
                }
            }
            
            
            let flm: Vec<u32> = self.windows.iter()
                .filter(|w| w.window_type == WindowType::Chess && w.focused)
                .map(|w| w.id)
                .collect();
            for id in flm {
                if let Some(chess) = self.chess_states.get_mut(&id) {
                    if chess.drag_from.is_some() {
                        
                        if let Some(aw) = self.windows.iter().find(|w| w.id == id) {
                            let yu = aw.x as i32 + 8;
                            let xp = aw.y as i32 + J_() as i32 + 4;
                            let ajs = aw.width.saturating_sub(16) as i32;
                            let cell_size: i32 = 48;
                            let tg = cell_size * 8;
                            let un = yu + (ajs - tg) / 2;
                            let ve = xp + 28;
                            
                            let col = (x - un) / cell_size;
                            let row = (y - ve) / cell_size;
                            chess.handle_mouse_release(col, row);
                        }
                    }
                }
            }
            
            
            let kka: Vec<u32> = self.windows.iter()
                .filter(|w| w.window_type == WindowType::Chess3D && w.focused)
                .map(|w| w.id)
                .collect();
            for id in kka {
                if let Some(state) = self.chess3d_states.get_mut(&id) {
                    state.handle_mouse_release();
                }
            }
            
            
            #[cfg(feature = "emulators")]
            {
            let mqn: Vec<(u32, Option<u32>)> = self.windows.iter()
                .filter(|w| w.window_type == WindowType::GameBoyInput && w.focused)
                .map(|w| (w.id, self.gb_input_links.get(&w.id).copied()))
                .collect();
            for (_iid, cmh) in mqn {
                let bsf = cmh.or_else(|| self.gameboy_states.keys().next().copied());
                if let Some(bbh) = bsf {
                    if let Some(an) = self.gameboy_states.get_mut(&bbh) {
                        an.handle_key_release(b'w');
                        an.handle_key_release(b'a');
                        an.handle_key_release(b's');
                        an.handle_key_release(b'd');
                        an.handle_key_release(b'x');
                        an.handle_key_release(b'z');
                        an.handle_key_release(b'c');
                        an.handle_key_release(b'\r');
                    }
                }
            }
            }
        }
    }
    
    
    pub fn handle_right_click(&mut self, x: i32, y: i32, pressed: bool) {
        if !pressed {
            return; 
        }
        
        
        self.context_menu.visible = false;
        self.start_menu_open = false;
        self.start_menu_search.clear();
        
        
        if let Some(fm_info) = self.windows.iter().find(|w| {
            w.window_type == WindowType::FileManager
            && x >= w.x && x < w.x + w.width as i32
            && y >= w.y + J_() as i32 + 36 + 1 + 24 
            && y < w.y + w.height as i32
        }).map(|w| (w.id, w.x, w.y, w.width, w.height, w.file_path.clone(), w.selected_index, w.content.len())) {
            let (sa, wx, wy, ca, _wh, file_path_opt, sel_idx, anw) = fm_info;
            let rq = self.fm_states.get(&sa).map(|f| if f.sidebar_collapsed { 0i32 } else { f.sidebar_width as i32 }).unwrap_or(180);
            
            
            if x >= wx + rq {
                
                let bn = wy + J_() as i32;
                let wu = bn + 36 + 1;
                let gfr = wu + 24 + 1;
                let ep = 26i32;
                let xb = 5usize.min(anw);
                let adp = if anw > xb + 2 { anw - xb - 2 } else { 0 };
                let qn = y - gfr;
                let scroll = self.windows.iter().find(|w| w.id == sa).map(|w| w.scroll_offset).unwrap_or(0);
                
                let bau = if qn >= 0 { Some(scroll + (qn / ep) as usize) } else { None };
                let isf = bau.map(|i| i < adp).unwrap_or(false);
                
                
                if let Some(idx) = bau {
                    if idx < adp {
                        if let Some(w) = self.windows.iter_mut().find(|w| w.id == sa) {
                            w.selected_index = idx;
                        }
                    }
                }
                
                
                let target_file = if isf {
                    if let Some(w) = self.windows.iter().find(|w| w.id == sa) {
                        let abp = xb + bau.unwrap_or(0);
                        if abp < w.content.len().saturating_sub(2) {
                            let line = &w.content[abp];
                            let name = Self::bbn(line);
                            if name != ".." { Some(String::from(name)) } else { None }
                        } else { None }
                    } else { None }
                } else { None };
                
                if isf && target_file.is_some() {
                    
                    self.context_menu = Jr {
                        visible: true,
                        x, y,
                        items: alloc::vec![
                            Aa { label: String::from("  Open          Enter"), action: ContextAction::Open },
                            Aa { label: String::from("  Open With..."), action: ContextAction::OpenWith },
                            Aa { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                            Aa { label: String::from("  Cut          Ctrl+X"), action: ContextAction::Cut },
                            Aa { label: String::from("  Copy         Ctrl+C"), action: ContextAction::Copy },
                            Aa { label: String::from("  Copy Path"), action: ContextAction::CopyPath },
                            Aa { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                            Aa { label: String::from("  Rename            F2"), action: ContextAction::Rename },
                            Aa { label: String::from("  Delete           Del"), action: ContextAction::Delete },
                            Aa { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                            Aa { label: String::from("  Properties"), action: ContextAction::Properties },
                        ],
                        selected_index: 0,
                        target_icon: None,
                        target_file,
                    };
                } else {
                    
                    self.context_menu = Jr {
                        visible: true,
                        x, y,
                        items: alloc::vec![
                            Aa { label: String::from("  New File         N"), action: ContextAction::NewFile },
                            Aa { label: String::from("  New Folder       D"), action: ContextAction::NewFolder },
                            Aa { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                            Aa { label: String::from("  Paste        Ctrl+V"), action: ContextAction::Paste },
                            Aa { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                            Aa { label: String::from("  Sort by Name"), action: ContextAction::SortByName },
                            Aa { label: String::from("  Sort by Size"), action: ContextAction::SortBySize },
                            Aa { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                            Aa { label: String::from("  Refresh          F5"), action: ContextAction::Refresh },
                            Aa { label: String::from("  Open in Terminal"), action: ContextAction::TerminalHere },
                            Aa { label: String::from("  Properties"), action: ContextAction::Properties },
                        ],
                        selected_index: 0,
                        target_icon: None,
                        target_file: file_path_opt,
                    };
                }
                return;
            }
        }
        
        
        if let Some(idx) = self.check_icon_index(x, y) {
            self.show_icon_context_menu(x, y, idx);
            return;
        }
        
        
        if y < (self.height - V_()) as i32 {
            self.show_desktop_context_menu(x, y);
        }
    }
    
    
    fn show_icon_context_menu(&mut self, x: i32, y: i32, icon_index: usize) {
        self.context_menu = Jr {
            visible: true,
            x,
            y,
            items: alloc::vec![
                Aa { label: String::from("  Open          Enter"), action: ContextAction::Open },
                Aa { label: String::from("  Open With..."), action: ContextAction::OpenWith },
                Aa { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                Aa { label: String::from("  Cut          Ctrl+X"), action: ContextAction::Cut },
                Aa { label: String::from("  Copy         Ctrl+C"), action: ContextAction::Copy },
                Aa { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                Aa { label: String::from("  Rename            F2"), action: ContextAction::Rename },
                Aa { label: String::from("  Delete           Del"), action: ContextAction::Delete },
                Aa { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                Aa { label: String::from("  Properties"), action: ContextAction::Properties },
            ],
            selected_index: 0,
            target_icon: Some(icon_index),
            target_file: None,
        };
    }
    
    
    fn show_desktop_context_menu(&mut self, x: i32, y: i32) {
        self.context_menu = Jr {
            visible: true,
            x,
            y,
            items: alloc::vec![
                Aa { label: String::from("  View              >"), action: ContextAction::ViewLargeIcons },
                Aa { label: String::from("  Sort by           >"), action: ContextAction::SortByName },
                Aa { label: String::from("  Refresh          F5"), action: ContextAction::Refresh },
                Aa { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                Aa { label: String::from("  Paste        Ctrl+V"), action: ContextAction::Paste },
                Aa { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                Aa { label: String::from("  New               >"), action: ContextAction::NewFile },
                Aa { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                Aa { label: String::from("  Open in Terminal"), action: ContextAction::TerminalHere },
                Aa { label: String::from("  Personalize"), action: ContextAction::Personalize },
                Aa { label: String::from("  Properties"), action: ContextAction::Properties },
            ],
            selected_index: 0,
            target_icon: None,
            target_file: None,
        };
    }
    
    
    fn check_context_menu_click(&self, x: i32, y: i32) -> Option<ContextAction> {
        if !self.context_menu.visible {
            return None;
        }
        
        let hu = self.context_menu.x;
        let ks = self.context_menu.y;
        let bhx = 150;
        let axs = 22;
        let dbf = self.context_menu.items.len() as i32 * axs;
        
        if x >= hu && x < hu + bhx && y >= ks && y < ks + dbf {
            let idx = ((y - ks) / axs) as usize;
            if idx < self.context_menu.items.len() {
                return Some(self.context_menu.items[idx].action);
            }
        }
        
        None
    }
    
    
    fn execute_context_action(&mut self, action: ContextAction) {
        let offset = (self.windows.len() as i32 * 25) % 200;
        
        
        let hzk = self.context_menu.target_file.clone();
        let cjs = self.context_menu.target_icon;
        
        
        let bcl = hzk.is_some() && cjs.is_none();
        
        match action {
            ContextAction::Open => {
                if bcl {
                    
                    if let Some(window) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager) {
                        let xb = 5usize.min(window.content.len());
                        let abp = xb + window.selected_index;
                        if abp < window.content.len().saturating_sub(2) {
                            let line = &window.content[abp];
                            let is_dir = line.contains("[D]");
                            let name = String::from(Self::bbn(line));
                            if is_dir {
                                self.navigate_file_manager(&name);
                            } else {
                                self.open_file(&name);
                            }
                        }
                    }
                } else if let Some(idx) = cjs {
                    let mnd = self.icons[idx].action;
                    self.handle_icon_action(mnd);
                }
            },
            ContextAction::OpenWith => {
                self.create_window("Open With", 300 + offset, 200 + offset, 400, 300, WindowType::FileAssociations);
            },
            ContextAction::Refresh => {
                if bcl {
                    if let Some(path) = hzk {
                        self.refresh_file_manager(&path);
                    }
                }
                crate::serial_println!("[GUI] Refreshed");
            },
            ContextAction::NewFile => {
                if bcl {
                    let ht = self.windows.iter()
                        .find(|w| w.focused && w.window_type == WindowType::FileManager)
                        .and_then(|w| w.file_path.clone())
                        .unwrap_or_else(|| String::from("/"));
                    let name = format!("new_file_{}.txt", self.frame_count % 1000);
                    let kg = if ht == "/" { format!("/{}", name) } else { format!("{}/{}", ht, name) };
                    let _ = crate::ramfs::bh(|fs| fs.touch(&kg));
                    crate::serial_println!("[FM] Created file: {}", kg);
                    self.refresh_file_manager(&ht);
                } else {
                    let filename = format!("/desktop/newfile_{}.txt", self.frame_count);
                    crate::ramfs::bh(|fs| { let _ = fs.write_file(&filename, b"New file created from desktop"); });
                }
            },
            ContextAction::NewFolder => {
                if bcl {
                    let ht = self.windows.iter()
                        .find(|w| w.focused && w.window_type == WindowType::FileManager)
                        .and_then(|w| w.file_path.clone())
                        .unwrap_or_else(|| String::from("/"));
                    let name = format!("folder_{}", self.frame_count % 1000);
                    let kg = if ht == "/" { format!("/{}", name) } else { format!("{}/{}", ht, name) };
                    let _ = crate::ramfs::bh(|fs| fs.mkdir(&kg));
                    crate::serial_println!("[FM] Created folder: {}", kg);
                    self.refresh_file_manager(&ht);
                } else {
                    let cil = format!("/desktop/folder_{}", self.frame_count);
                    crate::ramfs::bh(|fs| { let _ = fs.mkdir(&cil); });
                }
            },
            ContextAction::Properties => {
                let (w, h) = (self.width, self.height);
                let pup = self.windows.len();
                let mnh = self.icons.len();
                let fr = self.create_window("Properties", 350 + offset, 250 + offset, 320, 220, WindowType::About);
                if let Some(window) = self.windows.iter_mut().find(|wnd| wnd.id == fr) {
                    window.content.clear();
                    window.content.push(String::from("═══════ System Properties ═══════"));
                    window.content.push(String::new());
                    window.content.push(format!("Display: {}x{}", w, h));
                    window.content.push(format!("Windows open: {}", pup + 1));
                    window.content.push(format!("Desktop icons: {}", mnh));
                    window.content.push(String::new());
                    window.content.push(String::from("Theme: GitHub Dark"));
                    window.content.push(String::from("OS: TrustOS v0.9.4"));
                }
            },
            ContextAction::Cut => {
                if bcl {
                    self.file_clipboard_copy(true);
                } else if let Some(idx) = cjs {
                    self.clipboard_icon = Some((idx, true));
                    let name = self.icons[idx].name.clone();
                    crate::keyboard::byb(&name);
                }
            },
            ContextAction::Copy => {
                if bcl {
                    self.file_clipboard_copy(false);
                } else if let Some(idx) = cjs {
                    self.clipboard_icon = Some((idx, false));
                    let name = self.icons[idx].name.clone();
                    crate::keyboard::byb(&name);
                }
            },
            ContextAction::Paste => {
                if bcl {
                    self.file_clipboard_paste();
                } else if let Some((amu, is_cut)) = self.clipboard_icon.take() {
                    if amu < self.icons.len() {
                        if !is_cut {
                            let src = self.icons[amu].clone();
                            let dbq = format!("{} (copy)", src.name);
                            let nja = Rr {
                                name: dbq.clone(),
                                icon_type: src.icon_type,
                                x: src.x + 10,
                                y: src.y + 10,
                                action: src.action,
                            };
                            self.icons.push(nja);
                        }
                    }
                }
            },
            ContextAction::CopyPath => {
                if bcl {
                    if let Some(window) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager) {
                        let ht = window.file_path.clone().unwrap_or_else(|| String::from("/"));
                        let xb = 5usize.min(window.content.len());
                        let abp = xb + window.selected_index;
                        if abp < window.content.len().saturating_sub(2) {
                            let name = Self::bbn(&window.content[abp]);
                            let xo = if ht == "/" { format!("/{}", name) } else { format!("{}/{}", ht, name) };
                            crate::keyboard::byb(&xo);
                            crate::serial_println!("[FM] Copied path: {}", xo);
                        }
                    }
                } else if let Some(idx) = cjs {
                    if idx < self.icons.len() {
                        let path = format!("/desktop/{}", self.icons[idx].name);
                        crate::keyboard::byb(&path);
                    }
                }
            },
            ContextAction::Delete => {
                if bcl {
                    if let Some(window) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager) {
                        let ht = window.file_path.clone().unwrap_or_else(|| String::from("/"));
                        let xb = 5usize.min(window.content.len());
                        let abp = xb + window.selected_index;
                        if abp < window.content.len().saturating_sub(2) {
                            let name = String::from(Self::bbn(&window.content[abp]));
                            if name != ".." {
                                let kg = if ht == "/" { format!("/{}", name) } else { format!("{}/{}", ht, name) };
                                let _ = crate::ramfs::bh(|fs| fs.rm(&kg));
                                crate::serial_println!("[FM] Deleted: {}", kg);
                            }
                        }
                        let cp = ht.clone();
                        drop(window);
                        self.refresh_file_manager(&cp);
                    }
                } else if let Some(idx) = cjs {
                    if idx < self.icons.len() {
                        self.icons.remove(idx);
                        self.clipboard_icon = None;
                    }
                }
            },
            ContextAction::Rename => {
                if bcl {
                    
                    if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileManager) {
                        let xb = 5usize.min(window.content.len());
                        let abp = xb + window.selected_index;
                        if abp < window.content.len().saturating_sub(2) {
                            let name = String::from(Self::bbn(&window.content[abp]));
                            if name != ".." {
                                self.input_buffer = name.clone();
                                window.title = format!("RENAME:{}", name);
                            }
                        }
                    }
                } else if let Some(idx) = cjs {
                    if idx < self.icons.len() {
                        crate::serial_println!("[GUI] Rename icon: {}", self.icons[idx].name);
                    }
                }
            },
            ContextAction::SortByName => {
                if bcl {
                    if let Some(sa) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager).map(|w| w.id) {
                        if let Some(kn) = self.fm_states.get_mut(&sa) {
                            if kn.sort_column == 0 { kn.sort_ascending = !kn.sort_ascending; } else { kn.sort_column = 0; kn.sort_ascending = true; }
                        }
                        if let Some(path) = self.windows.iter().find(|w| w.id == sa).and_then(|w| w.file_path.clone()) {
                            self.refresh_file_manager(&path);
                        }
                    }
                }
            },
            ContextAction::SortBySize => {
                if bcl {
                    if let Some(sa) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager).map(|w| w.id) {
                        if let Some(kn) = self.fm_states.get_mut(&sa) {
                            if kn.sort_column == 2 { kn.sort_ascending = !kn.sort_ascending; } else { kn.sort_column = 2; kn.sort_ascending = true; }
                        }
                        if let Some(path) = self.windows.iter().find(|w| w.id == sa).and_then(|w| w.file_path.clone()) {
                            self.refresh_file_manager(&path);
                        }
                    }
                }
            },
            ContextAction::SortByDate => {
                crate::serial_println!("[GUI] Sort by date (not yet supported)");
            },
            ContextAction::ViewLargeIcons | ContextAction::ViewSmallIcons | ContextAction::ViewList => {
                crate::serial_println!("[GUI] View mode changed");
            },
            ContextAction::Personalize => {
                self.create_window("Personalization", 250 + offset, 150 + offset, 400, 300, WindowType::Settings);
            },
            ContextAction::TerminalHere => {
                self.create_window("Terminal", 200 + offset, 120 + offset, 500, 350, WindowType::Terminal);
            },
            ContextAction::Cancel => {},
        }
    }
    
    
    fn check_icon_index(&self, x: i32, y: i32) -> Option<usize> {
        
        if x < 0 || x >= (BW_() + 10) as i32 {
            return None;
        }
        let atn = self.height.saturating_sub(V_());
        let cbu = self.icons.len().max(1) as u32;
        let padding = 12u32;
        let available = atn.saturating_sub(padding * 2);
        let axn = available / cbu;
        let start_y = padding + (available - axn * cbu) / 2;
        
        for (idx, _icon) in self.icons.iter().enumerate() {
            let gg = (start_y + idx as u32 * axn) as i32;
            if y >= gg && y < gg + axn as i32 {
                return Some(idx);
            }
        }
        None
    }
    
    
    fn handle_icon_action(&mut self, action: IconAction) {
        let offset = (self.windows.len() as i32 * 25) % 200;
        let id = match action {
            IconAction::OpenTerminal => {
                self.create_window("Terminal", 120 + offset, 60 + offset, 640, 440, WindowType::Terminal)
            },
            IconAction::OpenFileManager => {
                self.create_window("Files", 140 + offset, 80 + offset, 520, 420, WindowType::FileManager)
            },
            IconAction::OpenCalculator => {
                self.create_window("Calculator", 350 + offset, 100 + offset, 300, 380, WindowType::Calculator)
            },
            IconAction::OpenNetwork => {
                self.create_window("NetScan", 140 + offset, 80 + offset, 640, 440, WindowType::Cn)
            },
            IconAction::OpenSettings => {
                self.create_window("Settings", 250 + offset, 120 + offset, 440, 340, WindowType::Settings)
            },
            IconAction::OpenAbout => {
                self.create_window("About TrustOS", 300 + offset, 140 + offset, 420, 280, WindowType::About)
            },
            IconAction::OpenMusicPlayer => {
                let dbj = self.width.saturating_sub(340) as i32;
                let dbk = self.height.saturating_sub(V_() + 600) as i32;
                self.create_window("Music Player", dbj, dbk.max(20), 320, 580, WindowType::MusicPlayer)
            },
            IconAction::OpenGame => {
                let dy = self.width;
                let dw = self.height;
                let id = self.create_window("TrustChess 3D", 0, 0, dy, dw - V_(), WindowType::Chess3D);
                if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                    w.maximized = true;
                }
                id
            },
            IconAction::OpenEditor => {
                self.create_window("TrustCode", 120 + offset, 50 + offset, 780, 560, WindowType::TextEditor)
            },
            IconAction::OpenGL3D => {
                self.create_window("TrustGL 3D Demo", 120 + offset, 50 + offset, 500, 420, WindowType::Demo3D)
            },
            IconAction::OpenBrowser => {
                self.create_window("TrustBrowser", 100 + offset, 40 + offset, 720, 520, WindowType::Browser)
            },
            IconAction::OpenModelEditor => {
                self.create_window("TrustEdit 3D", 80 + offset, 40 + offset, 780, 560, WindowType::ModelEditor)
            },
            IconAction::OpenGame3D => {
                self.create_window("TrustDoom 3D", 60 + offset, 30 + offset, 720, 540, WindowType::Game3D)
            },
            #[cfg(feature = "emulators")]
            IconAction::OpenNes => {
                self.create_window("NES Emulator", 80 + offset, 50 + offset, 512, 480, WindowType::NesEmu)
            },
            #[cfg(feature = "emulators")]
            IconAction::OpenGameBoy => {
                self.create_window("Game Boy", 100 + offset, 60 + offset, 480, 432, WindowType::GameBoyEmu)
            },
            #[cfg(feature = "emulators")]
            IconAction::OpenGameLab => {
                let dy = self.width;
                let dw = self.height;
                
                let dtd = 490i32;
                let gex = (dy as i32 - dtd).max(400) as u32;
                let gew = dw - V_();
                let cbh = self.create_window("Game Lab", dtd, 0, gex, gew, WindowType::GameLab);
                cbh
            },
        };
        
        self.focus_window(id);
    }
    
    fn handle_taskbar_click(&mut self, x: i32, _y: i32) {
        
        if x >= (self.width - 8) as i32 {
            self.toggle_show_desktop();
            crate::serial_println!("[GUI] Show Desktop corner clicked");
            return;
        }
        
        
        if x >= 4 && x < 120 {
            self.start_menu_open = !self.start_menu_open;
            if !self.start_menu_open {
                self.start_menu_search.clear();
            }
            return;
        }
        
        
        let dft = self.width - 120;
        let jfr = dft - 44;
        if x >= jfr as i32 && x < (jfr + 40) as i32 {
            let akg = match self.desktop_tier {
                DesktopTier::Full => DesktopTier::Standard,
                DesktopTier::Standard => DesktopTier::Minimal,
                DesktopTier::Minimal | DesktopTier::CliOnly => DesktopTier::Full,
            };
            self.desktop_tier = akg;
            self.tier_manual_override = true;
            self.fps_low_count = 0;
            self.fps_high_count = 0;
            self.needs_full_redraw = true;
            self.background_cached = false;
            crate::serial_println!("[Desktop] Tier toggle from gear icon: {:?}", akg);
            return;
        }

        
        let fdp = dft;
        if x >= fdp as i32 && x < (fdp + 20) as i32 {
            
            for w in &self.windows {
                if w.window_type == WindowType::WifiNetworks {
                    let id = w.id;
                    self.focus_window(id);
                    return;
                }
            }
            self.create_window("WiFi Networks", 200, 100, 420, 500, WindowType::WifiNetworks);
            return;
        }
        
        
        let ecc = self.windows.len();
        if ecc > 0 {
            let gu = 96u32;
            let rj = 6u32;
            let aaj = ecc as u32 * (gu + rj) - rj;
            let start_x = (self.width.saturating_sub(aaj)) / 2;
            
            for (i, w) in self.windows.iter().enumerate() {
                let zs = start_x + i as u32 * (gu + rj);
                if x >= zs as i32 && x < (zs + gu) as i32 {
                    let id = w.id;
                    
                    if w.focused && !w.minimized {
                        self.minimize_window(id);
                    } else {
                        self.focus_window(id);
                    }
                    return;
                }
            }
        }
    }
    
    
    fn open_settings_panel(&mut self) {
        
        for w in &self.windows {
            if w.window_type == WindowType::Settings {
                let id = w.id;
                self.focus_window(id);
                return;
            }
        }
        
        self.create_window("Settings", 180, 80, 620, 440, WindowType::Settings);
    }
    
    
    fn check_start_menu_click(&self, x: i32, y: i32) -> Option<u8> {
        
        let pz = 480u32;
        let rv = 680u32;
        let hu = 4i32;
        let ks = (self.height - V_() - rv - 8) as i32;
        
        
        if x < hu || x >= hu + pz as i32 || y < ks || y >= ks + rv as i32 {
            return None;
        }
        
        
        let dsy = ks + 78;
        
        
        let jww: [&str; 16] = [
            "Terminal", "Files", "Calculator", "Network", "Text Editor",
            "TrustEdit 3D", "Browser", "Snake", "Chess", "Chess 3D",
            "NES Emulator", "Game Boy", "TrustLab", "Music Player", "TrustWave", "Settings",
        ];
        let hfl: [u8; 16] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];
        
        
        let nwo: [&str; 3] = ["Exit Desktop", "Shutdown", "Reboot"];
        let nwl: [u8; 3] = [16, 17, 18];
        
        let search = self.start_menu_search.trim();
        let apb: String = search.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
        
        
        let ble = 2u32;
        let cel = (pz - 24) / ble;
        let bpk = 44u32;
        let cek = 4u32;
        
        let lvl: alloc::vec::Vec<u8> = if search.is_empty() {
            hfl.to_vec()
        } else {
            hfl.iter().filter(|&&idx| {
                let label: String = jww[idx as usize].chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                label.contains(apb.as_str())
            }).copied().collect()
        };
        
        
        if y >= dsy && y < ks + rv as i32 - 110 {
            for (cio, &awc) in lvl.iter().enumerate() {
                let col = (cio % ble as usize) as i32;
                let row = (cio / ble as usize) as i32;
                let bnd = hu + 10 + col * (cel + cek) as i32;
                let ru = dsy + row * (bpk + cek) as i32;
                
                if x >= bnd && x < bnd + cel as i32
                    && y >= ru && y < ru + bpk as i32 {
                    return Some(awc);
                }
            }
        }
        
        
        let ewy = ks + rv as i32 - 106;
        let ivs = ewy + 8;
        if y >= ivs {
            for (pi, &pidx) in nwl.iter().enumerate() {
                if !apb.is_empty() {
                    let das: String = nwo[pi].chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                    if !das.contains(apb.as_str()) { continue; }
                }
                let iiq = ivs + (pi as i32 * 30);
                if y >= iiq && y < iiq + 28 {
                    return Some(pidx);
                }
            }
        }
        
        None
    }
    
    fn handle_menu_action(&mut self, action: u8) {
        
        
        
        match action {
            0 => { 
                let x = 100 + (self.windows.len() as i32 * 30);
                let y = 60 + (self.windows.len() as i32 * 20);
                self.create_window("Terminal", x, y, 640, 440, WindowType::Terminal);
            },
            1 => { 
                self.create_window("File Explorer", 100, 60, 780, 520, WindowType::FileManager);
            },
            2 => { 
                self.create_window("Calculator", 350, 100, 300, 380, WindowType::Calculator);
            },
            3 => { 
                self.create_window("NetScan", 140, 80, 640, 440, WindowType::Cn);
            },
            4 => { 
                self.create_window("TrustCode", 120, 50, 780, 560, WindowType::TextEditor);
            },
            5 => { 
                self.create_window("TrustEdit 3D", 80, 40, 780, 560, WindowType::ModelEditor);
            },
            6 => { 
                self.create_window("TrustBrowser", 100, 40, 720, 520, WindowType::Browser);
            },
            7 => { 
                let dy = self.width;
                let dw = self.height;
                let id = self.create_window("TrustChess 3D", 0, 0, dy, dw - V_(), WindowType::Chess3D);
                
                if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                    w.maximized = true;
                }
            },
            8 => { 
                self.create_window("TrustChess", 180, 60, 520, 560, WindowType::Chess);
            },
            9 => { 
                self.create_window("Snake Game", 220, 80, 380, 400, WindowType::Game);
            },
            10 => { 
                #[cfg(feature = "emulators")]
                self.create_window("NES Emulator", 80, 40, 560, 520, WindowType::NesEmu);
            },
            11 => { 
                #[cfg(feature = "emulators")]
                self.create_window("Game Boy", 100, 40, 520, 480, WindowType::GameBoyEmu);
            },
            12 => { 
                self.open_lab_mode();
            },
            13 => { 
                crate::serial_println!("[GUI] Opening Music Player...");
                let dbj = self.width.saturating_sub(320) as i32;
                let dbk = self.height.saturating_sub(V_() + 600) as i32;
                crate::serial_println!("[GUI] Music Player pos: {}x{}", dbj, dbk.max(20));
                self.create_window("Music Player", dbj, dbk.max(20), 320, 580, WindowType::MusicPlayer);
                crate::serial_println!("[GUI] Music Player window created OK");
            },
            14 => { 
                self.open_wifi_analyzer();
            },
            15 => { 
                self.open_settings_panel();
            },
            16 => { 
                crate::serial_println!("[GUI] Exit Desktop from start menu");
                NW_.store(true, Ordering::SeqCst);
            },
            17 => { 
                crate::serial_println!("[SYSTEM] Shutdown sequence initiated");
                self.shutdown_active = true;
                self.shutdown_start_tick = crate::logger::eg();
                self.shutdown_phase = 0;
                
                self.start_menu_open = false;
                self.start_menu_search.clear();
            },
            18 => { 
                crate::serial_println!("[SYSTEM] Reboot requested");
                
                unsafe {
                    let mut port = crate::arch::Port::<u8>::new(0x64);
                    port.write(0xFE);
                }
                loop { crate::arch::acb(); }
            },
            _ => {}
        }
    }
    
    
    pub fn handle_keyboard_input(&mut self, key: u8) {
        use crate::keyboard::{T_, S_};
        
        self.windows_dirty = true;
        crate::serial_println!("[KBD-DBG] handle_keyboard_input key={} (0x{:02X}) lock={} start_menu={}",
            key, key, self.lock_screen_active, self.start_menu_open);
        
        
        if self.lock_screen_active {
            self.handle_lock_screen_key(key);
            return;
        }
        
        
        if self.start_menu_open {
            match key {
                0x1B => { 
                    self.start_menu_open = false;
                    self.start_menu_search.clear();
                    self.start_menu_selected = -1;
                },
                0x08 | 0x7F => { 
                    self.start_menu_search.pop();
                    self.start_menu_selected = -1; 
                },
                k if k == T_ => { 
                    if self.start_menu_selected > 0 {
                        self.start_menu_selected -= 1;
                    } else {
                        
                        self.start_menu_selected = 16;
                    }
                },
                k if k == S_ => { 
                    if self.start_menu_selected < 16 {
                        self.start_menu_selected += 1;
                    } else {
                        self.start_menu_selected = 0;
                    }
                },
                0x0D | 0x0A => { 
                    if self.start_menu_selected >= 0 && self.start_menu_selected <= 18 {
                        
                        let action = self.start_menu_selected as u8;
                        self.start_menu_open = false;
                        self.start_menu_search.clear();
                        self.start_menu_selected = -1;
                        self.handle_menu_action(action);
                        return;
                    }
                    
                    let jum: [&str; 19] = [
                        "Terminal", "Files", "Calculator", "Network", "Text Editor",
                        "TrustEdit 3D", "Browser", "Snake", "Chess", "Chess 3D",
                        "NES Emulator", "Game Boy", "TrustLab", "Music Player",
                        "TrustWave", "Settings", "Exit Desktop", "Shutdown", "Reboot",
                    ];
                    let search = self.start_menu_search.trim();
                    if !search.is_empty() {
                        let apb: String = search.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                        for (i, label) in jum.iter().enumerate() {
                            let dtf: String = label.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                            if dtf.contains(apb.as_str()) {
                                self.start_menu_open = false;
                                self.start_menu_search.clear();
                                self.start_menu_selected = -1;
                                self.handle_menu_action(i as u8);
                                return;
                            }
                        }
                    }
                },
                b' '..=b'~' => { 
                    if self.start_menu_search.len() < 32 {
                        self.start_menu_search.push(key as char);
                        self.start_menu_selected = -1; 
                    }
                },
                _ => {}
            }
            return;
        }

        
        let enb = self.windows.iter().find(|w| w.focused).map(|w| (w.window_type, w.id));
        crate::serial_println!("[KBD-DBG] focused_info={:?} n_windows={}",
            enb.map(|(_, id)| id), self.windows.len());
        
        if let Some((wt, fr)) = enb {
            match wt {
                WindowType::Terminal => {
                    self.handle_terminal_key(key);
                },
                WindowType::FileManager => {
                    
                    let ctrl = crate::keyboard::sx(0x1D);
                    if ctrl && (key == 3 || key == b'c' || key == b'C') {
                        self.file_clipboard_copy(false);
                        return;
                    }
                    if ctrl && (key == 24 || key == b'x' || key == b'X') {
                        self.file_clipboard_copy(true);
                        return;
                    }
                    if ctrl && (key == 22 || key == b'v' || key == b'V') {
                        self.file_clipboard_paste();
                        return;
                    }
                    
                    if key == b'v' || key == b'V' {
                        let current = self.fm_view_modes.get(&fr).copied().unwrap_or(FileManagerViewMode::List);
                        let iqc = match current {
                            FileManagerViewMode::List => FileManagerViewMode::IconGrid,
                            FileManagerViewMode::IconGrid => FileManagerViewMode::Details,
                            FileManagerViewMode::Details => FileManagerViewMode::Tiles,
                            FileManagerViewMode::Tiles => FileManagerViewMode::List,
                        };
                        self.fm_view_modes.insert(fr, iqc);
                        crate::serial_println!("[FM] View mode: {:?}-like for window {}", 
                            match iqc { FileManagerViewMode::List => "List", FileManagerViewMode::IconGrid => "Grid", FileManagerViewMode::Details => "Details", FileManagerViewMode::Tiles => "Tiles" },
                            fr);
                        return;
                    }
                    self.handle_filemanager_key(key);
                },
                WindowType::ImageViewer => {
                    self.handle_image_viewer_key(key);
                },
                WindowType::FileAssociations => {
                    self.handle_fileassoc_key(key);
                },
                WindowType::Settings => {
                    self.handle_settings_key(key);
                },
                WindowType::Cn => {
                    self.handle_netscan_key(key);
                },
                WindowType::TextEditor => {
                    
                    if let Some(editor) = self.editor_states.get_mut(&fr) {
                        editor.handle_key(key);
                    }
                },
                WindowType::ModelEditor => {
                    
                    if let Some(state) = self.model_editor_states.get_mut(&fr) {
                        state.handle_key(key);
                    }
                },
                WindowType::Calculator => {
                    if let Some(sq) = self.calculator_states.get_mut(&fr) {
                        match key {
                            b'0'..=b'9' => sq.press_digit(key as char),
                            b'.' => sq.press_dot(),
                            b'+' => sq.press_operator('+'),
                            b'-' => sq.press_operator('-'),
                            b'*' => sq.press_operator('*'),
                            b'/' => sq.press_operator('/'),
                            b'%' => sq.press_operator('%'),
                            b'(' => sq.press_paren('('),
                            b')' => sq.press_paren(')'),
                            b'=' | 0x0D | 0x0A => sq.press_equals(), 
                            b'c' | b'C' => sq.press_clear(),
                            0x08 => sq.press_backspace(), 
                            0x7F => sq.press_backspace(), 
                            b's' => sq.press_func("sqrt"), 
                            _ => {}
                        }
                    }
                },
                WindowType::Game => {
                    if let Some(snake) = self.snake_states.get_mut(&fr) {
                        snake.handle_key(key);
                    }
                },
                WindowType::Game3D => {
                    if let Some(game) = self.game3d_states.get_mut(&fr) {
                        game.handle_key(key);
                    }
                },
                WindowType::Chess => {
                    if let Some(chess) = self.chess_states.get_mut(&fr) {
                        chess.handle_key(key);
                    }
                },
                WindowType::Chess3D => {
                    if let Some(state) = self.chess3d_states.get_mut(&fr) {
                        state.handle_key(key);
                    }
                },
                #[cfg(feature = "emulators")]
                WindowType::NesEmu => {
                    if let Some(an) = self.nes_states.get_mut(&fr) {
                        an.handle_key(key);
                    }
                },
                #[cfg(feature = "emulators")]
                WindowType::GameBoyEmu => {
                    if let Some(an) = self.gameboy_states.get_mut(&fr) {
                        an.handle_key(key);
                    }
                },
                WindowType::BinaryViewer => {
                    if let Some(viewer) = self.binary_viewer_states.get_mut(&fr) {
                        use crate::keyboard::{T_, S_, AI_, AJ_, AM_, AO_, CW_, CV_};
                        match key {
                            T_ => viewer.handle_scancode(0x48),
                            S_ => viewer.handle_scancode(0x50),
                            AI_ => viewer.handle_scancode(0x4B),
                            AJ_ => viewer.handle_scancode(0x4D),
                            AM_ => viewer.handle_scancode(0x49),
                            AO_ => viewer.handle_scancode(0x51),
                            CW_ => viewer.handle_scancode(0x47),
                            CV_ => viewer.handle_scancode(0x4F),
                            0x09 => viewer.handle_scancode(0x0F), 
                            0x0D | 0x0A => viewer.handle_scancode(0x1C), 
                            _ => viewer.handle_key(key as char),
                        }
                    }
                },
                WindowType::LabMode => {
                    if let Some(lab) = self.lab_states.get_mut(&fr) {
                        
                        if key >= 0x20 && key < 0x7F {
                            lab.handle_char(key as char);
                        } else {
                            lab.handle_key(key);
                        }
                    }
                },
                WindowType::WifiAnalyzer => {
                    if let Some(apn) = self.wifi_analyzer_states.get_mut(&fr) {
                        apn.handle_key(key);
                    }
                },
                #[cfg(feature = "emulators")]
                WindowType::GameLab => {
                    if let Some(lab) = self.gamelab_states.get_mut(&fr) {
                        
                        if key == 0x0D || key == 0x0A {
                            if lab.active_tab == crate::game_lab::LabTab::Search {
                                
                                let bsf = lab.linked_gb_id
                                    .or_else(|| self.gameboy_states.keys().next().copied());
                                if let Some(bbh) = bsf {
                                    let lgn = !lab.search_active;
                                    if lgn {
                                        if let Some(an) = self.gameboy_states.get(&bbh) {
                                            if let Some(gl) = self.gamelab_states.get_mut(&fr) {
                                                gl.search_initial(an);
                                            }
                                        }
                                    } else {
                                        if let Some(an) = self.gameboy_states.get(&bbh) {
                                            if let Some(gl) = self.gamelab_states.get_mut(&fr) {
                                                gl.search_filter(an);
                                            }
                                        }
                                    }
                                }
                                return;
                            }
                        }
                        lab.handle_key(key);
                    }
                },
                WindowType::Browser => {
                    use crate::keyboard::{AI_, AJ_, CW_, CV_, DE_, AM_, AO_};
                    let ctrl = crate::keyboard::sx(0x1D);
                    crate::serial_println!("[BROWSER] Key received: {} (0x{:02X}) cursor={} url_len={} sel={}", 
                        if key >= 0x20 && key < 0x7F { key as char } else { '?' }, key,
                        self.browser_url_cursor, self.browser_url_input.len(), self.browser_url_select_all);
                    
                    
                    if self.browser_url_select_all {
                        match key {
                            0x08 | _ if key == DE_ => {
                                
                                self.browser_url_input.clear();
                                self.browser_url_cursor = 0;
                                self.browser_url_select_all = false;
                                return;
                            },
                            0x1B => {
                                
                                self.browser_url_select_all = false;
                                return;
                            },
                            0x0D | 0x0A => {
                                
                                self.browser_url_select_all = false;
                            },
                            32..=126 => {
                                
                                self.browser_url_input.clear();
                                self.browser_url_input.push(key as char);
                                self.browser_url_cursor = 1;
                                self.browser_url_select_all = false;
                                return;
                            },
                            _ => {
                                
                                self.browser_url_select_all = false;
                            }
                        }
                    }
                    
                    
                    if ctrl && (key == b'a' || key == b'A') {
                        self.browser_url_select_all = true;
                        self.browser_url_cursor = self.browser_url_input.len();
                        return;
                    }
                    
                    
                    if self.browser_loading && key != 0x1B {
                        crate::serial_println!("[BROWSER] Key ignored: loading in progress");
                    } else {
                    match key {
                        0x08 => { 
                            if self.browser_url_cursor > 0 {
                                self.browser_url_cursor -= 1;
                                if self.browser_url_cursor < self.browser_url_input.len() {
                                    self.browser_url_input.remove(self.browser_url_cursor);
                                }
                            }
                        },
                        0x0D | 0x0A => { 
                            if !self.browser_url_input.is_empty() && !self.browser_loading {
                                self.browser_loading = true;
                                let url = self.browser_url_input.clone();
                                crate::serial_println!("[DESKTOP] Browser navigate async: {}", url);
                                {
                                    let mut pending = ABC_.lock();
                                    *pending = Some(url);
                                }
                                ST_.store(true, Ordering::SeqCst);
                                crate::thread::dzu("browser-nav", hir, 0);
                            }
                        },
                        0x1B => { 
                            if self.browser_loading {
                                self.browser_loading = false;
                            } else {
                                self.browser_url_input.clear();
                                self.browser_url_cursor = 0;
                            }
                        },
                        _ if key == AI_ => {
                            if ctrl {
                                
                                while self.browser_url_cursor > 0 {
                                    self.browser_url_cursor -= 1;
                                    if self.browser_url_cursor > 0 {
                                        let c = self.browser_url_input.as_bytes()[self.browser_url_cursor - 1];
                                        if c == b' ' || c == b'/' || c == b'.' || c == b':' {
                                            break;
                                        }
                                    }
                                }
                            } else if self.browser_url_cursor > 0 {
                                self.browser_url_cursor -= 1;
                            }
                        },
                        _ if key == AJ_ => {
                            if ctrl {
                                
                                let len = self.browser_url_input.len();
                                while self.browser_url_cursor < len {
                                    self.browser_url_cursor += 1;
                                    if self.browser_url_cursor < len {
                                        let c = self.browser_url_input.as_bytes()[self.browser_url_cursor];
                                        if c == b' ' || c == b'/' || c == b'.' || c == b':' {
                                            break;
                                        }
                                    }
                                }
                            } else if self.browser_url_cursor < self.browser_url_input.len() {
                                self.browser_url_cursor += 1;
                            }
                        },
                        _ if key == CW_ => {
                            self.browser_url_cursor = 0;
                        },
                        _ if key == CV_ => {
                            self.browser_url_cursor = self.browser_url_input.len();
                        },
                        _ if key == DE_ => {
                            if self.browser_url_cursor < self.browser_url_input.len() {
                                self.browser_url_input.remove(self.browser_url_cursor);
                            }
                        },
                        _ if key == AM_ => {
                            
                            if let Some(ref mut browser) = self.browser {
                                browser.scroll(-200);
                            }
                        },
                        _ if key == AO_ => {
                            
                            if let Some(ref mut browser) = self.browser {
                                browser.scroll(200);
                            }
                        },
                        _ if ctrl && (key == b'l' || key == b'L') => {
                            
                            self.browser_url_select_all = true;
                            self.browser_url_cursor = self.browser_url_input.len();
                        },
                        _ if ctrl && (key == b'r' || key == b'R') => {
                            
                            if let Some(ref mut browser) = self.browser {
                                let _ = browser.refresh();
                            }
                        },
                        _ if ctrl && (key == b'a' || key == b'A') => {
                            
                            self.browser_url_select_all = true;
                            self.browser_url_cursor = self.browser_url_input.len();
                        },
                        _ if key == b'\t' => {
                            
                            if !self.browser_url_input.contains("://") && !self.browser_url_input.is_empty() {
                                self.browser_url_input = alloc::format!("http://{}", self.browser_url_input);
                                self.browser_url_cursor = self.browser_url_input.len();
                            }
                        },
                        32..=126 => { 
                            if self.browser_url_input.len() < 512 {
                                if self.browser_url_cursor >= self.browser_url_input.len() {
                                    self.browser_url_input.push(key as char);
                                } else {
                                    self.browser_url_input.insert(self.browser_url_cursor, key as char);
                                }
                                self.browser_url_cursor += 1;
                            }
                        },
                        _ => {}
                    }
                    }
                },
                WindowType::HexViewer => {
                    use crate::keyboard::{T_, S_, AM_, AO_, CW_, CV_};
                    if let Some(window) = self.windows.iter_mut().find(|w| w.id == fr) {
                        let oe = ((window.height.saturating_sub(J_() + 20)) / 16) as usize;
                        let aab = window.content.len().saturating_sub(oe);
                        match key {
                            T_ => window.scroll_offset = window.scroll_offset.saturating_sub(1),
                            S_ => window.scroll_offset = (window.scroll_offset + 1).min(aab),
                            AM_ => window.scroll_offset = window.scroll_offset.saturating_sub(oe),
                            AO_ => window.scroll_offset = (window.scroll_offset + oe).min(aab),
                            CW_ => window.scroll_offset = 0,
                            CV_ => window.scroll_offset = aab,
                            _ => {}
                        }
                    }
                },
                WindowType::WifiPassword => {
                    match key {
                        0x1B => { 
                            self.windows.retain(|w| w.id != fr);
                        },
                        0x08 | 0x7F => { 
                            self.wifi_password_input.pop();
                        },
                        0x0D | 0x0A => { 
                            if !self.wifi_password_input.is_empty() {
                                crate::drivers::net::wifi::eyl(
                                    &self.wifi_connecting_ssid,
                                    &self.wifi_password_input,
                                );
                                self.windows.retain(|w| w.id != fr);
                            } else {
                                self.wifi_error_msg = Some(String::from("Password cannot be empty"));
                            }
                        },
                        b' '..=b'~' => { 
                            if self.wifi_password_input.len() < 128 {
                                self.wifi_password_input.push(key as char);
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }
    
    
    fn handle_filemanager_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_, DE_};
        
        
        {
            let lxh = self.windows.iter()
                .find(|w| w.focused && w.window_type == WindowType::FileManager)
                .map(|w| w.id);
            if let Some(sa) = lxh {
                let search_focused = self.fm_states.get(&sa).map(|f| f.search_focused).unwrap_or(false);
                if search_focused {
                    if key == 0x1B { 
                        if let Some(kn) = self.fm_states.get_mut(&sa) {
                            kn.search_focused = false;
                            kn.search_query.clear();
                        }
                        let path = self.windows.iter().find(|w| w.id == sa)
                            .and_then(|w| w.file_path.clone()).unwrap_or_else(|| String::from("/"));
                        self.refresh_file_manager(&path);
                        return;
                    } else if key == 0x08 { 
                        if let Some(kn) = self.fm_states.get_mut(&sa) {
                            kn.search_query.pop();
                        }
                        let path = self.windows.iter().find(|w| w.id == sa)
                            .and_then(|w| w.file_path.clone()).unwrap_or_else(|| String::from("/"));
                        self.refresh_file_manager(&path);
                        return;
                    } else if key == 0x0D || key == 0x0A { 
                        if let Some(kn) = self.fm_states.get_mut(&sa) {
                            kn.search_focused = false;
                        }
                        return;
                    } else if key >= 0x20 && key < 0x7F {
                        if let Some(kn) = self.fm_states.get_mut(&sa) {
                            if kn.search_query.len() < 32 {
                                kn.search_query.push(key as char);
                            }
                        }
                        let path = self.windows.iter().find(|w| w.id == sa)
                            .and_then(|w| w.file_path.clone()).unwrap_or_else(|| String::from("/"));
                        self.refresh_file_manager(&path);
                        return;
                    }
                    return;
                }
            }
        }
        
        let mut action: Option<(String, bool)> = None; 
        let mut hre: Option<String> = None;
        let mut dbp = false;
        let mut ipx = false;
        let mut ofd = false;
        
        let mut izo: Option<(String, String, String)> = None; 
        
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileManager) {
            
            if window.title.starts_with("RENAME:") {
                if key == 0x0D || key == 0x0A { 
                    let evp = String::from(&window.title[7..]);
                    let dbq = self.input_buffer.clone();
                    self.input_buffer.clear();
                    window.title = String::from("File Manager");
                    let ht = window.file_path.clone().unwrap_or_else(|| String::from("/"));
                    izo = Some((evp, dbq, ht));
                } else if key == 0x08 { 
                    self.input_buffer.pop();
                    return;
                } else if key == 0x1B { 
                    self.input_buffer.clear();
                    window.title = String::from("File Manager");
                    return;
                } else if key >= 0x20 && key < 0x7F {
                    self.input_buffer.push(key as char);
                    return;
                }
                return;
            }
            
            
            let adp = window.content.len().saturating_sub(7); 
            
            if key == T_ {
                if window.selected_index > 0 {
                    window.selected_index -= 1;
                }
            } else if key == S_ {
                if window.selected_index < adp.saturating_sub(1) {
                    window.selected_index += 1;
                }
            } else if key == 0x08 { 
                action = Some((String::from(".."), true));
            } else if key == DE_ { 
                let idx = window.selected_index + 5;
                if idx < window.content.len().saturating_sub(2) {
                    let line = &window.content[idx];
                    if let Some(sj) = line.find(']') {
                        if sj + 2 < line.len() {
                            let ef = &line[sj + 2..];
                            if let Some(aec) = ef.find(' ') {
                                let filename = String::from(ef[..aec].trim());
                                if filename != ".." {
                                    hre = Some(filename);
                                }
                            }
                        }
                    }
                }
            } else if key == b'n' || key == b'N' { 
                dbp = true;
            } else if key == b'd' || key == b'D' { 
                ipx = true;
            } else if key == b'r' || key == b'R' { 
                ofd = true;
                let idx = window.selected_index + 5;
                if idx < window.content.len().saturating_sub(2) {
                    let line = &window.content[idx];
                    if let Some(sj) = line.find(']') {
                        if sj + 2 < line.len() {
                            let ef = &line[sj + 2..];
                            if let Some(aec) = ef.find(' ') {
                                let filename = String::from(ef[..aec].trim());
                                if filename != ".." {
                                    self.input_buffer = filename.clone();
                                    window.title = format!("RENAME:{}", filename);
                                }
                            }
                        }
                    }
                }
            } else if key == 0x0D || key == 0x0A { 
                
                let idx = window.selected_index + 5; 
                if idx < window.content.len().saturating_sub(2) { 
                    let line = &window.content[idx];
                    
                    if let Some(sj) = line.find(']') {
                        if sj + 2 < line.len() {
                            let ef = &line[sj + 2..];
                            if let Some(aec) = ef.find(' ') {
                                let filename = String::from(ef[..aec].trim());
                                let is_dir = line.contains("[D]");
                                action = Some((filename, is_dir));
                            }
                        }
                    }
                }
            }
        }
        
        
        if let Some((evp, dbq, ht)) = izo {
            let isd = if ht == "/" { format!("/{}", evp) } else { format!("{}/{}", ht, evp) };
            let iqf = if ht == "/" { format!("/{}", dbq) } else { format!("{}/{}", ht, dbq) };
            let _ = crate::ramfs::bh(|fs| fs.mv(&isd, &iqf));
            crate::serial_println!("[FM] Renamed: {} -> {}", isd, iqf);
            self.refresh_file_manager(&ht);
            return;
        }
        
        
        if let Some(filename) = hre {
            let ht = self.windows.iter()
                .find(|w| w.focused && w.window_type == WindowType::FileManager)
                .and_then(|w| w.file_path.clone())
                .unwrap_or_else(|| String::from("/"));
            let kg = if ht == "/" { format!("/{}", filename) } else { format!("{}/{}", ht, filename) };
            let _ = crate::ramfs::bh(|fs| fs.rm(&kg));
            crate::serial_println!("[FM] Deleted: {}", kg);
            self.refresh_file_manager(&ht);
            return;
        }
        
        
        if dbp {
            let ht = self.windows.iter()
                .find(|w| w.focused && w.window_type == WindowType::FileManager)
                .and_then(|w| w.file_path.clone())
                .unwrap_or_else(|| String::from("/"));
            let name = format!("new_file_{}.txt", self.frame_count % 1000);
            let kg = if ht == "/" { format!("/{}", name) } else { format!("{}/{}", ht, name) };
            let _ = crate::ramfs::bh(|fs| fs.touch(&kg));
            crate::serial_println!("[FM] Created file: {}", kg);
            self.refresh_file_manager(&ht);
            return;
        }
        
        
        if ipx {
            let ht = self.windows.iter()
                .find(|w| w.focused && w.window_type == WindowType::FileManager)
                .and_then(|w| w.file_path.clone())
                .unwrap_or_else(|| String::from("/"));
            let name = format!("folder_{}", self.frame_count % 1000);
            let kg = if ht == "/" { format!("/{}", name) } else { format!("{}/{}", ht, name) };
            let _ = crate::ramfs::bh(|fs| fs.mkdir(&kg));
            crate::serial_println!("[FM] Created folder: {}", kg);
            self.refresh_file_manager(&ht);
            return;
        }
        
        
        if let Some((filename, is_dir)) = action {
            if is_dir {
                
                self.navigate_file_manager(&filename);
            } else {
                self.open_file(&filename);
            }
        }
    }
    
    
    fn refresh_file_manager(&mut self, path: &str) {
        
        let (dzt, gvo, search_q) = {
            let sa = self.windows.iter()
                .find(|w| w.focused && w.window_type == WindowType::FileManager)
                .map(|w| w.id);
            if let Some(sa) = sa {
                if let Some(kn) = self.fm_states.get(&sa) {
                    (kn.sort_column, kn.sort_ascending, kn.search_query.clone())
                } else { (0, true, String::new()) }
            } else { return; }
        };
        
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileManager) {
            window.content.clear();
            window.content.push(String::from("=== File Manager ==="));
            window.content.push(format!("Path: {}", path));
            window.content.push(String::from(""));
            window.content.push(String::from("  Name              Type       Size    Program"));
            window.content.push(String::from("  ────────────────────────────────────────────"));
            
            if path != "/" {
                window.content.push(String::from("  [D] ..             DIR        ---     ---"));
            }
            
            let nrw = if path == "/" { Some("/") } else { Some(path) };
            if let Ok(entries) = crate::ramfs::bh(|fs| fs.ls(nrw)) {
                
                let apb: String = search_q.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                let mut filtered: Vec<&(String, crate::ramfs::FileType, usize)> = if apb.is_empty() {
                    entries.iter().collect()
                } else {
                    entries.iter().filter(|(name, _, _)| {
                        let duw: String = name.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                        duw.contains(apb.as_str())
                    }).collect()
                };
                
                
                filtered.sort_by(|a, b| {
                    
                    let hdo = a.1 == crate::ramfs::FileType::Directory;
                    let jyy = b.1 == crate::ramfs::FileType::Directory;
                    if hdo != jyy {
                        return if hdo { core::cmp::Ordering::Less } else { core::cmp::Ordering::Greater };
                    }
                    let isq = match dzt {
                        1 => { 
                            let ltc = a.0.rsplit('.').next().unwrap_or("");
                            let ltd = b.0.rsplit('.').next().unwrap_or("");
                            ltc.cmp(ltd)
                        }
                        2 => a.2.cmp(&b.2), 
                        _ => a.0.cmp(&b.0), 
                    };
                    if gvo { isq } else { isq.reverse() }
                });
                
                for (name, wf, size) in filtered.iter().take(200) {
                    let icon = if *wf == crate::ramfs::FileType::Directory { 
                        "[D]" 
                    } else { 
                        crate::file_assoc::get_file_icon(name)
                    };
                    let azb = if *wf == crate::ramfs::FileType::Directory {
                        String::from("---")
                    } else {
                        String::from(crate::file_assoc::cyr(name).name())
                    };
                    let fxy = if *wf == crate::ramfs::FileType::Directory { "DIR" } else { "FILE" };
                    window.content.push(format!("  {} {:<14} {:<10} {:<7} {}", icon, name, fxy, size, azb));
                }
            }
            if window.content.len() <= 5 + if path != "/" { 1 } else { 0 } {
                window.content.push(String::from("  (empty directory)"));
            }
            window.content.push(String::from(""));
            window.content.push(String::from("  [Del] Delete | [N] New File | [D] New Folder | [F2] Rename"));
            
            window.file_path = Some(String::from(path));
            window.selected_index = 0;
            window.scroll_offset = 0;
        }
    }
    
    
    fn navigate_file_manager(&mut self, cil: &str) {
        
        let (bcx, sa) = {
            if let Some(window) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager) {
                let ht = window.file_path.clone().unwrap_or_else(|| String::from("/"));
                let bcx = if cil == ".." {
                    if ht == "/" {
                        String::from("/")
                    } else {
                        let jw = ht.trim_end_matches('/');
                        match jw.rfind('/') {
                            Some(0) => String::from("/"),
                            Some(pos) => String::from(&jw[..pos]),
                            None => String::from("/"),
                        }
                    }
                } else if ht == "/" {
                    format!("/{}", cil)
                } else {
                    format!("{}/{}", ht.trim_end_matches('/'), cil)
                };
                crate::serial_println!("[FM] Navigate: {} -> {}", ht, bcx);
                (bcx, window.id)
            } else { return; }
        };
        
        
        if let Some(kn) = self.fm_states.get_mut(&sa) {
            kn.search_query.clear();
            kn.search_focused = false;
        }
        
        
        if let Some(window) = self.windows.iter_mut().find(|w| w.id == sa) {
            window.file_path = Some(bcx.clone());
        }
        
        
        self.refresh_file_manager(&bcx);
        
        
        if let Some(kn) = self.fm_states.get_mut(&sa) {
            kn.push_history(&bcx);
        }
    }
    
    
    fn open_file(&mut self, filename: &str) {
        use crate::file_assoc::{cyr, Program};
        
        let program = cyr(filename);
        let offset = (self.windows.len() as i32 * 25) % 150;
        
        match program {
            Program::TextEditor => {
                let id = self.create_window(&format!("TrustCode: {}", filename), 150 + offset, 80 + offset, 700, 500, WindowType::TextEditor);
                
                if let Some(editor) = self.editor_states.get_mut(&id) {
                    editor.load_file(filename);
                }
                crate::serial_println!("[TrustCode] Opened: {}", filename);
            },
            Program::ImageViewer => {
                let id = self.create_window(&format!("View: {}", filename), 180 + offset, 100 + offset, 500, 420, WindowType::ImageViewer);
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.file_path = Some(String::from(filename));
                    window.content.clear();
                    
                    
                    let file_path = format!("/{}", filename);
                    if let Ok(raw_data) = crate::ramfs::bh(|fs| fs.read_file(&file_path).map(|d| d.to_vec())) {
                        
                        if let Some(iv) = crate::theme::bmp::dtq(&raw_data) {
                            let mut state = ImageViewerState::new();
                            state.img_width = iv.width;
                            state.img_height = iv.height;
                            state.pixels = iv.pixels;
                            
                            let lwn = (480 * 100) / iv.width.max(1);
                            let lwm = (360 * 100) / iv.height.max(1);
                            state.zoom = lwn.min(lwm).min(200);
                            self.image_viewer_states.insert(id, state);
                            crate::serial_println!("[ImageViewer] Loaded BMP: {}x{}", iv.width, iv.height);
                            window.content.push(format!("Image: {} ({}x{} BMP)", filename, iv.width, iv.height));
                        } else {
                            
                            window.content.push(format!("=== Image: {} ===", filename));
                            window.content.push(format!("Size: {} bytes", raw_data.len()));
                            if raw_data.len() >= 2 && &raw_data[0..2] == b"BM" {
                                window.content.push(String::from("BMP detected but failed to parse"));
                            } else {
                                window.content.push(String::from("Format not supported (BMP only)"));
                            }
                            self.image_viewer_states.insert(id, ImageViewerState::new());
                        }
                    } else {
                        window.content.push(String::from("Failed to read file"));
                        self.image_viewer_states.insert(id, ImageViewerState::new());
                    }
                }
            },
            Program::HexViewer => {
                let id = self.create_window(&format!("Hex: {}", filename), 160 + offset, 80 + offset, 500, 350, WindowType::HexViewer);
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.file_path = Some(String::from(filename));
                    window.content.clear();
                    window.content.push(format!("=== Hex View: {} ===", filename));
                    window.content.push(String::new());
                    window.content.push(String::from("Offset   00 01 02 03 04 05 06 07  ASCII"));
                    window.content.push(String::from("──────── ─────────────────────── ────────"));
                    
                    let file_path = format!("/{}", filename);
                    if let Ok(content) = crate::ramfs::bh(|fs| fs.read_file(&file_path).map(|d| d.to_vec())) {
                        let total_bytes = content.len();
                        for (i, df) in content.chunks(8).enumerate() {
                            let offset = i * 8;
                            let ga: String = df.iter()
                                .map(|b| format!("{:02X} ", b))
                                .collect();
                            let ascii: String = df.iter()
                                .map(|&b| if b >= 0x20 && b < 0x7F { b as char } else { '.' })
                                .collect();
                            window.content.push(format!("{:08X} {:<24} {}", offset, ga, ascii));
                        }
                        window.content.push(String::new());
                        window.content.push(format!("Total: {} bytes ({} lines)", total_bytes, window.content.len() - 4));
                    }
                    window.scroll_offset = 0;
                }
            },
            Program::Terminal => {
                
                crate::serial_println!("[EXEC] Would execute: {}", filename);
                let id = self.create_window("Execution", 200 + offset, 150 + offset, 400, 200, WindowType::Terminal);
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.content.clear();
                    window.content.push(format!("Executing: {}", filename));
                    window.content.push(String::from(""));
                    window.content.push(String::from("(ELF execution not yet integrated in GUI)"));
                }
            },
            _ => {
                
                crate::serial_println!("[OPEN] No handler for: {}", filename);
            }
        }
    }
    
    
    fn handle_settings_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_};
        
        
        if key == T_ {
            if self.settings_category > 0 {
                self.settings_category -= 1;
            }
            return;
        }
        if key == S_ {
            if self.settings_category < 7 {
                self.settings_category += 1;
            }
            return;
        }
        
        match self.settings_category {
            0 => { 
                if key == b'1' {
                    pkq();
                } else if key == b'2' {
                    let current = *GP_.lock();
                    let next = if current <= 0.5 { 1.0 } else if current <= 1.0 { 2.0 } else { 0.5 };
                    *GP_.lock() = next;
                } else if key == b'3' {
                    
                    let akg = match self.desktop_tier {
                        DesktopTier::Full => DesktopTier::Standard,
                        DesktopTier::Standard => DesktopTier::Minimal,
                        DesktopTier::Minimal | DesktopTier::CliOnly => DesktopTier::Full,
                    };
                    self.desktop_tier = akg;
                    self.tier_manual_override = true;
                    self.fps_low_count = 0;
                    self.fps_high_count = 0;
                    self.needs_full_redraw = true;
                    self.background_cached = false;
                    crate::serial_println!("[Desktop] Manual tier change: {:?} (override=ON)", akg);
                } else if key == b'4' {
                    
                    self.tier_manual_override = !self.tier_manual_override;
                    self.fps_low_count = 0;
                    self.fps_high_count = 0;
                    crate::serial_println!("[Desktop] Manual override: {}", if self.tier_manual_override { "ON" } else { "OFF (auto)" });
                }
            },
            1 => { 
                if key == b'1' {
                    let vd = &mut S.lock().sys_volume;
                    *vd = (*vd + 10).min(100);
                } else if key == b'2' {
                    let vd = &mut S.lock().sys_volume;
                    *vd = vd.saturating_sub(10);
                }
            },
            2 => { 
                
            },
            3 => { 
                if key == b'1' {
                    
                    let current = crate::theme::Dj.read().name.clone();
                    let next = if current == "windows11_dark" { "dark" } else { "windows11" };
                    crate::theme::jex(next);
                    self.needs_full_redraw = true;
                    self.background_cached = false;
                }
            },
            4 => { 
                if key == b'1' {
                    crate::accessibility::gzg();
                    self.needs_full_redraw = true;
                    self.background_cached = false;
                } else if key == b'2' {
                    crate::accessibility::hqf();
                } else if key == b'3' {
                    crate::accessibility::hqe();
                } else if key == b'4' {
                    crate::accessibility::jnd();
                } else if key == b'5' {
                    crate::accessibility::hqg();
                }
            },
            5 => { 
                
            },
            6 => { 
                if key == b'3' || key == 0x0D {
                    let offset = (self.windows.len() as i32 * 20) % 100;
                    self.create_window("File Associations", 250 + offset, 130 + offset, 500, 400, WindowType::FileAssociations);
                }
            },
            7 => { 
                
            },
            _ => {}
        }
    }
    
    
    fn draw_settings_gui(&self, window: &Window) {
        let wx = window.x;
        let wy = window.y;
        let ca = window.width;
        let er = window.height;
        
        if ca < 200 || er < 160 { return; }
        
        let bn = wy + J_() as i32;
        let en = er.saturating_sub(J_());
        let lv = wx.max(0) as u32;
        
        let dso = crate::accessibility::btq();
        let cuc = if dso { 0xFF0A0A0A } else { 0xFF060E08 };
        let fiv = if dso { 0xFF000000 } else { 0xFF0A140C };
        let ph = 0xFF2A6A3Au32;
        let ql = I_;
        let ou = 0xFF88AA88;
        let qe = 0xFFBBDDBB;
        let rg = 0xFF446644;
        
        
        let rq = 140u32;
        framebuffer::fill_rect(lv, bn as u32, rq, en, cuc);
        
        let cgr = [
            ("Display",      "@"),
            ("Sound",        "~"),
            ("Taskbar",      "_"),
            ("Personal.",    "*"),
            ("Access.",      "A"),
            ("Network",      "N"),
            ("Apps",         "#"),
            ("About",        "?"),
        ];
        
        let sy = 32i32;
        let mut ak = bn + 8;
        for (i, (label, icon)) in cgr.iter().enumerate() {
            let is_active = i as u8 == self.settings_category;
            
            if is_active {
                draw_rounded_rect(lv as i32 + 4, ak - 1, rq - 8, sy as u32 - 2, 4, 0xFF0C2A14);
                framebuffer::fill_rect(lv + 2, (ak + 2) as u32, 3, (sy - 6) as u32, ql);
            }
            
            let c = if is_active { ql } else { ou };
            self.draw_text_smooth(lv as i32 + 14, ak + 8, icon, if is_active { ql } else { rg });
            self.draw_text_smooth(lv as i32 + 28, ak + 8, label, c);
            ak += sy;
        }
        
        
        framebuffer::fill_rect(lv + rq - 1, bn as u32, 1, en, 0xFF1A3A1A);
        
        
        let cx = lv + rq;
        let aq = ca.saturating_sub(rq);
        framebuffer::fill_rect(cx, bn as u32, aq, en, fiv);
        
        let p = cx as i32 + 20; 
        let mut o = bn + 16;  
        let bw = 22i32;
        
        match self.settings_category {
            0 => { 
                self.draw_text_smooth(p, o, "Display", ql);
                self.draw_text_smooth(p + 1, o, "Display", ql); 
                o += bw + 8;
                
                self.draw_text_smooth(p, o, "Resolution", ou);
                self.draw_text_smooth(p + 120, o, &alloc::format!("{}x{}", self.width, self.height), qe);
                o += bw;
                
                let cei = crate::theme::Dj.read().name.clone();
                self.draw_text_smooth(p, o, "Theme", ou);
                self.draw_text_smooth(p + 120, o, &cei, qe);
                o += bw + 8;
                
                
                let jwf = awb();
                self.draw_settings_toggle(p, o, "[1] Animations", jwf);
                o += bw;
                
                
                let speed = *GP_.lock();
                self.draw_text_smooth(p, o, "[2] Anim Speed", ou);
                self.draw_text_smooth(p + 180, o, &alloc::format!("{:.1}x", speed), qe);
                o += bw + 8;
                
                
                framebuffer::mn((p) as u32, (o + 2) as u32, aq.saturating_sub(40), 0xFF1A3A1A);
                o += bw;
                self.draw_text_smooth(p, o, "Desktop Mode", ql);
                o += bw;
                
                let pjj = match self.desktop_tier {
                    DesktopTier::Full => "Full",
                    DesktopTier::Standard => "Standard",
                    DesktopTier::Minimal => "Minimal",
                    DesktopTier::CliOnly => "CLI Only",
                };
                self.draw_text_smooth(p, o, "[3] Mode", ou);
                self.draw_text_smooth(p + 120, o, pjj, qe);
                o += bw;
                
                self.draw_settings_toggle(p, o, "[4] Manual Override", self.tier_manual_override);
                o += bw;
                
                if !self.tier_manual_override {
                    self.draw_text_smooth(p + 12, o, "(auto-adjusts based on FPS)", rg);
                } else {
                    self.draw_text_smooth(p + 12, o, "(locked, no auto-downgrade)", qe);
                }
                o += bw;
            },
            1 => { 
                self.draw_text_smooth(p, o, "Sound", ql);
                self.draw_text_smooth(p + 1, o, "Sound", ql);
                o += bw + 8;
                
                self.draw_text_smooth(p, o, "Master Volume", ou);
                let vd = self.sys_volume;
                self.draw_settings_slider(p + 140, o, aq.saturating_sub(180) as i32, vd, 100);
                o += bw;
                
                self.draw_text_smooth(p, o, "[1] Volume +  [2] Volume -", rg);
                o += bw + 8;
                
                
                self.draw_text_smooth(p, o, "Audio Device", ou);
                o += bw;
                let fte = if crate::drivers::hda::is_initialized() { "Intel HDA (active)" } else { "Not detected" };
                self.draw_text_smooth(p + 12, o, fte, rg);
            },
            2 => { 
                self.draw_text_smooth(p, o, "Taskbar", ql);
                self.draw_text_smooth(p + 1, o, "Taskbar", ql);
                o += bw + 8;
                
                let aiv = crate::theme::taskbar();
                self.draw_text_smooth(p, o, "Position", ou);
                let bdb = match aiv.position {
                    crate::theme::TaskbarPosition::Bottom => "Bottom",
                    crate::theme::TaskbarPosition::Top => "Top",
                    crate::theme::TaskbarPosition::Left => "Left",
                    crate::theme::TaskbarPosition::Right => "Right",
                };
                self.draw_text_smooth(p + 120, o, bdb, qe);
                o += bw;
                
                self.draw_text_smooth(p, o, "Height", ou);
                self.draw_text_smooth(p + 120, o, &alloc::format!("{}px", aiv.height), qe);
                o += bw;
                
                self.draw_settings_toggle(p, o, "Show Clock", aiv.show_clock);
                o += bw;
                
                self.draw_settings_toggle(p, o, "Show Date", aiv.show_date);
                o += bw;
                
                self.draw_settings_toggle(p, o, "Centered Icons", aiv.centered_icons);
            },
            3 => { 
                self.draw_text_smooth(p, o, "Personalization", ql);
                self.draw_text_smooth(p + 1, o, "Personalization", ql);
                o += bw + 8;
                
                let cei = crate::theme::Dj.read().name.clone();
                self.draw_text_smooth(p, o, "[1] Theme", ou);
                self.draw_text_smooth(p + 120, o, &cei, qe);
                o += bw;
                
                self.draw_text_smooth(p, o, "Available themes:", rg);
                o += bw;
                let pip = ["dark_green", "windows11_dark"];
                let labels = ["TrustOS Dark", "Windows 11 Dark"];
                for (i, label) in labels.iter().enumerate() {
                    let is_current = cei == pip[i];
                    let c = if is_current { ql } else { ou };
                    let marker = if is_current { " *" } else { "  " };
                    self.draw_text_smooth(p + 16, o, &alloc::format!("{}{}", marker, label), c);
                    o += bw;
                }
                o += 8;
                
                let colors = crate::theme::colors();
                self.draw_text_smooth(p, o, "Accent Color", ou);
                
                framebuffer::fill_rect((p + 120) as u32, o as u32, 20, 14, colors.accent);
                o += bw;
                
                self.draw_text_smooth(p, o, "Background", ou);
                framebuffer::fill_rect((p + 120) as u32, o as u32, 20, 14, colors.background);
            },
            4 => { 
                self.draw_text_smooth(p, o, "Accessibility", ql);
                self.draw_text_smooth(p + 1, o, "Accessibility", ql);
                o += bw + 8;
                
                self.draw_settings_toggle(p, o, "[1] High Contrast", crate::accessibility::btq());
                o += bw;
                
                self.draw_text_smooth(p, o, "[2] Font Size", ou);
                self.draw_text_smooth(p + 160, o, crate::accessibility::cyn().label(), qe);
                o += bw;
                
                self.draw_text_smooth(p, o, "[3] Cursor Size", ou);
                self.draw_text_smooth(p + 160, o, crate::accessibility::cyl().label(), qe);
                o += bw;
                
                self.draw_settings_toggle(p, o, "[4] Sticky Keys", crate::accessibility::bnc());
                o += bw;
                
                self.draw_text_smooth(p, o, "[5] Mouse Speed", ou);
                self.draw_text_smooth(p + 160, o, crate::accessibility::cyq().label(), qe);
            },
            5 => { 
                self.draw_text_smooth(p, o, "Network", ql);
                self.draw_text_smooth(p + 1, o, "Network", ql);
                o += bw + 8;
                
                
                self.draw_text_smooth(p, o, "Interface", ou);
                o += bw;
                
                if let Some(mac) = crate::network::aqu() {
                    self.draw_text_smooth(p + 12, o, &alloc::format!("MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]), qe);
                } else {
                    self.draw_text_smooth(p + 12, o, "MAC: Not available", rg);
                }
                o += bw;
                
                if let Some((ip, mask, fz)) = crate::network::rd() {
                    self.draw_text_smooth(p + 12, o, &alloc::format!("IP:   {}", ip), qe);
                    o += bw;
                    self.draw_text_smooth(p + 12, o, &alloc::format!("Mask: {}", mask), qe);
                    o += bw;
                    if let Some(g) = fz {
                        self.draw_text_smooth(p + 12, o, &alloc::format!("GW:   {}", g), qe);
                    }
                } else {
                    self.draw_text_smooth(p + 12, o, "IP: Waiting for DHCP...", rg);
                }
                o += bw + 8;
                
                let driver = if crate::virtio_net::is_initialized() { "virtio-net (active)" }
                    else if crate::drivers::net::aoh() { "RTL8169/e1000 (active)" }
                    else { "No driver loaded" };
                self.draw_text_smooth(p, o, "Driver", ou);
                self.draw_text_smooth(p + 80, o, driver, qe);
            },
            6 => { 
                self.draw_text_smooth(p, o, "Default Apps", ql);
                self.draw_text_smooth(p + 1, o, "Default Apps", ql);
                o += bw + 8;
                
                let fhq = crate::file_assoc::iko();
                self.draw_text_smooth(p, o, "Extension", rg);
                self.draw_text_smooth(p + 100, o, "Program", rg);
                self.draw_text_smooth(p + 220, o, "Type", rg);
                o += 4;
                framebuffer::mn((p) as u32, (o + 12) as u32, aq.saturating_sub(40), 0xFF1A3A1A);
                o += bw;
                
                for (ext, azb, desc) in fhq.iter().take(10) {
                    self.draw_text_smooth(p, o, &alloc::format!(".{}", ext), qe);
                    self.draw_text_smooth(p + 100, o, azb, ou);
                    self.draw_text_smooth(p + 220, o, desc, rg);
                    o += bw;
                }
                o += 8;
                self.draw_text_smooth(p, o, "[3] Edit File Associations...", ou);
            },
            7 => { 
                self.draw_text_smooth(p, o, "About TrustOS", ql);
                self.draw_text_smooth(p + 1, o, "About TrustOS", ql);
                o += bw + 8;
                
                self.draw_text_smooth(p, o, "TrustOS", 0xFFCCEECC);
                self.draw_text_smooth(p + 1, o, "TrustOS", 0xFFCCEECC);
                o += bw;
                self.draw_text_smooth(p, o, "Version 0.2.0", qe);
                o += bw;
                self.draw_text_smooth(p, o, "Bare-metal OS written in Rust", ou);
                o += bw + 8;
                
                self.draw_text_smooth(p, o, "Kernel", rg);
                self.draw_text_smooth(p + 80, o, "trustos_kernel (x86_64)", qe);
                o += bw;
                
                self.draw_text_smooth(p, o, "Arch", rg);
                self.draw_text_smooth(p + 80, o, "x86_64", qe);
                o += bw;
                
                self.draw_text_smooth(p, o, "Display", rg);
                self.draw_text_smooth(p + 80, o, &alloc::format!("{}x{}", self.width, self.height), qe);
                o += bw;
                
                self.draw_text_smooth(p, o, "AI", rg);
                self.draw_text_smooth(p + 80, o, "JARVIS (Transformer 4.4M params)", qe);
                o += bw + 8;
                
                self.draw_text_smooth(p, o, "(c) 2026 Nathan", ou);
            },
            _ => {}
        }
    }
    
    
    fn draw_settings_toggle(&self, x: i32, y: i32, label: &str, enabled: bool) {
        let ou = 0xFF88AA88;
        let ql = I_;
        self.draw_text_smooth(x, y, label, ou);
        
        let bu = x + 180;
        let gr = 36u32;
        let bwj = 16u32;
        let pms = if enabled { 0xFF1A5A2A } else { 0xFF1A1A1A };
        draw_rounded_rect(bu, y, gr, bwj, 8, pms);
        iu(bu, y, gr, bwj, 8, if enabled { ql } else { 0xFF333333 });
        
        let cbf = if enabled { bu + gr as i32 - 14 } else { bu + 2 };
        let mvx = if enabled { ql } else { 0xFF666666 };
        for ad in 0..12u32 {
            for dx in 0..12u32 {
                let lh = dx as i32 - 6;
                let kf = ad as i32 - 6;
                if lh * lh + kf * kf <= 36 {
                    framebuffer::cz((cbf + dx as i32) as u32, (y as u32 + 2 + ad), mvx);
                }
            }
        }
    }
    
    
    fn draw_settings_slider(&self, x: i32, y: i32, width: i32, value: u32, sh: u32) {
        let bwm = width.max(40) as u32;
        let ada = 6u32;
        let ty = y + 5;
        
        
        draw_rounded_rect(x, ty, bwm, ada, 3, 0xFF1A1A1A);
        
        
        let rb = ((value as u64 * bwm as u64) / sh.max(1) as u64) as u32;
        if rb > 0 {
            draw_rounded_rect(x, ty, rb.min(bwm), ada, 3, 0xFF1A5A2A);
        }
        
        
        let cbf = x + rb as i32;
        for ad in 0..10u32 {
            for dx in 0..10u32 {
                let lh = dx as i32 - 5;
                let kf = ad as i32 - 5;
                if lh * lh + kf * kf <= 25 {
                    framebuffer::cz((cbf + dx as i32 - 5).max(0) as u32, (ty as u32 - 2 + ad), I_);
                }
            }
        }
        
        
        self.draw_text_smooth(x + bwm as i32 + 8, y, &alloc::format!("{}", value), 0xFFBBDDBB);
    }
    
    
    fn handle_netscan_key(&mut self, key: u8) {
        
        if key >= b'1' && key <= b'6' {
            self.netscan_tab = key - b'1';
            return;
        }
        use crate::keyboard::{AI_, AJ_};
        if key == AI_ {
            self.netscan_tab = self.netscan_tab.saturating_sub(1);
            return;
        }
        if key == AJ_ {
            if self.netscan_tab < 5 { self.netscan_tab += 1; }
            return;
        }
        
        match self.netscan_tab {
            1 => { 
                if key == b's' || key == b'S' {
                    if let Some((_ip, _mask, fz)) = crate::network::rd() {
                        if let Some(g) = fz {
                            let target = *g.as_bytes();
                            let (results, stats) = crate::netscan::port_scanner::ixj(target);
                            if let Some(window) = self.windows.iter_mut().find(|w| w.window_type == WindowType::Cn) {
                                window.content.clear();
                                window.content.push(alloc::format!("Scan: {} | Open: {} | Closed: {} | {:.0}ms",
                                    crate::netscan::uw(target), stats.open, stats.closed, stats.elapsed_ms));
                                for ej in &results {
                                    let acr = match ej.state {
                                        crate::netscan::port_scanner::PortState::Open => "OPEN",
                                        crate::netscan::port_scanner::PortState::Closed => "closed",
                                        crate::netscan::port_scanner::PortState::Filtered => "filtered",
                                        _ => "unknown",
                                    };
                                    window.content.push(alloc::format!("  Port {}: {} ({})", ej.port, acr, ej.service));
                                }
                                if results.is_empty() {
                                    window.content.push(String::from("  No open ports found"));
                                }
                            }
                        }
                    }
                }
            },
            2 => { 
                if key == b'd' || key == b'D' {
                    let aba = crate::netscan::discovery::fhl(3000);
                    if let Some(window) = self.windows.iter_mut().find(|w| w.window_type == WindowType::Cn) {
                        window.content.clear();
                        window.content.push(alloc::format!("ARP Sweep: {} hosts found", aba.len()));
                        for host in &aba {
                            let bhv = match host.mac {
                                Some(m) => crate::netscan::bzx(m),
                                None => String::from("??:??:??:??:??:??"),
                            };
                            window.content.push(alloc::format!("  {} - {} ({}ms)",
                                crate::netscan::uw(host.ip), bhv, host.rtt_ms));
                        }
                        if aba.is_empty() {
                            window.content.push(String::from("  No hosts discovered"));
                        }
                    }
                }
            },
            3 => { 
                if key == b's' || key == b'S' {
                    if crate::netscan::sniffer::btp() {
                        crate::netscan::sniffer::dex();
                    } else {
                        crate::netscan::sniffer::deu();
                    }
                }
            },
            4 => { 
                if key == b't' || key == b'T' {
                    if let Some((_ip, _mask, fz)) = crate::network::rd() {
                        if let Some(g) = fz {
                            let target = *g.as_bytes();
                            let bcb = crate::netscan::traceroute::trace(target, 30, 5000);
                            let cjt = crate::netscan::traceroute::lxu(&bcb);
                            if let Some(window) = self.windows.iter_mut().find(|w| w.window_type == WindowType::Cn) {
                                window.content.clear();
                                for line in cjt.lines() {
                                    window.content.push(String::from(line));
                                }
                            }
                        }
                    }
                }
            },
            5 => { 
                if key == b'v' || key == b'V' {
                    if let Some((_ip, _mask, fz)) = crate::network::rd() {
                        if let Some(g) = fz {
                            let target = *g.as_bytes();
                            
                            let (port_results, _) = crate::netscan::port_scanner::ixj(target);
                            let bil: alloc::vec::Vec<u16> = port_results.iter()
                                .filter(|aa| matches!(aa.state, crate::netscan::port_scanner::PortState::Open))
                                .map(|aa| aa.port)
                                .collect();
                            let results = crate::netscan::vuln::scan(target, &bil);
                            let report = crate::netscan::vuln::format_report(target, &results);
                            if let Some(window) = self.windows.iter_mut().find(|w| w.window_type == WindowType::Cn) {
                                window.content.clear();
                                for line in report.lines() {
                                    window.content.push(String::from(line));
                                }
                            }
                        }
                    }
                }
            },
            _ => {}
        }
    }
    
    
    fn draw_netscan_gui(&self, window: &Window) {
        let wx = window.x;
        let wy = window.y;
        let ca = window.width;
        let er = window.height;
        
        if ca < 200 || er < 120 { return; }
        
        let bn = wy + J_() as i32;
        let en = er.saturating_sub(J_());
        let lv = wx.max(0) as u32;
        
        let bg = 0xFF0A140Cu32;
        let pcr = 0xFF060E08u32;
        let pcq = 0xFF0C2A14u32;
        let ql = I_;
        let ou = 0xFF88AA88u32;
        let qe = 0xFFBBDDBBu32;
        let rg = 0xFF446644u32;
        let ri = 0xFF1A3A1Au32;
        
        
        framebuffer::fill_rect(lv, bn as u32, ca, en, bg);
        
        
        let bph = 28u32;
        framebuffer::fill_rect(lv, bn as u32, ca, bph, pcr);
        framebuffer::fill_rect(lv, (bn + bph as i32) as u32, ca, 1, ri);
        
        let tabs = ["Dashboard", "PortScan", "Discovery", "Sniffer", "Traceroute", "VulnScan"];
        let zm = (ca / tabs.len() as u32).max(80);
        
        for (i, label) in tabs.iter().enumerate() {
            let bu = lv + (i as u32 * zm);
            let is_active = i as u8 == self.netscan_tab;
            
            if is_active {
                framebuffer::fill_rect(bu, bn as u32, zm, bph, pcq);
                
                framebuffer::fill_rect(bu + 4, (bn + bph as i32 - 2) as u32, zm - 8, 2, ql);
            }
            
            let c = if is_active { ql } else { rg };
            
            let acy = label.len() as i32 * 8;
            let kd = bu as i32 + (zm as i32 - acy) / 2;
            self.draw_text_smooth(kd, bn + 7, label, c);
        }
        
        
        let cx = lv as i32 + 16;
        let mut u = bn + bph as i32 + 12;
        let bw = 20i32;
        let efn = ca.saturating_sub(32);
        
        match self.netscan_tab {
            0 => { 
                self.draw_text_smooth(cx, u, "Network Dashboard", ql);
                self.draw_text_smooth(cx + 1, u, "Network Dashboard", ql);
                u += bw + 8;
                
                
                let bfn = crate::virtio_net::is_initialized() || crate::drivers::net::aoh();
                let bdw = if bfn { 0xFF33DD66u32 } else { 0xFFDD3333u32 };
                let crc = if bfn { "Connected" } else { "Disconnected" };
                self.draw_text_smooth(cx, u, "Status:", ou);
                
                for ad in 0..8u32 {
                    for dx in 0..8u32 {
                        let lh = dx as i32 - 4;
                        let kf = ad as i32 - 4;
                        if lh * lh + kf * kf <= 16 {
                            framebuffer::cz((cx + 70 + dx as i32) as u32, (u + 4 + ad as i32) as u32, bdw);
                        }
                    }
                }
                self.draw_text_smooth(cx + 84, u, crc, bdw);
                u += bw;
                
                
                let driver = if crate::virtio_net::is_initialized() { "virtio-net" }
                    else if crate::drivers::net::aoh() { "RTL8169/e1000" }
                    else { "None" };
                self.draw_text_smooth(cx, u, "Driver:", ou);
                self.draw_text_smooth(cx + 70, u, driver, qe);
                u += bw;
                
                
                if let Some(mac) = crate::network::aqu() {
                    self.draw_text_smooth(cx, u, "MAC:", ou);
                    self.draw_text_smooth(cx + 70, u, &alloc::format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]), qe);
                }
                u += bw;
                
                
                if let Some((ip, mask, fz)) = crate::network::rd() {
                    self.draw_text_smooth(cx, u, "IP:", ou);
                    self.draw_text_smooth(cx + 70, u, &alloc::format!("{}", ip), qe);
                    u += bw;
                    self.draw_text_smooth(cx, u, "Subnet:", ou);
                    self.draw_text_smooth(cx + 70, u, &alloc::format!("{}", mask), qe);
                    u += bw;
                    if let Some(g) = fz {
                        self.draw_text_smooth(cx, u, "Gateway:", ou);
                        self.draw_text_smooth(cx + 70, u, &alloc::format!("{}", g), qe);
                        u += bw;
                    }
                } else {
                    self.draw_text_smooth(cx, u, "IPv4:", ou);
                    self.draw_text_smooth(cx + 70, u, "Waiting for DHCP...", rg);
                    u += bw;
                }
                
                u += 8;
                
                let stats = crate::network::get_stats();
                self.draw_text_smooth(cx, u, "Packets", rg);
                u += bw;
                self.draw_text_smooth(cx + 8, u, &alloc::format!("TX: {}  RX: {}", stats.packets_sent, stats.packets_received), qe);
                u += bw;
                self.draw_text_smooth(cx + 8, u, &alloc::format!("Bytes TX: {}  RX: {}", stats.bytes_sent, stats.bytes_received), qe);
                
                u += bw + 8;
                self.draw_text_smooth(cx, u, "Use tabs [1-6] or Left/Right to navigate", rg);
            },
            1 => { 
                self.draw_text_smooth(cx, u, "Port Scanner", ql);
                self.draw_text_smooth(cx + 1, u, "Port Scanner", ql);
                u += bw + 8;
                
                if let Some((_ip, _mask, fz)) = crate::network::rd() {
                    if let Some(g) = fz {
                        self.draw_text_smooth(cx, u, "Target:", ou);
                        self.draw_text_smooth(cx + 70, u, &alloc::format!("{} (gateway)", g), qe);
                        u += bw + 4;
                    }
                }
                
                self.draw_text_smooth(cx, u, "[S] Start Quick Scan", ou);
                u += bw + 8;
                
                
                if !window.content.is_empty() {
                    framebuffer::fill_rect(lv + 8, u as u32, efn, 1, ri);
                    u += 6;
                    self.draw_text_smooth(cx, u, "Results:", rg);
                    u += bw;
                    for line in window.content.iter() {
                        if u > wy + er as i32 - 20 { break; }
                        let c = if line.contains("OPEN") { 0xFF33DD66u32 } else { qe };
                        self.draw_text_smooth(cx + 8, u, line, c);
                        u += bw;
                    }
                }
            },
            2 => { 
                self.draw_text_smooth(cx, u, "Network Discovery", ql);
                self.draw_text_smooth(cx + 1, u, "Network Discovery", ql);
                u += bw + 8;
                
                self.draw_text_smooth(cx, u, "[D] Run ARP Sweep", ou);
                u += bw + 8;
                
                if !window.content.is_empty() {
                    framebuffer::fill_rect(lv + 8, u as u32, efn, 1, ri);
                    u += 6;
                    for line in window.content.iter() {
                        if u > wy + er as i32 - 20 { break; }
                        self.draw_text_smooth(cx + 8, u, line, qe);
                        u += bw;
                    }
                }
            },
            3 => { 
                self.draw_text_smooth(cx, u, "Packet Sniffer", ql);
                self.draw_text_smooth(cx + 1, u, "Packet Sniffer", ql);
                u += bw + 8;
                
                let fkw = crate::netscan::sniffer::btp();
                let status = if fkw { "Capturing..." } else { "Idle" };
                let dr = if fkw { 0xFF33DD66u32 } else { rg };
                self.draw_text_smooth(cx, u, "Status:", ou);
                self.draw_text_smooth(cx + 70, u, status, dr);
                u += bw;
                
                let pkt = if fkw { "[S] Stop Capture" } else { "[S] Start Capture" };
                self.draw_text_smooth(cx, u, pkt, ou);
                u += bw + 8;
                
                let (total_pkts, total_bytes, awl) = crate::netscan::sniffer::get_stats();
                self.draw_text_smooth(cx, u, "Captured:", ou);
                self.draw_text_smooth(cx + 80, u, &alloc::format!("{} packets", total_pkts), qe);
                u += bw;
                self.draw_text_smooth(cx, u, "Bytes:", ou);
                self.draw_text_smooth(cx + 80, u, &alloc::format!("{}", total_bytes), qe);
                u += bw;
                self.draw_text_smooth(cx, u, "Buffered:", ou);
                self.draw_text_smooth(cx + 80, u, &alloc::format!("{}", awl), qe);
            },
            4 => { 
                self.draw_text_smooth(cx, u, "Traceroute", ql);
                self.draw_text_smooth(cx + 1, u, "Traceroute", ql);
                u += bw + 8;
                
                if let Some((_ip, _mask, fz)) = crate::network::rd() {
                    if let Some(g) = fz {
                        self.draw_text_smooth(cx, u, "Target:", ou);
                        self.draw_text_smooth(cx + 70, u, &alloc::format!("{}", g), qe);
                        u += bw + 4;
                    }
                }
                
                self.draw_text_smooth(cx, u, "[T] Run Traceroute", ou);
                u += bw + 8;
                
                if !window.content.is_empty() {
                    framebuffer::fill_rect(lv + 8, u as u32, efn, 1, ri);
                    u += 6;
                    for line in window.content.iter() {
                        if u > wy + er as i32 - 20 { break; }
                        self.draw_text_smooth(cx + 8, u, line, qe);
                        u += bw;
                    }
                }
            },
            5 => { 
                self.draw_text_smooth(cx, u, "Vulnerability Scanner", ql);
                self.draw_text_smooth(cx + 1, u, "Vulnerability Scanner", ql);
                u += bw + 8;
                
                if let Some((_ip, _mask, fz)) = crate::network::rd() {
                    if let Some(g) = fz {
                        self.draw_text_smooth(cx, u, "Target:", ou);
                        self.draw_text_smooth(cx + 70, u, &alloc::format!("{}", g), qe);
                        u += bw + 4;
                    }
                }
                
                self.draw_text_smooth(cx, u, "[V] Run Vulnerability Scan", ou);
                u += bw + 8;
                
                if !window.content.is_empty() {
                    framebuffer::fill_rect(lv + 8, u as u32, efn, 1, ri);
                    u += 6;
                    for line in window.content.iter() {
                        if u > wy + er as i32 - 20 { break; }
                        let c = if line.contains("VULN") || line.contains("HIGH") { 0xFFDD3333u32 }
                            else if line.contains("WARN") || line.contains("MEDIUM") { 0xFFDDAA33u32 }
                            else { qe };
                        self.draw_text_smooth(cx + 8, u, line, c);
                        u += bw;
                    }
                }
            },
            _ => {}
        }
    }
    
    
    fn qtk(&mut self) {
        if let Some(window) = self.windows.iter_mut().find(|w| w.window_type == WindowType::Settings) {
            window.content.clear();
            window.content.push(String::from("=== Settings ==="));
            window.content.push(String::from(""));
            window.content.push(format!("Resolution: {}x{}", self.width, self.height));
            window.content.push(String::from("Theme: Dark Green"));
            window.content.push(String::from(""));
            window.content.push(String::from("--- Animations ---"));
            let dhs = if awb() { "ON " } else { "OFF" };
            let fhc = *GP_.lock();
            window.content.push(format!("[1] Animations: {}", dhs));
            window.content.push(format!("[2] Speed: {:.1}x", fhc));
            window.content.push(String::from(""));
            window.content.push(String::from("--- Accessibility ---"));
            let ads = if crate::accessibility::btq() { "ON " } else { "OFF" };
            window.content.push(format!("[5] High Contrast: {}", ads));
            window.content.push(format!("[6] Font Size: {}", crate::accessibility::cyn().label()));
            window.content.push(format!("[7] Cursor Size: {}", crate::accessibility::cyl().label()));
            window.content.push(format!("[8] Sticky Keys: {}", if crate::accessibility::bnc() { "ON" } else { "OFF" }));
            window.content.push(format!("[9] Mouse Speed: {}", crate::accessibility::cyq().label()));
            window.content.push(String::from(""));
            window.content.push(String::from("--- Other ---"));
            window.content.push(String::from("[3] File Associations"));
            window.content.push(String::from("[4] About System"));
        }
    }
    
    
    fn handle_fileassoc_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_};
        
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileAssociations) {
            let ikr = 4; 
            let dti = window.content.len().saturating_sub(2);
            let gfq = dti.saturating_sub(ikr);
            
            if key == T_ && window.selected_index > 0 {
                window.selected_index -= 1;
            } else if key == S_ && window.selected_index < gfq.saturating_sub(1) {
                window.selected_index += 1;
            } else if key == 0x0D || key == 0x0A {
                
                let idx = ikr + window.selected_index;
                if idx < dti {
                    
                    let line = &window.content[idx];
                    if let Some(ext_end) = line.find('|') {
                        let ext = line[1..ext_end].trim().trim_start_matches('.');
                        
                        use crate::file_assoc::{Program, set_program, cyr};
                        let current = cyr(&format!("test.{}", ext));
                        let next = match current {
                            Program::TextEditor => Program::ImageViewer,
                            Program::ImageViewer => Program::HexViewer,
                            Program::HexViewer => Program::Terminal,
                            Program::Terminal => Program::TextEditor,
                            _ => Program::TextEditor,
                        };
                        set_program(ext, next.clone());
                        
                        crate::serial_println!("[ASSOC] {} -> {}", ext, next.name());
                    }
                }
            }
        }
    }
    
    
    fn clear_terminal_suggestions(&mut self) {
        if self.terminal_suggestion_count > 0 {
            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                for _ in 0..self.terminal_suggestion_count {
                    window.content.pop();
                }
            }
            self.terminal_suggestion_count = 0;
        }
    }
    
    
    fn show_terminal_suggestions(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }
        let cch = self.input_buffer.as_str();
        let commands = crate::shell::AJS_;
        let matches: Vec<&str> = commands.iter().copied()
            .filter(|c| c.starts_with(cch) && *c != cch)
            .collect();
        if matches.is_empty() {
            return;
        }
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
            
            let display: Vec<&str> = matches.iter().copied().take(6).collect();
            let line = format!("  \x01M> {}", display.join("  "));
            window.content.push(line);
            self.terminal_suggestion_count = 1;
            
            if matches.len() > 6 {
                window.content.push(format!("    \x01M... +{} more", matches.len() - 6));
                self.terminal_suggestion_count = 2;
            }
        }
    }
    
    
    fn aya(asi: &str) -> String {
        let fm = crate::rtc::aou();
        let cwd = crate::ramfs::bh(|fs| {
            let aa = fs.pwd();
            String::from(aa)
        });
        let lfk = if cwd == "/" { String::from("~") } else { cwd };
        format!("\x01B[{:02}:{:02}:{:02}] \x01Rroot\x01M@trustos\x01M:\x01B{}\x01M$ \x01G{}", fm.hour, fm.minute, fm.second, lfk, asi)
    }
    
    
    fn handle_terminal_key(&mut self, key: u8) {
        use crate::keyboard::{AM_, AO_};
        
        self.clear_terminal_suggestions();
        
        
        if key == AM_ || key == AO_ {
            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                let line_height = 16usize;
                let chp = (window.height as usize).saturating_sub(J_() as usize + 16);
                let oe = if line_height > 0 { chp / line_height } else { 1 };
                let aab = window.content.len().saturating_sub(oe);
                if key == AM_ {
                    window.scroll_offset = window.scroll_offset.saturating_sub(oe);
                } else {
                    window.scroll_offset = (window.scroll_offset + oe).min(aab);
                }
            }
            return;
        }
        
        if key == 0x08 { 
            if !self.input_buffer.is_empty() {
                self.input_buffer.pop();
                if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                    if let Some(last) = window.content.last_mut() {
                        *last = Self::aya(&format!("{}_", self.input_buffer));
                    }
                }
            }
        } else if key == 0x09 { 
            let cch = self.input_buffer.clone();
            if !cch.is_empty() {
                
                if let Some(space_pos) = cch.rfind(' ') {
                    
                    let hyh = &cch[space_pos + 1..];
                    if !hyh.is_empty() {
                        
                        let mut dpm: Vec<String> = Vec::new();
                        if let Ok(entries) = crate::ramfs::bh(|fs| fs.ls(Some("/"))) {
                            for (name, wf, bek) in entries.iter() {
                                let duw: String = name.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                                let nrr: String = hyh.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                                if duw.starts_with(&nrr) {
                                    let asi = if *wf == crate::ramfs::FileType::Directory { "/" } else { "" };
                                    dpm.push(format!("{}{}", name, asi));
                                }
                            }
                        }
                        if dpm.len() == 1 {
                            let kqp = &cch[..=space_pos];
                            self.input_buffer = format!("{}{}", kqp, dpm[0]);
                            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                                if let Some(last) = window.content.last_mut() {
                                    *last = Self::aya(&format!("{}_", self.input_buffer));
                                }
                            }
                        } else if dpm.len() > 1 {
                            let ggp: String = dpm.join("  ");
                            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                                window.content.push(ggp);
                                window.content.push(Self::aya(&format!("{}_", self.input_buffer)));
                            }
                        }
                    }
                } else {
                    
                    let commands = crate::shell::AJS_;
                    let nrs = cch.as_str();
                    let matches: Vec<&str> = commands.iter().copied().filter(|c| c.starts_with(nrs)).collect();
                    if matches.len() == 1 {
                        self.input_buffer = String::from(matches[0]);
                        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                            if let Some(last) = window.content.last_mut() {
                                *last = Self::aya(&format!("{}_", self.input_buffer));
                            }
                        }
                    } else if matches.len() > 1 {
                        let ggp: String = matches.join("  ");
                        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                            window.content.push(ggp);
                            window.content.push(Self::aya(&format!("{}_", self.input_buffer)));
                        }
                    }
                }
            }
        } else if key == 0xF0 { 
            if !self.command_history.is_empty() {
                match self.history_index {
                    None => {
                        
                        self.saved_input = self.input_buffer.clone();
                        let idx = self.command_history.len() - 1;
                        self.history_index = Some(idx);
                        self.input_buffer = self.command_history[idx].clone();
                    }
                    Some(i) if i > 0 => {
                        let idx = i - 1;
                        self.history_index = Some(idx);
                        self.input_buffer = self.command_history[idx].clone();
                    }
                    _ => {} 
                }
                if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                    if let Some(last) = window.content.last_mut() {
                        *last = Self::aya(&format!("{}_", self.input_buffer));
                    }
                }
            }
        } else if key == 0xF1 { 
            if let Some(i) = self.history_index {
                if i + 1 < self.command_history.len() {
                    let idx = i + 1;
                    self.history_index = Some(idx);
                    self.input_buffer = self.command_history[idx].clone();
                } else {
                    
                    self.history_index = None;
                    self.input_buffer = self.saved_input.clone();
                }
                if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                    if let Some(last) = window.content.last_mut() {
                        *last = Self::aya(&format!("{}_", self.input_buffer));
                    }
                }
            }
        } else if key == 0x0D || key == 0x0A { 
            let cmd = self.input_buffer.clone();
            self.input_buffer.clear();
            
            if !cmd.trim().is_empty() {
                let dnx = self.command_history.last().map(|h| h == &cmd).unwrap_or(false);
                if !dnx {
                    self.command_history.push(cmd.clone());
                }
            }
            self.history_index = None;
            self.saved_input.clear();
            
            let output = Self::lsa(&cmd);
            
            
            let hmf = cmd.trim();
            if hmf.starts_with("play ") {
                let db = hmf.strip_prefix("play ").unwrap_or("").trim();
                match db {
                    "u2" | "untitled2" | "lofi" | "untitled" => {
                        
                        let dbj = self.width.saturating_sub(320) as i32;
                        let dbk = self.height.saturating_sub(V_() + 600) as i32;
                        let sa = self.create_window("Music Player", dbj, dbk.max(20), 320, 580, WindowType::MusicPlayer);
                        if let Some(mp_state) = self.music_player_states.get_mut(&sa) {
                            mp_state.play_track(0);
                        }
                    },
                    _ => {},
                }
            }
            
            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                
                if cmd.trim() == "clear" {
                    window.content.clear();
                    window.content.push(Self::aya("_"));
                    window.scroll_offset = 0;
                } else {
                    
                    if window.content.last().map(|j| j.contains("$ ")).unwrap_or(false) {
                        window.content.pop();
                    }
                    
                    window.content.push(Self::aya(&cmd));
                    
                    for line in output {
                        window.content.push(line);
                    }
                    
                    
                    window.content.push(Self::aya("_"));
                    
                    
                    let line_height = 16usize;
                    let chp = (window.height as usize).saturating_sub(J_() as usize + 16);
                    let oe = if line_height > 0 { chp / line_height } else { 1 };
                    if window.content.len() > oe {
                        window.scroll_offset = window.content.len() - oe;
                    } else {
                        window.scroll_offset = 0;
                    }
                }
            }
        } else if key >= 0x20 && key < 0x7F {
            self.input_buffer.push(key as char);
            
            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                if let Some(last) = window.content.last_mut() {
                    *last = Self::aya(&format!("{}_", self.input_buffer));
                }
            }
        }
        
        
        self.show_terminal_suggestions();
    }
    
    
    fn lsa(cmd: &str) -> Vec<String> {
        let mut output = Vec::new();
        let cmd = cmd.trim();
        
        
        crate::serial_println!("[TERM] Executing command: '{}' len={}", cmd, cmd.len());
        
        if cmd.is_empty() {
            return output;
        }
        
        match cmd {
            "help" => {
                output.push(String::from("\x01HTrustOS Terminal \x01M- Available Commands"));
                output.push(String::from(""));
                
                output.push(String::from("\x01Y[File System]"));
                output.push(String::from("  \x01Gls \x01B[dir]      \x01WList directory contents"));
                output.push(String::from("  \x01Gcd \x01B<dir>      \x01WChange directory"));
                output.push(String::from("  \x01Gpwd            \x01WPrint working directory"));
                output.push(String::from("  \x01Gcat \x01B<file>    \x01WShow file contents"));
                output.push(String::from("  \x01Gmkdir \x01B<name>  \x01WCreate directory"));
                output.push(String::from("  \x01Gtouch \x01B<name>  \x01WCreate empty file"));
                output.push(String::from("  \x01Grm \x01B<file>     \x01WRemove file"));
                output.push(String::from("  \x01Gcp \x01B<src> <dst>\x01WCopy file"));
                output.push(String::from("  \x01Gmv \x01B<src> <dst>\x01WMove/rename file"));
                output.push(String::from("  \x01Gtree \x01B[path]   \x01WDirectory tree"));
                output.push(String::from("  \x01Gfind \x01B<p> <n>  \x01WSearch files by name"));
                output.push(String::from("  \x01Gstat \x01B<file>   \x01WFile metadata"));
                output.push(String::from(""));
                
                output.push(String::from("\x01Y[System]"));
                output.push(String::from("  \x01Gdate           \x01WCurrent date and time"));
                output.push(String::from("  \x01Guname          \x01WSystem information"));
                output.push(String::from("  \x01Gfree           \x01WMemory usage"));
                output.push(String::from("  \x01Gps             \x01WList processes"));
                output.push(String::from("  \x01Guptime         \x01WSystem uptime"));
                output.push(String::from("  \x01Gdf             \x01WDisk usage"));
                output.push(String::from("  \x01Gwhoami         \x01WCurrent user"));
                output.push(String::from("  \x01Ghostname       \x01WSystem hostname"));
                output.push(String::from("  \x01Gneofetch       \x01WSystem info banner"));
                output.push(String::from("  \x01Ghistory        \x01WCommand history"));
                output.push(String::from("  \x01Genv            \x01WEnvironment variables"));
                output.push(String::from("  \x01Gdmesg          \x01WKernel log messages"));
                output.push(String::from(""));
                
                output.push(String::from("\x01Y[Network]"));
                output.push(String::from("  \x01Gnet            \x01WNetwork interface status"));
                output.push(String::from("  \x01Gifconfig       \x01WNetwork configuration"));
                output.push(String::from("  \x01Gping \x01B<host>   \x01WICMP echo test"));
                output.push(String::from("  \x01Gcurl \x01B<url>    \x01WHTTP client"));
                output.push(String::from("  \x01Gnslookup \x01B<h>  \x01WDNS lookup"));
                output.push(String::from("  \x01Gnetstat        \x01WActive connections"));
                output.push(String::from(""));
                
                output.push(String::from("\x01Y[Graphics & Demos]"));
                output.push(String::from("  \x01Gshader \x01B<name>  \x01WGPU shader (plasma/fire/tunnel...)"));
                output.push(String::from("  \x01Gmatrix3d       \x01W3D Matrix tunnel"));
                output.push(String::from("  \x01Gshowcase3d     \x01W3D cinematic demo"));
                output.push(String::from("  \x01Gfilled3d       \x01WFilled 3D test"));
                output.push(String::from("  \x01Gchess          \x01WChess game vs AI"));
                output.push(String::from("  \x01Gchess3d        \x01W3D chess (Matrix style)"));
                output.push(String::from("  \x01Ggameboy        \x01WGame Boy emulator"));
                output.push(String::from("  \x01Gmatrix         \x01WMatrix rain animation"));
                output.push(String::from("  \x01Gneofetch       \x01WASCII system info"));
                output.push(String::from(""));
                
                output.push(String::from("\x01Y[Audio]"));
                output.push(String::from("  \x01Gplay \x01B<track>  \x01WPlay music (u2, lofi)"));
                output.push(String::from("  \x01Gsynth \x01B<cmd>   \x01WPolyphonic synthesizer"));
                output.push(String::from("  \x01Gdaw \x01B<cmd>     \x01WDigital audio workstation"));
                output.push(String::from("  \x01Gbeep \x01B[hz] [ms]\x01WPlay a tone"));
                output.push(String::from(""));
                
                output.push(String::from("\x01Y[AI / Jarvis]"));
                output.push(String::from("  \x01Gjarvis \x01B<cmd>  \x01WAI assistant (chat/brain/hw)"));
                output.push(String::from("  \x01Gj \x01B<query>     \x01WJarvis shortcut"));
                output.push(String::from(""));
                
                output.push(String::from("\x01Y[Text Processing]"));
                output.push(String::from("  \x01Ggrep \x01B<pat> <f>\x01WSearch pattern in file"));
                output.push(String::from("  \x01Gsort \x01B<file>   \x01WSort lines"));
                output.push(String::from("  \x01Gdiff \x01B<a> <b>  \x01WCompare files"));
                output.push(String::from("  \x01Ghexdump \x01B<f>   \x01WHex dump file"));
                output.push(String::from(""));
                
                output.push(String::from("\x01Y[Shell]"));
                output.push(String::from("  \x01Ghelp           \x01WShow this help"));
                output.push(String::from("  \x01Gecho \x01B<text>   \x01WPrint text"));
                output.push(String::from("  \x01Gclear          \x01WClear terminal"));
                output.push(String::from("  \x01Gexit           \x01WClose terminal"));
                output.push(String::from(""));
                output.push(String::from("\x01M  All boot shell commands also available (220+)"));
            },
            
            "matrix3d" | "tunnel" | "holomatrix" | "3d" => {
                output.push(String::from("✓ Matrix Tunnel 3D - ESC to exit"));
                
                
                let fb = crate::framebuffer::fyq();
                let width = crate::framebuffer::width();
                let height = crate::framebuffer::height();
                
                
                crate::gpu_emu::init(fb, width, height);
                if let Some(shader_fn) = crate::gpu_emu::fyx("tunnel") {
                    crate::gpu_emu::set_shader(shader_fn);
                }
                
                let mut frames = 0u32;
                loop {
                    if let Some(key) = crate::keyboard::kr() {
                        if key == 27 { break; }
                    }
                    
                    #[cfg(target_arch = "x86_64")]
                    crate::gpu_emu::ftc();
                    #[cfg(not(target_arch = "x86_64"))]
                    crate::gpu_emu::draw();
                    
                    crate::gpu_emu::tick(16);
                    frames += 1;
                    
                    if frames % 60 == 0 {
                        crate::framebuffer::draw_text("MATRIX 3D TUNNEL | ESC=exit", 10, 10, 0xFF00FF00);
                    }
                }
                output.push(format!("Tunnel ended ({} frames)", frames));
            },
            "ls" => {
                let cwd = crate::ramfs::bh(|fs| String::from(fs.pwd()));
                output.push(format!("\x01MDirectory: \x01B{}", cwd));
                if let Ok(entries) = crate::ramfs::bh(|fs| fs.ls(None)) {
                    for (name, wf, size) in entries.iter().take(20) {
                        let icon = if *wf == crate::ramfs::FileType::Directory { "\x01B" } else { "\x01M" };
                        let ws = if *wf == crate::ramfs::FileType::Directory { "/" } else { "" };
                        output.push(format!("  {}{}{}  \x01M{} bytes", icon, name, ws, size));
                    }
                    if entries.is_empty() {
                        output.push(String::from("\x01M  (empty directory)"));
                    }
                }
            },
            "ls /" => {
                output.push(String::from("\x01MDirectory: \x01B/"));
                if let Ok(entries) = crate::ramfs::bh(|fs| fs.ls(Some("/"))) {
                    for (name, wf, size) in entries.iter().take(20) {
                        let icon = if *wf == crate::ramfs::FileType::Directory { "\x01B" } else { "\x01M" };
                        let ws = if *wf == crate::ramfs::FileType::Directory { "/" } else { "" };
                        output.push(format!("  {}{}{}  \x01M{} bytes", icon, name, ws, size));
                    }
                    if entries.is_empty() {
                        output.push(String::from("\x01M  (empty directory)"));
                    }
                }
            },
            _ if cmd.starts_with("ls ") => {
                let path = &cmd[3..];
                output.push(format!("\x01MDirectory: \x01B{}", path));
                if let Ok(entries) = crate::ramfs::bh(|fs| fs.ls(Some(path))) {
                    for (name, wf, size) in entries.iter().take(20) {
                        let icon = if *wf == crate::ramfs::FileType::Directory { "\x01B" } else { "\x01M" };
                        let ws = if *wf == crate::ramfs::FileType::Directory { "/" } else { "" };
                        output.push(format!("  {}{}{}  \x01M{} bytes", icon, name, ws, size));
                    }
                    if entries.is_empty() {
                        output.push(String::from("\x01M  (empty directory)"));
                    }
                } else {
                    output.push(format!("\x01Rls: cannot access '{}': No such file or directory", path));
                }
            },
            "pwd" => {
                let cwd = crate::ramfs::bh(|fs| String::from(fs.pwd()));
                output.push(format!("\x01B{}", cwd));
            },
            "clear" => {
                
            },
            "date" | "time" => {
                let fm = crate::rtc::aou();
                output.push(format!("\x01B{:04}-{:02}-{:02} \x01W{:02}:{:02}:{:02}", 
                    fm.year, fm.month, fm.day, fm.hour, fm.minute, fm.second));
            },
            "uname" | "uname -a" | "version" => {
                output.push(String::from("\x01GTrustOS \x01W0.1.1 \x01Bx86_64 \x01MRust Kernel"));
                output.push(format!("\x01MHeap: \x01W{} MB", crate::memory::atz() / 1024 / 1024));
            },
            "neofetch" => {
                output.push(String::from("\x01G  _____               _    ___  ___"));
                output.push(String::from("\x01G |_   _| __ _   _ ___| |_ / _ \\/ __|"));
                output.push(String::from("\x01G   | || '__| | | / __| __| | | \\__ \\"));
                output.push(String::from("\x01G   | || |  | |_| \\__ \\ |_| |_| |__) |"));
                output.push(String::from("\x01G   |_||_|   \\__,_|___/\\__|\\___/|___/"));
                output.push(String::from(""));
                output.push(String::from("\x01BOS\x01M:      \x01WTrustOS 0.1.1"));
                output.push(String::from("\x01BKernel\x01M:  \x01Wtrustos_kernel"));
                output.push(String::from("\x01BArch\x01M:    \x01Wx86_64"));
                output.push(format!("\x01BUptime\x01M:  \x01W{}m {}s", crate::logger::eg() / 100 / 60, (crate::logger::eg() / 100) % 60));
                output.push(format!("\x01BMemory\x01M:  \x01W{} MB", crate::memory::atz() / 1024 / 1024));
                output.push(format!("\x01BShell\x01M:   \x01Wtrustsh"));
                output.push(format!("\x01BDisplay\x01M: \x01W{}x{}", crate::framebuffer::width(), crate::framebuffer::height()));
            },
            "whoami" | "user" | "users" | "id" => {
                output.push(String::from("\x01Groot"));
            },
            "hostname" => {
                output.push(String::from("\x01Gtrustos"));
            },
            "history" => {
                
                let axk = crate::desktop::S.lock().command_history.clone();
                if axk.is_empty() {
                    output.push(String::from("\x01M  (no history yet)"));
                } else {
                    for (i, entry) in axk.iter().enumerate() {
                        output.push(format!("\x01M  {}  {}", i + 1, entry));
                    }
                }
            },
            "free" | "mem" => {
                let mku = crate::memory::atz() / 1024 / 1024;
                output.push(String::from("\x01YMemory Usage:"));
                output.push(format!("  \x01BHeap Size: \x01W{} MB", mku));
                output.push(String::from("  \x01BKernel:   \x01GActive"));
            },
            "net" | "ifconfig" | "ip" | "ipconfig" => {
                output.push(String::from("\x01YNetwork Status:"));
                if crate::network::sw() {
                    if let Some((mac, ip, _state)) = crate::network::cyp() {
                        output.push(format!("  \x01BMAC: \x01W{}", mac));
                        if let Some(ip) = ip {
                            output.push(format!("  \x01BIP:  \x01W{}", ip));
                        }
                        output.push(String::from("  \x01BStatus: \x01GConnected"));
                    }
                } else {
                    output.push(String::from("  \x01BStatus: \x01RNo network"));
                }
            },
            _ if cmd.starts_with("cat ") => {
                let filename = &cmd[4..].trim();
                if let Ok(content) = crate::ramfs::bh(|fs| fs.read_file(filename).map(|d| d.to_vec())) {
                    if let Ok(text) = core::str::from_utf8(&content) {
                        for line in text.lines().take(20) {
                            output.push(String::from(line));
                        }
                    } else {
                        output.push(format!("cat: {}: binary file", filename));
                    }
                } else {
                    output.push(format!("cat: {}: No such file", filename));
                }
            },
            _ if cmd.starts_with("echo ") => {
                output.push(String::from(&cmd[5..]));
            },
            "cd" => {
                
                let _ = crate::ramfs::bh(|fs| fs.cd("/"));
            },
            _ if cmd.starts_with("cd ") => {
                let path = &cmd[3..].trim();
                match crate::ramfs::bh(|fs| fs.cd(path)) {
                    Ok(()) => {
                        let cwd = crate::ramfs::bh(|fs| String::from(fs.pwd()));
                        output.push(format!("\x01B{}", cwd));
                    },
                    Err(e) => output.push(format!("\x01Rcd: {}: {}", path, e.as_str())),
                }
            },
            _ if cmd.starts_with("mkdir ") => {
                let path = cmd[6..].trim();
                match crate::ramfs::bh(|fs| fs.mkdir(path)) {
                    Ok(()) => output.push(format!("\x01Gmkdir: \x01Wcreated '\x01B{}\x01W'", path)),
                    Err(e) => output.push(format!("\x01Rmkdir: {}: {}", path, e.as_str())),
                }
            },
            _ if cmd.starts_with("touch ") => {
                let path = cmd[6..].trim();
                match crate::ramfs::bh(|fs| fs.touch(path)) {
                    Ok(()) => output.push(format!("\x01Gtouch: \x01Wcreated '\x01B{}\x01W'", path)),
                    Err(e) => output.push(format!("\x01Rtouch: {}: {}", path, e.as_str())),
                }
            },
            _ if cmd.starts_with("rm ") || cmd.starts_with("del ") => {
                let path = if cmd.starts_with("rm ") { cmd[3..].trim() } else { cmd[4..].trim() };
                match crate::ramfs::bh(|fs| fs.rm(path)) {
                    Ok(()) => output.push(format!("\x01Grm: \x01Wremoved '\x01B{}\x01W'", path)),
                    Err(e) => output.push(format!("\x01Rrm: {}: {}", path, e.as_str())),
                }
            },
            "shader" | "vgpu" => {
                output.push(String::from("╔═══════════════════════════════════════╗"));
                output.push(String::from("║     Virtual GPU - Shader Demo         ║"));
                output.push(String::from("╠═══════════════════════════════════════╣"));
                output.push(String::from("║ shader plasma    - Plasma waves       ║"));
                output.push(String::from("║ shader fire      - Fire effect        ║"));
                output.push(String::from("║ shader mandelbrot- Fractal zoom       ║"));
                output.push(String::from("║ shader matrix    - Matrix rain        ║"));
                output.push(String::from("║ shader tunnel    - 3D HOLOMATRIX      ║"));
                output.push(String::from("║ shader shapes    - 3D OBJECTS         ║"));
                output.push(String::from("║ shader parallax  - Depth layers       ║"));
                output.push(String::from("║ shader gradient  - Test gradient      ║"));
                output.push(String::from("╚═══════════════════════════════════════╝"));
                output.push(String::from("Press ESC to exit shader demo"));
            },
            _ if cmd.starts_with("shader ") => {
                let bov = cmd.trim_start_matches("shader ").trim();
                if let Some(shader_fn) = crate::gpu_emu::fyx(bov) {
                    output.push(format!("✓ Starting shader: {} (ESC to exit)", bov));
                    
                    
                    let fb = crate::framebuffer::fyq();
                    let width = crate::framebuffer::width();
                    let height = crate::framebuffer::height();
                    
                    
                    crate::gpu_emu::init(fb, width, height);
                    crate::gpu_emu::set_shader(shader_fn);
                    
                    
                    let mut frames = 0u32;
                    
                    loop {
                        
                        if let Some(key) = crate::keyboard::kr() {
                            if key == 27 { break; }
                        }
                        
                        
                        #[cfg(target_arch = "x86_64")]
                        crate::gpu_emu::ftc();
                        #[cfg(not(target_arch = "x86_64"))]
                        crate::gpu_emu::draw();
                        
                        
                        crate::gpu_emu::tick(16);
                        frames += 1;
                        
                        
                        if frames % 60 == 0 {
                            crate::framebuffer::draw_text(&format!("FPS: ~60 | {} | ESC=exit", bov), 10, 10, 0xFFFFFFFF);
                        }
                    }
                    
                    output.push(format!("Shader ended ({} frames)", frames));
                } else {
                    output.push(format!("Unknown shader: {}", bov));
                    output.push(String::from("Available: plasma, fire, mandelbrot, matrix, tunnel, parallax, gradient"));
                }
            },
            "ps" | "procs" | "top" => {
                output.push(String::from("\x01BPID  \x01BSTATE    \x01BNAME"));
                output.push(String::from("  \x01W1  \x01GRunning  \x01Winit"));
                output.push(String::from("  \x01W2  \x01GRunning  \x01Wdesktop"));
                output.push(String::from("  \x01W3  \x01GRunning  \x01Wterminal"));
            },
            "uptime" => {
                let gx = crate::logger::eg();
                let im = gx / 100;
                let acf = im / 60;
                output.push(format!("\x01BUptime: \x01W{}m {}s", acf, im % 60));
            },
            "df" | "lsblk" => {
                output.push(String::from("\x01BFilesystem      Size  Used  Avail Use%"));
                output.push(String::from("\x01Wramfs           32M   1M    31M   3%"));
            },
            "showcase3d" | "demo3d" => {
                output.push(String::from("\u{2713} Showcase 3D Cinematic - ESC to skip scenes"));
                drop(output);
                crate::shell::desktop::hme();
                return Vec::new();
            },
            "filled3d" => {
                output.push(String::from("\u{2713} Filled 3D Test - ESC to exit"));
                drop(output);
                crate::shell::desktop::hlw();
                return Vec::new();
            },
            "exit" | "quit" => {
                output.push(String::from("\x01MUse the X button to close the terminal"));
            },
            "desktop" | "gui" | "mobile" => {
                output.push(String::from("\x01MAlready in desktop mode."));
            },
            "chess" => {
                output.push(String::from("\x01G\u{265A} TrustChess \x01M— Opening chess window..."));
                output.push(String::from("\x01MPlay vs AI (Black). Arrow keys, Enter, Esc."));
            },
            "chess3d" => {
                output.push(String::from("\x01G\u{265A} TrustChess 3D \x01M— Opening 3D chess window..."));
                output.push(String::from("\x01MWASD:Camera  ZX:Zoom  O:Auto-rotate  Click:Move"));
            },
            "gameboy" | "gb" => {
                output.push(String::from("\x01G\u{1F3AE} Game Boy \x01M— Opening Game Boy window..."));
                output.push(String::from("\x01MWASD:D-Pad X/Space:A Z:B C:Select Enter:Start"));
            },
            _ if cmd.starts_with("play ") || cmd == "play" => {
                let db = cmd.strip_prefix("play ").unwrap_or("").trim();
                if db.is_empty() {
                    output.push(String::from("\x01Y\u{266B} Usage: \x01Gplay u2"));
                    output.push(String::from("\x01MTracks: u2, untitled2, lofi"));
                } else {
                    match db {
                        "u2" | "untitled2" | "lofi" | "untitled" => {
                            output.push(String::from("\x01G\u{266B} Playing Untitled (2) — Lo-Fi"));
                            output.push(String::from("\x01MOpening Music Player widget..."));
                            
                        },
                        _ => {
                            output.push(format!("\x01RTrack not found: \x01W{}", db));
                            output.push(String::from("\x01MAvailable: u2, untitled2, lofi"));
                        },
                    }
                }
            },
            _ if cmd.starts_with("j ") || cmd.starts_with("jarvis ") || cmd == "j" || cmd == "jarvis" => {
                
                if VQ_.load(core::sync::atomic::Ordering::SeqCst) {
                    output.push(String::from("\x01Y[Jarvis] \x01MStill thinking... please wait."));
                } else {
                    VQ_.store(true, core::sync::atomic::Ordering::SeqCst);
                    {
                        let mut pending = AZQ_.lock();
                        *pending = Some(String::from(cmd));
                    }
                    crate::thread::dzu("jarvis-bg", muv, 0);
                    output.push(String::from("\x01Y[Jarvis] \x01M\u{1F4AD} Thinking..."));
                }
            },
            _ => {
                
                
                crate::shell::fcj(); 
                crate::shell::DL_.store(true, core::sync::atomic::Ordering::SeqCst);
                crate::shell::aav(cmd);
                crate::shell::DL_.store(false, core::sync::atomic::Ordering::SeqCst);
                let captured = crate::shell::fcj();
                if !captured.is_empty() {
                    for line in captured.lines() {
                        output.push(String::from(line));
                    }
                }
            },
        }
        
        output
    }
    
    
    pub fn handle_move(&mut self, x: i32, y: i32) {
        self.cursor_x = x.clamp(0, self.width as i32 - 1);
        self.cursor_y = y.clamp(0, self.height as i32 - 1);
        
        
        if self.drag_state.is_some() {
            self.update_drag(x, y);
        }
        
        for w in &mut self.windows {
            
            if w.dragging && !w.maximized {
                w.x = (x - w.drag_offset_x).max(0).min(self.width as i32 - 50);
                w.y = (y - w.drag_offset_y).max(0).min(self.height as i32 - V_() as i32 - J_() as i32);
                
                
                let bze = 16i32;
                let dy = self.width as i32;
                let dw = (self.height - V_()) as i32;
                let kh = dw / 2;
                
                if x <= bze && y <= bze + kh / 4 {
                    self.snap_preview = Some(SnapDir::TopLeft);
                } else if x <= bze && y >= dw - kh / 4 {
                    self.snap_preview = Some(SnapDir::BottomLeft);
                } else if x <= bze {
                    self.snap_preview = Some(SnapDir::Left);
                } else if x >= dy - bze && y <= bze + kh / 4 {
                    self.snap_preview = Some(SnapDir::TopRight);
                } else if x >= dy - bze && y >= dw - kh / 4 {
                    self.snap_preview = Some(SnapDir::BottomRight);
                } else if x >= dy - bze {
                    self.snap_preview = Some(SnapDir::Right);
                } else {
                    self.snap_preview = None;
                }
            }
            
            
            if w.resizing != ResizeEdge::None {
                let dx = x - w.drag_offset_x;
                let ad = y - w.drag_offset_y;
                
                
                match w.resizing {
                    ResizeEdge::Right | ResizeEdge::BottomRight | ResizeEdge::TopRight => {
                        let aym = (w.width as i32 + dx).max(w.min_width as i32) as u32;
                        w.width = aym.min(self.width - w.x as u32);
                        w.drag_offset_x = x;
                    }
                    _ => {}
                }
                
                
                match w.resizing {
                    ResizeEdge::Left | ResizeEdge::BottomLeft | ResizeEdge::TopLeft => {
                        let aym = (w.width as i32 - dx).max(w.min_width as i32) as u32;
                        if aym != w.width as u32 {
                            w.x += (w.width as i32 - aym as i32);
                            w.width = aym;
                        }
                        w.drag_offset_x = x;
                    }
                    _ => {}
                }
                
                
                match w.resizing {
                    ResizeEdge::Bottom | ResizeEdge::BottomRight | ResizeEdge::BottomLeft => {
                        let ayk = (w.height as i32 + ad).max(w.min_height as i32) as u32;
                        w.height = ayk.min(self.height - V_() - w.y as u32);
                        w.drag_offset_y = y;
                    }
                    _ => {}
                }
                
                
                match w.resizing {
                    ResizeEdge::Top | ResizeEdge::TopLeft | ResizeEdge::TopRight => {
                        let ayk = (w.height as i32 - ad).max(w.min_height as i32) as u32;
                        if ayk != w.height as u32 {
                            w.y += (w.height as i32 - ayk as i32);
                            w.height = ayk;
                        }
                        w.drag_offset_y = y;
                    }
                    _ => {}
                }
            }
        }
        
        
        let ghz: Option<(u32, i32, i32, u32, u32)> = self.windows.iter()
            .find(|w| w.focused && !w.minimized && w.window_type == WindowType::ModelEditor)
            .map(|w| (w.id, w.x, w.y, w.width, w.height));
        if let Some((fr, wx, wy, ca, er)) = ghz {
            let vx = x - wx;
            let vy = y - wy - J_() as i32;
            let bt = ca as usize;
            let ex = er.saturating_sub(J_()) as usize;
            if let Some(state) = self.model_editor_states.get_mut(&fr) {
                state.handle_mouse_move(vx, vy, bt, ex);
            }
        }
        
        
        let kkc: Option<u32> = self.windows.iter()
            .find(|w| w.focused && !w.minimized && w.window_type == WindowType::Chess)
            .map(|w| w.id);
        if let Some(fr) = kkc {
            if let Some(chess) = self.chess_states.get_mut(&fr) {
                if chess.drag_from.is_some() {
                    chess.update_drag_position(x, y);
                }
            }
        }
        
        
        let fll: Option<(u32, i32, i32)> = self.windows.iter()
            .find(|w| w.focused && !w.minimized && w.window_type == WindowType::Chess3D)
            .map(|w| (w.id, w.x, w.y));
        if let Some((fr, wx, wy)) = fll {
            if let Some(state) = self.chess3d_states.get_mut(&fr) {
                let sk = x - wx;
                let qn = y - wy - J_() as i32;
                state.handle_mouse_move(sk, qn);
            }
        }
    }
    
    
    pub fn handle_scroll(&mut self, mk: i8) {
        self.windows_dirty = true;
        
        let ghz = self.windows.iter().rev().find(|w| w.focused && !w.minimized && w.window_type == WindowType::ModelEditor).map(|w| w.id);
        if let Some(fr) = ghz {
            if let Some(state) = self.model_editor_states.get_mut(&fr) {
                state.handle_scroll(mk);
            }
            return;
        }
        
        let fll = self.windows.iter().rev().find(|w| w.focused && !w.minimized && w.window_type == WindowType::Chess3D).map(|w| w.id);
        if let Some(fr) = fll {
            if let Some(state) = self.chess3d_states.get_mut(&fr) {
                state.handle_scroll(mk);
            }
            return;
        }
        
        if let Some(window) = self.windows.iter_mut().rev().find(|w| w.focused && !w.minimized) {
            match window.window_type {
                WindowType::FileManager | WindowType::TextEditor | WindowType::HexViewer | 
                WindowType::FileAssociations | WindowType::Terminal => {
                    let aab = if window.content.len() > 10 {
                        window.content.len() - 10
                    } else {
                        0
                    };
                    
                    if mk > 0 {
                        
                        if window.scroll_offset > 0 {
                            window.scroll_offset = window.scroll_offset.saturating_sub(3);
                        }
                    } else if mk < 0 {
                        
                        window.scroll_offset = (window.scroll_offset + 3).min(aab);
                    }
                },
                _ => {}
            }
        }
    }
    
    
    
    pub fn process_touch_input(&mut self) {
        
        self.gesture_buffer.clear();
        self.gesture_recognizer.process_all(&mut self.gesture_buffer);
        
        
        if !self.gesture_buffer.is_empty() {
            self.touch_mode = true; 
        }
        
        
        let mut axf: [(u8, i32, i32, i32, i32, i32); 8] = [(0, 0, 0, 0, 0, 0); 8];
        let mut aqq = 0usize;
        
        for gesture in self.gesture_buffer.iter() {
            if aqq >= 8 { break; }
            
            match gesture {
                crate::gesture::GestureEvent::Tap { x, y } => {
                    axf[aqq] = (1, *x, *y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::DoubleTap { x, y } => {
                    axf[aqq] = (2, *x, *y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::LongPress { x, y } => {
                    axf[aqq] = (3, *x, *y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Swipe { direction, start_x, start_y, awy, doq, .. } => {
                    let fsf = match direction {
                        crate::gesture::SwipeDirection::Left => 0,
                        crate::gesture::SwipeDirection::Right => 1,
                        crate::gesture::SwipeDirection::Up => 2,
                        crate::gesture::SwipeDirection::Down => 3,
                    };
                    axf[aqq] = (4, *start_x, *start_y, *awy, *doq, fsf);
                }
                crate::gesture::GestureEvent::EdgeSwipe { origin, progress } => {
                    let nnx = match origin {
                        crate::gesture::EdgeOrigin::Bottom => 0,
                        crate::gesture::EdgeOrigin::Top => 1,
                        crate::gesture::EdgeOrigin::Left => 2,
                        crate::gesture::EdgeOrigin::Right => 3,
                    };
                    axf[aqq] = (5, nnx, *progress, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Pinch { center_x, center_y, scale } => {
                    axf[aqq] = (6, *center_x, *center_y, *scale, 0, 0);
                }
                crate::gesture::GestureEvent::Scroll { delta_x, delta_y } => {
                    axf[aqq] = (7, *delta_x, *delta_y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::ThreeFingerSwipe { direction } => {
                    let fsf = match direction {
                        crate::gesture::SwipeDirection::Left => 0,
                        crate::gesture::SwipeDirection::Right => 1,
                        crate::gesture::SwipeDirection::Up => 2,
                        crate::gesture::SwipeDirection::Down => 3,
                    };
                    axf[aqq] = (8, fsf, 0, 0, 0, 0);
                }
                crate::gesture::GestureEvent::TouchDown { x, y } => {
                    axf[aqq] = (9, *x, *y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::TouchMove { x, y } => {
                    axf[aqq] = (10, *x, *y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::TouchUp { x, y } => {
                    axf[aqq] = (11, *x, *y, 0, 0, 0);
                }
                crate::gesture::GestureEvent::Drag { x, y, start_x, start_y } => {
                    axf[aqq] = (12, *x, *y, *start_x, *start_y, 0);
                }
            }
            aqq += 1;
        }
        
        
        for i in 0..aqq {
            let (gtype, a, b, c, d, e) = axf[i];
            match gtype {
                1 => { 
                    self.update_cursor(a, b);
                    self.handle_click(a, b, true);
                    self.handle_click(a, b, false);
                }
                2 => { 
                    self.update_cursor(a, b);
                    self.handle_click(a, b, true);
                    self.handle_click(a, b, false);
                    self.handle_click(a, b, true);
                    self.handle_click(a, b, false);
                }
                3 => { 
                    self.update_cursor(a, b);
                    self.handle_right_click(a, b, true);
                    self.handle_right_click(a, b, false);
                }
                4 => { 
                    match e {
                        0 => {  }
                        1 => {  }
                        2 => {  }
                        3 => {  }
                        _ => {}
                    }
                }
                5 => { 
                    match a {
                        0 => { 
                            if !self.start_menu_open {
                                self.start_menu_open = true;
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
                    let ezq = if b > 0 { -1i8 } else if b < 0 { 1i8 } else { 0i8 };
                    if ezq != 0 {
                        self.handle_scroll(ezq);
                    }
                }
                8 => { 
                    
                    self.cycle_windows();
                }
                9 => { 
                    self.update_cursor(a, b);
                }
                10 => { 
                    self.update_cursor(a, b);
                }
                11 => { 
                    
                }
                12 => { 
                    self.update_cursor(a, b);
                }
                _ => {}
            }
        }
    }
    
    
    fn update_cursor(&mut self, x: i32, y: i32) {
        self.cursor_x = x.clamp(0, self.width as i32 - 1);
        self.cursor_y = y.clamp(0, self.height as i32 - 1);
    }
    
    
    fn cycle_windows(&mut self) {
        if self.windows.len() < 2 {
            return;
        }
        
        let lxg = self.windows.iter().position(|w| w.focused);
        if let Some(idx) = lxg {
            let next = (idx + 1) % self.windows.len();
            for w in self.windows.iter_mut() {
                w.focused = false;
            }
            self.windows[next].focused = true;
        }
    }
    
    
    pub fn draw(&mut self) {
        self.frame_count += 1;
        
        
        if self.shutdown_active {
            self.draw_shutdown_sequence();
            return;
        }
        
        
        
        
        
        if self.frame_count <= 3 {
            crate::serial_println!("[Desktop] safe frame {} / 3", self.frame_count);
            
            let mouse = crate::mouse::get_state();
            
            framebuffer::awo(0xFF010200);
            framebuffer::dix();
            
            self.draw_desktop_icons();
            self.draw_taskbar();
            self.draw_cursor();
            
            self.last_cursor_x = mouse.x;
            self.last_cursor_y = mouse.y;
            self.last_window_count = self.windows.len();
            self.last_start_menu_open = self.start_menu_open;
            self.last_context_menu_visible = self.context_menu.visible;
            framebuffer::civ();
            framebuffer::ii();
            crate::serial_println!("[Desktop] safe frame {} done", self.frame_count);
            return;
        }
        
        
        self.auto_adjust_tier();
        
        
        self.fps_frame_accum += 1;
        let gju = crate::logger::eg();
        if self.fps_last_tick == 0 { self.fps_last_tick = gju; }
        let bb = gju.saturating_sub(self.fps_last_tick);
        
        if bb >= 100 {
            self.fps_current = ((self.fps_frame_accum as u64 * 100) / bb.max(1)) as u32;
            self.fps_frame_accum = 0;
            self.fps_last_tick = gju;
        }
        
        
        self.update_animations();
        
        
        crate::drivers::net::wifi::poll();
        self.sys_wifi_connected = crate::drivers::net::wifi::czx();
        
        
        let (snake_ids, snake_n) = { let mut b = [0u32; 32]; let mut ae = 0; for &k in self.snake_states.keys() { if ae < 32 { b[ae] = k; ae += 1; } } (b, ae) };
        for &id in &snake_ids[..snake_n] {
            let is_active = self.windows.iter().any(|w| w.id == id && w.focused && w.visible && !w.minimized);
            if is_active {
                if let Some(snake) = self.snake_states.get_mut(&id) {
                    snake.tick();
                }
            }
        }
        
        
        let (mp_ids, mp_n) = { let mut b = [0u32; 32]; let mut ae = 0; for &k in self.music_player_states.keys() { if ae < 32 { b[ae] = k; ae += 1; } } (b, ae) };
        for &id in &mp_ids[..mp_n] {
            if let Some(ic) = self.music_player_states.get_mut(&id) {
                ic.tick();
            }
        }
        
        
        let (game3d_ids, game3d_n) = { let mut b = [0u32; 32]; let mut ae = 0; for &k in self.game3d_states.keys() { if ae < 32 { b[ae] = k; ae += 1; } } (b, ae) };
        for &id in &game3d_ids[..game3d_n] {
            let is_active = self.windows.iter().any(|w| w.id == id && w.focused && w.visible && !w.minimized);
            if is_active {
                if let Some(game) = self.game3d_states.get_mut(&id) {
                    game.tick();
                }
            }
        }
        
        
        #[cfg(feature = "emulators")]
        {
        let (nes_ids, nes_n) = { let mut b = [0u32; 32]; let mut ae = 0; for &k in self.nes_states.keys() { if ae < 32 { b[ae] = k; ae += 1; } } (b, ae) };
        for &id in &nes_ids[..nes_n] {
            let is_active = self.windows.iter().any(|w| w.id == id && w.focused && w.visible && !w.minimized);
            if is_active {
                if let Some(an) = self.nes_states.get_mut(&id) {
                    an.tick();
                }
            }
        }
        }
        
        
        
        #[cfg(feature = "emulators")]
        {
        let (gb_ids, gb_n) = { let mut b = [0u32; 32]; let mut ae = 0; for &k in self.gameboy_states.keys() { if ae < 32 { b[ae] = k; ae += 1; } } (b, ae) };
        for &id in &gb_ids[..gb_n] {
            let is_active = self.windows.iter().any(|w| w.id == id && w.visible && !w.minimized && !w.pending_close);
            if is_active {
                
                let cbh = self.gamelab_states.iter()
                    .find(|(_, lab)| lab.linked_gb_id == Some(id))
                    .map(|(&lid, _)| lid)
                    .or_else(|| self.gamelab_states.keys().next().copied());
                
                
                let (paused, mut step_one, mut step_frame, speed_idx, trace_enabled) =
                    if let Some(lid) = cbh {
                        if let Some(lab) = self.gamelab_states.get(&lid) {
                            (lab.paused, lab.step_one, lab.step_frame, lab.speed_idx, lab.trace_enabled)
                        } else { (false, false, false, 2, false) }
                    } else { (false, false, false, 2, false) };
                
                
                if paused && !step_one && !step_frame {
                    
                    if let Some(an) = self.gameboy_states.get_mut(&id) {
                        if !crate::keyboard::sx(0x11) { an.handle_key_release(b'w'); }
                        if !crate::keyboard::sx(0x1E) { an.handle_key_release(b'a'); }
                        if !crate::keyboard::sx(0x1F) { an.handle_key_release(b's'); }
                        if !crate::keyboard::sx(0x20) { an.handle_key_release(b'd'); }
                        if !crate::keyboard::sx(0x2D) { an.handle_key_release(b'x'); }
                        if !crate::keyboard::sx(0x2C) { an.handle_key_release(b'z'); }
                        if !crate::keyboard::sx(0x2E) { an.handle_key_release(b'c'); }
                        if !crate::keyboard::sx(0x1C) { an.handle_key_release(b'\r'); }
                    }
                    continue;
                }

                
                let gx = match speed_idx {
                    0 => if self.frame_count % 4 == 0 { 1 } else { 0 }, 
                    1 => if self.frame_count % 2 == 0 { 1 } else { 0 }, 
                    2 => 1, 
                    3 => 2, 
                    4 => 4, 
                    _ => 1,
                };

                if let Some(an) = self.gameboy_states.get_mut(&id) {
                    
                    if !crate::keyboard::sx(0x11) { an.handle_key_release(b'w'); }
                    if !crate::keyboard::sx(0x1E) { an.handle_key_release(b'a'); }
                    if !crate::keyboard::sx(0x1F) { an.handle_key_release(b's'); }
                    if !crate::keyboard::sx(0x20) { an.handle_key_release(b'd'); }
                    if !crate::keyboard::sx(0x2D) { an.handle_key_release(b'x'); }
                    if !crate::keyboard::sx(0x2C) { an.handle_key_release(b'z'); }
                    if !crate::keyboard::sx(0x2E) { an.handle_key_release(b'c'); }
                    if !crate::keyboard::sx(0x1C) { an.handle_key_release(b'\r'); }

                    
                    if trace_enabled {
                        if let Some(lid) = cbh {
                            
                            let pc = an.cpu.pc;
                            let a = an.cpu.a;
                            let f = an.cpu.f;
                            let sp = an.cpu.sp;
                            let opcode = crate::game_lab::aik(an, pc);
                            drop(an); 
                            if let Some(lab) = self.gamelab_states.get_mut(&lid) {
                                if lab.trace.len() >= 64 { lab.trace.remove(0); }
                                lab.trace.push(crate::game_lab::Vl { pc, opcode, a, f, sp });
                            }
                            
                            if let Some(an) = self.gameboy_states.get_mut(&id) {
                                for _ in 0..gx { an.tick(); }
                            }
                            
                            if step_one || step_frame {
                                if let Some(lab) = self.gamelab_states.get_mut(&lid) {
                                    lab.step_one = false;
                                    lab.step_frame = false;
                                }
                            }
                            
                            if let Some(an) = self.gameboy_states.get(&id) {
                                if let Some(lab) = self.gamelab_states.get_mut(&lid) {
                                    lab.update_watches(an);
                                    crate::game_lab::jpg(lab, an);
                                    
                                    if lab.should_break(an.cpu.pc) {
                                        lab.paused = true;
                                    }
                                }
                            }
                            continue; 
                        }
                    }

                    
                    for _ in 0..gx { an.tick(); }
                }

                
                if let Some(lid) = cbh {
                    if step_one || step_frame {
                        if let Some(lab) = self.gamelab_states.get_mut(&lid) {
                            lab.step_one = false;
                            lab.step_frame = false;
                        }
                    }
                    if let Some(an) = self.gameboy_states.get(&id) {
                        if let Some(lab) = self.gamelab_states.get_mut(&lid) {
                            lab.update_watches(an);
                            crate::game_lab::jpg(lab, an);
                            if lab.should_break(an.cpu.pc) {
                                lab.paused = true;
                            }
                        }
                    }
                }
            }
        }
        }
        
        
        let (flm, chess_n) = { let mut b = [0u32; 32]; let mut ae = 0; for &k in self.chess_states.keys() { if ae < 32 { b[ae] = k; ae += 1; } } (b, ae) };
        for &id in &flm[..chess_n] {
            let is_active = self.windows.iter().any(|w| w.id == id && w.visible && !w.minimized);
            if is_active {
                if let Some(chess) = self.chess_states.get_mut(&id) {
                    chess.tick_timer(16); 
                }
            }
        }
        
        
        let (lab_ids, lab_n) = { let mut b = [0u32; 32]; let mut ae = 0; for &k in self.lab_states.keys() { if ae < 32 { b[ae] = k; ae += 1; } } (b, ae) };
        for &id in &lab_ids[..lab_n] {
            let is_active = self.windows.iter().any(|w| w.id == id && w.visible && !w.minimized);
            if is_active {
                if let Some(lab) = self.lab_states.get_mut(&id) {
                    lab.tick();
                }
            }
        }
        
        
        let (wa_ids, wa_n) = { let mut b = [0u32; 32]; let mut ae = 0; for &k in self.wifi_analyzer_states.keys() { if ae < 32 { b[ae] = k; ae += 1; } } (b, ae) };
        for &id in &wa_ids[..wa_n] {
            let is_active = self.windows.iter().any(|w| w.id == id && w.visible && !w.minimized);
            if is_active {
                if let Some(apn) = self.wifi_analyzer_states.get_mut(&id) {
                    apn.tick();
                }
            }
        }
        
        
        #[cfg(feature = "emulators")]
        {
        let (gamelab_ids, gamelab_n) = { let mut b = [0u32; 32]; let mut ae = 0; for &k in self.gamelab_states.keys() { if ae < 32 { b[ae] = k; ae += 1; } } (b, ae) };
        for &id in &gamelab_ids[..gamelab_n] {
            let is_active = self.windows.iter().any(|w| w.id == id && w.visible && !w.minimized);
            if is_active {
                if let Some(lab) = self.gamelab_states.get_mut(&id) {
                    lab.tick();
                }
            }
        }
        }
        
        
        if self.frame_count % 9 == 0 {
            self.cursor_blink = !self.cursor_blink;
            
            for w in self.windows.iter_mut() {
                if w.focused && w.visible && !w.minimized {
                    match w.window_type {
                        WindowType::Terminal | WindowType::TextEditor => { w.dirty = true; }
                        _ => {}
                    }
                }
            }
        }
        
        
        let mouse = crate::mouse::get_state();
        
        
        if self.render_mode == RenderMode::OpenGL {
            self.draw_opengl();
            return;
        }
        
        
        if self.render_mode == RenderMode::GpuAccelerated {
            self.draw_gpu_accelerated();
            return;
        }
        
        
        
        
        let eeg = self.windows.len() != self.last_window_count;
        let ghm = self.start_menu_open != self.last_start_menu_open 
                        || self.context_menu.visible != self.last_context_menu_visible;
        
        
        if eeg {
            self.needs_full_redraw = true;
            self.windows_dirty = true;
            framebuffer::eqy();
        }
        
        
        
        if self.windows.iter().any(|w| w.dragging || w.resizing != ResizeEdge::None) {
            self.windows_dirty = true;
        }
        
        {
            let gbb = self.windows.iter().rev()
                .find(|w| w.visible && !w.minimized && {
                    let cg = self.cursor_x;
                    let cr = self.cursor_y;
                    cg >= w.x && cg < w.x + w.width as i32
                        && cr >= w.y && cr < w.y + J_() as i32
                })
                .map(|w| w.id);
            if gbb != self.prev_hover_window_id {
                
                for w in self.windows.iter_mut() {
                    if Some(w.id) == gbb || Some(w.id) == self.prev_hover_window_id {
                        w.dirty = true;
                    }
                }
                self.prev_hover_window_id = gbb;
            }
        }
        
        for w in self.windows.iter_mut() {
            if !w.visible || w.minimized { continue; }
            match w.window_type {
                WindowType::Game | WindowType::Game3D | WindowType::Chess
                | WindowType::Chess3D | WindowType::Demo3D => { w.dirty = true; }
                
                
                WindowType::MusicPlayer => {
                    if self.frame_count % 10 == 0 { w.dirty = true; }
                }
                _ => {}
            }
            #[cfg(feature = "emulators")]
            match w.window_type {
                WindowType::NesEmu | WindowType::GameBoyEmu | WindowType::GameLab => { w.dirty = true; }
                _ => {}
            }
        }
        
        
        if self.mobile_state.active {
            self.draw_mobile_mode();
            framebuffer::ii();
            return;
        }
        
        
        
        
        
        
        framebuffer::dix();
        if GQ_.load(Ordering::Relaxed) {
            if !eha() {
                
                self.draw_background();
            }
        } else {
            self.draw_background();
        }
        self.needs_full_redraw = false;
        self.draw_desktop_icons();
        
        
        
        
        
        
        
        let ief = self.windows.iter().any(|w| w.visible && !w.minimized);
        
        let jwl = self.windows_dirty
            || eeg
            || ghm
            || !framebuffer::mud()
            || self.windows.iter().any(|w| w.visible && !w.minimized && w.dirty);
        
        if jwl || !ief {
            for window in &self.windows {
                if window.visible && !window.minimized {
                    self.draw_window(window);
                }
            }
            self.draw_editor_windows();
            self.draw_model_editor_windows();
            self.draw_game3d_windows();
            self.draw_chess3d_windows();
            #[cfg(feature = "emulators")]
            self.draw_nes_windows();
            #[cfg(feature = "emulators")]
            self.draw_gameboy_windows();
            
            if ief {
                framebuffer::kgu();
            }
            
            for w in self.windows.iter_mut() {
                w.dirty = false;
            }
            self.windows_dirty = false;
        } else {
            
            
            
            for window in &self.windows {
                if window.visible && !window.minimized {
                    framebuffer::kco(
                        window.x, window.y,
                        window.width, window.height,
                        0, 
                    );
                }
            }
        }
        
        
        if let Some(gvm) = self.snap_preview {
            let ang = self.height - V_();
            let nk = self.width / 2;
            let kh = ang / 2;
            let (am, ak, dy, dw) = match gvm {
                SnapDir::Left       => (0, 0, nk, ang),
                SnapDir::Right      => (nk, 0, nk, ang),
                SnapDir::TopLeft    => (0, 0, nk, kh),
                SnapDir::TopRight   => (nk, 0, nk, kh),
                SnapDir::BottomLeft => (0, kh, nk, kh),
                SnapDir::BottomRight => (nk, kh, nk, kh),
            };
            
            framebuffer::co(am, ak, dy, dw, 0x00FF66, 18);
            
            framebuffer::draw_rect(am + 2, ak + 2, dy.saturating_sub(4), dw.saturating_sub(4), Y_);
            framebuffer::draw_rect(am + 3, ak + 3, dy.saturating_sub(6), dw.saturating_sub(6), Q_);
        }
        
        
        self.draw_taskbar();
        
        
        if self.start_menu_open {
            self.draw_start_menu();
        }
        
        
        if self.context_menu.visible {
            self.draw_context_menu();
        }
        
        
        self.draw_drag_ghost();
        
        
        if self.lock_screen_active {
            self.draw_lock_screen();
        }

        
        self.draw_cursor();
        
        
        self.last_cursor_x = mouse.x;
        self.last_cursor_y = mouse.y;
        self.last_window_count = self.windows.len();
        self.last_start_menu_open = self.start_menu_open;
        self.last_context_menu_visible = self.context_menu.visible;
        
        
        framebuffer::civ();
        framebuffer::ii();
    }
    
    
    fn draw_opengl(&mut self) {
        use crate::graphics::opengl::*;
        
        
        let fm = 1.0 / 60.0; 
        
        
        {
            let mut bfm = compositor::compositor();
            bfm.update(fm);
            
            
            for window in &self.windows {
                if let Some(surface) = bfm.get_surface_mut(window.id) {
                    surface.x = window.x as f32;
                    surface.y = window.y as f32;
                    surface.width = window.width as f32;
                    surface.height = window.height as f32;
                    surface.focused = window.focused;
                    surface.visible = window.visible && !window.minimized;
                }
            }
        }
        
        
        compositor::ofh();
        
        
        
        self.draw_taskbar();
        
        if self.start_menu_open {
            self.draw_start_menu();
        }
        
        if self.context_menu.visible {
            self.draw_context_menu();
        }
        
        
        self.draw_desktop_icons();
        
        
        self.draw_cursor();
        
        
        self.last_window_count = self.windows.len();
        self.last_start_menu_open = self.start_menu_open;
        self.last_context_menu_visible = self.context_menu.visible;
        
        
        framebuffer::ii();
    }
    
    
    
    
    
    
    
    fn add_dirty_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        if self.dirty_rect_count < 32 {
            self.dirty_rects[self.dirty_rect_count] = (x, y, w, h);
            self.dirty_rect_count += 1;
        }
    }
    
    
    fn draw_gpu_accelerated(&mut self) {
        let mouse = crate::mouse::get_state();
        let eeg = self.windows.len() != self.last_window_count;
        let ghm = self.start_menu_open != self.last_start_menu_open
                        || self.context_menu.visible != self.last_context_menu_visible;
        let lar = mouse.x != self.last_cursor_x || mouse.y != self.last_cursor_y;
        
        
        self.dirty_rect_count = 0;
        let qgc = self.needs_full_redraw || eeg;
        
        
        if eeg || ghm || self.needs_full_redraw {
            
            self.add_dirty_rect(0, 0, self.width, self.height);
            self.needs_full_redraw = false;
        } else {
            
            if lar {
                
                let gks = (self.last_cursor_x.max(0) as u32).saturating_sub(2);
                let gkt = (self.last_cursor_y.max(0) as u32).saturating_sub(2);
                self.add_dirty_rect(gks, gkt, 24, 24);
                
                let cbw = (mouse.x.max(0) as u32).saturating_sub(2);
                let afk = (mouse.y.max(0) as u32).saturating_sub(2);
                self.add_dirty_rect(cbw, afk, 24, 24);
            }
            
            
            let puq: Vec<(u32, u32, u32, u32)> = self.windows.iter()
                .filter(|w| w.visible && !w.minimized)
                .map(|w| (w.x.max(0) as u32, w.y.max(0) as u32, w.width, w.height))
                .collect();
            for (wx, wy, ca, er) in puq {
                self.add_dirty_rect(wx, wy, ca, er);
            }
            
            if self.frame_count % 60 == 0 {
                self.add_dirty_rect(0, self.height.saturating_sub(40), self.width, 40);
            }
        }
        
        
        if self.mobile_state.active {
            self.draw_mobile_mode();
        } else {
            framebuffer::dix();
            let qjs = self.windows.iter().any(|w| w.visible && !w.minimized);
            
            if GQ_.load(Ordering::Relaxed) {
                if !eha() {
                    framebuffer::awo(0xFF000000);
                    self.draw_background();
                }
            } else {
                framebuffer::awo(0xFF000000);
                self.draw_background();
            }
            self.draw_desktop_icons();
            
            for window in &self.windows {
                if window.visible && !window.minimized {
                    self.draw_window(window);
                }
            }
            self.draw_editor_windows();
            self.draw_model_editor_windows();
            self.draw_game3d_windows();
            self.draw_chess3d_windows();
            #[cfg(feature = "emulators")]
            self.draw_nes_windows();
            #[cfg(feature = "emulators")]
            self.draw_gameboy_windows();
            self.draw_taskbar();
            if self.start_menu_open { self.draw_start_menu(); }
            if self.context_menu.visible { self.draw_context_menu(); }
            self.draw_drag_ghost();
            if self.lock_screen_active { self.draw_lock_screen(); }
            self.draw_cursor();
            framebuffer::civ();
        }
        
        
        self.last_cursor_x = mouse.x;
        self.last_cursor_y = mouse.y;
        self.last_window_count = self.windows.len();
        self.last_start_menu_open = self.start_menu_open;
        self.last_context_menu_visible = self.context_menu.visible;
        
        
        if crate::drivers::virtio_gpu::sw() && self.dirty_rect_count > 0 && self.dirty_rect_count < 32 {
            
            crate::drivers::virtio_gpu::nwv(
                &self.dirty_rects[..self.dirty_rect_count]
            );
            
            framebuffer::oza();
        } else {
            framebuffer::ii();
        }
        
        self.gpu_frame_skip = self.gpu_frame_skip.wrapping_add(1);
    }
    
    
    fn draw_context_menu(&self) {
        let hu = self.context_menu.x;
        let ks = self.context_menu.y;
        let bhx = 200i32;
        let axs = 28;
        let dbf = self.context_menu.items.len() as i32 * axs + 8;
        let padding = 4;
        let bfo: u32 = 8;
        
        
        for i in (1..=6).rev() {
            let alpha = (18 - i * 2).max(4) as u32;
            let bjd = alpha << 24;
            draw_rounded_rect(
                hu + i, ks + i + 2,
                bhx as u32, dbf as u32,
                bfo + 2, bjd,
            );
        }
        
        
        draw_rounded_rect(
            hu, ks,
            bhx as u32, dbf as u32,
            bfo, 0xFF0C1210,
        );
        
        
        
        for row in 0..dbf.min(20) {
            let eok = (12 - row * 12 / 20).max(0) as u32;
            if eok > 0 {
                let ayx = (eok << 24) | 0x00FFFFFF;
                
                let clf = if row < bfo as i32 { (bfo as i32 - cxr((bfo as i32 * bfo as i32) - (bfo as i32 - row) * (bfo as i32 - row))) } else { 0 };
                let fe = hu + clf;
                let mo = bhx - clf * 2;
                if mo > 0 {
                    crate::framebuffer::fill_rect(fe as u32, (ks + row) as u32, mo as u32, 1, ayx);
                }
            }
        }
        
        
        iu(
            hu, ks,
            bhx as u32, dbf as u32,
            bfo, GR_,
        );
        
        
        crate::framebuffer::fill_rect(
            (hu + bfo as i32) as u32, ks as u32,
            (bhx - bfo as i32 * 2) as u32, 1, EP_,
        );
        
        
        for (idx, item) in self.context_menu.items.iter().enumerate() {
            let ru = ks + padding + idx as i32 * axs;
            
            let vl = self.cursor_x >= hu && self.cursor_x < hu + bhx
                && self.cursor_y >= ru && self.cursor_y < ru + axs;
            
            if vl && item.action != ContextAction::Cancel && !item.label.starts_with("─") {
                
                draw_rounded_rect(
                    hu + 4, ru,
                    (bhx - 8) as u32, (axs - 2) as u32,
                    6, Q_,
                );
                draw_rounded_rect(
                    hu + 6, ru + 1,
                    (bhx - 12) as u32, (axs - 4) as u32,
                    5, ANM_,
                );
                
                draw_rounded_rect(
                    hu + 4, ru + 4,
                    2, (axs - 10) as u32,
                    1, I_,
                );
            }
            
            
            if item.label.starts_with("─") {
                framebuffer::fill_rect(
                    (hu + 12) as u32, (ru + axs / 2) as u32,
                    (bhx - 24) as u32, 1,
                    Q_
                );
            } else {
                let text_color = if vl { AH_ } else { BM_ };
                self.draw_text(hu + 16, ru + 6, &item.label, text_color);
            }
        }
    }

    
    
    
    fn analyze_global_audio(&mut self) {
        
        if self.global_fft_re.len() < 256 {
            self.global_fft_re.resize(256, 0.0);
            self.global_fft_im.resize(256, 0.0);
        }
        if self.global_energy_hist.len() < 43 {
            self.global_energy_hist.resize(43, 0.0);
        }

        
        if !crate::drivers::hda::is_playing() {
            
            self.global_sub_bass *= 0.92;
            self.global_bass *= 0.92;
            self.global_mid *= 0.92;
            self.global_treble *= 0.92;
            self.global_energy *= 0.92;
            self.global_beat *= 0.85;
            if self.global_energy < 0.001 {
                self.global_audio_active = false;
            }
            return;
        }

        
        let lgf = crate::drivers::hda::cym();
        let alw = crate::drivers::hda::dqq();
        
        let (buf_ptr, buf_cap) = match lgf {
            Some((aa, c)) if !aa.is_null() && c > 512 => (aa, c),
            _ => return,
        };

        
        
        let ggi = (alw as usize) / 2;
        
        let aca = 256usize;
        let ode = if ggi >= aca * 2 {
            ggi - aca * 2
        } else {
            
            buf_cap.saturating_sub(aca * 2 - ggi)
        };

        let mut yw: f32 = 0.0;
        for i in 0..aca {
            let idx = (ode + i * 2) % buf_cap; 
            let j = unsafe { *buf_ptr.add(idx) } as f32;
            self.global_fft_re[i] = j;
            self.global_fft_im[i] = 0.0;
            let a = if j >= 0.0 { j } else { -j };
            if a > yw { yw = a; }
        }

        
        if yw < 10.0 {
            self.global_audio_active = false;
            self.global_sub_bass *= 0.92;
            self.global_bass *= 0.92;
            self.global_mid *= 0.92;
            self.global_treble *= 0.92;
            self.global_energy *= 0.92;
            self.global_beat *= 0.85;
            return;
        }
        self.global_audio_active = true;

        
        if yw > self.global_peak_rms {
            self.global_peak_rms += (yw - self.global_peak_rms) * 0.3;
        } else {
            self.global_peak_rms *= 0.9995;
        }
        let bmi = if self.global_peak_rms > 100.0 { 16000.0 / self.global_peak_rms } else { 1.0 };

        
        for i in 0..aca {
            let t = i as f32 / aca as f32;
            let drf = 0.5 * (1.0 - libm::cosf(2.0 * core::f32::consts::PI * t));
            self.global_fft_re[i] *= drf * bmi / 32768.0;
        }

        
        {
            let xh = &mut self.global_fft_re[..aca];
            let xq = &mut self.global_fft_im[..aca];
            
            let mut ay = 0usize;
            for i in 0..aca {
                if i < ay { xh.swap(i, ay); xq.swap(i, ay); }
                let mut m = aca >> 1;
                while m >= 1 && ay >= m { ay -= m; m >>= 1; }
                ay += m;
            }
            
            let mut step = 2usize;
            while step <= aca {
                let cw = step >> 1;
                let cc = -core::f32::consts::PI / cw as f32;
                let (asv, bww) = (libm::sinf(cc), libm::cosf(cc));
                for k in (0..aca).step_by(step) {
                    let (mut aep, mut ld) = (1.0f32, 0.0f32);
                    for m in 0..cw {
                        let ard = k + m;
                        let axt = ard + cw;
                        let tr = aep * xh[axt] - ld * xq[axt];
                        let cej = aep * xq[axt] + ld * xh[axt];
                        xh[axt] = xh[ard] - tr; xq[axt] = xq[ard] - cej;
                        xh[ard] += tr; xq[ard] += cej;
                        let njw = aep * bww - ld * asv;
                        ld = aep * asv + ld * bww;
                        aep = njw;
                    }
                }
                step <<= 1;
            }
        }

        
        let bue = |xh: &[f32], xq: &[f32], lo: usize, hi: usize| -> f32 {
            let mut j = 0.0f32;
            for i in lo..hi.min(128) {
                j += libm::sqrtf(xh[i] * xh[i] + xq[i] * xq[i]);
            }
            j / (hi - lo).max(1) as f32
        };
        let cov = bue(&self.global_fft_re, &self.global_fft_im, 1, 2);
        let biu = bue(&self.global_fft_re, &self.global_fft_im, 2, 4);
        let cda = bue(&self.global_fft_re, &self.global_fft_im, 4, 16);
        let dxh = bue(&self.global_fft_re, &self.global_fft_im, 16, 60);
        let gpr = cov * 1.5 + biu * 1.2 + cda * 0.5 + dxh * 0.2;

        
        let afs = |prev: f32, new: f32, a: f32, r: f32| -> f32 {
            if new > prev { prev + (new - prev) * a } else { prev + (new - prev) * r }
        };
        self.global_sub_bass = afs(self.global_sub_bass, cov.min(1.0), 0.75, 0.10);
        self.global_bass = afs(self.global_bass, biu.min(1.0), 0.70, 0.10);
        self.global_mid = afs(self.global_mid, cda.min(1.0), 0.60, 0.12);
        self.global_treble = afs(self.global_treble, dxh.min(1.0), 0.70, 0.16);
        self.global_energy = afs(self.global_energy, gpr.min(1.5), 0.65, 0.10);

        
        let beu = cov + biu * 0.8;
        self.global_energy_hist[self.global_hist_idx] = beu;
        self.global_hist_idx = (self.global_hist_idx + 1) % 43;
        if self.global_hist_count < 43 { self.global_hist_count += 1; }
        let oz = self.global_hist_count.max(1) as f32;
        let ns: f32 = self.global_energy_hist.iter().take(self.global_hist_count).sum::<f32>() / oz;
        let mut cex = 0.0f32;
        for i in 0..self.global_hist_count {
            let d = self.global_energy_hist[i] - ns;
            cex += d * d;
        }
        let edo = cex / oz;
        let amz = (-15.0 * edo + 1.45f32).max(1.05).min(1.5);
        let glb = beu - self.global_prev_energy;
        if beu > ns * amz && glb > 0.002 && self.global_hist_count > 5 {
            let strength = ((beu - ns * amz) / ns.max(0.001)).min(1.0);
            self.global_beat = (0.6 + strength * 0.4).min(1.0);
        } else {
            self.global_beat *= 0.88;
            if self.global_beat < 0.02 { self.global_beat = 0.0; }
        }
        self.global_prev_energy = beu;
    }
    
    
    
    
    
    fn draw_mobile_mode(&mut self) {
        
        let (vx, vy, bt, ex) = crate::mobile::hjt(self.width, self.height);
        self.mobile_state.vp_x = vx;
        self.mobile_state.vp_y = vy;
        self.mobile_state.vp_w = bt;
        self.mobile_state.vp_h = ex;

        
        framebuffer::awo(0xFF000000);

        
        if GQ_.load(Ordering::Relaxed) {
            if !eha() {
                self.draw_background();
            }
        } else {
            self.draw_background();
        }

        
        if vx > 0 {
            framebuffer::fill_rect(0, 0, vx as u32, self.height, 0xFF020202);
            framebuffer::fill_rect((vx + bt as i32) as u32, 0, (self.width as i32 - vx - bt as i32).max(0) as u32, self.height, 0xFF020202);
        }

        
        crate::mobile::lkk(vx, vy, bt, ex);

        
        crate::mobile::pjf(&mut self.mobile_state);

        
        if self.frame_count % 60 == 0 || self.mobile_state.time_str.is_empty() {
            let fm = crate::rtc::aou();
            use core::fmt::Write;
            self.mobile_state.time_str.clear();
            let _ = core::write!(self.mobile_state.time_str, "{:02}:{:02}", fm.hour, fm.minute);
        }

        let frame = self.mobile_state.anim_frame;
        let view = self.mobile_state.view;
        let time_str = self.mobile_state.time_str.clone();
        let hl = self.mobile_state.highlighted_icon;

        
        crate::mobile::draw_status_bar(vx, vy, bt, ex, &time_str, frame);

        match view {
            crate::mobile::MobileView::Home => {
                crate::mobile::htk(vx, vy, bt, ex, hl, frame);
                
                let jyd = crate::mobile::Mr {
                    playing: self.global_audio_active,
                    beat: self.global_beat,
                    energy: self.global_energy,
                    sub_bass: self.global_sub_bass,
                    bass: self.global_bass,
                    mid: self.global_mid,
                    treble: self.global_treble,
                    frame: self.frame_count,
                };
                crate::mobile::ljo(vx, vy, bt, ex, &jyd,
                    self.mobile_state.music_dropdown_open,
                    self.mobile_state.music_viz_mode);
                crate::mobile::draw_dock(vx, vy, bt, ex, -1, frame);
                crate::mobile::ekm(vx, vy, bt, ex);
            }
            crate::mobile::MobileView::AppFullscreen => {
                
                let awc = self.mobile_state.active_app_id.unwrap_or(0);
                let fhe = if (awc as usize) < crate::mobile::jwv() {
                    crate::mobile::fhe(awc as usize)
                } else { "App" };
                crate::mobile::lhn(vx, vy, bt, fhe, frame);
                
                let jye = crate::mobile::Mr {
                    playing: self.global_audio_active,
                    beat: self.global_beat,
                    energy: self.global_energy,
                    sub_bass: self.global_sub_bass,
                    bass: self.global_bass,
                    mid: self.global_mid,
                    treble: self.global_treble,
                    frame: self.frame_count,
                };
                crate::mobile::ljn(
                    vx, vy, bt, ex,
                    awc, self.frame_count, &jye,
                    &self.mobile_state,
                );
                
                crate::mobile::ekm(vx, vy, bt, ex);
            }
            crate::mobile::MobileView::AppSwitcher => {
                crate::mobile::lhx(vx, vy, bt, ex, &[], 0, frame);
                crate::mobile::ekm(vx, vy, bt, ex);
            }
            crate::mobile::MobileView::ControlCenter => {
                crate::mobile::htk(vx, vy, bt, ex, hl, frame);
                crate::mobile::draw_dock(vx, vy, bt, ex, -1, frame);
                crate::mobile::lik(vx, vy, bt, ex, self.mobile_state.cc_progress, frame);
                crate::mobile::ekm(vx, vy, bt, ex);
            }
        }

        
        self.draw_cursor();
    }

    
    fn apply_mobile_action(&mut self, action: crate::mobile::MobileAction) {
        use crate::mobile::MobileAction;
        match action {
            MobileAction::None => {}
            MobileAction::GoHome => {
                self.mobile_state.view = crate::mobile::MobileView::Home;
                self.mobile_state.active_app_id = None;
            }
            MobileAction::OpenSwitcher => {
                self.mobile_state.view = crate::mobile::MobileView::AppSwitcher;
            }
            MobileAction::OpenControlCenter => {
                self.mobile_state.view = crate::mobile::MobileView::ControlCenter;
                self.mobile_state.cc_progress = 1; 
            }
            MobileAction::CloseControlCenter => {
                self.mobile_state.view = crate::mobile::MobileView::Home;
            }
            MobileAction::LaunchApp(idx) => {
                self.mobile_state.view = crate::mobile::MobileView::AppFullscreen;
                self.mobile_state.active_app_id = Some(idx as u32);
                crate::serial_println!("[Mobile] Launch app #{}", idx);
            }
            MobileAction::LaunchDockApp(slot) => {
                let idx = crate::mobile::lgq(slot as usize);
                self.mobile_state.view = crate::mobile::MobileView::AppFullscreen;
                self.mobile_state.active_app_id = Some(idx as u32);
                crate::serial_println!("[Mobile] Launch dock app slot={} -> idx={}", slot, idx);
            }
            MobileAction::BackFromApp => {
                self.mobile_state.view = crate::mobile::MobileView::Home;
                self.mobile_state.active_app_id = None;
            }
            MobileAction::CloseSwitcherCard(id) => {
                self.mobile_state.closing_cards.push((id, 255));
            }
            MobileAction::MusicTogglePlay => {
                
                const GD_: u32 = 0xFFFF_FFFE;
                if !self.music_player_states.contains_key(&GD_) {
                    self.music_player_states.insert(GD_, MusicPlayerState::new());
                }
                if let Some(ic) = self.music_player_states.get_mut(&GD_) {
                    match ic.state {
                        PlaybackState::Stopped => ic.play_track(0),
                        PlaybackState::Playing | PlaybackState::Paused => ic.toggle_pause(),
                    }
                }
            }
            MobileAction::MusicStop => {
                const GD_: u32 = 0xFFFF_FFFE;
                if let Some(ic) = self.music_player_states.get_mut(&GD_) {
                    ic.stop();
                }
            }
            MobileAction::MusicToggleDropdown => {
                self.mobile_state.music_dropdown_open = !self.mobile_state.music_dropdown_open;
            }
            MobileAction::MusicSetVizMode(mode) => {
                self.mobile_state.music_viz_mode = mode;
                self.mobile_state.music_dropdown_open = false;
                
                self.visualizer.mode = mode;
                crate::serial_println!("[Mobile] Viz mode set to {} ({})", mode,
                    crate::visualizer::PE_[mode as usize % crate::visualizer::JJ_ as usize]);
            }
            MobileAction::CalcButton(ahl) => {
                
                let dh = &mut self.mobile_state;
                match ahl {
                    16 => { 
                        dh.calc_display.clear();
                        dh.calc_op = 0;
                        dh.calc_operand = 0;
                        dh.calc_fresh = false;
                    }
                    17 => { 
                        if !dh.calc_display.is_empty() && dh.calc_display != "0" {
                            if dh.calc_display.starts_with('-') {
                                dh.calc_display.remove(0);
                            } else {
                                dh.calc_display.insert(0, '-');
                            }
                        }
                    }
                    18 => { 
                        if let Ok(v) = dh.calc_display.parse::<i64>() {
                            let result = v / 100;
                            dh.calc_display.clear();
                            use core::fmt::Write;
                            let _ = core::write!(dh.calc_display, "{}", result);
                        }
                    }
                    10 => { 
                        if dh.calc_fresh { dh.calc_display.clear(); dh.calc_fresh = false; }
                        if !dh.calc_display.contains('.') {
                            if dh.calc_display.is_empty() { dh.calc_display.push('0'); }
                            dh.calc_display.push('.');
                        }
                    }
                    15 => { 
                        let current = dh.calc_display.parse::<i64>().unwrap_or(0);
                        let result = match dh.calc_op {
                            1 => dh.calc_operand + current,
                            2 => dh.calc_operand - current,
                            3 => dh.calc_operand * current,
                            4 => if current != 0 { dh.calc_operand / current } else { 0 },
                            _ => current,
                        };
                        dh.calc_display.clear();
                        use core::fmt::Write;
                        let _ = core::write!(dh.calc_display, "{}", result);
                        dh.calc_op = 0;
                        dh.calc_operand = 0;
                        dh.calc_fresh = true;
                    }
                    11 | 12 | 13 | 14 => { 
                        let current = dh.calc_display.parse::<i64>().unwrap_or(0);
                        
                        if dh.calc_op > 0 && !dh.calc_fresh {
                            let result = match dh.calc_op {
                                1 => dh.calc_operand + current,
                                2 => dh.calc_operand - current,
                                3 => dh.calc_operand * current,
                                4 => if current != 0 { dh.calc_operand / current } else { 0 },
                                _ => current,
                            };
                            dh.calc_operand = result;
                            dh.calc_display.clear();
                            use core::fmt::Write;
                            let _ = core::write!(dh.calc_display, "{}", result);
                        } else {
                            dh.calc_operand = current;
                        }
                        dh.calc_op = ahl - 10; 
                        dh.calc_fresh = true;
                    }
                    0..=9 => { 
                        if dh.calc_fresh {
                            dh.calc_display.clear();
                            dh.calc_fresh = false;
                        }
                        if dh.calc_display == "0" { dh.calc_display.clear(); }
                        if dh.calc_display.len() < 15 {
                            dh.calc_display.push((b'0' + ahl) as char);
                        }
                    }
                    _ => {}
                }
                crate::serial_println!("[Mobile] Calc: display={}", dh.calc_display);
            }
            MobileAction::FilesTap(idx) => {
                let dh = &mut self.mobile_state;
                dh.files_selected = idx as i32;
                
                if idx < 4 && dh.files_depth == 0 {
                    dh.files_depth = 1;
                    dh.files_selected = -1;
                }
                crate::serial_println!("[Mobile] Files: tap idx={} depth={}", idx, dh.files_depth);
            }
            MobileAction::FilesBack => {
                self.mobile_state.files_depth = self.mobile_state.files_depth.saturating_sub(1);
                self.mobile_state.files_selected = -1;
            }
            MobileAction::SettingsTap(idx) => {
                let dh = &mut self.mobile_state;
                dh.settings_selected = idx as i32;
                if (idx as usize) < dh.settings_toggles.len() {
                    dh.settings_toggles[idx as usize] = !dh.settings_toggles[idx as usize];
                }
                crate::serial_println!("[Mobile] Settings: toggled idx={}", idx);
            }
            MobileAction::GamesTap(idx) => {
                self.mobile_state.games_selected = idx as i32;
                crate::serial_println!("[Mobile] Games: selected idx={}", idx);
            }
            MobileAction::BrowserNav(za) => {
                self.mobile_state.browser_page = za;
                crate::serial_println!("[Mobile] Browser: page={}", za);
            }
            MobileAction::EditorTap(line) => {
                self.mobile_state.editor_cursor_line = line as u32;
            }
            MobileAction::EditorSwitchTab(tab) => {
                self.mobile_state.editor_tab = tab;
            }
            MobileAction::ChessTap(cu) => {
                let dh = &mut self.mobile_state;
                if dh.chess_selected == cu as i32 {
                    dh.chess_selected = -1; 
                } else if dh.chess_selected >= 0 {
                    
                    dh.chess_turn = 1 - dh.chess_turn;
                    dh.chess_selected = -1;
                    crate::serial_println!("[Mobile] Chess: move to sq={}", cu);
                } else {
                    dh.chess_selected = cu as i32;
                }
            }
            MobileAction::MusicAppToggle => {
                const GD_: u32 = 0xFFFF_FFFE;
                if !self.music_player_states.contains_key(&GD_) {
                    self.music_player_states.insert(GD_, MusicPlayerState::new());
                }
                if let Some(ic) = self.music_player_states.get_mut(&GD_) {
                    match ic.state {
                        PlaybackState::Stopped => ic.play_track(0),
                        PlaybackState::Playing | PlaybackState::Paused => ic.toggle_pause(),
                    }
                }
            }
            MobileAction::TermSubmit => {
                let dh = &mut self.mobile_state;
                
                let commands = ["help", "uname", "ls", "pwd", "whoami", "date", "free -h", "uptime"];
                let kor = dh.term_lines.len() / 2; 
                let cmd = commands[kor % commands.len()];
                dh.term_lines.push(alloc::format!("$ {}", cmd));
                let fa = match cmd {
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
                dh.term_lines.push(alloc::string::String::from(fa));
                
                if dh.term_lines.len() > 40 {
                    dh.term_lines.drain(0..2);
                }
            }
        }
    }

    
    
    
    
    
    
    
    
    
    fn draw_shutdown_sequence(&mut self) {
        let cy = crate::logger::eg();
        let bb = cy.saturating_sub(self.shutdown_start_tick); 
        
        
        const PU_: u64 = 50;   
        const PV_: u64 = 100;  
        const HQ_: u64 = 140;  
        const XN_: u64 = 180;  
        const CMT_: u64 = 200;  
        
        let width = self.width;
        let height = self.height;
        
        framebuffer::awo(0xFF010200);
        framebuffer::dix();
        
        
        
        
        let edf = if bb >= PU_ {
            255u8
        } else {
            ((bb * 255) / PU_) as u8
        };
        
        
        let dtz = if bb < PU_ {
            0u8
        } else if bb >= PV_ {
            255u8
        } else {
            (((bb - PU_) * 255) / (PV_ - PU_)) as u8
        };
        
        
        let fem = if bb < PV_ {
            0u8
        } else if bb >= HQ_ {
            255u8
        } else {
            (((bb - PV_) * 255) / (HQ_ - PV_)) as u8
        };
        
        
        let ilb = if bb < HQ_ {
            0u8
        } else if bb >= XN_ {
            255u8
        } else {
            (((bb - HQ_) * 255) / (XN_ - HQ_)) as u8
        };
        
        
        
        
        if dtz < 255 {
            
            if GQ_.load(Ordering::Relaxed) {
                if !eha() {
                    self.draw_background();
                }
            } else {
                self.draw_background();
            }
            
            
            if dtz > 0 {
                framebuffer::co(0, 0, width, height, 0x000000, dtz as u32);
            }
            
            
            if fem > 0 && fem < 255 {
                
                
                let ua = fem.saturating_sub(dtz);
                if ua > 0 {
                    
                    let cx = width / 4;
                    let u = height / 4;
                    framebuffer::co(cx, u, width / 2, height / 2, 0x000000, ua as u32);
                }
            }
            
            
            if ilb > 0 {
                let ark = crate::logo_bitmap::BA_ as u32;
                let arj = crate::logo_bitmap::BN_ as u32;
                let cbn = (width / 2).saturating_sub(ark / 2);
                let cbo = (height / 2).saturating_sub(arj / 2);
                let ua = ilb.saturating_sub(fem.max(dtz));
                if ua > 0 {
                    let pad = 20u32;
                    framebuffer::co(
                        cbn.saturating_sub(pad), cbo.saturating_sub(pad),
                        ark + pad * 2, arj + pad * 2,
                        0x000000, ua as u32,
                    );
                }
            }
        }
        
        
        
        if edf < 255 {
            
            let osj = (edf as i32 * BW_() as i32) / 255;
            let pdl = (edf as i32 * V_() as i32) / 255;
            
            
            if osj < BW_() as i32 {
                
                self.draw_desktop_icons();
                
                framebuffer::co(0, 0, BW_() as u32 + 10, height, 0x000000, edf as u32);
            }
            
            
            if pdl < V_() as i32 {
                self.draw_taskbar();
                framebuffer::co(0, height.saturating_sub(V_() as u32), width, V_() as u32, 0x000000, edf as u32);
            }
        }
        
        
        if bb >= HQ_ {
            let pie = if bb < XN_ {
                (((bb - HQ_) * 255) / (XN_ - HQ_)) as u8
            } else {
                255u8
            };
            let g = (pie as u32 * 0xAA) / 255;
            let color = 0xFF000000 | (g << 8);
            let bk = "Shutting down...";
            let ew = 8u32;
            let acy = bk.len() as u32 * ew;
            let bu = (width / 2).saturating_sub(acy / 2);
            let ty = height / 2 + 40;
            let mut cx = bu;
            for ch in bk.chars() {
                framebuffer::px(cx, ty, ch, color);
                cx += ew;
            }
        }
        
        framebuffer::civ();
        framebuffer::ii();
        
        
        if bb >= CMT_ {
            crate::serial_println!("[SHUTDOWN] Animation complete, powering off...");
            
            framebuffer::awo(0xFF000000);
            framebuffer::dix();
            framebuffer::civ();
            framebuffer::ii();
            
            
            for (_id, ic) in self.music_player_states.iter_mut() {
                ic.state = PlaybackState::Stopped;
            }
            
            
            crate::acpi::shutdown();
            
        }
    }

    fn draw_background(&mut self) {
        
        
        
        
        let matrix_cols = self.matrix_cols;
        const EZ_: usize = 40;
        let num_layers: usize = if self.desktop_tier >= DesktopTier::Full {
            4
        } else if self.desktop_tier >= DesktopTier::Standard {
            2
        } else {
            1  
        };
        
        
        
        
        
        
        const CHM_: [usize; 4]  = [28, 20, 14, 8];
        const CHG_: [f32; 4]       = [0.28, 0.50, 0.78, 1.0];
        const CHK_: [[f32; 4]; 3] = [
            [0.44, 0.88, 1.62, 2.25],
            [0.75, 1.50, 2.75, 3.75],
            [1.19, 2.38, 4.38, 6.25],
        ];
        let preset = (self.matrix_rain_preset as usize).min(2);
        let mxi: [f32; 4] = CHK_[preset];
        const CHL_: [f32; 4]      = [0.0, 0.3, 1.0, 2.0];
        const CHF_: [usize; 4] = [2, 3, 4, 6];
        const CHE_: [i16; 4]    = [ 0,  0,  0,  0];
        const CHD_: [i16; 4]    = [-4,  0,  4,  8];
        const CHC_: [i16; 4]    = [ 0,  0,  0,  0];
        const CHH_: [f32; 4]       = [0.0, 1.5, 3.5, 6.0];
        const CHJ_: [u32; 4]    = [5, 7, 10, 14];
        const CHI_: [u32; 4]    = [10, 14, 20, 28];
        const CKD_: [u32; 4]   = [3, 4, 6, 7];
        const CKC_: [u32; 4]   = [6, 8, 12, 14];
        const CKB_: [usize; 4] = [2, 3, 6, 8];
        let dsq = self.mobile_state.active;
        
        let height = self.height.saturating_sub(V_());
        let width = self.width;
        
        
        self.analyze_global_audio();
        let bud = self.global_beat;
        let cmm = self.global_energy;
        let eto = self.global_sub_bass;
        let etm = self.global_bass;
        let etn = self.global_mid;
        let etp = self.global_treble;
        let axy = self.global_audio_active;
        
        
        let hhd = axy && bud > 0.5;
        if hhd && !self.matrix_last_beat {
            self.matrix_beat_count = self.matrix_beat_count.wrapping_add(1);
        }
        self.matrix_last_beat = hhd;
        
        let jqm = axy && (self.matrix_beat_count % 8 == 7);
        
        
        framebuffer::fill_rect(0, 0, width, height, 0xFF010200);
        
        let eyf = height * 88 / 100;
        if eyf < height {
            framebuffer::fill_rect(0, eyf, width, height - eyf, 0xFF020300);
        }
        
        let gpm = framebuffer::FrameCtx::snapshot();
        
        
        
        
        
        if self.desktop_tier >= DesktopTier::Standard
            && (self.desktop_tier >= DesktopTier::Full || self.frame_count % 3 == 0)
        {
            let owb = self.frame_count as u32;
            let step = 12u32; 
            let mut ak = 0u32;
            while ak < height {
                let mut am = 0u32;
                while am < width {
                    
                    let h = (am.wrapping_mul(2654435761)).wrapping_add(ak.wrapping_mul(340573321));
                    let h = h ^ (h >> 16);
                    if h % 97 == 0 {
                        
                        let fh = (h >> 8) % step;
                        let hk = (h >> 14) % step;
                        let p = am + fh;
                        let o = ak + hk;
                        if p < width && o < height && o < eyf {
                            
                            let phase = owb.wrapping_add(h & 0xFF).wrapping_mul(3);
                            let pnp = ((phase & 255) as i32 - 128).unsigned_abs(); 
                            let cml = 40 + (pnp * 60 / 128) as u32; 
                            let c = 0xFF000000 | (cml << 16) | (cml << 8) | cml;
                            gpm.put_pixel(p, o, c);
                        }
                    }
                    am += step;
                }
                ak += step;
            }
        }
        
        if !self.matrix_initialized {
            return;
        }
        
        
        
        if self.frame_count < 5 { crate::serial_println!("[FRAME] #{} start", self.frame_count); }
        
        
        
        
        if self.desktop_tier >= DesktopTier::Full {
            crate::visualizer::update(
                &mut self.visualizer,
                width, self.height,
                matrix_cols,
                bud, cmm,
                eto, etm, etn, etp,
                axy,
            );
        }
        
        if self.frame_count < 5 { crate::serial_println!("[FRAME] #{} viz done", self.frame_count); }
        
        
        if self.desktop_tier >= DesktopTier::Full {
            crate::drone_swarm::update(&mut self.drone_swarm);
        }
        
        
        let ilc = height / 2;
        let nan = width / 2;
        
        let emy = 300.0f32;
        let hzi = 250.0f32; 
        
        
        
        
        let ati = (width + matrix_cols as u32 - 1) / matrix_cols as u32;
        let idf = matrix_cols as f32 / 2.0;
        
        let flow_time = self.frame_count as f32 * 0.008;
        
        if self.frame_count < 5 { crate::serial_println!("[FRAME] #{} rain start", self.frame_count); }
        
        let (fb_ptr, fb_stride, _fb_height) = framebuffer::lyk();
        let mjc = !self.matrix_overrides.is_empty();
        for bj in 0..num_layers {
        
        
        let jkd = CHL_[bj];
        let ozd = match bj { 4 => 0.010f32, 5 => 0.014, 3 => 0.006, _ => 0.0 };
        let oze = if jkd > 0.0 {
            let phase = (self.frame_count as f32) * ozd;
            
            let sin = crate::graphics::holomatrix::azr;
            let afq = sin(phase);
            let azn = sin(phase * 1.7 + 2.0) * 0.4;
            ((afq + azn) * jkd) as i32
        } else { 0i32 };
        let hmo = if dsq { CKB_[bj] } else { CHF_[bj] };
        let jxy = CHE_[bj];
        let hft = CHD_[bj];
        let jxx = CHC_[bj];
        let hzh = CHH_[bj];
        
        let hzg = if self.visualizer.mode == 7 { hzh * 2.5 } else { hzh };
        let bbx = if dsq { CKD_[bj] } else { CHJ_[bj] };
        let bbw = if dsq { CKC_[bj] } else { CHI_[bj] };
        
        for col in 0..matrix_cols.min(self.matrix_heads.len() / num_layers.max(1)) {
            
            if hmo > 1 && (col % hmo) != 0 { continue; }
            
            let idx = col * num_layers.max(1) + bj;
            let speed = self.matrix_speeds[idx];
            let seed = self.matrix_seeds[idx];
            
            
            let cgf = (col as u32 * width) / matrix_cols as u32 + ati / 2;
            let x = (cgf as i32 + oze).max(0).min(width as i32 - 1) as u32;
            
            
            let gfc = CHM_[bj];
            let mxf: f32 = CHG_[bj];
            let ijp: f32 = mxi[bj];
            
            
            
            let bsw = ((col as f32) - idf).abs() / idf;
            let fxm = (bsw * bsw).min(1.0);
            
            let lxz = (fxm * (bbw as f32 * 0.15)) as u32;
            let hve: u32 = bbw + lxz;
            
            let lyb: i32 = (100.0 - fxm * 12.0) as i32;
            
            let lxy: f32 = 1.0 - fxm * 0.04;
            
            
            
            let lys = (self.matrix_seeds[col * num_layers.max(1)] >> 3) % 4;
            
            let (band_amp, band_r_base, band_g_base, band_b_base) = if axy {
                match lys {
                    0 => (eto,  0u8, 180u8, 0u8),   
                    1 => (etm,      0u8, 200u8, 0u8),   
                    2 => (etn,       0u8, 220u8, 0u8),   
                    _ => (etp,    0u8, 210u8, 0u8),   
                }
            } else {
                (0.0, 0u8, 200u8, 0u8) 
            };
            
            let lyu = if axy {
                (0.3 + band_amp * 1.2).min(1.5)
            } else { 1.0 };
            
            
            
            let kvj = jqm && ((col.wrapping_mul(7) ^ self.matrix_beat_count as usize) % 16 == 0);
            
            
            let fip = if axy {
                let t = (col as u32 * 2) % (matrix_cols as u32);
                let kvi = if t < matrix_cols as u32 {
                    t as f32 / matrix_cols as f32
                } else {
                    2.0 - t as f32 / matrix_cols as f32
                };
                let naj = bud * (0.5 + kvi * 0.5);
                (naj * 6.0 + band_amp * 4.0) as i32
            } else { 0 };
            
            
            
            let mej = crate::visualizer::hmz(&self.visualizer, col) as i32;
            let our = ((speed as f32) * ijp) as i32;
            let lol = (((our + fip) * mej / 100) * lyb / 100).max(1);
            let afk = self.matrix_heads[idx] + lol;
            if afk > height as i32 + (gfc as i32 * hve as i32) {
                let dbr = seed.wrapping_mul(1103515245).wrapping_add(12345);
                self.matrix_seeds[idx] = dbr;
                self.matrix_heads[idx] = -((dbr % (height / 3)) as i32);
                let chars: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
                for i in 0..gfc.min(EZ_) {
                    let cs = dbr.wrapping_add((i as u32).wrapping_mul(7919));
                    self.matrix_chars[idx * EZ_ + i] = chars[(cs as usize) % chars.len()];
                }
            } else {
                self.matrix_heads[idx] = afk;
            }
            
            
            if kvj { continue; }
            
            let head_y = self.matrix_heads[idx];
            
            
            let fns = (speed as f32) * ijp;
            let dzy = (fns / 5.0).min(1.0); 
            let lqf = if axy { cmm * 0.3 } else { 0.0 };
            let kaz = if axy { bud * 0.15 } else { 0.0 };
            let cgm = ((0.3 + dzy * 0.7 + lqf + kaz) * lxy * mxf).min(1.5);
            
            
            let mei = axy && col < self.visualizer.column_bounds.len() && {
                let (bmin, bmax) = self.visualizer.column_bounds[col];
                bmin >= 0 && bmax > bmin
            };
            
            
            
            let chl = ((col as u32).wrapping_mul(2654435761u32)) >> 20; 
            let kvk = 0.55 + (chl % 100) as f32 / 110.0; 
            let ouw = ((gfc as f32) * (0.5 + dzy * 0.5) * kvk) as usize;
            let hvf = ouw.max(4).min(EZ_);
            
            for i in 0..hvf {
                
                if bj == 0 && (i & 1) == 1 { continue; }
                
                let yl = head_y - (i as i32 * hve as i32);
                if yl < 0 || yl >= height as i32 { continue; }
                
                
                if jqm && (i % 5 == 0) && i > 3 { continue; }
                
                
                let jok = if axy { (cmm * 30.0) as u8 } else { 0 };
                let ltz = (200u8 as u16 / (hvf as u16).max(1)) as u8;
                
                let jhf = (dzy * 30.0) as u8;
                let base = if i == 0 { 255u8 }
                    else if i == 1 { (230u8 + jhf / 2).min(255).saturating_add(jok / 2) }
                    else { (210u8 + jok / 3 + jhf / 3).saturating_sub((i as u8).saturating_mul(ltz.max(3))) };
                if base < (if mei { 2 } else { 3 }) { continue; }
                
                let brightness = ((base as f32) * cgm).min(255.0) as u8;
                
                
                
                
                let (r, g, b) = if i == 0 {
                    
                    let ffd = (0.50 + dzy * 0.45).min(0.95);
                    let hgn = 1.0 - ffd;
                    
                    let czh = ((band_r_base as f32 * hgn + 180.0 * ffd) * cgm).min(190.0) as i16;
                    let cze = ((band_g_base as f32 * hgn + 255.0 * ffd) * cgm).min(255.0) as i16;
                    let czc = ((180.0 * ffd) * cgm).min(190.0) as i16;
                    let fiq = if axy { (bud * 8.0).min(15.0) as i16 } else { 0 };
                    
                    let ko = (czh + fiq / 4 + jxy).max(0).min(190) as u8;
                    let fg = (cze + fiq + hft).max(0).min(255) as u8;
                    let fb = (czc + fiq / 4 + jxx).max(0).min(190) as u8;
                    
                    let ko = ko.min(fg);
                    let fb = fb.min(fg);
                    (ko, fg, fb)
                } else {
                    
                    let ln = brightness as f32 / 255.0;
                    let axb = lyu;
                    if self.visualizer.palette == 23 {
                        
                        let (alg, ahp, cb) = crate::visualizer::oba(
                            col, i, self.matrix_seeds[idx],
                        );
                        let ko = (alg as f32 * ln * axb).min(255.0) as u8;
                        let fg = (ahp as f32 * ln * axb).min(255.0) as u8;
                        let fb = (cb as f32 * ln * axb).min(255.0) as u8;
                        (ko, fg, fb)
                    } else {
                        let ous = 0.8 + dzy * 0.4; 
                        let tr = 0i16; 
                        let bwi = ((band_g_base as f32 * axb * ln * ous).min(255.0)) as i16;
                        let aiv = 0i16; 
                        
                        let ko = 0u8; 
                        let fg = (bwi + hft).max(0).min(255) as u8;
                        let fb = 0u8; 
                        (ko, fg, fb)
                    }
                };
                
                
                let (mut r, mut g, mut b) = (r, g, b);
                let mut dqt: u8 = 0;
                let mut fzb: u8 = 128;
                
                
                if bj >= 1 {
                    let dg = crate::visualizer::fli(
                        &self.visualizer, col, yl,
                        self.visualizer.beat_pulse, cmm,
                    );
                    if dg.glow > 0 || dg.ripple > 0 || dg.fresnel > 0 || dg.specular > 0
                        || dg.scanline > 0 || dg.inner_glow > 0 || dg.shadow > 0 {
                        let (mr, ayf, aop) = crate::visualizer::gia(
                            r, g, b, dg.glow, dg.depth, dg.ripple,
                            dg.fresnel, dg.specular,
                            dg.ao, dg.bloom, dg.scanline, dg.inner_glow, dg.shadow,
                            bud, cmm,
                            eto, etm, etn, etp,
                            self.visualizer.palette,
                        );
                        r = mr; g = ayf; b = aop;
                        dqt = dg.trail_boost;
                        fzb = dg.depth;
                    }
                    
                    if dg.target_blend > 0 {
                        let t = dg.target_blend as f32 / 255.0;
                        let ki = 1.0 - t;
                        r = (r as f32 * ki + dg.target_r as f32 * t) as u8;
                        g = (g as f32 * ki + dg.target_g as f32 * t) as u8;
                        b = (b as f32 * ki + dg.target_b as f32 * t) as u8;
                        dqt = dg.trail_boost;
                    }
                    
                    if dg.dim > 0 {
                        let bts = 1.0 - (dg.dim as f32 / 255.0);
                        r = (r as f32 * bts) as u8;
                        g = (g as f32 * bts) as u8;
                        b = (b as f32 * bts) as u8;
                    }
                }
                
                if dqt > 0 {
                    let ahj = 1.0 + dqt as f32 / 100.0;
                    r = (r as f32 * ahj).min(255.0) as u8;
                    g = (g as f32 * ahj).min(255.0) as u8;
                    b = (b as f32 * ahj).min(255.0) as u8;
                }
                
                
                
                
                let lxb = if hzg > 0.0 && bj >= 3 {
                    let dx = x as f32 - nan as f32;
                    let ad = yl as f32 - ilc as f32;
                    let wz = dx * dx + ad * ad;
                    
                    let amn = emy * emy;
                    let isy = (emy + hzi) * (emy + hzi);
                    let boj = if wz < amn {
                        1.0
                    } else if wz < isy {
                        
                        1.0 - (wz - amn) / (isy - amn)
                    } else {
                        0.0
                    };
                    if boj > 0.01 {
                        let u = yl as f32;
                        let cx = col as f32;
                        let sin = crate::graphics::holomatrix::azr;
                        let ayt = sin(u * 0.0045 + cx * 0.13 + flow_time);
                        let ayu = sin(u * 0.012 + cx * 0.07 + flow_time * 1.6 + 3.0) * 0.4;
                        let ayv = sin(u * 0.028 + cx * 0.21 + flow_time * 2.3 + 1.5) * 0.15;
                        ((ayt + ayu + ayv) * hzg * boj) as i32
                    } else { 0 }
                } else { 0 };
                let cta = (x as i32 + lxb).max(0).min(width as i32 - 1) as u32;
                
                
                
                let cww = if bj >= 3 {
                    crate::drone_swarm::query(
                        &self.drone_swarm, cta as f32, yl as f32,
                    )
                } else {
                    crate::drone_swarm::DroneInteraction { brightness: 1.0, color_r: 0, color_g: 0, color_b: 0 }
                };
                if cww.brightness != 1.0 || cww.color_r != 0 {
                    let bqp = cww.brightness;
                    r = ((r as f32 * bqp).min(255.0)) as u8;
                    g = ((g as f32 * bqp).min(255.0)) as u8;
                    b = ((b as f32 * bqp).min(255.0)) as u8;
                    r = ((r as i16 + cww.color_r).max(0).min(255)) as u8;
                    g = ((g as i16 + cww.color_g).max(0).min(255)) as u8;
                    b = ((b as i16 + cww.color_b).max(0).min(255)) as u8;
                }
                
                let color = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                
                
                
                let igy = dqt > 30;
                let nhe = if igy { 10u32 } else { 28u32 };
                let bfe = seed.wrapping_add((i as u32 * 7919) ^ (self.frame_count as u32 / nhe));
                let chars: &[u8] = if igy {
                    if fzb > 180 {
                        
                        b"@#$%&WM8BOXZNHK"
                    } else if fzb < 80 {
                        
                        b".:;~-'`"
                    } else {
                        
                        b"0123456789ABCDEF"
                    }
                } else {
                    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|"
                };
                let c = chars[(bfe as usize) % chars.len()] as char;
                let du = crate::framebuffer::font::ol(c);
                
                
                
                let igc = self.matrix_projection.active
                    && cta + bbx > self.matrix_projection.x
                    && cta < self.matrix_projection.x + self.matrix_projection.width
                    && (yl as u32) + bbw > self.matrix_projection.y
                    && (yl as u32) < self.matrix_projection.y + self.matrix_projection.height;

                
                let hjz = idx * EZ_ + i;
                let mju = !igc && mjc && self.matrix_overrides.contains_key(&hjz);
                
                let alg = ((color >> 16) & 0xFF) as u8;
                let ahp = ((color >> 8) & 0xFF) as u8;
                let cb = (color & 0xFF) as u8;
                
                if igc {
                    
                    
                    
                    
                    let oa = &self.matrix_projection;
                    let intensity = brightness as f32 / 255.0;
                    for cag in 0..bbw as usize {
                        let o = yl as u32 + cag as u32;
                        if o >= height { continue; }
                        if o < oa.y || o >= oa.y + oa.height { continue; }
                        let scanline: f32 = if o & 1 == 0 { 1.0 } else { 0.96 };
                        let drw = o > height * 88 / 100;
                        let ckx = (o - oa.y) as usize;
                        for bf in 0..bbx {
                            let p = cta + bf;
                            if p >= width { continue; }
                            if p < oa.x || p >= oa.x + oa.width { continue; }
                            let ckw = (p - oa.x) as usize;
                            let bux = oa.pixels[ckx * oa.width as usize + ckw];
                            if bux & 0xFF000000 == 0 { continue; }
                            let ej = ((bux >> 16) & 0xFF) as f32;
                            let abe = ((bux >> 8) & 0xFF) as f32;
                            let ji = (bux & 0xFF) as f32;
                            let mut ko = (ej * intensity).min(255.0) as u8;
                            let mut fg = (abe * intensity).min(255.0) as u8;
                            let mut fb = (ji * intensity).min(255.0) as u8;
                            ko = ((ko as f32 * scanline).min(255.0)) as u8;
                            fg = ((fg as f32 * scanline).min(255.0)) as u8;
                            fb = ((fb as f32 * scanline).min(255.0)) as u8;
                            if drw {
                                fg = (fg as u16 + 10).min(255) as u8;
                            }
                            let br = 0xFF000000 | ((ko as u32) << 16) | ((fg as u32) << 8) | (fb as u32);
                            gpm.put_pixel(p, o, br);
                        }
                    }
                } else if mju {
                    
                    let khx = &self.matrix_overrides[&hjz];
                    for cag in 0..16usize {
                        let o = yl as u32 + cag as u32;
                        if o >= height { continue; }
                        let scanline: f32 = if o & 1 == 0 { 1.0 } else { 0.96 };
                        let drw = o > height * 88 / 100;
                        for bf in 0..8u32 {
                            let bux = khx.pixels[cag * 8 + bf as usize];
                            if bux & 0xFF000000 == 0 { continue; } 
                            let p = cta + bf;
                            if p >= width { continue; }
                            
                            let ej = ((bux >> 16) & 0xFF) as f32;
                            let abe = ((bux >> 8) & 0xFF) as f32;
                            let ji = (bux & 0xFF) as f32;
                            
                            
                            let intensity = brightness as f32 / 255.0;
                            let mut ko = (ej * intensity).min(255.0) as u8;
                            let mut fg = (abe * intensity).min(255.0) as u8;
                            let mut fb = (ji * intensity).min(255.0) as u8;
                            if bj > 0 {
                                fg = (fg as u16 + 30u16).min(255) as u8;
                            }
                            ko = ((ko as f32 * scanline).min(255.0)) as u8;
                            fg = ((fg as f32 * scanline).min(255.0)) as u8;
                            fb = ((fb as f32 * scanline).min(255.0)) as u8;
                            if drw {
                                fg = (fg as u16 + 10).min(255) as u8;
                            }
                            let br = 0xFF000000 | ((ko as u32) << 16) | ((fg as u32) << 8) | (fb as u32);
                            gpm.put_pixel(p, o, br);
                        }
                    }
                } else {
                    
                    
                    let lux = if bj > 0 { 30u16 } else { 0u16 };
                    let dhu = alg;
                    let fgn = (ahp as u16 + lux).min(255) as u8;
                    let ffv = cb;
                    let kvq = 0xFF000000 | ((dhu as u32) << 16) | ((fgn as u32) << 8) | (ffv as u32);
                    let pb = (dhu as u16 * 245 >> 8) as u8;
                    let akl = (fgn as u16 * 245 >> 8) as u8;
                    let cv = (ffv as u16 * 245 >> 8) as u8;
                    let kvr = 0xFF000000 | ((pb as u32) << 16) | ((akl as u32) << 8) | (cv as u32);
                    let odz = height * 88 / 100;
                    let ogv = (fgn as u16 + 10).min(255) as u8;
                    let ogw = (akl as u16 + 10).min(255) as u8;
                    let kvs = 0xFF000000 | ((dhu as u32) << 16) | ((ogv as u32) << 8) | (ffv as u32);
                    let kvt = 0xFF000000 | ((pb as u32) << 16) | ((ogw as u32) << 8) | (cv as u32);
                    let pqk = !fb_ptr.is_null();
                    for ak in 0..bbw {
                        let o = yl as u32 + ak;
                        if o >= height { continue; }
                        let amv = ((ak * 16) / bbw).min(15) as usize;
                        let bits = du[amv];
                        if bits == 0 { continue; }
                        let mth = o & 1 != 0;
                        let drw = o > odz;
                        let br = match (mth, drw) {
                            (false, false) => kvq,
                            (true, false) => kvr,
                            (false, true) => kvs,
                            (true, true) => kvt,
                        };
                        if pqk {
                            
                            let bop = o as usize * fb_stride as usize;
                            for am in 0..bbx {
                                let gvu = ((am * 8) / bbx).min(7);
                                if bits & (0x80 >> gvu) != 0 {
                                    let p = cta + am;
                                    if p < width {
                                        unsafe { *fb_ptr.add(bop + p as usize) = br; }
                                    }
                                }
                            }
                        } else {
                            for am in 0..bbx {
                                let gvu = ((am * 8) / bbx).min(7);
                                if bits & (0x80 >> gvu) != 0 {
                                    let p = cta + am;
                                    if p < width {
                                        framebuffer::cz(p, o, br);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } 
        } 
        
        
        
        
        
        
        if !dsq && self.desktop_tier >= DesktopTier::Full {
            const CLL_: usize = 4;
            const ADV_: usize = 16;
            const ADU_: u32 = 16;  
            
            for col in 0..matrix_cols.min(self.visualizer.column_bounds.len()) {
                
                let (bmin, bmax) = self.visualizer.column_bounds[col];
                if bmin < 0 || bmax <= bmin { continue; }
                
                let x = (col as u32 * width) / matrix_cols as u32 + ati / 2;
                for fill in 0..CLL_ {
                    let fws = (col as u32).wrapping_mul(2654435761)
                        ^ ((fill as u32 + 17).wrapping_mul(0x9E3779B9));
                    let lvi = 1 + (fws % 3);
                    
                    
                    let sn = (height + ADV_ as u32 * ADU_) as u32;
                    let obp = (self.frame_count as u32)
                        .wrapping_mul(lvi)
                        .wrapping_add(fws);
                    let psg = (obp % sn.max(1)) as i32
                        - (ADV_ as i32 * ADU_ as i32);
                    
                    for i in 0..ADV_ {
                        let yl = psg - (i as i32 * ADU_ as i32);
                        if yl < 0 || yl >= height as i32 { continue; }
                        
                        
                        let oq = 12i32;
                        if yl < bmin - oq || yl > bmax + oq { continue; }
                        
                        let dg = crate::visualizer::fli(
                            &self.visualizer, col, yl,
                            self.visualizer.beat_pulse, cmm,
                        );
                        
                        
                        if dg.glow == 0 && dg.target_blend == 0 { continue; }
                        
                        
                        let base = if i == 0 { 180u8 }
                            else { 120u8.saturating_sub((i as u8).saturating_mul(7)) };
                        if base < 10 { continue; }
                        
                        
                        let fse = (base as u32 / 8) as u8;
                        let fsd = (base as u32 / 3) as u8;
                        let fsc = (base as u32 / 7) as u8;
                        let (mut mr, mut ayf, mut aop) = crate::visualizer::gia(
                            fse, fsd, fsc,
                            dg.glow, dg.depth, dg.ripple,
                            dg.fresnel, dg.specular,
                            dg.ao, dg.bloom, dg.scanline, dg.inner_glow, dg.shadow,
                            bud, cmm,
                            eto, etm, etn, etp,
                            self.visualizer.palette,
                        );
                        
                        if dg.target_blend > 0 {
                            let t = dg.target_blend as f32 / 255.0;
                            let ki = 1.0 - t;
                            mr = (mr as f32 * ki + dg.target_r as f32 * t) as u8;
                            ayf = (ayf as f32 * ki + dg.target_g as f32 * t) as u8;
                            aop = (aop as f32 * ki + dg.target_b as f32 * t) as u8;
                        }
                        
                        let color = 0xFF000000 | ((mr as u32) << 16) | ((ayf as u32) << 8) | (aop as u32);
                        
                        
                        let cs = fws.wrapping_add(
                            (i as u32 * 7919) ^ (self.frame_count as u32 / 8)
                        );
                        let hyk: &[u8] = b"@#$%&WM8BOX0ZNHK";
                        let c = hyk[(cs as usize) % hyk.len()] as char;
                        let du = crate::framebuffer::font::ol(c);
                        
                        for (cag, &bits) in du.iter().enumerate() {
                            let o = yl as u32 + cag as u32;
                            if o >= height || bits == 0 { continue; }
                            if !fb_ptr.is_null() {
                                let bop = o as usize * fb_stride as usize;
                                for bf in 0..8u32 {
                                    if bits & (0x80 >> bf) != 0 {
                                        let p = x + bf;
                                        if p < width {
                                            unsafe { *fb_ptr.add(bop + p as usize) = color; }
                                        }
                                    }
                                }
                            } else {
                                for bf in 0..8u32 {
                                    if bits & (0x80 >> bf) != 0 {
                                        let p = x + bf;
                                        if p < width {
                                            framebuffer::cz(p, o, color);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if self.frame_count < 5 { crate::serial_println!("[FRAME] #{} rain+fill done", self.frame_count); }
        
        
        
        if self.frame_count < 5 { crate::serial_println!("[FRAME] #{} logo start", self.frame_count); }
        
        
        
        if !dsq {
            let ark = crate::logo_bitmap::BA_ as u32;
            let arj = crate::logo_bitmap::BN_ as u32;
            let cbn = (width / 2).saturating_sub(ark / 2);
            let cbo = ilc.saturating_sub(arj / 2);
            
            
            
            
            if self.desktop_tier >= DesktopTier::Full && self.frame_count % 4 == 0 {
                for ly in (0..arj).step_by(2) {
                    for fe in 0..ark {
                        if !crate::logo_bitmap::dtu(fe as usize, ly as usize) { continue; }
                        let p = cbn + fe;
                        let o = cbo + ly;
                        if p >= width || o >= height { continue; }
                        let fze: u32 = if axy { 35 + (bud * 50.0) as u32 } else { 30 };
                        
                        framebuffer::cz(p, o, 0xFF000000 | (fze.min(255) << 8));
                    }
                }
            }
            
            
            for ly in 0..arj {
                for fe in 0..ark {
                    let abq = crate::logo_bitmap::bhr(fe as usize, ly as usize);
                    let a = (abq >> 24) & 0xFF;
                    let r = (abq >> 16) & 0xFF;
                    let g = (abq >> 8) & 0xFF;
                    let b = abq & 0xFF;
                    
                    if a < 20 { continue; }
                    let dtw = (r * 77 + g * 150 + b * 29) >> 8;
                    if dtw < 30 { continue; }
                    
                    let p = cbn + fe;
                    let o = cbo + ly;
                    if p >= width || o >= height { continue; }
                    
                    if dtw >= 60 {
                        let fio = if axy { (bud * 20.0).min(30.0) as u32 } else { 0 };
                        let ej = (r + fio).min(255);
                        let abe = (g + fio).min(255);
                        let ji = (b + fio).min(255);
                        framebuffer::cz(p, o, 0xFF000000 | (ej << 16) | (abe << 8) | ji);
                    } else {
                        let alpha = ((dtw as u32) * 255 / 60).min(255);
                        let bg = framebuffer::fyu(p, o);
                        let ki = 255 - alpha;
                        let nr = (r * alpha + ((bg >> 16) & 0xFF) * ki) / 255;
                        let ayn = (g * alpha + ((bg >> 8) & 0xFF) * ki) / 255;
                        let ayj = (b * alpha + (bg & 0xFF) * ki) / 255;
                        framebuffer::cz(p, o, 0xFF000000 | (nr << 16) | (ayn << 8) | ayj);
                    }
                }
            }
        }
        
        
    }
    
    
    fn qee(&self, wp_data: &crate::theme::Jg, screen_height: u32) {
        use crate::theme::WallpaperMode;
        let mode = crate::theme::Dj.read().wallpaper.mode;
        let hcn = framebuffer::FrameCtx::snapshot();
        
        match mode {
            WallpaperMode::Stretch => {
                
                let dgs = wp_data.width;
                let jrj = wp_data.height;
                let ddw = self.width;
                
                for ak in 0..screen_height {
                    
                    let jhu = (ak as u64 * ((jrj as u64 - 1) << 8)) / screen_height as u64;
                    let az = (jhu >> 8) as u32;
                    let y1 = (az + 1).min(jrj - 1);
                    let hj = (jhu & 0xFF) as u32; 
                    let czm = 256 - hj;
                    
                    for am in 0..ddw {
                        let jht = (am as u64 * ((dgs as u64 - 1) << 8)) / ddw as u64;
                        let bm = (jht >> 8) as u32;
                        let x1 = (bm + 1).min(dgs - 1);
                        let dg = (jht & 0xFF) as u32;
                        let czl = 256 - dg;
                        
                        
                        let mmx = (az * dgs + bm) as usize;
                        let mmz = (az * dgs + x1) as usize;
                        let mmy = (y1 * dgs + bm) as usize;
                        let ifi = (y1 * dgs + x1) as usize;
                        
                        if ifi < wp_data.pixels.len() {
                            let anq = wp_data.pixels[mmx];
                            let apx = wp_data.pixels[mmz];
                            let anr = wp_data.pixels[mmy];
                            let apy = wp_data.pixels[ifi];
                            
                            
                            let r = ( ((anq >> 16) & 0xFF) * czl * czm
                                    + ((apx >> 16) & 0xFF) * dg * czm
                                    + ((anr >> 16) & 0xFF) * czl * hj
                                    + ((apy >> 16) & 0xFF) * dg * hj ) >> 16;
                            let g = ( ((anq >> 8) & 0xFF) * czl * czm
                                    + ((apx >> 8) & 0xFF) * dg * czm
                                    + ((anr >> 8) & 0xFF) * czl * hj
                                    + ((apy >> 8) & 0xFF) * dg * hj ) >> 16;
                            let b = ( (anq & 0xFF) * czl * czm
                                    + (apx & 0xFF) * dg * czm
                                    + (anr & 0xFF) * czl * hj
                                    + (apy & 0xFF) * dg * hj ) >> 16;
                            
                            hcn.put_pixel(am, ak, 0xFF000000 | (r << 16) | (g << 8) | b);
                        }
                    }
                }
            }
            WallpaperMode::Center => {
                
                let bg = crate::theme::colors().background;
                framebuffer::fill_rect(0, 0, self.width, screen_height, bg);
                
                let bny = self.width.saturating_sub(wp_data.width) / 2;
                let bnz = screen_height.saturating_sub(wp_data.height) / 2;
                
                for y in 0..wp_data.height.min(screen_height) {
                    for x in 0..wp_data.width.min(self.width) {
                        let idx = (y * wp_data.width + x) as usize;
                        if idx < wp_data.pixels.len() {
                            hcn.put_pixel(bny + x, bnz + y, wp_data.pixels[idx]);
                        }
                    }
                }
            }
            WallpaperMode::Tile => {
                
                let mut ad = 0;
                while ad < screen_height {
                    let mut dx = 0;
                    while dx < self.width {
                        for y in 0..wp_data.height {
                            if ad + y >= screen_height { break; }
                            for x in 0..wp_data.width {
                                if dx + x >= self.width { break; }
                                let idx = (y * wp_data.width + x) as usize;
                                if idx < wp_data.pixels.len() {
                                    hcn.put_pixel(dx + x, ad + y, wp_data.pixels[idx]);
                                }
                            }
                        }
                        dx += wp_data.width;
                    }
                    ad += wp_data.height;
                }
            }
            _ => {
                
                let color = crate::theme::Dj.read().wallpaper.fallback_color;
                framebuffer::fill_rect(0, 0, self.width, screen_height, color);
            }
        }
    }
    
    
    
    
    fn qdv(&self) {
        let center_x = self.width / 2;
        let center_y = (self.height - V_()) / 2 - 30;
        
        
        let bby = 0xFF50E050u32;  
        let mgb = 0xFF1A6B1Au32;    
        let fjh = 0xFF080808u32;     
        let jsd = 0xFFC0E020u32;  
        let dkr = 0xFF40C040u32; 
        let iqs = 0xFF60E060u32;     
        let rad = 0xFF999999u32;      
        let rae = 0xFF40CC40u32;     
        let gli = 0xFF30A030u32; 
        
        
        let apd = 80u32;
        let bje = 100u32;
        let am = center_x - apd / 2;
        let ak = center_y - bje / 2;
        
        for y in 0..bje {
            let zi = y as f32 / bje as f32;
            let hch = if zi < 0.45 {
                1.0
            } else {
                let t = (zi - 0.45) / 0.55;
                (1.0 - t * t).max(0.0)
            };
            let w = (apd as f32 * hch).max(2.0) as u32;
            let aam = (apd - w) / 2;
            
            for dx in 0..w {
                let p = am + aam + dx;
                let o = ak + y;
                
                let afh = aam + dx;
                let fsb = (afh as f32 / apd as f32) + (zi * 0.2);
                let fill = if fsb < 0.5 { fjh } else { mgb };
                framebuffer::cz(p, o, fill);
            }
            
            
            if w > 2 {
                framebuffer::cz(am + aam, ak + y, gli);
                framebuffer::cz(am + aam + w - 1, ak + y, gli);
            }
        }
        
        framebuffer::mn(am, ak, apd, gli);
        
        
        let bnj = center_x;
        let dtr = ak + 30;
        
        for ad in 0..14u32 {
            for dx in 0..20u32 {
                let lh = dx as i32 - 10;
                let kf = ad as i32;
                let amm = 10i32;
                let boi = 6i32;
                if kf <= amm && (lh * lh + (kf - amm) * (kf - amm)) <= amm * amm
                   && (lh * lh + (kf - amm) * (kf - amm)) >= boi * boi {
                    framebuffer::cz(bnj - 10 + dx, dtr - 14 + ad, jsd);
                }
            }
        }
        
        framebuffer::fill_rect(bnj - 12, dtr, 24, 18, jsd);
        
        for ad in 0..6u32 {
            for dx in 0..6u32 {
                let lh = dx as i32 - 3;
                let kf = ad as i32 - 3;
                if lh * lh + kf * kf <= 9 {
                    framebuffer::cz(bnj - 3 + dx, dtr + 4 + ad, fjh);
                }
            }
        }
        
        framebuffer::fill_rect(bnj - 1, dtr + 9, 3, 5, fjh);
        
        
        let clq = dtr + 18;
        let gen = ak + bje + 50;
        
        
        for ky in clq..gen {
            framebuffer::cz(bnj - 1, ky, dkr);
            framebuffer::cz(bnj, ky, dkr);
            framebuffer::cz(bnj + 1, ky, dkr);
        }
        
        
        let fjs: &[(u32, i32, u32)] = &[
            (clq + 8, -20, 6),   
            (clq + 8, 18, 6),    
            (clq + 22, -25, 5),  
            (clq + 22, 22, 5),   
            (clq + 36, -15, 4),  
            (clq + 36, 15, 4),   
        ];
        
        for &(dc, bx_off, node_r) in fjs {
            if dc >= self.height.saturating_sub(V_()) { continue; }
            
            let dzm: i32 = if bx_off < 0 { -1 } else { 1 };
            let jtf = if bx_off < 0 { -bx_off } else { bx_off };
            for dx in 0..jtf {
                let p = (bnj as i32 + dzm * dx) as u32;
                if p < self.width {
                    framebuffer::cz(p, dc, dkr);
                    framebuffer::cz(p, dc + 1, dkr);
                }
            }
            
            let nko = (bnj as i32 + bx_off) as u32;
            for ndy in 0..node_r {
                for ndx in 0..node_r {
                    let lh = ndx as i32 - node_r as i32 / 2;
                    let kf = ndy as i32 - node_r as i32 / 2;
                    if lh * lh + kf * kf <= (node_r as i32 / 2) * (node_r as i32 / 2) {
                        let p = nko + ndx;
                        let o = dc + ndy;
                        if p < self.width && o < self.height.saturating_sub(V_()) {
                            framebuffer::cz(p, o, iqs);
                        }
                    }
                }
            }
        }
        
        
        if gen + 4 < self.height.saturating_sub(V_()) {
            for ad in 0..8u32 {
                for dx in 0..8u32 {
                    let lh = dx as i32 - 4;
                    let kf = ad as i32 - 4;
                    if lh * lh + kf * kf <= 16 {
                        framebuffer::cz(bnj - 4 + dx, gen + ad, iqs);
                    }
                }
            }
        }
        
    }
    
    fn draw_desktop_icons(&self) {
        
        
        
        
        let atn = self.height.saturating_sub(V_());
        let htb = framebuffer::FrameCtx::snapshot();
        
        
        
        for ad in 0..atn {
            for dx in 0..(BW_() + 10) {
                let ku = htb.get_pixel(dx, ad);
                let ajp = ((ku >> 16) & 0xFF) as u32;
                let cir = ((ku >> 8) & 0xFF) as u32;
                let bsd = (ku & 0xFF) as u32;
                
                let nr = (ajp * 25 / 100 + 4 * 75 / 100).min(255);
                let ayn = (cir * 25 / 100 + 8 * 75 / 100).min(255);
                let ayj = (bsd * 25 / 100 + 4 * 75 / 100).min(255);
                htb.put_pixel(dx, ad, 0xFF000000 | (nr << 16) | (ayn << 8) | ayj);
            }
        }
        
        framebuffer::fill_rect(BW_() + 9, 0, 1, atn, AP_);
        
        let rl = 36u32;
        let cbu = self.icons.len().max(1) as u32;
        let padding = 12u32;
        let available = atn.saturating_sub(padding * 2);
        let axn = available / cbu;
        let start_y = padding + (available - axn * cbu) / 2;
        
        for (i, icon) in self.icons.iter().enumerate() {
            let bi = 12u32;
            let gg = start_y + (i as u32) * axn;
            if gg + rl > atn { break; }
            
            
            let vl = self.cursor_x >= 0 && self.cursor_x < (BW_() + 10) as i32
                && self.cursor_y >= gg as i32 && self.cursor_y < (gg + axn) as i32;
            
            
            let icon_color = if vl { I_ } else { BJ_ };
            let ace = if vl { I_ } else { 0xFF556655 };
            
            
            if vl {
                
                let bbv = 6u32;
                let hc = bi.saturating_sub(bbv);
                let jh = gg.saturating_sub(bbv);
                let fz = rl + bbv * 2;
                let agl = rl + 20 + bbv * 2;
                for gdy in 0..agl {
                    for gdx in 0..fz {
                        let p = hc + gdx;
                        let o = jh + gdy;
                        if p >= BW_() + 10 || o >= atn { continue; }
                        
                        let lp = if gdx < bbv { bbv - gdx } 
                            else if gdx > fz - bbv { gdx - (fz - bbv) } 
                            else { 0 };
                        let eqr = if gdy < bbv { bbv - gdy }
                            else if gdy > agl - bbv { gdy - (agl - bbv) }
                            else { 0 };
                        let em = lp.max(eqr);
                        if em > 0 {
                            let intensity = (20u32.saturating_sub(em * 4)).min(20) as u8;
                            if intensity > 0 {
                                let ku = framebuffer::fyu(p, o);
                                let cir = ((ku >> 8) & 0xFF) as u8;
                                let cnb = cir.saturating_add(intensity);
                                let bex = (ku & 0xFFFF00FF) | ((cnb as u32) << 8);
                                framebuffer::cz(p, o, bex);
                            }
                        }
                    }
                }
                
                draw_rounded_rect((bi as i32) - 3, (gg as i32) - 2, rl + 6, rl + 16, 6, 0xFF001A0A);
                iu((bi as i32) - 3, (gg as i32) - 2, rl + 6, rl + 16, 6, GR_);
            }
            
            
            let zr = match icon.icon_type {
                IconType::Terminal => 0xFF20CC60u32,  
                IconType::Folder => 0xFFDDAA30u32,    
                IconType::Editor => 0xFF5090E0u32,    
                IconType::Calculator => 0xFFCC6633u32, 
                IconType::Network => 0xFF40AADDu32,    
                IconType::Game => 0xFFCC4444u32,       
                IconType::Chess => 0xFFEECC88u32,      
                IconType::Settings => 0xFF9988BBu32,   
                IconType::Browser => 0xFF4488DDu32,    
                IconType::GameBoy => 0xFF88BB44u32,    
                _ => icon_color,
            };
            
            draw_rounded_rect(bi as i32, gg as i32, rl, rl, 6, 0xFF060A06);
            if vl {
                
                iu(bi as i32, gg as i32, rl, rl, 6, zr);
            } else {
                iu(bi as i32, gg as i32, rl, rl, 6, AP_);
            }
            
            
            let oy = if vl { zr } else { icon_color };
            
            
            let cx = bi + rl / 2;
            let u = gg + rl / 2;
            use crate::icons::IconType;
            match icon.icon_type {
                IconType::Terminal => {
                    
                    
                    draw_rounded_rect((cx - 15) as i32, (u - 11) as i32, 30, 22, 3, oy);
                    
                    framebuffer::fill_rect(cx - 13, u - 9, 26, 16, 0xFF050A05);
                    
                    framebuffer::mn(cx - 13, u - 9, 26, zr);
                    
                    self.draw_text((cx - 9) as i32, (u - 5) as i32, "$", 0xFF40FF60);
                    framebuffer::fill_rect(cx - 3, u - 3, 8, 2, 0xFF40FF60);
                    
                    framebuffer::fill_rect(cx - 3, u + 8, 6, 3, oy);
                    framebuffer::fill_rect(cx - 6, u + 10, 12, 2, oy);
                },
                IconType::Folder => {
                    
                    
                    draw_rounded_rect((cx - 14) as i32, (u - 10) as i32, 14, 6, 2, oy);
                    
                    draw_rounded_rect((cx - 14) as i32, (u - 5) as i32, 28, 18, 2, oy);
                    
                    framebuffer::fill_rect(cx - 12, u - 3, 24, 13, 0xFF0A0A06);
                    
                    framebuffer::fill_rect(cx - 8, u, 14, 1, 0xFF404020);
                    framebuffer::fill_rect(cx - 8, u + 3, 10, 1, 0xFF404020);
                    framebuffer::fill_rect(cx - 8, u + 6, 16, 1, 0xFF404020);
                    
                    framebuffer::fill_rect(cx - 2, u - 5, 4, 2, zr);
                },
                IconType::Editor => {
                    
                    
                    draw_rounded_rect((cx - 11) as i32, (u - 13) as i32, 22, 26, 2, oy);
                    
                    framebuffer::fill_rect(cx + 5, u - 13, 6, 6, 0xFF0A0A0A);
                    framebuffer::mn(cx + 5, u - 13, 1, oy);
                    framebuffer::zv(cx + 5, u - 13, 6, oy);
                    framebuffer::mn(cx + 5, u - 8, 6, oy);
                    
                    framebuffer::fill_rect(cx - 9, u - 7, 18, 18, 0xFF080C08);
                    
                    for row in 0..5u32 {
                        framebuffer::fill_rect(cx - 8, u - 5 + row * 3, 2, 1, 0xFF335533);
                    }
                    
                    framebuffer::fill_rect(cx - 4, u - 5, 7, 1, 0xFF6688CC);  
                    framebuffer::fill_rect(cx - 4, u - 2, 10, 1, oy);  
                    framebuffer::fill_rect(cx - 4, u + 1, 6, 1, 0xFFCC8844);  
                    framebuffer::fill_rect(cx - 4, u + 4, 12, 1, oy);  
                    framebuffer::fill_rect(cx - 4, u + 7, 5, 1, 0xFF88BB44);  
                },
                IconType::Calculator => {
                    
                    draw_rounded_rect((cx - 11) as i32, (u - 13) as i32, 22, 26, 3, oy);
                    
                    framebuffer::fill_rect(cx - 9, u - 11, 18, 22, 0xFF0C0C0A);
                    
                    draw_rounded_rect((cx - 8) as i32, (u - 10) as i32, 16, 7, 1, 0xFF1A3320);
                    self.draw_text((cx - 5) as i32, (u - 10) as i32, "42", 0xFF40FF40);
                    
                    for row in 0..3u32 {
                        for col in 0..4u32 {
                            let bx = cx - 8 + col * 5;
                            let dc = u - 0 + row * 4;
                            let keh = if col == 3 { zr } else { oy };
                            framebuffer::fill_rect(bx, dc, 3, 2, keh);
                        }
                    }
                },
                IconType::Network => {
                    
                    let jxi = cx as i32;
                    let hfo = (u + 6) as i32;
                    
                    for dq in 0..3u32 {
                        let r = 5 + dq * 4;
                        let ju = (r * r) as i32;
                        let exn = ((r.saturating_sub(2)) * (r.saturating_sub(2))) as i32;
                        for ad in -(r as i32)..=0 {
                            for dx in -(r as i32)..=(r as i32) {
                                let bgb = dx * dx + ad * ad;
                                if bgb <= ju && bgb >= exn {
                                    let p = (jxi + dx) as u32;
                                    let o = (hfo + ad) as u32;
                                    if p >= bi && p < bi + rl && o >= gg && o < gg + rl {
                                        let color = if dq == 0 { 
                                            if vl { zr } else { Q_ }
                                        } else if dq == 1 { 
                                            if vl { zr } else { BJ_ }
                                        } else { 
                                            oy 
                                        };
                                        framebuffer::cz(p, o, color);
                                    }
                                }
                            }
                        }
                    }
                    
                    for ad in -1..=1i32 {
                        for dx in -1..=1i32 {
                            if dx*dx+ad*ad <= 1 {
                                framebuffer::cz((cx as i32 + dx) as u32, (hfo + ad) as u32, oy);
                            }
                        }
                    }
                },
                IconType::Game => {
                    
                    
                    draw_rounded_rect((cx - 15) as i32, (u - 6) as i32, 30, 16, 5, oy);
                    
                    framebuffer::fill_rect(cx - 13, u - 4, 26, 12, 0xFF0A0A0A);
                    
                    draw_rounded_rect((cx - 15) as i32, (u - 2) as i32, 6, 10, 2, oy);
                    
                    draw_rounded_rect((cx + 9) as i32, (u - 2) as i32, 6, 10, 2, oy);
                    
                    framebuffer::fill_rect(cx - 10, u - 1, 7, 2, oy);
                    framebuffer::fill_rect(cx - 8, u - 3, 2, 7, oy);
                    
                    framebuffer::fill_rect(cx + 4, u - 3, 3, 3, 0xFF4488DD);  
                    framebuffer::fill_rect(cx + 8, u - 1, 3, 3, DJ_);  
                    framebuffer::fill_rect(cx + 4, u + 1, 3, 3, 0xFF44DD44);  
                    framebuffer::fill_rect(cx + 1, u - 1, 3, 3, 0xFFDDDD44);  
                },
                IconType::Chess => {
                    
                    let pc = if vl { 0xFFFFDD88 } else { oy };
                    
                    framebuffer::fill_rect(cx - 8, u + 6, 16, 4, pc);
                    
                    framebuffer::fill_rect(cx - 6, u + 2, 12, 4, pc);
                    
                    framebuffer::fill_rect(cx - 4, u - 6, 8, 8, pc);
                    
                    framebuffer::fill_rect(cx - 6, u - 10, 3, 5, pc);
                    framebuffer::fill_rect(cx - 1, u - 12, 2, 7, pc);
                    framebuffer::fill_rect(cx + 3, u - 10, 3, 5, pc);
                    
                    framebuffer::fill_rect(cx - 1, u - 14, 2, 4, pc);
                    framebuffer::fill_rect(cx - 2, u - 13, 4, 2, pc);
                },
                IconType::Settings => {
                    
                    for ad in 0..20u32 {
                        for dx in 0..20u32 {
                            let lh = dx as i32 - 10;
                            let kf = ad as i32 - 10;
                            let wz = lh * lh + kf * kf;
                            
                            if wz >= 36 && wz <= 72 {
                                framebuffer::cz(cx - 10 + dx, u - 10 + ad, oy);
                            }
                            
                            if wz <= 12 {
                                framebuffer::cz(cx - 10 + dx, u - 10 + ad, oy);
                            }
                            
                            if wz >= 10 && wz <= 16 {
                                framebuffer::cz(cx - 10 + dx, u - 10 + ad, zr);
                            }
                        }
                    }
                    
                    let ebi: &[(i32, i32)] = &[(0, -10), (0, 10), (-10, 0), (10, 0), (-7, -7), (7, -7), (-7, 7), (7, 7)];
                    for &(bu, ty) in ebi {
                        let p = (cx as i32 + bu) as u32;
                        let o = (u as i32 + ty) as u32;
                        framebuffer::fill_rect(p.saturating_sub(2), o.saturating_sub(1), 4, 3, oy);
                    }
                },
                IconType::Browser => {
                    
                    for ad in 0..22u32 {
                        for dx in 0..22u32 {
                            let lh = dx as i32 - 11;
                            let kf = ad as i32 - 11;
                            let wz = lh * lh + kf * kf;
                            
                            if wz <= 110 {
                                framebuffer::cz(cx - 11 + dx, u - 11 + ad, 0xFF0A1A2A);
                            }
                            
                            if wz >= 100 && wz <= 121 {
                                framebuffer::cz(cx - 11 + dx, u - 11 + ad, oy);
                            }
                        }
                    }
                    
                    framebuffer::fill_rect(cx - 10, u, 20, 1, oy);
                    
                    framebuffer::fill_rect(cx, u - 10, 1, 20, oy);
                    
                    for ad in 0..20u32 {
                        let kf = ad as i32 - 10;
                        let val = 100 - kf * kf;
                        if val > 0 {
                            let ejp = (cxr(val) * 2 / 5) as u32;
                            if cx + ejp < bi + rl {
                                framebuffer::cz(cx + ejp, u - 10 + ad, oy);
                            }
                            if cx >= ejp + bi {
                                framebuffer::cz(cx.saturating_sub(ejp), u - 10 + ad, oy);
                            }
                        }
                    }
                    
                    framebuffer::fill_rect(cx - 9, u - 5, 18, 1, oy);
                    framebuffer::fill_rect(cx - 9, u + 5, 18, 1, oy);
                },
                IconType::GameBoy => {
                    
                    draw_rounded_rect((cx - 10) as i32, (u - 13) as i32, 20, 26, 3, oy);
                    framebuffer::fill_rect(cx - 8, u - 11, 16, 22, 0xFF1A1A1A);
                    
                    draw_rounded_rect((cx - 7) as i32, (u - 10) as i32, 14, 11, 1, 0xFF1A3320);
                    
                    framebuffer::fill_rect(cx - 2, u - 8, 4, 4, 0xFF40CC40);
                    framebuffer::fill_rect(cx - 3, u - 4, 6, 2, 0xFF40CC40);
                    
                    framebuffer::fill_rect(cx - 7, u + 4, 5, 2, 0xFF333333);
                    framebuffer::fill_rect(cx - 5, u + 2, 2, 6, 0xFF333333);
                    
                    framebuffer::fill_rect(cx + 3, u + 3, 3, 3, DJ_);
                    framebuffer::fill_rect(cx + 1, u + 5, 3, 3, 0xFF4488DD);
                    
                    for i in 0..3u32 {
                        framebuffer::fill_rect(cx + 2 + i * 3, u + 10, 1, 2, 0xFF333333);
                    }
                },
                IconType::About => {
                    
                    for ad in 0..20u32 {
                        for dx in 0..20u32 {
                            let lh = dx as i32 - 10;
                            let kf = ad as i32 - 10;
                            let wz = lh * lh + kf * kf;
                            if wz >= 72 && wz <= 100 {
                                framebuffer::cz(cx - 10 + dx, u - 10 + ad, oy);
                            }
                        }
                    }
                    
                    framebuffer::fill_rect(cx - 1, u - 6, 2, 2, zr); 
                    framebuffer::fill_rect(cx - 1, u - 2, 2, 8, zr); 
                    framebuffer::fill_rect(cx - 3, u + 5, 6, 1, zr); 
                },
                IconType::ModelEditor => {
                    
                    
                    let cxn = cx as i32 - 8;
                    let cxo = u as i32 - 2;
                    framebuffer::fill_rect(cxn as u32, cxo as u32, 14, 12, 0xFF162016);
                    framebuffer::draw_rect(cxn as u32, cxo as u32, 14, 12, oy);
                    
                    for i in 0..14i32 {
                        framebuffer::cz((cxn + i + 4) as u32, (cxo - 4) as u32, oy);
                        framebuffer::cz((cxn + i + 2) as u32, (cxo - 2) as u32, zr);
                    }
                    
                    framebuffer::zv((cxn + 17) as u32, (cxo - 4) as u32, 12, oy);
                    for ay in 0..4u32 {
                        framebuffer::cz((cxn + 14 + ay as i32) as u32, (cxo + ay as i32 - 4) as u32, oy);
                    }
                },
                IconType::GameLab => {
                    
                    framebuffer::fill_rect(cx - 3, u - 12, 6, 8, oy); 
                    framebuffer::fill_rect(cx - 5, u - 12, 10, 2, oy); 
                    
                    for row in 0..10u32 {
                        let nk = 3 + row;
                        framebuffer::fill_rect(cx.saturating_sub(nk), u - 4 + row, nk * 2, 1, oy);
                    }
                    
                    for row in 4..10u32 {
                        let nk = row;
                        framebuffer::fill_rect(cx.saturating_sub(nk) + 1, u - 4 + row, (nk * 2).saturating_sub(2), 1, zr);
                    }
                    
                    framebuffer::fill_rect(cx - 2, u + 1, 2, 2, 0xFF80FF80);
                    framebuffer::fill_rect(cx + 1, u + 3, 2, 2, 0xFF80FF80);
                },
                _ => {
                    
                    iu((cx - 10) as i32, (u - 10) as i32, 20, 20, 3, oy);
                    for i in 0..6i32 {
                        framebuffer::cz((cx as i32 + i) as u32, (u as i32 - i) as u32, zr);
                        framebuffer::cz((cx as i32 - i) as u32, (u as i32 - i) as u32, zr);
                        framebuffer::cz((cx as i32 + i) as u32, (u as i32 + i) as u32, zr);
                        framebuffer::cz((cx as i32 - i) as u32, (u as i32 + i) as u32, zr);
                    }
                },
            }
            
            
            let name = &icon.name;
            let acy = name.len() as u32 * 8;
            let kd = bi + (rl / 2).saturating_sub(acy / 2);
            self.draw_text_smooth(kd as i32, (gg + rl + 2) as i32, name, ace);
        }
    }
    
    fn draw_taskbar(&mut self) {
        let y = self.height - V_();
        
        
        
        
        
        
        {
            let radius = 6u32;
            let dk = radius as i32;
            let ju = dk * dk;
            let w = self.width;
            
            
            for row in 0..radius {
                let eds = dk - row as i32;
                let epk = cxr(ju - eds * eds) as u32;
                let iju = radius - epk;
                let jqf = w.saturating_sub(iju * 2);
                if jqf > 0 {
                    framebuffer::co(iju, y + row, jqf, 1, 0x040A06, 165);
                }
            }
            
            framebuffer::co(0, y + radius, w, V_() - radius, 0x040A06, 165);
            
            framebuffer::co(0, y, w, V_(), 0x00AA44, 10);
            
            if w > radius * 2 {
                for p in radius..(w - radius) {
                    framebuffer::cz(p, y, AW_);
                }
            }
            
            
            for row in 0..radius {
                let eds = dk - row as i32;
                let epk = cxr(ju - eds * eds) as u32;
                let aue = radius - epk;
                let asa = w - radius + epk;
                if aue < w {
                    framebuffer::cz(aue, y + row, AW_);
                }
                if asa > 0 && asa - 1 < w {
                    framebuffer::cz(asa - 1, y + row, AW_);
                }
            }
        }
        
        
        let fbm = self.cursor_x >= 4 && self.cursor_x < 120 && self.cursor_y >= y as i32;
        if fbm || self.start_menu_open {
            draw_rounded_rect(6, (y + 7) as i32, 110, 34, 10, 0xFF003318);
            framebuffer::co(6, y + 7, 110, 34, 0x00CC66, 60);
            
            framebuffer::co(4, y + 5, 114, 1, 0x00FF66, 25);
        }
        let ri = if fbm || self.start_menu_open { EP_ } else { AP_ };
        iu(6, (y + 7) as i32, 110, 34, 10, ri);
        let jox = if fbm || self.start_menu_open { I_ } else { AH_ };
        self.draw_text_smooth(20, (y + 15) as i32, "TrustOS", jox);
        
        if fbm || self.start_menu_open {
            self.draw_text_smooth(21, (y + 15) as i32, "TrustOS", jox);
        }
        
        
        let ecc = self.windows.len();
        let gu = 96u32;
        let hn = 34u32;
        let rj = 6u32;
        let aaj = if ecc > 0 { ecc as u32 * (gu + rj) - rj } else { 0 };
        let start_x = (self.width.saturating_sub(aaj)) / 2;
        
        for (i, w) in self.windows.iter().enumerate() {
            let zs = start_x + i as u32 * (gu + rj);
            let ed = y + 7;
            
            let ern = self.cursor_x >= zs as i32 && self.cursor_x < (zs + gu) as i32
                && self.cursor_y >= y as i32;
            
            
            if w.focused {
                draw_rounded_rect(zs as i32, ed as i32, gu, hn, 8, 0xFF001A0A);
                framebuffer::co(zs, ed, gu, hn, 0x00AA44, 70);
                
                framebuffer::co(zs + 4, ed, gu - 8, 1, 0x00FF66, 35);
            } else if ern {
                draw_rounded_rect(zs as i32, ed as i32, gu, hn, 8, 0xFF000D05);
                framebuffer::co(zs, ed, gu, hn, 0x008833, 50);
            }
            
            let kax = if w.focused { EP_ } else if ern { GR_ } else { AP_ };
            iu(zs as i32, ed as i32, gu, hn, 8, kax);
            
            
            let drr = match w.window_type {
                WindowType::Terminal => ">_",
                WindowType::FileManager => "[]",
                WindowType::Calculator => "##",
                WindowType::Browser => "WW",
                WindowType::TextEditor => "Tx",
                WindowType::Game => "Sk",
                WindowType::MusicPlayer => "Mu",
                _ => "::",
            };
            let icon_color = if w.focused { I_ } else { Q_ };
            self.draw_text_smooth((zs + 8) as i32, (ed + 10) as i32, drr, icon_color);
            
            
            let pkd = 7;
            let title: String = w.title.chars().take(pkd).collect();
            let text_color = if w.focused { I_ } else { BM_ };
            self.draw_text_smooth((zs + 28) as i32, (ed + 10) as i32, &title, text_color);
            
            
            if w.focused {
                let gci = 60u32.min(gu - 14);
                let igk = zs + (gu - gci) / 2;
                draw_rounded_rect((igk) as i32, (y + V_() - 5) as i32, gci, 3, 1, I_);
                framebuffer::co(igk.saturating_sub(2), y + V_() - 7, gci + 4, 2, I_, 50);
            } else if !w.minimized {
                let dnm = zs + gu / 2 - 2;
                framebuffer::fill_rect(dnm, y + V_() - 4, 4, 2, BJ_);
            }
        }
        
        
        
        let dfs = 12u32; 
        
        
        let mut bjs = self.width - 8 - 8 - dfs; 
        
        
        let mbo = 20u32;
        let cka = bjs - mbo;
        let eny = y + 16;
        bjs = cka - dfs;
        
        
        let kle = 64u32;
        let eid = bjs - kle;
        let time = self.get_time_string();
        self.draw_text_smooth(eid as i32, (y + 10) as i32, &time, ads(I_, 0xFFFFFFFF));
        
        self.draw_text_smooth((eid + 1) as i32, (y + 10) as i32, &time, ads(I_, 0xFFFFFFFF));
        let dmi = self.get_date_string();
        self.draw_text_smooth(eid as i32, (y + 27) as i32, &dmi, ads(BM_, 0xFFCCCCCC));
        bjs = eid - dfs;
        
        
        let jzt = 36u32;
        let bhe = bjs - jzt;
        let czr = y + 8;
        let hog = ((self.frame_count % 7) + 2).min(6) as u32;
        self.draw_text(bhe as i32, (czr + 2) as i32, "C", Q_);
        let egh = bhe + 12;
        for gq in 0..8u32 {
            let gtm = if gq < hog {
                if hog > 6 { DJ_ } else { I_ }
            } else { Q_ };
            framebuffer::fill_rect(egh + gq * 3, czr + 3, 2, 8, gtm);
        }
        let ing = {
            let av = 16u32;
            let used = ((self.windows.len() as u32 * 2) + 4).min(av);
            (used * 8 / av).min(8)
        };
        self.draw_text(bhe as i32, (czr + 17) as i32, "M", Q_);
        for gq in 0..8u32 {
            let gtm = if gq < ing {
                if ing > 6 { GN_ } else { I_ }
            } else { Q_ };
            framebuffer::fill_rect(egh + gq * 3, czr + 18, 2, 8, gtm);
        }
        bjs = bhe - dfs;
        
        
        let cyc = format!("{}fps", self.fps_current);
        let fxo = if self.fps_current >= 55 { AH_ } else if self.fps_current >= 30 { GN_ } else { DJ_ };
        let lyf = (cyc.len() as u32) * 8 + 4;
        let dqd = bjs - lyf;
        self.draw_text_smooth(dqd as i32, (y + 17) as i32, &cyc, fxo);
        bjs = dqd - dfs;
        
        
        let fft = crate::accessibility::oww();
        if !fft.is_empty() {
            let jsx = (fft.len() as u32) * 8 + 4;
            let hdn = bjs - jsx;
            self.draw_text_smooth(hdn as i32, (y + 17) as i32, &fft, ads(GN_, 0xFFFFFF00));
            bjs = hdn - dfs;
        }
        
        
        let pnm = 100u32;
        let fdp = bjs - pnm;
        self.draw_sys_tray_indicators(fdp, y + 10);
        let iay = self.cursor_x >= (cka as i32 - 4) && self.cursor_x < (cka as i32 + 20)
            && self.cursor_y >= y as i32;
        let fyg = if iay { I_ } else { BM_ };
        if iay {
            framebuffer::co(cka - 2, eny - 2, 20, 20, 0x00CC66, 30);
        }
        for ad in 0..16u32 {
            for dx in 0..16u32 {
                let lh = dx as i32 - 8;
                let kf = ad as i32 - 8;
                let wz = lh * lh + kf * kf;
                if wz >= 25 && wz <= 56 {
                    framebuffer::cz(cka + dx, eny + ad, fyg);
                }
                if wz <= 6 {
                    framebuffer::cz(cka + dx, eny + ad, fyg);
                }
            }
        }
        let ebi: &[(i32, i32)] = &[(0, -8), (0, 8), (-8, 0), (8, 0), (-6, -6), (6, -6), (-6, 6), (6, 6)];
        for &(bu, ty) in ebi {
            let p = (cka as i32 + 8 + bu) as u32;
            let o = (eny as i32 + 8 + ty) as u32;
            framebuffer::fill_rect(p.saturating_sub(1), o.saturating_sub(1), 3, 3, fyg);
        }
        
        
        let gtc = self.width - 8;
        let omf = 8u32;
        let ome = self.cursor_x >= gtc as i32 && self.cursor_y >= y as i32;
        let omd = if ome { Y_ } else { Q_ };
        framebuffer::fill_rect(gtc, y, omf, V_(), omd);
        framebuffer::fill_rect(gtc, y + 6, 1, V_() - 12, BJ_);
    }
    
    fn get_time_string(&mut self) -> String {
        
        
        if self.frame_count - self.last_rtc_frame >= 60 || self.cached_time_str.is_empty() {
            let fm = crate::rtc::aou();
            self.cached_time_str = format!("{:02}:{:02}", fm.hour, fm.minute);
            self.cached_date_str = format!("{:02}/{:02}", fm.month, fm.day);
            self.last_rtc_frame = self.frame_count;
        }
        self.cached_time_str.clone()
    }
    
    fn get_date_string(&self) -> String {
        self.cached_date_str.clone()
    }
    
    fn draw_start_menu(&self) {
        let pz = 480u32;
        let rv = 680u32;
        let hu = 4i32;
        let ks = (self.height - V_() - rv - 8) as i32;
        
        let dso = crate::accessibility::btq();
        
        
        
        
        
        
        if dso {
            framebuffer::fill_rect(hu as u32, ks as u32, pz, rv, 0xFF000000);
        } else {
            draw_rounded_rect(hu, ks, pz, rv, 14, 0xFF060A08);
            framebuffer::co(hu as u32, ks as u32, pz, rv, 0x060A08, 185);
        }
        
        
        let kdn = ads(GR_, 0xFFFFFFFF);
        iu(hu, ks, pz, rv, 14, kdn);
        
        framebuffer::co((hu + 14) as u32, ks as u32, pz - 28, 1, 0x00FF66, 20);
        
        
        if dso {
            framebuffer::fill_rect((hu + 2) as u32, (ks + 2) as u32, pz - 4, 28, 0xFF1A1A1A);
        } else {
            framebuffer::co((hu + 2) as u32, (ks + 2) as u32, pz - 4, 28, 0x002200, 160);
        }
        self.draw_text_smooth(hu + 14, ks + 8, "TrustOS Menu", ads(I_, 0xFFFFFF00));
        self.draw_text_smooth(hu + 15, ks + 8, "TrustOS Menu", ads(I_, 0xFFFFFF00)); 
        
        
        framebuffer::mn((hu + 2) as u32, (ks + 30) as u32, pz - 4, Q_);
        
        
        let agz = ks + 34;
        let cqd = 36u32;
        let deb = 12i32;
        let acm = pz - deb as u32 * 2;
        draw_rounded_rect(hu + deb, agz, acm, cqd, 10, 0xFF0A120A);
        iu(hu + deb, agz, acm, cqd, 10, Q_);
        
        framebuffer::co((hu + deb + 4) as u32, agz as u32, acm - 8, 1, 0x00FF66, 15);
        
        
        let cmn = hu + deb + 12;
        let cmo = agz + 10;
        for ad in 0..10u32 {
            for dx in 0..10u32 {
                let lh = dx as i32 - 5;
                let kf = ad as i32 - 5;
                let em = lh * lh + kf * kf;
                if em >= 12 && em <= 25 {
                    framebuffer::cz((cmn + dx as i32) as u32, (cmo + ad as i32) as u32, BM_);
                }
            }
        }
        framebuffer::fill_rect((cmn + 8) as u32, (cmo + 8) as u32, 4, 2, BM_);
        
        
        let gtf = hu + deb + 26;
        if self.start_menu_search.is_empty() {
            self.draw_text_smooth(gtf, agz + 12, "Search apps...", Q_);
        } else {
            self.draw_text_smooth(gtf, agz + 12, &self.start_menu_search, I_);
            let cursor_x = gtf + (self.start_menu_search.len() as i32 * 8);
            if self.cursor_blink {
                framebuffer::fill_rect(cursor_x as u32, (agz + 10) as u32, 2, 16, I_);
            }
        }
        
        let dsy = agz + cqd as i32 + 8;
        
        
        let items: [(&str, &str, bool); 19] = [
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
            ("Wv", "TrustWave", false),
            ("@)", "Settings", false),
            ("<-", "Exit Desktop", true),
            ("!!", "Shutdown", true),
            (">>", "Reboot", true),
        ];
        
        
        let search = self.start_menu_search.trim();
        let apb: alloc::string::String = search.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
        
        
        let ble = 2u32;
        let cel = (pz - 24) / ble;
        let bpk = 44u32;
        let cek = 4u32;
        let mut cio = 0usize;
        
        for (ard, (icon, label, is_special)) in items.iter().enumerate() {
            if *is_special { continue; }
            
            if !apb.is_empty() {
                let dtf: alloc::string::String = label.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                if !dtf.contains(apb.as_str()) {
                    continue;
                }
            }
            
            let col = (cio % ble as usize) as u32;
            let row = (cio / ble as usize) as u32;
            let bnd = hu + 10 + col as i32 * (cel + cek) as i32;
            let ru = dsy + (row as i32 * (bpk + cek) as i32);
            cio += 1;
            
            
            if ru + bpk as i32 > ks + rv as i32 - 110 { break; }
            
            let vl = self.cursor_x >= bnd 
                && self.cursor_x < bnd + cel as i32
                && self.cursor_y >= ru 
                && self.cursor_y < ru + bpk as i32;
            let hd = self.start_menu_selected == ard as i32;
            
            
            if vl || hd {
                draw_rounded_rect(bnd, ru, cel, bpk, 8, 0xFF0A2A14);
                framebuffer::co(bnd as u32, ru as u32, cel, bpk, 0x00AA44, if hd { 70 } else { 50 });
                iu(bnd, ru, cel, bpk, 8, Q_);
            }
            
            
            let cao = bnd + 22;
            let caq = ru + bpk as i32 / 2;
            let bcg = 14i32;
            let ifl = bcg * bcg;
            let mnf = if vl || hd { 0xFF0A3A1A } else { 0xFF0C1810 };
            for ad in -bcg..=bcg {
                for dx in -bcg..=bcg {
                    if dx * dx + ad * ad <= ifl {
                        framebuffer::cz((cao + dx) as u32, (caq + ad) as u32, mnf);
                    }
                }
            }
            
            for ad in -bcg..=bcg {
                for dx in -bcg..=bcg {
                    let jq = dx * dx + ad * ad;
                    if jq >= (bcg - 1) * (bcg - 1) && jq <= ifl {
                        let bc = if vl || hd { Y_ } else { Q_ };
                        framebuffer::cz((cao + dx) as u32, (caq + ad) as u32, bc);
                    }
                }
            }
            
            
            let icon_color = if vl || hd { I_ } else { AH_ };
            self.draw_text_smooth(cao - 8, caq - 6, icon, icon_color);
            
            
            let ace = if vl || hd { I_ } else { AB_ };
            self.draw_text_smooth(bnd + 42, caq - 6, label, ace);
        }
        
        
        let nwn = if apb.is_empty() { true } else {
            items[14..].iter().any(|(_, label, _)| {
                let das: String = label.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                das.contains(apb.as_str())
            })
        };
        if cio == 0 && !nwn && !apb.is_empty() {
            let nkn = dsy + 12;
            self.draw_text_smooth(hu + 40, nkn, "No results found", Q_);
        }
        
        
        let ewy = ks + rv as i32 - 106;
        framebuffer::mn((hu + 12) as u32, ewy as u32, pz - 24, Q_);
        
        let nwm: [(&str, &str, u8); 3] = [
            ("<-", "Exit Desktop", 14),
            ("!!", "Shutdown", 15),
            (">>", "Reboot", 16),
        ];
        
        for (pi, (icon, label, idx)) in nwm.iter().enumerate() {
            if !apb.is_empty() {
                let dtf: String = label.chars().map(|c| if c.is_ascii_uppercase() { (c as u8 + 32) as char } else { c }).collect();
                if !dtf.contains(apb.as_str()) {
                    continue;
                }
            }
            
            let ru = ewy + 8 + (pi as i32 * 30);
            let sy = 28u32;
            
            let vl = self.cursor_x >= hu 
                && self.cursor_x < hu + pz as i32
                && self.cursor_y >= ru 
                && self.cursor_y < ru + sy as i32;
            let hd = self.start_menu_selected == *idx as i32;
            
            if vl || hd {
                draw_rounded_rect(hu + 8, ru, pz - 16, sy, 6, 0xFF1A0808);
                framebuffer::co((hu + 8) as u32, ru as u32, pz - 16, sy, 0xAA2222, if hd { 50 } else { 35 });
            }
            
            let icon_color = if vl || hd { DJ_ } else { 0xFF994444 };
            self.draw_text_smooth(hu + 18, ru + 8, icon, icon_color);
            
            let ace = if vl || hd { DJ_ } else { 0xFFAA4444 };
            self.draw_text_smooth(hu + 44, ru + 8, label, ace);
        }
        
        
        let jpz = ks + rv as i32 - 22;
        framebuffer::mn((hu + 8) as u32, jpz as u32, pz - 16, Q_);
        self.draw_text(hu + 14, jpz + 6, "TrustOS v0.4.2", BJ_);
    }
    
    fn draw_window(&self, window: &Window) {
        let x = window.x;
        let y = window.y;
        let w = window.width;
        let h = window.height;
        
        
        
        
        
        let corner_radius = if window.maximized { 0u32 } else { DFG_() };
        
        
        
        if !window.maximized && w > 4 && h > 4 {
            if self.desktop_tier >= DesktopTier::Full {
                framebuffer::co((x + 10) as u32, (y + 10) as u32, w + 2, h + 2, 0x000000, 14);
                framebuffer::co((x + 7) as u32, (y + 7) as u32, w + 2, h + 2, 0x000000, 18);
                framebuffer::co((x + 5) as u32, (y + 5) as u32, w, h, 0x000000, 22);
                framebuffer::co((x + 3) as u32, (y + 3) as u32, w, h, 0x000000, 16);
                framebuffer::co((x + 1) as u32, (y + 1) as u32, w + 2, h + 2, 0x000000, 8);
            } else if self.desktop_tier >= DesktopTier::Standard {
                framebuffer::co((x + 5) as u32, (y + 5) as u32, w, h, 0x000000, 30);
                framebuffer::co((x + 2) as u32, (y + 2) as u32, w + 1, h + 1, 0x000000, 15);
            } else {
                
                framebuffer::fill_rect((x + 4) as u32, (y + 4) as u32, w, h, 0xFF080808);
            }
            if window.focused {
                
                framebuffer::co((x - 1) as u32, (y - 1) as u32, w + 2, h + 2, 0x00FF66, 10);
            }
        }
        
        
        
        if self.desktop_tier >= DesktopTier::Full {
            if corner_radius > 0 {
                lko(x, y, w, h, corner_radius, 0x080C08, 160);
            } else {
                framebuffer::co(x as u32, y as u32, w, h, 0x080C08, 160);
            }
        } else {
            if corner_radius > 0 {
                draw_rounded_rect(x, y, w, h, corner_radius, 0xFF0A0E0A);
            } else {
                framebuffer::fill_rect(x as u32, y as u32, w, h, 0xFF0A0E0A);
            }
        }
        
        
        let ri = if window.focused {
            ads(GR_, 0xFFFFFFFF)
        } else {
            ads(AP_, 0xFF888888)
        };
        let cui = if window.focused { Q_ } else { AP_ };
        if self.desktop_tier >= DesktopTier::Full {
            
            if corner_radius > 0 {
                iu(x, y, w, h, corner_radius, ri);
                iu(x + 1, y + 1, w.saturating_sub(2), h.saturating_sub(2), corner_radius.saturating_sub(1), cui);
                iu(x + 2, y + 2, w.saturating_sub(4), h.saturating_sub(4), corner_radius.saturating_sub(2), ri);
                iu(x + 3, y + 3, w.saturating_sub(6), h.saturating_sub(6), corner_radius.saturating_sub(3), cui);
            } else {
                framebuffer::draw_rect(x as u32, y as u32, w, h, ri);
                framebuffer::draw_rect((x + 1) as u32, (y + 1) as u32, w.saturating_sub(2), h.saturating_sub(2), cui);
                framebuffer::draw_rect((x + 2) as u32, (y + 2) as u32, w.saturating_sub(4), h.saturating_sub(4), ri);
                framebuffer::draw_rect((x + 3) as u32, (y + 3) as u32, w.saturating_sub(6), h.saturating_sub(6), cui);
            }
        } else {
            
            if corner_radius > 0 {
                iu(x, y, w, h, corner_radius, ri);
                iu(x + 1, y + 1, w.saturating_sub(2), h.saturating_sub(2), corner_radius.saturating_sub(1), cui);
            } else {
                framebuffer::draw_rect(x as u32, y as u32, w, h, ri);
                framebuffer::draw_rect((x + 1) as u32, (y + 1) as u32, w.saturating_sub(2), h.saturating_sub(2), cui);
            }
        }
        
        
        if window.focused && !window.maximized && w > 20 && h > 20 {
            let th = window.on_resize_edge(self.cursor_x, self.cursor_y);
            let aog = 0x00FF66u32;
            let caf = 40u32;
            let gt = 4u32;
            let agl = if h > 4 { h - 4 } else { 1 };
            let fz = if w > 4 { w - 4 } else { 1 };
            match th {
                ResizeEdge::Left | ResizeEdge::TopLeft | ResizeEdge::BottomLeft => {
                    framebuffer::co(x as u32, (y + 2) as u32, gt, agl, aog, caf);
                }
                _ => {}
            }
            match th {
                ResizeEdge::Right | ResizeEdge::TopRight | ResizeEdge::BottomRight => {
                    framebuffer::co((x + w as i32 - gt as i32) as u32, (y + 2) as u32, gt, agl, aog, caf);
                }
                _ => {}
            }
            match th {
                ResizeEdge::Top | ResizeEdge::TopLeft | ResizeEdge::TopRight => {
                    framebuffer::co((x + 2) as u32, y as u32, fz, gt, aog, caf);
                }
                _ => {}
            }
            match th {
                ResizeEdge::Bottom | ResizeEdge::BottomLeft | ResizeEdge::BottomRight => {
                    framebuffer::co((x + 2) as u32, (y + h as i32 - gt as i32) as u32, fz, gt, aog, caf);
                }
                _ => {}
            }
        }

        
        
        
        let bpn = J_();
        let dfk = (x + 3) as u32;
        let dfj = w.saturating_sub(6);
        if self.desktop_tier >= DesktopTier::Full {
            
            if window.focused {
                framebuffer::co(dfk, (y + 3) as u32, dfj, bpn - 3, 0x0E2210, 190);
                framebuffer::co(dfk, (y + 3) as u32, dfj, 1, 0x00FF66, 30);
                framebuffer::co(dfk, (y + 4) as u32, dfj, 1, 0x00CC55, 15);
            } else {
                framebuffer::co(dfk, (y + 3) as u32, dfj, bpn - 3, 0x080C08, 175);
            }
        } else {
            
            if window.focused {
                framebuffer::fill_rect(dfk, (y + 3) as u32, dfj, bpn - 3, 0xFF0E2210);
            } else {
                framebuffer::fill_rect(dfk, (y + 3) as u32, dfj, bpn - 3, 0xFF080C08);
            }
        }
        
        
        framebuffer::mn((x + 3) as u32, (y + bpn as i32) as u32, w.saturating_sub(6), 
            if window.focused { Q_ } else { AP_ });

        
        
        
        let gu = 28u32;
        let hn = bpn - 4;
        let ed = (y + 3) as u32;
        let cg = self.cursor_x;
        let cr = self.cursor_y;
        
        
        let adl = x + w as i32 - gu as i32 - 3;
        let hlo = cg >= adl && cg < adl + gu as i32 
            && cr >= ed as i32 && cr < ed as i32 + hn as i32;
        let close_bg = if hlo { 0xFFCC3333 } else if window.focused { 0xFF2A1414 } else { 0xFF1A1A1A };
        framebuffer::fill_rect(adl as u32, ed, gu, hn, close_bg);
        
        let ejq = adl + gu as i32 / 2;
        let ejr = ed as i32 + hn as i32 / 2;
        let ffo = if hlo { 0xFFFFFFFF } else if window.focused { 0xFFCC4444 } else { 0xFF666666 };
        for i in -3..=3i32 {
            framebuffer::cz((ejq + i) as u32, (ejr + i) as u32, ffo);
            framebuffer::cz((ejq + i) as u32, (ejr - i) as u32, ffo);
            
            framebuffer::cz((ejq + i + 1) as u32, (ejr + i) as u32, ffo);
            framebuffer::cz((ejq + i + 1) as u32, (ejr - i) as u32, ffo);
        }
        
        
        let aly = adl - gu as i32;
        let iml = cg >= aly && cg < aly + gu as i32 
            && cr >= ed as i32 && cr < ed as i32 + hn as i32;
        let ggs = if iml { 0xFF1A3A20 } else { 0xFF0E0E0E };
        framebuffer::fill_rect(aly as u32, ed, gu, hn, ggs);
        let bln = aly + gu as i32 / 2;
        let blo = ed as i32 + hn as i32 / 2;
        let bnl = if iml { 0xFF44DD66 } else if window.focused { 0xFF227744 } else { 0xFF555555 };
        if window.maximized {
            
            for i in -2..=1i32 {
                framebuffer::cz((bln + i + 1) as u32, (blo - 3) as u32, bnl);
                framebuffer::cz((bln + 3) as u32, (blo + i - 1) as u32, bnl);
            }
            for i in -2..=2i32 {
                framebuffer::cz((bln + i - 1) as u32, (blo - 1) as u32, bnl);
                framebuffer::cz((bln + i - 1) as u32, (blo + 3) as u32, bnl);
                framebuffer::cz((bln - 3) as u32, (blo + i + 1) as u32, bnl);
                framebuffer::cz((bln + 1) as u32, (blo + i + 1) as u32, bnl);
            }
        } else {
            
            for i in -3..=3i32 {
                framebuffer::cz((bln + i) as u32, (blo - 3) as u32, bnl);
                framebuffer::cz((bln + i) as u32, (blo + 3) as u32, bnl);
                framebuffer::cz((bln - 3) as u32, (blo + i) as u32, bnl);
                framebuffer::cz((bln + 3) as u32, (blo + i) as u32, bnl);
            }
        }
        
        
        let ayg = aly - gu as i32;
        let ins = cg >= ayg && cg < ayg + gu as i32 
            && cr >= ed as i32 && cr < ed as i32 + hn as i32;
        let ghr = if ins { 0xFF2A2A10 } else { 0xFF0E0E0E };
        framebuffer::fill_rect(ayg as u32, ed, gu, hn, ghr);
        let hqa = ayg + gu as i32 / 2;
        let hqc = ed as i32 + hn as i32 / 2;
        let ipg = if ins { 0xFFFFBB33 } else if window.focused { 0xFF886622 } else { 0xFF555555 };
        
        for i in -3..=3i32 {
            framebuffer::cz((hqa + i) as u32, hqc as u32, ipg);
            framebuffer::cz((hqa + i) as u32, (hqc + 1) as u32, ipg);
        }
        
        
        framebuffer::fill_rect(ayg as u32, ed, 1, hn, AP_);
        framebuffer::fill_rect(aly as u32, ed, 1, hn, AP_);
        framebuffer::fill_rect(adl as u32, ed, 1, hn, AP_);
        
        
        let adt = x + 10;
        let drr = match window.window_type {
            WindowType::Terminal => ">_",
            WindowType::FileManager => "[]",
            WindowType::Calculator => "##",
            WindowType::Browser => "WW",
            WindowType::ModelEditor => "/\\",
            WindowType::TextEditor => "Tx",
            WindowType::Game => "Sk",
            WindowType::Chess => "Kk",
            WindowType::Chess3D => "C3",
            WindowType::MusicPlayer => "Mu",
            _ => "::",
        };
        let icon_color = if window.focused { I_ } else { BM_ };
        self.draw_text_smooth(adt, y + (bpn as i32 / 2) - 6, drr, icon_color);
        
        
        let text_color = if window.focused {
            ads(AB_, 0xFFFFFFFF)
        } else {
            ads(O_, 0xFFCCCCCC)
        };
        let pke = window.title.len() as i32 * 8;
        let pkb = x + (w as i32 / 2) - (pke / 2);
        let avk = pkb.max(adt + 24);
        self.draw_text_smooth(avk, y + (bpn as i32 / 2) - 6, &window.title, text_color);
        
        
        
        
        let bn = y + bpn as i32;
        let en = h - bpn;
        
        
        if self.desktop_tier >= DesktopTier::Full {
            framebuffer::fill_rect((x + 3) as u32, (bn + 1) as u32, w.saturating_sub(6), en.saturating_sub(4), 0xFF080808);
            framebuffer::co((x + 3) as u32, (bn + 1) as u32, w.saturating_sub(6), en.saturating_sub(4), 0x060A06, 210);
        } else {
            framebuffer::fill_rect((x + 3) as u32, (bn + 1) as u32, w.saturating_sub(6), en.saturating_sub(4), 0xFF080A08);
        }
        
        
        let aex = (x + 3).max(0) as u32;
        let lg = (bn + 1).max(0) as u32;
        let zt = w.saturating_sub(6);
        let ur = en.saturating_sub(4);
        framebuffer::jey(aex, lg, zt, ur);
        
        
        self.draw_window_content(window);
        
        
        framebuffer::hlf();
    }
    
    
    fn qdx(&self, x: u32, y: u32, size: u32, color: u32, hovered: bool) {
        if hovered {
            
            framebuffer::fill_rect(x.saturating_sub(1), y.saturating_sub(1), size + 2, size + 2, 
                (color & 0x00FFFFFF) | 0x40000000);
        }
        framebuffer::fill_rect(x, y, size, size, color);
    }
    
    
    fn hia(&self, hw: u32, jf: u32, t: f32) -> u32 {
        let uh = ((hw >> 16) & 0xFF) as f32;
        let bbu = ((hw >> 8) & 0xFF) as f32;
        let gf = (hw & 0xFF) as f32;
        let ju = ((jf >> 16) & 0xFF) as f32;
        let axe = ((jf >> 8) & 0xFF) as f32;
        let iq = (jf & 0xFF) as f32;
        
        let r = (uh + (ju - uh) * t) as u32;
        let g = (bbu + (axe - bbu) * t) as u32;
        let b = (gf + (iq - gf) * t) as u32;
        
        0xFF000000 | (r << 16) | (g << 8) | b
    }
    
    fn draw_window_content(&self, window: &Window) {
        let ho = window.x + 8;
        let bn = window.y + J_() as i32 + 8;
        
        
        if window.window_type == WindowType::TextEditor {
            return;
        }
        
        
        if window.window_type == WindowType::ModelEditor {
            return;
        }
        
        
        if window.window_type == WindowType::Game3D {
            return;
        }

        
        #[cfg(feature = "emulators")]
        if window.window_type == WindowType::GameBoyEmu
            || window.window_type == WindowType::GameBoyInput
            || window.window_type == WindowType::NesEmu
        {
            return;
        }
        
        
        if window.window_type == WindowType::Calculator {
            self.draw_calculator(window);
            return;
        }
        
        
        
        
        if window.window_type == WindowType::MusicPlayer {
            self.draw_music_player(window);
            return;
        }
        
        
        
        
        if window.window_type == WindowType::WifiNetworks {
            self.draw_wifi_networks(window);
            return;
        }
        
        
        
        
        if window.window_type == WindowType::WifiPassword {
            self.draw_wifi_password(window);
            return;
        }
        
        
        
        
        if window.window_type == WindowType::FileManager {
            self.draw_file_manager_gui(window);
            return;
        }
        
        
        
        
        if window.window_type == WindowType::ImageViewer {
            self.draw_image_viewer(window);
            return;
        }
        
        
        if window.window_type == WindowType::Demo3D {
            self.draw_3d_demo(window);
            return;
        }
        
        
        if window.window_type == WindowType::Game {
            self.draw_snake_game(window);
            return;
        }
        
        
        if window.window_type == WindowType::Chess {
            self.draw_chess_game(window);
            return;
        }
        
        
        if window.window_type == WindowType::Chess3D {
            return;
        }
        
        
        if window.window_type == WindowType::BinaryViewer {
            self.draw_binary_viewer(window);
            return;
        }
        
        
        if window.window_type == WindowType::LabMode {
            if let Some(state) = self.lab_states.get(&window.id) {
                crate::lab_mode::lji(state, window.x, window.y, window.width, window.height);
            }
            return;
        }

        
        if window.window_type == WindowType::WifiAnalyzer {
            if let Some(state) = self.wifi_analyzer_states.get(&window.id) {
                crate::wifi_analyzer::draw(state, window.x, window.y, window.width, window.height);
            }
            return;
        }
        
        
        #[cfg(feature = "emulators")]
        if window.window_type == WindowType::GameLab {
            if let Some(lab_state) = self.gamelab_states.get(&window.id) {
                
                let lpd = if let Some(cmh) = lab_state.linked_gb_id {
                    self.gameboy_states.get(&cmh)
                } else {
                    
                    self.gameboy_states.values().next()
                };
                crate::game_lab::lix(lab_state, lpd, window.x, window.y, window.width, window.height);
            }
            return;
        }
        
        
        if window.window_type == WindowType::Browser {
            self.draw_browser(window);
            return;
        }
        
        
        
        
        if window.window_type == WindowType::Settings {
            self.draw_settings_gui(window);
            return;
        }
        
        
        
        
        if window.window_type == WindowType::Cn {
            self.draw_netscan_gui(window);
            return;
        }
        
        
        
        
        if window.window_type == WindowType::Terminal {
            let line_height = 16i32;
            let chp = (window.height as i32 - J_() as i32 - 16).max(0) as usize;
            let oe = if line_height as usize > 0 { chp / line_height as usize } else { 0 };
            let total_lines = window.content.len();
            
            
            let scroll = window.scroll_offset;
            let start = scroll;
            let end = (start + oe).min(total_lines);
            
            for idx in start..end {
                let line = &window.content[idx];
                let aka = bn + ((idx - start) as i32 * line_height);
                if aka >= window.y + window.height as i32 - 8 {
                    break;
                }
                
                
                
                
                if line.contains('\x01') {
                    let mut cx = ho;
                    let mut current_color = B_;
                    let mut chars = line.chars().peekable();
                    while let Some(ch) = chars.next() {
                        if ch == '\x01' {
                            if let Some(&code) = chars.peek() {
                                chars.next();
                                current_color = match code {
                                    'R' => DJ_,
                                    'G' => I_,
                                    'B' => RQ_,
                                    'W' => AB_,
                                    'Y' => GN_,
                                    'M' => Y_,
                                    'D' => Q_,
                                    'N' => B_,
                                    'H' => 0xFF00FFAA,
                                    'A' => BM_,
                                    'S' => BJ_,
                                    _ => current_color,
                                };
                            }
                        } else {
                            crate::framebuffer::px(cx as u32, aka as u32, ch, current_color);
                            cx += 8;
                        }
                    }
                } else {
                    
                    let jw = line.trim_start();
                    if jw.starts_with("root@trustos") || jw.starts_with("$") {
                        
                        let mut cx = ho;
                        if let Some(dollar_pos) = line.find('$') {
                            
                            let bak = &line[..dollar_pos];
                            
                            if let Some(at_pos) = bak.find('@') {
                                
                                for ch in bak[..at_pos].chars() {
                                    crate::framebuffer::px(cx as u32, aka as u32, ch, I_);
                                    cx += 8;
                                }
                                
                                crate::framebuffer::px(cx as u32, aka as u32, '@', Q_);
                                cx += 8;
                                
                                let epm = &bak[at_pos + 1..];
                                
                                if let Some(bfk) = epm.find(':') {
                                    for ch in epm[..bfk].chars() {
                                        crate::framebuffer::px(cx as u32, aka as u32, ch, RQ_);
                                        cx += 8;
                                    }
                                    crate::framebuffer::px(cx as u32, aka as u32, ':', Q_);
                                    cx += 8;
                                    
                                    for ch in epm[bfk + 1..].chars() {
                                        crate::framebuffer::px(cx as u32, aka as u32, ch, GN_);
                                        cx += 8;
                                    }
                                } else {
                                    for ch in epm.chars() {
                                        crate::framebuffer::px(cx as u32, aka as u32, ch, RQ_);
                                        cx += 8;
                                    }
                                }
                            } else {
                                for ch in bak.chars() {
                                    crate::framebuffer::px(cx as u32, aka as u32, ch, AH_);
                                    cx += 8;
                                }
                            }
                            
                            crate::framebuffer::px(cx as u32, aka as u32, '$', I_);
                            cx += 8;
                            
                            for ch in line[dollar_pos + 1..].chars() {
                                crate::framebuffer::px(cx as u32, aka as u32, ch, AB_);
                                cx += 8;
                            }
                        } else {
                            self.draw_text(ho, aka, line, B_);
                        }
                    } else {
                        
                        self.draw_text(ho, aka, line, B_);
                    }
                }
            }
            
            
            let ezs = 6u32;
            let cqa = (window.x + window.width as i32 - ezs as i32 - 3) as u32;
            let bwn = (window.y + J_() as i32 + 2) as u32;
            let ada = window.height.saturating_sub(J_() + 4);
            
            if total_lines > oe {
                
                framebuffer::co(cqa, bwn, ezs, ada, 0x0A1A0F, 80);
                
                
                let zo = ((oe as u32 * ada) / total_lines as u32).max(20);
                let aab = total_lines.saturating_sub(oe);
                let akn = if aab > 0 {
                    bwn + ((scroll as u32 * (ada - zo)) / aab as u32)
                } else {
                    bwn
                };
                
                draw_rounded_rect(cqa as i32, akn as i32, ezs, zo, 3, Y_);
                
                framebuffer::co(cqa + 1, akn + 1, ezs - 2, 1, 0x00FF66, 30);
            }
            
            return;
        }
        
        
        let nhz = matches!(window.window_type, 
            WindowType::FileManager | WindowType::FileAssociations);
        
        
        let (ded, ezw) = match window.window_type {
            WindowType::FileManager => (5, window.content.len().saturating_sub(2)),
            WindowType::FileAssociations => (4, window.content.len().saturating_sub(2)),
            _ => (0, 0),
        };
        
        
        let scroll = if window.window_type == WindowType::HexViewer {
            window.scroll_offset
        } else {
            0
        };
        
        for (idx, line) in window.content.iter().enumerate().skip(scroll) {
            let i = idx - scroll;
            let aka = bn + (i as i32 * 16);
            if aka >= window.y + window.height as i32 - 8 {
                break;
            }
            
            
            let hd = nhz 
                && idx >= ded 
                && idx < ezw 
                && (idx - ded) == window.selected_index;
            
            if hd {
                
                framebuffer::fill_rect(
                    ho as u32 - 4, 
                    aka as u32 - 2, 
                    window.width - 16, 
                    18, 
                    0xFF003300
                );
                self.draw_text(ho, aka, line, G_);
            } else {
                self.draw_text(ho, aka, line, B_);
            }
        }
    }
    
    
    fn draw_editor_windows(&mut self) {
        
        let fud: Vec<(u32, i32, i32, u32, u32)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::TextEditor && w.visible && !w.minimized)
            .map(|w| (w.id, w.x, w.y, w.width, w.height))
            .collect();
        
        for (fr, wx, wy, ca, er) in fud {
            if let Some(editor) = self.editor_states.get_mut(&fr) {
                let ho = wx;
                let bn = wy + J_() as i32;
                let hy = ca;
                let en = er.saturating_sub(J_());
                
                
                framebuffer::jey(
                    (ho + 2).max(0) as u32,
                    bn.max(0) as u32,
                    hy.saturating_sub(4),
                    en,
                );
                
                bvh(
                    editor,
                    ho, bn, hy, en,
                    &|x, y, text, color| {
                        
                        for (i, ch) in text.chars().enumerate() {
                            let cx = (x + (i as i32 * 8)) as u32;
                            let u = y as u32;
                            crate::framebuffer::px(cx, u, ch, color);
                        }
                    },
                    &|x, y, ch, color| {
                        crate::framebuffer::px(x as u32, y as u32, ch, color);
                    },
                );
                
                framebuffer::hlf();
            }
        }
    }
    
    
    fn draw_model_editor_windows(&mut self) {
        
        let fud: Vec<(u32, i32, i32, u32, u32)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::ModelEditor && w.visible && !w.minimized)
            .map(|w| (w.id, w.x, w.y, w.width, w.height))
            .collect();
        
        for (fr, wx, wy, ca, er) in fud {
            if let Some(state) = self.model_editor_states.get_mut(&fr) {
                let ho = wx as u32;
                let bn = (wy + J_() as i32) as u32;
                let hy = ca;
                let en = er.saturating_sub(J_());
                
                if hy < 80 || en < 80 { continue; }
                
                
                let buf_w = hy as usize;
                let buf_h = en as usize;
                let mut buf = alloc::vec![0u32; buf_w * buf_h];
                
                state.render(&mut buf, buf_w, buf_h);
                
                
                for o in 0..buf_h {
                    for p in 0..buf_w {
                        let color = buf[o * buf_w + p];
                        let am = ho + p as u32;
                        let ak = bn + o as u32;
                        framebuffer::cz(am, ak, color);
                    }
                }
            }
        }
    }
    
    
    fn draw_game3d_windows(&mut self) {
        let mbb: Vec<(u32, i32, i32, u32, u32)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::Game3D && w.visible && !w.minimized)
            .map(|w| (w.id, w.x, w.y, w.width, w.height))
            .collect();
        
        for (fr, wx, wy, ca, er) in mbb {
            if let Some(state) = self.game3d_states.get_mut(&fr) {
                let ho = wx as u32;
                let bn = (wy + J_() as i32) as u32;
                let hy = ca;
                let en = er.saturating_sub(J_());
                
                if hy < 80 || en < 60 { continue; }
                
                let buf_w = hy as usize;
                let buf_h = en as usize;
                let mut buf = alloc::vec![0u32; buf_w * buf_h];
                
                state.render(&mut buf, buf_w, buf_h);
                
                
                for o in 0..buf_h {
                    for p in 0..buf_w {
                        let color = buf[o * buf_w + p];
                        let am = ho + p as u32;
                        let ak = bn + o as u32;
                        framebuffer::cz(am, ak, color);
                    }
                }
            }
        }
    }
    
    
    fn draw_chess3d_windows(&mut self) {
        let kkb: Vec<(u32, i32, i32, u32, u32)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::Chess3D && w.visible && !w.minimized && !w.pending_close)
            .map(|w| (w.id, w.x, w.y, w.width, w.height))
            .collect();
        
        for (fr, wx, wy, ca, er) in kkb {
            if let Some(state) = self.chess3d_states.get_mut(&fr) {
                let ho = wx as u32;
                let bn = (wy + J_() as i32) as u32;
                let hy = ca;
                let en = er.saturating_sub(J_());
                
                if hy < 100 || en < 100 { continue; }
                
                state.tick();
                
                let buf_w = hy as usize;
                let buf_h = en as usize;
                let mut buf = alloc::vec![0u32; buf_w * buf_h];
                
                state.render(&mut buf, buf_w, buf_h);
                
                
                for o in 0..buf_h {
                    for p in 0..buf_w {
                        let color = buf[o * buf_w + p];
                        let am = ho + p as u32;
                        let ak = bn + o as u32;
                        framebuffer::cz(am, ak, color);
                    }
                }
            }
        }
    }
    
    
    #[cfg(feature = "emulators")]
    fn draw_nes_windows(&mut self) {
        let nig: Vec<(u32, i32, i32, u32, u32)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::NesEmu && w.visible && !w.minimized && !w.pending_close)
            .map(|w| (w.id, w.x, w.y, w.width, w.height))
            .collect();
        
        for (fr, wx, wy, ca, er) in nig {
            if let Some(an) = self.nes_states.get_mut(&fr) {
                let ho = wx as u32;
                let bn = (wy + J_() as i32) as u32;
                let hy = ca;
                let en = er.saturating_sub(J_());
                
                if hy < 80 || en < 60 { continue; }
                
                let buf_w = hy as usize;
                let buf_h = en as usize;
                let mut buf = alloc::vec![0u32; buf_w * buf_h];
                
                an.render(&mut buf, buf_w, buf_h);
                
                for o in 0..buf_h {
                    for p in 0..buf_w {
                        let color = buf[o * buf_w + p];
                        let am = ho + p as u32;
                        let ak = bn + o as u32;
                        framebuffer::cz(am, ak, color);
                    }
                }
            }
        }
    }
    
    
    #[cfg(feature = "emulators")]
    fn draw_gameboy_windows(&mut self) {
        let mbf: Vec<(u32, i32, i32, u32, u32, bool)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::GameBoyEmu && w.visible && !w.minimized && !w.pending_close)
            .map(|w| (w.id, w.x, w.y, w.width, w.height, w.focused))
            .collect();
        
        let rv: u32 = 22;
        
        for (fr, wx, wy, ca, er, _focused) in mbf {
            if let Some(an) = self.gameboy_states.get_mut(&fr) {
                let ho = wx as u32;
                let bn = (wy + J_() as i32) as u32;
                let hy = ca;
                let en = er.saturating_sub(J_());
                
                if hy < 80 || en < 60 { continue; }
                
                
                framebuffer::fill_rect(ho, bn, hy, rv, 0xFF0E1418);
                framebuffer::fill_rect(ho, bn + rv - 1, hy, 1, 0xFF1E3028);
                
                
                let nsp = alloc::format!("PC:{:04X}", an.cpu.pc);
                let nbk = alloc::format!("LY:{:3}", an.gpu.ly);
                let nft = match an.gpu.mode {
                    0 => "HBL",
                    1 => "VBL",
                    2 => "OAM",
                    3 => "DRW",
                    _ => "???",
                };
                let jzl = alloc::format!("BK:{}", an.cart.rom_bank);
                
                let mut bu = ho + 4;
                for ch in nsp.chars() { framebuffer::px(bu, bn + 4, ch, 0xFF58A6FF); bu += 8; }
                bu += 8;
                for ch in nbk.chars() { framebuffer::px(bu, bn + 4, ch, 0xFF80FFAA); bu += 8; }
                bu += 8;
                for ch in nft.chars() { framebuffer::px(bu, bn + 4, ch, 0xFFD29922); bu += 8; }
                bu += 8;
                for ch in jzl.chars() { framebuffer::px(bu, bn + 4, ch, 0xFF9CD8B0); bu += 8; }
                
                if an.cgb_mode {
                    bu += 8;
                    let dzw = if an.key1 & 0x80 != 0 { "2x" } else { "1x" };
                    for ch in "CGB".chars() { framebuffer::px(bu, bn + 4, ch, 0xFF00FF88); bu += 8; }
                    bu += 4;
                    for ch in dzw.chars() { framebuffer::px(bu, bn + 4, ch, 0xFF79C0FF); bu += 8; }
                }
                
                
                
                let cld: u32 = 48;
                let btk = ho + hy - cld - 4;
                framebuffer::fill_rect(btk, bn + 2, cld, rv - 4, 0xFF1A3028);
                framebuffer::fill_rect(btk, bn + 2, cld, 1, 0xFF2A4A38);
                framebuffer::fill_rect(btk, bn + rv - 3, cld, 1, 0xFF2A4A38);
                let mus = btk + 4;
                for (i, ch) in "INPUT".chars().enumerate() {
                    framebuffer::px(mus + i as u32 * 8, bn + 5, ch, 0xFF00FF88);
                }
                
                
                let clu: u32 = 32;
                let clv = btk - clu - 6;
                framebuffer::fill_rect(clv, bn + 2, clu, rv - 4, 0xFF1A2838);
                framebuffer::fill_rect(clv, bn + 2, clu, 1, 0xFF2A3A58);
                framebuffer::fill_rect(clv, bn + rv - 3, clu, 1, 0xFF2A3A58);
                let nbe = clv + 4;
                for (i, ch) in "LAB".chars().enumerate() {
                    framebuffer::px(nbe + i as u32 * 8, bn + 5, ch, 0xFF58A6FF);
                }
                
                
                let xp = bn + rv;
                let alo = en.saturating_sub(rv);
                
                if alo < 40 { continue; }
                
                let buf_w = hy as usize;
                let buf_h = alo as usize;
                let mut buf = alloc::vec![0u32; buf_w * buf_h];
                
                an.render(&mut buf, buf_w, buf_h);
                
                for o in 0..buf_h {
                    for p in 0..buf_w {
                        let color = buf[o * buf_w + p];
                        let am = ho + p as u32;
                        let ak = xp + o as u32;
                        framebuffer::cz(am, ak, color);
                    }
                }
            }
        }
        
        
        let mqp: Vec<(u32, i32, i32, u32, u32, Option<u32>)> = self.windows.iter()
            .filter(|w| w.window_type == WindowType::GameBoyInput && w.visible && !w.minimized && !w.pending_close)
            .map(|w| {
                let myt = self.gb_input_links.get(&w.id).copied();
                (w.id, w.x, w.y, w.width, w.height, myt)
            })
            .collect();
        
        for (_win_id, wx, wy, ca, er, cmh) in mqp {
            let cx = wx as u32;
            let u = (wy + J_() as i32) as u32;
            let aq = ca;
            let ch = er.saturating_sub(J_());
            
            if aq < 60 || ch < 40 { continue; }
            
            
            let lpc = if let Some(lid) = cmh {
                self.gameboy_states.get(&lid)
            } else {
                self.gameboy_states.values().next()
            };
            
            crate::game_lab::ljg(lpc, cx, u, aq, ch);
        }
    }
    
    
    fn draw_3d_demo(&self, window: &Window) {
        use crate::graphics::opengl::*;
        use crate::graphics::texture;
        
        let bba = window.x as u32 + 10;
        let bfv = window.y as u32 + J_() + 10;
        let ajk = window.width.saturating_sub(20);
        let aqf = window.height.saturating_sub(J_() + 20);
        
        if ajk < 80 || aqf < 80 {
            return;
        }
        
        
        ice(ajk, aqf);
        mew(bba as i32, bfv as i32, ajk, aqf);
        
        
        icd(0.04, 0.06, 0.04, 1.0);
        icc(AEM_ | AEN_);
        
        
        fzc(UR_);
        
        
        let bqh = ajk as f32 / aqf as f32;
        eoi(OE_);
        dqu();
        mfc(45.0, bqh, 0.1, 100.0);
        
        
        eoi(AEO_);
        dqu();
        icj(
            3.0, 2.0, 4.0,   
            0.0, 0.0, 0.0,   
            0.0, 1.0, 0.0    
        );
        
        
        let cc = (self.frame_count as f32 * 0.5) % 360.0;
        eoj(cc, 0.0, 1.0, 0.0);
        eoj(cc * 0.3, 1.0, 0.0, 0.0);
        
        
        let j = 0.8;
        
        aqw(KZ_);
        
        
        bmm(1.0, 0.2, 0.2);
        ahz(0.0, 0.0, 1.0);
        dt(-j, -j, j);
        dt(j, -j, j);
        dt(j, j, j);
        dt(-j, j, j);
        
        
        bmm(0.2, 1.0, 0.2);
        ahz(0.0, 0.0, -1.0);
        dt(j, -j, -j);
        dt(-j, -j, -j);
        dt(-j, j, -j);
        dt(j, j, -j);
        
        
        bmm(0.2, 0.2, 1.0);
        ahz(0.0, 1.0, 0.0);
        dt(-j, j, j);
        dt(j, j, j);
        dt(j, j, -j);
        dt(-j, j, -j);
        
        
        bmm(1.0, 1.0, 0.2);
        ahz(0.0, -1.0, 0.0);
        dt(-j, -j, -j);
        dt(j, -j, -j);
        dt(j, -j, j);
        dt(-j, -j, j);
        
        
        bmm(1.0, 0.2, 1.0);
        ahz(1.0, 0.0, 0.0);
        dt(j, -j, j);
        dt(j, -j, -j);
        dt(j, j, -j);
        dt(j, j, j);
        
        
        bmm(0.2, 1.0, 1.0);
        ahz(-1.0, 0.0, 0.0);
        dt(-j, -j, -j);
        dt(-j, -j, j);
        dt(-j, j, j);
        dt(-j, j, -j);
        
        aqx();
        
        
        icg();
        mev(2.5, 0.0, 0.0); 
        eoj(cc * 0.7, 0.3, 1.0, 0.2);
        
        
        static mut ASA_: u32 = 0;
        static mut BJD_: bool = false;
        unsafe {
            if !BJD_ {
                ldd(&mut ASA_);
                BJD_ = true;
            }
            lde(0.0, ASA_);
        }
        icf();
        
        
        dqu();
        icj(3.0, 2.0, 4.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        
        aqw(UU_);
        
        bmm(1.0, 0.0, 0.0);
        dt(0.0, 0.0, 0.0);
        dt(2.0, 0.0, 0.0);
        
        bmm(0.0, 1.0, 0.0);
        dt(0.0, 0.0, 0.0);
        dt(0.0, 2.0, 0.0);
        
        bmm(0.0, 0.0, 1.0);
        dt(0.0, 0.0, 0.0);
        dt(0.0, 0.0, 2.0);
        aqx();
        
        
        self.draw_text(bba as i32 + 8, bfv as i32 + 8, "TrustGL OpenGL Demo", AH_);
        self.draw_text(bba as i32 + 8, bfv as i32 + 24, "Software 3D + Textures", BM_);
        
        
        let bpe = bfv as i32 + aqf as i32 - 24;
        self.draw_text(bba as i32 + 8, bpe, "Left: Color Cube | Right: Textured Cube", Y_);
        self.draw_text(bba as i32 + 8, bpe, "Vertices: 8 | Edges: 12 | Faces: 6", Y_);
    }
    
    
    fn qdt(&self, bm: i32, az: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - bm).abs();
        let ad = -(y1 - az).abs();
        let am = if bm < x1 { 1 } else { -1 };
        let ak = if az < y1 { 1 } else { -1 };
        let mut err = dx + ad;
        let mut x = bm;
        let mut y = az;
        
        loop {
            if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
                framebuffer::cz(x as u32, y as u32, color);
            }
            if x == x1 && y == y1 { break; }
            let pg = 2 * err;
            if pg >= ad {
                err += ad;
                x += am;
            }
            if pg <= dx {
                err += dx;
                y += ak;
            }
        }
    }
    
    
    fn draw_snake_game(&self, window: &Window) {
        let yu = window.x as u32 + 10;
        let xp = window.y as u32 + J_() + 10;
        let ajs = window.width.saturating_sub(20);
        let alo = window.height.saturating_sub(J_() + 20);
        
        if ajs < 80 || alo < 80 {
            return;
        }
        
        
        framebuffer::fill_rect(yu, xp, ajs, alo, 0xFF0A0E0B);
        
        
        for i in 0..ajs {
            framebuffer::cz(yu + i, xp, Y_);
            framebuffer::cz(yu + i, xp + alo - 1, Y_);
        }
        for i in 0..alo {
            framebuffer::cz(yu, xp + i, Y_);
            framebuffer::cz(yu + ajs - 1, xp + i, Y_);
        }
        
        
        if let Some(snake) = self.snake_states.get(&window.id) {
            let cell_size: u32 = 14;
            let fzn = yu + 10;
            let fzo = xp + 36;
            
            
            for jh in 0..snake.grid_h {
                for hc in 0..snake.grid_w {
                    let p = fzn + hc as u32 * cell_size;
                    let o = fzo + jh as u32 * cell_size;
                    if p + cell_size < yu + ajs && o + cell_size < xp + alo {
                        let bg = if (hc + jh) % 2 == 0 { 0xFF0D120E } else { 0xFF0B100C };
                        framebuffer::fill_rect(p, o, cell_size, cell_size, bg);
                    }
                }
            }
            
            
            for (i, &(am, ak)) in snake.snake.iter().enumerate() {
                let p = fzn + am as u32 * cell_size;
                let o = fzo + ak as u32 * cell_size;
                let color = if i == 0 { 
                    0xFF00FF00 
                } else {
                    let ln = (0xCC - (i as u32 * 8).min(0x80)) as u32;
                    0xFF000000 | (ln << 8) 
                };
                
                if p + cell_size < yu + ajs && o + cell_size < xp + alo {
                    framebuffer::fill_rect(p + 1, o + 1, cell_size - 2, cell_size - 2, color);
                    
                    if i == 0 {
                        let (ex1, ey1, ex2, ey2) = match snake.direction {
                            (1, 0) => (cell_size-4, 3, cell_size-4, cell_size-5), 
                            (-1, 0) => (2, 3, 2, cell_size-5),                     
                            (0, -1) => (3, 2, cell_size-5, 2),                     
                            _ => (3, cell_size-4, cell_size-5, cell_size-4),       
                        };
                        framebuffer::cz(p + ex1, o + ey1, 0xFF000000);
                        framebuffer::cz(p + ex2, o + ey2, 0xFF000000);
                    }
                }
            }
            
            
            let dg = fzn + snake.food.0 as u32 * cell_size;
            let hj = fzo + snake.food.1 as u32 * cell_size;
            if dg + cell_size < yu + ajs && hj + cell_size < xp + alo {
                framebuffer::fill_rect(dg + 2, hj + 2, cell_size - 4, cell_size - 4, 0xFFFF4444);
                framebuffer::cz(dg + cell_size/2, hj + 1, 0xFF00AA00); 
            }
            
            
            self.draw_text(yu as i32 + 8, xp as i32 + 8, "SNAKE", G_);
            
            
            let dyq = if snake.high_score > 0 {
                format!("Score: {}  Best: {}", snake.score, snake.high_score)
            } else {
                format!("Score: {}", snake.score)
            };
            self.draw_text(yu as i32 + ajs as i32 - 170, xp as i32 + 8, &dyq, AH_);
            
            if snake.game_over {
                
                let fh = yu + ajs / 2 - 60;
                let hk = xp + alo / 2 - 20;
                framebuffer::fill_rect(fh - 4, hk - 4, 128, 58, 0xCC000000);
                self.draw_text(fh as i32, hk as i32, "GAME OVER", 0xFFFF4444);
                let lvo = format!("Score: {}", snake.score);
                self.draw_text(fh as i32 + 4, hk as i32 + 18, &lvo, AH_);
                self.draw_text(fh as i32 - 8, hk as i32 + 36, "Press ENTER", BM_);
            } else if snake.paused {
                
                let fh = yu + ajs / 2 - 50;
                let hk = xp + alo / 2 - 20;
                framebuffer::fill_rect(fh - 4, hk - 4, 110, 48, 0xCC000000);
                self.draw_text(fh as i32 + 8, hk as i32, "PAUSED", 0xFFFFCC00);
                self.draw_text(fh as i32 - 4, hk as i32 + 20, "P to resume", BM_);
            } else {
                
                self.draw_text(yu as i32 + 8, xp as i32 + alo as i32 - 18, 
                               "Arrows to move | P pause", BM_);
            }
        }
    }
    
    
    fn htj(p: u32, o: u32, piece: i8) {
        let hdp = if piece < 0 { -piece } else { piece };
        let xr = piece > 0;

        let fill = if xr { 0xFFE8E0D0_u32 } else { 0xFF2A2A2A_u32 };
        let isz = if xr { 0xFF1A1A1A_u32 } else { 0xFF888888_u32 };

        
        
        let au: &[(u32, u32, u32, u32)] = match hdp {
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

        
        for &(x, y, w, h) in au {
            framebuffer::fill_rect(p + x - 1, o + y - 1, w + 2, h + 2, isz);
        }
        
        for &(x, y, w, h) in au {
            framebuffer::fill_rect(p + x, o + y, w, h, fill);
        }
        
        let hl = if xr { 0x66FFFFFF_u32 } else { 0x44FFFFFF_u32 };
        for &(x, y, w, h) in au {
            if w > 4 && h > 2 {
                framebuffer::fill_rect(p + x + 1, o + y + 1, 1, h - 2, hl);
            }
        }

        
        if hdp == 3 {
            let jgo = isz;
            framebuffer::fill_rect(p + 22, o + 14, 4, 1, jgo);
            framebuffer::fill_rect(p + 21, o + 15, 4, 1, jgo);
        }
    }

    
    fn draw_chess_game(&self, window: &Window) {
        let yu = window.x as u32 + 8;
        let xp = window.y as u32 + J_() + 4;
        let ajs = window.width.saturating_sub(16);
        let alo = window.height.saturating_sub(J_() + 8);
        
        if ajs < 200 || alo < 200 {
            return;
        }
        
        
        framebuffer::fill_rect(yu, xp, ajs, alo, 0xFF0A0E0B);
        
        if let Some(chess) = self.chess_states.get(&window.id) {
            
            let cell_size: u32 = 48;
            let tg = cell_size * 8;
            let un = yu + (ajs.saturating_sub(tg)) / 2;
            let ve = xp + 28;
            
            
            self.draw_text(yu as i32 + 8, xp as i32 + 6, "TRUSTCHESS", I_);
            
            
            let score = chess.material_score();
            let olq = if score > 0 {
                format!("+{}", score / 100)
            } else if score < 0 {
                format!("{}", score / 100)
            } else {
                String::from("=")
            };
            let gsw = if score > 0 { 0xFFFFFFFF } else if score < 0 { 0xFFCC4444 } else { Y_ };
            
            self.draw_text(yu as i32 + 96, xp as i32 + 6, &olq, gsw);
            
            
            let lel = match chess.ai_depth { 1 => "Easy", 2 => "Med", _ => "Hard" };
            self.draw_text(yu as i32 + 130, xp as i32 + 6, lel, Y_);
            
            
            if chess.timer_enabled {
                let kee = crate::chess::ChessState::format_time(chess.black_time_ms);
                let pvj = crate::chess::ChessState::format_time(chess.white_time_ms);
                
                let pjp = if !chess.white_turn && chess.timer_started { 0xFFCC4444 } else { Y_ };
                self.draw_text(un as i32 + tg as i32 + 8, ve as i32 + 4, &kee, pjp);
                crate::framebuffer::px(un + tg + 8, ve + 14, 'B', 0xFFCC4444);
                
                let pjq = if chess.white_turn && chess.timer_started { 0xFFFFFFFF } else { Y_ };
                self.draw_text(un as i32 + tg as i32 + 8, ve as i32 + tg as i32 - 20, &pvj, pjq);
                crate::framebuffer::px(un + tg + 8, ve + tg - 10, 'W', 0xFFFFFFFF);
            }
            
            
            for row in 0..8u32 {
                for col in 0..8u32 {
                    let cu = (row * 8 + col) as usize;
                    let p = un + col * cell_size;
                    let o = ve + row * cell_size;
                    
                    
                    let bhj = (row + col) % 2 == 0;
                    let mut bg = if bhj { 0xFF3D5A3D } else { 0xFF1A2E1A };
                    
                    
                    if chess.selected == Some(cu) {
                        bg = 0xFF5A7A2A; 
                    }
                    
                    
                    if chess.valid_moves.contains(&cu) {
                        bg = if bhj { 0xFF4A8A4A } else { 0xFF2A6A2A };
                    }
                    
                    
                    if chess.last_move_from == Some(cu) || chess.last_move_to == Some(cu) {
                        bg = if bhj { 0xFF5A6A3A } else { 0xFF3A4A2A };
                    }
                    
                    
                    if chess.cursor == cu {
                        bg = 0xFF00AA44; 
                    }
                    
                    framebuffer::fill_rect(p, o, cell_size, cell_size, bg);
                    
                    
                    let piece = chess.board[cu];
                    let gdq = chess.drag_from == Some(cu) && chess.dragging_piece.is_some();
                    if piece != 0 && !gdq {
                        Self::htj(p, o, piece);
                    }
                    
                    
                    if chess.valid_moves.contains(&cu) && (piece == 0 || gdq) {
                        let dnm = p + cell_size / 2 - 3;
                        let fst = o + cell_size / 2 - 3;
                        framebuffer::fill_rect(dnm, fst, 6, 6, 0xFF00FF66);
                    }
                    
                    
                    if chess.valid_moves.contains(&cu) && piece != 0 && !gdq {
                        
                        for dx in 0..4u32 {
                            framebuffer::cz(p + dx, o, 0xFF00FF66);
                            framebuffer::cz(p, o + dx, 0xFF00FF66);
                            framebuffer::cz(p + cell_size - 1 - dx, o, 0xFF00FF66);
                            framebuffer::cz(p + cell_size - 1, o + dx, 0xFF00FF66);
                            framebuffer::cz(p + dx, o + cell_size - 1, 0xFF00FF66);
                            framebuffer::cz(p, o + cell_size - 1 - dx, 0xFF00FF66);
                            framebuffer::cz(p + cell_size - 1 - dx, o + cell_size - 1, 0xFF00FF66);
                            framebuffer::cz(p + cell_size - 1, o + cell_size - 1 - dx, 0xFF00FF66);
                        }
                    }
                }
            }
            
            
            if let (Some(_from), Some(dp)) = (chess.drag_from, chess.dragging_piece) {
                let dx = chess.drag_pixel_x;
                let ad = chess.drag_pixel_y;
                if dx > 24 && ad > 24 {
                    Self::htj(dx as u32 - 24, ad as u32 - 24, dp);
                }
            }
            
            
            for i in 0..tg {
                framebuffer::cz(un + i, ve, Y_);
                framebuffer::cz(un + i, ve + tg, Y_);
            }
            for i in 0..tg + 1 {
                framebuffer::cz(un, ve + i, Y_);
                framebuffer::cz(un + tg, ve + i, Y_);
            }
            
            
            for c in 0..8u32 {
                let label = (b'a' + c as u8) as char;
                crate::framebuffer::px(un + c * cell_size + cell_size / 2 - 4, ve + tg + 4, label, BM_);
            }
            
            for r in 0..8u32 {
                let label = (b'8' - r as u8) as char;
                crate::framebuffer::px(un - 14, ve + r * cell_size + cell_size / 2 - 6, label, BM_);
            }
            
            
            let gk = ve + tg + 18;
            let ek = tg;
            let hs = 6u32;
            framebuffer::fill_rect(un, gk, ek, hs, 0xFF1A1A1A);
            
            let cbr = 2000i32; 
            let bqy = score.clamp(-cbr, cbr);
            let center = un + ek / 2;
            if bqy > 0 {
                let rb = ((bqy as u32) * (ek / 2)) / cbr as u32;
                framebuffer::fill_rect(center, gk, rb.min(ek / 2), hs, 0xFFFFFFFF);
            } else if bqy < 0 {
                let rb = (((-bqy) as u32) * (ek / 2)) / cbr as u32;
                let rb = rb.min(ek / 2);
                framebuffer::fill_rect(center - rb, gk, rb, hs, 0xFFCC4444);
            }
            
            framebuffer::fill_rect(center, gk, 1, hs, Y_);
            
            
            let status_y = gk + hs + 6;
            let ngo = match chess.phase {
                crate::chess::GamePhase::Check => DJ_,
                crate::chess::GamePhase::Checkmate => 0xFFFF4444,
                crate::chess::GamePhase::Stalemate => GN_,
                crate::chess::GamePhase::Promotion => RQ_,
                _ => I_,
            };
            self.draw_text(un as i32, status_y as i32, &chess.message, ngo);
            
            
            let dfw = if chess.white_turn { "White" } else { "Black" };
            let ecu = if chess.white_turn { 0xFFFFFFFF } else { 0xFFCC4444 };
            self.draw_text(un as i32 + tg as i32 - 60, status_y as i32, dfw, ecu);
            
            
            let mln = status_y as u32 + 18;
            let iev = if chess.move_history.len() > 6 { chess.move_history.len() - 6 } else { 0 };
            let mut aib = un as i32;
            for (i, m) in chess.move_history[iev..].iter().enumerate() {
                let num = iev + i + 1;
                let entry = format!("{}. {} ", num, m);
                self.draw_text(aib, mln as i32, &entry, Y_);
                aib += entry.len() as i32 * 8 + 4;
                if aib > un as i32 + tg as i32 - 40 {
                    break; 
                }
            }
            
            
            let epf = xp + alo - 30;
            self.draw_text(yu as i32 + 4, epf as i32,
                           "Mouse:Click/Drag  Arrows:Move  Enter:Select", BM_);
            self.draw_text(yu as i32 + 4, epf as i32 + 12,
                           "Esc:Desel  R:Reset  T:Timer  D:Difficulty", BM_);
        }
    }
    
    
    fn draw_binary_viewer(&self, window: &Window) {
        if let Some(state) = self.binary_viewer_states.get(&window.id) {
            let draw_text = |x: i32, y: i32, text: &str, color: u32| {
                self.draw_text(x, y, text, color);
            };
            crate::apps::binary_viewer::draw_binary_viewer(
                state,
                window.x, window.y,
                window.width, window.height,
                &draw_text,
            );
        }
    }

    
    pub fn open_binary_viewer(&mut self, path: &str) -> Result<u32, &'static str> {
        let analysis = crate::binary_analysis::hfe(path)?;
        let state = crate::apps::binary_viewer::BinaryViewerState::new(analysis, path);
        
        let ebu = alloc::format!("TrustView — {}", path);
        
        let id = self.create_window(&ebu, 50, 50, 1100, 650, WindowType::BinaryViewer);
        self.binary_viewer_states.insert(id, state);
        Ok(id)
    }

    
    pub fn open_lab_mode(&mut self) -> u32 {
        let id = self.create_window("TrustLab \u{2014} OS Introspection", 30, 30, 1200, 700, WindowType::LabMode);
        
        let dy = crate::framebuffer::width() as u32;
        let dw = crate::framebuffer::height() as u32;
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.saved_x = w.x;
            w.saved_y = w.y;
            w.saved_width = w.width;
            w.saved_height = w.height;
            w.x = 0;
            w.y = 0;
            w.width = dy;
            w.height = dw - V_();
            w.maximized = true;
        }
        self.focus_window(id);
        id
    }

    
    pub fn open_wifi_analyzer(&mut self) -> u32 {
        let id = self.create_window("TrustWave \u{2014} WiFi Analyzer", 30, 30, 1200, 700, WindowType::WifiAnalyzer);
        
        let dy = crate::framebuffer::width() as u32;
        let dw = crate::framebuffer::height() as u32;
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.saved_x = w.x;
            w.saved_y = w.y;
            w.saved_width = w.width;
            w.saved_height = w.height;
            w.x = 0;
            w.y = 0;
            w.width = dy;
            w.height = dw - V_();
            w.maximized = true;
        }
        self.focus_window(id);
        id
    }

    
    
    
    
    fn draw_image_viewer(&self, window: &Window) {
        let wx = window.x;
        let wy = window.y;
        let ca = window.width;
        let er = window.height;
        if ca < 60 || er < 80 { return; }
        
        let bn = wy + J_() as i32;
        let en = er.saturating_sub(J_() + 28); 
        let lv = if wx < 0 { 0u32 } else { wx as u32 };
        let ojw = bn as u32;
        
        
        framebuffer::fill_rect(lv + 2, ojw, ca.saturating_sub(4), en, 0xFF080808);
        
        if let Some(state) = self.image_viewer_states.get(&window.id) {
            if state.img_width > 0 && state.img_height > 0 && !state.pixels.is_empty() {
                
                let jsj = state.zoom as u32;
                let bfz = (state.img_width * jsj) / 100;
                let bsa = (state.img_height * jsj) / 100;
                
                
                let bny = (ca as i32 - bfz as i32) / 2 + state.pan_x;
                let bnz = (en as i32 - bsa as i32) / 2 + state.pan_y;
                
                
                let screen_w = framebuffer::width();
                let screen_h = framebuffer::height();
                
                for ad in 0..bsa {
                    let nn = bn + bnz + ad as i32;
                    if nn < bn || nn >= bn + en as i32 { continue; }
                    if nn < 0 || nn >= screen_h as i32 { continue; }
                    
                    
                    let aft = (ad * state.img_height) / bsa.max(1);
                    if aft >= state.img_height { continue; }
                    
                    for dx in 0..bfz {
                        let lw = wx + bny + dx as i32;
                        if lw < wx + 2 || lw >= wx + ca as i32 - 2 { continue; }
                        if lw < 0 || lw >= screen_w as i32 { continue; }
                        
                        let ahc = (dx * state.img_width) / bfz.max(1);
                        if ahc >= state.img_width { continue; }
                        
                        let ct = state.pixels[(aft * state.img_width + ahc) as usize];
                        
                        if (ct >> 24) == 0 { continue; }
                        framebuffer::cz(lw as u32, nn as u32, ct | 0xFF000000);
                    }
                }
                
                
                let status_y = (bn + en as i32) as u32;
                framebuffer::fill_rect(lv + 2, status_y, ca.saturating_sub(4), 24, 0xFF0A1A12);
                framebuffer::mn(lv + 2, status_y, ca.saturating_sub(4), 0xFF1A2A1A);
                
                let info = alloc::format!("{}x{} | Zoom: {}% | +/- to zoom | Arrows to pan", 
                    state.img_width, state.img_height, state.zoom);
                self.draw_text_smooth(wx + 10, status_y as i32 + 5, &info, BJ_);
            } else {
                
                self.draw_text_smooth(wx + ca as i32 / 2 - 60, bn + en as i32 / 2, "No image loaded", Q_);
                self.draw_text_smooth(wx + ca as i32 / 2 - 80, bn + en as i32 / 2 + 20, "Open a .bmp file to view it", Q_);
            }
        } else {
            self.draw_text_smooth(wx + 20, bn + 30, "Image Viewer — open a file", Q_);
        }
    }
    
    fn handle_image_viewer_key(&mut self, key: u8) {
        let fr = match self.windows.iter().find(|w| w.focused && w.window_type == WindowType::ImageViewer) {
            Some(w) => w.id,
            None => return,
        };
        if let Some(state) = self.image_viewer_states.get_mut(&fr) {
            match key {
                b'+' | b'=' => { state.zoom = (state.zoom + 10).min(500); }
                b'-' => { state.zoom = state.zoom.saturating_sub(10).max(10); }
                b'0' => { state.zoom = 100; state.pan_x = 0; state.pan_y = 0; } 
                _ => {
                    if key == crate::keyboard::T_ { state.pan_y += 20; }
                    else if key == crate::keyboard::S_ { state.pan_y -= 20; }
                    else if key == crate::keyboard::AI_ { state.pan_x += 20; }
                    else if key == crate::keyboard::AJ_ { state.pan_x -= 20; }
                }
            }
        }
    }

    
    
    
    
    fn draw_file_manager_icon_grid(&self, window: &Window) {
        let wx = window.x;
        let wy = window.y;
        let ca = window.width;
        let er = window.height;
        if ca < 80 || er < 100 { return; }
        
        let eiw = wy + J_() as i32;
        let lv = if wx < 0 { 0u32 } else { wx as u32 };
        
        
        let rq = self.fm_states.get(&window.id).map(|f| if f.sidebar_collapsed { 0u32 } else { f.sidebar_width }).unwrap_or(180);
        let aju = wx + rq as i32;
        let grid_w = ca.saturating_sub(rq);
        
        
        let kbp = 0xFF0A120Cu32;
        let djg = 0xFF0C140Cu32;
        let cuc = 0xFF081008u32;
        let dje = 0xFF0A3818u32;
        let fct = 0xFF80CC90u32;
        let cku = 0xFFDDAA30u32;
        let gbm = 0xFF60AA80u32;
        let dzd = 0xFF142014u32;
        
        
        let aiy = 36u32;
        framebuffer::fill_rect(lv, eiw as u32, ca, aiy, djg);
        
        let ed = eiw + 7;
        let of = 22u32;
        
        let ehj = self.fm_states.get(&window.id).map(|f| f.can_go_back()).unwrap_or(false);
        let jyz = if ehj { AH_ } else { 0xFF1A2A1A };
        draw_rounded_rect(wx + 8, ed, of, of, 4, 0xFF101810);
        self.draw_text(wx + 14, ed + 4, "<", jyz);
        
        let ehk = self.fm_states.get(&window.id).map(|f| f.can_go_forward()).unwrap_or(false);
        let mav = if ehk { AH_ } else { 0xFF1A2A1A };
        draw_rounded_rect(wx + 34, ed, of, of, 4, 0xFF101810);
        self.draw_text(wx + 40, ed + 4, ">", mav);
        
        draw_rounded_rect(wx + 60, ed, of, of, 4, 0xFF101810);
        iu(wx + 60, ed, of, of, 4, Q_);
        self.draw_text(wx + 66, ed + 4, "^", BJ_);
        
        
        let ccj = wx + 90;
        let cnx = (ca as i32).saturating_sub(106);
        if cnx > 10 {
            draw_rounded_rect(ccj, ed, cnx as u32, of, 6, 0xFF080E08);
            iu(ccj, ed, cnx as u32, of, 6, dzd);
            let ht = window.file_path.as_deref().unwrap_or("/");
            self.draw_text_smooth(ccj + 10, ed + 5, ht, I_);
        }
        
        framebuffer::mn(lv, (eiw + aiy as i32) as u32, ca, dzd);
        
        
        let wu = eiw + aiy as i32 + 1;
        let ano = er.saturating_sub(J_() + aiy + 1 + 26);
        
        if rq > 0 && ano > 20 {
            framebuffer::fill_rect(lv, wu as u32, rq, ano, cuc);
            let mut ak = wu + 8;
            let sy = 24i32;
            let am = wx + 6;
            let jgk = rq.saturating_sub(12);
            
            self.draw_text_smooth(am + 4, ak, "Quick Access", 0xFF3A7A4A);
            ak += 20;
            if let Some(kn) = self.fm_states.get(&window.id) {
                for (name, path) in kn.quick_access.iter() {
                    if ak + sy > wu + ano as i32 - 40 { break; }
                    let is_current = window.file_path.as_deref() == Some(path.as_str());
                    if is_current {
                        draw_rounded_rect(am, ak - 2, jgk, sy as u32, 4, 0xFF0C2810);
                        framebuffer::fill_rect(lv + 2, ak as u32, 3, (sy - 4) as u32, I_);
                    }
                    let bmv = (am + 12) as u32;
                    let bmw = (ak + 2) as u32;
                    framebuffer::fill_rect(bmv, bmw, 6, 2, cku);
                    framebuffer::fill_rect(bmv, bmw + 2, 12, 8, cku);
                    let c = if is_current { I_ } else { 0xFF50AA60 };
                    self.draw_text_smooth(am + 30, ak + 3, name, c);
                    ak += sy;
                }
            }
            ak += 6;
            framebuffer::mn(lv + 10, ak as u32, rq.saturating_sub(20), dzd);
            ak += 10;
            self.draw_text_smooth(am + 4, ak, "This PC", 0xFF3A7A4A);
            ak += 20;
            let drives = [("Local Disk (C:)", "/"), ("RAM Disk", "/tmp"), ("Devices", "/dev"), ("System", "/proc")];
            for (name, path) in &drives {
                if ak + sy > wu + ano as i32 - 4 { break; }
                let is_current = window.file_path.as_deref() == Some(*path);
                if is_current {
                    draw_rounded_rect(am, ak - 2, jgk, sy as u32, 4, 0xFF0C2810);
                    framebuffer::fill_rect(lv + 2, ak as u32, 3, (sy - 4) as u32, I_);
                }
                let c = if is_current { I_ } else { 0xFF50AA60 };
                self.draw_text_smooth(am + 30, ak + 3, name, c);
                ak += sy;
            }
            framebuffer::fill_rect(lv + rq - 1, wu as u32, 1, ano, dzd);
        }
        
        
        let bmq = wu as u32;
        let cah = ano.saturating_sub(2);
        if cah < 8 { return; }
        framebuffer::fill_rect(aju.max(0) as u32, bmq, grid_w, cah, kbp);
        
        
        let bcf = 90u32;
        let cam = 80u32;
        let cols = ((grid_w.saturating_sub(20)) / bcf).max(1);
        let glu = (grid_w.saturating_sub(cols * bcf)) / 2;
        
        
        let xb = 5usize.min(window.content.len());
        let cjk = if window.content.len() > xb + 2 { window.content.len() - 2 } else { window.content.len() };
        let axc: Vec<&str> = if cjk > xb {
            window.content[xb..cjk].iter().map(|j| j.as_str()).collect()
        } else { Vec::new() };
        
        if axc.is_empty() {
            self.draw_text_smooth(aju + 40, bmq as i32 + 30, "This folder is empty.", Q_);
        }
        
        let ndo = (cah / cam).max(1) as usize;
        let scroll_row = window.scroll_offset / cols as usize;
        
        for (idx, entry) in axc.iter().enumerate() {
            let row = idx / cols as usize;
            let col = idx % cols as usize;
            
            
            if row < scroll_row { continue; }
            let ekh = row - scroll_row;
            if ekh >= ndo { break; }
            
            let bqx = aju.max(0) as u32 + glu + col as u32 * bcf;
            let aho = bmq + ekh as u32 * cam;
            if aho + cam > bmq + cah { break; }
            
            let hd = idx == window.selected_index;
            let is_dir = entry.contains("[D]");
            
            
            if hd {
                draw_rounded_rect(bqx as i32 + 4, aho as i32 + 2, bcf - 8, cam - 4, 6, dje);
                iu(bqx as i32 + 4, aho as i32 + 2, bcf - 8, cam - 4, 6, 0xFF1A5A2A);
            }
            
            
            let adt = bqx + (bcf - 32) / 2;
            let adu = aho + 6;
            if is_dir {
                
                let br = if hd { 0xFFEEBB40 } else { cku };
                framebuffer::fill_rect(adt, adu, 16, 6, br);
                framebuffer::fill_rect(adt, adu + 6, 32, 20, br);
                framebuffer::fill_rect(adt + 2, adu + 10, 28, 14, 0xFF0A0A04);
                framebuffer::fill_rect(adt + 6, adu + 14, 16, 2, 0xFF302A10);
                framebuffer::fill_rect(adt + 6, adu + 18, 12, 2, 0xFF302A10);
            } else {
                
                let ext = Self::bbn(entry);
                let (br, fia, ext_label) = if ext.ends_with(".rs") || ext.ends_with(".c") || ext.ends_with(".h") {
                    (if hd { 0xFFFFAA66 } else { 0xFFDD7733 }, 0xFFFF6633, "RS")
                } else if ext.ends_with(".txt") || ext.ends_with(".md") || ext.ends_with(".log") {
                    (if hd { 0xFF88BBEE } else { 0xFF4488CC }, 0xFF4488CC, if ext.ends_with(".md") { "MD" } else { "TXT" })
                } else if ext.ends_with(".toml") || ext.ends_with(".json") || ext.ends_with(".cfg") {
                    (if hd { 0xFFEEDD66 } else { 0xFFDDAA00 }, 0xFFDDAA00, "CFG")
                } else if ext.ends_with(".bmp") || ext.ends_with(".png") || ext.ends_with(".jpg") {
                    (if hd { 0xFF66DD88 } else { 0xFF33BB66 }, 0xFF33BB66, "IMG")
                } else if ext.ends_with(".wav") || ext.ends_with(".mp3") {
                    (if hd { 0xFFFF88CC } else { 0xFFEE55AA }, 0xFFEE55AA, "SND")
                } else if ext.ends_with(".sh") || ext.ends_with(".elf") {
                    (if hd { 0xFFCC88FF } else { 0xFF9966DD }, 0xFF9966DD, "EXE")
                } else {
                    (if hd { 0xFF80DD99 } else { gbm }, 0xFF60AA80, "")
                };
                framebuffer::fill_rect(adt, adu, 28, 28, br);
                framebuffer::fill_rect(adt + 18, adu, 10, 10, 0xFF0A140A);
                framebuffer::fill_rect(adt + 18, adu, 2, 10, br);
                framebuffer::fill_rect(adt + 18, adu + 8, 10, 2, br);
                framebuffer::fill_rect(adt + 3, adu + 12, 22, 14, 0xFF040A04);
                
                framebuffer::fill_rect(adt, adu, 3, 28, fia);
                
                if !ext_label.is_empty() {
                    self.draw_text((adt + 5) as i32, (adu + 15) as i32, ext_label, 0xFF203020);
                }
            }
            
            
            let name = Self::bbn(entry);
            let nd = (bcf / 8).min(10) as usize;
            let cwr: String = if name.len() > nd {
                let mut j: String = name.chars().take(nd - 2).collect();
                j.push_str("..");
                j
            } else {
                String::from(name)
            };
            let bcv = bqx as i32 + (bcf as i32 - cwr.len() as i32 * 8) / 2;
            let nhq = (aho + cam - 20) as i32;
            let ayi = if hd { I_ } else { fct };
            self.draw_text_smooth(bcv, nhq, &cwr, ayi);
        }
        
        
        let status_y = (wy + er as i32).saturating_sub(24) as u32;
        framebuffer::fill_rect(lv, status_y, ca, 24, djg);
        framebuffer::mn(lv, status_y, ca, dzd);
        let dsx = axc.len();
        let crc = if dsx == 1 { String::from("1 item") } else { alloc::format!("{} items", dsx) };
        self.draw_text_smooth(aju + 10, status_y as i32 + 6, &crc, 0xFF406850);
    }
    
    fn bbn(entry: &str) -> &str {
        let jw = entry.trim();
        if let Some(bracket_end) = jw.find(']') {
            let fgm = if bracket_end + 1 < jw.len() { &jw[bracket_end + 1..] } else { "" };
            let au: Vec<&str> = fgm.split_whitespace().collect();
            if !au.is_empty() { au[0] } else { "???" }
        } else {
            jw
        }
    }

    
    
    
    
    fn handle_file_manager_click(&mut self, x: i32, y: i32, window_id: u32) {
        let (wt, wx, wy, ca, er, file_path_opt, anw, selected_idx) = {
            if let Some(w) = self.windows.iter().find(|w| w.id == window_id && w.window_type == WindowType::FileManager) {
                (w.window_type, w.x, w.y, w.width, w.height, w.file_path.clone(), w.content.len(), w.selected_index)
            } else { return; }
        };
        
        let bn = wy + J_() as i32;
        let aiy = 36i32;
        let rq = self.fm_states.get(&window_id).map(|f| if f.sidebar_collapsed { 0i32 } else { f.sidebar_width as i32 }).unwrap_or(180);
        
        
        let ed = bn + 7;
        let of = 22i32;
        
        
        if x >= wx + 8 && x < wx + 8 + of && y >= ed && y < ed + of {
            
            let jzb = self.fm_states.get_mut(&window_id).and_then(|f| f.go_back().map(|j| String::from(j)));
            if let Some(path) = jzb {
                self.navigate_file_manager_to(window_id, &path);
            }
            return;
        }
        
        if x >= wx + 34 && x < wx + 34 + of && y >= ed && y < ed + of {
            let may = self.fm_states.get_mut(&window_id).and_then(|f| f.go_forward().map(|j| String::from(j)));
            if let Some(path) = may {
                self.navigate_file_manager_to(window_id, &path);
            }
            return;
        }
        
        if x >= wx + 60 && x < wx + 60 + of && y >= ed && y < ed + of {
            self.navigate_file_manager("..");
            return;
        }
        
        
        let acm = if ca > 400 { 180i32 } else if ca > 300 { 120i32 } else { 0i32 };
        if acm > 0 {
            let am = wx + ca as i32 - acm - 8;
            if x >= am && x < am + acm && y >= ed && y < ed + of {
                if let Some(kn) = self.fm_states.get_mut(&window_id) {
                    kn.search_focused = true;
                }
                return;
            } else {
                
                if let Some(kn) = self.fm_states.get_mut(&window_id) {
                    kn.search_focused = false;
                }
            }
        }
        
        
        let wu = bn + aiy + 1;
        let awr = 24i32;
        let ho = wx + rq;
        let hy = ca as i32 - rq;
        if y >= wu && y < wu + awr && x >= ho {
            let cvj = ho + (hy * 52 / 100);
            let cvi = ho + (hy * 68 / 100);
            let dlb = ho + (hy * 82 / 100);
            
            let hlk: u8 = if hy > 420 && x >= dlb { 3 }
                else if hy > 300 && x >= cvi { 2 }
                else if hy > 200 && x >= cvj { 1 }
                else { 0 };
            
            if let Some(kn) = self.fm_states.get_mut(&window_id) {
                if kn.sort_column == hlk {
                    kn.sort_ascending = !kn.sort_ascending;
                } else {
                    kn.sort_column = hlk;
                    kn.sort_ascending = true;
                }
            }
            
            let path = file_path_opt.clone().unwrap_or_else(|| String::from("/"));
            self.refresh_file_manager(&path);
            return;
        }
        
        
        let ano = er.saturating_sub(J_() + aiy as u32 + 1 + 26);
        let status_y = bn + aiy + 1 + ano as i32;
        if y >= status_y && y < status_y + 24 && ca > 300 {
            let adb = wx + ca as i32 - 120;
            let aem = 24i32;
            
            if x >= adb && x < adb + aem {
                self.fm_view_modes.insert(window_id, FileManagerViewMode::List);
                return;
            }
            
            if x >= adb + aem + 4 && x < adb + aem * 2 + 4 {
                self.fm_view_modes.insert(window_id, FileManagerViewMode::IconGrid);
                return;
            }
            
            if x >= adb + (aem + 4) * 2 && x < adb + (aem + 4) * 2 + aem {
                self.fm_view_modes.insert(window_id, FileManagerViewMode::Details);
                return;
            }
        }
        
        
        let wu = bn + aiy + 1;
        if rq > 0 && x >= wx && x < wx + rq {
            let sy = 24i32;
            let mut ak = wu + 28; 
            
            
            if let Some(kn) = self.fm_states.get(&window_id) {
                let oaa: Vec<String> = kn.quick_access.iter().map(|(_, aa)| aa.clone()).collect();
                for (i, path) in oaa.iter().enumerate() {
                    if y >= ak && y < ak + sy {
                        crate::serial_println!("[FM] Sidebar click: Quick Access -> {}", path);
                        self.navigate_file_manager_to(window_id, path);
                        return;
                    }
                    ak += sy;
                }
            }
            
            
            ak += 36; 
            
            
            let drives = ["/", "/tmp", "/dev", "/proc"];
            for path in &drives {
                if y >= ak && y < ak + sy {
                    crate::serial_println!("[FM] Sidebar click: Drive -> {}", path);
                    self.navigate_file_manager_to(window_id, path);
                    return;
                }
                ak += sy;
            }
            return; 
        }
        
        
        let msp = self.fm_view_modes.get(&window_id).copied().unwrap_or(FileManagerViewMode::List) == FileManagerViewMode::IconGrid;
        let xb = 5usize.min(anw);
        let cjk = if anw > xb + 2 { anw - 2 } else { anw };
        let adp = cjk.saturating_sub(xb);
        
        if msp {
            
            let bmq = bn + aiy + 1;
            let bcf = 90i32;
            let cam = 80i32;
            let ho = wx + rq;
            let grid_w = ca as i32 - rq;
            let cols = ((grid_w - 20) / bcf).max(1);
            let glu = (grid_w - cols * bcf) / 2;
            let scroll_row = (self.windows.iter().find(|w| w.id == window_id).map(|w| w.scroll_offset).unwrap_or(0) / cols as usize) as i32;
            
            let sk = x - ho - glu;
            let qn = y - bmq;
            if sk >= 0 && qn >= 0 {
                let col = sk / bcf;
                let ekh = qn / cam;
                let cfs = ekh + scroll_row;
                let idx = cfs * cols + col;
                if idx >= 0 && (idx as usize) < adp {
                    let bau = idx as usize;
                    if bau == selected_idx && crate::mouse::erj() {
                        self.open_selected_file_at(window_id, bau);
                        return;
                    }
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == window_id) {
                        w.selected_index = bau;
                    }
                }
            }
        } else {
            
            let ho = wx + rq;
            let awr = 24i32;
            let gfr = wu + awr + 1;
            let ep = 26i32;
            
            let qn = y - gfr;
            if qn >= 0 && x >= ho {
                let scroll_offset = self.windows.iter().find(|w| w.id == window_id).map(|w| w.scroll_offset).unwrap_or(0);
                let bau = scroll_offset + (qn / ep) as usize;
                if bau < adp {
                    if bau == selected_idx && crate::mouse::erj() {
                        self.open_selected_file_at(window_id, bau);
                        return;
                    }
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == window_id) {
                        w.selected_index = bau;
                    }
                }
            }
        }
    }
    
    
    fn navigate_file_manager_to(&mut self, window_id: u32, path: &str) {
        
        let hce: Vec<u32> = self.windows.iter().filter(|w| w.focused).map(|w| w.id).collect();
        for w in &mut self.windows {
            w.focused = w.id == window_id;
        }
        
        if let Some(window) = self.windows.iter_mut().find(|w| w.id == window_id) {
            window.file_path = Some(String::from(path));
        }
        self.refresh_file_manager(path);
        
        if let Some(kn) = self.fm_states.get_mut(&window_id) {
            kn.push_history(path);
        }
        
        for w in &mut self.windows {
            w.focused = hce.contains(&w.id);
        }
    }
    
    fn open_selected_file_at(&mut self, window_id: u32, ado: usize) {
        let (filename, is_dir) = {
            if let Some(w) = self.windows.iter().find(|w| w.id == window_id) {
                let xb = 5usize.min(w.content.len());
                let abp = xb + ado;
                if abp < w.content.len().saturating_sub(2) {
                    let line = &w.content[abp];
                    let is_dir = line.contains("[D]");
                    let name = Self::bbn(line);
                    (String::from(name), is_dir)
                } else { return; }
            } else { return; }
        };
        
        if is_dir {
            self.navigate_file_manager(&filename);
        } else {
            self.open_file(&filename);
        }
    }

    
    
    
    
    fn qxp(&mut self, window_id: u32) {
        if let Some(w) = self.windows.iter().find(|w| w.id == window_id && w.window_type == WindowType::FileManager) {
            let xb = 5usize.min(w.content.len());
            let abp = xb + w.selected_index;
            if abp < w.content.len().saturating_sub(2) {
                let line = &w.content[abp];
                let is_dir = line.contains("[D]");
                let name = Self::bbn(line);
                if name == ".." { return; }
                let ht = w.file_path.clone().unwrap_or_else(|| String::from("/"));
                let kg = if ht == "/" {
                    alloc::format!("/{}", name)
                } else {
                    alloc::format!("{}/{}", ht, name)
                };
                self.drag_state = Some(Xx {
                    source_path: kg,
                    filename: String::from(name),
                    is_dir,
                    start_x: self.cursor_x,
                    start_y: self.cursor_y,
                    current_x: self.cursor_x,
                    current_y: self.cursor_y,
                    source_window_id: window_id,
                    active: true,
                });
                crate::serial_println!("[DnD] Started drag: {}", name);
            }
        }
    }
    
    fn update_drag(&mut self, x: i32, y: i32) {
        if let Some(ref mut drag) = self.drag_state {
            drag.current_x = x;
            drag.current_y = y;
        }
    }
    
    fn finish_drag(&mut self, x: i32, y: i32) {
        let lhj = self.drag_state.take();
        if let Some(drag) = lhj {
            
            let pdj = self.windows.iter()
                .filter(|w| w.window_type == WindowType::FileManager && w.id != drag.source_window_id)
                .find(|w| x >= w.x && x < w.x + w.width as i32 && y >= w.y && y < w.y + w.height as i32);
            
            if let Some(target) = pdj {
                let gyc = target.file_path.clone().unwrap_or_else(|| String::from("/"));
                let bfw = if gyc == "/" {
                    alloc::format!("/{}", drag.filename)
                } else {
                    alloc::format!("{}/{}", gyc, drag.filename)
                };
                
                
                if !drag.is_dir {
                    if let Ok(data) = crate::ramfs::bh(|fs| fs.read_file(&drag.source_path).map(|d| d.to_vec())) {
                        let _ = crate::ramfs::bh(|fs| fs.write_file(&bfw, &data));
                        crate::serial_println!("[DnD] Copied {} -> {}", drag.source_path, bfw);
                    }
                } else {
                    let _ = crate::ramfs::bh(|fs| fs.mkdir(&bfw));
                    crate::serial_println!("[DnD] Created dir: {}", bfw);
                }
                
                
                self.refresh_file_manager_by_id(target.id, &gyc);
            } else if y >= (self.height - V_()) as i32 {
                
                crate::serial_println!("[DnD] Dropped on taskbar, ignoring");
            } else {
                
                crate::serial_println!("[DnD] Dropped on desktop: {}", drag.filename);
            }
        }
    }
    
    fn draw_drag_ghost(&self) {
        if let Some(ref drag) = self.drag_state {
            if !drag.active { return; }
            let hc = drag.current_x;
            let jh = drag.current_y;
            
            
            framebuffer::co(hc as u32, jh as u32, 70, 22, 0x0C1410, 180);
            iu(hc, jh, 70, 22, 4, I_);
            
            
            if drag.is_dir {
                framebuffer::fill_rect((hc + 4) as u32, (jh + 4) as u32, 14, 14, 0xFFDDAA30);
            } else {
                framebuffer::fill_rect((hc + 4) as u32, (jh + 4) as u32, 12, 14, 0xFF60AA80);
            }
            
            
            let nd = 6;
            let name: String = drag.filename.chars().take(nd).collect();
            self.draw_text(hc + 22, jh + 5, &name, I_);
        }
    }
    
    fn refresh_file_manager_by_id(&mut self, sa: u32, path: &str) {
        
        let hce: Vec<u32> = self.windows.iter().filter(|w| w.focused).map(|w| w.id).collect();
        for w in &mut self.windows {
            w.focused = w.id == sa;
        }
        self.refresh_file_manager(path);
        
        for w in &mut self.windows {
            w.focused = hce.contains(&w.id);
        }
    }

    
    
    
    
    fn file_clipboard_copy(&mut self, cut: bool) {
        if let Some(w) = self.windows.iter().find(|w| w.focused && w.window_type == WindowType::FileManager) {
            let xb = 5usize.min(w.content.len());
            let abp = xb + w.selected_index;
            if abp < w.content.len().saturating_sub(2) {
                let line = &w.content[abp];
                let name = Self::bbn(line);
                if name == ".." { return; }
                let ht = w.file_path.clone().unwrap_or_else(|| String::from("/"));
                let kg = if ht == "/" {
                    alloc::format!("/{}", name)
                } else {
                    alloc::format!("{}/{}", ht, name)
                };
                let op = if cut { "Cut" } else { "Copied" };
                crate::serial_println!("[FM] {} file: {}", op, kg);
                self.file_clipboard = Some(Yv {
                    path: kg,
                    name: String::from(name),
                    is_cut: cut,
                });
                
                crate::keyboard::byb(name);
            }
        }
    }
    
    fn file_clipboard_paste(&mut self) {
        let clipboard = self.file_clipboard.take();
        if let Some(entry) = clipboard {
            let ht = self.windows.iter()
                .find(|w| w.focused && w.window_type == WindowType::FileManager)
                .and_then(|w| w.file_path.clone())
                .unwrap_or_else(|| String::from("/"));
            
            let mt = if ht == "/" {
                alloc::format!("/{}", entry.name)
            } else {
                alloc::format!("{}/{}", ht, entry.name)
            };
            
            if entry.is_cut {
                
                let _ = crate::ramfs::bh(|fs| fs.mv(&entry.path, &mt));
                crate::serial_println!("[FM] Moved {} -> {}", entry.path, mt);
            } else {
                
                if let Ok(data) = crate::ramfs::bh(|fs| fs.read_file(&entry.path).map(|d| d.to_vec())) {
                    let _ = crate::ramfs::bh(|fs| fs.write_file(&mt, &data));
                    crate::serial_println!("[FM] Pasted {} -> {}", entry.path, mt);
                }
                
                self.file_clipboard = Some(entry);
            }
            
            self.refresh_file_manager(&ht);
        }
    }

    
    
    
    
    fn draw_lock_screen(&self) {
        let dy = self.width;
        let dw = self.height;
        
        
        framebuffer::fill_rect(0, 0, dy, dw, 0xFF040808);
        
        
        let cols = dy / 10;
        for c in 0..cols {
            let seed = c.wrapping_mul(7919).wrapping_add(self.frame_count as u32);
            let awr = (seed % 20) + 3;
            let chm = c * 10;
            let kvm = (seed.wrapping_mul(13) % dw) as i32;
            for r in 0..awr {
                let cm = kvm + r as i32 * 14;
                if cm >= 0 && cm < dw as i32 - 14 {
                    let brightness = (255 - r * 12).max(20);
                    let color = (brightness << 8) | 0xFF000000;
                    let ehp = ((seed.wrapping_add(r)) % 26 + 65) as u8 as char;
                    let mut buf = [0u8; 4];
                    let fle = ehp.encode_utf8(&mut buf);
                    framebuffer::draw_text(fle, chm, cm as u32, color);
                }
            }
        }
        
        
        let he = 360u32;
        let ug = 280u32;
        let p = (dy - he) / 2;
        let o = (dw - ug) / 2;
        let ork = if self.lock_screen_shake > 0 {
            let bqf = (self.lock_screen_shake as i32 * 3) % 13 - 6;
            bqf
        } else { 0 };
        let p = (p as i32 + ork) as u32;
        
        
        framebuffer::co(p, o, he, ug, 0x0C1A12, 200);
        draw_rounded_rect(p as i32, o as i32, he, ug, 12, 0xFF0A1A0F);
        iu(p as i32, o as i32, he, ug, 12, Y_);
        
        
        let avk = (p + he / 2).saturating_sub(40) as i32;
        self.draw_text_smooth(avk, (o + 30) as i32, "TrustOS", I_);
        
        
        let auf = p + he / 2 - 12;
        let afi = o + 70;
        
        framebuffer::fill_rect(auf, afi, 24, 3, BJ_);
        framebuffer::fill_rect(auf, afi, 3, 16, BJ_);
        framebuffer::fill_rect(auf + 21, afi, 3, 16, BJ_);
        
        framebuffer::fill_rect(auf - 4, afi + 16, 32, 22, Y_);
        framebuffer::fill_rect(auf + 8, afi + 22, 8, 10, 0xFF040A08);
        
        
        let nak = (p + he / 2).saturating_sub(24) as i32;
        self.draw_text_smooth(nak, (afi + 48) as i32, "Locked", BM_);
        
        
        let time = &self.cached_time_str;
        if !time.is_empty() {
            let gyu = (p + he / 2).saturating_sub((time.len() as u32 * 8) / 2) as i32;
            self.draw_text_smooth(gyu, (o + 150) as i32, time, I_);
        }
        let dmi = &self.cached_date_str;
        if !dmi.is_empty() {
            let lbu = (p + he / 2).saturating_sub((dmi.len() as u32 * 8) / 2) as i32;
            self.draw_text_smooth(lbu, (o + 170) as i32, dmi, BM_);
        }
        
        
        let sv = o + 200;
        let caw = 200u32;
        let aua = p + (he - caw) / 2;
        draw_rounded_rect(aua as i32, sv as i32, caw, 30, 6, 0xFF081208);
        iu(aua as i32, sv as i32, caw, 30, 6, Q_);
        
        
        let dnn: String = self.lock_screen_input.chars().map(|_| '*').collect();
        if dnn.is_empty() {
            self.draw_text_smooth((aua + 8) as i32, (sv + 8) as i32, "Enter PIN...", Q_);
        } else {
            self.draw_text_smooth((aua + 8) as i32, (sv + 8) as i32, &dnn, I_);
        }
        
        
        if self.cursor_blink {
            let cx = aua + 8 + dnn.len() as u32 * 8;
            framebuffer::fill_rect(cx, sv + 6, 2, 18, I_);
        }
        
        
        self.draw_text_smooth((p + he / 2 - 80) as i32, (sv + 42) as i32, "Press Enter to unlock", Q_);
        
        
        if self.lock_screen_shake > 0 {
            self.draw_text_smooth((p + he / 2 - 50) as i32, (sv + 60) as i32, "Wrong PIN!", 0xFFCC4444);
        }
    }
    
    fn handle_lock_screen_key(&mut self, key: u8) {
        if self.lock_screen_shake > 0 {
            self.lock_screen_shake = self.lock_screen_shake.saturating_sub(1);
        }
        
        if key == 0x0D || key == 0x0A { 
            
            if self.lock_screen_input.is_empty() || self.lock_screen_input == "0000" || self.lock_screen_input == "1234" {
                self.lock_screen_active = false;
                self.lock_screen_input.clear();
                crate::serial_println!("[LOCK] Screen unlocked");
            } else {
                
                self.lock_screen_shake = 15;
                self.lock_screen_input.clear();
                crate::serial_println!("[LOCK] Wrong PIN");
            }
        } else if key == 0x08 { 
            self.lock_screen_input.pop();
        } else if key >= 0x20 && key < 0x7F && self.lock_screen_input.len() < 16 {
            self.lock_screen_input.push(key as char);
        }
    }

    
    
    
    
    fn draw_sys_tray_indicators(&self, dft: u32, tray_y: u32) {
        
        let jrc = dft;
        let jrd = tray_y + 2;
        let hci = if self.sys_wifi_connected { I_ } else { 0xFF553333 };
        
        for arc in 0..3u32 {
            let r = 3 + arc * 3;
            let cx = jrc + 8;
            let u = jrd + 12;
            
            for dhr in 0..8u32 {
                let dx = (dhr * r) / 8;
                let dof = (r * r).saturating_sub(dx * dx);
                
                let mut ad = 0u32;
                while (ad + 1) * (ad + 1) <= dof { ad += 1; }
                let p = cx + dx;
                let o = u.saturating_sub(ad);
                if p > 0 && o > 0 {
                    framebuffer::cz(p, o, hci);
                    
                    if cx >= dx {
                        framebuffer::cz(cx - dx, o, hci);
                    }
                }
            }
        }
        
        framebuffer::fill_rect(jrc + 7, jrd + 11, 3, 3, hci);
        
        
        let edz = dft + 22;
        let apm = tray_y + 3;
        let hbv = BJ_;
        
        framebuffer::fill_rect(edz, apm + 4, 4, 6, hbv);
        framebuffer::fill_rect(edz + 4, apm + 2, 3, 10, hbv);
        
        let pug = (self.sys_volume / 34).min(3); 
        for w in 0..pug {
            let pvk = edz + 9 + w * 3;
            let jra = 2 + w * 2;
            let pvl = apm + 7;
            framebuffer::fill_rect(pvk, pvl.saturating_sub(jra), 1, jra * 2, hbv);
        }
        if self.sys_volume == 0 {
            
            framebuffer::fill_rect(edz + 9, apm + 3, 1, 8, 0xFFCC4444);
            framebuffer::fill_rect(edz + 12, apm + 3, 1, 8, 0xFFCC4444);
        }
        
        
        let egl = dft + 44;
        let egm = tray_y + 4;
        let egk = 18u32;
        let hgy = 8u32;
        
        framebuffer::draw_rect(egl, egm, egk, hgy, Q_);
        
        framebuffer::fill_rect(egl + egk, egm + 2, 2, 4, Q_);
        
        let rb = ((self.sys_battery as u32 * (egk - 2)) / 100).max(1);
        let kam = if self.sys_battery > 50 { I_ }
            else if self.sys_battery > 20 { GN_ }
            else { DJ_ };
        framebuffer::fill_rect(egl + 1, egm + 1, rb, hgy - 2, kam);
        
        
        let kan = alloc::format!("{}%", self.sys_battery);
        self.draw_text((egl + egk + 5) as i32, egm as i32, &kan, Q_);
    }

    
    fn draw_file_manager_gui(&self, window: &Window) {
        
        let fei = self.fm_view_modes.get(&window.id).copied().unwrap_or(FileManagerViewMode::List);
        if fei == FileManagerViewMode::IconGrid {
            self.draw_file_manager_icon_grid(window);
            return;
        }
        
        let wx = window.x;
        let wy = window.y;
        let ca = window.width;
        let er = window.height;
        
        if ca < 120 || er < 140 { return; }
        
        let bn = wy + J_() as i32;
        let lv = wx.max(0) as u32;
        
        
        let cuc     = 0xFF081008u32;  
        let hhs = 0xFF0C2810u32;  
        let kbx = 0xFF0A1C0Cu32;  
        let fiv     = 0xFF0A120Cu32;  
        let djg     = 0xFF0C140Cu32;  
        let fiw      = 0xFF0C180Eu32;  
        let fiy    = 0xFF0A120Cu32;
        let fja     = 0xFF0C140Cu32;
        let fiz   = 0xFF0E1E10u32;  
        let dje    = 0xFF0A3818u32;  
        let kbw      = 0xFF060C06u32;  
        let jmf   = 0xFF50AA60u32;
        let gyk = 0xFF3A7A4Au32;  
        let fcu    = 0xFF50CC70u32;
        let fct      = 0xFF80CC90u32;
        let rg       = 0xFF406850u32;
        let cku    = 0xFFDDAA30u32;
        let gbm      = 0xFF60AA80u32;
        let bot      = 0xFF142014u32;
        let accent         = I_;
        
        
        let kn = self.fm_states.get(&window.id);
        let rq = kn.map(|f| if f.sidebar_collapsed { 0u32 } else { f.sidebar_width }).unwrap_or(180);
        let mmg = kn.and_then(|f| f.hover_index);
        
        
        let aiy = 36u32;
        framebuffer::fill_rect(lv, bn as u32, ca, aiy, djg);
        
        let ed = bn + 7;
        let of = 22u32;
        
        
        let ehj = kn.map(|f| f.can_go_back()).unwrap_or(false);
        let jza = if ehj { AH_ } else { 0xFF1A2A1A };
        draw_rounded_rect(wx + 8, ed, of, of, 4, 0xFF101810);
        if ehj { iu(wx + 8, ed, of, of, 4, Q_); }
        self.draw_text(wx + 14, ed + 4, "<", jza);
        
        
        let ehk = kn.map(|f| f.can_go_forward()).unwrap_or(false);
        let maw = if ehk { AH_ } else { 0xFF1A2A1A };
        draw_rounded_rect(wx + 34, ed, of, of, 4, 0xFF101810);
        if ehk { iu(wx + 34, ed, of, of, 4, Q_); }
        self.draw_text(wx + 40, ed + 4, ">", maw);
        
        
        draw_rounded_rect(wx + 60, ed, of, of, 4, 0xFF101810);
        iu(wx + 60, ed, of, of, 4, Q_);
        self.draw_text(wx + 66, ed + 4, "^", BJ_);
        
        
        let ccj = wx + 90;
        let acm = if ca > 400 { 180i32 } else if ca > 300 { 120i32 } else { 0i32 };
        let cnx = (ca as i32 - 100 - acm - 10).max(60);
        
        draw_rounded_rect(ccj, ed, cnx as u32, of, 6, 0xFF080E08);
        iu(ccj, ed, cnx as u32, of, 6, bot);
        
        
        let ht = window.file_path.as_deref().unwrap_or("/");
        let mut p = ccj + 10;
        let au: Vec<&str> = ht.split('/').filter(|j| !j.is_empty()).collect();
        
        
        self.draw_text_smooth(p, ed + 5, "\x07", 0xFF40AA50); 
        p += 12;
        
        if au.is_empty() {
            self.draw_text_smooth(p, ed + 5, "This PC", accent);
        } else {
            self.draw_text_smooth(p, ed + 5, "This PC", BJ_);
            p += 56;
            for (i, jn) in au.iter().enumerate() {
                if p > ccj + cnx - 30 { 
                    self.draw_text_smooth(p, ed + 5, "...", Q_);
                    break; 
                }
                
                self.draw_text_smooth(p, ed + 5, ">", 0xFF2A4A30);
                p += 12;
                let clo = i == au.len() - 1;
                let c = if clo { accent } else { BJ_ };
                self.draw_text_smooth(p, ed + 5, jn, c);
                p += (jn.len() as i32) * 8 + 6;
            }
        }
        
        
        if acm > 0 {
            let am = wx + ca as i32 - acm - 8;
            let search_focused = kn.map(|f| f.search_focused).unwrap_or(false);
            let okj = if search_focused { 0xFF081008 } else { kbw };
            let okk = if search_focused { accent } else { bot };
            draw_rounded_rect(am, ed, acm as u32, of, 6, okj);
            iu(am, ed, acm as u32, of, 6, okk);
            
            self.draw_text_smooth(am + 8, ed + 5, "\x0F", if search_focused { accent } else { Q_ });
            let query = kn.map(|f| f.search_query.as_str()).unwrap_or("");
            if query.is_empty() {
                self.draw_text_smooth(am + 22, ed + 5, "Search", rg);
            } else {
                self.draw_text_smooth(am + 22, ed + 5, query, fct);
            }
            
            if search_focused && (self.frame_count / 30) % 2 == 0 {
                let cursor_x = am + 22 + (query.len() as i32) * 8;
                framebuffer::fill_rect(cursor_x as u32, (ed + 4) as u32, 1, 14, accent);
            }
        }
        
        
        framebuffer::mn(lv, (bn + aiy as i32) as u32, ca, bot);
        
        
        let wu = bn + aiy as i32 + 1;
        let ano = er.saturating_sub(J_() + aiy + 1 + 26); 
        
        if rq > 0 && ano > 20 {
            framebuffer::fill_rect(lv, wu as u32, rq, ano, cuc);
            
            let mut ak = wu + 8;
            let sy = 24i32;
            let bvu = wx + 6;
            let gva = rq.saturating_sub(12);
            
            
            self.draw_text_smooth(bvu + 4, ak, "Quick Access", gyk);
            
            self.draw_text_smooth(bvu as i32 + gva as i32 - 8, ak, "v", gyk);
            ak += 20;
            
            if let Some(fm_s) = kn {
                for (i, (name, path)) in fm_s.quick_access.iter().enumerate() {
                    if ak + sy > wu + ano as i32 - 40 { break; }
                    
                    let is_current = window.file_path.as_deref() == Some(path.as_str());
                    let vl = fm_s.sidebar_selected == i as i32;
                    
                    
                    let dyb = if is_current { hhs } else if vl { kbx } else { cuc };
                    if is_current || vl {
                        draw_rounded_rect(bvu, ak - 2, gva, sy as u32, 4, dyb);
                    }
                    
                    if is_current {
                        framebuffer::fill_rect(lv + 2, ak as u32, 3, (sy - 4) as u32, accent);
                    }
                    
                    
                    let bmv = (bvu + 12) as u32;
                    let bmw = (ak + 2) as u32;
                    framebuffer::fill_rect(bmv, bmw, 6, 2, cku);
                    framebuffer::fill_rect(bmv, bmw + 2, 12, 8, cku);
                    framebuffer::fill_rect(bmv + 1, bmw + 4, 10, 5, 0xFF0A0A04);
                    
                    let ayi = if is_current { accent } else { jmf };
                    self.draw_text_smooth(bvu + 30, ak + 3, name, ayi);
                    
                    ak += sy;
                }
            }
            
            
            ak += 6;
            framebuffer::mn(lv + 10, ak as u32, rq.saturating_sub(20), bot);
            ak += 10;
            
            
            self.draw_text_smooth(bvu + 4, ak, "This PC", gyk);
            ak += 20;
            
            
            let drives = [
                ("\x07", "Local Disk (C:)", "/"),
                ("\x07", "RAM Disk",        "/tmp"),
                ("\x07", "Devices",         "/dev"),
                ("\x07", "System",          "/proc"),
            ];
            
            for (icon, name, path) in &drives {
                if ak + sy > wu + ano as i32 - 4 { break; }
                let is_current = window.file_path.as_deref() == Some(*path);
                
                if is_current {
                    draw_rounded_rect(bvu, ak - 2, gva, sy as u32, 4, hhs);
                    framebuffer::fill_rect(lv + 2, ak as u32, 3, (sy - 4) as u32, accent);
                }
                
                
                let bmv = (bvu + 12) as u32;
                let bmw = (ak + 2) as u32;
                framebuffer::fill_rect(bmv, bmw, 12, 10, 0xFF406050);
                framebuffer::fill_rect(bmv + 1, bmw + 1, 10, 3, 0xFF60AA80);
                framebuffer::fill_rect(bmv + 4, bmw + 5, 4, 3, 0xFF80CC90);
                
                let c = if is_current { accent } else { jmf };
                self.draw_text_smooth(bvu + 30, ak + 3, name, c);
                ak += sy;
            }
            
            
            framebuffer::fill_rect(lv + rq - 1, wu as u32, 1, ano, bot);
        }
        
        
        let ho = wx + rq as i32;
        let hy = ca.saturating_sub(rq);
        
        
        let awr = 24u32;
        framebuffer::fill_rect((ho.max(0)) as u32, wu as u32, hy, awr, fiw);
        
        
        let hmn = ho + 36;
        let cvj = ho + (hy as i32 * 52 / 100);
        let cvi = ho + (hy as i32 * 68 / 100);
        let dlb = ho + (hy as i32 * 82 / 100);
        
        let axm = wu + 5;
        
        
        let dzt = kn.map(|f| f.sort_column).unwrap_or(0);
        let gvo = kn.map(|f| f.sort_ascending).unwrap_or(true);
        let gvn = if gvo { "v" } else { "^" };
        
        
        self.draw_text_smooth(hmn, axm, "Name", fcu);
        if dzt == 0 { self.draw_text_smooth(hmn + 40, axm, gvn, Q_); }
        
        if hy > 200 {
            framebuffer::fill_rect(cvj as u32 - 2, wu as u32 + 4, 1, awr - 8, bot);
            self.draw_text_smooth(cvj, axm, "Type", fcu);
            if dzt == 1 { self.draw_text_smooth(cvj + 36, axm, gvn, Q_); }
        }
        if hy > 300 {
            framebuffer::fill_rect(cvi as u32 - 2, wu as u32 + 4, 1, awr - 8, bot);
            self.draw_text_smooth(cvi, axm, "Size", fcu);
            if dzt == 2 { self.draw_text_smooth(cvi + 36, axm, gvn, Q_); }
        }
        if hy > 420 {
            framebuffer::fill_rect(dlb as u32 - 2, wu as u32 + 4, 1, awr - 8, bot);
            self.draw_text_smooth(dlb, axm, "Open with", fcu);
        }
        
        framebuffer::mn((ho.max(0)) as u32, (wu + awr as i32) as u32, hy, bot);
        
        
        let gc = wu + awr as i32 + 1;
        let abc = ano.saturating_sub(awr + 27); 
        if abc < 8 { return; }
        
        framebuffer::fill_rect((ho.max(0)) as u32, gc as u32, hy, abc, fiv);
        
        let ep = 26u32; 
        let aac = (abc / ep).max(1) as usize;
        
        
        let xb = 5usize.min(window.content.len());
        let cjk = if window.content.len() > xb + 2 { window.content.len() - 2 } else { window.content.len() };
        let axc: Vec<&str> = if cjk > xb {
            window.content[xb..cjk].iter().map(|j| j.as_str()).collect()
        } else { Vec::new() };
        
        if axc.is_empty() {
            self.draw_text_smooth(ho + 30, gc + 30, "This folder is empty.", rg);
            self.draw_text_smooth(ho + 30, gc + 50, "Press N to create a file, D for a folder.", Q_);
        }
        
        let scroll = window.scroll_offset;
        let hbq = axc.len().min(aac);
        
        for pt in 0..hbq {
            let ado = scroll + pt;
            if ado >= axc.len() { break; }
            let line = axc[ado];
            let cm = gc as u32 + (pt as u32) * ep;
            if cm + ep > gc as u32 + abc { break; }
            
            let hd = ado == window.selected_index;
            let is_dir = line.contains("[D]");
            let vl = mmg == Some(ado);
            
            
            let dyb = if hd {
                dje
            } else if vl {
                fiz
            } else if pt % 2 == 0 {
                fiy
            } else {
                fja
            };
            framebuffer::fill_rect((ho.max(0)) as u32, cm, hy, ep, dyb);
            
            
            if hd {
                
                framebuffer::fill_rect((ho.max(0)) as u32, cm + 3, 3, ep - 6, accent);
                
                iu(ho, cm as i32, hy, ep, 3, 0xFF1A4A28);
            }
            
            let ie = (cm + 6) as i32;
            let jbn = if hd { accent } else { fct };
            
            
            let bi = (ho + 10).max(0) as u32;
            let gg = cm + 3;
            let arb = 20u32;
            
            if is_dir {
                
                let br = if hd { 0xFFEECC50 } else { cku };
                let emi = if hd { 0xFFCCAA30 } else { 0xFFBB8820 };
                
                framebuffer::fill_rect(bi, gg, arb / 2, 4, br);
                
                framebuffer::fill_rect(bi, gg + 4, arb, arb - 4, br);
                
                framebuffer::fill_rect(bi + 2, gg + 7, arb - 4, arb - 10, emi);
                
                framebuffer::fill_rect(bi + 4, gg + 9, arb - 8, 1, 0xFF0A0A04);
                framebuffer::fill_rect(bi + 4, gg + 12, arb / 2, 1, 0xFF0A0A04);
            } else {
                
                let br = if hd { 0xFF80DDAA } else { gbm };
                let emi = 0xFF0A140A;
                
                framebuffer::fill_rect(bi + 2, gg, arb - 6, arb, br);
                
                framebuffer::fill_rect(bi + arb - 8, gg, 4, 6, emi);
                framebuffer::fill_rect(bi + arb - 8, gg, 1, 6, br);
                framebuffer::fill_rect(bi + arb - 8, gg + 5, 4, 1, br);
                
                framebuffer::fill_rect(bi + 4, gg + 8, arb - 10, arb - 10, emi);
                
                framebuffer::fill_rect(bi + 5, gg + 10, 8, 1, 0xFF1A3A1A);
                framebuffer::fill_rect(bi + 5, gg + 13, 6, 1, 0xFF1A3A1A);
                framebuffer::fill_rect(bi + 5, gg + 16, 7, 1, 0xFF1A3A1A);
                
                
                let awz = Self::bbn(line);
                let fia = if awz.ends_with(".rs") { 0xFFFF6633 }       
                    else if awz.ends_with(".txt") { 0xFF4488CC }               
                    else if awz.ends_with(".md") { 0xFF5599DD }                
                    else if awz.ends_with(".toml") { 0xFF8866BB }              
                    else if awz.ends_with(".json") { 0xFFDDAA00 }              
                    else if awz.ends_with(".html") || awz.ends_with(".htm") { 0xFFEE6633 }
                    else if awz.ends_with(".css") { 0xFF3399EE }
                    else if awz.ends_with(".png") || awz.ends_with(".jpg") || awz.ends_with(".bmp") { 0xFF33BB66 }
                    else if awz.ends_with(".mp3") || awz.ends_with(".wav") { 0xFFEE55AA }
                    else { 0xFF446644 };
                framebuffer::fill_rect(bi + 3, gg + arb - 5, 6, 4, fia);
            }
            
            
            let jw = line.trim();
            let (name_str, ws, td, prog_str) = if let Some(bracket_end) = jw.find(']') {
                let fgm = if bracket_end + 1 < jw.len() { &jw[bracket_end + 1..] } else { "" };
                let au: Vec<&str> = fgm.split_whitespace().collect();
                (
                    if !au.is_empty() { au[0] } else { "???" },
                    if au.len() > 1 { au[1] } else { "" },
                    if au.len() > 2 { au[2] } else { "" },
                    if au.len() > 3 { au[3] } else { "" },
                )
            } else {
                (jw, "", "", "")
            };
            
            
            
            let bcv = ho + 36;
            if let Some(dot_pos) = name_str.rfind('.') {
                let base = &name_str[..dot_pos];
                let ext = &name_str[dot_pos..];
                self.draw_text_smooth(bcv, ie, base, jbn);
                let ltk = bcv + (base.len() as i32) * 8;
                self.draw_text_smooth(ltk, ie, ext, if hd { BJ_ } else { rg });
            } else {
                self.draw_text_smooth(bcv, ie, name_str, jbn);
            }
            
            
            if hy > 200 {
                let hah = if is_dir { "File folder" } else {
                    match name_str.rsplit('.').next() {
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
                        _ => ws,
                    }
                };
                let wo = if hd { BJ_ } else { 0xFF50886A };
                self.draw_text_smooth(cvj, ie, hah, wo);
            }
            
            
            if hy > 300 {
                let otj = if is_dir {
                    String::from("")
                } else if let Ok(bytes) = td.parse::<u64>() {
                    if bytes < 1024 { alloc::format!("{} B", bytes) }
                    else if bytes < 1024 * 1024 { alloc::format!("{} KB", bytes / 1024) }
                    else { alloc::format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)) }
                } else {
                    String::from(td)
                };
                let dr = if hd { BJ_ } else { 0xFF50886A };
                self.draw_text_smooth(cvi, ie, &otj, dr);
            }
            
            
            if hy > 420 {
                let pc = if hd { Q_ } else { 0xFF406050 };
                self.draw_text_smooth(dlb, ie, prog_str, pc);
            }
            
            
            framebuffer::mn((ho.max(0)) as u32, cm + ep - 1, hy, 0xFF0E160E);
        }
        
        
        if axc.len() > aac && abc > 20 {
            let gsn = 5u32;
            let yc = (ho as u32 + hy).saturating_sub(gsn + 2);
            let ada = abc.saturating_sub(4);
            framebuffer::fill_rect(yc, gc as u32 + 2, gsn, ada, 0xFF0A160C);
            let av = axc.len() as u32;
            let visible = aac as u32;
            let zo = ((visible * ada) / av.max(1)).max(20).min(ada);
            let aab = av.saturating_sub(visible);
            let akn = if aab > 0 {
                gc as u32 + 2 + ((scroll as u32 * ada.saturating_sub(zo)) / aab)
            } else {
                gc as u32 + 2
            };
            draw_rounded_rect(yc as i32, akn as i32, gsn, zo, 2, 0xFF204030);
        }
        
        
        let status_y = (wu + ano as i32) as u32;
        let aej = 24u32;
        framebuffer::fill_rect(lv, status_y, ca, aej, djg);
        framebuffer::mn(lv, status_y, ca, bot);
        
        
        let dsx = axc.len();
        let crc = if dsx == 1 {
            String::from("1 item")
        } else {
            alloc::format!("{} items", dsx)
        };
        self.draw_text_smooth(wx + rq as i32 + 10, status_y as i32 + 6, &crc, rg);
        
        
        if window.selected_index < axc.len() {
            let jeh = Self::bbn(axc[window.selected_index]);
            if jeh != ".." {
                let onf = alloc::format!("| {}", jeh);
                self.draw_text_smooth(wx + rq as i32 + 80, status_y as i32 + 6, &onf, Q_);
            }
        }
        
        
        if ca > 300 {
            let apl = status_y as i32 + 3;
            let adb = wx + ca as i32 - 120;
            let aem = 24i32;
            
            
            let ikl = fei == FileManagerViewMode::List;
            let gfp = if ikl { accent } else { Q_ };
            draw_rounded_rect(adb, apl, aem as u32, 18, 3, if ikl { 0xFF102810 } else { 0xFF0A140A });
            
            framebuffer::fill_rect((adb + 5) as u32, (apl + 4) as u32, 14, 2, gfp);
            framebuffer::fill_rect((adb + 5) as u32, (apl + 8) as u32, 14, 2, gfp);
            framebuffer::fill_rect((adb + 5) as u32, (apl + 12) as u32, 14, 2, gfp);
            
            
            let icu = fei == FileManagerViewMode::IconGrid;
            let eom = if icu { accent } else { Q_ };
            draw_rounded_rect(adb + aem + 4, apl, aem as u32, 18, 3, if icu { 0xFF102810 } else { 0xFF0A140A });
            
            framebuffer::fill_rect((adb + aem + 8) as u32, (apl + 4) as u32, 6, 5, eom);
            framebuffer::fill_rect((adb + aem + 16) as u32, (apl + 4) as u32, 6, 5, eom);
            framebuffer::fill_rect((adb + aem + 8) as u32, (apl + 11) as u32, 6, 5, eom);
            framebuffer::fill_rect((adb + aem + 16) as u32, (apl + 11) as u32, 6, 5, eom);
            
            
            let hrs = fei == FileManagerViewMode::Details;
            let cwk = if hrs { accent } else { Q_ };
            draw_rounded_rect(adb + (aem + 4) * 2, apl, aem as u32, 18, 3, if hrs { 0xFF102810 } else { 0xFF0A140A });
            
            framebuffer::fill_rect((adb + (aem + 4) * 2 + 5) as u32, (apl + 4) as u32, 3, 2, cwk);
            framebuffer::fill_rect((adb + (aem + 4) * 2 + 10) as u32, (apl + 4) as u32, 8, 2, cwk);
            framebuffer::fill_rect((adb + (aem + 4) * 2 + 5) as u32, (apl + 8) as u32, 3, 2, cwk);
            framebuffer::fill_rect((adb + (aem + 4) * 2 + 10) as u32, (apl + 8) as u32, 8, 2, cwk);
            framebuffer::fill_rect((adb + (aem + 4) * 2 + 5) as u32, (apl + 12) as u32, 3, 2, cwk);
            framebuffer::fill_rect((adb + (aem + 4) * 2 + 10) as u32, (apl + 12) as u32, 8, 2, cwk);
        }
    }

    
    
    
    fn draw_music_player(&self, window: &Window) {
        let wx = window.x as u32;
        let wy = window.y as u32 + J_();
        let ca = window.width;
        let er = window.height.saturating_sub(J_());

        if ca < 80 || er < 80 { return; }

        
        framebuffer::co(wx, wy, ca, er, 0x060D0A, 210);
        
        framebuffer::co(wx + 1, wy + 1, ca - 2, 1, 0x00FF66, 30);
        framebuffer::co(wx + 1, wy + er - 1, ca - 2, 1, 0x00FF66, 18);
        framebuffer::co(wx, wy + 1, 1, er - 2, 0x00FF66, 22);
        framebuffer::co(wx + ca - 1, wy + 1, 1, er - 2, 0x00FF66, 22);

        let state = match self.music_player_states.get(&window.id) {
            Some(j) => j,
            None => return,
        };

        let pad = 10u32;
        let lp = wx + pad;
        let rn = ca.saturating_sub(pad * 2);
        let aq = crate::graphics::scaling::agg() as u32;

        
        
        
        let dtk = wy + 6;
        
        self.draw_text(lp as i32, dtk as i32, "LIBRARY", 0xFF44886A);
        if state.num_tracks > 0 {
            let cht = alloc::format!("{} tracks", state.num_tracks);
            let foq = (lp + rn).saturating_sub(cht.len() as u32 * aq);
            self.draw_text(foq as i32, dtk as i32, &cht, 0xFF336655);
        }

        let gc = dtk + 16;
        let aac = 5usize;
        let ep = 20u32;
        let abc = if state.num_tracks == 0 { ep } else { (state.num_tracks.min(aac) as u32) * ep };

        
        framebuffer::co(lp, gc, rn, abc, 0x0A1A12, 180);
        
        framebuffer::co(lp, gc, rn, 1, 0x00FF66, 18);
        framebuffer::co(lp, gc + abc - 1, rn, 1, 0x00FF66, 12);

        if state.num_tracks == 0 {
            self.draw_text(lp as i32 + 8, (gc + 4) as i32, "No tracks found", 0xFF556655);
        } else {
            let scroll = state.track_list_scroll.min(state.num_tracks.saturating_sub(aac));
            for pt in 0..aac {
                let ecl = scroll + pt;
                if ecl >= state.num_tracks { break; }
                let cm = gc + pt as u32 * ep;
                let is_current = ecl == state.current_track && state.state != PlaybackState::Stopped;

                
                if is_current {
                    framebuffer::co(lp + 1, cm + 1, rn - 2, ep - 2, 0x00AA44, 40);
                }

                
                let rw = alloc::format!("{}.", ecl + 1);
                let dvm = if is_current { 0xFF00FFAA } else { 0xFF446655 };
                self.draw_text(lp as i32 + 6, (cm + 3) as i32, &rw, dvm);

                
                let name = if ecl < state.track_names.len() {
                    &state.track_names[ecl]
                } else {
                    "Unknown"
                };
                let nd = ((rn - 30) / aq) as usize;
                let cwr = if name.len() > nd {
                    &name[..nd.min(name.len())]
                } else {
                    name
                };
                let ayi = if is_current { 0xFF00FFCC } else { 0xFF88BBAA };
                self.draw_text(lp as i32 + 26, (cm + 3) as i32, cwr, ayi);

                
                if is_current && state.state == PlaybackState::Playing {
                    self.draw_text(lp as i32 + rn as i32 - 14, (cm + 3) as i32, ">", 0xFF00FF88);
                }
            }
        }

        
        
        
        let evi = gc + abc + 10;
        self.draw_text(lp as i32, evi as i32, "NOW PLAYING", 0xFF336655);

        
        let dzs = evi + 16;
        let title = &state.song_title;
        self.draw_text(lp as i32, dzs as i32, title, 0xFF00FFAA);
        self.draw_text(lp as i32 + 1, dzs as i32, title, 0xFF00FFAA);

        
        let status_y = dzs + 16;
        let status = match state.state {
            PlaybackState::Playing => "PLAYING",
            PlaybackState::Paused  => "PAUSED",
            PlaybackState::Stopped => "STOPPED",
        };
        let bdw = match state.state {
            PlaybackState::Playing => 0xFF00CC66,
            PlaybackState::Paused  => 0xFF00AA88,
            PlaybackState::Stopped => 0xFF666666,
        };
        self.draw_text(lp as i32, status_y as i32, status, bdw);

        
        let bbi = (state.elapsed_ms / 1000) as u32;
        let bee = (state.total_ms / 1000) as u32;
        let time_str = alloc::format!(
            "{}:{:02} / {}:{:02}",
            bbi / 60, bbi % 60,
            bee / 60, bee % 60
        );
        let gyu = (lp + rn).saturating_sub(time_str.len() as u32 * aq);
        self.draw_text(gyu as i32, status_y as i32, &time_str, 0xFF88CCAA);

        
        let azc = status_y + 18;
        let gop = 4u32;
        framebuffer::co(lp, azc, rn, gop, 0x1A3322, 200);
        if state.total_ms > 0 {
            let rb = ((state.elapsed_ms as u64 * rn as u64) / state.total_ms.max(1) as u64) as u32;
            let rb = rb.min(rn);
            if rb > 0 {
                framebuffer::fill_rect(lp, azc, rb, gop, 0xFF00FF88);
                if rb > 2 {
                    framebuffer::co(lp + rb - 2, azc.saturating_sub(1), 4, gop + 2, 0x00FF88, 120);
                }
            }
        }

        
        let bpy = azc + 12;
        let bwu = 60u32;
        framebuffer::co(lp, bpy, rn, bwu, 0x030908, 160);
        framebuffer::co(lp, bpy, rn, 1, 0x00FF66, 20);
        framebuffer::co(lp, bpy + bwu - 1, rn, 1, 0x00FF66, 12);

        let ags = bpy + bwu / 2;
        let kh = (bwu / 2 - 3) as f32;

        if state.state == PlaybackState::Playing || state.state == PlaybackState::Paused {
            let eus = rn.min(128) as usize;
            let egs = state.beat;
            for i in 0..eus {
                let pud = (state.wave_idx + i) % 128;
                let sample = state.waveform[pud];
                let ank = sample * (1.0 + egs * 0.5);
                let hdd = (ank * kh).max(-kh).min(kh) as i32;
                let p = lp + i as u32;
                let o = (ags as i32 + hdd) as u32;
                let o = o.max(bpy + 2).min(bpy + bwu - 3);

                let iaq = 0xCCu32;
                let hgh = (egs * 180.0) as u32;
                let ixo = (state.energy * 60.0).min(60.0) as u32;
                let center = ags;
                if o < center {
                    for bqc in o..center {
                        let ln = 1.0 - ((center - bqc) as f32 / kh).min(1.0) * 0.4;
                        let c = 0xFF000000 | (((ixo as f32 * ln) as u32).min(0xFF) << 16)
                            | (((iaq as f32 * ln) as u32).min(0xFF) << 8)
                            | ((hgh as f32 * ln) as u32).min(0xFF);
                        framebuffer::cz(p, bqc, c);
                    }
                } else {
                    for bqc in center..=o {
                        let ln = 1.0 - ((bqc - center) as f32 / kh).min(1.0) * 0.4;
                        let c = 0xFF000000 | (((ixo as f32 * ln) as u32).min(0xFF) << 16)
                            | (((iaq as f32 * ln) as u32).min(0xFF) << 8)
                            | ((hgh as f32 * ln) as u32).min(0xFF);
                        framebuffer::cz(p, bqc, c);
                    }
                }
                framebuffer::cz(p, o, 0xFF00FFCC);
            }
            if egs > 0.3 {
                let lws = ((egs - 0.3) * 50.0) as u32;
                framebuffer::co(lp, bpy, rn, bwu, 0x00FF88, lws);
            }
        } else {
            framebuffer::fill_rect(lp + 4, ags, rn - 8, 1, 0xFF223322);
        }

        
        let cgd = bpy + bwu + 4;
        let hs = 14u32;
        let ek = rn / 4 - 3;
        let apt = [
            (state.sub_bass, 0xFF00FF44, "SB"),
            (state.bass, 0xFF00CC88, "BA"),
            (state.mid, 0xFF00AACC, "MD"),
            (state.treble, 0xFF8866FF, "TR"),
        ];
        for (bal, (level, color, label)) in apt.iter().enumerate() {
            let bx = lp + bal as u32 * (ek + 3);
            framebuffer::co(bx, cgd, ek, hs, 0x0E1E14, 160);
            let fill = (level.min(1.0) * ek as f32) as u32;
            if fill > 0 {
                framebuffer::fill_rect(bx, cgd, fill, hs, *color);
                framebuffer::co(bx, cgd, fill, hs, 0xFFFFFF, 12);
            }
            self.draw_text(bx as i32 + 2, cgd as i32 + 2, label, 0xFF99BB99);
        }

        
        
        
        let aqc = cgd + hs + 8;
        let hn = 28u32;

        
        let azt = 36u32;
        let cog = 64u32;
        let gap = 4u32;
        let gzu = azt * 3 + cog + gap * 3;
        let haa = lp + (rn.saturating_sub(gzu)) / 2;

        
        fn afc(this: &Desktop, bx: u32, dc: u32, fv: u32, ov: u32, label: &str, bg: u32, border: u32, crl: u32) {
            let aq = crate::graphics::scaling::agg() as u32;
            
            framebuffer::co(bx, dc, fv, ov, bg, 210);
            
            framebuffer::co(bx + 1, dc, fv - 2, 1, border, 80);
            
            framebuffer::co(bx + 1, dc + ov - 1, fv - 2, 1, 0x000000, 60);
            
            framebuffer::co(bx, dc + 1, 1, ov - 2, border, 30);
            framebuffer::co(bx + fv - 1, dc + 1, 1, ov - 2, border, 30);
            
            framebuffer::co(bx + 1, dc + 1, fv - 2, 2, 0xFFFFFF, 12);
            
            let acy = label.len() as u32 * aq;
            let bu = bx + (fv.saturating_sub(acy)) / 2;
            let ty = dc + (ov.saturating_sub(12)) / 2;
            this.draw_text(bu as i32, ty as i32, label, crl);
        }

        
        let amh = haa;
        afc(self, amh, aqc, azt, hn, "|<", 0x142820, 0x00AA88, 0xFF88CCAA);

        
        let dct = amh + azt + gap;
        let nvk = match state.state {
            PlaybackState::Playing => "PAUSE",
            _ => "PLAY",
        };
        let nvg = match state.state {
            PlaybackState::Playing => 0x0A5530,
            _ => 0x084428,
        };
        afc(self, dct, aqc, cog, hn, nvk, nvg, 0x00FF88, 0xFF00FFAA);

        
        let dey = dct + cog + gap;
        afc(self, dey, aqc, azt, hn, "STOP", 0x2A1610, 0xCC6633, 0xFFFF8844);

        
        let evc = dey + azt + gap;
        afc(self, evc, aqc, azt, hn, ">|", 0x142820, 0x00AA88, 0xFF88CCAA);

        
        let apm = aqc + hn + 8;
        let edw = 10u32;
        self.draw_text(lp as i32, apm as i32, "VOL", 0xFF44886A);

        let ecm = lp + 30;
        let bwm = rn.saturating_sub(72);
        framebuffer::co(ecm, apm + 3, bwm, 4, 0x1A3322, 200);
        let hbw = (state.volume as u32 * bwm) / 100;
        if hbw > 0 {
            framebuffer::fill_rect(ecm, apm + 3, hbw, 4, 0xFF00CC88);
        }
        
        let cbf = ecm + hbw;
        if cbf + 4 <= ecm + bwm + 4 {
            framebuffer::fill_rect(cbf, apm, 4, edw, 0xFF00FFAA);
        }
        let edy = alloc::format!("{}%", state.volume);
        let pta = ecm + bwm + 6;
        self.draw_text(pta as i32, apm as i32, &edy, 0xFF88CCAA);

        
        
        
        let ent = apm + edw + 10;
        
        framebuffer::co(lp, ent, rn, 1, 0x00FF66, 20);
        let ens = ent + 4;
        self.draw_text(lp as i32, ens as i32, "EFFECTS", 0xFF336655);

        let aax = 24u32;
        let sb = 24u32;
        let aok = 36u32;

        
        let bjm = ens + 16;
        self.draw_text(lp as i32, bjm as i32 + 4, "SYNC", 0xFF44886A);
        let jkl = lp + aok + 4;
        
        afc(self, jkl, bjm, sb, aax, "-", 0x142820, 0x00AA88, 0xFF88CCAA);
        
        let jkm = alloc::format!("{}ms", state.av_offset_ms);
        let gwx = jkl + sb + 4;
        let gww = 52u32;
        framebuffer::co(gwx, bjm, gww, aax, 0x0A1A12, 180);
        let oyw = gwx + (gww.saturating_sub(jkm.len() as u32 * aq)) / 2;
        self.draw_text(oyw as i32, bjm as i32 + 5, &jkm, 0xFF88CCAA);
        
        let dfd = gwx + gww + 4;
        afc(self, dfd, bjm, sb, aax, "+", 0x142820, 0x00AA88, 0xFF88CCAA);
        
        let fby = dfd + sb + 4;
        afc(self, fby, bjm, sb, aax, "0", 0x1A1A14, 0x888855, 0xFFCCAA66);

        
        let bpz = bjm + aax + 4;
        self.draw_text(lp as i32, bpz as i32 + 4, "VIZ", 0xFF44886A);
        let jqg = lp + aok + 4;
        
        afc(self, jqg, bpz, sb, aax, "<", 0x142820, 0x00AA88, 0xFF88CCAA);
        
        let edu = self.visualizer.mode as usize % crate::visualizer::JJ_ as usize;
        let hbr = crate::visualizer::PE_[edu];
        let hbs = jqg + sb + 4;
        let dgj = rn.saturating_sub(aok + 4 + sb * 2 + 12);
        framebuffer::co(hbs, bpz, dgj, aax, 0x0A1A12, 180);
        let imn = (dgj / aq) as usize;
        let jqh = if hbr.len() > imn { &hbr[..imn] } else { hbr };
        let psz = hbs + (dgj.saturating_sub(jqh.len() as u32 * aq)) / 2;
        self.draw_text(psz as i32, bpz as i32 + 5, jqh, 0xFF00DDAA);
        
        let fen = hbs + dgj + 4;
        afc(self, fen, bpz, sb, aax, ">", 0x142820, 0x00AA88, 0xFF88CCAA);

        
        let bod = bpz + aax + 4;
        self.draw_text(lp as i32, bod as i32 + 4, "PAL", 0xFF44886A);
        let ith = lp + aok + 4;
        
        afc(self, ith, bod, sb, aax, "<", 0x142820, 0x8866CC, 0xFFAA88EE);
        
        let npp = self.visualizer.palette as usize % crate::visualizer::AHY_ as usize;
        let glw = crate::visualizer::CLV_[npp];
        let glx = ith + sb + 4;
        let dcf = rn.saturating_sub(aok + 4 + sb * 2 + 12);
        framebuffer::co(glx, bod, dcf, aax, 0x0A1A12, 180);
        let imq = (dcf / aq) as usize;
        let iti = if glw.len() > imq { &glw[..imq] } else { glw };
        let nvy = glx + (dcf.saturating_sub(iti.len() as u32 * aq)) / 2;
        self.draw_text(nvy as i32, bod as i32 + 5, iti, 0xFFCC88FF);
        
        let ewb = glx + dcf + 4;
        afc(self, ewb, bod, sb, aax, ">", 0x142820, 0x8866CC, 0xFFAA88EE);

        
        let bok = bod + aax + 4;
        self.draw_text(lp as i32, bok as i32 + 4, "RAIN", 0xFF44886A);
        let ixr = lp + aok + 4;
        
        afc(self, ixr, bok, sb, aax, "<", 0x142820, 0x00AA88, 0xFF88CCAA);
        
        let oaz = (self.matrix_rain_preset as usize).min(2);
        let oay = ["Slow", "Mid", "Fast"];
        let ixs = oay[oaz];
        let gpn = ixr + sb + 4;
        let dxg = rn.saturating_sub(aok + 4 + sb * 2 + 12);
        framebuffer::co(gpn, bok, dxg, aax, 0x0A1A12, 180);
        let ohr = gpn + (dxg.saturating_sub(ixs.len() as u32 * aq)) / 2;
        self.draw_text(ohr as i32, bok as i32 + 5, ixs, 0xFF88DDAA);
        
        let exp = gpn + dxg + 4;
        afc(self, exp, bok, sb, aax, ">", 0x142820, 0x00AA88, 0xFF88CCAA);
    }

    
    fn draw_calculator(&self, window: &Window) {
        let cx = window.x as u32 + 4;
        let u = window.y as u32 + J_() + 4;
        let aq = window.width.saturating_sub(8);
        let ch = window.height.saturating_sub(J_() + 8);
        
        if aq < 100 || ch < 120 {
            return;
        }
        
        
        
        
        
        
        
        let atm = 72u32;
        
        framebuffer::fill_rect(cx + 6, u + 6, aq - 12, atm, 0xFF0D0D1A);
        framebuffer::co(cx + 6, u + 6, aq - 12, atm / 2, 0x1A1A3E, 60);
        
        iu((cx + 6) as i32, (u + 6) as i32, aq - 12, atm, 6, Q_);
        
        framebuffer::co(cx + 7, u + 7, aq - 14, 1, 0x4444AA, 40);
        
        
        let bga = if let Some(sq) = self.calculator_states.get(&window.id) {
            &sq.display
        } else {
            "0"
        };
        
        
        let pii = bga.len() as i32;
        let ew = 12; 
        let kd = cx as i32 + aq as i32 - 18 - pii * ew;
        for (i, ch) in bga.chars().enumerate() {
            let p = kd + i as i32 * ew;
            let o = u as i32 + 28;
            let mut buf = [0u8; 4];
            let j = ch.encode_utf8(&mut buf);
            
            self.draw_text(p, o, j, 0xFFFFFFFF);
            self.draw_text(p + 1, o, j, 0xFFFFFFFF);
            self.draw_text(p, o + 1, j, 0xFFEEEEEE);
        }
        
        
        if let Some(sq) = self.calculator_states.get(&window.id) {
            if sq.just_evaluated && !sq.expression.is_empty() {
                self.draw_text(cx as i32 + 14, u as i32 + 14, "=", I_);
            }
        }
        
        
        let ehf = u + atm + 16;
        let djt = 5u32;
        let djs = 4u32;
        let rj = 6u32;
        let jyt = aq.saturating_sub(16);
        let jyr = ch.saturating_sub(atm + 28);
        let gu = (jyt - rj * (djs - 1)) / djs;
        let hn = ((jyr - rj * (djt - 1)) / djt).min(52);
        
        let buttons = [
            ["C", "(", ")", "%"],
            ["7", "8", "9", "/"],
            ["4", "5", "6", "*"],
            ["1", "2", "3", "-"],
            ["0", ".", "=", "+"],
        ];
        
        for (row, btn_row) in buttons.iter().enumerate() {
            for (col, label) in btn_row.iter().enumerate() {
                let bx = cx + 6 + col as u32 * (gu + rj);
                let dc = ehf + row as u32 * (hn + rj);
                
                let gdz = matches!(*label, "+" | "-" | "*" | "/" | "%" | "=");
                let gdr = *label == "C" || *label == "(" || *label == ")";
                
                
                let (bxv, btn_border) = if gdz {
                    if *label == "=" {
                        (Y_, I_)
                    } else {
                        (0xFF1A2A22, Q_)
                    }
                } else if gdr {
                    (0xFF2A1A28, 0xFF442244)
                } else {
                    (0xFF181C20, 0xFF2A2E34)
                };
                
                
                let ifc = self.cursor_x >= bx as i32 && self.cursor_x < (bx + gu) as i32
                    && self.cursor_y >= dc as i32 && self.cursor_y < (dc + hn) as i32;
                
                let bg = if ifc {
                    
                    let r = ((bxv >> 16) & 0xFF).min(220) + 30;
                    let g = ((bxv >> 8) & 0xFF).min(220) + 30;
                    let b = (bxv & 0xFF).min(220) + 30;
                    0xFF000000 | (r << 16) | (g << 8) | b
                } else {
                    bxv
                };
                
                
                if hn > 8 {
                    framebuffer::co(bx + 2, dc + 2, gu, hn, 0x000000, 30);
                }
                
                
                let hiw = 8u32.min(hn / 3);
                draw_rounded_rect(bx as i32, dc as i32, gu, hn, hiw, bg);
                iu(bx as i32, dc as i32, gu, hn, hiw, 
                    if ifc { I_ } else { btn_border });
                
                
                if hn > 12 {
                    framebuffer::co(bx + 3, dc + 1, gu.saturating_sub(6), 1, 0xFFFFFF, 15);
                }
                
                
                let mo = label.len() as u32 * 8;
                let fe = bx + (gu.saturating_sub(mo)) / 2;
                let ly = dc + (hn / 2).saturating_sub(5);
                let text_color = if *label == "=" { 
                    0xFF000000 
                } else if gdz { 
                    I_ 
                } else if gdr {
                    GN_
                } else { 
                    AB_ 
                };
                self.draw_text(fe as i32, ly as i32, label, text_color);
                
                if gdz || gdr {
                    self.draw_text(fe as i32 + 1, ly as i32, label, text_color);
                }
            }
        }
    }
    
    
    
    
    const SU_: u32 = 28;
    const KD_: u32 = 38;
    const ANU_: u32 = 20;
    const ABB_: u32 = 2; 

    
    
    fn browser_layout(&self, window: &Window)
        -> (u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32)
    {
        let bx = window.x as u32 + Self::ABB_;
        let dc = window.y as u32 + J_();
        let fv = window.width.saturating_sub(Self::ABB_ * 2);
        let ov = window.height.saturating_sub(J_() + Self::ABB_);

        let amy = dc;                                 
        let bcw = amy + Self::SU_;    
        let bnr: u32 = 28;                      
        
        let iph = bnr * 3 + 6 * 3;
        let ahf = bx + 8 + iph + 4;
        let uk = bcw + 4;
        let avo = Self::KD_ - 8;
        let asr = fv.saturating_sub(iph + 20 + 40); 

        let bn = bcw + Self::KD_;
        let en = ov.saturating_sub(Self::SU_ + Self::KD_ + Self::ANU_);
        let status_y = bn + en;

        (bx, dc, fv, ov, amy, bcw, ahf, uk, asr, avo, bn, en, status_y, bnr)
    }

    
    
    

    fn draw_wifi_networks(&self, window: &Window) {
        let wx = window.x;
        let wy = window.y;
        let ca = window.width;
        let er = window.height;
        if ca < 200 || er < 200 { return; }

        let bn = wy + J_() as i32;
        let lv = wx.max(0) as u32;

        let fix      = 0xFF0A120Cu32;
        let fiw     = 0xFF0C180Eu32;
        let fiy   = 0xFF0A120Cu32;
        let fja    = 0xFF0C140Cu32;
        let fiz  = 0xFF0E1E10u32;
        let dje   = 0xFF0A3818u32;
        let fcs   = 0xFF80CC90u32;
        let rg      = 0xFF406850u32;
        let jmc  = 0xFF50CC70u32;
        let dhb  = I_;
        let gvd   = 0xFF00CC66u32;
        let dzn     = 0xFFCCAA00u32;
        let dzo   = 0xFFCC4444u32;

        
        framebuffer::fill_rect(lv, bn as u32, ca, er - J_(), fix);

        
        let acc = 36u32;
        framebuffer::fill_rect(lv, bn as u32, ca, acc, fiw);
        framebuffer::mn(lv, (bn + acc as i32) as u32, ca, Q_);

        
        let wifi_state = crate::drivers::net::wifi::state();
        let acr = match wifi_state {
            crate::drivers::net::wifi::WifiState::NoHardware => "No WiFi Hardware",
            crate::drivers::net::wifi::WifiState::Disabled => "WiFi Disabled",
            crate::drivers::net::wifi::WifiState::Disconnected => "WiFi Disconnected",
            crate::drivers::net::wifi::WifiState::Scanning => "Scanning...",
            crate::drivers::net::wifi::WifiState::Connecting => "Connecting...",
            crate::drivers::net::wifi::WifiState::Authenticating => "Authenticating...",
            crate::drivers::net::wifi::WifiState::Connected => "Connected",
            crate::drivers::net::wifi::WifiState::Failed => "Connection Failed",
        };

        self.draw_text_smooth((wx + 12) as i32, (bn + 10) as i32, "WiFi Networks", jmc);
        self.draw_text_smooth((wx + 13) as i32, (bn + 10) as i32, "WiFi Networks", jmc);

        
        let oxb = (wx as u32 + ca).saturating_sub(acr.len() as u32 * 8 + 16);
        let bdw = match wifi_state {
            crate::drivers::net::wifi::WifiState::Connected => gvd,
            crate::drivers::net::wifi::WifiState::Scanning => dzn,
            crate::drivers::net::wifi::WifiState::Failed => dzo,
            _ => rg,
        };
        self.draw_text_smooth(oxb as i32, (bn + 10) as i32, acr, bdw);

        
        let mut gc = bn + acc as i32 + 2;
        if let Some(ssid) = crate::drivers::net::wifi::connected_ssid() {
            let dig = 40u32;
            framebuffer::co(lv + 4, gc as u32, ca - 8, dig, 0x003310, 180);
            iu((wx + 4) as i32, gc, ca - 8, dig, 6, dhb);

            self.draw_text_smooth((wx + 14) as i32, gc + 6, ">>", gvd);
            self.draw_text_smooth((wx + 34) as i32, gc + 6, &format!("Connected: {}", ssid), dhb);

            if let Some(sig) = crate::drivers::net::wifi::signal_strength() {
                let osr = format!("{} dBm", sig);
                self.draw_text_smooth((wx + 34) as i32, gc + 22, &osr, rg);
            }

            
            let cwo = (wx as u32 + ca).saturating_sub(100);
            draw_rounded_rect(cwo as i32, gc + 8, 80, 24, 6, 0xFF331111);
            iu(cwo as i32, gc + 8, 80, 24, 6, dzo);
            self.draw_text_smooth((cwo + 8) as i32, gc + 14, "Disconnect", dzo);

            gc += dig as i32 + 4;
        }

        
        let cpz = 80u32;
        let ddu = (wx as u32 + ca).saturating_sub(cpz + 8);
        let gst = gc;
        let dac = wifi_state == crate::drivers::net::wifi::WifiState::Scanning;
        let okw = if dac { 0xFF0A2A10u32 } else { 0xFF0C1C0Cu32 };
        draw_rounded_rect(ddu as i32, gst, cpz, 26, 6, okw);
        iu(ddu as i32, gst, cpz, 26, 6, if dac { dzn } else { dhb });
        let jdh = if dac { "Scanning" } else { "Scan" };
        let olf = ddu + (cpz - jdh.len() as u32 * 8) / 2;
        self.draw_text_smooth(olf as i32, gst + 7, jdh, if dac { dzn } else { dhb });

        self.draw_text_smooth((wx + 12) as i32, gc + 6, "Available Networks", rg);
        gc += 30;

        framebuffer::mn(lv + 4, gc as u32, ca - 8, Q_);
        gc += 2;

        
        let dva = crate::drivers::net::wifi::cys();
        let ep = 44i32;
        let yh = ((er as i32 - (gc - wy) - 8) / ep).max(1) as usize;

        if dva.is_empty() && !dac {
            self.draw_text_smooth((wx + 20) as i32, gc + 20, "No networks found. Click Scan to search.", rg);
        } else if dva.is_empty() && dac {
            let dnn = match (self.frame_count / 15) % 4 {
                0 => ".",
                1 => "..",
                2 => "...",
                _ => "",
            };
            self.draw_text_smooth((wx + 20) as i32, gc + 20, &format!("Scanning for networks{}", dnn), dzn);
        }

        for (i, net) in dva.iter().enumerate().skip(self.wifi_scroll_offset).take(yh) {
            let mf = gc + ((i - self.wifi_scroll_offset) as i32 * ep);
            let hd = i == self.wifi_selected_index;
            let ern = self.cursor_x >= wx && self.cursor_x < wx + ca as i32
                && self.cursor_y >= mf && self.cursor_y < mf + ep;

            let dyb = if hd { dje }
                else if ern { fiz }
                else if i % 2 == 0 { fiy }
                else { fja };
            framebuffer::fill_rect(lv + 4, mf as u32, ca - 8, ep as u32, dyb);

            
            let bars = net.signal_bars();
            let pv = wx + 12;
            for b in 0..4u32 {
                let hgr = 4 + b * 4;
                let jzp = mf + 30;
                let bxq = if b < bars as u32 {
                    if bars >= 3 { gvd } else if bars >= 2 { dzn } else { dzo }
                } else {
                    0xFF1A2A1Au32
                };
                framebuffer::fill_rect(
                    (pv + b as i32 * 6) as u32,
                    (jzp - hgr as i32) as u32,
                    4,
                    hgr,
                    bxq,
                );
            }

            
            let jhx = wx + 42;
            let ovn = if hd { dhb } else { fcs };
            let gvz = if net.ssid.is_empty() { "(Hidden Network)" } else { &net.ssid };
            self.draw_text_smooth(jhx, mf + 8, gvz, ovn);

            
            let gck = format!("{} | Ch {} | {} MHz | {} dBm",
                net.security.as_str(), net.channel, net.frequency_mhz, net.signal_dbm);
            self.draw_text_smooth(jhx, mf + 24, &gck, rg);

            
            if net.security != crate::drivers::net::wifi::WifiSecurity::Open {
                let auf = (wx as u32 + ca).saturating_sub(30);
                framebuffer::draw_rect(auf + 2, (mf + 8) as u32, 8, 6, rg);
                framebuffer::fill_rect(auf, (mf + 14) as u32, 12, 10, rg);
            }

            framebuffer::mn(lv + 8, (mf + ep - 1) as u32, ca - 16, 0xFF142014);
        }

        
        if let Some(ref bk) = self.wifi_error_msg {
            let elo = (wy as u32 + er).saturating_sub(30);
            framebuffer::co(lv + 4, elo, ca - 8, 24, 0x331111, 200);
            self.draw_text_smooth((wx + 12) as i32, (elo + 6) as i32, bk, dzo);
        }
    }

    
    
    

    fn draw_wifi_password(&self, window: &Window) {
        let wx = window.x;
        let wy = window.y;
        let ca = window.width;
        let er = window.height;
        if ca < 200 || er < 150 { return; }

        let bn = wy + J_() as i32;
        let lv = wx.max(0) as u32;

        let fix     = 0xFF0A120Cu32;
        let fcs = 0xFF80CC90u32;
        let rg    = 0xFF406850u32;
        let accent      = I_;
        let mqm    = 0xFF060C06u32;
        let kgi   = 0xFF0A3818u32;
        let kgh  = 0xFF1A0A0Au32;

        
        framebuffer::fill_rect(lv, bn as u32, ca, er - J_(), fix);

        
        let cao = wx as u32 + ca / 2;
        let caq = bn as u32 + 30;
        for arc in 0..4u32 {
            let r = 6 + arc * 6;
            for dhr in 0..16u32 {
                let dx = (dhr * r) / 16;
                let dof = (r * r).saturating_sub(dx * dx);
                let mut ad = 0u32;
                while (ad + 1) * (ad + 1) <= dof { ad += 1; }
                framebuffer::cz(cao + dx, caq.saturating_sub(ad), accent);
                if cao >= dx {
                    framebuffer::cz(cao - dx, caq.saturating_sub(ad), accent);
                }
            }
        }
        framebuffer::fill_rect(cao - 2, caq, 5, 5, accent);

        
        let jhw = format!("Connect to: {}", self.wifi_connecting_ssid);
        let bhn = (wx as u32 + (ca - jhw.len() as u32 * 8) / 2) as i32;
        self.draw_text_smooth(bhn, bn + 60, &jhw, fcs);

        
        self.draw_text_smooth(wx + 20, bn + 90, "Password:", rg);

        
        let sv = (bn + 108) as u32;
        let caw = ca - 32;
        let drz = 32u32;
        draw_rounded_rect((wx + 16) as i32, sv as i32, caw, drz, 6, mqm);
        iu((wx + 16) as i32, sv as i32, caw, drz, 6, Q_);

        
        let bga = if self.wifi_show_password {
            self.wifi_password_input.clone()
        } else {
            "*".repeat(self.wifi_password_input.len())
        };
        let kd = wx + 24;
        let ie = sv as i32 + 9;
        if bga.is_empty() {
            self.draw_text_smooth(kd, ie, "Enter password...", Q_);
        } else {
            let nd = ((caw as usize).saturating_sub(40)) / 8;
            let visible: String = bga.chars().rev().take(nd).collect::<String>().chars().rev().collect();
            self.draw_text_smooth(kd, ie, &visible, fcs);
        }

        
        if self.cursor_blink {
            let nd = ((caw as usize).saturating_sub(40)) / 8;
            let cursor_x = kd + bga.len().min(nd) as i32 * 8;
            framebuffer::fill_rect(cursor_x as u32, (ie - 1) as u32, 2, 14, accent);
        }

        
        let cen = sv as i32 + drz as i32 + 8;
        let kiv = if self.wifi_show_password { accent } else { Q_ };
        framebuffer::draw_rect((wx + 20) as u32, cen as u32, 14, 14, kiv);
        if self.wifi_show_password {
            framebuffer::fill_rect((wx + 23) as u32, (cen + 7) as u32, 3, 3, accent);
            framebuffer::fill_rect((wx + 26) as u32, (cen + 5) as u32, 3, 3, accent);
            framebuffer::fill_rect((wx + 29) as u32, (cen + 3) as u32, 3, 3, accent);
        }
        self.draw_text_smooth(wx + 40, cen + 2, "Show password", rg);

        
        let baq = (wy as u32 + er).saturating_sub(50);
        let anp = 100u32;
        let bkz = 32u32;
        let dka = 16u32;
        let gzn = anp * 2 + dka;
        let bxw = (wx as u32 + (ca - gzn) / 2) as i32;

        
        draw_rounded_rect(bxw, baq as i32, anp, bkz, 8, kgi);
        iu(bxw, baq as i32, anp, bkz, 8, accent);
        let kxc = bxw + (anp as i32 - 56) / 2;
        self.draw_text_smooth(kxc, baq as i32 + 9, "Connect", accent);

        
        let cus = bxw + anp as i32 + dka as i32;
        draw_rounded_rect(cus, baq as i32, anp, bkz, 8, kgh);
        iu(cus, baq as i32, anp, bkz, 8, 0xFFCC4444);
        let khf = cus + (anp as i32 - 48) / 2;
        self.draw_text_smooth(khf, baq as i32 + 9, "Cancel", 0xFFCC4444);

        
        if let Some(ref bk) = self.wifi_error_msg {
            let elo = baq - 24;
            let lrm = (wx as u32 + (ca - bk.len() as u32 * 8) / 2) as i32;
            self.draw_text_smooth(lrm, elo as i32, bk, 0xFFCC4444);
        }
    }

    
    fn draw_browser(&self, window: &Window) {
        let (bx, _by, fv, ov, amy, bcw,
             ahf, uk, asr, avo,
             bn, en, status_y, bnr)
            = self.browser_layout(window);

        if fv < 120 || ov < 100 { return; }

        let aq = crate::graphics::scaling::agg() as i32;
        let ch = crate::graphics::scaling::cgu();

        
        framebuffer::fill_rect(bx, amy, fv, Self::SU_, 0xFF202124);
        
        let dfh = bx + 8;
        let zm: u32 = 200.min(fv.saturating_sub(60));
        let bph = Self::SU_ - 4;
        
        framebuffer::fill_rect(dfh + 2, amy + 4, zm - 4, bph, 0xFF35363A);
        framebuffer::fill_rect(dfh, amy + 6, 2, bph - 2, 0xFF35363A);
        framebuffer::fill_rect(dfh + zm - 2, amy + 6, 2, bph - 2, 0xFF35363A);
        
        let gxx = if let Some(ref browser) = self.browser {
            if let Some(ref doc) = browser.document {
                if doc.title.is_empty() { alloc::string::String::from("New Tab") } else { doc.title.clone() }
            } else { alloc::string::String::from("New Tab") }
        } else { alloc::string::String::from("New Tab") };
        let lfn: alloc::string::String = if gxx.len() > 22 {
            let j: alloc::string::String = gxx.chars().take(20).collect();
            alloc::format!("{}...", j)
        } else { gxx };
        self.draw_text(dfh as i32 + 10, (amy + 8) as i32, &lfn, 0xFFE8EAED);
        
        self.draw_text((dfh + zm - 18) as i32, (amy + 8) as i32, "x", 0xFF999999);
        
        let ivh = dfh + zm + 6;
        framebuffer::fill_rect(ivh, amy + 6, 24, bph, 0xFF2A2A2E);
        self.draw_text(ivh as i32 + 8, (amy + 8) as i32, "+", 0xFF999999);

        
        framebuffer::fill_rect(bx, bcw, fv, Self::KD_, 0xFF35363A);
        
        framebuffer::fill_rect(bx, bcw, fv, 1, 0xFF4A4A4E);

        
        let bqv = bcw + Self::KD_ / 2; 
        let atd = bnr / 2;
        let mut dkb = bx + 12u32;
        
        let fsy = |cx: u32, u: u32, r: u32, hover_col: u32| {
            
            let clf = r / 3;
            framebuffer::fill_rect(cx - r + clf, u - r, (r - clf) * 2, r * 2, hover_col);
            framebuffer::fill_rect(cx - r, u - r + clf, r * 2, (r - clf) * 2, hover_col);
        };
        
        let hgj = dkb + atd;
        fsy(hgj, bqv, atd, 0xFF4A4A4E);
        self.draw_text((hgj - 4) as i32, (bqv - 6) as i32, "<", 0xFFE8EAED);
        dkb += bnr + 6;
        
        let iao = dkb + atd;
        fsy(iao, bqv, atd, 0xFF4A4A4E);
        self.draw_text((iao - 4) as i32, (bqv - 6) as i32, ">", 0xFFE8EAED);
        dkb += bnr + 6;
        
        let gqu = dkb + atd;
        fsy(gqu, bqv, atd, 0xFF4A4A4E);
        if self.browser_loading {
            self.draw_text((gqu - 4) as i32, (bqv - 6) as i32, "X", 0xFFE8EAED);
        } else {
            self.draw_text((gqu - 4) as i32, (bqv - 6) as i32, "R", 0xFFE8EAED);
        }

        
        
        let ahe = avo / 2; 
        framebuffer::fill_rect(ahf + ahe, uk, asr.saturating_sub(ahe * 2), avo, 0xFF202124);
        
        framebuffer::fill_rect(ahf, uk + ahe / 2, ahe, avo - ahe, 0xFF202124);
        framebuffer::fill_rect(ahf + 1, uk + ahe / 4, ahe - 1, ahe / 2, 0xFF202124);
        
        framebuffer::fill_rect(ahf + asr - ahe, uk + ahe / 2, ahe, avo - ahe, 0xFF202124);
        framebuffer::fill_rect(ahf + asr - ahe, uk + ahe / 4, ahe - 1, ahe / 2, 0xFF202124);
        
        if window.focused {
            framebuffer::fill_rect(ahf + ahe, uk, asr.saturating_sub(ahe * 2), 1, 0xFF8AB4F8);
            framebuffer::fill_rect(ahf + ahe, uk + avo - 1, asr.saturating_sub(ahe * 2), 1, 0xFF8AB4F8);
        }

        
        let adt = ahf as i32 + 8;
        let ie = uk as i32 + (avo as i32 - ch as i32) / 2;
        let mjo = self.browser_url_input.starts_with("https://");
        if mjo {
            self.draw_text(adt, ie, "S", 0xFF81C995);
        } else {
            
            self.draw_text(adt + 1, ie, "i", 0xFF999999);
        }
        
        framebuffer::fill_rect((ahf + 22) as u32, uk + 5, 1, avo - 10, 0xFF3C3C3C);

        
        let dgc = ahf as i32 + 28;
        let jpm = if self.browser_url_input.is_empty() {
            "Search or enter URL"
        } else {
            &self.browser_url_input
        };
        let text_color = if self.browser_url_input.is_empty() { 0xFF9AA0A6 } else { 0xFFE8EAED };

        
        let pif = (asr as i32).saturating_sub(42);
        let aac = if aq > 0 { (pif / aq).max(1) as usize } else { 40 };
        let jpl = jpm.len();
        let ezr = if self.browser_url_cursor > aac {
            self.browser_url_cursor - aac + 1
        } else { 0 };
        let psh = (ezr + aac).min(jpl);
        let edt = if ezr < jpl { &jpm[ezr..psh] } else { "" };

        if self.browser_loading {
            self.draw_text(dgc, ie, "Loading...", 0xFF8AB4F8);
        } else {
            self.draw_text(dgc, ie, edt, text_color);
        }

        
        if !self.browser_loading && self.browser_url_select_all && !self.browser_url_input.is_empty() {
            let jei = (edt.len() as u32) * aq as u32;
            if jei > 0 {
                framebuffer::fill_rect(dgc as u32, uk + 3, jei.min(asr - 34), avo - 6, 0xFF3574E0);
                
                self.draw_text(dgc, ie, edt, 0xFFFFFFFF);
            }
        }

        
        if !self.browser_loading && window.focused {
            if self.cursor_blink {
                let kuz = self.browser_url_cursor.saturating_sub(ezr);
                let cx = dgc + (kuz as i32) * aq;
                if cx >= dgc && cx < (ahf + asr - 8) as i32 {
                    framebuffer::fill_rect(cx as u32, uk + 4, 2, avo - 8, 0xFF8AB4F8);
                }
            }
        }

        
        let hu = ahf + asr + 6;
        let ks = bcw + 8;
        let cwu: u32 = 3;
        let mtm = self.browser.as_ref().map(|b| b.show_raw_html).unwrap_or(false);
        let ghn = if mtm { 0xFF8AB4F8 } else { 0xFF999999 };
        framebuffer::fill_rect(hu + 4, ks + 2, cwu, cwu, ghn);
        framebuffer::fill_rect(hu + 4, ks + 8, cwu, cwu, ghn);
        framebuffer::fill_rect(hu + 4, ks + 14, cwu, cwu, ghn);

        
        if let Some(ref browser) = self.browser {
            if browser.show_raw_html && !browser.raw_html.is_empty() {
                framebuffer::fill_rect(bx, bn, fv, en, 0xFF1E1E1E);
                self.draw_raw_html_view(bx as i32, bn as i32, fv, en, &browser.raw_html, browser.scroll_y);
            } else if let Some(ref doc) = browser.document {
                crate::browser::ofl(doc, bx as i32, bn as i32, fv, en, browser.scroll_y);
            } else {
                self.draw_browser_welcome(bx, bn, fv, en);
            }
        } else {
            self.draw_browser_welcome(bx, bn, fv, en);
        }

        
        framebuffer::fill_rect(bx, status_y, fv, Self::ANU_, 0xFF202124);
        let crc = if let Some(ref browser) = self.browser {
            match &browser.status {
                crate::browser::BrowserStatus::Idle => alloc::string::String::from("Ready"),
                crate::browser::BrowserStatus::Loading => alloc::string::String::from("Loading..."),
                crate::browser::BrowserStatus::Ready => {
                    if !browser.resources.is_empty() {
                        alloc::format!("Done  ({} resources)", browser.resources.len())
                    } else {
                        alloc::string::String::from("Done")
                    }
                },
                crate::browser::BrowserStatus::Error(e) => e.clone(),
            }
        } else { alloc::string::String::from("Ready") };
        self.draw_text(bx as i32 + 8, status_y as i32 + 3, &crc, 0xFF9AA0A6);
    }

    
    fn draw_browser_welcome(&self, bx: u32, u: u32, fv: u32, ch: u32) {
        
        framebuffer::fill_rect(bx, u, fv, ch, 0xFFFFFFFF);

        let arn = bx as i32 + fv as i32 / 2;
        let ags = u as i32 + ch as i32 / 2 - 50;

        
        let title = "TrustBrowser";
        let fcm = crate::graphics::scaling::agg() as i32;
        let bu = arn - (title.len() as i32 * fcm) / 2;
        self.draw_text(bu, ags, title, 0xFF202124);

        
        let bkx: u32 = 360.min(fv.saturating_sub(40));
        let apw: u32 = 34;
        let ala = (arn - bkx as i32 / 2).max(bx as i32 + 4) as u32;
        let agf = (ags + 30) as u32;
        framebuffer::fill_rect(ala + 4, agf, bkx - 8, apw, 0xFFF1F3F4);
        framebuffer::fill_rect(ala, agf + 4, 4, apw - 8, 0xFFF1F3F4);
        framebuffer::fill_rect(ala + bkx - 4, agf + 4, 4, apw - 8, 0xFFF1F3F4);
        
        framebuffer::fill_rect(ala + 4, agf, bkx - 8, 1, 0xFFDFE1E5);
        framebuffer::fill_rect(ala + 4, agf + apw - 1, bkx - 8, 1, 0xFFDFE1E5);
        
        self.draw_text(ala as i32 + 14, agf as i32 + 9, "Search or type a URL", 0xFF9AA0A6);

        
        let cbm = agf as i32 + apw as i32 + 24;
        let gfo = ["example.com", "10.0.2.2", "google.com"];
        let dap: i32 = 100;
        let aaj = gfo.len() as i32 * dap + (gfo.len() as i32 - 1) * 12;
        let mut fe = arn - aaj / 2;
        for label in &gfo {
            
            framebuffer::fill_rect(fe as u32, cbm as u32, dap as u32, 28, 0xFFF1F3F4);
            framebuffer::fill_rect(fe as u32, cbm as u32, dap as u32, 1, 0xFFDFE1E5);
            framebuffer::fill_rect(fe as u32, (cbm + 27) as u32, dap as u32, 1, 0xFFDFE1E5);
            let gr = label.len() as i32 * fcm;
            self.draw_text(fe + (dap - gr) / 2, cbm + 7, label, 0xFF1A73E8);
            fe += dap + 12;
        }
    }
    
    
    fn draw_raw_html_view(&self, x: i32, y: i32, width: u32, height: u32, ajx: &str, scroll_y: i32) {
        let aq = crate::graphics::scaling::agg() as i32;
        let line_height = crate::graphics::scaling::cgu() as i32 + 2;
        let nd = if aq > 0 { (width as usize).saturating_sub(56) / aq as usize } else { 60 };

        let mut dnt = y + 8 - scroll_y;
        let aye = y + height as i32 - 8;
        let mut axw = 1;

        for line in ajx.lines() {
            if dnt > aye { break; }
            if dnt >= y - line_height {
                let gfm = alloc::format!("{:4} ", axw);
                self.draw_text(x + 4, dnt, &gfm, 0xFF6E7681);
                let lfl: alloc::string::String = if line.len() > nd {
                    let t: alloc::string::String = line.chars().take(nd.saturating_sub(3)).collect();
                    alloc::format!("{}...", t)
                } else { alloc::string::String::from(line) };
                self.draw_syntax_highlighted(x + 5 * aq + 8, dnt, &lfl);
            }
            dnt += line_height;
            axw += 1;
        }
    }

    
    fn draw_syntax_highlighted(&self, x: i32, y: i32, line: &str) {
        let aq = crate::graphics::scaling::agg() as i32;
        let mut current_x = x;
        let mut eqk = false;
        let mut bcj = false;
        let mut gcc = false;
        let mut jjb = '"';

        let chars: alloc::vec::Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            let color = if bcj {
                0xFFCE9178
            } else if c == '<' || c == '>' || c == '/' {
                eqk = c == '<';
                if c == '>' { gcc = false; }
                0xFF569CD6
            } else if eqk && c == '=' {
                gcc = true;
                0xFF9CDCFE
            } else if eqk && (c == '"' || c == '\'') {
                bcj = true;
                jjb = c;
                0xFFCE9178
            } else if gcc && !c.is_whitespace() {
                0xFF4EC9B0
            } else if eqk && !c.is_whitespace() && c != '=' {
                0xFF569CD6
            } else {
                0xFFD4D4D4
            };
            if bcj && i > 0 && c == jjb && chars[i-1] != '\\' {
                bcj = false;
            }
            let j = alloc::format!("{}", c);
            self.draw_text(current_x, y, &j, color);
            current_x += aq;
            i += 1;
        }
    }
    
    
    fn handle_browser_click(&mut self, x: i32, y: i32, nw: i32, qr: i32, ul: u32) {
        crate::serial_println!("[BROWSER-DBG] handle_browser_click x={} y={} win_x={} win_y={} win_w={}",
            x, y, nw, qr, ul);
        
        
        let pkj = Window {
            id: 0, title: String::new(),
            x: nw, y: qr,
            width: ul,
            height: self.windows.iter()
                .find(|w| w.x == nw && w.y == qr && w.width == ul)
                .map(|w| w.height).unwrap_or(500),
            min_width: 0, min_height: 0,
            visible: true, focused: true, minimized: false, maximized: false,
            dragging: false, resizing: ResizeEdge::None,
            drag_offset_x: 0, drag_offset_y: 0,
            saved_x: 0, saved_y: 0, saved_width: 0, saved_height: 0,
            window_type: WindowType::Browser,
            content: Vec::new(), file_path: None,
            selected_index: 0, scroll_offset: 0,
            animation: WindowAnimation::new(), pending_close: false, dirty: true,
        };
        let (_bx, _by, fv, _bh, _tab_y, bcw,
             ahf, uk, asr, avo,
             _content_y, _content_h, _status_y, bnr)
            = self.browser_layout(&pkj);

        crate::serial_println!("[BROWSER-DBG] layout: url_bar=({},{} {}x{}) nav_y={} bw={} click=({},{})",
            ahf, uk, asr, avo, bcw, fv, x, y);

        if fv < 120 { return; }

        let cx = x as u32;
        let u = y as u32;

        
        let atd = bnr / 2;
        let bqv = bcw + Self::KD_ / 2;
        let mut zs = _bx + 12 + atd;
        
        let gav = |bx_c: u32| -> bool {
            let dx = (cx as i32 - bx_c as i32).unsigned_abs();
            let ad = (u as i32 - bqv as i32).unsigned_abs();
            dx <= atd && ad <= atd
        };
        
        if gav(zs) {
            crate::serial_println!("[BROWSER] Back button clicked");
            if let Some(ref mut browser) = self.browser { let _ = browser.back(); }
            return;
        }
        zs += bnr + 6;
        
        if gav(zs) {
            crate::serial_println!("[BROWSER] Forward button clicked");
            if let Some(ref mut browser) = self.browser { let _ = browser.forward(); }
            return;
        }
        zs += bnr + 6;
        
        if gav(zs) {
            crate::serial_println!("[BROWSER] Refresh button clicked");
            if let Some(ref mut browser) = self.browser { let _ = browser.refresh(); }
            return;
        }

        
        if cx >= ahf && cx < ahf + asr
            && u >= uk && u < uk + avo
        {
            
            if crate::mouse::erj() {
                crate::mouse::jah();
                self.browser_url_select_all = true;
                self.browser_url_cursor = self.browser_url_input.len();
                crate::serial_println!("[BROWSER] URL bar double-clicked, select all");
                return;
            }
            
            self.browser_url_select_all = false;
            let aq = crate::graphics::scaling::agg();
            if aq > 0 {
                let pim = ahf + 26;
                let sk = cx.saturating_sub(pim);
                let dkn = (sk / aq) as usize;
                self.browser_url_cursor = dkn.min(self.browser_url_input.len());
                crate::serial_println!("[BROWSER] URL bar clicked, cursor={}", self.browser_url_cursor);
            }
            return;
        }

        
        let hu = ahf + asr + 6;
        let ks = bcw + 8;
        if cx >= hu && cx < hu + 16 && u >= ks && u < ks + 22 {
            crate::serial_println!("[BROWSER] Menu (view toggle) clicked");
            if let Some(ref mut browser) = self.browser {
                browser.toggle_view_mode();
            }
            return;
        }
    }
    
    fn draw_cursor(&self) {
        
        let mut brj = CursorMode::Arrow;
        
        
        for w in self.windows.iter().rev() {
            if w.minimized || w.maximized { continue; }
            let th = w.on_resize_edge(self.cursor_x, self.cursor_y);
            match th {
                ResizeEdge::Left | ResizeEdge::Right => { brj = CursorMode::ResizeH; break; },
                ResizeEdge::Top | ResizeEdge::Bottom => { brj = CursorMode::ResizeV; break; },
                ResizeEdge::TopLeft | ResizeEdge::BottomRight => { brj = CursorMode::ResizeNWSE; break; },
                ResizeEdge::TopRight | ResizeEdge::BottomLeft => { brj = CursorMode::ResizeNESW; break; },
                _ => {},
            }
            
            if self.cursor_x >= w.x && self.cursor_x < w.x + w.width as i32
                && self.cursor_y >= w.y && self.cursor_y < w.y + w.height as i32 {
                break;
            }
        }
        
        
        for w in &self.windows {
            match w.resizing {
                ResizeEdge::Left | ResizeEdge::Right => { brj = CursorMode::ResizeH; break; },
                ResizeEdge::Top | ResizeEdge::Bottom => { brj = CursorMode::ResizeV; break; },
                ResizeEdge::TopLeft | ResizeEdge::BottomRight => { brj = CursorMode::ResizeNWSE; break; },
                ResizeEdge::TopRight | ResizeEdge::BottomLeft => { brj = CursorMode::ResizeNESW; break; },
                _ => {},
            }
        }
        
        match brj {
            CursorMode::Arrow | CursorMode::Grab => self.draw_arrow_cursor(),
            CursorMode::ResizeH => self.draw_resize_cursor_h(),
            CursorMode::ResizeV => self.draw_resize_cursor_v(),
            CursorMode::ResizeNWSE => self.draw_resize_cursor_nwse(),
            CursorMode::ResizeNESW => self.draw_resize_cursor_nesw(),
        }
    }
    
    
    fn draw_arrow_cursor(&self) {
        let cs = crate::accessibility::cyl().scale();
        let ads = crate::accessibility::btq();
        
        
        let bjd = 0x40000000u32;
        for offset in 1..=(2 * cs as i32) {
            let am = self.cursor_x + offset;
            let ak = self.cursor_y + offset;
            if am >= 0 && ak >= 0 && am < self.width as i32 && ak < self.height as i32 {
                for ad in 0..(12 * cs as i32) {
                    let o = (ak + ad) as u32;
                    let p = am as u32;
                    if o < self.height && p < self.width {
                        framebuffer::cz(p, o, bjd);
                    }
                }
            }
        }
        
        
        let bob = if ads { 0xFF000000u32 } else { Y_ };
        let bso = if ads { 0xFFFFFFFFu32 } else { AH_ };
        
        
        let cursor: [[u8; 12]; 16] = [
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
        
        for (u, row) in cursor.iter().enumerate() {
            for (cx, &ct) in row.iter().enumerate() {
                if ct == 0 { continue; }
                let color = match ct {
                    1 => bob,
                    2 => bso,
                    _ => continue,
                };
                
                for ak in 0..cs {
                    for am in 0..cs {
                        let p = (self.cursor_x + cx as i32 * cs as i32 + am as i32) as u32;
                        let o = (self.cursor_y + u as i32 * cs as i32 + ak as i32) as u32;
                        if p < self.width && o < self.height {
                            framebuffer::cz(p, o, color);
                        }
                    }
                }
            }
        }
    }
    
    
    fn draw_resize_cursor_h(&self) {
        let cg = self.cursor_x;
        let cr = self.cursor_y;
        
        
        for i in 0..7i32 {
            let p = (cg - 7 + i) as u32;
            let o = cr as u32;
            if p < self.width && o < self.height {
                framebuffer::cz(p, o, I_);
                if o > 0 { framebuffer::cz(p, o - 1, Y_); }
                if o + 1 < self.height { framebuffer::cz(p, o + 1, Y_); }
            }
        }
        
        for i in 0..7i32 {
            let p = (cg + 1 + i) as u32;
            let o = cr as u32;
            if p < self.width && o < self.height {
                framebuffer::cz(p, o, I_);
                if o > 0 { framebuffer::cz(p, o - 1, Y_); }
                if o + 1 < self.height { framebuffer::cz(p, o + 1, Y_); }
            }
        }
        
        for d in 1..=4i32 {
            let p = (cg - 7 + d) as u32;
            if p < self.width {
                if (cr - d) >= 0 { framebuffer::cz(p, (cr - d) as u32, I_); }
                if (cr + d) < self.height as i32 { framebuffer::cz(p, (cr + d) as u32, I_); }
            }
        }
        
        for d in 1..=4i32 {
            let p = (cg + 7 - d) as u32;
            if p < self.width {
                if (cr - d) >= 0 { framebuffer::cz(p, (cr - d) as u32, I_); }
                if (cr + d) < self.height as i32 { framebuffer::cz(p, (cr + d) as u32, I_); }
            }
        }
        
        if cg >= 0 && cr >= 0 && (cg as u32) < self.width && (cr as u32) < self.height {
            framebuffer::cz(cg as u32, cr as u32, 0xFFFFFFFF);
        }
    }
    
    
    fn draw_resize_cursor_v(&self) {
        let cg = self.cursor_x;
        let cr = self.cursor_y;
        
        for i in 0..7i32 {
            let p = cg as u32;
            let o = (cr - 7 + i) as u32;
            if p < self.width && o < self.height {
                framebuffer::cz(p, o, I_);
                if p > 0 { framebuffer::cz(p - 1, o, Y_); }
                if p + 1 < self.width { framebuffer::cz(p + 1, o, Y_); }
            }
        }
        for i in 0..7i32 {
            let p = cg as u32;
            let o = (cr + 1 + i) as u32;
            if p < self.width && o < self.height {
                framebuffer::cz(p, o, I_);
                if p > 0 { framebuffer::cz(p - 1, o, Y_); }
                if p + 1 < self.width { framebuffer::cz(p + 1, o, Y_); }
            }
        }
        
        for d in 1..=4i32 {
            let o = (cr - 7 + d) as u32;
            if o < self.height {
                if (cg - d) >= 0 { framebuffer::cz((cg - d) as u32, o, I_); }
                if (cg + d) < self.width as i32 { framebuffer::cz((cg + d) as u32, o, I_); }
            }
        }
        
        for d in 1..=4i32 {
            let o = (cr + 7 - d) as u32;
            if o < self.height {
                if (cg - d) >= 0 { framebuffer::cz((cg - d) as u32, o, I_); }
                if (cg + d) < self.width as i32 { framebuffer::cz((cg + d) as u32, o, I_); }
            }
        }
        if cg >= 0 && cr >= 0 && (cg as u32) < self.width && (cr as u32) < self.height {
            framebuffer::cz(cg as u32, cr as u32, 0xFFFFFFFF);
        }
    }
    
    
    fn draw_resize_cursor_nwse(&self) {
        let cg = self.cursor_x;
        let cr = self.cursor_y;
        
        for i in -6..=6i32 {
            let p = (cg + i) as u32;
            let o = (cr + i) as u32;
            if p < self.width && o < self.height {
                framebuffer::cz(p, o, I_);
                if p + 1 < self.width { framebuffer::cz(p + 1, o, Y_); }
                if o + 1 < self.height { framebuffer::cz(p, o + 1, Y_); }
            }
        }
        
        for d in 1..=3i32 {
            let bx = cg - 6 + d;
            let dc = cr - 6;
            if bx >= 0 && (dc as u32) < self.height { framebuffer::cz(bx as u32, dc as u32, I_); }
            let bfa = cg - 6;
            let bfb = cr - 6 + d;
            if bfa >= 0 && bfb >= 0 { framebuffer::cz(bfa as u32, bfb as u32, I_); }
        }
        
        for d in 1..=3i32 {
            let bx = cg + 6 - d;
            let dc = cr + 6;
            if (bx as u32) < self.width && (dc as u32) < self.height { framebuffer::cz(bx as u32, dc as u32, I_); }
            let bfa = cg + 6;
            let bfb = cr + 6 - d;
            if (bfa as u32) < self.width && (bfb as u32) < self.height { framebuffer::cz(bfa as u32, bfb as u32, I_); }
        }
    }
    
    
    fn draw_resize_cursor_nesw(&self) {
        let cg = self.cursor_x;
        let cr = self.cursor_y;
        
        for i in -6..=6i32 {
            let p = (cg + i) as u32;
            let o = (cr - i) as u32;
            if p < self.width && o < self.height {
                framebuffer::cz(p, o, I_);
                if p > 0 { framebuffer::cz(p - 1, o, Y_); }
                if o + 1 < self.height { framebuffer::cz(p, o + 1, Y_); }
            }
        }
        
        for d in 1..=3i32 {
            let bx = cg + 6 - d;
            let dc = cr - 6;
            if (bx as u32) < self.width && dc >= 0 { framebuffer::cz(bx as u32, dc as u32, I_); }
            let bfa = cg + 6;
            let bfb = cr - 6 + d;
            if (bfa as u32) < self.width && bfb >= 0 { framebuffer::cz(bfa as u32, bfb as u32, I_); }
        }
        
        for d in 1..=3i32 {
            let bx = cg - 6 + d;
            let dc = cr + 6;
            if bx >= 0 && (dc as u32) < self.height { framebuffer::cz(bx as u32, dc as u32, I_); }
            let bfa = cg - 6;
            let bfb = cr + 6 - d;
            if bfa >= 0 && (bfb as u32) < self.height { framebuffer::cz(bfa as u32, bfb as u32, I_); }
        }
    }
    
    fn draw_text(&self, x: i32, y: i32, text: &str, color: u32) {
        
        let gko = framebuffer::dqp();
        framebuffer::bdr(color);
        
        let aq = crate::graphics::scaling::agg() as i32;
        for (i, c) in text.chars().enumerate() {
            let p = x + (i as i32 * aq);
            if p >= 0 && p < self.width as i32 && y >= 0 && y < self.height as i32 {
                crate::graphics::scaling::fta(p as u32, y as u32, c, color);
            }
        }
        
        framebuffer::bdr(gko);
    }
    
    fn draw_char(&self, x: u32, y: u32, c: char, color: u32) {
        
        crate::graphics::scaling::fta(x, y, c, color);
    }
    
    
    fn draw_text_smooth(&self, x: i32, y: i32, text: &str, color: u32) {
        let aq = crate::graphics::scaling::agg() as i32;
        let ha = crate::graphics::scaling::aqv();
        let jsk = 16u32 * ha;
        let pwy = 8u32 * ha;
        let fb_w = self.width;
        let fb_h = self.height;
        
        let bsn = ((color >> 16) & 0xFF) as u32;
        let bsm = ((color >> 8) & 0xFF) as u32;
        let bsl = (color & 0xFF) as u32;
        
        for (i, c) in text.chars().enumerate() {
            let cx = x + (i as i32 * aq);
            if cx < 0 || cx >= fb_w as i32 || y < 0 || y >= fb_h as i32 { continue; }
            
            let du = framebuffer::font::ol(c);
            
            for row in 0..16u32 {
                let bits = du[row as usize];
                let prev = if row > 0 { du[row as usize - 1] } else { 0u8 };
                let next = if row < 15 { du[row as usize + 1] } else { 0u8 };
                
                for col in 0..8u32 {
                    let mask = 0x80u8 >> col;
                    let gec = bits & mask != 0;
                    
                    if gec {
                        
                        for ak in 0..ha {
                            for am in 0..ha {
                                let p = cx as u32 + col * ha + am;
                                let o = y as u32 + row * ha + ak;
                                if p < fb_w && o < fb_h {
                                    framebuffer::cz(p, o, color);
                                }
                            }
                        }
                    } else {
                        
                        let left  = col > 0 && (bits & (mask << 1)) != 0;
                        let right = col < 7 && (bits & (mask >> 1)) != 0;
                        let top   = prev & mask != 0;
                        let age   = next & mask != 0;
                        
                        let gyy = col > 0 && (prev & (mask << 1)) != 0;
                        let tr = col < 7 && (prev & (mask >> 1)) != 0;
                        let bl = col > 0 && (next & (mask << 1)) != 0;
                        let yi = col < 7 && (next & (mask >> 1)) != 0;
                        
                        
                        let khn = (left as u32) + (right as u32) + (top as u32) + (age as u32);
                        let fsb = (gyy as u32) + (tr as u32) + (bl as u32) + (yi as u32);
                        let score = khn * 2 + fsb; 
                        
                        if score > 0 {
                            
                            let alpha = if score >= 6 { 140u32 }
                                else if score >= 4 { 100u32 }
                                else if score >= 2 { 60u32 }
                                else { 35u32 };
                            let ki = 255 - alpha;
                            for ak in 0..ha {
                                for am in 0..ha {
                                    let p = cx as u32 + col * ha + am;
                                    let o = y as u32 + row * ha + ak;
                                    if p < fb_w && o < fb_h {
                                        let bg = framebuffer::fyu(p, o);
                                        let awg = (bg >> 16) & 0xFF;
                                        let awf = (bg >> 8) & 0xFF;
                                        let awe = bg & 0xFF;
                                        let r = (bsn * alpha + awg * ki) / 255;
                                        let g = (bsm * alpha + awf * ki) / 255;
                                        let b = (bsl * alpha + awe * ki) / 255;
                                        framebuffer::cz(p, o, 0xFF000000 | (r << 16) | (g << 8) | b);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_wifi_networks_click(&mut self, x: i32, y: i32, fr: u32) {
        let window = match self.windows.iter().find(|w| w.id == fr) {
            Some(w) => w,
            None => return,
        };
        let wx = window.x;
        let wy = window.y;
        let ca = window.width;
        let er = window.height;
        let bn = wy + J_() as i32;
        let acc = 36i32;

        
        let mut gc = bn + acc + 2;
        if crate::drivers::net::wifi::connected_ssid().is_some() {
            let dig = 40i32;
            
            let cwo = (wx as i32 + ca as i32) - 100;
            if x >= cwo && x < cwo + 80 && y >= gc + 8 && y < gc + 32 {
                crate::drivers::net::wifi::disconnect();
                return;
            }
            gc += dig + 4;
        }

        
        let cpz = 80i32;
        let ddu = (wx as i32 + ca as i32) - cpz - 8;
        if x >= ddu && x < ddu + cpz
            && y >= gc && y < gc + 26 {
            crate::drivers::net::wifi::eaj();
            return;
        }

        gc += 32; 

        
        let dva = crate::drivers::net::wifi::cys();
        let ep = 44i32;
        for (i, net) in dva.iter().enumerate().skip(self.wifi_scroll_offset) {
            let mf = gc + ((i - self.wifi_scroll_offset) as i32 * ep);
            if mf + ep > wy + er as i32 { break; }
            if y >= mf && y < mf + ep && x >= wx && x < wx + ca as i32 {
                self.wifi_selected_index = i;
                
                if net.security == crate::drivers::net::wifi::WifiSecurity::Open {
                    crate::drivers::net::wifi::eyl(&net.ssid, "");
                } else {
                    self.wifi_connecting_ssid = net.ssid.clone();
                    self.wifi_password_input.clear();
                    self.wifi_error_msg = None;
                    self.create_window("WiFi Password", 250, 150, 360, 300, WindowType::WifiPassword);
                }
                return;
            }
        }
    }

    fn handle_wifi_password_click(&mut self, x: i32, y: i32, fr: u32) {
        let window = match self.windows.iter().find(|w| w.id == fr) {
            Some(w) => w,
            None => return,
        };
        let wx = window.x;
        let wy = window.y;
        let ca = window.width;
        let er = window.height;
        let bn = wy + J_() as i32;

        let sv = bn + 108;
        let drz = 32;

        
        let cen = sv + drz + 8;
        if x >= wx + 20 && x < wx + 34 && y >= cen && y < cen + 14 {
            self.wifi_show_password = !self.wifi_show_password;
            return;
        }

        
        let baq = (wy as i32 + er as i32) - 50;
        let anp = 100i32;
        let bkz = 32i32;
        let dka = 16i32;
        let gzn = anp * 2 + dka;
        let bxw = wx + (ca as i32 - gzn) / 2;

        
        if x >= bxw && x < bxw + anp
            && y >= baq && y < baq + bkz {
            if self.wifi_password_input.is_empty() {
                self.wifi_error_msg = Some(String::from("Password cannot be empty"));
            } else {
                crate::drivers::net::wifi::eyl(
                    &self.wifi_connecting_ssid,
                    &self.wifi_password_input,
                );
                
                self.windows.retain(|w| w.id != fr);
            }
            return;
        }

        
        let cus = bxw + anp + dka;
        if x >= cus && x < cus + anp
            && y >= baq && y < baq + bkz {
            self.windows.retain(|w| w.id != fr);
            return;
        }
    }
}


fn qhh(c: char) -> [u8; 16] {
    
    match c {
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


pub static S: Mutex<Desktop> = Mutex::new(Desktop::new());


pub fn init(width: u32, height: u32) {
    S.lock().init(width, height);
    crate::serial_println!("[GUI] Desktop initialized: {}x{} (double-buffered)", width, height);
}


pub fn create_window(title: &str, x: i32, y: i32, width: u32, height: u32) -> u32 {
    S.lock().create_window(title, x, y, width, height, WindowType::Empty)
}


pub fn qbz(x: i32, y: i32) -> u32 {
    S.lock().create_window("Terminal", x, y, 640, 440, WindowType::Terminal)
}


pub fn qby(x: i32, y: i32) -> u32 {
    S.lock().create_window("System Info", x, y, 300, 220, WindowType::SystemInfo)
}


pub fn close_window(id: u32) {
    S.lock().close_window(id);
}


pub fn update_cursor(x: i32, y: i32) {
    S.lock().handle_move(x, y);
}


pub fn handle_click(x: i32, y: i32, pressed: bool) {
    S.lock().handle_click(x, y, pressed);
}


pub fn draw() {
    S.lock().draw();
}


pub fn mhy(key: u8) {
    S.lock().handle_keyboard_input(key);
}


pub fn handle_right_click(x: i32, y: i32, pressed: bool) {
    S.lock().handle_right_click(x, y, pressed);
}


pub fn handle_scroll(mk: i8) {
    S.lock().handle_scroll(mk);
}


pub fn run() {
    use crate::gui::engine::{self, HotkeyAction};
    NW_.store(false, Ordering::SeqCst);
    
    
    engine::igw();
    crate::gui::vsync::init();
    
    
    
    unsafe {
        
        static mut CSD_: bool = true;
        CSD_ = true;
    }
    
    {
        let mut d = S.lock();
        d.context_menu.visible = false;
    }
    
    crate::serial_println!("[GUI] Starting desktop environment...");
    crate::serial_println!("[GUI] Hotkeys: Alt+Tab, Win+Arrows, Alt+F4, Win=Start");
    crate::serial_println!("[GUI] Target: ~60 FPS (16.6ms) with spin-loop frame limiting");
    
    
    let mut ilg: u32 = 0;

    loop {
        
        if NW_.load(Ordering::SeqCst) {
            crate::serial_println!("[GUI] Desktop exit requested, returning to shell");
            break;
        }
        let cyd = engine::yy();
        
        
        
        
        
        
        let mouse = crate::mouse::get_state();
        update_cursor(mouse.x, mouse.y);
        
        
        
        {
            let fgx = crate::keyboard::sx(0x38);
            let hcj = crate::keyboard::sx(0x5B);
            let fph = crate::keyboard::sx(0x1D);
            let jle = crate::keyboard::sx(0x0F);
            static mut BFQ_: bool = false;
            unsafe {
                if (fgx || hcj || fph) && jle && !BFQ_ {
                    if !engine::dsk() {
                        engine::jig();
                    } else {
                        engine::hfb();
                    }
                }
                BFQ_ = jle;
            }
        }
        
        
        
        
        
        
        let mut ijc = 0u32;
        while let Some(key) = crate::keyboard::ya() {
            ijc += 1;
            if ijc > 32 { break; }
            crate::serial_println!("[INPUT-DBG] key={} (0x{:02X})", key, key);
            
            let adf = crate::keyboard::sx(0x38);
            let hdh = crate::keyboard::sx(0x1D);
            let aw = crate::keyboard::sx(0x5B);
            
            
            
            
            if key == 27 {
                crate::serial_println!("[GUI] ESC pressed");
                
                if let Some(mut d) = S.try_lock() {
                    
                    if d.mobile_state.active {
                        crate::serial_println!("[GUI] ESC: mobile mode, exiting to shell");
                        drop(d);
                        NW_.store(true, Ordering::SeqCst);
                        continue;
                    }
                    
                    if d.start_menu_open {
                        d.start_menu_open = false;
                        d.start_menu_search.clear();
                        d.start_menu_selected = -1;
                        crate::serial_println!("[GUI] ESC: closed start menu");
                        drop(d);
                        continue;
                    }
                    
                    let loi = {
                        let focused = d.windows.iter().find(|w| w.focused && !w.minimized);
                        if let Some(w) = focused {
                            if w.window_type == WindowType::TextEditor {
                                if let Some(editor) = d.editor_states.get(&w.id) {
                                    editor.find_query.is_some() || editor.goto_line_input.is_some()
                                } else { false }
                            } else { false }
                        } else { false }
                    };
                    if loi {
                        let sa = d.windows.iter().find(|w| w.focused && !w.minimized).map(|w| w.id);
                        if let Some(id) = sa {
                            if let Some(editor) = d.editor_states.get_mut(&id) {
                                editor.handle_key(27);
                            }
                        }
                        drop(d);
                        continue;
                    }
                    
                    
                    let kdz = {
                        let focused = d.windows.iter().find(|w| w.focused && !w.minimized);
                        focused.map(|w| w.window_type == WindowType::Browser).unwrap_or(false)
                    };
                    if kdz {
                        d.handle_keyboard_input(27);
                        drop(d);
                        continue;
                    }
                    
                    let enb = d.windows.iter().find(|w| w.focused && !w.minimized).map(|w| w.id);
                    if let Some(sa) = enb {
                        crate::serial_println!("[GUI] ESC: closing window {}", sa);
                        d.close_focused_window();
                        crate::serial_println!("[GUI] ESC: window closed OK");
                    } else {
                        crate::serial_println!("[GUI] ESC: no focused window, ignoring");
                    }
                    drop(d);
                } else {
                    crate::serial_println!("[GUI] ESC: lock busy, skipping");
                }
                continue;
            }
            
            
            
            {
                static mut ADP_: bool = false;
                let hxp = crate::keyboard::sx(0x3B);
                unsafe {
                    if hxp && !adf && !aw && !ADP_ {
                        ADP_ = true;
                        let mut d = S.lock();
                        d.show_shortcuts = !d.show_shortcuts;
                        crate::serial_println!("[GUI] F1: shortcuts overlay = {}", d.show_shortcuts);
                        drop(d);
                    }
                    if !hxp { ADP_ = false; }
                }
            }
            
            
            if (adf || aw || hdh) && key == 9 {
                if !engine::dsk() {
                    engine::jig();
                } else {
                    engine::hfb();
                }
                continue;
            }
            
            
            if aw && key == crate::keyboard::AI_ {
                S.lock().snap_focused_window(SnapDir::Left);
                unsafe { DI_ = true; }
                continue;
            }
            
            if aw && key == crate::keyboard::AJ_ {
                S.lock().snap_focused_window(SnapDir::Right);
                unsafe { DI_ = true; }
                continue;
            }
            
            if aw && key == crate::keyboard::T_ {
                S.lock().toggle_maximize_focused();
                unsafe { DI_ = true; }
                continue;
            }
            
            if aw && key == crate::keyboard::S_ {
                S.lock().minimize_focused_window();
                unsafe { DI_ = true; }
                continue;
            }
            
            
            if aw && (key == b'd' || key == b'D') {
                S.lock().toggle_show_desktop();
                unsafe { DI_ = true; }
                crate::serial_println!("[GUI] Win+D: toggle show desktop");
                continue;
            }
            
            
            if aw && (key == b'e' || key == b'E') {
                S.lock().create_window("File Explorer", 100, 60, 780, 520, WindowType::FileManager);
                unsafe { DI_ = true; }
                crate::serial_println!("[GUI] Win+E: open file manager");
                continue;
            }
            
            
            if aw && (key == b'i' || key == b'I') {
                S.lock().open_settings_panel();
                unsafe { DI_ = true; }
                crate::serial_println!("[GUI] Win+I: open settings");
                continue;
            }
            
            
            if aw && (key == b'h' || key == b'H') {
                crate::accessibility::gzg();
                let mut d = S.lock();
                d.needs_full_redraw = true;
                d.background_cached = false;
                drop(d);
                unsafe { DI_ = true; }
                crate::serial_println!("[GUI] Win+H: toggle high contrast");
                continue;
            }
            
            
            if aw && (key == b'l' || key == b'L') {
                let mut d = S.lock();
                d.lock_screen_active = true;
                d.lock_screen_input.clear();
                d.lock_screen_shake = 0;
                drop(d);
                unsafe { DI_ = true; }
                crate::serial_println!("[GUI] Win+L: lock screen");
                continue;
            }
            
            
            if aw && key != 0 {
                unsafe { DI_ = true; }
            }
            
            
            
            if adf && crate::keyboard::sx(0x3E) {
                let mut d = S.lock();
                let mjl = d.windows.iter().any(|w| w.focused && !w.minimized);
                if mjl {
                    d.close_focused_window();
                    crate::serial_println!("[GUI] Alt+F4: closed focused window");
                } else {
                    crate::serial_println!("[GUI] Alt+F4: no window, exiting desktop");
                    NW_.store(true, Ordering::SeqCst);
                }
                drop(d);
                continue;
            }
            
            
            crate::serial_println!("[MAIN-DBG] passing key {} (0x{:02X}) to handle_keyboard", key, key);
            mhy(key);
        }
        
        
        if engine::dsk() {
            let fgx = crate::keyboard::sx(0x38);
            let hcj = crate::keyboard::sx(0x5B);
            let fph = crate::keyboard::sx(0x1D);
            if !fgx && !hcj && !fph {
                let selected = engine::lwe();
                S.lock().focus_window_by_index(selected as usize);
            }
        }
        
        
        
        static mut AGD_: bool = false;
        static mut DI_: bool = false;
        {
            let fff = crate::keyboard::sx(0x5B);
            unsafe {
                if fff && !AGD_ {
                    
                    DI_ = false;
                }
                if fff {
                    
                    if engine::dsk() {
                        DI_ = true;
                    }
                }
                if !fff && AGD_ && !DI_ {
                    
                    let mut d = S.lock();
                    d.start_menu_open = !d.start_menu_open;
                }
                AGD_ = fff;
            }
        }
        
        
        static mut BAV_: bool = false;
        let left = mouse.left_button;
        unsafe {
            if left != BAV_ {
                if left {
                    crate::serial_println!("[INPUT-DBG] mouse click at ({},{})", mouse.x, mouse.y);
                }
                
                if left {
                    let mut d = S.lock();
                    if d.start_menu_open {
                        
                    }
                    drop(d);
                }
                handle_click(mouse.x, mouse.y, left);
                BAV_ = left;
            }
        }
        
        
        static mut AGA_: bool = false;
        static mut BGO_: bool = false;
        let right = mouse.right_button;
        unsafe {
            if !BGO_ {
                
                AGA_ = right;
                BGO_ = true;
            }
            if right != AGA_ {
                handle_right_click(mouse.x, mouse.y, right);
                AGA_ = right;
            }
        }
        
        
        let scroll = crate::mouse::mds();
        if scroll != 0 {
            handle_scroll(scroll);
        }
        
        
        
        
        {
            let mut d = S.lock();
            d.process_touch_input();
            drop(d);
        }
        
        
        
        
        {
            let result = {
                let mut r = AZR_.lock();
                r.take()
            };
            if let Some(lines) = result {
                let mut d = S.lock();
                
                if let Some(window) = d.windows.iter_mut().find(|w| w.window_type == WindowType::Terminal) {
                    
                    if window.content.last().map(|j| j.contains("$ ")).unwrap_or(false) {
                        window.content.pop();
                    }
                    
                    for line in &lines {
                        window.content.push(line.clone());
                    }
                    
                    window.content.push(Desktop::aya("_"));
                    
                    let line_height = 16usize;
                    let chp = (window.height as usize).saturating_sub(J_() as usize + 16);
                    let oe = if line_height > 0 { chp / line_height } else { 1 };
                    if window.content.len() > oe {
                        window.scroll_offset = window.content.len() - oe;
                    } else {
                        window.scroll_offset = 0;
                    }
                }
                drop(d);
            }
        }

        
        
        
        {
            let result = {
                let mut r = ANT_.lock();
                r.take()
            };
            if let Some(gir) = result {
                let mut d = S.lock();
                match gir {
                    Ok((final_url, status_code, headers, body)) => {
                        crate::serial_println!("[BROWSER-BG] Received {} bytes, status {}", body.len(), status_code);
                        if let Some(ref mut browser) = d.browser {
                            if status_code >= 400 {
                                browser.status = crate::browser::BrowserStatus::Error(alloc::format!("HTTP {}", status_code));
                                browser.raw_html = alloc::format!(
                                    "<html><body><h1>HTTP Error {}</h1><p>The server returned an error for {}</p></body></html>",
                                    status_code, final_url
                                );
                                browser.document = Some(crate::browser::boe(&browser.raw_html));
                            } else if status_code >= 300 && status_code < 400 {
                                
                                let axx = headers.iter()
                                    .find(|(k, _)| k.to_lowercase() == "location")
                                    .map(|(_, v)| v.clone());
                                if let Some(loc) = axx {
                                    crate::serial_println!("[BROWSER-BG] Redirect {} -> {}", status_code, loc);
                                    d.browser_url_input = loc.clone();
                                    d.browser_url_cursor = d.browser_url_input.len();
                                    
                                    {
                                        let mut pending = ABC_.lock();
                                        *pending = Some(loc);
                                    }
                                    ST_.store(true, Ordering::SeqCst);
                                    crate::thread::dzu("browser-nav", hir, 0);
                                    drop(d);
                                    
                                    continue;
                                }
                            } else {
                                
                                let ajx = core::str::from_utf8(&body).unwrap_or("");
                                browser.raw_html = String::from(ajx);
                                browser.process_set_cookies(&headers, &final_url);
                                browser.document = Some(crate::browser::boe(ajx));
                                browser.execute_scripts();
                                browser.extract_resources(&final_url);
                                
                                if browser.history_index < browser.history.len() {
                                    browser.history.truncate(browser.history_index);
                                }
                                browser.history.push(final_url.clone());
                                browser.history_index = browser.history.len();
                                browser.current_url = final_url.clone();
                                browser.scroll_y = 0;
                                browser.status = crate::browser::BrowserStatus::Ready;
                            }
                            d.browser_url_input = browser.current_url.clone();
                            d.browser_url_cursor = d.browser_url_input.len();
                        }
                    }
                    Err(e) => {
                        crate::serial_println!("[BROWSER-BG] Navigation error: {}", e);
                        if let Some(ref mut browser) = d.browser {
                            browser.status = crate::browser::BrowserStatus::Error(e.clone());
                            browser.raw_html = alloc::format!(
                                "<html><body><h1>Error</h1><p>{}</p></body></html>", e
                            );
                            browser.document = Some(crate::browser::boe(&browser.raw_html));
                            browser.scroll_y = 0;
                        }
                    }
                }
                d.browser_loading = false;
                drop(d);
            }
        }

        
        
        
        
        
        draw();
        
        
        if engine::dsk() {
            ofe();
        }
        
        
        
        
        
        
        
        
        {
            let d = S.lock();
            let dzi = d.show_shortcuts;
            drop(d);
            if dzi {
                ofq();
            }
        }
        
        
        ofo();
        
        
        {
            let d = S.lock();
            let w = d.width;
            let h = d.height;
            drop(d);
            let lyl = engine::yy().saturating_sub(cyd);
            crate::devtools::ofi(w, h, lyl);
        }
        
        
        #[cfg(debug_assertions)]
        {
            let fps = engine::fyp();
            
        }
        
        
        
        
        let izz = engine::yy().saturating_sub(cyd);
        
        {
            let d = S.lock();
            let br = d.frame_count;
            drop(d);
            if br % 120 == 0 && br > 0 {
                let fps = crate::gui::vsync::fps();
                crate::serial_println!("[PERF] frame={} render={}us fps={}", br, izz, fps);
            }
        }
        crate::gui::vsync::lym(cyd);
        ilg = ilg.saturating_add(1);
    }
    
    
    
    
    crate::serial_println!("[GUI] Desktop exiting, cleaning up...");
    
    {
        let mut d = S.lock();
        for (_id, ic) in d.music_player_states.iter_mut() {
            ic.stop();
        }
        crate::serial_println!("[GUI] All music players stopped");
    }
    crate::framebuffer::pr(false);
    crate::framebuffer::clear();
    crate::serial_println!("[GUI] Desktop exited cleanly");
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SnapDir {
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}


fn ofe() {
    let desktop = S.lock();
    let eef = desktop.get_window_info();
    if eef.is_empty() { return; }
    
    let screen_w = desktop.width;
    let screen_h = desktop.height;
    drop(desktop);
    
    let selection = crate::gui::engine::jvj();
    let count = eef.len() as i32;
    let idx = ((selection % count) + count) % count;
    
    
    let aji: u32 = 150;
    let aev: u32 = 100;
    let gap: u32 = 12;
    let aac: u32 = 6; 
    let hbq = (eef.len() as u32).min(aac);
    let aaj = hbq * (aji + gap) + gap;
    let pjz: u32 = 30;
    let sn = aev + gap * 2 + pjz + 14;
    
    let fh = (screen_w as i32 - aaj as i32) / 2;
    let hk = (screen_h as i32 - sn as i32) / 2;
    
    
    draw_rounded_rect(fh - 2, hk - 2, aaj + 4, sn + 4, 14, 0x40000000);
    draw_rounded_rect(fh, hk, aaj, sn, 12, 0xE8101420);
    
    iu(fh, hk, aaj, sn, 12, 0x3000FF66);
    
    axd(fh + 14, hk + 1, aaj as i32 - 28, 1, 0x20FFFFFF);
    
    
    draw_text_centered(fh + aaj as i32 / 2, hk + 8, "Switch Window", 0xFF888888);
    
    
    for (i, (title, wt)) in eef.iter().enumerate() {
        if i as u32 >= aac { break; }
        let cx = fh + gap as i32 + i as i32 * (aji + gap) as i32;
        let u = hk + gap as i32 + 22;
        
        let hd = i as i32 == idx;
        
        
        if hd {
            
            draw_rounded_rect(cx - 2, u - 2, aji + 4, aev + 4, 8, 0x3000FF66);
            draw_rounded_rect(cx, u, aji, aev, 6, 0xFF1A2A20);
            
            iu(cx, u, aji, aev, 6, 0xFF00CC55);
        } else {
            draw_rounded_rect(cx, u, aji, aev, 6, 0xFF1A1E28);
            iu(cx, u, aji, aev, 6, 0xFF2A2E38);
        }
        
        
        let icon = put(*wt);
        let adt = cx + (aji as i32 - crate::graphics::scaling::auh(icon) as i32) / 2;
        let adu = u + 20;
        let icon_color = if hd { 0xFF00FF66 } else { 0xFF667766 };
        draw_text(adt, adu, icon, icon_color);
        
        
        let hah = puu(*wt);
        let ace = if hd { 0xFF00CC55 } else { 0xFF555555 };
        draw_text_centered(cx + aji as i32 / 2, u + aev as i32 - 22, hah, ace);
        
        
        let dzh: alloc::string::String = title.chars().take(16).collect();
        let bwl = if hd { 0xFFFFFFFF } else { 0xFF999999 };
        draw_text_centered(cx + aji as i32 / 2, u + aev as i32 + 6, &dzh, bwl);
    }
    
    
    if eef.len() > 1 {
        draw_text_centered(fh + aaj as i32 / 2, hk + sn as i32 - 18, 
            "Tab: next  |  Release Alt: select", 0xFF555555);
    }
}


fn ofq() {
    let desktop = S.lock();
    let screen_w = desktop.width;
    let screen_h = desktop.height;
    drop(desktop);

    
    let cgr: &[(&str, &[(&str, &str)])] = &[
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

    
    let ble: u32 = 2;
    let col_w: u32 = 300;
    let ep: u32 = 18;
    let fkz: u32 = 8;
    let acc: u32 = 40;
    let hzm: u32 = 24;
    let oq: u32 = 20;

    
    let mut crx: u32 = 0;
    for (_, entries) in cgr.iter() {
        crx += 1 + entries.len() as u32; 
    }
    
    let jbo = (crx + 1) / 2;
    let en = jbo * ep + ((cgr.len() as u32 + 1) / 2) * fkz;
    let he = ble * col_w + oq * 3;
    let ug = acc + en + hzm + oq;

    let fh = (screen_w as i32 - he as i32) / 2;
    let hk = (screen_h as i32 - ug as i32) / 2;

    
    draw_rounded_rect(fh - 2, hk - 2, he + 4, ug + 4, 14, 0x50000000);
    draw_rounded_rect(fh, hk, he, ug, 12, 0xF0101420);
    iu(fh, hk, he, ug, 12, 0x5000FF66);
    
    axd(fh + 14, hk + 1, he as i32 - 28, 1, 0x20FFFFFF);

    
    draw_text_centered(fh + he as i32 / 2, hk + 12, "Keyboard Shortcuts", 0xFF00FF66);
    
    axd(fh + oq as i32, hk + acc as i32 - 6, he as i32 - oq as i32 * 2, 1, 0x3000FF66);

    
    let mut col = 0u32;
    let mut ddq = 0u32;
    let mut fky = 0usize;

    for (dki, entries) in cgr.iter() {
        
        let giu = 1 + entries.len() as u32;
        if ddq + giu > jbo && col < ble - 1 {
            col += 1;
            ddq = 0;
        }

        let cx = fh + oq as i32 + col as i32 * (col_w as i32 + oq as i32);
        let u = hk + acc as i32 + ddq as i32 * ep as i32 + fky as i32 * fkz as i32;

        
        draw_text(cx, u, dki, 0xFF00CC55);
        ddq += 1;

        
        for (key, desc) in entries.iter() {
            let qz = hk + acc as i32 + ddq as i32 * ep as i32 + fky as i32 * fkz as i32;
            
            let bhl = crate::graphics::scaling::auh(key) as i32 + 10;
            draw_rounded_rect(cx + 4, qz - 1, bhl as u32, 16, 4, 0xFF1A2A20);
            iu(cx + 4, qz - 1, bhl as u32, 16, 4, 0xFF00AA44);
            draw_text(cx + 9, qz + 1, key, 0xFF00FF66);
            
            draw_text(cx + bhl + 14, qz + 1, desc, 0xFFAAAAAA);
            ddq += 1;
        }
        fky += 1;
    }

    
    draw_text_centered(fh + he as i32 / 2, hk + ug as i32 - hzm as i32 + 4,
        "Press F1 to close", 0xFF555555);
}


fn put(wt: WindowType) -> &'static str {
    match wt {
        WindowType::Terminal => ">_",
        WindowType::SystemInfo => "[i]",
        WindowType::About => "(?)",
        WindowType::Calculator => "[#]",
        WindowType::FileManager => "[/]",
        WindowType::TextEditor => "[=]",
        WindowType::Cn => "[~]",
        WindowType::Settings => "{*}",
        WindowType::ImageViewer => "[^]",
        WindowType::Browser => "</>",
        WindowType::Game => "[*]",
        WindowType::Chess | WindowType::Chess3D => "[K]",
        WindowType::ModelEditor => "[3D]",
        WindowType::Game3D => "[3D]",
        WindowType::MusicPlayer => "[~]",
        WindowType::LabMode => "{L}",
        WindowType::BinaryViewer => "0x",
        _ => "[.]",
    }
}


fn puu(wt: WindowType) -> &'static str {
    match wt {
        WindowType::Terminal => "Terminal",
        WindowType::SystemInfo => "System",
        WindowType::About => "About",
        WindowType::Calculator => "Calc",
        WindowType::FileManager => "Files",
        WindowType::TextEditor => "Editor",
        WindowType::Cn => "NetScan",
        WindowType::Settings => "Settings",
        WindowType::ImageViewer => "Images",
        WindowType::Browser => "Browser",
        WindowType::Game => "Snake",
        WindowType::Chess => "Chess",
        WindowType::Chess3D => "Chess 3D",
        WindowType::ModelEditor => "3D Edit",
        WindowType::Game3D => "FPS",
        WindowType::MusicPlayer => "Music",
        WindowType::LabMode => "Lab",
        WindowType::BinaryViewer => "BinView",
        _ => "Window",
    }
}


fn qtw() {
    use crate::gui::engine::{ibw, StartAction};
    
    let desktop = S.lock();
    let screen_h = desktop.height;
    drop(desktop);
    
    let pz: u32 = 280;
    let rv: u32 = 350;
    let x: i32 = 10;
    let y: i32 = screen_h as i32 - V_() as i32 - rv as i32 - 5;
    
    
    draw_rounded_rect(x, y, pz, rv, 12, 0xF0101520);
    draw_rounded_rect(x + 1, y + 1, pz - 2, rv - 2, 11, 0xF0181C25);
    
    
    draw_text(x + 20, y + 15, "TrustOS", 0xFF00FF66);
    draw_text(x + 90, y + 15, "v0.1", 0xFF606060);
    
    
    draw_line(x + 15, y + 35, x + pz as i32 - 15, y + 35, 0xFF303540);
    
    
    let items = ibw();
    let mut gg = y + 45;
    
    for item in items.iter() {
        if item.icon == 255 {
            
            draw_line(x + 15, gg + 5, x + pz as i32 - 15, gg + 5, 0xFF303540);
            gg += 12;
        } else {
            
            draw_text(x + 40, gg, &item.name, 0xFFCCCCCC);
            gg += 28;
        }
    }
    
    
    let agz = y + rv as i32 - 45;
    draw_rounded_rect(x + 15, agz, pz - 30, 30, 6, 0xFF252A35);
    draw_text(x + 25, agz + 7, "Search apps...", 0xFF606060);
}


fn ofo() {
    use crate::gui::engine::{ibr, NotifyPriority};
    
    let desktop = S.lock();
    let screen_w = desktop.width;
    drop(desktop);
    
    let ayp = ibr();
    if ayp.is_empty() { return; }
    
    let mut y: i32 = 55; 
    
    for toast in ayp.iter() {
        let w: u32 = 320;
        let mjw = toast.progress.is_some();
        let h: u32 = if mjw { 78 } else { 64 };
        let opacity = toast.opacity();
        if opacity == 0 { continue; }
        
        
        let bb = toast.elapsed_ms();
        let otr = if bb < 300 {
            ((300 - bb) * 40 / 300) as i32
        } else {
            0
        };
        let x = screen_w as i32 - w as i32 - 15 + otr;
        
        
        let kbn = (opacity as u32 * 0xF0 / 255) << 24;
        let bg_color = kbn | 0x00141820;
        
        
        let caf = (opacity as u32 * 0x18 / 255) << 24;
        draw_rounded_rect(x - 1, y - 1, w + 2, h + 2, 11, caf | 0x00000000);
        
        
        draw_rounded_rect(x, y, w, h, 10, bg_color);
        
        
        let eok = (opacity as u32 * 0x15 / 255) << 24;
        axd(x + 12, y + 1, w as i32 - 24, 1, eok | 0x00FFFFFF);
        
        
        let zr = toast.get_color();
        let jti = (opacity as u32 * ((zr >> 24) & 0xFF) / 255) << 24;
        let hdr = zr & 0x00FFFFFF;
        axd(x + 2, y + 8, 3, h as i32 - 16, jti | hdr);
        
        
        let icon = match toast.priority {
            NotifyPriority::Info => "[i]",
            NotifyPriority::Warning => "/!\\",
            NotifyPriority::Error => "[X]",
            NotifyPriority::Success => "[v]",
        };
        let mne = (opacity as u32 * 0xFF / 255) << 24;
        draw_text(x + 14, y + 12, icon, mne | hdr);
        
        
        let pjy = (opacity as u32 * 0xFF / 255) << 24;
        let pkf: alloc::string::String = toast.title.chars().take(28).collect();
        draw_text(x + 48, y + 12, &pkf, pjy | 0x00EEEEEE);
        
        
        let ngn = (opacity as u32 * 0xBB / 255) << 24;
        let ngp: alloc::string::String = toast.message.chars().take(36).collect();
        draw_text(x + 14, y + 34, &ngp, ngn | 0x00999999);
        
        
        if let Some(bup) = toast.progress {
            let gk = y + 54;
            let ek = w - 28;
            let fic = (opacity as u32 * 0xFF / 255) << 24;
            draw_rounded_rect(x + 14, gk, ek, 8, 3, fic | 0x00252A35);
            let rb = (ek * bup as u32 / 100).max(1);
            if rb > 4 {
                draw_rounded_rect(x + 14, gk, rb, 8, 3, fic | 0x0000CC55);
            }
            
            let ewk = alloc::format!("{}%", bup);
            draw_text(x + w as i32 - 40, gk - 1, &ewk, fic | 0x00777777);
        }
        
        
        let kdo = (opacity as u32 * 0x10 / 255) << 24;
        axd(x + 10, y + h as i32 - 1, w as i32 - 20, 1, kdo | 0x00FFFFFF);
        
        y += h as i32 + 8;
    }
}


fn draw_text(x: i32, y: i32, text: &str, color: u32) {
    crate::graphics::scaling::ekr(x, y, text, color);
}


fn draw_text_centered(cx: i32, y: i32, text: &str, color: u32) {
    let w = crate::graphics::scaling::auh(text) as i32;
    draw_text(cx - w / 2, y, text, color);
}


fn draw_line(x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
    
    if y1 == y2 {
        let (eek, csz) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        for x in eek..=csz {
            if x >= 0 {
                crate::framebuffer::draw_pixel(x as u32, y1 as u32, color);
            }
        }
    }
}


fn draw_rect(x: i32, y: i32, w: u32, h: u32, color: u32) {
    for ad in 0..h {
        for dx in 0..w {
            crate::framebuffer::draw_pixel((x + dx as i32) as u32, (y + ad as i32) as u32, color);
        }
    }
}


fn lko(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32, alpha: u32) {
    if w == 0 || h == 0 { return; }
    let r = radius.min(w / 2).min(h / 2);

    if r == 0 {
        if x >= 0 && y >= 0 {
            crate::framebuffer::co(x as u32, y as u32, w, h, color, alpha);
        }
        return;
    }

    let ld = w as i32;
    let hi = h as i32;
    let dk = r as i32;

    cjm(x, y + dk, ld, hi - dk * 2, color, alpha);
    cjm(x + dk, y, ld - dk * 2, dk, color, alpha);
    cjm(x + dk, y + hi - dk, ld - dk * 2, dk, color, alpha);

    let ju = dk * dk;
    for ad in 0..dk {
        let dx = cxr(ju - ad * ad);
        cjm(x + dk - dx, y + dk - ad - 1, dx, 1, color, alpha);
        cjm(x + ld - dk, y + dk - ad - 1, dx, 1, color, alpha);
        cjm(x + dk - dx, y + hi - dk + ad, dx, 1, color, alpha);
        cjm(x + ld - dk, y + hi - dk + ad, dx, 1, color, alpha);
    }
}


fn draw_rounded_rect(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    let r = radius.min(w / 2).min(h / 2);

    if r == 0 {
        
        if x >= 0 && y >= 0 {
            crate::framebuffer::fill_rect(x as u32, y as u32, w, h, color);
        }
        return;
    }

    let ld = w as i32;
    let hi = h as i32;
    let dk = r as i32;

    
    
    axd(x, y + dk, ld, hi - dk * 2, color);
    
    axd(x + dk, y, ld - dk * 2, dk, color);
    
    axd(x + dk, y + hi - dk, ld - dk * 2, dk, color);

    
    
    let ju = dk * dk;
    for ad in 0..dk {
        
        let dx = cxr(ju - ad * ad);
        
        axd(x + dk - dx, y + dk - ad - 1, dx, 1, color);
        
        axd(x + ld - dk, y + dk - ad - 1, dx, 1, color);
        
        axd(x + dk - dx, y + hi - dk + ad, dx, 1, color);
        
        axd(x + ld - dk, y + hi - dk + ad, dx, 1, color);
    }
}


fn iu(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    let r = radius.min(w / 2).min(h / 2);
    let ld = w as i32;
    let hi = h as i32;
    let dk = r as i32;

    if r == 0 {
        if x >= 0 && y >= 0 {
            crate::framebuffer::draw_rect(x as u32, y as u32, w, h, color);
        }
        return;
    }

    
    for p in dk..ld - dk {
        bdc(x + p, y, color);            
        bdc(x + p, y + hi - 1, color);   
    }
    for o in dk..hi - dk {
        bdc(x, y + o, color);            
        bdc(x + ld - 1, y + o, color);   
    }

    
    let mut cx = dk;
    let mut u = 0i32;
    let mut err = 0i32;
    while cx >= u {
        
        bdc(x + dk - cx, y + dk - u, color);
        bdc(x + dk - u, y + dk - cx, color);
        
        bdc(x + ld - 1 - dk + cx, y + dk - u, color);
        bdc(x + ld - 1 - dk + u, y + dk - cx, color);
        
        bdc(x + dk - cx, y + hi - 1 - dk + u, color);
        bdc(x + dk - u, y + hi - 1 - dk + cx, color);
        
        bdc(x + ld - 1 - dk + cx, y + hi - 1 - dk + u, color);
        bdc(x + ld - 1 - dk + u, y + hi - 1 - dk + cx, color);

        u += 1;
        err += 1 + 2 * u;
        if 2 * (err - cx) + 1 > 0 {
            cx -= 1;
            err += 1 - 2 * cx;
        }
    }
}


#[inline]
fn axd(x: i32, y: i32, w: i32, h: i32, color: u32) {
    if w <= 0 || h <= 0 { return; }
    let p = x.max(0) as u32;
    let o = y.max(0) as u32;
    let aq = if x < 0 { (w + x).max(0) as u32 } else { w as u32 };
    let ch = if y < 0 { (h + y).max(0) as u32 } else { h as u32 };
    if aq > 0 && ch > 0 {
        crate::framebuffer::fill_rect(p, o, aq, ch, color);
    }
}


fn cjm(x: i32, y: i32, w: i32, h: i32, color: u32, alpha: u32) {
    if w <= 0 || h <= 0 { return; }
    let p = x.max(0) as u32;
    let o = y.max(0) as u32;
    let aq = if x < 0 { (w + x).max(0) as u32 } else { w as u32 };
    let ch = if y < 0 { (h + y).max(0) as u32 } else { h as u32 };
    if aq > 0 && ch > 0 {
        crate::framebuffer::co(p, o, aq, ch, color, alpha);
    }
}


#[inline]
fn bdc(x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 {
        crate::framebuffer::draw_pixel(x as u32, y as u32, color);
    }
}


#[inline]
fn cxr(v: i32) -> i32 {
    if v <= 0 { return 0; }
    let mut x = v;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + v / x) / 2;
    }
    x
}


#[inline]
fn ey() -> u64 {
    crate::arch::timestamp()
}


pub fn set_render_mode(mode: RenderMode) {
    S.lock().set_render_mode(mode);
    let bcu = match mode {
        RenderMode::Classic => "Classic",
        RenderMode::OpenGL => "OpenGL Compositor",
        RenderMode::GpuAccelerated => "GPU Accelerated",
    };
    crate::serial_println!("[GUI] Render mode: {}", bcu);
}


pub fn set_theme(theme: CompositorTheme) {
    S.lock().set_theme(theme);
    let cei = match theme {
        CompositorTheme::Flat => "Flat",
        CompositorTheme::Modern => "Modern",
        CompositorTheme::Glass => "Glass",
        CompositorTheme::Neon => "Neon",
        CompositorTheme::Minimal => "Minimal",
    };
    crate::serial_println!("[GUI] Compositor theme: {}", cei);
}


pub fn qij() -> RenderMode {
    S.lock().render_mode
}
