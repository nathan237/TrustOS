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
pub mod dhcpd;
pub mod tftpd;
pub mod dns;
pub mod http;
pub mod https;
pub mod socket;
pub mod ipv6;
pub mod icmpv6;
pub mod firewall;

use alloc::vec::Vec;
use alloc::collections::VecDeque;
use spin::Mutex;

/// Ethernet frame header
#[repr(C, packed)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct EthernetFrame {
    pub dst_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ethertype: u16,  // Big endian!
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl EthernetFrame {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SIZE: usize = 14;
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        
        Some(        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Self) })
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn ethertype(&self) -> u16 {
        u16::from_be(self.ethertype)
    }
}

/// EtherType constants
pub mod ethertype {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IPV4: u16 = 0x0800;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ARP: u16 = 0x0806;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IPV6: u16 = 0x86DD;
}

/// Received packet queue (for processing)
static RECEIVE_QUEUE: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());

/// Process incoming packet
pub fn process_packet(data: Vec<u8>) {
    if data.len() < EthernetFrame::SIZE {
        return;
    }

    // Feed packet to sniffer if capture is active
    crate::netscan::sniffer::process_packet(&data);
    
    let frame = // Correspondance de motifs — branchement exhaustif de Rust.
match EthernetFrame::parse(&data) {
        Some(f) => f,
        None => return,
    };
    
    let payload = &data[EthernetFrame::SIZE..];
    
        // Correspondance de motifs — branchement exhaustif de Rust.
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
    
    // Poll TFTP server retransmissions
    tftpd::poll();

    // Check TCP retransmissions
    tcp::check_retransmits();
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
