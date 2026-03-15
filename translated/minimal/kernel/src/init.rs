






use alloc::string::String;
use alloc::vec::Vec;

use crate::process::{self, Ah, ProcessState, ProcessFlags, IS_};


pub struct Cfw {
    
    pub wib: Vec<String>,
    
    pub wqq: bool,
}

impl Default for Cfw {
    fn default() -> Self {
        Self {
            wib: Vec::new(),
            wqq: true,
        }
    }
}


pub fn ay() {
    crate::log!("[INIT] Starting init process (PID 1)...");
    
    
    
    let yxr = crate::process::Process::new(
        IS_,
        0, 
        "init",
        ProcessFlags(ProcessFlags::Dm | ProcessFlags::Cad | ProcessFlags::Ps)
    );
    
    
    
    
    crate::log!("[INIT] Init process ready");
    crate::log!("[OK] System initialization complete");
    
    
    
    
    
    
    
    
    vlj();
}


fn vlj() {
    crate::serial_println!("");
    crate::serial_println!("========================================");
    crate::serial_println!("  TrustOS System Ready");
    crate::serial_println!("========================================");
    crate::serial_println!("");
    
    
    crate::serial_println!("Mounted filesystems:");
    for (path, eqw) in crate::vfs::hqa() {
        crate::serial_println!("  {} -> {}", path, eqw);
    }
    crate::serial_println!("");
    
    
    let vmg = process::az();
    crate::serial_println!("Processes: {}", vmg);
    
    
    let afa = crate::memory::heap::mr();
    crate::serial_println!("Heap used: {} KB", afa / 1024);
    
    crate::serial_println!("");
}


pub fn vsw() {
    let xxm: Vec<(Ah, Ah)> = process::aoy()
        .iter()
        .kwb(|(ce, _, g)| {
            if *g == ProcessState::Vf {
                process::ela(*ce, |ai| (*ce, ai.bfb))
            } else {
                None
            }
        })
        .collect();
    
    for (ce, bfb) in xxm {
        
        if bfb == IS_ || process::ela(bfb, |_| ()).is_none() {
            if let Ok(aj) = process::ccm(ce) {
                crate::log_debug!("[INIT] Reaped zombie process {} (exit code {})", ce, aj);
            }
        }
    }
}


pub fn cbu() {
    crate::log!("[INIT] System shutdown requested");
    
    
    for (ce, j, _) in process::aoy() {
        if ce > 1 {
            crate::log_debug!("[INIT] Terminating process {} ({})", ce, j);
            process::dsm(ce).bq();
        }
    }
    
    
    crate::log!("[INIT] Syncing filesystems...");
    
    
    crate::log!("[INIT] System halted");
}


pub fn jlq() {
    crate::log!("[INIT] System reboot requested");
    cbu();
    
    
    
    crate::log!("[INIT] Rebooting...");
    
    
    unsafe {
        
        while (crate::arch::Port::<u8>::new(0x64).read() & 0x02) != 0 {}
        
        crate::arch::Port::<u8>::new(0x64).write(0xFE);
    }
}


pub fn vw() {
    
    vsw();
}
