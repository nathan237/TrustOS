//! Legacy RAM Disk Driver (DEPRECATED)
//!
//! This module provides a simple flat ramdisk with no filesystem abstraction.
//! It exists for backward compatibility only.
//!
//! **All new code should use [`crate::vfs`] instead**, which provides:
//! - A proper Virtual File System with mount points
//! - Device files via devfs
//! - Proc filesystem via procfs
//! - FAT32 and ext4 support
//! - TrustFS (native filesystem)
//! - Block caching and write-ahead logging
//!
//! This module will be removed in a future release.

use spin::Mutex;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

/// Disk sector size
pub const SECTOR_SIZE: usize = 512;

/// RAM disk size: 128 KB (256 sectors) - fits in 256KB heap
pub const RAMDISK_SECTORS: u64 = 256;
pub const RAMDISK_SIZE: usize = (RAMDISK_SECTORS as usize) * SECTOR_SIZE;

/// Maximum sectors per read/write
pub const MAX_SECTORS: u8 = 128;

/// Disk information
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub model: String,
    pub serial: String,
    pub sectors: u64,
    pub size_mb: u64,
    pub present: bool,
}

/// RAM Disk storage
struct RamDisk {
    data: Vec<u8>,
    info: DiskInfo,
}

/// Global disk state
static DISK: Mutex<Option<RamDisk>> = Mutex::new(None);

/// Statistics
static READ_COUNT: AtomicU64 = AtomicU64::new(0);
static WRITE_COUNT: AtomicU64 = AtomicU64::new(0);
static BYTES_READ: AtomicU64 = AtomicU64::new(0);
static BYTES_WRITTEN: AtomicU64 = AtomicU64::new(0);

/// Initialize disk driver (RAM Disk)
pub fn init() {
    let data = vec![0u8; RAMDISK_SIZE];
    
    let info = DiskInfo {
        model: String::from("T-RustOS RAMDisk"),
        serial: String::from("RAMD-0001"),
        sectors: RAMDISK_SECTORS,
        size_mb: (RAMDISK_SIZE / 1024 / 1024) as u64,
        present: true,
    };
    
    crate::log!("[DISK] RAMDisk initialized: {} MB ({} sectors)", 
        info.size_mb, info.sectors);
    
    *DISK.lock() = Some(RamDisk { data, info });
    
    // Initialize with a simple filesystem signature
    init_filesystem();
}

/// Initialize basic filesystem structure on the RAM disk
fn init_filesystem() {
    // Write a simple boot sector / filesystem header
    let mut disk = DISK.lock();
    if let Some(ref mut ramdisk) = *disk {
        // Sector 0: Boot sector with magic signature
        ramdisk.data[0] = b'T';
        ramdisk.data[1] = b'R';
        ramdisk.data[2] = b'U';
        ramdisk.data[3] = b'S';
        ramdisk.data[4] = b'T';
        ramdisk.data[5] = b'F';
        ramdisk.data[6] = b'S';  // "TRUSTFS" magic
        ramdisk.data[7] = 0x01; // Version 1
        
        // Sector 1: Root directory (empty)
        let sector1_offset = SECTOR_SIZE;
        ramdisk.data[sector1_offset] = 0; // No entries yet
        
        crate::log!("[DISK] Filesystem initialized (TRUSTFS v1)");
    }
}

/// Check if disk is available
pub fn is_available() -> bool {
    DISK.lock().is_some()
}

/// Get disk information
pub fn get_info() -> Option<DiskInfo> {
    DISK.lock().as_ref().map(|d| d.info.clone())
}

/// Get disk statistics
pub fn get_stats() -> (u64, u64, u64, u64) {
    (
        READ_COUNT.load(Ordering::Relaxed),
        WRITE_COUNT.load(Ordering::Relaxed),
        BYTES_READ.load(Ordering::Relaxed),
        BYTES_WRITTEN.load(Ordering::Relaxed),
    )
}

/// Read sectors from RAM disk
pub fn read_sectors(lba: u64, count: u8, buffer: &mut [u8]) -> Result<usize, &'static str> {
    if count == 0 {
        return Err("Invalid sector count");
    }
    
    let required_size = count as usize * SECTOR_SIZE;
    if buffer.len() < required_size {
        return Err("Buffer too small");
    }
    
    let disk = DISK.lock();
    let ramdisk = disk.as_ref().ok_or("Disk not initialized")?;
    
    let start_offset = (lba as usize) * SECTOR_SIZE;
    let end_offset = start_offset + required_size;
    
    if end_offset > ramdisk.data.len() {
        return Err("Read past end of disk");
    }
    
    buffer[..required_size].copy_from_slice(&ramdisk.data[start_offset..end_offset]);
    
    READ_COUNT.fetch_add(count as u64, Ordering::Relaxed);
    BYTES_READ.fetch_add(required_size as u64, Ordering::Relaxed);
    
    Ok(required_size)
}

