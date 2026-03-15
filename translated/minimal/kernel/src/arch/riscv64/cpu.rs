




#[inline(always)]
pub fn jln() -> u64 {
    let sp: u64;
    unsafe {
        core::arch::asm!("mv {}, sp", bd(reg) sp, options(nomem, nostack, preserves_flags));
    }
    sp
}


#[inline(always)]
pub fn pae() -> u64 {
    let ghm: u64;
    unsafe {
        core::arch::asm!("mv {}, s0", bd(reg) ghm, options(nomem, nostack, preserves_flags));
    }
    ghm
}


#[inline(always)]
pub fn jat() {
    unsafe {
        core::arch::asm!("fence iorw, iorw", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn hbf() {
    unsafe {
        core::arch::asm!("ebreak", options(nomem, nostack));
    }
}


#[inline(always)]
pub fn cxt() {
    unsafe {
        core::arch::asm!("fence iorw, iorw", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn yqi() {
    unsafe {
        core::arch::asm!("fence.i", options(nomem, nostack, preserves_flags));
    }
}






#[inline(always)]
pub fn vsm() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("csrr {}, sstatus", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub unsafe fn zwy(ap: u64) {
    core::arch::asm!("csrw sstatus, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub fn vsl() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("csrr {}, sie", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub unsafe fn xvs(ap: u64) {
    core::arch::asm!("csrw sie, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub fn zic() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("csrr {}, sip", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub fn zii() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("csrr {}, stvec", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub unsafe fn zxa(ap: u64) {
    core::arch::asm!("csrw stvec, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub fn zib() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("csrr {}, sepc", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub unsafe fn zwv(ap: u64) {
    core::arch::asm!("csrw sepc, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub fn zia() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("csrr {}, scause", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub fn zih() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("csrr {}, stval", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub fn vsk() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("csrr {}, satp", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub unsafe fn xvr(ap: u64) {
    core::arch::asm!("csrw satp, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
    
    core::arch::asm!("sfence.vma", options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub fn zie() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("csrr {}, sscratch", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub unsafe fn zwx(ap: u64) {
    core::arch::asm!("csrw sscratch, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub fn vqy() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("rdcycle {}", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


#[inline(always)]
pub fn vrd() -> u64 {
    let ap: u64;
    unsafe {
        core::arch::asm!("rdtime {}", bd(reg) ap, options(nomem, nostack, preserves_flags));
    }
    ap
}


pub fn ywd() -> u64 {
    
    let aaz: u64;
    unsafe {
        core::arch::asm!("mv {}, tp", bd(reg) aaz, options(nomem, nostack, preserves_flags));
    }
    aaz
}






#[inline(always)]
pub unsafe fn wr(ag: u64) -> u32 {
    let ap: u32;
    core::arch::asm!(
        "lw {val}, 0({addr})",
        ag = in(reg) ag,
        ap = bd(reg) ap,
        options(nostack, preserves_flags)
    );
    ap
}


#[inline(always)]
pub unsafe fn sk(ag: u64, ap: u32) {
    core::arch::asm!(
        "sw {val}, 0({addr})",
        ag = in(reg) ag,
        ap = in(reg) ap,
        options(nostack, preserves_flags)
    );
}


#[inline(always)]
pub unsafe fn fom(ag: u64) -> u8 {
    let ap: u32;
    core::arch::asm!(
        "lbu {val}, 0({addr})",
        ag = in(reg) ag,
        ap = bd(reg) ap,
        options(nostack, preserves_flags)
    );
    ap as u8
}


#[inline(always)]
pub unsafe fn djp(ag: u64, ap: u8) {
    core::arch::asm!(
        "sb {val}, 0({addr})",
        ag = in(reg) ag,
        ap = in(reg) ap,
        options(nostack, preserves_flags)
    );
}






pub mod sstatus {
    pub const Clr: u64 = 1 << 1;   
    pub const Dhs: u64 = 1 << 5;  
    pub const Dhu: u64 = 1 << 8;   
    pub const Dia: u64 = 1 << 18;  
    pub const Dcg: u64 = 1 << 19;  
}


pub mod sie_bits {
    pub const Cmd: u64 = 1 << 1;  
    pub const Cmf: u64 = 1 << 5;  
    pub const Cld: u64 = 1 << 9;  
}


pub mod satp_mode {
    pub const Crr: u64 = 0;
    pub const Cmi: u64 = 8 << 60;  
    pub const Cmj: u64 = 9 << 60;  
    pub const Dib: u64 = 10 << 60; 
}
