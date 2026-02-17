//! Userland Support - Ring 3 Execution
//!
//! Provides infrastructure for running user-mode processes:
//! - SYSCALL/SYSRET configuration with STAR MSR
//! - Ring 3 entry mechanism (IRETQ)
//! - Context switching between Ring 0 and Ring 3
//! - Stack management for user processes

use crate::gdt::{KERNEL_CODE_SELECTOR, KERNEL_DATA_SELECTOR, USER_CODE_SELECTOR, USER_DATA_SELECTOR};
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

/// User stack top (below kernel space)
pub const USER_STACK_TOP: u64 = 0x0000_7FFF_FFFF_0000;
/// User stack size (1 MB)
pub const USER_STACK_SIZE: usize = 1024 * 1024;
/// User code start address
pub const USER_CODE_BASE: u64 = 0x0000_0000_0040_0000;

/// Initialize userland support
/// 
/// Configures SYSCALL/SYSRET mechanism for Ring 3 ↔ Ring 0 transitions.
/// Must be called AFTER GDT is initialized with Ring 3 segments.
pub fn init() {
    unsafe {
        // Enable SYSCALL/SYSRET in EFER MSR
        let efer = Efer::read();
        Efer::write(efer | EferFlags::SYSTEM_CALL_EXTENSIONS);
        
        // Configure STAR MSR for segment selectors
        // STAR layout:
        // - Bits 63:48 = SYSRET CS and SS (user segments)
        // - Bits 47:32 = SYSCALL CS and SS (kernel segments)
        // 
        // On SYSCALL:  CS = STAR[47:32], SS = STAR[47:32] + 8
        // On SYSRET:   CS = STAR[63:48] + 16, SS = STAR[63:48] + 8
        //
        // Our GDT layout:
        // 0x00: null, 0x08: kernel code, 0x10: kernel data, 0x18: user code, 0x20: user data
        // 
        // For SYSCALL: we want CS=0x08, SS=0x10 → set bits [47:32] = 0x08
        // For SYSRET:  we want CS=0x1B (0x18|3), SS=0x23 (0x20|3)
        //   SYSRET loads CS = STAR[63:48] + 16, so STAR[63:48] = 0x18 - 16 = 0x08
        //   But wait, SYSRET 64-bit: CS = STAR[63:48] + 16, SS = STAR[63:48] + 8
        //   We need CS=0x18|3=0x1B and SS=0x20|3=0x23
        //   So STAR[63:48] should be 0x18 - 16 = 0x08? No...
        //
        // Actually for 64-bit SYSRET:
        //   CS.sel = STAR[63:48] + 16  (and RPL forced to 3)
        //   SS.sel = STAR[63:48] + 8   (and RPL forced to 3)
        //
        // We want: CS = 0x18 (user code), SS = 0x20 (user data)
        //   CS: 0x18 = STAR[63:48] + 16 → STAR[63:48] = 0x08
        //   SS: 0x20 = STAR[63:48] + 8 = 0x08 + 8 = 0x10 ← WRONG! We get 0x10, not 0x20
        //
        // The trick: GDT must be ordered as:
        //   kernel code, kernel data, user data, user code
        // Then STAR[63:48] = user_data - 8
        //
        // OR we use the standard layout and set STAR correctly:
        // Our layout: null(0), kcode(0x08), kdata(0x10), ucode(0x18), udata(0x20)
        //
        // For SYSRET to work: we need udata at offset ucode-8
        // So we swap: null(0), kcode(0x08), kdata(0x10), udata(0x18), ucode(0x20)
        //
        // Let's keep our current layout and note that we need to update selectors:
        // Actually, the x86_64 crate handles this. Let's use Star::write_raw:
        
        // STAR format: user_base (for sysret) | kernel_base (for syscall)
        // kernel_base = 0x08 (kernel code at 0x08, kernel data at 0x10)
        // user_base = needs (user_code - 16) since sysret adds 16
        //
        // Our current GDT: kcode=0x08, kdata=0x10, ucode=0x18, udata=0x20
        // For sysret64: CS = base + 16 = 0x18, SS = base + 8 = 0x10 (wrong!)
        //
        // We need to reorder GDT to: kcode, kdata, udata, ucode (0x08, 0x10, 0x18, 0x20)
        // Then: CS = base + 16 = 0x08 + 16 = 0x18 (wrong order now)
        //
        // Standard solution: put udata before ucode
        // GDT: null, kcode(0x08), kdata(0x10), udata(0x18), ucode(0x20)
        // user_base = 0x10 (0x18 - 8)
        // sysret: CS = 0x10 + 16 = 0x20 (user code), SS = 0x10 + 8 = 0x18 (user data) ✓
        
        // Write STAR MSR directly
        // bits 47:32 = kernel CS selector = 0x08
        // bits 63:48 = sysret base = 0x10 (so CS = 0x10+16=0x20, SS = 0x10+8=0x18)
        let star_value: u64 = (0x10u64 << 48) | (0x08u64 << 32);
        core::arch::asm!(
            "wrmsr",
            in("ecx") 0xC0000081u32, // IA32_STAR
            in("eax") star_value as u32,
            in("edx") (star_value >> 32) as u32,
        );
        
        // Set LSTAR MSR (syscall entry point)
        LStar::write(VirtAddr::new(syscall_entry as *const () as u64));
        
        // Set SFMASK MSR (RFLAGS to clear on syscall)
        // Clear IF (interrupts), DF (direction), TF (trap), AC (alignment check)
        SFMask::write(
            RFlags::INTERRUPT_FLAG | 
            RFlags::DIRECTION_FLAG | 
            RFlags::TRAP_FLAG |
            RFlags::ALIGNMENT_CHECK
        );
    }
    
    crate::log!("[USERLAND] SYSCALL/SYSRET configured (STAR, LSTAR, SFMASK)");
}

