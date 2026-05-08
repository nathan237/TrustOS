//! Non-x86 stub for signals subsystem.
//! POSIX signal delivery requires architecture-specific CpuContext register
//! access (rax/rip/rflags). On aarch64/riscv64 we expose a no-op API matching
//! the public surface used elsewhere in the kernel.

#![allow(dead_code)]

pub const SIG_DFL: u64 = 0;
pub const SIG_IGN: u64 = 1;
pub const SIG_ERR: u64 = u64::MAX;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SigAction {
    pub sa_handler: u64,
    pub sa_flags: u64,
    pub sa_restorer: u64,
    pub sa_mask: u64,
}

impl Default for SigAction {
    fn default() -> Self {
        Self { sa_handler: SIG_DFL, sa_flags: 0, sa_restorer: 0, sa_mask: 0 }
    }
}

pub fn init_process(_pid: u32) {}
pub fn cleanup_process(_pid: u32) {}
pub fn kill(_target_pid: u32, _signo: u32, _sender_pid: u32) -> Result<(), i32> { Err(-38) }
pub fn kill_process_group(_pgid: u32, _signo: u32) -> Result<(), i32> { Err(-38) }
pub fn set_action(_pid: u32, _signo: u32, action: SigAction) -> Result<SigAction, i32> { Ok(action) }
pub fn get_action(_pid: u32, _signo: u32) -> Result<SigAction, i32> { Ok(SigAction::default()) }
pub fn set_mask(_pid: u32, _how: u32, _set: u64, old_set: &mut u64) -> Result<(), i32> {
    *old_set = 0;
    Ok(())
}
pub fn check_signals(_pid: u32) -> Option<u32> { None }
