//! DHCP Server with PXE Boot Support
//!
//! Implements a minimal DHCP server (RFC 2131) that responds to PXE boot
//! requests, allowing TrustOS to self-replicate across the network.
//! Assigns IPs from a pool and provides PXE options (next-server, boot-file).

use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use spin::Mutex;

/// DHCP server running flag
static RUNNING: AtomicBool = AtomicBool::new(false);

/// Number of leases currently active
static LEASE_COUNT: AtomicU8 = AtomicU8::new(0);

/// Maximum clients we can serve
const MAXIMUM_LEASES: usize = 16;

/// DHCP lease entry
#[derive(Clone, Copy)]
struct Lease {
    mac: [u8; 6],
    ip: [u8; 4],
    active: bool,
    granted_at: u64,
    lease_time: u32,
}

// Implementation block — defines methods for the type above.
impl Lease {
    const fn empty() -> Self {
        Self {
            mac: [0; 6],
            ip: [0; 4],
            active: false,
            granted_at: 0,
            lease_time: 86400, // 24h default
        }
    }
}

/// PXE boot configuration
#[derive(Clone, Copy)]
struct PxeConfig {
    /// Server IP (our IP) — used as TFTP next-server
    server_ip: [u8; 4],
    /// Subnet mask for clients
    subnet: [u8; 4],
    /// Gateway (usually us)
    gateway: [u8; 4],
    /// Base IP for pool (last octet increments)
    pool_base: [u8; 4],
    /// Number of IPs in pool
    pool_size: u8,
    /// PXE boot filename
    boot_file: [u8; 128],
    boot_file_length: usize,
}

// Implementation block — defines methods for the type above.
impl PxeConfig {
    const fn default() -> Self {
        Self {
            server_ip: [10, 0, 2, 1],
            subnet: [255, 255, 255, 0],
            gateway: [10, 0, 2, 1],
            pool_base: [10, 0, 2, 100],
            pool_size: 16,
            boot_file: [0; 128],
            boot_file_length: 0,
        }
    }
}

// Global shared state guarded by a Mutex (mutual exclusion lock).
static LEASES: Mutex<[Lease; MAXIMUM_LEASES]> = Mutex::new([Lease::empty(); MAXIMUM_LEASES]);
// Global shared state guarded by a Mutex (mutual exclusion lock).
static CONFIG: Mutex<PxeConfig> = Mutex::new(PxeConfig::default());

/// DHCP message types (same as client)
mod msg_type {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DISCOVER: u8 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OFFER: u8 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const REQUEST: u8 = 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const _DECLINE: u8 = 4;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ACK: u8 = 5;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const NAK: u8 = 6;
}

/// DHCP option codes
mod option {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SUBNET_MASK: u8 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ROUTER: u8 = 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DNS_SERVER: u8 = 6;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HOSTNAME: u8 = 12;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const REQUESTED_IP: u8 = 50;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const LEASE_TIME: u8 = 51;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const MESSAGE_TYPE: u8 = 53;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SERVER_ID: u8 = 54;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const TFTP_SERVER_NAME: u8 = 66;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const BOOT_FILE_NAME: u8 = 67;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PXE_VENDOR_CLASS: u8 = 60;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const END: u8 = 255;
}

/// Check if DHCP server is running
pub fn is_running() -> bool {
    RUNNING.load(Ordering::Relaxed)
}

/// Get number of active leases
pub fn active_leases() -> u8 {
    LEASE_COUNT.load(Ordering::Relaxed)
}

/// Start the DHCP/PXE server
pub fn start(server_ip: [u8; 4], subnet: [u8; 4], pool_start: [u8; 4], pool_size: u8, boot_filename: &str) {
    if RUNNING.load(Ordering::Relaxed) {
        crate::serial_println!("[DHCPD] Already running");
        return;
    }

    let mut cfg = CONFIG.lock();
    cfg.server_ip = server_ip;
    cfg.subnet = subnet;
    cfg.gateway = server_ip; // We are the gateway
    cfg.pool_base = pool_start;
    cfg.pool_size = pool_size;

    // Copy boot filename
    let bytes = boot_filename.as_bytes();
    let len = bytes.len().minimum(127);
    cfg.boot_file[..len].copy_from_slice(&bytes[..len]);
    cfg.boot_file_length = len;
    drop(cfg);

    // Clear old leases
    let mut leases = LEASES.lock();
    for l in leases.iterator_mut() {
        *l = Lease::empty();
    }
    drop(leases);
    LEASE_COUNT.store(0, Ordering::Relaxed);

    RUNNING.store(true, Ordering::Relaxed);
    crate::serial_println!("[DHCPD] PXE DHCP server started on {}.{}.{}.{}",
        server_ip[0], server_ip[1], server_ip[2], server_ip[3]);
    crate::serial_println!("[DHCPD] Pool: {}.{}.{}.{} - {}.{}.{}.{} ({} IPs)",
        pool_start[0], pool_start[1], pool_start[2], pool_start[3],
        pool_start[0], pool_start[1], pool_start[2], pool_start[3] + pool_size - 1,
        pool_size);
    crate::serial_println!("[DHCPD] PXE boot file: {}", boot_filename);
}

