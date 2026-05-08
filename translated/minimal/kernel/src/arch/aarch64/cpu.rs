




#[inline(always)]
pub fn exy() -> u64 {
    let sp: u64;
    unsafe {
        core::arch::asm!("mov {}, sp", out(reg) sp, options(nomem, nostack, preserves_flags));
    }
    sp
}


#[inline(always)]
pub fn iyk() -> u64 {
    let fp: u64;
    unsafe {
        core::arch::asm!("mov {}, x29", out(reg) fp, options(nomem, nostack, preserves_flags));
    }
    fp
}


#[inline(always)]
pub fn qsa() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, DAIF", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub fn erb() {
    unsafe {
        core::arch::asm!("isb", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn breakpoint() {
    unsafe {
        core::arch::asm!("brk #0", options(nomem, nostack));
    }
}


#[inline(always)]
pub fn htz() {
    unsafe {
        core::arch::asm!("dsb sy", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn dsv() {
    unsafe {
        core::arch::asm!("isb", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn qde() {
    unsafe {
        core::arch::asm!("dmb sy", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn qsl() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, MPIDR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub fn qsk() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, MIDR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub fn current_el() -> u8 {
    let el: u64;
    unsafe {
        core::arch::asm!("mrs {}, CurrentEL", out(reg) el, options(nomem, nostack, preserves_flags));
    }
    ((el >> 2) & 0x3) as u8
}


#[inline(always)]
pub unsafe fn gqf() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, SCTLR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}


#[inline(always)]
pub unsafe fn jrq(val: u64) {
    core::arch::asm!("msr SCTLR_EL1, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    dsv();
}


#[inline(always)]
pub unsafe fn qsv() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, TCR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}


#[inline(always)]
pub unsafe fn rdn(val: u64) {
    core::arch::asm!("msr TCR_EL1, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    dsv();
}


#[inline(always)]
pub unsafe fn qsx() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, VBAR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}


#[inline(always)]
pub unsafe fn pvg(val: u64) {
    core::arch::asm!("msr VBAR_EL1, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    dsv();
}


#[inline(always)]
pub unsafe fn iyj() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, ESR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}


#[inline(always)]
pub unsafe fn gqc() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, FAR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}


#[inline(always)]
pub unsafe fn qsb() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, ELR_EL1", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}


#[inline(always)]
pub unsafe fn qsp() -> u64 {
    let val: u64;
    core::arch::asm!("mrs {}, SP_EL0", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}


#[inline(always)]
pub unsafe fn rdg(val: u64) {
    core::arch::asm!("msr SP_EL0, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}






#[inline(always)]
pub unsafe fn kj(addr: u64) -> u32 {
    let val: u32;
    core::arch::asm!(
        "ldr {val:w}, [{addr}]",
        addr = in(reg) addr,
        val = out(reg) val,
        options(nostack, preserves_flags)
    );
    val
}


#[inline(always)]
pub unsafe fn ib(addr: u64, val: u32) {
    core::arch::asm!(
        "str {val:w}, [{addr}]",
        addr = in(reg) addr,
        val = in(reg) val,
        options(nostack, preserves_flags)
    );
}


#[inline(always)]
pub unsafe fn cmx(addr: u64) -> u8 {
    let val: u32;
    core::arch::asm!(
        "ldrb {val:w}, [{addr}]",
        addr = in(reg) addr,
        val = out(reg) val,
        options(nostack, preserves_flags)
    );
    val as u8
}


#[inline(always)]
pub unsafe fn bhy(addr: u64, val: u8) {
    core::arch::asm!(
        "strb {val:w}, [{addr}]",
        addr = in(reg) addr,
        val = in(reg) val as u32,
        options(nostack, preserves_flags)
    );
}
