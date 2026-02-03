//! TrustOS GUI System
//!
//! Modern Windows 11-like desktop experience with:
//! - 60 FPS VSync with CPU sleep (HLT)
//! - Rounded corners and drop shadows
//! - Fluent Design System colors
//! - Alt+Tab window switcher
//! - Start Menu
//! - Window snapping (Win+Arrow)
//! - Toast notifications
//! - Contextual cursors
//! - Keyboard shortcuts

pub mod engine;
pub mod windows11;

pub use engine::*;
pub use windows11::*;

/// Initialize the GUI system
pub fn init() {
    engine::init_timing();
    crate::serial_println!("[GUI] TrustOS GUI Engine initialized (Windows 11 style)");
}

pub fn get_framebuffer_addr() -> u64 {
    crate::framebuffer::get_fb_addr() as u64
}
