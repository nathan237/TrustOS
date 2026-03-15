


























const KQ_: usize = 24;


pub const ADP_: u64 = 0xFEC0_0000;


const Atw: u64 = 0x00;  
const Atx: u64 = 0x10;     


#[derive(Debug, Clone)]
pub struct IoApicState {
    
    pub ad: u8,
    
    pub esz: u32,
    
    pub ctu: [u64; KQ_],
}

impl Default for IoApicState {
    fn default() -> Self {
        
        let mut ctu = [0u64; KQ_];
        for bt in ctu.el() {
            *bt = 1 << 16; 
        }
        
        Self {
            ad: 1, 
            esz: 0,
            ctu,
        }
    }
}

impl IoApicState {
    
    
    pub fn read(&self, l: u64) -> u32 {
        match l {
            Atw => self.esz,
            Atx => self.gql(self.esz),
            _ => 0,
        }
    }
    
    
    pub fn write(&mut self, l: u64, bn: u32) {
        match l {
            Atw => {
                self.esz = bn;
            }
            Atx => {
                self.ihl(self.esz, bn);
            }
            _ => {}
        }
    }
    
    
    fn gql(&self, index: u32) -> u32 {
        match index {
            
            0x00 => (self.ad as u32) << 24,
            
            
            0x01 => {
                let uld = (KQ_ - 1) as u32;
                (uld << 16) | 0x20 
            }
            
            
            0x02 => (self.ad as u32) << 24,
            
            
            
            0x10..=0x3F => {
                let bea = ((index - 0x10) / 2) as usize;
                let lge = (index & 1) != 0;
                
                if bea < KQ_ {
                    if lge {
                        (self.ctu[bea] >> 32) as u32
                    } else {
                        self.ctu[bea] as u32
                    }
                } else {
                    0
                }
            }
            
            _ => 0,
        }
    }
    
    
    fn ihl(&mut self, index: u32, bn: u32) {
        match index {
            
            0x00 => {
                self.ad = ((bn >> 24) & 0xF) as u8;
            }
            
            
            0x01 | 0x02 => {}
            
            
            0x10..=0x3F => {
                let bea = ((index - 0x10) / 2) as usize;
                let lge = (index & 1) != 0;
                
                if bea < KQ_ {
                    if lge {
                        
                        self.ctu[bea] = 
                            (self.ctu[bea] & 0x0000_0000_FFFF_FFFF)
                            | ((bn as u64) << 32);
                    } else {
                        
                        
                        let jmr: u32 = (1 << 12) | (1 << 14);
                        let uxq = self.ctu[bea] as u32;
                        let lnx = (bn & !jmr) | (uxq & jmr);
                        self.ctu[bea] = 
                            (self.ctu[bea] & 0xFFFF_FFFF_0000_0000)
                            | (lnx as u64);
                    }
                }
            }
            
            _ => {}
        }
    }
    
    
    
    pub fn hli(&self, bup: u8) -> Option<Bkf> {
        let w = bup as usize;
        if w >= KQ_ {
            return None;
        }
        
        let bt = self.ctu[w];
        let wj = (bt & 0xFF) as u8;
        let iqu = ((bt >> 8) & 0x7) as u8;
        let rwi = ((bt >> 11) & 1) != 0; 
        let dkr = ((bt >> 13) & 1) != 0;   
        let dmt = ((bt >> 15) & 1) != 0;     
        let bnm = ((bt >> 16) & 1) != 0;
        let nku = ((bt >> 56) & 0xFF) as u8;
        
        Some(Bkf {
            wj,
            iqu,
            rwh: rwi,
            qfa: dkr,
            oiy: dmt,
            bnm,
            nku,
        })
    }
}


#[derive(Debug, Clone)]
pub struct Bkf {
    
    pub wj: u8,
    
    pub iqu: u8,
    
    pub rwh: bool,
    
    pub qfa: bool,
    
    pub oiy: bool,
    
    pub bnm: bool,
    
    pub nku: u8,
}





#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn psj() {
        let ioapic = IoApicState::default();
        assert_eq!(ioapic.ad, 1);
        assert_eq!(ioapic.esz, 0);
        
        for bt in &ioapic.ctu {
            assert_ne!(*bt & (1 << 16), 0, "Entry should be masked");
        }
    }
    
    #[test]
    fn zrt() {
        let ioapic = IoApicState::default();
        let hnq = ioapic.gql(0x00);
        assert_eq!((hnq >> 24) & 0xF, 1);
    }
    
    #[test]
    fn psl() {
        let ioapic = IoApicState::default();
        let axh = ioapic.gql(0x01);
        assert_eq!(axh & 0xFF, 0x20); 
        assert_eq!((axh >> 16) & 0xFF, 23); 
    }
    
    #[test]
    fn zru() {
        let mut ioapic = IoApicState::default();
        
        ioapic.ihl(0x10, 0x0000_0030);
        
        ioapic.ihl(0x11, 0x0000_0000);
        
        let hh = ioapic.gql(0x10);
        assert_eq!(hh & 0xFF, 0x30); 
        assert_eq!((hh >> 16) & 1, 0); 
        
        let bia = ioapic.hli(0).unwrap();
        assert_eq!(bia.wj, 0x30);
        assert!(!bia.bnm);
        assert_eq!(bia.iqu, 0); 
    }
    
    #[test]
    fn psk() {
        let mut ioapic = IoApicState::default();
        
        ioapic.write(Atw, 0x01); 
        let axh = ioapic.read(Atx);
        assert_eq!(axh & 0xFF, 0x20);
    }
}
