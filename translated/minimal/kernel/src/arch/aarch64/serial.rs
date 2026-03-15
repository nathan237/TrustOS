




use super::cpu::{wr, sk, fom, djp};




static mut EA_: u64 = 0x0900_0000;


const Buy: u64 = 0x000;     
const Djy: u64 = 0x004;    
const Bac: u64 = 0x018;     
const Coh: u64 = 0x024;   
const Cog: u64 = 0x028;   
const CZE_: u64 = 0x02C;  
const Bux: u64 = 0x030;     
const Coj: u64 = 0x038;   
const Coi: u64 = 0x044;    


const BVN_: u32 = 1 << 5;   
const ASM_: u32 = 1 << 4;   
const DMG_: u32 = 1 << 3;   


pub fn wih(ar: u64) {
    unsafe { EA_ = ar; }
}


pub fn init() {
    unsafe {
        let ar = EA_;
        
        
        sk(ar + Bux, 0);
        
        
        sk(ar + Coi, 0x7FF);
        
        
        
        
        sk(ar + Coh, 26);
        sk(ar + Cog, 3);
        
        
        sk(ar + CZE_, (0b11 << 5) | (1 << 4));
        
        
        sk(ar + Coj, 0);
        
        
        sk(ar + Bux, (1 << 0) | (1 << 8) | (1 << 9));
    }
}


pub fn cco(hf: u8) {
    unsafe {
        let ar = EA_;
        
        while wr(ar + Bac) & BVN_ != 0 {
            core::hint::hc();
        }
        sk(ar + Buy, hf as u32);
    }
}


pub fn ahx(bf: &[u8]) {
    for &o in bf {
        if o == b'\n' {
            cco(b'\r');
        }
        cco(o);
    }
}


pub fn dlb() -> Option<u8> {
    unsafe {
        let ar = EA_;
        if wr(ar + Bac) & ASM_ == 0 {
            Some((wr(ar + Buy) & 0xFF) as u8)
        } else {
            None
        }
    }
}


pub fn nji() -> bool {
    unsafe {
        let ar = EA_;
        wr(ar + Bac) & ASM_ == 0
    }
}
