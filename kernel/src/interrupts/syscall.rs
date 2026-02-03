//! SYSCALL/SYSRET Handler
//! 
//! Sets up the SYSCALL instruction for userland -> kernel transitions.
//! Uses MSRs to configure entry point and segments.
//!
//! NOTE: For now, we just log syscalls since we don't have Ring 3 yet.
//! When userland is ready, we'll set up proper SYSCALL/SYSRET with GDT.

use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

/// Initialize SYSCALL/SYSRET mechanism
pub fn init() {
    unsafe {
        // Enable SYSCALL/SYSRET in EFER MSR
        let efer = Efer::read();
        Efer::write(efer | EferFlags::SYSTEM_CALL_EXTENSIONS);
        
        // Set LSTAR MSR (syscall entry point)
        // This is the address the CPU jumps to on SYSCALL instruction
        LStar::write(VirtAddr::new(syscall_entry as *const () as u64));
        
        // Set SFMASK MSR (RFLAGS to clear on syscall)
        // Clear interrupt flag to disable interrupts during syscall entry
        SFMask::write(RFlags::INTERRUPT_FLAG | RFlags::DIRECTION_FLAG);
        
        // NOTE: STAR MSR not set - we'll configure GDT properly when implementing Ring 3
        // For now, syscalls from kernel space will work via LSTAR
    }
    
    crate::log!("SYSCALL handler initialized (kernel mode only)");
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
        
        // Call Rust syscall handler
        // Arguments already in correct registers: rdi, rsi, rdx, r10, r8, r9
        // Move r10 to rcx for C calling convention
        "mov rcx, r10",
        // rax = syscall number, becomes first arg after shift
        // We need: handler(syscall_num, arg1, arg2, arg3, arg4, arg5, arg6)
        // So: rdi=num, rsi=arg1, rdx=arg2, rcx=arg3, r8=arg4, r9=arg5
        // But we have: rax=num, rdi=arg1, rsi=arg2, rdx=arg3, r10=arg4, r8=arg5, r9=arg6
        
        // Shuffle: move args, put syscall num in rdi
        "mov r12, rdi",      // Save arg1
        "mov rdi, rax",      // syscall_num -> rdi (first arg)
        "mov r13, rsi",      // Save arg2
        "mov rsi, r12",      // arg1 -> rsi (second arg)
        "mov r14, rdx",      // Save arg3
        "mov rdx, r13",      // arg2 -> rdx (third arg)
        "mov rcx, r14",      // arg3 -> rcx (fourth arg)
        // r8 and r9 are already in position for 5th and 6th args
        
        // Call the Rust handler
        "call {handler}",
        
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
#[no_mangle]
pub extern "C" fn syscall_handler_rust(
    num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    _arg4: u64,
    _arg5: u64,
) -> u64 {
    // Forward to the syscall module
    crate::syscall::handle(num, arg1, arg2, arg3)
}
