//! x86_64 Boot / Early Init
//!
//! Platform-specific early initialization that runs before the generic kernel.

/// Early platform init — called before anything else
/// On x86_64: enable NXE, set up basic CPU features
pub fn early_init() {
    // Enable NX bit for page table security
    super::memory::enable_nx();
}

/// Limine boot protocol is used on x86_64
/// The actual Limine request handling is in main.rs since it's
/// the same crate (limine supports x86_64 and aarch64)
pub const BOOTLOADER: &str = "Limine";
