

#[derive(Debug, Clone, Copy)]
pub enum Feature {
    Tsc, TscInvariant, Rdtscp, Sse, Sse2, Sse3, Ssse3,
    Sse41, Sse42, Avx, Avx2, Avx512f, AesNi, Pclmulqdq,
    Sha, Rdrand, Rdseed, Vmx, Svm,
}

impl core::fmt::Display for Feature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn ido(_feature: Feature) -> bool { false }
pub fn nxe() {
    crate::serial_println!("CPU features: none (non-x86_64 stub)");
}
