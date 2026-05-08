













use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use spin::Mutex;


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PkgStatus {
    Available,
    Installed,
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PkgCategory {
    System,
    Network,
    Security,
    Development,
    Games,
    Multimedia,
    Utilities,
}

impl PkgCategory {
    pub fn name(&self) -> &'static str {
        match self {
            PkgCategory::System => "system",
            PkgCategory::Network => "network",
            PkgCategory::Security => "security",
            PkgCategory::Development => "development",
            PkgCategory::Games => "games",
            PkgCategory::Multimedia => "multimedia",
            PkgCategory::Utilities => "utilities",
        }
    }
}


#[derive(Clone)]
pub struct At {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub category: PkgCategory,
    pub size_kb: u32,
    pub dependencies: &'static [&'static str],
    pub status: PkgStatus,
}


static Ig: Mutex<Vec<String>> = Mutex::new(Vec::new());


fn bla() -> Vec<At> {
    vec![
        
        At { name: "coreutils", version: "1.0.0", description: "Core POSIX utilities (ls, cp, mv, cat, grep, sort)", category: PkgCategory::System, size_kb: 48, dependencies: &[], status: PkgStatus::Installed },
        At { name: "tsh", version: "1.0.0", description: "TrustOS Shell — command interpreter with pipes and scripting", category: PkgCategory::System, size_kb: 64, dependencies: &[], status: PkgStatus::Installed },
        At { name: "init", version: "1.0.0", description: "System initialization and service manager", category: PkgCategory::System, size_kb: 16, dependencies: &[], status: PkgStatus::Installed },
        At { name: "devtools", version: "1.0.0", description: "Developer tools: profiler, dmesg, memdbg, peek/poke", category: PkgCategory::Development, size_kb: 32, dependencies: &[], status: PkgStatus::Installed },

        
        At { name: "netstack", version: "1.0.0", description: "TCP/IP network stack (ARP, DHCP, DNS, TCP, UDP)", category: PkgCategory::Network, size_kb: 96, dependencies: &[], status: PkgStatus::Installed },
        At { name: "curl", version: "1.0.0", description: "HTTP client for web requests (curl, wget)", category: PkgCategory::Network, size_kb: 24, dependencies: &["netstack"], status: PkgStatus::Installed },
        At { name: "httpd", version: "1.0.0", description: "HTTP server — serve web pages from TrustOS", category: PkgCategory::Network, size_kb: 32, dependencies: &["netstack"], status: PkgStatus::Available },
        At { name: "browser", version: "1.0.0", description: "Text-mode web browser with HTML/CSS rendering", category: PkgCategory::Network, size_kb: 56, dependencies: &["netstack", "curl"], status: PkgStatus::Installed },
        At { name: "tls13", version: "1.0.0", description: "TLS 1.3 cryptographic library (AES-GCM, ChaCha20, x25519)", category: PkgCategory::Security, size_kb: 80, dependencies: &[], status: PkgStatus::Installed },

        
        At { name: "trustscan", version: "1.0.0", description: "Network security toolkit (port scan, sniffer, vuln scanner)", category: PkgCategory::Security, size_kb: 64, dependencies: &["netstack"], status: PkgStatus::Installed },
        At { name: "firewall", version: "0.1.0", description: "Packet filtering firewall with iptables-like rules", category: PkgCategory::Security, size_kb: 28, dependencies: &["netstack"], status: PkgStatus::Available },
        At { name: "auth", version: "1.0.0", description: "User authentication and access control", category: PkgCategory::Security, size_kb: 16, dependencies: &[], status: PkgStatus::Installed },

        
        At { name: "trustlang", version: "1.0.0", description: "TrustLang programming language (compiler + bytecode VM)", category: PkgCategory::Development, size_kb: 96, dependencies: &[], status: PkgStatus::Installed },
        At { name: "elftools", version: "1.0.0", description: "ELF binary analysis tools (objdump, readelf, disasm)", category: PkgCategory::Development, size_kb: 40, dependencies: &[], status: PkgStatus::Installed },
        At { name: "git", version: "0.1.0", description: "Version control system (basic clone, commit, log)", category: PkgCategory::Development, size_kb: 64, dependencies: &["netstack"], status: PkgStatus::Available },
        At { name: "scripting", version: "1.0.0", description: "Shell scripting engine (variables, if/for/while, arithmetic)", category: PkgCategory::Development, size_kb: 24, dependencies: &["tsh"], status: PkgStatus::Available },

        
        At { name: "snake", version: "1.0.0", description: "Classic Snake game", category: PkgCategory::Games, size_kb: 8, dependencies: &[], status: PkgStatus::Installed },
        At { name: "chess", version: "1.0.0", description: "Chess engine with AI opponent (minimax + alpha-beta)", category: PkgCategory::Games, size_kb: 48, dependencies: &[], status: PkgStatus::Installed },
        At { name: "doom", version: "0.1.0", description: "3D raycasting FPS game engine", category: PkgCategory::Games, size_kb: 64, dependencies: &[], status: PkgStatus::Installed },
        At { name: "nes", version: "1.0.0", description: "NES emulator (6502 CPU, 2C02 PPU, iNES ROMs)", category: PkgCategory::Games, size_kb: 56, dependencies: &[], status: PkgStatus::Installed },
        At { name: "gameboy", version: "1.0.0", description: "Game Boy emulator (LR35902, MBC1/3/5)", category: PkgCategory::Games, size_kb: 48, dependencies: &[], status: PkgStatus::Installed },
        At { name: "tetris", version: "0.1.0", description: "Tetris — falling blocks puzzle", category: PkgCategory::Games, size_kb: 8, dependencies: &[], status: PkgStatus::Available },

        
        At { name: "cosmic", version: "1.0.0", description: "COSMIC desktop environment (window manager + compositor)", category: PkgCategory::Multimedia, size_kb: 256, dependencies: &[], status: PkgStatus::Installed },
        At { name: "synth", version: "1.0.0", description: "TrustSynth — polyphonic audio synthesizer", category: PkgCategory::Multimedia, size_kb: 32, dependencies: &[], status: PkgStatus::Installed },
        At { name: "video", version: "1.0.0", description: "TrustVideo — custom video codec and player", category: PkgCategory::Multimedia, size_kb: 40, dependencies: &[], status: PkgStatus::Installed },
        At { name: "imageview", version: "1.0.0", description: "Image viewer with PPM/BMP support", category: PkgCategory::Multimedia, size_kb: 16, dependencies: &[], status: PkgStatus::Installed },

        
        At { name: "hypervisor", version: "1.0.0", description: "Type-1 hypervisor (VT-x / AMD SVM)", category: PkgCategory::Utilities, size_kb: 192, dependencies: &[], status: PkgStatus::Installed },
        At { name: "lab", version: "1.0.0", description: "TrustLab — real-time OS introspection laboratory", category: PkgCategory::Utilities, size_kb: 48, dependencies: &[], status: PkgStatus::Installed },
        At { name: "jarvis", version: "1.0.0", description: "Jarvis AI assistant — natural language OS control", category: PkgCategory::Utilities, size_kb: 32, dependencies: &[], status: PkgStatus::Installed },
        At { name: "wayland", version: "1.0.0", description: "Wayland compositor (native display server)", category: PkgCategory::Utilities, size_kb: 64, dependencies: &[], status: PkgStatus::Installed },
    ]
}


