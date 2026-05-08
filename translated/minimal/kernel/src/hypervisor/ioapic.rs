


























const LJ_: usize = 24;


pub const AFG_: u64 = 0xFEC0_0000;


const Sz: u64 = 0x00;  
const Ta: u64 = 0x10;     


#[derive(Debug, Clone)]
pub struct IoApicState {
    
    pub id: u8,
    
    pub ioregsel: u32,
    
    pub redir_table: [u64; LJ_],
}

impl Default for IoApicState {
    fn default() -> Self {
        
        let mut redir_table = [0u64; LJ_];
        for entry in redir_table.iter_mut() {
            *entry = 1 << 16; 
        }
        
        Self {
            id: 1, 
            ioregsel: 0,
            redir_table,
        }
    }
}

impl IoApicState {
    
    
    pub fn read(&self, offset: u64) -> u32 {
        match offset {
            Sz => self.ioregsel,
            Ta => self.read_register(self.ioregsel),
            _ => 0,
        }
    }
    
    
    pub fn write(&mut self, offset: u64, value: u32) {
        match offset {
            Sz => {
                self.ioregsel = value;
            }
            Ta => {
                self.write_register(self.ioregsel, value);
            }
            _ => {}
        }
    }
    
    
    fn read_register(&self, index: u32) -> u32 {
        match index {
            
            0x00 => (self.id as u32) << 24,
            
            
            0x01 => {
                let ncw = (LJ_ - 1) as u32;
                (ncw << 16) | 0x20 
            }
            
            
            0x02 => (self.id as u32) << 24,
            
            
            
            0x10..=0x3F => {
                let ado = ((index - 0x10) / 2) as usize;
                let dsp = (index & 1) != 0;
                
                if ado < LJ_ {
                    if dsp {
                        (self.redir_table[ado] >> 32) as u32
                    } else {
                        self.redir_table[ado] as u32
                    }
                } else {
                    0
                }
            }
            
            _ => 0,
        }
    }
    
    
    fn write_register(&mut self, index: u32, value: u32) {
        match index {
            
            0x00 => {
                self.id = ((value >> 24) & 0xF) as u8;
            }
            
            
            0x01 | 0x02 => {}
            
            
            0x10..=0x3F => {
                let ado = ((index - 0x10) / 2) as usize;
                let dsp = (index & 1) != 0;
                
                if ado < LJ_ {
                    if dsp {
                        
                        self.redir_table[ado] = 
                            (self.redir_table[ado] & 0x0000_0000_FFFF_FFFF)
                            | ((value as u64) << 32);
                    } else {
                        
                        
                        let eyw: u32 = (1 << 12) | (1 << 14);
                        let nmr = self.redir_table[ado] as u32;
                        let new_lo = (value & !eyw) | (nmr & eyw);
                        self.redir_table[ado] = 
                            (self.redir_table[ado] & 0xFFFF_FFFF_0000_0000)
                            | (new_lo as u64);
                    }
                }
            }
            
            _ => {}
        }
    }
    
    
    
    pub fn get_irq_route(&self, gsi: u8) -> Option<Aaj> {
        let idx = gsi as usize;
        if idx >= LJ_ {
            return None;
        }
        
        let entry = self.redir_table[idx];
        let vector = (entry & 0xFF) as u8;
        let delivery_mode = ((entry >> 8) & 0x7) as u8;
        let ldr = ((entry >> 11) & 1) != 0; 
        let polarity = ((entry >> 13) & 1) != 0;   
        let trigger = ((entry >> 15) & 1) != 0;     
        let masked = ((entry >> 16) & 1) != 0;
        let hrr = ((entry >> 56) & 0xFF) as u8;
        
        Some(Aaj {
            vector,
            delivery_mode,
            dest_logical: ldr,
            active_low: polarity,
            level_triggered: trigger,
            masked,
            hrr,
        })
    }
}


#[derive(Debug, Clone)]
pub struct Aaj {
    
    pub vector: u8,
    
    pub delivery_mode: u8,
    
    pub dest_logical: bool,
    
    pub active_low: bool,
    
    pub level_triggered: bool,
    
    pub masked: bool,
    
    pub hrr: u8,
}





#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn jlx() {
        let ioapic = IoApicState::default();
        assert_eq!(ioapic.id, 1);
        assert_eq!(ioapic.ioregsel, 0);
        
        for entry in &ioapic.redir_table {
            assert_ne!(*entry & (1 << 16), 0, "Entry should be masked");
        }
    }
    
    #[test]
    fn qzn() {
        let ioapic = IoApicState::default();
        let drs = ioapic.read_register(0x00);
        assert_eq!((drs >> 24) & 0xF, 1);
    }
    
    #[test]
    fn jlz() {
        let ioapic = IoApicState::default();
        let tu = ioapic.read_register(0x01);
        assert_eq!(tu & 0xFF, 0x20); 
        assert_eq!((tu >> 16) & 0xFF, 23); 
    }
    
    #[test]
    fn qzo() {
        let mut ioapic = IoApicState::default();
        
        ioapic.write_register(0x10, 0x0000_0030);
        
        ioapic.write_register(0x11, 0x0000_0000);
        
        let lo = ioapic.read_register(0x10);
        assert_eq!(lo & 0xFF, 0x30); 
        assert_eq!((lo >> 16) & 1, 0); 
        
        let afo = ioapic.get_irq_route(0).unwrap();
        assert_eq!(afo.vector, 0x30);
        assert!(!afo.masked);
        assert_eq!(afo.delivery_mode, 0); 
    }
    
    #[test]
    fn jly() {
        let mut ioapic = IoApicState::default();
        
        ioapic.write(Sz, 0x01); 
        let tu = ioapic.read(Ta);
        assert_eq!(tu & 0xFF, 0x20);
    }
}
