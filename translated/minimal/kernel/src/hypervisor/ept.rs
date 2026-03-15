







use alloc::boxed::Box;
use alloc::vec::Vec;
use super::{HypervisorError, Result};
use super::vmx::dmy;






#[derive(Clone, Copy)]
pub struct EptPointer(pub u64);

impl EptPointer {
    pub fn new(cgp: u64) -> Self {
        
        let snc = (cgp & 0xFFFF_FFFF_FFFF_F000) | (3 << 3) | 6;
        EptPointer(snc)
    }
    
    pub fn cvr(&self) -> u64 {
        self.0
    }
}


pub mod flags {
    pub const Cm: u64 = 1 << 0;
    pub const Db: u64 = 1 << 1;
    pub const Mz: u64 = 1 << 2;
    pub const DTV_: u64 = 0x38; 
    pub const BAH_: u64 = 6 << 3;
    pub const DTW_: u64 = 0 << 3;
    pub const DQV_: u64 = 1 << 6;
    pub const AYP_: u64 = 1 << 7;
    pub const Bbh: u64 = 1 << 8;
    pub const Beb: u64 = 1 << 9;
    pub const DLI_: u64 = 1 << 10;
}


#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct EptEntry(u64);

impl EptEntry {
    pub const fn azs() -> Self {
        EptEntry(0)
    }
    
    pub fn css(cig: u64) -> Self {
        EptEntry(cig | flags::Cm | flags::Db | flags::Mz)
    }
    
    pub fn lnz(dai: u64, wbr: bool) -> Self {
        let mut bt = dai | flags::BAH_;
        if wbr {
            bt |= flags::Cm | flags::Db | flags::Mz;
        }
        EptEntry(bt)
    }
    
    pub fn lnw(dai: u64) -> Self {
        EptEntry(dai | flags::Cm | flags::Db | flags::Mz 
                 | flags::AYP_ | flags::BAH_)
    }
    
    pub fn xo(&self) -> bool {
        (self.0 & (flags::Cm | flags::Db | flags::Mz)) != 0
    }
    
    pub fn yzm(&self) -> bool {
        (self.0 & flags::AYP_) != 0
    }
    
    pub fn ki(&self) -> u64 {
        self.0 & 0xFFFF_FFFF_FFFF_F000
    }
}


#[repr(C, align(4096))]
pub struct EptTable {
    ch: [EptEntry; 512],
}

impl EptTable {
    pub fn new() -> Self {
        EptTable {
            ch: [EptEntry::azs(); 512],
        }
    }
    
    pub fn bt(&self, index: usize) -> &EptEntry {
        &self.ch[index]
    }
    
    pub fn epx(&mut self, index: usize) -> &mut EptEntry {
        &mut self.ch[index]
    }
}


pub struct EptManager {
    
    wc: Box<EptTable>,
    
    huv: Vec<Box<EptTable>>,
    
    hux: Vec<Box<EptTable>>,
    
    frp: Vec<Box<EptTable>>,
    
    dht: usize,
}

impl EptManager {
    
