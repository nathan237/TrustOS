
























use alloc::vec::Vec;
use alloc::vec;
use core::sync::atomic::{AtomicU64, Ordering};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum S2Perm {
    
    None,
    
    ReadOnly,
    
    ReadWrite,
    
    ReadExec,
    
    Full,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum S2MemType {
    
    Normal,
    
    Device,
}


#[derive(Debug, Clone)]
pub struct Nd {
    
    pub ipa_base: u64,
    
    pub pa_base: u64,
    
    pub size: u64,
    
    pub perm: S2Perm,
    
    pub mem_type: S2MemType,
    
    pub label: &'static str,
}


mod desc {
    
    pub const Kw: u64 = 1 << 0;
    
    pub const Nj: u64 = 1 << 1;
    
    pub const Ahg: u64 = Kw;
    
    pub const Anb: u64  = Kw | Nj;

    
    pub const Agu: u64 = 1 << 10;

    
    
    pub const CTO_: u64  = 0b00 << 6;
    
    pub const BHA_: u64    = 0b01 << 6;
    
    pub const EHJ_: u64    = 0b10 << 6;
    
    pub const BHB_: u64    = 0b11 << 6;

    
    
    pub const DFQ_: u64    = 0b00 << 53;
    
    pub const EOI_: u64     = 0b01 << 53;
    
    pub const DFP_: u64     = 0b10 << 53;

    
    
    pub const CJO_: u64 = 0b0000 << 2;
    
    pub const CJP_: u64 = 0b1111 << 2;
    
    pub const DXK_: u64 = 0b0101 << 2;

    
    pub const CWW_: u64 = 0b11 << 8;
    pub const EKC_: u64 = 0b10 << 8;
    pub const CWX_: u64  = 0b00 << 8;
}


const BO_: u64 = 4096;
const OT_: u64 = 2 * 1024 * 1024;     
const DWA_: u64 = 1024 * 1024 * 1024;   


const ET_: usize = 512;


pub struct Stage2Tables {
    
    l1_table: *mut u64,
    
    allocated_pages: Vec<*mut u64>,
    
    mappings: Vec<Nd>,
    
    vttbr: u64,
}

unsafe impl Send for Stage2Tables {}
unsafe impl Sync for Stage2Tables {}

impl Stage2Tables {
    
    pub fn new(vmid: u16) -> Self {
        let clt = Self::fgu();
        let pa = clt as u64;

        
        let vttbr = ((vmid as u64) << 48) | (pa & 0x0000_FFFF_FFFF_F000);

        Stage2Tables {
            l1_table: clt,
            allocated_pages: vec![clt],
            mappings: Vec::new(),
            vttbr,
        }
    }

    
    pub fn vttbr(&self) -> u64 {
        self.vttbr
    }

    
    pub fn map_ram(&mut self, base: u64, size: u64) {
        self.map_region(base, base, size, S2Perm::Full, S2MemType::Normal, "RAM");
    }

    
    
    pub fn qoj(&mut self, base: u64, size: u64, label: &'static str) {
        self.map_region(base, base, size, S2Perm::ReadWrite, S2MemType::Device, label);
    }

    
    
    pub fn trap_mmio(&mut self, base: u64, size: u64, label: &'static str) {
        
        self.mappings.push(Nd {
            ipa_base: base,
            pa_base: base,
            size,
            perm: S2Perm::None,
            mem_type: S2MemType::Device,
            label,
        });
        
    }

    
    pub fn map_region(
        &mut self,
        ipa_base: u64,
        pa_base: u64,
        size: u64,
        perm: S2Perm,
        mem_type: S2MemType,
        label: &'static str,
    ) {
        self.mappings.push(Nd {
            ipa_base, pa_base, size, perm, mem_type, label,
        });

        if perm == S2Perm::None {
            return; 
        }

        
        let attr = self.build_descriptor_attrs(perm, mem_type);

        
        let mut ipa = ipa_base & !0xFFF;
        let mut pa = pa_base & !0xFFF;
        let end = ipa_base + size;

        while ipa < end {
            
            if ipa & (OT_ - 1) == 0
                && pa & (OT_ - 1) == 0
                && ipa + OT_ <= end
            {
                self.map_2mb_block(ipa, pa, attr);
                ipa += OT_;
                pa += OT_;
            } else {
                self.map_4kb_page(ipa, pa, attr);
                ipa += BO_;
                pa += BO_;
            }
        }
    }

    
    fn build_descriptor_attrs(&self, perm: S2Perm, mem_type: S2MemType) -> u64 {
        let mut attr: u64 = desc::Agu;

        
        attr |= match perm {
            S2Perm::None => desc::CTO_,
            S2Perm::ReadOnly => desc::BHA_,
            S2Perm::ReadWrite => desc::BHB_,
            S2Perm::ReadExec => desc::BHA_,
            S2Perm::Full => desc::BHB_,
        };

        
        attr |= match perm {
            S2Perm::ReadExec | S2Perm::Full => desc::DFQ_,
            _ => desc::DFP_,
        };

        
        attr |= match mem_type {
            S2MemType::Normal => desc::CJP_ | desc::CWW_,
            S2MemType::Device => desc::CJO_ | desc::CWX_,
        };

        attr
    }

    
    fn map_2mb_block(&mut self, ipa: u64, pa: u64, attr: u64) {
        let axu = ((ipa >> 30) & 0x1FF) as usize;
        let bnf = ((ipa >> 21) & 0x1FF) as usize;

        
        let alv = self.get_or_create_l2(axu);

        
        unsafe {
            let entry = pa & 0x0000_FFFF_FFE0_0000 | attr | desc::Ahg;
            alv.add(bnf).write_volatile(entry);
        }
    }

    
    fn map_4kb_page(&mut self, ipa: u64, pa: u64, attr: u64) {
        let axu = ((ipa >> 30) & 0x1FF) as usize;
        let bnf = ((ipa >> 21) & 0x1FF) as usize;
        let mwe = ((ipa >> 12) & 0x1FF) as usize;

        let alv = self.get_or_create_l2(axu);
        let btw = self.get_or_create_l3(alv, bnf);

        
        unsafe {
            let entry = pa & 0x0000_FFFF_FFFF_F000 | attr | desc::Anb;
            btw.add(mwe).write_volatile(entry);
        }
    }

    
    fn get_or_create_l2(&mut self, axu: usize) -> *mut u64 {
        unsafe {
            let entry = self.l1_table.add(axu).read_volatile();
            if entry & desc::Kw != 0 && entry & desc::Nj != 0 {
                
                (entry & 0x0000_FFFF_FFFF_F000) as *mut u64
            } else {
                
                let alv = Self::fgu();
                self.allocated_pages.push(alv);
                let gxy = (alv as u64 & 0x0000_FFFF_FFFF_F000) | desc::Kw | desc::Nj;
                self.l1_table.add(axu).write_volatile(gxy);
                alv
            }
        }
    }

    
    fn get_or_create_l3(&mut self, alv: *mut u64, bnf: usize) -> *mut u64 {
        unsafe {
            let entry = alv.add(bnf).read_volatile();
            if entry & desc::Kw != 0 && entry & desc::Nj != 0 {
                (entry & 0x0000_FFFF_FFFF_F000) as *mut u64
            } else {
                let btw = Self::fgu();
                self.allocated_pages.push(btw);
                let gxy = (btw as u64 & 0x0000_FFFF_FFFF_F000) | desc::Kw | desc::Nj;
                alv.add(bnf).write_volatile(gxy);
                btw
            }
        }
    }

    
    fn fgu() -> *mut u64 {
        
        let layout = core::alloc::Layout::from_size_align(4096, 4096).unwrap();
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) as *mut u64 };
        if ptr.is_null() {
            panic!("Stage-2 page table allocation failed");
        }
        ptr
    }

    
    pub fn cxy(&self) {
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

    
    pub fn mappings(&self) -> &[Nd] {
        &self.mappings
    }

    
    pub fn qna(&self, ipa: u64) -> Option<&Nd> {
        self.mappings.iter().find(|m| {
            m.perm == S2Perm::None
                && ipa >= m.ipa_base
                && ipa < m.ipa_base + m.size
        })
    }
}
