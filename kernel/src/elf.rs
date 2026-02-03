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
const ET_EXEC: u16 = 2;    // Executable
const ET_DYN: u16 = 3;     // Shared object (PIE)

/// ELF machine type
const EM_X86_64: u16 = 62;

/// Program header types
const PT_NULL: u32 = 0;
const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const PT_INTERP: u32 = 3;
const PT_NOTE: u32 = 4;
const PT_PHDR: u32 = 6;

/// Program header flags
pub const PF_X: u32 = 1; // Execute
pub const PF_W: u32 = 2; // Write
pub const PF_R: u32 = 4; // Read

/// ELF64 file header
#[repr(C)]
#[derive(Clone, Copy, Debug)]
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

impl Elf64Header {
    pub const SIZE: usize = 64;
    
    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> Option<&Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        
        let header = unsafe { &*(data.as_ptr() as *const Self) };
        
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
        self.e_type == ET_EXEC || self.e_type == ET_DYN
    }
}

/// ELF64 program header
#[repr(C)]
#[derive(Clone, Copy, Debug)]
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

impl Elf64Phdr {
    pub const SIZE: usize = 56;
    
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
#[derive(Clone, Copy, Debug)]
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
pub struct LoadedSegment {
    pub vaddr: u64,
    pub size: u64,
    pub flags: u32,
    pub data: Vec<u8>,
}

/// Loaded ELF info
#[derive(Clone, Debug)]
pub struct LoadedElf {
    pub entry_point: u64,
    pub segments: Vec<LoadedSegment>,
    pub min_vaddr: u64,
    pub max_vaddr: u64,
}

/// ELF loading errors
#[derive(Clone, Copy, Debug)]
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

pub type ElfResult<T> = Result<T, ElfError>;

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

/// Load an ELF from bytes
pub fn load_from_bytes(data: &[u8]) -> ElfResult<LoadedElf> {
    // Parse header
    let header = Elf64Header::from_bytes(data)
        .ok_or(ElfError::InvalidMagic)?;
    
    if !header.is_executable() {
        return Err(ElfError::NotExecutable);
    }
    
    crate::log_debug!("[ELF] Loading executable, entry: {:#x}", header.e_entry);
    
    let mut segments = Vec::new();
    let mut min_vaddr = u64::MAX;
    let mut max_vaddr = 0u64;
    
    // Parse program headers
    let ph_offset = header.e_phoff as usize;
    let ph_size = header.e_phentsize as usize;
    let ph_count = header.e_phnum as usize;
    
    for i in 0..ph_count {
        let offset = ph_offset + i * ph_size;
        if offset + Elf64Phdr::SIZE > data.len() {
            return Err(ElfError::InvalidProgramHeader);
        }
        
        let phdr = unsafe { &*(data[offset..].as_ptr() as *const Elf64Phdr) };
        
        if !phdr.is_load() {
            continue;
        }
        
        crate::log_debug!("[ELF] LOAD segment: vaddr={:#x}, filesz={}, memsz={}, flags={:#x}",
            phdr.p_vaddr, phdr.p_filesz, phdr.p_memsz, phdr.p_flags);
        
        // Track address range
        if phdr.p_vaddr < min_vaddr {
            min_vaddr = phdr.p_vaddr;
        }
        if phdr.p_vaddr + phdr.p_memsz > max_vaddr {
            max_vaddr = phdr.p_vaddr + phdr.p_memsz;
        }
        
        // Load segment data
        let file_offset = phdr.p_offset as usize;
        let file_size = phdr.p_filesz as usize;
        let mem_size = phdr.p_memsz as usize;
        
        if file_offset + file_size > data.len() {
            return Err(ElfError::InvalidProgramHeader);
        }
        
        // Create segment buffer (zero-initialized for BSS)
        let mut segment_data = alloc::vec![0u8; mem_size];
        segment_data[..file_size].copy_from_slice(&data[file_offset..file_offset + file_size]);
        
        segments.push(LoadedSegment {
            vaddr: phdr.p_vaddr,
            size: phdr.p_memsz,
            flags: phdr.p_flags,
            data: segment_data,
        });
    }
    
    if segments.is_empty() {
        return Err(ElfError::InvalidProgramHeader);
    }
    
    Ok(LoadedElf {
        entry_point: header.e_entry,
        segments,
        min_vaddr,
        max_vaddr,
    })
}

/// Check if data is a valid ELF file
pub fn is_elf(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    data[0..4] == ELF_MAGIC
}

/// Get ELF file info without loading
pub fn get_info(data: &[u8]) -> ElfResult<(u64, usize)> {
    let header = Elf64Header::from_bytes(data)
        .ok_or(ElfError::InvalidMagic)?;
    
    Ok((header.e_entry, header.e_phnum as usize))
}
