//! TrustView — Advanced ELF Section & Symbol Parser
//!
//! Parses ELF64 section headers, symbol tables, string tables,
//! dynamic entries, relocations, and PLT/GOT for binary analysis.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;

// ──── Section Header Types ─────────────────────────────────────────────────

pub const SHT_NULL: u32 = 0;
pub const SHT_PROGBITS: u32 = 1;
pub const SHT_SYMTAB: u32 = 2;
pub const SHT_STRTAB: u32 = 3;
pub const SHT_RELA: u32 = 4;
pub const SHT_HASH: u32 = 5;
pub const SHT_DYNAMIC: u32 = 6;
pub const SHT_NOTE: u32 = 7;
pub const SHT_NOBITS: u32 = 8;
pub const SHT_REL: u32 = 9;
pub const SHT_DYNSYM: u32 = 11;
pub const SHT_INIT_ARRAY: u32 = 14;
pub const SHT_FINI_ARRAY: u32 = 15;
pub const SHT_GNU_HASH: u32 = 0x6FFFFFF6;
pub const SHT_GNU_VERSYM: u32 = 0x6FFFFFFF;
pub const SHT_GNU_VERNEED: u32 = 0x6FFFFFFE;

// ──── Section Header Flags ─────────────────────────────────────────────────

pub const SHF_WRITE: u64 = 0x1;
pub const SHF_ALLOC: u64 = 0x2;
pub const SHF_EXECINSTR: u64 = 0x4;
pub const SHF_MERGE: u64 = 0x10;
pub const SHF_STRINGS: u64 = 0x20;

// ──── Symbol Binding & Type ────────────────────────────────────────────────

pub const STB_LOCAL: u8 = 0;
pub const STB_GLOBAL: u8 = 1;
pub const STB_WEAK: u8 = 2;

pub const STT_NOTYPE: u8 = 0;
pub const STT_OBJECT: u8 = 1;
pub const STT_FUNC: u8 = 2;
pub const STT_SECTION: u8 = 3;
pub const STT_FILE: u8 = 4;

// ──── Program Header Types ─────────────────────────────────────────────────

pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;
pub const PT_NOTE: u32 = 4;
pub const PT_GNU_EH_FRAME: u32 = 0x6474E550;
pub const PT_GNU_STACK: u32 = 0x6474E551;
pub const PT_GNU_RELRO: u32 = 0x6474E552;

// ──── Dynamic Entry Tags ───────────────────────────────────────────────────

pub const DT_NULL: i64 = 0;
pub const DT_NEEDED: i64 = 1;
pub const DT_PLTGOT: i64 = 3;
pub const DT_HASH: i64 = 4;
pub const DT_STRTAB: i64 = 5;
pub const DT_SYMTAB: i64 = 6;
pub const DT_RELA: i64 = 7;
pub const DT_RELASZ: i64 = 8;
pub const DT_RELAENT: i64 = 9;
pub const DT_STRSZ: i64 = 10;
pub const DT_INIT: i64 = 12;
pub const DT_FINI: i64 = 13;
pub const DT_SONAME: i64 = 14;
pub const DT_RPATH: i64 = 15;
pub const DT_SYMBOLIC: i64 = 16;
pub const DT_JMPREL: i64 = 23;
pub const DT_BIND_NOW: i64 = 24;
pub const DT_RUNPATH: i64 = 29;
pub const DT_FLAGS: i64 = 30;
pub const DT_GNU_HASH: i64 = 0x6FFFFEF5;
pub const DT_VERNEED: i64 = 0x6FFFFFFE;
pub const DT_VERNEEDNUM: i64 = 0x6FFFFFFF;

// ──── Parsed Structures ────────────────────────────────────────────────────

