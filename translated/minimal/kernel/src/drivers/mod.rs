




pub mod net;
pub mod ahci;
pub mod ata;
pub mod usb;
pub mod usb_storage;
pub mod xhci;
pub mod checkm8;
pub mod input;
pub mod pci_ids;
pub mod partition;
pub mod virtio_gpu;
pub mod hda;
pub mod amdgpu;
pub mod nvidia;
pub mod thinkpad_ec;



#[cfg(target_arch = "aarch64")]
pub mod apple;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverCategory {
    Network,
    Storage,
    Display,
    Input,
    Audio,
    Qs,
    Other,
}


#[derive(Debug, Clone)]
pub struct Bb {
    pub name: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub category: DriverCategory,
    pub vendor_ids: &'static [(u16, u16)],  
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverStatus {
    Unloaded,
    Loading,
    Running,
    Error,
    Suspended,
}


pub trait Cw: Send + Sync {
    
    fn info(&self) -> &Bb;
    
    
    fn probe(&mut self, pci_device: &crate::pci::L) -> Result<(), &'static str>;
    
    
    fn start(&mut self) -> Result<(), &'static str>;
    
    
    fn stop(&mut self) -> Result<(), &'static str>;
    
    
    fn status(&self) -> DriverStatus;
    
    
    fn btc(&mut self) {}
}


struct Adk {
    info: Bb,
    factory: fn() -> Box<dyn Cw>,
}


static Ca: Mutex<Vec<Adk>> = Mutex::new(Vec::new());


static Aao: Mutex<Vec<Box<dyn Cw>>> = Mutex::new(Vec::new());


pub fn register(info: Bb, factory: fn() -> Box<dyn Cw>) {
    let mut ary = Ca.lock();
    crate::log_debug!("[DRIVERS] Registered: {} v{}", info.name, info.version);
    ary.push(Adk { info, factory });
}


pub fn goi(go: &crate::pci::L) -> Option<usize> {
    let ary = Ca.lock();
    
    for entry in ary.iter() {
        
        if entry.info.category == DriverCategory::Network {
            continue;
        }
        for &(vendor, device) in entry.info.vendor_ids {
            
            if go.vendor_id == vendor && 
               (device == 0xFFFF || go.device_id == device) {
                
                
                let mut driver = (entry.factory)();
                
                
                match driver.probe(go) {
                    Ok(()) => {
                        crate::log!("[DRIVERS] Loaded {} for {:04X}:{:04X}",
                            entry.info.name, go.vendor_id, go.device_id);
                        
                        
                        if let Err(e) = driver.start() {
                            crate::log_warn!("[DRIVERS] Failed to start {}: {}", entry.info.name, e);
                            return None;
                        }
                        
                        let mut bhq = Aao.lock();
                        let idx = bhq.len();
                        bhq.push(driver);
                        return Some(idx);
                    }
                    Err(e) => {
                        crate::log_debug!("[DRIVERS] {} probe failed: {}", entry.info.name, e);
                    }
                }
            }
        }
    }
    
    None
}


pub fn gga() -> usize {
    Aao.lock().len()
}


pub fn qnp() -> Vec<Bb> {
    Ca.lock().iter().map(|e| e.info.clone()).collect()
}


pub fn init() {
    crate::log!("[DRIVERS] Initializing driver framework...");
    
    
    net::oei();
    
    
    input::init();
    
    let count = Ca.lock().len();
    crate::log!("[DRIVERS] {} drivers registered", count);
}


pub fn gom() {
    crate::serial_println!("[DRIVERS] Probing storage controllers...");
    
    let devices = crate::pci::aqs();
    
    for s in &devices {
        
        if s.class_code == 0x01 && s.subclass == 0x06 && s.prog_if == 0x01 {
            crate::serial_println!("[AHCI] Controller detected at {:02X}:{:02X}.{} (BAR5={:#x})",
                s.bus, s.device, s.function, s.bar[5]);
            let dih = s.bar[5] as u64;
            if ahci::init(dih) {
                crate::serial_println!("[DRIVERS] AHCI controller initialized");
                
                ahci::mnl();
            }
        }
        
        
        if s.class_code == 0x01 && s.subclass == 0x01 {
            
            if ata::igt() {
                crate::serial_println!("[DRIVERS] IDE controller initialized");
            }
        }
        
        
        if s.class_code == 0x01 && s.subclass == 0x08 {
            crate::serial_println!("[DRIVERS] NVMe controller at {:02X}:{:02X}.{} ({:04X}:{:04X})",
                s.bus, s.device, s.function, s.vendor_id, s.device_id);
            
        }
        
        
        if s.class_code == 0x0C && s.subclass == 0x03 {
            let pqi = match s.prog_if {
                0x00 => "UHCI (USB 1.0)",
                0x10 => "OHCI (USB 1.1)",
                0x20 => "EHCI (USB 2.0)",
                0x30 => "xHCI (USB 3.0)",
                _ => "Unknown USB",
            };
            crate::serial_println!("[USB] Controller detected: {} at {:02X}:{:02X}.{} (BAR0={:#x})", 
                pqi, s.bus, s.device, s.function, s.bar[0]);
            
            
            if s.prog_if == 0x30 {
                
                let bar0 = s.bar_address(0).unwrap_or(s.bar[0] as u64);
                crate::pci::bzi(s);
                crate::pci::bzj(s);
                if xhci::init(bar0) {
                    crate::serial_println!("[DRIVERS] xHCI controller initialized with {} devices", 
                        xhci::aqg());
                }
            }
        }
    }
    
    
    if !ahci::is_initialized() && !ata::is_initialized() {
        crate::serial_println!("[DRIVERS] Trying legacy IDE ports...");
        let _ = ata::igt();
    }
    
    
    if crate::nvme::is_initialized() {
        if let Some((model, _serial, size, aol)) = crate::nvme::rk() {
            let aop = (size * aol as u64) / (1024 * 1024);
            crate::serial_println!("[DRIVERS] Storage: NVMe {} ({} MB)", model, aop);
        }
    } else if ahci::is_initialized() {
        crate::serial_println!("[DRIVERS] Storage: AHCI with {} ports", ahci::ibt());
    } else if ata::is_initialized() {
        crate::serial_println!("[DRIVERS] Storage: IDE with {} drives", ata::eta().len());
    } else {
        crate::serial_println!("[DRIVERS] Storage: No persistent storage (using RAM disk)");
    }
}


pub fn ied() -> bool {
    ahci::is_initialized() || ata::is_initialized() || crate::nvme::is_initialized()
}


pub fn pyh() {
    let devices = crate::pci::scan();
    let mut bhq = 0;
    
    for s in &devices {
        if goi(s).is_some() {
            bhq += 1;
        }
    }
    
    crate::log!("[DRIVERS] Auto-probe complete: {} drivers loaded", bhq);
}
