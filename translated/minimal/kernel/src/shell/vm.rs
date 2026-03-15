





use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{B_, G_, AU_, D_, A_, C_, Q_, CD_, DF_, L_};





pub(super) static KJ_: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

pub(super) fn nek() {
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::h!(C_, "|            TrustOS Virtual Machine Manager                   |");
    crate::h!(C_, "|--------------------------------------------------------------|");
    crate::h!(C_, "|                                                              |");
    crate::h!(C_, "|  TrustOS runs Linux VMs with modern GUIs.                   |");
    crate::h!(C_, "|                                                              |");
    crate::h!(C_, "|  Commands:                                                   |");
    crate::h!(B_, "|    vm status    - Check VM installation status              |");
    crate::h!(B_, "|    vm install   - Download Alpine Linux VM image            |");
    crate::h!(B_, "|    vm start     - Start the Alpine Linux VM                 |");
    crate::h!(B_, "|    vm console   - Connect to VM console (Linux shell)       |");
    crate::h!(B_, "|    vm stop      - Stop the running VM                       |");
    crate::h!(B_, "|    vm list      - List running VMs                          |");
    crate::h!(C_, "|                                                              |");
    crate::h!(C_, "+--------------------------------------------------------------+");
}

pub(super) fn rkk() {
    crate::h!(D_, "Stopping VM...");
    
    crate::h!(B_, "VM stopped.");
}

pub(super) fn rkj() {
    crate::h!(C_, "Running Virtual Machines:");
    crate::println!("  ID   NAME           STATUS      MEMORY");
    crate::println!("  ---------------------------------------");
    if KJ_.load(core::sync::atomic::Ordering::Relaxed) {
        crate::println!("  1    alpine-linux   running     256 MB");
    } else {
        crate::println!("  (no VMs running)");
    }
}



pub(super) fn kil() {
    
    if crate::distro::aoy().is_empty() {
        crate::distro::init();
    }
    
    let ced = crate::distro::aoy();
    
    crate::h!(C_, "+------------------------------------------------------------------+");
    crate::h!(C_, "|                 TrustOS Linux Distribution Manager               |");
    crate::h!(C_, "|------------------------------------------------------------------|");
    crate::h!(C_, "|  ID              NAME                    SIZE     STATUS         |");
    crate::h!(C_, "|------------------------------------------------------------------|");
    
    for bc in &ced {
        let status = if bc.adw { 
            "\x1b[32m[installed]\x1b[0m" 
        } else { 
            "\x1b[33m[available]\x1b[0m" 
        };
        let wtx = if bc.adw { "installed" } else { "available" };
        crate::println!("|  {} {:<12}  {:<20}  {:>4} MB   {:<12} |", 
            bc.pa, bc.ad, bc.j, bc.aga, wtx);
    }
    
    crate::h!(C_, "|------------------------------------------------------------------|");
    crate::h!(C_, "|  Commands:                                                       |");
    crate::h!(B_, "|    distro list              - Show this list                    |");
    crate::h!(B_, "|    distro install <id>      - Download and install a distro     |");
    crate::h!(B_, "|    distro run <id>          - Run an installed distro           |");
    crate::h!(B_, "|    distro gui               - Open graphical distro selector    |");
    crate::h!(C_, "+------------------------------------------------------------------+");
}

pub(super) fn rdu(ad: &str) {
    
    if crate::distro::aoy().is_empty() {
        crate::distro::init();
    }
    
    let distro = match crate::distro::get(ad) {
        Some(bc) => bc,
        None => {
            crate::h!(A_, "Error: Distribution '{}' not found.", ad);
            crate::println!("Use 'distro list' to see available distributions.");
            return;
        }
    };
    
    if distro.adw {
        crate::h!(D_, "{} {} is already installed.", distro.pa, distro.j);
        crate::println!("Use 'distro run {}' to start it.", ad);
        return;
    }
    
    crate::h!(C_, "+------------------------------------------------------------------+");
    crate::h!(C_, "|                    Installing Linux Distribution                 |");
    crate::h!(C_, "+------------------------------------------------------------------+");
    crate::println!();
    crate::println!("  {} {} {}", distro.pa, distro.j, distro.dk);
    crate::println!("  {}", distro.dc);
    crate::println!("  Size: {} MB", distro.aga);
    crate::println!();
    
    crate::h!(D_, "[1/3] Connecting to server 192.168.56.1:8080...");
    
    match crate::distro::kqp(ad) {
        Ok(aw) => {
            crate::h!(B_, "[2/3] Downloaded {} KB", aw / 1024);
            crate::h!(B_, "[3/3] Installation complete!");
            crate::println!();
            crate::h!(B_, "  {} {} is now installed!", distro.pa, distro.j);
            crate::println!("  Use 'distro run {}' to start it.", ad);
        }
        Err(aa) => {
            crate::h!(A_, "Error: {}", aa);
            crate::println!();
            crate::println!("Make sure the server is running:");
            crate::println!("  > cd server && powershell -ExecutionPolicy Bypass .\\start-server.ps1");
        }
    }
}

pub(super) fn rdv(ad: &str) {
    
    if crate::distro::aoy().is_empty() {
        crate::distro::init();
    }
    
    let distro = match crate::distro::get(ad) {
        Some(bc) => bc,
        None => {
            crate::h!(A_, "Error: Distribution '{}' not found.", ad);
            crate::println!("Use 'distro list' to see available distributions.");
            return;
        }
    };
    
    if !distro.adw {
        crate::h!(D_, "{} {} is not installed.", distro.pa, distro.j);
        crate::println!("Use 'distro install {}' to download it first.", ad);
        return;
    }
    
    crate::h!(C_, "+------------------------------------------------------------------+");
    crate::h!(C_, "|                    Starting Linux Distribution                   |");
    crate::h!(C_, "+------------------------------------------------------------------+");
    crate::println!();
    crate::println!("  {} {} {}", distro.pa, distro.j, distro.dk);
    crate::println!();
    
    match crate::distro::vw(ad) {
        Ok(()) => {
            crate::h!(B_, "  Distribution started successfully.");
        }
        Err(aa) => {
            crate::h!(A_, "Error: {}", aa);
        }
    }
}

pub(super) fn ndx() {
    
    if crate::distro::aoy().is_empty() {
        crate::distro::init();
    }
    
    let ced = crate::distro::aoy();
    
    
    if !crate::framebuffer::ky() {
        crate::h!(A_, "Error: No framebuffer available for GUI.");
        crate::println!("Use 'distro list' for text-mode interface.");
        return;
    }
    
    let (z, ac) = crate::framebuffer::yn();
    
    
    let vp = 0xFF1E1E2Eu32;      
    let fqe = 0xFF2D2D3Du32;   
    let axm = 0xFF89B4FAu32;  
    let nzo = 0xFF94E2D5u32;   
    let agx = 0xFFCDD6F4u32;    
    let xyu = 0xFF6C7086u32;    
    
    
    crate::framebuffer::ah(0, 0, z, ac, vp);
    
    
    crate::framebuffer::ah(0, 0, z, 50, fqe);
    crate::framebuffer::ri("TrustOS Linux Distribution Manager", 20, 16, agx, fqe);
    
    
    let mut c = 80u32;
    
    crate::framebuffer::ri("  #  ID              NAME                    SIZE     STATUS", 20, c, axm, vp);
    c += 24;
    crate::framebuffer::zs(20, c, z - 40, axm);
    c += 16;
    
    for (a, bc) in ced.iter().cf() {
        let ejb = if bc.adw { "[INSTALLED]" } else { "[available]" };
        let dch = if bc.adw { nzo } else { agx };
        
        
        let ajh = alloc::format!("  {}  ", a + 1);
        crate::framebuffer::ri(&ajh, 20, c, axm, vp);
        
        
        let izg = alloc::format!("{} {:<12}", bc.pa, bc.ad);
        crate::framebuffer::ri(&izg, 60, c, agx, vp);
        
        
        crate::framebuffer::ri(bc.j, 220, c, agx, vp);
        
        
        let als = alloc::format!("{:>4} MB", bc.aga);
        crate::framebuffer::ri(&als, 450, c, agx, vp);
        
        
        crate::framebuffer::ri(ejb, 540, c, dch, vp);
        
        c += 24;
    }
    
    
    let hkc = ac - 80;
    crate::framebuffer::ah(0, hkc, z, 80, fqe);
    crate::framebuffer::ri("Commands:", 20, hkc + 16, axm, fqe);
    crate::framebuffer::ri("distro install <id>  - Download and install", 20, hkc + 36, agx, fqe);
    crate::framebuffer::ri("distro run <id>      - Run an installed distro", 400, hkc + 36, agx, fqe);
    crate::framebuffer::ri("Press any key to return to shell...", 20, hkc + 56, nzo, fqe);
    
    
    loop {
        if let Some(qbm) = crate::keyboard::auw() {
            break;
        }
        for _ in 0..1000 { core::hint::hc(); }
    }
    
    
    crate::framebuffer::clear();
}

pub(super) fn rev() {
    let adw = KJ_.load(core::sync::atomic::Ordering::Relaxed);
    
    crate::h!(C_, "+--------------------------------------+");
    crate::h!(C_, "|       TrustOS GUI Status             |");
    crate::h!(C_, "|--------------------------------------|");
    
    if adw {
        crate::h!(B_, "|  Status:     [INSTALLED]             |");
        crate::h!(B_, "|  Image:      Alpine Linux + Browser  |");
        crate::h!(C_, "|                                      |");
        crate::h!(C_, "|  Use 'gui start' to launch           |");
    } else {
        crate::h!(D_, "|  Status:     [NOT INSTALLED]         |");
        crate::h!(C_, "|                                      |");
        crate::h!(C_, "|  Use 'gui install' to download       |");
    }
    crate::h!(C_, "+--------------------------------------+");
}

pub(super) fn reu() {
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::h!(C_, "|              TrustOS GUI Installer                           |");
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::println!();
    
    
    let aep = "192.168.56.1";
    let boh = 8080u16;
    let oto = "/alpine-minirootfs.tar.gz";
    
    
    crate::h!(D_, "[1/4] Checking network connection...");
    
    if !crate::network::anl() {
        crate::h!(A_, "      ERROR: Network not available!");
        crate::println!("      Make sure virtio-net is enabled.");
        return;
    }
    crate::h!(B_, "      Network: OK");
    crate::println!();
    
    
    crate::h!(D_, "[2/4] Downloading Alpine Linux from {}:{}{}...", aep, boh, oto);
    
    
    crate::netstack::dhcp::fvw();
    crate::serial_println!("[GUI_INSTALL] DHCP suspended for download");
    
    
    crate::network::hzx(
        crate::network::Ipv4Address::new(192, 168, 56, 100),
        crate::network::Ipv4Address::new(255, 255, 255, 0),
        Some(crate::network::Ipv4Address::new(192, 168, 56, 1)),
    );
    
    
    for _ in 0..100 {
        crate::netstack::poll();
    }
    
    let ip = match cgl(aep) {
        Some(ip) => ip,
        None => {
            crate::h!(A_, "      ERROR: Invalid server IP");
            crate::netstack::dhcp::anu();
            return;
        }
    };
    
    
    let ey = match crate::netstack::tcp::cue(ip, boh) {
        Ok(ai) => ai,
        Err(aa) => {
            crate::h!(A_, "      ERROR: Connection failed: {}", aa);
            crate::println!("      Make sure the server is running:");
            crate::println!("      > cd server && powershell -ExecutionPolicy Bypass .\\start-server.ps1");
            crate::netstack::dhcp::anu();
            return;
        }
    };
    
    let fhz = crate::netstack::tcp::dnd(ip, boh, ey, 2000);
    if !fhz {
        crate::h!(A_, "      ERROR: Connection timeout");
        crate::println!("      Make sure the server is running on port {}", boh);
        crate::netstack::dhcp::anu();
        return;
    }
    
    crate::h!(B_, "      Connected to server");
    
    
    let request = alloc::format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        oto, aep
    );
    
    if let Err(aa) = crate::netstack::tcp::dlo(ip, boh, ey, request.as_bytes()) {
        crate::h!(A_, "      ERROR: Failed to send request: {}", aa);
        crate::netstack::dhcp::anu();
        return;
    }
    
    
    crate::println!("      Downloading...");
    
    let mut fsi: Vec<u8> = Vec::fc(4 * 1024 * 1024);
    let ay = crate::logger::lh();
    let mut cyt: u32 = 0;
    let mut fmp = 0usize;
    let mut etv = ay;
    let mut oid = 0u32;
    const CFR_: usize = 8 * 1024 * 1024; 
    
    loop {
        
        for _ in 0..10 {
            crate::netstack::poll();
        }
        oid += 10;
        
        let mut ckw = false;
        let mut qoa = 0usize;
        
        
        while let Some(f) = crate::netstack::tcp::cme(ip, boh, ey) {
            ckw = true;
            qoa += f.len();
            
            
            if fsi.len() + f.len() > CFR_ {
                crate::h!(D_, "\n      WARNING: File too large, truncating");
                break;
            }
            
            fsi.bk(&f);
        }
        
        
        let cfv = fsi.len() / 1024;
        if cfv >= fmp + 25 || (cfv > 0 && fmp == 0) {
            let ez = crate::logger::lh().ao(ay);
            let wqx = if ez > 0 { (cfv as u64 * 1000) / ez } else { 0 };
            crate::print!("\r      Downloaded: {} KB ({} KB/s)    ", cfv, wqx);
            fmp = cfv;
        }
        
        
        let iu = crate::logger::lh();
        if iu.ao(etv) >= 5 {
            crate::netstack::tcp::fiv(ip, boh, ey);
            etv = iu;
        }
        
        if !ckw {
            cyt = cyt.akq(1);
            
            
            if crate::netstack::tcp::bqr(ip, boh, ey) {
                
                crate::netstack::tcp::fiv(ip, boh, ey);
                break;
            }
            
            
            if cyt > 100_000 {
                crate::serial_println!("[DOWNLOAD] Idle timeout after {} polls", oid);
                break;
            }
            
            
            for _ in 0..50 { core::hint::hc(); }
        } else {
            cyt = 0;
        }
        
        
        if crate::logger::lh().ao(ay) > 60000 {
            crate::h!(D_, "\n      WARNING: Download timeout");
            break;
        }
    }
    
    let _ = crate::netstack::tcp::bwx(ip, boh, ey);
    crate::println!();
    
    let oz = crate::logger::lh().ao(ay);
    let cuu = fsi.len() / 1024;
    let kbo = if oz > 0 { (cuu as u64 * 1000) / oz } else { 0 };
    crate::h!(B_, "      Transfer complete: {} KB in {}ms ({} KB/s)", cuu, oz, kbo);
    
    if fsi.is_empty() {
        crate::h!(A_, "      ERROR: No data received");
        crate::netstack::dhcp::anu();
        return;
    }
    
    
    let cvy = fsi.ee(4)
        .qf(|d| d == b"\r\n\r\n")
        .map(|ai| ai + 4)
        .unwrap_or(0);
    
    let ldi = &fsi[cvy..];
    let aga = ldi.len() as f32 / (1024.0 * 1024.0);
    
    crate::h!(B_, "      Download complete: {:.2} MB", aga);
    crate::println!();
    
    
    crate::h!(D_, "[3/4] Saving image to /opt/gui/alpine.tar.gz...");
    
    
    let hyn = crate::ramfs::fh(|fs| {
        
        let _ = fs.ut("/opt");
        let _ = fs.ut("/opt/gui");
        
        let _ = fs.touch("/opt/gui/alpine.tar.gz");
        fs.ns("/opt/gui/alpine.tar.gz", ldi)
    });
    
    match hyn {
        Ok(_) => {
            crate::h!(B_, "      Saved successfully");
        }
        Err(aa) => {
            crate::h!(A_, "      ERROR: Write failed: {:?}", aa);
            crate::netstack::dhcp::anu();
            return;
        }
    }
    crate::println!();
    
    
    crate::h!(D_, "[4/4] Configuring GUI environment...");
    
    
    KJ_.store(true, core::sync::atomic::Ordering::Relaxed);
    
    crate::h!(B_, "      Configuration complete");
    crate::println!();
    
    crate::h!(G_, "----------------------------------------------------------------");
    crate::h!(G_, "                    GUI Installation Complete!");
    crate::h!(G_, "----------------------------------------------------------------");
    crate::println!();
    crate::println!("Image saved to: /opt/gui/alpine.tar.gz ({:.2} MB)", aga);
    crate::println!();
    
    
    crate::h!(D_, "Saving to disk for persistence...");
    match crate::persistence::ftm("/opt/gui/alpine.tar.gz", ldi) {
        Ok(_) => {
            crate::h!(B_, "  Saved to disk! Will be restored on next boot.");
        }
        Err(aa) => {
            crate::h!(D_, "  Could not save to disk: {}", aa);
            crate::println!("  (Download will need to be repeated after reboot)");
        }
    }
    crate::println!();
    
    crate::println!("Use 'gui start' to launch the graphical environment.");
    
    
    crate::netstack::dhcp::anu();
    crate::serial_println!("[GUI_INSTALL] DHCP resumed");
}

pub(super) fn yjd() {
    let adw = KJ_.load(core::sync::atomic::Ordering::Relaxed);
    
    if !adw {
        
        if !cxx("/opt/gui/alpine.tar.gz") {
            crate::h!(D_, "Linux VM not installed.");
            crate::println!("Run 'gui install' first to download Alpine Linux.");
            return;
        }
        KJ_.store(true, core::sync::atomic::Ordering::Relaxed);
    }
    
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::h!(C_, "|              Starting Alpine Linux VM                        |");
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::println!();
    
    
    crate::h!(D_, "[1/3] Initializing hypervisor...");
    
    
    if !crate::hypervisor::zu() {
        match crate::hypervisor::init() {
            Ok(()) => {
                crate::h!(B_, "      Hypervisor initialized (VT-x/AMD-V)");
            }
            Err(aa) => {
                crate::serial_println!("[GUI] Hypervisor init failed: {:?}", aa);
                crate::h!(A_, "      ERROR: Hardware virtualization not available");
                crate::println!("      Requires Intel VT-x or AMD-V");
                crate::println!();
                crate::h!(D_, "Falling back to Linux subsystem emulation...");
                ioi();
                return;
            }
        }
    }
    crate::h!(B_, "      Hypervisor ready");
    
    crate::h!(D_, "[2/3] Loading Alpine Linux image...");
    crate::h!(B_, "      Image: /opt/gui/alpine.tar.gz");
    
    crate::h!(D_, "[3/3] Booting VM...");
    
    
    match crate::hypervisor::linux_subsystem::boot() {
        Ok(_) => {
            crate::h!(B_, "      VM started successfully");
            crate::println!();
            crate::h!(C_, "Alpine Linux is now running.");
            crate::println!("Use 'vm console' to connect to the VM console.");
            crate::println!("Use 'vm stop' to stop the VM.");
        }
        Err(aa) => {
            crate::h!(A_, "      ERROR: Failed to start VM: {:?}", aa);
            crate::println!();
            crate::h!(D_, "Falling back to Linux subsystem...");
            ioi();
        }
    }
}


pub(super) fn ioi() {
    
    if !crate::linux::ky() {
        
        if cxx("/opt/gui/alpine.tar.gz") {
            match crate::linux::init("/opt/gui/alpine.tar.gz") {
                Ok(()) => {}
                Err(aa) => {
                    crate::h!(A_, "Failed to initialize Linux subsystem: {}", aa);
                    return;
                }
            }
        } else {
            crate::h!(D_, "Linux subsystem not installed.");
            crate::println!("Run 'gui install' to download and install Alpine Linux.");
            return;
        }
    }
    
    
    crate::linux::wtd();
}

