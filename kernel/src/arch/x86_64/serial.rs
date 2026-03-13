//! x86_64 Serial Port Driver (UART 16550)
//!
//! Provides serial I/O via COM1 (port 0x3F8) for debug output.
//! Includes UART detection via loopback test and write timeout to prevent
//! infinite hangs on hardware without a serial controller.

use super::cpu::{inb, outb};
use core::sync::atomic::{AtomicBool, Ordering};

/// COM1 base I/O port
const COM1: u16 = 0x3F8;

/// Maximum iterations to wait for transmit buffer ready
const WRITE_TIMEOUT: u32 = 100_000;

/// Whether a real UART was detected during init
static SERIAL_PRESENT: AtomicBool = AtomicBool::new(false);

/// Check if serial port hardware is present
pub fn is_present() -> bool {
    SERIAL_PRESENT.load(Ordering::Relaxed)
}

/// Initialize COM1 serial port with hardware detection
pub fn init() {
    unsafe {
        // Loopback test: set MCR bit 4 (loopback mode), write a test byte, read it back
        outb(COM1 + 4, 0x1E); // Enable loopback mode
        outb(COM1, 0xAE);     // Send test byte
        if inb(COM1) != 0xAE {
            // No UART detected — leave SERIAL_PRESENT as false
            return;
        }

        // UART exists — configure it normally
        outb(COM1 + 4, 0x0F); // Disable loopback, enable OUT1/OUT2/RTS/DTR
        outb(COM1 + 1, 0x00); // Disable all interrupts
        outb(COM1 + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(COM1 + 0, 0x01); // Set divisor to 1 (115200 baud)
        outb(COM1 + 1, 0x00); //   (hi byte)
        outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(COM1 + 2, 0xC7); // Enable FIFO, clear them, 14-byte threshold
        outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set

        SERIAL_PRESENT.store(true, Ordering::SeqCst);
    }
}

/// Write a single byte to serial (with timeout — never hangs)
pub fn write_byte(byte: u8) {
    if !SERIAL_PRESENT.load(Ordering::Relaxed) {
        return;
    }
    unsafe {
        // Wait until transmit buffer is empty, with timeout
        let mut timeout = WRITE_TIMEOUT;
        while inb(COM1 + 5) & 0x20 == 0 {
            timeout -= 1;
            if timeout == 0 {
                return; // Give up — UART stuck
            }
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
    if !SERIAL_PRESENT.load(Ordering::Relaxed) {
        return None;
    }
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
