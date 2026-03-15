//! UDP Protocol

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU16, Ordering};
use spin::Mutex;

/// Received UDP datagram with source address info
struct UdpDatagram {
    src_ip: [u8; 4],
    src_port: u16,
    data: Vec<u8>,
}

static RX_UDP: Mutex<BTreeMap<u16, Vec<UdpDatagram>>> = Mutex::new(BTreeMap::new());
static NEXT_UDP_PORT: AtomicU16 = AtomicU16::new(49152);

pub fn handle_packet(data: &[u8], src_ip: [u8; 4]) {
    if data.len() < 8 {
        return;
    }

    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let payload = &data[8..];

    // DHCP client (port 68)
    if dst_port == 68 {
        crate::netstack::dhcp::handle_packet(payload);
        return;
    }

    // DHCP server (port 67) — PXE boot requests
    if dst_port == 67 {
        crate::netstack::dhcpd::handle_packet(payload);
        return;
    }

    // TFTP server (port 69) — initial read requests
    if dst_port == 69 {
        crate::netstack::tftpd::handle_request_packet(payload, src_ip, src_port);
        return;
    }

    // TFTP transfer sessions (ephemeral ports 50000+)
    if crate::netstack::tftpd::is_transfer_port(dst_port) {
        crate::netstack::tftpd::handle_transfer_packet(payload, src_ip, src_port, dst_port);
        return;
    }

    let mut rx = RX_UDP.lock();
    let queue = rx.entry(dst_port).or_insert_with(Vec::new);
    if queue.len() < 32 {
        queue.push(UdpDatagram {
            src_ip,
            src_port,
            data: payload.to_vec(),
        });
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
        Some(queue.remove(0).data)
    }
}

/// Receive a UDP datagram with source address info (for recvfrom)
pub fn recv_from(port: u16) -> Option<(Vec<u8>, [u8; 4], u16)> {
    let mut rx = RX_UDP.lock();
    let queue = rx.get_mut(&port)?;
    if queue.is_empty() {
        None
    } else {
        let dgram = queue.remove(0);
        Some((dgram.data, dgram.src_ip, dgram.src_port))
    }
}
