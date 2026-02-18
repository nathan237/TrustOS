//! IP Protocol (IPv4)

use alloc::vec::Vec;

/// IPv4 header structure (minimum 20 bytes)
#[repr(C, packed)]
struct Ipv4Header {
    version_ihl: u8,      // Version (4 bits) + IHL (4 bits)
    dscp_ecn: u8,         // DSCP (6 bits) + ECN (2 bits)
    total_length: u16,
    identification: u16,
    flags_fragment: u16,  // Flags (3 bits) + Fragment offset (13 bits)
    ttl: u8,
    protocol: u8,
    checksum: u16,
    source: [u8; 4],
    dest: [u8; 4],
}

/// Calculate IP checksum
fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    
    while i < data.len() - 1 {
        sum += ((data[i] as u32) << 8) | (data[i + 1] as u32);
        i += 2;
    }
    
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    
    !sum as u16
}

/// Handle incoming IPv4 packet
pub fn handle_packet(data: &[u8]) {
    if data.len() < 20 {
        return;
    }
    
    let version = data[0] >> 4;
    let ihl = (data[0] & 0x0F) as usize;
    let header_len = ihl * 4;
    
    if version != 4 || data.len() < header_len {
        return;
    }
    
    // Get total length from IP header (bytes 2-3)
    let total_length = u16::from_be_bytes([data[2], data[3]]) as usize;
    
    // Validate total length
    if total_length < header_len || total_length > data.len() {
        return;
    }
    
    let ttl = data[8];
    let protocol = data[9];
    let source = [data[12], data[13], data[14], data[15]];
    let dest = [data[16], data[17], data[18], data[19]];
    
    // Use total_length to get actual payload, ignoring Ethernet padding
    let payload = &data[header_len..total_length];
    
    // (packet logging removed to keep serial clean)
    
    match protocol {
        1 => crate::netstack::icmp::handle_packet(payload, ttl, source), // ICMP
        6 => crate::netstack::tcp::handle_packet(payload, source, dest), // TCP
        17 => crate::netstack::udp::handle_packet(payload),      // UDP
        _ => {
            crate::serial_println!("[IP] Unsupported protocol {}", protocol);
        }
    }
}

/// Send IPv4 packet with custom TTL
pub fn send_packet_with_ttl(dest_ip: [u8; 4], protocol: u8, payload: &[u8], ttl: u8) -> Result<(), &'static str> {
    let (source_ip, subnet, gateway) = crate::network::get_ipv4_config()
        .map(|(ip, mask, gw)| (*ip.as_bytes(), *mask.as_bytes(), gw.map(|g| *g.as_bytes())))
        .unwrap_or(([192, 168, 56, 100], [255, 255, 255, 0], None));

    let on_same_subnet = |ip: [u8; 4]| {
        (ip[0] & subnet[0]) == (source_ip[0] & subnet[0]) &&
        (ip[1] & subnet[1]) == (source_ip[1] & subnet[1]) &&
        (ip[2] & subnet[2]) == (source_ip[2] & subnet[2]) &&
        (ip[3] & subnet[3]) == (source_ip[3] & subnet[3])
    };

    let next_hop_ip = if on_same_subnet(dest_ip) {
        dest_ip
    } else if let Some(gw) = gateway {
        if gw != [0, 0, 0, 0] { gw } else { dest_ip }
    } else {
        dest_ip
    };

    let mut header = Vec::new();
    let total_length = 20 + payload.len();
    header.push(0x45);
    header.push(0);
    header.extend_from_slice(&(total_length as u16).to_be_bytes());
    header.extend_from_slice(&0u16.to_be_bytes());
    header.extend_from_slice(&0x4000u16.to_be_bytes());
    header.push(ttl);
    header.push(protocol);
    header.push(0); header.push(0);
    header.extend_from_slice(&source_ip);
    header.extend_from_slice(&dest_ip);

    let csum = checksum(&header);
    header[10] = (csum >> 8) as u8;
    header[11] = (csum & 0xFF) as u8;

    let mut packet = header;
    packet.extend_from_slice(payload);

    let dest_mac = match crate::netstack::arp::resolve(next_hop_ip) {
        Some(mac) => mac,
        None => {
            crate::netstack::arp::send_request(next_hop_ip)?;
            let start = crate::logger::get_ticks();
            let mut spins: u32 = 0;
            loop {
                crate::netstack::poll();
                if let Some(mac) = crate::netstack::arp::resolve(next_hop_ip) {
                    break mac;
                }
                if crate::logger::get_ticks().saturating_sub(start) > 1000 {
                    return Err("ARP timeout");
                }
                spins = spins.wrapping_add(1);
                if spins > 2_000_000 { return Err("ARP timeout"); }
                x86_64::instructions::hlt();
            }
        }
    };

    crate::netstack::send_frame(dest_mac, 0x0800, &packet)
}