/// Parsed ELF section
#[derive(Debug, Clone)]
pub struct Section {
    /// Section index
    pub index: usize,
    /// Section name (from .shstrtab)
    pub name: String,
    /// Section type (SHT_*)
    pub sh_type: u32,
    /// Section flags (SHF_*)
    pub flags: u64,
    /// Virtual address
    pub addr: u64,
    /// File offset
    pub offset: u64,
    /// Section size
    pub size: u64,
    /// Link to another section
    pub link: u32,
    /// Additional info
    pub info: u32,
    /// Alignment
    pub addralign: u64,
    /// Entry size for table sections
    pub entsize: u64,
}

impl Section {
    pub fn type_name(&self) -> &'static str {
        match self.sh_type {
            SHT_NULL => "NULL",
            SHT_PROGBITS => "PROGBITS",
            SHT_SYMTAB => "SYMTAB",
            SHT_STRTAB => "STRTAB",
            SHT_RELA => "RELA",
            SHT_HASH => "HASH",
            SHT_DYNAMIC => "DYNAMIC",
            SHT_NOTE => "NOTE",
            SHT_NOBITS => "NOBITS",
            SHT_REL => "REL",
            SHT_DYNSYM => "DYNSYM",
            SHT_INIT_ARRAY => "INIT_ARRAY",
            SHT_FINI_ARRAY => "FINI_ARRAY",
            SHT_GNU_HASH => "GNU_HASH",
            SHT_GNU_VERSYM => "GNU_VERSYM",
            SHT_GNU_VERNEED => "GNU_VERNEED",
            _ => "UNKNOWN",
        }
    }

    pub fn flags_string(&self) -> String {
        let mut s = String::new();
        if self.flags & SHF_WRITE != 0 { s.push('W'); }
        if self.flags & SHF_ALLOC != 0 { s.push('A'); }
        if self.flags & SHF_EXECINSTR != 0 { s.push('X'); }
        if self.flags & SHF_MERGE != 0 { s.push('M'); }
        if self.flags & SHF_STRINGS != 0 { s.push('S'); }
        if s.is_empty() { s.push('-'); }
        s
    }

    /// Is this a code section?
    pub fn is_executable(&self) -> bool {
        self.flags & SHF_EXECINSTR != 0
    }

    /// Is this loaded into memory?
    pub fn is_alloc(&self) -> bool {
        self.flags & SHF_ALLOC != 0
    }
}

/// Parsed ELF symbol
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol value (address)
    pub value: u64,
    /// Symbol size
    pub size: u64,
    /// Symbol type (STT_*)
    pub sym_type: u8,
    /// Symbol binding (STB_*)
    pub binding: u8,
    /// Visibility
    pub visibility: u8,
    /// Section index
    pub section_index: u16,
}

impl Symbol {
    pub fn type_name(&self) -> &'static str {
        match self.sym_type {
            STT_NOTYPE => "NOTYPE",
            STT_OBJECT => "OBJECT",
            STT_FUNC => "FUNC",
            STT_SECTION => "SECTION",
            STT_FILE => "FILE",
            _ => "UNKNOWN",
        }
    }

    pub fn binding_name(&self) -> &'static str {
        match self.binding {
            STB_LOCAL => "LOCAL",
            STB_GLOBAL => "GLOBAL",
            STB_WEAK => "WEAK",
            _ => "?",
        }
    }

    /// Is this a function?
    pub fn is_function(&self) -> bool {
        self.sym_type == STT_FUNC
    }

    /// Is this a data object?
    pub fn is_object(&self) -> bool {
        self.sym_type == STT_OBJECT
    }

    /// Has a valid address?
    pub fn is_defined(&self) -> bool {
        self.section_index != 0 && self.value != 0
    }
}

/// Parsed program header
#[derive(Debug, Clone)]
pub struct ProgramHeader {
    /// Segment type (PT_*)
    pub p_type: u32,
    /// Segment flags (PF_R | PF_W | PF_X)
    pub flags: u32,
    /// File offset
    pub offset: u64,
    /// Virtual address
    pub vaddr: u64,
    /// Physical address
    pub paddr: u64,
    /// File size
    pub filesz: u64,
    /// Memory size
    pub memsz: u64,
    /// Alignment
    pub align: u64,
}

