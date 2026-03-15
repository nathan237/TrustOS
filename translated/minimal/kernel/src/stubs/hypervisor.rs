





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
    Ef,
    Ct,
    F,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmEventType {
    Dln,
    Lj,
    Daa,
    Dcq,
    Fv,
    Qg,
}

pub struct Uw {
    pub bqo: VmEventType,
    pub fk: u64,
    pub aet: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViolationType {
    Read,
    Write,
    Ahw,
}

pub struct Lj {
    pub fk: u64,
    pub hmc: u64,
    pub hmb: Option<u64>,
    pub igm: ViolationType,
    pub aet: u64,
    pub wb: u64,
}

pub fn init() -> Result<(), String> {
    #[cfg(target_arch = "aarch64")]
    {
        if arm_hv::fma() {
            return Ok(());
        }
        return Err(String::from("ARM EL2 not available (not booted at EL2)"));
    }
    #[cfg(not(target_arch = "aarch64"))]
    Err(String::from("Hypervisor not available (non-x86_64)"))
}
pub fn cbu() -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn zu() -> bool {
    #[cfg(target_arch = "aarch64")]
    { arm_hv::fma() }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}
pub fn kbt() -> String {
    #[cfg(target_arch = "aarch64")]
    {
        if arm_hv::fma() {
            return String::from("ARM EL2 Hypervisor (Stage-2 MMIO Spy)");
        }
    }
    String::from("Hypervisor not available")
}
pub fn dna() -> usize { 0 }
pub fn jma() -> String { String::new() }
pub fn jmb() -> String { String::new() }
pub fn nya(jxu: usize) -> Vec<Uw> { Vec::new() }
pub fn fyk() -> bool { false }
pub fn pyu() -> usize { 0 }
pub fn fhx() -> u64 { 0 }
pub fn pap(jxu: usize) -> Vec<Lj> { Vec::new() }
pub fn dk() -> &'static str { "N/A" }
pub fn logo() -> &'static str { "" }
pub fn dpg(blu: &str, yax: usize) -> Result<u64, String> {
    Err(String::from("Hypervisor not available"))
}
pub fn gte(ddq: u64, qca: &str) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn jru(ddq: u64) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn hpy() -> Vec<String> { Vec::new() }
pub fn elx(ddq: u64, xzu: &str, qca: &str, ycl: bool) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn iwo(ddq: u64) -> String { String::new() }
pub fn leo(ddq: u64, yaa: &[u8]) -> Result<(), String> {
    Err(String::from("Hypervisor not available"))
}
pub fn avo() -> CpuVendor { CpuVendor::F }
pub fn dpw() -> CpuVendor { CpuVendor::F }
