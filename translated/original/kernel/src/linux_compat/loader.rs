//! Linux ELF Binary Loader
//!
//! Loads Linux ELF64 binaries into memory.

use alloc::vec::Vec;
use alloc::string::String;

/// Loaded binary information
pub struct LoadedBinary {
    /// Entry point address
    pub entry: u64,
    /// Base address where binary is loaded
    pub base: u64,
    /// End of loaded segments
    pub end: u64,
    /// Initial brk value (end of data segment)
    pub brk_start: u64,
    /// Loaded segments
    pub segments: Vec<Segment>,
    /// Interpreter path (if dynamic)
    pub interp: Option<String>,
}

/// A loaded memory segment
pub struct Segment {
    pub addr: u64,
    pub size: u64,
    pub data: Vec<u8>,
    pub flags: u32,
}

/// Load a Linux binary from the rootfs
pub fn load_linux_binary(path: &str) -> Result<LoadedBinary, &'static str> {
    crate::serial_println!("[LOADER] Loading: {}", path);
    
    // Read the file
    let data = crate::linux::rootfs::read_file(path)
        .map_err(|_| "file not found")?;
    
    // Parse ELF header
    if data.len() < 64 {
        return Err("file too small");
    }
    
    // Check magic
    if &data[0..4] != b"\x7fELF" {
        return Err("not an ELF file");
    }
    
    // Check 64-bit
    if data[4] != 2 {
        return Err("not a 64-bit ELF");
    }
    
    // Check little-endian
    if data[5] != 1 {
        return Err("not little-endian");
    }
    
    // Parse header fields
    let e_type = u16::from_le_bytes([data[16], data[17]]);
    let e_machine = u16::from_le_bytes([data[18], data[19]]);
    let e_entry = u64::from_le_bytes(data[24..32].try_into().unwrap());
    let e_phoff = u64::from_le_bytes(data[32..40].try_into().unwrap());
    let e_phentsize = u16::from_le_bytes([data[54], data[55]]);
    let e_phnum = u16::from_le_bytes([data[56], data[57]]);
    
    crate::serial_println!("[LOADER] ELF type={}, machine={}, entry={:#x}, phnum={}", 
        e_type, e_machine, e_entry, e_phnum);
    
    // Check executable
    if e_type != 2 && e_type != 3 {
        return Err("not an executable");
    }
    
    // Check x86_64
    if e_machine != 62 {
        return Err("not x86_64");
    }
    
    let mut segments = Vec::new();
    let mut min_addr = u64::MAX;
    let mut max_addr = 0u64;
    let mut interp: Option<String> = None;
    
    // Parse program headers
    for i in 0..e_phnum as usize {
        let ph_offset = e_phoff as usize + i * e_phentsize as usize;
        if ph_offset + 56 > data.len() {
            continue;
        }
        
        let p_type = u32::from_le_bytes(data[ph_offset..ph_offset+4].try_into().unwrap());
        let p_flags = u32::from_le_bytes(data[ph_offset+4..ph_offset+8].try_into().unwrap());
        let p_offset = u64::from_le_bytes(data[ph_offset+8..ph_offset+16].try_into().unwrap());
        let p_vaddr = u64::from_le_bytes(data[ph_offset+16..ph_offset+24].try_into().unwrap());
        let p_filesz = u64::from_le_bytes(data[ph_offset+32..ph_offset+40].try_into().unwrap());
        let p_memsz = u64::from_le_bytes(data[ph_offset+40..ph_offset+48].try_into().unwrap());
        
        match p_type {
            1 => {
                // PT_LOAD - loadable segment
                crate::serial_println!("[LOADER] LOAD segment: vaddr={:#x}, filesz={}, memsz={}", 
                    p_vaddr, p_filesz, p_memsz);
                
                // Track address range
                if p_vaddr < min_addr {
                    min_addr = p_vaddr;
                }
                if p_vaddr + p_memsz > max_addr {
                    max_addr = p_vaddr + p_memsz;
                }
                
                // Load segment data
                let mut segment_data = alloc::vec![0u8; p_memsz as usize];
                let file_start = p_offset as usize;
                let file_end = (p_offset + p_filesz) as usize;
                
                if file_end <= data.len() {
                    segment_data[..p_filesz as usize].copy_from_slice(&data[file_start..file_end]);
                }
                
                segments.push(Segment {
                    addr: p_vaddr,
                    size: p_memsz,
                    data: segment_data,
                    flags: p_flags,
                });
            }
            3 => {
                // PT_INTERP - interpreter path
                let start = p_offset as usize;
                let end = (p_offset + p_filesz) as usize;
                if end <= data.len() {
                    let path = &data[start..end];
                    let path_str = core::str::from_utf8(path)
                        .unwrap_or("")
                        .trim_end_matches('\0');
                    interp = Some(String::from(path_str));
                    crate::serial_println!("[LOADER] Interpreter: {}", path_str);
                }
            }
            _ => {}
        }
    }
    
    if segments.is_empty() {
        return Err("no loadable segments");
    }
    
    // For static binaries, use the ELF entry point
    // For dynamic binaries, we'd need to load the interpreter
    let entry = if interp.is_some() {
        crate::serial_println!("[LOADER] Warning: dynamic binary, interpreter not supported yet");
        e_entry
    } else {
        e_entry
    };
    
    Ok(LoadedBinary {
        entry,
        base: min_addr,
        end: max_addr,
        brk_start: (max_addr + 4095) & !4095, // Page-aligned
        segments,
        interp,
    })
}
