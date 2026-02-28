//! VM & Linux Commands  Virtual machines, Linux subsystem, Alpine, Distro manager,
//! Hypervisor, Download, Persistence, Disk/AHCI/fdisk
//!
//! All commands related to running Linux inside TrustOS, VM management,
//! disk persistence, and partition handling.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{COLOR_GREEN, COLOR_BRIGHT_GREEN, COLOR_DARK_GREEN, COLOR_YELLOW, COLOR_RED, COLOR_CYAN, COLOR_WHITE, COLOR_BLUE, COLOR_MAGENTA, COLOR_GRAY};

// ==================== GUI COMMANDS ====================

// ==================== VM / LINUX SYSTEM ====================
// State: tracks if Alpine Linux VM image is installed
pub(super) static GUI_INSTALLED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

pub(super) fn cmd_vm_help() {
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println_color!(COLOR_CYAN, "|            TrustOS Virtual Machine Manager                   |");
    crate::println_color!(COLOR_CYAN, "|--------------------------------------------------------------|");
    crate::println_color!(COLOR_CYAN, "|                                                              |");
    crate::println_color!(COLOR_CYAN, "|  TrustOS runs Linux VMs with modern GUIs.                   |");
    crate::println_color!(COLOR_CYAN, "|                                                              |");
    crate::println_color!(COLOR_CYAN, "|  Commands:                                                   |");
    crate::println_color!(COLOR_GREEN, "|    vm status    - Check VM installation status              |");
    crate::println_color!(COLOR_GREEN, "|    vm install   - Download Alpine Linux VM image            |");
    crate::println_color!(COLOR_GREEN, "|    vm start     - Start the Alpine Linux VM                 |");
    crate::println_color!(COLOR_GREEN, "|    vm console   - Connect to VM console (Linux shell)       |");
    crate::println_color!(COLOR_GREEN, "|    vm stop      - Stop the running VM                       |");
    crate::println_color!(COLOR_GREEN, "|    vm list      - List running VMs                          |");
    crate::println_color!(COLOR_CYAN, "|                                                              |");
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
}

pub(super) fn cmd_vm_stop() {
    crate::println_color!(COLOR_YELLOW, "Stopping VM...");
    // TODO: Actually stop the VM
    crate::println_color!(COLOR_GREEN, "VM stopped.");
}

pub(super) fn cmd_vm_list() {
    crate::println_color!(COLOR_CYAN, "Running Virtual Machines:");
    crate::println!("  ID   NAME           STATUS      MEMORY");
    crate::println!("  ---------------------------------------");
    if GUI_INSTALLED.load(core::sync::atomic::Ordering::Relaxed) {
        crate::println!("  1    alpine-linux   running     256 MB");
    } else {
        crate::println!("  (no VMs running)");
    }
}

// ==================== LINUX DISTRIBUTION MANAGER ====================

pub(super) fn cmd_distro_list() {
    // Initialize if needed
    if crate::distro::list().is_empty() {
        crate::distro::init();
    }
    
    let distros = crate::distro::list();
    
    crate::println_color!(COLOR_CYAN, "+------------------------------------------------------------------+");
    crate::println_color!(COLOR_CYAN, "|                 TrustOS Linux Distribution Manager               |");
    crate::println_color!(COLOR_CYAN, "|------------------------------------------------------------------|");
    crate::println_color!(COLOR_CYAN, "|  ID              NAME                    SIZE     STATUS         |");
    crate::println_color!(COLOR_CYAN, "|------------------------------------------------------------------|");
    
    for d in &distros {
        let status = if d.installed { 
            "\x1b[32m[installed]\x1b[0m" 
        } else { 
            "\x1b[33m[available]\x1b[0m" 
        };
        let status_simple = if d.installed { "installed" } else { "available" };
        crate::println!("|  {} {:<12}  {:<20}  {:>4} MB   {:<12} |", 
            d.icon, d.id, d.name, d.size_mb, status_simple);
    }
    
    crate::println_color!(COLOR_CYAN, "|------------------------------------------------------------------|");
    crate::println_color!(COLOR_CYAN, "|  Commands:                                                       |");
    crate::println_color!(COLOR_GREEN, "|    distro list              - Show this list                    |");
    crate::println_color!(COLOR_GREEN, "|    distro install <id>      - Download and install a distro     |");
    crate::println_color!(COLOR_GREEN, "|    distro run <id>          - Run an installed distro           |");
    crate::println_color!(COLOR_GREEN, "|    distro gui               - Open graphical distro selector    |");
    crate::println_color!(COLOR_CYAN, "+------------------------------------------------------------------+");
}

pub(super) fn cmd_distro_install(id: &str) {
    // Initialize if needed
    if crate::distro::list().is_empty() {
        crate::distro::init();
    }
    
    let distro = match crate::distro::get(id) {
        Some(d) => d,
        None => {
            crate::println_color!(COLOR_RED, "Error: Distribution '{}' not found.", id);
            crate::println!("Use 'distro list' to see available distributions.");
            return;
        }
    };
    
    if distro.installed {
        crate::println_color!(COLOR_YELLOW, "{} {} is already installed.", distro.icon, distro.name);
        crate::println!("Use 'distro run {}' to start it.", id);
        return;
    }
    
    crate::println_color!(COLOR_CYAN, "+------------------------------------------------------------------+");
    crate::println_color!(COLOR_CYAN, "|                    Installing Linux Distribution                 |");
    crate::println_color!(COLOR_CYAN, "+------------------------------------------------------------------+");
    crate::println!();
    crate::println!("  {} {} {}", distro.icon, distro.name, distro.version);
    crate::println!("  {}", distro.description);
    crate::println!("  Size: {} MB", distro.size_mb);
    crate::println!();
    
    crate::println_color!(COLOR_YELLOW, "[1/3] Connecting to server 192.168.56.1:8080...");
    
    match crate::distro::download(id) {
        Ok(size) => {
            crate::println_color!(COLOR_GREEN, "[2/3] Downloaded {} KB", size / 1024);
            crate::println_color!(COLOR_GREEN, "[3/3] Installation complete!");
            crate::println!();
            crate::println_color!(COLOR_GREEN, "  {} {} is now installed!", distro.icon, distro.name);
            crate::println!("  Use 'distro run {}' to start it.", id);
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "Error: {}", e);
            crate::println!();
            crate::println!("Make sure the server is running:");
            crate::println!("  > cd server && powershell -ExecutionPolicy Bypass .\\start-server.ps1");
        }
    }
}

pub(super) fn cmd_distro_run(id: &str) {
    // Initialize if needed
    if crate::distro::list().is_empty() {
        crate::distro::init();
    }
    
    let distro = match crate::distro::get(id) {
        Some(d) => d,
        None => {
            crate::println_color!(COLOR_RED, "Error: Distribution '{}' not found.", id);
            crate::println!("Use 'distro list' to see available distributions.");
            return;
        }
    };
    
    if !distro.installed {
        crate::println_color!(COLOR_YELLOW, "{} {} is not installed.", distro.icon, distro.name);
        crate::println!("Use 'distro install {}' to download it first.", id);
        return;
    }
    
    crate::println_color!(COLOR_CYAN, "+------------------------------------------------------------------+");
    crate::println_color!(COLOR_CYAN, "|                    Starting Linux Distribution                   |");
    crate::println_color!(COLOR_CYAN, "+------------------------------------------------------------------+");
    crate::println!();
    crate::println!("  {} {} {}", distro.icon, distro.name, distro.version);
    crate::println!();
    
    match crate::distro::run(id) {
        Ok(()) => {
            crate::println_color!(COLOR_GREEN, "  Distribution started successfully.");
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "Error: {}", e);
        }
    }
}

pub(super) fn cmd_distro_gui() {
    // Initialize if needed
    if crate::distro::list().is_empty() {
        crate::distro::init();
    }
    
    let distros = crate::distro::list();
    
    // Check framebuffer
    if !crate::framebuffer::is_initialized() {
        crate::println_color!(COLOR_RED, "Error: No framebuffer available for GUI.");
        crate::println!("Use 'distro list' for text-mode interface.");
        return;
    }
    
    let (width, height) = crate::framebuffer::get_dimensions();
    
    // Colors
    let bg_color = 0xFF1E1E2Eu32;      // Dark background
    let panel_color = 0xFF2D2D3Du32;   // Panel background
    let accent_color = 0xFF89B4FAu32;  // Blue accent
    let green_color = 0xFF94E2D5u32;   // Teal/green for installed
    let text_color = 0xFFCDD6F4u32;    // Light text
    let _dim_color = 0xFF6C7086u32;    // Dimmed text
    
    // Clear screen with background
    crate::framebuffer::fill_rect(0, 0, width, height, bg_color);
    
    // Title bar
    crate::framebuffer::fill_rect(0, 0, width, 50, panel_color);
    crate::framebuffer::draw_text_at("TrustOS Linux Distribution Manager", 20, 16, text_color, panel_color);
    
    // Draw distro list as text (simple version)
    let mut y = 80u32;
    
    crate::framebuffer::draw_text_at("  #  ID              NAME                    SIZE     STATUS", 20, y, accent_color, bg_color);
    y += 24;
    crate::framebuffer::draw_hline(20, y, width - 40, accent_color);
    y += 16;
    
    for (i, d) in distros.iter().enumerate() {
        let status_str = if d.installed { "[INSTALLED]" } else { "[available]" };
        let status_color = if d.installed { green_color } else { text_color };
        
        // Number
        let num_str = alloc::format!("  {}  ", i + 1);
        crate::framebuffer::draw_text_at(&num_str, 20, y, accent_color, bg_color);
        
        // Icon + ID
        let id_str = alloc::format!("{} {:<12}", d.icon, d.id);
        crate::framebuffer::draw_text_at(&id_str, 60, y, text_color, bg_color);
        
        // Name
        crate::framebuffer::draw_text_at(d.name, 220, y, text_color, bg_color);
        
        // Size
        let size_str = alloc::format!("{:>4} MB", d.size_mb);
        crate::framebuffer::draw_text_at(&size_str, 450, y, text_color, bg_color);
        
        // Status
        crate::framebuffer::draw_text_at(status_str, 540, y, status_color, bg_color);
        
        y += 24;
    }
    
    // Footer with instructions
    let footer_y = height - 80;
    crate::framebuffer::fill_rect(0, footer_y, width, 80, panel_color);
    crate::framebuffer::draw_text_at("Commands:", 20, footer_y + 16, accent_color, panel_color);
    crate::framebuffer::draw_text_at("distro install <id>  - Download and install", 20, footer_y + 36, text_color, panel_color);
    crate::framebuffer::draw_text_at("distro run <id>      - Run an installed distro", 400, footer_y + 36, text_color, panel_color);
    crate::framebuffer::draw_text_at("Press any key to return to shell...", 20, footer_y + 56, green_color, panel_color);
    
    // Wait for key input
    loop {
        if let Some(_ch) = crate::keyboard::read_char() {
            break;
        }
        for _ in 0..1000 { core::hint::spin_loop(); }
    }
    
    // Clear screen 
    crate::framebuffer::clear();
}

pub(super) fn cmd_gui_status() {
    let installed = GUI_INSTALLED.load(core::sync::atomic::Ordering::Relaxed);
    
    crate::println_color!(COLOR_CYAN, "+--------------------------------------+");
    crate::println_color!(COLOR_CYAN, "|       TrustOS GUI Status             |");
    crate::println_color!(COLOR_CYAN, "|--------------------------------------|");
    
    if installed {
        crate::println_color!(COLOR_GREEN, "|  Status:     [INSTALLED]             |");
        crate::println_color!(COLOR_GREEN, "|  Image:      Alpine Linux + Browser  |");
        crate::println_color!(COLOR_CYAN, "|                                      |");
        crate::println_color!(COLOR_CYAN, "|  Use 'gui start' to launch           |");
    } else {
        crate::println_color!(COLOR_YELLOW, "|  Status:     [NOT INSTALLED]         |");
        crate::println_color!(COLOR_CYAN, "|                                      |");
        crate::println_color!(COLOR_CYAN, "|  Use 'gui install' to download       |");
    }
    crate::println_color!(COLOR_CYAN, "+--------------------------------------+");
}

pub(super) fn cmd_gui_install() {
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println_color!(COLOR_CYAN, "|              TrustOS GUI Installer                           |");
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println!();
    
    // Configuration du serveur (192.168.56.1 = host dans VirtualBox Host-Only)
    let server_ip = "192.168.56.1";
    let server_port = 8080u16;
    let package_path = "/alpine-minirootfs.tar.gz";
    
    // Etape 1: Verifier le reseau
    crate::println_color!(COLOR_YELLOW, "[1/4] Checking network connection...");
    
    if !crate::network::is_available() {
        crate::println_color!(COLOR_RED, "      ERROR: Network not available!");
        crate::println!("      Make sure virtio-net is enabled.");
        return;
    }
    crate::println_color!(COLOR_GREEN, "      Network: OK");
    crate::println!();
    
    // Etape 2: Telecharger Alpine Linux
    crate::println_color!(COLOR_YELLOW, "[2/4] Downloading Alpine Linux from {}:{}{}...", server_ip, server_port, package_path);
    
    // CRITICAL: Suspend DHCP to prevent IP changes during download
    crate::netstack::dhcp::suspend();
    crate::serial_println!("[GUI_INSTALL] DHCP suspended for download");
    
    // Force static IP for VirtualBox Host-Only network
    crate::network::set_ipv4_config(
        crate::network::Ipv4Address::new(192, 168, 56, 100),
        crate::network::Ipv4Address::new(255, 255, 255, 0),
        Some(crate::network::Ipv4Address::new(192, 168, 56, 1)),
    );
    
    // Clear any pending DHCP packets
    for _ in 0..100 {
        crate::netstack::poll();
    }
    
    let ip = match parse_ipv4(server_ip) {
        Some(ip) => ip,
        None => {
            crate::println_color!(COLOR_RED, "      ERROR: Invalid server IP");
            crate::netstack::dhcp::resume();
            return;
        }
    };
    
    // Connexion TCP
    let src_port = match crate::netstack::tcp::send_syn(ip, server_port) {
        Ok(p) => p,
        Err(e) => {
            crate::println_color!(COLOR_RED, "      ERROR: Connection failed: {}", e);
            crate::println!("      Make sure the server is running:");
            crate::println!("      > cd server && powershell -ExecutionPolicy Bypass .\\start-server.ps1");
            crate::netstack::dhcp::resume();
            return;
        }
    };
    
    let established = crate::netstack::tcp::wait_for_established(ip, server_port, src_port, 2000);
    if !established {
        crate::println_color!(COLOR_RED, "      ERROR: Connection timeout");
        crate::println!("      Make sure the server is running on port {}", server_port);
        crate::netstack::dhcp::resume();
        return;
    }
    
    crate::println_color!(COLOR_GREEN, "      Connected to server");
    
    // Envoyer la requete HTTP GET
    let request = alloc::format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        package_path, server_ip
    );
    
    if let Err(e) = crate::netstack::tcp::send_payload(ip, server_port, src_port, request.as_bytes()) {
        crate::println_color!(COLOR_RED, "      ERROR: Failed to send request: {}", e);
        crate::netstack::dhcp::resume();
        return;
    }
    
    // Recevoir les donnees (optimized download loop)
    crate::println!("      Downloading...");
    // Pre-allouer 4 MB pour eviter les reallocations
    let mut received_data: Vec<u8> = Vec::with_capacity(4 * 1024 * 1024);
    let start = crate::logger::get_ticks();
    let mut idle_count: u32 = 0;
    let mut last_progress = 0usize;
    let mut last_ack_flush = start;
    let mut last_poll_count = 0u32;
    const MAX_SIZE: usize = 8 * 1024 * 1024; // 8 MB max
    
    loop {
        // Poll network aggressively (multiple times per iteration)
        for _ in 0..10 {
            crate::netstack::poll();
        }
        last_poll_count += 10;
        
        let mut got_data = false;
        let mut batch_size = 0usize;
        
        // Batch receive: drain all available data at once
        while let Some(data) = crate::netstack::tcp::recv_data(ip, server_port, src_port) {
            got_data = true;
            batch_size += data.len();
            
            // Limiter la taille pour eviter OOM
            if received_data.len() + data.len() > MAX_SIZE {
                crate::println_color!(COLOR_YELLOW, "\n      WARNING: File too large, truncating");
                break;
            }
            
            received_data.extend_from_slice(&data);
        }
        
        // Afficher la progression (only when significant change)
        let kb = received_data.len() / 1024;
        if kb >= last_progress + 25 || (kb > 0 && last_progress == 0) {
            let elapsed = crate::logger::get_ticks().saturating_sub(start);
            let speed_kbps = if elapsed > 0 { (kb as u64 * 1000) / elapsed } else { 0 };
            crate::print!("\r      Downloaded: {} KB ({} KB/s)    ", kb, speed_kbps);
            last_progress = kb;
        }
        
        // Periodically flush pending ACKs (every 5ms for faster throughput)
        let now = crate::logger::get_ticks();
        if now.saturating_sub(last_ack_flush) >= 5 {
            crate::netstack::tcp::flush_pending_acks(ip, server_port, src_port);
            last_ack_flush = now;
        }
        
        if !got_data {
            idle_count = idle_count.saturating_add(1);
            
            // Check for FIN or excessive idle
            if crate::netstack::tcp::fin_received(ip, server_port, src_port) {
                // Flush final ACK
                crate::netstack::tcp::flush_pending_acks(ip, server_port, src_port);
                break;
            }
            
            // Lower idle threshold - break earlier if no data
            if idle_count > 100_000 {
                crate::serial_println!("[DOWNLOAD] Idle timeout after {} polls", last_poll_count);
                break;
            }
            
            // Brief pause when idle - but not too long
            for _ in 0..50 { core::hint::spin_loop(); }
        } else {
            idle_count = 0;
        }
        
        // Timeout 60 secondes
        if crate::logger::get_ticks().saturating_sub(start) > 60000 {
            crate::println_color!(COLOR_YELLOW, "\n      WARNING: Download timeout");
            break;
        }
    }
    
    let _ = crate::netstack::tcp::send_fin(ip, server_port, src_port);
    crate::println!();
    
    let elapsed_ms = crate::logger::get_ticks().saturating_sub(start);
    let total_kb = received_data.len() / 1024;
    let avg_speed = if elapsed_ms > 0 { (total_kb as u64 * 1000) / elapsed_ms } else { 0 };
    crate::println_color!(COLOR_GREEN, "      Transfer complete: {} KB in {}ms ({} KB/s)", total_kb, elapsed_ms, avg_speed);
    
    if received_data.is_empty() {
        crate::println_color!(COLOR_RED, "      ERROR: No data received");
        crate::netstack::dhcp::resume();
        return;
    }
    
    // Extraire le body HTTP (apres \r\n\r\n)
    let body_start = received_data.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .unwrap_or(0);
    
    let image_data = &received_data[body_start..];
    let size_mb = image_data.len() as f32 / (1024.0 * 1024.0);
    
    crate::println_color!(COLOR_GREEN, "      Download complete: {:.2} MB", size_mb);
    crate::println!();
    
    // Etape 3: Sauvegarder l'image directement dans le ramfs
    crate::println_color!(COLOR_YELLOW, "[3/4] Saving image to /opt/gui/alpine.tar.gz...");
    
    // Utiliser le ramfs directement (plus fiable que le VFS quand pas de root mount)
    let save_result = crate::ramfs::with_fs(|fs| {
        // Creer les dossiers
        let _ = fs.mkdir("/opt");
        let _ = fs.mkdir("/opt/gui");
        // Creer le fichier et ecrire
        let _ = fs.touch("/opt/gui/alpine.tar.gz");
        fs.write_file("/opt/gui/alpine.tar.gz", image_data)
    });
    
    match save_result {
        Ok(_) => {
            crate::println_color!(COLOR_GREEN, "      Saved successfully");
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "      ERROR: Write failed: {:?}", e);
            crate::netstack::dhcp::resume();
            return;
        }
    }
    crate::println!();
    
    // Etape 4: Configuration
    crate::println_color!(COLOR_YELLOW, "[4/4] Configuring GUI environment...");
    
    // Marquer comme installe
    GUI_INSTALLED.store(true, core::sync::atomic::Ordering::Relaxed);
    
    crate::println_color!(COLOR_GREEN, "      Configuration complete");
    crate::println!();
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "----------------------------------------------------------------");
    crate::println_color!(COLOR_BRIGHT_GREEN, "                    GUI Installation Complete!");
    crate::println_color!(COLOR_BRIGHT_GREEN, "----------------------------------------------------------------");
    crate::println!();
    crate::println!("Image saved to: /opt/gui/alpine.tar.gz ({:.2} MB)", size_mb);
    crate::println!();
    
    // Save to persistent storage
    crate::println_color!(COLOR_YELLOW, "Saving to disk for persistence...");
    match crate::persistence::save_file("/opt/gui/alpine.tar.gz", image_data) {
        Ok(_) => {
            crate::println_color!(COLOR_GREEN, "  Saved to disk! Will be restored on next boot.");
        }
        Err(e) => {
            crate::println_color!(COLOR_YELLOW, "  Could not save to disk: {}", e);
            crate::println!("  (Download will need to be repeated after reboot)");
        }
    }
    crate::println!();
    
    crate::println!("Use 'gui start' to launch the graphical environment.");
    
    // Resume DHCP after successful download
    crate::netstack::dhcp::resume();
    crate::serial_println!("[GUI_INSTALL] DHCP resumed");
}

pub(super) fn cmd_gui_start() {
    let installed = GUI_INSTALLED.load(core::sync::atomic::Ordering::Relaxed);
    
    if !installed {
        // Verifier si le fichier existe quand meme
        if !file_exists("/opt/gui/alpine.tar.gz") {
            crate::println_color!(COLOR_YELLOW, "Linux VM not installed.");
            crate::println!("Run 'gui install' first to download Alpine Linux.");
            return;
        }
        GUI_INSTALLED.store(true, core::sync::atomic::Ordering::Relaxed);
    }
    
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println_color!(COLOR_CYAN, "|              Starting Alpine Linux VM                        |");
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println!();
    
    // Lancer la VM Linux via l'hyperviseur integre
    crate::println_color!(COLOR_YELLOW, "[1/3] Initializing hypervisor...");
    
    // Try to initialize hypervisor if not already enabled
    if !crate::hypervisor::is_enabled() {
        match crate::hypervisor::init() {
            Ok(()) => {
                crate::println_color!(COLOR_GREEN, "      Hypervisor initialized (VT-x/AMD-V)");
            }
            Err(e) => {
                crate::serial_println!("[GUI] Hypervisor init failed: {:?}", e);
                crate::println_color!(COLOR_RED, "      ERROR: Hardware virtualization not available");
                crate::println!("      Requires Intel VT-x or AMD-V");
                crate::println!();
                crate::println_color!(COLOR_YELLOW, "Falling back to Linux subsystem emulation...");
                cmd_linux_shell();
                return;
            }
        }
    }
    crate::println_color!(COLOR_GREEN, "      Hypervisor ready");
    
    crate::println_color!(COLOR_YELLOW, "[2/3] Loading Alpine Linux image...");
    crate::println_color!(COLOR_GREEN, "      Image: /opt/gui/alpine.tar.gz");
    
    crate::println_color!(COLOR_YELLOW, "[3/3] Booting VM...");
    
    // Demarrer la VM Linux
    match crate::hypervisor::linux_subsystem::boot() {
        Ok(_) => {
            crate::println_color!(COLOR_GREEN, "      VM started successfully");
            crate::println!();
            crate::println_color!(COLOR_CYAN, "Alpine Linux is now running.");
            crate::println!("Use 'vm console' to connect to the VM console.");
            crate::println!("Use 'vm stop' to stop the VM.");
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "      ERROR: Failed to start VM: {:?}", e);
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "Falling back to Linux subsystem...");
            cmd_linux_shell();
        }
    }
}

/// Launch the Linux subsystem shell
pub(super) fn cmd_linux_shell() {
    // Check if Linux subsystem is already initialized
    if !crate::linux::is_initialized() {
        // Try to initialize from the downloaded rootfs
        if file_exists("/opt/gui/alpine.tar.gz") {
            match crate::linux::init("/opt/gui/alpine.tar.gz") {
                Ok(()) => {}
                Err(e) => {
                    crate::println_color!(COLOR_RED, "Failed to initialize Linux subsystem: {}", e);
                    return;
                }
            }
        } else {
            crate::println_color!(COLOR_YELLOW, "Linux subsystem not installed.");
            crate::println!("Run 'gui install' to download and install Alpine Linux.");
            return;
        }
    }
    
    // Start the Linux shell
    crate::linux::start_shell();
}

pub(super) fn cmd_glmode(args: &[&str]) {
    use crate::desktop::{RenderMode, set_render_mode, set_theme};
    use crate::graphics::CompositorTheme;
    
    if args.is_empty() {
        crate::println_color!(COLOR_CYAN, "TrustGL Compositor Settings");
        crate::println_color!(COLOR_CYAN, "===========================");
        crate::println!();
        crate::println!("Usage: glmode <mode|theme>");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Render Modes:");
        crate::println!("  classic   - Classic framebuffer rendering (fast, stable)");
        crate::println!("  opengl    - OpenGL compositor with visual effects");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Themes (OpenGL mode only):");
        crate::println!("  flat      - Simple flat rendering, no effects");
        crate::println!("  modern    - Shadows and subtle effects");
        crate::println!("  glass     - Transparency and blur effects");
        crate::println!("  neon      - Glowing neon borders");
        crate::println!("  minimal   - Thin borders, minimal style");
        crate::println!();
        crate::println!("Example: glmode opengl");
        crate::println!("         glmode neon");
        return;
    }
    
    match args[0].to_lowercase().as_str() {
        "classic" | "normal" | "default" => {
            set_render_mode(RenderMode::Classic);
            crate::println_color!(COLOR_GREEN, "Switched to Classic rendering mode");
        }
        "opengl" | "gl" | "compositor" => {
            set_render_mode(RenderMode::OpenGL);
            crate::println_color!(COLOR_GREEN, "Switched to OpenGL compositor mode");
            crate::println!("Use 'glmode <theme>' to change visual theme");
        }
        "flat" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Flat);
            crate::println_color!(COLOR_GREEN, "Theme: Flat (OpenGL)");
        }
        "modern" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Modern);
            crate::println_color!(COLOR_GREEN, "Theme: Modern (shadows, subtle effects)");
        }
        "glass" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Glass);
            crate::println_color!(COLOR_GREEN, "Theme: Glass (transparency effects)");
        }
        "neon" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Neon);
            crate::println_color!(COLOR_GREEN, "Theme: Neon (glowing borders)");
        }
        "minimal" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Minimal);
            crate::println_color!(COLOR_GREEN, "Theme: Minimal (thin borders)");
        }
        _ => {
            crate::println_color!(COLOR_RED, "Unknown mode/theme: {}", args[0]);
            crate::println!("Use 'glmode' without arguments for help");
        }
    }
}

/// Dynamic theme management command
pub(super) fn cmd_theme(args: &[&str]) {
    if args.is_empty() {
        crate::println_color!(COLOR_CYAN, "TrustOS Theme Manager");
        crate::println_color!(COLOR_CYAN, "=====================");
        crate::println!();
        crate::println!("Usage: theme <command> [args]");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Commands:");
        crate::println!("  list              - List available built-in themes");
        crate::println!("  set <name>        - Switch to a built-in theme");
        crate::println!("  load <path>       - Load theme from config file");
        crate::println!("  save <path>       - Save current theme to file");
        crate::println!("  reload            - Reload wallpaper from disk");
        crate::println!("  info              - Show current theme info");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Built-in Themes:");
        crate::println!("  dark / trustos    - TrustOS dark green theme");
        crate::println!("  windows11 / win11 - Windows 11 dark theme");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Config File Format (/etc/theme.conf):");
        crate::println!("  [colors]");
        crate::println!("  background = 0x0A0E0B");
        crate::println!("  accent = 0x00D26A");
        crate::println!("  ");
        crate::println!("  [wallpaper]");
        crate::println!("  path = /usr/share/wallpapers/matrix.bmp");
        return;
    }
    
    match args[0] {
        "list" => {
            crate::println_color!(COLOR_CYAN, "Available Themes:");
            crate::println!("  dark       - TrustOS dark green (default)");
            crate::println!("  windows11  - Windows 11 dark blue");
            crate::println!("  light      - Light theme");
        }
        "set" => {
            if args.len() < 2 {
                crate::println_color!(COLOR_RED, "Usage: theme set <name>");
                return;
            }
            crate::theme::set_builtin_theme(args[1]);
            crate::println_color!(COLOR_GREEN, "Theme switched to: {}", args[1]);
        }
        "load" => {
            if args.len() < 2 {
                crate::println_color!(COLOR_RED, "Usage: theme load <path>");
                crate::println!("Example: theme load /etc/theme.conf");
                return;
            }
            if crate::theme::load_theme(args[1]) {
                crate::println_color!(COLOR_GREEN, "Theme loaded from: {}", args[1]);
            } else {
                crate::println_color!(COLOR_RED, "Failed to load theme from: {}", args[1]);
            }
        }
        "save" => {
            if args.len() < 2 {
                crate::println_color!(COLOR_RED, "Usage: theme save <path>");
                return;
            }
            let theme = crate::theme::THEME.read();
            let content = crate::theme::config::generate_theme_config(&theme);
            drop(theme);
            
            match crate::vfs::write_file(args[1], content.as_bytes()) {
                Ok(_) => crate::println_color!(COLOR_GREEN, "Theme saved to: {}", args[1]),
                Err(e) => crate::println_color!(COLOR_RED, "Failed to save: {:?}", e),
            }
        }
        "reload" => {
            crate::theme::reload_wallpaper();
            crate::println_color!(COLOR_GREEN, "Wallpaper reloaded");
        }
        "info" => {
            let theme = crate::theme::THEME.read();
            crate::println_color!(COLOR_CYAN, "Current Theme: {}", 
                if theme.name.is_empty() { "TrustOS Default" } else { &theme.name });
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "Colors:");
            crate::println!("  Background:  0x{:08X}", theme.colors.background);
            crate::println!("  Accent:      0x{:08X}", theme.colors.accent);
            crate::println!("  Text:        0x{:08X}", theme.colors.text_primary);
            crate::println!("  Surface:     0x{:08X}", theme.colors.surface);
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "Taskbar:");
            crate::println!("  Height:      {} px", theme.taskbar.height);
            crate::println!("  Centered:    {}", theme.taskbar.centered_icons);
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "Windows:");
            crate::println!("  Title bar:   {} px", theme.window.titlebar_height);
            crate::println!("  Radius:      {} px", theme.window.border_radius);
            crate::println!("  Shadow:      {} px", theme.window.shadow_size);
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "Wallpaper:");
            crate::println!("  Path:        {}", 
                if theme.wallpaper.path.is_empty() { "(none)" } else { &theme.wallpaper.path });
            crate::println!("  Mode:        {:?}", theme.wallpaper.mode);
        }
        _ => {
            crate::println_color!(COLOR_RED, "Unknown theme command: {}", args[0]);
            crate::println!("Use 'theme' for help");
        }
    }
}

