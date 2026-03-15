//! UDP Protocol

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU16, Ordering};
use spin::Mutex;

/// Received UDP datagram with source address info
struct UdpDatagram {
    source_ip: [u8; 4],
    source_port: u16,
    data: Vec<u8>,
}

// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static RECEIVE_UDP: Mutex<BTreeMap<u16, Vec<UdpDatagram>>> = Mutex::new(BTreeMap::new());
// Variable atomique — accès thread-safe sans verrou.
static NEXT_UDP_PORT: AtomicU16 = AtomicU16::new(49152);

// Fonction publique — appelable depuis d'autres modules.
pub fn handle_packet(data: &[u8], source_ip: [u8; 4]) {
    if data.len() < 8 {
        return;
    }

    let source_port = u16::from_be_bytes([data[0], data[1]]);
    let destination_port = u16::from_be_bytes([data[2], data[3]]);
    let payload = &data[8..];

    // DHCP client (port 68)
    if destination_port == 68 {
        crate::netstack::dhcp::handle_packet(payload);
        return;
    }

    // DHCP server (port 67) — PXE boot requests
    if destination_port == 67 {
        crate::netstack::dhcpd::handle_packet(payload);
        return;
    }

    // TFTP server (port 69) — initial read requests
    if destination_port == 69 {
        crate::netstack::tftpd::handle_request_packet(payload, source_ip, source_port);
        return;
    }

    // TFTP transfer sessions (ephemeral ports 50000+)
    if crate::netstack::tftpd::is_transfer_port(destination_port) {
        crate::netstack::tftpd::handle_transfer_packet(payload, source_ip, source_port, destination_port);
        return;
    }

    let mut receive = RECEIVE_UDP.lock();
    let queue = receive.entry(destination_port).or_insert_with(Vec::new);
    if queue.len() < 32 {
        queue.push(UdpDatagram {
            source_ip,
            source_port,
            data: payload.to_vec(),
        });
    }
}

// Fonction publique — appelable depuis d'autres modules.
pub fn allocator_ephemeral_port() -> u16 {
    NEXT_UDP_PORT.fetch_add(1, Ordering::Relaxed)
}

// Fonction publique — appelable depuis d'autres modules.
pub fn send_to(dest_ip: [u8; 4], dest_port: u16, source_port: u16, payload: &[u8]) -> Result<(), &'static str> {
    let length = 8 + payload.len();
    let mut segment = Vec::with_capacity(length);
    segment.extend_from_slice(&source_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&(length as u16).to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes()); // checksum (optional)
    segment.extend_from_slice(payload);

    crate::netstack::ip::send_packet(dest_ip, 17, &segment)
}

// Fonction publique — appelable depuis d'autres modules.
pub fn recv_on(port: u16) -> Option<Vec<u8>> {
    let mut receive = RECEIVE_UDP.lock();
    let queue = receive.get_mut(&port)?;
    if queue.is_empty() {
        None
    } else {
        Some(queue.remove(0).data)
    }
}

/// Receive a UDP datagram with source address info (for recvfrom)
pub fn recv_from(port: u16) -> Option<(Vec<u8>, [u8; 4], u16)> {
    let mut receive = RECEIVE_UDP.lock();
    let queue = receive.get_mut(&port)?;
    if queue.is_empty() {
        None
    } else {
        let dgram = queue.remove(0);
        Some((dgram.data, dgram.source_ip, dgram.source_port))
    }
}
