




#[inline(always)]
pub fn timestamp() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, CNTPCT_EL0", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub fn frequency() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, CNTFRQ_EL0", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


pub fn opn(value: u64) {
    unsafe {
        core::arch::asm!(
            "msr CNTP_CVAL_EL0, {}",
            in(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }
}


pub fn eli() {
    unsafe {
        core::arch::asm!(
            "msr CNTP_CTL_EL0, {}",
            in(reg) 1u64, 
            options(nomem, nostack, preserves_flags)
        );
    }
}


pub fn lfa() {
    unsafe {
        core::arch::asm!(
            "msr CNTP_CTL_EL0, {}",
            in(reg) 0u64,
            options(nomem, nostack, preserves_flags)
        );
    }
}


pub fn fah(gx: u64) {
    let current = timestamp();
    opn(current + gx);
    eli();
}


pub fn qwk(us: u64) {
    let freq = frequency();
    let gx = (us * freq) / 1_000_000;
    fah(gx);
}
