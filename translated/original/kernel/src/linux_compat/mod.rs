//! Linux Binary Compatibility Layer
//!
//! Allows TrustOS to execute Linux ELF binaries by translating
//! Linux syscalls to TrustOS equivalents (similar to WSL1).
//!
//! Uses an x86_64 instruction interpreter - no hardware virtualization needed!

pub mod syscall;
pub mod loader;
pub mod runtime;
pub mod interpreter;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

/// Linux process context
pub struct LinuxProcess {
    /// Process ID
    pub pid: u32,
    /// Binary path
    pub path: String,
    /// Arguments
    pub argv: Vec<String>,
    /// Environment variables
    pub envp: Vec<String>,
    /// Current working directory
    pub cwd: String,
    /// Break pointer (for brk syscall)
    pub brk: u64,
    /// File descriptors (fd -> TrustOS fd mapping)
    pub fds: [Option<u32>; 256],
    /// Exit code (when terminated)
    pub exit_code: Option<i32>,
}

impl LinuxProcess {
    pub fn new(path: &str, argv: Vec<String>, envp: Vec<String>) -> Self {
        let mut fds = [None; 256];
        // Standard file descriptors
        fds[0] = Some(0); // stdin
        fds[1] = Some(1); // stdout  
        fds[2] = Some(2); // stderr
        
        Self {
            pid: allocate_pid(),
            path: String::from(path),
            argv,
            envp,
            cwd: String::from("/"),
            brk: 0,
            fds,
            exit_code: None,
        }
    }
}

static NEXT_PID: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(1000);

fn allocate_pid() -> u32 {
    NEXT_PID.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
}

/// Execute a Linux binary using the interpreter
pub fn exec(path: &str, args: &[&str]) -> Result<i32, &'static str> {
    crate::serial_println!("[LINUX] Executing: {} {:?}", path, args);
    
    // Read the ELF binary - try ramfs first, then linux rootfs
    let elf_data = crate::ramfs::with_fs(|fs| {
        fs.read_file(path).map(|d| d.to_vec())
    }).or_else(|_| crate::linux::rootfs::read_file(path))?;
    
    // Build argv
    let mut argv_strs: Vec<&str> = Vec::new();
    argv_strs.push(path);
    for arg in args {
        argv_strs.push(*arg);
    }
    
    // Run using the interpreter (no hardware virtualization needed!)
    crate::println!("╔══════════════════════════════════════════════════════════════╗");
    crate::println!("║  Linux Binary Interpreter                                    ║");
    crate::println!("║  Executing: {}                       ", path);
    crate::println!("╚══════════════════════════════════════════════════════════════╝");
    
    match interpreter::run_binary(&elf_data, &argv_strs) {
        Ok(exit_code) => {
            crate::println!("\n[Process exited with code {}]", exit_code);
            Ok(exit_code)
        }
        Err(e) => {
            crate::println_color!(0xFF0000, "\n[Process error: {}]", e);
            Err(e)
        }
    }
}

/// Check if a file is a Linux ELF binary
pub fn is_linux_binary(path: &str) -> bool {
    if let Ok(data) = crate::linux::rootfs::read_file(path) {
        // Check ELF magic
        data.len() >= 4 && &data[0..4] == b"\x7fELF"
    } else {
        false
    }
}
