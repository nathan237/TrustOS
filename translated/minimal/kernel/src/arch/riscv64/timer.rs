



use super::cpu;


#[inline(always)]
pub fn aea() -> u64 {
    cpu::vrd()
}


#[inline(always)]
pub fn yl() -> u64 {
    cpu::vqy()
}





pub fn wjq(wuj: u64) {
    unsafe {
        
        
        core::arch::asm!(
            "ecall",
            in("a7") 0x54494D45u64, 
            in("a6") 0u64,           
            in("a0") wuj,
            options(nostack)
        );
    }
}


pub fn jpd(aaq: u64) {
    let cv = aea();
    wjq(cv + aaq);
}



pub fn fjc() -> u64 {
    
    
    10_000_000
}
