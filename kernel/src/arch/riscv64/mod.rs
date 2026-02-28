//! RISC-V 64-bit (RV64GC) Architecture Implementation
//!
//! Support for RISC-V 64-bit processors (QEMU virt, SiFive, StarFive).
//! Uses Limine bootloader (supports riscv64 UEFI).

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
