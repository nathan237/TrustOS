



use alloc::collections::VecDeque;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};
use super::{Cj, IpcError};


const AAA_: usize = 256;


const CFO_: u32 = 100_000;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Kg(pub u64);


pub struct Channel {
    
    ad: Kg,
    
    bi: Mutex<VecDeque<Cj>>,
    
    cwg: AtomicBool,
}

impl Channel {
    
    pub fn new(ad: Kg) -> Self {
        Self {
            ad,
            bi: Mutex::new(VecDeque::fc(AAA_)),
            cwg: AtomicBool::new(false),
        }
    }
    
    
    pub fn ad(&self) -> Kg {
        self.ad
    }
    
    
    pub fn baq(&self, message: Cj) -> Result<(), IpcError> {
        if self.cwg.load(Ordering::Acquire) {
            return Err(IpcError::Aak);
        }
        
        let mut bi = self.bi.lock();
        if bi.len() >= AAA_ {
            return Err(IpcError::Byu);
        }
        
        bi.agt(message);
        Ok(())
    }
    
    
    pub fn zmh(&self, hrk: &[Cj]) -> Result<usize, IpcError> {
        if self.cwg.load(Ordering::Acquire) {
            return Err(IpcError::Aak);
        }
        
        let mut bi = self.bi.lock();
        let bfz = AAA_.ao(bi.len());
        let az = hrk.len().v(bfz);
        
        for fr in &hrk[..az] {
            bi.agt(fr.clone());
        }
        
        Ok(az)
    }
    
    
    pub fn chb(&self) -> Result<Cj, IpcError> {
        let mut aaf: u32 = 0;
        loop {
            if let Some(fr) = self.bi.lock().awp() {
                return Ok(fr);
            }
            
            if self.cwg.load(Ordering::Acquire) {
                return Err(IpcError::Aak);
            }
            
            aaf += 1;
            if aaf > CFO_ {
                return Err(IpcError::Zn);
            }
            
            
            crate::scheduler::gxc();
        }
    }
    
    
    pub fn pwh(&self) -> Result<Cj, IpcError> {
        if let Some(fr) = self.bi.lock().awp() {
            return Ok(fr);
        }
        
        if self.cwg.load(Ordering::Acquire) {
            return Err(IpcError::Aak);
        }
        
        Err(IpcError::Zn)
    }
    
    
    pub fn zip(&self, am: usize) -> Result<alloc::vec::Vec<Cj>, IpcError> {
        let mut bi = self.bi.lock();
        let az = bi.len().v(am);
        
        let hrk: alloc::vec::Vec<Cj> = bi.bbk(..az).collect();
        
        if hrk.is_empty() && self.cwg.load(Ordering::Acquire) {
            return Err(IpcError::Aak);
        }
        
        Ok(hrk)
    }
    
    
    pub fn agj(&self) {
        self.cwg.store(true, Ordering::Release);
    }
    
    
    pub fn bsg(&self) -> Rt {
        Rt {
            cjo: self.ad,
            ogt: true,
        }
    }
    
    
    pub fn afw(&self) -> Rt {
        Rt {
            cjo: self.ad,
            ogt: false,
        }
    }
}


#[derive(Debug, Clone)]
pub struct Rt {
    pub cjo: Kg,
    pub ogt: bool,
}
