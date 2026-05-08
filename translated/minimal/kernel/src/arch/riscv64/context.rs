




#[derive(Debug, Clone)]
#[repr(C)]
pub struct CpuContext {
    
    pub x: [u64; 32],
    
    pub pc: u64,
    
    pub sstatus: u64,
    
    pub satp: u64,
    
    pub tp: u64,
    
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
    
    
    pub fn jfc(&mut self, entry: u64) {
        self.pc = entry;
    }
    
    
    pub fn jfl(&mut self, sp: u64) {
        self.x[2] = sp;
    }
    
    
    pub fn jfh(&mut self, jd: u64) {
        self.satp = jd;
    }
    
    
    pub fn instruction_pointer(&self) -> u64 {
        self.pc
    }
    
    
    pub fn jic(&self) -> u64 {
        self.x[2]
    }
}
