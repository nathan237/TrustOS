



use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;


static ACR_: AtomicBool = AtomicBool::new(false);


pub fn crf() {
    ACR_.store(true, Ordering::SeqCst);
}


pub fn resume() {
    ACR_.store(false, Ordering::SeqCst);
}


pub fn mtt() -> bool {
    ACR_.load(Ordering::Relaxed)
}


mod msg_type {
    pub const Ro: u8 = 1;
    pub const Ps: u8 = 2;
    pub const Iq: u8 = 3;
    pub const Dk: u8 = 5;
    pub const Kf: u8 = 6;
}


mod option {
    pub const Dx: u8 = 0;
    pub const YS_: u8 = 1;
    pub const Qa: u8 = 3;
    pub const GX_: u8 = 6;
    pub const Zo: u8 = 12;
    pub const AJA_: u8 = 50;
    pub const VZ_: u8 = 51;
    pub const PF_: u8 = 53;
    pub const YF_: u8 = 54;
    pub const CMB_: u8 = 55;
    pub const Lq: u8 = 255;
}


#[derive(Debug, Clone, Copy, PartialEq)]
enum DhcpState {
    Init,
    Selecting,
    Requesting,
    Bound,
    Renewing,
    Rebinding,
}


struct Om {
    state: DhcpState,
    xid: u32,
    offered_ip: [u8; 4],
    server_ip: [u8; 4],
    subnet_mask: [u8; 4],
    gateway: [u8; 4],
    dns_server: [u8; 4],
    lease_time: u32,
    bound_time: u64,
    last_send: u64,
    retries: u8,
}

static Ec: Mutex<Om> = Mutex::new(Om {
    state: DhcpState::Init,
    xid: 0x12345678,
    offered_ip: [0; 4],
    server_ip: [0; 4],
    subnet_mask: [255, 255, 255, 0],
    gateway: [0; 4],
    dns_server: [8, 8, 8, 8],
    lease_time: 0,
    bound_time: 0,
    last_send: 0,
    retries: 0,
});

static Cq: AtomicBool = AtomicBool::new(false);
static Jj: AtomicBool = AtomicBool::new(false);


pub fn start() {
    Cq.store(true, Ordering::SeqCst);
    Jj.store(false, Ordering::SeqCst);
    
    let mut oh = Ec.lock();
    oh.state = DhcpState::Init;
    oh.xid = dqm();
    oh.retries = 0;
    drop(oh);
    
    crate::log!("[DHCP] Client started");
    let _ = dzb();
}


pub fn clk() -> bool {
    Jj.load(Ordering::Relaxed)
}


pub fn ibj() -> Option<([u8; 4], [u8; 4], [u8; 4], [u8; 4])> {
    if !clk() { return None; }
    let oh = Ec.lock();
    Some((oh.offered_ip, oh.subnet_mask, oh.gateway, oh.dns_server))
}

fn dqm() -> u32 {
    let gx = crate::logger::eg() as u32;
    let mac = crate::drivers::net::aqt().unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    gx ^ ((mac[4] as u32) << 8) ^ (mac[5] as u32)
}

fn cun(msg_type: u8, oh: &Om) -> Vec<u8> {
    fkd(msg_type, oh, [0u8; 4])
}

