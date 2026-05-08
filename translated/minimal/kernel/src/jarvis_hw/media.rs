













use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;





#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryFormat {
    Elf32,
    Elf64,
    Pe32,       
    Pe64,       
    MachO64,    
    FlatBinary, 
    FatImage,   
    Ext4Image,  
    NtfsImage,  
    Gpt,        
    Fu,        
    Unknown,
}

impl BinaryFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryFormat::Elf32 => "ELF32",
            BinaryFormat::Elf64 => "ELF64",
            BinaryFormat::Pe32 => "PE32",
            BinaryFormat::Pe64 => "PE64",
            BinaryFormat::MachO64 => "Mach-O 64",
            BinaryFormat::FlatBinary => "Flat Binary",
            BinaryFormat::FatImage => "FAT Filesystem",
            BinaryFormat::Ext4Image => "ext4 Filesystem",
            BinaryFormat::NtfsImage => "NTFS Filesystem",
            BinaryFormat::Gpt => "GPT Partition Table",
            BinaryFormat::Fu => "MBR Partition Table",
            BinaryFormat::Unknown => "Unknown",
        }
    }
}


pub fn dmx(data: &[u8]) -> BinaryFormat {
    if data.len() < 16 { return BinaryFormat::Unknown; }

    
    if data[0] == 0x7F && data[1] == b'E' && data[2] == b'L' && data[3] == b'F' {
        return if data[4] == 2 { BinaryFormat::Elf64 } else { BinaryFormat::Elf32 };
    }

    
    if data[0] == b'M' && data[1] == b'Z' && data.len() >= 64 {
        
        let ewm = u32::from_le_bytes([data[0x3C], data[0x3D], data[0x3E], data[0x3F]]) as usize;
        if ewm + 6 < data.len() && data[ewm] == b'P' && data[ewm + 1] == b'E' {
            
            let gld = ewm + 24;
            if gld + 2 <= data.len() {
                let nnp = u16::from_le_bytes([data[gld], data[gld + 1]]);
                return if nnp == 0x020B { BinaryFormat::Pe64 } else { BinaryFormat::Pe32 };
            }
            return BinaryFormat::Pe32;
        }
    }

    
    if data.len() >= 4 {
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic == 0xFEEDFACF || magic == 0xCFFAEDFE {
            return BinaryFormat::MachO64;
        }
    }

    
    if data.len() >= 520 && &data[512..520] == b"EFI PART" {
        return BinaryFormat::Gpt;
    }

    
    if data.len() >= 512 && data[510] == 0x55 && data[511] == 0xAA {
        
        if data.len() >= 62 && (data[54..62] == *b"FAT12   " || data[54..62] == *b"FAT16   ") {
            return BinaryFormat::FatImage;
        }
        if data.len() >= 90 && data[82..90] == *b"FAT32   " {
            return BinaryFormat::FatImage;
        }
        return BinaryFormat::Fu;
    }

    
    if data.len() >= 11 && &data[3..11] == b"NTFS    " {
        return BinaryFormat::NtfsImage;
    }

    
    if data.len() >= 1082 {
        let lth = u16::from_le_bytes([data[1080], data[1081]]);
        if lth == 0xEF53 {
            return BinaryFormat::Ext4Image;
        }
    }

    BinaryFormat::Unknown
}





#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryArch {
    X86,
    X86_64,
    Arm32,
    Aarch64,
    Riscv32,
    Riscv64,
    Mips32,
    Mips64,
    Wasm,
    Unknown,
}

impl BinaryArch {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryArch::X86 => "x86",
            BinaryArch::X86_64 => "x86_64",
            BinaryArch::Arm32 => "ARM32",
            BinaryArch::Aarch64 => "AArch64",
            BinaryArch::Riscv32 => "RISC-V 32",
            BinaryArch::Riscv64 => "RISC-V 64",
            BinaryArch::Mips32 => "MIPS32",
            BinaryArch::Mips64 => "MIPS64",
            BinaryArch::Wasm => "WebAssembly",
            BinaryArch::Unknown => "Unknown",
        }
    }
}


