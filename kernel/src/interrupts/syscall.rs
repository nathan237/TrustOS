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
