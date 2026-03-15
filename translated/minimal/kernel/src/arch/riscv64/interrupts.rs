



use super::cpu;


#[inline(always)]
pub fn aiy() {
    unsafe {
        core::arch::asm!("csrsi sstatus, 0x2", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn cwz() {
    unsafe {
        core::arch::asm!("csrci sstatus, 0x2", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn gag() -> bool {
    let sstatus = cpu::vsm();
    sstatus & cpu::sstatus::Clr != 0
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