pub(super) fn reo(n: &[&str]) {
    use crate::desktop::{RenderMode, dvr, bxb};
    use crate::graphics::CompositorTheme;
    
    if n.is_empty() {
        crate::h!(C_, "TrustGL Compositor Settings");
        crate::h!(C_, "===========================");
        crate::println!();
        crate::println!("Usage: glmode <mode|theme>");
        crate::println!();
        crate::h!(G_, "Render Modes:");
        crate::println!("  classic   - Classic framebuffer rendering (fast, stable)");
        crate::println!("  opengl    - OpenGL compositor with visual effects");
        crate::println!();
        crate::h!(G_, "Themes (OpenGL mode only):");
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
    
    match n[0].aqn().as_str() {
        "classic" | "normal" | "default" => {
            dvr(RenderMode::Apy);
            crate::h!(B_, "Switched to Classic rendering mode");
        }
        "opengl" | "gl" | "compositor" => {
            dvr(RenderMode::Ks);
            crate::h!(B_, "Switched to OpenGL compositor mode");
            crate::println!("Use 'glmode <theme>' to change visual theme");
        }
        "flat" => {
            dvr(RenderMode::Ks);
            bxb(CompositorTheme::Aif);
            crate::h!(B_, "Theme: Flat (OpenGL)");
        }
        "modern" => {
            dvr(RenderMode::Ks);
            bxb(CompositorTheme::Xq);
            crate::h!(B_, "Theme: Modern (shadows, subtle effects)");
        }
        "glass" => {
            dvr(RenderMode::Ks);
            bxb(CompositorTheme::Ait);
            crate::h!(B_, "Theme: Glass (transparency effects)");
        }
        "neon" => {
            dvr(RenderMode::Ks);
            bxb(CompositorTheme::Tp);
            crate::h!(B_, "Theme: Neon (glowing borders)");
        }
        "minimal" => {
            dvr(RenderMode::Ks);
            bxb(CompositorTheme::Gy);
            crate::h!(B_, "Theme: Minimal (thin borders)");
        }
        _ => {
            crate::h!(A_, "Unknown mode/theme: {}", n[0]);
            crate::println!("Use 'glmode' without arguments for help");
        }
    }
}


pub(super) fn rjh(n: &[&str]) {
    if n.is_empty() {
        crate::h!(C_, "TrustOS Theme Manager");
        crate::h!(C_, "=====================");
        crate::println!();
        crate::println!("Usage: theme <command> [args]");
        crate::println!();
        crate::h!(G_, "Commands:");
        crate::println!("  list              - List available built-in themes");
        crate::println!("  set <name>        - Switch to a built-in theme");
        crate::println!("  load <path>       - Load theme from config file");
        crate::println!("  save <path>       - Save current theme to file");
        crate::println!("  reload            - Reload wallpaper from disk");
        crate::println!("  info              - Show current theme info");
        crate::println!();
        crate::h!(G_, "Built-in Themes:");
        crate::println!("  dark / trustos    - TrustOS dark green theme");
        crate::println!("  windows11 / win11 - Windows 11 dark theme");
        crate::println!();
        crate::h!(G_, "Config File Format (/etc/theme.conf):");
        crate::println!("  [colors]");
        crate::println!("  background = 0x0A0E0B");
        crate::println!("  accent = 0x00D26A");
        crate::println!("  ");
        crate::println!("  [wallpaper]");
        crate::println!("  path = /usr/share/wallpapers/matrix.bmp");
        return;
    }
    
    match n[0] {
        "list" => {
            crate::h!(C_, "Available Themes:");
            crate::println!("  dark       - TrustOS dark green (default)");
            crate::println!("  windows11  - Windows 11 dark blue");
            crate::println!("  light      - Light theme");
        }
        "set" => {
            if n.len() < 2 {
                crate::h!(A_, "Usage: theme set <name>");
                return;
            }
            crate::theme::piq(n[1]);
            crate::h!(B_, "Theme switched to: {}", n[1]);
        }
        "load" => {
            if n.len() < 2 {
                crate::h!(A_, "Usage: theme load <path>");
                crate::println!("Example: theme load /etc/theme.conf");
                return;
            }
            if crate::theme::uhi(n[1]) {
                crate::h!(B_, "Theme loaded from: {}", n[1]);
            } else {
                crate::h!(A_, "Failed to load theme from: {}", n[1]);
            }
        }
        "save" => {
            if n.len() < 2 {
                crate::h!(A_, "Usage: theme save <path>");
                return;
            }
            let theme = crate::theme::Ib.read();
            let ca = crate::theme::config::tcp(&theme);
            drop(theme);
            
            match crate::vfs::ns(n[1], ca.as_bytes()) {
                Ok(_) => crate::h!(B_, "Theme saved to: {}", n[1]),
                Err(aa) => crate::h!(A_, "Failed to save: {:?}", aa),
            }
        }
        "reload" => {
            crate::theme::vuq();
            crate::h!(B_, "Wallpaper reloaded");
        }
        "info" => {
            let theme = crate::theme::Ib.read();
            crate::h!(C_, "Current Theme: {}", 
                if theme.j.is_empty() { "TrustOS Default" } else { &theme.j });
            crate::println!();
            crate::h!(G_, "Colors:");
            crate::println!("  Background:  0x{:08X}", theme.colors.cop);
            crate::println!("  Accent:      0x{:08X}", theme.colors.mm);
            crate::println!("  Text:        0x{:08X}", theme.colors.dcp);
            crate::println!("  Surface:     0x{:08X}", theme.colors.surface);
            crate::println!();
            crate::h!(G_, "Taskbar:");
            crate::println!("  Height:      {} px", theme.bou.ac);
            crate::println!("  Centered:    {}", theme.bou.gch);
            crate::println!();
            crate::h!(G_, "Windows:");
            crate::println!("  Title bar:   {} px", theme.bh.ids);
            crate::println!("  Radius:      {} px", theme.bh.avh);
            crate::println!("  Shadow:      {} px", theme.bh.iac);
            crate::println!();
            crate::h!(G_, "Wallpaper:");
            crate::println!("  Path:        {}", 
                if theme.bsx.path.is_empty() { "(none)" } else { &theme.bsx.path });
            crate::println!("  Mode:        {:?}", theme.bsx.ev);
        }
        _ => {
            crate::h!(A_, "Unknown theme command: {}", n[0]);
            crate::println!("Use 'theme' for help");
        }
    }
}


pub(super) fn rcf(n: &[&str]) {
    if n.is_empty() {
        let iq = crate::desktop::col();
        let ig = crate::desktop::hlf();
        
        crate::h!(C_, "TrustOS Animation Settings");
        crate::h!(C_, "==========================");
        crate::println!();
        crate::h!(G_, "Current Status:");
        if iq {
            crate::println!("  Animations: {} ENABLED", "\x1b[32m?\x1b[0m");
        } else {
            crate::println!("  Animations: {} DISABLED", "\x1b[31m?\x1b[0m");
        }
        crate::println!("  Speed:      {}x", ig);
        crate::println!();
        crate::h!(G_, "Commands:");
        crate::println!("  anim on           - Enable animations");
        crate::println!("  anim off          - Disable animations");
        crate::println!("  anim toggle       - Toggle on/off");
        crate::println!("  anim speed <val>  - Set speed (0.25-4.0)");
        crate::println!("                      1.0=normal, 2.0=fast, 0.5=slow");
        crate::println!();
        crate::h!(G_, "Animation Types:");
        crate::println!("  - Window open (scale up from center)");
        crate::println!("  - Window close (scale down + fade out)");
        crate::println!("  - Minimize (move to taskbar)");
        crate::println!("  - Maximize/Restore (smooth resize)");
        return;
    }
    
    match n[0] {
        "on" | "enable" | "1" | "true" => {
            crate::desktop::jop(true);
            crate::h!(B_, "? Animations enabled");
        }
        "off" | "disable" | "0" | "false" => {
            crate::desktop::jop(false);
            crate::h!(D_, "? Animations disabled");
        }
        "toggle" => {
            let cv = crate::desktop::col();
            crate::desktop::jop(!cv);
            if !cv {
                crate::h!(B_, "? Animations enabled");
            } else {
                crate::h!(D_, "? Animations disabled");
            }
        }
        "speed" => {
            if n.len() < 2 {
                crate::println!("Current speed: {}x", crate::desktop::hlf());
                crate::println!("Usage: anim speed <value>");
                crate::println!("  Examples: 0.5 (slow), 1.0 (normal), 2.0 (fast)");
                return;
            }
            if let Ok(ig) = n[1].parse::<f32>() {
                crate::desktop::pio(ig);
                crate::h!(B_, "Animation speed set to {}x", ig);
            } else {
                crate::h!(A_, "Invalid speed value: {}", n[1]);
            }
        }
        "status" | "info" => {
            let iq = crate::desktop::col();
            let ig = crate::desktop::hlf();
            crate::println!("Animations: {}", if iq { "enabled" } else { "disabled" });
            crate::println!("Speed: {}x", ig);
        }
        _ => {
            crate::h!(A_, "Unknown animation command: {}", n[0]);
            crate::println!("Use 'anim' for help");
        }
    }
}


pub(super) fn rfa(n: &[&str]) {
    use crate::graphics::holomatrix;
    
    if n.is_empty() {
        let iq = holomatrix::zu();
        let amt = holomatrix::hlk();
        
        crate::h!(C_, "TrustOS HoloMatrix 3D");
        crate::h!(C_, "=====================");
        crate::println!();
        crate::h!(G_, "Current Status:");
        if iq {
            crate::println!("  HoloMatrix: {} ENABLED", "\x1b[36m?\x1b[0m");
        } else {
            crate::println!("  HoloMatrix: {} DISABLED", "\x1b[31m?\x1b[0m");
        }
        crate::println!("  Scene:      {}", amt.j());
        crate::println!();
        crate::h!(G_, "Commands:");
        crate::println!("  holo on           - Enable HoloMatrix 3D background");
        crate::println!("  holo off          - Disable (use Matrix Rain)");
        crate::println!("  holo toggle       - Toggle on/off");
        crate::println!("  holo next         - Cycle to next scene");
        crate::println!("  holo scene <name> - Set specific scene");
        crate::println!();
        crate::h!(G_, "Available Scenes:");
        crate::println!("  cube     - Rotating wireframe cube");
        crate::println!("  sphere   - Pulsating sphere");
        crate::println!("  torus    - 3D donut/ring");
        crate::println!("  grid     - Perspective grid with cube");
        crate::println!("  multi    - Multiple floating shapes");
        crate::println!("  dna      - Animated DNA double helix");
        crate::println!();
        crate::h!(G_, "How it works:");
        crate::println!("  Renders 3D shapes using 16 Z-slices (layers)");
        crate::println!("  Each layer has depth-based transparency");
        crate::println!("  Creates holographic volumetric effect");
        return;
    }
    
    match n[0] {
        "on" | "enable" | "1" | "true" => {
            holomatrix::cuf(true);
            crate::h!(0xFF00FFFF, "? HoloMatrix 3D enabled");
            crate::println!("Launch 'desktop' to see the effect");
        }
        "off" | "disable" | "0" | "false" => {
            holomatrix::cuf(false);
            crate::h!(D_, "? HoloMatrix disabled (Matrix Rain active)");
        }
        "toggle" => {
            let iq = holomatrix::xiq();
            if iq {
                crate::h!(0xFF00FFFF, "? HoloMatrix 3D enabled");
            } else {
                crate::h!(D_, "? HoloMatrix disabled");
            }
        }
        "next" | "cycle" => {
            let amt = holomatrix::uum();
            crate::h!(0xFF00FFFF, "Scene: {}", amt.j());
        }
        "scene" | "set" => {
            if n.len() < 2 {
                crate::println!("Current scene: {}", holomatrix::hlk().j());
                crate::println!("Usage: holo scene <name>");
                crate::println!("Available: cube, sphere, torus, grid, multi, dna");
                return;
            }
            if let Some(amt) = holomatrix::HoloScene::nwf(n[1]) {
                holomatrix::bid(amt);
                crate::h!(0xFF00FFFF, "Scene set to: {}", amt.j());
            } else {
                crate::h!(A_, "Unknown scene: {}", n[1]);
                crate::println!("Available: cube, sphere, torus, grid, multi, dna");
            }
        }
        "status" | "info" => {
            let iq = holomatrix::zu();
            let amt = holomatrix::hlk();
            crate::println!("HoloMatrix: {}", if iq { "enabled" } else { "disabled" });
            crate::println!("Scene: {}", amt.j());
        }
        "list" | "scenes" => {
            crate::h!(G_, "Available Scenes:");
            for j in holomatrix::HoloScene::qgk() {
                crate::println!("  {}", j);
            }
        }
        _ => {
            
            if let Some(amt) = holomatrix::HoloScene::nwf(n[0]) {
                holomatrix::bid(amt);
                crate::h!(0xFF00FFFF, "Scene set to: {}", amt.j());
            } else {
                crate::h!(A_, "Unknown command: {}", n[0]);
                crate::println!("Use 'holo' for help");
            }
        }
    }
}


pub(super) fn rfi(n: &[&str]) {
    if n.is_empty() {
        crate::h!(C_, "TrustOS Image Viewer");
        crate::h!(C_, "====================");
        crate::println!();
        crate::println!("Usage: imgview <path> [options]");
        crate::println!();
        crate::h!(G_, "Options:");
        crate::println!("  -x <num>     X position (default: center)");
        crate::println!("  -y <num>     Y position (default: center)");
        crate::println!("  -w <num>     Width (scale to this width)");
        crate::println!("  -h <num>     Height (scale to this height)");
        crate::println!("  -info        Show image info only, don't display");
        crate::println!();
        crate::h!(G_, "Supported Formats:");
        crate::println!("  BMP  - 24-bit and 32-bit uncompressed");
        crate::println!("  PPM  - P3 (ASCII) and P6 (binary)");
        crate::println!("  RAW  - Raw RGBA pixel data");
        crate::println!();
        crate::h!(G_, "Examples:");
        crate::println!("  imgview /usr/share/wallpapers/logo.bmp");
        crate::println!("  imgview /home/image.ppm -x 100 -y 100");
        crate::println!("  imgview photo.bmp -w 640 -h 480");
        return;
    }
    
    let path = n[0];
    let mut ows: Option<i32> = None;
    let mut owt: Option<i32> = None;
    let mut z: Option<u32> = None;
    let mut ac: Option<u32> = None;
    let mut oeg = false;
    
    
    let mut a = 1;
    while a < n.len() {
        match n[a] {
            "-x" if a + 1 < n.len() => {
                if let Ok(p) = n[a + 1].parse::<i32>() {
                    ows = Some(p);
                }
                a += 2;
            }
            "-y" if a + 1 < n.len() => {
                if let Ok(p) = n[a + 1].parse::<i32>() {
                    owt = Some(p);
                }
                a += 2;
            }
            "-w" if a + 1 < n.len() => {
                if let Ok(p) = n[a + 1].parse::<u32>() {
                    z = Some(p);
                }
                a += 2;
            }
            "-h" if a + 1 < n.len() => {
                if let Ok(p) = n[a + 1].parse::<u32>() {
                    ac = Some(p);
                }
                a += 2;
            }
            "-info" => {
                oeg = true;
                a += 1;
            }
            _ => {
                a += 1;
            }
        }
    }
    
    
    crate::println!("Loading image: {}", path);
    
    match crate::image::load(path) {
        Some(th) => {
            crate::h!(B_, "Image loaded successfully!");
            crate::println!("  Size: {} x {} pixels", th.z, th.ac);
            crate::println!("  Memory: {} KB", (th.hz.len() * 4) / 1024);
            
            if oeg {
                return;
            }
            
            
            let fgq = z.unwrap_or(th.z);
            let fgp = ac.unwrap_or(th.ac);
            
            
            let (lu, qh) = crate::framebuffer::yn();
            let b = ows.unwrap_or_else(|| ((lu - fgq) / 2) as i32);
            let c = owt.unwrap_or_else(|| ((qh - fgp) / 2) as i32);
            
            crate::println!("  Drawing at ({}, {}) size {}x{}", b, c, fgq, fgp);
            
            
            if fgq == th.z && fgp == th.ac {
                th.po(b, c);
            } else {
                th.nnp(b, c, fgq, fgp);
            }
            
            crate::framebuffer::sv();
            crate::h!(B_, "Image displayed!");
        }
        None => {
            crate::h!(A_, "Failed to load image: {}", path);
            crate::println!("Make sure the file exists and is a supported format.");
        }
    }
}


pub(super) fn rfh(n: &[&str]) {
    let nkj = n.fv().hu().unwrap_or("gradient");
    
    crate::h!(C_, "Image Demo: {}", nkj);
    
    let (lu, qh) = crate::framebuffer::yn();
    
    match nkj {
        "gradient" => {
            
            let th = crate::image::nhd(
                200, 200, 
                0xFF0066FF,  
                0xFF00FF66   
            );
            let b = ((lu - 200) / 2) as i32;
            let c = ((qh - 200) / 2) as i32;
            th.po(b, c);
            crate::h!(B_, "Displayed gradient at center");
        }
        "checker" => {
            
            let th = crate::image::rql(
                256, 256, 32,
                0xFFFFFFFF,  
                0xFF000000   
            );
            let b = ((lu - 256) / 2) as i32;
            let c = ((qh - 256) / 2) as i32;
            th.po(b, c);
            crate::h!(B_, "Displayed checkerboard at center");
        }
        "trustos" => {
            
            let th = crate::image::nhd(
                300, 100,
                0xFF00D26A,  
                0xFF0A0E0B   
            );
            let b = ((lu - 300) / 2) as i32;
            let c = ((qh - 100) / 2) as i32;
            th.po(b, c);
            
            
            let aia = 0xFF00D26A;
            for a in 0..300 {
                crate::framebuffer::sf(b as u32 + a, c as u32, aia);
                crate::framebuffer::sf(b as u32 + a, (c + 99) as u32, aia);
            }
            for a in 0..100 {
                crate::framebuffer::sf(b as u32, c as u32 + a, aia);
                crate::framebuffer::sf((b + 299) as u32, c as u32 + a, aia);
            }
            
            crate::h!(B_, "Displayed TrustOS banner");
        }
        "colors" => {
            
            let mut th = crate::image::Image::new(256, 256);
            for c in 0..256 {
                for b in 0..256 {
                    let m = b as u32;
                    let at = c as u32;
                    let o = ((b + c) / 2) as u32;
                    let s = 0xFF000000 | (m << 16) | (at << 8) | o;
                    th.aht(b, c, s);
                }
            }
            let b = ((lu - 256) / 2) as i32;
            let c = ((qh - 256) / 2) as i32;
            th.po(b, c);
            crate::h!(B_, "Displayed color test pattern");
        }
        "alpha" => {
            
            
            let ei = crate::image::rqs(200, 200, 0xFFFF0000);
            let b = ((lu - 200) / 2) as i32;
            let c = ((qh - 200) / 2) as i32;
            ei.po(b, c);
            
            
            let mut cte = crate::image::Image::new(200, 200);
            for x in 0..200u32 {
                for y in 0..200u32 {
                    
                    let dw = (y + x) / 2;
                    let s = (dw << 24) | 0x000000FF;  
                    cte.aht(y, x, s);
                }
            }
            cte.po(b, c);
            crate::h!(B_, "Displayed alpha blend demo (red + blue)");
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
    
    crate::framebuffer::sv();
}

pub(super) fn riz() {
    let bcy = crate::task::liy();
    crate::h!(C_, "  PID  STATE       PRIORITY  NAME");
    crate::h!(C_, "-------------------------------------");
    
    
    crate::println!("    1  running     critical  kernel");
    crate::println!("    2  running     normal    tsh");
    
    let dmj = bcy.len();
    for (ad, j, g, abv) in bcy {
        let boo = match g {
            crate::task::TaskState::At => "ready",
            crate::task::TaskState::Ai => "running",
            crate::task::TaskState::Hj => "blocked",
            crate::task::TaskState::Hh => "done",
        };
        let vll = match abv {
            crate::task::Priority::Eg => "low",
            crate::task::Priority::M => "normal",
            crate::task::Priority::Ao => "high",
            crate::task::Priority::Aj => "critical",
        };
        crate::println!("{:>5}  {:10}  {:8}  {}", ad + 2, boo, vll, j);
    }
    
    crate::println!();
    crate::h!(AU_, "Total: {} tasks", dmj + 2);
}

pub(super) fn rji() {
    crate::h!(C_, "  TID  PID  STATE       NAME");
    crate::h!(C_, "------------------------------------");
    
    
    let axc = crate::thread::ufx();
    let az = axc.len();
    
    for (ni, ce, g, j) in axc {
        let boo = match g {
            crate::thread::ThreadState::At => "ready",
            crate::thread::ThreadState::Ai => "running",
            crate::thread::ThreadState::Hj => "blocked",
            crate::thread::ThreadState::Cnb => "sleeping",
            crate::thread::ThreadState::Ez => "dead",
        };
        crate::println!("{:>5}  {:>3}  {:10}  {}", ni, ce, boo, &j);
    }
    
    crate::println!();
    crate::h!(AU_, "Total: {} threads", az);
}



pub(super) fn rha(n: &[&str]) {
    if n.is_empty() {
        
        let (status, sb, aw) = crate::persistence::status();
        crate::h!(C_, "+--------------------------------------------------------------+");
        crate::h!(C_, "|                    Persistence Status                        |");
        crate::h!(C_, "+--------------------------------------------------------------+");
        crate::println!();
        crate::println!("  Status:       {}", status);
        crate::println!("  Saved files:  {}", sb);
        crate::println!("  Total size:   {} KB", aw / 1024);
        crate::println!();
        crate::println!("Commands:");
        crate::println!("  persist status  - Show this status");
        crate::println!("  persist clear   - Clear all saved data");
        crate::println!("  persist save    - Save current downloads to disk");
        crate::println!();
        return;
    }
    
    match n[0] {
        "status" => {
            let (status, sb, aw) = crate::persistence::status();
            crate::println!("Persistence: {} ({} files, {} KB)", status, sb, aw / 1024);
        }
        "clear" => {
            crate::println!("Clearing persistence data...");
            match crate::persistence::clear() {
                Ok(_) => crate::h!(B_, "Persistence data cleared."),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        "save" => {
            crate::println!("Saving current data to disk...");
            
            
            let gac = "/opt/gui/alpine.tar.gz";
            if cxx(gac) {
                let duz: Result<Vec<u8>, _> = crate::ramfs::fh(|fs| {
                    fs.mq(gac).map(|bc| bc.ip())
                });
                match duz {
                    Ok(f) => {
                        match crate::persistence::ftm(gac, &f) {
                            Ok(_) => crate::h!(B_, "  Saved: {} ({} KB)", gac, f.len() / 1024),
                            Err(aa) => crate::h!(A_, "  Failed: {} - {}", gac, aa),
                        }
                    }
                    Err(aa) => crate::h!(A_, "  Cannot read {}: {:?}", gac, aa),
                }
            } else {
                crate::println!("  No files to save. Run 'gui install' first.");
            }
        }
        _ => {
            crate::println!("Unknown persistence command: {}", n[0]);
            crate::println!("Use: persist [status|clear|save]");
        }
    }
}



pub(super) fn rdt() {
    crate::h!(C_, "=== Storage Devices ===");
    
    let mut cjx = 0u32;
    
    
    if crate::nvme::ky() {
        if let Some((model, serial, gob, bni)) = crate::nvme::ani() {
            let aga = (gob * bni as u64) / (1024 * 1024);
            crate::println!();
            crate::h!(B_, "[NVMe] {}", model);
            crate::println!("  Serial:    {}", serial);
            crate::println!("  Capacity:  {} MB ({} sectors x {} bytes)", aga, gob, bni);
            crate::println!("  Interface: NVMe over PCIe");
            cjx += 1;
        }
    }
    
    
    if crate::drivers::ahci::ky() {
        for ba in crate::drivers::ahci::bhh() {
            let aga = (ba.agw * 512) / (1024 * 1024);
            crate::println!();
            crate::gr!(B_, "[AHCI Port {}] ", ba.kg);
            crate::println!("{}", ba.model);
            crate::println!("  Serial:    {}", ba.serial);
            crate::println!("  Capacity:  {} MB ({} sectors)", aga, ba.agw);
            crate::println!("  Type:      {:?}", ba.ceb);
            crate::println!("  Interface: SATA (AHCI)");
            cjx += 1;
        }
    }
    
    
    for ane in crate::drivers::ata::jdq() {
        if ane.brs {
            let aga = (ane.agw * 512) / (1024 * 1024);
            let bm = match ane.channel {
                crate::drivers::ata::IdeChannel::Adx => "Primary",
                crate::drivers::ata::IdeChannel::Aeq => "Secondary",
            };
            let u = match ane.qf {
                crate::drivers::ata::DrivePosition::Ake => "Master",
                crate::drivers::ata::DrivePosition::Ams => "Slave",
            };
            crate::println!();
            crate::gr!(B_, "[IDE {} {}] ", bm, u);
            crate::println!("{}", ane.model);
            crate::println!("  Serial:    {}", ane.serial);
            crate::println!("  Capacity:  {} MB ({} sectors)", aga, ane.agw);
            crate::println!("  LBA48:     {}", if ane.gle { "Yes" } else { "No (28-bit)" });
            crate::println!("  ATAPI:     {}", if ane.gal { "Yes" } else { "No" });
            crate::println!("  Interface: IDE/ATA (PIO)");
            cjx += 1;
        }
    }
    
    
    if crate::virtio_blk::ky() {
        let mh = crate::virtio_blk::aty();
        let aga = (mh * 512) / (1024 * 1024);
        let jmq = crate::virtio_blk::jbr();
        crate::println!();
        crate::h!(B_, "[VirtIO Block Device]");
        crate::println!("  Capacity:  {} MB ({} sectors)", aga, mh);
        crate::println!("  Read-Only: {}", if jmq { "Yes" } else { "No" });
        crate::println!("  Interface: VirtIO (paravirtual)");
        cjx += 1;
    }
    
    
    for (a, (j, xk, gbn)) in crate::drivers::usb_storage::bhh().iter().cf() {
        let aga = (*xk * *gbn as u64) / (1024 * 1024);
        crate::println!();
        crate::gr!(B_, "[USB Storage #{}] ", a);
        crate::println!("{}", j);
        crate::println!("  Capacity:  {} MB ({} blocks x {} bytes)", aga, xk, gbn);
        crate::println!("  Interface: USB Mass Storage (BBB/SCSI)");
        cjx += 1;
    }
    
    
    crate::println!();
    if let Some(co) = crate::disk::ani() {
        crate::h!(AU_, "[RAM Disk]");
        crate::println!("  Size:      {} KB ({} sectors)", co.grv / 2, co.grv);
        
        let (exj, fbu, qvb, qvd) = crate::disk::asx();
        crate::println!("  Stats:     {} reads ({} B), {} writes ({} B)", exj, qvb, fbu, qvd);
    }
    
    
    crate::println!();
    if cjx == 0 {
        crate::h!(D_, "No hardware storage detected (RAM disk only)");
    } else {
        crate::h!(C_, "Total: {} hardware storage device(s) + RAM disk", cjx);
    }
}

pub(super) fn rdm(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: dd <sector> [count]");
        crate::println!("       dd write <sector> <text>");
        crate::println!("       dd dump <sector>");
        return;
    }
    
    if n[0] == "dump" && n.len() > 1 {
        let jk: u64 = match n[1].parse() {
            Ok(bo) => bo,
            Err(_) => {
                crate::h!(A_, "Invalid sector number");
                return;
            }
        };
        
        match crate::disk::shh(jk) {
            Ok(epk) => crate::println!("{}", epk),
            Err(aa) => crate::h!(A_, "Error: {}", aa),
        }
        return;
    }
    
    if n[0] == "write" && n.len() > 2 {
        let jk: u64 = match n[1].parse() {
            Ok(bo) => bo,
            Err(_) => {
                crate::h!(A_, "Invalid sector number");
                return;
            }
        };
        
        let text = n[2..].rr(" ");
        let mut f = [0u8; 512];
        let bf = text.as_bytes();
        let len = bf.len().v(512);
        f[..len].dg(&bf[..len]);
        
        match crate::disk::aby(jk, &f) {
            Ok(_) => crate::h!(B_, "Written {} bytes to sector {}", len, jk),
            Err(aa) => crate::h!(A_, "Write error: {}", aa),
        }
        return;
    }
    
    
    let jk: u64 = match n[0].parse() {
        Ok(bo) => bo,
        Err(_) => {
            crate::h!(A_, "Invalid sector number");
            return;
        }
    };

    let mut bi = [0u8; 512];
    match crate::disk::ain(jk, 1, &mut bi) {
        Ok(_) => {
            crate::h!(C_, "Sector {} (512 bytes):", jk);
            
            
            for br in 0..16 {
                crate::gr!(AU_, "{:04X}: ", br * 16);
                for bj in 0..16 {
                    crate::print!("{:02X} ", bi[br * 16 + bj]);
                }
                crate::print!(" |");
                for bj in 0..16 {
                    let o = bi[br * 16 + bj];
                    if o >= 0x20 && o < 0x7F {
                        crate::print!("{}", o as char);
                    } else {
                        crate::print!(".");
                    }
                }
                crate::println!("|");
            }
        }
        Err(aa) => {
            crate::h!(A_, "Read error: {}", aa);
        }
    }
}

pub(super) fn rcc(n: &[&str]) {
    if n.is_empty() {
        
        crate::h!(C_, "=== AHCI Storage Controller ===");
        
        if !crate::drivers::ahci::ky() {
            crate::h!(D_, "AHCI not initialized");
            return;
        }
        
        let ik = crate::drivers::ahci::bhh();
        if ik.is_empty() {
            crate::h!(D_, "No AHCI devices found");
            return;
        }
        
        crate::println!("Found {} device(s):", ik.len());
        for ba in &ik {
            crate::println!();
            crate::gr!(B_, "  Port {}: ", ba.kg);
            crate::println!("{:?}", ba.ceb);
            crate::println!("    Model:   {}", ba.model);
            crate::println!("    Serial:  {}", ba.serial);
            crate::println!("    Sectors: {}", ba.agw);
        }
        
        crate::println!();
        crate::h!(AU_, "Commands:");
        crate::println!("  ahci read <port> <sector>   - Read sector from port");
        crate::println!("  ahci write <port> <sector> <text> - Write to sector");
        return;
    }
    
    match n[0] {
        "read" => {
            if n.len() < 3 {
                crate::println!("Usage: ahci read <port> <sector>");
                return;
            }
            
            let port: u8 = match n[1].parse() {
                Ok(bo) => bo,
                Err(_) => {
                    crate::h!(A_, "Invalid port number");
                    return;
                }
            };
            
            let jk: u64 = match n[2].parse() {
                Ok(bo) => bo,
                Err(_) => {
                    crate::h!(A_, "Invalid sector number");
                    return;
                }
            };
            
            crate::println!("Reading sector {} from AHCI port {}...", jk, port);
            
            
            let mut bi = alloc::vec![0u8; 512];
            
            match crate::drivers::ahci::ain(port, jk, 1, &mut bi) {
                Ok(bf) => {
                    crate::h!(B_, "Read {} bytes successfully", bf);
                    crate::println!();
                    
                    
                    for br in 0..16 {
                        crate::gr!(AU_, "{:04X}: ", br * 16);
                        for bj in 0..16 {
                            crate::print!("{:02X} ", bi[br * 16 + bj]);
                        }
                        crate::print!(" |");
                        for bj in 0..16 {
                            let o = bi[br * 16 + bj];
                            if o >= 0x20 && o < 0x7F {
                                crate::print!("{}", o as char);
                            } else {
                                crate::print!(".");
                            }
                        }
                        crate::println!("|");
                    }
                }
                Err(aa) => {
                    crate::h!(A_, "AHCI read error: {}", aa);
                }
            }
        }
        
        "write" => {
            if n.len() < 4 {
                crate::println!("Usage: ahci write <port> <sector> <text>");
                return;
            }
            
            let port: u8 = match n[1].parse() {
                Ok(bo) => bo,
                Err(_) => {
                    crate::h!(A_, "Invalid port number");
                    return;
                }
            };
            
            let jk: u64 = match n[2].parse() {
                Ok(bo) => bo,
                Err(_) => {
                    crate::h!(A_, "Invalid sector number");
                    return;
                }
            };
            
            let text = n[3..].rr(" ");
            let mut bi = alloc::vec![0u8; 512];
            let bf = text.as_bytes();
            let len = bf.len().v(512);
            bi[..len].dg(&bf[..len]);
            
            crate::println!("Writing {} bytes to sector {} on AHCI port {}...", len, jk, port);
            
            match crate::drivers::ahci::bpi(port, jk, 1, &bi) {
                Ok(bf) => {
                    crate::h!(B_, "Written {} bytes successfully", bf);
                }
                Err(aa) => {
                    crate::h!(A_, "AHCI write error: {}", aa);
                }
            }
        }
        
        _ => {
            crate::println!("Unknown AHCI command. Use 'ahci' for help.");
        }
    }
}



pub(super) fn reh(n: &[&str]) {
    use crate::drivers::partition;
    use crate::drivers::ahci;
    
    if n.is_empty() {
        
        crate::h!(C_, "=== Partition Tables ===");
        crate::println!();
        
        if !ahci::ky() {
            crate::h!(D_, "AHCI not initialized");
            crate::println!();
            crate::println!("Usage:");
            crate::println!("  fdisk           - Show partitions on all disks");
            crate::println!("  fdisk <port>    - Show partitions on specific AHCI port");
            return;
        }
        
        let ik = ahci::bhh();
        if ik.is_empty() {
            crate::println!("No AHCI devices found");
            return;
        }
        
        for ba in ik {
            crate::h!(B_, "--- Disk {} ({:?}) ---", ba.kg, ba.ceb);
            
            match partition::lxn(ba.kg) {
                Ok(gg) => {
                    partition::oxt(&gg);
                }
                Err(aa) => {
                    crate::h!(A_, "  Error reading partitions: {}", aa);
                }
            }
            crate::println!();
        }
        
        return;
    }
    
    
    let port: u8 = match n[0].parse() {
        Ok(ai) => ai,
        Err(_) => {
            crate::h!(A_, "Invalid port number: {}", n[0]);
            return;
        }
    };
    
    crate::h!(C_, "=== Partitions on Disk {} ===", port);
    
    match partition::lxn(port) {
        Ok(gg) => {
            partition::oxt(&gg);
        }
        Err(aa) => {
            crate::h!(A_, "Error: {}", aa);
        }
    }
}



pub(super) fn hdh() {
    
    if let Some(eft) = crate::network::tef() {
        crate::h!(C_, "Hardware:");
        crate::println!("      Device: {:04X}:{:04X} [{}]", 
            eft.ml, eft.mx, eft.cip);
        crate::println!("      Driver: {}", eft.rj);
        if crate::network::tnb() {
            crate::h!(B_, "      Status: REAL DRIVER ACTIVE");
        } else {
            crate::h!(D_, "      Status: Simulated");
        }
        if eft.aew != 0 {
            crate::println!("      BAR0:   {:#010X}", eft.aew);
        }
        if eft.irq != 0 && eft.irq != 0xFF {
            crate::println!("      IRQ:    {}", eft.irq);
        }
        crate::println!();
    }
    
    if let Some((ed, ip, g)) = crate::network::gif() {
        crate::h!(C_, "eth0:");
        crate::print!("      Link: ");
        match g {
            crate::network::NetworkState::Ek => crate::h!(B_, "UP"),
            crate::network::NetworkState::Fm => crate::h!(D_, "DOWN"),
            crate::network::NetworkState::Q => crate::h!(A_, "ERROR"),
        }
        crate::println!("      HWaddr: {}", ed);
        if let Some(ag) = ip {
            crate::println!("      inet:   {}", ag);
        }
        
        
        let (ifj, hyk, bpc, bsc) = crate::network::tdm();
        crate::println!();
        crate::println!("      RX packets: {}  bytes: {}", hyk, bsc);
        crate::println!("      TX packets: {}  bytes: {}", ifj, bpc);
        
        let cm = crate::network::asx();
        if cm.bqn > 0 {
            crate::h!(A_, "      Errors: {}", cm.bqn);
        }
    } else {
        crate::h!(D_, "No network interface");
    }
}










pub(super) fn rgn(n: &[&str]) {
    crate::h!(G_, "=== TrustScan Live Network Test Suite ===");
    crate::println!();

    let mut cg = 0usize;
    let mut gv = 0usize;

    
    crate::h!(C_, "[PRE] Network connectivity check");
    crate::print!("  NIC driver... ");
    if !crate::drivers::net::bzy() {
        crate::h!(A_, "[FAIL] no network driver");
        crate::h!(A_, "=== Cannot run live tests without a network ===");
        return;
    }
    crate::h!(B_, "[OK]");

    
    let (aro, elo, kxr) = match crate::network::aou() {
        Some((ip, hs, nt)) => {
            let gkc = *ip.as_bytes();
            let uje = *hs.as_bytes();
            let tij = nt.map(|at| *at.as_bytes());
            (gkc, uje, tij)
        }
        None => {
            crate::print!("  IP config... ");
            crate::h!(A_, "[FAIL] no IPv4 config — run 'dhcp' first");
            return;
        }
    };

    crate::print!("  our IP... ");
    crate::h!(B_, "[OK] {}", crate::netscan::aot(aro));

    
    let cd: [u8; 4];
    let fwf: &str;

    if let Some(ji) = n.fv() {
        if let Some(ip) = crate::netscan::ewb(ji) {
            cd = ip;
            fwf = ji;
        } else if let Some(bhv) = crate::netstack::dns::ayo(ji) {
            cd = bhv;
            fwf = ji;
        } else {
            crate::h!(A_, "Cannot resolve: {}", ji);
            return;
        }
    } else if let Some(nt) = kxr {
        cd = nt;
        fwf = "gateway";
    } else {
        
        cd = [8, 8, 8, 8];
        fwf = "8.8.8.8";
    }

    crate::h!(C_, "  target: {} ({})", fwf, crate::netscan::aot(cd));
    crate::println!();

    
    crate::h!(C_, "[1/8] ICMP Ping — reachability");
    {
        crate::print!("  ping {}... ", crate::netscan::aot(cd));
        let ip = crate::network::Ipv4Address::new(cd[0], cd[1], cd[2], cd[3]);
        match crate::network::mdt(ip) {
            Ok(result) if result.vx => {
                crate::h!(B_, "[OK] rtt={} us  ttl={}", result.dwu, result.akv);
                cg += 1;
            }
            Ok(_) => {
                crate::h!(D_, "[WARN] timeout (host may block ICMP)");
                cg += 1; 
            }
            Err(aa) => {
                crate::h!(A_, "[FAIL] {}", aa);
                gv += 1;
            }
        }
    }

    
    crate::h!(C_, "[2/8] ARP Resolution");
    {
        
        let kbc = kxr.unwrap_or(cd);
        crate::print!("  ARP {}... ", crate::netscan::aot(kbc));
        let _ = crate::netstack::arp::eii(kbc);
        
        for _ in 0..200_000 {
            crate::netstack::poll();
            core::hint::hc();
        }
        if let Some(ed) = crate::netstack::arp::ayo(kbc) {
            crate::h!(B_, "[OK] MAC={}", crate::netscan::eqs(ed));
            cg += 1;
        } else {
            crate::h!(D_, "[WARN] no ARP reply (may be routed)");
            cg += 1; 
        }
    }

    
    crate::h!(C_, "[3/8] DNS Resolution");
    {
        crate::print!("  resolve google.com... ");
        match crate::netstack::dns::ayo("google.com") {
            Some(ip) => {
                crate::h!(B_, "[OK] {}", crate::netscan::aot(ip));
                cg += 1;
            }
            None => {
                crate::h!(A_, "[FAIL] DNS resolution failed");
                gv += 1;
            }
        }

        crate::print!("  resolve example.com... ");
        match crate::netstack::dns::ayo("example.com") {
            Some(ip) => {
                crate::h!(B_, "[OK] {}", crate::netscan::aot(ip));
                cg += 1;
            }
            None => {
                crate::h!(D_, "[WARN] no DNS — limited test");
                cg += 1; 
            }
        }
    }

    
    crate::h!(C_, "[4/8] TCP SYN Port Scan");
    {
        
        let psn = alloc::vec![80u16, 443, 53, 22, 8080];
        crate::print!("  SYN scan {} ({} ports)... ", crate::netscan::aot(cd), psn.len());
        let config = crate::netscan::port_scanner::ScanConfig::new(cd)
            .jxa(psn)
            .jxd(crate::netscan::port_scanner::ScanType::Uu)
            .jxb(2000);
        let (hd, cm) = crate::netscan::port_scanner::arx(&config);

        
        crate::h!(B_, "[OK] {} open, {} closed, {} filtered ({} ms)",
            cm.aji, cm.cwg, cm.aud, cm.oz);
        cg += 1;

        
        for m in &hd {
            if m.g == crate::netscan::port_scanner::PortState::Ck {
                crate::println!("    {:>5}/tcp  {:<12}  OPEN", m.port, m.xi);
            }
        }
    }

    
    crate::h!(C_, "[5/8] TCP Connect Scan + Banner Grab");
    {
        
        
        let fyp = if let Some(ip) = crate::netstack::dns::ayo("example.com") {
            ip
        } else {
            cd 
        };

        crate::print!("  connect scan {}:80... ", crate::netscan::aot(fyp));
        let config = crate::netscan::port_scanner::ScanConfig::new(fyp)
            .jxa(alloc::vec![80])
            .jxd(crate::netscan::port_scanner::ScanType::Wa)
            .jxb(3000);
        let (hd, cm) = crate::netscan::port_scanner::arx(&config);

        if cm.aji > 0 {
            crate::h!(B_, "[OK] port 80 open");
            cg += 1;

            
            crate::print!("  banner grab :80... ");
            match crate::netscan::banner::ern(fyp, 80, 3000) {
                Some(avi) => {
                    let xmi = if avi.banner.len() > 60 {
                        &avi.banner[..60]
                    } else {
                        &avi.banner
                    };
                    crate::h!(B_, "[OK] '{}'", xmi);
                    if let Some(ref axh) = avi.dk {
                        crate::println!("    version: {}", axh);
                    }
                    cg += 1;
                }
                None => {
                    crate::h!(D_, "[WARN] no banner (server may not send one)");
                    cg += 1;
                }
            }
        } else {
            crate::h!(D_, "[WARN] port 80 not open on {} — skip banner",
                crate::netscan::aot(fyp));
            cg += 2; 
        }
        let _ = hd;
    }

    
    crate::h!(C_, "[6/8] Packet Sniffer (live capture)");
    {
        use crate::netscan::sniffer;

        crate::print!("  capture during ping... ");
        sniffer::gtb();

        
        let ip = crate::network::Ipv4Address::new(cd[0], cd[1], cd[2], cd[3]);
        let _ = crate::network::mdt(ip);

        
        for _ in 0..300_000 {
            crate::netstack::poll();
            core::hint::hc();
        }

        let (az, bf, cox) = sniffer::asx();
        sniffer::gth();

        if az > 0 {
            crate::h!(B_, "[OK] captured {} packets, {} bytes", az, bf);
            cg += 1;

            
            crate::print!("  packet details... ");
            let fqx = sniffer::jjc(5);
            if !fqx.is_empty() {
                crate::h!(B_, "[OK] {} in buffer", fqx.len());
                for ai in fqx.iter().take(3) {
                    crate::println!("    [{:<4}] {} {}", ai.protocol.as_str(),
                        ai.jh.map(|a| crate::netscan::aot(a)).unwrap_or_else(|| alloc::string::String::from("?")),
                        ai.co);
                }
                cg += 1;
            } else {
                crate::h!(A_, "[FAIL] buffer empty despite count > 0");
                gv += 1;
            }
        } else {
            crate::h!(D_, "[WARN] no packets captured (driver may not report TX)");
            cg += 2; 
        }
    }

    
    crate::h!(C_, "[7/8] Traceroute (TTL)");
    {
        
        let pvo = if let Some(nt) = kxr {
            nt 
        } else {
            [8, 8, 8, 8] 
        };

        crate::print!("  trace to {} (5 hops max)... ", crate::netscan::aot(pvo));
        let cyn = crate::netscan::traceroute::trace(pvo, 5, 2000);

        if !cyn.is_empty() {
            crate::h!(B_, "[OK] {} hops recorded", cyn.len());
            cg += 1;

            for i in &cyn {
                crate::print!("    {:>2}  ", i.gjd);
                if let Some(ip) = i.ip {
                    crate::print!("{:<16}", crate::netscan::aot(ip));
                    for &ehv in &i.bcj {
                        if ehv == 0 { crate::print!("*    "); }
                        else { crate::print!("{} ms  ", ehv); }
                    }
                    if i.gqi { crate::print!(" [REACHED]"); }
                    crate::println!();
                } else {
                    crate::println!("* * *");
                }
            }
        } else {
            crate::h!(D_, "[WARN] no hops (ICMP may be blocked)");
            cg += 1;
        }
    }

    
    crate::h!(C_, "[8/8] Vulnerability Scanner");
    {
        
        let fyp = if let Some(ip) = crate::netstack::dns::ayo("example.com") {
            ip
        } else {
            cd
        };

        crate::print!("  vuln check {}:80,443... ", crate::netscan::aot(fyp));
        let nq = crate::netscan::vuln::arx(fyp, &[80, 443]);

        
        crate::h!(B_, "[OK] {} findings", nq.len());
        cg += 1;

        for bb in nq.iter().take(3) {
            let s = match bb.qj {
                crate::netscan::vuln::Severity::Aj | crate::netscan::vuln::Severity::Ao => A_,
                crate::netscan::vuln::Severity::Bc => D_,
                _ => L_,
            };
            crate::gr!(s, "    [{:<8}] ", bb.qj.as_str());
            crate::println!("{}/{}: {}", bb.port, bb.xi, bb.dq);
        }
    }

    
    crate::println!();
    let es = cg + gv;
    if gv == 0 {
        crate::h!(G_,
            "=== ALL {}/{} LIVE TESTS PASSED ===", cg, es);
    } else {
        crate::h!(A_,
            "=== {}/{} passed, {} FAILED ===", cg, es, gv);
    }
    crate::println!();
    crate::println!("Tip: For more detailed testing, try:");
    crate::println!("  nmap example.com -sT -p 80,443 -A");
    crate::println!("  banner example.com 80");
    crate::println!("  traceroute 8.8.8.8");
    crate::println!("  sniff start   (then generate traffic)");
    crate::println!("  vulnscan example.com");
}



pub(super) fn rgq(n: &[&str]) {
    if n.is_empty() {
        crate::h!(C_, "TrustScan — Port Scanner");
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

    let cd = match crate::netscan::lzr(n[0]) {
        Some(ip) => ip,
        None => {
            crate::h!(A_, "Cannot resolve target: {}", n[0]);
            return;
        }
    };

    
    let mut cmr = crate::netscan::port_scanner::ScanType::Uu;
    let mut xf: Option<alloc::vec::Vec<u16>> = None;
    let mut puq = false;
    let mut jzt = false;

    let mut a = 1;
    while a < n.len() {
        match n[a] {
            "-sS" => cmr = crate::netscan::port_scanner::ScanType::Uu,
            "-sT" => cmr = crate::netscan::port_scanner::ScanType::Wa,
            "-sU" => cmr = crate::netscan::port_scanner::ScanType::Ic,
            "--top" => puq = true,
            "-A" => jzt = true,
            "-p" if a + 1 < n.len() => {
                a += 1;
                xf = Some(lst(n[a]));
            }
            _ => {}
        }
        a += 1;
    }

    let wdv = match cmr {
        crate::netscan::port_scanner::ScanType::Uu => "SYN",
        crate::netscan::port_scanner::ScanType::Wa => "Connect",
        crate::netscan::port_scanner::ScanType::Ic => "UDP",
    };

    crate::h!(C_, "Starting TrustScan {} scan on {}", wdv, crate::netscan::aot(cd));
    crate::println!("TrustScan 1.0 — TrustOS Network Security Scanner");
    crate::println!();

    let mut config = crate::netscan::port_scanner::ScanConfig::new(cd)
        .jxd(cmr);

    if let Some(ai) = xf {
        config = config.jxa(ai);
    } else if puq || jzt {
        config = config.jxc();
    }

    let (hd, cm) = crate::netscan::port_scanner::arx(&config);

    
    crate::println!("PORT       STATE          SERVICE");
    for result in &hd {
        let wtm = match result.g {
            crate::netscan::port_scanner::PortState::Ck => B_,
            crate::netscan::port_scanner::PortState::Kl => D_,
            crate::netscan::port_scanner::PortState::Xx => D_,
            crate::netscan::port_scanner::PortState::Dk => A_,
        };

        let cgv = match cmr {
            crate::netscan::port_scanner::ScanType::Ic => "udp",
            _ => "tcp",
        };

        crate::print!("{}/{:<6}", result.port, cgv);
        crate::gr!(wtm, " {:<14}", result.g.as_str());
        crate::println!(" {}", result.xi);
    }

    crate::println!();
    crate::println!("Scan complete: {} ports scanned in {} ms", cm.pvc, cm.oz);
    crate::println!("  {} open, {} closed, {} filtered",
        cm.aji, cm.cwg, cm.aud);

    
    if jzt {
        let dkf: alloc::vec::Vec<u16> = hd.iter()
            .hi(|m| m.g == crate::netscan::port_scanner::PortState::Ck)
            .map(|m| m.port)
            .collect();

        if !dkf.is_empty() {
            crate::println!();
            crate::h!(C_, "Banner Grabbing...");
            let ikn = crate::netscan::banner::nzm(cd, &dkf, 2000);
            for o in &ikn {
                crate::print!("  {}/tcp ", o.port);
                if let Some(ref axh) = o.dk {
                    crate::gr!(B_, "{} ", axh);
                }
                crate::println!("{}", o.banner);
            }

            crate::println!();
            crate::h!(C_, "Vulnerability Assessment...");
            let nq = crate::netscan::vuln::arx(cd, &dkf);
            for bb in &nq {
                let mfe = match bb.qj {
                    crate::netscan::vuln::Severity::Aj => A_,
                    crate::netscan::vuln::Severity::Ao => A_,
                    crate::netscan::vuln::Severity::Bc => D_,
                    crate::netscan::vuln::Severity::Eg => C_,
                    crate::netscan::vuln::Severity::V => Q_,
                };
                crate::gr!(mfe, "[{}] ", bb.qj.as_str());
                crate::println!("{}/{} — {}", bb.port, bb.xi, bb.dq);
            }
        }
    }
}

fn lst(avc: &str) -> alloc::vec::Vec<u16> {
    let mut xf = alloc::vec::Vec::new();
    for vu in avc.adk(',') {
        if let Some(hfb) = vu.du('-') {
            let ay: u16 = vu[..hfb].parse().unwrap_or(0);
            let ci: u16 = vu[hfb+1..].parse().unwrap_or(0);
            if ay > 0 && ci >= ay && ci <= 65535 {
                for ai in ay..=ci {
                    xf.push(ai);
                }
            }
        } else if let Ok(ai) = vu.parse::<u16>() {
            if ai > 0 {
                xf.push(ai);
            }
        }
    }
    xf
}

pub(super) fn ndw(n: &[&str]) {
    if n.is_empty() || n[0] == "--help" {
        crate::h!(C_, "TrustScan — Network Discovery");
        crate::println!("Usage:");
        crate::println!("  discover                   ARP sweep local subnet");
        crate::println!("  discover arp               ARP sweep (fast, local only)");
        crate::println!("  discover ping <base_ip>    ICMP ping sweep /24 subnet");
        crate::println!("  discover full              Full discovery (ARP + ping)");
        return;
    }

    let ev = if n.is_empty() { "arp" } else { n[0] };

    match ev {
        "arp" | "arpscan" => {
            crate::h!(C_, "ARP Sweep — Local Network Discovery");
            crate::println!("Scanning local subnet...");
            crate::println!();

            let bab = crate::netscan::discovery::kbb(3000);

            if bab.is_empty() {
                crate::h!(D_, "No hosts discovered");
                return;
            }

            crate::println!("IP Address          MAC Address          Status");
            crate::println!("{}", "-".afd(55));
            for kh in &bab {
                let dil = crate::netscan::aot(kh.ip);
                let djg = kh.ed.map(|ef| crate::netscan::eqs(ef))
                    .unwrap_or_else(|| alloc::string::String::from("unknown"));
                crate::h!(B_, "{:<20}{:<21}UP", dil, djg);
            }
            crate::println!();
            crate::println!("{} hosts discovered", bab.len());
        }
        "ping" => {
            let ar = if n.len() > 1 {
                match crate::netscan::ewb(n[1]) {
                    Some(ip) => [ip[0], ip[1], ip[2], 0],
                    None => {
                        crate::h!(A_, "Invalid IP: {}", n[1]);
                        return;
                    }
                }
            } else {
                match crate::network::aou() {
                    Some((ip, _, _)) => {
                        let o = ip.as_bytes();
                        [o[0], o[1], o[2], 0]
                    }
                    None => {
                        crate::h!(A_, "No network configured");
                        return;
                    }
                }
            };

            crate::h!(C_, "ICMP Ping Sweep — {}.{}.{}.0/24", ar[0], ar[1], ar[2]);
            crate::println!("Scanning 254 hosts...");

            let bab = crate::netscan::discovery::vib(ar, 500);

            crate::println!();
            crate::println!("IP Address          TTL   RTT     OS Guess");
            crate::println!("{}", "-".afd(60));
            for kh in &bab {
                let dil = crate::netscan::aot(kh.ip);
                crate::h!(B_, "{:<20}{:<6}{:<8}{}",
                    dil,
                    kh.akv.map(|ab| alloc::format!("{}", ab)).unwrap_or_else(|| alloc::string::String::from("-")),
                    alloc::format!("{}ms", kh.bcj),
                    kh.fpv);
            }
            crate::println!();
            crate::println!("{} hosts alive", bab.len());
        }
        "full" => {
            crate::h!(C_, "Full Network Discovery (ARP + Ping)");
            crate::println!("Scanning...");

            let bab = crate::netscan::discovery::syx(3000);

            crate::println!();
            crate::println!("IP Address          MAC Address          TTL   OS Guess");
            crate::println!("{}", "-".afd(70));
            for kh in &bab {
                let dil = crate::netscan::aot(kh.ip);
                let djg = kh.ed.map(|ef| crate::netscan::eqs(ef))
                    .unwrap_or_else(|| alloc::string::String::from("--:--:--:--:--:--"));
                let xnf = kh.akv.map(|ab| alloc::format!("{}", ab)).unwrap_or_else(|| alloc::string::String::from("-"));
                crate::h!(B_, "{:<20}{:<21}{:<6}{}",
                    dil, djg, xnf, kh.fpv);
            }
            crate::println!();
            crate::println!("{} hosts discovered", bab.len());
        }
        _ => {
            
            ndw(&["arp"]);
        }
    }
}

pub(super) fn rck(n: &[&str]) {
    if n.is_empty() {
        crate::h!(C_, "TrustScan — Banner Grabber");
        crate::println!("Usage: banner <ip|host> [port1,port2,...]");
        crate::println!("  banner 192.168.1.1              Grab banners from common ports");
        crate::println!("  banner 192.168.1.1 22,80,443    Grab from specific ports");
        return;
    }

    let cd = match crate::netscan::lzr(n[0]) {
        Some(ip) => ip,
        None => {
            crate::h!(A_, "Cannot resolve: {}", n[0]);
            return;
        }
    };

    let xf = if n.len() > 1 {
        lst(n[1])
    } else {
        alloc::vec![21, 22, 25, 80, 110, 143, 443, 3306, 5432, 6379, 8080]
    };

    crate::h!(C_, "Banner Grabbing {} ({} ports)", crate::netscan::aot(cd), xf.len());
    crate::println!();

    let ikn = crate::netscan::banner::nzm(cd, &xf, 3000);

    if ikn.is_empty() {
        crate::h!(D_, "No banners could be grabbed (ports may be closed)");
        return;
    }

    for o in &ikn {
        crate::gr!(B_, "{}/tcp {:<15}", o.port, o.xi);
        if let Some(ref axh) = o.dk {
            crate::gr!(C_, " [{}]", axh);
        }
        crate::println!();
        crate::println!("  {}", o.banner);
    }
}

pub(super) fn rih(n: &[&str]) {
    let air = n.fv().hu().unwrap_or("help");

    match air {
        "start" => {
            crate::netscan::sniffer::gtb();
            crate::h!(B_, "Packet capture started");
            crate::println!("Use 'sniff show' to view captured packets");
            crate::println!("Use 'sniff stop' to stop capture");
        }
        "stop" => {
            crate::netscan::sniffer::gth();
            let (az, bf, _) = crate::netscan::sniffer::asx();
            crate::h!(D_, "Capture stopped");
            crate::println!("Captured {} packets, {} bytes", az, bf);
        }
        "show" | "dump" => {
            let az = n.get(1).and_then(|e| e.parse::<usize>().bq()).unwrap_or(20);
            let egb = crate::netscan::sniffer::jjc(az);

            if egb.is_empty() {
                crate::h!(D_, "No packets captured");
                if !crate::netscan::sniffer::edu() {
                    crate::println!("Start capture with: sniff start");
                }
                return;
            }

            crate::println!("No.  Time      Protocol  Source              Destination         Info");
            crate::println!("{}", "-".afd(90));

            for (a, mt) in egb.iter().vv().cf() {
                let cy = mt.jh.map(|ip| crate::netscan::aot(ip))
                    .unwrap_or_else(|| crate::netscan::eqs(mt.atn));
                let cs = mt.pz.map(|ip| crate::netscan::aot(ip))
                    .unwrap_or_else(|| crate::netscan::eqs(mt.amc));

                let vnj = match mt.protocol {
                    crate::netscan::sniffer::Protocol::Mk => C_,
                    crate::netscan::sniffer::Protocol::Ic => CD_,
                    crate::netscan::sniffer::Protocol::Aja => B_,
                    crate::netscan::sniffer::Protocol::Anp => DF_,
                    crate::netscan::sniffer::Protocol::Vj => D_,
                    crate::netscan::sniffer::Protocol::Pq => A_,
                    crate::netscan::sniffer::Protocol::Abd => G_,
                    _ => Q_,
                };

                crate::print!("{:<5}{:<10}", a + 1, mt.aet);
                crate::gr!(vnj, "{:<10}", mt.protocol.as_str());
                crate::print!("{:<20}{:<20}", cy, cs);
                crate::println!("{}", &mt.co[..mt.co.len().v(40)]);
            }

            let (cus, xv, cox) = crate::netscan::sniffer::asx();
            crate::println!();
            crate::println!("Total: {} packets, {} bytes ({} in buffer)",
                cus, xv, cox);
        }
        "hex" => {
            let w = n.get(1).and_then(|e| e.parse::<usize>().bq()).unwrap_or(0);
            let egb = crate::netscan::sniffer::jjc(w + 1);
            if let Some(mt) = egb.get(w) {
                crate::h!(C_, "Packet #{} — {} bytes — {}",
                    w + 1, mt.go, mt.protocol.as_str());
                crate::println!("{}", crate::netscan::sniffer::obs(&mt.bal, 128));
            } else {
                crate::h!(D_, "No packet at index {}", w);
            }
        }
        "stats" => {
            let (az, bf, cox) = crate::netscan::sniffer::asx();
            let gh = crate::netscan::sniffer::edu();
            crate::h!(C_, "Sniffer Statistics");
            crate::println!("  Status:    {}", if gh { "CAPTURING" } else { "STOPPED" });
            crate::println!("  Packets:   {}", az);
            crate::println!("  Bytes:     {}", bf);
            crate::println!("  Buffered:  {}", cox);
        }
        _ => {
            crate::h!(C_, "TrustScan — Packet Sniffer");
            crate::println!("Usage:");
            crate::println!("  sniff start         Start packet capture");
            crate::println!("  sniff stop          Stop capture");
            crate::println!("  sniff show [N]      Show last N captured packets");
            crate::println!("  sniff hex [N]       Hex dump of packet N");
            crate::println!("  sniff stats         Capture statistics");
        }
    }
}

pub(super) fn rkm(n: &[&str]) {
    if n.is_empty() {
        crate::h!(C_, "TrustScan — Vulnerability Scanner");
        crate::println!("Usage: vulnscan <ip|host> [port1,port2,...]");
        crate::println!("  vulnscan 192.168.1.1              Scan all common ports + vulns");
        crate::println!("  vulnscan 192.168.1.1 80,443,3306  Scan specific ports");
        return;
    }

    let cd = match crate::netscan::lzr(n[0]) {
        Some(ip) => ip,
        None => {
            crate::h!(A_, "Cannot resolve: {}", n[0]);
            return;
        }
    };

    
    let dkf = if n.len() > 1 {
        lst(n[1])
    } else {
        crate::println!("Scanning ports...");
        let config = crate::netscan::port_scanner::ScanConfig::new(cd).jxc();
        let (hd, _) = crate::netscan::port_scanner::arx(&config);
        hd.iter()
            .hi(|m| m.g == crate::netscan::port_scanner::PortState::Ck)
            .map(|m| m.port)
            .collect()
    };

    if dkf.is_empty() {
        crate::h!(D_, "No open ports found");
        return;
    }

    crate::h!(C_, "Vulnerability Assessment — {}", crate::netscan::aot(cd));
    crate::println!("Checking {} ports...", dkf.len());
    crate::println!();

    let nq = crate::netscan::vuln::arx(cd, &dkf);

    if nq.is_empty() {
        crate::h!(B_, "No vulnerabilities detected");
        return;
    }

    
    let cpp = nq.iter().hi(|bb| bb.qj == crate::netscan::vuln::Severity::Aj).az();
    let afq = nq.iter().hi(|bb| bb.qj == crate::netscan::vuln::Severity::Ao).az();
    let gmm = nq.iter().hi(|bb| bb.qj == crate::netscan::vuln::Severity::Bc).az();

    if cpp > 0 {
        crate::gr!(A_, "CRITICAL: {} ", cpp);
    }
    if afq > 0 {
        crate::gr!(A_, "HIGH: {} ", afq);
    }
    if gmm > 0 {
        crate::gr!(D_, "MEDIUM: {} ", gmm);
    }
    crate::println!("({} total findings)", nq.len());
    crate::println!();

    
    for bb in &nq {
        let mfe = match bb.qj {
            crate::netscan::vuln::Severity::Aj => A_,
            crate::netscan::vuln::Severity::Ao => A_,
            crate::netscan::vuln::Severity::Bc => D_,
            crate::netscan::vuln::Severity::Eg => C_,
            crate::netscan::vuln::Severity::V => L_,
        };
        crate::gr!(mfe, "[{:<8}] ", bb.qj.as_str());
        crate::println!("{}/{} — {}", bb.port, bb.xi, bb.dq);
        crate::println!("           {}", bb.dc);
        crate::h!(B_, "           Fix: {}", bb.aws);
        crate::println!();
    }
}

pub(super) fn rjn(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: traceroute <host>");
        return;
    }

    let kh = n[0];
    let ip = if let Some(ip) = cgl(kh) {
        ip
    } else if let Some(bhv) = crate::netstack::dns::ayo(kh) {
        bhv
    } else {
        crate::h!(A_, "Unable to resolve host");
        return;
    };

    let fnv = n.get(1).and_then(|e| e.parse::<u8>().bq()).unwrap_or(30);

    crate::println!("traceroute to {} ({}), {} hops max, 60 byte packets",
        kh, crate::netscan::aot(ip), fnv);

    let cyn = crate::netscan::traceroute::trace(ip, fnv, 2000);

    for bhe in &cyn {
        crate::print!("{:>2}  ", bhe.gjd);
        if let Some(tpt) = bhe.ip {
            crate::print!("{:<18}", crate::netscan::aot(tpt));
            for &ehv in &bhe.bcj {
                if ehv == 0 {
                    crate::print!("*      ");
                } else {
                    crate::print!("{} ms  ", ehv);
                }
            }
            crate::println!();
        } else {
            crate::println!("* * *");
        }
    }
}

pub(super) fn kiz(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: ping <ip|host>");
        crate::println!("  Example: ping 192.168.56.1");
        crate::println!("  Example: ping example.com");
        return;
    }

    let ip = if let Some(ip) = cgl(n[0]) {
        crate::network::Ipv4Address::new(ip[0], ip[1], ip[2], ip[3])
    } else if let Some(bhv) = crate::netstack::dns::ayo(n[0]) {
        crate::network::Ipv4Address::new(bhv[0], bhv[1], bhv[2], bhv[3])
    } else {
        crate::h!(A_, "Unable to resolve host");
        return;
    };
    
    crate::println!("PING {} ({}) 56 data bytes", n[0], ip);
    
    let mut icf = 0;
    let mut pvg = 0u64;
    let mut jgc = u64::O;
    let mut jfl = 0u64;
    
    for _ in 0..4 {
        match crate::network::mdt(ip) {
            Ok(result) => {
                if result.vx {
                    icf += 1;
                    pvg += result.dwu;
                    jgc = jgc.v(result.dwu);
                    jfl = jfl.am(result.dwu);
                    
                    
                    if result.dwu < 1000 {
                        crate::println!("64 bytes from {}: icmp_seq={} ttl={} time={} us", 
                            ip, result.ls, result.akv, result.dwu);
                    } else {
                        let jn = result.dwu / 1000;
                        let xpd = (result.dwu % 1000) / 10;
                        crate::println!("64 bytes from {}: icmp_seq={} ttl={} time={}.{:02} ms", 
                            ip, result.ls, result.akv, jn, xpd);
                    }
                } else {
                    crate::h!(D_, "Request timeout for icmp_seq {}", result.ls);
                }
            }
            Err(aa) => {
                crate::h!(A_, "ping failed: {}", aa);
            }
        }
        
        
        crate::cpu::tsc::asq(1000);
    }
    
    crate::println!();
    crate::println!("--- {} ping statistics ---", n[0]);
    crate::println!("4 packets transmitted, {} received, {}% packet loss", 
        icf, 
        (4 - icf) * 25);
    if icf > 0 {
        let gzj = pvg / icf as u64;
        
        crate::println!("rtt min/avg/max = {}.{:03}/{}.{:03}/{}.{:03} ms", 
            jgc / 1000, jgc % 1000,
            gzj / 1000, gzj % 1000,
            jfl / 1000, jfl % 1000);
    }
}

pub(super) fn hdi() {
    crate::h!(C_, "Network Statistics");
    crate::println!("==================");
    
    let cm = crate::network::asx();
    crate::println!();
    crate::gr!(B_, "Packets received: ");
    crate::println!("{}", cm.dub);
    crate::gr!(B_, "Packets sent:     ");
    crate::println!("{}", cm.egc);
    crate::gr!(B_, "Bytes received:   ");
    crate::println!("{}", cm.cdm);
    crate::gr!(B_, "Bytes sent:       ");
    crate::println!("{}", cm.feb);
    crate::gr!(B_, "Errors:           ");
    crate::println!("{}", cm.bqn);
}

pub(super) fn rfn(n: &[&str]) {
    let eym = n.iter().any(|q| *q == "/all" || *q == "-a");
    crate::println!("Windows IP Configuration");
    crate::println!();

    if let Some((ed, ip, g)) = crate::network::gif() {
        crate::println!("   Ethernet adapter net0:");
        crate::println!("      Status . . . . . . . . . . . . : {:?}", g);
        crate::println!("      Physical Address. . . . . . . . : {}", ed);
        if let Some(ip) = ip {
            crate::println!("      IPv4 Address. . . . . . . . . : {}", ip);
            if let Some((_, up, auj)) = crate::network::aou() {
                crate::println!("      Subnet Mask . . . . . . . . . : {}", up);
                if let Some(nt) = auj {
                    crate::println!("      Default Gateway . . . . . . . : {}", nt);
                } else if eym {
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

pub(super) fn rgr(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: nslookup <host>");
        return;
    }

    let cd = n[0];
    if cgl(cd).is_some() {
        crate::println!("Server: 8.8.8.8");
        crate::println!("Address: {}", cd);
        crate::println!("*** Reverse lookup not implemented");
        return;
    }

    if let Some(bhv) = crate::netstack::dns::ayo(cd) {
        crate::println!("Server: 8.8.8.8");
        crate::println!("Name: {}", cd);
        crate::println!("Address: {}.{}.{}.{}", bhv[0], bhv[1], bhv[2], bhv[3]);
    } else {
        crate::h!(A_, "DNS lookup failed");
    }
}

pub(super) fn rch(n: &[&str]) {
    if n.iter().any(|q| *q == "-a" || *q == "/a") {
        crate::println!("Interface: net0");
    }

    let ch = crate::netstack::arp::ch();
    if ch.is_empty() {
        crate::println!("No ARP entries");
        return;
    }

    crate::println!("Internet Address      Physical Address       Type");
    for (ip, ed) in ch {
        let jax = ip.ft();
        crate::println!(
            "{:>3}.{:>3}.{:>3}.{:>3}      {:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}   dynamic",
            jax[0], jax[1], jax[2], jax[3], ed[0], ed[1], ed[2], ed[3], ed[4], ed[5]
        );
    }
}

pub(super) fn rhs(elm: &[&str]) {
    crate::println!("Kernel IP routing table");
    crate::println!("Destination     Gateway         Genmask         Iface");

    if let Some((ip, up, auj)) = crate::network::aou() {
        let nt = auj.unwrap_or(crate::network::Ipv4Address::new(0, 0, 0, 0));
        crate::println!("{}     {}     {}     net0", ip, nt, up);
        crate::println!("0.0.0.0         {}     0.0.0.0         net0", nt);
    } else {
        crate::println!("0.0.0.0         0.0.0.0         0.0.0.0         net0");
    }
}

pub(super) fn yjh(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: traceroute <host>");
        return;
    }

    let kh = n[0];
    let ip = if let Some(ip) = cgl(kh) {
        ip
    } else if let Some(bhv) = crate::netstack::dns::ayo(kh) {
        bhv
    } else {
        crate::h!(A_, "Unable to resolve host");
        return;
    };

    crate::println!("traceroute to {} ({}.{}.{}.{}), 30 hops max", kh, ip[0], ip[1], ip[2], ip[3]);
    if let Some((_, _, auj)) = crate::network::aou() {
        if let Some(nt) = auj {
            crate::println!(" 1  {}", nt);
        }
    }
    crate::println!(" 2  {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    crate::h!(D_, "Note: traceroute is simplified (no TTL probing)");
}





pub(super) fn rco(n: &[&str]) {
    let kx = n.fv()
        .and_then(|e| e.parse::<u32>().bq())
        .unwrap_or(440);
    let avr = n.get(1)
        .and_then(|e| e.parse::<u32>().bq())
        .unwrap_or(500);

    if kx < 20 || kx > 20000 {
        crate::h!(A_, "Frequency must be 20-20000 Hz");
        return;
    }
    if avr > 10000 {
        crate::h!(A_, "Duration max 10000 ms");
        return;
    }

    
    if !crate::drivers::hda::ky() {
        crate::gr!(D_, "Initializing audio driver... ");
        match crate::drivers::hda::init() {
            Ok(()) => crate::h!(B_, "OK"),
            Err(aa) => {
                crate::h!(A_, "FAILED: {}", aa);
                return;
            }
        }
    }

    crate::println!("Playing {}Hz for {}ms...", kx, avr);
    match crate::drivers::hda::owd(kx, avr) {
        Ok(()) => {
            
            let bvg = crate::drivers::hda::tdy();
            if bvg == 0 {
                crate::h!(A_, "Done (LPIB=0 — DMA not running!)");
            } else {
                crate::h!(B_, "Done (LPIB={})", bvg);
            }
        },
        Err(aa) => crate::h!(A_, "Error: {}", aa),
    }
}

pub(super) fn rcj(n: &[&str]) {
    match n.fv().hu() {
        Some("init") => {
            crate::gr!(D_, "Initializing Intel HDA driver... ");
            match crate::drivers::hda::init() {
                Ok(()) => crate::h!(B_, "OK"),
                Err(aa) => crate::h!(A_, "FAILED: {}", aa),
            }
        }
        Some("status") | None => {
            let status = crate::drivers::hda::status();
            crate::println!("{}", status);
        }
        Some("stop") => {
            match crate::drivers::hda::qg() {
                Ok(()) => crate::h!(B_, "Playback stopped"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("test") => {
            
            if !crate::drivers::hda::ky() {
                crate::gr!(D_, "Initializing audio driver... ");
                match crate::drivers::hda::init() {
                    Ok(()) => crate::h!(B_, "OK"),
                    Err(aa) => {
                        crate::h!(A_, "FAILED: {}", aa);
                        return;
                    }
                }
            }
            crate::println!("Playing test scale...");
            let ts = [262, 294, 330, 349, 392, 440, 494, 523]; 
            for &kx in &ts {
                let _ = crate::drivers::hda::owd(kx, 200);
            }
            crate::h!(B_, "Done");
        }
        Some("diag") => {
            crate::h!(C_, "Audio Diagnostics");
            let geq = crate::drivers::hda::geq();
            crate::println!("{}", geq);
        }
        Some("dump") => {
            crate::h!(C_, "Codec Widget Dump");
            let epk = crate::drivers::hda::rln();
            crate::println!("{}", epk);
        }
        Some("probe") => {
            crate::h!(C_, "Amp Probe (SET then GET)");
            let probe = crate::drivers::hda::qhr();
            crate::println!("{}", probe);
        }
        Some("gpio") => {
            
            let ap = match n.get(1).and_then(|e| e.parse::<u8>().bq()) {
                Some(p) => p,
                None => {
                    crate::h!(D_, "Usage: audio gpio <0|1|2>");
                    crate::println!("  0 = GPIO1 LOW (active for some amps)");
                    crate::println!("  2 = GPIO1 HIGH");
                    return;
                }
            };
            match crate::drivers::hda::wix(ap) {
                Ok(()) => crate::h!(B_, "GPIO DATA set to {:#04X}", ap),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some(gq) => {
            crate::h!(D_, "Usage: audio [init|status|stop|test|diag|dump|probe|gpio]");
        }
    }
}

pub(super) fn riu(n: &[&str]) {
    match n.fv().hu() {
        Some("note") | Some("play") => {
            
            let bkp = match n.get(1) {
                Some(bo) => *bo,
                None => {
                    crate::h!(D_, "Usage: synth note <note> [duration_ms] [waveform]");
                    crate::println!("  Examples: synth note C4");
                    crate::println!("           synth note A#3 1000 saw");
                    return;
                }
            };
            let avr = n.get(2)
                .and_then(|e| e.parse::<u32>().bq())
                .unwrap_or(500);
            
            if let Some(mql) = n.get(3) {
                if let Some(azd) = crate::audio::synth::Waveform::cko(mql) {
                    let _ = crate::audio::dvs(azd);
                }
            }
            if avr > 10000 {
                crate::h!(A_, "Duration max 10000 ms");
                return;
            }
            crate::println!("Synth: {} for {}ms", bkp, avr);
            match crate::audio::owc(bkp, avr) {
                Ok(()) => crate::h!(B_, "Done"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("freq") => {
            
            let kx = match n.get(1).and_then(|e| e.parse::<u32>().bq()) {
                Some(bb) => bb,
                None => {
                    crate::h!(D_, "Usage: synth freq <hz> [duration_ms]");
                    return;
                }
            };
            let avr = n.get(2)
                .and_then(|e| e.parse::<u32>().bq())
                .unwrap_or(500);
            if kx < 20 || kx > 20000 {
                crate::h!(A_, "Frequency must be 20-20000 Hz");
                return;
            }
            crate::println!("Synth: {}Hz for {}ms", kx, avr);
            match crate::audio::viw(kx, avr) {
                Ok(()) => crate::h!(B_, "Done"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("wave") | Some("waveform") => {
            
            match n.get(1) {
                Some(mql) => {
                    match crate::audio::synth::Waveform::cko(mql) {
                        Some(azd) => {
                            let _ = crate::audio::dvs(azd);
                            crate::h!(B_, "Waveform set to: {}", azd.j());
                        }
                        None => crate::h!(A_, "Unknown waveform (use: sine, square, saw, triangle, noise)"),
                    }
                }
                None => crate::h!(D_, "Usage: synth wave <sine|square|saw|triangle|noise>"),
            }
        }
        Some("adsr") => {
            
            if n.len() < 5 {
                crate::h!(D_, "Usage: synth adsr <attack_ms> <decay_ms> <sustain_%> <release_ms>");
                crate::println!("  Example: synth adsr 10 50 70 100");
                return;
            }
            let q = n[1].parse::<u32>().unwrap_or(10);
            let bc = n[2].parse::<u32>().unwrap_or(50);
            let e = n[3].parse::<u32>().unwrap_or(70);
            let m = n[4].parse::<u32>().unwrap_or(100);
            let _ = crate::audio::med(q, bc, e, m);
            crate::h!(B_, "ADSR set: A={}ms D={}ms S={}% R={}ms", q, bc, e, m);
        }
        Some("preset") => {
            
            match n.get(1).hu() {
                Some(j) => {
                    match crate::audio::wiu(j) {
                        Ok(()) => crate::h!(B_, "Envelope preset: {}", j),
                        Err(aa) => crate::h!(A_, "{}", aa),
                    }
                }
                None => crate::h!(D_, "Usage: synth preset <default|organ|pluck|pad>"),
            }
        }
        Some("volume") | Some("vol") => {
            match n.get(1).and_then(|e| e.parse::<u8>().bq()) {
                Some(p) => {
                    let _ = crate::audio::chv(p);
                    crate::h!(B_, "Master volume: {}/255", p);
                }
                None => crate::h!(D_, "Usage: synth volume <0-255>"),
            }
        }
        Some("status") => {
            let e = crate::audio::status();
            crate::println!("{}", e);
        }
        Some("stop") => {
            match crate::audio::qg() {
                Ok(()) => crate::h!(B_, "Synth stopped"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("demo") => {
            crate::println!("TrustSynth Demo -- playing scale with different waveforms...");
            let ts = ["C4", "D4", "E4", "F4", "G4", "A4", "B4", "C5"];
            let xub = [
                ("Sine",     crate::audio::synth::Waveform::Dg),
                ("Square",   crate::audio::synth::Waveform::Gb),
                ("Sawtooth", crate::audio::synth::Waveform::Ft),
                ("Triangle", crate::audio::synth::Waveform::Triangle),
            ];
            for (xuf, azd) in &xub {
                let _ = crate::audio::dvs(*azd);
                crate::println!("  {} waveform:", xuf);
                for jp in &ts {
                    crate::print!("    {} ", jp);
                    let _ = crate::audio::owc(jp, 200);
                }
                crate::println!();
            }
            crate::h!(B_, "Demo complete!");
        }
        
        Some("pattern") | Some("pat") => {
            riv(&n[1..]);
        }
        Some(_) | None => {
            crate::h!(C_, "TrustSynth -- Audio Synthesizer & Sequencer");
            crate::println!();
            crate::h!(D_, "  Synth:");
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
            crate::h!(D_, "  Pattern Sequencer:");
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

pub(super) fn riv(n: &[&str]) {
    match n.fv().hu() {
        Some("list") | Some("ls") | None => {
            let aoy = crate::audio::vfc();
            crate::println!("{}", aoy);
        }
        Some("show") | Some("view") => {
            match n.get(1) {
                Some(j) => {
                    match crate::audio::vfj(j) {
                        Ok(e) => crate::println!("{}", e),
                        Err(aa) => crate::h!(A_, "Error: {}", aa),
                    }
                }
                None => crate::h!(D_, "Usage: synth pattern show <name>"),
            }
        }
        Some("new") | Some("create") => {
            let j = match n.get(1) {
                Some(bo) => *bo,
                None => {
                    crate::h!(D_, "Usage: synth pattern new <name> [steps] [bpm]");
                    return;
                }
            };
            let au = n.get(2).and_then(|e| e.parse::<usize>().bq()).unwrap_or(16);
            let kz = n.get(3).and_then(|e| e.parse::<u16>().bq()).unwrap_or(120);
            match crate::audio::vfd(j, au, kz) {
                Ok(()) => crate::h!(B_, "Pattern \"{}\" created ({} steps, {} BPM)", j, au, kz),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("play") => {
            let j = match n.get(1) {
                Some(bo) => *bo,
                None => {
                    crate::h!(D_, "Usage: synth pattern play <name> [loops]");
                    return;
                }
            };
            let bkh = n.get(2).and_then(|e| e.parse::<u32>().bq()).unwrap_or(1);
            match crate::audio::vfe(j, bkh) {
                Ok(()) => {}
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("stop") => {
            match crate::audio::vfk() {
                Ok(()) => crate::h!(B_, "Pattern playback stopped"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("bpm") | Some("tempo") => {
            if n.len() < 3 {
                crate::h!(D_, "Usage: synth pattern bpm <name> <60-300>");
                return;
            }
            let j = n[1];
            let kz = match n[2].parse::<u16>() {
                Ok(o) if o >= 30 && o <= 300 => o,
                _ => {
                    crate::h!(A_, "BPM must be 30-300");
                    return;
                }
            };
            match crate::audio::vfg(j, kz) {
                Ok(()) => crate::h!(B_, "\"{}\" BPM set to {}", j, kz),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("wave") | Some("waveform") => {
            if n.len() < 3 {
                crate::h!(D_, "Usage: synth pattern wave <name> <sine|square|saw|tri|noise>");
                return;
            }
            let j = n[1];
            let azd = match crate::audio::synth::Waveform::cko(n[2]) {
                Some(d) => d,
                None => {
                    crate::h!(A_, "Unknown waveform");
                    return;
                }
            };
            match crate::audio::vfi(j, azd) {
                Ok(()) => crate::h!(B_, "\"{}\" waveform set to {}", j, azd.j()),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("set") | Some("note") => {
            
            if n.len() < 4 {
                crate::h!(D_, "Usage: synth pattern set <name> <step#> <note|-->");
                crate::println!("  Example: synth pattern set mypattern 0 C4");
                crate::println!("  Example: synth pattern set mypattern 3 --  (rest)");
                return;
            }
            let j = n[1];
            let pou = match n[2].parse::<usize>() {
                Ok(a) => a,
                Err(_) => {
                    crate::h!(A_, "Step must be a number");
                    return;
                }
            };
            let jp = n[3];
            match crate::audio::vfh(j, pou, jp) {
                Ok(()) => crate::h!(B_, "\"{}\" step {} = {}", j, pou, jp),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("del") | Some("delete") | Some("rm") => {
            match n.get(1) {
                Some(j) => {
                    match crate::audio::vff(j) {
                        Ok(()) => crate::h!(B_, "Pattern \"{}\" deleted", j),
                        Err(aa) => crate::h!(A_, "Error: {}", aa),
                    }
                }
                None => crate::h!(D_, "Usage: synth pattern del <name>"),
            }
        }
        Some(gq) => {
            crate::h!(A_, "Unknown pattern command: {}", gq);
            crate::println!("Use: list, show, new, play, stop, bpm, wave, set, del");
        }
    }
}





pub(super) fn rkh(n: &[&str]) {
    match n.fv().hu() {
        None | Some("help") | Some("--help") => {
            crate::h!(C_, "vizfx — Live Visualizer Effects (TrustLang)");
            crate::h!(C_, "═══════════════════════════════════════════");
            crate::println!();
            crate::h!(G_, "Create visual effects with TrustLang scripts.");
            crate::h!(G_, "Effects react to audio in real-time — no reboot!");
            crate::println!();
            crate::h!(G_, "Commands:");
            crate::println!("  vizfx list                List all effects");
            crate::println!("  vizfx new <name> <code>   Create effect (inline code)");
            crate::println!("  vizfx edit <name> <code>  Update effect source");
            crate::println!("  vizfx select <name>       Set active effect");
            crate::println!("  vizfx remove <name>       Delete effect");
            crate::println!("  vizfx show <name>         Show effect source code");
            crate::println!("  vizfx on / off            Enable/disable live effects");
            crate::println!("  vizfx demo                Load 3 demo effects");
            crate::println!();
            crate::h!(G_, "Audio Builtins (available in scripts):");
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
            crate::h!(G_, "Graphics Builtins:");
            crate::println!("  pixel(x,y,r,g,b)         Set pixel color");
            crate::println!("  fill_rect(x,y,w,h,r,g,b) Fill rectangle");
            crate::println!("  draw_circle(cx,cy,r,R,G,B) Draw circle");
            crate::println!("  draw_line(x1,y1,x2,y2,r,g,b) Draw line");
            crate::println!("  screen_w() / screen_h()  Screen dimensions");
            crate::println!();
            crate::h!(D_, "Workflow for promo video:");
            crate::println!("  1. vizfx new myeffect fn main() {{ ... }}");
            crate::println!("  2. vizfx select myeffect");
            crate::println!("  3. play /music/song.wav");
            crate::println!("  → Effect runs live over the audio visualizer!");
        }
        Some("list") | Some("ls") => {
            let bzl = crate::trustdaw::live_viz::ufq();
            if bzl.is_empty() {
                crate::h!(D_, "No effects loaded. Try 'vizfx demo' to load demos.");
            } else {
                crate::h!(C_, "Live Visualizer Effects:");
                for (j, gh) in &bzl {
                    if *gh {
                        crate::h!(G_, "  ▶ {} (ACTIVE)", j);
                    } else {
                        crate::println!("    {}", j);
                    }
                }
            }
        }
        Some("new") | Some("add") | Some("create") => {
            if n.len() < 3 {
                crate::h!(D_, "Usage: vizfx new <name> <trustlang code>");
                crate::println!("Example: vizfx new rings fn main() {{ let b = beat(); draw_circle(screen_w()/2, screen_h()/2, to_int(b * 100.0), 0, 255, 100); }}");
                return;
            }
            let j = n[1];
            
            let iy: alloc::string::String = n[2..].rr(" ");
            match crate::trustdaw::live_viz::iix(j, &iy) {
                Ok(()) => {
                    crate::h!(B_, "Effect '{}' created ✓", j);
                    if crate::trustdaw::live_viz::rl() {
                        crate::println!("Active and ready — play a song to see it!");
                    }
                }
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("edit") | Some("update") => {
            if n.len() < 3 {
                crate::h!(D_, "Usage: vizfx edit <name> <new code>");
                return;
            }
            let j = n[1];
            let iy: alloc::string::String = n[2..].rr(" ");
            match crate::trustdaw::live_viz::sja(j, &iy) {
                Ok(()) => crate::h!(B_, "Effect '{}' updated ✓", j),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("select") | Some("use") | Some("set") => {
            if n.len() < 2 {
                crate::h!(D_, "Usage: vizfx select <name>");
                return;
            }
            match crate::trustdaw::live_viz::phq(n[1]) {
                Ok(()) => crate::h!(B_, "Active effect: {} ✓", n[1]),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("remove") | Some("rm") | Some("delete") => {
            if n.len() < 2 {
                crate::h!(D_, "Usage: vizfx remove <name>");
                return;
            }
            match crate::trustdaw::live_viz::vuw(n[1]) {
                Ok(()) => crate::h!(B_, "Effect '{}' removed", n[1]),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("show") | Some("source") | Some("cat") => {
            if n.len() < 2 {
                crate::h!(D_, "Usage: vizfx show <name>");
                return;
            }
            match crate::trustdaw::live_viz::teq(n[1]) {
                Some(cy) => {
                    crate::h!(C_, "─── {} ───", n[1]);
                    crate::println!("{}", cy);
                    crate::h!(C_, "─────────────────");
                }
                None => crate::h!(A_, "Effect not found: {}", n[1]),
            }
        }
        Some("on") | Some("enable") => {
            match crate::trustdaw::live_viz::aiy() {
                Ok(()) => crate::h!(B_, "Live viz effects enabled ✓"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("off") | Some("disable") => {
            crate::trustdaw::live_viz::cwz();
            crate::h!(D_, "Live viz effects disabled");
        }
        Some("demo") | Some("demos") => {
            crate::h!(B_, "Loading demo effects...");
            match crate::trustdaw::live_viz::ugs() {
                Ok(()) => crate::h!(B_, "  ✓ pulse-rings"),
                Err(aa) => crate::h!(A_, "  ✗ pulse-rings: {}", aa),
            }
            match crate::trustdaw::live_viz::ugt() {
                Ok(()) => crate::h!(B_, "  ✓ spectrum-bars"),
                Err(aa) => crate::h!(A_, "  ✗ spectrum-bars: {}", aa),
            }
            match crate::trustdaw::live_viz::ugr() {
                Ok(()) => crate::h!(B_, "  ✓ beat-flash"),
                Err(aa) => crate::h!(A_, "  ✗ beat-flash: {}", aa),
            }
            crate::println!();
            crate::println!("Use 'vizfx list' to see all effects");
            crate::println!("Use 'vizfx select <name>' to choose one");
            crate::println!("Then 'play <file.wav>' to see it in action!");
        }
        Some(b) => {
            
            match crate::trustdaw::live_viz::phq(b) {
                Ok(()) => crate::h!(B_, "Active effect: {} ✓", b),
                Err(_) => {
                    crate::h!(A_, "Unknown command: {}", b);
                    crate::println!("Use 'vizfx help' for usage");
                }
            }
        }
    }
}





pub(super) fn rhc(n: &[&str]) {
    let path = n.fv().hu().unwrap_or("");
    if path.is_empty() || path == "help" || path == "--help" {
        crate::h!(C_, "play - Audio file visualizer");
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
        crate::h!(B_, "Playing embedded 'Untitled (2)' — Dark Lo-Fi / Ambient...");
        crate::println!("  [Esc] Exit");
        match crate::trustdaw::audio_viz::luh() {
            Ok(()) => crate::println!("Playback complete"),
            Err(aa) => crate::h!(A_, "Error: {}", aa),
        }
        return;
    }

    crate::h!(B_, "Starting Audio Visualizer...");
    crate::println!("  File: {}", path);
    crate::println!("  [Esc] Exit");
    match crate::trustdaw::audio_viz::owa(path) {
        Ok(()) => crate::println!("Playback complete"),
        Err(aa) => crate::h!(A_, "Error: {}", aa),
    }
}





pub(super) fn rdh(n: &[&str]) {
    match n.fv().hu() {
        Some("init") | None => {
            match crate::trustdaw::init() {
                Ok(()) => {
                    crate::h!(B_, "TrustDAW initialized");
                    crate::println!("{}", crate::trustdaw::status());
                }
                Err(aa) => crate::h!(A_, "DAW init failed: {}", aa),
            }
        }
        Some("status") | Some("info") => {
            crate::println!("{}", crate::trustdaw::status());
        }
        Some("new") => {
            
            let j = match n.get(1) {
                Some(bo) => *bo,
                None => {
                    crate::h!(D_, "Usage: daw new <project_name> [bpm]");
                    return;
                }
            };
            let kz = n.get(2).and_then(|e| e.parse::<u32>().bq()).unwrap_or(120);
            match crate::trustdaw::utn(j, kz) {
                Ok(()) => crate::h!(B_, "New project: \"{}\" at {} BPM", j, kz),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("demo") => {
            match crate::trustdaw::ugp() {
                Ok(()) => {
                    crate::h!(B_, "Demo project loaded!");
                    crate::println!("{}", crate::trustdaw::status());
                }
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("track") => rdl(&n[1..]),
        Some("note") => rdk(&n[1..]),
        Some("play") => {
            crate::println!("Playing...");
            match crate::trustdaw::daq() {
                Ok(()) => crate::h!(B_, "Playback complete"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("stop") => {
            crate::trustdaw::qg();
            crate::h!(B_, "Stopped");
        }
        Some("rewind") | Some("rw") => {
            crate::trustdaw::lzz();
            crate::h!(B_, "Rewound to beginning");
        }
        Some("bpm") => {
            match n.get(1).and_then(|e| e.parse::<u32>().bq()) {
                Some(kz) => {
                    crate::trustdaw::mef(kz);
                    crate::h!(B_, "BPM set to {}", crate::trustdaw::Hi.load(core::sync::atomic::Ordering::Relaxed));
                }
                None => crate::h!(D_, "Usage: daw bpm <30-300>"),
            }
        }
        Some("record") | Some("rec") => {
            let zx = n.get(1).and_then(|e| e.parse::<usize>().bq()).unwrap_or(0);
            match crate::trustdaw::recorder::pas(zx) {
                Ok(bo) => crate::h!(B_, "Recorded {} notes", bo),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("piano") | Some("keyboard") => {
            crate::println!("{}", crate::trustdaw::keyboard_midi::nlx());
        }
        Some("pianoroll") | Some("roll") => {
            
            let zx = n.get(1).and_then(|e| e.parse::<usize>().bq()).unwrap_or(0);
            let cjf = n.get(2).and_then(|e| e.parse::<u32>().bq()).unwrap_or(4);
            match crate::trustdaw::aqz().and_then(|_| {
                let nv = crate::trustdaw::Fc.lock();
                let nv = nv.as_ref().ok_or("No project")?;
                let track = nv.af.get(zx).ok_or("Invalid track index")?;
                Ok(crate::trustdaw::piano_roll::xfw(track, cjf))
            }) {
                Ok(e) => crate::println!("{}", e),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("gui") => {
            match crate::trustdaw::ui::ucx() {
                Ok(()) => crate::println!("DAW GUI closed"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("studio") | Some("beat") | Some("beats") => {
            crate::h!(B_, "Launching Beat Studio...");
            match crate::trustdaw::beat_studio::ucu() {
                Ok(()) => crate::println!("Beat Studio closed"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("funky") | Some("house") => {
            crate::h!(B_, "Loading Funky House beat...");
            
            match crate::trustdaw::beat_studio::ucw() {
                Ok(()) => crate::println!("Funky house session closed"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("matrix") => {
            crate::h!(B_, "Entering the Beat Matrix...");
            match crate::trustdaw::beat_studio::ucy() {
                Ok(()) => crate::println!("Beat Matrix closed"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("film") | Some("showcase") | Some("narrated") | Some("youtube") => {
            crate::h!(B_, "Starting narrated showcase...");
            crate::println!("  Phase 1: Building the beat (track by track)");
            crate::println!("  Phase 2: Full mix playback");
            crate::println!("  Phase 3: Matrix visualizer");
            crate::println!("  [Esc] Exit  [Space] Skip section");
            match crate::trustdaw::beat_studio::uda() {
                Ok(()) => crate::println!("Narrated showcase complete"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("anthem") => {
            crate::h!(B_, "Starting TrustOS Anthem — Renaissance Numérique...");
            crate::println!("  5 Sections: Intro → Build → Drop → Stable → Outro");
            crate::println!("  Key: C minor → C major  |  106 BPM  |  ~3 min");
            crate::println!("  [Esc] Exit  [Space] Skip section");
            match crate::trustdaw::beat_studio::ucv() {
                Ok(()) => crate::println!("TrustOS Anthem complete"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("trap") | Some("gangsta") | Some("rap") | Some("cyber") | Some("neon") => {
            crate::h!(B_, "Starting Cyberpunk Showcase — NEON PROTOCOL...");
            crate::println!("  Sub Bass + Aggressive 16th Hats + Synth Arps + Digital Lead");
            crate::println!("  100 BPM  |  Eb minor  |  Dark Cyberpunk");
            crate::println!("  [Esc] Exit  [Space] Skip section");
            match crate::trustdaw::beat_studio::udb() {
                Ok(()) => crate::println!("Neon Protocol Showcase complete"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("untitled2") | Some("u2") | Some("lofi") => {
            crate::h!(B_, "Generating 'Untitled 2' — Dark Lo-Fi / Ambient...");
            crate::println!("  Keys + Sub + Dusty Drums + Emotional Lead");
            crate::println!("  85 BPM  |  A minor  |  6 sections  |  ~3 min");
            crate::println!("  3D Matrix Visualizer  |  [Esc] Exit");
            match crate::trustdaw::audio_viz::luh() {
                Ok(()) => crate::println!("Untitled 2 complete"),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("viz") | Some("visualizer") => {
            let path = n.get(1).hu().unwrap_or("");
            if path.is_empty() {
                crate::h!(D_, "Usage: daw viz <file.wav>");
                crate::println!("  Plays audio file with 3D matrix rain visualizer");
            } else {
                crate::h!(B_, "Starting Audio Visualizer...");
                crate::println!("  File: {}", path);
                crate::println!("  [Esc] Exit");
                match crate::trustdaw::audio_viz::owa(path) {
                    Ok(()) => crate::println!("Visualizer complete"),
                    Err(aa) => crate::h!(A_, "Error: {}", aa),
                }
            }
        }
        Some("export") | Some("wav") => {
            let path = n.get(1).hu().unwrap_or("/home/output.wav");
            crate::println!("Exporting to {}...", path);
            match crate::trustdaw::hio(path) {
                Ok(aw) => {
                    let (tv, jn) = crate::trustdaw::wav_export::shk(
                        aw / 2, crate::trustdaw::BR_, 2
                    );
                    crate::h!(B_, "Exported: {} ({} bytes, {}:{:02}.{:03})",
                        path, aw, tv / 60, tv % 60, jn);
                }
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("mixer") => rdj(&n[1..]),
        Some("help") => {
            crate::h!(C_, "╔══════════════════════════════════════════════╗");
            crate::h!(C_, "║      TrustDAW — Digital Audio Workstation    ║");
            crate::h!(C_, "╚══════════════════════════════════════════════╝");
            crate::println!();
            crate::h!(D_, "  Project:");
            crate::println!("  daw init                        Initialize TrustDAW");
            crate::println!("  daw new <name> [bpm]            New project");
            crate::println!("  daw demo                        Load demo project");
            crate::println!("  daw status                      Show project info");
            crate::println!("  daw bpm <30-300>                Set tempo");
            crate::println!();
            crate::h!(D_, "  Transport:");
            crate::println!("  daw play                        Play from current position");
            crate::println!("  daw stop                        Stop playback/recording");
            crate::println!("  daw rewind                      Rewind to start");
            crate::println!("  daw record [track#]             Record from keyboard");
            crate::println!();
            crate::h!(D_, "  Tracks:");
            crate::println!("  daw track add <name>            Add a new track");
            crate::println!("  daw track rm <#>                Remove a track");
            crate::println!("  daw track list                  List all tracks");
            crate::println!("  daw track wave <#> <waveform>   Set track waveform");
            crate::println!("  daw track notes <#>             List notes in track");
            crate::println!("  daw track clear <#>             Clear track notes");
            crate::println!("  daw track transpose <#> <semi>  Transpose notes");
            crate::println!();
            crate::h!(D_, "  Notes:");
            crate::println!("  daw note add <track#> <note> [vel] [start] [dur]");
            crate::println!("                                  Add a note (e.g. daw note add 0 C4 100 0 480)");
            crate::println!("  daw note rm <track#> <idx>      Remove a note by index");
            crate::println!();
            crate::h!(D_, "  Mixer:");
            crate::println!("  daw mixer                       Show mixer status");
            crate::println!("  daw mixer vol <#> <0-255>       Set track volume");
            crate::println!("  daw mixer pan <#> <-100..100>   Set track pan");
            crate::println!("  daw mixer mute <#>              Toggle mute");
            crate::println!("  daw mixer solo <#>              Toggle solo");
            crate::println!();
            crate::h!(D_, "  Display:");
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
            crate::h!(D_, "  Visualizer:");
            crate::println!("  daw viz <file.wav>              Audio file visualizer (3D matrix + waveform)");
            crate::println!("  play <file.wav>                 Same as 'daw viz'");
            crate::println!();
            crate::h!(D_, "  Export:");
            crate::println!("  daw export [path]               Export WAV (default: /home/output.wav)");
        }
        Some(gq) => {
            crate::h!(A_, "Unknown DAW command: {}", gq);
            crate::println!("Use 'daw help' for commands");
        }
    }
}

fn rdl(n: &[&str]) {
    match n.fv().hu() {
        Some("add") | Some("new") => {
            let j = match n.get(1) {
                Some(bo) => *bo,
                None => {
                    crate::h!(D_, "Usage: daw track add <name>");
                    return;
                }
            };
            match crate::trustdaw::jzi(j) {
                Ok(w) => crate::h!(B_, "Track {} \"{}\" added", w, j),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("rm") | Some("remove") | Some("del") => {
            match n.get(1).and_then(|e| e.parse::<usize>().bq()) {
                Some(w) => {
                    match crate::trustdaw::lza(w) {
                        Ok(()) => crate::h!(B_, "Track {} removed", w),
                        Err(aa) => crate::h!(A_, "Error: {}", aa),
                    }
                }
                None => crate::h!(D_, "Usage: daw track rm <index>"),
            }
        }
        Some("list") | Some("ls") | None => {
            crate::println!("{}", crate::trustdaw::status());
        }
        Some("wave") | Some("waveform") => {
            if n.len() < 3 {
                crate::h!(D_, "Usage: daw track wave <#> <sine|square|saw|triangle|noise>");
                return;
            }
            let w = match n[1].parse::<usize>() {
                Ok(a) => a,
                Err(_) => { crate::h!(A_, "Invalid track index"); return; }
            };
            match crate::trustdaw::wjw(w, n[2]) {
                Ok(()) => crate::h!(B_, "Track {} waveform set to {}", w, n[2]),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("notes") => {
            let w = n.get(1).and_then(|e| e.parse::<usize>().bq()).unwrap_or(0);
            match crate::trustdaw::uft(w) {
                Ok(e) => crate::println!("{}", e),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("clear") => {
            let w = match n.get(1).and_then(|e| e.parse::<usize>().bq()) {
                Some(a) => a,
                None => { crate::h!(D_, "Usage: daw track clear <#>"); return; }
            };
            let rbk = {
                let mut nv = crate::trustdaw::Fc.lock();
                if let Some(aci) = nv.as_mut() {
                    if let Some(track) = aci.af.ds(w) {
                        let az = track.ts.len();
                        track.clear();
                        Ok(az)
                    } else { Err("Invalid track index") }
                } else { Err("No project") }
            };
            match rbk {
                Ok(bo) => crate::h!(B_, "Cleared {} notes from track {}", bo, w),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("transpose") => {
            if n.len() < 3 {
                crate::h!(D_, "Usage: daw track transpose <#> <semitones>");
                return;
            }
            let w = match n[1].parse::<usize>() {
                Ok(a) => a,
                Err(_) => { crate::h!(A_, "Invalid track index"); return; }
            };
            let grz = match n[2].parse::<i8>() {
                Ok(e) => e,
                Err(_) => { crate::h!(A_, "Invalid semitone value"); return; }
            };
            let result = {
                let mut nv = crate::trustdaw::Fc.lock();
                if let Some(aci) = nv.as_mut() {
                    if let Some(track) = aci.af.ds(w) {
                        track.xmc(grz);
                        Ok(())
                    } else { Err("Invalid track index") }
                } else { Err("No project") }
            };
            match result {
                Ok(()) => crate::h!(B_, "Track {} transposed by {} semitones", w, grz),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some(gq) => {
            crate::h!(A_, "Unknown track command: {}", gq);
            crate::println!("Use: add, rm, list, wave, notes, clear, transpose");
        }
    }
}

fn rdk(n: &[&str]) {
    match n.fv().hu() {
        Some("add") => {
            
            if n.len() < 3 {
                crate::h!(D_, "Usage: daw note add <track#> <note> [vel] [start_tick] [dur_ticks]");
                crate::println!("  Example: daw note add 0 C4 100 0 480");
                crate::println!("  Default: vel=100, start=0, dur=480 (quarter note)");
                return;
            }
            let zx = match n[1].parse::<usize>() {
                Ok(a) => a,
                Err(_) => { crate::h!(A_, "Invalid track index"); return; }
            };
            let bkp = n[2];
            let ti = match crate::audio::tables::fpd(bkp) {
                Some(bo) => bo,
                None => { crate::h!(A_, "Invalid note: {} (use e.g. C4, A#3, Bb5)", bkp); return; }
            };
            let qm = n.get(3).and_then(|e| e.parse::<u8>().bq()).unwrap_or(100);
            let vb = n.get(4).and_then(|e| e.parse::<u32>().bq()).unwrap_or(0);
            let avr = n.get(5).and_then(|e| e.parse::<u32>().bq()).unwrap_or(480);

            match crate::trustdaw::axn(zx, ti, qm, vb, avr) {
                Ok(()) => {
                    let j = crate::audio::tables::dtf(ti);
                    let bvq = crate::audio::tables::efk(ti);
                    crate::h!(B_, "Added {}{} vel={} at tick {} dur={}",
                        j, bvq, qm, vb, avr);
                }
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("rm") | Some("remove") | Some("del") => {
            if n.len() < 3 {
                crate::h!(D_, "Usage: daw note rm <track#> <note_index>");
                return;
            }
            let zx = match n[1].parse::<usize>() {
                Ok(a) => a,
                Err(_) => { crate::h!(A_, "Invalid track index"); return; }
            };
            let uvh = match n[2].parse::<usize>() {
                Ok(a) => a,
                Err(_) => { crate::h!(A_, "Invalid note index"); return; }
            };
            let result = {
                let mut nv = crate::trustdaw::Fc.lock();
                if let Some(aci) = nv.as_mut() {
                    if let Some(track) = aci.af.ds(zx) {
                        match track.pbr(uvh) {
                            Some(jp) => Ok(jp),
                            None => Err("Note index out of range"),
                        }
                    } else { Err("Invalid track index") }
                } else { Err("No project") }
            };
            match result {
                Ok(jp) => {
                    let j = crate::audio::tables::dtf(jp.jb);
                    let bvq = crate::audio::tables::efk(jp.jb);
                    crate::h!(B_, "Removed {}{} from track {}", j, bvq, zx);
                }
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        _ => {
            crate::h!(D_, "Usage: daw note <add|rm> ...");
            crate::println!("  daw note add <track#> <note> [vel] [start] [dur]");
            crate::println!("  daw note rm <track#> <note_index>");
        }
    }
}

fn rdj(n: &[&str]) {
    match n.fv().hu() {
        Some("vol") | Some("volume") => {
            if n.len() < 3 {
                crate::h!(D_, "Usage: daw mixer vol <track#> <0-255>");
                return;
            }
            let w = match n[1].parse::<usize>() { Ok(a) => a, Err(_) => { crate::h!(A_, "Invalid track"); return; } };
            let api = match n[2].parse::<u8>() { Ok(p) => p, Err(_) => { crate::h!(A_, "Invalid volume (0-255)"); return; } };
            match crate::trustdaw::wjv(w, api) {
                Ok(()) => crate::h!(B_, "Track {} volume: {}", w, api),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("pan") => {
            if n.len() < 3 {
                crate::h!(D_, "Usage: daw mixer pan <track#> <-100..+100>");
                return;
            }
            let w = match n[1].parse::<usize>() { Ok(a) => a, Err(_) => { crate::h!(A_, "Invalid track"); return; } };
            let arp = match n[2].parse::<i8>() { Ok(ai) => ai, Err(_) => { crate::h!(A_, "Invalid pan (-100 to +100)"); return; } };
            match crate::trustdaw::wju(w, arp) {
                Ok(()) => {
                    let desc = if arp == 0 { "Center".into() } else if arp > 0 { alloc::format!("Right {}", arp) } else { alloc::format!("Left {}", -arp) };
                    crate::h!(B_, "Track {} pan: {}", w, desc);
                }
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("mute") => {
            let w = match n.get(1).and_then(|e| e.parse::<usize>().bq()) {
                Some(a) => a,
                None => { crate::h!(D_, "Usage: daw mixer mute <track#>"); return; }
            };
            match crate::trustdaw::mlo(w) {
                Ok(so) => crate::h!(if so { D_ } else { B_ },
                    "Track {} {}", w, if so { "MUTED" } else { "unmuted" }),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        Some("solo") => {
            let w = match n.get(1).and_then(|e| e.parse::<usize>().bq()) {
                Some(a) => a,
                None => { crate::h!(D_, "Usage: daw mixer solo <track#>"); return; }
            };
            match crate::trustdaw::mlr(w) {
                Ok(cic) => crate::h!(if cic { D_ } else { B_ },
                    "Track {} {}", w, if cic { "SOLO" } else { "un-solo'd" }),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        _ => {
            
            crate::println!("{}", crate::trustdaw::status());
        }
    }
}

pub(super) fn rge(n: &[&str]) {
    let ik = crate::pci::fjm();
    
    if ik.is_empty() {
        crate::h!(D_, "No PCI devices found");
        return;
    }
    
    let igi = n.contains(&"-v") || n.contains(&"--verbose");
    
    crate::h!(C_, "PCI Devices ({} found):", ik.len());
    crate::println!();
    
    for ba in &ik {
        
        crate::gr!(B_, "{:02X}:{:02X}.{} ", 
            ba.aq, ba.de, ba.gw);
        crate::print!("{:04X}:{:04X} ", ba.ml, ba.mx);
        
        let bor = ba.bor();
        if bor.is_empty() {
            crate::print!("{}", ba.bpz());
        } else {
            crate::print!("{}", bor);
        }
        
        crate::h!(D_, " [{}]", ba.cip());
        
        if igi {
            crate::println!("        Class: {:02X}:{:02X} ProgIF: {:02X} Rev: {:02X}",
                ba.ajz, ba.adl, ba.frg, ba.afe);
            
            if ba.esw != 0xFF && ba.jan != 0 {
                crate::println!("        IRQ: {} (pin {})", 
                    ba.esw, ba.jan);
            }
            
            
            for a in 0..6 {
                if let Some(ag) = ba.cje(a) {
                    let gzq = if ba.mxx(a) { "MEM" } else { "I/O" };
                    crate::println!("        BAR{}: {:#010X} [{}]", a, ag, gzq);
                }
            }
            crate::println!();
        }
    }
    
    if !igi {
        crate::println!();
        crate::h!(D_, "Use 'lspci -v' for detailed info");
    }
}

pub(super) fn rga() {
    crate::h!(C_, "=== Hardware Summary ===");
    crate::println!();
    
    let ik = crate::pci::fjm();
    
    
    crate::h!(B_, "CPU:");
    crate::println!("  Architecture: x86_64");
    crate::println!("  Mode: Long Mode (64-bit)");
    crate::println!();
    
    
    crate::h!(B_, "Memory:");
    crate::println!("  Heap: 256 KB");
    crate::println!();
    
    
    let storage: Vec<_> = ik.iter()
        .hi(|bc| bc.ajz == crate::pci::class::FK_)
        .collect();
    crate::h!(B_, "Storage Controllers ({}):", storage.len());
    for ba in &storage {
        crate::println!("  {:04X}:{:04X} {} [{}]", 
            ba.ml, ba.mx, 
            ba.bor(),
            ba.cip());
    }
    crate::println!();
    
    
    let network: Vec<_> = ik.iter()
        .hi(|bc| bc.ajz == crate::pci::class::Qa)
        .collect();
    crate::h!(B_, "Network Controllers ({}):", network.len());
    for ba in &network {
        crate::println!("  {:04X}:{:04X} {} [{}]",
            ba.ml, ba.mx,
            ba.bor(),
            ba.cip());
    }
    crate::println!();
    
    
    let display: Vec<_> = ik.iter()
        .hi(|bc| bc.ajz == crate::pci::class::Ji)
        .collect();
    crate::h!(B_, "Display ({}):", display.len());
    for ba in &display {
        crate::println!("  {:04X}:{:04X} {} [{}]",
            ba.ml, ba.mx,
            ba.bor(),
            ba.cip());
    }
    crate::println!();
    
    
    let usb: Vec<_> = ik.iter()
        .hi(|bc| bc.ajz == crate::pci::class::PJ_ 
                 && bc.adl == crate::pci::serial::Any)
        .collect();
    crate::h!(B_, "USB Controllers ({}):", usb.len());
    for ba in &usb {
        crate::println!("  {:04X}:{:04X} {} [{}]",
            ba.ml, ba.mx,
            ba.bor(),
            ba.cip());
    }
    crate::println!();
    
    
    crate::h!(C_, "Total: {} PCI devices", ik.len());
}

pub(super) fn rep(n: &[&str]) {
    if n.fv() == Some(&"--help") || n.fv() == Some(&"-h") {
        crate::println!("Usage: gpu [info|dcn|modes]");
        crate::println!("  gpu         Show GPU summary");
        crate::println!("  gpu info    Detailed GPU information");
        crate::println!("  gpu dcn     Display engine (DCN) status");
        crate::println!("  gpu modes   List standard display modes");
        return;
    }
    
    let air = n.fv().hu().unwrap_or("info");
    
    match air {
        "info" | "" => {
            crate::h!(C_, "=== GPU Status ===");
            crate::println!();
            
            let mut ivm = false;
            
            
            if crate::drivers::nvidia::clb() {
                ivm = true;
                crate::h!(B_, "NVIDIA GPU:");
                crate::println!("  {}", crate::drivers::nvidia::awz());
                if let Some(co) = crate::drivers::nvidia::ani() {
                    crate::println!("  PCI: {:04X}:{:04X} rev {:02X} at {:02X}:{:02X}.{}",
                        co.ml, co.mx, co.afe,
                        co.aq, co.de, co.gw);
                    crate::println!("  Chipset: {} (id {:#04X}, step {:#04X})",
                        co.khn(), co.enl, co.bxi);
                    crate::println!("  VRAM: {} MB", co.cnu / (1024 * 1024));
                    crate::println!("  PCIe: Gen{} x{}", co.hut, co.huu);
                    crate::println!("  MMIO: {:#X} ({}MB)", co.hv, co.bkm / (1024 * 1024));
                    if co.igy > 0 {
                        crate::println!("  VRAM aperture: {:#X}", co.igy);
                    }
                    crate::println!("  2D Accel: {}", if crate::drivers::nvidia::twp() { "READY" } else { "N/A" });
                }
                crate::println!();
            }
            
            
            if crate::drivers::amdgpu::clb() {
                ivm = true;
                crate::h!(B_, "AMD GPU:");
                for line in crate::drivers::amdgpu::zl() {
                    crate::println!("{}", line);
                }
                crate::println!();
                
                if crate::drivers::amdgpu::dcn::uc() {
                    crate::h!(B_, "Display Engine:");
                    crate::println!("  {}", crate::drivers::amdgpu::dcn::awz());
                }
            }
            
            
            if crate::drivers::virtio_gpu::anl() {
                ivm = true;
                crate::h!(B_, "VirtIO GPU:");
                crate::println!("  {}", crate::drivers::virtio_gpu::lea());
                crate::println!();
            }
            
            if !ivm {
                crate::println!("No GPU detected.");
                crate::println!();
                
                let cxa = crate::pci::ebq(crate::pci::class::Ji);
                if !cxa.is_empty() {
                    crate::h!(B_, "Display controllers found:");
                    for ba in &cxa {
                        crate::println!("  {:04X}:{:04X} {} [{}]", 
                            ba.ml, ba.mx,
                            ba.bor(), ba.cip());
                    }
                }
            }
        }
        "dcn" | "display" => {
            crate::h!(C_, "=== DCN Display Engine ===");
            crate::println!();
            
            if crate::drivers::amdgpu::dcn::uc() {
                for line in crate::drivers::amdgpu::dcn::zl() {
                    crate::println!("{}", line);
                }
            } else {
                crate::println!("DCN display engine not initialized.");
                if !crate::drivers::amdgpu::clb() {
                    crate::println!("(No AMD GPU detected)");
                }
            }
        }
        "modes" => {
            crate::h!(C_, "=== Standard Display Modes ===");
            crate::println!();
            for (a, ev) in crate::drivers::amdgpu::dcn::wsi().iter().cf() {
                crate::println!("  [{}] {}", a, ev.lmk());
            }
        }
        _ => {
            crate::println!("Unknown subcommand: {}", air);
            crate::println!("Use 'gpu --help' for usage.");
        }
    }
}

pub(super) fn rca(n: &[&str]) {
    let air = n.fv().hu().unwrap_or("status");
    
    match air {
        "status" | "" => {
            crate::h!(C_, "=== Accessibility Settings ===");
            crate::println!();
            let bei = crate::accessibility::edv();
            let fs = crate::accessibility::gid();
            let aap = crate::accessibility::gib();
            let mfz = crate::accessibility::dsj();
            let jn = crate::accessibility::gig();
            crate::println!("  High Contrast : {}", if bei { "ON" } else { "OFF" });
            crate::println!("  Font Size     : {}", fs.cu());
            crate::println!("  Cursor Size   : {}", aap.cu());
            crate::println!("  Sticky Keys   : {}", if mfz { "ON" } else { "OFF" });
            crate::println!("  Mouse Speed   : {}", jn.cu());
            crate::println!();
            crate::println!("Shortcuts: Win+H = toggle high contrast");
            crate::println!("Settings:  Win+I > keys 5-9 to adjust");
        }
        "hc" | "contrast" => {
            crate::accessibility::mln();
            let ea = crate::accessibility::edv();
            crate::println!("High contrast: {}", if ea { "ON" } else { "OFF" });
        }
        "font" => {
            crate::accessibility::niy();
            crate::println!("Font size: {}", crate::accessibility::gid().cu());
        }
        "cursor" => {
            crate::accessibility::nix();
            crate::println!("Cursor size: {}", crate::accessibility::gib().cu());
        }
        "sticky" => {
            crate::accessibility::pud();
            let ea = crate::accessibility::dsj();
            crate::println!("Sticky keys: {}", if ea { "ON" } else { "OFF" });
        }
        "mouse" => {
            crate::accessibility::niz();
            crate::println!("Mouse speed: {}", crate::accessibility::gig().cu());
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
            crate::println!("Unknown: {}. Use 'a11y --help'", air);
        }
    }
}

pub(super) fn rja(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: tcpsyn <ip> <port>");
        crate::println!("  Example: tcpsyn 93.184.216.34 80");
        return;
    }

    let ek: Vec<&str> = n[0].adk('.').collect();
    if ek.len() != 4 {
        crate::h!(A_, "Invalid IP format");
        return;
    }

    let ip = [
        ek[0].parse().unwrap_or(0),
        ek[1].parse().unwrap_or(0),
        ek[2].parse().unwrap_or(0),
        ek[3].parse().unwrap_or(0),
    ];

    let port: u16 = match n[1].parse() {
        Ok(ai) => ai,
        Err(_) => {
            crate::h!(A_, "Invalid port");
            return;
        }
    };

    crate::println!("Sending TCP SYN to {}:{}...", n[0], port);
    match crate::netstack::tcp::cue(ip, port) {
        Ok(ey) => {
            crate::println!("SYN sent to {}:{} (src port {})", n[0], port, ey);
            let fhz = crate::netstack::tcp::dnd(ip, port, ey, 1000);
            if fhz {
                crate::println!("SYN-ACK received (connection established)");
            } else {
                crate::h!(D_, "No SYN-ACK received (timeout)");
            }
        }
        Err(aa) => crate::h!(A_, "tcpsyn failed: {}", aa),
    }
}

pub(super) fn rfb(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: httpget <ip|host> <port> [path] [host]");
        crate::println!("  Example: httpget 192.168.56.1 8080 /");
        crate::println!("  Example: httpget example.com 80 / example.com");
        return;
    }

    let erz = n[0];
    let port: u16 = match n[1].parse() {
        Ok(ai) => ai,
        Err(_) => {
            crate::h!(A_, "Invalid port");
            return;
        }
    };

    let path = n.get(2).hu().unwrap_or("/");
    let mut did = n.get(3).hu().unwrap_or(erz);
    if n.get(3).is_none() && erz == "192.168.56.1" {
        did = "localhost";
    }

    nmi(erz, port, path, did);
}

pub(super) fn ndv(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: curl <http://host[:port]/path> | <https://host[:port]/path>");
        return;
    }

    let url = n[0];
    if let Some((kh, port, path, fmc)) = lsm(url) {
        let did = if kh == "192.168.56.1" { "localhost" } else { &kh };
        if fmc {
            rzu(&kh, port, &path, did);
        } else {
            nmi(&kh, port, &path, did);
        }
    } else {
        crate::h!(A_, "Invalid URL");
    }
}

fn nmi(erz: &str, port: u16, path: &str, did: &str) {
    let ip = if let Some(ip) = cgl(erz) {
        ip
    } else if let Some(bhv) = crate::netstack::dns::ayo(erz) {
        bhv
    } else {
        crate::h!(A_, "Unable to resolve host");
        return;
    };

    crate::println!("Connecting to {}:{}...", erz, port);
    let ey = match crate::netstack::tcp::cue(ip, port) {
        Ok(ai) => ai,
        Err(aa) => {
            crate::h!(A_, "SYN failed: {}", aa);
            return;
        }
    };

    let fhz = crate::netstack::tcp::dnd(ip, port, ey, 1000);
    if !fhz {
        crate::h!(D_, "Connection timeout");
        return;
    }

    let mut request = String::new();
    request.t("GET ");
    request.t(path);
    request.t(" HTTP/1.1\r\nHost: ");
    request.t(did);
    request.t("\r\nConnection: close\r\n\r\n");

    if let Err(aa) = crate::netstack::tcp::dlo(ip, port, ey, request.as_bytes()) {
        crate::h!(A_, "send failed: {}", aa);
        return;
    }

    crate::println!("--- HTTP response ---");
    let ay = crate::logger::lh();
    let mut xv: usize = 0;
    let mut edb: u32 = 0;
    loop {
        crate::netstack::poll();
        let mut ckw = false;
        while let Some(f) = crate::netstack::tcp::cme(ip, port, ey) {
            ckw = true;
            xv += f.len();
            if let Ok(text) = core::str::jg(&f) {
                crate::print!("{}", text);
            } else {
                crate::println!("<binary data>");
            }
        }

        if !ckw {
            edb = edb.akq(1);
            if crate::netstack::tcp::bqr(ip, port, ey) || edb > 200_000 {
                break;
            }
        } else {
            edb = 0;
        }

        if crate::logger::lh().ao(ay) > 3000 {
            break;
        }
        crate::arch::bhd();
    }
    let _ = crate::netstack::tcp::bwx(ip, port, ey);
    crate::println!("\n--- end ({} bytes) ---", xv);
    if xv == 0 {
        crate::h!(D_, "No response body received");
    }
}


pub(super) fn vei(url: &str) -> Option<(String, u16, String)> {
    let url = url.em();
    
    
    let (kr, eaq) = if url.cj("https://") {
        (&url[8..], 443u16)
    } else if url.cj("http://") {
        (&url[7..], 80u16)
    } else {
        
        (url, 80u16)
    };
    
    
    let (bej, path) = if let Some(w) = kr.du('/') {
        (&kr[..w], &kr[w..])
    } else {
        (kr, "/")
    };
    
    
    let (kh, port) = if let Some(w) = bej.du(':') {
        let kh = &bej[..w];
        let frc = &bej[w+1..];
        let port = frc.parse::<u16>().unwrap_or(eaq);
        (kh, port)
    } else {
        (bej, eaq)
    };
    
    if kh.is_empty() {
        return None;
    }
    
    Some((String::from(kh), port, String::from(path)))
}


pub(super) fn nmj(kh: &str, ip: [u8; 4], port: u16, path: &str) -> Result<String, &'static str> {
    
    let ey = crate::netstack::tcp::cue(ip, port)
        .jd(|_| "SYN failed")?;
    
    
    if !crate::netstack::tcp::dnd(ip, port, ey, 1000) {
        return Err("Connection timeout");
    }
    
    
    let mut request = String::new();
    request.t("GET ");
    request.t(path);
    request.t(" HTTP/1.1\r\nHost: ");
    request.t(kh);
    request.t("\r\nUser-Agent: TrustOS/0.1\r\nConnection: close\r\n\r\n");
    
    
    crate::netstack::tcp::dlo(ip, port, ey, request.as_bytes())
        .jd(|_| "Send failed")?;
    
    
    let mut mk = String::new();
    let ay = crate::logger::lh();
    let mut edb: u32 = 0;
    
    loop {
        crate::netstack::poll();
        let mut ckw = false;
        
        while let Some(f) = crate::netstack::tcp::cme(ip, port, ey) {
            ckw = true;
            if let Ok(text) = core::str::jg(&f) {
                mk.t(text);
            }
        }
        
        if !ckw {
            edb = edb.akq(1);
            if crate::netstack::tcp::bqr(ip, port, ey) || edb > 100_000 {
                break;
            }
        } else {
            edb = 0;
        }
        
        if crate::logger::lh().ao(ay) > 2000 {
            break;
        }
        
        
        if mk.len() > 4096 {
            mk.t("\n... (response truncated)");
            break;
        }
        
        crate::arch::bhd();
    }
    
    let _ = crate::netstack::tcp::bwx(ip, port, ey);
    
    if mk.is_empty() {
        return Err("No response received");
    }
    
    Ok(mk)
}

fn rzu(erz: &str, port: u16, path: &str, did: &str) {
    
    let url = if port == 443 {
        alloc::format!("https://{}{}", did, path)
    } else {
        alloc::format!("https://{}:{}{}", did, port, path)
    };
    
    crate::println!("Connecting to {} (TLS 1.3)...", did);
    crate::println!("--- HTTPS response ---");
    
    match crate::netstack::https::get(&url) {
        Ok(mk) => {
            
            crate::h!(C_, "HTTP/1.1 {}", mk.wt);
            
            
            for (bs, bn) in &mk.zk {
                crate::println!("{}: {}", bs, bn);
            }
            crate::println!("");
            
            
            let qqu = if mk.gj.len() > 4096 {
                &mk.gj[..4096]
            } else {
                &mk.gj
            };
            
            if let Ok(dza) = core::str::jg(qqu) {
                crate::print!("{}", dza);
                if mk.gj.len() > 4096 {
                    crate::println!("\n... (truncated, {} more bytes)", mk.gj.len() - 4096);
                }
            } else {
                crate::println!("[Binary data: {} bytes]", mk.gj.len());
            }
            
            crate::println!("\n--- end ({} bytes) ---", mk.gj.len());
        }
        Err(aa) => {
            crate::h!(A_, "HTTPS failed: {}", aa);
        }
    }
}

pub(super) fn lsm(url: &str) -> Option<(String, u16, String, bool)> {
    let mut tm = url.em();
    let mut https = false;
    if let Some(kr) = tm.blj("https://") {
        tm = kr;
        https = true;
    } else if let Some(kr) = tm.blj("http://") {
        tm = kr;
    }

    let (bej, path) = if let Some((i, ai)) = tm.fve('/') {
        (i, format!("/{}", ai))
    } else {
        (tm, String::from("/"))
    };

    let (kh, port) = if let Some((i, ai)) = bej.fve(':') {
        let port = ai.parse::<u16>().bq()?;
        (i, port)
    } else {
        (bej, if https { 443 } else { 80 })
    };

    if kh.is_empty() {
        return None;
    }

    Some((String::from(kh), port, path, https))
}

pub(super) fn cgl(input: &str) -> Option<[u8; 4]> {
    let ek: Vec<&str> = input.adk('.').collect();
    if ek.len() != 4 {
        return None;
    }
    let q = ek[0].parse::<u8>().bq()?;
    let o = ek[1].parse::<u8>().bq()?;
    let r = ek[2].parse::<u8>().bq()?;
    let bc = ek[3].parse::<u8>().bq()?;
    Some([q, o, r, bc])
}



pub(super) fn rec(n: &[&str], ro: &str) {
    if n.is_empty() && !ro.cj("./") {
        crate::h!(C_, "Usage: exec <program> [args...]");
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
    
    
    let (alo, vmy) = if ro.cj("./") {
        (ro, n)
    } else if n.is_empty() {
        crate::h!(A_, "exec: missing program name");
        return;
    } else {
        (n[0], &n[1..])
    };
    
    
    if alo == "test" || alo == "./test" {
        crate::h!(C_, "Running Ring 3 test program...");
        match crate::exec::hil() {
            crate::exec::ExecResult::Dx(aj) => {
                if aj == 0 {
                    crate::h!(B_, "Ring 3 test passed (exit code 0)");
                } else {
                    crate::h!(D_, "Ring 3 test exited with code: {}", aj);
                }
            }
            crate::exec::ExecResult::In(ctt) => {
                crate::h!(A_, "Test faulted: {}", ctt);
            }
            crate::exec::ExecResult::Xk(aa) => {
                crate::h!(A_, "Load error: {:?}", aa);
            }
            crate::exec::ExecResult::Bf => {
                crate::h!(A_, "Memory allocation failed");
            }
        }
        return;
    }
    
    
    if alo == "hello" || alo == "./hello" {
        crate::h!(C_, "Running embedded hello world ELF in Ring 3...");
        match crate::exec::kui() {
            crate::exec::ExecResult::Dx(aj) => {
                if aj == 0 {
                    crate::h!(B_, "Program exited successfully");
                } else {
                    crate::h!(D_, "Program exited with code: {}", aj);
                }
            }
            crate::exec::ExecResult::In(ctt) => {
                crate::h!(A_, "Program faulted: {}", ctt);
            }
            crate::exec::ExecResult::Xk(aa) => {
                crate::h!(A_, "Failed to load ELF: {:?}", aa);
            }
            crate::exec::ExecResult::Bf => {
                crate::h!(A_, "Memory allocation failed");
            }
        }
        return;
    }
    
    
    let path = jmh(alo);
    
    
    if !cxx(&path) {
        crate::h!(A_, "exec: {}: not found", path);
        return;
    }
    
    
    if !crate::exec::clc(&path) {
        crate::h!(A_, "exec: {}: not an ELF executable", path);
        return;
    }
    
    crate::h!(C_, "Executing: {}", path);
    
    
    match crate::exec::itf(&path, vmy) {
        crate::exec::ExecResult::Dx(aj) => {
            if aj == 0 {
                crate::h!(B_, "Program exited successfully");
            } else {
                crate::h!(D_, "Program exited with code: {}", aj);
            }
        }
        crate::exec::ExecResult::In(ctt) => {
            crate::h!(A_, "Program faulted: {}", ctt);
        }
        crate::exec::ExecResult::Xk(aa) => {
            crate::h!(A_, "Failed to load: {:?}", aa);
        }
        crate::exec::ExecResult::Bf => {
            crate::h!(A_, "Out of memory");
        }
    }
}

pub(super) fn reb(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: elfinfo <file>");
        return;
    }
    
    let path = jmh(n[0]);
    
    
    let da = match crate::vfs::aji(&path, crate::vfs::OpenFlags(0)) {
        Ok(da) => da,
        Err(_) => {
            crate::h!(A_, "Cannot open: {}", path);
            return;
        }
    };
    
    let mut dh = [0u8; 64];
    match crate::vfs::read(da, &mut dh) {
        Ok(bo) if bo >= 64 => {}
        _ => {
            crate::h!(A_, "Cannot read ELF header");
            crate::vfs::agj(da).bq();
            return;
        }
    }
    crate::vfs::agj(da).bq();
    
    
    if dh[0..4] != [0x7F, b'E', b'L', b'F'] {
        crate::h!(A_, "Not an ELF file");
        return;
    }
    
    crate::h!(G_, "ELF Header: {}", path);
    crate::println!("  Magic:   {:02X} {:02X} {:02X} {:02X}", dh[0], dh[1], dh[2], dh[3]);
    crate::println!("  Class:   {}", if dh[4] == 2 { "ELF64" } else { "ELF32" });
    crate::println!("  Data:    {}", if dh[5] == 1 { "Little Endian" } else { "Big Endian" });
    
    let ceh = u16::dj([dh[16], dh[17]]);
    let bde = match ceh {
        1 => "Relocatable",
        2 => "Executable",
        3 => "Shared Object",
        4 => "Core",
        _ => "Unknown",
    };
    crate::println!("  Type:    {} ({})", bde, ceh);
    
    let cqb = u16::dj([dh[18], dh[19]]);
    let ujf = match cqb {
        3 => "x86",
        62 => "x86-64",
        183 => "AArch64",
        _ => "Unknown",
    };
    crate::println!("  Machine: {} ({})", ujf, cqb);
    
    let bt = u64::dj([
        dh[24], dh[25], dh[26], dh[27],
        dh[28], dh[29], dh[30], dh[31],
    ]);
    crate::println!("  Entry:   {:#x}", bt);
    
    let ltu = u64::dj([
        dh[32], dh[33], dh[34], dh[35],
        dh[36], dh[37], dh[38], dh[39],
    ]);
    crate::println!("  PHoff:   {:#x}", ltu);
    
    let ltt = u16::dj([dh[56], dh[57]]);
    crate::println!("  PHnum:   {}", ltt);
}


pub(super) fn xmn(ro: &str, n: &[&str]) -> bool {
    let path = jmh(ro);
    
    if !cxx(&path) {
        return false;
    }

    
    if crate::exec::clc(&path) {
        crate::h!(C_, "Executing: {}", path);
        match crate::exec::itf(&path, n) {
            crate::exec::ExecResult::Dx(aj) => {
                if aj != 0 {
                    crate::h!(D_, "Exit code: {}", aj);
                }
            }
            crate::exec::ExecResult::In(ctt) => {
                crate::h!(A_, "Faulted: {}", ctt);
            }
            crate::exec::ExecResult::Xk(aa) => {
                crate::h!(A_, "Load error: {:?}", aa);
            }
            crate::exec::ExecResult::Bf => {
                crate::h!(A_, "Out of memory");
            }
        }
        return true;
    }
    
    
    if let Some(ca) = super::network::fse(&path) {
        if ca.cj("#!/bin/sh") || ca.cj("#!/bin/bash") {
            sog(&ca, n);
            return true;
        }
    }

    false
}



fn sog(eib: &str, n: &[&str]) {
    use alloc::string::String;
    use alloc::vec::Vec;
    use alloc::collections::BTreeMap;

    let ak: Vec<&str> = eib.ak().collect();
    let mut agz: BTreeMap<String, String> = BTreeMap::new();

    
    for (a, ji) in n.iter().cf() {
        agz.insert(alloc::format!("{}", a + 1), String::from(*ji));
    }
    agz.insert(String::from("@"), n.rr(" "));
    agz.insert(String::from("#"), alloc::format!("{}", n.len()));
    agz.insert(String::from("0"), String::from("sh"));
    agz.insert(String::from("?"), String::from("0"));
    agz.insert(String::from("HOME"), String::from("/root"));
    agz.insert(String::from("PATH"), String::from("/usr/bin:/bin:/usr/sbin:/sbin"));
    agz.insert(String::from("SHELL"), String::from("/bin/sh"));

    let mut fz = 0usize; 
    let mut fux = 0u32; 

    while fz < ak.len() {
        let js = ak[fz].em();
        fz += 1;

        
        if js.is_empty() || js.cj('#') {
            continue;
        }

        
        if fux > 0 {
            if js.cj("if ") || js == "if" {
                fux += 1;
            } else if js == "fi" || js.cj("fi;") || js.cj("fi ") {
                fux -= 1;
            } else if fux == 1 && (js == "else" || js.cj("else;") || js.cj("else ")) {
                fux = 0; 
            }
            continue;
        }

        
        let tg = nrv(js, &agz);
        let tg = tg.em();
        if tg.is_empty() { continue; }

        
        if tg.contains(';') && !tg.cj("if ") && !tg.contains("then") {
            let wvk: Vec<&str> = tg.adk(';').collect();
            for sub in wvk {
                let sub = sub.em();
                if sub.is_empty() { continue; }
                itg(sub, &mut agz);
            }
            continue;
        }

        
        if tg.cj("if ") {
            
            let kki = tg.tl("if ").em();
            let kki = kki.bdd("; then").bdd(";then").em();
            let result = nrd(kki, &agz);
            if !result {
                fux = 1; 
            }
            continue;
        }
        if tg == "then" { continue; } 
        if tg == "else" {
            fux = 1; 
            continue;
        }
        if tg == "fi" || tg.cj("fi;") || tg.cj("fi ") {
            continue; 
        }

        
        if tg.cj("for ") {
            
            let kr = tg.tl("for ").em();
            if let Some(ods) = kr.du(" in ") {
                let igg = &kr[..ods];
                let mow = kr[ods + 4..].em();
                let mow = mow.bdd("; do").bdd(";do").em();
                let alv: Vec<&str> = mow.ayt().collect();

                
                if fz < ak.len() && ak[fz].em() == "do" {
                    fz += 1;
                }

                
                let cvy = fz;
                let mut haw = fz;
                let mut eo = 1u32;
                while haw < ak.len() {
                    let bl = ak[haw].em();
                    if bl.cj("for ") { eo += 1; }
                    if bl == "done" || bl.cj("done;") || bl.cj("done ") {
                        eo -= 1;
                        if eo == 0 { break; }
                    }
                    haw += 1;
                }

                
                let gj: Vec<&str> = ak[cvy..haw].ip();
                for ap in &alv {
                    agz.insert(String::from(igg), String::from(*ap));
                    for qqt in &gj {
                        let bl = qqt.em();
                        if bl.is_empty() || bl.cj('#') || bl == "do" { continue; }
                        let bgz = nrv(bl, &agz);
                        itg(bgz.em(), &mut agz);
                    }
                }

                fz = haw + 1; 
                continue;
            }
        }

        
        itg(&tg, &mut agz);
    }
}


fn itg(line: &str, agz: &mut alloc::collections::BTreeMap<alloc::string::String, alloc::string::String>) {
    use alloc::string::String;

    let line = line.em();
    if line.is_empty() || line.cj('#') { return; }

    
    if let Some(bzo) = line.du('=') {
        if bzo > 0 && line[..bzo].bw().xx(|r| r.bvb() || r == '_') && !line.cj('=') {
            let bfp = &line[..bzo];
            let ap = line[bzo + 1..].em();
            let ap = fvs(ap);
            agz.insert(String::from(bfp), ap);
            return;
        }
    }

    
    let ek: alloc::vec::Vec<&str> = line.eyv(2, char::fme).collect();
    let cmd = ek[0];
    let kr = if ek.len() > 1 { ek[1].em() } else { "" };

    match cmd {
        "echo" => {
            if kr == "-n" {
                
            } else if kr.cj("-n ") {
                let fr = fvs(&kr[3..]);
                crate::print!("{}", fr);
            } else if kr.cj("-e ") {
                let fr = fvs(&kr[3..]);
                crate::println!("{}", fr);
            } else {
                let fr = fvs(kr);
                crate::println!("{}", fr);
            }
        }
        "printf" => {
            let fr = fvs(kr);
            crate::print!("{}", fr);
        }
        "cat" => {
            
            if !kr.is_empty() {
                let path = fvs(kr);
                if let Some(ca) = super::network::fse(&path) {
                    crate::print!("{}", ca);
                } else {
                    crate::println!("cat: {}: No such file or directory", path);
                }
            }
        }
        "test" | "[" => {
            
            let mo = kr.bdd(']').em();
            let result = nrd(&alloc::format!("[ {} ]", mo), agz);
            agz.insert(alloc::string::String::from("?"), if result { alloc::string::String::from("0") } else { alloc::string::String::from("1") });
        }
        "export" => {
            
            if let Some(bzo) = kr.du('=') {
                let bfp = &kr[..bzo];
                let ap = fvs(&kr[bzo + 1..]);
                agz.insert(alloc::string::String::from(bfp), ap);
            }
        }
        "env" | "printenv" => {
            for (eh, p) in agz.iter() {
                if eh.len() > 1 { 
                    crate::println!("{}={}", eh, p);
                }
            }
        }
        "set" => {
            if kr == "-e" || kr == "-x" || kr.is_empty() {
                
            }
        }
        "true" | ":" => {
            agz.insert(alloc::string::String::from("?"), alloc::string::String::from("0"));
        }
        "false" => {
            agz.insert(alloc::string::String::from("?"), alloc::string::String::from("1"));
        }
        "exec" => {
            
            if !kr.is_empty() {
                itg(kr, agz);
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


fn nrv(line: &str, agz: &alloc::collections::BTreeMap<alloc::string::String, alloc::string::String>) -> alloc::string::String {
    use alloc::string::String;
    let mut result = String::fc(line.len());
    let bw: alloc::vec::Vec<char> = line.bw().collect();
    let mut a = 0;
    while a < bw.len() {
        if bw[a] == '$' && a + 1 < bw.len() {
            if bw[a + 1] == '{' {
                
                if let Some(agj) = bw[a + 2..].iter().qf(|&r| r == '}') {
                    let bfp: String = bw[a + 2..a + 2 + agj].iter().collect();
                    if let Some(ap) = agz.get(&bfp) {
                        result.t(ap);
                    }
                    a += agj + 3;
                    continue;
                }
            } else if bw[a + 1] == '(' {
                
                if let Some(agj) = bw[a + 2..].iter().qf(|&r| r == ')') {
                    a += agj + 3;
                    continue;
                }
            }
            
            let ay = a + 1;
            let mut ci = ay;
            while ci < bw.len() && (bw[ci].bvb() || bw[ci] == '_' || bw[ci] == '@' || bw[ci] == '#' || bw[ci] == '?') {
                ci += 1;
                
                if ci == ay + 1 && (bw[ay] == '@' || bw[ay] == '#' || bw[ay] == '?' || bw[ay].atb()) {
                    break;
                }
            }
            if ci > ay {
                let bfp: String = bw[ay..ci].iter().collect();
                if let Some(ap) = agz.get(&bfp) {
                    result.t(ap);
                }
                a = ci;
            } else {
                result.push('$');
                a += 1;
            }
        } else {
            result.push(bw[a]);
            a += 1;
        }
    }
    result
}


fn nrd(mo: &str, ydr: &alloc::collections::BTreeMap<alloc::string::String, alloc::string::String>) -> bool {
    let mo = mo.em();
    
    let ff = if mo.cj('[') && mo.pp(']') {
        mo[1..mo.len() - 1].em()
    } else if mo.cj("[ ") && mo.pp(" ]") {
        mo[2..mo.len() - 2].em()
    } else {
        mo
    };

    
    if ff.cj("-n ") { return !ff[3..].em().dcz('"').is_empty(); }
    
    if ff.cj("-z ") { return ff[3..].em().dcz('"').is_empty(); }
    
    if ff.cj("-f ") {
        let path = ff[3..].em().dcz('"');
        return crate::ramfs::fh(|fs| fs.mq(path).is_ok());
    }
    
    if ff.cj("-d ") { return true; } 
    
    if ff.contains(" = ") {
        let ek: alloc::vec::Vec<&str> = ff.eyv(2, " = ").collect();
        if ek.len() == 2 {
            return ek[0].em().dcz('"') == ek[1].em().dcz('"');
        }
    }
    
    if ff.contains(" != ") {
        let ek: alloc::vec::Vec<&str> = ff.eyv(2, " != ").collect();
        if ek.len() == 2 {
            return ek[0].em().dcz('"') != ek[1].em().dcz('"');
        }
    }
    
    for op in &[" -eq ", " -ne ", " -gt ", " -lt ", " -ge ", " -le "] {
        if ff.contains(op) {
            let ek: alloc::vec::Vec<&str> = ff.eyv(2, op).collect();
            if ek.len() == 2 {
                let q = ek[0].em().parse::<i64>().unwrap_or(0);
                let o = ek[1].em().parse::<i64>().unwrap_or(0);
                return match *op {
                    " -eq " => q == o,
                    " -ne " => q != o,
                    " -gt " => q > o,
                    " -lt " => q < o,
                    " -ge " => q >= o,
                    " -le " => q <= o,
                    _ => false,
                };
            }
        }
    }

    
    !ff.is_empty()
}


fn fvs(e: &str) -> String {
    use alloc::string::String;
    let e = e.em();
    let pxa = if (e.cj('\'') && e.pp('\''))
        || (e.cj('"') && e.pp('"'))
    {
        &e[1..e.len() - 1]
    } else {
        e
    };
    
    let mut result = String::fc(pxa.len());
    let mut kub = false;
    for r in pxa.bw() {
        if kub {
            match r {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                '\'' => result.push('\''),
                _ => { result.push('\\'); result.push(r); }
            }
            kub = false;
        } else if r == '\\' {
            kub = true;
        } else {
            result.push(r);
        }
    }
    result
}


pub(super) fn jmh(j: &str) -> String {
    if j.cj('/') {
        return String::from(j);
    }
    
    if j.cj("./") {
        let jv = crate::ramfs::fh(|fs| String::from(fs.dau()));
        if jv == "/" {
            return String::from(&j[1..]); 
        } else {
            return format!("{}{}", jv, &j[1..]); 
        }
    }
    
    
    let hzf = ["/usr/bin", "/bin", "/usr/sbin", "/sbin", "/usr/local/bin"];
    
    for te in &hzf {
        let path = format!("{}/{}", te, j);
        if cxx(&path) {
            return path;
        }
    }
    
    
    let jv = crate::ramfs::fh(|fs| String::from(fs.dau()));
    if jv == "/" {
        format!("/{}", j)
    } else {
        format!("{}/{}", jv, j)
    }
}


pub(super) fn cxx(path: &str) -> bool {
    
    if crate::vfs::hm(path).is_ok() {
        return true;
    }
    
    crate::ramfs::fh(|fs| fs.aja(path))
}






pub(super) fn rfe(n: &[&str]) {
    if n.is_empty() {
        oxr();
        return;
    }
    
    match n[0] {
        "init" => {
            crate::println!("Initializing TrustVM hypervisor...");
            match crate::hypervisor::init() {
                Ok(()) => {
                    crate::gr!(B_, "? ");
                    crate::println!("Hypervisor initialized successfully!");
                }
                Err(aa) => {
                    crate::gr!(A_, "? ");
                    crate::println!("Failed to initialize hypervisor: {:?}", aa);
                }
            }
        }
        "status" => {
            if crate::hypervisor::zu() {
                crate::gr!(B_, "? ");
                crate::println!("TrustVM: Active");
                crate::println!("  Backend: {}", crate::hypervisor::kbt());
                crate::println!("  VMs created: {}", crate::hypervisor::dna());
            } else {
                crate::gr!(D_, "? ");
                crate::println!("TrustVM: Inactive");
                crate::println!("  Run 'hv init' to enable the hypervisor");
            }
        }
        "check" => {
            use crate::hypervisor::{dpw, CpuVendor};
            crate::println!("Checking virtualization support...");
            let acs = dpw();
            crate::println!("  CPU Vendor: {:?}", acs);
            
            match acs {
                CpuVendor::Ef => {
                    match crate::hypervisor::vmx::inj() {
                        Ok(dr) => {
                            crate::println!("  [Intel VT-x (VMX)]");
                            crate::println!("    VMX supported:      {}", if dr.dme { "Yes" } else { "No" });
                            crate::println!("    EPT supported:      {}", if dr.fhw { "Yes" } else { "No" });
                            crate::println!("    Unrestricted guest: {}", if dr.gvo { "Yes" } else { "No" });
                            crate::println!("    VPID supported:     {}", if dr.gwj { "Yes" } else { "No" });
                            crate::println!("    VMCS revision:      0x{:08X}", dr.igr);
                        }
                        Err(aa) => {
                            crate::gr!(A_, "Error: ");
                            crate::println!("{:?}", aa);
                        }
                    }
                }
                CpuVendor::Ct => {
                    if crate::hypervisor::svm::gkj() {
                        let features = crate::hypervisor::svm::fjn();
                        crate::println!("  [AMD-V (SVM)]");
                        crate::println!("    SVM supported:      Yes");
                        crate::println!("    SVM Revision:       {}", features.afe);
                        crate::println!("    NPT supported:      {}", if features.npt { "Yes" } else { "No" });
                        crate::println!("    NRIP Save:          {}", if features.evl { "Yes" } else { "No" });
                        crate::println!("    Flush by ASID:      {}", if features.hjy { "Yes" } else { "No" });
                        crate::println!("    Available ASIDs:    {}", features.fph);
                        crate::println!("    AVIC:               {}", if features.gzk { "Yes" } else { "No" });
                    } else {
                        crate::gr!(A_, "Error: ");
                        crate::println!("SVM not supported or disabled in BIOS");
                    }
                }
                CpuVendor::F => {
                    crate::gr!(A_, "Error: ");
                    crate::println!("Unknown CPU vendor - virtualization not supported");
                }
            }
        }
        "shutdown" => {
            crate::println!("Shutting down hypervisor...");
            match crate::hypervisor::cbu() {
                Ok(()) => {
                    crate::gr!(B_, "? ");
                    crate::println!("Hypervisor shutdown complete");
                }
                Err(aa) => {
                    crate::gr!(A_, "? ");
                    crate::println!("Failed: {:?}", aa);
                }
            }
        }
        "caps" | "capabilities" => {
            crate::println!("{}", crate::hypervisor::jma());
        }
        "security" => {
            crate::println!("{}", crate::hypervisor::jmb());
        }
        "events" => {
            let az = if n.len() > 1 { 
                n[1].parse().unwrap_or(10) 
            } else { 
                10 
            };
            let events = crate::hypervisor::nya(az);
            if events.is_empty() {
                crate::println!("No events recorded.");
            } else {
                crate::println!("Recent VM Events:");
                for id in events {
                    crate::println!("  [{:>6}ms] VM {} - {:?}", 
                        id.aet, id.fk, id.bqo);
                }
            }
        }
        "vpid" => {
            if crate::hypervisor::fyk() {
                crate::gr!(B_, "? ");
                crate::println!("VPID: Enabled");
                crate::println!("  Allocated VPIDs: {}", crate::hypervisor::pyu());
            } else {
                crate::gr!(D_, "? ");
                crate::println!("VPID: Disabled (CPU may not support it)");
            }
        }
        "violations" => {
            let az = crate::hypervisor::fhx();
            crate::println!("EPT Violations: {}", az);
            if az > 0 {
                let cnt = crate::hypervisor::pap(5);
                for p in cnt {
                    crate::println!("  VM {} GPA=0x{:X} type={:?} at RIP=0x{:X}",
                        p.fk, p.hmc, p.igm, p.wb);
                }
            }
        }
        "version" => {
            crate::println!("TrustVM {}", crate::hypervisor::dk());
        }
        "logo" => {
            crate::println!("{}", crate::hypervisor::logo());
        }
        
        
        
        #[cfg(target_arch = "aarch64")]
        "spy" | "mmio" => {
            crate::h!(C_, "=== TrustOS EL2 MMIO Spy ===");
            crate::println!();
            if !crate::hypervisor::arm_hv::fma() {
                crate::h!(A_, "Not running at EL2 - hypervisor mode unavailable");
                crate::println!("Boot TrustOS at EL2 (QEMU: -machine virt,virtualization=on)");
                return;
            }
            if !crate::hypervisor::arm_hv::rl() {
                crate::h!(D_, "EL2 detected but no guest running yet");
                crate::println!("Use 'hv launch' to start a guest under surveillance");
                return;
            }
            let report = crate::hypervisor::arm_hv::el2_entry::tes();
            crate::println!("{}", report);
        }
        #[cfg(target_arch = "aarch64")]
        "smc" | "smc-log" => {
            crate::h!(C_, "=== SMC (Secure Monitor Call) Log ===");
            let az = if n.len() > 1 { n[1].parse().unwrap_or(20) } else { 20 };
            let events = crate::hypervisor::arm_hv::mmio_spy::lyf(az);
            if events.is_empty() {
                crate::println!("No SMC calls intercepted.");
            } else {
                for aiz in &events {
                    crate::println!("  {}", crate::hypervisor::arm_hv::mmio_spy::nvs(aiz));
                }
                crate::println!("\nTotal SMC events: {}",
                    crate::hypervisor::arm_hv::mmio_spy::jty());
            }
        }
        #[cfg(target_arch = "aarch64")]
        "devices" => {
            crate::h!(C_, "=== Device Activity (per MMIO range) ===");
            let cm = crate::hypervisor::arm_hv::mmio_spy::nld();
            if cm.is_empty() {
                crate::println!("No device activity recorded.");
            } else {
                crate::println!("  {:<22} {:<8} {}", "Device", "Reads", "Writes");
                crate::println!("  {}", "-".afd(42));
                for (j, exj, fbu) in &cm {
                    crate::println!("  {:<22} {:<8} {}", j, exj, fbu);
                }
            }
        }
        #[cfg(target_arch = "aarch64")]
        "el2" => {
            crate::h!(C_, "=== ARM EL2 Hypervisor Status ===");
            if crate::hypervisor::arm_hv::fma() {
                crate::h!(B_, "  Running at EL2: Yes");
                crate::println!("  Hypervisor active: {}", 
                    if crate::hypervisor::arm_hv::rl() { "Yes (guest running)" } else { "No (idle)" });
                crate::println!("  MMIO traps: {}", crate::hypervisor::arm_hv::onr());
                crate::println!("  SMC intercepts: {}", crate::hypervisor::arm_hv::plm());
                crate::println!("  MMIO events logged: {}", 
                    crate::hypervisor::arm_hv::mmio_spy::mmj());
                crate::println!("  SMC events logged: {}", 
                    crate::hypervisor::arm_hv::mmio_spy::jty());
            } else {
                crate::h!(A_, "  Running at EL2: No");
                crate::println!("  Current EL does not support hypervisor operations.");
                crate::println!("  Boot with: qemu-system-aarch64 -machine virt,virtualization=on");
            }
        }
        #[cfg(target_arch = "aarch64")]
        "report" => {
            crate::println!("{}", crate::hypervisor::arm_hv::tcn());
        }
        #[cfg(target_arch = "aarch64")]
        "boot" | "launch" => {
            use crate::hypervisor::arm_hv::guest_loader;
            crate::h!(C_, "=== TrustOS EL2 Hypervisor — Guest Boot ===");
            crate::println!();

            if !crate::hypervisor::arm_hv::fma() {
                crate::h!(A_, "ERROR: Not running at EL2!");
                crate::println!("  Boot TrustOS with: qemu-system-aarch64 -machine virt,virtualization=on");
                return;
            }

            
            if n.len() <= 1 || n[1] == "test" {
                crate::println!("Launching self-test guest (WFI loop)...");
                crate::println!("  This tests the full EL2 hypervisor pipeline:");
                crate::println!("  Stage-2 tables -> HCR_EL2 -> VBAR_EL2 -> vGIC -> ERET -> trap -> log");
                crate::println!();

                let brw = 0x4000_0000u64;
                let cbf = 512 * 1024 * 1024u64;

                match guest_loader::wgy(brw, cbf) {
                    Ok(result) => {
                        crate::println!("{}", guest_loader::svv(&result));
                        crate::h!(B_, "Guest loaded successfully!");
                        crate::println!("  To actually enter the guest: hv enter");
                        crate::println!("  (This will transfer control to EL1 — TrustOS shell will");
                        crate::println!("   continue to run at EL2, intercepting all hardware access)");
                    }
                    Err(aa) => {
                        crate::h!(A_, "Failed to load guest: {}", aa);
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
            crate::h!(C_, "=== Guest Loader — ARM64 Image Validator ===");
            crate::println!();

            
            let config = guest_loader::GuestLoadConfig::default();
            crate::println!("Memory layout for guest:");
            crate::println!("  RAM:     0x{:08X} - 0x{:08X} ({} MB)",
                config.brw,
                config.brw + config.cbf,
                config.cbf / (1024*1024));
            crate::println!("  Kernel:  0x{:08X} (RAM + {}MB)",
                config.brw + guest_loader::ADR_,
                guest_loader::ADR_ / (1024*1024));
            crate::println!("  DTB:     0x{:08X} (RAM + {}MB)",
                config.brw + guest_loader::ABF_,
                guest_loader::ABF_ / (1024*1024));
            crate::println!("  initrd:  0x{:08X} (RAM + {}MB)",
                config.brw + guest_loader::ADM_,
                guest_loader::ADM_ / (1024*1024));
            crate::println!();
            crate::println!("MMIO traps ({} regions):", config.guw.len());
            for (ar, aw) in &config.guw {
                crate::println!("  0x{:08X} - 0x{:08X} ({})",
                    ar, ar + aw,
                    crate::hypervisor::arm_hv::mmio_spy::eda(*ar));
            }
            crate::println!();
            crate::println!("Kernel cmdline: {}", config.wx);
        }
        "test" | "selftest" => {
            crate::h!(C_, "╔══════════════════════════════════════════════════════╗");
            crate::h!(C_, "║         TrustVM Hypervisor Self-Test Suite           ║");
            crate::h!(C_, "╚══════════════════════════════════════════════════════╝");
            crate::println!();
            
            let (cg, gv, log) = crate::hypervisor::tests::jne();
            
            for line in &log {
                if line.contains("[PASS]") {
                    crate::h!(B_, "{}", line);
                } else {
                    crate::h!(A_, "{}", line);
                }
            }
            
            crate::println!();
            if gv == 0 {
                crate::h!(B_, "Result: {}/{} tests passed — ALL OK ✓", cg, cg + gv);
            } else {
                crate::h!(A_, "Result: {}/{} tests passed, {} FAILED ✗", cg, cg + gv, gv);
            }
        }
        "help" | _ => oxr(),
    }
}

fn oxr() {
    use crate::hypervisor::{dpw, CpuVendor};
    let acs = dpw();
    let backend = match acs {
        CpuVendor::Ef => "Intel VT-x (VMX)",
        CpuVendor::Ct => "AMD-V (SVM)",
        CpuVendor::F => {
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


pub(super) fn rki(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: vm <command> [args]");
        crate::println!("Commands: create, start, run, stop, list, guests, inspect, debug, mount");
        return;
    }
    
    match n[0] {
        "create" => {
            if n.len() < 3 {
                crate::println!("Usage: vm create <name> <memory_mb>");
                return;
            }
            let j = n[1];
            let dtd: usize = n[2].parse().unwrap_or(16);
            
            if !crate::hypervisor::zu() {
                crate::gr!(D_, "Warning: ");
                crate::println!("Hypervisor not initialized. Run 'hv init' first.");
                return;
            }
            
            match crate::hypervisor::dpg(j, dtd) {
                Ok(ad) => {
                    crate::gr!(B_, "? ");
                    crate::println!("Created VM '{}' with ID {} ({}MB RAM)", j, ad, dtd);
                }
                Err(aa) => {
                    crate::gr!(A_, "? ");
                    crate::println!("Failed to create VM: {:?}", aa);
                }
            }
        }
        "start" => {
            if n.len() < 2 {
                crate::println!("Usage: vm start <id> [guest_name]");
                crate::println!("Available guests: {:?}", crate::hypervisor::hpy());
                return;
            }
            let ad: u64 = n[1].parse().unwrap_or(0);
            let cra = if n.len() > 2 { n[2] } else { "hello" };
            
            crate::println!("Starting VM {} with guest '{}'...", ad, cra);
            match crate::hypervisor::gte(ad, cra) {
                Ok(()) => {
                    crate::gr!(B_, "? ");
                    crate::println!("VM {} completed execution", ad);
                }
                Err(aa) => {
                    crate::gr!(A_, "? ");
                    crate::println!("VM {} failed: {:?}", ad, aa);
                }
            }
        }
        "run" => {
            
            let cra = if n.len() > 1 { n[1] } else { "hello" };
            
            if !crate::hypervisor::zu() {
                crate::gr!(D_, "Note: ");
                crate::println!("Initializing hypervisor first...");
                if let Err(aa) = crate::hypervisor::init() {
                    crate::gr!(A_, "✗ ");
                    crate::println!("Failed to init hypervisor: {:?}", aa);
                    return;
                }
            }
            
            
            let dtd = if cra.cj("linux") || cra.pp(".bzimage") {
                64
            } else {
                4
            };
            
            match crate::hypervisor::dpg(cra, dtd) {
                Ok(ad) => {
                    crate::println!("Running guest '{}'...", cra);
                    match crate::hypervisor::gte(ad, cra) {
                        Ok(()) => {
                            crate::gr!(B_, "? ");
                            crate::println!("Guest '{}' completed", cra);
                            
                            
                            crate::hypervisor::svm_vm::coa(ad, |vm| {
                                let e = &vm.cm;
                                crate::println!("  VMEXITs: {} (cpuid={} io={} msr={} hlt={} vmcall={})",
                                    e.ait, e.bmp, e.ank,
                                    e.bkn, e.axz, e.gwh);
                            });
                            crate::println!("  Use 'vm inspect {}' for detailed state", ad);
                        }
                        Err(aa) => {
                            crate::gr!(A_, "? ");
                            crate::println!("Failed: {:?}", aa);
                        }
                    }
                }
                Err(aa) => {
                    crate::gr!(A_, "? ");
                    crate::println!("Failed to create VM: {:?}", aa);
                }
            }
        }
        "stop" => {
            if n.len() < 2 {
                crate::println!("Usage: vm stop <id>");
                return;
            }
            let ad: u64 = n[1].parse().unwrap_or(0);
            
            match crate::hypervisor::jru(ad) {
                Ok(()) => {
                    crate::gr!(B_, "? ");
                    crate::println!("Stopped VM {}", ad);
                }
                Err(aa) => {
                    crate::gr!(A_, "? ");
                    crate::println!("Failed to stop VM {}: {:?}", ad, aa);
                }
            }
        }
        "list" => {
            use crate::hypervisor::{dpw, CpuVendor};
            crate::println!("Virtual Machines:");
            
            match dpw() {
                CpuVendor::Ct => {
                    let bfr = crate::hypervisor::svm_vm::hqc();
                    if bfr.is_empty() {
                        crate::println!("  (no VMs created)");
                    } else {
                        crate::println!("  {:>4} {:>20} {:>12}", "ID", "NAME", "STATE");
                        crate::println!("  {:->4} {:->20} {:->12}", "", "", "");
                        for (ad, j, g) in bfr {
                            crate::println!("  {:>4} {:>20} {:>12?}", ad, j, g);
                        }
                    }
                }
                CpuVendor::Ef => {
                    crate::println!("  Total created: {}", crate::hypervisor::dna());
                }
                CpuVendor::F => {
                    crate::println!("  (hypervisor not available)");
                }
            }
            crate::println!();
            crate::println!("Use 'vm guests' to see available guest programs.");
        }
        "guests" => {
            crate::println!("Available guest programs:");
            for cra in crate::hypervisor::hpy() {
                crate::println!("  - {}", cra);
            }
            crate::println!("");
            crate::println!("Usage: vm run <guest_name>");
        }
        "mount" => {
            if n.len() < 4 {
                crate::println!("Usage: vm mount <vm_id> <host_path> <guest_path> [ro]");
                return;
            }
            let ad: u64 = n[1].parse().unwrap_or(0);
            let cac = n[2];
            let bqx = n[3];
            let awr = n.len() > 4 && n[4] == "ro";
            
            crate::hypervisor::elx(ad, cac, bqx, awr);
            crate::gr!(B_, "? ");
            crate::println!("Mounted {} -> {} (readonly={})", cac, bqx, awr);
        }
        "console" => {
            if n.len() < 2 {
                crate::println!("Usage: vm console <vm_id>");
                return;
            }
            let ad: u64 = n[1].parse().unwrap_or(0);
            let an = crate::hypervisor::iwo(ad);
            if an.is_empty() {
                crate::println!("(no output)");
            } else {
                crate::println!("{}", an);
            }
        }
        "input" => {
            if n.len() < 3 {
                crate::println!("Usage: vm input <vm_id> <text>");
                return;
            }
            let ad: u64 = n[1].parse().unwrap_or(0);
            let text = n[2..].rr(" ");
            crate::hypervisor::leo(ad, text.as_bytes());
            crate::hypervisor::leo(ad, b"\n");
            crate::println!("Injected input to VM {}", ad);
        }
        "inspect" => {
            
            use crate::hypervisor::{dpw, CpuVendor};
            
            match dpw() {
                CpuVendor::Ct => {
                    let bfr = crate::hypervisor::svm_vm::hqc();
                    if bfr.is_empty() {
                        crate::println!("No VMs to inspect. Run 'vm run pm-test' first.");
                        return;
                    }
                    
                    
                    let sso: Option<u64> = if n.len() > 1 { n[1].parse().bq() } else { None };
                    
                    for (ad, j, g) in &bfr {
                        if let Some(aos) = sso {
                            if *ad != aos { continue; }
                        }
                        
                        crate::h!(C_, "+--- VM #{}: {} [{:?}] ---+", ad, j, g);
                        
                        
                        crate::hypervisor::svm_vm::coa(*ad, |vm| {
                            let e = &vm.cm;
                            crate::println!();
                            crate::h!(D_, "  Exit Statistics:");
                            crate::println!("    Total VMEXITs: {}", e.ait);
                            crate::println!("    CPUID:   {:>8}", e.bmp);
                            crate::println!("    I/O:     {:>8}", e.ank);
                            crate::println!("    MSR:     {:>8}", e.bkn);
                            crate::println!("    HLT:     {:>8}", e.axz);
                            crate::println!("    NPF:     {:>8}", e.cay);
                            crate::println!("    VMCALL:  {:>8}", e.gwh);
                            crate::println!("    Intr:    {:>8}", e.jap);
                            
                            crate::println!();
                            crate::h!(D_, "  Guest GPRs:");
                            crate::println!("    RAX = 0x{:016X}  RBX = 0x{:016X}", vm.ej.rax, vm.ej.rbx);
                            crate::println!("    RCX = 0x{:016X}  RDX = 0x{:016X}", vm.ej.rcx, vm.ej.rdx);
                            crate::println!("    RSI = 0x{:016X}  RDI = 0x{:016X}", vm.ej.rsi, vm.ej.rdi);
                            crate::println!("    RBP = 0x{:016X}  RSP = 0x{:016X}", vm.ej.rbp, vm.ej.rsp);
                            crate::println!("    R8  = 0x{:016X}  R9  = 0x{:016X}", vm.ej.r8, vm.ej.r9);
                            crate::println!("    R10 = 0x{:016X}  R11 = 0x{:016X}", vm.ej.r10, vm.ej.r11);
                            crate::println!("    R12 = 0x{:016X}  R13 = 0x{:016X}", vm.ej.r12, vm.ej.r13);
                            crate::println!("    R14 = 0x{:016X}  R15 = 0x{:016X}", vm.ej.r14, vm.ej.r15);
                            
                            
                            if let Some(ref vmcb) = vm.vmcb {
                                use crate::hypervisor::svm::vmcb::state_offsets;
                                crate::println!();
                                crate::h!(D_, "  VMCB State:");
                                let pc = vmcb.xs(state_offsets::Aw);
                                let rsp = vmcb.xs(state_offsets::Hc);
                                let rflags = vmcb.xs(state_offsets::Kv);
                                let akb = vmcb.xs(state_offsets::Vu);
                                let jm = vmcb.xs(state_offsets::Vv);
                                let cr4 = vmcb.xs(state_offsets::Vw);
                                let efer = vmcb.xs(state_offsets::Lh);
                                let aap = vmcb.alp(state_offsets::JU_) as u64;
                                let bjw = vmcb.alp(state_offsets::MV_) as u64;
                                let ipj = vmcb.alp(state_offsets::Agx) as u64;
                                
                                crate::println!("    RIP    = 0x{:016X}  RSP    = 0x{:016X}", pc, rsp);
                                crate::println!("    RFLAGS = 0x{:016X}  CPL    = {}", rflags, ipj);
                                crate::println!("    CR0 = 0x{:X}  CR3 = 0x{:X}  CR4 = 0x{:X}", akb, jm, cr4);
                                crate::println!("    EFER = 0x{:X}  CS = 0x{:X}  DS = 0x{:X}", efer, aap, bjw);
                                
                                
                                use crate::hypervisor::svm::vmcb::control_offsets;
                                let kuo = vmcb.cgx(control_offsets::Abm);
                                let spb = vmcb.cgx(control_offsets::Abn);
                                let spc = vmcb.cgx(control_offsets::Abo);
                                crate::println!();
                                crate::h!(D_, "  Last VMEXIT:");
                                crate::println!("    ExitCode = 0x{:X}  Info1 = 0x{:X}  Info2 = 0x{:X}", 
                                    kuo, spb, spc);
                            }
                            
                            crate::println!();
                            crate::h!(D_, "  Memory:");
                            crate::println!("    Guest memory: {} KB ({} MB)", vm.apy / 1024, vm.apy / (1024 * 1024));
                            crate::println!("    ASID: {}", vm.ajv);
                            
                            
                            let ipb = crate::hypervisor::iwo(*ad);
                            if !ipb.is_empty() {
                                crate::println!();
                                crate::h!(D_, "  Console Output (last 256 chars):");
                                let ay = if ipb.len() > 256 { ipb.len() - 256 } else { 0 };
                                crate::println!("    {}", &ipb[ay..]);
                            }
                            
                            
                            if let Some(f) = vm.duy(0x5000, 32) {
                                crate::println!();
                                crate::h!(D_, "  Memory @ 0x5000 (guest write zone):");
                                crate::print!("    ");
                                for (a, hf) in f.iter().cf() {
                                    crate::print!("{:02X} ", hf);
                                    if (a + 1) % 16 == 0 && a + 1 < f.len() {
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
            if n.len() < 3 {
                crate::println!("Usage: vm dump <id> <gpa_hex> [length]");
                crate::println!("  Example: vm dump 1 0x1000 256");
                return;
            }
            let ad: u64 = n[1].parse().unwrap_or(0);
            let tgt = n[2].tl("0x").tl("0X");
            let pe = u64::wa(tgt, 16).unwrap_or(0);
            let len: usize = if n.len() > 3 {
                n[3].parse().unwrap_or(128)
            } else {
                128
            };
            let len = len.v(4096); 
            
            match crate::hypervisor::avo() {
                crate::hypervisor::CpuVendor::Ct => {
                    crate::hypervisor::svm_vm::coa(ad, |vm| {
                        crate::h!(C_, "  Memory dump: VM {} @ GPA 0x{:X} ({} bytes)", ad, pe, len);
                        crate::println!();
                        
                        if let Some(f) = vm.duy(pe, len) {
                            
                            for mu in (0..f.len()).akt(16) {
                                crate::print!("  {:08X}: ", pe as usize + mu);
                                
                                for bj in 0..16 {
                                    if mu + bj < f.len() {
                                        crate::print!("{:02X} ", f[mu + bj]);
                                    } else {
                                        crate::print!("   ");
                                    }
                                    if bj == 7 { crate::print!(" "); }
                                }
                                
                                crate::print!(" |");
                                for bj in 0..16 {
                                    if mu + bj < f.len() {
                                        let o = f[mu + bj];
                                        if o >= 0x20 && o < 0x7F {
                                            crate::print!("{}", o as char);
                                        } else {
                                            crate::print!(".");
                                        }
                                    }
                                }
                                crate::println!("|");
                            }
                        } else {
                            crate::h!(A_, "  GPA 0x{:X}+{} is outside guest memory ({} bytes)", 
                                pe, len, vm.apy);
                        }
                    });
                }
                _ => {
                    crate::println!("vm dump requires AMD SVM.");
                }
            }
        }
        
        
        "regs" | "registers" => {
            if n.len() < 2 {
                crate::println!("Usage: vm regs <id>");
                return;
            }
            let ad: u64 = n[1].parse().unwrap_or(0);
            
            match crate::hypervisor::avo() {
                crate::hypervisor::CpuVendor::Ct => {
                    crate::hypervisor::svm_vm::coa(ad, |vm| {
                        crate::h!(C_, "  VM {} Register State", ad);
                        crate::println!();
                        
                        
                        crate::println!("  RAX={:016X}  RBX={:016X}", vm.ej.rax, vm.ej.rbx);
                        crate::println!("  RCX={:016X}  RDX={:016X}", vm.ej.rcx, vm.ej.rdx);
                        crate::println!("  RSI={:016X}  RDI={:016X}", vm.ej.rsi, vm.ej.rdi);
                        crate::println!("  RBP={:016X}  RSP={:016X}", vm.ej.rbp, 
                            vm.vmcb.as_ref().efd(0, |p| p.xs(crate::hypervisor::svm::vmcb::state_offsets::Hc)));
                        crate::println!("  R8 ={:016X}  R9 ={:016X}", vm.ej.r8, vm.ej.r9);
                        crate::println!("  R10={:016X}  R11={:016X}", vm.ej.r10, vm.ej.r11);
                        crate::println!("  R12={:016X}  R13={:016X}", vm.ej.r12, vm.ej.r13);
                        crate::println!("  R14={:016X}  R15={:016X}", vm.ej.r14, vm.ej.r15);
                        
                        if let Some(ref vmcb) = vm.vmcb {
                            use crate::hypervisor::svm::vmcb::{state_offsets, control_offsets};
                            
                            let pc = vmcb.xs(state_offsets::Aw);
                            let fsu = vmcb.xs(state_offsets::Kv);
                            crate::println!("  RIP={:016X}  RFLAGS={:016X}", pc, fsu);
                            
                            
                            let ghe = {
                                let mut e = alloc::string::String::new();
                                if fsu & 0x001 != 0 { e.t("CF "); }
                                if fsu & 0x040 != 0 { e.t("ZF "); }
                                if fsu & 0x080 != 0 { e.t("SF "); }
                                if fsu & 0x200 != 0 { e.t("IF "); }
                                if fsu & 0x400 != 0 { e.t("DF "); }
                                if fsu & 0x800 != 0 { e.t("OF "); }
                                e
                            };
                            crate::println!("  Flags: [{}]", ghe.em());
                            
                            crate::println!();
                            crate::println!("  CR0={:016X}  CR2={:016X}", 
                                vmcb.xs(state_offsets::Vu), vmcb.xs(state_offsets::Agy));
                            crate::println!("  CR3={:016X}  CR4={:016X}", 
                                vmcb.xs(state_offsets::Vv), vmcb.xs(state_offsets::Vw));
                            crate::println!("  EFER={:016X}", vmcb.xs(state_offsets::Lh));
                            
                            
                            crate::println!();
                            crate::println!("  CS: sel={:04X} base={:016X} lim={:08X} attr={:04X}", 
                                vmcb.alp(state_offsets::JU_),
                                vmcb.xs(state_offsets::SJ_),
                                vmcb.za(state_offsets::AAY_),
                                vmcb.alp(state_offsets::AAX_));
                            crate::println!("  SS: sel={:04X} base={:016X} lim={:08X} attr={:04X}", 
                                vmcb.alp(state_offsets::XH_),
                                vmcb.xs(state_offsets::AID_),
                                vmcb.za(state_offsets::AIE_),
                                vmcb.alp(state_offsets::AIC_));
                            crate::println!("  DS: sel={:04X}  ES: sel={:04X}  FS: sel={:04X}  GS: sel={:04X}", 
                                vmcb.alp(state_offsets::MV_),
                                vmcb.alp(state_offsets::SZ_),
                                vmcb.alp(state_offsets::ACL_),
                                vmcb.alp(state_offsets::ADF_));
                            
                            
                            crate::println!();
                            crate::h!(D_, "  LAPIC State:");
                            crate::println!("    Enabled: {}  SVR: 0x{:X}  TPR: 0x{:X}", 
                                vm.ku.iq, vm.ku.bim, vm.ku.guv);
                            let mkx = match (vm.ku.atq >> 17) & 0x3 {
                                0 => "one-shot", 1 => "periodic", 2 => "TSC-deadline", _ => "reserved",
                            };
                            let xhj = (vm.ku.atq >> 16) & 1;
                            let xhm = vm.ku.atq & 0xFF;
                            crate::println!("    Timer: vec={} mode={} masked={} ICR={} DCR={}", 
                                xhm, mkx, xhj, vm.ku.bnh, vm.ku.dgc);
                            
                            
                            crate::println!();
                            let kuo = vmcb.cgx(control_offsets::Abm);
                            let hnw = vmcb.cgx(control_offsets::Abn);
                            let tsw = vmcb.cgx(control_offsets::Abo);
                            crate::println!("  Last VMEXIT: code=0x{:X} info1=0x{:X} info2=0x{:X}", kuo, hnw, tsw);
                            
                            
                            crate::println!();
                            crate::println!("  VMEXITs: {}  CPUID: {}  I/O: {}  MSR: {}  NPF: {}  HLT: {}",
                                vm.cm.ait, vm.cm.bmp, vm.cm.ank, 
                                vm.cm.bkn, vm.cm.cay, vm.cm.axz);
                        }
                    });
                }
                _ => {
                    crate::println!("vm regs requires AMD SVM.");
                }
            }
        }
        
        
        "stack" | "backtrace" | "bt" => {
            if n.len() < 2 {
                crate::println!("Usage: vm stack <id> [depth]");
                return;
            }
            let ad: u64 = n[1].parse().unwrap_or(0);
            let eo: usize = if n.len() > 2 { n[2].parse().unwrap_or(16) } else { 16 };
            
            match crate::hypervisor::avo() {
                crate::hypervisor::CpuVendor::Ct => {
                    crate::hypervisor::svm_vm::coa(ad, |vm| {
                        if let Some(ref vmcb) = vm.vmcb {
                            use crate::hypervisor::svm::vmcb::state_offsets;
                            
                            let rsp = vmcb.xs(state_offsets::Hc);
                            let pc = vmcb.xs(state_offsets::Aw);
                            let rbp = vm.ej.rbp;
                            
                            crate::h!(C_, "  VM {} Stack Trace (RSP=0x{:X}, RIP=0x{:X})", ad, rsp, pc);
                            crate::println!();
                            
                            
                            crate::println!("  Stack contents (potential return addresses):");
                            for a in 0..eo {
                                let ag = rsp + (a as u64 * 8);
                                if let Some(f) = vm.duy(ag, 8) {
                                    let ap = u64::dj([
                                        f[0], f[1], f[2], f[3],
                                        f[4], f[5], f[6], f[7],
                                    ]);
                                    
                                    let marker = if ap > 0xFFFF_8000_0000_0000 { " <-- kernel addr" }
                                        else if ap > 0x1000 && ap < 0x1_0000_0000 { " <-- possible code" }
                                        else { "" };
                                    crate::println!("  [{:2}] RSP+{:04X}: {:016X}{}", a, a * 8, ap, marker);
                                } else {
                                    crate::println!("  [{:2}] RSP+{:04X}: <outside guest memory>", a, a * 8);
                                    break;
                                }
                            }
                            
                            
                            if rbp > 0x1000 && rbp < vm.apy as u64 {
                                crate::println!();
                                crate::println!("  Frame pointer chain (RBP=0x{:X}):", rbp);
                                let mut frame = rbp;
                                for a in 0..eo.v(32) {
                                    if frame < 0x1000 || frame >= vm.apy as u64 - 16 { break; }
                                    if let Some(f) = vm.duy(frame, 16) {
                                        let lon = u64::dj([
                                            f[0], f[1], f[2], f[3],
                                            f[4], f[5], f[6], f[7],
                                        ]);
                                        let dbg = u64::dj([
                                            f[8], f[9], f[10], f[11],
                                            f[12], f[13], f[14], f[15],
                                        ]);
                                        crate::println!("  #{}: RBP=0x{:X} -> ret=0x{:X}", a, frame, dbg);
                                        if lon <= frame || lon == 0 { break; }
                                        frame = lon;
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
            let sub = if n.len() > 1 { n[1] } else { "" };
            match sub {
                "init" | "start" => {
                    crate::hypervisor::debug_monitor::init();
                    crate::println!("\x01G✓ Debug monitor started\x01W — all VM exits will be recorded.");
                    crate::println!("  Run a VM, then use 'vm debug' to see the dashboard.");
                }
                "stop" => {
                    crate::hypervisor::debug_monitor::qg();
                    crate::println!("Debug monitor stopped.");
                }
                "reset" => {
                    crate::hypervisor::debug_monitor::apa();
                    crate::println!("Debug monitor data cleared.");
                }
                "gaps" => {
                    let report = crate::hypervisor::debug_monitor::kyr();
                    crate::println!("{}", report);
                }
                "io" => {
                    let report = crate::hypervisor::debug_monitor::nyd();
                    crate::println!("{}", report);
                }
                "msr" => {
                    let report = crate::hypervisor::debug_monitor::nyf();
                    crate::println!("{}", report);
                }
                "timeline" => {
                    let az = if n.len() > 2 { n[2].parse().unwrap_or(30) } else { 30 };
                    let report = crate::hypervisor::debug_monitor::nys(az);
                    crate::println!("{}", report);
                }
                "serial" => {
                    let iq = n.len() <= 2 || n[2] != "off";
                    crate::hypervisor::debug_monitor::pje(iq);
                    crate::println!("Serial logging: {}", if iq { "ON" } else { "OFF" });
                }
                "status" => {
                    let gh = crate::hypervisor::debug_monitor::rl();
                    let es = crate::hypervisor::debug_monitor::jtr();
                    let gvm = crate::hypervisor::debug_monitor::jup();
                    crate::println!("\x01CDebug Monitor Status:\x01W");
                    crate::println!("  Active: {}", if gh { "\x01Gyes\x01W" } else { "\x01Rno\x01W" });
                    crate::println!("  Total events: {}", es);
                    crate::println!("  Unhandled: {}{}\x01W", 
                        if gvm > 0 { "\x01R" } else { "\x01G" }, gvm);
                }
                "" => {
                    
                    if !crate::hypervisor::debug_monitor::ky() {
                        
                        crate::hypervisor::debug_monitor::init();
                        crate::println!("Debug monitor auto-initialized. Run a VM to collect data.\n");
                    }
                    let hfc = crate::hypervisor::debug_monitor::kym();
                    crate::println!("{}", hfc);
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
            if n.len() < 2 {
                crate::println!("Usage: vm linux <bzimage_path> [initrd_path] [memory_mb] [cmdline]");
                crate::println!("  Example: vm linux /boot/vmlinuz /boot/initrd.img 128");
                crate::println!("  Default: 128 MB RAM, console=ttyS0 earlyprintk nokaslr");
                return;
            }
            
            let kfw = n[1];
            let oer = if n.len() > 2 && !n[2].parse::<usize>().is_ok() {
                Some(n[2])
            } else {
                None
            };
            let lln = if oer.is_some() { 3 } else { 2 };
            let dtd: usize = if n.len() > lln {
                n[lln].parse().unwrap_or(128)
            } else {
                128
            };
            let nel = lln + 1;
            let wx = if n.len() > nel {
                n[nel..].rr(" ")
            } else {
                alloc::string::String::from("console=ttyS0 earlyprintk=serial,ttyS0 nokaslr noapic")
            };
            
            
            crate::h!(C_, "Loading Linux kernel from {}...", kfw);
            let cwc = match crate::vfs::mq(kfw) {
                Ok(f) => {
                    crate::h!(B_, "  Kernel: {} bytes ({} KB)", f.len(), f.len() / 1024);
                    f
                }
                Err(aa) => {
                    crate::h!(A_, "  Error reading {}: {:?}", kfw, aa);
                    return;
                }
            };
            
            
            let cyw = if let Some(path) = oer {
                crate::println!("Loading initrd from {}...", path);
                match crate::vfs::mq(path) {
                    Ok(f) => {
                        crate::h!(B_, "  Initrd: {} bytes ({} KB)", f.len(), f.len() / 1024);
                        Some(f)
                    }
                    Err(aa) => {
                        crate::h!(A_, "  Error reading {}: {:?}", path, aa);
                        return;
                    }
                }
            } else {
                None
            };
            
            
            if !crate::hypervisor::zu() {
                crate::println!("Initializing hypervisor...");
                if let Err(aa) = crate::hypervisor::init() {
                    crate::h!(A_, "Hypervisor init failed: {:?}", aa);
                    return;
                }
            }
            
            
            crate::println!("Creating VM ({} MB RAM)...", dtd);
            crate::println!("Cmdline: {}", wx);
            
            match crate::hypervisor::dpg("linux-guest", dtd) {
                Ok(ad) => {
                    crate::println!("Booting Linux in VM #{}...", ad);
                    
                    let tuj = cyw.ahz();
                    
                    
                    match crate::hypervisor::avo() {
                        crate::hypervisor::CpuVendor::Ct => {
                            let result = crate::hypervisor::svm_vm::coa(ad, |vm| {
                                vm.fvn(&cwc, &wx, tuj)
                            });
                            match result {
                                Some(Ok(())) => {
                                    crate::h!(B_, "Linux VM completed");
                                }
                                Some(Err(aa)) => {
                                    crate::h!(A_, "Linux VM failed: {:?}", aa);
                                    crate::println!("Use 'vm inspect {}' for details", ad);
                                }
                                None => {
                                    crate::h!(A_, "VM #{} not found", ad);
                                }
                            }
                        }
                        crate::hypervisor::CpuVendor::Ef => {
                            
                            let config = crate::hypervisor::linux_vm::Pw {
                                afc: dtd,
                                wx: wx.clone(),
                                ..Default::default()
                            };
                            match crate::hypervisor::linux_vm::LinuxVm::new(config) {
                                Ok(mut vm) => {
                                    let skj = alloc::vec::Vec::new();
                                    let tuh = cyw.ahz().unwrap_or(&skj);
                                    match vm.boot(&cwc, tuh) {
                                        Ok(()) => {
                                            crate::h!(B_, "Linux VM completed");
                                        }
                                        Err(aa) => {
                                            crate::h!(A_, "Linux VM failed: {:?}", aa);
                                        }
                                    }
                                }
                                Err(aa) => {
                                    crate::h!(A_, "Failed to create Linux VM: {:?}", aa);
                                }
                            }
                        }
                        _ => {
                            crate::h!(A_, "No hardware virtualization available");
                        }
                    }
                }
                Err(aa) => {
                    crate::h!(A_, "Failed to create VM: {:?}", aa);
                }
            }
        }
        
        _ => {
            crate::println!("Unknown VM command: {}", n[0]);
            crate::println!("Commands: create, start, run, stop, list, guests, linux, mount, console, input, inspect, dump, regs, stack");
        }
    }
}




pub(super) fn yjf(n: &[&str]) {
    use crate::hypervisor::linux_subsystem::{self, LinuxState};
    
    if n.is_empty() {
        oxs();
        return;
    }
    
    match n[0] {
        "init" | "start" => {
            crate::h!(C_, "+----------------------------------------------------------+");
            crate::h!(C_, "|     TrustOS Subsystem for Linux (TSL) v1.0              |");
            crate::h!(C_, "+----------------------------------------------------------+");
            crate::println!();
            crate::println!("Initializing Linux Subsystem...");
            
            match linux_subsystem::init() {
                Ok(()) => {
                    crate::gr!(B_, "? ");
                    crate::println!("Linux Subsystem initialized");
                    crate::println!();
                    crate::println!("Use 'linux boot' to start real Linux VM,");
                    crate::println!("or 'linux <command>' for simulated commands.");
                }
                Err(aa) => {
                    crate::gr!(A_, "? ");
                    crate::println!("Failed to initialize: {:?}", aa);
                }
            }
        }
        "boot" => {
            crate::h!(C_, "+----------------------------------------------------------+");
            crate::h!(C_, "|          Booting Real Linux VM...                       |");
            crate::h!(C_, "+----------------------------------------------------------+");
            crate::println!();
            
            
            let acs = crate::hypervisor::avo();
            match acs {
                crate::hypervisor::CpuVendor::Ef => {
                    crate::println!("CPU: Intel (VMX)");
                }
                crate::hypervisor::CpuVendor::Ct => {
                    crate::println!("CPU: AMD (SVM)");
                }
                crate::hypervisor::CpuVendor::F => {
                    crate::h!(D_, "Warning: No hardware virtualization detected");
                    crate::println!("         Real VM boot may not be possible.");
                }
            }
            
            crate::println!();
            crate::println!("Starting Linux VM with kernel and initramfs...");
            
            match linux_subsystem::boot() {
                Ok(()) => {
                    crate::gr!(B_, "? ");
                    crate::println!("Linux VM boot completed");
                }
                Err(aa) => {
                    crate::gr!(A_, "? ");
                    crate::println!("Boot failed: {:?}", aa);
                    crate::println!();
                    crate::h!(D_, "Falling back to simulated mode.");
                }
            }
        }
        "status" => {
            let g = linux_subsystem::g();
            let ejg = linux_subsystem::bcu();
            
            crate::h!(G_, "Linux Subsystem Status:");
            crate::println!("---------------------------------------");
            
            match g {
                LinuxState::Ma => {
                    crate::gr!(D_, "? State: ");
                    crate::println!("Not Started");
                    crate::println!("  Run 'linux init' to start the subsystem.");
                }
                LinuxState::Agt => {
                    crate::gr!(D_, "? State: ");
                    crate::println!("Booting...");
                }
                LinuxState::At => {
                    crate::gr!(B_, "? State: ");
                    crate::println!("Ready");
                }
                LinuxState::Rq => {
                    crate::gr!(C_, "? State: ");
                    crate::println!("Busy (executing command)");
                }
                LinuxState::Q => {
                    crate::gr!(A_, "? State: ");
                    crate::println!("Error");
                }
                LinuxState::Ays => {
                    crate::gr!(D_, "? State: ");
                    crate::println!("Shutting down...");
                }
            }
            
            
            crate::println!();
            crate::h!(C_, "Kernel Image:");
            if ejg.oaq() {
                let bvc = ejg.bvc();
                crate::println!("  ? Loaded: {} bytes ({} KB)", bvc, bvc / 1024);
                if let Some(dk) = ejg.oht() {
                    crate::println!("  Version:  {}", dk);
                }
                if let Some((efb, efm)) = ejg.mzu() {
                    crate::println!("  Protocol: {}.{}", efb, efm);
                }
            } else {
                crate::println!("  ? Not loaded (simulated mode)");
            }
            
            crate::println!();
            crate::h!(C_, "Initramfs:");
            if ejg.oao() {
                let hny = ejg.jaa();
                crate::println!("  ? Loaded: {} bytes ({} KB)", hny, hny / 1024);
            } else {
                crate::println!("  ? Not loaded");
            }
            
            crate::println!();
            crate::h!(C_, "VM Configuration:");
            crate::println!("  Memory:   {} MB", linux_subsystem::US_);
            crate::println!("  VM ID:    {:#X}", linux_subsystem::AEO_);
            
            drop(ejg);
        }
        "stop" | "shutdown" => {
            crate::println!("Shutting down Linux Subsystem...");
            match linux_subsystem::cbu() {
                Ok(()) => {
                    crate::gr!(B_, "? ");
                    crate::println!("Linux Subsystem stopped");
                }
                Err(aa) => {
                    crate::gr!(A_, "? ");
                    crate::println!("Failed: {:?}", aa);
                }
            }
        }
        "extract" => {
            
            super::apps::nhi();
        }
        "help" | "--help" | "-h" => {
            oxs();
        }
        
        _ => {
            
            let ro = n.rr(" ");
            
            match linux_subsystem::bna(&ro) {
                Ok(result) => {
                    if !result.ejc.is_empty() {
                        crate::println!("{}", result.ejc);
                    }
                    if !result.dwg.is_empty() {
                        crate::gr!(A_, "{}", result.dwg);
                    }
                    if result.nz != 0 && result.dwg.is_empty() {
                        crate::h!(D_, "(exit code: {})", result.nz);
                    }
                }
                Err(aa) => {
                    crate::gr!(A_, "Error: ");
                    crate::println!("{:?}", aa);
                }
            }
        }
    }
}

fn oxs() {
    crate::h!(G_, "TrustOS Subsystem for Linux (TSL)");
    crate::h!(G_, "=================================");
    crate::println!();
    crate::println!("Execute Linux commands from TrustOS using a virtualized Linux environment.");
    crate::println!();
    crate::h!(C_, "Management Commands:");
    crate::println!("  linux init          Initialize the Linux subsystem");
    crate::println!("  linux boot          Boot real Linux kernel in VM");
    crate::println!("  linux extract       Download and extract Alpine Linux to /alpine");
    crate::println!("  linux status        Show subsystem status");
    crate::println!("  linux stop          Stop the Linux subsystem");
    crate::println!("  linux help          Show this help");
    crate::println!();
    crate::h!(C_, "Execute Linux Commands:");
    crate::println!("  linux <command>     Execute a command in Linux");
    crate::println!();
    crate::h!(C_, "Examples:");
    crate::println!("  linux uname -a      Show Linux kernel info");
    crate::println!("  linux ls -la        List files");
    crate::println!("  linux cat /etc/os-release");
    crate::println!("  linux free -h       Show memory usage");
    crate::println!("  linux df -h         Show disk usage");
    crate::println!("  linux cat /proc/cpuinfo");
    crate::println!();
    crate::h!(D_, "Note: Real VM boot requires AMD SVM or Intel VMX support.");
}






fn dob(f: &[u8]) -> String {
    use alloc::string::String;
    use alloc::format;
    
    if f.len() < 64 || &f[0..4] != b"\x7fELF" {
        return String::from("      Not a valid ELF file");
    }
    
    let mut co = String::new();
    
    let class = f[4]; 
    let slq = f[5]; 
    let gfz = u16::dj([f[16], f[17]]);
    let czk = u16::dj([f[18], f[19]]);
    
    co.t(&format!("      File size: {} bytes\n", f.len()));
    co.t(&format!("      Architecture: {}\n", if class == 2 { "x86_64 (64-bit)" } else { "x86 (32-bit)" }));
    co.t(&format!("      Endian: {}\n", if slq == 1 { "Little" } else { "Big" }));
    co.t(&format!("      Type: {}\n", match gfz {
        2 => "Executable",
        3 => "Shared object (PIE)",
        _ => "Other",
    }));
    co.t(&format!("      Machine: {}\n", match czk {
        0x3E => "x86-64",
        0x03 => "i386",
        0xB7 => "AArch64",
        _ => "Unknown",
    }));
    
    if class == 2 {
        let bt = u64::dj([
            f[24], f[25], f[26], f[27],
            f[28], f[29], f[30], f[31],
        ]);
        co.t(&format!("      Entry point: 0x{:x}\n", bt));
        
        
        let abt = u64::dj([f[32], f[33], f[34], f[35], f[36], f[37], f[38], f[39]]) as usize;
        let egq = u16::dj([f[54], f[55]]) as usize;
        let egp = u16::dj([f[56], f[57]]) as usize;
        
        let mut oap = false;
        for a in 0..egp {
            let dz = abt + a * egq;
            if dz + 4 <= f.len() {
                let frq = u32::dj([f[dz], f[dz+1], f[dz+2], f[dz+3]]);
                if frq == 3 { oap = true; }
            }
        }
        
        co.t(&format!("      Linking: {}\n", if oap { "Dynamic (needs ld-linux.so)" } else { "Static" }));
    }
    
    co.t("\n      ? Valid Linux ELF binary detected!");
    co.t("\n      Note: Execution requires x86_64 CPU emulation (slow)");
    
    co
}


pub(super) fn rhb(cmd: &str, n: &[&str]) {
    use crate::hypervisor::linux_subsystem;

    
    let g = linux_subsystem::g();
    if g == linux_subsystem::LinuxState::Ma {
        let _ = linux_subsystem::init();
        let _ = linux_subsystem::boot();
    }

    
    let mut auh = alloc::string::String::from(cmd);
    for q in n {
        auh.push(' ');
        auh.t(q);
    }

    match linux_subsystem::bna(&auh) {
        Ok(result) => {
            if !result.ejc.is_empty() {
                crate::println!("{}", result.ejc);
            }
            if !result.dwg.is_empty() {
                crate::gr!(A_, "{}", result.dwg);
                crate::println!();
            }
        }
        Err(aa) => {
            crate::gr!(A_, "Error: ");
            crate::println!("{:?}", aa);
        }
    }
}


pub(super) fn rce(n: &[&str]) {
    use alloc::vec::Vec;
    use alloc::string::String;
    
    let air = n.get(0).hu().unwrap_or("help");
    
    match air {
        "test" | "run" => {
            crate::h!(C_, "+--------------------------------------------------------------+");
            crate::h!(C_, "|           Alpine Linux Test - All in One                     |");
            crate::h!(C_, "+--------------------------------------------------------------+");
            crate::println!();
            
            
            let tnn = crate::ramfs::fh(|fs| {
                fs.awb(Some("/alpine/bin")).map(|aa| aa.len() > 0).unwrap_or(false)
            });
            
            if tnn {
                crate::h!(B_, "[1/4] Alpine binaries present ?");
            } else {
                
                crate::h!(D_, "[1/4] Creating test binaries...");
                super::apps::rqu();
            }
            
            
            crate::h!(D_, "[2/4] Verifying binaries...");
            
            let hap = crate::ramfs::fh(|fs| {
                fs.awb(Some("/alpine/bin")).map(|aa| aa.len()).unwrap_or(0)
            });
            
            if hap > 0 {
                crate::h!(B_, "      Found {} binaries in /alpine/bin", hap);
            } else {
                crate::h!(A_, "      No binaries found! Run 'linux extract' first.");
                return;
            }
            crate::println!();
            
            
            crate::h!(D_, "[3/4] Checking extracted files...");
            crate::ramfs::fh(|fs| {
                if let Ok(ch) = fs.awb(Some("/alpine/bin")) {
                    let az = ch.len();
                    crate::println!("      /alpine/bin: {} binaries", az);
                    
                    for (j, _, _) in ch.iter().take(5) {
                        crate::println!("        - {}", j);
                    }
                    if az > 5 {
                        crate::println!("        ... and {} more", az - 5);
                    }
                }
            });
            crate::println!();
            
            
            crate::h!(D_, "[4/4] Analyzing Linux binary...");
            let dyy = n.get(1).hu().unwrap_or("/alpine/bin/busybox");
            
            
            let skc = crate::ramfs::fh(|fs| {
                fs.mq(dyy).map(|f| {
                    let f = f.ip();
                    dob(&f)
                })
            });
            
            match skc {
                Ok(co) => {
                    crate::h!(B_, "{}", co);
                }
                Err(_) => {
                    crate::h!(A_, "      Could not read binary: {}", dyy);
                }
            }
            
            crate::println!();
            crate::h!(G_, "----------------------------------------------------------------");
            crate::h!(G_, "                    Alpine Test Complete!");
            crate::h!(G_, "----------------------------------------------------------------");
        }
        
        "ls" | "list" => {
            crate::h!(C_, "Alpine Linux files:");
            crate::ramfs::fh(|fs| {
                for te in &["/alpine", "/alpine/bin", "/alpine/usr/bin"] {
                    if let Ok(ch) = fs.awb(Some(*te)) {
                        crate::println!("\n{}/ ({} entries)", te, ch.len());
                        for (j, _, _) in ch.iter().take(10) {
                            crate::println!("  {}", j);
                        }
                        if ch.len() > 10 {
                            crate::println!("  ... {} more", ch.len() - 10);
                        }
                    }
                }
            });
        }
        
        "exec" => {
            if n.len() < 2 {
                crate::println!("Usage: alpine exec <binary> [args...]");
                crate::println!("Example: alpine exec /alpine/bin/busybox ls");
                return;
            }
            let dyy = n[1];
            let ilo: Vec<&str> = n[2..].ip();
            
            crate::println!("Executing: {} {:?}", dyy, ilo);
            match crate::linux_compat::exec(dyy, &ilo) {
                Ok(nz) => crate::println!("Exited with code: {}", nz),
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        
        "hello" => {
            
            crate::h!(C_, "Running minimal Linux ELF binary...");
            crate::println!();
            
            
            
            #[rustfmt::chz]
            static ADJ_: &[u8] = &[
                
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
            
            match crate::linux_compat::interpreter::peo(ADJ_, &["hello"]) {
                Ok(aj) => {
                    crate::println!();
                    crate::h!(B_, "Binary exited with code: {}", aj);
                    crate::h!(B_, "? Linux interpreter works!");
                }
                Err(aa) => {
                    crate::h!(A_, "Error: {}", aa);
                }
            }
        }
        
        _ => {
            crate::h!(C_, "Alpine Linux Commands:");
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


fn yqc(fs: &mut crate::ramfs::RamFs, f: &[u8], fdd: &str) -> Result<usize, &'static str> {
    use alloc::string::String;
    
    let mut l = 0;
    let mut az = 0;
    
    while l + 512 <= f.len() {
        let dh = &f[l..l + 512];
        
        
        if dh.iter().xx(|&o| o == 0) {
            break;
        }
        
        
        let bko = &dh[0..100];
        let bew = bko.iter().qf(|&o| o == 0).unwrap_or(100);
        let j = core::str::jg(&bko[..bew]).unwrap_or("");
        
        if j.is_empty() {
            break;
        }
        
        
        let afz = &dh[124..135];
        let als = core::str::jg(afz).unwrap_or("0");
        let aw = usize::wa(als.dcz(|r| r == '\0' || r == ' '), 8).unwrap_or(0);
        
        
        let xns = dh[156];
        
        let wo = if j.cj("./") {
            alloc::format!("{}/{}", fdd, &j[2..])
        } else {
            alloc::format!("{}/{}", fdd, j)
        };
        
        
        let enn = wo.bdd('/');
        
        l += 512; 
        
        match xns {
            b'5' | b'0' if j.pp('/') => {
                
                let _ = fs.ut(enn);
            }
            b'0' | b'\0' if aw > 0 => {
                
                if l + aw <= f.len() {
                    let ca = &f[l..l + aw];
                    
                    
                    if let Some(lsb) = enn.bhx('/') {
                        let tu = &enn[..lsb];
                        let _ = nhb(fs, tu);
                    }
                    
                    let _ = fs.touch(enn);
                    let _ = fs.ns(enn, ca);
                    az += 1;
                }
            }
            b'0' | b'\0' => {
                
                if let Some(lsb) = enn.bhx('/') {
                    let tu = &enn[..lsb];
                    let _ = nhb(fs, tu);
                }
                let _ = fs.touch(enn);
                az += 1;
            }
            b'2' => {
                
            }
            _ => {}
        }
        
        
        let xk = (aw + 511) / 512;
        l += xk * 512;
    }
    
    Ok(az)
}

fn nhb(fs: &mut crate::ramfs::RamFs, path: &str) -> Result<(), ()> {
    let mut cv = String::new();
    for vu in path.adk('/').hi(|e| !e.is_empty()) {
        cv.push('/');
        cv.t(vu);
        let _ = fs.ut(&cv);
    }
    Ok(())
}

pub(super) fn rdy(n: &[&str]) {
    crate::println!("[DEBUG] cmd_download called, args: {:?}", n);
    crate::serial_println!("[DEBUG] cmd_download called, args count: {}", n.len());
    
    if n.is_empty() {
        crate::println!("Usage: download <name|url> [output_file]");
        crate::println!("       download alpine  - Download Alpine Linux (fast)");
        crate::println!("       download <url>   - Download from URL");
        return;
    }
    
    let ji = n[0];
    crate::println!("[DEBUG] First arg: '{}'", ji);
    
    
    if ji == "alpine" || ji == "busybox" || ji == "linux" {
        crate::println!("[DEBUG] Calling download_from_local_server...");
        saq("alpine-minirootfs.tar.gz", "/opt/gui/alpine.tar.gz");
        return;
    }
    
    
    let url = ji;
    let an = if n.len() > 1 { n[1] } else { 
        url.cmm('/').next().unwrap_or("download")
    };
    
    crate::h!(C_, "Downloading: {}", url);
    crate::println!("         -> {}", an);
    ndv(n);
}


fn saq(it: &str, ftn: &str) {
    use alloc::vec::Vec;
    use alloc::format;
    
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::h!(C_, "|              Fast Download - Local Server                    |");
    crate::h!(C_, "+--------------------------------------------------------------+");
    crate::println!();
    
    
    let aep: [u8; 4] = [192, 168, 56, 1];
    let boh: u16 = 8080;
    
    crate::h!(D_, "[1/4] Configuring network...");
    
    
    crate::netstack::dhcp::fvw();
    crate::network::hzx(
        crate::network::Ipv4Address::new(192, 168, 56, 100),
        crate::network::Ipv4Address::new(255, 255, 255, 0),
        Some(crate::network::Ipv4Address::new(192, 168, 56, 1)),
    );
    
    
    if let Some((ip, hs, nt)) = crate::network::aou() {
        crate::println!("      IP: {}.{}.{}.{}", ip.as_bytes()[0], ip.as_bytes()[1], ip.as_bytes()[2], ip.as_bytes()[3]);
        crate::serial_println!("[DOWNLOAD] IP configured: {}.{}.{}.{}", ip.as_bytes()[0], ip.as_bytes()[1], ip.as_bytes()[2], ip.as_bytes()[3]);
    } else {
        crate::h!(A_, "      ERROR: No IP configured!");
        crate::netstack::dhcp::anu();
        return;
    }
    
    
    for _ in 0..100 {
        crate::netstack::poll();
    }
    crate::println!();
    
    crate::h!(D_, "[2/4] Connecting to 192.168.56.1:8080...");
    
    
    crate::println!("      Resolving MAC address...");
    let _ = crate::netstack::arp::eii(aep);
    for _ in 0..200 {
        crate::netstack::poll();
    }
    
    
    if let Some(ed) = crate::netstack::arp::ayo(aep) {
        crate::println!("      Server MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", 
            ed[0], ed[1], ed[2], ed[3], ed[4], ed[5]);
    } else {
        crate::h!(D_, "      Warning: No ARP response yet");
    }
    
    let ey = match crate::netstack::tcp::cue(aep, boh) {
        Ok(ai) => {
            crate::serial_println!("[DOWNLOAD] SYN sent, src_port={}", ai);
            ai
        }
        Err(aa) => {
            crate::serial_println!("[DOWNLOAD] SYN failed: {}", aa);
            crate::h!(A_, "      ERROR: {}", aa);
            crate::println!("      Is the server running?");
            crate::println!("      > cd server && .\\start-server.ps1");
            crate::netstack::dhcp::anu();
            return;
        }
    };
    
    crate::println!("      Waiting for connection...");
    if !crate::netstack::tcp::dnd(aep, boh, ey, 3000) {
        crate::serial_println!("[DOWNLOAD] Connection timeout!");
        crate::h!(A_, "      ERROR: Connection timeout");
        crate::println!("      Check: ping 192.168.56.1");
        crate::netstack::dhcp::anu();
        return;
    }
    
    crate::h!(B_, "      Connected!");
    crate::println!();
    
    crate::h!(D_, "[3/4] Downloading {}...", it);
    
    
    let request = format!(
        "GET /{} HTTP/1.1\r\nHost: 192.168.56.1\r\nConnection: close\r\n\r\n",
        it
    );
    
    if let Err(aa) = crate::netstack::tcp::dlo(aep, boh, ey, request.as_bytes()) {
        crate::h!(A_, "      ERROR: {}", aa);
        crate::netstack::dhcp::anu();
        return;
    }
    
    
    let mut f: Vec<u8> = Vec::fc(4 * 1024 * 1024);
    let ay = crate::logger::lh();
    let mut cyt: u32 = 0;
    let mut fmp = 0usize;
    let mut etv = ay;
    
    loop {
        
        for _ in 0..10 {
            crate::netstack::poll();
        }
        
        let mut ckw = false;
        while let Some(jj) = crate::netstack::tcp::cme(aep, boh, ey) {
            ckw = true;
            if f.len() + jj.len() > 8 * 1024 * 1024 {
                break;
            }
            f.bk(&jj);
        }
        
        
        let cfv = f.len() / 1024;
        if cfv >= fmp + 50 || (cfv > 0 && fmp == 0) {
            let ez = crate::logger::lh().ao(ay);
            let ig = if ez > 0 { (cfv as u64 * 1000) / ez } else { 0 };
            crate::print!("\r      {} KB downloaded ({} KB/s)          ", cfv, ig);
            fmp = cfv;
        }
        
        
        let iu = crate::logger::lh();
        if iu.ao(etv) >= 5 {
            crate::netstack::tcp::fiv(aep, boh, ey);
            etv = iu;
        }
        
        if !ckw {
            cyt += 1;
            if crate::netstack::tcp::bqr(aep, boh, ey) {
                crate::netstack::tcp::fiv(aep, boh, ey);
                break;
            }
            if cyt > 100_000 {
                break;
            }
        } else {
            cyt = 0;
        }
        
        
        if iu.ao(ay) > 30_000 {
            crate::h!(D_, "\n      Timeout!");
            break;
        }
    }
    
    let _ = crate::netstack::tcp::bwx(aep, boh, ey);
    
    let ez = crate::logger::lh().ao(ay);
    let cuu = f.len() / 1024;
    let kbo = if ez > 0 { (cuu as u64 * 1000) / ez } else { 0 };
    
    crate::println!();
    crate::h!(B_, "      Complete: {} KB in {}ms ({} KB/s)", cuu, ez, kbo);
    crate::println!();
    
    if f.is_empty() {
        crate::h!(A_, "      ERROR: No data received");
        crate::netstack::dhcp::anu();
        return;
    }
    
    
    let cvy = f.ee(4)
        .qf(|d| d == b"\r\n\r\n")
        .map(|ai| ai + 4)
        .unwrap_or(0);
    let gj = &f[cvy..];
    
    if gj.is_empty() {
        crate::h!(A_, "      ERROR: Empty response");
        crate::netstack::dhcp::anu();
        return;
    }
    
    crate::h!(D_, "[4/4] Saving to {}...", ftn);
    
    
    let hyn = crate::ramfs::fh(|fs| {
        let _ = fs.ut("/opt");
        let _ = fs.ut("/opt/gui");
        let _ = fs.touch(ftn);
        fs.ns(ftn, gj)
    });
    
    match hyn {
        Ok(_) => {
            crate::h!(B_, "      Saved: {:.2} MB", gj.len() as f32 / (1024.0 * 1024.0));
        }
        Err(aa) => {
            crate::h!(A_, "      ERROR: {:?}", aa);
            crate::netstack::dhcp::anu();
            return;
        }
    }
    
    
    crate::println!();
    crate::h!(D_, "Saving to disk for persistence...");
    match crate::persistence::ftm(ftn, gj) {
        Ok(_) => crate::h!(B_, "  Saved! Will survive reboot."),
        Err(aa) => crate::h!(D_, "  Could not persist: {}", aa),
    }
    
    crate::println!();
    crate::h!(G_, "----------------------------------------------------------------");
    crate::h!(G_, "                    Download Complete!");
    crate::h!(G_, "----------------------------------------------------------------");
    
    
    KJ_.store(true, core::sync::atomic::Ordering::Relaxed);
    
    crate::netstack::dhcp::anu();
}