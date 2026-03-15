



use x86_64::structures::idt::{Fi, PageFaultErrorCode};
use core::sync::atomic::{AtomicBool, Ordering};
use super::pic::{self, Qh};


pub extern "x86-interrupt" fn qru(amw: Fi) {
    crate::log_warn!("EXCEPTION: BREAKPOINT\n{:#?}", amw);
}


pub extern "x86-interrupt" fn sap(
    amw: Fi,
    xza: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", amw);
}






pub extern "x86-interrupt" fn vaw(
    amw: Fi,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    use crate::memory::paging::{PageFlags, UserMemoryRegion};
    
    let bha = Cr2::read().cvr();
    
    
    crate::trace::bry(crate::trace::EventType::Mb, bha);
    
    
    let lgo = error_code.contains(PageFaultErrorCode::EJR_);
    let lgk = error_code.contains(PageFaultErrorCode::EBG_);
    let rm = error_code.contains(PageFaultErrorCode::DEJ_);
    
    
    if lgo && lgk && rm {
        if crate::memory::cow::tjf(bha) {
            return; 
        }
    }
    
    
    if lgo && !lgk {
        let jm: u64;
        unsafe { core::arch::asm!("mov {}, cr3", bd(reg) jm, options(nostack, preserves_flags)); }
        if crate::memory::swap::tla(jm, bha) {
            return; 
        }
    }
    
    if lgo && !lgk {
        
        let dkk = bha & !0xFFF;
        
        
        let tsl = bha >= UserMemoryRegion::CF_
            && bha < crate::exec::dfj();
        
        let eiy = crate::exec::rsb();
        let tsp = eiy > 0
            && bha >= eiy.ao(4096 * 16) 
            && bha < UserMemoryRegion::PP_;
        
        
        let klt: u64;
        unsafe { core::arch::asm!("mov {}, cr3", bd(reg) klt, options(nostack, preserves_flags)); }
        let vma = crate::memory::vma::uii(klt, bha);
        
        if tsl || tsp || vma.is_some() {
            
            let vay = if let Some(ref vma) = vma {
                crate::memory::vma::vni(vma.prot)
            } else {
                PageFlags::EW_
            };
            
            
            let ht = crate::memory::frame::azg()
                .or_else(|| crate::memory::swap::xmm()); 
            if let Some(ht) = ht {
                let lki = crate::exec::jwy(|atm| {
                    atm.bnl(dkk, ht, vay)
                });
                
                if lki == Some(Some(())) {
                    
                    crate::memory::swap::xlm(klt, dkk, ht);
                    return; 
                }
                
                
                crate::memory::frame::apt(ht);
            }
            
            
            crate::serial_println!("[PF] OOM for demand page at {:#x}, killing user process", bha);
            unsafe { crate::userland::ctw(-11); } 
        }
        
        
        crate::serial_println!(
            "[PF] SEGFAULT: user accessed invalid addr {:#x} (brk={:#x}, stack_bottom={:#x})",
            bha, crate::exec::dfj(), eiy
        );
        unsafe { crate::userland::ctw(-11); } 
    }
    
    
    crate::log_error!(
        "EXCEPTION: PAGE FAULT\n\
        Accessed Address: {:#x}\n\
        Error Code: {:?}\n\
        {:#?}",
        bha,
        error_code,
        amw
    );
    
    panic!("Page fault at {:#x}", bha);
}


pub extern "x86-interrupt" fn tbw(
    amw: Fi,
    error_code: u64,
) {
    
    if amw.dzo & 3 == 3 {
        crate::serial_println!(
            "[GPF] User-mode GPF at RIP={:#x} error_code={}, killing process",
            amw.edk.cvr(),
            error_code
        );
        unsafe { crate::userland::ctw(-11); } 
    }

    
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\n\
        Error Code: {}\n\
        {:#?}",
        error_code, amw
    );
}


pub extern "x86-interrupt" fn tvz(amw: Fi) {
    
    if amw.dzo & 3 == 3 {
        crate::serial_println!(
            "[UD] User-mode invalid opcode at RIP={:#x}, killing process",
            amw.edk.cvr()
        );
        unsafe { crate::userland::ctw(-4); } 
    }

    
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", amw);
}


pub extern "x86-interrupt" fn rze(amw: Fi) {
    
    if amw.dzo & 3 == 3 {
        crate::serial_println!(
            "[DE] User-mode divide error at RIP={:#x}, killing process",
            amw.edk.cvr()
        );
        unsafe { crate::userland::ctw(-8); } 
    }

    
    panic!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", amw);
}


pub extern "x86-interrupt" fn rxc(amw: Fi) {
    if amw.dzo & 3 == 3 {
        crate::serial_println!(
            "[NM] User-mode #NM at RIP={:#x}, killing process",
            amw.edk.cvr()
        );
        unsafe { crate::userland::ctw(-4); } 
    }
    panic!("EXCEPTION: DEVICE NOT AVAILABLE (#NM) — FPU/SSE not enabled\n{:#?}", amw);
}


pub extern "x86-interrupt" fn wsd(
    amw: Fi,
    error_code: u64,
) {
    if amw.dzo & 3 == 3 {
        crate::serial_println!(
            "[SS] User-mode stack fault at RIP={:#x} error_code={}, killing process",
            amw.edk.cvr(),
            error_code
        );
        unsafe { crate::userland::ctw(-11); } 
    }
    panic!(
        "EXCEPTION: STACK-SEGMENT FAULT (#SS)\nError Code: {}\n{:#?}",
        error_code, amw
    );
}


