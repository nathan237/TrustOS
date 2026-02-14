//! Interrupt Handlers
//! 
//! Individual handlers for CPU exceptions and hardware interrupts.

use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use core::sync::atomic::{AtomicBool, Ordering};
use super::pic::{self, PICS};

/// Breakpoint exception handler
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::log_warn!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// Double fault handler (unrecoverable)
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

/// Page fault handler
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    
    let addr = Cr2::read();
    
    crate::log_error!(
        "EXCEPTION: PAGE FAULT\n\
        Accessed Address: {:?}\n\
        Error Code: {:?}\n\
        {:#?}",
        addr,
        error_code,
        stack_frame
    );
    
    // Record in trace
    crate::trace::record_event(crate::trace::EventType::PageFault, addr.as_u64());
    
    // For now, panic on page fault
    // TODO: Implement proper page fault handling
    panic!("Page fault at {:?}", addr);
}

/// General protection fault handler
pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\n\
        Error Code: {}\n\
        {:#?}",
        error_code, stack_frame
    );
}

/// Invalid opcode handler
pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

/// Divide by zero handler
pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

/// Timer interrupt handler
pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    if !BOOTSTRAP_READY.load(Ordering::Relaxed) {
        unsafe {
            PICS.lock().notify_end_of_interrupt(pic::InterruptIndex::Timer.as_u8());
        }
        return;
    }
    // Update tick counter
    crate::logger::tick();
    crate::time::tick();
    
    // Record timer event
    crate::trace::record_event(crate::trace::EventType::TimerTick, 0);
    
    // TrustLab trace: emit timer event at reduced rate
    {
        static TIMER_DIVISOR: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
        let count = TIMER_DIVISOR.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        if count % 100 == 0 {
            crate::lab_mode::trace_bus::emit_static(
                crate::lab_mode::trace_bus::EventCategory::Interrupt,
                "timer tick (x100)",
                count,
            );
        }
    }
    
    // Notify scheduler
    crate::scheduler::on_timer_tick();
    
    // Send EOI
    unsafe {
        PICS.lock().notify_end_of_interrupt(pic::InterruptIndex::Timer.as_u8());
    }
}

/// Keyboard interrupt handler
pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    
    // Check if data is from mouse (bit 5 of status register)
    let mut status_port = Port::<u8>::new(0x64);
    let status: u8 = unsafe { status_port.read() };
    
    // If bit 5 is set, data is from mouse - ignore it here
    if status & 0x20 != 0 {
        // Consume the byte to clear the buffer
        let mut data_port = Port::<u8>::new(0x60);
        let _: u8 = unsafe { data_port.read() };
        // Send EOI
        unsafe {
            PICS.lock().notify_end_of_interrupt(pic::InterruptIndex::Keyboard.as_u8());
        }
        return;
    }
    
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    // Process scancode through keyboard driver
    crate::keyboard::handle_scancode(scancode);
    
    // TrustLab trace: emit keyboard event
    crate::lab_mode::trace_bus::emit_static(
        crate::lab_mode::trace_bus::EventCategory::Keyboard,
        "key press",
        scancode as u64,
    );
    
    // Send EOI
    unsafe {
        PICS.lock().notify_end_of_interrupt(pic::InterruptIndex::Keyboard.as_u8());
    }
}

static BOOTSTRAP_READY: AtomicBool = AtomicBool::new(false);

pub fn set_bootstrap_ready(ready: bool) {
    BOOTSTRAP_READY.store(ready, Ordering::SeqCst);
}

/// Mouse interrupt handler (IRQ12)
pub extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Handle mouse data
    crate::mouse::handle_interrupt();
    
    // Send EOI to both PICs (IRQ12 is on PIC2)
    unsafe {
        PICS.lock().notify_end_of_interrupt(pic::InterruptIndex::Mouse.as_u8());
    }
}
