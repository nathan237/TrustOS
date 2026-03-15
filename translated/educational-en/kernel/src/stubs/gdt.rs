//! GDT stub for non-x86_64 architectures
//!
//! Provides segment selector constants and no-op functions.

pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const KERNEL_DATA_SELECTOR: u16 = 0x10;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const USER_DATA_SELECTOR: u16 = 0x1B;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const USER_CODE_SELECTOR: u16 = 0x23;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const TSS_SELECTOR: u16 = 0x28;

// Public function — callable from other modules.
pub fn init() {}
// Public function — callable from other modules.
pub fn initialize_ap(_cpu_id: u32) {}
// Public function — callable from other modules.
pub fn set_kernel_stack(_stack_top: u64) {}
// Public function — callable from other modules.
pub fn current_ring() -> u8 { 0 }
// Public function — callable from other modules.
pub fn is_kernel_mode() -> bool { true }
// Public function — callable from other modules.
pub fn is_user_mode() -> bool { false }
