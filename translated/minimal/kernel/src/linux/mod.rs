




pub mod rootfs;
pub mod shell;
pub mod tar;
pub mod busybox;

use alloc::string::String;
use spin::Mutex;


pub struct LinuxSubsystem {
    
    jr: bool,
    
    grc: String,
    
    jv: String,
    
    ajc: String,
    
    ox: String,
}

impl LinuxSubsystem {
    pub const fn new() -> Self {
        Self {
            jr: false,
            grc: String::new(),
            jv: String::new(),
            ajc: String::new(),
            ox: String::new(),
        }
    }
    
    pub fn init(&mut self, grc: &str) -> Result<(), &'static str> {
        self.grc = String::from(grc);
        self.jv = String::from("/root");
        self.ajc = String::from("alpine");
        self.ox = String::from("root");
        self.jr = true;
        Ok(())
    }
    
    pub fn ky(&self) -> bool {
        self.jr
    }
    
    pub fn jv(&self) -> &str {
        &self.jv
    }
    
    pub fn wiq(&mut self, path: &str) {
        self.jv = String::from(path);
    }
    
    pub fn ajc(&self) -> &str {
        &self.ajc
    }
    
    pub fn ox(&self) -> &str {
        &self.ox
    }
    
    pub fn grc(&self) -> &str {
        &self.grc
    }
}


static Aus: Mutex<LinuxSubsystem> = Mutex::new(LinuxSubsystem::new());


pub fn init(waa: &str) -> Result<(), &'static str> {
    crate::println!();
    crate::h!(0x00FFFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::h!(0x00FFFF, "║           Installing Linux Subsystem...                      ║");
    crate::h!(0x00FFFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    
    
    crate::print!("[1/4] Creating Linux filesystem structure... ");
    rootfs::rqt()?;
    crate::h!(0x00FF00, "OK");
    
    
    crate::print!("[2/4] Extracting Alpine rootfs... ");
    let hir = tar::sqk(waa, "/linux")?;
    crate::h!(0x00FF00, "OK ({} files)", hir);
    
    
    crate::print!("[3/4] Configuring system... ");
    rootfs::wkr()?;
    crate::h!(0x00FF00, "OK");
    
    
    crate::print!("[4/4] Initializing subsystem... ");
    Aus.lock().init("/linux")?;
    crate::h!(0x00FF00, "OK");
    
    crate::println!();
    crate::h!(0x00FF00, "        Linux Subsystem Ready!");
    crate::println!();
    
    Ok(())
}


pub fn ky() -> bool {
    Aus.lock().ky()
}


pub fn wtd() {
    if !ky() {
        crate::h!(0xFF6600, "Linux subsystem not installed.");
        crate::println!("Run 'gui install' first to download and install Alpine Linux.");
        return;
    }
    
    shell::vw();
}


pub fn bna(cmd: &str) -> Result<(), &'static str> {
    shell::azu(cmd)
}


pub fn bsz<G, Ac>(bb: G) -> Ac
where
    G: FnOnce(&mut LinuxSubsystem) -> Ac,
{
    bb(&mut Aus.lock())
}
