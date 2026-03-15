//! Realtek RTL8139 Network Driver (Stub)
//!
//! Driver for Realtek RTL8139 NICs.
//! Common in older VirtualBox and QEMU configurations.

use alloc::boxed::Box;
use alloc::vec::Vec;

use super::{NetworkDriver, NetStats};
use crate::drivers::{Driver, DriverInfo, DriverStatus, DriverCategory};
use crate::pci::PciDevice;

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

impl Driver for Rtl8139Driver {
    fn info(&self) -> &DriverInfo {
        &DRIVER_INFO
    }
    
    fn probe(&mut self, _pci_device: &PciDevice) -> Result<(), &'static str> {
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

impl NetworkDriver for Rtl8139Driver {
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

const DRIVER_INFO: DriverInfo = DriverInfo {
    name: "rtl8139",
    version: "0.1.0",
    author: "T-RustOs Team",
    category: DriverCategory::Network,
    vendor_ids: &[
        (0x10EC, 0x8139),  // RTL8139
    ],
};

pub fn register() {
    crate::drivers::register(DRIVER_INFO, || {
        Box::new(Rtl8139Driver::new())
    });
    crate::drivers::net::register_net_driver(DRIVER_INFO, || {
        Box::new(Rtl8139Driver::new())
    });
}
