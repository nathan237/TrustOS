//! TCP Protocol (minimal scaffolding)

use alloc::collections::{BTreeMap, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU16, Ordering};
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

/// TCP connection state — full RFC 793 state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    LastAck,
    TimeWait,
    Closing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ConnectionId {
    src_ip: u32,
    dst_ip: u32,
    src_port: u16,
    dst_port: u16,
}

#[derive(Debug, Clone)]
struct TcpConnection {
    state: TcpState,
    seq: u32, // next seq to send
    ack: u32, // next ack to send
    fin_received: bool,
    fin_sent: bool,
    // Delayed ACK: count packets since last ACK
    pending_acks: u8,
    last_ack_time: u64,
    // Retransmission: store last sent segment for retransmit on timeout
    last_sent_seq: u32,
    last_sent_time: u64,
    retransmit_count: u8,
}

static CONNECTIONS: Mutex<BTreeMap<ConnectionId, TcpConnection>> = Mutex::new(BTreeMap::new());
static RX_DATA: Mutex<BTreeMap<ConnectionId, VecDeque<Vec<u8>>>> = Mutex::new(BTreeMap::new());
static NEXT_EPHEMERAL_PORT: AtomicU16 = AtomicU16::new(49152);
/// Secret key for RFC 6528 ISN generation (set once at boot)
static ISN_SECRET: Mutex<[u8; 16]> = Mutex::new([0u8; 16]);

/// Generate a cryptographically unpredictable Initial Sequence Number (RFC 6528).
/// ISN = SHA-256(src_ip, dst_ip, src_port, dst_port, secret)[0..4] + time_component
fn generate_isn(src_ip: [u8; 4], dst_ip: [u8; 4], src_port: u16, dst_port: u16) -> u32 {
    let secret = ISN_SECRET.lock();
    let mut data = [0u8; 28]; // 4+4+2+2+16
    data[0..4].copy_from_slice(&src_ip);
    data[4..8].copy_from_slice(&dst_ip);
    data[8..10].copy_from_slice(&src_port.to_be_bytes());
    data[10..12].copy_from_slice(&dst_port.to_be_bytes());
    data[12..28].copy_from_slice(&*secret);
    drop(secret);
    let hash = crate::tls13::crypto::sha256(&data);
    let h = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
    // Add a time component (4 µs clock) to prevent ISN reuse across connections
    let ticks = crate::logger::get_ticks() as u32;
    h.wrapping_add(ticks)
}

/// Initialize the ISN secret key (call once at boot, after RNG init)
pub fn init_isn_secret() {
    let mut s = ISN_SECRET.lock();
    crate::rng::secure_fill_bytes(&mut *s);
    crate::serial_println!("[TCP] ISN secret initialized (RFC 6528)");
}

/// Listener for server-side TCP (tracks port → accept queue)
struct Listener {
    backlog: u32,
    accept_queue: VecDeque<ConnectionId>,
}

/// Active listeners keyed by port
static LISTENERS: Mutex<BTreeMap<u16, Listener>> = Mutex::new(BTreeMap::new());

/// Register a listening port
pub fn listen_on(port: u16, backlog: u32) {
    LISTENERS.lock().insert(port, Listener { backlog, accept_queue: VecDeque::new() });
    crate::serial_println!("[TCP] Listening on port {}", port);
}

/// Stop listening on a port
pub fn stop_listening(port: u16) {
    LISTENERS.lock().remove(&port);
}

/// Accept a completed connection from the listener queue.
/// Returns (our_src_port, remote_ip, remote_port) or None.
pub fn accept_connection(listen_port: u16) -> Option<(u16, [u8; 4], u16)> {
    let mut listeners = LISTENERS.lock();
    let listener = listeners.get_mut(&listen_port)?;
    let conn_id = listener.accept_queue.pop_front()?;
    let remote_ip = [
        ((conn_id.dst_ip >> 24) & 0xFF) as u8,
        ((conn_id.dst_ip >> 16) & 0xFF) as u8,
        ((conn_id.dst_ip >> 8) & 0xFF) as u8,
        (conn_id.dst_ip & 0xFF) as u8,
    ];
    Some((conn_id.src_port, remote_ip, conn_id.dst_port))
}

