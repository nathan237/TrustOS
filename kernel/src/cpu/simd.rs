//! SIMD - SSE/AVX Support
//!
//! Enable and manage SIMD extensions for vectorized operations.

/// Enable SSE/SSE2 support
/// Required for x86_64, but needs explicit enabling in kernel mode
pub fn enable_sse() {
    unsafe {
        // Read CR0
        let mut cr0: u64;
        core::arch::asm!("mov {}, cr0", out(reg) cr0);
        
        // Clear EM (bit 2) - Emulation flag must be 0
        // Set MP (bit 1) - Monitor coprocessor
        cr0 = (cr0 & !(1 << 2)) | (1 << 1);
        
        core::arch::asm!("mov cr0, {}", in(reg) cr0);
        
        // Read CR4
        let mut cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        
        // Set OSFXSR (bit 9) - OS supports FXSAVE/FXRSTOR
        // Set OSXMMEXCPT (bit 10) - OS supports SIMD exceptions
        cr4 |= (1 << 9) | (1 << 10);
        
        core::arch::asm!("mov cr4, {}", in(reg) cr4);
    }
    
    crate::serial_println!("[SIMD] SSE/SSE2 enabled");
}

/// Enable AVX support (if available)
pub fn enable_avx() -> bool {
    // Check if AVX is supported
    let caps = super::capabilities();
    if caps.map(|c| !c.avx).unwrap_or(true) {
        return false;
    }
    
    unsafe {
        // Read CR4
        let mut cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        
        // Set OSXSAVE (bit 18) - OS supports XSAVE
        cr4 |= 1 << 18;
        core::arch::asm!("mov cr4, {}", in(reg) cr4);
        
        // Set XCR0 to enable AVX state
        // Bit 0: x87 FPU
        // Bit 1: SSE
        // Bit 2: AVX
        let xcr0: u64 = 0b111; // x87 + SSE + AVX
        
        core::arch::asm!(
            "xsetbv",
            in("ecx") 0u32,
            in("eax") (xcr0 & 0xFFFFFFFF) as u32,
            in("edx") (xcr0 >> 32) as u32,
        );
    }
    
    crate::serial_println!("[SIMD] AVX enabled");
    true
}

/// Enable AVX-512 (if available)
pub fn enable_avx512() -> bool {
    let caps = super::capabilities();
    if caps.map(|c| !c.avx512f).unwrap_or(true) {
        return false;
    }
    
    // First enable AVX
    if !enable_avx() {
        return false;
    }
    
    unsafe {
        // Extend XCR0 for AVX-512
        // Bits 5-7: AVX-512 state
        let xcr0: u64 = 0b11100111; // x87 + SSE + AVX + AVX-512
        
        core::arch::asm!(
            "xsetbv",
            in("ecx") 0u32,
            in("eax") (xcr0 & 0xFFFFFFFF) as u32,
            in("edx") (xcr0 >> 32) as u32,
        );
    }
    
    crate::serial_println!("[SIMD] AVX-512 enabled");
    true
}

/// Fast memory copy (basic version - SSE intrinsics disabled for x86_64-unknown-none target)
/// When SSE is enabled in the target, replace with the SSE2 version
#[inline]
pub unsafe fn memcpy_sse2(dst: *mut u8, src: *const u8, len: usize) {
    // Basic copy - compiler may optimize this
    let mut i = 0;
    while i + 8 <= len {
        // Copy 8 bytes at a time (u64)
        let data = (src.add(i) as *const u64).read_unaligned();
        (dst.add(i) as *mut u64).write_unaligned(data);
        i += 8;
    }
    while i < len {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
}

/// Fast memory set (basic version)
#[inline]
pub unsafe fn memset_sse2(dst: *mut u8, value: u8, len: usize) {
    // Create a u64 filled with the byte value
    let fill: u64 = 0x0101010101010101u64 * (value as u64);
    
    let mut i = 0;
    while i + 8 <= len {
        (dst.add(i) as *mut u64).write_unaligned(fill);
        i += 8;
    }
    while i < len {
        *dst.add(i) = value;
        i += 1;
    }
}

/// Fast memory compare (basic version)
#[inline]
pub unsafe fn memcmp_sse2(a: *const u8, b: *const u8, len: usize) -> bool {
    let mut i = 0;
    while i + 8 <= len {
        let va = (a.add(i) as *const u64).read_unaligned();
        let vb = (b.add(i) as *const u64).read_unaligned();
        if va != vb {
            return false;
        }
        i += 8;
    }
    while i < len {
        if *a.add(i) != *b.add(i) {
            return false;
        }
        i += 1;
    }
    true
}

/// XOR blocks (basic version)
#[inline]
pub unsafe fn xor_blocks_sse2(dst: *mut u8, src: *const u8, len: usize) {
    let mut i = 0;
    while i + 8 <= len {
        let a = (dst.add(i) as *mut u64).read_unaligned();
        let b = (src.add(i) as *const u64).read_unaligned();
        (dst.add(i) as *mut u64).write_unaligned(a ^ b);
        i += 8;
    }
    while i < len {
        *dst.add(i) ^= *src.add(i);
        i += 1;
    }
}
