//! TCP Protocol (minimal scaffolding)

use alloc::collections::{BTreeMap, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU16, Ordering};
use spin::Mutex;

/// TCP flags
pub mod flags {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const FIN: u8 = 0x01;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYN: u8 = 0x02;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const RST: u8 = 0x04;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PSH: u8 = 0x08;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ACK: u8 = 0x10;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const URG: u8 = 0x20;
}

/// TCP connection state — full RFC 793 state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Enumeration — a type that can be one of several variants.
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    LastAcknowledge,
    TimeWait,
    Closing,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ConnectionId {
    source_ip: u32,
    destination_ip: u32,
    source_port: u16,
    destination_port: u16,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone)]
struct TcpConnection {
    state: TcpState,
    sequence: u32, // next seq to send
    acknowledge: u32, // next ack to send
    fin_received: bool,
    fin_sent: bool,
    // Delayed ACK: count packets since last ACK
    pending_acks: u8,
    last_acknowledge_time: u64,
    // Retransmission tracking
    last_sent_sequence: u32,
    last_sent_time: u64,
    retransmit_count: u8,
    /// Oldest unacknowledged sequence number
    snd_una: u32,
}

/// Unacknowledged segment awaiting retransmission
#[derive(Clone)]
struct UnackedSegment {
    connection_id: ConnectionId,
    sequence: u32,
    data: Vec<u8>,       // raw TCP payload (not full segment)
    dest_ip: [u8; 4],
    dest_port: u16,
    source_port: u16,
    sent_time: u64,
    retries: u8,
}

/// Sliding window of unacked segments for retransmission.
/// Bounded to MAX_RETRANSMIT_QUEUE entries to limit memory usage.
static RETRANSMIT_QUEUE: Mutex<VecDeque<UnackedSegment>> = Mutex::new(VecDeque::new());
/// Max segments kept for retransmission (bounded memory: ~32 * 1400 = ~44KB)
const MAXIMUM_RETRANSMIT_QUEUE: usize = 32;

// Global shared state guarded by a Mutex (mutual exclusion lock).
static CONNECTIONS: Mutex<BTreeMap<ConnectionId, TcpConnection>> = Mutex::new(BTreeMap::new());
// Global shared state guarded by a Mutex (mutual exclusion lock).
static RECEIVE_DATA: Mutex<BTreeMap<ConnectionId, VecDeque<Vec<u8>>>> = Mutex::new(BTreeMap::new());
// Atomic variable — provides lock-free thread-safe access.
static NEXT_EPHEMERAL_PORT: AtomicU16 = AtomicU16::new(49152);
/// Secret key for RFC 6528 ISN generation (set once at boot)
static ISN_SECRET: Mutex<[u8; 16]> = Mutex::new([0u8; 16]);

/// Generate a cryptographically unpredictable Initial Sequence Number (RFC 6528).
/// ISN = SHA-256(src_ip, dst_ip, src_port, dst_port, secret)[0..4] + time_component
fn generate_isn(source_ip: [u8; 4], destination_ip: [u8; 4], source_port: u16, destination_port: u16) -> u32 {
    let secret = ISN_SECRET.lock();
    let mut data = [0u8; 28]; // 4+4+2+2+16
    data[0..4].copy_from_slice(&source_ip);
    data[4..8].copy_from_slice(&destination_ip);
    data[8..10].copy_from_slice(&source_port.to_be_bytes());
    data[10..12].copy_from_slice(&destination_port.to_be_bytes());
    data[12..28].copy_from_slice(&*secret);
    drop(secret);
    let hash = crate::tls13::crypto::sha256(&data);
    let h = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
    // Add a time component (4 µs clock) to prevent ISN reuse across connections
    let ticks = crate::logger::get_ticks() as u32;
    h.wrapping_add(ticks)
}

/// Initialize the ISN secret key (call once at boot, after RNG init)
pub fn initialize_isn_secret() {
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
    let connection_id = listener.accept_queue.pop_front()?;
    let remote_ip = [
        ((connection_id.destination_ip >> 24) & 0xFF) as u8,
        ((connection_id.destination_ip >> 16) & 0xFF) as u8,
        ((connection_id.destination_ip >> 8) & 0xFF) as u8,
        (connection_id.destination_ip & 0xFF) as u8,
    ];
    Some((connection_id.source_port, remote_ip, connection_id.destination_port))
}