/// Performance: batch ACK threshold (send ACK every N packets or after timeout)
const DELAYED_ACK_PACKETS: u8 = 4;
const DELAYED_ACK_MS: u64 = 20;

/// TCP Window size (larger = faster downloads)
const TCP_WINDOW_SIZE: u16 = 65535;

/// Retransmission timeout in ms
const RETRANSMIT_TIMEOUT_MS: u64 = 1000;
/// Maximum retransmission attempts
const MAX_RETRANSMITS: u8 = 3;

/// Maximum simultaneous TCP connections (DoS protection)
const MAX_CONNECTIONS: usize = 512;
/// TIME_WAIT duration in ticks (~60 s at 1 kHz)
const TIME_WAIT_TICKS: u64 = 60_000;

fn ip_to_u32(ip: [u8; 4]) -> u32 {
    ((ip[0] as u32) << 24) | ((ip[1] as u32) << 16) | ((ip[2] as u32) << 8) | (ip[3] as u32)
}

/// Incremental Internet checksum — accumulate 16-bit words into a running sum
fn checksum_accumulate(sum: &mut u32, data: &[u8]) {
    let mut i = 0;
    while i + 1 < data.len() {
        *sum += ((data[i] as u32) << 8) | (data[i + 1] as u32);
        i += 2;
    }
    if i < data.len() {
        *sum += (data[i] as u32) << 8;
    }
}

fn checksum_finalize(mut sum: u32) -> u16 {
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    checksum_accumulate(&mut sum, data);
    checksum_finalize(sum)
}

/// TCP checksum using pseudo-header — **zero-allocation** (stack-only).
fn tcp_checksum(src_ip: [u8; 4], dst_ip: [u8; 4], segment: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    // Pseudo-header fields fed incrementally — no heap Vec needed
    checksum_accumulate(&mut sum, &src_ip);
    checksum_accumulate(&mut sum, &dst_ip);
    let proto_len: [u8; 4] = [0, 6, (segment.len() >> 8) as u8, segment.len() as u8];
    checksum_accumulate(&mut sum, &proto_len);
    checksum_accumulate(&mut sum, segment);
    checksum_finalize(sum)
}

/// Public TCP checksum for use by packet crafting / replay modules.
pub fn tcp_checksum_external(src_ip: [u8; 4], dst_ip: [u8; 4], segment: &[u8]) -> u16 {
    tcp_checksum(src_ip, dst_ip, segment)
}

/// Garbage-collect stale TIME_WAIT / dead connections.
/// Called periodically (e.g. from `tcp_poll` or timer tick).
pub fn gc_stale_connections() {
    let now = crate::logger::get_ticks();
    let mut conns = CONNECTIONS.lock();
    let mut rx = RX_DATA.lock();
    conns.retain(|id, conn| {
        let keep = match conn.state {
            TcpState::TimeWait => now.wrapping_sub(conn.last_ack_time) < TIME_WAIT_TICKS,
            TcpState::Closed => false,
            _ => true,
        };
        if !keep {
            rx.remove(id);
        }
        keep
    });
}

/// Number of active TCP connections.
pub fn connection_count() -> usize {
    CONNECTIONS.lock().len()
}

/// List active connections as human-readable strings for dashboard display.
pub fn list_connections() -> Vec<String> {
    let conns = CONNECTIONS.lock();
    let mut out = Vec::with_capacity(conns.len());
    for (id, conn) in conns.iter() {
        let dst = [
            ((id.dst_ip >> 24) & 0xFF) as u8,
            ((id.dst_ip >> 16) & 0xFF) as u8,
            ((id.dst_ip >> 8) & 0xFF) as u8,
            (id.dst_ip & 0xFF) as u8,
        ];
        out.push(alloc::format!(
            "{:<6} {}.{}.{}.{}:{:<5}  {:?}",
            id.src_port, dst[0], dst[1], dst[2], dst[3], id.dst_port, conn.state
        ));
    }
    out
}

