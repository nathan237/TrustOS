







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
    
    static ref Bja: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        
        idt.hbf.bsh(handlers::qru);
        idt.ymq.bsh(handlers::sap);
        idt.zep.bsh(handlers::vaw);
        idt.ysj.bsh(handlers::tbw);
        idt.yyn.bsh(handlers::tvz);
        idt.ymi.bsh(handlers::rze);
        idt.ylx.bsh(handlers::rxc);
        idt.zpk.bsh(handlers::wsd);
        idt.zxk.bsh(handlers::xwg);
        idt.zom.bsh(handlers::wok);
        
        
        idt[pic::InterruptIndex::Timer.kbe()]
            .bsh(handlers::xhi);
        idt[pic::InterruptIndex::Hs.kbe()]
            .bsh(handlers::ubi);
        idt[pic::InterruptIndex::Cp.kbe()]
            .bsh(handlers::ups);
        
        
        idt[crate::apic::AIV_ as usize]
            .bsh(handlers::qjl);
        idt[crate::apic::UJ_ as usize]
            .bsh(handlers::qjj);
        idt[crate::apic::VJ_ as usize]
            .bsh(handlers::qjk);
        
        
        idt[0xFE].bsh(handlers::wpy);
        
        
        idt[0xFD].bsh(handlers::vxp);
        
        
        idt[crate::apic::HH_ as usize]
            .bsh(handlers::xrw);
        
        idt
    };
}


pub fn init() {
    #[cfg(target_arch = "x86_64")]
    {
        
        Bja.load();
        
        
        unsafe {
            pic::Qh.lock().cfp();
        }
        
        
        syscall::init();
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        crate::arch::platform::interrupts::ttu();
    }
    
    
    crate::arch::ofa();
    
    crate::log_debug!("Interrupts initialized and enabled");
}



pub fn ugz() {
    #[cfg(target_arch = "x86_64")]
    Bja.load();
}


pub fn cvh<G, Ac>(bb: G) -> Ac
where
    G: FnOnce() -> Ac,
{
    crate::arch::cvh(bb)
}


pub fn mee(ack: bool) {
    #[cfg(target_arch = "x86_64")]
    handlers::mee(ack);
    #[cfg(target_arch = "aarch64")]
    {
        AKB_.store(ack, core::sync::atomic::Ordering::SeqCst);
        if ack {
            
            crate::arch::platform::gic::isr(10);
            crate::serial_println!("[BOOTSTRAP] aarch64 timer started (10ms ticks)");
        }
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    let _ = ack;
}

#[cfg(target_arch = "aarch64")]
static AKB_: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);


pub fn ofs() -> bool {
    #[cfg(target_arch = "x86_64")]
    { return true; } 
    #[cfg(target_arch = "aarch64")]
    { return AKB_.load(core::sync::atomic::Ordering::SeqCst); }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    { false }
}
