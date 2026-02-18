//! Port Scanner — TCP SYN, TCP Connect, and UDP scanning
//!
//! Provides nmap-style port scanning capabilities:
//! - SYN scan (half-open, stealthier)
//! - TCP connect scan (full handshake)
//! - UDP scan (send probe, detect ICMP unreachable)
//!
//! Bare-metal advantage: direct packet crafting without raw socket privileges.

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

/// Port state result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    Open,
    Closed,
    Filtered,        // No response (firewall?)
    OpenFiltered,    // UDP: no response (could be open or filtered)
}

impl PortState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PortState::Open => "open",
            PortState::Closed => "closed",
            PortState::Filtered => "filtered",
            PortState::OpenFiltered => "open|filtered",
        }
    }
}

/// Scan result for a single port
#[derive(Debug, Clone)]
pub struct PortResult {
    pub port: u16,
    pub state: PortState,
    pub service: &'static str,
    pub banner: Option<String>,
}

/// Scan type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanType {
    Syn,         // Half-open SYN scan
    Connect,     // Full TCP connect
    Udp,         // UDP scan
}

/// Scan configuration
#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub target: [u8; 4],
    pub ports: Vec<u16>,
    pub scan_type: ScanType,
    pub timeout_ms: u32,
    pub grab_banner: bool,
}

impl ScanConfig {
    pub fn new(target: [u8; 4]) -> Self {
        Self {
            target,
            ports: super::COMMON_PORTS.to_vec(),
            scan_type: ScanType::Syn,
            timeout_ms: 1500,
            grab_banner: false,
        }
    }

    pub fn with_ports(mut self, ports: Vec<u16>) -> Self {
        self.ports = ports;
        self
    }

    pub fn with_range(mut self, start: u16, end: u16) -> Self {
        self.ports = (start..=end).collect();
        self
    }

    pub fn with_type(mut self, scan_type: ScanType) -> Self {
        self.scan_type = scan_type;
        self
    }

    pub fn with_timeout(mut self, ms: u32) -> Self {
        self.timeout_ms = ms;
        self
    }

    pub fn with_banner(mut self, grab: bool) -> Self {
        self.grab_banner = grab;
        self
    }

    pub fn with_top_ports(mut self) -> Self {
        self.ports = super::TOP_100_PORTS.to_vec();
        self
    }
}

/// Scan statistics
#[derive(Debug, Default)]
pub struct ScanStats {
    pub total_ports: usize,
    pub open: usize,
    pub closed: usize,
    pub filtered: usize,
    pub elapsed_ms: u64,
}

/// Run a port scan with the given configuration
pub fn scan(config: &ScanConfig) -> (Vec<PortResult>, ScanStats) {
    let start = crate::logger::get_ticks();
    let mut results = Vec::new();
    let mut stats = ScanStats {
        total_ports: config.ports.len(),
        ..Default::default()
    };

    for &port in &config.ports {
        let result = match config.scan_type {
            ScanType::Syn => syn_scan_port(config.target, port, config.timeout_ms),
            ScanType::Connect => connect_scan_port(config.target, port, config.timeout_ms),
            ScanType::Udp => udp_scan_port(config.target, port, config.timeout_ms),
        };

        match result.state {
            PortState::Open => stats.open += 1,
            PortState::Closed => stats.closed += 1,
            PortState::Filtered | PortState::OpenFiltered => stats.filtered += 1,
        }

        // Only store open/filtered ports (like nmap default)
        if result.state != PortState::Closed {
            results.push(result);
        }
    }

    stats.elapsed_ms = crate::logger::get_ticks().saturating_sub(start);
    (results, stats)
}