pub fn ldt(data: &[u8]) -> BinaryArch {
    let format = dmx(data);

    match format {
        BinaryFormat::Elf32 | BinaryFormat::Elf64 => {
            if data.len() < 20 { return BinaryArch::Unknown; }
            let e_machine = u16::from_le_bytes([data[18], data[19]]);
            match e_machine {
                0x03 => BinaryArch::X86,
                0x3E => BinaryArch::X86_64,
                0x28 => BinaryArch::Arm32,
                0xB7 => BinaryArch::Aarch64,
                0xF3 => if data[4] == 2 { BinaryArch::Riscv64 } else { BinaryArch::Riscv32 },
                0x08 => BinaryArch::Mips32,
                _ => BinaryArch::Unknown,
            }
        }
        BinaryFormat::Pe32 => BinaryArch::X86,
        BinaryFormat::Pe64 => BinaryArch::X86_64,
        BinaryFormat::MachO64 => {
            if data.len() >= 8 {
                let kyt = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                match kyt {
                    0x01000007 | 7 => BinaryArch::X86_64,
                    0x0100000C | 12 => BinaryArch::Aarch64,
                    _ => BinaryArch::Unknown,
                }
            } else {
                BinaryArch::Unknown
            }
        }
        _ => BinaryArch::Unknown,
    }
}






#[derive(Clone)]
pub struct Hf {
    pub format: BinaryFormat,
    pub arch: BinaryArch,
    pub size_bytes: usize,
    
    pub sections: Vec<Uz>,
    
    pub syscalls_found: Vec<String>,
    
    pub interesting_strings: Vec<String>,
    
    pub translatable: bool,
    
    pub rv_disasm_preview: String,
    
    pub security_notes: Vec<String>,
}

#[derive(Clone)]
pub struct Uz {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub flags: String,
}


pub fn jvu(data: &[u8]) -> Hf {
    let format = dmx(data);
    let arch = ldt(data);
    let sections = ltt(data, format);
    let strings = fvx(data);
    let security = jxv(data, format, arch);

    
    let (translatable, rv_disasm, syscalls) = match format {
        BinaryFormat::Elf64 | BinaryFormat::Elf32 => {
            pob(data)
        }
        _ => (false, String::new(), Vec::new()),
    };

    Hf {
        format,
        arch,
        size_bytes: data.len(),
        sections,
        syscalls_found: syscalls,
        interesting_strings: strings,
        translatable,
        rv_disasm_preview: rv_disasm,
        security_notes: security,
    }
}


fn ltt(data: &[u8], format: BinaryFormat) -> Vec<Uz> {
    let mut sections = Vec::new();

    match format {
        BinaryFormat::Elf64 => {
            if data.len() < 64 { return sections; }

            
            let jfw = u64::from_le_bytes([
                data[40], data[41], data[42], data[43],
                data[44], data[45], data[46], data[47],
            ]) as usize;

            
            let guq = u16::from_le_bytes([data[58], data[59]]) as usize;
            
            let jfv = u16::from_le_bytes([data[60], data[61]]) as usize;

            if jfw == 0 || guq < 64 || jfv > 100 { return sections; }

            for i in 0..jfv.min(50) {
                let base = jfw + i * guq;
                if base + 64 > data.len() { break; }

                let sh_type = u32::from_le_bytes([
                    data[base + 4], data[base + 5], data[base + 6], data[base + 7]]);
                let fam = u64::from_le_bytes([
                    data[base + 8], data[base + 9], data[base + 10], data[base + 11],
                    data[base + 12], data[base + 13], data[base + 14], data[base + 15]]);
                let jfx = u64::from_le_bytes([
                    data[base + 24], data[base + 25], data[base + 26], data[base + 27],
                    data[base + 28], data[base + 29], data[base + 30], data[base + 31]]) as usize;
                let jfy = u64::from_le_bytes([
                    data[base + 32], data[base + 33], data[base + 34], data[base + 35],
                    data[base + 36], data[base + 37], data[base + 38], data[base + 39]]) as usize;

                let ws = match sh_type {
                    0 => continue, 
                    1 => "PROGBITS",
                    2 => "SYMTAB",
                    3 => "STRTAB",
                    4 => "RELA",
                    5 => "HASH",
                    6 => "DYNAMIC",
                    7 => "NOTE",
                    8 => "NOBITS",
                    _ => "OTHER",
                };

                let mut cxx = String::new();
                if fam & 1 != 0 { cxx.push('W'); }
                if fam & 2 != 0 { cxx.push('A'); }
                if fam & 4 != 0 { cxx.push('X'); }

                sections.push(Uz {
                    name: String::from(ws),
                    offset: jfx,
                    size: jfy,
                    flags: cxx,
                });
            }
        }
        _ => {}
    }

    sections
}


