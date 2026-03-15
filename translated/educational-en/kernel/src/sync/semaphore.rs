//! Semaphore
//!
//! Counting semaphore for resource limiting.

use core::sync::atomic::{AtomicI32, Ordering};

/// Counting semaphore
pub struct Semaphore {
    count: AtomicI32,
}

// Implementation block — defines methods for the type above.
impl Semaphore {
    /// Create new semaphore with initial count
    pub const fn new(count: i32) -> Self {
        Self { count: AtomicI32::new(count) }
    }
    
    /// Acquire (decrement) - blocks if count is 0
    pub fn acquire(&self) {
                // Infinite loop — runs until an explicit `break`.
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
    
    /// Try to acquire - returns false if would block
    pub fn try_acquire(&self) -> bool {
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
    
    /// Release (increment)
    pub fn release(&self) {
        self.count.fetch_add(1, Ordering::Release);
    }
    
    /// Get current count
    pub fn count(&self) -> i32 {
        self.count.load(Ordering::Relaxed)
    }
}

/// Binary semaphore (mutex-like)
pub // Type alias — gives an existing type a new name for clarity.
type BinarySemaphore = Semaphore;

// Implementation block — defines methods for the type above.
impl BinarySemaphore {
    pub const fn new_binary() -> Self {
        Self::new(1)
    }
}
