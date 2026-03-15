




pub struct CpuSecurityFeatures {
    pub cia: bool,  
    pub cul: bool,  
    pub ddd: bool,  
    pub vt: bool,    
}

impl CpuSecurityFeatures {
    
    pub fn dgf() -> Self {
        let mut features = Self {
            cia: false,
            cul: false,
            ddd: false,
            vt: false,
        };
        
        #[cfg(target_arch = "x86_64")]
        {
            
            let dfh = unsafe { core::arch::x86_64::qbf(7, 0) };
            
            features.cia = (dfh.ebx & (1 << 7)) != 0;
            features.cul = (dfh.ebx & (1 << 20)) != 0;
            features.ddd = (dfh.ecx & (1 << 2)) != 0;
            
            
            let rqa = unsafe { core::arch::x86_64::ddo(0x80000001) };
            features.vt = (rqa.edx & (1 << 20)) != 0;
        }
        
        features
    }
}


mod cr4 {
    pub const Cmb: u64 = 1 << 20;  
    pub const Clz: u64 = 1 << 21;  
    pub const Cos: u64 = 1 << 11;  
}



pub fn npw() -> bool {
    let features = CpuSecurityFeatures::dgf();
    
    if !features.cia {
        crate::log_warn!("[SECURITY] SMEP not supported by CPU");
        return false;
    }
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let cr4: u64;
        core::arch::asm!("mov {}, cr4", bd(reg) cr4);
        core::arch::asm!("mov cr4, {}", in(reg) cr4 | cr4::Cmb);
    }
    #[cfg(not(target_arch = "x86_64"))]
    return false;
    
    crate::log_debug!("[SECURITY] SMEP enabled");
    true
}



pub fn sle() -> bool {
    let features = CpuSecurityFeatures::dgf();
    
    if !features.cul {
        crate::log_warn!("[SECURITY] SMAP not supported by CPU");
        return false;
    }
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let cr4: u64;
        core::arch::asm!("mov {}, cr4", bd(reg) cr4);
        core::arch::asm!("mov cr4, {}", in(reg) cr4 | cr4::Clz);
    }
    #[cfg(not(target_arch = "x86_64"))]
    return false;
    
    crate::log_debug!("[SECURITY] SMAP enabled");
    true
}



pub fn npx() -> bool {
    let features = CpuSecurityFeatures::dgf();
    
    if !features.ddd {
        crate::log_warn!("[SECURITY] UMIP not supported by CPU");
        return false;
    }
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let cr4: u64;
        core::arch::asm!("mov {}, cr4", bd(reg) cr4);
        core::arch::asm!("mov cr4, {}", in(reg) cr4 | cr4::Cos);
    }
    #[cfg(not(target_arch = "x86_64"))]
    return false;
    
    crate::log_debug!("[SECURITY] UMIP enabled");
    true
}



#[inline(always)]
pub fn rxy() -> Ayx {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        
        core::arch::asm!("stac", options(nomem, nostack));
    }
    Ayx
}


pub struct Ayx;

impl Drop for Ayx {
    #[inline(always)]
    fn drop(&mut self) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            
            core::arch::asm!("clac", options(nomem, nostack));
        }
    }
}


pub fn init() -> CpuSecurityFeatures {
    let features = CpuSecurityFeatures::dgf();
    
    crate::log!("[SECURITY] CPU features: SMEP={}, SMAP={}, UMIP={}, NX={}",
        features.cia, features.cul, features.ddd, features.vt);
    
    
    
    
    
    if features.cia {
        npw();
    }
    
    
    
    
    
    
    
    if features.ddd {
        npx();
    }
    
    features
}