impl ProgramHeader {
    pub fn type_name(&self) -> &'static str {
        match self.p_type {
            PT_LOAD => "LOAD",
            PT_DYNAMIC => "DYNAMIC",
            PT_INTERP => "INTERP",
            PT_NOTE => "NOTE",
            PT_GNU_EH_FRAME => "GNU_EH_FRAME",
            PT_GNU_STACK => "GNU_STACK",
            PT_GNU_RELRO => "GNU_RELRO",
            6 => "PHDR",
            _ => "UNKNOWN",
        }
    }

    pub fn flags_string(&self) -> String {
        let mut s = String::new();
        s.push(if self.flags & 4 != 0 { 'R' } else { '-' });
        s.push(if self.flags & 2 != 0 { 'W' } else { '-' });
        s.push(if self.flags & 1 != 0 { 'X' } else { '-' });
        s
    }
}

/// Parsed dynamic entry
#[derive(Debug, Clone)]
pub struct DynamicEntry {
    pub tag: i64,
    pub value: u64,
}

impl DynamicEntry {
    pub fn tag_name(&self) -> &'static str {
        match self.tag {
            DT_NULL => "NULL",
            DT_NEEDED => "NEEDED",
            DT_PLTGOT => "PLTGOT",
            DT_HASH => "HASH",
            DT_STRTAB => "STRTAB",
            DT_SYMTAB => "SYMTAB",
            DT_RELA => "RELA",
            DT_RELASZ => "RELASZ",
            DT_RELAENT => "RELAENT",
            DT_STRSZ => "STRSZ",
            DT_INIT => "INIT",
            DT_FINI => "FINI",
            DT_SONAME => "SONAME",
            DT_RPATH => "RPATH",
            DT_JMPREL => "JMPREL",
            DT_BIND_NOW => "BIND_NOW",
            DT_RUNPATH => "RUNPATH",
            DT_FLAGS => "FLAGS",
            DT_GNU_HASH => "GNU_HASH",
            DT_VERNEED => "VERNEED",
            DT_VERNEEDNUM => "VERNEEDNUM",
            _ => "?",
        }
    }
}

/// Parsed relocation entry
#[derive(Debug, Clone)]
pub struct Relocation {
    /// Offset in section
    pub offset: u64,
    /// Relocation type
    pub rtype: u32,
    /// Symbol index
    pub sym_index: u32,
    /// Addend
    pub addend: i64,
    /// Symbol name (resolved)
    pub sym_name: String,
}

impl Relocation {
    pub fn type_name(&self) -> &'static str {
        match self.rtype {
            0 => "R_X86_64_NONE",
            1 => "R_X86_64_64",
            2 => "R_X86_64_PC32",
            5 => "R_X86_64_COPY",
            6 => "R_X86_64_GLOB_DAT",
            7 => "R_X86_64_JUMP_SLOT",
            8 => "R_X86_64_RELATIVE",
            10 => "R_X86_64_32",
            11 => "R_X86_64_32S",
            _ => "R_X86_64_?",
        }
    }
}

/// Extracted string with location
#[derive(Debug, Clone)]
pub struct ExtractedString {
    /// File offset
    pub offset: u64,
    /// Virtual address (if in a loaded section)
    pub vaddr: Option<u64>,
    /// The string content
    pub content: String,
    /// Which section it belongs to
    pub section: String,
}

// ──── ELF Header Info ──────────────────────────────────────────────────────

/// High-level ELF header analysis
#[derive(Debug, Clone)]
pub struct ElfInfo {
    /// ELF class name (ELF64)
    pub class: &'static str,
    /// Data encoding (LE/BE)
    pub data: &'static str,
    /// OS/ABI
    pub osabi: &'static str,
    /// ELF type (EXEC, DYN, REL)
    pub elf_type: &'static str,
    /// Machine
    pub machine: &'static str,
    /// Entry point
    pub entry: u64,
    /// Program header offset
    pub phoff: u64,
    /// Section header offset
    pub shoff: u64,
    /// Number of program headers
    pub phnum: u16,
    /// Number of section headers
    pub shnum: u16,
    /// Section name string table index
    pub shstrndx: u16,
    /// File size
    pub file_size: usize,
}

