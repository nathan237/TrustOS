








pub mod features;
pub mod tsc;
pub mod simd;
pub mod smp;

use core::sync::atomic::{AtomicBool, Ordering};


static ARA_: AtomicBool = AtomicBool::new(false);
static mut AQY_: Option<CpuCapabilities> = None;


#[derive(Debug, Clone)]
pub struct CpuCapabilities {
    
    pub vendor: CpuVendor,
    pub family: u8,
    pub model: u8,
    pub stepping: u8,
    pub brand_string: [u8; 48],
    
    
    pub tsc: bool,           
    pub tsc_invariant: bool, 
    pub tsc_deadline: bool,  
    pub rdtscp: bool,        
    
    
    pub sse: bool,
    pub sse2: bool,
    pub sse3: bool,
    pub ssse3: bool,
    pub sse4_1: bool,
    pub sse4_2: bool,
    pub avx: bool,
    pub avx2: bool,
    pub avx512f: bool,
    pub fma: bool,
    
    
    pub aesni: bool,         
    pub pclmulqdq: bool,     
    pub sha_ext: bool,       
    pub rdrand: bool,        
    pub rdseed: bool,        
    
    
    pub smep: bool,
    pub smap: bool,
    pub umip: bool,
    pub nx: bool,
    
    
    pub vmx: bool,           
    pub svm: bool,           
    
    
    pub max_logical_cpus: u8,
    pub max_physical_cpus: u8,
    pub apic_id: u8,
    
    
    pub tsc_frequency_hz: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CpuVendor {
    Intel,
    Amd,
    Unknown,
}

impl CpuCapabilities {
    
