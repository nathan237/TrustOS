



pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
pub const ALG_: u16 = 0x1B;
pub const ALF_: u16 = 0x23;
pub const TSS_SELECTOR: u16 = 0x28;

pub fn init() {}
pub fn cau(_cpu_id: u32) {}
pub fn jfg(_stack_top: u64) {}
pub fn fpw() -> u8 { 0 }
pub fn msv() -> bool { true }
pub fn mub() -> bool { false }
