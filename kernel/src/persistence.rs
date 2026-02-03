//! Persistence System for TrustOS
//!
//! Saves downloaded files and configuration to disk for persistence across reboots.
//! Uses a simple custom format on raw disk sectors.

use alloc::vec::Vec;
use alloc::string::String;
use alloc::vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};

/// Magic signature for persistence data
const PERSIST_MAGIC: &[u8; 8] = b"TRUSTPST";

/// Version of persistence format
const PERSIST_VERSION: u32 = 1;

/// Start sector for persistence data (after first 1MB to avoid boot area)
const PERSIST_START_SECTOR: u64 = 2048; // 1MB offset

/// Maximum persistence size: 16MB
const PERSIST_MAX_SECTORS: u64 = 32768; // 16MB

/// Sector size
const SECTOR_SIZE: usize = 512;

/// Whether persistence is available
static PERSISTENCE_AVAILABLE: AtomicBool = AtomicBool::new(false);

/// Whether user accepted persistence restore
static PERSISTENCE_ENABLED: AtomicBool = AtomicBool::new(false);

/// AHCI port for persistence (detected at init)
static PERSIST_PORT: Mutex<Option<u8>> = Mutex::new(None);

/// Persistence header (stored in first sector)
#[repr(C, packed)]
struct PersistHeader {
    magic: [u8; 8],      // "TRUSTPST"
    version: u32,         // Format version
    entry_count: u32,     // Number of files stored
    total_size: u64,      // Total data size
    checksum: u32,        // Simple checksum
    _reserved: [u8; 484], // Pad to 512 bytes
}

/// File entry header
#[repr(C, packed)]
struct FileEntry {
    path_len: u16,        // Length of path string
    data_len: u32,        // Length of file data
    _reserved: [u8; 2],   // Padding
    // Followed by: path bytes, then data bytes
}

/// Initialize persistence system
pub fn init() {
    crate::serial_println!("[PERSIST] Initializing persistence system...");
    
    // Find AHCI disk
    let port = find_ahci_disk();
    if port.is_none() {
        crate::serial_println!("[PERSIST] No AHCI disk found, persistence disabled");
        return;
    }
    
    let port = port.unwrap();
    *PERSIST_PORT.lock() = Some(port);
    
    // Check if persistence data exists
    if check_persistence_exists(port) {
        PERSISTENCE_AVAILABLE.store(true, Ordering::Relaxed);
        crate::serial_println!("[PERSIST] Found existing persistence data on port {}", port);
    } else {
        crate::serial_println!("[PERSIST] No existing persistence data found");
    }
}

/// Find an AHCI disk port
fn find_ahci_disk() -> Option<u8> {
    // Get list of AHCI ports
    let ports = crate::drivers::ahci::list_devices();
    for port in ports {
        // Check if it's a disk (ATA device)
        if port.sector_count > 0 {
            // Try to read a sector to verify it's working
            let mut buffer = [0u8; 512];
            if crate::drivers::ahci::read_sectors(port.port_num, 0, 1, &mut buffer).is_ok() {
                return Some(port.port_num);
            }
        }
    }
    None
}

/// Check if persistence data exists on disk
fn check_persistence_exists(port: u8) -> bool {
    let mut buffer = [0u8; 512];
    
    if crate::drivers::ahci::read_sectors(port, PERSIST_START_SECTOR, 1, &mut buffer).is_err() {
        return false;
    }
    
    // Check magic signature
    &buffer[0..8] == PERSIST_MAGIC
}

