









use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use alloc::collections::VecDeque;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};


const CIO_: usize = 256;


static NA_: Mutex<VecDeque<Cb>> = Mutex::new(VecDeque::new());

static SZ_: AtomicBool = AtomicBool::new(false);

static XH_: AtomicU64 = AtomicU64::new(0);

static ABE_: AtomicU64 = AtomicU64::new(0);


#[derive(Debug, Clone)]
pub struct Cb {
    pub timestamp_ms: u64,
    pub length: usize,
    pub protocol: Protocol,
    pub src_ip: Option<[u8; 4]>,
    pub dst_ip: Option<[u8; 4]>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub src_mac: [u8; 6],
    pub dst_mac: [u8; 6],
    pub flags: u8,           
    pub info: String,        
    pub raw_data: Vec<u8>,   
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Arp,
    Icmp,
    Tcp,
    Udp,
    Dns,
    Http,
    Tls,
    Dhcp,
    Ipv6,
    Unknown(u8),
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Arp => "ARP",
            Protocol::Icmp => "ICMP",
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
            Protocol::Dns => "DNS",
            Protocol::Http => "HTTP",
            Protocol::Tls => "TLS",
            Protocol::Dhcp => "DHCP",
            Protocol::Ipv6 => "IPv6",
            Protocol::Unknown(_) => "???",
        }
    }
}


#[derive(Debug, Clone, Default)]
pub struct CaptureFilter {
    pub src_ip: Option<[u8; 4]>,
    pub dst_ip: Option<[u8; 4]>,
    pub port: Option<u16>,
    pub protocol: Option<Protocol>,
}


pub fn deu() {
    SZ_.store(true, Ordering::SeqCst);
    XH_.store(0, Ordering::SeqCst);
    ABE_.store(0, Ordering::SeqCst);
    NA_.lock().clear();
    crate::serial_println!("[SNIFFER] Capture started");
}


pub fn dex() {
    SZ_.store(false, Ordering::SeqCst);
    crate::serial_println!("[SNIFFER] Capture stopped");
}


pub fn btp() -> bool {
    SZ_.load(Ordering::SeqCst)
}


pub fn get_stats() -> (u64, u64, usize) {
    let count = XH_.load(Ordering::SeqCst);
    let bytes = ABE_.load(Ordering::SeqCst);
    let awl = NA_.lock().len();
    (count, bytes, awl)
}


pub fn npc() -> u64 {
    XH_.load(Ordering::Relaxed)
}


pub fn fyk() -> Vec<Cb> {
    let mut buf = NA_.lock();
    buf.drain(..).collect()
}


pub fn ewn(count: usize) -> Vec<Cb> {
    let buf = NA_.lock();
    buf.iter().rev().take(count).cloned().collect()
}



pub fn exa(dm: &[u8]) {
    if !SZ_.load(Ordering::SeqCst) {
        return;
    }
    if dm.len() < 14 {
        return;
    }

    XH_.fetch_add(1, Ordering::SeqCst);
    ABE_.fetch_add(dm.len() as u64, Ordering::SeqCst);

    let be = lfs(dm);

    let mut buf = NA_.lock();
    if buf.len() >= CIO_ {
        buf.pop_front();
    }
    buf.push_back(be);
}


fn lfs(dm: &[u8]) -> Cb {
    let dst_mac = [dm[0], dm[1], dm[2], dm[3], dm[4], dm[5]];
    let src_mac = [dm[6], dm[7], dm[8], dm[9], dm[10], dm[11]];
    let ethertype = u16::from_be_bytes([dm[12], dm[13]]);

    let timestamp_ms = crate::logger::eg();
    let raw_data = dm[..dm.len().min(128)].to_vec();

    match ethertype {
        0x0806 => lfp(&dm[14..], dst_mac, src_mac, timestamp_ms, raw_data, dm.len()),
        0x0800 => lfr(&dm[14..], dst_mac, src_mac, timestamp_ms, raw_data, dm.len()),
        0x86DD => Cb {
            timestamp_ms,
            length: dm.len(),
            protocol: Protocol::Ipv6,
            src_ip: None,
            dst_ip: None,
            src_port: None,
            dst_port: None,
            src_mac,
            dst_mac,
            flags: 0,
            info: String::from("IPv6 packet"),
            raw_data,
        },
        _ => Cb {
            timestamp_ms,
            length: dm.len(),
            protocol: Protocol::Unknown(0),
            src_ip: None,
            dst_ip: None,
            src_port: None,
            dst_port: None,
            src_mac,
            dst_mac,
            flags: 0,
            info: format!("EtherType 0x{:04X}", ethertype),
            raw_data,
        },
    }
}

