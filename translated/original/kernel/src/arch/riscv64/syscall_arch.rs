//! RISC-V 64 System Call Architecture
//!
//! RISC-V uses ECALL instruction for system calls.
//! In S-mode, ECALL from U-mode generates an Environment Call exception
//! routed through the trap vector (stvec).

/// Initialize syscall mechanism on RISC-V
///
/// On RISC-V, system calls use ECALL instruction which generates a
/// synchronous exception handled via the stvec trap vector.
/// No special MSR-like setup needed (unlike x86_64 SYSCALL).
pub fn init_syscall(_handler_addr: u64) {
    // ECALL-based syscalls are handled through the trap vector.
    // The stvec setup is done in the interrupt initialization.
}
