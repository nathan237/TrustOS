




use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{Ha, NetStats};
use crate::drivers::{Gi, Co, DriverStatus, DriverCategory};
use crate::pci::S;


use crate::virtio_net::VirtioNet as VirtioNetImpl;


pub struct VirtioNetDriver {
    ff: Option<VirtioNetImpl>,
    status: DriverStatus,
    ifj: AtomicU64,
    hyk: AtomicU64,
    bpc: AtomicU64,
    bsc: AtomicU64,
}

impl VirtioNetDriver {
    pub fn new() -> Self {
        Self {
            ff: None,
            status: DriverStatus::Aff,
            ifj: AtomicU64::new(0),
            hyk: AtomicU64::new(0),
            bpc: AtomicU64::new(0),
            bsc: AtomicU64::new(0),
        }
    }
}

impl Gi for VirtioNetDriver {
    fn co(&self) -> &Co {
        &BZ_
    }
    
    fn probe(&mut self, cgm: &S) -> Result<(), &'static str> {
        self.status = DriverStatus::Py;
        
        
        let mut rj = VirtioNetImpl::new(cgm)?;
        rj.pjr()?;
        rj.pjs()?;
        
        
        let irq = cgm.esw;
        if irq > 0 && irq < 255 {
            crate::apic::jmw(irq, crate::apic::HH_);
            crate::serial_println!("[virtio-net-drv] IRQ {} routed to vector {}", irq, crate::apic::HH_);
        }
        
        
        crate::virtio_net::wjc(rj.agq());
        
        self.ff = Some(rj);
        Ok(())
    }
    
    fn ay(&mut self) -> Result<(), &'static str> {
        let rj = self.ff.as_mut().ok_or("Driver not probed")?;
        rj.ay()?;
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

impl Ha for VirtioNetDriver {
    fn csg(&self) -> [u8; 6] {
        self.ff.as_ref()
            .map(|bc| bc.ed())
            .unwrap_or([0; 6])
    }
    
    fn aik(&self) -> bool {
        self.ff.as_ref()
            .map(|bc| bc.txy())
            .unwrap_or(false)
    }
    
    fn baq(&mut self, f: &[u8]) -> Result<(), &'static str> {
        let rj = self.ff.as_mut().ok_or("Driver not initialized")?;
        rj.baq(f)?;
        self.ifj.fetch_add(1, Ordering::Relaxed);
        self.bpc.fetch_add(f.len() as u64, Ordering::Relaxed);
        Ok(())
    }
    
    fn chb(&mut self) -> Option<Vec<u8>> {
        let rj = self.ff.as_mut()?;
        if let Some(ex) = rj.owk() {
            self.hyk.fetch_add(1, Ordering::Relaxed);
            self.bsc.fetch_add(ex.len() as u64, Ordering::Relaxed);
            Some(ex)
        } else {
            None
        }
    }
    
    fn poll(&mut self) {
        if let Some(rj) = self.ff.as_mut() {
            rj.owm();
            
        }
    }
    
    fn cm(&self) -> NetStats {
        NetStats {
            cuz: self.ifj.load(Ordering::Relaxed),
            dbo: self.hyk.load(Ordering::Relaxed),
            bpc: self.bpc.load(Ordering::Relaxed),
            bsc: self.bsc.load(Ordering::Relaxed),
            dmv: 0,
            dbn: 0,
            mnn: 0,
            mbk: 0,
        }
    }
}


const BZ_: Co = Co {
    j: "virtio-net",
    dk: "1.0.0",
    gzh: "T-RustOs Team",
    gb: DriverCategory::As,
    fye: &[
        (0x1AF4, 0x1000),  
        (0x1AF4, 0x1041),  
    ],
};


pub fn nw() {
    crate::drivers::nw(BZ_, || {
        Box::new(VirtioNetDriver::new())
    });
    crate::drivers::net::jly(BZ_, || {
        Box::new(VirtioNetDriver::new())
    });
}
