




use core::sync::atomic::{AtomicU64, Ordering};
use alloc::boxed::Box;
use alloc::vec::Vec;


pub const BM_: usize = 4096;

pub const EG_: usize = 512;


#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PageFlags(u64);

impl PageFlags {
    pub const Cz: u64 = 1 << 0;
    pub const Ff: u64 = 1 << 1;
    pub const Gq: u64 = 1 << 2;
    pub const AJX_: u64 = 1 << 3;
    pub const BBQ_: u64 = 1 << 4;
    pub const Bbh: u64 = 1 << 5;
    pub const Beb: u64 = 1 << 6;
    pub const DT_: u64 = 1 << 7;
    pub const Bhr: u64 = 1 << 8;
    pub const DL_: u64 = 1 << 63;
    
    
    
    
    
    
    
    
    pub const CIL_: u64 = 1 << 7;
    
    
    pub const DSA_: Self = Self(Self::Cz);
    
    
    pub const DSB_: Self = Self(Self::Cz | Self::Ff | Self::DL_);
    
    
    pub const DSC_: Self = Self(Self::Cz | Self::DL_);
    
    
    pub const JG_: Self = Self(Self::Cz | Self::Gq);
    
    
    pub const EW_: Self = Self(Self::Cz | Self::Ff | Self::Gq | Self::DL_);
    
    
    pub const DAC_: Self = Self(Self::Cz | Self::Gq | Self::DL_);
    
    pub const fn new(flags: u64) -> Self {
        Self(flags)
    }
    
    pub const fn fs(&self) -> u64 {
        self.0
    }
    
    pub fn xo(&self) -> bool {
        self.0 & Self::Cz != 0
    }
    
    pub fn edz(&self) -> bool {
        self.0 & Self::Ff != 0
    }
    
    pub fn jbw(&self) -> bool {
        self.0 & Self::Gq != 0
    }
    
    pub fn clc(&self) -> bool {
        self.0 & Self::DL_ == 0
    }
}


#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    
    const YV_: u64 = 0x000F_FFFF_FFFF_F000;
    
    pub const fn new() -> Self {
        Self(0)
    }
    
    pub fn oj(&mut self, ki: u64, flags: PageFlags) {
        self.0 = (ki & Self::YV_) | flags.fs();
    }
    
    pub fn clear(&mut self) {
        self.0 = 0;
    }
    
    pub fn ki(&self) -> u64 {
        self.0 & Self::YV_
    }
    
    pub fn flags(&self) -> PageFlags {
        PageFlags(self.0 & !Self::YV_)
    }
    
    pub fn xo(&self) -> bool {
        self.0 & PageFlags::Cz != 0
    }
    
    pub fn zad(&self) -> bool {
        self.0 == 0
    }
}


#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    pub(crate) ch: [PageTableEntry; EG_],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            ch: [PageTableEntry::new(); EG_],
        }
    }
    
    pub fn ajs(&mut self) {
        for bt in self.ch.el() {
            bt.clear();
        }
    }
}


pub struct AddressSpace {
    
    cgp: u64,
    
    jik: Vec<Box<PageTable>>,
    
    lr: u64,
}

impl AddressSpace {
    
    pub fn new() -> Option<Self> {
        let hp = crate::memory::lr();
        
        
        let mut wc = Box::new(PageTable::new());
        wc.ajs();
        
        
        let dum = &*wc as *const PageTable as u64;
        let cgp = dum.enj(hp)?;
        
        let mut jik = Vec::new();
        jik.push(wc);
        
        Some(Self {
            cgp,
            jik,
            lr: hp,
        })
    }
    
    
    pub fn dtn() -> Option<Self> {
        
        
        
        #[cfg(not(target_arch = "x86_64"))]
        return None;

        #[cfg(target_arch = "x86_64")]
        {
            let mut atm = Self::new()?;
            atm.ujr()?;
            Some(atm)
        }
    }
    
    
    pub fn jm(&self) -> u64 {
        self.cgp
    }
    
    
    fn ujr(&mut self) -> Option<()> {
        
        let kmr: u64;
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("mov {}, cr3", bd(reg) kmr);
        }
        #[cfg(not(target_arch = "x86_64"))]
        { kmr = 0; }
        
        
        let rsa = kmr + self.lr;
        let nii = unsafe { 
            &*(rsa as *const PageTable) 
        };
        
        
        let uzp = self.cgp + self.lr;
        let uzo = unsafe { 
            &mut *(uzp as *mut PageTable) 
        };
        
        
        
