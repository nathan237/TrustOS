//! Programmable Interrupt Controller (8259 PIC)
//! 
//! Manages hardware interrupt routing from legacy PIC.

use spin::Mutex;
use x86_64::instructions::port::Port;

/// PIC1 command port
const PIC1_COMMAND: u16 = 0x20;
/// PIC1 data port
const PIC1_DATA: u16 = 0x21;
/// PIC2 command port
const PIC2_COMMAND: u16 = 0xA0;
/// PIC2 data port
const PIC2_DATA: u16 = 0xA1;

/// End of interrupt command
const PIC_EOI: u8 = 0x20;

/// PIC1 offset in IDT
const PIC1_OFFSET: u8 = 32;
/// PIC2 offset in IDT
const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

/// Hardware interrupt indices
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC1_OFFSET,
    Keyboard = PIC1_OFFSET + 1,
    Mouse = PIC2_OFFSET + 4,  // IRQ12 = PIC2 IRQ4
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    
    pub fn as_usize(self) -> usize {
        self as usize
    }
}

/// Chained PIC controller
pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    /// Create new chained PIC controller
    pub const fn new() -> Self {
        Self {
            pics: [
                Pic::new(PIC1_COMMAND, PIC1_DATA, PIC1_OFFSET),
                Pic::new(PIC2_COMMAND, PIC2_DATA, PIC2_OFFSET),
            ],
        }
    }
    
    /// Initialize both PICs
    pub unsafe fn initialize(&mut self) {
        // ICW1: Start initialization sequence
        self.pics[0].command.write(0x11);
        self.pics[1].command.write(0x11);
        
        // ICW2: Set vector offsets
        self.pics[0].data.write(self.pics[0].offset);
        self.pics[1].data.write(self.pics[1].offset);
        
        // ICW3: Configure cascading
        self.pics[0].data.write(4); // PIC2 at IRQ2
        self.pics[1].data.write(2); // Cascade identity
        
        // ICW4: Set 8086 mode
        self.pics[0].data.write(0x01);
        self.pics[1].data.write(0x01);
        
        // Mask all interrupts except timer, keyboard, and cascade
        self.pics[0].data.write(0b11111000); // Enable IRQ0 (timer), IRQ1 (keyboard), IRQ2 (cascade)
        self.pics[1].data.write(0b11101111); // Enable IRQ12 (mouse) = PIC2 IRQ4
    }
    
    /// Notify end of interrupt
    pub unsafe fn notify_end_of_interrupt(&mut self, irq: u8) {
        if irq >= self.pics[1].offset {
            self.pics[1].command.write(PIC_EOI);
        }
        self.pics[0].command.write(PIC_EOI);
    }
}

/// Single PIC controller
struct Pic {
    command: Port<u8>,
    data: Port<u8>,
    offset: u8,
}

impl Pic {
    const fn new(command_port: u16, data_port: u16, offset: u8) -> Self {
        Self {
            command: Port::new(command_port),
            data: Port::new(data_port),
            offset,
        }
    }
}

/// Global PIC instance
pub static PICS: Mutex<ChainedPics> = Mutex::new(ChainedPics::new());
