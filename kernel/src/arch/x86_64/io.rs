//! x86_64 I/O Port Abstraction
//!
//! Type-safe I/O port access for x86 hardware.
//! On non-x86 architectures, equivalent functionality uses MMIO.

use super::cpu;

/// A type-safe I/O port for byte-width access
pub struct Port {
    port: u16,
}

impl Port {
    /// Create a new I/O port handle
    pub const fn new(port: u16) -> Self {
        Self { port }
    }
    
    /// Read a byte from the port
    #[inline(always)]
    pub unsafe fn read_u8(&self) -> u8 {
        cpu::inb(self.port)
    }
    
    /// Write a byte to the port
    #[inline(always)]
    pub unsafe fn write_u8(&self, val: u8) {
        cpu::outb(self.port, val);
    }
    
    /// Read a word from the port
    #[inline(always)]
    pub unsafe fn read_u16(&self) -> u16 {
        cpu::inw(self.port)
    }
    
    /// Write a word to the port
    #[inline(always)]
    pub unsafe fn write_u16(&self, val: u16) {
        cpu::outw(self.port, val);
    }
    
    /// Read a dword from the port
    #[inline(always)]
    pub unsafe fn read_u32(&self) -> u32 {
        cpu::inl(self.port)
    }
    
    /// Write a dword to the port
    #[inline(always)]
    pub unsafe fn write_u32(&self, val: u32) {
        cpu::outl(self.port, val);
    }
    
    /// Get the port number
    pub const fn port_number(&self) -> u16 {
        self.port
    }
}
