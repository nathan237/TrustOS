




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


#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct EthernetFrame {
    pub dst_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ethertype: u16,  
}

impl EthernetFrame {
    pub const Z: usize = 14;
    
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::Z {
            return None;
        }
        
        Some(unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Self) })
    }
    
    pub fn ethertype(&self) -> u16 {
        u16::from_be(self.ethertype)
    }
}


pub mod ethertype {
    pub const Tb: u16 = 0x0800;
    pub const Qz: u16 = 0x0806;
    pub const Zz: u16 = 0x86DD;
}


static AJG_: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());


pub fn exa(data: Vec<u8>) {
    if data.len() < EthernetFrame::Z {
        return;
    }

    
    crate::netscan::sniffer::exa(&data);
    
    let frame = match EthernetFrame::parse(&data) {
        Some(f) => f,
        None => return,
    };
    
    let payload = &data[EthernetFrame::Z..];
    
    match frame.ethertype() {
        ethertype::Qz => {
            arp::alq(payload);
        }
        ethertype::Tb => {
            ip::alq(payload);
        }
        ethertype::Zz => {
            ipv6::alq(payload);
        }
        _ => {
            
        }
    }
}


pub fn poll() {
    
    crate::drivers::net::poll();
    
    
    while let Some(be) = crate::drivers::net::receive() {
        exa(be);
    }
    
    
    dhcp::poll();
    
    
    tftpd::poll();

    
    tcp::kjp();
}


pub fn cdq(dst_mac: [u8; 6], ethertype: u16, payload: &[u8]) -> Result<(), &'static str> {
    let src_mac = crate::drivers::net::aqt()
        .or_else(crate::network::aqu)
        .ok_or("No MAC address")?;
    
    let mut frame = Vec::with_capacity(64); 
    frame.extend_from_slice(&dst_mac);
    frame.extend_from_slice(&src_mac);
    frame.extend_from_slice(&ethertype.to_be_bytes());
    frame.extend_from_slice(payload);
    
    
    while frame.len() < 60 {
        frame.push(0);
    }
    
    crate::drivers::net::send(&frame)
}
