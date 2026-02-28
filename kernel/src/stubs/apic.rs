//! APIC stub for non-x86_64 architectures
//!
//! Provides constants and no-op functions matching the real apic module API.

pub const TIMER_VECTOR: u8 = 48;
pub const SPURIOUS_VEC: u8 = 0xFF;
pub const IPI_VECTOR: u8 = 0xFE;
pub const IRQ_BASE: u8 = 49;
pub const KEYBOARD_VECTOR: u8 = 50;
pub const MOUSE_VECTOR: u8 = 61;
pub const VIRTIO_VECTOR: u8 = 62;

pub fn init() -> bool { false }
pub fn init_ap() {}
pub fn lapic_eoi() {}
pub fn lapic_id() -> u32 { 0 }
pub fn start_timer(_interval_ms: u64) {}
pub fn stop_timer() {}
pub fn send_ipi(_target_apic_id: u32, _vector: u8) {}
pub fn send_ipi_all_others(_vector: u8) {}
pub fn is_enabled() -> bool { false }
pub fn ticks_per_ms() -> u64 { 0 }
pub fn route_pci_irq(_irq: u8, _vector: u8) {}
