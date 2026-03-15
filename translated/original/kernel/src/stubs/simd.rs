//! SIMD stub for non-x86_64 architectures

pub fn enable_sse() {}
pub fn enable_avx() -> bool { false }
pub fn enable_avx512() -> bool { false }

pub unsafe fn memcpy_sse2(dst: *mut u8, src: *const u8, len: usize) {
    core::ptr::copy_nonoverlapping(src, dst, len);
}

pub unsafe fn memset_sse2(dst: *mut u8, value: u8, len: usize) {
    core::ptr::write_bytes(dst, value, len);
}

pub unsafe fn memcmp_sse2(a: *const u8, b: *const u8, len: usize) -> bool {
    for i in 0..len {
        if *a.add(i) != *b.add(i) { return false; }
    }
    true
}

pub unsafe fn xor_blocks_sse2(dst: *mut u8, src: *const u8, len: usize) {
    for i in 0..len {
        *dst.add(i) ^= *src.add(i);
    }
}
