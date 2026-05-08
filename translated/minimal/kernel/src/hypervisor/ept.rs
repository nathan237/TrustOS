







use alloc::boxed::Box;
use alloc::vec::Vec;
use super::{HypervisorError, Result};
use super::vmx::bjv;






#[derive(Clone, Copy)]
pub struct EptPointer(pub u64);

impl EptPointer {
    pub fn new(pml4_phys: u64) -> Self {
        
        let lrk = (pml4_phys & 0xFFFF_FFFF_FFFF_F000) | (3 << 3) | 6;
        EptPointer(lrk)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}


pub mod flags {
    pub const Ba: u64 = 1 << 0;
    pub const Bh: u64 = 1 << 1;
    pub const Fm: u64 = 1 << 2;
    pub const DXM_: u64 = 0x38; 
    pub const BCJ_: u64 = 6 << 3;
    pub const DXN_: u64 = 0 << 3;
    pub const DUP_: u64 = 1 << 6;
    pub const BAQ_: u64 = 1 << 7;
    pub const Wc: u64 = 1 << 8;
    pub const Xm: u64 = 1 << 9;
    pub const DOX_: u64 = 1 << 10;
}


#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct EptEntry(u64);

impl EptEntry {
    pub const fn empty() -> Self {
        EptEntry(0)
    }
    
    pub fn ayl(asj: u64) -> Self {
        EptEntry(asj | flags::Ba | flags::Bh | flags::Fm)
    }
    
    pub fn gje(bcy: u64, rwx: bool) -> Self {
        let mut entry = bcy | flags::BCJ_;
        if rwx {
            entry |= flags::Ba | flags::Bh | flags::Fm;
        }
        EptEntry(entry)
    }
    
    pub fn gjc(bcy: u64) -> Self {
        EptEntry(bcy | flags::Ba | flags::Bh | flags::Fm 
                 | flags::BAQ_ | flags::BCJ_)
    }
    
    pub fn is_present(&self) -> bool {
        (self.0 & (flags::Ba | flags::Bh | flags::Fm)) != 0
    }
    
    pub fn qml(&self) -> bool {
        (self.0 & flags::BAQ_) != 0
    }
    
    pub fn phys_addr(&self) -> u64 {
        self.0 & 0xFFFF_FFFF_FFFF_F000
    }
}


#[repr(C, align(4096))]
pub struct EptTable {
    entries: [EptEntry; 512],
}

impl EptTable {
    pub fn new() -> Self {
        EptTable {
            entries: [EptEntry::empty(); 512],
        }
    }
    
    pub fn entry(&self, index: usize) -> &EptEntry {
        &self.entries[index]
    }
    
    pub fn entry_mut(&mut self, index: usize) -> &mut EptEntry {
        &mut self.entries[index]
    }
}


pub struct EptManager {
    
    pml4: Box<EptTable>,
    
    pdpts: Vec<Box<EptTable>>,
    
    pds: Vec<Box<EptTable>>,
    
    pts: Vec<Box<EptTable>>,
    
    guest_memory_size: usize,
}

impl EptManager {
    
