//! TrustOS I/O Utilities
//!
//! Provides print!, println! macros and basic I/O functions.

use core::fmt::{self, Write};
use crate::syscall;

/// Writer that outputs to stdout
pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        syscall::write(syscall::STDOUT, s.as_bytes())
            .map(|_| ())
            .map_err(|_| fmt::Error)
    }
}

/// Writer that outputs to stderr
pub struct Stderr;

impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        syscall::write(syscall::STDERR, s.as_bytes())
            .map(|_| ())
            .map_err(|_| fmt::Error)
    }
}

/// Print to stdout
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($crate::io::Stdout, $($arg)*);
    }};
}

/// Print to stdout with newline
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($crate::io::Stdout, $($arg)*);
        let _ = $crate::io::Stdout.write_str("\n");
    }};
}

/// Print to stderr
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($crate::io::Stderr, $($arg)*);
    }};
}

/// Print to stderr with newline
#[macro_export]
macro_rules! eprintln {
    () => { $crate::eprint!("\n") };
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($crate::io::Stderr, $($arg)*);
        let _ = $crate::io::Stderr.write_str("\n");
    }};
}

/// Read a line from stdin into a buffer
/// Returns the number of bytes read (excluding null terminator)
pub fn read_line(buf: &mut [u8]) -> Result<usize, i64> {
    syscall::read(syscall::STDIN, buf)
}

/// Print a string directly (no formatting)
pub fn puts(s: &str) {
    let _ = syscall::write(syscall::STDOUT, s.as_bytes());
}

/// Print a string with newline
pub fn puts_ln(s: &str) {
    let _ = syscall::write(syscall::STDOUT, s.as_bytes());
    let _ = syscall::write(syscall::STDOUT, b"\n");
}
