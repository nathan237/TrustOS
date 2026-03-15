




#[inline(always)]
pub fn aea() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("mrs {}, CNTPCT_EL0", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub fn fjc() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("mrs {}, CNTFRQ_EL0", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


pub fn wjr(bn: u64) {
    unsafe {
        core::arch::asm!(
            "msr CNTP_CVAL_EL0, {}",
            in(reg) bn,
            options(nomem, nostack, preserves_flags)
        );
    }
}


pub fn isr() {
    unsafe {
        core::arch::asm!(
            "msr CNTP_CTL_EL0, {}",
            in(reg) 1u64, 
            options(nomem, nostack, preserves_flags)
        );
    }
}


pub fn rxz() {
    unsafe {
        core::arch::asm!(
            "msr CNTP_CTL_EL0, {}",
            in(reg) 0u64,
            options(nomem, nostack, preserves_flags)
        );
    }
}


pub fn jpd(qb: u64) {
    let cv = aea();
    wjr(cv + qb);
    isr();
}


pub fn zno(ifz: u64) {
    let kx = fjc();
    let qb = (ifz * kx) / 1_000_000;
    jpd(qb);
}