/// Ask user if they want to restore persistence
pub fn prompt_restore() -> bool {
    if !PERSISTENCE_AVAILABLE.load(Ordering::Relaxed) {
        return false;
    }
    
    crate::println!();
    crate::println_color!(0x00FFFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(0x00FFFF, "║           Saved Data Detected                                ║");
    crate::println_color!(0x00FFFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    crate::println!("  Previously downloaded files were found on disk.");
    crate::println!("  Do you want to restore them? (Y/n)");
    crate::println!();
    crate::print!("  > ");
    
    // Wait for user input
    let response = read_char_blocking();
    
    let restore = response != b'n' && response != b'N';
    
    if restore {
        crate::println!("Yes");
        crate::println!();
        crate::println_color!(0x00FF00, "  Restoring saved data...");
        
        if let Err(e) = restore_all() {
            crate::println_color!(0xFF0000, "  Error restoring: {}", e);
            return false;
        }
        
        PERSISTENCE_ENABLED.store(true, Ordering::Relaxed);
        crate::println_color!(0x00FF00, "  Data restored successfully!");
    } else {
        crate::println!("No");
        crate::println!("  Starting fresh.");
    }
    
    crate::println!();
    restore
}

/// Read a character blocking
fn read_char_blocking() -> u8 {
    loop {
        if let Some(c) = crate::keyboard::read_char() {
            return c;
        }
        core::hint::spin_loop();
    }
}

/// Save a file to persistent storage
pub fn save_file(path: &str, data: &[u8]) -> Result<(), &'static str> {
    let port = PERSIST_PORT.lock().ok_or("Persistence not available")?;
    
    crate::serial_println!("[PERSIST] Saving {} ({} bytes)", path, data.len());
    
    // Read current header
    let mut header_buf = [0u8; 512];
    let mut header = if check_persistence_exists(port) {
        crate::drivers::ahci::read_sectors(port, PERSIST_START_SECTOR, 1, &mut header_buf)
            .map_err(|_| "Failed to read header")?;
        parse_header(&header_buf)?
    } else {
        // Create new header
        PersistHeader {
            magic: *PERSIST_MAGIC,
            version: PERSIST_VERSION,
            entry_count: 0,
            total_size: 0,
            checksum: 0,
            _reserved: [0; 484],
        }
    };
    
    // Find next available sector
    let data_start_sector = PERSIST_START_SECTOR + 1 + (header.total_size + 511) / 512;
    
    // Check size limit
    if data_start_sector + ((data.len() as u64 + path.len() as u64 + 16) / 512) + 1 
        > PERSIST_START_SECTOR + PERSIST_MAX_SECTORS {
        return Err("Persistence storage full");
    }
    
    // Build entry: [FileEntry header][path bytes][data bytes]
    let entry_header = FileEntry {
        path_len: path.len() as u16,
        data_len: data.len() as u32,
        _reserved: [0; 2],
    };
    
    let mut entry_data: Vec<u8> = Vec::new();
    
    // Add entry header bytes
    let header_bytes: [u8; 8] = unsafe { core::mem::transmute(entry_header) };
    entry_data.extend_from_slice(&header_bytes);
    
    // Add path
    entry_data.extend_from_slice(path.as_bytes());
    
    // Add data
    entry_data.extend_from_slice(data);
    
    // Pad to sector boundary
    while entry_data.len() % 512 != 0 {
        entry_data.push(0);
    }
    
    // Write entry sectors
    let sectors_needed = entry_data.len() / 512;
    for i in 0..sectors_needed {
        let sector = data_start_sector + i as u64;
        let offset = i * 512;
        let mut sector_buf = [0u8; 512];
        sector_buf.copy_from_slice(&entry_data[offset..offset + 512]);
        
        crate::drivers::ahci::write_sectors(port, sector, 1, &sector_buf)
            .map_err(|_| "Failed to write data sector")?;
    }
    
    // Update header
    header.entry_count += 1;
    header.total_size += entry_data.len() as u64;
    header.checksum = compute_checksum(&entry_data);
    
    // Write updated header
    let mut header_out = [0u8; 512];
    header_out[0..8].copy_from_slice(&header.magic);
    header_out[8..12].copy_from_slice(&header.version.to_le_bytes());
    header_out[12..16].copy_from_slice(&header.entry_count.to_le_bytes());
    header_out[16..24].copy_from_slice(&header.total_size.to_le_bytes());
    header_out[24..28].copy_from_slice(&header.checksum.to_le_bytes());
    
    crate::drivers::ahci::write_sectors(port, PERSIST_START_SECTOR, 1, &header_out)
        .map_err(|_| "Failed to write header")?;
    
    crate::serial_println!("[PERSIST] Saved {} successfully", path);
    Ok(())
}

/// Restore all files from persistent storage
fn restore_all() -> Result<(), &'static str> {
    let port = *PERSIST_PORT.lock();
    let port = port.ok_or("Persistence not available")?;
    
    // Read header
    let mut header_buf = [0u8; 512];
    crate::drivers::ahci::read_sectors(port, PERSIST_START_SECTOR, 1, &mut header_buf)
        .map_err(|_| "Failed to read header")?;
    
    let header = parse_header(&header_buf)?;
    
    let entry_count = header.entry_count;
    let total_size = header.total_size;
    
    // Sanity checks to prevent crashes from corrupted data
    if entry_count > 1000 {
        return Err("Corrupted: too many entries");
    }
    if total_size > 100 * 1024 * 1024 {
        return Err("Corrupted: size too large");
    }
    if total_size == 0 {
        return Ok(()); // Nothing to restore
    }
    
    crate::serial_println!("[PERSIST] Restoring {} files ({} bytes)", 
        entry_count, total_size);
    
    // Read all data sectors
    let total_sectors = (total_size + 511) / 512;
    let mut all_data: Vec<u8> = vec![0u8; total_size as usize];
    
    for i in 0..total_sectors {
        let sector = PERSIST_START_SECTOR + 1 + i;
        let offset = (i as usize) * 512;
        let mut sector_buf = [0u8; 512];
        
        crate::drivers::ahci::read_sectors(port, sector, 1, &mut sector_buf)
            .map_err(|_| "Failed to read data sector")?;
        
        let copy_len = core::cmp::min(512, all_data.len() - offset);
        all_data[offset..offset + copy_len].copy_from_slice(&sector_buf[..copy_len]);
    }
    
    // Parse entries
    let mut offset = 0;
    let mut restored = 0;
    
    while offset + 8 <= all_data.len() && restored < header.entry_count {
        // Read entry header
        let path_len = u16::from_le_bytes([all_data[offset], all_data[offset + 1]]) as usize;
        let data_len = u32::from_le_bytes([
            all_data[offset + 2], all_data[offset + 3],
            all_data[offset + 4], all_data[offset + 5],
        ]) as usize;
        
        offset += 8;
        
        if offset + path_len + data_len > all_data.len() {
            break;
        }
        
        // Read path
        let path = match core::str::from_utf8(&all_data[offset..offset + path_len]) {
            Ok(s) => s,
            Err(_) => {
                offset += path_len + data_len;
                continue;
            }
        };
        offset += path_len;
        
        // Read data
        let data = &all_data[offset..offset + data_len];
        offset += data_len;
        
        // Align to 512-byte boundary for next entry
        offset = (offset + 511) & !511;
        
        // Restore file to ramfs
        crate::serial_println!("[PERSIST] Restoring: {} ({} bytes)", path, data_len);
        
        // Create parent directories and file
        let result = crate::ramfs::with_fs(|fs| {
            // Create parent directories
            let mut current = String::new();
            for part in path.split('/').filter(|p| !p.is_empty()) {
                if !path.ends_with(part) {
                    current.push('/');
                    current.push_str(part);
                    let _ = fs.mkdir(&current);
                }
            }
            
            // Write file
            let _ = fs.touch(path);
            fs.write_file(path, data)
        });
        
        if result.is_ok() {
            restored += 1;
            crate::print!(".");
        }
    }
    
    crate::println!();
    crate::println!("  Restored {} files", restored);
    
    Ok(())
}

