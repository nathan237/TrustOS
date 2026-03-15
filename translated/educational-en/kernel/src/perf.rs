//! Performance optimization utilities

#[inline(always)]
// Public function — callable from other modules.
pub fn spin_hint() {
    core::hint::spin_loop();
}

#[inline(always)]
// Public function — callable from other modules.
pub fn likely(b: bool) -> bool {
    b
}

#[inline(always)]
// Public function — callable from other modules.
pub fn unlikely(b: bool) -> bool {
    b
}