/// Jump to Ring 3 and execute user code
/// 
/// This uses IRETQ to transition from Ring 0 to Ring 3.
/// The stack is set up with the interrupt frame format:
/// - SS (user data selector with RPL 3)
/// - RSP (user stack pointer)
/// - RFLAGS (with IF set for interrupts)
/// - CS (user code selector with RPL 3)
/// - RIP (user code entry point)
/// 
/// # Safety
/// The caller must ensure:
/// - `entry_point` is a valid user-space address with executable code
/// - `user_stack` points to valid, mapped user-space stack memory
/// - The user address space is properly set up (CR3)
#[inline(never)]
pub unsafe fn jump_to_ring3(entry_point: u64, user_stack: u64) -> ! {
    // User code selector: 0x20 with RPL 3 = 0x23
    // User data selector: 0x18 with RPL 3 = 0x1B
    const USER_CS: u64 = 0x20 | 3; // 0x23
    const USER_SS: u64 = 0x18 | 3; // 0x1B
    
    // RFLAGS: enable interrupts (IF=1), reserved bit 1 must be set
    const USER_RFLAGS: u64 = 0x202; // IF=1, reserved=1
    
    crate::log_debug!("[USERLAND] Jumping to Ring 3: RIP={:#x}, RSP={:#x}", entry_point, user_stack);
    
    core::arch::asm!(
        // Push interrupt frame for IRETQ
        "push {ss}",        // SS
        "push {rsp}",       // RSP
        "push {rflags}",    // RFLAGS
        "push {cs}",        // CS
        "push {rip}",       // RIP
        
        // Clear all general purpose registers for security
        "xor rax, rax",
        "xor rbx, rbx",
        "xor rcx, rcx",
        "xor rdx, rdx",
        "xor rsi, rsi",
        "xor rdi, rdi",
        "xor rbp, rbp",
        "xor r8, r8",
        "xor r9, r9",
        "xor r10, r10",
        "xor r11, r11",
        "xor r12, r12",
        "xor r13, r13",
        "xor r14, r14",
        "xor r15, r15",
        
        // Jump to Ring 3
        "iretq",
        
        ss = in(reg) USER_SS,
        rsp = in(reg) user_stack,
        rflags = in(reg) USER_RFLAGS,
        cs = in(reg) USER_CS,
        rip = in(reg) entry_point,
        options(noreturn)
    );
}

/// Jump to Ring 3 with arguments (for passing argc, argv to user main)
#[inline(never)]
pub unsafe fn jump_to_ring3_with_args(entry_point: u64, user_stack: u64, arg1: u64, arg2: u64) -> ! {
    const USER_CS: u64 = 0x20 | 3;
    const USER_SS: u64 = 0x18 | 3;
    const USER_RFLAGS: u64 = 0x202;
    
    crate::log_debug!("[USERLAND] Jumping to Ring 3: RIP={:#x}, RSP={:#x}, args=({}, {:#x})", 
        entry_point, user_stack, arg1, arg2);
    
    core::arch::asm!(
        // Push interrupt frame for IRETQ
        "push {ss}",
        "push {rsp}",
        "push {rflags}",
        "push {cs}",
        "push {rip}",
        
        // Set up arguments in RDI and RSI (System V ABI)
        "mov rdi, {arg1}",
        "mov rsi, {arg2}",
        
        // Clear other registers
        "xor rax, rax",
        "xor rbx, rbx",
        "xor rcx, rcx",
        "xor rdx, rdx",
        "xor rbp, rbp",
        "xor r8, r8",
        "xor r9, r9",
        "xor r10, r10",
        "xor r11, r11",
        "xor r12, r12",
        "xor r13, r13",
        "xor r14, r14",
        "xor r15, r15",
        
        "iretq",
        
        ss = in(reg) USER_SS,
        rsp = in(reg) user_stack,
        rflags = in(reg) USER_RFLAGS,
        cs = in(reg) USER_CS,
        rip = in(reg) entry_point,
        arg1 = in(reg) arg1,
        arg2 = in(reg) arg2,
        options(noreturn)
    );
}

