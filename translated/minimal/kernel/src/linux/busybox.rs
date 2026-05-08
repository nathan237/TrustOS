



use alloc::string::String;
use alloc::vec::Vec;


pub fn is_executable(path: &str) -> bool {
    if let Ok(content) = super::rootfs::read_file(path) {
        
        if content.len() >= 4 && &content[0..4] == b"\x7fELF" {
            return true;
        }
        
        if content.len() >= 2 && &content[0..2] == b"#!" {
            return true;
        }
    }
    false
}


pub fn file_type(path: &str) -> &'static str {
    if let Ok(content) = super::rootfs::read_file(path) {
        if content.len() >= 4 && &content[0..4] == b"\x7fELF" {
            return "ELF 64-bit LSB executable, x86-64";
        }
        if content.len() >= 2 && &content[0..2] == b"#!" {
            return "POSIX shell script, ASCII text executable";
        }
        if content.iter().all(|&b| b.is_ascii()) {
            return "ASCII text";
        }
        return "data";
    }
    "cannot open"
}


pub fn qqd(content: &[u8]) -> Vec<String> {
    let mut commands = Vec::new();
    
    if let Ok(text) = core::str::from_utf8(content) {
        for line in text.lines() {
            let line = line.trim();
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            commands.push(String::from(line));
        }
    }
    
    commands
}
