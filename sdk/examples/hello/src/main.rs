//! Hello World for TrustOS
//!
//! A minimal example demonstrating TrustOS SDK usage.

#![no_std]
#![no_main]

use trustos_rt::*;

/// Entry point - called by the OS loader
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Print hello message
    println!("Hello from TrustOS!");
    println!("This is a user-space application.");
    println!();
    
    // Show process ID
    let pid = getpid();
    println!("My PID: {}", pid);
    
    // Exit successfully
    exit(0);
}
