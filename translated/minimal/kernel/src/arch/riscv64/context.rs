




#[derive(Debug, Clone)]
#[repr(C)]
pub struct CpuContext {
    
    pub b: [u64; 32],
    
    pub fz: u64,
    
    pub sstatus: u64,
    
    pub jnl: u64,
    
    pub aaz: u64,
    
    pub kww: [u8; 256],
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuContext {
    pub const fn new() -> Self {
        Self {
            b: [0; 32],
            fz: 0,
            sstatus: 0,
            jnl: 0,
            aaz: 0,
            kww: [0; 256],
        }
    }
    
    
    pub fn pix(&mut self, bt: u64) {
        self.fz = bt;
    }
    
    
    pub fn pjg(&mut self, sp: u64) {
        self.b[2] = sp;
    }
    
    
    pub fn pjc(&mut self, se: u64) {
        self.jnl = se;
    }
    
    
    pub fn edk(&self) -> u64 {
        self.fz
    }
    
    
    pub fn pns(&self) -> u64 {
        self.b[2]
    }
}
