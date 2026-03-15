//! Architecture Abstraction Layer
//!
//! Provides a unified interface across all supported CPU architectures:
//! - x86_64 (AMD64/Intel 64)
//! - aarch64 (ARM64 / ARMv8-A)
//! - riscv64 (RISC-V 64-bit, RV64GC)
//!
//! Each architecture implements the same public API so the rest of the kernel
//! can remain architecture-independent.

// ============================================================================
// Architecture-specific modules
// ============================================================================

#[cfg(target_arch = "x86_64")]
#[path = "x86_64/mod.rs"]
pub mod platform;

#[cfg(target_arch = "aarch64")]
#[path = "aarch64/mod.rs"]
pub mod platform;

#[cfg(target_arch = "riscv64")]
#[path = "riscv64/mod.rs"]
pub mod platform;

// ============================================================================
// Re-exports — unified arch API used by the rest of the kernel
// ============================================================================

pub use platform::cpu;
pub use platform::interrupts;
pub use platform::serial;
pub use platform::memory;
pub use platform::context;
pub use platform::timer;
pub use platform::boot;
pub use platform::syscall_arch;

// ============================================================================
// Portable convenience functions (delegate to platform)
// ============================================================================

/// Halt the CPU until the next interrupt (HLT / WFI / WFI)
#[inline(always)]
pub fn halt() {
    platform::halt();
}

/// Infinite halt loop — never returns
#[inline(always)]
pub fn halt_loop() -> ! {
    loop {
        platform::halt();
    }
}

/// Enable hardware interrupts (STI / MSR DAIFClr / CSR sie)
#[inline(always)]
pub fn interrupts_enable() {
    platform::interrupts::enable();
}

/// Disable hardware interrupts (CLI / MSR DAIFSet / CSR clear sie)
#[inline(always)]
pub fn interrupts_disable() {
    platform::interrupts::disable();
}

/// Run a closure with interrupts disabled, restoring previous state after
#[inline(always)]
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    platform::interrupts::without_interrupts(f)
}

/// Check if interrupts are currently enabled
#[inline(always)]
pub fn are_interrupts_enabled() -> bool {
    platform::interrupts::are_enabled()
}

/// Flush a single TLB entry for the given virtual address
#[inline(always)]
pub fn flush_tlb(addr: u64) {
    platform::memory::flush_tlb(addr);
}

/// Flush the entire TLB
#[inline(always)]
pub fn flush_tlb_all() {
    platform::memory::flush_tlb_all();
}

/// Read the page table root register (CR3 / TTBR0_EL1 / satp)
#[inline(always)]
pub fn read_page_table_root() -> u64 {
    platform::memory::read_page_table_root()
}

/// Write the page table root register
#[inline(always)]
pub fn write_page_table_root(val: u64) {
    platform::memory::write_page_table_root(val);
}

/// Read the current stack pointer
#[inline(always)]
pub fn read_stack_pointer() -> u64 {
    platform::cpu::read_stack_pointer()
}

/// Insert an I/O wait (for slow devices)
#[inline(always)]
pub fn io_wait() {
    platform::cpu::io_wait();
}

/// Trigger a software breakpoint (INT3 / BRK / EBREAK)
#[inline(always)]
pub fn breakpoint() {
    platform::cpu::breakpoint();
}

/// Get a monotonic timestamp counter value (TSC / CNTPCT / rdcycle)
#[inline(always)]
pub fn timestamp() -> u64 {
    platform::timer::timestamp()
}

/// Architecture name string
pub const fn arch_name() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    { "x86_64" }
    #[cfg(target_arch = "aarch64")]
    { "aarch64" }
    #[cfg(target_arch = "riscv64")]
    { "riscv64gc" }
}

/// Page size for this architecture (all use 4 KiB base pages)
pub const PAGE_SIZE: usize = 4096;

/// Kernel virtual base address (higher half)
pub const KERNEL_VIRT_BASE: u64 = {
    #[cfg(target_arch = "x86_64")]
    { 0xFFFF_FFFF_8000_0000 } // -2 GiB
    #[cfg(target_arch = "aarch64")]
    { 0xFFFF_0000_0000_0000 } // upper VA range (TTBR1)
    #[cfg(target_arch = "riscv64")]
    { 0xFFFF_FFFF_C000_0000 } // Sv48 higher half
};

/// Maximum physical address bits
pub const PHYS_ADDR_BITS: u32 = {
    #[cfg(target_arch = "x86_64")]
    { 52 }
    #[cfg(target_arch = "aarch64")]
    { 48 }
    #[cfg(target_arch = "riscv64")]
    { 56 } // Sv48 uses 56-bit phys
};

// ============================================================================
// Portable I/O Port abstraction
// ============================================================================

/// Portable I/O Port type.
/// On x86_64: real port I/O via IN/OUT instructions.
/// On other architectures: no-op stub (I/O ports don't exist on ARM/RISC-V).
#[cfg(target_arch = "x86_64")]
pub type Port<T> = x86_64::instructions::port::Port<T>;

#[cfg(not(target_arch = "x86_64"))]
pub mod port_stub {
    use core::marker::PhantomData;

    /// Stub Port for non-x86 architectures
    pub struct Port<T: PortValue> {
        _phantom: PhantomData<T>,
        port: u16,
    }

    /// Trait for port-accessible value types
    pub trait PortValue: Copy + Default {}
    impl PortValue for u8 {}
    impl PortValue for u16 {}
    impl PortValue for u32 {}

    impl<T: PortValue> Port<T> {
        pub const fn new(port: u16) -> Self {
            Self { _phantom: PhantomData, port }
        }
        pub unsafe fn read(&mut self) -> T { T::default() }
        pub unsafe fn write(&mut self, _value: T) {}
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub type Port<T> = port_stub::Port<T>;

