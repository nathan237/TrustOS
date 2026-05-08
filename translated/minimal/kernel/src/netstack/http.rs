



use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;


#[derive(Debug)]
pub struct Ib {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Ib {
    
    pub fn header(&self, name: &str) -> Option<&str> {
        let duw = fdb(name);
        self.headers.iter()
            .find(|(k, _)| fdb(k) == duw)
            .map(|(_, v)| v.as_str())
    }
    
    
    pub fn body_str(&self) -> Option<&str> {
        core::str::from_utf8(&self.body).ok()
    }
}


fn fdb(j: &str) -> String {
    j.chars().map(|c| {
        if c >= 'A' && c <= 'Z' { (c as u8 + 32) as char } else { c }
    }).collect()
}


pub fn get(url: &str) -> Result<Ib, &'static str> {
    request("GET", url, None, None)
}


pub fn nwh(url: &str, content_type: &str, body: &[u8]) -> Result<Ib, &'static str> {
    request("POST", url, Some(content_type), Some(body))
}


fn nrm(url: &str) -> Result<(&str, u16, &str), &'static str> {
    let url = url.strip_prefix("http://").unwrap_or(url);
    
    let (host_port, path) = match url.find('/') {
        Some(i) => (&url[..i], &url[i..]),
        None => (url, "/"),
    };
    
    let (host, port) = match host_port.find(':') {
        Some(i) => (&host_port[..i], host_port[i+1..].parse().unwrap_or(80)),
        None => (host_port, 80),
    };
    
    if host.is_empty() {
        return Err("Empty host");
    }
    
    Ok((host, port, path))
}


fn request(aui: &str, url: &str, content_type: Option<&str>, body: Option<&[u8]>) -> Result<Ib, &'static str> {
    jaf(aui, url, content_type, body, 0)
}


fn jaf(aui: &str, url: &str, content_type: Option<&str>, body: Option<&[u8]>, depth: u32) -> Result<Ib, &'static str> {
    if depth > 5 {
        return Err("Too many redirects");
    }
    
    let (host, port, path) = nrm(url)?;
    
    
    let ip = if let Ok(evn) = bof(host) {
        evn
    } else {
        crate::netstack::dns::yb(host).ok_or("DNS resolution failed")?
    };
    
    crate::serial_println!("[HTTP] {} {}.{}.{}.{}:{}{}", aui, ip[0], ip[1], ip[2], ip[3], port, path);
    
    
    let src_port = crate::netstack::tcp::azp(ip, port)?;
    
    if !crate::netstack::tcp::bjy(ip, port, src_port, 5000) {
        return Err("TCP connection timeout");
    }
    
    
    let mut bvk = format!("{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: TrustOS/1.0\r\n", aui, path, host);
    
    if let Some(wb) = content_type {
        bvk.push_str(&format!("Content-Type: {}\r\n", wb));
    }
    
    if let Some(b) = body {
        bvk.push_str(&format!("Content-Length: {}\r\n", b.len()));
    }
    
    bvk.push_str("\r\n");
    
    
    crate::netstack::tcp::bjc(ip, port, src_port, bvk.as_bytes())?;
    
    if let Some(b) = body {
        crate::netstack::tcp::bjc(ip, port, src_port, b)?;
    }
    
    
    let mut bon = Vec::new();
    let start = crate::logger::eg();
    
    loop {
        crate::netstack::poll();
        
        while let Some(df) = crate::netstack::tcp::aus(ip, port, src_port) {
            bon.extend_from_slice(&df);
        }
        
        
        if bon.len() > 12 && kxj(&bon) {
            break;
        }
        
        
        if crate::netstack::tcp::fin_received(ip, port, src_port) {
            break;
        }
        
        
        if crate::logger::eg().saturating_sub(start) > 5000 {
            break;
        }
        
        
        crate::thread::ajc();
    }
    
    
    let _ = crate::netstack::tcp::ams(ip, port, src_port);
    
    
    let fa = nra(&bon)?;
    
    
    if fa.status_code >= 300 && fa.status_code < 400 {
        if let Some(axx) = fa.header("Location") {
            
            
            if axx.starts_with("https://") {
                crate::serial_println!("[HTTP] Redirect to HTTPS: {} -> {}", fa.status_code, axx);
                return Ok(fa);
            }
            
            let cpe = if axx.starts_with("http://") {
                String::from(axx)
            } else if axx.starts_with("/") {
                format!("http://{}:{}{}", host, port, axx)
            } else {
                
                let dij = match path.rfind('/') {
                    Some(i) => &path[..=i],
                    None => "/",
                };
                format!("http://{}:{}{}{}", host, port, dij, axx)
            };
            crate::serial_println!("[HTTP] Redirect {} -> {}", fa.status_code, cpe);
            return jaf(aui, &cpe, content_type, body, depth + 1);
        }
    }
    
    Ok(fa)
}


