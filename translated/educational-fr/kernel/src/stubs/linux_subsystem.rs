//! Linux subsystem stub for non-x86_64 architectures

use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LINUX_VM_ID: u64 = 0xFFFF_FFFF_FFFF_0001;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LINUX_VM_MEMORY_MB: usize = 64;

// Structure publique — visible à l'extérieur de ce module.
pub struct LinuxSubsystem {
    _private: (),
}

// Structure publique — visible à l'extérieur de ce module.
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static SUBSYSTEM: Mutex<LinuxSubsystem> = Mutex::new(LinuxSubsystem { _private: () });

// Fonction publique — appelable depuis d'autres modules.
pub fn subsystem() -> spin::MutexGuard<'static, LinuxSubsystem> {
    SUBSYSTEM.lock()
}

// Fonction publique — appelable depuis d'autres modules.
pub fn init() -> Result<(), String> {
    Err(String::from("Linux subsystem not available on this architecture"))
}

// Fonction publique — appelable depuis d'autres modules.
pub fn boot() -> Result<(), String> {
    Err(String::from("Linux subsystem not available on this architecture"))
}

// Fonction publique — appelable depuis d'autres modules.
pub fn shutdown() -> Result<(), String> {
    Err(String::from("Linux subsystem not available on this architecture"))
}

// Fonction publique — appelable depuis d'autres modules.
pub fn state() -> LinuxState {
    LinuxState::NotStarted
}

// Fonction publique — appelable depuis d'autres modules.
pub fn execute(command: &str) -> Result<CommandResult, String> {
    let _ = command;
    Err(String::from("Linux subsystem not available on this architecture"))
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl LinuxSubsystem {
        // Fonction publique — appelable depuis d'autres modules.
pub fn set_embedded_images(&mut self, _kernel: &'static [u8], _initramfs: &'static [u8]) {}
        // Fonction publique — appelable depuis d'autres modules.
pub fn is_running(&self) -> bool { false }
        // Fonction publique — appelable depuis d'autres modules.
pub fn execute_command(&mut self, _command: &str) -> Result<String, String> {
        Err(String::from("Not available"))
    }
        // Fonction publique — appelable depuis d'autres modules.
pub fn has_kernel(&self) -> bool { false }
        // Fonction publique — appelable depuis d'autres modules.
pub fn kernel_size(&self) -> usize { 0 }
        // Fonction publique — appelable depuis d'autres modules.
pub fn has_initramfs(&self) -> bool { false }
        // Fonction publique — appelable depuis d'autres modules.
pub fn initramfs_size(&self) -> usize { 0 }
        // Fonction publique — appelable depuis d'autres modules.
pub fn kernel_version_string(&self) -> Option<String> { None }
        // Fonction publique — appelable depuis d'autres modules.
pub fn boot_protocol_version(&self) -> Option<(u8, u8)> { None }
        // Fonction publique — appelable depuis d'autres modules.
pub fn is_package_installed(&self, _name: &str) -> bool { false }
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum LinuxState {
    NotStarted,
    Booting,
    Ready,
    Busy,
    Error,
    ShuttingDown,
}
