//! ICMP Protocol (Internet Control Message Protocol)

use alloc::vec::Vec;
use spin::Mutex;

/// ICMP types
pub const ICMP_ECHO_REPLY: u8 = 0;
pub const ICMP_DEST_UNREACHABLE: u8 = 3;
pub const ICMP_TIME_EXCEEDED: u8 = 11;
pub const ICMP_ECHO_REQUEST: u8 = 8;

/// Pending ping responses
static PING_RESPONSES: Mutex<Vec<PingResponse>> = Mutex::new(Vec::new());

/// ICMP error responses (time exceeded, destination unreachable)
static ICMP_ERRORS: Mutex<Vec<IcmpError>> = Mutex::new(Vec::new());

/// ICMP error info (for traceroute / scan detection)
#[derive(Debug, Clone, Copy)]
pub struct IcmpError {
    pub error_type: u8,        // 3=dest unreachable, 11=time exceeded
    pub code: u8,
    pub source_ip: [u8; 4],   // IP of the router that sent the error
    pub original_dest: [u8; 4], // original destination (from embedded IP header)
    pub original_proto: u8,     // original protocol
    pub original_id: u16,       // identification from original IP header
}

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
pub fn handle_packet(data: &[u8], ttl: u8, source_ip: [u8; 4]) {
    if data.len() < 8 {
        return;
    }
    
    let type_ = data[0];
    let code = data[1];
    let id = u16::from_be_bytes([data[4], data[5]]);
    let seq = u16::from_be_bytes([data[6], data[7]]);
    
    match type_ {
        ICMP_ECHO_REQUEST => {
            crate::serial_println!("[ICMP] Echo request id={} seq={}", id, seq);
            send_echo_reply(id, seq, &data[8..]);
        }
        ICMP_ECHO_REPLY => {
            crate::serial_println!("[ICMP] Echo reply id={} seq={} ttl={}", id, seq, ttl);
            PING_RESPONSES.lock().push(PingResponse {
                seq,
                ttl,
                success: true,
            });
        }
        ICMP_TIME_EXCEEDED | ICMP_DEST_UNREACHABLE => {
            // Extract embedded IP header from ICMP error payload
            // ICMP error: 8 bytes header + original IP header (>=20 bytes)
            if data.len() >= 8 + 20 {
                let embedded = &data[8..];
                let orig_dest = [embedded[16], embedded[17], embedded[18], embedded[19]];
                let orig_proto = embedded[9];
                let orig_id = u16::from_be_bytes([embedded[4], embedded[5]]);
                crate::serial_println!("[ICMP] {} from {}.{}.{}.{} code={} orig_dest={}.{}.{}.{}",
                    if type_ == ICMP_TIME_EXCEEDED { "Time Exceeded" } else { "Dest Unreachable" },
                    source_ip[0], source_ip[1], source_ip[2], source_ip[3],
                    code,
                    orig_dest[0], orig_dest[1], orig_dest[2], orig_dest[3]);
                ICMP_ERRORS.lock().push(IcmpError {
                    error_type: type_,
                    code,
                    source_ip,
                    original_dest: orig_dest,
                    original_proto: orig_proto,
                    original_id: orig_id,
                });
            }
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
        crate::arch::halt();
    }
}

/// Clear all pending responses
pub fn clear_responses() {
    PING_RESPONSES.lock().clear();
}

/// Wait for an ICMP error (time-exceeded or dest-unreachable) for a specific destination
pub fn wait_for_error(dest_ip: [u8; 4], timeout_ms: u32) -> Option<IcmpError> {
    let start = crate::logger::get_ticks();
    let mut spins: u32 = 0;
    loop {
        crate::netstack::poll();

        let mut errors = ICMP_ERRORS.lock();
        if let Some(pos) = errors.iter().position(|e| e.original_dest == dest_ip) {
            return Some(errors.remove(pos));
        }
        drop(errors);

        if crate::logger::get_ticks().saturating_sub(start) > timeout_ms as u64 {
            return None;
        }
        spins = spins.wrapping_add(1);
        if spins > 2_000_000 { return None; }
        crate::arch::halt();
    }
}

/// Wait for either a ping response OR an ICMP error (for traceroute)
pub fn wait_for_response_or_error(seq: u16, dest_ip: [u8; 4], timeout_ms: u32) -> TracerouteResult {
    let start = crate::logger::get_ticks();
    let mut spins: u32 = 0;
    loop {
        crate::netstack::poll();

        // Check for echo reply (reached destination)
        {
            let mut responses = PING_RESPONSES.lock();
            if let Some(pos) = responses.iter().position(|r| r.seq == seq) {
                let resp = responses.remove(pos);
                let elapsed = crate::logger::get_ticks().saturating_sub(start);
                return TracerouteResult::Reached { ip: dest_ip, ttl: resp.ttl, rtt_ms: elapsed };
            }
        }

        // Check for ICMP error (intermediate hop)
        {
            let mut errors = ICMP_ERRORS.lock();
            if let Some(pos) = errors.iter().position(|e| e.original_dest == dest_ip) {
                let err = errors.remove(pos);
                let elapsed = crate::logger::get_ticks().saturating_sub(start);
                return TracerouteResult::Hop { ip: err.source_ip, rtt_ms: elapsed, error_type: err.error_type };
            }
        }

        if crate::logger::get_ticks().saturating_sub(start) > timeout_ms as u64 {
            return TracerouteResult::Timeout;
        }
        spins = spins.wrapping_add(1);
        if spins > 2_000_000 { return TracerouteResult::Timeout; }
        crate::arch::halt();
    }
}

/// Clear pending ICMP errors
pub fn clear_errors() {
    ICMP_ERRORS.lock().clear();
}

/// Traceroute probe result
#[derive(Debug, Clone, Copy)]
pub enum TracerouteResult {
    Hop { ip: [u8; 4], rtt_ms: u64, error_type: u8 },
    Reached { ip: [u8; 4], ttl: u8, rtt_ms: u64 },
    Timeout,
}
