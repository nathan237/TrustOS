





use alloc::string::String;
use alloc::vec::Vec;


pub mod features;
pub mod tsc;
pub mod simd;
pub mod smp;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuVendor {
    Intel,
    Amd,
    Unknown,
}


pub struct CpuCapabilities {
    pub vendor: CpuVendor,
    pub family: u8,
    pub model: u8,
    pub stepping: u8,
    pub apic_id: u8,
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
    pub aesni: bool,
    pub pclmulqdq: bool,
    pub sha_ext: bool,
    pub rdrand: bool,
    pub rdseed: bool,
    pub nx: bool,
    pub smep: bool,
    pub smap: bool,
    pub umip: bool,
    pub vmx: bool,
    pub svm: bool,
    pub max_logical_cpus: u8,
    pub max_physical_cpus: u8,
    pub tsc_frequency_hz: u64,
}

impl CpuCapabilities {
    pub fn bfx() -> Self {
        Self {
            vendor: CpuVendor::Unknown,
            family: 0,
            model: 0,
            stepping: 0,
            apic_id: 0,
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
            nx: false,
            smep: false,
            smap: false,
            umip: false,
            vmx: false,
            svm: false,
            max_logical_cpus: 1,
            max_physical_cpus: 1,
            tsc_frequency_hz: 1_000_000_000,
        }
    }

    pub fn brand(&self) -> &str {
        crate::arch::fhg()
    }
}

static mut Wt: Option<CpuCapabilities> = None;

pub fn init() {
    unsafe { Wt = Some(CpuCapabilities::bfx()); }
}

pub fn capabilities() -> Option<&'static CpuCapabilities> {
    unsafe { Wt.as_ref() }
}

pub fn hac() -> u64 {
    1_000_000_000 
}

pub fn has_aesni() -> bool { false }
pub fn has_rdrand() -> bool { false }
pub fn cvr() -> u8 { 1 }

pub fn rdrand() -> Option<u64> { None }
pub fn rdseed() -> Option<u64> { None }