/// Syscall entry point (naked function)
/// 
/// This is called by the SYSCALL instruction from Ring 3.
/// On entry:
/// - RCX = user RIP (return address)
/// - R11 = user RFLAGS
/// - RDI, RSI, RDX, R10, R8, R9 = syscall arguments
/// - RAX = syscall number
/// 
/// We must:
/// 1. Switch to kernel stack (from TSS.RSP0)
/// 2. Save user context
/// 3. Call Rust handler
/// 4. Restore context and SYSRETQ
#[unsafe(naked)]
extern "C" fn syscall_entry() {
    core::arch::naked_asm!(
        // SYSCALL has loaded:
        // - RCX = user RIP
        // - R11 = user RFLAGS
        // 
        // We're still on user stack! Need to switch to kernel stack.
        // Save user RSP to a temp variable (NOT a register — we must
        // preserve the user's callee-saved regs R12-R15 across syscalls).
        
        // Save user RSP in a temp variable
        "mov [rip + {user_rsp_temp}], rsp",
        
        // Load kernel stack
        "mov rsp, [rip + {kernel_stack}]",
        
        // Push user context from saved location
        "push QWORD PTR [rip + {user_rsp_temp}]",   // User RSP
        "push rcx",          // User RIP (saved by SYSCALL)
        "push r11",          // User RFLAGS (saved by SYSCALL)
        
        // Push callee-saved registers (user's original values, unmodified!)
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        
        // Arguments are in: RAX=num, RDI=a1, RSI=a2, RDX=a3, R10=a4, R8=a5, R9=a6
        // C calling convention: RDI, RSI, RDX, RCX, R8, R9, [rsp]=7th
        // We need: handler(num, a1, a2, a3, a4, a5, a6)
        
        // Push a6 (r9) onto the stack as the 7th C argument
        "push r9",
        
        // Shuffle the remaining six register arguments without clobbering
        "mov r15, r8",       // save a5
        "mov r12, rdi",      // save a1
        "mov r13, rsi",      // save a2
        "mov r14, rdx",      // save a3
        
        "mov rdi, rax",      // num  -> rdi (1st)
        "mov rsi, r12",      // a1   -> rsi (2nd)
        "mov rdx, r13",      // a2   -> rdx (3rd)
        "mov rcx, r14",      // a3   -> rcx (4th)
        "mov r8,  r10",      // a4   -> r8  (5th)
        "mov r9,  r15",      // a5   -> r9  (6th)
        
        // Call Rust handler
        "call {handler}",
        
        // Clean up pushed a6
        "add rsp, 8",
        
        // Result is in RAX
        
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
        "pop rsp",           // User RSP
        
        // Return to Ring 3
        "sysretq",
        
        kernel_stack = sym KERNEL_SYSCALL_STACK_TOP,
        user_rsp_temp = sym USER_RSP_TEMP,
        handler = sym crate::interrupts::syscall::syscall_handler_rust,
    );
}

/// Temporary storage for user RSP during syscall entry (before stack switch)
#[no_mangle]
static mut USER_RSP_TEMP: u64 = 0;

/// Kernel stack for syscall handling (separate from interrupt stacks)
static mut KERNEL_SYSCALL_STACK: [u8; 65536] = [0; 65536]; // 64 KB
/// Pointer to top of kernel syscall stack
#[no_mangle]
static mut KERNEL_SYSCALL_STACK_TOP: u64 = 0;

/// Initialize kernel syscall stack
pub fn init_syscall_stack() {
    unsafe {
        let stack_bottom = KERNEL_SYSCALL_STACK.as_ptr() as u64;
        KERNEL_SYSCALL_STACK_TOP = stack_bottom + 65536;
        crate::log_debug!("[USERLAND] Syscall stack at {:#x}", KERNEL_SYSCALL_STACK_TOP);
    }
}

// Note: syscall_handler_rust is defined in interrupts/syscall.rs
// The syscall_entry function above references it via sym

// ───────────────────────────────────────────────────────
// Ring 3 Process Execution with Return-to-Kernel Support
// ───────────────────────────────────────────────────────

/// Saved kernel RSP for returning from Ring 3
static mut KERNEL_RETURN_RSP: u64 = 0;
/// Saved kernel return point (RIP) for returning from Ring 3
static mut KERNEL_RETURN_RIP: u64 = 0;
/// Whether a Ring 3 process is currently executing
static mut USERLAND_PROCESS_ACTIVE: bool = false;

