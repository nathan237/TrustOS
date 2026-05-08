



use alloc::vec::Vec;
use alloc::string::String;


const ATI_: [u8; 4] = [0x7F, b'E', b'L', b'F'];


const Aiv: u8 = 2;


const Aiw: u8 = 1; 


const BXI_: u16 = 2;    
const ATK_: u16 = 3;     


const BWI_: u16 = 62;


const EFR_: u32 = 0;
const JM_: u32 = 1;
const AIN_: u32 = 2;
const XU_: u32 = 3;
const BFU_: u32 = 4;
const EFS_: u32 = 6;
const BFS_: u32 = 0x6474e552;
const BFT_: u32 = 0x6474e551;


pub const CMO_: u32 = 1; 
pub const CMN_: u32 = 2; 
pub const CMM_: u32 = 4; 


const UB_: i64 = 0;
const UA_: i64 = 1;     
const BVM_: i64 = 2;  
const ASX_: i64 = 3;
const ASW_: i64 = 4;
const ADE_: i64 = 5;     
const ADF_: i64 = 6;    
const ADA_: i64 = 7;      
const ADC_: i64 = 8;
const ADB_: i64 = 9;
const ADD_: i64 = 10;
const DOL_: i64 = 11;
const ACY_: i64 = 12;
const ACW_: i64 = 13;
const ASZ_: i64 = 14;
const ASY_: i64 = 15;
const BVO_: i64 = 16;
const DOI_: i64 = 17;
const DOK_: i64 = 18;
const DOJ_: i64 = 19;
const DOH_: i64 = 20;
const DOG_: i64 = 21;
const DOM_: i64 = 22;
const ACZ_: i64 = 23;
const BVK_: i64 = 25;
const BVG_: i64 = 26;
const BVL_: i64 = 27;
const BVH_: i64 = 28;
const ACX_: i64 = 30;
const BVI_: i64 = 0x6ffffffb;


