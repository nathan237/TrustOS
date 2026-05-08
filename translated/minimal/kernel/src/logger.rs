




use core::sync::atomic::{AtomicU64, Ordering};


static BJF_: AtomicU64 = AtomicU64::new(0);


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


pub fn ckc() -> u64 {
    BJF_.load(Ordering::Relaxed)
}


pub fn eg() -> u64 {
    ckc()
}


pub fn tick() {
    BJF_.fetch_add(1, Ordering::Relaxed);
}


#[doc(hidden)]
pub fn jsp(level: LogLevel, args: core::fmt::Arguments) {
    let timestamp = ckc();
    let cpu_id = 0u8; 
    
    crate::serial::bxg(format_args!(
        "[{:>10}][CPU{}][{}] {}\n",
        timestamp,
        cpu_id,
        level.as_str(),
        args
    ));
}


#[macro_export]
macro_rules! eth {
    ($level:expr, $($db:tt)*) => {
        $crate::logger::jsp($level, format_args!($($db)*))
    };
}


#[macro_export]
macro_rules! log {
    ($($db:tt)*) => {
        $crate::eth!($crate::logger::LogLevel::Info, $($db)*)
    };
}


#[macro_export]
macro_rules! log_debug {
    ($($db:tt)*) => {
        $crate::eth!($crate::logger::LogLevel::Debug, $($db)*)
    };
}


#[macro_export]
macro_rules! log_warn {
    ($($db:tt)*) => {
        $crate::eth!($crate::logger::LogLevel::Warn, $($db)*)
    };
}


#[macro_export]
macro_rules! log_error {
    ($($db:tt)*) => {
        $crate::eth!($crate::logger::LogLevel::Error, $($db)*)
    };
}
