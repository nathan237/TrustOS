



use spin::Mutex;
use x86_64::instructions::port::Port;


const CMV_: u16 = 0x20;

const CMW_: u16 = 0x21;

const CMX_: u16 = 0xA0;

const CMY_: u16 = 0xA1;


const BFH_: u8 = 0x20;


const XO_: u8 = 32;

const BFG_: u8 = XO_ + 8;


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = XO_,
    Keyboard = XO_ + 1,
    Mouse = BFG_ + 4,  
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    
    pub fn as_usize(self) -> usize {
        self as usize
    }
}


pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    
    pub const fn new() -> Self {
        Self {
            pics: [
                Pic::new(CMV_, CMW_, XO_),
                Pic::new(CMX_, CMY_, BFG_),
            ],
        }
    }
    
    
    pub unsafe fn initialize(&mut self) {
        
        self.pics[0].command.write(0x11);
        self.pics[1].command.write(0x11);
        
        
        self.pics[0].data.write(self.pics[0].offset);
        self.pics[1].data.write(self.pics[1].offset);
        
        
        self.pics[0].data.write(4); 
        self.pics[1].data.write(2); 
        
        
        self.pics[0].data.write(0x01);
        self.pics[1].data.write(0x01);
        
        
        self.pics[0].data.write(0b11111000); 
        self.pics[1].data.write(0b11101111); 
    }
    
    
    pub unsafe fn notify_end_of_interrupt(&mut self, irq: u8) {
        if irq >= self.pics[1].offset {
            self.pics[1].command.write(BFH_);
        }
        self.pics[0].command.write(BFH_);
    }
}


struct Pic {
    command: Port<u8>,
    data: Port<u8>,
    offset: u8,
}

impl Pic {
    const fn new(command_port: u16, zu: u16, offset: u8) -> Self {
        Self {
            command: Port::new(command_port),
            data: Port::new(zu),
            offset,
        }
    }
}


pub static Gv: Mutex<ChainedPics> = Mutex::new(ChainedPics::new());
