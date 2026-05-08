





use alloc::string::String;
use alloc::vec::Vec;


pub mod linux_subsystem;
pub mod linux_vm;
pub mod debug_monitor;
pub mod vmx;
pub mod svm;
pub mod svm_vm;
pub mod vmi;
pub mod tests;


#[cfg(target_arch = "aarch64")]
#[path = "../hypervisor/arm_hv/mod.rs"]
pub mod arm_hv;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuVendor {
    Intel,
    Amd,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmEventType {
    VmExit,
    Ev,
    IoAccess,
    MsrAccess,
    Interrupt,
    Other,
}

pub struct Je {
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

pub struct Ev {
    pub vm_id: u64,
    pub guest_physical: u64,
    pub drb: Option<u64>,
    pub violation_type: ViolationType,
    pub timestamp_ms: u64,
    pub guest_rip: u64,
}

pub fn init() -> Result<(), String> {
    #[cfg(target_arch = "aarch64")]
    {
        if arm_hv::cll() {
            return Ok(());
        }
        return Err(String::from("ARM EL2 not available (not booted at EL2)"));
    }
    #[cfg(not(target_arch = "aarch64"))]
    Err(String::from("Hypervisor not available (non-x86_64)"))
}
pub fn shutdown() -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn lq() -> bool {
    #[cfg(target_arch = "aarch64")]
    { arm_hv::cll() }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}
pub fn fhy() -> String {
    #[cfg(target_arch = "aarch64")]
    {
        if arm_hv::cll() {
            return String::from("ARM EL2 Hypervisor (Stage-2 MMIO Spy)");
        }
    }
    String::from("Hypervisor not available")
}
pub fn vm_count() -> usize { 0 }
pub fn eyj() -> String { String::new() }
pub fn eyk() -> String { String::new() }
pub fn ibl(_count: usize) -> Vec<Je> { Vec::new() }
pub fn csm() -> bool { false }
pub fn jqo() -> usize { 0 }
pub fn ept_violations() -> u64 { 0 }
pub fn iys(_count: usize) -> Vec<Ev> { Vec::new() }
pub fn version() -> &'static str { "N/A" }
pub fn logo() -> &'static str { "" }
pub fn blh(_name: &str, _mem_mb: usize) -> Result<u64, String> {
    Err(String::from("Hypervisor not available"))
}
pub fn dev(_id: u64, _guest: &str) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn fbu(_id: u64) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn dtj() -> Vec<String> { Vec::new() }
pub fn add_mount(_id: u64, _host: &str, _guest: &str, _ro: bool) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn eoa(_id: u64) -> String { String::new() }
pub fn gct(_id: u64, _input: &[u8]) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn cpu_vendor() -> CpuVendor { CpuVendor::Unknown }
pub fn blt() -> CpuVendor { CpuVendor::Unknown }
