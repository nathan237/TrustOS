




use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};


pub const CSF_: [u8; 4] = [192, 168, 56, 1];
pub const PK_: u16 = 8080;


#[derive(Clone)]
pub struct Nk {
    pub ad: &'static str,
    pub j: &'static str,
    pub dk: &'static str,
    pub dc: &'static str,
    pub aga: u32,
    pub it: &'static str,
    pub pa: &'static str,
    pub adw: bool,
}


pub static Aav: Mutex<Vec<Nk>> = Mutex::new(Vec::new());


pub fn init() {
    let mut ced = Aav.lock();
    ced.clear();
    
    
    ced.push(Nk {
        ad: "alpine",
        j: "Alpine Linux",
        dk: "3.19",
        dc: "Lightweight security-oriented Linux",
        aga: 3,
        it: "alpine-minirootfs.tar.gz",
        pa: "🏔",
        adw: false,
    });
    
    ced.push(Nk {
        ad: "busybox",
        j: "BusyBox",
        dk: "1.36",
        dc: "Minimal Unix utilities",
        aga: 1,
        it: "busybox.tar.gz",
        pa: "📦",
        adw: false,
    });
    
    ced.push(Nk {
        ad: "void-musl",
        j: "Void Linux (musl)",
        dk: "2024",
        dc: "Void Linux with musl libc",
        aga: 50,
        it: "void-musl.tar.gz",
        pa: "⚫",
        adw: false,
    });
    
    
    ced.push(Nk {
        ad: "debian-mini",
        j: "Debian Minimal",
        dk: "12",
        dc: "Debian base system",
        aga: 150,
        it: "debian-mini.tar.gz",
        pa: "🌀",
        adw: false,
    });
    
    ced.push(Nk {
        ad: "ubuntu-core",
        j: "Ubuntu Core",
        dk: "22.04",
        dc: "Ubuntu minimal cloud image",
        aga: 200,
        it: "ubuntu-core.tar.gz",
        pa: "🟠",
        adw: false,
    });
    
    ced.push(Nk {
        ad: "arch-bootstrap",
        j: "Arch Linux",
        dk: "2024.02",
        dc: "Arch Linux bootstrap",
        aga: 180,
        it: "arch-bootstrap.tar.gz",
        pa: "🔵",
        adw: false,
    });
    
    
    for distro in ced.el() {
        let path = format!("/opt/linux/{}", distro.it);
        
        let aja = crate::ramfs::fh(|fs| {
            fs.hm(&path).is_ok()
        });
        if aja {
            distro.adw = true;
        }
    }
    
    crate::serial_println!("[DISTRO] Initialized {} distributions", ced.len());
}


pub fn aoy() -> Vec<Nk> {
    Aav.lock().clone()
}


pub fn get(ad: &str) -> Option<Nk> {
    Aav.lock().iter().du(|bc| bc.ad == ad).abn()
}


pub fn ete(ad: &str) -> bool {
    Aav.lock().iter().du(|bc| bc.ad == ad).map(|bc| bc.adw).unwrap_or(false)
}


pub fn ukb(ad: &str) {
    let mut ced = Aav.lock();
    if let Some(distro) = ced.el().du(|bc| bc.ad == ad) {
        distro.adw = true;
    }
}


