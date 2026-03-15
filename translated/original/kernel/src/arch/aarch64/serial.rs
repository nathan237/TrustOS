//! aarch64 Serial Port Driver (PL011 UART)
//!
//! PL011 is the standard UART on ARM platforms (QEMU virt, Raspberry Pi).
//! On QEMU virt machine, PL011 is at MMIO address 0x0900_0000.

use super::cpu::{mmio_read32, mmio_write32, mmio_read8, mmio_write8};

/// PL011 UART base address for QEMU virt machine
/// On real hardware (RPi4), this would be 0xFE20_1000
/// Limine maps this via HHDM, so we use the physical address + HHDM offset
static mut UART_BASE: u64 = 0x0900_0000;

// PL011 register offsets
const UARTDR: u64 = 0x000;     // Data Register
const UARTRSR: u64 = 0x004;    // Receive Status Register
const UARTFR: u64 = 0x018;     // Flag Register
const UARTIBRD: u64 = 0x024;   // Integer Baud Rate Divisor
const UARTFBRD: u64 = 0x028;   // Fractional Baud Rate Divisor
const UARTLCR_H: u64 = 0x02C;  // Line Control Register
const UARTCR: u64 = 0x030;     // Control Register
const UARTIMSC: u64 = 0x038;   // Interrupt Mask Set/Clear Register
const UARTICR: u64 = 0x044;    // Interrupt Clear Register

// Flag register bits
const FR_TXFF: u32 = 1 << 5;   // Transmit FIFO full
const FR_RXFE: u32 = 1 << 4;   // Receive FIFO empty
const FR_BUSY: u32 = 1 << 3;   // UART busy

/// Set the UART base address (called during early init with HHDM offset)
pub fn set_base(base: u64) {
    unsafe { UART_BASE = base; }
}

/// Initialize PL011 UART
pub fn init() {
    unsafe {
        let base = UART_BASE;
        
        // Disable UART
        mmio_write32(base + UARTCR, 0);
        
        // Clear pending interrupts
        mmio_write32(base + UARTICR, 0x7FF);
        
        // Set baud rate to 115200 (assuming 48MHz UART clock on QEMU virt)
        // IBRD = UARTCLK / (16 * BaudRate) = 48000000 / (16 * 115200) = 26
        // FBRD = round((0.041667 * 64) + 0.5) = 3
        mmio_write32(base + UARTIBRD, 26);
        mmio_write32(base + UARTFBRD, 3);
        
        // 8 bits, no parity, 1 stop bit, FIFO enabled
        mmio_write32(base + UARTLCR_H, (0b11 << 5) | (1 << 4));
        
        // Mask all interrupts (we poll)
        mmio_write32(base + UARTIMSC, 0);
        
        // Enable UART, TX and RX
        mmio_write32(base + UARTCR, (1 << 0) | (1 << 8) | (1 << 9));
    }
}

/// Write a single byte to serial
pub fn write_byte(byte: u8) {
    unsafe {
        let base = UART_BASE;
        // Wait until transmit FIFO is not full
        while mmio_read32(base + UARTFR) & FR_TXFF != 0 {
            core::hint::spin_loop();
        }
        mmio_write32(base + UARTDR, byte as u32);
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
        let base = UART_BASE;
        if mmio_read32(base + UARTFR) & FR_RXFE == 0 {
            Some((mmio_read32(base + UARTDR) & 0xFF) as u8)
        } else {
            None
        }
    }
}

/// Check if data is available to read
pub fn data_available() -> bool {
    unsafe {
        let base = UART_BASE;
        mmio_read32(base + UARTFR) & FR_RXFE == 0
    }
}
