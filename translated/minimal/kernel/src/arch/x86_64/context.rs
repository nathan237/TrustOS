




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
    pub pc: u64,
    pub rflags: u64,
    pub jm: u64,
    
    pub swq: [u8; 512],
}

impl CpuContext {
    pub const fn new() -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            pc: 0, rflags: 0x202, 
            jm: 0,
            swq: [0; 512],
        }
    }
    
    
    pub fn pix(&mut self, bt: u64) {
        self.pc = bt;
    }
    
    
    pub fn pjg(&mut self, sp: u64) {
        self.rsp = sp;
    }
    
    
    pub fn pjc(&mut self, se: u64) {
        self.jm = se;
    }
    
    
    pub fn edk(&self) -> u64 {
        self.pc
    }
    
    
    pub fn pns(&self) -> u64 {
        self.rsp
    }
}
