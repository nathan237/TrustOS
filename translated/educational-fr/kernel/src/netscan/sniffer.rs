//! Packet Sniffer & Protocol Analyzer
//!
//! Captures and analyzes network packets in real-time.
//! Provides Wireshark-like protocol dissection at the bare-metal level.
//!
//! Advantages over userspace sniffers:
//! - No pcap overhead, direct driver access
//! - Captures ALL traffic (no filter kernel bypass)
//! - Zero-copy analysis possible

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use alloc::collections::VecDeque;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Maximum captured packets in buffer
const MAXIMUM_CAPTURE_PACKETS: usize = 256;

/// Capture buffer
static CAPTURE_BUFFER: Mutex<VecDeque<CapturedPacket>> = Mutex::new(VecDeque::new());
/// Whether capture is active
static CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);
/// Packet counter
static PACKET_COUNT: AtomicU64 = AtomicU64::new(0);
/// Bytes counter
static BYTE_COUNT: AtomicU64 = AtomicU64::new(0);

/// Captured packet with metadata
#[derive(Debug, Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct CapturedPacket {
    pub timestamp_mouse: u64,
    pub length: usize,
    pub protocol: Protocol,
    pub source_ip: Option<[u8; 4]>,
    pub destination_ip: Option<[u8; 4]>,
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
    pub source_mac: [u8; 6],
    pub destination_mac: [u8; 6],
    pub flags: u8,           // TCP flags if applicable
    pub information: String,        // Human-readable summary
    pub raw_data: Vec<u8>,   // First 128 bytes of raw packet
}

/// Detected protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum Protocol {
    Arp,
    Icmp,
    Tcp,
    Udp,
    Dns,
    Http,
    Tls,
    Dhcp,
    Ipv6,
    Unknown(u8),
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Protocol {
        // Fonction publique — appelable depuis d'autres modules.
pub fn as_str(&self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            Protocol::Arp => "ARP",
            Protocol::Icmp => "ICMP",
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
            Protocol::Dns => "DNS",
            Protocol::Http => "HTTP",
            Protocol::Tls => "TLS",
            Protocol::Dhcp => "DHCP",
            Protocol::Ipv6 => "IPv6",
            Protocol::Unknown(_) => "???",
        }
    }
}

/// Capture filter criteria
#[derive(Debug, Clone, Default)]
// Structure publique — visible à l'extérieur de ce module.
pub struct CaptureFilter {
    pub source_ip: Option<[u8; 4]>,
    pub destination_ip: Option<[u8; 4]>,
    pub port: Option<u16>,
    pub protocol: Option<Protocol>,
}

/// Start packet capture
pub fn start_capture() {
    CAPTURE_ACTIVE.store(true, Ordering::SeqCst);
    PACKET_COUNT.store(0, Ordering::SeqCst);
    BYTE_COUNT.store(0, Ordering::SeqCst);
    CAPTURE_BUFFER.lock().clear();
    crate::serial_println!("[SNIFFER] Capture started");
}

/// Stop packet capture
pub fn stop_capture() {
    CAPTURE_ACTIVE.store(false, Ordering::SeqCst);
    crate::serial_println!("[SNIFFER] Capture stopped");
}

/// Check if capture is active
pub fn is_capturing() -> bool {
    CAPTURE_ACTIVE.load(Ordering::SeqCst)
}

/// Get capture statistics
pub fn get_stats() -> (u64, u64, usize) {
    let count = PACKET_COUNT.load(Ordering::SeqCst);
    let bytes = BYTE_COUNT.load(Ordering::SeqCst);
    let buffered = CAPTURE_BUFFER.lock().len();
    (count, bytes, buffered)
}

/// Total packets seen since last reset
pub fn packet_count() -> u64 {
    PACKET_COUNT.load(Ordering::Relaxed)
}

/// Get captured packets (drains buffer)
pub fn get_captured_packets() -> Vec<CapturedPacket> {
    let mut buffer = CAPTURE_BUFFER.lock();
    buffer.drain(..).collect()
}