fn fkd(msg_type: u8, oh: &Om, cvc: [u8; 4]) -> Vec<u8> {
    let mac = crate::drivers::net::aqt().unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    let mut be = Vec::with_capacity(300);
    
    
    be.push(1); be.push(1); be.push(6); be.push(0);
    be.extend_from_slice(&oh.xid.to_be_bytes());
    be.extend_from_slice(&0u16.to_be_bytes());
    be.extend_from_slice(&0x8000u16.to_be_bytes());
    be.extend_from_slice(&cvc); 
    be.extend_from_slice(&[0u8; 12]); 
    be.extend_from_slice(&mac);
    be.extend_from_slice(&[0u8; 10 + 64 + 128]); 
    be.extend_from_slice(&[99, 130, 83, 99]); 
    
    
    be.extend_from_slice(&[option::PF_, 1, msg_type]);
    
    if msg_type == msg_type::Iq && oh.offered_ip != [0; 4] {
        
        
        if oh.state != DhcpState::Renewing && oh.state != DhcpState::Rebinding {
            be.extend_from_slice(&[option::AJA_, 4]);
            be.extend_from_slice(&oh.offered_ip);
            if oh.server_ip != [0; 4] {
                be.extend_from_slice(&[option::YF_, 4]);
                be.extend_from_slice(&oh.server_ip);
            }
        }
    }
    
    be.extend_from_slice(&[option::CMB_, 4, option::YS_, option::Qa, option::GX_, option::VZ_]);
    be.extend_from_slice(&[option::Zo, 7]);
    be.extend_from_slice(b"trustos");
    be.push(option::Lq);
    
    while be.len() < 300 { be.push(0); }
    be
}

fn dzb() -> Result<(), &'static str> {
    let mut oh = Ec.lock();
    oh.state = DhcpState::Selecting;
    oh.last_send = crate::logger::eg();
    let be = cun(msg_type::Ro, &oh);
    drop(oh);
    crate::serial_println!("[DHCP] Sending DISCOVER");
    gtv(&be)
}

fn bos() -> Result<(), &'static str> {
    let mut oh = Ec.lock();
    oh.state = DhcpState::Requesting;
    oh.last_send = crate::logger::eg();
    let be = cun(msg_type::Iq, &oh);
    drop(oh);
    crate::serial_println!("[DHCP] Sending REQUEST");
    gtv(&be)
}


fn jen() -> Result<(), &'static str> {
    let mut oh = Ec.lock();
    oh.state = DhcpState::Renewing;
    oh.xid = dqm();
    oh.last_send = crate::logger::eg();
    let cvc = oh.offered_ip;
    let ain = oh.server_ip;
    let be = fkd(msg_type::Iq, &oh, cvc);
    drop(oh);
    crate::serial_println!("[DHCP] Sending RENEW (unicast to {}.{}.{}.{})", ain[0], ain[1], ain[2], ain[3]);
    onn(&be, cvc, ain)
}


fn gtz() -> Result<(), &'static str> {
    let mut oh = Ec.lock();
    oh.state = DhcpState::Rebinding;
    oh.xid = dqm();
    oh.last_send = crate::logger::eg();
    let cvc = oh.offered_ip;
    let be = fkd(msg_type::Iq, &oh, cvc);
    drop(oh);
    crate::serial_println!("[DHCP] Sending REBIND (broadcast)");
    gtv(&be)
}

fn gtv(payload: &[u8]) -> Result<(), &'static str> {
    jek(payload, [0, 0, 0, 0], [255, 255, 255, 255], [0xFF; 6])
}


fn onn(payload: &[u8], src_ip: [u8; 4], dst_ip: [u8; 4]) -> Result<(), &'static str> {
    
    let dst_mac = crate::netstack::arp::yb(dst_ip).unwrap_or([0xFF; 6]);
    jek(payload, src_ip, dst_ip, dst_mac)
}