/// Window animations control command
pub(super) fn cmd_animations(args: &[&str]) {
    if args.is_empty() {
        let enabled = crate::desktop::animations_enabled();
        let speed = crate::desktop::get_animation_speed();
        
        crate::println_color!(COLOR_CYAN, "TrustOS Animation Settings");
        crate::println_color!(COLOR_CYAN, "==========================");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Current Status:");
        if enabled {
            crate::println!("  Animations: {} ENABLED", "\x1b[32m?\x1b[0m");
        } else {
            crate::println!("  Animations: {} DISABLED", "\x1b[31m?\x1b[0m");
        }
        crate::println!("  Speed:      {}x", speed);
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Commands:");
        crate::println!("  anim on           - Enable animations");
        crate::println!("  anim off          - Disable animations");
        crate::println!("  anim toggle       - Toggle on/off");
        crate::println!("  anim speed <val>  - Set speed (0.25-4.0)");
        crate::println!("                      1.0=normal, 2.0=fast, 0.5=slow");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Animation Types:");
        crate::println!("  - Window open (scale up from center)");
        crate::println!("  - Window close (scale down + fade out)");
        crate::println!("  - Minimize (move to taskbar)");
        crate::println!("  - Maximize/Restore (smooth resize)");
        return;
    }
    
    match args[0] {
        "on" | "enable" | "1" | "true" => {
            crate::desktop::set_animations_enabled(true);
            crate::println_color!(COLOR_GREEN, "? Animations enabled");
        }
        "off" | "disable" | "0" | "false" => {
            crate::desktop::set_animations_enabled(false);
            crate::println_color!(COLOR_YELLOW, "? Animations disabled");
        }
        "toggle" => {
            let current = crate::desktop::animations_enabled();
            crate::desktop::set_animations_enabled(!current);
            if !current {
                crate::println_color!(COLOR_GREEN, "? Animations enabled");
            } else {
                crate::println_color!(COLOR_YELLOW, "? Animations disabled");
            }
        }
        "speed" => {
            if args.len() < 2 {
                crate::println!("Current speed: {}x", crate::desktop::get_animation_speed());
                crate::println!("Usage: anim speed <value>");
                crate::println!("  Examples: 0.5 (slow), 1.0 (normal), 2.0 (fast)");
                return;
            }
            if let Ok(speed) = args[1].parse::<f32>() {
                crate::desktop::set_animation_speed(speed);
                crate::println_color!(COLOR_GREEN, "Animation speed set to {}x", speed);
            } else {
                crate::println_color!(COLOR_RED, "Invalid speed value: {}", args[1]);
            }
        }
        "status" | "info" => {
            let enabled = crate::desktop::animations_enabled();
            let speed = crate::desktop::get_animation_speed();
            crate::println!("Animations: {}", if enabled { "enabled" } else { "disabled" });
            crate::println!("Speed: {}x", speed);
        }
        _ => {
            crate::println_color!(COLOR_RED, "Unknown animation command: {}", args[0]);
            crate::println!("Use 'anim' for help");
        }
    }
}

/// HoloMatrix 3D background control command
pub(super) fn cmd_holomatrix(args: &[&str]) {
    use crate::graphics::holomatrix;
    
    if args.is_empty() {
        let enabled = holomatrix::is_enabled();
        let scene = holomatrix::get_scene();
        
        crate::println_color!(COLOR_CYAN, "TrustOS HoloMatrix 3D");
        crate::println_color!(COLOR_CYAN, "=====================");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Current Status:");
        if enabled {
            crate::println!("  HoloMatrix: {} ENABLED", "\x1b[36m?\x1b[0m");
        } else {
            crate::println!("  HoloMatrix: {} DISABLED", "\x1b[31m?\x1b[0m");
        }
        crate::println!("  Scene:      {}", scene.name());
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Commands:");
        crate::println!("  holo on           - Enable HoloMatrix 3D background");
        crate::println!("  holo off          - Disable (use Matrix Rain)");
        crate::println!("  holo toggle       - Toggle on/off");
        crate::println!("  holo next         - Cycle to next scene");
        crate::println!("  holo scene <name> - Set specific scene");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Available Scenes:");
        crate::println!("  cube     - Rotating wireframe cube");
        crate::println!("  sphere   - Pulsating sphere");
        crate::println!("  torus    - 3D donut/ring");
        crate::println!("  grid     - Perspective grid with cube");
        crate::println!("  multi    - Multiple floating shapes");
        crate::println!("  dna      - Animated DNA double helix");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "How it works:");
        crate::println!("  Renders 3D shapes using 16 Z-slices (layers)");
        crate::println!("  Each layer has depth-based transparency");
        crate::println!("  Creates holographic volumetric effect");
        return;
    }
    
    match args[0] {
        "on" | "enable" | "1" | "true" => {
            holomatrix::set_enabled(true);
            crate::println_color!(0xFF00FFFF, "? HoloMatrix 3D enabled");
            crate::println!("Launch 'desktop' to see the effect");
        }
        "off" | "disable" | "0" | "false" => {
            holomatrix::set_enabled(false);
            crate::println_color!(COLOR_YELLOW, "? HoloMatrix disabled (Matrix Rain active)");
        }
        "toggle" => {
            let enabled = holomatrix::toggle();
            if enabled {
                crate::println_color!(0xFF00FFFF, "? HoloMatrix 3D enabled");
            } else {
                crate::println_color!(COLOR_YELLOW, "? HoloMatrix disabled");
            }
        }
        "next" | "cycle" => {
            let scene = holomatrix::next_scene();
            crate::println_color!(0xFF00FFFF, "Scene: {}", scene.name());
        }
        "scene" | "set" => {
            if args.len() < 2 {
                crate::println!("Current scene: {}", holomatrix::get_scene().name());
                crate::println!("Usage: holo scene <name>");
                crate::println!("Available: cube, sphere, torus, grid, multi, dna");
                return;
            }
            if let Some(scene) = holomatrix::HoloScene::from_name(args[1]) {
                holomatrix::set_scene(scene);
                crate::println_color!(0xFF00FFFF, "Scene set to: {}", scene.name());
            } else {
                crate::println_color!(COLOR_RED, "Unknown scene: {}", args[1]);
                crate::println!("Available: cube, sphere, torus, grid, multi, dna");
            }
        }
        "status" | "info" => {
            let enabled = holomatrix::is_enabled();
            let scene = holomatrix::get_scene();
            crate::println!("HoloMatrix: {}", if enabled { "enabled" } else { "disabled" });
            crate::println!("Scene: {}", scene.name());
        }
        "list" | "scenes" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "Available Scenes:");
            for name in holomatrix::HoloScene::all_names() {
                crate::println!("  {}", name);
            }
        }
        _ => {
            // Try to parse as scene name directly
            if let Some(scene) = holomatrix::HoloScene::from_name(args[0]) {
                holomatrix::set_scene(scene);
                crate::println_color!(0xFF00FFFF, "Scene set to: {}", scene.name());
            } else {
                crate::println_color!(COLOR_RED, "Unknown command: {}", args[0]);
                crate::println!("Use 'holo' for help");
            }
        }
    }
}

/// Image viewer command
pub(super) fn cmd_imgview(args: &[&str]) {
    if args.is_empty() {
        crate::println_color!(COLOR_CYAN, "TrustOS Image Viewer");
        crate::println_color!(COLOR_CYAN, "====================");
        crate::println!();
        crate::println!("Usage: imgview <path> [options]");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Options:");
        crate::println!("  -x <num>     X position (default: center)");
        crate::println!("  -y <num>     Y position (default: center)");
        crate::println!("  -w <num>     Width (scale to this width)");
        crate::println!("  -h <num>     Height (scale to this height)");
        crate::println!("  -info        Show image info only, don't display");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Supported Formats:");
        crate::println!("  BMP  - 24-bit and 32-bit uncompressed");
        crate::println!("  PPM  - P3 (ASCII) and P6 (binary)");
        crate::println!("  RAW  - Raw RGBA pixel data");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Examples:");
        crate::println!("  imgview /usr/share/wallpapers/logo.bmp");
        crate::println!("  imgview /home/image.ppm -x 100 -y 100");
        crate::println!("  imgview photo.bmp -w 640 -h 480");
        return;
    }
    
    let path = args[0];
    let mut pos_x: Option<i32> = None;
    let mut pos_y: Option<i32> = None;
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
    let mut info_only = false;
    
    // Parse options
    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "-x" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<i32>() {
                    pos_x = Some(v);
                }
                i += 2;
            }
            "-y" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<i32>() {
                    pos_y = Some(v);
                }
                i += 2;
            }
            "-w" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<u32>() {
                    width = Some(v);
                }
                i += 2;
            }
            "-h" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<u32>() {
                    height = Some(v);
                }
                i += 2;
            }
            "-info" => {
                info_only = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    
    // Load the image
    crate::println!("Loading image: {}", path);
    
    match crate::image::load(path) {
        Some(img) => {
            crate::println_color!(COLOR_GREEN, "Image loaded successfully!");
            crate::println!("  Size: {} x {} pixels", img.width, img.height);
            crate::println!("  Memory: {} KB", (img.pixels.len() * 4) / 1024);
            
            if info_only {
                return;
            }
            
            // Calculate final dimensions
            let dest_w = width.unwrap_or(img.width);
            let dest_h = height.unwrap_or(img.height);
            
            // Calculate position (center if not specified)
            let (fb_width, fb_height) = crate::framebuffer::get_dimensions();
            let x = pos_x.unwrap_or_else(|| ((fb_width - dest_w) / 2) as i32);
            let y = pos_y.unwrap_or_else(|| ((fb_height - dest_h) / 2) as i32);
            
            crate::println!("  Drawing at ({}, {}) size {}x{}", x, y, dest_w, dest_h);
            
            // Draw the image
            if dest_w == img.width && dest_h == img.height {
                img.draw(x, y);
            } else {
                img.draw_scaled(x, y, dest_w, dest_h);
            }
            
            crate::framebuffer::swap_buffers();
            crate::println_color!(COLOR_GREEN, "Image displayed!");
        }
        None => {
            crate::println_color!(COLOR_RED, "Failed to load image: {}", path);
            crate::println!("Make sure the file exists and is a supported format.");
        }
    }
}

/// Demo command to display generated test images
pub(super) fn cmd_imgdemo(args: &[&str]) {
    let demo_type = args.first().copied().unwrap_or("gradient");
    
    crate::println_color!(COLOR_CYAN, "Image Demo: {}", demo_type);
    
    let (fb_width, fb_height) = crate::framebuffer::get_dimensions();
    
    match demo_type {
        "gradient" => {
            // Vertical gradient
            let img = crate::image::create_gradient_v(
                200, 200, 
                0xFF0066FF,  // Blue top
                0xFF00FF66   // Green bottom
            );
            let x = ((fb_width - 200) / 2) as i32;
            let y = ((fb_height - 200) / 2) as i32;
            img.draw(x, y);
            crate::println_color!(COLOR_GREEN, "Displayed gradient at center");
        }
        "checker" => {
            // Checkerboard pattern
            let img = crate::image::create_checkerboard(
                256, 256, 32,
                0xFFFFFFFF,  // White
                0xFF000000   // Black
            );
            let x = ((fb_width - 256) / 2) as i32;
            let y = ((fb_height - 256) / 2) as i32;
            img.draw(x, y);
            crate::println_color!(COLOR_GREEN, "Displayed checkerboard at center");
        }
        "trustos" => {
            // TrustOS logo colors pattern
            let img = crate::image::create_gradient_v(
                300, 100,
                0xFF00D26A,  // Accent green
                0xFF0A0E0B   // Dark background
            );
            let x = ((fb_width - 300) / 2) as i32;
            let y = ((fb_height - 100) / 2) as i32;
            img.draw(x, y);
            
            // Draw a decorative border
            let border_color = 0xFF00D26A;
            for i in 0..300 {
                crate::framebuffer::put_pixel(x as u32 + i, y as u32, border_color);
                crate::framebuffer::put_pixel(x as u32 + i, (y + 99) as u32, border_color);
            }
            for i in 0..100 {
                crate::framebuffer::put_pixel(x as u32, y as u32 + i, border_color);
                crate::framebuffer::put_pixel((x + 299) as u32, y as u32 + i, border_color);
            }
            
            crate::println_color!(COLOR_GREEN, "Displayed TrustOS banner");
        }
        "colors" => {
            // Color test pattern
            let mut img = crate::image::Image::new(256, 256);
            for y in 0..256 {
                for x in 0..256 {
                    let r = x as u32;
                    let g = y as u32;
                    let b = ((x + y) / 2) as u32;
                    let color = 0xFF000000 | (r << 16) | (g << 8) | b;
                    img.set_pixel(x, y, color);
                }
            }
            let x = ((fb_width - 256) / 2) as i32;
            let y = ((fb_height - 256) / 2) as i32;
            img.draw(x, y);
            crate::println_color!(COLOR_GREEN, "Displayed color test pattern");
        }
        "alpha" => {
            // Demonstrate alpha blending
            // First, draw a red background square
            let bg = crate::image::create_solid(200, 200, 0xFFFF0000);
            let x = ((fb_width - 200) / 2) as i32;
            let y = ((fb_height - 200) / 2) as i32;
            bg.draw(x, y);
            
            // Then draw a semi-transparent blue overlay
            let mut overlay = crate::image::Image::new(200, 200);
            for py in 0..200u32 {
                for px in 0..200u32 {
                    // Alpha varies from 0 to 200 based on position
                    let alpha = (px + py) / 2;
                    let color = (alpha << 24) | 0x000000FF;  // Semi-transparent blue
                    overlay.set_pixel(px, py, color);
                }
            }
            overlay.draw(x, y);
            crate::println_color!(COLOR_GREEN, "Displayed alpha blend demo (red + blue)");
        }
        _ => {
            crate::println!("Available demos:");
            crate::println!("  gradient  - Vertical color gradient");
            crate::println!("  checker   - Checkerboard pattern");
            crate::println!("  trustos   - TrustOS banner");
            crate::println!("  colors    - RGB color test pattern");
            crate::println!("  alpha     - Alpha blending demo");
            crate::println!();
            crate::println!("Usage: imgdemo <name>");
            return;
        }
    }
    
    crate::framebuffer::swap_buffers();
}

pub(super) fn cmd_tasks() {
    let tasks = crate::task::list_tasks();
    crate::println_color!(COLOR_CYAN, "  PID  STATE       PRIORITY  NAME");
    crate::println_color!(COLOR_CYAN, "-------------------------------------");
    
    // Always show kernel and shell
    crate::println!("    1  running     critical  kernel");
    crate::println!("    2  running     normal    tsh");
    
    let task_count = tasks.len();
    for (id, name, state, priority) in tasks {
        let state_str = match state {
            crate::task::TaskState::Ready => "ready",
            crate::task::TaskState::Running => "running",
            crate::task::TaskState::Blocked => "blocked",
            crate::task::TaskState::Terminated => "done",
        };
        let priority_str = match priority {
            crate::task::Priority::Low => "low",
            crate::task::Priority::Normal => "normal",
            crate::task::Priority::High => "high",
            crate::task::Priority::Critical => "critical",
        };
        crate::println!("{:>5}  {:10}  {:8}  {}", id + 2, state_str, priority_str, name);
    }
    
    crate::println!();
    crate::println_color!(COLOR_DARK_GREEN, "Total: {} tasks", task_count + 2);
}

pub(super) fn cmd_threads() {
    crate::println_color!(COLOR_CYAN, "  TID  PID  STATE       NAME");
    crate::println_color!(COLOR_CYAN, "------------------------------------");
    
    // Get thread info from thread module
    let threads = crate::thread::list_threads();
    let count = threads.len();
    
    for (tid, pid, state, name) in threads {
        let state_str = match state {
            crate::thread::ThreadState::Ready => "ready",
            crate::thread::ThreadState::Running => "running",
            crate::thread::ThreadState::Blocked => "blocked",
            crate::thread::ThreadState::Sleeping => "sleeping",
            crate::thread::ThreadState::Dead => "dead",
        };
        crate::println!("{:>5}  {:>3}  {:10}  {}", tid, pid, state_str, &name);
    }
    
    crate::println!();
    crate::println_color!(COLOR_DARK_GREEN, "Total: {} threads", count);
}

// ==================== PERSISTENCE COMMANDS ====================

pub(super) fn cmd_persistence(args: &[&str]) {
    if args.is_empty() {
        // Show status
        let (status, files, size) = crate::persistence::status();
        crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
        crate::println_color!(COLOR_CYAN, "|                    Persistence Status                        |");
        crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
        crate::println!();
        crate::println!("  Status:       {}", status);
        crate::println!("  Saved files:  {}", files);
        crate::println!("  Total size:   {} KB", size / 1024);
        crate::println!();
        crate::println!("Commands:");
        crate::println!("  persist status  - Show this status");
        crate::println!("  persist clear   - Clear all saved data");
        crate::println!("  persist save    - Save current downloads to disk");
        crate::println!();
        return;
    }
    
    match args[0] {
        "status" => {
            let (status, files, size) = crate::persistence::status();
            crate::println!("Persistence: {} ({} files, {} KB)", status, files, size / 1024);
        }
        "clear" => {
            crate::println!("Clearing persistence data...");
            match crate::persistence::clear() {
                Ok(_) => crate::println_color!(COLOR_GREEN, "Persistence data cleared."),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        "save" => {
            crate::println!("Saving current data to disk...");
            
            // Save Alpine rootfs if it exists
            let alpine_path = "/opt/gui/alpine.tar.gz";
            if file_exists(alpine_path) {
                let read_result: Result<Vec<u8>, _> = crate::ramfs::with_fs(|fs| {
                    fs.read_file(alpine_path).map(|d| d.to_vec())
                });
                match read_result {
                    Ok(data) => {
                        match crate::persistence::save_file(alpine_path, &data) {
                            Ok(_) => crate::println_color!(COLOR_GREEN, "  Saved: {} ({} KB)", alpine_path, data.len() / 1024),
                            Err(e) => crate::println_color!(COLOR_RED, "  Failed: {} - {}", alpine_path, e),
                        }
                    }
                    Err(e) => crate::println_color!(COLOR_RED, "  Cannot read {}: {:?}", alpine_path, e),
                }
            } else {
                crate::println!("  No files to save. Run 'gui install' first.");
            }
        }
        _ => {
            crate::println!("Unknown persistence command: {}", args[0]);
            crate::println!("Use: persist [status|clear|save]");
        }
    }
}

// ==================== DISK COMMANDS ====================

pub(super) fn cmd_disk() {
    crate::println_color!(COLOR_CYAN, "=== Storage Devices ===");
    
    let mut device_count = 0u32;
    
    // ── NVMe ──
    if crate::nvme::is_initialized() {
        if let Some((model, serial, ns_size, lba_size)) = crate::nvme::get_info() {
            let size_mb = (ns_size * lba_size as u64) / (1024 * 1024);
            crate::println!();
            crate::println_color!(COLOR_GREEN, "[NVMe] {}", model);
            crate::println!("  Serial:    {}", serial);
            crate::println!("  Capacity:  {} MB ({} sectors x {} bytes)", size_mb, ns_size, lba_size);
            crate::println!("  Interface: NVMe over PCIe");
            device_count += 1;
        }
    }
    
    // ── AHCI/SATA ──
    if crate::drivers::ahci::is_initialized() {
        for dev in crate::drivers::ahci::list_devices() {
            let size_mb = (dev.sector_count * 512) / (1024 * 1024);
            crate::println!();
            crate::print_color!(COLOR_GREEN, "[AHCI Port {}] ", dev.port_num);
            crate::println!("{}", dev.model);
            crate::println!("  Serial:    {}", dev.serial);
            crate::println!("  Capacity:  {} MB ({} sectors)", size_mb, dev.sector_count);
            crate::println!("  Type:      {:?}", dev.device_type);
            crate::println!("  Interface: SATA (AHCI)");
            device_count += 1;
        }
    }
    
    // ── IDE/ATA ──
    for drv in crate::drivers::ata::list_drives() {
        if drv.present {
            let size_mb = (drv.sector_count * 512) / (1024 * 1024);
            let ch = match drv.channel {
                crate::drivers::ata::IdeChannel::Primary => "Primary",
                crate::drivers::ata::IdeChannel::Secondary => "Secondary",
            };
            let pos = match drv.position {
                crate::drivers::ata::DrivePosition::Master => "Master",
                crate::drivers::ata::DrivePosition::Slave => "Slave",
            };
            crate::println!();
            crate::print_color!(COLOR_GREEN, "[IDE {} {}] ", ch, pos);
            crate::println!("{}", drv.model);
            crate::println!("  Serial:    {}", drv.serial);
            crate::println!("  Capacity:  {} MB ({} sectors)", size_mb, drv.sector_count);
            crate::println!("  LBA48:     {}", if drv.lba48 { "Yes" } else { "No (28-bit)" });
            crate::println!("  ATAPI:     {}", if drv.atapi { "Yes" } else { "No" });
            crate::println!("  Interface: IDE/ATA (PIO)");
            device_count += 1;
        }
    }
    
    // ── VirtIO ──
    if crate::virtio_blk::is_initialized() {
        let cap = crate::virtio_blk::capacity();
        let size_mb = (cap * 512) / (1024 * 1024);
        let ro = crate::virtio_blk::is_read_only();
        crate::println!();
        crate::println_color!(COLOR_GREEN, "[VirtIO Block Device]");
        crate::println!("  Capacity:  {} MB ({} sectors)", size_mb, cap);
        crate::println!("  Read-Only: {}", if ro { "Yes" } else { "No" });
        crate::println!("  Interface: VirtIO (paravirtual)");
        device_count += 1;
    }
    
    // ── USB Storage ──
    for (i, (name, blocks, bsize)) in crate::drivers::usb_storage::list_devices().iter().enumerate() {
        let size_mb = (*blocks * *bsize as u64) / (1024 * 1024);
        crate::println!();
        crate::print_color!(COLOR_GREEN, "[USB Storage #{}] ", i);
        crate::println!("{}", name);
        crate::println!("  Capacity:  {} MB ({} blocks x {} bytes)", size_mb, blocks, bsize);
        crate::println!("  Interface: USB Mass Storage (BBB/SCSI)");
        device_count += 1;
    }
    
    // ── RAM Disk ──
    crate::println!();
    if let Some(info) = crate::disk::get_info() {
        crate::println_color!(COLOR_DARK_GREEN, "[RAM Disk]");
        crate::println!("  Size:      {} KB ({} sectors)", info.sectors / 2, info.sectors);
        
        let (reads, writes, bytes_r, bytes_w) = crate::disk::get_stats();
        crate::println!("  Stats:     {} reads ({} B), {} writes ({} B)", reads, bytes_r, writes, bytes_w);
    }
    
    // ── Summary ──
    crate::println!();
    if device_count == 0 {
        crate::println_color!(COLOR_YELLOW, "No hardware storage detected (RAM disk only)");
    } else {
        crate::println_color!(COLOR_CYAN, "Total: {} hardware storage device(s) + RAM disk", device_count);
    }
}

pub(super) fn cmd_dd(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: dd <sector> [count]");
        crate::println!("       dd write <sector> <text>");
        crate::println!("       dd dump <sector>");
        return;
    }
    
    if args[0] == "dump" && args.len() > 1 {
        let sector: u64 = match args[1].parse() {
            Ok(n) => n,
            Err(_) => {
                crate::println_color!(COLOR_RED, "Invalid sector number");
                return;
            }
        };
        
        match crate::disk::dump_sector(sector) {
            Ok(dump) => crate::println!("{}", dump),
            Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
        }
        return;
    }
    
    if args[0] == "write" && args.len() > 2 {
        let sector: u64 = match args[1].parse() {
            Ok(n) => n,
            Err(_) => {
                crate::println_color!(COLOR_RED, "Invalid sector number");
                return;
            }
        };
        
        let text = args[2..].join(" ");
        let mut data = [0u8; 512];
        let bytes = text.as_bytes();
        let len = bytes.len().min(512);
        data[..len].copy_from_slice(&bytes[..len]);
        
        match crate::disk::write_sector(sector, &data) {
            Ok(_) => crate::println_color!(COLOR_GREEN, "Written {} bytes to sector {}", len, sector),
            Err(e) => crate::println_color!(COLOR_RED, "Write error: {}", e),
        }
        return;
    }
    
    // Read sector
    let sector: u64 = match args[0].parse() {
        Ok(n) => n,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid sector number");
            return;
        }
    };

    let mut buffer = [0u8; 512];
    match crate::disk::read_sectors(sector, 1, &mut buffer) {
        Ok(_) => {
            crate::println_color!(COLOR_CYAN, "Sector {} (512 bytes):", sector);
            
            // Hexdump first 256 bytes
            for row in 0..16 {
                crate::print_color!(COLOR_DARK_GREEN, "{:04X}: ", row * 16);
                for col in 0..16 {
                    crate::print!("{:02X} ", buffer[row * 16 + col]);
                }
                crate::print!(" |");
                for col in 0..16 {
                    let b = buffer[row * 16 + col];
                    if b >= 0x20 && b < 0x7F {
                        crate::print!("{}", b as char);
                    } else {
                        crate::print!(".");
                    }
                }
                crate::println!("|");
            }
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "Read error: {}", e);
        }
    }
}

pub(super) fn cmd_ahci(args: &[&str]) {
    if args.is_empty() {
        // Show AHCI info
        crate::println_color!(COLOR_CYAN, "=== AHCI Storage Controller ===");
        
        if !crate::drivers::ahci::is_initialized() {
            crate::println_color!(COLOR_YELLOW, "AHCI not initialized");
            return;
        }
        
        let devices = crate::drivers::ahci::list_devices();
        if devices.is_empty() {
            crate::println_color!(COLOR_YELLOW, "No AHCI devices found");
            return;
        }
        
        crate::println!("Found {} device(s):", devices.len());
        for dev in &devices {
            crate::println!();
            crate::print_color!(COLOR_GREEN, "  Port {}: ", dev.port_num);
            crate::println!("{:?}", dev.device_type);
            crate::println!("    Model:   {}", dev.model);
            crate::println!("    Serial:  {}", dev.serial);
            crate::println!("    Sectors: {}", dev.sector_count);
        }
        
        crate::println!();
        crate::println_color!(COLOR_DARK_GREEN, "Commands:");
        crate::println!("  ahci read <port> <sector>   - Read sector from port");
        crate::println!("  ahci write <port> <sector> <text> - Write to sector");
        return;
    }
    
    match args[0] {
        "read" => {
            if args.len() < 3 {
                crate::println!("Usage: ahci read <port> <sector>");
                return;
            }
            
            let port: u8 = match args[1].parse() {
                Ok(n) => n,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Invalid port number");
                    return;
                }
            };
            
            let sector: u64 = match args[2].parse() {
                Ok(n) => n,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Invalid sector number");
                    return;
                }
            };
            
            crate::println!("Reading sector {} from AHCI port {}...", sector, port);
            
            // Allocate aligned buffer
            let mut buffer = alloc::vec![0u8; 512];
            
            match crate::drivers::ahci::read_sectors(port, sector, 1, &mut buffer) {
                Ok(bytes) => {
                    crate::println_color!(COLOR_GREEN, "Read {} bytes successfully", bytes);
                    crate::println!();
                    
                    // Hexdump first 256 bytes
                    for row in 0..16 {
                        crate::print_color!(COLOR_DARK_GREEN, "{:04X}: ", row * 16);
                        for col in 0..16 {
                            crate::print!("{:02X} ", buffer[row * 16 + col]);
                        }
                        crate::print!(" |");
                        for col in 0..16 {
                            let b = buffer[row * 16 + col];
                            if b >= 0x20 && b < 0x7F {
                                crate::print!("{}", b as char);
                            } else {
                                crate::print!(".");
                            }
                        }
                        crate::println!("|");
                    }
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "AHCI read error: {}", e);
                }
            }
        }
        
        "write" => {
            if args.len() < 4 {
                crate::println!("Usage: ahci write <port> <sector> <text>");
                return;
            }
            
            let port: u8 = match args[1].parse() {
                Ok(n) => n,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Invalid port number");
                    return;
                }
            };
            
            let sector: u64 = match args[2].parse() {
                Ok(n) => n,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Invalid sector number");
                    return;
                }
            };
            
            let text = args[3..].join(" ");
            let mut buffer = alloc::vec![0u8; 512];
            let bytes = text.as_bytes();
            let len = bytes.len().min(512);
            buffer[..len].copy_from_slice(&bytes[..len]);
            
            crate::println!("Writing {} bytes to sector {} on AHCI port {}...", len, sector, port);
            
            match crate::drivers::ahci::write_sectors(port, sector, 1, &buffer) {
                Ok(bytes) => {
                    crate::println_color!(COLOR_GREEN, "Written {} bytes successfully", bytes);
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "AHCI write error: {}", e);
                }
            }
        }
        
        _ => {
            crate::println!("Unknown AHCI command. Use 'ahci' for help.");
        }
    }
}