        for a in 256..512 {
            if nii.ch[a].xo() {
                uzo.ch[a] = nii.ch[a];
            }
        }
        
        Some(())
    }
    
    
    pub fn bnl(&mut self, ju: u64, ht: u64, flags: PageFlags) -> Option<()> {
        
        let wd = ((ju >> 39) & 0x1FF) as usize;
        let ru = ((ju >> 30) & 0x1FF) as usize;
        let rn = ((ju >> 21) & 0x1FF) as usize;
        let yf = ((ju >> 12) & 0x1FF) as usize;
        
        
        let dum = self.cgp + self.lr;
        let wc = dum as *mut PageTable;
        
        
        let auu = unsafe { self.dqo(&mut (*wc).ch[wd])? };
        let ss = (auu + self.lr) as *mut PageTable;
        
        
        let ayi = unsafe { self.dqo(&mut (*ss).ch[ru])? };
        let sr = (ayi + self.lr) as *mut PageTable;
        
        
        let bwe = unsafe { self.dqo(&mut (*sr).ch[rn])? };
        let se = (bwe + self.lr) as *mut PageTable;
        
        
        unsafe { (*se).ch[yf].oj(ht, flags); }
        
        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) ju, options(nostack, preserves_flags));
        }
        
        Some(())
    }
    
    
    fn dqo(&mut self, bt: &mut PageTableEntry) -> Option<u64> {
        if bt.xo() {
            Some(bt.ki())
        } else {
            
            let mut css = Box::new(PageTable::new());
            css.ajs();
            
            let ejk = &*css as *const PageTable as u64;
            let cig = ejk.enj(self.lr)?;
            
            
            bt.oj(cig, PageFlags::new(
                PageFlags::Cz | PageFlags::Ff | PageFlags::Gq
            ));
            
            
            self.jik.push(css);
            
            Some(cig)
        }
    }


    pub fn jew(&mut self, jvq: u64, ltv: u64, aw: usize, flags: PageFlags) -> Option<()> {
        let bcd = (aw + BM_ - 1) / BM_;
        
        for a in 0..bcd {
            let l = (a * BM_) as u64;
            self.bnl(jvq + l, ltv + l, flags)?;
        }
        
        Some(())
    }
    
    
    pub fn xoj(&mut self, ju: u64) -> Option<()> {
        let wd = ((ju >> 39) & 0x1FF) as usize;
        let ru = ((ju >> 30) & 0x1FF) as usize;
        let rn = ((ju >> 21) & 0x1FF) as usize;
        let yf = ((ju >> 12) & 0x1FF) as usize;
        
        let dum = self.cgp + self.lr;
        let wc = unsafe { &mut *(dum as *mut PageTable) };
        
        if !wc.ch[wd].xo() {
            return None;
        }
        
        let dan = wc.ch[wd].ki() + self.lr;
        let ss = unsafe { &mut *(dan as *mut PageTable) };
        
        if !ss.ch[ru].xo() {
            return None;
        }
        
        let dam = ss.ch[ru].ki() + self.lr;
        let sr = unsafe { &mut *(dam as *mut PageTable) };
        
        if !sr.ch[rn].xo() {
            return None;
        }
        
        let gqa = sr.ch[rn].ki() + self.lr;
        let se = unsafe { &mut *(gqa as *mut PageTable) };
        
        se.ch[yf].clear();
        
        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) ju, options(nostack, preserves_flags));
        }
        
        Some(())
    }
    
    
    
    
    pub fn dmr(&self, ju: u64) -> Option<u64> {
        let wd = ((ju >> 39) & 0x1FF) as usize;
        let ru = ((ju >> 30) & 0x1FF) as usize;
        let rn = ((ju >> 21) & 0x1FF) as usize;
        let yf = ((ju >> 12) & 0x1FF) as usize;
        let huc = ju & 0xFFF;
        
        let wc = unsafe { &*((self.cgp + self.lr) as *const PageTable) };
        if !wc.ch[wd].xo() { return None; }
        
        let ss = unsafe { &*((wc.ch[wd].ki() + self.lr) as *const PageTable) };
        if !ss.ch[ru].xo() { return None; }
        
        let sr = unsafe { &*((ss.ch[ru].ki() + self.lr) as *const PageTable) };
        if !sr.ch[rn].xo() { return None; }
        
        let se = unsafe { &*((sr.ch[rn].ki() + self.lr) as *const PageTable) };
        if !se.ch[yf].xo() { return None; }
        
        Some(se.ch[yf].ki() + huc)
    }
    
    
    pub unsafe fn fci(&self) {
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!(
            "mov cr3, {}",
            in(reg) self.cgp,
            options(nostack, preserves_flags)
        );
    }
    
    
    pub fn yyu(&self, ju: u64, jme: PageFlags) -> bool {
        let wd = ((ju >> 39) & 0x1FF) as usize;
        let ru = ((ju >> 30) & 0x1FF) as usize;
        let rn = ((ju >> 21) & 0x1FF) as usize;
        let yf = ((ju >> 12) & 0x1FF) as usize;
        
        let dum = self.cgp + self.lr;
        let wc = unsafe { &*(dum as *const PageTable) };
        
        if !wc.ch[wd].xo() {
            return false;
        }
        
        let dan = wc.ch[wd].ki() + self.lr;
        let ss = unsafe { &*(dan as *const PageTable) };
        
        if !ss.ch[ru].xo() {
            return false;
        }
        
        let dam = ss.ch[ru].ki() + self.lr;
        let sr = unsafe { &*(dam as *const PageTable) };
        
        if !sr.ch[rn].xo() {
            return false;
        }
        
        let gqa = sr.ch[rn].ki() + self.lr;
        let se = unsafe { &*(gqa as *const PageTable) };
        
        if !se.ch[yf].xo() {
            return false;
        }
        
        let nqq = se.ch[yf].flags();
        
        
        if jme.edz() && !nqq.edz() {
            return false;
        }
        if jme.jbw() && !nqq.jbw() {
            return false;
        }
        
        true
    }
}

