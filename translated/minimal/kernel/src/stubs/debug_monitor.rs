

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
pub fn akj(
    _vm_id: u64,
    _category: DebugCategory,
    _address: u64,
    _status: HandleStatus,
    _ip: u64,
    bek: usize,
    _detail: &str,
) {}
pub fn fdf() -> u64 { 0 }
pub fn fdw() -> u64 { 0 }
pub fn fym() -> String { String::from("Debug monitor not available") }
pub fn fyr() -> String { String::new() }
pub fn ibo() -> String { String::new() }
pub fn ibq() -> String { String::new() }
pub fn ibz(_count: usize) -> String { String::new() }
pub fn jfj(_enabled: bool) {}
