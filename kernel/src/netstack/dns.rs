//! Minimal DNS resolver (A records only)

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU16, Ordering};

static NEXT_ID: AtomicU16 = AtomicU16::new(1);

fn write_qname(buf: &mut Vec<u8>, name: &str) -> bool {
    for label in name.split('.') {
        let len = label.len();
        if len == 0 || len > 63 {
            return false;
        }
        buf.push(len as u8);
        buf.extend_from_slice(label.as_bytes());
    }
    buf.push(0);
    true
}

fn skip_name(data: &[u8], mut idx: usize) -> Option<usize> {
    loop {
        if idx >= data.len() {
            return None;
        }
        let len = data[idx];
        if len & 0xC0 == 0xC0 {
            if idx + 1 >= data.len() {
                return None;
            }
            return Some(idx + 2);
        }
        if len == 0 {
            return Some(idx + 1);
        }
        idx += 1 + len as usize;
    }
}

pub fn resolve(name: &str) -> Option<[u8; 4]> {
    let dns_server = [8, 8, 8, 8];
    let src_port = crate::netstack::udp::alloc_ephemeral_port();
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

    let _ = crate::netstack::udp::send_to(dns_server, 53, src_port, &query);

    let start = crate::logger::get_ticks();
    loop {
        crate::netstack::poll();
        if let Some(resp) = crate::netstack::udp::recv_on(src_port) {
            if resp.len() < 12 {
                continue;
            }
            if u16::from_be_bytes([resp[0], resp[1]]) != id {
                continue;
            }
            let flags = u16::from_be_bytes([resp[2], resp[3]]);
            if (flags & 0x8000) == 0 {
                continue;
            }
            let qd = u16::from_be_bytes([resp[4], resp[5]]) as usize;
            let an = u16::from_be_bytes([resp[6], resp[7]]) as usize;

            let mut idx = 12;
            for _ in 0..qd {
                idx = skip_name(&resp, idx)?;
                if idx + 4 > resp.len() {
                    return None;
                }
                idx += 4; // QTYPE + QCLASS
            }

            for _ in 0..an {
                idx = skip_name(&resp, idx)?;
                if idx + 10 > resp.len() {
                    return None;
                }
                let rtype = u16::from_be_bytes([resp[idx], resp[idx + 1]]);
                let rclass = u16::from_be_bytes([resp[idx + 2], resp[idx + 3]]);
                let rdlen = u16::from_be_bytes([resp[idx + 8], resp[idx + 9]]) as usize;
                idx += 10;
                if idx + rdlen > resp.len() {
                    return None;
                }
                if rtype == 1 && rclass == 1 && rdlen == 4 {
                    return Some([resp[idx], resp[idx + 1], resp[idx + 2], resp[idx + 3]]);
                }
                idx += rdlen;
            }
        }

        if crate::logger::get_ticks().saturating_sub(start) > 1500 {
            return None;
        }
        core::hint::spin_loop();
    }
}