/// Stop the DHCP server
pub fn stop() {
    RUNNING.store(false, Ordering::Relaxed);
    crate::serial_println!("[DHCPD] Server stopped");
}

/// Handle incoming DHCP packet on port 67 (server port)
/// Called from udp.rs when a packet arrives on port 67
pub fn handle_packet(data: &[u8]) {
    if !RUNNING.load(Ordering::Relaxed) {
        return;
    }

    // Minimum BOOTP packet size: 236 bytes + 4 bytes magic cookie
    if data.len() < 240 {
        return;
    }

    // Only handle BOOTREQUEST (op=1)
    if data[0] != 1 {
        return;
    }

    let xid = [data[4], data[5], data[6], data[7]];
    let client_mac = [data[28], data[29], data[30], data[31], data[32], data[33]];

    // Check magic cookie (99, 130, 83, 99)
    if data[236] != 99 || data[237] != 130 || data[238] != 83 || data[239] != 99 {
        return;
    }

    // Parse options to find message type and requested IP
    let options = &data[240..];
    let mut message_type_value: u8 = 0;
    let mut requested_ip: Option<[u8; 4]> = None;
    let mut is_pxe = false;
    let mut i = 0;

    while i < options.len() {
        let opt = options[i];
        if opt == option::END {
            break;
        }
        if opt == 0 {
            i += 1; // PAD
            continue;
        }
        if i + 1 >= options.len() {
            break;
        }
        let len = options[i + 1] as usize;
        if i + 2 + len > options.len() {
            break;
        }
        let value = &options[i + 2..i + 2 + len];

                // Pattern matching — Rust's exhaustive branching construct.
match opt {
            option::MESSAGE_TYPE => {
                if len >= 1 { message_type_value = value[0]; }
            }
            option::REQUESTED_IP => {
                if len >= 4 {
                    requested_ip = Some([value[0], value[1], value[2], value[3]]);
                }
            }
            option::PXE_VENDOR_CLASS => {
                // Client is PXE if vendor class starts with "PXEClient"
                if len >= 9 && &value[..9] == b"PXEClient" {
                    is_pxe = true;
                }
            }
            _ => {}
        }

        i += 2 + len;
    }

    crate::serial_println!("[DHCPD] Received {} from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} (PXE: {})",
                // Pattern matching — Rust's exhaustive branching construct.
match message_type_value {
            1 => "DISCOVER",
            3 => "REQUEST",
            _ => "UNKNOWN",
        },
        client_mac[0], client_mac[1], client_mac[2],
        client_mac[3], client_mac[4], client_mac[5],
        is_pxe);

        // Pattern matching — Rust's exhaustive branching construct.
match message_type_value {
        msg_type::DISCOVER => {
            // Allocate an IP and send OFFER
            if let Some(offer_ip) = find_or_allocate_ip(&client_mac) {
                send_response(msg_type::OFFER, &xid, &client_mac, &offer_ip, is_pxe);
            } else {
                crate::serial_println!("[DHCPD] No IPs available in pool!");
            }
        }
        msg_type::REQUEST => {
            // Confirm the requested IP
            let ip = requested_ip.unwrap_or_else(|| {
                // If no requested IP, check if we have an existing lease
                find_lease_by_mac(&client_mac).unwrap_or([0; 4])
            });

            if ip == [0; 4] {
                send_response(msg_type::NAK, &xid, &client_mac, &[0; 4], false);
                return;
            }

            // Validate the IP is in our pool and assigned to this MAC
            if validate_and_confirm_lease(&client_mac, &ip) {
                send_response(msg_type::ACK, &xid, &client_mac, &ip, is_pxe);
            } else {
                send_response(msg_type::NAK, &xid, &client_mac, &[0; 4], false);
            }
        }
        _ => {}
    }
}