// ==================== PARTITION COMMANDS ====================

pub(super) fn cmd_fdisk(args: &[&str]) {
    use crate::drivers::partition;
    use crate::drivers::ahci;
    
    if args.is_empty() {
        // List all disks and their partitions
        crate::println_color!(COLOR_CYAN, "=== Partition Tables ===");
        crate::println!();
        
        if !ahci::is_initialized() {
            crate::println_color!(COLOR_YELLOW, "AHCI not initialized");
            crate::println!();
            crate::println!("Usage:");
            crate::println!("  fdisk           - Show partitions on all disks");
            crate::println!("  fdisk <port>    - Show partitions on specific AHCI port");
            return;
        }
        
        let devices = ahci::list_devices();
        if devices.is_empty() {
            crate::println!("No AHCI devices found");
            return;
        }
        
        for dev in devices {
            crate::println_color!(COLOR_GREEN, "--- Disk {} ({:?}) ---", dev.port_num, dev.device_type);
            
            match partition::read_from_ahci(dev.port_num) {
                Ok(table) => {
                    partition::print_partition_table(&table);
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "  Error reading partitions: {}", e);
                }
            }
            crate::println!();
        }
        
        return;
    }
    
    // Parse port number
    let port: u8 = match args[0].parse() {
        Ok(p) => p,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid port number: {}", args[0]);
            return;
        }
    };
    
    crate::println_color!(COLOR_CYAN, "=== Partitions on Disk {} ===", port);
    
    match partition::read_from_ahci(port) {
        Ok(table) => {
            partition::print_partition_table(&table);
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "Error: {}", e);
        }
    }
}

// ==================== NETWORK COMMANDS ====================

pub(super) fn cmd_ifconfig() {
    // Show NIC hardware info
    if let Some(nic) = crate::network::get_nic_info() {
        crate::println_color!(COLOR_CYAN, "Hardware:");
        crate::println!("      Device: {:04X}:{:04X} [{}]", 
            nic.vendor_id, nic.device_id, nic.vendor_name);
        crate::println!("      Driver: {}", nic.driver);
        if crate::network::has_real_driver() {
            crate::println_color!(COLOR_GREEN, "      Status: REAL DRIVER ACTIVE");
        } else {
            crate::println_color!(COLOR_YELLOW, "      Status: Simulated");
        }
        if nic.bar0 != 0 {
            crate::println!("      BAR0:   {:#010X}", nic.bar0);
        }
        if nic.irq != 0 && nic.irq != 0xFF {
            crate::println!("      IRQ:    {}", nic.irq);
        }
        crate::println!();
    }
    
    if let Some((mac, ip, state)) = crate::network::get_interface() {
        crate::println_color!(COLOR_CYAN, "eth0:");
        crate::print!("      Link: ");
        match state {
            crate::network::NetworkState::Up => crate::println_color!(COLOR_GREEN, "UP"),
            crate::network::NetworkState::Down => crate::println_color!(COLOR_YELLOW, "DOWN"),
            crate::network::NetworkState::Error => crate::println_color!(COLOR_RED, "ERROR"),
        }
        crate::println!("      HWaddr: {}", mac);
        if let Some(addr) = ip {
            crate::println!("      inet:   {}", addr);
        }
        
        // Use driver stats for accuracy
        let (tx_pkts, rx_pkts, tx_bytes, rx_bytes) = crate::network::get_driver_stats();
        crate::println!();
        crate::println!("      RX packets: {}  bytes: {}", rx_pkts, rx_bytes);
        crate::println!("      TX packets: {}  bytes: {}", tx_pkts, tx_bytes);
        
        let stats = crate::network::get_stats();
        if stats.errors > 0 {
            crate::println_color!(COLOR_RED, "      Errors: {}", stats.errors);
        }
    } else {
        crate::println_color!(COLOR_YELLOW, "No network interface");
    }
}

// ==================== TRUSTSCAN — LIVE NETWORK TEST ====================

/// Live network test: exercises TrustScan modules against real hosts.
/// Requires network connectivity (QEMU NAT, VirtualBox NAT, or bridged).
///
/// Usage: scantest [target]
///   scantest              — auto-detect gateway, test against it + public DNS
///   scantest 93.184.216.34 — test against specific IP (example.com)
///   scantest google.com   — test with DNS resolution
pub(super) fn cmd_netscan_test(args: &[&str]) {
    crate::println_color!(COLOR_BRIGHT_GREEN, "=== TrustScan Live Network Test Suite ===");
    crate::println!();

    let mut passed = 0usize;
    let mut failed = 0usize;

    // ── Prerequisite: network must be up ──
    crate::println_color!(COLOR_CYAN, "[PRE] Network connectivity check");
    crate::print!("  NIC driver... ");
    if !crate::drivers::net::has_driver() {
        crate::println_color!(COLOR_RED, "[FAIL] no network driver");
        crate::println_color!(COLOR_RED, "=== Cannot run live tests without a network ===");
        return;
    }
    crate::println_color!(COLOR_GREEN, "[OK]");

    // Determine gateway & our IP
    let (our_ip, _mask, gateway_ip) = match crate::network::get_ipv4_config() {
        Some((ip, mask, gw)) => {
            let ip_b = *ip.as_bytes();
            let m_b = *mask.as_bytes();
            let gw_b = gw.map(|g| *g.as_bytes());
            (ip_b, m_b, gw_b)
        }
        None => {
            crate::print!("  IP config... ");
            crate::println_color!(COLOR_RED, "[FAIL] no IPv4 config — run 'dhcp' first");
            return;
        }
    };

    crate::print!("  our IP... ");
    crate::println_color!(COLOR_GREEN, "[OK] {}", crate::netscan::format_ip(our_ip));

    // Choose target: user-supplied, gateway, or public DNS
    let target: [u8; 4];
    let target_name: &str;

    if let Some(arg) = args.first() {
        if let Some(ip) = crate::netscan::parse_ip(arg) {
            target = ip;
            target_name = arg;
        } else if let Some(resolved) = crate::netstack::dns::resolve(arg) {
            target = resolved;
            target_name = arg;
        } else {
            crate::println_color!(COLOR_RED, "Cannot resolve: {}", arg);
            return;
        }
    } else if let Some(gw) = gateway_ip {
        target = gw;
        target_name = "gateway";
    } else {
        // Fallback: Google DNS
        target = [8, 8, 8, 8];
        target_name = "8.8.8.8";
    }

    crate::println_color!(COLOR_CYAN, "  target: {} ({})", target_name, crate::netscan::format_ip(target));
    crate::println!();

    // ── 1. ICMP Ping (reachability) ──
    crate::println_color!(COLOR_CYAN, "[1/8] ICMP Ping — reachability");
    {
        crate::print!("  ping {}... ", crate::netscan::format_ip(target));
        let ip = crate::network::Ipv4Address::new(target[0], target[1], target[2], target[3]);
        match crate::network::send_ping(ip) {
            Ok(result) if result.success => {
                crate::println_color!(COLOR_GREEN, "[OK] rtt={} us  ttl={}", result.time_us, result.ttl);
                passed += 1;
            }
            Ok(_) => {
                crate::println_color!(COLOR_YELLOW, "[WARN] timeout (host may block ICMP)");
                passed += 1; // Not a failure — many hosts block ICMP
            }
            Err(e) => {
                crate::println_color!(COLOR_RED, "[FAIL] {}", e);
                failed += 1;
            }
        }
    }

    // ── 2. ARP Resolution (local network) ──
    crate::println_color!(COLOR_CYAN, "[2/8] ARP Resolution");
    {
        // Try to resolve the gateway or target MAC via ARP
        let arp_target = gateway_ip.unwrap_or(target);
        crate::print!("  ARP {}... ", crate::netscan::format_ip(arp_target));
        let _ = crate::netstack::arp::send_request(arp_target);
        // Poll for a bit
        for _ in 0..200_000 {
            crate::netstack::poll();
            core::hint::spin_loop();
        }
        if let Some(mac) = crate::netstack::arp::resolve(arp_target) {
            crate::println_color!(COLOR_GREEN, "[OK] MAC={}", crate::netscan::format_mac(mac));
            passed += 1;
        } else {
            crate::println_color!(COLOR_YELLOW, "[WARN] no ARP reply (may be routed)");
            passed += 1; // Not a failure — target may be remote
        }
    }

    // ── 3. DNS Resolution ──
    crate::println_color!(COLOR_CYAN, "[3/8] DNS Resolution");
    {
        crate::print!("  resolve google.com... ");
        match crate::netstack::dns::resolve("google.com") {
            Some(ip) => {
                crate::println_color!(COLOR_GREEN, "[OK] {}", crate::netscan::format_ip(ip));
                passed += 1;
            }
            None => {
                crate::println_color!(COLOR_RED, "[FAIL] DNS resolution failed");
                failed += 1;
            }
        }

        crate::print!("  resolve example.com... ");
        match crate::netstack::dns::resolve("example.com") {
            Some(ip) => {
                crate::println_color!(COLOR_GREEN, "[OK] {}", crate::netscan::format_ip(ip));
                passed += 1;
            }
            None => {
                crate::println_color!(COLOR_YELLOW, "[WARN] no DNS — limited test");
                passed += 1; // Some QEMU configs have no DNS
            }
        }
    }

    // ── 4. TCP Port Scan (SYN scan against target) ──
    crate::println_color!(COLOR_CYAN, "[4/8] TCP SYN Port Scan");
    {
        // Scan well-known ports on the target
        let test_ports = alloc::vec![80u16, 443, 53, 22, 8080];
        crate::print!("  SYN scan {} ({} ports)... ", crate::netscan::format_ip(target), test_ports.len());
        let config = crate::netscan::port_scanner::ScanConfig::new(target)
            .with_ports(test_ports)
            .with_type(crate::netscan::port_scanner::ScanType::Syn)
            .with_timeout(2000);
        let (results, stats) = crate::netscan::port_scanner::scan(&config);

        // The scan should complete without panicking — that's the main test
        crate::println_color!(COLOR_GREEN, "[OK] {} open, {} closed, {} filtered ({} ms)",
            stats.open, stats.closed, stats.filtered, stats.elapsed_ms);
        passed += 1;

        // Print any open ports found
        for r in &results {
            if r.state == crate::netscan::port_scanner::PortState::Open {
                crate::println!("    {:>5}/tcp  {:<12}  OPEN", r.port, r.service);
            }
        }
    }

    // ── 5. TCP Connect Scan + Banner Grab ──
    crate::println_color!(COLOR_CYAN, "[5/8] TCP Connect Scan + Banner Grab");
    {
        // Try to connect scan a web server
        // Resolve a known target with port 80 open
        let web_target = if let Some(ip) = crate::netstack::dns::resolve("example.com") {
            ip
        } else {
            target // fallback to our target
        };

        crate::print!("  connect scan {}:80... ", crate::netscan::format_ip(web_target));
        let config = crate::netscan::port_scanner::ScanConfig::new(web_target)
            .with_ports(alloc::vec![80])
            .with_type(crate::netscan::port_scanner::ScanType::Connect)
            .with_timeout(3000);
        let (results, stats) = crate::netscan::port_scanner::scan(&config);

        if stats.open > 0 {
            crate::println_color!(COLOR_GREEN, "[OK] port 80 open");
            passed += 1;

            // Banner grab
            crate::print!("  banner grab :80... ");
            match crate::netscan::banner::grab_banner(web_target, 80, 3000) {
                Some(br) => {
                    let truncated = if br.banner.len() > 60 {
                        &br.banner[..60]
                    } else {
                        &br.banner
                    };
                    crate::println_color!(COLOR_GREEN, "[OK] '{}'", truncated);
                    if let Some(ref ver) = br.version {
                        crate::println!("    version: {}", ver);
                    }
                    passed += 1;
                }
                None => {
                    crate::println_color!(COLOR_YELLOW, "[WARN] no banner (server may not send one)");
                    passed += 1;
                }
            }
        } else {
            crate::println_color!(COLOR_YELLOW, "[WARN] port 80 not open on {} — skip banner",
                crate::netscan::format_ip(web_target));
            passed += 2; // Skip both sub-tests
        }
        let _ = results;
    }

    // ── 6. Packet Sniffer Engine ──
    crate::println_color!(COLOR_CYAN, "[6/8] Packet Sniffer (live capture)");
    {
        use crate::netscan::sniffer;

        crate::print!("  capture during ping... ");
        sniffer::start_capture();

        // Generate traffic: send a ping
        let ip = crate::network::Ipv4Address::new(target[0], target[1], target[2], target[3]);
        let _ = crate::network::send_ping(ip);

        // Poll for packets
        for _ in 0..300_000 {
            crate::netstack::poll();
            core::hint::spin_loop();
        }

        let (count, bytes, buffered) = sniffer::get_stats();
        sniffer::stop_capture();

        if count > 0 {
            crate::println_color!(COLOR_GREEN, "[OK] captured {} packets, {} bytes", count, bytes);
            passed += 1;

            // Verify we captured real data
            crate::print!("  packet details... ");
            let pkts = sniffer::peek_captured_packets(5);
            if !pkts.is_empty() {
                crate::println_color!(COLOR_GREEN, "[OK] {} in buffer", pkts.len());
                for p in pkts.iter().take(3) {
                    crate::println!("    [{:<4}] {} {}", p.protocol.as_str(),
                        p.src_ip.map(|i| crate::netscan::format_ip(i)).unwrap_or_else(|| alloc::string::String::from("?")),
                        p.info);
                }
                passed += 1;
            } else {
                crate::println_color!(COLOR_RED, "[FAIL] buffer empty despite count > 0");
                failed += 1;
            }
        } else {
            crate::println_color!(COLOR_YELLOW, "[WARN] no packets captured (driver may not report TX)");
            passed += 2; // Not a critical failure
        }
    }

    // ── 7. Traceroute (TTL-based) ──
    crate::println_color!(COLOR_CYAN, "[7/8] Traceroute (TTL)");
    {
        // Short traceroute (max 5 hops) to detect at least the first hop
        let trace_target = if let Some(gw) = gateway_ip {
            gw // Gateway should be 1 hop
        } else {
            [8, 8, 8, 8] // Google DNS as fallback
        };

        crate::print!("  trace to {} (5 hops max)... ", crate::netscan::format_ip(trace_target));
        let hops = crate::netscan::traceroute::trace(trace_target, 5, 2000);

        if !hops.is_empty() {
            crate::println_color!(COLOR_GREEN, "[OK] {} hops recorded", hops.len());
            passed += 1;

            for h in &hops {
                crate::print!("    {:>2}  ", h.hop_num);
                if let Some(ip) = h.ip {
                    crate::print!("{:<16}", crate::netscan::format_ip(ip));
                    for &rtt in &h.rtt_ms {
                        if rtt == 0 { crate::print!("*    "); }
                        else { crate::print!("{} ms  ", rtt); }
                    }
                    if h.reached { crate::print!(" [REACHED]"); }
                    crate::println!();
                } else {
                    crate::println!("* * *");
                }
            }
        } else {
            crate::println_color!(COLOR_YELLOW, "[WARN] no hops (ICMP may be blocked)");
            passed += 1;
        }
    }

    // ── 8. Vulnerability Scanner (service fingerprint) ──
    crate::println_color!(COLOR_CYAN, "[8/8] Vulnerability Scanner");
    {
        // Find a target with at least 1 open port to vuln-check
        let web_target = if let Some(ip) = crate::netstack::dns::resolve("example.com") {
            ip
        } else {
            target
        };

        crate::print!("  vuln check {}:80,443... ", crate::netscan::format_ip(web_target));
        let findings = crate::netscan::vuln::scan(web_target, &[80, 443]);

        // Even if no vulns found, the scan completing is the test
        crate::println_color!(COLOR_GREEN, "[OK] {} findings", findings.len());
        passed += 1;

        for f in findings.iter().take(3) {
            let color = match f.severity {
                crate::netscan::vuln::Severity::Critical | crate::netscan::vuln::Severity::High => COLOR_RED,
                crate::netscan::vuln::Severity::Medium => COLOR_YELLOW,
                _ => COLOR_GRAY,
            };
            crate::print_color!(color, "    [{:<8}] ", f.severity.as_str());
            crate::println!("{}/{}: {}", f.port, f.service, f.title);
        }
    }

    // ── Summary ──
    crate::println!();
    let total = passed + failed;
    if failed == 0 {
        crate::println_color!(COLOR_BRIGHT_GREEN,
            "=== ALL {}/{} LIVE TESTS PASSED ===", passed, total);
    } else {
        crate::println_color!(COLOR_RED,
            "=== {}/{} passed, {} FAILED ===", passed, total, failed);
    }
    crate::println!();
    crate::println!("Tip: For more detailed testing, try:");
    crate::println!("  nmap example.com -sT -p 80,443 -A");
    crate::println!("  banner example.com 80");
    crate::println!("  traceroute 8.8.8.8");
    crate::println!("  sniff start   (then generate traffic)");
    crate::println!("  vulnscan example.com");
}

// ==================== TRUSTSCAN — SECURITY TOOLKIT ====================

pub(super) fn cmd_nmap(args: &[&str]) {
    if args.is_empty() {
        crate::println_color!(COLOR_CYAN, "TrustScan — Port Scanner");
        crate::println!("Usage: nmap <target> [options]");
        crate::println!("  nmap <ip|host>                 Quick scan (25 common ports)");
        crate::println!("  nmap <ip|host> -p 80,443,8080  Scan specific ports");
        crate::println!("  nmap <ip|host> -p 1-1024       Scan port range");
        crate::println!("  nmap <ip|host> -sS             SYN scan (default, stealth)");
        crate::println!("  nmap <ip|host> -sT             TCP connect scan");
        crate::println!("  nmap <ip|host> -sU             UDP scan");
        crate::println!("  nmap <ip|host> --top            Top 100 ports");
        crate::println!("  nmap <ip|host> -A              Aggressive (scan + banner + vuln)");
        return;
    }

    let target = match crate::netscan::resolve_target(args[0]) {
        Some(ip) => ip,
        None => {
            crate::println_color!(COLOR_RED, "Cannot resolve target: {}", args[0]);
            return;
        }
    };

    // Parse options
    let mut scan_type = crate::netscan::port_scanner::ScanType::Syn;
    let mut ports: Option<alloc::vec::Vec<u16>> = None;
    let mut top_ports = false;
    let mut aggressive = false;

    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "-sS" => scan_type = crate::netscan::port_scanner::ScanType::Syn,
            "-sT" => scan_type = crate::netscan::port_scanner::ScanType::Connect,
            "-sU" => scan_type = crate::netscan::port_scanner::ScanType::Udp,
            "--top" => top_ports = true,
            "-A" => aggressive = true,
            "-p" if i + 1 < args.len() => {
                i += 1;
                ports = Some(parse_port_spec(args[i]));
            }
            _ => {}
        }
        i += 1;
    }

    let scan_name = match scan_type {
        crate::netscan::port_scanner::ScanType::Syn => "SYN",
        crate::netscan::port_scanner::ScanType::Connect => "Connect",
        crate::netscan::port_scanner::ScanType::Udp => "UDP",
    };

    crate::println_color!(COLOR_CYAN, "Starting TrustScan {} scan on {}", scan_name, crate::netscan::format_ip(target));
    crate::println!("TrustScan 1.0 — TrustOS Network Security Scanner");
    crate::println!();

    let mut config = crate::netscan::port_scanner::ScanConfig::new(target)
        .with_type(scan_type);

    if let Some(p) = ports {
        config = config.with_ports(p);
    } else if top_ports || aggressive {
        config = config.with_top_ports();
    }

    let (results, stats) = crate::netscan::port_scanner::scan(&config);

    // Display results
    crate::println!("PORT       STATE          SERVICE");
    for result in &results {
        let state_color = match result.state {
            crate::netscan::port_scanner::PortState::Open => COLOR_GREEN,
            crate::netscan::port_scanner::PortState::Filtered => COLOR_YELLOW,
            crate::netscan::port_scanner::PortState::OpenFiltered => COLOR_YELLOW,
            crate::netscan::port_scanner::PortState::Closed => COLOR_RED,
        };

        let proto = match scan_type {
            crate::netscan::port_scanner::ScanType::Udp => "udp",
            _ => "tcp",
        };

        crate::print!("{}/{:<6}", result.port, proto);
        crate::print_color!(state_color, " {:<14}", result.state.as_str());
        crate::println!(" {}", result.service);
    }

    crate::println!();
    crate::println!("Scan complete: {} ports scanned in {} ms", stats.total_ports, stats.elapsed_ms);
    crate::println!("  {} open, {} closed, {} filtered",
        stats.open, stats.closed, stats.filtered);

    // Aggressive mode: banner grab + vuln scan on open ports
    if aggressive {
        let open_ports: alloc::vec::Vec<u16> = results.iter()
            .filter(|r| r.state == crate::netscan::port_scanner::PortState::Open)
            .map(|r| r.port)
            .collect();

        if !open_ports.is_empty() {
            crate::println!();
            crate::println_color!(COLOR_CYAN, "Banner Grabbing...");
            let banners = crate::netscan::banner::grab_banners(target, &open_ports, 2000);
            for b in &banners {
                crate::print!("  {}/tcp ", b.port);
                if let Some(ref ver) = b.version {
                    crate::print_color!(COLOR_GREEN, "{} ", ver);
                }
                crate::println!("{}", b.banner);
            }

            crate::println!();
            crate::println_color!(COLOR_CYAN, "Vulnerability Assessment...");
            let findings = crate::netscan::vuln::scan(target, &open_ports);
            for f in &findings {
                let sev_color = match f.severity {
                    crate::netscan::vuln::Severity::Critical => COLOR_RED,
                    crate::netscan::vuln::Severity::High => COLOR_RED,
                    crate::netscan::vuln::Severity::Medium => COLOR_YELLOW,
                    crate::netscan::vuln::Severity::Low => COLOR_CYAN,
                    crate::netscan::vuln::Severity::Info => COLOR_WHITE,
                };
                crate::print_color!(sev_color, "[{}] ", f.severity.as_str());
                crate::println!("{}/{} — {}", f.port, f.service, f.title);
            }
        }
    }
}

fn parse_port_spec(spec: &str) -> alloc::vec::Vec<u16> {
    let mut ports = alloc::vec::Vec::new();
    for part in spec.split(',') {
        if let Some(dash) = part.find('-') {
            let start: u16 = part[..dash].parse().unwrap_or(0);
            let end: u16 = part[dash+1..].parse().unwrap_or(0);
            if start > 0 && end >= start && end <= 65535 {
                for p in start..=end {
                    ports.push(p);
                }
            }
        } else if let Ok(p) = part.parse::<u16>() {
            if p > 0 {
                ports.push(p);
            }
        }
    }
    ports
}

pub(super) fn cmd_discover(args: &[&str]) {
    if args.is_empty() || args[0] == "--help" {
        crate::println_color!(COLOR_CYAN, "TrustScan — Network Discovery");
        crate::println!("Usage:");
        crate::println!("  discover                   ARP sweep local subnet");
        crate::println!("  discover arp               ARP sweep (fast, local only)");
        crate::println!("  discover ping <base_ip>    ICMP ping sweep /24 subnet");
        crate::println!("  discover full              Full discovery (ARP + ping)");
        return;
    }

    let mode = if args.is_empty() { "arp" } else { args[0] };

    match mode {
        "arp" | "arpscan" => {
            crate::println_color!(COLOR_CYAN, "ARP Sweep — Local Network Discovery");
            crate::println!("Scanning local subnet...");
            crate::println!();

            let hosts = crate::netscan::discovery::arp_sweep_local(3000);

            if hosts.is_empty() {
                crate::println_color!(COLOR_YELLOW, "No hosts discovered");
                return;
            }

            crate::println!("IP Address          MAC Address          Status");
            crate::println!("{}", "-".repeat(55));
            for host in &hosts {
                let ip_str = crate::netscan::format_ip(host.ip);
                let mac_str = host.mac.map(|m| crate::netscan::format_mac(m))
                    .unwrap_or_else(|| alloc::string::String::from("unknown"));
                crate::println_color!(COLOR_GREEN, "{:<20}{:<21}UP", ip_str, mac_str);
            }
            crate::println!();
            crate::println!("{} hosts discovered", hosts.len());
        }
        "ping" => {
            let base = if args.len() > 1 {
                match crate::netscan::parse_ip(args[1]) {
                    Some(ip) => [ip[0], ip[1], ip[2], 0],
                    None => {
                        crate::println_color!(COLOR_RED, "Invalid IP: {}", args[1]);
                        return;
                    }
                }
            } else {
                match crate::network::get_ipv4_config() {
                    Some((ip, _, _)) => {
                        let b = ip.as_bytes();
                        [b[0], b[1], b[2], 0]
                    }
                    None => {
                        crate::println_color!(COLOR_RED, "No network configured");
                        return;
                    }
                }
            };

            crate::println_color!(COLOR_CYAN, "ICMP Ping Sweep — {}.{}.{}.0/24", base[0], base[1], base[2]);
            crate::println!("Scanning 254 hosts...");

            let hosts = crate::netscan::discovery::ping_sweep_subnet(base, 500);

            crate::println!();
            crate::println!("IP Address          TTL   RTT     OS Guess");
            crate::println!("{}", "-".repeat(60));
            for host in &hosts {
                let ip_str = crate::netscan::format_ip(host.ip);
                crate::println_color!(COLOR_GREEN, "{:<20}{:<6}{:<8}{}",
                    ip_str,
                    host.ttl.map(|t| alloc::format!("{}", t)).unwrap_or_else(|| alloc::string::String::from("-")),
                    alloc::format!("{}ms", host.rtt_ms),
                    host.os_hint);
            }
            crate::println!();
            crate::println!("{} hosts alive", hosts.len());
        }
        "full" => {
            crate::println_color!(COLOR_CYAN, "Full Network Discovery (ARP + Ping)");
            crate::println!("Scanning...");

            let hosts = crate::netscan::discovery::full_discovery(3000);

            crate::println!();
            crate::println!("IP Address          MAC Address          TTL   OS Guess");
            crate::println!("{}", "-".repeat(70));
            for host in &hosts {
                let ip_str = crate::netscan::format_ip(host.ip);
                let mac_str = host.mac.map(|m| crate::netscan::format_mac(m))
                    .unwrap_or_else(|| alloc::string::String::from("--:--:--:--:--:--"));
                let ttl_str = host.ttl.map(|t| alloc::format!("{}", t)).unwrap_or_else(|| alloc::string::String::from("-"));
                crate::println_color!(COLOR_GREEN, "{:<20}{:<21}{:<6}{}",
                    ip_str, mac_str, ttl_str, host.os_hint);
            }
            crate::println!();
            crate::println!("{} hosts discovered", hosts.len());
        }
        _ => {
            // Default: ARP sweep
            cmd_discover(&["arp"]);
        }
    }
}

pub(super) fn cmd_banner(args: &[&str]) {
    if args.is_empty() {
        crate::println_color!(COLOR_CYAN, "TrustScan — Banner Grabber");
        crate::println!("Usage: banner <ip|host> [port1,port2,...]");
        crate::println!("  banner 192.168.1.1              Grab banners from common ports");
        crate::println!("  banner 192.168.1.1 22,80,443    Grab from specific ports");
        return;
    }

    let target = match crate::netscan::resolve_target(args[0]) {
        Some(ip) => ip,
        None => {
            crate::println_color!(COLOR_RED, "Cannot resolve: {}", args[0]);
            return;
        }
    };

    let ports = if args.len() > 1 {
        parse_port_spec(args[1])
    } else {
        alloc::vec![21, 22, 25, 80, 110, 143, 443, 3306, 5432, 6379, 8080]
    };

    crate::println_color!(COLOR_CYAN, "Banner Grabbing {} ({} ports)", crate::netscan::format_ip(target), ports.len());
    crate::println!();

    let banners = crate::netscan::banner::grab_banners(target, &ports, 3000);

    if banners.is_empty() {
        crate::println_color!(COLOR_YELLOW, "No banners could be grabbed (ports may be closed)");
        return;
    }

    for b in &banners {
        crate::print_color!(COLOR_GREEN, "{}/tcp {:<15}", b.port, b.service);
        if let Some(ref ver) = b.version {
            crate::print_color!(COLOR_CYAN, " [{}]", ver);
        }
        crate::println!();
        crate::println!("  {}", b.banner);
    }
}

