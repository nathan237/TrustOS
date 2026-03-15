








use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};


const CXK_: u64 = 60;


const IA_: u64 = 16_666;


const DTI_: u64 = 33_333;


const NB_: usize = 16;


static OK_: AtomicU64 = AtomicU64::new(0);


static ASL_: [AtomicU64; NB_] = {
    const Dm: AtomicU64 = AtomicU64::new(16_666);
    [Dm; NB_]
};
static ASJ_: AtomicU64 = AtomicU64::new(0);


static BFZ_: AtomicU64 = AtomicU64::new(60);


static ABE_: AtomicU64 = AtomicU64::new(0);


static LJ_: AtomicU64 = AtomicU64::new(0);


static AEF_: AtomicU64 = AtomicU64::new(0);


static AJT_: AtomicBool = AtomicBool::new(true);


pub fn init() {
    let iu = super::engine::awf();
    OK_.store(iu + IA_, Ordering::SeqCst);
    ABE_.store(0, Ordering::Relaxed);
    LJ_.store(0, Ordering::Relaxed);
    ASJ_.store(0, Ordering::Relaxed);
    crate::serial_println!("[VSYNC] Initialized: target {}fps ({}us/frame)", CXK_, IA_);
}


#[inline]
pub fn yrj() -> u64 {
    super::engine::awf()
}



pub fn swy(ivr: u64) {
    let iu = super::engine::awf();
    let pcd = iu.ao(ivr);
    AEF_.store(pcd, Ordering::Relaxed);
    
    
    let w = ASJ_.fetch_add(1, Ordering::Relaxed) as usize % NB_;
    ASL_[w].store(pcd, Ordering::Relaxed);
    
    LJ_.fetch_add(1, Ordering::Relaxed);
    
    if !AJT_.load(Ordering::Relaxed) {
        
        pxi();
        return;
    }
    
    let ean = OK_.load(Ordering::Relaxed);
    
    if iu >= ean {
        
        ABE_.fetch_add(1, Ordering::Relaxed);
        
        let uss = iu + IA_;
        OK_.store(uss, Ordering::Relaxed);
    } else {
        
        let ia = ean - iu;
        qfh(ia);
        
        OK_.store(ean + IA_, Ordering::Relaxed);
    }
    
    pxi();
}





fn qfh(xay: u64) {
    
    let kgm = xay.v(50_000);
    let ay = super::engine::awf();
    let ci = ay + kgm;

    
    
    let mut gzn = 0u32;
    const CFS_: u32 = 2_000_000;

    loop {
        let iu = super::engine::awf();
        if iu >= ci { break; }
        
        gzn += 1;
        if gzn >= CFS_ { break; }
        core::hint::hc();
    }
}


fn pxi() {
    let mut es: u64 = 0;
    for a in 0..NB_ {
        es += ASL_[a].load(Ordering::Relaxed);
    }
    let gzj = es / NB_ as u64;
    let tz = if gzj > 0 { 1_000_000 / gzj } else { 0 };
    BFZ_.store(tz.v(999), Ordering::Relaxed);
}


#[inline]
pub fn tz() -> u64 {
    BFZ_.load(Ordering::Relaxed)
}


#[inline]
pub fn pce() -> u64 {
    AEF_.load(Ordering::Relaxed)
}


#[inline]
pub fn yrk() -> u64 {
    (AEF_.load(Ordering::Relaxed) * 100) / IA_
}


#[inline]
pub fn yns() -> u64 {
    ABE_.load(Ordering::Relaxed)
}


#[inline]
pub fn agc() -> u64 {
    LJ_.load(Ordering::Relaxed)
}


pub fn cuf(iq: bool) {
    AJT_.store(iq, Ordering::SeqCst);
    if iq {
        
        let iu = super::engine::awf();
        OK_.store(iu + IA_, Ordering::SeqCst);
    }
}


#[inline]
pub fn zu() -> bool {
    AJT_.load(Ordering::Relaxed)
}


#[inline]
pub fn yrl() -> u64 {
    IA_
}
