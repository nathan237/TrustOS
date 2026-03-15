

use alloc::vec::Vec;




fn okq() -> ([u8; 4], [u8; 4], Option<[u8; 4]>) {
    let ed = crate::drivers::net::cez()
        .unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    
    let kh = if ed[5] == 0 { 1 } else if ed[5] == 255 { 254 } else { ed[5] };
    ([10, 0, 100, kh], [255, 255, 255, 0], None)
}



fn oft(aac: [u8; 4], cy: &[u8; 4], hs: &[u8; 4]) -> bool {
    
    if aac == [255, 255, 255, 255] {
        return true;
    }
    
    (aac[0] & !hs[0]) == !hs[0] &&
    (aac[1] & !hs[1]) == !hs[1] &&
    (aac[2] & !hs[2]) == !hs[2] &&
    (aac[3] & !hs[3]) == !hs[3]
}


#[repr(C, packed)]
struct Dad {
    zvh: u8,      
    ynu: u8,         
    dmo: u16,
    yxj: u16,
    yqx: u16,  
    akv: u8,
    protocol: u8,
    bmj: u16,
    iy: [u8; 4],
    aac: [u8; 4],
}


fn bmj(f: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut a = 0;
    
    while a < f.len() - 1 {
        sum += ((f[a] as u32) << 8) | (f[a + 1] as u32);
        a += 2;
    }
    
    if a < f.len() {
        sum += (f[a] as u32) << 8;
    }
    
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    
    !sum as u16
}


pub fn bur(f: &[u8]) {
    if f.len() < 20 {
        return;
    }
    
    let dk = f[0] >> 4;
    let ldh = (f[0] & 0x0F) as usize;
    let ect = ldh * 4;
    
    if dk != 4 || f.len() < ect {
        return;
    }
    
    
    let dmo = u16::oa([f[2], f[3]]) as usize;
    
    
    if dmo < ect || dmo > f.len() {
        return;
    }
    
    let akv = f[8];
    let protocol = f[9];
    let iy = [f[12], f[13], f[14], f[15]];
    let aac = [f[16], f[17], f[18], f[19]];
    
    
    let ew = &f[ect..dmo];
    
    
    let (bom, bmx) = kus(protocol, ew);
    if !crate::netstack::firewall::ssp(protocol, iy, aac, bom, bmx, dmo) {
        return; 
    }
    
    match protocol {
        1 => crate::netstack::icmp::bur(ew, akv, iy), 
        6 => crate::netstack::tcp::bur(ew, iy, aac), 
        17 => crate::netstack::udp::bur(ew, iy),      
        _ => {
            crate::serial_println!("[IP] Unsupported protocol {}", protocol);
        }
    }
}


pub fn whj(kv: [u8; 4], protocol: u8, ew: &[u8], akv: u8) -> Result<(), &'static str> {
    let (bct, up, auj) = crate::network::aou()
        .map(|(ip, hs, nt)| (*ip.as_bytes(), *hs.as_bytes(), nt.map(|at| *at.as_bytes())))
        .unwrap_or_else(okq);

    
    let (bom, bmx) = kus(protocol, ew);
    if !crate::netstack::firewall::ntw(protocol, bct, kv, bom, bmx, 20 + ew.len()) {
        return Err("Blocked by firewall");
    }

    let lql = |ip: [u8; 4]| {
        (ip[0] & up[0]) == (bct[0] & up[0]) &&
        (ip[1] & up[1]) == (bct[1] & up[1]) &&
        (ip[2] & up[2]) == (bct[2] & up[2]) &&
        (ip[3] & up[3]) == (bct[3] & up[3])
    };

    let foy = if lql(kv) {
        kv
    } else if let Some(nt) = auj {
        if nt != [0, 0, 0, 0] { nt } else { kv }
    } else {
        kv
    };

    let mut dh = Vec::new();
    let dmo = 20 + ew.len();
    dh.push(0x45);
    dh.push(0);
    dh.bk(&(dmo as u16).ft());
    dh.bk(&0u16.ft());
    dh.bk(&0x4000u16.ft());
    dh.push(akv);
    dh.push(protocol);
    dh.push(0); dh.push(0);
    dh.bk(&bct);
    dh.bk(&kv);

    let td = bmj(&dh);
    dh[10] = (td >> 8) as u8;
    dh[11] = (td & 0xFF) as u8;

    let mut ex = dh;
    ex.bk(ew);

    let kpk = if oft(kv, &bct, &up) {
        [0xFF; 6]
    } else {
        match crate::netstack::arp::ayo(foy) {
            Some(ed) => ed,
            None => {
                crate::netstack::arp::eii(foy)?;
                let ay = crate::logger::lh();
                let mut aaf: u32 = 0;
                loop {
                    crate::netstack::poll();
                    if let Some(ed) = crate::netstack::arp::ayo(foy) {
                        break ed;
                    }
                    if crate::logger::lh().ao(ay) > 1000 {
                        return Err("ARP timeout");
                    }
                    aaf = aaf.cn(1);
                    if aaf > 2_000_000 { return Err("ARP timeout"); }
                    crate::arch::bhd();
                }
            }
        }
    };

    crate::netstack::fug(kpk, 0x0800, &ex)
}


