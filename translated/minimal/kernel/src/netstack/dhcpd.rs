





use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use spin::Mutex;


static Ja: AtomicBool = AtomicBool::new(false);


static AEK_: AtomicU8 = AtomicU8::new(0);


const AFC_: usize = 16;


#[derive(Clone, Copy)]
struct Lease {
    ed: [u8; 6],
    ip: [u8; 4],
    gh: bool,
    hlv: u64,
    fmw: u32,
}

impl Lease {
    const fn azs() -> Self {
        Self {
            ed: [0; 6],
            ip: [0; 4],
            gh: false,
            hlv: 0,
            fmw: 86400, 
        }
    }
}


#[derive(Clone, Copy)]
struct PxeConfig {
    
    aep: [u8; 4],
    
    up: [u8; 4],
    
    auj: [u8; 4],
    
    dun: [u8; 4],
    
    duo: u8,
    
    ily: [u8; 128],
    fdp: usize,
}

impl PxeConfig {
    const fn default() -> Self {
        Self {
            aep: [10, 0, 2, 1],
            up: [255, 255, 255, 0],
            auj: [10, 0, 2, 1],
            dun: [10, 0, 2, 100],
            duo: 16,
            ily: [0; 128],
            fdp: 0,
        }
    }
}

static Acm: Mutex<[Lease; AFC_]> = Mutex::new([Lease::azs(); AFC_]);
static Pa: Mutex<PxeConfig> = Mutex::new(PxeConfig::default());


mod msg_type {
    pub const Aqn: u8 = 1;
    pub const Akp: u8 = 2;
    pub const Ua: u8 = 3;
    pub const ELD_: u8 = 4;
    pub const Ie: u8 = 5;
    pub const Xr: u8 = 6;
}


mod option {
    pub const XL_: u8 = 1;
    pub const Als: u8 = 3;
    pub const GG_: u8 = 6;
    pub const Bim: u8 = 12;
    pub const AHE_: u8 = 50;
    pub const UQ_: u8 = 51;
    pub const OH_: u8 = 53;
    pub const WY_: u8 = 54;
    pub const CXW_: u8 = 66;
    pub const BLI_: u8 = 67;
    pub const CMR_: u8 = 60;
    pub const Abj: u8 = 255;
}


pub fn dsi() -> bool {
    Ja.load(Ordering::Relaxed)
}


pub fn qez() -> u8 {
    AEK_.load(Ordering::Relaxed)
}


pub fn ay(aep: [u8; 4], up: [u8; 4], dkt: [u8; 4], duo: u8, mzt: &str) {
    if Ja.load(Ordering::Relaxed) {
        crate::serial_println!("[DHCPD] Already running");
        return;
    }

    let mut cfg = Pa.lock();
    cfg.aep = aep;
    cfg.up = up;
    cfg.auj = aep; 
    cfg.dun = dkt;
    cfg.duo = duo;

    
    let bf = mzt.as_bytes();
    let len = bf.len().v(127);
    cfg.ily[..len].dg(&bf[..len]);
    cfg.fdp = len;
    drop(cfg);

    
    let mut bkf = Acm.lock();
    for dm in bkf.el() {
        *dm = Lease::azs();
    }
    drop(bkf);
    AEK_.store(0, Ordering::Relaxed);

    Ja.store(true, Ordering::Relaxed);
    crate::serial_println!("[DHCPD] PXE DHCP server started on {}.{}.{}.{}",
        aep[0], aep[1], aep[2], aep[3]);
    crate::serial_println!("[DHCPD] Pool: {}.{}.{}.{} - {}.{}.{}.{} ({} IPs)",
        dkt[0], dkt[1], dkt[2], dkt[3],
        dkt[0], dkt[1], dkt[2], dkt[3] + duo - 1,
        duo);
    crate::serial_println!("[DHCPD] PXE boot file: {}", mzt);
}


pub fn qg() {
    Ja.store(false, Ordering::Relaxed);
    crate::serial_println!("[DHCPD] Server stopped");
}



