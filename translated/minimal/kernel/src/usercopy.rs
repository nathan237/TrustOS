





use core::mem::size_of;
use core::slice;
use alloc::vec::Vec;
use alloc::string::String;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserCopyError {
    
    Qf,
    
    Nq,
    
    Overflow,
    
    Mb,
    
    Awu,
    
    Ajh,
}

impl UserCopyError {
    
    pub fn zsv(self) -> i64 {
        match self {
            Self::Qf | Self::Nq => -14, 
            Self::Overflow => -14, 
            Self::Mb => -14, 
            Self::Awu => -13, 
            Self::Ajh => -22, 
        }
    }
}


pub struct Coy {
    ptr: u64,
    len: usize,
    bjb: bool,
}

impl Coy {
    
    pub fn jmq(ptr: u64, len: usize) -> Result<Self, UserCopyError> {
        Self::dxi(ptr, len, false)?;
        Ok(Self { ptr, len, bjb: false })
    }
    
    
    pub fn yq(ptr: u64, len: usize) -> Result<Self, UserCopyError> {
        Self::dxi(ptr, len, true)?;
        Ok(Self { ptr, len, bjb: true })
    }
    
    
    pub fn zwl(ptr: u64, len: usize) -> Result<Self, UserCopyError> {
        Self::dxi(ptr, len, true)?;
        Ok(Self { ptr, len, bjb: true })
    }
    
    
    fn dxi(ptr: u64, len: usize, write: bool) -> Result<(), UserCopyError> {
        
        if ptr == 0 && len == 0 {
            return Ok(());
        }
        
        if ptr == 0 {
            return Err(UserCopyError::Qf);
        }
        
        
        let ci = ptr.ink(len as u64)
            .ok_or(UserCopyError::Overflow)?;
        
        
        if !crate::memory::aov(ptr) {
            return Err(UserCopyError::Nq);
        }
        
        if !crate::memory::aov(ci.ao(1)) {
            return Err(UserCopyError::Nq);
        }
        
        
        if !crate::memory::sw(ptr, len, write) {
            return Err(UserCopyError::Mb);
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
    
    
    pub fn zik(&self, k: &mut [u8]) -> Result<usize, UserCopyError> {
        let ajp = k.len().v(self.len);
        nfv(&mut k[..ajp], self.ptr)?;
        Ok(ajp)
    }
    
    
    pub fn zws(&self, k: &[u8]) -> Result<usize, UserCopyError> {
        if !self.bjb {
            return Err(UserCopyError::Awu);
        }
        let dwy = k.len().v(self.len);
        nfz(self.ptr, &k[..dwy])?;
        Ok(dwy)
    }
    
    
    pub unsafe fn zhq<T: Copy>(&self) -> Result<T, UserCopyError> {
        if self.len < size_of::<T>() {
            return Err(UserCopyError::Ajh);
        }
        
        let mut bn: T = core::mem::zeroed();
        let slice = slice::bef(
            &mut bn as *mut T as *mut u8,
            size_of::<T>()
        );
        nfv(slice, self.ptr)?;
        Ok(bn)
    }
    
    
    pub unsafe fn zwr<T: Copy>(&self, bn: &T) -> Result<(), UserCopyError> {
        if !self.bjb {
            return Err(UserCopyError::Awu);
        }
        if self.len < size_of::<T>() {
            return Err(UserCopyError::Ajh);
        }
        
        let slice = slice::anh(
            bn as *const T as *const u8,
            size_of::<T>()
        );
        nfz(self.ptr, slice)?;
        Ok(())
    }
    
    
    pub fn zdu(self) -> Option<Self> {
        if self.ptr == 0 {
            None
        } else {
            Some(self)
        }
    }
}


pub fn nfv(cs: &mut [u8], aob: u64) -> Result<(), UserCopyError> {
    if cs.is_empty() {
        return Ok(());
    }
    
    if aob == 0 {
        return Err(UserCopyError::Qf);
    }
    
    
    if !crate::memory::aov(aob) {
        return Err(UserCopyError::Nq);
    }
    
    
    if !crate::memory::sw(aob, cs.len(), false) {
        return Err(UserCopyError::Mb);
    }
    
    
    unsafe {
        let cy = aob as *const u8;
        core::ptr::copy_nonoverlapping(cy, cs.mw(), cs.len());
    }
    
    Ok(())
}


pub fn nfz(alc: u64, cy: &[u8]) -> Result<(), UserCopyError> {
    if cy.is_empty() {
        return Ok(());
    }
    
    if alc == 0 {
        return Err(UserCopyError::Qf);
    }
    
    
    if !crate::memory::aov(alc) {
        return Err(UserCopyError::Nq);
    }
    
    
    if !crate::memory::sw(alc, cy.len(), true) {
        return Err(UserCopyError::Mb);
    }
    
    
    unsafe {
        let cs = alc as *mut u8;
        core::ptr::copy_nonoverlapping(cy.fq(), cs, cy.len());
    }
    
    Ok(())
}


pub fn ykb(ptr: u64, cat: usize) -> Result<String, UserCopyError> {
    if ptr == 0 {
        return Err(UserCopyError::Qf);
    }
    
    if !crate::memory::aov(ptr) {
        return Err(UserCopyError::Nq);
    }
    
    let mut result = Vec::fc(256);
    let mut l = 0u64;
    
    loop {
        if l as usize >= cat {
            break;
        }
        
        let ag = ptr.ink(l)
            .ok_or(UserCopyError::Overflow)?;
        
        if !crate::memory::sw(ag, 1, false) {
            return Err(UserCopyError::Mb);
        }
        
        let hf = unsafe { *(ag as *const u8) };
        
        if hf == 0 {
            break;
        }
        
        result.push(hf);
        l += 1;
    }
    
    String::jg(result).jd(|_| UserCopyError::Ajh)
}


pub fn zig<T: Copy>(ptr: u64) -> Result<T, UserCopyError> {
    if ptr == 0 {
        return Err(UserCopyError::Qf);
    }
    
    if !crate::memory::aov(ptr) {
        return Err(UserCopyError::Nq);
    }
    
    if !crate::memory::sw(ptr, size_of::<T>(), false) {
        return Err(UserCopyError::Mb);
    }
    
    unsafe {
        Ok(core::ptr::read(ptr as *const T))
    }
}


pub fn zwz<T: Copy>(ptr: u64, bn: &T) -> Result<(), UserCopyError> {
    if ptr == 0 {
        return Err(UserCopyError::Qf);
    }
    
    if !crate::memory::aov(ptr) {
        return Err(UserCopyError::Nq);
    }
    
    if !crate::memory::sw(ptr, size_of::<T>(), true) {
        return Err(UserCopyError::Mb);
    }
    
    unsafe {
        core::ptr::write(ptr as *mut T, *bn);
    }
    
    Ok(())
}


pub fn zvd(ptr: u64, len: usize, write: bool) -> Result<(), UserCopyError> {
    if ptr == 0 && len == 0 {
        return Ok(());
    }
    
    if ptr == 0 {
        return Err(UserCopyError::Qf);
    }
    
    let ci = ptr.ink(len as u64)
        .ok_or(UserCopyError::Overflow)?;
    
    if !crate::memory::aov(ptr) || !crate::memory::aov(ci.ao(1)) {
        return Err(UserCopyError::Nq);
    }
    
    if !crate::memory::sw(ptr, len, write) {
        return Err(UserCopyError::Mb);
    }
    
    Ok(())
}