/// Write sectors to RAM disk
pub fn write_sectors(lba: u64, count: u8, buffer: &[u8]) -> Result<usize, &'static str> {
    if count == 0 {
        return Err("Invalid sector count");
    }
    
    let required_size = count as usize * SECTOR_SIZE;
    if buffer.len() < required_size {
        return Err("Buffer too small");
    }
    
    let mut disk = DISK.lock();
    let ramdisk = disk.as_mut().ok_or("Disk not initialized")?;
    
    let start_offset = (lba as usize) * SECTOR_SIZE;
    let end_offset = start_offset + required_size;
    
    if end_offset > ramdisk.data.len() {
        return Err("Write past end of disk");
    }
    
    ramdisk.data[start_offset..end_offset].copy_from_slice(&buffer[..required_size]);
    
    WRITE_COUNT.fetch_add(count as u64, Ordering::Relaxed);
    BYTES_WRITTEN.fetch_add(required_size as u64, Ordering::Relaxed);
    
    Ok(required_size)
}

/// Read a single sector
pub fn read_sector(lba: u64) -> Result<[u8; SECTOR_SIZE], &'static str> {
    let mut buffer = [0u8; SECTOR_SIZE];
    read_sectors(lba, 1, &mut buffer)?;
    Ok(buffer)
}

/// Write a single sector
pub fn write_sector(lba: u64, data: &[u8; SECTOR_SIZE]) -> Result<(), &'static str> {
    write_sectors(lba, 1, data)?;
    Ok(())
}

/// Format the disk (clear all data)
pub fn format() -> Result<(), &'static str> {
    {
        let mut disk = DISK.lock();
        let ramdisk = disk.as_mut().ok_or("Disk not initialized")?;
        
        // Zero out all data
        for byte in ramdisk.data.iter_mut() {
            *byte = 0;
        }
    }
    
    // Re-initialize filesystem
    init_filesystem();
    
    // Reset stats
    READ_COUNT.store(0, Ordering::Relaxed);
    WRITE_COUNT.store(0, Ordering::Relaxed);
    BYTES_READ.store(0, Ordering::Relaxed);
    BYTES_WRITTEN.store(0, Ordering::Relaxed);
    
    crate::log!("[DISK] Disk formatted");
    Ok(())
}

/// Dump the first few bytes of a sector (for debugging)
pub fn dump_sector(lba: u64) -> Result<String, &'static str> {
    let sector = read_sector(lba)?;
    
    use alloc::format;
    let mut result = format!("Sector {} (LBA):\n", lba);
    
    // Show first 64 bytes in hex
    for row in 0..4 {
        let offset = row * 16;
        result.push_str(&format!("{:04X}: ", offset));
        
        for i in 0..16 {
            result.push_str(&format!("{:02X} ", sector[offset + i]));
        }
        
        result.push_str(" |");
        for i in 0..16 {
            let c = sector[offset + i];
            if c >= 0x20 && c < 0x7F {
                result.push(c as char);
            } else {
                result.push('.');
            }
        }
        result.push_str("|\n");
    }
    
    Ok(result)
}

//=============================================================================
// Simple File System Operations
//=============================================================================

/// File entry in the directory
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub size: u32,
    pub start_sector: u32,
    pub is_directory: bool,
}

const MAX_FILENAME: usize = 32;
const MAX_FILES: usize = 10;
const DIR_SECTOR: u64 = 1;
const DATA_START_SECTOR: u64 = 10;

/// List files in root directory
pub fn list_files() -> Result<Vec<FileEntry>, &'static str> {
    let sector = read_sector(DIR_SECTOR)?;
    let mut files = Vec::new();
    
    let file_count = sector[0] as usize;
    if file_count > MAX_FILES {
        return Err("Corrupted directory");
    }
    
    let entry_size = 1 + MAX_FILENAME + 4 + 4 + 1; // flags + name + size + sector + type
    
    for i in 0..file_count {
        let offset = 1 + i * entry_size;
        
        if offset + entry_size > SECTOR_SIZE {
            break;
        }
        
        if sector[offset] == 0 {
            continue; // Deleted entry
        }
        
        // Read name (null-terminated)
        let name_start = offset + 1;
        let mut name_end = name_start;
        while name_end < name_start + MAX_FILENAME && sector[name_end] != 0 {
            name_end += 1;
        }
        let name = String::from_utf8_lossy(&sector[name_start..name_end]).into_owned();
        
        let size_offset = offset + 1 + MAX_FILENAME;
        let size = u32::from_le_bytes([
            sector[size_offset],
            sector[size_offset + 1],
            sector[size_offset + 2],
            sector[size_offset + 3],
        ]);
        
        let sector_offset = size_offset + 4;
        let start_sector = u32::from_le_bytes([
            sector[sector_offset],
            sector[sector_offset + 1],
            sector[sector_offset + 2],
            sector[sector_offset + 3],
        ]);
        
        let is_directory = sector[sector_offset + 4] != 0;
        
        files.push(FileEntry {
            name,
            size,
            start_sector,
            is_directory,
        });
    }
    
    Ok(files)
}

