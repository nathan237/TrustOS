//! ELF Executable Loader
//!
//! Parses and loads ELF64 executables for userspace execution.

use alloc::vec::Vec;
use alloc::string::String;

/// ELF magic number
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// ELF class
const ELFCLASS64: u8 = 2;

/// ELF data encoding
const ELFDATA2LSB: u8 = 1; // Little endian

/// ELF type
const ET_EXECUTE: u16 = 2;    // Executable
const ET_DYN: u16 = 3;     // Shared object (PIE)

/// ELF machine type
const EM_X86_64: u16 = 62;

/// Program header types
const PT_NULL: u32 = 0;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const PT_LOAD: u32 = 1;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const PT_DYNAMIC: u32 = 2;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const PT_INTERP: u32 = 3;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const PT_NOTE: u32 = 4;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const PT_PHDR: u32 = 6;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const PT_GNU_RELRO: u32 = 0x6474e552;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const PT_GNU_STACK: u32 = 0x6474e551;

/// Program header flags
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PF_X: u32 = 1; // Execute
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PF_W: u32 = 2; // Write
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PF_R: u32 = 4; // Read

/// Dynamic section tag types
const DT_NULL: i64 = 0;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_NEEDED: i64 = 1;     // Name of needed library
const DT_PLTRELSZ: i64 = 2;  // Bytes of PLT relocs
const DT_PLTGOT: i64 = 3;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_HASH: i64 = 4;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_STRTAB: i64 = 5;     // String table offset
const DT_SYMTAB: i64 = 6;    // Symbol table offset
const DT_RELA: i64 = 7;      // Rela relocs
const DT_RELASZ: i64 = 8;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_RELAENT: i64 = 9;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_STRSZ: i64 = 10;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_SYMENT: i64 = 11;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_INITIALIZE: i64 = 12;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_FINI: i64 = 13;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_SONAME: i64 = 14;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_RPATH: i64 = 15;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_SYMBOLIC: i64 = 16;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_RELATIVE: i64 = 17;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_RELSZ: i64 = 18;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_RELENT: i64 = 19;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_PLTREL: i64 = 20;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_DEBUG_DT: i64 = 21;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_TEXTREL: i64 = 22;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_JMPREL: i64 = 23;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_INITIALIZE_ARRAY: i64 = 25;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_FINI_ARRAY: i64 = 26;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_INITIALIZE_ARRAYSZ: i64 = 27;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_FINI_ARRAYSZ: i64 = 28;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_FLAGS: i64 = 30;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DT_FLAGS_1: i64 = 0x6ffffffb;

/// Relocation types for x86_64
const R_X86_64_NONE: u32 = 0;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const R_X86_64_64: u32 = 1;        // S + A
const R_X86_64_GLOB_DAT: u32 = 6;  // S
const R_X86_64_JUMP_SLOT: u32 = 7; // S
const R_X86_64_RELATIVE: u32 = 8;  // B + A
const R_X86_64_IRELATIVE: u32 = 37;

/// ELF64 dynamic section entry
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
// Public structure — visible outside this module.
pub struct Elf64Dyn {
    pub d_tag: i64,
    pub d_val: u64,
}

/// ELF64 symbol table entry
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
// Public structure — visible outside this module.
pub struct Elf64Sym {
    pub st_name: u32,
    pub st_information: u8,
    pub st_other: u8,
    pub st_shndx: u16,
    pub st_value: u64,
    pub st_size: u64,
}

/// ELF64 Rela relocation entry
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
// Public structure — visible outside this module.
pub struct Elf64Rela {
    pub r_offset: u64,
    pub r_info: u64,
    pub r_addend: i64,
}

// Implementation block — defines methods for the type above.
impl Elf64Rela {
        // Public function — callable from other modules.
pub fn sym_idx(&self) -> u32 { (self.r_info >> 32) as u32 }
        // Public function — callable from other modules.
pub fn rel_type(&self) -> u32 { (self.r_info & 0xFFFF_FFFF) as u32 }
}

