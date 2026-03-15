//! FAT32 Filesystem Driver
//!
//! Implements read support for FAT32 filesystems.
//! FAT32 is widely used on USB drives, SD cards, and Windows partitions.
//!
//! Structure:
//! - Boot sector (sector 0) with BPB
//! - Reserved sectors
//! - FAT tables (1 or 2 copies)
//! - Data region (clusters)

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use spin::RwLock;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{
    FileSystem, FileOps, DirOps, Stat, DirEntry, FileType,
    Ino, VfsResult, VfsError
};

const SECTOR_SIZE: usize = 512;

/// FAT32 Boot Sector / BPB (BIOS Parameter Block)
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Fat32BootSector {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    num_fats: u8,
    root_entry_count: u16,  // 0 for FAT32
    total_sectors_16: u16,  // 0 for FAT32
    media_type: u8,
    fat_size_16: u16,       // 0 for FAT32
    sectors_per_track: u16,
    num_heads: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
    // FAT32 specific
    fat_size_32: u32,
    ext_flags: u16,
    fs_version: u16,
    root_cluster: u32,
    fs_info: u16,
    backup_boot: u16,
    reserved: [u8; 12],
    drive_number: u8,
    reserved1: u8,
    boot_sig: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_type: [u8; 8],
}

impl Fat32BootSector {
    fn is_valid(&self) -> bool {
        // Check for FAT32 signature - use addr_of! for packed struct fields
        let bytes_per_sector = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.bytes_per_sector)) };
        let sectors_per_cluster = self.sectors_per_cluster;
        let reserved = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.reserved_sectors)) };
        let num_fats = self.num_fats;
        
        bytes_per_sector >= 512 && 
        bytes_per_sector <= 4096 &&
        sectors_per_cluster >= 1 &&
        sectors_per_cluster <= 128 &&
        reserved >= 1 &&
        num_fats >= 1
    }
    
    fn cluster_size(&self) -> usize {
        let bps = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.bytes_per_sector)) } as usize;
        let spc = self.sectors_per_cluster as usize;
        bps * spc
    }
    
    fn first_data_sector(&self) -> u64 {
        let reserved = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.reserved_sectors)) } as u64;
        let num_fats = self.num_fats as u64;
        let fat_size = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.fat_size_32)) } as u64;
        reserved + (num_fats * fat_size)
    }
    
    fn first_fat_sector(&self) -> u64 {
        (unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.reserved_sectors)) }) as u64
    }
    
    fn cluster_to_sector(&self, cluster: u32) -> u64 {
        let spc = self.sectors_per_cluster as u64;
        self.first_data_sector() + ((cluster - 2) as u64 * spc)
    }
}

/// FAT32 Directory Entry (32 bytes)
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Fat32DirEntry {
    name: [u8; 8],
    ext: [u8; 3],
    attr: u8,
    nt_reserved: u8,
    create_time_tenth: u8,
    create_time: u16,
    create_date: u16,
    access_date: u16,
    cluster_hi: u16,
    modify_time: u16,
    modify_date: u16,
    cluster_lo: u16,
    file_size: u32,
}

impl Fat32DirEntry {
    const ATTR_READ_ONLY: u8 = 0x01;
    const ATTR_HIDDEN: u8 = 0x02;
    const ATTR_SYSTEM: u8 = 0x04;
    const ATTR_VOLUME_ID: u8 = 0x08;
    const ATTR_DIRECTORY: u8 = 0x10;
    const ATTR_ARCHIVE: u8 = 0x20;
    const ATTR_LONG_NAME: u8 = 0x0F;
    
    fn is_free(&self) -> bool {
        self.name[0] == 0x00 || self.name[0] == 0xE5
    }
    
    fn is_end(&self) -> bool {
        self.name[0] == 0x00
    }
    
    fn is_long_name(&self) -> bool {
        (self.attr & Self::ATTR_LONG_NAME) == Self::ATTR_LONG_NAME
    }
    
    fn is_directory(&self) -> bool {
        (self.attr & Self::ATTR_DIRECTORY) != 0
    }
    
    fn is_volume_label(&self) -> bool {
        (self.attr & Self::ATTR_VOLUME_ID) != 0 && !self.is_long_name()
    }
    
    fn cluster(&self) -> u32 {
        let hi = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.cluster_hi)) } as u32;
        let lo = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.cluster_lo)) } as u32;
        (hi << 16) | lo
    }
    
    fn size(&self) -> u32 {
        unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.file_size)) }
    }
    
    fn get_short_name(&self) -> String {
        let name_bytes = self.name;
        let ext_bytes = self.ext;
        
        // Convert 8.3 name to string
        let name: String = name_bytes.iter()
            .take_while(|&&c| c != b' ' && c != 0)
            .map(|&c| {
                if c == 0x05 { 0xE5 as char } // Special case for deleted marker
                else { c as char }
            })
            .collect();
        
        let ext: String = ext_bytes.iter()
            .take_while(|&&c| c != b' ' && c != 0)
            .map(|&c| c as char)
            .collect();
        
        if ext.is_empty() {
            name
        } else {
            alloc::format!("{}.{}", name, ext)
        }
    }
}

