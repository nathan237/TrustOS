

use alloc::vec::Vec;




fn ilm() -> ([u8; 4], [u8; 4], Option<[u8; 4]>) {
    let mac = crate::drivers::net::aqt()
        .unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    
    let host = if mac[5] == 0 { 1 } else if mac[5] == 255 { 254 } else { mac[5] };
    ([10, 0, 100, host], [255, 255, 255, 0], None)
}



fn ihu(mt: [u8; 4], src: &[u8; 4], mask: &[u8; 4]) -> bool {
    
    if mt == [255, 255, 255, 255] {
        return true;
    }
    
    (mt[0] & !mask[0]) == !mask[0] &&
    (mt[1] & !mask[1]) == !mask[1] &&
    (mt[2] & !mask[2]) == !mask[2] &&
    (mt[3] & !mask[3]) == !mask[3]
}


#[repr(C, packed)]
struct Aya {
    version_ihl: u8,      
    dscp_ecn: u8,         
    bjq: u16,
    identification: u16,
    flags_fragment: u16,  
    ttl: u8,
    protocol: u8,
    checksum: u16,
    source: [u8; 4],
    mt: [u8; 4],
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


pub fn alq(data: &[u8]) {
    if data.len() < 20 {
        return;
    }
    
    let version = data[0] >> 4;
    let gbr = (data[0] & 0x0F) as usize;
    let bte = gbr * 4;
    
    if version != 4 || data.len() < bte {
        return;
    }
    
    
    let bjq = u16::from_be_bytes([data[2], data[3]]) as usize;
    
    
    if bjq < bte || bjq > data.len() {
        return;
    }
    
    let ttl = data[8];
    let protocol = data[9];
    let source = [data[12], data[13], data[14], data[15]];
    let mt = [data[16], data[17], data[18], data[19]];
    
    
    let payload = &data[bte..bjq];
    
    
    let (ais, ahv) = fvw(protocol, payload);
    if !crate::netstack::firewall::lvk(protocol, source, mt, ais, ahv, bjq) {
        return; 
    }
    
    match protocol {
        1 => crate::netstack::icmp::alq(payload, ttl, source), 
        6 => crate::netstack::tcp::alq(payload, source, mt), 
        17 => crate::netstack::udp::alq(payload, source),      
        _ => {
            crate::serial_println!("[IP] Unsupported protocol {}", protocol);
        }
    }
}


pub fn onw(dest_ip: [u8; 4], protocol: u8, payload: &[u8], ttl: u8) -> Result<(), &'static str> {
    let (source_ip, subnet, gateway) = crate::network::rd()
        .map(|(ip, mask, fz)| (*ip.as_bytes(), *mask.as_bytes(), fz.map(|g| *g.as_bytes())))
        .unwrap_or_else(ilm);

    
    let (ais, ahv) = fvw(protocol, payload);
    if !crate::netstack::firewall::hyo(protocol, source_ip, dest_ip, ais, ahv, 20 + payload.len()) {
        return Err("Blocked by firewall");
    }

    let gkx = |ip: [u8; 4]| {
        (ip[0] & subnet[0]) == (source_ip[0] & subnet[0]) &&
        (ip[1] & subnet[1]) == (source_ip[1] & subnet[1]) &&
        (ip[2] & subnet[2]) == (source_ip[2] & subnet[2]) &&
        (ip[3] & subnet[3]) == (source_ip[3] & subnet[3])
    };

    let cnf = if gkx(dest_ip) {
        dest_ip
    } else if let Some(fz) = gateway {
        if fz != [0, 0, 0, 0] { fz } else { dest_ip }
    } else {
        dest_ip
    };

    let mut header = Vec::new();
    let bjq = 20 + payload.len();
    header.push(0x45);
    header.push(0);
    header.extend_from_slice(&(bjq as u16).to_be_bytes());
    header.extend_from_slice(&0u16.to_be_bytes());
    header.extend_from_slice(&0x4000u16.to_be_bytes());
    header.push(ttl);
    header.push(protocol);
    header.push(0); header.push(0);
    header.extend_from_slice(&source_ip);
    header.extend_from_slice(&dest_ip);

    let ig = checksum(&header);
    header[10] = (ig >> 8) as u8;
    header[11] = (ig & 0xFF) as u8;

