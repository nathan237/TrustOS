//! Performance optimization utilities

#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn spin_hint() {
    core::hint::spin_loop();
}

#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn likely(b: bool) -> bool {
    b
}

#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn unlikely(b: bool) -> bool {
    b
}
