







use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};


pub const BO_: u64 = 4096;


pub const OW_: u64 = 2 * 1024 * 1024;


pub const ON_: u64 = 1024 * 1024 * 1024;


pub mod flags {
    pub const Bg: u64 = 1 << 0;
    pub const Cg: u64 = 1 << 1;
    pub const Cz: u64 = 1 << 2;
    pub const ALS_: u64 = 1 << 3;
    pub const DIC_: u64 = 1 << 4;
    pub const Wc: u64 = 1 << 5;
    pub const Xm: u64 = 1 << 6;
    pub const EE_: u64 = 1 << 7;  
    pub const Zd: u64 = 1 << 8;
    pub const DT_: u64 = 1 << 63;
    
    
    pub const Uq: u64 = Bg | Cg | Cz;
    
    
    pub const Bbj: u64 = Bg | Cz;
    
    
    pub const Adh: u64 = Bg | Cg | Cz | DT_;
    
    
    pub const Adf: u64 = Bg | Cz | DT_;
}


#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NptEntry(u64);

impl NptEntry {
    pub const fn empty() -> Self {
        Self(0)
    }
    
    pub const fn new(phys_addr: u64, flags: u64) -> Self {
        Self((phys_addr & 0x000F_FFFF_FFFF_F000) | flags)
    }
    
    #[inline]
    pub fn is_present(&self) -> bool {
        (self.0 & flags::Bg) != 0
    }
    
    #[inline]
    pub fn is_huge(&self) -> bool {
        (self.0 & flags::EE_) != 0
    }
    
    #[inline]
    pub fn is_writable(&self) -> bool {
        (self.0 & flags::Cg) != 0
    }
    
    #[inline]
    pub fn is_executable(&self) -> bool {
        (self.0 & flags::DT_) == 0
    }
    
    #[inline]
    pub fn phys_addr(&self) -> u64 {
        self.0 & 0x000F_FFFF_FFFF_F000
    }
    
    #[inline]
    pub fn flags(&self) -> u64 {
        self.0 & 0xFFF0_0000_0000_0FFF
    }
    
    pub fn set(&mut self, phys_addr: u64, flags: u64) {
        self.0 = (phys_addr & 0x000F_FFFF_FFFF_F000) | flags;
    }
    
    pub fn clear(&mut self) {
        self.0 = 0;
    }
    
    pub fn dm(&self) -> u64 {
        self.0
    }
}


#[repr(C, align(4096))]
pub struct NptTable {
    entries: [NptEntry; 512],
}

impl NptTable {
    pub const fn new() -> Self {
        Self {
            entries: [NptEntry::empty(); 512],
        }
    }
    
    #[inline]
    pub fn entry(&self, index: usize) -> &NptEntry {
        &self.entries[index]
    }
    
    #[inline]
    pub fn entry_mut(&mut self, index: usize) -> &mut NptEntry {
        &mut self.entries[index]
    }
    
    pub fn phys_addr(&self) -> u64 {
        let virt = self as *const _ as u64;
        virt.wrapping_sub(crate::memory::hhdm_offset())
    }
}


pub struct Npt {
    
    pml4: Box<NptTable>,
    
    
    tables: Vec<Box<NptTable>>,
    
    
    guest_memory_size: u64,
    
    
    host_phys_base: u64,
    
    
    asid: u32,
}

impl Npt {
    
    pub fn new(asid: u32) -> Self {
        Self {
            pml4: Box::new(NptTable::new()),
            tables: Vec::new(),
            guest_memory_size: 0,
            host_phys_base: 0,
            asid,
        }
    }
    
    
    pub fn ncr3(&self) -> u64 {
        self.pml4.phys_addr()
    }
    
    
    pub fn cr3(&self) -> u64 {
        self.ncr3()
    }
    
    
    pub fn asid(&self) -> u32 {
        self.asid
    }
    
    
    
    
    pub fn map_range(
        &mut self,
        zy: u64,
        bha: u64,
        size: u64,
        bda: u64,
    ) -> Result<(), &'static str> {
        let mut offset = 0u64;
        
