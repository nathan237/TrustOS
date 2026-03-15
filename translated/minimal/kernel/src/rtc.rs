



use crate::arch::Port;
use spin::Mutex;


const BMS_: u16 = 0x70;

const BMT_: u16 = 0x71;


const CPL_: u8 = 0x00;
const CPJ_: u8 = 0x02;
const CPH_: u8 = 0x04;
const CPG_: u8 = 0x07;
const CPK_: u8 = 0x08;
const CPO_: u8 = 0x09;
const EDD_: u8 = 0x32; 
const CPM_: u8 = 0x0A;
const CPN_: u8 = 0x0B;


static CPI_: Mutex<()> = Mutex::new(());


#[derive(Clone, Copy, Debug)]
pub struct Aax {
    pub ccq: u16,
    pub caw: u8,
    pub cjw: u8,
    pub bek: u8,
    pub bri: u8,
    pub chr: u8,
}

impl Aax {
    
    pub fn format(&self) -> alloc::string::String {
        alloc::format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.ccq, self.caw, self.cjw,
            self.bek, self.bri, self.chr
        )
    }
    
    
    pub fn yrh(&self) -> alloc::string::String {
        alloc::format!("{:04}-{:02}-{:02}", self.ccq, self.caw, self.cjw)
    }
    
    
    pub fn ivj(&self) -> alloc::string::String {
        alloc::format!("{:02}:{:02}:{:02}", self.bek, self.bri, self.chr)
    }
}


fn enr(reg: u8) -> u8 {
    let mut fzv = Port::<u8>::new(BMS_);
    let mut axr = Port::<u8>::new(BMT_);
    
    unsafe {
        
        fzv.write(0x80 | reg);
        let ap = axr.read();
        
        fzv.write(reg & 0x7F);
        ap
    }
}


fn tzi() -> bool {
    enr(CPM_) & 0x80 != 0
}


fn gau(myg: u8) -> u8 {
    ((myg >> 4) * 10) + (myg & 0x0F)
}


fn pwg() -> Option<Aax> {
    let qci = CPI_.lock();

    
    let mut aaf: u32 = 0;
    while tzi() {
        core::hint::hc();
        aaf = aaf.cn(1);
        if aaf >= 1_000_000 {
            return None;
        }
    }

    
    let mut chr = enr(CPL_);
    let mut bri = enr(CPJ_);
    let mut bek = enr(CPH_);
    let mut cjw = enr(CPG_);
    let mut caw = enr(CPK_);
    let mut ccq = enr(CPO_);

    
    let pol = enr(CPN_);

    
    if pol & 0x04 == 0 {
        chr = gau(chr);
        bri = gau(bri);
        bek = gau(bek & 0x7F) | (bek & 0x80);
        cjw = gau(cjw);
        caw = gau(caw);
        ccq = gau(ccq);
    }

    
    if pol & 0x02 == 0 && bek & 0x80 != 0 {
        bek = ((bek & 0x7F) + 12) % 24;
    }

    
    let sze = 2000u16 + ccq as u16;

    Some(Aax {
        ccq: sze,
        caw,
        cjw,
        bek,
        bri,
        chr,
    })
}


pub fn cgz() -> Aax {
    pwg().unwrap_or(Aax {
        ccq: 2000,
        caw: 1,
        cjw: 1,
        bek: 0,
        bri: 0,
        chr: 0,
    })
}


pub fn nyr() -> u32 {
    let os = cgz();
    os.bek as u32 * 3600 + os.bri as u32 * 60 + os.chr as u32
}


pub fn xmo() -> bool {
    if let Some(os) = pwg() {
        crate::serial_println!("[RTC] Initialized: {}", os.format());
        true
    } else {
        crate::serial_println!("[RTC] Init skipped (no RTC)");
        false
    }
}
