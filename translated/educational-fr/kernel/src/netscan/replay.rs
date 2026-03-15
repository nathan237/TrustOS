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
// Structure publique — visible à l'extérieur de ce module.
pub struct ReplayResult {
    pub packets_sent: u64,
    pub packets_failed: u64,
    pub information: String,
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
pub fn craft_syn(source_ip: [u8; 4], destination_ip: [u8; 4], source_port: u16, destination_port: u16, sequence: u32) -> Vec<u8> {
    let mut seg = Vec::with_capacity(20);
    seg.extend_from_slice(&source_port.to_be_bytes());
    seg.extend_from_slice(&destination_port.to_be_bytes());
    seg.extend_from_slice(&sequence.to_be_bytes());
    seg.extend_from_slice(&0u32.to_be_bytes()); // ack
    seg.push(0x50); // data offset = 5 words
    seg.push(0x02); // SYN flag
    seg.extend_from_slice(&65535u16.to_be_bytes()); // window
    seg.extend_from_slice(&0u16.to_be_bytes()); // checksum placeholder
    seg.extend_from_slice(&0u16.to_be_bytes()); // urgent ptr
    // Fill in checksum
    let csum = crate::netstack::tcp::tcp_checksum_external(source_ip, destination_ip, &seg);
    seg[16] = (csum >> 8) as u8;
    seg[17] = (csum & 0xFF) as u8;
    seg
}

/// Build an ICMP echo request (ping).
pub fn craft_icmp_echo(id: u16, sequence_number: u16, payload: &[u8]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(8 + payload.len());
    packet.push(8); // type = echo request
    packet.push(0); // code
    packet.extend_from_slice(&0u16.to_be_bytes()); // checksum placeholder
    packet.extend_from_slice(&id.to_be_bytes());
    packet.extend_from_slice(&sequence_number.to_be_bytes());
    packet.extend_from_slice(payload);
    // ICMP checksum
    let csum = icmp_checksum(&packet);
    packet[2] = (csum >> 8) as u8;
    packet[3] = (csum & 0xFF) as u8;
    packet
}

/// Build a UDP datagram.
pub fn craft_udp(source_port: u16, destination_port: u16, payload: &[u8]) -> Vec<u8> {
    let len = 8 + payload.len() as u16;
    let mut packet = Vec::with_capacity(len as usize);
    packet.extend_from_slice(&source_port.to_be_bytes());
    packet.extend_from_slice(&destination_port.to_be_bytes());
    packet.extend_from_slice(&len.to_be_bytes());
    packet.extend_from_slice(&0u16.to_be_bytes()); // checksum (optional for UDP/IPv4)
    packet.extend_from_slice(payload);
    packet
}

// ── Replay from sniffer capture ────────────────────────────────────────

/// Replay all packets currently in the sniffer capture buffer.
pub fn replay_capture() -> ReplayResult {
    let captured = super::sniffer::get_captured_packets();
    let mut sent = 0u64;
    let mut fail = 0u64;
    for packet in &captured {
        if packet.raw_data.len() >= 14 {
                        // Correspondance de motifs — branchement exhaustif de Rust.
match inject_raw(&packet.raw_data) {
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
        information: format!("Replayed {}/{} packets", sent, sent + fail),
    }
}

/// Replay a single captured packet by index.
pub fn replay_packet(index: usize) -> Result<(), &'static str> {
    let captured = super::sniffer::get_captured_packets();
    let packet = captured.get(index).ok_or("packet index out of range")?;
    if packet.raw_data.len() < 14 {
        return Err("captured packet too short");
    }
    inject_raw(&packet.raw_data)
}

// ── Security test probes ───────────────────────────────────────────────

/// Send N SYN packets to a target (for firewall / IDS stress testing).
/// Returns number of packets successfully sent.
pub fn syn_flood_test(destination_ip: [u8; 4], destination_port: u16, count: u32) -> u32 {
    let source_ip = crate::network::get_ipv4_config()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([10, 0, 2, 15]);
    let mut ok = 0u32;
    for i in 0..count {
        let source_port = 1024 + (i as u16 % 64000);
        let sequence = crate::rng::secure_random_u32();
        let syn = craft_syn(source_ip, destination_ip, source_port, destination_port, sequence);
        // Send as raw IP (protocol 6 = TCP)
        if inject_ip(destination_ip, 6, &syn).is_ok() {
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
