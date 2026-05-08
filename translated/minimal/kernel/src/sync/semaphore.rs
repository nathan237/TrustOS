



use core::sync::atomic::{AtomicI32, Ordering};


pub struct Qg {
    count: AtomicI32,
}

impl Qg {
    
    pub const fn new(count: i32) -> Self {
        Self { count: AtomicI32::new(count) }
    }
    
    
    pub fn pxo(&self) {
        loop {
            let count = self.count.load(Ordering::Relaxed);
            if count <= 0 {
                core::hint::spin_loop();
                continue;
            }
            
            if self.count.compare_exchange_weak(
                count,
                count - 1,
                Ordering::Acquire,
                Ordering::Relaxed
            ).is_ok() {
                return;
            }
        }
    }
    
    
    pub fn rba(&self) -> bool {
        let count = self.count.load(Ordering::Relaxed);
        if count <= 0 {
            return false;
        }
        
        self.count.compare_exchange(
            count,
            count - 1,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok()
    }
    
    
    pub fn release(&self) {
        self.count.fetch_add(1, Ordering::Release);
    }
    
    
    pub fn count(&self) -> i32 {
        self.count.load(Ordering::Relaxed)
    }
}


pub type Ahk = Qg;

impl Ahk {
    pub const fn qpj() -> Self {
        Self::new(1)
    }
}