// ──── Full ELF Analysis ────────────────────────────────────────────────────

/// Complete parsed ELF file
#[derive(Debug)]
pub struct ElfAnalysis {
    /// Header info
    pub info: ElfInfo,
    /// Program headers
    pub programs: Vec<ProgramHeader>,
    /// Section headers
    pub sections: Vec<Section>,
    /// Symbol table (.symtab)
    pub symbols: Vec<Symbol>,
    /// Dynamic symbols (.dynsym)
    pub dynamic_symbols: Vec<Symbol>,
    /// Dynamic entries
    pub dynamic: Vec<DynamicEntry>,
    /// Relocations
    pub relocations: Vec<Relocation>,
    /// Extracted strings
    pub strings: Vec<ExtractedString>,
    /// Interpreter path (e.g., /lib64/ld-linux-x86-64.so.2)
    pub interpreter: Option<String>,
    /// Needed shared libraries
    pub needed_libs: Vec<String>,
    /// Address → symbol name mapping (for disassembly annotation)
    pub addr_to_symbol: BTreeMap<u64, String>,
    /// Raw data reference (for hex view)
    raw_data_len: usize,
}

impl ElfAnalysis {
    /// Get section by name
    pub fn section_by_name(&self, name: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.name == name)
    }

    /// Get sections that contain code
    pub fn code_sections(&self) -> Vec<&Section> {
        self.sections.iter().filter(|s| s.is_executable()).collect()
    }

    /// Get function symbols sorted by address
    pub fn functions(&self) -> Vec<&Symbol> {
        let mut funcs: Vec<&Symbol> = self.symbols.iter()
            .chain(self.dynamic_symbols.iter())
            .filter(|s| s.is_function() && s.is_defined())
            .collect();
        funcs.sort_by_key(|s| s.value);
        funcs.dedup_by_key(|s| s.value);
        funcs
    }

    /// Get symbol at address
    pub fn symbol_at(&self, addr: u64) -> Option<&str> {
        self.addr_to_symbol.get(&addr).map(|s| s.as_str())
    }

    /// Find which section an address belongs to
    pub fn section_for_addr(&self, addr: u64) -> Option<&Section> {
        self.sections.iter().find(|s| {
            s.is_alloc() && addr >= s.addr && addr < s.addr + s.size
        })
    }

    /// Get file size
    pub fn file_size(&self) -> usize {
        self.raw_data_len
    }
}

// ──── Parser ───────────────────────────────────────────────────────────────