fn jek(payload: &[u8], src_ip: [u8; 4], dst_ip: [u8; 4], dst_mac: [u8; 6]) -> Result<(), &'static str> {
    let mut udp = Vec::with_capacity(8 + payload.len());
    udp.extend_from_slice(&68u16.to_be_bytes()); 
    udp.extend_from_slice(&67u16.to_be_bytes()); 
    udp.extend_from_slice(&((8 + payload.len()) as u16).to_be_bytes());
    udp.extend_from_slice(&0u16.to_be_bytes());
    udp.extend_from_slice(payload);
    
    let mut ip = Vec::with_capacity(20 + udp.len());
    ip.push(0x45); ip.push(0);
    ip.extend_from_slice(&((20 + udp.len()) as u16).to_be_bytes());
    ip.extend_from_slice(&[0, 0, 0, 0]); 
    ip.push(64); ip.push(17); 
    ip.extend_from_slice(&0u16.to_be_bytes()); 
    ip.extend_from_slice(&src_ip);
    ip.extend_from_slice(&dst_ip);
    
    let mut sum: u32 = 0;
    for i in (0..20).step_by(2) { sum += ((ip[i] as u32) << 8) | (ip[i + 1] as u32); }
    while sum >> 16 != 0 { sum = (sum & 0xFFFF) + (sum >> 16); }
    let ig = !(sum as u16);
    ip[10] = (ig >> 8) as u8; ip[11] = (ig & 0xFF) as u8;
    ip.extend_from_slice(&udp);
    
    crate::netstack::cdq(dst_mac, crate::netstack::ethertype::Tb, &ip)
}

pub fn alq(data: &[u8]) {
    if !Cq.load(Ordering::Relaxed) || data.len() < 240 { return; }
    if data[0] != 2 { return; } 
    
    let xid = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    { let c = Ec.lock(); if xid != c.xid { return; } }
    
    let akt = [data[16], data[17], data[18], data[19]];
    let osi = [data[20], data[21], data[22], data[23]];
    if data[236..240] != [99, 130, 83, 99] { return; }
    
    let (mut msg_type, mut subnet, mut fz, mut dns, mut server_id, mut lease) = 
        (0u8, [255,255,255,0], [0u8;4], [8,8,8,8], osi, 86400u32);
    
    let mut i = 240;
    while i < data.len() {
        let cnn = data[i];
        if cnn == option::Lq { break; }
        if cnn == option::Dx { i += 1; continue; }
        if i + 1 >= data.len() { break; }
        let len = data[i + 1] as usize;
        if i + 2 + len > data.len() { break; }
        let v = &data[i + 2..i + 2 + len];
        match cnn {
            option::PF_ if len >= 1 => msg_type = v[0],
            option::YS_ if len >= 4 => subnet = [v[0], v[1], v[2], v[3]],
            option::Qa if len >= 4 => fz = [v[0], v[1], v[2], v[3]],
            option::GX_ if len >= 4 => dns = [v[0], v[1], v[2], v[3]],
            option::YF_ if len >= 4 => server_id = [v[0], v[1], v[2], v[3]],
            option::VZ_ if len >= 4 => lease = u32::from_be_bytes([v[0], v[1], v[2], v[3]]),
            _ => {}
        }
        i += 2 + len;
    }
    
    match msg_type {
        msg_type::Ps => {
            crate::serial_println!("[DHCP] OFFER: {}.{}.{}.{}", akt[0], akt[1], akt[2], akt[3]);
            let mut c = Ec.lock();
            if c.state == DhcpState::Selecting {
                c.offered_ip = akt; c.server_ip = server_id;
                c.subnet_mask = subnet; c.gateway = fz; c.dns_server = dns;
                drop(c);
                let _ = bos();
            }
        }
        msg_type::Dk => {
            crate::log!("[DHCP] ACK: {}.{}.{}.{} (lease={}s)", akt[0], akt[1], akt[2], akt[3], lease);
            
            
            if mtt() {
                crate::serial_println!("[DHCP] Suspended - ignoring ACK");
                return;
            }
            
            let mut c = Ec.lock();
            c.state = DhcpState::Bound; c.offered_ip = akt;
            c.subnet_mask = subnet; c.gateway = fz; c.dns_server = dns; c.lease_time = lease;
            c.bound_time = crate::logger::eg();
            let ip = crate::network::Ipv4Address::new(akt[0], akt[1], akt[2], akt[3]);
            let mask = crate::network::Ipv4Address::new(subnet[0], subnet[1], subnet[2], subnet[3]);
            let mgu = crate::network::Ipv4Address::new(fz[0], fz[1], fz[2], fz[3]);
            crate::network::deh(ip, mask, Some(mgu));
            
            crate::network::oou(dns);
            drop(c);
            Jj.store(true, Ordering::SeqCst);
            crate::log!("[DHCP] Configured: IP={}.{}.{}.{} GW={}.{}.{}.{} DNS={}.{}.{}.{}", 
                akt[0], akt[1], akt[2], akt[3], 
                fz[0], fz[1], fz[2], fz[3],
                dns[0], dns[1], dns[2], dns[3]);
        }
        msg_type::Kf => {
            crate::log_warn!("[DHCP] NAK, restarting");
            Ec.lock().state = DhcpState::Init;
            Jj.store(false, Ordering::SeqCst);
            let _ = dzb();
        }
        _ => {}
    }
}

