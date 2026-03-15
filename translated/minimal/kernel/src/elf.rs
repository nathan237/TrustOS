



use alloc::vec::Vec;
use alloc::string::String;


const ARF_: [u8; 4] = [0x7F, b'E', b'L', b'F'];


const Cbd: u8 = 2;


const Cbe: u8 = 1; 


const BUM_: u16 = 2;    
const ARH_: u16 = 3;     


const BTM_: u16 = 62;


const ECB_: u32 = 0;
const IU_: u32 = 1;
const AGT_: u32 = 2;
const WL_: u32 = 3;
const BDR_: u32 = 4;
const ECC_: u32 = 6;
const BDP_: u32 = 0x6474e552;
const BDQ_: u32 = 0x6474e551;


pub const CJF_: u32 = 1; 
pub const CJE_: u32 = 2; 
pub const CJD_: u32 = 4; 


const SU_: i64 = 0;
const ST_: i64 = 1;     
const BSQ_: i64 = 2;  
const AQU_: i64 = 3;
const AQT_: i64 = 4;
const ABO_: i64 = 5;     
const ABP_: i64 = 6;    
const ABK_: i64 = 7;      
const ABM_: i64 = 8;
const ABL_: i64 = 9;
const ABN_: i64 = 10;
const DKX_: i64 = 11;
const ABI_: i64 = 12;
const ABG_: i64 = 13;
const AQW_: i64 = 14;
const AQV_: i64 = 15;
const BSS_: i64 = 16;
const DKU_: i64 = 17;
const DKW_: i64 = 18;
const DKV_: i64 = 19;
const DKT_: i64 = 20;
const DKS_: i64 = 21;
const DKY_: i64 = 22;
const ABJ_: i64 = 23;
const BSO_: i64 = 25;
const BSK_: i64 = 26;
const BSP_: i64 = 27;
const BSL_: i64 = 28;
const ABH_: i64 = 30;
const BSM_: i64 = 0x6ffffffb;


