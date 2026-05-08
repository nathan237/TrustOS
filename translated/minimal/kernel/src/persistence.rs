




use alloc::vec::Vec;
use alloc::string::String;
use alloc::vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};


const XL_: &[u8; 8] = b"TRUSTPST";


const BFB_: u32 = 1;


const GF_: u64 = 2048; 


const CMG_: u64 = 32768; 


const H_: usize = 512;


static AID_: AtomicBool = AtomicBool::new(false);


static BFA_: AtomicBool = AtomicBool::new(false);


static PT_: Mutex<Option<u8>> = Mutex::new(None);


#[repr(C, packed)]
struct Uj {
    magic: [u8; 8],      
    version: u32,         
    entry_count: u32,     
    total_size: u64,      
    checksum: u32,        
    _reserved: [u8; 484], 
}


#[repr(C, packed)]
struct Oz {
    dch: u16,        
    atl: u32,        
    _reserved: [u8; 2],   
    
}


pub fn init() {
    crate::serial_println!("[PERSIST] Initializing persistence system...");
    
    
    let port = lvq();
    if port.is_none() {
        crate::serial_println!("[PERSIST] No AHCI disk found, persistence disabled");
        return;
    }
    
    let port = port.unwrap();
    *PT_.lock() = Some(port);
    
    
    if hkr(port) {
        AID_.store(true, Ordering::Relaxed);
        crate::serial_println!("[PERSIST] Found existing persistence data on port {}", port);
    } else {
        crate::serial_println!("[PERSIST] No existing persistence data found");
    }
}


fn lvq() -> Option<u8> {
    
    let ports = crate::drivers::ahci::adz();
    for port in ports {
        
        if port.sector_count > 0 {
            
            let mut buffer = [0u8; 512];
            if crate::drivers::ahci::read_sectors(port.port_num, 0, 1, &mut buffer).is_ok() {
                return Some(port.port_num);
            }
        }
    }
    None
}


fn hkr(port: u8) -> bool {
    let mut buffer = [0u8; 512];
    
    if crate::drivers::ahci::read_sectors(port, GF_, 1, &mut buffer).is_err() {
        return false;
    }
    
    
    &buffer[0..8] == XL_
}


