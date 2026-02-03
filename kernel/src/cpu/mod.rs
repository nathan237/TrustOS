//! CPU Hardware Exploitation Module
//!
//! Full hardware capability detection and utilization:
//! - CPUID feature detection
//! - TSC high-precision timing (nanoseconds)
//! - AES-NI hardware acceleration
//! - SSE/AVX SIMD detection
//! - Multi-core/SMP support

pub mod features;
pub mod tsc;
pub mod simd;
pub mod smp;

use core::sync::atomic::{AtomicBool, Ordering};

/// Global CPU capabilities (initialized once at boot)
static CPU_INIT: AtomicBool = AtomicBool::new(false);
static mut CPU_CAPS: Option<CpuCapabilities> = None;

/// Complete CPU capabilities structure
#[derive(Debug, Clone)]
pub struct CpuCapabilities {
    // Vendor and model
    pub vendor: CpuVendor,
    pub family: u8,
    pub model: u8,
    pub stepping: u8,
    pub brand_string: [u8; 48],
    
    // Core features
    pub tsc: bool,           // Time Stamp Counter
    pub tsc_invariant: bool, // TSC doesn't change with power states
    pub tsc_deadline: bool,  // TSC deadline timer
    pub rdtscp: bool,        // RDTSCP instruction
    
    // SIMD
    pub sse: bool,
    pub sse2: bool,
    pub sse3: bool,
    pub ssse3: bool,
    pub sse4_1: bool,
    pub sse4_2: bool,
    pub avx: bool,
    pub avx2: bool,
    pub avx512f: bool,
    
    // Crypto acceleration
    pub aesni: bool,         // AES-NI instructions
    pub pclmulqdq: bool,     // Carryless multiplication (for GCM)
    pub sha_ext: bool,       // SHA-1/SHA-256 hardware
    pub rdrand: bool,        // Hardware RNG
    pub rdseed: bool,        // Hardware RNG seed
    
    // Security
    pub smep: bool,
    pub smap: bool,
    pub umip: bool,
    pub nx: bool,
    
    // Virtualization
    pub vmx: bool,           // Intel VT-x
    pub svm: bool,           // AMD-V
    
    // Multi-core
    pub max_logical_cpus: u8,
    pub max_physical_cpus: u8,
    pub apic_id: u8,
    
