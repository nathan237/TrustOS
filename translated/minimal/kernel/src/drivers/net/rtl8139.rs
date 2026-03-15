




use alloc::boxed::Box;
use alloc::vec::Vec;

use super::{Ha, NetStats};
use crate::drivers::{Gi, Co, DriverStatus, DriverCategory};
use crate::pci::S;

pub struct Rtl8139Driver {
    status: DriverStatus,
    ed: [u8; 6],
}

impl Rtl8139Driver {
    pub fn new() -> Self {
        Self {
            status: DriverStatus::Aff,
            ed: [0x52, 0x54, 0x00, 0x81, 0x39, 0x00],
        }
    }
}

impl Gi for Rtl8139Driver {
    fn co(&self) -> &Co {
        &BZ_
    }
    
    fn probe(&mut self, ybv: &S) -> Result<(), &'static str> {
        self.status = DriverStatus::Py;
        crate::log!("[rtl8139] Driver probe - not yet implemented");
        Err("RTL8139 driver not implemented yet")
    }
    
    fn ay(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Ai;
        Ok(())
    }
    
    fn qg(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Ky;
        Ok(())
    }
    
    fn status(&self) -> DriverStatus {
        self.status
    }
}

impl Ha for Rtl8139Driver {
    fn csg(&self) -> [u8; 6] {
        self.ed
    }
    
    fn aik(&self) -> bool {
        false
    }
    
    fn baq(&mut self, iia: &[u8]) -> Result<(), &'static str> {
        Err("Not implemented")
    }
    
    fn chb(&mut self) -> Option<Vec<u8>> {
        None
    }
    
    fn poll(&mut self) {}
    
    fn cm(&self) -> NetStats {
        NetStats::default()
    }
}

const BZ_: Co = Co {
    j: "rtl8139",
    dk: "0.1.0",
    gzh: "T-RustOs Team",
    gb: DriverCategory::As,
    fye: &[
        (0x10EC, 0x8139),  
    ],
};

pub fn nw() {
    crate::drivers::nw(BZ_, || {
        Box::new(Rtl8139Driver::new())
    });
    crate::drivers::net::jly(BZ_, || {
        Box::new(Rtl8139Driver::new())
    });
}
