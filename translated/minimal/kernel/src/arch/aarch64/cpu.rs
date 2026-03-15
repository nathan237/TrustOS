




#[inline(always)]
pub fn jln() -> u64 {
    let sp: u64;
    unsafe {
        core::arch::asm!("mov {}, sp", bd(reg) sp, options(nomem, nostack, preserves_flags));
    }
    sp
}


#[inline(always)]
pub fn pae() -> u64 {
    let ghm: u64;
    unsafe {
        core::arch::asm!("mov {}, x29", bd(reg) ghm, options(nomem, nostack, preserves_flags));
    }
    ghm
}


#[inline(always)]
pub fn zho() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("mrs {}, DAIF", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub fn jat() {
    unsafe {
        core::arch::asm!("isb", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn hbf() {
    unsafe {
        core::arch::asm!("brk #0", options(nomem, nostack));
    }
}


#[inline(always)]
pub fn nny() {
    unsafe {
        core::arch::asm!("dsb sy", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn hpd() {
    unsafe {
        core::arch::asm!("isb", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn ymm() {
    unsafe {
        core::arch::asm!("dmb sy", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn zhz() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("mrs {}, MPIDR_EL1", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub fn zhy() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("mrs {}, MIDR_EL1", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub fn kms() -> u8 {
    let ij: u64;
    unsafe {
        core::arch::asm!("mrs {}, CurrentEL", bd(reg) ij, options(nomem, nostack, preserves_flags));
    }
    ((ij >> 2) & 0x3) as u8
}


#[inline(always)]
pub unsafe fn lxs() -> u64 {
    let ap: u64;
    core::arch::asm!("mrs {}, SCTLR_EL1", bd(reg) ap, options(nomem, nostack, preserves_flags));
    ap
}


#[inline(always)]
pub unsafe fn qab(ap: u64) {
    core::arch::asm!("msr SCTLR_EL1, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
    hpd();
}


#[inline(always)]
pub unsafe fn zij() -> u64 {
    let ap: u64;
    core::arch::asm!("mrs {}, TCR_EL1", bd(reg) ap, options(nomem, nostack, preserves_flags));
    ap
}


#[inline(always)]
pub unsafe fn zxb(ap: u64) {
    core::arch::asm!("msr TCR_EL1, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
    hpd();
}


#[inline(always)]
pub unsafe fn zil() -> u64 {
    let ap: u64;
    core::arch::asm!("mrs {}, VBAR_EL1", bd(reg) ap, options(nomem, nostack, preserves_flags));
    ap
}


#[inline(always)]
pub unsafe fn xvv(ap: u64) {
    core::arch::asm!("msr VBAR_EL1, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
    hpd();
}


#[inline(always)]
pub unsafe fn pad() -> u64 {
    let ap: u64;
    core::arch::asm!("mrs {}, ESR_EL1", bd(reg) ap, options(nomem, nostack, preserves_flags));
    ap
}


#[inline(always)]
pub unsafe fn lxm() -> u64 {
    let ap: u64;
    core::arch::asm!("mrs {}, FAR_EL1", bd(reg) ap, options(nomem, nostack, preserves_flags));
    ap
}


#[inline(always)]
pub unsafe fn zhp() -> u64 {
    let ap: u64;
    core::arch::asm!("mrs {}, ELR_EL1", bd(reg) ap, options(nomem, nostack, preserves_flags));
    ap
}


#[inline(always)]
pub unsafe fn zid() -> u64 {
    let ap: u64;
    core::arch::asm!("mrs {}, SP_EL0", bd(reg) ap, options(nomem, nostack, preserves_flags));
    ap
}


#[inline(always)]
pub unsafe fn zww(ap: u64) {
    core::arch::asm!("msr SP_EL0, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
}






#[inline(always)]
pub unsafe fn wr(ag: u64) -> u32 {
    let ap: u32;
    core::arch::asm!(
        "ldr {val:w}, [{addr}]",
        ag = in(reg) ag,
        ap = bd(reg) ap,
        options(nostack, preserves_flags)
    );
    ap
}


#[inline(always)]
pub unsafe fn sk(ag: u64, ap: u32) {
    core::arch::asm!(
        "str {val:w}, [{addr}]",
        ag = in(reg) ag,
        ap = in(reg) ap,
        options(nostack, preserves_flags)
    );
}


#[inline(always)]
pub unsafe fn fom(ag: u64) -> u8 {
    let ap: u32;
    core::arch::asm!(
        "ldrb {val:w}, [{addr}]",
        ag = in(reg) ag,
        ap = bd(reg) ap,
        options(nostack, preserves_flags)
    );
    ap as u8
}


#[inline(always)]
pub unsafe fn djp(ag: u64, ap: u8) {
    core::arch::asm!(
        "strb {val:w}, [{addr}]",
        ag = in(reg) ag,
        ap = in(reg) ap as u32,
        options(nostack, preserves_flags)
    );
}
