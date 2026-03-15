//! Apple S5L UART Driver (Samsung-derived)
//!
//! All Apple SoCs (A7–A17, M1–M4) use a UART derived from Samsung's
//! Exynos serial controller (S5L8900 lineage). This is NOT an ARM PL011.
//!
//! The Samsung UART has a different register layout:
//! - ULCON (line control) at offset 0x00
//! - UCON  (control) at 0x04  
//! - UFCON (FIFO control) at 0x08
//! - UMCON (modem control) at 0x0C
//! - UTRSTAT (TX/RX status) at 0x10
//! - UTXH (TX holding) at 0x20
//! - URXH (RX holding) at 0x24
//! - UBRDIV (baud rate divisor) at 0x28
//!
//! On Apple Silicon specifically, the UART is further customized:
//! - FIFO depth varies (16 or 64 entries)
//! - Clock source from PMGR (not standard ARM clock tree)
//! - Often at 115200 baud by default (iBoot configures it)
//!
//! Typical UART0 base addresses (from Device Tree):
//!   A10 (T8010, iPhone 7):   0x235200000
//!   A11 (T8015, iPhone 8/X): 0x235200000
//!   A12 (T8020, iPhone XR):  0x235010000
//!
//! References:
//! - Asahi Linux: drivers/tty/serial/samsung_tty.c
//! - Linux: Documentation/devicetree/bindings/serial/samsung_uart.yaml
//! - m1n1 (Asahi bootloader): uart.py

use core::ptr;
use core::fmt;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════
// Samsung UART Register Offsets (Apple variant)
// ═══════════════════════════════════════════════════════════════════════════

/// Line Control Register — data bits, stop bits, parity
const ULCON: usize   = 0x000;
/// Control Register — TX/RX enable, IRQ mode, DMA mode
const UCON: usize    = 0x004;
/// FIFO Control Register — FIFO enable, trigger levels, reset
const UFCON: usize   = 0x008;
/// Modem Control Register — RTS/CTS flow control
const UMCON: usize   = 0x00C;
/// TX/RX Status Register — buffer empty/full, errors
const UTRSTAT: usize = 0x010;
/// Error Status Register — overrun, break, frame, parity errors
const UERSTAT: usize = 0x014;
/// FIFO Status Register — TX/RX FIFO fill levels
const UFSTAT: usize  = 0x018;
/// TX Buffer (write byte here to transmit)
const UTXH: usize    = 0x020;
/// RX Buffer (read byte here to receive)
const URXH: usize    = 0x024;
/// Baud Rate Divisor
const UBRDIV: usize  = 0x028;
/// Fractional Baud Rate Divider (Samsung extension)
const UFRACVAL: usize = 0x02C;

// ULCON bits
const ULCON_8N1: u32       = 0x03;      // 8 data bits, no parity, 1 stop bit
const ULCON_PARITY_NONE: u32 = 0 << 3;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const ULCON_STOP_1: u32    = 0 << 2;

// UCON bits  
const UCON_RECEIVE_INTERRUPT_REQUEST: u32     = 1 << 0;    // RX interrupt or polling mode
const UCON_TRANSMIT_INTERRUPT_REQUEST: u32     = 1 << 2;    // TX interrupt or polling mode
const UCON_RECEIVE_ERROR_INTERRUPT_REQUEST: u32 = 1 << 6;    // RX error interrupt
const UCON_RECEIVE_TIMEOUT: u32 = 1 << 7;    // RX timeout enable
const UCON_TXMODE_INTERRUPT_REQUEST: u32 = 0x01 << 2; // TX mode: interrupt
const UCON_RXMODE_INTERRUPT_REQUEST: u32 = 0x01 << 0; // RX mode: interrupt

// UFCON bits
const UFCON_FIFO_EN: u32   = 1 << 0;    // Enable FIFO
const UFCON_RECEIVE_RESET: u32  = 1 << 1;    // Reset RX FIFO
const UFCON_TRANSMIT_RESET: u32  = 1 << 2;    // Reset TX FIFO
// RX trigger level: 1/4 FIFO
const UFCON_RECEIVE_TRIG_8: u32 = 0x01 << 4;
// TX trigger level: empty
const UFCON_TRANSMIT_TRIG_0: u32 = 0x00 << 6;

