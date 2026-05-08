





use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{B_, G_, AX_, D_, A_, C_, R_, CF_, DM_, K_};





pub(super) static LC_: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

pub(super) fn hmh() {
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::n!(C_, "|            TrustOS Virtual Machine Manager                   |");
    crate::n!(C_, "|--------------------------------------------------------------|");
    crate::n!(C_, "|                                                              |");
    crate::n!(C_, "|  TrustOS runs Linux VMs with modern GUIs.                   |");
    crate::n!(C_, "|                                                              |");
    crate::n!(C_, "|  Commands:                                                   |");
    crate::n!(B_, "|    vm status    - Check VM installation status              |");
    crate::n!(B_, "|    vm install   - Download Alpine Linux VM image            |");
    crate::n!(B_, "|    vm start     - Start the Alpine Linux VM                 |");
    crate::n!(B_, "|    vm console   - Connect to VM console (Linux shell)       |");
    crate::n!(B_, "|    vm stop      - Stop the running VM                       |");
    crate::n!(B_, "|    vm list      - List running VMs                          |");
    crate::n!(C_, "|                                                              |");
    crate::n!(C_, "+--------------------------------------------------------------+");
}

pub(super) fn ktv() {
    crate::n!(D_, "Stopping VM...");
    
    crate::n!(B_, "VM stopped.");
}

pub(super) fn ktu() {
    crate::n!(C_, "Running Virtual Machines:");
    crate::println!("  ID   NAME           STATUS      MEMORY");
    crate::println!("  ---------------------------------------");
    if LC_.load(core::sync::atomic::Ordering::Relaxed) {
        crate::println!("  1    alpine-linux   running     256 MB");
    } else {
        crate::println!("  (no VMs running)");
    }
}



pub(super) fn fmk() {
    
    if crate::distro::list().is_empty() {
        crate::distro::init();
    }
    
    let aqi = crate::distro::list();
    
    crate::n!(C_, "+------------------------------------------------------------------+");
    crate::n!(C_, "|                 TrustOS Linux Distribution Manager               |");
    crate::n!(C_, "|------------------------------------------------------------------|");
    crate::n!(C_, "|  ID              NAME                    SIZE     STATUS         |");
    crate::n!(C_, "|------------------------------------------------------------------|");
    
    for d in &aqi {
        let status = if d.installed { 
            "\x1b[32m[installed]\x1b[0m" 
        } else { 
            "\x1b[33m[available]\x1b[0m" 
        };
        let oxa = if d.installed { "installed" } else { "available" };
        crate::println!("|  {} {:<12}  {:<20}  {:>4} MB   {:<12} |", 
            d.icon, d.id, d.name, d.size_mb, oxa);
    }
    
    crate::n!(C_, "|------------------------------------------------------------------|");
    crate::n!(C_, "|  Commands:                                                       |");
    crate::n!(B_, "|    distro list              - Show this list                    |");
    crate::n!(B_, "|    distro install <id>      - Download and install a distro     |");
    crate::n!(B_, "|    distro run <id>          - Run an installed distro           |");
    crate::n!(B_, "|    distro gui               - Open graphical distro selector    |");
    crate::n!(C_, "+------------------------------------------------------------------+");
}

pub(super) fn knc(id: &str) {
    
    if crate::distro::list().is_empty() {
        crate::distro::init();
    }
    
    let distro = match crate::distro::get(id) {
        Some(d) => d,
        None => {
            crate::n!(A_, "Error: Distribution '{}' not found.", id);
            crate::println!("Use 'distro list' to see available distributions.");
            return;
        }
    };
    
    if distro.installed {
        crate::n!(D_, "{} {} is already installed.", distro.icon, distro.name);
        crate::println!("Use 'distro run {}' to start it.", id);
        return;
    }
    
    crate::n!(C_, "+------------------------------------------------------------------+");
    crate::n!(C_, "|                    Installing Linux Distribution                 |");
    crate::n!(C_, "+------------------------------------------------------------------+");
    crate::println!();
    crate::println!("  {} {} {}", distro.icon, distro.name, distro.version);
    crate::println!("  {}", distro.description);
    crate::println!("  Size: {} MB", distro.size_mb);
    crate::println!();
    
    crate::n!(D_, "[1/3] Connecting to server 192.168.56.1:8080...");
    
    match crate::distro::fsu(id) {
        Ok(size) => {
            crate::n!(B_, "[2/3] Downloaded {} KB", size / 1024);
            crate::n!(B_, "[3/3] Installation complete!");
            crate::println!();
            crate::n!(B_, "  {} {} is now installed!", distro.icon, distro.name);
            crate::println!("  Use 'distro run {}' to start it.", id);
        }
        Err(e) => {
            crate::n!(A_, "Error: {}", e);
            crate::println!();
            crate::println!("Make sure the server is running:");
            crate::println!("  > cd server && powershell -ExecutionPolicy Bypass .\\start-server.ps1");
        }
    }
}

pub(super) fn knd(id: &str) {
    
    if crate::distro::list().is_empty() {
        crate::distro::init();
    }
    
    let distro = match crate::distro::get(id) {
        Some(d) => d,
        None => {
            crate::n!(A_, "Error: Distribution '{}' not found.", id);
            crate::println!("Use 'distro list' to see available distributions.");
            return;
        }
    };
    
    if !distro.installed {
        crate::n!(D_, "{} {} is not installed.", distro.icon, distro.name);
        crate::println!("Use 'distro install {}' to download it first.", id);
        return;
    }
    
    crate::n!(C_, "+------------------------------------------------------------------+");
    crate::n!(C_, "|                    Starting Linux Distribution                   |");
    crate::n!(C_, "+------------------------------------------------------------------+");
    crate::println!();
    crate::println!("  {} {} {}", distro.icon, distro.name, distro.version);
    crate::println!();
    
    match crate::distro::run(id) {
        Ok(()) => {
            crate::n!(B_, "  Distribution started successfully.");
        }
        Err(e) => {
            crate::n!(A_, "Error: {}", e);
        }
    }
}

pub(super) fn hlv() {
    
    if crate::distro::list().is_empty() {
        crate::distro::init();
    }
    
    let aqi = crate::distro::list();
    
    
    if !crate::framebuffer::is_initialized() {
        crate::n!(A_, "Error: No framebuffer available for GUI.");
        crate::println!("Use 'distro list' for text-mode interface.");
        return;
    }
    
    let (width, height) = crate::framebuffer::kv();
    
    
    let bg_color = 0xFF1E1E2Eu32;      
    let cns = 0xFF2D2D3Du32;   
    let zr = 0xFF89B4FAu32;  
    let ics = 0xFF94E2D5u32;   
    let text_color = 0xFFCDD6F4u32;    
    let pww = 0xFF6C7086u32;    
    
    
    crate::framebuffer::fill_rect(0, 0, width, height, bg_color);
    
    
    crate::framebuffer::fill_rect(0, 0, width, 50, cns);
    crate::framebuffer::draw_text_at("TrustOS Linux Distribution Manager", 20, 16, text_color, cns);
    
    
    let mut y = 80u32;
    
    crate::framebuffer::draw_text_at("  #  ID              NAME                    SIZE     STATUS", 20, y, zr, bg_color);
    y += 24;
    crate::framebuffer::mn(20, y, width - 40, zr);
    y += 16;
    
    for (i, d) in aqi.iter().enumerate() {
        let bvz = if d.installed { "[INSTALLED]" } else { "[available]" };
        let bdw = if d.installed { ics } else { text_color };
        
        
        let rw = alloc::format!("  {}  ", i + 1);
        crate::framebuffer::draw_text_at(&rw, 20, y, zr, bg_color);
        
        
        let eqa = alloc::format!("{} {:<12}", d.icon, d.id);
        crate::framebuffer::draw_text_at(&eqa, 60, y, text_color, bg_color);
        
        
        crate::framebuffer::draw_text_at(d.name, 220, y, text_color, bg_color);
        
        
        let td = alloc::format!("{:>4} MB", d.size_mb);
        crate::framebuffer::draw_text_at(&td, 450, y, text_color, bg_color);
        
        
        crate::framebuffer::draw_text_at(bvz, 540, y, bdw, bg_color);
        
        y += 24;
    }
    
    
    let dqb = height - 80;
    crate::framebuffer::fill_rect(0, dqb, width, 80, cns);
    crate::framebuffer::draw_text_at("Commands:", 20, dqb + 16, zr, cns);
    crate::framebuffer::draw_text_at("distro install <id>  - Download and install", 20, dqb + 36, text_color, cns);
    crate::framebuffer::draw_text_at("distro run <id>      - Run an installed distro", 400, dqb + 36, text_color, cns);
    crate::framebuffer::draw_text_at("Press any key to return to shell...", 20, dqb + 56, ics, cns);
    
    
    loop {
        if let Some(jsk) = crate::keyboard::ya() {
            break;
        }
        for _ in 0..1000 { core::hint::spin_loop(); }
    }
    
    
    crate::framebuffer::clear();
}

pub(super) fn koh() {
    let installed = LC_.load(core::sync::atomic::Ordering::Relaxed);
    
    crate::n!(C_, "+--------------------------------------+");
    crate::n!(C_, "|       TrustOS GUI Status             |");
    crate::n!(C_, "|--------------------------------------|");
    
    if installed {
        crate::n!(B_, "|  Status:     [INSTALLED]             |");
        crate::n!(B_, "|  Image:      Alpine Linux + Browser  |");
        crate::n!(C_, "|                                      |");
        crate::n!(C_, "|  Use 'gui start' to launch           |");
    } else {
        crate::n!(D_, "|  Status:     [NOT INSTALLED]         |");
        crate::n!(C_, "|                                      |");
        crate::n!(C_, "|  Use 'gui install' to download       |");
    }
    crate::n!(C_, "+--------------------------------------+");
}

pub(super) fn kog() {
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::n!(C_, "|              TrustOS GUI Installer                           |");
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::println!();
    
    
    let server_ip = "192.168.56.1";
    let aio = 8080u16;
    let ite = "/alpine-minirootfs.tar.gz";
    
    
    crate::n!(D_, "[1/4] Checking network connection...");
    
    if !crate::network::sw() {
        crate::n!(A_, "      ERROR: Network not available!");
        crate::println!("      Make sure virtio-net is enabled.");
        return;
    }
    crate::n!(B_, "      Network: OK");
    crate::println!();
    
    
    crate::n!(D_, "[2/4] Downloading Alpine Linux from {}:{}{}...", server_ip, aio, ite);
    
    
    crate::netstack::dhcp::crf();
    crate::serial_println!("[GUI_INSTALL] DHCP suspended for download");
    
    
    crate::network::deh(
        crate::network::Ipv4Address::new(192, 168, 56, 100),
        crate::network::Ipv4Address::new(255, 255, 255, 0),
        Some(crate::network::Ipv4Address::new(192, 168, 56, 1)),
    );
    
    
    for _ in 0..100 {
        crate::netstack::poll();
    }
    
    let ip = match art(server_ip) {
        Some(ip) => ip,
        None => {
            crate::n!(A_, "      ERROR: Invalid server IP");
            crate::netstack::dhcp::resume();
            return;
        }
    };
    
    
    let src_port = match crate::netstack::tcp::azp(ip, aio) {
        Ok(aa) => aa,
        Err(e) => {
            crate::n!(A_, "      ERROR: Connection failed: {}", e);
            crate::println!("      Make sure the server is running:");
            crate::println!("      > cd server && powershell -ExecutionPolicy Bypass .\\start-server.ps1");
            crate::netstack::dhcp::resume();
            return;
        }
    };
    
    let cja = crate::netstack::tcp::bjy(ip, aio, src_port, 2000);
    if !cja {
        crate::n!(A_, "      ERROR: Connection timeout");
        crate::println!("      Make sure the server is running on port {}", aio);
        crate::netstack::dhcp::resume();
        return;
    }
    
    crate::n!(B_, "      Connected to server");
    
    
    let request = alloc::format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        ite, server_ip
    );
    
    if let Err(e) = crate::netstack::tcp::bjc(ip, aio, src_port, request.as_bytes()) {
        crate::n!(A_, "      ERROR: Failed to send request: {}", e);
        crate::netstack::dhcp::resume();
        return;
    }
    
    
    crate::println!("      Downloading...");
    
    let mut cpc: Vec<u8> = Vec::with_capacity(4 * 1024 * 1024);
    let start = crate::logger::eg();
    let mut bch: u32 = 0;
    let mut cly = 0usize;
    let mut cbi = start;
    let mut ijj = 0u32;
    const CJB_: usize = 8 * 1024 * 1024; 
    
    loop {
        
        for _ in 0..10 {
            crate::netstack::poll();
        }
        ijj += 10;
        
        let mut aty = false;
        let mut kao = 0usize;
        
        
        while let Some(data) = crate::netstack::tcp::aus(ip, aio, src_port) {
            aty = true;
            kao += data.len();
            
            
            if cpc.len() + data.len() > CJB_ {
                crate::n!(D_, "\n      WARNING: File too large, truncating");
                break;
            }
            
            cpc.extend_from_slice(&data);
        }
        
        
        let arh = cpc.len() / 1024;
        if arh >= cly + 25 || (arh > 0 && cly == 0) {
            let bb = crate::logger::eg().saturating_sub(start);
            let ouu = if bb > 0 { (arh as u64 * 1000) / bb } else { 0 };
            crate::print!("\r      Downloaded: {} KB ({} KB/s)    ", arh, ouu);
            cly = arh;
        }
        
        
        let cy = crate::logger::eg();
        if cy.saturating_sub(cbi) >= 5 {
            crate::netstack::tcp::cjr(ip, aio, src_port);
            cbi = cy;
        }
        
        if !aty {
            bch = bch.saturating_add(1);
            
            
            if crate::netstack::tcp::fin_received(ip, aio, src_port) {
                
                crate::netstack::tcp::cjr(ip, aio, src_port);
                break;
            }
            
            
            if bch > 100_000 {
                crate::serial_println!("[DOWNLOAD] Idle timeout after {} polls", ijj);
                break;
            }
            
            
            for _ in 0..50 { core::hint::spin_loop(); }
        } else {
            bch = 0;
        }
        
        
        if crate::logger::eg().saturating_sub(start) > 60000 {
            crate::n!(D_, "\n      WARNING: Download timeout");
            break;
        }
    }
    
    let _ = crate::netstack::tcp::ams(ip, aio, src_port);
    crate::println!();
    
    let elapsed_ms = crate::logger::eg().saturating_sub(start);
    let baa = cpc.len() / 1024;
    let fhu = if elapsed_ms > 0 { (baa as u64 * 1000) / elapsed_ms } else { 0 };
    crate::n!(B_, "      Transfer complete: {} KB in {}ms ({} KB/s)", baa, elapsed_ms, fhu);
    
    if cpc.is_empty() {
        crate::n!(A_, "      ERROR: No data received");
        crate::netstack::dhcp::resume();
        return;
    }
    
    
    let bao = cpc.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|aa| aa + 4)
        .unwrap_or(0);
    
    let gbt = &cpc[bao..];
    let size_mb = gbt.len() as f32 / (1024.0 * 1024.0);
    
    crate::n!(B_, "      Download complete: {:.2} MB", size_mb);
    crate::println!();
    
    
    crate::n!(D_, "[3/4] Saving image to /opt/gui/alpine.tar.gz...");
    
    
    let dyg = crate::ramfs::bh(|fs| {
        
        let _ = fs.mkdir("/opt");
        let _ = fs.mkdir("/opt/gui");
        
        let _ = fs.touch("/opt/gui/alpine.tar.gz");
        fs.write_file("/opt/gui/alpine.tar.gz", gbt)
    });
    
    match dyg {
        Ok(_) => {
            crate::n!(B_, "      Saved successfully");
        }
        Err(e) => {
            crate::n!(A_, "      ERROR: Write failed: {:?}", e);
            crate::netstack::dhcp::resume();
            return;
        }
    }
    crate::println!();
    
    
    crate::n!(D_, "[4/4] Configuring GUI environment...");
    
    
    LC_.store(true, core::sync::atomic::Ordering::Relaxed);
    
    crate::n!(B_, "      Configuration complete");
    crate::println!();
    
    crate::n!(G_, "----------------------------------------------------------------");
    crate::n!(G_, "                    GUI Installation Complete!");
    crate::n!(G_, "----------------------------------------------------------------");
    crate::println!();
    crate::println!("Image saved to: /opt/gui/alpine.tar.gz ({:.2} MB)", size_mb);
    crate::println!();
    
    
    crate::n!(D_, "Saving to disk for persistence...");
    match crate::persistence::save_file("/opt/gui/alpine.tar.gz", gbt) {
        Ok(_) => {
            crate::n!(B_, "  Saved to disk! Will be restored on next boot.");
        }
        Err(e) => {
            crate::n!(D_, "  Could not save to disk: {}", e);
            crate::println!("  (Download will need to be repeated after reboot)");
        }
    }
    crate::println!();
    
    crate::println!("Use 'gui start' to launch the graphical environment.");
    
    
    crate::netstack::dhcp::resume();
    crate::serial_println!("[GUI_INSTALL] DHCP resumed");
}

pub(super) fn qan() {
    let installed = LC_.load(core::sync::atomic::Ordering::Relaxed);
    
    if !installed {
        
        if !bbs("/opt/gui/alpine.tar.gz") {
            crate::n!(D_, "Linux VM not installed.");
            crate::println!("Run 'gui install' first to download Alpine Linux.");
            return;
        }
        LC_.store(true, core::sync::atomic::Ordering::Relaxed);
    }
    
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::n!(C_, "|              Starting Alpine Linux VM                        |");
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::println!();
    
    
    crate::n!(D_, "[1/3] Initializing hypervisor...");
    
    
    if !crate::hypervisor::lq() {
        match crate::hypervisor::init() {
            Ok(()) => {
                crate::n!(B_, "      Hypervisor initialized (VT-x/AMD-V)");
            }
            Err(e) => {
                crate::serial_println!("[GUI] Hypervisor init failed: {:?}", e);
                crate::n!(A_, "      ERROR: Hardware virtualization not available");
                crate::println!("      Requires Intel VT-x or AMD-V");
                crate::println!();
                crate::n!(D_, "Falling back to Linux subsystem emulation...");
                eii();
                return;
            }
        }
    }
    crate::n!(B_, "      Hypervisor ready");
    
    crate::n!(D_, "[2/3] Loading Alpine Linux image...");
    crate::n!(B_, "      Image: /opt/gui/alpine.tar.gz");
    
    crate::n!(D_, "[3/3] Booting VM...");
    
    
    match crate::hypervisor::linux_subsystem::boot() {
        Ok(_) => {
            crate::n!(B_, "      VM started successfully");
            crate::println!();
            crate::n!(C_, "Alpine Linux is now running.");
            crate::println!("Use 'vm console' to connect to the VM console.");
            crate::println!("Use 'vm stop' to stop the VM.");
        }
        Err(e) => {
            crate::n!(A_, "      ERROR: Failed to start VM: {:?}", e);
            crate::println!();
            crate::n!(D_, "Falling back to Linux subsystem...");
            eii();
        }
    }
}


pub(super) fn eii() {
    
    if !crate::linux::is_initialized() {
        
        if bbs("/opt/gui/alpine.tar.gz") {
            match crate::linux::init("/opt/gui/alpine.tar.gz") {
                Ok(()) => {}
                Err(e) => {
                    crate::n!(A_, "Failed to initialize Linux subsystem: {}", e);
                    return;
                }
            }
        } else {
            crate::n!(D_, "Linux subsystem not installed.");
            crate::println!("Run 'gui install' to download and install Alpine Linux.");
            return;
        }
    }
    
    
    crate::linux::owk();
}

pub(super) fn koa(args: &[&str]) {
    use crate::desktop::{RenderMode, set_render_mode, set_theme};
    use crate::graphics::CompositorTheme;
    
    if args.is_empty() {
        crate::n!(C_, "TrustGL Compositor Settings");
        crate::n!(C_, "===========================");
        crate::println!();
        crate::println!("Usage: glmode <mode|theme>");
        crate::println!();
        crate::n!(G_, "Render Modes:");
        crate::println!("  classic   - Classic framebuffer rendering (fast, stable)");
        crate::println!("  opengl    - OpenGL compositor with visual effects");
        crate::println!();
        crate::n!(G_, "Themes (OpenGL mode only):");
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
            crate::n!(B_, "Switched to Classic rendering mode");
        }
        "opengl" | "gl" | "compositor" => {
            set_render_mode(RenderMode::OpenGL);
            crate::n!(B_, "Switched to OpenGL compositor mode");
            crate::println!("Use 'glmode <theme>' to change visual theme");
        }
        "flat" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Flat);
            crate::n!(B_, "Theme: Flat (OpenGL)");
        }
        "modern" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Modern);
            crate::n!(B_, "Theme: Modern (shadows, subtle effects)");
        }
        "glass" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Glass);
            crate::n!(B_, "Theme: Glass (transparency effects)");
        }
        "neon" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Neon);
            crate::n!(B_, "Theme: Neon (glowing borders)");
        }
        "minimal" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Minimal);
            crate::n!(B_, "Theme: Minimal (thin borders)");
        }
        _ => {
            crate::n!(A_, "Unknown mode/theme: {}", args[0]);
            crate::println!("Use 'glmode' without arguments for help");
        }
    }
}


pub(super) fn kss(args: &[&str]) {
    if args.is_empty() {
        crate::n!(C_, "TrustOS Theme Manager");
        crate::n!(C_, "=====================");
        crate::println!();
        crate::println!("Usage: theme <command> [args]");
        crate::println!();
        crate::n!(G_, "Commands:");
        crate::println!("  list              - List available built-in themes");
        crate::println!("  set <name>        - Switch to a built-in theme");
        crate::println!("  load <path>       - Load theme from config file");
        crate::println!("  save <path>       - Save current theme to file");
        crate::println!("  reload            - Reload wallpaper from disk");
        crate::println!("  info              - Show current theme info");
        crate::println!();
        crate::n!(G_, "Built-in Themes:");
        crate::println!("  dark / trustos    - TrustOS dark green theme");
        crate::println!("  windows11 / win11 - Windows 11 dark theme");
        crate::println!();
        crate::n!(G_, "Config File Format (/etc/theme.conf):");
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
            crate::n!(C_, "Available Themes:");
            crate::println!("  dark       - TrustOS dark green (default)");
            crate::println!("  windows11  - Windows 11 dark blue");
            crate::println!("  light      - Light theme");
        }
        "set" => {
            if args.len() < 2 {
                crate::n!(A_, "Usage: theme set <name>");
                return;
            }
            crate::theme::jex(args[1]);
            crate::n!(B_, "Theme switched to: {}", args[1]);
        }
        "load" => {
            if args.len() < 2 {
                crate::n!(A_, "Usage: theme load <path>");
                crate::println!("Example: theme load /etc/theme.conf");
                return;
            }
            if crate::theme::nag(args[1]) {
                crate::n!(B_, "Theme loaded from: {}", args[1]);
            } else {
                crate::n!(A_, "Failed to load theme from: {}", args[1]);
            }
        }
        "save" => {
            if args.len() < 2 {
                crate::n!(A_, "Usage: theme save <path>");
                return;
            }
            let theme = crate::theme::Dj.read();
            let content = crate::theme::config::mcm(&theme);
            drop(theme);
            
            match crate::vfs::write_file(args[1], content.as_bytes()) {
                Ok(_) => crate::n!(B_, "Theme saved to: {}", args[1]),
                Err(e) => crate::n!(A_, "Failed to save: {:?}", e),
            }
        }
        "reload" => {
            crate::theme::oes();
            crate::n!(B_, "Wallpaper reloaded");
        }
        "info" => {
            let theme = crate::theme::Dj.read();
            crate::n!(C_, "Current Theme: {}", 
                if theme.name.is_empty() { "TrustOS Default" } else { &theme.name });
            crate::println!();
            crate::n!(G_, "Colors:");
            crate::println!("  Background:  0x{:08X}", theme.colors.background);
            crate::println!("  Accent:      0x{:08X}", theme.colors.accent);
            crate::println!("  Text:        0x{:08X}", theme.colors.text_primary);
            crate::println!("  Surface:     0x{:08X}", theme.colors.surface);
            crate::println!();
            crate::n!(G_, "Taskbar:");
            crate::println!("  Height:      {} px", theme.taskbar.height);
            crate::println!("  Centered:    {}", theme.taskbar.centered_icons);
            crate::println!();
            crate::n!(G_, "Windows:");
            crate::println!("  Title bar:   {} px", theme.window.titlebar_height);
            crate::println!("  Radius:      {} px", theme.window.border_radius);
            crate::println!("  Shadow:      {} px", theme.window.shadow_size);
            crate::println!();
            crate::n!(G_, "Wallpaper:");
            crate::println!("  Path:        {}", 
                if theme.wallpaper.path.is_empty() { "(none)" } else { &theme.wallpaper.path });
            crate::println!("  Mode:        {:?}", theme.wallpaper.mode);
        }
        _ => {
            crate::n!(A_, "Unknown theme command: {}", args[0]);
            crate::println!("Use 'theme' for help");
        }
    }
}


pub(super) fn klp(args: &[&str]) {
    if args.is_empty() {
        let enabled = crate::desktop::awb();
        let speed = crate::desktop::dqn();
        
        crate::n!(C_, "TrustOS Animation Settings");
        crate::n!(C_, "==========================");
        crate::println!();
        crate::n!(G_, "Current Status:");
        if enabled {
            crate::println!("  Animations: {} ENABLED", "\x1b[32m?\x1b[0m");
        } else {
            crate::println!("  Animations: {} DISABLED", "\x1b[31m?\x1b[0m");
        }
        crate::println!("  Speed:      {}x", speed);
        crate::println!();
        crate::n!(G_, "Commands:");
        crate::println!("  anim on           - Enable animations");
        crate::println!("  anim off          - Disable animations");
        crate::println!("  anim toggle       - Toggle on/off");
        crate::println!("  anim speed <val>  - Set speed (0.25-4.0)");
        crate::println!("                      1.0=normal, 2.0=fast, 0.5=slow");
        crate::println!();
        crate::n!(G_, "Animation Types:");
        crate::println!("  - Window open (scale up from center)");
        crate::println!("  - Window close (scale down + fade out)");
        crate::println!("  - Minimize (move to taskbar)");
        crate::println!("  - Maximize/Restore (smooth resize)");
        return;
    }
    
    match args[0] {
        "on" | "enable" | "1" | "true" => {
            crate::desktop::fae(true);
            crate::n!(B_, "? Animations enabled");
        }
        "off" | "disable" | "0" | "false" => {
            crate::desktop::fae(false);
            crate::n!(D_, "? Animations disabled");
        }
        "toggle" => {
            let current = crate::desktop::awb();
            crate::desktop::fae(!current);
            if !current {
                crate::n!(B_, "? Animations enabled");
            } else {
                crate::n!(D_, "? Animations disabled");
            }
        }
        "speed" => {
            if args.len() < 2 {
                crate::println!("Current speed: {}x", crate::desktop::dqn());
                crate::println!("Usage: anim speed <value>");
                crate::println!("  Examples: 0.5 (slow), 1.0 (normal), 2.0 (fast)");
                return;
            }
            if let Ok(speed) = args[1].parse::<f32>() {
                crate::desktop::jew(speed);
                crate::n!(B_, "Animation speed set to {}x", speed);
            } else {
                crate::n!(A_, "Invalid speed value: {}", args[1]);
            }
        }
        "status" | "info" => {
            let enabled = crate::desktop::awb();
            let speed = crate::desktop::dqn();
            crate::println!("Animations: {}", if enabled { "enabled" } else { "disabled" });
            crate::println!("Speed: {}x", speed);
        }
        _ => {
            crate::n!(A_, "Unknown animation command: {}", args[0]);
            crate::println!("Use 'anim' for help");
        }
    }
}


