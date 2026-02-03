//! ICMP Protocol (Internet Control Message Protocol)

use alloc::vec::Vec;
use spin::Mutex;

/// ICMP types
pub const ICMP_ECHO_REPLY: u8 = 0;
pub const ICMP_ECHO_REQUEST: u8 = 8;

/// Pending ping responses
static PING_RESPONSES: Mutex<Vec<PingResponse>> = Mutex::new(Vec::new());

/// Ping response data
#[derive(Debug, Clone, Copy)]
pub struct PingResponse {
    pub seq: u16,
    pub ttl: u8,
    pub success: bool,
}

/// ICMP packet structure (minimum 8 bytes header)
#[repr(C, packed)]
struct IcmpHeader {
    type_: u8,
    code: u8,
    checksum: u16,
    identifier: u16,
    sequence: u16,
}

/// Calculate Internet checksum (RFC 1071)
fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    
    // Sum 16-bit words
    while i < data.len() - 1 {
        sum += ((data[i] as u32) << 8) | (data[i + 1] as u32);
        i += 2;
    }
    
    // Add odd byte if present
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    
    // Fold 32-bit sum to 16 bits
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    
    !sum as u16
}

/// Handle incoming ICMP packet
pub fn handle_packet(data: &[u8], ttl: u8) {
    if data.len() < 8 {
        return;
    }
    
    let type_ = data[0];
    let code = data[1];
    let id = u16::from_be_bytes([data[4], data[5]]);
    let seq = u16::from_be_bytes([data[6], data[7]]);
    
    match type_ {
        ICMP_ECHO_REQUEST => {
            // Respond to ping request
            crate::serial_println!("[ICMP] Echo request id={} seq={}", id, seq);
            send_echo_reply(id, seq, &data[8..]);
        }
        ICMP_ECHO_REPLY => {
            // Store ping response
            crate::serial_println!("[ICMP] Echo reply id={} seq={} ttl={}", id, seq, ttl);
            PING_RESPONSES.lock().push(PingResponse {
                seq,
                ttl,
                success: true,
            });
        }
        _ => {
            crate::serial_println!("[ICMP] Type {} code {} (unhandled)", type_, code);
        }
    }
}

/// Send ICMP echo request (ping)
pub fn send_echo_request(dest_ip: [u8; 4], id: u16, seq: u16) -> Result<(), &'static str> {
    // Build ICMP echo request
    let mut packet = Vec::new();
    
    // ICMP header
    packet.push(ICMP_ECHO_REQUEST); // Type
    packet.push(0); // Code
    packet.push(0); packet.push(0); // Checksum (will calculate)
    packet.extend_from_slice(&id.to_be_bytes());
    packet.extend_from_slice(&seq.to_be_bytes());
    
    // Payload (timestamp + padding)
    let timestamp = crate::time::uptime_ms() as u32;
    packet.extend_from_slice(&timestamp.to_be_bytes());
    for i in 0..52 {
        packet.push((0x10 + i) as u8); // Pattern data
    }
    
    // Calculate checksum
    let csum = checksum(&packet);
    packet[2] = (csum >> 8) as u8;
    packet[3] = (csum & 0xFF) as u8;
    
    // Send via IP layer
    crate::netstack::ip::send_packet(dest_ip, 1, &packet)?;
    
    crate::serial_println!("[ICMP] Sent echo request to {}.{}.{}.{} id={} seq={}", 
        dest_ip[0], dest_ip[1], dest_ip[2], dest_ip[3], id, seq);
    
    Ok(())
}

/// Send ICMP echo reply
fn send_echo_reply(id: u16, seq: u16, payload: &[u8]) {
    // Build ICMP echo reply
    let mut packet = Vec::new();
    
    packet.push(ICMP_ECHO_REPLY); // Type
    packet.push(0); // Code
    packet.push(0); packet.push(0); // Checksum (will calculate)
    packet.extend_from_slice(&id.to_be_bytes());
    packet.extend_from_slice(&seq.to_be_bytes());
    packet.extend_from_slice(payload);
    
    // Calculate checksum
    let csum = checksum(&packet);
    packet[2] = (csum >> 8) as u8;
    packet[3] = (csum & 0xFF) as u8;
    
    // TODO: Send back to source IP (need to track source in handle_packet)
    crate::serial_println!("[ICMP] Would send echo reply id={} seq={}", id, seq);
}

/// Wait for ping response
pub fn wait_for_response(seq: u16, timeout_ms: u32) -> Option<PingResponse> {
    let start = crate::logger::get_ticks();
    let mut spins: u32 = 0;
    
    loop {
        // Poll network to process incoming packets
        crate::netstack::poll();

        // Check if we have a response
        let mut responses = PING_RESPONSES.lock();
        if let Some(pos) = responses.iter().position(|r| r.seq == seq) {
            let response = responses.remove(pos);
            return Some(response);
        }
        drop(responses);
        
        // Check timeout
        if crate::logger::get_ticks() - start > timeout_ms as u64 {
            return None;
        }

        spins = spins.wrapping_add(1);
        if spins > 2_000_000 {
            return None;
        }
        
        // Yield CPU
        x86_64::instructions::hlt();
    }
}

/// Clear all pending responses
pub fn clear_responses() {
    PING_RESPONSES.lock().clear();
}
