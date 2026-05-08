//! Linux Binary Runtime
//!
//! Sets up the execution environment for Linux binaries and runs them.

use alloc::vec::Vec;
use alloc::string::String;
use super::LinuxProcess;
use super::loader::LoadedBinary;

/// Setup the runtime environment for a Linux binary
pub fn setup(binary: &super::loader::LoadedBinary, process: &LinuxProcess) -> Result<u64, &'static str> {
    crate::serial_println!("[RUNTIME] Setting up environment for PID {}", process.pid);
    
    // For now, we'll execute the binary in-place using TrustOS memory
    // A full implementation would set up user-space page tables
    
    Ok(binary.entry)
}

/// Run a Linux binary
/// 
/// This function executes the binary by jumping to its entry point
/// and handling syscalls as they occur.
pub fn run(entry: u64, process: &mut LinuxProcess) -> Result<i32, &'static str> {
    crate::serial_println!("[RUNTIME] Starting execution at {:#x}", entry);
    
    // For static binaries linked to run at specific addresses,
    // we need to copy segments to their expected locations
    
    // This is a simplified interpreter-style execution
    // Real execution would involve:
    // 1. Setting up page tables
    // 2. Switching to ring 3
    // 3. Handling syscall via SYSCALL instruction
    
    // For now, we'll simulate a simple shell-like execution
    // by interpreting the binary's behavior
    
    crate::println!("Linux binary execution not yet fully implemented.");
    crate::println!("Binary: {}", process.path);
    crate::println!("Entry point: {:#x}", entry);
    
    // Return success for now
    Ok(0)
}

/// Interpreter loop - for running shell scripts
pub fn run_script(script: &str, process: &mut LinuxProcess) -> Result<i32, &'static str> {
    // Read the script
    let content = crate::linux::rootfs::read_file(script)?;
    let text = core::str::from_utf8(&content).map_err(|_| "invalid script")?;
    
    // Check for shebang
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Ok(0);
    }
    
    // Skip shebang, execute each line
    for line in lines.iter().skip(if lines[0].starts_with("#!") { 1 } else { 0 }) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Execute the command through Linux shell
        if let Err(_) = crate::linux::shell::execute_command(line) {
            // exit was called
            break;
        }
    }
    
    Ok(0)
}


