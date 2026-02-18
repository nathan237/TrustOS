//! TrustPkg — Package manager for TrustOS
//!
//! Manages installable packages (built-in apps, scripts, libraries).
//! Packages are registered in a catalog and can be installed/removed.
//!
//! Commands:
//!   trustpkg list              — show available packages
//!   trustpkg search <query>    — search packages
//!   trustpkg install <pkg>     — install a package
//!   trustpkg remove <pkg>      — remove a package  
//!   trustpkg info <pkg>        — show package details
//!   trustpkg installed         — list installed packages
//!   trustpkg update            — update package catalog

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use spin::Mutex;

/// Package status
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PkgStatus {
    Available,
    Installed,
}

/// Package category
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

/// Package descriptor
#[derive(Clone)]
pub struct Package {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub category: PkgCategory,
    pub size_kb: u32,
    pub dependencies: &'static [&'static str],
    pub status: PkgStatus,
}

/// Global installed packages tracker
static INSTALLED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Built-in package catalog
fn catalog() -> Vec<Package> {
    vec![
        // System
        Package { name: "coreutils", version: "1.0.0", description: "Core POSIX utilities (ls, cp, mv, cat, grep, sort)", category: PkgCategory::System, size_kb: 48, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "tsh", version: "1.0.0", description: "TrustOS Shell — command interpreter with pipes and scripting", category: PkgCategory::System, size_kb: 64, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "init", version: "1.0.0", description: "System initialization and service manager", category: PkgCategory::System, size_kb: 16, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "devtools", version: "1.0.0", description: "Developer tools: profiler, dmesg, memdbg, peek/poke", category: PkgCategory::Development, size_kb: 32, dependencies: &[], status: PkgStatus::Installed },

        // Network
        Package { name: "netstack", version: "1.0.0", description: "TCP/IP network stack (ARP, DHCP, DNS, TCP, UDP)", category: PkgCategory::Network, size_kb: 96, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "curl", version: "1.0.0", description: "HTTP client for web requests (curl, wget)", category: PkgCategory::Network, size_kb: 24, dependencies: &["netstack"], status: PkgStatus::Installed },
        Package { name: "httpd", version: "1.0.0", description: "HTTP server — serve web pages from TrustOS", category: PkgCategory::Network, size_kb: 32, dependencies: &["netstack"], status: PkgStatus::Available },
        Package { name: "browser", version: "1.0.0", description: "Text-mode web browser with HTML/CSS rendering", category: PkgCategory::Network, size_kb: 56, dependencies: &["netstack", "curl"], status: PkgStatus::Installed },
        Package { name: "tls13", version: "1.0.0", description: "TLS 1.3 cryptographic library (AES-GCM, ChaCha20, x25519)", category: PkgCategory::Security, size_kb: 80, dependencies: &[], status: PkgStatus::Installed },

        // Security
        Package { name: "trustscan", version: "1.0.0", description: "Network security toolkit (port scan, sniffer, vuln scanner)", category: PkgCategory::Security, size_kb: 64, dependencies: &["netstack"], status: PkgStatus::Installed },
        Package { name: "firewall", version: "0.1.0", description: "Packet filtering firewall with iptables-like rules", category: PkgCategory::Security, size_kb: 28, dependencies: &["netstack"], status: PkgStatus::Available },
        Package { name: "auth", version: "1.0.0", description: "User authentication and access control", category: PkgCategory::Security, size_kb: 16, dependencies: &[], status: PkgStatus::Installed },

        // Development
        Package { name: "trustlang", version: "1.0.0", description: "TrustLang programming language (compiler + bytecode VM)", category: PkgCategory::Development, size_kb: 96, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "elftools", version: "1.0.0", description: "ELF binary analysis tools (objdump, readelf, disasm)", category: PkgCategory::Development, size_kb: 40, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "git", version: "0.1.0", description: "Version control system (basic clone, commit, log)", category: PkgCategory::Development, size_kb: 64, dependencies: &["netstack"], status: PkgStatus::Available },
        Package { name: "scripting", version: "1.0.0", description: "Shell scripting engine (variables, if/for/while, arithmetic)", category: PkgCategory::Development, size_kb: 24, dependencies: &["tsh"], status: PkgStatus::Available },

        // Games
        Package { name: "snake", version: "1.0.0", description: "Classic Snake game", category: PkgCategory::Games, size_kb: 8, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "chess", version: "1.0.0", description: "Chess engine with AI opponent (minimax + alpha-beta)", category: PkgCategory::Games, size_kb: 48, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "mario64", version: "1.0.0", description: "Super Mario 64 clone with 3D renderer", category: PkgCategory::Games, size_kb: 128, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "doom", version: "0.1.0", description: "3D raycasting FPS game engine", category: PkgCategory::Games, size_kb: 64, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "nes", version: "1.0.0", description: "NES emulator (6502 CPU, 2C02 PPU, iNES ROMs)", category: PkgCategory::Games, size_kb: 56, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "gameboy", version: "1.0.0", description: "Game Boy emulator (LR35902, MBC1/3/5)", category: PkgCategory::Games, size_kb: 48, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "tetris", version: "0.1.0", description: "Tetris — falling blocks puzzle", category: PkgCategory::Games, size_kb: 8, dependencies: &[], status: PkgStatus::Available },

        // Multimedia
        Package { name: "cosmic", version: "1.0.0", description: "COSMIC desktop environment (window manager + compositor)", category: PkgCategory::Multimedia, size_kb: 256, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "synth", version: "1.0.0", description: "TrustSynth — polyphonic audio synthesizer", category: PkgCategory::Multimedia, size_kb: 32, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "video", version: "1.0.0", description: "TrustVideo — custom video codec and player", category: PkgCategory::Multimedia, size_kb: 40, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "imageview", version: "1.0.0", description: "Image viewer with PPM/BMP support", category: PkgCategory::Multimedia, size_kb: 16, dependencies: &[], status: PkgStatus::Installed },

        // Utilities
        Package { name: "hypervisor", version: "1.0.0", description: "Type-1 hypervisor (VT-x / AMD SVM)", category: PkgCategory::Utilities, size_kb: 192, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "lab", version: "1.0.0", description: "TrustLab — real-time OS introspection laboratory", category: PkgCategory::Utilities, size_kb: 48, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "jarvis", version: "1.0.0", description: "Jarvis AI assistant — natural language OS control", category: PkgCategory::Utilities, size_kb: 32, dependencies: &[], status: PkgStatus::Installed },
        Package { name: "wayland", version: "1.0.0", description: "Wayland compositor (native display server)", category: PkgCategory::Utilities, size_kb: 64, dependencies: &[], status: PkgStatus::Installed },
    ]
}

