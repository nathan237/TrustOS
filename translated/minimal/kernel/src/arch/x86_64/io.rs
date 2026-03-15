




use super::cpu;


pub struct Port {
    port: u16,
}

impl Port {
    
    pub const fn new(port: u16) -> Self {
        Self { port }
    }
    
    
    #[inline(always)]
    pub unsafe fn ady(&self) -> u8 {
        cpu::cfn(self.port)
    }
    
    
    #[inline(always)]
    pub unsafe fn cvj(&self, ap: u8) {
        cpu::bkt(self.port, ap);
    }
    
    
    #[inline(always)]
    pub unsafe fn alp(&self) -> u16 {
        cpu::jar(self.port)
    }
    
    
    #[inline(always)]
    pub unsafe fn aqr(&self, ap: u16) {
        cpu::jie(self.port, ap);
    }
    
    
    #[inline(always)]
    pub unsafe fn za(&self) -> u32 {
        cpu::jac(self.port)
    }
    
    
    #[inline(always)]
    pub unsafe fn sx(&self, ap: u32) {
        cpu::jic(self.port, ap);
    }
    
    
    pub const fn zfx(&self) -> u16 {
        self.port
    }
}
