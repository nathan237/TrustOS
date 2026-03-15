








use core::sync::atomic::{AtomicBool, Ordering};
use super::cpu;






const ATA_: u64 = 0x0800_0000;

const ASW_: u64 = 0x0801_0000;





const TJ_: u64       = 0x000;  
const ACS_: u64      = 0x004;  
const BXJ_: u64  = 0x100;  
const ATB_: u64  = 0x180;  
const DNH_: u64    = 0x200;  
const BXG_: u64    = 0x280;  
const ATC_: u64 = 0x400;  
const BXM_: u64  = 0x800;  
const BXF_: u64      = 0xC00;  





const ASY_: u64  = 0x000;  
const ASZ_: u64   = 0x004;  
const ASX_: u64   = 0x008;  
const BXE_: u64   = 0x00C;  
const BXD_: u64  = 0x010;  






pub const LG_: u32 = 30;


pub const BGF_: u32 = 1023;


static ATD_: AtomicBool = AtomicBool::new(false);





#[inline(always)]
unsafe fn tfo(l: u64) -> u32 {
    cpu::wr(ATA_ + l)
}

#[inline(always)]
unsafe fn dre(l: u64, ap: u32) {
    cpu::sk(ATA_ + l, ap);
}

#[inline(always)]
unsafe fn tfn(l: u64) -> u32 {
    cpu::wr(ASW_ + l)
}

#[inline(always)]
unsafe fn fjq(l: u64, ap: u32) {
    cpu::sk(ASW_ + l, ap);
}









pub fn init() -> bool {
    unsafe {
        

        
        dre(TJ_, 0);

        
        let gvg = tfo(ACS_);
        let fnw = ((gvg & 0x1F) + 1) * 32;

        crate::serial_println!("[GIC] Distributor: {} IRQ lines", fnw);

        
        let pbg = fnw / 32;
        for a in 0..pbg {
            dre(ATB_ + (a as u64) * 4, 0xFFFF_FFFF);
        }

        
        for a in 0..pbg {
            dre(BXG_ + (a as u64) * 4, 0xFFFF_FFFF);
        }

        
        for a in 8..(fnw / 4) {
            
            dre(ATC_ + (a as u64) * 4, 0xA0A0_A0A0);
        }
        for a in 8..(fnw / 4) {
            
            dre(BXM_ + (a as u64) * 4, 0x0101_0101);
        }

        
        for a in 2..(fnw / 16) {
            dre(BXF_ + (a as u64) * 4, 0);
        }

        

        
        for a in 4..8u64 {
            dre(ATC_ + a * 4, 0x9090_9090);
        }

        
        dre(TJ_, 0x3);

        

        
        fjq(ASZ_, 0xFF);

        
        fjq(ASX_, 0);

        
        fjq(ASY_, 0x3);

        
        cpu::nny();
        cpu::hpd();

        ATD_.store(true, Ordering::Release);
        crate::serial_println!("[GIC] Initialized (GICD + GICC)");
    }

    true
}


pub fn eso() {
    unsafe {
        
        kte(LG_);

        
        fjq(ASZ_, 0xFF);
        fjq(ASX_, 0);
        fjq(ASY_, 0x3);

        cpu::nny();
        cpu::hpd();
    }
}


pub fn kte(irq: u32) {
    let reg = irq / 32;
    let ga = irq % 32;
    unsafe {
        dre(BXJ_ + (reg as u64) * 4, 1 << ga);
    }
}


pub fn nlr(irq: u32) {
    let reg = irq / 32;
    let ga = irq % 32;
    unsafe {
        dre(ATB_ + (reg as u64) * 4, 1 << ga);
    }
}



pub fn mtl() -> u32 {
    unsafe { tfn(BXE_) & 0x3FF }
}


pub fn ktu(irq: u32) {
    unsafe {
        fjq(BXD_, irq);
    }
}


pub fn ky() -> bool {
    ATD_.load(Ordering::Acquire)
}




pub fn isr(edm: u64) {
    
    kte(LG_);

    
    let kx = super::timer::fjc();
    let qb = (edm * kx) / 1000;
    super::timer::jpd(qb);

    crate::serial_println!("[GIC] Timer IRQ enabled (PPI {}, {}ms, {} ticks)", 
        LG_, edm, qb);
}


pub fn jro(edm: u64) {
    isr(edm);
}


pub fn pox() {
    nlr(LG_);
    super::timer::rxz();
}


pub fn lye(edm: u64) {
    let kx = super::timer::fjc();
    let qb = (edm * kx) / 1000;
    super::timer::jpd(qb);
}
