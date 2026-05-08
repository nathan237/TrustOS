




use super::cpu::{self, msr};





pub fn qlf(handler_addr: u64) {
    unsafe {
        
        let efer = cpu::gqa(msr::IA32_EFER);
        cpu::eei(msr::IA32_EFER, efer | msr::BWC_);
        
        
        
        
        let owa = (0x0008u64 << 32) | (0x0010u64 << 48);
        cpu::eei(msr::CEI_, owa);
        
        
        cpu::eei(msr::CEF_, handler_addr);
        
        
        cpu::eei(msr::CEE_, 0x200 | 0x400); 
    }
}