/// List all packages
pub fn list_packages() {
    let cat = catalog();
    let installed = INSTALLED.lock();

    crate::println_color!(crate::framebuffer::COLOR_BRIGHT_GREEN,
        "TrustPkg — Package Manager for TrustOS");
    crate::println_color!(crate::framebuffer::COLOR_BRIGHT_GREEN,
        "============================================");
    crate::println!();

    let mut current_cat = "";
    for pkg in &cat {
        let cat_name = pkg.category.name();
        if cat_name != current_cat {
            crate::println_color!(crate::framebuffer::COLOR_CYAN, "  [{}]", cat_name.to_uppercase());
            current_cat = cat_name;
        }

        let is_installed = pkg.status == PkgStatus::Installed
            || installed.iter().any(|n| n == pkg.name);
        let status_str = if is_installed { "[installed]" } else { "[available]" };
        let color = if is_installed {
            crate::framebuffer::COLOR_GREEN
        } else {
            crate::framebuffer::COLOR_YELLOW
        };

        crate::print!("    ");
        crate::print_color!(color, "{:<16}", pkg.name);
        crate::print!(" {:<11} v{:<8} {:>4} KB  ", status_str, pkg.version, pkg.size_kb);
        crate::println!("{}", pkg.description);
    }

    let installed_count = cat.iter().filter(|p| p.status == PkgStatus::Installed).count()
        + installed.len();
    let total = cat.len();
    crate::println!();
    crate::println!("  {}/{} packages installed", installed_count, total);
}

