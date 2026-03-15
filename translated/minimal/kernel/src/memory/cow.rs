




use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use super::paging::{PageTable, PageFlags, AddressSpace, EG_, BM_};
use super::frame;


pub const SH_: u64 = 1 << 9;


static Axi: Mutex<BTreeMap<u64, u32>> = Mutex::new(BTreeMap::new());


pub fn vtq(ht: u64) {
    let awl = ht & !0xFFF;
    let mut rc = Axi.lock();
    let az = rc.bt(awl).gom(1);
    *az += 1;
}


pub fn vtp(ht: u64) -> bool {
    let awl = ht & !0xFFF;
    let mut rc = Axi.lock();
    if let Some(az) = rc.ds(&awl) {
        *az = az.ao(1);
        if *az == 0 {
            rc.remove(&awl);
            return true;
        }
    }
    false
}


pub fn vto(ht: u64) -> u32 {
    let awl = ht & !0xFFF;
    Axi.lock().get(&awl).hu().unwrap_or(1)
}




pub fn tjf(bha: u64) -> bool {
    #[cfg(not(target_arch = "x86_64"))]
    { let _ = bha; return false; } 
    
    #[cfg(target_arch = "x86_64")]
    {
    let dkk = bha & !0xFFF;
    let hp = super::lr();

    let jm: u64;
    unsafe { core::arch::asm!("mov {}, cr3", bd(reg) jm, options(nostack, preserves_flags)); }

    let wd = ((dkk >> 39) & 0x1FF) as usize;
    let ru = ((dkk >> 30) & 0x1FF) as usize;
    let rn   = ((dkk >> 21) & 0x1FF) as usize;
    let yf   = ((dkk >> 12) & 0x1FF) as usize;

    let wc = unsafe { &*((jm + hp) as *const PageTable) };
    if !wc.ch[wd].xo() { return false; }
    let ss = unsafe { &*((wc.ch[wd].ki() + hp) as *const PageTable) };
    if !ss.ch[ru].xo() { return false; }
    let sr = unsafe { &*((ss.ch[ru].ki() + hp) as *const PageTable) };
    if !sr.ch[rn].xo() { return false; }
    let se = unsafe { &mut *((sr.ch[rn].ki() + hp) as *mut PageTable) };
    if !se.ch[yf].xo() { return false; }

    let flags = se.ch[yf].flags().fs();

    
    if flags & SH_ == 0 {
        return false;
    }

    let jhr = se.ch[yf].ki();
    let rc = vto(jhr);

    if rc > 1 {
        
        let fow = match frame::fcq() {
            Some(ai) => ai,
            None => return false, 
        };
        unsafe {
            core::ptr::copy_nonoverlapping(
                (jhr + hp) as *const u8,
                (fow + hp) as *mut u8,
                BM_,
            );
        }
        vtp(jhr);
        let fot = (flags & !SH_) | PageFlags::Ff;
        se.ch[yf].oj(fow, PageFlags::new(fot));
    } else {
        
        let fot = (flags & !SH_) | PageFlags::Ff;
        se.ch[yf].oj(jhr, PageFlags::new(fot));
    }

    
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) dkk, options(nostack, preserves_flags)); }
    true
    } 
}






pub fn rbt(huf: u64) -> Option<AddressSpace> {
    #[cfg(not(target_arch = "x86_64"))]
    { let _ = huf; return None; } 
    
    #[cfg(target_arch = "x86_64")]
    {
    let hp = super::lr();
    let mut aeh = AddressSpace::dtn()?;
    let otv = unsafe { &*((huf + hp) as *const PageTable) };

    for wd in 0..256 {
        if !otv.ch[wd].xo() { continue; }
        let ss = unsafe {
            &*((otv.ch[wd].ki() + hp) as *const PageTable)
        };

        for ru in 0..EG_ {
            if !ss.ch[ru].xo() { continue; }
            if ss.ch[ru].flags().fs() & PageFlags::DT_ != 0 { continue; }
            let sr = unsafe {
                &*((ss.ch[ru].ki() + hp) as *const PageTable)
            };

            for rn in 0..EG_ {
                if !sr.ch[rn].xo() { continue; }
                if sr.ch[rn].flags().fs() & PageFlags::DT_ != 0 { continue; }
                let jir = unsafe {
                    &mut *((sr.ch[rn].ki() + hp) as *mut PageTable)
                };

                for yf in 0..EG_ {
                    if !jir.ch[yf].xo() { continue; }

                    let ht  = jir.ch[yf].ki();
                    let flags = jir.ch[yf].flags().fs();

                    
                    let ngk = (flags & !PageFlags::Ff) | SH_;

                    
                    jir.ch[yf].oj(ht, PageFlags::new(ngk));

                    let ju = ((wd as u64) << 39)
                             | ((ru as u64) << 30)
                             | ((rn   as u64) << 21)
                             | ((yf   as u64) << 12);

                    
                    aeh.bnl(ju, ht, PageFlags::new(ngk))?;

                    
                    vtq(ht);

                    
                    #[cfg(target_arch = "x86_64")]
                    unsafe {
                        core::arch::asm!("invlpg [{}]", in(reg) ju,
                                         options(nostack, preserves_flags));
                    }
                }
            }
        }
    }

    Some(aeh)
    } 
}