impl AddressSpace {
    
    
    
    
    pub fn vuo(&self) -> usize {
        let hp = self.lr;
        let wc = unsafe { &*((self.cgp + hp) as *const PageTable) };
        let mut equ = 0usize;

        for wd in 0..256 {
            if !wc.ch[wd].xo() { continue; }
            let ss = unsafe { &*((wc.ch[wd].ki() + hp) as *const PageTable) };

            for ru in 0..EG_ {
                if !ss.ch[ru].xo() { continue; }
                
                if ss.ch[ru].flags().0 & PageFlags::DT_ != 0 { continue; }
                let sr = unsafe { &*((ss.ch[ru].ki() + hp) as *const PageTable) };

                for rn in 0..EG_ {
                    if !sr.ch[rn].xo() { continue; }
                    
                    if sr.ch[rn].flags().0 & PageFlags::DT_ != 0 { continue; }
                    let se = unsafe { &*((sr.ch[rn].ki() + hp) as *const PageTable) };

                    for yf in 0..EG_ {
                        if !se.ch[yf].xo() { continue; }
                        let ht = se.ch[yf].ki();
                        crate::memory::frame::apt(ht);
                        equ += 1;
                    }
                }
            }
        }
        equ
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        
        let equ = self.vuo();
        if equ > 0 {
            crate::log_debug!("[PAGING] Dropped address space: freed {} user frames ({} KB)",
                equ, equ * 4);
        }
        
    }
}


