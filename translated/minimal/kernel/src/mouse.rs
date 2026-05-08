




use crate::arch::Port;
use spin::Mutex;
use core::sync::atomic::{AtomicI32, AtomicBool, AtomicI8, AtomicU8, AtomicU64, Ordering};


const HR_: u16 = 0x60;
const HS_: u16 = 0x64;
const AIM_: u16 = 0x64;


static JG_: AtomicI32 = AtomicI32::new(640);
static JH_: AtomicI32 = AtomicI32::new(400);
static WA_: AtomicBool = AtomicBool::new(false);
static YA_: AtomicBool = AtomicBool::new(false);
static AHH_: AtomicBool = AtomicBool::new(false);
static YC_: AtomicI8 = AtomicI8::new(0);


static JP_: AtomicI32 = AtomicI32::new(1280);
static JO_: AtomicI32 = AtomicI32::new(800);


#[derive(Clone, Copy, Debug, Default)]
pub struct Abn {
    pub x: i32,
    pub y: i32,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    pub ezq: i8,
}


static BAS_: AtomicU64 = AtomicU64::new(0);
static ABN_: Mutex<u8> = Mutex::new(0);


static AEY_: AtomicBool = AtomicBool::new(false);


static BEM_: AtomicU8 = AtomicU8::new(0);
static BEN_: AtomicU8 = AtomicU8::new(0);
static BEO_: AtomicU8 = AtomicU8::new(0);
static BEP_: AtomicU8 = AtomicU8::new(0);
static XI_: AtomicU8 = AtomicU8::new(0);


fn ptl() {
    let mut status = Port::<u8>::new(HS_);
    for _ in 0..100_000 {
        if unsafe { status.read() } & 0x01 != 0 {
            return;
        }
        core::hint::spin_loop();
    }
}


fn jqs() {
    let mut status = Port::<u8>::new(HS_);
    for _ in 0..100_000 {
        if unsafe { status.read() } & 0x02 == 0 {
            return;
        }
        core::hint::spin_loop();
    }
}


fn dwy(cmd: u8) {
    let mut command = Port::<u8>::new(AIM_);
    jqs();
    unsafe { command.write(cmd); }
}


fn gov(data: u8) {
    let mut port = Port::<u8>::new(HR_);
    jqs();
    unsafe { port.write(data); }
}


fn exf() -> u8 {
    let mut port = Port::<u8>::new(HR_);
    ptl();
    unsafe { port.read() }
}


fn dbi(cmd: u8) {
    dwy(0xD4); 
    gov(cmd);
    exf(); 
}


fn gig(data: u8) {
    dwy(0xD4); 
    gov(data);
    exf(); 
}


pub fn init() {
    
    dwy(0xA8);
    
    
    dwy(0x20);
    let status = exf();
    
    
    let status = (status | 0x02) & !0x20;
    dwy(0x60);
    gov(status);
    
    
    dbi(0xF6);
    
    
    
    
    dbi(0xF3); gig(200); 
    dbi(0xF3); gig(100); 
    dbi(0xF3); gig(80);  
    
    
    dbi(0xF2); 
    let device_id = exf();
    if device_id == 3 || device_id == 4 {
        AEY_.store(true, Ordering::Relaxed);
        crate::serial_println!("[MOUSE] IntelliMouse scroll wheel enabled (ID={})", device_id);
    }
    
    
    dbi(0xF4);
    
    crate::serial_println!("[MOUSE] PS/2 mouse initialized (ID={})", device_id);
}


pub fn set_screen_size(width: u32, height: u32) {
    JP_.store(width as i32, Ordering::Relaxed);
    JO_.store(height as i32, Ordering::Relaxed);
}



