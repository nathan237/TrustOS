//! Packet Replay & Injection — CyberLab Security Testing
//!
//! Allows replaying captured packets and crafting custom packets for security
//! analysis. Operates at raw Ethernet / IP level using direct driver access.
//!
//! Features:
//! - Replay captured packets from sniffer buffer
//! - Craft custom TCP/UDP/ICMP packets
//! - SYN flood simulation (for testing firewall rules)
//! - ARP spoof detection probe
//! - Raw Ethernet frame injection

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

/// Total packets injected (statistics)
static INJECTED_COUNT: AtomicU64 = AtomicU64::new(0);

/// Replay result
#[derive(Debug)]
pub struct ReplayResult {
    pub packets_sent: u64,
    pub packets_failed: u64,
    pub info: String,
}

// ── Raw injection ──────────────────────────────────────────────────────

/// Inject a raw Ethernet frame onto the wire.
pub fn inject_raw(frame: &[u8]) -> Result<(), &'static str> {
    if frame.len() < 14 {
        return Err("frame too short (min 14 bytes for Ethernet header)");
    }
    crate::network::send_packet(frame)?;
    INJECTED_COUNT.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

/// Inject a raw IP packet (builds Ethernet header automatically).
pub fn inject_ip(dest_ip: [u8; 4], protocol: u8, payload: &[u8]) -> Result<(), &'static str> {
    crate::netstack::ip::send_packet(dest_ip, protocol, payload)?;
    INJECTED_COUNT.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

// ── Packet crafting helpers ────────────────────────────────────────────

/// Build a minimal TCP SYN packet (for port knocking / stealth probes).
pub fn craft_syn(src_ip: [u8; 4], dst_ip: [u8; 4], src_port: u16, dst_port: u16, seq: u32) -> Vec<u8> {
    let mut seg = Vec::with_capacity(20);
    seg.extend_from_slice(&src_port.to_be_bytes());
    seg.extend_from_slice(&dst_port.to_be_bytes());
    seg.extend_from_slice(&seq.to_be_bytes());
    seg.extend_from_slice(&0u32.to_be_bytes()); // ack
    seg.push(0x50); // data offset = 5 words
    seg.push(0x02); // SYN flag
    seg.extend_from_slice(&65535u16.to_be_bytes()); // window
    seg.extend_from_slice(&0u16.to_be_bytes()); // checksum placeholder
    seg.extend_from_slice(&0u16.to_be_bytes()); // urgent ptr
    // Fill in checksum
    let csum = crate::netstack::tcp::tcp_checksum_external(src_ip, dst_ip, &seg);
    seg[16] = (csum >> 8) as u8;
    seg[17] = (csum & 0xFF) as u8;
    seg
}

/// Build an ICMP echo request (ping).
pub fn craft_icmp_echo(id: u16, seq_num: u16, payload: &[u8]) -> Vec<u8> {
    let mut pkt = Vec::with_capacity(8 + payload.len());
    pkt.push(8); // type = echo request
    pkt.push(0); // code
    pkt.extend_from_slice(&0u16.to_be_bytes()); // checksum placeholder
    pkt.extend_from_slice(&id.to_be_bytes());
    pkt.extend_from_slice(&seq_num.to_be_bytes());
    pkt.extend_from_slice(payload);
    // ICMP checksum
    let csum = icmp_checksum(&pkt);
    pkt[2] = (csum >> 8) as u8;
    pkt[3] = (csum & 0xFF) as u8;
    pkt
}

/// Build a UDP datagram.
pub fn craft_udp(src_port: u16, dst_port: u16, payload: &[u8]) -> Vec<u8> {
    let len = 8 + payload.len() as u16;
    let mut pkt = Vec::with_capacity(len as usize);
    pkt.extend_from_slice(&src_port.to_be_bytes());
    pkt.extend_from_slice(&dst_port.to_be_bytes());
    pkt.extend_from_slice(&len.to_be_bytes());
    pkt.extend_from_slice(&0u16.to_be_bytes()); // checksum (optional for UDP/IPv4)
    pkt.extend_from_slice(payload);
    pkt
}

// ── Replay from sniffer capture ────────────────────────────────────────

/// Replay all packets currently in the sniffer capture buffer.
pub fn replay_capture() -> ReplayResult {
    let captured = super::sniffer::get_captured_packets();
    let mut sent = 0u64;
    let mut fail = 0u64;
    for pkt in &captured {
        if pkt.raw_data.len() >= 14 {
            match inject_raw(&pkt.raw_data) {
                Ok(()) => sent += 1,
                Err(_) => fail += 1,
            }
        } else {
            fail += 1;
        }
    }
    ReplayResult {
        packets_sent: sent,
        packets_failed: fail,
        info: format!("Replayed {}/{} packets", sent, sent + fail),
    }
}

/// Replay a single captured packet by index.
pub fn replay_packet(index: usize) -> Result<(), &'static str> {
    let captured = super::sniffer::get_captured_packets();
    let pkt = captured.get(index).ok_or("packet index out of range")?;
    if pkt.raw_data.len() < 14 {
        return Err("captured packet too short");
    }
    inject_raw(&pkt.raw_data)
}

// ── Security test probes ───────────────────────────────────────────────

/// Send N SYN packets to a target (for firewall / IDS stress testing).
/// Returns number of packets successfully sent.
pub fn syn_flood_test(dst_ip: [u8; 4], dst_port: u16, count: u32) -> u32 {
    let src_ip = crate::network::get_ipv4_config()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([10, 0, 2, 15]);
    let mut ok = 0u32;
    for i in 0..count {
        let src_port = 1024 + (i as u16 % 64000);
        let seq = crate::rng::secure_random_u32();
        let syn = craft_syn(src_ip, dst_ip, src_port, dst_port, seq);
        // Send as raw IP (protocol 6 = TCP)
        if inject_ip(dst_ip, 6, &syn).is_ok() {
            ok += 1;
        }
    }
    ok
}

/// Send an ARP request (for ARP spoof detection / host probing).
pub fn arp_probe(target_ip: [u8; 4]) -> Result<(), &'static str> {
    crate::netstack::arp::send_request(target_ip)?;
    INJECTED_COUNT.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

// ── Statistics ─────────────────────────────────────────────────────────

/// Total packets injected since boot
pub fn total_injected() -> u64 {
    INJECTED_COUNT.load(Ordering::Relaxed)
}

/// Reset injection counter
pub fn reset_stats() {
    INJECTED_COUNT.store(0, Ordering::Relaxed);
}

// ── Internal helpers ───────────────────────────────────────────────────

fn icmp_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += ((data[i] as u32) << 8) | (data[i + 1] as u32);
        i += 2;
    }
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}
