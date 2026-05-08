








use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};


const DBC_: u64 = 60;


const IU_: u64 = 16_666;


const DXA_: u64 = 33_333;


const OA_: usize = 16;


static PI_: AtomicU64 = AtomicU64::new(0);


static AUP_: [AtomicU64; OA_] = {
    const Bm: AtomicU64 = AtomicU64::new(16_666);
    [Bm; OA_]
};
static AUN_: AtomicU64 = AtomicU64::new(0);


static BID_: AtomicU64 = AtomicU64::new(60);


static ACU_: AtomicU64 = AtomicU64::new(0);


static MD_: AtomicU64 = AtomicU64::new(0);


static AFZ_: AtomicU64 = AtomicU64::new(0);


static ALO_: AtomicBool = AtomicBool::new(true);


pub fn init() {
    let cy = super::engine::yy();
    PI_.store(cy + IU_, Ordering::SeqCst);
    ACU_.store(0, Ordering::Relaxed);
    MD_.store(0, Ordering::Relaxed);
    AUN_.store(0, Ordering::Relaxed);
    crate::serial_println!("[VSYNC] Initialized: target {}fps ({}us/frame)", DBC_, IU_);
}


#[inline]
pub fn qgf() -> u64 {
    super::engine::yy()
}



pub fn lym(frame_start_us: u64) {
    let cy = super::engine::yy();
    let izy = cy.saturating_sub(frame_start_us);
    AFZ_.store(izy, Ordering::Relaxed);
    
    
    let idx = AUN_.fetch_add(1, Ordering::Relaxed) as usize % OA_;
    AUP_[idx].store(izy, Ordering::Relaxed);
    
    MD_.fetch_add(1, Ordering::Relaxed);
    
    if !ALO_.load(Ordering::Relaxed) {
        
        jph();
        return;
    }
    
    let brq = PI_.load(Ordering::Relaxed);
    
    if cy >= brq {
        
        ACU_.fetch_add(1, Ordering::Relaxed);
        
        let nix = cy + IU_;
        PI_.store(nix, Ordering::Relaxed);
    } else {
        
        let ck = brq - cy;
        jtu(ck);
        
        PI_.store(brq + IU_, Ordering::Relaxed);
    }
    
    jph();
}





fn jtu(target_us: u64) {
    
    let fkv = target_us.min(50_000);
    let start = super::engine::yy();
    let end = start + fkv;

    
    
    let mut dif = 0u32;
    const CJC_: u32 = 2_000_000;

    loop {
        let cy = super::engine::yy();
        if cy >= end { break; }
        
        dif += 1;
        if dif >= CJC_ { break; }
        core::hint::spin_loop();
    }
}


fn jph() {
    let mut av: u64 = 0;
    for i in 0..OA_ {
        av += AUP_[i].load(Ordering::Relaxed);
    }
    let dic = av / OA_ as u64;
    let fps = if dic > 0 { 1_000_000 / dic } else { 0 };
    BID_.store(fps.min(999), Ordering::Relaxed);
}


#[inline]
pub fn fps() -> u64 {
    BID_.load(Ordering::Relaxed)
}


#[inline]
pub fn izz() -> u64 {
    AFZ_.load(Ordering::Relaxed)
}


#[inline]
pub fn qgg() -> u64 {
    (AFZ_.load(Ordering::Relaxed) * 100) / IU_
}


#[inline]
pub fn qef() -> u64 {
    ACU_.load(Ordering::Relaxed)
}


#[inline]
pub fn total_frames() -> u64 {
    MD_.load(Ordering::Relaxed)
}


pub fn set_enabled(enabled: bool) {
    ALO_.store(enabled, Ordering::SeqCst);
    if enabled {
        
        let cy = super::engine::yy();
        PI_.store(cy + IU_, Ordering::SeqCst);
    }
}


#[inline]
pub fn lq() -> bool {
    ALO_.load(Ordering::Relaxed)
}


#[inline]
pub fn qgh() -> u64 {
    IU_
}
