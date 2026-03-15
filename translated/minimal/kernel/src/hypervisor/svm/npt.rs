







use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};


pub const BM_: u64 = 4096;


pub const NY_: u64 = 2 * 1024 * 1024;


pub const NM_: u64 = 1024 * 1024 * 1024;


pub mod flags {
    pub const Cz: u64 = 1 << 0;
    pub const Ff: u64 = 1 << 1;
    pub const Gq: u64 = 1 << 2;
    pub const AJX_: u64 = 1 << 3;
    pub const DEI_: u64 = 1 << 4;
    pub const Bbh: u64 = 1 << 5;
    pub const Beb: u64 = 1 << 6;
    pub const DT_: u64 = 1 << 7;  
    pub const Bhr: u64 = 1 << 8;
    pub const DL_: u64 = 1 << 63;
    
    
    pub const Axk: u64 = Cz | Ff | Gq;
    
    
    pub const Dfq: u64 = Cz | Gq;
    
    
    pub const Bqh: u64 = Cz | Ff | Gq | DL_;
    
    
    pub const Bqf: u64 = Cz | Gq | DL_;
}


#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NptEntry(u64);

impl NptEntry {
    pub const fn azs() -> Self {
        Self(0)
    }
    
    pub const fn new(ki: u64, flags: u64) -> Self {
        Self((ki & 0x000F_FFFF_FFFF_F000) | flags)
    }
    
    #[inline]
    pub fn xo(&self) -> bool {
        (self.0 & flags::Cz) != 0
    }
    
    #[inline]
    pub fn jbl(&self) -> bool {
        (self.0 & flags::DT_) != 0
    }
    
    #[inline]
    pub fn edz(&self) -> bool {
        (self.0 & flags::Ff) != 0
    }
    
    #[inline]
    pub fn clc(&self) -> bool {
        (self.0 & flags::DL_) == 0
    }
    
    #[inline]
    pub fn ki(&self) -> u64 {
        self.0 & 0x000F_FFFF_FFFF_F000
    }
    
    #[inline]
    pub fn flags(&self) -> u64 {
        self.0 & 0xFFF0_0000_0000_0FFF
    }
    
    pub fn oj(&mut self, ki: u64, flags: u64) {
        self.0 = (ki & 0x000F_FFFF_FFFF_F000) | flags;
    }
    
    pub fn clear(&mut self) {
        self.0 = 0;
    }
    
    pub fn js(&self) -> u64 {
        self.0
    }
}


#[repr(C, align(4096))]
pub struct NptTable {
    ch: [NptEntry; 512],
}

impl NptTable {
    pub const fn new() -> Self {
        Self {
            ch: [NptEntry::azs(); 512],
        }
    }
    
    #[inline]
    pub fn bt(&self, index: usize) -> &NptEntry {
        &self.ch[index]
    }
    
    #[inline]
    pub fn epx(&mut self, index: usize) -> &mut NptEntry {
        &mut self.ch[index]
    }
    
    pub fn ki(&self) -> u64 {
        let ju = self as *const _ as u64;
        ju.nj(crate::memory::lr())
    }
}


pub struct Npt {
    
    wc: Box<NptTable>,
    
    
    tables: Vec<Box<NptTable>>,
    
    
    dht: u64,
    
    
    fkt: u64,
    
    
    ajv: u32,
}

impl Npt {
    
    pub fn new(ajv: u32) -> Self {
        Self {
            wc: Box::new(NptTable::new()),
            tables: Vec::new(),
            dht: 0,
            fkt: 0,
            ajv,
        }
    }
    
    
    pub fn lnp(&self) -> u64 {
        self.wc.ki()
    }
    
    
    pub fn jm(&self) -> u64 {
        self.lnp()
    }
    
    
    pub fn ajv(&self) -> u32 {
        self.ajv
    }
    
    
    
    
    pub fn jew(
        &mut self,
        axy: u64,
        die: u64,
        aw: u64,
        dao: u64,
    ) -> Result<(), &'static str> {
        let mut l = 0u64;
        
        while l < aw {
            let pe = axy + l;
            let dif = die + l;
            let ia = aw - l;
            
            
            if ia >= NM_ 
                && (pe & (NM_ - 1)) == 0 
                && (dif & (NM_ - 1)) == 0 
            {
                self.ujm(pe, dif, dao)?;
                l += NM_;
            } else if ia >= NY_ 
                && (pe & (NY_ - 1)) == 0 
                && (dif & (NY_ - 1)) == 0 
            {
                self.ujo(pe, dif, dao)?;
                l += NY_;
            } else {
                self.lkh(pe, dif, dao)?;
                l += BM_;
            }
        }
        
