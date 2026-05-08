




use alloc::boxed::Box;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};

use super::{Cw, Bb, DriverStatus, DriverCategory, register};
use crate::pci::L;


pub trait Dd: Cw {
    
    fn mac_address(&self) -> [u8; 6];
    
    
    fn link_up(&self) -> bool;
    
    
    fn cbj(&self) -> u32 { 0 }
    
    
    fn send(&mut self, data: &[u8]) -> Result<(), &'static str>;
    
    
    fn receive(&mut self) -> Option<Vec<u8>>;
    
    
    fn poll(&mut self);
    
    
    fn stats(&self) -> NetStats;
    
    
    fn jfi(&mut self, _enabled: bool) -> Result<(), &'static str> {
        Err("Not supported")
    }
    
    
    fn pxt(&mut self, _mac: [u8; 6]) -> Result<(), &'static str> {
        Err("Not supported")
    }
}


#[derive(Debug, Clone, Copy, Default)]
pub struct NetStats {
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub tx_errors: u64,
    pub rx_errors: u64,
    pub tx_dropped: u64,
    pub rx_dropped: u64,
}


static IB_: Mutex<Option<Box<dyn Dd>>> = Mutex::new(None);
static ASS_: AtomicBool = AtomicBool::new(false);


struct Adl {
    info: Bb,
    factory: fn() -> Box<dyn Dd>,
}


static BDL_: Mutex<Vec<Adl>> = Mutex::new(Vec::new());





mod virtio;





mod e1000;





mod rtl8139;





mod rtl8169;





pub mod wifi;
pub mod iwl4965;






pub fn oei() {
    virtio::register();
    e1000::register();
    rtl8139::register();
    rtl8169::register();
}


pub fn eyh(info: Bb, factory: fn() -> Box<dyn Dd>) {
    let mut ary = BDL_.lock();
    ary.push(Adl { info, factory });
}


pub fn goi(go: &L) -> bool {
    let ary = BDL_.lock();
    for entry in ary.iter() {
        for &(vendor, device) in entry.info.vendor_ids {
            if go.vendor_id == vendor && (device == 0xFFFF || go.device_id == device) {
                let mut driver = (entry.factory)();
                match driver.probe(go) {
                    Ok(()) => {
                        if let Err(e) = driver.start() {
                            crate::log_warn!("[DRIVERS] Failed to start {}: {}", entry.info.name, e);
                            return false;
                        }
                        *IB_.lock() = Some(driver);
                        ASS_.store(true, Ordering::SeqCst);
                        return true;
                    }
                    Err(e) => {
                        crate::log_debug!("[DRIVERS] {} probe failed: {}", entry.info.name, e);
                    }
                }
            }
        }
    }
    false
}


pub fn aoh() -> bool {
    ASS_.load(Ordering::Relaxed)
}


pub fn aqt() -> Option<[u8; 6]> {
    IB_.lock().as_ref().map(|d| d.mac_address())
}


pub fn link_up() -> bool {
    IB_.lock().as_ref().map(|d| d.link_up()).unwrap_or(false)
}


pub fn send(data: &[u8]) -> Result<(), &'static str> {
    let mut jg = IB_.lock();
    let driver = jg.as_mut().ok_or("No network driver")?;
    driver.send(data)
}


pub fn receive() -> Option<Vec<u8>> {
    let mut jg = IB_.lock();
    jg.as_mut().and_then(|d| d.receive())
}


pub fn poll() {
    if let Some(driver) = IB_.lock().as_mut() {
        driver.poll();
    }
}


pub fn stats() -> NetStats {
    IB_.lock().as_ref()
        .map(|d| d.stats())
        .unwrap_or_default()
}
