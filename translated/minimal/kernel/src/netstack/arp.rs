



use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;


#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ArpPacket {
    pub ock: u16,       
    pub frq: u16,       
    pub tpg: u8,         
    pub hvh: u8,         
    pub ayh: u16,   
    pub eik: [u8; 6],
    pub eij: [u8; 4],
    pub jsl: [u8; 6],
    pub blk: [u8; 4],
}

impl ArpPacket {
    pub const Am: usize = 28;
    
    pub fn parse(f: &[u8]) -> Option<Self> {
        if f.len() < Self::Am {
            return None;
        }
        Some(unsafe { core::ptr::md(f.fq() as *const Self) })
    }
    
    pub fn ayh(&self) -> u16 {
        u16::eqv(self.ayh)
    }
}


static RC_: Mutex<BTreeMap<u32, [u8; 6]>> = Mutex::new(BTreeMap::new());


pub fn bur(f: &[u8]) {
    let ex = match ArpPacket::parse(f) {
        Some(ai) => ai,
        None => return,
    };
    
    
    if u16::eqv(ex.ock) != 1 || u16::eqv(ex.frq) != 0x0800 {
        return;
    }
    
    let eij = u32::oa(ex.eij);
    
    
    {
        let mut bdq = RC_.lock();
        bdq.insert(eij, ex.eik);
    }
    
    match ex.ayh() {
        1 => {
            
            lba(&ex);
        }
        2 => {
            
            crate::log_debug!("[ARP] Reply from {}.{}.{}.{} = {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                ex.eij[0], ex.eij[1], ex.eij[2], ex.eij[3],
                ex.eik[0], ex.eik[1], ex.eik[2],
                ex.eik[3], ex.eik[4], ex.eik[5]);
        }
        _ => {}
    }
}


fn lba(ex: &ArpPacket) {
    
    let aro = match crate::network::gif() {
        Some((_, Some(ip), _)) => ip,
        _ => return,
    };
    
    let xaw = u32::oa(ex.blk);
    let uzm = aro.as_bytes();
    let uzn = u32::oa(*uzm);
    
    if xaw != uzn {
        return; 
    }
    
    
    let _ = whm(ex.eij, ex.eik);
}


fn whm(blk: [u8; 4], jsl: [u8; 6]) -> Result<(), &'static str> {
    let lqz = crate::drivers::net::cez()
        .or_else(crate::network::ckt)
        .ok_or("No MAC")?;
    let aro = match crate::network::gif() {
        Some((_, Some(ip), _)) => *ip.as_bytes(),
        _ => return Err("No IP"),
    };
    
    let mut ex = Vec::fc(ArpPacket::Am);
    ex.bk(&1u16.ft());      
    ex.bk(&0x0800u16.ft()); 
    ex.push(6);                                     
    ex.push(4);                                     
    ex.bk(&2u16.ft());      
    ex.bk(&lqz);                 
    ex.bk(&aro);                  
    ex.bk(&jsl);              
    ex.bk(&blk);               
    
    crate::netstack::fug(jsl, crate::netstack::ethertype::Aot, &ex)?;
    
    crate::log_debug!("[ARP] Sent reply to {}.{}.{}.{}",
        blk[0], blk[1], blk[2], blk[3]);
    
    Ok(())
}


pub fn eii(blk: [u8; 4]) -> Result<(), &'static str> {
    let lqz = crate::drivers::net::cez()
        .or_else(crate::network::ckt)
        .ok_or("No MAC")?;
    let aro = match crate::network::gif() {
        Some((_, Some(ip), _)) => *ip.as_bytes(),
        _ => return Err("No IP"),
    };
    
    let mut ex = Vec::fc(ArpPacket::Am);
    ex.bk(&1u16.ft());      
    ex.bk(&0x0800u16.ft()); 
    ex.push(6);                                     
    ex.push(4);                                     
    ex.bk(&1u16.ft());      
    ex.bk(&lqz);                 
    ex.bk(&aro);                  
    ex.bk(&[0; 6]);                  
    ex.bk(&blk);               
    
    let nad = [0xFF; 6];
    crate::netstack::fug(nad, crate::netstack::ethertype::Aot, &ex)?;
    
    crate::log_debug!("[ARP] Sent request for {}.{}.{}.{}",
        blk[0], blk[1], blk[2], blk[3]);
    
    Ok(())
}


pub fn cga(ip: u32) -> Option<[u8; 6]> {
    RC_.lock().get(&ip).hu()
}


pub fn ayo(ip: [u8; 4]) -> Option<[u8; 6]> {
    let flx = u32::oa(ip);
    cga(flx)
}


pub fn yhe() -> usize {
    RC_.lock().len()
}


pub fn ch() -> alloc::vec::Vec<(u32, [u8; 6])> {
    let bdq = RC_.lock();
    bdq.iter().map(|(ip, ed)| (*ip, *ed)).collect()
}
