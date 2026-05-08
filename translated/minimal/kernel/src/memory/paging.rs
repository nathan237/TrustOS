




use core::sync::atomic::{AtomicU64, Ordering};
use alloc::boxed::Box;
use alloc::vec::Vec;


pub const BO_: usize = 4096;

pub const ET_: usize = 512;


#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PageFlags(u64);

impl PageFlags {
    pub const Bg: u64 = 1 << 0;
    pub const Cg: u64 = 1 << 1;
    pub const Cz: u64 = 1 << 2;
    pub const ALS_: u64 = 1 << 3;
    pub const BDT_: u64 = 1 << 4;
    pub const Wc: u64 = 1 << 5;
    pub const Xm: u64 = 1 << 6;
    pub const EE_: u64 = 1 << 7;
    pub const Zd: u64 = 1 << 8;
    pub const DT_: u64 = 1 << 63;
    
    
    
    
    
    
    
    
    pub const CLU_: u64 = 1 << 7;
    
    
    pub const DVT_: Self = Self(Self::Bg);
    
    
    pub const DVU_: Self = Self(Self::Bg | Self::Cg | Self::DT_);
    
    
    pub const DVV_: Self = Self(Self::Bg | Self::DT_);
    
    
    pub const JZ_: Self = Self(Self::Bg | Self::Cz);
    
    
    pub const FM_: Self = Self(Self::Bg | Self::Cg | Self::Cz | Self::DT_);
    
    
    pub const DDU_: Self = Self(Self::Bg | Self::Cz | Self::DT_);
    
    pub const fn new(flags: u64) -> Self {
        Self(flags)
    }
    
    pub const fn bits(&self) -> u64 {
        self.0
    }
    
    pub fn is_present(&self) -> bool {
        self.0 & Self::Bg != 0
    }
    
    pub fn is_writable(&self) -> bool {
        self.0 & Self::Cg != 0
    }
    
    pub fn is_user(&self) -> bool {
        self.0 & Self::Cz != 0
    }
    
    pub fn is_executable(&self) -> bool {
        self.0 & Self::DT_ == 0
    }
}


#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    
    const AAA_: u64 = 0x000F_FFFF_FFFF_F000;
    
    pub const fn new() -> Self {
        Self(0)
    }
    
    pub fn set(&mut self, phys_addr: u64, flags: PageFlags) {
        self.0 = (phys_addr & Self::AAA_) | flags.bits();
    }
    
    pub fn clear(&mut self) {
        self.0 = 0;
    }
    
    pub fn phys_addr(&self) -> u64 {
        self.0 & Self::AAA_
    }
    
    pub fn flags(&self) -> PageFlags {
        PageFlags(self.0 & !Self::AAA_)
    }
    
    pub fn is_present(&self) -> bool {
        self.0 & PageFlags::Bg != 0
    }
    
    pub fn qnb(&self) -> bool {
        self.0 == 0
    }
}


#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    pub(crate) entries: [PageTableEntry; ET_],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); ET_],
        }
    }
    
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.clear();
        }
    }
}


pub struct AddressSpace {
    
    pml4_phys: u64,
    
    page_tables: Vec<Box<PageTable>>,
    
    hhdm_offset: u64,
}

impl AddressSpace {
    
    pub fn new() -> Option<Self> {
        let bz = crate::memory::hhdm_offset();
        
        
        let mut pml4 = Box::new(PageTable::new());
        pml4.zero();
        
        
        let dcu = &*pml4 as *const PageTable as u64;
        let pml4_phys = dcu.checked_sub(bz)?;
        
        let mut page_tables = Vec::new();
        page_tables.push(pml4);
        
        Some(Self {
            pml4_phys,
            page_tables,
            hhdm_offset: bz,
        })
    }
    
    
    pub fn bnt() -> Option<Self> {
        
        
        
        #[cfg(not(target_arch = "x86_64"))]
        return None;

        #[cfg(target_arch = "x86_64")]
        {
            let mut space = Self::new()?;
            space.map_kernel_space()?;
            Some(space)
        }
    }
    
    
    pub fn cr3(&self) -> u64 {
        self.pml4_phys
    }
    
    
    fn map_kernel_space(&mut self) -> Option<()> {
        
        let current_cr3: u64;
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("mov {}, cr3", out(reg) current_cr3);
        }
        #[cfg(not(target_arch = "x86_64"))]
        { current_cr3 = 0; }
        
        
        let lao = current_cr3 + self.hhdm_offset;
        let hpt = unsafe { 
            &*(lao as *const PageTable) 
        };
        
        
        let noe = self.pml4_phys + self.hhdm_offset;
        let nod = unsafe { 
            &mut *(noe as *mut PageTable) 
        };
        
        
        
