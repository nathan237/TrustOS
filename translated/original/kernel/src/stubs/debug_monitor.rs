//! Debug monitor stub

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum HandleStatus {
    Handled,
    Unhandled,
}

pub fn init() {}
pub fn stop() {}
pub fn reset() {}
pub fn is_initialized() -> bool { false }
pub fn is_active() -> bool { false }
pub fn record_event(
    _vm_id: u64,
    _category: DebugCategory,
    _address: u64,
    _status: HandleStatus,
    _ip: u64,
    _size: usize,
    _detail: &str,
) {}
pub fn total_events() -> u64 { 0 }
pub fn unhandled_count() -> u64 { 0 }
pub fn get_dashboard() -> String { String::from("Debug monitor not available") }
pub fn get_gaps_report() -> String { String::new() }
pub fn get_io_heatmap() -> String { String::new() }
pub fn get_msr_report() -> String { String::new() }
pub fn get_timeline(_count: usize) -> String { String::new() }
pub fn set_serial_log(_enabled: bool) {}
