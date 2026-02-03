//! mbedtls allocator hooks for freestanding builds.

use core::cmp::min;
use core::ffi::c_void;
use core::mem::{align_of, size_of};
use core::ptr;

use crate::memory::heap;

#[repr(C)]
struct Header {
    size: usize,
    align: usize,
}

#[no_mangle]
pub extern "C" fn mbedtls_platform_calloc(n: usize, size: usize) -> *mut c_void {
    let total = match n.checked_mul(size) {
        Some(0) | None => return ptr::null_mut(),
        Some(v) => v,
    };

    let header_size = size_of::<Header>();
    let align = align_of::<Header>();
    let alloc_size = match total.checked_add(header_size) {
        Some(v) => v,
        None => return ptr::null_mut(),
    };

    let base: *mut u8 = match heap::allocate(alloc_size, align) {
        Some(ptr) => ptr,
        None => return ptr::null_mut(),
    };

    unsafe {
        let header = base as *mut Header;
        (*header).size = alloc_size;
        (*header).align = align;
        let data = base.add(header_size);
        ptr::write_bytes(data, 0, total);
        data as *mut c_void
    }
}

#[no_mangle]
pub extern "C" fn mbedtls_platform_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        let header_size = size_of::<Header>();
        let base = (ptr as *mut u8).sub(header_size);
        let header = base as *mut Header;
        heap::deallocate(base, (*header).size, (*header).align);
    }
}

#[no_mangle]
pub extern "C" fn calloc(n: usize, size: usize) -> *mut c_void {
    mbedtls_platform_calloc(n, size)
}

#[no_mangle]
pub extern "C" fn free(ptr: *mut c_void) {
    mbedtls_platform_free(ptr)
}

#[no_mangle]
pub extern "C" fn malloc(size: usize) -> *mut c_void {
    mbedtls_platform_calloc(1, size)
}

#[no_mangle]
pub extern "C" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    if ptr.is_null() {
        return mbedtls_platform_calloc(1, size);
    }
    if size == 0 {
        mbedtls_platform_free(ptr);
        return ptr::null_mut();
    }

    unsafe {
        let header_size = size_of::<Header>();
        let base = (ptr as *mut u8).sub(header_size);
        let header = base as *mut Header;
        let old_payload = (*header).size.saturating_sub(header_size);

        let new_ptr = mbedtls_platform_calloc(1, size) as *mut u8;
        if new_ptr.is_null() {
            return ptr::null_mut();
        }

        let copy_len = min(old_payload, size);
        ptr::copy_nonoverlapping(ptr as *const u8, new_ptr, copy_len);
        mbedtls_platform_free(ptr);
        new_ptr as *mut c_void
    }
}
