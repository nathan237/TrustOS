//! GDT stub for non-x86_64 architectures
//!
//! Provides segment selector constants and no-op functions.

pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
pub const USER_DATA_SELECTOR: u16 = 0x1B;
pub const USER_CODE_SELECTOR: u16 = 0x23;
pub const TSS_SELECTOR: u16 = 0x28;

pub fn init() {}
pub fn init_ap(_cpu_id: u32) {}
pub fn set_kernel_stack(_stack_top: u64) {}
pub fn current_ring() -> u8 { 0 }
pub fn is_kernel_mode() -> bool { true }
pub fn is_user_mode() -> bool { false }
