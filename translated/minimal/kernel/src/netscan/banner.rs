




use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;


#[derive(Debug, Clone)]
pub struct Od {
    pub port: u16,
    pub service: &'static str,
    pub banner: String,
    pub version: Option<String>,
}


pub fn grab_banner(target: [u8; 4], port: u16, timeout_ms: u32) -> Option<Od> {
    let service = super::cqk(port);

    
    let src_port = crate::netstack::tcp::azp(target, port).ok()?;
    if !crate::netstack::tcp::bjy(target, port, src_port, timeout_ms) {
        return None;
    }

    
    let probe = mdq(port);
    if !probe.is_empty() {
        let _ = crate::netstack::tcp::bjc(target, port, src_port, &probe);
    }

    
    let mut efz = Vec::new();
    let start = crate::logger::eg();
    let mut my: u32 = 0;

    loop {
        crate::netstack::poll();

        if let Some(data) = crate::netstack::tcp::aus(target, port, src_port) {
            efz.extend_from_slice(&data);
            
            if efz.len() > 4 {
                break;
            }
        }

        if crate::logger::eg().saturating_sub(start) > timeout_ms as u64 {
            break;
        }
        my = my.wrapping_add(1);
        if my > 500_000 { break; }
        core::hint::spin_loop();
    }

    
    let _ = crate::netstack::tcp::ams(target, port, src_port);

    if efz.is_empty() {
        return None;
    }

    
    let hgo = okc(&efz);
    let version = ltw(&hgo, port);

    Some(Od {
        port,
        service,
        banner: hgo,
        version,
    })
}


pub fn icp(target: [u8; 4], ports: &[u16], timeout_ms: u32) -> Vec<Od> {
    let mut results = Vec::new();
    for &port in ports {
        if let Some(result) = grab_banner(target, port, timeout_ms) {
            results.push(result);
        }
    }
    results
}


fn mdq(port: u16) -> Vec<u8> {
    match port {
        
        80 | 8080 | 8000 | 8008 | 8443 | 443 => {
            b"GET / HTTP/1.0\r\nHost: target\r\nUser-Agent: TrustScan/1.0\r\n\r\n".to_vec()
        }
        
        21 => Vec::new(),
        
        22 => Vec::new(),
        
        25 | 587 | 465 => Vec::new(),
        
        110 | 995 => Vec::new(),
        
        143 | 993 => Vec::new(),
        
        3306 => Vec::new(),
        
        5432 => Vec::new(),
        
        6379 => b"INFO server\r\n".to_vec(),
        
        554 => b"OPTIONS * RTSP/1.0\r\nCSeq: 1\r\n\r\n".to_vec(),
        
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
        
        _ => b"\r\n".to_vec(),
    }
}


fn okc(data: &[u8]) -> String {
    let mut j = String::new();
    let aoo = data.len().min(256); 

    for &b in &data[..aoo] {
        match b {
            0x20..=0x7E => j.push(b as char),
            b'\r' => {} 
            b'\n' => {
                if !j.ends_with(' ') && !j.is_empty() {
                    j.push(' ');
                }
            }
            _ => {
                if !j.ends_with('.') {
                    j.push('.');
                }
            }
        }
    }

    j.trim().into()
}


fn ltw(banner: &str, port: u16) -> Option<String> {
    let cgb = banner.to_ascii_lowercase();

    
    if port == 22 || cgb.starts_with("ssh-") {
        if let Some(version) = banner.split_whitespace().next() {
            return Some(version.into());
        }
    }

    
    if cgb.contains("server:") {
        for line in banner.split(' ') {
            let l = line.trim();
            if l.starts_with("Server:") || l.starts_with("server:") {
                return Some(l[7..].trim().into());
            }
        }
    }

    
    if cgb.contains("apache") {
        return Some("Apache".into());
    }
    if cgb.contains("nginx") {
        return Some("nginx".into());
    }

    
    if cgb.contains("ftp") && banner.starts_with("220") {
        return Some(banner.trim_start_matches("220").trim().into());
    }

    
    if banner.starts_with("220") && cgb.contains("smtp") {
        return Some(banner.trim_start_matches("220").trim().into());
    }

    
    if port == 3306 && banner.len() > 5 {
        return Some(format!("MySQL ({})", &banner[..banner.len().min(30)]));
    }

    
    if cgb.contains("redis_version:") {
        for jn in banner.split(' ') {
            if jn.starts_with("redis_version:") {
                return Some(jn.into());
            }
        }
    }

    None
}
