



use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use core::sync::atomic::{AtomicBool, Ordering};
use super::pic::{self, Gv};


pub extern "x86-interrupt" fn kdu(stack_frame: InterruptStackFrame) {
    crate::log_warn!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}


pub extern "x86-interrupt" fn lhg(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}






pub extern "x86-interrupt" fn npj(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    use crate::memory::paging::{PageFlags, UserMemoryRegion};
    
    let aff = Cr2::read().as_u64();
    
    
    crate::trace::akj(crate::trace::EventType::PageFault, aff);
    
    
    let gee = error_code.contains(PageFaultErrorCode::USER_MODE);
    let gea = error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION);
    let is_write = error_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE);
    
    
    if gee && gea && is_write {
        if crate::memory::cow::mhn(aff) {
            return; 
        }
    }
    
    
    if gee && !gea {
        let cr3: u64;
        unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags)); }
        if crate::memory::swap::mim(cr3, aff) {
            return; 
        }
    }
    
    if gee && !gea {
        
        let page_addr = aff & !0xFFF;
        
        
        let mok = aff >= UserMemoryRegion::CH_
            && aff < crate::exec::bfr();
        
        let bvx = crate::exec::lap();
        let mon = bvx > 0
            && aff >= bvx.saturating_sub(4096 * 16) 
            && aff < UserMemoryRegion::QM_;
        
        
        let cr3_val: u64;
        unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3_val, options(nostack, preserves_flags)); }
        let vma = crate::memory::vma::nas(cr3_val, aff);
        
        if mok || mon || vma.is_some() {
            
            let npl = if let Some(ref vma) = vma {
                crate::memory::vma::nyx(vma.prot)
            } else {
                PageFlags::FM_
            };
            
            
            let phys = crate::memory::frame::aan()
                .or_else(|| crate::memory::swap::pnu()); 
            if let Some(phys) = phys {
                let ggn = crate::exec::ffh(|space| {
                    space.map_page(page_addr, phys, npl)
                });
                
                if ggn == Some(Some(())) {
                    
                    crate::memory::swap::pmx(cr3_val, page_addr, phys);
                    return; 
                }
                
                
                crate::memory::frame::vk(phys);
            }
            
            
            crate::serial_println!("[PF] OOM for demand page at {:#x}, killing user process", aff);
            unsafe { crate::userland::azi(-11); } 
        }
        
        
        crate::serial_println!(
            "[PF] SEGFAULT: user accessed invalid addr {:#x} (brk={:#x}, stack_bottom={:#x})",
            aff, crate::exec::bfr(), bvx
        );
        unsafe { crate::userland::azi(-11); } 
    }
    
    
    crate::log_error!(
        "EXCEPTION: PAGE FAULT\n\
        Accessed Address: {:#x}\n\
        Error Code: {:?}\n\
        {:#?}",
        aff,
        error_code,
        stack_frame
    );
    
    panic!("Page fault at {:#x}", aff);
}


pub extern "x86-interrupt" fn mce(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    
    if stack_frame.code_segment & 3 == 3 {
        crate::serial_println!(
            "[GPF] User-mode GPF at RIP={:#x} error_code={}, killing process",
            stack_frame.instruction_pointer.as_u64(),
            error_code
        );
        unsafe { crate::userland::azi(-11); } 
    }

    
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\n\
        Error Code: {}\n\
        {:#?}",
        error_code, stack_frame
    );
}


pub extern "x86-interrupt" fn mrl(stack_frame: InterruptStackFrame) {
    
    if stack_frame.code_segment & 3 == 3 {
        crate::serial_println!(
            "[UD] User-mode invalid opcode at RIP={:#x}, killing process",
            stack_frame.instruction_pointer.as_u64()
        );
        unsafe { crate::userland::azi(-4); } 
    }

    
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}


pub extern "x86-interrupt" fn lgb(stack_frame: InterruptStackFrame) {
    
    if stack_frame.code_segment & 3 == 3 {
        crate::serial_println!(
            "[DE] User-mode divide error at RIP={:#x}, killing process",
            stack_frame.instruction_pointer.as_u64()
        );
        unsafe { crate::userland::azi(-8); } 
    }

    
    panic!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}


pub extern "x86-interrupt" fn lef(stack_frame: InterruptStackFrame) {
    if stack_frame.code_segment & 3 == 3 {
        crate::serial_println!(
            "[NM] User-mode #NM at RIP={:#x}, killing process",
            stack_frame.instruction_pointer.as_u64()
        );
        unsafe { crate::userland::azi(-4); } 
    }
    panic!("EXCEPTION: DEVICE NOT AVAILABLE (#NM) — FPU/SSE not enabled\n{:#?}", stack_frame);
}


