//! Linux Distribution Manager
//!
//! Manages downloading and running Linux distributions in TrustOS.
//! Works with VirtualBox Host-Only networking (192.168.56.1 = host).

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};

/// Server configuration for VirtualBox Host-Only network
pub const SERVER_IP: [u8; 4] = [192, 168, 56, 1];
pub const SERVER_PORT: u16 = 8080;

/// Available Linux distribution
#[derive(Clone)]
pub struct LinuxDistro {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub size_mb: u32,
    pub filename: &'static str,
    pub icon: &'static str,
    pub installed: bool,
}

/// List of available distributions (hardcoded for now, could be fetched from server)
pub static DISTROS: Mutex<Vec<LinuxDistro>> = Mutex::new(Vec::new());

/// Initialize the distro list
pub fn init() {
    let mut distros = DISTROS.lock();
    distros.clear();
    
    // Minimal distros (small, fast to download)
    distros.push(LinuxDistro {
        id: "alpine",
        name: "Alpine Linux",
        version: "3.19",
        description: "Lightweight security-oriented Linux",
        size_mb: 3,
        filename: "alpine-minirootfs.tar.gz",
        icon: "🏔",
        installed: false,
    });
    
    distros.push(LinuxDistro {
        id: "busybox",
        name: "BusyBox",
        version: "1.36",
        description: "Minimal Unix utilities",
        size_mb: 1,
        filename: "busybox.tar.gz",
        icon: "📦",
        installed: false,
    });
    
    distros.push(LinuxDistro {
        id: "void-musl",
        name: "Void Linux (musl)",
        version: "2024",
        description: "Void Linux with musl libc",
        size_mb: 50,
        filename: "void-musl.tar.gz",
        icon: "⚫",
        installed: false,
    });
    
    // Full distros (larger)
    distros.push(LinuxDistro {
        id: "debian-mini",
        name: "Debian Minimal",
        version: "12",
        description: "Debian base system",
        size_mb: 150,
        filename: "debian-mini.tar.gz",
        icon: "🌀",
        installed: false,
    });
    
    distros.push(LinuxDistro {
        id: "ubuntu-core",
        name: "Ubuntu Core",
        version: "22.04",
        description: "Ubuntu minimal cloud image",
        size_mb: 200,
        filename: "ubuntu-core.tar.gz",
        icon: "🟠",
        installed: false,
    });
    
    distros.push(LinuxDistro {
        id: "arch-bootstrap",
        name: "Arch Linux",
        version: "2024.02",
        description: "Arch Linux bootstrap",
        size_mb: 180,
        filename: "arch-bootstrap.tar.gz",
        icon: "🔵",
        installed: false,
    });
    
    // Check which distros are already installed
    for distro in distros.iter_mut() {
        let path = format!("/opt/linux/{}", distro.filename);
        // Check if file exists in ramfs
        let exists = crate::ramfs::with_fs(|fs| {
            fs.stat(&path).is_ok()
        });
        if exists {
            distro.installed = true;
        }
    }
    
    crate::serial_println!("[DISTRO] Initialized {} distributions", distros.len());
}

/// Get list of all available distros
pub fn list() -> Vec<LinuxDistro> {
    DISTROS.lock().clone()
}

/// Get a specific distro by ID
pub fn get(id: &str) -> Option<LinuxDistro> {
    DISTROS.lock().iter().find(|d| d.id == id).cloned()
}

/// Check if a distro is installed
pub fn is_installed(id: &str) -> bool {
    DISTROS.lock().iter().find(|d| d.id == id).map(|d| d.installed).unwrap_or(false)
}

/// Mark a distro as installed
pub fn mark_installed(id: &str) {
    let mut distros = DISTROS.lock();
    if let Some(distro) = distros.iter_mut().find(|d| d.id == id) {
        distro.installed = true;
    }
}

