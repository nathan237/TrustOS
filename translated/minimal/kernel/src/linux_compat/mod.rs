






pub mod syscall;
pub mod loader;
pub mod runtime;
pub mod interpreter;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;


pub struct LinuxProcess {
    
    pub pid: u32,
    
    pub path: String,
    
    pub argv: Vec<String>,
    
    pub bzm: Vec<String>,
    
    pub cwd: String,
    
    pub brk: u64,
    
    pub fds: [Option<u32>; 256],
    
    pub exit_code: Option<i32>,
}

impl LinuxProcess {
    pub fn new(path: &str, argv: Vec<String>, bzm: Vec<String>) -> Self {
        let mut fds = [None; 256];
        
        fds[0] = Some(0); 
        fds[1] = Some(1); 
        fds[2] = Some(2); 
        
        Self {
            pid: juz(),
            path: String::from(path),
            argv,
            bzm,
            cwd: String::from("/"),
            brk: 0,
            fds,
            exit_code: None,
        }
    }
}

static CLC_: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(1000);

fn juz() -> u32 {
    CLC_.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
}


pub fn exec(path: &str, args: &[&str]) -> Result<i32, &'static str> {
    crate::serial_println!("[LINUX] Executing: {} {:?}", path, args);
    
    
    let gz = crate::ramfs::bh(|fs| {
        fs.read_file(path).map(|d| d.to_vec())
    }).or_else(|_| crate::linux::rootfs::read_file(path))?;
    
    
    let mut bxn: Vec<&str> = Vec::new();
    bxn.push(path);
    for db in args {
        bxn.push(*db);
    }
    
    
    crate::println!("╔══════════════════════════════════════════════════════════════╗");
    crate::println!("║  Linux Binary Interpreter                                    ║");
    crate::println!("║  Executing: {}                       ", path);
    crate::println!("╚══════════════════════════════════════════════════════════════╝");
    
    match interpreter::jbu(&gz, &bxn) {
        Ok(exit_code) => {
            crate::println!("\n[Process exited with code {}]", exit_code);
            Ok(exit_code)
        }
        Err(e) => {
            crate::n!(0xFF0000, "\n[Process error: {}]", e);
            Err(e)
        }
    }
}


pub fn msy(path: &str) -> bool {
    if let Ok(data) = crate::linux::rootfs::read_file(path) {
        
        data.len() >= 4 && &data[0..4] == b"\x7fELF"
    } else {
        false
    }
}