pub fn mzg() {
    let hx = bla();
    let installed = Ig.lock();

    crate::n!(crate::framebuffer::G_,
        "TrustPkg — Package Manager for TrustOS");
    crate::n!(crate::framebuffer::G_,
        "============================================");
    crate::println!();

    let mut hpo = "";
    for gh in &hx {
        let dki = gh.category.name();
        if dki != hpo {
            crate::n!(crate::framebuffer::C_, "  [{}]", dki.to_uppercase());
            hpo = dki;
        }

        let cbb = gh.status == PkgStatus::Installed
            || installed.iter().any(|ae| ae == gh.name);
        let bvz = if cbb { "[installed]" } else { "[available]" };
        let color = if cbb {
            crate::framebuffer::B_
        } else {
            crate::framebuffer::D_
        };

        crate::print!("    ");
        crate::bq!(color, "{:<16}", gh.name);
        crate::print!(" {:<11} v{:<8} {:>4} KB  ", bvz, gh.version, gh.size_kb);
        crate::println!("{}", gh.description);
    }

    let gcy = hx.iter().filter(|aa| aa.status == PkgStatus::Installed).count()
        + installed.len();
    let av = hx.len();
    crate::println!();
    crate::println!("  {}/{} packages installed", gcy, av);
}


pub fn search(query: &str) {
    let hx = bla();
    let installed = Ig.lock();
    let q = query.to_lowercase();
    let gpe: &str = &q;

    let mut nj = 0;
    for gh in &hx {
        let matches = gh.name.contains(gpe)
            || gh.description.to_lowercase().contains(gpe)
            || gh.category.name().contains(gpe);

        if matches {
            let cbb = gh.status == PkgStatus::Installed
                || installed.iter().any(|ae| ae == gh.name);
            let tag = if cbb { "[installed]" } else { "[available]" };
            crate::bq!(crate::framebuffer::C_, "  {:<16}", gh.name);
            crate::println!(" {} — {}", tag, gh.description);
            nj += 1;
        }
    }

    if nj == 0 {
        crate::n!(crate::framebuffer::D_,
            "No packages matching '{}'", query);
    } else {
        crate::println!();
        crate::println!("  {} package(s) found", nj);
    }
}


