





pub fn ktg() {
    unsafe {
        
        let mut akb: u64;
        core::arch::asm!("mov {}, cr0", bd(reg) akb);
        
        
        
        akb = (akb & !(1 << 2)) | (1 << 1);
        
        core::arch::asm!("mov cr0, {}", in(reg) akb);
        
        
        let mut cr4: u64;
        core::arch::asm!("mov {}, cr4", bd(reg) cr4);
        
        
        
        cr4 |= (1 << 9) | (1 << 10);
        
        core::arch::asm!("mov cr4, {}", in(reg) cr4);
        
        
        
        core::arch::asm!("fninit");
        
        
        
        
        
        let lnd: u32 = 0x1F80;
        core::arch::asm!("ldmxcsr [{}]", in(reg) &lnd, options(nostack));
    }
    
    crate::serial_println!("[SIMD] SSE/SSE2 enabled (FPU init + MXCSR masked)");
}


pub fn ktd() -> bool {
    
    let dr = super::bme();
    if dr.map(|r| !r.dof).unwrap_or(true) {
        return false;
    }
    
    unsafe {
        
        let mut cr4: u64;
        core::arch::asm!("mov {}, cr4", bd(reg) cr4);
        
        
        cr4 |= 1 << 18;
        core::arch::asm!("mov cr4, {}", in(reg) cr4);
        
        
        
        
        
        let ihr: u64 = 0b111; 
        
        core::arch::asm!(
            "xsetbv",
            in("ecx") 0u32,
            in("eax") (ihr & 0xFFFFFFFF) as u32,
            in("edx") (ihr >> 32) as u32,
        );
    }
    
    crate::serial_println!("[SIMD] AVX enabled");
    true
}


pub fn sky() -> bool {
    let dr = super::bme();
    if dr.map(|r| !r.eml).unwrap_or(true) {
        return false;
    }
    
    
    if !ktd() {
        return false;
    }
    
    unsafe {
        
        
        let ihr: u64 = 0b11100111; 
        
        core::arch::asm!(
            "xsetbv",
            in("ecx") 0u32,
            in("eax") (ihr & 0xFFFFFFFF) as u32,
            in("edx") (ihr >> 32) as u32,
        );
    }
    
    crate::serial_println!("[SIMD] AVX-512 enabled");
    true
}



#[inline]
pub unsafe fn umz(cs: *mut u8, cy: *const u8, len: usize) {
    
    let mut a = 0;
    while a + 8 <= len {
        
        let f = (cy.add(a) as *const u64).md();
        (cs.add(a) as *mut u64).qae(f);
        a += 8;
    }
    while a < len {
        *cs.add(a) = *cy.add(a);
        a += 1;
    }
}


#[inline]
pub unsafe fn unb(cs: *mut u8, bn: u8, len: usize) {
    
    let vi: u64 = 0x0101010101010101u64 * (bn as u64);
    
    let mut a = 0;
    while a + 8 <= len {
        (cs.add(a) as *mut u64).qae(vi);
        a += 8;
    }
    while a < len {
        *cs.add(a) = bn;
        a += 1;
    }
}


#[inline]
pub unsafe fn umy(q: *const u8, o: *const u8, len: usize) -> bool {
    let mut a = 0;
    while a + 8 <= len {
        let asf = (q.add(a) as *const u64).md();
        let cci = (o.add(a) as *const u64).md();
        if asf != cci {
            return false;
        }
        a += 8;
    }
    while a < len {
        if *q.add(a) != *o.add(a) {
            return false;
        }
        a += 1;
    }
    true
}


#[inline]
pub unsafe fn xwo(cs: *mut u8, cy: *const u8, len: usize) {
    let mut a = 0;
    while a + 8 <= len {
        let q = (cs.add(a) as *mut u64).md();
        let o = (cy.add(a) as *const u64).md();
        (cs.add(a) as *mut u64).qae(q ^ o);
        a += 8;
    }
    while a < len {
        *cs.add(a) ^= *cy.add(a);
        a += 1;
    }
}
