




use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;


pub mod prot {
    pub const COE_: u32 = 0;
    pub const COF_: u32 = 1;
    pub const XT_: u32 = 2;
    pub const AIL_: u32 = 4;
}


pub mod flags {
    pub const AGM_: u32 = 0x20;
    pub const BBH_: u32 = 0x02;
    pub const CIK_: u32 = 0x01;
}


#[derive(Clone, Debug)]
pub struct He {
    
    pub start: u64,
    
    pub end: u64,
    
    pub prot: u32,
    
    pub flags: u32,
}

impl He {
    
    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr < self.end
    }
    
    
    pub fn size(&self) -> u64 {
        self.end - self.start
    }
}


static KA_: Mutex<BTreeMap<u64, Vec<He>>> = Mutex::new(BTreeMap::new());


pub fn jty(cr3: u64, vma: He) {
    let mut bs = KA_.lock();
    bs.entry(cr3).or_insert_with(Vec::new).push(vma);
}


pub fn nas(cr3: u64, addr: u64) -> Option<He> {
    let bs = KA_.lock();
    if let Some(vmas) = bs.get(&cr3) {
        for vma in vmas {
            if vma.contains(addr) {
                return Some(vma.clone());
            }
        }
    }
    None
}


pub fn ofc(cr3: u64, start: u64, end: u64) {
    let mut bs = KA_.lock();
    if let Some(vmas) = bs.get_mut(&cr3) {
        
        let mut euy = Vec::new();
        for vma in vmas.drain(..) {
            if vma.end <= start || vma.start >= end {
                
                euy.push(vma);
            } else {
                
                if vma.start < start {
                    euy.push(He {
                        start: vma.start,
                        end: start,
                        prot: vma.prot,
                        flags: vma.flags,
                    });
                }
                if vma.end > end {
                    euy.push(He {
                        start: end,
                        end: vma.end,
                        prot: vma.prot,
                        flags: vma.flags,
                    });
                }
            }
        }
        *vmas = euy;
    }
}


pub fn rbs(cr3: u64, start: u64, end: u64, new_prot: u32) {
    let mut bs = KA_.lock();
    if let Some(vmas) = bs.get_mut(&cr3) {
        for vma in vmas.iter_mut() {
            if vma.start < end && vma.end > start {
                vma.prot = new_prot;
            }
        }
    }
}


pub fn qac(src_cr3: u64, dst_cr3: u64) {
    let mut bs = KA_.lock();
    if let Some(vmas) = bs.get(&src_cr3) {
        let cloned = vmas.clone();
        bs.insert(dst_cr3, cloned);
    }
}


pub fn qto(cr3: u64) {
    KA_.lock().remove(&cr3);
}


pub fn mzk(cr3: u64) -> Vec<He> {
    let bs = KA_.lock();
    bs.get(&cr3).cloned().unwrap_or_default()
}


pub fn nyx(prot_flags: u32) -> crate::memory::paging::PageFlags {
    use crate::memory::paging::PageFlags;
    
    let mut f = PageFlags::Bg | PageFlags::Cz | PageFlags::DT_;
    if (prot_flags & prot::XT_) != 0 {
        f |= PageFlags::Cg;
    }
    if (prot_flags & prot::AIL_) != 0 {
        
        f = PageFlags::Bg | PageFlags::Cz;
        if (prot_flags & prot::XT_) != 0 {
            f |= PageFlags::Cg;
        }
    }
    PageFlags::new(f)
}
