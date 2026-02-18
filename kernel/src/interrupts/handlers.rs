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

/// Page fault handler — implements demand paging for user processes
///
/// If the faulting address belongs to a valid user region (heap or stack)
/// and the page is simply not yet mapped, we allocate a frame and map it.
/// Otherwise we kill the process (or panic if in kernel mode).
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    use crate::memory::paging::{PageFlags, UserMemoryRegion};
    
    let fault_addr = Cr2::read().as_u64();
    
    // Record in trace
    crate::trace::record_event(crate::trace::EventType::PageFault, fault_addr);
    
    // ── Demand paging: only for user-mode faults on non-present pages ──
    let is_user_fault = error_code.contains(PageFaultErrorCode::USER_MODE);
    let is_protection = error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION);
    let is_write = error_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE);
    
    // ── COW: user write fault on copy-on-write page ──
    if is_user_fault && is_protection && is_write {
        if crate::memory::cow::handle_cow_fault(fault_addr) {
            return; // COW page resolved — resume user process
        }
    }
    
    // ── Swap: check if page was swapped out ──
    if is_user_fault && !is_protection {
        let cr3: u64;
        unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags)); }
        if crate::memory::swap::handle_swap_fault(cr3, fault_addr) {
            return; // Page swapped back in — resume
        }
    }
    
    if is_user_fault && !is_protection {
        // Fault on a non-present page from Ring 3 — try to service it
        let page_addr = fault_addr & !0xFFF;
        
        // Check if address is in a valid user region
        let in_heap = fault_addr >= UserMemoryRegion::HEAP_START
            && fault_addr < crate::exec::current_brk();
        
        let stack_bottom = crate::exec::current_stack_bottom();
        let in_stack = stack_bottom > 0
            && fault_addr >= stack_bottom.saturating_sub(4096 * 16) // allow 16 pages of stack growth
            && fault_addr < UserMemoryRegion::STACK_TOP;
        
        if in_heap || in_stack {
            // Allocate a physical frame and map it
            let phys = crate::memory::frame::alloc_frame_zeroed()
                .or_else(|| crate::memory::swap::try_evict_page()); // try swap eviction on OOM
            if let Some(phys) = phys {
                let mapped = crate::exec::with_current_address_space(|space| {
                    space.map_page(page_addr, phys, PageFlags::USER_DATA)
                });
                
                if mapped == Some(Some(())) {
                    // Track page for swap subsystem
                    let cr3_val: u64;
                    unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3_val, options(nostack, preserves_flags)); }
                    crate::memory::swap::track_page(cr3_val, page_addr, phys);
                    crate::serial_println!("[PF] Demand-paged {:#x} (phys {:#x})", page_addr, phys);
                    return; // Resume user process — IRET back to the faulting instruction
                }
                
                // Mapping failed — free the frame
                crate::memory::frame::free_frame(phys);
            }
            
            // OOM or mapping failure — kill process
            crate::serial_println!("[PF] OOM for demand page at {:#x}, killing user process", fault_addr);
            unsafe { crate::userland::return_from_ring3(-11); } // SIGSEGV
        }
        
        // Invalid user address — segfault
        crate::serial_println!(
            "[PF] SEGFAULT: user accessed invalid addr {:#x} (brk={:#x}, stack_bottom={:#x})",
            fault_addr, crate::exec::current_brk(), stack_bottom
        );
        unsafe { crate::userland::return_from_ring3(-11); } // SIGSEGV
    }
    
    // ── Kernel page fault or protection violation — fatal ──
    crate::log_error!(
        "EXCEPTION: PAGE FAULT\n\
        Accessed Address: {:#x}\n\
        Error Code: {:?}\n\
        {:#?}",
        fault_addr,
        error_code,
        stack_frame
    );
    
    panic!("Page fault at {:#x}", fault_addr);
}

/// General protection fault handler
pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // Check if fault came from Ring 3 (user mode)
    if stack_frame.code_segment & 3 == 3 {
        crate::serial_println!(
            "[GPF] User-mode GPF at RIP={:#x} error_code={}, killing process",
            stack_frame.instruction_pointer.as_u64(),
            error_code
        );
        unsafe { crate::userland::return_from_ring3(-11); } // SIGSEGV
    }

    // Kernel GPF — fatal
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\n\
        Error Code: {}\n\
        {:#?}",
        error_code, stack_frame
    );
}

