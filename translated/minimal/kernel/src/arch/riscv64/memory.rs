




use super::cpu;


#[inline(always)]
pub fn ghg(ag: u64) {
    unsafe {
        core::arch::asm!(
            "sfence.vma {}, zero",
            in(reg) ag,
            options(nostack, preserves_flags)
        );
    }
}


#[inline(always)]
pub fn ivc() {
    unsafe {
        core::arch::asm!("sfence.vma", options(nomem, nostack, preserves_flags));
    }
}







#[inline(always)]
pub fn dle() -> u64 {
    cpu::vsk()
}


#[inline(always)]
pub fn dnj(ap: u64) {
    unsafe { cpu::xvr(ap); }
}


pub fn zbw(mas: u64, ajv: u16) -> u64 {
    cpu::satp_mode::Cmj | ((ajv as u64) << 44) | mas
}


pub fn zbv(mas: u64, ajv: u16) -> u64 {
    cpu::satp_mode::Cmi | ((ajv as u64) << 44) | mas
}


pub fn zlo(jnl: u64) -> u64 {
    jnl & 0x00000FFFFFFFFFFF 
}