fn lfp(data: &[u8], dst_mac: [u8; 6], src_mac: [u8; 6], jy: u64, dm: Vec<u8>, len: usize) -> Cb {
    let mut info = String::from("ARP");
    let mut src_ip = None;
    let mut dst_ip = None;

    if data.len() >= 28 {
        let op = u16::from_be_bytes([data[6], data[7]]);
        let sender = [data[14], data[15], data[16], data[17]];
        let target = [data[24], data[25], data[26], data[27]];
        src_ip = Some(sender);
        dst_ip = Some(target);

        info = match op {
            1 => format!("Who has {}? Tell {}", super::uw(target), super::uw(sender)),
            2 => format!("{} is at {}", super::uw(sender), super::bzx(src_mac)),
            _ => format!("ARP op={}", op),
        };
    }

    Cb {
        timestamp_ms: jy, length: len, protocol: Protocol::Arp,
        src_ip, dst_ip, src_port: None, dst_port: None,
        src_mac, dst_mac, flags: 0, info, raw_data: dm,
    }
}

fn lfr(data: &[u8], dst_mac: [u8; 6], src_mac: [u8; 6], jy: u64, dm: Vec<u8>, len: usize) -> Cb {
    if data.len() < 20 {
        return Cb {
            timestamp_ms: jy, length: len, protocol: Protocol::Unknown(0),
            src_ip: None, dst_ip: None, src_port: None, dst_port: None,
            src_mac, dst_mac, flags: 0, info: String::from("Malformed IPv4"), raw_data: dm,
        };
    }

    let gbr = (data[0] & 0x0F) as usize;
    let bte = gbr * 4;
    let protocol = data[9];
    let src_ip = [data[12], data[13], data[14], data[15]];
    let dst_ip = [data[16], data[17], data[18], data[19]];

    if data.len() < bte {
        return Cb {
            timestamp_ms: jy, length: len, protocol: Protocol::Unknown(protocol),
            src_ip: Some(src_ip), dst_ip: Some(dst_ip), src_port: None, dst_port: None,
            src_mac, dst_mac, flags: 0, info: format!("IPv4 proto={}", protocol), raw_data: dm,
        };
    }

    let payload = &data[bte..];

    match protocol {
        1 => lfq(payload, src_ip, dst_ip, dst_mac, src_mac, jy, dm, len),
        6 => lft(payload, src_ip, dst_ip, dst_mac, src_mac, jy, dm, len),
        17 => lfu(payload, src_ip, dst_ip, dst_mac, src_mac, jy, dm, len),
        _ => Cb {
            timestamp_ms: jy, length: len, protocol: Protocol::Unknown(protocol),
            src_ip: Some(src_ip), dst_ip: Some(dst_ip), src_port: None, dst_port: None,
            src_mac, dst_mac, flags: 0, info: format!("IP Proto {}", protocol), raw_data: dm,
        },
    }
}

fn lfq(data: &[u8], src_ip: [u8; 4], dst_ip: [u8; 4], dst_mac: [u8; 6], src_mac: [u8; 6], jy: u64, dm: Vec<u8>, len: usize) -> Cb {
    let info = if data.len() >= 8 {
        let ifk = data[0];
        let code = data[1];
        match ifk {
            0 => format!("Echo Reply seq={}", u16::from_be_bytes([data[6], data[7]])),
            3 => format!("Destination Unreachable code={}", code),
            8 => format!("Echo Request seq={}", u16::from_be_bytes([data[6], data[7]])),
            11 => format!("Time Exceeded (TTL={}) code={}", code, code),
            _ => format!("ICMP type={} code={}", ifk, code),
        }
    } else {
        String::from("ICMP (truncated)")
    };

    Cb {
        timestamp_ms: jy, length: len, protocol: Protocol::Icmp,
        src_ip: Some(src_ip), dst_ip: Some(dst_ip), src_port: None, dst_port: None,
        src_mac, dst_mac, flags: 0, info, raw_data: dm,
    }
}

