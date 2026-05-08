



use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use crate::tls13::{TlsSession, TlsError, hsy};
use crate::netstack::tcp;


fn lqj() {
    
    if !crate::netstack::dhcp::clk() {
        crate::serial_println!("[HTTPS] Waiting for network (DHCP)...");
        crate::netstack::dhcp::start();
        
        
        let start = crate::logger::eg();
        while !crate::netstack::dhcp::clk() {
            crate::netstack::poll();
            
            if crate::logger::eg().saturating_sub(start) > 5000 {
                crate::serial_println!("[HTTPS] DHCP timeout, continuing anyway");
                break;
            }
            
            
            crate::thread::ajc();
        }
        
        if crate::netstack::dhcp::clk() {
            crate::serial_println!("[HTTPS] Network ready");
        }
    }
}


pub struct Jz {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Jz {
    pub fn body_string(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}


#[derive(Debug)]
pub enum HttpsError {
    DnsError,
    ConnectionFailed,
    TlsError(TlsError),
    Timeout,
    InvalidResponse,
}

impl fmt::Display for HttpsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpsError::DnsError => write!(f, "DNS resolution failed"),
            HttpsError::ConnectionFailed => write!(f, "Connection failed"),
            HttpsError::TlsError(e) => write!(f, "TLS error: {:?}", e),
            HttpsError::Timeout => write!(f, "Connection timeout"),
            HttpsError::InvalidResponse => write!(f, "Invalid HTTP response"),
        }
    }
}

impl From<TlsError> for HttpsError {
    fn from(e: TlsError) -> Self {
        HttpsError::TlsError(e)
    }
}


pub fn get(url: &str) -> Result<Jz, HttpsError> {
    ibn(url, 0)
}


fn ibn(url: &str, depth: u32) -> Result<Jz, HttpsError> {
    if depth > 5 {
        return Err(HttpsError::InvalidResponse);
    }
    
    
    let (host, path, port) = nqn(url)?;
    
    crate::serial_println!("[HTTPS] GET https://{}:{}{}", host, port, path);
    
    
    lqj();
    
    
    let ip = crate::netstack::dns::yb(&host)
        .ok_or(HttpsError::DnsError)?;
    
    crate::serial_println!("[HTTPS] Resolved {} -> {}.{}.{}.{}", host, ip[0], ip[1], ip[2], ip[3]);
    
    
    let src_port = tcp::azp(ip, port)
        .map_err(|_| HttpsError::ConnectionFailed)?;
    
    
    if !tcp::bjy(ip, port, src_port, 5000) {
        return Err(HttpsError::ConnectionFailed);
    }
    
    crate::serial_println!("[HTTPS] TCP connected, starting TLS handshake");
    
    
    let mut by = TlsSession::new(&host);
    
    
    {
        let mut send = |data: &[u8]| -> Result<(), TlsError> {
            tcp::bjc(ip, port, src_port, data)
                .map_err(|_| TlsError::ConnectionFailed)
        };
        
        let mut gqs = 0u32;
        let mut recv = |buf: &mut [u8]| -> Result<usize, TlsError> {
            
            for _ in 0..100 {
                crate::netstack::poll();
                
                if let Some(data) = tcp::aus(ip, port, src_port) {
                    let len = data.len().min(buf.len());
                    buf[..len].copy_from_slice(&data[..len]);
                    gqs = 0;
                    return Ok(len);
                }
                
                
                crate::thread::ajc();
            }
            
            gqs += 1;
            if gqs > 50 {
                
                crate::serial_println!("[TLS] Too many recv attempts, giving up");
                return Err(TlsError::ConnectionClosed);
            }
            
            Err(TlsError::WouldBlock)
        };
        
        hsy(&mut by, &mut send, &mut recv)?;
    }
    
    crate::serial_println!("[HTTPS] TLS handshake complete");
    
    
    let request = alloc::format!(
        "GET {} HTTP/1.1\r\n\
         Host: {}\r\n\
         User-Agent: TrustOS/1.0\r\n\
         Accept: */*\r\n\
         Connection: close\r\n\
         \r\n",
        path, host
    );
    
    
    let fur = by.encrypt(request.as_bytes())?;
    tcp::bjc(ip, port, src_port, &fur)
        .map_err(|_| HttpsError::ConnectionFailed)?;
    
    crate::serial_println!("[HTTPS] Request sent, waiting for response");
    
    
    let mut bon = Vec::new();
    
    for _ in 0..200 {
        crate::netstack::poll();
        
        if let Some(data) = tcp::aus(ip, port, src_port) {
            
            if let Some(ry) = nym(&mut by, &data) {
                bon.extend_from_slice(&ry);
            }
        }
        
        
        if bon.len() > 12 {
            
            if bon.windows(4).any(|w| w == b"\r\n\r\n") {
                
                if mto(&bon) {
                    break;
                }
            }
        }
        
        
        if tcp::fin_received(ip, port, src_port) {
            break;
        }
        
        crate::thread::ajc();
    }
    
    
    let _ = tcp::ams(ip, port, src_port);
    
    
    let fa = nqm(&bon)?;
    
    
    if fa.status_code >= 300 && fa.status_code < 400 {
        
        for (name, value) in &fa.headers {
            if name.to_lowercase() == "location" {
                let cpe = if value.starts_with("http://") || value.starts_with("https://") {
                    value.clone()
                } else if value.starts_with("/") {
                    alloc::format!("https://{}:{}{}", host, port, value)
                } else {
                    let dij = match path.rfind('/') {
                        Some(i) => &path[..=i],
                        None => "/",
                    };
                    alloc::format!("https://{}:{}{}{}", host, port, dij, value)
                };
                crate::serial_println!("[HTTPS] Redirect {} -> {}", fa.status_code, cpe);
                
                if cpe.starts_with("http://") {
                    return crate::netstack::http::get(&cpe)
                        .map(|r| Jz {
                            status_code: r.status_code,
                            headers: r.headers,
                            body: r.body,
                        })
                        .map_err(|_| HttpsError::ConnectionFailed);
                }
                return ibn(&cpe, depth + 1);
            }
        }
    }
    
    Ok(fa)
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


fn nqn(url: &str) -> Result<(String, String, u16), HttpsError> {
    let url = url.strip_prefix("https://").unwrap_or(url);
    
    
    let (host_port, path) = if let Some(slash_pos) = url.find('/') {
        (&url[..slash_pos], &url[slash_pos..])
    } else {
        (url, "/")
    };
    
    
    let (host, port) = if let Some(bfk) = host_port.rfind(':') {
        let bva = &host_port[bfk + 1..];
        let port = bva.parse().unwrap_or(443);
        (&host_port[..bfk], port)
    } else {
        (host_port, 443u16)
    };
    
    Ok((String::from(host), String::from(path), port))
}


fn nym(by: &mut TlsSession, data: &[u8]) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let mut pos = 0;
    
    while pos + 5 <= data.len() {
        let length = u16::from_be_bytes([data[pos + 3], data[pos + 4]]) as usize;
        
        if pos + 5 + length > data.len() {
            break;
        }
        
        if let Ok(Some(ry)) = by.process_record(&data[pos..pos + 5 + length]) {
            
            if let Some((&content_type, content)) = ry.split_last() {
                if content_type == 23 || content_type == 0 {
                    
                    result.extend_from_slice(content);
                }
            } else {
                result.extend_from_slice(&ry);
            }
        }
        
        pos += 5 + length;
    }
    
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}