pub fn blc(kv: [u8; 4], protocol: u8, ew: &[u8]) -> Result<(), &'static str> {
    
    let (bct, up, auj) = crate::network::aou()
        .map(|(ip, hs, nt)| (*ip.as_bytes(), *hs.as_bytes(), nt.map(|at| *at.as_bytes())))
        .unwrap_or_else(okq);

    
    let (bom, bmx) = kus(protocol, ew);
    if !crate::netstack::firewall::ntw(protocol, bct, kv, bom, bmx, 20 + ew.len()) {
        return Err("Blocked by firewall");
    }

    
    let lql = |ip: [u8; 4]| {
        (ip[0] & up[0]) == (bct[0] & up[0]) &&
        (ip[1] & up[1]) == (bct[1] & up[1]) &&
        (ip[2] & up[2]) == (bct[2] & up[2]) &&
        (ip[3] & up[3]) == (bct[3] & up[3])
    };

    let foy = if lql(kv) {
        kv
    } else if let Some(nt) = auj {
        
        if nt != [0, 0, 0, 0] {
            nt
        } else {
            kv 
        }
    } else {
        kv
    };
    
    
    let mut dh = Vec::new();
    let dmo = 20 + ew.len();
    
    dh.push(0x45); 
    dh.push(0);    
    dh.bk(&(dmo as u16).ft()); 
    dh.bk(&0u16.ft()); 
    dh.bk(&0x4000u16.ft()); 
    dh.push(64);   
    dh.push(protocol); 
    dh.push(0); dh.push(0); 
    dh.bk(&bct); 
    dh.bk(&kv);   
    
    
    let td = bmj(&dh);
    dh[10] = (td >> 8) as u8;
    dh[11] = (td & 0xFF) as u8;
    
    
    
    
    let mut ex = dh;
    ex.bk(ew);
    
    
    let kpk = if oft(kv, &bct, &up) {
        [0xFF; 6]
    } else {
        match crate::netstack::arp::ayo(foy) {
            Some(ed) => ed,
            None => {
                crate::netstack::arp::eii(foy)?;
                let ay = crate::logger::lh();
                let mut aaf: u32 = 0;
                loop {
                    crate::netstack::poll();
                    if let Some(ed) = crate::netstack::arp::ayo(foy) {
                        break ed;
                    }
                    if crate::logger::lh().ao(ay) > 1000 {
                        return Err("ARP timeout");
                    }
                    aaf = aaf.cn(1);
                    if aaf > 2_000_000 {
                        return Err("ARP timeout");
                    }
                    crate::arch::bhd();
                }
            }
        }
    };
    
    
    crate::netstack::fug(kpk, 0x0800, &ex)?;
    
    
    
    Ok(())
}


fn kus(protocol: u8, ew: &[u8]) -> (u16, u16) {
    match protocol {
        6 | 17 => {
            
            if ew.len() >= 4 {
                let bom = u16::oa([ew[0], ew[1]]);
                let bmx = u16::oa([ew[2], ew[3]]);
                (bom, bmx)
            } else {
                (0, 0)
            }
        }
        _ => (0, 0),
    }
}
