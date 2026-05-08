




#[inline(always)]
pub fn exy() -> u64 {
    let sp: u64;
    unsafe {
        core::arch::asm!("mv {}, sp", out(reg) sp, options(nomem, nostack, preserves_flags));
    }
    sp
}


#[inline(always)]
pub fn iyk() -> u64 {
    let fp: u64;
    unsafe {
        core::arch::asm!("mv {}, s0", out(reg) fp, options(nomem, nostack, preserves_flags));
    }
    fp
}


#[inline(always)]
pub fn erb() {
    unsafe {
        core::arch::asm!("fence iorw, iorw", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn breakpoint() {
    unsafe {
        core::arch::asm!("ebreak", options(nomem, nostack));
    }
}


#[inline(always)]
pub fn bbq() {
    unsafe {
        core::arch::asm!("fence iorw, iorw", options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn qfm() {
    unsafe {
        core::arch::asm!("fence.i", options(nomem, nostack, preserves_flags));
    }
}






#[inline(always)]
pub fn odd() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, sstatus", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub unsafe fn rdj(val: u64) {
    core::arch::asm!("csrw sstatus, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub fn odc() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, sie", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub unsafe fn pvd(val: u64) {
    core::arch::asm!("csrw sie, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub fn qso() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, sip", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub fn qsu() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, stvec", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub unsafe fn rdl(val: u64) {
    core::arch::asm!("csrw stvec, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub fn qsn() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, sepc", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub unsafe fn rdf(val: u64) {
    core::arch::asm!("csrw sepc, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub fn qsm() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub fn qst() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, stval", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub fn odb() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, satp", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub unsafe fn pvc(val: u64) {
    core::arch::asm!("csrw satp, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    
    core::arch::asm!("sfence.vma", options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub fn qsq() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("csrr {}, sscratch", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub unsafe fn rdh(val: u64) {
    core::arch::asm!("csrw sscratch, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub fn ocd() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("rdcycle {}", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


#[inline(always)]
pub fn och() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("rdtime {}", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}


pub fn qkh() -> u64 {
    
    let tp: u64;
    unsafe {
        core::arch::asm!("mv {}, tp", out(reg) tp, options(nomem, nostack, preserves_flags));
    }
    tp
}






#[inline(always)]
pub unsafe fn kj(addr: u64) -> u32 {
    let val: u32;
    core::arch::asm!(
        "lw {val}, 0({addr})",
        addr = in(reg) addr,
        val = out(reg) val,
        options(nostack, preserves_flags)
    );
    val
}


#[inline(always)]
pub unsafe fn ib(addr: u64, val: u32) {
    core::arch::asm!(
        "sw {val}, 0({addr})",
        addr = in(reg) addr,
        val = in(reg) val,
        options(nostack, preserves_flags)
    );
}


#[inline(always)]
pub unsafe fn cmx(addr: u64) -> u8 {
    let val: u32;
    core::arch::asm!(
        "lbu {val}, 0({addr})",
        addr = in(reg) addr,
        val = out(reg) val,
        options(nostack, preserves_flags)
    );
    val as u8
}


#[inline(always)]
pub unsafe fn bhy(addr: u64, val: u8) {
    core::arch::asm!(
        "sb {val}, 0({addr})",
        addr = in(reg) addr,
        val = in(reg) val,
        options(nostack, preserves_flags)
    );
}






pub mod sstatus {
    pub const Apg: u64 = 1 << 1;   
    pub const Bcu: u64 = 1 << 5;  
    pub const Bcw: u64 = 1 << 8;   
    pub const Bdc: u64 = 1 << 18;  
    pub const Azl: u64 = 1 << 19;  
}


pub mod sie_bits {
    pub const Apr: u64 = 1 << 1;  
    pub const Apt: u64 = 1 << 5;  
    pub const Aos: u64 = 1 << 9;  
}


pub mod satp_mode {
    pub const Asx: u64 = 0;
    pub const Apw: u64 = 8 << 60;  
    pub const Apx: u64 = 9 << 60;  
    pub const Bdd: u64 = 10 << 60; 
}
