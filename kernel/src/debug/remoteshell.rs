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

/// Port to listen on for shell commands
pub const LISTEN_PORT: u16 = 7777;

/// Port for screencap requests (separate to avoid blocking commands)
pub const SCREENCAP_PORT: u16 = 7779;

/// Source port for replies (must match LISTEN_PORT so connected UDP sockets accept them)
const REPLY_SRC_PORT: u16 = 7777;

/// Maximum output size per response (fit in UDP without fragmentation)
const MAX_RESPONSE: usize = 1400;

/// Enable the remote shell listener.
pub fn start() {
    ENABLED.store(true, Ordering::SeqCst);
    crate::debug::watchdog_enable(10_000);
    // Arm TCO hardware watchdog: 17 steps × 0.6s = ~10s timeout
    // Survives triple faults, deadlocks, NMI storms — hardware reset
    crate::debug::tco_watchdog_init(17);
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

    // Pet the watchdog on every poll cycle (proves the main loop is alive)
    crate::debug::watchdog_pet();
    // Always-on APIC watchdog: if shell loop stops polling for >8s, ISA reboot.
    // Survives any silent hang in the kernel as long as APIC timer keeps firing.
    crate::apic::watchdog_kick(8_000);

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

    if cmd == "reboot" {
        IN_EXEC.store(false, Ordering::Release);
        let msg = b"REBOOTING NOW...\n";
        let _ = crate::netstack::udp::send_to(src_ip, src_port, REPLY_SRC_PORT, msg);
        crate::serial_println!("[remoteshell] Remote reboot requested!");
        crate::acpi::reboot();
    }

    // ── Input injection (Remote Desktop) ──
    // key:<ascii_byte>  — inject single key (decimal)
    // keys:<string>     — inject string of characters + Enter
    // mouse:<dx>,<dy>,<buttons>,<scroll>  — inject mouse event
    //   buttons: bit0=left, bit1=right, bit2=middle
    // mouseto:<x>,<y>,<buttons>,<scroll>  — move mouse to absolute position
    if let Some(val) = cmd.strip_prefix("key:") {
        let code = parse_u8(val);
        if code != 0 {
            crate::keyboard::push_key(code);
        }
        let _ = crate::netstack::udp::send_to(src_ip, src_port, REPLY_SRC_PORT, b"[ok]\n");
        IN_EXEC.store(false, Ordering::Release);
        return;
    }
    if let Some(text) = cmd.strip_prefix("keys:") {
        for c in text.bytes() {
            crate::keyboard::push_key(c);
        }
        let _ = crate::netstack::udp::send_to(src_ip, src_port, REPLY_SRC_PORT, b"[ok]\n");
        IN_EXEC.store(false, Ordering::Release);
        return;
    }
    if let Some(args) = cmd.strip_prefix("mouse:") {
        // Format: dx,dy,buttons,scroll
        let parts: [i32; 4] = parse_4_ints(args);
        let left = (parts[2] & 1) != 0;
        let right = (parts[2] & 2) != 0;
        let middle = (parts[2] & 4) != 0;
        crate::mouse::inject_usb_mouse(parts[0], parts[1], left, right, middle, parts[3] as i8);
        let _ = crate::netstack::udp::send_to(src_ip, src_port, REPLY_SRC_PORT, b"[ok]\n");
        IN_EXEC.store(false, Ordering::Release);
        return;
    }
    if let Some(args) = cmd.strip_prefix("mouseto:") {
        // Format: x,y,buttons,scroll — absolute position
        let parts: [i32; 4] = parse_4_ints(args);
        let (cx, cy) = crate::mouse::get_position();
        let dx = parts[0] - cx;
        let dy = parts[1] - cy;
        let left = (parts[2] & 1) != 0;
        let right = (parts[2] & 2) != 0;
        let middle = (parts[2] & 4) != 0;
        crate::mouse::inject_usb_mouse(dx, dy, left, right, middle, parts[3] as i8);
        let _ = crate::netstack::udp::send_to(src_ip, src_port, REPLY_SRC_PORT, b"[ok]\n");
        IN_EXEC.store(false, Ordering::Release);
        return;
    }
    if cmd == "mousepos" {
        let (x, y) = crate::mouse::get_position();
        let msg = alloc::format!("{},{}\n", x, y);
        let _ = crate::netstack::udp::send_to(src_ip, src_port, REPLY_SRC_PORT, msg.as_bytes());
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
        crate::serial_println!(
            "[remoteshell][reply] {}.{}.{}.{}:{} => [ok]",
            src_ip[0], src_ip[1], src_ip[2], src_ip[3], src_port
        );
        match crate::netstack::udp::send_to(src_ip, src_port, REPLY_SRC_PORT, b"[ok]\n") {
            Ok(()) => crate::serial_println!("[remoteshell] ACK sent to {}.{}.{}.{}:{}", src_ip[0], src_ip[1], src_ip[2], src_ip[3], src_port),
            Err(e) => crate::serial_println!("[remoteshell] ACK SEND FAILED: {}", e),
        }
        return;
    }

    crate::serial_println!(
        "[remoteshell][reply] {} bytes to {}.{}.{}.{}:{}",
        output_bytes.len(),
        src_ip[0], src_ip[1], src_ip[2], src_ip[3], src_port
    );
    for line in output.lines() {
        let line = line.trim_end_matches('\r');
        if !line.is_empty() {
            crate::serial_println!("[remoteshell][reply] {}", line);
        }
    }

    let mut offset = 0;
    while offset < output_bytes.len() {
        let end = core::cmp::min(offset + MAX_RESPONSE, output_bytes.len());
        match crate::netstack::udp::send_to(src_ip, src_port, REPLY_SRC_PORT, &output_bytes[offset..end]) {
            Ok(()) => crate::serial_println!("[remoteshell] Reply chunk {}/{} sent", end, output_bytes.len()),
            Err(e) => crate::serial_println!("[remoteshell] SEND FAILED: {}", e),
        }
        offset = end;
    }
}

