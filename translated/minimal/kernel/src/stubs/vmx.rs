
use alloc::string::String;

pub struct Afp {
    pub dme: bool,
    pub fhw: bool,
    pub gvo: bool,
    pub gwj: bool,
    pub igr: u32,
}

pub fn inj() -> Result<Afp, String> {
    Err(String::from("VMX not available on this architecture"))
}
