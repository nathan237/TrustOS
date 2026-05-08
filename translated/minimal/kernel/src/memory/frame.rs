





use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use alloc::vec;
use alloc::vec::Vec;


const CS_: u64 = 4096;


static UN_: Mutex<Option<Sj>> = Mutex::new(None);


static MD_: AtomicU64 = AtomicU64::new(0);

static RG_: AtomicU64 = AtomicU64::new(0);


struct Sj {
    
    bitmap: Vec<u64>,
    
    base_phys: u64,
    
    total_frames: usize,
    
    next_hint: usize,
}

impl Sj {
    
    fn alloc(&mut self) -> Option<u64> {
        let um = self.bitmap.len();
        
        
        for offset in 0..um {
            let idx = (self.next_hint + offset) % um;
            let fx = self.bitmap[idx];
            
            if fx == u64::MAX {
                continue; 
            }
            
            
            let bf = (!fx).trailing_zeros() as usize;
            let bss = idx * 64 + bf;
            
            if bss >= self.total_frames {
                continue;
            }
            
            
            self.bitmap[idx] |= 1u64 << bf;
            self.next_hint = idx;
            
            RG_.fetch_add(1, Ordering::Relaxed);
            
            return Some(self.base_phys + bss as u64 * CS_);
        }
        
        None 
    }
    
    
    
    fn alloc_below(&mut self, jm: u64) -> Option<u64> {
        let ncx = if jm <= self.base_phys {
            return None;
        } else {
            ((jm - self.base_phys) / CS_) as usize
        };
        let cap = ncx.min(self.total_frames);
        let um = (cap + 63) / 64;
        
        for idx in 0..um {
            let fx = self.bitmap[idx];
            if fx == u64::MAX {
                continue;
            }
            let bf = (!fx).trailing_zeros() as usize;
            let bss = idx * 64 + bf;
            if bss >= cap {
                continue;
            }
            self.bitmap[idx] |= 1u64 << bf;
            RG_.fetch_add(1, Ordering::Relaxed);
            return Some(self.base_phys + bss as u64 * CS_);
        }
        None
    }
    
    
    fn free(&mut self, phys: u64) {
        if phys < self.base_phys {
            return;
        }
        let bss = ((phys - self.base_phys) / CS_) as usize;
        if bss >= self.total_frames {
            return;
        }
        let jrg = bss / 64;
        let bew = bss % 64;
        
        if self.bitmap[jrg] & (1u64 << bew) != 0 {
            self.bitmap[jrg] &= !(1u64 << bew);
            RG_.fetch_sub(1, Ordering::Relaxed);
        }
    }
}


pub struct Mw {
    pub base: u64,
    pub length: u64,
}





pub fn init(cew: &[Mw], bgx: u64, atz: u64) {
    if cew.is_empty() {
        crate::serial_println!("[FRAME] No usable regions — frame allocator disabled");
        return;
    }
    
    
    let nfj = match cew.iter().map(|r| r.base).min() {
        Some(v) => v,
        None => { crate::serial_println!("[FRAME] BUG: no min in non-empty regions"); return; }
    };
    let ndf = match cew.iter().map(|r| r.base + r.length).max() {
        Some(v) => v,
        None => { crate::serial_println!("[FRAME] BUG: no max in non-empty regions"); return; }
    };
    
    
    let base_phys = nfj & !(CS_ - 1);
    let eca = (ndf + CS_ - 1) & !(CS_ - 1);
    let total_frames = ((eca - base_phys) / CS_) as usize;
    
    
    let kca = (total_frames + 63) / 64;
    let mut bitmap = vec![u64::MAX; kca];
    
    
    for qd in cew {
        let gqx = (qd.base.max(base_phys) - base_phys) / CS_;
        let cdf = ((qd.base + qd.length).min(eca) - base_phys) / CS_;
        
        for frame in gqx..cdf {
            let fx = frame as usize / 64;
            let bf = frame as usize % 64;
            bitmap[fx] &= !(1u64 << bf);
        }
    }
    
    
    let heap_end = bgx + atz;
    if bgx >= base_phys && bgx < eca {
        let owg = ((bgx - base_phys) / CS_) as usize;
        let civ = (((heap_end.min(eca)) - base_phys) / CS_) as usize;
        for frame in owg..civ {
            let fx = frame / 64;
            let bf = frame % 64;
            bitmap[fx] |= 1u64 << bf;
        }
    }
    
    
    
    if base_phys < 0x10_0000 {
        let nax = (0x10_0000u64.min(eca) - base_phys) / CS_;
        for frame in 0..nax as usize {
            let fx = frame / 64;
            let bf = frame % 64;
            bitmap[fx] |= 1u64 << bf;
        }
    }
    
    
    let mut enj: u64 = 0;
    for i in 0..total_frames {
        let fx = i / 64;
        let bf = i % 64;
        if bitmap[fx] & (1u64 << bf) == 0 {
            enj += 1;
        }
    }
    let jpo = total_frames as u64 - enj;
    
    MD_.store(total_frames as u64, Ordering::SeqCst);
    RG_.store(jpo, Ordering::SeqCst);
    
    crate::serial_println!("[FRAME] Allocator ready: {} total frames, {} free ({} MB), {} used",
        total_frames, enj, enj * 4 / 1024, jpo);
    
    *UN_.lock() = Some(Sj {
        bitmap,
        base_phys,
        total_frames,
        next_hint: 0,
    });
}