        for i in 256..512 {
            if hpt.entries[i].is_present() {
                nod.entries[i] = hpt.entries[i];
            }
        }
        
        Some(())
    }
    
    
    pub fn map_page(&mut self, virt: u64, phys: u64, flags: PageFlags) -> Option<()> {
        
        let lu = ((virt >> 39) & 0x1FF) as usize;
        let jc = ((virt >> 30) & 0x1FF) as usize;
        let iw = ((virt >> 21) & 0x1FF) as usize;
        let mw = ((virt >> 12) & 0x1FF) as usize;
        
        
        let dcu = self.pml4_phys + self.hhdm_offset;
        let pml4 = dcu as *mut PageTable;
        
        
        let xz = unsafe { self.ensure_table_at(&mut (*pml4).entries[lu])? };
        let jt = (xz + self.hhdm_offset) as *mut PageTable;
        
        
        let aae = unsafe { self.ensure_table_at(&mut (*jt).entries[jc])? };
        let js = (aae + self.hhdm_offset) as *mut PageTable;
        
        
        let amj = unsafe { self.ensure_table_at(&mut (*js).entries[iw])? };
        let jd = (amj + self.hhdm_offset) as *mut PageTable;
        
        
        unsafe { (*jd).entries[mw].set(phys, flags); }
        
        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
        }
        
        Some(())
    }
    
    
    fn ensure_table_at(&mut self, entry: &mut PageTableEntry) -> Option<u64> {
        if entry.is_present() {
            Some(entry.phys_addr())
        } else {
            
            let mut ayl = Box::new(PageTable::new());
            ayl.zero();
            
            let bwf = &*ayl as *const PageTable as u64;
            let asj = bwf.checked_sub(self.hhdm_offset)?;
            
            
            entry.set(asj, PageFlags::new(
                PageFlags::Bg | PageFlags::Cg | PageFlags::Cz
            ));
            
            
            self.page_tables.push(ayl);
            
            Some(asj)
        }
    }


    pub fn map_range(&mut self, virt_start: u64, gmy: u64, size: usize, flags: PageFlags) -> Option<()> {
        let acg = (size + BO_ - 1) / BO_;
        
        for i in 0..acg {
            let offset = (i * BO_) as u64;
            self.map_page(virt_start + offset, gmy + offset, flags)?;
        }
        
        Some(())
    }
    
    
    pub fn unmap_page(&mut self, virt: u64) -> Option<()> {
        let entry = Self::ptp(self.pml4_phys, self.hhdm_offset, virt)?;
        entry.clear();
        
        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
        }
        
        Some(())
    }
    
    
    
    #[inline(always)]
    fn hcc(cr3: u64, bz: u64, virt: u64) -> Option<&'static PageTableEntry> {
        let lu = ((virt >> 39) & 0x1FF) as usize;
        let jc = ((virt >> 30) & 0x1FF) as usize;
        let iw   = ((virt >> 21) & 0x1FF) as usize;
        let mw   = ((virt >> 12) & 0x1FF) as usize;

        let pml4 = unsafe { &*((cr3 + bz) as *const PageTable) };
        if !pml4.entries[lu].is_present() { return None; }

        let jt = unsafe { &*((pml4.entries[lu].phys_addr() + bz) as *const PageTable) };
        if !jt.entries[jc].is_present() { return None; }

        let js = unsafe { &*((jt.entries[jc].phys_addr() + bz) as *const PageTable) };
        if !js.entries[iw].is_present() { return None; }

        let jd = unsafe { &*((js.entries[iw].phys_addr() + bz) as *const PageTable) };
        if !jd.entries[mw].is_present() { return None; }

        Some(&jd.entries[mw])
    }

    
    #[inline(always)]
    fn ptp(cr3: u64, bz: u64, virt: u64) -> Option<&'static mut PageTableEntry> {
        let lu = ((virt >> 39) & 0x1FF) as usize;
        let jc = ((virt >> 30) & 0x1FF) as usize;
        let iw   = ((virt >> 21) & 0x1FF) as usize;
        let mw   = ((virt >> 12) & 0x1FF) as usize;

        let pml4 = unsafe { &*((cr3 + bz) as *mut PageTable) };
        if !pml4.entries[lu].is_present() { return None; }

        let jt = unsafe { &*((pml4.entries[lu].phys_addr() + bz) as *mut PageTable) };
        if !jt.entries[jc].is_present() { return None; }

        let js = unsafe { &*((jt.entries[jc].phys_addr() + bz) as *mut PageTable) };
        if !js.entries[iw].is_present() { return None; }

        let jd = unsafe { &mut *((js.entries[iw].phys_addr() + bz) as *mut PageTable) };
        if !jd.entries[mw].is_present() { return None; }

        Some(&mut jd.entries[mw])
    }

    
    
    
    pub fn translate(&self, virt: u64) -> Option<u64> {
        let entry = Self::hcc(self.pml4_phys, self.hhdm_offset, virt)?;
        Some(entry.phys_addr() + (virt & 0xFFF))
    }
    
    
    pub unsafe fn activate(&self) {
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!(
            "mov cr3, {}",
            in(reg) self.pml4_phys,
            options(nostack, preserves_flags)
        );
    }
    
    
    pub fn qlx(&self, virt: u64, eym: PageFlags) -> bool {
        let entry = match Self::hcc(self.pml4_phys, self.hhdm_offset, virt) {
            Some(e) => e,
            None => return false,
        };
        let hwh = entry.flags();
        if eym.is_writable() && !hwh.is_writable() { return false; }
        if eym.is_user() && !hwh.is_user() { return false; }
        true
    }
}