/// Long File Name Entry (LFN)
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Fat32LfnEntry {
    order: u8,
    name1: [u16; 5],  // First 5 UCS-2 chars
    attr: u8,         // Always 0x0F
    lfn_type: u8,     // Always 0
    checksum: u8,
    name2: [u16; 6],  // Next 6 UCS-2 chars
    cluster: u16,     // Always 0
    name3: [u16; 2],  // Last 2 UCS-2 chars
}

impl Fat32LfnEntry {
    fn get_chars(&self) -> Vec<char> {
        let mut chars = Vec::with_capacity(13);
        
        // Copy packed fields using read_unaligned
        let name1 = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.name1)) };
        let name2 = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.name2)) };
        let name3 = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.name3)) };
        
        for &c in &name1 {
            if c == 0 || c == 0xFFFF { return chars; }
            chars.push(char::from_u32(c as u32).unwrap_or('?'));
        }
        for &c in &name2 {
            if c == 0 || c == 0xFFFF { return chars; }
            chars.push(char::from_u32(c as u32).unwrap_or('?'));
        }
        for &c in &name3 {
            if c == 0 || c == 0xFFFF { return chars; }
            chars.push(char::from_u32(c as u32).unwrap_or('?'));
        }
        
        chars
    }
    
    fn order(&self) -> u8 {
        self.order & 0x3F
    }
    
    fn is_last(&self) -> bool {
        (self.order & 0x40) != 0
    }
}

/// FAT entry values
const FAT_FREE: u32 = 0x00000000;
const FAT_EOC_MIN: u32 = 0x0FFFFFF8;  // End of cluster chain
const FAT_BAD: u32 = 0x0FFFFFF7;

/// Cached inode info
struct Fat32Inode {
    cluster: u32,
    size: u64,
    is_dir: bool,
    name: String,
    /// Directory cluster containing this entry (0 for root)
    dir_cluster: u32,
    /// Byte offset within the directory cluster data
    dir_entry_offset: usize,
}

/// Block device reader/writer trait
pub trait BlockDevice: Send + Sync {
    fn read_sector(&self, sector: u64, buffer: &mut [u8]) -> Result<(), ()>;
    fn write_sector(&self, sector: u64, buffer: &[u8]) -> Result<(), ()>;
    fn sector_size(&self) -> usize { SECTOR_SIZE }
}

/// Legacy alias for compatibility
pub trait BlockReader: BlockDevice {}
impl<T: BlockDevice> BlockReader for T {}

/// VirtIO Block Device (wraps the virtio_blk module)
pub struct VirtioBlockDevice;

impl BlockDevice for VirtioBlockDevice {
    fn read_sector(&self, sector: u64, buffer: &mut [u8]) -> Result<(), ()> {
        // virtio_blk::read_sectors takes a generic &mut [u8]
        crate::virtio_blk::read_sectors(sector, 1, buffer)
            .map_err(|_| ())
    }
    
    fn write_sector(&self, sector: u64, buffer: &[u8]) -> Result<(), ()> {
        crate::virtio_blk::write_sectors(sector, 1, buffer)
            .map_err(|_| ())
    }
}

/// AHCI Block Device
pub struct AhciBlockReader {
    port: usize,
    partition_start: u64,  // Partition start LBA
}

impl AhciBlockReader {
    pub fn new(port: usize, partition_start: u64) -> Self {
        Self { port, partition_start }
    }
}

impl BlockDevice for AhciBlockReader {
    fn read_sector(&self, sector: u64, buffer: &mut [u8]) -> Result<(), ()> {
        let absolute_sector = self.partition_start + sector;
        crate::drivers::ahci::read_sectors(self.port as u8, absolute_sector, 1, buffer)
            .map(|_| ())
            .map_err(|_| ())
    }
    
    fn write_sector(&self, sector: u64, buffer: &[u8]) -> Result<(), ()> {
        let absolute_sector = self.partition_start + sector;
        crate::drivers::ahci::write_sectors(self.port as u8, absolute_sector, 1, buffer)
            .map(|_| ())
            .map_err(|_| ())
    }
}

/// FAT32 Filesystem
pub struct Fat32Fs {
    reader: Arc<dyn BlockDevice>,
    bpb: Fat32BootSector,
    inodes: RwLock<BTreeMap<Ino, Fat32Inode>>,
    next_ino: AtomicU64,
    root_ino: Ino,
}

impl Fat32Fs {
    /// Mount a FAT32 filesystem from a block device
    pub fn mount(reader: Arc<dyn BlockDevice>) -> VfsResult<Self> {
        // Read boot sector
        let mut boot_buf = [0u8; SECTOR_SIZE];
        reader.read_sector(0, &mut boot_buf)
            .map_err(|_| VfsError::IoError)?;
        
        // Check boot signature
        if boot_buf[510] != 0x55 || boot_buf[511] != 0xAA {
            crate::log_warn!("[FAT32] Invalid boot signature");
            return Err(VfsError::InvalidPath);
        }
        
        // Parse BPB
        let bpb = unsafe { 
            core::ptr::read_unaligned(boot_buf.as_ptr() as *const Fat32BootSector)
        };
        
        if !bpb.is_valid() {
            crate::log_warn!("[FAT32] Invalid BPB");
            return Err(VfsError::InvalidPath);
        }
        
        let root_cluster = { bpb.root_cluster };
        let bytes_per_sector = { bpb.bytes_per_sector };
        let sectors_per_cluster = { bpb.sectors_per_cluster };
        
        crate::log!("[FAT32] Mounted: {} bytes/sector, {} sectors/cluster, root cluster {}",
            bytes_per_sector, sectors_per_cluster, root_cluster);
        
        let fs = Self {
            reader,
            bpb,
            inodes: RwLock::new(BTreeMap::new()),
            next_ino: AtomicU64::new(2),
            root_ino: 1,
        };
        
        // Create root inode
        {
            let mut inodes = fs.inodes.write();
            inodes.insert(1, Fat32Inode {
                cluster: root_cluster,
                size: 0,
                is_dir: true,
                name: String::from("/"),
                dir_cluster: 0,
                dir_entry_offset: 0,
            });
        }
        
        Ok(fs)
    }
    
