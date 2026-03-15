





use super::cpu::{fom, djp};



static mut EA_: u64 = 0x1000_0000;


const Cns: u64 = 0;   
const Cjf: u64 = 0;   
const Cfi: u64 = 1;   
const Ccw: u64 = 2;   
const Bkj: u64 = 3;   
const Cha: u64 = 4;   
const Aut: u64 = 5;   
const Cak: u64 = 0;   
const Caj: u64 = 1;   


const AZE_: u8 = 1 << 0;
const CEX_: u8 = 1 << 5;


pub fn wih(ar: u64) {
    unsafe { EA_ = ar; }
}


pub fn init() {
    unsafe {
        let ar = EA_;
        
        
        djp(ar + Cfi, 0x00);
        
        
        djp(ar + Bkj, 0x80);
        
        
        djp(ar + Cak, 0x01);
        djp(ar + Caj, 0x00);
        
        
        djp(ar + Bkj, 0x03);
        
        
        djp(ar + Ccw, 0xC7);
        
        
        djp(ar + Cha, 0x0B);
    }
}


pub fn cco(hf: u8) {
    unsafe {
        let ar = EA_;
        
        while fom(ar + Aut) & CEX_ == 0 {
            core::hint::hc();
        }
        djp(ar + Cns, hf);
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
        if fom(ar + Aut) & AZE_ != 0 {
            Some(fom(ar + Cjf))
        } else {
            None
        }
    }
}


pub fn nji() -> bool {
    unsafe {
        let ar = EA_;
        fom(ar + Aut) & AZE_ != 0
    }
}
