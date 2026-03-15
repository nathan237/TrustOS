//! Network Discovery — Host discovery via ARP sweep and ICMP ping sweep
//!
//! - ARP sweep: fast local network scan (broadcast ARP, collect replies)
//! - Ping sweep: ICMP echo to detect live hosts across subnets
//! - OS fingerprinting hints via TTL analysis

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

/// Discovered host information
#[derive(Debug, Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct HostInformation {
    pub ip: [u8; 4],
    pub mac: Option<[u8; 6]>,
    pub hostname: Option<String>,
    pub ttl: Option<u8>,
    pub rtt_mouse: u64,
    pub os_hint: &'static str,
}

/// Guess OS from TTL value
fn guess_os_from_ttl(ttl: u8) -> &'static str {
        // Correspondance de motifs — branchement exhaustif de Rust.
match ttl {
        // TTL values after traversing hops
        t if t <= 32 => "Unknown (low TTL)",
        t if t <= 64 => "Linux/Unix/macOS",
        t if t <= 128 => "Windows",
        _ => "Cisco/Network device",
    }
}

/// ARP sweep: discover all hosts on the local subnet
///
/// Sends ARP requests to every IP in the subnet, then collects replies.
/// This is the fastest method for local network discovery.
pub fn arp_sweep(subnet_start: [u8; 4], subnet_end: [u8; 4], timeout_mouse: u32) -> Vec<HostInformation> {
    let mut hosts = Vec::new();

    let start_u32 = u32::from_be_bytes(subnet_start);
    let end_u32 = u32::from_be_bytes(subnet_end);

    if end_u32 < start_u32 || end_u32 - start_u32 > 1024 {
        return hosts; // Safety limit: max 1024 hosts
    }

    // Phase 1: Send all ARP requests (burst)
    for ip_u32 in start_u32..=end_u32 {
        let ip = ip_u32.to_be_bytes();
        let _ = crate::netstack::arp::send_request(ip);
        // Small delay between packets to avoid flooding
        for _ in 0..10000 { core::hint::spin_loop(); }
    }

    // Phase 2: Poll and collect responses
    let start = crate::logger::get_ticks();
    let mut spins: u32 = 0;
        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        crate::netstack::poll();

        if crate::logger::get_ticks().saturating_sub(start) > timeout_mouse as u64 {
            break;
        }
        spins = spins.wrapping_add(1);
        if spins > 1_000_000 { break; }
        crate::arch::halt();
    }

    // Phase 3: Read ARP cache for discovered hosts
    let entries = crate::netstack::arp::entries();
    for (ip_u32, mac) in entries {
        let ip = ip_u32.to_be_bytes();
        // Only include IPs in our scan range
        if ip_u32 >= start_u32 && ip_u32 <= end_u32 {
            hosts.push(HostInformation {
                ip,
                mac: Some(mac),
                hostname: None,
                ttl: None,
                rtt_mouse: 0,
                os_hint: "Unknown",
            });
        }
    }

    hosts.sort_by_key(|h| u32::from_be_bytes(h.ip));
    hosts
}

/// ARP sweep the local /24 subnet
pub fn arp_sweep_local(timeout_mouse: u32) -> Vec<HostInformation> {
    let (our_ip, subnet, _) = // Correspondance de motifs — branchement exhaustif de Rust.
match crate::network::get_ipv4_config() {
        Some((ip, mask, gw)) => (*ip.as_bytes(), *mask.as_bytes(), gw),
        None => return Vec::new(),
    };

    // Calculate subnet range
    let net_start = [
        our_ip[0] & subnet[0],
        our_ip[1] & subnet[1],
        our_ip[2] & subnet[2],
        (our_ip[3] & subnet[3]).wrapping_add(1), // Skip network address
    ];
    let net_end = [
        our_ip[0] | !subnet[0],
        our_ip[1] | !subnet[1],
        our_ip[2] | !subnet[2],
        (our_ip[3] | !subnet[3]).wrapping_sub(1), // Skip broadcast
    ];

    arp_sweep(net_start, net_end, timeout_mouse)
}

/// ICMP ping sweep: discover hosts via ping
///
/// Slower than ARP but works across subnets/routers.
pub fn ping_sweep(targets: &[[u8; 4]], timeout_mouse: u32) -> Vec<HostInformation> {
    let mut hosts = Vec::new();

    crate::netstack::icmp::clear_responses();

    for (i, &target) in targets.iter().enumerate() {
        let sequence = (i + 1) as u16;
        let start = crate::logger::get_ticks();

        if crate::netstack::icmp::send_echo_request(target, 0x5CA2, sequence).is_err() {
            continue;
        }

                // Correspondance de motifs — branchement exhaustif de Rust.
match crate::netstack::icmp::wait_for_response(sequence, timeout_mouse) {
            Some(response) if response.success => {
                let rtt_mouse = crate::logger::get_ticks().saturating_sub(start);
                hosts.push(HostInformation {
                    ip: target,
                    mac: crate::netstack::arp::resolve(target),
                    hostname: None,
                    ttl: Some(response.ttl),
                    rtt_mouse,
                    os_hint: guess_os_from_ttl(response.ttl),
                });
            }
            _ => {} // Host down or filtered
        }
    }

    hosts
}

/// Ping sweep a /24 subnet
pub fn ping_sweep_subnet(base_ip: [u8; 4], timeout_per_host_mouse: u32) -> Vec<HostInformation> {
    let mut targets = Vec::new();
    for i in 1..=254u8 {
        targets.push([base_ip[0], base_ip[1], base_ip[2], i]);
    }
    ping_sweep(&targets, timeout_per_host_mouse)
}

/// Combined discovery: ARP sweep (fast) + Ping sweep (for remaining)
pub fn full_discovery(timeout_mouse: u32) -> Vec<HostInformation> {
    // Start with ARP sweep (fast, local only)
    let mut hosts = arp_sweep_local(timeout_mouse);

    // Add ping data for each found host (get TTL/OS info)
    for host in &mut hosts {
        crate::netstack::icmp::clear_responses();
        let start = crate::logger::get_ticks();
        if crate::netstack::icmp::send_echo_request(host.ip, 0x5CA1, 1).is_ok() {
            if let Some(response) = crate::netstack::icmp::wait_for_response(1, 500) {
                host.ttl = Some(response.ttl);
                host.rtt_mouse = crate::logger::get_ticks().saturating_sub(start);
                host.os_hint = guess_os_from_ttl(response.ttl);
            }
        }
    }

    hosts
}
