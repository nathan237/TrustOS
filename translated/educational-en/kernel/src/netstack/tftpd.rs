//! TFTP Server Implementation (RFC 1350)
//!
//! Serves boot files for PXE network boot. TrustOS uses this to serve:
//! - limine-bios-pxe.bin (the PXE bootloader)
//! - limine.conf (boot configuration)
//! - trustos_kernel (the kernel binary itself, from Limine's kernel_file data)
//!
//! TFTP uses UDP port 69 for initial requests, then negotiates ephemeral ports.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

/// TFTP server running flag
static RUNNING: AtomicBool = AtomicBool::new(false);

/// Number of files served
static FILES_SERVED: AtomicU64 = AtomicU64::new(0);

/// Virtual filesystem: filename → data
static VIRTUAL_FILES: Mutex<BTreeMap<&'static str, &'static [u8]>> = Mutex::new(BTreeMap::new());

/// Active transfer sessions
static SESSIONS: Mutex<BTreeMap<u16, TransferSession>> = Mutex::new(BTreeMap::new());

/// Next ephemeral port for TFTP data transfer
static NEXT_TID: Mutex<u16> = Mutex::new(50000);

/// TFTP opcodes
mod opcode {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const RRQ: u16 = 1;   // Read request
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const WRQ: u16 = 2;   // Write request (we don't support)
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DATA: u16 = 3;  // Data packet
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ACK: u16 = 4;   // Acknowledgment
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ERROR: u16 = 5; // Error
}

/// TFTP error codes
mod error_code {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const NOT_FOUND: u16 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ACCESS_VIOLATION: u16 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const _DISK_FULL: u16 = 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ILLEGAL_OPERATION: u16 = 4;
}

/// TFTP block size
const BLOCK_SIZE: usize = 512;

/// A TFTP transfer session
struct TransferSession {
    /// Client IP
    client_ip: [u8; 4],
    /// Client port (TID)
    client_port: u16,
    /// Our local port for this transfer
    local_port: u16,
    /// File data being transferred
    file_data: &'static [u8],
    /// Current block number (1-based)
    current_block: u16,
    /// Total number of blocks
    total_blocks: u16,
    /// Whether transfer is complete
    complete: bool,
    /// Last send timestamp
    last_send: u64,
    /// Retry count
    retries: u8,
}

/// Check if TFTP server is running
pub fn is_running() -> bool {
    RUNNING.load(Ordering::Relaxed)
}

/// Get stats
pub fn files_served() -> u64 {
    FILES_SERVED.load(Ordering::Relaxed)
}

/// Register a virtual file for serving
pub fn register_file(name: &'static str, data: &'static [u8]) {
    let mut files = VIRTUAL_FILES.lock();
    files.insert(name, data);
    crate::serial_println!("[TFTPD] Registered file '{}' ({} bytes)", name, data.len());
}

/// Start the TFTP server
pub fn start() {
    if RUNNING.load(Ordering::Relaxed) {
        crate::serial_println!("[TFTPD] Already running");
        return;
    }

    RUNNING.store(true, Ordering::Relaxed);
    FILES_SERVED.store(0, Ordering::Relaxed);

    let files = VIRTUAL_FILES.lock();
    crate::serial_println!("[TFTPD] TFTP server started on port 69 ({} files registered)", files.len());
    for (name, data) in files.iter() {
        crate::serial_println!("[TFTPD]   {} ({} bytes, {} blocks)",
            name, data.len(), (data.len() + BLOCK_SIZE - 1) / BLOCK_SIZE);
    }
}

/// Stop the TFTP server
pub fn stop() {
    RUNNING.store(false, Ordering::Relaxed);
    let mut sessions = SESSIONS.lock();
    sessions.clear();
    crate::serial_println!("[TFTPD] Server stopped");
}

/// Handle incoming TFTP packet on port 69 (initial request port)
/// Called from UDP handler
pub fn handle_request_packet(data: &[u8], src_ip: [u8; 4], src_port: u16) {
    if !RUNNING.load(Ordering::Relaxed) || data.len() < 4 {
        return;
    }

    let opcode = u16::from_be_bytes([data[0], data[1]]);

        // Pattern matching — Rust's exhaustive branching construct.
match opcode {
        opcode::RRQ => handle_read_request(data, src_ip, src_port),
        opcode::WRQ => {
            // We don't support writes — send error
            send_error(src_ip, src_port, 69, error_code::ACCESS_VIOLATION, "Write not supported");
        }
        _ => {
            send_error(src_ip, src_port, 69, error_code::ILLEGAL_OPERATION, "Invalid opcode");
        }
    }
}

