












pub mod port_scanner;
pub mod discovery;
pub mod banner;
pub mod sniffer;
pub mod traceroute;
pub mod vuln;
pub mod replay;

use alloc::string::String;
use alloc::format;


pub fn aot(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}


pub fn eqs(ed: [u8; 6]) -> String {
    format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        ed[0], ed[1], ed[2], ed[3], ed[4], ed[5])
}


pub fn ewb(e: &str) -> Option<[u8; 4]> {
    let ek: alloc::vec::Vec<&str> = e.adk('.').collect();
    if ek.len() != 4 { return None; }
    Some([
        ek[0].parse().bq()?,
        ek[1].parse().bq()?,
        ek[2].parse().bq()?,
        ek[3].parse().bq()?,
    ])
}


pub fn lzr(cd: &str) -> Option<[u8; 4]> {
    if let Some(ip) = ewb(cd) {
        Some(ip)
    } else {
        crate::netstack::dns::ayo(cd)
    }
}


pub const AAL_: &[u16] = &[
    21, 22, 23, 25, 53, 80, 110, 111, 135, 139, 143, 443, 445,
    993, 995, 1433, 1521, 3306, 3389, 5432, 5900, 6379, 8080, 8443, 9200,
];


pub const BHM_: &[u16] = &[
    7, 9, 13, 21, 22, 23, 25, 26, 37, 53, 79, 80, 81, 88, 106, 110, 111,
    113, 119, 135, 139, 143, 144, 179, 199, 389, 427, 443, 444, 445, 465,
    513, 514, 515, 543, 544, 548, 554, 587, 631, 646, 873, 990, 993, 995,
    1025, 1026, 1027, 1028, 1029, 1110, 1433, 1720, 1723, 1755, 1900,
    2000, 2001, 2049, 2121, 2717, 3000, 3128, 3306, 3389, 3986, 4899,
    5000, 5009, 5051, 5060, 5101, 5190, 5357, 5432, 5631, 5666, 5800,
    5900, 6000, 6001, 6646, 7070, 8000, 8008, 8009, 8080, 8081, 8443,
    8888, 9100, 9999, 10000, 32768, 49152, 49153, 49154, 49155, 49156,
    49157,
];


pub fn fui(port: u16) -> &'static str {
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