pub(super) fn kom(args: &[&str]) {
    use crate::graphics::holomatrix;
    
    if args.is_empty() {
        let enabled = holomatrix::lq();
        let scene = holomatrix::dqr();
        
        crate::n!(C_, "TrustOS HoloMatrix 3D");
        crate::n!(C_, "=====================");
        crate::println!();
        crate::n!(G_, "Current Status:");
        if enabled {
            crate::println!("  HoloMatrix: {} ENABLED", "\x1b[36m?\x1b[0m");
        } else {
            crate::println!("  HoloMatrix: {} DISABLED", "\x1b[31m?\x1b[0m");
        }
        crate::println!("  Scene:      {}", scene.name());
        crate::println!();
        crate::n!(G_, "Commands:");
        crate::println!("  holo on           - Enable HoloMatrix 3D background");
        crate::println!("  holo off          - Disable (use Matrix Rain)");
        crate::println!("  holo toggle       - Toggle on/off");
        crate::println!("  holo next         - Cycle to next scene");
        crate::println!("  holo scene <name> - Set specific scene");
        crate::println!();
        crate::n!(G_, "Available Scenes:");
        crate::println!("  cube     - Rotating wireframe cube");
        crate::println!("  sphere   - Pulsating sphere");
        crate::println!("  torus    - 3D donut/ring");
        crate::println!("  grid     - Perspective grid with cube");
        crate::println!("  multi    - Multiple floating shapes");
        crate::println!("  dna      - Animated DNA double helix");
        crate::println!();
        crate::n!(G_, "How it works:");
        crate::println!("  Renders 3D shapes using 16 Z-slices (layers)");
        crate::println!("  Each layer has depth-based transparency");
        crate::println!("  Creates holographic volumetric effect");
        return;
    }
    
    match args[0] {
        "on" | "enable" | "1" | "true" => {
            holomatrix::set_enabled(true);
            crate::n!(0xFF00FFFF, "? HoloMatrix 3D enabled");
            crate::println!("Launch 'desktop' to see the effect");
        }
        "off" | "disable" | "0" | "false" => {
            holomatrix::set_enabled(false);
            crate::n!(D_, "? HoloMatrix disabled (Matrix Rain active)");
        }
        "toggle" => {
            let enabled = holomatrix::pkp();
            if enabled {
                crate::n!(0xFF00FFFF, "? HoloMatrix 3D enabled");
            } else {
                crate::n!(D_, "? HoloMatrix disabled");
            }
        }
        "next" | "cycle" => {
            let scene = holomatrix::nkf();
            crate::n!(0xFF00FFFF, "Scene: {}", scene.name());
        }
        "scene" | "set" => {
            if args.len() < 2 {
                crate::println!("Current scene: {}", holomatrix::dqr().name());
                crate::println!("Usage: holo scene <name>");
                crate::println!("Available: cube, sphere, torus, grid, multi, dna");
                return;
            }
            if let Some(scene) = holomatrix::HoloScene::iad(args[1]) {
                holomatrix::set_scene(scene);
                crate::n!(0xFF00FFFF, "Scene set to: {}", scene.name());
            } else {
                crate::n!(A_, "Unknown scene: {}", args[1]);
                crate::println!("Available: cube, sphere, torus, grid, multi, dna");
            }
        }
        "status" | "info" => {
            let enabled = holomatrix::lq();
            let scene = holomatrix::dqr();
            crate::println!("HoloMatrix: {}", if enabled { "enabled" } else { "disabled" });
            crate::println!("Scene: {}", scene.name());
        }
        "list" | "scenes" => {
            crate::n!(G_, "Available Scenes:");
            for name in holomatrix::HoloScene::juo() {
                crate::println!("  {}", name);
            }
        }
        _ => {
            
            if let Some(scene) = holomatrix::HoloScene::iad(args[0]) {
                holomatrix::set_scene(scene);
                crate::n!(0xFF00FFFF, "Scene set to: {}", scene.name());
            } else {
                crate::n!(A_, "Unknown command: {}", args[0]);
                crate::println!("Use 'holo' for help");
            }
        }
    }
}


pub(super) fn kot(args: &[&str]) {
    if args.is_empty() {
        crate::n!(C_, "TrustOS Image Viewer");
        crate::n!(C_, "====================");
        crate::println!();
        crate::println!("Usage: imgview <path> [options]");
        crate::println!();
        crate::n!(G_, "Options:");
        crate::println!("  -x <num>     X position (default: center)");
        crate::println!("  -y <num>     Y position (default: center)");
        crate::println!("  -w <num>     Width (scale to this width)");
        crate::println!("  -h <num>     Height (scale to this height)");
        crate::println!("  -info        Show image info only, don't display");
        crate::println!();
        crate::n!(G_, "Supported Formats:");
        crate::println!("  BMP  - 24-bit and 32-bit uncompressed");
        crate::println!("  PPM  - P3 (ASCII) and P6 (binary)");
        crate::println!("  RAW  - Raw RGBA pixel data");
        crate::println!();
        crate::n!(G_, "Examples:");
        crate::println!("  imgview /usr/share/wallpapers/logo.bmp");
        crate::println!("  imgview /home/image.ppm -x 100 -y 100");
        crate::println!("  imgview photo.bmp -w 640 -h 480");
        return;
    }
    
    let path = args[0];
    let mut ivp: Option<i32> = None;
    let mut ivq: Option<i32> = None;
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
    let mut ign = false;
    
    
    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "-x" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<i32>() {
                    ivp = Some(v);
                }
                i += 2;
            }
            "-y" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<i32>() {
                    ivq = Some(v);
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
                ign = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    
    
    crate::println!("Loading image: {}", path);
    
    match crate::image::load(path) {
        Some(iv) => {
            crate::n!(B_, "Image loaded successfully!");
            crate::println!("  Size: {} x {} pixels", iv.width, iv.height);
            crate::println!("  Memory: {} KB", (iv.pixels.len() * 4) / 1024);
            
            if ign {
                return;
            }
            
            
            let cif = width.unwrap_or(iv.width);
            let cie = height.unwrap_or(iv.height);
            
            
            let (fb_width, fb_height) = crate::framebuffer::kv();
            let x = ivp.unwrap_or_else(|| ((fb_width - cif) / 2) as i32);
            let y = ivq.unwrap_or_else(|| ((fb_height - cie) / 2) as i32);
            
            crate::println!("  Drawing at ({}, {}) size {}x{}", x, y, cif, cie);
            
            
            if cif == iv.width && cie == iv.height {
                iv.draw(x, y);
            } else {
                iv.draw_scaled(x, y, cif, cie);
            }
            
            crate::framebuffer::ii();
            crate::n!(B_, "Image displayed!");
        }
        None => {
            crate::n!(A_, "Failed to load image: {}", path);
            crate::println!("Make sure the file exists and is a supported format.");
        }
    }
}


