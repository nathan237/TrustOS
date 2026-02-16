//! Interrupt Handling Subsystem
//! 
//! Manages CPU interrupts, exceptions, and hardware IRQs.
//! Routes interrupts to appropriate handlers.

mod idt;
mod handlers;
mod pic;
pub mod syscall;

use x86_64::structures::idt::InterruptDescriptorTable;
use lazy_static::lazy_static;

lazy_static! {
    /// Interrupt Descriptor Table
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // CPU exceptions
        idt.breakpoint.set_handler_fn(handlers::breakpoint_handler);
        idt.double_fault.set_handler_fn(handlers::double_fault_handler);
        idt.page_fault.set_handler_fn(handlers::page_fault_handler);
        idt.general_protection_fault.set_handler_fn(handlers::general_protection_fault_handler);
        idt.invalid_opcode.set_handler_fn(handlers::invalid_opcode_handler);
        idt.divide_error.set_handler_fn(handlers::divide_error_handler);
        
        // Hardware interrupts
        idt[pic::InterruptIndex::Timer.as_usize()]
            .set_handler_fn(handlers::timer_interrupt_handler);
        idt[pic::InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(handlers::keyboard_interrupt_handler);
        idt[pic::InterruptIndex::Mouse.as_usize()]
            .set_handler_fn(handlers::mouse_interrupt_handler);
        
        // SMP IPI wakeup vector (0xFE = 254) - wakes APs from HLT
        idt[0xFE].set_handler_fn(handlers::smp_ipi_handler);
        
        idt
    };
}

/// Initialize interrupt handling
pub fn init() {
    // Load IDT
    IDT.load();
    
    // Initialize PIC
    unsafe {
        pic::PICS.lock().initialize();
    }
    
    // Initialize SYSCALL/SYSRET for userland
    syscall::init();
    
    // Enable interrupts
    x86_64::instructions::interrupts::enable();
    
    crate::log_debug!("IDT loaded, PIC initialized, SYSCALL ready, interrupts enabled");
}

/// Load IDT on an Application Processor (AP)
/// Called from AP entry point so it can handle IPI vectors
pub fn load_idt_on_ap() {
    IDT.load();
}

/// Disable interrupts and run closure
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    x86_64::instructions::interrupts::without_interrupts(f)
}

/// Allow timer handler to run once bootstrapping is ready
pub fn set_bootstrap_ready(ready: bool) {
    handlers::set_bootstrap_ready(ready);
}
