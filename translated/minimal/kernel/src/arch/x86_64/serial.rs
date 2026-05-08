





use super::cpu::{om, vp};
use core::sync::atomic::{AtomicBool, Ordering};


const Cu: u16 = 0x3F8;


const DFN_: u32 = 100_000;


static YE_: AtomicBool = AtomicBool::new(false);


pub fn is_present() -> bool {
    YE_.load(Ordering::Relaxed)
}


pub fn init() {
    unsafe {
        
        vp(Cu + 4, 0x1E); 
        vp(Cu, 0xAE);     
        if om(Cu) != 0xAE {
            
            return;
        }

        
        vp(Cu + 4, 0x0F); 
        vp(Cu + 1, 0x00); 
        vp(Cu + 3, 0x80); 
        vp(Cu + 0, 0x01); 
        vp(Cu + 1, 0x00); 
        vp(Cu + 3, 0x03); 
        vp(Cu + 2, 0xC7); 
        vp(Cu + 4, 0x0B); 

        YE_.store(true, Ordering::SeqCst);
    }
}


pub fn write_byte(byte: u8) {
    if !YE_.load(Ordering::Relaxed) {
        return;
    }
    unsafe {
        
        let mut mz = DFN_;
        while om(Cu + 5) & 0x20 == 0 {
            mz -= 1;
            if mz == 0 {
                return; 
            }
            core::hint::spin_loop();
        }
        vp(Cu, byte);
    }
}


pub fn write_bytes(bytes: &[u8]) {
    for &b in bytes {
        if b == b'\n' {
            write_byte(b'\r');
        }
        write_byte(b);
    }
}


pub fn read_byte() -> Option<u8> {
    if !YE_.load(Ordering::Relaxed) {
        return None;
    }
    unsafe {
        if om(Cu + 5) & 0x01 != 0 {
            Some(om(Cu))
        } else {
            None
        }
    }
}


pub fn hqn() -> bool {
    unsafe { om(Cu + 5) & 0x01 != 0 }
}
