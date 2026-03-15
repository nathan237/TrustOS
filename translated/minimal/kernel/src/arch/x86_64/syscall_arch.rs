




use super::cpu::{self, msr};





pub fn yxu(tlu: u64) {
    unsafe {
        
        let efer = cpu::lxk(msr::CN_);
        cpu::ihm(msr::CN_, efer | msr::BTG_);
        
        
        
        
        let wsj = (0x0008u64 << 32) | (0x0010u64 << 48);
        cpu::ihm(msr::CAX_, wsj);
        
        
        cpu::ihm(msr::CAU_, tlu);
        
        
        cpu::ihm(msr::CAT_, 0x200 | 0x400); 
    }
}
