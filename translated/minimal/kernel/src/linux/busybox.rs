



use alloc::string::String;
use alloc::vec::Vec;


pub fn clc(path: &str) -> bool {
    if let Ok(ca) = super::rootfs::mq(path) {
        
        if ca.len() >= 4 && &ca[0..4] == b"\x7fELF" {
            return true;
        }
        
        if ca.len() >= 2 && &ca[0..2] == b"#!" {
            return true;
        }
    }
    false
}


pub fn kd(path: &str) -> &'static str {
    if let Ok(ca) = super::rootfs::mq(path) {
        if ca.len() >= 4 && &ca[0..4] == b"\x7fELF" {
            return "ELF 64-bit LSB executable, x86-64";
        }
        if ca.len() >= 2 && &ca[0..2] == b"#!" {
            return "POSIX shell script, ASCII text executable";
        }
        if ca.iter().xx(|&o| o.ofo()) {
            return "ASCII text";
        }
        return "data";
    }
    "cannot open"
}


pub fn zet(ca: &[u8]) -> Vec<String> {
    let mut commands = Vec::new();
    
    if let Ok(text) = core::str::jg(ca) {
        for line in text.ak() {
            let line = line.em();
            
            if line.is_empty() || line.cj('#') {
                continue;
            }
            commands.push(String::from(line));
        }
    }
    
    commands
}
