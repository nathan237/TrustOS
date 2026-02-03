//! TCP Protocol (minimal scaffolding)

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, Ordering};
use spin::Mutex;

/// TCP flags
pub mod flags {
    pub const FIN: u8 = 0x01;
    pub const SYN: u8 = 0x02;
    pub const RST: u8 = 0x04;
    pub const PSH: u8 = 0x08;
    pub const ACK: u8 = 0x10;
    pub const URG: u8 = 0x20;
}

/// TCP connection state (minimal)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    SynSent,
    Established,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ConnectionId {
    src_ip: u32,
    dst_ip: u32,
    src_port: u16,
    dst_port: u16,
}

#[derive(Debug, Clone, Copy)]
struct TcpConnection {
    state: TcpState,
    seq: u32, // next seq to send
    ack: u32, // next ack to send
    fin_received: bool,
    fin_sent: bool,
    // Delayed ACK: count packets since last ACK
    pending_acks: u8,
    last_ack_time: u64,
}

static CONNECTIONS: Mutex<BTreeMap<ConnectionId, TcpConnection>> = Mutex::new(BTreeMap::new());
static RX_DATA: Mutex<BTreeMap<ConnectionId, Vec<Vec<u8>>>> = Mutex::new(BTreeMap::new());
static NEXT_EPHEMERAL_PORT: AtomicU16 = AtomicU16::new(49152);
static NEXT_SEQ: AtomicU32 = AtomicU32::new(1);

/// Performance: batch ACK threshold (send ACK every N packets or after timeout)
const DELAYED_ACK_PACKETS: u8 = 4;
const DELAYED_ACK_MS: u64 = 20;

/// TCP Window size (larger = faster downloads)
const TCP_WINDOW_SIZE: u16 = 65535;

fn ip_to_u32(ip: [u8; 4]) -> u32 {
    ((ip[0] as u32) << 24) | ((ip[1] as u32) << 16) | ((ip[2] as u32) << 8) | (ip[3] as u32)
}

