//! Kernel logging subsystem
//! 
//! Provides structured logging with timestamps and log levels
//! for kernel debugging and observability.

use core::sync::atomic::{AtomicU64, Ordering};

/// Global tick counter for timestamps
static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO ",
            LogLevel::Warn => "WARN ",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
}

/// Get current tick count as timestamp
pub fn get_timestamp() -> u64 {
    TICK_COUNTER.load(Ordering::Relaxed)
}

/// Alias for get_timestamp
pub fn get_ticks() -> u64 {
    get_timestamp()
}

/// Increment tick counter (called by timer interrupt)
pub fn tick() {
    TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Internal log function
#[doc(hidden)]
pub fn _log(level: LogLevel, args: core::fmt::Arguments) {
    let timestamp = get_timestamp();
    let cpu_id = 0u8; // TODO: Get actual CPU ID
    
    crate::serial::_print(format_args!(
        "[{:>10}][CPU{}][{}] {}\n",
        timestamp,
        cpu_id,
        level.as_str(),
        args
    ));
}

/// Log macro with level
#[macro_export]
macro_rules! log_level {
    ($level:expr, $($arg:tt)*) => {
        $crate::logger::_log($level, format_args!($($arg)*))
    };
}

/// Info log (default)
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::log_level!($crate::logger::LogLevel::Info, $($arg)*)
    };
}

/// Debug log
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::log_level!($crate::logger::LogLevel::Debug, $($arg)*)
    };
}

/// Warning log
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::log_level!($crate::logger::LogLevel::Warn, $($arg)*)
    };
}

/// Error log
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::log_level!($crate::logger::LogLevel::Error, $($arg)*)
    };
}
