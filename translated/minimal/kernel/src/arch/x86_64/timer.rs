



use super::cpu;


#[inline(always)]
pub fn timestamp() -> u64 {
    cpu::gqb()
}


const XQ_: u16 = 0x40;
const XR_: u16 = 0x43;
const CNE_: u32 = 1_193_182; 


pub fn qql(hz: u32) {
    let divisor = if hz == 0 { 0xFFFF } else { CNE_ / hz };
    let divisor = divisor.min(0xFFFF) as u16;
    
    unsafe {
        
        cpu::vp(XR_, 0x34);
        cpu::vp(XQ_, (divisor & 0xFF) as u8);
        cpu::vp(XQ_, (divisor >> 8) as u8);
    }
}


pub fn qqk() -> u16 {
    unsafe {
        cpu::vp(XR_, 0x00); 
        let lo = cpu::om(XQ_) as u16;
        let hi = cpu::om(XQ_) as u16;
        (hi << 8) | lo
    }
}
