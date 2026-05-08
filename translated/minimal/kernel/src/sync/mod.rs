




pub mod futex;
pub mod percpu;
pub mod rwlock;
pub mod semaphore;
pub mod barrier;

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};


pub struct Qk<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Qk<T> {}
unsafe impl<T: Send> Sync for Qk<T> {}

impl<T> Qk<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    
    #[inline]
    pub fn lock(&self) -> Ja<T> {
        let mut eab = 0u32;
        
        while self.locked.compare_exchange_weak(
            false, true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() {
            
            eab += 1;
            for _ in 0..(1 << eab.min(6)) {
                core::hint::spin_loop();
            }
        }
        
        Ja { lock: self }
    }
    
    #[inline]
    pub fn try_lock(&self) -> Option<Ja<T>> {
        if self.locked.compare_exchange(
            false, true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(Ja { lock: self })
        } else {
            None
        }
    }
    
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }
}

pub struct Ja<'a, T> {
    lock: &'a Qk<T>,
}

impl<T> Deref for Ja<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for Ja<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for Ja<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}


pub struct Qm<T> {
    next_ticket: AtomicU64,
    now_serving: AtomicU64,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Qm<T> {}
unsafe impl<T: Send> Sync for Qm<T> {}

impl<T> Qm<T> {
    pub const fn new(data: T) -> Self {
        Self {
            next_ticket: AtomicU64::new(0),
            now_serving: AtomicU64::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    pub fn lock(&self) -> Nm<T> {
        let pjh = self.next_ticket.fetch_add(1, Ordering::Relaxed);
        
        while self.now_serving.load(Ordering::Acquire) != pjh {
            core::hint::spin_loop();
        }
        
        Nm { lock: self }
    }
}

pub struct Nm<'a, T> {
    lock: &'a Qm<T>,
}

impl<T> Deref for Nm<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for Nm<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for Nm<'_, T> {
    fn drop(&mut self) {
        self.lock.now_serving.fetch_add(1, Ordering::Release);
    }
}


pub struct Once {
    state: AtomicU32,
}

const BDY_: u32 = 0;
const CLR_: u32 = 1;
const XF_: u32 = 2;

impl Once {
    pub const fn new() -> Self {
        Self { state: AtomicU32::new(BDY_) }
    }
    
    pub fn call_once<F: FnOnce()>(&self, f: F) {
        if self.state.load(Ordering::Acquire) == XF_ {
            return;
        }
        
        
        if self.state.compare_exchange(
            BDY_, CLR_,
            Ordering::AcqRel,
            Ordering::Acquire
        ).is_ok() {
            f();
            self.state.store(XF_, Ordering::Release);
        } else {
            
            while self.state.load(Ordering::Acquire) != XF_ {
                core::hint::spin_loop();
            }
        }
    }
    
    pub fn qmf(&self) -> bool {
        self.state.load(Ordering::Acquire) == XF_
    }
}


pub struct Ph<T, F = fn() -> T> {
    cell: UnsafeCell<Option<T>>,
    init: UnsafeCell<Option<F>>,
    once: Once,
}

unsafe impl<T: Send + Sync, F: Send> Send for Ph<T, F> {}
unsafe impl<T: Send + Sync, F: Send> Sync for Ph<T, F> {}

impl<T, F: FnOnce() -> T> Ph<T, F> {
    pub const fn new(init: F) -> Self {
        Self {
            cell: UnsafeCell::new(None),
            init: UnsafeCell::new(Some(init)),
            once: Once::new(),
        }
    }
    
    pub fn get(&self) -> &T {
        self.once.call_once(|| {
            let init = unsafe { (*self.init.get()).take().unwrap() };
            let value = init();
            unsafe { *self.cell.get() = Some(value) };
        });
        
        unsafe { (*self.cell.get()).as_ref().unwrap() }
    }
}

impl<T, F: FnOnce() -> T> Deref for Ph<T, F> {
    type Target = T;
    
    fn deref(&self) -> &T {
        self.get()
    }
}


pub struct Ahd {
    count: AtomicU32,
}

impl Ahd {
    pub const fn new(are: u32) -> Self {
        Self { count: AtomicU32::new(are) }
    }
    
    #[inline]
    pub fn bmy(&self) -> u32 {
        self.count.fetch_add(1, Ordering::Relaxed) + 1
    }
    
    #[inline]
    pub fn ox(&self) -> u32 {
        let prev = self.count.fetch_sub(1, Ordering::Release);
        if prev == 1 {
            core::sync::atomic::fence(Ordering::Acquire);
        }
        prev - 1
    }
    
    #[inline]
    pub fn get(&self) -> u32 {
        self.count.load(Ordering::Relaxed)
    }
}


pub struct Va<T> {
    seq: AtomicU64,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Va<T> {}
unsafe impl<T: Send + Sync> Sync for Va<T> {}

impl<T: Copy> Va<T> {
    pub const fn new(data: T) -> Self {
        Self {
            seq: AtomicU64::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    
    pub fn read(&self) -> T {
        loop {
            let jer = self.seq.load(Ordering::Acquire);
            if jer & 1 != 0 {
                
                core::hint::spin_loop();
                continue;
            }
            
            let value = unsafe { *self.data.get() };
            
            let ooh = self.seq.load(Ordering::Acquire);
            if jer == ooh {
                return value;
            }
            
            core::hint::spin_loop();
        }
    }
    
    
    pub fn write(&self, value: T) {
        
        self.seq.fetch_add(1, Ordering::Acquire);
        
        unsafe { *self.data.get() = value };
        
        
        self.seq.fetch_add(1, Ordering::Release);
    }
}
