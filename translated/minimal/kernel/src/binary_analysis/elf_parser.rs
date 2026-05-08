




use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;



pub const BHW_: u32 = 0;
pub const CWU_: u32 = 1;
pub const BHY_: u32 = 2;
pub const CWV_: u32 = 3;
pub const AJT_: u32 = 4;
pub const CWQ_: u32 = 5;
pub const BHU_: u32 = 6;
pub const CWT_: u32 = 7;
pub const CWS_: u32 = 8;
pub const BHX_: u32 = 9;
pub const BHV_: u32 = 11;
pub const CWR_: u32 = 14;
pub const CWM_: u32 = 15;
pub const CWN_: u32 = 0x6FFFFFF6;
pub const CWP_: u32 = 0x6FFFFFFF;
pub const CWO_: u32 = 0x6FFFFFFE;



pub const CWL_: u64 = 0x1;
pub const BHS_: u64 = 0x2;
pub const BHT_: u64 = 0x4;
pub const CWJ_: u64 = 0x10;
pub const CWK_: u64 = 0x20;



pub const CXZ_: u8 = 0;
pub const CXY_: u8 = 1;
pub const CYA_: u8 = 2;

pub const CYH_: u8 = 0;
pub const BIR_: u8 = 1;
pub const BIQ_: u8 = 2;
pub const CYI_: u8 = 3;
pub const CYG_: u8 = 4;



pub const JM_: u32 = 1;
pub const AIN_: u32 = 2;
pub const XU_: u32 = 3;
pub const BFU_: u32 = 4;
pub const CPV_: u32 = 0x6474E550;
pub const BFT_: u32 = 0x6474E551;
pub const BFS_: u32 = 0x6474E552;



pub const UB_: i64 = 0;
pub const UA_: i64 = 1;
pub const ASX_: i64 = 3;
pub const ASW_: i64 = 4;
pub const ADE_: i64 = 5;
pub const ADF_: i64 = 6;
pub const ADA_: i64 = 7;
pub const ADC_: i64 = 8;
pub const ADB_: i64 = 9;
pub const ADD_: i64 = 10;
pub const ACY_: i64 = 12;
pub const ACW_: i64 = 13;
pub const ASZ_: i64 = 14;
pub const ASY_: i64 = 15;
pub const BVO_: i64 = 16;
pub const ACZ_: i64 = 23;
pub const BVF_: i64 = 24;
pub const BVN_: i64 = 29;
pub const ACX_: i64 = 30;
pub const BVJ_: i64 = 0x6FFFFEF5;
pub const BVP_: i64 = 0x6FFFFFFE;
pub const BVQ_: i64 = 0x6FFFFFFF;




#[derive(Debug, Clone)]
pub struct Ct {
    
    pub index: usize,
    
    pub name: String,
    
    pub sh_type: u32,
    
    pub flags: u64,
    
    pub addr: u64,
    
    pub offset: u64,
    
    pub size: u64,
    
    pub link: u32,
    
    pub info: u32,
    
    pub addralign: u64,
    
    pub entsize: u64,
}

impl Ct {
    pub fn type_name(&self) -> &'static str {
        match self.sh_type {
            BHW_ => "NULL",
            CWU_ => "PROGBITS",
            BHY_ => "SYMTAB",
            CWV_ => "STRTAB",
            AJT_ => "RELA",
            CWQ_ => "HASH",
            BHU_ => "DYNAMIC",
            CWT_ => "NOTE",
            CWS_ => "NOBITS",
            BHX_ => "REL",
            BHV_ => "DYNSYM",
            CWR_ => "INIT_ARRAY",
            CWM_ => "FINI_ARRAY",
            CWN_ => "GNU_HASH",
            CWP_ => "GNU_VERSYM",
            CWO_ => "GNU_VERNEED",
            _ => "UNKNOWN",
        }
    }

    pub fn flags_string(&self) -> String {
        let mut j = String::new();
        if self.flags & CWL_ != 0 { j.push('W'); }
        if self.flags & BHS_ != 0 { j.push('A'); }
        if self.flags & BHT_ != 0 { j.push('X'); }
        if self.flags & CWJ_ != 0 { j.push('M'); }
        if self.flags & CWK_ != 0 { j.push('S'); }
        if j.is_empty() { j.push('-'); }
        j
    }

    
    pub fn is_executable(&self) -> bool {
        self.flags & BHT_ != 0
    }

    
    pub fn is_alloc(&self) -> bool {
        self.flags & BHS_ != 0
    }
}


