





use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;


#[derive(Debug, Clone)]
pub struct Gn {
    pub ip: [u8; 4],
    pub mac: Option<[u8; 6]>,
    pub hostname: Option<String>,
    pub ttl: Option<u8>,
    pub rtt_ms: u64,
    pub os_hint: &'static str,
}


fn icz(ttl: u8) -> &'static str {
    match ttl {
        
        t if t <= 32 => "Unknown (low TTL)",
        t if t <= 64 => "Linux/Unix/macOS",
        t if t <= 128 => "Windows",
        _ => "Cisco/Network device",
    }
}





pub fn jxr(subnet_start: [u8; 4], subnet_end: [u8; 4], timeout_ms: u32) -> Vec<Gn> {
    let mut aba = Vec::new();

    let fbq = u32::from_be_bytes(subnet_start);
    let elk = u32::from_be_bytes(subnet_end);

    if elk < fbq || elk - fbq > 1024 {
        return aba; 
    }

    
    for clj in fbq..=elk {
        let ip = clj.to_be_bytes();
        let _ = crate::netstack::arp::bos(ip);
        
        for _ in 0..10000 { core::hint::spin_loop(); }
    }

    
    let start = crate::logger::eg();
    let mut my: u32 = 0;
    loop {
        crate::netstack::poll();

        if crate::logger::eg().saturating_sub(start) > timeout_ms as u64 {
            break;
        }
        my = my.wrapping_add(1);
        if my > 1_000_000 { break; }
        crate::arch::acb();
    }

    
    let entries = crate::netstack::arp::entries();
    for (clj, mac) in entries {
        let ip = clj.to_be_bytes();
        
        if clj >= fbq && clj <= elk {
            aba.push(Gn {
                ip,
                mac: Some(mac),
                hostname: None,
                ttl: None,
                rtt_ms: 0,
                os_hint: "Unknown",
            });
        }
    }

    aba.sort_by_key(|h| u32::from_be_bytes(h.ip));
    aba
}


pub fn fhl(timeout_ms: u32) -> Vec<Gn> {
    let (wj, subnet, _) = match crate::network::rd() {
        Some((ip, mask, fz)) => (*ip.as_bytes(), *mask.as_bytes(), fz),
        None => return Vec::new(),
    };

    
    let nil = [
        wj[0] & subnet[0],
        wj[1] & subnet[1],
        wj[2] & subnet[2],
        (wj[3] & subnet[3]).wrapping_add(1), 
    ];
    let nij = [
        wj[0] | !subnet[0],
        wj[1] | !subnet[1],
        wj[2] | !subnet[2],
        (wj[3] | !subnet[3]).wrapping_sub(1), 
    ];

    jxr(nil, nij, timeout_ms)
}




pub fn nut(fcl: &[[u8; 4]], timeout_ms: u32) -> Vec<Gn> {
    let mut aba = Vec::new();

    crate::netstack::icmp::dkt();

    for (i, &target) in fcl.iter().enumerate() {
        let seq = (i + 1) as u16;
        let start = crate::logger::eg();

        if crate::netstack::icmp::gtw(target, 0x5CA2, seq).is_err() {
            continue;
        }

        match crate::netstack::icmp::hcb(seq, timeout_ms) {
            Some(eo) if eo.success => {
                let rtt_ms = crate::logger::eg().saturating_sub(start);
                aba.push(Gn {
                    ip: target,
                    mac: crate::netstack::arp::yb(target),
                    hostname: None,
                    ttl: Some(eo.ttl),
                    rtt_ms,
                    os_hint: icz(eo.ttl),
                });
            }
            _ => {} 
        }
    }

    aba
}


pub fn nuu(base_ip: [u8; 4], timeout_per_host_ms: u32) -> Vec<Gn> {
    let mut fcl = Vec::new();
    for i in 1..=254u8 {
        fcl.push([base_ip[0], base_ip[1], base_ip[2], i]);
    }
    nut(&fcl, timeout_per_host_ms)
}


pub fn mad(timeout_ms: u32) -> Vec<Gn> {
    
    let mut aba = fhl(timeout_ms);

    
    for host in &mut aba {
        crate::netstack::icmp::dkt();
        let start = crate::logger::eg();
        if crate::netstack::icmp::gtw(host.ip, 0x5CA1, 1).is_ok() {
            if let Some(eo) = crate::netstack::icmp::hcb(1, 500) {
                host.ttl = Some(eo.ttl);
                host.rtt_ms = crate::logger::eg().saturating_sub(start);
                host.os_hint = icz(eo.ttl);
            }
        }
    }

    aba
}