pub fn poll() {
    if !Cq.load(Ordering::Relaxed) { return; }
    let cy = crate::logger::eg();
    let mut c = Ec.lock();
    
    match c.state {
        DhcpState::Bound => {
            
            let elapsed_ms = cy.saturating_sub(c.bound_time);
            let daj = (c.lease_time as u64) * 1000;
            let ll = daj / 2;        
            let np = daj * 7 / 8;    
            
            if elapsed_ms >= np {
                crate::serial_println!("[DHCP] T2 expired, rebinding");
                drop(c);
                let _ = gtz();
            } else if elapsed_ms >= ll {
                crate::serial_println!("[DHCP] T1 expired, renewing");
                drop(c);
                let _ = jen();
            }
        }
        DhcpState::Renewing => {
            
            let elapsed_ms = cy.saturating_sub(c.bound_time);
            let daj = (c.lease_time as u64) * 1000;
            let np = daj * 7 / 8;
            
            if elapsed_ms >= np {
                crate::serial_println!("[DHCP] Renew failed, rebinding");
                drop(c);
                let _ = gtz();
            } else if cy.saturating_sub(c.last_send) > 30_000 {
                drop(c);
                let _ = jen();
            }
        }
        DhcpState::Rebinding => {
            
            let elapsed_ms = cy.saturating_sub(c.bound_time);
            let daj = (c.lease_time as u64) * 1000;
            
            if elapsed_ms >= daj {
                crate::log_warn!("[DHCP] Lease expired, restarting");
                c.state = DhcpState::Init;
                c.retries = 0;
                drop(c);
                Jj.store(false, Ordering::SeqCst);
                let _ = dzb();
            } else if cy.saturating_sub(c.last_send) > 30_000 {
                drop(c);
                let _ = gtz();
            }
        }
        DhcpState::Selecting | DhcpState::Requesting => {
            let mz = if c.state == DhcpState::Init { 1000 } else { 3000 };
            if cy.saturating_sub(c.last_send) > mz {
                c.retries += 1;
                if c.retries > 5 { c.state = DhcpState::Init; c.xid = dqm(); c.retries = 0; }
                let state = c.state;
                drop(c);
                match state {
                    DhcpState::Init | DhcpState::Selecting => { let _ = dzb(); }
                    DhcpState::Requesting => { let _ = bos(); }
                    _ => {}
                }
            }
        }
        DhcpState::Init => {
            if cy.saturating_sub(c.last_send) > 1000 {
                c.retries += 1;
                if c.retries > 10 {
                    
                    drop(c);
                    Cq.store(false, Ordering::SeqCst);
                    crate::log!("[DHCP] No server found, applying fallback 10.0.0.2/24");
                    crate::network::opd(
                        crate::network::Ipv4Address::new(10, 0, 0, 2),
                        crate::network::Ipv4Address::new(255, 255, 255, 0),
                        crate::network::Ipv4Address::new(10, 0, 0, 1),
                    );
                    Jj.store(true, Ordering::SeqCst);
                    return;
                }
                if c.retries > 5 { c.xid = dqm(); }
                drop(c);
                let _ = dzb();
            }
        }
    }
}