#[derive(Debug, Clone)]
pub struct Cy {
    
    pub name: String,
    
    pub value: u64,
    
    pub size: u64,
    
    pub sym_type: u8,
    
    pub binding: u8,
    
    pub jqe: u8,
    
    pub section_index: u16,
}

impl Cy {
    pub fn type_name(&self) -> &'static str {
        match self.sym_type {
            CYH_ => "NOTYPE",
            BIR_ => "OBJECT",
            BIQ_ => "FUNC",
            CYI_ => "SECTION",
            CYG_ => "FILE",
            _ => "UNKNOWN",
        }
    }

    pub fn binding_name(&self) -> &'static str {
        match self.binding {
            CXZ_ => "LOCAL",
            CXY_ => "GLOBAL",
            CYA_ => "WEAK",
            _ => "?",
        }
    }

    
    pub fn is_function(&self) -> bool {
        self.sym_type == BIQ_
    }

    
    pub fn qms(&self) -> bool {
        self.sym_type == BIR_
    }

    
    pub fn is_defined(&self) -> bool {
        self.section_index != 0 && self.value != 0
    }
}


#[derive(Debug, Clone)]
pub struct Gz {
    
    pub p_type: u32,
    
    pub flags: u32,
    
    pub offset: u64,
    
    pub vaddr: u64,
    
    pub itf: u64,
    
    pub filesz: u64,
    
    pub memsz: u64,
    
    pub align: u64,
}

impl Gz {
    pub fn type_name(&self) -> &'static str {
        match self.p_type {
            JM_ => "LOAD",
            AIN_ => "DYNAMIC",
            XU_ => "INTERP",
            BFU_ => "NOTE",
            CPV_ => "GNU_EH_FRAME",
            BFT_ => "GNU_STACK",
            BFS_ => "GNU_RELRO",
            6 => "PHDR",
            _ => "UNKNOWN",
        }
    }

    pub fn flags_string(&self) -> String {
        let mut j = String::new();
        j.push(if self.flags & 4 != 0 { 'R' } else { '-' });
        j.push(if self.flags & 2 != 0 { 'W' } else { '-' });
        j.push(if self.flags & 1 != 0 { 'X' } else { '-' });
        j
    }
}


#[derive(Debug, Clone)]
pub struct Lo {
    pub tag: i64,
    pub value: u64,
}

impl Lo {
    pub fn qys(&self) -> &'static str {
        match self.tag {
            UB_ => "NULL",
            UA_ => "NEEDED",
            ASX_ => "PLTGOT",
            ASW_ => "HASH",
            ADE_ => "STRTAB",
            ADF_ => "SYMTAB",
            ADA_ => "RELA",
            ADC_ => "RELASZ",
            ADB_ => "RELAENT",
            ADD_ => "STRSZ",
            ACY_ => "INIT",
            ACW_ => "FINI",
            ASZ_ => "SONAME",
            ASY_ => "RPATH",
            ACZ_ => "JMPREL",
            BVF_ => "BIND_NOW",
            BVN_ => "RUNPATH",
            ACX_ => "FLAGS",
            BVJ_ => "GNU_HASH",
            BVP_ => "VERNEED",
            BVQ_ => "VERNEEDNUM",
            _ => "?",
        }
    }
}


#[derive(Debug, Clone)]
pub struct Mz {
    
    pub offset: u64,
    
    pub rtype: u32,
    
    pub fbw: u32,
    
    pub addend: i64,
    
    pub sym_name: String,
}

impl Mz {
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


#[derive(Debug, Clone)]
pub struct Ma {
    
    pub offset: u64,
    
    pub vaddr: Option<u64>,
    
    pub content: String,
    
    pub section: String,
}




#[derive(Debug, Clone)]
pub struct Rw {
    
    pub class: &'static str,
    
    pub data: &'static str,
    
    pub osabi: &'static str,
    
    pub elf_type: &'static str,
    
    pub machine: &'static str,
    
    pub entry: u64,
    
