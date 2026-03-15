




pub mod heap;
pub mod frame;
pub mod paging;
pub mod cow;
pub mod swap;
pub mod vma;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use spin::Mutex;

pub use paging::{AddressSpace, PageFlags, sw, aov, txu};





const AFD_: usize = 64;

struct BootMemoryMap {
    ch: [(u64, u64, u8); AFD_],
    az: usize,
}

impl BootMemoryMap {
    const fn new() -> Self {
        Self { ch: [(0, 0, 0); AFD_], az: 0 }
    }
    fn push(&mut self, ar: u64, go: u64, ifl: u8) {
        if self.az < AFD_ {
            self.ch[self.az] = (ar, go, ifl);
            self.az += 1;
        }
    }
    fn gai(&self) -> &[(u64, u64, u8)] {
        &self.ch[..self.az]
    }
}

static ALW_: Mutex<BootMemoryMap> = Mutex::new(BootMemoryMap::new());


pub fn wuv(ar: u64, go: u64, ifl: u8) {
    if let Some(mut map) = ALW_.try_lock() {
        map.push(ar, go, ifl);
    }
}


pub fn tdz() -> Vec<(u64, u64, u8)> {
    ALW_.lock().gai().ip()
}


static CF_: AtomicUsize = AtomicUsize::new(0);

static DS_: AtomicU64 = AtomicU64::new(0xFFFF_8000_0000_0000);

pub const TW_: usize = 64 * 1024 * 1024;

pub const BZB_: usize = 512 * 1024 * 1024;


static TV_: AtomicUsize = AtomicUsize::new(64 * 1024 * 1024);


pub fn cre() -> usize {
    TV_.load(Ordering::Relaxed)
}


#[deprecated(jp = "Use memory::heap_size() instead")]
pub const DPD_: usize = 64 * 1024 * 1024;


static BHN_: AtomicU64 = AtomicU64::new(0);


pub fn wjt(bf: u64) {
    BHN_.store(bf, Ordering::SeqCst);
}


pub fn fxc() -> u64 {
    BHN_.load(Ordering::Relaxed)
}


pub fn rnh(xko: u64) -> usize {
    let exd = (xko / 4) as usize;
    exd.qp(TW_, BZB_)
}


pub fn lej(lr: u64, moj: u64, giz: usize) {
    
    DS_.store(lr, Ordering::SeqCst);
    
    
    TV_.store(giz, Ordering::SeqCst);
    
    
    let dhz = moj;
    let lca = lr + dhz;
    
    CF_.store(lca as usize, Ordering::SeqCst);
    
    
    heap::lec(lca as usize, giz);
    crate::log!("Heap initialized: {} MB at virt {:#x} (phys {:#x})", 
        giz / 1024 / 1024, lca, dhz);
}


pub fn yxv(lr: u64, moj: u64) {
    lej(lr, moj, TW_);
}


pub fn init() {
    
    
    let lby = 0xFFFF_8000_0100_0000usize; 
    CF_.store(lby, Ordering::SeqCst);
    let aw = TW_;
    TV_.store(aw, Ordering::SeqCst);
    heap::lec(lby, aw);
    crate::log!("Heap initialized (fallback): {} MB at {:#x}", aw / 1024 / 1024, lby);
}



pub fn tta(ovk: u64, giz: usize) {
    
    DS_.store(0, Ordering::SeqCst);
    TV_.store(giz, Ordering::SeqCst);
    CF_.store(ovk as usize, Ordering::SeqCst);
    
    heap::lec(ovk as usize, giz);
}


pub fn cm() -> Bme {
    let (sxa, ceu) = frame::cm();
    Bme {
        afa: heap::mr(),
        buv: heap::aez(),
        ceu: ceu as usize,
        dhj: (sxa - ceu) as usize,
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Bme {
    pub afa: usize,
    pub buv: usize,
    pub ceu: usize,
    pub dhj: usize,
}


pub fn lr() -> u64 {
    DS_.load(Ordering::Relaxed)
}


pub fn auv(ht: u64) -> u64 {
    lr() + ht
}


pub fn abw(ju: u64) -> Option<u64> {
    let hp = lr();
    if ju >= hp {
        Some(ju - hp)
    } else {
        None
    }
}






pub fn bki(ki: u64, aw: usize) -> Result<u64, &'static str> {
    
    
    
    
    let vd = auv(ki);
    
    
    let aus = 4096u64;
    let eiz = ki & !0xFFF;
    let ktm = (ki + aw as u64 + 0xFFF) & !0xFFF;
    let dtt = ((ktm - eiz) / aus) as usize;
    
    crate::serial_println!("[MMIO] Mapping {:#x} -> {:#x} ({} pages)", 
        ki, vd, dtt);
    
    
    for a in 0..dtt {
        let dai = eiz + (a as u64 * aus);
        let egd = auv(dai);
        
        paging::oky(egd, dai)?;
    }
    
    
    for a in 0..dtt {
        let egd = auv(eiz + (a as u64 * aus));
        unsafe {
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("invlpg [{}]", in(reg) egd, options(nostack, preserves_flags));
            #[cfg(not(target_arch = "x86_64"))]
            crate::arch::ghg(egd);
        }
    }
    
    Ok(vd)
}


pub fn zuc(ydt: u64, dds: usize) {
    
}



pub fn vst(qdj: u32, ag: u64) -> Result<u64, i32> {
    
    
    if !aov(ag) {
        return Err(-14); 
    }
    
    
    let bn = unsafe { core::ptr::read_volatile(ag as *const u64) };
    Ok(bn)
}



pub fn xvu(qdj: u32, ag: u64, bn: u64) -> Result<(), i32> {
    
    
    if !aov(ag) {
        return Err(-14); 
    }
    
    
    unsafe { core::ptr::write_volatile(ag as *mut u64, bn) };
    Ok(())
}
