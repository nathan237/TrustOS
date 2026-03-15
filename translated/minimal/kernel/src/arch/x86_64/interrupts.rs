




#[inline(always)]
pub fn aiy() {
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
}


#[inline(always)]
pub fn cwz() {
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack));
    }
}


#[inline(always)]
pub fn gag() -> bool {
    let flags: u64;
    unsafe {
        core::arch::asm!(
            "pushfq",
            "pop {}",
            bd(reg) flags,
            options(nomem, preserves_flags)
        );
    }
    flags & (1 << 9) != 0 
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
