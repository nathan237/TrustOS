




#[inline(always)]
pub fn enable() {
    unsafe {
        core::arch::asm!("msr DAIFClr, #0x2", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn bbc() {
    unsafe {
        core::arch::asm!("msr DAIFSet, #0x2", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn ctq() -> bool {
    let daif: u64;
    unsafe {
        core::arch::asm!("mrs {}, DAIF", out(reg) daif, options(nomem, nostack, preserves_flags));
    }
    
    daif & (1 << 7) == 0
}


#[inline(always)]
pub fn bag<F, U>(f: F) -> U
where
    F: FnOnce() -> U,
{
    let cfc = ctq();
    if cfc {
        bbc();
    }
    let result = f();
    if cfc {
        enable();
    }
    result
}




pub fn mpk() {
    super::vectors::init();
    super::gic::init();
    
    crate::log!("aarch64 platform interrupts initialized (GICv2, timer deferred)");
}
