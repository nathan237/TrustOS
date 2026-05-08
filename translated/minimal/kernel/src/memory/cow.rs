




use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use super::paging::{PageTable, PageFlags, AddressSpace, ET_, BO_};
use super::frame;


pub const TK_: u64 = 1 << 9;


static Un: Mutex<BTreeMap<u64, u32>> = Mutex::new(BTreeMap::new());


pub fn ody(phys: u64) {
    let za = phys & !0xFFF;
    let mut rc = Un.lock();
    let count = rc.entry(za).or_insert(1);
    *count += 1;
}


pub fn odx(phys: u64) -> bool {
    let za = phys & !0xFFF;
    let mut rc = Un.lock();
    if let Some(count) = rc.get_mut(&za) {
        *count = count.saturating_sub(1);
        if *count == 0 {
            rc.remove(&za);
            return true;
        }
    }
    false
}


pub fn odw(phys: u64) -> u32 {
    let za = phys & !0xFFF;
    Un.lock().get(&za).copied().unwrap_or(1)
}




pub fn mhn(aff: u64) -> bool {
    #[cfg(not(target_arch = "x86_64"))]
    { let _ = aff; return false; } 
    
    #[cfg(target_arch = "x86_64")]
    {
    let page_addr = aff & !0xFFF;
    let bz = super::hhdm_offset();

    let cr3: u64;
    unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags)); }

    let lu = ((page_addr >> 39) & 0x1FF) as usize;
    let jc = ((page_addr >> 30) & 0x1FF) as usize;
    let iw   = ((page_addr >> 21) & 0x1FF) as usize;
    let mw   = ((page_addr >> 12) & 0x1FF) as usize;

    let pml4 = unsafe { &*((cr3 + bz) as *const PageTable) };
    if !pml4.entries[lu].is_present() { return false; }
    let jt = unsafe { &*((pml4.entries[lu].phys_addr() + bz) as *const PageTable) };
    if !jt.entries[jc].is_present() { return false; }
    let js = unsafe { &*((jt.entries[jc].phys_addr() + bz) as *const PageTable) };
    if !js.entries[iw].is_present() { return false; }
    let jd = unsafe { &mut *((js.entries[iw].phys_addr() + bz) as *mut PageTable) };
    if !jd.entries[mw].is_present() { return false; }

    let flags = jd.entries[mw].flags().bits();

    
    if flags & TK_ == 0 {
        return false;
    }

    let evq = jd.entries[mw].phys_addr();
    let rc = odw(evq);

    if rc > 1 {
        
        let cnd = match frame::cfv() {
            Some(aa) => aa,
            None => return false, 
        };
        unsafe {
            core::ptr::copy_nonoverlapping(
                (evq + bz) as *const u8,
                (cnd + bz) as *mut u8,
                BO_,
            );
        }
        odx(evq);
        let cna = (flags & !TK_) | PageFlags::Cg;
        jd.entries[mw].set(cnd, PageFlags::new(cna));
    } else {
        
        let cna = (flags & !TK_) | PageFlags::Cg;
        jd.entries[mw].set(evq, PageFlags::new(cna));
    }

    
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) page_addr, options(nostack, preserves_flags)); }
    true
    } 
}






pub fn klf(parent_cr3: u64) -> Option<AddressSpace> {
    #[cfg(not(target_arch = "x86_64"))]
    { let _ = parent_cr3; return None; } 
    
    #[cfg(target_arch = "x86_64")]
    {
    let bz = super::hhdm_offset();
    let mut pd = AddressSpace::bnt()?;
    let itl = unsafe { &*((parent_cr3 + bz) as *const PageTable) };

    for lu in 0..256 {
        if !itl.entries[lu].is_present() { continue; }
        let jt = unsafe {
            &*((itl.entries[lu].phys_addr() + bz) as *const PageTable)
        };

        for jc in 0..ET_ {
            if !jt.entries[jc].is_present() { continue; }
            if jt.entries[jc].flags().bits() & PageFlags::EE_ != 0 { continue; }
            let js = unsafe {
                &*((jt.entries[jc].phys_addr() + bz) as *const PageTable)
            };

            for iw in 0..ET_ {
                if !js.entries[iw].is_present() { continue; }
                if js.entries[iw].flags().bits() & PageFlags::EE_ != 0 { continue; }
                let ewf = unsafe {
                    &mut *((js.entries[iw].phys_addr() + bz) as *mut PageTable)
                };

                for mw in 0..ET_ {
                    if !ewf.entries[mw].is_present() { continue; }

                    let phys  = ewf.entries[mw].phys_addr();
                    let flags = ewf.entries[mw].flags().bits();

                    
                    let hod = (flags & !PageFlags::Cg) | TK_;

                    
                    ewf.entries[mw].set(phys, PageFlags::new(hod));

                    let virt = ((lu as u64) << 39)
                             | ((jc as u64) << 30)
                             | ((iw   as u64) << 21)
                             | ((mw   as u64) << 12);

                    
                    pd.map_page(virt, phys, PageFlags::new(hod))?;

                    
                    ody(phys);
                }
            }
        }
    }

    
    
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));
        core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack, preserves_flags));
    }

    Some(pd)
    } 
}