fn get_source_ip() -> [u8; 4] {
    crate::network::get_ipv4_config()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([10, 0, 2, 15])
}

/// Handle incoming TCP packet — full state machine
pub fn handle_packet(data: &[u8], src_ip: [u8; 4], dst_ip: [u8; 4]) {
    if data.len() < 20 {
        return;
    }

    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let seq = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let ack_num = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    let data_offset = data[12] >> 4;
    let tcp_flags = data[13];
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

    let payload = &data[header_len..];
    let fin  = (tcp_flags & flags::FIN) != 0;
    let syn  = (tcp_flags & flags::SYN) != 0;
    let ack  = (tcp_flags & flags::ACK) != 0;
    let rst  = (tcp_flags & flags::RST) != 0;
    let psh  = (tcp_flags & flags::PSH) != 0;
    let payload_len = payload.len() as u32;

    let mut conns = CONNECTIONS.lock();

    if let Some(conn) = conns.get_mut(&conn_id) {
        // ── RST — immediately close ──
        if rst {
            conn.state = TcpState::Closed;
            conn.fin_received = true;
            conn.fin_sent = true;
            return;
        }

        if conn.fin_received && conn.fin_sent && conn.state == TcpState::Closed {
            return;
        }

        match conn.state {
            // ── Client handshake: waiting for SYN-ACK ──
            TcpState::SynSent => {
                if syn && ack {
                    conn.state = TcpState::Established;
                    conn.ack = seq.wrapping_add(1);
                    conn.pending_acks = 0;
                    conn.last_ack_time = crate::logger::get_ticks();
                    drop(conns);
                    let _ = send_ack(src_ip, src_port, dst_port);
                }
            }

            // ── Server handshake: waiting for ACK to complete 3-way ──
            TcpState::SynReceived => {
                if ack {
                    conn.state = TcpState::Established;
                    let listen_port = conn_id.src_port;
                    drop(conns);
                    // Move connection to the accept queue
                    let mut listeners = LISTENERS.lock();
                    if let Some(listener) = listeners.get_mut(&listen_port) {
                        listener.accept_queue.push_back(conn_id);
                    }
                }
            }

            // ── Data transfer ──
            TcpState::Established => {
                // Store payload
                if !payload.is_empty() {
                    let new_ack = seq.wrapping_add(payload_len);
                    if new_ack > conn.ack {
                        conn.ack = new_ack;
                        let mut rx = RX_DATA.lock();
                        rx.entry(conn_id).or_insert_with(VecDeque::new).push_back(payload.to_vec());
                    }
                }

                if fin {
                    // Peer initiated close → CloseWait
                    conn.ack = seq.wrapping_add(payload_len).wrapping_add(1);
                    conn.fin_received = true;
                    conn.state = TcpState::CloseWait;
                    drop(conns);
                    let _ = send_ack(src_ip, src_port, dst_port);
                    return;
                }

                // Delayed ACK logic
                if !payload.is_empty() {
                    conn.pending_acks = conn.pending_acks.saturating_add(1);
                    let now = crate::logger::get_ticks();
                    let should_ack = psh
                        || conn.pending_acks >= DELAYED_ACK_PACKETS
                        || now.saturating_sub(conn.last_ack_time) >= DELAYED_ACK_MS;
                    if should_ack {
                        conn.pending_acks = 0;
                        conn.last_ack_time = now;
                        drop(conns);
                        let _ = send_ack(src_ip, src_port, dst_port);
                    }
                }
            }

            // ── Active close: we sent FIN, waiting for ACK/FIN ──
            TcpState::FinWait1 => {
                if fin && ack {
                    // Simultaneous close shortcut → TimeWait
                    conn.ack = seq.wrapping_add(1);
                    conn.fin_received = true;
                    conn.state = TcpState::TimeWait;
                    conn.last_ack_time = crate::logger::get_ticks();
                    drop(conns);
                    let _ = send_ack(src_ip, src_port, dst_port);
                } else if fin {
                    conn.ack = seq.wrapping_add(1);
                    conn.fin_received = true;
                    conn.state = TcpState::Closing;
                    drop(conns);
                    let _ = send_ack(src_ip, src_port, dst_port);
                } else if ack {
                    conn.state = TcpState::FinWait2;
                }
            }

            TcpState::FinWait2 => {
                if fin {
                    conn.ack = seq.wrapping_add(1);
                    conn.fin_received = true;
                    conn.state = TcpState::TimeWait;
                    conn.last_ack_time = crate::logger::get_ticks();
                    drop(conns);
                    let _ = send_ack(src_ip, src_port, dst_port);
                }
            }

            TcpState::Closing => {
                if ack {
                    conn.state = TcpState::TimeWait;
                    conn.last_ack_time = crate::logger::get_ticks(); // start TIME_WAIT timer
                }
            }

            TcpState::LastAck => {
                if ack {
                    conn.state = TcpState::Closed;
                }
            }

            TcpState::CloseWait => {
                // May still receive data retransmissions
                if !payload.is_empty() {
                    conn.ack = seq.wrapping_add(payload_len);
                    let mut rx = RX_DATA.lock();
                    rx.entry(conn_id).or_insert_with(VecDeque::new).push_back(payload.to_vec());
                }
            }

            _ => {}
        }
    } else {
        // ── No existing connection — check listeners for incoming SYN ──
        if syn && !ack {
            // Enforce connection limit (DoS protection) — inline GC to avoid double lock
            if conns.len() >= MAX_CONNECTIONS {
                let now = crate::logger::get_ticks();
                let mut rx = RX_DATA.lock();
                conns.retain(|id, c| {
                    let keep = match c.state {
                        TcpState::TimeWait => now.wrapping_sub(c.last_ack_time) < TIME_WAIT_TICKS,
                        TcpState::Closed => false,
                        _ => true,
                    };
                    if !keep { rx.remove(id); }
                    keep
                });
            }
            if conns.len() >= MAX_CONNECTIONS {
                crate::serial_println!("[TCP] Connection limit reached ({}), dropping SYN", MAX_CONNECTIONS);
                return;
            }
            let mut listeners = LISTENERS.lock();
            if let Some(listener) = listeners.get_mut(&dst_port) {
                if listener.accept_queue.len() < listener.backlog as usize + 16 {
                    let src_ip_bytes = [
                        ((conn_id.src_ip >> 24) & 0xFF) as u8,
                        ((conn_id.src_ip >> 16) & 0xFF) as u8,
                        ((conn_id.src_ip >> 8) & 0xFF) as u8,
                        (conn_id.src_ip & 0xFF) as u8,
                    ];
                    let dst_ip_bytes = [
                        ((conn_id.dst_ip >> 24) & 0xFF) as u8,
                        ((conn_id.dst_ip >> 16) & 0xFF) as u8,
                        ((conn_id.dst_ip >> 8) & 0xFF) as u8,
                        (conn_id.dst_ip & 0xFF) as u8,
                    ];
                    let init_seq = generate_isn(src_ip_bytes, dst_ip_bytes, conn_id.src_port, conn_id.dst_port);
                    let new_conn = TcpConnection {
                        state: TcpState::SynReceived,
                        seq: init_seq.wrapping_add(1),
                        ack: seq.wrapping_add(1),
                        fin_received: false,
                        fin_sent: false,
                        pending_acks: 0,
                        last_ack_time: crate::logger::get_ticks(),
                        last_sent_seq: init_seq,
                        last_sent_time: crate::logger::get_ticks(),
                        retransmit_count: 0,
                    };
                    drop(listeners);
                    conns.insert(conn_id, new_conn);
                    drop(conns);
                    // Send SYN-ACK
                    let _ = send_syn_ack(src_ip, src_port, dst_port,
                                         seq.wrapping_add(1), init_seq);
                }
            }
        }
    }
}

