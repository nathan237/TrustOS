













use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use spin::Mutex;


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PkgStatus {
    Zx,
    Bo,
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PkgCategory {
    Ank,
    As,
    De,
    Wf,
    Sq,
    Adh,
    Afj,
}

impl PkgCategory {
    pub fn j(&self) -> &'static str {
        match self {
            PkgCategory::Ank => "system",
            PkgCategory::As => "network",
            PkgCategory::De => "security",
            PkgCategory::Wf => "development",
            PkgCategory::Sq => "games",
            PkgCategory::Adh => "multimedia",
            PkgCategory::Afj => "utilities",
        }
    }
}


#[derive(Clone)]
pub struct By {
    pub j: &'static str,
    pub dk: &'static str,
    pub dc: &'static str,
    pub gb: PkgCategory,
    pub gs: u32,
    pub alz: &'static [&'static str],
    pub status: PkgStatus,
}


static Sy: Mutex<Vec<String>> = Mutex::new(Vec::new());


fn dos() -> Vec<By> {
    vec![
        
        By { j: "coreutils", dk: "1.0.0", dc: "Core POSIX utilities (ls, cp, mv, cat, grep, sort)", gb: PkgCategory::Ank, gs: 48, alz: &[], status: PkgStatus::Bo },
        By { j: "tsh", dk: "1.0.0", dc: "TrustOS Shell — command interpreter with pipes and scripting", gb: PkgCategory::Ank, gs: 64, alz: &[], status: PkgStatus::Bo },
        By { j: "init", dk: "1.0.0", dc: "System initialization and service manager", gb: PkgCategory::Ank, gs: 16, alz: &[], status: PkgStatus::Bo },
        By { j: "devtools", dk: "1.0.0", dc: "Developer tools: profiler, dmesg, memdbg, peek/poke", gb: PkgCategory::Wf, gs: 32, alz: &[], status: PkgStatus::Bo },

        
        By { j: "netstack", dk: "1.0.0", dc: "TCP/IP network stack (ARP, DHCP, DNS, TCP, UDP)", gb: PkgCategory::As, gs: 96, alz: &[], status: PkgStatus::Bo },
        By { j: "curl", dk: "1.0.0", dc: "HTTP client for web requests (curl, wget)", gb: PkgCategory::As, gs: 24, alz: &["netstack"], status: PkgStatus::Bo },
        By { j: "httpd", dk: "1.0.0", dc: "HTTP server — serve web pages from TrustOS", gb: PkgCategory::As, gs: 32, alz: &["netstack"], status: PkgStatus::Zx },
        By { j: "browser", dk: "1.0.0", dc: "Text-mode web browser with HTML/CSS rendering", gb: PkgCategory::As, gs: 56, alz: &["netstack", "curl"], status: PkgStatus::Bo },
        By { j: "tls13", dk: "1.0.0", dc: "TLS 1.3 cryptographic library (AES-GCM, ChaCha20, x25519)", gb: PkgCategory::De, gs: 80, alz: &[], status: PkgStatus::Bo },

        
        By { j: "trustscan", dk: "1.0.0", dc: "Network security toolkit (port scan, sniffer, vuln scanner)", gb: PkgCategory::De, gs: 64, alz: &["netstack"], status: PkgStatus::Bo },
        By { j: "firewall", dk: "0.1.0", dc: "Packet filtering firewall with iptables-like rules", gb: PkgCategory::De, gs: 28, alz: &["netstack"], status: PkgStatus::Zx },
        By { j: "auth", dk: "1.0.0", dc: "User authentication and access control", gb: PkgCategory::De, gs: 16, alz: &[], status: PkgStatus::Bo },

        
        By { j: "trustlang", dk: "1.0.0", dc: "TrustLang programming language (compiler + bytecode VM)", gb: PkgCategory::Wf, gs: 96, alz: &[], status: PkgStatus::Bo },
        By { j: "elftools", dk: "1.0.0", dc: "ELF binary analysis tools (objdump, readelf, disasm)", gb: PkgCategory::Wf, gs: 40, alz: &[], status: PkgStatus::Bo },
        By { j: "git", dk: "0.1.0", dc: "Version control system (basic clone, commit, log)", gb: PkgCategory::Wf, gs: 64, alz: &["netstack"], status: PkgStatus::Zx },
        By { j: "scripting", dk: "1.0.0", dc: "Shell scripting engine (variables, if/for/while, arithmetic)", gb: PkgCategory::Wf, gs: 24, alz: &["tsh"], status: PkgStatus::Zx },

        
        By { j: "snake", dk: "1.0.0", dc: "Classic Snake game", gb: PkgCategory::Sq, gs: 8, alz: &[], status: PkgStatus::Bo },
        By { j: "chess", dk: "1.0.0", dc: "Chess engine with AI opponent (minimax + alpha-beta)", gb: PkgCategory::Sq, gs: 48, alz: &[], status: PkgStatus::Bo },
        By { j: "doom", dk: "0.1.0", dc: "3D raycasting FPS game engine", gb: PkgCategory::Sq, gs: 64, alz: &[], status: PkgStatus::Bo },
        By { j: "nes", dk: "1.0.0", dc: "NES emulator (6502 CPU, 2C02 PPU, iNES ROMs)", gb: PkgCategory::Sq, gs: 56, alz: &[], status: PkgStatus::Bo },
        By { j: "gameboy", dk: "1.0.0", dc: "Game Boy emulator (LR35902, MBC1/3/5)", gb: PkgCategory::Sq, gs: 48, alz: &[], status: PkgStatus::Bo },
        By { j: "tetris", dk: "0.1.0", dc: "Tetris — falling blocks puzzle", gb: PkgCategory::Sq, gs: 8, alz: &[], status: PkgStatus::Zx },

        
        By { j: "cosmic", dk: "1.0.0", dc: "COSMIC desktop environment (window manager + compositor)", gb: PkgCategory::Adh, gs: 256, alz: &[], status: PkgStatus::Bo },
        By { j: "synth", dk: "1.0.0", dc: "TrustSynth — polyphonic audio synthesizer", gb: PkgCategory::Adh, gs: 32, alz: &[], status: PkgStatus::Bo },
        By { j: "video", dk: "1.0.0", dc: "TrustVideo — custom video codec and player", gb: PkgCategory::Adh, gs: 40, alz: &[], status: PkgStatus::Bo },
        By { j: "imageview", dk: "1.0.0", dc: "Image viewer with PPM/BMP support", gb: PkgCategory::Adh, gs: 16, alz: &[], status: PkgStatus::Bo },

        
        By { j: "hypervisor", dk: "1.0.0", dc: "Type-1 hypervisor (VT-x / AMD SVM)", gb: PkgCategory::Afj, gs: 192, alz: &[], status: PkgStatus::Bo },
        By { j: "lab", dk: "1.0.0", dc: "TrustLab — real-time OS introspection laboratory", gb: PkgCategory::Afj, gs: 48, alz: &[], status: PkgStatus::Bo },
        By { j: "jarvis", dk: "1.0.0", dc: "Jarvis AI assistant — natural language OS control", gb: PkgCategory::Afj, gs: 32, alz: &[], status: PkgStatus::Bo },
        By { j: "wayland", dk: "1.0.0", dc: "Wayland compositor (native display server)", gb: PkgCategory::Afj, gs: 64, alz: &[], status: PkgStatus::Bo },
    ]
}


