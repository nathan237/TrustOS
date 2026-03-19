//! UDP Remote Shell — Bidirectional shell access over Ethernet
//!
//! Receives shell commands via UDP and sends back output.
//! Works alongside netconsole (which streams serial_println output).
//!
//! Protocol (simple text-based):
//!   PC → Board:  UDP port 7777, payload = shell command (UTF-8)
//!   Board → PC:  UDP reply to sender's IP:port, payload = command output
//!
//! Auto-starts during boot if a NIC driver is active.
//!
//! PC side:
//!   python scripts/remote_console.py --ip <board-ip>

use alloc::string::String;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

/// Whether remote shell is actively listening
static ENABLED: AtomicBool = AtomicBool::new(false);

/// Reentrance guard — prevents re-entering poll while executing a command
static IN_EXEC: AtomicBool = AtomicBool::new(false);

/// Own capture buffer (avoids conflict with shell's CAPTURE_MODE)
static REMOTE_BUF: Mutex<String> = Mutex::new(String::new());

/// Port to listen on
pub const LISTEN_PORT: u16 = 7777;

/// Source port for replies
const REPLY_SRC_PORT: u16 = 7778;

/// Maximum output size per response (fit in UDP without fragmentation)
const MAX_RESPONSE: usize = 1400;

/// Enable the remote shell listener.
pub fn start() {
    ENABLED.store(true, Ordering::SeqCst);
    crate::serial_println!("[remoteshell] Listening on UDP port {}", LISTEN_PORT);
}

/// Disable the remote shell listener.
pub fn stop() {
    ENABLED.store(false, Ordering::SeqCst);
    crate::serial_println!("[remoteshell] Stopped");
}

/// Check if remote shell is active.
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Poll for incoming commands. Called from netstack::poll().
///
/// Checks the UDP receive queue on LISTEN_PORT, executes any command found,
/// and sends the captured output back to the sender.
pub fn poll() {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }

    // Prevent re-entrance (execute_command may trigger netstack::poll indirectly)
    if IN_EXEC.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
        return;
    }

    // Check for incoming command
    let received = crate::netstack::udp::recv_from(LISTEN_PORT);
    let (data, src_ip, src_port) = match received {
        Some(d) => d,
        None => {
            IN_EXEC.store(false, Ordering::Release);
            return;
        }
    };

    // Parse command (UTF-8)
    let cmd = match core::str::from_utf8(&data) {
        Ok(s) => s.trim(),
        Err(_) => {
            IN_EXEC.store(false, Ordering::Release);
            return;
        }
    };

    if cmd.is_empty() {
        IN_EXEC.store(false, Ordering::Release);
        return;
    }

    crate::serial_println!(
        "[remoteshell] CMD from {}.{}.{}.{}:{}: {}",
        src_ip[0], src_ip[1], src_ip[2], src_ip[3], src_port, cmd
    );

    // Execute with output capture using our own buffer
    {
        let mut buf = REMOTE_BUF.lock();
        buf.clear();
    }
    crate::shell::CAPTURE_MODE.store(true, Ordering::SeqCst);
    crate::shell::execute_command(cmd);
    let output = crate::shell::take_captured();
    crate::shell::CAPTURE_MODE.store(false, Ordering::SeqCst);

    IN_EXEC.store(false, Ordering::Release);

    // Send output back (may need multiple packets for large output)
    let output_bytes = output.as_bytes();
    if output_bytes.is_empty() {
        // Send empty ACK so the PC knows the command was executed
        let _ = crate::netstack::udp::send_to(src_ip, src_port, REPLY_SRC_PORT, b"[ok]\n");
        return;
    }

    let mut offset = 0;
    while offset < output_bytes.len() {
        let end = core::cmp::min(offset + MAX_RESPONSE, output_bytes.len());
        let _ = crate::netstack::udp::send_to(src_ip, src_port, REPLY_SRC_PORT, &output_bytes[offset..end]);
        offset = end;
    }
}
