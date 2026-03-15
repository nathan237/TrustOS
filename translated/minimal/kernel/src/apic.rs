













use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};





const ADW_: u32         = 0x020;  
const ADY_: u32    = 0x030;  
const UM_: u32        = 0x080;  
const ADU_: u32        = 0x0B0;  
const NW_: u32        = 0x0F0;  
const AYH_: u32     = 0x300;  
const AYF_: u32     = 0x310;  
const ID_: u32  = 0x320;  
const CDK_: u32  = 0x350;  
const AYL_: u32  = 0x360;  
const ADV_: u32  = 0x370;  
const KN_: u32  = 0x380;  
const ADX_: u32  = 0x390;  
const NX_: u32  = 0x3E0;  


const CUS_: u32 = 1 << 8;
const CUC_: u32  = 0xFF;


const CYG_: u32   = 1 << 17;
const AIU_: u32     = 1 << 16;


const BHD_: u32     = 0x03; 





const DRJ_: u32       = 0x00;
const CCE_: u32      = 0x01;
const AXG_: u32   = 0x10; 


const CCC_: u64            = 1 << 16;
const AXF_: u64   = 1 << 15;
const AXE_: u64        = 1 << 13;








pub const AIV_: u8      = 48;

pub const CUB_: u8      = 0xFF;

pub const CCN_: u8        = 0xFE;



pub const ADQ_: u8          = 49;


pub const UJ_: u8   = ADQ_ + 1;  

pub const VJ_: u8      = ADQ_ + 12; 

pub const HH_: u8     = 62;





static AKR_: AtomicBool = AtomicBool::new(false);
static UL_: AtomicU64 = AtomicU64::new(0);
static UG_: AtomicU64 = AtomicU64::new(0);


static QA_: AtomicU64 = AtomicU64::new(0);






#[inline]
unsafe fn jck(l: u32) -> u32 {
    let ar = UL_.load(Ordering::Relaxed);
    core::ptr::read_volatile((ar + l as u64) as *const u32)
}


#[inline]
unsafe fn brf(l: u32, bn: u32) {
    let ar = UL_.load(Ordering::Relaxed);
    core::ptr::write_volatile((ar + l as u64) as *mut u32, bn);
}


pub fn dsp() {
    unsafe {
        brf(ADU_, 0);
    }
}


pub fn ett() -> u32 {
    unsafe { jck(ADW_) >> 24 }
}


fn npv() {
    unsafe {
        
        let bim = jck(NW_);
        brf(NW_, bim | CUS_ | CUC_);
        
        
        brf(UM_, 0);
    }
}



const TB_: u64 = 1000;



fn qvm() -> u64 {
    unsafe {
        
        brf(NX_, BHD_);
        brf(ID_, AIU_);
        
        
        brf(KN_, 0xFFFF_FFFF);
        
        
        crate::cpu::tsc::rd(10);
        
        
        let ia = jck(ADX_);
        let ez = 0xFFFF_FFFFu64 - ia as u64;
        
        
        brf(ID_, AIU_);
        
        
        let dcy = ez / 10;
        
        if dcy == 0 {
            crate::serial_println!("[APIC] WARNING: PIT calibration failed (elapsed={}), using fallback {} ticks/ms", ez, TB_);
            return TB_;
        }
        
        crate::serial_println!("[APIC] Timer calibrated: {} ticks/ms ({} ticks in 10ms)", dcy, ez);
        dcy
    }
}



pub fn jro(edm: u64) {
    let mut dcy = QA_.load(Ordering::Relaxed);
    if dcy == 0 {
        crate::serial_println!("[APIC] WARNING: Timer not calibrated, using fallback {} ticks/ms", TB_);
        dcy = TB_;
        QA_.store(dcy, Ordering::SeqCst);
    }
    
    let az = dcy * edm;
    
    unsafe {
        
        brf(NX_, BHD_);
        
        
        brf(ID_, CYG_ | AIV_ as u32);
        
        
        brf(KN_, az as u32);
    }
    
    crate::serial_println!("[APIC] Timer started: {}ms interval, count={}", edm, az);
}


