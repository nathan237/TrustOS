





use super::cpu::{cmx, bhy};



static mut EM_: u64 = 0x1000_0000;


const Aqi: u64 = 0;   
const Anl: u64 = 0;   
const Alh: u64 = 1;   
const Ajq: u64 = 2;   
const Aam: u64 = 3;   
const Amn: u64 = 4;   
const Tj: u64 = 5;   
const Aij: u64 = 0;   
const Aii: u64 = 1;   


const BBF_: u8 = 1 << 0;
const CIG_: u8 = 1 << 5;


pub fn oop(base: u64) {
    unsafe { EM_ = base; }
}


pub fn init() {
    unsafe {
        let base = EM_;
        
        
        bhy(base + Alh, 0x00);
        
        
        bhy(base + Aam, 0x80);
        
        
        bhy(base + Aij, 0x01);
        bhy(base + Aii, 0x00);
        
        
        bhy(base + Aam, 0x03);
        
        
        bhy(base + Ajq, 0xC7);
        
        
        bhy(base + Amn, 0x0B);
    }
}


pub fn write_byte(byte: u8) {
    unsafe {
        let base = EM_;
        
        while cmx(base + Tj) & CIG_ == 0 {
            core::hint::spin_loop();
        }
        bhy(base + Aqi, byte);
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
    unsafe {
        let base = EM_;
        if cmx(base + Tj) & BBF_ != 0 {
            Some(cmx(base + Anl))
        } else {
            None
        }
    }
}


pub fn hqn() -> bool {
    unsafe {
        let base = EM_;
        cmx(base + Tj) & BBF_ != 0
    }
}