/// Parse header from buffer
fn parse_header(buf: &[u8; 512]) -> Result<PersistHeader, &'static str> {
    if &buf[0..8] != PERSIST_MAGIC {
        return Err("Invalid magic signature");
    }
    
    let version = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
    if version != PERSIST_VERSION {
        return Err("Incompatible version");
    }
    
    Ok(PersistHeader {
        magic: *PERSIST_MAGIC,
        version,
        entry_count: u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]),
        total_size: u64::from_le_bytes([
            buf[16], buf[17], buf[18], buf[19],
            buf[20], buf[21], buf[22], buf[23],
        ]),
        checksum: u32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]),
        _reserved: [0; 484],
    })
}

/// Compute simple checksum
fn compute_checksum(data: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    for byte in data {
        sum = sum.wrapping_add(*byte as u32);
    }
    sum
}

/// Clear all persisted data
pub fn clear() -> Result<(), &'static str> {
    let port = *PERSIST_PORT.lock();
    let port = port.ok_or("Persistence not available")?;
    
    // Write empty header
    let header_out = [0u8; 512];
    crate::drivers::ahci::write_sectors(port, PERSIST_START_SECTOR, 1, &header_out)
        .map_err(|_| "Failed to clear persistence")?;
    
    crate::serial_println!("[PERSIST] Persistence data cleared");
    Ok(())
}

/// Check if persistence is enabled
pub fn is_enabled() -> bool {
    PERSISTENCE_ENABLED.load(Ordering::Relaxed)
}

/// Check if persistence is available
pub fn is_available() -> bool {
    PERSISTENCE_AVAILABLE.load(Ordering::Relaxed)
}

/// Get persistence status
pub fn status() -> (&'static str, u32, u64) {
    let port = *PERSIST_PORT.lock();
    
    if port.is_none() {
        return ("No disk", 0, 0);
    }
    
    let port = port.unwrap();
    let mut header_buf = [0u8; 512];
    
    if crate::drivers::ahci::read_sectors(port, PERSIST_START_SECTOR, 1, &mut header_buf).is_err() {
        return ("Read error", 0, 0);
    }
    
    if let Ok(header) = parse_header(&header_buf) {
        ("Active", header.entry_count, header.total_size)
    } else {
        ("Empty", 0, 0)
    }
}
