//! Safe User/Kernel Memory Copy
//!
//! Provides safe primitives for copying data between userspace and kernel.
//! Critical for security - prevents userspace from tricking kernel into
//! reading/writing arbitrary kernel memory.

use core::mem::size_of;
use core::slice;
use alloc::vec::Vec;
use alloc::string::String;

/// Error codes for usercopy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserCopyError {
    /// Pointer is null
    NullPointer,
    /// Address is not in user space
    NotUserSpace,
    /// Address range overflows
    Overflow,
    /// Page not mapped or not accessible
    PageFault,
    /// Insufficient permissions (e.g., writing to read-only)
    Permission,
    /// Length is invalid
    InvalidLength,
}

impl UserCopyError {
    /// Convert to errno
    pub fn to_errno(self) -> i64 {
        match self {
            Self::NullPointer | Self::NotUserSpace => -14, // EFAULT
            Self::Overflow => -14, // EFAULT
            Self::PageFault => -14, // EFAULT
            Self::Permission => -13, // EACCES
            Self::InvalidLength => -22, // EINVAL
        }
    }
}

/// User-space memory slice (validated)
pub struct UserSlice {
    ptr: u64,
    len: usize,
    writable: bool,
}

impl UserSlice {
    /// Create read-only user slice
    pub fn ro(ptr: u64, len: usize) -> Result<Self, UserCopyError> {
        Self::validate(ptr, len, false)?;
        Ok(Self { ptr, len, writable: false })
    }
    
    /// Create read-write user slice
    pub fn rw(ptr: u64, len: usize) -> Result<Self, UserCopyError> {
        Self::validate(ptr, len, true)?;
        Ok(Self { ptr, len, writable: true })
    }
    
    /// Create write-only user slice
    pub fn wo(ptr: u64, len: usize) -> Result<Self, UserCopyError> {
        Self::validate(ptr, len, true)?;
        Ok(Self { ptr, len, writable: true })
    }
    
    /// Validate user pointer
    fn validate(ptr: u64, len: usize, write: bool) -> Result<(), UserCopyError> {
        // Allow null pointer with zero length
        if ptr == 0 && len == 0 {
            return Ok(());
        }
        
        if ptr == 0 {
            return Err(UserCopyError::NullPointer);
        }
        
        // Check for overflow
        let end = ptr.checked_add(len as u64)
            .ok_or(UserCopyError::Overflow)?;
        
        // Check address is in user space
        if !crate::memory::is_user_address(ptr) {
            return Err(UserCopyError::NotUserSpace);
        }
        
        if !crate::memory::is_user_address(end.saturating_sub(1)) {
            return Err(UserCopyError::NotUserSpace);
        }
        
        // Validate pages are mapped and accessible
        if !crate::memory::validate_user_ptr(ptr, len, write) {
            return Err(UserCopyError::PageFault);
        }
        
        Ok(())
    }
    
    /// Get pointer
    pub fn ptr(&self) -> u64 {
        self.ptr
    }
    
