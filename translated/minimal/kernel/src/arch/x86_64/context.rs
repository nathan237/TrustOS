




#[derive(Debug, Clone)]
#[repr(C)]
pub struct CpuContext {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
    pub cr3: u64,
    
    pub fpu_state: [u8; 512],
}

impl CpuContext {
    pub const fn new() -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0, rflags: 0x202, 
            cr3: 0,
            fpu_state: [0; 512],
        }
    }
    
    
    pub fn jfc(&mut self, entry: u64) {
        self.rip = entry;
    }
    
    
    pub fn jfl(&mut self, sp: u64) {
        self.rsp = sp;
    }
    
    
    pub fn jfh(&mut self, jd: u64) {
        self.cr3 = jd;
    }
    
    
    pub fn instruction_pointer(&self) -> u64 {
        self.rip
    }
    
    
    pub fn jic(&self) -> u64 {
        self.rsp
    }
}
