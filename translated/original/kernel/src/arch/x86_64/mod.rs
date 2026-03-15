//! x86_64 Architecture Implementation
//!
//! Wraps the existing x86_64-specific code into the unified arch interface.
//! This is the primary/reference architecture for TrustOS.

pub mod cpu;
pub mod interrupts;
pub mod serial;
pub mod memory;
pub mod context;
pub mod timer;
pub mod boot;
pub mod syscall_arch;
pub mod io;

/// Halt the CPU until the next interrupt (HLT instruction)
#[inline(always)]
pub fn halt() {
    unsafe {
        core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}