pub fn cfv() -> Option<u64> {
    UN_.lock().as_mut()?.alloc()
}


pub fn vk(phys: u64) {
    if let Some(alloc) = UN_.lock().as_mut() {
        alloc.free(phys);
    }
}


pub fn aan() -> Option<u64> {
    let phys = cfv()?;
    let bz = crate::memory::hhdm_offset();
    let virt = phys + bz;
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    unsafe {
        core::ptr::write_bytes(virt as *mut u8, 0, CS_ as usize);
    }
    Some(phys)
}



pub fn juv() -> Option<u64> {
    UN_.lock().as_mut()?.alloc_below(0x1_0000_0000)
}


pub fn pxz() -> Option<u64> {
    let phys = juv()?;
    let bz = crate::memory::hhdm_offset();
    let virt = phys + bz;
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    unsafe {
        core::ptr::write_bytes(virt as *mut u8, 0, CS_ as usize);
    }
    Some(phys)
}


pub fn stats() -> (u64, u64) {
    (MD_.load(Ordering::Relaxed), RG_.load(Ordering::Relaxed))
}


pub fn cdp() -> (usize, usize) {
    let mut passed = 0usize;
    let mut bv = 0usize;

    
    match cfv() {
        Some(phys) => {
            if phys & 0xFFF == 0 {
                crate::serial_println!("[FRAME-TEST] alloc page-aligned: PASS");
                passed += 1;
            } else {
                crate::serial_println!("[FRAME-TEST] alloc NOT page-aligned ({:#x}): FAIL", phys);
                bv += 1;
            }
            vk(phys);
        }
        None => {
            crate::serial_println!("[FRAME-TEST] alloc returned None: FAIL");
            bv += 1;
        }
    }

    
    match aan() {
        Some(phys) => {
            let bz = crate::memory::hhdm_offset();
            let za = unsafe { core::slice::from_raw_parts((phys + bz) as *const u8, 4096) };
            if za.iter().all(|&b| b == 0) {
                crate::serial_println!("[FRAME-TEST] alloc_zeroed all zeros: PASS");
                passed += 1;
            } else {
                crate::serial_println!("[FRAME-TEST] alloc_zeroed NOT zeroed: FAIL");
                bv += 1;
            }
            vk(phys);
        }
        None => {
            crate::serial_println!("[FRAME-TEST] alloc_zeroed returned None: FAIL");
            bv += 1;
        }
    }

    
    if let Some(frame1) = cfv() {
        vk(frame1);
        if cfv().is_some() {
            crate::serial_println!("[FRAME-TEST] free + realloc: PASS");
            passed += 1;
            
        } else {
            crate::serial_println!("[FRAME-TEST] realloc after free: FAIL");
            bv += 1;
        }
    }

    
    let mut frames = alloc::vec::Vec::new();
    let mut gyi = true;
    for _ in 0..16 {
        match cfv() {
            Some(f) => {
                if frames.contains(&f) {
                    crate::serial_println!("[FRAME-TEST] duplicate frame {:#x}: FAIL", f);
                    gyi = false;
                    break;
                }
                frames.push(f);
            }
            None => {
                crate::serial_println!("[FRAME-TEST] OOM during multi-alloc: FAIL");
                gyi = false;
                break;
            }
        }
    }
    for f in &frames {
        vk(*f);
    }
    if gyi {
        crate::serial_println!("[FRAME-TEST] 16 unique frames: PASS");
        passed += 1;
    } else {
        bv += 1;
    }

    
    let (_, used_before) = stats();
    if let Some(f) = cfv() {
        let (_, used_after) = stats();
        if used_after == used_before + 1 {
            crate::serial_println!("[FRAME-TEST] stats consistent: PASS");
            passed += 1;
        } else {
            crate::serial_println!("[FRAME-TEST] stats before={} after={}: FAIL", used_before, used_after);
            bv += 1;
        }
        vk(f);
    } else {
        crate::serial_println!("[FRAME-TEST] stats test alloc failed: FAIL");
        bv += 1;
    }

    (passed, bv)
}
