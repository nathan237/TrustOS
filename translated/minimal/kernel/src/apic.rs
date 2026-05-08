













use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};





const AFQ_: u32         = 0x020;  
const AFS_: u32    = 0x030;  
const VV_: u32        = 0x080;  
const AFO_: u32        = 0x0B0;  
const OU_: u32        = 0x0F0;  
const BAI_: u32     = 0x300;  
const BAG_: u32     = 0x310;  
const IX_: u32  = 0x320;  
const CGT_: u32  = 0x350;  
const BAM_: u32  = 0x360;  
const AFP_: u32  = 0x370;  
const LG_: u32  = 0x380;  
const AFR_: u32  = 0x390;  
const OV_: u32  = 0x3E0;  


const CYK_: u32 = 1 << 8;
const CXT_: u32  = 0xFF;


const DBY_: u32   = 1 << 17;
const AKR_: u32     = 1 << 16;


const BJH_: u32     = 0x03; 





const DVC_: u32       = 0x00;
const CFP_: u32      = 0x01;
const AZH_: u32   = 0x10; 


const CFN_: u64            = 1 << 16;
const AZG_: u64   = 1 << 15;
const AZF_: u64        = 1 << 13;








pub const AKS_: u8      = 48;

pub const CXS_: u8      = 0xFF;

pub const CFY_: u8        = 0xFE;



pub const AFH_: u8          = 49;


pub const VS_: u8   = AFH_ + 1;  

pub const WS_: u8      = AFH_ + 12; 

pub const HZ_: u8     = 62;





static AML_: AtomicBool = AtomicBool::new(false);
static VU_: AtomicU64 = AtomicU64::new(0);
static VP_: AtomicU64 = AtomicU64::new(0);


static QX_: AtomicU64 = AtomicU64::new(0);






#[inline]
unsafe fn esh(offset: u32) -> u32 {
    let base = VU_.load(Ordering::Relaxed);
    core::ptr::read_volatile((base + offset as u64) as *const u32)
}


#[inline]
unsafe fn ajz(offset: u32, value: u32) {
    let base = VU_.load(Ordering::Relaxed);
    core::ptr::write_volatile((base + offset as u64) as *mut u32, value);
}


pub fn bng() {
    unsafe {
        ajz(AFO_, 0);
    }
}


pub fn lapic_id() -> u32 {
    unsafe { esh(AFQ_) >> 24 }
}


fn hvp() {
    unsafe {
        
        let svr = esh(OU_);
        ajz(OU_, svr | CYK_ | CXT_);
        
        
        ajz(VV_, 0);
    }
}



const UH_: u64 = 1000;



fn kgz() -> u64 {
    unsafe {
        
        ajz(OV_, BJH_);
        ajz(IX_, AKR_);
        
        
        ajz(LG_, 0xFFFF_FFFF);
        
        
        crate::cpu::tsc::hq(10);
        
        
        let ck = esh(AFR_);
        let bb = 0xFFFF_FFFFu64 - ck as u64;
        
        
        ajz(IX_, AKR_);
        
        
        let bef = bb / 10;
        
        if bef == 0 {
            crate::serial_println!("[APIC] WARNING: PIT calibration failed (elapsed={}), using fallback {} ticks/ms", bb, UH_);
            return UH_;
        }
        
        crate::serial_println!("[APIC] Timer calibrated: {} ticks/ms ({} ticks in 10ms)", bef, bb);
        bef
    }
}



pub fn fbp(interval_ms: u64) {
    let mut bef = QX_.load(Ordering::Relaxed);
    if bef == 0 {
        crate::serial_println!("[APIC] WARNING: Timer not calibrated, using fallback {} ticks/ms", UH_);
        bef = UH_;
        QX_.store(bef, Ordering::SeqCst);
    }
    
    let count = bef * interval_ms;
    
    unsafe {
        
        ajz(OV_, BJH_);
        
        
        ajz(IX_, DBY_ | AKS_ as u32);
        
        
        ajz(LG_, count as u32);
    }
    
    crate::serial_println!("[APIC] Timer started: {}ms interval, count={}", interval_ms, count);
}


