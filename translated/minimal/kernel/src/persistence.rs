




use alloc::vec::Vec;
use alloc::string::String;
use alloc::vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};


const WC_: &[u8; 8] = b"TRUSTPST";


const BCY_: u32 = 1;


const FQ_: u64 = 2048; 


const CIX_: u64 = 32768; 


const H_: usize = 512;


static AGJ_: AtomicBool = AtomicBool::new(false);


static BCX_: AtomicBool = AtomicBool::new(false);


static OV_: Mutex<Option<u8>> = Mutex::new(None);


#[repr(C, packed)]
struct Awv {
    sj: [u8; 8],      
    dk: u32,         
    ame: u32,     
    aay: u64,      
    bmj: u32,        
    asi: [u8; 484], 
}


#[repr(C, packed)]
struct Aie {
    gox: u16,        
    cwv: u32,        
    asi: [u8; 2],   
    
}


pub fn init() {
    crate::serial_println!("[PERSIST] Initializing persistence system...");
    
    
    let port = ssx();
    if port.is_none() {
        crate::serial_println!("[PERSIST] No AHCI disk found, persistence disabled");
        return;
    }
    
    let port = port.unwrap();
    *OV_.lock() = Some(port);
    
    
    if ncr(port) {
        AGJ_.store(true, Ordering::Relaxed);
        crate::serial_println!("[PERSIST] Found existing persistence data on port {}", port);
    } else {
        crate::serial_println!("[PERSIST] No existing persistence data found");
    }
}


fn ssx() -> Option<u8> {
    
    let xf = crate::drivers::ahci::bhh();
    for port in xf {
        
        if port.agw > 0 {
            
            let mut bi = [0u8; 512];
            if crate::drivers::ahci::ain(port.kg, 0, 1, &mut bi).is_ok() {
                return Some(port.kg);
            }
        }
    }
    None
}


fn ncr(port: u8) -> bool {
    let mut bi = [0u8; 512];
    
    if crate::drivers::ahci::ain(port, FQ_, 1, &mut bi).is_err() {
        return false;
    }
    
    
    &bi[0..8] == WC_
}