static AXP_: AtomicU64 = AtomicU64::new(0);


pub fn init() {
    
    let jm: u64;
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("mov {}, cr3", bd(reg) jm);
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        
        core::arch::asm!("mrs {}, TTBR1_EL1", bd(reg) jm, options(nomem, nostack));
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    { jm = 0; }
    AXP_.store(jm, Ordering::SeqCst);
    
    
    ktf();
    
    crate::log_debug!("Paging initialized, kernel CR3: {:#x}, NX enabled", jm);
}


fn ktf() {
    #[cfg(target_arch = "x86_64")]
    {
        const CN_: u32 = 0xC0000080;
        const CIH_: u64 = 1 << 11;
        
        unsafe {
            
            let eax: u32;
            let edx: u32;
            core::arch::asm!(
                "rdmsr",
                in("ecx") CN_,
                bd("eax") eax,
                bd("edx") edx,
            );
            let efer = ((edx as u64) << 32) | (eax as u64);
            
            
            let opp = efer | CIH_;
            let ail = opp as u32;
            let afq = (opp >> 32) as u32;
            
            core::arch::asm!(
                "wrmsr",
                in("ecx") CN_,
                in("eax") ail,
                in("edx") afq,
            );
        }
    }
}


pub fn ade() -> u64 {
    AXP_.load(Ordering::Relaxed)
}


pub fn aov(ag: u64) -> bool {
    ag < 0x0000_8000_0000_0000
}


pub fn txu(ag: u64) -> bool {
    ag >= 0xFFFF_8000_0000_0000
}


pub fn sw(ptr: u64, len: usize, write: bool) -> bool {
    
    if !aov(ptr) {
        return false;
    }
    
    
    let ci = ptr.akq(len as u64);
    if !aov(ci) {
        return false;
    }
    
    
    if len == 0 {
        return true;
    }
    
    
    
    let hp = crate::memory::lr();
    let jm: u64;
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("mov {}, cr3", bd(reg) jm, options(nostack, preserves_flags)); }
    #[cfg(not(target_arch = "x86_64"))]
    { jm = 0; }
    
    let jme = if write {
        PageFlags::new(PageFlags::Cz | PageFlags::Gq | PageFlags::Ff)
    } else {
        PageFlags::new(PageFlags::Cz | PageFlags::Gq)
    };
    
    
    let eiz = ptr & !0xFFF;
    let ktm = (ci.ao(1)) & !0xFFF;
    let mut awl = eiz;
    
    loop {
        if !qzi(jm, hp, awl, jme) {
            return false;
        }
        if awl >= ktm {
            break;
        }
        awl += 0x1000;
    }
    
    true
}


fn qzi(jm: u64, hp: u64, ju: u64, cbj: PageFlags) -> bool {
    let wd = ((ju >> 39) & 0x1FF) as usize;
    let ru = ((ju >> 30) & 0x1FF) as usize;
    let rn   = ((ju >> 21) & 0x1FF) as usize;
    let yf   = ((ju >> 12) & 0x1FF) as usize;
    
    let wc = unsafe { &*((jm + hp) as *const PageTable) };
    if !wc.ch[wd].xo() { return false; }
    
    let ss = unsafe { &*((wc.ch[wd].ki() + hp) as *const PageTable) };
    if !ss.ch[ru].xo() { return false; }
    
    let sr = unsafe { &*((ss.ch[ru].ki() + hp) as *const PageTable) };
    if !sr.ch[rn].xo() { return false; }
    
    let se = unsafe { &*((sr.ch[rn].ki() + hp) as *const PageTable) };
    if !se.ch[yf].xo() { return false; }
    
    let flags = se.ch[yf].flags();
    if cbj.jbw() && !flags.jbw() { return false; }
    if cbj.edz() && !flags.edz() { return false; }
    
    true
}


