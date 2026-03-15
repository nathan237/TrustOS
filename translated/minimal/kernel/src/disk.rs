














use spin::Mutex;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};


pub const H_: usize = 512;


pub const BDT_: u64 = 256;
pub const BDU_: usize = (BDT_ as usize) * H_;


pub const DTN_: u8 = 128;


#[derive(Debug, Clone)]
pub struct Wi {
    pub model: String,
    pub serial: String,
    pub grv: u64,
    pub aga: u64,
    pub brs: bool,
}


struct Bqj {
    f: Vec<u8>,
    co: Wi,
}


static Rz: Mutex<Option<Bqj>> = Mutex::new(None);


static AGZ_: AtomicU64 = AtomicU64::new(0);
static AJW_: AtomicU64 = AtomicU64::new(0);
static JO_: AtomicU64 = AtomicU64::new(0);
static JP_: AtomicU64 = AtomicU64::new(0);


pub fn init() {
    let f = vec![0u8; BDU_];
    
    let co = Wi {
        model: String::from("T-RustOS RAMDisk"),
        serial: String::from("RAMD-0001"),
        grv: BDT_,
        aga: (BDU_ / 1024 / 1024) as u64,
        brs: true,
    };
    
    crate::log!("[DISK] RAMDisk initialized: {} MB ({} sectors)", 
        co.aga, co.grv);
    
    *Rz.lock() = Some(Bqj { f, co });
    
    
    oek();
}


fn oek() {
    
    let mut disk = Rz.lock();
    if let Some(ref mut bwm) = *disk {
        
        bwm.f[0] = b'T';
        bwm.f[1] = b'R';
        bwm.f[2] = b'U';
        bwm.f[3] = b'S';
        bwm.f[4] = b'T';
        bwm.f[5] = b'F';
        bwm.f[6] = b'S';  
        bwm.f[7] = 0x01; 
        
        
        let wgb = H_;
        bwm.f[wgb] = 0; 
        
        crate::log!("[DISK] Filesystem initialized (TRUSTFS v1)");
    }
}


pub fn anl() -> bool {
    Rz.lock().is_some()
}


pub fn ani() -> Option<Wi> {
    Rz.lock().as_ref().map(|bc| bc.co.clone())
}


pub fn asx() -> (u64, u64, u64, u64) {
    (
        AGZ_.load(Ordering::Relaxed),
        AJW_.load(Ordering::Relaxed),
        JO_.load(Ordering::Relaxed),
        JP_.load(Ordering::Relaxed),
    )
}


pub fn ain(qa: u64, az: u8, bi: &mut [u8]) -> Result<usize, &'static str> {
    if az == 0 {
        return Err("Invalid sector count");
    }
    
    let bod = az as usize * H_;
    if bi.len() < bod {
        return Err("Buffer too small");
    }
    
    let disk = Rz.lock();
    let bwm = disk.as_ref().ok_or("Disk not initialized")?;
    
    let cun = (qa as usize) * H_;
    let epw = cun + bod;
    
    if epw > bwm.f.len() {
        return Err("Read past end of disk");
    }
    
    bi[..bod].dg(&bwm.f[cun..epw]);
    
    AGZ_.fetch_add(az as u64, Ordering::Relaxed);
    JO_.fetch_add(bod as u64, Ordering::Relaxed);
    
    Ok(bod)
}


pub fn bpi(qa: u64, az: u8, bi: &[u8]) -> Result<usize, &'static str> {
    if az == 0 {
        return Err("Invalid sector count");
    }
    
    let bod = az as usize * H_;
    if bi.len() < bod {
        return Err("Buffer too small");
    }
    
    let mut disk = Rz.lock();
    let bwm = disk.as_mut().ok_or("Disk not initialized")?;
    
    let cun = (qa as usize) * H_;
    let epw = cun + bod;
    
    if epw > bwm.f.len() {
        return Err("Write past end of disk");
    }
    
    bwm.f[cun..epw].dg(&bi[..bod]);
    
    AJW_.fetch_add(az as u64, Ordering::Relaxed);
    JP_.fetch_add(bod as u64, Ordering::Relaxed);
    
    Ok(bod)
}


pub fn xr(qa: u64) -> Result<[u8; H_], &'static str> {
    let mut bi = [0u8; H_];
    ain(qa, 1, &mut bi)?;
    Ok(bi)
}


pub fn aby(qa: u64, f: &[u8; H_]) -> Result<(), &'static str> {
    bpi(qa, 1, f)?;
    Ok(())
}


pub fn format() -> Result<(), &'static str> {
    {
        let mut disk = Rz.lock();
        let bwm = disk.as_mut().ok_or("Disk not initialized")?;
        
        
        for hf in bwm.f.el() {
            *hf = 0;
        }
    }
    
    
    oek();
    
    
    AGZ_.store(0, Ordering::Relaxed);
    AJW_.store(0, Ordering::Relaxed);
    JO_.store(0, Ordering::Relaxed);
    JP_.store(0, Ordering::Relaxed);
    
    crate::log!("[DISK] Disk formatted");
    Ok(())
}


