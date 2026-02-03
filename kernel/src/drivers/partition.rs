//! Partition Table Parser
//!
//! Supports MBR (Master Boot Record) and GPT (GUID Partition Table) formats.
//! 
//! MBR: Legacy format, max 4 primary partitions, 2TB limit
//! GPT: Modern format, 128+ partitions, 18 EB limit

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

/// Sector size in bytes
const SECTOR_SIZE: usize = 512;

/// MBR signature (last 2 bytes of sector 0)
const MBR_SIGNATURE: u16 = 0xAA55;

/// GPT signature "EFI PART"
const GPT_SIGNATURE: u64 = 0x5452415020494645;

/// Protective MBR partition type for GPT
const MBR_TYPE_GPT_PROTECTIVE: u8 = 0xEE;

// ============================================================================
// Partition Entry (unified representation)
// ============================================================================

/// Partition information (works for both MBR and GPT)
#[derive(Debug, Clone)]
pub struct Partition {
    /// Partition number (1-based)
    pub number: u8,
    /// Starting LBA
    pub start_lba: u64,
    /// Size in sectors
    pub size_sectors: u64,
    /// Partition type
    pub partition_type: PartitionType,
    /// Bootable flag
    pub bootable: bool,
    /// Partition name (GPT only)
    pub name: String,
    /// Partition GUID (GPT only)
    pub guid: Option<[u8; 16]>,
}

impl Partition {
    /// Get size in bytes
    pub fn size_bytes(&self) -> u64 {
        self.size_sectors * SECTOR_SIZE as u64
    }
    
    /// Get size in human readable format
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
    
    /// Get ending LBA
    pub fn end_lba(&self) -> u64 {
        self.start_lba + self.size_sectors - 1
    }
}

/// Partition type (common types)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionType {
    /// Empty/unused
    Empty,
    /// FAT12
    Fat12,
    /// FAT16 (< 32MB)
    Fat16Small,
    /// FAT16 (>= 32MB)
    Fat16,
    /// Extended partition (MBR)
    Extended,
    /// FAT32
    Fat32,
    /// FAT32 with LBA
    Fat32Lba,
    /// NTFS / exFAT
    Ntfs,
    /// Linux swap
    LinuxSwap,
    /// Linux filesystem (ext2/3/4)
    LinuxFilesystem,
    /// Linux LVM
    LinuxLvm,
    /// EFI System Partition
    EfiSystem,
    /// Microsoft Reserved
    MicrosoftReserved,
    /// Microsoft Basic Data (NTFS, FAT32)
    MicrosoftBasicData,
    /// Linux Filesystem (GPT)
    LinuxFilesystemGpt,
    /// Linux Root (x86-64)
    LinuxRoot,
    /// Linux Home
    LinuxHome,
    /// GPT Protective MBR
    GptProtective,
    /// Unknown type
    Unknown(u8),
    /// Unknown GPT type
    UnknownGpt([u8; 16]),
}