pub struct UserMemoryRegion {
    pub ay: u64,
    pub ci: u64,
    pub zdo: u64,
}

impl UserMemoryRegion {
    
    pub const DFE_: u64 = 0x0000_0000_0040_0000;    
    pub const DFD_: u64 = 0x0000_0000_1000_0000;      
    pub const CF_: u64 = 0x0000_0000_1000_0000;    
    pub const BZA_: u64 = 0x0000_0000_8000_0000;      
    pub const PP_: u64 = 0x0000_7FFF_FFFF_0000;     
    pub const IZ_: u64 = 0x0000_0000_0010_0000;    
}



pub fn oky(ju: u64, ht: u64) -> Result<(), &'static str> {
    use alloc::boxed::Box;
    
    let hp = crate::memory::lr();
    
    
    let wd = ((ju >> 39) & 0x1FF) as usize;
    let ru = ((ju >> 30) & 0x1FF) as usize;
    let rn = ((ju >> 21) & 0x1FF) as usize;
    let yf = ((ju >> 12) & 0x1FF) as usize;
    
    
    let jm: u64;
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("mov {}, cr3", bd(reg) jm);
    }
    #[cfg(not(target_arch = "x86_64"))]
    { jm = 0; }
    
    
    let wc = unsafe { &mut *((jm + hp) as *mut PageTable) };
    
    
    let ss = if wc.ch[wd].xo() {
        let auu = wc.ch[wd].ki();
        unsafe { &mut *((auu + hp) as *mut PageTable) }
    } else {
        
        
        crate::serial_println!("[MMIO] Creating PDPT for PML4[{}] (phys={:#x})", wd, ht);
        let uti = Box::new(PageTable::new());
        let dan = Box::lfi(uti) as u64;
        let auu = dan.enj(hp).ok_or("Cannot convert PDPT virt to phys")?;
        
        let flags = PageFlags::new(PageFlags::Cz | PageFlags::Ff);
        wc.ch[wd].oj(auu, flags);
        
        unsafe { &mut *(dan as *mut PageTable) }
    };
    
    
    let sr = if ss.ch[ru].xo() {
        let ayi = ss.ch[ru].ki();
        unsafe { &mut *((ayi + hp) as *mut PageTable) }
    } else {
        
        crate::serial_println!("[MMIO] Creating PD for PDPT[{}]", ru);
        let uth = Box::new(PageTable::new());
        let dam = Box::lfi(uth) as u64;
        let ayi = dam.enj(hp).ok_or("Cannot convert PD virt to phys")?;
        
        
        let flags = PageFlags::new(PageFlags::Cz | PageFlags::Ff);
        ss.ch[ru].oj(ayi, flags);
        
        unsafe { &mut *(dam as *mut PageTable) }
    };
    
    
    let se = if sr.ch[rn].xo() {
        
        if sr.ch[rn].flags().0 & PageFlags::DT_ != 0 {
            
            return Ok(());
        }
        let bwe = sr.ch[rn].ki();
        unsafe { &mut *((bwe + hp) as *mut PageTable) }
    } else {
        
        crate::serial_println!("[MMIO] Creating PT for PD[{}]", rn);
        let utp = Box::new(PageTable::new());
        let gqa = Box::lfi(utp) as u64;
        let bwe = gqa.enj(hp).ok_or("Cannot convert PT virt to phys")?;
        
        
        let flags = PageFlags::new(PageFlags::Cz | PageFlags::Ff);
        sr.ch[rn].oj(bwe, flags);
        
        unsafe { &mut *(gqa as *mut PageTable) }
    };
    
    
    if se.ch[yf].xo() {
        
        let sox = se.ch[yf].ki();
        if sox == (ht & !0xFFF) {
            
            return Ok(());
        }
        
        crate::serial_println!("[MMIO] Updating existing mapping at PT[{}]", yf);
    }
    
    
    let uow = PageFlags::new(
        PageFlags::Cz | 
        PageFlags::Ff | 
        PageFlags::BBQ_ | 
        PageFlags::AJX_ |
        PageFlags::DL_
    );
    
    se.ch[yf].oj(ht & !0xFFF, uow);
    
    Ok(())
}


















