

use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

pub const AGI_: u64 = 0xFFFF_FFFF_FFFF_0001;
pub const WB_: usize = 64;

pub struct LinuxSubsystem {
    _private: (),
}

pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

static Apv: Mutex<LinuxSubsystem> = Mutex::new(LinuxSubsystem { _private: () });

pub fn acs() -> spin::MutexGuard<'static, LinuxSubsystem> {
    Apv.lock()
}

pub fn init() -> Result<(), String> {
    Err(String::from("Linux subsystem not available on this architecture"))
}

pub fn boot() -> Result<(), String> {
    Err(String::from("Linux subsystem not available on this architecture"))
}

pub fn shutdown() -> Result<(), String> {
    Err(String::from("Linux subsystem not available on this architecture"))
}

pub fn state() -> LinuxState {
    LinuxState::NotStarted
}

pub fn execute(command: &str) -> Result<CommandResult, String> {
    let _ = command;
    Err(String::from("Linux subsystem not available on this architecture"))
}

impl LinuxSubsystem {
    pub fn set_embedded_images(&mut self, _kernel: &'static [u8], _initramfs: &'static [u8]) {}
    pub fn is_running(&self) -> bool { false }
    pub fn aav(&mut self, _cmd: &str) -> Result<String, String> {
        Err(String::from("Not available"))
    }
    pub fn has_kernel(&self) -> bool { false }
    pub fn kernel_size(&self) -> usize { 0 }
    pub fn has_initramfs(&self) -> bool { false }
    pub fn initramfs_size(&self) -> usize { 0 }
    pub fn kernel_version_string(&self) -> Option<String> { None }
    pub fn boot_protocol_version(&self) -> Option<(u8, u8)> { None }
    pub fn is_package_installed(&self, _name: &str) -> bool { false }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinuxState {
    NotStarted,
    Booting,
    Ready,
    Busy,
    Error,
    ShuttingDown,
}
