








use core::sync::atomic::{AtomicBool, Ordering};
use super::cpu;






const AVE_: u64 = 0x0800_0000;

const AVA_: u64 = 0x0801_0000;





const UP_: u64       = 0x000;  
const AEI_: u64      = 0x004;  
const CAP_: u64  = 0x100;  
const AVF_: u64  = 0x180;  
const DRB_: u64    = 0x200;  
const CAM_: u64    = 0x280;  
const AVG_: u64 = 0x400;  
const CAS_: u64  = 0x800;  
const CAL_: u64      = 0xC00;  





const AVC_: u64  = 0x000;  
const AVD_: u64   = 0x004;  
const AVB_: u64   = 0x008;  
const CAK_: u64   = 0x00C;  
const CAJ_: u64  = 0x010;  






pub const MB_: u32 = 30;


pub const BIJ_: u32 = 1023;


static AVH_: AtomicBool = AtomicBool::new(false);





#[inline(always)]
unsafe fn men(offset: u64) -> u32 {
    cpu::kj(AVE_ + offset)
}

#[inline(always)]
unsafe fn bml(offset: u64, val: u32) {
    cpu::ib(AVE_ + offset, val);
}

#[inline(always)]
unsafe fn mel(offset: u64) -> u32 {
    cpu::kj(AVA_ + offset)
}

#[inline(always)]
unsafe fn ckd(offset: u64, val: u32) {
    cpu::ib(AVA_ + offset, val);
}









pub fn init() -> bool {
    unsafe {
        

        
        bml(UP_, 0);

        
        let dfy = men(AEI_);
        let cmv = ((dfy & 0x1F) + 1) * 32;

        crate::serial_println!("[GIC] Distributor: {} IRQ lines", cmv);

        
        let izg = cmv / 32;
        for i in 0..izg {
            bml(AVF_ + (i as u64) * 4, 0xFFFF_FFFF);
        }

        
        for i in 0..izg {
            bml(CAM_ + (i as u64) * 4, 0xFFFF_FFFF);
        }

        
        for i in 8..(cmv / 4) {
            
            bml(AVG_ + (i as u64) * 4, 0xA0A0_A0A0);
        }
        for i in 8..(cmv / 4) {
            
            bml(CAS_ + (i as u64) * 4, 0x0101_0101);
        }

        
        for i in 2..(cmv / 16) {
            bml(CAL_ + (i as u64) * 4, 0);
        }

        

        
        for i in 4..8u64 {
            bml(AVG_ + i * 4, 0x9090_9090);
        }

        
        bml(UP_, 0x3);

        

        
        ckd(AVD_, 0xFF);

        
        ckd(AVB_, 0);

        
        ckd(AVC_, 0x3);

        
        cpu::htz();
        cpu::dsv();

        AVH_.store(true, Ordering::Release);
        crate::serial_println!("[GIC] Initialized (GICD + GICC)");
    }

    true
}


pub fn cau() {
    unsafe {
        
        fum(MB_);

        
        ckd(AVD_, 0xFF);
        ckd(AVB_, 0);
        ckd(AVC_, 0x3);

        cpu::htz();
        cpu::dsv();
    }
}


pub fn fum(irq: u32) {
    let reg = irq / 32;
    let bf = irq % 32;
    unsafe {
        bml(CAP_ + (reg as u64) * 4, 1 << bf);
    }
}


pub fn hsk(irq: u32) {
    let reg = irq / 32;
    let bf = irq % 32;
    unsafe {
        bml(AVF_ + (reg as u64) * 4, 1 << bf);
    }
}



pub fn hdt() -> u32 {
    unsafe { mel(CAK_) & 0x3FF }
}


pub fn fvb(irq: u32) {
    unsafe {
        ckd(CAJ_, irq);
    }
}


pub fn is_initialized() -> bool {
    AVH_.load(Ordering::Acquire)
}




pub fn eli(interval_ms: u64) {
    
    fum(MB_);

    
    let freq = super::timer::frequency();
    let gx = (interval_ms * freq) / 1000;
    super::timer::fah(gx);

    crate::serial_println!("[GIC] Timer IRQ enabled (PPI {}, {}ms, {} ticks)", 
        MB_, interval_ms, gx);
}


pub fn fbp(interval_ms: u64) {
    eli(interval_ms);
}


pub fn jiy() {
    hsk(MB_);
    super::timer::lfa();
}


pub fn gqp(interval_ms: u64) {
    let freq = super::timer::frequency();
    let gx = (interval_ms * freq) / 1000;
    super::timer::fah(gx);
}
