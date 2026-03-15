

use alloc::string::String;

#[derive(Clone)]
pub struct Pw {
    pub afc: usize,
    pub wx: String,
    pub jvj: u32,
    pub gsc: bool,
    pub virtio_console: bool,
}

impl Default for Pw {
    fn default() -> Self {
        Self {
            afc: 64,
            wx: String::new(),
            jvj: 1,
            gsc: false,
            virtio_console: false,
        }
    }
}

pub struct LinuxVm;

impl LinuxVm {
    pub fn new(xyk: Pw) -> Result<Self, String> {
        Err(String::from("Linux VM not available on this architecture"))
    }

    pub fn boot(&mut self, qbl: &[u8], qcg: &[u8]) -> Result<(), String> {
        Err(String::from("Linux VM not available on this architecture"))
    }
}