fn fvx(data: &[u8]) -> Vec<String> {
    let mut strings = Vec::new();
    let mut current = String::new();

    for &byte in data.iter().take(64 * 1024) { 
        if byte >= 0x20 && byte < 0x7F {
            current.push(byte as char);
        } else {
            if current.len() >= 6 {
                
                let gj = current.to_ascii_lowercase();
                let mqy = gj.contains("http")
                    || gj.contains("password")
                    || gj.contains("key")
                    || gj.contains("token")
                    || gj.contains("secret")
                    || gj.contains("root")
                    || gj.contains("admin")
                    || gj.contains("linux")
                    || gj.contains("android")
                    || gj.contains("error")
                    || gj.contains("/dev/")
                    || gj.contains("/proc/")
                    || gj.contains("/sys/")
                    || gj.contains(".so")
                    || gj.contains(".dll")
                    || (current.len() >= 20); 

                if mqy && strings.len() < 50 {
                    strings.push(current.clone());
                }
            }
            current.clear();
        }
    }

    strings
}


fn jxv(data: &[u8], format: BinaryFormat, arch: BinaryArch) -> Vec<String> {
    let mut notes = Vec::new();

    match format {
        BinaryFormat::Elf64 | BinaryFormat::Elf32 => {
            
            if mjj(data) {
                notes.push(String::from("WARN: Executable stack detected (NX disabled)"));
            }

            
            if !mka(data) {
                notes.push(String::from("NOTE: No RELRO — GOT/PLT writable"));
            }

            
            let mkf = data.windows(4).any(|w| {
                w == [0x02, 0x00, 0x00, 0x00] 
            });
            if !mkf {
                notes.push(String::from("INFO: Likely stripped (no symbol table)"));
            }
        }
        BinaryFormat::Pe32 | BinaryFormat::Pe64 => {
            notes.push(format!("PE binary ({})", arch.as_str()));
            
        }
        _ => {}
    }

    notes
}

fn mjj(data: &[u8]) -> bool {
    
    if data.len() < 64 { return false; }
    let nv = u64::from_le_bytes([
        data[32], data[33], data[34], data[35],
        data[36], data[37], data[38], data[39],
    ]) as usize;
    let ium = u16::from_le_bytes([data[54], data[55]]) as usize;
    let ntz = u16::from_le_bytes([data[56], data[57]]) as usize;

    if nv == 0 || ium < 56 { return false; }

    for i in 0..ntz.min(20) {
        let base = nv + i * ium;
        if base + 8 > data.len() { break; }
        let p_type = u32::from_le_bytes([data[base], data[base+1], data[base+2], data[base+3]]);
        if p_type == 0x6474E551 { 
            let p_flags = u32::from_le_bytes([data[base+4], data[base+5], data[base+6], data[base+7]]);
            return p_flags & 1 != 0; 
        }
    }
    false
}

