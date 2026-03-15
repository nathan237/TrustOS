//! JARVIS RPC — Binary Remote Procedure Call Protocol
//!
//! Lightweight RPC over TCP for inter-JARVIS communication.
//! Used for weight synchronization, gradient exchange, inference delegation,
//! and cluster coordination commands.
//!
//! # Wire Format
//!
//! ```text
//! Request:
//! [0..4]   Magic: b"JRPC"
//! [4..8]   Request ID (u32)
//! [8]      Command byte
//! [9..13]  Payload length (u32, big-endian)
//! [13..]   Payload (variable)
//!
//! Response:
//! [0..4]   Magic: b"JRPC"
//! [4..8]   Request ID (u32, echoed)
//! [8]      Status: 0=OK, 1=Error, 2=Busy
//! [9..13]  Payload length (u32, big-endian)
//! [13..]   Payload (variable)
//! ```

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

const MAGIC: &[u8; 4] = b"JRPC";
// Compile-time constant — evaluated at compilation, zero runtime cost.
const HEADER_SIZE: usize = 13;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const MAXIMUM_PAYLOAD_SIZE: usize = 32 * 1024 * 1024; // 32 MB max (for full model weights)
const CONNECT_TIMEOUT_MOUSE: u32 = 30000;

// ═══════════════════════════════════════════════════════════════════════════════
// RPC Commands
// ═══════════════════════════════════════════════════════════════════════════════

/// RPC command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
// Enumeration — a type that can be one of several variants.
pub enum Command {
    /// Ping — check if peer is alive. Response: empty payload.
    Ping = 0,

    /// Get model weights — request full serialized weights.
    /// Response payload: [f32] flat array as bytes.
    GetWeights = 1,

    /// Push model weights to peer — payload is serialized weights.
    /// Response: status only.
    PushWeights = 2,

    /// Push gradient update — payload contains gradient delta as [f32] bytes.
    /// Response: status only.
    PushGradients = 3,

    /// Request inference — payload is prompt bytes (UTF-8).
    /// Response payload: generated text (UTF-8).
    Inference = 4,

    /// Get peer status — no payload.
    /// Response payload: JSON-like status string.
    GetStatus = 5,

    /// Election vote request — payload: candidate term (u64).
    /// Response payload: vote granted (1 byte: 0=no, 1=yes).
    VoteRequest = 6,

    /// Leader heartbeat (RAFT) — payload: leader term (u64).
    /// Response: ACK.
    LeaderHeartbeat = 7,

    /// Append training data — payload: UTF-8 text to train on.
    /// Response: loss (f32 as 4 bytes).
    TrainData = 8,

    /// Get training steps count.
    /// Response payload: u64 as 8 bytes.
    GetTrainingSteps = 9,

    /// Execute a distributed task — payload: UTF-8 task description.
    /// Response payload: UTF-8 result string.
    TaskExecute = 10,

    /// Get a chunk of model weights — payload: offset(u32) + length(u32).
    /// Response payload: bytes at [offset..offset+length] of serialized weights.
    /// For memory-efficient transfer of large models.
    GetWeightsChunk = 11,
}

// Implementation block — defines methods for the type above.
impl Command {
    fn from_byte(b: u8) -> Option<Self> {
                // Pattern matching — Rust's exhaustive branching construct.
match b {
            0 => Some(Command::Ping),
            1 => Some(Command::GetWeights),
            2 => Some(Command::PushWeights),
            3 => Some(Command::PushGradients),
            4 => Some(Command::Inference),
            5 => Some(Command::GetStatus),
            6 => Some(Command::VoteRequest),
            7 => Some(Command::LeaderHeartbeat),
            8 => Some(Command::TrainData),
            9 => Some(Command::GetTrainingSteps),
            10 => Some(Command::TaskExecute),
            11 => Some(Command::GetWeightsChunk),
            _ => None,
        }
    }
}