/// Find existing lease for MAC or allocate new IP from pool
fn find_or_allocate_ip(mac: &[u8; 6]) -> Option<[u8; 4]> {
    let mut leases = LEASES.lock();
    let cfg = CONFIG.lock();

    // Check for existing lease
    for lease in leases.iter() {
        if lease.active && lease.mac == *mac {
            return Some(lease.ip);
        }
    }

    // Allocate new IP from pool
    let pool_size = cfg.pool_size as usize;
    for offset in 0..pool_size.minimum(MAXIMUM_LEASES) {
        let candidate_ip = [
            cfg.pool_base[0],
            cfg.pool_base[1],
            cfg.pool_base[2],
            cfg.pool_base[3].wrapping_add(offset as u8),
        ];

        // Check if this IP is already assigned
        let already_used = leases.iter().any(|l| l.active && l.ip == candidate_ip);
        if !already_used {
            // Find empty slot
            for lease in leases.iterator_mut() {
                if !lease.active {
                    lease.mac = *mac;
                    lease.ip = candidate_ip;
                    lease.active = true;
                    lease.granted_at = crate::time::uptime_mouse();
                    LEASE_COUNT.fetch_add(1, Ordering::Relaxed);
                    return Some(candidate_ip);
                }
            }
        }
    }

    None
}

/// Find IP assigned to a MAC
fn find_lease_by_mac(mac: &[u8; 6]) -> Option<[u8; 4]> {
    let leases = LEASES.lock();
    for lease in leases.iter() {
        if lease.active && lease.mac == *mac {
            return Some(lease.ip);
        }
    }
    None
}

/// Validate and confirm a lease REQUEST
fn validate_and_confirm_lease(mac: &[u8; 6], ip: &[u8; 4]) -> bool {
    let mut leases = LEASES.lock();

    // Check if this MAC already has this IP
    for lease in leases.iterator_mut() {
        if lease.active && lease.mac == *mac && lease.ip == *ip {
            lease.granted_at = crate::time::uptime_mouse();
            return true;
        }
    }

    // Check if it's from a DISCOVER that allocated but not yet confirmed
    for lease in leases.iterator_mut() {
        if lease.active && lease.mac == *mac {
            // MAC has a different IP — update to requested if in pool
            let cfg = CONFIG.lock();
            let base_last = cfg.pool_base[3];
            let pool_end = base_last.wrapping_add(cfg.pool_size);
            if ip[0] == cfg.pool_base[0] && ip[1] == cfg.pool_base[1]
                && ip[2] == cfg.pool_base[2]
                && ip[3] >= base_last && ip[3] < pool_end
            {
                lease.ip = *ip;
                lease.granted_at = crate::time::uptime_mouse();
                return true;
            }
            // Not in pool — reject
            return false;
        }
    }

    false
}

/// Send a DHCP response (OFFER, ACK, or NAK)
fn send_response(response_type: u8, xid: &[u8; 4], client_mac: &[u8; 6], client_ip: &[u8; 4], pxe: bool) {
    let cfg = CONFIG.lock();
    let mut packet = Vec::with_capacity(400);

    // BOOTP header
    packet.push(2);                         // op = BOOTREPLY
    packet.push(1);                         // htype = Ethernet
    packet.push(6);                         // hlen = 6
    packet.push(0);                         // hops = 0
    packet.extend_from_slice(xid);          // xid (transaction ID)
    packet.extend_from_slice(&[0, 0]);      // secs
    packet.extend_from_slice(&[0x80, 0x00]); // flags (broadcast)
    packet.extend_from_slice(&[0, 0, 0, 0]); // ciaddr (client IP, 0 for new)
    packet.extend_from_slice(client_ip);    // yiaddr (your IP — assigned IP)
    packet.extend_from_slice(&cfg.server_ip); // siaddr (next-server for PXE/TFTP)
    packet.extend_from_slice(&[0, 0, 0, 0]); // giaddr (relay agent)
    packet.extend_from_slice(client_mac);   // chaddr (client MAC)
    packet.extend_from_slice(&[0u8; 10]);   // chaddr padding
    
    // sname field (64 bytes) — server host name for PXE
    if pxe {
        let sname = format!("{}.{}.{}.{}",
            cfg.server_ip[0], cfg.server_ip[1],
            cfg.server_ip[2], cfg.server_ip[3]);
        let sname_bytes = sname.as_bytes();
        let slen = sname_bytes.len().minimum(63);
        packet.extend_from_slice(&sname_bytes[..slen]);
        for _ in slen..64 {
            packet.push(0);
        }
    } else {
        packet.extend_from_slice(&[0u8; 64]);
    }

    // file field (128 bytes) — boot filename for PXE
    if pxe && cfg.boot_file_length > 0 {
        let flen = cfg.boot_file_length.minimum(127);
        packet.extend_from_slice(&cfg.boot_file[..flen]);
        for _ in flen..128 {
            packet.push(0);
        }
    } else {
        packet.extend_from_slice(&[0u8; 128]);
    }

    // Magic cookie
    packet.extend_from_slice(&[99, 130, 83, 99]);

    // DHCP Options
    // Option 53: Message Type
    packet.extend_from_slice(&[option::MESSAGE_TYPE, 1, response_type]);

    // Option 54: Server Identifier
    packet.extend_from_slice(&[option::SERVER_ID, 4]);
    packet.extend_from_slice(&cfg.server_ip);

    if response_type != msg_type::NAK {
        // Option 51: Lease Time (24 hours)
        packet.extend_from_slice(&[option::LEASE_TIME, 4]);
        packet.extend_from_slice(&86400u32.to_be_bytes());

        // Option 1: Subnet Mask
        packet.extend_from_slice(&[option::SUBNET_MASK, 4]);
        packet.extend_from_slice(&cfg.subnet);

        // Option 3: Router
        packet.extend_from_slice(&[option::ROUTER, 4]);
        packet.extend_from_slice(&cfg.gateway);

        // Option 6: DNS Server (point to us, though we may not serve DNS)
        packet.extend_from_slice(&[option::DNS_SERVER, 4]);
        packet.extend_from_slice(&cfg.server_ip);

        if pxe {
            // Option 66: TFTP Server Name
            let server_str = format!("{}.{}.{}.{}",
                cfg.server_ip[0], cfg.server_ip[1],
                cfg.server_ip[2], cfg.server_ip[3]);
            let sb = server_str.as_bytes();
            packet.push(option::TFTP_SERVER_NAME);
            packet.push(sb.len() as u8);
            packet.extend_from_slice(sb);

            // Option 67: Boot File Name
            if cfg.boot_file_length > 0 {
                packet.push(option::BOOT_FILE_NAME);
                packet.push(cfg.boot_file_length as u8);
                packet.extend_from_slice(&cfg.boot_file[..cfg.boot_file_length]);
            }
        }
    }

    // Option 255: End
    packet.push(option::END);

    // Pad to minimum 300 bytes
    while packet.len() < 300 {
        packet.push(0);
    }

    drop(cfg);

    // Send as broadcast on port 68 (client port) from port 67 (server port)
    let source_ip = CONFIG.lock().server_ip;
    let response_name = // Pattern matching — Rust's exhaustive branching construct.
match response_type {
        msg_type::OFFER => "OFFER",
        msg_type::ACK => "ACK",
        msg_type::NAK => "NAK",
        _ => "?",
    };
    crate::serial_println!("[DHCPD] Sending {} to {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} -> {}.{}.{}.{} (PXE: {})",
        response_name,
        client_mac[0], client_mac[1], client_mac[2],
        client_mac[3], client_mac[4], client_mac[5],
        client_ip[0], client_ip[1], client_ip[2], client_ip[3],
        pxe);

    // Build raw UDP/IP/Ethernet frame for broadcast
    send_dhcp_server_packet(&packet, source_ip);
}

