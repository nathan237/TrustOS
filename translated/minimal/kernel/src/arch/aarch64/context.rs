




#[derive(Debug, Clone)]
#[repr(C)]
pub struct CpuContext {
    
    pub x: [u64; 31],
    
    pub sp: u64,
    
    pub pc: u64,
    
    pub pstate: u64,
    
    pub ttbr0: u64,
    
    pub tpidr_el0: u64,
    
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
            pstate: 0x0, 
            ttbr0: 0,
            tpidr_el0: 0,
            fp_state: [0; 512],
        }
    }
    
    
    pub fn jfc(&mut self, entry: u64) {
        self.pc = entry;
    }
    
    
    pub fn jfl(&mut self, sp: u64) {
        self.sp = sp;
    }
    
    
    pub fn jfh(&mut self, jd: u64) {
        self.ttbr0 = jd;
    }
    
    
    pub fn instruction_pointer(&self) -> u64 {
        self.pc
    }
    
    
    pub fn jic(&self) -> u64 {
        self.sp
    }
}