/// Performance: batch ACK threshold (send ACK every N packets or after timeout)
const DELAYED_ACKNOWLEDGE_PACKETS: u8 = 4;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const DELAYED_ACKNOWLEDGE_MOUSE: u64 = 20;

/// TCP Window size (larger = faster downloads)
const TCP_WINDOW_SIZE: u16 = 65535;

/// Retransmission timeout in ms
const RETRANSMIT_TIMEOUT_MOUSE: u64 = 1000;
/// Maximum retransmission attempts
const MAXIMUM_RETRANSMITS: u8 = 3;

/// Maximum simultaneous TCP connections (DoS protection)
const MAXIMUM_CONNECTIONS: usize = 512;
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
fn tcp_checksum(source_ip: [u8; 4], destination_ip: [u8; 4], segment: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    // Pseudo-header fields fed incrementally — no heap Vec needed
    checksum_accumulate(&mut sum, &source_ip);
    checksum_accumulate(&mut sum, &destination_ip);
    let protocol_length: [u8; 4] = [0, 6, (segment.len() >> 8) as u8, segment.len() as u8];
    checksum_accumulate(&mut sum, &protocol_length);
    checksum_accumulate(&mut sum, segment);
    checksum_finalize(sum)
}

/// Public TCP checksum for use by packet crafting / replay modules.
pub fn tcp_checksum_external(source_ip: [u8; 4], destination_ip: [u8; 4], segment: &[u8]) -> u16 {
    tcp_checksum(source_ip, destination_ip, segment)
}

