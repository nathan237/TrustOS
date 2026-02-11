//! DHCP Client Implementation
//!
//! Implements DHCP (RFC 2131) for automatic IP configuration.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

/// Flag to suspend DHCP updates (used during downloads to prevent IP changes)
static DHCP_SUSPENDED: AtomicBool = AtomicBool::new(false);

/// Suspend DHCP updates (prevents IP changes during network operations)
pub fn suspend() {
    DHCP_SUSPENDED.store(true, Ordering::SeqCst);
}

/// Resume DHCP updates
pub fn resume() {
    DHCP_SUSPENDED.store(false, Ordering::SeqCst);
}

/// Check if DHCP is suspended
pub fn is_suspended() -> bool {
    DHCP_SUSPENDED.load(Ordering::Relaxed)
}

/// DHCP message types
mod msg_type {
    pub const DISCOVER: u8 = 1;
    pub const OFFER: u8 = 2;
    pub const REQUEST: u8 = 3;
    pub const ACK: u8 = 5;
    pub const NAK: u8 = 6;
}

/// DHCP option codes
mod option {
    pub const PAD: u8 = 0;
    pub const SUBNET_MASK: u8 = 1;
    pub const ROUTER: u8 = 3;
    pub const DNS_SERVER: u8 = 6;
    pub const HOSTNAME: u8 = 12;
    pub const REQUESTED_IP: u8 = 50;
    pub const LEASE_TIME: u8 = 51;
    pub const MSG_TYPE: u8 = 53;
    pub const SERVER_ID: u8 = 54;
    pub const PARAM_REQUEST: u8 = 55;
    pub const END: u8 = 255;
}

/// DHCP state machine
#[derive(Debug, Clone, Copy, PartialEq)]
enum DhcpState {
    Init,
    Selecting,
    Requesting,
    Bound,
}

/// DHCP client state
struct DhcpClient {
    state: DhcpState,
    xid: u32,
    offered_ip: [u8; 4],
    server_ip: [u8; 4],
    subnet_mask: [u8; 4],
    gateway: [u8; 4],
    dns_server: [u8; 4],
    lease_time: u32,
    last_send: u64,
    retries: u8,
}

static CLIENT: Mutex<DhcpClient> = Mutex::new(DhcpClient {
    state: DhcpState::Init,
    xid: 0x12345678,
    offered_ip: [0; 4],
    server_ip: [0; 4],
    subnet_mask: [255, 255, 255, 0],
    gateway: [0; 4],
    dns_server: [8, 8, 8, 8],
    lease_time: 0,
    last_send: 0,
    retries: 0,
});

static ENABLED: AtomicBool = AtomicBool::new(false);
static BOUND: AtomicBool = AtomicBool::new(false);

/// Start DHCP client
pub fn start() {
    ENABLED.store(true, Ordering::SeqCst);
    BOUND.store(false, Ordering::SeqCst);
    
    let mut client = CLIENT.lock();
    client.state = DhcpState::Init;
    client.xid = generate_xid();
    client.retries = 0;
    drop(client);
    
    crate::log!("[DHCP] Client started");
    let _ = send_discover();
}

/// Check if we have a valid lease
pub fn is_bound() -> bool {
    BOUND.load(Ordering::Relaxed)
}

/// Get assigned IP configuration
pub fn get_config() -> Option<([u8; 4], [u8; 4], [u8; 4], [u8; 4])> {
    if !is_bound() { return None; }
    let client = CLIENT.lock();
    Some((client.offered_ip, client.subnet_mask, client.gateway, client.dns_server))
}

fn generate_xid() -> u32 {
    let ticks = crate::logger::get_ticks() as u32;
    let mac = crate::drivers::net::get_mac().unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    ticks ^ ((mac[4] as u32) << 8) ^ (mac[5] as u32)
}

