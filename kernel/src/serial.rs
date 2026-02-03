//! Serial port driver for debug output
//! 
//! Provides serial communication via UART 16550 for early boot debugging
//! and kernel logging.

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;
use core::fmt;
use x86_64::instructions::port::Port;

/// Standard COM1 port address
const SERIAL_IO_PORT: u16 = 0x3F8;

lazy_static! {
    /// Global serial port instance protected by spinlock
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(SERIAL_IO_PORT) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

/// Initialize serial port
pub fn init() {
    // Serial port is lazily initialized on first use
    // Force initialization here
    let _ = SERIAL1.lock();
}

/// Print to serial port (internal use)
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    
    // Disable interrupts to prevent deadlock
    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// Try to read a byte from serial (non-blocking)
pub fn read_byte() -> Option<u8> {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        let mut lsr = Port::<u8>::new(SERIAL_IO_PORT + 5);
        let status: u8 = unsafe { lsr.read() };
        if status & 0x01 == 0 {
            return None;
        }

        let mut data = Port::<u8>::new(SERIAL_IO_PORT);
        Some(unsafe { data.read() })
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