    pub gmx: u64,
    
    pub shoff: u64,
    
    pub gmw: u16,
    
    pub shnum: u16,
    
    pub shstrndx: u16,
    
    pub file_size: usize,
}




#[derive(Debug)]
pub struct Lz {
    
    pub info: Rw,
    
    pub programs: Vec<Gz>,
    
    pub sections: Vec<Ct>,
    
    pub symbols: Vec<Cy>,
    
    pub dynamic_symbols: Vec<Cy>,
    
    pub dynamic: Vec<Lo>,
    
    pub relocations: Vec<Mz>,
    
    pub strings: Vec<Ma>,
    
    pub interpreter: Option<String>,
    
    pub needed_libs: Vec<String>,
    
    pub addr_to_symbol: BTreeMap<u64, String>,
    
    raw_data_len: usize,
}

impl Lz {
    
    pub fn qva(&self, name: &str) -> Option<&Ct> {
        self.sections.iter().find(|j| j.name == name)
    }

    
    pub fn code_sections(&self) -> Vec<&Ct> {
        self.sections.iter().filter(|j| j.is_executable()).collect()
    }

    
    pub fn functions(&self) -> Vec<&Cy> {
        let mut dqj: Vec<&Cy> = self.symbols.iter()
            .chain(self.dynamic_symbols.iter())
            .filter(|j| j.is_function() && j.is_defined())
            .collect();
        dqj.sort_by_key(|j| j.value);
        dqj.dedup_by_key(|j| j.value);
        dqj
    }

    
    pub fn qyi(&self, addr: u64) -> Option<&str> {
        self.addr_to_symbol.get(&addr).map(|j| j.as_str())
    }

    
    pub fn section_for_addr(&self, addr: u64) -> Option<&Ct> {
        self.sections.iter().find(|j| {
            j.is_alloc() && addr >= j.addr && addr < j.addr + j.size
        })
    }

    
    pub fn file_size(&self) -> usize {
        self.raw_data_len
    }
}




pub fn nqd(data: &[u8]) -> Result<Lz, &'static str> {
    if data.len() < 64 {
        return Err("File too small for ELF header");
    }
    if &data[0..4] != b"\x7FELF" {
        return Err("Not an ELF file (bad magic)");
    }

    
    let class = data[4];
    if class != 2 {
        return Err("Not a 64-bit ELF");
    }
    let hqp = data[5];
    if hqp != 1 {
        return Err("Not little-endian (unsupported)");
    }

    let e_type = ceu(data, 16);
    let e_machine = ceu(data, 18);
    let e_entry = afv(data, 24);
    let e_phoff = afv(data, 32);
    let ftx = afv(data, 40);
    let e_phentsize = ceu(data, 54);
    let e_phnum = ceu(data, 56);
    let hur = ceu(data, 58);
    let ftv = ceu(data, 60);
    let fty = ceu(data, 62);

    let info = Rw {
        class: "ELF64",
        data: if hqp == 1 { "Little Endian" } else { "Big Endian" },
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
        gmx: e_phoff,
        shoff: ftx,
        gmw: e_phnum,
        shnum: ftv,
        shstrndx: fty,
        file_size: data.len(),
    };

    
    let mut programs = Vec::new();
    let mut interpreter = None;

    for i in 0..e_phnum as usize {
        let off = e_phoff as usize + i * e_phentsize as usize;
        if off + 56 > data.len() { break; }

        let p_type = csd(data, off);
        let flags = csd(data, off + 4);
        let offset = afv(data, off + 8);
        let vaddr = afv(data, off + 16);
        let itf = afv(data, off + 24);
        let filesz = afv(data, off + 32);
        let memsz = afv(data, off + 40);
        let align = afv(data, off + 48);

        
        if p_type == XU_ {
            let start = offset as usize;
            let end = (offset + filesz) as usize;
            if end <= data.len() {
                let j = &data[start..end];
                let len = j.iter().position(|&b| b == 0).unwrap_or(j.len());
                interpreter = Some(String::from(core::str::from_utf8(&j[..len]).unwrap_or("?")));
            }
        }

        programs.push(Gz {
            p_type, flags, offset, vaddr, itf, filesz, memsz, align,
        });
    }

    
    let sections = nrc(data, ftx as usize, hur as usize, ftv as usize, fty as usize);

    
    let symbols = ity(data, &sections, BHY_);
    let dynamic_symbols = ity(data, &sections, BHV_);

    
    let mut addr_to_symbol = BTreeMap::new();
    for sym in symbols.iter().chain(dynamic_symbols.iter()) {
        if sym.is_defined() && !sym.name.is_empty() {
            addr_to_symbol.insert(sym.value, sym.name.clone());
        }
    }

    
    let (dynamic, needed_libs) = gmh(data, &sections);

    
    let relocations = nqy(data, &sections, &dynamic_symbols);

    
    let strings = lto(data, &sections, &programs);

