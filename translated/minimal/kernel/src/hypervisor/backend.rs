











use alloc::string::String;
use alloc::vec::Vec;

use super::{HypervisorError, Result};






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    Cu,
    Ai,
    Cl,
    Af,
    Gu,
}


#[derive(Debug, Clone, Default)]
pub struct VmStats {
    pub ait: u64,
    pub bmp: u64,
    pub ank: u64,
    pub bkn: u64,
    pub axz: u64,
    pub omw: u64,  
    pub ocr: u64,
    pub oez: u64,
}


#[derive(Debug, Clone, Copy)]
pub enum GuestMode {
    
    Bqu { mi: u64 },
    
    Bpm { mi: u64, ahu: u64 },
    
    Blg { mi: u64, ahu: u64, gbg: u64 },
    
    Blo { mi: u64, ahu: u64, jm: u64 },
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    Ajf,
    Agh,
}









pub trait Ats {
    
    fn mxn(&self) -> BackendType;
    
    
    fn fk(&self) -> u64;
    
    
    fn jvx(&self) -> &str;
    
    
    fn g(&self) -> VmState;
    
    
    fn cm(&self) -> VmStats;
    
    
    fn apy(&self) -> usize;
    
    
    
    
    fn cfp(&mut self) -> Result<()>;
    
    
    fn diy(&mut self, f: &[u8], dst: u64) -> Result<()>;
    
    
    fn pjn(&mut self, ev: GuestMode) -> Result<()>;
    
    
    fn vw(&mut self) -> Result<()>;
    
    
    fn ima(&mut self, uz: &[u8], wx: &str, apw: Option<&[u8]>) -> Result<()>;
    
    
    fn rb(&mut self) -> Result<()>;
    
    
    fn anu(&mut self) -> Result<()>;
    
    
    
    
    fn duy(&self, pe: u64, len: usize) -> Option<&[u8]>;
    
    
    fn jxg(&mut self, pe: u64, f: &[u8]) -> Result<()>;
}







pub fn dpg(j: &str, afc: usize) -> Result<alloc::boxed::Box<dyn Ats>> {
    match super::avo() {
        super::CpuVendor::Ct => {
            let vm = super::svm_vm::SvmVirtualMachine::new(j, afc)?;
            Ok(alloc::boxed::Box::new(Btp { vm }))
        }
        super::CpuVendor::Ef => {
            let ad = super::AJR_.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
            let vm = super::vm::VirtualMachine::new(ad, j, afc)?;
            Ok(alloc::boxed::Box::new(Bwc { vm }))
        }
        super::CpuVendor::F => {
            Err(HypervisorError::Tr)
        }
    }
}






pub struct Btp {
    pub vm: super::svm_vm::SvmVirtualMachine,
}

impl Ats for Btp {
    fn mxn(&self) -> BackendType { BackendType::Agh }
    
    fn fk(&self) -> u64 { self.vm.ad }
    
    fn jvx(&self) -> &str { &self.vm.j }
    
    fn g(&self) -> VmState {
        match self.vm.drd() {
            super::svm_vm::SvmVmState::Cu => VmState::Cu,
            super::svm_vm::SvmVmState::Ai => VmState::Ai,
            super::svm_vm::SvmVmState::Cl  => VmState::Cl,
            super::svm_vm::SvmVmState::Af => VmState::Af,
            super::svm_vm::SvmVmState::Gu => VmState::Gu,
        }
    }
    
    fn cm(&self) -> VmStats {
        let e = self.vm.asx();
        VmStats {
            ait: e.ait,
            bmp: e.bmp,
            ank: e.ank,
            bkn: e.bkn,
            axz: e.axz,
            omw: e.cay,
            ocr: e.gwh,
            oez: e.jap,
        }
    }
    
    fn apy(&self) -> usize { self.vm.apy }
    
    fn cfp(&mut self) -> Result<()> { self.vm.cfp() }
    
    fn diy(&mut self, f: &[u8], dst: u64) -> Result<()> {
        self.vm.diy(f, dst)
    }
    
