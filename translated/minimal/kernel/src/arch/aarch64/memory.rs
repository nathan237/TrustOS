




use super::cpu;


#[inline(always)]
pub fn ghg(ag: u64) {
    unsafe {
        
        
        let asf = ag >> 12;
        core::arch::asm!(
            "tlbi vae1is, {}",
            "dsb ish",
            "isb",
            in(reg) asf,
            options(nostack, preserves_flags)
        );
    }
}


#[inline(always)]
pub fn ivc() {
    unsafe {
        core::arch::asm!(
            "tlbi vmalle1is",
            "dsb ish",
            "isb",
            options(nomem, nostack, preserves_flags)
        );
    }
}


#[inline(always)]
pub fn dle() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("mrs {}, TTBR0_EL1", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub fn dnj(ap: u64) {
    unsafe {
        core::arch::asm!(
            "msr TTBR0_EL1, {}",
            "isb",
            in(reg) ap,
            options(nostack, preserves_flags)
        );
    }
}


#[inline(always)]
pub fn zhw() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("mrs {}, TTBR1_EL1", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub fn zwt(ap: u64) {
    unsafe {
        core::arch::asm!(
            "msr TTBR1_EL1, {}",
            "isb",
            in(reg) ap,
            options(nostack, preserves_flags)
        );
    }
}


pub fn ype() {
    unsafe {
        let mut dln = cpu::lxs();
        dln |= 1 << 0;  
        dln |= 1 << 2;  
        dln |= 1 << 12; 
        cpu::qab(dln);
    }
}


pub fn yzr() -> bool {
    unsafe {
        let dln = cpu::lxs();
        dln & 1 != 0
    }
}