/// Send a TCP SYN (start of connection)
pub fn send_syn(dest_ip: [u8; 4], dest_port: u16) -> Result<u16, &'static str> {
    let src_ip = get_source_ip();
    // (SYN IP log removed)
    let src_port = NEXT_EPHEMERAL_PORT.fetch_add(1, Ordering::Relaxed);
    let seq = generate_isn(src_ip, dest_ip, src_port, dest_port);

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
    
    // (SYN segment debug removed)

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
        last_sent_seq: seq,
        last_sent_time: crate::logger::get_ticks(),
        retransmit_count: 0,
    });

    // (SYN direction log removed)

    crate::netstack::ip::send_packet(dest_ip, 6, &segment)?;
    Ok(src_port)
}

/// Send a TCP SYN-ACK (server side of handshake)
fn send_syn_ack(dest_ip: [u8; 4], dest_port: u16, src_port: u16, ack_num: u32, seq: u32) -> Result<(), &'static str> {
    let src_ip = get_source_ip();

    let mut segment = Vec::with_capacity(20);
    segment.extend_from_slice(&src_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&seq.to_be_bytes());
    segment.extend_from_slice(&ack_num.to_be_bytes());
    segment.push(0x50);
    segment.push(flags::SYN | flags::ACK);
    segment.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());

    let csum = tcp_checksum(src_ip, dest_ip, &segment);
    segment[16] = (csum >> 8) as u8;
    segment[17] = (csum & 0xFF) as u8;

    crate::netstack::ip::send_packet(dest_ip, 6, &segment)
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
        // State transitions for active/passive close
        match conn.state {
            TcpState::Established => conn.state = TcpState::FinWait1,
            TcpState::CloseWait  => conn.state = TcpState::LastAck,
            _ => {}
        }
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
                return queue.pop_front();
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

