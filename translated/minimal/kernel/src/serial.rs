









use spin::Mutex;
use core::fmt;


static CSE_: Mutex<()> = Mutex::new(());


pub fn init() {
    crate::arch::serial::init();
}


struct Bsm;

impl fmt::Write for Bsm {
    fn write_str(&mut self, e: &str) -> fmt::Result {
        crate::arch::serial::ahx(e.as_bytes());
        Ok(())
    }
}


#[doc(hidden)]
pub fn elt(n: fmt::Arguments) {
    use core::fmt::Write;
    
    
    crate::devtools::qwh(n);
    
    
    crate::arch::cvh(|| {
        let qci = CSE_.lock();
        let mut fyy = Bsm;
        fyy.write_fmt(n).expect("Printing to serial failed");
    });
}


pub fn dlb() -> Option<u8> {
    crate::arch::cvh(|| {
        crate::arch::serial::dlb()
    })
}


pub fn xmu() -> Option<u8> {
    dlb()
}


#[macro_export]
macro_rules! serial_print {
    ($($ji:tt)*) => {
        $crate::serial::elt(format_args!($($ji)*))
    };
}


#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($ji:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($ji)*
    ));
}
