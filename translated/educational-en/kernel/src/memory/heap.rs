//! Kernel Heap Allocator
//! 
//! Tracked heap allocator for kernel memory allocations.
//! Wraps linked_list_allocator with devtools allocation tracking.

use linked_list_allocator::LockedHeap;
use core::alloc::{GlobalAllocator, Layout};

/// Inner heap allocator
static INNER: LockedHeap = LockedHeap::empty();

/// Tracked wrapper that forwards to LockedHeap + records stats
struct TrackedAllocator;

// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl GlobalAllocator for TrackedAllocator {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { INNER.alloc(layout) };
        if !ptr.is_null() {
            crate::devtools::track_allocator(layout.size());
        }
        ptr
    }

        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { INNER.dealloc(ptr, layout); }
        crate::devtools::track_dealloc(layout.size());
    }
}

#[global_allocator]
static ALLOCATOR: TrackedAllocator = TrackedAllocator;

/// Initialize kernel heap at specified address with given size
pub fn initialize_at(heap_start: usize, heap_size: usize) {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        INNER.lock().init(heap_start as *mut u8, heap_size);
    }
}

/// Get used heap space in bytes
pub fn used() -> usize {
    INNER.lock().used()
}

/// Get free heap space in bytes
pub fn free() -> usize {
    INNER.lock().free()
}

/// Allocate memory with alignment
/// Returns None if allocation fails
pub fn allocate(size: usize, align: usize) -> Option<*mut u8> {
    use core::alloc::Layout;
    
    let layout = Layout::from_size_align(size, align).ok()?;
    let ptr = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        alloc::alloc::alloc(layout)
    };
    
    if ptr.is_null() {
        None
    } else {
        Some(ptr)
    }
}

/// Deallocate memory
/// 
/// # Safety
/// - ptr must have been allocated by this allocator
/// - size and align must match the original allocation
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn deallocate(ptr: *mut u8, size: usize, align: usize) {
    use core::alloc::Layout;
    
    if let Ok(layout) = Layout::from_size_align(size, align) {
        alloc::alloc::dealloc(ptr, layout);
    }
}