fn checksum(data: &[u8]) -> u16 {
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

fn tcp_checksum(src_ip: [u8; 4], dst_ip: [u8; 4], segment: &[u8]) -> u16 {
    let mut pseudo = Vec::with_capacity(12 + segment.len());
    pseudo.extend_from_slice(&src_ip);
    pseudo.extend_from_slice(&dst_ip);
    pseudo.push(0);
    pseudo.push(6); // TCP protocol
    pseudo.extend_from_slice(&(segment.len() as u16).to_be_bytes());
    pseudo.extend_from_slice(segment);
    checksum(&pseudo)
}

fn get_source_ip() -> [u8; 4] {
    crate::network::get_ipv4_config()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([10, 0, 2, 15])
}

/// Handle incoming TCP packet (optimized with delayed ACK)
pub fn handle_packet(data: &[u8], src_ip: [u8; 4], dst_ip: [u8; 4]) {
    if data.len() < 20 {
        return;
    }

    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let seq = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let ack = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    let data_offset = data[12] >> 4;
    let flags = data[13];
    let header_len = (data_offset as usize) * 4;

    if data.len() < header_len {
        return;
    }

    let conn_id = ConnectionId {
        src_ip: ip_to_u32(dst_ip),
        dst_ip: ip_to_u32(src_ip),
        src_port: dst_port,
        dst_port: src_port,
    };

    let mut conns = CONNECTIONS.lock();
    let conn = match conns.get_mut(&conn_id) {
        Some(c) => c,
        None => return,
    };

    if conn.fin_received && conn.fin_sent {
        return;
    }

    // Handle SYN-ACK (connection establishment) - always ACK immediately
    if (flags & flags::SYN) != 0 && (flags & flags::ACK) != 0 {
        conn.state = TcpState::Established;
        conn.ack = seq.wrapping_add(1);
        conn.pending_acks = 0;
        conn.last_ack_time = crate::logger::get_ticks();
        crate::serial_println!("[TCP] Connection established");
        drop(conns);
        let _ = send_ack(src_ip, src_port, dst_port);
        return;
    }

    let payload = &data[header_len..];
    let fin = (flags & flags::FIN) != 0;
    let psh = (flags & flags::PSH) != 0;
    let payload_len = payload.len() as u32;

    if fin && payload_len == 0 && conn.fin_received && seq.wrapping_add(1) <= conn.ack {
        return;
    }

    if !payload.is_empty() || fin {
        let fin_inc = if fin { 1 } else { 0 };
        let new_ack = seq.wrapping_add(payload_len).wrapping_add(fin_inc);
        let should_send_fin = fin && !conn.fin_sent;

        if new_ack > conn.ack {
            if !payload.is_empty() {
                // Store payload in RX buffer
                let mut rx = RX_DATA.lock();
                rx.entry(conn_id).or_insert_with(Vec::new).push(payload.to_vec());
                drop(rx);
            }

            conn.ack = new_ack;
            conn.pending_acks = conn.pending_acks.saturating_add(1);
        }

        if fin {
            conn.fin_received = true;
        }

        // Delayed ACK: only send ACK when:
        // 1. FIN received (must ACK immediately)
        // 2. PSH flag set (sender wants immediate response)
        // 3. Accumulated enough packets
        // 4. Timeout exceeded
        let now = crate::logger::get_ticks();
        let should_ack = fin 
            || psh 
            || conn.pending_acks >= DELAYED_ACK_PACKETS
            || now.saturating_sub(conn.last_ack_time) >= DELAYED_ACK_MS;

        if should_ack {
            conn.pending_acks = 0;
            conn.last_ack_time = now;
            drop(conns);
            let _ = send_ack(src_ip, src_port, dst_port);
            if should_send_fin {
                let _ = send_fin(src_ip, src_port, dst_port);
            }
        } else {
            drop(conns);
        }
        if should_send_fin {
            let _ = send_fin(src_ip, src_port, dst_port);
        }

    }
}

/// Send a TCP SYN (start of connection)
pub fn send_syn(dest_ip: [u8; 4], dest_port: u16) -> Result<u16, &'static str> {
    let src_ip = get_source_ip();
    crate::serial_println!("[TCP] src_ip={}.{}.{}.{} dest_ip={}.{}.{}.{}",
        src_ip[0], src_ip[1], src_ip[2], src_ip[3],
        dest_ip[0], dest_ip[1], dest_ip[2], dest_ip[3]);
    let src_port = NEXT_EPHEMERAL_PORT.fetch_add(1, Ordering::Relaxed);
    let seq = NEXT_SEQ.fetch_add(1, Ordering::Relaxed);

    let mut segment = Vec::with_capacity(20);
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&seq.to_be_bytes());
    segment.extend_from_slice(&0u32.to_be_bytes()); // ack
    segment.push(0x50); // data offset=5, no options
    segment.push(flags::SYN);
    segment.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes()); // max window size
    segment.extend_from_slice(&0u16.to_be_bytes()); // checksum (fill later)
    segment.extend_from_slice(&0u16.to_be_bytes()); // urgent pointer

    let csum = tcp_checksum(src_ip, dest_ip, &segment);
    segment[16] = (csum >> 8) as u8;
    segment[17] = (csum & 0xFF) as u8;
    
    // Debug: print the full TCP segment
    crate::serial_println!("[TCP] SYN segment: {:02x}{:02x} {:02x}{:02x} {:02x}{:02x}{:02x}{:02x} {:02x}{:02x}{:02x}{:02x} {:02x}{:02x} {:02x}{:02x}{:02x}{:02x} {:02x}{:02x} csum=0x{:04x}",
        segment[0], segment[1], segment[2], segment[3],
        segment[4], segment[5], segment[6], segment[7],
        segment[8], segment[9], segment[10], segment[11],
        segment[12], segment[13], segment[14], segment[15],
        segment[16], segment[17], segment[18], segment[19], csum);

    let conn_id = ConnectionId {
        src_ip: ip_to_u32(src_ip),
        dst_ip: ip_to_u32(dest_ip),
        src_port,
        dst_port: dest_port,
    };
    CONNECTIONS.lock().insert(conn_id, TcpConnection {
        state: TcpState::SynSent,
        seq: seq.wrapping_add(1),
        ack: 0,
        fin_received: false,
        fin_sent: false,
        pending_acks: 0,
        last_ack_time: crate::logger::get_ticks(),
    });

    crate::serial_println!(
        "[TCP] SYN -> {}.{}.{}.{}:{}",
        dest_ip[0], dest_ip[1], dest_ip[2], dest_ip[3], dest_port
    );

    crate::netstack::ip::send_packet(dest_ip, 6, &segment)?;
    Ok(src_port)
}

