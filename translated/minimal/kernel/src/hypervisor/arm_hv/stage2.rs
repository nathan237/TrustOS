
























use alloc::vec::Vec;
use alloc::vec;
use core::sync::atomic::{AtomicU64, Ordering};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum S2Perm {
    
    None,
    
    Bz,
    
    Jx,
    
    Bqs,
    
    Bv,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum S2MemType {
    
    M,
    
    Wg,
}


#[derive(Debug, Clone)]
pub struct Aem {
    
    pub fly: u64,
    
    pub jih: u64,
    
    pub aw: u64,
    
    pub egm: S2Perm,
    
    pub gmn: S2MemType,
    
    pub cu: &'static str,
}


mod desc {
    
    pub const Zh: u64 = 1 << 0;
    
    pub const Aeu: u64 = 1 << 1;
    
    pub const Byh: u64 = Zh;
    
    pub const Cio: u64  = Zh | Aeu;

    
    pub const Bxf: u64 = 1 << 10;

    
    
    pub const CPZ_: u64  = 0b00 << 6;
    
    pub const BEX_: u64    = 0b01 << 6;
    
    pub const EDR_: u64    = 0b10 << 6;
    
    pub const BEY_: u64    = 0b11 << 6;

    
    
    pub const DBV_: u64    = 0b00 << 53;
    
    pub const EKX_: u64     = 0b01 << 53;
    
    pub const DBU_: u64     = 0b10 << 53;

    
    
    pub const CGE_: u64 = 0b0000 << 2;
    
    pub const CGF_: u64 = 0b1111 << 2;
    
    pub const DTT_: u64 = 0b0101 << 2;

    
    pub const CTF_: u64 = 0b11 << 8;
    pub const EGJ_: u64 = 0b10 << 8;
    pub const CTG_: u64  = 0b00 << 8;
}


const BM_: u64 = 4096;
const NV_: u64 = 2 * 1024 * 1024;     
const DSH_: u64 = 1024 * 1024 * 1024;   


const EG_: usize = 512;


pub struct Stage2Tables {
    
    lho: *mut u64,
    
    kad: Vec<*mut u64>,
    
    gmc: Vec<Aem>,
    
    fbe: u64,
}

unsafe impl Send for Stage2Tables {}
unsafe impl Sync for Stage2Tables {}

impl Stage2Tables {
    
    pub fn new(xsl: u16) -> Self {
        let fmk = Self::kac();
        let awk = fmk as u64;

        
        let fbe = ((xsl as u64) << 48) | (awk & 0x0000_FFFF_FFFF_F000);

        Stage2Tables {
            lho: fmk,
            kad: vec![fmk],
            gmc: Vec::new(),
            fbe,
        }
    }

    
    pub fn fbe(&self) -> u64 {
        self.fbe
    }

    
    pub fn ujs(&mut self, ar: u64, aw: u64) {
        self.okz(ar, ar, aw, S2Perm::Bv, S2MemType::M, "RAM");
    }

    
    
    pub fn zbx(&mut self, ar: u64, aw: u64, cu: &'static str) {
        self.okz(ar, ar, aw, S2Perm::Jx, S2MemType::Wg, cu);
    }

    
    
    pub fn guw(&mut self, ar: u64, aw: u64, cu: &'static str) {
        
        self.gmc.push(Aem {
            fly: ar,
            jih: ar,
            aw,
            egm: S2Perm::None,
            gmn: S2MemType::Wg,
            cu,
        });
        
    }

    
    pub fn okz(
        &mut self,
        fly: u64,
        jih: u64,
        aw: u64,
        egm: S2Perm,
        gmn: S2MemType,
        cu: &'static str,
    ) {
        self.gmc.push(Aem {
            fly, jih, aw, egm, gmn, cu,
        });

        if egm == S2Perm::None {
            return; 
        }

        
        let qn = self.qtd(egm, gmn);

        
        let mut akh = fly & !0xFFF;
        let mut awk = jih & !0xFFF;
        let ci = fly + aw;

        while akh < ci {
            
            if akh & (NV_ - 1) == 0
                && awk & (NV_ - 1) == 0
                && akh + NV_ <= ci
            {
                self.ujn(akh, awk, qn);
                akh += NV_;
                awk += NV_;
            } else {
                self.lkh(akh, awk, qn);
                akh += BM_;
                awk += BM_;
            }
        }
    }

    
    fn qtd(&self, egm: S2Perm, gmn: S2MemType) -> u64 {
        let mut qn: u64 = desc::Bxf;

        
        qn |= match egm {
            S2Perm::None => desc::CPZ_,
            S2Perm::Bz => desc::BEX_,
            S2Perm::Jx => desc::BEY_,
            S2Perm::Bqs => desc::BEX_,
            S2Perm::Bv => desc::BEY_,
        };

        
        qn |= match egm {
            S2Perm::Bqs | S2Perm::Bv => desc::DBV_,
            _ => desc::DBU_,
        };

        
        qn |= match gmn {
            S2MemType::M => desc::CGF_ | desc::CTF_,
            S2MemType::Wg => desc::CGE_ | desc::CTG_,
        };

        qn
    }

    
    fn ujn(&mut self, akh: u64, awk: u64, qn: u64) {
        let crx = ((akh >> 30) & 0x1FF) as usize;
        let dsn = ((akh >> 21) & 0x1FF) as usize;

        
        let bvd = self.nyh(crx);

        
        unsafe {
            let bt = awk & 0x0000_FFFF_FFE0_0000 | qn | desc::Byh;
            bvd.add(dsn).write_volatile(bt);
        }
    }

    
    fn lkh(&mut self, akh: u64, awk: u64, qn: u64) {
        let crx = ((akh >> 30) & 0x1FF) as usize;
        let dsn = ((akh >> 21) & 0x1FF) as usize;
        let ubu = ((akh >> 12) & 0x1FF) as usize;

        let bvd = self.nyh(crx);
        let eei = self.teg(bvd, dsn);

        
        unsafe {
            let bt = awk & 0x0000_FFFF_FFFF_F000 | qn | desc::Cio;
            eei.add(ubu).write_volatile(bt);
        }
    }

    
    fn nyh(&mut self, crx: usize) -> *mut u64 {
        unsafe {
            let bt = self.lho.add(crx).read_volatile();
            if bt & desc::Zh != 0 && bt & desc::Aeu != 0 {
                
                (bt & 0x0000_FFFF_FFFF_F000) as *mut u64
            } else {
                
                let bvd = Self::kac();
                self.kad.push(bvd);
                let mjr = (bvd as u64 & 0x0000_FFFF_FFFF_F000) | desc::Zh | desc::Aeu;
                self.lho.add(crx).write_volatile(mjr);
                bvd
            }
        }
    }

    
    fn teg(&mut self, bvd: *mut u64, dsn: usize) -> *mut u64 {
        unsafe {
            let bt = bvd.add(dsn).read_volatile();
            if bt & desc::Zh != 0 && bt & desc::Aeu != 0 {
                (bt & 0x0000_FFFF_FFFF_F000) as *mut u64
            } else {
                let eei = Self::kac();
                self.kad.push(eei);
                let mjr = (eei as u64 & 0x0000_FFFF_FFFF_F000) | desc::Zh | desc::Aeu;
                bvd.add(dsn).write_volatile(mjr);
                eei
            }
        }
    }

    
    fn kac() -> *mut u64 {
        
        let layout = core::alloc::Layout::bjy(4096, 4096).unwrap();
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) as *mut u64 };
        if ptr.abq() {
            panic!("Stage-2 page table allocation failed");
        }
        ptr
    }

    
    pub fn ghg(&self) {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!(
                "dsb ishst",
                "tlbi vmalls12e1is",  
                "dsb ish",
                "isb",
                options(nomem, nostack)
            );
        }
    }

    
    pub fn gmc(&self) -> &[Aem] {
        &self.gmc
    }

    
    pub fn zac(&self, akh: u64) -> Option<&Aem> {
        self.gmc.iter().du(|ef| {
            ef.egm == S2Perm::None
                && akh >= ef.fly
                && akh < ef.fly + ef.aw
        })
    }
}