/// ELF64 file header
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
// Public structure — visible outside this module.
pub struct Elf64Header {
    pub e_ident: [u8; 16],      // Magic number and other info
    pub e_type: u16,            // Object file type
    pub e_machine: u16,         // Architecture
    pub e_version: u32,         // Object file version
    pub e_entry: u64,           // Entry point virtual address
    pub e_phoff: u64,           // Program header table file offset
    pub e_shoff: u64,           // Section header table file offset
    pub e_flags: u32,           // Processor-specific flags
    pub e_ehsize: u16,          // ELF header size
    pub e_phentsize: u16,       // Program header table entry size
    pub e_phnum: u16,           // Program header table entry count
    pub e_shentsize: u16,       // Section header table entry size
    pub e_shnum: u16,           // Section header table entry count
    pub e_shstrndx: u16,        // Section header string table index
}

// Implementation block — defines methods for the type above.
impl Elf64Header {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SIZE: usize = 64;
    
    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> Option<&Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        
        let header = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(data.as_ptr() as *const Self) };
        
        // Validate magic
        if header.e_ident[0..4] != ELF_MAGIC {
            return None;
        }
        
        // Check 64-bit
        if header.e_ident[4] != ELFCLASS64 {
            return None;
        }
        
        // Check little endian
        if header.e_ident[5] != ELFDATA2LSB {
            return None;
        }
        
        // Check x86_64
        if header.e_machine != EM_X86_64 {
            return None;
        }
        
        Some(header)
    }
    
    /// Check if this is an executable
    pub fn is_executable(&self) -> bool {
        self.e_type == ET_EXECUTE || self.e_type == ET_DYN
    }
}

/// ELF64 program header
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
// Public structure — visible outside this module.
pub struct Elf64Phdr {
    pub p_type: u32,        // Segment type
    pub p_flags: u32,       // Segment flags
    pub p_offset: u64,      // Segment file offset
    pub p_vaddr: u64,       // Segment virtual address
    pub p_paddr: u64,       // Segment physical address
    pub p_filesz: u64,      // Segment size in file
    pub p_memsz: u64,       // Segment size in memory
    pub p_align: u64,       // Segment alignment
}

// Implementation block — defines methods for the type above.
impl Elf64Phdr {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SIZE: usize = 56;
    
    /// Check if segment is loadable
    pub fn is_load(&self) -> bool {
        self.p_type == PT_LOAD
    }
    
    /// Check if executable
    pub fn is_executable(&self) -> bool {
        (self.p_flags & PF_X) != 0
    }
    
    /// Check if writable
    pub fn is_writable(&self) -> bool {
        (self.p_flags & PF_W) != 0
    }
    
    /// Check if readable
    pub fn is_readable(&self) -> bool {
        (self.p_flags & PF_R) != 0
    }
}

/// ELF64 section header
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
// Public structure — visible outside this module.
pub struct Elf64Shdr {
    pub sh_name: u32,       // Section name (string table index)
    pub sh_type: u32,       // Section type
    pub sh_flags: u64,      // Section flags
    pub sh_addr: u64,       // Section virtual address
    pub sh_offset: u64,     // Section file offset
    pub sh_size: u64,       // Section size
    pub sh_link: u32,       // Link to another section
    pub sh_info: u32,       // Additional section info
    pub sh_addralign: u64,  // Section alignment
    pub sh_entsize: u64,    // Entry size if section holds table
}

/// Loaded segment info
#[derive(Clone, Debug)]
// Public structure — visible outside this module.
pub struct LoadedSegment {
    pub vaddr: u64,
    pub size: u64,
    pub flags: u32,
    pub data: Vec<u8>,
}

/// Dynamic linking info parsed from PT_DYNAMIC
#[derive(Clone, Debug, Default)]
// Public structure — visible outside this module.
pub struct DynamicInfo {
    /// Interpreter path from PT_INTERP (e.g. "/lib/ld-linux-x86-64.so.2")
    pub interp: Option<String>,
    /// Needed shared libraries (DT_NEEDED)
    pub needed_libs: Vec<String>,
    /// Rela relocation table (file offset, count)
    pub rela_offset: u64,
    pub rela_count: usize,
    /// JMPREL (PLT relocations)
    pub jmprel_offset: u64,
    pub jmprel_count: usize,
    /// Symbol table file offset
    pub symtab_offset: u64,
    /// String table file offset + size
    pub strtab_offset: u64,
    pub strtab_size: usize,
    /// INIT / FINI addresses (virtual)
    pub init_addr: u64,
    pub fini_addr: u64,
    /// INIT_ARRAY / FINI_ARRAY
    pub init_array_addr: u64,
    pub init_array_size: usize,
    pub fini_array_addr: u64,
    pub fini_array_size: usize,
    /// FLAGS
    pub flags: u64,
    pub flags_1: u64,
    /// Has PT_DYNAMIC at all?
    pub has_dynamic: bool,
}