/// Send ACK for an established connection
pub fn send_ack(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> Result<(), &'static str> {
    let src_ip = get_source_ip();
    let conn_id = ConnectionId {
        src_ip: ip_to_u32(src_ip),
        dst_ip: ip_to_u32(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    let (seq, ack) = {
        let conns = CONNECTIONS.lock();
        let conn = conns.get(&conn_id).ok_or("Connection not found")?;
        (conn.seq, conn.ack)
    };

    let mut segment = Vec::with_capacity(20);
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&seq.to_be_bytes());
    segment.extend_from_slice(&ack.to_be_bytes());
    segment.push(0x50);
    segment.push(flags::ACK);
    segment.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());

    let csum = tcp_checksum(src_ip, dest_ip, &segment);
    segment[16] = (csum >> 8) as u8;
    segment[17] = (csum & 0xFF) as u8;

    crate::netstack::ip::send_packet(dest_ip, 6, &segment)
}

/// Send TCP payload with PSH|ACK
pub fn send_payload(dest_ip: [u8; 4], dest_port: u16, src_port: u16, payload: &[u8]) -> Result<(), &'static str> {
    let src_ip = get_source_ip();
    let conn_id = ConnectionId {
        src_ip: ip_to_u32(src_ip),
        dst_ip: ip_to_u32(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    let (seq, ack) = {
        let conns = CONNECTIONS.lock();
        let conn = conns.get(&conn_id).ok_or("Connection not found")?;
        (conn.seq, conn.ack)
    };

    let mut segment = Vec::with_capacity(20 + payload.len());
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&seq.to_be_bytes());
    segment.extend_from_slice(&ack.to_be_bytes());
    segment.push(0x50);
    segment.push(flags::PSH | flags::ACK);
    segment.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(payload);

    let csum = tcp_checksum(src_ip, dest_ip, &segment);
    segment[16] = (csum >> 8) as u8;
    segment[17] = (csum & 0xFF) as u8;

    crate::netstack::ip::send_packet(dest_ip, 6, &segment)?;

    let mut conns = CONNECTIONS.lock();
    if let Some(conn) = conns.get_mut(&conn_id) {
        conn.seq = conn.seq.wrapping_add(payload.len() as u32);
    }
    Ok(())
}

/// Send TCP FIN (close connection)
pub fn send_fin(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> Result<(), &'static str> {
    let src_ip = get_source_ip();
    let conn_id = ConnectionId {
        src_ip: ip_to_u32(src_ip),
        dst_ip: ip_to_u32(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    let (seq, ack, already_sent) = {
        let conns = CONNECTIONS.lock();
        let conn = conns.get(&conn_id).ok_or("Connection not found")?;
        (conn.seq, conn.ack, conn.fin_sent)
    };

    if already_sent {
        return Ok(());
    }

    let mut segment = Vec::with_capacity(20);
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&seq.to_be_bytes());
    segment.extend_from_slice(&ack.to_be_bytes());
    segment.push(0x50);
    segment.push(flags::FIN | flags::ACK);
    segment.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());

    let csum = tcp_checksum(src_ip, dest_ip, &segment);
    segment[16] = (csum >> 8) as u8;
    segment[17] = (csum & 0xFF) as u8;

    crate::netstack::ip::send_packet(dest_ip, 6, &segment)?;

    let mut conns = CONNECTIONS.lock();
    if let Some(conn) = conns.get_mut(&conn_id) {
        conn.seq = conn.seq.wrapping_add(1);
        conn.fin_sent = true;
    }
    Ok(())
}

