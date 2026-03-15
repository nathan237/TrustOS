



use alloc::vec::Vec;
use alloc::string::String;
use super::LinuxProcess;
use super::loader::Ajv;


pub fn aeq(dyy: &super::loader::Ajv, process: &LinuxProcess) -> Result<u64, &'static str> {
    crate::serial_println!("[RUNTIME] Setting up environment for PID {}", process.ce);
    
    
    
    
    Ok(dyy.bt)
}





pub fn vw(bt: u64, process: &mut LinuxProcess) -> Result<i32, &'static str> {
    crate::serial_println!("[RUNTIME] Starting execution at {:#x}", bt);
    
    
    
    
    
    
    
    
    
    
    
    
    
    crate::println!("Linux binary execution not yet fully implemented.");
    crate::println!("Binary: {}", process.path);
    crate::println!("Entry point: {:#x}", bt);
    
    
    Ok(0)
}


pub fn wbl(eib: &str, process: &mut LinuxProcess) -> Result<i32, &'static str> {
    
    let ca = crate::linux::rootfs::mq(eib)?;
    let text = core::str::jg(&ca).jd(|_| "invalid script")?;
    
    
    let ak: Vec<&str> = text.ak().collect();
    if ak.is_empty() {
        return Ok(0);
    }
    
    
    for line in ak.iter().chz(if ak[0].cj("#!") { 1 } else { 0 }) {
        let line = line.em();
        if line.is_empty() || line.cj('#') {
            continue;
        }
        
        
        if let Err(_) = crate::linux::shell::azu(line) {
            
            break;
        }
    }
    
    Ok(0)
}

