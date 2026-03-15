//! Debug monitor stub

use alloc::string::String;
use alloc::vec::Vec;

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum DebugCategory {
    IoPortIn,
    IoPortOut,
    MsrRead,
    MsrWrite,
    Mmio,
    CpuId,
    Interrupt,
    Other,
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum HandleStatus {
    Handled,
    Unhandled,
}

// Fonction publique — appelable depuis d'autres modules.
pub fn init() {}
// Fonction publique — appelable depuis d'autres modules.
pub fn stop() {}
// Fonction publique — appelable depuis d'autres modules.
pub fn reset() {}
// Fonction publique — appelable depuis d'autres modules.
pub fn is_initialized() -> bool { false }
// Fonction publique — appelable depuis d'autres modules.
pub fn is_active() -> bool { false }
// Fonction publique — appelable depuis d'autres modules.
pub fn record_event(
    _vm_id: u64,
    _category: DebugCategory,
    _address: u64,
    _status: HandleStatus,
    _ip: u64,
    _size: usize,
    _detail: &str,
) {}
// Fonction publique — appelable depuis d'autres modules.
pub fn total_events() -> u64 { 0 }
// Fonction publique — appelable depuis d'autres modules.
pub fn unhandled_count() -> u64 { 0 }
// Fonction publique — appelable depuis d'autres modules.
pub fn get_dashboard() -> String { String::from("Debug monitor not available") }
// Fonction publique — appelable depuis d'autres modules.
pub fn get_gaps_report() -> String { String::new() }
// Fonction publique — appelable depuis d'autres modules.
pub fn get_io_heatmap() -> String { String::new() }
// Fonction publique — appelable depuis d'autres modules.
pub fn get_msr_report() -> String { String::new() }
// Fonction publique — appelable depuis d'autres modules.
pub fn get_timeline(_count: usize) -> String { String::new() }
// Fonction publique — appelable depuis d'autres modules.
pub fn set_serial_log(_enabled: bool) {}
