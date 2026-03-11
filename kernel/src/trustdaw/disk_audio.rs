//! Load audio from the AHCI data disk at runtime.
//!
//! Disk layout (prepared by `prepare-data-disk.py`):
//!   Sector 0       : Header — magic "TWAV", version, track table
//!   Sector 1..N    : Track data (raw WAV bytes, contiguous)
//!
//! Version 1: single track (legacy)
//! Version 2: multi-track (up to 10 tracks)

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

const SECTOR_SIZE: usize = 512;
/// Read 8 sectors at a time (4KB = one page, DMA-safe).
/// Using a small page-aligned intermediate buffer avoids issues with large heap
/// allocations whose physical addresses may not be contiguous across pages.
const DMA_READ_SECTORS: u16 = 8;
const DMA_READ_BYTES: usize = DMA_READ_SECTORS as usize * SECTOR_SIZE; // 4096
/// Maximum tracks in v2 format
const MAX_TRACKS: usize = 10;
/// Bytes per track entry in header
const ENTRY_SIZE: usize = 48;

/// Page-aligned intermediate buffer for AHCI DMA reads.
/// AHCI DMA requires a physically contiguous buffer. The kernel heap may
/// not guarantee physical contiguity for large allocations.
/// This small buffer is always within a single 4KB page.
#[repr(C, align(4096))]
struct DmaReadBuf([u8; DMA_READ_BYTES]);

static mut DMA_READ_BUF: DmaReadBuf = DmaReadBuf([0u8; DMA_READ_BYTES]);

/// Track info parsed from the disk header
#[derive(Clone)]
pub struct TrackInfo {
    pub wav_size: usize,
    pub start_lba: u64,
    pub sector_count: usize,
    pub crc32: u32,
    pub name: String,
}

/// Disk track table (cached after first read)
pub struct DiskTrackTable {
    pub tracks: Vec<TrackInfo>,
}

/// Try to find which AHCI port has our audio data disk.
fn find_data_port() -> Option<u8> {
    if !crate::drivers::ahci::is_initialized() {
        return None;
    }
    let devices = crate::drivers::ahci::list_devices();
    for dev in &devices {
        if dev.port_num == 2 {
            return Some(2);
        }
    }
    for dev in &devices {
        if dev.port_num > 1 && dev.device_type == crate::drivers::ahci::AhciDeviceType::Sata {
            return Some(dev.port_num);
        }
    }
    None
}

/// Read the track table from the audio disk.
pub fn read_track_table() -> Result<DiskTrackTable, &'static str> {
    let port = find_data_port().ok_or("No data disk found on AHCI")?;

    crate::serial_println!("[DISK-AUDIO] Reading header from port {}...", port);

    let mut header_buf = vec![0u8; SECTOR_SIZE];
    crate::drivers::ahci::read_sectors(port, 0, 1, &mut header_buf)?;

    if &header_buf[0..4] != b"TWAV" {
        return Err("Invalid data disk header (no TWAV magic)");
    }

    let version = u32::from_le_bytes([header_buf[4], header_buf[5], header_buf[6], header_buf[7]]);

    match version {
        1 => {
            // Legacy single-track format
            let wav_size = u64::from_le_bytes([
                header_buf[8], header_buf[9], header_buf[10], header_buf[11],
                header_buf[12], header_buf[13], header_buf[14], header_buf[15],
            ]) as usize;
            let start_lba = u32::from_le_bytes([
                header_buf[16], header_buf[17], header_buf[18], header_buf[19],
            ]) as u64;
            let sector_count = u32::from_le_bytes([
                header_buf[20], header_buf[21], header_buf[22], header_buf[23],
            ]) as usize;
            let crc32 = u32::from_le_bytes([
                header_buf[24], header_buf[25], header_buf[26], header_buf[27],
            ]);
            crate::serial_println!("[DISK-AUDIO] v1: 1 track, {} bytes", wav_size);
            Ok(DiskTrackTable {
                tracks: alloc::vec![TrackInfo {
                    wav_size, start_lba, sector_count, crc32,
                    name: String::from("Untitled (2)"),
                }],
            })
        }
        2 => {
            let num_tracks = u32::from_le_bytes([
                header_buf[8], header_buf[9], header_buf[10], header_buf[11],
            ]) as usize;
            let num_tracks = num_tracks.min(MAX_TRACKS);
            let mut tracks = Vec::with_capacity(num_tracks);
            for i in 0..num_tracks {
                let off = 16 + i * ENTRY_SIZE;
                if off + ENTRY_SIZE > SECTOR_SIZE { break; }
                let wav_size = u64::from_le_bytes([
                    header_buf[off], header_buf[off+1], header_buf[off+2], header_buf[off+3],
                    header_buf[off+4], header_buf[off+5], header_buf[off+6], header_buf[off+7],
                ]) as usize;
                let start_lba = u32::from_le_bytes([
                    header_buf[off+8], header_buf[off+9], header_buf[off+10], header_buf[off+11],
                ]) as u64;
                let sector_count = u32::from_le_bytes([
                    header_buf[off+12], header_buf[off+13], header_buf[off+14], header_buf[off+15],
                ]) as usize;
                let crc32 = u32::from_le_bytes([
                    header_buf[off+16], header_buf[off+17], header_buf[off+18], header_buf[off+19],
                ]);
                // Name: 28 bytes null-padded UTF-8
                let name_bytes = &header_buf[off+20..off+48];
                let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(28);
                let name = core::str::from_utf8(&name_bytes[..name_len])
                    .unwrap_or("Unknown")
                    .into();
                crate::serial_println!("[DISK-AUDIO] Track {}: '{}' {} bytes, LBA {}", i, name, wav_size, start_lba);
                tracks.push(TrackInfo { wav_size, start_lba, sector_count, crc32, name });
            }
            if tracks.is_empty() {
                return Err("No tracks in v2 header");
            }
            Ok(DiskTrackTable { tracks })
        }
        _ => Err("Unsupported data disk version"),
    }
}

