//! IPv6 Protocol Implementation
//!
//! Implements IPv6 header parsing, link-local addressing (fe80::/10),
//! and ICMPv6/NDP (Neighbor Discovery Protocol) for basic IPv6 support.

use alloc::vec::Vec;
use core::fmt;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

/// IPv6 address (128 bits)
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Ipv6Address(pub [u8; 16]);

impl Ipv6Address {
    pub const UNSPECIFIED: Self = Self([0; 16]);
    pub const LOOPBACK: Self = Self([0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,1]);
    /// All-nodes multicast (ff02::1)
    pub const ALL_NODES: Self = Self([0xff,0x02,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,1]);
    /// All-routers multicast (ff02::2)
    pub const ALL_ROUTERS: Self = Self([0xff,0x02,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,2]);

    /// Create new IPv6 address from 16 bytes
    pub const fn new(bytes: [u8; 16]) -> Self { Self(bytes) }

    /// Generate EUI-64 link-local address from MAC (fe80::XX:XXff:feXX:XXXX)
    pub fn from_mac_link_local(mac: [u8; 6]) -> Self {
        let mut addr = [0u8; 16];
        addr[0] = 0xfe; addr[1] = 0x80;
        // Interface ID from EUI-64
        addr[8] = mac[0] ^ 0x02;  // flip U/L bit
        addr[9] = mac[1];
        addr[10] = mac[2];
        addr[11] = 0xff;
        addr[12] = 0xfe;
        addr[13] = mac[3];
        addr[14] = mac[4];
        addr[15] = mac[5];
        Self(addr)
    }

    /// Check if this is a link-local address (fe80::/10)
    pub fn is_link_local(&self) -> bool {
        self.0[0] == 0xfe && (self.0[1] & 0xc0) == 0x80
    }

    /// Check if this is a multicast address (ff00::/8)
    pub fn is_multicast(&self) -> bool {
        self.0[0] == 0xff
    }

    /// Compute solicited-node multicast address (ff02::1:ffXX:XXXX)
    pub fn solicited_node_multicast(&self) -> Self {
        let mut addr = [0u8; 16];
        addr[0] = 0xff; addr[1] = 0x02;
        addr[11] = 0x01; addr[12] = 0xff;
        addr[13] = self.0[13];
        addr[14] = self.0[14];
        addr[15] = self.0[15];
        Self(addr)
    }

    /// Compute Ethernet multicast MAC for an IPv6 multicast address
    pub fn multicast_mac(&self) -> [u8; 6] {
        [0x33, 0x33, self.0[12], self.0[13], self.0[14], self.0[15]]
    }
}

impl fmt::Display for Ipv6Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Simplified display (no :: compression for simplicity)
        let a = &self.0;
        write!(f, "{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
            a[0],a[1], a[2],a[3], a[4],a[5], a[6],a[7],
            a[8],a[9], a[10],a[11], a[12],a[13], a[14],a[15])
    }
}

impl fmt::Debug for Ipv6Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ipv6({})", self)
    }
}

/// IPv6 Next Header (protocol) values
pub mod next_header {
    pub const HOP_BY_HOP: u8 = 0;
    pub const TCP: u8 = 6;
    pub const UDP: u8 = 17;
    pub const ICMPV6: u8 = 58;
    pub const NO_NEXT: u8 = 59;
}

/// IPv6 header (40 bytes, fixed size)
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Ipv6Header {
    pub version_tc_fl: u32,   // Version(4) + Traffic Class(8) + Flow Label(20)
    pub payload_length: u16,  // Big endian
    pub next_header: u8,
    pub hop_limit: u8,
    pub src: [u8; 16],
    pub dst: [u8; 16],
}

impl Ipv6Header {
    pub const SIZE: usize = 40;

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE { return None; }
        Some(unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Self) })
    }

    pub fn version(&self) -> u8 {
        ((u32::from_be(self.version_tc_fl) >> 28) & 0xF) as u8
    }

    pub fn payload_len(&self) -> u16 {
        u16::from_be(self.payload_length)
    }

    pub fn src_addr(&self) -> Ipv6Address { Ipv6Address(self.src) }
    pub fn dst_addr(&self) -> Ipv6Address { Ipv6Address(self.dst) }
}

// ═══════════════════════════════════════════════════════════════════
// State
// ═══════════════════════════════════════════════════════════════════

static ENABLED: AtomicBool = AtomicBool::new(false);

struct Ipv6State {
    link_local: Ipv6Address,
}

static STATE: Mutex<Ipv6State> = Mutex::new(Ipv6State {
    link_local: Ipv6Address::UNSPECIFIED,
});

