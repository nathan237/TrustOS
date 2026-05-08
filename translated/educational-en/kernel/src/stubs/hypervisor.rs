//! Hypervisor stub for non-x86_64 architectures
//!
//! Hardware virtualization (VMX/SVM) is x86_64-specific.
//! On AArch64, we have our own ARM EL2 hypervisor for MMIO spying.
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

// On AArch64: include the real ARM EL2 Hypervisor module
#[cfg(target_arch = "aarch64")]
#[path = "../hypervisor/arm_hv/mod.rs"]
pub mod arm_hv;

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Enumeration — a type that can be one of several variants.
pub enum CpuVendor {
    Intel,
    Amd,
    Unknown,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum VmEventType {
    VmExit,
    EptViolation,
    IoAccess,
    MsrAccess,
    Interrupt,
    Other,
}

// Public structure — visible outside this module.
pub struct VmEvent {
    pub event_type: VmEventType,
    pub vm_id: u64,
    pub timestamp_ms: u64,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum ViolationType {
    Read,
    Write,
    Execute,
}

// Public structure — visible outside this module.
pub struct EptViolation {
    pub vm_id: u64,
    pub guest_physical: u64,
    pub guest_linear: Option<u64>,
    pub violation_type: ViolationType,
    pub timestamp_ms: u64,
    pub guest_rip: u64,
}

// Public function — callable from other modules.
pub fn init() -> Result<(), String> {
    #[cfg(target_arch = "aarch64")]
    {
        if arm_hv::is_el2() {
            return Ok(());
        }
        return Err(String::from("ARM EL2 not available (not booted at EL2)"));
    }
    #[cfg(not(target_arch = "aarch64"))]
    Err(String::from("Hypervisor not available (non-x86_64)"))
}
// Public function — callable from other modules.
pub fn shutdown() -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
// Public function — callable from other modules.
pub fn is_enabled() -> bool {
    #[cfg(target_arch = "aarch64")]
    { arm_hv::is_el2() }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}
// Public function — callable from other modules.
pub fn backend_information() -> String {
    #[cfg(target_arch = "aarch64")]
    {
        if arm_hv::is_el2() {
            return String::from("ARM EL2 Hypervisor (Stage-2 MMIO Spy)");
        }
    }
    String::from("Hypervisor not available")
}
// Public function — callable from other modules.
pub fn vm_count() -> usize { 0 }
// Public function — callable from other modules.
pub fn render_capabilities() -> String { String::new() }
// Public function — callable from other modules.
pub fn render_security_status() -> String { String::new() }
// Public function — callable from other modules.
pub fn get_events(_count: usize) -> Vec<VmEvent> { Vec::new() }
// Public function — callable from other modules.
pub fn vpid_enabled() -> bool { false }
// Public function — callable from other modules.
pub fn vpid_count() -> usize { 0 }
// Public function — callable from other modules.
pub fn ept_violations() -> u64 { 0 }
// Public function — callable from other modules.
pub fn recent_ept_violations(_count: usize) -> Vec<EptViolation> { Vec::new() }
// Public function — callable from other modules.
pub fn version() -> &'static str { "N/A" }
// Public function — callable from other modules.
pub fn logo() -> &'static str { "" }
// Public function — callable from other modules.
pub fn create_vm(_name: &str, _mem_mb: usize) -> Result<u64, String> {
    Err(String::from("Hypervisor not available"))
}
// Public function — callable from other modules.
pub fn start_vm_with_guest(_id: u64, _guest: &str) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
// Public function — callable from other modules.
pub fn stop_vm(_id: u64) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
// Public function — callable from other modules.
pub fn list_guests() -> Vec<String> { Vec::new() }
// Public function — callable from other modules.
pub fn add_mount(_id: u64, _host: &str, _guest: &str, _ro: bool) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
// Public function — callable from other modules.
pub fn get_console_output(_id: u64) -> String { String::new() }
// Public function — callable from other modules.
pub fn inject_console_input(_id: u64, _input: &[u8]) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
// Public function — callable from other modules.
pub fn cpu_vendor() -> CpuVendor { CpuVendor::Unknown }
// Public function — callable from other modules.
pub fn detect_cpu_vendor() -> CpuVendor { CpuVendor::Unknown }