fn kxj(data: &[u8]) -> bool {
    
    let bca = match hyw(data) {
        Some(v) => v,
        None => return false,
    };
    
    
    if let Some(cl) = lvt(data, bca) {
        let kdb = data.len() - bca;
        return kdb >= cl;
    }
    
    
    if erh(data, bca) {
        
        let body = &data[bca..];
        for i in 0..body.len().saturating_sub(4) {
            if body[i] == b'0' && body[i+1] == b'\r' && body[i+2] == b'\n' && body[i+3] == b'\r' && body.len() > i + 4 && body[i+4] == b'\n' {
                return true;
            }
        }
        return false;
    }
    
    
    false
}

fn hyw(data: &[u8]) -> Option<usize> {
    for i in 0..data.len().saturating_sub(3) {
        if &data[i..i+4] == b"\r\n\r\n" {
            return Some(i + 4);
        }
    }
    None
}

fn lvt(data: &[u8], bca: usize) -> Option<usize> {
    let headers = core::str::from_utf8(&data[..bca]).ok()?;
    for line in headers.lines() {
        let gj = fdb(line);
        if gj.starts_with("content-length:") {
            let val = line[15..].trim();
            return val.parse().ok();
        }
    }
    None
}


fn fra(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut pos = 0;
    
    while pos < data.len() {
        
        let mut ari = pos;
        while ari + 1 < data.len() {
            if data[ari] == b'\r' && data[ari + 1] == b'\n' {
                break;
            }
            ari += 1;
        }
        
        if ari + 1 >= data.len() {
            break;
        }
        
        
        let td = core::str::from_utf8(&data[pos..ari]).unwrap_or("0");
        let td = td.split(';').next().unwrap_or("0").trim();
        let rs = usize::from_str_radix(td, 16).unwrap_or(0);
        
        if rs == 0 {
            break; 
        }
        
        let cgz = ari + 2; 
        let cgy = cgz + rs;
        
        if cgy > data.len() {
            
            result.extend_from_slice(&data[cgz..]);
            break;
        }
        
        result.extend_from_slice(&data[cgz..cgy]);
        pos = cgy + 2; 
    }
    
    result
}


fn erh(data: &[u8], bca: usize) -> bool {
    if let Ok(headers) = core::str::from_utf8(&data[..bca]) {
        for line in headers.lines() {
            let gj = fdb(line);
            if gj.starts_with("transfer-encoding:") && gj.contains("chunked") {
                return true;
            }
        }
    }
    false
}


fn nra(data: &[u8]) -> Result<Ib, &'static str> {
    let bca = hyw(data).ok_or("Incomplete response")?;
    
    let mkr = core::str::from_utf8(&data[..bca])
        .map_err(|_| "Invalid UTF-8 in headers")?;
    
    let mut lines = mkr.lines();
    let ahd = lines.next().ok_or("No status line")?;
    
    
    let au: Vec<&str> = ahd.splitn(3, ' ').collect();
    if au.len() < 2 {
        return Err("Invalid status line");
    }
    
    let status_code: u16 = au[1].parse().map_err(|_| "Invalid status code")?;
    
    
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some(ald) = line.find(':') {
            let key = String::from(line[..ald].trim());
            let value = String::from(line[ald+1..].trim());
            headers.push((key, value));
        }
    }
    
    
    let cou = &data[bca..];
    let body = if erh(data, bca) {
        crate::serial_println!("[HTTP] Decoding chunked transfer encoding ({} raw bytes)", cou.len());
        fra(cou)
    } else {
        cou.to_vec()
    };
    
    Ok(Ib {
        status_code,
        headers,
        body,
    })
}


fn bof(j: &str) -> Result<[u8; 4], ()> {
    let au: Vec<&str> = j.split('.').collect();
    if au.len() != 4 {
        return Err(());
    }
    
    let a: u8 = au[0].parse().map_err(|_| ())?;
    let b: u8 = au[1].parse().map_err(|_| ())?;
    let c: u8 = au[2].parse().map_err(|_| ())?;
    let d: u8 = au[3].parse().map_err(|_| ())?;
    
    Ok([a, b, c, d])
}
