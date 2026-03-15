//! aarch64 Context Switching
//!
//! Save/restore CPU registers for task switching on ARM64.

/// Saved CPU context for a thread/process
#[derive(Debug, Clone)]
#[repr(C)]
pub struct CpuContext {
    // General-purpose registers x0-x30
    pub x: [u64; 31],
    // Stack pointer
    pub sp: u64,
    // Program counter (ELR_EL1 for exception return)
    pub pc: u64,
    // Saved Process Status Register (SPSR_EL1)
    pub pstate: u64,
    // Page table base (TTBR0_EL1)
    pub ttbr0: u64,
    // Thread pointer (TPIDR_EL0)
    pub tpidr_el0: u64,
    /// FP/SIMD state (32 x 128-bit registers = 512 bytes)
    pub fp_state: [u8; 512],
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuContext {
    pub const fn new() -> Self {
        Self {
            x: [0; 31],
            sp: 0,
            pc: 0,
            pstate: 0x0, // EL0t (user mode, SP_EL0)
            ttbr0: 0,
            tpidr_el0: 0,
            fp_state: [0; 512],
        }
    }
    
    /// Set the instruction pointer (PC / ELR_EL1)
    pub fn set_entry(&mut self, entry: u64) {
        self.pc = entry;
    }
    
    /// Set the stack pointer
    pub fn set_stack(&mut self, sp: u64) {
        self.sp = sp;
    }
    
    /// Set the page table root (TTBR0_EL1)
    pub fn set_page_table(&mut self, pt: u64) {
        self.ttbr0 = pt;
    }
    
    /// Get the instruction pointer
    pub fn instruction_pointer(&self) -> u64 {
        self.pc
    }
    
    /// Get the stack pointer
    pub fn stack_pointer(&self) -> u64 {
        self.sp
    }
}
