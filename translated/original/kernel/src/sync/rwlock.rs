//! Read-Write Lock
//!
//! Allows multiple readers or single writer.
//! Optimized for read-heavy workloads.

use core::sync::atomic::{AtomicU32, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Reader-Writer lock with writer preference
pub struct RwLock<T> {
    /// Lock state: 
    /// - 0 = unlocked
    /// - WRITER = writer holds lock
    /// - 1..WRITER-1 = number of readers
    state: AtomicU32,
    data: UnsafeCell<T>,
}

const WRITER: u32 = 0x8000_0000;
const MAX_READERS: u32 = WRITER - 1;

unsafe impl<T: Send> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    /// Acquire read lock
    pub fn read(&self) -> RwLockReadGuard<T> {
        loop {
            let state = self.state.load(Ordering::Relaxed);
            
            // Can't read if writer holds lock
            if state >= WRITER {
                core::hint::spin_loop();
                crate::scheduler::yield_now();
                continue;
            }
            
            // Try to increment reader count
            if state >= MAX_READERS {
                // Too many readers (shouldn't happen)
                core::hint::spin_loop();
                crate::scheduler::yield_now();
                continue;
            }
            
            if self.state.compare_exchange_weak(
                state,
                state + 1,
                Ordering::Acquire,
                Ordering::Relaxed
            ).is_ok() {
                return RwLockReadGuard { lock: self };
            }
        }
    }
    
    /// Try to acquire read lock
    pub fn try_read(&self) -> Option<RwLockReadGuard<T>> {
        let state = self.state.load(Ordering::Relaxed);
        
        if state >= WRITER || state >= MAX_READERS {
            return None;
        }
        
        if self.state.compare_exchange(
            state,
            state + 1,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(RwLockReadGuard { lock: self })
        } else {
            None
        }
    }
    
    /// Acquire write lock
    pub fn write(&self) -> RwLockWriteGuard<T> {
        loop {
            // Try to set writer bit
            if self.state.compare_exchange_weak(
                0,
                WRITER,
                Ordering::Acquire,
                Ordering::Relaxed
            ).is_ok() {
                return RwLockWriteGuard { lock: self };
            }
            
            core::hint::spin_loop();
            crate::scheduler::yield_now();
        }
    }
    
    /// Try to acquire write lock
    pub fn try_write(&self) -> Option<RwLockWriteGuard<T>> {
        if self.state.compare_exchange(
            0,
            WRITER,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(RwLockWriteGuard { lock: self })
        } else {
            None
        }
    }
    
    /// Get number of readers
    pub fn reader_count(&self) -> u32 {
        let state = self.state.load(Ordering::Relaxed);
        if state >= WRITER { 0 } else { state }
    }
    
    /// Check if writer holds lock
    pub fn is_write_locked(&self) -> bool {
        self.state.load(Ordering::Relaxed) >= WRITER
    }
}

pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

pub struct RwLockWriteGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release);
    }
}

/// Read-Write lock with reader preference (good for mostly-read data)
pub struct RwSpinLock<T> {
    readers: AtomicU32,
    writer: AtomicU32,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for RwSpinLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwSpinLock<T> {}

impl<T> RwSpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            readers: AtomicU32::new(0),
            writer: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    pub fn read(&self) -> RwSpinReadGuard<T> {
        loop {
            // Wait for no writer
            while self.writer.load(Ordering::Relaxed) != 0 {
                core::hint::spin_loop();
            }
            
            // Try to add reader
            self.readers.fetch_add(1, Ordering::Acquire);
            
            // Check if writer snuck in
            if self.writer.load(Ordering::Relaxed) == 0 {
                return RwSpinReadGuard { lock: self };
            }
            
            // Writer won, back off
            self.readers.fetch_sub(1, Ordering::Release);
        }
    }
    
    pub fn write(&self) -> RwSpinWriteGuard<T> {
        // Acquire writer lock
        while self.writer.compare_exchange(
            0, 1,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() {
            core::hint::spin_loop();
        }
        
        // Wait for readers to drain
        while self.readers.load(Ordering::Relaxed) != 0 {
            core::hint::spin_loop();
        }
        
        RwSpinWriteGuard { lock: self }
    }
}

pub struct RwSpinReadGuard<'a, T> {
    lock: &'a RwSpinLock<T>,
}

impl<T> Deref for RwSpinReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for RwSpinReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.readers.fetch_sub(1, Ordering::Release);
    }
}

pub struct RwSpinWriteGuard<'a, T> {
    lock: &'a RwSpinLock<T>,
}

impl<T> Deref for RwSpinWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for RwSpinWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for RwSpinWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.writer.store(0, Ordering::Release);
    }
}