pub fn shh(qa: u64) -> Result<String, &'static str> {
    let jk = xr(qa)?;
    
    use alloc::format;
    let mut result = format!("Sector {} (LBA):\n", qa);
    
    
    for br in 0..4 {
        let l = br * 16;
        result.t(&format!("{:04X}: ", l));
        
        for a in 0..16 {
            result.t(&format!("{:02X} ", jk[l + a]));
        }
        
        result.t(" |");
        for a in 0..16 {
            let r = jk[l + a];
            if r >= 0x20 && r < 0x7F {
                result.push(r as char);
            } else {
                result.push('.');
            }
        }
        result.t("|\n");
    }
    
    Ok(result)
}






#[derive(Debug, Clone)]
pub struct Aie {
    pub j: String,
    pub aw: u32,
    pub awy: u32,
    pub cfr: bool,
}

const GS_: usize = 32;
const AZN_: usize = 10;
const MU_: u64 = 1;
const BC_: u64 = 10;


pub fn jdr() -> Result<Vec<Aie>, &'static str> {
    let jk = xr(MU_)?;
    let mut sb = Vec::new();
    
    let bec = jk[0] as usize;
    if bec > AZN_ {
        return Err("Corrupted directory");
    }
    
    let acy = 1 + GS_ + 4 + 4 + 1; 
    
    for a in 0..bec {
        let l = 1 + a * acy;
        
        if l + acy > H_ {
            break;
        }
        
        if jk[l] == 0 {
            continue; 
        }
        
        
        let akj = l + 1;
        let mut bew = akj;
        while bew < akj + GS_ && jk[bew] != 0 {
            bew += 1;
        }
        let j = String::azw(&jk[akj..bew]).bkc();
        
        let cmv = l + 1 + GS_;
        let aw = u32::dj([
            jk[cmv],
            jk[cmv + 1],
            jk[cmv + 2],
            jk[cmv + 3],
        ]);
        
        let cmu = cmv + 4;
        let awy = u32::dj([
            jk[cmu],
            jk[cmu + 1],
            jk[cmu + 2],
            jk[cmu + 3],
        ]);
        
        let cfr = jk[cmu + 4] != 0;
        
        sb.push(Aie {
            j,
            aw,
            awy,
            cfr,
        });
    }
    
    Ok(sb)
}


pub fn ykj(j: &str, f: &[u8]) -> Result<(), &'static str> {
    if j.len() > GS_ - 1 {
        return Err("Filename too long");
    }
    
    
    let mut cec = xr(MU_)?;
    let bec = cec[0] as usize;
    
    if bec >= AZN_ {
        return Err("Directory full");
    }
    
    
    let dbu = (f.len() + H_ - 1) / H_;
    if dbu > 255 {
        return Err("File too large");
    }
    
    let awy = BC_ + (bec as u64 * 256); 
    
    
    let mut ia = f;
    let mut nil = awy;
    
    while !ia.is_empty() {
        let mut phg = [0u8; H_];
        let aiw = ia.len().v(H_);
        phg[..aiw].dg(&ia[..aiw]);
        
        aby(nil, &phg)?;
        
        ia = &ia[aiw..];
        nil += 1;
    }
    
    
    let acy = 1 + GS_ + 4 + 4 + 1;
    let bql = 1 + bec * acy;
    
    cec[bql] = 1; 
    
    
    for (a, hf) in j.bf().cf() {
        cec[bql + 1 + a] = hf;
    }
    
    
    let afz = (f.len() as u32).ho();
    let cmv = bql + 1 + GS_;
    cec[cmv..cmv + 4].dg(&afz);
    
    
    let wgc = (awy as u32).ho();
    let cmu = cmv + 4;
    cec[cmu..cmu + 4].dg(&wgc);
    
    
    cec[cmu + 4] = 0;
    
    
    cec[0] = (bec + 1) as u8;
    
    aby(MU_, &cec)?;
    
    crate::log!("[DISK] Created file: {} ({} bytes)", j, f.len());
    
    Ok(())
}


pub fn mq(j: &str) -> Result<Vec<u8>, &'static str> {
    let sb = jdr()?;
    
    let file = sb.iter().du(|bb| bb.j == j)
        .ok_or("File not found")?;
    
    let dbu = (file.aw as usize + H_ - 1) / H_;
    let dbu = dbu.am(1);
    let mut f = Vec::fc(file.aw as usize);
    
    for a in 0..dbu {
        let jk = xr(file.awy as u64 + a as u64)?;
        let ia = file.aw as usize - f.len();
        let aiw = ia.v(H_);
        f.bk(&jk[..aiw]);
    }
    
    Ok(f)
}


pub fn ylo(j: &str) -> Result<(), &'static str> {
    let mut cec = xr(MU_)?;
    let bec = cec[0] as usize;
    
    let acy = 1 + GS_ + 4 + 4 + 1;
    
    for a in 0..bec {
        let l = 1 + a * acy;
        
        if cec[l] == 0 {
            continue;
        }
        
        
        let akj = l + 1;
        let mut bew = akj;
        while bew < akj + GS_ && cec[bew] != 0 {
            bew += 1;
        }
        let cxm = String::azw(&cec[akj..bew]);
        
        if cxm == j {
            
            cec[l] = 0;
            aby(MU_, &cec)?;
            crate::log!("[DISK] Deleted file: {}", j);
            return Ok(());
        }
    }
    
    Err("File not found")
}