/// Invalid opcode handler
pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    // Check if fault came from Ring 3 (user mode)
    if stack_frame.code_segment & 3 == 3 {
        crate::serial_println!(
            "[UD] User-mode invalid opcode at RIP={:#x}, killing process",
            stack_frame.instruction_pointer.as_u64()
        );
        unsafe { crate::userland::return_from_ring3(-4); } // SIGILL
    }

    // Kernel invalid opcode — fatal
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

/// Divide by zero handler
pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    // Check if fault came from Ring 3 (user mode)
    if stack_frame.code_segment & 3 == 3 {
        crate::serial_println!(
            "[DE] User-mode divide error at RIP={:#x}, killing process",
            stack_frame.instruction_pointer.as_u64()
        );
        unsafe { crate::userland::return_from_ring3(-8); } // SIGFPE
    }

    // Kernel divide error — fatal
    panic!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

/// Timer interrupt handler (legacy PIC — vector 32)
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
    
    // Notify thread scheduler (real context switch)
    crate::thread::on_timer_tick();
    
    // Send EOI
    unsafe {
        PICS.lock().notify_end_of_interrupt(pic::InterruptIndex::Timer.as_u8());
    }
}

/// APIC Timer interrupt handler (vector 48) — preemptive scheduling
pub extern "x86-interrupt" fn apic_timer_handler(_stack_frame: InterruptStackFrame) {
    if !BOOTSTRAP_READY.load(Ordering::Relaxed) {
        crate::apic::lapic_eoi();
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
    
    // Notify thread scheduler — this does real preemptive context switching!
    crate::thread::on_timer_tick();
    
    // Send LAPIC EOI
    crate::apic::lapic_eoi();
}

/// APIC-routed keyboard handler (vector 50)
pub extern "x86-interrupt" fn apic_keyboard_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    
    // Check if data is from mouse (bit 5 of status register)
    let mut status_port = Port::<u8>::new(0x64);
    let status: u8 = unsafe { status_port.read() };
    
    if status & 0x20 != 0 {
        let mut data_port = Port::<u8>::new(0x60);
        let _: u8 = unsafe { data_port.read() };
        crate::apic::lapic_eoi();
        return;
    }
    
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    crate::keyboard::handle_scancode(scancode);
    
    crate::lab_mode::trace_bus::emit_static(
        crate::lab_mode::trace_bus::EventCategory::Keyboard,
        "key press",
        scancode as u64,
    );
    
    crate::apic::lapic_eoi();
}

/// APIC-routed mouse handler (vector 61)
pub extern "x86-interrupt" fn apic_mouse_handler(_stack_frame: InterruptStackFrame) {
    crate::mouse::handle_interrupt();
    crate::apic::lapic_eoi();
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

/// SMP IPI wakeup handler (vector 0xFE)
/// This interrupt does nothing except wake the AP from HLT.
/// The actual work check happens in the AP loop after returning from HLT.
pub extern "x86-interrupt" fn smp_ipi_handler(_stack_frame: InterruptStackFrame) {
    // Send EOI via APIC module (or raw LAPIC write as fallback)
    if crate::apic::is_enabled() {
        crate::apic::lapic_eoi();
    } else {
        unsafe {
            let lapic_virt = crate::memory::phys_to_virt(crate::acpi::local_apic_address());
            let lapic = lapic_virt as *mut u32;
            core::ptr::write_volatile(lapic.byte_add(0xB0), 0);
        }
    }
}

/// SMP reschedule IPI handler (vector 0xFD)
/// Triggers a scheduling pass on the receiving CPU.
pub extern "x86-interrupt" fn reschedule_ipi_handler(_stack_frame: InterruptStackFrame) {
    crate::apic::lapic_eoi();
    crate::thread::schedule();
}

/// VirtIO shared interrupt handler (vector 62)
/// Checks ISR status on all VirtIO devices and dispatches accordingly.
pub extern "x86-interrupt" fn virtio_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Check virtio-net
    if crate::virtio_net::is_initialized() {
        crate::virtio_net::handle_interrupt();
    }
    
    // Check virtio-blk
    if crate::virtio_blk::is_initialized() {
        crate::virtio_blk::handle_interrupt();
    }
    
    crate::apic::lapic_eoi();
}
