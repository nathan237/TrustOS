//! Linux subsystem stub for non-x86_64 architectures

use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

pub const LINUX_VM_ID: u64 = 0xFFFF_FFFF_FFFF_0001;
pub const LINUX_VM_MEMORY_MB: usize = 64;

pub struct LinuxSubsystem {
    _private: (),
}

pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

static SUBSYSTEM: Mutex<LinuxSubsystem> = Mutex::new(LinuxSubsystem { _private: () });

pub fn subsystem() -> spin::MutexGuard<'static, LinuxSubsystem> {
    SUBSYSTEM.lock()
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
    pub fn execute_command(&mut self, _cmd: &str) -> Result<String, String> {
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