    /// Get length
    pub fn len(&self) -> usize {
        self.len
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    /// Read into kernel buffer
    pub fn read_to(&self, buf: &mut [u8]) -> Result<usize, UserCopyError> {
        let to_read = buf.len().min(self.len);
        copy_from_user(&mut buf[..to_read], self.ptr)?;
        Ok(to_read)
    }
    
    /// Write from kernel buffer
    pub fn write_from(&self, buf: &[u8]) -> Result<usize, UserCopyError> {
        if !self.writable {
            return Err(UserCopyError::Permission);
        }
        let to_write = buf.len().min(self.len);
        copy_to_user(self.ptr, &buf[..to_write])?;
        Ok(to_write)
    }
    
    /// Read exact type from user
    pub unsafe fn read_exact<T: Copy>(&self) -> Result<T, UserCopyError> {
        if self.len < size_of::<T>() {
            return Err(UserCopyError::InvalidLength);
        }
        
        let mut value: T = core::mem::zeroed();
        let slice = slice::from_raw_parts_mut(
            &mut value as *mut T as *mut u8,
            size_of::<T>()
        );
        copy_from_user(slice, self.ptr)?;
        Ok(value)
    }
    
    /// Write exact type to user
    pub unsafe fn write_exact<T: Copy>(&self, value: &T) -> Result<(), UserCopyError> {
        if !self.writable {
            return Err(UserCopyError::Permission);
        }
        if self.len < size_of::<T>() {
            return Err(UserCopyError::InvalidLength);
        }
        
        let slice = slice::from_raw_parts(
            value as *const T as *const u8,
            size_of::<T>()
        );
        copy_to_user(self.ptr, slice)?;
        Ok(())
    }
    
    /// Return None if pointer is null
    pub fn none_if_null(self) -> Option<Self> {
        if self.ptr == 0 {
            None
        } else {
            Some(self)
        }
    }
}

/// Copy data from user space to kernel buffer
pub fn copy_from_user(dst: &mut [u8], src_ptr: u64) -> Result<(), UserCopyError> {
    if dst.is_empty() {
        return Ok(());
    }
    
    if src_ptr == 0 {
        return Err(UserCopyError::NullPointer);
    }
    
    // Validate source is in user space
    if !crate::memory::is_user_address(src_ptr) {
        return Err(UserCopyError::NotUserSpace);
    }
    
    // Validate pages
    if !crate::memory::validate_user_ptr(src_ptr, dst.len(), false) {
        return Err(UserCopyError::PageFault);
    }
    
    // Perform copy (in real kernel, this might use special instructions)
    unsafe {
        let src = src_ptr as *const u8;
        core::ptr::copy_nonoverlapping(src, dst.as_mut_ptr(), dst.len());
    }
    
    Ok(())
}

/// Copy data from kernel buffer to user space
pub fn copy_to_user(dst_ptr: u64, src: &[u8]) -> Result<(), UserCopyError> {
    if src.is_empty() {
        return Ok(());
    }
    
    if dst_ptr == 0 {
        return Err(UserCopyError::NullPointer);
    }
    
    // Validate destination is in user space
    if !crate::memory::is_user_address(dst_ptr) {
        return Err(UserCopyError::NotUserSpace);
    }
    
    // Validate pages (need write access)
    if !crate::memory::validate_user_ptr(dst_ptr, src.len(), true) {
        return Err(UserCopyError::PageFault);
    }
    
    // Perform copy
    unsafe {
        let dst = dst_ptr as *mut u8;
        core::ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
    }
    
    Ok(())
}

/// Read a null-terminated string from user space
pub fn copy_string_from_user(ptr: u64, max_len: usize) -> Result<String, UserCopyError> {
    if ptr == 0 {
        return Err(UserCopyError::NullPointer);
    }
    
    if !crate::memory::is_user_address(ptr) {
        return Err(UserCopyError::NotUserSpace);
    }
    
    let mut result = Vec::with_capacity(256);
    let mut offset = 0u64;
    
    loop {
        if offset as usize >= max_len {
            break;
        }
        
        let addr = ptr.checked_add(offset)
            .ok_or(UserCopyError::Overflow)?;
        
        if !crate::memory::validate_user_ptr(addr, 1, false) {
            return Err(UserCopyError::PageFault);
        }
        
        let byte = unsafe { *(addr as *const u8) };
        
        if byte == 0 {
            break;
        }
        
        result.push(byte);
        offset += 1;
    }
    
    String::from_utf8(result).map_err(|_| UserCopyError::InvalidLength)
}

/// Read a fixed-size struct from user space
pub fn read_struct_from_user<T: Copy>(ptr: u64) -> Result<T, UserCopyError> {
    if ptr == 0 {
        return Err(UserCopyError::NullPointer);
    }
    
    if !crate::memory::is_user_address(ptr) {
        return Err(UserCopyError::NotUserSpace);
    }
    
    if !crate::memory::validate_user_ptr(ptr, size_of::<T>(), false) {
        return Err(UserCopyError::PageFault);
    }
    
    unsafe {
        Ok(core::ptr::read(ptr as *const T))
    }
}

/// Write a fixed-size struct to user space
pub fn write_struct_to_user<T: Copy>(ptr: u64, value: &T) -> Result<(), UserCopyError> {
    if ptr == 0 {
        return Err(UserCopyError::NullPointer);
    }
    
    if !crate::memory::is_user_address(ptr) {
        return Err(UserCopyError::NotUserSpace);
    }
    
    if !crate::memory::validate_user_ptr(ptr, size_of::<T>(), true) {
        return Err(UserCopyError::PageFault);
    }
    
    unsafe {
        core::ptr::write(ptr as *mut T, *value);
    }
    
    Ok(())
}

/// Validate a user pointer range without copying
pub fn validate_user_region(ptr: u64, len: usize, write: bool) -> Result<(), UserCopyError> {
    if ptr == 0 && len == 0 {
        return Ok(());
    }
    
    if ptr == 0 {
        return Err(UserCopyError::NullPointer);
    }
    
    let end = ptr.checked_add(len as u64)
        .ok_or(UserCopyError::Overflow)?;
    
    if !crate::memory::is_user_address(ptr) || !crate::memory::is_user_address(end.saturating_sub(1)) {
        return Err(UserCopyError::NotUserSpace);
    }
    
    if !crate::memory::validate_user_ptr(ptr, len, write) {
        return Err(UserCopyError::PageFault);
    }
    
    Ok(())
}
