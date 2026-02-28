//! x86_64 CPU primitives
//!
//! CPU identification, control registers, and low-level operations.

/// Read the stack pointer (RSP)
#[inline(always)]
pub fn read_stack_pointer() -> u64 {
    let rsp: u64;
    unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) rsp, options(nomem, nostack, preserves_flags));
    }
    rsp
}

/// Read the frame pointer (RBP)
#[inline(always)]
pub fn read_frame_pointer() -> u64 {
    let rbp: u64;
    unsafe {
        core::arch::asm!("mov {}, rbp", out(reg) rbp, options(nomem, nostack, preserves_flags));
    }
    rbp
}

/// Read RFLAGS register
#[inline(always)]
pub fn read_flags() -> u64 {
    let flags: u64;
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
pub fn io_wait() {
    unsafe {
        core::arch::asm!("out 0x80, al", in("al") 0u8, options(nomem, nostack, preserves_flags));
    }
}

/// Software breakpoint (INT 3)
#[inline(always)]
pub fn breakpoint() {
    unsafe {
        core::arch::asm!("int3", options(nomem, nostack));
    }
}

/// Read a Model-Specific Register (MSR)
#[inline(always)]
pub unsafe fn rdmsr(msr: u32) -> u64 {
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
pub unsafe fn wrmsr(msr: u32, value: u64) {
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
pub unsafe fn read_cr0() -> u64 {
    let val: u64;
    core::arch::asm!("mov {}, cr0", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}

/// Write CR0
#[inline(always)]
pub unsafe fn write_cr0(val: u64) {
    core::arch::asm!("mov cr0, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read CR2 (page fault linear address)
#[inline(always)]
pub unsafe fn read_cr2() -> u64 {
    let val: u64;
    core::arch::asm!("mov {}, cr2", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}

/// Read CR3 (page table base)
#[inline(always)]
pub unsafe fn read_cr3() -> u64 {
    let val: u64;
    core::arch::asm!("mov {}, cr3", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}

/// Write CR3 (page table base)
#[inline(always)]
pub unsafe fn write_cr3(val: u64) {
    core::arch::asm!("mov cr3, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Read CR4
#[inline(always)]
pub unsafe fn read_cr4() -> u64 {
    let val: u64;
    core::arch::asm!("mov {}, cr4", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}

/// Write CR4
#[inline(always)]
pub unsafe fn write_cr4(val: u64) {
    core::arch::asm!("mov cr4, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}

/// Execute CPUID instruction
#[inline(always)]
pub unsafe fn cpuid(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
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
pub fn rdtsc() -> u64 {
    let (high, low): (u32, u32);
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
pub fn rdrand() -> Option<u64> {
    let val: u64;
    let ok: u8;
    unsafe {
        core::arch::asm!(
            "rdrand {}",
            "setc {}",
            out(reg) val,
            out(reg_byte) ok,
            options(nomem, nostack)
        );
    }
    if ok != 0 { Some(val) } else { None }
}

// ============================================================================
// I/O Port Access (x86-specific)
// ============================================================================

/// Read a byte from an I/O port
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") val,
        options(nomem, nostack, preserves_flags)
    );
    val
}

/// Write a byte to an I/O port
#[inline(always)]
pub unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}

/// Read a word from an I/O port
#[inline(always)]
pub unsafe fn inw(port: u16) -> u16 {
    let val: u16;
    core::arch::asm!(
        "in ax, dx",
        in("dx") port,
        out("ax") val,
        options(nomem, nostack, preserves_flags)
    );
    val
}

/// Write a word to an I/O port
#[inline(always)]
pub unsafe fn outw(port: u16, val: u16) {
    core::arch::asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") val,
        options(nomem, nostack, preserves_flags)
    );
}

/// Read a dword from an I/O port
#[inline(always)]
pub unsafe fn inl(port: u16) -> u32 {
    let val: u32;
    core::arch::asm!(
        "in eax, dx",
        in("dx") port,
        out("eax") val,
        options(nomem, nostack, preserves_flags)
    );
    val
}

/// Write a dword to an I/O port
#[inline(always)]
pub unsafe fn outl(port: u16, val: u32) {
    core::arch::asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") val,
        options(nomem, nostack, preserves_flags)
    );
}

/// MSR constants
pub mod msr {
    pub const IA32_EFER: u32 = 0xC0000080;
    pub const IA32_STAR: u32 = 0xC0000081;
    pub const IA32_LSTAR: u32 = 0xC0000082;
    pub const IA32_FMASK: u32 = 0xC0000084;
    pub const IA32_FS_BASE: u32 = 0xC0000100;
    pub const IA32_GS_BASE: u32 = 0xC0000101;
    pub const IA32_KERNEL_GS_BASE: u32 = 0xC0000102;
    pub const IA32_APIC_BASE: u32 = 0x1B;
    pub const IA32_PAT: u32 = 0x277;
    
    /// EFER bits
    pub const EFER_SCE: u64 = 1 << 0;  // SYSCALL Enable
    pub const EFER_LME: u64 = 1 << 8;  // Long Mode Enable
    pub const EFER_LMA: u64 = 1 << 10; // Long Mode Active
    pub const EFER_NXE: u64 = 1 << 11; // No-Execute Enable
}