/// Garbage-collect stale TIME_WAIT / dead connections.
/// Called periodically (e.g. from `tcp_poll` or timer tick).
pub fn gc_stale_connections() {
    let now = crate::logger::get_ticks();
    let mut conns = CONNECTIONS.lock();
    let mut receive = RECEIVE_DATA.lock();
    conns.retain(|id, connection| {
        let keep = // Pattern matching — Rust's exhaustive branching construct.
match connection.state {
            TcpState::TimeWait => now.wrapping_sub(connection.last_acknowledge_time) < TIME_WAIT_TICKS,
            TcpState::Closed => false,
            _ => true,
        };
        if !keep {
            receive.remove(id);
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
    for (id, connection) in conns.iter() {
        let destination = [
            ((id.destination_ip >> 24) & 0xFF) as u8,
            ((id.destination_ip >> 16) & 0xFF) as u8,
            ((id.destination_ip >> 8) & 0xFF) as u8,
            (id.destination_ip & 0xFF) as u8,
        ];
        out.push(alloc::format!(
            "{:<6} {}.{}.{}.{}:{:<5}  {:?}",
            id.source_port, destination[0], destination[1], destination[2], destination[3], id.destination_port, connection.state
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
pub fn handle_packet(data: &[u8], source_ip: [u8; 4], destination_ip: [u8; 4]) {
    if data.len() < 20 {
        return;
    }

    let source_port = u16::from_be_bytes([data[0], data[1]]);
    let destination_port = u16::from_be_bytes([data[2], data[3]]);
    let sequence = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let acknowledge_number = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    let data_offset = data[12] >> 4;
    let tcp_flags = data[13];
    let header_length = (data_offset as usize) * 4;

    if data.len() < header_length {
        return;
    }

    let connection_id = ConnectionId {
        source_ip: ip_to_u32(destination_ip),
        destination_ip: ip_to_u32(source_ip),
        source_port: destination_port,
        destination_port: source_port,
    };

    let payload = &data[header_length..];
    let fin  = (tcp_flags & flags::FIN) != 0;
    let syn  = (tcp_flags & flags::SYN) != 0;
    let acknowledge  = (tcp_flags & flags::ACK) != 0;
    let rst  = (tcp_flags & flags::RST) != 0;
    let psh  = (tcp_flags & flags::PSH) != 0;
    let payload_length = payload.len() as u32;

    let mut conns = CONNECTIONS.lock();

    if let Some(connection) = conns.get_mut(&connection_id) {
        // ── RST — immediately close ──
        if rst {
            connection.state = TcpState::Closed;
            connection.fin_received = true;
            connection.fin_sent = true;
            return;
        }

        if connection.fin_received && connection.fin_sent && connection.state == TcpState::Closed {
            return;
        }

                // Pattern matching — Rust's exhaustive branching construct.
match connection.state {
            // ── Client handshake: waiting for SYN-ACK ──
            TcpState::SynSent => {
                if syn && acknowledge {
                    connection.state = TcpState::Established;
                    connection.acknowledge = sequence.wrapping_add(1);
                    connection.pending_acks = 0;
                    connection.last_acknowledge_time = crate::logger::get_ticks();
                    drop(conns);
                    let _ = send_acknowledge(source_ip, source_port, destination_port);
                }
            }

            // ── Server handshake: waiting for ACK to complete 3-way ──
            TcpState::SynReceived => {
                if acknowledge {
                    connection.state = TcpState::Established;
                    let listen_port = connection_id.source_port;
                    drop(conns);
                    // Move connection to the accept queue
                    let mut listeners = LISTENERS.lock();
                    if let Some(listener) = listeners.get_mut(&listen_port) {
                        listener.accept_queue.push_back(connection_id);
                    }
                }
            }

            // ── Data transfer ──
            TcpState::Established => {
                // Process ACK: advance snd_una and purge retransmit queue
                if acknowledge && acknowledge_number > connection.snd_una {
                    connection.snd_una = acknowledge_number;
                    connection.retransmit_count = 0;
                    // Purge acknowledged segments from retransmit queue
                    let mut rtx = RETRANSMIT_QUEUE.lock();
                    rtx.retain(|seg| {
                        seg.connection_id != connection_id || seg.sequence.wrapping_add(seg.data.len() as u32) > acknowledge_number
                    });
                }

                // Store payload
                if !payload.is_empty() {
                    let new_acknowledge = sequence.wrapping_add(payload_length);
                    if new_acknowledge > connection.acknowledge {
                        connection.acknowledge = new_acknowledge;
                        let mut receive = RECEIVE_DATA.lock();
                        receive.entry(connection_id).or_insert_with(VecDeque::new).push_back(payload.to_vec());
                    }
                }

                if fin {
                    // Peer initiated close → CloseWait
                    connection.acknowledge = sequence.wrapping_add(payload_length).wrapping_add(1);
                    connection.fin_received = true;
                    connection.state = TcpState::CloseWait;
                    drop(conns);
                    let _ = send_acknowledge(source_ip, source_port, destination_port);
                    return;
                }

                // Delayed ACK logic
                if !payload.is_empty() {
                    connection.pending_acks = connection.pending_acks.saturating_add(1);
                    let now = crate::logger::get_ticks();
                    let should_acknowledge = psh
                        || connection.pending_acks >= DELAYED_ACKNOWLEDGE_PACKETS
                        || now.saturating_sub(connection.last_acknowledge_time) >= DELAYED_ACKNOWLEDGE_MOUSE;
                    if should_acknowledge {
                        connection.pending_acks = 0;
                        connection.last_acknowledge_time = now;
                        drop(conns);
                        let _ = send_acknowledge(source_ip, source_port, destination_port);
                    }
                }
            }

            // ── Active close: we sent FIN, waiting for ACK/FIN ──
            TcpState::FinWait1 => {
                if fin && acknowledge {
                    // Simultaneous close shortcut → TimeWait
                    connection.acknowledge = sequence.wrapping_add(1);
                    connection.fin_received = true;
                    connection.state = TcpState::TimeWait;
                    connection.last_acknowledge_time = crate::logger::get_ticks();
                    drop(conns);
                    let _ = send_acknowledge(source_ip, source_port, destination_port);
                } else if fin {
                    connection.acknowledge = sequence.wrapping_add(1);
                    connection.fin_received = true;
                    connection.state = TcpState::Closing;
                    drop(conns);
                    let _ = send_acknowledge(source_ip, source_port, destination_port);
                } else if acknowledge {
                    connection.state = TcpState::FinWait2;
                }
            }

            TcpState::FinWait2 => {
                if fin {
                    connection.acknowledge = sequence.wrapping_add(1);
                    connection.fin_received = true;
                    connection.state = TcpState::TimeWait;
                    connection.last_acknowledge_time = crate::logger::get_ticks();
                    drop(conns);
                    let _ = send_acknowledge(source_ip, source_port, destination_port);
                }
            }

            TcpState::Closing => {
                if acknowledge {
                    connection.state = TcpState::TimeWait;
                    connection.last_acknowledge_time = crate::logger::get_ticks(); // start TIME_WAIT timer
                }
            }

            TcpState::LastAcknowledge => {
                if acknowledge {
                    connection.state = TcpState::Closed;
                }
            }

            TcpState::CloseWait => {
                // May still receive data retransmissions
                if !payload.is_empty() {
                    connection.acknowledge = sequence.wrapping_add(payload_length);
                    let mut receive = RECEIVE_DATA.lock();
                    receive.entry(connection_id).or_insert_with(VecDeque::new).push_back(payload.to_vec());
                }
            }

            _ => {}
        }
    } else {
        // ── No existing connection — check listeners for incoming SYN ──
        if syn && !acknowledge {
            // Enforce connection limit (DoS protection) — inline GC to avoid double lock
            if conns.len() >= MAXIMUM_CONNECTIONS {
                let now = crate::logger::get_ticks();
                let mut receive = RECEIVE_DATA.lock();
                conns.retain(|id, c| {
                    let keep = // Pattern matching — Rust's exhaustive branching construct.
match c.state {
                        TcpState::TimeWait => now.wrapping_sub(c.last_acknowledge_time) < TIME_WAIT_TICKS,
                        TcpState::Closed => false,
                        _ => true,
                    };
                    if !keep { receive.remove(id); }
                    keep
                });
            }
            if conns.len() >= MAXIMUM_CONNECTIONS {
                crate::serial_println!("[TCP] Connection limit reached ({}), dropping SYN", MAXIMUM_CONNECTIONS);
                return;
            }
            let mut listeners = LISTENERS.lock();
            if let Some(listener) = listeners.get_mut(&destination_port) {
                if listener.accept_queue.len() < listener.backlog as usize + 16 {
                    let source_ip_bytes = [
                        ((connection_id.source_ip >> 24) & 0xFF) as u8,
                        ((connection_id.source_ip >> 16) & 0xFF) as u8,
                        ((connection_id.source_ip >> 8) & 0xFF) as u8,
                        (connection_id.source_ip & 0xFF) as u8,
                    ];
                    let destination_ip_bytes = [
                        ((connection_id.destination_ip >> 24) & 0xFF) as u8,
                        ((connection_id.destination_ip >> 16) & 0xFF) as u8,
                        ((connection_id.destination_ip >> 8) & 0xFF) as u8,
                        (connection_id.destination_ip & 0xFF) as u8,
                    ];
                    let initialize_sequence = generate_isn(source_ip_bytes, destination_ip_bytes, connection_id.source_port, connection_id.destination_port);
                    let new_connection = TcpConnection {
                        state: TcpState::SynReceived,
                        sequence: initialize_sequence.wrapping_add(1),
                        acknowledge: sequence.wrapping_add(1),
                        fin_received: false,
                        fin_sent: false,
                        pending_acks: 0,
                        last_acknowledge_time: crate::logger::get_ticks(),
                        last_sent_sequence: initialize_sequence,
                        last_sent_time: crate::logger::get_ticks(),
                        retransmit_count: 0,
                        snd_una: initialize_sequence.wrapping_add(1),
                    };
                    drop(listeners);
                    conns.insert(connection_id, new_connection);
                    drop(conns);
                    // Send SYN-ACK
                    let _ = send_syn_acknowledge(source_ip, source_port, destination_port,
                                         sequence.wrapping_add(1), initialize_sequence);
                }
            }
        }
    }
}

/// Send a TCP SYN (start of connection)
pub fn send_syn(dest_ip: [u8; 4], dest_port: u16) -> Result<u16, &'static str> {
    let source_ip = get_source_ip();
    // (SYN IP log removed)
    let source_port = NEXT_EPHEMERAL_PORT.fetch_add(1, Ordering::Relaxed);
    let sequence = generate_isn(source_ip, dest_ip, source_port, dest_port);

    let mut segment = Vec::with_capacity(20);
    segment.extend_from_slice(&source_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&sequence.to_be_bytes());
    segment.extend_from_slice(&0u32.to_be_bytes()); // ack
    segment.push(0x50); // data offset=5, no options
    segment.push(flags::SYN);
    segment.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes()); // max window size
    segment.extend_from_slice(&0u16.to_be_bytes()); // checksum (fill later)
    segment.extend_from_slice(&0u16.to_be_bytes()); // urgent pointer

    let csum = tcp_checksum(source_ip, dest_ip, &segment);
    segment[16] = (csum >> 8) as u8;
    segment[17] = (csum & 0xFF) as u8;
    
    // (SYN segment debug removed)

    let connection_id = ConnectionId {
        source_ip: ip_to_u32(source_ip),
        destination_ip: ip_to_u32(dest_ip),
        source_port,
        destination_port: dest_port,
    };
    CONNECTIONS.lock().insert(connection_id, TcpConnection {
        state: TcpState::SynSent,
        sequence: sequence.wrapping_add(1),
        acknowledge: 0,
        fin_received: false,
        fin_sent: false,
        pending_acks: 0,
        last_acknowledge_time: crate::logger::get_ticks(),
        last_sent_sequence: sequence,
        last_sent_time: crate::logger::get_ticks(),
        retransmit_count: 0,
        snd_una: sequence.wrapping_add(1),
    });

    // (SYN direction log removed)

    crate::netstack::ip::send_packet(dest_ip, 6, &segment)?;
    Ok(source_port)
}

/// Send a TCP SYN-ACK (server side of handshake)
fn send_syn_acknowledge(dest_ip: [u8; 4], dest_port: u16, source_port: u16, acknowledge_number: u32, sequence: u32) -> Result<(), &'static str> {
    let source_ip = get_source_ip();

    let mut segment = Vec::with_capacity(20);
    segment.extend_from_slice(&source_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&sequence.to_be_bytes());
    segment.extend_from_slice(&acknowledge_number.to_be_bytes());
    segment.push(0x50);
    segment.push(flags::SYN | flags::ACK);
    segment.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());

    let csum = tcp_checksum(source_ip, dest_ip, &segment);
    segment[16] = (csum >> 8) as u8;
    segment[17] = (csum & 0xFF) as u8;

    crate::netstack::ip::send_packet(dest_ip, 6, &segment)
}

/// Send ACK for an established connection
pub fn send_acknowledge(dest_ip: [u8; 4], dest_port: u16, source_port: u16) -> Result<(), &'static str> {
    let source_ip = get_source_ip();
    let connection_id = ConnectionId {
        source_ip: ip_to_u32(source_ip),
        destination_ip: ip_to_u32(dest_ip),
        source_port,
        destination_port: dest_port,
    };

    let (sequence, acknowledge) = {
        let conns = CONNECTIONS.lock();
        let connection = conns.get(&connection_id).ok_or("Connection not found")?;
        (connection.sequence, connection.acknowledge)
    };

    let mut segment = Vec::with_capacity(20);
    segment.extend_from_slice(&source_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&sequence.to_be_bytes());
    segment.extend_from_slice(&acknowledge.to_be_bytes());
    segment.push(0x50);
    segment.push(flags::ACK);
    segment.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());

    let csum = tcp_checksum(source_ip, dest_ip, &segment);
    segment[16] = (csum >> 8) as u8;
    segment[17] = (csum & 0xFF) as u8;

    crate::netstack::ip::send_packet(dest_ip, 6, &segment)
}

/// Send TCP payload with PSH|ACK
pub fn send_payload(dest_ip: [u8; 4], dest_port: u16, source_port: u16, payload: &[u8]) -> Result<(), &'static str> {
    let source_ip = get_source_ip();
    let connection_id = ConnectionId {
        source_ip: ip_to_u32(source_ip),
        destination_ip: ip_to_u32(dest_ip),
        source_port,
        destination_port: dest_port,
    };

    let (sequence, acknowledge) = {
        let conns = CONNECTIONS.lock();
        let connection = conns.get(&connection_id).ok_or("Connection not found")?;
        (connection.sequence, connection.acknowledge)
    };

    let mut segment = Vec::with_capacity(20 + payload.len());
    segment.extend_from_slice(&source_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&sequence.to_be_bytes());
    segment.extend_from_slice(&acknowledge.to_be_bytes());
    segment.push(0x50);
    segment.push(flags::PSH | flags::ACK);
    segment.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(payload);

    let csum = tcp_checksum(source_ip, dest_ip, &segment);
    segment[16] = (csum >> 8) as u8;
    segment[17] = (csum & 0xFF) as u8;

    crate::netstack::ip::send_packet(dest_ip, 6, &segment)?;

    let now = crate::time::uptime_mouse();
    let mut conns = CONNECTIONS.lock();
    if let Some(connection) = conns.get_mut(&connection_id) {
        connection.last_sent_sequence = sequence;
        connection.last_sent_time = now;
        connection.sequence = connection.sequence.wrapping_add(payload.len() as u32);
    }
    drop(conns);

    // Queue for retransmission (bounded sliding window)
    let mut rtx = RETRANSMIT_QUEUE.lock();
    if rtx.len() >= MAXIMUM_RETRANSMIT_QUEUE {
        rtx.pop_front(); // drop oldest — it's either ACKed or too old
    }
    rtx.push_back(UnackedSegment {
        connection_id,
        sequence,
        data: payload.to_vec(),
        dest_ip,
        dest_port,
        source_port,
        sent_time: now,
        retries: 0,
    });

    Ok(())
}