pub fn jiy() {
    unsafe {
        ajz(IX_, AKR_);
        ajz(LG_, 0);
    }
}


pub fn gtx(target_apic_id: u32, vector: u8) {
    unsafe {
        
        ajz(BAG_, target_apic_id << 24);
        
        ajz(BAI_, vector as u32);
    }
}


pub fn jel(vector: u8) {
    unsafe {
        ajz(BAG_, 0);
        
        ajz(BAI_, (0b11 << 18) | vector as u32);
    }
}






unsafe fn gdl(reg: u32, value: u32) {
    let base = VP_.load(Ordering::Relaxed);
    
    core::ptr::write_volatile(base as *mut u32, reg);
    
    core::ptr::write_volatile((base + 0x10) as *mut u32, value);
}


unsafe fn ihl(reg: u32) -> u32 {
    let base = VP_.load(Ordering::Relaxed);
    core::ptr::write_volatile(base as *mut u32, reg);
    core::ptr::read_volatile((base + 0x10) as *const u32)
}






unsafe fn gdk(irq: u8, vector: u8, dest_apic: u8, flags: u64) {
    let dxo = AZH_ + (irq as u32) * 2;
    let oee = dxo + 1;
    
    
    let lqs = (vector as u64) | flags;
    
    let lqq = (dest_apic as u64) << 24;
    
    gdl(dxo, lqs as u32);
    gdl(oee, lqq as u32);
}


unsafe fn mrp() -> u8 {
    let tu = ihl(CFP_);
    ((tu >> 16) & 0xFF) as u8
}







fn oqk() {
    let ctg = match crate::acpi::rk() {
        Some(info) => info,
        None => {
            crate::serial_println!("[APIC] WARNING: No ACPI info, cannot set up I/O APIC");
            return;
        }
    };
    
    if ctg.io_apics.is_empty() {
        crate::serial_println!("[APIC] WARNING: No I/O APIC found in MADT");
        return;
    }
    
    let ioapic = &ctg.io_apics[0];
    
    let mrq = match crate::memory::yv(ioapic.address, 4096) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[APIC] Failed to map I/O APIC MMIO at {:#x}: {}", ioapic.address, e);
            return;
        }
    };
    VP_.store(mrq, Ordering::SeqCst);
    
    unsafe {
        let max_entries = mrp();
        crate::serial_println!("[APIC] I/O APIC id={}, addr={:#x}, GSI base={}, max_entries={}",
            ioapic.id, ioapic.address, ioapic.gsi_base, max_entries);
        
        
        for i in 0..=max_entries {
            let dxo = AZH_ + (i as u32) * 2;
            let lo = ihl(dxo);
            gdl(dxo, lo | CFN_ as u32);
        }
        
        
        let mut gel: u8 = 1;
        let mut iiy: u64 = 0; 
        
        
        for ovr in &ctg.int_overrides {
            if ovr.source == 1 {
                gel = ovr.gsi as u8;
                iiy = itc(ovr);
                crate::serial_println!("[APIC] Keyboard IRQ override: ISA 1 → GSI {}", ovr.gsi);
            }
        }
        gdk(gel, VS_, 0, iiy);
        crate::serial_println!("[APIC] Routed keyboard: IRQ {} → vector {}", gel, VS_);
        
        
        let mut gie: u8 = 12;
        let mut ioj: u64 = 0;
        
        for ovr in &ctg.int_overrides {
            if ovr.source == 12 {
                gie = ovr.gsi as u8;
                ioj = itc(ovr);
                crate::serial_println!("[APIC] Mouse IRQ override: ISA 12 → GSI {}", ovr.gsi);
            }
        }
        gdk(gie, WS_, 0, ioj);
        crate::serial_println!("[APIC] Routed mouse: IRQ {} → vector {}", gie, WS_);
    }
}


