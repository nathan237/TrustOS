











use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};


static OQ_: AtomicU64 = AtomicU64::new(0);


#[derive(Debug)]
pub struct Adm {
    pub packets_sent: u64,
    pub packets_failed: u64,
    pub info: String,
}




pub fn gcu(frame: &[u8]) -> Result<(), &'static str> {
    if frame.len() < 14 {
        return Err("frame too short (min 14 bytes for Ethernet header)");
    }
    crate::network::aha(frame)?;
    OQ_.fetch_add(1, Ordering::Relaxed);
    Ok(())
}


pub fn mqa(dest_ip: [u8; 4], protocol: u8, payload: &[u8]) -> Result<(), &'static str> {
    crate::netstack::ip::aha(dest_ip, protocol, payload)?;
    OQ_.fetch_add(1, Ordering::Relaxed);
    Ok(())
}




pub fn kzc(src_ip: [u8; 4], dst_ip: [u8; 4], src_port: u16, dst_port: u16, seq: u32) -> Vec<u8> {
    let mut gq = Vec::with_capacity(20);
    gq.extend_from_slice(&src_port.to_be_bytes());
    gq.extend_from_slice(&dst_port.to_be_bytes());
    gq.extend_from_slice(&seq.to_be_bytes());
    gq.extend_from_slice(&0u32.to_be_bytes()); 
    gq.push(0x50); 
    gq.push(0x02); 
    gq.extend_from_slice(&65535u16.to_be_bytes()); 
    gq.extend_from_slice(&0u16.to_be_bytes()); 
    gq.extend_from_slice(&0u16.to_be_bytes()); 
    
    let ig = crate::netstack::tcp::jlr(src_ip, dst_ip, &gq);
    gq[16] = (ig >> 8) as u8;
    gq[17] = (ig & 0xFF) as u8;
    gq
}


pub fn qbn(id: u16, seq_num: u16, payload: &[u8]) -> Vec<u8> {
    let mut fj = Vec::with_capacity(8 + payload.len());
    fj.push(8); 
    fj.push(0); 
    fj.extend_from_slice(&0u16.to_be_bytes()); 
    fj.extend_from_slice(&id.to_be_bytes());
    fj.extend_from_slice(&seq_num.to_be_bytes());
    fj.extend_from_slice(payload);
    
    let ig = gbl(&fj);
    fj[2] = (ig >> 8) as u8;
    fj[3] = (ig & 0xFF) as u8;
    fj
}


pub fn qbo(src_port: u16, dst_port: u16, payload: &[u8]) -> Vec<u8> {
    let len = 8 + payload.len() as u16;
    let mut fj = Vec::with_capacity(len as usize);
    fj.extend_from_slice(&src_port.to_be_bytes());
    fj.extend_from_slice(&dst_port.to_be_bytes());
    fj.extend_from_slice(&len.to_be_bytes());
    fj.extend_from_slice(&0u16.to_be_bytes()); 
    fj.extend_from_slice(payload);
    fj
}




pub fn qua() -> Adm {
    let captured = super::sniffer::fyk();
    let mut deg = 0u64;
    let mut gv = 0u64;
    for fj in &captured {
        if fj.raw_data.len() >= 14 {
            match gcu(&fj.raw_data) {
                Ok(()) => deg += 1,
                Err(_) => gv += 1,
            }
        } else {
            gv += 1;
        }
    }
    Adm {
        packets_sent: deg,
        packets_failed: gv,
        info: format!("Replayed {}/{} packets", deg, deg + gv),
    }
}


pub fn qub(index: usize) -> Result<(), &'static str> {
    let captured = super::sniffer::fyk();
    let fj = captured.get(index).ok_or("packet index out of range")?;
    if fj.raw_data.len() < 14 {
        return Err("captured packet too short");
    }
    gcu(&fj.raw_data)
}





pub fn qyj(dst_ip: [u8; 4], dst_port: u16, count: u32) -> u32 {
    let src_ip = crate::network::rd()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([10, 0, 2, 15]);
    let mut ok = 0u32;
    for i in 0..count {
        let src_port = 1024 + (i as u16 % 64000);
        let seq = crate::rng::omx();
        let fbx = kzc(src_ip, dst_ip, src_port, dst_port, seq);
        
        if mqa(dst_ip, 6, &fbx).is_ok() {
            ok += 1;
        }
    }
    ok
}


pub fn pyg(target_ip: [u8; 4]) -> Result<(), &'static str> {
    crate::netstack::arp::bos(target_ip)?;
    OQ_.fetch_add(1, Ordering::Relaxed);
    Ok(())
}




pub fn plz() -> u64 {
    OQ_.load(Ordering::Relaxed)
}


pub fn jai() {
    OQ_.store(0, Ordering::Relaxed);
}



fn gbl(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += ((data[i] as u32) << 8) | (data[i + 1] as u32);
        i += 2;
    }
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}
