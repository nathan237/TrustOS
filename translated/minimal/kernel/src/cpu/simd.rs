





pub fn fuo() {
    unsafe {
        
        let mut cr0: u64;
        core::arch::asm!("mov {}, cr0", out(reg) cr0);
        
        
        
        cr0 = (cr0 & !(1 << 2)) | (1 << 1);
        
        core::arch::asm!("mov cr0, {}", in(reg) cr0);
        
        
        let mut cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        
        
        
        cr4 |= (1 << 9) | (1 << 10);
        
        core::arch::asm!("mov cr4, {}", in(reg) cr4);
        
        
        
        core::arch::asm!("fninit");
        
        
        
        
        
        let mxcsr: u32 = 0x1F80;
        core::arch::asm!("ldmxcsr [{}]", in(reg) &mxcsr, options(nostack));
    }
    
    crate::serial_println!("[SIMD] SSE/SSE2 enabled (FPU init + MXCSR masked)");
}


pub fn ful() -> bool {
    
    let caps = super::capabilities();
    if caps.map(|c| !c.avx).unwrap_or(true) {
        return false;
    }
    
    unsafe {
        
        let mut cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        
        
        cr4 |= 1 << 18;
        core::arch::asm!("mov cr4, {}", in(reg) cr4);
        
        
        
        
        
        let xcr0: u64 = 0b111; 
        
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


pub fn lpq() -> bool {
    let caps = super::capabilities();
    if caps.map(|c| !c.avx512f).unwrap_or(true) {
        return false;
    }
    
    
    if !ful() {
        return false;
    }
    
    unsafe {
        
        
        let xcr0: u64 = 0b11100111; 
        
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



#[inline]
pub unsafe fn nef(dst: *mut u8, src: *const u8, len: usize) {
    
    let mut i = 0;
    while i + 8 <= len {
        
        let data = (src.add(i) as *const u64).read_unaligned();
        (dst.add(i) as *mut u64).write_unaligned(data);
        i += 8;
    }
    while i < len {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
}


#[inline]
pub unsafe fn nei(dst: *mut u8, value: u8, len: usize) {
    
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


#[inline]
pub unsafe fn nee(a: *const u8, b: *const u8, len: usize) -> bool {
    let mut i = 0;
    while i + 8 <= len {
        let va = (a.add(i) as *const u64).read_unaligned();
        let apk = (b.add(i) as *const u64).read_unaligned();
        if va != apk {
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


#[inline]
pub unsafe fn pvu(dst: *mut u8, src: *const u8, len: usize) {
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
