



use super::cpu;


#[inline(always)]
pub fn enable() {
    unsafe {
        core::arch::asm!("csrsi sstatus, 0x2", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn bbc() {
    unsafe {
        core::arch::asm!("csrci sstatus, 0x2", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn ctq() -> bool {
    let sstatus = cpu::odd();
    sstatus & cpu::sstatus::Apg != 0
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