pub fn btc() {
    let mut zu = Port::<u8>::new(HR_);
    let byte = unsafe { zu.read() };
    
    
    let fzz = AEY_.load(Ordering::Relaxed);
    let npd: u8 = if fzz { 4 } else { 3 };
    let idx = XI_.load(Ordering::Relaxed);
    
    
    if idx == 0 && byte & 0x08 == 0 {
        return; 
    }
    
    
    match idx {
        0 => BEM_.store(byte, Ordering::Relaxed),
        1 => BEN_.store(byte, Ordering::Relaxed),
        2 => BEO_.store(byte, Ordering::Relaxed),
        3 => BEP_.store(byte, Ordering::Relaxed),
        _ => {
            
            XI_.store(0, Ordering::Relaxed);
            return;
        }
    }
    
    let euz = idx + 1;
    if euz < npd {
        XI_.store(euz, Ordering::Relaxed);
        return;
    }
    
    
    XI_.store(0, Ordering::Relaxed);
    
    
    let kl = BEM_.load(Ordering::Relaxed);
    let gf = BEN_.load(Ordering::Relaxed);
    let iq = BEO_.load(Ordering::Relaxed);
    let sc = if fzz { BEP_.load(Ordering::Relaxed) } else { 0 };
    
    
    WA_.store(kl & 0x01 != 0, Ordering::Relaxed);
    YA_.store(kl & 0x02 != 0, Ordering::Relaxed);
    AHH_.store(kl & 0x04 != 0, Ordering::Relaxed);
    
    
    let gpv = gf as i8 as i32;
    let cox = iq as i8 as i32;
    
    
    let (x_rel, y_rel) = crate::accessibility::jxe(gpv, cox);
    
    
    let pvq = kl & 0x40 != 0;
    let pwe = kl & 0x80 != 0;
    if pvq || pwe {
        return;
    }
    
    let width = JP_.load(Ordering::Relaxed);
    let height = JO_.load(Ordering::Relaxed);
    
    let gks = JG_.load(Ordering::Relaxed);
    let gkt = JH_.load(Ordering::Relaxed);
    
    
    let cbw = (gks + x_rel).clamp(0, width - 1);
    let afk = (gkt - y_rel).clamp(0, height - 1);
    
    JG_.store(cbw, Ordering::Relaxed);
    JH_.store(afk, Ordering::Relaxed);
    
    
    if fzz {
        let jsh = sc as i8;
        if jsh != 0 {
            let _ = YC_.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |qb| {
                Some(qb.saturating_add(jsh))
            });
        }
    }
}


pub fn get_state() -> Abn {
    Abn {
        x: JG_.load(Ordering::Relaxed),
        y: JH_.load(Ordering::Relaxed),
        left_button: WA_.load(Ordering::Relaxed),
        right_button: YA_.load(Ordering::Relaxed),
        middle_button: AHH_.load(Ordering::Relaxed),
        ezq: YC_.swap(0, Ordering::Relaxed),
    }
}


pub fn mqc(dx: i32, ad: i32, left: bool, right: bool, middle: bool, scroll: i8) {
    let w = JP_.load(Ordering::Relaxed);
    let h = JO_.load(Ordering::Relaxed);

    
    let cbw = JG_.load(Ordering::Relaxed) + dx;
    let afk = JH_.load(Ordering::Relaxed) + ad;
    JG_.store(cbw.max(0).min(w - 1), Ordering::Relaxed);
    JH_.store(afk.max(0).min(h - 1), Ordering::Relaxed);

    WA_.store(left, Ordering::Relaxed);
    YA_.store(right, Ordering::Relaxed);
    AHH_.store(middle, Ordering::Relaxed);
    if scroll != 0 {
        YC_.store(scroll, Ordering::Relaxed);
    }
}


pub fn mds() -> i8 {
    YC_.swap(0, Ordering::Relaxed)
}


pub fn odr() {
    
    let cy = crate::logger::eg();
    let last = BAS_.load(Ordering::Relaxed);
    BAS_.store(cy, Ordering::Relaxed);
    
    let mut count = ABN_.lock();
    
    if cy - last < 30 {
        *count = 2;
    } else {
        *count = 1;
    }
}


pub fn erj() -> bool {
    let count = ABN_.lock();
    *count >= 2
}


pub fn jah() {
    *ABN_.lock() = 0;
}


pub fn qif() -> (i32, i32) {
    (JG_.load(Ordering::Relaxed), JH_.load(Ordering::Relaxed))
}


pub fn qmm() -> bool {
    WA_.load(Ordering::Relaxed)
}


pub fn qmx() -> bool {
    YA_.load(Ordering::Relaxed)
}


pub fn is_initialized() -> bool {
    
    
    AEY_.load(Ordering::Relaxed) || true  
}





static CGZ_: AtomicI32 = AtomicI32::new(640);
static CHA_: AtomicI32 = AtomicI32::new(400);



pub fn qhp() -> Option<(i32, i32)> {
    let hpm = JG_.load(Ordering::Relaxed);
    let hpn = JH_.load(Ordering::Relaxed);
    let esj = CGZ_.swap(hpm, Ordering::Relaxed);
    let mwt = CHA_.swap(hpn, Ordering::Relaxed);
    
    let dx = hpm - esj;
    let ad = hpn - mwt;
    
    if dx != 0 || ad != 0 {
        Some((dx, ad))
    } else {
        None
    }
}