pub fn nyv() -> bool {
    if !AID_.load(Ordering::Relaxed) {
        return false;
    }
    
    crate::println!();
    crate::n!(0x00FFFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::n!(0x00FFFF, "║           Saved Data Detected                                ║");
    crate::n!(0x00FFFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    crate::println!("  Previously downloaded files were found on disk.");
    crate::println!("  Do you want to restore them? (Y/n)");
    crate::println!();
    crate::print!("  > ");
    
    
    let fa = ocj();
    
    let jal = fa != b'n' && fa != b'N';
    
    if jal {
        crate::println!("Yes");
        crate::println!();
        crate::n!(0x00FF00, "  Restoring saved data...");
        
        if let Err(e) = ogm() {
            crate::n!(0xFF0000, "  Error restoring: {}", e);
            return false;
        }
        
        BFA_.store(true, Ordering::Relaxed);
        crate::n!(0x00FF00, "  Data restored successfully!");
    } else {
        crate::println!("No");
        crate::println!("  Starting fresh.");
    }
    
    crate::println!();
    jal
}


fn ocj() -> u8 {
    loop {
        if let Some(c) = crate::keyboard::ya() {
            return c;
        }
        core::hint::spin_loop();
    }
}


pub fn save_file(path: &str, data: &[u8]) -> Result<(), &'static str> {
    let port = PT_.lock().ok_or("Persistence not available")?;
    
    crate::serial_println!("[PERSIST] Saving {} ({} bytes)", path, data.len());
    
    
    let mut jz = [0u8; 512];
    let mut header = if hkr(port) {
        crate::drivers::ahci::read_sectors(port, GF_, 1, &mut jz)
            .map_err(|_| "Failed to read header")?;
        cnu(&jz)?
    } else {
        
        Uj {
            magic: *XL_,
            version: BFB_,
            entry_count: 0,
            total_size: 0,
            checksum: 0,
            _reserved: [0; 484],
        }
    };
    
    
    let hqs = GF_ + 1 + (header.total_size + 511) / 512;
    
    
    if hqs + ((data.len() as u64 + path.len() as u64 + 16) / 512) + 1 
        > GF_ + CMG_ {
        return Err("Persistence storage full");
    }
    
    
    let aob = Oz {
        dch: path.len() as u16,
        atl: data.len() as u32,
        _reserved: [0; 2],
    };
    
    let mut bsh: Vec<u8> = Vec::new();
    
    
    let czd: [u8; 8] = unsafe { core::mem::transmute(aob) };
    bsh.extend_from_slice(&czd);
    
    
    bsh.extend_from_slice(path.as_bytes());
    
    
    bsh.extend_from_slice(data);
    
    
    while bsh.len() % 512 != 0 {
        bsh.push(0);
    }
    
    
    let bdq = bsh.len() / 512;
    for i in 0..bdq {
        let dj = hqs + i as u64;
        let offset = i * 512;
        let mut mx = [0u8; 512];
        mx.copy_from_slice(&bsh[offset..offset + 512]);
        
        crate::drivers::ahci::write_sectors(port, dj, 1, &mx)
            .map_err(|_| "Failed to write data sector")?;
    }
    
    
    header.entry_count += 1;
    header.total_size += bsh.len() as u64;
    header.checksum = kwk(&bsh);
    
    
    let mut caj = [0u8; 512];
    caj[0..8].copy_from_slice(&header.magic);
    caj[8..12].copy_from_slice(&header.version.to_le_bytes());
    caj[12..16].copy_from_slice(&header.entry_count.to_le_bytes());
    caj[16..24].copy_from_slice(&header.total_size.to_le_bytes());
    caj[24..28].copy_from_slice(&header.checksum.to_le_bytes());
    
    crate::drivers::ahci::write_sectors(port, GF_, 1, &caj)
        .map_err(|_| "Failed to write header")?;
    
    crate::serial_println!("[PERSIST] Saved {} successfully", path);
    Ok(())
}


fn ogm() -> Result<(), &'static str> {
    let port = *PT_.lock();
    let port = port.ok_or("Persistence not available")?;
    
    
    let mut jz = [0u8; 512];
    crate::drivers::ahci::read_sectors(port, GF_, 1, &mut jz)
        .map_err(|_| "Failed to read header")?;
    
    let header = cnu(&jz)?;
    
    let entry_count = header.entry_count;
    let total_size = header.total_size;
    
    
    if entry_count > 1000 {
        return Err("Corrupted: too many entries");
    }
    if total_size > 100 * 1024 * 1024 {
        return Err("Corrupted: size too large");
    }
    if total_size == 0 {
        return Ok(()); 
    }
    
    crate::serial_println!("[PERSIST] Restoring {} files ({} bytes)", 
        entry_count, total_size);
    
    
    let zp = (total_size + 511) / 512;
    let mut bai: Vec<u8> = vec![0u8; total_size as usize];
    
    for i in 0..zp {
        let dj = GF_ + 1 + i;
        let offset = (i as usize) * 512;
        let mut mx = [0u8; 512];
        
        crate::drivers::ahci::read_sectors(port, dj, 1, &mut mx)
            .map_err(|_| "Failed to read data sector")?;
        
        let mb = core::cmp::min(512, bai.len() - offset);
        bai[offset..offset + mb].copy_from_slice(&mx[..mb]);
    }
    
    
    let mut offset = 0;
    let mut grj = 0;
    
    while offset + 8 <= bai.len() && grj < header.entry_count {
        
        let dch = u16::from_le_bytes([bai[offset], bai[offset + 1]]) as usize;
        let atl = u32::from_le_bytes([
            bai[offset + 2], bai[offset + 3],
            bai[offset + 4], bai[offset + 5],
        ]) as usize;
        
        offset += 8;
        
        if offset + dch + atl > bai.len() {
            break;
        }
        
        
        let path = match core::str::from_utf8(&bai[offset..offset + dch]) {
            Ok(j) => j,
            Err(_) => {
                offset += dch + atl;
                continue;
            }
        };
        offset += dch;
        
        
        let data = &bai[offset..offset + atl];
        offset += atl;
        
        
        offset = (offset + 511) & !511;
        
        
        crate::serial_println!("[PERSIST] Restoring: {} ({} bytes)", path, atl);
        
        
        let result = crate::ramfs::bh(|fs| {
            
            let mut current = String::new();
            for jn in path.split('/').filter(|aa| !aa.is_empty()) {
                if !path.ends_with(jn) {
                    current.push('/');
                    current.push_str(jn);
                    let _ = fs.mkdir(&current);
                }
            }
            
            
            let _ = fs.touch(path);
            fs.write_file(path, data)
        });
        
        if result.is_ok() {
            grj += 1;
            crate::print!(".");
        }
    }
    
    crate::println!();
    crate::println!("  Restored {} files", grj);
    
    Ok(())
}


fn cnu(buf: &[u8; 512]) -> Result<Uj, &'static str> {
    if &buf[0..8] != XL_ {
        return Err("Invalid magic signature");
    }
    
    let version = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
    if version != BFB_ {
        return Err("Incompatible version");
    }
    
    Ok(Uj {
        magic: *XL_,
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


fn kwk(data: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    for byte in data {
        sum = sum.wrapping_add(*byte as u32);
    }
    sum
}


pub fn clear() -> Result<(), &'static str> {
    let port = *PT_.lock();
    let port = port.ok_or("Persistence not available")?;
    
    
    let caj = [0u8; 512];
    crate::drivers::ahci::write_sectors(port, GF_, 1, &caj)
        .map_err(|_| "Failed to clear persistence")?;
    
    crate::serial_println!("[PERSIST] Persistence data cleared");
    Ok(())
}


pub fn lq() -> bool {
    BFA_.load(Ordering::Relaxed)
}


pub fn sw() -> bool {
    AID_.load(Ordering::Relaxed)
}


pub fn status() -> (&'static str, u32, u64) {
    let port = *PT_.lock();
    
    if port.is_none() {
        return ("No disk", 0, 0);
    }
    
    let port = port.unwrap();
    let mut jz = [0u8; 512];
    
    if crate::drivers::ahci::read_sectors(port, GF_, 1, &mut jz).is_err() {
        return ("Read error", 0, 0);
    }
    
    if let Ok(header) = cnu(&jz) {
        ("Active", header.entry_count, header.total_size)
    } else {
        ("Empty", 0, 0)
    }
}
