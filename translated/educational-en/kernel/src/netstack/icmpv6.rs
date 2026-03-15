//! ICMPv6 + Neighbor Discovery Protocol (NDP)
//!
//! Implements:
//! - ICMPv6 Echo Request/Reply (ping6)
//! - NDP Neighbor Solicitation / Advertisement (address resolution)
//! - NDP Router Solicitation (auto-configuration)
//! - Neighbor cache for MAC resolution

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

use super::ipv6::{self, Ipv6Address, next_header};

/// ICMPv6 message types
pub mod icmpv6_type {
    // Error messages
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DEST_UNREACHABLE: u8 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PACKET_TOO_BIG: u8 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const TIME_EXCEEDED: u8 = 3;

    // Informational
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ECHO_REQUEST: u8 = 128;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ECHO_REPLY: u8 = 129;

    // NDP
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ROUTER_SOLICITATION: u8 = 133;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ROUTER_ADVERTISEMENT: u8 = 134;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const NEIGHBOR_SOLICITATION: u8 = 135;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const NEIGHBOR_ADVERTISEMENT: u8 = 136;
}

/// NDP option types
pub mod ndp_option {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SOURCE_LINK_ADDRESS: u8 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const TARGET_LINK_ADDRESS: u8 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PREFIX_INFORMATION: u8 = 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const MTU: u8 = 5;
}

/// Neighbor cache entry
#[derive(Clone)]
struct NeighborEntry {
    mac: [u8; 6],
    #[allow(dead_code)]
    timestamp: u64,
}

/// Neighbor cache (IPv6 → MAC mapping, like ARP for IPv6)
static NEIGHBOR_CACHE: Mutex<BTreeMap<[u8; 16], NeighborEntry>> = Mutex::new(BTreeMap::new());

/// Look up a neighbor's MAC address
pub fn lookup_neighbor(address: Ipv6Address) -> Option<[u8; 6]> {
    NEIGHBOR_CACHE.lock().get(&address.0).map(|e| e.mac)
}

/// Insert into neighbor cache
fn cache_neighbor(address: Ipv6Address, mac: [u8; 6]) {
    let entry = NeighborEntry {
        mac,
        timestamp: crate::logger::get_ticks(),
    };
    NEIGHBOR_CACHE.lock().insert(address.0, entry);
}

// ═══════════════════════════════════════════════════════════════════
// Incoming packet handling
// ═══════════════════════════════════════════════════════════════════