/// Parse a complete ELF file from raw bytes
pub fn parse_elf(data: &[u8]) -> Result<ElfAnalysis, &'static str> {
    if data.len() < 64 {
        return Err("File too small for ELF header");
    }
    if &data[0..4] != b"\x7FELF" {
        return Err("Not an ELF file (bad magic)");
    }

    // ── Parse ELF header ──
    let class = data[4];
    if class != 2 {
        return Err("Not a 64-bit ELF");
    }
    let data_enc = data[5];
    if data_enc != 1 {
        return Err("Not little-endian (unsupported)");
    }

    let e_type = u16le(data, 16);
    let e_machine = u16le(data, 18);
    let e_entry = u64le(data, 24);
    let e_phoff = u64le(data, 32);
    let e_shoff = u64le(data, 40);
    let e_phentsize = u16le(data, 54);
    let e_phnum = u16le(data, 56);
    let e_shentsize = u16le(data, 58);
    let e_shnum = u16le(data, 60);
    let e_shstrndx = u16le(data, 62);

    let info = ElfInfo {
        class: "ELF64",
        data: if data_enc == 1 { "Little Endian" } else { "Big Endian" },
        osabi: match data[7] {
            0 => "UNIX System V",
            3 => "Linux",
            _ => "Unknown",
        },
        elf_type: match e_type {
            1 => "REL (Relocatable)",
            2 => "EXEC (Executable)",
            3 => "DYN (Shared/PIE)",
            4 => "CORE (Core dump)",
            _ => "Unknown",
        },
        machine: match e_machine {
            62 => "x86-64",
            3 => "x86 (i386)",
            40 => "ARM",
            183 => "AArch64",
            243 => "RISC-V",
            _ => "Unknown",
        },
        entry: e_entry,
        phoff: e_phoff,
        shoff: e_shoff,
        phnum: e_phnum,
        shnum: e_shnum,
        shstrndx: e_shstrndx,
        file_size: data.len(),
    };

    // ── Parse program headers ──
    let mut programs = Vec::new();
    let mut interpreter = None;

    for i in 0..e_phnum as usize {
        let off = e_phoff as usize + i * e_phentsize as usize;
        if off + 56 > data.len() { break; }

        let p_type = u32le(data, off);
        let flags = u32le(data, off + 4);
        let offset = u64le(data, off + 8);
        let vaddr = u64le(data, off + 16);
        let paddr = u64le(data, off + 24);
        let filesz = u64le(data, off + 32);
        let memsz = u64le(data, off + 40);
        let align = u64le(data, off + 48);

        // Extract interpreter
        if p_type == PT_INTERP {
            let start = offset as usize;
            let end = (offset + filesz) as usize;
            if end <= data.len() {
                let s = &data[start..end];
                let len = s.iter().position(|&b| b == 0).unwrap_or(s.len());
                interpreter = Some(String::from(core::str::from_utf8(&s[..len]).unwrap_or("?")));
            }
        }

        programs.push(ProgramHeader {
            p_type, flags, offset, vaddr, paddr, filesz, memsz, align,
        });
    }

    // ── Parse section headers ──
    let sections = parse_sections(data, e_shoff as usize, e_shentsize as usize, e_shnum as usize, e_shstrndx as usize);

    // ── Parse symbol tables ──
    let symbols = parse_symbol_table(data, &sections, SHT_SYMTAB);
    let dynamic_symbols = parse_symbol_table(data, &sections, SHT_DYNSYM);

    // ── Build address-to-symbol map ──
    let mut addr_to_symbol = BTreeMap::new();
    for sym in symbols.iter().chain(dynamic_symbols.iter()) {
        if sym.is_defined() && !sym.name.is_empty() {
            addr_to_symbol.insert(sym.value, sym.name.clone());
        }
    }

    // ── Parse dynamic section ──
    let (dynamic, needed_libs) = parse_dynamic(data, &sections);

    // ── Parse relocations ──
    let relocations = parse_relocations(data, &sections, &dynamic_symbols);

    // ── Extract strings ──
    let strings = extract_all_strings(data, &sections, &programs);

    Ok(ElfAnalysis {
        info,
        programs,
        sections,
        symbols,
        dynamic_symbols,
        dynamic,
        relocations,
        strings,
        interpreter,
        needed_libs,
        addr_to_symbol,
        raw_data_len: data.len(),
    })
}

// ──── Internal Helpers ─────────────────────────────────────────────────────

