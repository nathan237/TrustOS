
use alloc::string::String;

pub struct Nv {
    pub supported: bool,
    pub ept_supported: bool,
    pub unrestricted_guest: bool,
    pub vpid_supported: bool,
    pub vmcs_revision_id: u32,
}

pub fn ehv() -> Result<Nv, String> {
    Err(String::from("VMX not available on this architecture"))
}
