//! UDP Protocol

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU16, Ordering};
use spin::Mutex;

static RX_UDP: Mutex<BTreeMap<u16, Vec<Vec<u8>>>> = Mutex::new(BTreeMap::new());
static NEXT_UDP_PORT: AtomicU16 = AtomicU16::new(49152);

pub fn handle_packet(data: &[u8]) {
    if data.len() < 8 {
        return;
    }

    let _src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let payload = &data[8..];

    // DHCP uses ports 67/68
    if dst_port == 68 {
        crate::netstack::dhcp::handle_packet(payload);
        return;
    }

    let mut rx = RX_UDP.lock();
    let queue = rx.entry(dst_port).or_insert_with(Vec::new);
    if queue.len() < 32 {
        queue.push(payload.to_vec());
    }
}

pub fn alloc_ephemeral_port() -> u16 {
    NEXT_UDP_PORT.fetch_add(1, Ordering::Relaxed)
}

pub fn send_to(dest_ip: [u8; 4], dest_port: u16, src_port: u16, payload: &[u8]) -> Result<(), &'static str> {
    let length = 8 + payload.len();
    let mut segment = Vec::with_capacity(length);
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&(length as u16).to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes()); // checksum (optional)
    segment.extend_from_slice(payload);

    crate::netstack::ip::send_packet(dest_ip, 17, &segment)
}

pub fn recv_on(port: u16) -> Option<Vec<u8>> {
    let mut rx = RX_UDP.lock();
    let queue = rx.get_mut(&port)?;
    if queue.is_empty() {
        None
    } else {
        Some(queue.remove(0))
    }
}