/// Send IPv4 packet
pub fn send_packet(dest_ip: [u8; 4], protocol: u8, payload: &[u8]) -> Result<(), &'static str> {
    // Get our IP/subnet/gateway (default to host-only network settings)
    let (source_ip, subnet, gateway) = crate::network::get_ipv4_config()
        .map(|(ip, mask, gw)| (*ip.as_bytes(), *mask.as_bytes(), gw.map(|g| *g.as_bytes())))
        .unwrap_or(([192, 168, 56, 100], [255, 255, 255, 0], None));

    // Decide next hop (route via gateway if off-subnet)
    let on_same_subnet = |ip: [u8; 4]| {
        (ip[0] & subnet[0]) == (source_ip[0] & subnet[0]) &&
        (ip[1] & subnet[1]) == (source_ip[1] & subnet[1]) &&
        (ip[2] & subnet[2]) == (source_ip[2] & subnet[2]) &&
        (ip[3] & subnet[3]) == (source_ip[3] & subnet[3])
    };

    let next_hop_ip = if on_same_subnet(dest_ip) {
        dest_ip
    } else if let Some(gw) = gateway {
        // Only use gateway if it's a valid address (not 0.0.0.0)
        if gw != [0, 0, 0, 0] {
            gw
        } else {
            dest_ip // No gateway, try direct
        }
    } else {
        dest_ip
    };
    
    // Build IPv4 header
    let mut header = Vec::new();
    let total_length = 20 + payload.len();
    
    header.push(0x45); // Version 4, IHL 5 (20 bytes)
    header.push(0);    // DSCP/ECN
    header.extend_from_slice(&(total_length as u16).to_be_bytes()); // Total length
    header.extend_from_slice(&0u16.to_be_bytes()); // Identification
    header.extend_from_slice(&0x4000u16.to_be_bytes()); // Flags: Don't fragment
    header.push(64);   // TTL
    header.push(protocol); // Protocol
    header.push(0); header.push(0); // Checksum (will calculate)
    header.extend_from_slice(&source_ip); // Source IP
    header.extend_from_slice(&dest_ip);   // Dest IP
    
    // Calculate header checksum
    let csum = checksum(&header);
    header[10] = (csum >> 8) as u8;
    header[11] = (csum & 0xFF) as u8;
    
    // (TCP header debug logging removed to keep serial clean)
    
    // Combine header + payload
    let mut packet = header;
    packet.extend_from_slice(payload);
    
    // Resolve MAC via ARP (next hop)
    let dest_mac = match crate::netstack::arp::resolve(next_hop_ip) {
        Some(mac) => mac,
        None => {
            // Send ARP request
            crate::netstack::arp::send_request(next_hop_ip)?;
            
            // Wait for ARP reply (with timeout)
            let start = crate::logger::get_ticks();
            let mut spins: u32 = 0;
            loop {
                crate::netstack::poll();

                if let Some(mac) = crate::netstack::arp::resolve(next_hop_ip) {
                    break mac;
                }
                if crate::logger::get_ticks().saturating_sub(start) > 1000 {
                    return Err("ARP timeout");
                }
                spins = spins.wrapping_add(1);
                if spins > 2_000_000 {
                    return Err("ARP timeout");
                }
                x86_64::instructions::hlt();
            }
        }
    };
    
    // Send via Ethernet
    crate::netstack::send_frame(dest_mac, 0x0800, &packet)?;
    
    // (send logging removed to keep serial clean)
    
    Ok(())
}
