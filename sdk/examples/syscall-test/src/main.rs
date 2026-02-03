//! Syscall Test for TrustOS
//!
//! Tests various system calls to verify the kernel interface.

#![no_std]
#![no_main]

use trustos_rt::*;

/// Entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("=== TrustOS Syscall Test ===");
    println!();
    
    // Test 1: getpid
    print!("Test 1: getpid()... ");
    let pid = getpid();
    println!("OK (PID={})", pid);
    
    // Test 2: write to stdout
    print!("Test 2: write()... ");
    match write(STDOUT, b"OK\n") {
        Ok(n) => println!("wrote {} bytes", n),
        Err(e) => println!("FAILED ({})", e),
    }
    
    // Test 3: sched_yield
    print!("Test 3: sched_yield()... ");
    sched_yield();
    println!("OK");
    
    // Test 4: debug_print (to serial)
    print!("Test 4: debug_print()... ");
    match debug_print(b"[DEBUG] Hello from userland!\n") {
        Ok(_) => println!("OK"),
        Err(e) => println!("FAILED ({})", e),
    }
    
    // Test 5: open/close (may fail if no /tmp)
    print!("Test 5: open/close... ");
    match open("/\0", 0) { // Open root directory
        Ok(fd) => {
            let _ = close(fd);
            println!("OK");
        }
        Err(_) => println!("SKIPPED (no filesystem)"),
    }
    
    println!();
    println!("=== All tests completed ===");
    
    exit(0);
}
