



use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;


pub const BUX_: u16 = 0;


const JS_: usize = 4096 * 5; 

lazy_static! {
    
    static ref Kt: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        
        
        tss.interrupt_stack_table[BUX_ as usize] = {
            static mut Aps: [u8; JS_] = [0; JS_];
            let stack_start = VirtAddr::from_ptr(unsafe { &Aps });
            let stack_end = stack_start + JS_ as u64;
            stack_end
        };
        
        tss
    };
}


pub fn qis() -> &'static TaskStateSegment {
    &Kt
}