/// Load a specific track by index from the audio disk.
pub fn load_track_from_disk(track_idx: usize) -> Result<(Vec<u8>, String), &'static str> {
    let table = read_track_table()?;
    if track_idx >= table.tracks.len() {
        return Err("Track index out of range");
    }
    let track = &table.tracks[track_idx];
    load_track_data(track)
}

/// Load track data given a TrackInfo.
/// Uses a small page-aligned DMA buffer to avoid physical address issues
/// with large heap allocations.
fn load_track_data(track: &TrackInfo) -> Result<(Vec<u8>, String), &'static str> {
    let port = find_data_port().ok_or("No data disk found on AHCI")?;

    if track.wav_size == 0 || track.wav_size > 60 * 1024 * 1024 {
        return Err("Invalid WAV size in header");
    }

    crate::serial_println!("[DISK-AUDIO] Loading '{}': {} bytes, {} sectors from LBA {}",
        track.name, track.wav_size, track.sector_count, track.start_lba);

    let mut wav_buf = Vec::with_capacity(track.wav_size);

    let mut sectors_remaining = track.sector_count;
    let mut current_lba = track.start_lba;

    while sectors_remaining > 0 {
        let chunk = (sectors_remaining as u16).min(DMA_READ_SECTORS);
        let chunk_bytes = chunk as usize * SECTOR_SIZE;

        // Read into page-aligned DMA-safe intermediate buffer
        let bytes_read = unsafe {
            crate::drivers::ahci::read_sectors(
                port, current_lba, chunk,
                &mut DMA_READ_BUF.0[..chunk_bytes],
            )?
        };

        // Copy from DMA buffer to target Vec (only up to what we still need)
        let needed = track.wav_size.saturating_sub(wav_buf.len());
        let to_copy = bytes_read.min(needed);
        if to_copy > 0 {
            unsafe {
                wav_buf.extend_from_slice(&DMA_READ_BUF.0[..to_copy]);
            }
        }

        current_lba += chunk as u64;
        sectors_remaining -= chunk as usize;
    }

    wav_buf.truncate(track.wav_size);
    let name = track.name.clone();
    crate::serial_println!("[DISK-AUDIO] Loaded '{}': {} bytes", name, wav_buf.len());
    Ok((wav_buf, name))
}

/// Get the number of tracks on the audio disk (0 if no disk).
pub fn track_count() -> usize {
    read_track_table().map(|t| t.tracks.len()).unwrap_or(0)
}

/// Legacy compatibility: load first track as before.
pub fn load_wav_from_disk() -> Result<Vec<u8>, &'static str> {
    let (data, _name) = load_track_from_disk(0)?;
    Ok(data)
}
