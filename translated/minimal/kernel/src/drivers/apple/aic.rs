








































use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use spin::Mutex;






const BME_: usize         = 0x0004;

const BLY_: usize        = 0x2004;

const BMI_: usize     = 0x2008;

const DGD_: usize = 0x200C;

const BMJ_: usize = 0x2010;

const BMH_: usize      = 0x2014;

const DGE_: usize = 0x2008;

const BMK_: usize = 0x200C;


const AMD_: usize = 0x3000;

const AMC_: usize   = 0x4000;

const BML_: usize   = 0x4080;

const DGG_: usize     = 0x4100;

const DGF_: usize     = 0x4180;



const BMF_: u32 = 0xFFFF;

const BMG_: u32 = 28;



const BMD_: u32 = 16;

const BLZ_: u32   = 0xFFFF;

const BMC_: u32  = 0;

const BMB_: u32   = 1;

const BMA_: u32   = 4;



const AMB_: u32 = 1 << 31;

const AMA_: u32 = 1 << 0;






const WH_: usize = 1024;


pub type Aah = fn(irq: u32);


#[derive(Clone, Copy)]
struct IrqConfig {
    
    handler: Option<Aah>,
    
    target_cpu: u8,
    
    enabled: bool,
    
    count: u64,
    
    name: &'static str,
}

impl Default for IrqConfig {
    fn default() -> Self {
        Self {
            handler: None,
            target_cpu: 0,
            enabled: false,
            count: 0,
            name: "unknown",
        }
    }
}


pub struct Wi {
    
    base: u64,
    
    num_irqs: u32,
    
    version: u32,
    
    num_cpus: u32,
    
    irqs: [IrqConfig; WH_],
    
    total_irqs: u64,
    
    total_ipis: u64,
    
    initialized: bool,
}

static Fh: Mutex<Option<Wi>> = Mutex::new(None);
static AAC_: AtomicBool = AtomicBool::new(false);






#[inline(always)]
unsafe fn fgp(base: u64, offset: usize) -> u32 {
    let addr = (base as usize + offset) as *const u32;
    ptr::read_volatile(addr)
}


#[inline(always)]
unsafe fn bxi(base: u64, offset: usize, val: u32) {
    let addr = (base as usize + offset) as *mut u32;
    ptr::write_volatile(addr, val);
}













pub unsafe fn init(base: u64, num_cpus: u32) -> Result<(), &'static str> {
    crate::serial_println!("[AIC] Initializing Apple Interrupt Controller @ {:#x}", base);
    
    
    let info = fgp(base, BME_);
    let num_irqs = info & BMF_;
    let version = (info >> BMG_) & 0xF;
    
    crate::serial_println!("[AIC] Version: AICv{}", version + 1);
    crate::serial_println!("[AIC] Hardware IRQs: {}", num_irqs);
    crate::serial_println!("[AIC] CPUs: {}", num_cpus);
    
    if num_irqs == 0 || num_irqs as usize > WH_ {
        return Err("AIC: invalid IRQ count from hardware");
    }
    
    let mut aic = Wi {
        base,
        num_irqs,
        version,
        num_cpus,
        irqs: [IrqConfig::default(); WH_],
        total_irqs: 0,
        total_ipis: 0,
        initialized: false,
    };
    
    
    let nly = (num_irqs + 31) / 32;
    for w in 0..nly as usize {
        bxi(base, AMC_ + w * 4, 0xFFFFFFFF);
    }
    
    
    for i in 0..num_irqs as usize {
        bxi(base, AMD_ + i * 4, 1 << 0); 
    }
    
    
    bxi(base, BMJ_, AMB_ | AMA_);
    
    aic.initialized = true;
    
    *Fh.lock() = Some(aic);
    AAC_.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[AIC] Initialization complete — all IRQs masked, IPIs enabled");
    Ok(())
}


pub fn qtl(irq: u32, name: &'static str, handler: Aah) -> Result<(), &'static str> {
    let mut jg = Fh.lock();
    let aic = jg.as_mut().ok_or("AIC not initialized")?;
    
    if irq >= aic.num_irqs {
        return Err("IRQ number out of range");
    }
    
    let config = &mut aic.irqs[irq as usize];
    config.handler = Some(handler);
    config.name = name;
    
    crate::serial_println!("[AIC] Registered IRQ {} → {}", irq, name);
    Ok(())
}


pub fn fum(irq: u32) -> Result<(), &'static str> {
    let mut jg = Fh.lock();
    let aic = jg.as_mut().ok_or("AIC not initialized")?;
    
    if irq >= aic.num_irqs {
        return Err("IRQ number out of range");
    }
    
    let fx = irq / 32;
    let bf = irq % 32;
    
    unsafe {
        bxi(aic.base, BML_ + fx as usize * 4, 1 << bf);
    }
    
    aic.irqs[irq as usize].enabled = true;
    Ok(())
}


