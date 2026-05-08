



use alloc::vec::Vec;
use alloc::string::String;
use super::LinuxProcess;
use super::loader::Pk;


pub fn pk(bqr: &super::loader::Pk, process: &LinuxProcess) -> Result<u64, &'static str> {
    crate::serial_println!("[RUNTIME] Setting up environment for PID {}", process.pid);
    
    
    
    
    Ok(bqr.entry)
}





pub fn run(entry: u64, process: &mut LinuxProcess) -> Result<i32, &'static str> {
    crate::serial_println!("[RUNTIME] Starting execution at {:#x}", entry);
    
    
    
    
    
    
    
    
    
    
    
    
    
    crate::println!("Linux binary execution not yet fully implemented.");
    crate::println!("Binary: {}", process.path);
    crate::println!("Entry point: {:#x}", entry);
    
    
    Ok(0)
}


pub fn ojf(script: &str, process: &mut LinuxProcess) -> Result<i32, &'static str> {
    
    let content = crate::linux::rootfs::read_file(script)?;
    let text = core::str::from_utf8(&content).map_err(|_| "invalid script")?;
    
    
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Ok(0);
    }
    
    
    for line in lines.iter().skip(if lines[0].starts_with("#!") { 1 } else { 0 }) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        
        if let Err(_) = crate::linux::shell::aav(line) {
            
            break;
        }
    }
    
    Ok(0)
}