pub extern "x86-interrupt" fn ovu(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    if stack_frame.code_segment & 3 == 3 {
        crate::serial_println!(
            "[SS] User-mode stack fault at RIP={:#x} error_code={}, killing process",
            stack_frame.instruction_pointer.as_u64(),
            error_code
        );
        unsafe { crate::userland::azi(-11); } 
    }
    panic!(
        "EXCEPTION: STACK-SEGMENT FAULT (#SS)\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}


pub extern "x86-interrupt" fn pvp(stack_frame: InterruptStackFrame) {
    if stack_frame.code_segment & 3 == 3 {
        crate::serial_println!(
            "[MF] User-mode x87 FPU error at RIP={:#x}, killing process",
            stack_frame.instruction_pointer.as_u64()
        );
        unsafe { crate::userland::azi(-8); } 
    }
    panic!("EXCEPTION: x87 FPU ERROR (#MF)\n{:#?}", stack_frame);
}


pub extern "x86-interrupt" fn osw(stack_frame: InterruptStackFrame) {
    if stack_frame.code_segment & 3 == 3 {
        crate::serial_println!(
            "[XM] User-mode SIMD exception at RIP={:#x}, killing process",
            stack_frame.instruction_pointer.as_u64()
        );
        unsafe { crate::userland::azi(-8); } 
    }
    panic!("EXCEPTION: SIMD FLOATING-POINT (#XM)\n{:#?}", stack_frame);
}


pub extern "x86-interrupt" fn pjr(_stack_frame: InterruptStackFrame) {
    if !MV_.load(Ordering::Relaxed) {
        unsafe {
            Gv.lock().notify_end_of_interrupt(pic::InterruptIndex::Timer.as_u8());
        }
        return;
    }
    
    crate::logger::tick();
    crate::time::tick();
    
    
    crate::trace::akj(crate::trace::EventType::TimerTick, 0);
    
    
    crate::thread::dvv();
    
    
    unsafe {
        Gv.lock().notify_end_of_interrupt(pic::InterruptIndex::Timer.as_u8());
    }
}


pub extern "x86-interrupt" fn jwt(_stack_frame: InterruptStackFrame) {
    if !MV_.load(Ordering::Relaxed) {
        crate::apic::bng();
        return;
    }
    
    
    crate::logger::tick();
    crate::time::tick();
    
    
    crate::trace::akj(crate::trace::EventType::TimerTick, 0);
    
    
    {
        static DBX_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
        let count = DBX_.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        if count % 100 == 0 {
            crate::lab_mode::trace_bus::bgi(
                crate::lab_mode::trace_bus::EventCategory::Interrupt,
                "timer tick (x100)",
                count,
            );
        }
    }
    
    
    crate::thread::dvv();
    
    
    crate::apic::bng();
}


pub extern "x86-interrupt" fn jwr(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    
    
    let mut bjk = Port::<u8>::new(0x64);
    let status: u8 = unsafe { bjk.read() };
    
    
    if status & 0x20 != 0 {
        let mut zu = Port::<u8>::new(0x60);
        let _: u8 = unsafe { zu.read() };
        crate::apic::bng();
        return;
    }
    
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    
    if !MV_.load(Ordering::Relaxed) {
        crate::apic::bng();
        return;
    }
    
    crate::keyboard::handle_scancode(scancode);
    
    crate::lab_mode::trace_bus::bgi(
        crate::lab_mode::trace_bus::EventCategory::Keyboard,
        "key press",
        scancode as u64,
    );
    
    crate::apic::bng();
}


pub extern "x86-interrupt" fn jws(_stack_frame: InterruptStackFrame) {
    crate::mouse::btc();
    crate::apic::bng();
}


pub extern "x86-interrupt" fn mvs(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    
    
    let mut bjk = Port::<u8>::new(0x64);
    let status: u8 = unsafe { bjk.read() };
    
    
    if status & 0x20 != 0 {
        
        let mut zu = Port::<u8>::new(0x60);
        let _: u8 = unsafe { zu.read() };
        
        unsafe {
            Gv.lock().notify_end_of_interrupt(pic::InterruptIndex::Keyboard.as_u8());
        }
        return;
    }
    
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    
    if !MV_.load(Ordering::Relaxed) {
        unsafe {
            Gv.lock().notify_end_of_interrupt(pic::InterruptIndex::Keyboard.as_u8());
        }
        return;
    }
    
    
    crate::keyboard::handle_scancode(scancode);
    
    
    crate::lab_mode::trace_bus::bgi(
        crate::lab_mode::trace_bus::EventCategory::Keyboard,
        "key press",
        scancode as u64,
    );
    
    
    unsafe {
        Gv.lock().notify_end_of_interrupt(pic::InterruptIndex::Keyboard.as_u8());
    }
}

static MV_: AtomicBool = AtomicBool::new(false);

pub fn gue(ready: bool) {
    MV_.store(ready, Ordering::SeqCst);
}


pub extern "x86-interrupt" fn ngh(_stack_frame: InterruptStackFrame) {
    
    crate::mouse::btc();
    
    
    unsafe {
        Gv.lock().notify_end_of_interrupt(pic::InterruptIndex::Mouse.as_u8());
    }
}




pub extern "x86-interrupt" fn oua(_stack_frame: InterruptStackFrame) {
    
    if crate::apic::lq() {
        crate::apic::bng();
    } else {
        unsafe {
            let dag = crate::memory::wk(crate::acpi::ggc());
            let lapic = dag as *mut u32;
            core::ptr::write_volatile(lapic.byte_add(0xB0), 0);
        }
    }
}



pub extern "x86-interrupt" fn oge(_stack_frame: InterruptStackFrame) {
    crate::apic::bng();
    crate::thread::boq();
}



pub extern "x86-interrupt" fn psf(_stack_frame: InterruptStackFrame) {
    
    if crate::virtio_net::is_initialized() {
        crate::virtio_net::btc();
    }
    
    
    if crate::drivers::net::aoh() && !crate::virtio_net::is_initialized() {
        crate::virtio_net::mhx();
    }
    
    
    if crate::virtio_blk::is_initialized() {
        crate::virtio_blk::btc();
    }
    
    crate::apic::bng();
}
