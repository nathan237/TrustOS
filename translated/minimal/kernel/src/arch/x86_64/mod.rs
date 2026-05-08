




pub mod cpu;
pub mod interrupts;
pub mod serial;
pub mod memory;
pub mod context;
pub mod timer;
pub mod boot;
pub mod syscall_arch;
pub mod io;


#[inline(always)]
pub fn acb() {
    unsafe {
        core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}
