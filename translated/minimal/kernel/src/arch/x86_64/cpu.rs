




#[inline(always)]
pub fn exy() -> u64 {
    let rsp: u64;
    unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) rsp, options(nomem, nostack, preserves_flags));
    }
    rsp
}


#[inline(always)]
pub fn iyk() -> u64 {
    let rbp: u64;
    unsafe {
        core::arch::asm!("mov {}, rbp", out(reg) rbp, options(nomem, nostack, preserves_flags));
    }
    rbp
}


#[inline(always)]
pub fn qsd() -> u64 {
    let flags: u64;
    unsafe {
        core::arch::asm!(
            "pushfq",
            "pop {}",
            out(reg) flags,
            options(nomem, preserves_flags)
        );
    }
    flags
}


#[inline(always)]
pub fn erb() {
    unsafe {
        core::arch::asm!("out 0x80, al", in("al") 0u8, options(nomem, nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn breakpoint() {
    unsafe {
        core::arch::asm!("int3", options(nomem, nostack));
    }
}


#[inline(always)]
pub unsafe fn gqa(msr: u32) -> u64 {
    let (high, low): (u32, u32);
    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nomem, nostack, preserves_flags)
    );
    ((high as u64) << 32) | (low as u64)
}


#[inline(always)]
pub unsafe fn eei(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack, preserves_flags)
    );
}


#[inline(always)]
pub unsafe fn qrx() -> u64 {
    let val: u64;
    core::arch::asm!("mov {}, cr0", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}


#[inline(always)]
pub unsafe fn rcz(val: u64) {
    core::arch::asm!("mov cr0, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub unsafe fn qry() -> u64 {
    let val: u64;
    core::arch::asm!("mov {}, cr2", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}


#[inline(always)]
pub unsafe fn iyh() -> u64 {
    let val: u64;
    core::arch::asm!("mov {}, cr3", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}


#[inline(always)]
pub unsafe fn jrm(val: u64) {
    core::arch::asm!("mov cr3, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub unsafe fn qrz() -> u64 {
    let val: u64;
    core::arch::asm!("mov {}, cr4", out(reg) val, options(nomem, nostack, preserves_flags));
    val
}


#[inline(always)]
pub unsafe fn rda(val: u64) {
    core::arch::asm!("mov cr4, {}", in(reg) val, options(nomem, nostack, preserves_flags));
}


#[inline(always)]
pub unsafe fn st(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
    let (eax, ebx, ecx, edx): (u32, u32, u32, u32);
    
    core::arch::asm!(
        "push rbx",
        "cpuid",
        "mov {ebx_out:e}, ebx",
        "pop rbx",
        inout("eax") leaf => eax,
        inout("ecx") subleaf => ecx,
        ebx_out = out(reg) ebx,
        out("edx") edx,
        options(nostack, preserves_flags)
    );
    (eax, ebx, ecx, edx)
}


#[inline(always)]
pub fn gqb() -> u64 {
    let (high, low): (u32, u32);
    unsafe {
        core::arch::asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((high as u64) << 32) | (low as u64)
}


#[inline(always)]
pub fn rdrand() -> Option<u64> {
    let val: u64;
    let ok: u8;
    unsafe {
        core::arch::asm!(
            "rdrand {}",
            "setc {}",
            out(reg) val,
            out(reg_byte) ok,
            options(nomem, nostack)
        );
    }
    if ok != 0 { Some(val) } else { None }
}






#[inline(always)]
pub unsafe fn om(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") val,
        options(nomem, nostack, preserves_flags)
    );
    val
}


#[inline(always)]
pub unsafe fn vp(port: u16, val: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}


#[inline(always)]
pub unsafe fn eqz(port: u16) -> u16 {
    let val: u16;
    core::arch::asm!(
        "in ax, dx",
        in("dx") port,
        out("ax") val,
        options(nomem, nostack, preserves_flags)
    );
    val
}


#[inline(always)]
pub unsafe fn evw(port: u16, val: u16) {
    core::arch::asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") val,
        options(nomem, nostack, preserves_flags)
    );
}


#[inline(always)]
pub unsafe fn eqp(port: u16) -> u32 {
    let val: u32;
    core::arch::asm!(
        "in eax, dx",
        in("dx") port,
        out("eax") val,
        options(nomem, nostack, preserves_flags)
    );
    val
}


#[inline(always)]
pub unsafe fn evv(port: u16, val: u32) {
    core::arch::asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") val,
        options(nomem, nostack, preserves_flags)
    );
}


pub mod msr {
    pub const IA32_EFER: u32 = 0xC0000080;
    pub const CEI_: u32 = 0xC0000081;
    pub const CEF_: u32 = 0xC0000082;
    pub const CEE_: u32 = 0xC0000084;
    pub const DTR_: u32 = 0xC0000100;
    pub const DTS_: u32 = 0xC0000101;
    pub const DTT_: u32 = 0xC0000102;
    pub const LD_: u32 = 0x1B;
    pub const LE_: u32 = 0x277;
    
    
    pub const BWC_: u64 = 1 << 0;  
    pub const DOS_: u64 = 1 << 8;  
    pub const DOR_: u64 = 1 << 10; 
    pub const ATH_: u64 = 1 << 11; 
}
