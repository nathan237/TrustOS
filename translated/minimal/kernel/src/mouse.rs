




use crate::arch::Port;
use spin::Mutex;
use core::sync::atomic::{AtomicI32, AtomicBool, AtomicI8, AtomicU8, AtomicU64, Ordering};


const HA_: u16 = 0x60;
const HB_: u16 = 0x64;
const AGS_: u16 = 0x64;


static IN_: AtomicI32 = AtomicI32::new(640);
static IO_: AtomicI32 = AtomicI32::new(400);
static UR_: AtomicBool = AtomicBool::new(false);
static WT_: AtomicBool = AtomicBool::new(false);
static AFN_: AtomicBool = AtomicBool::new(false);
static WV_: AtomicI8 = AtomicI8::new(0);


static IW_: AtomicI32 = AtomicI32::new(1280);
static IV_: AtomicI32 = AtomicI32::new(800);


#[derive(Clone, Copy, Debug, Default)]
pub struct Bmr {
    pub b: i32,
    pub c: i32,
    pub jda: bool,
    pub pdd: bool,
    pub uoh: bool,
    pub jnw: i8,
}


static AYR_: AtomicU64 = AtomicU64::new(0);
static AAB_: Mutex<u8> = Mutex::new(0);


static ADI_: AtomicBool = AtomicBool::new(false);


static BCJ_: AtomicU8 = AtomicU8::new(0);
static BCK_: AtomicU8 = AtomicU8::new(0);
static BCL_: AtomicU8 = AtomicU8::new(0);
static BCM_: AtomicU8 = AtomicU8::new(0);
static VZ_: AtomicU8 = AtomicU8::new(0);


fn xtl() {
    let mut status = Port::<u8>::new(HB_);
    for _ in 0..100_000 {
        if unsafe { status.read() } & 0x01 != 0 {
            return;
        }
        core::hint::hc();
    }
}


fn pza() {
    let mut status = Port::<u8>::new(HB_);
    for _ in 0..100_000 {
        if unsafe { status.read() } & 0x02 == 0 {
            return;
        }
        core::hint::hc();
    }
}


fn hwc(cmd: u8) {
    let mut ro = Port::<u8>::new(AGS_);
    pza();
    unsafe { ro.write(cmd); }
}


fn lwa(f: u8) {
    let mut port = Port::<u8>::new(HA_);
    pza();
    unsafe { port.write(f); }
}


fn jkl() -> u8 {
    let mut port = Port::<u8>::new(HA_);
    xtl();
    unsafe { port.read() }
}


fn gmy(cmd: u8) {
    hwc(0xD4); 
    lwa(cmd);
    jkl(); 
}


fn lmt(f: u8) {
    hwc(0xD4); 
    lwa(f);
    jkl(); 
}


pub fn init() {
    
    hwc(0xA8);
    
    
    hwc(0x20);
    let status = jkl();
    
    
    let status = (status | 0x02) & !0x20;
    hwc(0x60);
    lwa(status);
    
    
    gmy(0xF6);
    
    
    
    
    gmy(0xF3); lmt(200); 
    gmy(0xF3); lmt(100); 
    gmy(0xF3); lmt(80);  
    
    
    gmy(0xF2); 
    let mx = jkl();
    if mx == 3 || mx == 4 {
        ADI_.store(true, Ordering::Relaxed);
        crate::serial_println!("[MOUSE] IntelliMouse scroll wheel enabled (ID={})", mx);
    }
    
    
    gmy(0xF4);
    
    crate::serial_println!("[MOUSE] PS/2 mouse initialized (ID={})", mx);
}


pub fn dbw(z: u32, ac: u32) {
    IW_.store(z as i32, Ordering::Relaxed);
    IV_.store(ac as i32, Ordering::Relaxed);
}