/// Poll for screencap requests on SCREENCAP_PORT (7779).
/// Completely independent from command processing — no IN_EXEC guard.
pub fn poll_screencap() {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }

    let received = crate::netstack::udp::recv_from(SCREENCAP_PORT);
    let (data, src_ip, src_port) = match received {
        Some(d) => d,
        None => return,
    };

    let cmd = match core::str::from_utf8(&data) {
        Ok(s) => s.trim(),
        Err(_) => return,
    };

    if cmd == "screencap" || cmd == "screenshot" {
        send_screencap(src_ip, src_port);
    } else if let Some(indices_str) = cmd.strip_prefix("resend:") {
        resend_chunks(src_ip, src_port, indices_str);
    }
}

/// Send framebuffer screenshot as binary UDP chunks.
///
/// Protocol:
///   Packet 0 (header): b"SCRN" + [width:u16 LE, height:u16 LE, bpp:u16 LE, total_chunks:u16 LE, pitch:u32 LE]
///   Packets 1..N (data): [chunk_index:u16 LE] + raw BGRA pixel data (up to 1398 bytes)
///
/// Port: replies on REPLY_SRC_PORT (7778) to caller's port
fn send_screencap(dst_ip: [u8; 4], dst_port: u16) {
    use core::sync::atomic::Ordering::Relaxed;

    let width = crate::framebuffer::FB_WIDTH.load(Relaxed) as u32;
    let height = crate::framebuffer::FB_HEIGHT.load(Relaxed) as u32;
    let pitch = crate::framebuffer::FB_PITCH.load(Relaxed) as u32;
    let bpp = crate::framebuffer::FB_BPP.load(Relaxed) as u32;
    let fb_ptr = crate::framebuffer::FB_ADDR.load(Relaxed);

    if fb_ptr.is_null() || width == 0 || height == 0 {
        let _ = crate::netstack::udp::send_to(dst_ip, dst_port, REPLY_SRC_PORT, b"[error] no framebuffer\n");
        return;
    }

    let bytes_per_pixel = (bpp / 8) as usize;
    let row_bytes = width as usize * bytes_per_pixel;
    let total_bytes = row_bytes * height as usize;

    const CHUNK_DATA: usize = 1396; // 1400 - 4 bytes header (2 chunk_index + 2 reserved)
    let total_chunks = (total_bytes + CHUNK_DATA - 1) / CHUNK_DATA;

    // Send header packet: "SCRN" + metadata
    let mut hdr = [0u8; 16];
    hdr[0..4].copy_from_slice(b"SCRN");
    hdr[4..6].copy_from_slice(&(width as u16).to_le_bytes());
    hdr[6..8].copy_from_slice(&(height as u16).to_le_bytes());
    hdr[8..10].copy_from_slice(&(bpp as u16).to_le_bytes());
    hdr[10..12].copy_from_slice(&(total_chunks as u16).to_le_bytes());
    hdr[12..16].copy_from_slice(&pitch.to_le_bytes());
    let _ = crate::netstack::udp::send_to(dst_ip, dst_port, REPLY_SRC_PORT, &hdr);

    // Small delay to let header arrive first
    for _ in 0..5000 { core::hint::spin_loop(); }

    // Send pixel data chunks
    let mut buf = [0u8; 1400]; // 4 header + 1396 data
    for chunk_idx in 0..total_chunks {
        let data_offset = chunk_idx * CHUNK_DATA;
        let data_len = core::cmp::min(CHUNK_DATA, total_bytes - data_offset);

        // Header: chunk index
        buf[0..2].copy_from_slice(&(chunk_idx as u16).to_le_bytes());
        buf[2..4].copy_from_slice(&(data_len as u16).to_le_bytes());

        // Copy pixel data from framebuffer row by row (pitch may differ from width*bpp)
        let mut copied = 0;
        while copied < data_len {
            let global_byte = data_offset + copied;
            let row = global_byte / row_bytes;
            let col_byte = global_byte % row_bytes;
            let remaining_in_row = row_bytes - col_byte;
            let to_copy = core::cmp::min(data_len - copied, remaining_in_row);

            let src_offset = row * pitch as usize + col_byte;
            unsafe {
                core::ptr::copy_nonoverlapping(
                    fb_ptr.add(src_offset),
                    buf.as_mut_ptr().add(4 + copied),
                    to_copy,
                );
            }
            copied += to_copy;
        }

        let _ = crate::netstack::udp::send_to(dst_ip, dst_port, REPLY_SRC_PORT, &buf[..4 + data_len]);

        // Brief pause every 32 packets to avoid overwhelming the TX ring
        if chunk_idx % 32 == 31 {
            for _ in 0..3000 { core::hint::spin_loop(); }
        }
    }
}