    pub fn new(dht: usize) -> Result<Self> {
        let mut okx = EptManager {
            wc: Box::new(EptTable::new()),
            huv: Vec::new(),
            hux: Vec::new(),
            frp: Vec::new(),
            dht,
        };
        
        
        
        okx.pjp(dht)?;
        
        Ok(okx)
    }
    
    
    pub fn sna(&self) -> EptPointer {
        let dum = self.wc.as_ref() as *const EptTable as u64;
        let cgp = dmy(dum);
        crate::serial_println!("[EPT] PML4 virt=0x{:016X} phys=0x{:016X}", dum, cgp);
        EptPointer::new(cgp)
    }
    
    
    fn pjp(&mut self, aw: usize) -> Result<()> {
        crate::serial_println!("[EPT] Setting up identity mapping for {} MB", aw / (1024 * 1024));
        
        
        let fqc = (aw + 0x1FFFFF) / 0x200000;
        let huw = ((fqc + 511) / 512).am(1);
        
        for ru in 0..huw {
            let mut sr = Box::new(EptTable::new());
            
            
            let eiz = ru * 512;
            for rn in 0..512 {
                let hub = eiz + rn;
                if hub >= fqc {
                    break;
                }
                
                let ki = (hub * 0x200000) as u64;
                sr.ch[rn] = EptEntry::lnw(ki);
            }
            
            let dam = sr.as_ref() as *const EptTable as u64;
            let ayi = dmy(dam);
            self.hux.push(sr);
            
            
            let mut ss = Box::new(EptTable::new());
            ss.ch[0] = EptEntry::css(ayi);
            
            let dan = ss.as_ref() as *const EptTable as u64;
            let auu = dmy(dan);
            self.huv.push(ss);
            
            
            self.wc.ch[ru] = EptEntry::css(auu);
        }
        
        crate::serial_println!("[EPT] Identity mapping configured: {} 2MB pages, {} PDPT(s)", 
                              fqc, huw);
        
        Ok(())
    }
    
    
    pub fn bnl(&mut self, axy: u64, die: u64, ddp: u64) -> Result<()> {
        let wd = ((axy >> 39) & 0x1FF) as usize;
        let ru = ((axy >> 30) & 0x1FF) as usize;
        let rn = ((axy >> 21) & 0x1FF) as usize;
        
        
        if !self.wc.ch[wd].xo() {
            let ss = Box::new(EptTable::new());
            let dan = ss.as_ref() as *const EptTable as u64;
            let auu = dmy(dan);
            self.wc.ch[wd] = EptEntry::css(auu);
            self.huv.push(ss);
        }
        
        
        let vgj = self.wc.ch[wd].ki();
        let vgk = crate::memory::auv(vgj);
        let lth = unsafe { &mut *(vgk as *mut EptTable) };
        
        if !lth.ch[ru].xo() {
            let sr = Box::new(EptTable::new());
            let dam = sr.as_ref() as *const EptTable as u64;
            let ayi = dmy(dam);
            lth.ch[ru] = EptEntry::css(ayi);
            self.hux.push(sr);
        }
        
        let vgf = lth.ch[ru].ki();
        let vgg = crate::memory::auv(vgf);
        let vge = unsafe { &mut *(vgg as *mut EptTable) };
        
        
        let qgf = die & !0x1FFFFF;
        vge.ch[rn] = EptEntry::lnw(qgf);
        
        Ok(())
    }

    
    
    
    
    pub fn wky(&mut self, fe: &[u8]) -> Result<()> {
        let tib = fe.fq() as u64;
        let ixk = dmy(tib);
        let aw = fe.len();
        
        crate::serial_println!("[EPT] Mapping GPA 0x0 -> HPA 0x{:X} ({} MB)",
                              ixk, aw / (1024 * 1024));
        
        
        self.wc = Box::new(EptTable::new());
        self.huv.clear();
        self.hux.clear();
        self.frp.clear();
        
        
        let fqc = (aw + 0x1FFFFF) / 0x200000;
        let huw = ((fqc + 511) / 512).am(1);
        
        for ru in 0..huw {
            let mut sr = Box::new(EptTable::new());
            
            let eiz = ru * 512;
            for rn in 0..512 {
                let hub = eiz + rn;
                if hub >= fqc {
                    break;
                }
                
                let die = ixk + (hub * 0x200000) as u64;
                sr.ch[rn] = EptEntry::lnw(die);
            }
            
            let dam = sr.as_ref() as *const EptTable as u64;
            let ayi = dmy(dam);
            self.hux.push(sr);
            
            let mut ss = Box::new(EptTable::new());
            ss.ch[0] = EptEntry::css(ayi);
            
            let dan = ss.as_ref() as *const EptTable as u64;
            let auu = dmy(dan);
            self.huv.push(ss);
            
            self.wc.ch[ru] = EptEntry::css(auu);
        }
        
        crate::serial_println!("[EPT] Guest memory mapping: {} 2MB pages, {} PDPT(s)",
                              fqc, huw);
        Ok(())
    }
}


pub fn ykm(fe: &[u8]) -> Result<EptManager> {
    let aw = fe.len().am(4 * 1024 * 1024); 
    EptManager::new(aw)
}
