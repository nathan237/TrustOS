



use crate::arch::Port;
use spin::Mutex;


const BPK_: u16 = 0x70;

const BPL_: u16 = 0x71;


const CTA_: u8 = 0x00;
const CSY_: u8 = 0x02;
const CSW_: u8 = 0x04;
const CSV_: u8 = 0x07;
const CSZ_: u8 = 0x08;
const CTD_: u8 = 0x09;
const EGV_: u8 = 0x32; 
const CTB_: u8 = 0x0A;
const CTC_: u8 = 0x0B;


static CSX_: Mutex<()> = Mutex::new(());


#[derive(Clone, Copy, Debug)]
pub struct Js {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Js {
    
    pub fn format(&self) -> alloc::string::String {
        alloc::format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day,
            self.hour, self.minute, self.second
        )
    }
    
    
    pub fn qgd(&self) -> alloc::string::String {
        alloc::format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
    
    
    pub fn format_time(&self) -> alloc::string::String {
        alloc::format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}


fn byd(reg: u8) -> u8 {
    let mut ctj = Port::<u8>::new(BPK_);
    let mut zu = Port::<u8>::new(BPL_);
    
    unsafe {
        
        ctj.write(0x80 | reg);
        let val = zu.read();
        
        ctj.write(reg & 0x7F);
        val
    }
}


fn mua() -> bool {
    byd(CTB_) & 0x80 != 0
}


fn ctx(bcd: u8) -> u8 {
    ((bcd >> 4) * 10) + (bcd & 0x0F)
}


fn jor() -> Option<Js> {
    let jso = CSX_.lock();

    
    let mut my: u32 = 0;
    while mua() {
        core::hint::spin_loop();
        my = my.wrapping_add(1);
        if my >= 1_000_000 {
            return None;
        }
    }

    
    let mut second = byd(CTA_);
    let mut minute = byd(CSY_);
    let mut hour = byd(CSW_);
    let mut day = byd(CSV_);
    let mut month = byd(CSZ_);
    let mut year = byd(CTD_);

    
    let jio = byd(CTC_);

    
    if jio & 0x04 == 0 {
        second = ctx(second);
        minute = ctx(minute);
        hour = ctx(hour & 0x7F) | (hour & 0x80);
        day = ctx(day);
        month = ctx(month);
        year = ctx(year);
    }

    
    if jio & 0x02 == 0 && hour & 0x80 != 0 {
        hour = ((hour & 0x7F) + 12) % 24;
    }

    
    let mak = 2000u16 + year as u16;

    Some(Js {
        year: mak,
        month,
        day,
        hour,
        minute,
        second,
    })
}


pub fn aou() -> Js {
    jor().unwrap_or(Js {
        year: 2000,
        month: 1,
        day: 1,
        hour: 0,
        minute: 0,
        second: 0,
    })
}


pub fn iby() -> u32 {
    let fm = aou();
    fm.hour as u32 * 3600 + fm.minute as u32 * 60 + fm.second as u32
}


pub fn pnw() -> bool {
    if let Some(fm) = jor() {
        crate::serial_println!("[RTC] Initialized: {}", fm.format());
        true
    } else {
        crate::serial_println!("[RTC] Init skipped (no RTC)");
        false
    }
}
