






pub mod syscall;
pub mod loader;
pub mod runtime;
pub mod interpreter;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;


pub struct LinuxProcess {
    
    pub ce: u32,
    
    pub path: String,
    
    pub cjc: Vec<String>,
    
    pub epy: Vec<String>,
    
    pub jv: String,
    
    pub den: u64,
    
    pub aho: [Option<u32>; 256],
    
    pub nz: Option<i32>,
}

impl LinuxProcess {
    pub fn new(path: &str, cjc: Vec<String>, epy: Vec<String>) -> Self {
        let mut aho = [None; 256];
        
        aho[0] = Some(0); 
        aho[1] = Some(1); 
        aho[2] = Some(2); 
        
        Self {
            ce: qgy(),
            path: String::from(path),
            cjc,
            epy,
            jv: String::from("/"),
            den: 0,
            aho,
            nz: None,
        }
    }
}

static CHT_: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(1000);

fn qgy() -> u32 {
    CHT_.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
}


pub fn exec(path: &str, n: &[&str]) -> Result<i32, &'static str> {
    crate::serial_println!("[LINUX] Executing: {} {:?}", path, n);
    
    
    let pu = crate::ramfs::fh(|fs| {
        fs.mq(path).map(|bc| bc.ip())
    }).or_else(|_| crate::linux::rootfs::mq(path))?;
    
    
    let mut emg: Vec<&str> = Vec::new();
    emg.push(path);
    for ji in n {
        emg.push(*ji);
    }
    
    
    crate::println!("╔══════════════════════════════════════════════════════════════╗");
    crate::println!("║  Linux Binary Interpreter                                    ║");
    crate::println!("║  Executing: {}                       ", path);
    crate::println!("╚══════════════════════════════════════════════════════════════╝");
    
    match interpreter::peo(&pu, &emg) {
        Ok(nz) => {
            crate::println!("\n[Process exited with code {}]", nz);
            Ok(nz)
        }
        Err(aa) => {
            crate::h!(0xFF0000, "\n[Process error: {}]", aa);
            Err(aa)
        }
    }
}


pub fn tya(path: &str) -> bool {
    if let Ok(f) = crate::linux::rootfs::mq(path) {
        
        f.len() >= 4 && &f[0..4] == b"\x7fELF"
    } else {
        false
    }
}