        self.dht = self.dht.am(axy + aw);
        if self.fkt == 0 {
            self.fkt = die;
        }
        
        Ok(())
    }
    
    
    fn lkh(&mut self, pe: u64, dif: u64, dao: u64) -> Result<(), &'static str> {
        let wd = ((pe >> 39) & 0x1FF) as usize;
        let ru = ((pe >> 30) & 0x1FF) as usize;
        let rn = ((pe >> 21) & 0x1FF) as usize;
        let yf = ((pe >> 12) & 0x1FF) as usize;
        
        
        let hvm: *mut NptTable = &mut *self.wc;
        let auu = self.dqo(hvm, wd)?;
        let ss = unsafe { &mut *((auu + crate::memory::lr()) as *mut NptTable) };
        
        
        let ayi = self.dqo(ss as *mut _, ru)?;
        let sr = unsafe { &mut *((ayi + crate::memory::lr()) as *mut NptTable) };
        
        
        let bwe = self.dqo(sr as *mut _, rn)?;
        let se = unsafe { &mut *((bwe + crate::memory::lr()) as *mut NptTable) };
        
        
        se.epx(yf).oj(dif, dao);
        
        Ok(())
    }
    
    
    fn ujo(&mut self, pe: u64, dif: u64, dao: u64) -> Result<(), &'static str> {
        let wd = ((pe >> 39) & 0x1FF) as usize;
        let ru = ((pe >> 30) & 0x1FF) as usize;
        let rn = ((pe >> 21) & 0x1FF) as usize;
        
        
        let hvm: *mut NptTable = &mut *self.wc;
        let auu = self.dqo(hvm, wd)?;
        let ss = unsafe { &mut *((auu + crate::memory::lr()) as *mut NptTable) };
        
        
        let ayi = self.dqo(ss as *mut _, ru)?;
        let sr = unsafe { &mut *((ayi + crate::memory::lr()) as *mut NptTable) };
        
        
        sr.epx(rn).oj(dif, dao | flags::DT_);
        
        Ok(())
    }
    
    
    fn ujm(&mut self, pe: u64, dif: u64, dao: u64) -> Result<(), &'static str> {
        let wd = ((pe >> 39) & 0x1FF) as usize;
        let ru = ((pe >> 30) & 0x1FF) as usize;
        
        
        let hvm: *mut NptTable = &mut *self.wc;
        let auu = self.dqo(hvm, wd)?;
        let ss = unsafe { &mut *((auu + crate::memory::lr()) as *mut NptTable) };
        
        
        ss.epx(ru).oj(dif, dao | flags::DT_);
        
        Ok(())
    }
    
    
    fn dqo(&mut self, lsd: *mut NptTable, index: usize) -> Result<u64, &'static str> {
        let tu = unsafe { &mut *lsd };
        
        if !tu.bt(index).xo() {
            
            let css = Box::new(NptTable::new());
            let cig = css.ki();
            self.tables.push(css);
            
            
            tu.epx(index).oj(cig, flags::Cz | flags::Ff | flags::Gq);
            Ok(cig)
        } else {
            Ok(tu.bt(index).ki())
        }
    }
    
    
    pub fn zub(&mut self, pe: u64) -> Result<(), &'static str> {
        let wd = ((pe >> 39) & 0x1FF) as usize;
        let ru = ((pe >> 30) & 0x1FF) as usize;
        let rn = ((pe >> 21) & 0x1FF) as usize;
        let yf = ((pe >> 12) & 0x1FF) as usize;
        
        
        if !self.wc.bt(wd).xo() {
            return Ok(());  
        }
        
        let auu = self.wc.bt(wd).ki();
        let ss = unsafe { &mut *((auu + crate::memory::lr()) as *mut NptTable) };
        
        if !ss.bt(ru).xo() {
            return Ok(());
        }
        
        if ss.bt(ru).jbl() {
            
            ss.epx(ru).clear();
            return Ok(());
        }
        
        let ayi = ss.bt(ru).ki();
        let sr = unsafe { &mut *((ayi + crate::memory::lr()) as *mut NptTable) };
        
        if !sr.bt(rn).xo() {
            return Ok(());
        }
        
        if sr.bt(rn).jbl() {
            
            sr.epx(rn).clear();
            return Ok(());
        }
        
        let bwe = sr.bt(rn).ki();
        let se = unsafe { &mut *((bwe + crate::memory::lr()) as *mut NptTable) };
        
        
        se.epx(yf).clear();
        
        Ok(())
    }
    
    
    pub fn dmr(&self, pe: u64) -> Option<u64> {
        let wd = ((pe >> 39) & 0x1FF) as usize;
        let ru = ((pe >> 30) & 0x1FF) as usize;
        let rn = ((pe >> 21) & 0x1FF) as usize;
        let yf = ((pe >> 12) & 0x1FF) as usize;
        
        if !self.wc.bt(wd).xo() {
            return None;
        }
        
        let auu = self.wc.bt(wd).ki();
        let ss = unsafe { &*((auu + crate::memory::lr()) as *const NptTable) };
        
        if !ss.bt(ru).xo() {
            return None;
        }
        
        if ss.bt(ru).jbl() {
            
            let ar = ss.bt(ru).ki();
            return Some(ar | (pe & (NM_ - 1)));
        }
        
        let ayi = ss.bt(ru).ki();
        let sr = unsafe { &*((ayi + crate::memory::lr()) as *const NptTable) };
        
        if !sr.bt(rn).xo() {
            return None;
        }
        
        if sr.bt(rn).jbl() {
            
            let ar = sr.bt(rn).ki();
            return Some(ar | (pe & (NY_ - 1)));
        }
        
        let bwe = sr.bt(rn).ki();
        let se = unsafe { &*((bwe + crate::memory::lr()) as *const NptTable) };
        
        if !se.bt(yf).xo() {
            return None;
        }
        
        
        let ar = se.bt(yf).ki();
        Some(ar | (pe & (BM_ - 1)))
    }
    
    
    
    pub fn pjp(&mut self, aw: u64) -> Result<(), &'static str> {
        self.jew(0, 0, aw, flags::Axk)
    }
    
    
    pub fn wkx(
        &mut self,
        die: u64,
        aw: u64,
    ) -> Result<(), &'static str> {
        
        self.jew(0, die, aw, flags::Axk)?;
        
        self.fkt = die;
        self.dht = aw;
        
        Ok(())
    }
    
    
    pub fn yys(&self) {
        unsafe {
            super::ofh(0, self.ajv);
        }
    }
    
    
    pub fn cm(&self) -> Bnl {
        Bnl {
            xah: 1 + self.tables.len(),  
            dht: self.dht,
            fkt: self.fkt,
            ajv: self.ajv,
        }
    }
}