    // TSC frequency (calibrated)
    pub tsc_frequency_hz: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CpuVendor {
    Intel,
    Amd,
    Unknown,
}

impl CpuCapabilities {
    /// Detect all CPU capabilities via CPUID
    pub fn detect() -> Self {
        let mut caps = Self {
            vendor: CpuVendor::Unknown,
            family: 0,
            model: 0,
            stepping: 0,
            brand_string: [0; 48],
            tsc: false,
            tsc_invariant: false,
            tsc_deadline: false,
            rdtscp: false,
            sse: false,
            sse2: false,
            sse3: false,
            ssse3: false,
            sse4_1: false,
            sse4_2: false,
            avx: false,
            avx2: false,
            avx512f: false,
            aesni: false,
            pclmulqdq: false,
            sha_ext: false,
            rdrand: false,
            rdseed: false,
            smep: false,
            smap: false,
            umip: false,
            nx: false,
            vmx: false,
            svm: false,
            max_logical_cpus: 1,
            max_physical_cpus: 1,
            apic_id: 0,
            tsc_frequency_hz: 0,
        };
        
        // CPUID leaf 0: Vendor ID
        let cpuid_0 = unsafe { core::arch::x86_64::__cpuid(0) };
        let max_basic_leaf = cpuid_0.eax;
        
        // Vendor string from EBX, EDX, ECX
        let vendor_bytes = [
            cpuid_0.ebx.to_le_bytes(),
            cpuid_0.edx.to_le_bytes(),
            cpuid_0.ecx.to_le_bytes(),
        ];
        
        let mut vendor_str = [0u8; 12];
        vendor_str[0..4].copy_from_slice(&vendor_bytes[0]);
        vendor_str[4..8].copy_from_slice(&vendor_bytes[1]);
        vendor_str[8..12].copy_from_slice(&vendor_bytes[2]);
        
        caps.vendor = match &vendor_str {
            b"GenuineIntel" => CpuVendor::Intel,
            b"AuthenticAMD" => CpuVendor::Amd,
            _ => CpuVendor::Unknown,
        };
        
        // CPUID leaf 1: Feature flags
        if max_basic_leaf >= 1 {
            let cpuid_1 = unsafe { core::arch::x86_64::__cpuid(1) };
            
            // Family/Model/Stepping
            caps.stepping = (cpuid_1.eax & 0xF) as u8;
            caps.model = ((cpuid_1.eax >> 4) & 0xF) as u8;
            caps.family = ((cpuid_1.eax >> 8) & 0xF) as u8;
            
            // Extended model/family
            if caps.family == 0xF {
                caps.family += ((cpuid_1.eax >> 20) & 0xFF) as u8;
            }
            if caps.family >= 6 {
                caps.model += (((cpuid_1.eax >> 16) & 0xF) << 4) as u8;
            }
            
            // APIC ID
            caps.apic_id = ((cpuid_1.ebx >> 24) & 0xFF) as u8;
            caps.max_logical_cpus = ((cpuid_1.ebx >> 16) & 0xFF) as u8;
            
            // ECX features
            caps.sse3 = (cpuid_1.ecx & (1 << 0)) != 0;
            caps.pclmulqdq = (cpuid_1.ecx & (1 << 1)) != 0;
            caps.ssse3 = (cpuid_1.ecx & (1 << 9)) != 0;
            caps.sse4_1 = (cpuid_1.ecx & (1 << 19)) != 0;
            caps.sse4_2 = (cpuid_1.ecx & (1 << 20)) != 0;
            caps.aesni = (cpuid_1.ecx & (1 << 25)) != 0;
            caps.avx = (cpuid_1.ecx & (1 << 28)) != 0;
            caps.rdrand = (cpuid_1.ecx & (1 << 30)) != 0;
            caps.vmx = (cpuid_1.ecx & (1 << 5)) != 0;
            caps.tsc_deadline = (cpuid_1.ecx & (1 << 24)) != 0;
            
            // EDX features
            caps.tsc = (cpuid_1.edx & (1 << 4)) != 0;
            caps.sse = (cpuid_1.edx & (1 << 25)) != 0;
            caps.sse2 = (cpuid_1.edx & (1 << 26)) != 0;
        }
        
        // CPUID leaf 7: Extended features
        if max_basic_leaf >= 7 {
            let cpuid_7 = unsafe { core::arch::x86_64::__cpuid_count(7, 0) };
            
            caps.smep = (cpuid_7.ebx & (1 << 7)) != 0;
            caps.avx2 = (cpuid_7.ebx & (1 << 5)) != 0;
            caps.avx512f = (cpuid_7.ebx & (1 << 16)) != 0;
            caps.sha_ext = (cpuid_7.ebx & (1 << 29)) != 0;
            caps.rdseed = (cpuid_7.ebx & (1 << 18)) != 0;
            caps.smap = (cpuid_7.ebx & (1 << 20)) != 0;
            caps.umip = (cpuid_7.ecx & (1 << 2)) != 0;
        }
        
        // Extended CPUID
        let cpuid_ext_max = unsafe { core::arch::x86_64::__cpuid(0x80000000) };
        let max_ext_leaf = cpuid_ext_max.eax;
        
        // CPUID 0x80000001: Extended features
        if max_ext_leaf >= 0x80000001 {
            let cpuid_ext1 = unsafe { core::arch::x86_64::__cpuid(0x80000001) };
            
            caps.nx = (cpuid_ext1.edx & (1 << 20)) != 0;
            caps.rdtscp = (cpuid_ext1.edx & (1 << 27)) != 0;
            caps.svm = (cpuid_ext1.ecx & (1 << 2)) != 0;
        }
        
        // CPUID 0x80000007: Invariant TSC
        if max_ext_leaf >= 0x80000007 {
            let cpuid_ext7 = unsafe { core::arch::x86_64::__cpuid(0x80000007) };
            caps.tsc_invariant = (cpuid_ext7.edx & (1 << 8)) != 0;
        }
        
        // Brand string (CPUID 0x80000002-4)
        if max_ext_leaf >= 0x80000004 {
            for i in 0..3 {
                let cpuid_brand = unsafe { core::arch::x86_64::__cpuid(0x80000002 + i) };
                let offset = (i as usize) * 16;
                caps.brand_string[offset..offset+4].copy_from_slice(&cpuid_brand.eax.to_le_bytes());
                caps.brand_string[offset+4..offset+8].copy_from_slice(&cpuid_brand.ebx.to_le_bytes());
                caps.brand_string[offset+8..offset+12].copy_from_slice(&cpuid_brand.ecx.to_le_bytes());
                caps.brand_string[offset+12..offset+16].copy_from_slice(&cpuid_brand.edx.to_le_bytes());
            }
        }
        
        caps
    }
    
