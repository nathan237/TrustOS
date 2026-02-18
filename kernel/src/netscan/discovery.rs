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
pub struct HostInfo {
    pub ip: [u8; 4],
    pub mac: Option<[u8; 6]>,
    pub hostname: Option<String>,
    pub ttl: Option<u8>,
    pub rtt_ms: u64,
    pub os_hint: &'static str,
}

/// Guess OS from TTL value
fn guess_os_from_ttl(ttl: u8) -> &'static str {
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
pub fn arp_sweep(subnet_start: [u8; 4], subnet_end: [u8; 4], timeout_ms: u32) -> Vec<HostInfo> {
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
    loop {
        crate::netstack::poll();

        if crate::logger::get_ticks().saturating_sub(start) > timeout_ms as u64 {
            break;
        }
        spins = spins.wrapping_add(1);
        if spins > 1_000_000 { break; }
        x86_64::instructions::hlt();
    }

    // Phase 3: Read ARP cache for discovered hosts
    let entries = crate::netstack::arp::entries();
    for (ip_u32, mac) in entries {
        let ip = ip_u32.to_be_bytes();
        // Only include IPs in our scan range
        if ip_u32 >= start_u32 && ip_u32 <= end_u32 {
            hosts.push(HostInfo {
                ip,
                mac: Some(mac),
                hostname: None,
                ttl: None,
                rtt_ms: 0,
                os_hint: "Unknown",
            });
        }
    }

    hosts.sort_by_key(|h| u32::from_be_bytes(h.ip));
    hosts
}

/// ARP sweep the local /24 subnet
pub fn arp_sweep_local(timeout_ms: u32) -> Vec<HostInfo> {
    let (our_ip, subnet, _) = match crate::network::get_ipv4_config() {
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

    arp_sweep(net_start, net_end, timeout_ms)
}

/// ICMP ping sweep: discover hosts via ping
///
/// Slower than ARP but works across subnets/routers.
pub fn ping_sweep(targets: &[[u8; 4]], timeout_ms: u32) -> Vec<HostInfo> {
    let mut hosts = Vec::new();

    crate::netstack::icmp::clear_responses();

    for (i, &target) in targets.iter().enumerate() {
        let seq = (i + 1) as u16;
        let start = crate::logger::get_ticks();

        if crate::netstack::icmp::send_echo_request(target, 0x5CA2, seq).is_err() {
            continue;
        }

        match crate::netstack::icmp::wait_for_response(seq, timeout_ms) {
            Some(resp) if resp.success => {
                let rtt_ms = crate::logger::get_ticks().saturating_sub(start);
                hosts.push(HostInfo {
                    ip: target,
                    mac: crate::netstack::arp::resolve(target),
                    hostname: None,
                    ttl: Some(resp.ttl),
                    rtt_ms,
                    os_hint: guess_os_from_ttl(resp.ttl),
                });
            }
            _ => {} // Host down or filtered
        }
    }

    hosts
}

/// Ping sweep a /24 subnet
pub fn ping_sweep_subnet(base_ip: [u8; 4], timeout_per_host_ms: u32) -> Vec<HostInfo> {
    let mut targets = Vec::new();
    for i in 1..=254u8 {
        targets.push([base_ip[0], base_ip[1], base_ip[2], i]);
    }
    ping_sweep(&targets, timeout_per_host_ms)
}

/// Combined discovery: ARP sweep (fast) + Ping sweep (for remaining)
pub fn full_discovery(timeout_ms: u32) -> Vec<HostInfo> {
    // Start with ARP sweep (fast, local only)
    let mut hosts = arp_sweep_local(timeout_ms);

    // Add ping data for each found host (get TTL/OS info)
    for host in &mut hosts {
        crate::netstack::icmp::clear_responses();
        let start = crate::logger::get_ticks();
        if crate::netstack::icmp::send_echo_request(host.ip, 0x5CA1, 1).is_ok() {
            if let Some(resp) = crate::netstack::icmp::wait_for_response(1, 500) {
                host.ttl = Some(resp.ttl);
                host.rtt_ms = crate::logger::get_ticks().saturating_sub(start);
                host.os_hint = guess_os_from_ttl(resp.ttl);
            }
        }
    }

    hosts
}