// UTRSTAT bits
const UTRSTAT_RECEIVE_READY: u32    = 1 << 0;  // RX buffer has data
const UTRSTAT_TRANSMIT_EMPTY: u32    = 1 << 1;  // TX buffer empty
const UTRSTAT_TRANSMIT_DONE: u32     = 1 << 2;  // TX shifter empty (fully sent)

// UFSTAT bits (Apple-specific FIFO depths may differ)
const UFSTAT_TRANSMIT_FULL: u32  = 1 << 14;     // TX FIFO full
const UFSTAT_RECEIVE_FULL: u32  = 1 << 6;      // RX FIFO full (varies by HW)

// ═══════════════════════════════════════════════════════════════════════════
// Driver State
// ═══════════════════════════════════════════════════════════════════════════

/// A single Apple UART instance
pub struct AppleUart {
    /// MMIO base address
    base: u64,
    /// UART index (0-3)
    index: u8,
    /// Current baud rate
    baud_rate: u32,
    /// Reference clock frequency (from PMGR, typically 24 MHz)
    ref_clock: u32,
    /// Total bytes transmitted
    transmit_count: u64,
    /// Total bytes received
    receive_count: u64,
    /// Whether this UART is initialized
    initialized: bool,
}

/// Up to 4 UART instances (Apple SoCs typically have 3-8)
static UARTS: Mutex<[Option<AppleUart>; 4]> = Mutex::new([None, None, None, None]);
// Atomic variable — provides lock-free thread-safe access.
static ANY_UART_INITIALIZE: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════
// MMIO helpers
// ═══════════════════════════════════════════════════════════════════════════

#[inline(always)]
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn uart_read32(base: u64, offset: usize) -> u32 {
    let address = (base as usize + offset) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32;
    ptr::read_volatile(address)
}

#[inline(always)]
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn uart_write32(base: u64, offset: usize, value: u32) {
    let address = (base as usize + offset) as *mut u32;
    ptr::write_volatile(address, value);
}

// ═══════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════

/// Initialize a UART at the given MMIO base.
///
/// `index`: which UART (0-3)
/// `base`: physical MMIO address (from Device Tree)
/// `ref_clock`: reference clock in Hz (typically 24_000_000 for Apple)
/// `baud`: desired baud rate (typically 115200)
///
/// # Safety
/// Caller must ensure `base` points to valid Apple UART MMIO.
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn init(index: u8, base: u64, ref_clock: u32, baud: u32) -> Result<(), &'static str> {
    if index >= 4 {
        return Err("UART index must be 0-3");
    }
    
    crate::serial_println!("[APPLE-UART] Initializing UART{} @ {:#x} ({}baud, {}MHz clock)",
        index, base, baud, ref_clock / 1_000_000);
    
    // Reset FIFOs
    uart_write32(base, UFCON, UFCON_FIFO_EN | UFCON_RECEIVE_RESET | UFCON_TRANSMIT_RESET);
    
    // Wait for FIFO reset to complete (few cycles)
    for _ in 0..100 {
        let status = uart_read32(base, UFCON);
        if status & (UFCON_RECEIVE_RESET | UFCON_TRANSMIT_RESET) == 0 {
            break;
        }
    }
    
    // Configure line: 8N1
    uart_write32(base, ULCON, ULCON_8N1 | ULCON_PARITY_NONE | ULCON_STOP_1);
    
    // Configure control: polling mode (simplest), RX error interrupt
    uart_write32(base, UCON, 
        UCON_RXMODE_INTERRUPT_REQUEST | UCON_TXMODE_INTERRUPT_REQUEST | UCON_RECEIVE_ERROR_INTERRUPT_REQUEST | UCON_RECEIVE_TIMEOUT);
    
    // Configure FIFO: enable, RX trigger at 8 bytes, TX trigger at empty
    uart_write32(base, UFCON, UFCON_FIFO_EN | UFCON_RECEIVE_TRIG_8 | UFCON_TRANSMIT_TRIG_0);
    
    // No modem control (no hardware flow control)
    uart_write32(base, UMCON, 0);
    
    // Set baud rate
    // Formula: divisor = (ref_clock / (baud * 16)) - 1
    // Fractional: frac = ((ref_clock % (baud * 16)) * 16) / (baud * 16)
    let divisor_16 = ref_clock / baud;
    let ubrdiv_value = (divisor_16 / 16) - 1;
    let ufrac_value = divisor_16 - (ubrdiv_value + 1) * 16;
    
    uart_write32(base, UBRDIV, ubrdiv_value);
    uart_write32(base, UFRACVAL, ufrac_value);
    
    crate::serial_println!("[APPLE-UART] UART{}: UBRDIV={}, UFRAC={}", 
        index, ubrdiv_value, ufrac_value);
    
    let uart = AppleUart {
        base,
        index,
        baud_rate: baud,
        ref_clock,
        transmit_count: 0,
        receive_count: 0,
        initialized: true,
    };
    
    UARTS.lock()[index as usize] = Some(uart);
    ANY_UART_INITIALIZE.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[APPLE-UART] UART{} ready", index);
    Ok(())
}