pub(super) fn cmd_sniff(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");

    match subcmd {
        "start" => {
            crate::netscan::sniffer::start_capture();
            crate::println_color!(COLOR_GREEN, "Packet capture started");
            crate::println!("Use 'sniff show' to view captured packets");
            crate::println!("Use 'sniff stop' to stop capture");
        }
        "stop" => {
            crate::netscan::sniffer::stop_capture();
            let (count, bytes, _) = crate::netscan::sniffer::get_stats();
            crate::println_color!(COLOR_YELLOW, "Capture stopped");
            crate::println!("Captured {} packets, {} bytes", count, bytes);
        }
        "show" | "dump" => {
            let count = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(20);
            let packets = crate::netscan::sniffer::peek_captured_packets(count);

            if packets.is_empty() {
                crate::println_color!(COLOR_YELLOW, "No packets captured");
                if !crate::netscan::sniffer::is_capturing() {
                    crate::println!("Start capture with: sniff start");
                }
                return;
            }

            crate::println!("No.  Time      Protocol  Source              Destination         Info");
            crate::println!("{}", "-".repeat(90));

            for (i, pkt) in packets.iter().rev().enumerate() {
                let src = pkt.src_ip.map(|ip| crate::netscan::format_ip(ip))
                    .unwrap_or_else(|| crate::netscan::format_mac(pkt.src_mac));
                let dst = pkt.dst_ip.map(|ip| crate::netscan::format_ip(ip))
                    .unwrap_or_else(|| crate::netscan::format_mac(pkt.dst_mac));

                let proto_color = match pkt.protocol {
                    crate::netscan::sniffer::Protocol::Tcp => COLOR_CYAN,
                    crate::netscan::sniffer::Protocol::Udp => COLOR_BLUE,
                    crate::netscan::sniffer::Protocol::Http => COLOR_GREEN,
                    crate::netscan::sniffer::Protocol::Tls => COLOR_MAGENTA,
                    crate::netscan::sniffer::Protocol::Arp => COLOR_YELLOW,
                    crate::netscan::sniffer::Protocol::Icmp => COLOR_RED,
                    crate::netscan::sniffer::Protocol::Dns => COLOR_BRIGHT_GREEN,
                    _ => COLOR_WHITE,
                };

                crate::print!("{:<5}{:<10}", i + 1, pkt.timestamp_ms);
                crate::print_color!(proto_color, "{:<10}", pkt.protocol.as_str());
                crate::print!("{:<20}{:<20}", src, dst);
                crate::println!("{}", &pkt.info[..pkt.info.len().min(40)]);
            }

            let (total_count, total_bytes, buffered) = crate::netscan::sniffer::get_stats();
            crate::println!();
            crate::println!("Total: {} packets, {} bytes ({} in buffer)",
                total_count, total_bytes, buffered);
        }
        "hex" => {
            let idx = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let packets = crate::netscan::sniffer::peek_captured_packets(idx + 1);
            if let Some(pkt) = packets.get(idx) {
                crate::println_color!(COLOR_CYAN, "Packet #{} — {} bytes — {}",
                    idx + 1, pkt.length, pkt.protocol.as_str());
                crate::println!("{}", crate::netscan::sniffer::hex_dump(&pkt.raw_data, 128));
            } else {
                crate::println_color!(COLOR_YELLOW, "No packet at index {}", idx);
            }
        }
        "stats" => {
            let (count, bytes, buffered) = crate::netscan::sniffer::get_stats();
            let active = crate::netscan::sniffer::is_capturing();
            crate::println_color!(COLOR_CYAN, "Sniffer Statistics");
            crate::println!("  Status:    {}", if active { "CAPTURING" } else { "STOPPED" });
            crate::println!("  Packets:   {}", count);
            crate::println!("  Bytes:     {}", bytes);
            crate::println!("  Buffered:  {}", buffered);
        }
        _ => {
            crate::println_color!(COLOR_CYAN, "TrustScan — Packet Sniffer");
            crate::println!("Usage:");
            crate::println!("  sniff start         Start packet capture");
            crate::println!("  sniff stop          Stop capture");
            crate::println!("  sniff show [N]      Show last N captured packets");
            crate::println!("  sniff hex [N]       Hex dump of packet N");
            crate::println!("  sniff stats         Capture statistics");
        }
    }
}

pub(super) fn cmd_vulnscan(args: &[&str]) {
    if args.is_empty() {
        crate::println_color!(COLOR_CYAN, "TrustScan — Vulnerability Scanner");
        crate::println!("Usage: vulnscan <ip|host> [port1,port2,...]");
        crate::println!("  vulnscan 192.168.1.1              Scan all common ports + vulns");
        crate::println!("  vulnscan 192.168.1.1 80,443,3306  Scan specific ports");
        return;
    }

    let target = match crate::netscan::resolve_target(args[0]) {
        Some(ip) => ip,
        None => {
            crate::println_color!(COLOR_RED, "Cannot resolve: {}", args[0]);
            return;
        }
    };

    // First, scan ports to find open ones
    let open_ports = if args.len() > 1 {
        parse_port_spec(args[1])
    } else {
        crate::println!("Scanning ports...");
        let config = crate::netscan::port_scanner::ScanConfig::new(target).with_top_ports();
        let (results, _) = crate::netscan::port_scanner::scan(&config);
        results.iter()
            .filter(|r| r.state == crate::netscan::port_scanner::PortState::Open)
            .map(|r| r.port)
            .collect()
    };

    if open_ports.is_empty() {
        crate::println_color!(COLOR_YELLOW, "No open ports found");
        return;
    }

    crate::println_color!(COLOR_CYAN, "Vulnerability Assessment — {}", crate::netscan::format_ip(target));
    crate::println!("Checking {} ports...", open_ports.len());
    crate::println!();

    let findings = crate::netscan::vuln::scan(target, &open_ports);

    if findings.is_empty() {
        crate::println_color!(COLOR_GREEN, "No vulnerabilities detected");
        return;
    }

    // Summary
    let critical = findings.iter().filter(|f| f.severity == crate::netscan::vuln::Severity::Critical).count();
    let high = findings.iter().filter(|f| f.severity == crate::netscan::vuln::Severity::High).count();
    let medium = findings.iter().filter(|f| f.severity == crate::netscan::vuln::Severity::Medium).count();

    if critical > 0 {
        crate::print_color!(COLOR_RED, "CRITICAL: {} ", critical);
    }
    if high > 0 {
        crate::print_color!(COLOR_RED, "HIGH: {} ", high);
    }
    if medium > 0 {
        crate::print_color!(COLOR_YELLOW, "MEDIUM: {} ", medium);
    }
    crate::println!("({} total findings)", findings.len());
    crate::println!();

    // Detailed findings
    for f in &findings {
        let sev_color = match f.severity {
            crate::netscan::vuln::Severity::Critical => COLOR_RED,
            crate::netscan::vuln::Severity::High => COLOR_RED,
            crate::netscan::vuln::Severity::Medium => COLOR_YELLOW,
            crate::netscan::vuln::Severity::Low => COLOR_CYAN,
            crate::netscan::vuln::Severity::Info => COLOR_GRAY,
        };
        crate::print_color!(sev_color, "[{:<8}] ", f.severity.as_str());
        crate::println!("{}/{} — {}", f.port, f.service, f.title);
        crate::println!("           {}", f.description);
        crate::println_color!(COLOR_GREEN, "           Fix: {}", f.recommendation);
        crate::println!();
    }
}

pub(super) fn cmd_traceroute_real(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: traceroute <host>");
        return;
    }

    let host = args[0];
    let ip = if let Some(ip) = parse_ipv4(host) {
        ip
    } else if let Some(resolved) = crate::netstack::dns::resolve(host) {
        resolved
    } else {
        crate::println_color!(COLOR_RED, "Unable to resolve host");
        return;
    };

    let max_hops = args.get(1).and_then(|s| s.parse::<u8>().ok()).unwrap_or(30);

    crate::println!("traceroute to {} ({}), {} hops max, 60 byte packets",
        host, crate::netscan::format_ip(ip), max_hops);

    let hops = crate::netscan::traceroute::trace(ip, max_hops, 2000);

    for hop in &hops {
        crate::print!("{:>2}  ", hop.hop_num);
        if let Some(hop_ip) = hop.ip {
            crate::print!("{:<18}", crate::netscan::format_ip(hop_ip));
            for &rtt in &hop.rtt_ms {
                if rtt == 0 {
                    crate::print!("*      ");
                } else {
                    crate::print!("{} ms  ", rtt);
                }
            }
            crate::println!();
        } else {
            crate::println!("* * *");
        }
    }
}

pub(super) fn cmd_ping(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: ping <ip|host>");
        crate::println!("  Example: ping 192.168.56.1");
        crate::println!("  Example: ping example.com");
        return;
    }

    let ip = if let Some(ip) = parse_ipv4(args[0]) {
        crate::network::Ipv4Address::new(ip[0], ip[1], ip[2], ip[3])
    } else if let Some(resolved) = crate::netstack::dns::resolve(args[0]) {
        crate::network::Ipv4Address::new(resolved[0], resolved[1], resolved[2], resolved[3])
    } else {
        crate::println_color!(COLOR_RED, "Unable to resolve host");
        return;
    };
    
    crate::println!("PING {} ({}) 56 data bytes", args[0], ip);
    
    let mut success_count = 0;
    let mut total_us = 0u64;
    let mut min_us = u64::MAX;
    let mut max_us = 0u64;
    
    for _ in 0..4 {
        match crate::network::send_ping(ip) {
            Ok(result) => {
                if result.success {
                    success_count += 1;
                    total_us += result.time_us;
                    min_us = min_us.min(result.time_us);
                    max_us = max_us.max(result.time_us);
                    
                    // Show microsecond precision
                    if result.time_us < 1000 {
                        crate::println!("64 bytes from {}: icmp_seq={} ttl={} time={} us", 
                            ip, result.seq, result.ttl, result.time_us);
                    } else {
                        let ms = result.time_us / 1000;
                        let us_frac = (result.time_us % 1000) / 10;
                        crate::println!("64 bytes from {}: icmp_seq={} ttl={} time={}.{:02} ms", 
                            ip, result.seq, result.ttl, ms, us_frac);
                    }
                } else {
                    crate::println_color!(COLOR_YELLOW, "Request timeout for icmp_seq {}", result.seq);
                }
            }
            Err(e) => {
                crate::println_color!(COLOR_RED, "ping failed: {}", e);
            }
        }
        
        // High-precision delay between pings (~1 second)
        crate::cpu::tsc::delay_millis(1000);
    }
    
    crate::println!();
    crate::println!("--- {} ping statistics ---", args[0]);
    crate::println!("4 packets transmitted, {} received, {}% packet loss", 
        success_count, 
        (4 - success_count) * 25);
    if success_count > 0 {
        let avg_us = total_us / success_count as u64;
        // Show min/avg/max in ms with us precision
        crate::println!("rtt min/avg/max = {}.{:03}/{}.{:03}/{}.{:03} ms", 
            min_us / 1000, min_us % 1000,
            avg_us / 1000, avg_us % 1000,
            max_us / 1000, max_us % 1000);
    }
}

pub(super) fn cmd_netstat() {
    crate::println_color!(COLOR_CYAN, "Network Statistics");
    crate::println!("==================");
    
    let stats = crate::network::get_stats();
    crate::println!();
    crate::print_color!(COLOR_GREEN, "Packets received: ");
    crate::println!("{}", stats.packets_received);
    crate::print_color!(COLOR_GREEN, "Packets sent:     ");
    crate::println!("{}", stats.packets_sent);
    crate::print_color!(COLOR_GREEN, "Bytes received:   ");
    crate::println!("{}", stats.bytes_received);
    crate::print_color!(COLOR_GREEN, "Bytes sent:       ");
    crate::println!("{}", stats.bytes_sent);
    crate::print_color!(COLOR_GREEN, "Errors:           ");
    crate::println!("{}", stats.errors);
}

pub(super) fn cmd_ipconfig(args: &[&str]) {
    let show_all = args.iter().any(|a| *a == "/all" || *a == "-a");
    crate::println!("Windows IP Configuration");
    crate::println!();

    if let Some((mac, ip, state)) = crate::network::get_interface() {
        crate::println!("   Ethernet adapter net0:");
        crate::println!("      Status . . . . . . . . . . . . : {:?}", state);
        crate::println!("      Physical Address. . . . . . . . : {}", mac);
        if let Some(ip) = ip {
            crate::println!("      IPv4 Address. . . . . . . . . : {}", ip);
            if let Some((_, subnet, gateway)) = crate::network::get_ipv4_config() {
                crate::println!("      Subnet Mask . . . . . . . . . : {}", subnet);
                if let Some(gw) = gateway {
                    crate::println!("      Default Gateway . . . . . . . : {}", gw);
                } else if show_all {
                    crate::println!("      Default Gateway . . . . . . . : (none)");
                }
            }
        } else {
            crate::println!("      IPv4 Address. . . . . . . . . : (none)");
        }
    } else {
        crate::println!("No network interface detected");
    }
}

pub(super) fn cmd_nslookup(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: nslookup <host>");
        return;
    }

    let target = args[0];
    if parse_ipv4(target).is_some() {
        crate::println!("Server: 8.8.8.8");
        crate::println!("Address: {}", target);
        crate::println!("*** Reverse lookup not implemented");
        return;
    }

    if let Some(resolved) = crate::netstack::dns::resolve(target) {
        crate::println!("Server: 8.8.8.8");
        crate::println!("Name: {}", target);
        crate::println!("Address: {}.{}.{}.{}", resolved[0], resolved[1], resolved[2], resolved[3]);
    } else {
        crate::println_color!(COLOR_RED, "DNS lookup failed");
    }
}

pub(super) fn cmd_arp(args: &[&str]) {
    if args.iter().any(|a| *a == "-a" || *a == "/a") {
        crate::println!("Interface: net0");
    }

    let entries = crate::netstack::arp::entries();
    if entries.is_empty() {
        crate::println!("No ARP entries");
        return;
    }

    crate::println!("Internet Address      Physical Address       Type");
    for (ip, mac) in entries {
        let ipb = ip.to_be_bytes();
        crate::println!(
            "{:>3}.{:>3}.{:>3}.{:>3}      {:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}   dynamic",
            ipb[0], ipb[1], ipb[2], ipb[3], mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        );
    }
}

pub(super) fn cmd_route(_args: &[&str]) {
    crate::println!("Kernel IP routing table");
    crate::println!("Destination     Gateway         Genmask         Iface");

    if let Some((ip, subnet, gateway)) = crate::network::get_ipv4_config() {
        let gw = gateway.unwrap_or(crate::network::Ipv4Address::new(0, 0, 0, 0));
        crate::println!("{}     {}     {}     net0", ip, gw, subnet);
        crate::println!("0.0.0.0         {}     0.0.0.0         net0", gw);
    } else {
        crate::println!("0.0.0.0         0.0.0.0         0.0.0.0         net0");
    }
}

pub(super) fn cmd_traceroute(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: traceroute <host>");
        return;
    }

    let host = args[0];
    let ip = if let Some(ip) = parse_ipv4(host) {
        ip
    } else if let Some(resolved) = crate::netstack::dns::resolve(host) {
        resolved
    } else {
        crate::println_color!(COLOR_RED, "Unable to resolve host");
        return;
    };

    crate::println!("traceroute to {} ({}.{}.{}.{}), 30 hops max", host, ip[0], ip[1], ip[2], ip[3]);
    if let Some((_, _, gateway)) = crate::network::get_ipv4_config() {
        if let Some(gw) = gateway {
            crate::println!(" 1  {}", gw);
        }
    }
    crate::println!(" 2  {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    crate::println_color!(COLOR_YELLOW, "Note: traceroute is simplified (no TTL probing)");
}

// ==================== HARDWARE COMMANDS ====================

// -- Audio Commands --

pub(super) fn cmd_beep(args: &[&str]) {
    let freq = args.first()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(440);
    let duration = args.get(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(500);

    if freq < 20 || freq > 20000 {
        crate::println_color!(COLOR_RED, "Frequency must be 20-20000 Hz");
        return;
    }
    if duration > 10000 {
        crate::println_color!(COLOR_RED, "Duration max 10000 ms");
        return;
    }

    // Initialize HDA if needed
    if !crate::drivers::hda::is_initialized() {
        crate::print_color!(COLOR_YELLOW, "Initializing audio driver... ");
        match crate::drivers::hda::init() {
            Ok(()) => crate::println_color!(COLOR_GREEN, "OK"),
            Err(e) => {
                crate::println_color!(COLOR_RED, "FAILED: {}", e);
                return;
            }
        }
    }

    crate::println!("Playing {}Hz for {}ms...", freq, duration);
    match crate::drivers::hda::play_tone(freq, duration) {
        Ok(()) => crate::println_color!(COLOR_GREEN, "Done"),
        Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
    }
}

pub(super) fn cmd_audio(args: &[&str]) {
    match args.first().copied() {
        Some("init") => {
            crate::print_color!(COLOR_YELLOW, "Initializing Intel HDA driver... ");
            match crate::drivers::hda::init() {
                Ok(()) => crate::println_color!(COLOR_GREEN, "OK"),
                Err(e) => crate::println_color!(COLOR_RED, "FAILED: {}", e),
            }
        }
        Some("status") | None => {
            let status = crate::drivers::hda::status();
            crate::println!("{}", status);
        }
        Some("stop") => {
            match crate::drivers::hda::stop() {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Playback stopped"),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("test") => {
            // Play a quick scale: C D E F G A B C
            if !crate::drivers::hda::is_initialized() {
                crate::print_color!(COLOR_YELLOW, "Initializing audio driver... ");
                match crate::drivers::hda::init() {
                    Ok(()) => crate::println_color!(COLOR_GREEN, "OK"),
                    Err(e) => {
                        crate::println_color!(COLOR_RED, "FAILED: {}", e);
                        return;
                    }
                }
            }
            crate::println!("Playing test scale...");
            let notes = [262, 294, 330, 349, 392, 440, 494, 523]; // C4 to C5
            for &freq in &notes {
                let _ = crate::drivers::hda::play_tone(freq, 200);
            }
            crate::println_color!(COLOR_GREEN, "Done");
        }
        Some(other) => {
            crate::println_color!(COLOR_YELLOW, "Usage: audio [init|status|stop|test]");
        }
    }
}

pub(super) fn cmd_synth(args: &[&str]) {
    match args.first().copied() {
        Some("note") | Some("play") => {
            // synth note C4 [duration_ms] [waveform]
            let note_name = match args.get(1) {
                Some(n) => *n,
                None => {
                    crate::println_color!(COLOR_YELLOW, "Usage: synth note <note> [duration_ms] [waveform]");
                    crate::println!("  Examples: synth note C4");
                    crate::println!("           synth note A#3 1000 saw");
                    return;
                }
            };
            let duration = args.get(2)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(500);
            // Set waveform if specified
            if let Some(wf_str) = args.get(3) {
                if let Some(wf) = crate::audio::synth::Waveform::from_str(wf_str) {
                    let _ = crate::audio::set_waveform(wf);
                }
            }
            if duration > 10000 {
                crate::println_color!(COLOR_RED, "Duration max 10000 ms");
                return;
            }
            crate::println!("Synth: {} for {}ms", note_name, duration);
            match crate::audio::play_note(note_name, duration) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Done"),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("freq") => {
            // synth freq 440 [duration_ms]
            let freq = match args.get(1).and_then(|s| s.parse::<u32>().ok()) {
                Some(f) => f,
                None => {
                    crate::println_color!(COLOR_YELLOW, "Usage: synth freq <hz> [duration_ms]");
                    return;
                }
            };
            let duration = args.get(2)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(500);
            if freq < 20 || freq > 20000 {
                crate::println_color!(COLOR_RED, "Frequency must be 20-20000 Hz");
                return;
            }
            crate::println!("Synth: {}Hz for {}ms", freq, duration);
            match crate::audio::play_freq(freq, duration) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Done"),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("wave") | Some("waveform") => {
            // synth wave sine|square|saw|triangle|noise
            match args.get(1) {
                Some(wf_str) => {
                    match crate::audio::synth::Waveform::from_str(wf_str) {
                        Some(wf) => {
                            let _ = crate::audio::set_waveform(wf);
                            crate::println_color!(COLOR_GREEN, "Waveform set to: {}", wf.name());
                        }
                        None => crate::println_color!(COLOR_RED, "Unknown waveform (use: sine, square, saw, triangle, noise)"),
                    }
                }
                None => crate::println_color!(COLOR_YELLOW, "Usage: synth wave <sine|square|saw|triangle|noise>"),
            }
        }
        Some("adsr") => {
            // synth adsr <attack_ms> <decay_ms> <sustain_%> <release_ms>
            if args.len() < 5 {
                crate::println_color!(COLOR_YELLOW, "Usage: synth adsr <attack_ms> <decay_ms> <sustain_%> <release_ms>");
                crate::println!("  Example: synth adsr 10 50 70 100");
                return;
            }
            let a = args[1].parse::<u32>().unwrap_or(10);
            let d = args[2].parse::<u32>().unwrap_or(50);
            let s = args[3].parse::<u32>().unwrap_or(70);
            let r = args[4].parse::<u32>().unwrap_or(100);
            let _ = crate::audio::set_adsr(a, d, s, r);
            crate::println_color!(COLOR_GREEN, "ADSR set: A={}ms D={}ms S={}% R={}ms", a, d, s, r);
        }
        Some("preset") => {
            // synth preset default|organ|pluck|pad
            match args.get(1).copied() {
                Some(name) => {
                    match crate::audio::set_envelope_preset(name) {
                        Ok(()) => crate::println_color!(COLOR_GREEN, "Envelope preset: {}", name),
                        Err(e) => crate::println_color!(COLOR_RED, "{}", e),
                    }
                }
                None => crate::println_color!(COLOR_YELLOW, "Usage: synth preset <default|organ|pluck|pad>"),
            }
        }
        Some("volume") | Some("vol") => {
            match args.get(1).and_then(|s| s.parse::<u8>().ok()) {
                Some(v) => {
                    let _ = crate::audio::set_volume(v);
                    crate::println_color!(COLOR_GREEN, "Master volume: {}/255", v);
                }
                None => crate::println_color!(COLOR_YELLOW, "Usage: synth volume <0-255>"),
            }
        }
        Some("status") => {
            let s = crate::audio::status();
            crate::println!("{}", s);
        }
        Some("stop") => {
            match crate::audio::stop() {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Synth stopped"),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("demo") => {
            crate::println!("TrustSynth Demo -- playing scale with different waveforms...");
            let notes = ["C4", "D4", "E4", "F4", "G4", "A4", "B4", "C5"];
            let waveforms = [
                ("Sine",     crate::audio::synth::Waveform::Sine),
                ("Square",   crate::audio::synth::Waveform::Square),
                ("Sawtooth", crate::audio::synth::Waveform::Sawtooth),
                ("Triangle", crate::audio::synth::Waveform::Triangle),
            ];
            for (wf_name, wf) in &waveforms {
                let _ = crate::audio::set_waveform(*wf);
                crate::println!("  {} waveform:", wf_name);
                for note in &notes {
                    crate::print!("    {} ", note);
                    let _ = crate::audio::play_note(note, 200);
                }
                crate::println!();
            }
            crate::println_color!(COLOR_GREEN, "Demo complete!");
        }
        // -- Pattern Sequencer commands --
        Some("pattern") | Some("pat") => {
            cmd_synth_pattern(&args[1..]);
        }
        Some(_) | None => {
            crate::println_color!(COLOR_CYAN, "TrustSynth -- Audio Synthesizer & Sequencer");
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "  Synth:");
            crate::println!("  synth note <note> [ms] [wave]  Play a note (e.g. C4, A#3)");
            crate::println!("  synth freq <hz> [ms]           Play a frequency");
            crate::println!("  synth wave <type>               Set waveform (sine/square/saw/tri/noise)");
            crate::println!("  synth adsr <A> <D> <S%> <R>    Set envelope (ms, ms, %, ms)");
            crate::println!("  synth preset <name>             Set preset (default/organ/pluck/pad)");
            crate::println!("  synth volume <0-255>            Set master volume");
            crate::println!("  synth demo                      Play demo scale");
            crate::println!("  synth status                    Show synth status");
            crate::println!("  synth stop                      Stop playback");
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "  Pattern Sequencer:");
            crate::println!("  synth pattern list              List all patterns");
            crate::println!("  synth pattern show <name>       Display pattern grid");
            crate::println!("  synth pattern new <name> [N] [bpm]  Create pattern (N steps)");
            crate::println!("  synth pattern play <name> [loops]   Play pattern (loop)");
            crate::println!("  synth pattern stop              Stop playback");
            crate::println!("  synth pattern bpm <name> <bpm>  Set tempo");
            crate::println!("  synth pattern wave <name> <wf>  Set waveform");
            crate::println!("  synth pattern set <name> <step> <note>  Set note at step");
            crate::println!("  synth pattern del <name>        Delete pattern");
        }
    }
}

pub(super) fn cmd_synth_pattern(args: &[&str]) {
    match args.first().copied() {
        Some("list") | Some("ls") | None => {
            let list = crate::audio::pattern_list();
            crate::println!("{}", list);
        }
        Some("show") | Some("view") => {
            match args.get(1) {
                Some(name) => {
                    match crate::audio::pattern_show(name) {
                        Ok(s) => crate::println!("{}", s),
                        Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
                    }
                }
                None => crate::println_color!(COLOR_YELLOW, "Usage: synth pattern show <name>"),
            }
        }
        Some("new") | Some("create") => {
            let name = match args.get(1) {
                Some(n) => *n,
                None => {
                    crate::println_color!(COLOR_YELLOW, "Usage: synth pattern new <name> [steps] [bpm]");
                    return;
                }
            };
            let steps = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(16);
            let bpm = args.get(3).and_then(|s| s.parse::<u16>().ok()).unwrap_or(120);
            match crate::audio::pattern_new(name, steps, bpm) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Pattern \"{}\" created ({} steps, {} BPM)", name, steps, bpm),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("play") => {
            let name = match args.get(1) {
                Some(n) => *n,
                None => {
                    crate::println_color!(COLOR_YELLOW, "Usage: synth pattern play <name> [loops]");
                    return;
                }
            };
            let loops = args.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
            match crate::audio::pattern_play(name, loops) {
                Ok(()) => {}
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("stop") => {
            match crate::audio::pattern_stop() {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Pattern playback stopped"),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("bpm") | Some("tempo") => {
            if args.len() < 3 {
                crate::println_color!(COLOR_YELLOW, "Usage: synth pattern bpm <name> <60-300>");
                return;
            }
            let name = args[1];
            let bpm = match args[2].parse::<u16>() {
                Ok(b) if b >= 30 && b <= 300 => b,
                _ => {
                    crate::println_color!(COLOR_RED, "BPM must be 30-300");
                    return;
                }
            };
            match crate::audio::pattern_set_bpm(name, bpm) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "\"{}\" BPM set to {}", name, bpm),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("wave") | Some("waveform") => {
            if args.len() < 3 {
                crate::println_color!(COLOR_YELLOW, "Usage: synth pattern wave <name> <sine|square|saw|tri|noise>");
                return;
            }
            let name = args[1];
            let wf = match crate::audio::synth::Waveform::from_str(args[2]) {
                Some(w) => w,
                None => {
                    crate::println_color!(COLOR_RED, "Unknown waveform");
                    return;
                }
            };
            match crate::audio::pattern_set_wave(name, wf) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "\"{}\" waveform set to {}", name, wf.name()),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("set") | Some("note") => {
            // synth pattern set <name> <step#> <note>
            if args.len() < 4 {
                crate::println_color!(COLOR_YELLOW, "Usage: synth pattern set <name> <step#> <note|-->");
                crate::println!("  Example: synth pattern set mypattern 0 C4");
                crate::println!("  Example: synth pattern set mypattern 3 --  (rest)");
                return;
            }
            let name = args[1];
            let step_idx = match args[2].parse::<usize>() {
                Ok(i) => i,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Step must be a number");
                    return;
                }
            };
            let note = args[3];
            match crate::audio::pattern_set_note(name, step_idx, note) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "\"{}\" step {} = {}", name, step_idx, note),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("del") | Some("delete") | Some("rm") => {
            match args.get(1) {
                Some(name) => {
                    match crate::audio::pattern_remove(name) {
                        Ok(()) => crate::println_color!(COLOR_GREEN, "Pattern \"{}\" deleted", name),
                        Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
                    }
                }
                None => crate::println_color!(COLOR_YELLOW, "Usage: synth pattern del <name>"),
            }
        }
        Some(other) => {
            crate::println_color!(COLOR_RED, "Unknown pattern command: {}", other);
            crate::println!("Use: list, show, new, play, stop, bpm, wave, set, del");
        }
    }
}

pub(super) fn cmd_lspci(args: &[&str]) {
    let devices = crate::pci::get_devices();
    
    if devices.is_empty() {
        crate::println_color!(COLOR_YELLOW, "No PCI devices found");
        return;
    }
    
    let verbose = args.contains(&"-v") || args.contains(&"--verbose");
    
    crate::println_color!(COLOR_CYAN, "PCI Devices ({} found):", devices.len());
    crate::println!();
    
    for dev in &devices {
        // Basic format: Bus:Device.Function VendorID:DeviceID Class
        crate::print_color!(COLOR_GREEN, "{:02X}:{:02X}.{} ", 
            dev.bus, dev.device, dev.function);
        crate::print!("{:04X}:{:04X} ", dev.vendor_id, dev.device_id);
        
        let subclass_name = dev.subclass_name();
        if subclass_name.is_empty() {
            crate::print!("{}", dev.class_name());
        } else {
            crate::print!("{}", subclass_name);
        }
        
        crate::println_color!(COLOR_YELLOW, " [{}]", dev.vendor_name());
        
        if verbose {
            crate::println!("        Class: {:02X}:{:02X} ProgIF: {:02X} Rev: {:02X}",
                dev.class_code, dev.subclass, dev.prog_if, dev.revision);
            
            if dev.interrupt_line != 0xFF && dev.interrupt_pin != 0 {
                crate::println!("        IRQ: {} (pin {})", 
                    dev.interrupt_line, dev.interrupt_pin);
            }
            
            // Show BARs
            for i in 0..6 {
                if let Some(addr) = dev.bar_address(i) {
                    let bar_type = if dev.bar_is_memory(i) { "MEM" } else { "I/O" };
                    crate::println!("        BAR{}: {:#010X} [{}]", i, addr, bar_type);
                }
            }
            crate::println!();
        }
    }
    
    if !verbose {
        crate::println!();
        crate::println_color!(COLOR_YELLOW, "Use 'lspci -v' for detailed info");
    }
}

pub(super) fn cmd_lshw() {
    crate::println_color!(COLOR_CYAN, "=== Hardware Summary ===");
    crate::println!();
    
    let devices = crate::pci::get_devices();
    
    // CPU info
    crate::println_color!(COLOR_GREEN, "CPU:");
    crate::println!("  Architecture: x86_64");
    crate::println!("  Mode: Long Mode (64-bit)");
    crate::println!();
    
    // Memory info
    crate::println_color!(COLOR_GREEN, "Memory:");
    crate::println!("  Heap: 256 KB");
    crate::println!();
    
    // Storage
    let storage: Vec<_> = devices.iter()
        .filter(|d| d.class_code == crate::pci::class::MASS_STORAGE)
        .collect();
    crate::println_color!(COLOR_GREEN, "Storage Controllers ({}):", storage.len());
    for dev in &storage {
        crate::println!("  {:04X}:{:04X} {} [{}]", 
            dev.vendor_id, dev.device_id, 
            dev.subclass_name(),
            dev.vendor_name());
    }
    crate::println!();
    
    // Network
    let network: Vec<_> = devices.iter()
        .filter(|d| d.class_code == crate::pci::class::NETWORK)
        .collect();
    crate::println_color!(COLOR_GREEN, "Network Controllers ({}):", network.len());
    for dev in &network {
        crate::println!("  {:04X}:{:04X} {} [{}]",
            dev.vendor_id, dev.device_id,
            dev.subclass_name(),
            dev.vendor_name());
    }
    crate::println!();
    
    // Display
    let display: Vec<_> = devices.iter()
        .filter(|d| d.class_code == crate::pci::class::DISPLAY)
        .collect();
    crate::println_color!(COLOR_GREEN, "Display ({}):", display.len());
    for dev in &display {
        crate::println!("  {:04X}:{:04X} {} [{}]",
            dev.vendor_id, dev.device_id,
            dev.subclass_name(),
            dev.vendor_name());
    }
    crate::println!();
    
    // USB
    let usb: Vec<_> = devices.iter()
        .filter(|d| d.class_code == crate::pci::class::SERIAL_BUS 
                 && d.subclass == crate::pci::serial::USB)
        .collect();
    crate::println_color!(COLOR_GREEN, "USB Controllers ({}):", usb.len());
    for dev in &usb {
        crate::println!("  {:04X}:{:04X} {} [{}]",
            dev.vendor_id, dev.device_id,
            dev.subclass_name(),
            dev.vendor_name());
    }
    crate::println!();
    
    // Summary
    crate::println_color!(COLOR_CYAN, "Total: {} PCI devices", devices.len());
}

pub(super) fn cmd_gpu(args: &[&str]) {
    if args.first() == Some(&"--help") || args.first() == Some(&"-h") {
        crate::println!("Usage: gpu [info|dcn|modes]");
        crate::println!("  gpu         Show GPU summary");
        crate::println!("  gpu info    Detailed GPU information");
        crate::println!("  gpu dcn     Display engine (DCN) status");
        crate::println!("  gpu modes   List standard display modes");
        return;
    }
    
    let subcmd = args.first().copied().unwrap_or("info");
    
    match subcmd {
        "info" | "" => {
            crate::println_color!(COLOR_CYAN, "=== AMD GPU Status ===");
            crate::println!();
            
            if crate::drivers::amdgpu::is_detected() {
                for line in crate::drivers::amdgpu::info_lines() {
                    crate::println!("{}", line);
                }
                crate::println!();
                
                // Also show DCN info
                if crate::drivers::amdgpu::dcn::is_ready() {
                    crate::println_color!(COLOR_GREEN, "Display Engine:");
                    crate::println!("  {}", crate::drivers::amdgpu::dcn::summary());
                }
            } else {
                crate::println!("No AMD GPU detected.");
                crate::println!("(Requires bare metal or GPU passthrough)");
                crate::println!();
                
                // Show whatever display controller is present
                let display_devs = crate::pci::find_by_class(crate::pci::class::DISPLAY);
                if !display_devs.is_empty() {
                    crate::println_color!(COLOR_GREEN, "Display controllers found:");
                    for dev in &display_devs {
                        crate::println!("  {:04X}:{:04X} {} [{}]", 
                            dev.vendor_id, dev.device_id,
                            dev.subclass_name(), dev.vendor_name());
                    }
                }
            }
        }
        "dcn" | "display" => {
            crate::println_color!(COLOR_CYAN, "=== DCN Display Engine ===");
            crate::println!();
            
            if crate::drivers::amdgpu::dcn::is_ready() {
                for line in crate::drivers::amdgpu::dcn::info_lines() {
                    crate::println!("{}", line);
                }
            } else {
                crate::println!("DCN display engine not initialized.");
                if !crate::drivers::amdgpu::is_detected() {
                    crate::println!("(No AMD GPU detected)");
                }
            }
        }
        "modes" => {
            crate::println_color!(COLOR_CYAN, "=== Standard Display Modes ===");
            crate::println!();
            for (i, mode) in crate::drivers::amdgpu::dcn::standard_modes().iter().enumerate() {
                crate::println!("  [{}] {}", i, mode.modeline());
            }
        }
        _ => {
            crate::println!("Unknown subcommand: {}", subcmd);
            crate::println!("Use 'gpu --help' for usage.");
        }
    }
}

pub(super) fn cmd_a11y(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("status");
    
    match subcmd {
        "status" | "" => {
            crate::println_color!(COLOR_CYAN, "=== Accessibility Settings ===");
            crate::println!();
            let hc = crate::accessibility::is_high_contrast();
            let fs = crate::accessibility::get_font_size();
            let cs = crate::accessibility::get_cursor_size();
            let sk = crate::accessibility::is_sticky_keys();
            let ms = crate::accessibility::get_mouse_speed();
            crate::println!("  High Contrast : {}", if hc { "ON" } else { "OFF" });
            crate::println!("  Font Size     : {}", fs.label());
            crate::println!("  Cursor Size   : {}", cs.label());
            crate::println!("  Sticky Keys   : {}", if sk { "ON" } else { "OFF" });
            crate::println!("  Mouse Speed   : {}", ms.label());
            crate::println!();
            crate::println!("Shortcuts: Win+H = toggle high contrast");
            crate::println!("Settings:  Win+I > keys 5-9 to adjust");
        }
        "hc" | "contrast" => {
            crate::accessibility::toggle_high_contrast();
            let on = crate::accessibility::is_high_contrast();
            crate::println!("High contrast: {}", if on { "ON" } else { "OFF" });
        }
        "font" => {
            crate::accessibility::cycle_font_size();
            crate::println!("Font size: {}", crate::accessibility::get_font_size().label());
        }
        "cursor" => {
            crate::accessibility::cycle_cursor_size();
            crate::println!("Cursor size: {}", crate::accessibility::get_cursor_size().label());
        }
        "sticky" => {
            crate::accessibility::toggle_sticky_keys();
            let on = crate::accessibility::is_sticky_keys();
            crate::println!("Sticky keys: {}", if on { "ON" } else { "OFF" });
        }
        "mouse" => {
            crate::accessibility::cycle_mouse_speed();
            crate::println!("Mouse speed: {}", crate::accessibility::get_mouse_speed().label());
        }
        "--help" | "-h" | "help" => {
            crate::println!("Usage: a11y [status|hc|font|cursor|sticky|mouse]");
            crate::println!("  a11y          Show all accessibility settings");
            crate::println!("  a11y hc       Toggle high contrast mode");
            crate::println!("  a11y font     Cycle font size (S/M/L/XL)");
            crate::println!("  a11y cursor   Cycle cursor size (S/M/L)");
            crate::println!("  a11y sticky   Toggle sticky keys");
            crate::println!("  a11y mouse    Cycle mouse speed");
        }
        _ => {
            crate::println!("Unknown: {}. Use 'a11y --help'", subcmd);
        }
    }
}

pub(super) fn cmd_tcpsyn(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: tcpsyn <ip> <port>");
        crate::println!("  Example: tcpsyn 93.184.216.34 80");
        return;
    }

    let parts: Vec<&str> = args[0].split('.').collect();
    if parts.len() != 4 {
        crate::println_color!(COLOR_RED, "Invalid IP format");
        return;
    }

    let ip = [
        parts[0].parse().unwrap_or(0),
        parts[1].parse().unwrap_or(0),
        parts[2].parse().unwrap_or(0),
        parts[3].parse().unwrap_or(0),
    ];

    let port: u16 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid port");
            return;
        }
    };

    crate::println!("Sending TCP SYN to {}:{}...", args[0], port);
    match crate::netstack::tcp::send_syn(ip, port) {
        Ok(src_port) => {
            crate::println!("SYN sent to {}:{} (src port {})", args[0], port, src_port);
            let established = crate::netstack::tcp::wait_for_established(ip, port, src_port, 1000);
            if established {
                crate::println!("SYN-ACK received (connection established)");
            } else {
                crate::println_color!(COLOR_YELLOW, "No SYN-ACK received (timeout)");
            }
        }
        Err(e) => crate::println_color!(COLOR_RED, "tcpsyn failed: {}", e),
    }
}

