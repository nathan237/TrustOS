




use alloc::boxed::Box;
use alloc::vec::Vec;

use super::{Dd, NetStats};
use crate::drivers::{Cw, Bb, DriverStatus, DriverCategory};
use crate::pci::L;

pub struct Rtl8139Driver {
    status: DriverStatus,
    mac: [u8; 6],
}

impl Rtl8139Driver {
    pub fn new() -> Self {
        Self {
            status: DriverStatus::Unloaded,
            mac: [0x52, 0x54, 0x00, 0x81, 0x39, 0x00],
        }
    }
}

impl Cw for Rtl8139Driver {
    fn info(&self) -> &Bb {
        &CA_
    }
    
    fn probe(&mut self, _pci_device: &L) -> Result<(), &'static str> {
        self.status = DriverStatus::Loading;
        crate::log!("[rtl8139] Driver probe - not yet implemented");
        Err("RTL8139 driver not implemented yet")
    }
    
    fn start(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Running;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Suspended;
        Ok(())
    }
    
    fn status(&self) -> DriverStatus {
        self.status
    }
}

impl Dd for Rtl8139Driver {
    fn mac_address(&self) -> [u8; 6] {
        self.mac
    }
    
    fn link_up(&self) -> bool {
        false
    }
    
    fn send(&mut self, _data: &[u8]) -> Result<(), &'static str> {
        Err("Not implemented")
    }
    
    fn receive(&mut self) -> Option<Vec<u8>> {
        None
    }
    
    fn poll(&mut self) {}
    
    fn stats(&self) -> NetStats {
        NetStats::default()
    }
}

const CA_: Bb = Bb {
    name: "rtl8139",
    version: "0.1.0",
    author: "T-RustOs Team",
    category: DriverCategory::Network,
    vendor_ids: &[
        (0x10EC, 0x8139),  
    ],
};

pub fn register() {
    crate::drivers::register(CA_, || {
        Box::new(Rtl8139Driver::new())
    });
    crate::drivers::net::eyh(CA_, || {
        Box::new(Rtl8139Driver::new())
    });
}
