//! x86_64 SYSCALL/SYSRET Setup
//!
//! Architecture-specific system call entry configuration.
//! The actual syscall handler setup uses MSRs (LSTAR, STAR, SFMASK).

use super::cpu::{self, msr};

/// Initialize SYSCALL/SYSRET MSRs
/// 
/// This configures the CPU so that the `syscall` instruction jumps to the
/// kernel syscall handler with the correct segment selectors.
pub fn init_syscall_msrs(handler_addr: u64) {
    unsafe {
        // Enable SCE (System Call Extensions) in EFER
        let efer = cpu::rdmsr(msr::IA32_EFER);
        cpu::wrmsr(msr::IA32_EFER, efer | msr::EFER_SCE);
        
        // STAR: segment selectors for SYSCALL/SYSRET
        // Bits 47:32 = Kernel CS (0x08), bits 63:48 = User CS base (0x10)
        // SYSRET loads CS = STAR[63:48]+16 = 0x20 (user code), SS = STAR[63:48]+8 = 0x18 (user data)
        let star = (0x0008u64 << 32) | (0x0010u64 << 48);
        cpu::wrmsr(msr::IA32_STAR, star);
        
        // LSTAR: syscall entry point
        cpu::wrmsr(msr::IA32_LSTAR, handler_addr);
        
        // SFMASK: flags to clear on SYSCALL (clear IF and DF)
        cpu::wrmsr(msr::IA32_FMASK, 0x200 | 0x400); // IF + DF
    }
}
