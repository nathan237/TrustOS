




use core::sync::atomic::{AtomicU64, Ordering};


static GL_: AtomicU64 = AtomicU64::new(0);


static ZE_: AtomicU64 = AtomicU64::new(0);


pub fn init(we: u64) {
    GL_.store(we, Ordering::Release);
    ZE_.store(ey(), Ordering::Release);
    
    crate::serial_println!("[TSC] Initialized: {} Hz ({} GHz)", 
        we, we / 1_000_000_000);
}


#[inline(always)]
pub fn ey() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}


#[inline(always)]
pub fn odh() -> u64 {
    
    
    unsafe {
        core::arch::asm!("lfence", options(nostack, preserves_flags));
        core::arch::x86_64::_rdtsc()
    }
}


#[inline(always)]
pub fn odi() -> (u64, u32) {
    let mut aux: u32;
    let tsc: u64;
    
    unsafe {
        let lo: u32;
        let hi: u32;
        core::arch::asm!(
            "rdtscp",
            out("eax") lo,
            out("edx") hi,
            out("ecx") aux,
            options(nostack)
        );
        tsc = ((hi as u64) << 32) | (lo as u64);
    }
    
    (tsc, aux)
}


pub fn we() -> u64 {
    GL_.load(Ordering::Acquire)
}


#[inline]
pub fn brn(cycles: u64) -> u64 {
    let freq = GL_.load(Ordering::Relaxed);
    if freq == 0 {
        return 0;
    }
    
    
    let bul = (cycles as u128 * 1_000_000_000u128) / freq as u128;
    bul as u64
}


#[inline]
pub fn fqf(cycles: u64) -> u64 {
    let freq = GL_.load(Ordering::Relaxed);
    if freq == 0 {
        return 0;
    }
    (cycles as u128 * 1_000_000u128 / freq as u128) as u64
}


#[inline]
pub fn fqg(cycles: u64) -> u64 {
    let freq = GL_.load(Ordering::Relaxed);
    if freq == 0 {
        return 0;
    }
    (cycles as u128 * 1_000u128 / freq as u128) as u64
}


pub fn dvi() -> u64 {
    let boot = ZE_.load(Ordering::Relaxed);
    let current = ey();
    let bb = current.saturating_sub(boot);
    brn(bb)
}


pub fn gjt() -> u64 {
    let boot = ZE_.load(Ordering::Relaxed);
    let current = ey();
    let bb = current.saturating_sub(boot);
    fqf(bb)
}


pub fn nlk() -> u64 {
    let boot = ZE_.load(Ordering::Relaxed);
    let current = ey();
    let bb = current.saturating_sub(boot);
    fqg(bb)
}


pub fn dmq(bul: u64) {
    let freq = GL_.load(Ordering::Relaxed);
    if freq == 0 {
        return;
    }
    
    let laz = (bul as u128 * freq as u128 / 1_000_000_000u128) as u64;
    let start = ey();
    let target = start + laz;
    
    while ey() < target {
        core::hint::spin_loop();
    }
}


pub fn ldb(micros: u64) {
    dmq(micros * 1_000);
}


pub fn ww(millis: u64) {
    dmq(millis * 1_000_000);
}



pub fn hq(millis: u64) {
    const CND_: u64 = 1_193_182;
    const CNC_: u16 = 0x42;
    const XR_: u16 = 0x43;
    
    const CIS_: u64 = 50;

    let mut ck = millis;
    while ck > 0 {
        let df = ck.min(CIS_);
        let iva = (CND_ * df / 1000) as u16;
        if iva == 0 { break; }

        unsafe {
            use x86_64::instructions::port::Port;
            let mut chg: Port<u8> = Port::new(XR_);
            let mut cux: Port<u8> = Port::new(CNC_);
            let mut eww: Port<u8> = Port::new(0x61);

            let gsk = eww.read();

            
            eww.write(gsk & !0x03);

            
            chg.write(0b10110000);
            cux.write(0xFF);
            cux.write(0xFF);

            
            eww.write((gsk | 0x01) & !0x02);

            
            for _ in 0..10 {
                let mut llv: Port<u8> = Port::new(0x80);
                llv.write(0);
            }

            
            chg.write(0b10000000);
            let lo = cux.read();
            let hi = cux.read();
            let owd = (hi as u16) << 8 | lo as u16;

            
            loop {
                chg.write(0b10000000);
                let lo = cux.read();
                let hi = cux.read();
                let current = (hi as u16) << 8 | lo as u16;

                if owd.wrapping_sub(current) >= iva {
                    break;
                }
                core::hint::spin_loop();
            }

            eww.write(gsk);
        }
        ck -= df;
    }
}



pub fn hju() -> u64 {
    
    if let Some(freq) = kgy() {
        return freq;
    }
    
    
    kgx()
}


fn kgy() -> Option<u64> {
    let cvu = unsafe { core::arch::x86_64::__cpuid(0x15) };
    
    
    
    if cvu.eax != 0 && cvu.ebx != 0 {
        let kzs = if cvu.ecx != 0 {
            cvu.ecx as u64
        } else {
            
            
            25_000_000u64
        };
        
        let aso = kzs * cvu.ebx as u64 / cvu.eax as u64;
        if aso > 100_000_000 { 
            return Some(aso);
        }
    }
    
    
    let cvt = unsafe { core::arch::x86_64::__cpuid(0) };
    if cvt.eax >= 0x16 {
        let hol = unsafe { core::arch::x86_64::__cpuid(0x16) };
        
        if hol.eax != 0 {
            let dqg = hol.eax as u64;
            return Some(dqg * 1_000_000);
        }
    }
    
    None
}






fn kgx() -> u64 {
    
    let start = ey();
    hq(200); 
    let end = ey();

    let bb = end - start;
    let freq = bb * 5; 

    crate::serial_println!("[TSC] PIT-polling calibration: {} cycles in 200ms → {} MHz",
        bb, freq / 1_000_000);

    freq
}


pub struct Stopwatch {
    start: u64,
}

impl Stopwatch {
    
    #[inline]
    pub fn start() -> Self {
        Self { start: ey() }
    }
    
    
    #[inline]
    pub fn elapsed_nanos(&self) -> u64 {
        let bb = ey() - self.start;
        brn(bb)
    }
    
    
    #[inline]
    pub fn elapsed_micros(&self) -> u64 {
        let bb = ey() - self.start;
        fqf(bb)
    }
    
    
    #[inline]
    pub fn lov(&self) -> u64 {
        let bb = ey() - self.start;
        fqg(bb)
    }
    
    
    #[inline]
    pub fn lou(&self) -> u64 {
        ey() - self.start
    }
    
    
    pub fn mwg(&mut self) -> u64 {
        let cy = ey();
        let bb = cy - self.start;
        self.start = cy;
        brn(bb)
    }
}
