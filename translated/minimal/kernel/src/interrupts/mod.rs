







#[cfg(target_arch = "x86_64")]
mod idt;
#[cfg(target_arch = "x86_64")]
mod handlers;
#[cfg(target_arch = "x86_64")]
mod pic;
#[cfg(target_arch = "x86_64")]
pub mod syscall;

#[cfg(not(target_arch = "x86_64"))]
pub mod syscall {
    pub fn init() {}
}

#[cfg(target_arch = "x86_64")]
use x86_64::structures::idt::InterruptDescriptorTable;
#[cfg(target_arch = "x86_64")]
use lazy_static::lazy_static;

#[cfg(target_arch = "x86_64")]
lazy_static! {
    
    static ref Zt: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        
        idt.breakpoint.set_handler_fn(handlers::kdu);
        idt.double_fault.set_handler_fn(handlers::lhg);
        idt.page_fault.set_handler_fn(handlers::npj);
        idt.general_protection_fault.set_handler_fn(handlers::mce);
        idt.invalid_opcode.set_handler_fn(handlers::mrl);
        idt.divide_error.set_handler_fn(handlers::lgb);
        idt.device_not_available.set_handler_fn(handlers::lef);
        idt.stack_segment_fault.set_handler_fn(handlers::ovu);
        idt.x87_floating_point.set_handler_fn(handlers::pvp);
        idt.simd_floating_point.set_handler_fn(handlers::osw);
        
        
        idt[pic::InterruptIndex::Timer.as_usize()]
            .set_handler_fn(handlers::pjr);
        idt[pic::InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(handlers::mvs);
        idt[pic::InterruptIndex::Mouse.as_usize()]
            .set_handler_fn(handlers::ngh);
        
        
        idt[crate::apic::AKS_ as usize]
            .set_handler_fn(handlers::jwt);
        idt[crate::apic::VS_ as usize]
            .set_handler_fn(handlers::jwr);
        idt[crate::apic::WS_ as usize]
            .set_handler_fn(handlers::jws);
        
        
        idt[0xFE].set_handler_fn(handlers::oua);
        
        
        idt[0xFD].set_handler_fn(handlers::oge);
        
        
        idt[crate::apic::HZ_ as usize]
            .set_handler_fn(handlers::psf);
        
        idt
    };
}


pub fn init() {
    #[cfg(target_arch = "x86_64")]
    {
        
        Zt.load();
        
        
        unsafe {
            pic::Gv.lock().initialize();
        }
        
        
        syscall::init();
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        crate::arch::platform::interrupts::mpk();
    }
    
    
    crate::arch::ihd();
    
    crate::log_debug!("Interrupts initialized and enabled");
}



pub fn nad() {
    #[cfg(target_arch = "x86_64")]
    Zt.load();
}


pub fn bag<F, U>(f: F) -> U
where
    F: FnOnce() -> U,
{
    crate::arch::bag(f)
}


pub fn gue(ready: bool) {
    #[cfg(target_arch = "x86_64")]
    handlers::gue(ready);
    #[cfg(target_arch = "aarch64")]
    {
        ALW_.store(ready, core::sync::atomic::Ordering::SeqCst);
        if ready {
            
            crate::arch::platform::gic::eli(10);
            crate::serial_println!("[BOOTSTRAP] aarch64 timer started (10ms ticks)");
        }
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    let _ = ready;
}

#[cfg(target_arch = "aarch64")]
static ALW_: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);


pub fn iht() -> bool {
    #[cfg(target_arch = "x86_64")]
    { return true; } 
    #[cfg(target_arch = "aarch64")]
    { return ALW_.load(core::sync::atomic::Ordering::SeqCst); }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    { false }
}