/// Loaded ELF info
#[derive(Clone, Debug)]
// Public structure — visible outside this module.
pub struct LoadedElf {
    pub entry_point: u64,
    pub segments: Vec<LoadedSegment>,
    pub min_vaddr: u64,
    pub max_vaddr: u64,
    /// Base address offset for PIE executables (ET_DYN)
    pub base_addr: u64,
    /// Whether this is a PIE/shared object
    pub is_pie: bool,
    /// Dynamic linking information
    pub dynamic: DynamicInfo,
    /// Relocation entries (already parsed)
    pub relocations: Vec<RelocationEntry>,
}

/// A parsed relocation
#[derive(Clone, Debug)]
// Public structure — visible outside this module.
pub struct RelocationEntry {
    pub offset: u64,
    pub rel_type: u32,
    pub sym_idx: u32,
    pub addend: i64,
}

/// ELF loading errors
#[derive(Clone, Copy, Debug)]
// Enumeration — a type that can be one of several variants.
pub enum ElfError {
    InvalidMagic,
    InvalidClass,
    InvalidMachine,
    NotExecutable,
    InvalidProgramHeader,
    IoError,
    TooLarge,
    OutOfMemory,
}

pub // Type alias — gives an existing type a new name for clarity.
type ElfResult<T> = Result<T, ElfError>;

/// Load an ELF file from a path
pub fn load_from_path(path: &str) -> ElfResult<LoadedElf> {
    // Open the file
    let fd = crate::vfs::open(path, crate::vfs::OpenFlags(crate::vfs::OpenFlags::O_RDONLY))
        .map_err(|_| ElfError::IoError)?;
    
    // Get file size
    let stat = crate::vfs::stat(path).map_err(|_| ElfError::IoError)?;
    let size = stat.size as usize;
    
    if size > 16 * 1024 * 1024 {  // 16 MB limit
        crate::vfs::close(fd).ok();
        return Err(ElfError::TooLarge);
    }
    
    // Read entire file
    let mut data = alloc::vec![0u8; size];
    crate::vfs::read(fd, &mut data).map_err(|_| ElfError::IoError)?;
    crate::vfs::close(fd).ok();
    
    // Parse and load
    load_from_bytes(&data)
}

