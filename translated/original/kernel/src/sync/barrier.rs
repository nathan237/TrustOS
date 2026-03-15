//! Barrier
//!
//! Synchronization barrier for coordinating multiple threads.

use core::sync::atomic::{AtomicU32, Ordering};

/// Reusable barrier
pub struct Barrier {
    /// Number of threads expected
    expected: u32,
    /// Current count
    count: AtomicU32,
    /// Generation (to allow reuse)
    generation: AtomicU32,
}

impl Barrier {
    /// Create barrier for N threads
    pub const fn new(n: u32) -> Self {
        Self {
            expected: n,
            count: AtomicU32::new(0),
            generation: AtomicU32::new(0),
        }
    }
    
    /// Wait at barrier - returns when all threads arrive
    pub fn wait(&self) -> bool {
        let gen = self.generation.load(Ordering::Relaxed);
        let arrived = self.count.fetch_add(1, Ordering::AcqRel) + 1;
        
        if arrived == self.expected {
            // Last one to arrive - release all
            self.count.store(0, Ordering::Relaxed);
            self.generation.fetch_add(1, Ordering::Release);
            true // Returns true for the thread that released
        } else {
            // Wait for generation to change
            while self.generation.load(Ordering::Acquire) == gen {
                core::hint::spin_loop();
            }
            false
        }
    }
    
    /// Reset barrier (unsafe - only when no threads waiting)
    pub unsafe fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
    }
}

/// One-shot barrier (single use)
pub struct OneShotBarrier {
    expected: u32,
    count: AtomicU32,
    released: AtomicU32,
}

impl OneShotBarrier {
    pub const fn new(n: u32) -> Self {
        Self {
            expected: n,
            count: AtomicU32::new(0),
            released: AtomicU32::new(0),
        }
    }
    
    pub fn wait(&self) {
        let arrived = self.count.fetch_add(1, Ordering::AcqRel) + 1;
        
        if arrived == self.expected {
            self.released.store(1, Ordering::Release);
        } else {
            while self.released.load(Ordering::Acquire) == 0 {
                core::hint::spin_loop();
            }
        }
    }
    
    pub fn is_released(&self) -> bool {
        self.released.load(Ordering::Relaxed) != 0
    }
}