/// Handle incoming data on a TFTP transfer port (ACK from client)
pub fn handle_transfer_packet(data: &[u8], src_ip: [u8; 4], src_port: u16, local_port: u16) {
    if !RUNNING.load(Ordering::Relaxed) || data.len() < 4 {
        return;
    }

    let opcode = u16::from_be_bytes([data[0], data[1]]);

    if opcode == opcode::ACK {
        let block = u16::from_be_bytes([data[2], data[3]]);
        handle_acknowledge(src_ip, src_port, local_port, block);
    }
}

/// Handle a Read Request (RRQ)
fn handle_read_request(data: &[u8], client_ip: [u8; 4], client_port: u16) {
    // Parse filename from RRQ: opcode(2) + filename(null-terminated) + mode(null-terminated)
    let payload = &data[2..];

    // Find null terminator for filename
    let filename_end = // Pattern matching — Rust's exhaustive branching construct.
match payload.iter().position(|&b| b == 0) {
        Some(pos) => pos,
        None => {
            send_error(client_ip, client_port, 69, error_code::ILLEGAL_OPERATION, "Bad request");
            return;
        }
    };

    let filename = // Pattern matching — Rust's exhaustive branching construct.
match core::str::from_utf8(&payload[..filename_end]) {
        Ok(s) => s,
        Err(_) => {
            send_error(client_ip, client_port, 69, error_code::NOT_FOUND, "Invalid filename");
            return;
        }
    };

    // Strip leading slashes for lookup
    let clean_name = filename.trim_start_matches('/');

    crate::serial_println!("[TFTPD] RRQ for '{}' from {}.{}.{}.{}:{}",
        clean_name,
        client_ip[0], client_ip[1], client_ip[2], client_ip[3],
        client_port);

    // Look up file
    let files = VIRTUAL_FILES.lock();
    let file_data = // Pattern matching — Rust's exhaustive branching construct.
match files.get(clean_name) {
        Some(data) => *data,
        None => {
            // Try with different path variations
            let alt_name = if clean_name.starts_with("boot/") {
                &clean_name[5..]
            } else {
                clean_name
            };
                        // Pattern matching — Rust's exhaustive branching construct.
match files.get(alt_name) {
                Some(data) => *data,
                None => {
                    drop(files);
                    crate::serial_println!("[TFTPD] File not found: '{}'", clean_name);
                    send_error(client_ip, client_port, 69, error_code::NOT_FOUND, "File not found");
                    return;
                }
            }
        }
    };
    drop(files);

    // Allocate a local TID (port) for this transfer
    let local_port = {
        let mut tid = NEXT_TID.lock();
        let port = *tid;
        *tid = tid.wrapping_add(1);
        if *tid < 50000 { *tid = 50000; }
        port
    };

    let total_blocks = ((file_data.len() + BLOCK_SIZE - 1) / BLOCK_SIZE).max(1) as u16;

    crate::serial_println!("[TFTPD] Starting transfer: '{}' ({} bytes, {} blocks) TID={}",
        clean_name, file_data.len(), total_blocks, local_port);

    // Create session
    let session = TransferSession {
        client_ip,
        client_port,
        local_port,
        file_data,
        current_block: 1,
        total_blocks,
        complete: false,
        last_send: crate::time::uptime_ms(),
        retries: 0,
    };

    // Send first data block
    send_data_block(&session);

    // Store session
    let mut sessions = SESSIONS.lock();
    sessions.insert(local_port, session);
}