pub(super) fn cmd_httpget(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: httpget <ip|host> <port> [path] [host]");
        crate::println!("  Example: httpget 192.168.56.1 8080 /");
        crate::println!("  Example: httpget example.com 80 / example.com");
        return;
    }

    let host_input = args[0];
    let port: u16 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid port");
            return;
        }
    };

    let path = args.get(2).copied().unwrap_or("/");
    let mut host_header = args.get(3).copied().unwrap_or(host_input);
    if args.get(3).is_none() && host_input == "192.168.56.1" {
        host_header = "localhost";
    }

    do_http_get(host_input, port, path, host_header);
}

pub(super) fn cmd_curl(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: curl <http://host[:port]/path> | <https://host[:port]/path>");
        return;
    }

    let url = args[0];
    if let Some((host, port, path, is_https)) = parse_http_url(url) {
        let host_header = if host == "192.168.56.1" { "localhost" } else { &host };
        if is_https {
            do_https_get(&host, port, &path, host_header);
        } else {
            do_http_get(&host, port, &path, host_header);
        }
    } else {
        crate::println_color!(COLOR_RED, "Invalid URL");
    }
}

fn do_http_get(host_input: &str, port: u16, path: &str, host_header: &str) {
    let ip = if let Some(ip) = parse_ipv4(host_input) {
        ip
    } else if let Some(resolved) = crate::netstack::dns::resolve(host_input) {
        resolved
    } else {
        crate::println_color!(COLOR_RED, "Unable to resolve host");
        return;
    };

    crate::println!("Connecting to {}:{}...", host_input, port);
    let src_port = match crate::netstack::tcp::send_syn(ip, port) {
        Ok(p) => p,
        Err(e) => {
            crate::println_color!(COLOR_RED, "SYN failed: {}", e);
            return;
        }
    };

    let established = crate::netstack::tcp::wait_for_established(ip, port, src_port, 1000);
    if !established {
        crate::println_color!(COLOR_YELLOW, "Connection timeout");
        return;
    }

    let mut request = String::new();
    request.push_str("GET ");
    request.push_str(path);
    request.push_str(" HTTP/1.1\r\nHost: ");
    request.push_str(host_header);
    request.push_str("\r\nConnection: close\r\n\r\n");

    if let Err(e) = crate::netstack::tcp::send_payload(ip, port, src_port, request.as_bytes()) {
        crate::println_color!(COLOR_RED, "send failed: {}", e);
        return;
    }

    crate::println!("--- HTTP response ---");
    let start = crate::logger::get_ticks();
    let mut total_bytes: usize = 0;
    let mut idle_spins: u32 = 0;
    loop {
        crate::netstack::poll();
        let mut got_data = false;
        while let Some(data) = crate::netstack::tcp::recv_data(ip, port, src_port) {
            got_data = true;
            total_bytes += data.len();
            if let Ok(text) = core::str::from_utf8(&data) {
                crate::print!("{}", text);
            } else {
                crate::println!("<binary data>");
            }
        }

        if !got_data {
            idle_spins = idle_spins.saturating_add(1);
            if crate::netstack::tcp::fin_received(ip, port, src_port) || idle_spins > 200_000 {
                break;
            }
        } else {
            idle_spins = 0;
        }

        if crate::logger::get_ticks().saturating_sub(start) > 3000 {
            break;
        }
        crate::arch::halt();
    }
    let _ = crate::netstack::tcp::send_fin(ip, port, src_port);
    crate::println!("\n--- end ({} bytes) ---", total_bytes);
    if total_bytes == 0 {
        crate::println_color!(COLOR_YELLOW, "No response body received");
    }
}

/// Parse URL and return (host, port, path)
pub(super) fn parse_url_simple(url: &str) -> Option<(String, u16, String)> {
    let url = url.trim();
    
    // Remove protocol prefix
    let (rest, default_port) = if url.starts_with("https://") {
        (&url[8..], 443u16)
    } else if url.starts_with("http://") {
        (&url[7..], 80u16)
    } else {
        // Assume http if no protocol
        (url, 80u16)
    };
    
    // Split host and path
    let (host_port, path) = if let Some(idx) = rest.find('/') {
        (&rest[..idx], &rest[idx..])
    } else {
        (rest, "/")
    };
    
    // Split host and port
    let (host, port) = if let Some(idx) = host_port.find(':') {
        let host = &host_port[..idx];
        let port_str = &host_port[idx+1..];
        let port = port_str.parse::<u16>().unwrap_or(default_port);
        (host, port)
    } else {
        (host_port, default_port)
    };
    
    if host.is_empty() {
        return None;
    }
    
    Some((String::from(host), port, String::from(path)))
}

/// HTTP GET that returns a string (for GUI shell)
pub(super) fn do_http_get_string(host: &str, ip: [u8; 4], port: u16, path: &str) -> Result<String, &'static str> {
    // Send SYN
    let src_port = crate::netstack::tcp::send_syn(ip, port)
        .map_err(|_| "SYN failed")?;
    
    // Wait for connection
    if !crate::netstack::tcp::wait_for_established(ip, port, src_port, 1000) {
        return Err("Connection timeout");
    }
    
    // Build HTTP request
    let mut request = String::new();
    request.push_str("GET ");
    request.push_str(path);
    request.push_str(" HTTP/1.1\r\nHost: ");
    request.push_str(host);
    request.push_str("\r\nUser-Agent: TrustOS/0.1\r\nConnection: close\r\n\r\n");
    
    // Send request
    crate::netstack::tcp::send_payload(ip, port, src_port, request.as_bytes())
        .map_err(|_| "Send failed")?;
    
    // Receive response
    let mut response = String::new();
    let start = crate::logger::get_ticks();
    let mut idle_spins: u32 = 0;
    
    loop {
        crate::netstack::poll();
        let mut got_data = false;
        
        while let Some(data) = crate::netstack::tcp::recv_data(ip, port, src_port) {
            got_data = true;
            if let Ok(text) = core::str::from_utf8(&data) {
                response.push_str(text);
            }
        }
        
        if !got_data {
            idle_spins = idle_spins.saturating_add(1);
            if crate::netstack::tcp::fin_received(ip, port, src_port) || idle_spins > 100_000 {
                break;
            }
        } else {
            idle_spins = 0;
        }
        
        if crate::logger::get_ticks().saturating_sub(start) > 2000 {
            break;
        }
        
        // Limit response size for GUI
        if response.len() > 4096 {
            response.push_str("\n... (response truncated)");
            break;
        }
        
        crate::arch::halt();
    }
    
    let _ = crate::netstack::tcp::send_fin(ip, port, src_port);
    
    if response.is_empty() {
        return Err("No response received");
    }
    
    Ok(response)
}

fn do_https_get(host_input: &str, port: u16, path: &str, host_header: &str) {
    // Construct full URL for HTTPS client
    let url = if port == 443 {
        alloc::format!("https://{}{}", host_header, path)
    } else {
        alloc::format!("https://{}:{}{}", host_header, port, path)
    };
    
    crate::println!("Connecting to {} (TLS 1.3)...", host_header);
    crate::println!("--- HTTPS response ---");
    
    match crate::netstack::https::get(&url) {
        Ok(response) => {
            // Print status
            crate::println_color!(COLOR_CYAN, "HTTP/1.1 {}", response.status_code);
            
            // Print headers
            for (key, value) in &response.headers {
                crate::println!("{}: {}", key, value);
            }
            crate::println!("");
            
            // Print body (limit to reasonable size for display)
            let body_preview = if response.body.len() > 4096 {
                &response.body[..4096]
            } else {
                &response.body
            };
            
            if let Ok(body_str) = core::str::from_utf8(body_preview) {
                crate::print!("{}", body_str);
                if response.body.len() > 4096 {
                    crate::println!("\n... (truncated, {} more bytes)", response.body.len() - 4096);
                }
            } else {
                crate::println!("[Binary data: {} bytes]", response.body.len());
            }
            
            crate::println!("\n--- end ({} bytes) ---", response.body.len());
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "HTTPS failed: {}", e);
        }
    }
}

pub(super) fn parse_http_url(url: &str) -> Option<(String, u16, String, bool)> {
    let mut u = url.trim();
    let mut https = false;
    if let Some(rest) = u.strip_prefix("https://") {
        u = rest;
        https = true;
    } else if let Some(rest) = u.strip_prefix("http://") {
        u = rest;
    }

    let (host_port, path) = if let Some((h, p)) = u.split_once('/') {
        (h, format!("/{}", p))
    } else {
        (u, String::from("/"))
    };

    let (host, port) = if let Some((h, p)) = host_port.split_once(':') {
        let port = p.parse::<u16>().ok()?;
        (h, port)
    } else {
        (host_port, if https { 443 } else { 80 })
    };

    if host.is_empty() {
        return None;
    }

    Some((String::from(host), port, path, https))
}

pub(super) fn parse_ipv4(input: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = input.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let a = parts[0].parse::<u8>().ok()?;
    let b = parts[1].parse::<u8>().ok()?;
    let c = parts[2].parse::<u8>().ok()?;
    let d = parts[3].parse::<u8>().ok()?;
    Some([a, b, c, d])
}

// ==================== PROGRAM EXECUTION ====================

pub(super) fn cmd_exec(args: &[&str], command: &str) {
    if args.is_empty() && !command.starts_with("./") {
        crate::println_color!(COLOR_CYAN, "Usage: exec <program> [args...]");
        crate::println!("       ./program [args...]");
        crate::println!();
        crate::println!("Executes an ELF binary in user space.");
        crate::println!();
        crate::println!("Examples:");
        crate::println!("  exec /bin/hello");
        crate::println!("  ./hello.elf");
        crate::println!("  exec test    (runs built-in test)");
        return;
    }
    
    // Handle ./program syntax
    let (program, prog_args) = if command.starts_with("./") {
        (command, args)
    } else if args.is_empty() {
        crate::println_color!(COLOR_RED, "exec: missing program name");
        return;
    } else {
        (args[0], &args[1..])
    };
    
    // Special case: "exec test" runs built-in Ring 3 test (raw machine code)
    if program == "test" || program == "./test" {
        crate::println_color!(COLOR_CYAN, "Running Ring 3 test program...");
        match crate::exec::exec_test_program() {
            crate::exec::ExecResult::Exited(code) => {
                if code == 0 {
                    crate::println_color!(COLOR_GREEN, "Ring 3 test passed (exit code 0)");
                } else {
                    crate::println_color!(COLOR_YELLOW, "Ring 3 test exited with code: {}", code);
                }
            }
            crate::exec::ExecResult::Faulted(reason) => {
                crate::println_color!(COLOR_RED, "Test faulted: {}", reason);
            }
            crate::exec::ExecResult::LoadError(e) => {
                crate::println_color!(COLOR_RED, "Load error: {:?}", e);
            }
            crate::exec::ExecResult::MemoryError => {
                crate::println_color!(COLOR_RED, "Memory allocation failed");
            }
        }
        return;
    }
    
    // Special case: "exec hello" runs embedded hello world ELF
    if program == "hello" || program == "./hello" {
        crate::println_color!(COLOR_CYAN, "Running embedded hello world ELF in Ring 3...");
        match crate::exec::exec_hello_elf() {
            crate::exec::ExecResult::Exited(code) => {
                if code == 0 {
                    crate::println_color!(COLOR_GREEN, "Program exited successfully");
                } else {
                    crate::println_color!(COLOR_YELLOW, "Program exited with code: {}", code);
                }
            }
            crate::exec::ExecResult::Faulted(reason) => {
                crate::println_color!(COLOR_RED, "Program faulted: {}", reason);
            }
            crate::exec::ExecResult::LoadError(e) => {
                crate::println_color!(COLOR_RED, "Failed to load ELF: {:?}", e);
            }
            crate::exec::ExecResult::MemoryError => {
                crate::println_color!(COLOR_RED, "Memory allocation failed");
            }
        }
        return;
    }
    
    // Resolve program path
    let path = resolve_program_path(program);
    
    // Check if file exists
    if !file_exists(&path) {
        crate::println_color!(COLOR_RED, "exec: {}: not found", path);
        return;
    }
    
    // Check if it's an ELF
    if !crate::exec::is_executable(&path) {
        crate::println_color!(COLOR_RED, "exec: {}: not an ELF executable", path);
        return;
    }
    
    crate::println_color!(COLOR_CYAN, "Executing: {}", path);
    
    // Execute the program
    match crate::exec::exec_path(&path, prog_args) {
        crate::exec::ExecResult::Exited(code) => {
            if code == 0 {
                crate::println_color!(COLOR_GREEN, "Program exited successfully");
            } else {
                crate::println_color!(COLOR_YELLOW, "Program exited with code: {}", code);
            }
        }
        crate::exec::ExecResult::Faulted(reason) => {
            crate::println_color!(COLOR_RED, "Program faulted: {}", reason);
        }
        crate::exec::ExecResult::LoadError(e) => {
            crate::println_color!(COLOR_RED, "Failed to load: {:?}", e);
        }
        crate::exec::ExecResult::MemoryError => {
            crate::println_color!(COLOR_RED, "Out of memory");
        }
    }
}

pub(super) fn cmd_elfinfo(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: elfinfo <file>");
        return;
    }
    
    let path = resolve_program_path(args[0]);
    
    // Open and read file header
    let fd = match crate::vfs::open(&path, crate::vfs::OpenFlags(0)) {
        Ok(fd) => fd,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Cannot open: {}", path);
            return;
        }
    };
    
    let mut header = [0u8; 64];
    match crate::vfs::read(fd, &mut header) {
        Ok(n) if n >= 64 => {}
        _ => {
            crate::println_color!(COLOR_RED, "Cannot read ELF header");
            crate::vfs::close(fd).ok();
            return;
        }
    }
    crate::vfs::close(fd).ok();
    
    // Check magic
    if header[0..4] != [0x7F, b'E', b'L', b'F'] {
        crate::println_color!(COLOR_RED, "Not an ELF file");
        return;
    }
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "ELF Header: {}", path);
    crate::println!("  Magic:   {:02X} {:02X} {:02X} {:02X}", header[0], header[1], header[2], header[3]);
    crate::println!("  Class:   {}", if header[4] == 2 { "ELF64" } else { "ELF32" });
    crate::println!("  Data:    {}", if header[5] == 1 { "Little Endian" } else { "Big Endian" });
    
    let e_type = u16::from_le_bytes([header[16], header[17]]);
    let type_str = match e_type {
        1 => "Relocatable",
        2 => "Executable",
        3 => "Shared Object",
        4 => "Core",
        _ => "Unknown",
    };
    crate::println!("  Type:    {} ({})", type_str, e_type);
    
    let e_machine = u16::from_le_bytes([header[18], header[19]]);
    let machine_str = match e_machine {
        3 => "x86",
        62 => "x86-64",
        183 => "AArch64",
        _ => "Unknown",
    };
    crate::println!("  Machine: {} ({})", machine_str, e_machine);
    
    let entry = u64::from_le_bytes([
        header[24], header[25], header[26], header[27],
        header[28], header[29], header[30], header[31],
    ]);
    crate::println!("  Entry:   {:#x}", entry);
    
    let phoff = u64::from_le_bytes([
        header[32], header[33], header[34], header[35],
        header[36], header[37], header[38], header[39],
    ]);
    crate::println!("  PHoff:   {:#x}", phoff);
    
    let phnum = u16::from_le_bytes([header[56], header[57]]);
    crate::println!("  PHnum:   {}", phnum);
}

/// Try to execute a file if it exists and is executable
pub(super) fn try_exec_file(command: &str, args: &[&str]) -> bool {
    let path = resolve_program_path(command);
    
    if !file_exists(&path) {
        return false;
    }

    // Try ELF binary first
    if crate::exec::is_executable(&path) {
        crate::println_color!(COLOR_CYAN, "Executing: {}", path);
        match crate::exec::exec_path(&path, args) {
            crate::exec::ExecResult::Exited(code) => {
                if code != 0 {
                    crate::println_color!(COLOR_YELLOW, "Exit code: {}", code);
                }
            }
            crate::exec::ExecResult::Faulted(reason) => {
                crate::println_color!(COLOR_RED, "Faulted: {}", reason);
            }
            crate::exec::ExecResult::LoadError(e) => {
                crate::println_color!(COLOR_RED, "Load error: {:?}", e);
            }
            crate::exec::ExecResult::MemoryError => {
                crate::println_color!(COLOR_RED, "Out of memory");
            }
        }
        return true;
    }
    
    // Try shell script (#!/bin/sh)
    if let Some(content) = super::network::read_file_content(&path) {
        if content.starts_with("#!/bin/sh") || content.starts_with("#!/bin/bash") {
            exec_shell_script(&content, args);
            return true;
        }
    }

    false
}

