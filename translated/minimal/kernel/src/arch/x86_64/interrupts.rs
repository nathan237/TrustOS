




#[inline(always)]
pub fn enable() {
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
}


#[inline(always)]
pub fn bbc() {
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack));
    }
}


#[inline(always)]
pub fn ctq() -> bool {
    let flags: u64;
    unsafe {
        core::arch::asm!(
            "pushfq",
            "pop {}",
            out(reg) flags,
            options(nomem, preserves_flags)
        );
    }
    flags & (1 << 9) != 0 
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
