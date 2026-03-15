//! Minimal DNS resolver (A records only) with caching

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU16, Ordering};
use spin::Mutex;

// Variable atomique — accès thread-safe sans verrou.
static NEXT_ID: AtomicU16 = AtomicU16::new(1);

/// DNS cache entry
struct DnsCacheEntry {
    ip: [u8; 4],
    expire_tick: u64,
}

/// Simple DNS cache — avoids repeated network round-trips for the same host
static DNS_CACHE: Mutex<BTreeMap<String, DnsCacheEntry>> = Mutex::new(BTreeMap::new());

/// Default cache TTL in ticks (~60 seconds at 1ms/tick)
const DNS_CACHE_TTL: u64 = 60_000;

fn write_qname(buffer: &mut Vec<u8>, name: &str) -> bool {
    for label in name.split('.') {
        let len = label.len();
        if len == 0 || len > 63 {
            return false;
        }
        buffer.push(len as u8);
        buffer.extend_from_slice(label.as_bytes());
    }
    buffer.push(0);
    true
}

fn skip_name(data: &[u8], mut index: usize) -> Option<usize> {
        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        if index >= data.len() {
            return None;
        }
        let len = data[index];
        if len & 0xC0 == 0xC0 {
            if index + 1 >= data.len() {
                return None;
            }
            return Some(index + 2);
        }
        if len == 0 {
            return Some(index + 1);
        }
        index += 1 + len as usize;
    }
}

// Fonction publique — appelable depuis d'autres modules.
pub fn resolve(name: &str) -> Option<[u8; 4]> {
    // Check cache first
    {
        let now = crate::logger::get_ticks();
        let cache = DNS_CACHE.lock();
        if let Some(entry) = cache.get(name) {
            if now < entry.expire_tick {
                return Some(entry.ip);
            }
        }
    }

    // Use DHCP-provided DNS, or platform-detected default
    let dns_server = crate::network::get_dns_server();
    let source_port = crate::netstack::udp::allocator_ephemeral_port();
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

    let mut query = Vec::with_capacity(64);
    query.extend_from_slice(&id.to_be_bytes());
    query.extend_from_slice(&0x0100u16.to_be_bytes()); // recursion desired
    query.extend_from_slice(&1u16.to_be_bytes()); // QDCOUNT
    query.extend_from_slice(&0u16.to_be_bytes()); // ANCOUNT
    query.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
    query.extend_from_slice(&0u16.to_be_bytes()); // ARCOUNT
    if !write_qname(&mut query, name) {
        return None;
    }
    query.extend_from_slice(&1u16.to_be_bytes()); // QTYPE=A
    query.extend_from_slice(&1u16.to_be_bytes()); // QCLASS=IN

    let _ = crate::netstack::udp::send_to(dns_server, 53, source_port, &query);

    let start = crate::logger::get_ticks();
        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        crate::netstack::poll();
        if let Some(response) = crate::netstack::udp::recv_on(source_port) {
            if response.len() < 12 {
                continue;
            }
            if u16::from_be_bytes([response[0], response[1]]) != id {
                continue;
            }
            let flags = u16::from_be_bytes([response[2], response[3]]);
            if (flags & 0x8000) == 0 {
                continue;
            }
            let qd = u16::from_be_bytes([response[4], response[5]]) as usize;
            let an = u16::from_be_bytes([response[6], response[7]]) as usize;

            let mut index = 12;
            for _ in 0..qd {
                index = skip_name(&response, index)?;
                if index + 4 > response.len() {
                    return None;
                }
                index += 4; // QTYPE + QCLASS
            }

            for _ in 0..an {
                index = skip_name(&response, index)?;
                if index + 10 > response.len() {
                    return None;
                }
                let rtype = u16::from_be_bytes([response[index], response[index + 1]]);
                let rclass = u16::from_be_bytes([response[index + 2], response[index + 3]]);
                let rdlen = u16::from_be_bytes([response[index + 8], response[index + 9]]) as usize;
                index += 10;
                if index + rdlen > response.len() {
                    return None;
                }
                if rtype == 1 && rclass == 1 && rdlen == 4 {
                    let ip = [response[index], response[index + 1], response[index + 2], response[index + 3]];
                    // Cache the result
                    let expire = crate::logger::get_ticks() + DNS_CACHE_TTL;
                    DNS_CACHE.lock().insert(
                        String::from(name),
                        DnsCacheEntry { ip, expire_tick: expire },
                    );
                    return Some(ip);
                }
                index += rdlen;
            }
        }

        if crate::logger::get_ticks().saturating_sub(start) > 1500 {
            return None;
        }
        core::hint::spin_loop();
    }
}
