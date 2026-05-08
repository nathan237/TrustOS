




use core::sync::atomic::{AtomicU32, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};


pub struct RwLock<T> {
    
    
    
    
    state: AtomicU32,
    data: UnsafeCell<T>,
}

const Jf: u32 = 0x8000_0000;
const BCB_: u32 = Jf - 1;

unsafe impl<T: Send> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    
    pub fn read(&self) -> Kn<T> {
        loop {
            let state = self.state.load(Ordering::Relaxed);
            
            
            if state >= Jf {
                core::hint::spin_loop();
                crate::scheduler::dgw();
                continue;
            }
            
            
            if state >= BCB_ {
                
                core::hint::spin_loop();
                crate::scheduler::dgw();
                continue;
            }
            
            if self.state.compare_exchange_weak(
                state,
                state + 1,
                Ordering::Acquire,
                Ordering::Relaxed
            ).is_ok() {
                return Kn { lock: self };
            }
        }
    }
    
    
    pub fn try_read(&self) -> Option<Kn<T>> {
        let state = self.state.load(Ordering::Relaxed);
        
        if state >= Jf || state >= BCB_ {
            return None;
        }
        
        if self.state.compare_exchange(
            state,
            state + 1,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(Kn { lock: self })
        } else {
            None
        }
    }
    
    
    pub fn write(&self) -> Is<T> {
        loop {
            
            if self.state.compare_exchange_weak(
                0,
                Jf,
                Ordering::Acquire,
                Ordering::Relaxed
            ).is_ok() {
                return Is { lock: self };
            }
            
            core::hint::spin_loop();
            crate::scheduler::dgw();
        }
    }
    
    
    pub fn try_write(&self) -> Option<Is<T>> {
        if self.state.compare_exchange(
            0,
            Jf,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(Is { lock: self })
        } else {
            None
        }
    }
    
    
    pub fn qsz(&self) -> u32 {
        let state = self.state.load(Ordering::Relaxed);
        if state >= Jf { 0 } else { state }
    }
    
    
    pub fn qnc(&self) -> bool {
        self.state.load(Ordering::Relaxed) >= Jf
    }
}

pub struct Kn<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for Kn<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for Kn<'_, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

pub struct Is<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for Is<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for Is<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for Is<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release);
    }
}


pub struct Na<T> {
    readers: AtomicU32,
    writer: AtomicU32,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Na<T> {}
unsafe impl<T: Send + Sync> Sync for Na<T> {}

impl<T> Na<T> {
    pub const fn new(data: T) -> Self {
        Self {
            readers: AtomicU32::new(0),
            writer: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    pub fn read(&self) -> Qb<T> {
        loop {
            
            while self.writer.load(Ordering::Relaxed) != 0 {
                core::hint::spin_loop();
            }
            
            
            self.readers.fetch_add(1, Ordering::Acquire);
            
            
            if self.writer.load(Ordering::Relaxed) == 0 {
                return Qb { lock: self };
            }
            
            
            self.readers.fetch_sub(1, Ordering::Release);
        }
    }
    
    pub fn write(&self) -> Nb<T> {
        
        while self.writer.compare_exchange(
            0, 1,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() {
            core::hint::spin_loop();
        }
        
        
        while self.readers.load(Ordering::Relaxed) != 0 {
            core::hint::spin_loop();
        }
        
        Nb { lock: self }
    }
}

pub struct Qb<'a, T> {
    lock: &'a Na<T>,
}

impl<T> Deref for Qb<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for Qb<'_, T> {
    fn drop(&mut self) {
        self.lock.readers.fetch_sub(1, Ordering::Release);
    }
}

pub struct Nb<'a, T> {
    lock: &'a Na<T>,
}

impl<T> Deref for Nb<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for Nb<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for Nb<'_, T> {
    fn drop(&mut self) {
        self.lock.writer.store(0, Ordering::Release);
    }
}
