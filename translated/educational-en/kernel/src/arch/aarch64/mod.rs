//! aarch64 (ARM64) Architecture Implementation
//!
//! Support for ARMv8-A processors (Raspberry Pi 4/5, Apple Silicon, QEMU virt).
//! Uses Limine bootloader (supports aarch64 UEFI).

pub mod cpu;
pub mod interrupts;
pub mod serial;
pub mod memory;
pub mod context;
pub mod timer;
pub mod boot;
pub mod syscall_arch;
pub mod gic;
pub mod vectors;

/// Halt the CPU until the next interrupt (WFI — Wait For Interrupt)
#[inline(always)]
// Public function — callable from other modules.
pub fn halt() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
    }
}
