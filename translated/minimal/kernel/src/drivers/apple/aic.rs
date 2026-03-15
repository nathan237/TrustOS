








































use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use spin::Mutex;






const BJU_: usize         = 0x0004;

const BJO_: usize        = 0x2004;

const BJY_: usize     = 0x2008;

const DCI_: usize = 0x200C;

const BJZ_: usize = 0x2010;

const BJX_: usize      = 0x2014;

const DCJ_: usize = 0x2008;

const BKA_: usize = 0x200C;


const AKJ_: usize = 0x3000;

const AKI_: usize   = 0x4000;

const BKB_: usize   = 0x4080;

const DCL_: usize     = 0x4100;

const DCK_: usize     = 0x4180;



const BJV_: u32 = 0xFFFF;

const BJW_: u32 = 28;



const BJT_: u32 = 16;

const BJP_: u32   = 0xFFFF;

const BJS_: u32  = 0;

const BJR_: u32   = 1;

const BJQ_: u32   = 4;



const AKH_: u32 = 1 << 31;

const AKG_: u32 = 1 << 0;






const UY_: usize = 1024;


pub type Bkd = fn(irq: u32);


#[derive(Clone, Copy)]
struct IrqConfig {
    
    cfd: Option<Bkd>,
    
    cih: u8,
    
    iq: bool,
    
    az: u64,
    
    j: &'static str,
}

impl Default for IrqConfig {
    fn default() -> Self {
        Self {
            cfd: None,
            cih: 0,
            iq: false,
            az: 0,
            j: "unknown",
        }
    }
}


pub struct Bbv {
    
    ar: u64,
    
    cln: u32,
    
    dk: u32,
    
    bcc: u32,
    
    eta: [IrqConfig; UY_],
    
    blm: u64,
    
    mmh: u64,
    
    jr: bool,
}

static Mq: Mutex<Option<Bbv>> = Mutex::new(None);
static YX_: AtomicBool = AtomicBool::new(false);






#[inline(always)]
unsafe fn jzu(ar: u64, l: usize) -> u32 {
    let ag = (ar as usize + l) as *const u32;
    ptr::read_volatile(ag)
}


#[inline(always)]
unsafe fn ema(ar: u64, l: usize, ap: u32) {
    let ag = (ar as usize + l) as *mut u32;
    ptr::write_volatile(ag, ap);
}













pub unsafe fn init(ar: u64, bcc: u32) -> Result<(), &'static str> {
    crate::serial_println!("[AIC] Initializing Apple Interrupt Controller @ {:#x}", ar);
    
    
    let co = jzu(ar, BJU_);
    let cln = co & BJV_;
    let dk = (co >> BJW_) & 0xF;
    
    crate::serial_println!("[AIC] Version: AICv{}", dk + 1);
    crate::serial_println!("[AIC] Hardware IRQs: {}", cln);
    crate::serial_println!("[AIC] CPUs: {}", bcc);
    
    if cln == 0 || cln as usize > UY_ {
        return Err("AIC: invalid IRQ count from hardware");
    }
    
    let mut aic = Bbv {
        ar,
        cln,
        dk,
        bcc,
        eta: [IrqConfig::default(); UY_],
        blm: 0,
        mmh: 0,
        jr: false,
    };
    
    
    let uwp = (cln + 31) / 32;
    for d in 0..uwp as usize {
        ema(ar, AKI_ + d * 4, 0xFFFFFFFF);
    }
    
    
    for a in 0..cln as usize {
        ema(ar, AKJ_ + a * 4, 1 << 0); 
    }
    
    
    ema(ar, BJZ_, AKH_ | AKG_);
    
    aic.jr = true;
    
    *Mq.lock() = Some(aic);
    YX_.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[AIC] Initialization complete — all IRQs masked, IPIs enabled");
    Ok(())
}


pub fn zja(irq: u32, j: &'static str, cfd: Bkd) -> Result<(), &'static str> {
    let mut adb = Mq.lock();
    let aic = adb.as_mut().ok_or("AIC not initialized")?;
    
    if irq >= aic.cln {
        return Err("IRQ number out of range");
    }
    
    let config = &mut aic.eta[irq as usize];
    config.cfd = Some(cfd);
    config.j = j;
    
    crate::serial_println!("[AIC] Registered IRQ {} → {}", irq, j);
    Ok(())
}


pub fn kte(irq: u32) -> Result<(), &'static str> {
    let mut adb = Mq.lock();
    let aic = adb.as_mut().ok_or("AIC not initialized")?;
    
    if irq >= aic.cln {
        return Err("IRQ number out of range");
    }
    
    let od = irq / 32;
    let ga = irq % 32;
    
    unsafe {
        ema(aic.ar, BKB_ + od as usize * 4, 1 << ga);
    }
    
    aic.eta[irq as usize].iq = true;
    Ok(())
}