pub fn ufu() {
    let rx = dos();
    let adw = Sy.lock();

    crate::h!(crate::framebuffer::G_,
        "TrustPkg — Package Manager for TrustOS");
    crate::h!(crate::framebuffer::G_,
        "============================================");
    crate::println!();

    let mut nie = "";
    for op in &rx {
        let hci = op.gb.j();
        if hci != nie {
            crate::h!(crate::framebuffer::C_, "  [{}]", hci.idx());
            nie = hci;
        }

        let ete = op.status == PkgStatus::Bo
            || adw.iter().any(|bo| bo == op.j);
        let ejb = if ete { "[installed]" } else { "[available]" };
        let s = if ete {
            crate::framebuffer::B_
        } else {
            crate::framebuffer::D_
        };

        crate::print!("    ");
        crate::gr!(s, "{:<16}", op.j);
        crate::print!(" {:<11} v{:<8} {:>4} KB  ", ejb, op.dk, op.gs);
        crate::println!("{}", op.dc);
    }

    let leu = rx.iter().hi(|ai| ai.status == PkgStatus::Bo).az()
        + adw.len();
    let es = rx.len();
    crate::println!();
    crate::println!("  {}/{} packages installed", leu, es);
}


pub fn anw(query: &str) {
    let rx = dos();
    let adw = Sy.lock();
    let fm = query.aqn();
    let lwl: &str = &fm;

    let mut aig = 0;
    for op in &rx {
        let oh = op.j.contains(lwl)
            || op.dc.aqn().contains(lwl)
            || op.gb.j().contains(lwl);

        if oh {
            let ete = op.status == PkgStatus::Bo
                || adw.iter().any(|bo| bo == op.j);
            let ll = if ete { "[installed]" } else { "[available]" };
            crate::gr!(crate::framebuffer::C_, "  {:<16}", op.j);
            crate::println!(" {} — {}", ll, op.dc);
            aig += 1;
        }
    }

    if aig == 0 {
        crate::h!(crate::framebuffer::D_,
            "No packages matching '{}'", query);
    } else {
        crate::println!();
        crate::println!("  {} package(s) found", aig);
    }
}


