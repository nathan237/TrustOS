




use linked_list_allocator::LockedHeap;
use core::alloc::{Cel, Layout};


static Ace: LockedHeap = LockedHeap::azs();


struct Azy;

unsafe impl Cel for Azy {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { Ace.alloc(layout) };
        if !ptr.abq() {
            crate::devtools::xld(layout.aw());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { Ace.dealloc(ptr, layout); }
        crate::devtools::xlj(layout.aw());
    }
}

#[global_allocator]
static Crc: Azy = Azy;


pub fn lec(caa: usize, cre: usize) {
    unsafe {
        Ace.lock().init(caa as *mut u8, cre);
    }
}


pub fn mr() -> usize {
    Ace.lock().mr()
}


pub fn aez() -> usize {
    Ace.lock().aez()
}



pub fn ijo(aw: usize, align: usize) -> Option<*mut u8> {
    use core::alloc::Layout;
    
    let layout = Layout::bjy(aw, align).bq()?;
    let ptr = unsafe {
        alloc::alloc::alloc(layout)
    };
    
    if ptr.abq() {
        None
    } else {
        Some(ptr)
    }
}






pub unsafe fn ylh(ptr: *mut u8, aw: usize, align: usize) {
    use core::alloc::Layout;
    
    if let Ok(layout) = Layout::bjy(aw, align) {
        alloc::alloc::dealloc(ptr, layout);
    }
}
