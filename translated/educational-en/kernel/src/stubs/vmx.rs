//! VMX stub
use alloc::string::String;

// Public structure — visible outside this module.
pub struct VmxCapabilities {
    pub supported: bool,
    pub ept_supported: bool,
    pub unrestricted_guest: bool,
    pub vpid_supported: bool,
    pub vmcs_revision_id: u32,
}

// Public function — callable from other modules.
pub fn check_vmx_support() -> Result<VmxCapabilities, String> {
    Err(String::from("VMX not available on this architecture"))
}
