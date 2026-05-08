














use spin::Mutex;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};


pub const H_: usize = 512;


pub const BFW_: u64 = 256;
pub const BFX_: usize = (BFW_ as usize) * H_;


pub const DXE_: u8 = 128;


#[derive(Debug, Clone)]
pub struct Jt {
    pub model: String,
    pub serial: String,
    pub sectors: u64,
    pub size_mb: u64,
    pub present: bool,
}


struct Adj {
    data: Vec<u8>,
    info: Jt,
}


static Hn: Mutex<Option<Adj>> = Mutex::new(None);


static AIT_: AtomicU64 = AtomicU64::new(0);
static ALR_: AtomicU64 = AtomicU64::new(0);
static KF_: AtomicU64 = AtomicU64::new(0);
static KG_: AtomicU64 = AtomicU64::new(0);


pub fn init() {
    let data = vec![0u8; BFX_];
    
    let info = Jt {
        model: String::from("T-RustOS RAMDisk"),
        serial: String::from("RAMD-0001"),
        sectors: BFW_,
        size_mb: (BFX_ / 1024 / 1024) as u64,
        present: true,
    };
    
    crate::log!("[DISK] RAMDisk initialized: {} MB ({} sectors)", 
        info.size_mb, info.sectors);
    
    *Hn.lock() = Some(Adj { data, info });
    
    
    igr();
}


fn igr() {
    
    let mut disk = Hn.lock();
    if let Some(ref mut amo) = *disk {
        
        amo.data[0] = b'T';
        amo.data[1] = b'R';
        amo.data[2] = b'U';
        amo.data[3] = b'S';
        amo.data[4] = b'T';
        amo.data[5] = b'F';
        amo.data[6] = b'S';  
        amo.data[7] = 0x01; 
        
        
        let omt = H_;
        amo.data[omt] = 0; 
        
        crate::log!("[DISK] Filesystem initialized (TRUSTFS v1)");
    }
}


pub fn sw() -> bool {
    Hn.lock().is_some()
}


pub fn rk() -> Option<Jt> {
    Hn.lock().as_ref().map(|d| d.info.clone())
}


pub fn get_stats() -> (u64, u64, u64, u64) {
    (
        AIT_.load(Ordering::Relaxed),
        ALR_.load(Ordering::Relaxed),
        KF_.load(Ordering::Relaxed),
        KG_.load(Ordering::Relaxed),
    )
}


pub fn read_sectors(hb: u64, count: u8, buffer: &mut [u8]) -> Result<usize, &'static str> {
    if count == 0 {
        return Err("Invalid sector count");
    }
    
    let aim = count as usize * H_;
    if buffer.len() < aim {
        return Err("Buffer too small");
    }
    
    let disk = Hn.lock();
    let amo = disk.as_ref().ok_or("Disk not initialized")?;
    
    let azv = (hb as usize) * H_;
    let bzl = azv + aim;
    
    if bzl > amo.data.len() {
        return Err("Read past end of disk");
    }
    
    buffer[..aim].copy_from_slice(&amo.data[azv..bzl]);
    
    AIT_.fetch_add(count as u64, Ordering::Relaxed);
    KF_.fetch_add(aim as u64, Ordering::Relaxed);
    
    Ok(aim)
}


pub fn write_sectors(hb: u64, count: u8, buffer: &[u8]) -> Result<usize, &'static str> {
    if count == 0 {
        return Err("Invalid sector count");
    }
    
    let aim = count as usize * H_;
    if buffer.len() < aim {
        return Err("Buffer too small");
    }
    
    let mut disk = Hn.lock();
    let amo = disk.as_mut().ok_or("Disk not initialized")?;
    
    let azv = (hb as usize) * H_;
    let bzl = azv + aim;
    
    if bzl > amo.data.len() {
        return Err("Write past end of disk");
    }
    
    amo.data[azv..bzl].copy_from_slice(&buffer[..aim]);
    
    ALR_.fetch_add(count as u64, Ordering::Relaxed);
    KG_.fetch_add(aim as u64, Ordering::Relaxed);
    
    Ok(aim)
}


pub fn read_sector(hb: u64) -> Result<[u8; H_], &'static str> {
    let mut buffer = [0u8; H_];
    read_sectors(hb, 1, &mut buffer)?;
    Ok(buffer)
}


pub fn write_sector(hb: u64, data: &[u8; H_]) -> Result<(), &'static str> {
    write_sectors(hb, 1, data)?;
    Ok(())
}


pub fn format() -> Result<(), &'static str> {
    {
        let mut disk = Hn.lock();
        let amo = disk.as_mut().ok_or("Disk not initialized")?;
        
        
        for byte in amo.data.iter_mut() {
            *byte = 0;
        }
    }
    
    
    igr();
    
    
    AIT_.store(0, Ordering::Relaxed);
    ALR_.store(0, Ordering::Relaxed);
    KF_.store(0, Ordering::Relaxed);
    KG_.store(0, Ordering::Relaxed);
    
    crate::log!("[DISK] Disk formatted");
    Ok(())
}


