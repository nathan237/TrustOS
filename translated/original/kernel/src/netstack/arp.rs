//! ARP Protocol Implementation
//!
//! Address Resolution Protocol - maps IP to MAC addresses.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

/// ARP packet structure
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ArpPacket {
    pub htype: u16,       // Hardware type (1 = Ethernet)
    pub ptype: u16,       // Protocol type (0x0800 = IPv4)
    pub hlen: u8,         // Hardware address length (6)
    pub plen: u8,         // Protocol address length (4)
    pub operation: u16,   // 1 = request, 2 = reply
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
}

impl ArpPacket {
    pub const SIZE: usize = 28;
    
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        Some(unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Self) })
    }
    
    pub fn operation(&self) -> u16 {
        u16::from_be(self.operation)
    }
}

/// ARP cache: IP -> MAC mapping
static ARP_CACHE: Mutex<BTreeMap<u32, [u8; 6]>> = Mutex::new(BTreeMap::new());

/// Handle incoming ARP packet
pub fn handle_packet(data: &[u8]) {
    let packet = match ArpPacket::parse(data) {
        Some(p) => p,
        None => return,
    };
    
    // Only handle Ethernet + IPv4
    if u16::from_be(packet.htype) != 1 || u16::from_be(packet.ptype) != 0x0800 {
        return;
    }
    
    let sender_ip = u32::from_be_bytes(packet.sender_ip);
    
    // Update ARP cache
    {
        let mut cache = ARP_CACHE.lock();
        cache.insert(sender_ip, packet.sender_mac);
    }
    
    match packet.operation() {
        1 => {
            // ARP Request
            handle_request(&packet);
        }
        2 => {
            // ARP Reply
            crate::log_debug!("[ARP] Reply from {}.{}.{}.{} = {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                packet.sender_ip[0], packet.sender_ip[1], packet.sender_ip[2], packet.sender_ip[3],
                packet.sender_mac[0], packet.sender_mac[1], packet.sender_mac[2],
                packet.sender_mac[3], packet.sender_mac[4], packet.sender_mac[5]);
        }
        _ => {}
    }
}

/// Handle ARP request
fn handle_request(packet: &ArpPacket) {
    // Check if it's for our IP
    let our_ip = match crate::network::get_interface() {
        Some((_, Some(ip), _)) => ip,
        _ => return,
    };
    
    let target_ip_u32 = u32::from_be_bytes(packet.target_ip);
    let our_ip_bytes = our_ip.as_bytes();
    let our_ip_u32 = u32::from_be_bytes(*our_ip_bytes);
    
    if target_ip_u32 != our_ip_u32 {
        return; // Not for us
    }
    
    // Send ARP reply
    let _ = send_reply(packet.sender_ip, packet.sender_mac);
}

/// Send ARP reply
fn send_reply(target_ip: [u8; 4], target_mac: [u8; 6]) -> Result<(), &'static str> {
    let our_mac = crate::drivers::net::get_mac()
        .or_else(crate::network::get_mac_address)
        .ok_or("No MAC")?;
    let our_ip = match crate::network::get_interface() {
        Some((_, Some(ip), _)) => *ip.as_bytes(),
        _ => return Err("No IP"),
    };
    
    let mut packet = Vec::with_capacity(ArpPacket::SIZE);
    packet.extend_from_slice(&1u16.to_be_bytes());      // Hardware type: Ethernet
    packet.extend_from_slice(&0x0800u16.to_be_bytes()); // Protocol type: IPv4
    packet.push(6);                                     // Hardware size
    packet.push(4);                                     // Protocol size
    packet.extend_from_slice(&2u16.to_be_bytes());      // Operation: Reply
    packet.extend_from_slice(&our_mac);                 // Sender MAC
    packet.extend_from_slice(&our_ip);                  // Sender IP
    packet.extend_from_slice(&target_mac);              // Target MAC
    packet.extend_from_slice(&target_ip);               // Target IP
    
    crate::netstack::send_frame(target_mac, crate::netstack::ethertype::ARP, &packet)?;
    
    crate::log_debug!("[ARP] Sent reply to {}.{}.{}.{}",
        target_ip[0], target_ip[1], target_ip[2], target_ip[3]);
    
    Ok(())
}

/// Send ARP request
pub fn send_request(target_ip: [u8; 4]) -> Result<(), &'static str> {
    let our_mac = crate::drivers::net::get_mac()
        .or_else(crate::network::get_mac_address)
        .ok_or("No MAC")?;
    let our_ip = match crate::network::get_interface() {
        Some((_, Some(ip), _)) => *ip.as_bytes(),
        _ => return Err("No IP"),
    };
    
    let mut packet = Vec::with_capacity(ArpPacket::SIZE);
    packet.extend_from_slice(&1u16.to_be_bytes());      // Hardware type: Ethernet
    packet.extend_from_slice(&0x0800u16.to_be_bytes()); // Protocol type: IPv4
    packet.push(6);                                     // Hardware size
    packet.push(4);                                     // Protocol size
    packet.extend_from_slice(&1u16.to_be_bytes());      // Operation: Request
    packet.extend_from_slice(&our_mac);                 // Sender MAC
    packet.extend_from_slice(&our_ip);                  // Sender IP
    packet.extend_from_slice(&[0; 6]);                  // Target MAC (unknown)
    packet.extend_from_slice(&target_ip);               // Target IP
    
    let broadcast = [0xFF; 6];
    crate::netstack::send_frame(broadcast, crate::netstack::ethertype::ARP, &packet)?;
    
    crate::log_debug!("[ARP] Sent request for {}.{}.{}.{}",
        target_ip[0], target_ip[1], target_ip[2], target_ip[3]);
    
    Ok(())
}

/// Lookup MAC address for an IP
pub fn lookup(ip: u32) -> Option<[u8; 6]> {
    ARP_CACHE.lock().get(&ip).copied()
}

/// Resolve IP to MAC (convenience function)
pub fn resolve(ip: [u8; 4]) -> Option<[u8; 6]> {
    let ip_u32 = u32::from_be_bytes(ip);
    lookup(ip_u32)
}

/// Get ARP cache size
pub fn cache_size() -> usize {
    ARP_CACHE.lock().len()
}

/// Get ARP cache entries (ip_u32, mac)
pub fn entries() -> alloc::vec::Vec<(u32, [u8; 6])> {
    let cache = ARP_CACHE.lock();
    cache.iter().map(|(ip, mac)| (*ip, *mac)).collect()
}
