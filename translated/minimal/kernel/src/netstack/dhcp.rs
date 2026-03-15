



use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;


static ABB_: AtomicBool = AtomicBool::new(false);


pub fn fvw() {
    ABB_.store(true, Ordering::SeqCst);
}


pub fn anu() {
    ABB_.store(false, Ordering::SeqCst);
}


pub fn tzb() -> bool {
    ABB_.load(Ordering::Relaxed)
}


mod msg_type {
    pub const Aqn: u8 = 1;
    pub const Akp: u8 = 2;
    pub const Ua: u8 = 3;
    pub const Ie: u8 = 5;
    pub const Xr: u8 = 6;
}


mod option {
    pub const Cin: u8 = 0;
    pub const XL_: u8 = 1;
    pub const Als: u8 = 3;
    pub const GG_: u8 = 6;
    pub const Bim: u8 = 12;
    pub const AHE_: u8 = 50;
    pub const UQ_: u8 = 51;
    pub const OH_: u8 = 53;
    pub const WY_: u8 = 54;
    pub const CIS_: u8 = 55;
    pub const Abj: u8 = 255;
}


#[derive(Debug, Clone, Copy, PartialEq)]
enum DhcpState {
    Nf,
    Amk,
    Axy,
    Vq,
    Axw,
    Axq,
}


struct Ahl {
    g: DhcpState,
    bxz: u32,
    evp: [u8; 4],
    aep: [u8; 4],
    jrw: [u8; 4],
    auj: [u8; 4],
    gfc: [u8; 4],
    fmw: u32,
    hbb: u64,
    clg: u64,
    arv: u8,
}

static Kf: Mutex<Ahl> = Mutex::new(Ahl {
    g: DhcpState::Nf,
    bxz: 0x12345678,
    evp: [0; 4],
    aep: [0; 4],
    jrw: [255, 255, 255, 0],
    auj: [0; 4],
    gfc: [8, 8, 8, 8],
    fmw: 0,
    hbb: 0,
    clg: 0,
    arv: 0,
});

static Li: AtomicBool = AtomicBool::new(false);
static Zy: AtomicBool = AtomicBool::new(false);


pub fn ay() {
    Li.store(true, Ordering::SeqCst);
    Zy.store(false, Ordering::SeqCst);
    
    let mut acx = Kf.lock();
    acx.g = DhcpState::Nf;
    acx.bxz = hld();
    acx.arv = 0;
    drop(acx);
    
    crate::log!("[DHCP] Client started");
    let _ = hzo();
}


pub fn flz() -> bool {
    Zy.load(Ordering::Relaxed)
}


pub fn nxw() -> Option<([u8; 4], [u8; 4], [u8; 4], [u8; 4])> {
    if !flz() { return None; }
    let acx = Kf.lock();
    Some((acx.evp, acx.jrw, acx.auj, acx.gfc))
}

fn hld() -> u32 {
    let qb = crate::logger::lh() as u32;
    let ed = crate::drivers::net::cez().unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    qb ^ ((ed[4] as u32) << 8) ^ (ed[5] as u32)
}

fn gbt(msg_type: u8, acx: &Ahl) -> Vec<u8> {
    kfl(msg_type, acx, [0u8; 4])
}

fn kfl(msg_type: u8, acx: &Ahl, gco: [u8; 4]) -> Vec<u8> {
    let ed = crate::drivers::net::cez().unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    let mut ex = Vec::fc(300);
    
    
    ex.push(1); ex.push(1); ex.push(6); ex.push(0);
    ex.bk(&acx.bxz.ft());
    ex.bk(&0u16.ft());
    ex.bk(&0x8000u16.ft());
    ex.bk(&gco); 
    ex.bk(&[0u8; 12]); 
    ex.bk(&ed);
    ex.bk(&[0u8; 10 + 64 + 128]); 
    ex.bk(&[99, 130, 83, 99]); 
    
    
    ex.bk(&[option::OH_, 1, msg_type]);
    
    if msg_type == msg_type::Ua && acx.evp != [0; 4] {
        
        
        if acx.g != DhcpState::Axw && acx.g != DhcpState::Axq {
            ex.bk(&[option::AHE_, 4]);
            ex.bk(&acx.evp);
            if acx.aep != [0; 4] {
                ex.bk(&[option::WY_, 4]);
                ex.bk(&acx.aep);
            }
        }
    }
    
    ex.bk(&[option::CIS_, 4, option::XL_, option::Als, option::GG_, option::UQ_]);
    ex.bk(&[option::Bim, 7]);
    ex.bk(b"trustos");
    ex.push(option::Abj);
    
    while ex.len() < 300 { ex.push(0); }
    ex
}

