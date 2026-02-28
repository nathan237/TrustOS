//! Serial port driver for debug output
//! 
//! Provides serial communication via architecture-specific UART for early boot
//! debugging and kernel logging.
//!
//! Uses the architecture abstraction layer (`crate::arch::serial`) to support:
//! - x86_64: UART 16550 via I/O ports (COM1 @ 0x3F8)
//! - aarch64: PL011 UART via MMIO (QEMU virt @ 0x0900_0000)
//! - riscv64: 16550 UART via MMIO (QEMU virt @ 0x1000_0000)

use spin::Mutex;
use core::fmt;

/// Global serial lock (prevents interleaved output from multiple cores)
static SERIAL_LOCK: Mutex<()> = Mutex::new(());

/// Initialize serial port via architecture abstraction
pub fn init() {
    crate::arch::serial::init();
}

/// Architecture-portable serial writer (implements core::fmt::Write)
struct SerialWriter;

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        crate::arch::serial::write_bytes(s.as_bytes());
        Ok(())
    }
}

/// Print to serial port (internal use) — also captures to dmesg ring buffer
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    
    // Capture to dmesg ring buffer (best-effort, never panic)
    crate::devtools::capture_serial_line(args);
    
    // Disable interrupts to prevent deadlock, then write
    crate::arch::without_interrupts(|| {
        let _lock = SERIAL_LOCK.lock();
        let mut writer = SerialWriter;
        writer.write_fmt(args).expect("Printing to serial failed");
    });
}

/// Try to read a byte from serial (non-blocking)
pub fn read_byte() -> Option<u8> {
    crate::arch::without_interrupts(|| {
        crate::arch::serial::read_byte()
    })
}

/// Try to read a byte from serial (alias for read_byte, for syscall compatibility)
pub fn try_read_byte() -> Option<u8> {
    read_byte()
}

/// Print to serial port
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

/// Print to serial port with newline
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*
    ));
}
