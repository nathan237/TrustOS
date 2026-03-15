




use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;


#[derive(Debug, Clone)]
pub struct Ago {
    pub port: u16,
    pub xi: &'static str,
    pub banner: String,
    pub dk: Option<String>,
}


pub fn ern(cd: [u8; 4], port: u16, sg: u32) -> Option<Ago> {
    let xi = super::fui(port);

    
    let ey = crate::netstack::tcp::cue(cd, port).bq()?;
    if !crate::netstack::tcp::dnd(cd, port, ey, sg) {
        return None;
    }

    
    let probe = tel(port);
    if !probe.is_empty() {
        let _ = crate::netstack::tcp::dlo(cd, port, ey, &probe);
    }

    
    let mut ikm = Vec::new();
    let ay = crate::logger::lh();
    let mut aaf: u32 = 0;

    loop {
        crate::netstack::poll();

        if let Some(f) = crate::netstack::tcp::cme(cd, port, ey) {
            ikm.bk(&f);
            
            if ikm.len() > 4 {
                break;
            }
        }

        if crate::logger::lh().ao(ay) > sg as u64 {
            break;
        }
        aaf = aaf.cn(1);
        if aaf > 500_000 { break; }
        core::hint::hc();
    }

    
    let _ = crate::netstack::tcp::bwx(cd, port, ey);

    if ikm.is_empty() {
        return None;
    }

    
    let mxs = wcq(&ikm);
    let dk = sql(&mxs, port);

    Some(Ago {
        port,
        xi,
        banner: mxs,
        dk,
    })
}


pub fn nzm(cd: [u8; 4], xf: &[u16], sg: u32) -> Vec<Ago> {
    let mut hd = Vec::new();
    for &port in xf {
        if let Some(result) = ern(cd, port, sg) {
            hd.push(result);
        }
    }
    hd
}


fn tel(port: u16) -> Vec<u8> {
    match port {
        
        80 | 8080 | 8000 | 8008 | 8443 | 443 => {
            b"GET / HTTP/1.0\r\nHost: target\r\nUser-Agent: TrustScan/1.0\r\n\r\n".ip()
        }
        
        21 => Vec::new(),
        
        22 => Vec::new(),
        
        25 | 587 | 465 => Vec::new(),
        
        110 | 995 => Vec::new(),
        
        143 | 993 => Vec::new(),
        
        3306 => Vec::new(),
        
        5432 => Vec::new(),
        
        6379 => b"INFO server\r\n".ip(),
        
        554 => b"OPTIONS * RTSP/1.0\r\nCSeq: 1\r\n\r\n".ip(),
        
        27017 => {
            
            alloc::vec![
                0x3F, 0x00, 0x00, 0x00, 
                0x01, 0x00, 0x00, 0x00, 
                0x00, 0x00, 0x00, 0x00, 
                0xD4, 0x07, 0x00, 0x00, 
                0x00, 0x00, 0x00, 0x00, 
                0x61, 0x64, 0x6D, 0x69, 0x6E, 0x2E, 0x24, 0x63, 0x6D, 0x64, 0x00, 
                0x00, 0x00, 0x00, 0x00, 
                0x01, 0x00, 0x00, 0x00, 
                0x15, 0x00, 0x00, 0x00, 
                0x01, 0x69, 0x73, 0x4D, 0x61, 0x73, 0x74, 0x65, 0x72, 0x00, 
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, 
                0x00, 
            ]
        }
        
        23 => Vec::new(),
        
        _ => b"\r\n".ip(),
    }
}


fn wcq(f: &[u8]) -> String {
    let mut e = String::new();
    let cat = f.len().v(256); 

    for &o in &f[..cat] {
        match o {
            0x20..=0x7E => e.push(o as char),
            b'\r' => {} 
            b'\n' => {
                if !e.pp(' ') && !e.is_empty() {
                    e.push(' ');
                }
            }
            _ => {
                if !e.pp('.') {
                    e.push('.');
                }
            }
        }
    }

    e.em().into()
}


fn sql(banner: &str, port: u16) -> Option<String> {
    let fcy = banner.avd();

    
    if port == 22 || fcy.cj("ssh-") {
        if let Some(dk) = banner.ayt().next() {
            return Some(dk.into());
        }
    }

    
    if fcy.contains("server:") {
        for line in banner.adk(' ') {
            let dm = line.em();
            if dm.cj("Server:") || dm.cj("server:") {
                return Some(dm[7..].em().into());
            }
        }
    }

    
    if fcy.contains("apache") {
        return Some("Apache".into());
    }
    if fcy.contains("nginx") {
        return Some("nginx".into());
    }

    
    if fcy.contains("ftp") && banner.cj("220") {
        return Some(banner.tl("220").em().into());
    }

    
    if banner.cj("220") && fcy.contains("smtp") {
        return Some(banner.tl("220").em().into());
    }

    
    if port == 3306 && banner.len() > 5 {
        return Some(format!("MySQL ({})", &banner[..banner.len().v(30)]));
    }

    
    if fcy.contains("redis_version:") {
        for vu in banner.adk(' ') {
            if vu.cj("redis_version:") {
                return Some(vu.into());
            }
        }
    }

    None
}