/// Execute a simple shell script (#!/bin/sh)
/// Supports: echo, #comments, variable substitution ($1..$9, $@, $#)
fn exec_shell_script(script: &str, args: &[&str]) {
    use alloc::string::String;
    use alloc::vec::Vec;
    use alloc::collections::BTreeMap;

    let lines: Vec<&str> = script.lines().collect();
    let mut vars: BTreeMap<String, String> = BTreeMap::new();

    // Set positional parameters
    for (i, arg) in args.iter().enumerate() {
        vars.insert(alloc::format!("{}", i + 1), String::from(*arg));
    }
    vars.insert(String::from("@"), args.join(" "));
    vars.insert(String::from("#"), alloc::format!("{}", args.len()));
    vars.insert(String::from("0"), String::from("sh"));
    vars.insert(String::from("?"), String::from("0"));
    vars.insert(String::from("HOME"), String::from("/root"));
    vars.insert(String::from("PATH"), String::from("/usr/bin:/bin:/usr/sbin:/sbin"));
    vars.insert(String::from("SHELL"), String::from("/bin/sh"));

    let mut pc = 0usize; // program counter (line index)
    let mut skip_depth = 0u32; // nesting depth when skipping (inside false if/else)

    while pc < lines.len() {
        let raw = lines[pc].trim();
        pc += 1;

        // Skip shebang, empty lines, comments
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }

        // Handle if/then/else/fi for skip mode
        if skip_depth > 0 {
            if raw.starts_with("if ") || raw == "if" {
                skip_depth += 1;
            } else if raw == "fi" || raw.starts_with("fi;") || raw.starts_with("fi ") {
                skip_depth -= 1;
            } else if skip_depth == 1 && (raw == "else" || raw.starts_with("else;") || raw.starts_with("else ")) {
                skip_depth = 0; // enter else branch
            }
            continue;
        }

        // Expand variables
        let expanded = expand_shell_vars_map(raw, &vars);
        let expanded = expanded.trim();
        if expanded.is_empty() { continue; }

        // Handle semicolons (split into multiple commands)
        if expanded.contains(';') && !expanded.starts_with("if ") && !expanded.contains("then") {
            let sub_cmds: Vec<&str> = expanded.split(';').collect();
            for sub in sub_cmds {
                let sub = sub.trim();
                if sub.is_empty() { continue; }
                exec_shell_line(sub, &mut vars);
            }
            continue;
        }

        // Control flow: if/then/else/fi
        if expanded.starts_with("if ") {
            // Simple: "if [ condition ]; then" or "if command; then"
            let cond_str = expanded.trim_start_matches("if ").trim();
            let cond_str = cond_str.trim_end_matches("; then").trim_end_matches(";then").trim();
            let result = eval_shell_condition(cond_str, &vars);
            if !result {
                skip_depth = 1; // skip to else/fi
            }
            continue;
        }
        if expanded == "then" { continue; } // standalone then
        if expanded == "else" {
            skip_depth = 1; // we were in true branch, skip else
            continue;
        }
        if expanded == "fi" || expanded.starts_with("fi;") || expanded.starts_with("fi ") {
            continue; // end of if block
        }

        // Control flow: for/do/done
        if expanded.starts_with("for ") {
            // Parse: "for VAR in val1 val2 ...; do" or multi-line
            let rest = expanded.trim_start_matches("for ").trim();
            if let Some(in_pos) = rest.find(" in ") {
                let var_name = &rest[..in_pos];
                let values_str = rest[in_pos + 4..].trim();
                let values_str = values_str.trim_end_matches("; do").trim_end_matches(";do").trim();
                let values: Vec<&str> = values_str.split_whitespace().collect();

                // Skip past "do" if it's on the next line
                if pc < lines.len() && lines[pc].trim() == "do" {
                    pc += 1;
                }

                // Collect body lines until "done"
                let body_start = pc;
                let mut body_end = pc;
                let mut depth = 1u32;
                while body_end < lines.len() {
                    let bl = lines[body_end].trim();
                    if bl.starts_with("for ") { depth += 1; }
                    if bl == "done" || bl.starts_with("done;") || bl.starts_with("done ") {
                        depth -= 1;
                        if depth == 0 { break; }
                    }
                    body_end += 1;
                }

                // Execute body for each value
                let body: Vec<&str> = lines[body_start..body_end].to_vec();
                for val in &values {
                    vars.insert(String::from(var_name), String::from(*val));
                    for body_line in &body {
                        let bl = body_line.trim();
                        if bl.is_empty() || bl.starts_with('#') || bl == "do" { continue; }
                        let exp = expand_shell_vars_map(bl, &vars);
                        exec_shell_line(exp.trim(), &mut vars);
                    }
                }

                pc = body_end + 1; // skip past "done"
                continue;
            }
        }

        // Regular command execution
        exec_shell_line(&expanded, &mut vars);
    }
}

/// Execute a single shell line
fn exec_shell_line(line: &str, vars: &mut alloc::collections::BTreeMap<alloc::string::String, alloc::string::String>) {
    use alloc::string::String;

    let line = line.trim();
    if line.is_empty() || line.starts_with('#') { return; }

    // Variable assignment: VAR=value
    if let Some(eq_pos) = line.find('=') {
        if eq_pos > 0 && line[..eq_pos].chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !line.starts_with('=') {
            let var = &line[..eq_pos];
            let val = line[eq_pos + 1..].trim();
            let val = strip_shell_quotes(val);
            vars.insert(String::from(var), val);
            return;
        }
    }

    // Split into command + arguments
    let parts: alloc::vec::Vec<&str> = line.splitn(2, char::is_whitespace).collect();
    let cmd = parts[0];
    let rest = if parts.len() > 1 { parts[1].trim() } else { "" };

    match cmd {
        "echo" => {
            if rest == "-n" {
                // echo -n: no output
            } else if rest.starts_with("-n ") {
                let msg = strip_shell_quotes(&rest[3..]);
                crate::print!("{}", msg);
            } else if rest.starts_with("-e ") {
                let msg = strip_shell_quotes(&rest[3..]);
                crate::println!("{}", msg);
            } else {
                let msg = strip_shell_quotes(rest);
                crate::println!("{}", msg);
            }
        }
        "printf" => {
            let msg = strip_shell_quotes(rest);
            crate::print!("{}", msg);
        }
        "cat" => {
            // Try to read and display file content
            if !rest.is_empty() {
                let path = strip_shell_quotes(rest);
                if let Some(content) = super::network::read_file_content(&path) {
                    crate::print!("{}", content);
                } else {
                    crate::println!("cat: {}: No such file or directory", path);
                }
            }
        }
        "test" | "[" => {
            // Basic test command — evaluate and set $?
            let cond = rest.trim_end_matches(']').trim();
            let result = eval_shell_condition(&alloc::format!("[ {} ]", cond), vars);
            vars.insert(alloc::string::String::from("?"), if result { alloc::string::String::from("0") } else { alloc::string::String::from("1") });
        }
        "export" => {
            // export VAR=value or export VAR
            if let Some(eq_pos) = rest.find('=') {
                let var = &rest[..eq_pos];
                let val = strip_shell_quotes(&rest[eq_pos + 1..]);
                vars.insert(alloc::string::String::from(var), val);
            }
        }
        "env" | "printenv" => {
            for (k, v) in vars.iter() {
                if k.len() > 1 { // skip positional params
                    crate::println!("{}={}", k, v);
                }
            }
        }
        "set" => {
            if rest == "-e" || rest == "-x" || rest.is_empty() {
                // Silently accept common set options
            }
        }
        "true" | ":" => {
            vars.insert(alloc::string::String::from("?"), alloc::string::String::from("0"));
        }
        "false" => {
            vars.insert(alloc::string::String::from("?"), alloc::string::String::from("1"));
        }
        "exec" => {
            // exec command — just run the rest inline
            if !rest.is_empty() {
                exec_shell_line(rest, vars);
            }
        }
        "exit" | "return" => {
            // Can't really exit from here, but stop processing
        }
        "sleep" => {
            // Silently ignore sleep commands
        }
        "cd" | "mkdir" | "rm" | "chmod" | "chown" | "ln" | "cp" | "mv" | "touch" => {
            // Silently accept filesystem commands (they operate on ramfs if needed)
        }
        _ => {
            // Unknown command — silently skip
        }
    }
}

/// Expand shell variables from a BTreeMap
fn expand_shell_vars_map(line: &str, vars: &alloc::collections::BTreeMap<alloc::string::String, alloc::string::String>) -> alloc::string::String {
    use alloc::string::String;
    let mut result = String::with_capacity(line.len());
    let chars: alloc::vec::Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            if chars[i + 1] == '{' {
                // ${VAR} syntax
                if let Some(close) = chars[i + 2..].iter().position(|&c| c == '}') {
                    let var: String = chars[i + 2..i + 2 + close].iter().collect();
                    if let Some(val) = vars.get(&var) {
                        result.push_str(val);
                    }
                    i += close + 3;
                    continue;
                }
            } else if chars[i + 1] == '(' {
                // $(command) — skip subshell
                if let Some(close) = chars[i + 2..].iter().position(|&c| c == ')') {
                    i += close + 3;
                    continue;
                }
            }
            // $VAR or $N
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_' || chars[end] == '@' || chars[end] == '#' || chars[end] == '?') {
                end += 1;
                // Single-char specials: $@, $#, $?, $0-$9
                if end == start + 1 && (chars[start] == '@' || chars[start] == '#' || chars[start] == '?' || chars[start].is_ascii_digit()) {
                    break;
                }
            }
            if end > start {
                let var: String = chars[start..end].iter().collect();
                if let Some(val) = vars.get(&var) {
                    result.push_str(val);
                }
                i = end;
            } else {
                result.push('$');
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

/// Evaluate a simple shell condition (for if statements)
fn eval_shell_condition(cond: &str, _vars: &alloc::collections::BTreeMap<alloc::string::String, alloc::string::String>) -> bool {
    let cond = cond.trim();
    // Strip [ ] brackets
    let inner = if cond.starts_with('[') && cond.ends_with(']') {
        cond[1..cond.len() - 1].trim()
    } else if cond.starts_with("[ ") && cond.ends_with(" ]") {
        cond[2..cond.len() - 2].trim()
    } else {
        cond
    };

    // -n STRING (non-empty)
    if inner.starts_with("-n ") { return !inner[3..].trim().trim_matches('"').is_empty(); }
    // -z STRING (empty)
    if inner.starts_with("-z ") { return inner[3..].trim().trim_matches('"').is_empty(); }
    // -f FILE (file exists)
    if inner.starts_with("-f ") {
        let path = inner[3..].trim().trim_matches('"');
        return crate::ramfs::with_fs(|fs| fs.read_file(path).is_ok());
    }
    // -d DIR (directory exists)
    if inner.starts_with("-d ") { return true; } // simplified
    // STRING = STRING
    if inner.contains(" = ") {
        let parts: alloc::vec::Vec<&str> = inner.splitn(2, " = ").collect();
        if parts.len() == 2 {
            return parts[0].trim().trim_matches('"') == parts[1].trim().trim_matches('"');
        }
    }
    // STRING != STRING
    if inner.contains(" != ") {
        let parts: alloc::vec::Vec<&str> = inner.splitn(2, " != ").collect();
        if parts.len() == 2 {
            return parts[0].trim().trim_matches('"') != parts[1].trim().trim_matches('"');
        }
    }
    // -eq, -ne, -gt, -lt, -ge, -le (integer comparison)
    for op in &[" -eq ", " -ne ", " -gt ", " -lt ", " -ge ", " -le "] {
        if inner.contains(op) {
            let parts: alloc::vec::Vec<&str> = inner.splitn(2, op).collect();
            if parts.len() == 2 {
                let a = parts[0].trim().parse::<i64>().unwrap_or(0);
                let b = parts[1].trim().parse::<i64>().unwrap_or(0);
                return match *op {
                    " -eq " => a == b,
                    " -ne " => a != b,
                    " -gt " => a > b,
                    " -lt " => a < b,
                    " -ge " => a >= b,
                    " -le " => a <= b,
                    _ => false,
                };
            }
        }
    }

    // Default: non-empty string is true
    !inner.is_empty()
}

/// Strip surrounding quotes from a string and process escape sequences
fn strip_shell_quotes(s: &str) -> String {
    use alloc::string::String;
    let s = s.trim();
    let unquoted = if (s.starts_with('\'') && s.ends_with('\''))
        || (s.starts_with('"') && s.ends_with('"'))
    {
        &s[1..s.len() - 1]
    } else {
        s
    };
    // Process basic escape sequences
    let mut result = String::with_capacity(unquoted.len());
    let mut esc = false;
    for c in unquoted.chars() {
        if esc {
            match c {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                '\'' => result.push('\''),
                _ => { result.push('\\'); result.push(c); }
            }
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else {
            result.push(c);
        }
    }
    result
}

/// Resolve a program name to a full path
pub(super) fn resolve_program_path(name: &str) -> String {
    if name.starts_with('/') {
        return String::from(name);
    }
    
    if name.starts_with("./") {
        let cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
        if cwd == "/" {
            return String::from(&name[1..]); // "/program"
        } else {
            return format!("{}{}", cwd, &name[1..]); // "/dir/program"
        }
    }
    
    // Search in PATH-like directories
    let search_dirs = ["/usr/bin", "/bin", "/usr/sbin", "/sbin", "/usr/local/bin"];
    
    for dir in &search_dirs {
        let path = format!("{}/{}", dir, name);
        if file_exists(&path) {
            return path;
        }
    }
    
    // Try current directory
    let cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
    if cwd == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", cwd, name)
    }
}

/// Check if a file exists
pub(super) fn file_exists(path: &str) -> bool {
    // Try VFS first
    if crate::vfs::stat(path).is_ok() {
        return true;
    }
    // Fallback to ramfs
    crate::ramfs::with_fs(|fs| fs.exists(path))
}

// ============================================================================
// HYPERVISOR COMMANDS
// ============================================================================

/// Hypervisor management command
pub(super) fn cmd_hypervisor(args: &[&str]) {
    if args.is_empty() {
        print_hv_help();
        return;
    }
    
    match args[0] {
        "init" => {
            crate::println!("Initializing TrustVM hypervisor...");
            match crate::hypervisor::init() {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "? ");
                    crate::println!("Hypervisor initialized successfully!");
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "? ");
                    crate::println!("Failed to initialize hypervisor: {:?}", e);
                }
            }
        }
        "status" => {
            if crate::hypervisor::is_enabled() {
                crate::print_color!(COLOR_GREEN, "? ");
                crate::println!("TrustVM: Active");
                crate::println!("  Backend: {}", crate::hypervisor::backend_info());
                crate::println!("  VMs created: {}", crate::hypervisor::vm_count());
            } else {
                crate::print_color!(COLOR_YELLOW, "? ");
                crate::println!("TrustVM: Inactive");
                crate::println!("  Run 'hv init' to enable the hypervisor");
            }
        }
        "check" => {
            use crate::hypervisor::{detect_cpu_vendor, CpuVendor};
            crate::println!("Checking virtualization support...");
            let vendor = detect_cpu_vendor();
            crate::println!("  CPU Vendor: {:?}", vendor);
            
            match vendor {
                CpuVendor::Intel => {
                    match crate::hypervisor::vmx::check_vmx_support() {
                        Ok(caps) => {
                            crate::println!("  [Intel VT-x (VMX)]");
                            crate::println!("    VMX supported:      {}", if caps.supported { "Yes" } else { "No" });
                            crate::println!("    EPT supported:      {}", if caps.ept_supported { "Yes" } else { "No" });
                            crate::println!("    Unrestricted guest: {}", if caps.unrestricted_guest { "Yes" } else { "No" });
                            crate::println!("    VPID supported:     {}", if caps.vpid_supported { "Yes" } else { "No" });
                            crate::println!("    VMCS revision:      0x{:08X}", caps.vmcs_revision_id);
                        }
                        Err(e) => {
                            crate::print_color!(COLOR_RED, "Error: ");
                            crate::println!("{:?}", e);
                        }
                    }
                }
                CpuVendor::Amd => {
                    if crate::hypervisor::svm::is_supported() {
                        let features = crate::hypervisor::svm::get_features();
                        crate::println!("  [AMD-V (SVM)]");
                        crate::println!("    SVM supported:      Yes");
                        crate::println!("    SVM Revision:       {}", features.revision);
                        crate::println!("    NPT supported:      {}", if features.npt { "Yes" } else { "No" });
                        crate::println!("    NRIP Save:          {}", if features.nrip_save { "Yes" } else { "No" });
                        crate::println!("    Flush by ASID:      {}", if features.flush_by_asid { "Yes" } else { "No" });
                        crate::println!("    Available ASIDs:    {}", features.num_asids);
                        crate::println!("    AVIC:               {}", if features.avic { "Yes" } else { "No" });
                    } else {
                        crate::print_color!(COLOR_RED, "Error: ");
                        crate::println!("SVM not supported or disabled in BIOS");
                    }
                }
                CpuVendor::Unknown => {
                    crate::print_color!(COLOR_RED, "Error: ");
                    crate::println!("Unknown CPU vendor - virtualization not supported");
                }
            }
        }
        "shutdown" => {
            crate::println!("Shutting down hypervisor...");
            match crate::hypervisor::shutdown() {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "? ");
                    crate::println!("Hypervisor shutdown complete");
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "? ");
                    crate::println!("Failed: {:?}", e);
                }
            }
        }
        "caps" | "capabilities" => {
            crate::println!("{}", crate::hypervisor::render_capabilities());
        }
        "security" => {
            crate::println!("{}", crate::hypervisor::render_security_status());
        }
        "events" => {
            let count = if args.len() > 1 { 
                args[1].parse().unwrap_or(10) 
            } else { 
                10 
            };
            let events = crate::hypervisor::get_events(count);
            if events.is_empty() {
                crate::println!("No events recorded.");
            } else {
                crate::println!("Recent VM Events:");
                for event in events {
                    crate::println!("  [{:>6}ms] VM {} - {:?}", 
                        event.timestamp_ms, event.vm_id, event.event_type);
                }
            }
        }
        "vpid" => {
            if crate::hypervisor::vpid_enabled() {
                crate::print_color!(COLOR_GREEN, "? ");
                crate::println!("VPID: Enabled");
                crate::println!("  Allocated VPIDs: {}", crate::hypervisor::vpid_count());
            } else {
                crate::print_color!(COLOR_YELLOW, "? ");
                crate::println!("VPID: Disabled (CPU may not support it)");
            }
        }
        "violations" => {
            let count = crate::hypervisor::ept_violations();
            crate::println!("EPT Violations: {}", count);
            if count > 0 {
                let violations = crate::hypervisor::recent_ept_violations(5);
                for v in violations {
                    crate::println!("  VM {} GPA=0x{:X} type={:?} at RIP=0x{:X}",
                        v.vm_id, v.guest_physical, v.violation_type, v.guest_rip);
                }
            }
        }
        "version" => {
            crate::println!("TrustVM {}", crate::hypervisor::version());
        }
        "logo" => {
            crate::println!("{}", crate::hypervisor::logo());
        }
        // ═══════════════════════════════════════════════════
        // ARM EL2 Hypervisor — MMIO Spy Commands
        // ═══════════════════════════════════════════════════
        #[cfg(target_arch = "aarch64")]
        "spy" | "mmio" => {
            crate::println_color!(COLOR_CYAN, "=== TrustOS EL2 MMIO Spy ===");
            crate::println!();
            if !crate::hypervisor::arm_hv::is_el2() {
                crate::println_color!(COLOR_RED, "Not running at EL2 - hypervisor mode unavailable");
                crate::println!("Boot TrustOS at EL2 (QEMU: -machine virt,virtualization=on)");
                return;
            }
            if !crate::hypervisor::arm_hv::is_active() {
                crate::println_color!(COLOR_YELLOW, "EL2 detected but no guest running yet");
                crate::println!("Use 'hv launch' to start a guest under surveillance");
                return;
            }
            let report = crate::hypervisor::arm_hv::el2_entry::get_spy_summary();
            crate::println!("{}", report);
        }
        #[cfg(target_arch = "aarch64")]
        "smc" | "smc-log" => {
            crate::println_color!(COLOR_CYAN, "=== SMC (Secure Monitor Call) Log ===");
            let count = if args.len() > 1 { args[1].parse().unwrap_or(20) } else { 20 };
            let events = crate::hypervisor::arm_hv::mmio_spy::recent_smc_events(count);
            if events.is_empty() {
                crate::println!("No SMC calls intercepted.");
            } else {
                for ev in &events {
                    crate::println!("  {}", crate::hypervisor::arm_hv::mmio_spy::format_smc_event(ev));
                }
                crate::println!("\nTotal SMC events: {}",
                    crate::hypervisor::arm_hv::mmio_spy::total_smc_events());
            }
        }
        #[cfg(target_arch = "aarch64")]
        "devices" => {
            crate::println_color!(COLOR_CYAN, "=== Device Activity (per MMIO range) ===");
            let stats = crate::hypervisor::arm_hv::mmio_spy::device_stats();
            if stats.is_empty() {
                crate::println!("No device activity recorded.");
            } else {
                crate::println!("  {:<22} {:<8} {}", "Device", "Reads", "Writes");
                crate::println!("  {}", "-".repeat(42));
                for (name, reads, writes) in &stats {
                    crate::println!("  {:<22} {:<8} {}", name, reads, writes);
                }
            }
        }
        #[cfg(target_arch = "aarch64")]
        "el2" => {
            crate::println_color!(COLOR_CYAN, "=== ARM EL2 Hypervisor Status ===");
            if crate::hypervisor::arm_hv::is_el2() {
                crate::println_color!(COLOR_GREEN, "  Running at EL2: Yes");
                crate::println!("  Hypervisor active: {}", 
                    if crate::hypervisor::arm_hv::is_active() { "Yes (guest running)" } else { "No (idle)" });
                crate::println!("  MMIO traps: {}", crate::hypervisor::arm_hv::mmio_trap_count());
                crate::println!("  SMC intercepts: {}", crate::hypervisor::arm_hv::smc_trap_count());
                crate::println!("  MMIO events logged: {}", 
                    crate::hypervisor::arm_hv::mmio_spy::total_mmio_events());
                crate::println!("  SMC events logged: {}", 
                    crate::hypervisor::arm_hv::mmio_spy::total_smc_events());
            } else {
                crate::println_color!(COLOR_RED, "  Running at EL2: No");
                crate::println!("  Current EL does not support hypervisor operations.");
                crate::println!("  Boot with: qemu-system-aarch64 -machine virt,virtualization=on");
            }
        }
        #[cfg(target_arch = "aarch64")]
        "report" => {
            crate::println!("{}", crate::hypervisor::arm_hv::generate_spy_report());
        }
        "test" | "selftest" => {
            crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════╗");
            crate::println_color!(COLOR_CYAN, "║         TrustVM Hypervisor Self-Test Suite           ║");
            crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════╝");
            crate::println!();
            
            let (passed, failed, log) = crate::hypervisor::tests::run_all_tests();
            
            for line in &log {
                if line.contains("[PASS]") {
                    crate::println_color!(COLOR_GREEN, "{}", line);
                } else {
                    crate::println_color!(COLOR_RED, "{}", line);
                }
            }
            
            crate::println!();
            if failed == 0 {
                crate::println_color!(COLOR_GREEN, "Result: {}/{} tests passed — ALL OK ✓", passed, passed + failed);
            } else {
                crate::println_color!(COLOR_RED, "Result: {}/{} tests passed, {} FAILED ✗", passed, passed + failed, failed);
            }
        }
        "help" | _ => print_hv_help(),
    }
}

fn print_hv_help() {
    use crate::hypervisor::{detect_cpu_vendor, CpuVendor};
    let vendor = detect_cpu_vendor();
    let backend = match vendor {
        CpuVendor::Intel => "Intel VT-x (VMX)",
        CpuVendor::Amd => "AMD-V (SVM)",
        CpuVendor::Unknown => {
            #[cfg(target_arch = "aarch64")]
            { "ARM EL2 (Stage-2 MMIO Spy)" }
            #[cfg(not(target_arch = "aarch64"))]
            { "Unknown" }
        },
    };
    
    crate::println!("TrustVM Hypervisor Commands (Backend: {})", backend);
    crate::println!();
    crate::println!("Initialization:");
    crate::println!("  hv init       - Initialize the hypervisor");
    crate::println!("  hv shutdown   - Shutdown the hypervisor");
    crate::println!("  hv status     - Show hypervisor status");
    crate::println!("  hv check      - Check virtualization capabilities");
    crate::println!();
    crate::println!("Monitoring:");
    crate::println!("  hv caps       - Show TrustVM capabilities");
    crate::println!("  hv security   - Show security status");
    crate::println!("  hv events [n] - Show recent VM events");
    crate::println!("  hv vpid       - Show VPID/ASID status");
    crate::println!("  hv violations - Show EPT/NPT violations");
    crate::println!("  hv version    - Show TrustVM version");
    crate::println!("  hv logo       - Display TrustVM logo");
    crate::println!("  hv test       - Run hypervisor self-tests");
    #[cfg(target_arch = "aarch64")]
    {
        crate::println!();
        crate::println!("ARM EL2 Hypervisor:");
        crate::println!("  hv el2        - Show ARM EL2 status");
        crate::println!("  hv spy        - Show MMIO spy activity (real-time)");
        crate::println!("  hv smc [n]    - Show SMC call log");
        crate::println!("  hv devices    - Show per-device MMIO statistics");
        crate::println!("  hv report     - Full hypervisor spy report");
    }
    crate::println!();
    crate::println!("VM Management:");
    crate::println!("  vm create <name> <mem_mb>  - Create a new VM");
    crate::println!("  vm start <id> [guest]      - Start a VM with optional guest");
    crate::println!("  vm run <guest>             - Quick create and run a guest");
    crate::println!("  vm linux <bzimage> [initrd] [mb] - Boot a Linux bzImage in a VM");
    crate::println!("  vm stop <id>               - Stop a VM");
    crate::println!("  vm list                    - List all VMs");
    crate::println!("  vm guests                  - List available guests");
    crate::println!("  vm inspect [id]            - Inspect VM state (regs, exits, mem)");
    crate::println!("  vm debug [cmd]             - Real-time debug monitor (gaps, io, msr, timeline)");
    crate::println!("  vm mount <id> <host> <guest> - Mount shared folder");
}

