




#[inline(always)]
pub fn jln() -> u64 {
    let rsp: u64;
    unsafe {
        core::arch::asm!("mov {}, rsp", bd(reg) rsp, options(nomem, nostack, preserves_flags));
    }
    rsp
}


#[inline(always)]
pub fn pae() -> u64 {
    let rbp: u64;
    unsafe {
        core::arch::asm!("mov {}, rbp", bd(reg) rbp, options(nomem, nostack, preserves_flags));
    }
    rbp
}


#[inline(always)]
pub fn zhr() -> u64 {
    let flags: u64;
    unsafe {
        core::arch::asm!(
            "pushfq",
            "pop {}",
            bd(reg) flags,
            options(nomem, preserves_flags)
        );
    }
    flags
}


#[inline(always)]
pub fn jat() {
    unsafe {
        core::arch::asm!("out 0x80, al", in("al") 0u8, options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn hbf() {
    unsafe {
        core::arch::asm!("int3", options(nomem, nostack));
    }
}


#[inline(always)]
pub unsafe fn lxk(msr: u32) -> u64 {
    let (afq, ail): (u32, u32);
    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        bd("eax") ail,
        bd("edx") afq,
        options(nomem, nostack, preserves_flags)
    );
    ((afq as u64) << 32) | (ail as u64)
}


#[inline(always)]
pub unsafe fn ihm(msr: u32, bn: u64) {
    let ail = bn as u32;
    let afq = (bn >> 32) as u32;
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") ail,
        in("edx") afq,
        options(nomem, nostack, preserves_flags)
    );
}


#[inline(always)]
pub unsafe fn zhl() -> u64 {
    let ap: u64;
    core::arch::asm!("mov {}, cr0", bd(reg) ap, options(nomem, nostack, preserves_flags));
    ap
}


#[inline(always)]
pub unsafe fn zwp(ap: u64) {
    core::arch::asm!("mov cr0, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub unsafe fn zhm() -> u64 {
    let ap: u64;
    core::arch::asm!("mov {}, cr2", bd(reg) ap, options(nomem, nostack, preserves_flags));
    ap
}


#[inline(always)]
pub unsafe fn ozy() -> u64 {
    let ap: u64;
    core::arch::asm!("mov {}, cr3", bd(reg) ap, options(nomem, nostack, preserves_flags));
    ap
}


#[inline(always)]
pub unsafe fn pzw(ap: u64) {
    core::arch::asm!("mov cr3, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub unsafe fn zhn() -> u64 {
    let ap: u64;
    core::arch::asm!("mov {}, cr4", bd(reg) ap, options(nomem, nostack, preserves_flags));
    ap
}


#[inline(always)]
pub unsafe fn zwq(ap: u64) {
    core::arch::asm!("mov cr4, {}", in(reg) ap, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub unsafe fn ipl(awa: u32, bxj: u32) -> (u32, u32, u32, u32) {
    let (eax, ebx, ecx, edx): (u32, u32, u32, u32);
    
    core::arch::asm!(
        "push rbx",
        "cpuid",
        "mov {ebx_out:e}, ebx",
        "pop rbx",
        inout("eax") awa => eax,
        inout("ecx") bxj => ecx,
        ish = bd(reg) ebx,
        bd("edx") edx,
        options(nostack, preserves_flags)
    );
    (eax, ebx, ecx, edx)
}


#[inline(always)]
pub fn lxl() -> u64 {
    let (afq, ail): (u32, u32);
    unsafe {
        core::arch::asm!(
            "rdtsc",
            bd("eax") ail,
            bd("edx") afq,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((afq as u64) << 32) | (ail as u64)
}


#[inline(always)]
pub fn cbg() -> Option<u64> {
    let ap: u64;
    let bq: u8;
    unsafe {
        core::arch::asm!(
            "rdrand {}",
            "setc {}",
            bd(reg) ap,
            bd(reg_byte) bq,
            options(nomem, nostack)
        );
    }
    if bq != 0 { Some(ap) } else { None }
}






#[inline(always)]
pub unsafe fn cfn(port: u16) -> u8 {
    let ap: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        bd("al") ap,
        options(nomem, nostack, preserves_flags)
    );
    ap
}


#[inline(always)]
pub unsafe fn bkt(port: u16, ap: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") ap,
        options(nomem, nostack, preserves_flags)
    );
}


#[inline(always)]
pub unsafe fn jar(port: u16) -> u16 {
    let ap: u16;
    core::arch::asm!(
        "in ax, dx",
        in("dx") port,
        bd("ax") ap,
        options(nomem, nostack, preserves_flags)
    );
    ap
}


#[inline(always)]
pub unsafe fn jie(port: u16, ap: u16) {
    core::arch::asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") ap,
        options(nomem, nostack, preserves_flags)
    );
}


#[inline(always)]
pub unsafe fn jac(port: u16) -> u32 {
    let ap: u32;
    core::arch::asm!(
        "in eax, dx",
        in("dx") port,
        bd("eax") ap,
        options(nomem, nostack, preserves_flags)
    );
    ap
}


#[inline(always)]
pub unsafe fn jic(port: u16, ap: u32) {
    core::arch::asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") ap,
        options(nomem, nostack, preserves_flags)
    );
}


pub mod msr {
    pub const CN_: u32 = 0xC0000080;
    pub const CAX_: u32 = 0xC0000081;
    pub const CAU_: u32 = 0xC0000082;
    pub const CAT_: u32 = 0xC0000084;
    pub const DPX_: u32 = 0xC0000100;
    pub const DPY_: u32 = 0xC0000101;
    pub const DPZ_: u32 = 0xC0000102;
    pub const KK_: u32 = 0x1B;
    pub const KL_: u32 = 0x277;
    
    
    pub const BTG_: u64 = 1 << 0;  
    pub const DLD_: u64 = 1 << 8;  
    pub const DLC_: u64 = 1 << 10; 
    pub const ARE_: u64 = 1 << 11; 
}
