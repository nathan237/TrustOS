












pub mod engine;
pub mod vsync;
pub mod windows11;

pub use engine::*;
pub use windows11::*;


pub fn init() {
    engine::oep();
    crate::serial_println!("[GUI] TrustOS GUI Engine initialized (Windows 11 style)");
}

pub fn ytf() -> u64 {
    crate::framebuffer::iwp() as u64
}
