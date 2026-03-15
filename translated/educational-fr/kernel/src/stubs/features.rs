//! CPU features stub for non-x86_64 architectures

#[derive(Debug, Clone, Copy)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum Feature {
    Tsc, TscInvariant, Rdtscp, Sse, Sse2, Sse3, Ssse3,
    Sse41, Sse42, Avx, Avx2, Avx512f, AesNi, Pclmulqdq,
    Sha, Rdrand, Rdseed, Vmx, Svm,
}

// Implémentation de trait — remplit un contrat comportemental.
impl core::fmt::Display for Feature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Fonction publique — appelable depuis d'autres modules.
pub fn has_feature(_feature: Feature) -> bool { false }
// Fonction publique — appelable depuis d'autres modules.
pub fn print_features() {
    crate::serial_println!("CPU features: none (non-x86_64 stub)");
}