/// Load an ELF from bytes — supports static, PIE, and dynamic executables
pub fn load_from_bytes(data: &[u8]) -> ElfResult<LoadedElf> {
    // Parse header
    let header = Elf64Header::from_bytes(data)
        .ok_or(ElfError::InvalidMagic)?;
    
    if !header.is_executable() {
        return Err(ElfError::NotExecutable);
    }
    
    let is_pie = header.e_type == ET_DYN;
    // PIE executables are loaded at a fixed base; static ELFs at their linked address
    let base_addr: u64 = if is_pie { 0x0040_0000 } else { 0 };
    
    crate::log_debug!("[ELF] Loading {} executable, entry: {:#x}, base: {:#x}",
        if is_pie { "PIE" } else { "static" }, header.e_entry, base_addr);
    
    let mut segments = Vec::new();
    let mut min_vaddr = u64::MAX;
    let mut max_vaddr = 0u64;
    let mut dynamic_information = DynamicInfo::default();
    let mut dynamic_phdr: Option<(u64, u64)> = None; // (offset, size)
    
    // Parse program headers
    let ph_offset = header.e_phoff as usize;
    let ph_size = header.e_phentsize as usize;
    let ph_count = header.e_phnum as usize;
    
    for i in 0..ph_count {
        let offset = ph_offset + i * ph_size;
        if offset + Elf64Phdr::SIZE > data.len() {
            return Err(ElfError::InvalidProgramHeader);
        }
        
        let phdr = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(data[offset..].as_ptr() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const Elf64Phdr) };
        
                // Pattern matching — Rust's exhaustive branching construct.
match phdr.p_type {
            PT_INTERP => {
                // Extract interpreter path
                let start = phdr.p_offset as usize;
                let end = start + phdr.p_filesz as usize;
                if end <= data.len() {
                    let interp_bytes = &data[start..end];
                    // Strip null terminator
                    let len = interp_bytes.iter().position(|&b| b == 0).unwrap_or(interp_bytes.len());
                    if let Ok(s) = core::str::from_utf8(&interp_bytes[..len]) {
                        dynamic_information.interp = Some(String::from(s));
                        crate::log_debug!("[ELF] PT_INTERP: {}", s);
                    }
                }
            }
            PT_DYNAMIC => {
                dynamic_information.has_dynamic = true;
                dynamic_phdr = Some((phdr.p_offset, phdr.p_filesz));
            }
            PT_LOAD => {
                let vaddr = phdr.p_vaddr + base_addr;
                crate::log_debug!("[ELF] LOAD segment: vaddr={:#x}, filesz={}, memsz={}, flags={:#x}",
                    vaddr, phdr.p_filesz, phdr.p_memsz, phdr.p_flags);
                
                if vaddr < min_vaddr { min_vaddr = vaddr; }
                if vaddr + phdr.p_memsz > max_vaddr { max_vaddr = vaddr + phdr.p_memsz; }
                
                let file_offset = phdr.p_offset as usize;
                let file_size = phdr.p_filesz as usize;
                let mem_size = phdr.p_memsz as usize;
                
                if file_offset + file_size > data.len() {
                    return Err(ElfError::InvalidProgramHeader);
                }
                
                let mut segment_data = alloc::vec![0u8; mem_size];
                segment_data[..file_size].copy_from_slice(&data[file_offset..file_offset + file_size]);
                
                segments.push(LoadedSegment {
                    vaddr,
                    size: phdr.p_memsz,
                    flags: phdr.p_flags,
                    data: segment_data,
                });
            }
            _ => {} // PT_NOTE, PT_GNU_STACK, PT_GNU_RELRO, etc.
        }
    }
    
    if segments.is_empty() {
        return Err(ElfError::InvalidProgramHeader);
    }
    
    // ── Parse PT_DYNAMIC section ──
    let mut relocations = Vec::new();
    if let Some((dyn_off, dyn_sz)) = dynamic_phdr {
        let start = dyn_off as usize;
        let end = start + dyn_sz as usize;
        if end <= data.len() {
            parse_dynamic(data, start, end, base_addr, &mut dynamic_information);
        }
        // Parse RELA relocations
        if dynamic_information.rela_count > 0 && (dynamic_information.rela_offset as usize) < data.len() {
            let rela_start = dynamic_information.rela_offset as usize;
            for i in 0..dynamic_information.rela_count {
                let off = rela_start + i * core::mem::size_of::<Elf64Rela>();
                if off + core::mem::size_of::<Elf64Rela>() > data.len() { break; }
                let rela = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(data[off..].as_ptr() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const Elf64Rela) };
                relocations.push(RelocationEntry {
                    offset: rela.r_offset,
                    rel_type: rela.rel_type(),
                    sym_idx: rela.sym_idx(),
                    addend: rela.r_addend,
                });
            }
        }
        // Parse JMPREL (PLT) relocations
        if dynamic_information.jmprel_count > 0 && (dynamic_information.jmprel_offset as usize) < data.len() {
            let jmp_start = dynamic_information.jmprel_offset as usize;
            for i in 0..dynamic_information.jmprel_count {
                let off = jmp_start + i * core::mem::size_of::<Elf64Rela>();
                if off + core::mem::size_of::<Elf64Rela>() > data.len() { break; }
                let rela = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(data[off..].as_ptr() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const Elf64Rela) };
                relocations.push(RelocationEntry {
                    offset: rela.r_offset,
                    rel_type: rela.rel_type(),
                    sym_idx: rela.sym_idx(),
                    addend: rela.r_addend,
                });
            }
        }
        crate::log_debug!("[ELF] Parsed {} relocations, {} needed libs",
            relocations.len(), dynamic_information.needed_libs.len());
    }
    
    Ok(LoadedElf {
        entry_point: header.e_entry + base_addr,
        segments,
        min_vaddr,
        max_vaddr,
        base_addr,
        is_pie,
        dynamic: dynamic_information,
        relocations,
    })
}