/// Response status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
// Enumeration — a type that can be one of several variants.
pub enum Status {
    Ok = 0,
    Error = 1,
    Busy = 2,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Global State
// ═══════════════════════════════════════════════════════════════════════════════

/// Whether the RPC server is listening
static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Next request ID for client calls
static NEXT_REQUEST_ID: AtomicU32 = AtomicU32::new(1);

/// Total RPC calls served
static CALLS_SERVED: AtomicU64 = AtomicU64::new(0);

/// Total RPC calls made as client
static CALLS_MADE: AtomicU64 = AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════════════════════
// Packet Building
// ═══════════════════════════════════════════════════════════════════════════════

/// Build an RPC request packet
fn build_request(request_id: u32, cmd: Command, payload: &[u8]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(HEADER_SIZE + payload.len());
    packet.extend_from_slice(MAGIC);
    packet.extend_from_slice(&request_id.to_be_bytes());
    packet.push(cmd as u8);
    packet.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    packet.extend_from_slice(payload);
    packet
}

/// Build an RPC response packet
fn build_response(request_id: u32, status: Status, payload: &[u8]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(HEADER_SIZE + payload.len());
    packet.extend_from_slice(MAGIC);
    packet.extend_from_slice(&request_id.to_be_bytes());
    packet.push(status as u8);
    packet.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    packet.extend_from_slice(payload);
    packet
}

/// Build just the RPC response header (for streaming large payloads separately)
fn build_response_header(request_id: u32, status: Status, payload_length: u32) -> Vec<u8> {
    let mut packet = Vec::with_capacity(HEADER_SIZE);
    packet.extend_from_slice(MAGIC);
    packet.extend_from_slice(&request_id.to_be_bytes());
    packet.push(status as u8);
    packet.extend_from_slice(&payload_length.to_be_bytes());
    packet
}

/// Parse an RPC header from raw bytes
/// Returns (req_id, cmd_or_status_byte, payload_len)
fn parse_header(data: &[u8]) -> Option<(u32, u8, u32)> {
    if data.len() < HEADER_SIZE {
        return None;
    }
    if &data[0..4] != MAGIC {
        return None;
    }
    let request_id = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let cmd = data[8];
    let payload_length = u32::from_be_bytes([data[9], data[10], data[11], data[12]]);

    if payload_length as usize > MAXIMUM_PAYLOAD_SIZE {
        return None;
    }

    Some((request_id, cmd, payload_length))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Client — Send RPC to a peer
// ═══════════════════════════════════════════════════════════════════════════════

/// Send an RPC command to a peer and wait for response
/// Returns (status, response_payload)
pub fn call(dest_ip: [u8; 4], dest_port: u16, cmd: Command, payload: &[u8]) -> Result<(Status, Vec<u8>), &'static str> {
    let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::SeqCst);

    // Connect
    let source_port = crate::netstack::tcp::send_syn(dest_ip, dest_port)?;

    if !crate::netstack::tcp::wait_for_established(dest_ip, dest_port, source_port, CONNECT_TIMEOUT_MOUSE) {
        return Err("RPC connect timeout");
    }
    // Send request
    let request = build_request(request_id, cmd, payload);
    crate::netstack::tcp::send_data(dest_ip, dest_port, source_port, &request)?;