pub fn nlr(irq: u32) -> Result<(), &'static str> {
    let mut adb = Mq.lock();
    let aic = adb.as_mut().ok_or("AIC not initialized")?;
    
    if irq >= aic.cln {
        return Err("IRQ number out of range");
    }
    
    let od = irq / 32;
    let ga = irq % 32;
    
    unsafe {
        ema(aic.ar, AKI_ + od as usize * 4, 1 << ga);
    }
    
    aic.eta[irq as usize].iq = false;
    Ok(())
}


pub fn zng(irq: u32, cpu: u32) -> Result<(), &'static str> {
    let mut adb = Mq.lock();
    let aic = adb.as_mut().ok_or("AIC not initialized")?;
    
    if irq >= aic.cln {
        return Err("IRQ number out of range");
    }
    if cpu >= aic.bcc {
        return Err("CPU number out of range");
    }
    
    unsafe {
        ema(aic.ar, AKJ_ + irq as usize * 4, 1 << cpu);
    }
    
    aic.eta[irq as usize].cih = cpu as u8;
    Ok(())
}


pub fn mds(cih: u32) -> Result<(), &'static str> {
    let adb = Mq.lock();
    let aic = adb.as_ref().ok_or("AIC not initialized")?;
    
    if cih >= aic.bcc {
        return Err("Target CPU out of range");
    }
    
    unsafe {
        
        
        
        
        ema(aic.ar, BKA_, 1 << cih);
    }
    
    Ok(())
}







pub unsafe fn yvw() -> bool {
    if !YX_.load(Ordering::SeqCst) {
        return false;
    }
    
    let mut adb = Mq.lock();
    let aic = match adb.as_mut() {
        Some(q) => q,
        None => return false,
    };
    
    
    let id = jzu(aic.ar, BJO_);
    let bqo = (id >> BJT_) & 0xFFFF;
    let hij = id & BJP_;
    
    match bqo {
        BJS_ => {
            
            false
        }
        
        BJR_ => {
            
            aic.blm += 1;
            
            if (hij as usize) < UY_ {
                let config = &mut aic.eta[hij as usize];
                config.az += 1;
                
                if let Some(cfd) = config.cfd {
                    
                    let tlv = cfd;
                    drop(adb);
                    tlv(hij);
                    return true;
                }
            }
            
            
            crate::serial_println!("[AIC] Unhandled IRQ {}", hij);
            true
        }
        
        BJQ_ => {
            
            aic.mmh += 1;
            
            
            let lfs = jzu(aic.ar, BJY_);
            
            
            ema(aic.ar, BJX_, lfs);
            
            if lfs & AKH_ != 0 {
                crate::serial_println!("[AIC] Self-IPI received");
            }
            if lfs & AKG_ != 0 {
                
                
                
                
                crate::serial_println!("[AIC] Cross-CPU IPI received");
            }
            
            true
        }
        
        _ => {
            crate::serial_println!("[AIC] Unknown event type {} num {}", bqo, hij);
            true
        }
    }
}


pub fn ky() -> bool {
    YX_.load(Ordering::SeqCst)
}


pub fn poq() -> String {
    let adb = Mq.lock();
    match adb.as_ref() {
        None => String::from("AIC: not initialized"),
        Some(aic) => {
            let iq = aic.eta[..aic.cln as usize]
                .iter()
                .hi(|r| r.iq)
                .az();
            format!(
                "AIC v{} @ {:#x}: {} IRQs ({} enabled), {} total handled, {} IPIs",
                aic.dk + 1, aic.ar, aic.cln,
                iq, aic.blm, aic.mmh
            )
        }
    }
}


pub fn zay() -> Vec<(u32, &'static str, bool, u64)> {
    let adb = Mq.lock();
    match adb.as_ref() {
        None => Vec::new(),
        Some(aic) => {
            let mut result = Vec::new();
            for a in 0..aic.cln as usize {
                let config = &aic.eta[a];
                if config.cfd.is_some() || config.iq {
                    result.push((
                        a as u32,
                        config.j,
                        config.iq,
                        config.az,
                    ));
                }
            }
            result
        }
    }
}
