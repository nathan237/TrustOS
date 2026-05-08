





use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use spin::Mutex;


static Dr: AtomicBool = AtomicBool::new(false);


static AGE_: AtomicU8 = AtomicU8::new(0);


const AGW_: usize = 16;


#[derive(Clone, Copy)]
struct Lease {
    mac: [u8; 6],
    ip: [u8; 4],
    active: bool,
    granted_at: u64,
    lease_time: u32,
}

impl Lease {
    const fn empty() -> Self {
        Self {
            mac: [0; 6],
            ip: [0; 4],
            active: false,
            granted_at: 0,
            lease_time: 86400, 
        }
    }
}


#[derive(Clone, Copy)]
struct PxeConfig {
    
    server_ip: [u8; 4],
    
    subnet: [u8; 4],
    
    gateway: [u8; 4],
    
    pool_base: [u8; 4],
    
    pool_size: u8,
    
    boot_file: [u8; 128],
    boot_file_len: usize,
}

impl PxeConfig {
    const fn default() -> Self {
        Self {
            server_ip: [10, 0, 2, 1],
            subnet: [255, 255, 255, 0],
            gateway: [10, 0, 2, 1],
            pool_base: [10, 0, 2, 100],
            pool_size: 16,
            boot_file: [0; 128],
            boot_file_len: 0,
        }
    }
}

static Mj: Mutex<[Lease; AGW_]> = Mutex::new([Lease::empty(); AGW_]);
static Gh: Mutex<PxeConfig> = Mutex::new(PxeConfig::default());


mod msg_type {
    pub const Ro: u8 = 1;
    pub const Ps: u8 = 2;
    pub const Iq: u8 = 3;
    pub const EOO_: u8 = 4;
    pub const Dk: u8 = 5;
    pub const Kf: u8 = 6;
}


mod option {
    pub const YS_: u8 = 1;
    pub const Qa: u8 = 3;
    pub const GX_: u8 = 6;
    pub const Zo: u8 = 12;
    pub const AJA_: u8 = 50;
    pub const VZ_: u8 = 51;
    pub const PF_: u8 = 53;
    pub const YF_: u8 = 54;
    pub const DBO_: u8 = 66;
    pub const BOA_: u8 = 67;
    pub const CQA_: u8 = 60;
    pub const Lq: u8 = 255;
}


pub fn is_running() -> bool {
    Dr.load(Ordering::Relaxed)
}


pub fn jtn() -> u8 {
    AGE_.load(Ordering::Relaxed)
}


pub fn start(server_ip: [u8; 4], subnet: [u8; 4], bis: [u8; 4], pool_size: u8, boot_filename: &str) {
    if Dr.load(Ordering::Relaxed) {
        crate::serial_println!("[DHCPD] Already running");
        return;
    }

    let mut cfg = Gh.lock();
    cfg.server_ip = server_ip;
    cfg.subnet = subnet;
    cfg.gateway = server_ip; 
    cfg.pool_base = bis;
    cfg.pool_size = pool_size;

    
    let bytes = boot_filename.as_bytes();
    let len = bytes.len().min(127);
    cfg.boot_file[..len].copy_from_slice(&bytes[..len]);
    cfg.boot_file_len = len;
    drop(cfg);

    
    let mut agp = Mj.lock();
    for l in agp.iter_mut() {
        *l = Lease::empty();
    }
    drop(agp);
    AGE_.store(0, Ordering::Relaxed);

    Dr.store(true, Ordering::Relaxed);
    crate::serial_println!("[DHCPD] PXE DHCP server started on {}.{}.{}.{}",
        server_ip[0], server_ip[1], server_ip[2], server_ip[3]);
    crate::serial_println!("[DHCPD] Pool: {}.{}.{}.{} - {}.{}.{}.{} ({} IPs)",
        bis[0], bis[1], bis[2], bis[3],
        bis[0], bis[1], bis[2], bis[3] + pool_size - 1,
        pool_size);
    crate::serial_println!("[DHCPD] PXE boot file: {}", boot_filename);
}


pub fn stop() {
    Dr.store(false, Ordering::Relaxed);
    crate::serial_println!("[DHCPD] Server stopped");
}



