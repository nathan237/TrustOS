




#[derive(Debug, Clone)]
#[repr(C)]
pub struct CpuContext {
    
    pub b: [u64; 31],
    
    pub sp: u64,
    
    pub fz: u64,
    
    pub vnx: u64,
    
    pub pwj: u64,
    
    pub xkv: u64,
    
    pub kww: [u8; 512],
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuContext {
    pub const fn new() -> Self {
        Self {
            b: [0; 31],
            sp: 0,
            fz: 0,
            vnx: 0x0, 
            pwj: 0,
            xkv: 0,
            kww: [0; 512],
        }
    }
    
    
    pub fn pix(&mut self, bt: u64) {
        self.fz = bt;
    }
    
    
    pub fn pjg(&mut self, sp: u64) {
        self.sp = sp;
    }
    
    
    pub fn pjc(&mut self, se: u64) {
        self.pwj = se;
    }
    
    
    pub fn edk(&self) -> u64 {
        self.fz
    }
    
    
    pub fn pns(&self) -> u64 {
        self.sp
    }
}
