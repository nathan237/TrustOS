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

/// Halt the CPU until the next interrupt (WFI — Wait For Interrupt)
#[inline(always)]
pub fn halt() {
    unsafe {
        core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
    }
}