pub fn alq(data: &[u8]) {
    if !Dr.load(Ordering::Relaxed) {
        return;
    }

    
    if data.len() < 240 {
        return;
    }

    
    if data[0] != 1 {
        return;
    }

    let xid = [data[4], data[5], data[6], data[7]];
    let aew = [data[28], data[29], data[30], data[31], data[32], data[33]];

    
    if data[236] != 99 || data[237] != 130 || data[238] != 83 || data[239] != 99 {
        return;
    }

    
    let options = &data[240..];
    let mut gii: u8 = 0;
    let mut jag: Option<[u8; 4]> = None;
    let mut ert = false;
    let mut i = 0;

    while i < options.len() {
        let cnn = options[i];
        if cnn == option::Lq {
            break;
        }
        if cnn == 0 {
            i += 1; 
            continue;
        }
        if i + 1 >= options.len() {
            break;
        }
        let len = options[i + 1] as usize;
        if i + 2 + len > options.len() {
            break;
        }
        let val = &options[i + 2..i + 2 + len];

        match cnn {
            option::PF_ => {
                if len >= 1 { gii = val[0]; }
            }
            option::AJA_ => {
                if len >= 4 {
                    jag = Some([val[0], val[1], val[2], val[3]]);
                }
            }
            option::CQA_ => {
                
                if len >= 9 && &val[..9] == b"PXEClient" {
                    ert = true;
                }
            }
            _ => {}
        }

        i += 2 + len;
    }

    crate::serial_println!("[DHCPD] Received {} from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} (PXE: {})",
        match gii {
            1 => "DISCOVER",
            3 => "REQUEST",
            _ => "UNKNOWN",
        },
        aew[0], aew[1], aew[2],
        aew[3], aew[4], aew[5],
        ert);

    match gii {
        msg_type::Ro => {
            
            if let Some(offer_ip) = lwb(&aew) {
                fac(msg_type::Ps, &xid, &aew, &offer_ip, ert);
            } else {
                crate::serial_println!("[DHCPD] No IPs available in pool!");
            }
        }
        msg_type::Iq => {
            
            let ip = jag.unwrap_or_else(|| {
                
                lvw(&aew).unwrap_or([0; 4])
            });

            if ip == [0; 4] {
                fac(msg_type::Kf, &xid, &aew, &[0; 4], false);
                return;
            }

            
            if prb(&aew, &ip) {
                fac(msg_type::Dk, &xid, &aew, &ip, ert);
            } else {
                fac(msg_type::Kf, &xid, &aew, &[0; 4], false);
            }
        }
        _ => {}
    }
}


fn lwb(mac: &[u8; 6]) -> Option<[u8; 4]> {
    let mut agp = Mj.lock();
    let cfg = Gh.lock();

    
    for lease in agp.iter() {
        if lease.active && lease.mac == *mac {
            return Some(lease.ip);
        }
    }

    
    let pool_size = cfg.pool_size as usize;
    for offset in 0..pool_size.min(AGW_) {
        let fkr = [
            cfg.pool_base[0],
            cfg.pool_base[1],
            cfg.pool_base[2],
            cfg.pool_base[3].wrapping_add(offset as u8),
        ];

        
        let jvg = agp.iter().any(|l| l.active && l.ip == fkr);
        if !jvg {
            
            for lease in agp.iter_mut() {
                if !lease.active {
                    lease.mac = *mac;
                    lease.ip = fkr;
                    lease.active = true;
                    lease.granted_at = crate::time::uptime_ms();
                    AGE_.fetch_add(1, Ordering::Relaxed);
                    return Some(fkr);
                }
            }
        }
    }

    None
}


fn lvw(mac: &[u8; 6]) -> Option<[u8; 4]> {
    let agp = Mj.lock();
    for lease in agp.iter() {
        if lease.active && lease.mac == *mac {
            return Some(lease.ip);
        }
    }
    None
}


fn prb(mac: &[u8; 6], ip: &[u8; 4]) -> bool {
    let mut agp = Mj.lock();

    
    for lease in agp.iter_mut() {
        if lease.active && lease.mac == *mac && lease.ip == *ip {
            lease.granted_at = crate::time::uptime_ms();
            return true;
        }
    }

    
    for lease in agp.iter_mut() {
        if lease.active && lease.mac == *mac {
            
            let cfg = Gh.lock();
            let hgx = cfg.pool_base[3];
            let nwc = hgx.wrapping_add(cfg.pool_size);
            if ip[0] == cfg.pool_base[0] && ip[1] == cfg.pool_base[1]
                && ip[2] == cfg.pool_base[2]
                && ip[3] >= hgx && ip[3] < nwc
            {
                lease.ip = *ip;
                lease.granted_at = crate::time::uptime_ms();
                return true;
            }
            
            return false;
        }
    }

    false
}