/// Download a distribution from the server
pub fn download(id: &str) -> Result<usize, &'static str> {
    let distro = get(id).ok_or("Distribution not found")?;
    
    crate::serial_println!("[DISTRO] Downloading {} ({} MB)...", distro.name, distro.size_mb);
    
    // Suspend DHCP to prevent IP changes during download
    crate::netstack::dhcp::suspend();
    
    // ALWAYS force static IP for VirtualBox host-only network
    use crate::network::Ipv4Address;
    crate::serial_println!("[DISTRO] Setting static IP 192.168.56.100 for download");
    crate::network::set_ipv4_config(
        Ipv4Address::new(192, 168, 56, 100),
        Ipv4Address::new(255, 255, 255, 0),
        Some(Ipv4Address::new(192, 168, 56, 1)),
    );
    
    // Small delay to let things settle and clear any pending DHCP packets
    for _ in 0..200 {
        crate::netstack::poll();
    }
    
    // Connect to server
    let ip = SERVER_IP;
    let port = SERVER_PORT;
    
    let src_port = match crate::netstack::tcp::send_syn(ip, port) {
        Ok(p) => p,
        Err(e) => {
            crate::netstack::dhcp::resume();
            return Err(e);
        }
    };
    
    if !crate::netstack::tcp::wait_for_established(ip, port, src_port, 3000) {
        crate::netstack::dhcp::resume();
        return Err("Connection timeout - is the server running?");
    }
    
    // Send HTTP GET request
    let path = format!("/{}", distro.filename);
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: 192.168.56.1\r\nConnection: close\r\n\r\n",
        path
    );
    
    crate::netstack::tcp::send_payload(ip, port, src_port, request.as_bytes())?;
    crate::serial_println!("[DISTRO] Request sent, waiting for response...");
    
    // Receive data with aggressive polling
    let mut data: Vec<u8> = Vec::with_capacity((distro.size_mb as usize + 1) * 1024 * 1024);
    let start = crate::logger::get_ticks();
    let mut idle_count: u32 = 0;
    let mut last_ack_flush = start;
    let mut last_log = start;
    let max_size = (distro.size_mb as usize + 10) * 1024 * 1024;
    
    loop {
        // Aggressive polling - multiple times per iteration
        for _ in 0..10 {
            crate::netstack::poll();
        }
        
        let mut got_data = false;
        
        while let Some(chunk) = crate::netstack::tcp::recv_data(ip, port, src_port) {
            got_data = true;
            if data.len() + chunk.len() > max_size {
                break;
            }
            data.extend_from_slice(&chunk);
        }
        
        // Log progress periodically
        let now = crate::logger::get_ticks();
        if now.saturating_sub(last_log) >= 1000 {
            crate::serial_println!("[DISTRO] Downloaded {} KB...", data.len() / 1024);
            last_log = now;
        }
        
        // Flush ACKs more frequently (every 5ms)
        if now.saturating_sub(last_ack_flush) >= 5 {
            crate::netstack::tcp::flush_pending_acks(ip, port, src_port);
            last_ack_flush = now;
        }
        
        if !got_data {
            idle_count += 1;
            if crate::netstack::tcp::fin_received(ip, port, src_port) {
                crate::netstack::tcp::flush_pending_acks(ip, port, src_port);
                crate::serial_println!("[DISTRO] FIN received, download complete");
                break;
            }
            if idle_count > 200_000 {
                crate::serial_println!("[DISTRO] Idle timeout");
                crate::netstack::tcp::flush_pending_acks(ip, port, src_port);
                break;
            }
            // Minimal spin
            for _ in 0..10 { core::hint::spin_loop(); }
        } else {
            idle_count = 0;
        }
        
        // Timeout 120 seconds for large files
        if crate::logger::get_ticks().saturating_sub(start) > 120_000 {
            crate::netstack::dhcp::resume();
            return Err("Download timeout");
        }
    }
    
    let _ = crate::netstack::tcp::send_fin(ip, port, src_port);
    
    if data.is_empty() {
        crate::netstack::dhcp::resume();
        return Err("No data received");
    }
    
    // Extract HTTP body
    let body_start = data.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .unwrap_or(0);
    
    let body = &data[body_start..];
    
    if body.is_empty() {
        return Err("Empty response body");
    }
    
    // Save to ramfs
    let save_path = format!("/opt/linux/{}", distro.filename);
    
    crate::ramfs::with_fs(|fs| {
        let _ = fs.mkdir("/opt");
        let _ = fs.mkdir("/opt/linux");
        let _ = fs.touch(&save_path);
        fs.write_file(&save_path, body)
    }).map_err(|_| "Failed to save file")?;
    
    // Mark as installed
    mark_installed(id);
    
    // Resume DHCP
    crate::netstack::dhcp::resume();
    
    crate::serial_println!("[DISTRO] {} installed ({} KB)", distro.name, body.len() / 1024);
    
    Ok(body.len())
}

/// Run a Linux distribution (via hypervisor or emulation)
pub fn run(id: &str) -> Result<(), &'static str> {
    let distro = get(id).ok_or("Distribution not found")?;
    
    if !distro.installed {
        return Err("Distribution not installed - use 'distro install <id>' first");
    }
    
    crate::serial_println!("[DISTRO] Starting {} {}...", distro.name, distro.version);
    
    // Try hardware virtualization first
    if crate::hypervisor::is_enabled() {
        crate::serial_println!("[DISTRO] Using hardware virtualization");
        // TODO: Launch via hypervisor
        return Ok(());
    }
    
    // Fallback to emulation
    crate::serial_println!("[DISTRO] Hardware virtualization not available, using emulation");
    
    Ok(())
}
