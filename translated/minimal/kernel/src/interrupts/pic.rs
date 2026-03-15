



use spin::Mutex;
use x86_64::instructions::port::Port;


const CJM_: u16 = 0x20;

const CJN_: u16 = 0x21;

const CJO_: u16 = 0xA0;

const CJP_: u16 = 0xA1;


const BDE_: u8 = 0x20;


const WF_: u8 = 32;

const BDD_: u8 = WF_ + 8;


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = WF_,
    Hs = WF_ + 1,
    Cp = BDD_ + 4,  
}

impl InterruptIndex {
    pub fn gaj(self) -> u8 {
        self as u8
    }
    
    pub fn kbe(self) -> usize {
        self as usize
    }
}


pub struct ChainedPics {
    cbb: [Pic; 2],
}

impl ChainedPics {
    
    pub const fn new() -> Self {
        Self {
            cbb: [
                Pic::new(CJM_, CJN_, WF_),
                Pic::new(CJO_, CJP_, BDD_),
            ],
        }
    }
    
    
    pub unsafe fn cfp(&mut self) {
        
        self.cbb[0].ro.write(0x11);
        self.cbb[1].ro.write(0x11);
        
        
        self.cbb[0].f.write(self.cbb[0].l);
        self.cbb[1].f.write(self.cbb[1].l);
        
        
        self.cbb[0].f.write(4); 
        self.cbb[1].f.write(2); 
        
        
        self.cbb[0].f.write(0x01);
        self.cbb[1].f.write(0x01);
        
        
        self.cbb[0].f.write(0b11111000); 
        self.cbb[1].f.write(0b11101111); 
    }
    
    
    pub unsafe fn goa(&mut self, irq: u8) {
        if irq >= self.cbb[1].l {
            self.cbb[1].ro.write(BDE_);
        }
        self.cbb[0].ro.write(BDE_);
    }
}


struct Pic {
    ro: Port<u8>,
    f: Port<u8>,
    l: u8,
}

impl Pic {
    const fn new(rmr: u16, axr: u16, l: u8) -> Self {
        Self {
            ro: Port::new(rmr),
            f: Port::new(axr),
            l,
        }
    }
}


pub static Qh: Mutex<ChainedPics> = Mutex::new(ChainedPics::new());
