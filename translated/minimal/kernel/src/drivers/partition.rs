






use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;


const H_: usize = 512;


const CJH_: u16 = 0xAA55;


const CBJ_: u64 = 0x5452415020494645;


const CJI_: u8 = 0xEE;






#[derive(Debug, Clone)]
pub struct Pv {
    
    pub number: u8,
    
    pub start_lba: u64,
    
    pub size_sectors: u64,
    
    pub partition_type: PartitionType,
    
    pub bootable: bool,
    
    pub name: String,
    
    pub guid: Option<[u8; 16]>,
}

impl Pv {
    
    pub fn size_bytes(&self) -> u64 {
        self.size_sectors * H_ as u64
    }
    
    
    pub fn size_human(&self) -> String {
        let bytes = self.size_bytes();
        if bytes >= 1024 * 1024 * 1024 * 1024 {
            format!("{:.1} TB", bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 * 1024 {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }
    
    
    pub fn end_lba(&self) -> u64 {
        self.start_lba + self.size_sectors - 1
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionType {
    
    Empty,
    
    Fat12,
    
    Fat16Small,
    
    Fat16,
    
    Extended,
    
    Fat32,
    
    Fat32Lba,
    
    Ntfs,
    
    LinuxSwap,
    
    LinuxFilesystem,
    
    LinuxLvm,
    
    EfiSystem,
    
    MicrosoftReserved,
    
    MicrosoftBasicData,
    
    LinuxFilesystemGpt,
    
    LinuxRoot,
    
    LinuxHome,
    
    GptProtective,
    
    Unknown(u8),
    
    UnknownGpt([u8; 16]),
}

impl PartitionType {
    
    pub fn lzl(type_byte: u8) -> Self {
        match type_byte {
            0x00 => Self::Empty,
            0x01 => Self::Fat12,
            0x04 => Self::Fat16Small,
            0x05 | 0x0F => Self::Extended,
            0x06 | 0x0E => Self::Fat16,
            0x07 => Self::Ntfs,
            0x0B => Self::Fat32,
            0x0C => Self::Fat32Lba,
            0x82 => Self::LinuxSwap,
            0x83 => Self::LinuxFilesystem,
            0x8E => Self::LinuxLvm,
            0xEE => Self::GptProtective,
            0xEF => Self::EfiSystem,
            other => Self::Unknown(other),
        }
    }
    
    
    pub fn lzg(guid: &[u8; 16]) -> Self {
        
        
        
        
        const BWD_: [u8; 16] = [
            0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11,
            0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E, 0xC9, 0x3B
        ];
        
        
        const CKO_: [u8; 16] = [
            0xA2, 0xA0, 0xD0, 0xEB, 0xE5, 0xB9, 0x33, 0x44,
            0x87, 0xC0, 0x68, 0xB6, 0xB7, 0x26, 0x99, 0xC7
        ];
        
        
        const CKP_: [u8; 16] = [
            0x16, 0xE3, 0xC9, 0xE3, 0x5C, 0x0B, 0xB8, 0x4D,
            0x81, 0x7D, 0xF9, 0x2D, 0xF0, 0x02, 0x15, 0xAE
        ];
        
        
        const CHW_: [u8; 16] = [
            0xAF, 0x3D, 0xC6, 0x0F, 0x83, 0x84, 0x72, 0x47,
            0x8E, 0x79, 0x3D, 0x69, 0xD8, 0x47, 0x7D, 0xE4
        ];
        
        
        const CHZ_: [u8; 16] = [
            0x6D, 0xFD, 0x57, 0x06, 0xAB, 0xA4, 0xC4, 0x43,
            0x84, 0xE5, 0x09, 0x33, 0xC8, 0x4B, 0x4F, 0x4F
        ];
        
        
        const CHY_: [u8; 16] = [
            0xE3, 0xBC, 0x68, 0x4F, 0xCD, 0xE8, 0xB1, 0x4D,
            0x96, 0xE7, 0xFB, 0xCA, 0xF9, 0x84, 0xB7, 0x09
        ];
        
        
        const CHX_: [u8; 16] = [
            0xE1, 0xC7, 0x3A, 0x93, 0xB4, 0x2E, 0x13, 0x4F,
            0xB8, 0x44, 0x0E, 0x14, 0xE2, 0xAE, 0xF9, 0x15
        ];
        
        if guid == &BWD_ { Self::EfiSystem }
        else if guid == &CKO_ { Self::MicrosoftBasicData }
        else if guid == &CKP_ { Self::MicrosoftReserved }
        else if guid == &CHW_ { Self::LinuxFilesystemGpt }
        else if guid == &CHZ_ { Self::LinuxSwap }
        else if guid == &CHY_ { Self::LinuxRoot }
        else if guid == &CHX_ { Self::LinuxHome }
        else if guid == &[0u8; 16] { Self::Empty }
        else { Self::UnknownGpt(*guid) }
    }
    
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Fat12 => "FAT12",
            Self::Fat16Small => "FAT16 (<32M)",
            Self::Fat16 => "FAT16",
            Self::Extended => "Extended",
            Self::Fat32 => "FAT32",
            Self::Fat32Lba => "FAT32 LBA",
            Self::Ntfs => "NTFS/exFAT",
            Self::LinuxSwap => "Linux swap",
            Self::LinuxFilesystem => "Linux",
            Self::LinuxLvm => "Linux LVM",
            Self::EfiSystem => "EFI System",
            Self::MicrosoftReserved => "MS Reserved",
            Self::MicrosoftBasicData => "MS Basic Data",
            Self::LinuxFilesystemGpt => "Linux",
            Self::LinuxRoot => "Linux root",
            Self::LinuxHome => "Linux home",
            Self::GptProtective => "GPT Protective",
            Self::Unknown(_) => "Unknown",
            Self::UnknownGpt(_) => "Unknown GPT",
        }
    }
}






#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Amt {
    
    boot_flag: u8,
    
    start_chs: [u8; 3],
    
    partition_type: u8,
    
    end_chs: [u8; 3],
    
    start_lba: u32,
    
    size_sectors: u32,
}


#[repr(C, packed)]
struct Fu {
    
    boot_code: [u8; 446],
    
    partitions: [Amt; 4],
    
    signature: u16,
}






#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Aku {
    
    signature: u64,
    
    revision: u32,
    
    bms: u32,
    
    header_crc32: u32,
    
    reserved: u32,
    
    fpr: u64,
    
    backup_lba: u64,
    
    first_usable_lba: u64,
    
    last_usable_lba: u64,
    
    disk_guid: [u8; 16],
    
    partition_entry_lba: u64,
    
    num_partition_entries: u32,
    
    partition_entry_size: u32,
    
    partition_entries_crc32: u32,
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Akv {
    
    type_guid: [u8; 16],
    
    partition_guid: [u8; 16],
    
    start_lba: u64,
    
    end_lba: u64,
    
    attributes: u64,
    
    name: [u16; 36],
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionTableType {
    
    None,
    
    Fu,
    
    Gpt,
}


#[derive(Debug, Clone)]
pub struct PartitionTable {
    
    pub table_type: PartitionTableType,
    
    pub partitions: Vec<Pv>,
    
    pub disk_guid: Option<[u8; 16]>,
    
    pub zp: u64,
}

impl PartitionTable {
    
    pub fn empty() -> Self {
        Self {
            table_type: PartitionTableType::None,
            partitions: Vec::new(),
            disk_guid: None,
            zp: 0,
        }
    }
}








pub fn dwf<F>(read_sector: F, zp: u64) -> Result<PartitionTable, &'static str>
where
    F: Fn(u64, &mut [u8]) -> Result<(), &'static str>,
{
    let mut ezv = [0u8; H_];
    read_sector(0, &mut ezv)?;
    
    
    let signature = u16::from_le_bytes([ezv[510], ezv[511]]);
    if signature != CJH_ {
        return Ok(PartitionTable::empty());
    }
    
    
    let eue = unsafe { &*(ezv.as_ptr() as *const Fu) };
    
    
    let mjx = eue.partitions.iter()
        .any(|aa| aa.partition_type == CJI_);
    
    if mjx {
        
        match gmj(&read_sector, zp) {
            Ok(bs) => return Ok(bs),
            Err(_) => {
                
            }
        }
    }
    
    
    gmm(eue, zp)
}


fn gmm(eue: &Fu, zp: u64) -> Result<PartitionTable, &'static str> {
    let mut partitions = Vec::new();
    
    for (i, entry) in eue.partitions.iter().enumerate() {
        if entry.partition_type == 0 || entry.size_sectors == 0 {
            continue;
        }
        
        let partition = Pv {
            number: (i + 1) as u8,
            start_lba: entry.start_lba as u64,
            size_sectors: entry.size_sectors as u64,
            partition_type: PartitionType::lzl(entry.partition_type),
            bootable: entry.boot_flag == 0x80,
            name: String::new(),
            guid: None,
        };
        
        partitions.push(partition);
    }
    
    Ok(PartitionTable {
        table_type: PartitionTableType::Fu,
        partitions,
        disk_guid: None,
        zp,
    })
}


fn gmj<F>(read_sector: &F, zp: u64) -> Result<PartitionTable, &'static str>
where
    F: Fn(u64, &mut [u8]) -> Result<(), &'static str>,
{
    let mut jdz = [0u8; H_];
    read_sector(1, &mut jdz)?;
    
    let header = unsafe { &*(jdz.as_ptr() as *const Aku) };
    
    
    if header.signature != CBJ_ {
        return Err("Invalid GPT signature");
    }
    
    let mut partitions = Vec::new();
    let oi = { header.partition_entry_size };
    let dvn = { header.num_partition_entries };
    let fuz = { header.partition_entry_lba };
    let lfd = { header.disk_guid };
    
    let cxf = H_ / oi as usize;
    let bdq = (dvn as usize + cxf - 1) / cxf;
    
    let mut itz = 1u8;
    
    for avb in 0..bdq {
        let mut dj = [0u8; H_];
        read_sector(fuz + avb as u64, &mut dj)?;
        
        for ado in 0..cxf {
            let offset = ado * oi as usize;
            if offset + 128 > H_ {
                break;
            }
            
            let entry = unsafe { 
                &*(dj.as_ptr().add(offset) as *const Akv) 
            };
            
            
            let type_guid = { entry.type_guid };
            let partition_guid = { entry.partition_guid };
            let start_lba = { entry.start_lba };
            let end_lba = { entry.end_lba };
            let attributes = { entry.attributes };
            let bbl = { entry.name };
            
            
            if type_guid == [0u8; 16] {
                continue;
            }
            
            
            let mut name = String::new();
            for &c in &bbl {
                if c == 0 {
                    break;
                }
                if c < 128 {
                    name.push(c as u8 as char);
                }
            }
            
            let partition = Pv {
                number: itz,
                start_lba,
                size_sectors: end_lba - start_lba + 1,
                partition_type: PartitionType::lzg(&type_guid),
                bootable: (attributes & 0x04) != 0, 
                name,
                guid: Some(partition_guid),
            };
            
            partitions.push(partition);
            itz += 1;
        }
    }
    
    Ok(PartitionTable {
        table_type: PartitionTableType::Gpt,
        partitions,
        disk_guid: Some(lfd),
        zp,
    })
}


pub fn lxl(guid: &[u8; 16]) -> String {
    
    
    format!(
        "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        guid[3], guid[2], guid[1], guid[0],
        guid[5], guid[4],
        guid[7], guid[6],
        guid[8], guid[9],
        guid[10], guid[11], guid[12], guid[13], guid[14], guid[15]
    )
}


pub fn iwn(bs: &PartitionTable) {
    match bs.table_type {
        PartitionTableType::None => {
            crate::println!("No partition table found");
            return;
        }
        PartitionTableType::Fu => {
            crate::println!("Partition table: MBR");
        }
        PartitionTableType::Gpt => {
            crate::println!("Partition table: GPT");
            if let Some(ref guid) = bs.disk_guid {
                crate::println!("Disk GUID: {}", lxl(guid));
            }
        }
    }
    
    if bs.partitions.is_empty() {
        crate::println!("No partitions found");
        return;
    }
    
    crate::println!();
    crate::println!("  #  Boot  Start LBA     End LBA       Size       Type");
    crate::println!("  ─────────────────────────────────────────────────────────");
    
    for aa in &bs.partitions {
        let boot_flag = if aa.bootable { "*" } else { " " };
        crate::println!(
            "  {}  {}     {:>12}  {:>12}  {:>10}   {}",
            aa.number,
            boot_flag,
            aa.start_lba,
            aa.end_lba(),
            aa.size_human(),
            aa.partition_type.name()
        );
        
        if !aa.name.is_empty() {
            crate::println!("                                              Name: {}", aa.name);
        }
    }
}






pub fn gqd(port: u8) -> Result<PartitionTable, &'static str> {
    use super::ahci;
    
    if !ahci::is_initialized() {
        return Err("AHCI not initialized");
    }
    
    
    let dne = ahci::fyw(port).ok_or("Port not found")?;
    let zp = dne.sector_count;
    
    
    let read_fn = |hb: u64, buffer: &mut [u8]| -> Result<(), &'static str> {
        if buffer.len() < H_ {
            return Err("Buffer too small");
        }
        let mut mx = [0u8; H_];
        ahci::read_sectors(port, hb, 1, &mut mx)?;
        buffer[..H_].copy_from_slice(&mx);
        Ok(())
    };
    
    dwf(read_fn, zp)
}