const AWL_: u32 = 0x277;


const DYY_: u8 = 0x06;  
const DYZ_: u8 = 0x04;  
const DYX_: u8 = 0x00;  
const BCR_: u8 = 0x01;  



pub fn wlh() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        
        let out: u32;
        let ous: u32;
        core::arch::asm!(
            "rdmsr",
            in("ecx") AWL_,
            bd("eax") out,
            bd("edx") ous,
        );
        
        let osi = ((ous as u64) << 32) | (out as u64);
        
        
        
        let oqa = (osi & !0x0000_0000_0000_FF00) | ((BCR_ as u64) << 8);
        
        let lnx = oqa as u32;
        let usu = (oqa >> 32) as u32;
        
        
        core::arch::asm!(
            "wrmsr",
            in("ecx") AWL_,
            in("eax") lnx,
            in("edx") usu,
        );
        
        
        core::arch::asm!(
            "mov {tmp}, cr3",
            "mov cr3, {tmp}",
            gup = bd(reg) _,
        );
        
        crate::serial_println!(
            "[PAT] Write-Combining enabled: PAT[1]=WC (was {:#04x}, now {:#04x})",
            (osi >> 8) & 0xFF,
            BCR_
        );
    }
}






pub fn vus(jvq: u64, afz: usize) -> Result<usize, &'static str> {
    let hp = crate::memory::lr();
    let dtt = (afz + BM_ - 1) / BM_;
    let mut jlz = 0usize;
    
    let jm: u64;
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("mov {}, cr3", bd(reg) jm); }
    #[cfg(not(target_arch = "x86_64"))]
    { jm = 0; }
    
    for gor in 0..dtt {
        let ju = jvq + (gor * BM_) as u64;
        
        let wd = ((ju >> 39) & 0x1FF) as usize;
        let ru = ((ju >> 30) & 0x1FF) as usize;
        let rn = ((ju >> 21) & 0x1FF) as usize;
        let yf = ((ju >> 12) & 0x1FF) as usize;
        
        let wc = unsafe { &mut *((jm + hp) as *mut PageTable) };
        if !wc.ch[wd].xo() { continue; }
        
        let auu = wc.ch[wd].ki();
        let ss = unsafe { &mut *((auu + hp) as *mut PageTable) };
        if !ss.ch[ru].xo() { continue; }
        
        
        if ss.ch[ru].flags().0 & PageFlags::DT_ != 0 { continue; }
        
        let ayi = ss.ch[ru].ki();
        let sr = unsafe { &mut *((ayi + hp) as *mut PageTable) };
        if !sr.ch[rn].xo() { continue; }
        
        
        if sr.ch[rn].flags().0 & PageFlags::DT_ != 0 { continue; }
        
        let bwe = sr.ch[rn].ki();
        let se = unsafe { &mut *((bwe + hp) as *mut PageTable) };
        if !se.ch[yf].xo() { continue; }
        
        
        let ki = se.ch[yf].ki();
        let uxp = se.ch[yf].flags().0;
        
        
        let fot = (uxp & !(PageFlags::BBQ_ | PageFlags::CIL_))
            | PageFlags::AJX_;  
        
        se.ch[yf].oj(ki, PageFlags::new(fot));
        
        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) ju, options(nostack, preserves_flags));
        }
        
        jlz += 1;
    }
    
    crate::serial_println!(
        "[PAT] Remapped {} pages as Write-Combining @ {:#x} ({} KB)",
        jlz, jvq, (jlz * BM_) / 1024
    );
    
    Ok(jlz)
}

