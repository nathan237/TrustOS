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
use core::sync::atomic::{AtomicU8, Ordering};

// Compile-time constant — evaluated at compilation, zero runtime cost.
const SECTOR_SIZE: usize = 512;
/// Read 128 sectors at a time (64KB) for fast bulk loading.
/// AHCI supports up to 128 sectors per command; larger batches = fewer commands.
const DMA_READ_SECTORS: u16 = 128;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DMA_READ_BYTES: usize = DMA_READ_SECTORS as usize * SECTOR_SIZE; // 65536
/// Maximum tracks in v2 format
const MAXIMUM_TRACKS: usize = 10;
/// Bytes per track entry in header
const ENTRY_SIZE: usize = 48;

/// Track info parsed from the disk header
#[derive(Clone)]
// Public structure — visible outside this module.
pub struct TrackInformation {
    pub wav_size: usize,
    pub start_lba: u64,
    pub sector_count: usize,
    pub crc32: u32,
    pub name: String,
}

/// Disk track table (cached after first read)
pub struct DiskTrackTable {
    pub tracks: Vec<TrackInformation>,
}

/// Cached AHCI port number for TWAV disk (0xFE = not scanned yet, 0xFF = not found)
static CACHED_PORT: AtomicU8 = AtomicU8::new(0xFE);

/// Try to find which AHCI port has our audio data disk.
/// Result is cached after first scan to avoid repeated port probing.
fn find_data_port() -> Option<u8> {
    let cached = CACHED_PORT.load(Ordering::Relaxed);
    if cached != 0xFE {
        return if cached == 0xFF { None } else { Some(cached) };
    }
    if !crate::drivers::ahci::is_initialized() {
        return None;
    }
    let devices = crate::drivers::ahci::list_devices();
    for dev in &devices {
        if dev.device_type == crate::drivers::ahci::AhciDeviceType::Sata && dev.sector_count > 64 {
            let mut probe = alloc::vec![0u8; 512];
            if crate::drivers::ahci::read_sectors(dev.port_num, 0, 1, &mut probe).is_ok() {
                if probe.len() >= 4 && &probe[0..4] == b"TWAV" {
                    crate::serial_println!("[DISK-AUDIO] Found TWAV disk on port {}", dev.port_num);
                    CACHED_PORT.store(dev.port_num, Ordering::Relaxed);
                    return Some(dev.port_num);
                }
            }
        }
    }
    CACHED_PORT.store(0xFF, Ordering::Relaxed);
    None
}

/// Read the track table from the audio disk.
pub fn read_track_table() -> Result<DiskTrackTable, &'static str> {
    let port = find_data_port().ok_or("No data disk found on AHCI")?;

    crate::serial_println!("[DISK-AUDIO] Reading header from port {}...", port);

    let mut header_buffer = vec![0u8; SECTOR_SIZE];
    crate::drivers::ahci::read_sectors(port, 0, 1, &mut header_buffer)?;

    if &header_buffer[0..4] != b"TWAV" {
        return Err("Invalid data disk header (no TWAV magic)");
    }

    let version = u32::from_le_bytes([header_buffer[4], header_buffer[5], header_buffer[6], header_buffer[7]]);

        // Pattern matching — Rust's exhaustive branching construct.
