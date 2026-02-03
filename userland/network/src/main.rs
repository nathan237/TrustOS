//! Network Service
//! 
//! Async network stack in userland.
//! TCP/IP implementation using smoltcp.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    network_main()
}

fn network_main() -> ! {
    log("NET: Network service starting");
    
    // TODO: Initialize network stack
    // TODO: Setup device driver communication
    
    log("NET: Ready for connections");
    
    loop {
        // TODO: Process network events
        // TODO: Handle socket operations
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
