






use alloc::string::String;
use alloc::vec::Vec;

use crate::process::{self, X, ProcessState, ProcessFlags, JK_};


pub struct Als {
    
    pub services: Vec<String>,
    
    pub spawn_shell: bool,
}

impl Default for Als {
    fn default() -> Self {
        Self {
            services: Vec::new(),
            spawn_shell: true,
        }
    }
}


pub fn start() {
    crate::log!("[INIT] Starting init process (PID 1)...");
    
    
    
    let qlc = crate::process::Process::new(
        JK_,
        0, 
        "init",
        ProcessFlags(ProcessFlags::Bm | ProcessFlags::Aie | ProcessFlags::Go)
    );
    
    
    
    
    crate::log!("[INIT] Init process ready");
    crate::log!("[OK] System initialization complete");
    
    
    
    
    
    
    
    
    nxh();
}


fn nxh() {
    crate::serial_println!("");
    crate::serial_println!("========================================");
    crate::serial_println!("  TrustOS System Ready");
    crate::serial_println!("========================================");
    crate::serial_println!("");
    
    
    crate::serial_println!("Mounted filesystems:");
    for (path, caa) in crate::vfs::dtl() {
        crate::serial_println!("  {} -> {}", path, caa);
    }
    crate::serial_println!("");
    
    
    let nyb = process::count();
    crate::serial_println!("Processes: {}", nyb);
    
    
    let heap_used = crate::memory::heap::used();
    crate::serial_println!("Heap used: {} KB", heap_used / 1024);
    
    crate::serial_println!("");
}


pub fn odl() {
    let pwn: Vec<(X, X)> = process::list()
        .iter()
        .filter_map(|(pid, _, state)| {
            if *state == ProcessState::Zombie {
                process::bwz(*pid, |aa| (*pid, aa.ppid))
            } else {
                None
            }
        })
        .collect();
    
    for (pid, ppid) in pwn {
        
        if ppid == JK_ || process::bwz(ppid, |_| ()).is_none() {
            if let Ok(code) = process::bqb(pid) {
                crate::log_debug!("[INIT] Reaped zombie process {} (exit code {})", pid, code);
            }
        }
    }
}


pub fn shutdown() {
    crate::log!("[INIT] System shutdown requested");
    
    
    for (pid, name, _) in process::list() {
        if pid > 1 {
            crate::log_debug!("[INIT] Terminating process {} ({})", pid, name);
            process::bne(pid).ok();
        }
    }
    
    
    crate::log!("[INIT] Syncing filesystems...");
    
    
    crate::log!("[INIT] System halted");
}


pub fn eya() {
    crate::log!("[INIT] System reboot requested");
    shutdown();
    
    
    
    crate::log!("[INIT] Rebooting...");
    
    
    unsafe {
        
        while (crate::arch::Port::<u8>::new(0x64).read() & 0x02) != 0 {}
        
        crate::arch::Port::<u8>::new(0x64).write(0xFE);
    }
}


pub fn run() {
    
    odl();
}
