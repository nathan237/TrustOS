









use spin::Mutex;
use core::fmt;


static CVV_: Mutex<()> = Mutex::new(());


pub fn init() {
    crate::arch::serial::init();
}


struct Aes;

impl fmt::Write for Aes {
    fn write_str(&mut self, j: &str) -> fmt::Result {
        crate::arch::serial::write_bytes(j.as_bytes());
        Ok(())
    }
}


#[doc(hidden)]
pub fn bxg(args: fmt::Arguments) {
    use core::fmt::Write;
    
    
    crate::devtools::khl(args);
    
    
    if crate::debug::netconsole::lq() && crate::memory::heap::free() > 0 {
        let j = alloc::format!("{}", args);
        crate::debug::netconsole::onu(&j);
    }
    
    
    crate::arch::bag(|| {
        let jso = CVV_.lock();
        let mut writer = Aes;
        writer.write_fmt(args).expect("Printing to serial failed");
    });
}


pub fn read_byte() -> Option<u8> {
    crate::arch::bag(|| {
        crate::arch::serial::read_byte()
    })
}


pub fn poa() -> Option<u8> {
    read_byte()
}


#[macro_export]
macro_rules! serial_print {
    ($($db:tt)*) => {
        $crate::serial::bxg(format_args!($($db)*))
    };
}


#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($db:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($db)*
    ));
}