/// Send TCP FIN (close connection)
pub fn send_fin(dest_ip: [u8; 4], dest_port: u16, source_port: u16) -> Result<(), &'static str> {
    let source_ip = get_source_ip();
    let connection_id = ConnectionId {
        source_ip: ip_to_u32(source_ip),
        destination_ip: ip_to_u32(dest_ip),
        source_port,
        destination_port: dest_port,
    };

    let (sequence, acknowledge, already_sent) = {
        let conns = CONNECTIONS.lock();
        let connection = conns.get(&connection_id).ok_or("Connection not found")?;
        (connection.sequence, connection.acknowledge, connection.fin_sent)
    };

    if already_sent {
        return Ok(());
    }

    let mut segment = Vec::with_capacity(20);
    segment.extend_from_slice(&source_port.to_be_bytes());
    segment.extend_from_slice(&dest_port.to_be_bytes());
    segment.extend_from_slice(&sequence.to_be_bytes());
    segment.extend_from_slice(&acknowledge.to_be_bytes());
    segment.push(0x50);
    segment.push(flags::FIN | flags::ACK);
    segment.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());
    segment.extend_from_slice(&0u16.to_be_bytes());

    let csum = tcp_checksum(source_ip, dest_ip, &segment);
    segment[16] = (csum >> 8) as u8;
    segment[17] = (csum & 0xFF) as u8;

    crate::netstack::ip::send_packet(dest_ip, 6, &segment)?;

    // Clear retransmit queue for this connection
    RETRANSMIT_QUEUE.lock().retain(|seg| seg.connection_id != connection_id);

    let mut conns = CONNECTIONS.lock();
    if let Some(connection) = conns.get_mut(&connection_id) {
        connection.sequence = connection.sequence.wrapping_add(1);
        connection.fin_sent = true;
        // State transitions for active/passive close
        match connection.state {
            TcpState::Established => connection.state = TcpState::FinWait1,
            TcpState::CloseWait  => connection.state = TcpState::LastAcknowledge,
            _ => {}
        }
    }
    Ok(())
}

