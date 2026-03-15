





























use core::ptr;
use core::fmt;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;






const Cop: usize   = 0x000;

const Col: usize    = 0x004;

const Bad: usize   = 0x008;

const Cor: usize   = 0x00C;

const Bvd: usize = 0x010;

const Djz: usize = 0x014;

const Coo: usize  = 0x018;

const Cov: usize    = 0x020;

const Cou: usize    = 0x024;

const Cok: usize  = 0x028;

const Con: usize = 0x02C;


const CZM_: u32       = 0x03;      
const CZN_: u32 = 0 << 3;
const CZO_: u32    = 0 << 2;


const EJC_: u32     = 1 << 0;    
const EJD_: u32     = 1 << 2;    
const CZG_: u32 = 1 << 6;    
const CZH_: u32 = 1 << 7;    
const CZI_: u32 = 0x01 << 2; 
const CZF_: u32 = 0x01 << 0; 


const BHV_: u32   = 1 << 0;    
const BHW_: u32  = 1 << 1;    
const BHX_: u32  = 1 << 2;    

const CZJ_: u32 = 0x01 << 4;

const CZK_: u32 = 0x00 << 6;


const DAE_: u32    = 1 << 0;  
const EJT_: u32    = 1 << 1;  
const DAF_: u32     = 1 << 2;  


const CZL_: u32  = 1 << 14;     
const EJE_: u32  = 1 << 6;      






pub struct Bbw {
    
    ar: u64,
    
    index: u8,
    
    myf: u32,
    
    hxg: u32,
    
    mnm: u64,
    
    mbj: u64,
    
    jr: bool,
}


static Afb: Mutex<[Option<Bbw>; 4]> = Mutex::new([None, None, None, None]);
static AKQ_: AtomicBool = AtomicBool::new(false);





#[inline(always)]
unsafe fn ifr(ar: u64, l: usize) -> u32 {
    let ag = (ar as usize + l) as *const u32;
    ptr::read_volatile(ag)
}

#[inline(always)]
unsafe fn faq(ar: u64, l: usize, ap: u32) {
    let ag = (ar as usize + l) as *mut u32;
    ptr::write_volatile(ag, ap);
}














pub unsafe fn init(index: u8, ar: u64, hxg: u32, kci: u32) -> Result<(), &'static str> {
    if index >= 4 {
        return Err("UART index must be 0-3");
    }
    
    crate::serial_println!("[APPLE-UART] Initializing UART{} @ {:#x} ({}baud, {}MHz clock)",
        index, ar, kci, hxg / 1_000_000);
    
    
    faq(ar, Bad, BHV_ | BHW_ | BHX_);
    
    
    for _ in 0..100 {
        let hm = ifr(ar, Bad);
        if hm & (BHW_ | BHX_) == 0 {
            break;
        }
    }
    
    
    faq(ar, Cop, CZM_ | CZN_ | CZO_);
    
    
    faq(ar, Col, 
        CZF_ | CZI_ | CZG_ | CZH_);
    
    
    faq(ar, Bad, BHV_ | CZJ_ | CZK_);
    
    
    faq(ar, Cor, 0);
    
    
    
    
    let nmc = hxg / kci;
    let mny = (nmc / 16) - 1;
    let pwy = nmc - (mny + 1) * 16;
    
    faq(ar, Cok, mny);
    faq(ar, Con, pwy);
    
    crate::serial_println!("[APPLE-UART] UART{}: UBRDIV={}, UFRAC={}", 
        index, mny, pwy);
    
    let uart = Bbw {
        ar,
        index,
        myf: kci,
        hxg,
        mnm: 0,
        mbj: 0,
        jr: true,
    };
    
    Afb.lock()[index as usize] = Some(uart);
    AKQ_.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[APPLE-UART] UART{} ready", index);
    Ok(())
}


pub fn cco(index: u8, hf: u8) {
    let mut adb = Afb.lock();
    let uart = match adb[index as usize].as_mut() {
        Some(tm) => tm,
        None => return,
    };
    
    unsafe {
        
        let mut aah = 1_000_000u32;
        while ifr(uart.ar, Coo) & CZL_ != 0 {
            aah -= 1;
            if aah == 0 {
                return; 
            }
        }
        
        faq(uart.ar, Cov, hf as u32);
        uart.mnm += 1;
    }
}



pub fn dlb(index: u8) -> Option<u8> {
    let mut adb = Afb.lock();
    let uart = match adb[index as usize].as_mut() {
        Some(tm) => tm,
        None => return None,
    };
    
    unsafe {
        let status = ifr(uart.ar, Bvd);
        if status & DAE_ != 0 {
            let hf = ifr(uart.ar, Cou) as u8;
            uart.mbj += 1;
            Some(hf)
        } else {
            None
        }
    }
}


pub fn write_str(index: u8, e: &str) {
    for hf in e.bf() {
        if hf == b'\n' {
            cco(index, b'\r'); 
        }
        cco(index, hf);
    }
}


pub fn zhj(index: u8, k: &mut [u8]) -> usize {
    let mut az = 0;
    while az < k.len() {
        match dlb(index) {
            Some(o) => {
                k[az] = o;
                az += 1;
            }
            None => break,
        }
    }
    az
}


pub fn zts(index: u8) -> bool {
    let adb = Afb.lock();
    match adb[index as usize].as_ref() {
        Some(uart) => unsafe {
            ifr(uart.ar, Bvd) & DAF_ != 0
        },
        None => true,
    }
}


pub fn ky() -> bool {
    AKQ_.load(Ordering::SeqCst)
}


pub fn status(index: u8) -> Option<alloc::string::String> {
    let adb = Afb.lock();
    adb[index as usize].as_ref().map(|uart| {
        alloc::format!(
            "UART{} @ {:#x}: {}baud, TX:{} RX:{}, clock:{}MHz",
            uart.index, uart.ar, uart.myf,
            uart.mnm, uart.mbj,
            uart.hxg / 1_000_000
        )
    })
}


pub struct Bve {
    index: u8,
}

impl Bve {
    pub const fn new(index: u8) -> Self {
        Self { index }
    }
}

impl fmt::Write for Bve {
    fn write_str(&mut self, e: &str) -> fmt::Result {
        write_str(self.index, e);
        Ok(())
    }
}