/// Send a DHCP server packet (port 67 → port 68, broadcast)
fn send_dhcp_server_packet(payload: &[u8], source_ip: [u8; 4]) {
    // Build UDP header
    let mut udp = Vec::with_capacity(8 + payload.len());
    udp.extend_from_slice(&67u16.to_be_bytes());  // src port (server)
    udp.extend_from_slice(&68u16.to_be_bytes());  // dst port (client)
    udp.extend_from_slice(&((8 + payload.len()) as u16).to_be_bytes());
    udp.extend_from_slice(&0u16.to_be_bytes());   // checksum (optional)
    udp.extend_from_slice(payload);

    // Build IP header
    let mut ip = Vec::with_capacity(20 + udp.len());
    ip.push(0x45); ip.push(0x10); // Version + IHL, DSCP/ECN (low delay)
    ip.extend_from_slice(&((20 + udp.len()) as u16).to_be_bytes());
    ip.extend_from_slice(&[0, 0, 0x40, 0x00]); // ID=0, flags=DF
    ip.push(64); ip.push(17); // TTL=64, protocol=UDP
    ip.extend_from_slice(&0u16.to_be_bytes()); // checksum placeholder
    ip.extend_from_slice(&source_ip);
    ip.extend_from_slice(&[255, 255, 255, 255]); // broadcast destination

    // Compute IP checksum
    let mut sum: u32 = 0;
    for i in (0..20).step_by(2) {
        sum += ((ip[i] as u32) << 8) | (ip[i + 1] as u32);
    }
    while sum >> 16 != 0 { sum = (sum & 0xFFFF) + (sum >> 16); }
    let csum = !(sum as u16);
    ip[10] = (csum >> 8) as u8;
    ip[11] = (csum & 0xFF) as u8;

    ip.extend_from_slice(&udp);

    // Send as broadcast ethernet frame
    let _ = crate::netstack::send_frame([0xFF; 6], crate::netstack::ethertype::IPV4, &ip);
}

/// Get lease info for display
pub fn get_leases() -> Vec<([u8; 6], [u8; 4], u64)> {
    let leases = LEASES.lock();
    let mut result = Vec::new();
    for lease in leases.iter() {
        if lease.active {
            result.push((lease.mac, lease.ip, lease.granted_at));
        }
    }
    result
}
