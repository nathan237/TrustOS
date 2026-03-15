//! Debug monitor stub

use alloc::string::String;
use alloc::vec::Vec;

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy)]
// Enumeration — a type that can be one of several variants.
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

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy)]
// Enumeration — a type that can be one of several variants.
pub enum HandleStatus {
    Handled,
    Unhandled,
}

// Public function — callable from other modules.
pub fn init() {}
// Public function — callable from other modules.
pub fn stop() {}
// Public function — callable from other modules.
pub fn reset() {}
// Public function — callable from other modules.
pub fn is_initialized() -> bool { false }
// Public function — callable from other modules.
pub fn is_active() -> bool { false }
// Public function — callable from other modules.
pub fn record_event(
    _vm_id: u64,
    _category: DebugCategory,
    _address: u64,
    _status: HandleStatus,
    _ip: u64,
    _size: usize,
    _detail: &str,
) {}
// Public function — callable from other modules.
pub fn total_events() -> u64 { 0 }
// Public function — callable from other modules.
pub fn unhandled_count() -> u64 { 0 }
// Public function — callable from other modules.
pub fn get_dashboard() -> String { String::from("Debug monitor not available") }
// Public function — callable from other modules.
pub fn get_gaps_report() -> String { String::new() }
// Public function — callable from other modules.
pub fn get_io_heatmap() -> String { String::new() }
// Public function — callable from other modules.
pub fn get_msr_report() -> String { String::new() }
// Public function — callable from other modules.
pub fn get_timeline(_count: usize) -> String { String::new() }
// Public function — callable from other modules.
pub fn set_serial_log(_enabled: bool) {}
