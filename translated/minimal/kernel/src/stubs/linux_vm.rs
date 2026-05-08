

use alloc::string::String;

#[derive(Clone)]
pub struct Gq {
    pub memory_mb: usize,
    pub cmdline: String,
    pub vcpus: u32,
    pub serial_console: bool,
    pub virtio_console: bool,
}

impl Default for Gq {
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

pub struct LinuxVm;

impl LinuxVm {
    pub fn new(_config: Gq) -> Result<Self, String> {
        Err(String::from("Linux VM not available on this architecture"))
    }

    pub fn boot(&mut self, _bzimage: &[u8], _initramfs: &[u8]) -> Result<(), String> {
        Err(String::from("Linux VM not available on this architecture"))
    }
}
