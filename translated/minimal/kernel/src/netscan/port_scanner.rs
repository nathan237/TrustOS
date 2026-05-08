








use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    Open,
    Closed,
    Filtered,        
    OpenFiltered,    
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


#[derive(Debug, Clone)]
pub struct Cf {
    pub port: u16,
    pub state: PortState,
    pub service: &'static str,
    pub banner: Option<String>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanType {
    Syn,         
    Connect,     
    Udp,         
}


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
            ports: super::ABY_.to_vec(),
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

    pub fn with_timeout(mut self, dh: u32) -> Self {
        self.timeout_ms = dh;
        self
    }

    pub fn with_banner(mut self, grab: bool) -> Self {
        self.grab_banner = grab;
        self
    }

    pub fn with_top_ports(mut self) -> Self {
        self.ports = super::BJQ_.to_vec();
        self
    }
}


#[derive(Debug, Default)]
pub struct Qe {
    pub total_ports: usize,
    pub open: usize,
    pub closed: usize,
    pub filtered: usize,
    pub elapsed_ms: u64,
}


pub fn scan(config: &ScanConfig) -> (Vec<Cf>, Qe) {
    let start = crate::logger::eg();
    let mut results = Vec::new();
    let mut stats = Qe {
        total_ports: config.ports.len(),
        ..Default::default()
    };

    for &port in &config.ports {
        let result = match config.scan_type {
            ScanType::Syn => ozn(config.target, port, config.timeout_ms),
            ScanType::Connect => kxb(config.target, port, config.timeout_ms),
            ScanType::Udp => ppi(config.target, port, config.timeout_ms),
        };

        match result.state {
            PortState::Open => stats.open += 1,
            PortState::Closed => stats.closed += 1,
            PortState::Filtered | PortState::OpenFiltered => stats.filtered += 1,
        }

        
        if result.state != PortState::Closed {
            results.push(result);
        }
    }

    stats.elapsed_ms = crate::logger::eg().saturating_sub(start);
    (results, stats)
}







fn ozn(target: [u8; 4], port: u16, timeout_ms: u32) -> Cf {
    let service = super::cqk(port);

    
    let src_port = match crate::netstack::tcp::azp(target, port) {
        Ok(aa) => aa,
        Err(_) => {
            return Cf { port, state: PortState::Filtered, service, banner: None };
        }
    };

    
    let start = crate::logger::eg();
    let mut my: u32 = 0;

    loop {
        crate::netstack::poll();

        if let Some(state) = crate::netstack::tcp::ibk(target, port, src_port) {
            match state {
                crate::netstack::tcp::TcpState::Established => {
                    
                    let _ = oob(target, port, src_port);
                    return Cf { port, state: PortState::Open, service, banner: None };
                }
                crate::netstack::tcp::TcpState::Closed => {
                    return Cf { port, state: PortState::Closed, service, banner: None };
                }
                _ => {}
            }
        }

        if crate::logger::eg().saturating_sub(start) > timeout_ms as u64 {
            return Cf { port, state: PortState::Filtered, service, banner: None };
        }
        my = my.wrapping_add(1);
        if my > 500_000 {
            return Cf { port, state: PortState::Filtered, service, banner: None };
        }
        core::hint::spin_loop();
    }
}


fn oob(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> Result<(), &'static str> {
    let src_ip = crate::network::rd()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([10, 0, 2, 15]);

    let mut segment = alloc::vec::Vec::with_capacity(20);
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&0u32.to_be_bytes()); 
    segment.extend_from_slice(&0u32.to_be_bytes()); 
    segment.push(0x50); 
    segment.push(crate::netstack::tcp::flags::Adg);
    segment.extend_from_slice(&0u16.to_be_bytes()); 
    segment.extend_from_slice(&0u16.to_be_bytes()); 
    segment.extend_from_slice(&0u16.to_be_bytes()); 

    
    let mut bit = alloc::vec::Vec::with_capacity(32);
    bit.extend_from_slice(&src_ip);
    bit.extend_from_slice(&dest_ip);
    bit.push(0);
    bit.push(6);
    bit.extend_from_slice(&(segment.len() as u16).to_be_bytes());
    bit.extend_from_slice(&segment);
    let ig = moq(&bit);
    segment[16] = (ig >> 8) as u8;
    segment[17] = (ig & 0xFF) as u8;

    crate::netstack::ip::aha(dest_ip, 6, &segment)
}