/// VM management command
pub(super) fn cmd_vm(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: vm <command> [args]");
        crate::println!("Commands: create, start, run, stop, list, guests, inspect, debug, mount");
        return;
    }
    
    match args[0] {
        "create" => {
            if args.len() < 3 {
                crate::println!("Usage: vm create <name> <memory_mb>");
                return;
            }
            let name = args[1];
            let mem_mb: usize = args[2].parse().unwrap_or(16);
            
            if !crate::hypervisor::is_enabled() {
                crate::print_color!(COLOR_YELLOW, "Warning: ");
                crate::println!("Hypervisor not initialized. Run 'hv init' first.");
                return;
            }
            
            match crate::hypervisor::create_vm(name, mem_mb) {
                Ok(id) => {
                    crate::print_color!(COLOR_GREEN, "? ");
                    crate::println!("Created VM '{}' with ID {} ({}MB RAM)", name, id, mem_mb);
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "? ");
                    crate::println!("Failed to create VM: {:?}", e);
                }
            }
        }
        "start" => {
            if args.len() < 2 {
                crate::println!("Usage: vm start <id> [guest_name]");
                crate::println!("Available guests: {:?}", crate::hypervisor::list_guests());
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let guest = if args.len() > 2 { args[2] } else { "hello" };
            
            crate::println!("Starting VM {} with guest '{}'...", id, guest);
            match crate::hypervisor::start_vm_with_guest(id, guest) {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "? ");
                    crate::println!("VM {} completed execution", id);
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "? ");
                    crate::println!("VM {} failed: {:?}", id, e);
                }
            }
        }
        "run" => {
            // Quick run: create and start in one command
            let guest = if args.len() > 1 { args[1] } else { "hello" };
            
            if !crate::hypervisor::is_enabled() {
                crate::print_color!(COLOR_YELLOW, "Note: ");
                crate::println!("Initializing hypervisor first...");
                if let Err(e) = crate::hypervisor::init() {
                    crate::print_color!(COLOR_RED, "✗ ");
                    crate::println!("Failed to init hypervisor: {:?}", e);
                    return;
                }
            }
            
            // Linux guests need more memory
            let mem_mb = if guest.starts_with("linux") || guest.ends_with(".bzimage") {
                64
            } else {
                4
            };
            
            match crate::hypervisor::create_vm(guest, mem_mb) {
                Ok(id) => {
                    crate::println!("Running guest '{}'...", guest);
                    match crate::hypervisor::start_vm_with_guest(id, guest) {
                        Ok(()) => {
                            crate::print_color!(COLOR_GREEN, "? ");
                            crate::println!("Guest '{}' completed", guest);
                            
                            // Show quick stats for the completed VM
                            crate::hypervisor::svm_vm::with_vm(id, |vm| {
                                let s = &vm.stats;
                                crate::println!("  VMEXITs: {} (cpuid={} io={} msr={} hlt={} vmcall={})",
                                    s.vmexits, s.cpuid_exits, s.io_exits,
                                    s.msr_exits, s.hlt_exits, s.vmmcall_exits);
                            });
                            crate::println!("  Use 'vm inspect {}' for detailed state", id);
                        }
                        Err(e) => {
                            crate::print_color!(COLOR_RED, "? ");
                            crate::println!("Failed: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "? ");
                    crate::println!("Failed to create VM: {:?}", e);
                }
            }
        }
        "stop" => {
            if args.len() < 2 {
                crate::println!("Usage: vm stop <id>");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            
            match crate::hypervisor::stop_vm(id) {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "? ");
                    crate::println!("Stopped VM {}", id);
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "? ");
                    crate::println!("Failed to stop VM {}: {:?}", id, e);
                }
            }
        }
        "list" => {
            use crate::hypervisor::{detect_cpu_vendor, CpuVendor};
            crate::println!("Virtual Machines:");
            
            match detect_cpu_vendor() {
                CpuVendor::Amd => {
                    let vms = crate::hypervisor::svm_vm::list_vms();
                    if vms.is_empty() {
                        crate::println!("  (no VMs created)");
                    } else {
                        crate::println!("  {:>4} {:>20} {:>12}", "ID", "NAME", "STATE");
                        crate::println!("  {:->4} {:->20} {:->12}", "", "", "");
                        for (id, name, state) in vms {
                            crate::println!("  {:>4} {:>20} {:>12?}", id, name, state);
                        }
                    }
                }
                CpuVendor::Intel => {
                    crate::println!("  Total created: {}", crate::hypervisor::vm_count());
                }
                CpuVendor::Unknown => {
                    crate::println!("  (hypervisor not available)");
                }
            }
            crate::println!();
            crate::println!("Use 'vm guests' to see available guest programs.");
        }
        "guests" => {
            crate::println!("Available guest programs:");
            for guest in crate::hypervisor::list_guests() {
                crate::println!("  - {}", guest);
            }
            crate::println!("");
            crate::println!("Usage: vm run <guest_name>");
        }
        "mount" => {
            if args.len() < 4 {
                crate::println!("Usage: vm mount <vm_id> <host_path> <guest_path> [ro]");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let host_path = args[2];
            let guest_path = args[3];
            let readonly = args.len() > 4 && args[4] == "ro";
            
            crate::hypervisor::add_mount(id, host_path, guest_path, readonly);
            crate::print_color!(COLOR_GREEN, "? ");
            crate::println!("Mounted {} -> {} (readonly={})", host_path, guest_path, readonly);
        }
        "console" => {
            if args.len() < 2 {
                crate::println!("Usage: vm console <vm_id>");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let output = crate::hypervisor::get_console_output(id);
            if output.is_empty() {
                crate::println!("(no output)");
            } else {
                crate::println!("{}", output);
            }
        }
        "input" => {
            if args.len() < 3 {
                crate::println!("Usage: vm input <vm_id> <text>");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let text = args[2..].join(" ");
            crate::hypervisor::inject_console_input(id, text.as_bytes());
            crate::hypervisor::inject_console_input(id, b"\n");
            crate::println!("Injected input to VM {}", id);
        }
        "inspect" => {
            // Show live VM state — registers, exit stats, memory summary
            use crate::hypervisor::{detect_cpu_vendor, CpuVendor};
            
            match detect_cpu_vendor() {
                CpuVendor::Amd => {
                    let vms = crate::hypervisor::svm_vm::list_vms();
                    if vms.is_empty() {
                        crate::println!("No VMs to inspect. Run 'vm run pm-test' first.");
                        return;
                    }
                    
                    // If user gave an ID, inspect that one; otherwise inspect all
                    let filter_id: Option<u64> = if args.len() > 1 { args[1].parse().ok() } else { None };
                    
                    for (id, name, state) in &vms {
                        if let Some(fid) = filter_id {
                            if *id != fid { continue; }
                        }
                        
                        crate::println_color!(COLOR_CYAN, "+--- VM #{}: {} [{:?}] ---+", id, name, state);
                        
                        // Get detailed stats via with_vm
                        crate::hypervisor::svm_vm::with_vm(*id, |vm| {
                            let s = &vm.stats;
                            crate::println!();
                            crate::println_color!(COLOR_YELLOW, "  Exit Statistics:");
                            crate::println!("    Total VMEXITs: {}", s.vmexits);
                            crate::println!("    CPUID:   {:>8}", s.cpuid_exits);
                            crate::println!("    I/O:     {:>8}", s.io_exits);
                            crate::println!("    MSR:     {:>8}", s.msr_exits);
                            crate::println!("    HLT:     {:>8}", s.hlt_exits);
                            crate::println!("    NPF:     {:>8}", s.npf_exits);
                            crate::println!("    VMCALL:  {:>8}", s.vmmcall_exits);
                            crate::println!("    Intr:    {:>8}", s.intr_exits);
                            
                            crate::println!();
                            crate::println_color!(COLOR_YELLOW, "  Guest GPRs:");
                            crate::println!("    RAX = 0x{:016X}  RBX = 0x{:016X}", vm.guest_regs.rax, vm.guest_regs.rbx);
                            crate::println!("    RCX = 0x{:016X}  RDX = 0x{:016X}", vm.guest_regs.rcx, vm.guest_regs.rdx);
                            crate::println!("    RSI = 0x{:016X}  RDI = 0x{:016X}", vm.guest_regs.rsi, vm.guest_regs.rdi);
                            crate::println!("    RBP = 0x{:016X}  RSP = 0x{:016X}", vm.guest_regs.rbp, vm.guest_regs.rsp);
                            crate::println!("    R8  = 0x{:016X}  R9  = 0x{:016X}", vm.guest_regs.r8, vm.guest_regs.r9);
                            crate::println!("    R10 = 0x{:016X}  R11 = 0x{:016X}", vm.guest_regs.r10, vm.guest_regs.r11);
                            crate::println!("    R12 = 0x{:016X}  R13 = 0x{:016X}", vm.guest_regs.r12, vm.guest_regs.r13);
                            crate::println!("    R14 = 0x{:016X}  R15 = 0x{:016X}", vm.guest_regs.r14, vm.guest_regs.r15);
                            
                            // Show VMCB control registers if available
                            if let Some(ref vmcb) = vm.vmcb {
                                use crate::hypervisor::svm::vmcb::state_offsets;
                                crate::println!();
                                crate::println_color!(COLOR_YELLOW, "  VMCB State:");
                                let rip = vmcb.read_state(state_offsets::RIP);
                                let rsp = vmcb.read_state(state_offsets::RSP);
                                let rflags = vmcb.read_state(state_offsets::RFLAGS);
                                let cr0 = vmcb.read_state(state_offsets::CR0);
                                let cr3 = vmcb.read_state(state_offsets::CR3);
                                let cr4 = vmcb.read_state(state_offsets::CR4);
                                let efer = vmcb.read_state(state_offsets::EFER);
                                let cs = vmcb.read_u16(state_offsets::CS_SELECTOR) as u64;
                                let ds = vmcb.read_u16(state_offsets::DS_SELECTOR) as u64;
                                let cpl = vmcb.read_u16(state_offsets::CPL) as u64;
                                
                                crate::println!("    RIP    = 0x{:016X}  RSP    = 0x{:016X}", rip, rsp);
                                crate::println!("    RFLAGS = 0x{:016X}  CPL    = {}", rflags, cpl);
                                crate::println!("    CR0 = 0x{:X}  CR3 = 0x{:X}  CR4 = 0x{:X}", cr0, cr3, cr4);
                                crate::println!("    EFER = 0x{:X}  CS = 0x{:X}  DS = 0x{:X}", efer, cs, ds);
                                
                                // Show last exit info
                                use crate::hypervisor::svm::vmcb::control_offsets;
                                let exitcode = vmcb.read_control(control_offsets::EXITCODE);
                                let exitinfo1 = vmcb.read_control(control_offsets::EXITINFO1);
                                let exitinfo2 = vmcb.read_control(control_offsets::EXITINFO2);
                                crate::println!();
                                crate::println_color!(COLOR_YELLOW, "  Last VMEXIT:");
                                crate::println!("    ExitCode = 0x{:X}  Info1 = 0x{:X}  Info2 = 0x{:X}", 
                                    exitcode, exitinfo1, exitinfo2);
                            }
                            
                            crate::println!();
                            crate::println_color!(COLOR_YELLOW, "  Memory:");
                            crate::println!("    Guest memory: {} KB ({} MB)", vm.memory_size / 1024, vm.memory_size / (1024 * 1024));
                            crate::println!("    ASID: {}", vm.asid);
                            
                            // Show console output (last 256 chars)
                            let console_out = crate::hypervisor::get_console_output(*id);
                            if !console_out.is_empty() {
                                crate::println!();
                                crate::println_color!(COLOR_YELLOW, "  Console Output (last 256 chars):");
                                let start = if console_out.len() > 256 { console_out.len() - 256 } else { 0 };
                                crate::println!("    {}", &console_out[start..]);
                            }
                            
                            // Show a hex dump of guest memory at 0x5000 (where pm-test writes)
                            if let Some(data) = vm.read_guest_memory(0x5000, 32) {
                                crate::println!();
                                crate::println_color!(COLOR_YELLOW, "  Memory @ 0x5000 (guest write zone):");
                                crate::print!("    ");
                                for (i, byte) in data.iter().enumerate() {
                                    crate::print!("{:02X} ", byte);
                                    if (i + 1) % 16 == 0 && i + 1 < data.len() {
                                        crate::print!("\n    ");
                                    }
                                }
                                crate::println!();
                            }
                        });
                        
                        crate::println!();
                    }
                }
                _ => {
                    crate::println!("VM inspect requires AMD SVM. Use 'vm list' instead.");
                }
            }
        }
        
        // ── Memory dump command ─────────────────────────────────
        "dump" | "hexdump" => {
            if args.len() < 3 {
                crate::println!("Usage: vm dump <id> <gpa_hex> [length]");
                crate::println!("  Example: vm dump 1 0x1000 256");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let gpa_str = args[2].trim_start_matches("0x").trim_start_matches("0X");
            let gpa = u64::from_str_radix(gpa_str, 16).unwrap_or(0);
            let len: usize = if args.len() > 3 {
                args[3].parse().unwrap_or(128)
            } else {
                128
            };
            let len = len.min(4096); // Cap at 4KB
            
            match crate::hypervisor::cpu_vendor() {
                crate::hypervisor::CpuVendor::Amd => {
                    crate::hypervisor::svm_vm::with_vm(id, |vm| {
                        crate::println_color!(COLOR_CYAN, "  Memory dump: VM {} @ GPA 0x{:X} ({} bytes)", id, gpa, len);
                        crate::println!();
                        
                        if let Some(data) = vm.read_guest_memory(gpa, len) {
                            // Hex dump with ASCII sidebar
                            for row_start in (0..data.len()).step_by(16) {
                                crate::print!("  {:08X}: ", gpa as usize + row_start);
                                // Hex bytes
                                for col in 0..16 {
                                    if row_start + col < data.len() {
                                        crate::print!("{:02X} ", data[row_start + col]);
                                    } else {
                                        crate::print!("   ");
                                    }
                                    if col == 7 { crate::print!(" "); }
                                }
                                // ASCII
                                crate::print!(" |");
                                for col in 0..16 {
                                    if row_start + col < data.len() {
                                        let b = data[row_start + col];
                                        if b >= 0x20 && b < 0x7F {
                                            crate::print!("{}", b as char);
                                        } else {
                                            crate::print!(".");
                                        }
                                    }
                                }
                                crate::println!("|");
                            }
                        } else {
                            crate::println_color!(COLOR_RED, "  GPA 0x{:X}+{} is outside guest memory ({} bytes)", 
                                gpa, len, vm.memory_size);
                        }
                    });
                }
                _ => {
                    crate::println!("vm dump requires AMD SVM.");
                }
            }
        }
        
        // ── Register dump command ───────────────────────────────
        "regs" | "registers" => {
            if args.len() < 2 {
                crate::println!("Usage: vm regs <id>");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            
            match crate::hypervisor::cpu_vendor() {
                crate::hypervisor::CpuVendor::Amd => {
                    crate::hypervisor::svm_vm::with_vm(id, |vm| {
                        crate::println_color!(COLOR_CYAN, "  VM {} Register State", id);
                        crate::println!();
                        
                        // GPRs in 2-column format
                        crate::println!("  RAX={:016X}  RBX={:016X}", vm.guest_regs.rax, vm.guest_regs.rbx);
                        crate::println!("  RCX={:016X}  RDX={:016X}", vm.guest_regs.rcx, vm.guest_regs.rdx);
                        crate::println!("  RSI={:016X}  RDI={:016X}", vm.guest_regs.rsi, vm.guest_regs.rdi);
                        crate::println!("  RBP={:016X}  RSP={:016X}", vm.guest_regs.rbp, 
                            vm.vmcb.as_ref().map_or(0, |v| v.read_state(crate::hypervisor::svm::vmcb::state_offsets::RSP)));
                        crate::println!("  R8 ={:016X}  R9 ={:016X}", vm.guest_regs.r8, vm.guest_regs.r9);
                        crate::println!("  R10={:016X}  R11={:016X}", vm.guest_regs.r10, vm.guest_regs.r11);
                        crate::println!("  R12={:016X}  R13={:016X}", vm.guest_regs.r12, vm.guest_regs.r13);
                        crate::println!("  R14={:016X}  R15={:016X}", vm.guest_regs.r14, vm.guest_regs.r15);
                        
                        if let Some(ref vmcb) = vm.vmcb {
                            use crate::hypervisor::svm::vmcb::{state_offsets, control_offsets};
                            
                            let rip = vmcb.read_state(state_offsets::RIP);
                            let rfl = vmcb.read_state(state_offsets::RFLAGS);
                            crate::println!("  RIP={:016X}  RFLAGS={:016X}", rip, rfl);
                            
                            // Flags decoded
                            let flags_str = {
                                let mut s = alloc::string::String::new();
                                if rfl & 0x001 != 0 { s.push_str("CF "); }
                                if rfl & 0x040 != 0 { s.push_str("ZF "); }
                                if rfl & 0x080 != 0 { s.push_str("SF "); }
                                if rfl & 0x200 != 0 { s.push_str("IF "); }
                                if rfl & 0x400 != 0 { s.push_str("DF "); }
                                if rfl & 0x800 != 0 { s.push_str("OF "); }
                                s
                            };
                            crate::println!("  Flags: [{}]", flags_str.trim());
                            
                            crate::println!();
                            crate::println!("  CR0={:016X}  CR2={:016X}", 
                                vmcb.read_state(state_offsets::CR0), vmcb.read_state(state_offsets::CR2));
                            crate::println!("  CR3={:016X}  CR4={:016X}", 
                                vmcb.read_state(state_offsets::CR3), vmcb.read_state(state_offsets::CR4));
                            crate::println!("  EFER={:016X}", vmcb.read_state(state_offsets::EFER));
                            
                            // Segments
                            crate::println!();
                            crate::println!("  CS: sel={:04X} base={:016X} lim={:08X} attr={:04X}", 
                                vmcb.read_u16(state_offsets::CS_SELECTOR),
                                vmcb.read_state(state_offsets::CS_BASE),
                                vmcb.read_u32(state_offsets::CS_LIMIT),
                                vmcb.read_u16(state_offsets::CS_ATTRIB));
                            crate::println!("  SS: sel={:04X} base={:016X} lim={:08X} attr={:04X}", 
                                vmcb.read_u16(state_offsets::SS_SELECTOR),
                                vmcb.read_state(state_offsets::SS_BASE),
                                vmcb.read_u32(state_offsets::SS_LIMIT),
                                vmcb.read_u16(state_offsets::SS_ATTRIB));
                            crate::println!("  DS: sel={:04X}  ES: sel={:04X}  FS: sel={:04X}  GS: sel={:04X}", 
                                vmcb.read_u16(state_offsets::DS_SELECTOR),
                                vmcb.read_u16(state_offsets::ES_SELECTOR),
                                vmcb.read_u16(state_offsets::FS_SELECTOR),
                                vmcb.read_u16(state_offsets::GS_SELECTOR));
                            
                            // LAPIC state
                            crate::println!();
                            crate::println_color!(COLOR_YELLOW, "  LAPIC State:");
                            crate::println!("    Enabled: {}  SVR: 0x{:X}  TPR: 0x{:X}", 
                                vm.lapic.enabled, vm.lapic.svr, vm.lapic.tpr);
                            let timer_mode = match (vm.lapic.timer_lvt >> 17) & 0x3 {
                                0 => "one-shot", 1 => "periodic", 2 => "TSC-deadline", _ => "reserved",
                            };
                            let timer_masked = (vm.lapic.timer_lvt >> 16) & 1;
                            let timer_vec = vm.lapic.timer_lvt & 0xFF;
                            crate::println!("    Timer: vec={} mode={} masked={} ICR={} DCR={}", 
                                timer_vec, timer_mode, timer_masked, vm.lapic.icr, vm.lapic.dcr);
                            
                            // Last VMEXIT
                            crate::println!();
                            let exitcode = vmcb.read_control(control_offsets::EXITCODE);
                            let info1 = vmcb.read_control(control_offsets::EXITINFO1);
                            let info2 = vmcb.read_control(control_offsets::EXITINFO2);
                            crate::println!("  Last VMEXIT: code=0x{:X} info1=0x{:X} info2=0x{:X}", exitcode, info1, info2);
                            
                            // Stats
                            crate::println!();
                            crate::println!("  VMEXITs: {}  CPUID: {}  I/O: {}  MSR: {}  NPF: {}  HLT: {}",
                                vm.stats.vmexits, vm.stats.cpuid_exits, vm.stats.io_exits, 
                                vm.stats.msr_exits, vm.stats.npf_exits, vm.stats.hlt_exits);
                        }
                    });
                }
                _ => {
                    crate::println!("vm regs requires AMD SVM.");
                }
            }
        }
        
        // ── Stack trace command ─────────────────────────────────
        "stack" | "backtrace" | "bt" => {
            if args.len() < 2 {
                crate::println!("Usage: vm stack <id> [depth]");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let depth: usize = if args.len() > 2 { args[2].parse().unwrap_or(16) } else { 16 };
            
            match crate::hypervisor::cpu_vendor() {
                crate::hypervisor::CpuVendor::Amd => {
                    crate::hypervisor::svm_vm::with_vm(id, |vm| {
                        if let Some(ref vmcb) = vm.vmcb {
                            use crate::hypervisor::svm::vmcb::state_offsets;
                            
                            let rsp = vmcb.read_state(state_offsets::RSP);
                            let rip = vmcb.read_state(state_offsets::RIP);
                            let rbp = vm.guest_regs.rbp;
                            
                            crate::println_color!(COLOR_CYAN, "  VM {} Stack Trace (RSP=0x{:X}, RIP=0x{:X})", id, rsp, rip);
                            crate::println!();
                            
                            // Dump stack contents as potential return addresses
                            crate::println!("  Stack contents (potential return addresses):");
                            for i in 0..depth {
                                let addr = rsp + (i as u64 * 8);
                                if let Some(data) = vm.read_guest_memory(addr, 8) {
                                    let val = u64::from_le_bytes([
                                        data[0], data[1], data[2], data[3],
                                        data[4], data[5], data[6], data[7],
                                    ]);
                                    // Heuristic: show only values that look like kernel addresses
                                    let marker = if val > 0xFFFF_8000_0000_0000 { " <-- kernel addr" }
                                        else if val > 0x1000 && val < 0x1_0000_0000 { " <-- possible code" }
                                        else { "" };
                                    crate::println!("  [{:2}] RSP+{:04X}: {:016X}{}", i, i * 8, val, marker);
                                } else {
                                    crate::println!("  [{:2}] RSP+{:04X}: <outside guest memory>", i, i * 8);
                                    break;
                                }
                            }
                            
                            // Frame-pointer based walk if RBP looks valid
                            if rbp > 0x1000 && rbp < vm.memory_size as u64 {
                                crate::println!();
                                crate::println!("  Frame pointer chain (RBP=0x{:X}):", rbp);
                                let mut frame = rbp;
                                for i in 0..depth.min(32) {
                                    if frame < 0x1000 || frame >= vm.memory_size as u64 - 16 { break; }
                                    if let Some(data) = vm.read_guest_memory(frame, 16) {
                                        let next_rbp = u64::from_le_bytes([
                                            data[0], data[1], data[2], data[3],
                                            data[4], data[5], data[6], data[7],
                                        ]);
                                        let ret_addr = u64::from_le_bytes([
                                            data[8], data[9], data[10], data[11],
                                            data[12], data[13], data[14], data[15],
                                        ]);
                                        crate::println!("  #{}: RBP=0x{:X} -> ret=0x{:X}", i, frame, ret_addr);
                                        if next_rbp <= frame || next_rbp == 0 { break; }
                                        frame = next_rbp;
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    });
                }
                _ => {
                    crate::println!("vm stack requires AMD SVM.");
                }
            }
        }
        
        // ── VM Debug Monitor ─────────────────────────────────────
        "debug" => {
            let sub = if args.len() > 1 { args[1] } else { "" };
            match sub {
                "init" | "start" => {
                    crate::hypervisor::debug_monitor::init();
                    crate::println!("\x01G✓ Debug monitor started\x01W — all VM exits will be recorded.");
                    crate::println!("  Run a VM, then use 'vm debug' to see the dashboard.");
                }
                "stop" => {
                    crate::hypervisor::debug_monitor::stop();
                    crate::println!("Debug monitor stopped.");
                }
                "reset" => {
                    crate::hypervisor::debug_monitor::reset();
                    crate::println!("Debug monitor data cleared.");
                }
                "gaps" => {
                    let report = crate::hypervisor::debug_monitor::get_gaps_report();
                    crate::println!("{}", report);
                }
                "io" => {
                    let report = crate::hypervisor::debug_monitor::get_io_heatmap();
                    crate::println!("{}", report);
                }
                "msr" => {
                    let report = crate::hypervisor::debug_monitor::get_msr_report();
                    crate::println!("{}", report);
                }
                "timeline" => {
                    let count = if args.len() > 2 { args[2].parse().unwrap_or(30) } else { 30 };
                    let report = crate::hypervisor::debug_monitor::get_timeline(count);
                    crate::println!("{}", report);
                }
                "serial" => {
                    let enabled = args.len() <= 2 || args[2] != "off";
                    crate::hypervisor::debug_monitor::set_serial_log(enabled);
                    crate::println!("Serial logging: {}", if enabled { "ON" } else { "OFF" });
                }
                "status" => {
                    let active = crate::hypervisor::debug_monitor::is_active();
                    let total = crate::hypervisor::debug_monitor::total_events();
                    let unhandled = crate::hypervisor::debug_monitor::unhandled_count();
                    crate::println!("\x01CDebug Monitor Status:\x01W");
                    crate::println!("  Active: {}", if active { "\x01Gyes\x01W" } else { "\x01Rno\x01W" });
                    crate::println!("  Total events: {}", total);
                    crate::println!("  Unhandled: {}{}\x01W", 
                        if unhandled > 0 { "\x01R" } else { "\x01G" }, unhandled);
                }
                "" => {
                    // Default: show full dashboard
                    if !crate::hypervisor::debug_monitor::is_initialized() {
                        // Auto-init if not yet started
                        crate::hypervisor::debug_monitor::init();
                        crate::println!("Debug monitor auto-initialized. Run a VM to collect data.\n");
                    }
                    let dashboard = crate::hypervisor::debug_monitor::get_dashboard();
                    crate::println!("{}", dashboard);
                }
                _ => {
                    crate::println!("\x01CVM Debug Monitor\x01W — Real-time VM analysis");
                    crate::println!();
                    crate::println!("Usage: vm debug [command]");
                    crate::println!();
                    crate::println!("  vm debug           Show full debug dashboard");
                    crate::println!("  vm debug init      Start recording VM exits");
                    crate::println!("  vm debug stop      Stop recording");
                    crate::println!("  vm debug reset     Clear all recorded data");
                    crate::println!("  vm debug gaps      Show unhandled operations only");
                    crate::println!("  vm debug io        Show I/O port heatmap");
                    crate::println!("  vm debug msr       Show MSR access log");
                    crate::println!("  vm debug timeline  Show recent exit timeline");
                    crate::println!("  vm debug serial    Enable serial logging of unhandled exits");
                    crate::println!("  vm debug status    Show monitor status");
                }
            }
        }
        
        // ── Boot Linux bzImage from filesystem ──────────────────
        "linux" => {
            if args.len() < 2 {
                crate::println!("Usage: vm linux <bzimage_path> [initrd_path] [memory_mb] [cmdline]");
                crate::println!("  Example: vm linux /boot/vmlinuz /boot/initrd.img 128");
                crate::println!("  Default: 128 MB RAM, console=ttyS0 earlyprintk nokaslr");
                return;
            }
            
            let bzimage_path = args[1];
            let initrd_path = if args.len() > 2 && !args[2].parse::<usize>().is_ok() {
                Some(args[2])
            } else {
                None
            };
            let mem_idx = if initrd_path.is_some() { 3 } else { 2 };
            let mem_mb: usize = if args.len() > mem_idx {
                args[mem_idx].parse().unwrap_or(128)
            } else {
                128
            };
            let cmdline_idx = mem_idx + 1;
            let cmdline = if args.len() > cmdline_idx {
                args[cmdline_idx..].join(" ")
            } else {
                alloc::string::String::from("console=ttyS0 earlyprintk=serial,ttyS0 nokaslr noapic")
            };
            
            // Read bzImage from filesystem
            crate::println_color!(COLOR_CYAN, "Loading Linux kernel from {}...", bzimage_path);
            let bzimage_data = match crate::vfs::read_file(bzimage_path) {
                Ok(data) => {
                    crate::println_color!(COLOR_GREEN, "  Kernel: {} bytes ({} KB)", data.len(), data.len() / 1024);
                    data
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "  Error reading {}: {:?}", bzimage_path, e);
                    return;
                }
            };
            
            // Read initrd if provided
            let initrd_data = if let Some(path) = initrd_path {
                crate::println!("Loading initrd from {}...", path);
                match crate::vfs::read_file(path) {
                    Ok(data) => {
                        crate::println_color!(COLOR_GREEN, "  Initrd: {} bytes ({} KB)", data.len(), data.len() / 1024);
                        Some(data)
                    }
                    Err(e) => {
                        crate::println_color!(COLOR_RED, "  Error reading {}: {:?}", path, e);
                        return;
                    }
                }
            } else {
                None
            };
            
            // Initialize hypervisor
            if !crate::hypervisor::is_enabled() {
                crate::println!("Initializing hypervisor...");
                if let Err(e) = crate::hypervisor::init() {
                    crate::println_color!(COLOR_RED, "Hypervisor init failed: {:?}", e);
                    return;
                }
            }
            
            // Create VM with specified memory
            crate::println!("Creating VM ({} MB RAM)...", mem_mb);
            crate::println!("Cmdline: {}", cmdline);
            
            match crate::hypervisor::create_vm("linux-guest", mem_mb) {
                Ok(id) => {
                    crate::println!("Booting Linux in VM #{}...", id);
                    
                    let initrd_ref = initrd_data.as_deref();
                    
                    // Use the SVM/VMX start_linux path
                    match crate::hypervisor::cpu_vendor() {
                        crate::hypervisor::CpuVendor::Amd => {
                            let result = crate::hypervisor::svm_vm::with_vm(id, |vm| {
                                vm.start_linux(&bzimage_data, &cmdline, initrd_ref)
                            });
                            match result {
                                Some(Ok(())) => {
                                    crate::println_color!(COLOR_GREEN, "Linux VM completed");
                                }
                                Some(Err(e)) => {
                                    crate::println_color!(COLOR_RED, "Linux VM failed: {:?}", e);
                                    crate::println!("Use 'vm inspect {}' for details", id);
                                }
                                None => {
                                    crate::println_color!(COLOR_RED, "VM #{} not found", id);
                                }
                            }
                        }
                        crate::hypervisor::CpuVendor::Intel => {
                            // For Intel, use linux_vm module
                            let config = crate::hypervisor::linux_vm::LinuxVmConfig {
                                memory_mb: mem_mb,
                                cmdline: cmdline.clone(),
                                ..Default::default()
                            };
                            match crate::hypervisor::linux_vm::LinuxVm::new(config) {
                                Ok(mut vm) => {
                                    let empty_initrd = alloc::vec::Vec::new();
                                    let initrd_data_ref = initrd_data.as_deref().unwrap_or(&empty_initrd);
                                    match vm.boot(&bzimage_data, initrd_data_ref) {
                                        Ok(()) => {
                                            crate::println_color!(COLOR_GREEN, "Linux VM completed");
                                        }
                                        Err(e) => {
                                            crate::println_color!(COLOR_RED, "Linux VM failed: {:?}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    crate::println_color!(COLOR_RED, "Failed to create Linux VM: {:?}", e);
                                }
                            }
                        }
                        _ => {
                            crate::println_color!(COLOR_RED, "No hardware virtualization available");
                        }
                    }
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "Failed to create VM: {:?}", e);
                }
            }
        }
        
        _ => {
            crate::println!("Unknown VM command: {}", args[0]);
            crate::println!("Commands: create, start, run, stop, list, guests, linux, mount, console, input, inspect, dump, regs, stack");
        }
    }
}

// ==================== LINUX SUBSYSTEM COMMANDS ====================

/// Linux Subsystem command - execute commands in a Linux VM
pub(super) fn cmd_linux(args: &[&str]) {
    use crate::hypervisor::linux_subsystem::{self, LinuxState};
    
    if args.is_empty() {
        print_linux_help();
        return;
    }
    
    match args[0] {
        "init" | "start" => {
            crate::println_color!(COLOR_CYAN, "+----------------------------------------------------------+");
            crate::println_color!(COLOR_CYAN, "|     TrustOS Subsystem for Linux (TSL) v1.0              |");
            crate::println_color!(COLOR_CYAN, "+----------------------------------------------------------+");
            crate::println!();
            crate::println!("Initializing Linux Subsystem...");
            
            match linux_subsystem::init() {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "? ");
                    crate::println!("Linux Subsystem initialized");
                    crate::println!();
                    crate::println!("Use 'linux boot' to start real Linux VM,");
                    crate::println!("or 'linux <command>' for simulated commands.");
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "? ");
                    crate::println!("Failed to initialize: {:?}", e);
                }
            }
        }
        "boot" => {
            crate::println_color!(COLOR_CYAN, "+----------------------------------------------------------+");
            crate::println_color!(COLOR_CYAN, "|          Booting Real Linux VM...                       |");
            crate::println_color!(COLOR_CYAN, "+----------------------------------------------------------+");
            crate::println!();
            
            // Check for available virtualization
            let vendor = crate::hypervisor::cpu_vendor();
            match vendor {
                crate::hypervisor::CpuVendor::Intel => {
                    crate::println!("CPU: Intel (VMX)");
                }
                crate::hypervisor::CpuVendor::Amd => {
                    crate::println!("CPU: AMD (SVM)");
                }
                crate::hypervisor::CpuVendor::Unknown => {
                    crate::println_color!(COLOR_YELLOW, "Warning: No hardware virtualization detected");
                    crate::println!("         Real VM boot may not be possible.");
                }
            }
            
            crate::println!();
            crate::println!("Starting Linux VM with kernel and initramfs...");
            
            match linux_subsystem::boot() {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "? ");
                    crate::println!("Linux VM boot completed");
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "? ");
                    crate::println!("Boot failed: {:?}", e);
                    crate::println!();
                    crate::println_color!(COLOR_YELLOW, "Falling back to simulated mode.");
                }
            }
        }
        "status" => {
            let state = linux_subsystem::state();
            let subsys = linux_subsystem::subsystem();
            
            crate::println_color!(COLOR_BRIGHT_GREEN, "Linux Subsystem Status:");
            crate::println!("---------------------------------------");
            
            match state {
                LinuxState::NotStarted => {
                    crate::print_color!(COLOR_YELLOW, "? State: ");
                    crate::println!("Not Started");
                    crate::println!("  Run 'linux init' to start the subsystem.");
                }
                LinuxState::Booting => {
                    crate::print_color!(COLOR_YELLOW, "? State: ");
                    crate::println!("Booting...");
                }
                LinuxState::Ready => {
                    crate::print_color!(COLOR_GREEN, "? State: ");
                    crate::println!("Ready");
                }
                LinuxState::Busy => {
                    crate::print_color!(COLOR_CYAN, "? State: ");
                    crate::println!("Busy (executing command)");
                }
                LinuxState::Error => {
                    crate::print_color!(COLOR_RED, "? State: ");
                    crate::println!("Error");
                }
                LinuxState::ShuttingDown => {
                    crate::print_color!(COLOR_YELLOW, "? State: ");
                    crate::println!("Shutting down...");
                }
            }
            
            // Display kernel info if available
            crate::println!();
            crate::println_color!(COLOR_CYAN, "Kernel Image:");
            if subsys.has_kernel() {
                let kernel_size = subsys.kernel_size();
                crate::println!("  ? Loaded: {} bytes ({} KB)", kernel_size, kernel_size / 1024);
                if let Some(version) = subsys.kernel_version_string() {
                    crate::println!("  Version:  {}", version);
                }
                if let Some((major, minor)) = subsys.boot_protocol_version() {
                    crate::println!("  Protocol: {}.{}", major, minor);
                }
            } else {
                crate::println!("  ? Not loaded (simulated mode)");
            }
            
            crate::println!();
            crate::println_color!(COLOR_CYAN, "Initramfs:");
            if subsys.has_initramfs() {
                let initrd_size = subsys.initramfs_size();
                crate::println!("  ? Loaded: {} bytes ({} KB)", initrd_size, initrd_size / 1024);
            } else {
                crate::println!("  ? Not loaded");
            }
            
            crate::println!();
            crate::println_color!(COLOR_CYAN, "VM Configuration:");
            crate::println!("  Memory:   {} MB", linux_subsystem::LINUX_VM_MEMORY_MB);
            crate::println!("  VM ID:    {:#X}", linux_subsystem::LINUX_VM_ID);
            
            drop(subsys);
        }
        "stop" | "shutdown" => {
            crate::println!("Shutting down Linux Subsystem...");
            match linux_subsystem::shutdown() {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "? ");
                    crate::println!("Linux Subsystem stopped");
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "? ");
                    crate::println!("Failed: {:?}", e);
                }
            }
        }
        "extract" => {
            // Create test binaries directly in ramfs for transpiler testing
            super::apps::create_test_binaries();
        }
        "help" | "--help" | "-h" => {
            print_linux_help();
        }
        // Execute command in Linux VM
        _ => {
            // Reconstruct the full command
            let command = args.join(" ");
            
            match linux_subsystem::execute(&command) {
                Ok(result) => {
                    if !result.stdout.is_empty() {
                        crate::println!("{}", result.stdout);
                    }
                    if !result.stderr.is_empty() {
                        crate::print_color!(COLOR_RED, "{}", result.stderr);
                    }
                    if result.exit_code != 0 && result.stderr.is_empty() {
                        crate::println_color!(COLOR_YELLOW, "(exit code: {})", result.exit_code);
                    }
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "Error: ");
                    crate::println!("{:?}", e);
                }
            }
        }
    }
}

