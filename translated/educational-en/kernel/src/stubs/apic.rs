//! APIC stub for non-x86_64 architectures
//!
//! Provides constants and no-op functions matching the real apic module API.

pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const TIMER_VECTOR: u8 = 48;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SPURIOUS_VEC: u8 = 0xFF;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const IPI_VECTOR: u8 = 0xFE;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const INTERRUPT_REQUEST_BASE: u8 = 49;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const KEYBOARD_VECTOR: u8 = 50;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MOUSE_VECTOR: u8 = 61;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const VIRTIO_VECTOR: u8 = 62;

// Public function — callable from other modules.
pub fn init() -> bool {
    #[cfg(target_arch = "aarch64")]
    { return crate::arch::platform::gic::is_initialized(); }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}
// Public function — callable from other modules.
pub fn initialize_ap() {}
// Public function — callable from other modules.
pub fn lapic_eoi() {
    #[cfg(target_arch = "aarch64")]
    if crate::arch::platform::gic::is_initialized() {
        crate::arch::platform::gic::eoi(0); // generic EOI
    }
}
// Public function — callable from other modules.
pub fn lapic_id() -> u32 { 0 }
// Public function — callable from other modules.
pub fn start_timer(_interval_ms: u64) {
    #[cfg(target_arch = "aarch64")]
    if crate::arch::platform::gic::is_initialized() {
        crate::arch::platform::gic::rearm_timer(_interval_ms);
    }
}
// Public function — callable from other modules.
pub fn stop_timer() {}
// Public function — callable from other modules.
pub fn send_ipi(_target_apic_id: u32, _vector: u8) {}
// Public function — callable from other modules.
pub fn send_ipi_all_others(_vector: u8) {}
// Public function — callable from other modules.
pub fn is_enabled() -> bool {
    #[cfg(target_arch = "aarch64")]
    { return crate::arch::platform::gic::is_initialized(); }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}
// Public function — callable from other modules.
pub fn ticks_per_mouse() -> u64 { 0 }
// Public function — callable from other modules.
pub fn route_pci_interrupt_request(_irq: u8, _vector: u8) {}
