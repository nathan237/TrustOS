

pub fn fuo() {}
pub fn ful() -> bool { false }
pub fn lpq() -> bool { false }

pub unsafe fn nef(dst: *mut u8, src: *const u8, len: usize) {
    core::ptr::copy_nonoverlapping(src, dst, len);
}

pub unsafe fn nei(dst: *mut u8, value: u8, len: usize) {
    core::ptr::write_bytes(dst, value, len);
}

pub unsafe fn nee(a: *const u8, b: *const u8, len: usize) -> bool {
    for i in 0..len {
        if *a.add(i) != *b.add(i) { return false; }
    }
    true
}

pub unsafe fn pvu(dst: *mut u8, src: *const u8, len: usize) {
    for i in 0..len {
        *dst.add(i) ^= *src.add(i);
    }
}