/// Search packages
pub fn search(query: &str) {
    let cat = catalog();
    let installed = INSTALLED.lock();
    let q = query.to_lowercase();
    let q_lower: &str = &q;

    let mut found = 0;
    for pkg in &cat {
        let matches = pkg.name.contains(q_lower)
            || pkg.description.to_lowercase().contains(q_lower)
            || pkg.category.name().contains(q_lower);

        if matches {
            let is_installed = pkg.status == PkgStatus::Installed
                || installed.iter().any(|n| n == pkg.name);
            let tag = if is_installed { "[installed]" } else { "[available]" };
            crate::print_color!(crate::framebuffer::COLOR_CYAN, "  {:<16}", pkg.name);
            crate::println!(" {} — {}", tag, pkg.description);
            found += 1;
        }
    }

    if found == 0 {
        crate::println_color!(crate::framebuffer::COLOR_YELLOW,
            "No packages matching '{}'", query);
    } else {
        crate::println!();
        crate::println!("  {} package(s) found", found);
    }
}

/// Install a package
pub fn install(name: &str) {
    let cat = catalog();
    let mut installed = INSTALLED.lock();

    // Check if already installed
    if let Some(pkg) = cat.iter().find(|p| p.name == name) {
        if pkg.status == PkgStatus::Installed || installed.iter().any(|n| n.as_str() == name) {
            crate::println_color!(crate::framebuffer::COLOR_YELLOW,
                "Package '{}' is already installed", name);
            return;
        }

        // Check dependencies
        for dep in pkg.dependencies {
            let dep_installed = cat.iter().any(|p| p.name == *dep && p.status == PkgStatus::Installed)
                || installed.iter().any(|n| n.as_str() == *dep);
            if !dep_installed {
                crate::println!("  Installing dependency: {}", dep);
                installed.push(String::from(*dep));
            }
        }

        // Simulate download + install
        crate::println!("  Downloading {}@{}...", pkg.name, pkg.version);
        for i in 0..5 {
            for _ in 0..200_000 { core::hint::spin_loop(); }
            crate::print!("\r  [{}>{}] {}%", 
                "=".repeat(i + 1), " ".repeat(4 - i), (i + 1) * 20);
        }
        crate::println!();
        crate::println!("  Unpacking {} ({} KB)...", pkg.name, pkg.size_kb);
        for _ in 0..100_000 { core::hint::spin_loop(); }
        crate::println!("  Configuring {}...", pkg.name);
        for _ in 0..100_000 { core::hint::spin_loop(); }

        installed.push(String::from(name));

        crate::println_color!(crate::framebuffer::COLOR_BRIGHT_GREEN,
            "  Package '{}' installed successfully!", name);

        // Create marker file in RAMFS
        let path = format!("/var/trustpkg/{}", name);
        let _ = crate::ramfs::with_fs(|fs| {
            let _ = fs.mkdir("/var");
            let _ = fs.mkdir("/var/trustpkg");
            fs.write_file(&path, format!("{}@{}\n", name, pkg.version).as_bytes())
        });
    } else {
        crate::println_color!(crate::framebuffer::COLOR_RED,
            "Package '{}' not found. Use 'trustpkg search' to find packages.", name);
    }
}

