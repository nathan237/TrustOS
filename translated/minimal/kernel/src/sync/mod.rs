




pub mod futex;
pub mod percpu;
pub mod rwlock;
pub mod semaphore;
pub mod barrier;

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};


pub struct Amw<T> {
    caq: AtomicBool,
    f: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Amw<T> {}
unsafe impl<T: Send> Sync for Amw<T> {}

impl<T> Amw<T> {
    pub const fn new(f: T) -> Self {
        Self {
            caq: AtomicBool::new(false),
            f: UnsafeCell::new(f),
        }
    }
    
    #[inline]
    pub fn lock(&self) -> Uo<T> {
        let mut ibf = 0u32;
        
        while self.caq.kka(
            false, true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() {
            
            ibf += 1;
            for _ in 0..(1 << ibf.v(6)) {
                core::hint::hc();
            }
        }
        
        Uo { lock: self }
    }
    
    #[inline]
    pub fn try_lock(&self) -> Option<Uo<T>> {
        if self.caq.compare_exchange(
            false, true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(Uo { lock: self })
        } else {
            None
        }
    }
    
    #[inline]
    pub fn ogg(&self) -> bool {
        self.caq.load(Ordering::Relaxed)
    }
}

pub struct Uo<'a, T> {
    lock: &'a Amw<T>,
}

impl<T> Deref for Uo<'_, T> {
    type Zb = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.f.get() }
    }
}

impl<T> DerefMut for Uo<'_, T> {
    fn kph(&mut self) -> &mut T {
        unsafe { &mut *self.lock.f.get() }
    }
}

impl<T> Drop for Uo<'_, T> {
    fn drop(&mut self) {
        self.lock.caq.store(false, Ordering::Release);
    }
}


pub struct Ann<T> {
    oqo: AtomicU64,
    lpa: AtomicU64,
    f: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Ann<T> {}
unsafe impl<T: Send> Sync for Ann<T> {}

impl<T> Ann<T> {
    pub const fn new(f: T) -> Self {
        Self {
            oqo: AtomicU64::new(0),
            lpa: AtomicU64::new(0),
            f: UnsafeCell::new(f),
        }
    }
    
    pub fn lock(&self) -> Aex<T> {
        let xgu = self.oqo.fetch_add(1, Ordering::Relaxed);
        
        while self.lpa.load(Ordering::Acquire) != xgu {
            core::hint::hc();
        }
        
        Aex { lock: self }
    }
}

pub struct Aex<'a, T> {
    lock: &'a Ann<T>,
}

impl<T> Deref for Aex<'_, T> {
    type Zb = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.f.get() }
    }
}

impl<T> DerefMut for Aex<'_, T> {
    fn kph(&mut self) -> &mut T {
        unsafe { &mut *self.lock.f.get() }
    }
}

impl<T> Drop for Aex<'_, T> {
    fn drop(&mut self) {
        self.lock.lpa.fetch_add(1, Ordering::Release);
    }
}


pub struct Once {
    g: AtomicU32,
}

const BBV_: u32 = 0;
const CII_: u32 = 1;
const VW_: u32 = 2;

impl Once {
    pub const fn new() -> Self {
        Self { g: AtomicU32::new(BBV_) }
    }
    
    pub fn nbm<G: FnOnce()>(&self, bb: G) {
        if self.g.load(Ordering::Acquire) == VW_ {
            return;
        }
        
        
        if self.g.compare_exchange(
            BBV_, CII_,
            Ordering::AcqRel,
            Ordering::Acquire
        ).is_ok() {
            bb();
            self.g.store(VW_, Ordering::Release);
        } else {
            
            while self.g.load(Ordering::Acquire) != VW_ {
                core::hint::hc();
            }
        }
    }
    
    pub fn yze(&self) -> bool {
        self.g.load(Ordering::Acquire) == VW_
    }
}


pub struct Ajq<T, G = fn() -> T> {
    cell: UnsafeCell<Option<T>>,
    init: UnsafeCell<Option<G>>,
    osm: Once,
}

unsafe impl<T: Send + Sync, G: Send> Send for Ajq<T, G> {}
unsafe impl<T: Send + Sync, G: Send> Sync for Ajq<T, G> {}

impl<T, G: FnOnce() -> T> Ajq<T, G> {
    pub const fn new(init: G) -> Self {
        Self {
            cell: UnsafeCell::new(None),
            init: UnsafeCell::new(Some(init)),
            osm: Once::new(),
        }
    }
    
    pub fn get(&self) -> &T {
        self.osm.nbm(|| {
            let init = unsafe { (*self.init.get()).take().unwrap() };
            let bn = init();
            unsafe { *self.cell.get() = Some(bn) };
        });
        
        unsafe { (*self.cell.get()).as_ref().unwrap() }
    }
}

impl<T, G: FnOnce() -> T> Deref for Ajq<T, G> {
    type Zb = T;
    
    fn deref(&self) -> &T {
        self.get()
    }
}


pub struct Bya {
    az: AtomicU32,
}

impl Bya {
    pub const fn new(cfo: u32) -> Self {
        Self { az: AtomicU32::new(cfo) }
    }
    
    #[inline]
    pub fn drz(&self) -> u32 {
        self.az.fetch_add(1, Ordering::Relaxed) + 1
    }
    
    #[inline]
    pub fn adr(&self) -> u32 {
        let vo = self.az.fetch_sub(1, Ordering::Release);
        if vo == 1 {
            core::sync::atomic::cxt(Ordering::Acquire);
        }
        vo - 1
    }
    
    #[inline]
    pub fn get(&self) -> u32 {
        self.az.load(Ordering::Relaxed)
    }
}


pub struct Ayp<T> {
    ls: AtomicU64,
    f: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Ayp<T> {}
unsafe impl<T: Send + Sync> Sync for Ayp<T> {}

impl<T: Copy> Ayp<T> {
    pub const fn new(f: T) -> Self {
        Self {
            ls: AtomicU64::new(0),
            f: UnsafeCell::new(f),
        }
    }
    
    
    pub fn read(&self) -> T {
        loop {
            let pic = self.ls.load(Ordering::Acquire);
            if pic & 1 != 0 {
                
                core::hint::hc();
                continue;
            }
            
            let bn = unsafe { *self.f.get() };
            
            let wht = self.ls.load(Ordering::Acquire);
            if pic == wht {
                return bn;
            }
            
            core::hint::hc();
        }
    }
    
    
    pub fn write(&self, bn: T) {
        
        self.ls.fetch_add(1, Ordering::Acquire);
        
        unsafe { *self.f.get() = bn };
        
        
        self.ls.fetch_add(1, Ordering::Release);
    }
}
