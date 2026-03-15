

use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

pub const AEO_: u64 = 0xFFFF_FFFF_FFFF_0001;
pub const US_: usize = 64;

pub struct LinuxSubsystem {
    qdl: (),
}

pub struct CommandResult {
    pub ejc: String,
    pub dwg: String,
    pub nz: i32,
}

static Cmh: Mutex<LinuxSubsystem> = Mutex::new(LinuxSubsystem { qdl: () });

pub fn bcu() -> spin::Aki<'static, LinuxSubsystem> {
    Cmh.lock()
}

pub fn init() -> Result<(), String> {
    Err(String::from("Linux subsystem not available on this architecture"))
}

pub fn boot() -> Result<(), String> {
    Err(String::from("Linux subsystem not available on this architecture"))
}

pub fn cbu() -> Result<(), String> {
    Err(String::from("Linux subsystem not available on this architecture"))
}

pub fn g() -> LinuxState {
    LinuxState::Ma
}

pub fn bna(ro: &str) -> Result<CommandResult, String> {
    let _ = ro;
    Err(String::from("Linux subsystem not available on this architecture"))
}

impl LinuxSubsystem {
    pub fn piw(&mut self, yah: &'static [u8], qcg: &'static [u8]) {}
    pub fn dsi(&self) -> bool { false }
    pub fn azu(&mut self, xyg: &str) -> Result<String, String> {
        Err(String::from("Not available"))
    }
    pub fn oaq(&self) -> bool { false }
    pub fn bvc(&self) -> usize { 0 }
    pub fn oao(&self) -> bool { false }
    pub fn jaa(&self) -> usize { 0 }
    pub fn oht(&self) -> Option<String> { None }
    pub fn mzu(&self) -> Option<(u8, u8)> { None }
    pub fn ogm(&self, blu: &str) -> bool { false }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinuxState {
    Ma,
    Agt,
    At,
    Rq,
    Q,
    Ays,
}
