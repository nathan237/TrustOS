




pub mod heap;
pub mod frame;
pub mod paging;
pub mod cow;
pub mod swap;
pub mod vma;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use spin::Mutex;

pub use paging::{AddressSpace, PageFlags, ij, ux, msu};





const AGX_: usize = 64;

struct BootMemoryMap {
    entries: [(u64, u64, u8); AGX_],
    count: usize,
}

impl BootMemoryMap {
    const fn new() -> Self {
        Self { entries: [(0, 0, 0); AGX_], count: 0 }
    }
    fn push(&mut self, base: u64, length: u64, ecy: u8) {
        if self.count < AGX_ {
            self.entries[self.count] = (base, length, ecy);
            self.count += 1;
        }
    }
    fn as_slice(&self) -> &[(u64, u64, u8)] {
        &self.entries[..self.count]
    }
}

static ANR_: Mutex<BootMemoryMap> = Mutex::new(BootMemoryMap::new());


pub fn oxu(base: u64, length: u64, ecy: u8) {
    if let Some(mut map) = ANR_.try_lock() {
        map.push(base, length, ecy);
    }
}


pub fn mdl() -> Vec<(u64, u64, u8)> {
    ANR_.lock().as_slice().to_vec()
}


static CH_: AtomicUsize = AtomicUsize::new(0);

static ED_: AtomicU64 = AtomicU64::new(0xFFFF_8000_0000_0000);

pub const VE_: usize = 64 * 1024 * 1024;

pub const CCM_: usize = 512 * 1024 * 1024;


static VD_: AtomicUsize = AtomicUsize::new(64 * 1024 * 1024);


pub fn atz() -> usize {
    VD_.load(Ordering::Relaxed)
}


#[deprecated(note = "Use memory::heap_size() instead")]
pub const DSX_: usize = 64 * 1024 * 1024;


static BJR_: AtomicU64 = AtomicU64::new(0);


pub fn opo(bytes: u64) {
    BJR_.store(bytes, Ordering::SeqCst);
}


pub fn ceo() -> u64 {
    BJR_.load(Ordering::Relaxed)
}


pub fn kwm(total_ram: u64) -> usize {
    let ccz = (total_ram / 4) as usize;
    ccz.clamp(VE_, CCM_)
}


pub fn gcr(hhdm_offset: u64, usable_base: u64, heap_bytes: usize) {
    
    ED_.store(hhdm_offset, Ordering::SeqCst);
    
    
    VD_.store(heap_bytes, Ordering::SeqCst);
    
    
    let bgx = usable_base;
    let gaq = hhdm_offset + bgx;
    
    CH_.store(gaq as usize, Ordering::SeqCst);
    
    
    heap::gcm(gaq as usize, heap_bytes);
    crate::log!("Heap initialized: {} MB at virt {:#x} (phys {:#x})", 
        heap_bytes / 1024 / 1024, gaq, bgx);
}


pub fn qlg(hhdm_offset: u64, usable_base: u64) {
    gcr(hhdm_offset, usable_base, VE_);
}


pub fn init() {
    
    
    let gan = 0xFFFF_8000_0100_0000usize; 
    CH_.store(gan, Ordering::SeqCst);
    let size = VE_;
    VD_.store(size, Ordering::SeqCst);
    heap::gcm(gan, size);
    crate::log!("Heap initialized (fallback): {} MB at {:#x}", size / 1024 / 1024, gan);
}



pub fn mov(phys_base: u64, heap_bytes: usize) {
    
    ED_.store(0, Ordering::SeqCst);
    VD_.store(heap_bytes, Ordering::SeqCst);
    CH_.store(phys_base as usize, Ordering::SeqCst);
    
    heap::gcm(phys_base as usize, heap_bytes);
}


pub fn stats() -> Abi {
    let (frames_total, frames_used) = frame::stats();
    Abi {
        heap_used: heap::used(),
        heap_free: heap::free(),
        frames_used: frames_used as usize,
        frames_free: (frames_total - frames_used) as usize,
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Abi {
    pub heap_used: usize,
    pub heap_free: usize,
    pub frames_used: usize,
    pub frames_free: usize,
}


pub fn hhdm_offset() -> u64 {
    ED_.load(Ordering::Relaxed)
}


pub fn wk(phys: u64) -> u64 {
    hhdm_offset() + phys
}


pub fn lc(virt: u64) -> Option<u64> {
    let bz = hhdm_offset();
    if virt >= bz {
        Some(virt - bz)
    } else {
        None
    }
}






pub fn yv(phys_addr: u64, size: usize) -> Result<u64, &'static str> {
    
    
    
    
    let virt_addr = wk(phys_addr);
    
    
    let xy = 4096u64;
    let bvy = phys_addr & !0xFFF;
    let fuu = (phys_addr + size as u64 + 0xFFF) & !0xFFF;
    let bnw = ((fuu - bvy) / xy) as usize;
    
    crate::serial_println!("[MMIO] Mapping {:#x} -> {:#x} ({} pages)", 
        phys_addr, virt_addr, bnw);
    
    
    for i in 0..bnw {
        let bcy = bvy + (i as u64 * xy);
        let page_virt = wk(bcy);
        
        paging::ilu(page_virt, bcy)?;
    }
    
    
    for i in 0..bnw {
        let page_virt = wk(bvy + (i as u64 * xy));
        unsafe {
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("invlpg [{}]", in(reg) page_virt, options(nostack, preserves_flags));
            #[cfg(not(target_arch = "x86_64"))]
            crate::arch::cxy(page_virt);
        }
    }
    
    Ok(virt_addr)
}


pub fn rbi(_virt_addr: u64, bek: usize) {
    
}



pub fn odj(_pid: u32, addr: u64) -> Result<u64, i32> {
    
    
    if !ux(addr) {
        return Err(-14); 
    }
    
    
    let value = unsafe { core::ptr::read_volatile(addr as *const u64) };
    Ok(value)
}



pub fn pvf(_pid: u32, addr: u64, value: u64) -> Result<(), i32> {
    
    
    if !ux(addr) {
        return Err(-14); 
    }
    
    
    unsafe { core::ptr::write_volatile(addr as *mut u64, value) };
    Ok(())
}
