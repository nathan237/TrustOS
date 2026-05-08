




pub struct CpuSecurityFeatures {
    pub smep: bool,  
    pub smap: bool,  
    pub umip: bool,  
    pub nx: bool,    
}

impl CpuSecurityFeatures {
    
    pub fn bfx() -> Self {
        let mut features = Self {
            smep: false,
            smap: false,
            umip: false,
            nx: false,
        };
        
        #[cfg(target_arch = "x86_64")]
        {
            
            let bfp = unsafe { core::arch::x86_64::__cpuid_count(7, 0) };
            
            features.smep = (bfp.ebx & (1 << 7)) != 0;
            features.smap = (bfp.ebx & (1 << 20)) != 0;
            features.umip = (bfp.ecx & (1 << 2)) != 0;
            
            
            let kyw = unsafe { core::arch::x86_64::__cpuid(0x80000001) };
            features.nx = (kyw.edx & (1 << 20)) != 0;
        }
        
        features
    }
}


mod cr4 {
    pub const SMEP: u64 = 1 << 20;  
    pub const SMAP: u64 = 1 << 21;  
    pub const UMIP: u64 = 1 << 11;  
}



pub fn hvq() -> bool {
    let features = CpuSecurityFeatures::bfx();
    
    if !features.smep {
        crate::log_warn!("[SECURITY] SMEP not supported by CPU");
        return false;
    }
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        core::arch::asm!("mov cr4, {}", in(reg) cr4 | cr4::SMEP);
    }
    #[cfg(not(target_arch = "x86_64"))]
    return false;
    
    crate::log_debug!("[SECURITY] SMEP enabled");
    true
}



pub fn lpu() -> bool {
    let features = CpuSecurityFeatures::bfx();
    
    if !features.smap {
        crate::log_warn!("[SECURITY] SMAP not supported by CPU");
        return false;
    }
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        core::arch::asm!("mov cr4, {}", in(reg) cr4 | cr4::SMAP);
    }
    #[cfg(not(target_arch = "x86_64"))]
    return false;
    
    crate::log_debug!("[SECURITY] SMAP enabled");
    true
}



pub fn hvr() -> bool {
    let features = CpuSecurityFeatures::bfx();
    
    if !features.umip {
        crate::log_warn!("[SECURITY] UMIP not supported by CPU");
        return false;
    }
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        core::arch::asm!("mov cr4, {}", in(reg) cr4 | cr4::UMIP);
    }
    #[cfg(not(target_arch = "x86_64"))]
    return false;
    
    crate::log_debug!("[SECURITY] UMIP enabled");
    true
}



#[inline(always)]
pub fn lez() -> Vc {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        
        core::arch::asm!("stac", options(nomem, nostack));
    }
    Vc
}


pub struct Vc;

impl Drop for Vc {
    #[inline(always)]
    fn drop(&mut self) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            
            core::arch::asm!("clac", options(nomem, nostack));
        }
    }
}


pub fn init() -> CpuSecurityFeatures {
    let features = CpuSecurityFeatures::bfx();
    
    crate::log!("[SECURITY] CPU features: SMEP={}, SMAP={}, UMIP={}, NX={}",
        features.smep, features.smap, features.umip, features.nx);
    
    
    
    
    
    if features.smep {
        hvq();
    }
    
    
    
    
    
    
    
    if features.umip {
        hvr();
    }
    
    features
}
