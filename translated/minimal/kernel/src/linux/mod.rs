




pub mod rootfs;
pub mod shell;
pub mod tar;
pub mod busybox;

use alloc::string::String;
use spin::Mutex;


pub struct LinuxSubsystem {
    
    initialized: bool,
    
    rootfs_path: String,
    
    cwd: String,
    
    hostname: String,
    
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


static Ti: Mutex<LinuxSubsystem> = Mutex::new(LinuxSubsystem::new());


pub fn init(rootfs_tarball: &str) -> Result<(), &'static str> {
    crate::println!();
    crate::n!(0x00FFFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::n!(0x00FFFF, "║           Installing Linux Subsystem...                      ║");
    crate::n!(0x00FFFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    
    
    crate::print!("[1/4] Creating Linux filesystem structure... ");
    rootfs::kzl()?;
    crate::n!(0x00FF00, "OK");
    
    
    crate::print!("[2/4] Extracting Alpine rootfs... ");
    let dpd = tar::ltv(rootfs_tarball, "/linux")?;
    crate::n!(0x00FF00, "OK ({} files)", dpd);
    
    
    crate::print!("[3/4] Configuring system... ");
    rootfs::oqg()?;
    crate::n!(0x00FF00, "OK");
    
    
    crate::print!("[4/4] Initializing subsystem... ");
    Ti.lock().init("/linux")?;
    crate::n!(0x00FF00, "OK");
    
    crate::println!();
    crate::n!(0x00FF00, "        Linux Subsystem Ready!");
    crate::println!();
    
    Ok(())
}


pub fn is_initialized() -> bool {
    Ti.lock().is_initialized()
}


pub fn owk() {
    if !is_initialized() {
        crate::n!(0xFF6600, "Linux subsystem not installed.");
        crate::println!("Run 'gui install' first to download and install Alpine Linux.");
        return;
    }
    
    shell::run();
}


pub fn execute(cmd: &str) -> Result<(), &'static str> {
    shell::aav(cmd)
}


pub fn akr<F, U>(f: F) -> U
where
    F: FnOnce(&mut LinuxSubsystem) -> U,
{
    f(&mut Ti.lock())
}