/// Resend specific chunks by index. Command format: "resend:0,5,12,45"
fn resend_chunks(dst_ip: [u8; 4], dst_port: u16, indices_str: &str) {
    use core::sync::atomic::Ordering::Relaxed;

    let width = crate::framebuffer::FB_WIDTH.load(Relaxed) as u32;
    let height = crate::framebuffer::FB_HEIGHT.load(Relaxed) as u32;
    let pitch = crate::framebuffer::FB_PITCH.load(Relaxed) as u32;
    let bpp = crate::framebuffer::FB_BPP.load(Relaxed) as u32;
    let fb_ptr = crate::framebuffer::FB_ADDR.load(Relaxed);

    if fb_ptr.is_null() || width == 0 || height == 0 {
        return;
    }

    let bytes_per_pixel = (bpp / 8) as usize;
    let row_bytes = width as usize * bytes_per_pixel;
    let total_bytes = row_bytes * height as usize;

    const CHUNK_DATA: usize = 1396;
    let total_chunks = (total_bytes + CHUNK_DATA - 1) / CHUNK_DATA;

    let mut buf = [0u8; 1400];
    let mut sent = 0u32;

    for part in indices_str.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() { continue; }

        // Parse chunk index
        let mut idx: usize = 0;
        let mut valid = false;
        for b in trimmed.bytes() {
            if b >= b'0' && b <= b'9' {
                idx = idx * 10 + (b - b'0') as usize;
                valid = true;
            } else {
                valid = false;
                break;
            }
        }
        if !valid || idx >= total_chunks { continue; }

        let data_offset = idx * CHUNK_DATA;
        let data_len = core::cmp::min(CHUNK_DATA, total_bytes - data_offset);

        buf[0..2].copy_from_slice(&(idx as u16).to_le_bytes());
        buf[2..4].copy_from_slice(&(data_len as u16).to_le_bytes());

        let mut copied = 0;
        while copied < data_len {
            let global_byte = data_offset + copied;
            let row = global_byte / row_bytes;
            let col_byte = global_byte % row_bytes;
            let remaining_in_row = row_bytes - col_byte;
            let to_copy = core::cmp::min(data_len - copied, remaining_in_row);

            let src_offset = row * pitch as usize + col_byte;
            unsafe {
                core::ptr::copy_nonoverlapping(
                    fb_ptr.add(src_offset),
                    buf.as_mut_ptr().add(4 + copied),
                    to_copy,
                );
            }
            copied += to_copy;
        }

        let _ = crate::netstack::udp::send_to(dst_ip, dst_port, REPLY_SRC_PORT, &buf[..4 + data_len]);
        sent += 1;

        // Pace retransmissions
        if sent % 32 == 0 {
            for _ in 0..3000 { core::hint::spin_loop(); }
        }
    }
}

/// Parse a decimal u8 from a string (e.g. "65" → 65, "0" → 0)
fn parse_u8(s: &str) -> u8 {
    let mut val: u16 = 0;
    for b in s.trim().bytes() {
        if b >= b'0' && b <= b'9' {
            val = val * 10 + (b - b'0') as u16;
        } else {
            break;
        }
    }
    if val > 255 { 0 } else { val as u8 }
}

/// Parse 4 comma-separated signed integers: "dx,dy,buttons,scroll"
fn parse_4_ints(s: &str) -> [i32; 4] {
    let mut result = [0i32; 4];
    let mut idx = 0;
    for part in s.split(',') {
        if idx >= 4 { break; }
        let trimmed = part.trim();
        let (neg, digits) = if let Some(rest) = trimmed.strip_prefix('-') {
            (true, rest)
        } else {
            (false, trimmed)
        };
        let mut val: i32 = 0;
        for b in digits.bytes() {
            if b >= b'0' && b <= b'9' {
                val = val * 10 + (b - b'0') as i32;
            } else {
                break;
            }
        }
        result[idx] = if neg { -val } else { val };
        idx += 1;
    }
    result
}
