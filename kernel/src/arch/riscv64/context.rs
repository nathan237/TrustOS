//! RISC-V 64 Context Switching
//!
//! Save/restore CPU registers for task switching.

/// Saved CPU context for a thread/process
#[derive(Debug, Clone)]
#[repr(C)]
pub struct CpuContext {
    // General-purpose registers x0-x31 (x0 is hardwired to 0, but stored for alignment)
    pub x: [u64; 32],
    // Program counter (sepc for trap return)
    pub pc: u64,
    // Supervisor status (sstatus)
    pub sstatus: u64,
    // Page table base (satp)
    pub satp: u64,
    // Thread pointer (tp / x4)
    pub tp: u64,
    /// FP state (32 x 64-bit FP registers = 256 bytes)
    pub fp_state: [u8; 256],
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuContext {
    pub const fn new() -> Self {
        Self {
            x: [0; 32],
            pc: 0,
            sstatus: 0,
            satp: 0,
            tp: 0,
            fp_state: [0; 256],
        }
    }
    
    /// Set the instruction pointer (PC / sepc)
    pub fn set_entry(&mut self, entry: u64) {
        self.pc = entry;
    }
    
    /// Set the stack pointer (x2 / sp)
    pub fn set_stack(&mut self, sp: u64) {
        self.x[2] = sp;
    }
    
    /// Set the page table root (satp)
    pub fn set_page_table(&mut self, pt: u64) {
        self.satp = pt;
    }
    
    /// Get the instruction pointer
    pub fn instruction_pointer(&self) -> u64 {
        self.pc
    }
    
    /// Get the stack pointer
    pub fn stack_pointer(&self) -> u64 {
        self.x[2]
    }
}
