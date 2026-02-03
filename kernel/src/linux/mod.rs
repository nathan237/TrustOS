//! Linux Subsystem for TrustOS
//!
//! This module provides a Linux-compatible environment within TrustOS,
//! allowing execution of Linux commands and management of a Linux rootfs.

pub mod rootfs;
pub mod shell;
pub mod tar;
pub mod busybox;

use alloc::string::String;
use spin::Mutex;

/// Linux subsystem state
pub struct LinuxSubsystem {
    /// Is the subsystem initialized?
    initialized: bool,
    /// Root filesystem path (in TrustOS ramfs)
    rootfs_path: String,
    /// Current working directory (relative to rootfs)
    cwd: String,
    /// Hostname
    hostname: String,
    /// Username (always root for now)
    username: String,
}

impl LinuxSubsystem {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            rootfs_path: String::new(),
            cwd: String::new(),
            hostname: String::new(),
            username: String::new(),
        }
    }
    
    pub fn init(&mut self, rootfs_path: &str) -> Result<(), &'static str> {
        self.rootfs_path = String::from(rootfs_path);
        self.cwd = String::from("/root");
        self.hostname = String::from("alpine");
        self.username = String::from("root");
        self.initialized = true;
        Ok(())
    }
    
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    pub fn cwd(&self) -> &str {
        &self.cwd
    }
    
    pub fn set_cwd(&mut self, path: &str) {
        self.cwd = String::from(path);
    }
    
    pub fn hostname(&self) -> &str {
        &self.hostname
    }
    
    pub fn username(&self) -> &str {
        &self.username
    }
    
    pub fn rootfs_path(&self) -> &str {
        &self.rootfs_path
    }
}

/// Global Linux subsystem instance
static LINUX: Mutex<LinuxSubsystem> = Mutex::new(LinuxSubsystem::new());

/// Initialize the Linux subsystem with an Alpine rootfs
pub fn init(rootfs_tarball: &str) -> Result<(), &'static str> {
    crate::println!();
    crate::println_color!(0x00FFFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(0x00FFFF, "║           Installing Linux Subsystem...                      ║");
    crate::println_color!(0x00FFFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    
    // Step 1: Create Linux rootfs directory structure
    crate::print!("[1/4] Creating Linux filesystem structure... ");
    rootfs::create_structure()?;
    crate::println_color!(0x00FF00, "OK");
    
    // Step 2: Extract Alpine rootfs
    crate::print!("[2/4] Extracting Alpine rootfs... ");
    let extracted = tar::extract_tarball(rootfs_tarball, "/linux")?;
    crate::println_color!(0x00FF00, "OK ({} files)", extracted);
    
    // Step 3: Setup essential files
    crate::print!("[3/4] Configuring system... ");
    rootfs::setup_essential_files()?;
    crate::println_color!(0x00FF00, "OK");
    
    // Step 4: Initialize subsystem
    crate::print!("[4/4] Initializing subsystem... ");
    LINUX.lock().init("/linux")?;
    crate::println_color!(0x00FF00, "OK");
    
    crate::println!();
    crate::println_color!(0x00FF00, "        Linux Subsystem Ready!");
    crate::println!();
    
    Ok(())
}

/// Check if Linux subsystem is initialized
pub fn is_initialized() -> bool {
    LINUX.lock().is_initialized()
}

/// Start the Linux shell
pub fn start_shell() {
    if !is_initialized() {
        crate::println_color!(0xFF6600, "Linux subsystem not installed.");
        crate::println!("Run 'gui install' first to download and install Alpine Linux.");
        return;
    }
    
    shell::run();
}

/// Execute a Linux command
pub fn execute(cmd: &str) -> Result<(), &'static str> {
    shell::execute_command(cmd)
}

/// Access the Linux subsystem
pub fn with_linux<F, R>(f: F) -> R
where
    F: FnOnce(&mut LinuxSubsystem) -> R,
{
    f(&mut LINUX.lock())
}