impl AddressSpace {
    
    
    
    
    pub fn release_user_frames(&self) -> usize {
        let bz = self.hhdm_offset;
        let pml4 = unsafe { &*((self.pml4_phys + bz) as *const PageTable) };
        let mut bzz = 0usize;

        for lu in 0..256 {
            if !pml4.entries[lu].is_present() { continue; }
            let jt = unsafe { &*((pml4.entries[lu].phys_addr() + bz) as *const PageTable) };

            for jc in 0..ET_ {
                if !jt.entries[jc].is_present() { continue; }
                
                if jt.entries[jc].flags().0 & PageFlags::EE_ != 0 { continue; }
                let js = unsafe { &*((jt.entries[jc].phys_addr() + bz) as *const PageTable) };

                for iw in 0..ET_ {
                    if !js.entries[iw].is_present() { continue; }
                    
                    if js.entries[iw].flags().0 & PageFlags::EE_ != 0 { continue; }
                    let jd = unsafe { &*((js.entries[iw].phys_addr() + bz) as *const PageTable) };

                    for mw in 0..ET_ {
                        if !jd.entries[mw].is_present() { continue; }
                        let phys = jd.entries[mw].phys_addr();
                        crate::memory::frame::vk(phys);
                        bzz += 1;
                    }
                }
            }
        }
        bzz
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        
        let bzz = self.release_user_frames();
        if bzz > 0 {
            crate::log_debug!("[PAGING] Dropped address space: freed {} user frames ({} KB)",
                bzz, bzz * 4);
        }
        
    }
}


static AZS_: AtomicU64 = AtomicU64::new(0);


pub fn init() {
    
    let cr3: u64;
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        
        core::arch::asm!("mrs {}, TTBR1_EL1", out(reg) cr3, options(nomem, nostack));
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    { cr3 = 0; }
    AZS_.store(cr3, Ordering::SeqCst);
    
    
    fun();
    
    crate::log_debug!("Paging initialized, kernel CR3: {:#x}, NX enabled", cr3);
}


fn fun() {
    #[cfg(target_arch = "x86_64")]
    {
        const IA32_EFER: u32 = 0xC0000080;
        const CLQ_: u64 = 1 << 11;
        
        unsafe {
            
            let eax: u32;
            let edx: u32;
            core::arch::asm!(
                "rdmsr",
                in("ecx") IA32_EFER,
                out("eax") eax,
                out("edx") edx,
            );
            let efer = ((edx as u64) << 32) | (eax as u64);
            
            
            let ipu = efer | CLQ_;
            let low = ipu as u32;
            let high = (ipu >> 32) as u32;
            
            core::arch::asm!(
                "wrmsr",
                in("ecx") IA32_EFER,
                in("eax") low,
                in("edx") high,
            );
        }
    }
}


pub fn kernel_cr3() -> u64 {
    AZS_.load(Ordering::Relaxed)
}


pub fn ux(addr: u64) -> bool {
    addr < 0x0000_8000_0000_0000
}


pub fn msu(addr: u64) -> bool {
    addr >= 0xFFFF_8000_0000_0000
}


