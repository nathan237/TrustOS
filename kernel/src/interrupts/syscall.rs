//! SYSCALL/SYSRET Handler
//! 
//! The actual SYSCALL entry point lives in `userland.rs` which configures
//! STAR, LSTAR, and SFMASK MSRs.  This module provides the Rust-side
//! handler (`syscall_handler_rust`) that the assembly trampoline calls,
//! plus a reference copy of a minimal naked entry point (not active).

// Imports kept for the reference syscall_entry below; not used by init().
#[allow(unused_imports)]
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask, Star};
#[allow(unused_imports)]
use x86_64::registers::rflags::RFlags;
#[allow(unused_imports)]
use x86_64::VirtAddr;

/// Initialize SYSCALL/SYSRET mechanism
///
/// NOTE: EFER, STAR, LSTAR, and SFMASK are all configured by
/// `userland::init()` which sets up the full SYSCALL/SYSRET path
/// including the correct entry point and segment selectors.
/// This function is intentionally a no-op; it exists only so the
/// rest of the codebase doesn't need to know that detail.
pub fn init() {
    // All MSR configuration is handled by userland::init().
    // The syscall_entry in this file is kept as a reference but is not used;
    // userland.rs::syscall_entry is the active LSTAR target.
    crate::log!("SYSCALL handler ready (entry point set by userland::init)");
}

/// Syscall entry point - called by SYSCALL instruction
/// 
/// Register convention (System V AMD64 ABI for syscalls):
/// - RAX = syscall number
/// - RDI = arg1
/// - RSI = arg2
/// - RDX = arg3
/// - R10 = arg4 (RCX is clobbered by SYSCALL)
/// - R8  = arg5
/// - R9  = arg6
/// - RAX = return value
#[unsafe(naked)]
unsafe extern "C" fn syscall_entry() {
    core::arch::naked_asm!(
        // Save user RCX (contains user RIP) and R11 (contains user RFLAGS)
        "push rcx",          // User RIP
        "push r11",          // User RFLAGS
        
        // Save callee-saved registers we'll use
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        
        // Shuffle Linux syscall ABI → C calling convention
        //
        // Linux:  rax=num, rdi=a1, rsi=a2, rdx=a3, r10=a4, r8=a5, r9=a6
        // C call: rdi=num, rsi=a1, rdx=a2, rcx=a3, r8=a4, r9=a5, [rsp]=a6
        //
        // Push a6 (r9) onto the stack as the 7th C argument, then
        // reshuffle the remaining six registers without clobbering.

        "push r9",                // 7th C arg = a6 (on stack)

        "mov r15, r8",            // save a5 (r8 is both source & dest)
        "mov r12, rdi",           // save a1
        "mov r13, rsi",           // save a2
        "mov r14, rdx",           // save a3

        "mov rdi, rax",           // num  → rdi (1st)
        "mov rsi, r12",           // a1   → rsi (2nd)
        "mov rdx, r13",           // a2   → rdx (3rd)
        "mov rcx, r14",           // a3   → rcx (4th)
        "mov r8,  r10",           // a4   → r8  (5th)
        "mov r9,  r15",           // a5   → r9  (6th)
        
        // Call the Rust handler
        "call {handler}",
        
        // Clean up pushed a6
        "add rsp, 8",
        
        // Result is in RAX - will be returned to user
        
        // Restore callee-saved registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        
        // Restore user RFLAGS and RIP
        "pop r11",           // User RFLAGS
        "pop rcx",           // User RIP
        
        // Return to userspace
        "sysretq",
        
        handler = sym syscall_handler_rust,
    );
}

/// Rust syscall handler - called from assembly entry point
///
/// Receives all 7 values: syscall number + 6 Linux arguments.
/// The first six arrive in registers (C convention), the seventh
/// (arg6) is passed on the stack by the assembly trampoline.
#[no_mangle]
pub extern "C" fn syscall_handler_rust(
    num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> u64 {
    // Forward all arguments to the full handler
    let ret = crate::syscall::handle_full(num, arg1, arg2, arg3, arg4, arg5, arg6);

    // Emit structured syscall event to TrustLab trace bus
    crate::lab_mode::trace_bus::emit_syscall(num, [arg1, arg2, arg3], ret);

    ret as u64
}
