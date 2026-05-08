





























use core::ptr;
use core::fmt;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;






const Arb: usize   = 0x000;

const Aqx: usize    = 0x004;

const Vp: usize   = 0x008;

const Are: usize   = 0x00C;

const Aft: usize = 0x010;

const Bed: usize = 0x014;

const Ara: usize  = 0x018;

const Arh: usize    = 0x020;

const Arg: usize    = 0x024;

const Aqw: usize  = 0x028;

const Aqz: usize = 0x02C;


const DDE_: u32       = 0x03;      
const DDF_: u32 = 0 << 3;
const DDG_: u32    = 0 << 2;


const EMR_: u32     = 1 << 0;    
const EMS_: u32     = 1 << 2;    
const DCY_: u32 = 1 << 6;    
const DCZ_: u32 = 1 << 7;    
const DDA_: u32 = 0x01 << 2; 
const DCX_: u32 = 0x01 << 0; 


const BJZ_: u32   = 1 << 0;    
const BKA_: u32  = 1 << 1;    
const BKB_: u32  = 1 << 2;    

const DDB_: u32 = 0x01 << 4;

const DDC_: u32 = 0x00 << 6;


const DDW_: u32    = 1 << 0;  
const ENH_: u32    = 1 << 1;  
const DDX_: u32     = 1 << 2;  


const DDD_: u32  = 1 << 14;     
const EMT_: u32  = 1 << 6;      






pub struct Wj {
    
    base: u64,
    
    index: u8,
    
    baud_rate: u32,
    
    ref_clock: u32,
    
    tx_count: u64,
    
    rx_count: u64,
    
    initialized: bool,
}


static No: Mutex<[Option<Wj>; 4]> = Mutex::new([None, None, None, None]);
static AMK_: AtomicBool = AtomicBool::new(false);





#[inline(always)]
unsafe fn ede(base: u64, offset: usize) -> u32 {
    let addr = (base as usize + offset) as *const u32;
    ptr::read_volatile(addr)
}

#[inline(always)]
unsafe fn cev(base: u64, offset: usize, val: u32) {
    let addr = (base as usize + offset) as *mut u32;
    ptr::write_volatile(addr, val);
}














pub unsafe fn init(index: u8, base: u64, ref_clock: u32, baud: u32) -> Result<(), &'static str> {
    if index >= 4 {
        return Err("UART index must be 0-3");
    }
    
    crate::serial_println!("[APPLE-UART] Initializing UART{} @ {:#x} ({}baud, {}MHz clock)",
        index, base, baud, ref_clock / 1_000_000);
    
    
    cev(base, Vp, BJZ_ | BKA_ | BKB_);
    
    
    for _ in 0..100 {
        let stat = ede(base, Vp);
        if stat & (BKA_ | BKB_) == 0 {
            break;
        }
    }
    
    
    cev(base, Arb, DDE_ | DDF_ | DDG_);
    
    
    cev(base, Aqx, 
        DCX_ | DDA_ | DCY_ | DCZ_);
    
    
    cev(base, Vp, BJZ_ | DDB_ | DDC_);
    
    
    cev(base, Are, 0);
    
    
    
    
    let hst = ref_clock / baud;
    let han = (hst / 16) - 1;
    let jpb = hst - (han + 1) * 16;
    
    cev(base, Aqw, han);
    cev(base, Aqz, jpb);
    
    crate::serial_println!("[APPLE-UART] UART{}: UBRDIV={}, UFRAC={}", 
        index, han, jpb);
    
    let uart = Wj {
        base,
        index,
        baud_rate: baud,
        ref_clock,
        tx_count: 0,
        rx_count: 0,
        initialized: true,
    };
    
    No.lock()[index as usize] = Some(uart);
    AMK_.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[APPLE-UART] UART{} ready", index);
    Ok(())
}


pub fn write_byte(index: u8, byte: u8) {
    let mut jg = No.lock();
    let uart = match jg[index as usize].as_mut() {
        Some(iy) => iy,
        None => return,
    };
    
    unsafe {
        
        let mut mz = 1_000_000u32;
        while ede(uart.base, Ara) & DDD_ != 0 {
            mz -= 1;
            if mz == 0 {
                return; 
            }
        }
        
        cev(uart.base, Arh, byte as u32);
        uart.tx_count += 1;
    }
}



pub fn read_byte(index: u8) -> Option<u8> {
    let mut jg = No.lock();
    let uart = match jg[index as usize].as_mut() {
        Some(iy) => iy,
        None => return None,
    };
    
    unsafe {
        let status = ede(uart.base, Aft);
        if status & DDW_ != 0 {
            let byte = ede(uart.base, Arg) as u8;
            uart.rx_count += 1;
            Some(byte)
        } else {
            None
        }
    }
}


pub fn write_str(index: u8, j: &str) {
    for byte in j.bytes() {
        if byte == b'\n' {
            write_byte(index, b'\r'); 
        }
        write_byte(index, byte);
    }
}


pub fn qrv(index: u8, buf: &mut [u8]) -> usize {
    let mut count = 0;
    while count < buf.len() {
        match read_byte(index) {
            Some(b) => {
                buf[count] = b;
                count += 1;
            }
            None => break,
        }
    }
    count
}


pub fn rbb(index: u8) -> bool {
    let jg = No.lock();
    match jg[index as usize].as_ref() {
        Some(uart) => unsafe {
            ede(uart.base, Aft) & DDX_ != 0
        },
        None => true,
    }
}


pub fn is_initialized() -> bool {
    AMK_.load(Ordering::SeqCst)
}


pub fn status(index: u8) -> Option<alloc::string::String> {
    let jg = No.lock();
    jg[index as usize].as_ref().map(|uart| {
        alloc::format!(
            "UART{} @ {:#x}: {}baud, TX:{} RX:{}, clock:{}MHz",
            uart.index, uart.base, uart.baud_rate,
            uart.tx_count, uart.rx_count,
            uart.ref_clock / 1_000_000
        )
    })
}


pub struct Afu {
    index: u8,
}

impl Afu {
    pub const fn new(index: u8) -> Self {
        Self { index }
    }
}

impl fmt::Write for Afu {
    fn write_str(&mut self, j: &str) -> fmt::Result {
        write_str(self.index, j);
        Ok(())
    }
}