pub extern "x86-interrupt" fn xwg(amw: Fi) {
    if amw.dzo & 3 == 3 {
        crate::serial_println!(
            "[MF] User-mode x87 FPU error at RIP={:#x}, killing process",
            amw.edk.cvr()
        );
        unsafe { crate::userland::ctw(-8); } 
    }
    panic!("EXCEPTION: x87 FPU ERROR (#MF)\n{:#?}", amw);
}


pub extern "x86-interrupt" fn wok(amw: Fi) {
    if amw.dzo & 3 == 3 {
        crate::serial_println!(
            "[XM] User-mode SIMD exception at RIP={:#x}, killing process",
            amw.edk.cvr()
        );
        unsafe { crate::userland::ctw(-8); } 
    }
    panic!("EXCEPTION: SIMD FLOATING-POINT (#XM)\n{:#?}", amw);
}


pub extern "x86-interrupt" fn xhi(elu: Fi) {
    if !LY_.load(Ordering::Relaxed) {
        unsafe {
            Qh.lock().goa(pic::InterruptIndex::Timer.gaj());
        }
        return;
    }
    
    crate::logger::or();
    crate::time::or();
    
    
    crate::trace::bry(crate::trace::EventType::Ano, 0);
    
    
    crate::thread::hto();
    
    
    unsafe {
        Qh.lock().goa(pic::InterruptIndex::Timer.gaj());
    }
}


pub extern "x86-interrupt" fn qjl(elu: Fi) {
    if !LY_.load(Ordering::Relaxed) {
        crate::apic::dsp();
        return;
    }
    
    
    crate::logger::or();
    crate::time::or();
    
    
    crate::trace::bry(crate::trace::EventType::Ano, 0);
    
    
    {
        static CYF_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
        let az = CYF_.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        if az % 100 == 0 {
            crate::lab_mode::trace_bus::dgy(
                crate::lab_mode::trace_bus::EventCategory::Fv,
                "timer tick (x100)",
                az,
            );
        }
    }
    
    
    crate::thread::hto();
    
    
    crate::apic::dsp();
}


pub extern "x86-interrupt" fn qjj(elu: Fi) {
    use x86_64::instructions::port::Port;
    
    
    let mut dma = Port::<u8>::new(0x64);
    let status: u8 = unsafe { dma.read() };
    
    
    if status & 0x20 != 0 {
        let mut axr = Port::<u8>::new(0x60);
        let _: u8 = unsafe { axr.read() };
        crate::apic::dsp();
        return;
    }
    
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    
    if !LY_.load(Ordering::Relaxed) {
        crate::apic::dsp();
        return;
    }
    
    crate::keyboard::crc(scancode);
    
    crate::lab_mode::trace_bus::dgy(
        crate::lab_mode::trace_bus::EventCategory::Hs,
        "key press",
        scancode as u64,
    );
    
    crate::apic::dsp();
}


pub extern "x86-interrupt" fn qjk(elu: Fi) {
    crate::mouse::eck();
    crate::apic::dsp();
}


pub extern "x86-interrupt" fn ubi(elu: Fi) {
    use x86_64::instructions::port::Port;
    
    
    let mut dma = Port::<u8>::new(0x64);
    let status: u8 = unsafe { dma.read() };
    
    
    if status & 0x20 != 0 {
        
        let mut axr = Port::<u8>::new(0x60);
        let _: u8 = unsafe { axr.read() };
        
        unsafe {
            Qh.lock().goa(pic::InterruptIndex::Hs.gaj());
        }
        return;
    }
    
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    
    if !LY_.load(Ordering::Relaxed) {
        unsafe {
            Qh.lock().goa(pic::InterruptIndex::Hs.gaj());
        }
        return;
    }
    
    
    crate::keyboard::crc(scancode);
    
    
    crate::lab_mode::trace_bus::dgy(
        crate::lab_mode::trace_bus::EventCategory::Hs,
        "key press",
        scancode as u64,
    );
    
    
    unsafe {
        Qh.lock().goa(pic::InterruptIndex::Hs.gaj());
    }
}

static LY_: AtomicBool = AtomicBool::new(false);

pub fn mee(ack: bool) {
    LY_.store(ack, Ordering::SeqCst);
}


pub extern "x86-interrupt" fn ups(elu: Fi) {
    
    crate::mouse::eck();
    
    
    unsafe {
        Qh.lock().goa(pic::InterruptIndex::Cp.gaj());
    }
}




pub extern "x86-interrupt" fn wpy(elu: Fi) {
    
    if crate::apic::zu() {
        crate::apic::dsp();
    } else {
        unsafe {
            let gkr = crate::memory::auv(crate::acpi::ljo());
            let ku = gkr as *mut u32;
            core::ptr::write_volatile(ku.ygw(0xB0), 0);
        }
    }
}



pub extern "x86-interrupt" fn vxp(elu: Fi) {
    crate::apic::dsp();
    crate::thread::dvk();
}



pub extern "x86-interrupt" fn xrw(elu: Fi) {
    
    if crate::virtio_net::ky() {
        crate::virtio_net::eck();
    }
    
    
    if crate::drivers::net::bzy() && !crate::virtio_net::ky() {
        crate::virtio_net::tjx();
    }
    
    
    if crate::virtio_blk::ky() {
        crate::virtio_blk::eck();
    }
    
    crate::apic::dsp();
}
