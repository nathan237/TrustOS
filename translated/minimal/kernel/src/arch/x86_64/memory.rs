



use super::cpu;


#[inline(always)]
pub fn ghg(ag: u64) {
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) ag, options(nostack, preserves_flags));
    }
}


#[inline(always)]
pub fn ivc() {
    unsafe {
        let jm = cpu::ozy();
        cpu::pzw(jm);
    }
}


#[inline(always)]
pub fn dle() -> u64 {
    unsafe { cpu::ozy() }
}


#[inline(always)]
pub fn dnj(ap: u64) {
    unsafe { cpu::pzw(ap); }
}


pub fn ktf() {
    unsafe {
        let efer = cpu::lxk(cpu::msr::CN_);
        cpu::ihm(cpu::msr::CN_, efer | cpu::msr::ARE_);
    }
}


pub fn yzs() -> bool {
    unsafe {
        let efer = cpu::lxk(cpu::msr::CN_);
        efer & cpu::msr::ARE_ != 0
    }
}
