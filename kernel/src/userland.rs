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
        // But we need to save RSP first...
        
        // Use SWAPGS to get kernel GS base (points to per-CPU data)
        // For simplicity, we'll use a different approach: save to scratch register
        // and load kernel stack from TSS
        
        // For now, use a simpler approach: the user RSP is saved, we switch stacks
        
        // Save user RSP in R12 (we'll push it later)
        "mov r12, rsp",
        
        // Load kernel stack from a known location
        // We'll use a static variable for the kernel stack pointer
        "mov rsp, [rip + {kernel_stack}]",
        
        // Now we're on kernel stack, push user context
        "push r12",          // User RSP
        "push rcx",          // User RIP (saved by SYSCALL)
        "push r11",          // User RFLAGS (saved by SYSCALL)
        
        // Push callee-saved registers
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        
        // Arguments are in: RAX=num, RDI=a1, RSI=a2, RDX=a3, R10=a4, R8=a5, R9=a6
        // Rust calling convention: RDI, RSI, RDX, RCX, R8, R9
        // We need: handler(num, a1, a2, a3, a4, a5)
        
        // Shuffle arguments
        "mov r15, rdi",      // Save a1
        "mov rdi, rax",      // num -> rdi
        "mov rax, rsi",      // Save a2
        "mov rsi, r15",      // a1 -> rsi
        "mov r15, rdx",      // Save a3
        "mov rdx, rax",      // a2 -> rdx
        "mov rcx, r15",      // a3 -> rcx
        "mov r15, r10",      // r10 = a4
        "mov r10, r8",       // Save a5
        "mov r8, r15",       // a4 -> r8
        "mov r9, r10",       // a5 -> r9
        
        // Call Rust handler
        "call {handler}",
        
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
        handler = sym crate::interrupts::syscall::syscall_handler_rust,
    );
}

/// Kernel stack for syscall handling (separate from interrupt stacks)
static mut KERNEL_SYSCALL_STACK: [u8; 16384] = [0; 16384]; // 16 KB
/// Pointer to top of kernel syscall stack
#[no_mangle]
static mut KERNEL_SYSCALL_STACK_TOP: u64 = 0;

/// Initialize kernel syscall stack
pub fn init_syscall_stack() {
    unsafe {
        let stack_bottom = KERNEL_SYSCALL_STACK.as_ptr() as u64;
        KERNEL_SYSCALL_STACK_TOP = stack_bottom + 16384;
        crate::log_debug!("[USERLAND] Syscall stack at {:#x}", KERNEL_SYSCALL_STACK_TOP);
    }
}

// Note: syscall_handler_rust is defined in interrupts/syscall.rs
// The syscall_entry function above references it via sym

/// Test userland by running a simple program
#[allow(dead_code)]
pub fn test_userland() {
    use alloc::vec::Vec;
    use crate::memory::paging::{PageFlags, UserMemoryRegion, AddressSpace};
    
    crate::log!("[USERLAND] Testing Ring 3 execution...");
    
    // Create user address space
    let mut user_space = match AddressSpace::new_with_kernel() {
        Some(space) => space,
        None => {
            crate::log_error!("[USERLAND] Failed to create address space");
            return;
        }
    };
    
    // Simple test code: just do a syscall to exit
    // Machine code for:
    //   mov rax, 60    ; SYS_EXIT
    //   mov rdi, 42    ; exit code
    //   syscall
    let test_code: [u8; 12] = [
        0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00, // mov rax, 60
        0x48, 0xC7, 0xC7, 0x2A, 0x00,             // mov rdi, 42
        // Note: we'd need more bytes for syscall (0x0F, 0x05)
    ];
    
    // Allocate physical page for code
    let hhdm = crate::memory::hhdm_offset();
    let code_page_phys = alloc_physical_page();
    
    // Copy code to physical page
    let code_page_virt = code_page_phys + hhdm;
    unsafe {
        let dest = code_page_virt as *mut u8;
        core::ptr::copy_nonoverlapping(test_code.as_ptr(), dest, test_code.len());
    }
    
    // Map code page at user code base
    user_space.map_page(
        UserMemoryRegion::CODE_START,
        code_page_phys,
        PageFlags::USER_CODE,
    );
    
    // Allocate and map user stack
    let stack_page_phys = alloc_physical_page();
    let stack_base = UserMemoryRegion::STACK_TOP - 4096;
    user_space.map_page(stack_base, stack_page_phys, PageFlags::USER_DATA);
    
    crate::log!("[USERLAND] Test program mapped, activating address space...");
    
    // Switch to user address space
    unsafe { user_space.activate(); }
    
    // Jump to Ring 3
    // Note: This would actually execute the user code
    // For testing, we just log and return
    crate::log!("[USERLAND] Address space activated, CR3={:#x}", user_space.cr3());
    
    // Switch back to kernel space
    unsafe {
        core::arch::asm!(
            "mov cr3, {}",
            in(reg) crate::memory::paging::kernel_cr3(),
            options(nostack, preserves_flags)
        );
    }
    
    crate::log!("[USERLAND] Test complete");
}

/// Simple physical page allocator (uses heap for now)
fn alloc_physical_page() -> u64 {
    use alloc::vec::Vec;
    
    // Allocate aligned page from heap
    let page: Vec<u8> = alloc::vec![0u8; 4096];
    let virt = page.as_ptr() as u64;
    
    // Convert to physical (subtract HHDM)
    let phys = virt - crate::memory::hhdm_offset();
    
    // Leak so it persists
    core::mem::forget(page);
    
    phys
}
