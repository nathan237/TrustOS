

























pub const ADK_: u64 = 0xFED0_0000;



const AVX_: u32 = 69_841_279; 


const EN_: usize = 3;


mod regs {
    pub const ASR_: u64      = 0x000;  
    pub const Bho: u64        = 0x010;  
    pub const Bhq: u64         = 0x020;  
    pub const AZF_: u64 = 0x0F0;  
    
    
    pub const LF_: u64   = 0x100;
    pub const LH_: u64 = 0x20;
    pub const BHJ_: u64      = 0x00;   
    pub const BHI_: u64      = 0x08;   
    pub const BHK_: u64       = 0x10;   
}


#[derive(Debug, Clone)]
pub struct HpetTimer {
    
    pub config: u64,
    
    pub dpb: u64,
    
    pub kxk: u64,
}

impl Default for HpetTimer {
    fn default() -> Self {
        Self {
            
            
            
            
            config: 0x0000_0000_00F0_0030, 
            dpb: 0,
            kxk: 0,
        }
    }
}


#[derive(Debug, Clone)]
pub struct HpetState {
    
    pub config: u64,
    
    pub cru: u64,
    
    pub dpe: u64,
    
    pub mng: u64,
    
    pub iq: bool,
    
    pub axe: [HpetTimer; EN_],
}

impl Default for HpetState {
    fn default() -> Self {
        let mut axe: [HpetTimer; EN_] = core::array::nwe(|_| HpetTimer::default());
        
        axe[0].config = 0x0000_0000_00F0_0070; 
        
        axe[1].config = 0x0000_0000_00F0_0030;
        axe[2].config = 0x0000_0000_00F0_0030;
        
        Self {
            config: 0,
            cru: 0,
            dpe: 0,
            mng: 0,
            iq: false,
            axe,
        }
    }
}

impl HpetState {
    
    
    fn lke(&self) -> u64 {
        if !self.iq {
            return self.dpe;
        }
        let xna = ow();
        let xmz = xna.nj(self.mng);
        
        
        
        
        let tqj = xmz / 140;
        self.dpe.cn(tqj)
    }
    
    
    
    pub fn read(&self, l: u64, aw: u8) -> u64 {
        let mou = match l {
            regs::ASR_ => {
                
                
                
                
                
                let uwo = (EN_ as u64 - 1) << 8;
                let eoc = 1u64 << 13;
                let vym = 0x01u64; 
                ((AVX_ as u64) << 32) | uwo | eoc | vym
            }
            regs::Bho => self.config,
            regs::Bhq => self.cru,
            regs::AZF_ => self.lke(),
            
            
            dz if dz >= regs::LF_ && dz < regs::LF_ + (EN_ as u64) * regs::LH_ + 0x18 => {
                let idp = dz - regs::LF_;
                let dwv = (idp / regs::LH_) as usize;
                let lym = idp % regs::LH_;
                
                if dwv < EN_ {
                    match lym {
                        regs::BHJ_ => self.axe[dwv].config,
                        regs::BHI_ => self.axe[dwv].dpb,
                        regs::BHK_  => self.axe[dwv].kxk,
                        _ => 0,
                    }
                } else {
                    0
                }
            }
            _ => 0,
        };
        
        
        if aw == 4 {
            if l & 0x4 != 0 {
                
                (mou >> 32) & 0xFFFF_FFFF
            } else {
                mou & 0xFFFF_FFFF
            }
        } else {
            mou
        }
    }
    
    
    pub fn write(&mut self, l: u64, bn: u64, aw: u8) {
        match l {
            regs::ASR_ => {} 
            
            regs::Bho => {
                let osg = self.iq;
                self.config = bn & 0x3; 
                self.iq = (bn & 1) != 0;
                
                if self.iq && !osg {
                    
                    self.mng = ow();
                } else if !self.iq && osg {
                    
                    self.dpe = self.lke();
                }
            }
            
            regs::Bhq => {
                
                self.cru &= !bn;
            }
            
            regs::AZF_ => {
                
                if !self.iq {
                    if aw == 4 {
                        if l & 0x4 != 0 {
                            self.dpe = (self.dpe & 0xFFFF_FFFF) | (bn << 32);
                        } else {
                            self.dpe = (self.dpe & !0xFFFF_FFFF) | (bn & 0xFFFF_FFFF);
                        }
                    } else {
                        self.dpe = bn;
                    }
                }
            }
            
            
            dz if dz >= regs::LF_ && dz < regs::LF_ + (EN_ as u64) * regs::LH_ + 0x18 => {
                let idp = dz - regs::LF_;
                let dwv = (idp / regs::LH_) as usize;
                let lym = idp % regs::LH_;
                
                if dwv < EN_ {
                    match lym {
                        regs::BHJ_ => {
                            
                            
                            
                            
                            
                            
                            
                            let jmr: u64 = self.axe[dwv].config & 0xFFFF_FFFF_FFFF_8181;
                            let xvf: u64 = 0x0000_0000_0000_7E7E;
                            self.axe[dwv].config = jmr | (bn & xvf);
                        }
                        regs::BHI_ => {
                            self.axe[dwv].dpb = bn;
                        }
                        regs::BHK_ => {
                            self.axe[dwv].kxk = bn;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    
    
    
    pub fn qzy(&self) -> [(bool, u8); EN_] {
        let va = self.lke();
        let mut result = [(false, 0u8); EN_];
        
        for a in 0..EN_ {
            let config = self.axe[a].config;
            let iq = (config >> 2) & 1 != 0;
            if !self.iq || !iq {
                continue;
            }
            let dpb = self.axe[a].dpb;
            if va >= dpb && dpb > 0 {
                let lft = ((config >> 9) & 0x1F) as u8;
                result[a] = (true, lft);
            }
        }
        result
    }
}


#[inline(always)]
fn ow() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::dxw()
    }
    #[cfg(not(target_arch = "x86_64"))]
    { 0 }
}





#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn psg() {
        let hpet = HpetState::default();
        assert!(!hpet.iq);
        assert_eq!(hpet.config, 0);
        assert_eq!(hpet.axe.len(), 3);
    }
    
    #[test]
    fn psi() {
        let hpet = HpetState::default();
        let cew = hpet.read(0x000, 8);
        let awn = (cew >> 32) as u32;
        assert_eq!(awn, AVX_);
        let uwn = ((cew >> 8) & 0x1F) as usize;
        assert_eq!(uwn, 2); 
        assert_ne!(cew & (1 << 13), 0); 
    }
    
    #[test]
    fn psh() {
        let mut hpet = HpetState::default();
        assert!(!hpet.iq);
        hpet.write(0x010, 1, 8); 
        assert!(hpet.iq);
        hpet.write(0x010, 0, 8); 
        assert!(!hpet.iq);
    }
    
    #[test]
    fn zrs() {
        let mut hpet = HpetState::default();
        hpet.write(0x0F0, 0x12345, 8); 
        let r = hpet.read(0x0F0, 8);
        assert_eq!(r, 0x12345);
    }
}