match version {
        1 => {
            // Legacy single-track format
            let wav_size = u64::from_le_bytes([
                header_buffer[8], header_buffer[9], header_buffer[10], header_buffer[11],
                header_buffer[12], header_buffer[13], header_buffer[14], header_buffer[15],
            ]) as usize;
            let start_lba = u32::from_le_bytes([
                header_buffer[16], header_buffer[17], header_buffer[18], header_buffer[19],
            ]) as u64;
            let sector_count = u32::from_le_bytes([
                header_buffer[20], header_buffer[21], header_buffer[22], header_buffer[23],
            ]) as usize;
            let crc32 = u32::from_le_bytes([
                header_buffer[24], header_buffer[25], header_buffer[26], header_buffer[27],
            ]);
            crate::serial_println!("[DISK-AUDIO] v1: 1 track, {} bytes", wav_size);
            Ok(DiskTrackTable {
                tracks: alloc::vec![TrackInformation {
                    wav_size, start_lba, sector_count, crc32,
                    name: String::from("Untitled (2)"),
                }],
            })
        }
        2 => {
            let num_tracks = u32::from_le_bytes([
                header_buffer[8], header_buffer[9], header_buffer[10], header_buffer[11],
            ]) as usize;
            let num_tracks = num_tracks.min(MAXIMUM_TRACKS);
            let mut tracks = Vec::with_capacity(num_tracks);
            for i in 0..num_tracks {
                let off = 16 + i * ENTRY_SIZE;
                if off + ENTRY_SIZE > SECTOR_SIZE { break; }
                let wav_size = u64::from_le_bytes([
                    header_buffer[off], header_buffer[off+1], header_buffer[off+2], header_buffer[off+3],
                    header_buffer[off+4], header_buffer[off+5], header_buffer[off+6], header_buffer[off+7],
                ]) as usize;
                let start_lba = u32::from_le_bytes([
                    header_buffer[off+8], header_buffer[off+9], header_buffer[off+10], header_buffer[off+11],
                ]) as u64;
                let sector_count = u32::from_le_bytes([
                    header_buffer[off+12], header_buffer[off+13], header_buffer[off+14], header_buffer[off+15],
                ]) as usize;
                let crc32 = u32::from_le_bytes([
                    header_buffer[off+16], header_buffer[off+17], header_buffer[off+18], header_buffer[off+19],
                ]);
                // Name: 28 bytes null-padded UTF-8
                let name_bytes = &header_buffer[off+20..off+48];
                let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(28);
                let name = core::str::from_utf8(&name_bytes[..name_len])
                    .unwrap_or("Unknown")
                    .into();
                crate::serial_println!("[DISK-AUDIO] Track {}: '{}' {} bytes, LBA {}", i, name, wav_size, start_lba);
                tracks.push(TrackInformation { wav_size, start_lba, sector_count, crc32, name });
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
pub fn load_track_from_disk(track_index: usize) -> Result<(Vec<u8>, String), &'static str> {
    let table = read_track_table()?;
    if track_index >= table.tracks.len() {
        return Err("Track index out of range");
    }
    let track = &table.tracks[track_index];
    load_track_data(track)
}

/// Load track data given a TrackInfo.
/// Uses a heap-allocated page-aligned DMA buffer so AHCI virt_to_phys
/// (which subtracts HHDM offset) maps correctly.  Static .bss buffers
/// live at the kernel's linked VA, outside the HHDM window, so DMA
/// writes to the wrong physical address and we read back zeros.
fn load_track_data(track: &TrackInformation) -> Result<(Vec<u8>, String), &'static str> {
    let port = find_data_port().ok_or("No data disk found on AHCI")?;

    if track.wav_size == 0 || track.wav_size > 60 * 1024 * 1024 {
        return Err("Invalid WAV size in header");
    }

    crate::serial_println!("[DISK-AUDIO] Loading '{}': {} bytes, {} sectors from LBA {}",
        track.name, track.wav_size, track.sector_count, track.start_lba);

    // Allocate DMA buffer FIRST (small, 64KB) before the large WAV buffer
    // so the allocator can service it from a clean free-list.
    let mut dma_buf = vec![0u8; DMA_READ_BYTES];

    let mut wav_buffer = Vec::with_capacity(track.wav_size);

    let mut sectors_remaining = track.sector_count;
    let mut current_lba = track.start_lba;

    while sectors_remaining > 0 {
        // Compute chunk in usize FIRST, then cast to u16.
        // Avoids truncation: e.g. 102496_usize as u16 = 37120, but
        // 102496_usize.min(128) = 128 then as u16 = 128.
        let chunk_sectors = sectors_remaining.min(DMA_READ_SECTORS as usize);
        let chunk = chunk_sectors as u16;
        let chunk_bytes = chunk_sectors * SECTOR_SIZE;

        crate::drivers::ahci::read_sectors(
            port, current_lba, chunk,
            &mut dma_buf[..chunk_bytes],
        )?;

        // Copy from DMA buffer to target Vec (only up to what we still need)
        let needed = track.wav_size.saturating_sub(wav_buffer.len());
        let to_copy = chunk_bytes.min(needed);
        if to_copy > 0 {
            wav_buffer.extend_from_slice(&dma_buf[..to_copy]);
        }

        current_lba += chunk_sectors as u64;
        sectors_remaining -= chunk_sectors;
    }

    wav_buffer.truncate(track.wav_size);
    let name = track.name.clone();
    crate::serial_println!("[DISK-AUDIO] Loaded '{}': {} bytes", name, wav_buffer.len());
    Ok((wav_buffer, name))
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

/// Get track names for the UI (reads header only, fast).
pub fn get_track_names() -> Vec<String> {
        // Pattern matching — Rust's exhaustive branching construct.
match read_track_table() {
        Ok(table) => table.tracks.iter().map(|t| t.name.clone()).collect(),
        Err(_) => Vec::new(),
    }
}
