




use super::cpu::{kj, ib, cmx, bhy};




static mut EM_: u64 = 0x0900_0000;


const Afo: u64 = 0x000;     
const Bec: u64 = 0x004;    
const Vo: u64 = 0x018;     
const Aqt: u64 = 0x024;   
const Aqs: u64 = 0x028;   
const DCW_: u64 = 0x02C;  
const Afn: u64 = 0x030;     
const Aqv: u64 = 0x038;   
const Aqu: u64 = 0x044;    


const BYT_: u32 = 1 << 5;   
const AUQ_: u32 = 1 << 4;   
const DQA_: u32 = 1 << 3;   


pub fn oop(base: u64) {
    unsafe { EM_ = base; }
}


pub fn init() {
    unsafe {
        let base = EM_;
        
        
        ib(base + Afn, 0);
        
        
        ib(base + Aqu, 0x7FF);
        
        
        
        
        ib(base + Aqt, 26);
        ib(base + Aqs, 3);
        
        
        ib(base + DCW_, (0b11 << 5) | (1 << 4));
        
        
        ib(base + Aqv, 0);
        
        
        ib(base + Afn, (1 << 0) | (1 << 8) | (1 << 9));
    }
}


pub fn write_byte(byte: u8) {
    unsafe {
        let base = EM_;
        
        while kj(base + Vo) & BYT_ != 0 {
            core::hint::spin_loop();
        }
        ib(base + Afo, byte as u32);
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
        if kj(base + Vo) & AUQ_ == 0 {
            Some((kj(base + Afo) & 0xFF) as u8)
        } else {
            None
        }
    }
}


pub fn hqn() -> bool {
    unsafe {
        let base = EM_;
        kj(base + Vo) & AUQ_ == 0
    }
}
