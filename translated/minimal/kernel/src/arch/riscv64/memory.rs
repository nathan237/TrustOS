




use super::cpu;


#[inline(always)]
pub fn cxy(addr: u64) {
    unsafe {
        core::arch::asm!(
            "sfence.vma {}, zero",
            in(reg) addr,
            options(nostack, preserves_flags)
        );
    }
}


#[inline(always)]
pub fn emz() {
    unsafe {
        core::arch::asm!("sfence.vma", options(nomem, nostack, preserves_flags));
    }
}







#[inline(always)]
pub fn biw() -> u64 {
    cpu::odb()
}


#[inline(always)]
pub fn bkc(val: u64) {
    unsafe { cpu::pvc(val); }
}


pub fn qoi(root_ppn: u64, asid: u16) -> u64 {
    cpu::satp_mode::Apx | ((asid as u64) << 44) | root_ppn
}


pub fn qoh(root_ppn: u64, asid: u16) -> u64 {
    cpu::satp_mode::Apw | ((asid as u64) << 44) | root_ppn
}


pub fn quq(satp: u64) -> u64 {
    satp & 0x00000FFFFFFFFFFF 
}
