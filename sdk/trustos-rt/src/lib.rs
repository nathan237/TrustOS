//! TrustOS Runtime Library
//!
//! Provides the entry point and syscall wrappers for TrustOS applications.
//! This is a no_std library for freestanding executables.

#![no_std]

pub mod syscall;
pub mod io;

pub use syscall::*;
pub use io::*;

use core::panic::PanicInfo;

/// Panic handler - required for no_std
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Try to print panic message
    let _ = debug_print_str("PANIC: ");
    if let Some(location) = info.location() {
        let _ = debug_print_str("at ");
        let _ = debug_print_str(location.file());
    }
    let _ = debug_print_str("\n");
    
    exit(1);
}

/// Debug print a string (for panic handler)
fn debug_print_str(s: &str) -> Result<(), ()> {
    syscall::write(2, s.as_bytes()).map(|_| ()).map_err(|_| ())
}