fn hzo() -> Result<(), &'static str> {
    let mut acx = Kf.lock();
    acx.g = DhcpState::Amk;
    acx.clg = crate::logger::lh();
    let ex = gbt(msg_type::Aqn, &acx);
    drop(acx);
    crate::serial_println!("[DHCP] Sending DISCOVER");
    mdq(&ex)
}

fn eii() -> Result<(), &'static str> {
    let mut acx = Kf.lock();
    acx.g = DhcpState::Axy;
    acx.clg = crate::logger::lh();
    let ex = gbt(msg_type::Ua, &acx);
    drop(acx);
    crate::serial_println!("[DHCP] Sending REQUEST");
    mdq(&ex)
}


fn phw() -> Result<(), &'static str> {
    let mut acx = Kf.lock();
    acx.g = DhcpState::Axw;
    acx.bxz = hld();
    acx.clg = crate::logger::lh();
    let gco = acx.evp;
    let bog = acx.aep;
    let ex = kfl(msg_type::Ua, &acx, gco);
    drop(acx);
    crate::serial_println!("[DHCP] Sending RENEW (unicast to {}.{}.{}.{})", bog[0], bog[1], bog[2], bog[3]);
    whb(&ex, gco, bog)
}


fn mdu() -> Result<(), &'static str> {
    let mut acx = Kf.lock();
    acx.g = DhcpState::Axq;
    acx.bxz = hld();
    acx.clg = crate::logger::lh();
    let gco = acx.evp;
    let ex = kfl(msg_type::Ua, &acx, gco);
    drop(acx);
    crate::serial_println!("[DHCP] Sending REBIND (broadcast)");
    mdq(&ex)
}

fn mdq(ew: &[u8]) -> Result<(), &'static str> {
    pht(ew, [0, 0, 0, 0], [255, 255, 255, 255], [0xFF; 6])
}


fn whb(ew: &[u8], jh: [u8; 4], pz: [u8; 4]) -> Result<(), &'static str> {
    
    let amc = crate::netstack::arp::ayo(pz).unwrap_or([0xFF; 6]);
    pht(ew, jh, pz, amc)
}

fn pht(ew: &[u8], jh: [u8; 4], pz: [u8; 4], amc: [u8; 6]) -> Result<(), &'static str> {
    let mut udp = Vec::fc(8 + ew.len());
    udp.bk(&68u16.ft()); 
    udp.bk(&67u16.ft()); 
    udp.bk(&((8 + ew.len()) as u16).ft());
    udp.bk(&0u16.ft());
    udp.bk(ew);
    
    let mut ip = Vec::fc(20 + udp.len());
    ip.push(0x45); ip.push(0);
    ip.bk(&((20 + udp.len()) as u16).ft());
    ip.bk(&[0, 0, 0, 0]); 
    ip.push(64); ip.push(17); 
    ip.bk(&0u16.ft()); 
    ip.bk(&jh);
    ip.bk(&pz);
    
    let mut sum: u32 = 0;
    for a in (0..20).akt(2) { sum += ((ip[a] as u32) << 8) | (ip[a + 1] as u32); }
    while sum >> 16 != 0 { sum = (sum & 0xFFFF) + (sum >> 16); }
    let td = !(sum as u16);
    ip[10] = (td >> 8) as u8; ip[11] = (td & 0xFF) as u8;
    ip.bk(&udp);
    
    crate::netstack::fug(amc, crate::netstack::ethertype::Aty, &ip)
}

