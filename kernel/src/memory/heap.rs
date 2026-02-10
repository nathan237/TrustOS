//! Kernel Heap Allocator
//! 
//! Lock-free heap allocator for kernel memory allocations.
//! Provides global allocator implementation.

use linked_list_allocator::LockedHeap;

/// Global kernel heap allocator
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize kernel heap at specified address with given size
pub fn init_at(heap_start: usize, heap_size: usize) {
    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
}

/// Get used heap space in bytes
pub fn used() -> usize {
    ALLOCATOR.lock().used()
}

/// Get free heap space in bytes
pub fn free() -> usize {
    ALLOCATOR.lock().free()
}

/// Allocate memory with alignment
/// Returns None if allocation fails
pub fn allocate(size: usize, align: usize) -> Option<*mut u8> {
    use core::alloc::Layout;
    
    let layout = Layout::from_size_align(size, align).ok()?;
    let ptr = unsafe {
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
pub unsafe fn deallocate(ptr: *mut u8, size: usize, align: usize) {
    use core::alloc::Layout;
    
    if let Ok(layout) = Layout::from_size_align(size, align) {
        alloc::alloc::dealloc(ptr, layout);
    }
}
