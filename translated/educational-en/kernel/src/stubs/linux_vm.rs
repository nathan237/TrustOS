//! Linux VM stub for non-x86_64 architectures

use alloc::string::String;

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone)]
// Public structure — visible outside this module.
pub struct LinuxVmConfig {
    pub memory_mb: usize,
    pub cmdline: String,
    pub vcpus: u32,
    pub serial_console: bool,
    pub virtio_console: bool,
}

// Trait implementation — fulfills a behavioral contract.
impl Default for LinuxVmConfig {
    fn default() -> Self {
        Self {
            memory_mb: 64,
            cmdline: String::new(),
            vcpus: 1,
            serial_console: false,
            virtio_console: false,
        }
    }
}

// Public structure — visible outside this module.
pub struct LinuxVm;

// Implementation block — defines methods for the type above.
impl LinuxVm {
        // Public function — callable from other modules.
pub fn new(_config: LinuxVmConfig) -> Result<Self, String> {
        Err(String::from("Linux VM not available on this architecture"))
    }

        // Public function — callable from other modules.
pub fn boot(&mut self, _bzimage: &[u8], _initramfs: &[u8]) -> Result<(), String> {
        Err(String::from("Linux VM not available on this architecture"))
    }
}