pub fn tvh(j: &str) {
    let rx = dos();
    let mut adw = Sy.lock();

    
    if let Some(op) = rx.iter().du(|ai| ai.j == j) {
        if op.status == PkgStatus::Bo || adw.iter().any(|bo| bo.as_str() == j) {
            crate::h!(crate::framebuffer::D_,
                "Package '{}' is already installed", j);
            return;
        }

        
        for gem in op.alz {
            let rvq = rx.iter().any(|ai| ai.j == *gem && ai.status == PkgStatus::Bo)
                || adw.iter().any(|bo| bo.as_str() == *gem);
            if !rvq {
                crate::println!("  Installing dependency: {}", gem);
                adw.push(String::from(*gem));
            }
        }

        
        crate::println!("  Downloading {}@{}...", op.j, op.dk);
        for a in 0..5 {
            for _ in 0..200_000 { core::hint::hc(); }
            crate::print!("\r  [{}>{}] {}%", 
                "=".afd(a + 1), " ".afd(4 - a), (a + 1) * 20);
        }
        crate::println!();
        crate::println!("  Unpacking {} ({} KB)...", op.j, op.gs);
        for _ in 0..100_000 { core::hint::hc(); }
        crate::println!("  Configuring {}...", op.j);
        for _ in 0..100_000 { core::hint::hc(); }

        adw.push(String::from(j));

        crate::h!(crate::framebuffer::G_,
            "  Package '{}' installed successfully!", j);

        
        let path = format!("/var/trustpkg/{}", j);
        let _ = crate::ramfs::fh(|fs| {
            let _ = fs.ut("/var");
            let _ = fs.ut("/var/trustpkg");
            fs.ns(&path, format!("{}@{}\n", j, op.dk).as_bytes())
        });
    } else {
        crate::h!(crate::framebuffer::A_,
            "Package '{}' not found. Use 'trustpkg search' to find packages.", j);
    }
}


pub fn remove(j: &str) {
    let rx = dos();
    let mut adw = Sy.lock();

    
    if let Some(op) = rx.iter().du(|ai| ai.j == j) {
        if op.status == PkgStatus::Bo && !adw.iter().any(|bo| bo.as_str() == j) {
            crate::h!(crate::framebuffer::A_,
                "Cannot remove core package '{}'", j);
            return;
        }
    }

    if let Some(u) = adw.iter().qf(|bo| bo.as_str() == j) {
        adw.remove(u);
        crate::println!("  Removing {}...", j);
        for _ in 0..100_000 { core::hint::hc(); }

        
        let path = format!("/var/trustpkg/{}", j);
        let _ = crate::ramfs::fh(|fs| fs.hb(&path));

        crate::h!(crate::framebuffer::B_,
            "  Package '{}' removed.", j);
    } else {
        crate::h!(crate::framebuffer::D_,
            "Package '{}' is not installed or is a core package.", j);
    }
}


pub fn co(j: &str) {
    let rx = dos();
    let adw = Sy.lock();

    if let Some(op) = rx.iter().du(|ai| ai.j == j) {
        let ete = op.status == PkgStatus::Bo
            || adw.iter().any(|bo| bo.as_str() == j);

        crate::h!(crate::framebuffer::C_, "Package: {}", op.j);
        crate::println!("  Version:     {}", op.dk);
        crate::println!("  Category:    {}", op.gb.j());
        crate::println!("  Size:        {} KB", op.gs);
        crate::println!("  Status:      {}", if ete { "installed" } else { "available" });
        crate::println!("  Description: {}", op.dc);
        if !op.alz.is_empty() {
            let jw: Vec<&str> = op.alz.iter().hu().collect();
            crate::println!("  Depends on:  {}", jw.rr(", "));
        }
    } else {
        crate::h!(crate::framebuffer::A_,
            "Package '{}' not found", j);
    }
}


pub fn ufr() {
    let rx = dos();
    let ang = Sy.lock();

    crate::h!(crate::framebuffer::G_, "Installed packages:");
    crate::println!();

    let mut az = 0;
    for op in &rx {
        if op.status == PkgStatus::Bo {
            crate::gr!(crate::framebuffer::B_, "  {:<16}", op.j);
            crate::println!(" v{:<8}  {}", op.dk, op.dc);
            az += 1;
        }
    }

    for j in ang.iter() {
        if let Some(op) = rx.iter().du(|ai| ai.j == j.as_str()) {
            crate::gr!(crate::framebuffer::C_, "  {:<16}", op.j);
            crate::println!(" v{:<8}  {} [user-installed]", op.dk, op.dc);
            az += 1;
        }
    }

    crate::println!();
    crate::println!("  {} package(s) installed", az);
}


pub fn qs() {
    crate::println!("  Fetching package index from trust://repo.trustos.dev/...");
    for a in 0..3 {
        for _ in 0..300_000 { core::hint::hc(); }
        crate::print!(".");
    }
    crate::println!();
    crate::h!(crate::framebuffer::B_,
        "  Package catalog updated. {} packages available.", dos().len());
}


pub fn leu() -> usize {
    let rx = dos();
    let ang = Sy.lock();
    rx.iter().hi(|ai| ai.status == PkgStatus::Bo).az() + ang.len()
}


pub fn cus() -> usize {
    dos().len()
}


pub fn otn(j: &str) -> bool {
    dos().iter().any(|ai| ai.j == j)
}
