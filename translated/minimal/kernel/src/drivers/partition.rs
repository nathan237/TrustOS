






use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;


const H_: usize = 512;


const CFX_: u16 = 0xAA55;


const BYD_: u64 = 0x5452415020494645;


const CFY_: u8 = 0xEE;






#[derive(Debug, Clone)]
pub struct Akz {
    
    pub aqb: u8,
    
    pub aag: u64,
    
    pub fuw: u64,
    
    pub duf: PartitionType,
    
    pub cji: bool,
    
    pub j: String,
    
    pub aar: Option<[u8; 16]>,
}

impl Akz {
    
    pub fn afz(&self) -> u64 {
        self.fuw * H_ as u64
    }
    
    
    pub fn ple(&self) -> String {
        let bf = self.afz();
        if bf >= 1024 * 1024 * 1024 * 1024 {
            format!("{:.1} TB", bf as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
        } else if bf >= 1024 * 1024 * 1024 {
            format!("{:.1} GB", bf as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bf >= 1024 * 1024 {
            format!("{:.1} MB", bf as f64 / (1024.0 * 1024.0))
        } else if bf >= 1024 {
            format!("{:.1} KB", bf as f64 / 1024.0)
        } else {
            format!("{} B", bf)
        }
    }
    
    
    pub fn fhr(&self) -> u64 {
        self.aag + self.fuw - 1
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionType {
    
    Jl,
    
    Bgr,
    
    Bgt,
    
    Bgs,
    
    Bgj,
    
    Asa,
    
    Asc,
    
    Awf,
    
    Auw,
    
    Blc,
    
    Blf,
    
    Arn,
    
    Bmf,
    
    Akg,
    
    Bld,
    
    Blh,
    
    Ble,
    
    Bia,
    
    F(u8),
    
    Bvj([u8; 16]),
}

impl PartitionType {
    
    pub fn syb(xnr: u8) -> Self {
        match xnr {
            0x00 => Self::Jl,
            0x01 => Self::Bgr,
            0x04 => Self::Bgt,
            0x05 | 0x0F => Self::Bgj,
            0x06 | 0x0E => Self::Bgs,
            0x07 => Self::Awf,
            0x0B => Self::Asa,
            0x0C => Self::Asc,
            0x82 => Self::Auw,
            0x83 => Self::Blc,
            0x8E => Self::Blf,
            0xEE => Self::Bia,
            0xEF => Self::Arn,
            gq => Self::F(gq),
        }
    }
    
    
    pub fn sxw(aar: &[u8; 16]) -> Self {
        
        
        
        
        const BTH_: [u8; 16] = [
            0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11,
            0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E, 0xC9, 0x3B
        ];
        
        
        const CHF_: [u8; 16] = [
            0xA2, 0xA0, 0xD0, 0xEB, 0xE5, 0xB9, 0x33, 0x44,
            0x87, 0xC0, 0x68, 0xB6, 0xB7, 0x26, 0x99, 0xC7
        ];
        
        
        const CHG_: [u8; 16] = [
            0x16, 0xE3, 0xC9, 0xE3, 0x5C, 0x0B, 0xB8, 0x4D,
            0x81, 0x7D, 0xF9, 0x2D, 0xF0, 0x02, 0x15, 0xAE
        ];
        
        
        const CEN_: [u8; 16] = [
            0xAF, 0x3D, 0xC6, 0x0F, 0x83, 0x84, 0x72, 0x47,
            0x8E, 0x79, 0x3D, 0x69, 0xD8, 0x47, 0x7D, 0xE4
        ];
        
        
        const CEQ_: [u8; 16] = [
            0x6D, 0xFD, 0x57, 0x06, 0xAB, 0xA4, 0xC4, 0x43,
            0x84, 0xE5, 0x09, 0x33, 0xC8, 0x4B, 0x4F, 0x4F
        ];
        
        
        const CEP_: [u8; 16] = [
            0xE3, 0xBC, 0x68, 0x4F, 0xCD, 0xE8, 0xB1, 0x4D,
            0x96, 0xE7, 0xFB, 0xCA, 0xF9, 0x84, 0xB7, 0x09
        ];
        
        
        const CEO_: [u8; 16] = [
            0xE1, 0xC7, 0x3A, 0x93, 0xB4, 0x2E, 0x13, 0x4F,
            0xB8, 0x44, 0x0E, 0x14, 0xE2, 0xAE, 0xF9, 0x15
        ];
        
        if aar == &BTH_ { Self::Arn }
        else if aar == &CHF_ { Self::Akg }
        else if aar == &CHG_ { Self::Bmf }
        else if aar == &CEN_ { Self::Bld }
        else if aar == &CEQ_ { Self::Auw }
        else if aar == &CEP_ { Self::Blh }
        else if aar == &CEO_ { Self::Ble }
        else if aar == &[0u8; 16] { Self::Jl }
        else { Self::Bvj(*aar) }
    }
    
    
    pub fn j(&self) -> &'static str {
        match self {
            Self::Jl => "Empty",
            Self::Bgr => "FAT12",
            Self::Bgt => "FAT16 (<32M)",
            Self::Bgs => "FAT16",
            Self::Bgj => "Extended",
            Self::Asa => "FAT32",
            Self::Asc => "FAT32 LBA",
            Self::Awf => "NTFS/exFAT",
            Self::Auw => "Linux swap",
            Self::Blc => "Linux",
            Self::Blf => "Linux LVM",
            Self::Arn => "EFI System",
            Self::Bmf => "MS Reserved",
            Self::Akg => "MS Basic Data",
            Self::Bld => "Linux",
            Self::Blh => "Linux root",
            Self::Ble => "Linux home",
            Self::Bia => "GPT Protective",
            Self::F(_) => "Unknown",
            Self::Bvj(_) => "Unknown GPT",
        }
    }
}






#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Chi {
    
    ilz: u8,
    
    zpm: [u8; 3],
    
    duf: u8,
    
    ypk: [u8; 3],
    
    aag: u32,
    
    fuw: u32,
}


#[repr(C, packed)]
struct Nm {
    
    ygs: [u8; 446],
    
    aqd: [Chi; 4],
    
    signature: u16,
}






#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Cem {
    
    signature: u64,
    
    afe: u32,
    
    drp: u32,
    
    ywy: u32,
    
    awt: u32,
    
    kmx: u64,
    
    yfi: u64,
    
    yqw: u64,
    
    zal: u64,
    
    gex: [u8; 16],
    
    ver: u64,
    
    uwk: u32,
    
    ves: u32,
    
    zeu: u32,
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Cen {
    
    fxq: [u8; 16],
    
    lta: [u8; 16],
    
    aag: u64,
    
    fhr: u64,
    
    fcv: u64,
    
    j: [u16; 36],
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionTableType {
    
    None,
    
    Nm,
    
    Wu,
}


#[derive(Debug, Clone)]
pub struct PartitionTable {
    
    pub gud: PartitionTableType,
    
    pub aqd: Vec<Akz>,
    
    pub gex: Option<[u8; 16]>,
    
    pub axf: u64,
}

impl PartitionTable {
    
    pub fn azs() -> Self {
        Self {
            gud: PartitionTableType::None,
            aqd: Vec::new(),
            gex: None,
            axf: 0,
        }
    }
}








pub fn hul<G>(xr: G, axf: u64) -> Result<PartitionTable, &'static str>
where
    G: Fn(u64, &mut [u8]) -> Result<(), &'static str>,
{
    let mut joc = [0u8; H_];
    xr(0, &mut joc)?;
    
    
    let signature = u16::dj([joc[510], joc[511]]);
    if signature != CFX_ {
        return Ok(PartitionTable::azs());
    }
    
    
    let jfo = unsafe { &*(joc.fq() as *const Nm) };
    
    
    let tmz = jfo.aqd.iter()
        .any(|ai| ai.duf == CFY_);
    
    if tmz {
        
        match lsk(&xr, axf) {
            Ok(gg) => return Ok(gg),
            Err(_) => {
                
            }
        }
    }
    
    
    lsp(jfo, axf)
}


fn lsp(jfo: &Nm, axf: u64) -> Result<PartitionTable, &'static str> {
    let mut aqd = Vec::new();
    
    for (a, bt) in jfo.aqd.iter().cf() {
        if bt.duf == 0 || bt.fuw == 0 {
            continue;
        }
        
        let partition = Akz {
            aqb: (a + 1) as u8,
            aag: bt.aag as u64,
            fuw: bt.fuw as u64,
            duf: PartitionType::syb(bt.duf),
            cji: bt.ilz == 0x80,
            j: String::new(),
            aar: None,
        };
        
        aqd.push(partition);
    }
    
    Ok(PartitionTable {
        gud: PartitionTableType::Nm,
        aqd,
        gex: None,
        axf,
    })
}


fn lsk<G>(xr: &G, axf: u64) -> Result<PartitionTable, &'static str>
where
    G: Fn(u64, &mut [u8]) -> Result<(), &'static str>,
{
    let mut phf = [0u8; H_];
    xr(1, &mut phf)?;
    
    let dh = unsafe { &*(phf.fq() as *const Cem) };
    
    
    if dh.signature != BYD_ {
        return Err("Invalid GPT signature");
    }
    
    let mut aqd = Vec::new();
    let acy = { dh.ves };
    let htd = { dh.uwk };
    let ktr = { dh.ver };
    let rye = { dh.gex };
    
    let ggh = H_ / acy as usize;
    let dbu = (htd as usize + ggh - 1) / ggh;
    
    let mut ouq = 1u8;
    
    for cmu in 0..dbu {
        let mut jk = [0u8; H_];
        xr(ktr + cmu as u64, &mut jk)?;
        
        for bea in 0..ggh {
            let l = bea * acy as usize;
            if l + 128 > H_ {
                break;
            }
            
            let bt = unsafe { 
                &*(jk.fq().add(l) as *const Cen) 
            };
            
            
            let fxq = { bt.fxq };
            let lta = { bt.lta };
            let aag = { bt.aag };
            let fhr = { bt.fhr };
            let fcv = { bt.fcv };
            let cxm = { bt.j };
            
            
            if fxq == [0u8; 16] {
                continue;
            }
            
            
            let mut j = String::new();
            for &r in &cxm {
                if r == 0 {
                    break;
                }
                if r < 128 {
                    j.push(r as u8 as char);
                }
            }
            
            let partition = Akz {
                aqb: ouq,
                aag,
                fuw: fhr - aag + 1,
                duf: PartitionType::sxw(&fxq),
                cji: (fcv & 0x04) != 0, 
                j,
                aar: Some(lta),
            };
            
            aqd.push(partition);
            ouq += 1;
        }
    }
    
    Ok(PartitionTable {
        gud: PartitionTableType::Wu,
        aqd,
        gex: Some(rye),
        axf,
    })
}


pub fn svt(aar: &[u8; 16]) -> String {
    
    
    format!(
        "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        aar[3], aar[2], aar[1], aar[0],
        aar[5], aar[4],
        aar[7], aar[6],
        aar[8], aar[9],
        aar[10], aar[11], aar[12], aar[13], aar[14], aar[15]
    )
}


pub fn oxt(gg: &PartitionTable) {
    match gg.gud {
        PartitionTableType::None => {
            crate::println!("No partition table found");
            return;
        }
        PartitionTableType::Nm => {
            crate::println!("Partition table: MBR");
        }
        PartitionTableType::Wu => {
            crate::println!("Partition table: GPT");
            if let Some(ref aar) = gg.gex {
                crate::println!("Disk GUID: {}", svt(aar));
            }
        }
    }
    
    if gg.aqd.is_empty() {
        crate::println!("No partitions found");
        return;
    }
    
    crate::println!();
    crate::println!("  #  Boot  Start LBA     End LBA       Size       Type");
    crate::println!("  ─────────────────────────────────────────────────────────");
    
    for ai in &gg.aqd {
        let ilz = if ai.cji { "*" } else { " " };
        crate::println!(
            "  {}  {}     {:>12}  {:>12}  {:>10}   {}",
            ai.aqb,
            ilz,
            ai.aag,
            ai.fhr(),
            ai.ple(),
            ai.duf.j()
        );
        
        if !ai.j.is_empty() {
            crate::println!("                                              Name: {}", ai.j);
        }
    }
}






pub fn lxn(port: u8) -> Result<PartitionTable, &'static str> {
    use super::ahci;
    
    if !ahci::ky() {
        return Err("AHCI not initialized");
    }
    
    
    let hgf = ahci::kyv(port).ok_or("Port not found")?;
    let axf = hgf.agw;
    
    
    let dld = |qa: u64, bi: &mut [u8]| -> Result<(), &'static str> {
        if bi.len() < H_ {
            return Err("Buffer too small");
        }
        let mut aae = [0u8; H_];
        ahci::ain(port, qa, 1, &mut aae)?;
        bi[..H_].dg(&aae);
        Ok(())
    };
    
    hul(dld, axf)
}