/// Parse the .dynamic section entries
fn parse_dynamic(data: &[u8], start: usize, end: usize, _base: u64, info: &mut DynamicInfo) {
    let entry_size = core::mem::size_of::<Elf64Dyn>();
    let mut rela_size: u64 = 0;
    let mut rela_ent: u64 = 0;
    let mut plt_relative_size: u64 = 0;
    let mut strtab_file_off: u64 = 0;
    let mut strtab_sz: u64 = 0;
    let mut needed_offsets: Vec<u64> = Vec::new();
    
    let mut off = start;
    while off + entry_size <= end {
        let dyn_entry = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(data[off..].as_ptr() as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const Elf64Dyn) };
                // Pattern matching — Rust's exhaustive branching construct.
match dyn_entry.d_tag {
            DT_NULL => break,
            DT_NEEDED => { needed_offsets.push(dyn_entry.d_val); }
            DT_STRTAB => { strtab_file_off = dyn_entry.d_val; }
            DT_STRSZ => { strtab_sz = dyn_entry.d_val; }
            DT_SYMTAB => { info.symtab_offset = dyn_entry.d_val; }
            DT_RELA => { info.rela_offset = dyn_entry.d_val; }
            DT_RELASZ => { rela_size = dyn_entry.d_val; }
            DT_RELAENT => { rela_ent = dyn_entry.d_val; }
            DT_JMPREL => { info.jmprel_offset = dyn_entry.d_val; }
            DT_PLTRELSZ => { plt_relative_size = dyn_entry.d_val; }
            DT_INITIALIZE => { info.init_addr = dyn_entry.d_val; }
            DT_FINI => { info.fini_addr = dyn_entry.d_val; }
            DT_INITIALIZE_ARRAY => { info.init_array_addr = dyn_entry.d_val; }
            DT_INITIALIZE_ARRAYSZ => { info.init_array_size = dyn_entry.d_val as usize; }
            DT_FINI_ARRAY => { info.fini_array_addr = dyn_entry.d_val; }
            DT_FINI_ARRAYSZ => { info.fini_array_size = dyn_entry.d_val as usize; }
            DT_FLAGS => { info.flags = dyn_entry.d_val; }
            DT_FLAGS_1 => { info.flags_1 = dyn_entry.d_val; }
            _ => {}
        }
        off += entry_size;
    }
    
    // Calculate relocation counts
    if rela_ent > 0 && rela_size > 0 {
        info.rela_count = (rela_size / rela_ent) as usize;
    }
    if plt_relative_size > 0 {
        let ent = if rela_ent > 0 { rela_ent } else { core::mem::size_of::<Elf64Rela>() as u64 };
        info.jmprel_count = (plt_relative_size / ent) as usize;
    }
    
    info.strtab_offset = strtab_file_off;
    info.strtab_size = strtab_sz as usize;
    
    // Resolve needed library names from string table
    // The strtab_file_off may be a virtual address; try to find it as a file offset
    // by looking in loaded segments. For simplicity, if it's within data range, use directly.
    let strtab_start = strtab_file_off as usize;
    if strtab_start < data.len() {
        for &name_off in &needed_offsets {
            let name_start = strtab_start + name_off as usize;
            if name_start < data.len() {
                let end_position = data[name_start..].iter().position(|&b| b == 0)
                    .unwrap_or(data.len() - name_start);
                if let Ok(name) = core::str::from_utf8(&data[name_start..name_start + end_position]) {
                    info.needed_libs.push(String::from(name));
                }
            }
        }
    }
}

/// Check if data is a valid ELF file
pub fn is_elf(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    data[0..4] == ELF_MAGIC
}

/// Get ELF file info without loading
pub fn get_information(data: &[u8]) -> ElfResult<(u64, usize)> {
    let header = Elf64Header::from_bytes(data)
        .ok_or(ElfError::InvalidMagic)?;
    
    Ok((header.e_entry, header.e_phnum as usize))
}
