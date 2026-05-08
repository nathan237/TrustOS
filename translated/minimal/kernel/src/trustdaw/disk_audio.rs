








use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use core::sync::atomic::{AtomicU8, Ordering};

const H_: usize = 512;


const ASM_: u16 = 128;
const BUT_: usize = ASM_ as usize * H_; 

const GB_: usize = 10;

const ATJ_: usize = 48;


#[derive(Clone)]
pub struct Qo {
    pub wav_size: usize,
    pub start_lba: u64,
    pub sector_count: usize,
    pub cvv: u32,
    pub name: String,
}


pub struct Rt {
    pub tracks: Vec<Qo>,
}


static ABF_: AtomicU8 = AtomicU8::new(0xFE);



fn hyv() -> Option<u8> {
    let bfd = ABF_.load(Ordering::Relaxed);
    if bfd != 0xFE {
        return if bfd == 0xFF { None } else { Some(bfd) };
    }
    if !crate::drivers::ahci::is_initialized() {
        return None;
    }
    let devices = crate::drivers::ahci::adz();
    for s in &devices {
        if s.device_type == crate::drivers::ahci::AhciDeviceType::Sata && s.sector_count > 64 {
            let mut probe = alloc::vec![0u8; 512];
            if crate::drivers::ahci::read_sectors(s.port_num, 0, 1, &mut probe).is_ok() {
                if probe.len() >= 4 && &probe[0..4] == b"TWAV" {
                    crate::serial_println!("[DISK-AUDIO] Found TWAV disk on port {}", s.port_num);
                    ABF_.store(s.port_num, Ordering::Relaxed);
                    return Some(s.port_num);
                }
            }
        }
    }
    ABF_.store(0xFF, Ordering::Relaxed);
    None
}


pub fn exz() -> Result<Rt, &'static str> {
    let port = hyv().ok_or("No data disk found on AHCI")?;

    crate::serial_println!("[DISK-AUDIO] Reading header from port {}...", port);

    let mut jz = vec![0u8; H_];
    crate::drivers::ahci::read_sectors(port, 0, 1, &mut jz)?;

    if &jz[0..4] != b"TWAV" {
        return Err("Invalid data disk header (no TWAV magic)");
    }

    let version = u32::from_le_bytes([jz[4], jz[5], jz[6], jz[7]]);

    match version {
        1 => {
            
            let wav_size = u64::from_le_bytes([
                jz[8], jz[9], jz[10], jz[11],
                jz[12], jz[13], jz[14], jz[15],
            ]) as usize;
            let start_lba = u32::from_le_bytes([
                jz[16], jz[17], jz[18], jz[19],
            ]) as u64;
            let sector_count = u32::from_le_bytes([
                jz[20], jz[21], jz[22], jz[23],
            ]) as usize;
            let cvv = u32::from_le_bytes([
                jz[24], jz[25], jz[26], jz[27],
            ]);
            crate::serial_println!("[DISK-AUDIO] v1: 1 track, {} bytes", wav_size);
            Ok(Rt {
                tracks: alloc::vec![Qo {
                    wav_size, start_lba, sector_count, cvv,
                    name: String::from("Untitled (2)"),
                }],
            })
        }
        2 => {
            let num_tracks = u32::from_le_bytes([
                jz[8], jz[9], jz[10], jz[11],
            ]) as usize;
            let num_tracks = num_tracks.min(GB_);
            let mut tracks = Vec::with_capacity(num_tracks);
            for i in 0..num_tracks {
                let off = 16 + i * ATJ_;
                if off + ATJ_ > H_ { break; }
                let wav_size = u64::from_le_bytes([
                    jz[off], jz[off+1], jz[off+2], jz[off+3],
                    jz[off+4], jz[off+5], jz[off+6], jz[off+7],
                ]) as usize;
                let start_lba = u32::from_le_bytes([
                    jz[off+8], jz[off+9], jz[off+10], jz[off+11],
                ]) as u64;
                let sector_count = u32::from_le_bytes([
                    jz[off+12], jz[off+13], jz[off+14], jz[off+15],
                ]) as usize;
                let cvv = u32::from_le_bytes([
                    jz[off+16], jz[off+17], jz[off+18], jz[off+19],
                ]);
                
                let agt = &jz[off+20..off+48];
                let name_len = agt.iter().position(|&b| b == 0).unwrap_or(28);
                let name = core::str::from_utf8(&agt[..name_len])
                    .unwrap_or("Unknown")
                    .into();
                crate::serial_println!("[DISK-AUDIO] Track {}: '{}' {} bytes, LBA {}", i, name, wav_size, start_lba);
                tracks.push(Qo { wav_size, start_lba, sector_count, cvv, name });
            }
            if tracks.is_empty() {
                return Err("No tracks in v2 header");
            }
            Ok(Rt { tracks })
        }
        _ => Err("Unsupported data disk version"),
    }
}


pub fn etf(mp: usize) -> Result<(Vec<u8>, String), &'static str> {
    let bs = exz()?;
    if mp >= bs.tracks.len() {
        return Err("Track index out of range");
    }
    let track = &bs.tracks[mp];
    nah(track)
}






fn nah(track: &Qo) -> Result<(Vec<u8>, String), &'static str> {
    let port = hyv().ok_or("No data disk found on AHCI")?;

    if track.wav_size == 0 || track.wav_size > 60 * 1024 * 1024 {
        return Err("Invalid WAV size in header");
    }

    crate::serial_println!("[DISK-AUDIO] Loading '{}': {} bytes, {} sectors from LBA {}",
        track.name, track.wav_size, track.sector_count, track.start_lba);

    
    
    let mut dma_buf = vec![0u8; BUT_];

    let mut eed = Vec::with_capacity(track.wav_size);

    let mut gtk = track.sector_count;
    let mut fpr = track.start_lba;

    while gtk > 0 {
        
        
        
        let ehy = gtk.min(ASM_ as usize);
        let df = ehy as u16;
        let blb = ehy * H_;

        crate::drivers::ahci::read_sectors(
            port, fpr, df,
            &mut dma_buf[..blb],
        )?;

        
        let giu = track.wav_size.saturating_sub(eed.len());
        let od = blb.min(giu);
        if od > 0 {
            eed.extend_from_slice(&dma_buf[..od]);
        }

        fpr += ehy as u64;
        gtk -= ehy;
    }

    eed.truncate(track.wav_size);
    let name = track.name.clone();
    crate::serial_println!("[DISK-AUDIO] Loaded '{}': {} bytes", name, eed.len());
    Ok((eed, name))
}


pub fn pmt() -> usize {
    exz().map(|t| t.tracks.len()).unwrap_or(0)
}


pub fn nai() -> Result<Vec<u8>, &'static str> {
    let (data, _name) = etf(0)?;
    Ok(data)
}


pub fn mdz() -> Vec<String> {
    match exz() {
        Ok(bs) => bs.tracks.iter().map(|t| t.name.clone()).collect(),
        Err(_) => Vec::new(),
    }
}