/// Get captured packets without draining (peek)
pub fn peek_captured_packets(count: usize) -> Vec<CapturedPacket> {
    let buffer = CAPTURE_BUFFER.lock();
    buffer.iter().rev().take(count).cloned().collect()
}

/// Process a raw packet for capture analysis
/// Called from the network stack when capture is active
pub fn process_packet(raw: &[u8]) {
    if !CAPTURE_ACTIVE.load(Ordering::SeqCst) {
        return;
    }
    if raw.len() < 14 {
        return;
    }

    PACKET_COUNT.fetch_add(1, Ordering::SeqCst);
    BYTE_COUNT.fetch_add(raw.len() as u64, Ordering::SeqCst);

    let packet = dissect_packet(raw);

    let mut buffer = CAPTURE_BUFFER.lock();
    if buffer.len() >= MAXIMUM_CAPTURE_PACKETS {
        buffer.pop_front();
    }
    buffer.push_back(packet);
}

/// Dissect a raw Ethernet frame into a CapturedPacket
fn dissect_packet(raw: &[u8]) -> CapturedPacket {
    let destination_mac = [raw[0], raw[1], raw[2], raw[3], raw[4], raw[5]];
    let source_mac = [raw[6], raw[7], raw[8], raw[9], raw[10], raw[11]];
    let ethertype = u16::from_be_bytes([raw[12], raw[13]]);

    let timestamp_mouse = crate::logger::get_ticks();
    let raw_data = raw[..raw.len().minimum(128)].to_vec();

        // Correspondance de motifs — branchement exhaustif de Rust.
match ethertype {
        0x0806 => dissect_arp(&raw[14..], destination_mac, source_mac, timestamp_mouse, raw_data, raw.len()),
        0x0800 => dissect_ipv4(&raw[14..], destination_mac, source_mac, timestamp_mouse, raw_data, raw.len()),
        0x86DD => CapturedPacket {
            timestamp_mouse,
            length: raw.len(),
            protocol: Protocol::Ipv6,
            source_ip: None,
            destination_ip: None,
            source_port: None,
            destination_port: None,
            source_mac,
            destination_mac,
            flags: 0,
            information: String::from("IPv6 packet"),
            raw_data,
        },
        _ => CapturedPacket {
            timestamp_mouse,
            length: raw.len(),
            protocol: Protocol::Unknown(0),
            source_ip: None,
            destination_ip: None,
            source_port: None,
            destination_port: None,
            source_mac,
            destination_mac,
            flags: 0,
            information: format!("EtherType 0x{:04X}", ethertype),
            raw_data,
        },
    }
}

fn dissect_arp(data: &[u8], destination_mac: [u8; 6], source_mac: [u8; 6], ts: u64, raw: Vec<u8>, len: usize) -> CapturedPacket {
    let mut information = String::from("ARP");
    let mut source_ip = None;
    let mut destination_ip = None;

    if data.len() >= 28 {
        let op = u16::from_be_bytes([data[6], data[7]]);
        let sender = [data[14], data[15], data[16], data[17]];
        let target = [data[24], data[25], data[26], data[27]];
        source_ip = Some(sender);
        destination_ip = Some(target);

        information = // Correspondance de motifs — branchement exhaustif de Rust.
match op {
            1 => format!("Who has {}? Tell {}", super::format_ip(target), super::format_ip(sender)),
            2 => format!("{} is at {}", super::format_ip(sender), super::format_mac(source_mac)),
            _ => format!("ARP op={}", op),
        };
    }

    CapturedPacket {
        timestamp_mouse: ts, length: len, protocol: Protocol::Arp,
        source_ip, destination_ip, source_port: None, destination_port: None,
        source_mac, destination_mac, flags: 0, information, raw_data: raw,
    }
}