fn lft(data: &[u8], src_ip: [u8; 4], dst_ip: [u8; 4], dst_mac: [u8; 6], src_mac: [u8; 6], jy: u64, dm: Vec<u8>, len: usize) -> Cb {
    if data.len() < 20 {
        return Cb {
            timestamp_ms: jy, length: len, protocol: Protocol::Tcp,
            src_ip: Some(src_ip), dst_ip: Some(dst_ip), src_port: None, dst_port: None,
            src_mac, dst_mac, flags: 0, info: String::from("TCP (truncated)"), raw_data: dm,
        };
    }

    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let seq = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let bdy = data[13];
    let data_offset = (data[12] >> 4) as usize * 4;
    let payload_len = if data.len() > data_offset { data.len() - data_offset } else { 0 };

    
    let protocol = if dst_port == 80 || src_port == 80 || dst_port == 8080 || src_port == 8080 {
        Protocol::Http
    } else if dst_port == 443 || src_port == 443 {
        Protocol::Tls
    } else if dst_port == 53 || src_port == 53 {
        Protocol::Dns
    } else {
        Protocol::Tcp
    };

    
    let mut bzs = String::new();
    if bdy & 0x02 != 0 { bzs.push_str("[SYN]"); }
    if bdy & 0x10 != 0 { bzs.push_str("[ACK]"); }
    if bdy & 0x01 != 0 { bzs.push_str("[FIN]"); }
    if bdy & 0x04 != 0 { bzs.push_str("[RST]"); }
    if bdy & 0x08 != 0 { bzs.push_str("[PSH]"); }
    if bzs.is_empty() { bzs.push_str("[...]"); }

    let info = format!("{} -> {} {} seq={} len={}", src_port, dst_port, bzs, seq, payload_len);

    Cb {
        timestamp_ms: jy, length: len, protocol,
        src_ip: Some(src_ip), dst_ip: Some(dst_ip),
        src_port: Some(src_port), dst_port: Some(dst_port),
        src_mac, dst_mac, flags: bdy, info, raw_data: dm,
    }
}

fn lfu(data: &[u8], src_ip: [u8; 4], dst_ip: [u8; 4], dst_mac: [u8; 6], src_mac: [u8; 6], jy: u64, dm: Vec<u8>, len: usize) -> Cb {
    if data.len() < 8 {
        return Cb {
            timestamp_ms: jy, length: len, protocol: Protocol::Udp,
            src_ip: Some(src_ip), dst_ip: Some(dst_ip), src_port: None, dst_port: None,
            src_mac, dst_mac, flags: 0, info: String::from("UDP (truncated)"), raw_data: dm,
        };
    }

    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let hao = u16::from_be_bytes([data[4], data[5]]);

    let protocol = if dst_port == 53 || src_port == 53 {
        Protocol::Dns
    } else if dst_port == 67 || dst_port == 68 || src_port == 67 || src_port == 68 {
        Protocol::Dhcp
    } else {
        Protocol::Udp
    };

    let info = format!("{} -> {} len={}", src_port, dst_port, hao);

    Cb {
        timestamp_ms: jy, length: len, protocol,
        src_ip: Some(src_ip), dst_ip: Some(dst_ip),
        src_port: Some(src_port), dst_port: Some(dst_port),
        src_mac, dst_mac, flags: 0, info, raw_data: dm,
    }
}


pub fn iet(data: &[u8], max_bytes: usize) -> String {
    let mut output = String::new();
    let len = data.len().min(max_bytes);

    for (i, df) in data[..len].chunks(16).enumerate() {
        output.push_str(&format!("{:04X}  ", i * 16));

        
        for (ay, &b) in df.iter().enumerate() {
            output.push_str(&format!("{:02X} ", b));
            if ay == 7 { output.push(' '); }
        }

        
        for _ in df.len()..16 {
            output.push_str("   ");
        }
        if df.len() <= 8 { output.push(' '); }

        output.push_str(" |");
        
        for &b in df {
            if (0x20..=0x7E).contains(&b) {
                output.push(b as char);
            } else {
                output.push('.');
            }
        }
        output.push_str("|\n");
    }

    output
}
