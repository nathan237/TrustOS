//! SIMD stub for non-x86_64 architectures

pub fn enable_sse() {}
// Fonction publique — appelable depuis d'autres modules.
pub fn enable_avx() -> bool { false }
// Fonction publique — appelable depuis d'autres modules.
pub fn enable_avx512() -> bool { false }

pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn memcpy_sse2(dst: *mut u8, src: *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, len: usize) {
    core::ptr::copy_nonoverlapping(src, dst, len);
}

pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn memset_sse2(dst: *mut u8, value: u8, len: usize) {
    core::ptr::write_bytes(dst, value, len);
}

pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn memcmp_sse2(a: *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, b: *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, len: usize) -> bool {
    for i in 0..len {
        if *a.add(i) != *b.add(i) { return false; }
    }
    true
}

pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn xor_blocks_sse2(dst: *mut u8, src: *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, len: usize) {
    for i in 0..len {
        *dst.add(i) ^= *src.add(i);
    }
}
