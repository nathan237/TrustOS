








use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use core::sync::atomic::{AtomicU8, Ordering};

const H_: usize = 512;


const AQJ_: u16 = 128;
const BRX_: usize = AQJ_ as usize * H_; 

const FM_: usize = 10;

const ARG_: usize = 48;


#[derive(Clone)]
pub struct Anr {
    pub ddj: usize,
    pub aag: u64,
    pub agw: usize,
    pub ipo: u32,
    pub j: String,
}


pub struct Aqz {
    pub af: Vec<Anr>,
}


static ZT_: AtomicU8 = AtomicU8::new(0xFE);



fn nue() -> Option<u8> {
    let ene = ZT_.load(Ordering::Relaxed);
    if ene != 0xFE {
        return if ene == 0xFF { None } else { Some(ene) };
    }
    if !crate::drivers::ahci::ky() {
        return None;
    }
    let ik = crate::drivers::ahci::bhh();
    for ba in &ik {
        if ba.ceb == crate::drivers::ahci::AhciDeviceType::Qr && ba.agw > 64 {
            let mut probe = alloc::vec![0u8; 512];
            if crate::drivers::ahci::ain(ba.kg, 0, 1, &mut probe).is_ok() {
                if probe.len() >= 4 && &probe[0..4] == b"TWAV" {
                    crate::serial_println!("[DISK-AUDIO] Found TWAV disk on port {}", ba.kg);
                    ZT_.store(ba.kg, Ordering::Relaxed);
                    return Some(ba.kg);
                }
            }
        }
    }
    ZT_.store(0xFF, Ordering::Relaxed);
    None
}


pub fn lxv() -> Result<Aqz, &'static str> {
    let port = nue().ok_or("No data disk found on AHCI")?;

    crate::serial_println!("[DISK-AUDIO] Reading header from port {}...", port);

    let mut vk = vec![0u8; H_];
    crate::drivers::ahci::ain(port, 0, 1, &mut vk)?;

    if &vk[0..4] != b"TWAV" {
        return Err("Invalid data disk header (no TWAV magic)");
    }

    let dk = u32::dj([vk[4], vk[5], vk[6], vk[7]]);

    match dk {
        1 => {
            
            let ddj = u64::dj([
                vk[8], vk[9], vk[10], vk[11],
                vk[12], vk[13], vk[14], vk[15],
            ]) as usize;
            let aag = u32::dj([
                vk[16], vk[17], vk[18], vk[19],
            ]) as u64;
            let agw = u32::dj([
                vk[20], vk[21], vk[22], vk[23],
            ]) as usize;
            let ipo = u32::dj([
                vk[24], vk[25], vk[26], vk[27],
            ]);
            crate::serial_println!("[DISK-AUDIO] v1: 1 track, {} bytes", ddj);
            Ok(Aqz {
                af: alloc::vec![Anr {
                    ddj, aag, agw, ipo,
                    j: String::from("Untitled (2)"),
                }],
            })
        }
        2 => {
            let alm = u32::dj([
                vk[8], vk[9], vk[10], vk[11],
            ]) as usize;
            let alm = alm.v(FM_);
            let mut af = Vec::fc(alm);
            for a in 0..alm {
                let dz = 16 + a * ARG_;
                if dz + ARG_ > H_ { break; }
                let ddj = u64::dj([
                    vk[dz], vk[dz+1], vk[dz+2], vk[dz+3],
                    vk[dz+4], vk[dz+5], vk[dz+6], vk[dz+7],
                ]) as usize;
                let aag = u32::dj([
                    vk[dz+8], vk[dz+9], vk[dz+10], vk[dz+11],
                ]) as u64;
                let agw = u32::dj([
                    vk[dz+12], vk[dz+13], vk[dz+14], vk[dz+15],
                ]) as usize;
                let ipo = u32::dj([
                    vk[dz+16], vk[dz+17], vk[dz+18], vk[dz+19],
                ]);
                
                let bko = &vk[dz+20..dz+48];
                let baf = bko.iter().qf(|&o| o == 0).unwrap_or(28);
                let j = core::str::jg(&bko[..baf])
                    .unwrap_or("Unknown")
                    .into();
                crate::serial_println!("[DISK-AUDIO] Track {}: '{}' {} bytes, LBA {}", a, j, ddj, aag);
                af.push(Anr { ddj, aag, agw, ipo, j });
            }
            if af.is_empty() {
                return Err("No tracks in v2 header");
            }
            Ok(Aqz { af })
        }
        _ => Err("Unsupported data disk version"),
    }
}


pub fn ojy(zx: usize) -> Result<(Vec<u8>, String), &'static str> {
    let gg = lxv()?;
    if zx >= gg.af.len() {
        return Err("Track index out of range");
    }
    let track = &gg.af[zx];
    uhj(track)
}






fn uhj(track: &Anr) -> Result<(Vec<u8>, String), &'static str> {
    let port = nue().ok_or("No data disk found on AHCI")?;

    if track.ddj == 0 || track.ddj > 60 * 1024 * 1024 {
        return Err("Invalid WAV size in header");
    }

    crate::serial_println!("[DISK-AUDIO] Loading '{}': {} bytes, {} sectors from LBA {}",
        track.j, track.ddj, track.agw, track.aag);

    
    
    let mut alb = vec![0u8; BRX_];

    let mut ihc = Vec::fc(track.ddj);

    let mut mdb = track.agw;
    let mut kmx = track.aag;

    while mdb > 0 {
        
        
        
        let inp = mdb.v(AQJ_ as usize);
        let jj = inp as u16;
        let dov = inp * H_;

        crate::drivers::ahci::ain(
            port, kmx, jj,
            &mut alb[..dov],
        )?;

        
        let lnr = track.ddj.ao(ihc.len());
        let acq = dov.v(lnr);
        if acq > 0 {
            ihc.bk(&alb[..acq]);
        }

        kmx += inp as u64;
        mdb -= inp;
    }

    ihc.dmu(track.ddj);
    let j = track.j.clone();
    crate::serial_println!("[DISK-AUDIO] Loaded '{}': {} bytes", j, ihc.len());
    Ok((ihc, j))
}


pub fn xli() -> usize {
    lxv().map(|ab| ab.af.len()).unwrap_or(0)
}


pub fn uhr() -> Result<Vec<u8>, &'static str> {
    let (f, blu) = ojy(0)?;
    Ok(f)
}


pub fn tey() -> Vec<String> {
    match lxv() {
        Ok(gg) => gg.af.iter().map(|ab| ab.j.clone()).collect(),
        Err(_) => Vec::new(),
    }
}
