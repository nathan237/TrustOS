//! Performance optimization utilities

#[inline(always)]
pub fn spin_hint() {
    core::hint::spin_loop();
}

#[inline(always)]
pub fn likely(b: bool) -> bool {
    b
}

#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    b
}
