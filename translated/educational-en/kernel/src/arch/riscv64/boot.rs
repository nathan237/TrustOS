//! RISC-V 64 Boot / Early Init
//!
//! Platform-specific early initialization for RISC-V.

/// Early platform init for RISC-V 64
pub fn early_initialize() {
    // Enable supervisor-mode external, timer, and software interrupts
    unsafe {
        let sie = super::cpu::read_sie();
        super::cpu::write_sie(
            sie | super::cpu::sie_bits::SEIE
                | super::cpu::sie_bits::STIE
                | super::cpu::sie_bits::SSIE
        );
    }
}

/// Limine boot protocol is used on riscv64 (UEFI)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const BOOTLOADER: &str = "Limine";