    /// Read a cluster
    fn read_cluster(&self, cluster: u32) -> VfsResult<Vec<u8>> {
        let sector = self.bpb.cluster_to_sector(cluster);
        let sectors_per_cluster = { self.bpb.sectors_per_cluster } as u64;
        let cluster_size = self.bpb.cluster_size();
        
        let mut data = vec![0u8; cluster_size];
        
        for i in 0..sectors_per_cluster {
            let offset = (i as usize) * SECTOR_SIZE;
            self.reader.read_sector(sector + i, &mut data[offset..offset + SECTOR_SIZE])
                .map_err(|_| VfsError::IoError)?;
        }
        
        Ok(data)
    }
    
    /// Read FAT entry for a cluster
    fn read_fat_entry(&self, cluster: u32) -> VfsResult<u32> {
        let fat_offset = cluster * 4;
        let bytes_per_sector = { self.bpb.bytes_per_sector } as u32;
        let fat_sector = self.bpb.first_fat_sector() + (fat_offset / bytes_per_sector) as u64;
        let offset_in_sector = (fat_offset % bytes_per_sector) as usize;
        
        let mut sector_buf = [0u8; SECTOR_SIZE];
        self.reader.read_sector(fat_sector, &mut sector_buf)
            .map_err(|_| VfsError::IoError)?;
        
        let entry = u32::from_le_bytes([
            sector_buf[offset_in_sector],
            sector_buf[offset_in_sector + 1],
            sector_buf[offset_in_sector + 2],
            sector_buf[offset_in_sector + 3],
        ]) & 0x0FFFFFFF;  // FAT32 uses 28 bits
        
        Ok(entry)
    }
    
    /// Get cluster chain for a file/directory
    fn get_cluster_chain(&self, start_cluster: u32) -> VfsResult<Vec<u32>> {
        let mut chain = Vec::new();
        let mut current = start_cluster;
        
        while current >= 2 && current < FAT_BAD {
            chain.push(current);
            current = self.read_fat_entry(current)?;
            
            // Safety limit
            if chain.len() > 1_000_000 {
                return Err(VfsError::IoError);
            }
        }
        
        Ok(chain)
    }
    
    /// Read all data from a cluster chain
    fn read_chain(&self, start_cluster: u32, size: Option<u64>) -> VfsResult<Vec<u8>> {
        let chain = self.get_cluster_chain(start_cluster)?;
        let cluster_size = self.bpb.cluster_size();
        let total_size = size.unwrap_or((chain.len() * cluster_size) as u64) as usize;
        
        let mut data = Vec::with_capacity(total_size);
        
        for cluster in chain {
            let cluster_data = self.read_cluster(cluster)?;
            data.extend_from_slice(&cluster_data);
            if data.len() >= total_size {
                break;
            }
        }
        
        data.truncate(total_size);
        Ok(data)
    }
    
    /// Write a cluster
    fn write_cluster(&self, cluster: u32, data: &[u8]) -> VfsResult<()> {
        let sector = self.bpb.cluster_to_sector(cluster);
        let sectors_per_cluster = { self.bpb.sectors_per_cluster } as u64;
        let cluster_size = self.bpb.cluster_size();
        
        if data.len() < cluster_size {
            return Err(VfsError::IoError);
        }
        
        for i in 0..sectors_per_cluster {
            let offset = (i as usize) * SECTOR_SIZE;
            self.reader.write_sector(sector + i, &data[offset..offset + SECTOR_SIZE])
                .map_err(|_| VfsError::IoError)?;
        }
        
        Ok(())
    }
    
