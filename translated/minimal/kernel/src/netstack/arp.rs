



use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;


#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ArpPacket {
    pub htype: u16,       
    pub ptype: u16,       
    pub hlen: u8,         
    pub plen: u8,         
    pub operation: u16,   
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
}

impl ArpPacket {
    pub const Z: usize = 28;
    
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::Z {
            return None;
        }
        Some(unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Self) })
    }
    
    pub fn operation(&self) -> u16 {
        u16::from_be(self.operation)
    }
}


static RY_: Mutex<BTreeMap<u32, [u8; 6]>> = Mutex::new(BTreeMap::new());


pub fn alq(data: &[u8]) {
    let be = match ArpPacket::parse(data) {
        Some(aa) => aa,
        None => return,
    };
    
    
    if u16::from_be(be.htype) != 1 || u16::from_be(be.ptype) != 0x0800 {
        return;
    }
    
    let sender_ip = u32::from_be_bytes(be.sender_ip);
    
    
    {
        let mut adk = RY_.lock();
        adk.insert(sender_ip, be.sender_mac);
    }
    
    match be.operation() {
        1 => {
            
            handle_request(&be);
        }
        2 => {
            
            crate::log_debug!("[ARP] Reply from {}.{}.{}.{} = {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                be.sender_ip[0], be.sender_ip[1], be.sender_ip[2], be.sender_ip[3],
                be.sender_mac[0], be.sender_mac[1], be.sender_mac[2],
                be.sender_mac[3], be.sender_mac[4], be.sender_mac[5]);
        }
        _ => {}
    }
}


fn handle_request(be: &ArpPacket) {
    
    let wj = match crate::network::cyp() {
        Some((_, Some(ip), _)) => ip,
        _ => return,
    };
    
    let pdf = u32::from_be_bytes(be.target_ip);
    let nob = wj.as_bytes();
    let noc = u32::from_be_bytes(*nob);
    
    if pdf != noc {
        return; 
    }
    
    
    let _ = onz(be.sender_ip, be.sender_mac);
}


fn onz(target_ip: [u8; 4], target_mac: [u8; 6]) -> Result<(), &'static str> {
    let glf = crate::drivers::net::aqt()
        .or_else(crate::network::aqu)
        .ok_or("No MAC")?;
    let wj = match crate::network::cyp() {
        Some((_, Some(ip), _)) => *ip.as_bytes(),
        _ => return Err("No IP"),
    };
    
    let mut be = Vec::with_capacity(ArpPacket::Z);
    be.extend_from_slice(&1u16.to_be_bytes());      
    be.extend_from_slice(&0x0800u16.to_be_bytes()); 
    be.push(6);                                     
    be.push(4);                                     
    be.extend_from_slice(&2u16.to_be_bytes());      
    be.extend_from_slice(&glf);                 
    be.extend_from_slice(&wj);                  
    be.extend_from_slice(&target_mac);              
    be.extend_from_slice(&target_ip);               
    
    crate::netstack::cdq(target_mac, crate::netstack::ethertype::Qz, &be)?;
    
    crate::log_debug!("[ARP] Sent reply to {}.{}.{}.{}",
        target_ip[0], target_ip[1], target_ip[2], target_ip[3]);
    
    Ok(())
}


pub fn bos(target_ip: [u8; 4]) -> Result<(), &'static str> {
    let glf = crate::drivers::net::aqt()
        .or_else(crate::network::aqu)
        .ok_or("No MAC")?;
    let wj = match crate::network::cyp() {
        Some((_, Some(ip), _)) => *ip.as_bytes(),
        _ => return Err("No IP"),
    };
    
    let mut be = Vec::with_capacity(ArpPacket::Z);
    be.extend_from_slice(&1u16.to_be_bytes());      
    be.extend_from_slice(&0x0800u16.to_be_bytes()); 
    be.push(6);                                     
    be.push(4);                                     
    be.extend_from_slice(&1u16.to_be_bytes());      
    be.extend_from_slice(&glf);                 
    be.extend_from_slice(&wj);                  
    be.extend_from_slice(&[0; 6]);                  
    be.extend_from_slice(&target_ip);               
    
    let hiq = [0xFF; 6];
    crate::netstack::cdq(hiq, crate::netstack::ethertype::Qz, &be)?;
    
    crate::log_debug!("[ARP] Sent request for {}.{}.{}.{}",
        target_ip[0], target_ip[1], target_ip[2], target_ip[3]);
    
    Ok(())
}


pub fn lookup(ip: u32) -> Option<[u8; 6]> {
    RY_.lock().get(&ip).copied()
}


pub fn yb(ip: [u8; 4]) -> Option<[u8; 6]> {
    let clj = u32::from_be_bytes(ip);
    lookup(clj)
}


pub fn pzb() -> usize {
    RY_.lock().len()
}


pub fn entries() -> alloc::vec::Vec<(u32, [u8; 6])> {
    let adk = RY_.lock();
    adk.iter().map(|(ip, mac)| (*ip, *mac)).collect()
}
