




use super::cpu;


#[inline(always)]
pub fn cxy(addr: u64) {
    unsafe {
        
        
        let va = addr >> 12;
        core::arch::asm!(
            "tlbi vae1is, {}",
            "dsb ish",
            "isb",
            in(reg) va,
            options(nostack, preserves_flags)
        );
    }
}


#[inline(always)]
pub fn emz() {
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
pub fn biw() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, TTBR0_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub fn bkc(val: u64) {
    unsafe {
        core::arch::asm!(
            "msr TTBR0_EL1, {}",
            "isb",
            in(reg) val,
            options(nostack, preserves_flags)
        );
    }
}


#[inline(always)]
pub fn qsi() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, TTBR1_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub fn rdd(val: u64) {
    unsafe {
        core::arch::asm!(
            "msr TTBR1_EL1, {}",
            "isb",
            in(reg) val,
            options(nostack, preserves_flags)
        );
    }
}


pub fn qew() {
    unsafe {
        let mut bjb = cpu::gqf();
        bjb |= 1 << 0;  
        bjb |= 1 << 2;  
        bjb |= 1 << 12; 
        cpu::jrq(bjb);
    }
}


pub fn qmq() -> bool {
    unsafe {
        let bjb = cpu::gqf();
        bjb & 1 != 0
    }
}