/// Write a single byte to the UART (blocking, waits for TX space)
pub fn write_byte(index: u8, byte: u8) {
    let mut guard = UARTS.lock();
    let uart = // Pattern matching — Rust's exhaustive branching construct.
match guard[index as usize].as_mut() {
        Some(u) => u,
        None => return,
    };
    
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        // Wait for TX buffer space
        let mut timeout = 1_000_000u32;
        while uart_read32(uart.base, UFSTAT) & UFSTAT_TRANSMIT_FULL != 0 {
            timeout -= 1;
            if timeout == 0 {
                return; // Timeout — don't hang forever
            }
        }
        
        uart_write32(uart.base, UTXH, byte as u32);
        uart.transmit_count += 1;
    }
}

/// Read a single byte from the UART (non-blocking)
/// Returns None if no data available
pub fn read_byte(index: u8) -> Option<u8> {
    let mut guard = UARTS.lock();
    let uart = // Pattern matching — Rust's exhaustive branching construct.
match guard[index as usize].as_mut() {
        Some(u) => u,
        None => return None,
    };
    
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let status = uart_read32(uart.base, UTRSTAT);
        if status & UTRSTAT_RECEIVE_READY != 0 {
            let byte = uart_read32(uart.base, URXH) as u8;
            uart.receive_count += 1;
            Some(byte)
        } else {
            None
        }
    }
}

/// Write a string to the UART
pub fn write_str(index: u8, s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' {
            write_byte(index, b'\r'); // CR before LF for serial terminals
        }
        write_byte(index, byte);
    }
}

/// Read all available bytes from RX FIFO
pub fn read_available(index: u8, buffer: &mut [u8]) -> usize {
    let mut count = 0;
    while count < buffer.len() {
                // Pattern matching — Rust's exhaustive branching construct.
match read_byte(index) {
            Some(b) => {
                buffer[count] = b;
                count += 1;
            }
            None => break,
        }
    }
    count
}

/// Check if TX is completely empty (all data sent)
pub fn transmit_done(index: u8) -> bool {
    let guard = UARTS.lock();
        // Pattern matching — Rust's exhaustive branching construct.
match guard[index as usize].as_ref() {
        Some(uart) => // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            uart_read32(uart.base, UTRSTAT) & UTRSTAT_TRANSMIT_DONE != 0
        },
        None => true,
    }
}

/// Check if any Apple UART is initialized
pub fn is_initialized() -> bool {
    ANY_UART_INITIALIZE.load(Ordering::SeqCst)
}

/// Get UART status for shell display
pub fn status(index: u8) -> Option<alloc::string::String> {
    let guard = UARTS.lock();
    guard[index as usize].as_ref().map(|uart| {
        alloc::format!(
            "UART{} @ {:#x}: {}baud, TX:{} RX:{}, clock:{}MHz",
            uart.index, uart.base, uart.baud_rate,
            uart.transmit_count, uart.receive_count,
            uart.ref_clock / 1_000_000
        )
    })
}

/// Implement core::fmt::Write for a UART index (for write!/writeln! macros)
pub struct UartWriter {
    index: u8,
}

// Implementation block — defines methods for the type above.
impl UartWriter {
    pub const fn new(index: u8) -> Self {
        Self { index }
    }
}

// Trait implementation — fulfills a behavioral contract.
impl fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str(self.index, s);
        Ok(())
    }
}