pub fn pox() {
    unsafe {
        brf(ID_, AIU_);
        brf(KN_, 0);
    }
}


pub fn mds(mjw: u32, wj: u8) {
    unsafe {
        
        brf(AYF_, mjw << 24);
        
        brf(AYH_, wj as u32);
    }
}


pub fn phu(wj: u8) {
    unsafe {
        brf(AYF_, 0);
        
        brf(AYH_, (0b11 << 18) | wj as u32);
    }
}






unsafe fn lfq(reg: u32, bn: u32) {
    let ar = UG_.load(Ordering::Relaxed);
    
    core::ptr::write_volatile(ar as *mut u32, reg);
    
    core::ptr::write_volatile((ar + 0x10) as *mut u32, bn);
}


unsafe fn ofj(reg: u32) -> u32 {
    let ar = UG_.load(Ordering::Relaxed);
    core::ptr::write_volatile(ar as *mut u32, reg);
    core::ptr::read_volatile((ar + 0x10) as *const u32)
}






unsafe fn lfp(irq: u8, wj: u8, rwf: u8, flags: u64) {
    let hxi = AXG_ + (irq as u32) * 2;
    let vtx = hxi + 1;
    
    
    let smg = (wj as u64) | flags;
    
    let smc = (rwf as u64) << 24;
    
    lfq(hxi, smg as u32);
    lfq(vtx, smc as u32);
}


unsafe fn twf() -> u8 {
    let axh = ofj(CCE_);
    ((axh >> 16) & 0xFF) as u8
}







fn wlb() {
    let fzr = match crate::acpi::ani() {
        Some(co) => co,
        None => {
            crate::serial_println!("[APIC] WARNING: No ACPI info, cannot set up I/O APIC");
            return;
        }
    };
    
    if fzr.cyx.is_empty() {
        crate::serial_println!("[APIC] WARNING: No I/O APIC found in MADT");
        return;
    }
    
    let ioapic = &fzr.cyx[0];
    
    let twg = match crate::memory::bki(ioapic.re, 4096) {
        Ok(p) => p,
        Err(aa) => {
            crate::serial_println!("[APIC] Failed to map I/O APIC MMIO at {:#x}: {}", ioapic.re, aa);
            return;
        }
    };
    UG_.store(twg, Ordering::SeqCst);
    
    unsafe {
        let hqy = twf();
        crate::serial_println!("[APIC] I/O APIC id={}, addr={:#x}, GSI base={}, max_entries={}",
            ioapic.ad, ioapic.re, ioapic.ech, hqy);
        
        
        for a in 0..=hqy {
            let hxi = AXG_ + (a as u32) * 2;
            let hh = ofj(hxi);
            lfq(hxi, hh | CCC_ as u32);
        }
        
        
        let mut lhb: u8 = 1;
        let mut ohq: u64 = 0; 
        
        
        for bvu in &fzr.gka {
            if bvu.iy == 1 {
                lhb = bvu.bup as u8;
                ohq = otl(bvu);
                crate::serial_println!("[APIC] Keyboard IRQ override: ISA 1 → GSI {}", bvu.bup);
            }
        }
        lfp(lhb, UJ_, 0, ohq);
        crate::serial_println!("[APIC] Routed keyboard: IRQ {} → vector {}", lhb, UJ_);
        
        
        let mut lmr: u8 = 12;
        let mut ooa: u64 = 0;
        
        for bvu in &fzr.gka {
            if bvu.iy == 12 {
                lmr = bvu.bup as u8;
                ooa = otl(bvu);
                crate::serial_println!("[APIC] Mouse IRQ override: ISA 12 → GSI {}", bvu.bup);
            }
        }
        lfp(lmr, VJ_, 0, ooa);
        crate::serial_println!("[APIC] Routed mouse: IRQ {} → vector {}", lmr, VJ_);
    }
}


