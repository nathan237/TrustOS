
#![allow(bgr)]

pub struct Timer {
    pub div: u16,       
    pub ejw: u8,       
    pub fww: u8,        
    pub ezl: u8,        
    pub gkb: bool, 
    vaf: u8, 
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0xABCC,
            ejw: 0,
            fww: 0,
            ezl: 0,
            gkb: false,
            vaf: 0,
        }
    }

    
    pub fn gu(&mut self, yl: u32) {
        for _ in 0..yl {
            let uxo = self.div;
            self.div = self.div.cn(4); 

            
            if self.ezl & 0x04 != 0 {
                let ga = match self.ezl & 0x03 {
                    0 => 9,  
                    1 => 3,  
                    2 => 5,  
                    3 => 7,  
                    _ => 9,
                };

                
                let uxm = (uxo >> ga) & 1;
                let usm = (self.div >> ga) & 1;

                if uxm == 1 && usm == 0 {
                    let (utv, lrg) = self.ejw.zem(1);
                    if lrg {
                        self.ejw = self.fww;
                        self.gkb = true;
                    } else {
                        self.ejw = utv;
                    }
                }
            }
        }
    }

    pub fn pac(&self) -> u8 {
        (self.div >> 8) as u8
    }

    pub fn xvi(&mut self) {
        self.div = 0;
    }
}