/// SYN scan a single port (half-open scan)
///
/// Send SYN → Response:
/// - SYN-ACK = open (send RST to close)
/// - RST = closed
/// - No response = filtered
fn syn_scan_port(target: [u8; 4], port: u16, timeout_ms: u32) -> PortResult {
    let service = super::service_name(port);

    // Send TCP SYN
    let src_port = match crate::netstack::tcp::send_syn(target, port) {
        Ok(p) => p,
        Err(_) => {
            return PortResult { port, state: PortState::Filtered, service, banner: None };
        }
    };

    // Wait for response with shorter timeout for speed
    let start = crate::logger::get_ticks();
    let mut spins: u32 = 0;

    loop {
        crate::netstack::poll();

        if let Some(state) = crate::netstack::tcp::get_connection_state(target, port, src_port) {
            match state {
                crate::netstack::tcp::TcpState::Established => {
                    // Port is open! Send RST to close quickly (stealth)
                    let _ = send_rst(target, port, src_port);
                    return PortResult { port, state: PortState::Open, service, banner: None };
                }
                crate::netstack::tcp::TcpState::Closed => {
                    return PortResult { port, state: PortState::Closed, service, banner: None };
                }
                _ => {}
            }
        }

        if crate::logger::get_ticks().saturating_sub(start) > timeout_ms as u64 {
            return PortResult { port, state: PortState::Filtered, service, banner: None };
        }
        spins = spins.wrapping_add(1);
        if spins > 500_000 {
            return PortResult { port, state: PortState::Filtered, service, banner: None };
        }
        core::hint::spin_loop();
    }
}

/// Send a TCP RST to close connection quickly
fn send_rst(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> Result<(), &'static str> {
    let src_ip = crate::network::get_ipv4_config()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([10, 0, 2, 15]);

    let mut segment = alloc::vec::Vec::with_capacity(20);
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&0u32.to_be_bytes()); // seq
    segment.extend_from_slice(&0u32.to_be_bytes()); // ack
    segment.push(0x50); // data offset=5
    segment.push(crate::netstack::tcp::flags::RST);
    segment.extend_from_slice(&0u16.to_be_bytes()); // window
    segment.extend_from_slice(&0u16.to_be_bytes()); // checksum
    segment.extend_from_slice(&0u16.to_be_bytes()); // urgent

    // Calculate TCP checksum
    let mut pseudo = alloc::vec::Vec::with_capacity(32);
    pseudo.extend_from_slice(&src_ip);
    pseudo.extend_from_slice(&dest_ip);
    pseudo.push(0);
    pseudo.push(6);
    pseudo.extend_from_slice(&(segment.len() as u16).to_be_bytes());
    pseudo.extend_from_slice(&segment);
    let csum = inet_checksum(&pseudo);
    segment[16] = (csum >> 8) as u8;
    segment[17] = (csum & 0xFF) as u8;

    crate::netstack::ip::send_packet(dest_ip, 6, &segment)
}

fn inet_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += ((data[i] as u32) << 8) | (data[i + 1] as u32);
        i += 2;
    }
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

/// TCP Connect scan: full three-way handshake
fn connect_scan_port(target: [u8; 4], port: u16, timeout_ms: u32) -> PortResult {
    let service = super::service_name(port);

    let src_port = match crate::netstack::tcp::send_syn(target, port) {
        Ok(p) => p,
        Err(_) => {
            return PortResult { port, state: PortState::Filtered, service, banner: None };
        }
    };

    // Wait for established connection
    let established = crate::netstack::tcp::wait_for_established(target, port, src_port, timeout_ms);

    if established {
        // Port is open — clean close
        let _ = crate::netstack::tcp::send_fin(target, port, src_port);
        PortResult { port, state: PortState::Open, service, banner: None }
    } else {
        // Check if closed (RST received) or filtered (timeout)
        match crate::netstack::tcp::get_connection_state(target, port, src_port) {
            Some(crate::netstack::tcp::TcpState::Closed) => {
                PortResult { port, state: PortState::Closed, service, banner: None }
            }
            _ => {
                PortResult { port, state: PortState::Filtered, service, banner: None }
            }
        }
    }
}