    Ok(Lz {
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



fn nrc(data: &[u8], shoff: usize, shentsize: usize, shnum: usize, shstrndx: usize) -> Vec<Ct> {
    if shoff == 0 || shnum == 0 {
        return Vec::new();
    }

    
    let mut exs: Vec<(u32, u32, u64, u64, u64, u64, u32, u32, u64, u64)> = Vec::new();
    for i in 0..shnum {
        let off = shoff + i * shentsize;
        if off + 64 > data.len() { break; }

        exs.push((
            csd(data, off),           
            csd(data, off + 4),       
            afv(data, off + 8),       
            afv(data, off + 16),      
            afv(data, off + 24),      
            afv(data, off + 32),      
            csd(data, off + 40),      
            csd(data, off + 44),      
            afv(data, off + 48),      
            afv(data, off + 56),      
        ));
    }

    
    let osh = if shstrndx < exs.len() {
        let (_, _, _, _, strtab_off, strtab_size, _, _, _, _) = exs[shstrndx];
        let start = strtab_off as usize;
        let end = (strtab_off + strtab_size) as usize;
        if end <= data.len() { Some(&data[start..end]) } else { None }
    } else {
        None
    };

    
    let mut sections = Vec::new();
    for (i, &(sh_name, sh_type, flags, addr, offset, size, link, info, addralign, entsize)) in exs.iter().enumerate() {
        let name = if let Some(cdz) = osh {
            gqg(cdz, sh_name as usize)
        } else {
            format!("section_{}", i)
        };

        sections.push(Ct {
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

fn ity(data: &[u8], sections: &[Ct], table_type: u32) -> Vec<Cy> {
    let mut symbols = Vec::new();

    
    let dfc = match sections.iter().find(|j| j.sh_type == table_type) {
        Some(j) => j,
        None => return symbols,
    };

    
    let jje = if (dfc.link as usize) < sections.len() {
        &sections[dfc.link as usize]
    } else {
        return symbols;
    };

    let eam = jje.offset as usize;
    let jjd = eam + jje.size as usize;
    if jjd > data.len() { return symbols; }
    let cdz = &data[eam..jjd];

    let ozk = dfc.offset as usize;
    let gwv = if dfc.entsize > 0 {
        dfc.size / dfc.entsize
    } else {
        0
    };

    for i in 0..gwv as usize {
        let off = ozk + i * 24; 
        if off + 24 > data.len() { break; }

        let jhy = csd(data, off);
        let gwa = data[off + 4];
        let jhz = data[off + 5];
        let jia = ceu(data, off + 6);
        let jib = afv(data, off + 8);
        let st_size = afv(data, off + 16);

        let name = gqg(cdz, jhy as usize);
        let sym_type = gwa & 0x0F;
        let binding = gwa >> 4;
        let jqe = jhz & 0x03;

        symbols.push(Cy {
            name,
            value: jib,
            size: st_size,
            sym_type,
            binding,
            jqe,
            section_index: jia,
        });
    }

    symbols
}

fn gmh(data: &[u8], sections: &[Ct]) -> (Vec<Lo>, Vec<String>) {
    let mut entries = Vec::new();
    let mut needed_libs = Vec::new();

    let ekz = match sections.iter().find(|j| j.sh_type == BHU_) {
        Some(j) => j,
        None => return (entries, needed_libs),
    };

    
    let lna = if (ekz.link as usize) < sections.len() {
        let j = &sections[ekz.link as usize];
        let start = j.offset as usize;
        let end = start + j.size as usize;
        if end <= data.len() { Some(&data[start..end]) } else { None }
    } else {
        None
    };

    let start = ekz.offset as usize;
    let count = ekz.size as usize / 16; 

    for i in 0..count {
        let off = start + i * 16;
        if off + 16 > data.len() { break; }

        let tag = ifj(data, off);
        let value = afv(data, off + 8);

        if tag == UB_ { break; }

        
        if tag == UA_ {
            if let Some(cdz) = lna {
                let name = gqg(cdz, value as usize);
                needed_libs.push(name);
            }
        }

        entries.push(Lo { tag, value });
    }

    (entries, needed_libs)
}

fn nqy(data: &[u8], sections: &[Ct], dynsyms: &[Cy]) -> Vec<Mz> {
    let mut relocations = Vec::new();

    for section in sections.iter() {
        if section.sh_type != AJT_ && section.sh_type != BHX_ {
            continue;
        }

        let start = section.offset as usize;
        let iii = section.sh_type == AJT_;
        let oi = if iii { 24 } else { 16 };
        let count = section.size as usize / oi;

        for i in 0..count {
            let off = start + i * oi;
            if off + oi > data.len() { break; }

            let r_offset = afv(data, off);
            let r_info = afv(data, off + 8);
            let r_addend = if iii { ifj(data, off + 16) } else { 0 };

            let fbw = (r_info >> 32) as u32;
            let rtype = (r_info & 0xFFFFFFFF) as u32;

            let sym_name = if (fbw as usize) < dynsyms.len() {
                dynsyms[fbw as usize].name.clone()
            } else {
                String::new()
            };

            relocations.push(Mz {
                offset: r_offset,
                rtype,
                fbw,
                addend: r_addend,
                sym_name,
            });
        }
    }

    relocations
}

fn lto(data: &[u8], sections: &[Ct], programs: &[Gz]) -> Vec<Ma> {
    let mut strings = Vec::new();

    
    for section in sections.iter() {
        if section.sh_type == BHW_ || section.size == 0 {
            continue;
        }

        let start = section.offset as usize;
        let end = start + section.size as usize;
        if end > data.len() { continue; }

        let omr = &data[start..end];
        let mut current = String::new();
        let mut eal = 0usize;

        for (i, &b) in omr.iter().enumerate() {
            if b >= 0x20 && b < 0x7F {
                if current.is_empty() {
                    eal = i;
                }
                current.push(b as char);
            } else {
                if current.len() >= 4 {
                    let aaw = start + eal;
                    
                    let vaddr = if section.is_alloc() && section.addr > 0 {
                        Some(section.addr + eal as u64)
                    } else {
                        None
                    };

                    strings.push(Ma {
                        offset: aaw as u64,
                        vaddr,
                        content: current.clone(),
                        section: section.name.clone(),
                    });
                }
                current.clear();
            }
        }

        
        if current.len() >= 4 {
            let aaw = start + eal;
            let vaddr = if section.is_alloc() && section.addr > 0 {
                Some(section.addr + eal as u64)
            } else {
                None
            };
            strings.push(Ma {
                offset: aaw as u64,
                vaddr,
                content: current,
                section: section.name.clone(),
            });
        }
    }

    
    strings.sort_by_key(|j| j.offset);
    strings.dedup_by_key(|j| j.offset);

    strings
}



fn ceu(data: &[u8], off: usize) -> u16 {
    if off + 2 > data.len() { return 0; }
    u16::from_le_bytes([data[off], data[off + 1]])
}

fn csd(data: &[u8], off: usize) -> u32 {
    if off + 4 > data.len() { return 0; }
    u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}

fn afv(data: &[u8], off: usize) -> u64 {
    if off + 8 > data.len() { return 0; }
    u64::from_le_bytes([
        data[off], data[off + 1], data[off + 2], data[off + 3],
        data[off + 4], data[off + 5], data[off + 6], data[off + 7],
    ])
}

fn ifj(data: &[u8], off: usize) -> i64 {
    afv(data, off) as i64
}

fn gqg(cdz: &[u8], offset: usize) -> String {
    if offset >= cdz.len() {
        return String::new();
    }
    let bytes = &cdz[offset..];
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from(core::str::from_utf8(&bytes[..len]).unwrap_or(""))
}
