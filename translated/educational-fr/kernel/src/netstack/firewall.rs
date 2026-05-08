//! Firewall / Packet Filter — iptables-like rule engine
//!
//! Implements INPUT, OUTPUT, and FORWARD chains with configurable rules.
//! Each rule matches on: protocol, source IP, dest IP, source port, dest port.
//! Actions: ACCEPT, DROP, LOG (log + accept), REJECT.
//!
//! Usage via shell:
//!   firewall status              Show chain policies and rules
//!   firewall add INPUT ...       Add rule to chain
//!   firewall del INPUT <n>       Delete rule by index
//!   firewall policy INPUT DROP   Set default policy
//!   firewall flush               Remove all rules
//!   firewall log                 Show recent log entries

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════

/// Firewall chain type
#[derive(Debug, Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum Chain {
    Input,
    Output,
    Forward,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Chain {
        // Fonction publique — appelable depuis d'autres modules.
pub fn name(&self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            Chain::Input => "INPUT",
            Chain::Output => "OUTPUT",
            Chain::Forward => "FORWARD",
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn from_str(s: &str) -> Option<Self> {
                // Correspondance de motifs — branchement exhaustif de Rust.
match s.to_uppercase().as_str() {
            "INPUT" => Some(Chain::Input),
            "OUTPUT" => Some(Chain::Output),
            "FORWARD" => Some(Chain::Forward),
            _ => None,
        }
    }
}

/// Rule action (target)
#[derive(Debug, Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum Action {
    Accept,
    Drop,
    Reject,
    Log, // Log and accept
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Action {
        // Fonction publique — appelable depuis d'autres modules.
pub fn name(&self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            Action::Accept => "ACCEPT",
            Action::Drop => "DROP",
            Action::Reject => "REJECT",
            Action::Log => "LOG",
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn from_str(s: &str) -> Option<Self> {
                // Correspondance de motifs — branchement exhaustif de Rust.
match s.to_uppercase().as_str() {
            "ACCEPT" => Some(Action::Accept),
            "DROP" => Some(Action::Drop),
            "REJECT" => Some(Action::Reject),
            "LOG" => Some(Action::Log),
            _ => None,
        }
    }
}

/// IP protocol matcher
#[derive(Debug, Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum Protocol {
    Any,
    Tcp,
    Udp,
    Icmp,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Protocol {
        // Fonction publique — appelable depuis d'autres modules.
pub fn name(&self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            Protocol::Any => "all",
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
            Protocol::Icmp => "icmp",
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn from_str(s: &str) -> Option<Self> {
                // Correspondance de motifs — branchement exhaustif de Rust.
match s.to_lowercase().as_str() {
            "all" | "any" | "*" => Some(Protocol::Any),
            "tcp" => Some(Protocol::Tcp),
            "udp" => Some(Protocol::Udp),
            "icmp" => Some(Protocol::Icmp),
            _ => None,
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn number(&self) -> Option<u8> {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            Protocol::Any => None,
            Protocol::Icmp => Some(1),
            Protocol::Tcp => Some(6),
            Protocol::Udp => Some(17),
        }
    }
}

/// IP address match (exact, subnet, or any)
#[derive(Debug, Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum IpMatch {
    Any,
    Exact([u8; 4]),
    Subnet([u8; 4], u8), // address + CIDR prefix length
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl IpMatch {
        // Fonction publique — appelable depuis d'autres modules.
pub fn matches(&self, ip: [u8; 4]) -> bool {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            IpMatch::Any => true,
            IpMatch::Exact(addr) => *addr == ip,
            IpMatch::Subnet(addr, prefix) => {
                if *prefix == 0 {
                    return true;
                }
                if *prefix >= 32 {
                    return *addr == ip;
                }
                let mask = !0u32 << (32 - prefix);
                let a = u32::from_be_bytes(*addr) & mask;
                let b = u32::from_be_bytes(ip) & mask;
                a == b
            }
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn parse(s: &str) -> Option<Self> {
        if s == "0.0.0.0/0" || s == "any" || s == "*" {
            return Some(IpMatch::Any);
        }
        if let Some((address_str, prefix_str)) = s.split_once('/') {
            let addr = parse_ipv4(address_str)?;
            let prefix: u8 = prefix_str.parse().ok()?;
            if prefix > 32 {
                return None;
            }
            Some(IpMatch::Subnet(addr, prefix))
        } else {
            let addr = parse_ipv4(s)?;
            Some(IpMatch::Exact(addr))
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn display(&self) -> String {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            IpMatch::Any => String::from("0.0.0.0/0"),
            IpMatch::Exact(a) => format!("{}.{}.{}.{}", a[0], a[1], a[2], a[3]),
            IpMatch::Subnet(a, p) => format!("{}.{}.{}.{}/{}", a[0], a[1], a[2], a[3], p),
        }
    }
}

/// Port match
#[derive(Debug, Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum PortMatch {
    Any,
    Exact(u16),
    Range(u16, u16),
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl PortMatch {
        // Fonction publique — appelable depuis d'autres modules.
pub fn matches(&self, port: u16) -> bool {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            PortMatch::Any => true,
            PortMatch::Exact(p) => *p == port,
            PortMatch::Range(lo, hi) => port >= *lo && port <= *hi,
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn parse(s: &str) -> Option<Self> {
        if s == "any" || s == "*" || s == "0" {
            return Some(PortMatch::Any);
        }
        if let Some((lo_str, hi_str)) = s.split_once(':') {
            let lo: u16 = lo_str.parse().ok()?;
            let hi: u16 = hi_str.parse().ok()?;
            Some(PortMatch::Range(lo, hi))
        } else {
            let p: u16 = s.parse().ok()?;
            Some(PortMatch::Exact(p))
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn display(&self) -> String {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            PortMatch::Any => String::from("*"),
            PortMatch::Exact(p) => format!("{}", p),
            PortMatch::Range(lo, hi) => format!("{}:{}", lo, hi),
        }
    }
}

/// A single firewall rule
#[derive(Debug, Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct Rule {
    pub chain: Chain,
    pub protocol: Protocol,
    pub src_ip: IpMatch,
    pub dst_ip: IpMatch,
    pub src_port: PortMatch,
    pub dst_port: PortMatch,
    pub action: Action,
    pub comment: String,
    /// Number of packets matched
    pub packets: u64,
    /// Number of bytes matched
    pub bytes: u64,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Rule {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new(chain: Chain, action: Action) -> Self {
        Self {
            chain,
            protocol: Protocol::Any,
            src_ip: IpMatch::Any,
            dst_ip: IpMatch::Any,
            src_port: PortMatch::Any,
            dst_port: PortMatch::Any,
            action,
            comment: String::new(),
            packets: 0,
            bytes: 0,
        }
    }

    /// Check if this rule matches a packet
    pub fn matches(&self, proto: u8, src: [u8; 4], dst: [u8; 4], sport: u16, dport: u16) -> bool {
        // Protocol match
        if let Some(p) = self.protocol.number() {
            if p != proto {
                return false;
            }
        }
        // IP match
        if !self.src_ip.matches(src) {
            return false;
        }
        if !self.dst_ip.matches(dst) {
            return false;
        }
        // Port match (only for TCP/UDP)
        if proto == 6 || proto == 17 {
            if !self.src_port.matches(sport) {
                return false;
            }
            if !self.dst_port.matches(dport) {
                return false;
            }
        }
        true
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Firewall State
// ═══════════════════════════════════════════════════════════════════════

struct FirewallState {
    rules: Vec<Rule>,
    input_policy: Action,
    output_policy: Action,
    forward_policy: Action,
    log_entries: Vec<String>,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl FirewallState {
    fn new() -> Self {
        Self {
            rules: Vec::new(),
            input_policy: Action::Accept,
            output_policy: Action::Accept,
            forward_policy: Action::Drop,
            log_entries: Vec::new(),
        }
    }

    fn get_policy(&self, chain: Chain) -> Action {
                // Correspondance de motifs — branchement exhaustif de Rust.
match chain {
            Chain::Input => self.input_policy,
            Chain::Output => self.output_policy,
            Chain::Forward => self.forward_policy,
        }
    }

    fn set_policy(&mut self, chain: Chain, action: Action) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match chain {
            Chain::Input => self.input_policy = action,
            Chain::Output => self.output_policy = action,
            Chain::Forward => self.forward_policy = action,
        }
    }

    fn add_log(&mut self, entry: String) {
        if self.log_entries.len() >= 256 {
            self.log_entries.remove(0);
        }
        self.log_entries.push(entry);
    }
}

// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static FIREWALL: Mutex<FirewallState> = Mutex::new(FirewallState {
    rules: Vec::new(),
    input_policy: Action::Accept,
    output_policy: Action::Accept,
    forward_policy: Action::Drop,
    log_entries: Vec::new(),
});

// Variable atomique — accès thread-safe sans verrou.
static ENABLED: AtomicBool = AtomicBool::new(false);
// Variable atomique — accès thread-safe sans verrou.
static PACKETS_ALLOWED: AtomicU64 = AtomicU64::new(0);
// Variable atomique — accès thread-safe sans verrou.
static PACKETS_DROPPED: AtomicU64 = AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════════════
// Packet Filtering API (called from ip.rs / send path)
// ═══════════════════════════════════════════════════════════════════════

/// Check if the firewall is enabled
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Enable/disable the firewall
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Release);
}

/// Filter an incoming packet. Returns true if ACCEPT, false if DROP.
pub fn filter_input(proto: u8, src: [u8; 4], dst: [u8; 4], sport: u16, dport: u16, packet_length: usize) -> bool {
    if !is_enabled() {
        return true;
    }
    filter_chain(Chain::Input, proto, src, dst, sport, dport, packet_length)
}

/// Filter an outgoing packet. Returns true if ACCEPT, false if DROP.
pub fn filter_output(proto: u8, src: [u8; 4], dst: [u8; 4], sport: u16, dport: u16, packet_length: usize) -> bool {
    if !is_enabled() {
        return true;
    }
    filter_chain(Chain::Output, proto, src, dst, sport, dport, packet_length)
}

/// Core filtering logic for a chain
fn filter_chain(chain: Chain, proto: u8, src: [u8; 4], dst: [u8; 4], sport: u16, dport: u16, packet_length: usize) -> bool {
    let mut fw = FIREWALL.lock();

    // Walk rules in order
    for rule in fw.rules.iter_mut() {
        if rule.chain != chain {
            continue;
        }
        if rule.matches(proto, src, dst, sport, dport) {
            rule.packets += 1;
            rule.bytes += packet_length as u64;

                        // Correspondance de motifs — branchement exhaustif de Rust.
match rule.action {
                Action::Accept => {
                    PACKETS_ALLOWED.fetch_add(1, Ordering::Relaxed);
                    return true;
                }
                Action::Drop => {
                    PACKETS_DROPPED.fetch_add(1, Ordering::Relaxed);
                    return false;
                }
                Action::Reject => {
                    PACKETS_DROPPED.fetch_add(1, Ordering::Relaxed);
                    send_reject(proto, src, dst, sport, dport);
                    return false;
                }
                Action::Log => {
                    let protocol_name = // Correspondance de motifs — branchement exhaustif de Rust.
match proto {
                        1 => "ICMP",
                        6 => "TCP",
                        17 => "UDP",
                        _ => "???",
                    };
                    let entry = format!(
                        "[FW {}] {} {}.{}.{}.{}:{} -> {}.{}.{}.{}:{} len={}",
                        chain.name(), protocol_name,
                        src[0], src[1], src[2], src[3], sport,
                        dst[0], dst[1], dst[2], dst[3], dport,
                        packet_length,
                    );
                    crate::serial_println!("{}", entry);
                    fw.add_log(entry);
                    PACKETS_ALLOWED.fetch_add(1, Ordering::Relaxed);
                    return true; // LOG = log + accept
                }
            }
        }
    }

    // No rule matched — apply default policy
    let policy = fw.get_policy(chain);
        // Correspondance de motifs — branchement exhaustif de Rust.
match policy {
        Action::Accept | Action::Log => {
            PACKETS_ALLOWED.fetch_add(1, Ordering::Relaxed);
            true
        }
        Action::Drop | Action::Reject => {
            PACKETS_DROPPED.fetch_add(1, Ordering::Relaxed);
            false
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Management API
// ═══════════════════════════════════════════════════════════════════════

/// Add a rule to the firewall
pub fn add_rule(rule: Rule) {
    FIREWALL.lock().rules.push(rule);
}

/// Insert a rule at a specific position
pub fn insert_rule(index: usize, rule: Rule) {
    let mut fw = FIREWALL.lock();
    if index <= fw.rules.len() {
        fw.rules.insert(index, rule);
    }
}

/// Delete a rule by index (within a chain)
pub fn delete_rule(chain: Chain, index: usize) -> bool {
    let mut fw = FIREWALL.lock();
    let mut chain_index = 0usize;
    let mut real_index = None;
    for (i, rule) in fw.rules.iter().enumerate() {
        if rule.chain == chain {
            if chain_index == index {
                real_index = Some(i);
                break;
            }
            chain_index += 1;
        }
    }
    if let Some(i) = real_index {
        fw.rules.remove(i);
        true
    } else {
        false
    }
}

/// Flush all rules (optionally for a specific chain)
pub fn flush(chain: Option<Chain>) {
    let mut fw = FIREWALL.lock();
        // Correspondance de motifs — branchement exhaustif de Rust.
match chain {
        Some(c) => fw.rules.retain(|r| r.chain != c),
        None => fw.rules.clear(),
    }
}

/// Set default policy for a chain
pub fn set_policy(chain: Chain, action: Action) {
    FIREWALL.lock().set_policy(chain, action);
}

/// Get all rules for a chain
pub fn list_rules(chain: Chain) -> Vec<Rule> {
    FIREWALL.lock().rules.iter().filter(|r| r.chain == chain).cloned().collect()
}

/// Get chain policy
pub fn get_policy(chain: Chain) -> Action {
    FIREWALL.lock().get_policy(chain)
}

/// Get stats
pub fn stats() -> (u64, u64) {
    (PACKETS_ALLOWED.load(Ordering::Relaxed), PACKETS_DROPPED.load(Ordering::Relaxed))
}

/// Get recent log entries
pub fn get_log() -> Vec<String> {
    FIREWALL.lock().log_entries.clone()
}

/// Clear log entries
pub fn clear_log() {
    FIREWALL.lock().log_entries.clear();
}

/// Reset stats
pub fn reset_stats() {
    PACKETS_ALLOWED.store(0, Ordering::Relaxed);
    PACKETS_DROPPED.store(0, Ordering::Relaxed);
    let mut fw = FIREWALL.lock();
    for rule in fw.rules.iter_mut() {
        rule.packets = 0;
        rule.bytes = 0;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════

fn parse_ipv4(s: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let a: u8 = parts[0].parse().ok()?;
    let b: u8 = parts[1].parse().ok()?;
    let c: u8 = parts[2].parse().ok()?;
    let d: u8 = parts[3].parse().ok()?;
    Some([a, b, c, d])
}

/// Send rejection response: TCP RST for TCP, ICMP Unreachable for others
fn send_reject(proto: u8, src: [u8; 4], dst: [u8; 4], sport: u16, dport: u16) {
    if proto == 6 {
        // TCP RST: swap src/dst, set RST flag, use seq=0 ack=0
        send_tcp_rst(src, dst, sport, dport);
    } else {
        // ICMP Destination Unreachable, Code 13 (Communication Administratively Prohibited)
        send_icmp_unreachable(src, dst);
    }
}

/// Send a TCP RST packet
fn send_tcp_rst(remote_ip: [u8; 4], local_ip: [u8; 4], remote_port: u16, local_port: u16) {
    // TCP header: 20 bytes minimum
    let mut seg = [0u8; 20];
    // Source port (our port)
    seg[0..2].copy_from_slice(&local_port.to_be_bytes());
    // Destination port (remote port)
    seg[2..4].copy_from_slice(&remote_port.to_be_bytes());
    // Sequence number = 0
    // Acknowledgment number = 0
    // Data offset (5 words = 20 bytes) + RST+ACK flags
    seg[12] = 0x50; // data offset = 5 (20 bytes)
    seg[13] = 0x14; // RST(0x04) + ACK(0x10)
    // Window = 0
    // Checksum placeholder
    // Urgent pointer = 0

    // Compute TCP checksum with pseudo-header
    let csum = super::tcp::tcp_checksum_external(local_ip, remote_ip, &seg);
    seg[16..18].copy_from_slice(&csum.to_be_bytes());

    let _ = super::ip::send_packet(remote_ip, 6, &seg);
}

/// Send ICMP Destination Unreachable (Type 3, Code 13 = Admin Prohibited)
fn send_icmp_unreachable(remote_ip: [u8; 4], _local_ip: [u8; 4]) {
    // ICMP Destination Unreachable: 8 bytes header
    let mut pkt = [0u8; 8];
    pkt[0] = 3;  // Type 3 = Destination Unreachable
    pkt[1] = 13; // Code 13 = Communication Administratively Prohibited
    // Checksum at [2..4], computed below
    // Unused/Next-hop MTU at [4..8] = 0

    // Compute ICMP checksum
    let mut sum: u32 = 0;
    for i in (0..pkt.len()).step_by(2) {
        let word = if i + 1 < pkt.len() {
            ((pkt[i] as u16) << 8) | (pkt[i + 1] as u16)
        } else {
            (pkt[i] as u16) << 8
        };
        sum += word as u32;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    let csum = !(sum as u16);
    pkt[2..4].copy_from_slice(&csum.to_be_bytes());

    let _ = super::ip::send_packet(remote_ip, 1, &pkt);
}
