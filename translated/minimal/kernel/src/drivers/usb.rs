




use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UsbControllerType {
    Buz,   
    Bnp,   
    Ark,   
    Bbe,   
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UsbDeviceClass {
    Cyn,
    Cer,         
    Dci,
    Rj,
    Zj,
    Dex,
    Qg(u8),
}


#[derive(Clone, Debug)]
pub struct Bvl {
    pub re: u8,
    pub class: UsbDeviceClass,
    pub ml: u16,
    pub cgt: u16,
    pub lkg: String,
    pub baj: String,
}


pub struct Bah {
    pub nft: UsbControllerType,
    pub sm: u64,
    pub ik: Vec<Bvl>,
    pub jr: bool,
}

static Vt: Mutex<Vec<Bah>> = Mutex::new(Vec::new());


pub fn ky() -> bool {
    Vt.lock().iter().any(|r| r.jr)
}


pub fn yxp(ar: u64) -> bool {
    if ar == 0 || ar == 0xFFFFFFFF {
        return false;
    }
    
    crate::serial_println!("[USB] EHCI controller at {:#x}", ar);
    
    
    
    
    let df = Bah {
        nft: UsbControllerType::Ark,
        sm: ar,
        ik: Vec::new(),
        jr: false,  
    };
    
    Vt.lock().push(df);
    
    
    
    
    
    
    
    
    false  
}


pub fn yxw(ar: u64) -> bool {
    if ar == 0 || ar == 0xFFFFFFFF {
        return false;
    }
    
    crate::serial_println!("[USB] xHCI controller at {:#x}", ar);
    
    let df = Bah {
        nft: UsbControllerType::Bbe,
        sm: ar,
        ik: Vec::new(),
        jr: false,  
    };
    
    Vt.lock().push(df);
    
    
    
    
    
    
    
    
    
    
    
    false  
}


pub fn smj() -> Vec<Bvl> {
    Vt.lock()
        .iter()
        .iva(|r| r.ik.clone())
        .collect()
}


pub fn roo() -> usize {
    Vt.lock().len()
}


pub fn tmo() -> bool {
    Vt.lock()
        .iter()
        .any(|r| r.ik.iter().any(|bc| bc.class == UsbDeviceClass::Cer))
}
