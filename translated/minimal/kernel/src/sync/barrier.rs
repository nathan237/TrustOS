



use core::sync::atomic::{AtomicU32, Ordering};


pub struct Byk {
    
    qy: u32,
    
    az: AtomicU32,
    
    iwm: AtomicU32,
}

impl Byk {
    
    pub const fn new(bo: u32) -> Self {
        Self {
            qy: bo,
            az: AtomicU32::new(0),
            iwm: AtomicU32::new(0),
        }
    }
    
    
    pub fn ccm(&self) -> bool {
        let tar = self.iwm.load(Ordering::Relaxed);
        let kbd = self.az.fetch_add(1, Ordering::AcqRel) + 1;
        
        if kbd == self.qy {
            
            self.az.store(0, Ordering::Relaxed);
            self.iwm.fetch_add(1, Ordering::Release);
            true 
        } else {
            
            while self.iwm.load(Ordering::Acquire) == tar {
                core::hint::hc();
            }
            false
        }
    }
    
    
    pub unsafe fn apa(&self) {
        self.az.store(0, Ordering::Relaxed);
    }
}


pub struct Cig {
    qy: u32,
    az: AtomicU32,
    hxl: AtomicU32,
}

impl Cig {
    pub const fn new(bo: u32) -> Self {
        Self {
            qy: bo,
            az: AtomicU32::new(0),
            hxl: AtomicU32::new(0),
        }
    }
    
    pub fn ccm(&self) {
        let kbd = self.az.fetch_add(1, Ordering::AcqRel) + 1;
        
        if kbd == self.qy {
            self.hxl.store(1, Ordering::Release);
        } else {
            while self.hxl.load(Ordering::Acquire) == 0 {
                core::hint::hc();
            }
        }
    }
    
    pub fn yzy(&self) -> bool {
        self.hxl.load(Ordering::Relaxed) != 0
    }
}
