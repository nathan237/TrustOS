//! BusyBox-like command implementations
//!
//! Provides implementations for common Linux utilities.

use alloc::string::String;
use alloc::vec::Vec;

/// Check if a file is executable (ELF binary or script)
pub fn is_executable(path: &str) -> bool {
    if let Ok(content) = super::rootfs::read_file(path) {
        // Check for ELF magic
        if content.len() >= 4 && &content[0..4] == b"\x7fELF" {
            return true;
        }
        // Check for shebang
        if content.len() >= 2 && &content[0..2] == b"#!" {
            return true;
        }
    }
    false
}

/// Get file type description
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

/// Parse a shell script and return commands
pub fn parse_script(content: &[u8]) -> Vec<String> {
    let mut commands = Vec::new();
    
    if let Ok(text) = core::str::from_utf8(content) {
        for line in text.lines() {
            let line = line.trim();
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            commands.push(String::from(line));
        }
    }
    
    commands
}