impl Drop for Npt {
    fn drop(&mut self) {
        
        
    }
}


#[derive(Debug)]
pub struct Bnl {
    pub xah: usize,
    pub dht: u64,
    pub fkt: u64,
    pub ajv: u32,
}


pub struct AsidAllocator {
    loh: AtomicU64,
    lkr: u32,
}

impl AsidAllocator {
    pub const fn new(lkr: u32) -> Self {
        Self {
            loh: AtomicU64::new(1),  
            lkr,
        }
    }
    
    
    pub fn ijo(&self) -> Option<u32> {
        let ajv = self.loh.fetch_add(1, Ordering::SeqCst);
        if ajv as u32 >= self.lkr {
            
            self.loh.store(1, Ordering::SeqCst);
            Some(1)
        } else {
            Some(ajv as u32)
        }
    }
    
    
    pub fn aez(&self, ajv: u32) {
        
        unsafe {
            super::ofh(0, ajv);
        }
    }
}


static AKV_: AsidAllocator = AsidAllocator::new(65536);


pub fn mva() -> Option<u32> {
    AKV_.ijo()
}


pub fn sxb(ajv: u32) {
    AKV_.aez(ajv);
}


pub fn ykn(fe: &[u8]) -> Result<Npt, &'static str> {
    let ajv = mva().ok_or("Failed to allocate ASID")?;
    let mut npt = Npt::new(ajv);
    
    
    let tie = fe.fq() as u64;
    let die = tie.nj(crate::memory::lr());
    
    
    npt.wkx(die, fe.len() as u64)?;
    
    Ok(npt)
}