    /// Get brand string as &str
    pub fn brand(&self) -> &str {
        let end = self.brand_string.iter()
            .position(|&b| b == 0)
            .unwrap_or(48);
        core::str::from_utf8(&self.brand_string[..end])
            .unwrap_or("Unknown CPU")
            .trim()
    }
}

/// Initialize CPU module - call once at boot
pub fn init() {
    if CPU_INIT.swap(true, Ordering::SeqCst) {
        return; // Already initialized
    }
    
    // Detect capabilities
    let mut caps = CpuCapabilities::detect();
    
    // Calibrate TSC
    caps.tsc_frequency_hz = tsc::calibrate_tsc();
    
    // Store globally
    unsafe {
        CPU_CAPS = Some(caps.clone());
    }
    
    // Print capabilities
    crate::serial_println!("[CPU] {}", caps.brand());
    crate::serial_println!("[CPU] Vendor: {:?}, Family: {}, Model: {}", 
        caps.vendor, caps.family, caps.model);
    crate::serial_println!("[CPU] TSC: {} (invariant: {}, freq: {} MHz)", 
        caps.tsc, caps.tsc_invariant, caps.tsc_frequency_hz / 1_000_000);
    crate::serial_println!("[CPU] SIMD: SSE={} SSE2={} SSE4.2={} AVX={} AVX2={}", 
        caps.sse, caps.sse2, caps.sse4_2, caps.avx, caps.avx2);
    crate::serial_println!("[CPU] Crypto: AES-NI={} PCLMULQDQ={} SHA={} RDRAND={}", 
        caps.aesni, caps.pclmulqdq, caps.sha_ext, caps.rdrand);
    crate::serial_println!("[CPU] Security: SMEP={} SMAP={} NX={}", 
        caps.smep, caps.smap, caps.nx);
    crate::serial_println!("[CPU] Virt: VMX={} SVM={}", caps.vmx, caps.svm);
    
    // Enable SIMD if available
    if caps.sse {
        simd::enable_sse();
    }
    
    // Initialize high-precision TSC
    tsc::init(caps.tsc_frequency_hz);
}

/// Get CPU capabilities (must call init() first)
pub fn capabilities() -> Option<&'static CpuCapabilities> {
    if !CPU_INIT.load(Ordering::Relaxed) {
        return None;
    }
    unsafe { CPU_CAPS.as_ref() }
}

/// Get TSC frequency in Hz
pub fn tsc_frequency() -> u64 {
    capabilities().map(|c| c.tsc_frequency_hz).unwrap_or(3_000_000_000)
}

/// Check if AES-NI is available
pub fn has_aesni() -> bool {
    capabilities().map(|c| c.aesni).unwrap_or(false)
}

/// Check if RDRAND is available
pub fn has_rdrand() -> bool {
    capabilities().map(|c| c.rdrand).unwrap_or(false)
}

/// Get hardware random number (RDRAND)
pub fn rdrand() -> Option<u64> {
    if !has_rdrand() {
        return None;
    }
    
    let mut value: u64;
    let success: u8;
    
    unsafe {
        core::arch::asm!(
            "rdrand {0}",
            "setc {1}",
            out(reg) value,
            out(reg_byte) success,
            options(nostack)
        );
    }
    
    if success != 0 {
        Some(value)
    } else {
        None
    }
}

/// Get hardware random seed (RDSEED) - higher quality than RDRAND
pub fn rdseed() -> Option<u64> {
    let caps = capabilities()?;
    if !caps.rdseed {
        return None;
    }
    
    let mut value: u64;
    let success: u8;
    
    unsafe {
        core::arch::asm!(
            "rdseed {0}",
            "setc {1}",
            out(reg) value,
            out(reg_byte) success,
            options(nostack)
        );
    }
    
    if success != 0 {
        Some(value)
    } else {
        None
    }
}