    pub fn bfx() -> Self {
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
            fma: false,
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
        
        
        let cvt = unsafe { core::arch::x86_64::__cpuid(0) };
        let img = cvt.eax;
        
        
        let hbi = [
            cvt.ebx.to_le_bytes(),
            cvt.edx.to_le_bytes(),
            cvt.ecx.to_le_bytes(),
        ];
        
        let mut bpw = [0u8; 12];
        bpw[0..4].copy_from_slice(&hbi[0]);
        bpw[4..8].copy_from_slice(&hbi[1]);
        bpw[8..12].copy_from_slice(&hbi[2]);
        
        caps.vendor = match &bpw {
            b"GenuineIntel" => CpuVendor::Intel,
            b"AuthenticAMD" => CpuVendor::Amd,
            _ => CpuVendor::Unknown,
        };
        
        
        if img >= 1 {
            let afa = unsafe { core::arch::x86_64::__cpuid(1) };
            
            
            caps.stepping = (afa.eax & 0xF) as u8;
            caps.model = ((afa.eax >> 4) & 0xF) as u8;
            caps.family = ((afa.eax >> 8) & 0xF) as u8;
            
            
            if caps.family == 0xF {
                caps.family += ((afa.eax >> 20) & 0xFF) as u8;
            }
            if caps.family >= 6 {
                caps.model += (((afa.eax >> 16) & 0xF) << 4) as u8;
            }
            
            
            caps.apic_id = ((afa.ebx >> 24) & 0xFF) as u8;
            caps.max_logical_cpus = ((afa.ebx >> 16) & 0xFF) as u8;
            
            
            caps.sse3 = (afa.ecx & (1 << 0)) != 0;
            caps.pclmulqdq = (afa.ecx & (1 << 1)) != 0;
            caps.ssse3 = (afa.ecx & (1 << 9)) != 0;
            caps.sse4_1 = (afa.ecx & (1 << 19)) != 0;
            caps.sse4_2 = (afa.ecx & (1 << 20)) != 0;
            caps.aesni = (afa.ecx & (1 << 25)) != 0;
            caps.avx = (afa.ecx & (1 << 28)) != 0;
            caps.fma = (afa.ecx & (1 << 12)) != 0;
            caps.rdrand = (afa.ecx & (1 << 30)) != 0;
            caps.vmx = (afa.ecx & (1 << 5)) != 0;
            caps.tsc_deadline = (afa.ecx & (1 << 24)) != 0;
            
            
            caps.tsc = (afa.edx & (1 << 4)) != 0;
            caps.sse = (afa.edx & (1 << 25)) != 0;
            caps.sse2 = (afa.edx & (1 << 26)) != 0;
        }
        
        
        if img >= 7 {
            let bfp = unsafe { core::arch::x86_64::__cpuid_count(7, 0) };
            
            caps.smep = (bfp.ebx & (1 << 7)) != 0;
            caps.avx2 = (bfp.ebx & (1 << 5)) != 0;
            caps.avx512f = (bfp.ebx & (1 << 16)) != 0;
            caps.sha_ext = (bfp.ebx & (1 << 29)) != 0;
            caps.rdseed = (bfp.ebx & (1 << 18)) != 0;
            caps.smap = (bfp.ebx & (1 << 20)) != 0;
            caps.umip = (bfp.ecx & (1 << 2)) != 0;
        }
        
        
        let kyy = unsafe { core::arch::x86_64::__cpuid(0x80000000) };
        let ggw = kyy.eax;
        
        
        if ggw >= 0x80000001 {
            let foz = unsafe { core::arch::x86_64::__cpuid(0x80000001) };
            
            caps.nx = (foz.edx & (1 << 20)) != 0;
            caps.rdtscp = (foz.edx & (1 << 27)) != 0;
            caps.svm = (foz.ecx & (1 << 2)) != 0;
        }
        
        
        if ggw >= 0x80000007 {
            let kyx = unsafe { core::arch::x86_64::__cpuid(0x80000007) };
            caps.tsc_invariant = (kyx.edx & (1 << 8)) != 0;
        }
        
        
        if ggw >= 0x80000004 {
            for i in 0..3 {
                let ejb = unsafe { core::arch::x86_64::__cpuid(0x80000002 + i) };
                let offset = (i as usize) * 16;
                caps.brand_string[offset..offset+4].copy_from_slice(&ejb.eax.to_le_bytes());
                caps.brand_string[offset+4..offset+8].copy_from_slice(&ejb.ebx.to_le_bytes());
                caps.brand_string[offset+8..offset+12].copy_from_slice(&ejb.ecx.to_le_bytes());
                caps.brand_string[offset+12..offset+16].copy_from_slice(&ejb.edx.to_le_bytes());
            }
        }
        
        caps
    }
    
    
    pub fn brand(&self) -> &str {
        let end = self.brand_string.iter()
            .position(|&b| b == 0)
            .unwrap_or(48);
        core::str::from_utf8(&self.brand_string[..end])
            .unwrap_or("Unknown CPU")
            .trim()
    }
}


pub fn init() {
    if ARA_.swap(true, Ordering::SeqCst) {
        return; 
    }
    
    
    let mut caps = CpuCapabilities::bfx();
    
    
    caps.tsc_frequency_hz = tsc::hju();
    
    
    unsafe {
        AQY_ = Some(caps.clone());
    }
    
    
    crate::serial_println!("[CPU] {}", caps.brand());
    crate::serial_println!("[CPU] Vendor: {:?}, Family: {}, Model: {}", 
        caps.vendor, caps.family, caps.model);
    crate::serial_println!("[CPU] TSC: {} (invariant: {}, freq: {} MHz)", 
        caps.tsc, caps.tsc_invariant, caps.tsc_frequency_hz / 1_000_000);
    crate::serial_println!("[CPU] SIMD: SSE={} SSE2={} SSE4.2={} AVX={} AVX2={} FMA={}", 
        caps.sse, caps.sse2, caps.sse4_2, caps.avx, caps.avx2, caps.fma);
    crate::serial_println!("[CPU] Crypto: AES-NI={} PCLMULQDQ={} SHA={} RDRAND={}", 
        caps.aesni, caps.pclmulqdq, caps.sha_ext, caps.rdrand);
    crate::serial_println!("[CPU] Security: SMEP={} SMAP={} NX={}", 
        caps.smep, caps.smap, caps.nx);
    crate::serial_println!("[CPU] Virt: VMX={} SVM={}", caps.vmx, caps.svm);
    
    
    if caps.sse {
        simd::fuo();
    }
    
    
    if caps.avx {
        simd::ful();
    }
    
    
    tsc::init(caps.tsc_frequency_hz);
}


pub fn capabilities() -> Option<&'static CpuCapabilities> {
    if !ARA_.load(Ordering::Relaxed) {
        return None;
    }
    unsafe { AQY_.as_ref() }
}


pub fn hac() -> u64 {
    capabilities().map(|c| c.tsc_frequency_hz).unwrap_or(3_000_000_000)
}


pub fn has_aesni() -> bool {
    capabilities().map(|c| c.aesni).unwrap_or(false)
}


pub fn has_rdrand() -> bool {
    capabilities().map(|c| c.rdrand).unwrap_or(false)
}


pub fn cvr() -> u8 {
    capabilities().map(|c| c.max_logical_cpus).unwrap_or(1)
}


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