/// Execute a user process in Ring 3, returning when it calls exit()
///
/// This uses a setjmp/longjmp-style mechanism:
/// 1. Saves kernel context (callee-saved regs + RSP) to statics
/// 2. IRETQ to Ring 3 user code
/// 3. When user calls exit(), `return_from_ring3()` restores kernel context
/// 4. Returns the exit code to the caller
///
/// # Safety
/// The caller must ensure:
/// - User address space is activated (CR3 set)
/// - `entry_point` is mapped and executable in user space
/// - `user_stack` is mapped and writable in user space
/// - Kernel mappings are present in the user page tables
#[inline(never)]
pub unsafe fn exec_ring3_process(entry_point: u64, user_stack: u64) -> i32 {
    const USER_CS: u64 = 0x20 | 3;  // 0x23
    const USER_SS: u64 = 0x18 | 3;  // 0x1B
    const USER_RFLAGS: u64 = 0x202; // IF=1, reserved=1

    let exit_code: i64;
    USERLAND_PROCESS_ACTIVE = true;

    // Compiler barrier: ensure entry_point and user_stack are materialized
    // before the asm block (prevents misoptimization of the inline asm inputs)
    let entry_point = core::hint::black_box(entry_point);
    let user_stack = core::hint::black_box(user_stack);

    core::arch::asm!(
        // Save return point address (label 2f) so return_from_ring3 can jump back
        "lea rax, [rip + 2f]",
        "mov [{return_rip}], rax",

        // Save callee-saved registers on kernel stack
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Save kernel RSP (points to saved callee-saved regs)
        "mov [{return_rsp}], rsp",

        // ── IRETQ frame: SS, RSP, RFLAGS, CS, RIP ──
        "push {ss}",
        "push {user_rsp}",
        "push {rflags}",
        "push {cs}",
        "push {entry}",

        // Clear all GPRs for clean Ring 3 entry
        "xor rax, rax",
        "xor rbx, rbx",
        "xor rcx, rcx",
        "xor rdx, rdx",
        "xor rsi, rsi",
        "xor rdi, rdi",
        "xor rbp, rbp",
        "xor r8, r8",
        "xor r9, r9",
        "xor r10, r10",
        "xor r11, r11",
        "xor r12, r12",
        "xor r13, r13",
        "xor r14, r14",
        "xor r15, r15",

        // ── Enter Ring 3! ──
        "iretq",

        // ╔══════════════════════════════════════════╗
        // ║  Return point — reached via              ║
        // ║  return_from_ring3() on exit() syscall   ║
        // ╚══════════════════════════════════════════╝
        "2:",

        // Restore callee-saved registers (pushed above)
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",

        // RAX already contains exit code (set by return_from_ring3)

        entry = in(reg) entry_point,
        user_rsp = in(reg) user_stack,
        ss = in(reg) USER_SS,
        cs = in(reg) USER_CS,
        rflags = in(reg) USER_RFLAGS,
        return_rsp = sym KERNEL_RETURN_RSP,
        return_rip = sym KERNEL_RETURN_RIP,
        // rax is written early (lea rax) and holds exit code at return
        out("rax") exit_code,
        // Caller-saved regs clobbered by XOR (after inputs consumed)
        lateout("rcx") _,
        lateout("rdx") _,
        lateout("rsi") _,
        lateout("rdi") _,
        lateout("r8") _,
        lateout("r9") _,
        lateout("r10") _,
        lateout("r11") _,
    );

    USERLAND_PROCESS_ACTIVE = false;
    exit_code as i32
}

/// Return from Ring 3 to the kernel.
///
/// Called from the EXIT syscall handler. Restores the kernel context
/// saved by `exec_ring3_process` and jumps back to the return point.
///
/// # Safety
/// Must only be called when `USERLAND_PROCESS_ACTIVE` is true.
pub unsafe fn return_from_ring3(exit_code: i32) -> ! {
    // Restore kernel CR3 (switch back from user address space)
    core::arch::asm!(
        "mov cr3, {cr3}",
        cr3 = in(reg) crate::memory::paging::kernel_cr3(),
        options(nostack, preserves_flags)
    );

    // Restore kernel RSP and jump to the return point in exec_ring3_process
    core::arch::asm!(
        "mov rax, {code}",
        "mov rsp, [{return_rsp}]",
        "sti",          // Re-enable interrupts (disabled by exception/syscall handler)
        "jmp [{return_rip}]",
        code = in(reg) exit_code as i64,
        return_rsp = sym KERNEL_RETURN_RSP,
        return_rip = sym KERNEL_RETURN_RIP,
        options(noreturn)
    );
}

/// Check if a Ring 3 process is currently executing
pub fn is_process_active() -> bool {
    unsafe { USERLAND_PROCESS_ACTIVE }
}