/// Remove a package
pub fn remove(name: &str) {
    let cat = catalog();
    let mut installed = INSTALLED.lock();

    // Check if it's a core package
    if let Some(pkg) = cat.iter().find(|p| p.name == name) {
        if pkg.status == PkgStatus::Installed && !installed.iter().any(|n| n.as_str() == name) {
            crate::println_color!(crate::framebuffer::COLOR_RED,
                "Cannot remove core package '{}'", name);
            return;
        }
    }

    if let Some(pos) = installed.iter().position(|n| n.as_str() == name) {
        installed.remove(pos);
        crate::println!("  Removing {}...", name);
        for _ in 0..100_000 { core::hint::spin_loop(); }

        // Remove marker file
        let path = format!("/var/trustpkg/{}", name);
        let _ = crate::ramfs::with_fs(|fs| fs.rm(&path));

        crate::println_color!(crate::framebuffer::COLOR_GREEN,
            "  Package '{}' removed.", name);
    } else {
        crate::println_color!(crate::framebuffer::COLOR_YELLOW,
            "Package '{}' is not installed or is a core package.", name);
    }
}

/// Show package info
pub fn info(name: &str) {
    let cat = catalog();
    let installed = INSTALLED.lock();

    if let Some(pkg) = cat.iter().find(|p| p.name == name) {
        let is_installed = pkg.status == PkgStatus::Installed
            || installed.iter().any(|n| n.as_str() == name);

        crate::println_color!(crate::framebuffer::COLOR_CYAN, "Package: {}", pkg.name);
        crate::println!("  Version:     {}", pkg.version);
        crate::println!("  Category:    {}", pkg.category.name());
        crate::println!("  Size:        {} KB", pkg.size_kb);
        crate::println!("  Status:      {}", if is_installed { "installed" } else { "available" });
        crate::println!("  Description: {}", pkg.description);
        if !pkg.dependencies.is_empty() {
            let deps: Vec<&str> = pkg.dependencies.iter().copied().collect();
            crate::println!("  Depends on:  {}", deps.join(", "));
        }
    } else {
        crate::println_color!(crate::framebuffer::COLOR_RED,
            "Package '{}' not found", name);
    }
}

/// List installed packages only
pub fn list_installed() {
    let cat = catalog();
    let extra = INSTALLED.lock();

    crate::println_color!(crate::framebuffer::COLOR_BRIGHT_GREEN, "Installed packages:");
    crate::println!();

    let mut count = 0;
    for pkg in &cat {
        if pkg.status == PkgStatus::Installed {
            crate::print_color!(crate::framebuffer::COLOR_GREEN, "  {:<16}", pkg.name);
            crate::println!(" v{:<8}  {}", pkg.version, pkg.description);
            count += 1;
        }
    }

    for name in extra.iter() {
        if let Some(pkg) = cat.iter().find(|p| p.name == name.as_str()) {
            crate::print_color!(crate::framebuffer::COLOR_CYAN, "  {:<16}", pkg.name);
            crate::println!(" v{:<8}  {} [user-installed]", pkg.version, pkg.description);
            count += 1;
        }
    }

    crate::println!();
    crate::println!("  {} package(s) installed", count);
}

/// Update catalog (simulated)
pub fn update() {
    crate::println!("  Fetching package index from trust://repo.trustos.dev/...");
    for i in 0..3 {
        for _ in 0..300_000 { core::hint::spin_loop(); }
        crate::print!(".");
    }
    crate::println!();
    crate::println_color!(crate::framebuffer::COLOR_GREEN,
        "  Package catalog updated. {} packages available.", catalog().len());
}

/// Count installed packages (for integration tests)
pub fn installed_count() -> usize {
    let cat = catalog();
    let extra = INSTALLED.lock();
    cat.iter().filter(|p| p.status == PkgStatus::Installed).count() + extra.len()
}

/// Count total packages
pub fn total_count() -> usize {
    catalog().len()
}

/// Check if a package exists in catalog
pub fn package_exists(name: &str) -> bool {
    catalog().iter().any(|p| p.name == name)
}
