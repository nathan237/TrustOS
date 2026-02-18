//! TrustScan — Network Security Scanning Toolkit
//!
//! Bare-metal network reconnaissance and security analysis tools.
//! Direct hardware access provides faster, stealthier scans than userspace tools.
//!
//! Modules:
//! - `port_scanner` — TCP SYN/Connect/UDP port scanning (nmap-style)
//! - `discovery`    — Host discovery via ARP sweep, ICMP ping sweep
//! - `banner`       — Service banner grabbing and version detection
//! - `sniffer`      — Packet capture and protocol analysis
//! - `traceroute`   — Real TTL-based traceroute with ICMP
//! - `vuln`         — Basic vulnerability fingerprinting

pub mod port_scanner;
pub mod discovery;
pub mod banner;
pub mod sniffer;
pub mod traceroute;
pub mod vuln;

use alloc::string::String;
use alloc::format;

/// Format IP address as string
pub fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

/// Format MAC address as string
pub fn format_mac(mac: [u8; 6]) -> String {
    format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5])
}

/// Parse an IP address string "a.b.c.d" into bytes
pub fn parse_ip(s: &str) -> Option<[u8; 4]> {
    let parts: alloc::vec::Vec<&str> = s.split('.').collect();
    if parts.len() != 4 { return None; }
    Some([
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
        parts[3].parse().ok()?,
    ])
}

/// Resolve hostname or IP string to IP bytes
pub fn resolve_target(target: &str) -> Option<[u8; 4]> {
    if let Some(ip) = parse_ip(target) {
        Some(ip)
    } else {
        crate::netstack::dns::resolve(target)
    }
}

/// Common well-known ports for quick scans
pub const COMMON_PORTS: &[u16] = &[
    21, 22, 23, 25, 53, 80, 110, 111, 135, 139, 143, 443, 445,
    993, 995, 1433, 1521, 3306, 3389, 5432, 5900, 6379, 8080, 8443, 9200,
];

/// Top 100 most common ports (nmap-style)
pub const TOP_100_PORTS: &[u16] = &[
    7, 9, 13, 21, 22, 23, 25, 26, 37, 53, 79, 80, 81, 88, 106, 110, 111,
    113, 119, 135, 139, 143, 144, 179, 199, 389, 427, 443, 444, 445, 465,
    513, 514, 515, 543, 544, 548, 554, 587, 631, 646, 873, 990, 993, 995,
    1025, 1026, 1027, 1028, 1029, 1110, 1433, 1720, 1723, 1755, 1900,
    2000, 2001, 2049, 2121, 2717, 3000, 3128, 3306, 3389, 3986, 4899,
    5000, 5009, 5051, 5060, 5101, 5190, 5357, 5432, 5631, 5666, 5800,
    5900, 6000, 6001, 6646, 7070, 8000, 8008, 8009, 8080, 8081, 8443,
    8888, 9100, 9999, 10000, 32768, 49152, 49153, 49154, 49155, 49156,
];

/// Get service name for a well-known port
pub fn service_name(port: u16) -> &'static str {
    match port {
        7 => "echo",
        9 => "discard",
        13 => "daytime",
        20 => "ftp-data",
        21 => "ftp",
        22 => "ssh",
        23 => "telnet",
        25 => "smtp",
        37 => "time",
        53 => "dns",
        67 => "dhcp-server",
        68 => "dhcp-client",
        69 => "tftp",
        79 => "finger",
        80 => "http",
        88 => "kerberos",
        110 => "pop3",
        111 => "rpcbind",
        113 => "ident",
        119 => "nntp",
        123 => "ntp",
        135 => "msrpc",
        137 => "netbios-ns",
        138 => "netbios-dgm",
        139 => "netbios-ssn",
        143 => "imap",
        161 => "snmp",
        162 => "snmptrap",
        179 => "bgp",
        389 => "ldap",
        443 => "https",
        445 => "microsoft-ds",
        465 => "smtps",
        514 => "syslog",
        515 => "printer",
        520 => "rip",
        543 => "klogin",
        544 => "kshell",
        554 => "rtsp",
        587 => "submission",
        631 => "ipp",
        636 => "ldaps",
        873 => "rsync",
        990 => "ftps",
        993 => "imaps",
        995 => "pop3s",
        1080 => "socks",
        1433 => "ms-sql",
        1434 => "ms-sql-m",
        1521 => "oracle",
        1723 => "pptp",
        1883 => "mqtt",
        2049 => "nfs",
        2181 => "zookeeper",
        3000 => "grafana",
        3128 => "squid",
        3306 => "mysql",
        3389 => "rdp",
        4443 => "pharos",
        5000 => "upnp",
        5060 => "sip",
        5432 => "postgresql",
        5672 => "amqp",
        5900 => "vnc",
        6379 => "redis",
        6443 => "k8s-api",
        6667 => "irc",
        8000 => "http-alt",
        8008 => "http-alt2",
        8080 => "http-proxy",
        8443 => "https-alt",
        8888 => "http-alt3",
        9090 => "prometheus",
        9200 => "elasticsearch",
        9418 => "git",
        11211 => "memcached",
        27017 => "mongodb",
        _ => "unknown",
    }
}
