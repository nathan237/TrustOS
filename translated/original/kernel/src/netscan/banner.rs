//! Service Banner Grabber — Connect to open ports and identify services
//!
//! Grabs service banners to identify software versions.
//! Sends protocol-specific probes to trigger informative responses.

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

/// Banner grab result
#[derive(Debug, Clone)]
pub struct BannerResult {
    pub port: u16,
    pub service: &'static str,
    pub banner: String,
    pub version: Option<String>,
}

/// Grab banner from an open TCP port
pub fn grab_banner(target: [u8; 4], port: u16, timeout_ms: u32) -> Option<BannerResult> {
    let service = super::service_name(port);

    // Connect to target
    let src_port = crate::netstack::tcp::send_syn(target, port).ok()?;
    if !crate::netstack::tcp::wait_for_established(target, port, src_port, timeout_ms) {
        return None;
    }

    // Send protocol-specific probe
    let probe = get_probe(port);
    if !probe.is_empty() {
        let _ = crate::netstack::tcp::send_payload(target, port, src_port, &probe);
    }

    // Wait for response
    let mut banner_data = Vec::new();
    let start = crate::logger::get_ticks();
    let mut spins: u32 = 0;

    loop {
        crate::netstack::poll();

        if let Some(data) = crate::netstack::tcp::recv_data(target, port, src_port) {
            banner_data.extend_from_slice(&data);
            // Got some data, break early if enough
            if banner_data.len() > 4 {
                break;
            }
        }

        if crate::logger::get_ticks().saturating_sub(start) > timeout_ms as u64 {
            break;
        }
        spins = spins.wrapping_add(1);
        if spins > 500_000 { break; }
        core::hint::spin_loop();
    }

    // Clean close
    let _ = crate::netstack::tcp::send_fin(target, port, src_port);

    if banner_data.is_empty() {
        return None;
    }

    // Parse banner
    let banner_str = sanitize_banner(&banner_data);
    let version = extract_version(&banner_str, port);

    Some(BannerResult {
        port,
        service,
        banner: banner_str,
        version,
    })
}

/// Grab banners from multiple ports
pub fn grab_banners(target: [u8; 4], ports: &[u16], timeout_ms: u32) -> Vec<BannerResult> {
    let mut results = Vec::new();
    for &port in ports {
        if let Some(result) = grab_banner(target, port, timeout_ms) {
            results.push(result);
        }
    }
    results
}

/// Get protocol-specific probe for a port
fn get_probe(port: u16) -> Vec<u8> {
    match port {
        // HTTP: send GET request
        80 | 8080 | 8000 | 8008 | 8443 | 443 => {
            b"GET / HTTP/1.0\r\nHost: target\r\nUser-Agent: TrustScan/1.0\r\n\r\n".to_vec()
        }
        // FTP: server sends banner on connect, no probe needed
        21 => Vec::new(),
        // SSH: server sends banner on connect
        22 => Vec::new(),
        // SMTP: server sends banner on connect
        25 | 587 | 465 => Vec::new(),
        // POP3: server sends banner on connect
        110 | 995 => Vec::new(),
        // IMAP: server sends banner on connect
        143 | 993 => Vec::new(),
        // MySQL: server sends greeting on connect
        3306 => Vec::new(),
        // PostgreSQL: send startup message (dummy)
        5432 => Vec::new(),
        // Redis: send INFO
        6379 => b"INFO server\r\n".to_vec(),
        // RTSP
        554 => b"OPTIONS * RTSP/1.0\r\nCSeq: 1\r\n\r\n".to_vec(),
        // MongoDB
        27017 => {
            // isMaster command
            alloc::vec![
                0x3F, 0x00, 0x00, 0x00, // message length
                0x01, 0x00, 0x00, 0x00, // request ID
                0x00, 0x00, 0x00, 0x00, // response to
                0xD4, 0x07, 0x00, 0x00, // opcode: OP_QUERY
                0x00, 0x00, 0x00, 0x00, // flags
                0x61, 0x64, 0x6D, 0x69, 0x6E, 0x2E, 0x24, 0x63, 0x6D, 0x64, 0x00, // admin.$cmd
                0x00, 0x00, 0x00, 0x00, // skip
                0x01, 0x00, 0x00, 0x00, // return
                0x15, 0x00, 0x00, 0x00, // doc size
                0x01, 0x69, 0x73, 0x4D, 0x61, 0x73, 0x74, 0x65, 0x72, 0x00, // "isMaster"
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // double 1.0
                0x00, // doc end
            ]
        }
        // Telnet: server typically sends banner
        23 => Vec::new(),
        // Generic probe
        _ => b"\r\n".to_vec(),
    }
}

/// Sanitize banner bytes to printable string
fn sanitize_banner(data: &[u8]) -> String {
    let mut s = String::new();
    let max_len = data.len().min(256); // Limit banner length

    for &b in &data[..max_len] {
        match b {
            0x20..=0x7E => s.push(b as char),
            b'\r' => {} // Skip CR
            b'\n' => {
                if !s.ends_with(' ') && !s.is_empty() {
                    s.push(' ');
                }
            }
            _ => {
                if !s.ends_with('.') {
                    s.push('.');
                }
            }
        }
    }

    s.trim().into()
}

/// Extract version info from banner
fn extract_version(banner: &str, port: u16) -> Option<String> {
    let banner_lower = banner.to_ascii_lowercase();

    // SSH
    if port == 22 || banner_lower.starts_with("ssh-") {
        if let Some(version) = banner.split_whitespace().next() {
            return Some(version.into());
        }
    }

    // HTTP Server header
    if banner_lower.contains("server:") {
        for line in banner.split(' ') {
            let l = line.trim();
            if l.starts_with("Server:") || l.starts_with("server:") {
                return Some(l[7..].trim().into());
            }
        }
    }

    // Apache / nginx
    if banner_lower.contains("apache") {
        return Some("Apache".into());
    }
    if banner_lower.contains("nginx") {
        return Some("nginx".into());
    }

    // FTP
    if banner_lower.contains("ftp") && banner.starts_with("220") {
        return Some(banner.trim_start_matches("220").trim().into());
    }

    // SMTP
    if banner.starts_with("220") && banner_lower.contains("smtp") {
        return Some(banner.trim_start_matches("220").trim().into());
    }

    // MySQL
    if port == 3306 && banner.len() > 5 {
        return Some(format!("MySQL ({})", &banner[..banner.len().min(30)]));
    }

    // Redis
    if banner_lower.contains("redis_version:") {
        for part in banner.split(' ') {
            if part.starts_with("redis_version:") {
                return Some(part.into());
            }
        }
    }

    None
}
