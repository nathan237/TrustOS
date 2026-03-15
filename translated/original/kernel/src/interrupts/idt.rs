//! Interrupt Descriptor Table utilities
//! 
//! Helper functions for IDT management.

use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;

/// Double fault stack index in TSS
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// Stack size for exception handlers
const STACK_SIZE: usize = 4096 * 5; // 20 KB

lazy_static! {
    /// Task State Segment for handling critical exceptions
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        
        // Set up separate stack for double faults
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE as u64;
            stack_end
        };
        
        tss
    };
}

/// Get TSS reference
pub fn get_tss() -> &'static TaskStateSegment {
    &TSS
}
