



use super::cpu;


#[inline(always)]
pub fn timestamp() -> u64 {
    cpu::och()
}


#[inline(always)]
pub fn cycles() -> u64 {
    cpu::ocd()
}





pub fn opm(stime_value: u64) {
    unsafe {
        
        
        core::arch::asm!(
            "ecall",
            in("a7") 0x54494D45u64, 
            in("a6") 0u64,           
            in("a0") stime_value,
            options(nostack)
        );
    }
}


pub fn fah(mk: u64) {
    let current = timestamp();
    opm(current + mk);
}



pub fn frequency() -> u64 {
    
    
    10_000_000
}
