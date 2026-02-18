//! Real TTL-based Traceroute
//!
//! Sends ICMP echo requests with incrementing TTL values.
//! Intermediate routers respond with ICMP Time Exceeded messages.
//! The final destination responds with ICMP Echo Reply.

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU16, Ordering};

static TRACE_SEQ: AtomicU16 = AtomicU16::new(1000);

/// A single traceroute hop result
#[derive(Debug, Clone)]
pub struct TraceHop {
    pub hop_num: u8,
    pub ip: Option<[u8; 4]>,
    pub hostname: Option<String>,
    pub rtt_ms: [u64; 3],   // 3 probes per hop
    pub reached: bool,      // true if this is the destination
}

/// Traceroute configuration
pub struct TraceConfig {
    pub max_hops: u8,
    pub probes_per_hop: u8,
    pub timeout_ms: u32,
    pub target: [u8; 4],
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            max_hops: 30,
            probes_per_hop: 3,
            timeout_ms: 2000,
            target: [0; 4],
        }
    }
}

/// Run a traceroute to the target
pub fn trace(target: [u8; 4], max_hops: u8, timeout_ms: u32) -> Vec<TraceHop> {
    let mut hops = Vec::new();

    crate::netstack::icmp::clear_responses();
    crate::netstack::icmp::clear_errors();

    for ttl in 1..=max_hops {
        let mut hop = TraceHop {
            hop_num: ttl,
            ip: None,
            hostname: None,
            rtt_ms: [0; 3],
            reached: false,
        };

        let mut got_response = false;

        for probe in 0..3 {
            let seq = TRACE_SEQ.fetch_add(1, Ordering::Relaxed);

            crate::netstack::icmp::clear_responses();
            crate::netstack::icmp::clear_errors();

            // Build and send ICMP echo request with specific TTL
            let mut icmp_packet = Vec::new();
            icmp_packet.push(8); // Echo Request
            icmp_packet.push(0); // Code
            icmp_packet.push(0); icmp_packet.push(0); // Checksum (fill later)
            icmp_packet.extend_from_slice(&0x5CA1u16.to_be_bytes()); // ID
            icmp_packet.extend_from_slice(&seq.to_be_bytes());

            // Payload: timestamp
            let ts = crate::time::uptime_ms() as u32;
            icmp_packet.extend_from_slice(&ts.to_be_bytes());
            for i in 0..20 {
                icmp_packet.push((0x41 + i) as u8);
            }

            // Calculate ICMP checksum
            let csum = icmp_checksum(&icmp_packet);
            icmp_packet[2] = (csum >> 8) as u8;
            icmp_packet[3] = (csum & 0xFF) as u8;

            let start = crate::logger::get_ticks();

            // Send via IP with custom TTL
            if crate::netstack::ip::send_packet_with_ttl(target, 1, &icmp_packet, ttl).is_err() {
                hop.rtt_ms[probe] = 0;
                continue;
            }

            // Wait for either echo reply or time exceeded
            let result = crate::netstack::icmp::wait_for_response_or_error(seq, target, timeout_ms);
            let elapsed = crate::logger::get_ticks().saturating_sub(start);

            match result {
                crate::netstack::icmp::TracerouteResult::Reached { ip, .. } => {
                    hop.ip = Some(ip);
                    hop.rtt_ms[probe] = elapsed;
                    hop.reached = true;
                    got_response = true;
                }
                crate::netstack::icmp::TracerouteResult::Hop { ip, rtt_ms, .. } => {
                    hop.ip = Some(ip);
                    hop.rtt_ms[probe] = rtt_ms;
                    got_response = true;
                }
                crate::netstack::icmp::TracerouteResult::Timeout => {
                    hop.rtt_ms[probe] = 0; // Will display as *
                }
            }
        }

        hops.push(hop.clone());

        if hop.reached {
            break;
        }
    }

    hops
}

/// ICMP checksum
fn icmp_checksum(data: &[u8]) -> u16 {
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

/// Format traceroute output as string
pub fn format_trace(hops: &[TraceHop]) -> String {
    let mut output = String::new();
    for hop in hops {
        output.push_str(&format!("{:>2}  ", hop.hop_num));

        if let Some(ip) = hop.ip {
            output.push_str(&super::format_ip(ip));
            output.push_str("  ");
            for &rtt in &hop.rtt_ms {
                if rtt == 0 {
                    output.push_str("*  ");
                } else if rtt < 1 {
                    output.push_str("<1 ms  ");
                } else {
                    output.push_str(&format!("{} ms  ", rtt));
                }
            }
        } else {
            output.push_str("* * *");
        }

        output.push('\n');
    }
    output
}
