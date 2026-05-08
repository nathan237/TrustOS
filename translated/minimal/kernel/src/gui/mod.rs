












pub mod engine;
pub mod vsync;
pub mod windows11;

pub use engine::*;
pub use windows11::*;


pub fn init() {
    engine::igw();
    crate::serial_println!("[GUI] TrustOS GUI Engine initialized (Windows 11 style)");
}

pub fn qhv() -> u64 {
    crate::framebuffer::eob() as u64
}