pub fn vng() -> bool {
    if !AGJ_.load(Ordering::Relaxed) {
        return false;
    }
    
    crate::println!();
    crate::h!(0x00FFFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::h!(0x00FFFF, "║           Saved Data Detected                                ║");
    crate::h!(0x00FFFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    crate::println!("  Previously downloaded files were found on disk.");
    crate::println!("  Do you want to restore them? (Y/n)");
    crate::println!();
    crate::print!("  > ");
    
    
    let mk = vrh();
    
    let pcv = mk != b'n' && mk != b'N';
    
    if pcv {
        crate::println!("Yes");
        crate::println!();
        crate::h!(0x00FF00, "  Restoring saved data...");
        
        if let Err(aa) = vye() {
            crate::h!(0xFF0000, "  Error restoring: {}", aa);
            return false;
        }
        
        BCX_.store(true, Ordering::Relaxed);
        crate::h!(0x00FF00, "  Data restored successfully!");
    } else {
        crate::println!("No");
        crate::println!("  Starting fresh.");
    }
    
    crate::println!();
    pcv
}


fn vrh() -> u8 {
    loop {
        if let Some(r) = crate::keyboard::auw() {
            return r;
        }
        core::hint::hc();
    }
}


pub fn ftm(path: &str, f: &[u8]) -> Result<(), &'static str> {
    let port = OV_.lock().ok_or("Persistence not available")?;
    
    crate::serial_println!("[PERSIST] Saving {} ({} bytes)", path, f.len());
    
    
    let mut vk = [0u8; 512];
    let mut dh = if ncr(port) {
        crate::drivers::ahci::ain(port, FQ_, 1, &mut vk)
            .jd(|_| "Failed to read header")?;
        fqg(&vk)?
    } else {
        
        Awv {
            sj: *WC_,
            dk: BCY_,
            ame: 0,
            aay: 0,
            bmj: 0,
            asi: [0; 484],
        }
    };
    
    
    let njr = FQ_ + 1 + (dh.aay + 511) / 512;
    
    
    if njr + ((f.len() as u64 + path.len() as u64 + 16) / 512) + 1 
        > FQ_ + CIX_ {
        return Err("Persistence storage full");
    }
    
    
    let bzn = Aie {
        gox: path.len() as u16,
        cwv: f.len() as u32,
        asi: [0; 2],
    };
    
    let mut ebg: Vec<u8> = Vec::new();
    
    
    let giy: [u8; 8] = unsafe { core::mem::transmute(bzn) };
    ebg.bk(&giy);
    
    
    ebg.bk(path.as_bytes());
    
    
    ebg.bk(f);
    
    
    while ebg.len() % 512 != 0 {
        ebg.push(0);
    }
    
    
    let dbu = ebg.len() / 512;
    for a in 0..dbu {
        let jk = njr + a as u64;
        let l = a * 512;
        let mut aae = [0u8; 512];
        aae.dg(&ebg[l..l + 512]);
        
        crate::drivers::ahci::bpi(port, jk, 1, &aae)
            .jd(|_| "Failed to write data sector")?;
    }
    
    
    dh.ame += 1;
    dh.aay += ebg.len() as u64;
    dh.bmj = rnf(&ebg);
    
    
    let mut erw = [0u8; 512];
    erw[0..8].dg(&dh.sj);
    erw[8..12].dg(&dh.dk.ho());
    erw[12..16].dg(&dh.ame.ho());
    erw[16..24].dg(&dh.aay.ho());
    erw[24..28].dg(&dh.bmj.ho());
    
    crate::drivers::ahci::bpi(port, FQ_, 1, &erw)
        .jd(|_| "Failed to write header")?;
    
    crate::serial_println!("[PERSIST] Saved {} successfully", path);
    Ok(())
}


fn vye() -> Result<(), &'static str> {
    let port = *OV_.lock();
    let port = port.ok_or("Persistence not available")?;
    
    
    let mut vk = [0u8; 512];
    crate::drivers::ahci::ain(port, FQ_, 1, &mut vk)
        .jd(|_| "Failed to read header")?;
    
    let dh = fqg(&vk)?;
    
    let ame = dh.ame;
    let aay = dh.aay;
    
    
    if ame > 1000 {
        return Err("Corrupted: too many entries");
    }
    if aay > 100 * 1024 * 1024 {
        return Err("Corrupted: size too large");
    }
    if aay == 0 {
        return Ok(()); 
    }
    
    crate::serial_println!("[PERSIST] Restoring {} files ({} bytes)", 
        ame, aay);
    
    
    let axf = (aay + 511) / 512;
    let mut cvn: Vec<u8> = vec![0u8; aay as usize];
    
    for a in 0..axf {
        let jk = FQ_ + 1 + a;
        let l = (a as usize) * 512;
        let mut aae = [0u8; 512];
        
        crate::drivers::ahci::ain(port, jk, 1, &mut aae)
            .jd(|_| "Failed to read data sector")?;
        
        let zg = core::cmp::v(512, cvn.len() - l);
        cvn[l..l + zg].dg(&aae[..zg]);
    }
    
    
    let mut l = 0;
    let mut lzw = 0;
    
    while l + 8 <= cvn.len() && lzw < dh.ame {
        
        let gox = u16::dj([cvn[l], cvn[l + 1]]) as usize;
        let cwv = u32::dj([
            cvn[l + 2], cvn[l + 3],
            cvn[l + 4], cvn[l + 5],
        ]) as usize;
        
        l += 8;
        
        if l + gox + cwv > cvn.len() {
            break;
        }
        
        
        let path = match core::str::jg(&cvn[l..l + gox]) {
            Ok(e) => e,
            Err(_) => {
                l += gox + cwv;
                continue;
            }
        };
        l += gox;
        
        
        let f = &cvn[l..l + cwv];
        l += cwv;
        
        
        l = (l + 511) & !511;
        
        
        crate::serial_println!("[PERSIST] Restoring: {} ({} bytes)", path, cwv);
        
        
        let result = crate::ramfs::fh(|fs| {
            
            let mut cv = String::new();
            for vu in path.adk('/').hi(|ai| !ai.is_empty()) {
                if !path.pp(vu) {
                    cv.push('/');
                    cv.t(vu);
                    let _ = fs.ut(&cv);
                }
            }
            
            
            let _ = fs.touch(path);
            fs.ns(path, f)
        });
        
        if result.is_ok() {
            lzw += 1;
            crate::print!(".");
        }
    }
    
    crate::println!();
    crate::println!("  Restored {} files", lzw);
    
    Ok(())
}


fn fqg(k: &[u8; 512]) -> Result<Awv, &'static str> {
    if &k[0..8] != WC_ {
        return Err("Invalid magic signature");
    }
    
    let dk = u32::dj([k[8], k[9], k[10], k[11]]);
    if dk != BCY_ {
        return Err("Incompatible version");
    }
    
    Ok(Awv {
        sj: *WC_,
        dk,
        ame: u32::dj([k[12], k[13], k[14], k[15]]),
        aay: u64::dj([
            k[16], k[17], k[18], k[19],
            k[20], k[21], k[22], k[23],
        ]),
        bmj: u32::dj([k[24], k[25], k[26], k[27]]),
        asi: [0; 484],
    })
}


fn rnf(f: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    for hf in f {
        sum = sum.cn(*hf as u32);
    }
    sum
}


pub fn clear() -> Result<(), &'static str> {
    let port = *OV_.lock();
    let port = port.ok_or("Persistence not available")?;
    
    
    let erw = [0u8; 512];
    crate::drivers::ahci::bpi(port, FQ_, 1, &erw)
        .jd(|_| "Failed to clear persistence")?;
    
    crate::serial_println!("[PERSIST] Persistence data cleared");
    Ok(())
}


pub fn zu() -> bool {
    BCX_.load(Ordering::Relaxed)
}


pub fn anl() -> bool {
    AGJ_.load(Ordering::Relaxed)
}


pub fn status() -> (&'static str, u32, u64) {
    let port = *OV_.lock();
    
    if port.is_none() {
        return ("No disk", 0, 0);
    }
    
    let port = port.unwrap();
    let mut vk = [0u8; 512];
    
    if crate::drivers::ahci::ain(port, FQ_, 1, &mut vk).is_err() {
        return ("Read error", 0, 0);
    }
    
    if let Ok(dh) = fqg(&vk) {
        ("Active", dh.ame, dh.aay)
    } else {
        ("Empty", 0, 0)
    }
}
