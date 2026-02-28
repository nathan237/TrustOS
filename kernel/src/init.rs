//! Init Process (PID 1)
//!
//! The first userspace process, responsible for:
//! - Starting system services
//! - Reaping zombie processes
//! - Managing system shutdown

use alloc::string::String;
use alloc::vec::Vec;

use crate::process::{self, Pid, ProcessState, ProcessFlags, PID_INIT};

/// Init configuration
pub struct InitConfig {
    /// Services to start automatically
    pub services: Vec<String>,
    /// Whether to spawn a shell
    pub spawn_shell: bool,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            services: Vec::new(),
            spawn_shell: true,
        }
    }
}

/// Initialize the init process
pub fn start() {
    crate::log!("[INIT] Starting init process (PID 1)...");
    
    // Create the init process
    let init_proc = crate::process::Process::new(
        PID_INIT,
        0, // Parent is kernel
        "init",
        ProcessFlags(ProcessFlags::INIT | ProcessFlags::DAEMON)
    );
    
    // Add to process table manually (special case for init)
    // This is handled by process::init() now
    
    crate::log!("[INIT] Init process ready");
    crate::log!("[OK] System initialization complete");
    
    // In a real OS, init would:
    // 1. Read /etc/inittab or /etc/init.d/
    // 2. Start system services
    // 3. Setup virtual consoles
    // 4. Spawn login shells
    
    // For now, we just print status
    print_system_info();
}

/// Print system information
fn print_system_info() {
    crate::serial_println!("");
    crate::serial_println!("========================================");
    crate::serial_println!("  TrustOS System Ready");
    crate::serial_println!("========================================");
    crate::serial_println!("");
    
    // Print mount points
    crate::serial_println!("Mounted filesystems:");
    for (path, fstype) in crate::vfs::list_mounts() {
        crate::serial_println!("  {} -> {}", path, fstype);
    }
    crate::serial_println!("");
    
    // Print process count
    let proc_count = process::count();
    crate::serial_println!("Processes: {}", proc_count);
    
    // Print memory info
    let heap_used = crate::memory::heap::used();
    crate::serial_println!("Heap used: {} KB", heap_used / 1024);
    
    crate::serial_println!("");
}

/// Reap zombie processes (called periodically)
pub fn reap_zombies() {
    let zombies: Vec<(Pid, Pid)> = process::list()
        .iter()
        .filter_map(|(pid, _, state)| {
            if *state == ProcessState::Zombie {
                process::with_process(*pid, |p| (*pid, p.ppid))
            } else {
                None
            }
        })
        .collect();
    
    for (pid, ppid) in zombies {
        // If parent is init (or parent doesn't exist), reap it
        if ppid == PID_INIT || process::with_process(ppid, |_| ()).is_none() {
            if let Ok(code) = process::wait(pid) {
                crate::log_debug!("[INIT] Reaped zombie process {} (exit code {})", pid, code);
            }
        }
    }
}

/// Handle shutdown request
pub fn shutdown() {
    crate::log!("[INIT] System shutdown requested");
    
    // Send SIGTERM to all processes (except kernel and init)
    for (pid, name, _) in process::list() {
        if pid > 1 {
            crate::log_debug!("[INIT] Terminating process {} ({})", pid, name);
            process::kill(pid).ok();
        }
    }
    
    // Sync filesystems
    crate::log!("[INIT] Syncing filesystems...");
    // vfs::sync_all() would go here
    
    crate::log!("[INIT] System halted");
}

/// Handle reboot request
pub fn reboot() {
    crate::log!("[INIT] System reboot requested");
    shutdown();
    
    // On x86, we can use keyboard controller to reboot
    // Or triple fault
    crate::log!("[INIT] Rebooting...");
    
    // Simple reboot via keyboard controller
    unsafe {
        // Wait for keyboard controller to be ready
        while (crate::arch::Port::<u8>::new(0x64).read() & 0x02) != 0 {}
        // Send reset command
        crate::arch::Port::<u8>::new(0x64).write(0xFE);
    }
}

/// Run init main loop (called from scheduler)
pub fn run() {
    // Periodically reap zombies
    reap_zombies();
}