/// Handle incoming ICMPv6 packet
pub fn handle_packet(source: Ipv6Address, destination: Ipv6Address, data: &[u8]) {
    if data.len() < 4 { return; } // ICMPv6 header is at least 4 bytes

    let msg_type = data[0];
    let _code = data[1];
    // Checksum at data[2..4] — we trust the NIC for now

    match msg_type {
        icmpv6_type::ECHO_REQUEST => {
            handle_echo_request(source, destination, data);
        }
        icmpv6_type::ECHO_REPLY => {
            handle_echo_reply(source, data);
        }
        icmpv6_type::NEIGHBOR_SOLICITATION => {
            handle_neighbor_solicitation(source, data);
        }
        icmpv6_type::NEIGHBOR_ADVERTISEMENT => {
            handle_neighbor_advertisement(source, data);
        }
        icmpv6_type::ROUTER_ADVERTISEMENT => {
            handle_router_advertisement(source, data);
        }
        _ => {
            crate::serial_println!("[ICMPv6] Unknown type {} from {}", msg_type, source);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Echo (ping6)
// ═══════════════════════════════════════════════════════════════════

fn handle_echo_request(source: Ipv6Address, _destination: Ipv6Address, data: &[u8]) {
    if data.len() < 8 { return; }

    crate::serial_println!("[ICMPv6] Echo Request from {}", source);

    let our_address = ipv6::link_local_address();

    // Build Echo Reply (type=129, code=0, same identifier+sequence+data)
    let mut reply = Vec::with_capacity(data.len());
    reply.push(icmpv6_type::ECHO_REPLY);
    reply.push(0); // code
    reply.push(0); reply.push(0); // checksum placeholder
    reply.extend_from_slice(&data[4..]); // copy identifier + sequence + payload

    // Compute checksum
    let csum = ipv6::icmpv6_checksum(&our_address, &source, &reply);
    reply[2] = (csum >> 8) as u8;
    reply[3] = (csum & 0xFF) as u8;

    let _ = ipv6::send_packet(source, next_header::ICMPV6, &reply);
}

fn handle_echo_reply(source: Ipv6Address, data: &[u8]) {
    if data.len() < 8 { return; }
    let id = u16::from_be_bytes([data[4], data[5]]);
    let sequence = u16::from_be_bytes([data[6], data[7]]);
    crate::serial_println!("[ICMPv6] Echo Reply from {}: id={} seq={}", source, id, sequence);
}

// ═══════════════════════════════════════════════════════════════════
// NDP — Neighbor Solicitation / Advertisement
// ═══════════════════════════════════════════════════════════════════

fn handle_neighbor_solicitation(source: Ipv6Address, data: &[u8]) {
    // NS format: type(1) + code(1) + csum(2) + reserved(4) + target(16) + options...
    if data.len() < 24 { return; }

    let target = Ipv6Address::new([
        data[8], data[9], data[10], data[11],
        data[12], data[13], data[14], data[15],
        data[16], data[17], data[18], data[19],
        data[20], data[21], data[22], data[23],
    ]);

    let our_address = ipv6::link_local_address();

    // Extract source link-layer address option if present
    parse_ndp_options(&data[24..], |opt_type, opt_data| {
        if opt_type == ndp_option::SOURCE_LINK_ADDRESS && opt_data.len() >= 6 {
            let mac = [opt_data[0], opt_data[1], opt_data[2], opt_data[3], opt_data[4], opt_data[5]];
            cache_neighbor(source, mac);
        }
    });

    // Only respond if they're asking about our address
    if target != our_address { return; }

    crate::serial_println!("[NDP] Neighbor Solicitation for {} from {}", target, source);

    // Send Neighbor Advertisement
    let _ = send_neighbor_advertisement(source, our_address);
}

fn handle_neighbor_advertisement(source: Ipv6Address, data: &[u8]) {
    if data.len() < 24 { return; }

    let target = Ipv6Address::new([
        data[8], data[9], data[10], data[11],
        data[12], data[13], data[14], data[15],
        data[16], data[17], data[18], data[19],
        data[20], data[21], data[22], data[23],
    ]);

    crate::serial_println!("[NDP] Neighbor Advertisement: {} is at {}", target, source);

    // Extract target link-layer address option
    parse_ndp_options(&data[24..], |opt_type, opt_data| {
        if opt_type == ndp_option::TARGET_LINK_ADDRESS && opt_data.len() >= 6 {
            let mac = [opt_data[0], opt_data[1], opt_data[2], opt_data[3], opt_data[4], opt_data[5]];
            cache_neighbor(target, mac);
        }
    });
}

// ═══════════════════════════════════════════════════════════════════
// NDP — Router Solicitation / Advertisement
// ═══════════════════════════════════════════════════════════════════

fn handle_router_advertisement(source: Ipv6Address, data: &[u8]) {
    // RA format: type(1) + code(1) + csum(2) + hop_limit(1) + flags(1) + lifetime(2)
    //            + reachable_time(4) + retrans_timer(4) + options...
    if data.len() < 16 { return; }

    let hop_limit = data[4];
    let router_lifetime = u16::from_be_bytes([data[6], data[7]]);

    crate::serial_println!("[NDP] Router Advertisement from {}: hop_limit={} lifetime={}s", 
        source, hop_limit, router_lifetime);

    // Parse options for prefix info
    parse_ndp_options(&data[16..], |opt_type, opt_data| {
                // Pattern matching — Rust's exhaustive branching construct.
match opt_type {
            ndp_option::PREFIX_INFORMATION if opt_data.len() >= 30 => {
                let prefix_length = opt_data[0];
                let flags = opt_data[1];
                let prefix = &opt_data[14..30];
                crate::serial_println!("[NDP]   Prefix: {:02x}{:02x}:{:02x}{:02x}::/{} flags={:#x}",
                    prefix[0], prefix[1], prefix[2], prefix[3], prefix_length, flags);
            }
            ndp_option::SOURCE_LINK_ADDRESS if opt_data.len() >= 6 => {
                let mac = [opt_data[0], opt_data[1], opt_data[2], opt_data[3], opt_data[4], opt_data[5]];
                cache_neighbor(source, mac);
                crate::serial_println!("[NDP]   Router MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
            }
            ndp_option::MTU if opt_data.len() >= 6 => {
                let mtu = u32::from_be_bytes([opt_data[2], opt_data[3], opt_data[4], opt_data[5]]);
                crate::serial_println!("[NDP]   MTU: {}", mtu);
            }
            _ => {}
        }
    });
}

// ═══════════════════════════════════════════════════════════════════
// Outgoing messages
// ═══════════════════════════════════════════════════════════════════

/// Send Router Solicitation (ff02::2)
pub fn send_router_solicitation(our_address: Ipv6Address) -> Result<(), &'static str> {
    let mac = crate::drivers::net::get_mac()
        .or_else(crate::network::get_mac_address)
        .unwrap_or([0; 6]);

    let destination = Ipv6Address::ALL_ROUTERS;

    // RS format: type(133) + code(0) + csum(2) + reserved(4) + source_link_addr option
    let mut message = Vec::with_capacity(16);
    message.push(icmpv6_type::ROUTER_SOLICITATION);
    message.push(0); // code
    message.push(0); message.push(0); // checksum placeholder
    message.extend_from_slice(&[0u8; 4]); // reserved

    // Source Link-Layer Address option (type=1, len=1 (in units of 8 bytes))
    message.push(ndp_option::SOURCE_LINK_ADDRESS);
    message.push(1); // length in 8-byte units
    message.extend_from_slice(&mac);

    // Compute checksum
    let csum = ipv6::icmpv6_checksum(&our_address, &destination, &message);
    message[2] = (csum >> 8) as u8;
    message[3] = (csum & 0xFF) as u8;

    crate::serial_println!("[NDP] Sending Router Solicitation");
    ipv6::send_packet_with_source(our_address, destination, next_header::ICMPV6, 255, &message)
}

/// Send Neighbor Advertisement in response to NS
fn send_neighbor_advertisement(destination: Ipv6Address, our_address: Ipv6Address) -> Result<(), &'static str> {
    let mac = crate::drivers::net::get_mac()
        .or_else(crate::network::get_mac_address)
        .unwrap_or([0; 6]);

    // NA format: type(136) + code(0) + csum(2) + flags(4) + target(16) + target_link_addr option
    let mut message = Vec::with_capacity(32);
    message.push(icmpv6_type::NEIGHBOR_ADVERTISEMENT);
    message.push(0); // code
    message.push(0); message.push(0); // checksum placeholder

    // Flags: Solicited(1<<30) | Override(1<<29) — big endian
    let flags: u32 = (1 << 30) | (1 << 29);
    message.extend_from_slice(&flags.to_be_bytes());

    // Target address (the address being advertised)
    message.extend_from_slice(&our_address.0);

    // Target Link-Layer Address option
    message.push(ndp_option::TARGET_LINK_ADDRESS);
    message.push(1); // length in 8-byte units
    message.extend_from_slice(&mac);

    // Compute checksum
    let csum = ipv6::icmpv6_checksum(&our_address, &destination, &message);
    message[2] = (csum >> 8) as u8;
    message[3] = (csum & 0xFF) as u8;

    crate::serial_println!("[NDP] Sending Neighbor Advertisement to {}", destination);
    ipv6::send_packet_with_source(our_address, destination, next_header::ICMPV6, 255, &message)
}

/// Send Neighbor Solicitation to resolve an IPv6 address
pub fn send_neighbor_solicitation(target: Ipv6Address) -> Result<(), &'static str> {
    let our_address = ipv6::link_local_address();
    let mac = crate::drivers::net::get_mac()
        .or_else(crate::network::get_mac_address)
        .unwrap_or([0; 6]);

    let destination = target.solicited_node_multicast();

    // NS format: type(135) + code(0) + csum(2) + reserved(4) + target(16) + source_link_addr option
    let mut message = Vec::with_capacity(32);
    message.push(icmpv6_type::NEIGHBOR_SOLICITATION);
    message.push(0); // code
    message.push(0); message.push(0); // checksum placeholder
    message.extend_from_slice(&[0u8; 4]); // reserved
    message.extend_from_slice(&target.0);

    // Source Link-Layer Address option
    message.push(ndp_option::SOURCE_LINK_ADDRESS);
    message.push(1);
    message.extend_from_slice(&mac);

    // Compute checksum
    let csum = ipv6::icmpv6_checksum(&our_address, &destination, &message);
    message[2] = (csum >> 8) as u8;
    message[3] = (csum & 0xFF) as u8;

    crate::serial_println!("[NDP] Sending Neighbor Solicitation for {}", target);
    ipv6::send_packet_with_source(our_address, destination, next_header::ICMPV6, 255, &message)
}

// ═══════════════════════════════════════════════════════════════════
// Helper: parse NDP options TLV
// ═══════════════════════════════════════════════════════════════════

fn parse_ndp_options<F: FnMut(u8, &[u8])>(data: &[u8], mut handler: F) {
    let mut i = 0;
    while i + 2 <= data.len() {
        let opt_type = data[i];
        let opt_length = data[i + 1] as usize * 8; // length in 8-byte units
        if opt_length == 0 || i + opt_length > data.len() { break; }
        // Option value starts at offset 2
        handler(opt_type, &data[i + 2..i + opt_length]);
        i += opt_length;
    }
}