// ═══════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════

/// Initialize IPv6 — generate link-local address from MAC
pub fn init() {
    let mac = crate::drivers::net::get_mac()
        .or_else(crate::network::get_mac_address)
        .unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);

    let link_local = Ipv6Address::from_mac_link_local(mac);
    STATE.lock().link_local = link_local;
    ENABLED.store(true, Ordering::SeqCst);

    crate::log!("[IPv6] Link-local: {}", link_local);

    // Send Router Solicitation
    let _ = super::icmpv6::send_router_solicitation(link_local);
}

/// Check if IPv6 is enabled
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Get our link-local address
pub fn link_local_addr() -> Ipv6Address {
    STATE.lock().link_local
}

/// Handle incoming IPv6 packet
pub fn handle_packet(data: &[u8]) {
    if !ENABLED.load(Ordering::Relaxed) { return; }

    let header = match Ipv6Header::parse(data) {
        Some(h) => h,
        None => return,
    };

    if header.version() != 6 { return; }

    let payload_len = header.payload_len() as usize;
    let payload = &data[Ipv6Header::SIZE..];
    if payload.len() < payload_len { return; }
    let payload = &payload[..payload_len];

    let dst = header.dst_addr();
    let our_addr = STATE.lock().link_local;

    // Check if packet is for us (unicast, multicast, or all-nodes)
    let for_us = dst == our_addr
        || dst == Ipv6Address::ALL_NODES
        || dst == our_addr.solicited_node_multicast()
        || dst.is_multicast();

    if !for_us { return; }

    match header.next_header {
        next_header::ICMPV6 => {
            super::icmpv6::handle_packet(header.src_addr(), header.dst_addr(), payload);
        }
        next_header::TCP => {
            crate::serial_println!("[IPv6] TCP packet from {} (not implemented)", header.src_addr());
        }
        next_header::UDP => {
            crate::serial_println!("[IPv6] UDP packet from {} (not implemented)", header.src_addr());
        }
        _ => {}
    }
}

/// Send an IPv6 packet
pub fn send_packet(dst: Ipv6Address, next_header: u8, payload: &[u8]) -> Result<(), &'static str> {
    let src = STATE.lock().link_local;
    if src == Ipv6Address::UNSPECIFIED {
        return Err("IPv6 not initialized");
    }

    send_packet_with_src(src, dst, next_header, 64, payload)
}

/// Send an IPv6 packet with explicit source address
pub fn send_packet_with_src(
    src: Ipv6Address,
    dst: Ipv6Address,
    next_header: u8,
    hop_limit: u8,
    payload: &[u8],
) -> Result<(), &'static str> {
    let mut packet = Vec::with_capacity(Ipv6Header::SIZE + payload.len());

    // Version(6) + Traffic Class(0) + Flow Label(0)
    let version_tc_fl: u32 = 6 << 28;
    packet.extend_from_slice(&version_tc_fl.to_be_bytes());
    packet.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    packet.push(next_header);
    packet.push(hop_limit);
    packet.extend_from_slice(&src.0);
    packet.extend_from_slice(&dst.0);
    packet.extend_from_slice(payload);

    // Determine destination MAC
    let dst_mac = if dst.is_multicast() {
        dst.multicast_mac()
    } else {
        // Try NDP neighbor cache, fallback to all-nodes multicast
        super::icmpv6::lookup_neighbor(dst).unwrap_or(Ipv6Address::ALL_NODES.multicast_mac())
    };

    super::send_frame(dst_mac, super::ethertype::IPV6, &packet)
}

/// Compute ICMPv6 checksum (pseudo-header + payload)
pub fn icmpv6_checksum(src: &Ipv6Address, dst: &Ipv6Address, payload: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    // Pseudo-header
    for chunk in src.0.chunks(2) {
        sum += ((chunk[0] as u32) << 8) | (chunk[1] as u32);
    }
    for chunk in dst.0.chunks(2) {
        sum += ((chunk[0] as u32) << 8) | (chunk[1] as u32);
    }
    // Upper-layer packet length (32-bit)
    sum += payload.len() as u32;
    // Next header = ICMPv6 = 58
    sum += next_header::ICMPV6 as u32;

    // Payload
    let mut i = 0;
    while i + 1 < payload.len() {
        sum += ((payload[i] as u32) << 8) | (payload[i + 1] as u32);
        i += 2;
    }
    if i < payload.len() {
        sum += (payload[i] as u32) << 8;
    }

    // Fold carries
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !(sum as u16)
}