    /// Write FAT entry for a cluster
    fn write_fat_entry(&self, cluster: u32, value: u32) -> VfsResult<()> {
        let fat_offset = cluster * 4;
        let bytes_per_sector = { self.bpb.bytes_per_sector } as u32;
        let fat_sector = self.bpb.first_fat_sector() + (fat_offset / bytes_per_sector) as u64;
        let offset_in_sector = (fat_offset % bytes_per_sector) as usize;
        
        // Read the sector
        let mut sector_buf = [0u8; SECTOR_SIZE];
        self.reader.read_sector(fat_sector, &mut sector_buf)
            .map_err(|_| VfsError::IoError)?;
        
        // Update the entry (preserve high 4 bits for FAT32)
        let new_value = (value & 0x0FFFFFFF) | 
                        (u32::from_le_bytes([sector_buf[offset_in_sector], 
                                            sector_buf[offset_in_sector + 1],
                                            sector_buf[offset_in_sector + 2], 
                                            sector_buf[offset_in_sector + 3]]) & 0xF0000000);
        
        sector_buf[offset_in_sector..offset_in_sector + 4]
            .copy_from_slice(&new_value.to_le_bytes());
        
        // Write back to primary FAT
        self.reader.write_sector(fat_sector, &sector_buf)
            .map_err(|_| VfsError::IoError)?;
        
        // Write to backup FAT if present
        let num_fats = { self.bpb.num_fats } as u64;
        if num_fats > 1 {
            let fat_size = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.bpb.fat_size_32)) } as u64;
            for fat_idx in 1..num_fats {
                let backup_sector = fat_sector + fat_idx * fat_size;
                let _ = self.reader.write_sector(backup_sector, &sector_buf);
            }
        }
        
        Ok(())
    }
    
    /// Allocate a new cluster
    fn allocate_cluster(&self) -> VfsResult<u32> {
        let bytes_per_sector = { self.bpb.bytes_per_sector } as u32;
        let fat_start = self.bpb.first_fat_sector();
        let fat_size = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.bpb.fat_size_32)) };
        let total_sectors_32 = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(self.bpb.total_sectors_32)) };
        let data_sectors = total_sectors_32 as u64 - self.bpb.first_data_sector();
        let spc = self.bpb.sectors_per_cluster as u64;
        let total_clusters = (data_sectors / spc) as u32 + 2;
        
        // Scan FAT for free cluster
        let entries_per_sector = bytes_per_sector / 4;
        let mut sector_buf = [0u8; SECTOR_SIZE];
        
        for sector_offset in 0..fat_size {
            self.reader.read_sector(fat_start + sector_offset as u64, &mut sector_buf)
                .map_err(|_| VfsError::IoError)?;
            
            for entry_idx in 0..entries_per_sector {
                let cluster = sector_offset * entries_per_sector + entry_idx;
                if cluster < 2 || cluster >= total_clusters {
                    continue;
                }
                
                let offset = (entry_idx * 4) as usize;
                let value = u32::from_le_bytes([
                    sector_buf[offset],
                    sector_buf[offset + 1],
                    sector_buf[offset + 2],
                    sector_buf[offset + 3],
                ]) & 0x0FFFFFFF;
                
                if value == FAT_FREE {
                    // Found free cluster - mark as end of chain
                    self.write_fat_entry(cluster, FAT_EOC_MIN)?;
                    return Ok(cluster);
                }
            }
        }
        
        Err(VfsError::NoSpace)
    }
    
    /// Extend cluster chain by adding a new cluster
    fn extend_chain(&self, last_cluster: u32) -> VfsResult<u32> {
        let new_cluster = self.allocate_cluster()?;
        self.write_fat_entry(last_cluster, new_cluster)?;
        Ok(new_cluster)
    }
    
    /// Write data to file starting at offset, extending if needed
    fn write_file_data(&self, start_cluster: u32, offset: u64, data: &[u8], current_size: u64) -> VfsResult<(u32, u64)> {
        let cluster_size = self.bpb.cluster_size();
        let mut chain = self.get_cluster_chain(start_cluster)?;
        
        // Calculate how many clusters we need
        let end_offset = offset + data.len() as u64;
        let clusters_needed = ((end_offset + cluster_size as u64 - 1) / cluster_size as u64) as usize;
        
        // Extend chain if needed
        while chain.len() < clusters_needed {
            let last = *chain.last().unwrap_or(&start_cluster);
            let new_cluster = if chain.is_empty() {
                self.allocate_cluster()?
            } else {
                self.extend_chain(last)?
            };
            chain.push(new_cluster);
        }
        
        // Write data to clusters
        let mut remaining = data;
        let mut write_offset = offset as usize;
        
        for &cluster in &chain {
            let cluster_start = (chain.iter().position(|&c| c == cluster).unwrap()) * cluster_size;
            let cluster_end = cluster_start + cluster_size;
            
            if write_offset >= cluster_end || remaining.is_empty() {
                continue;
            }
            
            if write_offset < cluster_start {
                write_offset = cluster_start;
            }
            
            // Read existing cluster data
            let mut cluster_data = self.read_cluster(cluster)?;
            
            // Calculate how much to write to this cluster
            let offset_in_cluster = write_offset - cluster_start;
            let space_in_cluster = cluster_size - offset_in_cluster;
            let to_write = core::cmp::min(space_in_cluster, remaining.len());
            
            // Copy data
            cluster_data[offset_in_cluster..offset_in_cluster + to_write]
                .copy_from_slice(&remaining[..to_write]);
            
            // Write cluster back
            self.write_cluster(cluster, &cluster_data)?;
            
            remaining = &remaining[to_write..];
            write_offset += to_write;
        }
        
        let new_size = core::cmp::max(current_size, end_offset);
        let new_start = *chain.first().unwrap_or(&start_cluster);
        
        Ok((new_start, new_size))
    }
    
    /// Generate 8.3 short name from long name
    fn generate_short_name(long_name: &str) -> [u8; 11] {
        let mut short = [b' '; 11];
        let upper = long_name.to_uppercase();
        
        // Split name and extension
        let (name_part, ext_part) = if let Some(dot_pos) = upper.rfind('.') {
            (&upper[..dot_pos], &upper[dot_pos + 1..])
        } else {
            (upper.as_str(), "")
        };
        
        // Copy name (max 8 chars)
        for (i, ch) in name_part.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').take(8).enumerate() {
            short[i] = ch as u8;
        }
        
        // Copy extension (max 3 chars)
        for (i, ch) in ext_part.chars().filter(|c| c.is_ascii_alphanumeric()).take(3).enumerate() {
            short[8 + i] = ch as u8;
        }
        
        short
    }
    
    /// Find free directory entry slot in directory cluster chain
    fn find_free_dir_entry(&self, dir_cluster: u32) -> VfsResult<(u32, usize)> {
        let cluster_size = self.bpb.cluster_size();
        let entry_size = core::mem::size_of::<Fat32DirEntry>();
        let entries_per_cluster = cluster_size / entry_size;
        
        let chain = self.get_cluster_chain(dir_cluster)?;
        
        for &cluster in &chain {
            let data = self.read_cluster(cluster)?;
            
            for i in 0..entries_per_cluster {
                let offset = i * entry_size;
                let first_byte = data[offset];
                
                if first_byte == 0x00 || first_byte == 0xE5 {
                    return Ok((cluster, offset));
                }
            }
        }
        
        // Need to extend directory
        let last = *chain.last().unwrap_or(&dir_cluster);
        let new_cluster = self.extend_chain(last)?;
        
        // Zero out new cluster
        let zero_data = vec![0u8; cluster_size];
        self.write_cluster(new_cluster, &zero_data)?;
        
        Ok((new_cluster, 0))
    }
    
    /// Create a new file or directory entry
    fn create_entry(&self, dir_cluster: u32, name: &str, is_dir: bool) -> VfsResult<(u32, u64)> {
        let (entry_cluster, entry_offset) = self.find_free_dir_entry(dir_cluster)?;
        
        // Allocate cluster for new file/directory
        let new_cluster = if is_dir {
            let cluster = self.allocate_cluster()?;
            // Initialize directory with . and ..
            let cluster_size = self.bpb.cluster_size();
            let mut dir_data = vec![0u8; cluster_size];
            
            // Create . entry
            let dot_entry = Fat32DirEntry {
                name: [b'.', b' ', b' ', b' ', b' ', b' ', b' ', b' '],
                ext: [b' ', b' ', b' '],
                attr: Fat32DirEntry::ATTR_DIRECTORY,
                nt_reserved: 0,
                create_time_tenth: 0,
                create_time: 0,
                create_date: 0,
                access_date: 0,
                cluster_hi: ((cluster >> 16) & 0xFFFF) as u16,
                modify_time: 0,
                modify_date: 0,
                cluster_lo: (cluster & 0xFFFF) as u16,
                file_size: 0,
            };
            
            // Create .. entry
            let dotdot_entry = Fat32DirEntry {
                name: [b'.', b'.', b' ', b' ', b' ', b' ', b' ', b' '],
                ext: [b' ', b' ', b' '],
                attr: Fat32DirEntry::ATTR_DIRECTORY,
                nt_reserved: 0,
                create_time_tenth: 0,
                create_time: 0,
                create_date: 0,
                access_date: 0,
                cluster_hi: ((dir_cluster >> 16) & 0xFFFF) as u16,
                modify_time: 0,
                modify_date: 0,
                cluster_lo: (dir_cluster & 0xFFFF) as u16,
                file_size: 0,
            };
            
            let entry_size = core::mem::size_of::<Fat32DirEntry>();
            unsafe {
                core::ptr::write(dir_data.as_mut_ptr() as *mut Fat32DirEntry, dot_entry);
                core::ptr::write((dir_data.as_mut_ptr().add(entry_size)) as *mut Fat32DirEntry, dotdot_entry);
            }
            
            self.write_cluster(cluster, &dir_data)?;
            cluster
        } else {
            0 // Files start with no cluster until data is written
        };
        
        // Create directory entry
        let short_name = Self::generate_short_name(name);
        let new_entry = Fat32DirEntry {
            name: short_name[..8].try_into().unwrap_or([b' '; 8]),
            ext: short_name[8..11].try_into().unwrap_or([b' '; 3]),
            attr: if is_dir { Fat32DirEntry::ATTR_DIRECTORY } else { Fat32DirEntry::ATTR_ARCHIVE },
            nt_reserved: 0,
            create_time_tenth: 0,
            create_time: 0,
            create_date: 0,
            access_date: 0,
            cluster_hi: ((new_cluster >> 16) & 0xFFFF) as u16,
            modify_time: 0,
            modify_date: 0,
            cluster_lo: (new_cluster & 0xFFFF) as u16,
            file_size: 0,
        };
        
        // Write entry to directory
        let mut cluster_data = self.read_cluster(entry_cluster)?;
        unsafe {
            core::ptr::write(
                cluster_data.as_mut_ptr().add(entry_offset) as *mut Fat32DirEntry,
                new_entry
            );
        }
        self.write_cluster(entry_cluster, &cluster_data)?;
        
        Ok((new_cluster, 0))
    }
    
    /// Update directory entry (size, cluster, etc.)
    fn update_dir_entry(&self, ino: Ino, new_cluster: u32, new_size: u64) -> VfsResult<()> {
        let inodes = self.inodes.read();
        let inode = inodes.get(&ino).ok_or(VfsError::NotFound)?;
        let dir_cluster = inode.dir_cluster;
        let dir_offset = inode.dir_entry_offset;
        drop(inodes);

        if dir_cluster == 0 {
            // Root inode or unknown location — skip
            return Ok(());
        }

        let entry_size = core::mem::size_of::<Fat32DirEntry>();
        let cluster_size = self.bpb.cluster_size();
        let entries_per_cluster = cluster_size / entry_size;

        // Walk the directory cluster chain to find the right cluster
        let chain = self.get_cluster_chain(dir_cluster)?;
        let cluster_index = dir_offset / cluster_size;
        let offset_in_cluster = dir_offset % cluster_size;

        if cluster_index >= chain.len() {
            return Err(VfsError::IoError);
        }

        let target_cluster = chain[cluster_index];
        let mut data = self.read_cluster(target_cluster)?;

        // Read existing entry
        let entry = unsafe {
            &mut *(data.as_mut_ptr().add(offset_in_cluster) as *mut Fat32DirEntry)
        };

        // Update cluster pointers
        entry.cluster_lo = (new_cluster & 0xFFFF) as u16;
        entry.cluster_hi = ((new_cluster >> 16) & 0xFFFF) as u16;

        // Update file size (only for regular files, not directories)
        if entry.attr & Fat32DirEntry::ATTR_DIRECTORY == 0 {
            entry.file_size = new_size as u32;
        }

        // Write cluster back
        self.write_cluster(target_cluster, &data)?;

        Ok(())
    }
    
    /// Delete a directory entry
    fn delete_entry(&self, dir_cluster: u32, name: &str) -> VfsResult<()> {
        let cluster_size = self.bpb.cluster_size();
        let entry_size = core::mem::size_of::<Fat32DirEntry>();
        let entries_per_cluster = cluster_size / entry_size;
        
        let chain = self.get_cluster_chain(dir_cluster)?;
        
        for &cluster in &chain {
            let mut data = self.read_cluster(cluster)?;
            
            for i in 0..entries_per_cluster {
                let offset = i * entry_size;
                let entry = unsafe {
                    core::ptr::read_unaligned(data[offset..].as_ptr() as *const Fat32DirEntry)
                };
                
                if entry.is_end() {
                    return Err(VfsError::NotFound);
                }
                
                if !entry.is_free() && !entry.is_long_name() && !entry.is_volume_label() {
                    let entry_name = entry.get_short_name();
                    if entry_name.eq_ignore_ascii_case(name) {
                        // Mark entry as deleted
                        data[offset] = 0xE5;
                        self.write_cluster(cluster, &data)?;
                        
                        // Free the cluster chain
                        let file_cluster = entry.cluster();
                        if file_cluster >= 2 {
                            self.free_cluster_chain(file_cluster)?;
                        }
                        
                        return Ok(());
                    }
                }
            }
        }
        
        Err(VfsError::NotFound)
    }
    
    /// Free a cluster chain
    fn free_cluster_chain(&self, start_cluster: u32) -> VfsResult<()> {
        let chain = self.get_cluster_chain(start_cluster)?;
        
        for cluster in chain {
            self.write_fat_entry(cluster, FAT_FREE)?;
        }
        
        Ok(())
    }
    
    /// Parse directory entries, returning (name, entry, byte_offset_in_chain)
    fn parse_directory(&self, cluster: u32) -> VfsResult<Vec<(String, Fat32DirEntry, usize)>> {
        let data = self.read_chain(cluster, None)?;
        let entry_size = core::mem::size_of::<Fat32DirEntry>();
        let mut entries = Vec::new();
        let mut lfn_parts: Vec<(u8, Vec<char>)> = Vec::new();
        
        let mut i = 0;
        while i + entry_size <= data.len() {
            let entry = unsafe {
                core::ptr::read_unaligned(data[i..].as_ptr() as *const Fat32DirEntry)
            };
            
            if entry.is_end() {
                break;
            }
            
            if entry.is_free() {
                lfn_parts.clear();
                i += entry_size;
                continue;
            }
            
            if entry.is_long_name() {
                // Parse LFN entry
                let lfn = unsafe {
                    core::ptr::read_unaligned(data[i..].as_ptr() as *const Fat32LfnEntry)
                };
                lfn_parts.push((lfn.order(), lfn.get_chars()));
            } else if !entry.is_volume_label() {
                // Regular entry - combine with LFN if available
                let name = if !lfn_parts.is_empty() {
                    // Sort LFN parts by order and combine
                    lfn_parts.sort_by_key(|(order, _)| *order);
                    let full_name: String = lfn_parts.iter()
                        .flat_map(|(_, chars)| chars.iter())
                        .collect();
                    lfn_parts.clear();
                    full_name
                } else {
                    entry.get_short_name()
                };
                
                // Skip . and ..
                if name != "." && name != ".." {
                    entries.push((name, entry, i));
                }
            }
            
            i += entry_size;
        }
        
        Ok(entries)
    }
    
    fn get_inode(&self, ino: Ino) -> VfsResult<Fat32Inode> {
        let inodes = self.inodes.read();
        inodes.get(&ino).cloned().ok_or(VfsError::NotFound)
    }
    
    fn alloc_ino(&self) -> Ino {
        self.next_ino.fetch_add(1, Ordering::SeqCst)
    }
}

