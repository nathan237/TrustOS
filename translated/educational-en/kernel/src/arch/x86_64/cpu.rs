//! x86_64 CPU primitives
//!
//! CPU identification, control registers, and low-level operations.

/// Read the stack pointer (RSP)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_stack_pointer() -> u64 {
    let rsp: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) rsp, options(nomem, nostack, preserves_flags));
    }
    rsp
}

/// Read the frame pointer (RBP)
#[inline(always)]
// Public function — callable from other modules.
pub fn read_frame_pointer() -> u64 {
    let rbp: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("mov {}, rbp", out(reg) rbp, options(nomem, nostack, preserves_flags));
    }
    rbp
}

/// Read RFLAGS register
#[inline(always)]
// Public function — callable from other modules.
pub fn read_flags() -> u64 {
    let flags: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!(
            "pushfq",
            "pop {}",
            out(reg) flags,
            options(nomem, preserves_flags)
        );
    }
    flags
}

/// I/O wait — write to unused port 0x80 to create a small delay
#[inline(always)]
// Public function — callable from other modules.
pub fn io_wait() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("out 0x80, al", in("al") 0u8, options(nomem, nostack, preserves_flags));
    }
}

/// Software breakpoint (INT 3)
#[inline(always)]
// Public function — callable from other modules.
pub fn breakpoint() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("int3", options(nomem, nostack));
    }
}

/// Read a Model-Specific Register (MSR)
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn rdmsr(msr: u32) -> u64 {
    let (high, low): (u32, u32);
    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nomem, nostack, preserves_flags)
    );
    ((high as u64) << 32) | (low as u64)
}

/// Write a Model-Specific Register (MSR)
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack, preserves_flags)
    );
}

/// Read CR0
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn read_cr0() -> u64 {
    let value: u64;
    core::arch::asm!("mov {}, cr0", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

/// Write CR0
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_cr0(value: u64) {
    core::arch::asm!("mov cr0, {}", in(reg) value, options(nomem, nostack, preserves_flags));
}

/// Read CR2 (page fault linear address)
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn read_cr2() -> u64 {
    let value: u64;
    core::arch::asm!("mov {}, cr2", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

/// Read CR3 (page table base)
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn read_cr3() -> u64 {
    let value: u64;
    core::arch::asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

/// Write CR3 (page table base)
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_cr3(value: u64) {
    core::arch::asm!("mov cr3, {}", in(reg) value, options(nomem, nostack, preserves_flags));
}

/// Read CR4
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn read_cr4() -> u64 {
    let value: u64;
    core::arch::asm!("mov {}, cr4", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

/// Write CR4
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write_cr4(value: u64) {
    core::arch::asm!("mov cr4, {}", in(reg) value, options(nomem, nostack, preserves_flags));
}

/// Execute CPUID instruction
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn cpuid(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
    let (eax, ebx, ecx, edx): (u32, u32, u32, u32);
    // rbx is reserved by LLVM, so we must save/restore it manually
    core::arch::asm!(
        "push rbx",
        "cpuid",
        "mov {ebx_out:e}, ebx",
        "pop rbx",
        inout("eax") leaf => eax,
        inout("ecx") subleaf => ecx,
        ebx_out = out(reg) ebx,
        out("edx") edx,
        options(nostack, preserves_flags)
    );
    (eax, ebx, ecx, edx)
}

/// Read Time Stamp Counter (RDTSC)
#[inline(always)]
// Public function — callable from other modules.
pub fn rdtsc() -> u64 {
    let (high, low): (u32, u32);
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// RDRAND - hardware random number
#[inline(always)]
// Public function — callable from other modules.
pub fn rdrand() -> Option<u64> {
    let value: u64;
    let ok: u8;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!(
            "rdrand {}",
            "setc {}",
            out(reg) value,
            out(reg_byte) ok,
            options(nomem, nostack)
        );
    }
    if ok != 0 { Some(value) } else { None }
}

// ============================================================================
// I/O Port Access (x86-specific)
// ============================================================================

/// Read a byte from an I/O port
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nomem, nostack, preserves_flags)
    );
    value
}

/// Write a byte to an I/O port
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

/// Read a word from an I/O port
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    core::arch::asm!(
        "in ax, dx",
        in("dx") port,
        out("ax") value,
        options(nomem, nostack, preserves_flags)
    );
    value
}

/// Write a word to an I/O port
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn outw(port: u16, value: u16) {
    core::arch::asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") value,
        options(nomem, nostack, preserves_flags)
    );
}

/// Read a dword from an I/O port
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    core::arch::asm!(
        "in eax, dx",
        in("dx") port,
        out("eax") value,
        options(nomem, nostack, preserves_flags)
    );
    value
}

/// Write a dword to an I/O port
#[inline(always)]
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn outl(port: u16, value: u32) {
    core::arch::asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") value,
        options(nomem, nostack, preserves_flags)
    );
}

/// MSR constants
pub mod msr {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const IA32_EFER: u32 = 0xC0000080;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const IA32_STAR: u32 = 0xC0000081;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const IA32_LSTAR: u32 = 0xC0000082;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const IA32_FMASK: u32 = 0xC0000084;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const IA32_FILESYSTEM_BASE: u32 = 0xC0000100;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const IA32_GS_BASE: u32 = 0xC0000101;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const IA32_KERNEL_GS_BASE: u32 = 0xC0000102;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const IA32_APIC_BASE: u32 = 0x1B;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const IA32_PAT: u32 = 0x277;
    
    /// EFER bits
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const EFER_SCE: u64 = 1 << 0;  // SYSCALL Enable
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const EFER_LME: u64 = 1 << 8;  // Long Mode Enable
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const EFER_LMA: u64 = 1 << 10; // Long Mode Active
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const EFER_NXE: u64 = 1 << 11; // No-Execute Enable
}
