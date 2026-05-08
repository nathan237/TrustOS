

use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

static BGT_: AtomicU64 = AtomicU64::new(0);
static AEX_: AtomicBool = AtomicBool::new(false);
static AXS_: AtomicBool = AtomicBool::new(false);


pub fn init() {
    #[cfg(target_arch = "x86_64")]
    let rdrand = unsafe {
        let ecx: u32;
        
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "pop rbx",
            in("eax") 1u32,
            lateout("eax") _,
            lateout("ecx") ecx,
            lateout("edx") _,
        );
        (ecx >> 30) & 1 == 1
    };
    #[cfg(not(target_arch = "x86_64"))]
    let rdrand = false;
    #[cfg(target_arch = "x86_64")]
    let rdseed = unsafe {
        let ebx_out: u32;
        
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {out:e}, ebx",
            "pop rbx",
            in("eax") 7u32,
            in("ecx") 0u32,
            out = out(reg) ebx_out,
            lateout("eax") _,
            lateout("edx") _,
        );
        (ebx_out >> 18) & 1 == 1
    };
    #[cfg(not(target_arch = "x86_64"))]
    let rdseed = false;
    AEX_.store(rdrand, Ordering::Relaxed);
    AXS_.store(rdseed, Ordering::Relaxed);
    crate::serial_println!("[RNG] RDRAND={} RDSEED={}", rdrand, rdseed);
}

fn onb() -> u64 {
    let gx = crate::logger::eg();
    let fm = crate::rtc::aou();
    let oiw = ((fm.year as u64) << 48)
        ^ ((fm.month as u64) << 40)
        ^ ((fm.day as u64) << 32)
        ^ ((fm.hour as u64) << 24)
        ^ ((fm.minute as u64) << 16)
        ^ ((fm.second as u64) << 8);
    let gqb = crate::arch::timestamp();
    gqb ^ gx ^ oiw ^ 0x9E3779B97F4A7C15
}



pub fn dvf() -> u64 {
    let mut state = BGT_.load(Ordering::Relaxed);
    if state == 0 {
        state = onb();
    }

    
    state ^= state >> 12;
    state ^= state << 25;
    state ^= state >> 27;
    state = state.wrapping_mul(0x2545F4914F6CDD1D);

    BGT_.store(state, Ordering::Relaxed);
    state
}

pub fn hyj(buf: &mut [u8]) {
    let mut i = 0;
    while i < buf.len() {
        let df = dvf().to_le_bytes();
        for &b in &df {
            if i >= buf.len() {
                break;
            }
            buf[i] = b;
            i += 1;
        }
    }
}


pub fn ixv() -> u8 {
    dvf() as u8
}


pub fn qrt() -> u32 {
    dvf() as u32
}





fn ocf() -> Option<u64> {
    #[cfg(not(target_arch = "x86_64"))]
    return None;

    #[cfg(target_arch = "x86_64")]
    {
        if !AEX_.load(Ordering::Relaxed) {
            return None;
        }
        for _ in 0..10 {
            let val: u64;
            let ok: u8;
            unsafe {
                core::arch::asm!(
                    "rdrand {v}",
                    "setc {ok}",
                    v = out(reg) val,
                    ok = out(reg_byte) ok,
                );
            }
            if ok != 0 {
                return Some(val);
            }
        }
        None
    }
}


fn ocg() -> Option<u64> {
    #[cfg(not(target_arch = "x86_64"))]
    return None;

    #[cfg(target_arch = "x86_64")]
    {
        if !AXS_.load(Ordering::Relaxed) {
            return None;
        }
        for _ in 0..10 {
            let val: u64;
            let ok: u8;
            unsafe {
                core::arch::asm!(
                    "rdseed {v}",
                    "setc {ok}",
                    v = out(reg) val,
                    ok = out(reg_byte) ok,
                );
            }
            if ok != 0 {
                return Some(val);
            }
        }
        None
    }
}



pub fn jed() -> u64 {
    if let Some(v) = ocg() {
        return v;
    }
    if let Some(v) = ocf() {
        return v;
    }
    
    let jy = crate::arch::timestamp();
    dvf() ^ jy
}


pub fn omx() -> u32 {
    jed() as u32
}


pub fn jeb(buf: &mut [u8]) {
    let mut i = 0;
    while i < buf.len() {
        let df = jed().to_le_bytes();
        for &b in &df {
            if i >= buf.len() {
                break;
            }
            buf[i] = b;
            i += 1;
        }
    }
}


pub fn mjq() -> bool {
    AEX_.load(Ordering::Relaxed)
}
