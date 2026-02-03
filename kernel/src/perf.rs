//! Performance optimization utilities

use core::arch::x86_64::_mm_pause;

#[inline(always)]
pub fn spin_hint() {
    unsafe { _mm_pause(); }
}

#[inline(always)]
pub fn likely(b: bool) -> bool {
    b
}

#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    b
}