    // Wait for response — drain all available data each iteration
    let mut response_buffer = Vec::new();
    let start = crate::time::uptime_mouse();
    let timeout_mouse = CONNECT_TIMEOUT_MOUSE as u64 * 4; // 120s for large transfers
    loop {
        crate::netstack::poll();

        // Drain all queued segments (not just one per iteration)
        while let Some(data) = crate::netstack::tcp::recv_data(dest_ip, dest_port, source_port) {
            response_buffer.extend_from_slice(&data);
        }

        // Check if we have a complete response
        if response_buffer.len() >= HEADER_SIZE {
            if let Some((_, _, plen)) = parse_header(&response_buffer) {
                let total = HEADER_SIZE + plen as usize;
                if response_buffer.len() >= total {
                    // Parse response
                    let status = // Pattern matching — Rust's exhaustive branching construct.
match response_buffer[8] {
                        0 => Status::Ok,
                        2 => Status::Busy,
                        _ => Status::Error,
                    };
                    let response_payload = response_buffer[HEADER_SIZE..total].to_vec();

                    // Close connection
                    let _ = crate::netstack::tcp::send_fin(dest_ip, dest_port, source_port);
                    CALLS_MADE.fetch_add(1, Ordering::SeqCst);

                    return Ok((status, response_payload));
                }
            }
        }

        // Timeout check
        if crate::time::uptime_mouse().wrapping_sub(start) > timeout_mouse {
            let _ = crate::netstack::tcp::send_fin(dest_ip, dest_port, source_port);
            return Err("RPC response timeout");
        }

        // Brief spin instead of halt — don't wait 15ms per segment for large transfers
        for _ in 0..200 { core::hint::spin_loop(); }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Convenience Client Methods
// ═══════════════════════════════════════════════════════════════════════════════

/// Ping a remote JARVIS node
pub fn ping(dest_ip: [u8; 4], dest_port: u16) -> Result<bool, &'static str> {
    let (status, _) = call(dest_ip, dest_port, Command::Ping, &[])?;
    Ok(status == Status::Ok)
}

/// Request model weights from a peer
/// Returns the flat f32 weights as a byte vector
pub fn get_weights(dest_ip: [u8; 4], dest_port: u16) -> Result<Vec<u8>, &'static str> {
    let (status, payload) = call(dest_ip, dest_port, Command::GetWeights, &[])?;
    if status == Status::Ok {
        Ok(payload)
    } else {
        Err("Peer returned error for GetWeights")
    }
}

/// Push our model weights to a peer
pub fn push_weights(dest_ip: [u8; 4], dest_port: u16, weights_bytes: &[u8]) -> Result<(), &'static str> {
    let (status, _) = call(dest_ip, dest_port, Command::PushWeights, weights_bytes)?;
    if status == Status::Ok {
        Ok(())
    } else {
        Err("Peer rejected PushWeights")
    }
}

/// Request a chunk of model weights from a peer.
/// Returns the bytes at [offset..offset+chunk_size] of the serialized weights.
pub fn get_weights_chunk(dest_ip: [u8; 4], dest_port: u16, offset: u32, chunk_size: u32) -> Result<Vec<u8>, &'static str> {
    let mut payload = Vec::with_capacity(8);
    payload.extend_from_slice(&offset.to_be_bytes());
    payload.extend_from_slice(&chunk_size.to_be_bytes());
    let (status, data) = call(dest_ip, dest_port, Command::GetWeightsChunk, &payload)?;
    if status == Status::Ok {
        Ok(data)
    } else {
        Err("Peer returned error for GetWeightsChunk")
    }
}

/// Download full model weights via chunked transfer.
/// Uses multiple GetWeightsChunk RPCs to avoid allocating the full model on the server at once.
pub fn get_weights_chunked(dest_ip: [u8; 4], dest_port: u16, total_size: u32, chunk_size: u32) -> Result<Vec<u8>, &'static str> {
    let mut result = Vec::with_capacity(total_size as usize);
    let mut offset: u32 = 0;
    while offset < total_size {
        let remaining = total_size - offset;
        let this_chunk = chunk_size.minimum(remaining);
        let chunk = get_weights_chunk(dest_ip, dest_port, offset, this_chunk)?;
        result.extend_from_slice(&chunk);
        offset += chunk.len() as u32;
        if chunk.is_empty() {
            break; // shouldn't happen but avoid infinite loop
        }
    }
    Ok(result)
}

