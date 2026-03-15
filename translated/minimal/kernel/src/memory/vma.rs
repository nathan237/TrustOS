




use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;


pub mod prot {
    pub const CKV_: u32 = 0;
    pub const CKW_: u32 = 1;
    pub const WK_: u32 = 2;
    pub const AGR_: u32 = 4;
}


pub mod flags {
    pub const AES_: u32 = 0x20;
    pub const AZG_: u32 = 0x02;
    pub const CFB_: u32 = 0x01;
}


#[derive(Clone, Debug)]
pub struct Rf {
    
    pub ay: u64,
    
    pub ci: u64,
    
    pub prot: u32,
    
    pub flags: u32,
}

impl Rf {
    
    pub fn contains(&self, ag: u64) -> bool {
        ag >= self.ay && ag < self.ci
    }
    
    
    pub fn aw(&self) -> u64 {
        self.ci - self.ay
    }
}


static JH_: Mutex<BTreeMap<u64, Vec<Rf>>> = Mutex::new(BTreeMap::new());


pub fn qfp(jm: u64, vma: Rf) {
    let mut gg = JH_.lock();
    gg.bt(jm).clq(Vec::new).push(vma);
}


pub fn uii(jm: u64, ag: u64) -> Option<Rf> {
    let gg = JH_.lock();
    if let Some(fba) = gg.get(&jm) {
        for vma in fba {
            if vma.contains(ag) {
                return Some(vma.clone());
            }
        }
    }
    None
}


pub fn vva(jm: u64, ay: u64, ci: u64) {
    let mut gg = JH_.lock();
    if let Some(fba) = gg.ds(&jm) {
        
        let mut jgv = Vec::new();
        for vma in fba.bbk(..) {
            if vma.ci <= ay || vma.ay >= ci {
                
                jgv.push(vma);
            } else {
                
                if vma.ay < ay {
                    jgv.push(Rf {
                        ay: vma.ay,
                        ci: ay,
                        prot: vma.prot,
                        flags: vma.flags,
                    });
                }
                if vma.ci > ci {
                    jgv.push(Rf {
                        ay: ci,
                        ci: vma.ci,
                        prot: vma.prot,
                        flags: vma.flags,
                    });
                }
            }
        }
        *fba = jgv;
    }
}


pub fn zuw(jm: u64, ay: u64, ci: u64, uto: u32) {
    let mut gg = JH_.lock();
    if let Some(fba) = gg.ds(&jm) {
        for vma in fba.el() {
            if vma.ay < ci && vma.ci > ay {
                vma.prot = uto;
            }
        }
    }
}


pub fn yir(wrr: u64, sgz: u64) {
    let mut gg = JH_.lock();
    if let Some(fba) = gg.get(&wrr) {
        let abn = fba.clone();
        gg.insert(sgz, abn);
    }
}


pub fn zjd(jm: u64) {
    JH_.lock().remove(&jm);
}


pub fn ufz(jm: u64) -> Vec<Rf> {
    let gg = JH_.lock();
    gg.get(&jm).abn().age()
}


pub fn vni(prot_flags: u32) -> crate::memory::paging::PageFlags {
    use crate::memory::paging::PageFlags;
    
    let mut bb = PageFlags::Cz | PageFlags::Gq | PageFlags::DL_;
    if (prot_flags & prot::WK_) != 0 {
        bb |= PageFlags::Ff;
    }
    if (prot_flags & prot::AGR_) != 0 {
        
        bb = PageFlags::Cz | PageFlags::Gq;
        if (prot_flags & prot::WK_) != 0 {
            bb |= PageFlags::Ff;
        }
    }
    PageFlags::new(bb)
}
