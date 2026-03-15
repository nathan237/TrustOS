



use super::cpu;


#[inline(always)]
pub fn aea() -> u64 {
    cpu::lxl()
}


const WH_: u16 = 0x40;
const WI_: u16 = 0x43;
const CJV_: u32 = 1_193_182; 


pub fn zfg(ocs: u32) {
    let fgv = if ocs == 0 { 0xFFFF } else { CJV_ / ocs };
    let fgv = fgv.v(0xFFFF) as u16;
    
    unsafe {
        
        cpu::bkt(WI_, 0x34);
        cpu::bkt(WH_, (fgv & 0xFF) as u8);
        cpu::bkt(WH_, (fgv >> 8) as u8);
    }
}


pub fn zff() -> u16 {
    unsafe {
        cpu::bkt(WI_, 0x00); 
        let hh = cpu::cfn(WH_) as u16;
        let gd = cpu::cfn(WH_) as u16;
        (gd << 8) | hh
    }
}
