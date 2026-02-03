//! Init/Supervisor Process
//! 
//! First userland process. Responsibilities:
//! - Start core services (filesystem, network)
//! - Monitor service health
//! - Handle service crashes and restart

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[path = "../../syscall.rs"]
mod syscall;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_main()
}

fn init_main() -> ! {
    log("Init: TRustOs supervisor starting");
    log("Init: Starting services");
    
    let _fs = spawn_service("fs", 0x1000);
    let _net = spawn_service("network", 0x2000);
    let _jarvis = spawn_service("jarvis", 0x3000);
    let _comp = spawn_service("compositor", 0x4000);
    let _shell = spawn_service("shell", 0x5000);
    
    log("Init: All services started");
    
    loop {
        syscall_yield();
    }
}

fn log(msg: &str) {
    let _ = msg;
}

fn spawn_service(name: &str, entry: u64) -> u64 {
    let _ = name;
    syscall::spawn(entry)
}

fn syscall_yield() {
    syscall::yield_cpu();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    log("Init: PANIC in supervisor!");
    loop {}
}