fn build_packet(msg_type: u8, client: &DhcpClient) -> Vec<u8> {
    let mac = crate::drivers::net::get_mac().unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    let mut packet = Vec::with_capacity(300);
    
    // BOOTP header
    packet.push(1); packet.push(1); packet.push(6); packet.push(0);
    packet.extend_from_slice(&client.xid.to_be_bytes());
    packet.extend_from_slice(&0u16.to_be_bytes());
    packet.extend_from_slice(&0x8000u16.to_be_bytes());
    packet.extend_from_slice(&[0u8; 16]); // ciaddr, yiaddr, siaddr, giaddr
    packet.extend_from_slice(&mac);
    packet.extend_from_slice(&[0u8; 10 + 64 + 128]); // padding + sname + file
    packet.extend_from_slice(&[99, 130, 83, 99]); // magic cookie
    
    // Options
    packet.extend_from_slice(&[option::MSG_TYPE, 1, msg_type]);
    
    if msg_type == msg_type::REQUEST && client.offered_ip != [0; 4] {
        packet.extend_from_slice(&[option::REQUESTED_IP, 4]);
        packet.extend_from_slice(&client.offered_ip);
        if client.server_ip != [0; 4] {
            packet.extend_from_slice(&[option::SERVER_ID, 4]);
            packet.extend_from_slice(&client.server_ip);
        }
    }
    
    packet.extend_from_slice(&[option::PARAM_REQUEST, 4, option::SUBNET_MASK, option::ROUTER, option::DNS_SERVER, option::LEASE_TIME]);
    packet.extend_from_slice(&[option::HOSTNAME, 7]);
    packet.extend_from_slice(b"trustos");
    packet.push(option::END);
    
    while packet.len() < 300 { packet.push(0); }
    packet
}

fn send_discover() -> Result<(), &'static str> {
    let mut client = CLIENT.lock();
    client.state = DhcpState::Selecting;
    client.last_send = crate::logger::get_ticks();
    let packet = build_packet(msg_type::DISCOVER, &client);
    drop(client);
    crate::serial_println!("[DHCP] Sending DISCOVER");
    send_dhcp_packet(&packet)
}

fn send_request() -> Result<(), &'static str> {
    let mut client = CLIENT.lock();
    client.state = DhcpState::Requesting;
    client.last_send = crate::logger::get_ticks();
    let packet = build_packet(msg_type::REQUEST, &client);
    drop(client);
    crate::serial_println!("[DHCP] Sending REQUEST");
    send_dhcp_packet(&packet)
}

fn send_dhcp_packet(payload: &[u8]) -> Result<(), &'static str> {
    let mut udp = Vec::with_capacity(8 + payload.len());
    udp.extend_from_slice(&68u16.to_be_bytes()); // src port
    udp.extend_from_slice(&67u16.to_be_bytes()); // dst port
    udp.extend_from_slice(&((8 + payload.len()) as u16).to_be_bytes());
    udp.extend_from_slice(&0u16.to_be_bytes());
    udp.extend_from_slice(payload);
    
    let mut ip = Vec::with_capacity(20 + udp.len());
    ip.push(0x45); ip.push(0);
    ip.extend_from_slice(&((20 + udp.len()) as u16).to_be_bytes());
    ip.extend_from_slice(&[0, 0, 0, 0]); // id + flags
    ip.push(64); ip.push(17); // TTL, protocol
    ip.extend_from_slice(&0u16.to_be_bytes()); // checksum
    ip.extend_from_slice(&[0, 0, 0, 0]); // src
    ip.extend_from_slice(&[255, 255, 255, 255]); // dst
    
    let mut sum: u32 = 0;
    for i in (0..20).step_by(2) { sum += ((ip[i] as u32) << 8) | (ip[i + 1] as u32); }
    while sum >> 16 != 0 { sum = (sum & 0xFFFF) + (sum >> 16); }
    let csum = !(sum as u16);
    ip[10] = (csum >> 8) as u8; ip[11] = (csum & 0xFF) as u8;
    ip.extend_from_slice(&udp);
    
    crate::netstack::send_frame([0xFF; 6], crate::netstack::ethertype::IPV4, &ip)
}

