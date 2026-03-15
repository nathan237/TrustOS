//! CPU Security Features
//!
//! Enables hardware security features like SMAP, SMEP, and others.

/// CPU features detected via CPUID
pub struct CpuSecurityFeatures {
    pub smep: bool,  // Supervisor Mode Execution Prevention
    pub smap: bool,  // Supervisor Mode Access Prevention
    pub umip: bool,  // User-Mode Instruction Prevention
    pub nx: bool,    // No-Execute bit support
}

impl CpuSecurityFeatures {
    /// Detect available security features
    pub fn detect() -> Self {
        let mut features = Self {
            smep: false,
            smap: false,
            umip: false,
            nx: false,
        };
        
        #[cfg(target_arch = "x86_64")]
        {
            // Check CPUID leaf 7, subleaf 0 for SMEP/SMAP/UMIP
            let cpuid_7 = unsafe { core::arch::x86_64::__cpuid_count(7, 0) };
            
            features.smep = (cpuid_7.ebx & (1 << 7)) != 0;
            features.smap = (cpuid_7.ebx & (1 << 20)) != 0;
            features.umip = (cpuid_7.ecx & (1 << 2)) != 0;
            
            // Check CPUID leaf 0x80000001 for NX support
            let cpuid_ext = unsafe { core::arch::x86_64::__cpuid(0x80000001) };
            features.nx = (cpuid_ext.edx & (1 << 20)) != 0;
        }
        
        features
    }
}

/// CR4 bits for security features
mod cr4 {
    pub const SMEP: u64 = 1 << 20;  // Supervisor Mode Execution Prevention
    pub const SMAP: u64 = 1 << 21;  // Supervisor Mode Access Prevention
    pub const UMIP: u64 = 1 << 11;  // User-Mode Instruction Prevention
}

/// Enable SMEP (Supervisor Mode Execution Prevention)
/// Prevents kernel from executing user-space code
pub fn enable_smep() -> bool {
    let features = CpuSecurityFeatures::detect();
    
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

/// Enable SMAP (Supervisor Mode Access Prevention)
/// Prevents kernel from accessing user-space memory without explicit permission
pub fn enable_smap() -> bool {
    let features = CpuSecurityFeatures::detect();
    
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

/// Enable UMIP (User-Mode Instruction Prevention)
/// Prevents user-space from using SGDT, SIDT, SLDT, SMSW, STR instructions
pub fn enable_umip() -> bool {
    let features = CpuSecurityFeatures::detect();
    
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

/// Temporarily disable SMAP to access user memory
/// Returns a guard that re-enables SMAP when dropped
#[inline(always)]
pub fn disable_smap_for_user_access() -> SmapGuard {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        // STAC - Set AC flag (allows supervisor access to user pages)
        core::arch::asm!("stac", options(nomem, nostack));
    }
    SmapGuard
}

/// Guard that re-enables SMAP when dropped
pub struct SmapGuard;

impl Drop for SmapGuard {
    #[inline(always)]
    fn drop(&mut self) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            // CLAC - Clear AC flag (re-enables SMAP protection)
            core::arch::asm!("clac", options(nomem, nostack));
        }
    }
}

/// Initialize all available CPU security features
pub fn init() -> CpuSecurityFeatures {
    let features = CpuSecurityFeatures::detect();
    
    crate::log!("[SECURITY] CPU features: SMEP={}, SMAP={}, UMIP={}, NX={}",
        features.smep, features.smap, features.umip, features.nx);
    
    // Enable features (but don't fail if not available)
    // Note: SMAP requires careful handling - we enable it but provide
    // disable_smap_for_user_access() for syscalls that need to copy user data
    
    if features.smep {
        enable_smep();
    }
    
    // SMAP disabled for now - requires updating all user memory accesses
    // to use the STAC/CLAC pattern. Enable once syscalls are updated.
    // if features.smap {
    //     enable_smap();
    // }
    
    if features.umip {
        enable_umip();
    }
    
    features
}
