//! VMX stub
use alloc::string::String;

pub struct VmxCapabilities {
    pub supported: bool,
    pub ept_supported: bool,
    pub unrestricted_guest: bool,
    pub vpid_supported: bool,
    pub vmcs_revision_id: u32,
}

pub fn check_vmx_support() -> Result<VmxCapabilities, String> {
    Err(String::from("VMX not available on this architecture"))
}