pub fn bur(f: &[u8]) {
    if !Ja.load(Ordering::Relaxed) {
        return;
    }

    
    if f.len() < 240 {
        return;
    }

    
    if f[0] != 1 {
        return;
    }

    let bxz = [f[4], f[5], f[6], f[7]];
    let bgh = [f[28], f[29], f[30], f[31], f[32], f[33]];

    
    if f[236] != 99 || f[237] != 130 || f[238] != 83 || f[239] != 99 {
        return;
    }

    
    let options = &f[240..];
    let mut lmw: u8 = 0;
    let mut pck: Option<[u8; 4]> = None;
    let mut jbq = false;
    let mut a = 0;

    while a < options.len() {
        let fpt = options[a];
        if fpt == option::Abj {
            break;
        }
        if fpt == 0 {
            a += 1; 
            continue;
        }
        if a + 1 >= options.len() {
            break;
        }
        let len = options[a + 1] as usize;
        if a + 2 + len > options.len() {
            break;
        }
        let ap = &options[a + 2..a + 2 + len];

        match fpt {
            option::OH_ => {
                if len >= 1 { lmw = ap[0]; }
            }
            option::AHE_ => {
                if len >= 4 {
                    pck = Some([ap[0], ap[1], ap[2], ap[3]]);
                }
            }
            option::CMR_ => {
                
                if len >= 9 && &ap[..9] == b"PXEClient" {
                    jbq = true;
                }
            }
            _ => {}
        }

        a += 2 + len;
    }

    crate::serial_println!("[DHCPD] Received {} from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} (PXE: {})",
        match lmw {
            1 => "DISCOVER",
            3 => "REQUEST",
            _ => "UNKNOWN",
        },
        bgh[0], bgh[1], bgh[2],
        bgh[3], bgh[4], bgh[5],
        jbq);

    match lmw {
        msg_type::Aqn => {
            
            if let Some(uxi) = stm(&bgh) {
                jok(msg_type::Akp, &bxz, &bgh, &uxi, jbq);
            } else {
                crate::serial_println!("[DHCPD] No IPs available in pool!");
            }
        }
        msg_type::Ua => {
            
            let ip = pck.unwrap_or_else(|| {
                
                sth(&bgh).unwrap_or([0; 4])
            });

            if ip == [0; 4] {
                jok(msg_type::Xr, &bxz, &bgh, &[0; 4], false);
                return;
            }

            
            if xql(&bgh, &ip) {
                jok(msg_type::Ie, &bxz, &bgh, &ip, jbq);
            } else {
                jok(msg_type::Xr, &bxz, &bgh, &[0; 4], false);
            }
        }
        _ => {}
    }
}


fn stm(ed: &[u8; 6]) -> Option<[u8; 4]> {
    let mut bkf = Acm.lock();
    let cfg = Pa.lock();

    
    for anm in bkf.iter() {
        if anm.gh && anm.ed == *ed {
            return Some(anm.ip);
        }
    }

    
    let duo = cfg.duo as usize;
    for l in 0..duo.v(AFC_) {
        let kgi = [
            cfg.dun[0],
            cfg.dun[1],
            cfg.dun[2],
            cfg.dun[3].cn(l as u8),
        ];

        
        let qhj = bkf.iter().any(|dm| dm.gh && dm.ip == kgi);
        if !qhj {
            
            for anm in bkf.el() {
                if !anm.gh {
                    anm.ed = *ed;
                    anm.ip = kgi;
                    anm.gh = true;
                    anm.hlv = crate::time::lc();
                    AEK_.fetch_add(1, Ordering::Relaxed);
                    return Some(kgi);
                }
            }
        }
    }

    None
}


fn sth(ed: &[u8; 6]) -> Option<[u8; 4]> {
    let bkf = Acm.lock();
    for anm in bkf.iter() {
        if anm.gh && anm.ed == *ed {
            return Some(anm.ip);
        }
    }
    None
}


fn xql(ed: &[u8; 6], ip: &[u8; 4]) -> bool {
    let mut bkf = Acm.lock();

    
    for anm in bkf.el() {
        if anm.gh && anm.ed == *ed && anm.ip == *ip {
            anm.hlv = crate::time::lc();
            return true;
        }
    }

    
    for anm in bkf.el() {
        if anm.gh && anm.ed == *ed {
            
            let cfg = Pa.lock();
            let myb = cfg.dun[3];
            let vjv = myb.cn(cfg.duo);
            if ip[0] == cfg.dun[0] && ip[1] == cfg.dun[1]
                && ip[2] == cfg.dun[2]
                && ip[3] >= myb && ip[3] < vjv
            {
                anm.ip = *ip;
                anm.hlv = crate::time::lc();
                return true;
            }
            
            return false;
        }
    }

    false
}


