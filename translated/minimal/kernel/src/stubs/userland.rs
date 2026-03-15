



use core::sync::atomic::AtomicU64;

pub const DAD_: u64 = 0x0000_7FFF_FFFF_0000;
pub const AJN_: usize = 1024 * 1024;
pub const DAB_: u64 = 0x0000_0000_0040_0000;

pub static mut YD_: u64 = 0;
pub static mut YC_: u64 = 0;
pub static mut YB_: u64 = 0;
pub static mut AHX_: u64 = 0;
pub static mut NT_: u64 = 0;
pub static LN_: AtomicU64 = AtomicU64::new(0);

pub fn init() {}

pub unsafe fn uau(jxw: u64, jyl: u64) -> ! {
    panic!("jump_to_ring3 not implemented for this architecture");
}

pub unsafe fn ohk(
    jxw: u64, jyl: u64, xxu: u64, xxv: u64,
) -> ! {
    panic!("jump_to_ring3_with_args not implemented for this architecture");
}

pub fn oen() {}

pub unsafe fn eqa(jxw: u64, jyl: u64) -> i32 {
    -1 
}

pub unsafe fn ctw(qbw: i32) -> ! {
    panic!("return_from_ring3 not implemented for this architecture");
}

pub fn jbp() -> bool { false }

pub unsafe fn udc(jxw: u64, jyl: u64) -> i32 {
    -1 
}

pub fn mop(qbw: i32) {}