impl PartitionType {
    /// Parse from MBR type byte
    pub fn from_mbr(type_byte: u8) -> Self {
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
    
    /// Parse from GPT GUID
    pub fn from_gpt_guid(guid: &[u8; 16]) -> Self {
        // GUIDs are stored in mixed-endian format
        // Compare against known GUIDs
        
        // EFI System: C12A7328-F81F-11D2-BA4B-00A0C93EC93B
        const EFI_SYSTEM: [u8; 16] = [
            0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11,
            0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E, 0xC9, 0x3B
        ];
        
        // Microsoft Basic Data: EBD0A0A2-B9E5-4433-87C0-68B6B72699C7
        const MS_BASIC_DATA: [u8; 16] = [
            0xA2, 0xA0, 0xD0, 0xEB, 0xE5, 0xB9, 0x33, 0x44,
            0x87, 0xC0, 0x68, 0xB6, 0xB7, 0x26, 0x99, 0xC7
        ];
        
        // Microsoft Reserved: E3C9E316-0B5C-4DB8-817D-F92DF00215AE
        const MS_RESERVED: [u8; 16] = [
            0x16, 0xE3, 0xC9, 0xE3, 0x5C, 0x0B, 0xB8, 0x4D,
            0x81, 0x7D, 0xF9, 0x2D, 0xF0, 0x02, 0x15, 0xAE
        ];
        
        // Linux Filesystem: 0FC63DAF-8483-4772-8E79-3D69D8477DE4
        const LINUX_FS: [u8; 16] = [
            0xAF, 0x3D, 0xC6, 0x0F, 0x83, 0x84, 0x72, 0x47,
            0x8E, 0x79, 0x3D, 0x69, 0xD8, 0x47, 0x7D, 0xE4
        ];
        
        // Linux Swap: 0657FD6D-A4AB-43C4-84E5-0933C84B4F4F
        const LINUX_SWAP: [u8; 16] = [
            0x6D, 0xFD, 0x57, 0x06, 0xAB, 0xA4, 0xC4, 0x43,
            0x84, 0xE5, 0x09, 0x33, 0xC8, 0x4B, 0x4F, 0x4F
        ];
        
        // Linux Root x86-64: 4F68BCE3-E8CD-4DB1-96E7-FBCAF984B709
        const LINUX_ROOT: [u8; 16] = [
            0xE3, 0xBC, 0x68, 0x4F, 0xCD, 0xE8, 0xB1, 0x4D,
            0x96, 0xE7, 0xFB, 0xCA, 0xF9, 0x84, 0xB7, 0x09
        ];
        
        // Linux Home: 933AC7E1-2EB4-4F13-B844-0E14E2AEF915
        const LINUX_HOME: [u8; 16] = [
            0xE1, 0xC7, 0x3A, 0x93, 0xB4, 0x2E, 0x13, 0x4F,
            0xB8, 0x44, 0x0E, 0x14, 0xE2, 0xAE, 0xF9, 0x15
        ];
        
        if guid == &EFI_SYSTEM { Self::EfiSystem }
        else if guid == &MS_BASIC_DATA { Self::MicrosoftBasicData }
        else if guid == &MS_RESERVED { Self::MicrosoftReserved }
        else if guid == &LINUX_FS { Self::LinuxFilesystemGpt }
        else if guid == &LINUX_SWAP { Self::LinuxSwap }
        else if guid == &LINUX_ROOT { Self::LinuxRoot }
        else if guid == &LINUX_HOME { Self::LinuxHome }
        else if guid == &[0u8; 16] { Self::Empty }
        else { Self::UnknownGpt(*guid) }
    }
    
    /// Get human-readable name
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

// ============================================================================
// MBR Structures
// ============================================================================

/// MBR Partition Entry (16 bytes)
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct MbrPartitionEntry {
    /// Boot indicator (0x80 = bootable)
    boot_flag: u8,
    /// Starting CHS (legacy, ignored)
    start_chs: [u8; 3],
    /// Partition type
    partition_type: u8,
    /// Ending CHS (legacy, ignored)
    end_chs: [u8; 3],
    /// Starting LBA
    start_lba: u32,
    /// Size in sectors
    size_sectors: u32,
}

/// MBR (Master Boot Record) - Sector 0
#[repr(C, packed)]
struct Mbr {
    /// Boot code (446 bytes)
    boot_code: [u8; 446],
    /// Partition entries (4 x 16 bytes)
    partitions: [MbrPartitionEntry; 4],
    /// Signature (0xAA55)
    signature: u16,
}

// ============================================================================
// GPT Structures
// ============================================================================

/// GPT Header (LBA 1)
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GptHeader {
    /// Signature "EFI PART"
    signature: u64,
    /// Revision (usually 0x00010000)
    revision: u32,
    /// Header size (usually 92)
    header_size: u32,
    /// CRC32 of header
    header_crc32: u32,
    /// Reserved
    reserved: u32,
    /// Current LBA (location of this header)
    current_lba: u64,
    /// Backup LBA (location of backup header)
    backup_lba: u64,
    /// First usable LBA
    first_usable_lba: u64,
    /// Last usable LBA
    last_usable_lba: u64,
    /// Disk GUID
    disk_guid: [u8; 16],
    /// Starting LBA of partition entries
    partition_entry_lba: u64,
    /// Number of partition entries
    num_partition_entries: u32,
    /// Size of each partition entry (usually 128)
    partition_entry_size: u32,
    /// CRC32 of partition entries
    partition_entries_crc32: u32,
}

/// GPT Partition Entry (128 bytes)
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GptPartitionEntry {
    /// Partition type GUID
    type_guid: [u8; 16],
    /// Unique partition GUID
    partition_guid: [u8; 16],
    /// Starting LBA
    start_lba: u64,
    /// Ending LBA (inclusive)
    end_lba: u64,
    /// Attributes
    attributes: u64,
    /// Partition name (UTF-16LE, 36 characters)
    name: [u16; 36],
}

// ============================================================================
// Partition Table Types
// ============================================================================

/// Detected partition table type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionTableType {
    /// No partition table found
    None,
    /// Master Boot Record (legacy)
    Mbr,
    /// GUID Partition Table (modern)
    Gpt,
}

/// Parsed partition table
#[derive(Debug, Clone)]
pub struct PartitionTable {
    /// Table type
    pub table_type: PartitionTableType,
    /// List of partitions
    pub partitions: Vec<Partition>,
    /// Disk GUID (GPT only)
    pub disk_guid: Option<[u8; 16]>,
    /// Total disk size in sectors (if known)
    pub total_sectors: u64,
}

impl PartitionTable {
    /// Create empty partition table
    pub fn empty() -> Self {
        Self {
            table_type: PartitionTableType::None,
            partitions: Vec::new(),
            disk_guid: None,
            total_sectors: 0,
        }
    }
}

// ============================================================================
// Parsing Functions
// ============================================================================

/// Parse partition table from disk
/// 
/// Reads sector 0 (MBR) and optionally sector 1 (GPT header)
pub fn parse_partition_table<F>(read_sector: F, total_sectors: u64) -> Result<PartitionTable, &'static str>
where
    F: Fn(u64, &mut [u8]) -> Result<(), &'static str>,
{
    let mut sector0 = [0u8; SECTOR_SIZE];
    read_sector(0, &mut sector0)?;
    
    // Check MBR signature
    let signature = u16::from_le_bytes([sector0[510], sector0[511]]);
    if signature != MBR_SIGNATURE {
        return Ok(PartitionTable::empty());
    }
    
    // Parse MBR
    let mbr = unsafe { &*(sector0.as_ptr() as *const Mbr) };
    
    // Check if this is a protective MBR (indicates GPT)
    let has_protective = mbr.partitions.iter()
        .any(|p| p.partition_type == MBR_TYPE_GPT_PROTECTIVE);
    
    if has_protective {
        // Try to parse GPT
        match parse_gpt(&read_sector, total_sectors) {
            Ok(table) => return Ok(table),
            Err(_) => {
                // Fall back to MBR if GPT parsing fails
            }
        }
    }
    
    // Parse as MBR
    parse_mbr(mbr, total_sectors)
}

/// Parse MBR partition table
fn parse_mbr(mbr: &Mbr, total_sectors: u64) -> Result<PartitionTable, &'static str> {
    let mut partitions = Vec::new();
    