fn jok(lzt: u8, bxz: &[u8; 4], bgh: &[u8; 6], bjn: &[u8; 4], jkp: bool) {
    let cfg = Pa.lock();
    let mut ex = Vec::fc(400);

    
    ex.push(2);                         
    ex.push(1);                         
    ex.push(6);                         
    ex.push(0);                         
    ex.bk(bxz);          
    ex.bk(&[0, 0]);      
    ex.bk(&[0x80, 0x00]); 
    ex.bk(&[0, 0, 0, 0]); 
    ex.bk(bjn);    
    ex.bk(&cfg.aep); 
    ex.bk(&[0, 0, 0, 0]); 
    ex.bk(bgh);   
    ex.bk(&[0u8; 10]);   
    
    
    if jkp {
        let wqb = format!("{}.{}.{}.{}",
            cfg.aep[0], cfg.aep[1],
            cfg.aep[2], cfg.aep[3]);
        let plu = wqb.as_bytes();
        let pli = plu.len().v(63);
        ex.bk(&plu[..pli]);
        for _ in pli..64 {
            ex.push(0);
        }
    } else {
        ex.bk(&[0u8; 64]);
    }

    
    if jkp && cfg.fdp > 0 {
        let nvb = cfg.fdp.v(127);
        ex.bk(&cfg.ily[..nvb]);
        for _ in nvb..128 {
            ex.push(0);
        }
    } else {
        ex.bk(&[0u8; 128]);
    }

    
    ex.bk(&[99, 130, 83, 99]);

    
    
    ex.bk(&[option::OH_, 1, lzt]);

    
    ex.bk(&[option::WY_, 4]);
    ex.bk(&cfg.aep);

    if lzt != msg_type::Xr {
        
        ex.bk(&[option::UQ_, 4]);
        ex.bk(&86400u32.ft());

        
        ex.bk(&[option::XL_, 4]);
        ex.bk(&cfg.up);

        
        ex.bk(&[option::Als, 4]);
        ex.bk(&cfg.auj);

        
        ex.bk(&[option::GG_, 4]);
        ex.bk(&cfg.aep);

        if jkp {
            
            let wia = format!("{}.{}.{}.{}",
                cfg.aep[0], cfg.aep[1],
                cfg.aep[2], cfg.aep[3]);
            let is = wia.as_bytes();
            ex.push(option::CXW_);
            ex.push(is.len() as u8);
            ex.bk(is);

            
            if cfg.fdp > 0 {
                ex.push(option::BLI_);
                ex.push(cfg.fdp as u8);
                ex.bk(&cfg.ily[..cfg.fdp]);
            }
        }
    }

    
    ex.push(option::Abj);

    
    while ex.len() < 300 {
        ex.push(0);
    }

    drop(cfg);

    
    let jh = Pa.lock().aep;
    let vyb = match lzt {
        msg_type::Akp => "OFFER",
        msg_type::Ie => "ACK",
        msg_type::Xr => "NAK",
        _ => "?",
    };
    crate::serial_println!("[DHCPD] Sending {} to {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} -> {}.{}.{}.{} (PXE: {})",
        vyb,
        bgh[0], bgh[1], bgh[2],
        bgh[3], bgh[4], bgh[5],
        bjn[0], bjn[1], bjn[2], bjn[3],
        jkp);

    
    wha(&ex, jh);
}


fn wha(ew: &[u8], jh: [u8; 4]) {
    
    let mut udp = Vec::fc(8 + ew.len());
    udp.bk(&67u16.ft());  
    udp.bk(&68u16.ft());  
    udp.bk(&((8 + ew.len()) as u16).ft());
    udp.bk(&0u16.ft());   
    udp.bk(ew);

    
    let mut ip = Vec::fc(20 + udp.len());
    ip.push(0x45); ip.push(0x10); 
    ip.bk(&((20 + udp.len()) as u16).ft());
    ip.bk(&[0, 0, 0x40, 0x00]); 
    ip.push(64); ip.push(17); 
    ip.bk(&0u16.ft()); 
    ip.bk(&jh);
    ip.bk(&[255, 255, 255, 255]); 

    
    let mut sum: u32 = 0;
    for a in (0..20).akt(2) {
        sum += ((ip[a] as u32) << 8) | (ip[a + 1] as u32);
    }
    while sum >> 16 != 0 { sum = (sum & 0xFFFF) + (sum >> 16); }
    let td = !(sum as u16);
    ip[10] = (td >> 8) as u8;
    ip[11] = (td & 0xFF) as u8;

    ip.bk(&udp);

    
    let _ = crate::netstack::fug([0xFF; 6], crate::netstack::ethertype::Aty, &ip);
}


pub fn tdw() -> Vec<([u8; 6], [u8; 4], u64)> {
    let bkf = Acm.lock();
    let mut result = Vec::new();
    for anm in bkf.iter() {
        if anm.gh {
            result.push((anm.ed, anm.ip, anm.hlv));
        }
    }
    result
}
