//! SIMD stub for non-x86_64 architectures

pub fn enable_sse() {}
// Public function — callable from other modules.
pub fn enable_avx() -> bool { false }
// Public function — callable from other modules.
pub fn enable_avx512() -> bool { false }

pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn memcpy_sse2(dst: *mut u8, src: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8, len: usize) {
    core::ptr::copy_nonoverlapping(src, dst, len);
}

pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn memset_sse2(dst: *mut u8, value: u8, len: usize) {
    core::ptr::write_bytes(dst, value, len);
}

pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn memcmp_sse2(a: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8, b: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8, len: usize) -> bool {
    for i in 0..len {
        if *a.add(i) != *b.add(i) { return false; }
    }
    true
}

pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn xor_blocks_sse2(dst: *mut u8, src: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8, len: usize) {
    for i in 0..len {
        *dst.add(i) ^= *src.add(i);
    }
}
