//! Network Protocol Stack
//!
//! Implements ARP, IP, ICMP, UDP, TCP, DHCP, DNS, HTTP protocols.
//! Also provides BSD-style socket API for userspace applications.

pub mod arp;
pub mod ip;
pub mod icmp;
pub mod tcp;
pub mod udp;
pub mod dhcp;
pub mod dns;
pub mod http;
pub mod https;
pub mod socket;
pub mod ipv6;
pub mod icmpv6;

use alloc::vec::Vec;
use alloc::collections::VecDeque;
use spin::Mutex;

/// Ethernet frame header
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct EthernetFrame {
    pub dst_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ethertype: u16,  // Big endian!
}

impl EthernetFrame {
    pub const SIZE: usize = 14;
    
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        
        Some(unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Self) })
    }
    
    pub fn ethertype(&self) -> u16 {
        u16::from_be(self.ethertype)
    }
}

/// EtherType constants
pub mod ethertype {
    pub const IPV4: u16 = 0x0800;
    pub const ARP: u16 = 0x0806;
    pub const IPV6: u16 = 0x86DD;
}

/// Received packet queue (for processing)
static RX_QUEUE: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());

/// Process incoming packet
pub fn process_packet(data: Vec<u8>) {
    if data.len() < EthernetFrame::SIZE {
        return;
    }

    // Feed packet to sniffer if capture is active
    crate::netscan::sniffer::process_packet(&data);
    
    let frame = match EthernetFrame::parse(&data) {
        Some(f) => f,
        None => return,
    };
    
    let payload = &data[EthernetFrame::SIZE..];
    
    match frame.ethertype() {
        ethertype::ARP => {
            arp::handle_packet(payload);
        }
        ethertype::IPV4 => {
            ip::handle_packet(payload);
        }
        ethertype::IPV6 => {
            ipv6::handle_packet(payload);
        }
        _ => {
            // Unknown protocol, ignore
        }
    }
}

/// Poll network driver and process packets
pub fn poll() {
    // Poll driver
    crate::drivers::net::poll();
    
    // Process received packets
    while let Some(packet) = crate::drivers::net::receive() {
        process_packet(packet);
    }
    
    // Poll DHCP client
    dhcp::poll();
}

/// Send raw ethernet frame
pub fn send_frame(dst_mac: [u8; 6], ethertype: u16, payload: &[u8]) -> Result<(), &'static str> {
    let src_mac = crate::drivers::net::get_mac()
        .or_else(crate::network::get_mac_address)
        .ok_or("No MAC address")?;
    
    let mut frame = Vec::with_capacity(64); // Minimum Ethernet frame size
    frame.extend_from_slice(&dst_mac);
    frame.extend_from_slice(&src_mac);
    frame.extend_from_slice(&ethertype.to_be_bytes());
    frame.extend_from_slice(payload);
    
    // Pad to minimum Ethernet frame size (60 bytes without FCS)
    while frame.len() < 60 {
        frame.push(0);
    }
    
    crate::drivers::net::send(&frame)
}