/// Flush any pending ACKs for a connection (call periodically during downloads)
pub fn flush_pending_acks(dest_ip: [u8; 4], dest_port: u16, src_port: u16) {
    let src_ip = get_source_ip();
    let conn_id = ConnectionId {
        src_ip: ip_to_u32(src_ip),
        dst_ip: ip_to_u32(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    let should_ack = {
        let mut conns = CONNECTIONS.lock();
        if let Some(conn) = conns.get_mut(&conn_id) {
            if conn.pending_acks > 0 {
                conn.pending_acks = 0;
                conn.last_ack_time = crate::logger::get_ticks();
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if should_ack {
        let _ = send_ack(dest_ip, dest_port, src_port);
    }
}

/// Receive buffered TCP payloads for a connection (optimized batch receive)
pub fn recv_data(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> Option<Vec<u8>> {
    let src_ip = get_source_ip();
    let conn_id = ConnectionId {
        src_ip: ip_to_u32(src_ip),
        dst_ip: ip_to_u32(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    {
        let mut rx = RX_DATA.lock();
        if let Some(queue) = rx.get_mut(&conn_id) {
            if !queue.is_empty() {
                return Some(queue.remove(0));
            }
        }
    }

    let mut conns = CONNECTIONS.lock();
    if let Some(conn) = conns.get(&conn_id) {
        if conn.fin_received && conn.fin_sent {
            conns.remove(&conn_id);
            RX_DATA.lock().remove(&conn_id);
        }
    }
    None
}

/// Check if FIN has been received for a connection
pub fn fin_received(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> bool {
    let src_ip = get_source_ip();
    let conn_id = ConnectionId {
        src_ip: ip_to_u32(src_ip),
        dst_ip: ip_to_u32(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    CONNECTIONS
        .lock()
        .get(&conn_id)
        .map(|c| c.fin_received)
        .unwrap_or(true)
}

/// Wait for a SYN-ACK (connection established)
pub fn wait_for_established(dest_ip: [u8; 4], dest_port: u16, src_port: u16, timeout_ms: u32) -> bool {
    let src_ip = get_source_ip();
    let conn_id = ConnectionId {
        src_ip: ip_to_u32(src_ip),
        dst_ip: ip_to_u32(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    let start = crate::logger::get_ticks();
    let mut spins: u32 = 0;
    loop {
        crate::netstack::poll();

        if let Some(conn) = CONNECTIONS.lock().get(&conn_id) {
            if conn.state == TcpState::Established {
                return true;
            }
        }

        if crate::logger::get_ticks().saturating_sub(start) > timeout_ms as u64 {
            return false;
        }
        spins = spins.wrapping_add(1);
        if spins > 2_000_000 {
            return false;
        }
        core::hint::spin_loop();
    }
}

// ============================================================================
// Socket API support functions
// ============================================================================

/// Check if a connection is established (for socket API)
pub fn is_connected(dest_ip: [u8; 4], dest_port: u16) -> bool {
    let src_ip = get_source_ip();
    let conns = CONNECTIONS.lock();
    
    for (id, conn) in conns.iter() {
        if id.dst_ip == ip_to_u32(dest_ip) && id.dst_port == dest_port {
            if conn.state == TcpState::Established {
                return true;
            }
        }
    }
    false
}

/// Send data on an established connection (for socket API)
pub fn send_data(dest_ip: [u8; 4], dest_port: u16, src_port: u16, data: &[u8]) -> Result<(), &'static str> {
    // Just wrap send_payload with bounds checking
    if data.is_empty() {
        return Ok(());
    }
    
    // Fragment large data into TCP segments (MSS ~1460 bytes)
    const MSS: usize = 1400;
    
    for chunk in data.chunks(MSS) {
        send_payload(dest_ip, dest_port, src_port, chunk)?;
    }
    
    Ok(())
}

/// Receive data from a connection (for socket API)
pub fn receive_data(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> Option<alloc::vec::Vec<u8>> {
    recv_data(dest_ip, dest_port, src_port)
}

/// Get connection state for debugging
pub fn get_connection_state(dest_ip: [u8; 4], dest_port: u16, src_port: u16) -> Option<TcpState> {
    let src_ip = get_source_ip();
    let conn_id = ConnectionId {
        src_ip: ip_to_u32(src_ip),
        dst_ip: ip_to_u32(dest_ip),
        src_port,
        dst_port: dest_port,
    };
    
    CONNECTIONS.lock().get(&conn_id).map(|c| c.state)
}
