




pub mod cpu;
pub mod interrupts;
pub mod serial;
pub mod memory;
pub mod context;
pub mod timer;
pub mod boot;
pub mod syscall_arch;
pub mod gic;
pub mod vectors;


#[inline(always)]
pub fn bhd() {
    unsafe {
        core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
    }
}