/// UDP scan: send probe, wait for ICMP unreachable = closed, no response = open|filtered
fn udp_scan_port(target: [u8; 4], port: u16, timeout_ms: u32) -> PortResult {
    let service = super::service_name(port);

    // Send UDP probe (service-specific payload for better detection)
    let payload = udp_probe_payload(port);
    let src_port = crate::netstack::udp::alloc_ephemeral_port();

    if crate::netstack::udp::send_to(target, port, src_port, &payload).is_err() {
        return PortResult { port, state: PortState::Filtered, service, banner: None };
    }

    // Wait for response or ICMP error
    let start = crate::logger::get_ticks();
    let mut spins: u32 = 0;

    loop {
        crate::netstack::poll();

        // Check for UDP response (port is definitely open)
        if crate::netstack::udp::recv_on(src_port).is_some() {
            return PortResult { port, state: PortState::Open, service, banner: None };
        }

        // Check for ICMP destination unreachable (port is closed)
        if let Some(err) = crate::netstack::icmp::wait_for_error(target, 0) {
            if err.error_type == crate::netstack::icmp::ICMP_DEST_UNREACHABLE && err.code == 3 {
                return PortResult { port, state: PortState::Closed, service, banner: None };
            }
        }

        if crate::logger::get_ticks().saturating_sub(start) > timeout_ms as u64 {
            return PortResult { port, state: PortState::OpenFiltered, service, banner: None };
        }
        spins = spins.wrapping_add(1);
        if spins > 500_000 {
            return PortResult { port, state: PortState::OpenFiltered, service, banner: None };
        }
        core::hint::spin_loop();
    }
}

/// Generate service-specific UDP probe payloads
fn udp_probe_payload(port: u16) -> Vec<u8> {
    match port {
        // DNS query for version.bind (service detection)
        53 => {
            let mut dns = Vec::new();
            dns.extend_from_slice(&[0x00, 0x01]); // Transaction ID
            dns.extend_from_slice(&[0x01, 0x00]); // Standard query
            dns.extend_from_slice(&[0x00, 0x01]); // 1 question
            dns.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // No answers/auth/additional
            dns.extend_from_slice(&[0x07]); // "version" length
            dns.extend_from_slice(b"version");
            dns.extend_from_slice(&[0x04]); // "bind" length
            dns.extend_from_slice(b"bind");
            dns.extend_from_slice(&[0x00]); // Root
            dns.extend_from_slice(&[0x00, 0x10]); // TXT record
            dns.extend_from_slice(&[0x00, 0x03]); // CH class
            dns
        }
        // SNMP GetRequest
        161 => {
            alloc::vec![
                0x30, 0x26, 0x02, 0x01, 0x01, 0x04, 0x06, 0x70, 0x75, 0x62, 0x6C, 0x69, 0x63,
                0xA0, 0x19, 0x02, 0x01, 0x01, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x30, 0x0E,
                0x30, 0x0C, 0x06, 0x08, 0x2B, 0x06, 0x01, 0x02, 0x01, 0x01, 0x01, 0x00, 0x05, 0x00,
            ]
        }
        // NTP version query
        123 => {
            let mut ntp = alloc::vec![0u8; 48];
            ntp[0] = 0x1B; // LI=0, Version=3, Mode=3 (client)
            ntp
        }
        // NetBIOS Name Service
        137 => {
            alloc::vec![
                0x80, 0x94, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x20, 0x43, 0x4B, 0x41,
                0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
                0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
                0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
                0x41, 0x41, 0x41, 0x41, 0x41, 0x00, 0x00, 0x21,
                0x00, 0x01,
            ]
        }
        // SSDP/UPnP discovery
        1900 => {
            let msg = b"M-SEARCH * HTTP/1.1\r\nHost:239.255.255.250:1900\r\nST:ssdp:all\r\nMAN:\"ssdp:discover\"\r\nMX:1\r\n\r\n";
            msg.to_vec()
        }
        // Generic empty probe
        _ => alloc::vec![0x00; 4],
    }
}

/// Quick scan: scan common ports on target
pub fn quick_scan(target: [u8; 4]) -> (Vec<PortResult>, ScanStats) {
    let config = ScanConfig::new(target);
    scan(&config)
}

/// Intensive scan: scan top 100 ports with banner grabbing
pub fn intensive_scan(target: [u8; 4]) -> (Vec<PortResult>, ScanStats) {
    let config = ScanConfig::new(target)
        .with_top_ports()
        .with_banner(true)
        .with_timeout(2000);
    scan(&config)
}