/// Create a new file
pub fn create_file(name: &str, data: &[u8]) -> Result<(), &'static str> {
    if name.len() > MAX_FILENAME - 1 {
        return Err("Filename too long");
    }
    
    // Read current directory
    let mut dir_sector = read_sector(DIR_SECTOR)?;
    let file_count = dir_sector[0] as usize;
    
    if file_count >= MAX_FILES {
        return Err("Directory full");
    }
    
    // Find a free data sector
    let sectors_needed = (data.len() + SECTOR_SIZE - 1) / SECTOR_SIZE;
    if sectors_needed > 255 {
        return Err("File too large");
    }
    
    let start_sector = DATA_START_SECTOR + (file_count as u64 * 256); // Simple allocation
    
    // Write file data
    let mut remaining = data;
    let mut current_sector = start_sector;
    
    while !remaining.is_empty() {
        let mut sector_data = [0u8; SECTOR_SIZE];
        let chunk_size = remaining.len().min(SECTOR_SIZE);
        sector_data[..chunk_size].copy_from_slice(&remaining[..chunk_size]);
        
        write_sector(current_sector, &sector_data)?;
        
        remaining = &remaining[chunk_size..];
        current_sector += 1;
    }
    
    // Add directory entry
    let entry_size = 1 + MAX_FILENAME + 4 + 4 + 1;
    let entry_offset = 1 + file_count * entry_size;
    
    dir_sector[entry_offset] = 1; // Active entry
    
    // Name
    for (i, byte) in name.bytes().enumerate() {
        dir_sector[entry_offset + 1 + i] = byte;
    }
    
    // Size
    let size_bytes = (data.len() as u32).to_le_bytes();
    let size_offset = entry_offset + 1 + MAX_FILENAME;
    dir_sector[size_offset..size_offset + 4].copy_from_slice(&size_bytes);
    
    // Start sector
    let sector_bytes = (start_sector as u32).to_le_bytes();
    let sector_offset = size_offset + 4;
    dir_sector[sector_offset..sector_offset + 4].copy_from_slice(&sector_bytes);
    
    // Type (0 = file)
    dir_sector[sector_offset + 4] = 0;
    
    // Update file count
    dir_sector[0] = (file_count + 1) as u8;
    
    write_sector(DIR_SECTOR, &dir_sector)?;
    
    crate::log!("[DISK] Created file: {} ({} bytes)", name, data.len());
    
    Ok(())
}

/// Read a file by name
pub fn read_file(name: &str) -> Result<Vec<u8>, &'static str> {
    let files = list_files()?;
    
    let file = files.iter().find(|f| f.name == name)
        .ok_or("File not found")?;
    
    let sectors_needed = (file.size as usize + SECTOR_SIZE - 1) / SECTOR_SIZE;
    let sectors_needed = sectors_needed.max(1);
    let mut data = Vec::with_capacity(file.size as usize);
    
    for i in 0..sectors_needed {
        let sector = read_sector(file.start_sector as u64 + i as u64)?;
        let remaining = file.size as usize - data.len();
        let chunk_size = remaining.min(SECTOR_SIZE);
        data.extend_from_slice(&sector[..chunk_size]);
    }
    
    Ok(data)
}

/// Delete a file by name
pub fn delete_file(name: &str) -> Result<(), &'static str> {
    let mut dir_sector = read_sector(DIR_SECTOR)?;
    let file_count = dir_sector[0] as usize;
    
    let entry_size = 1 + MAX_FILENAME + 4 + 4 + 1;
    
    for i in 0..file_count {
        let offset = 1 + i * entry_size;
        
        if dir_sector[offset] == 0 {
            continue;
        }
        
        // Check name
        let name_start = offset + 1;
        let mut name_end = name_start;
        while name_end < name_start + MAX_FILENAME && dir_sector[name_end] != 0 {
            name_end += 1;
        }
        let entry_name = String::from_utf8_lossy(&dir_sector[name_start..name_end]);
        
        if entry_name == name {
            // Mark as deleted
            dir_sector[offset] = 0;
            write_sector(DIR_SECTOR, &dir_sector)?;
            crate::log!("[DISK] Deleted file: {}", name);
            return Ok(());
        }
    }
    
    Err("File not found")
}