    pub fn new(guest_memory_size: usize) -> Result<Self> {
        let mut ils = EptManager {
            pml4: Box::new(EptTable::new()),
            pdpts: Vec::new(),
            pds: Vec::new(),
            pts: Vec::new(),
            guest_memory_size,
        };
        
        
        
        ils.setup_identity_mapping(guest_memory_size)?;
        
        Ok(ils)
    }
    
    
    pub fn ept_pointer(&self) -> EptPointer {
        let dcu = self.pml4.as_ref() as *const EptTable as u64;
        let pml4_phys = bjv(dcu);
        crate::serial_println!("[EPT] PML4 virt=0x{:016X} phys=0x{:016X}", dcu, pml4_phys);
        EptPointer::new(pml4_phys)
    }
    
    
    fn setup_identity_mapping(&mut self, size: usize) -> Result<()> {
        crate::serial_println!("[EPT] Setting up identity mapping for {} MB", size / (1024 * 1024));
        
        
        let cnr = (size + 0x1FFFFF) / 0x200000;
        let dwj = ((cnr + 511) / 512).max(1);
        
        for jc in 0..dwj {
            let mut js = Box::new(EptTable::new());
            
            
            let bvy = jc * 512;
            for iw in 0..512 {
                let dwc = bvy + iw;
                if dwc >= cnr {
                    break;
                }
                
                let phys_addr = (dwc * 0x200000) as u64;
                js.entries[iw] = EptEntry::gjc(phys_addr);
            }
            
            let cco = js.as_ref() as *const EptTable as u64;
            let aae = bjv(cco);
            self.pds.push(js);
            
            
            let mut jt = Box::new(EptTable::new());
            jt.entries[0] = EptEntry::ayl(aae);
            
            let ccp = jt.as_ref() as *const EptTable as u64;
            let xz = bjv(ccp);
            self.pdpts.push(jt);
            
            
            self.pml4.entries[jc] = EptEntry::ayl(xz);
        }
        
        crate::serial_println!("[EPT] Identity mapping configured: {} 2MB pages, {} PDPT(s)", 
                              cnr, dwj);
        
        Ok(())
    }
    
    
    pub fn map_page(&mut self, zy: u64, bha: u64, bej: u64) -> Result<()> {
        let lu = ((zy >> 39) & 0x1FF) as usize;
        let jc = ((zy >> 30) & 0x1FF) as usize;
        let iw = ((zy >> 21) & 0x1FF) as usize;
        
        
        if !self.pml4.entries[lu].is_present() {
            let jt = Box::new(EptTable::new());
            let ccp = jt.as_ref() as *const EptTable as u64;
            let xz = bjv(ccp);
            self.pml4.entries[lu] = EptEntry::ayl(xz);
            self.pdpts.push(jt);
        }
        
        
        let ntj = self.pml4.entries[lu].phys_addr();
        let ntk = crate::memory::wk(ntj);
        let gmr = unsafe { &mut *(ntk as *mut EptTable) };
        
        if !gmr.entries[jc].is_present() {
            let js = Box::new(EptTable::new());
            let cco = js.as_ref() as *const EptTable as u64;
            let aae = bjv(cco);
            gmr.entries[jc] = EptEntry::ayl(aae);
            self.pds.push(js);
        }
        
        let nte = gmr.entries[jc].phys_addr();
        let ntf = crate::memory::wk(nte);
        let ntd = unsafe { &mut *(ntf as *mut EptTable) };
        
        
        let juj = bha & !0x1FFFFF;
        ntd.entries[iw] = EptEntry::gjc(juj);
        
        Ok(())
    }

    
    
    
    
    pub fn setup_guest_memory_mapping(&mut self, guest_memory: &[u8]) -> Result<()> {
        let mgk = guest_memory.as_ptr() as u64;
        let eop = bjv(mgk);
        let size = guest_memory.len();
        
        crate::serial_println!("[EPT] Mapping GPA 0x0 -> HPA 0x{:X} ({} MB)",
                              eop, size / (1024 * 1024));
        
        
        self.pml4 = Box::new(EptTable::new());
        self.pdpts.clear();
        self.pds.clear();
        self.pts.clear();
        
        
        let cnr = (size + 0x1FFFFF) / 0x200000;
        let dwj = ((cnr + 511) / 512).max(1);
        
        for jc in 0..dwj {
            let mut js = Box::new(EptTable::new());
            
            let bvy = jc * 512;
            for iw in 0..512 {
                let dwc = bvy + iw;
                if dwc >= cnr {
                    break;
                }
                
                let bha = eop + (dwc * 0x200000) as u64;
                js.entries[iw] = EptEntry::gjc(bha);
            }
            
            let cco = js.as_ref() as *const EptTable as u64;
            let aae = bjv(cco);
            self.pds.push(js);
            
            let mut jt = Box::new(EptTable::new());
            jt.entries[0] = EptEntry::ayl(aae);
            
            let ccp = jt.as_ref() as *const EptTable as u64;
            let xz = bjv(ccp);
            self.pdpts.push(jt);
            
            self.pml4.entries[jc] = EptEntry::ayl(xz);
        }
        
        crate::serial_println!("[EPT] Guest memory mapping: {} 2MB pages, {} PDPT(s)",
                              cnr, dwj);
        Ok(())
    }
}


pub fn qbt(guest_memory: &[u8]) -> Result<EptManager> {
    let size = guest_memory.len().max(4 * 1024 * 1024); 
    EptManager::new(size)
}
