




use super::cpu;


pub struct Port {
    port: u16,
}

impl Port {
    
    pub const fn new(port: u16) -> Self {
        Self { port }
    }
    
    
    #[inline(always)]
    pub unsafe fn read_u8(&self) -> u8 {
        cpu::om(self.port)
    }
    
    
    #[inline(always)]
    pub unsafe fn write_u8(&self, val: u8) {
        cpu::vp(self.port, val);
    }
    
    
    #[inline(always)]
    pub unsafe fn read_u16(&self) -> u16 {
        cpu::eqz(self.port)
    }
    
    
    #[inline(always)]
    pub unsafe fn write_u16(&self, val: u16) {
        cpu::evw(self.port, val);
    }
    
    
    #[inline(always)]
    pub unsafe fn read_u32(&self) -> u32 {
        cpu::eqp(self.port)
    }
    
    
    #[inline(always)]
    pub unsafe fn write_u32(&self, val: u32) {
        cpu::evv(self.port, val);
    }
    
    
    pub const fn qqt(&self) -> u16 {
        self.port
    }
}