pub fn mqq(name: &str) {
    let hx = bla();
    let mut installed = Ig.lock();

    
    if let Some(gh) = hx.iter().find(|aa| aa.name == name) {
        if gh.status == PkgStatus::Installed || installed.iter().any(|ae| ae.as_str() == name) {
            crate::n!(crate::framebuffer::D_,
                "Package '{}' is already installed", name);
            return;
        }

        
        for dep in gh.dependencies {
            let ldg = hx.iter().any(|aa| aa.name == *dep && aa.status == PkgStatus::Installed)
                || installed.iter().any(|ae| ae.as_str() == *dep);
            if !ldg {
                crate::println!("  Installing dependency: {}", dep);
                installed.push(String::from(*dep));
            }
        }

        
        crate::println!("  Downloading {}@{}...", gh.name, gh.version);
        for i in 0..5 {
            for _ in 0..200_000 { core::hint::spin_loop(); }
            crate::print!("\r  [{}>{}] {}%", 
                "=".repeat(i + 1), " ".repeat(4 - i), (i + 1) * 20);
        }
        crate::println!();
        crate::println!("  Unpacking {} ({} KB)...", gh.name, gh.size_kb);
        for _ in 0..100_000 { core::hint::spin_loop(); }
        crate::println!("  Configuring {}...", gh.name);
        for _ in 0..100_000 { core::hint::spin_loop(); }

        installed.push(String::from(name));

        crate::n!(crate::framebuffer::G_,
            "  Package '{}' installed successfully!", name);

        
        let path = format!("/var/trustpkg/{}", name);
        let _ = crate::ramfs::bh(|fs| {
            let _ = fs.mkdir("/var");
            let _ = fs.mkdir("/var/trustpkg");
            fs.write_file(&path, format!("{}@{}\n", name, gh.version).as_bytes())
        });
    } else {
        crate::n!(crate::framebuffer::A_,
            "Package '{}' not found. Use 'trustpkg search' to find packages.", name);
    }
}


pub fn remove(name: &str) {
    let hx = bla();
    let mut installed = Ig.lock();

    
    if let Some(gh) = hx.iter().find(|aa| aa.name == name) {
        if gh.status == PkgStatus::Installed && !installed.iter().any(|ae| ae.as_str() == name) {
            crate::n!(crate::framebuffer::A_,
                "Cannot remove core package '{}'", name);
            return;
        }
    }

    if let Some(pos) = installed.iter().position(|ae| ae.as_str() == name) {
        installed.remove(pos);
        crate::println!("  Removing {}...", name);
        for _ in 0..100_000 { core::hint::spin_loop(); }

        
        let path = format!("/var/trustpkg/{}", name);
        let _ = crate::ramfs::bh(|fs| fs.rm(&path));

        crate::n!(crate::framebuffer::B_,
            "  Package '{}' removed.", name);
    } else {
        crate::n!(crate::framebuffer::D_,
            "Package '{}' is not installed or is a core package.", name);
    }
}


pub fn info(name: &str) {
    let hx = bla();
    let installed = Ig.lock();

    if let Some(gh) = hx.iter().find(|aa| aa.name == name) {
        let cbb = gh.status == PkgStatus::Installed
            || installed.iter().any(|ae| ae.as_str() == name);

        crate::n!(crate::framebuffer::C_, "Package: {}", gh.name);
        crate::println!("  Version:     {}", gh.version);
        crate::println!("  Category:    {}", gh.category.name());
        crate::println!("  Size:        {} KB", gh.size_kb);
        crate::println!("  Status:      {}", if cbb { "installed" } else { "available" });
        crate::println!("  Description: {}", gh.description);
        if !gh.dependencies.is_empty() {
            let deps: Vec<&str> = gh.dependencies.iter().copied().collect();
            crate::println!("  Depends on:  {}", deps.join(", "));
        }
    } else {
        crate::n!(crate::framebuffer::A_,
            "Package '{}' not found", name);
    }
}


pub fn mzd() {
    let hx = bla();
    let ua = Ig.lock();

    crate::n!(crate::framebuffer::G_, "Installed packages:");
    crate::println!();

    let mut count = 0;
    for gh in &hx {
        if gh.status == PkgStatus::Installed {
            crate::bq!(crate::framebuffer::B_, "  {:<16}", gh.name);
            crate::println!(" v{:<8}  {}", gh.version, gh.description);
            count += 1;
        }
    }

    for name in ua.iter() {
        if let Some(gh) = hx.iter().find(|aa| aa.name == name.as_str()) {
            crate::bq!(crate::framebuffer::C_, "  {:<16}", gh.name);
            crate::println!(" v{:<8}  {} [user-installed]", gh.version, gh.description);
            count += 1;
        }
    }

    crate::println!();
    crate::println!("  {} package(s) installed", count);
}


pub fn update() {
    crate::println!("  Fetching package index from trust://repo.trustos.dev/...");
    for i in 0..3 {
        for _ in 0..300_000 { core::hint::spin_loop(); }
        crate::print!(".");
    }
    crate::println!();
    crate::n!(crate::framebuffer::B_,
        "  Package catalog updated. {} packages available.", bla().len());
}


pub fn gcy() -> usize {
    let hx = bla();
    let ua = Ig.lock();
    hx.iter().filter(|aa| aa.status == PkgStatus::Installed).count() + ua.len()
}


pub fn total_count() -> usize {
    bla().len()
}


pub fn itd(name: &str) -> bool {
    bla().iter().any(|aa| aa.name == name)
}
