//! RISC-V 64 Serial Port Driver (16550 UART via MMIO)
//!
//! Most RISC-V boards (QEMU virt, SiFive) use a 16550-compatible UART
//! but accessed via MMIO instead of I/O ports.
//! On QEMU virt, the UART is at 0x1000_0000.

use super::cpu::{mmio_read8, mmio_write8};

/// UART base address for QEMU virt machine
/// On SiFive HiFive boards, this may be different
static mut UART_BASE: u64 = 0x1000_0000;

// 16550 register offsets
const THR: u64 = 0;   // Transmit Holding Register (write)
const RBR: u64 = 0;   // Receive Buffer Register (read)
const IER: u64 = 1;   // Interrupt Enable Register
const FCR: u64 = 2;   // FIFO Control Register (write)
const LCR: u64 = 3;   // Line Control Register
const MCR: u64 = 4;   // Modem Control Register
const LSR: u64 = 5;   // Line Status Register
const DLL: u64 = 0;   // Divisor Latch Low (when DLAB=1)
const DLH: u64 = 1;   // Divisor Latch High (when DLAB=1)

// LSR bits
const LSR_DATA_READY: u8 = 1 << 0;
const LSR_THR_EMPTY: u8 = 1 << 5;

/// Set the UART base address (called during early init with HHDM offset)
pub fn set_base(base: u64) {
    unsafe { UART_BASE = base; }
}

/// Initialize 16550 UART via MMIO
pub fn init() {
    unsafe {
        let base = UART_BASE;
        
        // Disable interrupts
        mmio_write8(base + IER, 0x00);
        
        // Enable DLAB (set baud rate divisor)
        mmio_write8(base + LCR, 0x80);
        
        // Set divisor to 1 (115200 baud with standard clock)
        mmio_write8(base + DLL, 0x01);
        mmio_write8(base + DLH, 0x00);
        
        // 8 bits, no parity, one stop bit, DLAB off
        mmio_write8(base + LCR, 0x03);
        
        // Enable FIFO, clear them, 14-byte threshold
        mmio_write8(base + FCR, 0xC7);
        
        // IRQs enabled, RTS/DSR set
        mmio_write8(base + MCR, 0x0B);
    }
}

/// Write a single byte to serial
pub fn write_byte(byte: u8) {
    unsafe {
        let base = UART_BASE;
        // Wait until transmit buffer is empty
        while mmio_read8(base + LSR) & LSR_THR_EMPTY == 0 {
            core::hint::spin_loop();
        }
        mmio_write8(base + THR, byte);
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
        if mmio_read8(base + LSR) & LSR_DATA_READY != 0 {
            Some(mmio_read8(base + RBR))
        } else {
            None
        }
    }
}

/// Check if data is available to read
pub fn data_available() -> bool {
    unsafe {
        let base = UART_BASE;
        mmio_read8(base + LSR) & LSR_DATA_READY != 0
    }
}