    let mut be = header;
    be.extend_from_slice(payload);

    let frs = if ihu(dest_ip, &source_ip, &subnet) {
        [0xFF; 6]
    } else {
        match crate::netstack::arp::yb(cnf) {
            Some(mac) => mac,
            None => {
                crate::netstack::arp::bos(cnf)?;
                let start = crate::logger::eg();
                let mut my: u32 = 0;
                loop {
                    crate::netstack::poll();
                    if let Some(mac) = crate::netstack::arp::yb(cnf) {
                        break mac;
                    }
                    if crate::logger::eg().saturating_sub(start) > 1000 {
                        return Err("ARP timeout");
                    }
                    my = my.wrapping_add(1);
                    if my > 2_000_000 { return Err("ARP timeout"); }
                    crate::arch::acb();
                }
            }
        }
    };

    crate::netstack::cdq(frs, 0x0800, &be)
}


pub fn aha(dest_ip: [u8; 4], protocol: u8, payload: &[u8]) -> Result<(), &'static str> {
    
    let (source_ip, subnet, gateway) = crate::network::rd()
        .map(|(ip, mask, fz)| (*ip.as_bytes(), *mask.as_bytes(), fz.map(|g| *g.as_bytes())))
        .unwrap_or_else(ilm);

    
    let (ais, ahv) = fvw(protocol, payload);
    if !crate::netstack::firewall::hyo(protocol, source_ip, dest_ip, ais, ahv, 20 + payload.len()) {
        return Err("Blocked by firewall");
    }

    
    let gkx = |ip: [u8; 4]| {
        (ip[0] & subnet[0]) == (source_ip[0] & subnet[0]) &&
        (ip[1] & subnet[1]) == (source_ip[1] & subnet[1]) &&
        (ip[2] & subnet[2]) == (source_ip[2] & subnet[2]) &&
        (ip[3] & subnet[3]) == (source_ip[3] & subnet[3])
    };

    let cnf = if gkx(dest_ip) {
        dest_ip
    } else if let Some(fz) = gateway {
        
        if fz != [0, 0, 0, 0] {
            fz
        } else {
            dest_ip 
        }
    } else {
        dest_ip
    };
    
    
    let mut header = Vec::new();
    let bjq = 20 + payload.len();
    
    header.push(0x45); 
    header.push(0);    
    header.extend_from_slice(&(bjq as u16).to_be_bytes()); 
    header.extend_from_slice(&0u16.to_be_bytes()); 
    header.extend_from_slice(&0x4000u16.to_be_bytes()); 
    header.push(64);   
    header.push(protocol); 
    header.push(0); header.push(0); 
    header.extend_from_slice(&source_ip); 
    header.extend_from_slice(&dest_ip);   
    
    
    let ig = checksum(&header);
    header[10] = (ig >> 8) as u8;
    header[11] = (ig & 0xFF) as u8;
    
    
    
    
    let mut be = header;
    be.extend_from_slice(payload);
    
    
    let frs = if ihu(dest_ip, &source_ip, &subnet) {
        [0xFF; 6]
    } else {
        match crate::netstack::arp::yb(cnf) {
            Some(mac) => mac,
            None => {
                crate::netstack::arp::bos(cnf)?;
                let start = crate::logger::eg();
                let mut my: u32 = 0;
                loop {
                    crate::netstack::poll();
                    if let Some(mac) = crate::netstack::arp::yb(cnf) {
                        break mac;
                    }
                    if crate::logger::eg().saturating_sub(start) > 1000 {
                        return Err("ARP timeout");
                    }
                    my = my.wrapping_add(1);
                    if my > 2_000_000 {
                        return Err("ARP timeout");
                    }
                    crate::arch::acb();
                }
            }
        }
    };
    
    
    crate::netstack::cdq(frs, 0x0800, &be)?;
    
    
    
    Ok(())
}


fn fvw(protocol: u8, payload: &[u8]) -> (u16, u16) {
    match protocol {
        6 | 17 => {
            
            if payload.len() >= 4 {
                let ais = u16::from_be_bytes([payload[0], payload[1]]);
                let ahv = u16::from_be_bytes([payload[2], payload[3]]);
                (ais, ahv)
            } else {
                (0, 0)
            }
        }
        _ => (0, 0),
    }
}
