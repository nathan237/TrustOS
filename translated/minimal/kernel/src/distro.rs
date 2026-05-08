




use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};


pub const CVW_: [u8; 4] = [192, 168, 56, 1];
pub const QH_: u16 = 8080;


#[derive(Clone)]
pub struct Ft {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub size_mb: u32,
    pub filename: &'static str,
    pub icon: &'static str,
    pub installed: bool,
}


pub static Lk: Mutex<Vec<Ft>> = Mutex::new(Vec::new());


pub fn init() {
    let mut aqi = Lk.lock();
    aqi.clear();
    
    
    aqi.push(Ft {
        id: "alpine",
        name: "Alpine Linux",
        version: "3.19",
        description: "Lightweight security-oriented Linux",
        size_mb: 3,
        filename: "alpine-minirootfs.tar.gz",
        icon: "🏔",
        installed: false,
    });
    
    aqi.push(Ft {
        id: "busybox",
        name: "BusyBox",
        version: "1.36",
        description: "Minimal Unix utilities",
        size_mb: 1,
        filename: "busybox.tar.gz",
        icon: "📦",
        installed: false,
    });
    
    aqi.push(Ft {
        id: "void-musl",
        name: "Void Linux (musl)",
        version: "2024",
        description: "Void Linux with musl libc",
        size_mb: 50,
        filename: "void-musl.tar.gz",
        icon: "⚫",
        installed: false,
    });
    
    
    aqi.push(Ft {
        id: "debian-mini",
        name: "Debian Minimal",
        version: "12",
        description: "Debian base system",
        size_mb: 150,
        filename: "debian-mini.tar.gz",
        icon: "🌀",
        installed: false,
    });
    
    aqi.push(Ft {
        id: "ubuntu-core",
        name: "Ubuntu Core",
        version: "22.04",
        description: "Ubuntu minimal cloud image",
        size_mb: 200,
        filename: "ubuntu-core.tar.gz",
        icon: "🟠",
        installed: false,
    });
    
    aqi.push(Ft {
        id: "arch-bootstrap",
        name: "Arch Linux",
        version: "2024.02",
        description: "Arch Linux bootstrap",
        size_mb: 180,
        filename: "arch-bootstrap.tar.gz",
        icon: "🔵",
        installed: false,
    });
    
    
    for distro in aqi.iter_mut() {
        let path = format!("/opt/linux/{}", distro.filename);
        
        let exists = crate::ramfs::bh(|fs| {
            fs.stat(&path).is_ok()
        });
        if exists {
            distro.installed = true;
        }
    }
    
    crate::serial_println!("[DISTRO] Initialized {} distributions", aqi.len());
}


pub fn list() -> Vec<Ft> {
    Lk.lock().clone()
}


pub fn get(id: &str) -> Option<Ft> {
    Lk.lock().iter().find(|d| d.id == id).cloned()
}


pub fn cbb(id: &str) -> bool {
    Lk.lock().iter().find(|d| d.id == id).map(|d| d.installed).unwrap_or(false)
}


pub fn nca(id: &str) {
    let mut aqi = Lk.lock();
    if let Some(distro) = aqi.iter_mut().find(|d| d.id == id) {
        distro.installed = true;
    }
}