fn otl(bvu: &crate::acpi::madt::Xc) -> u64 {
    let mut flags: u64 = 0;
    
    
    match bvu.dkr {
        0 => {} 
        1 => {} 
        3 => flags |= AXE_,
        _ => {}
    }
    
    
    match bvu.dmt {
        0 => {} 
        1 => {} 
        3 => flags |= AXF_,
        _ => {}
    }
    
    flags
}





unsafe fn rxx() {
    use x86_64::instructions::port::Port;
    
    
    let mut vhq = Port::<u8>::new(0x21);
    let mut vhr = Port::<u8>::new(0xA1);
    
    vhq.write(0xFF);
    vhr.write(0xFF);
    
    crate::serial_println!("[APIC] Legacy PIC disabled (all IRQs masked)");
}



fn nfl() {
    let co = match crate::acpi::ani() {
        Some(a) => a,
        None => return,
    };
    
    if co.fne.is_empty() {
        
        unsafe {
            
            brf(AYL_, 0x0400);
        }
        crate::serial_println!("[APIC] NMI: default LINT1=NMI (no MADT entries)");
        return;
    }
    
    for evi in &co.fne {
        
        
        let ufh = if evi.gln == 0 { CDK_ } else { AYL_ };
        
        
        let mut jee: u32 = 0x0400; 
        
        
        if evi.dkr == 3 {
            jee |= 1 << 13; 
        }
        
        
        if evi.dmt == 3 {
            jee |= 1 << 15; 
        }
        
        unsafe { brf(ufh, jee); }
        crate::serial_println!("[APIC] NMI: LINT{} = NMI (pol={}, trig={}, lvt={:#x})",
            evi.gln, evi.dkr, evi.dmt, jee);
    }
}











pub fn init() -> bool {
    let jcj = crate::acpi::ljo();
    if jcj == 0 {
        crate::serial_println!("[APIC] No LAPIC address from ACPI, staying on PIC");
        return false;
    }
    
    
    
    let gkr = match crate::memory::bki(jcj, 4096) {
        Ok(p) => p,
        Err(aa) => {
            crate::serial_println!("[APIC] Failed to map LAPIC MMIO at {:#x}: {}", jcj, aa);
            return false;
        }
    };
    UL_.store(gkr, Ordering::SeqCst);
    
    crate::serial_println!("[APIC] LAPIC at phys={:#x}, virt={:#x}", jcj, gkr);
    
    
    unsafe { rxx(); }
    
    
    npv();
    
    let ad = ett();
    let dk = unsafe { jck(ADY_) } & 0xFF;
    crate::serial_println!("[APIC] LAPIC enabled: id={}, version={:#x}", ad, dk);
    
    
    let dcy = qvm();
    QA_.store(dcy, Ordering::SeqCst);
    
    
    wlb();
    
    
    nfl();
    
    
    jro(10);
    
    AKR_.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[APIC] ✓ APIC fully initialized — preemptive scheduling enabled");
    true
}


pub fn eso() {
    let gkr = UL_.load(Ordering::Relaxed);
    if gkr == 0 {
        return;
    }
    
    npv();
    
    
    nfl();
    
    
    let dcy = QA_.load(Ordering::Relaxed);
    if dcy > 0 {
        
        jro(10);
    }
    
    let ad = ett();
    crate::serial_println!("[APIC] AP LAPIC enabled: id={}", ad);
}


pub fn zu() -> bool {
    AKR_.load(Ordering::Relaxed)
}


pub fn xgv() -> u64 {
    QA_.load(Ordering::Relaxed)
}





pub fn jmw(irq: u8, wj: u8) {
    if !zu() {
        return;
    }
    let jau = UG_.load(Ordering::Relaxed);
    if jau == 0 {
        crate::serial_println!("[APIC] Cannot route IRQ {}: IOAPIC not initialized", irq);
        return;
    }
    unsafe {
        
        let flags = AXF_ | AXE_;
        lfp(irq, wj, 0, flags);
    }
    crate::serial_println!("[APIC] Routed PCI IRQ {} → vector {} (level/low)", irq, wj);
}
