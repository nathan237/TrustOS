//! Linux subsystem stub for non-x86_64 architectures

use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const LINUX_VM_ID: u64 = 0xFFFF_FFFF_FFFF_0001;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const LINUX_VM_MEMORY_MB: usize = 64;

// Public structure — visible outside this module.
pub struct LinuxSubsystem {
    _private: (),
}

// Public structure — visible outside this module.
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

// Global shared state guarded by a Mutex (mutual exclusion lock).
static SUBSYSTEM: Mutex<LinuxSubsystem> = Mutex::new(LinuxSubsystem { _private: () });

// Public function — callable from other modules.
pub fn subsystem() -> spin::MutexGuard<'static, LinuxSubsystem> {
    SUBSYSTEM.lock()
}

// Public function — callable from other modules.
pub fn init() -> Result<(), String> {
    Err(String::from("Linux subsystem not available on this architecture"))
}

// Public function — callable from other modules.
pub fn boot() -> Result<(), String> {
    Err(String::from("Linux subsystem not available on this architecture"))
}

// Public function — callable from other modules.
pub fn shutdown() -> Result<(), String> {
    Err(String::from("Linux subsystem not available on this architecture"))
}

// Public function — callable from other modules.
pub fn state() -> LinuxState {
    LinuxState::NotStarted
}

// Public function — callable from other modules.
pub fn execute(command: &str) -> Result<CommandResult, String> {
    let _ = command;
    Err(String::from("Linux subsystem not available on this architecture"))
}

// Implementation block — defines methods for the type above.
impl LinuxSubsystem {
        // Public function — callable from other modules.
pub fn set_embedded_images(&mut self, _kernel: &'static [u8], _initramfs: &'static [u8]) {}
        // Public function — callable from other modules.
pub fn is_running(&self) -> bool { false }
        // Public function — callable from other modules.
pub fn execute_command(&mut self, _command: &str) -> Result<String, String> {
        Err(String::from("Not available"))
    }
        // Public function — callable from other modules.
pub fn has_kernel(&self) -> bool { false }
        // Public function — callable from other modules.
pub fn kernel_size(&self) -> usize { 0 }
        // Public function — callable from other modules.
pub fn has_initramfs(&self) -> bool { false }
        // Public function — callable from other modules.
pub fn initramfs_size(&self) -> usize { 0 }
        // Public function — callable from other modules.
pub fn kernel_version_string(&self) -> Option<String> { None }
        // Public function — callable from other modules.
pub fn boot_protocol_version(&self) -> Option<(u8, u8)> { None }
        // Public function — callable from other modules.
pub fn is_package_installed(&self, _name: &str) -> bool { false }
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum LinuxState {
    NotStarted,
    Booting,
    Ready,
    Busy,
    Error,
    ShuttingDown,
}