/// Flush any pending ACKs for a connection (call periodically during downloads)
pub fn flush_pending_acks(dest_ip: [u8; 4], dest_port: u16, source_port: u16) {
    let source_ip = get_source_ip();
    let connection_id = ConnectionId {
        source_ip: ip_to_u32(source_ip),
        destination_ip: ip_to_u32(dest_ip),
        source_port,
        destination_port: dest_port,
    };

    let should_acknowledge = {
        let mut conns = CONNECTIONS.lock();
        if let Some(connection) = conns.get_mut(&connection_id) {
            if connection.pending_acks > 0 {
                connection.pending_acks = 0;
                connection.last_acknowledge_time = crate::logger::get_ticks();
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if should_acknowledge {
        let _ = send_acknowledge(dest_ip, dest_port, source_port);
    }
}

/// Receive buffered TCP payloads for a connection (optimized batch receive)
pub fn recv_data(dest_ip: [u8; 4], dest_port: u16, source_port: u16) -> Option<Vec<u8>> {
    let source_ip = get_source_ip();
    let connection_id = ConnectionId {
        source_ip: ip_to_u32(source_ip),
        destination_ip: ip_to_u32(dest_ip),
        source_port,
        destination_port: dest_port,
    };

    {
        let mut receive = RECEIVE_DATA.lock();
        if let Some(queue) = receive.get_mut(&connection_id) {
            if !queue.is_empty() {
                return queue.pop_front();
            }
        }
    }

    let mut conns = CONNECTIONS.lock();
    if let Some(connection) = conns.get(&connection_id) {
        if connection.fin_received && connection.fin_sent {
            conns.remove(&connection_id);
            RECEIVE_DATA.lock().remove(&connection_id);
        }
    }
    None
}

/// Check if FIN has been received for a connection
pub fn fin_received(dest_ip: [u8; 4], dest_port: u16, source_port: u16) -> bool {
    let source_ip = get_source_ip();
    let connection_id = ConnectionId {
        source_ip: ip_to_u32(source_ip),
        destination_ip: ip_to_u32(dest_ip),
        source_port,
        destination_port: dest_port,
    };

    CONNECTIONS
        .lock()
        .get(&connection_id)
        .map(|c| c.fin_received)
        .unwrap_or(true)
}

/// Wait for a SYN-ACK (connection established) with SYN retransmission
pub fn wait_for_established(dest_ip: [u8; 4], dest_port: u16, source_port: u16, timeout_mouse: u32) -> bool {
    let source_ip = get_source_ip();
    let connection_id = ConnectionId {
        source_ip: ip_to_u32(source_ip),
        destination_ip: ip_to_u32(dest_ip),
        source_port,
        destination_port: dest_port,
    };

    let start = crate::logger::get_ticks();
    let mut last_syn_time = start;
    let mut syn_retries: u8 = 0;
    let mut spins: u32 = 0;
    
        // Infinite loop — runs until an explicit `break`.
loop {
        crate::netstack::poll();

        if let Some(connection) = CONNECTIONS.lock().get(&connection_id) {
            if connection.state == TcpState::Established {
                return true;
            }
            // Connection was RST'd
            if connection.state == TcpState::Closed {
                return false;
            }
        }

        let now = crate::logger::get_ticks();
        
        // Retransmit SYN if no SYN-ACK received within timeout
        if now.saturating_sub(last_syn_time) > RETRANSMIT_TIMEOUT_MOUSE && syn_retries < MAXIMUM_RETRANSMITS {
            syn_retries += 1;
            last_syn_time = now;
            // (SYN retransmit log removed)
            
            // Rebuild and resend SYN
            let sequence = {
                let conns = CONNECTIONS.lock();
                conns.get(&connection_id).map(|c| c.last_sent_sequence).unwrap_or(0)
            };
            
            let mut segment = Vec::with_capacity(20);
            segment.extend_from_slice(&source_port.to_be_bytes());
            segment.extend_from_slice(&dest_port.to_be_bytes());
            segment.extend_from_slice(&sequence.to_be_bytes());
            segment.extend_from_slice(&0u32.to_be_bytes());
            segment.push(0x50);
            segment.push(flags::SYN);
            segment.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes());
            segment.extend_from_slice(&0u16.to_be_bytes());
            segment.extend_from_slice(&0u16.to_be_bytes());
            
            let csum = tcp_checksum(source_ip, dest_ip, &segment);
            segment[16] = (csum >> 8) as u8;
            segment[17] = (csum & 0xFF) as u8;
            
            let _ = crate::netstack::ip::send_packet(dest_ip, 6, &segment);
        }

        if now.saturating_sub(start) > timeout_mouse as u64 {
            return false;
        }
        spins = spins.wrapping_add(1);
        if spins > 2_000_000 {
            return false;
        }
        // Yield to other threads so desktop frame loop stays responsive
        crate::thread::yield_thread();
    }
}

// ============================================================================
// Socket API support functions
// ============================================================================

/// Check if a connection is established (for socket API)
pub fn is_connected(dest_ip: [u8; 4], dest_port: u16) -> bool {
    let source_ip = get_source_ip();
    let conns = CONNECTIONS.lock();
    
    // Direct lookup: try all known src_ports for this dest
    // (more efficient than iterating all connections)
    for (id, connection) in conns.iter() {
        if id.destination_ip == ip_to_u32(dest_ip) && id.destination_port == dest_port
            && id.source_ip == ip_to_u32(source_ip)
            && connection.state == TcpState::Established
        {
            return true;
        }
    }
    false
}

/// Send data on an established connection (for socket API)
pub fn send_data(dest_ip: [u8; 4], dest_port: u16, source_port: u16, data: &[u8]) -> Result<(), &'static str> {
    if data.is_empty() {
        return Ok(());
    }
    
    // Fragment large data into TCP segments (MSS ~1460 bytes)
    const MSS: usize = 1400;
    // Small batch size to avoid overwhelming receiver (no flow control)
    const BATCH: usize = 4;
    
    let total_segments = (data.len() + MSS - 1) / MSS;
    
    for (i, chunk) in data.chunks(MSS).enumerate() {
        // Retry on TX queue full instead of aborting
        let mut retries = 0u32;
                // Infinite loop — runs until an explicit `break`.
loop {
                        // Pattern matching — Rust's exhaustive branching construct.
match send_payload(dest_ip, dest_port, source_port, chunk) {
                Ok(()) => break,
                Err(e) if retries < 200 => {
                    // TX queue full or transient error — poll and retry
                    crate::netstack::poll();
                    crate::thread::yield_thread();
                    retries += 1;
                }
                Err(e) => return Err(e),
            }
        }
        
        // Poll and pace between batches to let receiver process
        if (i + 1) % BATCH == 0 {
            crate::netstack::poll();
            // Yield to let other threads run between batches
            crate::thread::yield_thread();
        }
        
    }
    
    Ok(())
}

/// Receive data from a connection (for socket API)
pub fn receive_data(dest_ip: [u8; 4], dest_port: u16, source_port: u16) -> Option<alloc::vec::Vec<u8>> {
    recv_data(dest_ip, dest_port, source_port)
}

/// Get connection state for debugging
pub fn get_connection_state(dest_ip: [u8; 4], dest_port: u16, source_port: u16) -> Option<TcpState> {
    let source_ip = get_source_ip();
    let connection_id = ConnectionId {
        source_ip: ip_to_u32(source_ip),
        destination_ip: ip_to_u32(dest_ip),
        source_port,
        destination_port: dest_port,
    };
    
    CONNECTIONS.lock().get(&connection_id).map(|c| c.state)
}

/// Check for timed-out unacknowledged segments and retransmit them.
/// Called periodically from the network poll loop.
pub fn check_retransmits() {
    let now = crate::time::uptime_mouse();
    let mut rtx = RETRANSMIT_QUEUE.lock();

    for seg in rtx.iterator_mut() {
        if now.wrapping_sub(seg.sent_time) < RETRANSMIT_TIMEOUT_MOUSE {
            continue;
        }
        if seg.retries >= MAXIMUM_RETRANSMITS {
            continue; // give up on this segment
        }

        // Check connection is still Established
        let connection_id = seg.connection_id;
        let conns = CONNECTIONS.lock();
        let still_active = conns.get(&connection_id)
            .map(|c| c.state == TcpState::Established && c.snd_una <= seg.sequence)
            .unwrap_or(false);
        drop(conns);

        if !still_active {
            continue;
        }

        // Rebuild and resend the segment
        let source_ip = get_source_ip();
        let dest_ip = seg.dest_ip;
        let acknowledge_value = {
            let conns = CONNECTIONS.lock();
            conns.get(&connection_id).map(|c| c.acknowledge).unwrap_or(0)
        };

        let mut tcp_seg = Vec::with_capacity(20 + seg.data.len());
        tcp_seg.extend_from_slice(&seg.source_port.to_be_bytes());
        tcp_seg.extend_from_slice(&seg.dest_port.to_be_bytes());
        tcp_seg.extend_from_slice(&seg.sequence.to_be_bytes());
        tcp_seg.extend_from_slice(&acknowledge_value.to_be_bytes());
        tcp_seg.push(0x50);
        tcp_seg.push(flags::PSH | flags::ACK);
        tcp_seg.extend_from_slice(&TCP_WINDOW_SIZE.to_be_bytes());
        tcp_seg.extend_from_slice(&0u16.to_be_bytes());
        tcp_seg.extend_from_slice(&0u16.to_be_bytes());
        tcp_seg.extend_from_slice(&seg.data);

        let csum = tcp_checksum(source_ip, dest_ip, &tcp_seg);
        tcp_seg[16] = (csum >> 8) as u8;
        tcp_seg[17] = (csum & 0xFF) as u8;

        let _ = crate::netstack::ip::send_packet(dest_ip, 6, &tcp_seg);
        seg.retries += 1;
        seg.sent_time = now; // reset timer with exponential backoff would be nice, but simple is fine
    }

    // Purge segments that exhausted retries
    rtx.retain(|seg| seg.retries < MAXIMUM_RETRANSMITS);
}

/// Clear retransmit queue entries for a specific connection (called on close)
pub fn clear_retransmit_queue(dest_ip: [u8; 4], dest_port: u16, source_port: u16) {
    let source_ip = get_source_ip();
    let connection_id = ConnectionId {
        source_ip: ip_to_u32(source_ip),
        destination_ip: ip_to_u32(dest_ip),
        source_port,
        destination_port: dest_port,
    };
    RETRANSMIT_QUEUE.lock().retain(|seg| seg.connection_id != connection_id);
}
