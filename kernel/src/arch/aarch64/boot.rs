//! aarch64 Boot / Early Init
//!
//! Platform-specific early initialization for ARM64.

/// Early platform init for aarch64
/// Sets up basic CPU features before the generic kernel runs.
pub fn early_init() {
    // On ARM64, Limine already sets up EL1 with MMU enabled.
    // We just need to ensure caches are on and alignment check is off.
    unsafe {
        let mut sctlr = super::cpu::read_sctlr_el1();
        sctlr |= 1 << 2;   // C — enable data cache
        sctlr |= 1 << 12;  // I — enable instruction cache
        sctlr &= !(1 << 1); // A — disable alignment check (for unaligned access)
        super::cpu::write_sctlr_el1(sctlr);
    }
}

/// Limine boot protocol is used on aarch64 (UEFI)
pub const BOOTLOADER: &str = "Limine";
