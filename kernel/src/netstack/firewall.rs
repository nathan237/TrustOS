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
pub enum Chain {
    Input,
    Output,
    Forward,
}

impl Chain {
    pub fn name(&self) -> &'static str {
        match self {
            Chain::Input => "INPUT",
            Chain::Output => "OUTPUT",
            Chain::Forward => "FORWARD",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
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
pub enum Action {
    Accept,
    Drop,
    Reject,
    Log, // Log and accept
}

impl Action {
    pub fn name(&self) -> &'static str {
        match self {
            Action::Accept => "ACCEPT",
            Action::Drop => "DROP",
            Action::Reject => "REJECT",
            Action::Log => "LOG",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
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
pub enum Protocol {
    Any,
    Tcp,
    Udp,
    Icmp,
}

impl Protocol {
    pub fn name(&self) -> &'static str {
        match self {
            Protocol::Any => "all",
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
            Protocol::Icmp => "icmp",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "all" | "any" | "*" => Some(Protocol::Any),
            "tcp" => Some(Protocol::Tcp),
            "udp" => Some(Protocol::Udp),
            "icmp" => Some(Protocol::Icmp),
            _ => None,
        }
    }

    pub fn number(&self) -> Option<u8> {
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
pub enum IpMatch {
    Any,
    Exact([u8; 4]),
    Subnet([u8; 4], u8), // address + CIDR prefix length
}

impl IpMatch {
    pub fn matches(&self, ip: [u8; 4]) -> bool {
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

    pub fn parse(s: &str) -> Option<Self> {
        if s == "0.0.0.0/0" || s == "any" || s == "*" {
            return Some(IpMatch::Any);
        }
        if let Some((addr_str, prefix_str)) = s.split_once('/') {
            let addr = parse_ipv4(addr_str)?;
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

    pub fn display(&self) -> String {
        match self {
            IpMatch::Any => String::from("0.0.0.0/0"),
            IpMatch::Exact(a) => format!("{}.{}.{}.{}", a[0], a[1], a[2], a[3]),
            IpMatch::Subnet(a, p) => format!("{}.{}.{}.{}/{}", a[0], a[1], a[2], a[3], p),
        }
    }
}

/// Port match
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortMatch {
    Any,
    Exact(u16),
    Range(u16, u16),
}

impl PortMatch {
    pub fn matches(&self, port: u16) -> bool {
        match self {
            PortMatch::Any => true,
            PortMatch::Exact(p) => *p == port,
            PortMatch::Range(lo, hi) => port >= *lo && port <= *hi,
        }
    }

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

    pub fn display(&self) -> String {
        match self {
            PortMatch::Any => String::from("*"),
            PortMatch::Exact(p) => format!("{}", p),
            PortMatch::Range(lo, hi) => format!("{}:{}", lo, hi),
        }
    }
}

/// A single firewall rule
#[derive(Debug, Clone)]
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

impl Rule {
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
        match chain {
            Chain::Input => self.input_policy,
            Chain::Output => self.output_policy,
            Chain::Forward => self.forward_policy,
        }
    }

    fn set_policy(&mut self, chain: Chain, action: Action) {
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

static FIREWALL: Mutex<FirewallState> = Mutex::new(FirewallState {
    rules: Vec::new(),
    input_policy: Action::Accept,
    output_policy: Action::Accept,
    forward_policy: Action::Drop,
    log_entries: Vec::new(),
});

static ENABLED: AtomicBool = AtomicBool::new(false);
static PACKETS_ALLOWED: AtomicU64 = AtomicU64::new(0);
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
pub fn filter_input(proto: u8, src: [u8; 4], dst: [u8; 4], sport: u16, dport: u16, pkt_len: usize) -> bool {
    if !is_enabled() {
        return true;
    }
    filter_chain(Chain::Input, proto, src, dst, sport, dport, pkt_len)
}

/// Filter an outgoing packet. Returns true if ACCEPT, false if DROP.
pub fn filter_output(proto: u8, src: [u8; 4], dst: [u8; 4], sport: u16, dport: u16, pkt_len: usize) -> bool {
    if !is_enabled() {
        return true;
    }
    filter_chain(Chain::Output, proto, src, dst, sport, dport, pkt_len)
}

/// Core filtering logic for a chain
fn filter_chain(chain: Chain, proto: u8, src: [u8; 4], dst: [u8; 4], sport: u16, dport: u16, pkt_len: usize) -> bool {
    let mut fw = FIREWALL.lock();

    // Walk rules in order
    for rule in fw.rules.iter_mut() {
        if rule.chain != chain {
            continue;
        }
        if rule.matches(proto, src, dst, sport, dport) {
            rule.packets += 1;
            rule.bytes += pkt_len as u64;

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
                    // TODO: send ICMP unreachable or TCP RST
                    return false;
                }
                Action::Log => {
                    let proto_name = match proto {
                        1 => "ICMP",
                        6 => "TCP",
                        17 => "UDP",
                        _ => "???",
                    };
                    let entry = format!(
                        "[FW {}] {} {}.{}.{}.{}:{} -> {}.{}.{}.{}:{} len={}",
                        chain.name(), proto_name,
                        src[0], src[1], src[2], src[3], sport,
                        dst[0], dst[1], dst[2], dst[3], dport,
                        pkt_len,
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
    let mut chain_idx = 0usize;
    let mut real_idx = None;
    for (i, rule) in fw.rules.iter().enumerate() {
        if rule.chain == chain {
            if chain_idx == index {
                real_idx = Some(i);
                break;
            }
            chain_idx += 1;
        }
    }
    if let Some(i) = real_idx {
        fw.rules.remove(i);
        true
    } else {
        false
    }
}

/// Flush all rules (optionally for a specific chain)
pub fn flush(chain: Option<Chain>) {
    let mut fw = FIREWALL.lock();
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