    fn pjn(&mut self, ev: GuestMode) -> Result<()> {
        match ev {
            GuestMode::Bqu { mi } => self.vm.jpk(mi),
            GuestMode::Bpm { mi, ahu } => {
                self.vm.iab(mi, ahu)
            }
            GuestMode::Blg { mi, ahu, gbg } => {
                self.vm.pjq(mi, ahu, gbg)
            }
            GuestMode::Blo { mi, ahu, jm } => {
                self.vm.mfb(mi, ahu, jm)
            }
        }
    }
    
    fn vw(&mut self) -> Result<()> { self.vm.ay() }
    
    fn ima(&mut self, uz: &[u8], wx: &str, apw: Option<&[u8]>) -> Result<()> {
        self.vm.fvn(uz, wx, apw)
    }
    
    fn rb(&mut self) -> Result<()> { self.vm.rb() }
    fn anu(&mut self) -> Result<()> { self.vm.anu() }
    
    fn duy(&self, pe: u64, len: usize) -> Option<&[u8]> {
        self.vm.duy(pe, len)
    }
    
    fn jxg(&mut self, pe: u64, f: &[u8]) -> Result<()> {
        self.vm.jxg(pe, f)
    }
}






pub struct Bwc {
    pub vm: super::vm::VirtualMachine,
}

impl Ats for Bwc {
    fn mxn(&self) -> BackendType { BackendType::Ajf }
    
    fn fk(&self) -> u64 { self.vm.ad }
    
    fn jvx(&self) -> &str { &self.vm.j }
    
    fn g(&self) -> VmState {
        match self.vm.g {
            super::vm::VmState::Cu => VmState::Cu,
            super::vm::VmState::Ai => VmState::Ai,
            super::vm::VmState::Cl  => VmState::Cl,
            super::vm::VmState::Af => VmState::Af,
            super::vm::VmState::Gu => VmState::Gu,
        }
    }
    
    fn cm(&self) -> VmStats {
        VmStats {
            ait: self.vm.cm.gwg,
            bmp: self.vm.cm.bmp,
            ank: self.vm.cm.ank,
            bkn: self.vm.cm.bkn,
            axz: self.vm.cm.axz,
            omw: self.vm.cm.fhx,
            ocr: 0,
            oez: 0,
        }
    }
    
    fn apy(&self) -> usize { self.vm.apy }
    
    fn cfp(&mut self) -> Result<()> { self.vm.cfp() }
    
    fn diy(&mut self, f: &[u8], dst: u64) -> Result<()> {
        self.vm.diy(f, dst)
    }
    
    fn pjn(&mut self, ev: GuestMode) -> Result<()> {
        match ev {
            GuestMode::Bpm { mi, ahu } |
            GuestMode::Blg { mi, ahu, .. } |
            GuestMode::Blo { mi, ahu, .. } => {
                
                
                self.vm.ay(mi, ahu)?;
                Ok(())
            }
            GuestMode::Bqu { mi } => {
                self.vm.ay(mi, 0x8000)?;
                Ok(())
            }
        }
    }
    
    fn vw(&mut self) -> Result<()> {
        
        
        Ok(())
    }
    
    fn ima(&mut self, uz: &[u8], wx: &str, apw: Option<&[u8]>) -> Result<()> {
        self.vm.fvn(uz, wx, apw)
    }
    
    fn rb(&mut self) -> Result<()> {
        self.vm.g = super::vm::VmState::Cl;
        Ok(())
    }
    
    fn anu(&mut self) -> Result<()> {
        self.vm.g = super::vm::VmState::Ai;
        Ok(())
    }
    
    fn duy(&self, pe: u64, len: usize) -> Option<&[u8]> {
        let l = pe as usize;
        if l + len <= self.vm.apy {
            
            None 
        } else {
            None
        }
    }
    
    fn jxg(&mut self, pe: u64, f: &[u8]) -> Result<()> {
        self.vm.diy(f, pe)
    }
}
