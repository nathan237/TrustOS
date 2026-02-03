//! Synchronization Primitives
//!
//! High-performance synchronization for kernel and userspace.
//! Inspired by Redox OS sync module but optimized for TrustOS.

pub mod futex;
pub mod percpu;
pub mod rwlock;
pub mod semaphore;
pub mod barrier;

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Spinlock with backoff
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    
    #[inline]
    pub fn lock(&self) -> SpinLockGuard<T> {
        let mut spin_count = 0u32;
        
        while self.locked.compare_exchange_weak(
            false, true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() {
            // Exponential backoff with spin hint
            spin_count += 1;
            for _ in 0..(1 << spin_count.min(6)) {
                core::hint::spin_loop();
            }
        }
        
        SpinLockGuard { lock: self }
    }
    
    #[inline]
    pub fn try_lock(&self) -> Option<SpinLockGuard<T>> {
        if self.locked.compare_exchange(
            false, true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(SpinLockGuard { lock: self })
        } else {
            None
        }
    }
    
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

/// Ticket lock for fair access
pub struct TicketLock<T> {
    next_ticket: AtomicU64,
    now_serving: AtomicU64,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for TicketLock<T> {}
unsafe impl<T: Send> Sync for TicketLock<T> {}

impl<T> TicketLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            next_ticket: AtomicU64::new(0),
            now_serving: AtomicU64::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    pub fn lock(&self) -> TicketLockGuard<T> {
        let ticket = self.next_ticket.fetch_add(1, Ordering::Relaxed);
        
        while self.now_serving.load(Ordering::Acquire) != ticket {
            core::hint::spin_loop();
        }
        
        TicketLockGuard { lock: self }
    }
}

pub struct TicketLockGuard<'a, T> {
    lock: &'a TicketLock<T>,
}

impl<T> Deref for TicketLockGuard<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for TicketLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for TicketLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.now_serving.fetch_add(1, Ordering::Release);
    }
}

/// Once initialization (like std::sync::Once)
pub struct Once {
    state: AtomicU32,
}

const ONCE_UNINIT: u32 = 0;
const ONCE_RUNNING: u32 = 1;
const ONCE_COMPLETE: u32 = 2;

impl Once {
    pub const fn new() -> Self {
        Self { state: AtomicU32::new(ONCE_UNINIT) }
    }
    
    pub fn call_once<F: FnOnce()>(&self, f: F) {
        if self.state.load(Ordering::Acquire) == ONCE_COMPLETE {
            return;
        }
        
        // Try to be the one to run
        if self.state.compare_exchange(
            ONCE_UNINIT, ONCE_RUNNING,
            Ordering::AcqRel,
            Ordering::Acquire
        ).is_ok() {
            f();
            self.state.store(ONCE_COMPLETE, Ordering::Release);
        } else {
            // Wait for completion
            while self.state.load(Ordering::Acquire) != ONCE_COMPLETE {
                core::hint::spin_loop();
            }
        }
    }
    
    pub fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == ONCE_COMPLETE
    }
}

/// Lazy initialization wrapper
pub struct Lazy<T, F = fn() -> T> {
    cell: UnsafeCell<Option<T>>,
    init: UnsafeCell<Option<F>>,
    once: Once,
}

unsafe impl<T: Send + Sync, F: Send> Send for Lazy<T, F> {}
unsafe impl<T: Send + Sync, F: Send> Sync for Lazy<T, F> {}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
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

impl<T, F: FnOnce() -> T> Deref for Lazy<T, F> {
    type Target = T;
    
    fn deref(&self) -> &T {
        self.get()
    }
}

/// Atomic reference counter
pub struct AtomicRefCount {
    count: AtomicU32,
}

impl AtomicRefCount {
    pub const fn new(initial: u32) -> Self {
        Self { count: AtomicU32::new(initial) }
    }
    
    #[inline]
    pub fn inc(&self) -> u32 {
        self.count.fetch_add(1, Ordering::Relaxed) + 1
    }
    
    #[inline]
    pub fn dec(&self) -> u32 {
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

/// Sequence lock for read-heavy workloads
pub struct SeqLock<T> {
    seq: AtomicU64,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for SeqLock<T> {}
unsafe impl<T: Send + Sync> Sync for SeqLock<T> {}

impl<T: Copy> SeqLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            seq: AtomicU64::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    /// Read value (may retry on concurrent write)
    pub fn read(&self) -> T {
        loop {
            let seq1 = self.seq.load(Ordering::Acquire);
            if seq1 & 1 != 0 {
                // Writer active, spin
                core::hint::spin_loop();
                continue;
            }
            
            let value = unsafe { *self.data.get() };
            
            let seq2 = self.seq.load(Ordering::Acquire);
            if seq1 == seq2 {
                return value;
            }
            // Value changed during read, retry
            core::hint::spin_loop();
        }
    }
    
    /// Write value
    pub fn write(&self, value: T) {
        // Increment to odd (write in progress)
        self.seq.fetch_add(1, Ordering::Acquire);
        
        unsafe { *self.data.get() = value };
        
        // Increment to even (write complete)
        self.seq.fetch_add(1, Ordering::Release);
    }
}
