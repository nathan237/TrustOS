








pub mod elf_parser;
pub mod disasm;
pub mod xrefs;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

pub use elf_parser::{Lz, Ct, Cy, Gz, Lo, Mz, Ma, Rw};
pub use disasm::Bj;
pub use xrefs::{XrefDatabase, Er, XrefType, Hp};




pub struct Hg {
    
    pub data: Vec<u8>,
    
    pub elf: Lz,
    
    pub instructions: Vec<Bj>,
    
    pub xrefs: XrefDatabase,
}

impl Hg {
    
    pub fn qkv(&self, offset: usize) -> Option<String> {
        if offset >= self.data.len() {
            return None;
        }

        let end = (offset + 16).min(self.data.len());
        let df = &self.data[offset..end];

        let mut ga = String::new();
        let mut ascii = String::new();

        for (i, &b) in df.iter().enumerate() {
            if i == 8 { ga.push(' '); }
            ga.push_str(&format!("{:02X} ", b));
            ascii.push(if b >= 0x20 && b < 0x7F { b as char } else { '.' });
        }

        
        let padding = 16 - df.len();
        for i in 0..padding {
            if df.len() + i == 8 { ga.push(' '); }
            ga.push_str("   ");
        }

        Some(format!("{:08X}  {}|{}|", offset, ga, ascii))
    }

    
    pub fn instruction_at(&self, addr: u64) -> Option<&Bj> {
        self.instructions.iter().find(|i| i.address == addr)
    }

    
    pub fn qlp(&self, start: u64, end: u64) -> Vec<&Bj> {
        self.instructions.iter()
            .filter(|i| i.address >= start && i.address < end)
            .collect()
    }

    
    pub fn qvb(&self, offset: u64) -> Option<&Ct> {
        self.elf.sections.iter().find(|j| {
            offset >= j.offset && offset < j.offset + j.size
        })
    }

    
    pub fn offset_to_vaddr(&self, offset: u64) -> Option<u64> {
        for rx in &self.elf.programs {
            if rx.p_type == elf_parser::JM_
                && offset >= rx.offset
                && offset < rx.offset + rx.filesz
            {
                return Some(rx.vaddr + (offset - rx.offset));
            }
        }
        None
    }

    
    pub fn vaddr_to_offset(&self, vaddr: u64) -> Option<u64> {
        for rx in &self.elf.programs {
            if rx.p_type == elf_parser::JM_
                && vaddr >= rx.vaddr
                && vaddr < rx.vaddr + rx.memsz
            {
                return Some(rx.offset + (vaddr - rx.vaddr));
            }
        }
        None
    }

    
    pub fn pyy(&self, vaddr: u64, len: usize) -> Option<&[u8]> {
        let offset = self.vaddr_to_offset(vaddr)? as usize;
        if offset + len <= self.data.len() {
            Some(&self.data[offset..offset + len])
        } else {
            None
        }
    }

    
    pub fn summary(&self) -> String {
        format!(
            "{} | {} | {} | {} bytes | {} sections | {} symbols | {} instructions | {}",
            self.elf.info.class,
            self.elf.info.machine,
            self.elf.info.elf_type,
            self.data.len(),
            self.elf.sections.len(),
            self.elf.symbols.len() + self.elf.dynamic_symbols.len(),
            self.instructions.len(),
            self.xrefs.summary(),
        )
    }
}




pub fn jvs(data: &[u8]) -> Result<Hg, &'static str> {
    
    let elf = elf_parser::nqd(data)?;

    
    let mut ctl = Vec::new();

    
    let code_sections = elf.code_sections();

    if code_sections.is_empty() {
        
        for rx in &elf.programs {
            if rx.p_type == elf_parser::JM_ && (rx.flags & 1) != 0 {
                let start = rx.offset as usize;
                let size = rx.filesz as usize;
                if start + size <= data.len() {
                    let code = &data[start..start + size];
                    let mut disasm = disasm::Disassembler::new(code, rx.vaddr);
                    let mut btl = disasm.disassemble_all();
                    ctl.append(&mut btl);
                }
            }
        }
    } else {
        for section in &code_sections {
            let start = section.offset as usize;
            let size = section.size as usize;
            if start + size <= data.len() && size > 0 {
                let code = &data[start..start + size];
                let mut disasm = disasm::Disassembler::new(code, section.addr);
                let mut btl = disasm.disassemble_all();
                ctl.append(&mut btl);
            }
        }
    }

    
    ctl.sort_by_key(|i| i.address);

    
    disasm::jwh(&mut ctl, &elf.addr_to_symbol);

    
    let xrefs = xrefs::XrefDatabase::ker(&ctl, &elf.addr_to_symbol);

    Ok(Hg {
        data: data.to_vec(),
        elf,
        instructions: ctl,
        xrefs,
    })
}


pub fn msm(data: &[u8]) -> bool {
    data.len() >= 4 && &data[0..4] == b"\x7FELF"
}


pub fn hfe(path: &str) -> Result<Hg, &'static str> {
    let fd = crate::vfs::open(path, crate::vfs::OpenFlags(crate::vfs::OpenFlags::PM_))
        .map_err(|_| "Failed to open file")?;

    let stat = crate::vfs::stat(path).map_err(|_| "Failed to stat file")?;
    let size = stat.size as usize;

    if size > 32 * 1024 * 1024 {
        crate::vfs::close(fd).ok();
        return Err("File too large (>32MB)");
    }

    let mut data = alloc::vec![0u8; size];
    crate::vfs::read(fd, &mut data).map_err(|_| "Failed to read file")?;
    crate::vfs::close(fd).ok();

    jvs(&data)
}
