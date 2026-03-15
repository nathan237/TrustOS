





use super::cpu::{cfn, bkt};
use core::sync::atomic::{AtomicBool, Ordering};


const Gg: u16 = 0x3F8;


const DBS_: u32 = 100_000;


static WX_: AtomicBool = AtomicBool::new(false);


pub fn xo() -> bool {
    WX_.load(Ordering::Relaxed)
}


pub fn init() {
    unsafe {
        
        bkt(Gg + 4, 0x1E); 
        bkt(Gg, 0xAE);     
        if cfn(Gg) != 0xAE {
            
            return;
        }

        
        bkt(Gg + 4, 0x0F); 
        bkt(Gg + 1, 0x00); 
        bkt(Gg + 3, 0x80); 
        bkt(Gg + 0, 0x01); 
        bkt(Gg + 1, 0x00); 
        bkt(Gg + 3, 0x03); 
        bkt(Gg + 2, 0xC7); 
        bkt(Gg + 4, 0x0B); 

        WX_.store(true, Ordering::SeqCst);
    }
}


pub fn cco(hf: u8) {
    if !WX_.load(Ordering::Relaxed) {
        return;
    }
    unsafe {
        
        let mut aah = DBS_;
        while cfn(Gg + 5) & 0x20 == 0 {
            aah -= 1;
            if aah == 0 {
                return; 
            }
            core::hint::hc();
        }
        bkt(Gg, hf);
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
    if !WX_.load(Ordering::Relaxed) {
        return None;
    }
    unsafe {
        if cfn(Gg + 5) & 0x01 != 0 {
            Some(cfn(Gg))
        } else {
            None
        }
    }
}


pub fn nji() -> bool {
    unsafe { cfn(Gg + 5) & 0x01 != 0 }
}
