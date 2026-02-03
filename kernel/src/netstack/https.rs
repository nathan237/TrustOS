//! HTTPS Client
//!
//! High-level HTTPS client using our pure Rust TLS 1.3 implementation.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use crate::tls13::{TlsSession, TlsError, do_handshake};
use crate::netstack::tcp;

/// HTTPS response
pub struct HttpsResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpsResponse {
    pub fn body_string(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

/// HTTPS Error
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

/// Perform an HTTPS GET request
pub fn get(url: &str) -> Result<HttpsResponse, HttpsError> {
    // Parse URL
    let (host, path, port) = parse_https_url(url)?;
    
    crate::serial_println!("[HTTPS] GET https://{}:{}{}", host, port, path);
    
    // Resolve DNS
    let ip = crate::netstack::dns::resolve(&host)
        .ok_or(HttpsError::DnsError)?;
    
    crate::serial_println!("[HTTPS] Resolved {} -> {}.{}.{}.{}", host, ip[0], ip[1], ip[2], ip[3]);
    
    // TCP connect
    let src_port = tcp::send_syn(ip, port)
        .map_err(|_| HttpsError::ConnectionFailed)?;
    
    // Wait for connection
    if !tcp::wait_for_established(ip, port, src_port, 5000) {
        return Err(HttpsError::ConnectionFailed);
    }
    
    crate::serial_println!("[HTTPS] TCP connected, starting TLS handshake");
    
    // Create TLS session
    let mut session = TlsSession::new(&host);
    
    // TLS handshake
    {
        let mut send = |data: &[u8]| -> Result<(), TlsError> {
            tcp::send_payload(ip, port, src_port, data)
                .map_err(|_| TlsError::ConnectionFailed)
        };
        
        let mut recv = |buf: &mut [u8]| -> Result<usize, TlsError> {
            // Poll and receive TCP data
            for _ in 0..100 {
                crate::netstack::poll();
                
                if let Some(data) = tcp::recv_data(ip, port, src_port) {
                    let len = data.len().min(buf.len());
                    buf[..len].copy_from_slice(&data[..len]);
                    return Ok(len);
                }
                
                // Small delay
                for _ in 0..50000 { core::hint::spin_loop(); }
            }
            
            Err(TlsError::WouldBlock)
        };
        
        do_handshake(&mut session, &mut send, &mut recv)?;
    }
    
    crate::serial_println!("[HTTPS] TLS handshake complete");
    
    // Build HTTP request
    let request = alloc::format!(
        "GET {} HTTP/1.1\r\n\
         Host: {}\r\n\
         User-Agent: TrustOS/1.0\r\n\
         Accept: */*\r\n\
         Connection: close\r\n\
         \r\n",
        path, host
    );
    
    // Encrypt and send request
    let encrypted = session.encrypt(request.as_bytes())?;
    tcp::send_payload(ip, port, src_port, &encrypted)
        .map_err(|_| HttpsError::ConnectionFailed)?;
    
    crate::serial_println!("[HTTPS] Request sent, waiting for response");
    
    // Receive and decrypt response
    let mut response_data = Vec::new();
    
    for _ in 0..200 {
        crate::netstack::poll();
        
        if let Some(data) = tcp::recv_data(ip, port, src_port) {
            // Decrypt the record
            if let Some(plaintext) = process_tls_records(&mut session, &data) {
                response_data.extend_from_slice(&plaintext);
            }
        }
        
        // Check if we have a complete response
        if response_data.len() > 12 {
            // Look for end of HTTP response
            if response_data.windows(4).any(|w| w == b"\r\n\r\n") {
                // Check Content-Length or chunked encoding
                if is_response_complete(&response_data) {
                    break;
                }
            }
        }
        
        // Check for connection close
        if tcp::fin_received(ip, port, src_port) {
            break;
        }
        
        for _ in 0..10000 { core::hint::spin_loop(); }
    }
    
    // Close connection
    let _ = tcp::send_fin(ip, port, src_port);
    
    // Parse HTTP response
    parse_http_response(&response_data)
}

/// Parse HTTPS URL
fn parse_https_url(url: &str) -> Result<(String, String, u16), HttpsError> {
    let url = url.strip_prefix("https://").unwrap_or(url);
    
    // Split host and path
    let (host_port, path) = if let Some(slash_pos) = url.find('/') {
        (&url[..slash_pos], &url[slash_pos..])
    } else {
        (url, "/")
    };
    
    // Split host and port
    let (host, port) = if let Some(colon_pos) = host_port.rfind(':') {
        let port_str = &host_port[colon_pos + 1..];
        let port = port_str.parse().unwrap_or(443);
        (&host_port[..colon_pos], port)
    } else {
        (host_port, 443u16)
    };
    
    Ok((String::from(host), String::from(path), port))
}

/// Process TLS records and extract plaintext
fn process_tls_records(session: &mut TlsSession, data: &[u8]) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let mut pos = 0;
    
    while pos + 5 <= data.len() {
        let length = u16::from_be_bytes([data[pos + 3], data[pos + 4]]) as usize;
        
        if pos + 5 + length > data.len() {
            break;
        }
        
        if let Ok(Some(plaintext)) = session.process_record(&data[pos..pos + 5 + length]) {
            // Skip the content type byte at the end
            if let Some((&content_type, content)) = plaintext.split_last() {
                if content_type == 23 || content_type == 0 {
                    // ApplicationData or padding
                    result.extend_from_slice(content);
                }
            } else {
                result.extend_from_slice(&plaintext);
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

/// Check if HTTP response is complete
fn is_response_complete(data: &[u8]) -> bool {
    let data_str = String::from_utf8_lossy(data);
    
    // Find headers end
    if let Some(headers_end) = data_str.find("\r\n\r\n") {
        let headers = &data_str[..headers_end];
        
        // Check for Content-Length
        for line in headers.lines() {
            if line.to_lowercase().starts_with("content-length:") {
                if let Some(len_str) = line.split(':').nth(1) {
                    if let Ok(content_len) = len_str.trim().parse::<usize>() {
                        let body_start = headers_end + 4;
                        return data.len() >= body_start + content_len;
                    }
                }
            }
        }
        
        // Check for Transfer-Encoding: chunked
        if headers.to_lowercase().contains("transfer-encoding: chunked") {
            // Look for final chunk (0\r\n\r\n)
            return data_str.contains("0\r\n\r\n") || data_str.ends_with("0\r\n");
        }
        
        // No Content-Length, assume Connection: close
        return true;
    }
    
    false
}

/// Parse HTTP response
fn parse_http_response(data: &[u8]) -> Result<HttpsResponse, HttpsError> {
    let data_str = String::from_utf8_lossy(data);
    
    // Find status line end
    let status_end = data_str.find("\r\n").ok_or(HttpsError::InvalidResponse)?;
    let status_line = &data_str[..status_end];
    
    // Parse status code
    let parts: Vec<&str> = status_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(HttpsError::InvalidResponse);
    }
    
    let status_code: u16 = parts[1].parse().map_err(|_| HttpsError::InvalidResponse)?;
    
    // Find headers end
    let headers_end = data_str.find("\r\n\r\n").ok_or(HttpsError::InvalidResponse)?;
    let headers_str = &data_str[status_end + 2..headers_end];
    
    // Parse headers
    let mut headers = Vec::new();
    for line in headers_str.lines() {
        if let Some(colon_pos) = line.find(':') {
            let name = line[..colon_pos].trim().to_string();
            let value = line[colon_pos + 1..].trim().to_string();
            headers.push((name, value));
        }
    }
    
    // Extract body
    let body_start = headers_end + 4;
    let body = if body_start < data.len() {
        data[body_start..].to_vec()
    } else {
        Vec::new()
    };
    
    Ok(HttpsResponse {
        status_code,
        headers,
        body,
    })
}
