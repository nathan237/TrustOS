

use alloc::vec::Vec;
use spin::Mutex;


pub const AYU_: u8 = 0;
pub const AYT_: u8 = 3;
pub const AYW_: u8 = 11;
pub const AYV_: u8 = 8;


static XP_: Mutex<Vec<Uk>> = Mutex::new(Vec::new());


static VI_: Mutex<Vec<Tc>> = Mutex::new(Vec::new());


#[derive(Debug, Clone, Copy)]
pub struct Tc {
    pub error_type: u8,        
    pub code: u8,
    pub source_ip: [u8; 4],   
    pub original_dest: [u8; 4], 
    pub original_proto: u8,     
    pub original_id: u16,       
}


#[derive(Debug, Clone, Copy)]
pub struct Uk {
    pub seq: u16,
    pub ttl: u8,
    pub success: bool,
}


#[repr(C, packed)]
struct Axu {
    ecw: u8,
    code: u8,
    checksum: u16,
    identifier: u16,
    sequence: u16,
}


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


pub fn alq(data: &[u8], ttl: u8, source_ip: [u8; 4]) {
    if data.len() < 8 {
        return;
    }
    
    let ecw = data[0];
    let code = data[1];
    let id = u16::from_be_bytes([data[4], data[5]]);
    let seq = u16::from_be_bytes([data[6], data[7]]);
    
    match ecw {
        AYV_ => {
            crate::serial_println!("[ICMP] Echo request id={} seq={}", id, seq);
            ono(id, seq, &data[8..]);
        }
        AYU_ => {
            crate::serial_println!("[ICMP] Echo reply id={} seq={} ttl={}", id, seq, ttl);
            XP_.lock().push(Uk {
                seq,
                ttl,
                success: true,
            });
        }
        AYW_ | AYT_ => {
            
            
            if data.len() >= 8 + 20 {
                let ciu = &data[8..];
                let dvz = [ciu[16], ciu[17], ciu[18], ciu[19]];
                let nnw = ciu[9];
                let nnu = u16::from_be_bytes([ciu[4], ciu[5]]);
                crate::serial_println!("[ICMP] {} from {}.{}.{}.{} code={} orig_dest={}.{}.{}.{}",
                    if ecw == AYW_ { "Time Exceeded" } else { "Dest Unreachable" },
                    source_ip[0], source_ip[1], source_ip[2], source_ip[3],
                    code,
                    dvz[0], dvz[1], dvz[2], dvz[3]);
                VI_.lock().push(Tc {
                    error_type: ecw,
                    code,
                    source_ip,
                    original_dest: dvz,
                    original_proto: nnw,
                    original_id: nnu,
                });
            }
        }
        _ => {
            crate::serial_println!("[ICMP] Type {} code {} (unhandled)", ecw, code);
        }
    }
}


pub fn gtw(dest_ip: [u8; 4], id: u16, seq: u16) -> Result<(), &'static str> {
    
    let mut be = Vec::new();
    
    
    be.push(AYV_); 
    be.push(0); 
    be.push(0); be.push(0); 
    be.extend_from_slice(&id.to_be_bytes());
    be.extend_from_slice(&seq.to_be_bytes());
    
    
    let timestamp = crate::time::uptime_ms() as u32;
    be.extend_from_slice(&timestamp.to_be_bytes());
    for i in 0..52 {
        be.push((0x10 + i) as u8); 
    }
    
    
    let ig = checksum(&be);
    be[2] = (ig >> 8) as u8;
    be[3] = (ig & 0xFF) as u8;
    
    
    crate::netstack::ip::aha(dest_ip, 1, &be)?;
    
    crate::serial_println!("[ICMP] Sent echo request to {}.{}.{}.{} id={} seq={}", 
        dest_ip[0], dest_ip[1], dest_ip[2], dest_ip[3], id, seq);
    
    Ok(())
}


fn ono(id: u16, seq: u16, payload: &[u8]) {
    
    let mut be = Vec::new();
    
    be.push(AYU_); 
    be.push(0); 
    be.push(0); be.push(0); 
    be.extend_from_slice(&id.to_be_bytes());
    be.extend_from_slice(&seq.to_be_bytes());
    be.extend_from_slice(payload);
    
    
    let ig = checksum(&be);
    be[2] = (ig >> 8) as u8;
    be[3] = (ig & 0xFF) as u8;
    
    
    crate::serial_println!("[ICMP] Would send echo reply id={} seq={}", id, seq);
}


pub fn hcb(seq: u16, timeout_ms: u32) -> Option<Uk> {
    let start = crate::logger::eg();
    let mut my: u32 = 0;
    
    loop {
        
        crate::netstack::poll();

        
        let mut ddo = XP_.lock();
        if let Some(pos) = ddo.iter().position(|r| r.seq == seq) {
            let fa = ddo.remove(pos);
            return Some(fa);
        }
        drop(ddo);
        
        
        if crate::logger::eg() - start > timeout_ms as u64 {
            return None;
        }

        my = my.wrapping_add(1);
        if my > 2_000_000 {
            return None;
        }
        
        
        crate::arch::acb();
    }
}


pub fn dkt() {
    XP_.lock().clear();
}


pub fn pti(dest_ip: [u8; 4], timeout_ms: u32) -> Option<Tc> {
    let start = crate::logger::eg();
    let mut my: u32 = 0;
    loop {
        crate::netstack::poll();

        let mut errors = VI_.lock();
        if let Some(pos) = errors.iter().position(|e| e.original_dest == dest_ip) {
            return Some(errors.remove(pos));
        }
        drop(errors);

        if crate::logger::eg().saturating_sub(start) > timeout_ms as u64 {
            return None;
        }
        my = my.wrapping_add(1);
        if my > 2_000_000 { return None; }
        crate::arch::acb();
    }
}


pub fn ptk(seq: u16, dest_ip: [u8; 4], timeout_ms: u32) -> TracerouteResult {
    let start = crate::logger::eg();
    let mut my: u32 = 0;
    loop {
        crate::netstack::poll();

        
        {
            let mut ddo = XP_.lock();
            if let Some(pos) = ddo.iter().position(|r| r.seq == seq) {
                let eo = ddo.remove(pos);
                let bb = crate::logger::eg().saturating_sub(start);
                return TracerouteResult::Reached { ip: dest_ip, ttl: eo.ttl, rtt_ms: bb };
            }
        }

        
        {
            let mut errors = VI_.lock();
            if let Some(pos) = errors.iter().position(|e| e.original_dest == dest_ip) {
                let err = errors.remove(pos);
                let bb = crate::logger::eg().saturating_sub(start);
                return TracerouteResult::Hop { ip: err.source_ip, rtt_ms: bb, error_type: err.error_type };
            }
        }

        if crate::logger::eg().saturating_sub(start) > timeout_ms as u64 {
            return TracerouteResult::Timeout;
        }
        my = my.wrapping_add(1);
        if my > 2_000_000 { return TracerouteResult::Timeout; }
        crate::arch::acb();
    }
}


pub fn hlh() {
    VI_.lock().clear();
}


#[derive(Debug, Clone, Copy)]
pub enum TracerouteResult {
    Hop { ip: [u8; 4], rtt_ms: u64, error_type: u8 },
    Reached { ip: [u8; 4], ttl: u8, rtt_ms: u64 },
    Timeout,
}
