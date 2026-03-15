




use core::sync::atomic::{AtomicU64, Ordering};


static BHB_: AtomicU64 = AtomicU64::new(0);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LogLevel {
    Ze = 0,
    Debug = 1,
    V = 2,
    Bwq = 3,
    Q = 4,
    Nd = 5,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Ze => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::V => "INFO ",
            LogLevel::Bwq => "WARN ",
            LogLevel::Q => "ERROR",
            LogLevel::Nd => "FATAL",
        }
    }
}


pub fn fjp() -> u64 {
    BHB_.load(Ordering::Relaxed)
}


pub fn lh() -> u64 {
    fjp()
}


pub fn or() {
    BHB_.fetch_add(1, Ordering::Relaxed);
}


#[doc(hidden)]
pub fn qcj(jy: LogLevel, n: core::fmt::Arguments) {
    let aea = fjp();
    let qq = 0u8; 
    
    crate::serial::elt(format_args!(
        "[{:>10}][CPU{}][{}] {}\n",
        aea,
        qq,
        jy.as_str(),
        n
    ));
}


#[macro_export]
macro_rules! jdx {
    ($jy:expr, $($ji:tt)*) => {
        $crate::logger::qcj($jy, format_args!($($ji)*))
    };
}


#[macro_export]
macro_rules! log {
    ($($ji:tt)*) => {
        $crate::jdx!($crate::logger::LogLevel::V, $($ji)*)
    };
}


#[macro_export]
macro_rules! log_debug {
    ($($ji:tt)*) => {
        $crate::jdx!($crate::logger::LogLevel::Debug, $($ji)*)
    };
}


#[macro_export]
macro_rules! log_warn {
    ($($ji:tt)*) => {
        $crate::jdx!($crate::logger::LogLevel::Bwq, $($ji)*)
    };
}


#[macro_export]
macro_rules! log_error {
    ($($ji:tt)*) => {
        $crate::jdx!($crate::logger::LogLevel::Q, $($ji)*)
    };
}
