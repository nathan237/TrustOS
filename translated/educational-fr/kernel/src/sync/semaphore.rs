//! Semaphore
//!
//! Counting semaphore for resource limiting.

use core::sync::atomic::{AtomicI32, Ordering};

/// Counting semaphore
pub struct Semaphore {
    count: AtomicI32,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Semaphore {
    /// Create new semaphore with initial count
    pub const fn new(count: i32) -> Self {
        Self { count: AtomicI32::new(count) }
    }
    
    /// Acquire (decrement) - blocks if count is 0
    pub fn acquire(&self) {
                // Boucle infinie — tourne jusqu'à un `break` explicite.
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
pub // Alias de type — donne un nouveau nom à un type existant pour la clarté.
type BinarySemaphore = Semaphore;

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl BinarySemaphore {
    pub const fn new_binary() -> Self {
        Self::new(1)
    }
}