pub fn ij(ptr: u64, len: usize, write: bool) -> bool {
    
    if !ux(ptr) {
        return false;
    }
    
    
    let end = ptr.saturating_add(len as u64);
    if !ux(end) {
        return false;
    }
    
    
    if len == 0 {
        return true;
    }
    
    
    
    let bz = crate::memory::hhdm_offset();
    let cr3: u64;
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags)); }
    #[cfg(not(target_arch = "x86_64"))]
    { cr3 = 0; }
    
    let eym = if write {
        PageFlags::new(PageFlags::Bg | PageFlags::Cz | PageFlags::Cg)
    } else {
        PageFlags::new(PageFlags::Bg | PageFlags::Cz)
    };
    
    
    let bvy = ptr & !0xFFF;
    let fuu = (end.saturating_sub(1)) & !0xFFF;
    let mut za = bvy;
    
    loop {
        if !kjj(cr3, bz, za, eym) {
            return false;
        }
        if za >= fuu {
            break;
        }
        za += 0x1000;
    }
    
    true
}


fn kjj(cr3: u64, bz: u64, virt: u64, aov: PageFlags) -> bool {
    let entry = match AddressSpace::hcc(cr3, bz, virt) {
        Some(e) => e,
        None => return false,
    };
    let flags = entry.flags();
    if aov.is_user() && !flags.is_user() { return false; }
    if aov.is_writable() && !flags.is_writable() { return false; }
    true
}


pub struct UserMemoryRegion {
    pub start: u64,
    pub end: u64,
    pub next_alloc: u64,
}

impl UserMemoryRegion {
    
    pub const DIX_: u64 = 0x0000_0000_0040_0000;    
    pub const DIW_: u64 = 0x0000_0000_1000_0000;      
    pub const CH_: u64 = 0x0000_0000_1000_0000;    
    pub const CCL_: u64 = 0x0000_0000_8000_0000;      
    pub const QM_: u64 = 0x0000_7FFF_FFFF_0000;     
    pub const JS_: u64 = 0x0000_0000_0010_0000;    
}



pub fn ilu(virt: u64, phys: u64) -> Result<(), &'static str> {
    use alloc::boxed::Box;
    
    let bz = crate::memory::hhdm_offset();
    
    
    let lu = ((virt >> 39) & 0x1FF) as usize;
    let jc = ((virt >> 30) & 0x1FF) as usize;
    let iw = ((virt >> 21) & 0x1FF) as usize;
    let mw = ((virt >> 12) & 0x1FF) as usize;
    
    
    let cr3: u64;
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
    }
    #[cfg(not(target_arch = "x86_64"))]
    { cr3 = 0; }
    
    
    let pml4 = unsafe { &mut *((cr3 + bz) as *mut PageTable) };
    
    
    let jt = if pml4.entries[lu].is_present() {
        let xz = pml4.entries[lu].phys_addr();
        unsafe { &mut *((xz + bz) as *mut PageTable) }
    } else {
        
        
        crate::serial_println!("[MMIO] Creating PDPT for PML4[{}] (phys={:#x})", lu, phys);
        let njj = Box::new(PageTable::new());
        let ccp = Box::into_raw(njj) as u64;
        let xz = ccp.checked_sub(bz).ok_or("Cannot convert PDPT virt to phys")?;
        
        let flags = PageFlags::new(PageFlags::Bg | PageFlags::Cg);
        pml4.entries[lu].set(xz, flags);
        
        unsafe { &mut *(ccp as *mut PageTable) }
    };
    
    
    let js = if jt.entries[jc].is_present() {
        let aae = jt.entries[jc].phys_addr();
        unsafe { &mut *((aae + bz) as *mut PageTable) }
    } else {
        
        crate::serial_println!("[MMIO] Creating PD for PDPT[{}]", jc);
        let nji = Box::new(PageTable::new());
        let cco = Box::into_raw(nji) as u64;
        let aae = cco.checked_sub(bz).ok_or("Cannot convert PD virt to phys")?;
        
        
        let flags = PageFlags::new(PageFlags::Bg | PageFlags::Cg);
        jt.entries[jc].set(aae, flags);
        
        unsafe { &mut *(cco as *mut PageTable) }
    };
    
    
    let jd = if js.entries[iw].is_present() {
        
        if js.entries[iw].flags().0 & PageFlags::EE_ != 0 {
            
            return Ok(());
        }
        let amj = js.entries[iw].phys_addr();
        unsafe { &mut *((amj + bz) as *mut PageTable) }
    } else {
        
        crate::serial_println!("[MMIO] Creating PT for PD[{}]", iw);
        let njp = Box::new(PageTable::new());
        let iwy = Box::into_raw(njp) as u64;
        let amj = iwy.checked_sub(bz).ok_or("Cannot convert PT virt to phys")?;
        
        
        let flags = PageFlags::new(PageFlags::Bg | PageFlags::Cg);
        js.entries[iw].set(amj, flags);
        
        unsafe { &mut *(iwy as *mut PageTable) }
    };
    
    
    if jd.entries[mw].is_present() {
        
        let lsi = jd.entries[mw].phys_addr();
        if lsi == (phys & !0xFFF) {
            
            return Ok(());
        }
        
        crate::serial_println!("[MMIO] Updating existing mapping at PT[{}]", mw);
    }
    
    
    let nfp = PageFlags::new(
        PageFlags::Bg | 
        PageFlags::Cg | 
        PageFlags::BDT_ | 
        PageFlags::ALS_ |
        PageFlags::DT_
    );
    
    jd.entries[mw].set(phys & !0xFFF, nfp);
    
    Ok(())
}


