fn mka(_data: &[u8]) -> bool {
    
    
    true 
}






fn pob(data: &[u8]) -> (bool, String, Vec<String>) {
    
    match crate::riscv_translator::jon(data) {
        Ok(disasm) => {
            let mut syscalls = Vec::new();

            
            for line in disasm.lines() {
                if line.contains("ECALL") || line.contains("SYSCALL") || line.contains("SVC") {
                    syscalls.push(String::from(line.trim()));
                }
            }

            
            let dww: String = disasm.lines()
                .take(40)
                .collect::<Vec<&str>>()
                .join("\n");

            (true, dww, syscalls)
        }
        Err(_) => (false, String::new(), Vec::new()),
    }
}






#[derive(Clone)]
pub struct Kj {
    pub index: u8,
    pub kind: String,
    pub start_lba: u64,
    pub cqs: u64,
    pub size_mb: u64,
    pub bootable: bool,
}


pub fn nqx(data: &[u8]) -> Vec<Kj> {
    let mut au = Vec::new();

    if data.len() < 512 { return au; }

    
    if data.len() >= 1024 && &data[512..520] == b"EFI PART" {
        gmj(data, &mut au);
    }
    
    else if data[510] == 0x55 && data[511] == 0xAA {
        gmm(data, &mut au);
    }

    au
}

fn gmm(data: &[u8], au: &mut Vec<Kj>) {
    
    for i in 0..4u8 {
        let base = 446 + i as usize * 16;
        if base + 16 > data.len() { break; }

        let status = data[base];
        let ptype = data[base + 4];
        let start_lba = u32::from_le_bytes([
            data[base + 8], data[base + 9], data[base + 10], data[base + 11]
        ]) as u64;
        let cqs = u32::from_le_bytes([
            data[base + 12], data[base + 13], data[base + 14], data[base + 15]
        ]) as u64;

        if ptype == 0 || cqs == 0 { continue; }

        let kind = match ptype {
            0x01 => "FAT12",
            0x04 | 0x06 | 0x0E => "FAT16",
            0x0B | 0x0C => "FAT32",
            0x07 => "NTFS/exFAT",
            0x82 => "Linux Swap",
            0x83 => "Linux",
            0xEE => "GPT Protective",
            0xEF => "EFI System",
            _ => "Unknown",
        };

        au.push(Kj {
            index: i,
            kind: String::from(kind),
            start_lba,
            cqs,
            size_mb: cqs * 512 / (1024 * 1024),
            bootable: status == 0x80,
        });
    }
}

fn gmj(data: &[u8], au: &mut Vec<Kj>) {
    if data.len() < 1024 { return; }

    
    let dvn = u32::from_le_bytes([data[592], data[593], data[594], data[595]]) as usize;
    let oi = u32::from_le_bytes([data[596], data[597], data[598], data[599]]) as usize;
    let fuz = u64::from_le_bytes([
        data[584], data[585], data[586], data[587],
        data[588], data[589], data[590], data[591],
    ]);

    let lql = (fuz * 512) as usize;
    if oi < 128 { return; }

    for i in 0..dvn.min(32) {
        let base = lql + i * oi;
        if base + 128 > data.len() { break; }

        
        let type_guid = &data[base..base + 16];
        if type_guid.iter().all(|&b| b == 0) { continue; }

        let start_lba = u64::from_le_bytes([
            data[base + 32], data[base + 33], data[base + 34], data[base + 35],
            data[base + 36], data[base + 37], data[base + 38], data[base + 39],
        ]);
        let end_lba = u64::from_le_bytes([
            data[base + 40], data[base + 41], data[base + 42], data[base + 43],
            data[base + 44], data[base + 45], data[base + 46], data[base + 47],
        ]);

        let cqs = end_lba.saturating_sub(start_lba) + 1;

        
        let kind = mnn(type_guid);

        au.push(Kj {
            index: i as u8,
            kind,
            start_lba,
            cqs,
            size_mb: cqs * 512 / (1024 * 1024),
            bootable: false,
        });
    }
}