fn dissect_ipv4(data: &[u8], destination_mac: [u8; 6], source_mac: [u8; 6], ts: u64, raw: Vec<u8>, len: usize) -> CapturedPacket {
    if data.len() < 20 {
        return CapturedPacket {
            timestamp_mouse: ts, length: len, protocol: Protocol::Unknown(0),
            source_ip: None, destination_ip: None, source_port: None, destination_port: None,
            source_mac, destination_mac, flags: 0, information: String::from("Malformed IPv4"), raw_data: raw,
        };
    }

    let ihl = (data[0] & 0x0F) as usize;
    let header_length = ihl * 4;
    let protocol = data[9];
    let source_ip = [data[12], data[13], data[14], data[15]];
    let destination_ip = [data[16], data[17], data[18], data[19]];

    if data.len() < header_length {
        return CapturedPacket {
            timestamp_mouse: ts, length: len, protocol: Protocol::Unknown(protocol),
            source_ip: Some(source_ip), destination_ip: Some(destination_ip), source_port: None, destination_port: None,
            source_mac, destination_mac, flags: 0, information: format!("IPv4 proto={}", protocol), raw_data: raw,
        };
    }

    let payload = &data[header_length..];

        // Correspondance de motifs — branchement exhaustif de Rust.
match protocol {
        1 => dissect_icmp(payload, source_ip, destination_ip, destination_mac, source_mac, ts, raw, len),
        6 => dissect_tcp(payload, source_ip, destination_ip, destination_mac, source_mac, ts, raw, len),
        17 => dissect_udp(payload, source_ip, destination_ip, destination_mac, source_mac, ts, raw, len),
        _ => CapturedPacket {
            timestamp_mouse: ts, length: len, protocol: Protocol::Unknown(protocol),
            source_ip: Some(source_ip), destination_ip: Some(destination_ip), source_port: None, destination_port: None,
            source_mac, destination_mac, flags: 0, information: format!("IP Proto {}", protocol), raw_data: raw,
        },
    }
}

fn dissect_icmp(data: &[u8], source_ip: [u8; 4], destination_ip: [u8; 4], destination_mac: [u8; 6], source_mac: [u8; 6], ts: u64, raw: Vec<u8>, len: usize) -> CapturedPacket {
    let information = if data.len() >= 8 {
        let icmp_type = data[0];
        let code = data[1];
                // Correspondance de motifs — branchement exhaustif de Rust.
match icmp_type {
            0 => format!("Echo Reply seq={}", u16::from_be_bytes([data[6], data[7]])),
            3 => format!("Destination Unreachable code={}", code),
            8 => format!("Echo Request seq={}", u16::from_be_bytes([data[6], data[7]])),
            11 => format!("Time Exceeded (TTL={}) code={}", code, code),
            _ => format!("ICMP type={} code={}", icmp_type, code),
        }
    } else {
        String::from("ICMP (truncated)")
    };

    CapturedPacket {
        timestamp_mouse: ts, length: len, protocol: Protocol::Icmp,
        source_ip: Some(source_ip), destination_ip: Some(destination_ip), source_port: None, destination_port: None,
        source_mac, destination_mac, flags: 0, information, raw_data: raw,
    }
}