const IA32_PAT_MSR: u32 = 0x277;


const ECP_: u8 = 0x06;  
const ECQ_: u8 = 0x04;  
const ECO_: u8 = 0x00;  
const BEU_: u8 = 0x01;  



pub fn oqn() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        
        let pat_lo: u32;
        let pat_hi: u32;
        core::arch::asm!(
            "rdmsr",
            in("ecx") IA32_PAT_MSR,
            out("eax") pat_lo,
            out("edx") pat_hi,
        );
        
        let isc = ((pat_hi as u64) << 32) | (pat_lo as u64);
        
        
        
        let iqe = (isc & !0x0000_0000_0000_FF00) | ((BEU_ as u64) << 8);
        
        let new_lo = iqe as u32;
        let new_hi = (iqe >> 32) as u32;
        
        
        core::arch::asm!(
            "wrmsr",
            in("ecx") IA32_PAT_MSR,
            in("eax") new_lo,
            in("edx") new_hi,
        );
        
        
        core::arch::asm!(
            "mov {tmp}, cr3",
            "mov cr3, {tmp}",
            tmp = out(reg) _,
        );
        
        crate::serial_println!(
            "[PAT] Write-Combining enabled: PAT[1]=WC (was {:#04x}, now {:#04x})",
            (isc >> 8) & 0xFF,
            BEU_
        );
    }
}






pub fn oeu(virt_start: u64, size_bytes: usize) -> Result<usize, &'static str> {
    let bz = crate::memory::hhdm_offset();
    let bnw = (size_bytes + BO_ - 1) / BO_;
    let mut eyi = 0usize;
    
    let cr3: u64;
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3); }
    #[cfg(not(target_arch = "x86_64"))]
    { cr3 = 0; }
    
    for dce in 0..bnw {
        let virt = virt_start + (dce * BO_) as u64;
        
        let lu = ((virt >> 39) & 0x1FF) as usize;
        let jc = ((virt >> 30) & 0x1FF) as usize;
        let iw = ((virt >> 21) & 0x1FF) as usize;
        let mw = ((virt >> 12) & 0x1FF) as usize;
        
        let pml4 = unsafe { &mut *((cr3 + bz) as *mut PageTable) };
        if !pml4.entries[lu].is_present() { continue; }
        
        let xz = pml4.entries[lu].phys_addr();
        let jt = unsafe { &mut *((xz + bz) as *mut PageTable) };
        if !jt.entries[jc].is_present() { continue; }
        
        
        if jt.entries[jc].flags().0 & PageFlags::EE_ != 0 { continue; }
        
        let aae = jt.entries[jc].phys_addr();
        let js = unsafe { &mut *((aae + bz) as *mut PageTable) };
        if !js.entries[iw].is_present() { continue; }
        
        
        if js.entries[iw].flags().0 & PageFlags::EE_ != 0 { continue; }
        
        let amj = js.entries[iw].phys_addr();
        let jd = unsafe { &mut *((amj + bz) as *mut PageTable) };
        if !jd.entries[mw].is_present() { continue; }
        
        
        let phys_addr = jd.entries[mw].phys_addr();
        let nmq = jd.entries[mw].flags().0;
        
        
        let cna = (nmq & !(PageFlags::BDT_ | PageFlags::CLU_))
            | PageFlags::ALS_;  
        
        jd.entries[mw].set(phys_addr, PageFlags::new(cna));
        
        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
        }
        
        eyi += 1;
    }
    
    crate::serial_println!(
        "[PAT] Remapped {} pages as Write-Combining @ {:#x} ({} KB)",
        eyi, virt_start, (eyi * BO_) / 1024
    );
    
    Ok(eyi)
}