pub fn kqp(ad: &str) -> Result<usize, &'static str> {
    let distro = get(ad).ok_or("Distribution not found")?;
    
    crate::serial_println!("[DISTRO] Downloading {} ({} MB)...", distro.j, distro.aga);
    
    
    crate::netstack::dhcp::fvw();
    
    
    use crate::network::Ipv4Address;
    crate::serial_println!("[DISTRO] Setting static IP 192.168.56.100 for download");
    crate::network::hzx(
        Ipv4Address::new(192, 168, 56, 100),
        Ipv4Address::new(255, 255, 255, 0),
        Some(Ipv4Address::new(192, 168, 56, 1)),
    );
    
    
    for _ in 0..200 {
        crate::netstack::poll();
    }
    
    
    let ip = CSF_;
    let port = PK_;
    
    let ey = match crate::netstack::tcp::cue(ip, port) {
        Ok(ai) => ai,
        Err(aa) => {
            crate::netstack::dhcp::anu();
            return Err(aa);
        }
    };
    
    if !crate::netstack::tcp::dnd(ip, port, ey, 3000) {
        crate::netstack::dhcp::anu();
        return Err("Connection timeout - is the server running?");
    }
    
    
    let path = format!("/{}", distro.it);
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: 192.168.56.1\r\nConnection: close\r\n\r\n",
        path
    );
    
    crate::netstack::tcp::dlo(ip, port, ey, request.as_bytes())?;
    crate::serial_println!("[DISTRO] Request sent, waiting for response...");
    
    
    let mut f: Vec<u8> = Vec::fc((distro.aga as usize + 1) * 1024 * 1024);
    let ay = crate::logger::lh();
    let mut cyt: u32 = 0;
    let mut etv = ay;
    let mut oic = ay;
    let ate = (distro.aga as usize + 10) * 1024 * 1024;
    
    loop {
        
        for _ in 0..10 {
            crate::netstack::poll();
        }
        
        let mut ckw = false;
        
        while let Some(jj) = crate::netstack::tcp::cme(ip, port, ey) {
            ckw = true;
            if f.len() + jj.len() > ate {
                break;
            }
            f.bk(&jj);
        }
        
        
        let iu = crate::logger::lh();
        if iu.ao(oic) >= 1000 {
            crate::serial_println!("[DISTRO] Downloaded {} KB...", f.len() / 1024);
            oic = iu;
        }
        
        
        if iu.ao(etv) >= 5 {
            crate::netstack::tcp::fiv(ip, port, ey);
            etv = iu;
        }
        
        if !ckw {
            cyt += 1;
            if crate::netstack::tcp::bqr(ip, port, ey) {
                crate::netstack::tcp::fiv(ip, port, ey);
                crate::serial_println!("[DISTRO] FIN received, download complete");
                break;
            }
            if cyt > 200_000 {
                crate::serial_println!("[DISTRO] Idle timeout");
                crate::netstack::tcp::fiv(ip, port, ey);
                break;
            }
            
            for _ in 0..10 { core::hint::hc(); }
        } else {
            cyt = 0;
        }
        
        
        if crate::logger::lh().ao(ay) > 120_000 {
            crate::netstack::dhcp::anu();
            return Err("Download timeout");
        }
    }
    
    let _ = crate::netstack::tcp::bwx(ip, port, ey);
    
    if f.is_empty() {
        crate::netstack::dhcp::anu();
        return Err("No data received");
    }
    
    
    let cvy = f.ee(4)
        .qf(|d| d == b"\r\n\r\n")
        .map(|ai| ai + 4)
        .unwrap_or(0);
    
    let gj = &f[cvy..];
    
    if gj.is_empty() {
        return Err("Empty response body");
    }
    
    
    let ftn = format!("/opt/linux/{}", distro.it);
    
    crate::ramfs::fh(|fs| {
        let _ = fs.ut("/opt");
        let _ = fs.ut("/opt/linux");
        let _ = fs.touch(&ftn);
        fs.ns(&ftn, gj)
    }).jd(|_| "Failed to save file")?;
    
    
    ukb(ad);
    
    
    crate::netstack::dhcp::anu();
    
    crate::serial_println!("[DISTRO] {} installed ({} KB)", distro.j, gj.len() / 1024);
    
    Ok(gj.len())
}


pub fn vw(ad: &str) -> Result<(), &'static str> {
    let distro = get(ad).ok_or("Distribution not found")?;
    
    if !distro.adw {
        return Err("Distribution not installed - use 'distro install <id>' first");
    }
    
    crate::serial_println!("[DISTRO] Starting {} {}...", distro.j, distro.dk);
    
    
    if crate::hypervisor::zu() {
        crate::serial_println!("[DISTRO] Using hardware virtualization");
        
        return Ok(());
    }
    
    
    crate::serial_println!("[DISTRO] Hardware virtualization not available, using emulation");
    
    Ok(())
}