pub fn lml(hb: u64) -> Result<String, &'static str> {
    let dj = read_sector(hb)?;
    
    use alloc::format;
    let mut result = format!("Sector {} (LBA):\n", hb);
    
    
    for row in 0..4 {
        let offset = row * 16;
        result.push_str(&format!("{:04X}: ", offset));
        
        for i in 0..16 {
            result.push_str(&format!("{:02X} ", dj[offset + i]));
        }
        
        result.push_str(" |");
        for i in 0..16 {
            let c = dj[offset + i];
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






#[derive(Debug, Clone)]
pub struct Oz {
    pub name: String,
    pub size: u32,
    pub start_sector: u32,
    pub is_directory: bool,
}

const HJ_: usize = 32;
const BBP_: usize = 10;
const NS_: u64 = 1;
const BD_: u64 = 10;


pub fn etb() -> Result<Vec<Oz>, &'static str> {
    let dj = read_sector(NS_)?;
    let mut files = Vec::new();
    
    let adp = dj[0] as usize;
    if adp > BBP_ {
        return Err("Corrupted directory");
    }
    
    let oi = 1 + HJ_ + 4 + 4 + 1; 
    
    for i in 0..adp {
        let offset = 1 + i * oi;
        
        if offset + oi > H_ {
            break;
        }
        
        if dj[offset] == 0 {
            continue; 
        }
        
        
        let sj = offset + 1;
        let mut aec = sj;
        while aec < sj + HJ_ && dj[aec] != 0 {
            aec += 1;
        }
        let name = String::from_utf8_lossy(&dj[sj..aec]).into_owned();
        
        let avc = offset + 1 + HJ_;
        let size = u32::from_le_bytes([
            dj[avc],
            dj[avc + 1],
            dj[avc + 2],
            dj[avc + 3],
        ]);
        
        let avb = avc + 4;
        let start_sector = u32::from_le_bytes([
            dj[avb],
            dj[avb + 1],
            dj[avb + 2],
            dj[avb + 3],
        ]);
        
        let is_directory = dj[avb + 4] != 0;
        
        files.push(Oz {
            name,
            size,
            start_sector,
            is_directory,
        });
    }
    
    Ok(files)
}


pub fn qbq(name: &str, data: &[u8]) -> Result<(), &'static str> {
    if name.len() > HJ_ - 1 {
        return Err("Filename too long");
    }
    
    
    let mut aqh = read_sector(NS_)?;
    let adp = aqh[0] as usize;
    
    if adp >= BBP_ {
        return Err("Directory full");
    }
    
    
    let bdq = (data.len() + H_ - 1) / H_;
    if bdq > 255 {
        return Err("File too large");
    }
    
    let start_sector = BD_ + (adp as u64 * 256); 
    
    
    let mut ck = data;
    let mut hpw = start_sector;
    
    while !ck.is_empty() {
        let mut jea = [0u8; H_];
        let rs = ck.len().min(H_);
        jea[..rs].copy_from_slice(&ck[..rs]);
        
        write_sector(hpw, &jea)?;
        
        ck = &ck[rs..];
        hpw += 1;
    }
    
    
    let oi = 1 + HJ_ + 4 + 4 + 1;
    let entry_offset = 1 + adp * oi;
    
    aqh[entry_offset] = 1; 
    
    
    for (i, byte) in name.bytes().enumerate() {
        aqh[entry_offset + 1 + i] = byte;
    }
    
    
    let size_bytes = (data.len() as u32).to_le_bytes();
    let avc = entry_offset + 1 + HJ_;
    aqh[avc..avc + 4].copy_from_slice(&size_bytes);
    
    
    let omu = (start_sector as u32).to_le_bytes();
    let avb = avc + 4;
    aqh[avb..avb + 4].copy_from_slice(&omu);
    
    
    aqh[avb + 4] = 0;
    
    
    aqh[0] = (adp + 1) as u8;
    
    write_sector(NS_, &aqh)?;
    
    crate::log!("[DISK] Created file: {} ({} bytes)", name, data.len());
    
    Ok(())
}


pub fn read_file(name: &str) -> Result<Vec<u8>, &'static str> {
    let files = etb()?;
    
    let file = files.iter().find(|f| f.name == name)
        .ok_or("File not found")?;
    
    let bdq = (file.size as usize + H_ - 1) / H_;
    let bdq = bdq.max(1);
    let mut data = Vec::with_capacity(file.size as usize);
    
    for i in 0..bdq {
        let dj = read_sector(file.start_sector as u64 + i as u64)?;
        let ck = file.size as usize - data.len();
        let rs = ck.min(H_);
        data.extend_from_slice(&dj[..rs]);
    }
    
    Ok(data)
}


pub fn qcn(name: &str) -> Result<(), &'static str> {
    let mut aqh = read_sector(NS_)?;
    let adp = aqh[0] as usize;
    
    let oi = 1 + HJ_ + 4 + 4 + 1;
    
    for i in 0..adp {
        let offset = 1 + i * oi;
        
        if aqh[offset] == 0 {
            continue;
        }
        
        
        let sj = offset + 1;
        let mut aec = sj;
        while aec < sj + HJ_ && aqh[aec] != 0 {
            aec += 1;
        }
        let bbl = String::from_utf8_lossy(&aqh[sj..aec]);
        
        if bbl == name {
            
            aqh[offset] = 0;
            write_sector(NS_, &aqh)?;
            crate::log!("[DISK] Deleted file: {}", name);
            return Ok(());
        }
    }
    
    Err("File not found")
}
