



use core::sync::atomic::AtomicU64;

pub const DDV_: u64 = 0x0000_7FFF_FFFF_0000;
pub const ALI_: usize = 1024 * 1024;
pub const DDT_: u64 = 0x0000_0000_0040_0000;

pub static mut USER_RSP_TEMP: u64 = 0;
pub static mut USER_RETURN_RIP: u64 = 0;
pub static mut USER_RETURN_RFLAGS: u64 = 0;
pub static mut SIGNAL_DELIVER_SIGNO: u64 = 0;
pub static mut KERNEL_SYSCALL_STACK_TOP: u64 = 0;
pub static MH_: AtomicU64 = AtomicU64::new(0);

pub fn init() {}

pub unsafe fn mvh(_entry_point: u64, _user_stack: u64) -> ! {
    panic!("jump_to_ring3 not implemented for this architecture");
}

pub unsafe fn jump_to_ring3_with_args(
    _entry_point: u64, _user_stack: u64, _arg1: u64, _arg2: u64,
) -> ! {
    panic!("jump_to_ring3_with_args not implemented for this architecture");
}

pub fn igu() {}

pub unsafe fn bzn(_entry_point: u64, _user_stack: u64) -> i32 {
    -1 
}

pub unsafe fn azi(_exit_code: i32) -> ! {
    panic!("return_from_ring3 not implemented for this architecture");
}

pub fn ers() -> bool { false }

pub unsafe fn mxd(_entry_point: u64, _user_stack: u64) -> i32 {
    -1 
}

pub fn haz(_exit_code: i32) {}
