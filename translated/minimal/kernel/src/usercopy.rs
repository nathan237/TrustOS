





use core::mem::size_of;
use core::slice;
use alloc::vec::Vec;
use alloc::string::String;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserCopyError {
    
    NullPointer,
    
    NotUserSpace,
    
    Overflow,
    
    PageFault,
    
    Permission,
    
    InvalidLength,
}

impl UserCopyError {
    
    pub fn raj(self) -> i64 {
        match self {
            Self::NullPointer | Self::NotUserSpace => -14, 
            Self::Overflow => -14, 
            Self::PageFault => -14, 
            Self::Permission => -13, 
            Self::InvalidLength => -22, 
        }
    }
}


pub struct Ari {
    ptr: u64,
    len: usize,
    writable: bool,
}

impl Ari {
    
    pub fn eyv(ptr: u64, len: usize) -> Result<Self, UserCopyError> {
        Self::bpu(ptr, len, false)?;
        Ok(Self { ptr, len, writable: false })
    }
    
    
    pub fn lk(ptr: u64, len: usize) -> Result<Self, UserCopyError> {
        Self::bpu(ptr, len, true)?;
        Ok(Self { ptr, len, writable: true })
    }
    
    
    pub fn rcw(ptr: u64, len: usize) -> Result<Self, UserCopyError> {
        Self::bpu(ptr, len, true)?;
        Ok(Self { ptr, len, writable: true })
    }
    
    
    fn bpu(ptr: u64, len: usize, write: bool) -> Result<(), UserCopyError> {
        
        if ptr == 0 && len == 0 {
            return Ok(());
        }
        
        if ptr == 0 {
            return Err(UserCopyError::NullPointer);
        }
        
        
        let end = ptr.checked_add(len as u64)
            .ok_or(UserCopyError::Overflow)?;
        
        
        if !crate::memory::ux(ptr) {
            return Err(UserCopyError::NotUserSpace);
        }
        
        if !crate::memory::ux(end.saturating_sub(1)) {
            return Err(UserCopyError::NotUserSpace);
        }
        
        
        if !crate::memory::ij(ptr, len, write) {
            return Err(UserCopyError::PageFault);
        }
        
        Ok(())
    }
    
    
    pub fn ptr(&self) -> u64 {
        self.ptr
    }
    
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    
    pub fn qsw(&self, buf: &mut [u8]) -> Result<usize, UserCopyError> {
        let rz = buf.len().min(self.len);
        hno(&mut buf[..rz], self.ptr)?;
        Ok(rz)
    }
    
    
    pub fn rdc(&self, buf: &[u8]) -> Result<usize, UserCopyError> {
        if !self.writable {
            return Err(UserCopyError::Permission);
        }
        let bpo = buf.len().min(self.len);
        hns(self.ptr, &buf[..bpo])?;
        Ok(bpo)
    }
    
    
    pub unsafe fn qsc<T: Copy>(&self) -> Result<T, UserCopyError> {
        if self.len < size_of::<T>() {
            return Err(UserCopyError::InvalidLength);
        }
        
        let mut value: T = core::mem::zeroed();
        let slice = slice::from_raw_parts_mut(
            &mut value as *mut T as *mut u8,
            size_of::<T>()
        );
        hno(slice, self.ptr)?;
        Ok(value)
    }
    
    
    pub unsafe fn rdb<T: Copy>(&self, value: &T) -> Result<(), UserCopyError> {
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
        hns(self.ptr, slice)?;
        Ok(())
    }
    
    
    pub fn qpp(self) -> Option<Self> {
        if self.ptr == 0 {
            None
        } else {
            Some(self)
        }
    }
}


pub fn hno(dst: &mut [u8], ps: u64) -> Result<(), UserCopyError> {
    if dst.is_empty() {
        return Ok(());
    }
    
    if ps == 0 {
        return Err(UserCopyError::NullPointer);
    }
    
    
    if !crate::memory::ux(ps) {
        return Err(UserCopyError::NotUserSpace);
    }
    
    
    if !crate::memory::ij(ps, dst.len(), false) {
        return Err(UserCopyError::PageFault);
    }
    
    
    unsafe {
        let src = ps as *const u8;
        core::ptr::copy_nonoverlapping(src, dst.as_mut_ptr(), dst.len());
    }
    
    Ok(())
}


pub fn hns(nt: u64, src: &[u8]) -> Result<(), UserCopyError> {
    if src.is_empty() {
        return Ok(());
    }
    
    if nt == 0 {
        return Err(UserCopyError::NullPointer);
    }
    
    
    if !crate::memory::ux(nt) {
        return Err(UserCopyError::NotUserSpace);
    }
    
    
    if !crate::memory::ij(nt, src.len(), true) {
        return Err(UserCopyError::PageFault);
    }
    
    
    unsafe {
        let dst = nt as *mut u8;
        core::ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
    }
    
    Ok(())
}


pub fn qbj(ptr: u64, aoo: usize) -> Result<String, UserCopyError> {
    if ptr == 0 {
        return Err(UserCopyError::NullPointer);
    }
    
    if !crate::memory::ux(ptr) {
        return Err(UserCopyError::NotUserSpace);
    }
    
    let mut result = Vec::with_capacity(256);
    let mut offset = 0u64;
    
    loop {
        if offset as usize >= aoo {
            break;
        }
        
        let addr = ptr.checked_add(offset)
            .ok_or(UserCopyError::Overflow)?;
        
        if !crate::memory::ij(addr, 1, false) {
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


pub fn qss<T: Copy>(ptr: u64) -> Result<T, UserCopyError> {
    if ptr == 0 {
        return Err(UserCopyError::NullPointer);
    }
    
    if !crate::memory::ux(ptr) {
        return Err(UserCopyError::NotUserSpace);
    }
    
    if !crate::memory::ij(ptr, size_of::<T>(), false) {
        return Err(UserCopyError::PageFault);
    }
    
    unsafe {
        Ok(core::ptr::read(ptr as *const T))
    }
}


pub fn rdk<T: Copy>(ptr: u64, value: &T) -> Result<(), UserCopyError> {
    if ptr == 0 {
        return Err(UserCopyError::NullPointer);
    }
    
    if !crate::memory::ux(ptr) {
        return Err(UserCopyError::NotUserSpace);
    }
    
    if !crate::memory::ij(ptr, size_of::<T>(), true) {
        return Err(UserCopyError::PageFault);
    }
    
    unsafe {
        core::ptr::write(ptr as *mut T, *value);
    }
    
    Ok(())
}


pub fn rbw(ptr: u64, len: usize, write: bool) -> Result<(), UserCopyError> {
    if ptr == 0 && len == 0 {
        return Ok(());
    }
    
    if ptr == 0 {
        return Err(UserCopyError::NullPointer);
    }
    
    let end = ptr.checked_add(len as u64)
        .ok_or(UserCopyError::Overflow)?;
    
    if !crate::memory::ux(ptr) || !crate::memory::ux(end.saturating_sub(1)) {
        return Err(UserCopyError::NotUserSpace);
    }
    
    if !crate::memory::ij(ptr, len, write) {
        return Err(UserCopyError::PageFault);
    }
    
    Ok(())
}
