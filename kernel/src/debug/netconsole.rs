//! UDP Netconsole — Stream kernel log output over the network
//!
//! Sends all `serial_println!` / `log!` output as UDP datagrams to a remote host.
//! This eliminates the need for a physical serial cable and allows real-time
//! debugging over Ethernet (e.g., 82566MM via e1000 driver).
//!
//! Usage from TrustOS shell:
//!   netconsole start 10.0.0.1        — Start streaming to 10.0.0.1:6666
//!   netconsole start 10.0.0.1 9999   — Start streaming to 10.0.0.1:9999
//!   netconsole stop                   — Stop streaming
//!   netconsole status                 — Show current config
//!
//! Listener on remote PC:
//!   ncat -u -l -p 6666               — or: socat UDP-LISTEN:6666 STDOUT

use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use spin::Mutex;

/// Whether netconsole is actively sending
static ENABLED: AtomicBool = AtomicBool::new(false);

/// Reentrance guard — prevents infinite recursion if UDP send itself logs
static IN_SEND: AtomicBool = AtomicBool::new(false);

/// Target IP address (network byte order stored as [u8; 4])
static TARGET_IP: Mutex<[u8; 4]> = Mutex::new([0u8; 4]);

/// Target UDP port
static TARGET_PORT: AtomicU16 = AtomicU16::new(6666);

/// Whether to use broadcast for sending (bypasses ARP)
static USE_BROADCAST: AtomicBool = AtomicBool::new(false);

/// Subnet broadcast IP (e.g., 10.0.0.255 for /24)
static BROADCAST_IP: Mutex<[u8; 4]> = Mutex::new([255u8; 4]);

/// Source UDP port (arbitrary high port)
const SRC_PORT: u16 = 6665;

/// Default target port
pub const DEFAULT_PORT: u16 = 6666;

/// Enable netconsole, sending to the given IP and port.
pub fn start(ip: [u8; 4], port: u16) {
    *TARGET_IP.lock() = ip;
    TARGET_PORT.store(port, Ordering::Relaxed);

    // Compute subnet broadcast IP from our config (or default to 255.255.255.255)
    // This lets us bypass ARP entirely — broadcast frames use FF:FF:FF:FF:FF:FF
    let bcast = if let Some((src_ip, mask, _)) = crate::network::get_ipv4_config() {
        let s = src_ip.as_bytes();
        let m = mask.as_bytes();
        [s[0] | !m[0], s[1] | !m[1], s[2] | !m[2], s[3] | !m[3]]
    } else {
        [255, 255, 255, 255]
    };
    *BROADCAST_IP.lock() = bcast;
    USE_BROADCAST.store(true, Ordering::Relaxed);

    ENABLED.store(true, Ordering::SeqCst);
    crate::serial_println!(
        "[netconsole] Streaming to {}.{}.{}.{}:{} (broadcast via {}.{}.{}.{})",
        ip[0], ip[1], ip[2], ip[3], port,
        bcast[0], bcast[1], bcast[2], bcast[3]
    );
}

/// Disable netconsole.
pub fn stop() {
    ENABLED.store(false, Ordering::SeqCst);
    crate::serial_println!("[netconsole] Stopped");
}

/// Check if netconsole is active.
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Get current target config.
pub fn status() -> ([u8; 4], u16, bool) {
    let ip = *TARGET_IP.lock();
    let port = TARGET_PORT.load(Ordering::Relaxed);
    let enabled = ENABLED.load(Ordering::Relaxed);
    (ip, port, enabled)
}

/// Send a log line over UDP. Called from `serial::_print()`.
///
/// This is fire-and-forget: if the network is down or the packet fails,
/// we silently drop it. Never panics, never blocks, never recurses.
pub fn send_line(msg: &str) {
    // Fast path: disabled → return immediately
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }

    // Prevent recursion: if the UDP/IP/ARP stack logs something, don't re-enter
    if IN_SEND.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
        return;
    }

    // Don't attempt before heap is available
    if crate::memory::heap::free() == 0 {
        IN_SEND.store(false, Ordering::Release);
        return;
    }

    // Use broadcast IP to bypass ARP (e1000 RX may not work after PXE handoff)
    let ip = if USE_BROADCAST.load(Ordering::Relaxed) {
        *BROADCAST_IP.lock()
    } else {
        *TARGET_IP.lock()
    };
    let port = TARGET_PORT.load(Ordering::Relaxed);

    // Truncate to fit in a single UDP datagram (no fragmentation)
    let payload = if msg.len() > 1400 { &msg[..1400] } else { msg };

    // Best-effort send — ignore all errors
    let _ = crate::netstack::udp::send_to(ip, port, SRC_PORT, payload.as_bytes());

    IN_SEND.store(false, Ordering::Release);
}

/// Parse an IP string like "10.0.0.1" into [u8; 4].
pub fn parse_ip(s: &str) -> Option<[u8; 4]> {
    let mut octets = [0u8; 4];
    let mut parts = s.split('.');
    for octet in &mut octets {
        let part = parts.next()?;
        *octet = part.trim().parse::<u8>().ok()?;
    }
    // Ensure no extra parts
    if parts.next().is_some() {
        return None;
    }
    Some(octets)
}