fn mnn(guid: &[u8]) -> String {
    
    
    if guid[0] == 0x28 && guid[1] == 0x73 && guid[2] == 0x2A && guid[3] == 0xC1 {
        return String::from("EFI System");
    }
    
    if guid[0] == 0xAF && guid[1] == 0x3D && guid[2] == 0xC6 && guid[3] == 0x0F {
        return String::from("Linux");
    }
    
    if guid[0] == 0x6D && guid[1] == 0xFD && guid[2] == 0x57 && guid[3] == 0x06 {
        return String::from("Linux Swap");
    }
    
    if guid[0] == 0xA2 && guid[1] == 0xA0 && guid[2] == 0xD0 && guid[3] == 0xEB {
        return String::from("Microsoft Basic Data");
    }
    String::from("Unknown")
}





impl Hf {
    pub fn format_report(&self) -> String {
        let mut j = String::new();

        j.push_str("\x01C╔══════════════════════════════════════════════════════════╗\n");
        j.push_str("║         JARVIS Binary Intelligence Report                ║\n");
        j.push_str("╚══════════════════════════════════════════════════════════╝\x01W\n\n");

        j.push_str(&format!("\x01Y[Format]\x01W {} ({})\n", self.format.as_str(), self.arch.as_str()));
        j.push_str(&format!("\x01Y[Size]\x01W {} bytes ({} KB)\n\n", self.size_bytes, self.size_bytes / 1024));

        
        if !self.sections.is_empty() {
            j.push_str("\x01Y[Sections]\x01W\n");
            for lx in &self.sections {
                j.push_str(&format!("  {:12} off=0x{:08X} size=0x{:06X} [{}]\n",
                    lx.name, lx.offset, lx.size, lx.flags));
            }
            j.push('\n');
        }

        
        if self.translatable {
            j.push_str("\x01G[RISC-V Translation]\x01W OK — binary decoded into universal IR\n");
            if !self.syscalls_found.is_empty() {
                j.push_str(&format!("  Syscalls detected: {}\n", self.syscalls_found.len()));
                for dr in self.syscalls_found.iter().take(10) {
                    j.push_str(&format!("    {}\n", dr));
                }
            }
            if !self.rv_disasm_preview.is_empty() {
                j.push_str("\n\x01C  --- Disassembly Preview ---\x01W\n");
                for line in self.rv_disasm_preview.lines().take(20) {
                    j.push_str(&format!("  {}\n", line));
                }
                j.push('\n');
            }
        } else if matches!(self.format, BinaryFormat::Elf32 | BinaryFormat::Elf64) {
            j.push_str("\x01R[RISC-V Translation]\x01W Failed — unsupported arch or corrupted\n\n");
        }

        
        if !self.interesting_strings.is_empty() {
            j.push_str(&format!("\x01Y[Interesting Strings]\x01W ({} found)\n", self.interesting_strings.len()));
            for uz in self.interesting_strings.iter().take(15) {
                j.push_str(&format!("  \"{}\"\n", uz));
            }
            j.push('\n');
        }

        
        if !self.security_notes.is_empty() {
            j.push_str("\x01Y[Security Assessment]\x01W\n");
            for note in &self.security_notes {
                j.push_str(&format!("  {}\n", note));
            }
        }

        j
    }
}


pub fn lxp(au: &[Kj]) -> String {
    let mut j = String::new();
    j.push_str("\x01C═══ Partition Table ═══\x01W\n");
    for aa in au {
        j.push_str(&format!("  #{}: {} — start=LBA {} size={} MB {}\n",
            aa.index, aa.kind, aa.start_lba, aa.size_mb,
            if aa.bootable { "[BOOT]" } else { "" }));
    }
    j
}