pub fn bur(f: &[u8]) {
    if !Li.load(Ordering::Relaxed) || f.len() < 240 { return; }
    if f[0] != 2 { return; } 
    
    let bxz = u32::oa([f[4], f[5], f[6], f[7]]);
    { let r = Kf.lock(); if bxz != r.bxz { return; } }
    
    let btb = [f[16], f[17], f[18], f[19]];
    let wnt = [f[20], f[21], f[22], f[23]];
    if f[236..240] != [99, 130, 83, 99] { return; }
    
    let (mut msg_type, mut up, mut nt, mut dns, mut pii, mut anm) = 
        (0u8, [255,255,255,0], [0u8;4], [8,8,8,8], wnt, 86400u32);
    
    let mut a = 240;
    while a < f.len() {
        let fpt = f[a];
        if fpt == option::Abj { break; }
        if fpt == option::Cin { a += 1; continue; }
        if a + 1 >= f.len() { break; }
        let len = f[a + 1] as usize;
        if a + 2 + len > f.len() { break; }
        let p = &f[a + 2..a + 2 + len];
        match fpt {
            option::OH_ if len >= 1 => msg_type = p[0],
            option::XL_ if len >= 4 => up = [p[0], p[1], p[2], p[3]],
            option::Als if len >= 4 => nt = [p[0], p[1], p[2], p[3]],
            option::GG_ if len >= 4 => dns = [p[0], p[1], p[2], p[3]],
            option::WY_ if len >= 4 => pii = [p[0], p[1], p[2], p[3]],
            option::UQ_ if len >= 4 => anm = u32::oa([p[0], p[1], p[2], p[3]]),
            _ => {}
        }
        a += 2 + len;
    }
    
    match msg_type {
        msg_type::Akp => {
            crate::serial_println!("[DHCP] OFFER: {}.{}.{}.{}", btb[0], btb[1], btb[2], btb[3]);
            let mut r = Kf.lock();
            if r.g == DhcpState::Amk {
                r.evp = btb; r.aep = pii;
                r.jrw = up; r.auj = nt; r.gfc = dns;
                drop(r);
                let _ = eii();
            }
        }
        msg_type::Ie => {
            crate::log!("[DHCP] ACK: {}.{}.{}.{} (lease={}s)", btb[0], btb[1], btb[2], btb[3], anm);
            
            
            if tzb() {
                crate::serial_println!("[DHCP] Suspended - ignoring ACK");
                return;
            }
            
            let mut r = Kf.lock();
            r.g = DhcpState::Vq; r.evp = btb;
            r.jrw = up; r.auj = nt; r.gfc = dns; r.fmw = anm;
            r.hbb = crate::logger::lh();
            let ip = crate::network::Ipv4Address::new(btb[0], btb[1], btb[2], btb[3]);
            let hs = crate::network::Ipv4Address::new(up[0], up[1], up[2], up[3]);
            let til = crate::network::Ipv4Address::new(nt[0], nt[1], nt[2], nt[3]);
            crate::network::hzx(ip, hs, Some(til));
            
            crate::network::wis(dns);
            drop(r);
            Zy.store(true, Ordering::SeqCst);
            crate::log!("[DHCP] Configured: IP={}.{}.{}.{} GW={}.{}.{}.{} DNS={}.{}.{}.{}", 
                btb[0], btb[1], btb[2], btb[3], 
                nt[0], nt[1], nt[2], nt[3],
                dns[0], dns[1], dns[2], dns[3]);
        }
        msg_type::Xr => {
            crate::log_warn!("[DHCP] NAK, restarting");
            Kf.lock().g = DhcpState::Nf;
            Zy.store(false, Ordering::SeqCst);
            let _ = hzo();
        }
        _ => {}
    }
}

pub fn poll() {
    if !Li.load(Ordering::Relaxed) { return; }
    let iu = crate::logger::lh();
    let mut r = Kf.lock();
    
    match r.g {
        DhcpState::Vq => {
            
            let oz = iu.ao(r.hbb);
            let glg = (r.fmw as u64) * 1000;
            let aax = glg / 2;        
            let aco = glg * 7 / 8;    
            
            if oz >= aco {
                crate::serial_println!("[DHCP] T2 expired, rebinding");
                drop(r);
                let _ = mdu();
            } else if oz >= aax {
                crate::serial_println!("[DHCP] T1 expired, renewing");
                drop(r);
                let _ = phw();
            }
        }
        DhcpState::Axw => {
            
            let oz = iu.ao(r.hbb);
            let glg = (r.fmw as u64) * 1000;
            let aco = glg * 7 / 8;
            
            if oz >= aco {
                crate::serial_println!("[DHCP] Renew failed, rebinding");
                drop(r);
                let _ = mdu();
            } else if iu.ao(r.clg) > 30_000 {
                drop(r);
                let _ = phw();
            }
        }
        DhcpState::Axq => {
            
            let oz = iu.ao(r.hbb);
            let glg = (r.fmw as u64) * 1000;
            
            if oz >= glg {
                crate::log_warn!("[DHCP] Lease expired, restarting");
                r.g = DhcpState::Nf;
                r.arv = 0;
                drop(r);
                Zy.store(false, Ordering::SeqCst);
                let _ = hzo();
            } else if iu.ao(r.clg) > 30_000 {
                drop(r);
                let _ = mdu();
            }
        }
        DhcpState::Amk | DhcpState::Axy => {
            let aah = if r.g == DhcpState::Nf { 1000 } else { 3000 };
            if iu.ao(r.clg) > aah {
                r.arv += 1;
                if r.arv > 5 { r.g = DhcpState::Nf; r.bxz = hld(); r.arv = 0; }
                let g = r.g;
                drop(r);
                match g {
                    DhcpState::Nf | DhcpState::Amk => { let _ = hzo(); }
                    DhcpState::Axy => { let _ = eii(); }
                    _ => {}
                }
            }
        }
        DhcpState::Nf => {
            if iu.ao(r.clg) > 1000 {
                r.arv += 1;
                if r.arv > 5 { r.bxz = hld(); r.arv = 0; }
                drop(r);
                let _ = hzo();
            }
        }
    }
}
