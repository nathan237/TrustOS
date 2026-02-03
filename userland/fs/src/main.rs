//! Filesystem Service
//! 
//! Userland filesystem service using IPC.
//! Handles: open, read, write, close, mkdir, rm

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    fs_main()
}

fn fs_main() -> ! {
    log("FS: Filesystem service starting");
    
    // TODO: Initialize in-memory filesystem
    // TODO: Register with supervisor
    
    log("FS: Ready for requests");
    
    loop {
        // TODO: Receive IPC messages
        // TODO: Handle filesystem operations
        syscall_yield();
    }
}

fn log(msg: &str) {
    let _ = msg;
}

fn syscall_yield() {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 6u64,
        );
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
