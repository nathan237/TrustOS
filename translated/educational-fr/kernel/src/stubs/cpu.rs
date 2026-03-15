//! CPU module stub for non-x86_64 architectures
//!
//! Provides the same public API as the real cpu module but with no-op/default
//! implementations. This allows consumer code to compile on all architectures
//! without modification.

use alloc::string::String;
use alloc::vec::Vec;

// Re-export submodules — stubs in same directory
pub mod features;
pub mod tsc;
pub mod simd;
pub mod smp;

/// CPU vendor enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum CpuVendor {
    Intel,
    Amd,
    Unknown,
}

/// CPU capabilities (stub — all features disabled)
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
    pub maximum_logical_cpus: u8,
    pub maximum_physical_cpus: u8,
    pub tsc_frequency_hz: u64,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl CpuCapabilities {
        // Fonction publique — appelable depuis d'autres modules.
pub fn detect() -> Self {
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
            maximum_logical_cpus: 1,
            maximum_physical_cpus: 1,
            tsc_frequency_hz: 1_000_000_000,
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn brand(&self) -> &str {
        crate::arch::arch_name()
    }
}

static mut CAPS: Option<CpuCapabilities> = None;

// Fonction publique — appelable depuis d'autres modules.
pub fn init() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { CAPS = Some(CpuCapabilities::detect()); }
}

// Fonction publique — appelable depuis d'autres modules.
pub fn capabilities() -> Option<&'static CpuCapabilities> {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { CAPS.as_ref() }
}

// Fonction publique — appelable depuis d'autres modules.
pub fn tsc_frequency() -> u64 {
    1_000_000_000 // 1 GHz default
}

// Fonction publique — appelable depuis d'autres modules.
pub fn has_aesni() -> bool { false }
// Fonction publique — appelable depuis d'autres modules.
pub fn has_rdrand() -> bool { false }
// Fonction publique — appelable depuis d'autres modules.
pub fn core_count() -> u8 { 1 }

// Fonction publique — appelable depuis d'autres modules.
pub fn rdrand() -> Option<u64> { None }
// Fonction publique — appelable depuis d'autres modules.
pub fn rdseed() -> Option<u64> { None }
