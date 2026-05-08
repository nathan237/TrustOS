

























pub const AFA_: u64 = 0xFED0_0000;



const AYA_: u32 = 69_841_279; 


const FB_: usize = 3;


mod regs {
    pub const AUV_: u64      = 0x000;  
    pub const Za: u64        = 0x010;  
    pub const Zc: u64         = 0x020;  
    pub const BBG_: u64 = 0x0F0;  
    
    
    pub const MA_: u64   = 0x100;
    pub const MC_: u64 = 0x20;
    pub const BJN_: u64      = 0x00;   
    pub const BJM_: u64      = 0x08;   
    pub const BJO_: u64       = 0x10;   
}


#[derive(Debug, Clone)]
pub struct HpetTimer {
    
    pub config: u64,
    
    pub comparator: u64,
    
    pub fsb_route: u64,
}

impl Default for HpetTimer {
    fn default() -> Self {
        Self {
            
            
            
            
            config: 0x0000_0000_00F0_0030, 
            comparator: 0,
            fsb_route: 0,
        }
    }
}


#[derive(Debug, Clone)]
pub struct HpetState {
    
    pub config: u64,
    
    pub isr: u64,
    
    pub counter_offset: u64,
    
    pub tsc_at_enable: u64,
    
    pub enabled: bool,
    
    pub timers: [HpetTimer; FB_],
}

impl Default for HpetState {
    fn default() -> Self {
        let mut timers: [HpetTimer; FB_] = core::array::from_fn(|_| HpetTimer::default());
        
        timers[0].config = 0x0000_0000_00F0_0070; 
        
        timers[1].config = 0x0000_0000_00F0_0030;
        timers[2].config = 0x0000_0000_00F0_0030;
        
        Self {
            config: 0,
            isr: 0,
            counter_offset: 0,
            tsc_at_enable: 0,
            enabled: false,
            timers,
        }
    }
}

impl HpetState {
    
    
    fn main_counter(&self) -> u64 {
        if !self.enabled {
            return self.counter_offset;
        }
        let pof = ey();
        let poe = pof.wrapping_sub(self.tsc_at_enable);
        
        
        
        
        let mmm = poe / 140;
        self.counter_offset.wrapping_add(mmm)
    }
    
    
    
    pub fn read(&self, offset: u64, size: u8) -> u64 {
        let hbd = match offset {
            regs::AUV_ => {
                
                
                
                
                
                let nlx = (FB_ as u64 - 1) << 8;
                let counter_64bit = 1u64 << 13;
                let ogs = 0x01u64; 
                ((AYA_ as u64) << 32) | nlx | counter_64bit | ogs
            }
            regs::Za => self.config,
            regs::Zc => self.isr,
            regs::BBG_ => self.main_counter(),
            
            
            off if off >= regs::MA_ && off < regs::MA_ + (FB_ as u64) * regs::MC_ + 0x18 => {
                let ebt = off - regs::MA_;
                let bpl = (ebt / regs::MC_) as usize;
                let gqw = ebt % regs::MC_;
                
                if bpl < FB_ {
                    match gqw {
                        regs::BJN_ => self.timers[bpl].config,
                        regs::BJM_ => self.timers[bpl].comparator,
                        regs::BJO_  => self.timers[bpl].fsb_route,
                        _ => 0,
                    }
                } else {
                    0
                }
            }
            _ => 0,
        };
        
        
        if size == 4 {
            if offset & 0x4 != 0 {
                
                (hbd >> 32) & 0xFFFF_FFFF
            } else {
                hbd & 0xFFFF_FFFF
            }
        } else {
            hbd
        }
    }
    
    
    pub fn write(&mut self, offset: u64, value: u64, size: u8) {
        match offset {
            regs::AUV_ => {} 
            
            regs::Za => {
                let isb = self.enabled;
                self.config = value & 0x3; 
                self.enabled = (value & 1) != 0;
                
                if self.enabled && !isb {
                    
                    self.tsc_at_enable = ey();
                } else if !self.enabled && isb {
                    
                    self.counter_offset = self.main_counter();
                }
            }
            
            regs::Zc => {
                
                self.isr &= !value;
            }
            
            regs::BBG_ => {
                
                if !self.enabled {
                    if size == 4 {
                        if offset & 0x4 != 0 {
                            self.counter_offset = (self.counter_offset & 0xFFFF_FFFF) | (value << 32);
                        } else {
                            self.counter_offset = (self.counter_offset & !0xFFFF_FFFF) | (value & 0xFFFF_FFFF);
                        }
                    } else {
                        self.counter_offset = value;
                    }
                }
            }
            
            
            off if off >= regs::MA_ && off < regs::MA_ + (FB_ as u64) * regs::MC_ + 0x18 => {
                let ebt = off - regs::MA_;
                let bpl = (ebt / regs::MC_) as usize;
                let gqw = ebt % regs::MC_;
                
                if bpl < FB_ {
                    match gqw {
                        regs::BJN_ => {
                            
                            
                            
                            
                            
                            
                            
                            let eyw: u64 = self.timers[bpl].config & 0xFFFF_FFFF_FFFF_8181;
                            let pva: u64 = 0x0000_0000_0000_7E7E;
                            self.timers[bpl].config = eyw | (value & pva);
                        }
                        regs::BJM_ => {
                            self.timers[bpl].comparator = value;
                        }
                        regs::BJO_ => {
                            self.timers[bpl].fsb_route = value;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    
    
    
    pub fn check_timers(&self) -> [(bool, u8); FB_] {
        let counter = self.main_counter();
        let mut result = [(false, 0u8); FB_];
        
        for i in 0..FB_ {
            let config = self.timers[i].config;
            let enabled = (config >> 2) & 1 != 0;
            if !self.enabled || !enabled {
                continue;
            }
            let comparator = self.timers[i].comparator;
            if counter >= comparator && comparator > 0 {
                let gdo = ((config >> 9) & 0x1F) as u8;
                result[i] = (true, gdo);
            }
        }
        result
    }
}


#[inline(always)]
fn ey() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
    #[cfg(not(target_arch = "x86_64"))]
    { 0 }
}





#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn jlu() {
        let hpet = HpetState::default();
        assert!(!hpet.enabled);
        assert_eq!(hpet.config, 0);
        assert_eq!(hpet.timers.len(), 3);
    }
    
    #[test]
    fn jlw() {
        let hpet = HpetState::default();
        let agk = hpet.read(0x000, 8);
        let zd = (agk >> 32) as u32;
        assert_eq!(zd, AYA_);
        let nlw = ((agk >> 8) & 0x1F) as usize;
        assert_eq!(nlw, 2); 
        assert_ne!(agk & (1 << 13), 0); 
    }
    
    #[test]
    fn jlv() {
        let mut hpet = HpetState::default();
        assert!(!hpet.enabled);
        hpet.write(0x010, 1, 8); 
        assert!(hpet.enabled);
        hpet.write(0x010, 0, 8); 
        assert!(!hpet.enabled);
    }
    
    #[test]
    fn qzm() {
        let mut hpet = HpetState::default();
        hpet.write(0x0F0, 0x12345, 8); 
        let c = hpet.read(0x0F0, 8);
        assert_eq!(c, 0x12345);
    }
}