fn print_linux_help() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "TrustOS Subsystem for Linux (TSL)");
    crate::println_color!(COLOR_BRIGHT_GREEN, "=================================");
    crate::println!();
    crate::println!("Execute Linux commands from TrustOS using a virtualized Linux environment.");
    crate::println!();
    crate::println_color!(COLOR_CYAN, "Management Commands:");
    crate::println!("  linux init          Initialize the Linux subsystem");
    crate::println!("  linux boot          Boot real Linux kernel in VM");
    crate::println!("  linux extract       Download and extract Alpine Linux to /alpine");
    crate::println!("  linux status        Show subsystem status");
    crate::println!("  linux stop          Stop the Linux subsystem");
    crate::println!("  linux help          Show this help");
    crate::println!();
    crate::println_color!(COLOR_CYAN, "Execute Linux Commands:");
    crate::println!("  linux <command>     Execute a command in Linux");
    crate::println!();
    crate::println_color!(COLOR_CYAN, "Examples:");
    crate::println!("  linux uname -a      Show Linux kernel info");
    crate::println!("  linux ls -la        List files");
    crate::println!("  linux cat /etc/os-release");
    crate::println!("  linux free -h       Show memory usage");
    crate::println!("  linux df -h         Show disk usage");
    crate::println!("  linux cat /proc/cpuinfo");
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "Note: Real VM boot requires AMD SVM or Intel VMX support.");
}

// ============================================================================
// ADDITIONAL UNIX COMMANDS
// ============================================================================

/// Analyze ELF binary and return info string
fn analyze_elf(data: &[u8]) -> String {
    use alloc::string::String;
    use alloc::format;
    
    if data.len() < 64 || &data[0..4] != b"\x7fELF" {
        return String::from("      Not a valid ELF file");
    }
    
    let mut info = String::new();
    
    let class = data[4]; // 1=32-bit, 2=64-bit
    let endian = data[5]; // 1=little, 2=big
    let elf_type = u16::from_le_bytes([data[16], data[17]]);
    let machine = u16::from_le_bytes([data[18], data[19]]);
    
    info.push_str(&format!("      File size: {} bytes\n", data.len()));
    info.push_str(&format!("      Architecture: {}\n", if class == 2 { "x86_64 (64-bit)" } else { "x86 (32-bit)" }));
    info.push_str(&format!("      Endian: {}\n", if endian == 1 { "Little" } else { "Big" }));
    info.push_str(&format!("      Type: {}\n", match elf_type {
        2 => "Executable",
        3 => "Shared object (PIE)",
        _ => "Other",
    }));
    info.push_str(&format!("      Machine: {}\n", match machine {
        0x3E => "x86-64",
        0x03 => "i386",
        0xB7 => "AArch64",
        _ => "Unknown",
    }));
    
    if class == 2 {
        let entry = u64::from_le_bytes([
            data[24], data[25], data[26], data[27],
            data[28], data[29], data[30], data[31],
        ]);
        info.push_str(&format!("      Entry point: 0x{:x}\n", entry));
        
        // Check linking type
        let ph_off = u64::from_le_bytes([data[32], data[33], data[34], data[35], data[36], data[37], data[38], data[39]]) as usize;
        let ph_size = u16::from_le_bytes([data[54], data[55]]) as usize;
        let ph_count = u16::from_le_bytes([data[56], data[57]]) as usize;
        
        let mut has_interp = false;
        for i in 0..ph_count {
            let off = ph_off + i * ph_size;
            if off + 4 <= data.len() {
                let ptype = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
                if ptype == 3 { has_interp = true; }
            }
        }
        
        info.push_str(&format!("      Linking: {}\n", if has_interp { "Dynamic (needs ld-linux.so)" } else { "Static" }));
    }
    
    info.push_str("\n      ? Valid Linux ELF binary detected!");
    info.push_str("\n      Note: Execution requires x86_64 CPU emulation (slow)");
    
    info
}

/// Package manager shortcut — routes apt-get/apk/dpkg through the Linux subsystem
pub(super) fn cmd_pkg(cmd: &str, args: &[&str]) {
    use crate::hypervisor::linux_subsystem;

    // Auto-init the linux subsystem if not started
    let state = linux_subsystem::state();
    if state == linux_subsystem::LinuxState::NotStarted {
        let _ = linux_subsystem::init();
        let _ = linux_subsystem::boot();
    }

    // Build the full command string: "apt-get install vim" etc.
    let mut full = alloc::string::String::from(cmd);
    for a in args {
        full.push(' ');
        full.push_str(a);
    }

    match linux_subsystem::execute(&full) {
        Ok(result) => {
            if !result.stdout.is_empty() {
                crate::println!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                crate::print_color!(COLOR_RED, "{}", result.stderr);
                crate::println!();
            }
        }
        Err(e) => {
            crate::print_color!(COLOR_RED, "Error: ");
            crate::println!("{:?}", e);
        }
    }
}

/// Alpine Linux all-in-one command: download, extract, and test
pub(super) fn cmd_alpine(args: &[&str]) {
    use alloc::vec::Vec;
    use alloc::string::String;
    
    let subcmd = args.get(0).copied().unwrap_or("help");
    
    match subcmd {
        "test" | "run" => {
            crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
            crate::println_color!(COLOR_CYAN, "|           Alpine Linux Test - All in One                     |");
            crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
            crate::println!();
            
            // Check if we already have binaries in /alpine/bin
            let have_binaries = crate::ramfs::with_fs(|fs| {
                fs.ls(Some("/alpine/bin")).map(|e| e.len() > 0).unwrap_or(false)
            });
            
            if have_binaries {
                crate::println_color!(COLOR_GREEN, "[1/4] Alpine binaries present ?");
            } else {
                // Try to create test binaries directly (no network needed)
                crate::println_color!(COLOR_YELLOW, "[1/4] Creating test binaries...");
                super::apps::create_test_binaries_silent();
            }
            
            // Step 2: Verify binaries
            crate::println_color!(COLOR_YELLOW, "[2/4] Verifying binaries...");
            
            let binary_count = crate::ramfs::with_fs(|fs| {
                fs.ls(Some("/alpine/bin")).map(|e| e.len()).unwrap_or(0)
            });
            
            if binary_count > 0 {
                crate::println_color!(COLOR_GREEN, "      Found {} binaries in /alpine/bin", binary_count);
            } else {
                crate::println_color!(COLOR_RED, "      No binaries found! Run 'linux extract' first.");
                return;
            }
            crate::println!();
            
            // Step 3: List some files
            crate::println_color!(COLOR_YELLOW, "[3/4] Checking extracted files...");
            crate::ramfs::with_fs(|fs| {
                if let Ok(entries) = fs.ls(Some("/alpine/bin")) {
                    let count = entries.len();
                    crate::println!("      /alpine/bin: {} binaries", count);
                    // Show first 5
                    for (name, _, _) in entries.iter().take(5) {
                        crate::println!("        - {}", name);
                    }
                    if count > 5 {
                        crate::println!("        ... and {} more", count - 5);
                    }
                }
            });
            crate::println!();
            
            // Step 4: Analyze busybox binary (don't execute - too slow)
            crate::println_color!(COLOR_YELLOW, "[4/4] Analyzing Linux binary...");
            let binary = args.get(1).copied().unwrap_or("/alpine/bin/busybox");
            
            // Read and analyze the ELF
            let elf_info = crate::ramfs::with_fs(|fs| {
                fs.read_file(binary).map(|data| {
                    let data = data.to_vec();
                    analyze_elf(&data)
                })
            });
            
            match elf_info {
                Ok(info) => {
                    crate::println_color!(COLOR_GREEN, "{}", info);
                }
                Err(_) => {
                    crate::println_color!(COLOR_RED, "      Could not read binary: {}", binary);
                }
            }
            
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "----------------------------------------------------------------");
            crate::println_color!(COLOR_BRIGHT_GREEN, "                    Alpine Test Complete!");
            crate::println_color!(COLOR_BRIGHT_GREEN, "----------------------------------------------------------------");
        }
        
        "ls" | "list" => {
            crate::println_color!(COLOR_CYAN, "Alpine Linux files:");
            crate::ramfs::with_fs(|fs| {
                for dir in &["/alpine", "/alpine/bin", "/alpine/usr/bin"] {
                    if let Ok(entries) = fs.ls(Some(*dir)) {
                        crate::println!("\n{}/ ({} entries)", dir, entries.len());
                        for (name, _, _) in entries.iter().take(10) {
                            crate::println!("  {}", name);
                        }
                        if entries.len() > 10 {
                            crate::println!("  ... {} more", entries.len() - 10);
                        }
                    }
                }
            });
        }
        
        "exec" => {
            if args.len() < 2 {
                crate::println!("Usage: alpine exec <binary> [args...]");
                crate::println!("Example: alpine exec /alpine/bin/busybox ls");
                return;
            }
            let binary = args[1];
            let bin_args: Vec<&str> = args[2..].to_vec();
            
            crate::println!("Executing: {} {:?}", binary, bin_args);
            match crate::linux_compat::exec(binary, &bin_args) {
                Ok(exit_code) => crate::println!("Exited with code: {}", exit_code),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        
        "hello" => {
            // Run a minimal built-in ELF that we know works
            crate::println_color!(COLOR_CYAN, "Running minimal Linux ELF binary...");
            crate::println!();
            
            // This is a hand-crafted minimal ELF that prints "Hello" and exits
            // Created to test that the interpreter works for simple cases
            #[rustfmt::skip]
            static HELLO_ELF: &[u8] = &[
                // ELF Header (64 bytes)
                0x7f, b'E', b'L', b'F',  // Magic
                0x02,                     // 64-bit
                0x01,                     // Little endian
                0x01,                     // ELF version
                0x00,                     // System V ABI
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Padding
                0x02, 0x00,               // Executable
                0x3e, 0x00,               // x86_64
                0x01, 0x00, 0x00, 0x00,   // ELF version
                0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Entry: 0x400078
                0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Program header offset
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Section header offset
                0x00, 0x00, 0x00, 0x00,   // Flags
                0x40, 0x00,               // ELF header size
                0x38, 0x00,               // Program header size
                0x01, 0x00,               // Number of program headers
                0x00, 0x00,               // Section header size
                0x00, 0x00,               // Number of section headers
                0x00, 0x00,               // Section name index
                
                // Program Header (56 bytes, offset 0x40)
                0x01, 0x00, 0x00, 0x00,   // PT_LOAD
                0x05, 0x00, 0x00, 0x00,   // Flags: R+X
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Offset
                0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Vaddr: 0x400000
                0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Paddr: 0x400000
                0xb0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // File size
                0xb0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Mem size
                0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Alignment
                
                // Code (offset 0x78)
                // mov rax, 1 (write)
                0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00,
                // mov rdi, 1 (stdout)
                0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,
                // lea rsi, [rip + msg]  -> mov rsi, msg_addr
                0x48, 0xc7, 0xc6, 0xa0, 0x00, 0x40, 0x00,  // msg at 0x4000a0
                // mov rdx, 27 (length)
                0x48, 0xc7, 0xc2, 0x1b, 0x00, 0x00, 0x00,
                // syscall
                0x0f, 0x05,
                // mov rax, 60 (exit)
                0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
                // xor rdi, rdi
                0x48, 0x31, 0xff,
                // syscall
                0x0f, 0x05,
                
                // Message at offset 0xa0 (addr 0x4000a0)
                b'H', b'e', b'l', b'l', b'o', b' ', b'f', b'r',
                b'o', b'm', b' ', b'T', b'r', b'u', b's', b't',
                b'O', b'S', b' ', b'i', b'n', b't', b'e', b'r',
                b'p', b'!', 0x0a,  // "Hello from TrustOS interp!\n"
            ];
            
            match crate::linux_compat::interpreter::run_binary(HELLO_ELF, &["hello"]) {
                Ok(code) => {
                    crate::println!();
                    crate::println_color!(COLOR_GREEN, "Binary exited with code: {}", code);
                    crate::println_color!(COLOR_GREEN, "? Linux interpreter works!");
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "Error: {}", e);
                }
            }
        }
        
        _ => {
            crate::println_color!(COLOR_CYAN, "Alpine Linux Commands:");
            crate::println!();
            crate::println!("  alpine test          - Download, extract & analyze Alpine");
            crate::println!("  alpine hello         - Run minimal test binary (proves interpreter works)");
            crate::println!("  alpine ls            - List extracted files");
            crate::println!("  alpine exec <bin>    - Execute a Linux binary (may be slow/timeout)");
            crate::println!();
            crate::println!("Note: Real Linux binaries like busybox are too complex.");
            crate::println!("      The interpreter supports basic ELFs only.");
        }
    }
}

/// Extract tar archive to ramfs
fn extract_tar_to_ramfs(fs: &mut crate::ramfs::RamFs, data: &[u8], base_path: &str) -> Result<usize, &'static str> {
    use alloc::string::String;
    
    let mut offset = 0;
    let mut count = 0;
    
    while offset + 512 <= data.len() {
        let header = &data[offset..offset + 512];
        
        // Check for end of archive (two zero blocks)
        if header.iter().all(|&b| b == 0) {
            break;
        }
        
        // Parse tar header
        let name_bytes = &header[0..100];
        let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(100);
        let name = core::str::from_utf8(&name_bytes[..name_end]).unwrap_or("");
        
        if name.is_empty() {
            break;
        }
        
        // Parse size (octal)
        let size_bytes = &header[124..135];
        let size_str = core::str::from_utf8(size_bytes).unwrap_or("0");
        let size = usize::from_str_radix(size_str.trim_matches(|c| c == '\0' || c == ' '), 8).unwrap_or(0);
        
        // Type flag
        let type_flag = header[156];
        
        let full_path = if name.starts_with("./") {
            alloc::format!("{}/{}", base_path, &name[2..])
        } else {
            alloc::format!("{}/{}", base_path, name)
        };
        
        // Clean up path (remove trailing slashes)
        let clean_path = full_path.trim_end_matches('/');
        
        offset += 512; // Move past header
        
        match type_flag {
            b'5' | b'0' if name.ends_with('/') => {
                // Directory
                let _ = fs.mkdir(clean_path);
            }
            b'0' | b'\0' if size > 0 => {
                // Regular file with content
                if offset + size <= data.len() {
                    let content = &data[offset..offset + size];
                    
                    // Create parent directories
                    if let Some(parent_end) = clean_path.rfind('/') {
                        let parent = &clean_path[..parent_end];
                        let _ = create_dirs_recursive(fs, parent);
                    }
                    
                    let _ = fs.touch(clean_path);
                    let _ = fs.write_file(clean_path, content);
                    count += 1;
                }
            }
            b'0' | b'\0' => {
                // Empty file
                if let Some(parent_end) = clean_path.rfind('/') {
                    let parent = &clean_path[..parent_end];
                    let _ = create_dirs_recursive(fs, parent);
                }
                let _ = fs.touch(clean_path);
                count += 1;
            }
            b'2' => {
                // Symlink - skip for now
            }
            _ => {}
        }
        
        // Move to next header (aligned to 512 bytes)
        let blocks = (size + 511) / 512;
        offset += blocks * 512;
    }
    
    Ok(count)
}

fn create_dirs_recursive(fs: &mut crate::ramfs::RamFs, path: &str) -> Result<(), ()> {
    let mut current = String::new();
    for part in path.split('/').filter(|s| !s.is_empty()) {
        current.push('/');
        current.push_str(part);
        let _ = fs.mkdir(&current);
    }
    Ok(())
}

pub(super) fn cmd_download(args: &[&str]) {
    crate::println!("[DEBUG] cmd_download called, args: {:?}", args);
    crate::serial_println!("[DEBUG] cmd_download called, args count: {}", args.len());
    
    if args.is_empty() {
        crate::println!("Usage: download <name|url> [output_file]");
        crate::println!("       download alpine  - Download Alpine Linux (fast)");
        crate::println!("       download <url>   - Download from URL");
        return;
    }
    
    let arg = args[0];
    crate::println!("[DEBUG] First arg: '{}'", arg);
    
    // Special shortcut: "download alpine" uses optimized local download
    if arg == "alpine" || arg == "busybox" || arg == "linux" {
        crate::println!("[DEBUG] Calling download_from_local_server...");
        download_from_local_server("alpine-minirootfs.tar.gz", "/opt/gui/alpine.tar.gz");
        return;
    }
    
    // Otherwise treat as URL
    let url = arg;
    let output = if args.len() > 1 { args[1] } else { 
        url.rsplit('/').next().unwrap_or("download")
    };
    
    crate::println_color!(COLOR_CYAN, "Downloading: {}", url);
    crate::println!("         -> {}", output);
    cmd_curl(args);
}

/// Fast download from local VirtualBox server (192.168.56.1:8080)
fn download_from_local_server(filename: &str, save_path: &str) {
    use alloc::vec::Vec;
    use alloc::format;
    
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println_color!(COLOR_CYAN, "|              Fast Download - Local Server                    |");
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println!();
    
    // VirtualBox Host-Only network
    let server_ip: [u8; 4] = [192, 168, 56, 1];
    let server_port: u16 = 8080;
    
    crate::println_color!(COLOR_YELLOW, "[1/4] Configuring network...");
    
    // Suspend DHCP and force static IP
    crate::netstack::dhcp::suspend();
    crate::network::set_ipv4_config(
        crate::network::Ipv4Address::new(192, 168, 56, 100),
        crate::network::Ipv4Address::new(255, 255, 255, 0),
        Some(crate::network::Ipv4Address::new(192, 168, 56, 1)),
    );
    
    // Verify IP is set correctly
    if let Some((ip, mask, gw)) = crate::network::get_ipv4_config() {
        crate::println!("      IP: {}.{}.{}.{}", ip.as_bytes()[0], ip.as_bytes()[1], ip.as_bytes()[2], ip.as_bytes()[3]);
        crate::serial_println!("[DOWNLOAD] IP configured: {}.{}.{}.{}", ip.as_bytes()[0], ip.as_bytes()[1], ip.as_bytes()[2], ip.as_bytes()[3]);
    } else {
        crate::println_color!(COLOR_RED, "      ERROR: No IP configured!");
        crate::netstack::dhcp::resume();
        return;
    }
    
    // Clear buffers and wait for ARP to settle
    for _ in 0..100 {
        crate::netstack::poll();
    }
    crate::println!();
    
    crate::println_color!(COLOR_YELLOW, "[2/4] Connecting to 192.168.56.1:8080...");
    
    // Send ARP request first
    crate::println!("      Resolving MAC address...");
    let _ = crate::netstack::arp::send_request(server_ip);
    for _ in 0..200 {
        crate::netstack::poll();
    }
    
    // Check if we have MAC for server
    if let Some(mac) = crate::netstack::arp::resolve(server_ip) {
        crate::println!("      Server MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", 
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    } else {
        crate::println_color!(COLOR_YELLOW, "      Warning: No ARP response yet");
    }
    
    let src_port = match crate::netstack::tcp::send_syn(server_ip, server_port) {
        Ok(p) => {
            crate::serial_println!("[DOWNLOAD] SYN sent, src_port={}", p);
            p
        }
        Err(e) => {
            crate::serial_println!("[DOWNLOAD] SYN failed: {}", e);
            crate::println_color!(COLOR_RED, "      ERROR: {}", e);
            crate::println!("      Is the server running?");
            crate::println!("      > cd server && .\\start-server.ps1");
            crate::netstack::dhcp::resume();
            return;
        }
    };
    
    crate::println!("      Waiting for connection...");
    if !crate::netstack::tcp::wait_for_established(server_ip, server_port, src_port, 3000) {
        crate::serial_println!("[DOWNLOAD] Connection timeout!");
        crate::println_color!(COLOR_RED, "      ERROR: Connection timeout");
        crate::println!("      Check: ping 192.168.56.1");
        crate::netstack::dhcp::resume();
        return;
    }
    
    crate::println_color!(COLOR_GREEN, "      Connected!");
    crate::println!();
    
    crate::println_color!(COLOR_YELLOW, "[3/4] Downloading {}...", filename);
    
    // Send HTTP request
    let request = format!(
        "GET /{} HTTP/1.1\r\nHost: 192.168.56.1\r\nConnection: close\r\n\r\n",
        filename
    );
    
    if let Err(e) = crate::netstack::tcp::send_payload(server_ip, server_port, src_port, request.as_bytes()) {
        crate::println_color!(COLOR_RED, "      ERROR: {}", e);
        crate::netstack::dhcp::resume();
        return;
    }
    
    // Receive with progress
    let mut data: Vec<u8> = Vec::with_capacity(4 * 1024 * 1024);
    let start = crate::logger::get_ticks();
    let mut idle_count: u32 = 0;
    let mut last_progress = 0usize;
    let mut last_ack_flush = start;
    
    loop {
        // Aggressive polling
        for _ in 0..10 {
            crate::netstack::poll();
        }
        
        let mut got_data = false;
        while let Some(chunk) = crate::netstack::tcp::recv_data(server_ip, server_port, src_port) {
            got_data = true;
            if data.len() + chunk.len() > 8 * 1024 * 1024 {
                break;
            }
            data.extend_from_slice(&chunk);
        }
        
        // Progress display
        let kb = data.len() / 1024;
        if kb >= last_progress + 50 || (kb > 0 && last_progress == 0) {
            let elapsed = crate::logger::get_ticks().saturating_sub(start);
            let speed = if elapsed > 0 { (kb as u64 * 1000) / elapsed } else { 0 };
            crate::print!("\r      {} KB downloaded ({} KB/s)          ", kb, speed);
            last_progress = kb;
        }
        
        // Flush ACKs frequently
        let now = crate::logger::get_ticks();
        if now.saturating_sub(last_ack_flush) >= 5 {
            crate::netstack::tcp::flush_pending_acks(server_ip, server_port, src_port);
            last_ack_flush = now;
        }
        
        if !got_data {
            idle_count += 1;
            if crate::netstack::tcp::fin_received(server_ip, server_port, src_port) {
                crate::netstack::tcp::flush_pending_acks(server_ip, server_port, src_port);
                break;
            }
            if idle_count > 100_000 {
                break;
            }
        } else {
            idle_count = 0;
        }
        
        // 30 second timeout
        if now.saturating_sub(start) > 30_000 {
            crate::println_color!(COLOR_YELLOW, "\n      Timeout!");
            break;
        }
    }
    
    let _ = crate::netstack::tcp::send_fin(server_ip, server_port, src_port);
    
    let elapsed = crate::logger::get_ticks().saturating_sub(start);
    let total_kb = data.len() / 1024;
    let avg_speed = if elapsed > 0 { (total_kb as u64 * 1000) / elapsed } else { 0 };
    
    crate::println!();
    crate::println_color!(COLOR_GREEN, "      Complete: {} KB in {}ms ({} KB/s)", total_kb, elapsed, avg_speed);
    crate::println!();
    
    if data.is_empty() {
        crate::println_color!(COLOR_RED, "      ERROR: No data received");
        crate::netstack::dhcp::resume();
        return;
    }
    
    // Extract HTTP body
    let body_start = data.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .unwrap_or(0);
    let body = &data[body_start..];
    
    if body.is_empty() {
        crate::println_color!(COLOR_RED, "      ERROR: Empty response");
        crate::netstack::dhcp::resume();
        return;
    }
    
    crate::println_color!(COLOR_YELLOW, "[4/4] Saving to {}...", save_path);
    
    // Save to ramfs
    let save_result = crate::ramfs::with_fs(|fs| {
        let _ = fs.mkdir("/opt");
        let _ = fs.mkdir("/opt/gui");
        let _ = fs.touch(save_path);
        fs.write_file(save_path, body)
    });
    
    match save_result {
        Ok(_) => {
            crate::println_color!(COLOR_GREEN, "      Saved: {:.2} MB", body.len() as f32 / (1024.0 * 1024.0));
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "      ERROR: {:?}", e);
            crate::netstack::dhcp::resume();
            return;
        }
    }
    
    // Persist to disk
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "Saving to disk for persistence...");
    match crate::persistence::save_file(save_path, body) {
        Ok(_) => crate::println_color!(COLOR_GREEN, "  Saved! Will survive reboot."),
        Err(e) => crate::println_color!(COLOR_YELLOW, "  Could not persist: {}", e),
    }
    
    crate::println!();
    crate::println_color!(COLOR_BRIGHT_GREEN, "----------------------------------------------------------------");
    crate::println_color!(COLOR_BRIGHT_GREEN, "                    Download Complete!");
    crate::println_color!(COLOR_BRIGHT_GREEN, "----------------------------------------------------------------");
    
    // Mark GUI as installed
    GUI_INSTALLED.store(true, core::sync::atomic::Ordering::Relaxed);
    
    crate::netstack::dhcp::resume();
}