fn mto(data: &[u8]) -> bool {
    let brp = String::from_utf8_lossy(data);
    
    
    if let Some(dri) = brp.find("\r\n\r\n") {
        let headers = &brp[..dri];
        
        
        for line in headers.lines() {
            if line.to_lowercase().starts_with("content-length:") {
                if let Some(len_str) = line.split(':').nth(1) {
                    if let Ok(anw) = len_str.trim().parse::<usize>() {
                        let bao = dri + 4;
                        return data.len() >= bao + anw;
                    }
                }
            }
        }
        
        
        if headers.to_lowercase().contains("transfer-encoding: chunked") {
            
            return brp.contains("0\r\n\r\n") || brp.ends_with("0\r\n");
        }
        
        
        return true;
    }
    
    false
}


fn nqm(data: &[u8]) -> Result<Jz, HttpsError> {
    let brp = String::from_utf8_lossy(data);
    
    
    let jip = brp.find("\r\n").ok_or(HttpsError::InvalidResponse)?;
    let ahd = &brp[..jip];
    
    
    let au: Vec<&str> = ahd.split_whitespace().collect();
    if au.len() < 2 {
        return Err(HttpsError::InvalidResponse);
    }
    
    let status_code: u16 = au[1].parse().map_err(|_| HttpsError::InvalidResponse)?;
    
    
    let dri = brp.find("\r\n\r\n").ok_or(HttpsError::InvalidResponse)?;
    let gam = &brp[jip + 2..dri];
    
    
    let mut headers = Vec::new();
    for line in gam.lines() {
        if let Some(bfk) = line.find(':') {
            let name = line[..bfk].trim().to_string();
            let value = line[bfk + 1..].trim().to_string();
            headers.push((name, value));
        }
    }
    
    
    let bao = dri + 4;
    let cou = if bao < data.len() {
        &data[bao..]
    } else {
        &[] as &[u8]
    };
    
    
    let erh = gam.to_lowercase().contains("transfer-encoding") 
        && gam.to_lowercase().contains("chunked");
    
    let body = if erh {
        crate::serial_println!("[HTTPS] Decoding chunked transfer encoding ({} raw bytes)", cou.len());
        fra(cou)
    } else {
        cou.to_vec()
    };
    
    Ok(Jz {
        status_code,
        headers,
        body,
    })
}