fn moq(data: &[u8]) -> u16 {
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


fn kxb(target: [u8; 4], port: u16, timeout_ms: u32) -> Cf {
    let service = super::cqk(port);

    let src_port = match crate::netstack::tcp::azp(target, port) {
        Ok(aa) => aa,
        Err(_) => {
            return Cf { port, state: PortState::Filtered, service, banner: None };
        }
    };

    
    let cja = crate::netstack::tcp::bjy(target, port, src_port, timeout_ms);

    if cja {
        
        let _ = crate::netstack::tcp::ams(target, port, src_port);
        Cf { port, state: PortState::Open, service, banner: None }
    } else {
        
        match crate::netstack::tcp::ibk(target, port, src_port) {
            Some(crate::netstack::tcp::TcpState::Closed) => {
                Cf { port, state: PortState::Closed, service, banner: None }
            }
            _ => {
                Cf { port, state: PortState::Filtered, service, banner: None }
            }
        }
    }
}


fn ppi(target: [u8; 4], port: u16, timeout_ms: u32) -> Cf {
    let service = super::cqk(port);

    
    let payload = pph(port);
    let src_port = crate::netstack::udp::heu();

    if crate::netstack::udp::azq(target, port, src_port, &payload).is_err() {
        return Cf { port, state: PortState::Filtered, service, banner: None };
    }

    
    let start = crate::logger::eg();
    let mut my: u32 = 0;

    loop {
        crate::netstack::poll();

        
        if crate::netstack::udp::eyc(src_port).is_some() {
            return Cf { port, state: PortState::Open, service, banner: None };
        }

        
        if let Some(err) = crate::netstack::icmp::pti(target, 0) {
            if err.error_type == crate::netstack::icmp::AYT_ && err.code == 3 {
                return Cf { port, state: PortState::Closed, service, banner: None };
            }
        }

        if crate::logger::eg().saturating_sub(start) > timeout_ms as u64 {
            return Cf { port, state: PortState::OpenFiltered, service, banner: None };
        }
        my = my.wrapping_add(1);
        if my > 500_000 {
            return Cf { port, state: PortState::OpenFiltered, service, banner: None };
        }
        core::hint::spin_loop();
    }
}


fn pph(port: u16) -> Vec<u8> {
    match port {
        
        53 => {
            let mut dns = Vec::new();
            dns.extend_from_slice(&[0x00, 0x01]); 
            dns.extend_from_slice(&[0x01, 0x00]); 
            dns.extend_from_slice(&[0x00, 0x01]); 
            dns.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); 
            dns.extend_from_slice(&[0x07]); 
            dns.extend_from_slice(b"version");
            dns.extend_from_slice(&[0x04]); 
            dns.extend_from_slice(b"bind");
            dns.extend_from_slice(&[0x00]); 
            dns.extend_from_slice(&[0x00, 0x10]); 
            dns.extend_from_slice(&[0x00, 0x03]); 
            dns
        }
        
        161 => {
            alloc::vec![
                0x30, 0x26, 0x02, 0x01, 0x01, 0x04, 0x06, 0x70, 0x75, 0x62, 0x6C, 0x69, 0x63,
                0xA0, 0x19, 0x02, 0x01, 0x01, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x30, 0x0E,
                0x30, 0x0C, 0x06, 0x08, 0x2B, 0x06, 0x01, 0x02, 0x01, 0x01, 0x01, 0x00, 0x05, 0x00,
            ]
        }
        
        123 => {
            let mut irf = alloc::vec![0u8; 48];
            irf[0] = 0x1B; 
            irf
        }
        
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
        
        1900 => {
            let bk = b"M-SEARCH * HTTP/1.1\r\nHost:239.255.255.250:1900\r\nST:ssdp:all\r\nMAN:\"ssdp:discover\"\r\nMX:1\r\n\r\n";
            bk.to_vec()
        }
        
        _ => alloc::vec![0x00; 4],
    }
}


pub fn ixj(target: [u8; 4]) -> (Vec<Cf>, Qe) {
    let config = ScanConfig::new(target);
    scan(&config)
}


pub fn qlq(target: [u8; 4]) -> (Vec<Cf>, Qe) {
    let config = ScanConfig::new(target)
        .with_top_ports()
        .with_banner(true)
        .with_timeout(2000);
    scan(&config)
}
