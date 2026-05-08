




use core::sync::atomic::{AtomicU64, Ordering};



pub fn hut() {
    
    
    unsafe {
        let mut bjb = super::cpu::gqf();
        bjb |= 1 << 2;   
        bjb |= 1 << 12;  
        bjb &= !(1 << 1); 
        super::cpu::jrq(bjb);
    }
}


pub const Wo: &str = "Limine";



















#[repr(C, align(4096))]
struct Pu([u64; 512]);


static mut BJW_: Pu = Pu([0u64; 512]);
static mut BJX_: Pu = Pu([0u64; 512]);


static BCT_: AtomicU64 = AtomicU64::new(0);


const BFV_: u64    = 1 << 0;     
const CPY_: u64    = 1 << 1;     
const CPU_: u64    = 0 << 1;     
const CPS_: u64       = 1 << 10;    
const CPX_: u64   = 0b11 << 8;  
const CPT_: u64 = 0b00 << 6; 
const CPZ_: u64      = 1 << 54;    
const CPW_: u64      = 1 << 53;    





const CII_: u64 = 4; 













pub unsafe fn oql(kernel_virt_base: u64, kernel_phys_base: u64) {
    
    
    let lc = |virt: u64| -> u64 {
        virt.wrapping_sub(kernel_virt_base).wrapping_add(kernel_phys_base)
    };

    
    let l0_phys = lc(&dm const BJW_ as u64);
    let mwb = lc(&dm const BJX_ as u64);

    
    
    let mair: u64;
    core::arch::asm!("mrs {}, MAIR_EL1", out(reg) mair, options(nomem, nostack));
    let jya: u64 = 0xFF << 32; 
    let new_mair = (mair & !jya) | (0x00u64 << 32); 
    core::arch::asm!("msr MAIR_EL1, {}", in(reg) new_mair, options(nomem, nostack));
    core::arch::asm!("isb", options(nomem, nostack));

    
    let kcq: u64 = 0x0000_0000  
        | BFV_
        | CPU_     
        | CPS_        
        | CPX_    
        | CPT_ 
        | (CII_ << 2)  
        | CPZ_       
        | CPW_;      

    BJX_.0[0] = kcq;

    
    let pct: u64 = mwb
        | BFV_
        | CPY_;    

    BJW_.0[0] = pct;

    
    core::arch::asm!("dsb ishst", options(nomem, nostack)); 
    core::arch::asm!("isb", options(nomem, nostack));

    
    
    core::arch::asm!("msr TTBR0_EL1, {}", in(reg) l0_phys, options(nomem, nostack));

    
    core::arch::asm!("tlbi vmalle1is", options(nomem, nostack));
    core::arch::asm!("dsb ish", options(nomem, nostack));
    core::arch::asm!("isb", options(nomem, nostack));

    BCT_.store(1, Ordering::Release);
}


pub fn qmp() -> bool {
    BCT_.load(Ordering::Acquire) != 0
}