pub fn eck() {
    let mut axr = Port::<u8>::new(HA_);
    let hf = unsafe { axr.read() };
    
    
    let lbg = ADI_.load(Ordering::Relaxed);
    let vap: u8 = if lbg { 4 } else { 3 };
    let w = VZ_.load(Ordering::Relaxed);
    
    
    if w == 0 && hf & 0x08 == 0 {
        return; 
    }
    
    
    match w {
        0 => BCJ_.store(hf, Ordering::Relaxed),
        1 => BCK_.store(hf, Ordering::Relaxed),
        2 => BCL_.store(hf, Ordering::Relaxed),
        3 => BCM_.store(hf, Ordering::Relaxed),
        _ => {
            
            VZ_.store(0, Ordering::Relaxed);
            return;
        }
    }
    
    let jgx = w + 1;
    if jgx < vap {
        VZ_.store(jgx, Ordering::Relaxed);
        return;
    }
    
    
    VZ_.store(0, Ordering::Relaxed);
    
    
    let wu = BCJ_.load(Ordering::Relaxed);
    let of = BCK_.load(Ordering::Relaxed);
    let tb = BCL_.load(Ordering::Relaxed);
    let ajw = if lbg { BCM_.load(Ordering::Relaxed) } else { 0 };
    
    
    UR_.store(wu & 0x01 != 0, Ordering::Relaxed);
    WT_.store(wu & 0x02 != 0, Ordering::Relaxed);
    AFN_.store(wu & 0x04 != 0, Ordering::Relaxed);
    
    
    let lxf = of as i8 as i32;
    let fsa = tb as i8 as i32;
    
    
    let (xwj, xxa) = crate::accessibility::qjz(lxf, fsa);
    
    
    let xwh = wu & 0x40 != 0;
    let xwz = wu & 0x80 != 0;
    if xwh || xwz {
        return;
    }
    
    let z = IW_.load(Ordering::Relaxed);
    let ac = IV_.load(Ordering::Relaxed);
    
    let lqc = IN_.load(Ordering::Relaxed);
    let lqd = IO_.load(Ordering::Relaxed);
    
    
    let evh = (lqc + xwj).qp(0, z - 1);
    let bhn = (lqd - xxa).qp(0, ac - 1);
    
    IN_.store(evh, Ordering::Relaxed);
    IO_.store(bhn, Ordering::Relaxed);
    
    
    if lbg {
        let qba = ajw as i8;
        if qba != 0 {
            let _ = WV_.yqk(Ordering::Relaxed, Ordering::Relaxed, |aft| {
                Some(aft.akq(qba))
            });
        }
    }
}


pub fn drd() -> Bmr {
    Bmr {
        b: IN_.load(Ordering::Relaxed),
        c: IO_.load(Ordering::Relaxed),
        jda: UR_.load(Ordering::Relaxed),
        pdd: WT_.load(Ordering::Relaxed),
        uoh: AFN_.load(Ordering::Relaxed),
        jnw: WV_.swap(0, Ordering::Relaxed),
    }
}


pub fn tup(dx: i32, bg: i32, fd: bool, hw: bool, uog: bool, jc: i8) {
    let d = IW_.load(Ordering::Relaxed);
    let i = IV_.load(Ordering::Relaxed);

    
    let evh = IN_.load(Ordering::Relaxed) + dx;
    let bhn = IO_.load(Ordering::Relaxed) + bg;
    IN_.store(evh.am(0).v(d - 1), Ordering::Relaxed);
    IO_.store(bhn.am(0).v(i - 1), Ordering::Relaxed);

    UR_.store(fd, Ordering::Relaxed);
    WT_.store(hw, Ordering::Relaxed);
    AFN_.store(uog, Ordering::Relaxed);
    if jc != 0 {
        WV_.store(jc, Ordering::Relaxed);
    }
}


pub fn teo() -> i8 {
    WV_.swap(0, Ordering::Relaxed)
}


pub fn vtf() {
    
    let iu = crate::logger::lh();
    let qv = AYR_.load(Ordering::Relaxed);
    AYR_.store(iu, Ordering::Relaxed);
    
    let mut az = AAB_.lock();
    
    if iu - qv < 30 {
        *az = 2;
    } else {
        *az = 1;
    }
}


pub fn jbf() -> bool {
    let az = AAB_.lock();
    *az >= 2
}


pub fn pcp() {
    *AAB_.lock() = 0;
}


pub fn yto() -> (i32, i32) {
    (IN_.load(Ordering::Relaxed), IO_.load(Ordering::Relaxed))
}


pub fn yzn() -> bool {
    UR_.load(Ordering::Relaxed)
}


pub fn yzz() -> bool {
    WT_.load(Ordering::Relaxed)
}


pub fn ky() -> bool {
    
    
    ADI_.load(Ordering::Relaxed) || true  
}





static CDQ_: AtomicI32 = AtomicI32::new(640);
static CDR_: AtomicI32 = AtomicI32::new(400);



pub fn yta() -> Option<(i32, i32)> {
    let nic = IN_.load(Ordering::Relaxed);
    let nid = IO_.load(Ordering::Relaxed);
    let jcv = CDQ_.swap(nic, Ordering::Relaxed);
    let ucs = CDR_.swap(nid, Ordering::Relaxed);
    
    let dx = nic - jcv;
    let bg = nid - ucs;
    
    if dx != 0 || bg != 0 {
        Some((dx, bg))
    } else {
        None
    }
}