fn itc(ovr: &crate::acpi::madt::Kb) -> u64 {
    let mut flags: u64 = 0;
    
    
    match ovr.polarity {
        0 => {} 
        1 => {} 
        3 => flags |= AZF_,
        _ => {}
    }
    
    
    match ovr.trigger {
        0 => {} 
        1 => {} 
        3 => flags |= AZG_,
        _ => {}
    }
    
    flags
}





unsafe fn ley() {
    use x86_64::instructions::port::Port;
    
    
    let mut nuj = Port::<u8>::new(0x21);
    let mut nuk = Port::<u8>::new(0xA1);
    
    nuj.write(0xFF);
    nuk.write(0xFF);
    
    crate::serial_println!("[APIC] Legacy PIC disabled (all IRQs masked)");
}



fn hnh() {
    let info = match crate::acpi::rk() {
        Some(i) => i,
        None => return,
    };
    
    if info.local_apic_nmis.is_empty() {
        
        unsafe {
            
            ajz(BAM_, 0x0400);
        }
        crate::serial_println!("[APIC] NMI: default LINT1=NMI (no MADT entries)");
        return;
    }
    
    for ayo in &info.local_apic_nmis {
        
        
        let myu = if ayo.lint == 0 { CGT_ } else { BAM_ };
        
        
        let mut etl: u32 = 0x0400; 
        
        
        if ayo.polarity == 3 {
            etl |= 1 << 13; 
        }
        
        
        if ayo.trigger == 3 {
            etl |= 1 << 15; 
        }
        
        unsafe { ajz(myu, etl); }
        crate::serial_println!("[APIC] NMI: LINT{} = NMI (pol={}, trig={}, lvt={:#x})",
            ayo.lint, ayo.polarity, ayo.trigger, etl);
    }
}











pub fn init() -> bool {
    let esg = crate::acpi::ggc();
    if esg == 0 {
        crate::serial_println!("[APIC] No LAPIC address from ACPI, staying on PIC");
        return false;
    }
    
    
    
    let dag = match crate::memory::yv(esg, 4096) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[APIC] Failed to map LAPIC MMIO at {:#x}: {}", esg, e);
            return false;
        }
    };
    VU_.store(dag, Ordering::SeqCst);
    
    crate::serial_println!("[APIC] LAPIC at phys={:#x}, virt={:#x}", esg, dag);
    
    
    unsafe { ley(); }
    
    
    hvp();
    
    let id = lapic_id();
    let version = unsafe { esh(AFS_) } & 0xFF;
    crate::serial_println!("[APIC] LAPIC enabled: id={}, version={:#x}", id, version);
    
    
    let bef = kgz();
    QX_.store(bef, Ordering::SeqCst);
    
    
    oqk();
    
    
    hnh();
    
    
    fbp(10);
    
    AML_.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[APIC] ✓ APIC fully initialized — preemptive scheduling enabled");
    true
}


pub fn cau() {
    let dag = VU_.load(Ordering::Relaxed);
    if dag == 0 {
        return;
    }
    
    hvp();
    
    
    hnh();
    
    
    let bef = QX_.load(Ordering::Relaxed);
    if bef > 0 {
        
        fbp(10);
    }
    
    let id = lapic_id();
    crate::serial_println!("[APIC] AP LAPIC enabled: id={}", id);
}


pub fn lq() -> bool {
    AML_.load(Ordering::Relaxed)
}


pub fn gyq() -> u64 {
    QX_.load(Ordering::Relaxed)
}





pub fn eyz(irq: u8, vector: u8) {
    if !lq() {
        return;
    }
    let erc = VP_.load(Ordering::Relaxed);
    if erc == 0 {
        crate::serial_println!("[APIC] Cannot route IRQ {}: IOAPIC not initialized", irq);
        return;
    }
    unsafe {
        
        let flags = AZG_ | AZF_;
        gdk(irq, vector, 0, flags);
    }
    crate::serial_println!("[APIC] Routed PCI IRQ {} → vector {} (level/low)", irq, vector);
}
