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
const MAX_CAPTURE_PACKETS: usize = 256;

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
pub struct CapturedPacket {
    pub timestamp_ms: u64,
    pub length: usize,
    pub protocol: Protocol,
    pub src_ip: Option<[u8; 4]>,
    pub dst_ip: Option<[u8; 4]>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub src_mac: [u8; 6],
    pub dst_mac: [u8; 6],
    pub flags: u8,           // TCP flags if applicable
    pub info: String,        // Human-readable summary
    pub raw_data: Vec<u8>,   // First 128 bytes of raw packet
}

/// Detected protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Protocol {
    pub fn as_str(&self) -> &'static str {
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
pub struct CaptureFilter {
    pub src_ip: Option<[u8; 4]>,
    pub dst_ip: Option<[u8; 4]>,
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

/// Get captured packets (drains buffer)
pub fn get_captured_packets() -> Vec<CapturedPacket> {
    let mut buf = CAPTURE_BUFFER.lock();
    buf.drain(..).collect()
}

/// Get captured packets without draining (peek)
pub fn peek_captured_packets(count: usize) -> Vec<CapturedPacket> {
    let buf = CAPTURE_BUFFER.lock();
    buf.iter().rev().take(count).cloned().collect()
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

    let mut buf = CAPTURE_BUFFER.lock();
    if buf.len() >= MAX_CAPTURE_PACKETS {
        buf.pop_front();
    }
    buf.push_back(packet);
}

/// Dissect a raw Ethernet frame into a CapturedPacket
fn dissect_packet(raw: &[u8]) -> CapturedPacket {
    let dst_mac = [raw[0], raw[1], raw[2], raw[3], raw[4], raw[5]];
    let src_mac = [raw[6], raw[7], raw[8], raw[9], raw[10], raw[11]];
    let ethertype = u16::from_be_bytes([raw[12], raw[13]]);

    let timestamp_ms = crate::logger::get_ticks();
    let raw_data = raw[..raw.len().min(128)].to_vec();

    match ethertype {
        0x0806 => dissect_arp(&raw[14..], dst_mac, src_mac, timestamp_ms, raw_data, raw.len()),
        0x0800 => dissect_ipv4(&raw[14..], dst_mac, src_mac, timestamp_ms, raw_data, raw.len()),
        0x86DD => CapturedPacket {
            timestamp_ms,
            length: raw.len(),
            protocol: Protocol::Ipv6,
            src_ip: None,
            dst_ip: None,
            src_port: None,
            dst_port: None,
            src_mac,
            dst_mac,
            flags: 0,
            info: String::from("IPv6 packet"),
            raw_data,
        },
        _ => CapturedPacket {
            timestamp_ms,
            length: raw.len(),
            protocol: Protocol::Unknown(0),
            src_ip: None,
            dst_ip: None,
            src_port: None,
            dst_port: None,
            src_mac,
            dst_mac,
            flags: 0,
            info: format!("EtherType 0x{:04X}", ethertype),
            raw_data,
        },
    }
}

fn dissect_arp(data: &[u8], dst_mac: [u8; 6], src_mac: [u8; 6], ts: u64, raw: Vec<u8>, len: usize) -> CapturedPacket {
    let mut info = String::from("ARP");
    let mut src_ip = None;
    let mut dst_ip = None;

    if data.len() >= 28 {
        let op = u16::from_be_bytes([data[6], data[7]]);
        let sender = [data[14], data[15], data[16], data[17]];
        let target = [data[24], data[25], data[26], data[27]];
        src_ip = Some(sender);
        dst_ip = Some(target);

        info = match op {
            1 => format!("Who has {}? Tell {}", super::format_ip(target), super::format_ip(sender)),
            2 => format!("{} is at {}", super::format_ip(sender), super::format_mac(src_mac)),
            _ => format!("ARP op={}", op),
        };
    }

    CapturedPacket {
        timestamp_ms: ts, length: len, protocol: Protocol::Arp,
        src_ip, dst_ip, src_port: None, dst_port: None,
        src_mac, dst_mac, flags: 0, info, raw_data: raw,
    }
}

fn dissect_ipv4(data: &[u8], dst_mac: [u8; 6], src_mac: [u8; 6], ts: u64, raw: Vec<u8>, len: usize) -> CapturedPacket {
    if data.len() < 20 {
        return CapturedPacket {
            timestamp_ms: ts, length: len, protocol: Protocol::Unknown(0),
            src_ip: None, dst_ip: None, src_port: None, dst_port: None,
            src_mac, dst_mac, flags: 0, info: String::from("Malformed IPv4"), raw_data: raw,
        };
    }

    let ihl = (data[0] & 0x0F) as usize;
    let header_len = ihl * 4;
    let protocol = data[9];
    let src_ip = [data[12], data[13], data[14], data[15]];
    let dst_ip = [data[16], data[17], data[18], data[19]];

    if data.len() < header_len {
        return CapturedPacket {
            timestamp_ms: ts, length: len, protocol: Protocol::Unknown(protocol),
            src_ip: Some(src_ip), dst_ip: Some(dst_ip), src_port: None, dst_port: None,
            src_mac, dst_mac, flags: 0, info: format!("IPv4 proto={}", protocol), raw_data: raw,
        };
    }

    let payload = &data[header_len..];

    match protocol {
        1 => dissect_icmp(payload, src_ip, dst_ip, dst_mac, src_mac, ts, raw, len),
        6 => dissect_tcp(payload, src_ip, dst_ip, dst_mac, src_mac, ts, raw, len),
        17 => dissect_udp(payload, src_ip, dst_ip, dst_mac, src_mac, ts, raw, len),
        _ => CapturedPacket {
            timestamp_ms: ts, length: len, protocol: Protocol::Unknown(protocol),
            src_ip: Some(src_ip), dst_ip: Some(dst_ip), src_port: None, dst_port: None,
            src_mac, dst_mac, flags: 0, info: format!("IP Proto {}", protocol), raw_data: raw,
        },
    }
}

fn dissect_icmp(data: &[u8], src_ip: [u8; 4], dst_ip: [u8; 4], dst_mac: [u8; 6], src_mac: [u8; 6], ts: u64, raw: Vec<u8>, len: usize) -> CapturedPacket {
    let info = if data.len() >= 8 {
        let icmp_type = data[0];
        let code = data[1];
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
        timestamp_ms: ts, length: len, protocol: Protocol::Icmp,
        src_ip: Some(src_ip), dst_ip: Some(dst_ip), src_port: None, dst_port: None,
        src_mac, dst_mac, flags: 0, info, raw_data: raw,
    }
}

fn dissect_tcp(data: &[u8], src_ip: [u8; 4], dst_ip: [u8; 4], dst_mac: [u8; 6], src_mac: [u8; 6], ts: u64, raw: Vec<u8>, len: usize) -> CapturedPacket {
    if data.len() < 20 {
        return CapturedPacket {
            timestamp_ms: ts, length: len, protocol: Protocol::Tcp,
            src_ip: Some(src_ip), dst_ip: Some(dst_ip), src_port: None, dst_port: None,
            src_mac, dst_mac, flags: 0, info: String::from("TCP (truncated)"), raw_data: raw,
        };
    }

    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let seq = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let tcp_flags = data[13];
    let data_offset = (data[12] >> 4) as usize * 4;
    let payload_len = if data.len() > data_offset { data.len() - data_offset } else { 0 };

    // Detect higher-level protocol
    let protocol = if dst_port == 80 || src_port == 80 || dst_port == 8080 || src_port == 8080 {
        Protocol::Http
    } else if dst_port == 443 || src_port == 443 {
        Protocol::Tls
    } else if dst_port == 53 || src_port == 53 {
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

    let info = format!("{} -> {} {} seq={} len={}", src_port, dst_port, flag_str, seq, payload_len);

    CapturedPacket {
        timestamp_ms: ts, length: len, protocol,
        src_ip: Some(src_ip), dst_ip: Some(dst_ip),
        src_port: Some(src_port), dst_port: Some(dst_port),
        src_mac, dst_mac, flags: tcp_flags, info, raw_data: raw,
    }
}

fn dissect_udp(data: &[u8], src_ip: [u8; 4], dst_ip: [u8; 4], dst_mac: [u8; 6], src_mac: [u8; 6], ts: u64, raw: Vec<u8>, len: usize) -> CapturedPacket {
    if data.len() < 8 {
        return CapturedPacket {
            timestamp_ms: ts, length: len, protocol: Protocol::Udp,
            src_ip: Some(src_ip), dst_ip: Some(dst_ip), src_port: None, dst_port: None,
            src_mac, dst_mac, flags: 0, info: String::from("UDP (truncated)"), raw_data: raw,
        };
    }

    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let udp_len = u16::from_be_bytes([data[4], data[5]]);

    let protocol = if dst_port == 53 || src_port == 53 {
        Protocol::Dns
    } else if dst_port == 67 || dst_port == 68 || src_port == 67 || src_port == 68 {
        Protocol::Dhcp
    } else {
        Protocol::Udp
    };

    let info = format!("{} -> {} len={}", src_port, dst_port, udp_len);

    CapturedPacket {
        timestamp_ms: ts, length: len, protocol,
        src_ip: Some(src_ip), dst_ip: Some(dst_ip),
        src_port: Some(src_port), dst_port: Some(dst_port),
        src_mac, dst_mac, flags: 0, info, raw_data: raw,
    }
}

/// Hex dump of raw packet data
pub fn hex_dump(data: &[u8], max_bytes: usize) -> String {
    let mut output = String::new();
    let len = data.len().min(max_bytes);

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
