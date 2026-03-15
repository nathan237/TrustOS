




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
    pub amc: [u8; 6],
    pub atn: [u8; 6],
    pub ethertype: u16,  
}

impl EthernetFrame {
    pub const Am: usize = 14;
    
    pub fn parse(f: &[u8]) -> Option<Self> {
        if f.len() < Self::Am {
            return None;
        }
        
        Some(unsafe { core::ptr::md(f.fq() as *const Self) })
    }
    
    pub fn ethertype(&self) -> u16 {
        u16::eqv(self.ethertype)
    }
}


pub mod ethertype {
    pub const Aty: u16 = 0x0800;
    pub const Aot: u16 = 0x0806;
    pub const Bjg: u16 = 0x86DD;
}


static AHK_: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());


pub fn jkc(f: Vec<u8>) {
    if f.len() < EthernetFrame::Am {
        return;
    }

    
    crate::netscan::sniffer::jkc(&f);
    
    let frame = match EthernetFrame::parse(&f) {
        Some(bb) => bb,
        None => return,
    };
    
    let ew = &f[EthernetFrame::Am..];
    
    match frame.ethertype() {
        ethertype::Aot => {
            arp::bur(ew);
        }
        ethertype::Aty => {
            ip::bur(ew);
        }
        ethertype::Bjg => {
            ipv6::bur(ew);
        }
        _ => {
            
        }
    }
}


pub fn poll() {
    
    crate::drivers::net::poll();
    
    
    while let Some(ex) = crate::drivers::net::chb() {
        jkc(ex);
    }
    
    
    dhcp::poll();
    
    
    tftpd::poll();

    
    tcp::qzp();
}


pub fn fug(amc: [u8; 6], ethertype: u16, ew: &[u8]) -> Result<(), &'static str> {
    let atn = crate::drivers::net::cez()
        .or_else(crate::network::ckt)
        .ok_or("No MAC address")?;
    
    let mut frame = Vec::fc(64); 
    frame.bk(&amc);
    frame.bk(&atn);
    frame.bk(&ethertype.ft());
    frame.bk(ew);
    
    
    while frame.len() < 60 {
        frame.push(0);
    }
    
    crate::drivers::net::baq(&frame)
}