const EDP_: u32 = 0;
const EDL_: u32 = 1;        
const EDM_: u32 = 6;  
const EDO_: u32 = 7; 
const EDQ_: u32 = 8;  
const EDN_: u32 = 37;


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Bfv {
    pub rsz: i64,
    pub bmq: u64,
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Cvv {
    pub pnm: u32,
    pub mhf: u8,
    pub pno: u8,
    pub pnp: u16,
    pub pnr: u64,
    pub gsz: u64,
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Ph {
    pub jla: u64,
    pub hwk: u64,
    pub jky: i64,
}

impl Ph {
    pub fn dwm(&self) -> u32 { (self.hwk >> 32) as u32 }
    pub fn fsp(&self) -> u32 { (self.hwk & 0xFFFF_FFFF) as u32 }
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64Header {
    pub ksg: [u8; 16],      
    pub ceh: u16,            
    pub cqb: u16,         
    pub yof: u32,         
    pub cxe: u64,           
    pub epo: u64,           
    pub ksl: u64,           
    pub yod: u32,           
    pub yoc: u16,          
    pub fhh: u16,       
    pub dqk: u16,           
    pub nov: u16,       
    pub ksk: u16,           
    pub ksm: u16,        
}

impl Elf64Header {
    pub const Am: usize = 64;
    
    
    pub fn eca(f: &[u8]) -> Option<&Self> {
        if f.len() < Self::Am {
            return None;
        }
        
        let dh = unsafe { &*(f.fq() as *const Self) };
        
        
        if dh.ksg[0..4] != ARF_ {
            return None;
        }
        
        
        if dh.ksg[4] != Cbd {
            return None;
        }
        
        
        if dh.ksg[5] != Cbe {
            return None;
        }
        
        
        if dh.cqb != BTM_ {
            return None;
        }
        
        Some(dh)
    }
    
    
    pub fn clc(&self) -> bool {
        self.ceh == BUM_ || self.ceh == ARH_
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64Phdr {
    pub bku: u32,        
    pub bvv: u32,       
    pub caz: u64,      
    pub ctg: u64,       
    pub zeo: u64,       
    pub cgh: u64,      
    pub ctf: u64,       
    pub zen: u64,       
}

impl Elf64Phdr {
    pub const Am: usize = 56;
    
    
    pub fn gkf(&self) -> bool {
        self.bku == IU_
    }
    
    
    pub fn clc(&self) -> bool {
        (self.bvv & CJF_) != 0
    }
    
    
    pub fn edz(&self) -> bool {
        (self.bvv & CJE_) != 0
    }
    
    
    pub fn ogr(&self) -> bool {
        (self.bvv & CJD_) != 0
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Cvu {
    pub pjv: u32,       
    pub dbx: u32,       
    pub jpp: u64,      
    pub zny: u64,       
    pub pjy: u64,     
    pub pjz: u64,       
    pub zob: u32,       
    pub zoa: u32,       
    pub znz: u64,  
    pub mfh: u64,    
}


#[derive(Clone, Debug)]
pub struct Blm {
    pub uy: u64,
    pub aw: u64,
    pub flags: u32,
    pub f: Vec<u8>,
}


#[derive(Clone, Debug, Default)]
pub struct DynamicInfo {
    
    pub ahp: Option<String>,
    
    pub djt: Vec<String>,
    
    pub lyx: u64,
    pub lyw: usize,
    
    pub lgw: u64,
    pub lgv: usize,
    
    pub wwz: u64,
    
    pub wvg: u64,
    pub mhy: usize,
    
    pub tsy: u64,
    pub sts: u64,
    
    pub ttc: u64,
    pub ttd: usize,
    pub stt: u64,
    pub stu: usize,
    
    pub flags: u64,
    pub sum: u64,
    
    pub tmh: bool,
}


#[derive(Clone, Debug)]
pub struct Acr {
    pub mi: u64,
    pub jq: Vec<Blm>,
    pub foj: u64,
    pub gmk: u64,
    
    pub sm: u64,
    
    pub jbo: bool,
    
    pub hhn: DynamicInfo,
    
    pub bwp: Vec<Axu>,
}


#[derive(Clone, Debug)]
pub struct Axu {
    pub l: u64,
    pub fsp: u32,
    pub dwm: u32,
    pub fcn: i64,
}


#[derive(Clone, Copy, Debug)]
pub enum ElfError {
    Bju,
    Czw,
    Czx,
    Chz,
    Aum,
    Av,
    Cod,
    Ns,
}

pub type Ahr<T> = Result<T, ElfError>;


pub fn ugx(path: &str) -> Ahr<Acr> {
    
    let da = crate::vfs::aji(path, crate::vfs::OpenFlags(crate::vfs::OpenFlags::OO_))
        .jd(|_| ElfError::Av)?;
    
    
    let hm = crate::vfs::hm(path).jd(|_| ElfError::Av)?;
    let aw = hm.aw as usize;
    
    if aw > 16 * 1024 * 1024 {  
        crate::vfs::agj(da).bq();
        return Err(ElfError::Cod);
    }
    
    
    let mut f = alloc::vec![0u8; aw];
    crate::vfs::read(da, &mut f).jd(|_| ElfError::Av)?;
    crate::vfs::agj(da).bq();
    
    
    ljf(&f)
}


pub fn ljf(f: &[u8]) -> Ahr<Acr> {
    
    let dh = Elf64Header::eca(f)
        .ok_or(ElfError::Bju)?;
    
    if !dh.clc() {
        return Err(ElfError::Chz);
    }
    
    let jbo = dh.ceh == ARH_;
    
    let sm: u64 = if jbo { 0x0040_0000 } else { 0 };
    
    crate::log_debug!("[ELF] Loading {} executable, entry: {:#x}, base: {:#x}",
        if jbo { "PIE" } else { "static" }, dh.cxe, sm);
    
    let mut jq = Vec::new();
    let mut foj = u64::O;
    let mut gmk = 0u64;
    let mut cqa = DynamicInfo::default();
    let mut noo: Option<(u64, u64)> = None; 
    
    
    let bnu = dh.epo as usize;
    let egq = dh.fhh as usize;
    let egp = dh.dqk as usize;
    
    for a in 0..egp {
        let l = bnu + a * egq;
        if l + Elf64Phdr::Am > f.len() {
            return Err(ElfError::Aum);
        }
        
        let ajj = unsafe { &*(f[l..].fq() as *const Elf64Phdr) };
        
        match ajj.bku {
            WL_ => {
                
                let ay = ajj.caz as usize;
                let ci = ay + ajj.cgh as usize;
                if ci <= f.len() {
                    let lfc = &f[ay..ci];
                    
                    let len = lfc.iter().qf(|&o| o == 0).unwrap_or(lfc.len());
                    if let Ok(e) = core::str::jg(&lfc[..len]) {
                        cqa.ahp = Some(String::from(e));
                        crate::log_debug!("[ELF] PT_INTERP: {}", e);
                    }
                }
            }
            AGT_ => {
                cqa.tmh = true;
                noo = Some((ajj.caz, ajj.cgh));
            }
            IU_ => {
                let uy = ajj.ctg + sm;
                crate::log_debug!("[ELF] LOAD segment: vaddr={:#x}, filesz={}, memsz={}, flags={:#x}",
                    uy, ajj.cgh, ajj.ctf, ajj.bvv);
                
                if uy < foj { foj = uy; }
                if uy + ajj.ctf > gmk { gmk = uy + ajj.ctf; }
                
                let azv = ajj.caz as usize;
                let yy = ajj.cgh as usize;
                let czr = ajj.ctf as usize;
                
                if azv + yy > f.len() {
                    return Err(ElfError::Aum);
                }
                
                let mut hzl = alloc::vec![0u8; czr];
                hzl[..yy].dg(&f[azv..azv + yy]);
                
                jq.push(Blm {
                    uy,
                    aw: ajj.ctf,
                    flags: ajj.bvv,
                    f: hzl,
                });
            }
            _ => {} 
        }
    }
    
    if jq.is_empty() {
        return Err(ElfError::Aum);
    }
    
    
    let mut bwp = Vec::new();
    if let Some((shx, shy)) = noo {
        let ay = shx as usize;
        let ci = ay + shy as usize;
        if ci <= f.len() {
            lsi(f, ay, ci, sm, &mut cqa);
        }
        
        if cqa.lyw > 0 && (cqa.lyx as usize) < f.len() {
            let vul = cqa.lyx as usize;
            for a in 0..cqa.lyw {
                let dz = vul + a * core::mem::size_of::<Ph>();
                if dz + core::mem::size_of::<Ph>() > f.len() { break; }
                let ehk = unsafe { &*(f[dz..].fq() as *const Ph) };
                bwp.push(Axu {
                    l: ehk.jla,
                    fsp: ehk.fsp(),
                    dwm: ehk.dwm(),
                    fcn: ehk.jky,
                });
            }
        }
        
        if cqa.lgv > 0 && (cqa.lgw as usize) < f.len() {
            let ual = cqa.lgw as usize;
            for a in 0..cqa.lgv {
                let dz = ual + a * core::mem::size_of::<Ph>();
                if dz + core::mem::size_of::<Ph>() > f.len() { break; }
                let ehk = unsafe { &*(f[dz..].fq() as *const Ph) };
                bwp.push(Axu {
                    l: ehk.jla,
                    fsp: ehk.fsp(),
                    dwm: ehk.dwm(),
                    fcn: ehk.jky,
                });
            }
        }
        crate::log_debug!("[ELF] Parsed {} relocations, {} needed libs",
            bwp.len(), cqa.djt.len());
    }
    
    Ok(Acr {
        mi: dh.cxe + sm,
        jq,
        foj,
        gmk,
        sm,
        jbo,
        hhn: cqa,
        bwp,
    })
}


fn lsi(f: &[u8], ay: usize, ci: usize, xxz: u64, co: &mut DynamicInfo) {
    let acy = core::mem::size_of::<Bfv>();
    let mut lyy: u64 = 0;
    let mut hxj: u64 = 0;
    let mut luk: u64 = 0;
    let mut mhx: u64 = 0;
    let mut ppf: u64 = 0;
    let mut opd: Vec<u64> = Vec::new();
    
    let mut dz = ay;
    while dz + acy <= ci {
        let bqi = unsafe { &*(f[dz..].fq() as *const Bfv) };
        match bqi.rsz {
            SU_ => break,
            ST_ => { opd.push(bqi.bmq); }
            ABO_ => { mhx = bqi.bmq; }
            ABN_ => { ppf = bqi.bmq; }
            ABP_ => { co.wwz = bqi.bmq; }
            ABK_ => { co.lyx = bqi.bmq; }
            ABM_ => { lyy = bqi.bmq; }
            ABL_ => { hxj = bqi.bmq; }
            ABJ_ => { co.lgw = bqi.bmq; }
            BSQ_ => { luk = bqi.bmq; }
            ABI_ => { co.tsy = bqi.bmq; }
            ABG_ => { co.sts = bqi.bmq; }
            BSO_ => { co.ttc = bqi.bmq; }
            BSP_ => { co.ttd = bqi.bmq as usize; }
            BSK_ => { co.stt = bqi.bmq; }
            BSL_ => { co.stu = bqi.bmq as usize; }
            ABH_ => { co.flags = bqi.bmq; }
            BSM_ => { co.sum = bqi.bmq; }
            _ => {}
        }
        dz += acy;
    }
    
    
    if hxj > 0 && lyy > 0 {
        co.lyw = (lyy / hxj) as usize;
    }
    if luk > 0 {
        let slw = if hxj > 0 { hxj } else { core::mem::size_of::<Ph>() as u64 };
        co.lgv = (luk / slw) as usize;
    }
    
    co.wvg = mhx;
    co.mhy = ppf as usize;
    
    
    
    
    let iby = mhx as usize;
    if iby < f.len() {
        for &lnj in &opd {
            let akj = iby + lnj as usize;
            if akj < f.len() {
                let sln = f[akj..].iter().qf(|&o| o == 0)
                    .unwrap_or(f.len() - akj);
                if let Ok(j) = core::str::jg(&f[akj..akj + sln]) {
                    co.djt.push(String::from(j));
                }
            }
        }
    }
}


pub fn txj(f: &[u8]) -> bool {
    if f.len() < 4 {
        return false;
    }
    f[0..4] == ARF_
}


pub fn ani(f: &[u8]) -> Ahr<(u64, usize)> {
    let dh = Elf64Header::eca(f)
        .ok_or(ElfError::Bju)?;
    
    Ok((dh.cxe, dh.dqk as usize))
}
