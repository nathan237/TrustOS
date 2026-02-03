//! HTTP Client
//!
//! Simple HTTP/1.1 client for GET/POST requests.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;

/// HTTP Response
#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Get header value by name (case-insensitive)
    pub fn header(&self, name: &str) -> Option<&str> {
        let name_lower = to_lower(name);
        self.headers.iter()
            .find(|(k, _)| to_lower(k) == name_lower)
            .map(|(_, v)| v.as_str())
    }
    
    /// Get body as string (if valid UTF-8)
    pub fn body_str(&self) -> Option<&str> {
        core::str::from_utf8(&self.body).ok()
    }
}

/// Convert to lowercase
fn to_lower(s: &str) -> String {
    s.chars().map(|c| {
        if c >= 'A' && c <= 'Z' { (c as u8 + 32) as char } else { c }
    }).collect()
}

/// Perform HTTP GET request
pub fn get(url: &str) -> Result<HttpResponse, &'static str> {
    request("GET", url, None, None)
}

/// Perform HTTP POST request
pub fn post(url: &str, content_type: &str, body: &[u8]) -> Result<HttpResponse, &'static str> {
    request("POST", url, Some(content_type), Some(body))
}

/// Parse URL into (host, port, path)
fn parse_url(url: &str) -> Result<(&str, u16, &str), &'static str> {
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

/// Perform HTTP request
fn request(method: &str, url: &str, content_type: Option<&str>, body: Option<&[u8]>) -> Result<HttpResponse, &'static str> {
    let (host, port, path) = parse_url(url)?;
    
    // Resolve hostname to IP
    let ip = if let Ok(octets) = parse_ip(host) {
        octets
    } else {
        crate::netstack::dns::resolve(host).ok_or("DNS resolution failed")?
    };
    
    crate::serial_println!("[HTTP] {} {}.{}.{}.{}:{}{}", method, ip[0], ip[1], ip[2], ip[3], port, path);
    
    // Connect via TCP
    let src_port = crate::netstack::tcp::send_syn(ip, port)?;
    
    if !crate::netstack::tcp::wait_for_established(ip, port, src_port, 5000) {
        return Err("TCP connection timeout");
    }
    
    // Build HTTP request
    let mut req = format!("{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: TrustOS/1.0\r\n", method, path, host);
    
    if let Some(ct) = content_type {
        req.push_str(&format!("Content-Type: {}\r\n", ct));
    }
    
    if let Some(b) = body {
        req.push_str(&format!("Content-Length: {}\r\n", b.len()));
    }
    
    req.push_str("\r\n");
    
    // Send request
    crate::netstack::tcp::send_payload(ip, port, src_port, req.as_bytes())?;
    
    if let Some(b) = body {
        crate::netstack::tcp::send_payload(ip, port, src_port, b)?;
    }
    
    // Receive response
    let mut response_data = Vec::new();
    let start = crate::logger::get_ticks();
    
    loop {
        crate::netstack::poll();
        
        while let Some(chunk) = crate::netstack::tcp::recv_data(ip, port, src_port) {
            response_data.extend_from_slice(&chunk);
        }
        
        // Check if we got complete response
        if response_data.len() > 12 && contains_end_marker(&response_data) {
            break;
        }
        
        // Check for FIN (connection closed by server)
        if crate::netstack::tcp::fin_received(ip, port, src_port) {
            break;
        }
        
        // Timeout after 10 seconds
        if crate::logger::get_ticks().saturating_sub(start) > 10000 {
            break;
        }
        
        core::hint::spin_loop();
    }
    
    // Send FIN to close connection
    let _ = crate::netstack::tcp::send_fin(ip, port, src_port);
    
    // Parse response
    parse_response(&response_data)
}

/// Check if response contains end marker (for chunked or content-length based)
fn contains_end_marker(data: &[u8]) -> bool {
    // Find header end
    let header_end = find_header_end(data);
    if header_end.is_none() {
        return false;
    }
    let header_end = header_end.unwrap();
    
    // Check Content-Length
    if let Some(cl) = find_content_length(data, header_end) {
        let body_len = data.len() - header_end;
        return body_len >= cl;
    }
    
    // For Connection: close, we wait for FIN
    false
}

fn find_header_end(data: &[u8]) -> Option<usize> {
    for i in 0..data.len().saturating_sub(3) {
        if &data[i..i+4] == b"\r\n\r\n" {
            return Some(i + 4);
        }
    }
    None
}

fn find_content_length(data: &[u8], header_end: usize) -> Option<usize> {
    let headers = core::str::from_utf8(&data[..header_end]).ok()?;
    for line in headers.lines() {
        let lower = to_lower(line);
        if lower.starts_with("content-length:") {
            let val = line[15..].trim();
            return val.parse().ok();
        }
    }
    None
}

/// Parse HTTP response
fn parse_response(data: &[u8]) -> Result<HttpResponse, &'static str> {
    let header_end = find_header_end(data).ok_or("Incomplete response")?;
    
    let header_str = core::str::from_utf8(&data[..header_end])
        .map_err(|_| "Invalid UTF-8 in headers")?;
    
    let mut lines = header_str.lines();
    let status_line = lines.next().ok_or("No status line")?;
    
    // Parse "HTTP/1.1 200 OK"
    let parts: Vec<&str> = status_line.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return Err("Invalid status line");
    }
    
    let status_code: u16 = parts[1].parse().map_err(|_| "Invalid status code")?;
    
    // Parse headers
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some(colon) = line.find(':') {
            let key = String::from(line[..colon].trim());
            let value = String::from(line[colon+1..].trim());
            headers.push((key, value));
        }
    }
    
    // Body
    let body = data[header_end..].to_vec();
    
    Ok(HttpResponse {
        status_code,
        headers,
        body,
    })
}

/// Parse IP address string
fn parse_ip(s: &str) -> Result<[u8; 4], ()> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return Err(());
    }
    
    let a: u8 = parts[0].parse().map_err(|_| ())?;
    let b: u8 = parts[1].parse().map_err(|_| ())?;
    let c: u8 = parts[2].parse().map_err(|_| ())?;
    let d: u8 = parts[3].parse().map_err(|_| ())?;
    
    Ok([a, b, c, d])
}