fn parse_sections(data: &[u8], shoff: usize, shentsize: usize, shnum: usize, shstrndx: usize) -> Vec<Section> {
    if shoff == 0 || shnum == 0 {
        return Vec::new();
    }

    // First pass: read raw section headers
    let mut raw_sections: Vec<(u32, u32, u64, u64, u64, u64, u32, u32, u64, u64)> = Vec::new();
    for i in 0..shnum {
        let off = shoff + i * shentsize;
        if off + 64 > data.len() { break; }

        raw_sections.push((
            u32le(data, off),           // sh_name
            u32le(data, off + 4),       // sh_type
            u64le(data, off + 8),       // sh_flags
            u64le(data, off + 16),      // sh_addr
            u64le(data, off + 24),      // sh_offset
            u64le(data, off + 32),      // sh_size
            u32le(data, off + 40),      // sh_link
            u32le(data, off + 44),      // sh_info
            u64le(data, off + 48),      // sh_addralign
            u64le(data, off + 56),      // sh_entsize
        ));
    }

    // Get section name string table
    let shstrtab_data = if shstrndx < raw_sections.len() {
        let (_, _, _, _, strtab_off, strtab_size, _, _, _, _) = raw_sections[shstrndx];
        let start = strtab_off as usize;
        let end = (strtab_off + strtab_size) as usize;
        if end <= data.len() { Some(&data[start..end]) } else { None }
    } else {
        None
    };

    // Second pass: build Section structs with names
    let mut sections = Vec::new();
    for (i, &(sh_name, sh_type, flags, addr, offset, size, link, info, addralign, entsize)) in raw_sections.iter().enumerate() {
        let name = if let Some(strtab) = shstrtab_data {
            read_string_at(strtab, sh_name as usize)
        } else {
            format!("section_{}", i)
        };

        sections.push(Section {
            index: i,
            name,
            sh_type,
            flags,
            addr,
            offset,
            size,
            link,
            info,
            addralign,
            entsize,
        });
    }

    sections
}

fn parse_symbol_table(data: &[u8], sections: &[Section], table_type: u32) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    // Find the symbol table section
    let symtab = match sections.iter().find(|s| s.sh_type == table_type) {
        Some(s) => s,
        None => return symbols,
    };

    // The linked section is the string table
    let strtab_section = if (symtab.link as usize) < sections.len() {
        &sections[symtab.link as usize]
    } else {
        return symbols;
    };

    let strtab_start = strtab_section.offset as usize;
    let strtab_end = strtab_start + strtab_section.size as usize;
    if strtab_end > data.len() { return symbols; }
    let strtab = &data[strtab_start..strtab_end];

    let sym_start = symtab.offset as usize;
    let sym_count = if symtab.entsize > 0 {
        symtab.size / symtab.entsize
    } else {
        0
    };

    for i in 0..sym_count as usize {
        let off = sym_start + i * 24; // Elf64_Sym = 24 bytes
        if off + 24 > data.len() { break; }

        let st_name = u32le(data, off);
        let st_info = data[off + 4];
        let st_other = data[off + 5];
        let st_shndx = u16le(data, off + 6);
        let st_value = u64le(data, off + 8);
        let st_size = u64le(data, off + 16);

        let name = read_string_at(strtab, st_name as usize);
        let sym_type = st_info & 0x0F;
        let binding = st_info >> 4;
        let visibility = st_other & 0x03;

        symbols.push(Symbol {
            name,
            value: st_value,
            size: st_size,
            sym_type,
            binding,
            visibility,
            section_index: st_shndx,
        });
    }

    symbols
}

fn parse_dynamic(data: &[u8], sections: &[Section]) -> (Vec<DynamicEntry>, Vec<String>) {
    let mut entries = Vec::new();
    let mut needed_libs = Vec::new();

    let dyn_section = match sections.iter().find(|s| s.sh_type == SHT_DYNAMIC) {
        Some(s) => s,
        None => return (entries, needed_libs),
    };

    // Find the dynamic string table
    let dynstr = if (dyn_section.link as usize) < sections.len() {
        let s = &sections[dyn_section.link as usize];
        let start = s.offset as usize;
        let end = start + s.size as usize;
        if end <= data.len() { Some(&data[start..end]) } else { None }
    } else {
        None
    };

    let start = dyn_section.offset as usize;
    let count = dyn_section.size as usize / 16; // Elf64_Dyn = 16 bytes

    for i in 0..count {
        let off = start + i * 16;
        if off + 16 > data.len() { break; }

        let tag = i64le(data, off);
        let value = u64le(data, off + 8);

        if tag == DT_NULL { break; }

        // Resolve DT_NEEDED to library names
        if tag == DT_NEEDED {
            if let Some(strtab) = dynstr {
                let name = read_string_at(strtab, value as usize);
                needed_libs.push(name);
            }
        }

        entries.push(DynamicEntry { tag, value });
    }

    (entries, needed_libs)
}