        while offset < size {
            let gm = zy + offset;
            let bhb = bha + offset;
            let ck = size - offset;
            
            
            if ck >= ON_ 
                && (gm & (ON_ - 1)) == 0 
                && (bhb & (ON_ - 1)) == 0 
            {
                self.map_1gb_page(gm, bhb, bda)?;
                offset += ON_;
            } else if ck >= OW_ 
                && (gm & (OW_ - 1)) == 0 
                && (bhb & (OW_ - 1)) == 0 
            {
                self.map_2mb_page(gm, bhb, bda)?;
                offset += OW_;
            } else {
                self.map_4kb_page(gm, bhb, bda)?;
                offset += BO_;
            }
        }
        
        self.guest_memory_size = self.guest_memory_size.max(zy + size);
        if self.host_phys_base == 0 {
            self.host_phys_base = bha;
        }
        
        Ok(())
    }
    
    
    fn map_4kb_page(&mut self, gm: u64, bhb: u64, bda: u64) -> Result<(), &'static str> {
        let lu = ((gm >> 39) & 0x1FF) as usize;
        let jc = ((gm >> 30) & 0x1FF) as usize;
        let iw = ((gm >> 21) & 0x1FF) as usize;
        let mw = ((gm >> 12) & 0x1FF) as usize;
        
        
        let dwr: *mut NptTable = &mut *self.pml4;
        let xz = self.ensure_table_at(dwr, lu)?;
        let jt = unsafe { &mut *((xz + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        
        let aae = self.ensure_table_at(jt as *mut _, jc)?;
        let js = unsafe { &mut *((aae + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        
        let amj = self.ensure_table_at(js as *mut _, iw)?;
        let jd = unsafe { &mut *((amj + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        
        jd.entry_mut(mw).set(bhb, bda);
        
        Ok(())
    }
    
    
    fn map_2mb_page(&mut self, gm: u64, bhb: u64, bda: u64) -> Result<(), &'static str> {
        let lu = ((gm >> 39) & 0x1FF) as usize;
        let jc = ((gm >> 30) & 0x1FF) as usize;
        let iw = ((gm >> 21) & 0x1FF) as usize;
        
        
        let dwr: *mut NptTable = &mut *self.pml4;
        let xz = self.ensure_table_at(dwr, lu)?;
        let jt = unsafe { &mut *((xz + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        
        let aae = self.ensure_table_at(jt as *mut _, jc)?;
        let js = unsafe { &mut *((aae + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        
        js.entry_mut(iw).set(bhb, bda | flags::EE_);
        
        Ok(())
    }
    
    
    fn map_1gb_page(&mut self, gm: u64, bhb: u64, bda: u64) -> Result<(), &'static str> {
        let lu = ((gm >> 39) & 0x1FF) as usize;
        let jc = ((gm >> 30) & 0x1FF) as usize;
        
        
        let dwr: *mut NptTable = &mut *self.pml4;
        let xz = self.ensure_table_at(dwr, lu)?;
        let jt = unsafe { &mut *((xz + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        
        jt.entry_mut(jc).set(bhb, bda | flags::EE_);
        
        Ok(())
    }
    
    
    fn ensure_table_at(&mut self, gmd: *mut NptTable, index: usize) -> Result<u64, &'static str> {
        let parent = unsafe { &mut *gmd };
        
        if !parent.entry(index).is_present() {
            
            let ayl = Box::new(NptTable::new());
            let asj = ayl.phys_addr();
            self.tables.push(ayl);
            
            
            parent.entry_mut(index).set(asj, flags::Bg | flags::Cg | flags::Cz);
            Ok(asj)
        } else {
            Ok(parent.entry(index).phys_addr())
        }
    }
    
    
    pub fn rbh(&mut self, gm: u64) -> Result<(), &'static str> {
        let lu = ((gm >> 39) & 0x1FF) as usize;
        let jc = ((gm >> 30) & 0x1FF) as usize;
        let iw = ((gm >> 21) & 0x1FF) as usize;
        let mw = ((gm >> 12) & 0x1FF) as usize;
        
        
        if !self.pml4.entry(lu).is_present() {
            return Ok(());  
        }
        
        let xz = self.pml4.entry(lu).phys_addr();
        let jt = unsafe { &mut *((xz + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        if !jt.entry(jc).is_present() {
            return Ok(());
        }
        
        if jt.entry(jc).is_huge() {
            
            jt.entry_mut(jc).clear();
            return Ok(());
        }
        
        let aae = jt.entry(jc).phys_addr();
        let js = unsafe { &mut *((aae + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        if !js.entry(iw).is_present() {
            return Ok(());
        }
        
        if js.entry(iw).is_huge() {
            
            js.entry_mut(iw).clear();
            return Ok(());
        }
        
        let amj = js.entry(iw).phys_addr();
        let jd = unsafe { &mut *((amj + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        
        jd.entry_mut(mw).clear();
        
        Ok(())
    }
    
    
    pub fn translate(&self, gm: u64) -> Option<u64> {
        let lu = ((gm >> 39) & 0x1FF) as usize;
        let jc = ((gm >> 30) & 0x1FF) as usize;
        let iw = ((gm >> 21) & 0x1FF) as usize;
        let mw = ((gm >> 12) & 0x1FF) as usize;
        
        if !self.pml4.entry(lu).is_present() {
            return None;
        }
        
        let xz = self.pml4.entry(lu).phys_addr();
        let jt = unsafe { &*((xz + crate::memory::hhdm_offset()) as *const NptTable) };
        
        if !jt.entry(jc).is_present() {
            return None;
        }
        
        if jt.entry(jc).is_huge() {
            
            let base = jt.entry(jc).phys_addr();
            return Some(base | (gm & (ON_ - 1)));
        }
        
        let aae = jt.entry(jc).phys_addr();
        let js = unsafe { &*((aae + crate::memory::hhdm_offset()) as *const NptTable) };
        
        if !js.entry(iw).is_present() {
            return None;
        }
        
        if js.entry(iw).is_huge() {
            
            let base = js.entry(iw).phys_addr();
            return Some(base | (gm & (OW_ - 1)));
        }
        
        let amj = js.entry(iw).phys_addr();
        let jd = unsafe { &*((amj + crate::memory::hhdm_offset()) as *const NptTable) };
        
        if !jd.entry(mw).is_present() {
            return None;
        }
        
        
        let base = jd.entry(mw).phys_addr();
        Some(base | (gm & (BO_ - 1)))
    }
    
    
    
    pub fn setup_identity_mapping(&mut self, size: u64) -> Result<(), &'static str> {
        self.map_range(0, 0, size, flags::Uq)
    }
    
    
    pub fn setup_guest_memory(
        &mut self,
        bha: u64,
        size: u64,
    ) -> Result<(), &'static str> {
        
        self.map_range(0, bha, size, flags::Uq)?;
        
        self.host_phys_base = bha;
        self.guest_memory_size = size;
        
        Ok(())
    }
    
    
    pub fn qlw(&self) {
        unsafe {
            super::ihj(0, self.asid);
        }
    }
    
    
    pub fn stats(&self) -> Abt {
        Abt {
            table_count: 1 + self.tables.len(),  
            guest_memory_size: self.guest_memory_size,
            host_phys_base: self.host_phys_base,
            asid: self.asid,
        }
    }
}

impl Drop for Npt {
    fn drop(&mut self) {
        
        
    }
}


#[derive(Debug)]
pub struct Abt {
    pub table_count: usize,
    pub guest_memory_size: u64,
    pub host_phys_base: u64,
    pub asid: u32,
}


pub struct AsidAllocator {
    next_asid: AtomicU64,
    max_asid: u32,
}

impl AsidAllocator {
    pub const fn new(max_asid: u32) -> Self {
        Self {
            next_asid: AtomicU64::new(1),  
            max_asid,
        }
    }
    
    
    pub fn allocate(&self) -> Option<u32> {
        let asid = self.next_asid.fetch_add(1, Ordering::SeqCst);
        if asid as u32 >= self.max_asid {
            
            self.next_asid.store(1, Ordering::SeqCst);
            Some(1)
        } else {
            Some(asid as u32)
        }
    }
    
    
    pub fn free(&self, asid: u32) {
        
        unsafe {
            super::ihj(0, asid);
        }
    }
}


static AMQ_: AsidAllocator = AsidAllocator::new(65536);


pub fn hev() -> Option<u32> {
    AMQ_.allocate()
}


pub fn lyo(asid: u32) {
    AMQ_.free(asid);
}


pub fn qbu(guest_memory: &[u8]) -> Result<Npt, &'static str> {
    let asid = hev().ok_or("Failed to allocate ASID")?;
    let mut npt = Npt::new(asid);
    
    
    let mgn = guest_memory.as_ptr() as u64;
    let bha = mgn.wrapping_sub(crate::memory::hhdm_offset());
    
    
    npt.setup_guest_memory(bha, guest_memory.len() as u64)?;
    
    Ok(npt)
}