/// Wait for a SYN-ACK (connection established) with SYN retransmission
pub fn wait_for_established(dest_ip: [u8; 4], dest_port: u16, src_port: u16, timeout_ms: u32) -> bool {
    let src_ip = get_source_ip();
    let conn_id = ConnectionId {
        src_ip: ip_to_u32(src_ip),
        dst_ip: ip_to_u32(dest_ip),
        src_port,
        dst_port: dest_port,
    };

    let start = crate::logger::get_ticks();
    let mut last_syn_time = start;
    let mut syn_retries: u8 = 0;
    let mut spins: u32 = 0;
    
    loop {
        crate::netstack::poll();

        if let Some(conn) = CONNECTIONS.lock().get(&conn_id) {
            if conn.state == TcpState::Established {
                return true;
            }
            // Connection was RST'd
            if conn.state == TcpState::Closed {
                return false;
            }
        }

        let now = crate::logger::get_ticks();
        
        // Retransmit SYN if no SYN-ACK received within timeout
        if now.saturating_sub(last_syn_time) > RETRANSMIT_TIMEOUT_MS && syn_retries < MAX_RETRANSMITS {
            syn_retries += 1;
            last_syn_time = now;
            // (SYN retransmit log removed)
            
            // Rebuild and resend SYN
            let seq = {
                let conns = CONNECTIONS.lock();
                conns.get(&conn_id).map(|c| c.last_sent_seq).unwrap_or(0)
            };
            
            let mut segment = Vec::with_capacity(20);
            segment.extend_from_slice(&src_port.to_be_bytes());
            segment.extend_from_slice(&dest_port.to_be_bytes());
            segment.extend_from_slice(&seq.to_be_bytes());
            segment.extend_from_slice(&0u32.to_be_bytes());
            segment.push(0x50);
            segment.push(flags::SYN);
            segment.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes());
            segment.extend_from_slice(&0u16.to_be_bytes());
            segment.extend_from_slice(&0u16.to_be_bytes());
            
            let csum = tcp_checksum(src_ip, dest_ip, &segment);
            segment[16] = (csum >> 8) as u8;
            segment[17] = (csum & 0xFF) as u8;
            
            let _ = crate::netstack::ip::send_packet(dest_ip, 6, &segment);
        }

        if now.saturating_sub(start) > timeout_ms as u64 {
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
    
    // Direct lookup: try all known src_ports for this dest
    // (more efficient than iterating all connections)
    for (id, conn) in conns.iter() {
        if id.dst_ip == ip_to_u32(dest_ip) && id.dst_port == dest_port
            && id.src_ip == ip_to_u32(src_ip)
            && conn.state == TcpState::Established
        {
            return true;
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