fn fac(response_type: u8, xid: &[u8; 4], aew: &[u8; 6], client_ip: &[u8; 4], pxe: bool) {
    let cfg = Gh.lock();
    let mut be = Vec::with_capacity(400);

    
    be.push(2);                         
    be.push(1);                         
    be.push(6);                         
    be.push(0);                         
    be.extend_from_slice(xid);          
    be.extend_from_slice(&[0, 0]);      
    be.extend_from_slice(&[0x80, 0x00]); 
    be.extend_from_slice(&[0, 0, 0, 0]); 
    be.extend_from_slice(client_ip);    
    be.extend_from_slice(&cfg.server_ip); 
    be.extend_from_slice(&[0, 0, 0, 0]); 
    be.extend_from_slice(aew);   
    be.extend_from_slice(&[0u8; 10]);   
    
    
    if pxe {
        let ouc = format!("{}.{}.{}.{}",
            cfg.server_ip[0], cfg.server_ip[1],
            cfg.server_ip[2], cfg.server_ip[3]);
        let jgx = ouc.as_bytes();
        let jgn = jgx.len().min(63);
        be.extend_from_slice(&jgx[..jgn]);
        for _ in jgn..64 {
            be.push(0);
        }
    } else {
        be.extend_from_slice(&[0u8; 64]);
    }

    
    if pxe && cfg.boot_file_len > 0 {
        let hze = cfg.boot_file_len.min(127);
        be.extend_from_slice(&cfg.boot_file[..hze]);
        for _ in hze..128 {
            be.push(0);
        }
    } else {
        be.extend_from_slice(&[0u8; 128]);
    }

    
    be.extend_from_slice(&[99, 130, 83, 99]);

    
    
    be.extend_from_slice(&[option::PF_, 1, response_type]);

    
    be.extend_from_slice(&[option::YF_, 4]);
    be.extend_from_slice(&cfg.server_ip);

    if response_type != msg_type::Kf {
        
        be.extend_from_slice(&[option::VZ_, 4]);
        be.extend_from_slice(&86400u32.to_be_bytes());

        
        be.extend_from_slice(&[option::YS_, 4]);
        be.extend_from_slice(&cfg.subnet);

        
        be.extend_from_slice(&[option::Qa, 4]);
        be.extend_from_slice(&cfg.gateway);

        
        be.extend_from_slice(&[option::GX_, 4]);
        be.extend_from_slice(&cfg.server_ip);

        if pxe {
            
            let ook = format!("{}.{}.{}.{}",
                cfg.server_ip[0], cfg.server_ip[1],
                cfg.server_ip[2], cfg.server_ip[3]);
            let cv = ook.as_bytes();
            be.push(option::DBO_);
            be.push(cv.len() as u8);
            be.extend_from_slice(cv);

            
            if cfg.boot_file_len > 0 {
                be.push(option::BOA_);
                be.push(cfg.boot_file_len as u8);
                be.extend_from_slice(&cfg.boot_file[..cfg.boot_file_len]);
            }
        }
    }

    
    be.push(option::Lq);

    
    while be.len() < 300 {
        be.push(0);
    }

    drop(cfg);

    
    let src_ip = Gh.lock().server_ip;
    let ogl = match response_type {
        msg_type::Ps => "OFFER",
        msg_type::Dk => "ACK",
        msg_type::Kf => "NAK",
        _ => "?",
    };
    crate::serial_println!("[DHCPD] Sending {} to {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} -> {}.{}.{}.{} (PXE: {})",
        ogl,
        aew[0], aew[1], aew[2],
        aew[3], aew[4], aew[5],
        client_ip[0], client_ip[1], client_ip[2], client_ip[3],
        pxe);

    
    onm(&be, src_ip);
}


fn onm(payload: &[u8], src_ip: [u8; 4]) {
    
    let mut udp = Vec::with_capacity(8 + payload.len());
    udp.extend_from_slice(&67u16.to_be_bytes());  
    udp.extend_from_slice(&68u16.to_be_bytes());  
    udp.extend_from_slice(&((8 + payload.len()) as u16).to_be_bytes());
    udp.extend_from_slice(&0u16.to_be_bytes());   
    udp.extend_from_slice(payload);

    
    let mut ip = Vec::with_capacity(20 + udp.len());
    ip.push(0x45); ip.push(0x10); 
    ip.extend_from_slice(&((20 + udp.len()) as u16).to_be_bytes());
    ip.extend_from_slice(&[0, 0, 0x40, 0x00]); 
    ip.push(64); ip.push(17); 
    ip.extend_from_slice(&0u16.to_be_bytes()); 
    ip.extend_from_slice(&src_ip);
    ip.extend_from_slice(&[255, 255, 255, 255]); 

    
    let mut sum: u32 = 0;
    for i in (0..20).step_by(2) {
        sum += ((ip[i] as u32) << 8) | (ip[i + 1] as u32);
    }
    while sum >> 16 != 0 { sum = (sum & 0xFFFF) + (sum >> 16); }
    let ig = !(sum as u16);
    ip[10] = (ig >> 8) as u8;
    ip[11] = (ig & 0xFF) as u8;

    ip.extend_from_slice(&udp);

    
    let _ = crate::netstack::cdq([0xFF; 6], crate::netstack::ethertype::Tb, &ip);
}


pub fn mdi() -> Vec<([u8; 6], [u8; 4], u64)> {
    let agp = Mj.lock();
    let mut result = Vec::new();
    for lease in agp.iter() {
        if lease.active {
            result.push((lease.mac, lease.ip, lease.granted_at));
        }
    }
    result
}