pub fn fsu(id: &str) -> Result<usize, &'static str> {
    let distro = get(id).ok_or("Distribution not found")?;
    
    crate::serial_println!("[DISTRO] Downloading {} ({} MB)...", distro.name, distro.size_mb);
    
    
    crate::netstack::dhcp::crf();
    
    
    use crate::network::Ipv4Address;
    crate::serial_println!("[DISTRO] Setting static IP 192.168.56.100 for download");
    crate::network::deh(
        Ipv4Address::new(192, 168, 56, 100),
        Ipv4Address::new(255, 255, 255, 0),
        Some(Ipv4Address::new(192, 168, 56, 1)),
    );
    
    
    for _ in 0..200 {
        crate::netstack::poll();
    }
    
    
    let ip = CVW_;
    let port = QH_;
    
    let src_port = match crate::netstack::tcp::azp(ip, port) {
        Ok(aa) => aa,
        Err(e) => {
            crate::netstack::dhcp::resume();
            return Err(e);
        }
    };
    
    if !crate::netstack::tcp::bjy(ip, port, src_port, 3000) {
        crate::netstack::dhcp::resume();
        return Err("Connection timeout - is the server running?");
    }
    
    
    let path = format!("/{}", distro.filename);
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: 192.168.56.1\r\nConnection: close\r\n\r\n",
        path
    );
    
    crate::netstack::tcp::bjc(ip, port, src_port, request.as_bytes())?;
    crate::serial_println!("[DISTRO] Request sent, waiting for response...");
    
    
    let mut data: Vec<u8> = Vec::with_capacity((distro.size_mb as usize + 1) * 1024 * 1024);
    let start = crate::logger::eg();
    let mut bch: u32 = 0;
    let mut cbi = start;
    let mut iji = start;
    let max_size = (distro.size_mb as usize + 10) * 1024 * 1024;
    
    loop {
        
        for _ in 0..10 {
            crate::netstack::poll();
        }
        
        let mut aty = false;
        
        while let Some(df) = crate::netstack::tcp::aus(ip, port, src_port) {
            aty = true;
            if data.len() + df.len() > max_size {
                break;
            }
            data.extend_from_slice(&df);
        }
        
        
        let cy = crate::logger::eg();
        if cy.saturating_sub(iji) >= 1000 {
            crate::serial_println!("[DISTRO] Downloaded {} KB...", data.len() / 1024);
            iji = cy;
        }
        
        
        if cy.saturating_sub(cbi) >= 5 {
            crate::netstack::tcp::cjr(ip, port, src_port);
            cbi = cy;
        }
        
        if !aty {
            bch += 1;
            if crate::netstack::tcp::fin_received(ip, port, src_port) {
                crate::netstack::tcp::cjr(ip, port, src_port);
                crate::serial_println!("[DISTRO] FIN received, download complete");
                break;
            }
            if bch > 200_000 {
                crate::serial_println!("[DISTRO] Idle timeout");
                crate::netstack::tcp::cjr(ip, port, src_port);
                break;
            }
            
            for _ in 0..10 { core::hint::spin_loop(); }
        } else {
            bch = 0;
        }
        
        
        if crate::logger::eg().saturating_sub(start) > 120_000 {
            crate::netstack::dhcp::resume();
            return Err("Download timeout");
        }
    }
    
    let _ = crate::netstack::tcp::ams(ip, port, src_port);
    
    if data.is_empty() {
        crate::netstack::dhcp::resume();
        return Err("No data received");
    }
    
    
    let bao = data.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|aa| aa + 4)
        .unwrap_or(0);
    
    let body = &data[bao..];
    
    if body.is_empty() {
        return Err("Empty response body");
    }
    
    
    let cpx = format!("/opt/linux/{}", distro.filename);
    
    crate::ramfs::bh(|fs| {
        let _ = fs.mkdir("/opt");
        let _ = fs.mkdir("/opt/linux");
        let _ = fs.touch(&cpx);
        fs.write_file(&cpx, body)
    }).map_err(|_| "Failed to save file")?;
    
    
    nca(id);
    
    
    crate::netstack::dhcp::resume();
    
    crate::serial_println!("[DISTRO] {} installed ({} KB)", distro.name, body.len() / 1024);
    
    Ok(body.len())
}


pub fn run(id: &str) -> Result<(), &'static str> {
    let distro = get(id).ok_or("Distribution not found")?;
    
    if !distro.installed {
        return Err("Distribution not installed - use 'distro install <id>' first");
    }
    
    crate::serial_println!("[DISTRO] Starting {} {}...", distro.name, distro.version);
    
    
    if crate::hypervisor::lq() {
        crate::serial_println!("[DISTRO] Using hardware virtualization");
        
        return Ok(());
    }
    
    
    crate::serial_println!("[DISTRO] Hardware virtualization not available, using emulation");
    
    Ok(())
}
