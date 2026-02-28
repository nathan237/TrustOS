//! Hypervisor stub for non-x86_64 architectures
//!
//! Hardware virtualization (VMX/SVM) is x86_64-specific.
//! This stub provides the same public API so consumer code compiles.

use alloc::string::String;
use alloc::vec::Vec;

// Re-export submodule stubs
pub mod linux_subsystem;
pub mod linux_vm;
pub mod debug_monitor;
pub mod vmx;
pub mod svm;
pub mod svm_vm;
pub mod vmi;
pub mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuVendor {
    Intel,
    Amd,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmEventType {
    VmExit,
    EptViolation,
    IoAccess,
    MsrAccess,
    Interrupt,
    Other,
}

pub struct VmEvent {
    pub event_type: VmEventType,
    pub vm_id: u64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViolationType {
    Read,
    Write,
    Execute,
}

pub struct EptViolation {
    pub vm_id: u64,
    pub guest_physical: u64,
    pub guest_linear: Option<u64>,
    pub violation_type: ViolationType,
    pub timestamp_ms: u64,
    pub guest_rip: u64,
}

pub fn init() -> Result<(), String> {
    Err(String::from("Hypervisor not available (non-x86_64)"))
}
pub fn shutdown() -> Result<(), String> {
    Err(String::from("Hypervisor not available (non-x86_64)"))
}
pub fn is_enabled() -> bool { false }
pub fn backend_info() -> String { String::from("Hypervisor not available (non-x86_64)") }
pub fn vm_count() -> usize { 0 }
pub fn render_capabilities() -> String { String::new() }
pub fn render_security_status() -> String { String::new() }
pub fn get_events(_count: usize) -> Vec<VmEvent> { Vec::new() }
pub fn vpid_enabled() -> bool { false }
pub fn vpid_count() -> usize { 0 }
pub fn ept_violations() -> u64 { 0 }
pub fn recent_ept_violations(_count: usize) -> Vec<EptViolation> { Vec::new() }
pub fn version() -> &'static str { "N/A" }
pub fn logo() -> &'static str { "" }
pub fn create_vm(_name: &str, _mem_mb: usize) -> Result<u64, String> {
    Err(String::from("Hypervisor not available"))
}
pub fn start_vm_with_guest(_id: u64, _guest: &str) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn stop_vm(_id: u64) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn list_guests() -> Vec<String> { Vec::new() }
pub fn add_mount(_id: u64, _host: &str, _guest: &str, _ro: bool) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn get_console_output(_id: u64) -> String { String::new() }
pub fn inject_console_input(_id: u64, _input: &[u8]) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn cpu_vendor() -> CpuVendor { CpuVendor::Unknown }
pub fn detect_cpu_vendor() -> CpuVendor { CpuVendor::Unknown }