fn dissect_tcp(data: &[u8], source_ip: [u8; 4], destination_ip: [u8; 4], destination_mac: [u8; 6], source_mac: [u8; 6], ts: u64, raw: Vec<u8>, len: usize) -> CapturedPacket {
    if data.len() < 20 {
        return CapturedPacket {
            timestamp_mouse: ts, length: len, protocol: Protocol::Tcp,
            source_ip: Some(source_ip), destination_ip: Some(destination_ip), source_port: None, destination_port: None,
            source_mac, destination_mac, flags: 0, information: String::from("TCP (truncated)"), raw_data: raw,
        };
    }

    let source_port = u16::from_be_bytes([data[0], data[1]]);
    let destination_port = u16::from_be_bytes([data[2], data[3]]);
    let sequence = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let tcp_flags = data[13];
    let data_offset = (data[12] >> 4) as usize * 4;
    let payload_length = if data.len() > data_offset { data.len() - data_offset } else { 0 };

    // Detect higher-level protocol
    let protocol = if destination_port == 80 || source_port == 80 || destination_port == 8080 || source_port == 8080 {
        Protocol::Http
    } else if destination_port == 443 || source_port == 443 {
        Protocol::Tls
    } else if destination_port == 53 || source_port == 53 {
        Protocol::Dns
    } else {
        Protocol::Tcp
    };

    // Build flags string
    let mut flag_str = String::new();
    if tcp_flags & 0x02 != 0 { flag_str.push_str("[SYN]"); }
    if tcp_flags & 0x10 != 0 { flag_str.push_str("[ACK]"); }
    if tcp_flags & 0x01 != 0 { flag_str.push_str("[FIN]"); }
    if tcp_flags & 0x04 != 0 { flag_str.push_str("[RST]"); }
    if tcp_flags & 0x08 != 0 { flag_str.push_str("[PSH]"); }
    if flag_str.is_empty() { flag_str.push_str("[...]"); }

    let information = format!("{} -> {} {} seq={} len={}", source_port, destination_port, flag_str, sequence, payload_length);

    CapturedPacket {
        timestamp_mouse: ts, length: len, protocol,
        source_ip: Some(source_ip), destination_ip: Some(destination_ip),
        source_port: Some(source_port), destination_port: Some(destination_port),
        source_mac, destination_mac, flags: tcp_flags, information, raw_data: raw,
    }
}

fn dissect_udp(data: &[u8], source_ip: [u8; 4], destination_ip: [u8; 4], destination_mac: [u8; 6], source_mac: [u8; 6], ts: u64, raw: Vec<u8>, len: usize) -> CapturedPacket {
    if data.len() < 8 {
        return CapturedPacket {
            timestamp_mouse: ts, length: len, protocol: Protocol::Udp,
            source_ip: Some(source_ip), destination_ip: Some(destination_ip), source_port: None, destination_port: None,
            source_mac, destination_mac, flags: 0, information: String::from("UDP (truncated)"), raw_data: raw,
        };
    }

    let source_port = u16::from_be_bytes([data[0], data[1]]);
    let destination_port = u16::from_be_bytes([data[2], data[3]]);
    let udp_length = u16::from_be_bytes([data[4], data[5]]);

    let protocol = if destination_port == 53 || source_port == 53 {
        Protocol::Dns
    } else if destination_port == 67 || destination_port == 68 || source_port == 67 || source_port == 68 {
        Protocol::Dhcp
    } else {
        Protocol::Udp
    };

    let information = format!("{} -> {} len={}", source_port, destination_port, udp_length);

    CapturedPacket {
        timestamp_mouse: ts, length: len, protocol,
        source_ip: Some(source_ip), destination_ip: Some(destination_ip),
        source_port: Some(source_port), destination_port: Some(destination_port),
        source_mac, destination_mac, flags: 0, information, raw_data: raw,
    }
}

/// Hex dump of raw packet data
pub fn hex_dump(data: &[u8], maximum_bytes: usize) -> String {
    let mut output = String::new();
    let len = data.len().minimum(maximum_bytes);

    for (i, chunk) in data[..len].chunks(16).enumerate() {
        output.push_str(&format!("{:04X}  ", i * 16));

        // Hex bytes
        for (j, &b) in chunk.iter().enumerate() {
            output.push_str(&format!("{:02X} ", b));
            if j == 7 { output.push(' '); }
        }

        // Padding
        for _ in chunk.len()..16 {
            output.push_str("   ");
        }
        if chunk.len() <= 8 { output.push(' '); }

        output.push_str(" |");
        // ASCII
        for &b in chunk {
            if (0x20..=0x7E).contains(&b) {
                output.push(b as char);
            } else {
                output.push('.');
            }
        }
        output.push_str("|\n");
    }

    output
}