pub fn hsk(irq: u32) -> Result<(), &'static str> {
    let mut jg = Fh.lock();
    let aic = jg.as_mut().ok_or("AIC not initialized")?;
    
    if irq >= aic.num_irqs {
        return Err("IRQ number out of range");
    }
    
    let fx = irq / 32;
    let bf = irq % 32;
    
    unsafe {
        bxi(aic.base, AMC_ + fx as usize * 4, 1 << bf);
    }
    
    aic.irqs[irq as usize].enabled = false;
    Ok(())
}


pub fn qwc(irq: u32, cpu: u32) -> Result<(), &'static str> {
    let mut jg = Fh.lock();
    let aic = jg.as_mut().ok_or("AIC not initialized")?;
    
    if irq >= aic.num_irqs {
        return Err("IRQ number out of range");
    }
    if cpu >= aic.num_cpus {
        return Err("CPU number out of range");
    }
    
    unsafe {
        bxi(aic.base, AMD_ + irq as usize * 4, 1 << cpu);
    }
    
    aic.irqs[irq as usize].target_cpu = cpu as u8;
    Ok(())
}


pub fn gtx(target_cpu: u32) -> Result<(), &'static str> {
    let jg = Fh.lock();
    let aic = jg.as_ref().ok_or("AIC not initialized")?;
    
    if target_cpu >= aic.num_cpus {
        return Err("Target CPU out of range");
    }
    
    unsafe {
        
        
        
        
        bxi(aic.base, BMK_, 1 << target_cpu);
    }
    
    Ok(())
}







pub unsafe fn qke() -> bool {
    if !AAC_.load(Ordering::SeqCst) {
        return false;
    }
    
    let mut jg = Fh.lock();
    let aic = match jg.as_mut() {
        Some(a) => a,
        None => return false,
    };
    
    
    let event = fgp(aic.base, BLY_);
    let event_type = (event >> BMD_) & 0xFFFF;
    let dow = event & BLZ_;
    
    match event_type {
        BMC_ => {
            
            false
        }
        
        BMB_ => {
            
            aic.total_irqs += 1;
            
            if (dow as usize) < WH_ {
                let config = &mut aic.irqs[dow as usize];
                config.count += 1;
                
                if let Some(handler) = config.handler {
                    
                    let miw = handler;
                    drop(jg);
                    miw(dow);
                    return true;
                }
            }
            
            
            crate::serial_println!("[AIC] Unhandled IRQ {}", dow);
            true
        }
        
        BMA_ => {
            
            aic.total_ipis += 1;
            
            
            let gdn = fgp(aic.base, BMI_);
            
            
            bxi(aic.base, BMH_, gdn);
            
            if gdn & AMB_ != 0 {
                crate::serial_println!("[AIC] Self-IPI received");
            }
            if gdn & AMA_ != 0 {
                
                
                
                
                crate::serial_println!("[AIC] Cross-CPU IPI received");
            }
            
            true
        }
        
        _ => {
            crate::serial_println!("[AIC] Unknown event type {} num {}", event_type, dow);
            true
        }
    }
}


pub fn is_initialized() -> bool {
    AAC_.load(Ordering::SeqCst)
}


pub fn jis() -> String {
    let jg = Fh.lock();
    match jg.as_ref() {
        None => String::from("AIC: not initialized"),
        Some(aic) => {
            let enabled = aic.irqs[..aic.num_irqs as usize]
                .iter()
                .filter(|c| c.enabled)
                .count();
            format!(
                "AIC v{} @ {:#x}: {} IRQs ({} enabled), {} total handled, {} IPIs",
                aic.version + 1, aic.base, aic.num_irqs,
                enabled, aic.total_irqs, aic.total_ipis
            )
        }
    }
}


pub fn qnn() -> Vec<(u32, &'static str, bool, u64)> {
    let jg = Fh.lock();
    match jg.as_ref() {
        None => Vec::new(),
        Some(aic) => {
            let mut result = Vec::new();
            for i in 0..aic.num_irqs as usize {
                let config = &aic.irqs[i];
                if config.handler.is_some() || config.enabled {
                    result.push((
                        i as u32,
                        config.name,
                        config.enabled,
                        config.count,
                    ));
                }
            }
            result
        }
    }
}