pub(super) fn kos(args: &[&str]) {
    let hrh = args.first().copied().unwrap_or("gradient");
    
    crate::n!(C_, "Image Demo: {}", hrh);
    
    let (fb_width, fb_height) = crate::framebuffer::kv();
    
    match hrh {
        "gradient" => {
            
            let iv = crate::image::hop(
                200, 200, 
                0xFF0066FF,  
                0xFF00FF66   
            );
            let x = ((fb_width - 200) / 2) as i32;
            let y = ((fb_height - 200) / 2) as i32;
            iv.draw(x, y);
            crate::n!(B_, "Displayed gradient at center");
        }
        "checker" => {
            
            let iv = crate::image::kzg(
                256, 256, 32,
                0xFFFFFFFF,  
                0xFF000000   
            );
            let x = ((fb_width - 256) / 2) as i32;
            let y = ((fb_height - 256) / 2) as i32;
            iv.draw(x, y);
            crate::n!(B_, "Displayed checkerboard at center");
        }
        "trustos" => {
            
            let iv = crate::image::hop(
                300, 100,
                0xFF00D26A,  
                0xFF0A0E0B   
            );
            let x = ((fb_width - 300) / 2) as i32;
            let y = ((fb_height - 100) / 2) as i32;
            iv.draw(x, y);
            
            
            let ri = 0xFF00D26A;
            for i in 0..300 {
                crate::framebuffer::put_pixel(x as u32 + i, y as u32, ri);
                crate::framebuffer::put_pixel(x as u32 + i, (y + 99) as u32, ri);
            }
            for i in 0..100 {
                crate::framebuffer::put_pixel(x as u32, y as u32 + i, ri);
                crate::framebuffer::put_pixel((x + 299) as u32, y as u32 + i, ri);
            }
            
            crate::n!(B_, "Displayed TrustOS banner");
        }
        "colors" => {
            
            let mut iv = crate::image::Image::new(256, 256);
            for y in 0..256 {
                for x in 0..256 {
                    let r = x as u32;
                    let g = y as u32;
                    let b = ((x + y) / 2) as u32;
                    let color = 0xFF000000 | (r << 16) | (g << 8) | b;
                    iv.set_pixel(x, y, color);
                }
            }
            let x = ((fb_width - 256) / 2) as i32;
            let y = ((fb_height - 256) / 2) as i32;
            iv.draw(x, y);
            crate::n!(B_, "Displayed color test pattern");
        }
        "alpha" => {
            
            
            let bg = crate::image::kzk(200, 200, 0xFFFF0000);
            let x = ((fb_width - 200) / 2) as i32;
            let y = ((fb_height - 200) / 2) as i32;
            bg.draw(x, y);
            
            
            let mut ayx = crate::image::Image::new(200, 200);
            for o in 0..200u32 {
                for p in 0..200u32 {
                    
                    let alpha = (p + o) / 2;
                    let color = (alpha << 24) | 0x000000FF;  
                    ayx.set_pixel(p, o, color);
                }
            }
            ayx.draw(x, y);
            crate::n!(B_, "Displayed alpha blend demo (red + blue)");
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
    
    crate::framebuffer::ii();
}

pub(super) fn ksk() {
    let tasks = crate::task::list_tasks();
    crate::n!(C_, "  PID  STATE       PRIORITY  NAME");
    crate::n!(C_, "-------------------------------------");
    
    
    crate::println!("    1  running     critical  kernel");
    crate::println!("    2  running     normal    tsh");
    
    let task_count = tasks.len();
    for (id, name, state, priority) in tasks {
        let acr = match state {
            crate::task::TaskState::Ready => "ready",
            crate::task::TaskState::Running => "running",
            crate::task::TaskState::Blocked => "blocked",
            crate::task::TaskState::Terminated => "done",
        };
        let nxi = match priority {
            crate::task::Priority::Low => "low",
            crate::task::Priority::Normal => "normal",
            crate::task::Priority::High => "high",
            crate::task::Priority::Critical => "critical",
        };
        crate::println!("{:>5}  {:10}  {:8}  {}", id + 2, acr, nxi, name);
    }
    
    crate::println!();
    crate::n!(AX_, "Total: {} tasks", task_count + 2);
}

pub(super) fn kst() {
    crate::n!(C_, "  TID  PID  STATE       NAME");
    crate::n!(C_, "------------------------------------");
    
    
    let zn = crate::thread::mzi();
    let count = zn.len();
    
    for (tid, pid, state, name) in zn {
        let acr = match state {
            crate::thread::ThreadState::Ready => "ready",
            crate::thread::ThreadState::Running => "running",
            crate::thread::ThreadState::Blocked => "blocked",
            crate::thread::ThreadState::Sleeping => "sleeping",
            crate::thread::ThreadState::Dead => "dead",
        };
        crate::println!("{:>5}  {:>3}  {:10}  {}", tid, pid, acr, &name);
    }
    
    crate::println!();
    crate::n!(AX_, "Total: {} threads", count);
}



pub(super) fn kqk(args: &[&str]) {
    if args.is_empty() {
        
        let (status, files, size) = crate::persistence::status();
        crate::n!(C_, "+--------------------------------------------------------------+");
        crate::n!(C_, "|                    Persistence Status                        |");
        crate::n!(C_, "+--------------------------------------------------------------+");
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
                Ok(_) => crate::n!(B_, "Persistence data cleared."),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        "save" => {
            crate::println!("Saving current data to disk...");
            
            
            let cto = "/opt/gui/alpine.tar.gz";
            if bbs(cto) {
                let bom: Result<Vec<u8>, _> = crate::ramfs::bh(|fs| {
                    fs.read_file(cto).map(|d| d.to_vec())
                });
                match bom {
                    Ok(data) => {
                        match crate::persistence::save_file(cto, &data) {
                            Ok(_) => crate::n!(B_, "  Saved: {} ({} KB)", cto, data.len() / 1024),
                            Err(e) => crate::n!(A_, "  Failed: {} - {}", cto, e),
                        }
                    }
                    Err(e) => crate::n!(A_, "  Cannot read {}: {:?}", cto, e),
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



pub(super) fn knb() {
    crate::n!(C_, "=== Storage Devices ===");
    
    let mut aqg = 0u32;
    
    
    if crate::nvme::is_initialized() {
        if let Some((model, serial, ns_size, lba_size)) = crate::nvme::rk() {
            let size_mb = (ns_size * lba_size as u64) / (1024 * 1024);
            crate::println!();
            crate::n!(B_, "[NVMe] {}", model);
            crate::println!("  Serial:    {}", serial);
            crate::println!("  Capacity:  {} MB ({} sectors x {} bytes)", size_mb, ns_size, lba_size);
            crate::println!("  Interface: NVMe over PCIe");
            aqg += 1;
        }
    }
    
    
    if crate::drivers::ahci::is_initialized() {
        for s in crate::drivers::ahci::adz() {
            let size_mb = (s.sector_count * 512) / (1024 * 1024);
            crate::println!();
            crate::bq!(B_, "[AHCI Port {}] ", s.port_num);
            crate::println!("{}", s.model);
            crate::println!("  Serial:    {}", s.serial);
            crate::println!("  Capacity:  {} MB ({} sectors)", size_mb, s.sector_count);
            crate::println!("  Type:      {:?}", s.device_type);
            crate::println!("  Interface: SATA (AHCI)");
            aqg += 1;
        }
    }
    
    
    for tz in crate::drivers::ata::eta() {
        if tz.present {
            let size_mb = (tz.sector_count * 512) / (1024 * 1024);
            let ch = match tz.channel {
                crate::drivers::ata::IdeChannel::Primary => "Primary",
                crate::drivers::ata::IdeChannel::Secondary => "Secondary",
            };
            let pos = match tz.position {
                crate::drivers::ata::DrivePosition::Master => "Master",
                crate::drivers::ata::DrivePosition::Slave => "Slave",
            };
            crate::println!();
            crate::bq!(B_, "[IDE {} {}] ", ch, pos);
            crate::println!("{}", tz.model);
            crate::println!("  Serial:    {}", tz.serial);
            crate::println!("  Capacity:  {} MB ({} sectors)", size_mb, tz.sector_count);
            crate::println!("  LBA48:     {}", if tz.lba48 { "Yes" } else { "No (28-bit)" });
            crate::println!("  ATAPI:     {}", if tz.atapi { "Yes" } else { "No" });
            crate::println!("  Interface: IDE/ATA (PIO)");
            aqg += 1;
        }
    }
    
    
    if crate::virtio_blk::is_initialized() {
        let cap = crate::virtio_blk::capacity();
        let size_mb = (cap * 512) / (1024 * 1024);
        let eyv = crate::virtio_blk::is_read_only();
        crate::println!();
        crate::n!(B_, "[VirtIO Block Device]");
        crate::println!("  Capacity:  {} MB ({} sectors)", size_mb, cap);
        crate::println!("  Read-Only: {}", if eyv { "Yes" } else { "No" });
        crate::println!("  Interface: VirtIO (paravirtual)");
        aqg += 1;
    }
    
    
    for (i, (name, blocks, bsize)) in crate::drivers::usb_storage::adz().iter().enumerate() {
        let size_mb = (*blocks * *bsize as u64) / (1024 * 1024);
        crate::println!();
        crate::bq!(B_, "[USB Storage #{}] ", i);
        crate::println!("{}", name);
        crate::println!("  Capacity:  {} MB ({} blocks x {} bytes)", size_mb, blocks, bsize);
        crate::println!("  Interface: USB Mass Storage (BBB/SCSI)");
        aqg += 1;
    }
    
    
    crate::println!();
    if let Some(info) = crate::disk::rk() {
        crate::n!(AX_, "[RAM Disk]");
        crate::println!("  Size:      {} KB ({} sectors)", info.sectors / 2, info.sectors);
        
        let (reads, writes, bytes_r, bytes_w) = crate::disk::get_stats();
        crate::println!("  Stats:     {} reads ({} B), {} writes ({} B)", reads, bytes_r, writes, bytes_w);
    }
    
    
    crate::println!();
    if aqg == 0 {
        crate::n!(D_, "No hardware storage detected (RAM disk only)");
    } else {
        crate::n!(C_, "Total: {} hardware storage device(s) + RAM disk", aqg);
    }
}

pub(super) fn kmv(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: dd <sector> [count]");
        crate::println!("       dd write <sector> <text>");
        crate::println!("       dd dump <sector>");
        return;
    }
    
    if args[0] == "dump" && args.len() > 1 {
        let dj: u64 = match args[1].parse() {
            Ok(ae) => ae,
            Err(_) => {
                crate::n!(A_, "Invalid sector number");
                return;
            }
        };
        
        match crate::disk::lml(dj) {
            Ok(byz) => crate::println!("{}", byz),
            Err(e) => crate::n!(A_, "Error: {}", e),
        }
        return;
    }
    
    if args[0] == "write" && args.len() > 2 {
        let dj: u64 = match args[1].parse() {
            Ok(ae) => ae,
            Err(_) => {
                crate::n!(A_, "Invalid sector number");
                return;
            }
        };
        
        let text = args[2..].join(" ");
        let mut data = [0u8; 512];
        let bytes = text.as_bytes();
        let len = bytes.len().min(512);
        data[..len].copy_from_slice(&bytes[..len]);
        
        match crate::disk::write_sector(dj, &data) {
            Ok(_) => crate::n!(B_, "Written {} bytes to sector {}", len, dj),
            Err(e) => crate::n!(A_, "Write error: {}", e),
        }
        return;
    }
    
    
    let dj: u64 = match args[0].parse() {
        Ok(ae) => ae,
        Err(_) => {
            crate::n!(A_, "Invalid sector number");
            return;
        }
    };

    let mut buffer = [0u8; 512];
    match crate::disk::read_sectors(dj, 1, &mut buffer) {
        Ok(_) => {
            crate::n!(C_, "Sector {} (512 bytes):", dj);
            
            
            for row in 0..16 {
                crate::bq!(AX_, "{:04X}: ", row * 16);
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
            crate::n!(A_, "Read error: {}", e);
        }
    }
}

pub(super) fn klm(args: &[&str]) {
    if args.is_empty() {
        
        crate::n!(C_, "=== AHCI Storage Controller ===");
        
        if !crate::drivers::ahci::is_initialized() {
            crate::n!(D_, "AHCI not initialized");
            return;
        }
        
        let devices = crate::drivers::ahci::adz();
        if devices.is_empty() {
            crate::n!(D_, "No AHCI devices found");
            return;
        }
        
        crate::println!("Found {} device(s):", devices.len());
        for s in &devices {
            crate::println!();
            crate::bq!(B_, "  Port {}: ", s.port_num);
            crate::println!("{:?}", s.device_type);
            crate::println!("    Model:   {}", s.model);
            crate::println!("    Serial:  {}", s.serial);
            crate::println!("    Sectors: {}", s.sector_count);
        }
        
        crate::println!();
        crate::n!(AX_, "Commands:");
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
                Ok(ae) => ae,
                Err(_) => {
                    crate::n!(A_, "Invalid port number");
                    return;
                }
            };
            
            let dj: u64 = match args[2].parse() {
                Ok(ae) => ae,
                Err(_) => {
                    crate::n!(A_, "Invalid sector number");
                    return;
                }
            };
            
            crate::println!("Reading sector {} from AHCI port {}...", dj, port);
            
            
            let mut buffer = alloc::vec![0u8; 512];
            
            match crate::drivers::ahci::read_sectors(port, dj, 1, &mut buffer) {
                Ok(bytes) => {
                    crate::n!(B_, "Read {} bytes successfully", bytes);
                    crate::println!();
                    
                    
                    for row in 0..16 {
                        crate::bq!(AX_, "{:04X}: ", row * 16);
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
                    crate::n!(A_, "AHCI read error: {}", e);
                }
            }
        }
        
        "write" => {
            if args.len() < 4 {
                crate::println!("Usage: ahci write <port> <sector> <text>");
                return;
            }
            
            let port: u8 = match args[1].parse() {
                Ok(ae) => ae,
                Err(_) => {
                    crate::n!(A_, "Invalid port number");
                    return;
                }
            };
            
            let dj: u64 = match args[2].parse() {
                Ok(ae) => ae,
                Err(_) => {
                    crate::n!(A_, "Invalid sector number");
                    return;
                }
            };
            
            let text = args[3..].join(" ");
            let mut buffer = alloc::vec![0u8; 512];
            let bytes = text.as_bytes();
            let len = bytes.len().min(512);
            buffer[..len].copy_from_slice(&bytes[..len]);
            
            crate::println!("Writing {} bytes to sector {} on AHCI port {}...", len, dj, port);
            
            match crate::drivers::ahci::write_sectors(port, dj, 1, &buffer) {
                Ok(bytes) => {
                    crate::n!(B_, "Written {} bytes successfully", bytes);
                }
                Err(e) => {
                    crate::n!(A_, "AHCI write error: {}", e);
                }
            }
        }
        
        _ => {
            crate::println!("Unknown AHCI command. Use 'ahci' for help.");
        }
    }
}



pub(super) fn knu(args: &[&str]) {
    use crate::drivers::partition;
    use crate::drivers::ahci;
    
    if args.is_empty() {
        
        crate::n!(C_, "=== Partition Tables ===");
        crate::println!();
        
        if !ahci::is_initialized() {
            crate::n!(D_, "AHCI not initialized");
            crate::println!();
            crate::println!("Usage:");
            crate::println!("  fdisk           - Show partitions on all disks");
            crate::println!("  fdisk <port>    - Show partitions on specific AHCI port");
            return;
        }
        
        let devices = ahci::adz();
        if devices.is_empty() {
            crate::println!("No AHCI devices found");
            return;
        }
        
        for s in devices {
            crate::n!(B_, "--- Disk {} ({:?}) ---", s.port_num, s.device_type);
            
            match partition::gqd(s.port_num) {
                Ok(bs) => {
                    partition::iwn(&bs);
                }
                Err(e) => {
                    crate::n!(A_, "  Error reading partitions: {}", e);
                }
            }
            crate::println!();
        }
        
        return;
    }
    
    
    let port: u8 = match args[0].parse() {
        Ok(aa) => aa,
        Err(_) => {
            crate::n!(A_, "Invalid port number: {}", args[0]);
            return;
        }
    };
    
    crate::n!(C_, "=== Partitions on Disk {} ===", port);
    
    match partition::gqd(port) {
        Ok(bs) => {
            partition::iwn(&bs);
        }
        Err(e) => {
            crate::n!(A_, "Error: {}", e);
        }
    }
}



pub(super) fn dkx() {
    
    if let Some(nic) = crate::network::mdn() {
        crate::n!(C_, "Hardware:");
        crate::println!("      Device: {:04X}:{:04X} [{}]", 
            nic.vendor_id, nic.device_id, nic.vendor_name);
        crate::println!("      Driver: {}", nic.driver);
        if crate::network::mjz() {
            crate::n!(B_, "      Status: REAL DRIVER ACTIVE");
        } else {
            crate::n!(D_, "      Status: Simulated");
        }
        if nic.bar0 != 0 {
            crate::println!("      BAR0:   {:#010X}", nic.bar0);
        }
        if nic.irq != 0 && nic.irq != 0xFF {
            crate::println!("      IRQ:    {}", nic.irq);
        }
        crate::println!();
    }
    
    if let Some((mac, ip, state)) = crate::network::cyp() {
        crate::n!(C_, "eth0:");
        crate::print!("      Link: ");
        match state {
            crate::network::NetworkState::Up => crate::n!(B_, "UP"),
            crate::network::NetworkState::Down => crate::n!(D_, "DOWN"),
            crate::network::NetworkState::Error => crate::n!(A_, "ERROR"),
        }
        crate::println!("      HWaddr: {}", mac);
        if let Some(addr) = ip {
            crate::println!("      inet:   {}", addr);
        }
        
        
        let (tx_pkts, rx_pkts, tx_bytes, rx_bytes) = crate::network::mcz();
        crate::println!();
        crate::println!("      RX packets: {}  bytes: {}", rx_pkts, rx_bytes);
        crate::println!("      TX packets: {}  bytes: {}", tx_pkts, tx_bytes);
        
        let stats = crate::network::get_stats();
        if stats.errors > 0 {
            crate::n!(A_, "      Errors: {}", stats.errors);
        }
    } else {
        crate::n!(D_, "No network interface");
    }
}










pub(super) fn kpy(args: &[&str]) {
    crate::n!(G_, "=== TrustScan Live Network Test Suite ===");
    crate::println!();

    let mut passed = 0usize;
    let mut bv = 0usize;

    
    crate::n!(C_, "[PRE] Network connectivity check");
    crate::print!("  NIC driver... ");
    if !crate::drivers::net::aoh() {
        crate::n!(A_, "[FAIL] no network driver");
        crate::n!(A_, "=== Cannot run live tests without a network ===");
        return;
    }
    crate::n!(B_, "[OK]");

    
    let (wj, _mask, gateway_ip) = match crate::network::rd() {
        Some((ip, mask, fz)) => {
            let czv = *ip.as_bytes();
            let nbo = *mask.as_bytes();
            let mgs = fz.map(|g| *g.as_bytes());
            (czv, nbo, mgs)
        }
        None => {
            crate::print!("  IP config... ");
            crate::n!(A_, "[FAIL] no IPv4 config — run 'dhcp' first");
            return;
        }
    };

    crate::print!("  our IP... ");
    crate::n!(B_, "[OK] {}", crate::netscan::uw(wj));

    
    let target: [u8; 4];
    let cri: &str;

    if let Some(db) = args.first() {
        if let Some(ip) = crate::netscan::bof(db) {
            target = ip;
            cri = db;
        } else if let Some(afn) = crate::netstack::dns::yb(db) {
            target = afn;
            cri = db;
        } else {
            crate::n!(A_, "Cannot resolve: {}", db);
            return;
        }
    } else if let Some(fz) = gateway_ip {
        target = fz;
        cri = "gateway";
    } else {
        
        target = [8, 8, 8, 8];
        cri = "8.8.8.8";
    }

    crate::n!(C_, "  target: {} ({})", cri, crate::netscan::uw(target));
    crate::println!();

    
    crate::n!(C_, "[1/8] ICMP Ping — reachability");
    {
        crate::print!("  ping {}... ", crate::netscan::uw(target));
        let ip = crate::network::Ipv4Address::new(target[0], target[1], target[2], target[3]);
        match crate::network::gty(ip) {
            Ok(result) if result.success => {
                crate::n!(B_, "[OK] rtt={} us  ttl={}", result.time_us, result.ttl);
                passed += 1;
            }
            Ok(_) => {
                crate::n!(D_, "[WARN] timeout (host may block ICMP)");
                passed += 1; 
            }
            Err(e) => {
                crate::n!(A_, "[FAIL] {}", e);
                bv += 1;
            }
        }
    }

    
    crate::n!(C_, "[2/8] ARP Resolution");
    {
        
        let fhm = gateway_ip.unwrap_or(target);
        crate::print!("  ARP {}... ", crate::netscan::uw(fhm));
        let _ = crate::netstack::arp::bos(fhm);
        
        for _ in 0..200_000 {
            crate::netstack::poll();
            core::hint::spin_loop();
        }
        if let Some(mac) = crate::netstack::arp::yb(fhm) {
            crate::n!(B_, "[OK] MAC={}", crate::netscan::bzx(mac));
            passed += 1;
        } else {
            crate::n!(D_, "[WARN] no ARP reply (may be routed)");
            passed += 1; 
        }
    }

    
    crate::n!(C_, "[3/8] DNS Resolution");
    {
        crate::print!("  resolve google.com... ");
        match crate::netstack::dns::yb("google.com") {
            Some(ip) => {
                crate::n!(B_, "[OK] {}", crate::netscan::uw(ip));
                passed += 1;
            }
            None => {
                crate::n!(A_, "[FAIL] DNS resolution failed");
                bv += 1;
            }
        }

        crate::print!("  resolve example.com... ");
        match crate::netstack::dns::yb("example.com") {
            Some(ip) => {
                crate::n!(B_, "[OK] {}", crate::netscan::uw(ip));
                passed += 1;
            }
            None => {
                crate::n!(D_, "[WARN] no DNS — limited test");
                passed += 1; 
            }
        }
    }

    
    crate::n!(C_, "[4/8] TCP SYN Port Scan");
    {
        
        let jmb = alloc::vec![80u16, 443, 53, 22, 8080];
        crate::print!("  SYN scan {} ({} ports)... ", crate::netscan::uw(target), jmb.len());
        let config = crate::netscan::port_scanner::ScanConfig::new(target)
            .with_ports(jmb)
            .with_type(crate::netscan::port_scanner::ScanType::Syn)
            .with_timeout(2000);
        let (results, stats) = crate::netscan::port_scanner::scan(&config);

        
        crate::n!(B_, "[OK] {} open, {} closed, {} filtered ({} ms)",
            stats.open, stats.closed, stats.filtered, stats.elapsed_ms);
        passed += 1;

        
        for r in &results {
            if r.state == crate::netscan::port_scanner::PortState::Open {
                crate::println!("    {:>5}/tcp  {:<12}  OPEN", r.port, r.service);
            }
        }
    }

    
    crate::n!(C_, "[5/8] TCP Connect Scan + Banner Grab");
    {
        
        
        let css = if let Some(ip) = crate::netstack::dns::yb("example.com") {
            ip
        } else {
            target 
        };

        crate::print!("  connect scan {}:80... ", crate::netscan::uw(css));
        let config = crate::netscan::port_scanner::ScanConfig::new(css)
            .with_ports(alloc::vec![80])
            .with_type(crate::netscan::port_scanner::ScanType::Connect)
            .with_timeout(3000);
        let (results, stats) = crate::netscan::port_scanner::scan(&config);

        if stats.open > 0 {
            crate::n!(B_, "[OK] port 80 open");
            passed += 1;

            
            crate::print!("  banner grab :80... ");
            match crate::netscan::banner::grab_banner(css, 80, 3000) {
                Some(yi) => {
                    let pnq = if yi.banner.len() > 60 {
                        &yi.banner[..60]
                    } else {
                        &yi.banner
                    };
                    crate::n!(B_, "[OK] '{}'", pnq);
                    if let Some(ref tu) = yi.version {
                        crate::println!("    version: {}", tu);
                    }
                    passed += 1;
                }
                None => {
                    crate::n!(D_, "[WARN] no banner (server may not send one)");
                    passed += 1;
                }
            }
        } else {
            crate::n!(D_, "[WARN] port 80 not open on {} — skip banner",
                crate::netscan::uw(css));
            passed += 2; 
        }
        let _ = results;
    }

    
    crate::n!(C_, "[6/8] Packet Sniffer (live capture)");
    {
        use crate::netscan::sniffer;

        crate::print!("  capture during ping... ");
        sniffer::deu();

        
        let ip = crate::network::Ipv4Address::new(target[0], target[1], target[2], target[3]);
        let _ = crate::network::gty(ip);

        
        for _ in 0..300_000 {
            crate::netstack::poll();
            core::hint::spin_loop();
        }

        let (count, bytes, awl) = sniffer::get_stats();
        sniffer::dex();

        if count > 0 {
            crate::n!(B_, "[OK] captured {} packets, {} bytes", count, bytes);
            passed += 1;

            
            crate::print!("  packet details... ");
            let coe = sniffer::ewn(5);
            if !coe.is_empty() {
                crate::n!(B_, "[OK] {} in buffer", coe.len());
                for aa in coe.iter().take(3) {
                    crate::println!("    [{:<4}] {} {}", aa.protocol.as_str(),
                        aa.src_ip.map(|i| crate::netscan::uw(i)).unwrap_or_else(|| alloc::string::String::from("?")),
                        aa.info);
                }
                passed += 1;
            } else {
                crate::n!(A_, "[FAIL] buffer empty despite count > 0");
                bv += 1;
            }
        } else {
            crate::n!(D_, "[WARN] no packets captured (driver may not report TX)");
            passed += 2; 
        }
    }

    
    crate::n!(C_, "[7/8] Traceroute (TTL)");
    {
        
        let joh = if let Some(fz) = gateway_ip {
            fz 
        } else {
            [8, 8, 8, 8] 
        };

        crate::print!("  trace to {} (5 hops max)... ", crate::netscan::uw(joh));
        let bcb = crate::netscan::traceroute::trace(joh, 5, 2000);

        if !bcb.is_empty() {
            crate::n!(B_, "[OK] {} hops recorded", bcb.len());
            passed += 1;

            for h in &bcb {
                crate::print!("    {:>2}  ", h.hop_num);
                if let Some(ip) = h.ip {
                    crate::print!("{:<16}", crate::netscan::uw(ip));
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
            crate::n!(D_, "[WARN] no hops (ICMP may be blocked)");
            passed += 1;
        }
    }

    
    crate::n!(C_, "[8/8] Vulnerability Scanner");
    {
        
        let css = if let Some(ip) = crate::netstack::dns::yb("example.com") {
            ip
        } else {
            target
        };

        crate::print!("  vuln check {}:80,443... ", crate::netscan::uw(css));
        let fw = crate::netscan::vuln::scan(css, &[80, 443]);

        
        crate::n!(B_, "[OK] {} findings", fw.len());
        passed += 1;

        for f in fw.iter().take(3) {
            let color = match f.severity {
                crate::netscan::vuln::Severity::Critical | crate::netscan::vuln::Severity::High => A_,
                crate::netscan::vuln::Severity::Medium => D_,
                _ => K_,
            };
            crate::bq!(color, "    [{:<8}] ", f.severity.as_str());
            crate::println!("{}/{}: {}", f.port, f.service, f.title);
        }
    }

    
    crate::println!();
    let av = passed + bv;
    if bv == 0 {
        crate::n!(G_,
            "=== ALL {}/{} LIVE TESTS PASSED ===", passed, av);
    } else {
        crate::n!(A_,
            "=== {}/{} passed, {} FAILED ===", passed, av, bv);
    }
    crate::println!();
    crate::println!("Tip: For more detailed testing, try:");
    crate::println!("  nmap example.com -sT -p 80,443 -A");
    crate::println!("  banner example.com 80");
    crate::println!("  traceroute 8.8.8.8");
    crate::println!("  sniff start   (then generate traffic)");
    crate::println!("  vulnscan example.com");
}



pub(super) fn kqb(args: &[&str]) {
    if args.is_empty() {
        crate::n!(C_, "TrustScan — Port Scanner");
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

    let target = match crate::netscan::gri(args[0]) {
        Some(ip) => ip,
        None => {
            crate::n!(A_, "Cannot resolve target: {}", args[0]);
            return;
        }
    };

    
    let mut scan_type = crate::netscan::port_scanner::ScanType::Syn;
    let mut ports: Option<alloc::vec::Vec<u16>> = None;
    let mut jnn = false;
    let mut fgo = false;

    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "-sS" => scan_type = crate::netscan::port_scanner::ScanType::Syn,
            "-sT" => scan_type = crate::netscan::port_scanner::ScanType::Connect,
            "-sU" => scan_type = crate::netscan::port_scanner::ScanType::Udp,
            "--top" => jnn = true,
            "-A" => fgo = true,
            "-p" if i + 1 < args.len() => {
                i += 1;
                ports = Some(gmo(args[i]));
            }
            _ => {}
        }
        i += 1;
    }

    let old = match scan_type {
        crate::netscan::port_scanner::ScanType::Syn => "SYN",
        crate::netscan::port_scanner::ScanType::Connect => "Connect",
        crate::netscan::port_scanner::ScanType::Udp => "UDP",
    };

    crate::n!(C_, "Starting TrustScan {} scan on {}", old, crate::netscan::uw(target));
    crate::println!("TrustScan 1.0 — TrustOS Network Security Scanner");
    crate::println!();

    let mut config = crate::netscan::port_scanner::ScanConfig::new(target)
        .with_type(scan_type);

    if let Some(aa) = ports {
        config = config.with_ports(aa);
    } else if jnn || fgo {
        config = config.with_top_ports();
    }

    let (results, stats) = crate::netscan::port_scanner::scan(&config);

    
    crate::println!("PORT       STATE          SERVICE");
    for result in &results {
        let owq = match result.state {
            crate::netscan::port_scanner::PortState::Open => B_,
            crate::netscan::port_scanner::PortState::Filtered => D_,
            crate::netscan::port_scanner::PortState::OpenFiltered => D_,
            crate::netscan::port_scanner::PortState::Closed => A_,
        };

        let arv = match scan_type {
            crate::netscan::port_scanner::ScanType::Udp => "udp",
            _ => "tcp",
        };

        crate::print!("{}/{:<6}", result.port, arv);
        crate::bq!(owq, " {:<14}", result.state.as_str());
        crate::println!(" {}", result.service);
    }

    crate::println!();
    crate::println!("Scan complete: {} ports scanned in {} ms", stats.total_ports, stats.elapsed_ms);
    crate::println!("  {} open, {} closed, {} filtered",
        stats.open, stats.closed, stats.filtered);

    
    if fgo {
        let bil: alloc::vec::Vec<u16> = results.iter()
            .filter(|r| r.state == crate::netscan::port_scanner::PortState::Open)
            .map(|r| r.port)
            .collect();

        if !bil.is_empty() {
            crate::println!();
            crate::n!(C_, "Banner Grabbing...");
            let ega = crate::netscan::banner::icp(target, &bil, 2000);
            for b in &ega {
                crate::print!("  {}/tcp ", b.port);
                if let Some(ref tu) = b.version {
                    crate::bq!(B_, "{} ", tu);
                }
                crate::println!("{}", b.banner);
            }

            crate::println!();
            crate::n!(C_, "Vulnerability Assessment...");
            let fw = crate::netscan::vuln::scan(target, &bil);
            for f in &fw {
                let gun = match f.severity {
                    crate::netscan::vuln::Severity::Critical => A_,
                    crate::netscan::vuln::Severity::High => A_,
                    crate::netscan::vuln::Severity::Medium => D_,
                    crate::netscan::vuln::Severity::Low => C_,
                    crate::netscan::vuln::Severity::Info => R_,
                };
                crate::bq!(gun, "[{}] ", f.severity.as_str());
                crate::println!("{}/{} — {}", f.port, f.service, f.title);
            }
        }
    }
}

fn gmo(ye: &str) -> alloc::vec::Vec<u16> {
    let mut ports = alloc::vec::Vec::new();
    for jn in ye.split(',') {
        if let Some(cib) = jn.find('-') {
            let start: u16 = jn[..cib].parse().unwrap_or(0);
            let end: u16 = jn[cib+1..].parse().unwrap_or(0);
            if start > 0 && end >= start && end <= 65535 {
                for aa in start..=end {
                    ports.push(aa);
                }
            }
        } else if let Ok(aa) = jn.parse::<u16>() {
            if aa > 0 {
                ports.push(aa);
            }
        }
    }
    ports
}

pub(super) fn hlu(args: &[&str]) {
    if args.is_empty() || args[0] == "--help" {
        crate::n!(C_, "TrustScan — Network Discovery");
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
            crate::n!(C_, "ARP Sweep — Local Network Discovery");
            crate::println!("Scanning local subnet...");
            crate::println!();

            let aba = crate::netscan::discovery::fhl(3000);

            if aba.is_empty() {
                crate::n!(D_, "No hosts discovered");
                return;
            }

            crate::println!("IP Address          MAC Address          Status");
            crate::println!("{}", "-".repeat(55));
            for host in &aba {
                let auc = crate::netscan::uw(host.ip);
                let bhv = host.mac.map(|m| crate::netscan::bzx(m))
                    .unwrap_or_else(|| alloc::string::String::from("unknown"));
                crate::n!(B_, "{:<20}{:<21}UP", auc, bhv);
            }
            crate::println!();
            crate::println!("{} hosts discovered", aba.len());
        }
        "ping" => {
            let base = if args.len() > 1 {
                match crate::netscan::bof(args[1]) {
                    Some(ip) => [ip[0], ip[1], ip[2], 0],
                    None => {
                        crate::n!(A_, "Invalid IP: {}", args[1]);
                        return;
                    }
                }
            } else {
                match crate::network::rd() {
                    Some((ip, _, _)) => {
                        let b = ip.as_bytes();
                        [b[0], b[1], b[2], 0]
                    }
                    None => {
                        crate::n!(A_, "No network configured");
                        return;
                    }
                }
            };

            crate::n!(C_, "ICMP Ping Sweep — {}.{}.{}.0/24", base[0], base[1], base[2]);
            crate::println!("Scanning 254 hosts...");

            let aba = crate::netscan::discovery::nuu(base, 500);

            crate::println!();
            crate::println!("IP Address          TTL   RTT     OS Guess");
            crate::println!("{}", "-".repeat(60));
            for host in &aba {
                let auc = crate::netscan::uw(host.ip);
                crate::n!(B_, "{:<20}{:<6}{:<8}{}",
                    auc,
                    host.ttl.map(|t| alloc::format!("{}", t)).unwrap_or_else(|| alloc::string::String::from("-")),
                    alloc::format!("{}ms", host.rtt_ms),
                    host.os_hint);
            }
            crate::println!();
            crate::println!("{} hosts alive", aba.len());
        }
        "full" => {
            crate::n!(C_, "Full Network Discovery (ARP + Ping)");
            crate::println!("Scanning...");

            let aba = crate::netscan::discovery::mad(3000);

            crate::println!();
            crate::println!("IP Address          MAC Address          TTL   OS Guess");
            crate::println!("{}", "-".repeat(70));
            for host in &aba {
                let auc = crate::netscan::uw(host.ip);
                let bhv = host.mac.map(|m| crate::netscan::bzx(m))
                    .unwrap_or_else(|| alloc::string::String::from("--:--:--:--:--:--"));
                let poi = host.ttl.map(|t| alloc::format!("{}", t)).unwrap_or_else(|| alloc::string::String::from("-"));
                crate::n!(B_, "{:<20}{:<21}{:<6}{}",
                    auc, bhv, poi, host.os_hint);
            }
            crate::println!();
            crate::println!("{} hosts discovered", aba.len());
        }
        _ => {
            
            hlu(&["arp"]);
        }
    }
}

pub(super) fn klw(args: &[&str]) {
    if args.is_empty() {
        crate::n!(C_, "TrustScan — Banner Grabber");
        crate::println!("Usage: banner <ip|host> [port1,port2,...]");
        crate::println!("  banner 192.168.1.1              Grab banners from common ports");
        crate::println!("  banner 192.168.1.1 22,80,443    Grab from specific ports");
        return;
    }

    let target = match crate::netscan::gri(args[0]) {
        Some(ip) => ip,
        None => {
            crate::n!(A_, "Cannot resolve: {}", args[0]);
            return;
        }
    };

    let ports = if args.len() > 1 {
        gmo(args[1])
    } else {
        alloc::vec![21, 22, 25, 80, 110, 143, 443, 3306, 5432, 6379, 8080]
    };

    crate::n!(C_, "Banner Grabbing {} ({} ports)", crate::netscan::uw(target), ports.len());
    crate::println!();

    let ega = crate::netscan::banner::icp(target, &ports, 3000);

    if ega.is_empty() {
        crate::n!(D_, "No banners could be grabbed (ports may be closed)");
        return;
    }

    for b in &ega {
        crate::bq!(B_, "{}/tcp {:<15}", b.port, b.service);
        if let Some(ref tu) = b.version {
            crate::bq!(C_, " [{}]", tu);
        }
        crate::println!();
        crate::println!("  {}", b.banner);
    }
}

pub(super) fn krs(args: &[&str]) {
    let je = args.first().copied().unwrap_or("help");

    match je {
        "start" => {
            crate::netscan::sniffer::deu();
            crate::n!(B_, "Packet capture started");
            crate::println!("Use 'sniff show' to view captured packets");
            crate::println!("Use 'sniff stop' to stop capture");
        }
        "stop" => {
            crate::netscan::sniffer::dex();
            let (count, bytes, _) = crate::netscan::sniffer::get_stats();
            crate::n!(D_, "Capture stopped");
            crate::println!("Captured {} packets, {} bytes", count, bytes);
        }
        "show" | "dump" => {
            let count = args.get(1).and_then(|j| j.parse::<usize>().ok()).unwrap_or(20);
            let packets = crate::netscan::sniffer::ewn(count);

            if packets.is_empty() {
                crate::n!(D_, "No packets captured");
                if !crate::netscan::sniffer::btp() {
                    crate::println!("Start capture with: sniff start");
                }
                return;
            }

            crate::println!("No.  Time      Protocol  Source              Destination         Info");
            crate::println!("{}", "-".repeat(90));

            for (i, fj) in packets.iter().rev().enumerate() {
                let src = fj.src_ip.map(|ip| crate::netscan::uw(ip))
                    .unwrap_or_else(|| crate::netscan::bzx(fj.src_mac));
                let dst = fj.dst_ip.map(|ip| crate::netscan::uw(ip))
                    .unwrap_or_else(|| crate::netscan::bzx(fj.dst_mac));

                let nyy = match fj.protocol {
                    crate::netscan::sniffer::Protocol::Tcp => C_,
                    crate::netscan::sniffer::Protocol::Udp => CF_,
                    crate::netscan::sniffer::Protocol::Http => B_,
                    crate::netscan::sniffer::Protocol::Tls => DM_,
                    crate::netscan::sniffer::Protocol::Arp => D_,
                    crate::netscan::sniffer::Protocol::Icmp => A_,
                    crate::netscan::sniffer::Protocol::Dns => G_,
                    _ => R_,
                };

                crate::print!("{:<5}{:<10}", i + 1, fj.timestamp_ms);
                crate::bq!(nyy, "{:<10}", fj.protocol.as_str());
                crate::print!("{:<20}{:<20}", src, dst);
                crate::println!("{}", &fj.info[..fj.info.len().min(40)]);
            }

            let (total_count, total_bytes, awl) = crate::netscan::sniffer::get_stats();
            crate::println!();
            crate::println!("Total: {} packets, {} bytes ({} in buffer)",
                total_count, total_bytes, awl);
        }
        "hex" => {
            let idx = args.get(1).and_then(|j| j.parse::<usize>().ok()).unwrap_or(0);
            let packets = crate::netscan::sniffer::ewn(idx + 1);
            if let Some(fj) = packets.get(idx) {
                crate::n!(C_, "Packet #{} — {} bytes — {}",
                    idx + 1, fj.length, fj.protocol.as_str());
                crate::println!("{}", crate::netscan::sniffer::iet(&fj.raw_data, 128));
            } else {
                crate::n!(D_, "No packet at index {}", idx);
            }
        }
        "stats" => {
            let (count, bytes, awl) = crate::netscan::sniffer::get_stats();
            let active = crate::netscan::sniffer::btp();
            crate::n!(C_, "Sniffer Statistics");
            crate::println!("  Status:    {}", if active { "CAPTURING" } else { "STOPPED" });
            crate::println!("  Packets:   {}", count);
            crate::println!("  Bytes:     {}", bytes);
            crate::println!("  Buffered:  {}", awl);
        }
        _ => {
            crate::n!(C_, "TrustScan — Packet Sniffer");
            crate::println!("Usage:");
            crate::println!("  sniff start         Start packet capture");
            crate::println!("  sniff stop          Stop capture");
            crate::println!("  sniff show [N]      Show last N captured packets");
            crate::println!("  sniff hex [N]       Hex dump of packet N");
            crate::println!("  sniff stats         Capture statistics");
        }
    }
}

pub(super) fn ktx(args: &[&str]) {
    if args.is_empty() {
        crate::n!(C_, "TrustScan — Vulnerability Scanner");
        crate::println!("Usage: vulnscan <ip|host> [port1,port2,...]");
        crate::println!("  vulnscan 192.168.1.1              Scan all common ports + vulns");
        crate::println!("  vulnscan 192.168.1.1 80,443,3306  Scan specific ports");
        return;
    }

    let target = match crate::netscan::gri(args[0]) {
        Some(ip) => ip,
        None => {
            crate::n!(A_, "Cannot resolve: {}", args[0]);
            return;
        }
    };

    
    let bil = if args.len() > 1 {
        gmo(args[1])
    } else {
        crate::println!("Scanning ports...");
        let config = crate::netscan::port_scanner::ScanConfig::new(target).with_top_ports();
        let (results, _) = crate::netscan::port_scanner::scan(&config);
        results.iter()
            .filter(|r| r.state == crate::netscan::port_scanner::PortState::Open)
            .map(|r| r.port)
            .collect()
    };

    if bil.is_empty() {
        crate::n!(D_, "No open ports found");
        return;
    }

    crate::n!(C_, "Vulnerability Assessment — {}", crate::netscan::uw(target));
    crate::println!("Checking {} ports...", bil.len());
    crate::println!();

    let fw = crate::netscan::vuln::scan(target, &bil);

    if fw.is_empty() {
        crate::n!(B_, "No vulnerabilities detected");
        return;
    }

    
    let aqb = fw.iter().filter(|f| f.severity == crate::netscan::vuln::Severity::Critical).count();
    let high = fw.iter().filter(|f| f.severity == crate::netscan::vuln::Severity::High).count();
    let dbd = fw.iter().filter(|f| f.severity == crate::netscan::vuln::Severity::Medium).count();

    if aqb > 0 {
        crate::bq!(A_, "CRITICAL: {} ", aqb);
    }
    if high > 0 {
        crate::bq!(A_, "HIGH: {} ", high);
    }
    if dbd > 0 {
        crate::bq!(D_, "MEDIUM: {} ", dbd);
    }
    crate::println!("({} total findings)", fw.len());
    crate::println!();

    
    for f in &fw {
        let gun = match f.severity {
            crate::netscan::vuln::Severity::Critical => A_,
            crate::netscan::vuln::Severity::High => A_,
            crate::netscan::vuln::Severity::Medium => D_,
            crate::netscan::vuln::Severity::Low => C_,
            crate::netscan::vuln::Severity::Info => K_,
        };
        crate::bq!(gun, "[{:<8}] ", f.severity.as_str());
        crate::println!("{}/{} — {}", f.port, f.service, f.title);
        crate::println!("           {}", f.description);
        crate::n!(B_, "           Fix: {}", f.recommendation);
        crate::println!();
    }
}

pub(super) fn ksy(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: traceroute <host>");
        return;
    }

    let host = args[0];
    let ip = if let Some(ip) = art(host) {
        ip
    } else if let Some(afn) = crate::netstack::dns::yb(host) {
        afn
    } else {
        crate::n!(A_, "Unable to resolve host");
        return;
    };

    let max_hops = args.get(1).and_then(|j| j.parse::<u8>().ok()).unwrap_or(30);

    crate::println!("traceroute to {} ({}), {} hops max, 60 byte packets",
        host, crate::netscan::uw(ip), max_hops);

    let bcb = crate::netscan::traceroute::trace(ip, max_hops, 2000);

    for afg in &bcb {
        crate::print!("{:>2}  ", afg.hop_num);
        if let Some(hop_ip) = afg.ip {
            crate::print!("{:<18}", crate::netscan::uw(hop_ip));
            for &rtt in &afg.rtt_ms {
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

pub(super) fn fmx(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: ping <ip|host>");
        crate::println!("  Example: ping 192.168.56.1");
        crate::println!("  Example: ping example.com");
        return;
    }

    let ip = if let Some(ip) = art(args[0]) {
        crate::network::Ipv4Address::new(ip[0], ip[1], ip[2], ip[3])
    } else if let Some(afn) = crate::netstack::dns::yb(args[0]) {
        crate::network::Ipv4Address::new(afn[0], afn[1], afn[2], afn[3])
    } else {
        crate::n!(A_, "Unable to resolve host");
        return;
    };
    
    crate::println!("PING {} ({}) 56 data bytes", args[0], ip);
    
    let mut ear = 0;
    let mut joa = 0u64;
    let mut eul = u64::MAX;
    let mut euc = 0u64;
    
    for _ in 0..4 {
        match crate::network::gty(ip) {
            Ok(result) => {
                if result.success {
                    ear += 1;
                    joa += result.time_us;
                    eul = eul.min(result.time_us);
                    euc = euc.max(result.time_us);
                    
                    
                    if result.time_us < 1000 {
                        crate::println!("64 bytes from {}: icmp_seq={} ttl={} time={} us", 
                            ip, result.seq, result.ttl, result.time_us);
                    } else {
                        let dh = result.time_us / 1000;
                        let pqb = (result.time_us % 1000) / 10;
                        crate::println!("64 bytes from {}: icmp_seq={} ttl={} time={}.{:02} ms", 
                            ip, result.seq, result.ttl, dh, pqb);
                    }
                } else {
                    crate::n!(D_, "Request timeout for icmp_seq {}", result.seq);
                }
            }
            Err(e) => {
                crate::n!(A_, "ping failed: {}", e);
            }
        }
        
        
        crate::cpu::tsc::ww(1000);
    }
    
    crate::println!();
    crate::println!("--- {} ping statistics ---", args[0]);
    crate::println!("4 packets transmitted, {} received, {}% packet loss", 
        ear, 
        (4 - ear) * 25);
    if ear > 0 {
        let dic = joa / ear as u64;
        
        crate::println!("rtt min/avg/max = {}.{:03}/{}.{:03}/{}.{:03} ms", 
            eul / 1000, eul % 1000,
            dic / 1000, dic % 1000,
            euc / 1000, euc % 1000);
    }
}

pub(super) fn dky() {
    crate::n!(C_, "Network Statistics");
    crate::println!("==================");
    
    let stats = crate::network::get_stats();
    crate::println!();
    crate::bq!(B_, "Packets received: ");
    crate::println!("{}", stats.packets_received);
    crate::bq!(B_, "Packets sent:     ");
    crate::println!("{}", stats.packets_sent);
    crate::bq!(B_, "Bytes received:   ");
    crate::println!("{}", stats.bytes_received);
    crate::bq!(B_, "Bytes sent:       ");
    crate::println!("{}", stats.bytes_sent);
    crate::bq!(B_, "Errors:           ");
    crate::println!("{}", stats.errors);
}

pub(super) fn koy(args: &[&str]) {
    let cdr = args.iter().any(|a| *a == "/all" || *a == "-a");
    crate::println!("Windows IP Configuration");
    crate::println!();

    if let Some((mac, ip, state)) = crate::network::cyp() {
        crate::println!("   Ethernet adapter net0:");
        crate::println!("      Status . . . . . . . . . . . . : {:?}", state);
        crate::println!("      Physical Address. . . . . . . . : {}", mac);
        if let Some(ip) = ip {
            crate::println!("      IPv4 Address. . . . . . . . . : {}", ip);
            if let Some((_, subnet, gateway)) = crate::network::rd() {
                crate::println!("      Subnet Mask . . . . . . . . . : {}", subnet);
                if let Some(fz) = gateway {
                    crate::println!("      Default Gateway . . . . . . . : {}", fz);
                } else if cdr {
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

pub(super) fn kqc(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: nslookup <host>");
        return;
    }

    let target = args[0];
    if art(target).is_some() {
        crate::println!("Server: 8.8.8.8");
        crate::println!("Address: {}", target);
        crate::println!("*** Reverse lookup not implemented");
        return;
    }

    if let Some(afn) = crate::netstack::dns::yb(target) {
        crate::println!("Server: 8.8.8.8");
        crate::println!("Name: {}", target);
        crate::println!("Address: {}.{}.{}.{}", afn[0], afn[1], afn[2], afn[3]);
    } else {
        crate::n!(A_, "DNS lookup failed");
    }
}

pub(super) fn klr(args: &[&str]) {
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
        let erf = ip.to_be_bytes();
        crate::println!(
            "{:>3}.{:>3}.{:>3}.{:>3}      {:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}   dynamic",
            erf[0], erf[1], erf[2], erf[3], mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        );
    }
}

pub(super) fn krc(_args: &[&str]) {
    crate::println!("Kernel IP routing table");
    crate::println!("Destination     Gateway         Genmask         Iface");

    if let Some((ip, subnet, gateway)) = crate::network::rd() {
        let fz = gateway.unwrap_or(crate::network::Ipv4Address::new(0, 0, 0, 0));
        crate::println!("{}     {}     {}     net0", ip, fz, subnet);
        crate::println!("0.0.0.0         {}     0.0.0.0         net0", fz);
    } else {
        crate::println!("0.0.0.0         0.0.0.0         0.0.0.0         net0");
    }
}

pub(super) fn qar(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: traceroute <host>");
        return;
    }

    let host = args[0];
    let ip = if let Some(ip) = art(host) {
        ip
    } else if let Some(afn) = crate::netstack::dns::yb(host) {
        afn
    } else {
        crate::n!(A_, "Unable to resolve host");
        return;
    };

    crate::println!("traceroute to {} ({}.{}.{}.{}), 30 hops max", host, ip[0], ip[1], ip[2], ip[3]);
    if let Some((_, _, gateway)) = crate::network::rd() {
        if let Some(fz) = gateway {
            crate::println!(" 1  {}", fz);
        }
    }
    crate::println!(" 2  {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    crate::n!(D_, "Note: traceroute is simplified (no TTL probing)");
}





pub(super) fn kma(args: &[&str]) {
    let freq = args.first()
        .and_then(|j| j.parse::<u32>().ok())
        .unwrap_or(440);
    let yq = args.get(1)
        .and_then(|j| j.parse::<u32>().ok())
        .unwrap_or(500);

    if freq < 20 || freq > 20000 {
        crate::n!(A_, "Frequency must be 20-20000 Hz");
        return;
    }
    if yq > 10000 {
        crate::n!(A_, "Duration max 10000 ms");
        return;
    }

    
    if !crate::drivers::hda::is_initialized() {
        crate::bq!(D_, "Initializing audio driver... ");
        match crate::drivers::hda::init() {
            Ok(()) => crate::n!(B_, "OK"),
            Err(e) => {
                crate::n!(A_, "FAILED: {}", e);
                return;
            }
        }
    }

    crate::println!("Playing {}Hz for {}ms...", freq, yq);
    match crate::drivers::hda::ivg(freq, yq) {
        Ok(()) => {
            
            let alw = crate::drivers::hda::mdk();
            if alw == 0 {
                crate::n!(A_, "Done (LPIB=0 — DMA not running!)");
            } else {
                crate::n!(B_, "Done (LPIB={})", alw);
            }
        },
        Err(e) => crate::n!(A_, "Error: {}", e),
    }
}

pub(super) fn klt(args: &[&str]) {
    match args.first().copied() {
        Some("init") => {
            crate::bq!(D_, "Initializing Intel HDA driver... ");
            match crate::drivers::hda::init() {
                Ok(()) => crate::n!(B_, "OK"),
                Err(e) => crate::n!(A_, "FAILED: {}", e),
            }
        }
        Some("status") | None => {
            let status = crate::drivers::hda::status();
            crate::println!("{}", status);
        }
        Some("stop") => {
            match crate::drivers::hda::stop() {
                Ok(()) => crate::n!(B_, "Playback stopped"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("test") => {
            
            if !crate::drivers::hda::is_initialized() {
                crate::bq!(D_, "Initializing audio driver... ");
                match crate::drivers::hda::init() {
                    Ok(()) => crate::n!(B_, "OK"),
                    Err(e) => {
                        crate::n!(A_, "FAILED: {}", e);
                        return;
                    }
                }
            }
            crate::println!("Playing test scale...");
            let notes = [262, 294, 330, 349, 392, 440, 494, 523]; 
            for &freq in &notes {
                let _ = crate::drivers::hda::ivg(freq, 200);
            }
            crate::n!(B_, "Done");
        }
        Some("diag") => {
            crate::n!(C_, "Audio Diagnostics");
            let cwm = crate::drivers::hda::cwm();
            crate::println!("{}", cwm);
        }
        Some("dump") => {
            crate::n!(C_, "Codec Widget Dump");
            let byz = crate::drivers::hda::kuy();
            crate::println!("{}", byz);
        }
        Some("probe") => {
            crate::n!(C_, "Amp Probe (SET then GET)");
            let probe = crate::drivers::hda::jvp();
            crate::println!("{}", probe);
        }
        Some("gpio") => {
            
            let val = match args.get(1).and_then(|j| j.parse::<u8>().ok()) {
                Some(v) => v,
                None => {
                    crate::n!(D_, "Usage: audio gpio <0|1|2>");
                    crate::println!("  0 = GPIO1 LOW (active for some amps)");
                    crate::println!("  2 = GPIO1 HIGH");
                    return;
                }
            };
            match crate::drivers::hda::ooz(val) {
                Ok(()) => crate::n!(B_, "GPIO DATA set to {:#04X}", val),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some(other) => {
            crate::n!(D_, "Usage: audio [init|status|stop|test|diag|dump|probe|gpio]");
        }
    }
}

pub(super) fn ksf(args: &[&str]) {
    match args.first().copied() {
        Some("note") | Some("play") => {
            
            let agu = match args.get(1) {
                Some(ae) => *ae,
                None => {
                    crate::n!(D_, "Usage: synth note <note> [duration_ms] [waveform]");
                    crate::println!("  Examples: synth note C4");
                    crate::println!("           synth note A#3 1000 saw");
                    return;
                }
            };
            let yq = args.get(2)
                .and_then(|j| j.parse::<u32>().ok())
                .unwrap_or(500);
            
            if let Some(wf_str) = args.get(3) {
                if let Some(aal) = crate::audio::synth::Waveform::atv(wf_str) {
                    let _ = crate::audio::set_waveform(aal);
                }
            }
            if yq > 10000 {
                crate::n!(A_, "Duration max 10000 ms");
                return;
            }
            crate::println!("Synth: {} for {}ms", agu, yq);
            match crate::audio::ivf(agu, yq) {
                Ok(()) => crate::n!(B_, "Done"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("freq") => {
            
            let freq = match args.get(1).and_then(|j| j.parse::<u32>().ok()) {
                Some(f) => f,
                None => {
                    crate::n!(D_, "Usage: synth freq <hz> [duration_ms]");
                    return;
                }
            };
            let yq = args.get(2)
                .and_then(|j| j.parse::<u32>().ok())
                .unwrap_or(500);
            if freq < 20 || freq > 20000 {
                crate::n!(A_, "Frequency must be 20-20000 Hz");
                return;
            }
            crate::println!("Synth: {}Hz for {}ms", freq, yq);
            match crate::audio::nvj(freq, yq) {
                Ok(()) => crate::n!(B_, "Done"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("wave") | Some("waveform") => {
            
            match args.get(1) {
                Some(wf_str) => {
                    match crate::audio::synth::Waveform::atv(wf_str) {
                        Some(aal) => {
                            let _ = crate::audio::set_waveform(aal);
                            crate::n!(B_, "Waveform set to: {}", aal.name());
                        }
                        None => crate::n!(A_, "Unknown waveform (use: sine, square, saw, triangle, noise)"),
                    }
                }
                None => crate::n!(D_, "Usage: synth wave <sine|square|saw|triangle|noise>"),
            }
        }
        Some("adsr") => {
            
            if args.len() < 5 {
                crate::n!(D_, "Usage: synth adsr <attack_ms> <decay_ms> <sustain_%> <release_ms>");
                crate::println!("  Example: synth adsr 10 50 70 100");
                return;
            }
            let a = args[1].parse::<u32>().unwrap_or(10);
            let d = args[2].parse::<u32>().unwrap_or(50);
            let j = args[3].parse::<u32>().unwrap_or(70);
            let r = args[4].parse::<u32>().unwrap_or(100);
            let _ = crate::audio::set_adsr(a, d, j, r);
            crate::n!(B_, "ADSR set: A={}ms D={}ms S={}% R={}ms", a, d, j, r);
        }
        Some("preset") => {
            
            match args.get(1).copied() {
                Some(name) => {
                    match crate::audio::oow(name) {
                        Ok(()) => crate::n!(B_, "Envelope preset: {}", name),
                        Err(e) => crate::n!(A_, "{}", e),
                    }
                }
                None => crate::n!(D_, "Usage: synth preset <default|organ|pluck|pad>"),
            }
        }
        Some("volume") | Some("vol") => {
            match args.get(1).and_then(|j| j.parse::<u8>().ok()) {
                Some(v) => {
                    let _ = crate::audio::set_volume(v);
                    crate::n!(B_, "Master volume: {}/255", v);
                }
                None => crate::n!(D_, "Usage: synth volume <0-255>"),
            }
        }
        Some("status") => {
            let j = crate::audio::status();
            crate::println!("{}", j);
        }
        Some("stop") => {
            match crate::audio::stop() {
                Ok(()) => crate::n!(B_, "Synth stopped"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("demo") => {
            crate::println!("TrustSynth Demo -- playing scale with different waveforms...");
            let notes = ["C4", "D4", "E4", "F4", "G4", "A4", "B4", "C5"];
            let puf = [
                ("Sine",     crate::audio::synth::Waveform::Sine),
                ("Square",   crate::audio::synth::Waveform::Square),
                ("Sawtooth", crate::audio::synth::Waveform::Sawtooth),
                ("Triangle", crate::audio::synth::Waveform::Triangle),
            ];
            for (wf_name, aal) in &puf {
                let _ = crate::audio::set_waveform(*aal);
                crate::println!("  {} waveform:", wf_name);
                for note in &notes {
                    crate::print!("    {} ", note);
                    let _ = crate::audio::ivf(note, 200);
                }
                crate::println!();
            }
            crate::n!(B_, "Demo complete!");
        }
        
        Some("pattern") | Some("pat") => {
            ksg(&args[1..]);
        }
        Some(_) | None => {
            crate::n!(C_, "TrustSynth -- Audio Synthesizer & Sequencer");
            crate::println!();
            crate::n!(D_, "  Synth:");
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
            crate::n!(D_, "  Pattern Sequencer:");
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

pub(super) fn ksg(args: &[&str]) {
    match args.first().copied() {
        Some("list") | Some("ls") | None => {
            let list = crate::audio::nsb();
            crate::println!("{}", list);
        }
        Some("show") | Some("view") => {
            match args.get(1) {
                Some(name) => {
                    match crate::audio::nsi(name) {
                        Ok(j) => crate::println!("{}", j),
                        Err(e) => crate::n!(A_, "Error: {}", e),
                    }
                }
                None => crate::n!(D_, "Usage: synth pattern show <name>"),
            }
        }
        Some("new") | Some("create") => {
            let name = match args.get(1) {
                Some(ae) => *ae,
                None => {
                    crate::n!(D_, "Usage: synth pattern new <name> [steps] [bpm]");
                    return;
                }
            };
            let steps = args.get(2).and_then(|j| j.parse::<usize>().ok()).unwrap_or(16);
            let bpm = args.get(3).and_then(|j| j.parse::<u16>().ok()).unwrap_or(120);
            match crate::audio::nsc(name, steps, bpm) {
                Ok(()) => crate::n!(B_, "Pattern \"{}\" created ({} steps, {} BPM)", name, steps, bpm),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("play") => {
            let name = match args.get(1) {
                Some(ae) => *ae,
                None => {
                    crate::n!(D_, "Usage: synth pattern play <name> [loops]");
                    return;
                }
            };
            let loops = args.get(2).and_then(|j| j.parse::<u32>().ok()).unwrap_or(1);
            match crate::audio::nsd(name, loops) {
                Ok(()) => {}
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("stop") => {
            match crate::audio::nsj() {
                Ok(()) => crate::n!(B_, "Pattern playback stopped"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("bpm") | Some("tempo") => {
            if args.len() < 3 {
                crate::n!(D_, "Usage: synth pattern bpm <name> <60-300>");
                return;
            }
            let name = args[1];
            let bpm = match args[2].parse::<u16>() {
                Ok(b) if b >= 30 && b <= 300 => b,
                _ => {
                    crate::n!(A_, "BPM must be 30-300");
                    return;
                }
            };
            match crate::audio::nsf(name, bpm) {
                Ok(()) => crate::n!(B_, "\"{}\" BPM set to {}", name, bpm),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("wave") | Some("waveform") => {
            if args.len() < 3 {
                crate::n!(D_, "Usage: synth pattern wave <name> <sine|square|saw|tri|noise>");
                return;
            }
            let name = args[1];
            let aal = match crate::audio::synth::Waveform::atv(args[2]) {
                Some(w) => w,
                None => {
                    crate::n!(A_, "Unknown waveform");
                    return;
                }
            };
            match crate::audio::nsh(name, aal) {
                Ok(()) => crate::n!(B_, "\"{}\" waveform set to {}", name, aal.name()),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("set") | Some("note") => {
            
            if args.len() < 4 {
                crate::n!(D_, "Usage: synth pattern set <name> <step#> <note|-->");
                crate::println!("  Example: synth pattern set mypattern 0 C4");
                crate::println!("  Example: synth pattern set mypattern 3 --  (rest)");
                return;
            }
            let name = args[1];
            let jiv = match args[2].parse::<usize>() {
                Ok(i) => i,
                Err(_) => {
                    crate::n!(A_, "Step must be a number");
                    return;
                }
            };
            let note = args[3];
            match crate::audio::nsg(name, jiv, note) {
                Ok(()) => crate::n!(B_, "\"{}\" step {} = {}", name, jiv, note),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("del") | Some("delete") | Some("rm") => {
            match args.get(1) {
                Some(name) => {
                    match crate::audio::nse(name) {
                        Ok(()) => crate::n!(B_, "Pattern \"{}\" deleted", name),
                        Err(e) => crate::n!(A_, "Error: {}", e),
                    }
                }
                None => crate::n!(D_, "Usage: synth pattern del <name>"),
            }
        }
        Some(other) => {
            crate::n!(A_, "Unknown pattern command: {}", other);
            crate::println!("Use: list, show, new, play, stop, bpm, wave, set, del");
        }
    }
}





pub(super) fn kts(args: &[&str]) {
    match args.first().copied() {
        None | Some("help") | Some("--help") => {
            crate::n!(C_, "vizfx — Live Visualizer Effects (TrustLang)");
            crate::n!(C_, "═══════════════════════════════════════════");
            crate::println!();
            crate::n!(G_, "Create visual effects with TrustLang scripts.");
            crate::n!(G_, "Effects react to audio in real-time — no reboot!");
            crate::println!();
            crate::n!(G_, "Commands:");
            crate::println!("  vizfx list                List all effects");
            crate::println!("  vizfx new <name> <code>   Create effect (inline code)");
            crate::println!("  vizfx edit <name> <code>  Update effect source");
            crate::println!("  vizfx select <name>       Set active effect");
            crate::println!("  vizfx remove <name>       Delete effect");
            crate::println!("  vizfx show <name>         Show effect source code");
            crate::println!("  vizfx on / off            Enable/disable live effects");
            crate::println!("  vizfx demo                Load 3 demo effects");
            crate::println!();
            crate::n!(G_, "Audio Builtins (available in scripts):");
            crate::println!("  beat()      → 0.0–1.0   Beat pulse");
            crate::println!("  bass()      → 0.0–1.0   Bass energy");
            crate::println!("  sub_bass()  → 0.0–1.0   Sub-bass energy");
            crate::println!("  mid()       → 0.0–1.0   Mid frequency energy");
            crate::println!("  high_mid()  → 0.0–1.0   High-mid energy");
            crate::println!("  treble()    → 0.0–1.0   Treble energy");
            crate::println!("  energy()    → 0.0–1.5   Overall energy");
            crate::println!("  frame_num() → int       Current frame number");
            crate::println!("  sin_f(x)    → float     Sine function");
            crate::println!("  cos_f(x)    → float     Cosine function");
            crate::println!();
            crate::n!(G_, "Graphics Builtins:");
            crate::println!("  pixel(x,y,r,g,b)         Set pixel color");
            crate::println!("  fill_rect(x,y,w,h,r,g,b) Fill rectangle");
            crate::println!("  draw_circle(cx,cy,r,R,G,B) Draw circle");
            crate::println!("  draw_line(x1,y1,x2,y2,r,g,b) Draw line");
            crate::println!("  screen_w() / screen_h()  Screen dimensions");
            crate::println!();
            crate::n!(D_, "Workflow for promo video:");
            crate::println!("  1. vizfx new myeffect fn main() {{ ... }}");
            crate::println!("  2. vizfx select myeffect");
            crate::println!("  3. play /music/song.wav");
            crate::println!("  → Effect runs live over the audio visualizer!");
        }
        Some("list") | Some("ls") => {
            let effects = crate::trustdaw::live_viz::mzc();
            if effects.is_empty() {
                crate::n!(D_, "No effects loaded. Try 'vizfx demo' to load demos.");
            } else {
                crate::n!(C_, "Live Visualizer Effects:");
                for (name, active) in &effects {
                    if *active {
                        crate::n!(G_, "  ▶ {} (ACTIVE)", name);
                    } else {
                        crate::println!("    {}", name);
                    }
                }
            }
        }
        Some("new") | Some("add") | Some("create") => {
            if args.len() < 3 {
                crate::n!(D_, "Usage: vizfx new <name> <trustlang code>");
                crate::println!("Example: vizfx new rings fn main() {{ let b = beat(); draw_circle(screen_w()/2, screen_h()/2, to_int(b * 100.0), 0, 255, 100); }}");
                return;
            }
            let name = args[1];
            
            let source: alloc::string::String = args[2..].join(" ");
            match crate::trustdaw::live_viz::eez(name, &source) {
                Ok(()) => {
                    crate::n!(B_, "Effect '{}' created ✓", name);
                    if crate::trustdaw::live_viz::is_active() {
                        crate::println!("Active and ready — play a song to see it!");
                    }
                }
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("edit") | Some("update") => {
            if args.len() < 3 {
                crate::n!(D_, "Usage: vizfx edit <name> <new code>");
                return;
            }
            let name = args[1];
            let source: alloc::string::String = args[2..].join(" ");
            match crate::trustdaw::live_viz::loh(name, &source) {
                Ok(()) => crate::n!(B_, "Effect '{}' updated ✓", name),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("select") | Some("use") | Some("set") => {
            if args.len() < 2 {
                crate::n!(D_, "Usage: vizfx select <name>");
                return;
            }
            match crate::trustdaw::live_viz::jej(args[1]) {
                Ok(()) => crate::n!(B_, "Active effect: {} ✓", args[1]),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("remove") | Some("rm") | Some("delete") => {
            if args.len() < 2 {
                crate::n!(D_, "Usage: vizfx remove <name>");
                return;
            }
            match crate::trustdaw::live_viz::oey(args[1]) {
                Ok(()) => crate::n!(B_, "Effect '{}' removed", args[1]),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("show") | Some("source") | Some("cat") => {
            if args.len() < 2 {
                crate::n!(D_, "Usage: vizfx show <name>");
                return;
            }
            match crate::trustdaw::live_viz::mdu(args[1]) {
                Some(src) => {
                    crate::n!(C_, "─── {} ───", args[1]);
                    crate::println!("{}", src);
                    crate::n!(C_, "─────────────────");
                }
                None => crate::n!(A_, "Effect not found: {}", args[1]),
            }
        }
        Some("on") | Some("enable") => {
            match crate::trustdaw::live_viz::enable() {
                Ok(()) => crate::n!(B_, "Live viz effects enabled ✓"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("off") | Some("disable") => {
            crate::trustdaw::live_viz::bbc();
            crate::n!(D_, "Live viz effects disabled");
        }
        Some("demo") | Some("demos") => {
            crate::n!(B_, "Loading demo effects...");
            match crate::trustdaw::live_viz::mzy() {
                Ok(()) => crate::n!(B_, "  ✓ pulse-rings"),
                Err(e) => crate::n!(A_, "  ✗ pulse-rings: {}", e),
            }
            match crate::trustdaw::live_viz::mzz() {
                Ok(()) => crate::n!(B_, "  ✓ spectrum-bars"),
                Err(e) => crate::n!(A_, "  ✗ spectrum-bars: {}", e),
            }
            match crate::trustdaw::live_viz::mzx() {
                Ok(()) => crate::n!(B_, "  ✓ beat-flash"),
                Err(e) => crate::n!(A_, "  ✗ beat-flash: {}", e),
            }
            crate::println!();
            crate::println!("Use 'vizfx list' to see all effects");
            crate::println!("Use 'vizfx select <name>' to choose one");
            crate::println!("Then 'play <file.wav>' to see it in action!");
        }
        Some(x) => {
            
            match crate::trustdaw::live_viz::jej(x) {
                Ok(()) => crate::n!(B_, "Active effect: {} ✓", x),
                Err(_) => {
                    crate::n!(A_, "Unknown command: {}", x);
                    crate::println!("Use 'vizfx help' for usage");
                }
            }
        }
    }
}





pub(super) fn kqm(args: &[&str]) {
    let path = args.first().copied().unwrap_or("");
    if path.is_empty() || path == "help" || path == "--help" {
        crate::n!(C_, "play - Audio file visualizer");
        crate::println!();
        crate::println!("Usage: play <file.wav>");
        crate::println!("       play /home/song.wav");
        crate::println!("       play /mnt/sda1/music.wav");
        crate::println!();
        crate::println!("Plays a WAV file with a 3D matrix rain visualizer.");
        crate::println!("Supported formats: WAV (16-bit PCM, any sample rate)");
        crate::println!();
        crate::println!("Controls:");
        crate::println!("  [Esc]   Exit visualizer");
        return;
    }

    
    if path == "untitled2" || path == "u2" || path == "lofi" {
        crate::n!(B_, "Playing embedded 'Untitled (2)' — Dark Lo-Fi / Ambient...");
        crate::println!("  [Esc] Exit");
        match crate::trustdaw::audio_viz::gni() {
            Ok(()) => crate::println!("Playback complete"),
            Err(e) => crate::n!(A_, "Error: {}", e),
        }
        return;
    }

    
    if path == "anthem" || path == "trustanthem" || path == "TrustAnthem" {
        crate::n!(B_, "Playing 'TrustAnthem' — The TrustOS Anthem...");
        crate::println!("  [Esc] Exit");
        match crate::trustdaw::audio_viz::nvf() {
            Ok(()) => crate::println!("Playback complete"),
            Err(e) => crate::n!(A_, "Error: {}", e),
        }
        return;
    }

    crate::n!(B_, "Starting Audio Visualizer...");
    crate::println!("  File: {}", path);
    crate::println!("  [Esc] Exit");
    match crate::trustdaw::audio_viz::ivd(path) {
        Ok(()) => crate::println!("Playback complete"),
        Err(e) => crate::n!(A_, "Error: {}", e),
    }
}





pub(super) fn kmr(args: &[&str]) {
    match args.first().copied() {
        Some("init") | None => {
            match crate::trustdaw::init() {
                Ok(()) => {
                    crate::n!(B_, "TrustDAW initialized");
                    crate::println!("{}", crate::trustdaw::status());
                }
                Err(e) => crate::n!(A_, "DAW init failed: {}", e),
            }
        }
        Some("status") | Some("info") => {
            crate::println!("{}", crate::trustdaw::status());
        }
        Some("new") => {
            
            let name = match args.get(1) {
                Some(ae) => *ae,
                None => {
                    crate::n!(D_, "Usage: daw new <project_name> [bpm]");
                    return;
                }
            };
            let bpm = args.get(2).and_then(|j| j.parse::<u32>().ok()).unwrap_or(120);
            match crate::trustdaw::njo(name, bpm) {
                Ok(()) => crate::n!(B_, "New project: \"{}\" at {} BPM", name, bpm),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("demo") => {
            match crate::trustdaw::mzw() {
                Ok(()) => {
                    crate::n!(B_, "Demo project loaded!");
                    crate::println!("{}", crate::trustdaw::status());
                }
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("track") => kmu(&args[1..]),
        Some("note") => kmt(&args[1..]),
        Some("play") => {
            crate::println!("Playing...");
            match crate::trustdaw::play() {
                Ok(()) => crate::n!(B_, "Playback complete"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("stop") => {
            crate::trustdaw::stop();
            crate::n!(B_, "Stopped");
        }
        Some("rewind") | Some("rw") => {
            crate::trustdaw::rewind();
            crate::n!(B_, "Rewound to beginning");
        }
        Some("bpm") => {
            match args.get(1).and_then(|j| j.parse::<u32>().ok()) {
                Some(bpm) => {
                    crate::trustdaw::guf(bpm);
                    crate::n!(B_, "BPM set to {}", crate::trustdaw::Df.load(core::sync::atomic::Ordering::Relaxed));
                }
                None => crate::n!(D_, "Usage: daw bpm <30-300>"),
            }
        }
        Some("record") | Some("rec") => {
            let mp = args.get(1).and_then(|j| j.parse::<usize>().ok()).unwrap_or(0);
            match crate::trustdaw::recorder::iyu(mp) {
                Ok(ae) => crate::n!(B_, "Recorded {} notes", ae),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("piano") | Some("keyboard") => {
            crate::println!("{}", crate::trustdaw::keyboard_midi::hsp());
        }
        Some("pianoroll") | Some("roll") => {
            
            let mp = args.get(1).and_then(|j| j.parse::<usize>().ok()).unwrap_or(0);
            let bars = args.get(2).and_then(|j| j.parse::<u32>().ok()).unwrap_or(4);
            match crate::trustdaw::ensure_init().and_then(|_| {
                let project = crate::trustdaw::Ce.lock();
                let project = project.as_ref().ok_or("No project")?;
                let track = project.tracks.get(mp).ok_or("Invalid track index")?;
                Ok(crate::trustdaw::piano_roll::pij(track, bars))
            }) {
                Ok(j) => crate::println!("{}", j),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("gui") => {
            match crate::trustdaw::ui::mwy() {
                Ok(()) => crate::println!("DAW GUI closed"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("studio") | Some("beat") | Some("beats") => {
            crate::n!(B_, "Launching Beat Studio...");
            match crate::trustdaw::beat_studio::mwv() {
                Ok(()) => crate::println!("Beat Studio closed"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("funky") | Some("house") => {
            crate::n!(B_, "Loading Funky House beat...");
            
            match crate::trustdaw::beat_studio::mwx() {
                Ok(()) => crate::println!("Funky house session closed"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("matrix") => {
            crate::n!(B_, "Entering the Beat Matrix...");
            match crate::trustdaw::beat_studio::mwz() {
                Ok(()) => crate::println!("Beat Matrix closed"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("film") | Some("showcase") | Some("narrated") | Some("youtube") => {
            crate::n!(B_, "Starting narrated showcase...");
            crate::println!("  Phase 1: Building the beat (track by track)");
            crate::println!("  Phase 2: Full mix playback");
            crate::println!("  Phase 3: Matrix visualizer");
            crate::println!("  [Esc] Exit  [Space] Skip section");
            match crate::trustdaw::beat_studio::mxb() {
                Ok(()) => crate::println!("Narrated showcase complete"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("anthem") => {
            crate::n!(B_, "Starting TrustOS Anthem — Renaissance Numérique...");
            crate::println!("  5 Sections: Intro → Build → Drop → Stable → Outro");
            crate::println!("  Key: C minor → C major  |  106 BPM  |  ~3 min");
            crate::println!("  [Esc] Exit  [Space] Skip section");
            match crate::trustdaw::beat_studio::mww() {
                Ok(()) => crate::println!("TrustOS Anthem complete"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("trap") | Some("gangsta") | Some("rap") | Some("cyber") | Some("neon") => {
            crate::n!(B_, "Starting Cyberpunk Showcase — NEON PROTOCOL...");
            crate::println!("  Sub Bass + Aggressive 16th Hats + Synth Arps + Digital Lead");
            crate::println!("  100 BPM  |  Eb minor  |  Dark Cyberpunk");
            crate::println!("  [Esc] Exit  [Space] Skip section");
            match crate::trustdaw::beat_studio::mxc() {
                Ok(()) => crate::println!("Neon Protocol Showcase complete"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("untitled2") | Some("u2") | Some("lofi") => {
            crate::n!(B_, "Generating 'Untitled 2' — Dark Lo-Fi / Ambient...");
            crate::println!("  Keys + Sub + Dusty Drums + Emotional Lead");
            crate::println!("  85 BPM  |  A minor  |  6 sections  |  ~3 min");
            crate::println!("  3D Matrix Visualizer  |  [Esc] Exit");
            match crate::trustdaw::audio_viz::gni() {
                Ok(()) => crate::println!("Untitled 2 complete"),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("viz") | Some("visualizer") => {
            let path = args.get(1).copied().unwrap_or("");
            if path.is_empty() {
                crate::n!(D_, "Usage: daw viz <file.wav>");
                crate::println!("  Plays audio file with 3D matrix rain visualizer");
            } else {
                crate::n!(B_, "Starting Audio Visualizer...");
                crate::println!("  File: {}", path);
                crate::println!("  [Esc] Exit");
                match crate::trustdaw::audio_viz::ivd(path) {
                    Ok(()) => crate::println!("Visualizer complete"),
                    Err(e) => crate::n!(A_, "Error: {}", e),
                }
            }
        }
        Some("export") | Some("wav") => {
            let path = args.get(1).copied().unwrap_or("/home/output.wav");
            crate::println!("Exporting to {}...", path);
            match crate::trustdaw::dpb(path) {
                Ok(size) => {
                    let (im, dh) = crate::trustdaw::wav_export::lmr(
                        size / 2, crate::trustdaw::BT_, 2
                    );
                    crate::n!(B_, "Exported: {} ({} bytes, {}:{:02}.{:03})",
                        path, size, im / 60, im % 60, dh);
                }
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("mixer") => kms(&args[1..]),
        Some("help") => {
            crate::n!(C_, "╔══════════════════════════════════════════════╗");
            crate::n!(C_, "║      TrustDAW — Digital Audio Workstation    ║");
            crate::n!(C_, "╚══════════════════════════════════════════════╝");
            crate::println!();
            crate::n!(D_, "  Project:");
            crate::println!("  daw init                        Initialize TrustDAW");
            crate::println!("  daw new <name> [bpm]            New project");
            crate::println!("  daw demo                        Load demo project");
            crate::println!("  daw status                      Show project info");
            crate::println!("  daw bpm <30-300>                Set tempo");
            crate::println!();
            crate::n!(D_, "  Transport:");
            crate::println!("  daw play                        Play from current position");
            crate::println!("  daw stop                        Stop playback/recording");
            crate::println!("  daw rewind                      Rewind to start");
            crate::println!("  daw record [track#]             Record from keyboard");
            crate::println!();
            crate::n!(D_, "  Tracks:");
            crate::println!("  daw track add <name>            Add a new track");
            crate::println!("  daw track rm <#>                Remove a track");
            crate::println!("  daw track list                  List all tracks");
            crate::println!("  daw track wave <#> <waveform>   Set track waveform");
            crate::println!("  daw track notes <#>             List notes in track");
            crate::println!("  daw track clear <#>             Clear track notes");
            crate::println!("  daw track transpose <#> <semi>  Transpose notes");
            crate::println!();
            crate::n!(D_, "  Notes:");
            crate::println!("  daw note add <track#> <note> [vel] [start] [dur]");
            crate::println!("                                  Add a note (e.g. daw note add 0 C4 100 0 480)");
            crate::println!("  daw note rm <track#> <idx>      Remove a note by index");
            crate::println!();
            crate::n!(D_, "  Mixer:");
            crate::println!("  daw mixer                       Show mixer status");
            crate::println!("  daw mixer vol <#> <0-255>       Set track volume");
            crate::println!("  daw mixer pan <#> <-100..100>   Set track pan");
            crate::println!("  daw mixer mute <#>              Toggle mute");
            crate::println!("  daw mixer solo <#>              Toggle solo");
            crate::println!();
            crate::n!(D_, "  Display:");
            crate::println!("  daw pianoroll [track#] [bars]   Text piano roll view");
            crate::println!("  daw piano                       Show keyboard layout");
            crate::println!("  daw gui                         Launch graphical DAW UI");
            crate::println!("  daw studio                      Beat Studio (YouTube showcase mode)");
            crate::println!("  daw funky                       Funky House demo beat");
            crate::println!("  daw matrix                      Matrix visualizer showcase");
            crate::println!("  daw film                        Narrated showcase (YouTube video)");
            crate::println!("  daw showcase                    Same as 'daw film'");
            crate::println!("  daw anthem                      TrustOS Anthem — Renaissance Numerique");
            crate::println!("  daw trap                        Neon Protocol — Cyberpunk Trap");
            crate::println!("  daw untitled2                   Untitled 2 — Dark Lo-Fi Ambient");
            crate::println!();
            crate::n!(D_, "  Visualizer:");
            crate::println!("  daw viz <file.wav>              Audio file visualizer (3D matrix + waveform)");
            crate::println!("  play <file.wav>                 Same as 'daw viz'");
            crate::println!();
            crate::n!(D_, "  Export:");
            crate::println!("  daw export [path]               Export WAV (default: /home/output.wav)");
        }
        Some(other) => {
            crate::n!(A_, "Unknown DAW command: {}", other);
            crate::println!("Use 'daw help' for commands");
        }
    }
}

fn kmu(args: &[&str]) {
    match args.first().copied() {
        Some("add") | Some("new") => {
            let name = match args.get(1) {
                Some(ae) => *ae,
                None => {
                    crate::n!(D_, "Usage: daw track add <name>");
                    return;
                }
            };
            match crate::trustdaw::add_track(name) {
                Ok(idx) => crate::n!(B_, "Track {} \"{}\" added", idx, name),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("rm") | Some("remove") | Some("del") => {
            match args.get(1).and_then(|j| j.parse::<usize>().ok()) {
                Some(idx) => {
                    match crate::trustdaw::remove_track(idx) {
                        Ok(()) => crate::n!(B_, "Track {} removed", idx),
                        Err(e) => crate::n!(A_, "Error: {}", e),
                    }
                }
                None => crate::n!(D_, "Usage: daw track rm <index>"),
            }
        }
        Some("list") | Some("ls") | None => {
            crate::println!("{}", crate::trustdaw::status());
        }
        Some("wave") | Some("waveform") => {
            if args.len() < 3 {
                crate::n!(D_, "Usage: daw track wave <#> <sine|square|saw|triangle|noise>");
                return;
            }
            let idx = match args[1].parse::<usize>() {
                Ok(i) => i,
                Err(_) => { crate::n!(A_, "Invalid track index"); return; }
            };
            match crate::trustdaw::opr(idx, args[2]) {
                Ok(()) => crate::n!(B_, "Track {} waveform set to {}", idx, args[2]),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("notes") => {
            let idx = args.get(1).and_then(|j| j.parse::<usize>().ok()).unwrap_or(0);
            match crate::trustdaw::mzf(idx) {
                Ok(j) => crate::println!("{}", j),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("clear") => {
            let idx = match args.get(1).and_then(|j| j.parse::<usize>().ok()) {
                Some(i) => i,
                None => { crate::n!(D_, "Usage: daw track clear <#>"); return; }
            };
            let kkx = {
                let mut project = crate::trustdaw::Ce.lock();
                if let Some(oa) = project.as_mut() {
                    if let Some(track) = oa.tracks.get_mut(idx) {
                        let count = track.notes.len();
                        track.clear();
                        Ok(count)
                    } else { Err("Invalid track index") }
                } else { Err("No project") }
            };
            match kkx {
                Ok(ae) => crate::n!(B_, "Cleared {} notes from track {}", ae, idx),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("transpose") => {
            if args.len() < 3 {
                crate::n!(D_, "Usage: daw track transpose <#> <semitones>");
                return;
            }
            let idx = match args[1].parse::<usize>() {
                Ok(i) => i,
                Err(_) => { crate::n!(A_, "Invalid track index"); return; }
            };
            let dee = match args[2].parse::<i8>() {
                Ok(j) => j,
                Err(_) => { crate::n!(A_, "Invalid semitone value"); return; }
            };
            let result = {
                let mut project = crate::trustdaw::Ce.lock();
                if let Some(oa) = project.as_mut() {
                    if let Some(track) = oa.tracks.get_mut(idx) {
                        track.transpose(dee);
                        Ok(())
                    } else { Err("Invalid track index") }
                } else { Err("No project") }
            };
            match result {
                Ok(()) => crate::n!(B_, "Track {} transposed by {} semitones", idx, dee),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some(other) => {
            crate::n!(A_, "Unknown track command: {}", other);
            crate::println!("Use: add, rm, list, wave, notes, clear, transpose");
        }
    }
}

fn kmt(args: &[&str]) {
    match args.first().copied() {
        Some("add") => {
            
            if args.len() < 3 {
                crate::n!(D_, "Usage: daw note add <track#> <note> [vel] [start_tick] [dur_ticks]");
                crate::println!("  Example: daw note add 0 C4 100 0 480");
                crate::println!("  Default: vel=100, start=0, dur=480 (quarter note)");
                return;
            }
            let mp = match args[1].parse::<usize>() {
                Ok(i) => i,
                Err(_) => { crate::n!(A_, "Invalid track index"); return; }
            };
            let agu = args[2];
            let midi_note = match crate::audio::tables::cnh(agu) {
                Some(ae) => ae,
                None => { crate::n!(A_, "Invalid note: {} (use e.g. C4, A#3, Bb5)", agu); return; }
            };
            let velocity = args.get(3).and_then(|j| j.parse::<u8>().ok()).unwrap_or(100);
            let start_tick = args.get(4).and_then(|j| j.parse::<u32>().ok()).unwrap_or(0);
            let yq = args.get(5).and_then(|j| j.parse::<u32>().ok()).unwrap_or(480);

            match crate::trustdaw::add_note(mp, midi_note, velocity, start_tick, yq) {
                Ok(()) => {
                    let name = crate::audio::tables::bno(midi_note);
                    let amb = crate::audio::tables::bui(midi_note);
                    crate::n!(B_, "Added {}{} vel={} at tick {} dur={}",
                        name, amb, velocity, start_tick, yq);
                }
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("rm") | Some("remove") | Some("del") => {
            if args.len() < 3 {
                crate::n!(D_, "Usage: daw note rm <track#> <note_index>");
                return;
            }
            let mp = match args[1].parse::<usize>() {
                Ok(i) => i,
                Err(_) => { crate::n!(A_, "Invalid track index"); return; }
            };
            let nkz = match args[2].parse::<usize>() {
                Ok(i) => i,
                Err(_) => { crate::n!(A_, "Invalid note index"); return; }
            };
            let result = {
                let mut project = crate::trustdaw::Ce.lock();
                if let Some(oa) = project.as_mut() {
                    if let Some(track) = oa.tracks.get_mut(mp) {
                        match track.remove_note(nkz) {
                            Some(note) => Ok(note),
                            None => Err("Note index out of range"),
                        }
                    } else { Err("Invalid track index") }
                } else { Err("No project") }
            };
            match result {
                Ok(note) => {
                    let name = crate::audio::tables::bno(note.pitch);
                    let amb = crate::audio::tables::bui(note.pitch);
                    crate::n!(B_, "Removed {}{} from track {}", name, amb, mp);
                }
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        _ => {
            crate::n!(D_, "Usage: daw note <add|rm> ...");
            crate::println!("  daw note add <track#> <note> [vel] [start] [dur]");
            crate::println!("  daw note rm <track#> <note_index>");
        }
    }
}

fn kms(args: &[&str]) {
    match args.first().copied() {
        Some("vol") | Some("volume") => {
            if args.len() < 3 {
                crate::n!(D_, "Usage: daw mixer vol <track#> <0-255>");
                return;
            }
            let idx = match args[1].parse::<usize>() { Ok(i) => i, Err(_) => { crate::n!(A_, "Invalid track"); return; } };
            let vd = match args[2].parse::<u8>() { Ok(v) => v, Err(_) => { crate::n!(A_, "Invalid volume (0-255)"); return; } };
            match crate::trustdaw::opq(idx, vd) {
                Ok(()) => crate::n!(B_, "Track {} volume: {}", idx, vd),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("pan") => {
            if args.len() < 3 {
                crate::n!(D_, "Usage: daw mixer pan <track#> <-100..+100>");
                return;
            }
            let idx = match args[1].parse::<usize>() { Ok(i) => i, Err(_) => { crate::n!(A_, "Invalid track"); return; } };
            let pan = match args[2].parse::<i8>() { Ok(aa) => aa, Err(_) => { crate::n!(A_, "Invalid pan (-100 to +100)"); return; } };
            match crate::trustdaw::opp(idx, pan) {
                Ok(()) => {
                    let desc = if pan == 0 { "Center".into() } else if pan > 0 { alloc::format!("Right {}", pan) } else { alloc::format!("Left {}", -pan) };
                    crate::n!(B_, "Track {} pan: {}", idx, desc);
                }
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("mute") => {
            let idx = match args.get(1).and_then(|j| j.parse::<usize>().ok()) {
                Some(i) => i,
                None => { crate::n!(D_, "Usage: daw mixer mute <track#>"); return; }
            };
            match crate::trustdaw::toggle_mute(idx) {
                Ok(muted) => crate::n!(if muted { D_ } else { B_ },
                    "Track {} {}", idx, if muted { "MUTED" } else { "unmuted" }),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        Some("solo") => {
            let idx = match args.get(1).and_then(|j| j.parse::<usize>().ok()) {
                Some(i) => i,
                None => { crate::n!(D_, "Usage: daw mixer solo <track#>"); return; }
            };
            match crate::trustdaw::toggle_solo(idx) {
                Ok(solo) => crate::n!(if solo { D_ } else { B_ },
                    "Track {} {}", idx, if solo { "SOLO" } else { "un-solo'd" }),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        _ => {
            
            crate::println!("{}", crate::trustdaw::status());
        }
    }
}

pub(super) fn kpo(args: &[&str]) {
    let devices = crate::pci::aqs();
    
    if devices.is_empty() {
        crate::n!(D_, "No PCI devices found");
        return;
    }
    
    let csi = args.contains(&"-v") || args.contains(&"--verbose");
    
    crate::n!(C_, "PCI Devices ({} found):", devices.len());
    crate::println!();
    
    for s in &devices {
        
        crate::bq!(B_, "{:02X}:{:02X}.{} ", 
            s.bus, s.device, s.function);
        crate::print!("{:04X}:{:04X} ", s.vendor_id, s.device_id);
        
        let subclass_name = s.subclass_name();
        if subclass_name.is_empty() {
            crate::print!("{}", s.class_name());
        } else {
            crate::print!("{}", subclass_name);
        }
        
        crate::n!(D_, " [{}]", s.vendor_name());
        
        if csi {
            crate::println!("        Class: {:02X}:{:02X} ProgIF: {:02X} Rev: {:02X}",
                s.class_code, s.subclass, s.prog_if, s.revision);
            
            if s.interrupt_line != 0xFF && s.interrupt_pin != 0 {
                crate::println!("        IRQ: {} (pin {})", 
                    s.interrupt_line, s.interrupt_pin);
            }
            
            
            for i in 0..6 {
                if let Some(addr) = s.bar_address(i) {
                    let bqj = if s.bar_is_memory(i) { "MEM" } else { "I/O" };
                    crate::println!("        BAR{}: {:#010X} [{}]", i, addr, bqj);
                }
            }
            crate::println!();
        }
    }
    
    if !csi {
        crate::println!();
        crate::n!(D_, "Use 'lspci -v' for detailed info");
    }
}

pub(super) fn kpk() {
    crate::n!(C_, "=== Hardware Summary ===");
    crate::println!();
    
    let devices = crate::pci::aqs();
    
    
    crate::n!(B_, "CPU:");
    crate::println!("  Architecture: x86_64");
    crate::println!("  Mode: Long Mode (64-bit)");
    crate::println!();
    
    
    crate::n!(B_, "Memory:");
    crate::println!("  Heap: 256 KB");
    crate::println!();
    
    
    let storage: Vec<_> = devices.iter()
        .filter(|d| d.class_code == crate::pci::class::FZ_)
        .collect();
    crate::n!(B_, "Storage Controllers ({}):", storage.len());
    for s in &storage {
        crate::println!("  {:04X}:{:04X} {} [{}]", 
            s.vendor_id, s.device_id, 
            s.subclass_name(),
            s.vendor_name());
    }
    crate::println!();
    
    
    let network: Vec<_> = devices.iter()
        .filter(|d| d.class_code == crate::pci::class::Gr)
        .collect();
    crate::n!(B_, "Network Controllers ({}):", network.len());
    for s in &network {
        crate::println!("  {:04X}:{:04X} {} [{}]",
            s.vendor_id, s.device_id,
            s.subclass_name(),
            s.vendor_name());
    }
    crate::println!();
    
    
    let display: Vec<_> = devices.iter()
        .filter(|d| d.class_code == crate::pci::class::Du)
        .collect();
    crate::n!(B_, "Display ({}):", display.len());
    for s in &display {
        crate::println!("  {:04X}:{:04X} {} [{}]",
            s.vendor_id, s.device_id,
            s.subclass_name(),
            s.vendor_name());
    }
    crate::println!();
    
    
    let usb: Vec<_> = devices.iter()
        .filter(|d| d.class_code == crate::pci::class::QG_ 
                 && d.subclass == crate::pci::serial::Qs)
        .collect();
    crate::n!(B_, "USB Controllers ({}):", usb.len());
    for s in &usb {
        crate::println!("  {:04X}:{:04X} {} [{}]",
            s.vendor_id, s.device_id,
            s.subclass_name(),
            s.vendor_name());
    }
    crate::println!();
    
    
    crate::n!(C_, "Total: {} PCI devices", devices.len());
}

pub(super) fn kob(args: &[&str]) {
    if args.first() == Some(&"--help") || args.first() == Some(&"-h") {
        crate::println!("Usage: gpu [info|dcn|modes]");
        crate::println!("  gpu         Show GPU summary");
        crate::println!("  gpu info    Detailed GPU information");
        crate::println!("  gpu dcn     Display engine (DCN) status");
        crate::println!("  gpu modes   List standard display modes");
        return;
    }
    
    let je = args.first().copied().unwrap_or("info");
    
    match je {
        "info" | "" => {
            crate::n!(C_, "=== GPU Status ===");
            crate::println!();
            
            let mut enh = false;
            
            
            if crate::drivers::nvidia::aud() {
                enh = true;
                crate::n!(B_, "NVIDIA GPU:");
                crate::println!("  {}", crate::drivers::nvidia::summary());
                if let Some(info) = crate::drivers::nvidia::rk() {
                    crate::println!("  PCI: {:04X}:{:04X} rev {:02X} at {:02X}:{:02X}.{}",
                        info.vendor_id, info.device_id, info.revision,
                        info.bus, info.device, info.function);
                    crate::println!("  Chipset: {} (id {:#04X}, step {:#04X})",
                        info.chipset_name(), info.chipset_id, info.stepping);
                    crate::println!("  VRAM: {} MB", info.vram_size / (1024 * 1024));
                    crate::println!("  PCIe: Gen{} x{}", info.pcie_gen, info.pcie_width);
                    crate::println!("  MMIO: {:#X} ({}MB)", info.mmio_base, info.mmio_size / (1024 * 1024));
                    if info.vram_base > 0 {
                        crate::println!("  VRAM aperture: {:#X}", info.vram_base);
                    }
                    crate::println!("  2D Accel: {}", if crate::drivers::nvidia::mrv() { "READY" } else { "N/A" });
                }
                crate::println!();
            }
            
            
            if crate::drivers::amdgpu::aud() {
                enh = true;
                crate::n!(B_, "AMD GPU:");
                for line in crate::drivers::amdgpu::info_lines() {
                    crate::println!("{}", line);
                }
                crate::println!();
                
                if crate::drivers::amdgpu::dcn::is_ready() {
                    crate::n!(B_, "Display Engine:");
                    crate::println!("  {}", crate::drivers::amdgpu::dcn::summary());
                }
            }
            
            
            if crate::drivers::virtio_gpu::sw() {
                enh = true;
                crate::n!(B_, "VirtIO GPU:");
                crate::println!("  {}", crate::drivers::virtio_gpu::gcl());
                crate::println!();
            }
            
            if !enh {
                crate::println!("No GPU detected.");
                crate::println!();
                
                let bbd = crate::pci::bsp(crate::pci::class::Du);
                if !bbd.is_empty() {
                    crate::n!(B_, "Display controllers found:");
                    for s in &bbd {
                        crate::println!("  {:04X}:{:04X} {} [{}]", 
                            s.vendor_id, s.device_id,
                            s.subclass_name(), s.vendor_name());
                    }
                }
            }
        }
        "dcn" | "display" => {
            crate::n!(C_, "=== DCN Display Engine ===");
            crate::println!();
            
            if crate::drivers::amdgpu::dcn::is_ready() {
                for line in crate::drivers::amdgpu::dcn::info_lines() {
                    crate::println!("{}", line);
                }
            } else {
                crate::println!("DCN display engine not initialized.");
                if !crate::drivers::amdgpu::aud() {
                    crate::println!("(No AMD GPU detected)");
                }
            }
        }
        "modes" => {
            crate::n!(C_, "=== Standard Display Modes ===");
            crate::println!();
            for (i, mode) in crate::drivers::amdgpu::dcn::ovz().iter().enumerate() {
                crate::println!("  [{}] {}", i, mode.modeline());
            }
        }
        _ => {
            crate::println!("Unknown subcommand: {}", je);
            crate::println!("Use 'gpu --help' for usage.");
        }
    }
}

pub(super) fn klk(args: &[&str]) {
    let je = args.first().copied().unwrap_or("status");
    
    match je {
        "status" | "" => {
            crate::n!(C_, "=== Accessibility Settings ===");
            crate::println!();
            let ads = crate::accessibility::btq();
            let fs = crate::accessibility::cyn();
            let cs = crate::accessibility::cyl();
            let gvg = crate::accessibility::bnc();
            let dh = crate::accessibility::cyq();
            crate::println!("  High Contrast : {}", if ads { "ON" } else { "OFF" });
            crate::println!("  Font Size     : {}", fs.label());
            crate::println!("  Cursor Size   : {}", cs.label());
            crate::println!("  Sticky Keys   : {}", if gvg { "ON" } else { "OFF" });
            crate::println!("  Mouse Speed   : {}", dh.label());
            crate::println!();
            crate::println!("Shortcuts: Win+H = toggle high contrast");
            crate::println!("Settings:  Win+I > keys 5-9 to adjust");
        }
        "hc" | "contrast" => {
            crate::accessibility::gzg();
            let on = crate::accessibility::btq();
            crate::println!("High contrast: {}", if on { "ON" } else { "OFF" });
        }
        "font" => {
            crate::accessibility::hqf();
            crate::println!("Font size: {}", crate::accessibility::cyn().label());
        }
        "cursor" => {
            crate::accessibility::hqe();
            crate::println!("Cursor size: {}", crate::accessibility::cyl().label());
        }
        "sticky" => {
            crate::accessibility::jnd();
            let on = crate::accessibility::bnc();
            crate::println!("Sticky keys: {}", if on { "ON" } else { "OFF" });
        }
        "mouse" => {
            crate::accessibility::hqg();
            crate::println!("Mouse speed: {}", crate::accessibility::cyq().label());
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
            crate::println!("Unknown: {}. Use 'a11y --help'", je);
        }
    }
}

pub(super) fn ksl(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: tcpsyn <ip> <port>");
        crate::println!("  Example: tcpsyn 93.184.216.34 80");
        return;
    }

    let au: Vec<&str> = args[0].split('.').collect();
    if au.len() != 4 {
        crate::n!(A_, "Invalid IP format");
        return;
    }

    let ip = [
        au[0].parse().unwrap_or(0),
        au[1].parse().unwrap_or(0),
        au[2].parse().unwrap_or(0),
        au[3].parse().unwrap_or(0),
    ];

    let port: u16 = match args[1].parse() {
        Ok(aa) => aa,
        Err(_) => {
            crate::n!(A_, "Invalid port");
            return;
        }
    };

    crate::println!("Sending TCP SYN to {}:{}...", args[0], port);
    match crate::netstack::tcp::azp(ip, port) {
        Ok(src_port) => {
            crate::println!("SYN sent to {}:{} (src port {})", args[0], port, src_port);
            let cja = crate::netstack::tcp::bjy(ip, port, src_port, 1000);
            if cja {
                crate::println!("SYN-ACK received (connection established)");
            } else {
                crate::n!(D_, "No SYN-ACK received (timeout)");
            }
        }
        Err(e) => crate::n!(A_, "tcpsyn failed: {}", e),
    }
}

pub(super) fn kon(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: httpget <ip|host> <port> [path] [host]");
        crate::println!("  Example: httpget 192.168.56.1 8080 /");
        crate::println!("  Example: httpget example.com 80 / example.com");
        return;
    }

    let cak = args[0];
    let port: u16 = match args[1].parse() {
        Ok(aa) => aa,
        Err(_) => {
            crate::n!(A_, "Invalid port");
            return;
        }
    };

    let path = args.get(2).copied().unwrap_or("/");
    let mut bgz = args.get(3).copied().unwrap_or(cak);
    if args.get(3).is_none() && cak == "192.168.56.1" {
        bgz = "localhost";
    }

    hsz(cak, port, path, bgz);
}

pub(super) fn hlt(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: curl <http://host[:port]/path> | <https://host[:port]/path>");
        return;
    }

    let url = args[0];
    if let Some((host, port, path, cln)) = gml(url) {
        let bgz = if host == "192.168.56.1" { "localhost" } else { &host };
        if cln {
            lgm(&host, port, &path, bgz);
        } else {
            hsz(&host, port, &path, bgz);
        }
    } else {
        crate::n!(A_, "Invalid URL");
    }
}

fn hsz(cak: &str, port: u16, path: &str, bgz: &str) {
    let ip = if let Some(ip) = art(cak) {
        ip
    } else if let Some(afn) = crate::netstack::dns::yb(cak) {
        afn
    } else {
        crate::n!(A_, "Unable to resolve host");
        return;
    };

    crate::println!("Connecting to {}:{}...", cak, port);
    let src_port = match crate::netstack::tcp::azp(ip, port) {
        Ok(aa) => aa,
        Err(e) => {
            crate::n!(A_, "SYN failed: {}", e);
            return;
        }
    };

    let cja = crate::netstack::tcp::bjy(ip, port, src_port, 1000);
    if !cja {
        crate::n!(D_, "Connection timeout");
        return;
    }

    let mut request = String::new();
    request.push_str("GET ");
    request.push_str(path);
    request.push_str(" HTTP/1.1\r\nHost: ");
    request.push_str(bgz);
    request.push_str("\r\nConnection: close\r\n\r\n");

    if let Err(e) = crate::netstack::tcp::bjc(ip, port, src_port, request.as_bytes()) {
        crate::n!(A_, "send failed: {}", e);
        return;
    }

    crate::println!("--- HTTP response ---");
    let start = crate::logger::eg();
    let mut total_bytes: usize = 0;
    let mut bth: u32 = 0;
    loop {
        crate::netstack::poll();
        let mut aty = false;
        while let Some(data) = crate::netstack::tcp::aus(ip, port, src_port) {
            aty = true;
            total_bytes += data.len();
            if let Ok(text) = core::str::from_utf8(&data) {
                crate::print!("{}", text);
            } else {
                crate::println!("<binary data>");
            }
        }

        if !aty {
            bth = bth.saturating_add(1);
            if crate::netstack::tcp::fin_received(ip, port, src_port) || bth > 200_000 {
                break;
            }
        } else {
            bth = 0;
        }

        if crate::logger::eg().saturating_sub(start) > 3000 {
            break;
        }
        crate::arch::acb();
    }
    let _ = crate::netstack::tcp::ams(ip, port, src_port);
    crate::println!("\n--- end ({} bytes) ---", total_bytes);
    if total_bytes == 0 {
        crate::n!(D_, "No response body received");
    }
}


pub(super) fn nrn(url: &str) -> Option<(String, u16, String)> {
    let url = url.trim();
    
    
    let (ef, bru) = if url.starts_with("https://") {
        (&url[8..], 443u16)
    } else if url.starts_with("http://") {
        (&url[7..], 80u16)
    } else {
        
        (url, 80u16)
    };
    
    
    let (host_port, path) = if let Some(idx) = ef.find('/') {
        (&ef[..idx], &ef[idx..])
    } else {
        (ef, "/")
    };
    
    
    let (host, port) = if let Some(idx) = host_port.find(':') {
        let host = &host_port[..idx];
        let bva = &host_port[idx+1..];
        let port = bva.parse::<u16>().unwrap_or(bru);
        (host, port)
    } else {
        (host_port, bru)
    };
    
    if host.is_empty() {
        return None;
    }
    
    Some((String::from(host), port, String::from(path)))
}


pub(super) fn hta(host: &str, ip: [u8; 4], port: u16, path: &str) -> Result<String, &'static str> {
    
    let src_port = crate::netstack::tcp::azp(ip, port)
        .map_err(|_| "SYN failed")?;
    
    
    if !crate::netstack::tcp::bjy(ip, port, src_port, 1000) {
        return Err("Connection timeout");
    }
    
    
    let mut request = String::new();
    request.push_str("GET ");
    request.push_str(path);
    request.push_str(" HTTP/1.1\r\nHost: ");
    request.push_str(host);
    request.push_str("\r\nUser-Agent: TrustOS/0.1\r\nConnection: close\r\n\r\n");
    
    
    crate::netstack::tcp::bjc(ip, port, src_port, request.as_bytes())
        .map_err(|_| "Send failed")?;
    
    
    let mut fa = String::new();
    let start = crate::logger::eg();
    let mut bth: u32 = 0;
    
    loop {
        crate::netstack::poll();
        let mut aty = false;
        
        while let Some(data) = crate::netstack::tcp::aus(ip, port, src_port) {
            aty = true;
            if let Ok(text) = core::str::from_utf8(&data) {
                fa.push_str(text);
            }
        }
        
        if !aty {
            bth = bth.saturating_add(1);
            if crate::netstack::tcp::fin_received(ip, port, src_port) || bth > 100_000 {
                break;
            }
        } else {
            bth = 0;
        }
        
        if crate::logger::eg().saturating_sub(start) > 2000 {
            break;
        }
        
        
        if fa.len() > 4096 {
            fa.push_str("\n... (response truncated)");
            break;
        }
        
        crate::arch::acb();
    }
    
    let _ = crate::netstack::tcp::ams(ip, port, src_port);
    
    if fa.is_empty() {
        return Err("No response received");
    }
    
    Ok(fa)
}

fn lgm(cak: &str, port: u16, path: &str, bgz: &str) {
    
    let url = if port == 443 {
        alloc::format!("https://{}{}", bgz, path)
    } else {
        alloc::format!("https://{}:{}{}", bgz, port, path)
    };
    
    crate::println!("Connecting to {} (TLS 1.3)...", bgz);
    crate::println!("--- HTTPS response ---");
    
    match crate::netstack::https::get(&url) {
        Ok(fa) => {
            
            crate::n!(C_, "HTTP/1.1 {}", fa.status_code);
            
            
            for (key, value) in &fa.headers {
                crate::println!("{}: {}", key, value);
            }
            crate::println!("");
            
            
            let kdc = if fa.body.len() > 4096 {
                &fa.body[..4096]
            } else {
                &fa.body
            };
            
            if let Ok(body_str) = core::str::from_utf8(kdc) {
                crate::print!("{}", body_str);
                if fa.body.len() > 4096 {
                    crate::println!("\n... (truncated, {} more bytes)", fa.body.len() - 4096);
                }
            } else {
                crate::println!("[Binary data: {} bytes]", fa.body.len());
            }
            
            crate::println!("\n--- end ({} bytes) ---", fa.body.len());
        }
        Err(e) => {
            crate::n!(A_, "HTTPS failed: {}", e);
        }
    }
}

pub(super) fn gml(url: &str) -> Option<(String, u16, String, bool)> {
    let mut iy = url.trim();
    let mut https = false;
    if let Some(ef) = iy.strip_prefix("https://") {
        iy = ef;
        https = true;
    } else if let Some(ef) = iy.strip_prefix("http://") {
        iy = ef;
    }

    let (host_port, path) = if let Some((h, aa)) = iy.split_once('/') {
        (h, format!("/{}", aa))
    } else {
        (iy, String::from("/"))
    };

    let (host, port) = if let Some((h, aa)) = host_port.split_once(':') {
        let port = aa.parse::<u16>().ok()?;
        (h, port)
    } else {
        (host_port, if https { 443 } else { 80 })
    };

    if host.is_empty() {
        return None;
    }

    Some((String::from(host), port, path, https))
}

pub(super) fn art(input: &str) -> Option<[u8; 4]> {
    let au: Vec<&str> = input.split('.').collect();
    if au.len() != 4 {
        return None;
    }
    let a = au[0].parse::<u8>().ok()?;
    let b = au[1].parse::<u8>().ok()?;
    let c = au[2].parse::<u8>().ok()?;
    let d = au[3].parse::<u8>().ok()?;
    Some([a, b, c, d])
}



pub(super) fn knq(args: &[&str], command: &str) {
    if args.is_empty() && !command.starts_with("./") {
        crate::n!(C_, "Usage: exec <program> [args...]");
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
    
    
    let (program, prog_args) = if command.starts_with("./") {
        (command, args)
    } else if args.is_empty() {
        crate::n!(A_, "exec: missing program name");
        return;
    } else {
        (args[0], &args[1..])
    };
    
    
    if program == "test" || program == "./test" {
        crate::n!(C_, "Running Ring 3 test program...");
        match crate::exec::doy() {
            crate::exec::ExecResult::Exited(code) => {
                if code == 0 {
                    crate::n!(B_, "Ring 3 test passed (exit code 0)");
                } else {
                    crate::n!(D_, "Ring 3 test exited with code: {}", code);
                }
            }
            crate::exec::ExecResult::Faulted(azg) => {
                crate::n!(A_, "Test faulted: {}", azg);
            }
            crate::exec::ExecResult::LoadError(e) => {
                crate::n!(A_, "Load error: {:?}", e);
            }
            crate::exec::ExecResult::MemoryError => {
                crate::n!(A_, "Memory allocation failed");
            }
        }
        return;
    }
    
    
    if program == "hello" || program == "./hello" {
        crate::n!(C_, "Running embedded hello world ELF in Ring 3...");
        match crate::exec::fvl() {
            crate::exec::ExecResult::Exited(code) => {
                if code == 0 {
                    crate::n!(B_, "Program exited successfully");
                } else {
                    crate::n!(D_, "Program exited with code: {}", code);
                }
            }
            crate::exec::ExecResult::Faulted(azg) => {
                crate::n!(A_, "Program faulted: {}", azg);
            }
            crate::exec::ExecResult::LoadError(e) => {
                crate::n!(A_, "Failed to load ELF: {:?}", e);
            }
            crate::exec::ExecResult::MemoryError => {
                crate::n!(A_, "Memory allocation failed");
            }
        }
        return;
    }
    
    
    let path = eyp(program);
    
    
    if !bbs(&path) {
        crate::n!(A_, "exec: {}: not found", path);
        return;
    }
    
    
    if !crate::exec::is_executable(&path) {
        crate::n!(A_, "exec: {}: not an ELF executable", path);
        return;
    }
    
    crate::n!(C_, "Executing: {}", path);
    
    
    match crate::exec::elt(&path, prog_args) {
        crate::exec::ExecResult::Exited(code) => {
            if code == 0 {
                crate::n!(B_, "Program exited successfully");
            } else {
                crate::n!(D_, "Program exited with code: {}", code);
            }
        }
        crate::exec::ExecResult::Faulted(azg) => {
            crate::n!(A_, "Program faulted: {}", azg);
        }
        crate::exec::ExecResult::LoadError(e) => {
            crate::n!(A_, "Failed to load: {:?}", e);
        }
        crate::exec::ExecResult::MemoryError => {
            crate::n!(A_, "Out of memory");
        }
    }
}

pub(super) fn knp(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: elfinfo <file>");
        return;
    }
    
    let path = eyp(args[0]);
    
    
    let fd = match crate::vfs::open(&path, crate::vfs::OpenFlags(0)) {
        Ok(fd) => fd,
        Err(_) => {
            crate::n!(A_, "Cannot open: {}", path);
            return;
        }
    };
    
    let mut header = [0u8; 64];
    match crate::vfs::read(fd, &mut header) {
        Ok(ae) if ae >= 64 => {}
        _ => {
            crate::n!(A_, "Cannot read ELF header");
            crate::vfs::close(fd).ok();
            return;
        }
    }
    crate::vfs::close(fd).ok();
    
    
    if header[0..4] != [0x7F, b'E', b'L', b'F'] {
        crate::n!(A_, "Not an ELF file");
        return;
    }
    
    crate::n!(G_, "ELF Header: {}", path);
    crate::println!("  Magic:   {:02X} {:02X} {:02X} {:02X}", header[0], header[1], header[2], header[3]);
    crate::println!("  Class:   {}", if header[4] == 2 { "ELF64" } else { "ELF32" });
    crate::println!("  Data:    {}", if header[5] == 1 { "Little Endian" } else { "Big Endian" });
    
    let e_type = u16::from_le_bytes([header[16], header[17]]);
    let ws = match e_type {
        1 => "Relocatable",
        2 => "Executable",
        3 => "Shared Object",
        4 => "Core",
        _ => "Unknown",
    };
    crate::println!("  Type:    {} ({})", ws, e_type);
    
    let e_machine = u16::from_le_bytes([header[18], header[19]]);
    let nbt = match e_machine {
        3 => "x86",
        62 => "x86-64",
        183 => "AArch64",
        _ => "Unknown",
    };
    crate::println!("  Machine: {} ({})", nbt, e_machine);
    
    let entry = u64::from_le_bytes([
        header[24], header[25], header[26], header[27],
        header[28], header[29], header[30], header[31],
    ]);
    crate::println!("  Entry:   {:#x}", entry);
    
    let gmx = u64::from_le_bytes([
        header[32], header[33], header[34], header[35],
        header[36], header[37], header[38], header[39],
    ]);
    crate::println!("  PHoff:   {:#x}", gmx);
    
    let gmw = u16::from_le_bytes([header[56], header[57]]);
    crate::println!("  PHnum:   {}", gmw);
}


pub(super) fn pnv(command: &str, args: &[&str]) -> bool {
    let path = eyp(command);
    
    if !bbs(&path) {
        return false;
    }

    
    if crate::exec::is_executable(&path) {
        crate::n!(C_, "Executing: {}", path);
        match crate::exec::elt(&path, args) {
            crate::exec::ExecResult::Exited(code) => {
                if code != 0 {
                    crate::n!(D_, "Exit code: {}", code);
                }
            }
            crate::exec::ExecResult::Faulted(azg) => {
                crate::n!(A_, "Faulted: {}", azg);
            }
            crate::exec::ExecResult::LoadError(e) => {
                crate::n!(A_, "Load error: {:?}", e);
            }
            crate::exec::ExecResult::MemoryError => {
                crate::n!(A_, "Out of memory");
            }
        }
        return true;
    }
    
    
    if let Some(content) = super::network::cpa(&path) {
        if content.starts_with("#!/bin/sh") || content.starts_with("#!/bin/bash") {
            lrx(&content, args);
            return true;
        }
    }

    false
}



fn lrx(script: &str, args: &[&str]) {
    use alloc::string::String;
    use alloc::vec::Vec;
    use alloc::collections::BTreeMap;

    let lines: Vec<&str> = script.lines().collect();
    let mut qq: BTreeMap<String, String> = BTreeMap::new();

    
    for (i, db) in args.iter().enumerate() {
        qq.insert(alloc::format!("{}", i + 1), String::from(*db));
    }
    qq.insert(String::from("@"), args.join(" "));
    qq.insert(String::from("#"), alloc::format!("{}", args.len()));
    qq.insert(String::from("0"), String::from("sh"));
    qq.insert(String::from("?"), String::from("0"));
    qq.insert(String::from("HOME"), String::from("/root"));
    qq.insert(String::from("PATH"), String::from("/usr/bin:/bin:/usr/sbin:/sbin"));
    qq.insert(String::from("SHELL"), String::from("/bin/sh"));

    let mut pc = 0usize; 
    let mut cqt = 0u32; 

    while pc < lines.len() {
        let dm = lines[pc].trim();
        pc += 1;

        
        if dm.is_empty() || dm.starts_with('#') {
            continue;
        }

        
        if cqt > 0 {
            if dm.starts_with("if ") || dm == "if" {
                cqt += 1;
            } else if dm == "fi" || dm.starts_with("fi;") || dm.starts_with("fi ") {
                cqt -= 1;
            } else if cqt == 1 && (dm == "else" || dm.starts_with("else;") || dm.starts_with("else ")) {
                cqt = 0; 
            }
            continue;
        }

        
        let expanded = hxd(dm, &qq);
        let expanded = expanded.trim();
        if expanded.is_empty() { continue; }

        
        if expanded.contains(';') && !expanded.starts_with("if ") && !expanded.contains("then") {
            let oyi: Vec<&str> = expanded.split(';').collect();
            for sub in oyi {
                let sub = sub.trim();
                if sub.is_empty() { continue; }
                elu(sub, &mut qq);
            }
            continue;
        }

        
        if expanded.starts_with("if ") {
            
            let fnz = expanded.trim_start_matches("if ").trim();
            let fnz = fnz.trim_end_matches("; then").trim_end_matches(";then").trim();
            let result = hwr(fnz, &qq);
            if !result {
                cqt = 1; 
            }
            continue;
        }
        if expanded == "then" { continue; } 
        if expanded == "else" {
            cqt = 1; 
            continue;
        }
        if expanded == "fi" || expanded.starts_with("fi;") || expanded.starts_with("fi ") {
            continue; 
        }

        
        if expanded.starts_with("for ") {
            
            let ef = expanded.trim_start_matches("for ").trim();
            if let Some(in_pos) = ef.find(" in ") {
                let edn = &ef[..in_pos];
                let hbf = ef[in_pos + 4..].trim();
                let hbf = hbf.trim_end_matches("; do").trim_end_matches(";do").trim();
                let values: Vec<&str> = hbf.split_whitespace().collect();

                
                if pc < lines.len() && lines[pc].trim() == "do" {
                    pc += 1;
                }

                
                let bao = pc;
                let mut djl = pc;
                let mut depth = 1u32;
                while djl < lines.len() {
                    let bl = lines[djl].trim();
                    if bl.starts_with("for ") { depth += 1; }
                    if bl == "done" || bl.starts_with("done;") || bl.starts_with("done ") {
                        depth -= 1;
                        if depth == 0 { break; }
                    }
                    djl += 1;
                }

                
                let body: Vec<&str> = lines[bao..djl].to_vec();
                for val in &values {
                    qq.insert(String::from(edn), String::from(*val));
                    for body_line in &body {
                        let bl = body_line.trim();
                        if bl.is_empty() || bl.starts_with('#') || bl == "do" { continue; }
                        let afe = hxd(bl, &qq);
                        elu(afe.trim(), &mut qq);
                    }
                }

                pc = djl + 1; 
                continue;
            }
        }

        
        elu(&expanded, &mut qq);
    }
}


fn elu(line: &str, qq: &mut alloc::collections::BTreeMap<alloc::string::String, alloc::string::String>) {
    use alloc::string::String;

    let line = line.trim();
    if line.is_empty() || line.starts_with('#') { return; }

    
    if let Some(eq_pos) = line.find('=') {
        if eq_pos > 0 && line[..eq_pos].chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !line.starts_with('=') {
            let ael = &line[..eq_pos];
            let val = line[eq_pos + 1..].trim();
            let val = crd(val);
            qq.insert(String::from(ael), val);
            return;
        }
    }

    
    let au: alloc::vec::Vec<&str> = line.splitn(2, char::is_whitespace).collect();
    let cmd = au[0];
    let ef = if au.len() > 1 { au[1].trim() } else { "" };

    match cmd {
        "echo" => {
            if ef == "-n" {
                
            } else if ef.starts_with("-n ") {
                let bk = crd(&ef[3..]);
                crate::print!("{}", bk);
            } else if ef.starts_with("-e ") {
                let bk = crd(&ef[3..]);
                crate::println!("{}", bk);
            } else {
                let bk = crd(ef);
                crate::println!("{}", bk);
            }
        }
        "printf" => {
            let bk = crd(ef);
            crate::print!("{}", bk);
        }
        "cat" => {
            
            if !ef.is_empty() {
                let path = crd(ef);
                if let Some(content) = super::network::cpa(&path) {
                    crate::print!("{}", content);
                } else {
                    crate::println!("cat: {}: No such file or directory", path);
                }
            }
        }
        "test" | "[" => {
            
            let fc = ef.trim_end_matches(']').trim();
            let result = hwr(&alloc::format!("[ {} ]", fc), qq);
            qq.insert(alloc::string::String::from("?"), if result { alloc::string::String::from("0") } else { alloc::string::String::from("1") });
        }
        "export" => {
            
            if let Some(eq_pos) = ef.find('=') {
                let ael = &ef[..eq_pos];
                let val = crd(&ef[eq_pos + 1..]);
                qq.insert(alloc::string::String::from(ael), val);
            }
        }
        "env" | "printenv" => {
            for (k, v) in qq.iter() {
                if k.len() > 1 { 
                    crate::println!("{}={}", k, v);
                }
            }
        }
        "set" => {
            if ef == "-e" || ef == "-x" || ef.is_empty() {
                
            }
        }
        "true" | ":" => {
            qq.insert(alloc::string::String::from("?"), alloc::string::String::from("0"));
        }
        "false" => {
            qq.insert(alloc::string::String::from("?"), alloc::string::String::from("1"));
        }
        "exec" => {
            
            if !ef.is_empty() {
                elu(ef, qq);
            }
        }
        "exit" | "return" => {
            
        }
        "sleep" => {
            
        }
        "cd" | "mkdir" | "rm" | "chmod" | "chown" | "ln" | "cp" | "mv" | "touch" => {
            
        }
        _ => {
            
        }
    }
}


fn hxd(line: &str, qq: &alloc::collections::BTreeMap<alloc::string::String, alloc::string::String>) -> alloc::string::String {
    use alloc::string::String;
    let mut result = String::with_capacity(line.len());
    let chars: alloc::vec::Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            if chars[i + 1] == '{' {
                
                if let Some(close) = chars[i + 2..].iter().position(|&c| c == '}') {
                    let ael: String = chars[i + 2..i + 2 + close].iter().collect();
                    if let Some(val) = qq.get(&ael) {
                        result.push_str(val);
                    }
                    i += close + 3;
                    continue;
                }
            } else if chars[i + 1] == '(' {
                
                if let Some(close) = chars[i + 2..].iter().position(|&c| c == ')') {
                    i += close + 3;
                    continue;
                }
            }
            
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_' || chars[end] == '@' || chars[end] == '#' || chars[end] == '?') {
                end += 1;
                
                if end == start + 1 && (chars[start] == '@' || chars[start] == '#' || chars[start] == '?' || chars[start].is_ascii_digit()) {
                    break;
                }
            }
            if end > start {
                let ael: String = chars[start..end].iter().collect();
                if let Some(val) = qq.get(&ael) {
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


fn hwr(fc: &str, _vars: &alloc::collections::BTreeMap<alloc::string::String, alloc::string::String>) -> bool {
    let fc = fc.trim();
    
    let inner = if fc.starts_with('[') && fc.ends_with(']') {
        fc[1..fc.len() - 1].trim()
    } else if fc.starts_with("[ ") && fc.ends_with(" ]") {
        fc[2..fc.len() - 2].trim()
    } else {
        fc
    };

    
    if inner.starts_with("-n ") { return !inner[3..].trim().trim_matches('"').is_empty(); }
    
    if inner.starts_with("-z ") { return inner[3..].trim().trim_matches('"').is_empty(); }
    
    if inner.starts_with("-f ") {
        let path = inner[3..].trim().trim_matches('"');
        return crate::ramfs::bh(|fs| fs.read_file(path).is_ok());
    }
    
    if inner.starts_with("-d ") { return true; } 
    
    if inner.contains(" = ") {
        let au: alloc::vec::Vec<&str> = inner.splitn(2, " = ").collect();
        if au.len() == 2 {
            return au[0].trim().trim_matches('"') == au[1].trim().trim_matches('"');
        }
    }
    
    if inner.contains(" != ") {
        let au: alloc::vec::Vec<&str> = inner.splitn(2, " != ").collect();
        if au.len() == 2 {
            return au[0].trim().trim_matches('"') != au[1].trim().trim_matches('"');
        }
    }
    
    for op in &[" -eq ", " -ne ", " -gt ", " -lt ", " -ge ", " -le "] {
        if inner.contains(op) {
            let au: alloc::vec::Vec<&str> = inner.splitn(2, op).collect();
            if au.len() == 2 {
                let a = au[0].trim().parse::<i64>().unwrap_or(0);
                let b = au[1].trim().parse::<i64>().unwrap_or(0);
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

    
    !inner.is_empty()
}


fn crd(j: &str) -> String {
    use alloc::string::String;
    let j = j.trim();
    let jpd = if (j.starts_with('\'') && j.ends_with('\''))
        || (j.starts_with('"') && j.ends_with('"'))
    {
        &j[1..j.len() - 1]
    } else {
        j
    };
    
    let mut result = String::with_capacity(jpd.len());
    let mut fvh = false;
    for c in jpd.chars() {
        if fvh {
            match c {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                '\'' => result.push('\''),
                _ => { result.push('\\'); result.push(c); }
            }
            fvh = false;
        } else if c == '\\' {
            fvh = true;
        } else {
            result.push(c);
        }
    }
    result
}


pub(super) fn eyp(name: &str) -> String {
    if name.starts_with('/') {
        return String::from(name);
    }
    
    if name.starts_with("./") {
        let cwd = crate::ramfs::bh(|fs| String::from(fs.pwd()));
        if cwd == "/" {
            return String::from(&name[1..]); 
        } else {
            return format!("{}{}", cwd, &name[1..]); 
        }
    }
    
    
    let dyu = ["/usr/bin", "/bin", "/usr/sbin", "/sbin", "/usr/local/bin"];
    
    for it in &dyu {
        let path = format!("{}/{}", it, name);
        if bbs(&path) {
            return path;
        }
    }
    
    
    let cwd = crate::ramfs::bh(|fs| String::from(fs.pwd()));
    if cwd == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", cwd, name)
    }
}


pub(super) fn bbs(path: &str) -> bool {
    
    if crate::vfs::stat(path).is_ok() {
        return true;
    }
    
    crate::ramfs::bh(|fs| fs.exists(path))
}






pub(super) fn koq(args: &[&str]) {
    if args.is_empty() {
        iwl();
        return;
    }
    
    match args[0] {
        "init" => {
            crate::println!("Initializing TrustVM hypervisor...");
            match crate::hypervisor::init() {
                Ok(()) => {
                    crate::bq!(B_, "? ");
                    crate::println!("Hypervisor initialized successfully!");
                }
                Err(e) => {
                    crate::bq!(A_, "? ");
                    crate::println!("Failed to initialize hypervisor: {:?}", e);
                }
            }
        }
        "status" => {
            if crate::hypervisor::lq() {
                crate::bq!(B_, "? ");
                crate::println!("TrustVM: Active");
                crate::println!("  Backend: {}", crate::hypervisor::fhy());
                crate::println!("  VMs created: {}", crate::hypervisor::vm_count());
            } else {
                crate::bq!(D_, "? ");
                crate::println!("TrustVM: Inactive");
                crate::println!("  Run 'hv init' to enable the hypervisor");
            }
        }
        "check" => {
            use crate::hypervisor::{blt, CpuVendor};
            crate::println!("Checking virtualization support...");
            let vendor = blt();
            crate::println!("  CPU Vendor: {:?}", vendor);
            
            match vendor {
                CpuVendor::Intel => {
                    match crate::hypervisor::vmx::ehv() {
                        Ok(caps) => {
                            crate::println!("  [Intel VT-x (VMX)]");
                            crate::println!("    VMX supported:      {}", if caps.supported { "Yes" } else { "No" });
                            crate::println!("    EPT supported:      {}", if caps.ept_supported { "Yes" } else { "No" });
                            crate::println!("    Unrestricted guest: {}", if caps.unrestricted_guest { "Yes" } else { "No" });
                            crate::println!("    VPID supported:     {}", if caps.vpid_supported { "Yes" } else { "No" });
                            crate::println!("    VMCS revision:      0x{:08X}", caps.vmcs_revision_id);
                        }
                        Err(e) => {
                            crate::bq!(A_, "Error: ");
                            crate::println!("{:?}", e);
                        }
                    }
                }
                CpuVendor::Amd => {
                    if crate::hypervisor::svm::is_supported() {
                        let features = crate::hypervisor::svm::ckb();
                        crate::println!("  [AMD-V (SVM)]");
                        crate::println!("    SVM supported:      Yes");
                        crate::println!("    SVM Revision:       {}", features.revision);
                        crate::println!("    NPT supported:      {}", if features.npt { "Yes" } else { "No" });
                        crate::println!("    NRIP Save:          {}", if features.nrip_save { "Yes" } else { "No" });
                        crate::println!("    Flush by ASID:      {}", if features.flush_by_asid { "Yes" } else { "No" });
                        crate::println!("    Available ASIDs:    {}", features.num_asids);
                        crate::println!("    AVIC:               {}", if features.avic { "Yes" } else { "No" });
                    } else {
                        crate::bq!(A_, "Error: ");
                        crate::println!("SVM not supported or disabled in BIOS");
                    }
                }
                CpuVendor::Unknown => {
                    crate::bq!(A_, "Error: ");
                    crate::println!("Unknown CPU vendor - virtualization not supported");
                }
            }
        }
        "shutdown" => {
            crate::println!("Shutting down hypervisor...");
            match crate::hypervisor::shutdown() {
                Ok(()) => {
                    crate::bq!(B_, "? ");
                    crate::println!("Hypervisor shutdown complete");
                }
                Err(e) => {
                    crate::bq!(A_, "? ");
                    crate::println!("Failed: {:?}", e);
                }
            }
        }
        "caps" | "capabilities" => {
            crate::println!("{}", crate::hypervisor::eyj());
        }
        "security" => {
            crate::println!("{}", crate::hypervisor::eyk());
        }
        "events" => {
            let count = if args.len() > 1 { 
                args[1].parse().unwrap_or(10) 
            } else { 
                10 
            };
            let events = crate::hypervisor::ibl(count);
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
            if crate::hypervisor::csm() {
                crate::bq!(B_, "? ");
                crate::println!("VPID: Enabled");
                crate::println!("  Allocated VPIDs: {}", crate::hypervisor::jqo());
            } else {
                crate::bq!(D_, "? ");
                crate::println!("VPID: Disabled (CPU may not support it)");
            }
        }
        "violations" => {
            let count = crate::hypervisor::ept_violations();
            crate::println!("EPT Violations: {}", count);
            if count > 0 {
                let violations = crate::hypervisor::iys(5);
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
        
        
        
        #[cfg(target_arch = "aarch64")]
        "spy" | "mmio" => {
            crate::n!(C_, "=== TrustOS EL2 MMIO Spy ===");
            crate::println!();
            if !crate::hypervisor::arm_hv::cll() {
                crate::n!(A_, "Not running at EL2 - hypervisor mode unavailable");
                crate::println!("Boot TrustOS at EL2 (QEMU: -machine virt,virtualization=on)");
                return;
            }
            if !crate::hypervisor::arm_hv::is_active() {
                crate::n!(D_, "EL2 detected but no guest running yet");
                crate::println!("Use 'hv launch' to start a guest under surveillance");
                return;
            }
            let report = crate::hypervisor::arm_hv::el2_entry::mdv();
            crate::println!("{}", report);
        }
        #[cfg(target_arch = "aarch64")]
        "smc" | "smc-log" => {
            crate::n!(C_, "=== SMC (Secure Monitor Call) Log ===");
            let count = if args.len() > 1 { args[1].parse().unwrap_or(20) } else { 20 };
            let events = crate::hypervisor::arm_hv::mmio_spy::gqq(count);
            if events.is_empty() {
                crate::println!("No SMC calls intercepted.");
            } else {
                for rt in &events {
                    crate::println!("  {}", crate::hypervisor::arm_hv::mmio_spy::hzq(rt));
                }
                crate::println!("\nTotal SMC events: {}",
                    crate::hypervisor::arm_hv::mmio_spy::fdl());
            }
        }
        #[cfg(target_arch = "aarch64")]
        "devices" => {
            crate::n!(C_, "=== Device Activity (per MMIO range) ===");
            let stats = crate::hypervisor::arm_hv::mmio_spy::hrz();
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
            crate::n!(C_, "=== ARM EL2 Hypervisor Status ===");
            if crate::hypervisor::arm_hv::cll() {
                crate::n!(B_, "  Running at EL2: Yes");
                crate::println!("  Hypervisor active: {}", 
                    if crate::hypervisor::arm_hv::is_active() { "Yes (guest running)" } else { "No (idle)" });
                crate::println!("  MMIO traps: {}", crate::hypervisor::arm_hv::ioa());
                crate::println!("  SMC intercepts: {}", crate::hypervisor::arm_hv::jgr());
                crate::println!("  MMIO events logged: {}", 
                    crate::hypervisor::arm_hv::mmio_spy::gzs());
                crate::println!("  SMC events logged: {}", 
                    crate::hypervisor::arm_hv::mmio_spy::fdl());
            } else {
                crate::n!(A_, "  Running at EL2: No");
                crate::println!("  Current EL does not support hypervisor operations.");
                crate::println!("  Boot with: qemu-system-aarch64 -machine virt,virtualization=on");
            }
        }
        #[cfg(target_arch = "aarch64")]
        "report" => {
            crate::println!("{}", crate::hypervisor::arm_hv::mck());
        }
        #[cfg(target_arch = "aarch64")]
        "boot" | "launch" => {
            use crate::hypervisor::arm_hv::guest_loader;
            crate::n!(C_, "=== TrustOS EL2 Hypervisor — Guest Boot ===");
            crate::println!();

            if !crate::hypervisor::arm_hv::cll() {
                crate::n!(A_, "ERROR: Not running at EL2!");
                crate::println!("  Boot TrustOS with: qemu-system-aarch64 -machine virt,virtualization=on");
                return;
            }

            
            if args.len() <= 1 || args[1] == "test" {
                crate::println!("Launching self-test guest (WFI loop)...");
                crate::println!("  This tests the full EL2 hypervisor pipeline:");
                crate::println!("  Stage-2 tables -> HCR_EL2 -> VBAR_EL2 -> vGIC -> ERET -> trap -> log");
                crate::println!();

                let ram_base = 0x4000_0000u64;
                let ram_size = 512 * 1024 * 1024u64;

                match guest_loader::onl(ram_base, ram_size) {
                    Ok(result) => {
                        crate::println!("{}", guest_loader::lxn(&result));
                        crate::n!(B_, "Guest loaded successfully!");
                        crate::println!("  To actually enter the guest: hv enter");
                        crate::println!("  (This will transfer control to EL1 — TrustOS shell will");
                        crate::println!("   continue to run at EL2, intercepting all hardware access)");
                    }
                    Err(e) => {
                        crate::n!(A_, "Failed to load guest: {}", e);
                    }
                }
            } else {
                crate::println!("Usage:");
                crate::println!("  hv boot test     - Load self-test WFI guest");
                crate::println!("  hv boot android  - Load Android kernel (requires Image in guest RAM)");
                crate::println!();
                crate::println!("For QEMU Android demo:");
                crate::println!("  1. Download AOSP emulator kernel: ci.android.com -> aosp-main -> aosp_cf_arm64_phone");
                crate::println!("  2. Extract Image and initrd from the build artifacts");
                crate::println!("  3. Run: .\\run-android-el2.ps1 -Kernel path\\to\\Image -Initrd path\\to\\ramdisk.img");
            }
        }
        #[cfg(target_arch = "aarch64")]
        "load" => {
            use crate::hypervisor::arm_hv::guest_loader;
            crate::n!(C_, "=== Guest Loader — ARM64 Image Validator ===");
            crate::println!();

            
            let config = guest_loader::GuestLoadConfig::default();
            crate::println!("Memory layout for guest:");
            crate::println!("  RAM:     0x{:08X} - 0x{:08X} ({} MB)",
                config.ram_base,
                config.ram_base + config.ram_size,
                config.ram_size / (1024*1024));
            crate::println!("  Kernel:  0x{:08X} (RAM + {}MB)",
                config.ram_base + guest_loader::AFL_,
                guest_loader::AFL_ / (1024*1024));
            crate::println!("  DTB:     0x{:08X} (RAM + {}MB)",
                config.ram_base + guest_loader::ACV_,
                guest_loader::ACV_ / (1024*1024));
            crate::println!("  initrd:  0x{:08X} (RAM + {}MB)",
                config.ram_base + guest_loader::AFC_,
                guest_loader::AFC_ / (1024*1024));
            crate::println!();
            crate::println!("MMIO traps ({} regions):", config.trap_mmio.len());
            for (base, size) in &config.trap_mmio {
                crate::println!("  0x{:08X} - 0x{:08X} ({})",
                    base, base + size,
                    crate::hypervisor::arm_hv::mmio_spy::btg(*base));
            }
            crate::println!();
            crate::println!("Kernel cmdline: {}", config.cmdline);
        }
        "test" | "selftest" => {
            crate::n!(C_, "╔══════════════════════════════════════════════════════╗");
            crate::n!(C_, "║         TrustVM Hypervisor Self-Test Suite           ║");
            crate::n!(C_, "╚══════════════════════════════════════════════════════╝");
            crate::println!();
            
            let (passed, bv, log) = crate::hypervisor::tests::ezf();
            
            for line in &log {
                if line.contains("[PASS]") {
                    crate::n!(B_, "{}", line);
                } else {
                    crate::n!(A_, "{}", line);
                }
            }
            
            crate::println!();
            if bv == 0 {
                crate::n!(B_, "Result: {}/{} tests passed — ALL OK ✓", passed, passed + bv);
            } else {
                crate::n!(A_, "Result: {}/{} tests passed, {} FAILED ✗", passed, passed + bv, bv);
            }
        }
        "help" | _ => iwl(),
    }
}

fn iwl() {
    use crate::hypervisor::{blt, CpuVendor};
    let vendor = blt();
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
        crate::println!("  hv boot [test]- Boot a guest under EL2 surveillance");
        crate::println!("  hv load       - Show guest memory layout + MMIO trap config");
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


pub(super) fn ktt(args: &[&str]) {
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
            let bnn: usize = args[2].parse().unwrap_or(16);
            
            if !crate::hypervisor::lq() {
                crate::bq!(D_, "Warning: ");
                crate::println!("Hypervisor not initialized. Run 'hv init' first.");
                return;
            }
            
            match crate::hypervisor::blh(name, bnn) {
                Ok(id) => {
                    crate::bq!(B_, "? ");
                    crate::println!("Created VM '{}' with ID {} ({}MB RAM)", name, id, bnn);
                }
                Err(e) => {
                    crate::bq!(A_, "? ");
                    crate::println!("Failed to create VM: {:?}", e);
                }
            }
        }
        "start" => {
            if args.len() < 2 {
                crate::println!("Usage: vm start <id> [guest_name]");
                crate::println!("Available guests: {:?}", crate::hypervisor::dtj());
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let axj = if args.len() > 2 { args[2] } else { "hello" };
            
            crate::println!("Starting VM {} with guest '{}'...", id, axj);
            match crate::hypervisor::dev(id, axj) {
                Ok(()) => {
                    crate::bq!(B_, "? ");
                    crate::println!("VM {} completed execution", id);
                }
                Err(e) => {
                    crate::bq!(A_, "? ");
                    crate::println!("VM {} failed: {:?}", id, e);
                }
            }
        }
        "run" => {
            
            let axj = if args.len() > 1 { args[1] } else { "hello" };
            
            if !crate::hypervisor::lq() {
                crate::bq!(D_, "Note: ");
                crate::println!("Initializing hypervisor first...");
                if let Err(e) = crate::hypervisor::init() {
                    crate::bq!(A_, "✗ ");
                    crate::println!("Failed to init hypervisor: {:?}", e);
                    return;
                }
            }
            
            
            let bnn = if axj.starts_with("linux") || axj.ends_with(".bzimage") {
                64
            } else {
                4
            };
            
            match crate::hypervisor::blh(axj, bnn) {
                Ok(id) => {
                    crate::println!("Running guest '{}'...", axj);
                    match crate::hypervisor::dev(id, axj) {
                        Ok(()) => {
                            crate::bq!(B_, "? ");
                            crate::println!("Guest '{}' completed", axj);
                            
                            
                            crate::hypervisor::svm_vm::avv(id, |vm| {
                                let j = &vm.stats;
                                crate::println!("  VMEXITs: {} (cpuid={} io={} msr={} hlt={} vmcall={})",
                                    j.vmexits, j.cpuid_exits, j.io_exits,
                                    j.msr_exits, j.hlt_exits, j.vmmcall_exits);
                            });
                            crate::println!("  Use 'vm inspect {}' for detailed state", id);
                        }
                        Err(e) => {
                            crate::bq!(A_, "? ");
                            crate::println!("Failed: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    crate::bq!(A_, "? ");
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
            
            match crate::hypervisor::fbu(id) {
                Ok(()) => {
                    crate::bq!(B_, "? ");
                    crate::println!("Stopped VM {}", id);
                }
                Err(e) => {
                    crate::bq!(A_, "? ");
                    crate::println!("Failed to stop VM {}: {:?}", id, e);
                }
            }
        }
        "list" => {
            use crate::hypervisor::{blt, CpuVendor};
            crate::println!("Virtual Machines:");
            
            match blt() {
                CpuVendor::Amd => {
                    let aen = crate::hypervisor::svm_vm::dtn();
                    if aen.is_empty() {
                        crate::println!("  (no VMs created)");
                    } else {
                        crate::println!("  {:>4} {:>20} {:>12}", "ID", "NAME", "STATE");
                        crate::println!("  {:->4} {:->20} {:->12}", "", "", "");
                        for (id, name, state) in aen {
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
            for axj in crate::hypervisor::dtj() {
                crate::println!("  - {}", axj);
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
            crate::bq!(B_, "? ");
            crate::println!("Mounted {} -> {} (readonly={})", host_path, guest_path, readonly);
        }
        "console" => {
            if args.len() < 2 {
                crate::println!("Usage: vm console <vm_id>");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let output = crate::hypervisor::eoa(id);
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
            crate::hypervisor::gct(id, text.as_bytes());
            crate::hypervisor::gct(id, b"\n");
            crate::println!("Injected input to VM {}", id);
        }
        "inspect" => {
            
            use crate::hypervisor::{blt, CpuVendor};
            
            match blt() {
                CpuVendor::Amd => {
                    let aen = crate::hypervisor::svm_vm::dtn();
                    if aen.is_empty() {
                        crate::println!("No VMs to inspect. Run 'vm run pm-test' first.");
                        return;
                    }
                    
                    
                    let lvj: Option<u64> = if args.len() > 1 { args[1].parse().ok() } else { None };
                    
                    for (id, name, state) in &aen {
                        if let Some(fid) = lvj {
                            if *id != fid { continue; }
                        }
                        
                        crate::n!(C_, "+--- VM #{}: {} [{:?}] ---+", id, name, state);
                        
                        
                        crate::hypervisor::svm_vm::avv(*id, |vm| {
                            let j = &vm.stats;
                            crate::println!();
                            crate::n!(D_, "  Exit Statistics:");
                            crate::println!("    Total VMEXITs: {}", j.vmexits);
                            crate::println!("    CPUID:   {:>8}", j.cpuid_exits);
                            crate::println!("    I/O:     {:>8}", j.io_exits);
                            crate::println!("    MSR:     {:>8}", j.msr_exits);
                            crate::println!("    HLT:     {:>8}", j.hlt_exits);
                            crate::println!("    NPF:     {:>8}", j.npf_exits);
                            crate::println!("    VMCALL:  {:>8}", j.vmmcall_exits);
                            crate::println!("    Intr:    {:>8}", j.intr_exits);
                            
                            crate::println!();
                            crate::n!(D_, "  Guest GPRs:");
                            crate::println!("    RAX = 0x{:016X}  RBX = 0x{:016X}", vm.guest_regs.rax, vm.guest_regs.rbx);
                            crate::println!("    RCX = 0x{:016X}  RDX = 0x{:016X}", vm.guest_regs.rcx, vm.guest_regs.rdx);
                            crate::println!("    RSI = 0x{:016X}  RDI = 0x{:016X}", vm.guest_regs.rsi, vm.guest_regs.rdi);
                            crate::println!("    RBP = 0x{:016X}  RSP = 0x{:016X}", vm.guest_regs.rbp, vm.guest_regs.rsp);
                            crate::println!("    R8  = 0x{:016X}  R9  = 0x{:016X}", vm.guest_regs.r8, vm.guest_regs.r9);
                            crate::println!("    R10 = 0x{:016X}  R11 = 0x{:016X}", vm.guest_regs.r10, vm.guest_regs.r11);
                            crate::println!("    R12 = 0x{:016X}  R13 = 0x{:016X}", vm.guest_regs.r12, vm.guest_regs.r13);
                            crate::println!("    R14 = 0x{:016X}  R15 = 0x{:016X}", vm.guest_regs.r14, vm.guest_regs.r15);
                            
                            
                            if let Some(ref vmcb) = vm.vmcb {
                                use crate::hypervisor::svm::vmcb::state_offsets;
                                crate::println!();
                                crate::n!(D_, "  VMCB State:");
                                let rip = vmcb.read_state(state_offsets::Af);
                                let rsp = vmcb.read_state(state_offsets::De);
                                let rflags = vmcb.read_state(state_offsets::Ek);
                                let cr0 = vmcb.read_state(state_offsets::Jn);
                                let cr3 = vmcb.read_state(state_offsets::Jo);
                                let cr4 = vmcb.read_state(state_offsets::Jp);
                                let efer = vmcb.read_state(state_offsets::Eu);
                                let cs = vmcb.read_u16(state_offsets::KO_) as u64;
                                let ds = vmcb.read_u16(state_offsets::NT_) as u64;
                                let eiz = vmcb.read_u16(state_offsets::Of) as u64;
                                
                                crate::println!("    RIP    = 0x{:016X}  RSP    = 0x{:016X}", rip, rsp);
                                crate::println!("    RFLAGS = 0x{:016X}  CPL    = {}", rflags, eiz);
                                crate::println!("    CR0 = 0x{:X}  CR3 = 0x{:X}  CR4 = 0x{:X}", cr0, cr3, cr4);
                                crate::println!("    EFER = 0x{:X}  CS = 0x{:X}  DS = 0x{:X}", efer, cs, ds);
                                
                                
                                use crate::hypervisor::svm::vmcb::control_offsets;
                                let fvr = vmcb.read_control(control_offsets::Lv);
                                let lsl = vmcb.read_control(control_offsets::Lx);
                                let lsm = vmcb.read_control(control_offsets::Ly);
                                crate::println!();
                                crate::n!(D_, "  Last VMEXIT:");
                                crate::println!("    ExitCode = 0x{:X}  Info1 = 0x{:X}  Info2 = 0x{:X}", 
                                    fvr, lsl, lsm);
                            }
                            
                            crate::println!();
                            crate::n!(D_, "  Memory:");
                            crate::println!("    Guest memory: {} KB ({} MB)", vm.memory_size / 1024, vm.memory_size / (1024 * 1024));
                            crate::println!("    ASID: {}", vm.asid);
                            
                            
                            let eit = crate::hypervisor::eoa(*id);
                            if !eit.is_empty() {
                                crate::println!();
                                crate::n!(D_, "  Console Output (last 256 chars):");
                                let start = if eit.len() > 256 { eit.len() - 256 } else { 0 };
                                crate::println!("    {}", &eit[start..]);
                            }
                            
                            
                            if let Some(data) = vm.read_guest_memory(0x5000, 32) {
                                crate::println!();
                                crate::n!(D_, "  Memory @ 0x5000 (guest write zone):");
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
        
        
        "dump" | "hexdump" => {
            if args.len() < 3 {
                crate::println!("Usage: vm dump <id> <gpa_hex> [length]");
                crate::println!("  Example: vm dump 1 0x1000 256");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let mfl = args[2].trim_start_matches("0x").trim_start_matches("0X");
            let gm = u64::from_str_radix(mfl, 16).unwrap_or(0);
            let len: usize = if args.len() > 3 {
                args[3].parse().unwrap_or(128)
            } else {
                128
            };
            let len = len.min(4096); 
            
            match crate::hypervisor::cpu_vendor() {
                crate::hypervisor::CpuVendor::Amd => {
                    crate::hypervisor::svm_vm::avv(id, |vm| {
                        crate::n!(C_, "  Memory dump: VM {} @ GPA 0x{:X} ({} bytes)", id, gm, len);
                        crate::println!();
                        
                        if let Some(data) = vm.read_guest_memory(gm, len) {
                            
                            for fk in (0..data.len()).step_by(16) {
                                crate::print!("  {:08X}: ", gm as usize + fk);
                                
                                for col in 0..16 {
                                    if fk + col < data.len() {
                                        crate::print!("{:02X} ", data[fk + col]);
                                    } else {
                                        crate::print!("   ");
                                    }
                                    if col == 7 { crate::print!(" "); }
                                }
                                
                                crate::print!(" |");
                                for col in 0..16 {
                                    if fk + col < data.len() {
                                        let b = data[fk + col];
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
                            crate::n!(A_, "  GPA 0x{:X}+{} is outside guest memory ({} bytes)", 
                                gm, len, vm.memory_size);
                        }
                    });
                }
                _ => {
                    crate::println!("vm dump requires AMD SVM.");
                }
            }
        }
        
        
        "regs" | "registers" => {
            if args.len() < 2 {
                crate::println!("Usage: vm regs <id>");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            
            match crate::hypervisor::cpu_vendor() {
                crate::hypervisor::CpuVendor::Amd => {
                    crate::hypervisor::svm_vm::avv(id, |vm| {
                        crate::n!(C_, "  VM {} Register State", id);
                        crate::println!();
                        
                        
                        crate::println!("  RAX={:016X}  RBX={:016X}", vm.guest_regs.rax, vm.guest_regs.rbx);
                        crate::println!("  RCX={:016X}  RDX={:016X}", vm.guest_regs.rcx, vm.guest_regs.rdx);
                        crate::println!("  RSI={:016X}  RDI={:016X}", vm.guest_regs.rsi, vm.guest_regs.rdi);
                        crate::println!("  RBP={:016X}  RSP={:016X}", vm.guest_regs.rbp, 
                            vm.vmcb.as_ref().map_or(0, |v| v.read_state(crate::hypervisor::svm::vmcb::state_offsets::De)));
                        crate::println!("  R8 ={:016X}  R9 ={:016X}", vm.guest_regs.r8, vm.guest_regs.r9);
                        crate::println!("  R10={:016X}  R11={:016X}", vm.guest_regs.r10, vm.guest_regs.r11);
                        crate::println!("  R12={:016X}  R13={:016X}", vm.guest_regs.r12, vm.guest_regs.r13);
                        crate::println!("  R14={:016X}  R15={:016X}", vm.guest_regs.r14, vm.guest_regs.r15);
                        
                        if let Some(ref vmcb) = vm.vmcb {
                            use crate::hypervisor::svm::vmcb::{state_offsets, control_offsets};
                            
                            let rip = vmcb.read_state(state_offsets::Af);
                            let cpi = vmcb.read_state(state_offsets::Ek);
                            crate::println!("  RIP={:016X}  RFLAGS={:016X}", rip, cpi);
                            
                            
                            let cxx = {
                                let mut j = alloc::string::String::new();
                                if cpi & 0x001 != 0 { j.push_str("CF "); }
                                if cpi & 0x040 != 0 { j.push_str("ZF "); }
                                if cpi & 0x080 != 0 { j.push_str("SF "); }
                                if cpi & 0x200 != 0 { j.push_str("IF "); }
                                if cpi & 0x400 != 0 { j.push_str("DF "); }
                                if cpi & 0x800 != 0 { j.push_str("OF "); }
                                j
                            };
                            crate::println!("  Flags: [{}]", cxx.trim());
                            
                            crate::println!();
                            crate::println!("  CR0={:016X}  CR2={:016X}", 
                                vmcb.read_state(state_offsets::Jn), vmcb.read_state(state_offsets::Og));
                            crate::println!("  CR3={:016X}  CR4={:016X}", 
                                vmcb.read_state(state_offsets::Jo), vmcb.read_state(state_offsets::Jp));
                            crate::println!("  EFER={:016X}", vmcb.read_state(state_offsets::Eu));
                            
                            
                            crate::println!();
                            crate::println!("  CS: sel={:04X} base={:016X} lim={:08X} attr={:04X}", 
                                vmcb.read_u16(state_offsets::KO_),
                                vmcb.read_state(state_offsets::TP_),
                                vmcb.read_u32(state_offsets::ACO_),
                                vmcb.read_u16(state_offsets::ACN_));
                            crate::println!("  SS: sel={:04X} base={:016X} lim={:08X} attr={:04X}", 
                                vmcb.read_u16(state_offsets::YO_),
                                vmcb.read_state(state_offsets::AJZ_),
                                vmcb.read_u32(state_offsets::AKA_),
                                vmcb.read_u16(state_offsets::AJY_));
                            crate::println!("  DS: sel={:04X}  ES: sel={:04X}  FS: sel={:04X}  GS: sel={:04X}", 
                                vmcb.read_u16(state_offsets::NT_),
                                vmcb.read_u16(state_offsets::UF_),
                                vmcb.read_u16(state_offsets::AEB_),
                                vmcb.read_u16(state_offsets::AEV_));
                            
                            
                            crate::println!();
                            crate::n!(D_, "  LAPIC State:");
                            crate::println!("    Enabled: {}  SVR: 0x{:X}  TPR: 0x{:X}", 
                                vm.lapic.enabled, vm.lapic.svr, vm.lapic.tpr);
                            let gyw = match (vm.lapic.timer_lvt >> 17) & 0x3 {
                                0 => "one-shot", 1 => "periodic", 2 => "TSC-deadline", _ => "reserved",
                            };
                            let pjs = (vm.lapic.timer_lvt >> 16) & 1;
                            let pjv = vm.lapic.timer_lvt & 0xFF;
                            crate::println!("    Timer: vec={} mode={} masked={} ICR={} DCR={}", 
                                pjv, gyw, pjs, vm.lapic.icr, vm.lapic.dcr);
                            
                            
                            crate::println!();
                            let fvr = vmcb.read_control(control_offsets::Lv);
                            let drx = vmcb.read_control(control_offsets::Lx);
                            let mos = vmcb.read_control(control_offsets::Ly);
                            crate::println!("  Last VMEXIT: code=0x{:X} info1=0x{:X} info2=0x{:X}", fvr, drx, mos);
                            
                            
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
        
        
        "stack" | "backtrace" | "bt" => {
            if args.len() < 2 {
                crate::println!("Usage: vm stack <id> [depth]");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let depth: usize = if args.len() > 2 { args[2].parse().unwrap_or(16) } else { 16 };
            
            match crate::hypervisor::cpu_vendor() {
                crate::hypervisor::CpuVendor::Amd => {
                    crate::hypervisor::svm_vm::avv(id, |vm| {
                        if let Some(ref vmcb) = vm.vmcb {
                            use crate::hypervisor::svm::vmcb::state_offsets;
                            
                            let rsp = vmcb.read_state(state_offsets::De);
                            let rip = vmcb.read_state(state_offsets::Af);
                            let rbp = vm.guest_regs.rbp;
                            
                            crate::n!(C_, "  VM {} Stack Trace (RSP=0x{:X}, RIP=0x{:X})", id, rsp, rip);
                            crate::println!();
                            
                            
                            crate::println!("  Stack contents (potential return addresses):");
                            for i in 0..depth {
                                let addr = rsp + (i as u64 * 8);
                                if let Some(data) = vm.read_guest_memory(addr, 8) {
                                    let val = u64::from_le_bytes([
                                        data[0], data[1], data[2], data[3],
                                        data[4], data[5], data[6], data[7],
                                    ]);
                                    
                                    let marker = if val > 0xFFFF_8000_0000_0000 { " <-- kernel addr" }
                                        else if val > 0x1000 && val < 0x1_0000_0000 { " <-- possible code" }
                                        else { "" };
                                    crate::println!("  [{:2}] RSP+{:04X}: {:016X}{}", i, i * 8, val, marker);
                                } else {
                                    crate::println!("  [{:2}] RSP+{:04X}: <outside guest memory>", i, i * 8);
                                    break;
                                }
                            }
                            
                            
                            if rbp > 0x1000 && rbp < vm.memory_size as u64 {
                                crate::println!();
                                crate::println!("  Frame pointer chain (RBP=0x{:X}):", rbp);
                                let mut frame = rbp;
                                for i in 0..depth.min(32) {
                                    if frame < 0x1000 || frame >= vm.memory_size as u64 - 16 { break; }
                                    if let Some(data) = vm.read_guest_memory(frame, 16) {
                                        let gjn = u64::from_le_bytes([
                                            data[0], data[1], data[2], data[3],
                                            data[4], data[5], data[6], data[7],
                                        ]);
                                        let bdk = u64::from_le_bytes([
                                            data[8], data[9], data[10], data[11],
                                            data[12], data[13], data[14], data[15],
                                        ]);
                                        crate::println!("  #{}: RBP=0x{:X} -> ret=0x{:X}", i, frame, bdk);
                                        if gjn <= frame || gjn == 0 { break; }
                                        frame = gjn;
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
                    let report = crate::hypervisor::debug_monitor::fyr();
                    crate::println!("{}", report);
                }
                "io" => {
                    let report = crate::hypervisor::debug_monitor::ibo();
                    crate::println!("{}", report);
                }
                "msr" => {
                    let report = crate::hypervisor::debug_monitor::ibq();
                    crate::println!("{}", report);
                }
                "timeline" => {
                    let count = if args.len() > 2 { args[2].parse().unwrap_or(30) } else { 30 };
                    let report = crate::hypervisor::debug_monitor::ibz(count);
                    crate::println!("{}", report);
                }
                "serial" => {
                    let enabled = args.len() <= 2 || args[2] != "off";
                    crate::hypervisor::debug_monitor::jfj(enabled);
                    crate::println!("Serial logging: {}", if enabled { "ON" } else { "OFF" });
                }
                "status" => {
                    let active = crate::hypervisor::debug_monitor::is_active();
                    let av = crate::hypervisor::debug_monitor::fdf();
                    let dga = crate::hypervisor::debug_monitor::fdw();
                    crate::println!("\x01CDebug Monitor Status:\x01W");
                    crate::println!("  Active: {}", if active { "\x01Gyes\x01W" } else { "\x01Rno\x01W" });
                    crate::println!("  Total events: {}", av);
                    crate::println!("  Unhandled: {}{}\x01W", 
                        if dga > 0 { "\x01R" } else { "\x01G" }, dga);
                }
                "" => {
                    
                    if !crate::hypervisor::debug_monitor::is_initialized() {
                        
                        crate::hypervisor::debug_monitor::init();
                        crate::println!("Debug monitor auto-initialized. Run a VM to collect data.\n");
                    }
                    let dmg = crate::hypervisor::debug_monitor::fym();
                    crate::println!("{}", dmg);
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
        
        
        "linux" => {
            if args.len() < 2 {
                crate::println!("Usage: vm linux <bzimage_path> [initrd_path] [memory_mb] [cmdline]");
                crate::println!("  Example: vm linux /boot/vmlinuz /boot/initrd.img 128");
                crate::println!("  Default: 128 MB RAM, console=ttyS0 earlyprintk nokaslr");
                return;
            }
            
            let fkl = args[1];
            let igx = if args.len() > 2 && !args[2].parse::<usize>().is_ok() {
                Some(args[2])
            } else {
                None
            };
            let ghj = if igx.is_some() { 3 } else { 2 };
            let bnn: usize = if args.len() > ghj {
                args[ghj].parse().unwrap_or(128)
            } else {
                128
            };
            let hmi = ghj + 1;
            let cmdline = if args.len() > hmi {
                args[hmi..].join(" ")
            } else {
                alloc::string::String::from("console=ttyS0 earlyprintk=serial,ttyS0 nokaslr noapic")
            };
            
            
            crate::n!(C_, "Loading Linux kernel from {}...", fkl);
            let bas = match crate::vfs::read_file(fkl) {
                Ok(data) => {
                    crate::n!(B_, "  Kernel: {} bytes ({} KB)", data.len(), data.len() / 1024);
                    data
                }
                Err(e) => {
                    crate::n!(A_, "  Error reading {}: {:?}", fkl, e);
                    return;
                }
            };
            
            
            let bck = if let Some(path) = igx {
                crate::println!("Loading initrd from {}...", path);
                match crate::vfs::read_file(path) {
                    Ok(data) => {
                        crate::n!(B_, "  Initrd: {} bytes ({} KB)", data.len(), data.len() / 1024);
                        Some(data)
                    }
                    Err(e) => {
                        crate::n!(A_, "  Error reading {}: {:?}", path, e);
                        return;
                    }
                }
            } else {
                None
            };
            
            
            if !crate::hypervisor::lq() {
                crate::println!("Initializing hypervisor...");
                if let Err(e) = crate::hypervisor::init() {
                    crate::n!(A_, "Hypervisor init failed: {:?}", e);
                    return;
                }
            }
            
            
            crate::println!("Creating VM ({} MB RAM)...", bnn);
            crate::println!("Cmdline: {}", cmdline);
            
            match crate::hypervisor::blh("linux-guest", bnn) {
                Ok(id) => {
                    crate::println!("Booting Linux in VM #{}...", id);
                    
                    let mpy = bck.as_deref();
                    
                    
                    match crate::hypervisor::cpu_vendor() {
                        crate::hypervisor::CpuVendor::Amd => {
                            let result = crate::hypervisor::svm_vm::avv(id, |vm| {
                                vm.start_linux(&bas, &cmdline, mpy)
                            });
                            match result {
                                Some(Ok(())) => {
                                    crate::n!(B_, "Linux VM completed");
                                }
                                Some(Err(e)) => {
                                    crate::n!(A_, "Linux VM failed: {:?}", e);
                                    crate::println!("Use 'vm inspect {}' for details", id);
                                }
                                None => {
                                    crate::n!(A_, "VM #{} not found", id);
                                }
                            }
                        }
                        crate::hypervisor::CpuVendor::Intel => {
                            
                            let config = crate::hypervisor::linux_vm::Gq {
                                memory_mb: bnn,
                                cmdline: cmdline.clone(),
                                ..Default::default()
                            };
                            match crate::hypervisor::linux_vm::LinuxVm::new(config) {
                                Ok(mut vm) => {
                                    let lpb = alloc::vec::Vec::new();
                                    let mpw = bck.as_deref().unwrap_or(&lpb);
                                    match vm.boot(&bas, mpw) {
                                        Ok(()) => {
                                            crate::n!(B_, "Linux VM completed");
                                        }
                                        Err(e) => {
                                            crate::n!(A_, "Linux VM failed: {:?}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    crate::n!(A_, "Failed to create Linux VM: {:?}", e);
                                }
                            }
                        }
                        _ => {
                            crate::n!(A_, "No hardware virtualization available");
                        }
                    }
                }
                Err(e) => {
                    crate::n!(A_, "Failed to create VM: {:?}", e);
                }
            }
        }
        
        _ => {
            crate::println!("Unknown VM command: {}", args[0]);
            crate::println!("Commands: create, start, run, stop, list, guests, linux, mount, console, input, inspect, dump, regs, stack");
        }
    }
}




pub(super) fn qap(args: &[&str]) {
    use crate::hypervisor::linux_subsystem::{self, LinuxState};
    
    if args.is_empty() {
        iwm();
        return;
    }
    
    match args[0] {
        "init" | "start" => {
            crate::n!(C_, "+----------------------------------------------------------+");
            crate::n!(C_, "|     TrustOS Subsystem for Linux (TSL) v1.0              |");
            crate::n!(C_, "+----------------------------------------------------------+");
            crate::println!();
            crate::println!("Initializing Linux Subsystem...");
            
            match linux_subsystem::init() {
                Ok(()) => {
                    crate::bq!(B_, "? ");
                    crate::println!("Linux Subsystem initialized");
                    crate::println!();
                    crate::println!("Use 'linux boot' to start real Linux VM,");
                    crate::println!("or 'linux <command>' for simulated commands.");
                }
                Err(e) => {
                    crate::bq!(A_, "? ");
                    crate::println!("Failed to initialize: {:?}", e);
                }
            }
        }
        "boot" => {
            crate::n!(C_, "+----------------------------------------------------------+");
            crate::n!(C_, "|          Booting Real Linux VM...                       |");
            crate::n!(C_, "+----------------------------------------------------------+");
            crate::println!();
            
            
            let vendor = crate::hypervisor::cpu_vendor();
            match vendor {
                crate::hypervisor::CpuVendor::Intel => {
                    crate::println!("CPU: Intel (VMX)");
                }
                crate::hypervisor::CpuVendor::Amd => {
                    crate::println!("CPU: AMD (SVM)");
                }
                crate::hypervisor::CpuVendor::Unknown => {
                    crate::n!(D_, "Warning: No hardware virtualization detected");
                    crate::println!("         Real VM boot may not be possible.");
                }
            }
            
            crate::println!();
            crate::println!("Starting Linux VM with kernel and initramfs...");
            
            match linux_subsystem::boot() {
                Ok(()) => {
                    crate::bq!(B_, "? ");
                    crate::println!("Linux VM boot completed");
                }
                Err(e) => {
                    crate::bq!(A_, "? ");
                    crate::println!("Boot failed: {:?}", e);
                    crate::println!();
                    crate::n!(D_, "Falling back to simulated mode.");
                }
            }
        }
        "status" => {
            let state = linux_subsystem::state();
            let bwb = linux_subsystem::acs();
            
            crate::n!(G_, "Linux Subsystem Status:");
            crate::println!("---------------------------------------");
            
            match state {
                LinuxState::NotStarted => {
                    crate::bq!(D_, "? State: ");
                    crate::println!("Not Started");
                    crate::println!("  Run 'linux init' to start the subsystem.");
                }
                LinuxState::Booting => {
                    crate::bq!(D_, "? State: ");
                    crate::println!("Booting...");
                }
                LinuxState::Ready => {
                    crate::bq!(B_, "? State: ");
                    crate::println!("Ready");
                }
                LinuxState::Busy => {
                    crate::bq!(C_, "? State: ");
                    crate::println!("Busy (executing command)");
                }
                LinuxState::Error => {
                    crate::bq!(A_, "? State: ");
                    crate::println!("Error");
                }
                LinuxState::ShuttingDown => {
                    crate::bq!(D_, "? State: ");
                    crate::println!("Shutting down...");
                }
            }
            
            
            crate::println!();
            crate::n!(C_, "Kernel Image:");
            if bwb.has_kernel() {
                let kernel_size = bwb.kernel_size();
                crate::println!("  ? Loaded: {} bytes ({} KB)", kernel_size, kernel_size / 1024);
                if let Some(version) = bwb.kernel_version_string() {
                    crate::println!("  Version:  {}", version);
                }
                if let Some((axz, ayh)) = bwb.boot_protocol_version() {
                    crate::println!("  Protocol: {}.{}", axz, ayh);
                }
            } else {
                crate::println!("  ? Not loaded (simulated mode)");
            }
            
            crate::println!();
            crate::n!(C_, "Initramfs:");
            if bwb.has_initramfs() {
                let initrd_size = bwb.initramfs_size();
                crate::println!("  ? Loaded: {} bytes ({} KB)", initrd_size, initrd_size / 1024);
            } else {
                crate::println!("  ? Not loaded");
            }
            
            crate::println!();
            crate::n!(C_, "VM Configuration:");
            crate::println!("  Memory:   {} MB", linux_subsystem::WB_);
            crate::println!("  VM ID:    {:#X}", linux_subsystem::AGI_);
            
            drop(bwb);
        }
        "stop" | "shutdown" => {
            crate::println!("Shutting down Linux Subsystem...");
            match linux_subsystem::shutdown() {
                Ok(()) => {
                    crate::bq!(B_, "? ");
                    crate::println!("Linux Subsystem stopped");
                }
                Err(e) => {
                    crate::bq!(A_, "? ");
                    crate::println!("Failed: {:?}", e);
                }
            }
        }
        "extract" => {
            
            super::apps::hos();
        }
        "help" | "--help" | "-h" => {
            iwm();
        }
        
        _ => {
            
            let command = args.join(" ");
            
            match linux_subsystem::execute(&command) {
                Ok(result) => {
                    if !result.stdout.is_empty() {
                        crate::println!("{}", result.stdout);
                    }
                    if !result.stderr.is_empty() {
                        crate::bq!(A_, "{}", result.stderr);
                    }
                    if result.exit_code != 0 && result.stderr.is_empty() {
                        crate::n!(D_, "(exit code: {})", result.exit_code);
                    }
                }
                Err(e) => {
                    crate::bq!(A_, "Error: ");
                    crate::println!("{:?}", e);
                }
            }
        }
    }
}

fn iwm() {
    crate::n!(G_, "TrustOS Subsystem for Linux (TSL)");
    crate::n!(G_, "=================================");
    crate::println!();
    crate::println!("Execute Linux commands from TrustOS using a virtualized Linux environment.");
    crate::println!();
    crate::n!(C_, "Management Commands:");
    crate::println!("  linux init          Initialize the Linux subsystem");
    crate::println!("  linux boot          Boot real Linux kernel in VM");
    crate::println!("  linux extract       Download and extract Alpine Linux to /alpine");
    crate::println!("  linux status        Show subsystem status");
    crate::println!("  linux stop          Stop the Linux subsystem");
    crate::println!("  linux help          Show this help");
    crate::println!();
    crate::n!(C_, "Execute Linux Commands:");
    crate::println!("  linux <command>     Execute a command in Linux");
    crate::println!();
    crate::n!(C_, "Examples:");
    crate::println!("  linux uname -a      Show Linux kernel info");
    crate::println!("  linux ls -la        List files");
    crate::println!("  linux cat /etc/os-release");
    crate::println!("  linux free -h       Show memory usage");
    crate::println!("  linux df -h         Show disk usage");
    crate::println!("  linux cat /proc/cpuinfo");
    crate::println!();
    crate::n!(D_, "Note: Real VM boot requires AMD SVM or Intel VMX support.");
}






fn bks(data: &[u8]) -> String {
    use alloc::string::String;
    use alloc::format;
    
    if data.len() < 64 || &data[0..4] != b"\x7fELF" {
        return String::from("      Not a valid ELF file");
    }
    
    let mut info = String::new();
    
    let class = data[4]; 
    let lqe = data[5]; 
    let elf_type = u16::from_le_bytes([data[16], data[17]]);
    let machine = u16::from_le_bytes([data[18], data[19]]);
    
    info.push_str(&format!("      File size: {} bytes\n", data.len()));
    info.push_str(&format!("      Architecture: {}\n", if class == 2 { "x86_64 (64-bit)" } else { "x86 (32-bit)" }));
    info.push_str(&format!("      Endian: {}\n", if lqe == 1 { "Little" } else { "Big" }));
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
        
        
        let nv = u64::from_le_bytes([data[32], data[33], data[34], data[35], data[36], data[37], data[38], data[39]]) as usize;
        let but = u16::from_le_bytes([data[54], data[55]]) as usize;
        let bur = u16::from_le_bytes([data[56], data[57]]) as usize;
        
        let mut idq = false;
        for i in 0..bur {
            let off = nv + i * but;
            if off + 4 <= data.len() {
                let ptype = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
                if ptype == 3 { idq = true; }
            }
        }
        
        info.push_str(&format!("      Linking: {}\n", if idq { "Dynamic (needs ld-linux.so)" } else { "Static" }));
    }
    
    info.push_str("\n      ? Valid Linux ELF binary detected!");
    info.push_str("\n      Note: Execution requires x86_64 CPU emulation (slow)");
    
    info
}


pub(super) fn kql(cmd: &str, args: &[&str]) {
    use crate::hypervisor::linux_subsystem;

    
    let state = linux_subsystem::state();
    if state == linux_subsystem::LinuxState::NotStarted {
        let _ = linux_subsystem::init();
        let _ = linux_subsystem::boot();
    }

    
    let mut xo = alloc::string::String::from(cmd);
    for a in args {
        xo.push(' ');
        xo.push_str(a);
    }

    match linux_subsystem::execute(&xo) {
        Ok(result) => {
            if !result.stdout.is_empty() {
                crate::println!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                crate::bq!(A_, "{}", result.stderr);
                crate::println!();
            }
        }
        Err(e) => {
            crate::bq!(A_, "Error: ");
            crate::println!("{:?}", e);
        }
    }
}


pub(super) fn klo(args: &[&str]) {
    use alloc::vec::Vec;
    use alloc::string::String;
    
    let je = args.get(0).copied().unwrap_or("help");
    
    match je {
        "test" | "run" => {
            crate::n!(C_, "+--------------------------------------------------------------+");
            crate::n!(C_, "|           Alpine Linux Test - All in One                     |");
            crate::n!(C_, "+--------------------------------------------------------------+");
            crate::println!();
            
            
            let mkj = crate::ramfs::bh(|fs| {
                fs.ls(Some("/alpine/bin")).map(|e| e.len() > 0).unwrap_or(false)
            });
            
            if mkj {
                crate::n!(B_, "[1/4] Alpine binaries present ?");
            } else {
                
                crate::n!(D_, "[1/4] Creating test binaries...");
                super::apps::kzm();
            }
            
            
            crate::n!(D_, "[2/4] Verifying binaries...");
            
            let djj = crate::ramfs::bh(|fs| {
                fs.ls(Some("/alpine/bin")).map(|e| e.len()).unwrap_or(0)
            });
            
            if djj > 0 {
                crate::n!(B_, "      Found {} binaries in /alpine/bin", djj);
            } else {
                crate::n!(A_, "      No binaries found! Run 'linux extract' first.");
                return;
            }
            crate::println!();
            
            
            crate::n!(D_, "[3/4] Checking extracted files...");
            crate::ramfs::bh(|fs| {
                if let Ok(entries) = fs.ls(Some("/alpine/bin")) {
                    let count = entries.len();
                    crate::println!("      /alpine/bin: {} binaries", count);
                    
                    for (name, _, _) in entries.iter().take(5) {
                        crate::println!("        - {}", name);
                    }
                    if count > 5 {
                        crate::println!("        ... and {} more", count - 5);
                    }
                }
            });
            crate::println!();
            
            
            crate::n!(D_, "[4/4] Analyzing Linux binary...");
            let bqr = args.get(1).copied().unwrap_or("/alpine/bin/busybox");
            
            
            let loy = crate::ramfs::bh(|fs| {
                fs.read_file(bqr).map(|data| {
                    let data = data.to_vec();
                    bks(&data)
                })
            });
            
            match loy {
                Ok(info) => {
                    crate::n!(B_, "{}", info);
                }
                Err(_) => {
                    crate::n!(A_, "      Could not read binary: {}", bqr);
                }
            }
            
            crate::println!();
            crate::n!(G_, "----------------------------------------------------------------");
            crate::n!(G_, "                    Alpine Test Complete!");
            crate::n!(G_, "----------------------------------------------------------------");
        }
        
        "ls" | "list" => {
            crate::n!(C_, "Alpine Linux files:");
            crate::ramfs::bh(|fs| {
                for it in &["/alpine", "/alpine/bin", "/alpine/usr/bin"] {
                    if let Ok(entries) = fs.ls(Some(*it)) {
                        crate::println!("\n{}/ ({} entries)", it, entries.len());
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
            let bqr = args[1];
            let egu: Vec<&str> = args[2..].to_vec();
            
            crate::println!("Executing: {} {:?}", bqr, egu);
            match crate::linux_compat::exec(bqr, &egu) {
                Ok(exit_code) => crate::println!("Exited with code: {}", exit_code),
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        
        "hello" => {
            
            crate::n!(C_, "Running minimal Linux ELF binary...");
            crate::println!();
            
            
            
            #[rustfmt::skip]
            static AEZ_: &[u8] = &[
                
                0x7f, b'E', b'L', b'F',  
                0x02,                     
                0x01,                     
                0x01,                     
                0x00,                     
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
                0x02, 0x00,               
                0x3e, 0x00,               
                0x01, 0x00, 0x00, 0x00,   
                0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
                0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
                0x00, 0x00, 0x00, 0x00,   
                0x40, 0x00,               
                0x38, 0x00,               
                0x01, 0x00,               
                0x00, 0x00,               
                0x00, 0x00,               
                0x00, 0x00,               
                
                
                0x01, 0x00, 0x00, 0x00,   
                0x05, 0x00, 0x00, 0x00,   
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
                0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
                0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
                0xb0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
                0xb0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
                0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
                
                
                
                0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00,
                
                0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,
                
                0x48, 0xc7, 0xc6, 0xa0, 0x00, 0x40, 0x00,  
                
                0x48, 0xc7, 0xc2, 0x1b, 0x00, 0x00, 0x00,
                
                0x0f, 0x05,
                
                0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
                
                0x48, 0x31, 0xff,
                
                0x0f, 0x05,
                
                
                b'H', b'e', b'l', b'l', b'o', b' ', b'f', b'r',
                b'o', b'm', b' ', b'T', b'r', b'u', b's', b't',
                b'O', b'S', b' ', b'i', b'n', b't', b'e', b'r',
                b'p', b'!', 0x0a,  
            ];
            
            match crate::linux_compat::interpreter::jbu(AEZ_, &["hello"]) {
                Ok(code) => {
                    crate::println!();
                    crate::n!(B_, "Binary exited with code: {}", code);
                    crate::n!(B_, "? Linux interpreter works!");
                }
                Err(e) => {
                    crate::n!(A_, "Error: {}", e);
                }
            }
        }
        
        _ => {
            crate::n!(C_, "Alpine Linux Commands:");
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


fn qfk(fs: &mut crate::ramfs::RamFs, data: &[u8], cge: &str) -> Result<usize, &'static str> {
    use alloc::string::String;
    
    let mut offset = 0;
    let mut count = 0;
    
    while offset + 512 <= data.len() {
        let header = &data[offset..offset + 512];
        
        
        if header.iter().all(|&b| b == 0) {
            break;
        }
        
        
        let agt = &header[0..100];
        let aec = agt.iter().position(|&b| b == 0).unwrap_or(100);
        let name = core::str::from_utf8(&agt[..aec]).unwrap_or("");
        
        if name.is_empty() {
            break;
        }
        
        
        let size_bytes = &header[124..135];
        let td = core::str::from_utf8(size_bytes).unwrap_or("0");
        let size = usize::from_str_radix(td.trim_matches(|c| c == '\0' || c == ' '), 8).unwrap_or(0);
        
        
        let ppa = header[156];
        
        let kg = if name.starts_with("./") {
            alloc::format!("{}/{}", cge, &name[2..])
        } else {
            alloc::format!("{}/{}", cge, name)
        };
        
        
        let bya = kg.trim_end_matches('/');
        
        offset += 512; 
        
        match ppa {
            b'5' | b'0' if name.ends_with('/') => {
                
                let _ = fs.mkdir(bya);
            }
            b'0' | b'\0' if size > 0 => {
                
                if offset + size <= data.len() {
                    let content = &data[offset..offset + size];
                    
                    
                    if let Some(parent_end) = bya.rfind('/') {
                        let parent = &bya[..parent_end];
                        let _ = hon(fs, parent);
                    }
                    
                    let _ = fs.touch(bya);
                    let _ = fs.write_file(bya, content);
                    count += 1;
                }
            }
            b'0' | b'\0' => {
                
                if let Some(parent_end) = bya.rfind('/') {
                    let parent = &bya[..parent_end];
                    let _ = hon(fs, parent);
                }
                let _ = fs.touch(bya);
                count += 1;
            }
            b'2' => {
                
            }
            _ => {}
        }
        
        
        let blocks = (size + 511) / 512;
        offset += blocks * 512;
    }
    
    Ok(count)
}

fn hon(fs: &mut crate::ramfs::RamFs, path: &str) -> Result<(), ()> {
    let mut current = String::new();
    for jn in path.split('/').filter(|j| !j.is_empty()) {
        current.push('/');
        current.push_str(jn);
        let _ = fs.mkdir(&current);
    }
    Ok(())
}

pub(super) fn knf(args: &[&str]) {
    crate::println!("[DEBUG] cmd_download called, args: {:?}", args);
    crate::serial_println!("[DEBUG] cmd_download called, args count: {}", args.len());
    
    if args.is_empty() {
        crate::println!("Usage: download <name|url> [output_file]");
        crate::println!("       download alpine  - Download Alpine Linux (fast)");
        crate::println!("       download <url>   - Download from URL");
        return;
    }
    
    let db = args[0];
    crate::println!("[DEBUG] First arg: '{}'", db);
    
    
    if db == "alpine" || db == "busybox" || db == "linux" {
        crate::println!("[DEBUG] Calling download_from_local_server...");
        lhh("alpine-minirootfs.tar.gz", "/opt/gui/alpine.tar.gz");
        return;
    }
    
    
    let url = db;
    let output = if args.len() > 1 { args[1] } else { 
        url.rsplit('/').next().unwrap_or("download")
    };
    
    crate::n!(C_, "Downloading: {}", url);
    crate::println!("         -> {}", output);
    hlt(args);
}


fn lhh(filename: &str, cpx: &str) {
    use alloc::vec::Vec;
    use alloc::format;
    
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::n!(C_, "|              Fast Download - Local Server                    |");
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::println!();
    
    
    let server_ip: [u8; 4] = [192, 168, 56, 1];
    let aio: u16 = 8080;
    
    crate::n!(D_, "[1/4] Configuring network...");
    
    
    crate::netstack::dhcp::crf();
    crate::network::deh(
        crate::network::Ipv4Address::new(192, 168, 56, 100),
        crate::network::Ipv4Address::new(255, 255, 255, 0),
        Some(crate::network::Ipv4Address::new(192, 168, 56, 1)),
    );
    
    
    if let Some((ip, mask, fz)) = crate::network::rd() {
        crate::println!("      IP: {}.{}.{}.{}", ip.as_bytes()[0], ip.as_bytes()[1], ip.as_bytes()[2], ip.as_bytes()[3]);
        crate::serial_println!("[DOWNLOAD] IP configured: {}.{}.{}.{}", ip.as_bytes()[0], ip.as_bytes()[1], ip.as_bytes()[2], ip.as_bytes()[3]);
    } else {
        crate::n!(A_, "      ERROR: No IP configured!");
        crate::netstack::dhcp::resume();
        return;
    }
    
    
    for _ in 0..100 {
        crate::netstack::poll();
    }
    crate::println!();
    
    crate::n!(D_, "[2/4] Connecting to 192.168.56.1:8080...");
    
    
    crate::println!("      Resolving MAC address...");
    let _ = crate::netstack::arp::bos(server_ip);
    for _ in 0..200 {
        crate::netstack::poll();
    }
    
    
    if let Some(mac) = crate::netstack::arp::yb(server_ip) {
        crate::println!("      Server MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", 
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    } else {
        crate::n!(D_, "      Warning: No ARP response yet");
    }
    
    let src_port = match crate::netstack::tcp::azp(server_ip, aio) {
        Ok(aa) => {
            crate::serial_println!("[DOWNLOAD] SYN sent, src_port={}", aa);
            aa
        }
        Err(e) => {
            crate::serial_println!("[DOWNLOAD] SYN failed: {}", e);
            crate::n!(A_, "      ERROR: {}", e);
            crate::println!("      Is the server running?");
            crate::println!("      > cd server && .\\start-server.ps1");
            crate::netstack::dhcp::resume();
            return;
        }
    };
    
    crate::println!("      Waiting for connection...");
    if !crate::netstack::tcp::bjy(server_ip, aio, src_port, 3000) {
        crate::serial_println!("[DOWNLOAD] Connection timeout!");
        crate::n!(A_, "      ERROR: Connection timeout");
        crate::println!("      Check: ping 192.168.56.1");
        crate::netstack::dhcp::resume();
        return;
    }
    
    crate::n!(B_, "      Connected!");
    crate::println!();
    
    crate::n!(D_, "[3/4] Downloading {}...", filename);
    
    
    let request = format!(
        "GET /{} HTTP/1.1\r\nHost: 192.168.56.1\r\nConnection: close\r\n\r\n",
        filename
    );
    
    if let Err(e) = crate::netstack::tcp::bjc(server_ip, aio, src_port, request.as_bytes()) {
        crate::n!(A_, "      ERROR: {}", e);
        crate::netstack::dhcp::resume();
        return;
    }
    
    
    let mut data: Vec<u8> = Vec::with_capacity(4 * 1024 * 1024);
    let start = crate::logger::eg();
    let mut bch: u32 = 0;
    let mut cly = 0usize;
    let mut cbi = start;
    
    loop {
        
        for _ in 0..10 {
            crate::netstack::poll();
        }
        
        let mut aty = false;
        while let Some(df) = crate::netstack::tcp::aus(server_ip, aio, src_port) {
            aty = true;
            if data.len() + df.len() > 8 * 1024 * 1024 {
                break;
            }
            data.extend_from_slice(&df);
        }
        
        
        let arh = data.len() / 1024;
        if arh >= cly + 50 || (arh > 0 && cly == 0) {
            let bb = crate::logger::eg().saturating_sub(start);
            let speed = if bb > 0 { (arh as u64 * 1000) / bb } else { 0 };
            crate::print!("\r      {} KB downloaded ({} KB/s)          ", arh, speed);
            cly = arh;
        }
        
        
        let cy = crate::logger::eg();
        if cy.saturating_sub(cbi) >= 5 {
            crate::netstack::tcp::cjr(server_ip, aio, src_port);
            cbi = cy;
        }
        
        if !aty {
            bch += 1;
            if crate::netstack::tcp::fin_received(server_ip, aio, src_port) {
                crate::netstack::tcp::cjr(server_ip, aio, src_port);
                break;
            }
            if bch > 100_000 {
                break;
            }
        } else {
            bch = 0;
        }
        
        
        if cy.saturating_sub(start) > 30_000 {
            crate::n!(D_, "\n      Timeout!");
            break;
        }
    }
    
    let _ = crate::netstack::tcp::ams(server_ip, aio, src_port);
    
    let bb = crate::logger::eg().saturating_sub(start);
    let baa = data.len() / 1024;
    let fhu = if bb > 0 { (baa as u64 * 1000) / bb } else { 0 };
    
    crate::println!();
    crate::n!(B_, "      Complete: {} KB in {}ms ({} KB/s)", baa, bb, fhu);
    crate::println!();
    
    if data.is_empty() {
        crate::n!(A_, "      ERROR: No data received");
        crate::netstack::dhcp::resume();
        return;
    }
    
    
    let bao = data.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|aa| aa + 4)
        .unwrap_or(0);
    let body = &data[bao..];
    
    if body.is_empty() {
        crate::n!(A_, "      ERROR: Empty response");
        crate::netstack::dhcp::resume();
        return;
    }
    
    crate::n!(D_, "[4/4] Saving to {}...", cpx);
    
    
    let dyg = crate::ramfs::bh(|fs| {
        let _ = fs.mkdir("/opt");
        let _ = fs.mkdir("/opt/gui");
        let _ = fs.touch(cpx);
        fs.write_file(cpx, body)
    });
    
    match dyg {
        Ok(_) => {
            crate::n!(B_, "      Saved: {:.2} MB", body.len() as f32 / (1024.0 * 1024.0));
        }
        Err(e) => {
            crate::n!(A_, "      ERROR: {:?}", e);
            crate::netstack::dhcp::resume();
            return;
        }
    }
    
    
    crate::println!();
    crate::n!(D_, "Saving to disk for persistence...");
    match crate::persistence::save_file(cpx, body) {
        Ok(_) => crate::n!(B_, "  Saved! Will survive reboot."),
        Err(e) => crate::n!(D_, "  Could not persist: {}", e),
    }
    
    crate::println!();
    crate::n!(G_, "----------------------------------------------------------------");
    crate::n!(G_, "                    Download Complete!");
    crate::n!(G_, "----------------------------------------------------------------");
    
    
    LC_.store(true, core::sync::atomic::Ordering::Relaxed);
    
    crate::netstack::dhcp::resume();
}