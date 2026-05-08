//! APIC stub for non-x86_64 architectures
//!
//! Provides constants and no-op functions matching the real apic module API.

pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIMER_VECTOR: u8 = 48;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SPURIOUS_VEC: u8 = 0xFF;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IPI_VECTOR: u8 = 0xFE;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const INTERRUPT_REQUEST_BASE: u8 = 49;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KEYBOARD_VECTOR: u8 = 50;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MOUSE_VECTOR: u8 = 61;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VIRTIO_VECTOR: u8 = 62;

// Fonction publique — appelable depuis d'autres modules.
pub fn init() -> bool {
    #[cfg(target_arch = "aarch64")]
    { return crate::arch::platform::gic::is_initialized(); }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}
// Fonction publique — appelable depuis d'autres modules.
pub fn initialize_ap() {}
// Fonction publique — appelable depuis d'autres modules.
pub fn lapic_eoi() {
    #[cfg(target_arch = "aarch64")]
    if crate::arch::platform::gic::is_initialized() {
        crate::arch::platform::gic::eoi(0); // generic EOI
    }
}
// Fonction publique — appelable depuis d'autres modules.
pub fn lapic_id() -> u32 { 0 }
// Fonction publique — appelable depuis d'autres modules.
pub fn start_timer(_interval_ms: u64) {
    #[cfg(target_arch = "aarch64")]
    if crate::arch::platform::gic::is_initialized() {
        crate::arch::platform::gic::rearm_timer(_interval_ms);
    }
}
// Fonction publique — appelable depuis d'autres modules.
pub fn stop_timer() {}
// Fonction publique — appelable depuis d'autres modules.
pub fn send_ipi(_target_apic_id: u32, _vector: u8) {}
// Fonction publique — appelable depuis d'autres modules.
pub fn send_ipi_all_others(_vector: u8) {}
// Fonction publique — appelable depuis d'autres modules.
pub fn is_enabled() -> bool {
    #[cfg(target_arch = "aarch64")]
    { return crate::arch::platform::gic::is_initialized(); }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}
// Fonction publique — appelable depuis d'autres modules.
pub fn ticks_per_mouse() -> u64 { 0 }
// Fonction publique — appelable depuis d'autres modules.
pub fn route_pci_interrupt_request(_irq: u8, _vector: u8) {}
