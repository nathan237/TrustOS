//! TrustView — Binary Analysis Engine
//!
//! Main module that ties together ELF parsing, disassembly, cross-references,
//! and string extraction into a unified binary analysis pipeline.
//!
//! Usage:
//!   let analysis = binary_analysis::analyze(data).unwrap();
//!   // Access: analysis.elf, analysis.instructions, analysis.xrefs, etc.

pub mod elf_parser;
pub mod disasm;
pub mod xrefs;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

pub use elf_parser::{ElfAnalysis, Section, Symbol, ProgramHeader, DynamicEntry, Relocation, ExtractedString, ElfInfo};
pub use disasm::Instruction;
pub use xrefs::{XrefDatabase, Xref, XrefType, DetectedFunction};

// ──── Unified Analysis Result ──────────────────────────────────────────────

/// Complete binary analysis — everything needed for the viewer
pub struct BinaryFile {
    /// Raw binary data
    pub data: Vec<u8>,
    /// Parsed ELF structure
    pub elf: ElfAnalysis,
    /// Disassembled instructions (all code sections)
    pub instructions: Vec<Instruction>,
    /// Cross-reference database
    pub xrefs: XrefDatabase,
}

impl BinaryFile {
    /// Get a hex dump line for a given offset (16 bytes per line)
    pub fn hex_line(&self, offset: usize) -> Option<String> {
        if offset >= self.data.len() {
            return None;
        }

        let end = (offset + 16).min(self.data.len());
        let chunk = &self.data[offset..end];

        let mut hex = String::new();
        let mut ascii = String::new();

        for (i, &b) in chunk.iter().enumerate() {
            if i == 8 { hex.push(' '); }
            hex.push_str(&format!("{:02X} ", b));
            ascii.push(if b >= 0x20 && b < 0x7F { b as char } else { '.' });
        }

        // Pad if less than 16 bytes
        let padding = 16 - chunk.len();
        for i in 0..padding {
            if chunk.len() + i == 8 { hex.push(' '); }
            hex.push_str("   ");
        }

        Some(format!("{:08X}  {}|{}|", offset, hex, ascii))
    }

    /// Get instruction at or near an address
    pub fn instruction_at(&self, addr: u64) -> Option<&Instruction> {
        self.instructions.iter().find(|i| i.address == addr)
    }

    /// Get instructions in an address range
    pub fn instructions_in_range(&self, start: u64, end: u64) -> Vec<&Instruction> {
        self.instructions.iter()
            .filter(|i| i.address >= start && i.address < end)
            .collect()
    }

    /// Find the section containing a file offset
    pub fn section_for_offset(&self, offset: u64) -> Option<&Section> {
        self.elf.sections.iter().find(|s| {
            offset >= s.offset && offset < s.offset + s.size
        })
    }

    /// Map a file offset to a virtual address
    pub fn offset_to_vaddr(&self, offset: u64) -> Option<u64> {
        for phdr in &self.elf.programs {
            if phdr.p_type == elf_parser::PT_LOAD
                && offset >= phdr.offset
                && offset < phdr.offset + phdr.filesz
            {
                return Some(phdr.vaddr + (offset - phdr.offset));
            }
        }
        None
    }

    /// Map a virtual address to a file offset
    pub fn vaddr_to_offset(&self, vaddr: u64) -> Option<u64> {
        for phdr in &self.elf.programs {
            if phdr.p_type == elf_parser::PT_LOAD
                && vaddr >= phdr.vaddr
                && vaddr < phdr.vaddr + phdr.memsz
            {
                return Some(phdr.offset + (vaddr - phdr.vaddr));
            }
        }
        None
    }

    /// Get raw bytes at a virtual address
    pub fn bytes_at_vaddr(&self, vaddr: u64, len: usize) -> Option<&[u8]> {
        let offset = self.vaddr_to_offset(vaddr)? as usize;
        if offset + len <= self.data.len() {
            Some(&self.data[offset..offset + len])
        } else {
            None
        }
    }

    /// Summary string for status bar
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

// ──── Analysis Pipeline ────────────────────────────────────────────────────

/// Analyze a binary file — full pipeline
pub fn analyze(data: &[u8]) -> Result<BinaryFile, &'static str> {
    // Step 1: Parse ELF
    let elf = elf_parser::parse_elf(data)?;

    // Step 2: Disassemble all code sections
    let mut all_instructions = Vec::new();

    // Get code sections
    let code_sections = elf.code_sections();

    if code_sections.is_empty() {
        // Fallback: disassemble from entry point using program headers
        for phdr in &elf.programs {
            if phdr.p_type == elf_parser::PT_LOAD && (phdr.flags & 1) != 0 {
                let start = phdr.offset as usize;
                let size = phdr.filesz as usize;
                if start + size <= data.len() {
                    let code = &data[start..start + size];
                    let mut disasm = disasm::Disassembler::new(code, phdr.vaddr);
                    let mut insts = disasm.disassemble_all();
                    all_instructions.append(&mut insts);
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
                let mut insts = disasm.disassemble_all();
                all_instructions.append(&mut insts);
            }
        }
    }

    // Sort by address
    all_instructions.sort_by_key(|i| i.address);

    // Step 3: Annotate with symbols and syscall info
    disasm::annotate_instructions(&mut all_instructions, &elf.addr_to_symbol);

    // Step 4: Build cross-references
    let xrefs = xrefs::XrefDatabase::build(&all_instructions, &elf.addr_to_symbol);

    Ok(BinaryFile {
        data: data.to_vec(),
        elf,
        instructions: all_instructions,
        xrefs,
    })
}

/// Quick check if data looks like an ELF file
pub fn is_elf(data: &[u8]) -> bool {
    data.len() >= 4 && &data[0..4] == b"\x7FELF"
}

/// Analyze from a VFS path
pub fn analyze_path(path: &str) -> Result<BinaryFile, &'static str> {
    let fd = crate::vfs::open(path, crate::vfs::OpenFlags(crate::vfs::OpenFlags::O_RDONLY))
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

    analyze(&data)
}
