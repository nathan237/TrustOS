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

// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl<T: Send> Send for SpinLock<T> {}
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl<T: Send> Sync for SpinLock<T> {}

// Implementation block — defines methods for the type above.
impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    
    #[inline]
        // Public function — callable from other modules.
pub fn lock(&self) -> SpinLockGuard<T> {
        let mut spin_count = 0u32;
        
        while self.locked.compare_exchange_weak(
            false, true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() {
            // Exponential backoff with spin hint
            spin_count += 1;
            for _ in 0..(1 << spin_count.minimum(6)) {
                core::hint::spin_loop();
            }
        }
        
        SpinLockGuard { lock: self }
    }
    
    #[inline]
        // Public function — callable from other modules.
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
        // Public function — callable from other modules.
pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }
}

// Public structure — visible outside this module.
pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

// Trait implementation — fulfills a behavioral contract.
impl<T> Deref for SpinLockGuard<'_, T> {
        // Type alias — gives an existing type a new name for clarity.
type Target = T;
    
    fn deref(&self) -> &T {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*self.lock.data.get() }
    }
}

// Trait implementation — fulfills a behavioral contract.
impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *self.lock.data.get() }
    }
}

// Trait implementation — fulfills a behavioral contract.
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

// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl<T: Send> Send for TicketLock<T> {}
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl<T: Send> Sync for TicketLock<T> {}

// Implementation block — defines methods for the type above.
impl<T> TicketLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            next_ticket: AtomicU64::new(0),
            now_serving: AtomicU64::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
        // Public function — callable from other modules.
pub fn lock(&self) -> TicketLockGuard<T> {
        let ticket = self.next_ticket.fetch_add(1, Ordering::Relaxed);
        
        while self.now_serving.load(Ordering::Acquire) != ticket {
            core::hint::spin_loop();
        }
        
        TicketLockGuard { lock: self }
    }
}

// Public structure — visible outside this module.
pub struct TicketLockGuard<'a, T> {
    lock: &'a TicketLock<T>,
}

// Trait implementation — fulfills a behavioral contract.
impl<T> Deref for TicketLockGuard<'_, T> {
        // Type alias — gives an existing type a new name for clarity.
type Target = T;
    
    fn deref(&self) -> &T {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*self.lock.data.get() }
    }
}

// Trait implementation — fulfills a behavioral contract.
impl<T> DerefMut for TicketLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *self.lock.data.get() }
    }
}

// Trait implementation — fulfills a behavioral contract.
impl<T> Drop for TicketLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.now_serving.fetch_add(1, Ordering::Release);
    }
}

/// Once initialization (like std::sync::Once)
pub struct Once {
    state: AtomicU32,
}

// Compile-time constant — evaluated at compilation, zero runtime cost.
const ONCE_UNINIT: u32 = 0;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const ONCE_RUNNING: u32 = 1;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const ONCE_COMPLETE: u32 = 2;

// Implementation block — defines methods for the type above.
impl Once {
    pub const fn new() -> Self {
        Self { state: AtomicU32::new(ONCE_UNINIT) }
    }
    
        // Public function — callable from other modules.
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
    
        // Public function — callable from other modules.
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

// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl<T: Send + Sync, F: Send> Send for Lazy<T, F> {}
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl<T: Send + Sync, F: Send> Sync for Lazy<T, F> {}

// Implementation block — defines methods for the type above.
impl<T, F: FnOnce() -> T> Lazy<T, F> {
    pub const fn new(init: F) -> Self {
        Self {
            cell: UnsafeCell::new(None),
            init: UnsafeCell::new(Some(init)),
            once: Once::new(),
        }
    }
    
        // Public function — callable from other modules.
pub fn get(&self) -> &T {
        self.once.call_once(|| {
            let init = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { (*self.init.get()).take().unwrap() };
            let value = init();
                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *self.cell.get() = Some(value) };
        });
        
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { (*self.cell.get()).as_ref().unwrap() }
    }
}

// Trait implementation — fulfills a behavioral contract.
impl<T, F: FnOnce() -> T> Deref for Lazy<T, F> {
        // Type alias — gives an existing type a new name for clarity.
type Target = T;
    
    fn deref(&self) -> &T {
        self.get()
    }
}

/// Atomic reference counter
pub struct AtomicRefCount {
    count: AtomicU32,
}

// Implementation block — defines methods for the type above.
impl AtomicRefCount {
    pub const fn new(initial: u32) -> Self {
        Self { count: AtomicU32::new(initial) }
    }
    
    #[inline]
        // Public function — callable from other modules.
pub fn inc(&self) -> u32 {
        self.count.fetch_add(1, Ordering::Relaxed) + 1
    }
    
    #[inline]
        // Public function — callable from other modules.
pub fn decrypt(&self) -> u32 {
        let previous = self.count.fetch_sub(1, Ordering::Release);
        if previous == 1 {
            core::sync::atomic::fence(Ordering::Acquire);
        }
        previous - 1
    }
    
    #[inline]
        // Public function — callable from other modules.
pub fn get(&self) -> u32 {
        self.count.load(Ordering::Relaxed)
    }
}

/// Sequence lock for read-heavy workloads
pub struct SequenceLock<T> {
    sequence: AtomicU64,
    data: UnsafeCell<T>,
}

// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl<T: Send> Send for SequenceLock<T> {}
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl<T: Send + Sync> Sync for SequenceLock<T> {}

// Implementation block — defines methods for the type above.
impl<T: Copy> SequenceLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            sequence: AtomicU64::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    /// Read value (may retry on concurrent write)
    pub fn read(&self) -> T {
                // Infinite loop — runs until an explicit `break`.
loop {
            let seq1 = self.sequence.load(Ordering::Acquire);
            if seq1 & 1 != 0 {
                // Writer active, spin
                core::hint::spin_loop();
                continue;
            }
            
            let value = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *self.data.get() };
            
            let seq2 = self.sequence.load(Ordering::Acquire);
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
        self.sequence.fetch_add(1, Ordering::Acquire);
        
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *self.data.get() = value };
        
        // Increment to even (write complete)
        self.sequence.fetch_add(1, Ordering::Release);
    }
}