// Clone implementation for Fat32Inode
impl Clone for Fat32Inode {
    fn clone(&self) -> Self {
        Self {
            cluster: self.cluster,
            size: self.size,
            is_dir: self.is_dir,
            dir_cluster: self.dir_cluster,
            dir_entry_offset: self.dir_entry_offset,
            name: self.name.clone(),
        }
    }
}

impl FileSystem for Fat32Fs {
    fn name(&self) -> &str {
        "fat32"
    }
    
    fn root_inode(&self) -> Ino {
        self.root_ino
    }
    
    fn get_file(&self, ino: Ino) -> VfsResult<Arc<dyn FileOps>> {
        let inode = self.get_inode(ino)?;
        if inode.is_dir {
            return Err(VfsError::IsDirectory);
        }
        
        Ok(Arc::new(Fat32File {
            fs: self as *const Fat32Fs,
            ino,
            cluster: RwLock::new(inode.cluster),
            size: RwLock::new(inode.size),
        }))
    }
    
    fn get_dir(&self, ino: Ino) -> VfsResult<Arc<dyn DirOps>> {
        let inode = self.get_inode(ino)?;
        if !inode.is_dir {
            return Err(VfsError::NotDirectory);
        }
        
        Ok(Arc::new(Fat32Dir {
            fs: self as *const Fat32Fs,
            ino,
            cluster: inode.cluster,
        }))
    }
    
