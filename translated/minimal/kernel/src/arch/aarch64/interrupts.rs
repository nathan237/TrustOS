




#[inline(always)]
pub fn aiy() {
    unsafe {
        core::arch::asm!("msr DAIFClr, #0x2", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn cwz() {
    unsafe {
        core::arch::asm!("msr DAIFSet, #0x2", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn gag() -> bool {
    let njh: u64;
    unsafe {
        core::arch::asm!("mrs {}, DAIF", bd(reg) njh, options(nomem, nostack, preserves_flags));
    }
    
    njh & (1 << 7) == 0
}


#[inline(always)]
pub fn cvh<G, Ac>(bb: G) -> Ac
where
    G: FnOnce() -> Ac,
{
    let fbg = gag();
    if fbg {
        cwz();
    }
    let result = bb();
    if fbg {
        aiy();
    }
    result
}




pub fn ttu() {
    super::vectors::init();
    super::gic::init();
    
    crate::log!("aarch64 platform interrupts initialized (GICv2, timer deferred)");
}