/// Handle an ACK from client
fn handle_acknowledge(client_ip: [u8; 4], client_port: u16, local_port: u16, block: u16) {
    let mut sessions = SESSIONS.lock();

    let session = // Pattern matching — Rust's exhaustive branching construct.
match sessions.get_mut(&local_port) {
        Some(s) => s,
        None => return,
    };

    // Verify it's from the right client
    if session.client_ip != client_ip || session.client_port != client_port {
        return;
    }

    if block == session.current_block {
        // Last block was received — advance
        session.current_block += 1;
        session.retries = 0;

        if session.current_block > session.total_blocks {
            // Transfer complete — check if we need to send a final empty block
            let last_block_size = session.file_data.len() % BLOCK_SIZE;
            if last_block_size == 0 && !session.file_data.is_empty() {
                // File size is exact multiple of 512 — send empty final block
                send_empty_final_block(session);
            }
            crate::serial_println!("[TFTPD] Transfer complete for TID={}", local_port);
            session.complete = true;
            FILES_SERVED.fetch_add(1, Ordering::Relaxed);
            let port = local_port;
            drop(sessions);
            cleanup_session(port);
            return;
        }

        // Send next block
        send_data_block(session);
        session.last_send = crate::time::uptime_ms();
    }
    // else: duplicate ACK, ignore
}

/// Send a data block
fn send_data_block(session: &TransferSession) {
    let block = session.current_block;
    let offset = ((block - 1) as usize) * BLOCK_SIZE;
    let end = (offset + BLOCK_SIZE).min(session.file_data.len());

    let data_slice = if offset < session.file_data.len() {
        &session.file_data[offset..end]
    } else {
        &[]
    };

    // Build TFTP DATA packet: opcode(2) + block#(2) + data(0-512)
    let mut packet = Vec::with_capacity(4 + data_slice.len());
    packet.extend_from_slice(&opcode::DATA.to_be_bytes());
    packet.extend_from_slice(&block.to_be_bytes());
    packet.extend_from_slice(data_slice);

    let _ = crate::netstack::udp::send_to(
        session.client_ip,
        session.client_port,
        session.local_port,
        &packet,
    );
}

/// Send empty final block (when file size is exact multiple of 512)
fn send_empty_final_block(session: &TransferSession) {
    let block = session.current_block;
    let mut packet = Vec::with_capacity(4);
    packet.extend_from_slice(&opcode::DATA.to_be_bytes());
    packet.extend_from_slice(&block.to_be_bytes());

    let _ = crate::netstack::udp::send_to(
        session.client_ip,
        session.client_port,
        session.local_port,
        &packet,
    );
}

/// Send a TFTP error packet
fn send_error(dest_ip: [u8; 4], dest_port: u16, src_port: u16, code: u16, msg: &str) {
    let message_bytes = msg.as_bytes();
    let mut packet = Vec::with_capacity(5 + message_bytes.len());
    packet.extend_from_slice(&opcode::ERROR.to_be_bytes());
    packet.extend_from_slice(&code.to_be_bytes());
    packet.extend_from_slice(message_bytes);
    packet.push(0); // null terminator

    let _ = crate::netstack::udp::send_to(dest_ip, dest_port, src_port, &packet);
}

/// Remove completed session
fn cleanup_session(port: u16) {
    let mut sessions = SESSIONS.lock();
    sessions.remove(&port);
}

/// Poll for retransmissions (call periodically)
pub fn poll() {
    if !RUNNING.load(Ordering::Relaxed) {
        return;
    }

    let now = crate::time::uptime_ms();
    let mut sessions = SESSIONS.lock();
    let mut to_remove = Vec::new();

    for (port, session) in sessions.iter_mut() {
        if session.complete {
            to_remove.push(*port);
            continue;
        }

        // Retransmit after 3 seconds
        if now.saturating_sub(session.last_send) > 3000 {
            if session.retries >= 5 {
                crate::serial_println!("[TFTPD] Transfer timeout for TID={}", port);
                to_remove.push(*port);
            } else {
                session.retries += 1;
                session.last_send = now;
                send_data_block(session);
                crate::serial_println!("[TFTPD] Retransmit block {} for TID={} (retry {})",
                    session.current_block, port, session.retries);
            }
        }
    }

    for port in to_remove {
        sessions.remove(&port);
    }
}

/// Get list of registered files
pub fn list_files() -> Vec<(&'static str, usize)> {
    let files = VIRTUAL_FILES.lock();
    files.iter().map(|(name, data)| (*name, data.len())).collect()
}

/// Get active transfer count
pub fn active_transfers() -> usize {
    let sessions = SESSIONS.lock();
    sessions.len()
}

/// Check if a specific port has an active TFTP transfer session
pub fn is_transfer_port(port: u16) -> bool {
    let sessions = SESSIONS.lock();
    sessions.contains_key(&port)
}
