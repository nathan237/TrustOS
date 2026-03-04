//! Load audio from the AHCI data disk at runtime.
//!
//! Disk layout (prepared by `prepare-data-disk.py`):
//!   Sector 0       : Header — magic "TWAV", version, sizes, CRC32
//!   Sector 1..N    : Raw WAV file bytes
//!
//! The data disk is attached on SATA port 1 in VirtualBox.

use alloc::vec::Vec;
use alloc::vec;

const SECTOR_SIZE: usize = 512;
/// Maximum sectors to read per AHCI command (driver limit = 128)
const MAX_SECTORS_PER_READ: u16 = 128;

/// Header stored in sector 0
#[repr(C)]
struct DiskAudioHeader {
    magic: [u8; 4],       // "TWAV"
    version: u32,          // 1
    wav_size: u64,         // bytes
    wav_start_lba: u32,    // 1
    wav_sector_count: u32, // ceil(wav_size / 512)
    crc32: u32,            // CRC32 of WAV data
}

/// Try to find which AHCI port has our audio data disk.
/// The audio disk is on SATA port 2 (port 0=CDROM, port 1=TrustFS persistence).
fn find_data_port() -> Option<u8> {
    if !crate::drivers::ahci::is_initialized() {
        return None;
    }
    let devices = crate::drivers::ahci::list_devices();
    // Prefer port 2 (configured in VBox for audio)
    for dev in &devices {
        if dev.port_num == 2 {
            return Some(2);
        }
    }
    // Fallback: any port > 1 that is a SATA drive
    for dev in &devices {
        if dev.port_num > 1 && dev.device_type == crate::drivers::ahci::AhciDeviceType::Sata {
            return Some(dev.port_num);
        }
    }
    None
}

/// Load the WAV file from the data disk into a Vec<u8>.
/// Returns the raw WAV bytes (including the WAV header).
pub fn load_wav_from_disk() -> Result<Vec<u8>, &'static str> {
    let port = find_data_port().ok_or("No data disk found on AHCI")?;
    
    crate::serial_println!("[DISK-AUDIO] Reading header from port {}...", port);
    
    // Read header sector (LBA 0)
    let mut header_buf = vec![0u8; SECTOR_SIZE];
    crate::drivers::ahci::read_sectors(port, 0, 1, &mut header_buf)?;
    
    // Parse header
    if &header_buf[0..4] != b"TWAV" {
        crate::serial_println!("[DISK-AUDIO] Bad magic: {:?}", &header_buf[0..4]);
        return Err("Invalid data disk header (no TWAV magic)");
    }
    
    let version = u32::from_le_bytes([header_buf[4], header_buf[5], header_buf[6], header_buf[7]]);
    if version != 1 {
        return Err("Unsupported data disk version");
    }
    
    let wav_size = u64::from_le_bytes([
        header_buf[8], header_buf[9], header_buf[10], header_buf[11],
        header_buf[12], header_buf[13], header_buf[14], header_buf[15],
    ]) as usize;
    
    let wav_start_lba = u32::from_le_bytes([
        header_buf[16], header_buf[17], header_buf[18], header_buf[19],
    ]) as u64;
    
    let wav_sector_count = u32::from_le_bytes([
        header_buf[20], header_buf[21], header_buf[22], header_buf[23],
    ]) as usize;
    
    let expected_crc = u32::from_le_bytes([
        header_buf[24], header_buf[25], header_buf[26], header_buf[27],
    ]);
    
    crate::serial_println!("[DISK-AUDIO] WAV: {} bytes ({:.2} MB), {} sectors, start LBA {}, CRC32={:#010x}",
        wav_size, wav_size as f64 / 1024.0 / 1024.0, wav_sector_count, wav_start_lba, expected_crc);
    
    if wav_size == 0 || wav_size > 60 * 1024 * 1024 {
        return Err("Invalid WAV size in header");
    }
    
    // Allocate buffer for entire WAV
    let total_read_bytes = wav_sector_count * SECTOR_SIZE;
    let mut wav_buf = vec![0u8; total_read_bytes];
    
    // Read in chunks of MAX_SECTORS_PER_READ
    let mut sectors_remaining = wav_sector_count;
    let mut current_lba = wav_start_lba;
    let mut buf_offset = 0;
    
    while sectors_remaining > 0 {
        let chunk = (sectors_remaining as u16).min(MAX_SECTORS_PER_READ);
        let chunk_bytes = chunk as usize * SECTOR_SIZE;
        
        crate::drivers::ahci::read_sectors(
            port,
            current_lba,
            chunk,
            &mut wav_buf[buf_offset..buf_offset + chunk_bytes],
        )?;
        
        current_lba += chunk as u64;
        buf_offset += chunk_bytes;
        sectors_remaining -= chunk as usize;
    }
    
    // Trim to actual WAV size
    wav_buf.truncate(wav_size);
    
    crate::serial_println!("[DISK-AUDIO] Loaded {} bytes from disk", wav_buf.len());
    
    Ok(wav_buf)
}
