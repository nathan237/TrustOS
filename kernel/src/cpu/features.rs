//! CPU Feature Detection Utilities
//!
//! Helper functions for runtime feature detection.

/// Check if a specific CPU feature is available
pub fn has_feature(feature: Feature) -> bool {
    let caps = super::capabilities();
    
    match feature {
        Feature::Tsc => caps.map(|c| c.tsc).unwrap_or(false),
        Feature::TscInvariant => caps.map(|c| c.tsc_invariant).unwrap_or(false),
        Feature::Rdtscp => caps.map(|c| c.rdtscp).unwrap_or(false),
        Feature::Sse => caps.map(|c| c.sse).unwrap_or(false),
        Feature::Sse2 => caps.map(|c| c.sse2).unwrap_or(false),
        Feature::Sse3 => caps.map(|c| c.sse3).unwrap_or(false),
        Feature::Ssse3 => caps.map(|c| c.ssse3).unwrap_or(false),
        Feature::Sse41 => caps.map(|c| c.sse4_1).unwrap_or(false),
        Feature::Sse42 => caps.map(|c| c.sse4_2).unwrap_or(false),
        Feature::Avx => caps.map(|c| c.avx).unwrap_or(false),
        Feature::Avx2 => caps.map(|c| c.avx2).unwrap_or(false),
        Feature::Avx512f => caps.map(|c| c.avx512f).unwrap_or(false),
        Feature::AesNi => caps.map(|c| c.aesni).unwrap_or(false),
        Feature::Pclmulqdq => caps.map(|c| c.pclmulqdq).unwrap_or(false),
        Feature::Sha => caps.map(|c| c.sha_ext).unwrap_or(false),
        Feature::Rdrand => caps.map(|c| c.rdrand).unwrap_or(false),
        Feature::Rdseed => caps.map(|c| c.rdseed).unwrap_or(false),
        Feature::Vmx => caps.map(|c| c.vmx).unwrap_or(false),
        Feature::Svm => caps.map(|c| c.svm).unwrap_or(false),
    }
}

/// CPU feature flags
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Feature {
    // Timing
    Tsc,
    TscInvariant,
    Rdtscp,
    
    // SIMD
    Sse,
    Sse2,
    Sse3,
    Ssse3,
    Sse41,
    Sse42,
    Avx,
    Avx2,
    Avx512f,
    
    // Crypto
    AesNi,
    Pclmulqdq,
    Sha,
    Rdrand,
    Rdseed,
    
    // Virtualization
    Vmx,
    Svm,
}

impl core::fmt::Display for Feature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Feature::Tsc => write!(f, "TSC"),
            Feature::TscInvariant => write!(f, "Invariant TSC"),
            Feature::Rdtscp => write!(f, "RDTSCP"),
            Feature::Sse => write!(f, "SSE"),
            Feature::Sse2 => write!(f, "SSE2"),
            Feature::Sse3 => write!(f, "SSE3"),
            Feature::Ssse3 => write!(f, "SSSE3"),
            Feature::Sse41 => write!(f, "SSE4.1"),
            Feature::Sse42 => write!(f, "SSE4.2"),
            Feature::Avx => write!(f, "AVX"),
            Feature::Avx2 => write!(f, "AVX2"),
            Feature::Avx512f => write!(f, "AVX-512F"),
            Feature::AesNi => write!(f, "AES-NI"),
            Feature::Pclmulqdq => write!(f, "PCLMULQDQ"),
            Feature::Sha => write!(f, "SHA-NI"),
            Feature::Rdrand => write!(f, "RDRAND"),
            Feature::Rdseed => write!(f, "RDSEED"),
            Feature::Vmx => write!(f, "VMX"),
            Feature::Svm => write!(f, "SVM"),
        }
    }
}

/// Print all available features
pub fn print_features() {
    let features = [
        Feature::Tsc,
        Feature::TscInvariant,
        Feature::Rdtscp,
        Feature::Sse,
        Feature::Sse2,
        Feature::Sse3,
        Feature::Ssse3,
        Feature::Sse41,
        Feature::Sse42,
        Feature::Avx,
        Feature::Avx2,
        Feature::Avx512f,
        Feature::AesNi,
        Feature::Pclmulqdq,
        Feature::Sha,
        Feature::Rdrand,
        Feature::Rdseed,
        Feature::Vmx,
        Feature::Svm,
    ];
    
    crate::println!("CPU Features:");
    for feature in features.iter() {
        let status = if has_feature(*feature) { "✓" } else { "✗" };
        crate::println!("  {} {}", status, feature);
    }
}