const EHH_: u32 = 0;
const EHD_: u32 = 1;        
const EHE_: u32 = 6;  
const EHG_: u32 = 7; 
const EHI_: u32 = 8;  
const EHF_: u32 = 37;


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Yi {
    pub d_tag: i64,
    pub d_val: u64,
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Avh {
    pub jhy: u32,
    pub gwa: u8,
    pub jhz: u8,
    pub jia: u16,
    pub jib: u64,
    pub st_size: u64,
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Gl {
    pub r_offset: u64,
    pub r_info: u64,
    pub r_addend: i64,
}

impl Gl {
    pub fn sym_idx(&self) -> u32 { (self.r_info >> 32) as u32 }
    pub fn rel_type(&self) -> u32 { (self.r_info & 0xFFFF_FFFF) as u32 }
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],      
    pub e_type: u16,            
    pub e_machine: u16,         
    pub e_version: u32,         
    pub e_entry: u64,           
    pub e_phoff: u64,           
    pub ftx: u64,           
    pub e_flags: u32,           
    pub e_ehsize: u16,          
    pub e_phentsize: u16,       
    pub e_phnum: u16,           
    pub hur: u16,       
    pub ftv: u16,           
    pub fty: u16,        
}

impl Elf64Header {
    pub const Z: usize = 64;
    
    
    pub fn bsv(data: &[u8]) -> Option<&Self> {
        if data.len() < Self::Z {
            return None;
        }
        
        let header = unsafe { &*(data.as_ptr() as *const Self) };
        
        
        if header.e_ident[0..4] != ATI_ {
            return None;
        }
        
        
        if header.e_ident[4] != Aiv {
            return None;
        }
        
        
        if header.e_ident[5] != Aiw {
            return None;
        }
        
        
        if header.e_machine != BWI_ {
            return None;
        }
        
        Some(header)
    }
    
    
    pub fn is_executable(&self) -> bool {
        self.e_type == BXI_ || self.e_type == ATK_
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64Phdr {
    pub p_type: u32,        
    pub p_flags: u32,       
    pub p_offset: u64,      
    pub p_vaddr: u64,       
    pub p_paddr: u64,       
    pub p_filesz: u64,      
    pub p_memsz: u64,       
    pub p_align: u64,       
}

impl Elf64Phdr {
    pub const Z: usize = 56;
    
    
    pub fn czz(&self) -> bool {
        self.p_type == JM_
    }
    
    
    pub fn is_executable(&self) -> bool {
        (self.p_flags & CMO_) != 0
    }
    
    
    pub fn is_writable(&self) -> bool {
        (self.p_flags & CMN_) != 0
    }
    
    
    pub fn is_readable(&self) -> bool {
        (self.p_flags & CMM_) != 0
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Avg {
    pub sh_name: u32,       
    pub sh_type: u32,       
    pub fam: u64,      
    pub sh_addr: u64,       
    pub jfx: u64,     
    pub jfy: u64,       
    pub sh_link: u32,       
    pub sh_info: u32,       
    pub sh_addralign: u64,  
    pub guq: u64,    
}


#[derive(Clone, Debug)]
pub struct Aax {
    pub vaddr: u64,
    pub size: u64,
    pub flags: u32,
    pub data: Vec<u8>,
}


#[derive(Clone, Debug, Default)]
pub struct DynamicInfo {
    
    pub interp: Option<String>,
    
    pub needed_libs: Vec<String>,
    
    pub rela_offset: u64,
    pub rela_count: usize,
    
    pub jmprel_offset: u64,
    pub jmprel_count: usize,
    
    pub symtab_offset: u64,
    
    pub strtab_offset: u64,
    pub strtab_size: usize,
    
    pub init_addr: u64,
    pub fini_addr: u64,
    
    pub init_array_addr: u64,
    pub init_array_size: usize,
    pub fini_array_addr: u64,
    pub fini_array_size: usize,
    
    pub flags: u64,
    pub flags_1: u64,
    
    pub has_dynamic: bool,
}


#[derive(Clone, Debug)]
pub struct Mm {
    pub entry_point: u64,
    pub segments: Vec<Aax>,
    pub min_vaddr: u64,
    pub max_vaddr: u64,
    
    pub base_addr: u64,
    
    pub erq: bool,
    
    pub dynamic: DynamicInfo,
    
    pub relocations: Vec<Us>,
}


#[derive(Clone, Debug)]
pub struct Us {
    pub offset: u64,
    pub rel_type: u32,
    pub sym_idx: u32,
    pub addend: i64,
}


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

pub type Os<T> = Result<T, ElfError>;


pub fn nac(path: &str) -> Os<Mm> {
    
    let fd = crate::vfs::open(path, crate::vfs::OpenFlags(crate::vfs::OpenFlags::PM_))
        .map_err(|_| ElfError::IoError)?;
    
    
    let stat = crate::vfs::stat(path).map_err(|_| ElfError::IoError)?;
    let size = stat.size as usize;
    
    if size > 16 * 1024 * 1024 {  
        crate::vfs::close(fd).ok();
        return Err(ElfError::TooLarge);
    }
    
    
    let mut data = alloc::vec![0u8; size];
    crate::vfs::read(fd, &mut data).map_err(|_| ElfError::IoError)?;
    crate::vfs::close(fd).ok();
    
    
    gfw(&data)
}


pub fn gfw(data: &[u8]) -> Os<Mm> {
    
    let header = Elf64Header::bsv(data)
        .ok_or(ElfError::InvalidMagic)?;
    
    if !header.is_executable() {
        return Err(ElfError::NotExecutable);
    }
    
    let erq = header.e_type == ATK_;
    
    let base_addr: u64 = if erq { 0x0040_0000 } else { 0 };
    
    crate::log_debug!("[ELF] Loading {} executable, entry: {:#x}, base: {:#x}",
        if erq { "PIE" } else { "static" }, header.e_entry, base_addr);
    
    let mut segments = Vec::new();
    let mut min_vaddr = u64::MAX;
    let mut max_vaddr = 0u64;
    let mut aww = DynamicInfo::default();
    let mut hun: Option<(u64, u64)> = None; 
    
    
    let aii = header.e_phoff as usize;
    let but = header.e_phentsize as usize;
    let bur = header.e_phnum as usize;
    
    for i in 0..bur {
        let offset = aii + i * but;
        if offset + Elf64Phdr::Z > data.len() {
            return Err(ElfError::InvalidProgramHeader);
        }
        
        let rx = unsafe { &*(data[offset..].as_ptr() as *const Elf64Phdr) };
        
        match rx.p_type {
            XU_ => {
                
                let start = rx.p_offset as usize;
                let end = start + rx.p_filesz as usize;
                if end <= data.len() {
                    let gdd = &data[start..end];
                    
                    let len = gdd.iter().position(|&b| b == 0).unwrap_or(gdd.len());
                    if let Ok(j) = core::str::from_utf8(&gdd[..len]) {
                        aww.interp = Some(String::from(j));
                        crate::log_debug!("[ELF] PT_INTERP: {}", j);
                    }
                }
            }
            AIN_ => {
                aww.has_dynamic = true;
                hun = Some((rx.p_offset, rx.p_filesz));
            }
            JM_ => {
                let vaddr = rx.p_vaddr + base_addr;
                crate::log_debug!("[ELF] LOAD segment: vaddr={:#x}, filesz={}, memsz={}, flags={:#x}",
                    vaddr, rx.p_filesz, rx.p_memsz, rx.p_flags);
                
                if vaddr < min_vaddr { min_vaddr = vaddr; }
                if vaddr + rx.p_memsz > max_vaddr { max_vaddr = vaddr + rx.p_memsz; }
                
                let aaw = rx.p_offset as usize;
                let file_size = rx.p_filesz as usize;
                let bcr = rx.p_memsz as usize;
                
                if aaw + file_size > data.len() {
                    return Err(ElfError::InvalidProgramHeader);
                }
                
                let mut dyy = alloc::vec![0u8; bcr];
                dyy[..file_size].copy_from_slice(&data[aaw..aaw + file_size]);
                
                segments.push(Aax {
                    vaddr,
                    size: rx.p_memsz,
                    flags: rx.p_flags,
                    data: dyy,
                });
            }
            _ => {} 
        }
    }
    
    if segments.is_empty() {
        return Err(ElfError::InvalidProgramHeader);
    }
    
    
    let mut relocations = Vec::new();
    if let Some((dyn_off, dyn_sz)) = hun {
        let start = dyn_off as usize;
        let end = start + dyn_sz as usize;
        if end <= data.len() {
            gmh(data, start, end, base_addr, &mut aww);
        }
        
        if aww.rela_count > 0 && (aww.rela_offset as usize) < data.len() {
            let oeq = aww.rela_offset as usize;
            for i in 0..aww.rela_count {
                let off = oeq + i * core::mem::size_of::<Gl>();
                if off + core::mem::size_of::<Gl>() > data.len() { break; }
                let bvg = unsafe { &*(data[off..].as_ptr() as *const Gl) };
                relocations.push(Us {
                    offset: bvg.r_offset,
                    rel_type: bvg.rel_type(),
                    sym_idx: bvg.sym_idx(),
                    addend: bvg.r_addend,
                });
            }
        }
        
        if aww.jmprel_count > 0 && (aww.jmprel_offset as usize) < data.len() {
            let mvb = aww.jmprel_offset as usize;
            for i in 0..aww.jmprel_count {
                let off = mvb + i * core::mem::size_of::<Gl>();
                if off + core::mem::size_of::<Gl>() > data.len() { break; }
                let bvg = unsafe { &*(data[off..].as_ptr() as *const Gl) };
                relocations.push(Us {
                    offset: bvg.r_offset,
                    rel_type: bvg.rel_type(),
                    sym_idx: bvg.sym_idx(),
                    addend: bvg.r_addend,
                });
            }
        }
        crate::log_debug!("[ELF] Parsed {} relocations, {} needed libs",
            relocations.len(), aww.needed_libs.len());
    }
    
    Ok(Mm {
        entry_point: header.e_entry + base_addr,
        segments,
        min_vaddr,
        max_vaddr,
        base_addr,
        erq,
        dynamic: aww,
        relocations,
    })
}


fn gmh(data: &[u8], start: usize, end: usize, _base: u64, info: &mut DynamicInfo) {
    let oi = core::mem::size_of::<Yi>();
    let mut gqy: u64 = 0;
    let mut dxp: u64 = 0;
    let mut gnl: u64 = 0;
    let mut gwp: u64 = 0;
    let mut jjf: u64 = 0;
    let mut ipj: Vec<u64> = Vec::new();
    
    let mut off = start;
    while off + oi <= end {
        let ajn = unsafe { &*(data[off..].as_ptr() as *const Yi) };
        match ajn.d_tag {
            UB_ => break,
            UA_ => { ipj.push(ajn.d_val); }
            ADE_ => { gwp = ajn.d_val; }
            ADD_ => { jjf = ajn.d_val; }
            ADF_ => { info.symtab_offset = ajn.d_val; }
            ADA_ => { info.rela_offset = ajn.d_val; }
            ADC_ => { gqy = ajn.d_val; }
            ADB_ => { dxp = ajn.d_val; }
            ACZ_ => { info.jmprel_offset = ajn.d_val; }
            BVM_ => { gnl = ajn.d_val; }
            ACY_ => { info.init_addr = ajn.d_val; }
            ACW_ => { info.fini_addr = ajn.d_val; }
            BVK_ => { info.init_array_addr = ajn.d_val; }
            BVL_ => { info.init_array_size = ajn.d_val as usize; }
            BVG_ => { info.fini_array_addr = ajn.d_val; }
            BVH_ => { info.fini_array_size = ajn.d_val as usize; }
            ACX_ => { info.flags = ajn.d_val; }
            BVI_ => { info.flags_1 = ajn.d_val; }
            _ => {}
        }
        off += oi;
    }
    
    
    if dxp > 0 && gqy > 0 {
        info.rela_count = (gqy / dxp) as usize;
    }
    if gnl > 0 {
        let lqk = if dxp > 0 { dxp } else { core::mem::size_of::<Gl>() as u64 };
        info.jmprel_count = (gnl / lqk) as usize;
    }
    
    info.strtab_offset = gwp;
    info.strtab_size = jjf as usize;
    
    
    
    
    let eam = gwp as usize;
    if eam < data.len() {
        for &gio in &ipj {
            let sj = eam + gio as usize;
            if sj < data.len() {
                let lqc = data[sj..].iter().position(|&b| b == 0)
                    .unwrap_or(data.len() - sj);
                if let Ok(name) = core::str::from_utf8(&data[sj..sj + lqc]) {
                    info.needed_libs.push(String::from(name));
                }
            }
        }
    }
}


pub fn msm(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    data[0..4] == ATI_
}


pub fn rk(data: &[u8]) -> Os<(u64, usize)> {
    let header = Elf64Header::bsv(data)
        .ok_or(ElfError::InvalidMagic)?;
    
    Ok((header.e_entry, header.e_phnum as usize))
}
