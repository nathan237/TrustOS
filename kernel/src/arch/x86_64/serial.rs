//! x86_64 Serial Port Driver (UART 16550)
//!
//! Provides serial I/O via COM1 (port 0x3F8) for debug output.

use super::cpu::{inb, outb};

/// COM1 base I/O port
const COM1: u16 = 0x3F8;

/// Initialize COM1 serial port
pub fn init() {
    unsafe {
        outb(COM1 + 1, 0x00); // Disable all interrupts
        outb(COM1 + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(COM1 + 0, 0x01); // Set divisor to 1 (115200 baud)
        outb(COM1 + 1, 0x00); //   (hi byte)
        outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(COM1 + 2, 0xC7); // Enable FIFO, clear them, 14-byte threshold
        outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }
}

/// Write a single byte to serial
pub fn write_byte(byte: u8) {
    unsafe {
        // Wait until transmit buffer is empty
        while inb(COM1 + 5) & 0x20 == 0 {
            core::hint::spin_loop();
        }
        outb(COM1, byte);
    }
}

/// Write a byte slice to serial
pub fn write_bytes(bytes: &[u8]) {
    for &b in bytes {
        if b == b'\n' {
            write_byte(b'\r');
        }
        write_byte(b);
    }
}

/// Try to read a byte from serial (non-blocking)
pub fn read_byte() -> Option<u8> {
    unsafe {
        if inb(COM1 + 5) & 0x01 != 0 {
            Some(inb(COM1))
        } else {
            None
        }
    }
}

/// Check if data is available to read
pub fn data_available() -> bool {
    unsafe { inb(COM1 + 5) & 0x01 != 0 }
}
