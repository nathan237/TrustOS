




use core::sync::atomic::{AtomicU64, Ordering};



pub fn noy() {
    
    
    unsafe {
        let mut dln = super::cpu::lxs();
        dln |= 1 << 2;   
        dln |= 1 << 12;  
        dln &= !(1 << 1); 
        super::cpu::qab(dln);
    }
}


pub const Bcj: &str = "Limine";



















#[repr(C, align(4096))]
struct Aky([u64; 512]);


static mut BHS_: Aky = Aky([0u64; 512]);
static mut BHT_: Aky = Aky([0u64; 512]);


static BAR_: AtomicU64 = AtomicU64::new(0);


const BDS_: u64    = 1 << 0;     
const CMP_: u64    = 1 << 1;     
const CML_: u64    = 0 << 1;     
const CMJ_: u64       = 1 << 10;    
const CMO_: u64   = 0b11 << 8;  
const CMK_: u64 = 0b00 << 6; 
const CMQ_: u64      = 1 << 54;    
const CMN_: u64      = 1 << 53;    





const CEZ_: u64 = 4; 













pub unsafe fn wle(ubd: u64, uba: u64) {
    
    
    let abw = |ju: u64| -> u64 {
        ju.nj(ubd).cn(uba)
    };

    
    let ubs = abw(&js const BHS_ as u64);
    let ubt = abw(&js const BHT_ as u64);

    
    
    let okv: u64;
    core::arch::asm!("mrs {}, MAIR_EL1", bd(reg) okv, options(nomem, nostack));
    let qkz: u64 = 0xFF << 32; 
    let utb = (okv & !qkz) | (0x00u64 << 32); 
    core::arch::asm!("msr MAIR_EL1, {}", in(reg) utb, options(nomem, nostack));
    core::arch::asm!("isb", options(nomem, nostack));

    
    let qqi: u64 = 0x0000_0000  
        | BDS_
        | CML_     
        | CMJ_        
        | CMO_    
        | CMK_ 
        | (CEZ_ << 2)  
        | CMQ_       
        | CMN_;      

    BHT_.0[0] = qqi;

    
    let xai: u64 = ubt
        | BDS_
        | CMP_;    

    BHS_.0[0] = xai;

    
    core::arch::asm!("dsb ishst", options(nomem, nostack)); 
    core::arch::asm!("isb", options(nomem, nostack));

    
    
    core::arch::asm!("msr TTBR0_EL1, {}", in(reg) ubs, options(nomem, nostack));

    
    core::arch::asm!("tlbi vmalle1is", options(nomem, nostack));
    core::arch::asm!("dsb ish", options(nomem, nostack));
    core::arch::asm!("isb", options(nomem, nostack));

    BAR_.store(1, Ordering::Release);
}


pub fn yzq() -> bool {
    BAR_.load(Ordering::Acquire) != 0
}
