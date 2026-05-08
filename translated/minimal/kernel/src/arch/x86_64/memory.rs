



use super::cpu;


#[inline(always)]
pub fn cxy(addr: u64) {
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn emz() {
    unsafe {
        let cr3 = cpu::iyh();
        cpu::jrm(cr3);
    }
}


#[inline(always)]
pub fn biw() -> u64 {
    unsafe { cpu::iyh() }
}


#[inline(always)]
pub fn bkc(val: u64) {
    unsafe { cpu::jrm(val); }
}


pub fn fun() {
    unsafe {
        let efer = cpu::gqa(cpu::msr::IA32_EFER);
        cpu::eei(cpu::msr::IA32_EFER, efer | cpu::msr::ATH_);
    }
}


pub fn qmr() -> bool {
    unsafe {
        let efer = cpu::gqa(cpu::msr::IA32_EFER);
        efer & cpu::msr::ATH_ != 0
    }
}
