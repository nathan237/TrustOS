//! aarch64 System Call Architecture
//!
//! ARM64 uses SVC (Supervisor Call) instruction for system calls.
//! The exception vector table routes SVC to the syscall handler.

/// Initialize syscall mechanism on aarch64
///
/// On ARM64, system calls use the SVC instruction which generates
/// a Synchronous exception routed through the exception vector table.
/// The vector table setup is handled in the interrupt module.
pub fn init_syscall(_handler_addr: u64) {
    // On ARM64, SVC-based syscalls are handled through the exception
    // vector table — no special MSR setup needed (unlike x86_64 SYSCALL).
    // The handler address routing is done in the exception vector table.
}