    for (i, entry) in mbr.partitions.iter().enumerate() {
        if entry.partition_type == 0 || entry.size_sectors == 0 {
            continue;
        }
        
        let partition = Partition {
            number: (i + 1) as u8,
            start_lba: entry.start_lba as u64,
            size_sectors: entry.size_sectors as u64,
            partition_type: PartitionType::from_mbr(entry.partition_type),
            bootable: entry.boot_flag == 0x80,
            name: String::new(),
            guid: None,
        };
        
        partitions.push(partition);
    }
    
    Ok(PartitionTable {
        table_type: PartitionTableType::Mbr,
        partitions,
        disk_guid: None,
        total_sectors,
    })
}

/// Parse GPT partition table
fn parse_gpt<F>(read_sector: &F, total_sectors: u64) -> Result<PartitionTable, &'static str>
where
    F: Fn(u64, &mut [u8]) -> Result<(), &'static str>,
{
    let mut sector1 = [0u8; SECTOR_SIZE];
    read_sector(1, &mut sector1)?;
    
    let header = unsafe { &*(sector1.as_ptr() as *const GptHeader) };
    
    // Verify GPT signature
    if header.signature != GPT_SIGNATURE {
        return Err("Invalid GPT signature");
    }
    
    let mut partitions = Vec::new();
    let entry_size = { header.partition_entry_size };
    let num_entries = { header.num_partition_entries };
    let entries_lba = { header.partition_entry_lba };
    let disk_guid_copy = { header.disk_guid };
    
    let entries_per_sector = SECTOR_SIZE / entry_size as usize;
    let sectors_needed = (num_entries as usize + entries_per_sector - 1) / entries_per_sector;
    
    let mut partition_number = 1u8;
    
    for sector_offset in 0..sectors_needed {
        let mut sector = [0u8; SECTOR_SIZE];
        read_sector(entries_lba + sector_offset as u64, &mut sector)?;
        
        for entry_idx in 0..entries_per_sector {
            let offset = entry_idx * entry_size as usize;
            if offset + 128 > SECTOR_SIZE {
                break;
            }
            
            let entry = unsafe { 
                &*(sector.as_ptr().add(offset) as *const GptPartitionEntry) 
            };
            
            // Copy packed fields to local variables
            let type_guid = { entry.type_guid };
            let partition_guid = { entry.partition_guid };
            let start_lba = { entry.start_lba };
            let end_lba = { entry.end_lba };
            let attributes = { entry.attributes };
            let entry_name = { entry.name };
            
            // Skip empty entries
            if type_guid == [0u8; 16] {
                continue;
            }
            
            // Parse partition name (UTF-16LE to ASCII)
            let mut name = String::new();
            for &c in &entry_name {
                if c == 0 {
                    break;
                }
                if c < 128 {
                    name.push(c as u8 as char);
                }
            }
            
            let partition = Partition {
                number: partition_number,
                start_lba,
                size_sectors: end_lba - start_lba + 1,
                partition_type: PartitionType::from_gpt_guid(&type_guid),
                bootable: (attributes & 0x04) != 0, // Legacy BIOS bootable
                name,
                guid: Some(partition_guid),
            };
            
            partitions.push(partition);
            partition_number += 1;
        }
    }
    
    Ok(PartitionTable {
        table_type: PartitionTableType::Gpt,
        partitions,
        disk_guid: Some(disk_guid_copy),
        total_sectors,
    })
}

