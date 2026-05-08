//! Realtek RTL8139 Network Driver (Stub)
//!
//! Driver for Realtek RTL8139 NICs.
//! Common in older VirtualBox and QEMU configurations.

use alloc::boxed::Box;
use alloc::vec::Vec;

use super::{NetworkDriver, NetStats};
use crate::drivers::{Driver, DriverInformation, DriverStatus, DriverCategory};
use crate::pci::PciDevice;

// Structure publique — visible à l'extérieur de ce module.
pub struct Rtl8139Driver {
    status: DriverStatus,
    mac: [u8; 6],
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Rtl8139Driver {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new() -> Self {
        Self {
            status: DriverStatus::Unloaded,
            mac: [0x52, 0x54, 0x00, 0x81, 0x39, 0x00],
        }
    }
}

// Implémentation de trait — remplit un contrat comportemental.
impl Driver for Rtl8139Driver {
    fn info(&self) -> &DriverInformation {
        &DRIVER_INFORMATION
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

// Implémentation de trait — remplit un contrat comportemental.
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

// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DRIVER_INFORMATION: DriverInformation = DriverInformation {
    name: "rtl8139",
    version: "0.1.0",
    author: "T-RustOs Team",
    category: DriverCategory::Network,
    vendor_ids: &[
        (0x10EC, 0x8139),  // RTL8139
    ],
};

// Fonction publique — appelable depuis d'autres modules.
pub fn register() {
    crate::drivers::register(DRIVER_INFORMATION, || {
        Box::new(Rtl8139Driver::new())
    });
    crate::drivers::net::register_net_driver(DRIVER_INFORMATION, || {
        Box::new(Rtl8139Driver::new())
    });
}
