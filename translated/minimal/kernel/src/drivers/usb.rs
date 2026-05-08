




use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UsbControllerType {
    Afp,   
    Abw,   
    Rv,   
    Wa,   
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UsbDeviceClass {
    Hub,
    HID,         
    MassStorage,
    Audio,
    Video,
    Printer,
    Other(u8),
}


#[derive(Clone, Debug)]
pub struct Afy {
    pub address: u8,
    pub class: UsbDeviceClass,
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: String,
    pub product: String,
}


pub struct Vr {
    pub controller_type: UsbControllerType,
    pub base_addr: u64,
    pub devices: Vec<Afy>,
    pub initialized: bool,
}

static Jm: Mutex<Vec<Vr>> = Mutex::new(Vec::new());


pub fn is_initialized() -> bool {
    Jm.lock().iter().any(|c| c.initialized)
}


pub fn qla(base: u64) -> bool {
    if base == 0 || base == 0xFFFFFFFF {
        return false;
    }
    
    crate::serial_println!("[USB] EHCI controller at {:#x}", base);
    
    
    
    
    let ar = Vr {
        controller_type: UsbControllerType::Rv,
        base_addr: base,
        devices: Vec::new(),
        initialized: false,  
    };
    
    Jm.lock().push(ar);
    
    
    
    
    
    
    
    
    false  
}


pub fn qlh(base: u64) -> bool {
    if base == 0 || base == 0xFFFFFFFF {
        return false;
    }
    
    crate::serial_println!("[USB] xHCI controller at {:#x}", base);
    
    let ar = Vr {
        controller_type: UsbControllerType::Wa,
        base_addr: base,
        devices: Vec::new(),
        initialized: false,  
    };
    
    Jm.lock().push(ar);
    
    
    
    
    
    
    
    
    
    
    
    false  
}


pub fn lqv() -> Vec<Afy> {
    Jm.lock()
        .iter()
        .flat_map(|c| c.devices.clone())
        .collect()
}


pub fn kxn() -> usize {
    Jm.lock().len()
}


pub fn mjm() -> bool {
    Jm.lock()
        .iter()
        .any(|c| c.devices.iter().any(|d| d.class == UsbDeviceClass::HID))
}
