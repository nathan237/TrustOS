




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
    As,
    Og,
    Display,
    Jp,
    Rj,
    Any,
    Qg,
}


#[derive(Debug, Clone)]
pub struct Co {
    pub j: &'static str,
    pub dk: &'static str,
    pub gzh: &'static str,
    pub gb: DriverCategory,
    pub fye: &'static [(u16, u16)],  
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverStatus {
    Aff,
    Py,
    Ai,
    Q,
    Ky,
}


pub trait Gi: Send + Sync {
    
    fn co(&self) -> &Co;
    
    
    fn probe(&mut self, cgm: &crate::pci::S) -> Result<(), &'static str>;
    
    
    fn ay(&mut self) -> Result<(), &'static str>;
    
    
    fn qg(&mut self) -> Result<(), &'static str>;
    
    
    fn status(&self) -> DriverStatus;
    
    
    fn eck(&mut self) {}
}


struct Bqv {
    co: Co,
    fig: fn() -> Box<dyn Gi>,
}


static Ev: Mutex<Vec<Bqv>> = Mutex::new(Vec::new());


static Bkl: Mutex<Vec<Box<dyn Gi>>> = Mutex::new(Vec::new());


pub fn nw(co: Co, fig: fn() -> Box<dyn Gi>) {
    let mut chc = Ev.lock();
    crate::log_debug!("[DRIVERS] Registered: {} v{}", co.j, co.dk);
    chc.push(Bqv { co, fig });
}


pub fn lvo(sq: &crate::pci::S) -> Option<usize> {
    let chc = Ev.lock();
    
    for bt in chc.iter() {
        
        if bt.co.gb == DriverCategory::As {
            continue;
        }
        for &(acs, de) in bt.co.fye {
            
            if sq.ml == acs && 
               (de == 0xFFFF || sq.mx == de) {
                
                
                let mut rj = (bt.fig)();
                
                
                match rj.probe(sq) {
                    Ok(()) => {
                        crate::log!("[DRIVERS] Loaded {} for {:04X}:{:04X}",
                            bt.co.j, sq.ml, sq.mx);
                        
                        
                        if let Err(aa) = rj.ay() {
                            crate::log_warn!("[DRIVERS] Failed to start {}: {}", bt.co.j, aa);
                            return None;
                        }
                        
                        let mut diz = Bkl.lock();
                        let w = diz.len();
                        diz.push(rj);
                        return Some(w);
                    }
                    Err(aa) => {
                        crate::log_debug!("[DRIVERS] {} probe failed: {}", bt.co.j, aa);
                    }
                }
            }
        }
    }
    
    None
}


pub fn ljl() -> usize {
    Bkl.lock().len()
}


pub fn zba() -> Vec<Co> {
    Ev.lock().iter().map(|aa| aa.co.clone()).collect()
}


pub fn init() {
    crate::log!("[DRIVERS] Initializing driver framework...");
    
    
    net::vuc();
    
    
    input::init();
    
    let az = Ev.lock().len();
    crate::log!("[DRIVERS] {} drivers registered", az);
}


pub fn lvr() {
    crate::serial_println!("[DRIVERS] Probing storage controllers...");
    
    let ik = crate::pci::fjm();
    
    for ba in &ik {
        
        if ba.ajz == 0x01 && ba.adl == 0x06 && ba.frg == 0x01 {
            crate::serial_println!("[AHCI] Controller detected at {:02X}:{:02X}.{} (BAR5={:#x})",
                ba.aq, ba.de, ba.gw, ba.bar[5]);
            let gzp = ba.bar[5] as u64;
            if ahci::init(gzp) {
                crate::serial_println!("[DRIVERS] AHCI controller initialized");
                
                ahci::tri();
            }
        }
        
        
        if ba.ajz == 0x01 && ba.adl == 0x01 {
            
            if ata::oem() {
                crate::serial_println!("[DRIVERS] IDE controller initialized");
            }
        }
        
        
        if ba.ajz == 0x01 && ba.adl == 0x08 {
            crate::serial_println!("[DRIVERS] NVMe controller at {:02X}:{:02X}.{} ({:04X}:{:04X})",
                ba.aq, ba.de, ba.gw, ba.ml, ba.mx);
            
        }
        
        
        if ba.ajz == 0x0C && ba.adl == 0x03 {
            let xpi = match ba.frg {
                0x00 => "UHCI (USB 1.0)",
                0x10 => "OHCI (USB 1.1)",
                0x20 => "EHCI (USB 2.0)",
                0x30 => "xHCI (USB 3.0)",
                _ => "Unknown USB",
            };
            crate::serial_println!("[USB] Controller detected: {} at {:02X}:{:02X}.{} (BAR0={:#x})", 
                xpi, ba.aq, ba.de, ba.gw, ba.bar[0]);
            
            
            if ba.frg == 0x30 {
                
                let aew = ba.cje(0).unwrap_or(ba.bar[0] as u64);
                crate::pci::fhp(ba);
                crate::pci::fhq(ba);
                if xhci::init(aew) {
                    crate::serial_println!("[DRIVERS] xHCI controller initialized with {} devices", 
                        xhci::cjx());
                }
            }
        }
    }
    
    
    if !ahci::ky() && !ata::ky() {
        crate::serial_println!("[DRIVERS] Trying legacy IDE ports...");
        let _ = ata::oem();
    }
    
    
    if crate::nvme::ky() {
        if let Some((model, msv, aw, cak)) = crate::nvme::ani() {
            let csm = (aw * cak as u64) / (1024 * 1024);
            crate::serial_println!("[DRIVERS] Storage: NVMe {} ({} MB)", model, csm);
        }
    } else if ahci::ky() {
        crate::serial_println!("[DRIVERS] Storage: AHCI with {} ports", ahci::nyj());
    } else if ata::ky() {
        crate::serial_println!("[DRIVERS] Storage: IDE with {} drives", ata::jdq().len());
    } else {
        crate::serial_println!("[DRIVERS] Storage: No persistent storage (using RAM disk)");
    }
}


pub fn oba() -> bool {
    ahci::ky() || ata::ky() || crate::nvme::ky()
}


pub fn yff() {
    let ik = crate::pci::arx();
    let mut diz = 0;
    
    for ba in &ik {
        if lvo(ba).is_some() {
            diz += 1;
        }
    }
    
    crate::log!("[DRIVERS] Auto-probe complete: {} drivers loaded", diz);
}
