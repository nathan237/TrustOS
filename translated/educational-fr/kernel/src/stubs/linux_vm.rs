//! Linux VM stub for non-x86_64 architectures

use alloc::string::String;

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct LinuxVmConfig {
    pub memory_mb: usize,
    pub cmdline: String,
    pub vcpus: u32,
    pub serial_console: bool,
    pub virtio_console: bool,
}

// Implémentation de trait — remplit un contrat comportemental.
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

// Structure publique — visible à l'extérieur de ce module.
pub struct LinuxVm;

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl LinuxVm {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new(_config: LinuxVmConfig) -> Result<Self, String> {
        Err(String::from("Linux VM not available on this architecture"))
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn boot(&mut self, _bzimage: &[u8], _initramfs: &[u8]) -> Result<(), String> {
        Err(String::from("Linux VM not available on this architecture"))
    }
}