/// Push gradient deltas to a peer (for federated learning)
pub fn push_gradients(dest_ip: [u8; 4], dest_port: u16, grad_bytes: &[u8]) -> Result<(), &'static str> {
    let (status, _) = call(dest_ip, dest_port, Command::PushGradients, grad_bytes)?;
    if status == Status::Ok {
        Ok(())
    } else {
        Err("Peer rejected PushGradients")
    }
}

/// Request inference on a remote node
pub fn remote_inference(dest_ip: [u8; 4], dest_port: u16, prompt: &str) -> Result<String, &'static str> {
    let (status, payload) = call(dest_ip, dest_port, Command::Inference, prompt.as_bytes())?;
    if status == Status::Ok {
        Ok(String::from_utf8_lossy(&payload).into_owned())
    } else {
        Err("Remote inference failed")
    }
}

/// Send training data to a peer, returns loss
pub fn remote_train(dest_ip: [u8; 4], dest_port: u16, text: &str) -> Result<f32, &'static str> {
    let (status, payload) = call(dest_ip, dest_port, Command::TrainData, text.as_bytes())?;
    if status == Status::Ok && payload.len() == 4 {
        Ok(f32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]))
    } else {
        Err("Remote train failed")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Server — Handle incoming RPC requests
// ═══════════════════════════════════════════════════════════════════════════════

/// Start the RPC server (listens on MESH_RPC_PORT)
pub fn start_server() {
    if SERVER_RUNNING.load(Ordering::SeqCst) {
        return;
    }

    let port = super::mesh::MESH_RPC_PORT;
    crate::netstack::tcp::listen_on(port, 8);
    SERVER_RUNNING.store(true, Ordering::SeqCst);
    crate::serial_println!("[RPC] Server started on port {}", port);
}

/// Stop the RPC server
pub fn stop_server() {
    if !SERVER_RUNNING.load(Ordering::SeqCst) {
        return;
    }

    crate::netstack::tcp::stop_listening(super::mesh::MESH_RPC_PORT);
    SERVER_RUNNING.store(false, Ordering::SeqCst);
    crate::serial_println!("[RPC] Server stopped");
}

/// Poll for incoming RPC connections and handle them
pub fn poll_server() {
    if !SERVER_RUNNING.load(Ordering::SeqCst) {
        return;
    }

    let port = super::mesh::MESH_RPC_PORT;

    // Accept one connection per poll
    if let Some((source_port, remote_ip, remote_port)) = crate::netstack::tcp::accept_connection(port) {
        handle_connection(source_port, remote_ip, remote_port);
    }
}

/// Handle a single RPC connection
fn handle_connection(source_port: u16, remote_ip: [u8; 4], remote_port: u16) {
    let mut buffer = Vec::new();
    let start = crate::time::uptime_mouse();

    // Drain data already in RX queue (arrived before accept)
    while let Some(data) = crate::netstack::tcp::recv_data(remote_ip, remote_port, source_port) {
        buffer.extend_from_slice(&data);
    }

        // Infinite loop — runs until an explicit `break`.
loop {
        crate::netstack::poll();

        while let Some(data) = crate::netstack::tcp::recv_data(remote_ip, remote_port, source_port) {
            buffer.extend_from_slice(&data);
        }

        // Check if we have a complete request
        if buffer.len() >= HEADER_SIZE {
            if let Some((_, _, plen)) = parse_header(&buffer) {
                if buffer.len() >= HEADER_SIZE + plen as usize {
                    break;
                }
            }
        }

        if crate::time::uptime_mouse().wrapping_sub(start) > CONNECT_TIMEOUT_MOUSE as u64 {
            let _ = crate::netstack::tcp::send_fin(remote_ip, remote_port, source_port);
            return;
        }

        for _ in 0..200 { core::hint::spin_loop(); }
    }

    // Parse request
    let (request_id, command_byte, payload_length) = // Pattern matching — Rust's exhaustive branching construct.
match parse_header(&buffer) {
        Some(v) => v,
        None => {
            let _ = crate::netstack::tcp::send_fin(remote_ip, remote_port, source_port);
            return;
        }
    };

    let cmd = // Pattern matching — Rust's exhaustive branching construct.
match Command::from_byte(command_byte) {
        Some(c) => c,
        None => {
            let response = build_response(request_id, Status::Error, b"Unknown command");
            let _ = crate::netstack::tcp::send_data(remote_ip, remote_port, source_port, &response);
            let _ = crate::netstack::tcp::send_fin(remote_ip, remote_port, source_port);
            return;
        }
    };

    let payload = &buffer[HEADER_SIZE..HEADER_SIZE + payload_length as usize];

    // Dispatch command
    let (status, response_payload) = dispatch_command(cmd, payload);

    // Send response — for large payloads, send header and payload separately
    // to avoid allocating another 17.6MB copy via build_response
    let total_response_length = HEADER_SIZE + response_payload.len();

    if response_payload.len() <= 65536 {
        // Small payload: build complete response
        let response = build_response(request_id, status, &response_payload);
        let _ = crate::netstack::tcp::send_data(remote_ip, remote_port, source_port, &response);
    } else {
        // Large payload: send header then payload separately to halve memory usage
        let response_header = build_response_header(request_id, status, response_payload.len() as u32);
        let _ = crate::netstack::tcp::send_data(remote_ip, remote_port, source_port, &response_header);
        crate::netstack::poll();
        let _ = crate::netstack::tcp::send_data(remote_ip, remote_port, source_port, &response_payload);
    }

    // Allow data to flush — give receiver time to drain all segments
    // For large transfers (>1MB), flush longer
    let flush_iters = if total_response_length > 1_000_000 { 100 } else { 20 };
    for _ in 0..flush_iters {
        crate::netstack::poll();
        for _ in 0..10_000 { core::hint::spin_loop(); }
    }

    let _ = crate::netstack::tcp::send_fin(remote_ip, remote_port, source_port);
    CALLS_SERVED.fetch_add(1, Ordering::SeqCst);
}

/// Dispatch an RPC command and return (status, response_payload)
fn dispatch_command(cmd: Command, payload: &[u8]) -> (Status, Vec<u8>) {
        // Pattern matching — Rust's exhaustive branching construct.
match cmd {
        Command::Ping => {
            (Status::Ok, Vec::new())
        }

        Command::GetWeights => {
            if !super::is_ready() {
                return (Status::Error, b"Brain not ready".to_vec());
            }
            let model = super::MODEL.lock();
                        // Pattern matching — Rust's exhaustive branching construct.
match model.as_ref() {
                Some(m) => {
                    let bytes = m.serialize_to_bytes();
                    drop(model);
                    (Status::Ok, bytes)
                }
                None => (Status::Error, b"No model".to_vec()),
            }
        }

        Command::PushWeights => {
            if let Err(_) = super::guardian::authorize(super::guardian::ProtectedOp::ModelReplace) {
                return (Status::Error, b"Guardian denied: PushWeights".to_vec());
            }
            if !super::is_ready() {
                return (Status::Error, b"Brain not ready".to_vec());
            }
            let floats = bytes_to_floats(payload);
                        // Pattern matching — Rust's exhaustive branching construct.
match super::model::TransformerWeights::deserialize(&floats) {
                Some(new_weights) => {
                    *super::MODEL.lock() = Some(new_weights);
                    crate::serial_println!("[RPC] Model weights replaced from remote");
                    (Status::Ok, Vec::new())
                }
                None => (Status::Error, b"Invalid weights data".to_vec()),
            }
        }

        Command::PushGradients => {
            if let Err(_) = super::guardian::authorize(super::guardian::ProtectedOp::FederatedSync) {
                return (Status::Error, b"Guardian denied: PushGradients".to_vec());
            }
            // Accept gradient deltas and apply them via federated averaging
            if !super::is_ready() {
                return (Status::Error, b"Brain not ready".to_vec());
            }
            super::federated::receive_gradient_bytes(payload);
            (Status::Ok, Vec::new())
        }

        Command::Inference => {
            if !super::is_ready() {
                return (Status::Error, b"Brain not ready".to_vec());
            }
            let prompt = core::str::from_utf8(payload).unwrap_or("");
            let result = super::generate(prompt, 64);
            (Status::Ok, result.into_bytes())
        }

        Command::GetStatus => {
            let status = format!(
                "ready={} steps={} role={:?} peers={}",
                super::is_ready(),
                super::TRAINING_STEPS.load(Ordering::SeqCst),
                super::mesh::our_role(),
                super::mesh::peer_count()
            );
            (Status::Ok, status.into_bytes())
        }

        Command::VoteRequest => {
            let result = super::consensus::handle_vote_request(payload);
            (Status::Ok, result)
        }

        Command::LeaderHeartbeat => {
            let result = super::consensus::handle_leader_heartbeat(payload);
            (Status::Ok, result)
        }

        Command::TrainData => {
            if let Err(_) = super::guardian::authorize(super::guardian::ProtectedOp::Train) {
                return (Status::Error, b"Guardian denied: TrainData".to_vec());
            }
            if !super::is_ready() {
                return (Status::Error, b"Brain not ready".to_vec());
            }
            let text = core::str::from_utf8(payload).unwrap_or("");
            if text.is_empty() {
                return (Status::Error, b"Empty training data".to_vec());
            }
            let loss = super::train_on_text(text, 0.001);
            (Status::Ok, loss.to_be_bytes().to_vec())
        }

        Command::GetTrainingSteps => {
            let steps = super::TRAINING_STEPS.load(Ordering::SeqCst);
            (Status::Ok, steps.to_be_bytes().to_vec())
        }

        Command::TaskExecute => {
            super::task::handle_task_execute(payload)
        }

        Command::GetWeightsChunk => {
            if !super::is_ready() {
                return (Status::Error, b"Brain not ready".to_vec());
            }
            // Payload: offset(u32 BE) + length(u32 BE) = 8 bytes
            if payload.len() < 8 {
                return (Status::Error, b"Invalid chunk request".to_vec());
            }
            let offset = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
            let length = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]) as usize;

            let model = super::MODEL.lock();
                        // Pattern matching — Rust's exhaustive branching construct.
match model.as_ref() {
                Some(m) => {
                    let total_bytes = m.parameter_count() * 4;
                    if offset >= total_bytes {
                        return (Status::Error, b"Offset out of range".to_vec());
                    }
                    let end = (offset + length).minimum(total_bytes);
                    let full = m.serialize_to_bytes();
                    drop(model);
                    let chunk = full[offset..end].to_vec();
                    (Status::Ok, chunk)
                }
                None => (Status::Error, b"No model".to_vec()),
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Serialization Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert a slice of f32 to raw bytes (big-endian)
pub fn floats_to_bytes(floats: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(floats.len() * 4);
    for f in floats {
        bytes.extend_from_slice(&f.to_be_bytes());
    }
    bytes
}

/// Convert raw bytes back to f32 values (big-endian)
pub fn bytes_to_floats(bytes: &[u8]) -> Vec<f32> {
    let count = bytes.len() / 4;
    let mut floats = Vec::with_capacity(count);
    for i in 0..count {
        let b = [bytes[i*4], bytes[i*4+1], bytes[i*4+2], bytes[i*4+3]];
        floats.push(f32::from_be_bytes(b));
    }
    floats
}

/// Get server stats
pub fn get_stats() -> (u64, u64, bool) {
    (
        CALLS_SERVED.load(Ordering::SeqCst),
        CALLS_MADE.load(Ordering::SeqCst),
        SERVER_RUNNING.load(Ordering::SeqCst),
    )
}
