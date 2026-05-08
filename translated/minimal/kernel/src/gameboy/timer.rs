
#![allow(dead_code)]

pub struct Timer {
    pub div: u16,       
    pub tima: u8,       
    pub tma: u8,        
    pub tac: u8,        
    pub interrupt: bool, 
    overflow_cycles: u8, 
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0xABCC,
            tima: 0,
            tma: 0,
            tac: 0,
            interrupt: false,
            overflow_cycles: 0,
        }
    }

    
    pub fn step(&mut self, cycles: u32) {
        for _ in 0..cycles {
            let nmo = self.div;
            self.div = self.div.wrapping_add(4); 

            
            if self.tac & 0x04 != 0 {
                let bf = match self.tac & 0x03 {
                    0 => 9,  
                    1 => 3,  
                    2 => 5,  
                    3 => 7,  
                    _ => 9,
                };

                
                let nmm = (nmo >> bf) & 1;
                let nir = (self.div >> bf) & 1;

                if nmm == 1 && nir == 0 {
                    let (new_tima, overflow) = self.tima.overflowing_add(1);
                    if overflow {
                        self.tima = self.tma;
                        self.interrupt = true;
                    } else {
                        self.tima = new_tima;
                    }
                }
            }
        }
    }

    pub fn read_div(&self) -> u8 {
        (self.div >> 8) as u8
    }

    pub fn write_div(&mut self) {
        self.div = 0;
    }
}
