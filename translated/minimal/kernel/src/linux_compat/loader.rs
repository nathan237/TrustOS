



use alloc::vec::Vec;
use alloc::string::String;


pub struct Pk {
    
    pub entry: u64,
    
    pub base: u64,
    
    pub end: u64,
    
    pub brk_start: u64,
    
    pub segments: Vec<Aer>,
    
    pub interp: Option<String>,
}


pub struct Aer {
    pub addr: u64,
    pub size: u64,
    pub data: Vec<u8>,
    pub flags: u32,
}


pub fn qns(path: &str) -> Result<Pk, &'static str> {
    crate::serial_println!("[LOADER] Loading: {}", path);
    
    
    let data = crate::linux::rootfs::read_file(path)
        .map_err(|_| "file not found")?;
    
    
    if data.len() < 64 {
        return Err("file too small");
    }
    
    
    if &data[0..4] != b"\x7fELF" {
        return Err("not an ELF file");
    }
    
    
    if data[4] != 2 {
        return Err("not a 64-bit ELF");
    }
    
    
    if data[5] != 1 {
        return Err("not little-endian");
    }
    
    
    let e_type = u16::from_le_bytes([data[16], data[17]]);
    let e_machine = u16::from_le_bytes([data[18], data[19]]);
    let e_entry = u64::from_le_bytes(data[24..32].try_into().unwrap());
    let e_phoff = u64::from_le_bytes(data[32..40].try_into().unwrap());
    let e_phentsize = u16::from_le_bytes([data[54], data[55]]);
    let e_phnum = u16::from_le_bytes([data[56], data[57]]);
    
    crate::serial_println!("[LOADER] ELF type={}, machine={}, entry={:#x}, phnum={}", 
        e_type, e_machine, e_entry, e_phnum);
    
    
    if e_type != 2 && e_type != 3 {
        return Err("not an executable");
    }
    
    
    if e_machine != 62 {
        return Err("not x86_64");
    }
    
    let mut segments = Vec::new();
    let mut ghq = u64::MAX;
    let mut cmt = 0u64;
    let mut interp: Option<String> = None;
    
    
    for i in 0..e_phnum as usize {
        let aii = e_phoff as usize + i * e_phentsize as usize;
        if aii + 56 > data.len() {
            continue;
        }
        
        let p_type = u32::from_le_bytes(data[aii..aii+4].try_into().unwrap());
        let p_flags = u32::from_le_bytes(data[aii+4..aii+8].try_into().unwrap());
        let p_offset = u64::from_le_bytes(data[aii+8..aii+16].try_into().unwrap());
        let p_vaddr = u64::from_le_bytes(data[aii+16..aii+24].try_into().unwrap());
        let p_filesz = u64::from_le_bytes(data[aii+32..aii+40].try_into().unwrap());
        let p_memsz = u64::from_le_bytes(data[aii+40..aii+48].try_into().unwrap());
        
        match p_type {
            1 => {
                
                crate::serial_println!("[LOADER] LOAD segment: vaddr={:#x}, filesz={}, memsz={}", 
                    p_vaddr, p_filesz, p_memsz);
                
                
                if p_vaddr < ghq {
                    ghq = p_vaddr;
                }
                if p_vaddr + p_memsz > cmt {
                    cmt = p_vaddr + p_memsz;
                }
                
                
                let mut dyy = alloc::vec![0u8; p_memsz as usize];
                let lvc = p_offset as usize;
                let hyf = (p_offset + p_filesz) as usize;
                
                if hyf <= data.len() {
                    dyy[..p_filesz as usize].copy_from_slice(&data[lvc..hyf]);
                }
                
                segments.push(Aer {
                    addr: p_vaddr,
                    size: p_memsz,
                    data: dyy,
                    flags: p_flags,
                });
            }
            3 => {
                
                let start = p_offset as usize;
                let end = (p_offset + p_filesz) as usize;
                if end <= data.len() {
                    let path = &data[start..end];
                    let iub = core::str::from_utf8(path)
                        .unwrap_or("")
                        .trim_end_matches('\0');
                    interp = Some(String::from(iub));
                    crate::serial_println!("[LOADER] Interpreter: {}", iub);
                }
            }
            _ => {}
        }
    }
    
    if segments.is_empty() {
        return Err("no loadable segments");
    }
    
    
    
    let entry = if interp.is_some() {
        crate::serial_println!("[LOADER] Warning: dynamic binary, interpreter not supported yet");
        e_entry
    } else {
        e_entry
    };
    
    Ok(Pk {
        entry,
        base: ghq,
        end: cmt,
        brk_start: (cmt + 4095) & !4095, 
        segments,
        interp,
    })
}
