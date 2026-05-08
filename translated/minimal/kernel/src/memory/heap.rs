




use linked_list_allocator::LockedHeap;
use core::alloc::{GlobalAlloc, Layout};


static Mg: LockedHeap = LockedHeap::empty();


struct Vm;

unsafe impl GlobalAlloc for Vm {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { Mg.alloc(layout) };
        if !ptr.is_null() {
            crate::devtools::pmp(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { Mg.dealloc(ptr, layout); }
        crate::devtools::pmu(layout.size());
    }
}

#[global_allocator]
static Asn: Vm = Vm;


pub fn gcm(heap_start: usize, atz: usize) {
    unsafe {
        Mg.lock().init(heap_start as *mut u8, atz);
    }
}


pub fn used() -> usize {
    Mg.lock().used()
}


pub fn free() -> usize {
    Mg.lock().free()
}



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






pub unsafe fn qch(ptr: *mut u8, size: usize, align: usize) {
    use core::alloc::Layout;
    
    if let Ok(layout) = Layout::from_size_align(size, align) {
        alloc::alloc::dealloc(ptr, layout);
    }
}