    fn stat(&self, ino: Ino) -> VfsResult<Stat> {
        let inode = self.get_inode(ino)?;
        
        Ok(Stat {
            ino,
            file_type: if inode.is_dir { FileType::Directory } else { FileType::Regular },
            size: inode.size,
            blocks: (inode.size + 511) / 512,
            block_size: self.bpb.cluster_size() as u32,
            mode: if inode.is_dir { 0o755 } else { 0o644 },
            uid: 0,
            gid: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }
}

/// FAT32 File operations
struct Fat32File {
    fs: *const Fat32Fs,
    ino: Ino,
    cluster: RwLock<u32>,
    size: RwLock<u64>,
}

unsafe impl Send for Fat32File {}
unsafe impl Sync for Fat32File {}

impl FileOps for Fat32File {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let size = *self.size.read();
        let cluster = *self.cluster.read();
        
        if offset >= size {
            return Ok(0);
        }
        
        let fs = unsafe { &*self.fs };
        let to_read = core::cmp::min(buf.len() as u64, size - offset) as usize;
        
        if cluster < 2 {
            // Empty file
            return Ok(0);
        }
        
        // Read file data
        let data = fs.read_chain(cluster, Some(size))?;
        
        let start = offset as usize;
        let end = start + to_read;
        
        if end <= data.len() {
            buf[..to_read].copy_from_slice(&data[start..end]);
            Ok(to_read)
        } else {
            Err(VfsError::IoError)
        }
    }
    
    fn write(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        
        let fs = unsafe { &*self.fs };
        let current_cluster = *self.cluster.read();
        let current_size = *self.size.read();
        
        // If file has no cluster yet, allocate one
        let start_cluster = if current_cluster < 2 {
            let new_cluster = fs.allocate_cluster()?;
            *self.cluster.write() = new_cluster;
            new_cluster
        } else {
            current_cluster
        };
        
        // Write data
        let (new_cluster, new_size) = fs.write_file_data(start_cluster, offset, buf, current_size)?;
        
        // Update file state
        *self.cluster.write() = new_cluster;
        *self.size.write() = new_size;
        
        // Update inode cache
        if let Some(mut inodes) = fs.inodes.try_write() {
            if let Some(inode) = inodes.get_mut(&self.ino) {
                inode.cluster = new_cluster;
                inode.size = new_size;
            }
        }
        
        // Persist size+cluster to directory entry on disk
        let _ = fs.update_dir_entry(self.ino, new_cluster, new_size);
        
        Ok(buf.len())
    }
    
    fn stat(&self) -> VfsResult<Stat> {
        let size = *self.size.read();
        Ok(Stat {
            ino: self.ino,
            file_type: FileType::Regular,
            size,
            blocks: (size + 511) / 512,
            block_size: 512,
            mode: 0o644,
            uid: 0,
            gid: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }
}

/// FAT32 Directory operations
struct Fat32Dir {
    fs: *const Fat32Fs,
    ino: Ino,
    cluster: u32,
}

unsafe impl Send for Fat32Dir {}
unsafe impl Sync for Fat32Dir {}

impl DirOps for Fat32Dir {
    fn lookup(&self, name: &str) -> VfsResult<Ino> {
        let fs = unsafe { &*self.fs };
        
        // Check cache first
        {
            let inodes = fs.inodes.read();
            for (ino, inode) in inodes.iter() {
                if inode.name.eq_ignore_ascii_case(name) {
                    return Ok(*ino);
                }
            }
        }
        
        // Parse directory
        let entries = fs.parse_directory(self.cluster)?;
        
        for (entry_name, entry, byte_offset) in entries {
            if entry_name.eq_ignore_ascii_case(name) {
                let ino = fs.alloc_ino();
                let inode = Fat32Inode {
                    cluster: entry.cluster(),
                    size: entry.size() as u64,
                    is_dir: entry.is_directory(),
                    name: entry_name,
                    dir_cluster: self.cluster,
                    dir_entry_offset: byte_offset,
                };
                fs.inodes.write().insert(ino, inode);
                return Ok(ino);
            }
        }
        
        Err(VfsError::NotFound)
    }
    
    fn readdir(&self) -> VfsResult<Vec<DirEntry>> {
        let fs = unsafe { &*self.fs };
        let entries = fs.parse_directory(self.cluster)?;
        
        let mut result = Vec::new();
        
        for (name, entry, byte_offset) in entries {
            let ino = fs.alloc_ino();
            let is_dir = entry.is_directory();
            
            // Cache inode
            fs.inodes.write().insert(ino, Fat32Inode {
                cluster: entry.cluster(),
                size: entry.size() as u64,
                is_dir,
                name: name.clone(),
                dir_cluster: self.cluster,
                dir_entry_offset: byte_offset,
            });
            
            result.push(DirEntry {
                name,
                ino,
                file_type: if is_dir { FileType::Directory } else { FileType::Regular },
            });
        }
        
        Ok(result)
    }
    
    fn create(&self, name: &str, file_type: FileType) -> VfsResult<Ino> {
        let fs = unsafe { &*self.fs };
        
        // Check if entry already exists
        if self.lookup(name).is_ok() {
            return Err(VfsError::AlreadyExists);
        }
        
        let is_dir = matches!(file_type, FileType::Directory);
        
        // Create the entry on disk
        let (cluster, _size) = fs.create_entry(self.cluster, name, is_dir)?;
        
        // Find the directory entry offset we just created
        let dir_entry_offset = if let Ok(entries) = fs.parse_directory(self.cluster) {
            entries.iter()
                .find(|(n, _, _)| n.eq_ignore_ascii_case(name))
                .map(|(_, _, off)| *off)
                .unwrap_or(0)
        } else {
            0
        };
        
        // Create inode
        let ino = fs.alloc_ino();
        fs.inodes.write().insert(ino, Fat32Inode {
            cluster,
            size: 0,
            is_dir,
            name: name.to_string(),
            dir_cluster: self.cluster,
            dir_entry_offset,
        });
        
        Ok(ino)
    }
    
    fn unlink(&self, name: &str) -> VfsResult<()> {
        let fs = unsafe { &*self.fs };
        
        // Don't allow unlinking . or ..
        if name == "." || name == ".." {
            return Err(VfsError::InvalidPath);
        }
        
        // Delete the entry from disk
        fs.delete_entry(self.cluster, name)?;
        
        // Remove from inode cache
        let mut inodes = fs.inodes.write();
        let to_remove: Vec<Ino> = inodes.iter()
            .filter(|(_, inode)| inode.name.eq_ignore_ascii_case(name))
            .map(|(ino, _)| *ino)
            .collect();
        
        for ino in to_remove {
            inodes.remove(&ino);
        }
        
        Ok(())
    }
    
    fn stat(&self) -> VfsResult<Stat> {
        let fs = unsafe { &*self.fs };
        fs.stat(self.ino)
    }
}

/// Try to auto-detect and mount FAT32 from AHCI
pub fn try_mount_fat32() -> Option<Arc<Fat32Fs>> {
    use crate::drivers::partition::{parse_partition_table, PartitionType};
    use crate::drivers::ahci;
    
    // Get AHCI ports info
    let devices = ahci::list_devices();
    crate::log_debug!("[FAT32] Checking {} AHCI devices", devices.len());
    
    for device in devices {
        let port = device.port_num;
        let total_sectors = device.sector_count;
        crate::log_debug!("[FAT32] Port {}: {} sectors", port, total_sectors);
        
        // Create read closure for this port
        let read_fn = |sector: u64, buf: &mut [u8]| -> Result<(), &'static str> {
            ahci::read_sectors(port, sector, 1, buf)
                .map(|_| ())
        };
        
        // First, try to mount as superfloppy (no partition table, entire disk is FAT32)
        let reader_whole = Arc::new(AhciBlockReader::new(port as usize, 0));
        if let Ok(fs) = Fat32Fs::mount(reader_whole) {
            crate::log!("[FAT32] Mounted superfloppy FAT32 from port {}", port);
            return Some(Arc::new(fs));
        }
        
        // If that fails, try to parse partition table
        if let Ok(table) = parse_partition_table(read_fn, total_sectors) {
            for partition in &table.partitions {
                // Check for FAT32 partition types
                match partition.partition_type {
                    PartitionType::Fat32 | 
                    PartitionType::Fat32Lba |
                    PartitionType::MicrosoftBasicData => {
                        crate::log!("[FAT32] Found partition at LBA {}", partition.start_lba);
                        
                        let reader = Arc::new(AhciBlockReader::new(port as usize, partition.start_lba));
                        
                        match Fat32Fs::mount(reader) {
                            Ok(fs) => {
                                crate::log!("[FAT32] Successfully mounted partition");
                                return Some(Arc::new(fs));
                            }
                            Err(e) => {
                                crate::log_warn!("[FAT32] Mount failed: {:?}", e);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    None
}
