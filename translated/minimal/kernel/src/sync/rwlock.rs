




use core::sync::atomic::{AtomicU32, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};


pub struct RwLock<T> {
    
    
    
    
    g: AtomicU32,
    f: UnsafeCell<T>,
}

const Ux: u32 = 0x8000_0000;
const AZZ_: u32 = Ux - 1;

unsafe impl<T: Send> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    pub const fn new(f: T) -> Self {
        Self {
            g: AtomicU32::new(0),
            f: UnsafeCell::new(f),
        }
    }
    
    
    pub fn read(&self) -> Yn<T> {
        loop {
            let g = self.g.load(Ordering::Relaxed);
            
            
            if g >= Ux {
                core::hint::hc();
                crate::scheduler::gxc();
                continue;
            }
            
            
            if g >= AZZ_ {
                
                core::hint::hc();
                crate::scheduler::gxc();
                continue;
            }
            
            if self.g.kka(
                g,
                g + 1,
                Ordering::Acquire,
                Ordering::Relaxed
            ).is_ok() {
                return Yn { lock: self };
            }
        }
    }
    
    
    pub fn mnf(&self) -> Option<Yn<T>> {
        let g = self.g.load(Ordering::Relaxed);
        
        if g >= Ux || g >= AZZ_ {
            return None;
        }
        
        if self.g.compare_exchange(
            g,
            g + 1,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(Yn { lock: self })
        } else {
            None
        }
    }
    
    
    pub fn write(&self) -> Ud<T> {
        loop {
            
            if self.g.kka(
                0,
                Ux,
                Ordering::Acquire,
                Ordering::Relaxed
            ).is_ok() {
                return Ud { lock: self };
            }
            
            core::hint::hc();
            crate::scheduler::gxc();
        }
    }
    
    
    pub fn ifb(&self) -> Option<Ud<T>> {
        if self.g.compare_exchange(
            0,
            Ux,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(Ud { lock: self })
        } else {
            None
        }
    }
    
    
    pub fn zin(&self) -> u32 {
        let g = self.g.load(Ordering::Relaxed);
        if g >= Ux { 0 } else { g }
    }
    
    
    pub fn zae(&self) -> bool {
        self.g.load(Ordering::Relaxed) >= Ux
    }
}

pub struct Yn<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for Yn<'_, T> {
    type Zb = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.f.get() }
    }
}

impl<T> Drop for Yn<'_, T> {
    fn drop(&mut self) {
        self.lock.g.fetch_sub(1, Ordering::Release);
    }
}

pub struct Ud<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for Ud<'_, T> {
    type Zb = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.f.get() }
    }
}

impl<T> DerefMut for Ud<'_, T> {
    fn kph(&mut self) -> &mut T {
        unsafe { &mut *self.lock.f.get() }
    }
}

impl<T> Drop for Ud<'_, T> {
    fn drop(&mut self) {
        self.lock.g.store(0, Ordering::Release);
    }
}


pub struct Aej<T> {
    hxc: AtomicU32,
    fyy: AtomicU32,
    f: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Aej<T> {}
unsafe impl<T: Send + Sync> Sync for Aej<T> {}

impl<T> Aej<T> {
    pub const fn new(f: T) -> Self {
        Self {
            hxc: AtomicU32::new(0),
            fyy: AtomicU32::new(0),
            f: UnsafeCell::new(f),
        }
    }
    
    pub fn read(&self) -> Amc<T> {
        loop {
            
            while self.fyy.load(Ordering::Relaxed) != 0 {
                core::hint::hc();
            }
            
            
            self.hxc.fetch_add(1, Ordering::Acquire);
            
            
            if self.fyy.load(Ordering::Relaxed) == 0 {
                return Amc { lock: self };
            }
            
            
            self.hxc.fetch_sub(1, Ordering::Release);
        }
    }
    
    pub fn write(&self) -> Aek<T> {
        
        while self.fyy.compare_exchange(
            0, 1,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() {
            core::hint::hc();
        }
        
        
        while self.hxc.load(Ordering::Relaxed) != 0 {
            core::hint::hc();
        }
        
        Aek { lock: self }
    }
}

pub struct Amc<'a, T> {
    lock: &'a Aej<T>,
}

impl<T> Deref for Amc<'_, T> {
    type Zb = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.f.get() }
    }
}

impl<T> Drop for Amc<'_, T> {
    fn drop(&mut self) {
        self.lock.hxc.fetch_sub(1, Ordering::Release);
    }
}

pub struct Aek<'a, T> {
    lock: &'a Aej<T>,
}

impl<T> Deref for Aek<'_, T> {
    type Zb = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.f.get() }
    }
}

impl<T> DerefMut for Aek<'_, T> {
    fn kph(&mut self) -> &mut T {
        unsafe { &mut *self.lock.f.get() }
    }
}

impl<T> Drop for Aek<'_, T> {
    fn drop(&mut self) {
        self.lock.fyy.store(0, Ordering::Release);
    }
}
