



use core::sync::atomic::{AtomicU32, Ordering};


pub struct Ahi {
    
    expected: u32,
    
    count: AtomicU32,
    
    generation: AtomicU32,
}

impl Ahi {
    
    pub const fn new(ae: u32) -> Self {
        Self {
            expected: ae,
            count: AtomicU32::new(0),
            generation: AtomicU32::new(0),
        }
    }
    
    
    pub fn bqb(&self) -> bool {
        let mbp = self.generation.load(Ordering::Relaxed);
        let fhn = self.count.fetch_add(1, Ordering::AcqRel) + 1;
        
        if fhn == self.expected {
            
            self.count.store(0, Ordering::Relaxed);
            self.generation.fetch_add(1, Ordering::Release);
            true 
        } else {
            
            while self.generation.load(Ordering::Acquire) == mbp {
                core::hint::spin_loop();
            }
            false
        }
    }
    
    
    pub unsafe fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
    }
}


pub struct Ana {
    expected: u32,
    count: AtomicU32,
    released: AtomicU32,
}

impl Ana {
    pub const fn new(ae: u32) -> Self {
        Self {
            expected: ae,
            count: AtomicU32::new(0),
            released: AtomicU32::new(0),
        }
    }
    
    pub fn bqb(&self) {
        let fhn = self.count.fetch_add(1, Ordering::AcqRel) + 1;
        
        if fhn == self.expected {
            self.released.store(1, Ordering::Release);
        } else {
            while self.released.load(Ordering::Acquire) == 0 {
                core::hint::spin_loop();
            }
        }
    }
    
    pub fn qmw(&self) -> bool {
        self.released.load(Ordering::Relaxed) != 0
    }
}