fn parse_relocations(data: &[u8], sections: &[Section], dynsyms: &[Symbol]) -> Vec<Relocation> {
    let mut relocations = Vec::new();

    for section in sections.iter() {
        if section.sh_type != SHT_RELA && section.sh_type != SHT_REL {
            continue;
        }

        let start = section.offset as usize;
        let is_rela = section.sh_type == SHT_RELA;
        let entry_size = if is_rela { 24 } else { 16 };
        let count = section.size as usize / entry_size;

        for i in 0..count {
            let off = start + i * entry_size;
            if off + entry_size > data.len() { break; }

            let r_offset = u64le(data, off);
            let r_info = u64le(data, off + 8);
            let r_addend = if is_rela { i64le(data, off + 16) } else { 0 };

            let sym_index = (r_info >> 32) as u32;
            let rtype = (r_info & 0xFFFFFFFF) as u32;

            let sym_name = if (sym_index as usize) < dynsyms.len() {
                dynsyms[sym_index as usize].name.clone()
            } else {
                String::new()
            };

            relocations.push(Relocation {
                offset: r_offset,
                rtype,
                sym_index,
                addend: r_addend,
                sym_name,
            });
        }
    }

    relocations
}

fn extract_all_strings(data: &[u8], sections: &[Section], programs: &[ProgramHeader]) -> Vec<ExtractedString> {
    let mut strings = Vec::new();

    // Extract from each allocatable section
    for section in sections.iter() {
        if section.sh_type == SHT_NULL || section.size == 0 {
            continue;
        }

        let start = section.offset as usize;
        let end = start + section.size as usize;
        if end > data.len() { continue; }

        let section_data = &data[start..end];
        let mut current = String::new();
        let mut str_start = 0usize;

        for (i, &b) in section_data.iter().enumerate() {
            if b >= 0x20 && b < 0x7F {
                if current.is_empty() {
                    str_start = i;
                }
                current.push(b as char);
            } else {
                if current.len() >= 4 {
                    let file_offset = start + str_start;
                    // Compute vaddr if this section has one
                    let vaddr = if section.is_alloc() && section.addr > 0 {
                        Some(section.addr + str_start as u64)
                    } else {
                        None
                    };

                    strings.push(ExtractedString {
                        offset: file_offset as u64,
                        vaddr,
                        content: current.clone(),
                        section: section.name.clone(),
                    });
                }
                current.clear();
            }
        }

        // Flush remaining
        if current.len() >= 4 {
            let file_offset = start + str_start;
            let vaddr = if section.is_alloc() && section.addr > 0 {
                Some(section.addr + str_start as u64)
            } else {
                None
            };
            strings.push(ExtractedString {
                offset: file_offset as u64,
                vaddr,
                content: current,
                section: section.name.clone(),
            });
        }
    }

    // Deduplicate by offset
    strings.sort_by_key(|s| s.offset);
    strings.dedup_by_key(|s| s.offset);

    strings
}

// ──── Byte Reading Utilities ───────────────────────────────────────────────

fn u16le(data: &[u8], off: usize) -> u16 {
    if off + 2 > data.len() { return 0; }
    u16::from_le_bytes([data[off], data[off + 1]])
}

fn u32le(data: &[u8], off: usize) -> u32 {
    if off + 4 > data.len() { return 0; }
    u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}

fn u64le(data: &[u8], off: usize) -> u64 {
    if off + 8 > data.len() { return 0; }
    u64::from_le_bytes([
        data[off], data[off + 1], data[off + 2], data[off + 3],
        data[off + 4], data[off + 5], data[off + 6], data[off + 7],
    ])
}

fn i64le(data: &[u8], off: usize) -> i64 {
    u64le(data, off) as i64
}

fn read_string_at(strtab: &[u8], offset: usize) -> String {
    if offset >= strtab.len() {
        return String::new();
    }
    let bytes = &strtab[offset..];
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from(core::str::from_utf8(&bytes[..len]).unwrap_or(""))
}