pub fn handle_packet(data: &[u8]) {
    if !ENABLED.load(Ordering::Relaxed) || data.len() < 240 { return; }
    if data[0] != 2 { return; } // BOOTREPLY
    
    let xid = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    { let c = CLIENT.lock(); if xid != c.xid { return; } }
    
    let yiaddr = [data[16], data[17], data[18], data[19]];
    let siaddr = [data[20], data[21], data[22], data[23]];
    if data[236..240] != [99, 130, 83, 99] { return; }
    
    let (mut msg_type, mut subnet, mut gw, mut dns, mut server_id, mut lease) = 
        (0u8, [255,255,255,0], [0u8;4], [8,8,8,8], siaddr, 86400u32);
    
    let mut i = 240;
    while i < data.len() {
        let opt = data[i];
        if opt == option::END { break; }
        if opt == option::PAD { i += 1; continue; }
        if i + 1 >= data.len() { break; }
        let len = data[i + 1] as usize;
        if i + 2 + len > data.len() { break; }
        let v = &data[i + 2..i + 2 + len];
        match opt {
            option::MSG_TYPE if len >= 1 => msg_type = v[0],
            option::SUBNET_MASK if len >= 4 => subnet = [v[0], v[1], v[2], v[3]],
            option::ROUTER if len >= 4 => gw = [v[0], v[1], v[2], v[3]],
            option::DNS_SERVER if len >= 4 => dns = [v[0], v[1], v[2], v[3]],
            option::SERVER_ID if len >= 4 => server_id = [v[0], v[1], v[2], v[3]],
            option::LEASE_TIME if len >= 4 => lease = u32::from_be_bytes([v[0], v[1], v[2], v[3]]),
            _ => {}
        }
        i += 2 + len;
    }
    
    match msg_type {
        msg_type::OFFER => {
            crate::serial_println!("[DHCP] OFFER: {}.{}.{}.{}", yiaddr[0], yiaddr[1], yiaddr[2], yiaddr[3]);
            let mut c = CLIENT.lock();
            if c.state == DhcpState::Selecting {
                c.offered_ip = yiaddr; c.server_ip = server_id;
                c.subnet_mask = subnet; c.gateway = gw; c.dns_server = dns;
                drop(c);
                let _ = send_request();
            }
        }
        msg_type::ACK => {
            crate::log!("[DHCP] ACK: {}.{}.{}.{} (lease={}s)", yiaddr[0], yiaddr[1], yiaddr[2], yiaddr[3], lease);
            
            // Don't apply DHCP config if suspended (e.g., during download)
            if is_suspended() {
                crate::serial_println!("[DHCP] Suspended - ignoring ACK");
                return;
            }
            
            let mut c = CLIENT.lock();
            c.state = DhcpState::Bound; c.offered_ip = yiaddr;
            c.subnet_mask = subnet; c.gateway = gw; c.dns_server = dns; c.lease_time = lease;
            let ip = crate::network::Ipv4Address::new(yiaddr[0], yiaddr[1], yiaddr[2], yiaddr[3]);
            let mask = crate::network::Ipv4Address::new(subnet[0], subnet[1], subnet[2], subnet[3]);
            let gwaddr = crate::network::Ipv4Address::new(gw[0], gw[1], gw[2], gw[3]);
            crate::network::set_ipv4_config(ip, mask, Some(gwaddr));
            // Update global DNS server from DHCP
            crate::network::set_dns_server(dns);
            drop(c);
            BOUND.store(true, Ordering::SeqCst);
            crate::log!("[DHCP] Configured: IP={}.{}.{}.{} GW={}.{}.{}.{} DNS={}.{}.{}.{}", 
                yiaddr[0], yiaddr[1], yiaddr[2], yiaddr[3], 
                gw[0], gw[1], gw[2], gw[3],
                dns[0], dns[1], dns[2], dns[3]);
        }
        msg_type::NAK => {
            crate::log_warn!("[DHCP] NAK, restarting");
            CLIENT.lock().state = DhcpState::Init;
            BOUND.store(false, Ordering::SeqCst);
            let _ = send_discover();
        }
        _ => {}
    }
}

pub fn poll() {
    if !ENABLED.load(Ordering::Relaxed) { return; }
    let now = crate::logger::get_ticks();
    let mut c = CLIENT.lock();
    let timeout = match c.state { DhcpState::Selecting | DhcpState::Requesting => 3000, DhcpState::Init => 1000, _ => return };
    if now.saturating_sub(c.last_send) > timeout {
        c.retries += 1;
        if c.retries > 5 { c.state = DhcpState::Init; c.xid = generate_xid(); c.retries = 0; }
        let state = c.state;
        drop(c);
        match state {
            DhcpState::Init | DhcpState::Selecting => { let _ = send_discover(); }
            DhcpState::Requesting => { let _ = send_request(); }
            _ => {}
        }
    }
}