/// Format GUID as string
pub fn format_guid(guid: &[u8; 16]) -> String {
    // GUID format: XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX
    // But stored in mixed-endian: first 3 groups are little-endian
    format!(
        "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        guid[3], guid[2], guid[1], guid[0],
        guid[5], guid[4],
        guid[7], guid[6],
        guid[8], guid[9],
        guid[10], guid[11], guid[12], guid[13], guid[14], guid[15]
    )
}

/// Print partition table to console
pub fn print_partition_table(table: &PartitionTable) {
    match table.table_type {
        PartitionTableType::None => {
            crate::println!("No partition table found");
            return;
        }
        PartitionTableType::Mbr => {
            crate::println!("Partition table: MBR");
        }
        PartitionTableType::Gpt => {
            crate::println!("Partition table: GPT");
            if let Some(ref guid) = table.disk_guid {
                crate::println!("Disk GUID: {}", format_guid(guid));
            }
        }
    }
    
    if table.partitions.is_empty() {
        crate::println!("No partitions found");
        return;
    }
    
    crate::println!();
    crate::println!("  #  Boot  Start LBA     End LBA       Size       Type");
    crate::println!("  ─────────────────────────────────────────────────────────");
    
    for p in &table.partitions {
        let boot_flag = if p.bootable { "*" } else { " " };
        crate::println!(
            "  {}  {}     {:>12}  {:>12}  {:>10}   {}",
            p.number,
            boot_flag,
            p.start_lba,
            p.end_lba(),
            p.size_human(),
            p.partition_type.name()
        );
        
        if !p.name.is_empty() {
            crate::println!("                                              Name: {}", p.name);
        }
    }
}

// ============================================================================
// High-level API for AHCI
// ============================================================================

/// Read partition table from AHCI port
pub fn read_from_ahci(port: u8) -> Result<PartitionTable, &'static str> {
    use super::ahci;
    
    if !ahci::is_initialized() {
        return Err("AHCI not initialized");
    }
    
    // Get disk info for total sectors
    let disk_info = ahci::get_port_info(port).ok_or("Port not found")?;
    let total_sectors = disk_info.sector_count;
    
    // Create read function
    let read_fn = |lba: u64, buffer: &mut [u8]| -> Result<(), &'static str> {
        if buffer.len() < SECTOR_SIZE {
            return Err("Buffer too small");
        }
        let mut sector_buf = [0u8; SECTOR_SIZE];
        ahci::read_sectors(port, lba, 1, &mut sector_buf)?;
        buffer[..SECTOR_SIZE].copy_from_slice(&sector_buf);
        Ok(())
    };
    
    parse_partition_table(read_fn, total_sectors)
}
