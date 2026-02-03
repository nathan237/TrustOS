//! Minimal C runtime shims for mbedtls.
//!
//! IMPORTANT: These functions use byte-by-byte loops instead of core::ptr
//! intrinsics to prevent the compiler from optimizing them back into calls
//! to memcpy/memmove/memset, which would cause infinite recursion.

/// Copy n bytes from src to dest (non-overlapping regions)
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // Byte-by-byte to prevent compiler optimization to memcpy call
    for i in 0..n {
        *dest.add(i) = *src.add(i);
    }
    dest
}

/// Copy n bytes from src to dest (handles overlapping regions)
#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // Byte-by-byte to prevent compiler optimization to memmove call
    if (dest as usize) < (src as usize) {
        // Copy forward
        for i in 0..n {
            *dest.add(i) = *src.add(i);
        }
    } else if (dest as usize) > (src as usize) {
        // Copy backward to handle overlap
        for i in (0..n).rev() {
            *dest.add(i) = *src.add(i);
        }
    }
    dest
}

/// Set n bytes to value c
#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    // Byte-by-byte to prevent compiler optimization to memset call
    let byte = c as u8;
    for i in 0..n {
        *s.add(i) = byte;
    }
    s
}

#[no_mangle]
pub extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = unsafe { *s1.add(i) };
        let b = unsafe { *s2.add(i) };
        if a != b {
            return a as i32 - b as i32;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn strlen(s: *const u8) -> usize {
    let mut len = 0usize;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}

#[no_mangle]
pub extern "C" fn strnlen(s: *const u8, maxlen: usize) -> usize {
    let mut len = 0usize;
    unsafe {
        while len < maxlen && *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}

#[no_mangle]
pub extern "C" fn strcmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut i = 0usize;
    unsafe {
        loop {
            let a = *s1.add(i);
            let b = *s2.add(i);
            if a != b {
                return a as i32 - b as i32;
            }
            if a == 0 {
                return 0;
            }
            i += 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn strncmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = unsafe { *s1.add(i) };
        let b = unsafe { *s2.add(i) };
        if a != b {
            return a as i32 - b as i32;
        }
        if a == 0 {
            return 0;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn strcpy(dest: *mut u8, src: *const u8) -> *mut u8 {
    let mut i = 0usize;
    unsafe {
        loop {
            let ch = *src.add(i);
            *dest.add(i) = ch;
            if ch == 0 {
                break;
            }
            i += 1;
        }
    }
    dest
}

#[no_mangle]
pub extern "C" fn strncpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0usize;
    unsafe {
        while i < n {
            let ch = *src.add(i);
            *dest.add(i) = ch;
            i += 1;
            if ch == 0 {
                break;
            }
        }
        while i < n {
            *dest.add(i) = 0;
            i += 1;
        }
    }
    dest
}

#[no_mangle]
pub extern "C" fn strchr(s: *const u8, c: i32) -> *mut u8 {
    let target = c as u8;
    let mut i = 0usize;
    unsafe {
        loop {
            let ch = *s.add(i);
            if ch == target {
                return s.add(i) as *mut u8;
            }
            if ch == 0 {
                return core::ptr::null_mut();
            }
            i += 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn strstr(haystack: *const u8, needle: *const u8) -> *mut u8 {
    if needle.is_null() || haystack.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        if *needle == 0 {
            return haystack as *mut u8;
        }
        let mut i = 0usize;
        loop {
            let h = *haystack.add(i);
            if h == 0 {
                return core::ptr::null_mut();
            }
            if h == *needle {
                let mut j = 1usize;
                loop {
                    let n = *needle.add(j);
                    if n == 0 {
                        return haystack.add(i) as *mut u8;
                    }
                    let hh = *haystack.add(i + j);
                    if hh == 0 || hh != n {
                        break;
                    }
                    j += 1;
                }
            }
            i += 1;
        }
    }
}

